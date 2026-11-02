//! line.rs
//! =======
//! `pygame.draw.line` und `pygame.draw.lines` - 285 bzw. 22 Aufrufe in der
//! Sammlung, 186 davon mit einer Strichstaerke ueber 1.
//!
//! Nachgebildet ist der Rechenweg aus `src_c/draw.c` von pygame-ce 2.5.7,
//! nicht eine "saubere" Linie. Fuenf Eigenheiten, die ein gutgemeinter
//! Nachbau verfehlen wuerde und die hier absichtlich mitwandern:
//!
//! * **Die dicke Linie ist kein gedrehtes Rechteck.** pygame laeuft die
//!   Bresenham-Linie ab und setzt je Schritt eine *achsenparallele* Spanne -
//!   quer zur laengeren Achse. Die Enden sind darum flach und stehen
//!   senkrecht, nicht rechtwinklig zur Linie.
//! * **Bei gerader Staerke liegt die Linie nicht mittig.** `width = 3` deckt
//!   `y-1 ..= y+1` ab, `width = 4` aber `y-1 ..= y+2`. Das haelt in
//!   `linie_breit` die Groesse `extra` fest (im Original `extra_width`).
//! * **Die Farbe wird roh geschrieben.** `draw.line` mit `(255,0,0,128)`
//!   hinterlaesst auf einer SRCALPHA-Flaeche genau diesen Wert; es gibt kein
//!   Alpha-Mischen wie beim Blit. Gemessen an echtem pygame.
//! * **`width < 1` zeichnet gar nichts** und gibt `Rect(start, 0, 0)` zurueck.
//! * **`lines` zeichnet jedes Teilstueck einzeln.** An den Knicken bleiben
//!   bei dicken Linien Kerben stehen - kein Fehler, sondern pygame.
//!
//! Das Clipping aendert die Pixelwahl **nicht**: der Bresenham-Lauf geht
//! ueber die volle Strecke, geschrieben wird nur, was ins Clip-Rechteck
//! faellt. Gemessen: dieselbe Linie klein gezeichnet und gross gezeichnet
//! und dann zugeschnitten ergibt Pixel fuer Pixel dasselbe.
//!
//! Zurueck kommt immer die Umgrenzung der **tatsaechlich** gesetzten Pixel.

use crate::color::{Color, IntoColor};
use crate::rect::Rect;
use crate::surface::Surface;

// ---------------------------------------------------------------------------
//  Buchfuehrung ueber das Gezeichnete
// ---------------------------------------------------------------------------

/// Sammelt, welche Pixel wirklich beschrieben wurden.
///
/// pygame fuehrt waehrend des Zeichnens Buch (`drawn_area`) und baut daraus
/// das Rueckgabe-Rechteck. Vorher ausrechnen laesst es sich nicht: was
/// ausserhalb des Clip-Rechtecks liegt, zaehlt nicht mit.
///
/// Liegt bewusst hier und nicht in `polygon.rs` - beide Dateien brauchen es,
/// und ein gemeinsames Zeichen-Grundmodul gibt es (noch) nicht.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Bereich {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl Bereich {
    /// Noch nichts gezeichnet - die Grenzen stehen verkehrt herum.
    pub(crate) fn leer() -> Bereich {
        Bereich {
            x1: i32::MAX,
            y1: i32::MAX,
            x2: i32::MIN,
            y2: i32::MIN,
        }
    }

    pub(crate) fn punkt(&mut self, x: i32, y: i32) {
        self.x1 = self.x1.min(x);
        self.y1 = self.y1.min(y);
        self.x2 = self.x2.max(x);
        self.y2 = self.y2.max(y);
    }

    /// Wie [`Bereich::punkt`], aber fuer eine bereits sortierte Strecke.
    pub(crate) fn strecke(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        self.x1 = self.x1.min(x1);
        self.y1 = self.y1.min(y1);
        self.x2 = self.x2.max(x2);
        self.y2 = self.y2.max(y2);
    }

    /// `None`, solange kein einziges Pixel gesetzt wurde.
    pub(crate) fn zu_rect(&self) -> Option<Rect> {
        if self.x1 == i32::MAX || self.y1 == i32::MAX || self.x2 == i32::MIN || self.y2 == i32::MIN
        {
            None
        } else {
            Some(Rect::new(
                self.x1,
                self.y1,
                self.x2 - self.x1 + 1,
                self.y2 - self.y1 + 1,
            ))
        }
    }
}

// ---------------------------------------------------------------------------
//  Kleinste Bausteine
// ---------------------------------------------------------------------------

/// Ein Pixel setzen und, falls es im Clip-Rechteck lag, vermerken.
///
/// `Surface::set_at` sagt nicht, ob geschrieben wurde, darum wird vorher
/// gefragt - genau wie pygames `set_and_check_rect`.
pub(crate) fn setze(s: &mut Surface, b: &mut Bereich, x: i32, y: i32, farbe: Color) {
    if s.get_clip().collidepoint((x, y)) {
        s.set_at((x, y), farbe);
        b.punkt(x, y);
    }
}

/// Beschneidet eine waagerechte Strecke am Clip-Rechteck.
///
/// `None`, wenn nichts uebrig bleibt. Die Spanne kommt aufsteigend zurueck
/// und liegt ganz im Clip-Rechteck (pygames `drawhorzlineclipbounding`).
pub(crate) fn spanne(clip: Rect, x1: i32, y: i32, x2: i32) -> Option<(i32, i32)> {
    if y < clip.y || y >= clip.y + clip.h {
        return None;
    }
    let (mut a, mut e) = if x2 < x1 { (x2, x1) } else { (x1, x2) };
    a = a.max(clip.x);
    e = e.min(clip.x + clip.w - 1);
    if e < clip.x || a >= clip.x + clip.w {
        return None;
    }
    Some((a, e))
}

/// Waagerechte Strecke, bereits beschnitten - ohne Buchfuehrung.
fn hlinie_roh(s: &mut Surface, farbe: Color, x1: i32, y: i32, x2: i32) {
    s.fill_rect(farbe, Rect::new(x1, y, x2 - x1 + 1, 1));
}

/// Senkrechte Strecke, bereits beschnitten - ohne Buchfuehrung.
fn vlinie_roh(s: &mut Surface, farbe: Color, y1: i32, x: i32, y2: i32) {
    s.fill_rect(farbe, Rect::new(x, y1, 1, y2 - y1 + 1));
}

/// Waagerechte Strecke mit Beschnitt **und** Buchfuehrung.
pub(crate) fn hlinie_bereich(
    s: &mut Surface,
    b: &mut Bereich,
    farbe: Color,
    x1: i32,
    y: i32,
    x2: i32,
) {
    if let Some((a, e)) = spanne(s.get_clip(), x1, y, x2) {
        b.strecke(a, y, e, y);
        hlinie_roh(s, farbe, a, y, e);
    }
}

// ---------------------------------------------------------------------------
//  Der Bresenham-Lauf
// ---------------------------------------------------------------------------

/// Der Startfehler, genau wie im C-Original.
///
/// Bei senkrechtlastigen Linien ist der Wert negativ, und dort kommt es auf
/// die Rundungsrichtung an: Rusts `/` schneidet wie C Richtung Null ab,
/// `-3 / 2` ergibt also `-1` und nicht `-2`. Mit der anderen Richtung
/// verschoebe sich die halbe Linie um ein Pixel.
#[inline]
fn startfehler(dx: i32, dy: i32) -> i32 {
    (if dx > dy { dx } else { -dy }) / 2
}

/// Einzelne Linie der Staerke 1 (pygames `draw_line`).
///
/// Waagerecht und senkrecht laufen an Bresenham vorbei - im Original, weil
/// es schneller ist, hier, weil es sonst ein anderes Pixelmuster gaebe.
pub(crate) fn linie(
    s: &mut Surface,
    b: &mut Bereich,
    farbe: Color,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
) {
    if x1 == x2 && y1 == y2 {
        setze(s, b, x1, y1, farbe);
        return;
    }
    if y1 == y2 {
        let schritt = if x1 < x2 { 1 } else { -1 };
        for i in 0..=(x1 - x2).abs() {
            setze(s, b, x1 + schritt * i, y1, farbe);
        }
        return;
    }
    if x1 == x2 {
        let schritt = if y1 < y2 { 1 } else { -1 };
        for i in 0..=(y1 - y2).abs() {
            setze(s, b, x1, y1 + schritt * i, farbe);
        }
        return;
    }
    let (mut x, mut y) = (x1, y1);
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = startfehler(dx, dy);
    while x != x2 || y != y2 {
        setze(s, b, x, y, farbe);
        let e2 = err;
        if e2 > -dx {
            err -= dy;
            x += sx;
        }
        if e2 < dy {
            err += dx;
            y += sy;
        }
    }
    setze(s, b, x2, y2, farbe);
}

/// Grobpruefung, ob eine dicke Linie das Clip-Rechteck ueberhaupt trifft
/// (pygames `clip_line`). Beschneidet nichts, sagt nur ja oder nein.
fn linie_sichtbar(clip: Rect, x1: i32, y1: i32, x2: i32, y2: i32, halbe: i32, quer: bool) -> bool {
    let (links, rechts, oben, unten) = if quer {
        (
            x1.min(x2) - halbe,
            x1.max(x2) + halbe,
            y1.min(y2),
            y1.max(y2),
        )
    } else {
        (
            x1.min(x2),
            x1.max(x2),
            y1.min(y2) - halbe,
            y1.max(y2) + halbe,
        )
    };
    !(clip.x > rechts || clip.y > unten || clip.x + clip.w <= links || clip.y + clip.h <= oben)
}

/// Dicke Linie (pygames `draw_line_width`).
///
/// Die Dicke waechst **quer zur laengeren Achse**: eine eher senkrechte
/// Linie wird durch waagerechte Spannen dick, eine eher waagerechte durch
/// senkrechte. Bei Gleichstand (`dx == dy`) entscheidet pygame fuer die
/// waagerechte Spanne - und ein Punkt (`dx == dy == 0`) faellt ebenfalls
/// dorthin, weshalb `line(p, p, 3)` drei Pixel **nebeneinander** setzt.
#[allow(clippy::too_many_arguments)]
pub(crate) fn linie_breit(
    s: &mut Surface,
    b: &mut Bereich,
    farbe: Color,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    breite: i32,
) {
    if breite < 1 {
        return;
    }
    if breite == 1 {
        linie(s, b, farbe, x1, y1, x2, y2);
        return;
    }
    let clip = s.get_clip();
    let ende_x = clip.x + clip.w - 1;
    let ende_y = clip.y + clip.h - 1;
    // Ungerade Staerken sitzen mittig, gerade haengen ein Pixel nach.
    let extra = 1 - (breite % 2);
    let halbe = breite / 2;
    let quer = (x1 - x2).abs() <= (y1 - y2).abs();

    if !linie_sichtbar(clip, x1, y1, x2, y2, halbe, quer) {
        return;
    }

    if x1 == x2 && y1 == y2 {
        // Einzelner Punkt: eine waagerechte Spanne. Dass `y1` im
        // Clip-Rechteck liegt, hat `linie_sichtbar` bereits geprueft.
        let anfang = ((x1 - halbe) + extra).max(clip.x);
        let schluss = ende_x.min(x1 + halbe);
        if anfang <= schluss {
            hlinie_roh(s, farbe, anfang, y1, schluss);
            b.strecke(anfang, y1, schluss, y1);
        }
        return;
    }

    let (mut x, mut y) = (x1, y1);
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = if x2 > x1 { 1 } else { -1 };
    let sy = if y2 > y1 { 1 } else { -1 };
    let mut err = startfehler(dx, dy);

    if quer {
        let schluss_y = y2.saturating_add(sy);
        while y != schluss_y {
            if clip.y <= y && y <= ende_y {
                let anfang = ((x - halbe) + extra).max(clip.x);
                let schluss = ende_x.min(x + halbe);
                if anfang <= schluss {
                    hlinie_roh(s, farbe, anfang, y, schluss);
                    b.strecke(anfang, y, schluss, y);
                }
            }
            let e2 = err;
            if e2 > -dx {
                err -= dy;
                x += sx;
            }
            if e2 < dy {
                err += dx;
                y += sy;
            }
        }
    } else {
        let schluss_x = x2.saturating_add(sx);
        while x != schluss_x {
            if clip.x <= x && x <= ende_x {
                let anfang = ((y - halbe) + extra).max(clip.y);
                let schluss = ende_y.min(y + halbe);
                if anfang <= schluss {
                    vlinie_roh(s, farbe, anfang, x, schluss);
                    b.strecke(x, anfang, x, schluss);
                }
            }
            let e2 = err;
            if e2 > -dx {
                err -= dy;
                x += sx;
            }
            if e2 < dy {
                err += dx;
                y += sy;
            }
        }
    }
}

// ---------------------------------------------------------------------------
//  Oeffentliche Zeichenfunktionen
// ---------------------------------------------------------------------------

/// Zeichnet eine Strecke von `a` nach `b`.
///
/// `width` ist die Strichstaerke; `width < 1` zeichnet nichts und gibt ein
/// leeres Rechteck am Startpunkt zurueck. Sonst kommt die Umgrenzung der
/// tatsaechlich gesetzten Pixel zurueck, auf das Clip-Rechteck beschnitten.
pub fn line(s: &mut Surface, c: impl IntoColor, a: (i32, i32), b: (i32, i32), width: i32) -> Rect {
    if width < 1 {
        return Rect::new(a.0, a.1, 0, 0);
    }
    let mut bereich = Bereich::leer();
    linie_breit(s, &mut bereich, c.into_color(), a.0, a.1, b.0, b.1, width);
    bereich.zu_rect().unwrap_or(Rect::new(a.0, a.1, 0, 0))
}

/// Linienzug durch alle Punkte; `closed` haengt den Rueckweg an.
///
/// Jedes Teilstueck wird **einzeln** gezeichnet. Bei `width > 1` bleiben an
/// den Knicken darum Kerben stehen - genau so macht es pygame, und ein
/// "verbesserter" Nachbau waere an dieser Stelle falsch.
///
/// Der Rueckweg entfaellt bei nur zwei Punkten (er laege auf dem Hinweg).
/// Weniger als zwei Punkte zeichnen nichts; pygame wirft dort einen
/// `ValueError`, den es in dieser Signatur nicht gibt.
pub fn lines(
    s: &mut Surface,
    c: impl IntoColor,
    closed: bool,
    pts: &[(i32, i32)],
    width: i32,
) -> Rect {
    let Some(&(x0, y0)) = pts.first() else {
        return Rect::new(0, 0, 0, 0);
    };
    if pts.len() < 2 || width < 1 {
        return Rect::new(x0, y0, 0, 0);
    }
    let farbe = c.into_color();
    let mut bereich = Bereich::leer();
    for paar in pts.windows(2) {
        let (a, b) = (paar[0], paar[1]);
        linie_breit(s, &mut bereich, farbe, a.0, a.1, b.0, b.1, width);
    }
    if closed && pts.len() > 2 {
        let e = pts[pts.len() - 1];
        linie_breit(s, &mut bereich, farbe, e.0, e.1, x0, y0, width);
    }
    bereich.zu_rect().unwrap_or(Rect::new(x0, y0, 0, 0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::circle::pruefhilfe::{
        gleich_wie_pygame, schwarz as leinwand, zeichenbild as muster,
    };
    use crate::surface::SRCALPHA;

    const WEISS: (u8, u8, u8, u8) = (255, 255, 255, 255);

    // ----- die acht Referenzfaelle aus tools/cases.py ---------------------

    #[test]
    fn referenz_line_waagerecht() {
        let mut s = leinwand(16, 8);
        assert_eq!(
            line(&mut s, WEISS, (2, 4), (13, 4), 1),
            Rect::new(2, 4, 12, 1)
        );
        gleich_wie_pygame("line_waagerecht", &s);
    }

    #[test]
    fn referenz_line_senkrecht() {
        let mut s = leinwand(8, 16);
        assert_eq!(
            line(&mut s, WEISS, (4, 2), (4, 13), 1),
            Rect::new(4, 2, 1, 12)
        );
        gleich_wie_pygame("line_senkrecht", &s);
    }

    #[test]
    fn referenz_line_schraeg() {
        let mut s = leinwand(16, 16);
        assert_eq!(
            line(&mut s, WEISS, (1, 1), (14, 12), 1),
            Rect::new(1, 1, 14, 12)
        );
        gleich_wie_pygame("line_schraeg", &s);
    }

    #[test]
    fn referenz_line_dick() {
        let mut s = leinwand(16, 12);
        assert_eq!(
            line(&mut s, WEISS, (1, 5), (14, 5), 3),
            Rect::new(1, 4, 14, 3)
        );
        gleich_wie_pygame("line_dick", &s);
    }

    #[test]
    fn referenz_line_dick_schraeg() {
        let mut s = leinwand(20, 20);
        assert_eq!(
            line(&mut s, WEISS, (2, 2), (17, 15), 4),
            Rect::new(2, 1, 16, 17)
        );
        gleich_wie_pygame("line_dick_schraeg", &s);
    }

    #[test]
    fn referenz_line_punkt() {
        let mut s = leinwand(8, 8);
        assert_eq!(
            line(&mut s, WEISS, (4, 4), (4, 4), 1),
            Rect::new(4, 4, 1, 1)
        );
        gleich_wie_pygame("line_punkt", &s);
    }

    #[test]
    fn referenz_lines_offen() {
        let mut s = leinwand(20, 20);
        let pts = [(2, 2), (10, 15), (17, 4)];
        assert_eq!(
            lines(&mut s, WEISS, false, &pts, 1),
            Rect::new(2, 2, 16, 14)
        );
        gleich_wie_pygame("lines_offen", &s);
    }

    #[test]
    fn referenz_lines_geschlossen() {
        let mut s = leinwand(20, 20);
        let pts = [(2, 2), (10, 15), (17, 4)];
        assert_eq!(lines(&mut s, WEISS, true, &pts, 1), Rect::new(2, 2, 16, 14));
        gleich_wie_pygame("lines_geschlossen", &s);
    }

    // ----- Randfaelle -----------------------------------------------------

    #[test]
    fn waagerecht_und_senkrecht_treffen_ihre_endpunkte() {
        let mut s = leinwand(16, 8);
        let r = line(&mut s, (255, 255, 255), (2, 4), (13, 4), 1);
        assert_eq!(r, Rect::new(2, 4, 12, 1), "gemessen an pygame");
        assert_eq!(s.get_at((2, 4)).r, 255);
        assert_eq!(s.get_at((13, 4)).r, 255, "der Endpunkt gehoert dazu");
        assert_eq!(s.get_at((14, 4)).r, 0);

        let mut s = leinwand(8, 16);
        let r = line(&mut s, (255, 255, 255), (4, 2), (4, 13), 1);
        assert_eq!(r, Rect::new(4, 2, 1, 12));
        assert_eq!(s.get_at((4, 13)).r, 255);
    }

    #[test]
    fn die_schraege_linie_trifft_die_gemessenen_stufen() {
        // Gemessen an echtem pygame-ce 2.5.7: (1,1) -> (14,12).
        let mut s = leinwand(16, 16);
        let r = line(&mut s, (255, 255, 255), (1, 1), (14, 12), 1);
        assert_eq!(r, Rect::new(1, 1, 14, 12));
        let erwartet = [
            (1, 1),
            (2, 2),
            (3, 3),
            (4, 4),
            (5, 4),
            (6, 5),
            (7, 6),
            (8, 7),
            (9, 8),
            (10, 9),
            (11, 9),
            (12, 10),
            (13, 11),
            (14, 12),
        ];
        for (x, y) in erwartet {
            assert_eq!(s.get_at((x, y)).r, 255, "Pixel ({x},{y}) fehlt");
        }
        let anzahl: usize = muster(&s).iter().map(|z| z.matches('#').count()).sum();
        assert_eq!(anzahl, erwartet.len(), "kein Pixel zu viel");
    }

    #[test]
    fn dicke_linien_wachsen_quer_zur_laengeren_achse() {
        // Gemessen: width 3 deckt y-1..y+1 ab, width 4 aber y-1..y+2.
        let mut s = leinwand(16, 12);
        let r = line(&mut s, (255, 255, 255), (1, 5), (14, 5), 3);
        assert_eq!(r, Rect::new(1, 4, 14, 3));
        assert_eq!(s.get_at((7, 4)).r, 255);
        assert_eq!(s.get_at((7, 6)).r, 255);
        assert_eq!(s.get_at((7, 7)).r, 0);

        let mut s = leinwand(16, 12);
        let r = line(&mut s, (255, 255, 255), (1, 5), (14, 5), 4);
        assert_eq!(
            r,
            Rect::new(1, 4, 14, 4),
            "gerade Staerke haengt nach unten"
        );

        // Senkrechte Linie: dieselbe Unwucht, nur in x.
        let mut s = leinwand(12, 16);
        let r = line(&mut s, (255, 255, 255), (5, 1), (5, 14), 4);
        assert_eq!(r, Rect::new(4, 1, 4, 14));
    }

    #[test]
    fn ein_punkt_wird_zur_waagerechten_spanne() {
        // Gemessen: line(p, p, 3) setzt drei Pixel nebeneinander, nicht
        // uebereinander - der Punkt faellt in pygames x-Zweig.
        let mut s = leinwand(8, 8);
        let r = line(&mut s, (255, 255, 255), (4, 4), (4, 4), 1);
        assert_eq!(r, Rect::new(4, 4, 1, 1));

        let mut s = leinwand(8, 8);
        let r = line(&mut s, (255, 255, 255), (4, 4), (4, 4), 3);
        assert_eq!(r, Rect::new(3, 4, 3, 1));
        assert_eq!(muster(&s)[4], "...###..");
    }

    #[test]
    fn breite_unter_eins_zeichnet_nichts() {
        let mut s = leinwand(12, 12);
        for breite in [0, -1, -7] {
            let r = line(&mut s, (255, 255, 255), (2, 2), (9, 9), breite);
            assert_eq!(r, Rect::new(2, 2, 0, 0), "Rueckgabe sitzt am Startpunkt");
        }
        assert!(muster(&s).iter().all(|z| !z.contains('#')));
    }

    #[test]
    fn das_clip_rechteck_begrenzt_und_bestimmt_die_rueckgabe() {
        let mut s = leinwand(24, 24);
        s.set_clip(Some(Rect::new(5, 5, 6, 6)));
        let r = line(&mut s, (255, 255, 255), (0, 0), (23, 23), 1);
        assert_eq!(r, Rect::new(5, 5, 6, 6), "nur das Sichtbare zaehlt");
        assert_eq!(s.get_at((4, 4)).r, 0);
        assert_eq!(s.get_at((5, 5)).r, 255);

        // Gar nichts getroffen: leeres Rechteck am Startpunkt.
        let mut s = leinwand(24, 24);
        let r = line(&mut s, (255, 255, 255), (30, 30), (40, 40), 1);
        assert_eq!(r, Rect::new(30, 30, 0, 0));
        assert!(muster(&s).iter().all(|z| !z.contains('#')));
    }

    #[test]
    fn beschneiden_aendert_die_pixelwahl_nicht() {
        // Gemessen: dieselbe Linie gross gezeichnet und zugeschnitten
        // ergibt genau das kleine Bild.
        let mut klein = leinwand(24, 24);
        line(&mut klein, (255, 255, 255), (-50, -30), (20, 10), 1);
        let mut gross = leinwand(200, 200);
        line(&mut gross, (255, 255, 255), (10, 30), (80, 70), 1);
        for y in 0..24 {
            for x in 0..24 {
                assert_eq!(
                    klein.get_at((x, y)).r,
                    gross.get_at((x + 60, y + 60)).r,
                    "({x},{y})"
                );
            }
        }
    }

    #[test]
    fn die_farbe_wird_ohne_mischen_geschrieben() {
        // Gemessen: draw.line mischt nicht, auch nicht auf SRCALPHA.
        let mut s = Surface::with_flags((8, 8), SRCALPHA);
        s.fill((50, 50, 50, 255));
        line(&mut s, (255, 0, 0, 128), (0, 4), (7, 4), 3);
        assert_eq!(s.get_at((3, 4)).as_tuple(), (255, 0, 0, 128));
        assert_eq!(s.get_at((3, 0)).as_tuple(), (50, 50, 50, 255));
    }

    #[test]
    fn linienzug_offen_und_geschlossen() {
        let pts = [(2, 2), (10, 15), (17, 4)];
        let mut offen = leinwand(20, 20);
        let r = lines(&mut offen, (255, 255, 255), false, &pts, 1);
        assert_eq!(r, Rect::new(2, 2, 16, 14));
        assert_eq!(offen.get_at((9, 3)).r, 0, "der Rueckweg fehlt");

        let mut zu = leinwand(20, 20);
        let r = lines(&mut zu, (255, 255, 255), true, &pts, 1);
        assert_eq!(r, Rect::new(2, 2, 16, 14));
        assert_eq!(zu.get_at((9, 3)).r, 255, "der Rueckweg ist da");
    }

    #[test]
    fn linienzug_randfaelle() {
        // Zwei Punkte: der Rueckweg entfaellt, es bleibt eine Strecke.
        let mut a = leinwand(12, 12);
        let r = lines(&mut a, (255, 255, 255), true, &[(2, 2), (9, 9)], 1);
        assert_eq!(r, Rect::new(2, 2, 8, 8));
        let mut b = leinwand(12, 12);
        lines(&mut b, (255, 255, 255), false, &[(2, 2), (9, 9)], 1);
        assert_eq!(muster(&a), muster(&b), "geschlossen aendert hier nichts");

        // Ein Punkt oder keiner: nichts zu tun.
        let mut s = leinwand(12, 12);
        assert_eq!(
            lines(&mut s, (255, 255, 255), true, &[(3, 4)], 1),
            Rect::new(3, 4, 0, 0)
        );
        assert_eq!(
            lines(&mut s, (255, 255, 255), true, &[], 1),
            Rect::new(0, 0, 0, 0)
        );
        assert_eq!(
            lines(&mut s, (255, 255, 255), true, &[(3, 4), (8, 8)], 0),
            Rect::new(3, 4, 0, 0)
        );
        assert!(muster(&s).iter().all(|z| !z.contains('#')));
    }

    #[test]
    fn dicker_linienzug_behaelt_seine_kerben() {
        // Gemessen: pygame zeichnet jedes Teilstueck einzeln, am Knick
        // bleibt darum eine Ecke offen. Nicht "reparieren".
        let mut s = leinwand(20, 20);
        let r = lines(
            &mut s,
            (255, 255, 255),
            false,
            &[(2, 10), (10, 10), (10, 2)],
            3,
        );
        assert_eq!(r, Rect::new(2, 2, 10, 10));
        let m = muster(&s);
        assert_eq!(m[9], "..##########........");
        assert_eq!(
            m[11], "..#########.........",
            "unten rechts fehlt ein Pixel"
        );
        assert_eq!(s.get_at((11, 11)).r, 0, "die Kerbe am Knick gehoert dazu");
    }
}
