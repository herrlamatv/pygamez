//! rect.rs
//! =======
//! Rechtecke - Gegenstueck zu `pygame.Rect`.
//!
//! pygame schreibt `rect.center = (x, y)` und **verschiebt** damit das
//! Rechteck, ohne seine Groesse zu aendern. Rust kennt keine Setter-
//! Eigenschaften, deshalb gibt es zu jedem der 20 virtuellen Attribute ein
//! Dreigespann:
//!
//! ```
//! # use rustgame_core::rect::Rect;
//! let r = Rect::new(0, 0, 10, 10);
//! assert_eq!(r.center(), (5, 5));           // lesen
//! let mut m = r;
//! m.set_center((100, 100));                  // setzen (verschiebt)
//! assert_eq!(m, Rect::new(95, 95, 10, 10));
//! let b = r.with_center((100, 100));         // Bauform fuer Ketten
//! assert_eq!(b, m);
//! ```
//!
//! Die Bauform `with_*` deckt den haeufigsten Fall des ganzen Projekts ab:
//! `surface.get_rect(center=…)` kommt 868-mal vor und wird hier zu
//! `surf.get_rect().with_center(c)`.
//!
//! **Bewusste Abweichung:** `Rect` ist hier `Copy`. pygames `Rect` ist ein
//! veraenderliches Objekt mit Referenzsemantik (`a = b` teilt dasselbe
//! Rechteck). Ohne `Copy` waere in Rust jeder Aufruf voller `.clone()`.
//! In der spaeteren Python-Huelle stimmt die Semantik wieder, weil `Rect`
//! dort ein echtes Objekt wird.

/// Ein Rechteck aus ganzzahligen Pixelkoordinaten.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Rect {
    /// Die linke Kante.
    pub x: i32,
    /// Die obere Kante.
    pub y: i32,
    /// Die Breite. Darf negativ sein - siehe [`Rect::normalized`].
    pub w: i32,
    /// Die Hoehe. Dito.
    pub h: i32,
}

/// Erzeugt zu einem virtuellen Attribut Leser, Setzer und Bauform.
///
/// Spart rund 400 Zeilen gleichfoermigen Rumpf und sorgt dafuer, dass alle
/// 20 Attribute sich garantiert gleich verhalten.
/// Eine `f32` auf `i32` abschneiden - **wie x86 es tut**, nicht wie Rust.
///
/// Das Original schreibt `(int)(self->r.w / maxratio)`. Fuer NaN oder
/// einen Wert ausserhalb von `i32` ist diese Umwandlung in C unbestimmt;
/// der uebersetzte Befehl (`cvttss2si`) liefert dann den
/// "integer indefinite value", also `i32::MIN`. Rusts `as` klemmt
/// stattdessen am Rand fest und gaebe `i32::MAX`.
///
/// Gemessen an pygame auf dieser Maschine:
///
/// ```text
///   Rect(0,0,0,0).fit(Rect(0,0,10,10))
///     -> Rect(-1073741819, -1073741819, -2147483648, -2147483648)
/// ```
///
/// Also `i32::MIN` und nicht `i32::MAX`. Gebraucht wird das nur von
/// [`Rect::fit`], und dort nur, wenn eine Groesse Null ist - genau die
/// Faelle, in denen die vorherige Fassung abgewichen ist.
fn x86_abschneiden(v: f32) -> i32 {
    // Die Schwelle ist `2^31` und nicht `i32::MAX`: `i32::MAX as f32`
    // rundet auf `2147483648.0` auf, und dieser Wert liegt schon
    // ausserhalb.
    if v.is_nan() || v >= 2_147_483_648.0 || v < -2_147_483_648.0 {
        i32::MIN
    } else {
        v as i32
    }
}

macro_rules! punkt_attr {
    ($l:ty, $z:ty, $lesen:ident, $setzen:ident, $bauen:ident, |$s:ident| $get:expr, |$m:ident, $v:ident| $set:expr) => {
        #[doc = concat!("Liest `", stringify!($lesen), "` wie in pygame.")]
        pub fn $lesen(&self) -> ($l, $l) {
            let $s = self;
            $get
        }

        #[doc = concat!("Setzt `", stringify!($lesen), "` - verschiebt das Rechteck, die Groesse bleibt.")]
        pub fn $setzen(&mut self, wert: ($z, $z)) -> &mut Self {
            let $m = self;
            let $v = wert;
            $set;
            $m
        }

        #[doc = concat!("Wie `", stringify!($setzen), "`, aber als Bauform fuer Ketten.")]
        pub fn $bauen(mut self, wert: ($z, $z)) -> Self {
            self.$setzen(wert);
            self
        }
    };
}
pub(crate) use punkt_attr;

/// Dasselbe fuer die Attribute, die nur eine Zahl tragen.
macro_rules! zahl_attr {
    ($l:ty, $z:ty, $lesen:ident, $setzen:ident, $bauen:ident, |$s:ident| $get:expr, |$m:ident, $v:ident| $set:expr) => {
        #[doc = concat!("Liest `", stringify!($lesen), "` wie in pygame.")]
        pub fn $lesen(&self) -> $l {
            let $s = self;
            $get
        }

        #[doc = concat!("Setzt `", stringify!($lesen), "` - verschiebt das Rechteck.")]
        pub fn $setzen(&mut self, wert: $z) -> &mut Self {
            let $m = self;
            let $v = wert;
            $set;
            $m
        }

        #[doc = concat!("Wie `", stringify!($setzen), "`, aber als Bauform.")]
        pub fn $bauen(mut self, wert: $z) -> Self {
            self.$setzen(wert);
            self
        }
    };
}
pub(crate) use zahl_attr;

impl Rect {
    /// Neues Rechteck.
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect { x, y, w, h }
    }

    /// Aus Position und Groesse (`pygame.Rect((x,y), (w,h))`).
    pub const fn from_parts(pos: (i32, i32), size: (i32, i32)) -> Rect {
        Rect::new(pos.0, pos.1, size.0, size.1)
    }

    // ----- Die einfachen Kanten ----------------------------------------

    zahl_attr!(i32, i32, left, set_left, with_left, |s| s.x, |m, v| m.x = v);
    zahl_attr!(i32, i32, top, set_top, with_top, |s| s.y, |m, v| m.y = v);
    zahl_attr!(i32, i32, right, set_right, with_right, |s| s.x + s.w, |m, v| m.x = v - m.w);
    zahl_attr!(i32, i32, bottom, set_bottom, with_bottom, |s| s.y + s.h, |m, v| m.y = v - m.h);
    zahl_attr!(i32, i32, centerx, set_centerx, with_centerx, |s| s.x + s.w / 2, |m, v| m.x = v - m.w / 2);
    zahl_attr!(i32, i32, centery, set_centery, with_centery, |s| s.y + s.h / 2, |m, v| m.y = v - m.h / 2);
    zahl_attr!(i32, i32, width, set_width, with_width, |s| s.w, |m, v| m.w = v);
    zahl_attr!(i32, i32, height, set_height, with_height, |s| s.h, |m, v| m.h = v);

    // ----- Die neun Anker ----------------------------------------------

    punkt_attr!(i32, i32, topleft, set_topleft, with_topleft,
        |s| (s.x, s.y), |m, v| { m.x = v.0; m.y = v.1 });
    punkt_attr!(i32, i32, topright, set_topright, with_topright,
        |s| (s.x + s.w, s.y), |m, v| { m.x = v.0 - m.w; m.y = v.1 });
    punkt_attr!(i32, i32, bottomleft, set_bottomleft, with_bottomleft,
        |s| (s.x, s.y + s.h), |m, v| { m.x = v.0; m.y = v.1 - m.h });
    punkt_attr!(i32, i32, bottomright, set_bottomright, with_bottomright,
        |s| (s.x + s.w, s.y + s.h), |m, v| { m.x = v.0 - m.w; m.y = v.1 - m.h });
    punkt_attr!(i32, i32, center, set_center, with_center,
        |s| (s.x + s.w / 2, s.y + s.h / 2),
        |m, v| { m.x = v.0 - m.w / 2; m.y = v.1 - m.h / 2 });
    punkt_attr!(i32, i32, midtop, set_midtop, with_midtop,
        |s| (s.x + s.w / 2, s.y), |m, v| { m.x = v.0 - m.w / 2; m.y = v.1 });
    punkt_attr!(i32, i32, midbottom, set_midbottom, with_midbottom,
        |s| (s.x + s.w / 2, s.y + s.h),
        |m, v| { m.x = v.0 - m.w / 2; m.y = v.1 - m.h });
    punkt_attr!(i32, i32, midleft, set_midleft, with_midleft,
        |s| (s.x, s.y + s.h / 2), |m, v| { m.x = v.0; m.y = v.1 - m.h / 2 });
    punkt_attr!(i32, i32, midright, set_midright, with_midright,
        |s| (s.x + s.w, s.y + s.h / 2),
        |m, v| { m.x = v.0 - m.w; m.y = v.1 - m.h / 2 });
    punkt_attr!(i32, i32, size, set_size, with_size,
        |s| (s.w, s.h), |m, v| { m.w = v.0; m.h = v.1 });

    // ----- Verschieben und Groesse aendern ------------------------------

    /// Verschobene Kopie. Heisst in pygame `move` - das ist in Rust ein
    /// Schluesselwort, daher der Unterstrich.
    pub fn move_(&self, dx: i32, dy: i32) -> Rect {
        Rect::new(self.x + dx, self.y + dy, self.w, self.h)
    }

    /// Verschiebt an Ort und Stelle.
    pub fn move_ip(&mut self, dx: i32, dy: i32) {
        self.x += dx;
        self.y += dy;
    }

    /// Um `dx`/`dy` vergroesserte Kopie, mittig - negativ verkleinert.
    pub fn inflate(&self, dx: i32, dy: i32) -> Rect {
        Rect::new(
            self.x - dx / 2,
            self.y - dy / 2,
            self.w + dx,
            self.h + dy,
        )
    }

    /// Vergroessert an Ort und Stelle.
    pub fn inflate_ip(&mut self, dx: i32, dy: i32) {
        *self = self.inflate(dx, dy);
    }

    /// Um Faktoren skalierte Kopie, mittig.
    ///
    /// Zeilengetreu aus `RectExport_scalebyIp` in `src_c/rect_impl.h`:
    ///
    /// ```c
    /// factor_x = factor_x < 0 ? -factor_x : factor_x;      /* Betrag! */
    /// factor_y = (factor_y > 0) ? factor_y : factor_x;
    /// x = (int)(x + (w / 2) - (w * factor_x / 2));
    /// w = (int)(w * factor_x);
    /// ```
    ///
    /// Drei Dinge daran sind nicht zu erraten, und ich habe sie erst nach
    /// zwei falschen Anlaeufen aus der Quelle geholt:
    ///
    /// 1. **Der Faktor wird betragsmaessig genommen** - ein negativer
    ///    Faktor verkleinert nicht, er wirkt wie der positive.
    /// 2. **Gerechnet wird in `f32`, nicht `f64`.** Bei
    ///    `Rect(28,13,20,22).scale_by(3.7)` liegt `20 * 3.7` in `f32`
    ///    genau auf 74, in `f64` knapp darunter - das ist der Unterschied
    ///    zwischen dem gemessenen `x = 1` und einer 0.
    /// 3. **`w / 2` ist Ganzzahlteilung, `w * f / 2` nicht.** In einer
    ///    Zeile stehen beide Teilungsarten nebeneinander.
    ///
    /// Ueber 2500 Zufallsfaelle: 2500 Treffer. Die naheliegende Fassung
    /// ("neue Groesse runden, dann mittig setzen") trifft 1126.
    pub fn scale_by(&self, fx: f32, fy: f32) -> Rect {
        let fx = fx.abs();
        let fy = fy.abs();
        // Ein Faktor von 0 fuer y heisst "nimm den von x".
        let fy = if fy > 0.0 { fy } else { fx };

        let bx = self.w as f32 * fx;
        let by = self.h as f32 * fy;
        Rect::new(
            (self.x as f32 + (self.w / 2) as f32 - bx / 2.0) as i32,
            (self.y as f32 + (self.h / 2) as f32 - by / 2.0) as i32,
            bx as i32,
            by as i32,
        )
    }

    /// Wie [`scale_by`](Self::scale_by), aber an Ort und Stelle.
    pub fn scale_by_ip(&mut self, fx: f32, fy: f32) {
        *self = self.scale_by(fx, fy);
    }

    /// Setzt alle vier Zahlen neu - `pygame.Rect.update`.
    ///
    /// Das ist nicht dasselbe wie eine Zuweisung: in pygame ist `Rect` ein
    /// Verweis, und `update` aendert **das vorhandene Objekt**, sodass
    /// jeder, der es haelt, die neuen Werte sieht. In Rust ist `Rect`
    /// `Copy`, hier bleibt also nur die Bequemlichkeit - der Unterschied
    /// steht in `API.md`.
    pub fn update(&mut self, x: i32, y: i32, w: i32, h: i32) {
        *self = Rect::new(x, y, w, h);
    }

    /// Eine Kopie. In pygame noetig, hier eine Hoeflichkeit - `Rect` ist
    /// `Copy`, ein `let b = a;` tut dasselbe.
    pub fn copy(&self) -> Rect {
        *self
    }

    // Die Mitte **relativ zur eigenen Ecke** - `pygame.Rect.relcenter`.
    //
    // Also `(w/2, h/2)`, nicht `(x + w/2, y + h/2)`. Gemessen:
    // `Rect(2,3,10,6).relcenter` ist `(5, 3)`, nicht `(7, 6)`.
    //
    // **Setzen aendert die Groesse, nicht die Lage** - `w = 2*v.0`,
    // `h = 2*v.1`, die Ecke bleibt. Gemessen:
    // `r = Rect(1,2,75,45); r.relcenter = (697, 345)` gibt
    // `Rect(1, 2, 1394, 690)`. Zeilengetreu aus
    // `RectExport_setrelcenter`, `rect_impl.h` Zeile 2851. Der Setzer
    // fehlte hier ganz, und die Huelle meldete darum
    // `attribute 'relcenter' ... is not writable`.
    punkt_attr!(i32, i32, relcenter, set_relcenter, with_relcenter,
        |s| (s.w / 2, s.h / 2), |m, v| { m.w = v.0 * 2; m.h = v.1 * 2 });

    /// Das erste Rechteck der Liste, das dieses hier beruehrt.
    pub fn collideobjects(&self, andere: &[Rect]) -> Option<Rect> {
        andere.iter().copied().find(|r| self.colliderect(*r))
    }

    /// Alle Rechtecke der Liste, die dieses hier beruehren.
    pub fn collideobjectsall(&self, andere: &[Rect]) -> Vec<Rect> {
        andere
            .iter()
            .copied()
            .filter(|r| self.colliderect(*r))
            .collect()
    }

    /// Dreht negative Breiten/Hoehen um, sodass `w`/`h` positiv sind.
    pub fn normalize(&mut self) {
        if self.w < 0 {
            self.x += self.w;
            self.w = -self.w;
        }
        if self.h < 0 {
            self.y += self.h;
            self.h = -self.h;
        }
    }

    /// Normalisierte Kopie.
    pub fn normalized(&self) -> Rect {
        let mut r = *self;
        r.normalize();
        r
    }

    /// True, wenn das Rechteck keine Flaeche hat.
    pub fn is_empty(&self) -> bool {
        self.w <= 0 || self.h <= 0
    }

    // ----- Beziehungen zu anderen Rechtecken ----------------------------

    /// Liegt der Punkt im Rechteck? Die rechte und untere Kante zaehlen
    /// **nicht** dazu - genau wie in pygame.
    pub fn collidepoint(&self, p: (i32, i32)) -> bool {
        p.0 >= self.x && p.0 < self.x + self.w && p.1 >= self.y && p.1 < self.y + self.h
    }

    /// Ueberschneiden sich die beiden Rechtecke?
    pub fn colliderect(&self, other: Rect) -> bool {
        // **Beide Seiten erst normalisieren.** pygame tut das hier -
        // aber **nicht** bei `collidepoint`, `contains` oder `clip`.
        // Gemessen an `Rect(3, 4, -6, -8)`:
        //
        // ```text
        //   colliderect(Rect(0,0,5,5))  -> true      (normalisiert)
        //   collidepoint((1, 1))        -> false     (normalisiert nicht)
        //   contains(Rect(0,0,1,1))     -> false     (normalisiert nicht)
        //   clip(Rect(0,0,5,5))         -> (3, 4, 0, 0)
        // ```
        //
        // Das ist unbequem, aber es ist das gemessene Verhalten.
        let a = self.normalized();
        let b = other.normalized();
        // Ein Rechteck ohne Flaeche beruehrt **nichts** - auch nicht ein
        // anderes, das es rein rechnerisch ueberlappt. Gemessen:
        // `Rect(14,36,38,0).colliderect(Rect(30,4,19,33))` ist `false`.
        if a.w <= 0 || a.h <= 0 || b.w <= 0 || b.h <= 0 {
            return false;
        }
        a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
    }

    /// Steckt `other` vollstaendig in diesem Rechteck?
    ///
    /// **Sechs Bedingungen, nicht vier.** Die letzten beiden
    /// (`x + w > other.x`, `y + h > other.y`) sehen ueberfluessig aus,
    /// sind es aber nicht: sie schliessen das flaechenlose Rechteck
    /// genau auf der rechten oder unteren Kante aus. Gemessen an
    /// `Rect(1, 2, 3, 4)`:
    ///
    /// ```text
    ///   contains(Rect(2, 3, 0, 0))  -> true    (innen, ohne Flaeche)
    ///   contains(Rect(4, 6, 0, 0))  -> false   (auf der Kante)
    /// ```
    ///
    /// Mit nur vier Bedingungen waere der zweite Fall wahr. Hier standen
    /// vier, und pygames eigener `test_contains` ist genau darueber
    /// gestolpert. Zeilengetreu aus `RectExport_contains_internal`,
    /// `rect_impl.h` Zeile 1979 - `FRect` hatte die sechs schon.
    ///
    /// **Nicht normalisiert** - anders als `colliderect`; siehe die
    /// Messreihe dort.
    pub fn contains(&self, other: Rect) -> bool {
        self.x <= other.x
            && self.y <= other.y
            && self.x + self.w >= other.x + other.w
            && self.y + self.h >= other.y + other.h
            && self.x + self.w > other.x
            && self.y + self.h > other.y
    }

    /// Index des ersten getroffenen Rechtecks.
    pub fn collidelist(&self, others: &[Rect]) -> Option<usize> {
        others.iter().position(|o| self.colliderect(*o))
    }

    /// Alle getroffenen Rechtecke.
    pub fn collidelistall(&self, others: &[Rect]) -> Vec<usize> {
        others
            .iter()
            .enumerate()
            .filter(|(_, o)| self.colliderect(**o))
            .map(|(i, _)| i)
            .collect()
    }

    /// Die gemeinsame Flaeche beider Rechtecke (leer, wenn sie sich nicht
    /// beruehren).
    pub fn clip(&self, other: Rect) -> Rect {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.w).min(other.x + other.w);
        let y2 = (self.y + self.h).min(other.y + other.h);
        if x2 <= x1 || y2 <= y1 {
            Rect::new(self.x, self.y, 0, 0)
        } else {
            Rect::new(x1, y1, x2 - x1, y2 - y1)
        }
    }

    /// Das kleinste Rechteck, das beide umschliesst.
    pub fn union(&self, other: Rect) -> Rect {
        let x1 = self.x.min(other.x);
        let y1 = self.y.min(other.y);
        let x2 = (self.x + self.w).max(other.x + other.w);
        let y2 = (self.y + self.h).max(other.y + other.h);
        Rect::new(x1, y1, x2 - x1, y2 - y1)
    }

    /// Vereinigung an Ort und Stelle.
    pub fn union_ip(&mut self, other: Rect) {
        *self = self.union(other);
    }

    /// Das kleinste Rechteck, das dieses und alle anderen umschliesst.
    pub fn unionall(&self, others: &[Rect]) -> Rect {
        others.iter().fold(*self, |acc, o| acc.union(*o))
    }

    /// Vereinigung mit allen, an Ort und Stelle.
    pub fn unionall_ip(&mut self, others: &[Rect]) {
        *self = self.unionall(others);
    }

    /// Verschiebt dieses Rechteck so, dass es ganz in `other` liegt.
    ///
    /// Ist es groesser als `other`, wird es darin zentriert.
    ///
    /// **Zwei Ganzzahlteilungen, nicht eine.** Hier stand
    /// `other.x + (other.w - self.w) / 2`, und das Original rechnet
    /// `argrect->x + argrect->w / 2 - self->r.w / 2`
    /// (`RectExport_clamp`, `rect_impl.h` Zeile 2029). Zusammengefasst
    /// ist das dieselbe Formel - in ganzen Zahlen aber nicht dasselbe
    /// Ergebnis, sobald die beiden Breiten verschieden gerade sind.
    /// Gemessen an pygame:
    ///
    /// ```text
    ///   Rect(1,2,30,40).clamp(Rect(5,6,7,8))  -> Rect(-7, -10, 30, 40)
    ///   Rect(0,0,4,4).clamp(Rect(0,0,3,3))    -> Rect(-1, -1, 4, 4)
    /// ```
    ///
    /// Die zusammengefasste Form gab `-6` und `0`. Genau die Art Fehler,
    /// gegen die in diesem Ordner "uebertragen, nicht raten" steht:
    /// gekuerzt sah die Formel besser aus und war falsch.
    pub fn clamp(&self, other: Rect) -> Rect {
        let mut r = *self;
        r.x = if self.w >= other.w {
            other.x + other.w / 2 - self.w / 2
        } else if self.x < other.x {
            other.x
        } else if self.x + self.w > other.x + other.w {
            other.x + other.w - self.w
        } else {
            self.x
        };
        r.y = if self.h >= other.h {
            other.y + other.h / 2 - self.h / 2
        } else if self.y < other.y {
            other.y
        } else if self.y + self.h > other.y + other.h {
            other.y + other.h - self.h
        } else {
            self.y
        };
        r
    }

    /// Einpassen an Ort und Stelle.
    pub fn clamp_ip(&mut self, other: Rect) {
        *self = self.clamp(other);
    }

    /// Groesstmoegliche Kopie mit gleichem Seitenverhaeltnis, die in `other`
    /// passt (Briefkasten-Rechnung).
    ///
    /// **Zeilengetreu aus `RectExport_fit`** (`rect_impl.h` Zeile 2064):
    /// pygame bildet die Verhaeltnisse **andersherum** (`self.w / other.w`),
    /// nimmt davon das **groessere** und **teilt** dadurch. Hier stand die
    /// umgekehrte Fassung - `min(other.w/self.w, …)` und ein Malnehmen -
    /// samt einer Absicherung gegen Groesse Null, die es im Original nicht
    /// gibt. Bei runden, positiven Zahlen ist das dasselbe; bei Null oder
    /// negativer Groesse nicht, und das letzte Bit der `f32`-Division
    /// entscheidet nach dem Abschneiden ueber eine ganze Einheit.
    /// `frect.rs` hatte die richtige Fassung schon, samt Vermerk - hier
    /// stand die geratene, und sie wich in 3997 von 6000 gewuerfelten
    /// Proben ab.
    ///
    /// **Ueberlauf wird umgeklappt, nicht gemeldet.** Wenn `maxratio` Null
    /// wird, kommt aus der Division NaN oder Unendlich, daraus
    /// `x86_abschneiden` `i32::MIN`, und `other.w - i32::MIN` laeuft
    /// ueber. Das Original rechnet in C weiter und klappt um; gemessen an
    /// pygame:
    ///
    /// ```text
    ///   Rect(0,0,0,0).fit(Rect(0,0,10,10))
    ///     -> Rect(-1073741819, -1073741819, -2147483648, -2147483648)
    /// ```
    pub fn fit(&self, other: Rect) -> Rect {
        let xratio = self.w as f32 / other.w as f32;
        let yratio = self.h as f32 / other.h as f32;
        let maxratio = if xratio > yratio { xratio } else { yratio };
        let w = x86_abschneiden(self.w as f32 / maxratio);
        let h = x86_abschneiden(self.h as f32 / maxratio);
        Rect::new(
            other.x.wrapping_add(other.w.wrapping_sub(w) / 2),
            other.y.wrapping_add(other.h.wrapping_sub(h) / 2),
            w,
            h,
        )
    }

    /// Schneidet eine Strecke am Rechteck zu.
    ///
    /// `None`, wenn die Strecke das Rechteck gar nicht beruehrt.
    ///
    /// **Cohen-Sutherland in ganzen Zahlen, nicht Liang-Barsky.** pygame
    /// reicht das an SDLs `SDL_IntersectRectAndLine` weiter, und das
    /// rechnet jeden Schnittpunkt in `Sint64`: erst multiplizieren, dann
    /// **ganzzahlig** teilen (zur Null hin abschneiden). Eine
    /// Liang-Barsky-Fassung liefert dieselbe Strecke, aber an 203 von
    /// 2500 Zufallsfaellen einen um eins verschobenen Endpunkt - und
    /// dieselbe Rechnung in `f64` noch an 176.
    ///
    /// Die Kanten gehoeren dazu: das Rechteck reicht von `x` bis
    /// `x + w - 1`.
    pub fn clipline(&self, line: (i32, i32, i32, i32)) -> Option<(i32, i32, i32, i32)> {
        // pygame normalisiert vorher - siehe [`colliderect`](Self::colliderect).
        let n = self.normalized();
        if n.w <= 0 || n.h <= 0 {
            return None;
        }
        let (mut x1, mut y1, mut x2, mut y2) = line;
        let (rx1, ry1) = (n.x, n.y);
        let (rx2, ry2) = (n.x + n.w - 1, n.y + n.h - 1);

        // Liegt alles innen? Dann bleibt die Strecke, wie sie ist.
        if x1 >= rx1
            && x1 <= rx2
            && x2 >= rx1
            && x2 <= rx2
            && y1 >= ry1
            && y1 <= ry2
            && y2 >= ry1
            && y2 <= ry2
        {
            return Some((x1, y1, x2, y2));
        }
        // Liegt alles auf **einer** Seite? Dann beruehrt sie nichts.
        if (x1 < rx1 && x2 < rx1)
            || (x1 > rx2 && x2 > rx2)
            || (y1 < ry1 && y2 < ry1)
            || (y1 > ry2 && y2 > ry2)
        {
            return None;
        }

        // Waagerecht und senkrecht sind Sonderfaelle - und zwar nicht aus
        // Bequemlichkeit: die allgemeine Rechnung teilte dort durch null.
        if y1 == y2 {
            return Some((x1.clamp(rx1, rx2), y1, x2.clamp(rx1, rx2), y2));
        }
        if x1 == x2 {
            return Some((x1, y1.clamp(ry1, ry2), x2, y2.clamp(ry1, ry2)));
        }

        /// Die vier Randbits von Cohen-Sutherland.
        fn lage(rx1: i32, ry1: i32, rx2: i32, ry2: i32, x: i32, y: i32) -> u8 {
            let mut c = 0u8;
            if y < ry1 {
                c |= 1; // oben
            } else if y > ry2 {
                c |= 2; // unten
            }
            if x < rx1 {
                c |= 4; // links
            } else if x > rx2 {
                c |= 8; // rechts
            }
            c
        }

        let mut c1 = lage(rx1, ry1, rx2, ry2, x1, y1);
        let mut c2 = lage(rx1, ry1, rx2, ry2, x2, y2);
        while c1 != 0 || c2 != 0 {
            if c1 & c2 != 0 {
                return None;
            }
            // Immer den Punkt bewegen, der draussen liegt - aber
            // **gerechnet wird stets von `(x1, y1)` aus**, auch wenn der
            // zweite Punkt wandert. Das steht so in SDL und liest sich
            // wie ein Tippfehler; es ist keiner. Wer stattdessen vom
            // bewegten Punkt aus rechnet, trifft 2285 von 2500 Faellen -
            // nah genug, um es zu uebersehen, und falsch genug, um in
            // einem Spiel aufzufallen.
            let c = if c1 != 0 { c1 } else { c2 };
            let (px, py) = (x1, y1);
            // **Ganzzahlig, nicht in Fliesskomma.** SDL rechnet das
            // Produkt in `Sint64` und teilt dann ganzzahlig - die
            // Division schneidet also zur Null hin ab, und zwar erst
            // **nach** der Multiplikation. In `f64` gerechnet kommt an
            // 176 von 2500 Zufallsfaellen ein um eins verschobener
            // Endpunkt heraus.
            let (nx, ny);
            if c & 1 != 0 || c & 2 != 0 {
                ny = if c & 1 != 0 { ry1 } else { ry2 };
                nx = (px as i64
                    + ((x2 - x1) as i64 * (ny - py) as i64) / (y2 - y1) as i64)
                    as i32;
            } else {
                nx = if c & 4 != 0 { rx1 } else { rx2 };
                ny = (py as i64
                    + ((y2 - y1) as i64 * (nx - px) as i64) / (x2 - x1) as i64)
                    as i32;
            }
            if c1 != 0 {
                x1 = nx;
                y1 = ny;
                c1 = lage(rx1, ry1, rx2, ry2, x1, y1);
            } else {
                x2 = nx;
                y2 = ny;
                c2 = lage(rx1, ry1, rx2, ry2, x2, y2);
            }
        }
        Some((x1, y1, x2, y2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anker_lesen_und_setzen_verschiebt_nur() {
        let r = Rect::new(10, 20, 30, 40);
        assert_eq!(r.topleft(), (10, 20));
        assert_eq!(r.center(), (25, 40));
        assert_eq!(r.bottomright(), (40, 60));
        assert_eq!(r.midtop(), (25, 20));
        assert_eq!(r.midleft(), (10, 40));

        let m = r.with_center((100, 100));
        assert_eq!(m.size(), (30, 40), "die Groesse darf sich nie aendern");
        assert_eq!(m.center(), (100, 100));

        let m = r.with_bottomright((0, 0));
        assert_eq!(m, Rect::new(-30, -40, 30, 40));
    }

    #[test]
    fn rechte_und_untere_kante_zaehlen_nicht_dazu() {
        // Genau wie in pygame: ein Punkt auf der rechten Kante liegt aussen.
        let r = Rect::new(0, 0, 10, 10);
        assert!(r.collidepoint((0, 0)));
        assert!(r.collidepoint((9, 9)));
        assert!(!r.collidepoint((10, 5)));
        assert!(!r.collidepoint((5, 10)));
        assert!(!r.collidepoint((-1, 5)));
    }

    #[test]
    fn beruehrung_ist_noch_keine_kollision() {
        let a = Rect::new(0, 0, 10, 10);
        assert!(a.colliderect(Rect::new(9, 9, 5, 5)));
        assert!(!a.colliderect(Rect::new(10, 0, 5, 5)), "Kante an Kante");
        assert!(!a.colliderect(Rect::new(0, 10, 5, 5)));
    }

    #[test]
    fn schnitt_und_vereinigung() {
        let a = Rect::new(0, 0, 10, 10);
        let b = Rect::new(5, 5, 10, 10);
        assert_eq!(a.clip(b), Rect::new(5, 5, 5, 5));
        assert_eq!(a.union(b), Rect::new(0, 0, 15, 15));
        // Ohne Ueberschneidung ist der Schnitt leer.
        assert!(a.clip(Rect::new(50, 50, 5, 5)).is_empty());
        assert_eq!(
            a.unionall(&[b, Rect::new(-5, -5, 2, 2)]),
            Rect::new(-5, -5, 20, 20)
        );
    }

    #[test]
    fn einpassen_zentriert_was_zu_gross_ist() {
        let gross = Rect::new(0, 0, 100, 100);
        let klein = Rect::new(0, 0, 10, 10);
        // Passt hinein -> wird nur verschoben.
        assert_eq!(klein.with_topleft((-50, -50)).clamp(gross), Rect::new(0, 0, 10, 10));
        // Passt nicht -> wird zentriert.
        assert_eq!(gross.clamp(klein).center(), klein.center());
    }

    #[test]
    fn negative_groessen_werden_gedreht() {
        let mut r = Rect::new(10, 10, -5, -5);
        r.normalize();
        assert_eq!(r, Rect::new(5, 5, 5, 5));
    }

    #[test]
    fn strecke_wird_am_rechteck_zugeschnitten() {
        let r = Rect::new(0, 0, 10, 10);
        // Ganz innen: bleibt unveraendert.
        assert_eq!(r.clipline((2, 2, 7, 7)), Some((2, 2, 7, 7)));
        // Ragt heraus: wird gekuerzt.
        let g = r.clipline((-5, 5, 15, 5)).expect("muss treffen");
        assert_eq!((g.0, g.2), (0, 9));
        // Trifft gar nicht.
        assert_eq!(r.clipline((20, 20, 30, 30)), None);
    }

    #[test]
    fn einpassen_haelt_das_seitenverhaeltnis() {
        let quelle = Rect::new(0, 0, 100, 50);     // 2:1
        let ziel = Rect::new(0, 0, 100, 100);
        let f = quelle.fit(ziel);
        assert_eq!(f.size(), (100, 50));
        assert_eq!(f.center(), ziel.center());
    }

    /// Die neuen Methoden - alle Sollwerte an echtem pygame-ce gemessen,
    /// Ausgangsrechteck `Rect(2, 3, 10, 6)`.
    #[test]
    fn die_nachzuegler_treffen_pygame() {
        let r = Rect::new(2, 3, 10, 6);

        // pygames `scale_by_ip(2)` skaliert **beide** Achsen.
        let mut a = r;
        a.scale_by_ip(2.0, 2.0);
        assert_eq!(a, Rect::new(-3, 0, 20, 12));
        let mut b = r;
        b.scale_by_ip(2.0, 0.5);
        assert_eq!(b, Rect::new(-3, 4, 20, 3));

        let mut c = r;
        c.update(1, 2, 3, 4);
        assert_eq!(c, Rect::new(1, 2, 3, 4));

        assert_eq!(r.copy(), r);

        // **Relativ** zur eigenen Ecke: (5, 3), nicht (7, 6).
        assert_eq!(r.relcenter(), (5, 3));

        let liste = [Rect::new(5, 5, 2, 2), Rect::new(90, 90, 2, 2)];
        let gross = Rect::new(0, 0, 10, 10);
        assert_eq!(gross.collideobjects(&liste), Some(Rect::new(5, 5, 2, 2)));
        assert_eq!(gross.collideobjectsall(&liste), vec![Rect::new(5, 5, 2, 2)]);
        assert_eq!(Rect::new(50, 50, 1, 1).collideobjects(&liste), None);

        // `move_to(center=..)` heisst hier `with_center(..)` - gemessen
        // liefert pygame dafuer (-5, -3, 10, 6).
        assert_eq!(r.with_center((0, 0)), Rect::new(-5, -3, 10, 6));
        assert_eq!(r.with_bottomright((5, 5)), Rect::new(-5, -1, 10, 6));
    }

    /// Die Zahlen stammen aus pygames eigenem `RectTypeTest.test_contains`.
    #[test]
    fn contains_schliesst_die_kante_aus() {
        let r = Rect::new(1, 2, 3, 4);
        assert!(r.contains(Rect::new(2, 3, 1, 1)));
        assert!(r.contains(r));
        // Ohne Flaeche, aber innen: enthalten.
        assert!(r.contains(Rect::new(2, 3, 0, 0)));
        // Ohne Flaeche, aber genau auf der rechten unteren Kante: **nicht**
        // enthalten. Das trennen nur die beiden letzten Bedingungen.
        assert!(!r.contains(Rect::new(4, 6, 0, 0)));
        assert!(!r.contains(Rect::new(4, 6, 1, 1)));
        assert!(!r.contains(Rect::new(0, 0, 1, 2)));
    }

    /// **Die Probe gegen die zusammengefasste Formel.**
    ///
    /// Alle vier Zahlen sind an pygame gemessen. Die erste und die letzte
    /// gehen nur auf, wenn `clamp` zweimal ganzzahlig teilt statt einmal
    /// die Differenz; die gekuerzte Fassung gab `-6` und `0`.
    #[test]
    fn clamp_teilt_zweimal() {
        assert_eq!(
            Rect::new(1, 2, 30, 40).clamp(Rect::new(5, 6, 7, 8)),
            Rect::new(-7, -10, 30, 40)
        );
        assert_eq!(
            Rect::new(1, 2, 30, 40).clamp(Rect::new(5, 6, 8, 8)),
            Rect::new(-6, -10, 30, 40)
        );
        assert_eq!(
            Rect::new(-3, -3, 5, 5).clamp(Rect::new(-10, -10, 4, 4)),
            Rect::new(-10, -10, 5, 5)
        );
        assert_eq!(
            Rect::new(0, 0, 4, 4).clamp(Rect::new(0, 0, 3, 3)),
            Rect::new(-1, -1, 4, 4)
        );
    }

    /// `fit` teilt durch das **groessere** Verhaeltnis, statt mit dem
    /// kleineren Kehrwert malzunehmen - und rechnet bei Groesse Null
    /// ungeruehrt weiter. Beide Zeilen sind an pygame gemessen.
    #[test]
    fn fit_teilt_durch_das_groessere_verhaeltnis() {
        assert_eq!(
            Rect::new(10, 10, 30, 30).fit(Rect::new(30, 30, 15, 10)),
            Rect::new(32, 30, 10, 10)
        );
        assert_eq!(
            Rect::new(0, 0, 3, 7).fit(Rect::new(0, 0, 10, 10)),
            Rect::new(3, 0, 4, 10)
        );
        // Ohne Flaeche: das Original sichert nichts ab, aus der Division
        // wird NaN, daraus `i32::MIN`, und die Differenz klappt um.
        assert_eq!(
            Rect::new(0, 0, 0, 0).fit(Rect::new(0, 0, 10, 10)),
            Rect::new(-1073741819, -1073741819, i32::MIN, i32::MIN)
        );
        assert_eq!(
            Rect::new(-26, -28, 0, -4).fit(Rect::new(23, -19, 8, 50)),
            Rect::new(-1073741797, -1073741818, i32::MIN, i32::MIN)
        );
    }

    /// Gemessen an pygame: `Rect(1,2,75,45).relcenter = (697, 345)`.
    #[test]
    fn relcenter_setzen_aendert_nur_die_groesse() {
        let mut r = Rect::new(1, 2, 75, 45);
        r.set_relcenter((697, 345));
        assert_eq!(r, Rect::new(1, 2, 1394, 690));
        assert_eq!(r.relcenter(), (697, 345));
    }
}
