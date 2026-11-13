//! menu.rs
//! =======
//! Gegenstueck zu `menu.py`: der Vorspiel-Bildschirm, die Optionen (drei
//! Reiter samt Design-Karten mit Vorschau), der Willkommens-Bildschirm des
//! ersten Starts und die Sprachwahl.
//!
//! Zeile fuer Zeile aus Python. Statt `self.app.launch_game(...)` legen die
//! Bildschirme einen [`Cmd`] ab (siehe `pgz_engine::shell`).

use pgz_core::i18n::{self, t, tf};
use pgz_core::settings as smod;
use pgz_engine::game::{Game, GameBase, GameInfo, InputEvent, Kind};
use pgz_engine::pg::{self, draw, Color, Rect, RectExt, SurfExt, BLEND_RGBA_MIN, BLEND_RGB_ADD};
use pgz_engine::shell::{self, Cmd};
use pgz_engine::{py, ui};

/// `menu.pretty_key(k)` - lesbarer Tastenname.
pub fn pretty_key(k: &str) -> String {
    if k.is_empty() {
        return t("key.none");
    }
    let m = match k {
        "space" => Some("key.space"),
        "Return" => Some("key.return"),
        "Up" => Some("key.up"),
        "Down" => Some("key.down"),
        "Left" => Some("key.left"),
        "Right" => Some("key.right"),
        "Escape" => Some("key.escape"),
        "Tab" => Some("key.tab"),
        _ => None,
    };
    if let Some(key) = m {
        return t(key);
    }
    if k.chars().count() == 1 {
        k.to_uppercase()
    } else {
        k.to_string()
    }
}

/// `menu.action_label(action)`
pub fn action_label(action: &str) -> String {
    t(&format!("action.{action}"))
}

// ===========================================================================
//  Vorspiel-Bildschirm
// ===========================================================================

#[derive(Clone)]
enum Was {
    Start(String),
    Options,
    Wiki,
    Back,
}

/// `menu.PreGameScreen`
pub struct PreGameScreen {
    b: GameBase,
    info: GameInfo,
    best: i64,
    buttons: Vec<(String, Was)>,
    rects: Vec<Rect>,
    row_start: usize,
    sel: usize,
}

const UTIL: usize = 3;

impl PreGameScreen {
    /// Baut den Bildschirm fuer ein Spiel.
    pub fn new(mut b: GameBase, info: GameInfo) -> PreGameScreen {
        b.accent = ui::game_color(info.class_name);
        let best = pgz_core::highscore::get(info.key);
        let mut s = PreGameScreen { b, info, best, buttons: Vec::new(), rects: Vec::new(), row_start: 0, sel: 0 };
        s.build_buttons();
        s
    }

    fn build_buttons(&mut self) {
        self.buttons.clear();
        if !self.info.modes.is_empty() {
            for (mode, label) in self.info.modes {
                self.buttons.push((t(label), Was::Start(mode.to_string())));
            }
        } else {
            self.buttons.push((t("pregame.single"), Was::Start("single".into())));
            if self.info.supports_multiplayer {
                self.buttons.push((t("pregame.multi"), Was::Start("multi".into())));
            }
        }
        self.buttons.push((t("pregame.options"), Was::Options));
        self.buttons.push((t("pregame.lamawiki"), Was::Wiki));
        self.buttons.push((t("pregame.back"), Was::Back));
        self.layout_buttons();
    }

    fn layout_buttons(&mut self) {
        self.rects.clear();
        self.row_start = self.buttons.len();
        let n = self.buttons.len() as i32;
        let n_modes = n - UTIL as i32;
        let (w, h) = (self.b.width, self.b.height);
        let (top, bottom) = (132, h - 34);
        let avail = bottom - top;
        let mut bw = 300.min(w - 40);
        let mut x = w / 2 - bw / 2;
        for (bh, gap) in [(40, 10), (32, 6)] {
            let total = n * (bh + gap) - gap;
            if total <= avail {
                let mut y0 = top.max(h / 2 - total / 2 + 6);
                y0 = y0.min(bottom - total);
                for i in 0..n {
                    self.rects.push(Rect::new(x, y0 + i * (bh + gap), bw, bh));
                }
                return;
            }
        }
        bw = 420.min(w - 40);
        x = w / 2 - bw / 2;
        let (mut bh, mut gap, mut total) = (32, 6, 0);
        for (b_, g_) in [(32, 6), (28, 5), (24, 4)] {
            bh = b_;
            gap = g_;
            total = n_modes * (bh + gap) + bh;
            if total <= avail || bh == 24 {
                break;
            }
        }
        let y0 = top.max((h / 2 - total / 2 + 6).min(bottom - total));
        for i in 0..n_modes {
            self.rects.push(Rect::new(x, y0 + i * (bh + gap), bw, bh));
        }
        let row_y = y0 + n_modes * (bh + gap);
        let cell = py::fdiv32(bw - gap * (UTIL as i32 - 1), UTIL as i32);
        for j in 0..UTIL as i32 {
            self.rects.push(Rect::new(x + j * (cell + gap), row_y, cell, bh));
        }
        self.row_start = n_modes as usize;
    }

    fn button_font(&self, i: usize) -> pg::Font {
        if i < self.row_start {
            return ui::f(19);
        }
        let label = &self.buttons[i].0;
        let room = self.rects[i].w - 18;
        for size in [17, 15, 14, 13, 12, 11] {
            let f = ui::f(size);
            if f.width(label) <= room {
                return f;
            }
        }
        ui::f(11)
    }

    fn start(&self, mode: &str) {
        if mode == "multi" && pgz_core::profiles::count() > 1 {
            shell::post(Cmd::PlayerPick { key: self.info.key.into(), mode: mode.into() });
            return;
        }
        shell::post(Cmd::Launch { key: self.info.key.into(), mode: mode.into(), p2: None });
    }

    fn activate(&mut self, i: usize) {
        self.b.play_sound("click");
        let r = self.rects[i];
        ui::spawn_burst(r.centerx() as f64, r.centery() as f64, Some(self.b.accent), 18, 260.0);
        match self.buttons[i].1.clone() {
            Was::Start(m) => self.start(&m),
            Was::Options => shell::post(Cmd::Options(Some(self.info.key.into()))),
            Was::Wiki => shell::post(Cmd::Wiki(Some(format!("game:{}", self.info.class_name)), Some(self.info.key.into()))),
            Was::Back => shell::post(Cmd::BackToMenu),
        }
    }
}

impl Game for PreGameScreen {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }
    fn reset(&mut self) {}
    fn update(&mut self, _dt: f64) {}
    fn screen_name(&self) -> String {
        t("pregame.name")
    }
    fn on_surface_changed(&mut self) {
        self.layout_buttons();
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        let n = self.buttons.len();
        match ev.kind {
            Kind::KeyDown => {
                let k = ev.key.as_str();
                let links = self.b.is_action(k, "left", None) || k == "Left";
                let rechts = self.b.is_action(k, "right", None) || k == "Right";
                if k == "Escape" {
                    shell::post(Cmd::BackToMenu);
                } else if self.b.is_action(k, "up", None) || k == "Up" {
                    if self.sel >= self.row_start && self.row_start > 0 {
                        self.sel = self.row_start - 1;
                    } else {
                        self.sel = py::pmod(self.sel as i64 - 1, n as i64) as usize;
                    }
                    self.b.play_sound("move");
                } else if self.b.is_action(k, "down", None) || k == "Down" {
                    self.sel = (self.sel + 1) % n;
                    self.b.play_sound("move");
                } else if self.row_start < n && self.sel >= self.row_start && (links || rechts) {
                    let step = if links { -1 } else { 1 };
                    let kk = py::pmod(self.sel as i64 - self.row_start as i64 + step, UTIL as i64);
                    self.sel = self.row_start + kk as usize;
                    self.b.play_sound("move");
                } else if k == "Return" || k == "space" {
                    self.activate(self.sel);
                }
            }
            Kind::MouseMove => {
                for (i, r) in self.rects.iter().enumerate() {
                    if r.hit(ev.pos) {
                        self.sel = i;
                    }
                }
            }
            Kind::MouseDown => {
                if let Some(i) = self.rects.iter().position(|r| r.hit(ev.pos)) {
                    self.activate(i);
                }
            }
            _ => {}
        }
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        let (w, h) = (self.b.width, self.b.height);
        ui::draw_background(&mut s, w, h);
        ui::draw_title_ex(&mut s, w, &self.info.name(), Some(&t("pregame.mode")), 64, None, None, Some(self.b.accent));
        for i in 0..self.buttons.len() {
            let f = self.button_font(i);
            ui::draw_button_ex(&mut s, self.rects[i], &self.buttons[i].0, &f, i == self.sel, None, None, Some(self.b.accent));
        }
        if self.best > 0 {
            let img = ui::fb(14).render(&tf("app.highscore", &[("hs", &self.best.to_string())]), true, ui::gold());
            let dot_w = 12;
            let (bw, bh) = (img.w() + 26 + dot_w, img.h() + 10);
            let chip = Rect::new(w / 2 - bw / 2, h - 66 - bh / 2, bw, bh);
            if self.rects.last().map(|r| r.bottom() + 8 < chip.top()).unwrap_or(true) {
                ui::draw_panel_ex(&mut s, chip, None, None, Some(bh / 2), false, None);
                draw::circle(&mut s, ui::gold(), (chip.x + 14, chip.centery()), 3, 0);
                s.blr(&img, img.get_rect().midleft_at((chip.x + 14 + dot_w / 2 + 4, chip.centery())));
            }
        }
        ui::draw_footer(&mut s, w, h, &t("pregame.hint"));
    }
}

// ===========================================================================
//  Optionen
// ===========================================================================

#[derive(Debug, Clone, PartialEq)]
enum ItemKind {
    Toggle { key: String, sect: Option<String>, default: bool },
    Volume,
    Resolution,
    Fps,
    Language,
    Preset,
    Bind { player: String, action: String },
    Theme(String),
    Button,
}

#[derive(Debug, Clone)]
struct Item {
    kind: ItemKind,
    rect: Rect,
    label: String,
    dec: Option<Rect>,
    inc: Option<Rect>,
    bar: Option<Rect>,
}

/// `menu.OptionsScreen`
pub struct OptionsScreen {
    b: GameBase,
    zurueck: Cmd,
    tab: &'static str,
    tab_hover: Option<&'static str>,
    capture: Option<(String, String)>,
    preset_idx: usize,
    res_idx: usize,
    fps_idx: usize,
    items: Vec<Item>,
    panels: Vec<(Rect, Option<String>)>,
    tab_rects: Vec<(Rect, &'static str)>,
    left_x: i32,
    sel: usize,
}

const TABS: [&str; 3] = ["general", "controls", "appearance"];

fn fps_index(fps: i32) -> usize {
    if let Some(i) = smod::FPS_OPTIONS.iter().position(|f| *f == fps) {
        return i;
    }
    (0..smod::FPS_OPTIONS.len()).min_by_key(|i| (smod::FPS_OPTIONS[*i] - fps).abs()).unwrap_or(3)
}

impl OptionsScreen {
    /// `OptionsScreen(..., on_close)` - `zurueck` ist der Weg beim Schliessen.
    pub fn new(b: GameBase, zurueck: Cmd) -> OptionsScreen {
        let st = shell::settings();
        let mut s = OptionsScreen {
            b,
            zurueck,
            tab: "general",
            tab_hover: None,
            capture: None,
            preset_idx: 0,
            res_idx: st.resolution_index(),
            fps_idx: fps_index(st.fps),
            items: Vec::new(),
            panels: Vec::new(),
            tab_rects: Vec::new(),
            left_x: 40,
            sel: 0,
        };
        s.build_items();
        s
    }

    fn font(&self) -> pg::Font {
        if ui::is_modern() {
            ui::f(17)
        } else {
            ui::fm(18)
        }
    }

    fn build_items(&mut self) {
        self.items.clear();
        self.panels.clear();
        let (w, h) = (self.b.width, self.b.height);
        let font = self.font();
        self.left_x = 40;
        let right_x = w / 2 + 10;
        let col_w = 300.min(w / 2 - 50);
        let tab_font = ui::fb(14);
        self.tab_rects.clear();
        let mut tx = self.left_x;
        for key in TABS {
            let tw = tab_font.width(&t(&format!("options.tab_{key}"))) + 26;
            self.tab_rects.push((Rect::new(tx, 48, tw, 26), key));
            tx += tw + 8;
        }
        let compact = h < 420;
        let mut items: Vec<Item> = Vec::new();
        let mut panels: Vec<(Rect, Option<String>)> = Vec::new();
        fn add(items: &mut Vec<Item>, kind: ItemKind, rect: Rect, label: String) -> usize {
            items.push(Item { kind, rect, label, dec: None, inc: None, bar: None });
            items.len() - 1
        }
        fn panel_for(items: &[Item], panels: &mut Vec<(Rect, Option<String>)>, group: &[usize], head: Option<String>) {
            if group.is_empty() {
                return;
            }
            let mut u = items[group[0]].rect;
            for i in &group[1..] {
                u.union_ip(items[*i].rect);
            }
            let mut p = u.inflate(28, 24);
            if head.is_some() {
                p.y -= 22;
                p.h += 22;
            }
            panels.push((p, head));
        }

        if self.tab == "general" {
            let mut y = 116;
            let mut grp = Vec::new();
            for (kind, label) in [
                (ItemKind::Toggle { key: "sound".into(), sect: None, default: false }, t("options.sound")),
                (ItemKind::Volume, t("options.volume")),
                (ItemKind::Toggle { key: "haptik".into(), sect: None, default: false }, t("options.haptik")),
            ] {
                grp.push(add(&mut items, kind, Rect::new(self.left_x, y, col_w, 30), label));
                y += 38;
            }
            panel_for(&items, &mut panels, &grp, Some(t("options.audio")));
            y += if compact { 18 } else { 30 };
            let mut lbl = t("options.replays");
            if font.width(&lbl) > col_w - 76 {
                lbl = t("app.replays");
            }
            let grp = vec![add(
                &mut items,
                ItemKind::Toggle { key: "enabled".into(), sect: Some("replay".into()), default: true },
                Rect::new(self.left_x, y, col_w, 30),
                lbl,
            )];
            panel_for(&items, &mut panels, &grp, if compact { None } else { Some(t("options.recording")) });

            let mut ry = 116;
            let mut grp = Vec::new();
            grp.push(add(
                &mut items,
                ItemKind::Toggle { key: "auto_resolution".into(), sect: None, default: false },
                Rect::new(right_x, ry, col_w, 30),
                t("options.auto_res"),
            ));
            ry += 38;
            grp.push(add(&mut items, ItemKind::Resolution, Rect::new(right_x, ry, col_w, 30), t("options.resolution")));
            ry += 38;
            grp.push(add(&mut items, ItemKind::Fps, Rect::new(right_x, ry, col_w, 30), t("options.fps")));
            ry += 38;
            grp.push(add(&mut items, ItemKind::Language, Rect::new(right_x, ry, col_w, 30), t("options.language")));
            panel_for(&items, &mut panels, &grp, Some(t("options.graphics")));
        } else if self.tab == "controls" {
            let py_ = if compact { 108 } else { 116 };
            let g = vec![add(&mut items, ItemKind::Preset, Rect::new(self.left_x, py_, col_w, 30), t("options.preset"))];
            panel_for(&items, &mut panels, &g, None);
            let by = if compact { 184 } else { 196 };
            let (step, bh) = if compact { (24, 22) } else { (30, 26) };
            for (player, px, head) in [("p1", self.left_x, t("common.player1")), ("p2", right_x, t("common.player2"))] {
                let mut y = by;
                let mut grp = Vec::new();
                for act in smod::ACTIONS {
                    grp.push(add(
                        &mut items,
                        ItemKind::Bind { player: player.into(), action: act.to_string() },
                        Rect::new(px, y, col_w, bh),
                        action_label(act),
                    ));
                    y += step;
                }
                panel_for(&items, &mut panels, &grp, Some(head));
            }
        } else {
            let gap = 14;
            let names = ui::theme_names();
            let n = names.len() as i32;
            let avail_w = w - 2 * self.left_x;
            let avail_h = h - 104 - 56;
            let mut best: Option<((i32, i32), i32, i32, i32, i32)> = None;
            for rws in 1..=3 {
                let cls = (n + rws - 1) / rws;
                let w_ = 300.min(py::fdiv32(avail_w - (cls - 1) * gap, cls));
                let h_ = 210.min(py::fdiv32(avail_h - (rws - 1) * gap, rws));
                let score = if w_ >= 90 { w_.min(260) } else { py::fdiv32(w_, 4) };
                let key = (score * h_.min(200), -rws);
                if best.map(|b| key > b.0).unwrap_or(true) {
                    best = Some((key, rws, cls, w_, h_));
                }
            }
            let (_, _rows, cols, cw, ch) = best.unwrap_or(((0, 0), 2, 5, 100, 100));
            for (i, theme) in names.iter().enumerate() {
                let (row, col) = (i as i32 / cols, i as i32 % cols);
                let in_row = cols.min(n - row * cols);
                let total = in_row * cw + (in_row - 1) * gap;
                let x0 = w / 2 - total / 2;
                add(&mut items, ItemKind::Theme(theme.to_string()), Rect::new(x0 + col * (cw + gap), 104 + row * (ch + gap), cw, ch), String::new());
            }
        }
        add(&mut items, ItemKind::Button, Rect::new(w / 2 - 190, h - 44, 380, 34), t("options.save_back"));
        self.items = items;
        self.panels = panels;
    }

    fn switch_tab(&mut self, key: &'static str) {
        if key == self.tab {
            return;
        }
        self.tab = key;
        self.build_items();
        self.sel = 0;
        self.b.play_sound("move");
    }

    fn close(&mut self) {
        shell::save_settings();
        shell::post(self.zurueck.clone());
    }

    fn click_item(&mut self, i: usize, pos: (i32, i32)) {
        let it = self.items[i].clone();
        match it.kind {
            ItemKind::Bind { .. } | ItemKind::Button | ItemKind::Theme(_) => self.activate(i),
            ItemKind::Volume => {
                if let Some(bar) = it.bar.filter(|b| b.w != 0) {
                    let v = (pos.0 - bar.x) as f64 / bar.w as f64;
                    shell::edit_settings(|s| s.volume = py::round_to(v, 2).clamp(0.0, 1.0));
                    self.save_and_beep();
                } else {
                    self.adjust(i, 1);
                }
            }
            _ => {
                if it.dec.map(|r| r.hit(pos)).unwrap_or(false) {
                    self.adjust(i, -1);
                } else if it.inc.map(|r| r.hit(pos)).unwrap_or(false) {
                    self.adjust(i, 1);
                } else if matches!(it.kind, ItemKind::Toggle { .. }) {
                    self.adjust(i, 1);
                }
            }
        }
    }

    fn capture_key(&mut self, key: &str) {
        if let Some((player, action)) = self.capture.take() {
            if key != "Escape" {
                shell::edit_settings(|s| s.bindings_mut(&player).set(&action, key));
                self.b.settings = shell::settings();
                shell::save_settings();
                self.b.play_sound("select");
            }
        }
    }

    fn key_nav(&mut self, key: &str) {
        let n = self.items.len() as i64;
        match key {
            "Escape" => self.close(),
            "Tab" => {
                let idx = TABS.iter().position(|t| *t == self.tab).unwrap_or(0);
                self.switch_tab(TABS[(idx + 1) % TABS.len()]);
            }
            "Up" | "w" => {
                self.sel = py::pmod(self.sel as i64 - 1, n) as usize;
                self.b.play_sound("move");
            }
            "Down" | "s" => {
                self.sel = py::pmod(self.sel as i64 + 1, n) as usize;
                self.b.play_sound("move");
            }
            "Left" | "a" => self.adjust(self.sel, -1),
            "Right" | "d" => self.adjust(self.sel, 1),
            "Return" | "space" => self.activate(self.sel),
            _ => {}
        }
    }

    fn toggle_get(&self, it: &Item) -> bool {
        let s = shell::settings();
        match &it.kind {
            ItemKind::Toggle { key, sect: Some(sect), default } => s.game_bool(sect, key, *default),
            ItemKind::Toggle { key, sect: None, .. } => match key.as_str() {
                "sound" => s.sound,
                "haptik" => s.haptik,
                "auto_resolution" => s.auto_resolution,
                _ => false,
            },
            _ => false,
        }
    }

    fn toggle_set(&self, it: &Item, value: bool) {
        match &it.kind {
            ItemKind::Toggle { key, sect: Some(sect), .. } => {
                shell::edit_settings(|s| s.set_game(sect, key, value.into()));
            }
            ItemKind::Toggle { key, sect: None, .. } => shell::edit_settings(|s| match key.as_str() {
                "sound" => s.sound = value,
                "haptik" => s.haptik = value,
                "auto_resolution" => s.auto_resolution = value,
                _ => {}
            }),
            _ => {}
        }
    }

    fn save_and_beep(&mut self) {
        shell::save_settings();
        self.b.settings = shell::settings();
        self.b.play_sound("select");
    }

    fn adjust(&mut self, i: usize, d: i64) {
        let it = self.items[i].clone();
        match &it.kind {
            ItemKind::Theme(_) => {
                let idxs: Vec<usize> =
                    self.items.iter().enumerate().filter(|(_, x)| matches!(x.kind, ItemKind::Theme(_))).map(|(i, _)| i).collect();
                if let Some(p) = idxs.iter().position(|x| *x == self.sel) {
                    self.sel = idxs[py::pmod(p as i64 + d, idxs.len() as i64) as usize];
                } else {
                    self.sel = idxs[0];
                }
                self.b.play_sound("move");
            }
            ItemKind::Toggle { key, .. } => {
                let neu = !self.toggle_get(&it);
                self.toggle_set(&it, neu);
                if key == "auto_resolution" {
                    shell::post(Cmd::SetAutoResolution(neu));
                }
                self.save_and_beep();
            }
            ItemKind::Volume => {
                shell::edit_settings(|s| {
                    let v = s.volume + 0.1 * d as f64;
                    s.volume = py::round_to(v, 2).clamp(0.0, 1.0);
                });
                self.save_and_beep();
            }
            ItemKind::Preset => {
                let presets = smod::presets();
                self.preset_idx = py::pmod(self.preset_idx as i64 + d, presets.len() as i64) as usize;
                let (_, p1, p2) = presets[self.preset_idx].clone();
                shell::edit_settings(|s| {
                    s.p1 = p1;
                    s.p2 = p2;
                });
                self.save_and_beep();
            }
            ItemKind::Resolution => {
                if shell::settings().auto_resolution {
                    return;
                }
                self.res_idx = py::pmod(self.res_idx as i64 + d, smod::RESOLUTIONS.len() as i64) as usize;
                let (_, rw, rh) = smod::RESOLUTIONS[self.res_idx];
                shell::post(Cmd::ApplyResolution(rw, rh, true));
                self.b.play_sound("select");
            }
            ItemKind::Fps => {
                self.fps_idx = py::pmod(self.fps_idx as i64 + d, smod::FPS_OPTIONS.len() as i64) as usize;
                shell::post(Cmd::ApplyFps(smod::FPS_OPTIONS[self.fps_idx]));
                self.b.play_sound("select");
            }
            ItemKind::Language => {
                let codes: Vec<&str> = i18n::available().iter().map(|(c, _)| *c).collect();
                let cur_lang = i18n::get_language();
                let cur = codes.iter().position(|c| *c == cur_lang).unwrap_or(0);
                i18n::set_language(codes[py::pmod(cur as i64 + d, codes.len() as i64) as usize], true);
                pgz_core::achievements::event("polyglot", None);
                shell::post(Cmd::RefreshLanguage);
                self.build_items();
                self.b.play_sound("select");
            }
            _ => {}
        }
    }

    fn activate(&mut self, i: usize) {
        let it = self.items[i].clone();
        match &it.kind {
            ItemKind::Bind { player, action } => {
                self.capture = Some((player.clone(), action.clone()));
                self.b.play_sound("click");
            }
            ItemKind::Button => {
                self.b.play_sound("click");
                self.close();
            }
            ItemKind::Theme(theme) => {
                if shell::settings().theme != *theme {
                    let sel = self.sel;
                    // Sofort anwenden (Seitenleiste und Spielflaeche).
                    ui::set_theme(theme);
                    shell::edit_settings(|s| s.theme = theme.clone());
                    shell::save_settings();
                    shell::post(Cmd::ApplyTheme(theme.clone()));
                    self.build_items();
                    self.sel = sel.min(self.items.len() - 1);
                }
                self.b.play_sound("select");
            }
            _ => self.adjust(i, 1),
        }
    }

    // ----- Zeichnen -------------------------------------------------------

    fn fit_render(&self, text: &str, color: Color, max_w: i32) -> pg::Surface {
        let mut img = self.font().render(text, true, color);
        let mono = !ui::is_modern();
        for size in [16, 14, 12] {
            if img.w() <= max_w {
                break;
            }
            img = ui::font(size, false, mono).render(text, true, color);
        }
        img
    }

    fn draw_arrow_value(&mut self, i: usize, farbe: Color, value: &str, value_col: Color) {
        let mut s = self.b.sf();
        let r = self.items[i].rect;
        let f = self.font();
        let label = f.render(&self.items[i].label, true, farbe);
        s.bl(&label, (r.x, r.y));
        let lt = f.render("<", true, farbe);
        let gt = f.render(">", true, farbe);
        let max_w = r.w - label.w() - lt.w() - gt.w() - 34;
        let val = self.fit_render(value, value_col, max_w);
        let pad = 10;
        let gx = r.right() - gt.w();
        let vx = gx - pad - val.w();
        let lx = vx - pad - lt.w();
        s.bl(&lt, (lx, r.y));
        s.bl(&val, (vx, r.y));
        s.bl(&gt, (gx, r.y));
        self.items[i].dec = Some(Rect::new(lx - 6, r.y - 4, lt.w() + 12, r.h + 8));
        self.items[i].inc = Some(Rect::new(gx - 6, r.y - 4, gt.w() + 12, r.h + 8));
    }

    fn draw_item(&mut self, i: usize, selected: bool) {
        let p = ui::pal();
        let it = self.items[i].clone();
        let r = it.rect;
        let farbe = if selected { p.text } else { p.text_dim };
        let mut s = self.b.sf();
        let st = shell::settings();
        match &it.kind {
            ItemKind::Toggle { .. } => {
                let an = self.toggle_get(&it);
                let wert = if an { t("common.on") } else { t("common.off") };
                self.draw_arrow_value(i, farbe, &wert, if an { p.green } else { p.text_dim });
            }
            ItemKind::Language => {
                let cur = i18n::get_language();
                let name = i18n::available().iter().find(|(c, _)| *c == cur).map(|(_, n)| n.to_string()).unwrap_or(cur);
                self.draw_arrow_value(i, farbe, &name, p.gold);
            }
            ItemKind::Volume => {
                let f = self.font();
                s.bl(&f.render(&it.label, true, farbe), (r.x, r.y));
                let v = st.volume;
                let pct = f.render(&format!("{}%", (v * 100.0) as i32), true, farbe);
                let bar = Rect::new(r.right() - pct.w() - 8 - 110, r.y + 8, 110, 12);
                self.items[i].bar = Some(bar);
                draw::rect_r(&mut s, p.btn, bar, 0, 6);
                let fill_w = (bar.w as f64 * v) as i32;
                if fill_w > 0 {
                    draw::rect_r(&mut s, p.accent, (bar.x, bar.y, fill_w, bar.h), 0, 6);
                }
                draw::rect_r(&mut s, p.border_light, bar, 1, 6);
                let knob_x = bar.x + 4.max((bar.w - 4).min(fill_w));
                draw::circle(&mut s, p.text, (knob_x, bar.centery()), 5, 0);
                s.bl(&pct, (r.right() - pct.w(), r.y));
            }
            ItemKind::Preset => {
                let presets = smod::presets();
                let name = t(presets[self.preset_idx].0);
                self.draw_arrow_value(i, farbe, &name, p.gold);
            }
            ItemKind::Resolution => {
                if st.auto_resolution {
                    self.items[i].dec = None;
                    self.items[i].inc = None;
                    let f = self.font();
                    let label = f.render(&it.label, true, farbe);
                    s.bl(&label, (r.x, r.y));
                    let txt = tf("options.res_auto", &[("w", &self.b.width.to_string()), ("h", &self.b.height.to_string())]);
                    let img = self.fit_render(&txt, p.text_dim, r.w - label.w() - 12);
                    s.bl(&img, (r.right() - img.w(), r.y));
                } else {
                    let name = t(smod::RESOLUTIONS[self.res_idx].0);
                    self.draw_arrow_value(i, farbe, &name, p.gold);
                }
            }
            ItemKind::Fps => {
                let v = smod::FPS_OPTIONS[self.fps_idx].to_string();
                self.draw_arrow_value(i, farbe, &v, p.green);
            }
            ItemKind::Bind { player, action } => {
                let f = self.font();
                s.bl(&f.render(&it.label, true, farbe), (r.x, r.y));
                let key = st.bindings(player).get(action).to_string();
                let img = f.render(&pretty_key(&key), true, p.gold);
                let chip = Rect::new(0, 0, img.w() + 14, img.h() + 4).midright_at((r.right(), r.centery()));
                draw::rect_r(&mut s, if selected { p.panel_light } else { p.btn }, chip, 0, 5);
                draw::rect_r(&mut s, p.border_light, chip, 1, 5);
                s.blr(&img, img.get_rect().center_at(chip.center()));
            }
            ItemKind::Theme(theme) => self.draw_theme_card(r, theme, selected),
            ItemKind::Button => {
                ui::draw_button(&mut s, r, &it.label, &self.font(), selected);
            }
        }
    }

    fn draw_theme_card(&mut self, r: Rect, theme: &str, selected: bool) {
        let p = ui::pal();
        let mut s = self.b.sf();
        let active = shell::settings().theme == theme;
        ui::draw_panel_ex(&mut s, r, None, None, Some(10), false, None);
        if selected {
            draw::rect_r(&mut s, p.accent, r, 2, 10);
        } else if active {
            draw::rect_r(&mut s, ui::mix(p.border, p.green, 0.55), r, 1, 10);
        }
        let pv = Rect::new(r.x + 12, r.y + 12, r.w - 24, 40.max((r.h as f64 * 0.42) as i32));
        draw_theme_preview(&mut s, pv, theme);

        let mut label = t(&format!("options.theme_{theme}"));
        let max_w = r.w - 28;
        let mut name_font = ui::fb(16);
        let mut passt = false;
        for fsize in [16, 14, 12, 11] {
            name_font = ui::fb(fsize);
            if name_font.width(&label) <= max_w {
                passt = true;
                break;
            }
        }
        if !passt {
            while !label.is_empty() && name_font.width(&format!("{label}...")) > max_w {
                label.pop();
            }
            label = format!("{}...", label.trim_end());
        }
        let name = name_font.render(&label, true, p.text);
        s.bl(&name, (r.x + 14, pv.bottom() + 10));
        let mut dy = pv.bottom() + 14 + name.h();
        let f12 = ui::f(12);
        let lh = f12.get_height() + 2;
        let bottom = r.bottom() - if active { 32 } else { 10 };
        let max_lines = 1.max(4.min(py::fdiv32(bottom - dy, lh))) as usize;
        let mut lines = wrap_text(&t(&format!("options.theme_{theme}_desc")), &f12, r.w - 28);
        if lines.len() > max_lines {
            lines.truncate(max_lines);
            let mut words: Vec<String> = lines.last().map(|l| l.split_whitespace().map(String::from).collect()).unwrap_or_default();
            while !words.is_empty() && f12.width(&format!("{} ...", words.join(" "))) > r.w - 28 {
                words.pop();
            }
            if let Some(l) = lines.last_mut() {
                *l = format!("{} ...", words.join(" "));
            }
        }
        for line in &lines {
            let img = f12.render(line, true, p.text_dim);
            s.bl(&img, (r.x + 14, dy));
            dy += img.h() + 2;
        }
        if active {
            let tag = ui::fb(11).render(&t("options.theme_active"), true, p.green);
            let chip = Rect::new(0, 0, tag.w() + 14, tag.h() + 6).bottomright_at((r.right() - 10, r.bottom() - 8));
            draw::rect_r(&mut s, ui::mix(p.panel, p.green, 0.18), chip, 0, chip.h / 2);
            draw::rect_r(&mut s, p.green, chip, 1, chip.h / 2);
            s.blr(&tag, tag.get_rect().center_at(chip.center()));
        }
    }

    fn draw_capture_overlay(&self) {
        let p = ui::pal();
        let mut s = self.b.sf();
        let (w, h) = (self.b.width, self.b.height);
        let mut ov = pg::surface_alpha(w, h);
        ov.fill_c(Color::rgba(8, 10, 18, 185));
        s.bl(&ov, (0, 0));
        let (cw, ch) = (420.min(w - 40), 170);
        let card = Rect::new(w / 2 - cw / 2, h / 2 - ch / 2, cw, ch);
        ui::draw_panel_ex(&mut s, card, None, None, Some(14), true, None);
        draw::rect_r(&mut s, p.accent, card, 2, 14);
        if !ui::is_modern() {
            let glow = (120.0 + 100.0 * ui::pulse(2.6, 0.35, 1.0)) as i32;
            draw::rect_r(&mut s, pg::rgb(glow / 3, glow / 2, glow), card.inflate(8, 8), 1, 16);
        }
        if let Some((player, action)) = &self.capture {
            let who = if player == "p1" { t("common.player1") } else { t("common.player2") };
            self.b.draw_center_text(&format!("{who}  -  {}", action_label(action)), &self.font(), p.accent, -44);
        }
        self.b.draw_center_text(&t("options.press_key"), &ui::fb(30), p.text, 0);
        self.b.draw_center_text(&t("options.cancel"), &self.font(), p.text_dim, 46);
    }
}

/// `OptionsScreen._wrap_text` - an Wortgrenzen umbrechen.
pub fn wrap_text(text: &str, fnt: &pg::Font, max_w: i32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        let probe = if cur.is_empty() { word.to_string() } else { format!("{cur} {word}") };
        if !cur.is_empty() && fnt.width(&probe) > max_w {
            lines.push(std::mem::replace(&mut cur, word.to_string()));
        } else {
            cur = probe;
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

/// `OptionsScreen._draw_theme_preview` - Mini-Vorschau eines Designs.
fn draw_theme_preview(target: &mut pg::Surface, rect: Rect, theme: &str) {
    let def = pgz_core::defs::THEMES.iter().find(|x| x.name == theme).unwrap_or(&pgz_core::defs::THEMES[0]);
    let c = &def.palette;
    let fxx = &def.fx;
    let (w, h) = (rect.w, rect.h);
    let mut pv = pg::surface_alpha(w, h);
    let c3 = ui::c3;

    fn grad_bar(pv: &mut pg::Surface, x: i32, y: i32, bar_w: i32, bar_h: i32, stops: &[Color]) {
        let seg = 1.max(stops.len() as i32 - 1);
        for i in 0..1.max(bar_w) {
            let f = i as f64 / 1.max(bar_w - 1) as f64 * seg as f64;
            let k = ((seg - 1) as usize).min(f as usize);
            pv.fill_r(ui::mix(stops[k], stops[k + 1], f - k as f64), (x + i, y, 1, bar_h));
        }
    }

    if let Some((m, a)) = fxx.pattern {
        // Feineres Muster als im Spiel (ui.draw_zigzag mit unit=max(6, h//6)).
        draw_zigzag_unit(&mut pv, w, h, c3(m), c3(a), 6.max(h / 6));
    } else {
        for i in 0..8 {
            let col = ui::mix(c3(c.bg_top), c3(c.bg_bottom), i as f64 / 7.0);
            draw::rect(&mut pv, col, (0, h * i / 8, w, h / 8 + 1), 0);
        }
    }
    if fxx.style == "v42" && !fxx.mesh.is_empty() {
        let mut glow = pg::surface(w, h);
        for blob in fxx.mesh {
            draw::circle(&mut glow, c3(blob.color), ((blob.x * w as f64) as i32, (blob.y * h as f64) as i32), 6.max((blob.size * h as f64 * 0.8) as i32), 0);
        }
        let small = pg::transform::smoothscale(&glow, 1.max(w / 6), 1.max(h / 6));
        pv.bl_flags(&pg::transform::smoothscale(&small, w, h), (0, 0), BLEND_RGB_ADD);
    }
    if fxx.aurora {
        let mut glow = pg::surface(w, h);
        draw::circle(&mut glow, Color::rgb(26, 42, 82), ((w as f64 * 0.25) as i32, (h as f64 * 0.35) as i32), 8.max(h / 2), 0);
        draw::circle(&mut glow, Color::rgb(46, 30, 82), ((w as f64 * 0.78) as i32, (h as f64 * 0.65) as i32), 8.max(h / 2), 0);
        let small = pg::transform::smoothscale(&glow, 1.max(w / 6), 1.max(h / 6));
        pv.bl_flags(&pg::transform::smoothscale(&small, w, h), (0, 0), BLEND_RGB_ADD);
    }
    if fxx.stars {
        for (fx0, fy0) in [(0.08, 0.18), (0.22, 0.62), (0.34, 0.30), (0.48, 0.75), (0.55, 0.14), (0.68, 0.52), (0.80, 0.24), (0.91, 0.66), (0.15, 0.85)] {
            let cc = ((120.0 + 90.0 * fy0) * fxx.star_bright) as i32;
            pv.fill_r(pg::rgb(cc, cc, 255.min(cc + 25)), ((fx0 * w as f64) as i32, (fy0 * h as f64) as i32, 2, 2));
        }
    }
    if fxx.celestial {
        ui::draw_black_hole(&mut pv, ((w as f64 * 0.22) as i32, (h as f64 * 0.46) as i32), 14.max((h as f64 * 0.52) as i32));
        ui::draw_saturn(&mut pv, ((w as f64 * 0.80) as i32, (h as f64 * 0.32) as i32), 6.max(h / 7));
    }
    let ac = c3(c.accent);
    let tw = (w as f64 * 0.42) as i32;
    draw::rect_r(&mut pv, c3(c.text), (w / 2 - tw / 2, 8, tw, 5), 0, 2);
    if let Some(g3) = fxx.grad3 {
        grad_bar(&mut pv, w / 2 - tw / 4, 17, tw / 2, 2, &[c3(g3[0]), c3(g3[1]), c3(g3[2])]);
    } else {
        draw::rect_r(&mut pv, ac, (w / 2 - tw / 4, 17, tw / 2, 2), 0, 1);
    }
    let (bw, bh) = ((w as f64 * 0.62) as i32, 12.max(h / 4));
    let br = Rect::new(w / 2 - bw / 2, h - bh - 8, bw, bh);
    if fxx.btn_glow {
        draw::rect_r(&mut pv, pg::alpha(ac, 70), br.inflate(8, 8), 0, 8);
        draw::rect_r(&mut pv, ui::mix(c3(c.btn), c3(c.btn_sel), 0.8), br, 0, 6);
        draw::rect_r(&mut pv, ac, br, 1, 6);
        draw::rect_r(&mut pv, ac, (br.x + 4, br.centery() - bh / 4, 3, bh / 2), 0, 1);
    } else if fxx.style == "v42" {
        draw::rect_r(&mut pv, c3(c.btn_sel), br, 0, 6);
        draw::rect_r(&mut pv, c3(c.border_light), br, 1, 6);
        draw::rect(&mut pv, ui::mix(c3(c.btn_sel), Color::rgb(255, 255, 255), 0.18), (br.x + 5, br.y + 1, 2.max(br.w - 10), 1), 0);
        let stops = match fxx.grad3 {
            Some(g3) => [c3(g3[0]), c3(g3[1]), c3(g3[2])],
            None => [ac, ac, ac],
        };
        let uw = 6.max((br.w as f64 * 0.5) as i32);
        grad_bar(&mut pv, br.centerx() - uw / 2, br.bottom() - 5, uw, 2, &stops);
    } else {
        draw::rect_r(&mut pv, c3(c.btn), br, 0, 5);
        draw::rect_r(&mut pv, ac, br, 1, 5);
        draw::rect_r(&mut pv, ac, (br.x + 5, br.centery() - bh / 4, 2, bh / 2), 0, 1);
    }
    draw::rect_r(&mut pv, c3(c.text_dim), (br.centerx() - bw / 5, br.centery() - 2, (bw as f64 * 0.4) as i32, 3), 0, 1);
    let mut mask = pg::surface_alpha(w, h);
    draw::rect_r(&mut mask, Color::rgba(255, 255, 255, 255), (0, 0, w, h), 0, 6);
    pv.bl_flags(&mask, (0, 0), BLEND_RGBA_MIN);
    target.bl(&pv, (rect.x, rect.y));
    draw::rect_r(target, c3(c.border_light), rect, 1, 6);
}

/// `ui.draw_zigzag(..., unit=u)` mit frei waehlbarer Kachelhoehe.
fn draw_zigzag_unit(surface: &mut pg::Surface, w: i32, h: i32, main: Color, alt: Color, unit: i32) {
    let ss = 4;
    let u = unit * ss;
    let hu = u / 2;
    let mut block = pg::surface(6 * u, 3 * u);
    block.fill_c(alt);
    for ty in 0..3 {
        for tx in 0..3 {
            let (ox, oy) = (tx * 2 * u, ty * u);
            draw::polygon(&mut block, main, &[(ox + u, oy), (ox, oy + u), (ox + 2 * u, oy + u)], 0);
            draw::polygon(&mut block, alt, &[(ox + u, oy + hu), (ox + hu, oy + u), (ox + u + hu, oy + u)], 0);
            draw::polygon(&mut block, main, &[(ox, oy), (ox + hu, oy + hu), (ox, oy + hu)], 0);
            draw::polygon(&mut block, main, &[(ox + 2 * u, oy), (ox + u + hu, oy + hu), (ox + 2 * u, oy + hu)], 0);
        }
    }
    let small = pg::transform::smoothscale(&block, 6 * unit, 3 * unit);
    let mut tile = pg::surface(2 * unit, unit);
    tile.bl_area(&small, (0, 0), (2 * unit, unit, 2 * unit, unit));
    let (tw, th) = (tile.w(), tile.h());
    let mut row = pg::surface(w, th);
    let mut x = 0;
    while x < w {
        row.bl(&tile, (x, 0));
        x += tw;
    }
    let mut y = 0;
    while y < h {
        surface.bl(&row, (0, y));
        y += th;
    }
}

impl Game for OptionsScreen {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }
    fn reset(&mut self) {}
    fn update(&mut self, _dt: f64) {}
    fn screen_name(&self) -> String {
        t("options.name")
    }
    fn on_surface_changed(&mut self) {
        self.build_items();
        if self.sel >= self.items.len() {
            self.sel = self.items.len() - 1;
        }
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        match ev.kind {
            Kind::KeyDown => {
                if self.capture.is_some() {
                    let k = ev.key.clone();
                    self.capture_key(&k);
                    return;
                }
                let k = ev.key.clone();
                self.key_nav(&k);
            }
            Kind::MouseMove => {
                self.tab_hover = None;
                for (r, key) in &self.tab_rects {
                    if r.hit(ev.pos) {
                        self.tab_hover = Some(key);
                    }
                }
                for (i, it) in self.items.iter().enumerate() {
                    if it.rect.hit(ev.pos) {
                        self.sel = i;
                    }
                }
            }
            Kind::MouseDown => {
                if let Some((_, key)) = self.tab_rects.iter().find(|(r, _)| r.hit(ev.pos)).cloned() {
                    self.switch_tab(key);
                    return;
                }
                if let Some(i) = self.items.iter().position(|it| it.rect.hit(ev.pos)) {
                    self.sel = i;
                    self.click_item(i, ev.pos);
                }
            }
            _ => {}
        }
    }

    fn draw(&mut self) {
        let p = ui::pal();
        let mut s = self.b.sf();
        let (w, h) = (self.b.width, self.b.height);
        ui::draw_background_ex(&mut s, w, h, false, None);
        let head_font = ui::fb(14);
        for (pr, head) in &self.panels {
            ui::draw_panel_ex(
                &mut s,
                *pr,
                None,
                None,
                Some(10),
                false,
                if ui::is_modern() { None } else { Some(ui::mix(p.panel, p.accent, 0.45)) },
            );
            if let Some(hd) = head {
                s.bl(&head_font.render(hd, true, p.accent), (pr.x + 14, pr.y + 8));
            }
        }
        let title = ui::fb(22).render(&t("options.title"), true, p.text);
        s.bl(&title, (self.left_x, 10));
        draw::rect_r(&mut s, p.accent, (self.left_x, 14 + title.h(), title.w(), 2), 0, 2);
        let tab_font = ui::fb(14);
        for (r, key) in &self.tab_rects {
            let active = *key == self.tab;
            if active {
                draw::rect_r(&mut s, p.panel_light, *r, 0, 8);
                draw::rect_r(&mut s, p.accent, (r.x + 8, r.bottom() - 3, r.w - 16, 3), 0, 2);
            } else if Some(*key) == self.tab_hover {
                draw::rect_r(&mut s, p.panel, *r, 0, 8);
                draw::rect_r(&mut s, p.border, *r, 1, 8);
            }
            let img = tab_font.render(&t(&format!("options.tab_{key}")), true, if active { p.text } else { p.text_dim });
            s.blr(&img, img.get_rect().center_at(r.center()));
        }
        let hint = ui::f(12).render(&t("options.tabs_hint"), true, p.text_faint);
        let tabs_right = self.tab_rects.last().map(|(r, _)| r.right()).unwrap_or(0);
        if tabs_right + 24 + hint.w() <= w - self.left_x {
            s.blr(&hint, hint.get_rect().midright_at((w - self.left_x, 48 + 13)));
        }
        if self.tab == "appearance" {
            s.bl(&ui::f(13).render(&t("options.theme_hint"), true, p.text_dim), (self.left_x, 82));
        }
        for i in 0..self.items.len() {
            let r = self.items[i].rect;
            let k = &self.items[i].kind;
            if i == self.sel && !matches!(k, ItemKind::Button | ItemKind::Theme(_)) {
                let hl = r.inflate(16, 8);
                draw::rect_r(&mut s, p.panel_light, hl, 0, 6);
                draw::rect_r(&mut s, p.accent, (hl.x, hl.y + 3, 3, hl.h - 6), 0, 2);
            }
            self.draw_item(i, i == self.sel);
        }
        if self.capture.is_some() {
            self.draw_capture_overlay();
        }
    }
}

// ===========================================================================
//  Willkommen (erster Start)
// ===========================================================================

const ROWS: [&str; 6] = ["auto", "resolution", "sound", "replay", "lang", "start"];
const PRIMARY_ORDER: [&str; 3] = ["en", "de", "fr"];

/// `menu.WelcomeScreen`
pub struct WelcomeScreen {
    b: GameBase,
    expanded: bool,
    res_idx: usize,
    langs: Vec<(String, String)>,
    row: usize,
    lang_sel: usize,
    panel1: Rect,
    ctrl_x: i32,
    head1_y: i32,
    rect_auto: Rect,
    rect_res: Rect,
    rect_sound: Rect,
    rect_replay: Rect,
    rect_start: Rect,
    lang_rects: Vec<Rect>,
    other_rect: Option<Rect>,
    show_lang_head: bool,
    lang_head_y: i32,
    lang_bwid: i32,
    lang_bottom: i32,
    res_dec: Option<Rect>,
    res_inc: Option<Rect>,
}

impl WelcomeScreen {
    /// Der Bildschirm.
    pub fn new(b: GameBase) -> WelcomeScreen {
        let res_idx = shell::settings().resolution_index();
        let z = Rect::new(0, 0, 0, 0);
        let mut s = WelcomeScreen {
            b,
            expanded: false,
            res_idx,
            langs: Vec::new(),
            row: 4,
            lang_sel: 0,
            panel1: z,
            ctrl_x: 0,
            head1_y: 0,
            rect_auto: z,
            rect_res: z,
            rect_sound: z,
            rect_replay: z,
            rect_start: z,
            lang_rects: Vec::new(),
            other_rect: None,
            show_lang_head: true,
            lang_head_y: 0,
            lang_bwid: 0,
            lang_bottom: 0,
            res_dec: None,
            res_inc: None,
        };
        s.build_langs();
        s.build();
        s.row = 4;
        s.lang_sel = s.active_lang_col();
        s
    }

    fn build_langs(&mut self) {
        let alle = i18n::available();
        self.langs = PRIMARY_ORDER
            .iter()
            .filter_map(|c| alle.iter().find(|(x, _)| x == c).map(|(c, n)| (c.to_string(), n.to_string())))
            .collect();
        if self.expanded {
            for (c, n) in i18n::EXTRA {
                self.langs.push((c.to_string(), n.to_string()));
            }
        }
    }

    fn active_lang_col(&self) -> usize {
        let cur = i18n::get_language();
        self.langs.iter().position(|(c, _)| *c == cur).unwrap_or(0)
    }

    fn build(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let compact = h < 380;
        let panel_w = 400.min(w - 40);
        let px = cx - panel_w / 2;
        self.ctrl_x = px + 18;
        let ctrl_w = panel_w - 36;
        let head1_y = if compact { 96 } else { 112 };
        let gap = if compact { 26 } else { 34 };
        self.head1_y = head1_y;
        self.rect_auto = Rect::new(self.ctrl_x, head1_y + 24, ctrl_w, 26);
        self.rect_res = Rect::new(self.ctrl_x, self.rect_auto.y + gap, ctrl_w, 26);
        self.rect_sound = Rect::new(self.ctrl_x, self.rect_res.y + gap, ctrl_w, 26);
        self.rect_replay = Rect::new(self.ctrl_x, self.rect_sound.y + gap, ctrl_w, 26);
        self.panel1 = Rect::new(px, head1_y - 6, panel_w, (self.rect_replay.bottom() + 12) - (head1_y - 6));
        self.show_lang_head = !compact;
        let lang_row_y = if compact {
            self.lang_head_y = self.panel1.bottom() + 12;
            self.panel1.bottom() + 12
        } else {
            self.lang_head_y = self.panel1.bottom() + 14;
            self.lang_head_y + 24
        };
        self.layout_langs(lang_row_y);
        let bw = 300.min(w - 100);
        let start_y = self.lang_bottom + if compact { 12 } else { 18 };
        self.rect_start = Rect::new(cx - bw / 2, start_y, bw, if compact { 34 } else { 40 });
    }

    fn layout_langs(&mut self, y: i32) {
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let gap = 8;
        let n = self.langs.len() as i32;
        if self.expanded {
            let compact = h < 380;
            let area_w = 620.min(w - 24);
            let cols = 5.min(n);
            let rows = (n + cols - 1) / cols;
            let bwid = 1.max(py::fdiv32(area_w - (cols - 1) * gap, cols));
            let start_h = if compact { 34 } else { 40 };
            let reserve = start_h + if compact { 12 } else { 18 } + 8;
            let avail = 60.max((h - reserve) - y);
            let mut lh = py::fdiv32(avail - (rows - 1) * gap, rows);
            lh = 24.max((if compact { 34 } else { 44 }).min(lh));
            let x0 = cx - area_w / 2;
            self.lang_rects = (0..n).map(|i| Rect::new(x0 + (i % cols) * (bwid + gap), y + (i / cols) * (lh + gap), bwid, lh)).collect();
            self.other_rect = None;
            self.lang_bwid = bwid;
            self.lang_bottom = y + rows * (lh + gap) - gap;
        } else {
            let lh = if h < 380 { 36 } else { 48 };
            let area_w = 452.min(w - 24);
            let other_w = 60.max(84.min(area_w / 6));
            let langs_w = area_w - other_w - gap;
            let bwid = 1.max(py::fdiv32(langs_w - (n - 1) * gap, n));
            let x0 = cx - area_w / 2;
            self.lang_rects = (0..n).map(|i| Rect::new(x0 + i * (bwid + gap), y, bwid, lh)).collect();
            self.other_rect = Some(Rect::new(x0 + langs_w + gap, y, other_w, lh));
            self.lang_bwid = bwid;
            self.lang_bottom = y + lh;
        }
    }

    fn lang_targets(&self) -> usize {
        self.langs.len() + usize::from(self.other_rect.is_some())
    }

    fn on_key(&mut self, key: &str) {
        match key {
            "Escape" => self.finish(),
            "Up" | "w" => {
                self.row = py::pmod(self.row as i64 - 1, ROWS.len() as i64) as usize;
                self.b.play_sound("move");
            }
            "Down" | "s" => {
                self.row = (self.row + 1) % ROWS.len();
                self.b.play_sound("move");
            }
            "Left" | "a" => self.horizontal(-1),
            "Right" | "d" => self.horizontal(1),
            "Return" | "space" => self.activate(),
            _ => {}
        }
    }

    fn horizontal(&mut self, d: i64) {
        match ROWS[self.row] {
            "auto" => self.set_auto(d > 0),
            "sound" => self.set_sound(d > 0),
            "replay" => self.set_replay(d > 0),
            "resolution" => self.change_res(d),
            "lang" => {
                self.lang_sel = py::pmod(self.lang_sel as i64 + d, self.lang_targets() as i64) as usize;
                self.b.play_sound("move");
            }
            _ => {}
        }
    }

    fn activate(&mut self) {
        let st = shell::settings();
        match ROWS[self.row] {
            "auto" => self.set_auto(!st.auto_resolution),
            "sound" => self.set_sound(!st.sound),
            "replay" => self.set_replay(!shell::replay_enabled(&st)),
            "resolution" => self.change_res(1),
            "lang" => {
                if self.lang_sel < self.langs.len() {
                    self.select_lang(self.lang_sel);
                } else {
                    self.expand();
                }
            }
            "start" => self.finish(),
            _ => {}
        }
    }

    fn hover(&mut self, pos: (i32, i32)) {
        if self.rect_auto.hit(pos) {
            self.row = 0;
        } else if self.rect_res.hit(pos) {
            self.row = 1;
        } else if self.rect_sound.hit(pos) {
            self.row = 2;
        } else if self.rect_replay.hit(pos) {
            self.row = 3;
        } else if self.rect_start.hit(pos) {
            self.row = 5;
        } else {
            if let Some(i) = self.lang_rects.iter().position(|r| r.hit(pos)) {
                self.row = 4;
                self.lang_sel = i;
                return;
            }
            if self.other_rect.map(|r| r.hit(pos)).unwrap_or(false) {
                self.row = 4;
                self.lang_sel = self.langs.len();
            }
        }
    }

    fn click(&mut self, pos: (i32, i32)) {
        let st = shell::settings();
        if self.rect_auto.hit(pos) {
            self.set_auto(!st.auto_resolution);
        } else if self.rect_sound.hit(pos) {
            self.set_sound(!st.sound);
        } else if self.rect_replay.hit(pos) {
            self.set_replay(!shell::replay_enabled(&st));
        } else if self.rect_res.hit(pos) {
            if self.res_dec.map(|r| r.hit(pos)).unwrap_or(false) {
                self.change_res(-1);
            } else if self.res_inc.map(|r| r.hit(pos)).unwrap_or(false) {
                self.change_res(1);
            } else if !st.auto_resolution {
                self.change_res(1);
            }
        } else if self.rect_start.hit(pos) {
            self.finish();
        } else {
            if let Some(i) = self.lang_rects.iter().position(|r| r.hit(pos)) {
                self.select_lang(i);
                return;
            }
            if self.other_rect.map(|r| r.hit(pos)).unwrap_or(false) {
                self.expand();
            }
        }
    }

    fn set_auto(&mut self, on: bool) {
        if shell::settings().auto_resolution == on {
            return;
        }
        shell::edit_settings(|s| s.auto_resolution = on);
        shell::post(Cmd::SetAutoResolution(on));
        shell::save_settings();
        self.b.play_sound("select");
    }

    fn set_sound(&mut self, on: bool) {
        if shell::settings().sound == on {
            return;
        }
        shell::edit_settings(|s| s.sound = on);
        shell::save_settings();
        self.b.settings = shell::settings();
        self.b.play_sound("select");
    }

    fn set_replay(&mut self, on: bool) {
        if shell::replay_enabled(&shell::settings()) == on {
            return;
        }
        shell::edit_settings(|s| s.set_game("replay", "enabled", on.into()));
        shell::save_settings();
        self.b.play_sound("select");
    }

    fn change_res(&mut self, d: i64) {
        if shell::settings().auto_resolution {
            return;
        }
        self.res_idx = py::pmod(self.res_idx as i64 + d, smod::RESOLUTIONS.len() as i64) as usize;
        let (_, rw, rh) = smod::RESOLUTIONS[self.res_idx];
        shell::post(Cmd::ApplyResolution(rw, rh, true));
        shell::save_settings();
        self.b.play_sound("select");
    }

    fn select_lang(&mut self, i: usize) {
        i18n::set_language(&self.langs[i].0, true);
        shell::post(Cmd::RefreshLanguage);
        self.build_langs();
        self.build();
        self.row = 4;
        self.lang_sel = i;
        self.b.play_sound("click");
        if let Some(r) = self.lang_rects.get(i) {
            ui::spawn_burst(r.centerx() as f64, r.centery() as f64, Some(ui::accent()), 18, 260.0);
        }
    }

    fn expand(&mut self) {
        if self.expanded {
            return;
        }
        self.expanded = true;
        self.build_langs();
        self.build();
        self.lang_sel = PRIMARY_ORDER.len();
        self.b.play_sound("click");
    }

    fn finish(&mut self) {
        i18n::set_language(&i18n::get_language(), true);
        shell::save_settings();
        self.b.play_sound("select");
        ui::spawn_burst(self.rect_start.centerx() as f64, self.rect_start.centery() as f64, Some(ui::green()), 18, 260.0);
        shell::post(Cmd::BackToMenu);
    }

    fn draw_row_focus(&self, rect: Rect) {
        let p = ui::pal();
        let mut s = self.b.sf();
        let hl = rect.inflate(16, 8);
        draw::rect_r(&mut s, p.panel_light, hl, 0, 6);
        draw::rect_r(&mut s, p.accent, (hl.x, hl.y + 3, 3, hl.h - 6), 0, 2);
    }

    fn draw_toggle(&self, rect: Rect, label: &str, on: bool, focused: bool) {
        let p = ui::pal();
        let mut s = self.b.sf();
        if focused {
            self.draw_row_focus(rect);
        }
        let col = if focused { p.text } else { p.text_dim };
        let img = ui::f(17).render(label, true, col);
        s.bl(&img, (rect.x, rect.centery() - img.h() / 2));
        let (pill_w, pill_h) = (46, 22);
        let pill = Rect::new(rect.right() - pill_w, rect.centery() - pill_h / 2, pill_w, pill_h);
        draw::rect_r(&mut s, ui::mix(p.btn, p.green, if on { 1.0 } else { 0.0 }), pill, 0, pill_h / 2);
        draw::rect_r(&mut s, p.border_light, pill, 1, pill_h / 2);
        let knob_r = pill_h / 2 - 3;
        let knob_x = if on { pill.right() - knob_r - 4 } else { pill.left() + knob_r + 4 };
        draw::circle(&mut s, p.text, (knob_x, pill.centery()), knob_r, 0);
        let word = ui::fb(13).render(&if on { t("common.on") } else { t("common.off") }, true, if on { p.green } else { p.text_dim });
        s.bl(&word, (pill.left() - 8 - word.w(), rect.centery() - word.h() / 2));
    }

    fn draw_resolution(&mut self, rect: Rect, focused: bool, auto_on: bool) {
        let p = ui::pal();
        let mut s = self.b.sf();
        if focused {
            self.draw_row_focus(rect);
        }
        let col = if focused { p.text } else { p.text_dim };
        let f = ui::f(17);
        let label = f.render(&t("options.resolution"), true, col);
        s.bl(&label, (rect.x, rect.centery() - label.h() / 2));
        if auto_on {
            self.res_dec = None;
            self.res_inc = None;
            let img = f.render(&tf("options.res_auto", &[("w", &self.b.width.to_string()), ("h", &self.b.height.to_string())]), true, p.text_dim);
            s.bl(&img, (rect.right() - img.w(), rect.centery() - img.h() / 2));
            return;
        }
        let lt = f.render("<", true, col);
        let gt = f.render(">", true, col);
        let val = ui::f(15).render(&t(smod::RESOLUTIONS[self.res_idx].0), true, p.gold);
        let pad = 10;
        let gx = rect.right() - gt.w();
        let vx = gx - pad - val.w();
        let lx = vx - pad - lt.w();
        let cy = rect.centery();
        s.bl(&lt, (lx, cy - lt.h() / 2));
        s.bl(&val, (vx, cy - val.h() / 2));
        s.bl(&gt, (gx, cy - gt.h() / 2));
        self.res_dec = Some(Rect::new(lx - 6, rect.y - 4, lt.w() + 12, rect.h + 8));
        self.res_inc = Some(Rect::new(gx - 6, rect.y - 4, gt.w() + 12, rect.h + 8));
    }
}

impl Game for WelcomeScreen {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }
    fn reset(&mut self) {}
    fn update(&mut self, _dt: f64) {}
    fn screen_name(&self) -> String {
        t("welcome.name")
    }
    fn on_surface_changed(&mut self) {
        self.build_langs();
        self.build();
        self.row = self.row.min(ROWS.len() - 1);
        self.lang_sel = self.lang_sel.min(self.lang_targets() - 1);
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        match ev.kind {
            Kind::KeyDown => {
                let k = ev.key.clone();
                self.on_key(&k);
            }
            Kind::MouseMove => self.hover(ev.pos),
            Kind::MouseDown => {
                self.hover(ev.pos);
                self.click(ev.pos);
            }
            _ => {}
        }
    }

    fn draw(&mut self) {
        let p = ui::pal();
        let mut s = self.b.sf();
        let (w, h) = (self.b.width, self.b.height);
        let compact = h < 380;
        ui::draw_background(&mut s, w, h);
        ui::draw_title_ex(
            &mut s,
            w,
            "PyGameZ",
            Some(&t("welcome.subtitle")),
            if compact { 30 } else { 42 },
            Some(&ui::fb(if compact { 26 } else { 32 })),
            Some(&ui::f(if compact { 14 } else { 16 })),
            None,
        );
        ui::draw_panel_ex(&mut s, self.panel1, None, None, Some(10), false, Some(ui::mix(p.panel, p.accent, 0.45)));
        s.bl(&ui::fb(14).render(&t("welcome.setup"), true, p.accent), (self.ctrl_x, self.head1_y + 2));
        let st = shell::settings();
        self.draw_toggle(self.rect_auto, &t("options.auto_res"), st.auto_resolution, self.row == 0);
        let (rr, foc) = (self.rect_res, self.row == 1);
        self.draw_resolution(rr, foc, st.auto_resolution);
        self.draw_toggle(self.rect_sound, &t("options.sound"), st.sound, self.row == 2);
        self.draw_toggle(self.rect_replay, &t("options.replays"), shell::replay_enabled(&st), self.row == 3);
        if self.show_lang_head {
            let head = ui::fb(15).render(&t("lang.name"), true, p.accent);
            s.blr(&head, head.get_rect().midtop_at((w / 2, self.lang_head_y)));
        }
        let lang_focus = self.row == 4;
        let bw = self.lang_bwid;
        let longest = self.langs.iter().map(|(_, l)| l.clone()).max_by_key(|l| l.chars().count()).unwrap_or_default();
        let mut bsize = if compact { 18 } else { 22 };
        while bsize > 11 && ui::fb(bsize).width(&longest) > bw - 12 {
            bsize -= 1;
        }
        let bfont = ui::fb(bsize);
        let cfont = ui::f(11);
        let cur = i18n::get_language();
        for (i, (code, label)) in self.langs.iter().enumerate() {
            let r = self.lang_rects[i];
            ui::draw_button_ex(&mut s, r, label, &bfont, lang_focus && self.lang_sel == i, Some(&code.to_uppercase()), Some(&cfont), None);
            if *code == cur {
                draw::rect_r(&mut s, p.green, r.inflate(4, 4), 2, 12);
            }
        }
        if let Some(or) = self.other_rect {
            let hot = lang_focus && self.lang_sel == self.langs.len();
            ui::draw_button_ex(&mut s, or, &t("welcome.more"), &ui::f(15), hot, None, None, Some(p.accent2));
        }
        ui::draw_button_ex(&mut s, self.rect_start, &t("welcome.start"), &ui::fb(if compact { 19 } else { 22 }), self.row == 5, None, None, Some(p.green));
        ui::draw_footer(&mut s, w, h, &t("welcome.hint"));
    }
}

// ===========================================================================
//  Sprachwahl
// ===========================================================================

/// `menu.LanguageScreen`
pub struct LanguageScreen {
    b: GameBase,
    expanded: bool,
    langs: Vec<(String, String)>,
    rects: Vec<Rect>,
    other_rect: Option<Rect>,
    cols: i32,
    sel: usize,
}

impl LanguageScreen {
    /// Der Bildschirm.
    pub fn new(b: GameBase) -> LanguageScreen {
        let mut s = LanguageScreen { b, expanded: i18n::has_language(), langs: Vec::new(), rects: Vec::new(), other_rect: None, cols: 1, sel: 0 };
        s.build();
        let cur = i18n::get_language();
        s.sel = s.langs.iter().position(|(c, _)| *c == cur).unwrap_or(0);
        s
    }

    fn build(&mut self) {
        self.langs = if self.expanded {
            i18n::available().iter().map(|(c, n)| (c.to_string(), n.to_string())).collect()
        } else {
            i18n::PRIMARY.iter().map(|(c, n)| (c.to_string(), n.to_string())).collect()
        };
        self.rects.clear();
        let n = self.langs.len() as i32;
        let (w, h) = (self.b.width, self.b.height);
        let area_w = 760.min(w - 40);
        let gap = 16;
        let top = 124;
        let bottom = h - if self.expanded { 44 } else { 88 };
        let cols = if n <= 4 {
            1
        } else {
            let mut c = 4;
            for cand in [2, 3, 4] {
                let cw = py::fdiv32(area_w - (cand - 1) * gap, cand);
                let rws = (n + cand - 1) / cand;
                if rws * 36 - 6 <= (bottom - top) && (cand == 2 || cw >= 120) {
                    c = cand;
                    break;
                }
            }
            c
        };
        self.cols = cols;
        let col_w = py::fdiv32(area_w - (cols - 1) * gap, cols);
        let rows = (n + cols - 1) / cols;
        let (mut bh, mut vgap) = (56, 16);
        while rows * (bh + vgap) - vgap > bottom - top && bh > 30 {
            bh -= 2;
            vgap = 6.max(vgap - 1);
        }
        let grid_h = rows * (bh + vgap) - vgap;
        let y0 = top + 0.max(py::fdiv32(bottom - top - grid_h, 2));
        let x0 = w / 2 - area_w / 2;
        for i in 0..n {
            let (r, c) = (i / cols, i % cols);
            self.rects.push(Rect::new(x0 + c * (col_w + gap), y0 + r * (bh + vgap), col_w, bh));
        }
        self.other_rect = if self.expanded {
            None
        } else {
            let ow = ui::f(14).width(&t("lang.other")) + 26;
            Some(Rect::new(w / 2 - ow / 2, h - 66, ow, 24))
        };
    }

    fn item_count(&self) -> usize {
        self.rects.len() + usize::from(!self.expanded)
    }

    fn expand(&mut self) {
        self.expanded = true;
        self.build();
        self.sel = i18n::PRIMARY.len().min(self.rects.len() - 1);
        self.b.play_sound("click");
    }

    fn choose(&mut self, i: usize) {
        if !self.expanded && i >= self.rects.len() {
            self.expand();
            return;
        }
        let changed = self.langs[i].0 != i18n::get_language();
        i18n::set_language(&self.langs[i].0, true);
        if changed {
            pgz_core::achievements::event("polyglot", None);
        }
        shell::post(Cmd::RefreshLanguage);
        self.b.play_sound("click");
        let r = self.rects[i];
        ui::spawn_burst(r.centerx() as f64, r.centery() as f64, Some(ui::accent()), 18, 260.0);
        shell::post(Cmd::BackToMenu);
    }
}

impl Game for LanguageScreen {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }
    fn reset(&mut self) {}
    fn update(&mut self, _dt: f64) {}
    fn screen_name(&self) -> String {
        t("lang.name")
    }
    fn on_surface_changed(&mut self) {
        self.build();
        self.sel = self.sel.min(self.item_count() - 1);
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        let n = self.item_count() as i64;
        match ev.kind {
            Kind::KeyDown => match ev.key.as_str() {
                "Left" | "a" => {
                    self.sel = py::pmod(self.sel as i64 - 1, n) as usize;
                    self.b.play_sound("move");
                }
                "Right" | "d" => {
                    self.sel = py::pmod(self.sel as i64 + 1, n) as usize;
                    self.b.play_sound("move");
                }
                "Up" | "w" => {
                    self.sel = py::pmod(self.sel as i64 - self.cols as i64, n) as usize;
                    self.b.play_sound("move");
                }
                "Down" | "s" => {
                    self.sel = py::pmod(self.sel as i64 + self.cols as i64, n) as usize;
                    self.b.play_sound("move");
                }
                "Return" | "space" => self.choose(self.sel),
                _ => {}
            },
            Kind::MouseMove => {
                for (i, r) in self.rects.iter().enumerate() {
                    if r.hit(ev.pos) {
                        self.sel = i;
                    }
                }
                if self.other_rect.map(|r| r.hit(ev.pos)).unwrap_or(false) {
                    self.sel = self.rects.len();
                }
            }
            Kind::MouseDown => {
                if let Some(i) = self.rects.iter().position(|r| r.hit(ev.pos)) {
                    self.choose(i);
                    return;
                }
                if self.other_rect.map(|r| r.hit(ev.pos)).unwrap_or(false) {
                    self.choose(self.rects.len());
                }
            }
            _ => {}
        }
    }

    fn draw(&mut self) {
        let p = ui::pal();
        let mut s = self.b.sf();
        let (w, h) = (self.b.width, self.b.height);
        ui::draw_background(&mut s, w, h);
        ui::draw_title_ex(&mut s, w, &t("lang.title"), None, 80, Some(&ui::fb(34)), None, None);
        let btn_w = self.rects.first().map(|r| r.w).unwrap_or(240);
        let longest = self.langs.iter().map(|(_, l)| l.clone()).max_by_key(|l| l.chars().count()).unwrap_or_default();
        let mut bsize = 24;
        while bsize > 13 && ui::fb(bsize).width(&longest) > btn_w - 20 {
            bsize -= 1;
        }
        let btn_font = ui::fb(bsize);
        let code_font = ui::f(if bsize >= 18 { 13 } else { 11 });
        for (i, (code, label)) in self.langs.iter().enumerate() {
            ui::draw_button_ex(&mut s, self.rects[i], label, &btn_font, i == self.sel, Some(&code.to_uppercase()), Some(&code_font), None);
        }
        if let Some(or) = self.other_rect {
            let hot = self.sel == self.rects.len();
            let col = if hot { p.text_dim } else { p.text_faint };
            let img = ui::f(14).render(&t("lang.other"), true, col);
            if hot {
                draw::rect_r(&mut s, p.panel_light, or, 0, 6);
                draw::rect_r(&mut s, p.border_light, or, 1, 6);
            }
            s.blr(&img, img.get_rect().center_at(or.center()));
        }
        ui::draw_footer(&mut s, w, h, &t("lang.hint"));
    }
}
