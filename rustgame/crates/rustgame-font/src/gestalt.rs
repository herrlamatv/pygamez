//! gestalt.rs
//! ==========
//! Eine Zeile setzen, wie SDL_ttf sie von HarfBuzz bekommt - ueber
//! `harfrust`.
//!
//! ## Warum
//!
//! SDL_ttf 2.24 setzt jede Zeile mit `hb_shape` (`TTF_Size_Internal`) und
//! legt jede Glyphe an `pos_x = x + x_offset`, `pos_y = y + ascent -
//! y_offset`; die Schreibmarke waechst um `x_advance`. Hier stand eine
//! eigene Rechnung - Zeichen ueber `cmap`, Vorschub, Paar-Kerning aus
//! GPOS -, und die traf 5951 von 6000 Saetzen aus PyGameZ' Sprachdateien.
//! Die 49 anderen, alle in Bahnschrift, zeigten, was sie nicht kannte:
//!
//! * `T-SPIN`, `MEGA-GEWINN!`: zwischen Grossbuchstaben hebt GPOS den
//!   Bindestrich um zwei Zeilen an (`y_offset`) - die eigene Rechnung
//!   kannte keine senkrechte Verschiebung,
//! * `BALL SAVED` ein Pixel schmaler - kontextuelles Kerning, das kein
//!   Paar-Lookup ist,
//! * das `$` hat bei pygame seinen Strich nur oben und unten: eine
//!   `FeatureVariations`-Ersetzung in GSUB.
//!
//! ## Wie `hb_ft` es macht - und hier
//!
//! * **Skala.** `hb_ft_font_changed` setzt `hb_font_set_scale` auf
//!   `(x_scale * upem + 2^15) >> 16` - FreeTypes 16.16-Faktor zurueck auf
//!   26.6 je Geviert. GPOS-Werte werden damit wie in HarfBuzz skaliert
//!   (`(v * mult + 32768) >> 16`, `mult = (scale << 16) / upem`), nicht mit
//!   `FT_MulFix`.
//! * **Vorschuebe** kommen ueber `FT_Get_Advance` mit denselben Ladeschaltern
//!   wie die Glyphen - also **gehintet**. Hier ueber [`harfrust::FontFuncs`]
//!   aus `Sicht::vorschub26`.
//! * **`FeatureVariations` in der Grundeinstellung.** HarfBuzz wertet die
//!   Bedingungen auch dann aus, wenn keine Koordinaten gesetzt sind
//!   (fehlende Achsen zaehlen als 0); `read-fonts` liefert fuer die
//!   Grundeinstellung grundsaetzlich **keine** Auswahl. Bahnschrifts
//!   einziger Eintrag gilt beiderseits der 0 - also auch dort, und daher
//!   das kurze `$`. Siehe [`auswahl_bei_null`].
//!
//! Wie SDL_ttf: Richtung links nach rechts, Schrift und Sprache geraten
//! (`hb_buffer_guess_segment_properties`), Nutzermerkmal nur `kern`.

use std::cell::OnceCell;

use skrifa::raw::tables::layout::{Condition, FeatureVariations};
use skrifa::raw::TableProvider;

/// Eine gesetzte Glyphe - in 26.6, wie `hb_glyph_position_t` bei `hb_ft`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Lage {
    pub(crate) gid: u16,
    pub(crate) x_advance: i32,
    pub(crate) y_advance: i32,
    pub(crate) x_offset: i32,
    pub(crate) y_offset: i32,
}

/// Die geladene Schrift fuer die Formung - einmal je Datei.
#[derive(Default)]
pub(crate) struct Gestalter {
    schrift: OnceCell<Option<harfrust::Font>>,
}

impl std::fmt::Debug for Gestalter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Gestalter")
            .field("geladen", &self.schrift.get().map(|s| s.is_some()))
            .finish()
    }
}

/// `hb_ft`s Vorschub: FreeTypes gehinteter Vorschub in 26.6.
struct Vorschuebe<'a>(&'a dyn Fn(u16) -> i32);

impl harfrust::FontFuncs for Vorschuebe<'_> {
    fn glyph_h_advance(&self, _font: &harfrust::ShaperFont, glyph: harfrust::GlyphId) -> i32 {
        (self.0)(glyph.to_u32() as u16)
    }
}

/// Welche `FeatureVariations`-Zeile HarfBuzz bei **lauter Nullen** nimmt.
///
/// `FeatureVariations::find_index` in HarfBuzz: die erste Zeile, deren
/// Bedingungen alle gelten; eine fehlende Bedingungsmenge gilt immer.
/// Format 1 (`axisIndex`, `min <= coord <= max`) ist das einzige, das die
/// HarfBuzz-Fassung in pygame kennt - jedes andere Format gilt als nicht
/// erfuellt.
fn auswahl_bei_null(fv: Option<FeatureVariations<'_>>) -> Option<u32> {
    let fv = fv?;
    for (i, zeile) in fv.feature_variation_records().iter().enumerate() {
        let gilt = match zeile.condition_set(fv.offset_data()) {
            None => true,
            Some(Ok(menge)) => menge.conditions().iter().all(|c| match c {
                Ok(Condition::Format1AxisRange(b)) => {
                    b.filter_range_min_value().to_bits() <= 0
                        && 0 <= b.filter_range_max_value().to_bits()
                }
                _ => false,
            }),
            Some(Err(_)) => false,
        };
        if gilt {
            return Some(i as u32);
        }
    }
    None
}

/// Die Schrift, an der `harfrust` dieselbe `FeatureVariations`-Auswahl
/// trifft wie HarfBuzz in der Grundeinstellung.
///
/// `read-fonts` faltet lauter Nullen zur Grundeinstellung zusammen und
/// waehlt dort nichts. Also wird die **kleinstmoegliche** Verschiebung
/// gesucht (1/16384 auf einer Achse), an der `read-fonts` genau die Zeile
/// waehlt, die HarfBuzz bei null nimmt - fuer GSUB **und** GPOS. Gibt es
/// keine solche, bleibt es bei der Grundeinstellung. Was die Verschiebung
/// sonst bewirkt, ist hier ohne Belang: Vorschuebe kommen ueber
/// [`Vorschuebe`], und Deltas aus `ItemVariationStore` sind bei 1/16384
/// um Groessenordnungen unter einer Schrifteinheit.
fn mit_grundauswahl(bytes: &[u8], index: u32, basis: harfrust::Font) -> harfrust::Font {
    let Ok(roh) = skrifa::raw::FontRef::from_index(bytes, index) else {
        return basis;
    };
    let soll = [
        roh.gsub()
            .ok()
            .and_then(|t| auswahl_bei_null(t.feature_variations().and_then(|f| f.ok()))),
        roh.gpos()
            .ok()
            .and_then(|t| auswahl_bei_null(t.feature_variations().and_then(|f| f.ok()))),
    ];
    if soll == [None, None] {
        return basis;
    }
    let achsen = roh.fvar().map(|f| f.axis_count() as usize).unwrap_or(0);
    for achse in 0..achsen {
        for schritt in [1i16, -1] {
            let mut koord = vec![harfrust::font::NormalizedCoord::default(); achsen];
            koord[achse] = harfrust::font::NormalizedCoord::from_bits(schritt);
            let kandidat = basis.instance_builder().normalized_coords(koord).build();
            let fv = kandidat.feature_variations();
            if [fv.gsub, fv.gpos] == soll {
                return kandidat;
            }
        }
    }
    basis
}

impl Gestalter {
    /// Setzt `text`: Glyphen und ihre Lagen in 26.6.
    ///
    /// `skala` ist `hb_font_set_scale` (siehe Modulkopf), `vorschub` der
    /// gehintete Vorschub einer Glyphe in 26.6. `None`, wenn die Datei sich
    /// nicht formen laesst - dann setzt der Aufrufer selbst.
    pub(crate) fn formen(
        &self,
        bytes: &[u8],
        index: u32,
        text: &str,
        skala: i32,
        kerning: bool,
        vorschub: &dyn Fn(u16) -> i32,
    ) -> Option<Vec<Lage>> {
        let schrift = self
            .schrift
            .get_or_init(|| {
                harfrust::Font::new(bytes.to_vec(), index)
                    .map(|f| mit_grundauswahl(bytes, index, f))
            })
            .as_ref()?;
        let funktionen = Vorschuebe(vorschub);
        let former = harfrust::ShaperFont::new(schrift)
            .with_scale(skala)
            .with_font_funcs(Some(&funktionen));
        let mut puffer = harfrust::Buffer::new();
        puffer.push_str(text);
        puffer.set_direction(harfrust::Direction::LeftToRight);
        puffer.guess_segment_properties();
        let merkmale = [harfrust::Feature::new(
            harfrust::Tag::new(b"kern"),
            u32::from(kerning),
            ..,
        )];
        harfrust::shape(
            &former,
            &mut puffer,
            harfrust::ShapeOptions::new().features(&merkmale),
        )
        .ok()?;
        Some(
            puffer
                .glyph_infos()
                .iter()
                .zip(puffer.glyph_positions())
                .map(|(g, p)| Lage {
                    gid: g.glyph_id as u16,
                    x_advance: p.x_advance,
                    y_advance: p.y_advance,
                    x_offset: p.x_offset,
                    y_offset: p.y_offset,
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ohne_tabelle_keine_auswahl() {
        assert_eq!(auswahl_bei_null(None), None);
    }
}
