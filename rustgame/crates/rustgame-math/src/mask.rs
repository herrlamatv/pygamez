//! mask.rs
//! =======
//! Gegenstueck zu `pygame.mask` - Bitmasken fuer pixelgenaue
//! Ueberschneidungen.
//!
//! Eine Maske ist ein Feld aus Wahrheitswerten in Flaechengroesse. Zwei
//! Masken lassen sich gegeneinander verschieben und auf Ueberschneidung
//! pruefen, ohne Pixel zu vergleichen - das ist der uebliche Weg fuer
//! genaue Treffererkennung, wenn Rechtecke zu grob sind.
//!
//! Vorlage ist `src_c/mask.c` aus pygame-ce 2.5.7. Uebertragen, nicht
//! nachempfunden - und bei [`Mask::angle`] sieht man, warum: die Rechnung
//! dort benutzt durchgehend **Ganzzahldivision**. Wer sie in Fliesskomma
//! ausfuehrt, bekommt einen anderen Winkel.

use rustgame_core::rect::Rect;
use rustgame_core::surface::{Surface, SRCALPHA};

/// Ein Feld aus gesetzten und leeren Stellen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mask {
    w: u32,
    h: u32,
    /// Die Stellen als Bits, **gepackt zu zweiunddreissig und
    /// spaltenweise angeordnet**.
    ///
    /// Hier stand lange ein `bool` je Stelle, mit der Begruendung, der
    /// Unterschied sei Speicher und nicht Verhalten. **Das war einmal
    /// wahr.** Seit `memoryview(mask)` geht, ist die Anordnung im Speicher
    /// selbst Teil der Schnittstelle, und `mask_test.test_buffer_interface`
    /// rechnet damit:
    ///
    /// ```python
    /// view = memoryview(mask)
    /// intwidth = 8 * view.strides[1]        # 32
    /// col = x // intwidth
    /// (view[col, y] >> (x % intwidth)) & 1
    /// ```
    ///
    /// Gemessen an echtem pygame:
    ///
    /// | Maske | `shape` | `strides` | `format` |
    /// |---|---|---|---|
    /// | 1000 x 100 | `(32, 100)` | `(400, 4)` | `L` |
    /// | 5 x 3 | `(1, 3)` | `(12, 4)` | `L` |
    ///
    /// Die Schrittweiten sagen alles: einen Schritt in `y` sind vier Bytes,
    /// einen Schritt in der Wortspalte hundert mal vier. Der Index eines
    /// Worts ist also `(x / 32) * hoehe + y` - **spaltenweise**, wie SDLs
    /// `bitmask_t` es ablegt, und nicht zeilenweise, wie man es erwartet.
    ///
    /// Die Bits oberhalb der Breite in der letzten Wortspalte bleiben
    /// **leer**; `count()` zaehlt sonst Stellen, die es nicht gibt.
    bits: Vec<u32>,
}

impl Mask {
    /// Eine neue Maske. `fill` setzt alle Stellen auf einmal.
    pub fn new(size: (u32, u32), fill: bool) -> Mask {
        let spalten = (size.0 as usize).div_ceil(32);
        let mut m = Mask {
            w: size.0,
            h: size.1,
            bits: vec![0u32; spalten * size.1 as usize],
        };
        if fill {
            m.fill();
        }
        m
    }

    /// Wie viele Worte eine Zeile breit ist.
    #[inline]
    fn spalten(&self) -> usize {
        (self.w as usize).div_ceil(32)
    }

    /// Wort und Bit zu einer Stelle - **ohne** Bereichspruefung.
    #[inline]
    fn stelle(&self, x: usize, y: usize) -> (usize, u32) {
        ((x / 32) * self.h as usize + y, (x % 32) as u32)
    }

    /// Die Maske der **gueltigen** Bits in der letzten Wortspalte.
    ///
    /// Bei einer Breite von 1000 hat die zweiunddreissigste Spalte nur acht
    /// echte Bits; die oberen vierundzwanzig gehoeren zu keiner Stelle. Wer
    /// sie mitzaehlt oder mit umdreht, bekommt `count()`-Werte, die es
    /// nicht gibt.
    #[inline]
    fn randmaske(&self) -> u32 {
        match self.w % 32 {
            0 => u32::MAX,
            n => (1u32 << n) - 1,
        }
    }

    /// Die rohen Worte - fuer `memoryview(mask)`.
    pub fn worte(&self) -> &[u32] {
        &self.bits
    }

    /// Die laufende Nummer einer Stelle - **nicht** die eines Worts.
    ///
    /// Die Zusammenhangs-Suchen merken sich je Stelle, ob sie schon dort
    /// waren. Solange ein Byte je Stelle im Speicher lag, war das derselbe
    /// Index wie der der Bits; seit die Bits gepackt sind, sind es zwei
    /// verschiedene Dinge, und sie zu verwechseln heisst, dass je
    /// zweiunddreissig Stellen ein Merker teilen.
    #[inline]
    fn flach(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 {
            return None;
        }
        Some(y as usize * self.w as usize + x as usize)
    }

    /// Breite und Hoehe in Pixeln.
    pub fn get_size(&self) -> (u32, u32) {
        (self.w, self.h)
    }

    /// Ein Rechteck der Maskengroesse, bei `(0, 0)`.
    pub fn get_rect(&self) -> Rect {
        Rect::new(0, 0, self.w as i32, self.h as i32)
    }

    #[inline]
    fn index(&self, x: i32, y: i32) -> Option<(usize, u32)> {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 {
            return None;
        }
        Some(self.stelle(x as usize, y as usize))
    }

    /// Ist diese Stelle gesetzt? Ausserhalb gilt **nicht gesetzt**.
    pub fn get_at(&self, pos: (i32, i32)) -> bool {
        match self.index(pos.0, pos.1) {
            None => false,
            Some((i, b)) => self.bits[i] >> b & 1 != 0,
        }
    }

    /// Setzt eine Stelle. Ausserhalb passiert nichts.
    pub fn set_at(&mut self, pos: (i32, i32), wert: bool) {
        if let Some((i, b)) = self.index(pos.0, pos.1) {
            if wert {
                self.bits[i] |= 1 << b;
            } else {
                self.bits[i] &= !(1 << b);
            }
        }
    }

    /// Wie viele Stellen sind gesetzt?
    pub fn count(&self) -> usize {
        self.bits.iter().map(|w| w.count_ones() as usize).sum()
    }

    /// Setzt alles - **bis zur Breite**, nicht bis zum Wortende.
    pub fn fill(&mut self) {
        let (spalten, h) = (self.spalten(), self.h as usize);
        let rand = self.randmaske();
        for i in 0..spalten {
            let wert = if i + 1 == spalten { rand } else { u32::MAX };
            for y in 0..h {
                self.bits[i * h + y] = wert;
            }
        }
    }

    /// Loescht alles.
    pub fn clear(&mut self) {
        self.bits.fill(0);
    }

    /// Kehrt jede Stelle um.
    pub fn invert(&mut self) {
        let (spalten, h) = (self.spalten(), self.h as usize);
        let rand = self.randmaske();
        for i in 0..spalten {
            let halten = if i + 1 == spalten { rand } else { u32::MAX };
            for y in 0..h {
                let z = i * h + y;
                self.bits[z] = !self.bits[z] & halten;
            }
        }
    }

    // ----- Ueberschneidungen ---------------------------------------------

    /// Die **erste** gemeinsame Stelle, wenn `other` um `offset`
    /// verschoben daraufliegt - oder `None`.
    ///
    /// Gesucht wird **zeilenweise** von oben nach unten, innerhalb einer
    /// Zeile von links nach rechts.
    ///
    /// Hier stand einmal das Gegenteil, mit der Begruendung "pygame
    /// durchlaeuft die Maske genauso" - und das war nie nachgemessen.
    /// Ueber 2564 zufaellige Ueberschneidungen trifft die zeilenweise
    /// Suche **2564-mal**, die spaltenweise nur 1412-mal. Bei genau
    /// einer Beruehrung faellt es nicht auf; bei mehreren liefert die
    /// falsche Richtung einen anderen, ebenfalls gueltigen Punkt - und
    /// `collide_mask` gibt ihn weiter.
    pub fn overlap(&self, other: &Mask, offset: (i32, i32)) -> Option<(i32, i32)> {
        for y in 0..self.h as i32 {
            for x in 0..self.w as i32 {
                if self.get_at((x, y)) && other.get_at((x - offset.0, y - offset.1)) {
                    return Some((x, y));
                }
            }
        }
        None
    }

    /// Wie viele Stellen ueberschneiden sich?
    pub fn overlap_area(&self, other: &Mask, offset: (i32, i32)) -> usize {
        let mut n = 0;
        for y in 0..self.h as i32 {
            for x in 0..self.w as i32 {
                if self.get_at((x, y)) && other.get_at((x - offset.0, y - offset.1)) {
                    n += 1;
                }
            }
        }
        n
    }

    /// Eine neue Maske **in eigener Groesse**, die nur die gemeinsamen
    /// Stellen traegt.
    pub fn overlap_mask(&self, other: &Mask, offset: (i32, i32)) -> Mask {
        let mut m = Mask::new((self.w, self.h), false);
        for y in 0..self.h as i32 {
            for x in 0..self.w as i32 {
                if self.get_at((x, y)) && other.get_at((x - offset.0, y - offset.1)) {
                    m.set_at((x, y), true);
                }
            }
        }
        m
    }

    /// Verodert `other` hinein (`self |= other`).
    pub fn draw(&mut self, other: &Mask, offset: (i32, i32)) {
        for y in 0..self.h as i32 {
            for x in 0..self.w as i32 {
                if other.get_at((x - offset.0, y - offset.1)) {
                    self.set_at((x, y), true);
                }
            }
        }
    }

    /// Loescht, was `other` abdeckt (`self &= !other`).
    pub fn erase(&mut self, other: &Mask, offset: (i32, i32)) {
        for y in 0..self.h as i32 {
            for x in 0..self.w as i32 {
                if other.get_at((x - offset.0, y - offset.1)) {
                    self.set_at((x, y), false);
                }
            }
        }
    }

    // ----- Auswertung -----------------------------------------------------

    /// Der Schwerpunkt der gesetzten Stellen, **abgerundet**.
    ///
    /// Leere Maske: `(0, 0)`.
    pub fn centroid(&self) -> (i32, i32) {
        let (mut sx, mut sy, mut n) = (0i64, 0i64, 0i64);
        for y in 0..self.h as i32 {
            for x in 0..self.w as i32 {
                if self.get_at((x, y)) {
                    sx += x as i64;
                    sy += y as i64;
                    n += 1;
                }
            }
        }
        if n == 0 {
            return (0, 0);
        }
        ((sx / n) as i32, (sy / n) as i32)
    }

    /// Die Ausrichtung der gesetzten Stellen in Grad.
    ///
    /// Zeilengetreu aus `mask_angle` in `src_c/mask.c` - und **jede**
    /// Division darin ist eine Ganzzahldivision (`long int`), auch die in
    /// `m11 / m00`, `m20 / m00` und `m02 / m00`. In Fliesskomma gerechnet
    /// kommt ein anderer Winkel heraus.
    ///
    /// Zwei Stellen bei `(2,3)` und `(5,1)` ergeben in pygame
    /// `13.282525588538995`; mit Fliesskommadivision waere es `9.2...`.
    pub fn angle(&self) -> f64 {
        let (mut m10, mut m01, mut m00, mut m20, mut m02, mut m11) =
            (0i64, 0i64, 0i64, 0i64, 0i64, 0i64);

        // pygame laeuft hier spaltenweise; fuer die Summen ist das egal,
        // aber es bleibt so, damit der Vergleich mit dem Original leicht
        // faellt.
        for x in 0..self.w as i32 {
            for y in 0..self.h as i32 {
                if self.get_at((x, y)) {
                    let (xl, yl) = (x as i64, y as i64);
                    m10 += xl;
                    m20 += xl * xl;
                    m11 += xl * yl;
                    m02 += yl * yl;
                    m01 += yl;
                    m00 += 1;
                }
            }
        }
        if m00 == 0 {
            return 0.0;
        }
        let xc = m10 / m00;
        let yc = m01 / m00;
        -90.0 * ((2 * (m11 / m00 - xc * yc)) as f64)
            .atan2(((m20 / m00 - xc * xc) - (m02 / m00 - yc * yc)) as f64)
            / std::f64::consts::PI
    }

    /// Die umschliessenden Rechtecke aller zusammenhaengenden Bereiche.
    ///
    /// Zusammenhaengend heisst hier **achtfach** benachbart - schraeg
    /// zaehlt mit, wie in pygame.
    pub fn get_bounding_rects(&self) -> Vec<Rect> {
        let mut gesehen = vec![false; self.w as usize * self.h as usize];
        let mut rechtecke = Vec::new();

        for y in 0..self.h as i32 {
            for x in 0..self.w as i32 {
                let Some(i) = self.flach(x, y) else { continue };
                if !self.get_at((x, y)) || gesehen[i] {
                    continue;
                }
                let (mut x1, mut y1, mut x2, mut y2) = (x, y, x, y);
                let mut stapel = vec![(x, y)];
                gesehen[i] = true;
                while let Some((cx, cy)) = stapel.pop() {
                    x1 = x1.min(cx);
                    y1 = y1.min(cy);
                    x2 = x2.max(cx);
                    y2 = y2.max(cy);
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            let (nx, ny) = (cx + dx, cy + dy);
                            if let Some(j) = self.flach(nx, ny) {
                                if self.get_at((nx, ny)) && !gesehen[j] {
                                    gesehen[j] = true;
                                    stapel.push((nx, ny));
                                }
                            }
                        }
                    }
                }
                rechtecke.push(Rect::new(x1, y1, x2 - x1 + 1, y2 - y1 + 1));
            }
        }
        rechtecke
    }

    /// Alle zusammenhaengenden Bereiche als eigene Masken, groesster zuerst.
    pub fn connected_components(&self, min_groesse: usize) -> Vec<Mask> {
        let mut gesehen = vec![false; self.w as usize * self.h as usize];
        let mut teile: Vec<Mask> = Vec::new();

        for y in 0..self.h as i32 {
            for x in 0..self.w as i32 {
                let Some(i) = self.flach(x, y) else { continue };
                if !self.get_at((x, y)) || gesehen[i] {
                    continue;
                }
                let mut teil = Mask::new((self.w, self.h), false);
                let mut stapel = vec![(x, y)];
                gesehen[i] = true;
                let mut n = 0usize;
                while let Some((cx, cy)) = stapel.pop() {
                    teil.set_at((cx, cy), true);
                    n += 1;
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            let (nx, ny) = (cx + dx, cy + dy);
                            if let Some(j) = self.flach(nx, ny) {
                                if self.get_at((nx, ny)) && !gesehen[j] {
                                    gesehen[j] = true;
                                    stapel.push((nx, ny));
                                }
                            }
                        }
                    }
                }
                if n >= min_groesse {
                    teile.push(teil);
                }
            }
        }
        teile.sort_by_key(|m| std::cmp::Reverse(m.count()));
        teile
    }

    /// Der groesste zusammenhaengende Bereich, als eigene Maske.
    pub fn connected_component_bei(&self, pos: (i32, i32)) -> Mask {
        // Gemessen: eine leere Stelle gibt eine **leere** Maske gleicher
        // Groesse, keinen Fehler.
        if !self.get_at(pos) {
            return Mask::new(self.get_size(), false);
        }
        for teil in self.connected_components(0) {
            if teil.get_at(pos) {
                return teil;
            }
        }
        Mask::new(self.get_size(), false)
    }

    /// Der groesste zusammenhaengende Bereich.
    pub fn connected_component(&self) -> Mask {
        self.connected_components(0)
            .into_iter()
            .next()
            .unwrap_or_else(|| Mask::new((self.w, self.h), false))
    }

    /// Auf eine andere Groesse gebracht - naechster Nachbar, wie in pygame.
    pub fn scale(&self, size: (u32, u32)) -> Mask {
        let mut m = Mask::new(size, false);
        if size.0 == 0 || size.1 == 0 || self.w == 0 || self.h == 0 {
            return m;
        }
        for y in 0..size.1 {
            for x in 0..size.0 {
                let qx = (x as u64 * self.w as u64 / size.0 as u64) as i32;
                let qy = (y as u64 * self.h as u64 / size.1 as u64) as i32;
                if self.get_at((qx, qy)) {
                    m.set_at((x as i32, y as i32), true);
                }
            }
        }
        m
    }

    // ----- Kontur und Faltung ---------------------------------------------

    /// Die Punkte der aeusseren Kontur, im Uhrzeigersinn.
    ///
    /// Zeilengetreu aus `mask_outline` in `src_c/mask.c`. Vier Dinge daran
    /// errate man nicht:
    ///
    /// 1. Die **Nachbarschaftstabelle** hat 14 statt 8 Eintraege. Die
    ///    ersten acht sind die Richtungen im Uhrzeigersinn, beginnend
    ///    rechts; die uebrigen sechs wiederholen den Anfang. Damit darf die
    ///    Suche ueber das Ende hinauslaufen, ohne zu rechnen - und
    ///    ausgerechnet **sechs** braucht sie (siehe 2.).
    /// 2. Die **Startrichtung** jeder Runde ist `(n + 6) & 7`, wobei `n`
    ///    die Richtung ist, aus der man kam. Die Rueckrichtung ist `n + 4`;
    ///    gesucht wird also zwei Schritte im Uhrzeigersinn dahinter, und
    ///    die Rueckrichtung kommt erst als vorletzte dran - bei
    ///    Tabellenstelle `(n + 6) & 7` plus 6, hoechstens also 13. Genau
    ///    darum ist die Tabelle 14 lang. Diese Startrichtung ist es, was
    ///    die Kontur **aussen** herumfuehrt.
    /// 3. Der **Startpunkt** wird zeilenweise gesucht (y aussen, x innen),
    ///    wie bei [`Mask::overlap`]. Bei `(3,1)`, `(2,2)`, `(1,3)` liefert
    ///    pygame `[(3,1), (2,2), (1,3), (2,2), (3,1)]` - spaltenweise
    ///    gesucht finge dieselbe Kontur bei `(1,3)` an.
    /// 4. Umfahren wird nur **ein** Bereich: der mit dem ersten gesetzten
    ///    Punkt. Zwei getrennte Klumpen ergeben nur die Kontur des oberen.
    ///
    /// `every` schreibt nur jeden `every`-ten Schritt auf; der Startpunkt
    /// steht immer da. `Mask((4,3), true).outline(3)` ergibt in pygame
    /// `[(0,0), (3,0), (2,2), (0,1)]`. Ein `every` groesser als der Umfang
    /// laesst nur den Startpunkt uebrig, `0` und negative Werte ebenso -
    /// beides gemessen, beides ohne Fehler.
    pub fn outline(&self, every: i32) -> Vec<(i32, i32)> {
        /// Die Richtungen aus `mask.c`, im Uhrzeigersinn ab rechts, danach
        /// die ersten sechs noch einmal.
        const A: [i32; 14] = [1, 1, 0, -1, -1, -1, 0, 1, 1, 1, 0, -1, -1, -1];
        const B: [i32; 14] = [0, 1, 1, 1, 0, -1, -1, -1, 0, 1, 1, 1, 0, -1];

        let mut punkte: Vec<(i32, i32)> = Vec::new();
        if self.w == 0 || self.h == 0 {
            return punkte;
        }

        // Eine Stelle Rand ringsum: dann braucht die Nachbarsuche keine
        // Grenzpruefung, und der Rand ist nie gesetzt.
        let mut m = Mask::new((self.w + 2, self.h + 2), false);
        m.draw(self, (1, 1));

        // Der erste gesetzte Punkt, zeilenweise. pygame prueft danach
        // `x == m->w - 1 && y == m->h - 1`; das trifft nur zu, wenn beide
        // Schleifen durchgelaufen sind, also nichts gefunden wurde.
        let mut start = None;
        'suche: for y in 1..m.h as i32 - 1 {
            for x in 1..m.w as i32 - 1 {
                if m.get_at((x, y)) {
                    start = Some((x, y));
                    break 'suche;
                }
            }
        }
        let (firstx, firsty) = match start {
            None => return punkte,
            Some(p) => p,
        };
        punkte.push((firstx - 1, firsty - 1));

        // `e` zaehlt die Schritte herunter. In pygame ist das ein `int`;
        // hier `i64`, damit `every == i32::MIN` nicht ueberlaeuft. Auf das
        // Ergebnis wirkt sich das nicht aus: die Runde endet nach dem
        // Umfang, nicht nach `e`.
        let mut e = every as i64;

        // Der zweite Punkt: einmal ringsum den ersten, in Tabellenfolge.
        let mut n = 0usize;
        let mut zweiter = None;
        for k in 0..8 {
            if m.get_at((firstx + A[k], firsty + B[k])) {
                n = k;
                zweiter = Some((firstx + A[k], firsty + B[k]));
                e -= 1;
                if e == 0 {
                    e = every as i64;
                    punkte.push((firstx + A[k] - 1, firsty + B[k] - 1));
                }
                break;
            }
        }
        // pygame erkennt "kein Nachbar" an `secx == 0`. Das geht auf, weil
        // ein gefundener Nachbar nie in Spalte 0 liegen kann - die gehoert
        // zum Rand und ist nie gesetzt.
        let (secx, secy) = match zweiter {
            None => return punkte,
            Some(p) => p,
        };
        let (mut currx, mut curry) = (secx, secy);

        loop {
            // Um den Punkt herum - einen Nachbarn hat er sicher, naemlich
            // mindestens den, von dem wir kommen.
            n = (n + 6) & 7;
            while !m.get_at((currx + A[n], curry + B[n])) {
                n += 1;
            }
            let nextx = currx + A[n];
            let nexty = curry + B[n];

            // Zurueck am Anfang, und der naechste Schritt waere wieder der
            // zweite Punkt: fertig. Das gilt unabhaengig von `e` - sonst
            // liefe ein grosses `every` ewig im Kreis.
            let fertig = curry == firsty && currx == firstx && secx == nextx && secy == nexty;
            e -= 1;
            if e == 0 {
                e = every as i64;
                if !fertig {
                    punkte.push((nextx - 1, nexty - 1));
                }
            }
            if fertig {
                break;
            }
            currx = nextx;
            curry = nexty;
        }

        punkte
    }

    /// Die Groesse, die [`Mask::convolve`] fuer eine frische Ergebnismaske
    /// waehlt: `a + b - 1` je Richtung, mindestens `0`.
    ///
    /// Gemessen: eine `3x2`-Maske mit einer `0x0`-Maske gefaltet ergibt
    /// `(2, 1)` - nicht `(0, 0)`.
    pub fn convolve_size(&self, other: &Mask) -> (u32, u32) {
        let w = (self.w as i64 + other.w as i64 - 1).max(0) as u32;
        let h = (self.h as i64 + other.h as i64 - 1).max(0) as u32;
        (w, h)
    }

    /// Die Faltung zweier Masken, in eine **neue** Maske der Groesse aus
    /// [`Mask::convolve_size`].
    pub fn convolve(&self, other: &Mask, offset: (i32, i32)) -> Mask {
        let mut ziel = Mask::new(self.convolve_size(other), false);
        self.convolve_into(other, &mut ziel, offset);
        ziel
    }

    /// Die Faltung in eine vorgegebene Maske.
    ///
    /// Zeilengetreu aus `bitmask_convolve` in `src_c/bitmask.c`. Die
    /// Feinheit ist die Verschiebung: der Versatz wird vor der Schleife um
    /// `other.w - 1` bzw. `other.h - 1` **erhoeht**, und je gesetzter
    /// Stelle `(x, y)` von `other` wird `self` bei `versatz - (x, y)`
    /// hineingezeichnet. Ohne diese Vorverschiebung landet das Ergebnis
    /// um die halbe Maskengroesse daneben.
    ///
    /// Gezeichnet wird mit **oder**: was in `output` schon stand, bleibt
    /// stehen, und was ueber den Rand ragt, faellt weg. Beides gemessen.
    ///
    /// Ist eine der drei Masken leer (Breite oder Hoehe `0`), passiert
    /// nichts - so steht es in `bitmask_convolve`.
    ///
    /// `output` ist hier eine **andere** Maske als `self` und `other`.
    /// pygame laesst zu, dass es dieselbe ist; das Ergebnis haengt dann
    /// aber an seiner Wortbreite. Gemessen mit einer `80x1`-Maske, einer
    /// gesetzten Stelle und `output == self`: ein Punkt bei `29` ergibt
    /// acht gesetzte Stellen, einer bei `30` nur sechs. Das ist kein
    /// Verhalten, das man nachbaut - siehe [`Mask::convolve`] in der
    /// Python-Huelle, die in diesem Fall von der Quelle abschreibt.
    pub fn convolve_into(&self, other: &Mask, output: &mut Mask, offset: (i32, i32)) {
        if self.w == 0
            || self.h == 0
            || other.w == 0
            || other.h == 0
            || output.w == 0
            || output.h == 0
        {
            return;
        }
        let vx = offset.0 + other.w as i32 - 1;
        let vy = offset.1 + other.h as i32 - 1;
        for y in 0..other.h as i32 {
            for x in 0..other.w as i32 {
                if other.get_at((x, y)) {
                    output.draw(self, (vx - x, vy - y));
                }
            }
        }
    }

    // ----- Bruecken zu Flaechen -------------------------------------------

    /// Macht aus der Maske eine Flaeche: gesetzt wird `setcolor`, leer
    /// `unsetcolor`. `None` laesst die jeweiligen Stellen unberuehrt.
    ///
    /// Die kurze Form von [`to_surface_voll`](Self::to_surface_voll).
    pub fn to_surface(
        &self,
        setcolor: Option<(u8, u8, u8, u8)>,
        unsetcolor: Option<(u8, u8, u8, u8)>,
    ) -> Surface {
        let mut s = Surface::with_flags((self.w, self.h), SRCALPHA);
        self.to_surface_voll(&mut s, None, None, setcolor, unsetcolor, (0, 0), None);
        s
    }

    /// `Mask.to_surface` in voller Form - **schreibt in `ziel`**.
    ///
    /// pygames Signatur lautet
    ///
    /// ```text
    /// to_surface(surface=None, setsurface=None, unsetsurface=None,
    ///            setcolor=(255,255,255,255), unsetcolor=(0,0,0,255),
    ///            dest=(0,0), area=None)
    /// ```
    ///
    /// und jeder dieser sieben Werte tut etwas anderes. Alles gemessen
    /// an pygame-ce 2.5.7:
    ///
    /// | Angabe | Wirkung |
    /// |---|---|
    /// | ohne alles | schwarze Flaeche, gesetzte Stellen weiss, `SRCALPHA` |
    /// | `setcolor` / `unsetcolor` | die beiden Farben |
    /// | `unsetcolor=None` | leere Stellen bleiben **unberuehrt** |
    /// | `setsurface` | statt `setcolor` der Pixel **derselben Stelle** |
    /// | `unsetsurface` | dito fuer die leeren Stellen |
    /// | `dest` | verschiebt, wo die Maske im Ziel landet |
    /// | `area` | nimmt nur diesen Ausschnitt der **Maske** |
    ///
    /// Die Stelle, an der eine Nachbildung leicht danebengreift, ist
    /// `setsurface`: dessen Pixel wird an **der Stelle in der Maske**
    /// genommen, nicht an der Stelle im Ziel. Bei `dest != (0, 0)` faellt
    /// beides auseinander.
    ///
    /// Reicht `setsurface` nicht so weit, gilt wieder `setcolor` -
    /// gemessen an einer Vorlage, die kleiner ist als die Maske.
    /// Der Teil von `area`, der **auf der Maske liegt**.
    ///
    /// Das ist die Stelle, an der eine Nachbildung am ehesten
    /// danebengreift. Gemessen an pygame-ce 2.5.7 mit einer vollen
    /// 7x7-Maske:
    ///
    /// ```text
    /// area = Rect(-3, 0, 7, 7)   ->  Flaeche 4x7, ganz weiss
    /// area = Rect( 3, 0, 7, 7)   ->  Flaeche 4x7, ganz weiss
    /// area = Rect( 0, 0, -2, -2) ->  Flaeche 0x0
    /// area = Rect(10,10,  3, 3)  ->  Flaeche 0x0
    /// ```
    ///
    /// Der ueberstehende Teil wird also **nicht gezeichnet** - weder
    /// mit `unsetcolor` noch sonstwie. Genau das war hier zuerst falsch:
    /// die Schleife lief ueber das ganze `area` und behandelte alles
    /// ausserhalb der Maske als "leer", malte dort also `unsetcolor`
    /// statt die Flaeche in Ruhe zu lassen. Acht Tests in
    /// `mask_test.py` haben es gefunden, weil sie das Ziel vorher rot
    /// faerben und danach pruefen, dass es rot **geblieben** ist.
    ///
    /// Der Schnitt bestimmt auch die Groesse einer **neu** angelegten
    /// Flaeche - nicht das ungeschnittene `area`.
    pub fn ausschnitt(groesse: (u32, u32), area: Option<(i32, i32, i32, i32)>) -> (i32, i32, i32, i32) {
        let (w, h) = (groesse.0 as i32, groesse.1 as i32);
        let (mut ax, mut ay, mut aw, mut ah) = area.unwrap_or((0, 0, w, h));

        // **Erst normalisieren, dann schneiden** - zeilengetreu nach
        // `mask.c:2224`, wo `pgRect_Normalize` vor dem Klemmen steht.
        //
        // Hier stand vorher `aw.max(0)`, also "eine negative Breite ist
        // keine Breite". Das war eine eigene Erfindung, und sie war
        // falsch: pygame **dreht** ein negatives Rechteck um, es
        // bezeichnet damit denselben Bereich von der anderen Seite.
        // `Rect((0, 1), (1, -1))` ist nach dem Drehen `(0, 0, 1, 1)` und
        // zeichnet Punkt (0, 0) - `test_to_surface__negative_sized_area_rect`
        // prueft genau das. Geklemmt kam dort eine leere Flaeche heraus.
        if aw < 0 {
            ax += aw;
            aw = -aw;
        }
        if ah < 0 {
            ay += ah;
            ah = -ah;
        }

        // Ein Ueberstand links/oben verkuerzt die Seite, statt sie zu
        // verschieben (`mask.c:2226`).
        if ax < 0 {
            aw += ax;
            ax = 0;
        }
        if ay < 0 {
            ah += ay;
            ay = 0;
        }

        (ax, ay, aw.min(w - ax).max(0), ah.min(h - ay).max(0))
    }

    #[allow(clippy::too_many_arguments)]
    /// `Mask.to_surface` mit allen sieben Argumenten.
    /// 
    /// Die kurze Form steht daneben; diese hier ist die, auf die
    /// `mask_test.py` seine zwanzig Faelle wirft.
    pub fn to_surface_voll(
        &self,
        ziel: &mut Surface,
        setsurface: Option<&Surface>,
        unsetsurface: Option<&Surface>,
        setcolor: Option<(u8, u8, u8, u8)>,
        unsetcolor: Option<(u8, u8, u8, u8)>,
        dest: (i32, i32),
        area: Option<(i32, i32, i32, i32)>,
    ) {
        let (ax, ay, aw, ah) = Self::ausschnitt(self.get_size(), area);
        for zy in 0..ah {
            for zx in 0..aw {
                let (mx, my) = (ax + zx, ay + zy);
                let gesetzt = self.get_at((mx, my));
                let vorlage = if gesetzt { setsurface } else { unsetsurface };
                let farbe = if gesetzt { setcolor } else { unsetcolor };
                let stelle = (dest.0 + zx, dest.1 + zy);
                if let Some(v) = vorlage {
                    let (vw, vh) = v.get_size();
                    if mx < vw as i32 && my < vh as i32 {
                        // **Die Stelle in der Maske**, nicht die im Ziel.
                        ziel.set_at(stelle, v.get_at((mx, my)));
                        continue;
                    }
                }
                if let Some(c) = farbe {
                    ziel.set_at(stelle, c);
                }
            }
        }
    }
}

/// Eine Maske aus allem, was undurchsichtig genug ist.
///
/// Gemessen: die Schwelle ist ein **echtes Groesser** - ein Pixel mit
/// Alpha genau `threshold` zaehlt **nicht**. Voreinstellung in pygame ist
/// 127.
/// Die Schwelle ist **vorzeichenbehaftet und ungeklemmt** - `i32`, nicht
/// `u8`.
///
/// `mask.c:898` schreibt `int threshold = 127;` und vergleicht spaeter
/// schlicht `alpha > threshold`. Ein `u8` sieht daneben harmloser aus, als
/// es ist: `from_surface(s, -1)` soll jeden Pixel setzen, weil auch
/// `alpha == 0` noch groesser als `-1` ist. Auf `0` geklemmt wird daraus
/// `0 > 0`, also **keiner** - und `test_from_surface` faellt mit
/// `0 != 99` bei `alpha=0, threshold=-1`.
///
/// Der Klemmvorgang stand hier mit dem Vermerk "gemessen klemmt pygame
/// sie stillschweigend". Das war nie gemessen, sondern erschlossen, und
/// die andere Richtung hatte es verdeckt: `threshold=256` verhaelt sich
/// geklemmt zufaellig richtig, weil `255 > 255` und `255 > 256` beide
/// falsch sind. Nur die Gegenprobe nach unten trennt die beiden Faelle.
pub fn from_surface(s: &Surface, threshold: i32) -> Mask {
    let (w, h) = s.get_size();
    let mut m = Mask::new((w, h), false);
    // Ohne Alphakanal hat jeder Pixel Alpha 255 - dann entscheidet die
    // Schluesselfarbe, sonst waere die Maske immer voll.
    let key = s.get_colorkey();
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let c = s.get_at((x, y));
            let gesetzt = match key {
                Some(k) => (c.r, c.g, c.b) != (k.r, k.g, k.b),
                None => c.a as i32 > threshold,
            };
            if gesetzt {
                m.set_at((x, y), true);
            }
        }
    }
    m
}

/// Eine Maske aus allen Pixeln, die einer Farbe nahe genug kommen.
pub fn from_threshold(s: &Surface, farbe: (u8, u8, u8), schwelle: (u8, u8, u8)) -> Mask {
    from_threshold_gegen(s, farbe, schwelle, None)
}

/// Wie [`from_threshold`], aber wahlweise gegen eine **zweite Flaeche**.
///
/// Mit `andere` tritt an die Stelle der einen Farbe der Pixel derselben
/// Stelle in jener Flaeche. Die Regel bleibt dieselbe - gesetzt wird, was
/// **nahe genug** ist, nicht was verschieden genug ist. Das ist die
/// Stelle, an der man sich leicht vertut: der Name `threshold` klingt
/// nach "ab hier zaehlt es", gemeint ist "bis hierhin gilt es als
/// gleich".
///
/// Gemessen an pygame-ce 2.5.7, drei Pixel mit Unterschied `(20, 0, 0)`:
///
/// ```text
/// schwelle (10, 10, 10)  ->  0 0 0    (20 ist nicht unter 10)
/// schwelle (30,  5,  5)  ->  1 1 1    (20 unter 30, 0 unter 5)
/// ```
///
/// Also **jeder** Kanal fuer sich, und **echtes Kleiner**.
pub fn from_threshold_gegen(
    s: &Surface,
    farbe: (u8, u8, u8),
    schwelle: (u8, u8, u8),
    andere: Option<&Surface>,
) -> Mask {
    let (w, h) = s.get_size();
    let mut m = Mask::new((w, h), false);
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let c = s.get_at((x, y));
            let ziel = match andere {
                None => farbe,
                Some(o) => {
                    let v = o.get_at((x, y));
                    (v.r, v.g, v.b)
                }
            };
            let nah = c.r.abs_diff(ziel.0) < schwelle.0
                && c.g.abs_diff(ziel.1) < schwelle.1
                && c.b.abs_diff(ziel.2) < schwelle.2;
            if nah {
                m.set_at((x, y), true);
            }
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sollwerte aus echtem pygame-ce 2.5.7.
    #[test]
    fn die_grundrechnungen_treffen_pygame() {
        let leer = Mask::new((8, 6), false);
        let voll = Mask::new((8, 6), true);
        assert_eq!(leer.get_size(), (8, 6));
        assert_eq!(leer.count(), 0);
        assert_eq!(voll.count(), 48);
        assert_eq!(leer.get_rect(), Rect::new(0, 0, 8, 6));

        let mut m = Mask::new((8, 6), false);
        m.set_at((2, 3), true);
        m.set_at((5, 1), true);
        assert_eq!(m.count(), 2);
        assert!(m.get_at((2, 3)));
        assert!(!m.get_at((0, 0)));
        assert!(!m.get_at((-1, 0)), "ausserhalb ist nie gesetzt");

        assert_eq!(m.centroid(), (3, 2));

        // Die umschliessenden Rechtecke - zeilenweise gefunden, also der
        // obere Punkt zuerst. Genau so meldet es pygame.
        let r = m.get_bounding_rects();
        assert_eq!(r, vec![Rect::new(5, 1, 1, 1), Rect::new(2, 3, 1, 1)]);
    }

    /// Der Winkel - und warum Ganzzahldivision kein Detail ist.
    #[test]
    fn der_winkel_rechnet_in_ganzen_zahlen() {
        let mut m = Mask::new((8, 6), false);
        m.set_at((2, 3), true);
        m.set_at((5, 1), true);
        // pygame liefert genau diesen Wert.
        assert!(
            (m.angle() - 13.282525588538995).abs() < 1e-12,
            "bekommen {}",
            m.angle()
        );
        // Leer ergibt 0, nicht NaN.
        assert_eq!(Mask::new((4, 4), false).angle(), 0.0);
    }

    #[test]
    fn ueberschneidungen_treffen_pygame() {
        let a = Mask::new((4, 4), true);
        let b = Mask::new((4, 4), true);
        assert_eq!(a.overlap(&b, (1, 1)), Some((1, 1)));
        assert_eq!(a.overlap_area(&b, (1, 1)), 9);
        assert_eq!(a.overlap_area(&b, (9, 9)), 0, "weit auseinander");
        assert_eq!(a.overlap(&b, (9, 9)), None);

        let c = a.overlap_mask(&b, (1, 1));
        assert_eq!(c.get_size(), (4, 4), "so gross wie die erste Maske");
        assert_eq!(c.count(), 9);
    }

    #[test]
    fn fuellen_loeschen_umkehren() {
        let mut m = Mask::new((3, 3), false);
        m.invert();
        assert_eq!(m.count(), 9);
        m.clear();
        assert_eq!(m.count(), 0);
        m.fill();
        assert_eq!(m.count(), 9);
    }

    #[test]
    fn zeichnen_und_loeschen_verknuepfen() {
        let mut a = Mask::new((4, 4), false);
        let b = Mask::new((2, 2), true);
        a.draw(&b, (1, 1));
        assert_eq!(a.count(), 4);
        assert!(a.get_at((1, 1)) && a.get_at((2, 2)));
        a.erase(&b, (2, 2));
        assert_eq!(a.count(), 3, "eine Stelle ueberlappt");
    }

    #[test]
    fn zusammenhaengende_bereiche_zaehlen_schraeg_mit() {
        let mut m = Mask::new((5, 5), false);
        // Zwei Stellen, nur schraeg benachbart - das zaehlt als **ein**
        // Bereich, weil pygame achtfach nachbart.
        m.set_at((1, 1), true);
        m.set_at((2, 2), true);
        // Und eine dritte weit weg.
        m.set_at((4, 4), true);
        assert_eq!(m.get_bounding_rects().len(), 2);

        let teile = m.connected_components(0);
        assert_eq!(teile.len(), 2);
        assert_eq!(teile[0].count(), 2, "der groesste zuerst");
        assert_eq!(m.connected_component().count(), 2);

        // Zu kleine Bereiche lassen sich aussortieren.
        assert_eq!(m.connected_components(2).len(), 1);
    }

    #[test]
    fn aus_einer_flaeche_und_zurueck() {
        let mut s = Surface::with_flags((4, 4), SRCALPHA);
        s.fill((0, 0, 0, 0));
        s.set_at((1, 1), (255, 0, 0, 255));
        let m = from_surface(&s, 127);
        assert_eq!(m.count(), 1);
        assert!(m.get_at((1, 1)));

        // Eine hohe Schwelle laesst den vollen Pixel noch durch.
        assert_eq!(from_surface(&s, 200).count(), 1);
        // Bei genau 255 nicht mehr - die Schwelle ist ein echtes Groesser.
        assert_eq!(from_surface(&s, 255).count(), 0);

        let zurueck = m.to_surface(Some((0, 255, 0, 255)), None);
        assert_eq!(zurueck.get_at((1, 1)).g, 255);
        assert_eq!(zurueck.get_at((0, 0)).a, 0, "leere Stellen bleiben durchsichtig");
    }

    /// Baut eine Maske aus einer Punktliste - spart Tipparbeit.
    fn maske(size: (u32, u32), punkte: &[(i32, i32)]) -> Mask {
        let mut m = Mask::new(size, false);
        for p in punkte {
            m.set_at(*p, true);
        }
        m
    }

    /// Alle Sollwerte aus echtem pygame-ce 2.5.7.
    #[test]
    fn die_kontur_trifft_pygame() {
        // Mask((6,5)) mit (2,2), (3,2), (2,3).
        let l = maske((6, 5), &[(2, 2), (3, 2), (2, 3)]);
        assert_eq!(l.outline(1), vec![(2, 2), (3, 2), (2, 3), (2, 2)]);

        // Mask((4,3), True) - im Uhrzeigersinn, Startpunkt zum Schluss
        // noch einmal.
        let voll = Mask::new((4, 3), true);
        assert_eq!(
            voll.outline(1),
            vec![
                (0, 0),
                (1, 0),
                (2, 0),
                (3, 0),
                (3, 1),
                (3, 2),
                (2, 2),
                (1, 2),
                (0, 2),
                (0, 1),
                (0, 0)
            ]
        );

        // Ein Ring: die Kontur laeuft aussen herum, das Loch bleibt
        // unbeachtet.
        let ring = maske(
            (5, 5),
            &[
                (1, 1),
                (2, 1),
                (3, 1),
                (1, 2),
                (3, 2),
                (1, 3),
                (2, 3),
                (3, 3),
            ],
        );
        assert_eq!(
            ring.outline(1),
            vec![
                (1, 1),
                (2, 1),
                (3, 1),
                (3, 2),
                (3, 3),
                (2, 3),
                (1, 3),
                (1, 2),
                (1, 1)
            ]
        );

        // Ein Plus - jeder Arm wird einzeln angelaufen.
        let plus = maske((5, 5), &[(2, 1), (1, 2), (2, 2), (3, 2), (2, 3)]);
        assert_eq!(plus.outline(1), vec![(2, 1), (3, 2), (2, 3), (1, 2), (2, 1)]);

        // Eine Linie: hin und zurueck, die Enden nur einmal.
        let zeile = maske((5, 3), &[(0, 1), (1, 1), (2, 1), (3, 1), (4, 1)]);
        assert_eq!(
            zeile.outline(1),
            vec![
                (0, 1),
                (1, 1),
                (2, 1),
                (3, 1),
                (4, 1),
                (3, 1),
                (2, 1),
                (1, 1),
                (0, 1)
            ]
        );
    }

    /// Der Startpunkt wird **zeilenweise** gesucht - wie bei `overlap`.
    ///
    /// Bei dieser Diagonale liegt der zeilenweise erste Punkt bei `(3,1)`,
    /// der spaltenweise erste bei `(1,3)`. pygame meldet `(3,1)` zuerst.
    #[test]
    fn die_kontur_faengt_zeilenweise_an() {
        let d = maske((6, 6), &[(3, 1), (2, 2), (1, 3)]);
        assert_eq!(d.outline(1), vec![(3, 1), (2, 2), (1, 3), (2, 2), (3, 1)]);
    }

    /// Randfaelle der Kontur, alle gemessen.
    #[test]
    fn die_kontur_haelt_die_randfaelle_aus() {
        // Leere Maske: leere Liste.
        assert_eq!(Mask::new((6, 5), false).outline(1), vec![]);
        // Maske ohne Flaeche: ebenso.
        assert_eq!(Mask::new((0, 0), false).outline(1), vec![]);
        assert_eq!(Mask::new((0, 5), false).outline(1), vec![]);
        assert_eq!(Mask::new((5, 0), false).outline(1), vec![]);

        // Ein einzelner Punkt: nur er selbst, nicht zweimal.
        assert_eq!(maske((5, 5), &[(2, 2)]).outline(1), vec![(2, 2)]);
        assert_eq!(maske((1, 1), &[(0, 0)]).outline(1), vec![(0, 0)]);
        // Auch ganz unten rechts - dort prueft pygame auf "nichts
        // gefunden", und der Punkt darf davon nicht getroffen werden.
        assert_eq!(maske((4, 3), &[(3, 2)]).outline(1), vec![(3, 2)]);
        assert_eq!(maske((4, 3), &[(0, 0)]).outline(1), vec![(0, 0)]);

        // Zwei getrennte Klumpen: nur der obere wird umfahren.
        let zwei = maske(
            (8, 6),
            &[(1, 1), (2, 1), (1, 2), (2, 2), (5, 3), (5, 4), (6, 4)],
        );
        assert_eq!(zwei.outline(1), vec![(1, 1), (2, 1), (2, 2), (1, 2), (1, 1)]);
    }

    /// `every` laesst Schritte aus - der Startpunkt bleibt immer stehen.
    #[test]
    fn die_kontur_ueberspringt_punkte() {
        let voll = Mask::new((4, 3), true);
        assert_eq!(
            voll.outline(2),
            vec![(0, 0), (2, 0), (3, 1), (2, 2), (0, 2), (0, 0)]
        );
        assert_eq!(voll.outline(3), vec![(0, 0), (3, 0), (2, 2), (0, 1)]);
        assert_eq!(voll.outline(4), vec![(0, 0), (3, 1), (0, 2)]);
        assert_eq!(voll.outline(5), vec![(0, 0), (3, 2), (0, 0)]);
        // Der Umfang ist 10 Schritte - ab 11 bleibt nur der Anfang.
        assert_eq!(voll.outline(11), vec![(0, 0)]);
        assert_eq!(voll.outline(100), vec![(0, 0)]);
        // 0 und negativ sind kein Fehler; es wird nur nie aufgeschrieben.
        assert_eq!(voll.outline(0), vec![(0, 0)]);
        assert_eq!(voll.outline(-1), vec![(0, 0)]);
        // Auch die Extreme sind in pygame gemessen - und in `i32` gerechnet
        // liefe `e` hier ueber.
        assert_eq!(voll.outline(i32::MIN), vec![(0, 0)]);
        assert_eq!(voll.outline(i32::MAX), vec![(0, 0)]);

        let l = maske((6, 5), &[(2, 2), (3, 2), (2, 3)]);
        assert_eq!(l.outline(2), vec![(2, 2), (2, 3)]);
        assert_eq!(l.outline(3), vec![(2, 2), (2, 2)]);
        assert_eq!(maske((5, 5), &[(2, 2)]).outline(5), vec![(2, 2)]);
    }

    /// Die Faltung - Sollwerte aus echtem pygame.
    #[test]
    fn die_faltung_trifft_pygame() {
        /// Die Maske als Zeilen aus `#` und `.` - so laesst sie sich lesen.
        fn zeilen(m: &Mask) -> Vec<String> {
            let (w, h) = m.get_size();
            (0..h as i32)
                .map(|y| {
                    (0..w as i32)
                        .map(|x| if m.get_at((x, y)) { '#' } else { '.' })
                        .collect()
                })
                .collect()
        }

        // a = ["##.", ".#."], b = ["#.", ".#"]
        let a = maske((3, 2), &[(0, 0), (1, 0), (1, 1)]);
        let b = maske((2, 2), &[(0, 0), (1, 1)]);

        let c = a.convolve(&b, (0, 0));
        assert_eq!(c.get_size(), (4, 3), "a + b - 1 je Richtung");
        assert_eq!(zeilen(&c), ["##..", ".##.", "..#."]);

        // Der Versatz verschiebt, die Groesse bleibt.
        let v = a.convolve(&b, (1, 1));
        assert_eq!(v.get_size(), (4, 3));
        assert_eq!(zeilen(&v), ["....", ".##.", "..##"]);

        let n = a.convolve(&b, (-1, -1));
        assert_eq!(zeilen(&n), ["##..", ".#..", "...."]);

        // Weit daneben: alles faellt heraus.
        assert_eq!(a.convolve(&b, (-5, -5)).count(), 0);

        // Eine leere zweite Maske ergibt nichts - aber in voller Groesse.
        let leer = a.convolve(&Mask::new((2, 2), false), (0, 0));
        assert_eq!(leer.get_size(), (4, 3));
        assert_eq!(leer.count(), 0);
    }

    /// Faltung in eine vorgegebene Maske - gemessen.
    #[test]
    fn die_faltung_schreibt_in_eine_vorgabe() {
        let a = maske((3, 2), &[(0, 0), (1, 0), (1, 1)]);
        let b = maske((2, 2), &[(0, 0), (1, 1)]);

        // Was schon drinsteht, bleibt stehen - gezeichnet wird mit oder.
        let mut ziel = maske((4, 4), &[(3, 3)]);
        a.convolve_into(&b, &mut ziel, (0, 0));
        assert_eq!(ziel.get_size(), (4, 4), "die Vorgabe behaelt ihre Groesse");
        assert!(ziel.get_at((3, 3)), "die alte Stelle steht noch");
        assert!(ziel.get_at((0, 0)) && ziel.get_at((1, 1)) && ziel.get_at((2, 2)));

        // Eine zu kleine Vorgabe schneidet ab: pygame liefert ["##", ".#"].
        let mut klein = Mask::new((2, 2), false);
        a.convolve_into(&b, &mut klein, (0, 0));
        assert!(klein.get_at((0, 0)) && klein.get_at((1, 0)) && klein.get_at((1, 1)));
        assert!(!klein.get_at((0, 1)));

        // Masken ohne Flaeche: die Groesse ist `a + b - 1`, mindestens 0.
        let ohne = Mask::new((0, 0), false);
        assert_eq!(a.convolve_size(&ohne), (2, 1));
        assert_eq!(ohne.convolve_size(&a), (2, 1));
        assert_eq!(ohne.convolve_size(&ohne), (0, 0));
        assert_eq!(a.convolve(&ohne, (0, 0)).count(), 0, "nichts zu falten");
    }

    #[test]
    fn skalieren_nimmt_den_naechsten_nachbarn() {
        let mut m = Mask::new((2, 2), false);
        m.set_at((0, 0), true);
        let gross = m.scale((4, 4));
        assert_eq!(gross.get_size(), (4, 4));
        assert_eq!(gross.count(), 4, "aus einer Stelle werden vier");
        assert!(gross.get_at((0, 0)) && gross.get_at((1, 1)));
        assert!(!gross.get_at((2, 2)));
    }
}
