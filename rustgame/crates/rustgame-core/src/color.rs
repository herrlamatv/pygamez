//! color.rs
//! ========
//! Farben - Gegenstueck zu `pygame.Color`.
//!
//! Intern rechnet rustgame durchgehend mit **ARGB8888** in einem `u32`, also
//! genau dem Format, das pygame auf dieser Plattform benutzt (gemessen:
//! Masken `0xFF0000, 0xFF00, 0xFF` plus `0xFF000000` bei `SRCALPHA`). Ein
//! einziges Format genuegt damit fuer alles - Paletten und 16-Bit-Flaechen
//! entfallen ersatzlos.

/// Eine Farbe mit vier Kanaelen.
///
/// Wie in pygame ist Alpha optional: wird keins angegeben, gilt 255.
///
/// ## Warum `repr(C)`
///
/// Seit `pygame.Color` einen Puffer herausgibt, ist die Reihenfolge dieser
/// vier Bytes im Speicher Teil der Schnittstelle: `memoryview(color)[0]`
/// muss Rot sein und `[3]` Alpha. Rust darf Felder sonst umsortieren - bei
/// vier gleich grossen Feldern tut es das praktisch nie, aber "praktisch
/// nie" ist keine Zusage, und hier haengt fremder Code daran.
/// `color_test.test_newbuf` schreibt durch den Puffer zurueck
/// (`imp[0] = 60` und dann `c.r == 60`); eine andere Ordnung faerbt die
/// Farbe um.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Color {
    /// Rot.
    pub r: u8,
    /// Gruen.
    pub g: u8,
    /// Blau.
    pub b: u8,
    /// Deckung - 255 ist voll deckend.
    pub a: u8,
}

impl Color {
    /// Neue Farbe, undurchsichtig.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b, a: 255 }
    }

    /// Neue Farbe mit Alpha.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color { r, g, b, a }
    }

    /// Aus einem gepackten ARGB-Wort.
    pub const fn from_argb(v: u32) -> Color {
        Color {
            a: (v >> 24) as u8,
            r: (v >> 16) as u8,
            g: (v >> 8) as u8,
            b: v as u8,
        }
    }

    /// Als gepacktes ARGB-Wort - so liegt die Farbe im Pixelpuffer.
    pub const fn to_argb(self) -> u32 {
        (self.a as u32) << 24 | (self.r as u32) << 16 | (self.g as u32) << 8 | self.b as u32
    }

    /// Die vier Kanaele als Tupel (wie `tuple(pygame.Color(...))`).
    pub const fn as_tuple(self) -> (u8, u8, u8, u8) {
        (self.r, self.g, self.b, self.a)
    }

    /// Dieselbe Farbe mit anderem Alpha.
    pub const fn with_alpha(self, a: u8) -> Color {
        Color { a, ..self }
    }

    /// Graustufe nach derselben Gewichtung wie `pygame.transform.grayscale`.
    pub fn grayscale(self) -> Color {
        let y = (self.r as f32 * 0.299 + self.g as f32 * 0.587 + self.b as f32 * 0.114) as u8;
        Color {
            r: y,
            g: y,
            b: y,
            a: self.a,
        }
    }

    // ----- Die Farbraeume ------------------------------------------------
    //
    // pygame rechnet hier in `double` und rundet **nicht einheitlich**:
    // `lerp` und `correct_gamma` runden kaufmaennisch, die Rueckwege aus
    // HSV und HSL schneiden ab. Das ist keine Nachlaessigkeit im Nachbau,
    // sondern gemessenes Verhalten - siehe die Tests am Dateiende.

    /// `(H, S, V, A)` - Farbton 0..360, Rest 0..100. Wie `pygame.Color.hsva`.
    pub fn hsva(self) -> (f64, f64, f64, f64) {
        let (r, g, b) = (
            self.r as f64 / 255.0,
            self.g as f64 / 255.0,
            self.b as f64 / 255.0,
        );
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let d = max - min;
        // `(100 * d) / max`, **nicht** `(d / max) * 100` - dieselbe
        // Rechnung, andere letzte Stelle. Gemessen: 600/600 gegen 399/600.
        let s = if max == 0.0 { 0.0 } else { (100.0 * d) / max };
        (
            farbton(r, g, b, max, d),
            s,
            max * 100.0,
            self.a as f64 / 255.0 * 100.0,
        )
    }

    /// `(H, S, L, A)` - wie `pygame.Color.hsla`.
    pub fn hsla(self) -> (f64, f64, f64, f64) {
        let (r, g, b) = (
            self.r as f64 / 255.0,
            self.g as f64 / 255.0,
            self.b as f64 / 255.0,
        );
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let d = max - min;
        let l = (max + min) / 2.0;
        // Auch hier zuerst mal 100 - siehe [`Color::hsva`].
        let s = if d == 0.0 {
            0.0
        } else if l <= 0.5 {
            100.0 * (d / (max + min))
        } else {
            100.0 * (d / (2.0 - max - min))
        };
        (
            farbton(r, g, b, max, d),
            s,
            l * 100.0,
            self.a as f64 / 255.0 * 100.0,
        )
    }

    /// Aus `(H, S, V, A)`.
    ///
    /// Zeilengetreu aus `src_c/color.c` (`_color_set_hsva`) uebertragen -
    /// samt der Reihenfolge der Rechenschritte. Das ist **kein** Purismus:
    /// siehe die Anmerkung bei [`Color::from_hsla`].
    pub fn from_hsva(h: f64, s: f64, v: f64, a: f64) -> Color {
        let (s, v) = (s / 100.0, v / 100.0);
        let hi = (h / 60.0).floor();
        let f = (h / 60.0) - hi;
        let p = v * (1.0 - s);
        let q = v * (1.0 - s * f);
        let t = v * (1.0 - s * (1.0 - f));

        let (r, g, b) = match hi as i32 {
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            5 => (v, p, q),
            // 0 und 6 sind dasselbe.
            _ => (v, t, p),
        };
        Color::rgba(
            abschneiden(r),
            abschneiden(g),
            abschneiden(b),
            abschneiden(a / 100.0),
        )
    }

    /// Aus `(H, S, L, A)`.
    ///
    /// Ebenfalls zeilengetreu aus `src_c/color.c` (`_color_set_hsla`), und
    /// hier ist der Grund greifbar: die naheliegende Formulierung
    /// `c = (1 - |2l-1|) * s` ist mathematisch gleichwertig, liefert aber
    /// **andere Zahlen**. Bei `l = 1.0, s = 0.17` ergibt pygames
    /// `q = l + s - (l * s)` in Fliesskomma nicht 1, sondern
    /// `0.9999999999999999` - mal 255 und abgeschnitten sind das **254**,
    /// nicht 255.
    ///
    /// Genau 16 von 662 Vergleichen sind daran gescheitert, bis die
    /// Reihenfolge stimmte. Wer solche Rechnungen "aufraeumt", baut
    /// Abweichungen ein, die niemand mehr findet.
    pub fn from_hsla(h: f64, s: f64, l: f64, a: f64) -> Color {
        const EIN_DRITTEL: f64 = 1.0 / 3.0;
        let alpha = abschneiden(a / 100.0);
        let s = s / 100.0;
        let l = l / 100.0;

        if s == 0.0 {
            let g = abschneiden(l);
            return Color::rgba(g, g, g, alpha);
        }

        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - (l * s)
        };
        let p = 2.0 * l - q;
        let ht = h / 360.0;

        /// Ein Kanal - im C-Original dreimal ausgeschrieben.
        fn kanal(mut h: f64, p: f64, q: f64) -> f64 {
            if h < 0.0 {
                h += 1.0;
            } else if h > 1.0 {
                h -= 1.0;
            }
            if h < 1.0 / 6.0 {
                p + ((q - p) * 6.0 * h)
            } else if h < 0.5 {
                q
            } else if h < 2.0 / 3.0 {
                p + ((q - p) * 6.0 * (2.0 / 3.0 - h))
            } else {
                p
            }
        }

        Color::rgba(
            abschneiden(kanal(ht + EIN_DRITTEL, p, q)),
            abschneiden(kanal(ht, p, q)),
            abschneiden(kanal(ht - EIN_DRITTEL, p, q)),
            alpha,
        )
    }

    /// `(C, M, Y)`, je 0..1 - schlicht das Gegenteil von RGB.
    pub fn cmy(self) -> (f64, f64, f64) {
        (
            1.0 - self.r as f64 / 255.0,
            1.0 - self.g as f64 / 255.0,
            1.0 - self.b as f64 / 255.0,
        )
    }

    /// Aus `(C, M, Y)`. Das Alpha bleibt, wie es war.
    pub fn from_cmy(self, c: f64, m: f64, y: f64) -> Color {
        Color::rgba(
            abschneiden(1.0 - c),
            abschneiden(1.0 - m),
            abschneiden(1.0 - y),
            self.a,
        )
    }

    /// Der I1I2I3-Farbraum nach Ohta - `pygame.Color.i1i2i3`.
    pub fn i1i2i3(self) -> (f64, f64, f64) {
        let (r, g, b) = (
            self.r as f64 / 255.0,
            self.g as f64 / 255.0,
            self.b as f64 / 255.0,
        );
        ((r + g + b) / 3.0, (r - b) / 2.0, (2.0 * g - r - b) / 4.0)
    }

    /// Aus `(I1, I2, I3)`. Das Alpha bleibt, wie es war.
    ///
    /// Zeilengetreu aus `_color_set_i1i2i3` in `src_c/color.c` - und das
    /// ist hier **kein** Formalismus. Die aufgeloeste Form
    ///
    /// ```text
    /// r = i1 + i2 - 2/3 * i3
    /// g = i1 + 4/3 * i3
    /// b = i1 - i2 - 2/3 * i3
    /// ```
    ///
    /// ist algebraisch dasselbe wie pygames Kette, aber nicht in
    /// Gleitkomma. Bei `Color(200, 10, 10)` hin und zurueck:
    ///
    /// ```text
    /// pygame   3*i1 - ar - ab  ->  0.03921568627450977 * 255 =  9.999999999999991  ->  9
    /// aufgeloest  i1 + 4/3*i3  ->  0.03921568627450982 * 255 = 10.000000000000004  -> 10
    /// ```
    ///
    /// Ein Zehnmillionstel Unterschied, und weil danach **abgeschnitten**
    /// wird statt gerundet, faellt er als ganze Stufe ins Ergebnis. Genau
    /// deshalb wird die Kette hier Schritt fuer Schritt nachgerechnet,
    /// obwohl sie umstaendlich aussieht.
    pub fn from_i1i2i3(self, i1: f64, i2: f64, i3: f64) -> Color {
        let ab = i1 - i2 - 2.0 * i3 / 3.0;
        let ar = 2.0 * i2 + ab;
        let ag = 3.0 * i1 - ar - ab;
        Color::rgba(abschneiden(ar), abschneiden(ag), abschneiden(ab), self.a)
    }

    /// Alle vier Kanaele als 0..1.
    pub fn normalize(self) -> (f64, f64, f64, f64) {
        (
            self.r as f64 / 255.0,
            self.g as f64 / 255.0,
            self.b as f64 / 255.0,
            self.a as f64 / 255.0,
        )
    }

    /// Wie [`normalize`](Self::normalize) - pygame hat beide Namen, einmal
    /// als Methode und einmal als Eigenschaft.
    pub fn normalized(self) -> (f64, f64, f64, f64) {
        self.normalize()
    }

    /// Setzt alle vier Kanaele neu - `pygame.Color.update`.
    ///
    /// In pygame ist `Color` veraenderlich und `update` aendert das
    /// vorhandene Objekt; hier ist `Color` `Copy`, es bleibt also die
    /// Bequemlichkeit. Der Unterschied steht in `API.md`.
    pub fn update(&mut self, r: u8, g: u8, b: u8, a: u8) {
        *self = Color::rgba(r, g, b, a);
    }

    /// Aus vier Werten zwischen 0 und 1.
    ///
    /// **Rundet kaufmaennisch**, anders als die Farbraeume daneben:
    ///
    /// ```c
    /// color->data[0] = (Uint8)round(frgba[0] * 255.0);
    /// ```
    ///
    /// Hier stand `abschneiden`, und das war eine Abweichung um eins:
    /// `Color(1,2,3,4).normalized = (0.5, 0.5, 0.5)` gibt in pygame
    /// `(128, 128, 128, 255)`, abgeschnitten aber `(127, …)`.
    /// Nachgemessen. `_color_set_normalized` ist die **einzige** Stelle in
    /// `color.c`, die `round()` benutzt - `correct_gamma` schreibt
    /// daneben ausdruecklich `* 255 + .5` samt Begruendung, und die
    /// Rueckwege aus HSV/HSL schneiden ab.
    pub fn from_normalized(r: f64, g: f64, b: f64, a: f64) -> Color {
        Color::rgba(
            kaufmaennisch(r),
            kaufmaennisch(g),
            kaufmaennisch(b),
            kaufmaennisch(a),
        )
    }

    /// Die Farbe mit ihrem Alpha vormultipliziert.
    ///
    /// Die Rechnung ist `((c + 1) * a) >> 8` - **nicht** `c * a / 255`. Der
    /// Unterschied ist gemessen: ueber alle 65.536 Paare aus Farbwert und
    /// Alpha liegt die naheliegende Formel 15.962-mal daneben, SDLs
    /// Rundungsmakro 18.760-mal. Diese hier: 0-mal.
    pub fn premul_alpha(self) -> Color {
        let a = self.a as u32;
        let f = |c: u8| (((c as u32 + 1) * a) >> 8) as u8;
        Color::rgba(f(self.r), f(self.g), f(self.b), self.a)
    }

    /// Gammakorrektur - `pygame.Color.correct_gamma`.
    pub fn correct_gamma(self, gamma: f64) -> Color {
        let f = |c: u8| runden((c as f64 / 255.0).powf(gamma));
        Color::rgba(f(self.r), f(self.g), f(self.b), f(self.a))
    }

    /// Mischung zwischen zwei Farben, `t` von 0 bis 1.
    ///
    /// pygame rechnet `round(a * (1 - t) + b * t)` - die **ganze** Summe
    /// wird gerundet. Die Fassung `a + (b - a) * t + 0.5` ist algebraisch
    /// dasselbe und liegt trotzdem daneben: `Color(..,250).lerp(schwarz,
    /// 0.77)` ergibt damit 57 statt der gemessenen 58.
    pub fn lerp(self, other: Color, t: f64) -> Color {
        let f = |a: u8, b: u8| {
            ((a as f64 * (1.0 - t) + b as f64 * t).round() as i64).clamp(0, 255) as u8
        };
        Color::rgba(
            f(self.r, other.r),
            f(self.g, other.g),
            f(self.b, other.b),
            f(self.a, other.a),
        )
    }

    /// `#rrggbbaa` - immer mit Alpha, immer klein geschrieben (wie pygame).
    pub fn hex(self) -> String {
        format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            self.r, self.g, self.b, self.a
        )
    }

    /// Aus `#rrggbb`, `#rrggbbaa`, `0xrrggbb` oder `0xrrggbbaa`.
    ///
    /// **Kein Farbname** - `Color.from_hex("red")` ist in pygame ein
    /// `ValueError`, weil dort `_color_set_hex` unmittelbar `_hexcolor`
    /// aufruft und die Namenstabelle nie zu sehen bekommt. Hier stand
    /// frueher `parse`, das beides annahm; der Kopf sagte ausserdem
    /// `#rgb`, was weder hier noch in pygame je ging (`_hexcolor`
    /// verlangt Laenge 7 oder 9).
    pub fn from_hex(text: &str) -> Option<Color> {
        hex_farbe(text)
    }
}

// ----- Hilfsrechnungen ---------------------------------------------------

/// Der Farbton in Grad, gemeinsam fuer HSV und HSL.
///
/// **Die Klammerung ist nicht austauschbar.** pygame multipliziert
/// *zuerst* mit 60 und nimmt den Rest *danach*; die naheliegende Fassung
/// (`((g-b)/d) % 6.0 * 60.0`) ist mathematisch dasselbe und trifft
/// trotzdem nur 471 von 600 Messwerten - der Unterschied ist ein ULP und
/// kippt hinterher einen ganzen Farbkanal, wenn man zurueckrechnet.
/// Ebenso stehen die Zweige fuer Gruen und Blau als `+ 120` und `+ 240`
/// **nach** der Multiplikation, nicht als `+ 2` und `+ 4` davor.
fn farbton(r: f64, g: f64, b: f64, max: f64, d: f64) -> f64 {
    if d == 0.0 {
        return 0.0;
    }
    if max == r {
        let h = (60.0 * ((g - b) / d)) % 360.0;
        return if h < 0.0 { h + 360.0 } else { h };
    }
    if max == g {
        return 60.0 * ((b - r) / d) + 120.0;
    }
    60.0 * ((r - g) / d) + 240.0
}

/// 0..1 auf 0..255 - **abschneiden**, nicht runden.
///
/// Gemessen: `hsla = (240, 50, 50, 100)` ergibt in pygame `(63, 63, 191)`.
/// Die genaue Rechnung liefert 63,75 und 191,25 - wer kaufmaennisch rundet,
/// bekommt 64 und 191 und liegt beim ersten Wert daneben.
fn abschneiden(v: f64) -> u8 {
    (v * 255.0).clamp(0.0, 255.0) as u8
}

/// 0..1 auf 0..255 - hier **kaufmaennisch**, anders als `abschneiden`.
///
/// Gemessen an `correct_gamma`: 188,94 wird 189, 225,83 wird 226. Dass
/// pygame beide Rundungen benutzt, ist keine Vermutung, sondern der Grund,
/// warum es hier zwei Funktionen gibt.
///
/// `+ 0.5` und abschneiden, nicht `round()` - so steht es in
/// `_color_correct_gamma`, mit der Begruendung daneben ("visual studio
/// doesn't have a round func"). Fuer `normalized` gilt das **nicht**,
/// siehe [`kaufmaennisch`]; der Unterschied zeigt sich bei Werten, deren
/// Summe mit 0,5 in Gleitkomma gerade noch aufrundet.
fn runden(v: f64) -> u8 {
    (v * 255.0 + 0.5).clamp(0.0, 255.0) as u8
}

/// 0..1 auf 0..255 mit Cs `round()` - nur fuer `normalized`.
///
/// Die dritte Rundung in dieser Datei, und die einzige, die wirklich
/// `round()` heisst. Warum sie nicht mit [`runden`] zusammengelegt ist:
/// die beiden stehen in `color.c` als verschiedene Ausdruecke da, und
/// `floor(v + 0.5)` und `round(v)` gehen fuer Werte wie
/// `0.49999999999999994` auseinander.
fn kaufmaennisch(v: f64) -> u8 {
    (v * 255.0).round().clamp(0.0, 255.0) as u8
}

/// Alles, was als Farbe durchgeht - damit sich die API wie in Python anfuehlt.
///
/// ```
/// use rustgame_core::color::{Color, IntoColor};
/// let a = (255, 0, 0).into_color();
/// let b = (255, 0, 0, 128).into_color();
/// let c = Color::rgb(255, 0, 0).into_color();
/// assert_eq!(a.r, 255);
/// assert_eq!(b.a, 128);
/// assert_eq!(c, a);
/// ```
pub trait IntoColor {
    /// Macht daraus eine [`Color`].
    fn into_color(self) -> Color;
}

impl IntoColor for Color {
    fn into_color(self) -> Color {
        self
    }
}

impl IntoColor for (u8, u8, u8) {
    fn into_color(self) -> Color {
        Color::rgb(self.0, self.1, self.2)
    }
}

impl IntoColor for (u8, u8, u8, u8) {
    fn into_color(self) -> Color {
        Color::rgba(self.0, self.1, self.2, self.3)
    }
}

impl IntoColor for [u8; 3] {
    fn into_color(self) -> Color {
        Color::rgb(self[0], self[1], self[2])
    }
}

impl IntoColor for [u8; 4] {
    fn into_color(self) -> Color {
        Color::rgba(self[0], self[1], self[2], self[3])
    }
}

impl IntoColor for u32 {
    /// Ein bereits gepacktes ARGB-Wort.
    fn into_color(self) -> Color {
        Color::from_argb(self)
    }
}

impl IntoColor for &str {
    /// `"#rrggbb"`, `"#rrggbbaa"` oder ein Farbname aus [`by_name`].
    ///
    /// Unbekannte Angaben werden zu Schwarz - pygame wirft hier einen
    /// Fehler, aber ein Zeichenaufruf soll nie das Programm mitreissen.
    fn into_color(self) -> Color {
        parse(self).unwrap_or(Color::rgb(0, 0, 0))
    }
}

/// Die erzeugte Namenstabelle - siehe `tools/gen_farben.py`.
///
/// Als `#[path]`-Modul eingehaengt und nicht ueber `lib.rs`, damit die
/// Tabelle dort sitzt, wo sie hingehoert: als Innenleben von `color`,
/// von aussen unsichtbar.
#[path = "farbnamen.rs"]
mod farbnamen;

/// Liest einen Farbnamen, `"#rrggbb[aa]"` oder `"0xrrggbb[aa]"`.
///
/// Die Reihenfolge ist die von `_parse_color_from_text` in `color.c`:
/// zuerst der Name **wie geschrieben**, dann die Hexschreibweise, und
/// erst danach der Name ohne Leerzeichen und klein geschrieben. Die
/// letzte Stufe ist der Grund, warum `Color("Alice Blue")` geht.
pub fn parse(text: &str) -> Option<Color> {
    if let Some(c) = tabelle(text) {
        return Some(c);
    }
    if let Some(c) = hex_farbe(text) {
        return Some(c);
    }
    tabelle(&text.replace(' ', "").to_ascii_lowercase())
}

/// Ein Farbname aus `pygame.colordict.THECOLORS` - alle 665.
///
/// Hier standen einmal 22 handgepflegte Namen, mit dem Vermerk, die
/// vollstaendige Liste werde "im Projekt nirgends benutzt". Fuer die
/// Spiele stimmte das; fuer die API-Treue nie - `color_test.py` geht
/// die ganze Tabelle durch. Jetzt wird sie erzeugt
/// ([`tools/gen_farben.py`](../../tools/gen_farben.py)).
pub fn by_name(name: &str) -> Option<Color> {
    tabelle(name).or_else(|| tabelle(&name.replace(' ', "").to_ascii_lowercase()))
}

/// Alle Farbnamen mit ihren Werten - die Vorlage fuer `THECOLORS`.
///
/// pygame legt dieselbe Tabelle dreimal aus (`pygame.colordict`,
/// `pygame.color` und, ueber den Import, unter beiden Namen dasselbe
/// `dict`-Objekt - gemessen mit `is`). Damit die Huelle sie nicht
/// nachbaut, gibt der Kern sie hier heraus.
pub fn alle_namen() -> &'static [(&'static str, (u8, u8, u8))] {
    farbnamen::TABELLE
}

/// Binaere Suche in der erzeugten Tabelle - **ohne** Zurechtbiegen.
fn tabelle(name: &str) -> Option<Color> {
    farbnamen::TABELLE
        .binary_search_by_key(&name, |(n, _)| n)
        .ok()
        .map(|i| {
            let (r, g, b) = farbnamen::TABELLE[i].1;
            Color::rgb(r, g, b)
        })
}

/// `#rrggbb`, `#rrggbbaa`, `0xrrggbb` oder `0xrrggbbaa` - sonst nichts.
///
/// Zeilengetreu nach `_hexcolor` in `color.c`: die Laenge muss **genau**
/// stimmen (7/9 mit `#`, 8/10 mit `0x`), und jede Ziffer geht durch
/// `_hextoint`, das ausser `0-9A-Fa-f` nichts durchlaesst. Deshalb
/// scheitern `"#-f000000"` und `"#f 000000"`, obwohl Rusts
/// `from_str_radix` beim ersten ein Vorzeichen saehe.
fn hex_farbe(text: &str) -> Option<Color> {
    let b = text.as_bytes();
    let ziffer = |c: u8| (c as char).to_digit(16).map(|v| v as u8);
    let paar = |i: usize| Some(ziffer(*b.get(i)?)? << 4 | ziffer(*b.get(i + 1)?)?);
    let ab = match (b.first(), b.get(1), b.len()) {
        (Some(b'#'), _, 7) | (Some(b'#'), _, 9) => 1,
        (Some(b'0'), Some(b'x'), 8) | (Some(b'0'), Some(b'x'), 10) => 2,
        _ => return None,
    };
    let a = if b.len() == ab + 8 { paar(ab + 6)? } else { 255 };
    Some(Color::rgba(paar(ab)?, paar(ab + 2)?, paar(ab + 4)?, a))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packen_und_entpacken_sind_umkehrbar() {
        for c in [
            Color::rgb(0, 0, 0),
            Color::rgb(255, 255, 255),
            Color::rgba(1, 2, 3, 4),
            Color::rgba(200, 100, 50, 128),
        ] {
            assert_eq!(Color::from_argb(c.to_argb()), c);
        }
    }

    #[test]
    fn das_format_ist_argb() {
        // Genau so liegt die Farbe im Pixelpuffer - und genau so erwartet
        // es pygame auf dieser Plattform (Masken 0xFF0000/0xFF00/0xFF).
        let c = Color::rgba(0x11, 0x22, 0x33, 0x44);
        assert_eq!(c.to_argb(), 0x44_11_22_33);
    }

    #[test]
    fn farben_kommen_in_jeder_schreibweise_an() {
        assert_eq!((1u8, 2u8, 3u8).into_color(), Color::rgb(1, 2, 3));
        assert_eq!((1u8, 2u8, 3u8, 4u8).into_color(), Color::rgba(1, 2, 3, 4));
        assert_eq!("#ff0000".into_color(), Color::rgb(255, 0, 0));
        assert_eq!("#ff000080".into_color(), Color::rgba(255, 0, 0, 128));
        assert_eq!("red".into_color(), Color::rgb(255, 0, 0));
        // Unbekanntes reisst nichts mit sich.
        assert_eq!("gibt-es-nicht".into_color(), Color::rgb(0, 0, 0));
    }

    #[test]
    fn ohne_alpha_ist_undurchsichtig() {
        assert_eq!(Color::rgb(1, 2, 3).a, 255);
    }

    /// Kurz genug: `(a - b).abs() < 1e-9`.
    fn nah(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    /// Alle Sollwerte stammen aus echtem pygame-ce 2.5.7.
    ///
    /// Der vollstaendige Abgleich lief ueber 662 Vergleiche - 15 Farben
    /// mal zehn Eigenschaften plus 512 Rueckwege aus HSVA und HSLA -
    /// und endete bei **0 Abweichungen**. Hier steht davon eine
    /// Auswahl, die jede Rechnung einmal beruehrt.
    #[test]
    fn die_farbraeume_rechnen_wie_pygame() {
        let c = Color::rgba(30, 140, 200, 90);

        let (h, s, v, a) = c.hsva();
        assert!(nah(h, 201.1764705882353), "Farbton {h}");
        assert!(nah(s, 84.99999999999999), "Saettigung {s}");
        assert!(nah(v, 78.43137254901961), "Hellwert {v}");
        assert!(nah(a, 35.294117647058826), "Alpha {a}");

        let (h, s, l, _) = c.hsla();
        assert!(nah(h, 201.1764705882353));
        assert!(nah(s, 73.91304347826086));
        assert!(nah(l, 45.09803921568628));

        let (cy, m, y) = c.cmy();
        assert!(nah(cy, 0.8823529411764706) && nah(m, 0.4509803921568627));
        assert!(nah(y, 0.21568627450980393));

        let (i1, i2, i3) = c.i1i2i3();
        assert!(nah(i1, 0.4836601307189543));
        assert!(nah(i2, -0.3333333333333333));
        assert!(nah(i3, 0.049019607843137275));

        assert_eq!(c.premul_alpha().as_tuple(), (10, 49, 70, 90));
        assert_eq!(c.correct_gamma(0.5).as_tuple(), (87, 189, 226, 151));
        assert_eq!(
            c.lerp(Color::rgba(255, 255, 255, 255), 0.25).as_tuple(),
            (86, 169, 214, 131)
        );
        assert_eq!(c.hex(), "#1e8cc85a");
        assert_eq!(Color::from_hex("#1e8cc85a"), Some(c));

        // Drei Rundungen, drei Ergebnisse - alle drei Sollwerte aus
        // pygame. `normalized` rundet kaufmaennisch (0,5 -> 128), die
        // Rueckwege aus HSL schneiden ab (0,5 -> 127).
        assert_eq!(
            Color::from_normalized(0.5, 0.5, 0.5, 1.0).as_tuple(),
            (128, 128, 128, 255)
        );
        assert_eq!(
            Color::from_normalized(0.0, 1.0, 0.002, 0.5).as_tuple(),
            (0, 255, 1, 128)
        );
    }

    /// I1I2I3 hin und zurueck - und die eine Farbe, die dabei verliert.
    ///
    /// Sechs der sieben Farben kommen unveraendert wieder heraus. Genau
    /// eine nicht: `(200, 10, 10)` wird `(200, 9, 9)`. Das ist **kein**
    /// Fehler der Uebertragung, sondern pygames Verhalten - und es ist der
    /// einzige Beleg dafuer, dass die Rechenkette aus `color.c` Schritt
    /// fuer Schritt nachgerechnet wird und nicht in der aufgeloesten Form
    /// steht. Die naemlich liefert hier `(200, 10, 9)`.
    #[test]
    fn i1i2i3_hin_und_zurueck_verliert_genau_wie_pygame() {
        /// Ausgangsfarbe und was nach hin und zurueck herauskommt.
        type Fall = ((u8, u8, u8), (u8, u8, u8));
        let faelle: &[Fall] = &[
            ((200, 10, 10), (200, 9, 9)),
            ((30, 140, 200), (30, 140, 200)),
            ((0, 0, 0), (0, 0, 0)),
            ((255, 255, 255), (255, 255, 255)),
            ((1, 2, 3), (1, 2, 3)),
            ((17, 200, 99), (17, 200, 99)),
            ((255, 0, 128), (255, 0, 128)),
        ];
        for &((r, g, b), soll) in faelle {
            let (i1, i2, i3) = Color::rgba(r, g, b, 255).i1i2i3();
            // Das Alpha kommt vom Ziel, nicht vom Weg - wie in pygame.
            let zurueck = Color::rgba(9, 9, 9, 9).from_i1i2i3(i1, i2, i3);
            assert_eq!(
                (zurueck.r, zurueck.g, zurueck.b),
                soll,
                "i1i2i3 von ({r}, {g}, {b})"
            );
            assert_eq!(zurueck.a, 9, "das Alpha bleibt unberuehrt");
        }
    }

    /// Die Rueckwege - und der Stolperstein, der sie interessant macht.
    #[test]
    fn die_rueckwege_treffen_pygames_rundung() {
        assert_eq!(
            Color::from_hsva(120.0, 100.0, 100.0, 100.0).as_tuple(),
            (0, 255, 0, 255)
        );
        // 63, nicht 64: pygame schneidet hier ab (63,75 -> 63).
        assert_eq!(
            Color::from_hsla(240.0, 50.0, 50.0, 100.0).as_tuple(),
            (63, 63, 191, 255)
        );

        // **Der Stolperstein.** Rechnerisch ist das reines Weiss. pygame
        // rechnet `q = l + s - (l * s)`, und in Fliesskomma ergibt
        // `1.0 + 0.17 - 0.17` nicht 1, sondern 0.9999999999999999 - mal
        // 255 und abgeschnitten also 254. Wer die Formel "aufraeumt",
        // faellt genau hier durch.
        assert_eq!(
            Color::from_hsla(0.0, 17.0, 100.0, 100.0).as_tuple(),
            (254, 255, 255, 255)
        );
        assert_eq!(
            Color::from_hsla(92.0, 17.0, 100.0, 100.0).as_tuple(),
            (255, 254, 255, 255)
        );
    }

    /// Die erzeugte Namenstabelle - Umfang, Sortierung, Stichproben.
    ///
    /// Die Sortierung ist keine Formsache: [`tabelle`] sucht binaer, und
    /// eine unsortierte Tabelle faende die Haelfte der Namen nicht, ohne
    /// dass irgendetwas scheitert.
    #[test]
    fn die_farbnamen_sind_vollstaendig_und_sortiert() {
        assert_eq!(alle_namen().len(), 665);
        assert!(alle_namen().windows(2).all(|p| p[0].0 < p[1].0));

        // Sollwerte aus `pygame.colordict.THECOLORS`.
        assert_eq!(by_name("aliceblue"), Some(Color::rgb(240, 248, 255)));
        assert_eq!(by_name("aquamarine3"), Some(Color::rgb(102, 205, 170)));
        assert_eq!(by_name("yellowgreen"), Some(Color::rgb(154, 205, 50)));
        // Gross geschrieben und mit Leerzeichen - beides laesst pygame zu.
        assert_eq!(by_name("Alice Blue"), by_name("aliceblue"));
        assert_eq!(by_name("gibtsnicht"), None);
    }

    /// Die Hexschreibweise - und was pygame dabei **ablehnt**.
    ///
    /// Die drei letzten Zeilen stehen so in `color_test.py`
    /// (`test_invalid_html_hex_codes`); sie sind der Grund, warum
    /// [`hex_farbe`] nicht einfach `u8::from_str_radix` aufruft.
    #[test]
    fn hex_liest_beide_vorsilben_und_lehnt_den_rest_ab() {
        assert_eq!(parse("#CC00CC"), Some(Color::rgba(204, 0, 204, 255)));
        assert_eq!(parse("#CC00CC11"), Some(Color::rgba(204, 0, 204, 17)));
        assert_eq!(parse("0xCC00CC"), Some(Color::rgba(204, 0, 204, 255)));
        assert_eq!(parse("0x1a2B3c4D"), Some(Color::rgba(26, 43, 60, 77)));

        assert_eq!(parse("# f000000"), None);
        assert_eq!(parse("#f 000000"), None);
        assert_eq!(parse("#-f000000"), None);
        // Zu kurz, zu lang, und der Name, den `from_hex` nicht nimmt.
        assert_eq!(parse("#FF"), None);
        assert_eq!(parse("#FFFFFFFFF"), None);
        assert_eq!(Color::from_hex("red"), None);
    }
}
