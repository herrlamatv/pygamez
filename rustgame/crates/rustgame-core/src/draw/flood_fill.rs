//! flood_fill.rs
//! =============
//! `pygame.draw.flood_fill` - die Farbeimer-Fuellung.
//!
//! Uebertragen aus `flood_fill_inner` in `src_c/draw.c`. Das Verfahren ist
//! eine **Breitensuche** ueber die Von-Neumann-Nachbarschaft (oben, unten,
//! links, rechts - keine Diagonalen), mit einem Bitfeld ueber dem
//! Clip-Rechteck, das sich merkt, welche Stellen bereits in der
//! Warteschlange standen. Der Kommentar im Original erklaert, warum nicht
//! einfach die geschriebene Farbe geprueft wird: *"we could check drawn
//! color, but that doesnt work for patterns"*.
//!
//! ## Die drei Stellen, an denen ein Nachbau daneben liegt
//!
//! 1. **Verglichen wird der rohe Pixelwert**, nicht die sichtbare Farbe.
//!    Im Original:
//!
//!    ```c
//!    SURF_GET_AT_FORMAT(old_color, surf, x1, y1, (Uint8 *)surf->pixels, pix);
//!    if (pattern == NULL && old_color == new_color) { /* nichts zu tun */ }
//!    ```
//!
//!    `SURF_GET_AT_FORMAT` liest das ganze 32-Bit-Wort, `new_color` kommt
//!    aus `pg_MappedColorFromObj`. Auf einer Flaeche **ohne** Alphakanal
//!    steht im vierten Byte darum eine Null, und ein Vergleich ueber
//!    `get_at` (das auf 255 maskiert) waere ein anderer Vergleich. Deshalb
//!    hier durchgehend [`Surface::get_at_mapped`] und [`Surface::abbilden`],
//!    nie `get_at`.
//!
//! 2. **Das Clip-Rechteck ist die Welt.** Nachbarn ausserhalb werden gar
//!    nicht erst betrachtet - die Fuellung laeuft also nicht "unter dem
//!    Clip hindurch" und taucht anderswo wieder auf.
//!
//! 3. **Ein Startpunkt ausserhalb des Clip-Rechtecks ist kein Fehler**,
//!    sondern tut einfach nichts (`goto flood_fill_finished`). Zurueck
//!    kommt dann `Rect(start, 0, 0)`.
//!
//! Gefuellt wird mit `unsafe_set_at`, also **ohne** Mischen: die Zielfarbe
//! landet roh in der Flaeche, wie bei allen Zeichenfunktionen ausser
//! `aaline`/`aacircle`.

use crate::color::IntoColor;
use crate::draw::line::Bereich;
use crate::rect::Rect;
use crate::surface::Surface;

/// Die Von-Neumann-Nachbarschaft, in der Reihenfolge des Originals.
///
/// Die Reihenfolge aendert am Ergebnis nichts (jede Stelle wird genau
/// einmal beschrieben), steht hier aber so, wie sie in `draw.c` steht.
const NACHBARN: [(i32, i32); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];

/// Der gemeinsame Kern von [`flood_fill`] und [`flood_fill_muster`].
///
/// `hole_farbe` liefert den rohen Pixelwert, der an `(x, y)` geschrieben
/// werden soll - bei einer festen Farbe immer derselbe, bei einem Muster
/// der Wert aus der Mustervorlage. `mit_muster` schaltet den Kurzschluss
/// "Startfarbe ist schon die Zielfarbe" ab, wie im Original.
fn fuelle(
    s: &mut Surface,
    start: (i32, i32),
    mit_muster: bool,
    hole_farbe: &dyn Fn(&Surface, i32, i32) -> u32,
) -> Rect {
    let (sx, sy) = start;
    let clip = s.get_clip();
    let mut b = Bereich::leer();

    // Startpunkt ausserhalb des Clip-Rechtecks: kein Fehler, nur nichts zu tun.
    if !(sx >= clip.x && sx < clip.x + clip.w && sy >= clip.y && sy < clip.y + clip.h) {
        return Rect::new(sx, sy, 0, 0);
    }

    // **Roh** gelesen - nicht ueber `get_at`, das den vierten Kanal einer
    // Flaeche ohne Alphakanal auf 255 maskieren wuerde.
    let alte_farbe = s.get_at_mapped((sx, sy));

    if !mit_muster && alte_farbe == hole_farbe(s, sx, sy) {
        return Rect::new(sx, sy, 0, 0);
    }

    // Bitfeld ueber dem Clip-Rechteck: wer stand schon in der Schlange?
    let breite = clip.w as usize;
    let mut gesehen = vec![false; breite * clip.h as usize];
    let marke = |x: i32, y: i32| (y - clip.y) as usize * breite + (x - clip.x) as usize;

    gesehen[marke(sx, sy)] = true;
    let mut rand = vec![(sx, sy)];
    let mut naechster: Vec<(i32, i32)> = Vec::new();

    while !rand.is_empty() {
        naechster.clear();
        for &(x, y) in &rand {
            // Erst beim Abarbeiten wird die Farbe geprueft, nicht beim
            // Einreihen - so steht es im Original.
            if s.get_at_mapped((x, y)) != alte_farbe {
                continue;
            }
            let neu = hole_farbe(s, x, y);
            s.set_at_ungeprueft((x, y), neu);
            b.punkt(x, y);

            for (dx, dy) in NACHBARN {
                let (nx, ny) = (x + dx, y + dy);
                if !(nx >= clip.x && nx < clip.x + clip.w && ny >= clip.y && ny < clip.y + clip.h) {
                    continue;
                }
                let i = marke(nx, ny);
                if gesehen[i] {
                    continue;
                }
                gesehen[i] = true;
                naechster.push((nx, ny));
            }
        }
        std::mem::swap(&mut rand, &mut naechster);
    }

    b.zu_rect().unwrap_or(Rect::new(sx, sy, 0, 0))
}

/// Fuellt die zusammenhaengende Flaeche gleicher Farbe um `start` herum.
///
/// Zurueck kommt das Rechteck der tatsaechlich beschriebenen Pixel; wurde
/// nichts geschrieben (Startpunkt ausserhalb des Clip-Rechtecks, oder dort
/// steht bereits die Zielfarbe), ist es `Rect(start, 0, 0)`.
pub fn flood_fill(s: &mut Surface, c: impl IntoColor, start: (i32, i32)) -> Rect {
    let farbe = s.abbilden(c);
    fuelle(s, start, false, &move |_s, _x, _y| farbe)
}

/// Fuellt mit einer **Mustervorlage** statt mit einer Farbe.
///
/// Das Muster wird gekachelt, und zwar an absoluten Flaechenkoordinaten
/// (`x % muster.w`, `y % muster.h`) - nicht relativ zum Startpunkt.
///
/// pygame wandelt die Vorlage vorher in das Format der Zielflaeche um
/// (`PG_ConvertSurface`). Hier tut das die Kette
/// `muster.get_at(..)` -> `s.abbilden(..)`: eine Vorlage ohne Alphakanal
/// liefert Alpha 255, und eine Zielflaeche ohne Alphakanal laesst es
/// wieder fallen.
///
/// Der Kurzschluss "Startfarbe ist schon die Zielfarbe" entfaellt mit
/// Muster - pygame prueft ihn nur fuer `pattern == NULL`.
pub fn flood_fill_muster(s: &mut Surface, muster: &Surface, start: (i32, i32)) -> Rect {
    let (mw, mh) = muster.get_size();
    if mw == 0 || mh == 0 {
        return Rect::new(start.0, start.1, 0, 0);
    }
    fuelle(s, start, true, &move |s, x, y| {
        let mx = x.rem_euclid(mw as i32);
        let my = y.rem_euclid(mh as i32);
        s.abbilden(muster.get_at((mx, my)))
    })
}

// ---------------------------------------------------------------------------
//  Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::circle::pruefhilfe::schwarz;
    use crate::surface::SRCALPHA;

    const ROT: (u8, u8, u8, u8) = (255, 0, 0, 255);

    /// **Der Vergleich geht ueber den rohen Pixelwert, Alphakanal
    /// eingeschlossen.** Ein Pixel, das sich nur im Alpha um eins
    /// unterscheidet, ist eine Wand - die Fuellung laeuft aussen herum.
    /// Gemessen an echtem pygame.
    #[test]
    fn ein_einziger_alphaschritt_ist_eine_wand() {
        let mut s = Surface::with_flags((4, 2), SRCALPHA);
        s.fill((5u8, 5u8, 5u8, 255u8));
        s.set_at((2, 0), (5u8, 5u8, 5u8, 254u8));
        let r = flood_fill(&mut s, (9u8, 9u8, 9u8, 9u8), (0, 0));
        assert_eq!(r, Rect::new(0, 0, 4, 2));
        let c = s.get_at((2, 0));
        assert_eq!(
            (c.r, c.g, c.b, c.a),
            (5, 5, 5, 254),
            "die Wand bleibt stehen"
        );
        // Ringsherum ist alles gefuellt - die Fuellung ist unten
        // vorbeigelaufen.
        for (x, y) in [(0, 0), (1, 0), (3, 0), (0, 1), (1, 1), (2, 1), (3, 1)] {
            let c = s.get_at((x, y));
            assert_eq!((c.r, c.g, c.b, c.a), (9, 9, 9, 9), "({x},{y})");
        }
    }

    /// Steht am Startpunkt bereits die Zielfarbe, passiert nichts - und
    /// ein Startpunkt ausserhalb ist **kein Fehler**, nur ein leeres
    /// Rechteck.
    #[test]
    fn kurzschluss_und_startpunkt_draussen() {
        let mut s = Surface::with_flags((4, 4), 0);
        s.fill((7u8, 8u8, 9u8));
        assert_eq!(
            flood_fill(&mut s, (7u8, 8u8, 9u8), (1, 1)),
            Rect::new(1, 1, 0, 0)
        );
        assert_eq!(
            flood_fill(&mut s, (1u8, 2u8, 3u8), (9, 9)),
            Rect::new(9, 9, 0, 0)
        );
        assert_eq!(s.get_at((1, 1)).r, 7, "unveraendert");
    }

    /// Das Clip-Rechteck ist die Welt: die Fuellung laeuft nicht darunter
    /// hindurch und taucht auch nicht daneben wieder auf.
    #[test]
    fn das_clip_rechteck_ist_die_welt() {
        let mut s = schwarz(8, 6);
        s.set_clip(Some(Rect::new(2, 1, 4, 3)));
        let r = flood_fill(&mut s, ROT, (3, 2));
        s.set_clip(None);
        assert_eq!(r, Rect::new(2, 1, 4, 3));
        for y in 0..6 {
            for x in 0..8 {
                let drin = (2..6).contains(&x) && (1..4).contains(&y);
                assert_eq!(s.get_at((x, y)).r, if drin { 255 } else { 0 }, "({x},{y})");
            }
        }
    }

    /// Die Fuellung laeuft nur ueber Kanten, nicht ueber Ecken (Von
    /// Neumann). Eine Diagonale aus Wandpixeln trennt darum sauber.
    #[test]
    fn diagonalen_werden_nicht_uebersprungen() {
        let mut s = schwarz(4, 4);
        for i in 0..4 {
            s.set_at((i, i), (0u8, 0u8, 255u8));
        }
        let r = flood_fill(&mut s, ROT, (3, 0));
        assert_eq!(r, Rect::new(1, 0, 3, 3));
        assert_eq!(s.get_at((3, 0)).r, 255);
        assert_eq!(s.get_at((0, 3)).r, 0, "die andere Seite bleibt schwarz");
    }

    /// Die Mustervorlage wird an **absoluten** Flaechenkoordinaten
    /// gekachelt (`x % w`, `y % h`), nicht relativ zum Startpunkt.
    #[test]
    fn das_muster_kachelt_an_der_flaeche() {
        let mut muster = Surface::with_flags((2, 2), 0);
        muster.fill((10u8, 20u8, 30u8));
        muster.set_at((1, 0), (40u8, 50u8, 60u8));
        muster.set_at((0, 1), (70u8, 80u8, 90u8));

        let mut s = schwarz(6, 4);
        let r = flood_fill_muster(&mut s, &muster, (0, 0));
        assert_eq!(r, Rect::new(0, 0, 6, 4));
        for y in 0..4 {
            for x in 0..6 {
                let c = s.get_at((x, y));
                let m = muster.get_at((x % 2, y % 2));
                assert_eq!((c.r, c.g, c.b), (m.r, m.g, m.b), "({x},{y})");
            }
        }
    }

    /// Mit Muster gibt es **keinen** Kurzschluss "Startfarbe ist schon die
    /// Zielfarbe" - pygame prueft ihn nur fuer eine feste Farbe.
    #[test]
    fn mit_muster_gibt_es_keinen_kurzschluss() {
        let mut muster = Surface::with_flags((1, 1), 0);
        muster.fill((0u8, 0u8, 0u8));
        let mut s = schwarz(3, 3);
        let r = flood_fill_muster(&mut s, &muster, (1, 1));
        assert_eq!(
            r,
            Rect::new(0, 0, 3, 3),
            "trotz gleicher Farbe wird die ganze Flaeche durchlaufen"
        );
    }
}
