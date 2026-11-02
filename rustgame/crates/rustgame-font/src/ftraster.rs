//! ftraster.rs
//! ===========
//! FreeTypes **Schwarzweiss-Rasterer**, uebertragen aus FreeType 2.11.1
//! (`src/raster/ftraster.c`, dazu `ftrend1.c` und der MONO-Zweig von
//! `ft_glyphslot_preset_bitmap` in `ftobjs.c`) - der Fassung, die in
//! pygame-ce 2.5.7 steckt.
//!
//! ## Warum ein eigener Rasterer
//!
//! `font.render(text, False, ..)` ist `TTF_RenderUTF8_Solid`, und SDL_ttf
//! rastert dafuer mit `FT_RENDER_MODE_MONO`. Das ist **nicht** die
//! Graustufenrechnung von [`crate::ftgrays`] mit einer Schwelle: ein Pixel
//! zaehlt, wenn sein **Mittelpunkt** zwischen zwei Kanten liegt, und wo
//! eine Kante duenner als ein Pixel ist, entscheidet die
//! "Dropout-Steuerung" nach den Regeln der TrueType-Spezifikation. Vorher
//! stand hier eine Schwelle bei der Haelfte der Deckung - gemessen traf das
//! **1 von 452** Zeichenbildern.
//!
//! ## Was uebertragen ist
//!
//! | C-Quelle | hier |
//! |---|---|
//! | `New_Profile`, `End_Profile`, `Insert_Y_Turn`, `Finalize_Profile_Table` | [`Arbeiter`] |
//! | `Split_Conic`, `Split_Cubic`, `Line_Up/Down`, `Bezier_Up/Down` | dto. |
//! | `Line_To`, `Conic_To`, `Cubic_To`, `Decompose_Curve`, `Convert_Glyph` | dto. |
//! | `InsNew`, `DelOld`, `Sort`, `Draw_Sweep` und die zwei Satz Sweep-Routinen | dto. |
//! | `Render_Single_Pass` (Unterbaender), `Render_Glyph`, `ft_black_render` | [`Arbeiter::rendern`] |
//! | `ft_raster1_render` + `ft_glyphslot_preset_bitmap` (MONO) | [`rendern`] |
//!
//! ## Der Speicher ist Teil des Verhaltens
//!
//! `ftraster.c` arbeitet in einem **festen Vorrat** von `FT_RENDER_POOL_SIZE
//! / sizeof(Long)` Eintraegen (16384 / 4 = **4096**: ein `long` hat unter
//! Windows vier Bytes). Profilkoepfe liegen mitten in diesem Vorrat
//! (`AlignProfileSize` = 13 Eintraege), die Wendepunkte am Ende. Laeuft er
//! ueber, rastert FreeType die Glyphe in **Unterbaendern** - und an den
//! Bandgrenzen kann die Dropout-Steuerung anders entscheiden. Darum wird
//! der Vorrat hier mitgezaehlt, Eintrag fuer Eintrag, und Profile werden
//! ueber ihre **Lage im Vorrat** angesprochen: `left->next == right`
//! vergleicht in C Zeiger, und ein Kopf, der leer blieb, wird vom
//! naechsten Profil wiederverwendet.

use std::collections::HashMap;

use crate::fixed::{mul_div, mul_div_no_round};
use crate::ftgrays::{Art, Umriss};

/// `MaxBezier` - wie tief ein Bogen geteilt werden kann.
const MAX_BEZIER: usize = 32;
/// `Pixel_Bits` - Nachkommabits der Eingabe (26.6).
const PIXEL_BITS: i32 = 6;

/// `Flow_Up`.
const FLOW_UP: u16 = 0x08;
/// `Overshoot_Top`.
const OVERSHOOT_TOP: u16 = 0x10;
/// `Overshoot_Bottom`.
const OVERSHOOT_BOTTOM: u16 = 0x20;

/// `FT_MAX_BLACK_POOL` unter Windows: 16384 Bytes zu je vier.
const VORRAT: isize = 4096;
/// `AlignProfileSize`: `(sizeof(TProfile) + sizeof(Alignment) - 1) /
/// sizeof(Long)` = (48 + 8 - 1) / 4 auf x64 mit 32-Bit-`long`.
const KOPF: isize = 13;

/// `FT_OUTLINE_IGNORE_DROPOUTS`.
pub(crate) const OUTLINE_IGNORE_DROPOUTS: u32 = 0x8;
/// `FT_OUTLINE_SMART_DROPOUTS`.
pub(crate) const OUTLINE_SMART_DROPOUTS: u32 = 0x10;
/// `FT_OUTLINE_INCLUDE_STUBS`.
pub(crate) const OUTLINE_INCLUDE_STUBS: u32 = 0x20;
/// `FT_OUTLINE_HIGH_PRECISION`.
pub(crate) const OUTLINE_HIGH_PRECISION: u32 = 0x100;
/// `FT_OUTLINE_SINGLE_PASS`.
pub(crate) const OUTLINE_SINGLE_PASS: u32 = 0x200;

/// Was der TrueType-Lader an den Umriss haengt und der Rasterer liest.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Rasterflaggen {
    /// `outline.flags` - nur die Bits oben zaehlen.
    pub(crate) flags: u32,
    /// `FT_CURVE_TAG_HAS_SCANMODE`: `(Punkt, Modus)`. `TT_Hint_Glyph`
    /// schreibt `GS.scan_type << 5` in die Marke des **ersten** Punktes
    /// jeder Glyphe mit eigenen Anweisungen - bei einer zusammengesetzten
    /// also an den Anfang jeder Komponente -, und `Decompose_Curve` liest
    /// sie am Anfang jeder Kontur als Dropout-Modus (`tags[0] >> 5`, drei
    /// Bit). **Die Marke schlaegt die Flaggen**: Arial meldet bei manchen
    /// Groessen `FT_OUTLINE_IGNORE_DROPOUTS` und rastert trotzdem mit
    /// Modus 1, gemessen an pygames `freetype.dll`.
    pub(crate) marken: Vec<(usize, u8)>,
}

/// Ein einfarbiges Glyphenbild: 0 oder 255 je Pixel, Zeile 0 oben.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Monobild {
    /// `bitmap_left`.
    pub(crate) left: i32,
    /// `bitmap_top`.
    pub(crate) top: i32,
    pub(crate) breite: usize,
    pub(crate) hoehe: usize,
    /// Ein Byte je Pixel, 0 oder 255.
    pub(crate) deckung: Vec<u8>,
}

/// `FT_Render_Glyph(slot, FT_RENDER_MODE_MONO)` ohne Ursprung.
///
/// Gibt `None`, wo FreeType einen Fehler meldet (Kasten zu gross,
/// ungueltiger Umriss) - SDL_ttf laesst die Glyphe dann weg.
pub(crate) fn rendern(u: &Umriss, fl: Rasterflaggen) -> Option<Monobild> {
    let (cx0, cy0, cx1, cy1) = u.kontrollkasten();
    let (xmin, ymin, xmax, ymax) = mono_kasten(cx0, cy0, cx1, cy1);
    if xmin < -0x8000 || xmax > 0x7FFF || ymin < -0x8000 || ymax > 0x7FFF {
        return None;
    }
    let breite = (xmax - xmin) as usize;
    let hoehe = (ymax - ymin) as usize;
    // `pitch = ((width + 15) >> 4) << 1` - auf zwei Bytes aufgerundet.
    let pitch = ((breite + 15) >> 4) << 1;
    let mut bild = Monobild {
        left: xmin as i32,
        top: ymax as i32,
        breite,
        hoehe,
        deckung: vec![0u8; breite * hoehe],
    };
    if u.punkte.is_empty() || u.enden.is_empty() || breite == 0 || hoehe == 0 {
        return Some(bild);
    }
    // `x_shift = -bitmap_left * 64`, `y_shift = (rows - bitmap_top) * 64`.
    let sx = -xmin * 64;
    let sy = (hoehe as i64 - ymax) * 64;
    let mut verschoben = u.clone();
    for p in &mut verschoben.punkte {
        p.0 += sx;
        p.1 += sy;
    }
    let mut bits = vec![0u8; pitch * hoehe];
    let mut w = Arbeiter::neu(&verschoben, fl.clone(), breite, hoehe, pitch);
    w.rendern(&mut bits).ok()?;
    for y in 0..hoehe {
        for x in 0..breite {
            if bits[y * pitch + (x >> 3)] & (0x80 >> (x & 7)) != 0 {
                bild.deckung[y * breite + x] = 255;
            }
        }
    }
    Some(bild)
}

/// Der MONO-Zweig von `ft_glyphslot_preset_bitmap`: Kasten
/// `(xmin, ymin, xmax, ymax)` in ganzen Pixeln.
///
/// "undocumented but confirmed: bbox values get rounded; we do asymmetric
/// rounding so that the center of a pixel gets always included" - und
/// faellt der Kasten zusammen, kommt ein Pixel auf die Seite, die mehr vom
/// Kontrollkasten deckt.
fn mono_kasten(cx0: i64, cy0: i64, cx1: i64, cy1: i64) -> (i64, i64, i64, i64) {
    let achse = |a: i64, b: i64| {
        let (mut lo, mut hi) = (a >> 6, b >> 6);
        let (ra, rb) = (a & 63, b & 63);
        lo += (ra + 31) >> 6;
        hi += (rb + 32) >> 6;
        if lo == hi {
            if ((ra + 31) & 63) - 31 + ((rb + 32) & 63) - 32 < 0 {
                lo -= 1;
            } else {
                hi += 1;
            }
        }
        (lo, hi)
    };
    let (x0, x1) = achse(cx0, cx1);
    let (y0, y1) = achse(cy0, cy1);
    (x0, y0, x1, y1)
}

/// `TStates`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Zustand {
    Unbekannt,
    Steigend,
    Fallend,
}

/// `TProfile`. Die Zeiger `link` und `next` sind **Lagen im Vorrat**.
#[derive(Debug, Clone, Copy, Default)]
struct Profil {
    x: i64,
    link: Option<isize>,
    offset: isize,
    flags: u16,
    height: i64,
    start: i64,
    count_l: i64,
    next: Option<isize>,
}

/// Welcher Satz Sweep-Routinen gerade gilt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Richtung {
    Senkrecht,
    Waagerecht,
}

/// Fehler wie `Raster_Err_*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fehler {
    Ungueltig,
    Ueberlauf,
    NegativeHoehe,
}

/// Ein Bezier-Punkt (`TPoint`).
type P = (i64, i64);

/// `black_TWorker`.
struct Arbeiter<'a> {
    umriss: &'a Umriss,
    flags: u32,
    marken: Vec<(usize, u8)>,

    precision_bits: i32,
    precision: i64,
    precision_half: i64,
    precision_scale: i64,
    precision_step: i64,
    precision_jitter: i64,

    /// Der Vorrat: Daten von vorn, Wendepunkte von hinten.
    buff: Vec<i64>,
    max_buff: isize,
    top: isize,
    fehler: Option<Fehler>,
    num_turns: isize,
    drop_out_control: u8,

    b_width: i64,
    b_rows: i64,
    pitch: isize,
    b_origin: isize,
    b_line: isize,

    last_x: i64,
    last_y: i64,
    min_y: i64,
    max_y: i64,

    num_profs: u16,
    fresh: bool,
    joint: bool,
    c_profile: isize,
    f_profile: Option<isize>,
    g_profile: Option<isize>,
    state: Zustand,

    /// Die Profilkoepfe, nach ihrer Lage im Vorrat.
    profile: HashMap<isize, Profil>,
    richtung: Richtung,
}

impl<'a> Arbeiter<'a> {
    fn neu(u: &'a Umriss, fl: Rasterflaggen, breite: usize, hoehe: usize, pitch: usize) -> Self {
        Arbeiter {
            umriss: u,
            flags: fl.flags,
            marken: fl.marken,
            precision_bits: 6,
            precision: 64,
            precision_half: 32,
            precision_scale: 1,
            precision_step: 32,
            precision_jitter: 2,
            buff: vec![0; VORRAT as usize],
            max_buff: 0,
            top: 0,
            fehler: None,
            num_turns: 0,
            drop_out_control: 0,
            b_width: breite as i64,
            b_rows: hoehe as i64,
            pitch: pitch as isize,
            b_origin: 0,
            b_line: 0,
            last_x: 0,
            last_y: 0,
            min_y: 0,
            max_y: 0,
            num_profs: 0,
            fresh: false,
            joint: false,
            c_profile: 0,
            f_profile: None,
            g_profile: None,
            state: Zustand::Unbekannt,
            profile: HashMap::new(),
            richtung: Richtung::Senkrecht,
        }
    }

    // ----- Rechenhilfen (`FLOOR`, `CEILING`, `TRUNC`, `FRAC`, `SCALED`) --

    #[inline]
    fn floor(&self, x: i64) -> i64 {
        x & -self.precision
    }
    #[inline]
    fn ceiling(&self, x: i64) -> i64 {
        (x + self.precision - 1) & -self.precision
    }
    #[inline]
    fn trunc(&self, x: i64) -> i64 {
        x >> self.precision_bits
    }
    #[inline]
    fn frac(&self, x: i64) -> i64 {
        x & (self.precision - 1)
    }
    #[inline]
    fn scaled(&self, x: i64) -> i64 {
        x * self.precision_scale - self.precision_half
    }
    #[inline]
    fn ist_unten_ueber(&self, x: i64) -> bool {
        self.ceiling(x) - x >= self.precision_half
    }
    #[inline]
    fn ist_oben_ueber(&self, x: i64) -> bool {
        x - self.floor(x) >= self.precision_half
    }
    /// `SMART`: welcher Pixel naeher an den Spannenenden liegt.
    #[inline]
    fn smart(&self, p: i64, q: i64) -> i64 {
        self.floor((p + q + self.precision * 63 / 64) >> 1)
    }

    fn p(&mut self, lage: isize) -> &mut Profil {
        self.profile.entry(lage).or_default()
    }
    fn pr(&self, lage: isize) -> Profil {
        self.profile.get(&lage).copied().unwrap_or_default()
    }

    /// `buff[i]` fuer Lesen und Schreiben; die Wendepunkte liegen bei
    /// `sizeBuff - n`.
    #[inline]
    fn turn(&self, n: isize) -> i64 {
        self.buff[(VORRAT - n) as usize]
    }

    // ----- Profile -------------------------------------------------------

    fn set_high_precision(&mut self, hoch: bool) {
        if hoch {
            self.precision_bits = 12;
            self.precision_step = 256;
            self.precision_jitter = 30;
        } else {
            self.precision_bits = 6;
            self.precision_step = 32;
            self.precision_jitter = 2;
        }
        self.precision = 1 << self.precision_bits;
        self.precision_half = self.precision >> 1;
        self.precision_scale = self.precision >> PIXEL_BITS;
    }

    fn new_profile(&mut self, zustand: Zustand, overshoot: bool) -> Result<(), Fehler> {
        if self.f_profile.is_none() {
            self.c_profile = self.top;
            self.f_profile = Some(self.c_profile);
            self.top += KOPF;
        }
        if self.top >= self.max_buff {
            return Err(Fehler::Ueberlauf);
        }
        let (top, doc) = (self.top, self.drop_out_control);
        let c = self.c_profile;
        {
            let p = self.p(c);
            p.start = 0;
            p.height = 0;
            p.offset = top;
            p.link = None;
            p.next = None;
            p.flags = u16::from(doc);
        }
        match zustand {
            Zustand::Steigend => {
                let p = self.p(c);
                p.flags |= FLOW_UP;
                if overshoot {
                    p.flags |= OVERSHOOT_BOTTOM;
                }
            }
            Zustand::Fallend => {
                if overshoot {
                    self.p(c).flags |= OVERSHOOT_TOP;
                }
            }
            Zustand::Unbekannt => return Err(Fehler::Ungueltig),
        }
        if self.g_profile.is_none() {
            self.g_profile = Some(c);
        }
        self.state = zustand;
        self.fresh = true;
        self.joint = false;
        Ok(())
    }

    fn end_profile(&mut self, overshoot: bool) -> Result<(), Fehler> {
        let c = self.c_profile;
        let h = (self.top - self.pr(c).offset) as i64;
        if h < 0 {
            return Err(Fehler::NegativeHoehe);
        }
        if h > 0 {
            {
                let p = self.p(c);
                p.height = h;
                if overshoot {
                    if p.flags & FLOW_UP != 0 {
                        p.flags |= OVERSHOOT_TOP;
                    } else {
                        p.flags |= OVERSHOOT_BOTTOM;
                    }
                }
            }
            let alt = c;
            self.c_profile = self.top;
            self.top += KOPF;
            let (neu, top) = (self.c_profile, self.top);
            {
                let p = self.p(neu);
                p.height = 0;
                p.offset = top;
            }
            self.p(alt).next = Some(neu);
            self.num_profs += 1;
        }
        if self.top >= self.max_buff {
            return Err(Fehler::Ueberlauf);
        }
        self.joint = false;
        Ok(())
    }

    fn insert_y_turn(&mut self, mut y: i64) -> Result<(), Fehler> {
        let mut n = self.num_turns - 1;
        // `y_turns = sizeBuff - numTurns`, also `y_turns[n]` bei
        // `VORRAT - numTurns + n`.
        let basis = VORRAT - self.num_turns;
        while n >= 0 && y < self.buff[(basis + n) as usize] {
            n -= 1;
        }
        if n >= 0 && y > self.buff[(basis + n) as usize] {
            loop {
                let i = (basis + n) as usize;
                std::mem::swap(&mut self.buff[i], &mut y);
                n -= 1;
                if n < 0 {
                    break;
                }
            }
        }
        if n < 0 {
            self.max_buff -= 1;
            if self.max_buff <= self.top {
                return Err(Fehler::Ueberlauf);
            }
            self.num_turns += 1;
            self.buff[(VORRAT - self.num_turns) as usize] = y;
        }
        Ok(())
    }

    fn finalize_profile_table(&mut self) -> Result<(), Fehler> {
        let mut n = self.num_profs;
        let mut lage = self.f_profile;
        if n > 1 && lage.is_some() {
            loop {
                let l = lage.expect("Profilkette");
                let (bottom, top);
                {
                    let mehr = n > 1;
                    let p = self.p(l);
                    p.link = if mehr { Some(p.offset + p.height as isize) } else { None };
                    if p.flags & FLOW_UP != 0 {
                        bottom = p.start;
                        top = p.start + p.height - 1;
                    } else {
                        bottom = p.start - p.height + 1;
                        top = p.start;
                        p.start = bottom;
                        p.offset += p.height as isize - 1;
                    }
                }
                self.insert_y_turn(bottom)?;
                self.insert_y_turn(top + 1)?;
                lage = self.pr(l).link;
                n -= 1;
                if n == 0 {
                    break;
                }
            }
        } else {
            self.f_profile = None;
        }
        Ok(())
    }

    // ----- Boegen -------------------------------------------------------

    fn split_conic(b: &mut [P], i: usize) {
        b[i + 4].0 = b[i + 2].0;
        let a = b[i].0 + b[i + 1].0;
        let c = b[i + 1].0 + b[i + 2].0;
        b[i + 3].0 = c >> 1;
        b[i + 2].0 = (a + c) >> 2;
        b[i + 1].0 = a >> 1;

        b[i + 4].1 = b[i + 2].1;
        let a = b[i].1 + b[i + 1].1;
        let c = b[i + 1].1 + b[i + 2].1;
        b[i + 3].1 = c >> 1;
        b[i + 2].1 = (a + c) >> 2;
        b[i + 1].1 = a >> 1;
    }

    fn split_cubic(b: &mut [P], i: usize) {
        b[i + 6].0 = b[i + 3].0;
        let mut a = b[i].0 + b[i + 1].0;
        let bb = b[i + 1].0 + b[i + 2].0;
        let mut c = b[i + 2].0 + b[i + 3].0;
        b[i + 5].0 = c >> 1;
        c += bb;
        b[i + 4].0 = c >> 2;
        b[i + 1].0 = a >> 1;
        a += bb;
        b[i + 2].0 = a >> 2;
        b[i + 3].0 = (a + c) >> 3;

        b[i + 6].1 = b[i + 3].1;
        let mut a = b[i].1 + b[i + 1].1;
        let bb = b[i + 1].1 + b[i + 2].1;
        let mut c = b[i + 2].1 + b[i + 3].1;
        b[i + 5].1 = c >> 1;
        c += bb;
        b[i + 4].1 = c >> 2;
        b[i + 1].1 = a >> 1;
        a += bb;
        b[i + 2].1 = a >> 2;
        b[i + 3].1 = (a + c) >> 3;
    }

    fn line_up(
        &mut self,
        mut x1: i64,
        y1: i64,
        x2: i64,
        y2: i64,
        miny: i64,
        maxy: i64,
    ) -> Result<(), Fehler> {
        let mut dx = x2 - x1;
        let dy = y2 - y1;
        if dy <= 0 || y2 < miny || y1 > maxy {
            return Ok(());
        }
        let (mut e1, f1);
        if y1 < miny {
            x1 += mul_div(dx, miny - y1, dy);
            e1 = self.trunc(miny);
            f1 = 0;
        } else {
            e1 = self.trunc(y1);
            f1 = self.frac(y1);
        }
        let (e2, f2);
        if y2 > maxy {
            e2 = self.trunc(maxy);
            f2 = 0;
        } else {
            e2 = self.trunc(y2);
            f2 = self.frac(y2);
        }
        if f1 > 0 {
            if e1 == e2 {
                return Ok(());
            }
            x1 += mul_div(dx, self.precision - f1, dy);
            e1 += 1;
        } else if self.joint {
            self.top -= 1;
            self.joint = false;
        }
        self.joint = f2 == 0;
        if self.fresh {
            let c = self.c_profile;
            self.p(c).start = e1;
            self.fresh = false;
        }
        let mut size = e2 - e1 + 1;
        if self.top + size as isize >= self.max_buff {
            return Err(Fehler::Ueberlauf);
        }
        // `( ras.precision * Dx ) % Dy` rechnet in C mit `long` - unter
        // Windows 32 Bit.
        let (ix, rx);
        if dx > 0 {
            ix = mul_div_no_round(self.precision, dx, dy);
            rx = ((self.precision * dx) as i32 as i64) % dy;
            dx = 1;
        } else {
            ix = -mul_div_no_round(self.precision, -dx, dy);
            rx = ((self.precision * -dx) as i32 as i64) % dy;
            dx = -1;
        }
        let mut ax = -dy;
        let mut top = self.top;
        while size > 0 {
            self.buff[top as usize] = x1;
            top += 1;
            x1 += ix;
            ax += rx;
            if ax >= 0 {
                ax -= dy;
                x1 += dx;
            }
            size -= 1;
        }
        self.top = top;
        Ok(())
    }

    fn line_down(
        &mut self,
        x1: i64,
        y1: i64,
        x2: i64,
        y2: i64,
        miny: i64,
        maxy: i64,
    ) -> Result<(), Fehler> {
        let fresh = self.fresh;
        let r = self.line_up(x1, -y1, x2, -y2, -maxy, -miny);
        if fresh && !self.fresh {
            let c = self.c_profile;
            let p = self.p(c);
            p.start = -p.start;
        }
        r
    }

    /// `Bezier_Up`. `arc` ist die Lage im Bogenstapel `b`.
    fn bezier_up(
        &mut self,
        grad: usize,
        b: &mut [P],
        mut arc: isize,
        miny: i64,
        maxy: i64,
    ) -> Result<(), Fehler> {
        let g = grad as isize;
        let mut y1 = b[(arc + g) as usize].1;
        let mut y2 = b[arc as usize].1;
        let mut top = self.top;

        if y2 < miny || y1 > maxy {
            self.top = top;
            return Ok(());
        }
        let mut e2 = self.floor(y2);
        if e2 > maxy {
            e2 = maxy;
        }
        let mut e0 = miny;
        let mut e;
        if y1 < miny {
            e = miny;
        } else {
            e = self.ceiling(y1);
            let f1 = self.frac(y1);
            e0 = e;
            if f1 == 0 {
                if self.joint {
                    top -= 1;
                    self.joint = false;
                }
                self.buff[top as usize] = b[(arc + g) as usize].0;
                top += 1;
                e += self.precision;
            }
        }
        if self.fresh {
            let (c, s) = (self.c_profile, self.trunc(e0));
            self.p(c).start = s;
            self.fresh = false;
        }
        if e2 < e {
            self.top = top;
            return Ok(());
        }
        if top + self.trunc(e2 - e) as isize + 1 >= self.max_buff {
            self.top = top;
            return Err(Fehler::Ueberlauf);
        }
        let start_arc = arc;
        loop {
            self.joint = false;
            y2 = b[arc as usize].1;
            if y2 > e {
                y1 = b[(arc + g) as usize].1;
                if y2 - y1 >= self.precision_step {
                    if grad == 2 {
                        Self::split_conic(b, arc as usize);
                    } else {
                        Self::split_cubic(b, arc as usize);
                    }
                    arc += g;
                } else {
                    let a = b[(arc + g) as usize].0;
                    // `FMulDiv` ist eine gewoehnliche Ganzzahldivision.
                    self.buff[top as usize] = a + (b[arc as usize].0 - a) * (e - y1) / (y2 - y1);
                    top += 1;
                    arc -= g;
                    e += self.precision;
                }
            } else {
                if y2 == e {
                    self.joint = true;
                    self.buff[top as usize] = b[arc as usize].0;
                    top += 1;
                    e += self.precision;
                }
                arc -= g;
            }
            if !(arc >= start_arc && e <= e2) {
                break;
            }
        }
        self.top = top;
        Ok(())
    }

    fn bezier_down(
        &mut self,
        grad: usize,
        b: &mut [P],
        arc: isize,
        miny: i64,
        maxy: i64,
    ) -> Result<(), Fehler> {
        let a = arc as usize;
        b[a].1 = -b[a].1;
        b[a + 1].1 = -b[a + 1].1;
        b[a + 2].1 = -b[a + 2].1;
        if grad > 2 {
            b[a + 3].1 = -b[a + 3].1;
        }
        let fresh = self.fresh;
        let r = self.bezier_up(grad, b, arc, -maxy, -miny);
        if fresh && !self.fresh {
            let c = self.c_profile;
            let p = self.p(c);
            p.start = -p.start;
        }
        b[a].1 = -b[a].1;
        r
    }

    fn line_to(&mut self, x: i64, y: i64) -> Result<(), Fehler> {
        match self.state {
            Zustand::Unbekannt => {
                if y > self.last_y {
                    let o = self.ist_unten_ueber(self.last_y);
                    self.new_profile(Zustand::Steigend, o)?;
                } else if y < self.last_y {
                    let o = self.ist_oben_ueber(self.last_y);
                    self.new_profile(Zustand::Fallend, o)?;
                }
            }
            Zustand::Steigend => {
                if y < self.last_y {
                    let o = self.ist_oben_ueber(self.last_y);
                    self.end_profile(o)?;
                    self.new_profile(Zustand::Fallend, o)?;
                }
            }
            Zustand::Fallend => {
                if y > self.last_y {
                    let o = self.ist_unten_ueber(self.last_y);
                    self.end_profile(o)?;
                    self.new_profile(Zustand::Steigend, o)?;
                }
            }
        }
        match self.state {
            Zustand::Steigend => {
                self.line_up(self.last_x, self.last_y, x, y, self.min_y, self.max_y)?
            }
            Zustand::Fallend => {
                self.line_down(self.last_x, self.last_y, x, y, self.min_y, self.max_y)?
            }
            Zustand::Unbekannt => {}
        }
        self.last_x = x;
        self.last_y = y;
        Ok(())
    }

    /// Gemeinsamer Teil von `Conic_To` und `Cubic_To`: einen
    /// y-monotonen Bogen einordnen und rastern.
    fn bogen_einordnen(
        &mut self,
        grad: usize,
        b: &mut [P],
        arc: isize,
        y1: i64,
        steigend: bool,
    ) -> Result<(), Fehler> {
        let z = if steigend { Zustand::Steigend } else { Zustand::Fallend };
        if self.state != z {
            let o = if steigend {
                self.ist_unten_ueber(y1)
            } else {
                self.ist_oben_ueber(y1)
            };
            if self.state != Zustand::Unbekannt {
                self.end_profile(o)?;
            }
            self.new_profile(z, o)?;
        }
        let (miny, maxy) = (self.min_y, self.max_y);
        if steigend {
            self.bezier_up(grad, b, arc, miny, maxy)
        } else {
            self.bezier_down(grad, b, arc, miny, maxy)
        }
    }

    fn conic_to(&mut self, cx: i64, cy: i64, x: i64, y: i64) -> Result<(), Fehler> {
        // Platz fuer `2 * MaxBezier + 1` Punkte, dazu Luft fuer den
        // letzten Teilungsschritt.
        let mut b = vec![(0i64, 0i64); 2 * MAX_BEZIER + 1 + 4];
        let mut arc: isize = 0;
        b[2] = (self.last_x, self.last_y);
        b[1] = (cx, cy);
        b[0] = (x, y);
        let (mut x3, mut y3);
        loop {
            let a = arc as usize;
            let y1 = b[a + 2].1;
            let y2 = b[a + 1].1;
            y3 = b[a].1;
            x3 = b[a].0;
            let (ymin, ymax) = if y1 <= y3 { (y1, y3) } else { (y3, y1) };
            if y2 < ymin || y2 > ymax {
                if a + 4 >= b.len() {
                    return Err(Fehler::Ungueltig);
                }
                Self::split_conic(&mut b, a);
                arc += 2;
            } else if y1 == y3 {
                arc -= 2;
            } else {
                self.bogen_einordnen(2, &mut b, arc, y1, y1 < y3)?;
                arc -= 2;
            }
            if arc < 0 {
                break;
            }
        }
        self.last_x = x3;
        self.last_y = y3;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn cubic_to(
        &mut self,
        cx1: i64,
        cy1: i64,
        cx2: i64,
        cy2: i64,
        x: i64,
        y: i64,
    ) -> Result<(), Fehler> {
        let mut b = vec![(0i64, 0i64); 3 * MAX_BEZIER + 1 + 6];
        let mut arc: isize = 0;
        b[3] = (self.last_x, self.last_y);
        b[2] = (cx1, cy1);
        b[1] = (cx2, cy2);
        b[0] = (x, y);
        let (mut x4, mut y4);
        loop {
            let a = arc as usize;
            let y1 = b[a + 3].1;
            let y2 = b[a + 2].1;
            let y3 = b[a + 1].1;
            y4 = b[a].1;
            x4 = b[a].0;
            let (ymin1, ymax1) = if y1 <= y4 { (y1, y4) } else { (y4, y1) };
            let (ymin2, ymax2) = if y2 <= y3 { (y2, y3) } else { (y3, y2) };
            if ymin2 < ymin1 || ymax2 > ymax1 {
                if a + 6 >= b.len() {
                    return Err(Fehler::Ungueltig);
                }
                Self::split_cubic(&mut b, a);
                arc += 3;
            } else if y1 == y4 {
                arc -= 3;
            } else {
                self.bogen_einordnen(3, &mut b, arc, y1, y1 <= y4)?;
                arc -= 3;
            }
            if arc < 0 {
                break;
            }
        }
        self.last_x = x4;
        self.last_y = y4;
        Ok(())
    }

    /// `Decompose_Curve` fuer die Punkte `first..=last`.
    fn decompose_curve(&mut self, first: usize, last: usize, flipped: bool) -> Result<(), Fehler> {
        let u = self.umriss;
        let pt = |s: &Self, i: usize| {
            let (x, y) = (s.scaled(u.punkte[i].0), s.scaled(u.punkte[i].1));
            if flipped {
                (y, x)
            } else {
                (x, y)
            }
        };
        let mut v_start = pt(self, first);
        let v_last = pt(self, last);
        let mut v_control;

        // "set scan mode if necessary"
        if let Some(&(_, m)) = self.marken.iter().find(|(i, _)| *i == first) {
            self.drop_out_control = m;
        }

        let mut point = first as isize;
        let mut limit = last as isize;
        let tag = u.arten[first];
        if tag == Art::Kubisch {
            return Err(Fehler::Ungueltig);
        }
        if tag == Art::Quadratisch {
            if u.arten[last] == Art::Auf {
                v_start = v_last;
                limit -= 1;
            } else {
                // C teilt `long` mit `/ 2` - gegen null.
                v_start.0 = (v_start.0 + v_last.0) / 2;
                v_start.1 = (v_start.1 + v_last.1) / 2;
            }
            point -= 1;
        }
        self.last_x = v_start.0;
        self.last_y = v_start.1;

        while point < limit {
            point += 1;
            let tag = u.arten[point as usize];
            match tag {
                Art::Auf => {
                    let (x, y) = pt(self, point as usize);
                    self.line_to(x, y)?;
                    continue;
                }
                Art::Quadratisch => {
                    v_control = pt(self, point as usize);
                    loop {
                        // `Do_Conic`
                        if point < limit {
                            point += 1;
                            let tag = u.arten[point as usize];
                            let (x, y) = pt(self, point as usize);
                            if tag == Art::Auf {
                                self.conic_to(v_control.0, v_control.1, x, y)?;
                                break;
                            }
                            if tag != Art::Quadratisch {
                                return Err(Fehler::Ungueltig);
                            }
                            let mitte = ((v_control.0 + x) / 2, (v_control.1 + y) / 2);
                            self.conic_to(v_control.0, v_control.1, mitte.0, mitte.1)?;
                            v_control = (x, y);
                            continue;
                        }
                        self.conic_to(v_control.0, v_control.1, v_start.0, v_start.1)?;
                        return Ok(());
                    }
                }
                Art::Kubisch => {
                    if point + 1 > limit || u.arten[(point + 1) as usize] != Art::Kubisch {
                        return Err(Fehler::Ungueltig);
                    }
                    point += 2;
                    let (x1, y1) = pt(self, (point - 2) as usize);
                    let (x2, y2) = pt(self, (point - 1) as usize);
                    if point <= limit {
                        let (x3, y3) = pt(self, point as usize);
                        self.cubic_to(x1, y1, x2, y2, x3, y3)?;
                        continue;
                    }
                    self.cubic_to(x1, y1, x2, y2, v_start.0, v_start.1)?;
                    return Ok(());
                }
            }
        }
        self.line_to(v_start.0, v_start.1)
    }

    /// `Convert_Glyph`. `Ok(false)` ist das `FAILURE` am Schluss ohne
    /// gesetzten Fehler.
    fn convert_glyph(&mut self, flipped: bool) -> Result<bool, Fehler> {
        self.f_profile = None;
        self.joint = false;
        self.fresh = false;
        self.max_buff = VORRAT - KOPF;
        self.num_turns = 0;
        self.profile.clear();
        self.c_profile = self.top;
        let top = self.top;
        self.p(top).offset = top;
        self.num_profs = 0;

        let mut start = 0usize;
        let enden = self.umriss.enden.clone();
        for &ende in &enden {
            self.state = Zustand::Unbekannt;
            self.g_profile = None;
            self.decompose_curve(start, ende, flipped)?;
            start = ende + 1;

            if self.frac(self.last_y) == 0 && self.last_y >= self.min_y && self.last_y <= self.max_y {
                if let Some(g) = self.g_profile {
                    let c = self.c_profile;
                    if self.pr(g).flags & FLOW_UP == self.pr(c).flags & FLOW_UP {
                        self.top -= 1;
                    }
                }
            }
            let last_profile = self.c_profile;
            let cp = self.pr(last_profile);
            let o = if self.top != cp.offset && cp.flags & FLOW_UP != 0 {
                self.ist_oben_ueber(self.last_y)
            } else {
                self.ist_unten_ueber(self.last_y)
            };
            self.end_profile(o)?;
            if let Some(g) = self.g_profile {
                self.p(last_profile).next = Some(g);
            }
        }
        self.finalize_profile_table()?;
        Ok(self.top < self.max_buff)
    }

    // ----- Listen ---------------------------------------------------------

    fn ins_new(&mut self, liste: &mut Option<isize>, profil: isize) {
        let x = self.pr(profil).x;
        // `old` ist entweder der Listenkopf (None) oder das `link` eines
        // Profils.
        let mut old: Option<isize> = None;
        let mut current = *liste;
        while let Some(c) = current {
            if x < self.pr(c).x {
                break;
            }
            old = Some(c);
            current = self.pr(c).link;
        }
        self.p(profil).link = current;
        match old {
            None => *liste = Some(profil),
            Some(o) => self.p(o).link = Some(profil),
        }
    }

    fn del_old(&mut self, liste: &mut Option<isize>, profil: isize) {
        let mut old: Option<isize> = None;
        let mut current = *liste;
        while let Some(c) = current {
            if c == profil {
                let l = self.pr(c).link;
                match old {
                    None => *liste = l,
                    Some(o) => self.p(o).link = l,
                }
                return;
            }
            old = Some(c);
            current = self.pr(c).link;
        }
    }

    fn sort(&mut self, liste: &mut Option<isize>) {
        let mut current = *liste;
        while let Some(c) = current {
            let off = self.pr(c).offset;
            let wert = self.buff[off as usize];
            let p = self.p(c);
            p.x = wert;
            p.offset += if p.flags & FLOW_UP != 0 { 1 } else { -1 };
            p.height -= 1;
            current = p.link;
        }

        let mut old: Option<isize> = None;
        let Some(mut current) = *liste else {
            return;
        };
        let mut next = self.pr(current).link;
        while let Some(n) = next {
            if self.pr(current).x <= self.pr(n).x {
                old = Some(current);
                match self.pr(current).link {
                    Some(c) => current = c,
                    None => return,
                }
            } else {
                // `*old = next; current->link = next->link; next->link = current;`
                match old {
                    None => *liste = Some(n),
                    Some(o) => self.p(o).link = Some(n),
                }
                let nl = self.pr(n).link;
                self.p(current).link = nl;
                self.p(n).link = Some(current);
                old = None;
                current = liste.expect("Liste");
            }
            next = self.pr(current).link;
        }
    }

    // ----- Sweeps ---------------------------------------------------------

    fn sweep_init(&mut self, min: i64) {
        if self.richtung == Richtung::Senkrecht {
            self.b_line = self.b_origin - min as isize * self.pitch;
        }
    }

    fn sweep_step(&mut self) {
        if self.richtung == Richtung::Senkrecht {
            self.b_line -= self.pitch;
        }
    }

    fn sweep_span(&mut self, bits: &mut [u8], y: i64, x1: i64, x2: i64, left: isize) {
        match self.richtung {
            Richtung::Senkrecht => {
                let doc = self.pr(left).flags & 7;
                let mut e1 = self.ceiling(x1);
                let mut e2 = self.floor(x2);
                if doc != 2 && x2 - x1 - self.precision <= self.precision_jitter && e1 != x1 && e2 != x2 {
                    e2 = e1;
                }
                e1 = self.trunc(e1);
                e2 = self.trunc(e2);
                if e2 >= 0 && e1 < self.b_width {
                    if e1 < 0 {
                        e1 = 0;
                    }
                    if e2 >= self.b_width {
                        e2 = self.b_width - 1;
                    }
                    let c1 = (e1 >> 3) as isize;
                    let mut c2 = (e2 >> 3) as isize;
                    let f1 = 0xFFu8 >> (e1 & 7);
                    let f2 = !(0x7Fu8 >> (e2 & 7));
                    let mut t = self.b_line + c1;
                    c2 -= c1;
                    if c2 > 0 {
                        bits[t as usize] |= f1;
                        loop {
                            c2 -= 1;
                            if c2 <= 0 {
                                break;
                            }
                            t += 1;
                            bits[t as usize] = 0xFF;
                        }
                        bits[(t + 1) as usize] |= f2;
                    } else {
                        bits[t as usize] |= f1 & f2;
                    }
                }
            }
            Richtung::Waagerecht => {
                let e1 = self.ceiling(x1);
                if x1 == e1 {
                    let e1 = self.trunc(e1);
                    if e1 >= 0 && e1 < self.b_rows {
                        let i = self.b_origin + (y >> 3) as isize - e1 as isize * self.pitch;
                        bits[i as usize] |= 0x80u8 >> (y & 7);
                    }
                }
                let e2 = self.floor(x2);
                if x2 == e2 {
                    let e2 = self.trunc(e2);
                    if e2 >= 0 && e2 < self.b_rows {
                        let i = self.b_origin + (y >> 3) as isize - e2 as isize * self.pitch;
                        bits[i as usize] |= 0x80u8 >> (y & 7);
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn sweep_drop(&mut self, bits: &mut [u8], y: i64, x1: i64, x2: i64, left: isize, right: isize) {
        let mut e1 = self.ceiling(x1);
        let e2 = self.floor(x2);
        let mut pxl = e1;
        let senkrecht = self.richtung == Richtung::Senkrecht;
        // Die Grenze der Zielachse: Breite beim senkrechten, Zeilenzahl
        // beim waagerechten Durchgang.
        let grenze = if senkrecht { self.b_width } else { self.b_rows };
        let gesetzt = |s: &Self, bits: &[u8], e: i64| -> bool {
            if senkrecht {
                let c1 = (e >> 3) as isize;
                let f1 = e & 7;
                e >= 0 && e < s.b_width && bits[(s.b_line + c1) as usize] & (0x80u8 >> f1) != 0
            } else {
                if !(e >= 0 && e < s.b_rows) {
                    return false;
                }
                let i = s.b_origin + (y >> 3) as isize - e as isize * s.pitch;
                bits[i as usize] & (0x80u8 >> (y & 7)) != 0
            }
        };

        if e1 > e2 {
            let lp = self.pr(left);
            let rp = self.pr(right);
            let doc = lp.flags & 7;
            if e1 == e2 + self.precision {
                match doc {
                    0 => pxl = e2,
                    4 => pxl = self.smart(x1, x2),
                    1 | 5 => {
                        // obere Spitze
                        if lp.next == Some(right)
                            && lp.height <= 0
                            && !(lp.flags & OVERSHOOT_TOP != 0 && x2 - x1 >= self.precision_half)
                        {
                            return;
                        }
                        // untere Spitze
                        if rp.next == Some(left)
                            && lp.start == y
                            && !(lp.flags & OVERSHOOT_BOTTOM != 0 && x2 - x1 >= self.precision_half)
                        {
                            return;
                        }
                        pxl = if doc == 1 { e2 } else { self.smart(x1, x2) };
                    }
                    _ => return,
                }
                // "If the drop-out would result in a pixel outside of the
                // bounding box, use the pixel inside of the bounding box"
                if pxl < 0 {
                    pxl = e1;
                } else if self.trunc(pxl) >= grenze {
                    pxl = e2;
                }
                // pruefen, dass das andere Pixel nicht schon gesetzt ist
                e1 = if pxl == e1 { e2 } else { e1 };
                e1 = self.trunc(e1);
                if gesetzt(self, bits, e1) {
                    return;
                }
            } else {
                return;
            }
        }

        let e1 = self.trunc(pxl);
        if senkrecht {
            if e1 >= 0 && e1 < self.b_width {
                let c1 = (e1 >> 3) as isize;
                bits[(self.b_line + c1) as usize] |= 0x80u8 >> (e1 & 7);
            }
        } else if e1 >= 0 && e1 < self.b_rows {
            let i = self.b_origin + (y >> 3) as isize - e1 as isize * self.pitch;
            bits[i as usize] |= 0x80u8 >> (y & 7);
        }
    }

    fn draw_sweep(&mut self, bits: &mut [u8]) -> Result<(), Fehler> {
        let mut waiting: Option<isize> = None;
        let mut draw_left: Option<isize> = None;
        let mut draw_right: Option<isize> = None;

        let mut p = self.f_profile;
        let mut max_y = self.trunc(self.min_y);
        let mut min_y = self.trunc(self.max_y);
        while let Some(l) = p {
            let q = self.pr(l).link;
            let pr = self.pr(l);
            let bottom = pr.start;
            let top = pr.start + pr.height - 1;
            if min_y > bottom {
                min_y = bottom;
            }
            if max_y < top {
                max_y = top;
            }
            self.p(l).x = 0;
            self.ins_new(&mut waiting, l);
            p = q;
        }
        if self.num_turns == 0 {
            return Err(Fehler::Ungueltig);
        }
        self.sweep_init(min_y);

        let mut p = waiting;
        while let Some(l) = p {
            let pr = self.p(l);
            pr.count_l = pr.start - min_y;
            p = pr.link;
        }

        let mut y = min_y;
        let mut y_height: i64 = 0;
        if self.num_turns > 0 && self.turn(self.num_turns) == min_y {
            self.num_turns -= 1;
        }
        while self.num_turns > 0 {
            let mut p = waiting;
            while let Some(l) = p {
                let q = self.pr(l).link;
                let c = {
                    let pr = self.p(l);
                    pr.count_l -= y_height;
                    pr.count_l
                };
                if c == 0 {
                    self.del_old(&mut waiting, l);
                    if self.pr(l).flags & FLOW_UP != 0 {
                        self.ins_new(&mut draw_left, l);
                    } else {
                        self.ins_new(&mut draw_right, l);
                    }
                }
                p = q;
            }

            self.sort(&mut draw_left);
            self.sort(&mut draw_right);

            let y_change = self.turn(self.num_turns);
            self.num_turns -= 1;
            y_height = y_change - y;

            while y < y_change {
                let mut dropouts = 0;
                let mut pl = draw_left;
                let mut pr = draw_right;
                while let (Some(l), Some(r)) = (pl, pr) {
                    let mut x1 = self.pr(l).x;
                    let mut x2 = self.pr(r).x;
                    if x1 > x2 {
                        std::mem::swap(&mut x1, &mut x2);
                    }
                    let e1 = self.floor(x1);
                    let e2 = self.ceiling(x2);
                    let mut gemalt = true;
                    if x2 - x1 <= self.precision && e1 != x1 && e2 != x2 && (e1 > e2 || e2 == e1 + self.precision) {
                        let doc = self.pr(l).flags & 7;
                        if doc != 2 {
                            self.p(l).x = x1;
                            self.p(r).x = x2;
                            self.p(l).count_l = 1;
                            dropouts += 1;
                        }
                        gemalt = false;
                    }
                    if gemalt {
                        self.sweep_span(bits, y, x1, x2, l);
                    }
                    pl = self.pr(l).link;
                    pr = self.pr(r).link;
                }

                if dropouts > 0 {
                    // `Scan_DropOuts`
                    let mut pl = draw_left;
                    let mut pr = draw_right;
                    while let (Some(l), Some(r)) = (pl, pr) {
                        if self.pr(l).count_l != 0 {
                            self.p(l).count_l = 0;
                            let (xl, xr) = (self.pr(l).x, self.pr(r).x);
                            self.sweep_drop(bits, y, xl, xr, l, r);
                        }
                        pl = self.pr(l).link;
                        pr = self.pr(r).link;
                    }
                }

                // `Next_Line`
                self.sweep_step();
                y += 1;
                if y < y_change {
                    self.sort(&mut draw_left);
                    self.sort(&mut draw_right);
                }
            }

            let mut p = draw_left;
            while let Some(l) = p {
                let q = self.pr(l).link;
                if self.pr(l).height == 0 {
                    self.del_old(&mut draw_left, l);
                }
                p = q;
            }
            let mut p = draw_right;
            while let Some(l) = p {
                let q = self.pr(l).link;
                if self.pr(l).height == 0 {
                    self.del_old(&mut draw_right, l);
                }
                p = q;
            }
        }
        while y <= max_y {
            self.sweep_step();
            y += 1;
        }
        Ok(())
    }

    /// `Render_Single_Pass` mit Unterbaendern.
    fn single_pass(&mut self, bits: &mut [u8], flipped: bool, mut y_min: i64, mut y_max: i64) -> Result<(), Fehler> {
        let mut stapel: Vec<i64> = Vec::with_capacity(32);
        loop {
            self.min_y = y_min * self.precision;
            self.max_y = y_max * self.precision;
            self.top = 0;
            self.fehler = None;
            match self.convert_glyph(flipped) {
                Err(Fehler::Ueberlauf) => {
                    if y_min == y_max {
                        return Err(Fehler::Ueberlauf);
                    }
                    let y_mid = (y_min + y_max) >> 1;
                    stapel.push(y_min);
                    y_min = y_mid + 1;
                    continue;
                }
                Err(e) => return Err(e),
                // `FAILURE` ohne Fehler: `ras.error` ist `Raster_Err_Ok`,
                // also kehrt `Render_Single_Pass` mit "kein Fehler" zurueck.
                Ok(false) => return Ok(()),
                Ok(true) => {}
            }
            if self.f_profile.is_some() {
                self.draw_sweep(bits)?;
            }
            match stapel.pop() {
                None => break,
                Some(unten) => {
                    y_max = y_min - 1;
                    y_min = unten;
                }
            }
        }
        Ok(())
    }

    /// `Render_Glyph`.
    fn rendern(&mut self, bits: &mut [u8]) -> Result<(), Fehler> {
        self.set_high_precision(self.flags & OUTLINE_HIGH_PRECISION != 0);
        if self.flags & OUTLINE_IGNORE_DROPOUTS != 0 {
            self.drop_out_control = 2;
        } else {
            self.drop_out_control = if self.flags & OUTLINE_SMART_DROPOUTS != 0 { 4 } else { 0 };
            if self.flags & OUTLINE_INCLUDE_STUBS == 0 {
                self.drop_out_control += 1;
            }
        }
        // Senkrecht
        self.richtung = Richtung::Senkrecht;
        self.b_origin = (self.b_rows as isize - 1) * self.pitch;
        self.single_pass(bits, false, 0, self.b_rows - 1)?;
        // Waagerecht
        if self.flags & OUTLINE_SINGLE_PASS == 0 {
            self.richtung = Richtung::Waagerecht;
            self.single_pass(bits, true, 0, self.b_width - 1)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quadrat(x0: i64, y0: i64, x1: i64, y1: i64) -> Umriss {
        Umriss {
            punkte: vec![(x0, y0), (x0, y1), (x1, y1), (x1, y0)],
            arten: vec![Art::Auf; 4],
            enden: vec![3],
            ueberlappt: false,
        }
    }

    #[test]
    fn kasten_rundet_asymmetrisch() {
        // (10, 20)-(70, 100): links (10+31)>>6 = 0, rechts 1 + (6+32)>>6 = 1.
        assert_eq!(mono_kasten(10, 20, 70, 100), (0, 0, 1, 2));
        // Ein Kasten, der zusammenfaellt, waechst zur Seite mit mehr Deckung.
        assert_eq!(mono_kasten(40, 0, 50, 64), (0, 0, 1, 1));
    }

    #[test]
    fn ganzes_quadrat_ist_voll() {
        let b = rendern(&quadrat(0, 0, 128, 128), Rasterflaggen::default()).unwrap();
        assert_eq!((b.left, b.top, b.breite, b.hoehe), (0, 2, 2, 2));
        assert_eq!(b.deckung, vec![255; 4]);
    }

    #[test]
    fn duenner_stamm_wird_gerettet() {
        // Ein Stamm von einem Viertelpixel Breite deckt keinen
        // Pixelmittelpunkt - die Dropout-Steuerung (Modus 1, der Normalfall
        // ohne Flaggen) setzt trotzdem eine Spalte.
        // Von 1,25 bis 1,40625 Pixel: kein Mittelpunkt liegt darin.
        let b = rendern(&quadrat(80, 0, 90, 256), Rasterflaggen::default()).unwrap();
        assert_eq!((b.breite, b.hoehe), (1, 4));
        assert_eq!(&b.deckung[1..3], &[255, 255]);
        // Ohne Dropout-Steuerung bleibt er leer.
        let ohne = Rasterflaggen { flags: OUTLINE_IGNORE_DROPOUTS, marken: Vec::new() };
        let b = rendern(&quadrat(80, 0, 90, 256), ohne).unwrap();
        assert!(b.deckung.iter().all(|&v| v == 0));
    }

    /// Gegen FreeType selbst: `RUSTGAME_MONO_ORAKEL` zeigt auf eine Datei,
    /// die `ft_orakel.py` mit pygames `freetype.dll` schreibt - je Glyphe
    /// FreeTypes **eigener** gehinteter Umriss, Flaggen, Marken und das
    /// Bild aus `FT_Render_Glyph(.., FT_RENDER_MODE_MONO)`. Damit ist der
    /// Rasterer allein gemessen, ohne dass das Hinting mitspielt.
    #[test]
    #[ignore]
    fn gegen_freetype() {
        let Ok(pfad) = std::env::var("RUSTGAME_MONO_ORAKEL") else {
            return;
        };
        let text = std::fs::read_to_string(pfad).unwrap();
        let (mut gleich, mut anders) = (0usize, Vec::new());
        for zeile in text.lines() {
            let teile: Vec<&str> = zeile.split(" | ").collect();
            let kopf: Vec<&str> = teile[0].split(' ').collect();
            let flags: u32 = kopf[3].parse().unwrap();
            let mut u = Umriss::default();
            let mut marken = Vec::new();
            if teile[1] != "-" {
                for (i, p) in teile[1].split(' ').enumerate() {
                    let w: Vec<i64> = p.split(',').map(|v| v.parse().unwrap()).collect();
                    u.punkte.push((w[0], w[1]));
                    u.arten.push(match w[2] & 3 {
                        1 => Art::Auf,
                        0 => Art::Quadratisch,
                        _ => Art::Kubisch,
                    });
                    if w[2] & 4 != 0 {
                        marken.push((i, ((w[2] & 0xFF) >> 5) as u8));
                    }
                }
                u.enden = teile[2].split(' ').map(|v| v.parse().unwrap()).collect();
            }
            let b: Vec<&str> = teile[3].split(' ').collect();
            let (l, t, w, h, p): (i32, i32, usize, usize, usize) = (
                b[0].parse().unwrap(),
                b[1].parse().unwrap(),
                b[2].parse().unwrap(),
                b[3].parse().unwrap(),
                b[4].parse().unwrap(),
            );
            let roh: Vec<u8> = if b[5] == "-" {
                Vec::new()
            } else {
                (0..b[5].len() / 2)
                    .map(|i| u8::from_str_radix(&b[5][2 * i..2 * i + 2], 16).unwrap())
                    .collect()
            };
            let mut soll = vec![0u8; w * h];
            for y in 0..h {
                for x in 0..w {
                    if roh[y * p + (x >> 3)] & (0x80 >> (x & 7)) != 0 {
                        soll[y * w + x] = 255;
                    }
                }
            }
            let ist = rendern(&u, Rasterflaggen { flags, marken });
            let ok = match &ist {
                Some(m) => (m.left, m.top, m.breite, m.hoehe, &m.deckung) == (l, t, w, h, &soll),
                None => false,
            };
            if ok {
                gleich += 1;
            } else {
                anders.push(teile[0].to_string());
            }
        }
        eprintln!("gleich {gleich}, anders {}", anders.len());
        for a in anders.iter().take(30) {
            eprintln!("  {a}");
        }
        assert!(anders.is_empty());
    }
}
