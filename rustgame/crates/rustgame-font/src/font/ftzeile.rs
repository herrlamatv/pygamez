//! ftzeile.rs
//! ==========
//! Eine Textzeile auf dem Weg von **`pygame.freetype`** - uebertragen aus
//! pygame-ce 2.5.7 `src_c/freetype/ft_layout.c` (`_PGFT_LoadGlyph`,
//! `size_text`, `position_glyphs`, `fill_text_bounding_box`) und
//! `ft_render.c` (`_PGFT_GetRenderMetrics`, `render`), dazu aus FreeType
//! 2.11.1 `FT_Outline_EmboldenXY`, `FT_Outline_Get_Orientation`,
//! `FT_Vector_NormLen` und `FT_Bitmap_Embolden`.
//!
//! ## Warum ein zweiter Zeilenaufbau neben dem von SDL_ttf
//!
//! `pygame.font` ist SDL_ttf mit HarfBuzz, `pygame.freetype` ist pygames
//! eigener Satz direkt auf FreeType - und die beiden setzen die Zeichen
//! verschieden aneinander. Bis Runde 19 rasterte die Huelle fuer
//! `freetype` den SDL_ttf-Weg und schnitt das Ergebnis zu; gemessen war
//! `"Hallo Welt"` dann 122 statt 116 Pixel breit, und **jeder** der vier
//! Stile (strong, oblique, wide, underline) war in keinem einzigen Bild
//! gleich: `strong` ist bei pygame `FT_Outline_Embolden` am Umriss, nicht
//! SDL_ttfs Verschmieren; `oblique` schert mit 0,22 statt 0,207; `wide`
//! ist `FT_Bitmap_Embolden`; und der Unterstrich ist ein Rechteck mit
//! Bruchkanten.
//!
//! ## Was hier **nicht** abgedeckt ist
//!
//! Senkrechter Satz mit einer Schrift, die eigene senkrechte Masse traegt
//! (`vmtx`), und zwei Groessenachsen beim Autohinter - dort gibt
//! [`Font::ft_zeile`] `None`, und die Huelle bleibt beim alten Weg. Eine
//! BDF-Schrift hat nur ihren Strike; welche Groessen ihn treffen, prueft
//! [`Font::bitmap_trifft`]. Gedrehter Text ist seit Runde 19 hier,
//! senkrechter Satz, Bruchgroessen, eingebettete Bitmaps (`EBDT`), CFF und
//! BDF-Schriften auch; zwei Achsen (`size=(24, 12)`) seit Runde 20.

use super::*;
use crate::ftgrays::Umriss;

/// `FX16_ONE`.
const FX16_ONE: i64 = 65536;
/// `SLANT_FACTOR` 0.22 als 16.16: `(FT_Fixed)(0.22 * 65536)`.
const SCHRAEGE: i64 = 14417;

/// Was `pygame.freetype` an einem Text einstellen kann - soweit es den
/// Zeilenaufbau beruehrt.
#[derive(Debug, Clone, Copy, Default)]
pub struct FtStil {
    /// `STYLE_STRONG`.
    pub strong: bool,
    /// `STYLE_OBLIQUE`.
    pub oblique: bool,
    /// `STYLE_WIDE`.
    pub wide: bool,
    /// `STYLE_UNDERLINE`.
    pub underline: bool,
    /// `Font.kerning`.
    pub kerning: bool,
    /// `not Font.antialiased`.
    pub mono: bool,
    /// `Font.pad`.
    pub pad: bool,
    /// `Font.strength` (Vorgabe 1/36).
    pub strength: f64,
    /// `Font.underline_adjustment` (Vorgabe 1.0).
    pub underline_adjustment: f64,
    /// Drehung in Grad, schon `mod 360` (`Font.rotation`).
    pub drehung: i64,
    /// `Font.vertical` - senkrechter Satz.
    pub senkrecht: bool,
    /// Die Nenngroesse in 26.6, wie FreeType sie anfragt
    /// (`FT_REQUEST_WIDTH`) - `0` heisst `ptsize << 6`. Damit gehen
    /// Bruchgroessen (`size=20.5`) und andere Aufloesungen.
    pub groesse26: i64,
    /// Die Hoehe in 26.6 (`FT_REQUEST_HEIGHT`), wenn sie eine andere ist
    /// (`size=(24, 12)`) - `0` heisst so hoch wie breit.
    pub groesse26_y: i64,
}

/// Ein gesetztes Zeichen: Lage in der Zielflaeche und Bild.
#[derive(Debug, Clone)]
pub struct FtGlyphe {
    /// Linke obere Ecke in Pixeln der Zielflaeche
    /// (`FX6_TRUNC(FX6_CEIL(offset + posn))`).
    pub x: i32,
    /// Dto. senkrecht.
    pub y: i32,
    /// Breite des Bildes.
    pub breite: usize,
    /// Hoehe des Bildes.
    pub hoehe: usize,
    /// Deckung, ein Byte je Pixel (einfarbig: 0 oder 255).
    pub deckung: Vec<u8>,
}

/// Was `_PGFT_GetMetrics` von einem geladenen Zeichen weitergibt - die
/// Kanten des Bildes (`image->left`, `image->top`, Breite, Zeilen) und
/// `h_metrics.advance_rotated`.
#[derive(Debug, Clone, Copy)]
pub struct FtMass {
    /// Die Glyphennummer; `0` heisst "nicht in der Schrift" - pygame gibt
    /// dafuer `None`.
    pub id: u16,
    /// `left`.
    pub min_x: i64,
    /// `left + width`.
    pub max_x: i64,
    /// `top - rows`.
    pub min_y: i64,
    /// `top`.
    pub max_y: i64,
    /// `advance_rotated / 64.0`.
    pub vorschub: (f64, f64),
}

/// Die ganze Zeile - was `render`, `render_to` und `get_rect` brauchen.
#[derive(Debug, Clone)]
pub struct FtZeile {
    /// Groesse der Zielflaeche (`_PGFT_GetRenderMetrics`).
    pub breite: u32,
    /// Dto.
    pub hoehe: u32,
    /// `get_rect().x`: `-FX6_TRUNC(FX6_FLOOR(offset.x))`.
    pub rx: i32,
    /// `get_rect().y`: `FX6_TRUNC(FX6_CEIL(offset.y))`.
    pub ry: i32,
    /// Die Zeichen in Satzreihenfolge.
    pub glyphen: Vec<FtGlyphe>,
    /// Der Unterstrich als Rechteck in 26.6 der Zielflaeche:
    /// `(x, y, w, h)` - `surface->fill` aus `render`.
    pub unterstrich: Option<(i64, i64, i64, i64)>,
    /// Ob es ueberhaupt Zeichen gab (`length > 0`).
    pub zeichen: bool,
    /// Je Zeichen die Masse fuer `get_metrics`.
    pub masse: Vec<FtMass>,
}

#[inline]
fn fx6_floor(x: i64) -> i64 {
    x & -64
}
#[inline]
fn fx6_ceil(x: i64) -> i64 {
    (x + 63) & -64
}
#[inline]
fn fx6_round(x: i64) -> i64 {
    (x + 32) & -64
}
#[inline]
fn fx6_trunc(x: i64) -> i64 {
    x >> 6
}
/// `FX16_CEIL_TO_FX6`.
#[inline]
fn fx16_ceil_to_fx6(x: i64) -> i64 {
    (x + 1023) >> 10
}
/// `DBL_TO_FX16`.
#[inline]
fn dbl_to_fx16(d: f64) -> i64 {
    (d * 65536.0) as i64
}

/// `FT_Vector_NormLen` (FreeType 2.11.1, `ftcalc.c`): normiert den Vektor
/// auf Laenge 1.0 (16.16) und gibt die alte Laenge zurueck. Ganzzahlig mit
/// den 32-Bit-Ueberlaeufen der Vorlage.
fn norm_len(v: &mut (i64, i64)) -> i64 {
    let (x_, y_) = (v.0 as i32, v.1 as i32);
    let (sx, sy) = (if x_ < 0 { -1i64 } else { 1 }, if y_ < 0 { -1i64 } else { 1 });
    let mut x = x_.unsigned_abs();
    let mut y = y_.unsigned_abs();
    if x == 0 {
        if y > 0 {
            v.1 = sy * 0x10000;
        }
        return i64::from(y);
    } else if y == 0 {
        if x > 0 {
            v.0 = sx * 0x10000;
        }
        return i64::from(x);
    }
    let mut l: u32 = if x > y { x + (y >> 1) } else { y + (x >> 1) };
    let mut shift: i32 = 31 - (31 - l.leading_zeros() as i32);
    shift -= 15 + i32::from(l >= (0xAAAA_AAAAu32 >> shift));
    if shift > 0 {
        x <<= shift;
        y <<= shift;
        l = if x > y { x + (y >> 1) } else { y + (x >> 1) };
    } else {
        x >>= -shift;
        y >>= -shift;
        l >>= -shift;
    }
    let mut b: i32 = 0x10000 - l as i32;
    let (xi, yi) = (x as i32, y as i32);
    let (mut u, mut w);
    loop {
        u = (xi.wrapping_add(xi.wrapping_mul(b) >> 16)) as u32;
        w = (yi.wrapping_add(yi.wrapping_mul(b) >> 16)) as u32;
        let mut z: i32 = -(u.wrapping_mul(u).wrapping_add(w.wrapping_mul(w)) as i32) / 0x200;
        z = z.wrapping_mul((0x10000 + b) >> 8) / 0x10000;
        b += z;
        if z <= 0 {
            break;
        }
    }
    v.0 = if sx < 0 { -i64::from(u) } else { i64::from(u) };
    v.1 = if sy < 0 { -i64::from(w) } else { i64::from(w) };
    let mut l = (0x10000i32.wrapping_add((u.wrapping_mul(x).wrapping_add(w.wrapping_mul(y)) as i32) / 0x10000)) as u32;
    if shift > 0 {
        l = (l + (1 << (shift - 1))) >> shift;
    } else {
        l <<= -shift;
    }
    i64::from(l)
}

/// `FT_Outline_Get_Orientation`: `Some(true)` fuer TrueType (im
/// Uhrzeigersinn), `Some(false)` fuer PostScript, `None` fuer keine.
fn orientierung(u: &Umriss) -> Option<bool> {
    if u.punkte.is_empty() {
        return Some(true);
    }
    let (x0, y0, x1, y1) = u.kontrollkasten();
    if x0 == x1 || y0 == y1 {
        return None;
    }
    if x0 < -0x100_0000 || y0 < -0x100_0000 || x1 > 0x100_0000 || y1 > 0x100_0000 {
        return None;
    }
    let msb = |z: u32| if z == 0 { 0 } else { 31 - z.leading_zeros() as i32 };
    let xshift = (msb((x1.abs() | x0.abs()) as u32) - 14).max(0);
    let yshift = (msb((y1 - y0) as u32) - 14).max(0);
    let mut area: i32 = 0;
    let mut first = 0usize;
    for &last in &u.enden {
        let mut prev = (u.punkte[last].0 >> xshift, u.punkte[last].1 >> yshift);
        for n in first..=last {
            let cur = (u.punkte[n].0 >> xshift, u.punkte[n].1 >> yshift);
            area = area.wrapping_add(((cur.1 - prev.1) as i32).wrapping_mul((cur.0 + prev.0) as i32));
            prev = cur;
        }
        first = last + 1;
    }
    match area.cmp(&0) {
        std::cmp::Ordering::Greater => Some(false),
        std::cmp::Ordering::Less => Some(true),
        std::cmp::Ordering::Equal => None,
    }
}

/// `FT_Outline_EmboldenXY` (2.11.1). `false` bei `Invalid_Argument`.
fn verdicken(u: &mut Umriss, xstrength: i64, ystrength: i64) -> bool {
    let xs = xstrength / 2;
    let ys = ystrength / 2;
    if xs == 0 && ys == 0 {
        return true;
    }
    let Some(truetype) = orientierung(u) else {
        return u.enden.is_empty();
    };
    let mut first = 0usize;
    let enden = u.enden.clone();
    for &last in &enden {
        let p = &mut u.punkte;
        let mut inn = (0i64, 0i64);
        let mut anchor = (0i64, 0i64);
        let mut l_in: i64 = 0;
        let mut l_anchor: i64 = 0;
        let (mut i, mut j, mut k): (usize, usize, isize) = (last, first, -1);
        while j != i && i as isize != k {
            let mut out;
            let l_out;
            if j as isize != k {
                out = (p[j].0 - p[i].0, p[j].1 - p[i].1);
                l_out = norm_len(&mut out);
                if l_out == 0 {
                    j = if j < last { j + 1 } else { first };
                    continue;
                }
            } else {
                out = anchor;
                l_out = l_anchor;
            }
            if l_in != 0 {
                if k < 0 {
                    k = i as isize;
                    anchor = inn;
                    l_anchor = l_in;
                }
                let mut d = mul_fix(inn.0, out.0) + mul_fix(inn.1, out.1);
                let mut shift;
                if d > -0xF000 {
                    d += 0x10000;
                    shift = (inn.1 + out.1, inn.0 + out.0);
                    if truetype {
                        shift.0 = -shift.0;
                    } else {
                        shift.1 = -shift.1;
                    }
                    let mut q = mul_fix(out.0, inn.1) - mul_fix(out.1, inn.0);
                    if truetype {
                        q = -q;
                    }
                    let l = l_in.min(l_out);
                    shift.0 = if mul_fix(xs, q) <= mul_fix(l, d) {
                        crate::fixed::mul_div(shift.0, xs, d)
                    } else {
                        crate::fixed::mul_div(shift.0, l, q)
                    };
                    shift.1 = if mul_fix(ys, q) <= mul_fix(l, d) {
                        crate::fixed::mul_div(shift.1, ys, d)
                    } else {
                        crate::fixed::mul_div(shift.1, l, q)
                    };
                } else {
                    shift = (0, 0);
                }
                while i != j {
                    p[i].0 += xs + shift.0;
                    p[i].1 += ys + shift.1;
                    i = if i < last { i + 1 } else { first };
                }
            } else {
                i = j;
            }
            inn = out;
            l_in = l_out;
            j = if j < last { j + 1 } else { first };
        }
        first = last + 1;
    }
    true
}

/// `FT_Bitmap_Embolden(bitmap, xStrength, 0)` fuer ein Bild mit einem
/// Byte je Pixel. Einfarbig (0/255) verhaelt es sich wie FreeTypes Bit-
/// Fassung (jedes Bit mit den `xstr` linken Nachbarn verodert, hoechstens
/// acht), in Graustufen addiert es bis 255.
fn bild_verbreitern(
    deckung: &[u8],
    breite: usize,
    hoehe: usize,
    xstrength: i64,
    mono: bool,
) -> (Vec<u8>, usize) {
    let mut xstr = (fx6_round(xstrength) >> 6) as usize;
    if xstr == 0 {
        return (deckung.to_vec(), breite);
    }
    if mono && xstr > 8 {
        xstr = 8;
    }
    let nb = breite + xstr;
    let mut aus = vec![0u8; nb * hoehe];
    for y in 0..hoehe {
        let quelle = &deckung[y * breite..(y + 1) * breite];
        for x in 0..nb {
            let mut v: u32 = 0;
            for i in 0..=xstr {
                if x >= i && x - i < breite {
                    let s = u32::from(quelle[x - i]);
                    if mono {
                        v |= s;
                    } else {
                        v = (v + s).min(255);
                        if v == 255 {
                            break;
                        }
                    }
                }
            }
            aus[y * nb + x] = v as u8;
        }
    }
    (aus, nb)
}

/// Ein geladenes Zeichen (`FontGlyph`): Bild und Masse in 26.6.
struct Geladen {
    breite: usize,
    hoehe: usize,
    deckung: Vec<u8>,
    /// `h_metrics.bearing_rotated`: die Bildecke `(left, top)`.
    lager: (i64, i64),
    /// `h_metrics.advance_rotated` - bei Drehung ein schraeger Vektor.
    vorschub: (i64, i64),
    /// `v_metrics.bearing_rotated` - nur beim senkrechten Satz gefuellt.
    lager_v: (i64, i64),
    /// `v_metrics.advance_rotated` - dto.
    vorschub_v: (i64, i64),
    /// `bitmap.pixel_mode == FT_PIXEL_MODE_GRAY` - ein geglaetteter Umriss
    /// oder ein Achtbit-Strike; ein Einbit-Strike und jeder einfarbige
    /// Umriss nicht.
    grau: bool,
}

/// `FT_Matrix_Multiply(a, b)`: `b = a * b`, Matrizen als `(xx, xy, yx, yy)`.
fn matrix_mal(a: (i64, i64, i64, i64), b: (i64, i64, i64, i64)) -> (i64, i64, i64, i64) {
    (
        mul_fix(a.0, b.0) + mul_fix(a.1, b.2),
        mul_fix(a.0, b.1) + mul_fix(a.1, b.3),
        mul_fix(a.2, b.0) + mul_fix(a.3, b.2),
        mul_fix(a.2, b.1) + mul_fix(a.3, b.3),
    )
}

impl Font {
    /// Eine Zeile auf `pygame.freetype`s Weg - siehe die Moduldoku.
    ///
    /// `None` heisst: dieser Fall ist hier nicht nachgebaut (Bitmapschrift,
    /// eingebettete Bitmap, Umriss nicht zu hinten) - die Huelle nimmt
    /// dann ihren alten Weg.
    pub fn ft_zeile(&self, text: &str, stil: &FtStil, strikes: bool) -> Option<FtZeile> {
        let sicht = Sicht::neu(&self.datei);
        let mut g26 = if stil.groesse26 > 0 {
            stil.groesse26
        } else {
            i64::from(self.ptsize.max(0)) << 6
        };
        // Die Hoehe (`FT_REQUEST_HEIGHT`); meist dieselbe.
        let mut g26_y = if stil.groesse26_y > 0 { stil.groesse26_y } else { g26 };
        // `FT_Request_Metrics`: `x_ppem = (scaled_w + 32) >> 6`, `y_ppem`
        // aus der Hoehe.
        let ppem = ((g26 + 32) >> 6) as u32;
        let ppem_y = ((g26_y + 32) >> 6) as u32;
        // Eine BDF-Schrift hat keinen Umriss, aber FreeTypes BDF-Treiber
        // liefert ihre Bilder genauso in den Satz - siehe `ft_laden_bdf`.
        let bdf = self.datei.punkte.as_ref();
        if bdf.is_none() && (ppem == 0 || ppem_y == 0 || !self.datei.hat_umrisse()) {
            return None;
        }
        // Ein Strike dieser Zeilenzahl gewinnt die Groessenanfrage, und
        // `FT_Select_Metrics` rechnet dann ganzzahlig. `FT_Match_Size`
        // verlangt Breite **und** Hoehe des Strikes.
        if ppem == ppem_y && self.datei.hat_strike(ppem as u16) {
            g26 = i64::from(ppem) << 6;
            g26_y = g26;
        }
        let upem = i64::from(self.datei.units_per_em.max(1));
        // `size->metrics.x_scale`/`y_scale` - die oeffentliche Skala,
        // ungerundet. Bei ganzer Groesse ist das `self.scale`.
        let skala = crate::fixed::div_fix(g26, upem);
        let skala_y = crate::fixed::div_fix(g26_y, upem);
        // Die Skala des Laders (`hinted_metrics`): mit `head.flags` Bit 3
        // die der gerundeten Groesse. **Nur TrueType** - das Runden steht in
        // `tt_size_reset`; der CFF-Treiber skaliert immer mit der
        // ungerundeten (gemessen an `test_fixed.otf` bei 20.5: Vorschub 10,
        // gerundet waeren es 11).
        let (lade_skala, lade_skala_y) = if sicht.ppem_ganzzahlig() && !sicht.ist_cff() {
            (
                crate::fixed::div_fix(i64::from(ppem) << 6, upem),
                crate::fixed::div_fix(i64::from(ppem_y) << 6, upem),
            )
        } else {
            (skala, skala_y)
        };
        let strength = dbl_to_fx16(stil.strength);
        // `size->metrics.x_ppem` - bei BDF die des einen Strikes.
        let x_ppem = bdf.map_or(i64::from(ppem), |b| i64::from(b.ppem.0));
        // `INT_TO_FX16(angle % 360)`
        let winkel = stil.drehung.rem_euclid(360) << 16;

        // ----- size_text ------------------------------------------------
        let zeichen: Vec<char> = text.chars().collect();
        let mut ids = Vec::with_capacity(zeichen.len());
        let mut kerning = Vec::with_capacity(zeichen.len());
        let mut prev: u16 = 0;
        for &c in &zeichen {
            let id = sicht.glyph(c);
            // `FT_Get_Kerning(.., FT_KERNING_UNFITTED)`: nur die alte
            // `kern`-Tabelle, mit `x_scale` multipliziert.
            let k = mul_fix(sicht.kerning_freetype(prev, id), skala);
            kerning.push(k);
            ids.push(id);
            prev = id;
        }
        // `ft_recompute_scaled_metrics`: Auf- und Unterlaenge, Zeilenhoehe
        // und groesster Vorschub, auf ganze Pixel gebracht - senkrecht mit
        // `y_scale`, nur der Vorschub mit `x_scale`.
        let d = &self.datei;
        let ascender = fx6_ceil(mul_fix(i64::from(d.ascender), skala_y));
        let descender = fx6_floor(mul_fix(i64::from(d.descender), skala_y));
        let zeilenhoehe = crate::fixed::pix_round26(mul_fix(
            i64::from(d.ascender) - i64::from(d.descender) + i64::from(d.line_gap),
            skala_y,
        ));
        let max_vorschub =
            crate::fixed::pix_round26(mul_fix(sicht.max_vorschub_einheiten(), skala));
        // `ft_layout.c`: der Unterstrich mit `y_scale`.
        let mut underline_pos = -mul_fix(i64::from(self.datei.underline_position), skala_y);
        let mut underline_size = mul_fix(i64::from(self.datei.underline_thickness), skala_y);
        // BDF (`BDF_Size_Select`): Auf- und Unterlaenge aus
        // `FONT_ASCENT`/`FONT_DESCENT`, Zeilenhoehe ihre Summe, groesster
        // Vorschub die Breite von `FONTBOUNDINGBOX`. Unterstrichmasse setzt
        // der Treiber keine - der Strich ist null dick und wird nie
        // gezeichnet (gemessen: `STYLE_UNDERLINE` aendert dort kein Pixel).
        let (ascender, descender, zeilenhoehe, max_vorschub) = match bdf {
            Some(b) => {
                underline_pos = 0;
                underline_size = 0;
                (
                    i64::from(b.ascent) << 6,
                    -(i64::from(b.descent) << 6),
                    i64::from(b.ascent + b.descent) << 6,
                    i64::from(b.kasten.0) << 6,
                )
            }
            None => (ascender, descender, zeilenhoehe, max_vorschub),
        };
        if stil.strong {
            let bold_str = strength * x_ppem;
            underline_size = mul_fix(underline_size, FX16_ONE + bold_str / 4);
        }

        // ----- load_glyphs ----------------------------------------------
        let mut geladen = Vec::with_capacity(ids.len());
        for &id in &ids {
            if let Some(b) = bdf {
                geladen.push(Self::ft_laden_bdf(b, id, stil, strength, x_ppem));
                continue;
            }
            // `get_load_flags`: ohne `FT_LOAD_NO_BITMAP` nur, wenn Strikes
            // erlaubt sind und weder fett noch schraeg noch gedreht wird.
            // Hat der Strike das Zeichen nicht, laedt FreeType den Umriss.
            if strikes
                && !stil.strong
                && !stil.oblique
                && stil.drehung.rem_euclid(360) == 0
                && ppem == ppem_y
            {
                if let Some(k) = self.datei.strike(id, ppem as u16) {
                    geladen.push(self.ft_laden_strike(id, k, ppem, stil, strength)?);
                    continue;
                }
            }
            geladen.push(self.ft_laden(
                id,
                (g26, g26_y),
                (lade_skala, lade_skala_y),
                stil,
                strength,
            )?);
        }

        // ----- position_glyphs ------------------------------------------
        let mut pen = (0i64, 0i64);
        let mut pen1 = (0i64, 0i64);
        let mut pen2;
        let (mut min_x, mut max_x) = (i64::MAX, i64::MIN);
        let (mut min_y, mut max_y) = (i64::MAX, i64::MIN);
        let mut top = i64::MIN;
        let mut lagen = Vec::with_capacity(geladen.len());
        for (n, g) in geladen.iter().enumerate() {
            pen2 = pen1;
            pen1 = pen;
            if stil.kerning {
                let mut k = (kerning[n], 0i64);
                if winkel != 0 {
                    k = crate::ftstroke::drehen(k, winkel);
                }
                pen.0 += fx6_round(k.0);
                pen.1 += fx6_round(k.1);
                if crate::ftstroke::laenge(pen2) > crate::ftstroke::laenge(pen) {
                    pen = pen2;
                }
            }
            let (lager, vorschub) = if stil.senkrecht {
                (g.lager_v, g.vorschub_v)
            } else {
                (g.lager, g.vorschub)
            };
            let (bx, by) = lager;
            let gw = (g.breite as i64) << 6;
            let gh = (g.hoehe as i64) << 6;
            top = top.max(by);
            min_x = min_x.min(pen.0 + bx);
            max_x = max_x.max(pen.0 + bx + gw);
            let posn_x = pen.0 + bx;
            pen.0 += vorschub.0;
            let posn_y;
            if stil.senkrecht {
                min_y = min_y.min(pen.1 + by);
                max_y = max_y.max(pen.1 + by + gh);
                posn_y = pen.1 + by;
                pen.1 += vorschub.1;
            } else {
                min_y = min_y.min(pen.1 - by);
                max_y = max_y.max(pen.1 - by + gh);
                posn_y = pen.1 - by;
                pen.1 -= vorschub.1;
            }
            lagen.push((posn_x, posn_y));
        }
        // "the special case of a trailing space"
        if let Some(letzt) = geladen.last() {
            if letzt.breite == 0 {
                if pen.0 < min_x {
                    min_x = pen.0;
                } else if pen.0 > max_x {
                    max_x = pen.0;
                }
                if pen.1 < min_y {
                    min_y = pen.1;
                } else if pen.1 > max_y {
                    max_y = pen.1;
                }
            }
        }

        // ----- fill_text_bounding_box -----------------------------------
        if geladen.is_empty() {
            min_x = 0;
            max_x = 0;
            min_y = if stil.senkrecht { 0 } else { -ascender };
            max_y = if stil.senkrecht { zeilenhoehe } else { -descender };
        }
        if stil.pad {
            if pen.0 > max_x {
                max_x = pen.0;
            } else if pen.0 < min_x {
                min_x = pen.0;
            }
            if pen.1 > max_y {
                max_y = pen.1;
            } else if pen.1 < min_y {
                min_y = pen.1;
            }
            if stil.senkrecht {
                // `right = max_advance / 2`
                let rechts = max_vorschub / 2;
                if max_x < rechts {
                    max_x = rechts;
                }
                if min_x > -rechts {
                    min_x = -rechts;
                }
                if min_y > 0 {
                    min_y = 0;
                }
            } else {
                if min_x > 0 {
                    min_x = 0;
                }
                if min_y > -ascender {
                    min_y = -ascender;
                }
                if max_y <= -descender {
                    max_y = -descender + 64;
                }
            }
        }

        // ----- _PGFT_GetRenderMetrics -----------------------------------
        let mut unterstrich = None;
        if stil.underline {
            let half = (underline_size + 1) / 2;
            let adj = dbl_to_fx16(stil.underline_adjustment);
            let pos = if adj < 0 {
                mul_fix(ascender, adj)
            } else {
                mul_fix(underline_pos, adj)
            };
            let oben = pos - half;
            let unten = oben + underline_size;
            if unten > max_y {
                max_y = unten;
            }
            if oben < min_y {
                min_y = oben;
            }
            unterstrich = Some((oben, underline_size));
        }
        let offset = (-min_x, -min_y);
        let w = fx6_trunc(fx6_ceil(max_x) - fx6_floor(min_x));
        let h = fx6_trunc(fx6_ceil(max_y) - fx6_floor(min_y));

        // ----- _PGFT_GetMetrics -----------------------------------------
        let masse = ids
            .iter()
            .zip(&geladen)
            .map(|(&id, g)| FtMass {
                id,
                min_x: g.lager.0 >> 6,
                max_x: (g.lager.0 >> 6) + g.breite as i64,
                min_y: (g.lager.1 >> 6) - g.hoehe as i64,
                max_y: g.lager.1 >> 6,
                vorschub: (g.vorschub.0 as f64 / 64.0, g.vorschub.1 as f64 / 64.0),
            })
            .collect();

        // ----- render ---------------------------------------------------
        let geladen_grau = geladen.iter().any(|g| g.grau);
        let glyphen = geladen
            .into_iter()
            .zip(lagen)
            .map(|(g, (px, py))| FtGlyphe {
                x: fx6_trunc(fx6_ceil(offset.0 + px)) as i32,
                y: fx6_trunc(fx6_ceil(offset.1 + py)) as i32,
                breite: g.breite,
                hoehe: g.hoehe,
                deckung: g.deckung,
            })
            .collect();
        // `render`: Bruchkanten nur, wenn **ein** Zeichen ein Graubild war
        // (`is_underline_gray`). Aus lauter Einbit-Strikes rastet der
        // Strich auch geglaettet auf ganze Pixel ein.
        let grau = geladen_grau;
        // `if (underline_size > 0)` - ein Strich der Dicke null fehlt ganz.
        let unterstrich = unterstrich
            .filter(|&(_, groesse)| w > 0 && h > 0 && groesse > 0)
            .map(|(oben, groesse)| {
            if !grau {
                (
                    fx6_ceil(offset.0 + min_x),
                    fx6_ceil(offset.1 + oben),
                    w << 6,
                    fx6_ceil(groesse),
                )
            } else {
                (offset.0 + min_x, offset.1 + oben, w << 6, groesse)
            }
        });
        Some(FtZeile {
            breite: w.max(0) as u32,
            hoehe: h.max(0) as u32,
            rx: -fx6_trunc(fx6_floor(offset.0)) as i32,
            ry: fx6_trunc(fx6_ceil(offset.1)) as i32,
            glyphen,
            unterstrich,
            zeichen: !zeichen.is_empty(),
            masse,
        })
    }

    /// Die erste Glyphe des Textes, die FreeType **nicht laden** wuerde -
    /// `pygame.freetype` sagt dann `Unable to load glyph for id N`
    /// (`load_glyphs` in `ft_layout.c`), und `get_metrics` gibt `None`.
    /// Das geschieht, wenn das CV-Programm in dieser Groesse scheitert
    /// (siehe `Hinter::gescheitert`): dann scheitert jede
    /// Glyphe, die nicht aus einem Strike kommt. Gemessen an `monbaiti.ttf`
    /// (`get_rect("Ag", size=12)`: id 36); hier wurde bis Runde 20 still
    /// der alte Weg genommen.
    pub fn ft_ladefehler(&self, text: &str, stil: &FtStil, strikes: bool) -> Option<u16> {
        if self.datei.punkte.is_some() || !self.datei.hat_umrisse() {
            return None;
        }
        let g26 = if stil.groesse26 > 0 {
            stil.groesse26
        } else {
            i64::from(self.ptsize.max(0)) << 6
        };
        let g26_y = if stil.groesse26_y > 0 { stil.groesse26_y } else { g26 };
        let (ppem, ppem_y) = ((g26 + 32) >> 6, (g26_y + 32) >> 6);
        let strike = ppem == ppem_y && self.datei.hat_strike(ppem as u16);
        let schluessel = if strike {
            ((ppem << 6) as u32, (ppem << 6) as u32)
        } else {
            (g26 as u32, g26_y as u32)
        };
        if !self.datei.hinting_gescheitert(schluessel) {
            return None;
        }
        let sicht = Sicht::neu(&self.datei);
        let aus_strike = strikes
            && strike
            && !stil.strong
            && !stil.oblique
            && stil.drehung.rem_euclid(360) == 0;
        text.chars()
            .map(|c| sicht.glyph(c))
            .find(|&id| !(aus_strike && self.datei.strike(id, ppem as u16).is_some()))
    }

    /// `_PGFT_LoadGlyph`.
    ///
    /// `g26` ist die Nenngroesse in 26.6 (Breite, Hoehe), `lade_skala` die
    /// Skala, mit der FreeTypes Lader rechnet (`x_scale`, `y_scale`; siehe
    /// [`Font::ft_zeile`]).
    fn ft_laden(
        &self,
        id: u16,
        g26: (i64, i64),
        lade_skala: (i64, i64),
        stil: &FtStil,
        strength: i64,
    ) -> Option<Geladen> {
        let d = &self.datei;
        let (mut u, fl, hori_advance) =
            d.ft_umriss((g26.0 as u32, g26.1 as u32), id, stil.mono)?;
        // Die Masse des Ladens (`slot->metrics`) - am gehinteten Umriss,
        // bevor er verdickt oder geschert wird.
        let ft_metrik = self.ft_glyphmasse(&u, hori_advance, stil.senkrecht, lade_skala.1)?;
        let x_ppem = (g26.0 + 32) >> 6;
        let lade_skala = lade_skala.0;
        let mut delta_x: i64 = 0;
        let mut delta_y: i64 = 0;
        if stil.strong {
            let bold_str = fx16_ceil_to_fx6(strength * x_ppem);
            let (bx0, by0, bx1, by1) = u.kontrollkasten();
            if !verdicken(&mut u, bold_str, bold_str) {
                return None;
            }
            let (ax0, ay0, ax1, ay1) = u.kontrollkasten();
            delta_x += (ax1 - ax0) - (bx1 - bx0);
            delta_y += (ay1 - ay0) - (by1 - by0);
        }
        // `fill_context`: Scherung, dann `FT_Matrix_Multiply(&rotate,
        // &transform)` - also Drehung mal Scherung.
        let winkel = stil.drehung.rem_euclid(360) << 16;
        let mut matrix = if stil.oblique {
            (FX16_ONE, SCHRAEGE, 0, FX16_ONE)
        } else {
            (FX16_ONE, 0, 0, FX16_ONE)
        };
        if winkel != 0 {
            let (c, s) = crate::ftstroke::einheit(winkel);
            matrix = matrix_mal((c, -s, s, c), matrix);
        }
        if stil.oblique || winkel != 0 {
            for p in &mut u.punkte {
                // `FT_Outline_Transform`
                let (x, y) = *p;
                p.0 = mul_fix(x, matrix.0) + mul_fix(y, matrix.1);
                p.1 = mul_fix(x, matrix.2) + mul_fix(y, matrix.3);
            }
        }
        let (left, top, breite, hoehe, deckung) = if stil.mono {
            let b = crate::ftraster::rendern(&u, fl)?;
            (b.left, b.top, b.breite, b.hoehe, b.deckung)
        } else {
            let b = crate::ftgrays::rendern(&u)?;
            (b.left, b.top, b.breite, b.hoehe, b.deckung)
        };
        let (deckung, breite) = if stil.wide {
            let x_strength = fx16_ceil_to_fx6(strength * x_ppem);
            if breite > 0 {
                let (dk, nb) = bild_verbreitern(&deckung, breite, hoehe, x_strength, stil.mono);
                delta_x += ((nb - breite) as i64) << 6;
                (dk, nb)
            } else {
                delta_x += x_strength;
                (deckung, breite)
            }
        } else {
            (deckung, breite)
        };
        let sicht = Sicht::neu(d);
        // Ohne Rueckwaertsmodus ist der Vorschub der gehintete (siehe
        // `Hinter::gehinteter_vorschub`) - nur mit einer Achse.
        let linear = if g26.0 == g26.1 {
            d.gehinteter_vorschub((g26.0 as u32, g26.1 as u32), id)
        } else {
            None
        };
        let mut vorschub = (
            linear.unwrap_or_else(|| sicht.vorschub26(id, lade_skala)) + delta_x,
            0i64,
        );
        let mut vorschub_v = (0i64, ft_metrik.vert_advance + delta_y);
        if winkel != 0 {
            vorschub = crate::ftstroke::drehen(vorschub, winkel);
            // `counter_rotation = INT_TO_FX16(360) - rotation_angle`
            vorschub_v = crate::ftstroke::drehen(vorschub_v, (360 << 16) - winkel);
        }
        let lager = (i64::from(left) << 6, i64::from(top) << 6);
        let lager_v = if winkel == 0 {
            (ft_metrik.vert_bearing_x - delta_x / 2, ft_metrik.vert_bearing_y)
        } else {
            // "Adjust the vertical metrics."
            let v_origin = crate::ftstroke::drehen(
                (
                    ft_metrik.hori_bearing_x - ft_metrik.vert_bearing_x + delta_x / 2,
                    ft_metrik.hori_bearing_y + ft_metrik.vert_bearing_y,
                ),
                winkel,
            );
            (lager.0 - v_origin.0, v_origin.1 - lager.1)
        };
        Some(Geladen {
            breite,
            hoehe,
            deckung,
            lager,
            vorschub,
            lager_v,
            vorschub_v,
            grau: !stil.mono,
        })
    }

    /// `_PGFT_LoadGlyph` fuer eine **BDF-Schrift** - FreeTypes
    /// `BDF_Glyph_Load` (`bdfdrivr.c`): das Bild der Datei, `bitmap_left`
    /// ist `BBX`-x, `bitmap_top` die Oberkante ueber der Grundlinie, der
    /// Vorschub `DWIDTH`. Die senkrechten Masse erfindet
    /// `ft_synthesize_vertical_metrics` mit der Hoehe von
    /// `FONTBOUNDINGBOX` als Vorschub. `STYLE_WIDE` ist
    /// `FT_Bitmap_Embolden` auf dem Einbit-Bild.
    ///
    /// Bis Runde 19 ging eine BDF-Schrift in `pygame.freetype` den alten
    /// Weg: `PyGameMono-18-100dpi.bdf` bei 18 war in **keinem** Bild gleich.
    fn ft_laden_bdf(
        b: &crate::bdf::Bitmapschrift,
        id: u16,
        stil: &FtStil,
        strength: i64,
        x_ppem: i64,
    ) -> Geladen {
        let (w, h, links, oben, vorschub, deck) = match b.bei(id) {
            Some(z) => (
                z.breite as usize,
                z.hoehe as usize,
                z.x_versatz,
                z.y_versatz + z.hoehe as i32,
                z.vorschub,
                z.deckung.clone(),
            ),
            None => (0, 0, 0, 0, 0, Vec::new()),
        };
        let mut delta_x: i64 = 0;
        let (deckung, breite) = if stil.wide {
            let x_strength = fx16_ceil_to_fx6(strength * x_ppem);
            if w > 0 {
                let (dk, nb) = bild_verbreitern(&deck, w, h, x_strength, true);
                delta_x += ((nb - w) as i64) << 6;
                (dk, nb)
            } else {
                delta_x += x_strength;
                (deck, w)
            }
        } else {
            (deck, w)
        };
        let hori_bearing_x = i64::from(links) << 6;
        let hori_bearing_y = i64::from(oben) << 6;
        let hori_advance = i64::from(vorschub) << 6;
        // `ft_synthesize_vertical_metrics(&slot->metrics, bbx.height * 64)`
        let mut hoehe = (h as i64) << 6;
        if hori_bearing_y < 0 {
            if hoehe < hori_bearing_y {
                hoehe = hori_bearing_y;
            }
        } else if hori_bearing_y > 0 {
            hoehe -= hori_bearing_y;
        }
        let mut vert_advance = i64::from(b.kasten.1) << 6;
        if vert_advance == 0 {
            vert_advance = hoehe * 12 / 10;
        }
        let vert_bearing_x = hori_bearing_x - hori_advance / 2;
        let vert_bearing_y = (vert_advance - hoehe) / 2;
        Geladen {
            breite,
            hoehe: h,
            deckung,
            lager: (hori_bearing_x, hori_bearing_y),
            vorschub: (hori_advance + delta_x, 0),
            lager_v: (vert_bearing_x - delta_x / 2, vert_bearing_y),
            vorschub_v: (0, vert_advance),
            grau: false,
        }
    }

    /// `_PGFT_LoadGlyph` fuer ein **eingebautes Bild**: `FT_Glyph_To_Bitmap`
    /// tut nichts, `left`/`top` sind `bitmap_left`/`bitmap_top` aus den
    /// sbit-Massen, der Vorschub ist deren `horiAdvance`. `STYLE_WIDE` ist
    /// auch hier `FT_Bitmap_Embolden`.
    ///
    /// Senkrecht gelten die senkrechten sbit-Masse; kleine Masse haben
    /// keine, dann stehen bei pygame alle Zeichen an derselben Stelle
    /// (gemessen an `PyGameMono.otf` bei 8 und 19).
    fn ft_laden_strike(
        &self,
        id: u16,
        k: crate::face::Strikebild,
        ppem: u32,
        stil: &FtStil,
        strength: i64,
    ) -> Option<Geladen> {
        let (vorschub_px, (vbx, vby, vadv)) = self.datei.strike_masse(id, ppem as u16)?;
        // `TT_Load_Glyph`, "sanity checks": steht ein Vorschub in den
        // sbit-Massen nicht (kleine Masse haben keinen senkrechten), nimmt
        // FreeType `linearHoriAdvance`/`linearVertAdvance` mal der Skala des
        // Strikes (`FT_Select_Metrics`: `y_ppem << 6` durch `units_per_EM`)
        // - **ungerundet**, denn `ft_glyphslot_grid_fit_metrics` laeuft nur
        // fuer Umrisse. `linearVertAdvance` ist ohne `vmtx` die Typo-Hoehe
        // (`TT_Get_VMetrics`). Gemessen an Calibri (Strikes 12 bis 19)
        // senkrecht bei 19: `"Hallo"` 85 Zeilen hoch, hier waren es 13.
        // **Nur TrueType**: `cff_slot_load` laedt Strikes ohne diese
        // Pruefung, dort bleibt der Vorschub null (`PyGameMono.otf`
        // senkrecht bei 19: 15 Zeilen).
        let sicht = Sicht::neu(&self.datei);
        let tt = !sicht.ist_cff();
        let skala = crate::fixed::div_fix(
            i64::from(ppem) << 6,
            i64::from(self.datei.units_per_em.max(1)),
        );
        let vorschub26 = if vorschub_px == 0 && tt {
            mul_fix(sicht.vorschub_einheiten(id), skala)
        } else {
            vorschub_px << 6
        };
        let vorschub_v26 = if vadv != 0 || !tt {
            vadv << 6
        } else if stil.senkrecht {
            mul_fix(sicht.senkrecht_einheiten()?.abs(), skala)
        } else {
            0
        };
        let mut delta_x: i64 = 0;
        // Ein Einbit-Bild kommt als 0/255 an; FreeType verbreitert es Bit
        // fuer Bit (`FT_PIXEL_MODE_MONO`), ein Graubild durch Aufaddieren.
        let mono = k.deckung.iter().all(|&v| v == 0 || v == 255);
        let (deckung, breite) = if stil.wide {
            let x_strength = fx16_ceil_to_fx6(strength * i64::from(ppem));
            if k.width > 0 {
                let (dk, nb) = bild_verbreitern(&k.deckung, k.width, k.rows, x_strength, mono);
                delta_x += ((nb - k.width) as i64) << 6;
                (dk, nb)
            } else {
                delta_x += x_strength;
                (k.deckung, k.width)
            }
        } else {
            (k.deckung, k.width)
        };
        Some(Geladen {
            breite,
            hoehe: k.rows,
            deckung,
            lager: (i64::from(k.left) << 6, i64::from(k.top) << 6),
            vorschub: (vorschub26 + delta_x, 0),
            lager_v: ((vbx << 6) - delta_x / 2, vby << 6),
            vorschub_v: (0, vorschub_v26),
            grau: !mono,
        })
    }

    /// Die senkrechten Masse eines Zeichens, wie FreeType 2.11.1 sie nach
    /// dem Laden in `slot->metrics` stehen hat: `compute_glyph_metrics`
    /// (`ttgload.c`) fuer eine Schrift **ohne** `vmtx`, danach
    /// `ft_glyphslot_grid_fit_metrics` (`ftobjs.c`).
    ///
    /// **pygame setzt `FT_LOAD_VERTICAL_LAYOUT` nie** (`get_load_flags`).
    /// Bei TrueType aendert das nichts an diesen Werten. Bei CFF schon:
    /// ohne das Flag erfindet `cff_slot_load` keine senkrechten Lager -
    /// `vertBearingX/Y` bleiben null, `vertAdvance` ist die Typo-Hoehe.
    /// Gemessen ueber `ctypes` an pygames `freetype.dll`: mit dem Flag
    /// waeren es bei `PyGameMono.otf` 12 `(-384, 384, 768)`, ohne
    /// `(0, 0, 768)`, und pygames Rechtecke passen nur zum zweiten.
    ///
    /// Waagerecht gesetzt braucht pygame davon nichts; dann sind es
    /// Nullen. Senkrecht mit einer Schrift, die `vmtx` hat, gibt es `None`.
    fn ft_glyphmasse(
        &self,
        u: &Umriss,
        hori_advance: Option<i64>,
        senkrecht: bool,
        y_scale: i64,
    ) -> Option<FtGlyphmasse> {
        if !senkrecht {
            return Some(FtGlyphmasse::default());
        }
        let sicht = Sicht::neu(&self.datei);
        let einheiten = sicht.senkrecht_einheiten()?;
        let (x0, y0, _, y1) = u.kontrollkasten();
        if sicht.ist_cff() {
            return Some(FtGlyphmasse {
                hori_bearing_x: fx6_floor(x0),
                hori_bearing_y: fx6_ceil(y1),
                vert_bearing_x: 0,
                vert_bearing_y: 0,
                vert_advance: fx6_round(mul_fix(einheiten, y_scale)),
            });
        }
        let hori_advance = hori_advance?;
        // `height = (FT_Short)FT_DivFix(bbox.yMax - bbox.yMin, y_scale)`
        let hoehe = i64::from(crate::fixed::div_fix(y1 - y0, y_scale) as i16);
        // `top = (advance - height) / 2` - C teilt zur Null hin, Rust auch.
        let top = mul_fix((einheiten - hoehe) / 2, y_scale);
        let advance = mul_fix(einheiten, y_scale);
        Some(FtGlyphmasse {
            hori_bearing_x: fx6_floor(x0),
            hori_bearing_y: fx6_ceil(y1),
            vert_bearing_x: fx6_floor(x0 - hori_advance / 2),
            vert_bearing_y: fx6_floor(top),
            vert_advance: fx6_round(advance),
        })
    }
}

/// Was [`Font::ft_glyphmasse`] aus `slot->metrics` liest - in 26.6, nach
/// dem Rasterschnitt.
#[derive(Debug, Clone, Copy, Default)]
struct FtGlyphmasse {
    hori_bearing_x: i64,
    hori_bearing_y: i64,
    vert_bearing_x: i64,
    vert_bearing_y: i64,
    vert_advance: i64,
}
