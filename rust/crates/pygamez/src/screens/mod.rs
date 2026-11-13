//! screens
//! =======
//! Die Bildschirme, die keine Spiele sind: `menu.py` (Vorspiel, Optionen,
//! Willkommen, Sprache), `profileview.py`, `progress.py`, `lamawiki.py` und
//! `replayview.py`.
//!
//! Die Bauhelfer unten geben `None`, solange ein Bildschirm (noch) fehlt -
//! das Programm bleibt dann einfach, wo es ist.

pub mod charts;
pub mod lamawiki;
pub mod menu;
pub mod profileview;
pub mod progress;
pub mod replayview;

use pgz_engine::Game;

use crate::app::App;

/// `profileview.LevelIntroScreen` - "Dein Startlevel".
pub fn level_intro(app: &App) -> Option<Box<dyn Game>> {
    Some(Box::new(profileview::LevelIntroScreen::new(app.menu_base())))
}

/// `profileview.PlayerPickScreen` - Profil fuer Spieler 2.
pub fn player_pick(app: &App, idx: usize, mode: &str) -> Option<Box<dyn Game>> {
    Some(Box::new(profileview::PlayerPickScreen::new(app.menu_base(), app.games[idx], mode)))
}

/// `progress.ProgressScreen`
pub fn progress(app: &App, tab: Option<String>) -> Option<Box<dyn Game>> {
    let mut s = progress::ProgressScreen::new(app.menu_base(), app.games.clone());
    if let Some(tab) = tab {
        s.set_tab(&tab);
    }
    Some(Box::new(s))
}

/// `profileview.ProfileScreen`
pub fn profiles(app: &App) -> Option<Box<dyn Game>> {
    Some(Box::new(profileview::ProfileScreen::new(app.menu_base())))
}

/// `lamawiki.LamaWikiScreen`
pub fn wiki(app: &App, page: Option<String>, back: Option<String>) -> Option<Box<dyn Game>> {
    Some(Box::new(lamawiki::LamaWikiScreen::new(app.menu_base(), page, back)))
}

/// `replayview.ReplayScreen`
pub fn replays(app: &App) -> Option<Box<dyn Game>> {
    Some(Box::new(replayview::ReplayScreen::new(app.menu_base(), app.games.clone(), None, None, false)))
}
