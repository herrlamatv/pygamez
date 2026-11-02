//! blend.rs
//! ========
//! Die Mischarten von pygame (`BLEND_*`) **und** die vier Rechenwege des
//! normalen Blits - Gegenstueck zu `src_c/alphablit.c` und zu den
//! Blittern, die SDL selbst mitbringt.
//!
//! Jede Art ist ein eigener Typ ohne Inhalt, der [`BlendOp`] umsetzt. Der
//! Blit waehlt **einmal pro Aufruf** aus (siehe `surface::Blitweg`), nicht
//! pro Pixel - genauso wie das C-Original, das seine Routine in
//! `SoftBlitPyGame` einmal per `switch` bestimmt und dann eine ganze
//! Funktion durchlaeuft.
//!
//! Die Grundformeln stehen in [`crate::blend_formula`]; dort ist auch
//! begruendet, warum es vier davon sind und nicht eine.

use crate::blend_formula::{
    blend_alpha, blend_alpha_colorkey, blend_alpha_flaechen_alpha_key, blend_channel,
    blend_channel_colorkey, blend_channel_flaechen_alpha, blend_channel_flaechen_alpha_key,
};

/// Die Kennzahlen aus `pygame.locals` - Werte wie im echten pygame.
pub mod flags {
    // Die `BLEND_*`-Zahlen sind **pygames**, nicht erfunden - `locals.py`
    // gibt sie so heraus, und `gen_konstanten.py` liest sie dort ab. Die
    // ersten fuenf haben darum zwei Namen: `BLEND_ADD` ist derselbe Wert wie
    // `BLEND_RGB_ADD`, weil pygame den kurzen Namen nie zurueckgezogen hat.
    /// Kanalweise addieren, Alpha bleibt - Zweitname von [`BLEND_RGB_ADD`].
    pub const BLEND_ADD: u32 = 1;
    /// Kanalweise abziehen - Zweitname von [`BLEND_RGB_SUB`].
    pub const BLEND_SUB: u32 = 2;
    /// Kanalweise multiplizieren - Zweitname von [`BLEND_RGB_MULT`].
    pub const BLEND_MULT: u32 = 3;
    /// Je Kanal der kleinere Wert - Zweitname von [`BLEND_RGB_MIN`].
    pub const BLEND_MIN: u32 = 4;
    /// Je Kanal der groessere Wert - Zweitname von [`BLEND_RGB_MAX`].
    pub const BLEND_MAX: u32 = 5;
    /// Addieren, **ohne** den Alphakanal.
    pub const BLEND_RGB_ADD: u32 = 1;
    /// Abziehen, ohne Alphakanal.
    pub const BLEND_RGB_SUB: u32 = 2;
    /// Multiplizieren, ohne Alphakanal.
    pub const BLEND_RGB_MULT: u32 = 3;
    /// Der kleinere Wert, ohne Alphakanal.
    pub const BLEND_RGB_MIN: u32 = 4;
    /// Der groessere Wert, ohne Alphakanal.
    pub const BLEND_RGB_MAX: u32 = 5;
    /// Addieren, **mit** dem Alphakanal.
    pub const BLEND_RGBA_ADD: u32 = 6;
    /// Abziehen, mit Alphakanal.
    pub const BLEND_RGBA_SUB: u32 = 7;
    /// Multiplizieren, mit Alphakanal.
    pub const BLEND_RGBA_MULT: u32 = 8;
    /// Der kleinere Wert, mit Alphakanal.
    pub const BLEND_RGBA_MIN: u32 = 9;
    /// Der groessere Wert, mit Alphakanal.
    pub const BLEND_RGBA_MAX: u32 = 10;
    /// Die Quelle ist **vormultipliziert** - ihr Alpha steckt schon in den
    /// Farbwerten, also wird nur noch addiert.
    pub const BLEND_PREMULTIPLIED: u32 = 11;
    /// SDL2s eigene Mischung statt pygames `ALPHA_BLEND_COMP`. Sie rundet
    /// anders; siehe [`crate::blend_formula`].
    pub const BLEND_ALPHA_SDL2: u32 = 12;
}

/// Zerlegt ein ARGB-Wort in seine vier Kanaele.
#[inline(always)]
pub fn unpack(v: u32) -> (u8, u8, u8, u8) {
    (
        (v >> 24) as u8, // a
        (v >> 16) as u8, // r
        (v >> 8) as u8,  // g
        v as u8,         // b
    )
}

/// Setzt vier Kanaele zu einem ARGB-Wort zusammen.
#[inline(always)]
pub fn pack(a: u8, r: u8, g: u8, b: u8) -> u32 {
    (a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | b as u32
}

/// Eine Mischrechnung: wie wird ein Quellpixel in ein Zielpixel gerechnet?
///
/// `src_alpha` ist das bereits verrechnete Quell-Alpha (Pixel-Alpha mal
/// Flaechen-Alpha); die Modi, die es nicht brauchen, ignorieren es.
pub trait BlendOp {
    /// Rechnet **ein** Pixel: `dst` und `src` als ARGB-Wort, `src_alpha`
    /// als das wirksame Alpha der Quelle (Pixelalpha mal Flaechenalpha).
    fn apply(dst: u32, src: u32, src_alpha: u8) -> u32;
}

// ---------------------------------------------------------------------------
//  Die vier Wege des normalen Blits (special_flags == 0)
// ---------------------------------------------------------------------------

/// **Weg 1** - pygames eigener Alpha-Blit (`alphablit.c`).
///
/// Zeilengetreu das Makro `ALPHA_BLEND` aus `src_c/surface.h`:
///
/// ```c
/// if (dA) {
///     dR = ALPHA_BLEND_COMP(sR, dR, sA);   /* usw. */
///     dA = sA + dA - ((sA * dA) / 255);
/// }
/// else {
///     dR = sR; dG = sG; dB = sB; dA = sA;
/// }
/// ```
///
/// Der `else`-Zweig ist kein Schoenheitsfehler, sondern messbar: auf ein
/// **voellig durchsichtiges** Ziel wird die Quelle *uebernommen* statt
/// hineingemischt. Ohne diesen Zweig weicht jeder Blit auf eine frische
/// `SRCALPHA`-Flaeche ab.
pub struct PygameAlphaBlend;
impl BlendOp for PygameAlphaBlend {
    #[inline(always)]
    fn apply(dst: u32, src: u32, src_alpha: u8) -> u32 {
        let (da, dr, dg, db) = unpack(dst);
        let (_, sr, sg, sb) = unpack(src);
        if da == 0 {
            return pack(src_alpha, sr, sg, sb);
        }
        pack(
            blend_alpha(da, src_alpha),
            blend_channel(dr, sr, src_alpha),
            blend_channel(dg, sg, src_alpha),
            blend_channel(db, sb, src_alpha),
        )
    }
}

/// **Weg 2** - SDLs langsamer Blitter (`SDL_Blit_Slow`).
///
/// Erreicht von einer Quelle **mit** Alphakanal **und** Schluesselfarbe.
/// Jeder Summand wird einzeln durch 255 geteilt.
pub struct SdlLangsam;
impl BlendOp for SdlLangsam {
    #[inline(always)]
    fn apply(dst: u32, src: u32, src_alpha: u8) -> u32 {
        let (da, dr, dg, db) = unpack(dst);
        let (_, sr, sg, sb) = unpack(src);
        pack(
            blend_alpha_colorkey(da, src_alpha),
            blend_channel_colorkey(dr, sr, src_alpha),
            blend_channel_colorkey(dg, sg, src_alpha),
            blend_channel_colorkey(db, sb, src_alpha),
        )
    }
}

/// **Weg 3** - SDLs Flaechen-Alpha-Blitter (`BlitRGBtoRGBSurfaceAlpha`).
///
/// Quelle ohne Alphakanal mit `set_alpha`, keine Schluesselfarbe. Das
/// Ziel-Alpha wird hart auf 255 gesetzt (`*dstp = d1 | d | 0xff000000`).
pub struct SdlFlaechenAlpha;
impl BlendOp for SdlFlaechenAlpha {
    #[inline(always)]
    fn apply(dst: u32, src: u32, src_alpha: u8) -> u32 {
        let (_, dr, dg, db) = unpack(dst);
        let (_, sr, sg, sb) = unpack(src);
        pack(
            255,
            blend_channel_flaechen_alpha(dr, sr, src_alpha),
            blend_channel_flaechen_alpha(dg, sg, src_alpha),
            blend_channel_flaechen_alpha(db, sb, src_alpha),
        )
    }
}

/// **Weg 4** - SDLs Flaechen-Alpha-Blitter **mit** Schluesselfarbe
/// (`BlitNtoNSurfaceAlphaKey`, Makro `ALPHA_BLEND_RGBA`).
pub struct SdlFlaechenAlphaKey;
impl BlendOp for SdlFlaechenAlphaKey {
    #[inline(always)]
    fn apply(dst: u32, src: u32, src_alpha: u8) -> u32 {
        let (da, dr, dg, db) = unpack(dst);
        let (_, sr, sg, sb) = unpack(src);
        pack(
            blend_alpha_flaechen_alpha_key(da, src_alpha),
            blend_channel_flaechen_alpha_key(dr, sr, src_alpha),
            blend_channel_flaechen_alpha_key(dg, sg, src_alpha),
            blend_channel_flaechen_alpha_key(db, sb, src_alpha),
        )
    }
}

/// **Weg 5** - SDLs `BlitRGBtoRGBPixelAlpha`, also `BLEND_ALPHA_SDL2`.
///
/// ## Warum keine kanalweise Formel passt
///
/// Hier rechnet SDL **Rot und Blau gemeinsam in einem Wort**:
///
/// ```c
/// /* SDL_blit_A.c, BlitRGBtoRGBPixelAlpha */
/// Uint32 alpha = s >> 24;
/// if (alpha) {
///     if (alpha == SDL_ALPHA_OPAQUE) {
///         *dstp = s;
///     } else {
///         d = *dstp;
///         s1 = s & 0xff00ff;            /* Rot und Blau zusammen */
///         d1 = d & 0xff00ff;
///         d1 = (d1 + ((s1 - d1) * alpha >> 8)) & 0xff00ff;
///         s &= 0xff00;                  /* Gruen allein */
///         d &= 0xff00;
///         d  = (d  + ((s  - d ) * alpha >> 8)) & 0xff00;
///         dalpha = alpha + (dalpha * (255 - alpha) >> 8);
///         *dstp = d1 | d | (dalpha << 24);
///     }
/// }
/// ```
///
/// Die Subtraktion `s1 - d1` laeuft ueber **das ganze 32-Bit-Wort**. Ist
/// das blaue Byte der Quelle kleiner als das des Ziels, borgt sie sich das
/// fehlende Bit beim roten - und die Maske `& 0xff00ff` raeumt es
/// hinterher weg. Das Ergebnis ist deshalb **nicht** dasselbe, als haette
/// man jeden Kanal einzeln gerechnet.
///
/// Das ist teuer gelernt. Ich hatte die Formel zuerst aus pygames
/// Erwartungstabelle **erschlossen** - `(s*sa)>>8 + (d*(255-sa))>>8` - und
/// sie stimmte auf vier nachgerechneten Faellen. Gegen die ganze Tabelle
/// waren es dann **40 von 64** Abweichungen, gegen echtes pygame 1270 von
/// 1280. Vier Treffer aus Zufall, weil bei ihnen kein Uebertrag anfiel.
/// Danach habe ich neun kanalweise Kandidaten mit acht Divisionskniffen
/// durchprobiert, 72 Paare - der beste lag noch bei 480 von 1728 falsch.
/// **Keine** kanalweise Formel kann es treffen, weil das Original keine
/// ist. Erst die Uebertragung stimmte: **0 von 108.000** bei den
/// Farbkanaelen, und mit dem Alphateil **0 von 5024** weiteren Proben.
///
/// ## Die drei Faelle
///
/// | Quell-Alpha | Ergebnis |
/// |---|---|
/// | 0 | Ziel bleibt, wie es war |
/// | 255 | **ganze Kopie**, samt Alpha |
/// | sonst | die Rechnung oben |
///
/// Der 255er-Fall ist die zweite Falle: SDLs eigener Quelltext hat dort an
/// manchen Stellen `(s & 0x00ffffff) | (*dstp & 0xff000000)`, behaelt also
/// das Ziel-Alpha. Gemessen behaelt pygame-ce 2.5.7 es **nicht** - es
/// kommt 255 heraus, das Alpha der Quelle. An den Farbkanaelen ist der
/// Unterschied unsichtbar, weshalb die 108.000 Farbproben ihn nicht
/// gefunden haben; erst die Alphaproben zeigten ihn, und dort war es
/// eindeutig: sieben Ausreisser, alle bei `sa == 255`.
///
/// ## Was hier **nicht** abgedeckt ist
///
/// `BLEND_ALPHA_SDL2` **zusammen mit** `set_alpha` auf der Quelle. Das ist
/// in SDL ein dritter Blitter (`SDL_COPY_MODULATE_ALPHA`), und gemessen
/// verhaelt er sich anders - mit Pixel-Alpha 128 und Flaechen-Alpha 128
/// gibt pygame `(3, 253, 210, 255)`, wobei das Ziel-Alpha auf 255 geht
/// statt auf 254. Diese Kombination kommt in pygames Testsuite nicht vor
/// und ist hier eine eingetragene Grenze; geraten wird sie nicht.
pub struct SdlPixelAlpha;
impl BlendOp for SdlPixelAlpha {
    #[inline(always)]
    fn apply(dst: u32, src: u32, _src_alpha: u8) -> u32 {
        // Das Alpha kommt aus dem **Quellpixel**, nicht aus `_src_alpha` -
        // SDL liest `s >> 24`. Eine Flaeche ohne Alphakanal traegt hier
        // schon 255 und faellt damit in den Kopierfall.
        let alpha = src >> 24;
        if alpha == 0 {
            return dst;
        }
        if alpha == 255 {
            return src;
        }
        let s1 = src & 0x00FF_00FF;
        let d1 = dst & 0x00FF_00FF;
        // `wrapping_sub` und `wrapping_mul`, weil der Uebertrag zwischen
        // den beiden Bytes gewollt ist - in C rechnet das ein `Uint32`
        // genauso, nur ohne dass man es hinschreiben muss.
        let rb = (d1.wrapping_add((s1.wrapping_sub(d1)).wrapping_mul(alpha) >> 8)) & 0x00FF_00FF;
        let s2 = src & 0x0000_FF00;
        let d2 = dst & 0x0000_FF00;
        let g = (d2.wrapping_add((s2.wrapping_sub(d2)).wrapping_mul(alpha) >> 8)) & 0x0000_FF00;
        let da = (dst >> 24) & 255;
        let na = (alpha + ((da * (255 - alpha)) >> 8)).min(255);
        rb | g | (na << 24)
    }
}

/// Einfaches Kopieren - kein Mischen (undurchsichtige Quelle).
pub struct Copy;
impl BlendOp for Copy {
    #[inline(always)]
    fn apply(_dst: u32, src: u32, _a: u8) -> u32 {
        src
    }
}

// ---------------------------------------------------------------------------
//  Die Mischarten (special_flags != 0)
// ---------------------------------------------------------------------------

/// Erzeugt die kanalweisen Modi. `$alpha` sagt, ob der Alphakanal
/// mitgerechnet wird (die `RGBA_`-Varianten) oder stehen bleibt.
macro_rules! kanal_modus {
    ($name:ident, $rechnung:expr, $alpha:expr) => {
        /// Ein Mischmodus. Nulltyp - er traegt nichts, er **ist** nur
        /// die Rechnung, damit der Uebersetzer sie in die Pixelschleife
        /// einsetzen kann statt eines Sprungs.
        pub struct $name;
        impl BlendOp for $name {
            #[inline(always)]
            fn apply(dst: u32, src: u32, _a: u8) -> u32 {
                let (da, dr, dg, db) = unpack(dst);
                let (sa, sr, sg, sb) = unpack(src);
                let f: fn(u8, u8) -> u8 = $rechnung;
                pack(
                    if $alpha { f(da, sa) } else { da },
                    f(dr, sr),
                    f(dg, sg),
                    f(db, sb),
                )
            }
        }
    };
}

// Die Rechnungen selbst - gemessen an echtem pygame:
//   dst=(100,150,200,255) src=(50,60,70,128)
//   ADD  -> (150,210,255)   SUB  -> (50,90,130)
//   MULT -> (20,36,55)      MIN  -> (50,60,70)   MAX -> (100,150,200)
//
// Achtung bei MULT: die naheliegende Rechnung `d * s / 255` ergibt fuer
// (100, 50) eine 19, pygame liefert aber 20. Richtig ist `(d*s + 255) >> 8`
// - exhaustiv ueber alle 65.536 Paare geprueft.
kanal_modus!(RgbAdd, |d, s| d.saturating_add(s), false);
kanal_modus!(RgbSub, |d, s| d.saturating_sub(s), false);
kanal_modus!(RgbMult, |d, s| ((((d as u32) * (s as u32)) + 255) >> 8) as u8, false);
kanal_modus!(RgbMin, |d, s| d.min(s), false);
kanal_modus!(RgbMax, |d, s| d.max(s), false);

kanal_modus!(RgbaAdd, |d, s| d.saturating_add(s), true);
kanal_modus!(RgbaSub, |d, s| d.saturating_sub(s), true);
kanal_modus!(RgbaMult, |d, s| ((((d as u32) * (s as u32)) + 255) >> 8) as u8, true);
kanal_modus!(RgbaMin, |d, s| d.min(s), true);
kanal_modus!(RgbaMax, |d, s| d.max(s), true);

/// Die Quelle bringt bereits mit Alpha multiplizierte Farben mit.
///
/// Uebertragen aus `ALPHA_BLEND_PREMULTIPLIED_COMP` in `src_c/surface.h`:
///
/// ```text
/// erg = sC + dC - ((dC + 1) * sA >> 8)
/// ```
///
/// Hier stand vorher die lehrbuchrichtige Fassung
/// `dC * (255 - sA) / 255 + sC`. Dieselbe Idee, andere Rundung - und
/// **jedes** Pixel lag um eins daneben:
///
/// ```text
/// Ziel (100,150,200,120), Quelle (50,60,70,80)
///   pygame / hier   (119, 163, 208, 163)
///   Lehrbuch        (118, 162, 207, 162)
/// ```
///
/// Der Ueberlauf ist **gesaettigt**, nicht abgeschnitten: `sC + dC` wird
/// bis 510 gross, und pygame liefert dort 255, nicht `510 & 0xFF == 254`.
/// Das ist nicht aus der Vorlage ablesbar - dort schreibt ein Makro in
/// ein `Uint8` - sondern nachgemessen:
///
/// ```text
/// Ziel (255,255,255,255), Quelle (255,255,255,0)  ->  (255,255,255,255)
/// ```
///
/// Das **Alpha rechnet mit** - mit sich selbst als `sA`, wie die
/// Farbkanaele auch.
pub struct Premultiplied;
impl BlendOp for Premultiplied {
    #[inline(always)]
    fn apply(dst: u32, src: u32, _a: u8) -> u32 {
        let (da, dr, dg, db) = unpack(dst);
        let (sa, sr, sg, sb) = unpack(src);
        let misch = |d: u8, s: u8| -> u8 {
            let v = s as i32 + d as i32 - (((d as i32 + 1) * sa as i32) >> 8);
            v.clamp(0, 255) as u8
        };
        pack(misch(da, sa), misch(dr, sr), misch(dg, sg), misch(db, sb))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Die Werte stammen aus einer Messung an echtem pygame-ce:
    /// `dst=(100,150,200,255)`, `src=(50,60,70,128)`.
    #[test]
    fn die_modi_rechnen_wie_pygame() {
        let dst = pack(255, 100, 150, 200);
        let src = pack(128, 50, 60, 70);

        let f = |v: u32| {
            let (a, r, g, b) = unpack(v);
            (r, g, b, a)
        };
        assert_eq!(f(RgbAdd::apply(dst, src, 128)), (150, 210, 255, 255));
        assert_eq!(f(RgbSub::apply(dst, src, 128)), (50, 90, 130, 255));
        assert_eq!(f(RgbMult::apply(dst, src, 128)), (20, 36, 55, 255));
        assert_eq!(f(RgbMin::apply(dst, src, 128)), (50, 60, 70, 255));
        assert_eq!(f(RgbMax::apply(dst, src, 128)), (100, 150, 200, 255));

        // Die RGBA-Varianten rechnen den Alphakanal mit.
        assert_eq!(f(RgbaMin::apply(dst, src, 128)), (50, 60, 70, 128));
        assert_eq!(f(RgbaMult::apply(dst, src, 128)), (20, 36, 55, 128));
        assert_eq!(f(RgbaSub::apply(dst, src, 128)), (50, 90, 130, 127));
        assert_eq!(f(RgbaMax::apply(dst, src, 128)), (100, 150, 200, 255));
        assert_eq!(f(RgbaAdd::apply(dst, src, 128)), (150, 210, 255, 255));
    }

    /// `BLEND_PREMULTIPLIED` - die Sollwerte an pygame gemessen.
    ///
    /// Der erste Fall ist der, der die Lehrbuchformel auffliegen liess:
    /// sie liefert `(118, 162, 207, 162)`, pygame eins mehr auf **jedem**
    /// Kanal. Die uebrigen decken den Ueberlauf ab - `sC + dC` wird dort
    /// bis 510 gross, und das Ergebnis ist gesaettigt, nicht
    /// abgeschnitten (`510 & 0xFF` waere 254).
    #[test]
    fn premultipliziert_rechnet_wie_pygame() {
        // (r, g, b, a) rein wie raus - so liest sich die Messung.
        let fall = |d: (u8, u8, u8, u8), s: (u8, u8, u8, u8)| {
            let v = Premultiplied::apply(
                pack(d.3, d.0, d.1, d.2),
                pack(s.3, s.0, s.1, s.2),
                s.3,
            );
            let (a, r, g, b) = unpack(v);
            (r, g, b, a)
        };
        assert_eq!(
            fall((100, 150, 200, 120), (50, 60, 70, 80)),
            (119, 163, 208, 163)
        );
        assert_eq!(
            fall((255, 255, 255, 255), (255, 255, 255, 0)),
            (255, 255, 255, 255),
            "gesaettigt - abgeschnitten waere 254"
        );
        assert_eq!(
            fall((200, 200, 200, 200), (200, 200, 200, 10)),
            (255, 255, 255, 203)
        );
        assert_eq!(
            fall((250, 250, 250, 250), (250, 250, 250, 1)),
            (255, 255, 255, 251)
        );
        assert_eq!(
            fall((128, 64, 32, 16), (200, 100, 50, 25)),
            (255, 158, 79, 40)
        );
        // Auf ein leeres Ziel wird die Quelle unveraendert uebernommen.
        assert_eq!(
            fall((0, 0, 0, 0), (255, 255, 255, 255)),
            (255, 255, 255, 255)
        );
    }

    #[test]
    fn alpha_mischen_trifft_die_messwerte() {
        // Auf undurchsichtiges Schwarz geblittet, Alpha 128.
        let dst = pack(255, 100, 150, 200);
        let src = pack(128, 50, 60, 70);
        let (_, r, g, b) = unpack(PygameAlphaBlend::apply(dst, src, 128));
        assert_eq!((r, g, b), (75, 105, 135), "gemessen an echtem pygame");
    }

    /// Auf ein voellig durchsichtiges Ziel wird **uebernommen**, nicht
    /// gemischt (der `else`-Zweig von `ALPHA_BLEND`).
    #[test]
    fn auf_durchsichtiges_ziel_wird_uebernommen() {
        let dst = pack(0, 200, 210, 220);
        let src = pack(90, 50, 60, 70);
        assert_eq!(
            PygameAlphaBlend::apply(dst, src, 90),
            pack(90, 50, 60, 70),
            "dA == 0 kopiert die Quelle"
        );
        // Mit einem Rest Deckkraft im Ziel wird wieder gemischt.
        let dst = pack(1, 200, 210, 220);
        assert_ne!(PygameAlphaBlend::apply(dst, src, 90), pack(90, 50, 60, 70));
    }

    /// Die vier Wege liefern fuer dieselbe Eingabe drei verschiedene
    /// Farben - der ganze Grund, warum die Auswahlkette noetig ist.
    #[test]
    fn die_vier_wege_sind_unterscheidbar() {
        let dst = pack(255, 100, 100, 100);
        let src = pack(255, 50, 50, 50);
        let rot = |v: u32| unpack(v).1;
        assert_eq!(rot(PygameAlphaBlend::apply(dst, src, 183)), 64);
        assert_eq!(rot(SdlLangsam::apply(dst, src, 183)), 63);
        assert_eq!(rot(SdlFlaechenAlphaKey::apply(dst, src, 183)), 65);
        assert_eq!(rot(SdlFlaechenAlpha::apply(dst, src, 183)), 64);
        // Weg 3 setzt das Ziel-Alpha hart auf 255, Weg 4 rechnet es.
        assert_eq!(unpack(SdlFlaechenAlpha::apply(pack(7, 0, 0, 0), src, 183)).0, 255);
        assert_eq!(unpack(SdlFlaechenAlphaKey::apply(pack(7, 0, 0, 0), src, 183)).0, 185);
    }

    #[test]
    fn packen_ist_umkehrbar() {
        for v in [0u32, 0xFFFFFFFF, 0x12345678, 0x00FF00FF] {
            let (a, r, g, b) = unpack(v);
            assert_eq!(pack(a, r, g, b), v);
        }
    }

    // ----- BLEND_ALPHA_SDL2: die Uebertragung, nicht die Nachrechnung ---

    /// Werte aus echtem pygame-ce 2.5.7, `special_flags=BLEND_ALPHA_SDL2`.
    ///
    /// Die Faelle sind so gewaehlt, dass sie den **Uebertrag** zwischen
    /// Rot und Blau treffen - genau den, an dem jede kanalweise Formel
    /// scheitert. Bei `s=(13,250,77)` auf `d=(0,255,255)` ist das blaue
    /// Byte der Quelle kleiner als das des Ziels, die Subtraktion borgt
    /// also beim roten.
    /// Werte aus echtem pygame-ce 2.5.7 (SDL 2.32.10), `BLEND_ALPHA_SDL2`.
    ///
    /// **Selbst gemessen, nicht aus pygames Testtabelle.** Das ist hier
    /// nicht Rechthaberei, sondern ein Fund: `surface_test.py` fuehrt in
    /// `results_expected` eine eigene Tabelle, und die weicht von dieser
    /// Maschine um bis zu **2** ab - fuer `(65,255,199,65)` auf
    /// `(0,255,255,255)` steht dort `(16, 253, 239, 253)`, gemessen kommt
    /// `(16, 255, 240, 254)`. Der Test besteht trotzdem, weil er mit
    /// `assertAlmostEqual(..., delta=4)` prueft: die Tabelle stammt von
    /// einer anderen SDL-Fassung, und pygame weiss das.
    ///
    /// Ich hatte die Tabelle zuerst als Sollwert genommen - und damit
    /// beinahe eine **richtige** Umsetzung "berichtigt", bis die
    /// Gegenmessung zeigte, dass echtes pygame genau das liefert, was hier
    /// herauskommt. Fremde Erwartungswerte sind Messwerte einer anderen
    /// Maschine, keine Vorschrift.
    ///
    /// Die Faelle treffen absichtlich den **Uebertrag** zwischen Rot und
    /// Blau: bei `s=(13,250,77)` auf `d=(0,255,255)` ist das blaue Byte der
    /// Quelle kleiner als das des Ziels, die Subtraktion borgt also beim
    /// roten.
    /// Quelle, Ziel, Ergebnis - je vier Kanaele. Mit Namen, weil clippy
    /// bei drei Vierertupeln in einer Zeile mit Recht widerspricht.
    type Probe = ((u8, u8, u8, u8), (u8, u8, u8, u8), (u8, u8, u8, u8));

    const SDL2_PROBEN: &[Probe] = &[
        // (Quelle, Ziel, Ergebnis)
        ((13, 250, 77, 1), (0, 255, 255, 255), (0, 254, 254, 254)),
        ((13, 250, 77, 128), (0, 255, 255, 255), (6, 252, 166, 254)),
        ((13, 250, 77, 255), (0, 255, 255, 255), (13, 250, 77, 255)),
        ((13, 250, 77, 0), (0, 255, 255, 255), (0, 255, 255, 255)),
        ((65, 255, 199, 65), (0, 255, 255, 255), (16, 255, 240, 254)),
        ((126, 255, 127, 126), (0, 255, 255, 255), (62, 255, 192, 254)),
        ((199, 255, 65, 199), (0, 255, 255, 255), (154, 255, 107, 254)),
        ((254, 255, 1, 254), (65, 255, 199, 199), (252, 255, 2, 254)),
        ((65, 255, 199, 65), (126, 255, 127, 127), (110, 255, 145, 159)),
        ((1, 2, 3, 7), (250, 240, 230, 9), (243, 233, 223, 15)),
        ((255, 0, 0, 64), (0, 0, 255, 192), (63, 0, 191, 207)),
    ];

    #[test]
    fn sdl2_alpha_wie_pygame() {
        for &(q, z, soll) in SDL2_PROBEN {
            let ergebnis = SdlPixelAlpha::apply(
                pack(z.3, z.0, z.1, z.2),
                pack(q.3, q.0, q.1, q.2),
                q.3,
            );
            let (a, r, g, b) = unpack(ergebnis);
            assert_eq!(
                (r, g, b, a),
                soll,
                "Quelle {q:?} auf Ziel {z:?}"
            );
        }
    }

    /// Der Uebertrag ist **nicht** wegzurechnen.
    ///
    /// Diese Probe steht hier, damit niemand die Uebertragung spaeter
    /// "aufraeumt", indem er sie kanalweise schreibt. Die naheliegende
    /// kanalweise Fassung `(s*a)>>8 + (d*(255-a))>>8` liegt bei genau dem
    /// Fall daneben, den der Uebertrag betrifft.
    #[test]
    fn sdl2_kanalweise_waere_falsch() {
        let (q, z) = ((13u32, 250u32, 77u32, 1u32), (0u32, 255u32, 255u32, 255u32));
        let kanalweise = |s: u32, d: u32, a: u32| {
            ((s * a) >> 8) + ((d * (255 - a)) >> 8)
        };
        // Blau: gemeinsam gerechnet kommt 254 heraus, kanalweise 253.
        assert_eq!(kanalweise(q.2, z.2, q.3), 253);
        let ergebnis = SdlPixelAlpha::apply(
            pack(z.3 as u8, z.0 as u8, z.1 as u8, z.2 as u8),
            pack(q.3 as u8, q.0 as u8, q.1 as u8, q.2 as u8),
            q.3 as u8,
        );
        assert_eq!(unpack(ergebnis).3, 254, "gemeinsam gerechnet sind es 254");
    }
}
