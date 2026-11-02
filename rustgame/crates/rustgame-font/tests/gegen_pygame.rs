//! Messwerte aus echtem pygame-ce 2.5.7, festgenagelt.
//!
//! Alle Zahlen hier stammen aus einem Lauf gegen die Vorlage:
//!
//! ```text
//! cd C:\Users\herrl\PycharmProjects\pygamez
//! set SDL_VIDEODRIVER=dummy
//! .venv\Scripts\python.exe -c "import pygame, os; pygame.font.init(); ..."
//! ```
//!
//! Gebraucht wird dafuer `freesansbold.ttf`. Liegt die Datei nicht da (siehe
//! [`rustgame_font::default_font_path`]), werden die Pruefungen
//! uebersprungen statt zu scheitern - ein fehlendes Beiwerk soll den Bau
//! nicht rot faerben.

use rustgame_core::color::Color;
use rustgame_font::{default_font_path, Font};

fn schrift(groesse: i32) -> Option<Font> {
    let p = default_font_path()?;
    Font::new(p, groesse).ok()
}

/// (Groesse, ascent, descent, height, linesize) - gemessen an pygame.
const METRIKEN: &[(i32, i32, i32, i32, i32)] = &[
    (8, 7, -1, 8, 9),
    (10, 8, -2, 10, 11),
    (11, 9, -2, 11, 12),
    (12, 10, -2, 12, 14),
    (14, 12, -2, 14, 16),
    (16, 13, -3, 16, 18),
    (18, 15, -3, 18, 20),
    (20, 16, -4, 20, 22),
    (24, 20, -4, 24, 27),
    (28, 23, -5, 28, 31),
    (32, 26, -6, 32, 35),
    (36, 29, -7, 36, 40),
    (40, 32, -8, 40, 44),
    (48, 39, -9, 48, 53),
    (64, 52, -12, 64, 70),
    (72, 58, -14, 72, 79),
    (96, 77, -19, 96, 105),
];

#[test]
fn metriken_treffen_exakt() {
    for &(g, asc, desc, h, ls) in METRIKEN {
        let Some(f) = schrift(g) else {
            eprintln!("freesansbold.ttf nicht gefunden - uebersprungen");
            return;
        };
        assert_eq!(
            (
                f.get_ascent(),
                f.get_descent(),
                f.get_height(),
                f.get_linesize()
            ),
            (asc, desc, h, ls),
            "freesansbold in Groesse {g}"
        );
    }
}

/// Der Unterstrich schiebt `get_height()` nach oben - und zwar um einen
/// Betrag, der nur stimmt, wenn `underline_position` wie in FreeType um die
/// halbe Strichdicke verschoben wird.
#[test]
fn unterstrich_hebt_die_hoehe() {
    for &(g, soll) in &[(8, 9), (16, 17), (48, 51)] {
        let Some(mut f) = schrift(g) else { return };
        assert_eq!(
            f.get_height(),
            METRIKEN.iter().find(|m| m.0 == g).unwrap().3
        );
        f.set_underline(true);
        assert_eq!(f.get_height(), soll, "mit Unterstrich in Groesse {g}");
        f.set_underline(false);
        f.set_strikethrough(true);
        assert_eq!(
            f.get_height(),
            METRIKEN.iter().find(|m| m.0 == g).unwrap().3,
            "Durchstreichung aendert die Hoehe nicht"
        );
    }
}

/// `pygame.font.Font(None, 16)` ist freesansbold in Groesse 11 - nicht 16.
#[test]
fn standardschrift_wird_kleingerechnet() {
    let Some(p) = default_font_path() else { return };
    let bytes = std::fs::read(p).unwrap();
    let f = Font::default_font(bytes, 16).unwrap();
    assert_eq!(f.get_point_size(), 11);
    assert_eq!((f.get_height(), f.get_linesize()), (11, 12));
}

/// Der Aufruf, der im Projekt 1002-mal vorkommt.
#[test]
fn render_mit_kantenglaettung_ohne_hintergrund() {
    let Some(f) = schrift(16) else { return };
    let s = f.render("Hi", true, Color::rgb(255, 0, 0), None).unwrap();

    // Hoehe ist linesize, nicht height - gemessen: 18 bei Groesse 16.
    assert_eq!(s.get_height(), 18);
    assert!(s.has_alpha(), "SRCALPHA muss gesetzt sein");
    assert!(s.get_colorkey().is_none());

    // Breite: pygame misst 16, erlaubt sind +-2.
    let (w, _) = f.size("Hi");
    assert!((w as i32 - 16).abs() <= 2, "Breite {w} statt 16 +-2");
    assert_eq!(
        w,
        s.get_width(),
        "size() und Flaeche muessen zusammenpassen"
    );

    // Die Farbe steht konstant im RGB-Teil, die Deckung im Alphakanal -
    // auch dort, wo gar nichts gezeichnet wurde.
    let mut gesehen = false;
    for y in 0..s.get_height() as i32 {
        for x in 0..s.get_width() as i32 {
            let c = s.get_at((x, y));
            assert_eq!((c.r, c.g, c.b), (255, 0, 0), "RGB bei ({x}, {y})");
            gesehen |= c.a > 0;
        }
    }
    assert!(gesehen, "die Flaeche ist leer geblieben");
}

#[test]
fn render_ohne_kantenglaettung_hat_schluesselfarbe() {
    let Some(f) = schrift(16) else { return };
    let s = f.render("Hi", false, Color::rgb(255, 0, 0), None).unwrap();
    assert!(!s.has_alpha(), "ohne Kantenglaettung kein SRCALPHA");
    // SDL_ttf legt die Gegenfarbe auf Palettenplatz 0 und macht sie
    // durchsichtig: 255 - (255, 0, 0) = (0, 255, 255).
    assert_eq!(s.get_colorkey(), Some(Color::rgb(0, 255, 255)));
    assert_eq!(s.get_height(), 18);
}

#[test]
fn render_mit_hintergrund_ist_undurchsichtig() {
    let Some(f) = schrift(16) else { return };
    for aa in [true, false] {
        let s = f
            .render("Hi", aa, Color::rgb(255, 0, 0), Some(Color::rgb(0, 0, 255)))
            .unwrap();
        assert!(!s.has_alpha(), "mit Hintergrund kein SRCALPHA (aa={aa})");
        assert!(s.get_colorkey().is_none(), "und keine Schluesselfarbe");
        assert_eq!(s.get_at((0, 0)).a, 255);
    }
}

/// Leerer Text: `0 x get_height()`, ohne Alphakanal - in allen drei
/// Betriebsarten.
#[test]
fn leerer_text_gibt_eine_flaeche_ohne_breite() {
    let Some(f) = schrift(16) else { return };
    for (aa, bg) in [
        (true, None),
        (false, None),
        (true, Some(Color::rgb(0, 0, 0))),
    ] {
        let s = f.render("", aa, Color::rgb(255, 255, 255), bg).unwrap();
        assert_eq!(s.get_size(), (0, 16));
        assert!(!s.has_alpha());
    }
}

/// Auch ohne Umbruchbreite trennt `\n` - pygame ruft die umbrechende
/// Variante von SDL_ttf auf. Gemessen bei Groesse 16: zwei Zeilen sind
/// 36 Pixel hoch, drei 54.
#[test]
fn zeilenumbrueche_werden_gesetzt() {
    let Some(f) = schrift(16) else { return };
    assert_eq!(
        f.render("a\nb", true, Color::rgb(255, 255, 255), None)
            .unwrap()
            .get_height(),
        36
    );
    assert_eq!(
        f.render("a\nbb\nccc", true, Color::rgb(255, 255, 255), None)
            .unwrap()
            .get_height(),
        54
    );
    // Ein abschliessendes \n macht **keine** leere Schlusszeile auf.
    assert_eq!(
        f.render("a\n", true, Color::rgb(255, 255, 255), None)
            .unwrap()
            .get_height(),
        18
    );
    // `size()` kennt die Umbrueche dagegen nicht.
    assert_eq!(f.size("a\nb").1, 16);
}

/// Die Vorschuebe kommen aus `hmtx` und muessen exakt sein.
#[test]
fn vorschuebe_treffen_exakt() {
    let Some(f) = schrift(16) else { return };
    let m = f.metrics("Hi");
    assert_eq!(m[0].unwrap().4, 12, "H");
    assert_eq!(m[1].unwrap().4, 4, "i");
    let m = f.metrics("Ax g");
    assert_eq!(m[0].unwrap().4, 12, "A");
    assert_eq!(m[1].unwrap().4, 9, "x");
    assert_eq!(m[2].unwrap().4, 4, "Leerzeichen");
    assert_eq!(m[3].unwrap().4, 10, "g");
}

/// Kerning: freesansbold hat kein GPOS, also gilt die alte kern-Tabelle -
/// und die hat fuer (x, x) ganze +100 Einheiten. Gemessen: bei Groesse 32
/// ist `size("x" * 11)` um 212 Pixel breiter als `size("x")`, nicht um 180.
#[test]
fn kerning_wird_angewandt() {
    let Some(f) = schrift(32) else { return };
    let eins = f.size("x").0 as i32;
    let elf = f.size(&"x".repeat(11)).0 as i32;
    let schritte = elf - eins;
    assert!(
        (schritte - 212).abs() <= 2,
        "10 Schritte mit Kerning: {schritte} statt 212 (ohne Kerning waeren es 180)"
    );
}

#[test]
fn render_to_setzt_auf_eine_vorhandene_flaeche() {
    let Some(f) = schrift(16) else { return };
    let mut ziel = rustgame_core::Surface::new((100, 40));
    let r = f
        .render_to(
            &mut ziel,
            (5, 5),
            "Hi",
            true,
            Color::rgb(255, 255, 255),
            None,
        )
        .unwrap();
    assert_eq!((r.x, r.y), (5, 5));
    assert_eq!(r.h, 18);
    // Irgendwo im beschriebenen Rechteck muss jetzt etwas Helles stehen.
    let mut hell = false;
    for y in r.y..r.y + r.h {
        for x in r.x..r.x + r.w {
            hell |= ziel.get_at((x, y)).r > 100;
        }
    }
    assert!(hell, "render_to hat nichts hinterlassen");
}

#[test]
fn fett_macht_breiter() {
    let Some(mut f) = schrift(32) else { return };
    let schmal = f.size("Hallo").0;
    f.set_bold(true);
    let breit = f.size("Hallo").0;
    // glyph_overhang = ptsize / 10 = 3, fuenf Zeichen -> 15 Pixel mehr.
    let zu = breit as i32 - schmal as i32;
    assert!((zu - 15).abs() <= 2, "fett bringt {zu} Pixel statt 15");
}

#[test]
fn systemschriften_sind_auffindbar() {
    if !cfg!(windows) {
        return;
    }
    let liste = rustgame_font::get_fonts();
    assert!(!liste.is_empty(), "keine Systemschrift gefunden");
    assert!(liste.iter().all(|n| n.chars().all(|c| c.is_alphanumeric())));
    // Die Kommaliste nimmt den ersten Namen, der sich aufloesen laesst.
    let a = rustgame_font::match_font("gibtesnicht,arial", false, false);
    let b = rustgame_font::match_font("arial", false, false);
    assert_eq!(a, b);
    assert!(rustgame_font::match_font("gibtesnicht", false, false).is_none());
}

#[test]
fn init_und_quit_schalten_den_merker() {
    rustgame_font::init();
    assert!(rustgame_font::get_init());
    rustgame_font::quit();
    assert!(!rustgame_font::get_init());
    rustgame_font::init();
    assert_eq!(rustgame_font::get_default_font(), "freesansbold.ttf");
}

/// `SysFont` mit Kommaliste: der erste aufloesbare Name gewinnt, und der
/// Rueckfall ist die mitgelieferte Schrift - dort dann mit dem 0.6875-Faktor,
/// weil pygame intern `Font(None, size)` baut.
#[test]
fn sysfont_nimmt_den_ersten_treffer() {
    if !cfg!(windows) {
        return;
    }
    let a = rustgame_font::sys_font("gibtesnicht,arial,consolas", 20, false, false).unwrap();
    let b = rustgame_font::sys_font("arial", 20, false, false).unwrap();
    assert_eq!(a.get_height(), b.get_height());
    assert_eq!(a.get_linesize(), b.get_linesize());

    if default_font_path().is_some() {
        let c = rustgame_font::sys_font("gibtesnicht", 16, false, false).unwrap();
        assert_eq!(
            c.get_point_size(),
            11,
            "Rueckfall geht ueber Font(None, size)"
        );
    }
}
