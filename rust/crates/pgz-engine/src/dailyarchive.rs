//! dailyarchive.rs
//! ===============
//! Gegenstueck zu `dailyarchive.py` - das Tages-Archiv der Tagesmodi
//! (Crossy Road: Tagesstrecke, Sudoku: Tagesraetsel, Wordle: Tageswort).
//!
//! Nachspielbar sind die letzten [`WINDOW`] Tage (heute eingeschlossen) und
//! jeder aeltere Tag, der schon gespielt wurde. Gespeichert wird in der
//! Section des Spiels als Liste von Zeilen, deren erstes Feld die
//! Tagesnummer ist (`seedrand::day_index`, Tage seit 2026-01-01).
//!
//! [`Picker`] ist das Auswahlfenster (Kalender + Leiste der aelteren Tage),
//! `draw_day_row` die Datums-Leiste `[<] [Heute . 26.11.] [>] [Archiv]` im
//! Setup. Zeile fuer Zeile aus Python (gleiche Masse, Farben, Reihenfolge).

use std::collections::BTreeMap;

use pgz_core::i18n::{t, tf};
use pgz_core::seedrand;
use serde_json::Value;

use crate::game::{GameBase, InputEvent, Kind};
use crate::pg::{self, draw, Color, Font, Rect, RectExt, SurfExt, Surface};
use crate::{charts, py, ui};

/// Tage im Kalender (heute eingeschlossen).
pub const WINDOW: i64 = 30;
/// Hoechstens so viele Tage merkt sich ein Archiv.
pub const MAX_DAYS: usize = 2000;

// ---------------------------------------------------------------------------
//  Tage & Daten
// ---------------------------------------------------------------------------

/// `today_index()`
pub fn today_index() -> i64 {
    seedrand::day_index(None)
}

/// `window_days(today)` - die letzten WINDOW Tage, aelteste zuerst.
pub fn window_days(today: i64) -> Vec<i64> {
    (today - WINDOW + 1..today + 1).collect()
}

/// `available(played, today)` - alle waehlbaren Tage (Fenster + aeltere
/// gespielte), aelteste zuerst.
pub fn available(played: impl IntoIterator<Item = i64>, today: Option<i64>) -> Vec<i64> {
    let today = today.unwrap_or_else(today_index);
    let mut days: Vec<i64> = window_days(today);
    for d in played {
        if 0 <= d && d <= today && !days.contains(&d) {
            days.push(d);
        }
    }
    days.sort_unstable();
    days
}

/// `clean_rows(raw, width)` - Archiv-Zeilen pruefen: `[tag, ...]` mit genau
/// `width` Feldern, nach Tag sortiert, doppelte Tage: die letzte gewinnt.
pub fn clean_rows(raw: Option<&Value>, width: usize) -> Vec<Vec<Value>> {
    let Some(Value::Array(raw)) = raw else { return Vec::new() };
    let mut rows: BTreeMap<i64, Vec<Value>> = BTreeMap::new();
    for row in raw {
        let Value::Array(row) = row else { continue };
        if row.len() != width {
            continue;
        }
        let Some(day) = row[0].as_i64() else { continue };
        if day < 0 {
            continue;
        }
        rows.insert(day, row.clone());
    }
    let out: Vec<Vec<Value>> = rows.into_values().collect();
    let skip = out.len().saturating_sub(MAX_DAYS);
    out.into_iter().skip(skip).collect()
}

/// Eine Archiv-Zeile: erstes Feld ist die Tagesnummer.
pub trait DayRow {
    /// `row[0]`
    fn day(&self) -> i64;
}

impl DayRow for [i64; 3] {
    fn day(&self) -> i64 {
        self[0]
    }
}

/// `find(rows, day)` - Index der Zeile eines Tages.
pub fn find<R: DayRow>(rows: &[R], day: i64) -> Option<usize> {
    rows.iter().position(|r| r.day() == day)
}

/// `upsert(rows, row)` - Zeile eintragen/ersetzen (Liste bleibt sortiert).
pub fn upsert<R: DayRow>(rows: &mut Vec<R>, row: R) {
    let day = row.day();
    for i in 0..rows.len() {
        if rows[i].day() == day {
            rows[i] = row;
            return;
        }
        if rows[i].day() > day {
            rows.insert(i, row);
            return;
        }
    }
    rows.push(row);
    if rows.len() > MAX_DAYS {
        let weg = rows.len() - MAX_DAYS;
        rows.drain(..weg);
    }
}

/// `date_str(day)`
pub fn date_str(day: i64) -> String {
    seedrand::date_of(day)
}

/// (Jahr, Monat, Tag, Wochentag Montag=0) einer Tagesnummer.
fn civil(day: i64) -> (i64, i64, i64, i64) {
    let epoch = seedrand::days_from_civil(2026, 1, 1) + day;
    let (y, m, d, _, _, _) = pgz_core::store::civil_from_unix(epoch * 86_400);
    // 1970-01-01 war ein Donnerstag (weekday 3)
    (y, m as i64, d as i64, (epoch + 3).rem_euclid(7))
}

/// `weekday()` einer Tagesnummer (Montag = 0).
pub fn weekday(day: i64) -> i64 {
    civil(day).3
}

/// `fit_text(fnt, text, room)` - kuerzt mit '...', bis der Text passt.
pub fn fit_text(fnt: &Font, text: &str, room: i32) -> String {
    if fnt.width(text) <= room {
        return text.to_string();
    }
    let mut chars: Vec<char> = text.chars().collect();
    while !chars.is_empty() && fnt.width(&(chars.iter().collect::<String>() + "\u{2026}")) > room {
        chars.pop();
    }
    if chars.is_empty() {
        String::new()
    } else {
        chars.iter().collect::<String>().trim_end().to_string() + "\u{2026}"
    }
}

/// `chip_date(day)` - '23.11.25'.
pub fn chip_date(day: i64) -> String {
    let (y, m, d, _) = civil(day);
    format!("{:02}.{:02}.{:02}", d, m, y % 100)
}

/// `short_date(day)` - '26.11.'.
pub fn short_date(day: i64) -> String {
    let (_, m, d, _) = civil(day);
    format!("{d:02}.{m:02}.")
}

/// `long_date(day)` - 'Mi, 26.11.2026' (Wochentag aus den Sprachdateien).
pub fn long_date(day: i64) -> String {
    let (y, m, d, wd) = civil(day);
    let names = charts::weekday_names();
    format!("{}, {:02}.{:02}.{:04}", names[wd as usize], d, m, y)
}

/// `day_label(day, today)` - 'Heute . 26.11.' / 'Gestern . 25.11.' / langes Datum.
pub fn day_label(day: i64, today: Option<i64>) -> String {
    let today = today.unwrap_or_else(today_index);
    if day == today {
        return tf("arch.today", &[("date", &short_date(day))]);
    }
    if day == today - 1 {
        return tf("arch.yesterday", &[("date", &short_date(day))]);
    }
    long_date(day)
}

/// `seed_hex(game, day)` - Seed des Tages als 8 Hex-Zeichen.
pub fn seed_hex(game: &str, day: i64) -> String {
    format!("{:08X}", seedrand::daily_seed(game, Some(&date_str(day))))
}

// ---------------------------------------------------------------------------
//  Datums-Leiste im Setup
// ---------------------------------------------------------------------------

/// `day_row_rects(x, y, w, h)` - (zurueck, Datum, vor, Archiv).
pub fn day_row_rects(x: i32, y: i32, w: i32, h: i32) -> [Rect; 4] {
    let gap = 6;
    let arrow = h.min(18.max(py::fdiv32(w, 12)));
    let arch = if w < 300 { h } else { 90.max(py::fdiv32(w, 4)) };
    let mid = w - 2 * arrow - arch - 3 * gap;
    let r_prev = Rect::new(x, y, arrow, h);
    let r_date = Rect::new(r_prev.right() + gap, y, mid, h);
    let r_next = Rect::new(r_date.right() + gap, y, arrow, h);
    let r_arch = Rect::new(r_next.right() + gap, y, x + w - (r_next.right() + gap), h);
    [r_prev, r_date, r_next, r_arch]
}

/// `draw_triangle(s, rect, direction, color)`
pub fn draw_triangle(s: &mut Surface, rect: Rect, direction: i32, color: Color) {
    let (cx, cy) = rect.center();
    let k = 4.max(py::fdiv32(rect.h, 5));
    let pts = if direction < 0 {
        [(cx + py::fdiv32(k, 2), cy - k), (cx + py::fdiv32(k, 2), cy + k), (cx - k, cy)]
    } else {
        [(cx - py::fdiv32(k, 2), cy - k), (cx - py::fdiv32(k, 2), cy + k), (cx + k, cy)]
    };
    draw::polygon(s, color, &pts, 0);
}

/// `draw_day_row(s, rects, day, accent, can_prev, can_next, fnt, archived)` -
/// die Datums-Leiste (Pfeile ausgegraut, wenn es nicht weitergeht).
#[allow(clippy::too_many_arguments)]
pub fn draw_day_row(
    s: &mut Surface,
    rects: &[Rect; 4],
    day: i64,
    accent: Color,
    can_prev: bool,
    can_next: bool,
    fnt: &Font,
    archived: bool,
) {
    let p = ui::pal();
    let [r_prev, r_date, r_next, r_arch] = *rects;
    let mut label = day_label(day, None);
    if fnt.width(&label) > r_date.w - 10 {
        label = short_date(day);
    }
    for (rc, d, on) in [(r_prev, -1, can_prev), (r_next, 1, can_next)] {
        draw::rect_r(s, p.btn, rc, 0, 8);
        draw::rect_r(s, p.border, rc, 1, 8);
        draw_triangle(s, rc, d, if on { p.text } else { p.text_faint });
    }
    draw::rect_r(s, if archived { p.btn_sel } else { p.btn }, r_date, 0, 8);
    draw::rect_r(s, if archived { p.gold } else { accent }, r_date, 1, 8);
    let img = fnt.render(&fit_text(fnt, &label, r_date.w - 10), true, if archived { p.gold } else { p.text });
    s.blr(&img, img.get_rect().center_at(r_date.center()));
    draw::rect_r(s, p.btn, r_arch, 0, 8);
    draw::rect_r(s, accent, r_arch, 1, 8);
    // Kalender-Symbol + Text (bei schmalem Knopf nur das Symbol)
    let side = 10.max(py::fdiv32(r_arch.h, 2));
    let mut ic = Rect::new(0, 0, side, side);
    let lab = fnt.render(&t("arch.button"), true, p.text);
    let mut total = ic.w + 6 + lab.w();
    let show_lab = total <= r_arch.w - 10;
    if !show_lab {
        total = ic.w;
    }
    ic.set_midleft((r_arch.centerx() - py::fdiv32(total, 2), r_arch.centery()));
    draw::rect_r(s, p.text_dim, ic, 1, 2);
    let ly = ic.y + py::fdiv32(ic.h, 3);
    draw::line(s, p.text_dim, (ic.x, ly), (ic.right() - 1, ly), 1);
    if show_lab {
        s.blr(&lab, lab.get_rect().midleft_at((ic.right() + 6, r_arch.centery())));
    }
}

// ---------------------------------------------------------------------------
//  Auswahlfenster
// ---------------------------------------------------------------------------

/// Antwort von [`Picker::handle_event`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wahl {
    /// `("pick", tag)`
    Pick(i64),
    /// `("close", None)`
    Close,
}

/// `dailyarchive.Picker` - Kalender der letzten WINDOW Tage + aeltere
/// gespielte Tage. `info`: {tag: (zustand, wert_text)} - zustand ist
/// "solved", "played", "started" (fehlt = offen).
#[derive(Debug, Clone)]
pub struct Picker {
    game_key: String,
    accent: Color,
    today: i64,
    info: BTreeMap<i64, (String, String)>,
    days: Vec<i64>,
    window: Vec<i64>,
    older: Vec<i64>,
    /// Gewaehlter Tag.
    pub sel: i64,
    hover: Option<i64>,
    /// Fensterrahmen (None = beim naechsten Zeichnen neu auslegen).
    pub rect: Option<Rect>,
    head_y: i32,
    cells: Vec<(Rect, Option<i64>)>,
    grid_x: i32,
    cell_w: i32,
    chips: Vec<(Rect, i64)>,
    older_y: i32,
}

impl Picker {
    /// `Picker(game_key, accent, sel, info)`
    pub fn new(game_key: &str, accent: Color, sel: i64, info: BTreeMap<i64, (String, String)>) -> Picker {
        let today = today_index();
        let days = available(info.keys().copied(), Some(today));
        let window = window_days(today);
        let older: Vec<i64> = days.iter().copied().filter(|d| *d < window[0]).collect();
        let sel = if days.contains(&sel) { sel } else { today };
        Picker {
            game_key: game_key.to_string(),
            accent,
            today,
            info,
            days,
            window,
            older,
            sel,
            hover: None,
            rect: None,
            head_y: 0,
            cells: Vec::new(),
            grid_x: 0,
            cell_w: 0,
            chips: Vec::new(),
            older_y: 0,
        }
    }

    /// `layout(W, H)`
    pub fn layout(&mut self, w: i32, h: i32) {
        let pw = 640.min(w - 32);
        let ph = (h - 32).min(470);
        let r = Rect::new(py::fdiv32(w, 2) - py::fdiv32(pw, 2), py::fdiv32(h, 2) - py::fdiv32(ph, 2), pw, ph);
        self.rect = Some(r);
        let top = r.y + 64;
        let older_h = if self.older.is_empty() { 0 } else { 54 };
        let foot = 54;
        let grid_h = r.bottom() - foot - older_h - top - 22;
        // Wochen-Raster: Montag der Woche des ersten Fenstertags
        let start = self.window[0] - weekday(self.window[0]);
        let weeks = py::fdiv(self.today - start, 7) as i32 + 1;
        let cw = py::fdiv32(r.w - 32 - 6 * 6, 7);
        let ch = 26.max(54.min(py::fdiv32(grid_h - (weeks - 1) * 5, 1.max(weeks))));
        let gx = r.x + py::fdiv32(r.w - (7 * cw + 6 * 6), 2);
        self.head_y = top;
        self.cells.clear();
        for w_i in 0..weeks {
            for d_i in 0..7 {
                let day = start + (w_i * 7 + d_i) as i64;
                let rc = Rect::new(gx + d_i * (cw + 6), top + 18 + w_i * (ch + 5), cw, ch);
                self.cells.push((rc, if self.window.contains(&day) { Some(day) } else { None }));
            }
        }
        self.grid_x = gx;
        self.cell_w = cw;
        // Leiste der aelteren gespielten Tage (neueste zuerst)
        self.chips.clear();
        if !self.older.is_empty() {
            let y = top + 18 + weeks * (ch + 5) + 22;
            let mut x = r.x + 16;
            let fnt = ui::fb(11);
            for &day in self.older.iter().rev() {
                let cw = fnt.width(&chip_date(day)) + 18;
                if x + cw > r.right() - 16 {
                    break;
                }
                self.chips.push((Rect::new(x, y, cw, 24), day));
                x += cw + 6;
            }
            self.older_y = y - 14;
        }
    }

    fn step(&mut self, d: i64) {
        let i = self.days.iter().position(|x| *x == self.sel).unwrap_or(0) as i64;
        let j = 0.max((self.days.len() as i64 - 1).min(i + d));
        self.sel = self.days[j as usize];
    }

    /// `handle_event(event, game)` - `Some(Wahl)` oder `None`.
    pub fn handle_event(&mut self, ev: &InputEvent, game: Option<&GameBase>) -> Option<Wahl> {
        let act = |k: &str, a: &str| game.map(|g| g.is_action(k, a, None)).unwrap_or(false);
        match ev.kind {
            Kind::KeyDown => {
                let k = ev.key.as_str();
                if k == "Escape" {
                    return Some(Wahl::Close);
                }
                if ["Return", "space", "KP_Enter"].contains(&k) {
                    return Some(Wahl::Pick(self.sel));
                }
                if k == "Left" || act(k, "left") {
                    self.step(-1);
                } else if k == "Right" || act(k, "right") {
                    self.step(1);
                } else if k == "Up" || act(k, "up") {
                    if self.days.contains(&(self.sel - 7)) {
                        self.sel -= 7;
                    } else {
                        self.step(-1);
                    }
                } else if k == "Down" || act(k, "down") {
                    if self.days.contains(&(self.sel + 7)) {
                        self.sel += 7;
                    } else {
                        self.step(1);
                    }
                } else if k == "Home" {
                    self.sel = self.days[0];
                } else if k == "End" {
                    self.sel = self.today;
                }
                return None;
            }
            Kind::MouseMove => {
                self.hover = self.hit(ev.pos);
                if let Some(h) = self.hover {
                    self.sel = h;
                }
            }
            Kind::MouseDown if ev.button == 1 => {
                if let Some(day) = self.hit(ev.pos) {
                    return Some(Wahl::Pick(day));
                }
                if let Some(r) = self.rect {
                    if !r.collidepoint(ev.pos) {
                        return Some(Wahl::Close);
                    }
                }
            }
            _ => {}
        }
        None
    }

    fn hit(&self, pos: (i32, i32)) -> Option<i64> {
        for (rc, day) in &self.cells {
            if let Some(d) = day {
                if rc.collidepoint(pos) {
                    return Some(*d);
                }
            }
        }
        for (rc, day) in &self.chips {
            if rc.collidepoint(pos) {
                return Some(*day);
            }
        }
        None
    }

    fn state_color(&self, state: Option<&str>) -> Color {
        let p = ui::pal();
        match state {
            Some("solved") => p.green,
            Some("played") => p.gold,
            Some("started") => self.accent,
            _ => p.text_faint,
        }
    }

    fn info_of(&self, day: i64) -> (Option<String>, String) {
        match self.info.get(&day) {
            Some((st, v)) => (Some(st.clone()).filter(|s| !s.is_empty()), v.clone()),
            None => (None, String::new()),
        }
    }

    /// `draw(s)`
    pub fn draw(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (s.w(), s.h());
        if self.rect.is_none() {
            self.layout(w, h);
        }
        let mut shade = pg::surface_alpha(w, h);
        shade.fill_c(Color::rgba(0, 0, 0, 150));
        s.bl(&shade, (0, 0));
        let r = self.rect.expect("ausgelegt");
        ui::draw_panel_ex(s, r, None, None, Some(14), true, None);
        draw::rect_r(s, self.accent, r, 1, 14);
        let title = ui::fb(20).render(&t("arch.title"), true, p.text);
        s.blr(&title, title.get_rect().midtop_at((r.centerx(), r.y + 12)));
        let sub = ui::f(12).render(&tf("arch.sub", &[("n", &WINDOW.to_string())]), true, p.text_dim);
        s.blr(&sub, sub.get_rect().midtop_at((r.centerx(), r.y + 40)));
        // Wochentage
        let names = charts::weekday_names();
        let hf = ui::fb(11);
        for (i, name) in names.iter().enumerate().take(7) {
            let img = hf.render(name, true, p.text_faint);
            let cx = self.grid_x + i as i32 * (self.cell_w + 6) + py::fdiv32(self.cell_w, 2);
            s.blr(&img, img.get_rect().midtop_at((cx, self.head_y)));
        }
        let nf = ui::fb(14);
        let vf = ui::f(10);
        for (rc, day) in self.cells.clone() {
            let Some(day) = day else { continue };
            let (state, value) = self.info_of(day);
            let sel = day == self.sel;
            draw::rect_r(s, if sel { p.btn_sel } else { p.btn }, rc, 0, 7);
            let border = if sel {
                self.accent
            } else if day == self.today {
                p.gold
            } else {
                p.border
            };
            draw::rect_r(s, border, rc, if sel || day == self.today { 2 } else { 1 }, 7);
            let num = civil(day).2;
            let img = nf.render(&num.to_string(), true, if state.is_some() { p.text } else { p.text_dim });
            s.bl(&img, (rc.x + 6, rc.y + 3));
            if state.is_some() {
                draw::circle(s, self.state_color(state.as_deref()), (rc.right() - 8, rc.y + 9), 4, 0);
            }
            if !value.is_empty() && rc.h >= 34 {
                let vi = vf.render(&fit_text(&vf, &value, rc.w - 8), true, self.state_color(state.as_deref()));
                s.blr(&vi, vi.get_rect().bottomleft_at((rc.x + 5, rc.bottom() - 3)));
            }
        }
        if !self.older.is_empty() {
            let lab = ui::fb(11).render(&tf("arch.older", &[("n", &self.older.len().to_string())]), true, p.text_faint);
            s.bl(&lab, (r.x + 16, self.older_y));
            let cf = ui::fb(11);
            for (rc, day) in self.chips.clone() {
                let (state, _) = self.info_of(day);
                let sel = day == self.sel;
                draw::rect_r(s, if sel { p.btn_sel } else { p.btn }, rc, 0, 12);
                let bc = if sel { self.accent } else { self.state_color(state.as_deref()) };
                draw::rect_r(s, bc, rc, 1, 12);
                let img = cf.render(&chip_date(day), true, p.text);
                s.blr(&img, img.get_rect().center_at(rc.center()));
            }
        }
        // Detailzeile des gewaehlten Tags + Hinweis
        let (state, value) = self.info_of(self.sel);
        let mut parts = vec![
            day_label(self.sel, Some(self.today)),
            tf("arch.seed", &[("s", &seed_hex(&self.game_key, self.sel))]),
        ];
        parts.push(t(&format!("arch.state.{}", state.as_deref().unwrap_or("open"))));
        if !value.is_empty() {
            parts.push(value);
        }
        let df = ui::fb(13);
        let col = if state.is_some() { self.state_color(state.as_deref()) } else { p.text };
        let det = df.render(&fit_text(&df, &parts.join("  \u{b7}  "), r.w - 24), true, col);
        s.blr(&det, det.get_rect().midbottom_at((r.centerx(), r.bottom() - 30)));
        let hint = ui::f(11).render(&t("arch.hint"), true, p.text_faint);
        s.blr(&hint, hint.get_rect().midbottom_at((r.centerx(), r.bottom() - 10)));
    }
}
