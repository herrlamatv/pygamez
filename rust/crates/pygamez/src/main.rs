//! PyGameZ - die Spielesammlung in Rust, auf rustgame.
//!
//! Einstieg: Datenschicht starten, Fenster oeffnen, Schleife fahren.
//!
//! ## Pruefstand
//!
//! Mit `PYGAMEZ_SHOT=<datei|ordner>` laeuft das Programm **ohne Fenster**
//! mit fester Uhr (60 Bilder je Sekunde, `pg::set_fixed_ticks`) und legt
//! danach ein Bild ab - wiederholbar Bild fuer Bild. Schalter:
//!
//! | Variable | Wirkung |
//! |---|---|
//! | `PYGAMEZ_SHOT` | Zieldatei; mit `PYGAMEZ_GAME=all` ein Ordner |
//! | `PYGAMEZ_SCREEN` | `menu`, `options`, `language`, `welcome`, `pregame:<spiel>`, `matches:<spiel>[:all]` |
//! | `PYGAMEZ_GAME` | Spiel-Schluessel oder `all` |
//! | `PYGAMEZ_MODE` | Modus des Spiels (Standard `single`) |
//! | `PYGAMEZ_THEME` / `PYGAMEZ_LANG` | Design / Sprache |
//! | `PYGAMEZ_FRAMES` | Bilder bis zum Foto (Standard 40) |
//! | `PYGAMEZ_KEYS` | Ereignisse, eines je Bild ab Bild 2: `Return,Down,Down` |
//! | `PYGAMEZ_KEYS_FILE` | dieselbe Liste aus einer Datei (fuer lange Folgen) |
//!
//! Ein Eintrag in `PYGAMEZ_KEYS` ist eine Taste (gedrueckt und gleich wieder
//! losgelassen) oder: `+Taste` / `-Taste` (nur druecken / nur loslassen),
//! `@x:y` (Linksklick in Spielkoordinaten), `@x:y:3` (Rechtsklick),
//! `~x:y` (Maus bewegen), `^dx:dy` (relative Mausbewegung, MOUSEREL),
//! `<x:y` / `>x:y` (Maustaste nur druecken / nur loslassen),
//! `%x:y:d` (Mausrad bei x, y um d Rasten, + = hoch), `*Taste`
//! (Tastenwiederholung: KEYDOWN mit `repeat`, ohne Loslassen), `!blur`
//! (Fokusverlust des Fensters: Tasten loslassen + `pause_on_blur`).
//! Leer = ein Bild ohne Ereignis.
//! | `PYGAMEZ_SEED` | Startwert fuer `random` |
//! | `PYGAMEZ_FULL` | `1` = ganzes Fenster samt Seitenleiste fotografieren |
//! | `PYGAMEZ_DATA_DIR` | eigener Datenordner |
//!
//! Mit Fenster: `PYGAMEZ_VOLLBILD_PROBE=<datei>` schaltet das Vollbild ueber
//! die echten Wege um (Bild 30: Taste F11, Bild 70: Knopf der Seitenleiste),
//! zieht das Fenster dann unter die Mindestgroesse (Bild 101), misst Fenster
//! und Puffer in jeder Stufe, legt je ein Bild `<datei>.<stufe>.png` ab und
//! beendet sich (`tools/vollbild_check.py`).
//!
//! `PYGAMEZ_TASTEN_PROBE=<datei>` oeffnet kein Spielfenster, sondern nur ein
//! verstecktes Fenster mit der Tastatur-Fensterprozedur (`fenster::
//! tasten_probe`); `tools/tasten_check.py` schickt ihm Tastennachrichten und
//! vergleicht das Ergebnis mit Tk.

mod app;
mod fenster;
mod icons;
mod input;
mod screens;
mod sidebar;
mod vollbild;

use std::time::{Duration, Instant};

use minifb::{MouseButton, MouseMode, Window, WindowOptions};
use pgz_engine::pg;
use pgz_engine::shell::{self, Cmd};
use pgz_engine::{audio, py};

use app::App;

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.is_empty())
}

fn start() -> App {
    pgz_core::init();
    pgz_engine::achv::register_switch_hook();
    let st = pgz_core::settings::load();
    shell::set_settings(st);
    if let Some(theme) = env("PYGAMEZ_THEME") {
        shell::edit_settings(|s| s.theme = theme);
    }
    if let Some(lang) = env("PYGAMEZ_LANG") {
        pgz_core::i18n::set_language(&lang, false);
    }
    pg::init_time();
    pg::init_font();
    if env("PYGAMEZ_SHOT").is_none() {
        audio::init();
    }
    if let Some(seed) = env("PYGAMEZ_SEED").and_then(|s| s.parse::<u64>().ok()) {
        py::seed(seed);
        py::set_deterministic(true);
    }
    App::new(pgz_games::all_games())
}

fn main() {
    if env("PYGAMEZ_SHOT").is_some() {
        pruefstand();
        return;
    }
    if let Some(ziel) = env("PYGAMEZ_TASTEN_PROBE") {
        fenster::tasten_probe(&ziel, &mut input::Tastatur::default());
        return;
    }
    // Wie main.py: vor dem ersten Fenster (und vor dem ersten Schrift-Messen
    // der Seitenleiste) DPI-bewusst werden.
    fenster::dpi_bewusst();
    let mut app = start();
    fenster(&mut app);
}

// ---------------------------------------------------------------------------
//  Das Fenster
// ---------------------------------------------------------------------------

/// Startgroesse wie `_fit_sidebar`: so hoch, dass die ganze Spieleliste
/// passt (hoechstens Bildschirm - 120), so breit wie Leiste + Spielflaeche im
/// Seitenverhaeltnis der Aufloesung (hoechstens Bildschirm - 80).
fn startgroesse(app: &mut App) -> (usize, usize) {
    let (sw, sh) = fenster::bildschirm();
    let need_w = app.sidebar.breite;
    let need_h = app.sidebar.bedarf_hoehe(app.games.len());
    let h = 560.max(need_h.min(sh - 120));
    let spiel_w = (h as f64 * app.game_w as f64 / app.game_h.max(1) as f64) as i32;
    let w = (sw - 80).min(need_w + 640.max(spiel_w));
    (w.max(1) as usize, h.max(1) as usize)
}

fn fenster(app: &mut App) {
    let (mut win_w, mut win_h) = startgroesse(app);
    let opts = WindowOptions { resize: true, scale_mode: minifb::ScaleMode::UpperLeft, ..WindowOptions::default() };
    let mut window = match Window::new(&pgz_core::i18n::t("app.title"), win_w, win_h, opts) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Fenster liess sich nicht oeffnen: {e}");
            return;
        }
    };
    // _set_window_icon, root.minsize(880, 560) und die Tastatur wie Tk.
    let symbol = fenster::symbol_setzen(window.get_window_handle());
    fenster::einhaengen(window.get_window_handle(), 880, 560);
    window.set_target_fps(0);
    // Wie Windows: erste Wiederholung nach etwa einer halben Sekunde.
    window.set_key_repeat_delay(0.45);
    window.set_key_repeat_rate(0.045);
    let zeichen = input::Zeichen::default();
    let liste = zeichen.liste.clone();
    window.set_input_callback(Box::new(zeichen));
    let mut tasten = input::Tastatur::default();

    let mut frame = pg::surface(win_w as i32, win_h as i32);
    let mut puffer: Vec<u32> = vec![0; win_w * win_h];
    let mut letzte = Instant::now();
    let mut maus_alt: Option<(i32, i32)> = None;
    let mut knoepfe = [false; 3];
    let mut titel = String::new();
    // Pruefschalter (siehe Kopf der Datei).
    let probe = env("PYGAMEZ_VOLLBILD_PROBE");
    let mut bild = 0u32;
    let mut bericht: Vec<String> = Vec::new();
    // Hatte das Fenster im letzten Bild den Fokus? (Fokusverlust = Wechsel
    // aktiv -> inaktiv, wie Tks <FocusOut> mit `focus_get() is None`.)
    let mut war_aktiv = false;

    while window.is_open() && !app.closing {
        let jetzt = Instant::now();
        let dt = (jetzt - letzte).as_secs_f64().min(0.1);
        letzte = jetzt;

        // Fenstergroesse: Puffer und Anzeigeflaeche nachziehen.
        let (w, h) = window.get_size();
        if (w, h) != (win_w, win_h) && w > 0 && h > 0 {
            win_w = w;
            win_h = h;
            frame = pg::surface(win_w as i32, win_h as i32);
            puffer = vec![0; win_w * win_h];
        }
        let sb = app.sidebar.breite;
        app.sync_display_size(win_w as i32 - sb, win_h as i32);

        // --- Tasten --------------------------------------------------------
        // Windows: Taste + Zeichen aus der eigenen Fensterprozedur (wie Tk);
        // sonst aus minifbs Tastenstand.
        let roh = fenster::tasten_holen();
        let aktiv = window.is_active();
        if aktiv {
            let gemeldet = match roh {
                Some(roh) => tasten.aus_nachrichten(roh),
                None => tasten.abholen(&window, &liste),
            };
            for t in gemeldet {
                if t.down {
                    app.key_down(&t.keysym, &t.ch, t.repeat);
                } else {
                    app.key_up(&t.keysym);
                }
            }
        } else {
            for k in tasten.alle_los() {
                app.key_up(&k);
            }
            if war_aktiv {
                // main._on_focus_out: Tasten loslassen, dann _pause_on_blur
                app.fokus_verloren();
            }
        }
        war_aktiv = aktiv;

        // --- Maus ----------------------------------------------------------
        if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Pass) {
            let pos = (mx as i32, my as i32);
            if maus_alt != Some(pos) {
                if pos.0 < sb {
                    app.sidebar.bewegung(pos);
                } else {
                    app.sidebar.verlassen();
                    if app.wants_capture() {
                        if let Some(alt) = maus_alt {
                            app.mouse_rel(pos.0 - alt.0, pos.1 - alt.1);
                        }
                    } else {
                        app.mouse_move(pos.0 - sb, pos.1);
                    }
                }
                maus_alt = Some(pos);
            }
            for (i, btn) in [MouseButton::Left, MouseButton::Middle, MouseButton::Right].iter().enumerate() {
                let unten = window.get_mouse_down(*btn);
                if unten != knoepfe[i] {
                    knoepfe[i] = unten;
                    // Tk-Nummern: 1 = links, 2 = Mitte, 3 = rechts.
                    let nummer = i as i32 + 1;
                    if pos.0 < sb {
                        if nummer == 1 {
                            let a = if unten {
                                app.sidebar.druecken(pos, app.games.len())
                            } else {
                                app.sidebar.loslassen(pos)
                            };
                            if let Some(a) = a {
                                app.seitenleiste(a);
                            }
                        }
                    } else if unten {
                        app.mouse_down(pos.0 - sb, pos.1, nummer);
                    } else {
                        app.mouse_up(pos.0 - sb, pos.1, nummer);
                    }
                }
            }
            if let Some((_, dy)) = window.get_scroll_wheel() {
                if dy != 0.0 {
                    let delta = if dy.abs() >= 1.0 { dy as i32 } else if dy > 0.0 { 1 } else { -1 };
                    if pos.0 < sb {
                        app.sidebar.rad(pos, delta);
                    } else {
                        app.wheel(pos.0 - sb, pos.1, delta);
                    }
                }
            }
        } else {
            app.sidebar.verlassen();
        }
        app.sidebar.animieren();

        // --- Ein Bild ------------------------------------------------------
        if probe.is_some() {
            bild += 1;
            match bild {
                30 => {
                    app.key_down("F11", "", false);
                    app.key_up("F11");
                }
                70 => app.seitenleiste(sidebar::Aktion::Knopf("fullscreen".into())),
                // Kleiner als root.minsize(880, 560) ziehen wollen.
                101 => fenster::innen_setzen(window.get_window_handle(), 300, 200),
                _ => {}
            }
        }
        app.frame(dt);
        if app.take_vollbild() {
            vollbild::umschalten(window.get_window_handle());
        }
        let neu_titel = pgz_core::i18n::t("app.title");
        if neu_titel != titel {
            window.set_title(&neu_titel);
            titel = neu_titel;
        }
        let games = app.games.clone();
        app.sidebar.draw(&mut frame, win_h as i32, &games);
        let sb = app.sidebar.breite;
        app.present(&mut frame, sb);
        frame.mit_pixeln(|px, pitch, (fw, fh)| {
            for y in 0..fh.min(win_h) {
                let n = fw.min(win_w);
                puffer[y * win_w..y * win_w + n].copy_from_slice(&px[y * pitch..y * pitch + n]);
            }
        });
        if window.update_with_buffer(&puffer, win_w, win_h).is_err() {
            break;
        }
        if let Some(ziel) = &probe {
            if bild == 1 {
                let (sw, sh) = fenster::bildschirm();
                let bedarf = app.sidebar.bedarf_hoehe(app.games.len());
                bericht.push(format!(
                    "start: symbol={symbol} dpi={} bildschirm={sw}x{sh} leiste={} bedarf={bedarf} aufloesung={}x{}",
                    fenster::dpi(),
                    app.sidebar.breite,
                    app.game_w,
                    app.game_h
                ));
            }
            let stufe = match bild {
                29 => Some("vorher"),
                60 => Some("vollbild"),
                100 => Some("zurueck"),
                115 => Some("klein"),
                _ => None,
            };
            if let Some(stufe) = stufe {
                bericht.push(format!("{stufe}: {} puffer={win_w}x{win_h}", vollbild::zustand(window.get_window_handle())));
                pg::image::save(&frame, format!("{ziel}.{stufe}.png"));
            }
            if bild >= 116 {
                std::fs::write(ziel, bericht.join("\n") + "\n").ok();
                break;
            }
        }

        // --- Takt ----------------------------------------------------------
        let soll = Duration::from_secs_f64(1.0 / app.fps.max(5) as f64);
        let gebraucht = jetzt.elapsed();
        if gebraucht < soll {
            std::thread::sleep(soll - gebraucht);
        }
    }
    app.beenden();
}

// ---------------------------------------------------------------------------
//  Der Pruefstand
// ---------------------------------------------------------------------------

/// Ein Eintrag aus `PYGAMEZ_KEYS` zustellen (siehe Kopf der Datei).
fn ereignis(app: &mut App, k: &str) {
    use pgz_engine::{InputEvent, Kind};
    let zahlen = |rest: &str| -> Vec<i32> { rest.split(':').filter_map(|z| z.parse().ok()).collect() };
    let zeichen = |k: &str| if k.chars().count() == 1 { k.to_string() } else { String::new() };
    if k.is_empty() {
    } else if k == "!blur" {
        // Fokusverlust des Fensters (wie oben in der Fensterschleife)
        app.fokus_verloren();
    } else if let (Some(rest), true) = (k.strip_prefix('@'), k.len() > 1) {
        let z = zahlen(rest);
        if z.len() >= 2 {
            let knopf = z.get(2).copied().unwrap_or(1);
            app.spiel_ereignis(&InputEvent::mouse(Kind::MouseDown, (z[0], z[1]), knopf));
            app.spiel_ereignis(&InputEvent::mouse(Kind::MouseUp, (z[0], z[1]), knopf));
        }
    } else if let (Some(z), true) = (k.get(..1), k.len() > 1 && (k.starts_with('<') || k.starts_with('>'))) {
        let w = zahlen(&k[1..]);
        if w.len() >= 2 {
            let art = if z == "<" { Kind::MouseDown } else { Kind::MouseUp };
            app.spiel_ereignis(&InputEvent::mouse(art, (w[0], w[1]), w.get(2).copied().unwrap_or(1)));
        }
    } else if let (Some(rest), true) = (k.strip_prefix('^'), k.len() > 1) {
        let z = zahlen(rest);
        if z.len() >= 2 {
            let mut ev = InputEvent::new(Kind::MouseRel);
            ev.rel = (z[0], z[1]);
            app.spiel_ereignis(&ev);
        }
    } else if let (Some(rest), true) = (k.strip_prefix('~'), k.len() > 1) {
        let z = zahlen(rest);
        if z.len() >= 2 {
            app.spiel_ereignis(&InputEvent::mouse(Kind::MouseMove, (z[0], z[1]), 1));
        }
    } else if let (Some(rest), true) = (k.strip_prefix('%'), k.len() > 1) {
        // Mausrad bei (x, y) um d Rasten (vergleich.py: WHEEL)
        let z = zahlen(rest);
        if z.len() >= 2 {
            let mut ev = InputEvent::mouse(Kind::Wheel, (z[0], z[1]), 1);
            ev.delta = z.get(2).copied().unwrap_or(1);
            app.spiel_ereignis(&ev);
        }
    } else if let (Some(rest), true) = (k.strip_prefix('*'), k.len() > 1) {
        // Tastenwiederholung des Systems (KEYDOWN mit repeat, ohne Loslassen)
        app.key_down(rest, &zeichen(rest), true);
    } else if let (Some(rest), true) = (k.strip_prefix('+'), k.len() > 1) {
        app.key_down(rest, &zeichen(rest), false);
    } else if let (Some(rest), true) = (k.strip_prefix('-'), k.len() > 1) {
        app.key_up(rest);
    } else {
        app.key_down(k, &zeichen(k), false);
        app.key_up(k);
    }
}

fn pruefstand() {
    let mut app = start();
    let ziel = env("PYGAMEZ_SHOT").unwrap_or_default();
    let frames: u64 = env("PYGAMEZ_FRAMES").and_then(|f| f.parse().ok()).unwrap_or(40);
    // Lange Folgen kommen aus einer Datei (Windows begrenzt Umgebung und
    // Befehlszeile auf rund 32000 Zeichen).
    let liste = env("PYGAMEZ_KEYS_FILE")
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|s| s.trim().to_string())
        .or_else(|| env("PYGAMEZ_KEYS"));
    let tasten: Vec<String> = liste.map(|k| k.split(',').map(String::from).collect()).unwrap_or_default();
    let mode = env("PYGAMEZ_MODE").unwrap_or_else(|| "single".into());
    let full = env("PYGAMEZ_FULL").is_some();
    let spiel = env("PYGAMEZ_GAME");
    let mut uhr = 1000u64;

    let lauf = |app: &mut App, uhr: &mut u64, pfad: &str| {
        for i in 0..frames {
            pg::set_fixed_ticks(Some(*uhr));
            if i >= 2 {
                if let Some(k) = tasten.get((i - 2) as usize) {
                    ereignis(app, k);
                }
            }
            app.frame(1.0 / 60.0);
            *uhr += 1000 / 60 + u64::from(i % 3 == 0);
        }
        if full {
            let (ww, wh) = (1180, 760);
            let mut frame = pg::surface(ww, wh);
            app.sync_display_size(ww - app.sidebar.breite, wh);
            let games = app.games.clone();
            app.sidebar.draw(&mut frame, wh, &games);
            let sb = app.sidebar.breite;
            app.present(&mut frame, sb);
            pg::image::save(&frame, pfad);
        } else {
            pg::image::save(&app.canvas, pfad);
        }
        println!("{pfad}");
    };

    match spiel.as_deref() {
        Some("all") => {
            std::fs::create_dir_all(&ziel).ok();
            let keys: Vec<String> = app.games.iter().map(|g| g.key.to_string()).collect();
            for key in keys {
                // Wie vergleich.py: Spiel und Uebergang entstehen schon zur Bild-Uhr
                pg::set_fixed_ticks(Some(uhr));
                shell::post(Cmd::Launch { key: key.clone(), mode: mode.clone(), p2: None });
                app.befehle();
                lauf(&mut app, &mut uhr, &format!("{ziel}/{key}.png"));
                app.zum_menue();
            }
        }
        Some(key) => {
            if !app.games.iter().any(|g| g.key == key) {
                eprintln!("Pruefstand: Spiel '{key}' ist nicht eingetragen");
                std::process::exit(2);
            }
            // Wie vergleich.py (uhr[0] = ticks[0] vor dem Spielstart): sonst
            // laeuft der Bildschirm-Uebergang schon vor dem ersten Bild ab.
            pg::set_fixed_ticks(Some(uhr));
            shell::post(Cmd::Launch { key: key.to_string(), mode, p2: None });
            app.befehle();
            lauf(&mut app, &mut uhr, &ziel);
        }
        None => {
            // Wie bei Spielen: Bildschirm und Uebergang entstehen zur Bild-Uhr.
            pg::set_fixed_ticks(Some(uhr));
            let screen = env("PYGAMEZ_SCREEN").unwrap_or_default();
            let (name, arg) = match screen.split_once(':') {
                Some((n, a)) => (n.to_string(), Some(a.to_string())),
                None => (screen.clone(), None),
            };
            match name.as_str() {
                "options" => shell::post(Cmd::Options(None)),
                "language" => shell::post(Cmd::Language),
                "welcome" => shell::post(Cmd::Welcome),
                "progress" => shell::post(Cmd::Progress(arg)),
                "profiles" => shell::post(Cmd::Profiles),
                "wiki" => shell::post(Cmd::Wiki(arg, None)),
                "replays" => shell::post(Cmd::Replays),
                "pick" => shell::post(Cmd::PlayerPick { key: arg.unwrap_or_default(), mode: "multi".into() }),
                "levelintro" => {
                    let s = screens::profileview::LevelIntroScreen::new(app.menu_base());
                    app.show_screen(Box::new(s));
                }
                "pregame" => shell::post(Cmd::PreGame(arg.unwrap_or_default())),
                "matches" => {
                    // matches:<spiel>[:reiter] - wie vergleich.py (ohne Rueckweg)
                    let arg = arg.unwrap_or_default();
                    let (spiel, reiter) = arg.split_once(':').unwrap_or((arg.as_str(), "recent"));
                    if let Some(info) = app.games.iter().find(|g| g.key == spiel).copied() {
                        let s = screens::matchview::MatchScreen::new(app.menu_base(), info, None, reiter);
                        app.show_screen(Box::new(s));
                    }
                }
                _ => {}
            }
            app.befehle();
            lauf(&mut app, &mut uhr, &ziel);
        }
    }
    app.beenden();
}
