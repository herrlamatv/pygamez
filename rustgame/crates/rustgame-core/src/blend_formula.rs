//! blend_formula.rs
//! =================
//! Die **vier** Rundungswege des normalen Blits.
//!
//! **Achtung:** diese Datei war einmal erzeugt (`tools/verify_alpha.py`),
//! wird jetzt aber **von Hand gepflegt** - das Skript kennt nur einen der
//! vier Wege und wuerde die anderen drei wegschreiben.
//!
//! Warum vier? Weil pygame fuer `surface.blit(..., special_flags=0)` nicht
//! *eine* Routine hat, sondern mehrere, und **jede rundet anders**. Welche
//! genommen wird, entscheidet eine Bedingungskette in
//! `src_c/surface.c:pgSurface_Blit`: sie schickt den Blit entweder in
//! pygames eigenes `src_c/alphablit.c` oder an SDLs `SDL_BlitSurface`, und
//! SDL waehlt darin nochmals zwischen mehreren Blittern.
//!
//! ```text
//! Quelle           Schluesselfarbe  Routine                        d=100 s=50 a=183
//! mit Alphakanal   nein             pygame alphablit_alpha                64
//! mit Alphakanal   ja               SDL_Blit_Slow                         63
//! ohne Alphakanal  ja               SDL BlitNtoNSurfaceAlphaKey           65
//! ohne Alphakanal  nein             SDL BlitRGBtoRGBSurfaceAlpha          64
//! ```
//!
//! Drei verschiedene Ergebnisse fuer dieselben drei Zahlen. Wer das aus
//! Messreihen zu rekonstruieren versucht, bekommt widerspruechliche Daten;
//! wer die C-Quelle liest, sieht vier getrennte Funktionen.
//!
//! Die Auswahlkette selbst steht in [`crate::surface`].

// ---------------------------------------------------------------------------
//  Weg 1: pygames eigener Alpha-Blit (alphablit.c -> ALPHA_BLEND)
// ---------------------------------------------------------------------------
//
// `src_c/surface.h`:
//
// ```c
// #define ALPHA_BLEND_COMP(sC, dC, sA) ((((sC - dC) * sA + sC) >> 8) + dC)
// ```
//
// Das ist **nicht** die Lehrbuchformel `(s*a + d*(255-a)) / 255`: bei
// dst=10, src=200, a=1 liefert die Lehrbuchformel 10, pygame aber 11. Der
// Grund ist die 256er-Skala (`>> 8` statt `/ 255`) und das `+ sC`, das die
// Raender exakt macht. Exhaustiv ueber alle 16.777.216 Kombinationen gegen
// pygame-ce 2.5.7 geprueft.

/// Ein Farbkanal auf pygames eigenem Weg (`ALPHA_BLEND_COMP`).
#[inline(always)]
pub fn blend_channel(dst: u8, src: u8, alpha: u8) -> u8 {
    // Umgeformt aus `(((s - d) * a + s) >> 8) + d`: die Quelle wiegt
    // (a + 1), das Ziel (256 - a). Beide Formen sind identisch, diese
    // kommt ohne Vorzeichen aus.
    let s = src as u32 * (alpha as u32 + 1);
    let d = dst as u32 * (256 - alpha as u32);
    ((s + d) >> 8) as u8
}

/// Das Ziel-Alpha auf pygames eigenem Weg.
///
/// `src_c/surface.h`: `dA = sA + dA - ((sA * dA) / 255)`.
#[inline(always)]
pub fn blend_alpha(dst_a: u8, src_a: u8) -> u8 {
    let d = dst_a as u32;
    let s = src_a as u32;
    (d + s - (d * s) / 255) as u8
}

// ---------------------------------------------------------------------------
//  Weg 2: SDLs langsamer Blitter (SDL_blit_slow.c)
// ---------------------------------------------------------------------------
//
// Dorthin geraet eine Quelle **mit** Alphakanal **und** Schluesselfarbe:
// `pgSurface_Blit` verlangt fuer den eigenen Weg ausdruecklich
// `!SDL_HasColorKey(src)`, gibt solche Blits also an SDL weiter - und SDLs
// `SDL_CalculateBlitA` kennt die Kombination `COLORKEY | BLEND` nicht,
// faellt somit auf `SDL_Blit_Slow` zurueck.
//
// ```c
// /* SDL_blit_slow.c */
// if (flags & SDL_COPY_MODULATE_ALPHA) srcA = (srcA * modulateA) / 255;
// if (srcA < 255) { srcR = (srcR * srcA) / 255; ... }   /* vormultiplizieren */
// case SDL_COPY_BLEND:
//     dstR = srcR + ((255 - srcA) * dstR) / 255;
//     dstA = srcA + ((255 - srcA) * dstA) / 255;
// ```
//
// Entscheidend ist, dass **jeder Summand einzeln** durch 255 geteilt wird -
// einmal beim Vormultiplizieren, einmal beim Mischen. Deshalb kommt hier
// 63 heraus, wo pygames eigener Weg 64 liefert. Ueber 2352 Kombinationen
// gegen echtes pygame geprueft: 0 Abweichungen.

/// Ein Farbkanal auf SDLs langsamem Weg (Quelle mit Alphakanal **und**
/// Schluesselfarbe).
#[inline(always)]
pub fn blend_channel_colorkey(dst: u8, src: u8, alpha: u8) -> u8 {
    let a = alpha as u32;
    ((src as u32 * a) / 255 + (dst as u32 * (255 - a)) / 255) as u8
}

/// Das Ziel-Alpha auf SDLs langsamem Weg.
#[inline(always)]
pub fn blend_alpha_colorkey(dst_a: u8, alpha: u8) -> u8 {
    let a = alpha as u32;
    (a + (dst_a as u32 * (255 - a)) / 255) as u8
}

// ---------------------------------------------------------------------------
//  Weg 3: SDLs Flaechen-Alpha-Blitter (SDL_blit_A.c)
// ---------------------------------------------------------------------------
//
// Eine Quelle **ohne** Alphakanal mit `set_alpha(a)`, a < 255, und **ohne**
// Schluesselfarbe landet in `BlitRGBtoRGBSurfaceAlpha`:
//
// ```c
// /* SDL_blit_A.c - 32 Bit auf 32 Bit, gleiche RGB-Masken */
// d1 = (d1 + ((s1 - d1) * alpha >> 8)) & 0x00ff00ff;
// d  = (d  + ((s  - d ) * alpha >> 8)) & 0x0000ff00;
// *dstp = d1 | d | 0xff000000;
// ```
//
// Also `>> 8` **ohne** das `+ s` aus pygames Formel - und die letzte Zeile
// erklaert die Messung "das Ziel-Alpha wird immer 255": der Blitter setzt
// es hart, ganz gleich, was vorher dort stand und was `set_alpha` sagt.

/// Ein Farbkanal auf SDLs Flaechen-Alpha-Weg (Quelle ohne Alphakanal, mit
/// `set_alpha`, ohne Schluesselfarbe).
///
/// Das Ziel-Alpha wird auf diesem Weg nicht gerechnet, sondern hart auf
/// 255 gesetzt - siehe `| 0xff000000` oben.
#[inline(always)]
pub fn blend_channel_flaechen_alpha(dst: u8, src: u8, alpha: u8) -> u8 {
    let d = dst as i32;
    let s = src as i32;
    // `>>` rundet in C wie in Rust auch bei negativen Zahlen ab.
    (d + (((s - d) * alpha as i32) >> 8)) as u8
}

// ---------------------------------------------------------------------------
//  Weg 4: SDLs Flaechen-Alpha-Blitter mit Schluesselfarbe (SDL_blit_A.c)
// ---------------------------------------------------------------------------
//
// Dieselbe Quelle, aber zusaetzlich mit Schluesselfarbe: jetzt greift
// `BlitNtoNSurfaceAlphaKey`, und das benutzt das Makro aus `SDL_blit.h`:
//
// ```c
// #define ALPHA_BLEND_RGBA(sR, sG, sB, sA, dR, dG, dB, dA)             \
//     do {                                                             \
//         dR = (Uint8)((((int)(sR - dR) * (int)sA) / 255) + dR);       \
//         ...                                                          \
//         dA = (Uint8)((int)sA + dA - ((int)sA * dA) / 255);           \
//     } while (0)
// ```
//
// `/ 255` statt `>> 8` - und weil die C-Division bei negativen Zahlen
// **zur Null hin** abschneidet statt abzurunden, ist das ein drittes,
// wieder eigenes Ergebnis: 65 statt 64 statt 63.

/// Ein Farbkanal auf SDLs Flaechen-Alpha-Weg **mit** Schluesselfarbe.
#[inline(always)]
pub fn blend_channel_flaechen_alpha_key(dst: u8, src: u8, alpha: u8) -> u8 {
    let d = dst as i32;
    let s = src as i32;
    // Rusts `/` auf `i32` schneidet zur Null hin ab - genau wie C. Das ist
    // hier kein Zufall, sondern der ganze Unterschied zu Weg 3.
    (d + ((s - d) * alpha as i32) / 255) as u8
}

/// Das Ziel-Alpha auf Weg 4: dieselbe Rechnung wie [`blend_alpha`], nur
/// mit dem Flaechen-Alpha als Quell-Alpha (`ALPHA_BLEND_RGBA`).
#[inline(always)]
pub fn blend_alpha_flaechen_alpha_key(dst_a: u8, alpha: u8) -> u8 {
    blend_alpha(dst_a, alpha)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Die Messwerte, an denen die naive Formel scheitert.
    #[test]
    fn stolpersteine_stimmen() {
        assert_eq!(blend_channel(10, 200, 1), 11, "die naive Formel sagt 10");
        assert_eq!(blend_channel(255, 0, 128), 127);
        assert_eq!(blend_channel(0, 255, 128), 128);
        assert_eq!(blend_channel(10, 200, 254), 199);
    }

    /// Die Randfaelle muessen sauber durchreichen.
    #[test]
    fn raender_sind_exakt() {
        for v in 0..=255u8 {
            assert_eq!(blend_channel(v, 99, 0), v, "alpha=0 laesst dst stehen");
            assert_eq!(blend_channel(99, v, 255), v, "alpha=255 nimmt src");
        }
    }

    #[test]
    fn ziel_alpha_rechnet_richtig() {
        assert_eq!(blend_alpha(0, 0), 0);
        assert_eq!(blend_alpha(255, 0), 255);
        assert_eq!(blend_alpha(0, 255), 255);
        assert_eq!(blend_alpha(200, 128), 228);
    }

    /// **Die Probe aufs Exempel:** dieselben drei Zahlen, vier Wege, drei
    /// verschiedene Ergebnisse. Jeder Wert ist an echtem pygame-ce 2.5.7
    /// gemessen (Ziel ohne Alphakanal, Quelle je nach Zeile).
    #[test]
    fn die_vier_wege_runden_verschieden() {
        // Ziel 100, Quelle 50, wirksames Alpha 183.
        assert_eq!(blend_channel(100, 50, 183), 64, "pygame alphablit_alpha");
        assert_eq!(
            blend_channel_colorkey(100, 50, 183),
            63,
            "SDL_Blit_Slow: Quelle mit Alphakanal und Schluesselfarbe"
        );
        assert_eq!(
            blend_channel_flaechen_alpha_key(100, 50, 183),
            65,
            "SDL BlitNtoNSurfaceAlphaKey: Quelle ohne Alphakanal, mit Schluesselfarbe"
        );
        assert_eq!(
            blend_channel_flaechen_alpha(100, 50, 183),
            64,
            "SDL BlitRGBtoRGBSurfaceAlpha: Quelle ohne Alphakanal"
        );

        // Ziel 17, Quelle 17, wirksames Alpha 241 - hier trennen sich die
        // Wege an einer anderen Stelle.
        assert_eq!(blend_channel(17, 17, 241), 17);
        assert_eq!(blend_channel_colorkey(17, 17, 241), 16);
        assert_eq!(blend_channel_flaechen_alpha_key(17, 17, 241), 17);
        assert_eq!(blend_channel_flaechen_alpha(17, 17, 241), 17);
    }

    #[test]
    fn die_raender_stimmen_auf_allen_wegen() {
        for d in [0u8, 1, 127, 254, 255] {
            for s in [0u8, 1, 127, 254, 255] {
                assert_eq!(blend_channel_colorkey(d, s, 0), d, "Alpha 0 = Ziel");
                assert_eq!(blend_channel_colorkey(d, s, 255), s, "Alpha 255 = Quelle");
                assert_eq!(blend_channel_flaechen_alpha_key(d, s, 0), d);
                assert_eq!(blend_channel_flaechen_alpha_key(d, s, 255), s);
                assert_eq!(blend_channel_flaechen_alpha(d, s, 0), d);
            }
            assert_eq!(blend_alpha_colorkey(d, 0), d);
            assert_eq!(blend_alpha_colorkey(d, 255), 255);
            assert_eq!(blend_alpha_flaechen_alpha_key(d, 255), 255);
        }
    }

    /// Weg 3 trifft bei Alpha 255 die Quelle **nicht** genau - deshalb
    /// waehlt SDL ihn nur, wenn wirklich ein Flaechen-Alpha unter 255
    /// gesetzt ist, und kopiert sonst.
    #[test]
    fn weg_drei_ist_bei_alpha_255_ungenau() {
        assert_eq!(blend_channel_flaechen_alpha(10, 200, 255), 199);
        assert_eq!(blend_channel(10, 200, 255), 200);
    }
}
