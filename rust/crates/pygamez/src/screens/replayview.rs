//! replayview.rs
//! =============
//! Der Replay-Screen: Archiv und Wiedergabe - uebertragen aus
//! `replayview.py`. Die Wiedergabe zeichnet das Spiel selbst: der Screen baut
//! eine normale Spielinstanz und faehrt sie ueber `replay_begin`/
//! `replay_seek`/`replay_draw` Bild fuer Bild durch die Aufnahme.

use pgz_core::i18n::{t, tf};
use pgz_core::replay;
use pgz_engine::filepick;
use pgz_engine::game::{Game, GameBase, GameInfo, InputEvent, Kind};
use pgz_engine::pg::{self, draw, Color, Rect, RectExt, SurfExt};
use pgz_engine::shell::{self, Cmd};
use pgz_engine::ui::{self, mix};
use serde_json::Value;

const SPEEDS: [f64; 4] = [0.5, 1.0, 2.0, 4.0];
const ROW_H: i32 = 52;
const BAR_H: i32 = 46;
const BAR_SHOW: f64 = 3.0;

fn class_of(key: &str) -> &'static str {
    match key {
        "minigolf" => "MiniGolfGame",
        "bowling" => "BowlingGame",
        "billiard" => "BilliardGame",
        "pinball" => "PinballGame",
        "snake" => "SnakeGame",
        "tetris" => "TetrisGame",
        _ => "",
    }
}

fn pad(game: &str) -> (usize, usize, usize) {
    match game {
        "pinball" => (6, 6, 27),
        "snake" | "tetris" => (0, 0, 27),
        _ => (12, 9, 27),
    }
}

fn seq_key(game: &str) -> &'static str {
    match game {
        "minigolf" => "replay.seq_golf",
        "bowling" => "replay.seq_bowl",
        "billiard" => "replay.seq_bil",
        "pinball" => "replay.seq_pin",
        _ => "replay.seq_part",
    }
}

fn game_accent(key: &str) -> Color {
    ui::game_color(class_of(key))
}

fn s_of(v: &Value, key: &str) -> String {
    match v.get(key) {
        Some(Value::String(s)) => s.clone(),
        _ => String::new(),
    }
}

fn truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().unwrap_or(0.0) != 0.0,
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(o)) => !o.is_empty(),
    }
}

/// Archiv + Wiedergabe. `pending` ist eine noch ungespeicherte Aufnahme.
pub struct ReplayScreen {
    b: GameBase,
    games: Vec<GameInfo>,
    from_game: bool,
    pending: Option<Value>,
    saved: bool,
    tab: &'static str,
    mode: &'static str,
    sel: usize,
    first: usize,
    tab_hover: Option<&'static str>,
    btn_hover: Option<&'static str>,
    hover_row: Option<usize>,
    confirm: Option<String>,
    toast: Option<String>,
    toast_t: f64,
    items: Vec<Value>,
    counts: Vec<(&'static str, usize)>,
    rep: Option<Value>,
    inst: Option<Box<dyn Game>>,
    scenes: Vec<Value>,
    lens: Vec<usize>,
    starts: Vec<usize>,
    total: usize,
    si: usize,
    pi: usize,
    playing: bool,
    speed_idx: usize,
    pre: usize,
    post: usize,
    post_end: usize,
    acc: f64,
    sound_done: bool,
    bar_t: f64,
    error: Option<String>,
    left: i32,
    right: i32,
    top: i32,
    bottom: i32,
    tab_font: pg::Font,
    tab_rects: Vec<(Rect, &'static str, String)>,
    btn_import: Rect,
    btn_export: Rect,
    head_y: i32,
    rows_visible: usize,
    row_rects: Vec<Rect>,
    bar_rect: Rect,
    seek_rect: Rect,
    save_rect: Rect,
}

impl ReplayScreen {
    /// `ReplayScreen(surface, w, h, app, on_close, pending, game)`; `from_game`
    /// = der Zurueck-Weg fuehrt ins geparkte Spiel.
    pub fn new(b: GameBase, games: Vec<GameInfo>, pending: Option<Value>, game: Option<String>, from_game: bool) -> ReplayScreen {
        let z = Rect::new(0, 0, 0, 0);
        let wanted = game.or_else(|| pending.as_ref().map(|p| s_of(p, "game"))).unwrap_or_default();
        let tab = replay::GAMES.iter().find(|g| **g == wanted).copied().unwrap_or(replay::GAMES[0]);
        let f = b.font.clone();
        let (pre, post, post_end) = pad("minigolf");
        let mut s = ReplayScreen {
            b,
            games,
            from_game,
            pending: pending.clone(),
            saved: false,
            tab,
            mode: "list",
            sel: 0,
            first: 0,
            tab_hover: None,
            btn_hover: None,
            hover_row: None,
            confirm: None,
            toast: None,
            toast_t: 0.0,
            items: Vec::new(),
            counts: Vec::new(),
            rep: None,
            inst: None,
            scenes: Vec::new(),
            lens: Vec::new(),
            starts: Vec::new(),
            total: 0,
            si: 0,
            pi: 0,
            playing: true,
            speed_idx: 1,
            pre,
            post,
            post_end,
            acc: 0.0,
            sound_done: false,
            bar_t: BAR_SHOW,
            error: None,
            left: 0,
            right: 0,
            top: 0,
            bottom: 0,
            tab_font: f,
            tab_rects: Vec::new(),
            btn_import: z,
            btn_export: z,
            head_y: 0,
            rows_visible: 1,
            row_rects: Vec::new(),
            bar_rect: z,
            seek_rect: z,
            save_rect: z,
        };
        s.load();
        s.build();
        if let Some(p) = pending {
            s.start(p);
        }
        s
    }

    fn game_name(&self, key: &str) -> String {
        self.games.iter().find(|g| g.key == key).map(|g| g.name()).unwrap_or_else(|| key.to_string())
    }

    fn load(&mut self) {
        self.counts = replay::GAMES.iter().map(|g| (*g, replay::load_game(g).len())).collect();
        self.items = replay::load_game(self.tab);
        self.sel = self.sel.min(self.items.len().saturating_sub(1));
        self.first = 0;
        self.confirm = None;
    }

    fn build(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        self.left = 24.max(w / 2 - 320);
        self.right = w - self.left;
        self.bottom = h - 44;
        self.tab_font = ui::fb(14);
        self.tab_rects.clear();
        let tab_right = w - 24;
        let (mut tx, mut ty) = (self.left, 94);
        for key in replay::GAMES {
            let n = self.counts.iter().find(|c| c.0 == key).map(|c| c.1).unwrap_or(0);
            let label = format!("{} ({})", self.game_name(key), n);
            let tw = self.tab_font.size(&label).0 + 26;
            if tx > self.left && tx + tw > tab_right {
                tx = self.left;
                ty += 30;
            }
            self.tab_rects.push((Rect::new(tx, ty, tw, 26), key, label));
            tx += tw + 8;
        }
        let head_y = ty + 26 + 8;
        let bw = 104;
        self.btn_import = Rect::new(self.right - bw, head_y, bw, 24);
        self.btn_export = Rect::new(self.btn_import.x - 8 - bw, head_y, bw, 24);
        self.head_y = head_y + 12;
        self.top = head_y + 24 + 12;
        self.rows_visible = 1.max(pgz_engine::py::fdiv32(self.bottom - self.top, ROW_H)) as usize;
        self.row_rects = (0..self.rows_visible as i32).map(|i| Rect::new(self.left, self.top + i * ROW_H, self.right - self.left, ROW_H - 6)).collect();
        self.bar_rect = Rect::new(0, h - BAR_H, w, BAR_H);
        self.seek_rect = Rect::new(0, h - BAR_H - 4, w, 10);
        let chip_w = 240.min(w - 40);
        self.save_rect = Rect::new(w / 2 - chip_w / 2, 10, chip_w, 28);
    }

    // ----- Wiedergabe starten/beenden ------------------------------------

    fn start(&mut self, rep: Value) {
        self.error = None;
        self.scenes = rep.get("scenes").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let game = s_of(&rep, "game");
        self.inst = None;
        if !self.scenes.is_empty() {
            if let Some(info) = self.games.iter().find(|g| g.key == game).copied() {
                let base = GameBase::new(self.b.surface.clone(), self.b.width, self.b.height, "single", shell::settings(), info.key, info.class_name);
                let mut inst = (info.make)(base);
                if inst.replay_begin(&rep) {
                    self.inst = Some(inst);
                }
            }
        }
        self.rep = Some(rep);
        if self.inst.is_none() {
            self.error = Some(t("replay.broken"));
            self.mode = "list";
            return;
        }
        let (pre, post, post_end) = pad(&game);
        self.pre = pre;
        self.post = post;
        self.post_end = post_end;
        self.lens.clear();
        self.starts.clear();
        let mut pos = 0;
        for sc in &self.scenes {
            let n = 1.max(replay::scene_len(sc));
            let end = truthy(sc.get("final")) || sc.get("knocked").is_some_and(|k| !k.is_null());
            self.starts.push(pos);
            let length = self.pre + n + if end { self.post_end } else { self.post };
            self.lens.push(length);
            pos += length;
        }
        self.total = pos;
        self.si = 0;
        self.pi = 0;
        self.acc = 0.0;
        self.sound_done = false;
        self.playing = true;
        self.bar_t = BAR_SHOW;
        self.mode = "play";
        self.seek_now();
    }

    fn stop(&mut self) {
        self.inst = None;
        self.rep = None;
        self.mode = "list";
        self.load();
        self.build();
    }

    fn close(&mut self) {
        self.b.play_sound("click");
        if self.from_game {
            shell::post(Cmd::ReturnFromReplay);
        } else {
            shell::post(Cmd::BackToMenu);
        }
    }

    // ----- Zeitachse ------------------------------------------------------

    fn rate(&self) -> f64 {
        self.rep.as_ref().and_then(|r| r.get("rate")).and_then(|v| v.as_f64()).filter(|v| *v != 0.0).unwrap_or(replay::RATE)
    }

    fn phase(&self) -> (usize, bool, bool) {
        let sc = &self.scenes[self.si];
        let n = 1.max(replay::scene_len(sc));
        if self.pi < self.pre {
            return (0, true, false);
        }
        if self.pi < self.pre + n {
            return (self.pi - self.pre, false, false);
        }
        (n - 1, false, truthy(sc.get("final")))
    }

    fn seek_now(&mut self) {
        if self.inst.is_none() || self.scenes.is_empty() {
            return;
        }
        let (frame, _, _) = self.phase();
        let si = self.si;
        if let Some(inst) = self.inst.as_mut() {
            inst.replay_seek(si, frame);
        }
    }

    fn elapsed(&self) -> usize {
        if self.starts.is_empty() {
            return 0;
        }
        self.starts[self.si] + self.pi
    }

    fn at_end(&self) -> bool {
        self.si + 1 >= self.scenes.len() && self.pi + 1 >= *self.lens.last().unwrap_or(&1)
    }

    fn advance(&mut self, steps: usize) {
        for _ in 0..steps {
            if self.at_end() {
                self.playing = false;
                return;
            }
            self.pi += 1;
            if self.pi >= self.lens[self.si] {
                self.si += 1;
                self.pi = 0;
                self.sound_done = false;
            } else if !self.sound_done {
                let sc = self.scenes[self.si].clone();
                let n = 1.max(replay::scene_len(&sc));
                if self.pi >= self.pre + n {
                    self.sound_done = true;
                    self.result_sound(&sc);
                }
            }
        }
        self.seek_now();
    }

    fn result_sound(&self, sc: &Value) {
        let mut key = s_of(sc, "result");
        if key.is_empty() {
            key = s_of(sc, "res");
        }
        let game = self.rep.as_ref().map(|r| s_of(r, "game")).unwrap_or_default();
        let fin = truthy(sc.get("final"));
        let end = s_of(sc, "end");
        if fin && ["snake", "tetris", "pinball"].contains(&game.as_str()) {
            self.b.play_sound("gameover");
        } else if end == "cup" || key == "bowl.strike" || key == "bil.win_you" {
            self.b.play_sound("win");
        } else if key == "bowl.spare" || fin {
            self.b.play_sound("point");
        } else if end == "water" || key == "bil.foul" || key == "bil.win_ai" {
            self.b.play_sound("hit");
        }
    }

    fn jump_scene(&mut self, d: i64) {
        if self.scenes.is_empty() {
            return;
        }
        if d < 0 && self.pi > self.pre + 4 {
            self.pi = 0;
        } else {
            self.si = 0.max((self.scenes.len() as i64 - 1).min(self.si as i64 + d)) as usize;
            self.pi = 0;
        }
        self.sound_done = false;
        self.bar_t = BAR_SHOW;
        self.seek_now();
        self.b.play_sound("move");
    }

    fn seek_ratio(&mut self, ratio: f64) {
        if self.total == 0 {
            return;
        }
        let target = 0.max((self.total as i64 - 1).min((self.total as f64 * ratio) as i64)) as usize;
        for i in (0..self.lens.len()).rev() {
            if target >= self.starts[i] {
                self.si = i;
                self.pi = (self.lens[i] - 1).min(target - self.starts[i]);
                break;
            }
        }
        self.sound_done = true;
        self.bar_t = BAR_SHOW;
        self.seek_now();
    }

    // ----- Speichern / Loeschen -------------------------------------------

    fn save_pending(&mut self) {
        let Some(p) = self.pending.clone() else { return };
        if self.saved || replay::is_saved(&s_of(&p, "game"), &s_of(&p, "id")) {
            self.saved = true;
            self.toast_msg(t("replay.already"));
            return;
        }
        let (ok, why) = replay::save_replay(&p);
        if ok {
            self.saved = true;
            self.toast_msg(t("replay.saved"));
            self.b.play_sound("level");
            ui::spawn_burst((self.b.width / 2) as f64, 24.0, Some(ui::pal().green), 18, 260.0);
            let total: usize = replay::GAMES.iter().map(|g| replay::load_game(g).len()).sum();
            pgz_core::achievements::event("replay_first", None);
            pgz_core::achievements::event("replay_5", Some(total as i64));
        } else if why == "full" {
            self.toast_msg(tf("replay.full", &[("n", &replay::MAX_PER_GAME.to_string())]));
            self.b.play_sound("hit");
        } else {
            self.toast_msg(t("replay.error"));
            self.b.play_sound("hit");
        }
    }

    fn current(&self) -> Option<Value> {
        if self.mode == "play" {
            return self.rep.clone();
        }
        self.items.get(self.sel).cloned()
    }

    fn export(&mut self, rep: Option<Value>) {
        let Some(rep) = rep.or_else(|| self.current()) else { return };
        let name = replay::default_filename(&rep);
        let path = if filepick::available() {
            match filepick::save_as(&name, &t("replay.export_title"), &[replay::EXT, ".json"]) {
                Some(p) => p,
                None => return,
            }
        } else {
            filepick::to_downloads(&name)
        };
        let (ok, _why) = replay::export_to(&rep, std::path::Path::new(&path));
        if ok {
            self.toast_msg(tf("replay.exported", &[("file", &filepick::short(&path, 44))]));
            self.b.play_sound("level");
            ui::spawn_burst((self.b.width / 2) as f64, 24.0, Some(ui::pal().accent2), 18, 260.0);
            pgz_core::achievements::event("replay_share", None);
        } else {
            self.toast_msg(t("replay.export_error"));
            self.b.play_sound("hit");
        }
    }

    fn import(&mut self) {
        if !filepick::available() {
            self.toast_msg(t("replay.no_dialog"));
            self.b.play_sound("hit");
            return;
        }
        let Some(path) = filepick::open_file(&t("replay.import_title"), &[replay::EXT, ".json"]) else { return };
        let (ok, why, rep) = replay::import_from(std::path::Path::new(&path), None);
        match rep.filter(|_| ok) {
            Some(rep) => {
                let g = s_of(&rep, "game");
                if let Some(k) = replay::GAMES.iter().find(|k| **k == g) {
                    self.tab = k;
                }
                self.load();
                self.build();
                let id = s_of(&rep, "id");
                if let Some(i) = self.items.iter().position(|it| s_of(it, "id") == id) {
                    self.sel = i;
                }
                self.toast_msg(tf("replay.imported", &[("title", &s_of(&rep, "title"))]));
                self.b.play_sound("level");
                ui::spawn_burst((self.b.width / 2) as f64, 24.0, Some(ui::pal().green), 18, 260.0);
            }
            None => {
                let w = if ["dup", "full", "game"].contains(&why) { why } else { "error" };
                self.toast_msg(t(&format!("replay.import_{w}")));
                self.b.play_sound("hit");
            }
        }
    }

    fn delete_selected(&mut self) {
        if self.items.is_empty() || self.mode != "list" {
            return;
        }
        let id = s_of(&self.items[self.sel], "id");
        if self.confirm.as_deref() != Some(id.as_str()) {
            self.confirm = Some(id);
            self.toast_msg(t("replay.confirm"));
            self.b.play_sound("move");
            return;
        }
        replay::delete_replay(self.tab, &id);
        self.toast_msg(t("replay.deleted"));
        self.b.play_sound("click");
        self.load();
        self.build();
    }

    fn toast_msg(&mut self, text: String) {
        self.toast = Some(text);
        self.toast_t = 2.6;
    }

    // ----- Eingabe --------------------------------------------------------

    fn event_play(&mut self, ev: &InputEvent) {
        match ev.kind {
            Kind::KeyDown => {
                self.bar_t = BAR_SHOW;
                let k = ev.key.as_str();
                if k == "Escape" || k == "BackSpace" {
                    if self.pending.is_some() {
                        self.close();
                    } else {
                        self.stop();
                    }
                } else if k == "space" || k == "Return" || self.b.is_action(k, "action", None) {
                    if self.at_end() {
                        self.si = 0;
                        self.pi = 0;
                        self.sound_done = false;
                        self.seek_now();
                    }
                    self.playing = !self.playing;
                    self.b.play_sound("click");
                } else if (k == "s" || k == "S") && self.pending.is_some() && !self.saved {
                    self.save_pending();
                } else if k == "Left" || self.b.is_action(k, "left", None) {
                    self.jump_scene(-1);
                } else if k == "Right" || self.b.is_action(k, "right", None) {
                    self.jump_scene(1);
                } else if k == "Up" || k == "plus" || k == "KP_Add" || self.b.is_action(k, "up", None) {
                    self.set_speed(1);
                } else if k == "Down" || k == "minus" || k == "KP_Subtract" || self.b.is_action(k, "down", None) {
                    self.set_speed(-1);
                } else if k == "e" || k == "E" {
                    let r = self.rep.clone();
                    self.export(r);
                }
            }
            Kind::MouseMove => self.bar_t = BAR_SHOW,
            Kind::MouseDown if ev.button == 1 => {
                self.bar_t = BAR_SHOW;
                if self.pending.is_some() && !self.saved && self.save_rect.hit(ev.pos) {
                    self.save_pending();
                } else if self.seek_rect.hit(ev.pos) || self.bar_rect.hit(ev.pos) {
                    if self.seek_rect.hit(ev.pos) {
                        self.seek_ratio(ev.pos.0 as f64 / 1.max(self.b.width) as f64);
                    }
                } else {
                    self.playing = !self.playing;
                }
            }
            Kind::Wheel => self.set_speed(if ev.delta > 0 { 1 } else { -1 }),
            _ => {}
        }
    }

    fn set_speed(&mut self, d: i64) {
        let idx = 0.max((SPEEDS.len() as i64 - 1).min(self.speed_idx as i64 + d)) as usize;
        if idx != self.speed_idx {
            self.speed_idx = idx;
            self.b.play_sound("move");
        }
        self.bar_t = BAR_SHOW;
    }

    fn tab_step(&mut self, d: i64) {
        let i = replay::GAMES.iter().position(|g| *g == self.tab).unwrap_or(0) as i64;
        let n = replay::GAMES.len() as i64;
        self.switch_tab(replay::GAMES[(i + d).rem_euclid(n) as usize]);
    }

    fn event_list(&mut self, ev: &InputEvent) {
        match ev.kind {
            Kind::KeyDown => {
                let k = ev.key.as_str();
                if k == "Escape" {
                    self.close();
                } else if k == "Tab" {
                    self.tab_step(1);
                } else if k == "Up" || k == "w" || self.b.is_action(k, "up", None) {
                    self.mv(-1);
                } else if k == "Down" || k == "s" || self.b.is_action(k, "down", None) {
                    self.mv(1);
                } else if k == "Left" || k == "a" {
                    self.tab_step(-1);
                } else if k == "Right" || k == "d" {
                    self.tab_step(1);
                } else if k == "Return" || k == "space" {
                    if let Some(r) = self.items.get(self.sel).cloned() {
                        self.b.play_sound("click");
                        self.start(r);
                    }
                } else if ["Delete", "KP_Delete", "x", "X"].contains(&k) {
                    self.delete_selected();
                } else if k == "e" || k == "E" {
                    self.export(None);
                } else if k == "i" || k == "I" {
                    self.import();
                }
            }
            Kind::MouseMove => {
                self.btn_hover = if self.btn_export.hit(ev.pos) {
                    Some("export")
                } else if self.btn_import.hit(ev.pos) {
                    Some("import")
                } else {
                    None
                };
                self.tab_hover = None;
                for (r, key, _) in &self.tab_rects {
                    if r.hit(ev.pos) {
                        self.tab_hover = Some(key);
                    }
                }
                self.hover_row = None;
                for (i, r) in self.row_rects.iter().enumerate() {
                    if r.hit(ev.pos) && self.first + i < self.items.len() {
                        self.hover_row = Some(i);
                        self.sel = self.first + i;
                    }
                }
            }
            Kind::MouseDown if ev.button == 1 => {
                if self.btn_export.hit(ev.pos) {
                    self.export(None);
                    return;
                }
                if self.btn_import.hit(ev.pos) {
                    self.import();
                    return;
                }
                if let Some(&(_, key, _)) = self.tab_rects.iter().find(|(r, _, _)| r.hit(ev.pos)) {
                    self.switch_tab(key);
                    return;
                }
                for i in 0..self.row_rects.len() {
                    if self.row_rects[i].hit(ev.pos) && self.first + i < self.items.len() {
                        self.sel = self.first + i;
                        self.b.play_sound("click");
                        let r = self.items[self.sel].clone();
                        self.start(r);
                        return;
                    }
                }
            }
            Kind::Wheel => self.mv(if ev.delta > 0 { -1 } else { 1 }),
            _ => {}
        }
    }

    fn mv(&mut self, d: i64) {
        if self.items.is_empty() {
            return;
        }
        self.sel = (self.sel as i64 + d).rem_euclid(self.items.len() as i64) as usize;
        self.confirm = None;
        if self.sel < self.first {
            self.first = self.sel;
        } else if self.sel >= self.first + self.rows_visible {
            self.first = self.sel + 1 - self.rows_visible;
        }
        self.b.play_sound("move");
    }

    fn switch_tab(&mut self, key: &'static str) {
        if key == self.tab {
            return;
        }
        self.tab = key;
        self.sel = 0;
        self.load();
        self.build();
        self.b.play_sound("move");
    }

    // ----- Zeichnen -------------------------------------------------------

    fn chip_y(&self) -> i32 {
        self.inst.as_ref().map(|i| i.replay_hud_h()).unwrap_or(40) + 8
    }

    fn draw_play(&mut self, s: &mut pg::Surface) {
        let (_, aiming, banner) = self.phase();
        if let Some(inst) = self.inst.as_mut() {
            inst.replay_draw(aiming, banner);
        }
        self.draw_chip(s);
        if self.pending.is_some() && (!self.saved || self.toast_t > 0.0) {
            self.draw_save_chip(s);
        }
        if self.bar_t > 0.0 || !self.playing {
            self.draw_bar(s);
        }
    }

    fn draw_chip(&self, s: &mut pg::Surface) {
        let p = ui::pal();
        let fnt = ui::fb(12);
        let img = fnt.render(&t("replay.chip"), true, p.text);
        let w = img.w() + 34;
        let h = img.h() + 10;
        let r = Rect::new(10, self.chip_y(), w, h);
        ui::draw_panel_ex(s, r, None, None, Some(h / 2), false, None);
        let col = mix(p.panel, p.red, 0.4 + 0.6 * ui::pulse_d(2.4));
        draw::circle(s, col, (r.x + 13, r.centery()), 5, 0);
        s.blr(&img, img.get_rect().midleft_at((r.x + 24, r.centery())));
    }

    fn draw_save_chip(&mut self, s: &mut pg::Surface) {
        let p = ui::pal();
        self.save_rect.y = self.chip_y() - 1;
        let r = self.save_rect;
        let done = self.saved;
        let label = if done { t("replay.saved_chip") } else { t("replay.save_chip") };
        ui::draw_panel_ex(s, r, Some(mix(p.panel, p.green, if done { 0.35 } else { 0.0 })), None, Some(14), false, None);
        draw::rect_r(s, if done { p.green } else { p.accent }, r, 1, 14);
        let img = ui::fb(13).render(&label, true, if done { p.green } else { p.text });
        s.blr(&img, img.get_rect().center_at(r.center()));
    }

    fn draw_bar(&self, s: &mut pg::Surface) {
        let p = ui::pal();
        let w = self.b.width;
        let bar = self.bar_rect;
        let mut panel = pg::surface_alpha(bar.w, bar.h);
        panel.fill_c(pg::rgba(10, 12, 18, 208));
        s.bl(&panel, (bar.x, bar.y));
        let rep = self.rep.as_ref().expect("rep");
        let game = s_of(rep, "game");
        let accent = game_accent(&game);
        draw::line(s, p.border, (0, bar.y), (w, bar.y), 1);
        let y = bar.y - 3;
        draw::rect(s, p.panel, (0, y, w, 5), 0);
        let done = self.elapsed() as f64 / 1.max(self.total) as f64;
        draw::rect(s, accent, (0, y, (w as f64 * done) as i32, 5), 0);
        for start in self.starts.iter().skip(1) {
            let x = (w as f64 * *start as f64 / 1.max(self.total) as f64) as i32;
            draw::line(s, p.border_light, (x, y), (x, y + 5), 1);
        }
        let small = ui::fb(13);
        let tiny = ui::font(11, false, false);
        let mut title_txt = s_of(rep, "title");
        if title_txt.is_empty() {
            title_txt = self.game_name(self.tab);
        }
        let title = small.render(&title_txt, true, p.text);
        s.bl(&title, (12, bar.y + 7));
        let mut sub_txt = s_of(rep, "sub");
        if rep.get("meta").and_then(|m| m.get("capped")).is_some_and(|c| truthy(Some(c))) {
            sub_txt = if sub_txt.is_empty() { t("replay.capped") } else { format!("{sub_txt}  \u{b7}  {}", t("replay.capped")) };
        }
        let sub = tiny.render(&sub_txt, true, p.text_dim);
        s.bl(&sub, (12, bar.y + 26));
        let rate = self.rate();
        let pos_txt = format!("{} / {}", replay::format_duration(self.elapsed() as f64 / rate), replay::format_duration(self.total as f64 / rate));
        let mid = small.render(&pos_txt, true, p.text_dim);
        s.blr(&mid, mid.get_rect().midtop_at((w / 2, bar.y + 7)));
        let label = tf(seq_key(&game), &[("n", &(self.si + 1).to_string()), ("total", &self.scenes.len().to_string())]);
        let seq = tiny.render(&label, true, accent);
        s.blr(&seq, seq.get_rect().midtop_at((w / 2, bar.y + 26)));
        let state = if self.playing { t("replay.playing") } else { t("replay.paused") };
        let st = small.render(&format!("{state}  {:.1}x", SPEEDS[self.speed_idx]), true, if !self.playing { p.gold } else { p.text_dim });
        s.blr(&st, st.get_rect().topright_at((w - 12, bar.y + 7)));
        let hint = tiny.render(&t("replay.hint_play"), true, p.text_faint);
        if hint.w() < w - 24 {
            s.blr(&hint, hint.get_rect().topright_at((w - 12, bar.y + 26)));
        }
    }

    fn draw_list(&self, s: &mut pg::Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        ui::draw_background_ex(s, w, h, false, None);
        ui::draw_title_ex(s, w, &t("replay.name"), Some(&t("replay.subtitle")), 26, Some(&ui::fb(30)), None, Some(p.accent2));
        for (r, key, label) in &self.tab_rects {
            let active = *key == self.tab;
            let hover = Some(*key) == self.tab_hover;
            draw::rect_r(s, if active || hover { p.panel_light } else { p.panel }, *r, 0, 13);
            draw::rect_r(s, if active { game_accent(key) } else { p.border }, *r, 1, 13);
            let img = self.tab_font.render(label, true, if active { p.text } else { p.text_dim });
            s.blr(&img, img.get_rect().center_at(r.center()));
        }
        let cnt = ui::fb(13).render(&tf("replay.count", &[("n", &self.items.len().to_string()), ("max", &replay::MAX_PER_GAME.to_string())]), true, p.text_dim);
        s.blr(&cnt, cnt.get_rect().midleft_at((self.left, self.head_y)));
        self.draw_btn(s, self.btn_export, &t("replay.btn_export"), !self.items.is_empty(), self.btn_hover == Some("export"));
        self.draw_btn(s, self.btn_import, &t("replay.btn_import"), true, self.btn_hover == Some("import"));
        if self.items.is_empty() {
            self.draw_empty(s);
        } else {
            for (i, r) in self.row_rects.iter().enumerate() {
                let idx = self.first + i;
                if idx >= self.items.len() {
                    break;
                }
                self.draw_row(s, *r, &self.items[idx], idx == self.sel);
            }
            if self.items.len() > self.rows_visible {
                self.draw_scrollbar(s);
            }
        }
        ui::draw_footer(s, w, h, &t("replay.hint_list"));
    }

    fn draw_btn(&self, s: &mut pg::Surface, r: Rect, label: &str, on: bool, hover: bool) {
        let p = ui::pal();
        draw::rect_r(s, if on && hover { p.panel_light } else { p.panel }, r, 0, 12);
        draw::rect_r(s, if on && hover { p.accent2 } else { p.border }, r, 1, 12);
        let img = ui::fb(12).render(label, true, if on { p.text } else { p.text_faint });
        s.blr(&img, img.get_rect().center_at(r.center()));
    }

    fn draw_row(&self, s: &mut pg::Surface, r: Rect, rep: &Value, selected: bool) {
        let p = ui::pal();
        let accent = game_accent(&s_of(rep, "game"));
        ui::draw_panel_ex(s, r, Some(if selected { p.panel_light } else { p.panel }), None, Some(10), false, None);
        if selected {
            draw::rect_r(s, accent, r, 1, 10);
        }
        draw::rect_r(s, accent, (r.x, r.y + 8, 3, r.h - 16), 0, 2);
        let (cx, cy) = (r.x + 30, r.centery());
        draw::circle(s, mix(p.btn, accent, 0.35), (cx, cy), 13, 0);
        draw::polygon(s, if selected { p.text } else { p.text_dim }, &[(cx - 4, cy - 6), (cx + 7, cy), (cx - 4, cy + 6)], 0);
        let title = ui::fb(16).render(&s_of(rep, "title"), true, if selected { p.text } else { p.text_dim });
        s.bl(&title, (r.x + 52, r.y + 8));
        if s_of(rep, "src") == "file" {
            let ax = r.x + 58 + title.w();
            let ay = r.y + 14;
            draw::line(s, p.accent2, (ax, ay), (ax, ay + 8), 2);
            draw::polygon(s, p.accent2, &[(ax - 4, ay + 6), (ax + 4, ay + 6), (ax, ay + 11)], 0);
        }
        let sub = ui::font(12, false, false).render(&s_of(rep, "sub"), true, p.text_faint);
        s.bl(&sub, (r.x + 52, r.y + 28));
        let dur = ui::fb(13).render(&replay::format_duration(replay::duration(rep)), true, accent);
        s.blr(&dur, dur.get_rect().topright_at((r.right() - 14, r.y + 8)));
        let mut stamp = s_of(rep, "date");
        let by = s_of(rep, "by");
        if !by.is_empty() {
            stamp = format!("{by}  \u{b7}  {stamp}");
        }
        let date = ui::font(11, false, false).render(&stamp, true, p.text_faint);
        s.blr(&date, date.get_rect().topright_at((r.right() - 14, r.y + 28)));
        if self.confirm.as_deref() == Some(s_of(rep, "id").as_str()) {
            let warn = ui::fb(11).render(&t("replay.confirm_short"), true, p.red);
            s.blr(&warn, warn.get_rect().midright_at((r.right() - 70, r.centery())));
        }
    }

    fn draw_empty(&self, s: &mut pg::Surface) {
        let p = ui::pal();
        let cx = self.b.width / 2;
        let cy = (self.top + self.bottom) / 2;
        let head = ui::fb(17).render(&tf("replay.empty", &[("game", &self.game_name(self.tab))]), true, p.text_dim);
        s.blr(&head, head.get_rect().center_at((cx, cy - 18)));
        let hint = ui::font(13, false, false).render(&t("replay.empty_hint"), true, p.text_faint);
        s.blr(&hint, hint.get_rect().center_at((cx, cy + 10)));
        if !shell::replay_enabled(&shell::settings()) {
            let off = ui::fb(13).render(&t("replay.disabled"), true, p.gold);
            s.blr(&off, off.get_rect().center_at((cx, cy + 38)));
        }
        if let Some(err) = &self.error {
            let e = ui::fb(13).render(err, true, p.red);
            s.blr(&e, e.get_rect().center_at((cx, cy + 62)));
        }
    }

    fn draw_scrollbar(&self, s: &mut pg::Surface) {
        let p = ui::pal();
        let (top, bottom) = (self.top, self.top + self.rows_visible as i32 * ROW_H);
        let track = Rect::new(self.right + 6, top, 4, bottom - top);
        draw::rect_r(s, p.panel, track, 0, 2);
        let frac = self.rows_visible as f64 / self.items.len() as f64;
        let hh = 24.max((track.h as f64 * frac) as i32);
        let pos = self.first as f64 / 1.max(self.items.len() as i64 - self.rows_visible as i64) as f64;
        let y = top + ((track.h - hh) as f64 * 1f64.min(pos)) as i32;
        draw::rect_r(s, p.border_light, (track.x, y, 4, hh), 0, 2);
    }

    fn draw_toast(&self, s: &mut pg::Surface) {
        let Some(toast) = &self.toast else { return };
        let p = ui::pal();
        let img = ui::fb(14).render(toast, true, p.text);
        let (w, h) = (img.w() + 30, img.h() + 14);
        let y = self.b.height - h - if self.mode == "play" { BAR_H + 14 } else { 46 };
        let r = Rect::new(self.b.width / 2 - w / 2, y, w, h);
        ui::draw_panel_ex(s, r, None, None, Some(h / 2), false, None);
        draw::rect_r(s, p.accent2, r, 1, h / 2);
        s.blr(&img, img.get_rect().center_at(r.center()));
    }
}

impl Game for ReplayScreen {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }
    fn reset(&mut self) {}
    fn screen_name(&self) -> String {
        t("replay.name")
    }
    fn on_surface_changed(&mut self) {
        self.build();
        if self.inst.is_some() {
            self.seek_now();
        }
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if self.mode == "play" {
            self.event_play(ev);
        } else {
            self.event_list(ev);
        }
    }

    fn update(&mut self, dt: f64) {
        if self.toast_t > 0.0 {
            self.toast_t -= dt;
            if self.toast_t <= 0.0 {
                self.toast = None;
            }
        }
        if self.mode != "play" || self.inst.is_none() {
            return;
        }
        if self.bar_t > 0.0 {
            self.bar_t -= dt;
        }
        if !self.playing {
            return;
        }
        self.acc += dt * SPEEDS[self.speed_idx];
        let step = 1.0 / self.rate();
        let mut steps = 0;
        while self.acc >= step && steps < 240 {
            self.acc -= step;
            steps += 1;
        }
        if steps > 0 {
            self.advance(steps);
        }
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        if self.mode == "play" && self.inst.is_some() {
            self.draw_play(&mut s);
        } else {
            self.draw_list(&mut s);
        }
        self.draw_toast(&mut s);
    }
}
