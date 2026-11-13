//! app.rs
//! ======
//! Gegenstueck zu `main.App`: die Schleife, der Startbildschirm (Logo,
//! Spiele-Pillen, Highscore-Laufband), Pause, Highscore-Banner,
//! Statuskarte und die Darstellung im Fenster.
//!
//! In Python treibt Tkinter alles an (`root.after` ruft `_loop`). Hier gibt
//! es eine gewoehnliche Schleife; was `_frame` je Bild tut, steht in
//! [`App::frame`] in derselben Reihenfolge.

use pgz_core::i18n::{t, tf};
use pgz_core::{achievements, highscore, stats};
use pgz_engine::game::{Game, GameBase, GameInfo, InputEvent, Kind};
use pgz_engine::pg::{self, draw, transform, Color, Rect, RectExt, SurfExt, Surface, BLEND_ADD, BLEND_RGBA_MULT};
use pgz_engine::shell::{self, AppInfo, Cmd};
use pgz_engine::{achv, audio, py, ui};

use crate::screens;
use crate::sidebar::{tausender, Aktion, Sidebar};


/// Das Programm.
pub struct App {
    pub games: Vec<GameInfo>,
    pub game_w: i32,
    pub game_h: i32,
    pub fps: i32,
    pub canvas: Surface,
    pub current: Option<Box<dyn Game>>,
    pub sidebar: Sidebar,
    menu_hover: Option<usize>,
    menu_tiles: Vec<(Rect, usize)>,
    ticker_key: Option<Vec<(String, i64)>>,
    ticker_img: Option<Surface>,
    ticker_band: Option<Surface>,
    logo_raw: Option<Surface>,
    logo_cache: Option<(i32, Option<Surface>)>,
    fps_n: i32,
    fps_t0: u64,
    pub closing: bool,
    /// Groesse der Anzeigeflaeche rechts der Leiste.
    pub disp_w: i32,
    pub disp_h: i32,
    scale: f64,
    off: (i32, i32),
    pub gehaltene: Vec<String>,
    /// Das Spiel, in das eine Wiederholung (Taste P) zurueckkehrt.
    parked: Option<Box<dyn Game>>,
    /// `toggle_fullscreen()` angefordert - die Fensterschleife schaltet um.
    vollbild_wunsch: bool,
}

impl App {
    /// `App.__init__` ohne Fenster.
    pub fn new(games: Vec<GameInfo>) -> App {
        let st = shell::settings();
        let (gw, gh) = st.resolution;
        ui::set_theme(&st.theme);
        let mut app = App {
            games,
            game_w: gw.max(1),
            game_h: gh.max(1),
            fps: st.fps.clamp(5, 240),
            canvas: pg::surface(gw.max(1), gh.max(1)),
            current: None,
            parked: None,
            vollbild_wunsch: false,
            sidebar: Sidebar::new(),
            menu_hover: None,
            menu_tiles: Vec::new(),
            ticker_key: None,
            ticker_img: None,
            ticker_band: None,
            logo_raw: pg::image::load_bytes(include_bytes!("../../../../logo/pygamez3-256.png")),
            logo_cache: None,
            fps_n: 0,
            fps_t0: 0,
            closing: false,
            disp_w: gw,
            disp_h: gh,
            scale: 1.0,
            off: (0, 0),
            gehaltene: Vec::new(),
        };
        app.sidebar.fit(&app.games);
        app.info_setzen();
        achievements::backfill_silent();
        if !pgz_core::i18n::has_language() {
            app.open_welcome();
        } else {
            app.maybe_show_level_intro();
        }
        app
    }

    fn info_setzen(&self) {
        shell::set_info(AppInfo { game_w: self.game_w, game_h: self.game_h, fps: self.fps });
    }

    /// Basisdaten fuer einen Menue-Bildschirm (`_Screen`).
    pub fn menu_base(&self) -> GameBase {
        let mut b = GameBase::new(self.canvas.clone(), self.game_w, self.game_h, "single", shell::settings(), "_menu", "");
        b.is_menu = true;
        b
    }

    fn game_index(&self, key: &str) -> Option<usize> {
        self.games.iter().position(|g| g.key == key)
    }

    // ----- Bildschirme wechseln ----------------------------------------------

    fn switch_current(&mut self, new: Option<Box<dyn Game>>, exit_old: bool) {
        self.release_keys();
        if let Some(old) = self.current.as_mut() {
            if exit_old {
                old.on_exit();
            }
        }
        self.current = new;
        let id = self.current.as_ref().map(|g| g.ident());
        let paused = self.current.as_ref().map(|g| g.base().paused).unwrap_or(false);
        audio::music_tick(id, paused, &shell::settings());
    }

    /// `show_screen(screen)`
    pub fn show_screen(&mut self, screen: Box<dyn Game>) {
        if let Some(old) = self.current.as_mut() {
            if !old.base().is_menu && !old.base().hs_saved {
                Self::highscore_speichern(old.as_mut());
            }
        }
        self.switch_current(Some(screen), true);
        self.menu_hover = None;
        ui::begin_transition(None);
    }

    /// `spiel_starten(cls)` - Vorspiel-Bildschirm.
    pub fn spiel_starten(&mut self, idx: usize) {
        self.sidebar.aktiv = Some(idx);
        let info = self.games[idx];
        let s = screens::menu::PreGameScreen::new(self.menu_base(), info);
        self.show_screen(Box::new(s));
    }

    /// `launch_game(cls, mode, p2)`
    pub fn launch_game(&mut self, idx: usize, mode: &str, p2: Option<String>) {
        self.switch_current(None, true);
        let info = self.games[idx];
        let mut base = GameBase::new(self.canvas.clone(), self.game_w, self.game_h, mode, shell::settings(), info.key, info.class_name);
        base.player_ids = [None, p2];
        let mut game = (info.make)(base);
        game.base_mut().paused = false;
        self.switch_current(Some(game), true);
        self.sidebar.aktiv = Some(idx);
        stats::game_started(info.key);
        achievements::on_game_started(info.key);
        ui::begin_transition(None);
    }

    /// `back_to_menu()`
    pub fn back_to_menu(&mut self) {
        self.switch_current(None, true);
        self.sidebar.status = t("app.no_game");
        self.sidebar.punkt = ui::tk_colors().text_dim;
        self.sidebar.aktiv = None;
        self.sidebar.refresh_scores();
        ui::begin_transition(None);
    }

    /// `zum_menue()` (Python: `zum_men\u00fc`)
    pub fn zum_menue(&mut self) {
        if let Some(cur) = self.current.as_mut() {
            Self::highscore_speichern(cur.as_mut());
        }
        stats::end_session();
        stats::flush();
        self.switch_current(None, true);
        self.sidebar.status = t("app.no_game");
        self.sidebar.punkt = ui::tk_colors().text_dim;
        self.sidebar.aktiv = None;
        self.sidebar.refresh_scores();
        ui::begin_transition(None);
    }

    fn open_welcome(&mut self) {
        let s = screens::menu::WelcomeScreen::new(self.menu_base());
        self.show_screen(Box::new(s));
    }

    fn maybe_show_level_intro(&mut self) {
        let mut flags = pgz_core::store::load_section("progress");
        if pgz_core::store::get_bool(&flags, "intro_seen", false) {
            return;
        }
        flags.insert("intro_seen".into(), true.into());
        pgz_core::store::save_section("progress", flags);
        let (_level, points) = stats::level_of(None);
        if points <= 0 {
            return;
        }
        if let Some(s) = screens::level_intro(self) {
            self.show_screen(s);
        }
    }

    /// `apply_resolution(w, h, persist)`
    pub fn apply_resolution(&mut self, w: i32, h: i32, persist: bool) {
        self.game_w = w.max(1);
        self.game_h = h.max(1);
        if persist {
            shell::edit_settings(|s| s.resolution = (self.game_w, self.game_h));
            shell::save_settings();
        }
        self.canvas = pg::surface(self.game_w, self.game_h);
        if let Some(cur) = self.current.as_mut() {
            let b = cur.base_mut();
            b.surface = self.canvas.clone();
            b.width = self.game_w;
            b.height = self.game_h;
            cur.on_surface_changed();
        }
        self.info_setzen();
    }

    fn match_resolution_to_window(&mut self) {
        if (self.game_w, self.game_h) != (self.disp_w, self.disp_h) {
            self.apply_resolution(self.disp_w, self.disp_h, false);
        }
    }

    /// `set_auto_resolution(on)`
    pub fn set_auto_resolution(&mut self, on: bool) {
        shell::edit_settings(|s| s.auto_resolution = on);
        if on {
            self.match_resolution_to_window();
        } else {
            let (w, h) = shell::settings().resolution;
            self.apply_resolution(w, h, true);
        }
    }

    /// `sync_display_size` - Groesse der Anzeigeflaeche hat sich geaendert.
    pub fn sync_display_size(&mut self, w: i32, h: i32) {
        let (w, h) = (w.max(1), h.max(1));
        if (w, h) == (self.disp_w, self.disp_h) {
            return;
        }
        self.disp_w = w;
        self.disp_h = h;
        if shell::settings().auto_resolution {
            self.match_resolution_to_window();
        }
    }

    // ----- Auftraege abarbeiten -----------------------------------------------

    /// Fuehrt die abgelegten Auftraege der Bildschirme aus.
    pub fn befehle(&mut self) {
        for c in shell::take() {
            match c {
                Cmd::PreGame(key) => {
                    if let Some(i) = self.game_index(&key) {
                        self.spiel_starten(i);
                    }
                }
                Cmd::Launch { key, mode, p2 } => {
                    if let Some(i) = self.game_index(&key) {
                        self.launch_game(i, &mode, p2);
                    }
                }
                Cmd::PlayerPick { key, mode } => {
                    if let Some(i) = self.game_index(&key) {
                        match screens::player_pick(self, i, &mode) {
                            Some(s) => self.show_screen(s),
                            None => self.launch_game(i, &mode, None),
                        }
                    }
                }
                Cmd::BackToMenu => self.back_to_menu(),
                Cmd::ToMenu => self.zum_menue(),
                Cmd::Options(back) => {
                    let zurueck = match back {
                        Some(k) => Cmd::PreGame(k),
                        None => Cmd::BackToMenu,
                    };
                    let s = screens::menu::OptionsScreen::new(self.menu_base(), zurueck);
                    self.show_screen(Box::new(s));
                }
                Cmd::Language => {
                    let s = screens::menu::LanguageScreen::new(self.menu_base());
                    self.show_screen(Box::new(s));
                }
                Cmd::Welcome => self.open_welcome(),
                Cmd::Progress(tab) => {
                    if let Some(s) = screens::progress(self, tab) {
                        self.show_screen(s);
                    }
                }
                Cmd::Profiles => {
                    if let Some(s) = screens::profiles(self) {
                        self.show_screen(s);
                    }
                }
                Cmd::Wiki(page, back) => {
                    if let Some(s) = screens::wiki(self, page, back) {
                        self.show_screen(s);
                    }
                }
                Cmd::Replays => {
                    if let Some(s) = screens::replays(self) {
                        self.show_screen(s);
                    }
                }
                Cmd::ReturnFromReplay => match self.parked.take() {
                    Some(g) => self.show_screen(g),
                    None => self.back_to_menu(),
                },
                Cmd::ApplyTheme(name) => {
                    ui::set_theme(&name);
                    shell::edit_settings(|s| s.theme = name.clone());
                    shell::save_settings();
                    self.ticker_key = None;
                    self.ticker_band = None;
                }
                Cmd::ApplyResolution(w, h, persist) => self.apply_resolution(w, h, persist),
                Cmd::SetAutoResolution(on) => self.set_auto_resolution(on),
                Cmd::ApplyFps(f) => {
                    self.fps = f.clamp(5, 240);
                    shell::edit_settings(|s| s.fps = self.fps);
                    shell::save_settings();
                    self.info_setzen();
                }
                Cmd::RefreshLanguage => {
                    self.ticker_key = None;
                    self.sidebar.fit(&self.games);
                    if self.current.is_none() {
                        self.sidebar.status = t("app.no_game");
                    }
                }
                Cmd::RefreshProfile => {
                    achievements::backfill_silent();
                    self.sidebar.refresh_scores();
                    self.ticker_key = None;
                    self.ticker_band = None;
                }
                Cmd::ToggleFullscreen => self.vollbild_wunsch = true,
                Cmd::Quit => self.beenden(),
            }
        }
    }

    // ----- Eingabe --------------------------------------------------------

    /// `_on_key` - ein Tastendruck aus dem Fenster.
    pub fn key_down(&mut self, key: &str, ch: &str, repeat: bool) {
        // F11 schaltet den Vollbildmodus um (Spiel bleibt eingebettet).
        if key == "F11" {
            if !repeat {
                self.vollbild_wunsch = true;
            }
            return;
        }
        if key == "Escape" {
            if repeat {
                return;
            }
            let Some(cur) = self.current.as_mut() else { return };
            if cur.base().is_menu || cur.wants_escape() {
                cur.handle_event(&InputEvent::key_down("Escape", "", false));
            } else if !cur.base().game_over {
                let pausieren = !cur.base().paused;
                if pausieren {
                    self.release_keys();
                }
                if let Some(cur) = self.current.as_mut() {
                    cur.base_mut().paused = pausieren;
                }
            }
            return;
        }
        if !repeat && !self.gehaltene.iter().any(|k| k == key) {
            self.gehaltene.push(key.to_string());
        }
        if let Some(cur) = self.current.as_mut() {
            if !cur.base().paused {
                cur.handle_event(&InputEvent::key_down(key, ch, repeat));
            }
        }
    }

    /// Eine losgelassene Taste.
    pub fn key_up(&mut self, key: &str) {
        self.gehaltene.retain(|k| k != key);
        if key == "Escape" || key == "F11" {
            return;
        }
        if let Some(cur) = self.current.as_mut() {
            if !cur.base().paused {
                cur.handle_event(&InputEvent::key_up(key));
            }
        }
    }

    /// `_release_keys` - alle gehaltenen Tasten loslassen.
    pub fn release_keys(&mut self) {
        let held = std::mem::take(&mut self.gehaltene);
        if let Some(cur) = self.current.as_mut() {
            if !cur.base().paused {
                for k in held {
                    if k != "Escape" && k != "F11" {
                        cur.handle_event(&InputEvent::key_up(&k));
                    }
                }
            }
        }
    }

    /// Fensterkoordinaten der Spielflaeche -> logische Koordinaten.
    pub fn to_logical(&self, x: i32, y: i32) -> (i32, i32) {
        let s = if self.scale == 0.0 { 1.0 } else { self.scale };
        let lx = ((x - self.off.0) as f64 / s).clamp(0.0, (self.game_w - 1) as f64);
        let ly = ((y - self.off.1) as f64 / s).clamp(0.0, (self.game_h - 1) as f64);
        (lx as i32, ly as i32)
    }

    /// Mausklick auf der Spielflaeche (Fensterkoordinaten rechts der Leiste).
    pub fn mouse_down(&mut self, x: i32, y: i32, button: i32) {
        let pos = self.to_logical(x, y);
        let Some(cur) = self.current.as_mut() else {
            if button == 1 {
                self.menu_click(pos);
            }
            return;
        };
        if cur.base().paused {
            return;
        }
        if button == 1 {
            cur.handle_event(&InputEvent::mouse(Kind::MouseDown, pos, 1));
        } else if button == 3 && cur.wants_right_click() {
            cur.handle_event(&InputEvent::mouse(Kind::MouseDown, pos, 3));
        }
    }

    /// Maustaste losgelassen.
    pub fn mouse_up(&mut self, x: i32, y: i32, button: i32) {
        let pos = self.to_logical(x, y);
        let Some(cur) = self.current.as_mut() else { return };
        if cur.base().paused {
            return;
        }
        if button == 1 || (button == 3 && cur.wants_right_click()) {
            cur.handle_event(&InputEvent::mouse(Kind::MouseUp, pos, button));
        }
    }

    /// Pruefstand: ein Ereignis in Spielkoordinaten direkt zustellen
    /// (wie `game.handle_event` in `tools/vergleich.py`).
    pub fn spiel_ereignis(&mut self, ev: &InputEvent) {
        if let Some(cur) = self.current.as_mut() {
            if !cur.base().paused {
                cur.handle_event(ev);
            }
        } else if ev.kind == Kind::MouseMove {
            // Startbildschirm: Hover im Spiele-Raster (wie `mouse_move`).
            self.menu_motion(ev.pos);
        }
    }

    /// Maus bewegt.
    pub fn mouse_move(&mut self, x: i32, y: i32) {
        let pos = self.to_logical(x, y);
        let Some(cur) = self.current.as_mut() else {
            self.menu_motion(pos);
            return;
        };
        if cur.base().paused {
            return;
        }
        cur.handle_event(&InputEvent::mouse(Kind::MouseMove, pos, 1));
    }

    /// Relative Bewegung (Pointer-Capture).
    pub fn mouse_rel(&mut self, dx: i32, dy: i32) {
        if let Some(cur) = self.current.as_mut() {
            if !cur.base().paused {
                let mut ev = InputEvent::new(Kind::MouseRel);
                ev.rel = (dx.clamp(-150, 150), dy.clamp(-150, 150));
                cur.handle_event(&ev);
            }
        }
    }

    /// Mausrad.
    pub fn wheel(&mut self, x: i32, y: i32, delta: i32) {
        let pos = self.to_logical(x, y);
        if let Some(cur) = self.current.as_mut() {
            if !cur.base().paused {
                let mut ev = InputEvent::mouse(Kind::Wheel, pos, 1);
                ev.delta = delta;
                cur.handle_event(&ev);
            }
        }
    }

    /// Will das aktive Spiel den Zeiger einfangen?
    pub fn wants_capture(&self) -> bool {
        self.current.as_ref().map(|c| !c.base().paused && c.capture_mouse()).unwrap_or(false)
    }

    /// Klick in der Seitenleiste.
    pub fn seitenleiste(&mut self, a: Aktion) {
        audio::play("click", Some(&shell::settings()));
        match a {
            Aktion::Spiel(i) => self.spiel_starten(i),
            Aktion::Profil => shell::post(Cmd::Profiles),
            Aktion::Knopf(k) => match k.as_str() {
                "quit" => self.beenden(),
                "back_to_menu" => self.zum_menue(),
                "options" => shell::post(Cmd::Options(None)),
                "language" => shell::post(Cmd::Language),
                "profiles" => shell::post(Cmd::Profiles),
                "progress" => shell::post(Cmd::Progress(None)),
                "lamawiki" => shell::post(Cmd::Wiki(None, None)),
                "replays" => shell::post(Cmd::Replays),
                "fullscreen" => shell::post(Cmd::ToggleFullscreen),
                _ => {}
            },
        }
    }

    // ----- Highscores -------------------------------------------------------

    fn highscore_speichern(game: &mut dyn Game) {
        let b = game.base_mut();
        if b.is_menu {
            return;
        }
        let (hs, rekord) = highscore::update(b.key, b.score);
        b.hs_value = hs;
        b.hs_record = rekord;
        if rekord {
            stats::record_broken(b.key);
        }
        stats::score_result(b.key, b.score, rekord);
        achievements::on_highscore(b.key, hs, false);
        achievements::check_stats(false);
    }

    fn draw_highscore_overlay(&mut self) {
        let Some(cur) = self.current.as_ref() else { return };
        let b = cur.base();
        let (hs, record, score) = (b.hs_value, b.hs_record, b.score);
        let p = ui::pal();
        let mut s = self.canvas.clone();
        if ui::fx().style == "v1" {
            let (text, farbe) = if record {
                (tf("app.new_highscore", &[("score", &score.to_string())]), Color::rgb(255, 215, 90))
            } else {
                (tf("app.highscore", &[("hs", &hs.to_string())]), Color::rgb(200, 205, 220))
            };
            let img = ui::font(20, true, true).render(&text, true, farbe);
            s.blr(&img, img.get_rect().center_at((self.game_w / 2, self.game_h - 16)));
            return;
        }
        let fnt = ui::fb(18);
        let img = if record {
            ui::grad_text(
                &fnt,
                &tf("app.new_highscore", &[("score", &score.to_string())]),
                Some(Color::rgb(255, 235, 170)),
                Some(Color::rgb(235, 175, 70)),
            )
        } else {
            fnt.render(&tf("app.highscore", &[("hs", &hs.to_string())]), true, p.text_dim)
        };
        let (bw, bh) = (img.w() + 36, img.h() + 14);
        let banner = Rect::new(self.game_w / 2 - bw / 2, self.game_h - bh - 10, bw, bh);
        ui::draw_panel_ex(&mut s, banner, None, None, Some(bh / 2), false, None);
        if record {
            let mut glow = pg::surface_alpha(bw + 8, bh + 8);
            draw::rect_r(&mut glow, pg::alpha(p.gold, (90.0 * ui::pulse(3.0, 0.35, 1.0)) as i32), (0, 0, bw + 8, bh + 8), 0, bh / 2 + 4);
            s.bl(&glow, (banner.x - 4, banner.y - 4));
            draw::rect_r(&mut s, p.gold, banner, 1, bh / 2);
        }
        s.blr(&img, img.get_rect().center_at(banner.center()));
    }

    // ----- Ein Bild -------------------------------------------------------

    /// `_frame(dt)` - aktualisieren und zeichnen (ohne Anzeige).
    pub fn frame(&mut self, dt: f64) {
        self.befehle();
        // Live-FPS fuer die Statuskarte.
        self.fps_n += 1;
        let now_ms = pg::ticks();
        if now_ms.saturating_sub(self.fps_t0) >= 500 {
            if self.fps_t0 != 0 {
                let fps = self.fps_n as f64 * 1000.0 / (now_ms - self.fps_t0) as f64;
                self.sidebar.perf = format!("{}x{}   \u{b7}   {:.0}/{} FPS", self.game_w, self.game_h, fps, self.fps);
            }
            self.fps_t0 = now_ms;
            self.fps_n = 0;
        }

        if self.current.is_none() {
            self.draw_menu_screen();
        } else {
            self.frame_game(dt);
        }

        let id = self.current.as_ref().map(|g| g.ident());
        let paused = self.current.as_ref().map(|g| g.base().paused).unwrap_or(false);
        audio::music_tick(id, paused, &shell::settings());
        stats::maybe_flush();
        let mut c = self.canvas.clone();
        ui::draw_fx(&mut c, self.game_w, self.game_h, dt);
        achv::push_toasts();
        achv::draw_toasts(&mut c, self.game_w, self.game_h, dt, &shell::settings());
        self.befehle();
    }

    /// Holt einen Vollbild-Wunsch ab (F11 oder Knopf in der Seitenleiste).
    pub fn take_vollbild(&mut self) -> bool {
        std::mem::take(&mut self.vollbild_wunsch)
    }

    /// `open_replay(rep, back_to=game)` - frische Aufnahme zeigen; das Spiel
    /// bleibt geparkt (kein `on_exit`) und kehrt danach zurueck.
    fn open_replay(&mut self, rep: serde_json::Value) {
        let game = rep.get("game").and_then(|g| g.as_str()).map(String::from);
        self.parked = self.current.take();
        let s = screens::replayview::ReplayScreen::new(self.menu_base(), self.games.clone(), Some(rep), game, true);
        self.show_screen(Box::new(s));
    }

    fn frame_game(&mut self, dt: f64) {
        // main._frame: die Spiele mit Aufzeichnung koennen am Rundenende die
        // Wiederholung anfordern (Taste P) - der Replay-Screen wird aktiv.
        if let Some(rep) = self.current.as_mut().and_then(|g| g.take_replay_request()) {
            self.open_replay(rep);
        }
        let Some(game) = self.current.as_mut() else { return };
        let is_menu = game.base().is_menu;
        if !game.base().paused && !game.base().game_over {
            game.update(dt);
            if !is_menu {
                stats::add_playtime(game.base().key, dt);
                let st = stats::session_time();
                if st >= 3600.0 {
                    achievements::on_playtime(st);
                }
            }
        }
        game.draw();
        let paused = game.base().paused;
        if paused {
            self.draw_pause_overlay();
        }
        self.update_status();
        let Some(game) = self.current.as_mut() else { return };
        if game.base().game_over && !game.base().hs_saved {
            Self::highscore_speichern(game.as_mut());
            game.base_mut().hs_saved = true;
            if game.base().hs_record {
                ui::spawn_confetti(self.game_w as f64, self.game_h as f64, 90);
            }
        }
        let b = game.base_mut();
        if b.game_over {
            b.was_over = true;
        } else {
            b.hs_saved = false;
            if b.resume_same_game {
                b.resume_same_game = false;
                b.was_over = false;
            } else if b.was_over && !is_menu {
                b.was_over = false;
                b.result_reported = false;
                b.duel_reported = false;
                stats::game_started(b.key);
                achievements::on_game_started(b.key);
            }
        }
        let show = game.base().game_over && !is_menu && game.show_highscore_banner();
        if show {
            self.draw_highscore_overlay();
        }
    }

    fn update_status(&mut self) {
        let Some(game) = self.current.as_ref() else { return };
        let tk = ui::tk_colors();
        let b = game.base();
        if b.is_menu {
            self.sidebar.status = tf("app.status_menu", &[("name", &game.screen_name())]);
            self.sidebar.punkt = tk.accent;
            return;
        }
        let hs = highscore::get(b.key);
        let (zustand, dot) = if b.paused {
            (t("app.state_pause"), tk.gold)
        } else if b.game_over {
            (t("app.state_over"), tk.red)
        } else {
            (t("app.state_running"), tk.green)
        };
        let name = self.games.iter().find(|g| g.key == b.key).map(|g| g.name()).unwrap_or_default();
        self.sidebar.punkt = dot;
        self.sidebar.status = tf(
            "app.status",
            &[("name", &name), ("state", &zustand), ("score", &b.score.to_string()), ("hs", &hs.to_string())],
        );
    }

    fn draw_pause_overlay(&mut self) {
        let p = ui::pal();
        let (w, h) = (self.game_w, self.game_h);
        let mut s = self.canvas.clone();
        if ui::fx().style == "v1" {
            let mut ov = pg::surface_alpha(w, h);
            ov.fill_c(Color::rgba(0, 0, 0, 150));
            s.bl(&ov, (0, 0));
            let img = ui::font(48, true, true).render(&t("app.pause"), true, Color::rgb(240, 240, 240));
            s.blr(&img, img.get_rect().center_at((w / 2, h / 2)));
            let img2 = ui::font(16, false, true).render(&t("app.pause_resume"), true, Color::rgb(200, 200, 200));
            s.blr(&img2, img2.get_rect().center_at((w / 2, h / 2 + 40)));
            return;
        }
        let small = transform::smoothscale(&s, 1.max(w / 10), 1.max(h / 10));
        s.bl(&transform::smoothscale(&small, w, h), (0, 0));
        let mut ov = pg::surface_alpha(w, h);
        ov.fill_c(Color::rgba(8, 10, 18, 140));
        s.bl(&ov, (0, 0));
        let (cw, ch) = (360.min(w - 40), 150);
        let card = Rect::new(w / 2 - cw / 2, h / 2 - ch / 2, cw, ch);
        ui::draw_panel_ex(&mut s, card, None, None, Some(14), true, None);
        if ui::fx().grad3.is_some() {
            ui::draw_grad_line(&mut s, card.centerx(), card.y + 1, card.w - 28, 3, None, true);
        } else {
            draw::rect_corners(&mut s, p.accent, (card.x, card.y, card.w, 4), 0, 0, 14, 14, -1, -1);
        }
        let img = if ui::is_modern() || !ui::fx().title_grad {
            ui::fb(42).render(&t("app.pause"), true, p.text)
        } else {
            draw::rect_r(&mut s, ui::mix(p.border, p.accent, ui::pulse(2.0, 0.35, 1.0)), card.inflate(8, 8), 1, 16);
            ui::grad_text(&ui::fb(42), &t("app.pause"), None, None)
        };
        s.blr(&img, img.get_rect().center_at((card.centerx(), card.centery() - 18)));
        let mut img2 = ui::f(16).render(&t("app.pause_resume"), true, p.text_dim);
        if !ui::is_modern() {
            img2.alpha_set((255.0 * ui::pulse(2.0, 0.5, 1.0)) as i32);
        }
        s.blr(&img2, img2.get_rect().center_at((card.centerx(), card.centery() + 34)));
    }

    // ----- Startbildschirm -------------------------------------------------

    fn menu_logo(&mut self, size: i32) -> Option<Surface> {
        if let Some((s, l)) = &self.logo_cache {
            if *s == size {
                return l.clone();
            }
        }
        let out = self.logo_raw.as_ref().map(|raw| {
            let scaled = transform::smoothscale(raw, size, size);
            let mut out = pg::surface_alpha(size, size);
            out.bl(&scaled, (0, 0));
            let mut mask = pg::surface_alpha(size, size);
            let r = 12.max(size / 8);
            draw::rect_r(&mut mask, Color::rgba(255, 255, 255, 255), (0, 0, size, size), 0, r);
            out.bl_flags(&mask, (0, 0), BLEND_RGBA_MULT);
            out
        });
        self.logo_cache = Some((size, out.clone()));
        out
    }

    fn draw_score_ticker(&mut self, s: &mut Surface, w: i32, h: i32) {
        let p = ui::pal();
        let mut entries: Vec<(String, i64)> = self
            .games
            .iter()
            .map(|g| (g.name(), self.sidebar.scores.get(g.key).copied().unwrap_or(0)))
            .filter(|(_, v)| *v > 0)
            .collect();
        if entries.is_empty() {
            return;
        }
        entries.sort_by_key(|e| -e.1);
        let key: Vec<(String, i64)> = entries.into_iter().take(10).collect();
        if self.ticker_key.as_ref() != Some(&key) {
            let teile: Vec<String> = key.iter().map(|(n, v)| format!("{}  {}", n.to_uppercase(), tausender(*v))).collect();
            let text = format!("{}   \u{2014}   {}", t("app.top_scores").to_uppercase(), teile.join("   \u{b7}   "));
            self.ticker_img = Some(ui::fb(14).render(&text, true, p.text_dim));
            self.ticker_key = Some(key);
        }
        let Some(img) = self.ticker_img.clone() else { return };
        let band_h = 26;
        let band_y = h - 64;
        if self.ticker_band.as_ref().map(|b| b.w() != w).unwrap_or(true) {
            let mut band = pg::surface_alpha(w, band_h);
            band.fill_c(pg::alpha(p.bg_top, 135));
            self.ticker_band = Some(band);
        }
        if let Some(band) = &self.ticker_band {
            s.bl(band, (0, band_y));
        }
        let line_col = ui::mix(p.bg_bottom, p.accent, 0.35);
        draw::line(s, line_col, (0, band_y), (w, band_y), 1);
        draw::line(s, line_col, (0, band_y + band_h), (w, band_y + band_h), 1);
        let (speed, gap) = (42.0, 140);
        let span = img.w() + gap;
        let off = py::fpmod(pg::ticks() as f64 / 1000.0 * speed, span as f64) as i32;
        let y = band_y + band_h / 2 - img.h() / 2;
        let prev = s.get_clip();
        s.set_clip(Some(Rect::new(0, band_y, w, band_h)));
        let mut x = -off;
        while x < w {
            s.bl(&img, (x, y));
            x += span;
        }
        s.set_clip(Some(prev));
    }

    fn flow_menu_tiles(&self, w: i32, avail: i32) -> (Vec<Vec<(usize, i32)>>, pg::Font, i32, i32, i32, i32) {
        let max_w = w - 60;
        let mut result = None;
        for fsize in [15, 13, 11] {
            let fnt = ui::f(fsize);
            let pad_x = fsize - 4;
            let row_h = fnt.get_height() + 12;
            let gap = 8;
            let mut rows: Vec<Vec<(usize, i32)>> = vec![Vec::new()];
            let mut rw = 0;
            for (i, g) in self.games.iter().enumerate() {
                let tw = fnt.width(&g.name()) + pad_x * 2 + 16;
                let mut add = tw + if rows.last().map(|r| !r.is_empty()).unwrap_or(false) { gap } else { 0 };
                if rw + add > max_w && rows.last().map(|r| !r.is_empty()).unwrap_or(false) {
                    rows.push(Vec::new());
                    rw = 0;
                    add = tw;
                }
                if let Some(r) = rows.last_mut() {
                    r.push((i, tw));
                }
                rw += add;
            }
            let n = rows.len() as i32;
            let total_h = n * row_h + (n - 1) * gap;
            let fertig = total_h <= avail;
            result = Some((rows, fnt, pad_x, row_h, gap, total_h));
            if fertig {
                break;
            }
        }
        result.expect("mindestens eine Schriftgroesse")
    }

    fn draw_menu_screen(&mut self) {
        let p = ui::pal();
        let fx = ui::fx();
        let (w, h) = (self.game_w, self.game_h);
        let mut s = self.canvas.clone();
        ui::draw_background(&mut s, w, h);
        let modern = ui::is_modern();
        let ticks = pg::ticks() as f64 / 1000.0;
        let cx = w / 2;
        if fx.celestial {
            let m = w.min(h);
            let bx = (w as f64 * 0.18) as i32;
            let by = (h as f64 * 0.28 + 4.0 * (ticks * 0.18).sin()) as i32;
            ui::draw_black_hole(&mut s, (bx, by), 48.max((m as f64 * 0.24) as i32));
            let sx = (w as f64 * 0.84) as i32;
            let sy = (h as f64 * 0.20 + 5.0 * (ticks * 0.14 + 2.0).sin()) as i32;
            ui::draw_saturn(&mut s, (sx, sy), 16.max((m as f64 * 0.075) as i32));
        }
        let amp = fx.menu_bob;
        let bob = if amp != 0 { (amp as f64 * (ticks * 1.3).sin()) as i32 } else { 0 };
        let size = 176.min(96.max(h / 4));
        let v1 = fx.style == "v1";
        let logo = if v1 { None } else { self.menu_logo(size) };
        let center_y0 = h / 2 - 46;
        let avail0 = (h - 72) - (center_y0 + size / 2 + 80);
        let (rows, fnt, pad_x, row_h, gap, tiles_h) = self.flow_menu_tiles(w, 30.max(avail0));
        let shift = 0.max(tiles_h - avail0);
        let center_y = (size / 2 + 14).max(center_y0 - shift) + bob;
        let (base_y, line_w);
        if let Some(logo) = logo {
            let lrect = logo.get_rect().center_at((cx, center_y));
            ui::draw_halo(&mut s, lrect.center(), (lrect.w.max(lrect.h) as f64 * 2.4) as i32, None);
            let rad = 12.max(size / 8);
            if modern || !fx.logo_glow {
                s.blr(&logo, lrect);
                draw::rect_r(&mut s, p.border_light, lrect.inflate(4, 4), 1, rad + 2);
            } else {
                let mut glow = pg::surface_alpha(lrect.w + 26, lrect.h + 26);
                let gr = glow.get_rect();
                draw::rect_r(&mut glow, pg::alpha(p.accent, (55.0 + 35.0 * ui::pulse(1.4, 0.35, 1.0)) as i32), gr, 0, rad + 8);
                s.blr(&glow, glow.get_rect().center_at(lrect.center()));
                s.blr(&logo, lrect);
                draw::rect_r(&mut s, p.accent, lrect.inflate(4, 4), 2, rad + 2);
            }
            base_y = lrect.bottom() - bob;
            line_w = lrect.w;
        } else {
            let logo_font = ui::font(64.min(40.max(w / 11)), true, v1);
            ui::draw_halo(&mut s, (cx, center_y), (w.min(h) as f64 * 0.7) as i32, None);
            let img = if modern || !fx.title_grad {
                logo_font.render("PyGameZ", true, p.text)
            } else {
                let mut glow = logo_font.render("PyGameZ", true, p.accent);
                glow.alpha_set((40.0 + 30.0 * ui::pulse(1.4, 0.35, 1.0)) as i32);
                for (dx, dy) in [(-2, 0), (2, 0), (0, -2), (0, 2)] {
                    s.blr(&glow, glow.get_rect().center_at((cx + dx, center_y + dy)));
                }
                ui::grad_text(&logo_font, "PyGameZ", None, None)
            };
            s.blr(&img, img.get_rect().center_at((cx, center_y)));
            base_y = center_y - bob + img.h() / 2;
            line_w = img.w() + 30;
        }
        if !modern && fx.menu_orbit {
            for (k, col) in [p.accent, p.accent2, p.gold].iter().enumerate() {
                let ang = ticks * (0.6 + 0.17 * k as f64) + k as f64 * 2.09;
                let ox = cx + (ang.cos() * (size / 2 + 34) as f64) as i32;
                let oy = center_y + (ang.sin() * (size / 2 + 12) as f64) as i32;
                if 2 <= ox && ox < w - 2 && 2 <= oy && oy < h - 2 {
                    let g = ui::mix(Color::rgb(0, 0, 0), *col, 0.5);
                    s.fill_rf(g, (ox - 2, oy - 2, 5, 5), BLEND_ADD);
                    draw::circle(&mut s, *col, (ox, oy), 2, 0);
                }
            }
        }
        if modern {
            let lw2 = 72.max(line_w.min(180));
            if fx.grad3.is_some() {
                ui::draw_grad_line(&mut s, cx, base_y + 12, lw2, 3, None, true);
            } else {
                draw::rect_r(&mut s, p.accent, (cx - lw2 / 2, base_y + 12, lw2, 2), 0, 2);
            }
        } else if !v1 {
            draw::rect_r(&mut s, p.accent, (cx - line_w / 2, base_y + 12, line_w, 3), 0, 2);
        }
        let sub = ui::f(18).render(&t("app.menu_title"), true, p.text_dim);
        s.blr(&sub, sub.get_rect().center_at((cx, base_y + 36)));
        let mut hint = ui::f(15).render(&t("app.menu_sub"), true, p.accent);
        if !modern {
            hint.alpha_set((255.0 * ui::pulse(2.2, 0.45, 1.0)) as i32);
        }
        s.blr(&hint, hint.get_rect().center_at((cx, base_y + 62)));
        self.draw_menu_tiles(&mut s, base_y + 80, h - 72, &rows, &fnt, pad_x, row_h, gap, tiles_h);
        self.draw_score_ticker(&mut s, w, h);
        ui::draw_footer(&mut s, w, h, &format!("{} Games   \u{b7}   {}x{} @ {} FPS", self.games.len(), self.game_w, self.game_h, self.fps));
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_menu_tiles(
        &mut self,
        s: &mut Surface,
        top: i32,
        bottom: i32,
        rows: &[Vec<(usize, i32)>],
        fnt: &pg::Font,
        pad_x: i32,
        row_h: i32,
        gap: i32,
        total_h: i32,
    ) {
        let p = ui::pal();
        let w = self.game_w;
        let avail = 30.max(bottom - top);
        let mut y = top + 36.min(0.max((avail - total_h) / 2));
        let hover = self.menu_hover;
        let v42 = ui::fx().style == "v42";
        self.menu_tiles.clear();
        for row in rows {
            let row_w: i32 = row.iter().map(|(_, tw)| *tw).sum::<i32>() + gap * (row.len() as i32 - 1);
            let mut x = w / 2 - row_w / 2;
            for (gi, tw) in row {
                let rect = Rect::new(x, y, *tw, row_h);
                let idx = self.menu_tiles.len();
                let info = self.games[*gi];
                let accent = ui::game_color(info.class_name);
                if Some(idx) == hover {
                    if !ui::is_modern() && ui::fx().style != "v1" {
                        let mut glow = pg::surface_alpha(tw + 14, row_h + 14);
                        let gr = glow.get_rect();
                        draw::rect_r(&mut glow, pg::alpha(accent, 55), gr, 0, row_h / 2 + 7);
                        s.bl(&glow, (x - 7, y - 7));
                    }
                    draw::rect_r(s, p.panel_light, rect, 0, row_h / 2);
                    draw::rect_r(s, accent, rect, 1, row_h / 2);
                } else {
                    draw::rect_r(s, p.panel, rect, 0, row_h / 2);
                    draw::rect_r(s, p.border, rect, 1, row_h / 2);
                }
                if v42 {
                    draw::rect(s, ui::mix(p.panel, Color::rgb(255, 255, 255), 0.10), (x + row_h / 2, y, 2.max(tw - row_h), 1), 0);
                }
                draw::circle(s, accent, (x + pad_x + 4, rect.centery()), 4, 0);
                let img = fnt.render(&info.name(), true, if Some(idx) == hover { p.text } else { p.text_dim });
                s.blr(&img, img.get_rect().midleft_at((x + pad_x + 14, rect.centery())));
                self.menu_tiles.push((rect, *gi));
                x += tw + gap;
            }
            y += row_h + gap;
        }
    }

    fn menu_motion(&mut self, pos: (i32, i32)) {
        self.menu_hover = self.menu_tiles.iter().position(|(r, _)| r.hit(pos));
    }

    fn menu_click(&mut self, pos: (i32, i32)) {
        if let Some((rect, gi)) = self.menu_tiles.iter().find(|(r, _)| r.hit(pos)).cloned() {
            ui::spawn_burst(rect.centerx() as f64, rect.centery() as f64, Some(ui::game_color(self.games[gi].class_name)), 18, 260.0);
            audio::play("click", Some(&shell::settings()));
            self.spiel_starten(gi);
        }
    }

    // ----- Anzeige ---------------------------------------------------------

    /// `_present` - die logische Flaeche mit Letterbox in `frame` setzen.
    ///
    /// `frame` ist das ganze Fenster; die Anzeigeflaeche beginnt bei `x0`.
    pub fn present(&mut self, frame: &mut Surface, x0: i32) {
        let (sw, sh) = (self.disp_w, self.disp_h);
        let scale = (sw as f64 / self.game_w as f64).min(sh as f64 / self.game_h as f64);
        let tw = 1.max((self.game_w as f64 * scale) as i32);
        let th = 1.max((self.game_h as f64 * scale) as i32);
        self.scale = scale;
        self.off = ((sw - tw) / 2, (sh - th) / 2);
        frame.fill_r(Color::rgb(0, 0, 0), (x0, 0, sw, sh));
        if (tw, th) == (self.game_w, self.game_h) {
            frame.bl(&self.canvas, (x0 + self.off.0, self.off.1));
        } else {
            frame.bl(&transform::scale(&self.canvas, tw, th), (x0 + self.off.0, self.off.1));
        }
    }

    /// `beenden()`
    pub fn beenden(&mut self) {
        if self.closing {
            return;
        }
        self.closing = true;
        if let Some(cur) = self.current.as_mut() {
            Self::highscore_speichern(cur.as_mut());
        }
        self.switch_current(None, true);
        stats::end_session();
        stats::flush();
        pgz_core::shutdown();
        audio::shutdown();
    }
}
