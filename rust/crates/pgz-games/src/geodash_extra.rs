//! geodash_extra.rs
//! ================
//! Der Reiter **Weitere** von Geometry Dash: die Level-Bibliothek -
//! uebertragen aus `games/geodash_extra.py`.
//!
//! Oben eine Kategorie-Leiste, darunter ein Suchfeld, darunter die Liste.
//! Jedes Level laesst sich spielen oder als Vorlage in die eigenen Level
//! kopieren. Fortschritt steht unter dem Schluessel `x:<id>`.

use pgz_engine::prelude::*;
use pgz_engine::ui::TextInput;
use serde_json::{json, Map, Value};

use crate::geodash_core as core;
use crate::geodash_draw as gdraw;
use crate::geodash_edit::{best_get, btn, count_get, draw_list_toast, kurz, new_level, Best, Counts, GdAction, GAME};
use crate::ugc;

/// Reihenfolge der Kategorie-Knoepfe.
pub const CATS: [&str; 11] = ["all", "starter", "rhythm", "coins", "ship", "ufo", "ball", "gravity", "speed", "wave", "demon"];
const BUTTONS: [&str; 3] = ["play", "template", "random"];

fn s_of<'a>(m: &'a Map<String, Value>, key: &str) -> &'a str {
    m.get(key).and_then(|v| v.as_str()).unwrap_or("")
}

/// `DIFF_KEY(diff)`
pub fn diff_key(diff: i64) -> &'static str {
    crate::geodash::DIFF_KEYS[diff.clamp(0, 5) as usize]
}

fn i_of(m: &Map<String, Value>, key: &str) -> i64 {
    match m.get(key) {
        Some(Value::Number(n)) => n.as_i64().unwrap_or_else(|| n.as_f64().unwrap_or(0.0) as i64),
        Some(Value::String(s)) => s.trim().parse().unwrap_or(0),
        Some(Value::Bool(b)) => *b as i64,
        _ => 0,
    }
}

/// Die Level-Bibliothek: filtern, suchen, spielen, als Vorlage kopieren.
pub struct ExtraList {
    levels: Vec<Map<String, Value>>,
    view: Vec<usize>,
    /// Kategorie.
    pub cat: &'static str,
    query: TextInput,
    /// Suchfeld hat den Fokus.
    pub search_focus: bool,
    /// Auswahl (Index in `view`).
    pub sel: usize,
    first: usize,
    toast: String,
    toast_t: f64,
    hay: Vec<String>,
    fnt: Font,
    tiny: Font,
    two_lines: bool,
    row_h: i32,
    /// Kategorie-Knoepfe.
    pub cat_rects: Vec<(&'static str, Rect)>,
    /// Suchfeld.
    pub search_rect: Rect,
    /// Knoepfe unten.
    pub btn_rects: Vec<(&'static str, Rect)>,
    list_top: i32,
    list_bottom: i32,
    rows_visible: usize,
    list_w: i32,
    width: i32,
}

impl ExtraList {
    /// `ExtraList(game)`
    pub fn new(b: &GameBase, tab_bottom: i32) -> ExtraList {
        let z = Rect::new(0, 0, 0, 0);
        let mut x = ExtraList {
            levels: Vec::new(),
            view: Vec::new(),
            cat: "all",
            query: TextInput::new("", 20, None, &t("gd.more.search")),
            search_focus: false,
            sel: 0,
            first: 0,
            toast: String::new(),
            toast_t: 0.0,
            hay: Vec::new(),
            fnt: b.font.clone(),
            tiny: b.font.clone(),
            two_lines: true,
            row_h: 26,
            cat_rects: Vec::new(),
            search_rect: z,
            btn_rects: Vec::new(),
            list_top: 0,
            list_bottom: 0,
            rows_visible: 1,
            list_w: 0,
            width: b.width,
        };
        x.reload();
        x.layout(b, tab_bottom);
        let cat = b.opt_value("geodash", "last_cat").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_else(|| "all".into());
        if let Some(c) = CATS.iter().find(|c| **c == cat) {
            x.cat = c;
        }
        x.filter();
        x
    }

    // ----- Daten --------------------------------------------------------

    /// `reload()`
    pub fn reload(&mut self) {
        if self.levels.is_empty() {
            self.levels = crate::geodash::load_extra_levels();
            self.hay = self.levels.iter().map(|d| format!("{} {} {}", s_of(d, "name"), s_of(d, "id"), s_of(d, "category")).to_lowercase()).collect();
        }
        self.filter();
    }

    fn filter(&mut self) {
        let q = self.query.text.trim().to_lowercase();
        self.view.clear();
        for (i, d) in self.levels.iter().enumerate() {
            if self.cat != "all" && s_of(d, "category") != self.cat {
                continue;
            }
            if !q.is_empty() && !self.hay[i].contains(&q) {
                continue;
            }
            self.view.push(i);
        }
        self.sel = self.sel.min(self.view.len().saturating_sub(1));
        self.clamp();
    }

    fn selected(&self) -> Option<&Map<String, Value>> {
        self.view.get(self.sel).map(|&i| &self.levels[i])
    }

    /// `has_next()`
    pub fn has_next(&self) -> bool {
        !self.view.is_empty() && self.sel + 1 < self.view.len()
    }

    /// `play_next()`
    pub fn play_next(&mut self, b: &mut GameBase) -> GdAction {
        if self.has_next() {
            self.sel += 1;
            self.clamp();
            return self.action(b, "play");
        }
        GdAction::None
    }

    /// Wie viele Level der Bibliothek sind schon geschafft?
    pub fn done_count(&self, best: &Best) -> usize {
        self.levels.iter().filter(|d| best_get(best, &format!("x:{}", s_of(d, "id")))[0] >= 100).count()
    }

    // ----- Layout -------------------------------------------------------

    /// `layout()`
    pub fn layout(&mut self, b: &GameBase, tab_bottom: i32) {
        let (w, h) = (b.width, b.height);
        self.width = w;
        self.fnt = ui::font(13.max(h / 30), false, false);
        self.tiny = ui::font(10.max(h / 40), false, false);
        let (fh, th) = (self.fnt.get_height(), self.tiny.get_height());
        self.two_lines = h >= 400;
        self.row_h = if self.two_lines { fh + th + 10 } else { 26.max(fh + 12) };
        let cols = if w >= 720 { 6 } else { 4 };
        let ch = 20.max(28.min(h / 18)).max(th + 8);
        let gap = 5;
        let cw = (w - 24 - gap * (cols - 1)) as f64 / cols as f64;
        let rows_c = (CATS.len() as i32 + cols - 1) / cols;
        self.cat_rects.clear();
        let top = tab_bottom + 6;
        for (i, key) in CATS.iter().enumerate() {
            let (r, c) = (i as i32 / cols, i as i32 % cols);
            self.cat_rects.push((key, Rect::new((12.0 + c as f64 * (cw + gap as f64)) as i32, top + r * (ch + gap), cw as i32, ch)));
        }
        let cat_bottom = top + rows_c * (ch + gap);
        let ff = 22.max(30.min(h / 15)).max(th + 10);
        self.search_rect = Rect::new(12, cat_bottom, w - 24, ff);
        let bh = 24.max(34.min(h / 13)).max(th + 12);
        let bw = (w - 24 - gap * (BUTTONS.len() as i32 - 1)) as f64 / BUTTONS.len() as f64;
        let bottom = h - 12;
        self.btn_rects = BUTTONS.iter().enumerate().map(|(i, k)| (*k, Rect::new((12.0 + i as f64 * (bw + gap as f64)) as i32, bottom - bh, bw as i32, bh))).collect();
        self.list_top = self.search_rect.bottom() + 6;
        self.list_bottom = bottom - bh - th - 8;
        self.rows_visible = 1.max(py::fdiv32(self.list_bottom - self.list_top - 4, self.row_h)) as usize;
        self.list_w = w - 30;
        self.clamp();
    }

    fn clamp(&mut self) {
        let n = self.view.len();
        let vis = self.rows_visible;
        self.first = self.first.min(n.saturating_sub(vis));
        if self.sel < self.first {
            self.first = self.sel;
        } else if self.sel >= self.first + vis {
            self.first = self.sel + 1 - vis;
        }
    }

    fn toast_msg(&mut self, text: String) {
        self.toast = text;
        self.toast_t = 2.8;
    }

    /// `update(dt)`
    pub fn update(&mut self, dt: f64) {
        if self.toast_t > 0.0 {
            self.toast_t -= dt;
            if self.toast_t <= 0.0 {
                self.toast.clear();
            }
        }
    }

    // ----- Eingabe ------------------------------------------------------

    /// `handle(event)`
    pub fn handle(&mut self, b: &mut GameBase, ev: &InputEvent) -> GdAction {
        match ev.kind {
            Kind::KeyDown => {
                let k = ev.key.as_str();
                if self.search_focus {
                    if k == "Return" || k == "KP_Enter" {
                        self.search_focus = false;
                        return GdAction::None;
                    }
                    if k == "Escape" {
                        self.query.set_text("");
                        self.search_focus = false;
                        self.filter();
                        return GdAction::None;
                    }
                    if self.query.handle(ev) {
                        self.filter();
                        return GdAction::None;
                    }
                }
                if k == "Up" || k == "Left" || b.is_action(k, "up", None) {
                    self.mv(b, -1);
                } else if k == "Down" || k == "Right" || b.is_action(k, "down", None) {
                    self.mv(b, 1);
                } else if k == "Prior" {
                    self.mv(b, -(self.rows_visible as i64));
                } else if k == "Next" {
                    self.mv(b, self.rows_visible as i64);
                } else if k == "Return" || k == "space" || k == "KP_Enter" {
                    return self.action(b, "play");
                } else if k == "t" || k == "T" {
                    return self.action(b, "template");
                } else if k == "r" || k == "R" {
                    return self.action(b, "random");
                } else if k == "slash" || k == "f" || k == "F" {
                    self.search_focus = true;
                } else if k == "c" || k == "C" {
                    self.step_cat(b, 1);
                }
            }
            Kind::Wheel => {
                let maxf = self.view.len().saturating_sub(self.rows_visible) as i64;
                self.first = 0.max(maxf.min(self.first as i64 - ev.delta as i64)) as usize;
            }
            Kind::MouseDown => {
                let pos = ev.pos;
                if let Some(&(key, _)) = self.cat_rects.iter().find(|(_, rc)| rc.hit(pos)) {
                    self.set_cat(b, key);
                    return GdAction::None;
                }
                if self.search_rect.hit(pos) {
                    self.search_focus = true;
                    return GdAction::None;
                }
                self.search_focus = false;
                if let Some(&(key, _)) = self.btn_rects.iter().find(|(_, rc)| rc.hit(pos)) {
                    return self.action(b, key);
                }
                for i in 0..self.rows_visible {
                    let idx = self.first + i;
                    if idx >= self.view.len() {
                        break;
                    }
                    if self.row_rect(i).hit(pos) {
                        if self.sel == idx {
                            return self.action(b, "play");
                        }
                        self.sel = idx;
                        b.play_sound("move");
                        return GdAction::None;
                    }
                }
            }
            _ => {}
        }
        GdAction::None
    }

    fn row_rect(&self, i: usize) -> Rect {
        Rect::new(12, self.list_top + i as i32 * self.row_h, self.list_w, self.row_h - 3)
    }

    fn mv(&mut self, b: &GameBase, d: i64) {
        if self.view.is_empty() {
            return;
        }
        self.sel = 0.max((self.view.len() as i64 - 1).min(self.sel as i64 + d)) as usize;
        self.clamp();
        b.play_sound("move");
    }

    fn set_cat(&mut self, b: &mut GameBase, key: &'static str) {
        if key == self.cat {
            return;
        }
        self.cat = key;
        self.sel = 0;
        self.first = 0;
        self.filter();
        b.set_opt("geodash", "last_cat", json!(key));
        b.play_sound("click");
    }

    fn step_cat(&mut self, b: &mut GameBase, d: i64) {
        let i = CATS.iter().position(|c| *c == self.cat).unwrap_or(0) as i64;
        let n = CATS.len() as i64;
        self.set_cat(b, CATS[(i + d).rem_euclid(n) as usize]);
    }

    fn action(&mut self, b: &mut GameBase, key: &str) -> GdAction {
        if key == "random" {
            if !self.view.is_empty() {
                self.sel = py::randrange(0, self.view.len() as i64) as usize;
                self.clamp();
                return self.action(b, "play");
            }
            return GdAction::None;
        }
        let Some(d) = self.selected().cloned() else {
            return GdAction::None;
        };
        if key == "play" {
            b.play_sound("click");
            return GdAction::PlayExtra(d);
        }
        if key == "template" {
            return self.copy_as_template(b, &d);
        }
        GdAction::None
    }

    fn copy_as_template(&mut self, b: &GameBase, d: &Map<String, Value>) -> GdAction {
        if ugc::is_full(GAME) {
            self.toast_msg(tf("gd.ugc.err.full", &[("max", &ugc::max_items(GAME).to_string())]));
            return GdAction::None;
        }
        let mut m = new_level(s_of(d, "name"), "", None, None);
        let sl = ugc::slug(if s_of(d, "id").is_empty() { "level" } else { s_of(d, "id") });
        m.insert("id".into(), json!(ugc::unique_id(&sl, GAME, None)));
        m.insert("objects".into(), d.get("objects").cloned().unwrap_or(json!([])));
        for key in ["speed", "mode", "bg", "ground", "music", "length"] {
            if let Some(v) = d.get(key) {
                m.insert(key.into(), v.clone());
            }
        }
        m.insert("from".into(), json!(s_of(d, "id")));
        let (ok, _why) = ugc::save_map(&m, GAME);
        if !ok {
            self.toast_msg(tf("gd.ugc.err.full", &[("max", &ugc::max_items(GAME).to_string())]));
            return GdAction::None;
        }
        b.play_sound("click");
        GdAction::EditCopied(m)
    }

    // ----- Zeichnen -----------------------------------------------------

    /// `draw(s)`
    pub fn draw(&self, s: &mut Surface, b: &GameBase, best: &Best, coins: &Counts) {
        let p = ui::pal();
        for &(key, rc) in &self.cat_rects {
            btn(s, rc, &t(&format!("gd.more.cat.{key}")), &self.tiny, key == self.cat, None, true);
        }
        self.query.draw(s, self.search_rect, &self.fnt, self.search_focus, false);
        if self.view.is_empty() {
            self.draw_empty(s, b);
        } else {
            for i in 0..self.rows_visible {
                let idx = self.first + i;
                if idx >= self.view.len() {
                    break;
                }
                self.draw_row(s, b, best, coins, &self.levels[self.view[idx]], idx, self.row_rect(i));
            }
            if self.view.len() > self.rows_visible {
                self.draw_scrollbar(s);
            }
        }
        let nl = self.levels.len().to_string();
        let line = format!(
            "{}   \u{00B7}   {}",
            tf("gd.more.count", &[("n", &self.view.len().to_string()), ("max", &nl)]),
            tf("gd.more.done", &[("n", &self.done_count(best).to_string()), ("max", &nl)])
        );
        let cnt = self.tiny.render(&line, true, p.text_dim);
        s.bl(&cnt, (12, self.list_bottom + 2));
        let has = self.selected().is_some();
        for &(key, rc) in &self.btn_rects {
            btn(s, rc, &t(&format!("gd.more.btn_{key}")), &self.tiny, false, None, has);
        }
        if !self.toast.is_empty() {
            draw_list_toast(s, &self.tiny, &self.toast, self.width, self.list_bottom);
        }
    }

    fn draw_empty(&self, s: &mut Surface, b: &GameBase) {
        let p = ui::pal();
        let bx = Rect::new(24, self.list_top + 10, b.width - 48, 50.max(self.list_bottom - self.list_top - 24));
        ui::draw_panel_ex(s, bx, None, None, None, false, None);
        let mut img = self.fnt.render(&t("gd.more.empty"), true, p.text_dim);
        if img.w() > bx.w - 16 {
            img = self.tiny.render(&t("gd.more.empty"), true, p.text_dim);
        }
        s.blr(&img, img.get_rect().center_at(bx.center()));
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_row(&self, s: &mut Surface, b: &GameBase, best: &Best, coins: &Counts, d: &Map<String, Value>, idx: usize, rc: Rect) {
        let p = ui::pal();
        let sel = idx == self.sel;
        draw::rect_r(s, if sel { p.panel_light } else { p.panel }, rc, 0, 7);
        draw::rect_r(s, if sel { b.accent } else { p.border }, rc, if sel { 2 } else { 1 }, 7);
        let pad = 10;
        let diff = i_of(d, "difficulty");
        let col = gdraw::DIFF_COLORS[diff.clamp(0, gdraw::DIFF_COLORS.len() as i64 - 1) as usize];
        draw::rect_r(s, col, (rc.x + 5, rc.y + 5, 5, rc.h - 10), 0, 2);
        let x = rc.x + pad + 6;
        let name = self.fnt.render(s_of(d, "name"), true, p.text);
        s.bl(&name, (x, rc.y + 3));
        let key = format!("x:{}", s_of(d, "id"));
        let bst = best_get(best, &key)[0];
        let mask = count_get(coins, &key);
        let cat = d.get("category").and_then(|v| v.as_str()).unwrap_or("all");
        let mut sub = format!(
            "{}  \u{00B7}  {}  \u{00B7}  {}",
            t(&format!("gd.more.cat.{cat}")),
            t(&format!("gd.diff.{}", diff_key(diff))),
            tf("gd.ugc.length", &[("n", &core::level_length(d).to_string())])
        );
        sub += &format!("  \u{00B7}  {}", tf("gd.coins_of", &[("n", &(mask & 7).count_ones().to_string()), ("max", "3")]));
        let mut img = self.tiny.render(&sub, true, p.text_dim);
        if img.w() > rc.w - 90 {
            img = self.tiny.render(&kurz(&self.tiny, &sub, rc.w - 90, 4, "..."), true, p.text_dim);
        }
        if self.two_lines {
            s.bl(&img, (x, rc.bottom() - img.h() - 3));
        }
        let pct = self.fnt.render(&format!("{bst}%"), true, if bst >= 100 { rgb(185, 242, 58) } else { p.text_dim });
        s.blr(&pct, pct.get_rect().midright_at((rc.right() - pad, rc.centery())));
    }

    fn draw_scrollbar(&self, s: &mut Surface) {
        let p = ui::pal();
        let track = Rect::new(self.list_w + 14, self.list_top, 4, self.rows_visible as i32 * self.row_h);
        draw::rect_r(s, p.panel, track, 0, 2);
        let frac = self.rows_visible as f64 / self.view.len() as f64;
        let hh = 24.max((track.h as f64 * frac) as i32);
        let pos = self.first as f64 / 1.max(self.view.len() as i64 - self.rows_visible as i64) as f64;
        let y = track.y + ((track.h - hh) as f64 * 1f64.min(pos)) as i32;
        draw::rect_r(s, p.border_light, (track.x, y, 4, hh), 0, 2);
    }
}
