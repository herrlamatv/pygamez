//! matchview.rs
//! ============
//! Der Matches-Screen eines Spiels - uebertragen aus `matchview.py`
//! (geoeffnet ueber den Knopf "Matches" im Vorspiel-Screen):
//!
//! - Reiter "Letzte Matches": Verlaufskurve der letzten 30 Ergebnisse
//!   (Rekorde markiert, Tooltip beim Ueberfahren) und die letzten 10 Matches.
//! - Reiter "Alle Matches": alle gespeicherten Matches (`matches::CAP` je
//!   Spiel), Filter nach Modus (Taste M), Sortierung neueste/beste (Taste O),
//!   Bildlauf mit Pfeilen/Mausrad/Bild auf/ab.
//!
//! Ueber beiden Reitern stehen die Summen. Daten: `pgz_core::matches`.
//! Akzentfarbe = Farbe des Spiels, dessen Matches gezeigt werden (Python
//! setzt sie nach `super().__init__`).

use pgz_core::i18n::{t, tf};
use pgz_core::matches::{self, Entry, Totals};
use pgz_core::seedrand;
use pgz_engine::dailyarchive::fit_text;
use pgz_engine::game::{Game, GameBase, GameInfo, InputEvent, Kind};
use pgz_engine::pg::{draw, Color, Font, Rect, RectExt, SurfExt, Surface};
use pgz_engine::shell::{self, Cmd};
use pgz_engine::ui::{self, mix};
use pgz_engine::{charts, py};
use serde_json::{Map, Value};

const ROW_H: i32 = 48;
const TABS: [&str; 2] = ["recent", "all"];
const CHART_N: usize = 30;
const RECENT_N: usize = 10;

/// `mode_label(game_cls, mode)` - Anzeigename eines Modus.
pub fn mode_label(info: &GameInfo, mode: &str) -> String {
    for (key, label_key) in info.modes {
        if *key == mode {
            return t(label_key);
        }
    }
    match mode {
        "single" => t("pregame.single"),
        "multi" => t("pregame.multi"),
        "" => "-".into(),
        m => m.to_string(),
    }
}

/// `fmt_num(value)` - Ganzzahl mit Tausenderpunkten.
pub fn fmt_num(value: i64) -> String {
    let sign = if value < 0 { "-" } else { "" };
    let digits = value.unsigned_abs().to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(c);
    }
    format!("{sign}{out}")
}

/// `fmt_score(entry)` - Chip-Spiele mit Vorzeichen (Gewinn/Verlust).
pub fn fmt_score(e: &Entry) -> String {
    let has_bet = e.info.as_ref().is_some_and(|i| i.contains_key("bet"));
    if has_bet {
        return format!("{}{}", if e.score > 0 { "+" } else { "" }, fmt_num(e.score));
    }
    fmt_num(e.score)
}

/// `int(value)` fuer einen Info-Wert (Zahl oder bool).
fn int_of(v: &Value) -> i64 {
    match v {
        Value::Bool(b) => *b as i64,
        Value::Number(n) => n.as_i64().unwrap_or(0),
        Value::String(s) => s.trim().parse().unwrap_or(0),
        _ => 0,
    }
}

/// `str(value)` wie Python.
fn str_of(v: &Value) -> String {
    match v {
        Value::Bool(true) => "True".into(),
        Value::Bool(false) => "False".into(),
        Value::String(s) => s.clone(),
        Value::Null => "None".into(),
        other => other.to_string(),
    }
}

/// `info_parts(game_cls, info)` - Detail-Texte (Reihenfolge wie `INFO_KEYS`).
pub fn info_parts(info_game: &GameInfo, info: Option<&Map<String, Value>>) -> Vec<String> {
    let mut out = Vec::new();
    let Some(info) = info else { return out };
    if info.is_empty() {
        return out;
    }
    for (key, label_key) in matches::INFO_KEYS {
        let Some(value) = info.get(*key) else { continue };
        let text = pgz_games::matchtext::match_text(info_game.key, key, value).unwrap_or_else(|| match *key {
            "day" => tf(label_key, &[("v", &seedrand::date_of(int_of(value)))]),
            "x" | "cr" => {
                let v = 0.max(int_of(value));
                tf(label_key, &[("v", &format!("{}.{:02}", v / 100, v % 100))])
            }
            "tm" => tf(label_key, &[("v", &matches::format_duration(int_of(value)))]),
            "bet" | "win" | "c" => tf(label_key, &[("v", &fmt_num(int_of(value)))]),
            _ => tf(label_key, &[("v", &str_of(value))]),
        });
        if !text.is_empty() {
            out.push(text);
        }
    }
    out
}

#[derive(Debug, Clone, PartialEq)]
enum Hover {
    Tab(&'static str),
    Filter(Option<String>),
    Sort,
}

/// `matchview.MatchScreen` - Letzte Matches / Alle Matches eines Spiels.
pub struct MatchScreen {
    b: GameBase,
    info: GameInfo,
    key: &'static str,
    /// Rueckweg: Vorspiel-Screen dieses Spiels (`on_close=self._reopen`).
    back: Option<String>,
    tab: &'static str,
    filter: Option<String>,
    sort_best: bool,
    first: usize,
    sel: usize,
    hover: Option<Hover>,
    chart_hits: Vec<(Rect, usize)>,
    chart_chron: Vec<Entry>,
    mouse: Option<(i32, i32)>,
    all: Vec<Entry>,
    tot: Totals,
    modes: Vec<String>,
    items: Vec<Entry>,
    left: i32,
    right: i32,
    bottom: i32,
    tab_font: Font,
    tab_rects: Vec<(Rect, &'static str, String)>,
    sum_rect: Rect,
    chart_rect: Option<Rect>,
    top: i32,
    filter_rects: Vec<(Rect, Option<String>, String)>,
    sort_rect: Option<Rect>,
    chip_font: Font,
    head_y: i32,
    rows_visible: usize,
    row_rects: Vec<Rect>,
}

impl MatchScreen {
    /// `MatchScreen(surface, w, h, app, game_cls, on_close, tab)`
    pub fn new(mut b: GameBase, info: GameInfo, back: Option<String>, tab: &str) -> MatchScreen {
        // Game.__init__ setzt die Akzentfarbe nach dem Klassennamen - hier gilt
        // die Farbe des Spiels, dessen Matches gezeigt werden.
        b.accent = ui::game_color(info.class_name);
        let tab = TABS.iter().copied().find(|x| *x == tab).unwrap_or("recent");
        let mut s = MatchScreen {
            b,
            info,
            key: info.key,
            back,
            tab,
            filter: None,
            sort_best: false,
            first: 0,
            sel: 0,
            hover: None,
            chart_hits: Vec::new(),
            chart_chron: Vec::new(),
            mouse: None,
            all: Vec::new(),
            tot: Totals::default(),
            modes: Vec::new(),
            items: Vec::new(),
            left: 0,
            right: 0,
            bottom: 0,
            tab_font: ui::fb(14),
            tab_rects: Vec::new(),
            sum_rect: Rect::new(0, 0, 0, 0),
            chart_rect: None,
            top: 0,
            filter_rects: Vec::new(),
            sort_rect: None,
            chip_font: ui::fb(12),
            head_y: 0,
            rows_visible: 1,
            row_rects: Vec::new(),
        };
        s.load();
        s.build();
        s
    }

    // ----- Daten & Layout -----------------------------------------------

    fn load(&mut self) {
        self.all = matches::entries(self.key, None);
        self.tot = matches::totals(self.key, None);
        self.modes = matches::modes_of(self.key, None);
        if let Some(f) = &self.filter {
            if !self.modes.contains(f) {
                self.filter = None;
            }
        }
        self.apply_view();
    }

    fn apply_view(&mut self) {
        let mut items: Vec<Entry> = match &self.filter {
            Some(f) => self.all.iter().filter(|e| &e.mode == f).cloned().collect(),
            None => self.all.clone(),
        };
        if self.sort_best {
            // Stabil: bei gleichen Punkten bleibt das neuere vorn.
            items.sort_by_key(|e| -(e.score as i128));
        }
        self.items = items;
        self.sel = self.sel.min(self.items.len().saturating_sub(1));
        self.first = self.first.min(self.items.len().saturating_sub(1));
    }

    fn build(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        self.left = 24.max(py::fdiv32(w, 2) - 340);
        self.right = w - self.left;
        self.bottom = h - 40;
        // Reiter mittig unter dem Titel
        self.tab_font = ui::fb(14);
        let labels: Vec<String> = TABS.iter().map(|k| t(&format!("match.tab_{k}"))).collect();
        let widths: Vec<i32> = labels.iter().map(|lb| self.tab_font.width(lb) + 30).collect();
        let total: i32 = widths.iter().sum::<i32>() + 8 * (widths.len() as i32 - 1);
        let mut x = py::fdiv32(w, 2) - py::fdiv32(total, 2);
        self.tab_rects.clear();
        for ((k, lb), tw) in TABS.iter().zip(labels).zip(widths) {
            self.tab_rects.push((Rect::new(x, 92, tw, 26), *k, lb));
            x += tw + 8;
        }
        // Summen-Leiste
        self.sum_rect = Rect::new(self.left, 126, self.right - self.left, 44);
        let top = self.sum_rect.bottom() + 10;
        if self.tab == "recent" {
            let ch = 64.max(130.min(py::fdiv32(self.bottom - top, 3)));
            let cr = Rect::new(self.left, top, self.right - self.left, ch);
            self.chart_rect = Some(cr);
            self.top = cr.bottom() + 10;
            self.filter_rects.clear();
            self.sort_rect = None;
        } else {
            self.chart_rect = None;
            // Filter-Chips (Alle + jeder Modus) links, Sortierung rechts
            let fnt = ui::fb(12);
            self.chip_font = fnt.clone();
            let sort_lb = if self.sort_best { t("match.sort_best") } else { t("match.sort_new") };
            let sw = fnt.width(&sort_lb) + 24;
            let sr = Rect::new(self.right - sw, top, sw, 24);
            self.sort_rect = Some(sr);
            self.filter_rects.clear();
            let mut x = self.left;
            let limit = sr.x - 8;
            let mut opts: Vec<Option<String>> = vec![None];
            opts.extend(self.modes.iter().cloned().map(Some));
            for mode in opts {
                let lb = match &mode {
                    None => t("match.filter_all"),
                    Some(m) => mode_label(&self.info, m),
                };
                let cw = fnt.width(&lb) + 24;
                if x + cw > limit {
                    break;
                }
                self.filter_rects.push((Rect::new(x, top, cw, 24), mode, lb));
                x += cw + 6;
            }
            self.head_y = top + 24 + 14;
            self.top = top + 24 + 28;
        }
        self.rows_visible = 1.max(py::fdiv32(self.bottom - self.top, ROW_H)) as usize;
        self.row_rects = (0..self.rows_visible as i32)
            .map(|i| Rect::new(self.left, self.top + i * ROW_H, self.right - self.left, ROW_H - 6))
            .collect();
    }

    /// `_rows()` - Eintraege, die im aktiven Reiter als Zeilen erscheinen.
    fn rows(&self) -> &[Entry] {
        if self.tab == "recent" {
            &self.all[..self.all.len().min(RECENT_N)]
        } else {
            &self.items
        }
    }

    // ----- Eingabe ------------------------------------------------------

    fn close(&mut self) {
        match &self.back {
            Some(k) => shell::post(Cmd::PreGame(k.clone())),
            None => shell::post(Cmd::BackToMenu),
        }
    }

    fn switch_tab(&mut self, tab: &'static str) {
        if tab == self.tab {
            return;
        }
        self.tab = tab;
        self.first = 0;
        self.sel = 0;
        self.build();
        self.b.play_sound("move");
    }

    fn cycle_filter(&mut self) {
        let mut options: Vec<Option<String>> = vec![None];
        options.extend(self.modes.iter().cloned().map(Some));
        let i = options.iter().position(|o| *o == self.filter).unwrap_or(0);
        self.filter = options[(i + 1) % options.len()].clone();
        self.first = 0;
        self.sel = 0;
        self.apply_view();
        self.b.play_sound("move");
    }

    fn toggle_sort(&mut self) {
        self.sort_best = !self.sort_best;
        self.first = 0;
        self.sel = 0;
        self.apply_view();
        self.build();
        self.b.play_sound("move");
    }

    fn mv(&mut self, d: i64) {
        let n = self.rows().len() as i64;
        if n == 0 {
            return;
        }
        let sel = 0.max((n - 1).min(self.sel as i64 + d));
        self.sel = sel as usize;
        let rv = self.rows_visible as i64;
        if sel < self.first as i64 {
            self.first = sel as usize;
        } else if sel >= self.first as i64 + rv {
            self.first = (sel - rv + 1) as usize;
        }
        self.b.play_sound("move");
    }

    // ----- Zeichnen -----------------------------------------------------

    fn draw_sums(&self, s: &mut Surface) {
        let p = ui::pal();
        let tot = &self.tot;
        let mut cells: Vec<(String, String)> = vec![(t("match.sum_n"), fmt_num(tot.n))];
        let decided = tot.w + tot.l + tot.d;
        if decided != 0 {
            let pct = py::round(100.0 * tot.w as f64 / decided as f64);
            cells.push((t("match.sum_w"), format!("{} ({} %)", fmt_num(tot.w), pct)));
        }
        if tot.n != 0 {
            cells.push((t("match.sum_best"), fmt_num(tot.best.unwrap_or(0))));
            cells.push((t("match.sum_avg"), fmt_num(py::round(tot.sum as f64 / tot.n as f64))));
            cells.push((t("match.sum_time"), matches::format_duration(tot.time)));
        }
        let r = self.sum_rect;
        let gap = 8;
        let n = cells.len() as i32;
        let cw = py::fdiv32(r.w - gap * (n - 1), n);
        let lab_f = ui::f(11);
        let val_f = ui::fb(16);
        for (i, (lab, val)) in cells.iter().enumerate() {
            let cell = Rect::new(r.x + i as i32 * (cw + gap), r.y, cw, r.h);
            ui::draw_panel_ex(s, cell, Some(p.panel), None, Some(10), false, None);
            let li = lab_f.render(lab, true, p.text_faint);
            s.blr(&li, li.get_rect().midtop_at((cell.centerx(), cell.y + 5)));
            let vi = val_f.render(val, true, if i == 0 { self.b.accent } else { p.text });
            s.blr(&vi, vi.get_rect().midbottom_at((cell.centerx(), cell.bottom() - 5)));
        }
    }

    fn draw_chart(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let r = self.chart_rect.expect("Reiter recent");
        ui::draw_panel_ex(s, r, Some(p.panel), None, Some(10), false, None);
        let chron: Vec<Entry> = self.all.iter().take(CHART_N).rev().cloned().collect();
        let lab = ui::fb(11).render(&tf("match.chart", &[("n", &chron.len().to_string())]), true, p.text_faint);
        s.bl(&lab, (r.x + 10, r.y + 6));
        // Index unter der Maus aus den Treffern des letzten Bilds - vor dem Leeren
        let hover = self.chart_index();
        self.chart_hits.clear();
        if chron.len() < 2 {
            let hint = ui::f(12).render(&t("match.chart_few"), true, p.text_faint);
            s.blr(&hint, hint.get_rect().center_at(r.center()));
            return;
        }
        let inner = Rect::new(r.x + 10, r.y + 34, r.w - 20, r.h - 42);
        let marks: Vec<usize> =
            chron.iter().enumerate().filter(|(_, e)| e.flags & matches::F_RECORD != 0).map(|(i, _)| i).collect();
        let values: Vec<f64> = chron.iter().map(|e| e.score as f64).collect();
        self.chart_chron = chron;
        let fmt = |v: f64| fmt_num(py::round(v));
        self.chart_hits = charts::line(s, inner, &values, Some(self.b.accent), &marks, Some(&fmt), hover);
    }

    /// `_chart_index()` - Kurvenpunkt unter der Maus (Treffer des letzten Bilds).
    fn chart_index(&self) -> Option<usize> {
        let m = self.mouse?;
        self.chart_hits.iter().find(|(rect, _)| rect.collidepoint(m)).map(|(_, idx)| *idx)
    }

    fn draw_chart_tip(&self, s: &mut Surface) {
        let Some(idx) = self.chart_index() else { return };
        if idx >= self.chart_chron.len() {
            return;
        }
        let e = &self.chart_chron[idx];
        let lines = vec![fmt_score(e), e.stamp.clone(), mode_label(&self.info, &e.mode)];
        charts::tooltip(s, self.mouse.expect("Maus"), &lines, None);
    }

    fn draw_filters(&self, s: &mut Surface) {
        let p = ui::pal();
        for (r, mode, lb) in &self.filter_rects {
            let active = *mode == self.filter;
            let hover = self.hover == Some(Hover::Filter(mode.clone()));
            draw::rect_r(s, if active || hover { p.panel_light } else { p.panel }, *r, 0, 12);
            draw::rect_r(s, if active { self.b.accent } else { p.border }, *r, 1, 12);
            let img = self.chip_font.render(lb, true, if active { p.text } else { p.text_dim });
            s.blr(&img, img.get_rect().center_at(r.center()));
        }
        let r = self.sort_rect.expect("Reiter all");
        let hover = self.hover == Some(Hover::Sort);
        draw::rect_r(s, if hover { p.panel_light } else { p.panel }, r, 0, 12);
        draw::rect_r(s, if hover { p.accent2 } else { p.border }, r, 1, 12);
        let lb = if self.sort_best { t("match.sort_best") } else { t("match.sort_new") };
        let img = self.chip_font.render(&lb, true, p.text);
        s.blr(&img, img.get_rect().center_at(r.center()));
        let cnt = ui::fb(12).render(
            &tf(
                "match.stored",
                &[
                    ("n", &fmt_num(self.tot.stored)),
                    ("total", &fmt_num(self.tot.n)),
                    ("max", &matches::CAP.to_string()),
                ],
            ),
            true,
            p.text_dim,
        );
        s.blr(&cnt, cnt.get_rect().midleft_at((self.left, self.head_y)));
    }

    fn badge(&self, s: &mut Surface, cx: i32, cy: i32, result: Option<i64>, record: bool) {
        let p = ui::pal();
        let col = match result {
            Some(matches::WIN) => p.green,
            Some(matches::LOSS) => p.red,
            Some(matches::DRAW) => p.gold,
            _ => mix(p.btn, self.b.accent, 0.35),
        };
        draw::circle(s, col, (cx, cy), 13, 0);
        let dark: Color = mix(col, (0, 0, 0), 0.55);
        match result {
            Some(matches::WIN) => {
                draw::lines(s, dark, false, &[(cx - 6, cy), (cx - 2, cy + 5), (cx + 6, cy - 5)], 3);
            }
            Some(matches::LOSS) => {
                draw::line(s, dark, (cx - 5, cy - 5), (cx + 5, cy + 5), 3);
                draw::line(s, dark, (cx - 5, cy + 5), (cx + 5, cy - 5), 3);
            }
            Some(matches::DRAW) => {
                draw::line(s, dark, (cx - 5, cy - 3), (cx + 5, cy - 3), 3);
                draw::line(s, dark, (cx - 5, cy + 3), (cx + 5, cy + 3), 3);
            }
            _ => {
                draw::circle(s, p.text_dim, (cx, cy), 3, 0);
            }
        }
        if record {
            draw::circle(s, p.gold, (cx, cy), 16, 2);
        }
    }

    fn draw_row(&self, s: &mut Surface, rect: Rect, e: &Entry, selected: bool) {
        let p = ui::pal();
        let flags = e.flags;
        let result = e.result_i();
        let col = if selected { p.panel_light } else { p.panel };
        ui::draw_panel_ex(s, rect, Some(col), None, Some(10), false, None);
        if selected {
            draw::rect_r(s, self.b.accent, rect, 1, 10);
        }
        draw::rect_r(s, self.b.accent, (rect.x, rect.y + 8, 3, rect.h - 16), 0, 2);
        self.badge(s, rect.x + 28, rect.centery(), result, flags & matches::F_RECORD != 0);

        let title = format!("{}  \u{b7}  {}", fmt_score(e), mode_label(&self.info, &e.mode));
        let tf15 = ui::fb(15);
        let title = fit_text(&tf15, &title, rect.w - 52 - 90);
        let img = tf15.render(&title, true, if selected { p.text } else { p.text_dim });
        s.bl(&img, (rect.x + 52, rect.y + 5));

        let mut parts: Vec<String> = Vec::new();
        if let Some(r) = result {
            parts.push(t(match r {
                matches::WIN => "match.win",
                matches::LOSS => "match.loss",
                _ => "match.draw",
            }));
        }
        if flags & matches::F_RECORD != 0 {
            parts.push(t("match.record"));
        }
        if flags & matches::F_ABORT != 0 {
            parts.push(t("match.aborted"));
        }
        if flags & matches::F_ARCHIVE != 0 {
            parts.push(t("match.archive"));
        } else if flags & matches::F_DAILY != 0 {
            parts.push(t("match.daily"));
        }
        parts.extend(info_parts(&self.info, e.info.as_ref()));
        let fnt = ui::f(11);
        let sub = fit_text(&fnt, &parts.join("  \u{b7}  "), rect.w - 52 - 150);
        let img = fnt.render(&sub, true, if flags & matches::F_RECORD != 0 { p.gold } else { p.text_faint });
        s.bl(&img, (rect.x + 52, rect.y + 25));

        let di = ui::fb(13).render(&matches::format_duration(e.dur), true, self.b.accent);
        s.blr(&di, di.get_rect().topright_at((rect.right() - 14, rect.y + 6)));
        let si = ui::f(11).render(&e.stamp, true, p.text_faint);
        s.blr(&si, si.get_rect().topright_at((rect.right() - 14, rect.y + 25)));
    }

    fn draw_empty(&self, s: &mut Surface) {
        let p = ui::pal();
        let cx = py::fdiv32(self.b.width, 2);
        let cy = py::fdiv32(self.top + self.bottom, 2);
        let head = ui::fb(17).render(&tf("match.empty", &[("game", &self.info.name())]), true, p.text_dim);
        s.blr(&head, head.get_rect().center_at((cx, cy - 12)));
        let hint = ui::f(13).render(&t("match.empty_hint"), true, p.text_faint);
        s.blr(&hint, hint.get_rect().center_at((cx, cy + 14)));
    }

    fn draw_scrollbar(&self, s: &mut Surface, n: usize) {
        let p = ui::pal();
        let (top, bottom) = (self.top, self.top + self.rows_visible as i32 * ROW_H);
        let track = Rect::new(self.right + 6, top, 4, bottom - top);
        draw::rect_r(s, p.panel, track, 0, 2);
        let frac = self.rows_visible as f64 / n as f64;
        let h = 24.max((track.h as f64 * frac) as i32);
        let pos = self.first as f64 / (1.max(n as i64 - self.rows_visible as i64)) as f64;
        let y = top + ((track.h - h) as f64 * 1.0f64.min(pos)) as i32;
        draw::rect_r(s, p.border_light, (track.x, y, 4, h), 0, 2);
    }
}

impl Game for MatchScreen {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }
    fn reset(&mut self) {}
    fn update(&mut self, _dt: f64) {}
    fn screen_name(&self) -> String {
        t("match.name")
    }
    fn on_surface_changed(&mut self) {
        self.build();
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        match ev.kind {
            Kind::KeyDown => {
                let k = ev.key.as_str();
                let rv = self.rows_visible as i64;
                if k == "Escape" {
                    self.close();
                } else if ["Tab", "ISO_Left_Tab", "Left", "Right"].contains(&k)
                    || self.b.is_action(k, "left", None)
                    || self.b.is_action(k, "right", None)
                {
                    self.switch_tab(if self.tab == "recent" { "all" } else { "recent" });
                } else if k == "Up" || self.b.is_action(k, "up", None) {
                    self.mv(-1);
                } else if k == "Down" || self.b.is_action(k, "down", None) {
                    self.mv(1);
                } else if k == "Prior" {
                    self.mv(-rv);
                } else if k == "Next" {
                    self.mv(rv);
                } else if k == "Home" {
                    let n = self.rows().len() as i64;
                    self.mv(-n);
                } else if k == "End" {
                    let n = self.rows().len() as i64;
                    self.mv(n);
                } else if (k == "m" || k == "M") && self.tab == "all" {
                    self.cycle_filter();
                } else if (k == "o" || k == "O") && self.tab == "all" {
                    self.toggle_sort();
                }
            }
            Kind::MouseMove => {
                self.mouse = Some(ev.pos);
                self.hover = None;
                for (r, k, _) in &self.tab_rects {
                    if r.collidepoint(ev.pos) {
                        self.hover = Some(Hover::Tab(*k));
                    }
                }
                for (r, mode, _) in &self.filter_rects {
                    if r.collidepoint(ev.pos) {
                        self.hover = Some(Hover::Filter(mode.clone()));
                    }
                }
                if self.sort_rect.is_some_and(|r| r.collidepoint(ev.pos)) {
                    self.hover = Some(Hover::Sort);
                }
                let n = self.rows().len();
                for (i, r) in self.row_rects.iter().enumerate() {
                    if r.collidepoint(ev.pos) && self.first + i < n {
                        self.sel = self.first + i;
                    }
                }
            }
            Kind::MouseDown if ev.button == 1 => {
                if let Some(k) = self.tab_rects.iter().find(|(r, _, _)| r.collidepoint(ev.pos)).map(|(_, k, _)| *k) {
                    self.switch_tab(k);
                    return;
                }
                if let Some(mode) =
                    self.filter_rects.iter().find(|(r, _, _)| r.collidepoint(ev.pos)).map(|(_, m, _)| m.clone())
                {
                    if mode != self.filter {
                        self.filter = mode;
                        self.first = 0;
                        self.sel = 0;
                        self.apply_view();
                        self.b.play_sound("move");
                    }
                    return;
                }
                if self.sort_rect.is_some_and(|r| r.collidepoint(ev.pos)) {
                    self.toggle_sort();
                }
            }
            Kind::Wheel => {
                self.mv(if ev.delta > 0 { -1 } else { 1 });
            }
            _ => {}
        }
    }

    fn draw(&mut self) {
        let p = ui::pal();
        let mut s = self.b.sf();
        let (w, h) = (self.b.width, self.b.height);
        ui::draw_background_ex(&mut s, w, h, false, None);
        ui::draw_title_ex(&mut s, w, &self.info.name(), Some(&t("match.subtitle")), 26, Some(&ui::fb(30)), None, Some(self.b.accent));
        for (r, k, lb) in &self.tab_rects {
            let active = *k == self.tab;
            let hover = self.hover == Some(Hover::Tab(*k));
            draw::rect_r(&mut s, if active || hover { p.panel_light } else { p.panel }, *r, 0, 13);
            draw::rect_r(&mut s, if active { self.b.accent } else { p.border }, *r, 1, 13);
            let img = self.tab_font.render(lb, true, if active { p.text } else { p.text_dim });
            s.blr(&img, img.get_rect().center_at(r.center()));
        }
        self.draw_sums(&mut s);
        if self.tab == "recent" {
            self.draw_chart(&mut s);
        } else {
            self.draw_filters(&mut s);
        }
        let rows: Vec<Entry> = self.rows().to_vec();
        if rows.is_empty() {
            self.draw_empty(&mut s);
        } else {
            for (i, r) in self.row_rects.iter().enumerate() {
                let idx = self.first + i;
                if idx >= rows.len() {
                    break;
                }
                self.draw_row(&mut s, *r, &rows[idx], idx == self.sel);
            }
            if rows.len() > self.rows_visible {
                self.draw_scrollbar(&mut s, rows.len());
            }
        }
        if self.tab == "recent" && !self.chart_hits.is_empty() && self.mouse.is_some() {
            self.draw_chart_tip(&mut s);
        }
        ui::draw_footer(&mut s, w, h, &t(&format!("match.hint_{}", self.tab)));
    }
}
