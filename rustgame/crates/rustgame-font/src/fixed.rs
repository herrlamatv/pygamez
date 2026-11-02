//! fixed.rs
//! ========
//! FreeTypes Festkommarechnung - zeilengetreu uebertragen.
//!
//! Alle Schriftmasse in pygame stammen aus SDL_ttf, und SDL_ttf rechnet mit
//! FreeTypes Ganzzahlarithmetik. Wer hier float einsetzt, liegt frueher oder
//! spaeter um ein Pixel daneben, weil FreeType an anderen Stellen rundet.
//! Deshalb steht das hier so, wie es in `ftcalc.c` und `SDL_ttf.c` steht:
//!
//! | FreeType | hier |
//! |---|---|
//! | `FT_MulFix(a, b)` | [`mul_fix`] - `a * b` in 16.16 |
//! | `FT_DivFix(a, b)` | [`div_fix`] - `a / b` in 16.16 |
//! | `FT_FLOOR(x)` | [`floor26`] - 26.6 abrunden |
//! | `FT_CEIL(x)` | [`ceil26`] - 26.6 aufrunden |
//! | `FT_PIX_ROUND(x)` | [`pix_round26`] - 26.6 auf ganze Pixel runden |
//!
//! Gemessen: mit diesen fuenf Funktionen treffen `get_ascent`,
//! `get_descent`, `get_height` und `get_linesize` **exakt**, ueber alle
//! geprueften Schriften und Groessen.

/// `FT_MulFix`: multipliziert eine Zahl mit einem 16.16-Faktor.
///
/// Die Vorlage aus `ftcalc.c` rundet zur naechsten ganzen Zahl und zieht bei
/// negativen Ergebnissen eine 1 ab, damit der arithmetische Schiebeschritt
/// symmetrisch bleibt:
/// `(a * b + 0x8000 - (ab < 0)) >> 16`.
#[inline]
pub fn mul_fix(a: i64, b: i64) -> i64 {
    let ab = a * b;
    (ab + 0x8000 - i64::from(ab < 0)) >> 16
}

/// `FT_DivFix`: teilt und liefert das Ergebnis als 16.16-Faktor.
///
/// Vorzeichen werden abgespalten, gerundet wird kaufmaennisch (`+ b/2`).
#[inline]
pub fn div_fix(a: i64, b: i64) -> i64 {
    let mut s = 1i64;
    let mut a = a;
    let mut b = b;
    if a < 0 {
        a = -a;
        s = -s;
    }
    if b < 0 {
        b = -b;
        s = -s;
    }
    let q = if b == 0 {
        0x7FFF_FFFF
    } else {
        ((a << 16) + (b >> 1)) / b
    };
    q * s
}

/// `FT_MulDiv`: `a * b / c` mit 64-Bit-Zwischenwert, gerundet, Vorzeichen
/// getrennt - und das Ergebnis als 32-Bit-`long` (Windows).
pub fn mul_div(a: i64, b: i64, c: i64) -> i64 {
    let s = (a < 0) ^ (b < 0) ^ (c < 0);
    let (a, b, c) = (a.unsigned_abs(), b.unsigned_abs(), c.unsigned_abs());
    let d = (a * b + (c >> 1)).checked_div(c).unwrap_or(0x7FFF_FFFF);
    let d = d as i32 as i64;
    if s {
        -d
    } else {
        d
    }
}

/// `FT_MulDiv_No_Round`: wie [`mul_div`], aber abgeschnitten.
pub fn mul_div_no_round(a: i64, b: i64, c: i64) -> i64 {
    let s = (a < 0) ^ (b < 0) ^ (c < 0);
    let (a, b, c) = (a.unsigned_abs(), b.unsigned_abs(), c.unsigned_abs());
    let d = (a * b).checked_div(c).unwrap_or(0x7FFF_FFFF);
    let d = d as i32 as i64;
    if s {
        -d
    } else {
        d
    }
}

/// `FT_FLOOR`: 26.6-Festkomma auf das naechstkleinere ganze Pixel.
#[inline]
pub fn floor26(x: i64) -> i32 {
    (x >> 6) as i32
}

/// `FT_CEIL`: 26.6-Festkomma auf das naechstgroessere ganze Pixel.
#[inline]
pub fn ceil26(x: i64) -> i32 {
    ((x + 63) >> 6) as i32
}

/// `FT_PIX_ROUND`: 26.6-Festkomma auf das naechste ganze Pixel - bleibt 26.6.
#[inline]
pub fn pix_round26(x: i64) -> i64 {
    (x + 32) & -64
}

/// Der Skalierungsfaktor einer Schrift: wie viele 16.16-Einheiten ein
/// Schrifteinheits-Schritt bei dieser Punktgroesse wert ist.
///
/// `FT_Set_Char_Size(face, 0, ptsize * 64, 0, 0)` landet in FreeType bei
/// einer **nominalen** Groessenanfrage mit 72 dpi; dort wird
/// `y_scale = FT_DivFix(ppem << 6, units_per_EM)` gerechnet, und `ppem` ist
/// bei 72 dpi genau `ptsize`.
#[inline]
pub fn scale_for(ptsize: i32, units_per_em: u16) -> i64 {
    div_fix(i64::from(ptsize) * 64, i64::from(units_per_em))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mul_fix_rundet_wie_freetype() {
        assert_eq!(mul_fix(1, 1 << 16), 1);
        assert_eq!(mul_fix(100, 1 << 15), 50);
        // Halbe Schritte: aufrunden nach oben, bei negativen Werten nach oben
        // im Betrag - das ist der Sinn des "- (ab < 0)".
        assert_eq!(mul_fix(1, 1 << 15), 1);
        assert_eq!(mul_fix(-1, 1 << 15), -1);
    }

    #[test]
    fn div_fix_ist_die_umkehrung() {
        let s = div_fix(16 * 64, 1000);
        assert_eq!(mul_fix(1000, s), 16 * 64);
    }

    #[test]
    fn runden_in_26_6() {
        assert_eq!(floor26(64), 1);
        assert_eq!(floor26(127), 1);
        assert_eq!(floor26(-1), -1);
        assert_eq!(ceil26(65), 2);
        assert_eq!(ceil26(64), 1);
        assert_eq!(pix_round26(96), 128);
        assert_eq!(pix_round26(95), 64);
    }

    /// Gemessen an pygame-ce 2.5.7: freesansbold.ttf, upem 1000,
    /// hhea ascender 800, descender -200, lineGap 90.
    #[test]
    fn freesansbold_metriken_bei_16() {
        let s = scale_for(16, 1000);
        assert_eq!(ceil26(mul_fix(800, s)), 13, "ascent");
        assert_eq!(ceil26(mul_fix(-200, s)), -3, "descent");
        assert_eq!(ceil26(mul_fix(800 - -200, s)), 16, "height");
        assert_eq!(ceil26(mul_fix(1090, s)), 18, "linesize");
    }
}
