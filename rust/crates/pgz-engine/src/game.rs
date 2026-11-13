//! game.rs
//! =======
//! Gegenstueck zu `game_base.py`: das Eingabe-Ereignis, die Basisdaten jedes
//! Spiels und das Spiel-Trait.
//!
//! In Python erbt jedes Spiel von `Game` und bekommt damit `score`,
//! `game_over`, `settings`, `font`, `accent`, `report_result()` und so fort.
//! In Rust traegt jedes Spiel eine [`GameBase`] (`self.b`) und setzt das
//! Trait [`Game`] um. Die Hilfsfunktionen der Basisklasse stehen an
//! [`GameBase`] - `self.b.play_sound("eat")` statt `self.play_sound("eat")`.

use pgz_core::settings::Settings;

use crate::pg::{Color, Font, RectExt, SurfExt, Surface};
use crate::{audio, ui};

// ---------------------------------------------------------------------------
//  InputEvent
// ---------------------------------------------------------------------------

/// Art eines Eingabe-Ereignisses (`InputEvent.KEYDOWN` ...).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Taste gedrueckt.
    KeyDown,
    /// Taste losgelassen.
    KeyUp,
    /// Maustaste gedrueckt.
    MouseDown,
    /// Maustaste losgelassen.
    MouseUp,
    /// Maus bewegt.
    MouseMove,
    /// Relative Mausbewegung (Pointer-Capture).
    MouseRel,
    /// Mausrad (delta in Rasten, + = hoch).
    Wheel,
}

/// `game_base.InputEvent` - ein vereinheitlichtes Eingabe-Ereignis.
///
/// `key` traegt den **Tkinter-keysym** wie in Python (`"Up"`, `"space"`,
/// `"Return"`, `"w"`, mit Umschalt `"W"`), `char` das wirklich getippte
/// Zeichen (leer bei Steuertasten).
#[derive(Debug, Clone, PartialEq)]
pub struct InputEvent {
    /// Die Art.
    pub kind: Kind,
    /// Tk-keysym (nur bei Tasten).
    pub key: String,
    /// Position relativ zur Spielflaeche (Maus).
    pub pos: (i32, i32),
    /// Maustaste: 1 = links, 3 = rechts.
    pub button: i32,
    /// Mausrad-Rasten.
    pub delta: i32,
    /// Relative Bewegung (Pointer-Capture).
    pub rel: (i32, i32),
    /// Das getippte Zeichen.
    pub char: String,
    /// Tastenwiederholung des Betriebssystems.
    pub repeat: bool,
    /// Physischer Tastencode.
    pub code: i32,
}

impl InputEvent {
    /// Ein leeres Ereignis der Art `kind`.
    pub fn new(kind: Kind) -> InputEvent {
        InputEvent {
            kind,
            key: String::new(),
            pos: (0, 0),
            button: 1,
            delta: 0,
            rel: (0, 0),
            char: String::new(),
            repeat: false,
            code: 0,
        }
    }

    /// Ein Tastendruck.
    pub fn key_down(key: &str, ch: &str, repeat: bool) -> InputEvent {
        InputEvent { key: key.to_string(), char: ch.to_string(), repeat, ..InputEvent::new(Kind::KeyDown) }
    }

    /// Eine losgelassene Taste.
    pub fn key_up(key: &str) -> InputEvent {
        InputEvent { key: key.to_string(), ..InputEvent::new(Kind::KeyUp) }
    }

    /// Ein Mausklick.
    pub fn mouse(kind: Kind, pos: (i32, i32), button: i32) -> InputEvent {
        InputEvent { pos, button, ..InputEvent::new(kind) }
    }

    /// `event.kind == KEYDOWN and event.key == k`
    pub fn is_key(&self, k: &str) -> bool {
        self.kind == Kind::KeyDown && self.key == k
    }

    /// `event.kind == KEYDOWN and event.key in ks`
    pub fn is_any(&self, ks: &[&str]) -> bool {
        self.kind == Kind::KeyDown && ks.contains(&self.key.as_str())
    }

    /// `event.key in ks` (ohne Art-Pruefung).
    pub fn key_in(&self, ks: &[&str]) -> bool {
        ks.contains(&self.key.as_str())
    }
}

// ---------------------------------------------------------------------------
//  Die Basis jedes Spiels
// ---------------------------------------------------------------------------

/// Was ein Spiel von `game_base.Game` erbt.
#[derive(Debug, Clone)]
pub struct GameBase {
    /// `self.surface` - die logische Spielflaeche (teilt den Speicher mit
    /// der Leinwand des Programms).
    pub surface: Surface,
    /// `self.width`
    pub width: i32,
    /// `self.height`
    pub height: i32,
    /// `self.mode` ("single", "multi" oder ein eigener Modus aus `MODES`).
    pub mode: String,
    /// `self.multiplayer`
    pub multiplayer: bool,
    /// `self.player_ids` - `None` = aktives Profil bzw. Gast.
    pub player_ids: [Option<String>; 2],
    /// `self.settings`
    pub settings: Settings,
    /// `self.score`
    pub score: i64,
    /// `self.game_over`
    pub game_over: bool,
    /// `self.paused`
    pub paused: bool,
    /// `self.font` (22)
    pub font: Font,
    /// `self.big_font` (48, fett)
    pub big_font: Font,
    /// `self.accent` - Akzentfarbe des Spiels.
    pub accent: Color,
    /// `highscore_key`
    pub key: &'static str,
    /// `is_menu` - Menue-/Optionsbildschirme zaehlen nicht.
    pub is_menu: bool,
    /// Riegel von `report_result` (eine Meldung je Partie).
    pub result_reported: bool,
    /// Riegel von `report_duel`.
    pub duel_reported: bool,
    /// `resume_from_game_over` wurde gerufen.
    pub resume_same_game: bool,
    /// `_hs_saved` - Highscore dieser Partie schon gesichert.
    pub hs_saved: bool,
    /// `_hs_value` - Highscore fuer das Banner.
    pub hs_value: i64,
    /// `_hs_record` - neuer Rekord?
    pub hs_record: bool,
    /// `_was_over` - war schon einmal Game Over (Neustart erkennen).
    pub was_over: bool,
}

impl GameBase {
    /// Basisdaten fuer ein Spiel mit Schluessel `key` und Klassenname `class_name`.
    pub fn new(
        surface: Surface,
        width: i32,
        height: i32,
        mode: &str,
        settings: Settings,
        key: &'static str,
        class_name: &str,
    ) -> GameBase {
        GameBase {
            surface,
            width,
            height,
            mode: mode.to_string(),
            multiplayer: mode == "multi",
            player_ids: [None, None],
            settings,
            score: 0,
            game_over: false,
            paused: false,
            font: ui::font(22, false, false),
            big_font: ui::font(48, true, false),
            accent: ui::game_color(class_name),
            key,
            is_menu: false,
            result_reported: false,
            duel_reported: false,
            resume_same_game: false,
            hs_saved: false,
            hs_value: 0,
            hs_record: false,
            was_over: false,
        }
    }

    /// Ein Handle auf die Spielflaeche zum Zeichnen.
    ///
    /// Der Klon teilt den Speicher - gezeichnet wird also auf die echte
    /// Flaeche, und `self` bleibt frei fuer andere Zugriffe.
    pub fn sf(&self) -> Surface {
        self.surface.clone()
    }

    /// `draw_center_text(text, font, color, y_offset)`
    pub fn draw_center_text(&self, text: &str, font: &Font, color: Color, y_offset: i32) {
        let img = font.render(text, true, color);
        let r = img.get_rect().center_at((self.width / 2, self.height / 2 + y_offset));
        self.sf().blr(&img, r);
    }

    /// `key_for(player, action)` - belegte Taste.
    pub fn key_for(&self, player: &str, action: &str) -> String {
        self.settings.bindings(player).get(action).to_string()
    }

    /// `is_action(key, action, player)` - ohne Spieler gegen beide pruefen.
    pub fn is_action(&self, key: &str, action: &str, player: Option<&str>) -> bool {
        let players: &[&str] = match player {
            Some(p) => &[if p == "p2" { "p2" } else { "p1" }],
            None => &["p1", "p2"],
        };
        players.iter().any(|p| !key.is_empty() && self.settings.bindings(p).get(action) == key)
    }

    /// `key_is_free(key)` - keiner Aktion von Spieler 1 oder 2 zugeordnet.
    pub fn key_is_free(&self, key: &str) -> bool {
        for p in ["p1", "p2"] {
            if self.settings.bindings(p).keys().contains(&key) {
                return false;
            }
        }
        true
    }

    /// `play_sound(name)`
    pub fn play_sound(&self, name: &str) {
        audio::play(name, Some(&self.settings));
    }

    /// `rumble(ms)`
    pub fn rumble(&self, ms: i32) {
        audio::rumble(Some(&self.settings), ms);
    }

    /// `report_result(won)` - Sieg/Niederlage an die Statistik, einmal je Partie.
    pub fn report_result(&mut self, won: bool) {
        if self.is_menu || self.result_reported {
            return;
        }
        self.result_reported = true;
        pgz_core::stats::record_result(self.key, won);
        pgz_core::achievements::check_stats(false);
    }

    /// `resume_from_game_over()` - Game Over aufheben, ohne neue Partie.
    pub fn resume_from_game_over(&mut self) {
        self.game_over = false;
        self.resume_same_game = true;
    }

    /// `new_round_result()` - Meldungen fuer die naechste Runde freigeben.
    pub fn new_round_result(&mut self) {
        self.result_reported = false;
        self.duel_reported = false;
    }

    /// `pname(index)` - Profilname eines Sitzplatzes.
    pub fn pname(&self, index: usize) -> String {
        let pid = self.player_ids.get(index).cloned().flatten();
        let pid = match (pid, index) {
            (None, 0) => Some(pgz_core::profiles::active_id()),
            (p, _) => p,
        };
        if let Some(p) = pid {
            if p != pgz_core::profiles::GUEST {
                return pgz_core::profiles::name_of(Some(&p));
            }
        }
        pgz_core::i18n::t(&format!("common.player{}", index + 1))
    }

    /// `pcolor(index)` - Profilfarbe eines Sitzplatzes.
    pub fn pcolor(&self, index: usize) -> Color {
        let pid = self.player_ids.get(index).cloned().flatten();
        let pid = match (pid, index) {
            (None, 0) => Some(pgz_core::profiles::active_id()),
            (p, _) => p,
        };
        if let Some(p) = pid {
            if p != pgz_core::profiles::GUEST {
                return ui::c3(pgz_core::profiles::color_rgb(Some(&p)));
            }
        }
        if index == 0 {
            self.accent
        } else {
            ui::accent2()
        }
    }

    /// `report_duel(winner)` - Sieger eines lokalen Duells (0 oder 1).
    pub fn report_duel(&mut self, winner: usize) {
        if self.is_menu || !self.multiplayer || self.duel_reported {
            return;
        }
        self.duel_reported = true;
        let p1 = self.player_ids[0].clone().unwrap_or_else(pgz_core::profiles::active_id);
        let p2 = self.player_ids[1].clone().unwrap_or_else(|| pgz_core::profiles::GUEST.to_string());
        pgz_core::stats::record_duel(self.key, &p2, winner == 0, Some(&p1));
        if p2 != pgz_core::profiles::GUEST {
            pgz_core::stats::record_duel(self.key, &p1, winner == 1, Some(&p2));
        }
        pgz_core::achievements::check_stats(false);
    }

    /// `ach_event(event_id, value)` - Erfolgs-Ereignis.
    pub fn ach_event(&self, event_id: &str, value: Option<i64>) {
        if self.is_menu {
            return;
        }
        pgz_core::achievements::event(event_id, value);
    }

    /// Spiel-Option lesen (`self.settings.get(section, {}).get(key, default)`).
    pub fn opt_bool(&self, section: &str, key: &str, default: bool) -> bool {
        self.settings.game_bool(section, key, default)
    }

    /// Ganzzahl-Option.
    pub fn opt_i64(&self, section: &str, key: &str, default: i64) -> i64 {
        self.settings.game_i64(section, key, default)
    }

    /// Text-Option.
    pub fn opt_str(&self, section: &str, key: &str, default: &str) -> String {
        self.settings.game_str(section, key, default)
    }

    /// Beliebiger Optionswert (`self.settings.get(section, {}).get(key)`).
    pub fn opt_value(&self, section: &str, key: &str) -> Option<serde_json::Value> {
        self.settings.games.get(section).and_then(|v| v.get(key)).cloned()
    }

    /// `float(opt.get(key, default))` wie in Python - `None`, wenn Python
    /// hier einen TypeError/ValueError werfen wuerde (Spiel nimmt dann den
    /// eigenen Ersatzwert).
    pub fn opt_float_py(&self, section: &str, key: &str, default: f64) -> Option<f64> {
        match self.opt_value(section, key) {
            None => Some(default),
            Some(serde_json::Value::Number(n)) => n.as_f64(),
            Some(serde_json::Value::Bool(b)) => Some(if b { 1.0 } else { 0.0 }),
            Some(serde_json::Value::String(st)) => st.trim().parse::<f64>().ok(),
            Some(_) => None,
        }
    }

    /// `int(opt.get(key, default))` wie in Python (`None` bei Fehler).
    pub fn opt_int_py(&self, section: &str, key: &str, default: i64) -> Option<i64> {
        match self.opt_value(section, key) {
            None => Some(default),
            Some(serde_json::Value::Number(n)) => n.as_i64().or_else(|| n.as_f64().map(|f| f.trunc() as i64)),
            Some(serde_json::Value::Bool(b)) => Some(b as i64),
            Some(serde_json::Value::String(st)) => st.trim().parse::<i64>().ok(),
            Some(_) => None,
        }
    }

    /// Option setzen **und** speichern (`settings_mod.save_settings`).
    ///
    /// In Python ist `self.settings` dasselbe Dict wie das des Programms;
    /// hier wird die Kopie des Spiels **und** die geteilte Tabelle
    /// geaendert, und gespeichert wird die geteilte.
    pub fn set_opt(&mut self, section: &str, key: &str, value: serde_json::Value) {
        self.settings.set_game(section, key, value.clone());
        crate::shell::edit_settings(|s| s.set_game(section, key, value));
        crate::shell::save_settings();
    }
}

// ---------------------------------------------------------------------------
//  Das Trait
// ---------------------------------------------------------------------------

/// `game_base.Game` - was jedes Spiel und jeder Bildschirm umsetzt.
pub trait Game {
    /// Die geerbten Felder.
    fn base(&self) -> &GameBase;
    /// Die geerbten Felder, veraenderbar.
    fn base_mut(&mut self) -> &mut GameBase;
    /// `reset()` - Startzustand.
    fn reset(&mut self);
    /// `update(dt)` - Logik, `dt` in Sekunden.
    fn update(&mut self, dt: f64);
    /// `draw()` - auf `self.base().surface` zeichnen.
    fn draw(&mut self);
    /// `handle_event(event)`
    fn handle_event(&mut self, ev: &InputEvent);
    /// `on_surface_changed()` - nach einem Aufloesungswechsel.
    fn on_surface_changed(&mut self) {}
    /// `on_exit()` - das Spiel wird verlassen.
    fn on_exit(&mut self) {}
    /// `wants_escape` - Escape selbst behandeln.
    fn wants_escape(&self) -> bool {
        false
    }
    /// `wants_right_click`
    fn wants_right_click(&self) -> bool {
        false
    }
    /// `capture_mouse` - Zeiger einfangen, relative Bewegung.
    fn capture_mouse(&self) -> bool {
        false
    }
    /// `show_highscore_banner`
    fn show_highscore_banner(&self) -> bool {
        true
    }
    /// `name` eines Menue-Bildschirms (Statuszeile); Spiele lassen es leer.
    fn screen_name(&self) -> String {
        String::new()
    }
    /// `replay_request` - Wiederholung anfordern (Taste P am Rundenende).
    fn take_replay_request(&mut self) -> Option<serde_json::Value> {
        None
    }
    /// `replay_begin(rep)` - Wiedergabe vorbereiten; `false` = das Spiel kann
    /// keine Wiederholungen zeigen.
    fn replay_begin(&mut self, _rep: &serde_json::Value) -> bool {
        false
    }
    /// `replay_seek(index, frame)` - Stand der Aufnahme herstellen.
    fn replay_seek(&mut self, _index: usize, _frame: usize) {}
    /// `replay_draw(aiming, banner)` - den Stand zeichnen.
    fn replay_draw(&mut self, _aiming: bool, _banner: bool) {}
    /// `getattr(inst, "hud_h", 40) + getattr(inst, "card_h", 0)` - Hoehe von
    /// HUD und Scorekarte (der Replay-Chip sitzt darunter).
    fn replay_hud_h(&self) -> i32 {
        40
    }
    /// Kennung fuer Musik und Wiedererkennung (`is` in Python).
    fn ident(&self) -> usize {
        self.base() as *const GameBase as usize
    }
}

/// Die festen Angaben einer Spielklasse (Klassenattribute in Python).
#[derive(Clone, Copy)]
pub struct GameInfo {
    /// `highscore_key`
    pub key: &'static str,
    /// Klassenname der Python-Fassung (`"SnakeGame"`) - fuer Farbe und Namen.
    pub class_name: &'static str,
    /// `supports_multiplayer`
    pub supports_multiplayer: bool,
    /// `MODES` - eigene Modus-Knoepfe `(mode_key, i18n_key)`.
    pub modes: &'static [(&'static str, &'static str)],
    /// Baut das Spiel (Gegenstueck zu `__init__` + `reset()`).
    pub make: fn(GameBase) -> Box<dyn Game>,
}

impl GameInfo {
    /// Anzeigename in der aktiven Sprache.
    pub fn name(&self) -> String {
        pgz_core::achievements::game_name(self.key)
    }
}

impl std::fmt::Debug for GameInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GameInfo({})", self.key)
    }
}
