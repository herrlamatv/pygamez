//! face.rs
//! =======
//! Die **Schriftdatei**: alles, was aus den Tabellen kommt und nicht aus der
//! Rasterung.
//!
//! Die Aufteilung ist nicht zufaellig. Pixelgleichheit mit FreeType ist mit
//! einem anderen Rasterer nicht zu haben - die Zahlen aus der Datei dagegen
//! schon, und zwar auf den Wert genau. Also werden sie hier geholt und
//! nirgendwo geschaetzt:
//!
//! * `hhea` liefert ascender/descender/lineGap -> `get_ascent` und Co.
//! * `hmtx` liefert die Vorschuebe -> Textbreite.
//! * `post` liefert Unterstrich-Lage und -Dicke.
//! * `GPOS` (sonst `kern`) liefert das Kerning.
//!
//! Gerastert wird mit fontdue; das steht in [`crate::glyph`].
//!
//! ## Warum GPOS und nicht nur `kern`
//!
//! SDL_ttf setzt das Layout mit HarfBuzz; HarfBuzz nimmt die alte
//! `kern`-Tabelle **nur dann**, wenn die Schrift gar keine GPOS-Tabelle hat.
//! Ohne diese Unterscheidung liegt die Breite bei modernen Windows-Schriften
//! daneben - gemessen bei Bahnschrift 21 Pixel auf einem laengeren Satz,
//! bei Segoe UI 8. Mit ihr: 0.

use std::rc::Rc;

use ttf_parser::gpos::PairAdjustment;
use ttf_parser::{Face as TtfFace, GlyphId, RasterImageFormat, Tag};

use crate::fixed;

/// Eine geladene Schriftdatei.
///
/// Die Rohbytes bleiben liegen, weil `ttf_parser::Face` nur ein Sichtfenster
/// darauf ist. Mehrere [`crate::Font`] (verschiedene Groessen derselben
/// Datei) teilen sich eine `Datei` ueber `Rc`.
pub struct Datei {
    bytes: Rc<Vec<u8>>,
    /// Index in einer Sammlung (`.ttc`), sonst 0.
    index: u32,
    /// Hat die Datei Umrisse? Bei einer Bitmapschrift nicht.
    umrisse: bool,
    /// fontdue - **erst bei Bedarf** angelegt, siehe [`Datei::raster`].
    raster: std::cell::OnceCell<Option<fontdue::Font>>,
    /// FreeTypes Sicht auf die `kern`-Tabelle, einmal gelesen - siehe
    /// [`kern_laden`].
    kern_bits: std::cell::OnceCell<(u32, u32, u32)>,
    /// Die Groesse, fuer die fontdue seine Geometrie aufbereitet.
    raster_groesse: i32,
    /// Die Pixel, wenn die Datei gar keine Umrisse hat (BDF).
    ///
    /// Siehe [`crate::bdf`]: eine solche Datei zaehlt ihre Zeichen als
    /// Pixelmuster auf. Alles darunter - Vorschub, Oberlaenge, Rasterung -
    /// kommt dann von hier statt aus `hmtx`, `hhea` und fontdue.
    pub(crate) punkte: Option<crate::bdf::Bitmapschrift>,
    pub(crate) units_per_em: u16,
    /// `hhea`-Werte (ersatzweise aus `OS/2`, siehe [`Datei::laden`]).
    pub(crate) ascender: i16,
    pub(crate) descender: i16,
    pub(crate) line_gap: i16,
    pub(crate) underline_position: i16,
    pub(crate) underline_thickness: i16,
    /// Die Lookups des `kern`-Merkmals in GPOS. Leer = alte `kern`-Tabelle.
    kern_lookups: Vec<u16>,
    /// Hat die Schrift ueberhaupt eine brauchbare GPOS-Tabelle?
    hat_gpos: bool,
    /// FreeTypes Weg zum Glyphenbild: gehinteter Umriss, `ftgrays`.
    /// Siehe [`crate::hinting`].
    pub(crate) hinter: crate::hinting::Hinter,
    /// HarfBuzz' Glyphenfolge (GSUB). Siehe [`crate::gestalt`].
    pub(crate) gestalter: crate::gestalt::Gestalter,
    /// FreeTypes `FT_STYLE_FLAG_BOLD` - der Schnitt **ist** fett.
    pub(crate) schnitt_fett: bool,
    /// FreeTypes `FT_STYLE_FLAG_ITALIC` - der Schnitt **ist** kursiv.
    pub(crate) schnitt_kursiv: bool,
}

/// `style_flags` wie in FreeTypes `sfobjs.c`: aus `OS/2.fsSelection`
/// (Bit 9 schraeg **oder** Bit 0 kursiv, Bit 5 fett) - und nur ohne
/// OS/2-Tabelle aus `head.macStyle` (Bit 0 fett, Bit 1 kursiv).
///
/// Roh gelesen, nicht ueber `ttf-parser`: dessen `style()` kennt Bit 9
/// erst ab OS/2-Fassung 4, FreeType immer.
fn schnitt_stil(tt: &TtfFace<'_>) -> (bool, bool) {
    let roh = tt.raw_face();
    if let Some(os2) = roh.table(Tag::from_bytes(b"OS/2")) {
        if os2.len() >= 64 {
            let fs = u16::from_be_bytes([os2[62], os2[63]]);
            return (fs & 32 != 0, fs & 512 != 0 || fs & 1 != 0);
        }
    }
    if let Some(head) = roh.table(Tag::from_bytes(b"head")) {
        if head.len() >= 46 {
            let m = u16::from_be_bytes([head[44], head[45]]);
            return (m & 1 != 0, m & 2 != 0);
        }
    }
    (false, false)
}

impl std::fmt::Debug for Datei {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Datei")
            .field("bytes", &self.bytes.len())
            .field("index", &self.index)
            .field("units_per_em", &self.units_per_em)
            .finish()
    }
}

impl Datei {
    /// Liest eine Schriftdatei aus dem Speicher.
    pub fn laden(bytes: Vec<u8>, index: u32, punkte: i32) -> Result<Datei, crate::FontError> {
        // **Erst die Pixel fragen.** Eine BDF-Datei faengt mit dem Wort
        // `STARTFONT` an und hat kein Tabellenverzeichnis; `TtfFace::parse`
        // lehnt sie ab, und der Aufrufer saehe nur "keine brauchbare
        // Schriftdatei". Genau daran scheiterte `freetype_test.setUpClass`
        // und mit ihm 43 Tests, die mit Bitmapschriften nichts zu tun haben.
        if let Some(b) = crate::bdf::lesen(&bytes) {
            return Ok(Datei::aus_punkten(Rc::new(bytes), index, b));
        }
        let bytes = Rc::new(bytes);
        let tt = TtfFace::parse(&bytes[..], index).map_err(|_| crate::FontError::Unlesbar)?;

        let upem = tt.units_per_em();
        if upem == 0 {
            return Err(crate::FontError::Unlesbar);
        }

        // FreeType 2.11.1 (`sfnt_load_face` in sfobjs.c): steht in
        // `OS/2.fsSelection` Bit 7 (`USE_TYPO_METRICS`), gelten die
        // Typo-Werte - **ohne** Blick auf die Fassung der Tabelle. Sonst
        // die hhea-Werte, und nur wenn dort beide null stehen, OS/2.
        //
        // Hier stand bis Runde 19, SDL_ttf sehe die Typo-Werte nie. Gemessen
        // an `test_sans.ttf` mit gesetztem Bit und Typo-Werten
        // 1500/-700/300: pygame meldet `get_ascent()` 15 statt 19 und
        // `get_linesize()` 25 statt 23. Unter den Windows-Schriften hat
        // keine das Bit **und** abweichende Werte - darum fiel es erst an
        // `PyGameMono.otf` auf.
        let hhea = tt.tables().hhea;
        let (mut asc, mut desc, mut gap) = (hhea.ascender, hhea.descender, hhea.line_gap);
        let typo_bit = tt
            .raw_face()
            .table(ttf_parser::Tag::from_bytes(b"OS/2"))
            .and_then(|d| d.get(62..64))
            .is_some_and(|b| u16::from_be_bytes([b[0], b[1]]) & 0x80 != 0);
        if let (true, Some(os2)) = (typo_bit, tt.tables().os2) {
            asc = os2.typographic_ascender();
            desc = os2.typographic_descender();
            gap = os2.typographic_line_gap();
        } else if asc == 0 && desc == 0 {
            if let Some(os2) = tt.tables().os2 {
                let (ta, td, tg) = (
                    os2.typographic_ascender(),
                    os2.typographic_descender(),
                    os2.typographic_line_gap(),
                );
                if ta != 0 || td != 0 {
                    asc = ta;
                    desc = td;
                    gap = tg;
                } else {
                    asc = os2.windows_ascender();
                    desc = -os2.windows_descender();
                    gap = 0;
                }
            }
        }

        // FreeType reicht die post-Tabelle nicht durch, sondern rueckt die
        // Unterstrich-Lage um die halbe Strichdicke nach unten (sfobjs.c):
        //     underline_position = post.position - post.thickness / 2
        // Ohne diese eine Zeile liegt `get_height()` mit gesetztem
        // Unterstrich daneben - bei freesansbold in Groesse 48 um 3 Pixel.
        let unter = tt.underline_metrics();
        let (up, ut) = match unter {
            Some(m) => (m.position - m.thickness / 2, m.thickness),
            None => (0, 0),
        };

        let (hat_gpos, kern_lookups) = kern_lookups_von(&tt);
        let (schnitt_fett, schnitt_kursiv) = schnitt_stil(&tt);

        Ok(Datei {
            bytes,
            index,
            umrisse: true,
            raster: std::cell::OnceCell::new(),
            kern_bits: std::cell::OnceCell::new(),
            raster_groesse: punkte.max(1),
            punkte: None,
            units_per_em: upem,
            ascender: asc,
            descender: desc,
            line_gap: gap,
            underline_position: up,
            underline_thickness: ut,
            kern_lookups,
            hat_gpos,
            hinter: crate::hinting::Hinter::default(),
            gestalter: crate::gestalt::Gestalter::default(),
            schnitt_fett,
            schnitt_kursiv,
        })
    }

    /// Eine Zeile, wie SDL_ttf sie von HarfBuzz bekommt - siehe
    /// [`crate::gestalt`].
    ///
    /// `scale` ist FreeTypes 16.16-Faktor (`size->metrics.x_scale`), daraus
    /// wird HarfBuzz' Skala wie in `hb_ft_font_changed`; `vorschub` liefert
    /// den gehinteten Vorschub einer Glyphe. `None` bei einer Bitmapschrift
    /// oder wenn die Formung scheitert - dann setzt der Aufrufer selbst.
    pub(crate) fn formen(
        &self,
        text: &str,
        scale: i64,
        vorschub: &dyn Fn(u16) -> i32,
    ) -> Option<Vec<crate::gestalt::Lage>> {
        // Eine Bitmapschrift hat keine Tabellen zum Formen.
        if !self.umrisse {
            return None;
        }
        let skala = ((scale * i64::from(self.units_per_em) + (1 << 15)) >> 16) as i32;
        self.gestalter
            .formen(&self.bytes, self.index, text, skala, true, vorschub)
    }

    /// fontdue - der Ausweichweg fuer alles, was FreeTypes Weg hier noch
    /// nicht nimmt (einfarbig, Kontur, kursiv).
    ///
    /// **Erst bei Bedarf angelegt.** `fontdue::Font::from_bytes` bereitet
    /// beim Laden die Geometrie **jeder** Glyphe der Datei auf. Gemessen in
    /// Runde 18: 32 neue Schriftgroessen kosteten hier 142 ms, bei pygame
    /// 16 ms - und seit Bild und Lage von FreeTypes Weg kommen, braucht der
    /// Normalfall fontdue gar nicht mehr.
    pub(crate) fn raster(&self) -> Option<&fontdue::Font> {
        if !self.umrisse {
            return None;
        }
        self.raster
            .get_or_init(|| {
                fontdue::Font::from_bytes(
                    self.bytes.as_slice(),
                    fontdue::FontSettings {
                        collection_index: self.index,
                        scale: self.raster_groesse as f32,
                        load_substitutions: false,
                    },
                )
                .ok()
            })
            .as_ref()
    }

    /// Das Glyphenbild auf FreeTypes Weg: gehinteter Umriss (`skrifa`),
    /// gerastert von [`crate::ftgrays`] - Bild **und** Metriken, denn
    /// SDL_ttf nimmt `sz_left/top/width/rows` aus genau diesem Kasten
    /// (`bitmap_left`, `bitmap_top`, `bitmap.width`, `bitmap.rows`, von
    /// `ft_glyphslot_preset_bitmap` schon beim Laden gesetzt).
    ///
    /// `None` bei einer Bitmapschrift oder wenn das Hinting scheitert -
    /// dann bleibt es beim alten Weg.
    ///
    /// Mit `schraeg` kommt ein zweites Bild dazu: SDL_ttf schert fuer
    /// `TTF_STYLE_ITALIC` den **gehinteten** Umriss mit `FT_Outline_Transform`
    /// (`xx = yy = 1 << 16`, `xy = GLYPH_ITALICS`) und rastert den - die
    /// Metriken bleiben die des ungescherten Kastens. Bis Runde 18 wurde
    /// hier stattdessen das fertige Bild zeilenweise verschoben.
    pub(crate) fn gehintet(
        &self,
        ppem: u32,
        gid: u16,
        schraeg: bool,
    ) -> Option<(crate::ftgrays::Graubild, Option<crate::ftgrays::Graubild>)> {
        if !self.umrisse || ppem == 0 {
            return None;
        }
        let u = self.hinter.umriss(&self.bytes, self.index, ppem, gid)?;
        let gerade = crate::ftgrays::rendern(&u)?;
        if !schraeg {
            return Some((gerade, None));
        }
        let mut geschert = u;
        for p in &mut geschert.punkte {
            // `FT_Outline_Transform`: x' = MulFix(x, xx) + MulFix(y, xy),
            // y' = MulFix(x, yx) + MulFix(y, yy); mit xx = yy = 1.0 bleibt
            // MulFix(v, 1 << 16) exakt v, und yx = 0.
            p.0 += fixed::mul_fix(p.1, crate::glyph::GLYPH_ITALICS);
        }
        let schief = crate::ftgrays::rendern(&geschert)?;
        Some((gerade, Some(schief)))
    }

    /// Ein Glyphenbild, das SDL_ttf **nach** dem Laden noch umbaut: einfarbig
    /// (`FT_RENDER_MODE_MONO`, also [`crate::ftraster`]) und/oder mit
    /// Kontur (`FT_Glyph_Stroke`, also [`crate::ftstroke`]).
    ///
    /// Die Reihenfolge ist die aus `Load_Glyph`: gehinteter Umriss, bei
    /// kursiv geschert, bei `outline > 0` umrandet (Radius `outline * 64`,
    /// runde Enden und Ecken), dann gerastert. Die **Masse** kommen aber
    /// aus dem NORMAL-Kasten des unveraenderten Umrisses - den setzt
    /// `FT_Load_Glyph`, bevor irgendetwas davon geschieht.
    ///
    /// Ein umrandeter Umriss ist ein neuer (`FT_Outline_New`): er rastert
    /// ohne `FT_OUTLINE_HIGH_PRECISION`, ohne Dropout-Flaggen und ohne
    /// Scan-Marken.
    pub(crate) fn gehintet_fertig(
        &self,
        ppem: u32,
        gid: u16,
        schraeg: bool,
        mono: bool,
        outline: i32,
    ) -> Option<crate::glyph::FtBild> {
        if !self.umrisse || ppem == 0 {
            return None;
        }
        let mut u = self.hinter.umriss(&self.bytes, self.index, ppem, gid)?;
        let kasten = crate::ftgrays::kasten_normal(&u);
        let mut fl = if mono && outline == 0 {
            self.hinter.rasterflaggen(&self.bytes, self.index, (ppem << 6, ppem << 6), gid)?
        } else {
            crate::ftraster::Rasterflaggen::default()
        };
        if schraeg {
            for p in &mut u.punkte {
                p.0 += fixed::mul_fix(p.1, crate::glyph::GLYPH_ITALICS);
            }
        }
        if outline > 0 {
            u = crate::ftstroke::konturieren(&u, i64::from(outline) * 64)?;
            fl = crate::ftraster::Rasterflaggen::default();
        }
        let (left, top, breite, hoehe, deckung) = if mono {
            let b = crate::ftraster::rendern(&u, fl)?;
            (b.left, b.top, b.breite, b.hoehe, b.deckung)
        } else {
            let b = crate::ftgrays::rendern(&u)?;
            (b.left, b.top, b.breite, b.hoehe, b.deckung)
        };
        Some(crate::glyph::FtBild {
            kasten,
            left,
            top,
            breite,
            hoehe,
            deckung,
            mono,
        })
    }

    /// Ob die Datei Umrisse hat (keine reine Bitmapschrift).
    pub(crate) fn hat_umrisse(&self) -> bool {
        self.umrisse
    }

    /// Der gehintete Umriss samt Rasterflaggen und `horiAdvance` vor dem
    /// Rasterschnitt (siehe [`crate::hinting::Hinter::umriss_mit_vorschub`])
    /// - fuer [`crate::font::Font::ft_zeile`].
    ///
    /// `groesse26` ist die Nenngroesse in 26.6, Breite und Hoehe
    /// (`pygame.freetype` kennt Bruchgroessen und zwei Achsen).
    /// Siehe [`crate::hinting::Hinter::gehinteter_vorschub`].
    pub(crate) fn gehinteter_vorschub(&self, groesse26: (u32, u32), gid: u16) -> Option<i64> {
        if !self.umrisse {
            return None;
        }
        self.hinter
            .gehinteter_vorschub(&self.bytes, self.index, groesse26, gid)
    }

    /// Siehe [`crate::hinting::Hinter::gescheitert`] - `false` fuer
    /// Bitmapschriften.
    pub(crate) fn hinting_gescheitert(&self, groesse26: (u32, u32)) -> bool {
        self.umrisse && self.hinter.gescheitert(&self.bytes, self.index, groesse26)
    }

    pub(crate) fn ft_umriss(
        &self,
        groesse26: (u32, u32),
        gid: u16,
        mono: bool,
    ) -> Option<(crate::ftgrays::Umriss, crate::ftraster::Rasterflaggen, Option<i64>)> {
        if !self.umrisse || groesse26.0 < 32 || groesse26.1 < 32 {
            return None;
        }
        let (u, vorschub) =
            self.hinter.umriss_mit_vorschub(&self.bytes, self.index, groesse26, gid)?;
        let fl = if mono {
            self.hinter.rasterflaggen(&self.bytes, self.index, groesse26, gid)?
        } else {
            crate::ftraster::Rasterflaggen::default()
        };
        Some((u, fl, vorschub))
    }

    /// Eine Datei, die aus Pixeln besteht.
    ///
    /// Die Uebersetzung ist die aus dem Kopf von [`crate::bdf`]: ein
    /// Geviert von `ppem` Einheiten macht aus jeder Schrifteinheit genau
    /// ein Pixel, sobald die verlangte Groesse die eingebaute ist. Damit
    /// rechnet [`crate::font`] unveraendert weiter.
    fn aus_punkten(bytes: Rc<Vec<u8>>, index: u32, b: crate::bdf::Bitmapschrift) -> Datei {
        let upem = b.ppem.1.max(1) as u16;
        let (asc, desc) = (b.ascent as i16, -(b.descent as i16));
        let (up, ut) = (b.unterstrich_lage, b.unterstrich_dicke);
        Datei {
            bytes,
            index,
            umrisse: false,
            raster: std::cell::OnceCell::new(),
            kern_bits: std::cell::OnceCell::new(),
            raster_groesse: 0,
            units_per_em: upem,
            ascender: asc,
            descender: desc,
            line_gap: 0,
            underline_position: up,
            underline_thickness: ut,
            kern_lookups: Vec::new(),
            hat_gpos: false,
            punkte: Some(b),
            hinter: crate::hinting::Hinter::default(),
            gestalter: crate::gestalt::Gestalter::default(),
            // BDF kennt kein `fsSelection`; FreeTypes BDF-Treiber liest
            // `WEIGHT_NAME`/`SLANT` - hier bleibt es beim normalen Schnitt.
            schnitt_fett: false,
            schnitt_kursiv: false,
        }
    }

    /// Das eingebaute Pixelbild eines Zeichens - oder `None`.
    ///
    /// ## Was eine Strike ist, und warum sie zaehlt
    ///
    /// Eine Schrift mit Umrissen darf daneben **fertige Pixelbilder** fuer
    /// bestimmte Groessen mitbringen; die Tabellen heissen `EBLC`/`EBDT`
    /// (schwarzweiss), `CBLC`/`CBDT` (farbig) oder `bloc`/`bdat` (Apple).
    /// Der Sinn: bei kleinen Groessen sieht ein von Hand gesetztes Bild
    /// besser aus als jede Rasterung.
    ///
    /// **Hier stand lange nur die Groessenliste.** `strichgroessen` in
    /// [`crate::font`] las aus `EBLC` die Zahl der Pixelgroessen, damit
    /// `get_sizes()` sie nennen kann - und gerastert wurde trotzdem
    /// immer der Umriss. Das war an zwei Tests ablesbar:
    /// `freetype_test.test_freetype_Font_use_bitmap_strikes` verlangt,
    /// dass sich das Ergebnis mit und ohne Strikes **unterscheidet**, und
    /// `test_freetype_Font_render` vergleicht `PyGameMono.otf` bei
    /// Groesse 8 mit einem Bild, das FontForge aus genau dieser Strike
    /// exportiert hat.
    ///
    /// ## Die Groesse muss **genau** stimmen
    ///
    /// `ttf-parser` sucht von sich aus die naechstliegende Strike heraus.
    /// FreeType tut das nicht: es nimmt ein eingebautes Bild nur, wenn
    /// die Pixelgroesse auf den Punkt passt, und rastert sonst den
    /// Umriss. Darum die Zeile `if bild.pixels_per_em != ppem`.
    ///
    /// Gemessen an `PyGameMono.otf` (Strikes bei 8 und 19 ppem):
    ///
    /// ```text
    /// Groesse  pygame.freetype          rustgame vorher
    ///   8      6x7, Strike              7x7, Umriss
    ///  19      aus der Strike           aus dem Umriss
    ///  24      aus dem Umriss           gleich
    /// ```
    ///
    /// ## Was nicht geht
    ///
    /// Ein Bild im PNG-Format (`sbix`, oder `CBDT` mit Format 17-19) wird
    /// **nicht** genommen: diese Kette fuehrt eine Deckung je Pixel, und
    /// ein farbiges Bild liesse sich darin nur als Schattenriss ablegen.
    /// Dann ist der Umriss die bessere Antwort. Farbige Bitmaps ohne
    /// Kompression (`BitmapPremulBgra32`) gehen als Schattenriss durch -
    /// ihr Alphakanal **ist** eine Deckung.
    pub(crate) fn strike(&self, gid: u16, ppem: u16) -> Option<Strikebild> {
        if ppem == 0 {
            return None;
        }
        let tt = self.tabellen()?;
        let bild = tt.glyph_raster_image(GlyphId(gid), ppem)?;
        if bild.pixels_per_em != ppem {
            return None;
        }
        let (w, h) = (bild.width as usize, bild.height as usize);
        if w == 0 || h == 0 {
            return None;
        }
        // `zeilenweise` heisst: jede Zeile faengt auf einer Bytegrenze an.
        // Die gepackten Formate zaehlen die Bits durch, ohne Luecke - das
        // ist der Unterschied zwischen `EBDT`-Format 1 und 2, und er
        // verschiebt jede Zeile ausser der ersten.
        let (tiefe, zeilenweise) = match bild.format {
            RasterImageFormat::BitmapMono => (1, true),
            RasterImageFormat::BitmapMonoPacked => (1, false),
            RasterImageFormat::BitmapGray2 => (2, true),
            RasterImageFormat::BitmapGray2Packed => (2, false),
            RasterImageFormat::BitmapGray4 => (4, true),
            RasterImageFormat::BitmapGray4Packed => (4, false),
            RasterImageFormat::BitmapGray8 => (8, true),
            RasterImageFormat::BitmapPremulBgra32 => (32, true),
            RasterImageFormat::PNG => return None,
        };
        let deckung = entpacken(bild.data, w, h, tiefe, zeilenweise)?;
        Some(Strikebild {
            left: i32::from(bild.x),
            // `ttf-parser` gibt `y` als **untere** Kante heraus (es rechnet
            // `bearingY - height`), die Kette hier will die obere.
            top: i32::from(bild.y) + h as i32,
            width: w,
            rows: h,
            deckung,
        })
    }

    /// Hat die Schrift einen Strike mit genau `ppem` Zeilen? Dann waehlt
    /// FreeTypes Groessenanfrage (`FT_Match_Size`) ihn - auch fuer eine
    /// Bruchgroesse, die auf `ppem` rundet - und rechnet mit seiner
    /// ganzzahligen Skala (`FT_Select_Metrics`).
    pub(crate) fn hat_strike(&self, ppem: u16) -> bool {
        use skrifa::raw::TableProvider;
        let Ok(font) = skrifa::raw::FontRef::from_index(&self.bytes, self.index) else {
            return false;
        };
        font.eblc()
            .map(|e| e.bitmap_sizes().iter().any(|g| u16::from(g.ppem_y()) == ppem))
            .unwrap_or(false)
    }

    /// Der Vorschub eines eingebauten Bildes in Pixeln: `horiAdvance` aus
    /// den sbit-Massen in `EBLC`/`EBDT` - so legt FreeTypes
    /// `load_sbit_image` ihn in `slot->metrics`, **nicht** aus `hmtx`.
    /// `ttf-parser` liest die Masse nicht mit heraus, `read-fonts` schon.
    ///
    /// Gesucht wird der Strike mit genau `ppem` Zeilen (`FT_Match_Size`).
    ///
    /// Dazu die senkrechten sbit-Masse `(vertBearingX, vertBearingY,
    /// vertAdvance)`: aus grossen Massen (`BigGlyphMetrics`) gelesen, bei
    /// kleinen setzt FreeTypes `tt_sbit_decoder_load_metrics` alle drei
    /// auf null.
    pub(crate) fn strike_masse(&self, gid: u16, ppem: u16) -> Option<(i64, (i64, i64, i64))> {
        use skrifa::raw::tables::bitmap::BitmapMetrics;
        use skrifa::raw::TableProvider;
        let font = skrifa::raw::FontRef::from_index(&self.bytes, self.index).ok()?;
        let eblc = font.eblc().ok()?;
        let ebdt = font.ebdt().ok()?;
        let gid = skrifa::GlyphId::new(u32::from(gid));
        for groesse in eblc.bitmap_sizes() {
            if u16::from(groesse.ppem_y()) != ppem {
                continue;
            }
            let Ok(ort) = groesse.location(eblc.offset_data(), gid) else {
                continue;
            };
            let daten = ebdt.data(&ort).ok()?;
            return Some(match daten.metrics {
                BitmapMetrics::Small(m) => (i64::from(m.advance), (0, 0, 0)),
                BitmapMetrics::Big(m) => (
                    i64::from(m.hori_advance),
                    (
                        i64::from(m.vert_bearing_x()),
                        i64::from(m.vert_bearing_y()),
                        i64::from(m.vert_advance),
                    ),
                ),
            });
        }
        None
    }

    /// Ein frisch aufgeschlagenes Sichtfenster auf die Tabellen.
    ///
    /// Das kostet nur einen Durchlauf durchs Tabellenverzeichnis, deshalb
    /// darf es je Layout-Durchgang einmal passieren - statt einer
    /// selbstbezueglichen Struktur, die das Fenster dauerhaft festhaelt.
    pub(crate) fn tabellen(&self) -> Option<TtfFace<'_>> {
        if self.punkte.is_some() {
            return None;
        }
        // Beim Laden schon einmal erfolgreich geparst.
        Some(TtfFace::parse(&self.bytes[..], self.index).expect("beim Laden bereits geprueft"))
    }
}

/// Sucht die Lookups, die zum Merkmal `kern` in GPOS gehoeren.
fn kern_lookups_von(tt: &TtfFace<'_>) -> (bool, Vec<u16>) {
    let Some(gpos) = tt.tables().gpos else {
        return (false, Vec::new());
    };
    // HarfBuzz betrachtet GPOS nur als vorhanden, wenn es auch Schriftsysteme
    // auffuehrt - sonst faellt es auf die kern-Tabelle zurueck.
    if gpos.scripts.is_empty() {
        return (false, Vec::new());
    }
    let mut idx: Vec<u16> = Vec::new();
    for feature in gpos.features {
        if feature.tag != Tag::from_bytes(b"kern") {
            continue;
        }
        for li in feature.lookup_indices {
            if !idx.contains(&li) {
                idx.push(li);
            }
        }
    }
    idx.sort_unstable();
    (true, idx)
}

/// Ein aufgeschlagenes Sichtfenster plus die Kerning-Quelle - lebt genau
/// einen Layout-Durchgang.
pub(crate) struct Sicht<'a> {
    /// Die Tabellen - bei einer Bitmapschrift gibt es keine.
    pub(crate) tt: Option<TtfFace<'a>>,
    pub(crate) datei: &'a Datei,
}

impl<'a> Sicht<'a> {
    pub(crate) fn neu(datei: &'a Datei) -> Sicht<'a> {
        Sicht {
            tt: datei.tabellen(),
            datei,
        }
    }

    /// Die Tabellen, oder ein leeres Sichtfenster fuer Bitmapschriften.
    ///
    /// Gibt es keine, liefert jede Frage danach den Wert, den eine Schrift
    /// ohne die betreffende Tabelle liefert - das ist dieselbe Antwort,
    /// die `ttf_parser` auf eine fehlende Tabelle gibt.
    fn hat_tabellen(&self) -> bool {
        self.tt.is_some()
    }

    /// Glyphnummer eines Zeichens, wie **SDL_ttf** sie sucht: beim Oeffnen
    /// waehlt `TTF_OpenFontIndexDPIRW` die Zeichentabelle (3,10), sonst die
    /// erste aus (3,1), (3,0), (2,1) und (0,*) in der Reihenfolge der
    /// Datei, und `get_char_index` fragt nur diese (`FT_Get_Char_Index`).
    /// Bei einer Symbolschrift ist das (3,0): `Font.metrics("")` gibt
    /// bei pygame das `A` von `symbol.ttf`, ttf-parser uebergeht (3,0) und
    /// gab hier das Ersatzzeichen. Ohne passende Tabelle bleibt FreeTypes
    /// eigene Wahl ([`Sicht::glyph`]).
    pub(crate) fn glyph_sdl_ttf(&self, c: char) -> u16 {
        use ttf_parser::PlatformId;
        let Some(tt) = &self.tt else {
            return self.glyph(c);
        };
        let Some(cmap) = tt.tables().cmap else {
            return 0;
        };
        let wahl = cmap
            .subtables
            .into_iter()
            .find(|s| s.platform_id == PlatformId::Windows && s.encoding_id == 10)
            .or_else(|| {
                cmap.subtables.into_iter().find(|s| {
                    matches!(
                        (s.platform_id, s.encoding_id),
                        (PlatformId::Windows, 1)
                            | (PlatformId::Windows, 0)
                            | (PlatformId::Iso, 1)
                            | (PlatformId::Unicode, _)
                    )
                })
            });
        match wahl {
            Some(s) => s.glyph_index(u32::from(c)).map_or(0, |g| g.0),
            None => self.glyph(c),
        }
    }

    /// Glyphnummer eines Zeichens; unbekannte Zeichen landen wie in FreeType
    /// auf `.notdef` (0).
    pub(crate) fn glyph(&self, c: char) -> u16 {
        match (&self.tt, &self.datei.punkte) {
            (Some(tt), _) => tt.glyph_index(c).map_or(0, |g| g.0),
            (None, Some(b)) => b.glyph(c),
            (None, None) => 0,
        }
    }

    /// Der waagerechte Vorschub aus `hmtx` in Schrifteinheiten
    /// (`linearHoriAdvance` vor dem Skalieren).
    pub(crate) fn vorschub_einheiten(&self, gid: u16) -> i64 {
        self.tt
            .as_ref()
            .and_then(|tt| tt.glyph_hor_advance(GlyphId(gid)))
            .map_or(0, i64::from)
    }

    /// Vorschub in 26.6 - **gerundet auf ganze Pixel**.
    ///
    /// Gemessen: pygame meldet ueber `Font.metrics()` genau
    /// `FT_PIX_ROUND(FT_MulFix(hmtx, scale)) / 64`. Weder Ab- noch
    /// Aufrunden trifft das; nur kaufmaennisches Runden. Geprueft ueber alle
    /// druckbaren ASCII-Zeichen von freesansbold, Arial, Consolas, Segoe UI,
    /// Bahnschrift, Tahoma, Verdana, Comic Sans und Impact.
    pub(crate) fn vorschub26(&self, gid: u16, scale: i64) -> i64 {
        let einheiten = match (&self.tt, &self.datei.punkte) {
            (Some(tt), _) => tt.glyph_hor_advance(GlyphId(gid)).unwrap_or(0),
            // Bei einer Bitmapschrift ist `DWIDTH` schon in Pixeln, und
            // ein Pixel ist hier eine Schrifteinheit - siehe
            // [`Datei::aus_punkten`].
            (None, Some(b)) => b.bei(gid).map(|z| z.vorschub).unwrap_or(0).max(0) as u16,
            (None, None) => 0,
        };
        fixed::pix_round26(fixed::mul_fix(i64::from(einheiten), scale))
    }

    /// Kerning zweier Glyphen in Schrifteinheiten.
    pub(crate) fn kerning(&self, links: u16, rechts: u16) -> i16 {
        // Eine Bitmapschrift hat keine Unterschneidung - BDF kennt keine.
        if !self.hat_tabellen() {
            return 0;
        }
        if self.datei.hat_gpos {
            self.kerning_gpos(links, rechts)
        } else {
            self.kerning_alt(links, rechts)
        }
    }

    fn kerning_gpos(&self, links: u16, rechts: u16) -> i16 {
        let Some(tt) = &self.tt else { return 0 };
        let Some(gpos) = tt.tables().gpos else {
            return 0;
        };
        let (l, r) = (GlyphId(links), GlyphId(rechts));
        for &li in &self.datei.kern_lookups {
            let Some(lookup) = gpos.lookups.get(li) else {
                continue;
            };
            for sub in lookup
                .subtables
                .into_iter::<ttf_parser::gpos::PositioningSubtable>()
            {
                let ttf_parser::gpos::PositioningSubtable::Pair(paar) = sub else {
                    continue;
                };
                if let Some(v) = paar_wert(&paar, l, r) {
                    if v != 0 {
                        return v;
                    }
                }
            }
        }
        0
    }

    /// Der senkrechte Vorschub in Schrifteinheiten, den FreeTypes
    /// `compute_glyph_metrics` fuer eine Schrift **ohne** `vmtx` erfindet:
    /// `OS/2.sTypoAscender - sTypoDescender`, ohne OS/2-Tabelle die
    /// `hhea`-Werte (roh, nicht die Ersatzwerte aus [`Datei::laden`]).
    ///
    /// `None`, wenn die Schrift eigene senkrechte Masse traegt (`vhea` mit
    /// Eintraegen) - die stehen dann in `vmtx` und sind hier nicht
    /// nachgebaut.
    pub(crate) fn senkrecht_einheiten(&self) -> Option<i64> {
        let tt = self.tt.as_ref()?;
        if tt.tables().vhea.is_some_and(|v| v.number_of_metrics > 0) {
            return None;
        }
        Some(match tt.tables().os2 {
            Some(os2) => {
                i64::from(os2.typographic_ascender()) - i64::from(os2.typographic_descender())
            }
            None => {
                let h = tt.tables().hhea;
                i64::from(h.ascender) - i64::from(h.descender)
            }
        })
    }

    /// `head.flags` Bit 3: "ppem auf ganze Zahlen runden". FreeTypes
    /// `tt_size_reset` rechnet dann **beim Laden** mit der Skala der
    /// gerundeten Groesse (`hinted_metrics`), waehrend die oeffentlichen
    /// Masse (`size->metrics`) ungerundet bleiben.
    pub(crate) fn ppem_ganzzahlig(&self) -> bool {
        self.tt
            .as_ref()
            .and_then(|tt| tt.raw_face().table(ttf_parser::Tag::from_bytes(b"head")))
            .and_then(|d| d.get(16..18))
            .is_some_and(|b| u16::from_be_bytes([b[0], b[1]]) & 8 != 0)
    }

    /// `hhea.advanceWidthMax` - FreeTypes `face->max_advance_width`.
    /// `ttf-parser` liest das Feld nicht, also roh ab Byte 10.
    pub(crate) fn max_vorschub_einheiten(&self) -> i64 {
        self.tt
            .as_ref()
            .and_then(|tt| tt.raw_face().table(ttf_parser::Tag::from_bytes(b"hhea")))
            .and_then(|d| d.get(10..12))
            .map_or(0, |b| i64::from(u16::from_be_bytes([b[0], b[1]])))
    }

    /// PostScript-Umrisse (`CFF `)? `CFF2` liest `ttf-parser` hier nicht.
    pub(crate) fn ist_cff(&self) -> bool {
        self.tt
            .as_ref()
            .is_some_and(|t| t.tables().cff.is_some())
    }

    /// `FT_Get_Kerning` in Schrifteinheiten: **nur** die alte `kern`-
    /// Tabelle - FreeTypes `tt_face_get_kerning` auf ihren rohen Bytes,
    /// siehe [`kern_laden`] und [`kern_holen`]. GPOS kennt FreeType hier
    /// nicht. Das ist `pygame.freetype`s Unterschneidung.
    pub(crate) fn kerning_freetype(&self, links: u16, rechts: u16) -> i64 {
        let Some(tt) = &self.tt else { return 0 };
        let Some(t) = tt.raw_face().table(Tag::from_bytes(b"kern")) else {
            return 0;
        };
        let bits = *self.datei.kern_bits.get_or_init(|| kern_laden(t));
        kern_holen(t, bits, links, rechts)
    }

    fn kerning_alt(&self, links: u16, rechts: u16) -> i16 {
        let Some(tt) = &self.tt else { return 0 };
        let Some(kern) = tt.tables().kern else {
            return 0;
        };
        for sub in kern.subtables {
            if !sub.horizontal || sub.has_cross_stream || sub.variable {
                continue;
            }
            if let Some(v) = sub.glyphs_kerning(GlyphId(links), GlyphId(rechts)) {
                return v;
            }
        }
        0
    }
}

fn be16(t: &[u8], i: usize) -> u32 {
    t.get(i..i + 2)
        .map_or(0, |b| u32::from(u16::from_be_bytes([b[0], b[1]])))
}

fn be32(t: &[u8], i: usize) -> u32 {
    t.get(i..i + 4)
        .map_or(0, |b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
}

/// `tt_face_load_kern` (`ttkern.c`, FreeType 2.11.1): wie viele
/// Teiltabellen FreeType ansieht, welche es benutzt und welche sortiert
/// sind - `(num_kern_tables, kern_avail_bits, kern_order_bits)`.
///
/// **Die Laenge einer Teiltabelle ist bei FreeType 16 Bit**, und
/// `handle broken count` kuerzt die Paarliste auf das, was hineinpasst.
/// Calibri hat 26.706 Paare (160.250 Bytes) in einer Teiltabelle, deren
/// Laengenfeld deshalb 29.178 sagt - FreeType sieht davon 4860 Paare,
/// ttf-parser alle. Gemessen: mit Kerning war hier jede Zeile in Calibri
/// schmaler als bei pygame (`"AVAWAT"` bei 24: 83 gegen 85).
fn kern_laden(t: &[u8]) -> (u32, u32, u32) {
    if t.len() < 4 {
        return (0, 0, 0);
    }
    let grenze = t.len();
    let anzahl = be16(t, 2).min(32);
    let (mut avail, mut ordered) = (0u32, 0u32);
    let mut p = 4usize;
    let mut nn = 0u32;
    while nn < anzahl {
        let mask = 1u32 << nn;
        if p + 6 > grenze {
            break;
        }
        let laenge = be16(t, p + 2) as usize;
        let coverage = be16(t, p + 4);
        if laenge <= 6 + 8 {
            break;
        }
        let naechste = (p + laenge).min(grenze);
        let mut q = p + 6;
        // nur Format 0, nur waagerecht
        if coverage >> 8 == 0 && coverage & 3 == 1 && q + 8 <= naechste {
            let mut paare = be16(t, q) as usize;
            q += 8;
            if naechste - q < 6 * paare {
                paare = (naechste - q) / 6;
            }
            avail |= mask;
            if paare > 0 {
                let mut alt = be32(t, q);
                q += 6;
                let mut rest = paare - 1;
                while rest > 0 {
                    let neu = be32(t, q);
                    if neu <= alt {
                        break;
                    }
                    q += 6;
                    alt = neu;
                    rest -= 1;
                }
                if rest == 0 {
                    ordered |= mask;
                }
            }
        }
        p = naechste;
        nn += 1;
    }
    (nn, avail, ordered)
}

/// `tt_face_get_kerning` (`ttkern.c`, FreeType 2.11.1) - mit den Bits
/// aus [`kern_laden`]: sortierte Teiltabellen binaer, die anderen der
/// Reihe nach, Bit 3 der Abdeckung ersetzt statt zu addieren.
fn kern_holen(t: &[u8], bits: (u32, u32, u32), links: u16, rechts: u16) -> i64 {
    let (anzahl, avail, ordered) = bits;
    let grenze = t.len();
    let schluessel = (u32::from(links) << 16) | u32::from(rechts);
    let mut ergebnis = 0i64;
    let mut p = 4usize;
    for nn in 0..anzahl {
        if p + 6 > grenze {
            break;
        }
        let mask = 1u32 << nn;
        let laenge = be16(t, p + 2) as usize;
        let coverage = be16(t, p + 4);
        let naechste = (p + laenge).min(grenze);
        if avail & mask != 0 && coverage >> 8 == 0 {
            let mut q = p + 6;
            let mut paare = be16(t, q) as usize;
            q += 8;
            if naechste - q < 6 * paare {
                paare = (naechste - q) / 6;
            }
            let mut wert = None;
            if ordered & mask != 0 {
                let (mut min, mut max) = (0usize, paare);
                while min < max {
                    let mitte = (min + max) >> 1;
                    let k = be32(t, q + 6 * mitte);
                    if k == schluessel {
                        wert = Some(be16(t, q + 6 * mitte + 4) as u16 as i16);
                        break;
                    }
                    if k < schluessel {
                        min = mitte + 1;
                    } else {
                        max = mitte;
                    }
                }
            } else {
                for i in 0..paare {
                    if be32(t, q + 6 * i) == schluessel {
                        wert = Some(be16(t, q + 6 * i + 4) as u16 as i16);
                        break;
                    }
                }
            }
            if let Some(w) = wert {
                if coverage & 8 != 0 {
                    ergebnis = i64::from(w);
                } else {
                    ergebnis += i64::from(w);
                }
            }
        }
        p = naechste;
    }
    ergebnis
}

/// Der Vorschub-Zuschlag aus einer GPOS-Paartabelle.
///
/// Nur `xAdvance` des **ersten** Glyphs zaehlt - das ist Kerning. Die
/// uebrigen Felder eines `ValueRecord` verschieben Glyphen, nicht die Zeile.
fn paar_wert(paar: &PairAdjustment<'_>, links: GlyphId, rechts: GlyphId) -> Option<i16> {
    match paar {
        PairAdjustment::Format1 { coverage, sets } => {
            let i = coverage.get(links)?;
            let satz = sets.get(i)?;
            let (v1, _) = satz.get(rechts)?;
            Some(v1.x_advance)
        }
        PairAdjustment::Format2 {
            coverage,
            classes,
            matrix,
        } => {
            coverage.get(links)?;
            let k1 = classes.0.get(links);
            let k2 = classes.1.get(rechts);
            let (v1, _) = matrix.get((k1, k2))?;
            Some(v1.x_advance)
        }
    }
}

/// Ein fertiges Pixelbild aus einer eingebauten Groesse.
///
/// Dieselben vier Zahlen wie ein gerasterter Umriss, damit
/// [`crate::glyph::rastern`] beide Quellen gleich behandelt: Ansatz links,
/// Oberkante ueber der Grundlinie, Breite, Hoehe - und eine Deckung je
/// Pixel.
pub(crate) struct Strikebild {
    /// Ansatz links von der Schreibmarke (`bearingX`).
    pub left: i32,
    /// Oberkante ueber der Grundlinie (`bearingY`).
    pub top: i32,
    pub width: usize,
    pub rows: usize,
    /// Ein Byte je Pixel, zeilenweise von oben.
    pub deckung: Vec<u8>,
}

/// Macht aus gepackten Bits eine Deckung je Pixel.
///
/// `tiefe` ist 1, 2, 4, 8 oder 32 Bit je Pixel; `zeilenweise` sagt, ob
/// jede Zeile auf einer Bytegrenze anfaengt. Das hoechste Bit des ersten
/// Bytes ist das linke obere Pixel - so steht es in der Norm, und so
/// liest es FreeType.
///
/// Die Werte werden auf 0..255 gestreckt, nicht verschoben: aus 1 Bit
/// wird 0 oder 255, aus 2 Bit 0/85/170/255, aus 4 Bit Vielfache von 17.
/// Eine Verschiebung (`v << 6`) liesse den hoechsten Wert bei 192 stehen,
/// und dann waere kein Pixel je ganz deckend.
///
/// `None`, wenn die Daten kuerzer sind als angekuendigt - ein
/// abgeschnittenes Bild ist kein halbes Bild, sondern keines.
fn entpacken(
    daten: &[u8],
    w: usize,
    h: usize,
    tiefe: u32,
    zeilenweise: bool,
) -> Option<Vec<u8>> {
    if tiefe == 32 {
        // Farbig und unkomprimiert: BGRA, schon vormultipliziert. Als
        // Deckung zaehlt der Alphakanal.
        if daten.len() < w * h * 4 {
            return None;
        }
        return Some((0..w * h).map(|i| daten[i * 4 + 3]).collect());
    }
    let zeilenbits = if zeilenweise {
        (w as u64 * u64::from(tiefe)).div_ceil(8) * 8
    } else {
        w as u64 * u64::from(tiefe)
    };
    let gebraucht = (zeilenbits * h as u64).div_ceil(8) as usize;
    if daten.len() < gebraucht {
        return None;
    }
    let hoechst = (1u32 << tiefe) - 1;
    let mut aus = Vec::with_capacity(w * h);
    for y in 0..h {
        let zeile = zeilenbits * y as u64;
        for x in 0..w {
            let bit = zeile + x as u64 * u64::from(tiefe);
            let mut v = 0u32;
            for k in 0..u64::from(tiefe) {
                let b = bit + k;
                let byte = daten[(b / 8) as usize];
                let gesetzt = (byte >> (7 - (b % 8))) & 1;
                v = (v << 1) | u32::from(gesetzt);
            }
            // Strecken, nicht verschieben - siehe oben.
            aus.push((v * 255 / hoechst) as u8);
        }
    }
    Some(aus)
}

#[cfg(test)]
mod strike_tests {
    use super::entpacken;

    /// Ein Bit je Pixel, zeilenweise: die zweite Zeile faengt im zweiten
    /// Byte an, auch wenn das erste nur drei Pixel brauchte.
    #[test]
    fn ein_bit_zeilenweise() {
        // 3x2: Zeile 1 = 101, Zeile 2 = 010
        let d = [0b1010_0000, 0b0100_0000];
        let v = entpacken(&d, 3, 2, 1, true).unwrap();
        assert_eq!(v, vec![255, 0, 255, 0, 255, 0]);
    }

    /// Dasselbe gepackt: die zweite Zeile faengt beim vierten Bit an.
    #[test]
    fn ein_bit_gepackt() {
        // 3x2 ohne Luecke: 101 010 -> 101010 00
        let d = [0b1010_1000];
        let v = entpacken(&d, 3, 2, 1, false).unwrap();
        assert_eq!(v, vec![255, 0, 255, 0, 255, 0]);
    }

    /// **Der Unterschied zwischen beiden ist messbar und kein Detail.**
    /// Dieselben Bytes, zwei Bedeutungen.
    #[test]
    fn zeilenweise_und_gepackt_sind_nicht_dasselbe() {
        let d = [0b1010_1000, 0b0000_0000];
        assert_ne!(
            entpacken(&d, 3, 2, 1, true).unwrap(),
            entpacken(&d, 3, 2, 1, false).unwrap()
        );
    }

    /// Graustufen werden gestreckt: der hoechste Wert ist 255, nicht 192.
    #[test]
    fn graustufen_reichen_bis_ganz_oben() {
        let d = [0b1101_1000]; // 2 Bit: 3, 1, 2, 0
        let v = entpacken(&d, 4, 1, 2, true).unwrap();
        assert_eq!(v, vec![255, 85, 170, 0]);
        let d4 = [0xF0, 0x08]; // 4 Bit: 15, 0, 0, 8
        assert_eq!(entpacken(&d4, 4, 1, 4, true).unwrap(), vec![255, 0, 0, 136]);
    }

    /// Acht Bit gehen unveraendert durch.
    #[test]
    fn acht_bit_bleibt_wie_es_ist() {
        let d = [0, 17, 128, 255];
        assert_eq!(entpacken(&d, 2, 2, 8, true).unwrap(), vec![0, 17, 128, 255]);
    }

    /// Zu wenig Daten gibt `None` und keine halbe Zeile.
    #[test]
    fn abgeschnitten_gibt_nichts() {
        assert!(entpacken(&[0xFF], 3, 2, 1, true).is_none());
        assert!(entpacken(&[0xFF, 0xFF, 0xFF], 2, 2, 32, true).is_none());
    }

    /// Farbig: der Alphakanal wird die Deckung.
    #[test]
    fn farbig_wird_ein_schattenriss() {
        let d = [1, 2, 3, 200, 4, 5, 6, 0];
        assert_eq!(entpacken(&d, 2, 1, 32, true).unwrap(), vec![200, 0]);
    }
}
