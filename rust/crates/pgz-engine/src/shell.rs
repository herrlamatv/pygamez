//! shell.rs
//! ========
//! Was in Python ueber `self.app` laeuft: die **eine** Einstellungs-Tabelle,
//! die Programm, Bildschirme und Spiele teilen, und die Befehle an das
//! Programm (`self.app.launch_game(...)`, `self.app.back_to_menu()` ...).
//!
//! In Python haelt jeder Bildschirm einen Verweis auf die App und ruft deren
//! Methoden. In Rust gaebe das einen Kreis aus Verweisen; stattdessen legen
//! Bildschirme einen [`Cmd`] ab, und das Programm arbeitet die Liste nach
//! dem Bild ab - dieselbe Wirkung, eine Bildlaenge spaeter.

use std::cell::RefCell;

use pgz_core::settings::{self, Settings};

/// Ein Auftrag an das Programm.
#[derive(Debug, Clone, PartialEq)]
pub enum Cmd {
    /// `spiel_starten(cls)` - Vorspiel-Bildschirm eines Spiels.
    PreGame(String),
    /// `launch_game(cls, mode, p2)`
    Launch {
        /// Highscore-Schluessel des Spiels.
        key: String,
        /// Modus ("single", "multi" oder ein eigener).
        mode: String,
        /// Profil von Spieler 2.
        p2: Option<String>,
    },
    /// Spieler-2-Auswahl vor einem Mehrspieler-Start.
    PlayerPick {
        /// Spiel.
        key: String,
        /// Modus.
        mode: String,
    },
    /// `back_to_menu()`
    BackToMenu,
    /// `zum_menue()` (Python: `zum_men\u00fc`) - mit Highscore und Statistik.
    ToMenu,
    /// `open_options()`; mit Spiel-Schluessel kehrt "Zurueck" zum Vorspiel zurueck.
    Options(Option<String>),
    /// `open_language()`
    Language,
    /// `open_welcome()`
    Welcome,
    /// `open_progress(tab)`
    Progress(Option<String>),
    /// `open_profiles()`
    Profiles,
    /// `open_lamawiki(page_id)`; mit Spiel-Schluessel zurueck zum Vorspiel.
    Wiki(Option<String>, Option<String>),
    /// `open_replays()`
    Replays,
    /// `PreGameScreen._open_matches()` - Match-Historie eines Spiels
    /// (Schliessen fuehrt zurueck zum Vorspiel-Screen).
    Matches(String),
    /// Zurueck aus der Wiederholung ins geparkte Spiel (`open_replay`, Rueckweg).
    ReturnFromReplay,
    /// `apply_theme(name)`
    ApplyTheme(String),
    /// `apply_resolution(w, h, persist)`
    ApplyResolution(i32, i32, bool),
    /// `set_auto_resolution(on)`
    SetAutoResolution(bool),
    /// `apply_fps(fps)`
    ApplyFps(i32),
    /// `refresh_language()`
    RefreshLanguage,
    /// `refresh_profile()`
    RefreshProfile,
    /// `toggle_fullscreen()`
    ToggleFullscreen,
    /// `beenden()`
    Quit,
}

/// Was Bildschirme ueber das Programm wissen muessen.
#[derive(Debug, Clone)]
pub struct AppInfo {
    /// Logische Breite der Spielflaeche.
    pub game_w: i32,
    /// Logische Hoehe.
    pub game_h: i32,
    /// Ziel-Bildrate.
    pub fps: i32,
}

thread_local! {
    static SETTINGS: RefCell<Settings> = RefCell::new(Settings::default());
    static CMDS: RefCell<Vec<Cmd>> = const { RefCell::new(Vec::new()) };
    static INFO: RefCell<AppInfo> = const { RefCell::new(AppInfo { game_w: 640, game_h: 480, fps: 60 }) };
}

/// Die geteilten Einstellungen (Kopie).
pub fn settings() -> Settings {
    SETTINGS.with(|s| s.borrow().clone())
}

/// Einstellungen ersetzen (beim Start nach `settings::load()`).
pub fn set_settings(s: Settings) {
    SETTINGS.with(|x| *x.borrow_mut() = s);
}

/// Einstellungen aendern.
pub fn edit_settings<R>(f: impl FnOnce(&mut Settings) -> R) -> R {
    SETTINGS.with(|s| f(&mut s.borrow_mut()))
}

/// `settings_mod.save_settings(self.settings)`
pub fn save_settings() {
    let s = settings();
    settings::save(&s);
}

/// Auftrag ablegen.
pub fn post(c: Cmd) {
    CMDS.with(|q| q.borrow_mut().push(c));
}

/// Alle Auftraege abholen.
pub fn take() -> Vec<Cmd> {
    CMDS.with(|q| std::mem::take(&mut *q.borrow_mut()))
}

/// Was das Programm gerade ist.
pub fn info() -> AppInfo {
    INFO.with(|i| i.borrow().clone())
}

/// Vom Programm gesetzt.
pub fn set_info(i: AppInfo) {
    INFO.with(|x| *x.borrow_mut() = i);
}

/// `replay.is_enabled(settings)`
pub fn replay_enabled(s: &Settings) -> bool {
    s.game_bool("replay", "enabled", true)
}
