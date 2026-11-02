//! font.rs
//! =======
//! `pygame.font.Font` - Metriken, Textbreite, Rendern.
//!
//! ## Der Weg, den pygame wirklich geht
//!
//! `Font.render(text, True, color)` sieht harmlos aus, laeuft in pygame-ce
//! 2.5.7 aber ueber **drei** Schichten, und jede davon hinterlaesst Spuren,
//! die man nachbauen muss:
//!
//! 1. `font.c` waehlt `TTF_RenderUTF8_Blended_Wrapped(font, text, fg, 0)` -
//!    also die **umbrechende** Variante, auch ohne Umbruchbreite. Daher
//!    zerlegt schon ein `\n` den Text in Zeilen, und daher ist die Flaeche
//!    `linesize` hoch und nicht `height`.
//! 2. `TTF_Render_Wrapped_Internal` legt die Flaeche an:
//!    `rowHeight = max(hoehe_des_ganzen_textes, linesize)`, dann
//!    `hoehe = rowHeight + linesize * (zeilen - 1)`.
//! 3. `Render_Line` kopiert die Glyphenbilder hinein - mit `|=`, nicht mit
//!    einer Mischung. Ueberlappende Zeichen **verodern** ihre Deckung.
//!
//! ## Was gemessen wurde
//!
//! | Groesse | Quelle |
//! |---|---|
//! | ascent, descent, height, linesize | `hhea` + FreeType-Festkomma, **exakt** |
//! | Vorschub | `hmtx`, auf ganze Pixel gerundet, **exakt** |
//! | Kerning | GPOS (sonst `kern`), ungerundet |
//! | Glyphenbild | fontdue - hier endet die Gleichheit |
//!
//! Die Flaeche bei `antialias=true` ohne Hintergrund traegt die Farbe
//! konstant im RGB-Teil; die Deckung steht allein im Alphakanal. Das ist
//! kein Kunstgriff, sondern `Create_Surface_Blended`: die Flaeche wird mit
//! `0x00RRGGBB` gefuellt und dann nur noch `|= deckung << 24`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use rustgame_core::color::{Color, IntoColor};
use rustgame_core::{Rect, Surface, SRCALPHA};
use ttf_parser::{Face as TtfFace, Tag};

use crate::face::{Datei, Sicht};
use crate::fixed::{self, ceil26, floor26, mul_fix, pix_round26};
use crate::glyph::{rastern, GlyphBild};
use crate::FontError;

/// Die fuenf Zahlen, die `Font.metrics` je Zeichen liefert:
/// `(minx, maxx, miny, maxy, advance)`.
pub type GlyphMass = (i32, i32, i32, i32, i32);

mod ftzeile;
pub use ftzeile::{FtGlyphe, FtMass, FtStil, FtZeile};

/// Die vier Stilschalter von `TTF_SetFontStyle`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Stil {
    /// Fett - nachgemacht durch Verschmieren nach rechts.
    pub bold: bool,
    /// Kursiv - nachgemacht durch Scheren.
    pub italic: bool,
    /// Unterstrichen.
    pub underline: bool,
    /// Durchgestrichen.
    pub strikethrough: bool,
}

impl Stil {
    /// Als Schluessel fuer den Bildzwischenspeicher.
    ///
    /// **Die Strike gehoert dazu.** Ohne das vierte Bit gab ein
    /// `use_bitmap_strikes = False` nach einem Durchgang mit Strikes
    /// dasselbe Bild zurueck - aus dem Zwischenspeicher. Genau diesen
    /// Vergleich stellt `test_freetype_Font_use_bitmap_strikes` an.
    fn schluessel(self, mono: bool, strike: bool) -> u8 {
        u8::from(self.bold)
            | (u8::from(self.italic) << 1)
            | (u8::from(mono) << 2)
            | (u8::from(strike) << 3)
    }
}

/// Wie die Zeilen eines mehrzeiligen Textes zueinander stehen -
/// `TTF_SetFontWrappedAlign`, in pygame das Attribut `Font.align`.
///
/// Die Zahlenwerte sind pygames `FONT_LEFT`, `FONT_CENTER`, `FONT_RIGHT`;
/// gemessen an pygame-ce 2.5.7: `pygame.FONT_LEFT, FONT_CENTER, FONT_RIGHT`
/// ergibt `0 1 2`, und `Font(None, 24).align` ist `0`.
///
/// Bei **einer** Zeile aendert die Ausrichtung nichts, denn dann ist die
/// Flaeche genau so breit wie die Zeile - auch das gemessen:
/// `render("Hallo", …).get_size()` ist fuer alle drei Werte `(41, 18)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Ausrichtung {
    #[default]
    /// Linksbuendig - `pygame.FONT_LEFT`, der Normalfall.
    Links,
    /// Zentriert - `pygame.FONT_CENTER`.
    Mitte,
    /// Rechtsbuendig - `pygame.FONT_RIGHT`.
    Rechts,
}

/// Ein Platz fuer ein Zeichen in einer Zeile.
#[derive(Debug, Clone, Copy)]
struct Platz {
    gid: u16,
    /// Schreibmarke in 26.6 - **vor** dem Abrunden auf ganze Pixel.
    x26: i64,
    /// Senkrechte Lage in 26.6, von oben: `ascent - y_offset` (SDL_ttfs
    /// `pos_y`). Ohne GPOS-Verschiebung ist das die Grundlinie.
    y26: i64,
}

/// Das Ergebnis von `TTF_Size_Internal` fuer eine Zeile.
#[derive(Debug, Default)]
struct Zeilenmass {
    w: i32,
    h: i32,
    xstart: i32,
    ystart: i32,
    plaetze: Vec<Platz>,
}

/// Eine geladene Schrift in einer festen Punktgroesse.
///
/// Gegenstueck zu `pygame.font.Font`.
///
/// **Klonbar**, und das kostet nichts: die Schriftdatei haengt an einem
/// `Rc`, geklont werden nur die Masse und der Stil. `pygame.freetype`
/// braucht das, weil dort jeder Aufruf eine andere Groesse mitgeben darf.
#[derive(Clone)]
pub struct Font {
    datei: Rc<Datei>,
    ptsize: i32,
    stil: Stil,
    /// 16.16-Faktor von Schrifteinheiten auf 26.6-Pixel.
    scale: i64,
    ascent: i32,
    descent: i32,
    height: i32,
    linesize: i32,
    /// Von Hand gesetzte Zeilenhoehe (`Font.set_linesize`).
    linesize_gesetzt: Option<i32>,
    /// Ausrichtung mehrzeiliger Texte (`Font.align`).
    align: Ausrichtung,
    line_thickness: i32,
    underline_top_row: i32,
    strikethrough_top_row: i32,
    glyph_overhang: i32,
    /// Wie dick die Kontur ist - `Font.outline`. 0 heisst: gefuellt.
    outline: i32,
    /// Eingebaute Pixelbilder benutzen, wenn es sie in dieser Groesse gibt?
    ///
    /// **Vorgabe ist `true`, und das ist gemessen.** Es ist nicht nur
    /// `pygame.freetype.Font.use_bitmap_strikes`, dessen Vorgabe wahr ist -
    /// auch SDL_ttf laedt ohne `FT_LOAD_NO_BITMAP`, nimmt also ebenfalls
    /// die Strike. Nachgesehen mit `PyGameMono.otf` bei Groesse 8, dieselbe
    /// Form in beiden Kisten:
    ///
    /// ```text
    ///            pygame.freetype          pygame.font
    ///              .####.                  .####...
    ///              #....#                  #....#..
    ///              ######                  ######..
    ///              #....#                  #....#..
    /// ```
    ///
    /// Der Umriss derselben Groesse sieht anders aus (7 statt 6 Spalten,
    /// doppelte Staemme) - wer hier auf `false` stellt, bekommt ihn.
    bitmap_strikes: bool,
    /// Waechst der Kasten, wenn der Unterstrich unten hinausragt?
    ///
    /// **Die beiden Textkisten antworten verschieden.** `pygame.font`
    /// (SDL_ttf) zeichnet den Strich in die Flaeche, die die Zeichen
    /// brauchen, und schneidet ihn ab, wenn er tiefer liegt.
    /// `pygame.freetype` nimmt ihn in den Kasten auf - gemessen gibt
    /// `get_rect(..., style=STYLE_UNDERLINE)` dort ein **hoeheres**
    /// Rechteck als ohne.
    ///
    /// Ein Schalter, kein zweiter Rechenweg: es geht um eine Zeile in der
    /// Hoehenrechnung.
    unterstrich_waechst: bool,
    bilder: RefCell<HashMap<(u16, u8), Rc<GlyphBild>>>,
}

impl std::fmt::Debug for Font {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Font")
            .field("ptsize", &self.ptsize)
            .field("stil", &self.stil)
            .field("height", &self.height)
            .field("linesize", &self.linesize)
            .finish()
    }
}

impl Font {
    /// Laedt eine Schriftdatei. Gegenstueck zu `pygame.font.Font(pfad, groesse)`.
    pub fn new(pfad: impl AsRef<Path>, groesse: i32) -> Result<Font, FontError> {
        let bytes = std::fs::read(pfad.as_ref()).map_err(FontError::Datei)?;
        Font::from_bytes(bytes, groesse)
    }

    /// Wie [`Font::new`], aber aus dem Speicher.
    pub fn from_bytes(bytes: Vec<u8>, groesse: i32) -> Result<Font, FontError> {
        Font::from_bytes_index(bytes, groesse, 0)
    }

    /// Wie [`Font::from_bytes`], mit Auswahl aus einer Sammlung (`.ttc`).
    pub fn from_bytes_index(bytes: Vec<u8>, groesse: i32, index: u32) -> Result<Font, FontError> {
        // pygame klemmt die Groesse auf >= 1, zweimal sogar.
        let groesse = groesse.max(1);
        let datei = Rc::new(Datei::laden(bytes, index, groesse)?);
        let mut f = Font {
            datei,
            ptsize: groesse,
            stil: Stil::default(),
            scale: 0,
            ascent: 0,
            descent: 0,
            height: 0,
            linesize: 0,
            linesize_gesetzt: None,
            align: Ausrichtung::Links,
            line_thickness: 0,
            underline_top_row: 0,
            strikethrough_top_row: 0,
            glyph_overhang: 0,
            outline: 0,
            bitmap_strikes: true,
            unterstrich_waechst: false,
            bilder: RefCell::new(HashMap::new()),
        };
        f.metriken_neu();
        Ok(f)
    }

    /// Die mitgelieferte Schrift in der Groesse, die `pygame.font.Font(None,
    /// groesse)` daraus macht.
    ///
    /// **Achtung, das ist kein Tippfehler:** pygame rechnet die Groesse hier
    /// mit `0.6875` klein (`font.c`, `font_init`). `Font(None, 16)` ist
    /// deshalb freesansbold in Groesse 11 - `get_height()` liefert 11, nicht
    /// 16. Wer den Pfad selbst angibt, bekommt die volle Groesse.
    pub fn default_font(bytes: Vec<u8>, groesse: i32) -> Result<Font, FontError> {
        let g = ((groesse.max(1) as f64) * 0.6875) as i32;
        Font::from_bytes(bytes, g.max(1))
    }

    // ----- Metriken -------------------------------------------------------

    /// `TTF_initFontMetrics` - Zeile fuer Zeile.
    fn metriken_neu(&mut self) {
        let d = &self.datei;
        let scale = fixed::scale_for(self.ptsize, d.units_per_em);
        self.scale = scale;
        let asc = i64::from(d.ascender);
        let desc = i64::from(d.descender);
        self.ascent = ceil26(mul_fix(asc, scale));
        self.descent = ceil26(mul_fix(desc, scale));
        self.height = ceil26(mul_fix(asc - desc, scale));
        self.linesize = ceil26(mul_fix(asc - desc + i64::from(d.line_gap), scale));

        let unter_off = floor26(mul_fix(i64::from(d.underline_position), scale));
        self.line_thickness = floor26(mul_fix(i64::from(d.underline_thickness), scale)).max(1);
        self.underline_top_row = self.ascent - unter_off - 1;
        self.strikethrough_top_row = self.height / 2;

        // **Die Kontur aendert nur die Striche** (`TTF_initFontMetrics`):
        // dicker um `2n`, um `n` hoeher. Hoehe, Ober- und Unterlaenge
        // bleiben - die `+2n` legen erst `TTF_Size_Internal`,
        // `TTF_GlyphMetrics32` und `TTF_FontAscent` darauf. Hier stand
        // bis Runde 19 `height += 2n`, `ascent += n`, `descent -= n`;
        // `size()` traf damit, `get_height()`, `get_ascent()`,
        // `get_descent()`, `metrics()` und die Bildgroesse nicht.
        if self.outline > 0 {
            let fo = self.outline;
            self.line_thickness += 2 * fo;
            self.underline_top_row -= fo;
            self.strikethrough_top_row -= fo;
        }
        self.underline_top_row = self.underline_top_row.max(0);
        self.strikethrough_top_row = self.strikethrough_top_row.max(0);

        // Unterstrich und Durchstreichung duerfen nicht aus der Flaeche
        // fallen - deshalb waechst `height` notfalls mit. Das ist der
        // Grund, warum `get_height()` sich aendert, sobald man
        // `set_underline(True)` setzt.
        if self.stil.underline {
            self.height = self
                .height
                .max(self.underline_top_row + self.line_thickness);
        }
        if self.stil.strikethrough {
            self.height = self
                .height
                .max(self.strikethrough_top_row + self.line_thickness);
        }

        self.glyph_overhang = self.ptsize / 10;
        self.bilder.borrow_mut().clear();
    }

    /// Hoehe einer Textzeile ohne Zeilenabstand (`TTF_FontHeight`).
    pub fn get_height(&self) -> i32 {
        self.height
    }

    /// Empfohlener Zeilenabstand (`TTF_FontLineSkip`).
    pub fn get_linesize(&self) -> i32 {
        self.linesize_gesetzt.unwrap_or(self.linesize)
    }

    /// Ueberschreibt den Zeilenabstand (`Font.set_linesize`).
    pub fn set_linesize(&mut self, linesize: i32) {
        self.linesize_gesetzt = Some(linesize);
    }

    /// Ausrichtung mehrzeiliger Texte (`Font.align`).
    pub fn get_align(&self) -> Ausrichtung {
        self.align
    }

    /// Setzt die Ausrichtung (`Font.align`).
    ///
    /// Aendert weder die Metriken noch die Groesse der Flaeche, nur die Lage
    /// der Zeilen darin - deshalb kein `metriken_neu()`.
    pub fn set_align(&mut self, align: Ausrichtung) {
        self.align = align;
    }

    /// Hoehe ueber der Grundlinie (`TTF_FontAscent`) - mit Kontur
    /// `ascent + 2 * outline`, gemessen an pygame (13, 15, 17, 19 fuer
    /// `Font(None, 24)` bei Kontur 0 bis 3).
    pub fn get_ascent(&self) -> i32 {
        self.ascent + 2 * self.outline
    }

    /// Tiefe unter der Grundlinie - **negativ** (`TTF_FontDescent`).
    pub fn get_descent(&self) -> i32 {
        self.descent
    }

    // ----- FreeTypes eigene Masse ----------------------------------------
    //
    // **SDL_ttf und FreeType runden verschieden, und beide haben recht.**
    // Dieselbe Schrift in derselben Groesse gibt zwei verschiedene
    // Unterlaengen, weil `TTF_FontDescent` aufrundet, wo
    // `FT_Request_Metrics` abrundet. Gemessen an `test_sans.ttf`:
    //
    // ```text
    // Groesse                  8   12   16   24   32   48   64
    // pygame.font  descent    -1   -2   -3   -5   -6  -10  -13
    // freetype     descender  -2   -3   -4   -6   -7  -11  -14
    // ```
    //
    // Die obere Zeile ist `TTF_FontDescent`, die untere
    // `Font.get_sized_descender`. Beide stimmen hier auf den Wert, und
    // beide **muessen** es: `pygame.font` und `pygame.freetype` sind zwei
    // Schnittstellen auf dieselbe Schrift, und jede hat ihre eigene
    // Rundung. Wer eine Zahl fuer beide nimmt, hat an einer Stelle unrecht.
    //
    // Die Formeln stehen in FreeTypes `FT_Request_Metrics`:
    //
    // ```c
    // metrics->ascender  = FT_PIX_CEIL (FT_MulFix(face->ascender,  y_scale));
    // metrics->descender = FT_PIX_FLOOR(FT_MulFix(face->descender, y_scale));
    // metrics->height    = FT_PIX_ROUND(FT_MulFix(face->height,    y_scale));
    // ```

    /// Hoehe ueber der Grundlinie, wie **FreeType** sie meldet.
    pub fn ft_ascender(&self) -> i32 {
        ceil26(mul_fix(i64::from(self.datei.ascender), self.scale))
    }

    /// Tiefe unter der Grundlinie, wie **FreeType** sie meldet - negativ.
    pub fn ft_descender(&self) -> i32 {
        floor26(mul_fix(i64::from(self.datei.descender), self.scale))
    }

    /// Zeilenabstand, wie **FreeType** ihn meldet.
    pub fn ft_height(&self) -> i32 {
        let d = &self.datei;
        let einheiten =
            i64::from(d.ascender) - i64::from(d.descender) + i64::from(d.line_gap);
        (pix_round26(mul_fix(einheiten, self.scale)) >> 6) as i32
    }

    /// Die Hoehe des **Glyphenkastens** - `Font.get_sized_glyph_height`.
    ///
    /// **Genau eins mehr als Auf- plus Unterlaenge**, und das ist
    /// gemessen, nicht hergeleitet. Nachgerechnet an `test_sans.ttf` fuer
    /// sieben Groessen:
    ///
    /// ```text
    /// Groesse        8   12   16   24   32   48   64
    /// asc            8   11   15   22   29   44   58
    /// desc          -2   -3   -4   -6   -7  -11  -14
    /// asc-desc+1    11   15   20   29   37   56   73
    /// pygame        11   15   20   29   37   56   73
    /// ```
    ///
    /// Der naheliegende Weg - die Umrandung aus der `head`-Tabelle mit
    /// dem Groessenfaktor - trifft nur zwei der sieben (10, 15, 20, 30,
    /// 39, 59, 78). Die Umrandung umfasst **jedes** Zeichen der Schrift,
    /// auch solche, die weit ueber die Auflaenge ragen; der Kasten, den
    /// `pad` auffuellt, ist der der **Zeile**.
    pub fn ft_glyph_height(&self) -> i32 {
        self.ft_ascender() - self.ft_descender() + 1
    }

    /// Auflaenge, Unterlaenge und Zeilenhoehe fuer eine **Nenngroesse in
    /// 26.6** - FreeTypes `ft_recompute_scaled_metrics` mit der
    /// ungerundeten `y_scale` (`FT_DivFix(scaled_h, units_per_EM)`). Das
    /// Ergebnis ist 26.6, schon auf ganze Pixel gebracht (CEIL, FLOOR,
    /// ROUND).
    ///
    /// `pygame.freetype` kennt Bruchgroessen, und seine `get_sized_*`
    /// lesen genau diese drei Werte. Bei ganzer Groesse ist es dasselbe wie
    /// [`Font::ft_ascender`] und Co.
    ///
    /// `x26` zaehlt mit, weil ein **Strike** die Groessenanfrage gewinnt,
    /// sobald Breite und Hoehe auf seine Pixelzahl runden
    /// (`FT_Match_Size`) - dann rechnet `FT_Select_Metrics` mit der ganzen
    /// Zahl. Gemessen an `cour.ttf` (Strikes 4 bis 21) bei `11.4`: Hoehe
    /// 12, hier stand 13; bei `(12, 11.4)` passt kein Strike, und es
    /// bleibt 13.
    pub fn ft_kopf26(&self, x26: i64, y26: i64) -> (i64, i64, i64) {
        let d = &self.datei;
        let (ppem, ppem_y) = ((x26 + 32) >> 6, (y26 + 32) >> 6);
        let y26 = if ppem == ppem_y && d.hat_strike(ppem as u16) {
            ppem_y << 6
        } else {
            y26
        };
        let y_scale = crate::fixed::div_fix(y26, i64::from(d.units_per_em.max(1)));
        let asc = (mul_fix(i64::from(d.ascender), y_scale) + 63) & -64;
        let desc = mul_fix(i64::from(d.descender), y_scale) & -64;
        let hoehe = pix_round26(mul_fix(
            i64::from(d.ascender) - i64::from(d.descender) + i64::from(d.line_gap),
            y_scale,
        ));
        (asc, desc, hoehe)
    }

    /// Ob SDL_ttf **jede** Glyphe dieser Schrift ablehnen wuerde: das
    /// CV-Programm scheitert in dieser Groesse (siehe
    /// `Hinter::gescheitert`), und es gibt keinen Strike,
    /// der die Bilder statt des Umrisses lieferte. Dann sagt `size()`
    /// `Couldn't find glyph`, `render()` `Text has zero width`, und
    /// `metrics()` gibt `None` (`SDL_ttf.c` 2.24.0, `Load_Glyph` und
    /// `TTF_Render_Internal`).
    pub fn ladefehler(&self) -> bool {
        let ppem = self.ptsize.max(0) as u32;
        ppem > 0
            && !self.datei.hat_strike(ppem as u16)
            && self.datei.hinting_gescheitert((ppem << 6, ppem << 6))
    }

    /// Punktgroesse, mit der die Schrift geoeffnet wurde.
    pub fn get_point_size(&self) -> i32 {
        self.ptsize
    }

    /// Setzt die Punktgroesse neu (`Font.set_point_size`).
    pub fn set_point_size(&mut self, groesse: i32) {
        self.ptsize = groesse.max(1);
        self.metriken_neu();
    }

    // ----- Namen ----------------------------------------------------------

    /// Der Familienname der Schrift (`Font.name`).
    ///
    /// Gemessen an pygame-ce 2.5.7: `Font(None, 24).name` ist `"FreeSans"`,
    /// und `Font(r"C:\Windows\Fonts\ARIALN.TTF", 24).name` ist `"Arial"` -
    /// **nicht** `"Arial Narrow"`, obwohl das in name-ID 1 steht. Der Grund
    /// ist die ID-Reihenfolge in `Font::namen_holen`.
    ///
    /// `set_bold(true)` aendert den Namen nicht (gemessen).
    pub fn name(&self) -> String {
        self.namen_holen(true)
    }

    /// Der Schnittname der Schrift (`Font.style_name`).
    ///
    /// Gemessen: `Font(None, 24).style_name` ist `"Bold"`, ARIALN liefert
    /// `"Narrow"`.
    pub fn style_name(&self) -> String {
        self.namen_holen(false)
    }

    // ----- Rohmasse in Schrifteinheiten -----------------------------------
    //
    // `pygame.font` gibt Pixel, `pygame.freetype` gibt **Schrifteinheiten**.
    // Gemessen an `freesansbold.ttf`:
    //
    // ```text
    // pygame.font.Font(None, 24).get_height()      16    Pixel
    // pygame.freetype.Font(None, 24).height      1090    Einheiten
    //                              .ascender      800
    //                              .descender    -200
    // ```
    //
    // Die 1090 sind `ascender - descender + line_gap` bei einem Quadrat
    // von 1000 Einheiten. Wer das mit Pixeln verwechselt, bekommt Zahlen,
    // die um den Faktor 45 danebenliegen - und keine Fehlermeldung.

    /// Wie viele Schrifteinheiten ein Geviert hat (`unitsPerEm`).
    pub fn units_per_em(&self) -> u16 {
        self.datei.units_per_em
    }

    /// Die Oberlaenge in **Schrifteinheiten**.
    pub fn roh_ascender(&self) -> i16 {
        self.datei.ascender
    }

    /// Die Unterlaenge in Schrifteinheiten - negativ.
    pub fn roh_descender(&self) -> i16 {
        self.datei.descender
    }

    /// Die Zeilenhoehe in Schrifteinheiten:
    /// `ascender - descender + line_gap`.
    pub fn roh_height(&self) -> i32 {
        self.datei.ascender as i32 - self.datei.descender as i32
            + self.datei.line_gap as i32
    }

    /// Der Zeilenabstand in Schrifteinheiten.
    pub fn roh_line_gap(&self) -> i16 {
        self.datei.line_gap
    }

    // ----- Was fuer eine Schrift ist das? ---------------------------------

    /// Wie viele Spalten das Verdicken anhaengt.
    ///
    /// SDL_ttf setzt ihn fest auf `ptsize / 10`; `pygame.freetype` hat
    /// dafuer den Regler `strength` und rechnet `ptsize * strength`. Beide
    /// meinen dasselbe Verfahren - das fertige Bild wird so oft nach
    /// rechts verschmiert -, nur mit verschiedenen Weiten. Deshalb gibt es
    /// hier einen Setzer statt einer zweiten Betriebsart.
    ///
    /// Gemessen: `freetype.Font(None, 24)` hat `strength = 1/36`, und bei
    /// Groesse 64 macht `STYLE_WIDE` jedes Zeichen genau **zwei** Pixel
    /// breiter - `round(64/36)`.
    /// Die Dicke der Kontur - siehe `crate::glyph::kontur`.
    pub fn get_outline(&self) -> i32 {
        self.outline
    }

    /// Werden eingebaute Pixelbilder benutzt?
    pub fn get_bitmap_strikes(&self) -> bool {
        self.bitmap_strikes
    }

    /// Schaltet sie ein oder aus.
    ///
    /// ## Wer hier entscheidet, und wer nicht
    ///
    /// Diese Kiste prueft zwei Dinge selbst: ob es fuer die verlangte
    /// Pixelgroesse ueberhaupt ein eingebautes Bild gibt, und ob eine
    /// Kontur verlangt ist (dafuer braucht es den Umriss).
    ///
    /// **Alles Weitere entscheidet der Aufrufer**, weil nur er es wissen
    /// kann. `pygame.freetype` teilt seine Stile in zwei Lager:
    ///
    /// ```text
    /// vertraeglich      vertical, wide, underline
    /// unvertraeglich    rotation, strong, oblique
    /// ```
    ///
    /// Der Unterschied ist von hier aus nicht zu sehen, denn `wide` und
    /// `strong` landen beide auf [`Stil::bold`] - sie unterscheiden sich
    /// nur im Zuschlag. `rotation` kennt diese Kiste gar nicht; gedreht
    /// wird eine Stufe weiter oben. Also stellt der Aufrufer den Schalter,
    /// und `freetype_test.test_freetype_Font_use_bitmap_strikes` prueft
    /// beide Lager einzeln ab.
    pub fn set_bitmap_strikes(&mut self, an: bool) {
        if self.bitmap_strikes == an {
            return;
        }
        self.bitmap_strikes = an;
        self.bilder.borrow_mut().clear();
    }

    /// Setzt sie. Die zwischengespeicherten Bilder haengen daran.
    pub fn set_outline(&mut self, n: i32) {
        if self.outline == n {
            return;
        }
        self.outline = n.max(0);
        self.bilder.borrow_mut().clear();
        // Die Zeilenhoehe haengt daran - siehe `metriken_neu`.
        self.metriken_neu();
    }

    /// Siehe `Font::unterstrich_waechst`.
    pub fn set_unterstrich_waechst(&mut self, an: bool) {
        self.unterstrich_waechst = an;
    }

    /// Wie viele Spalten das Verschmieren fuer **fett** nimmt.
    /// 
    /// SDL_ttf nimmt `ptsize / 10`; `pygame.freetype` rechnet fuer
    /// `STYLE_WIDE` anders - siehe `freitext.rs::stil_anwenden`.
    pub fn set_glyph_overhang(&mut self, spalten: i32) {
        if self.glyph_overhang == spalten {
            return;
        }
        self.glyph_overhang = spalten.max(0);
        // Die Bilder haengen am Ueberhang - der Zwischenspeicher muss weg.
        self.bilder.borrow_mut().clear();
    }

    /// Hat die Schrift fuer dieses Zeichen ein eigenes Glyph?
    ///
    /// **Die beiden Textkisten antworten verschieden darauf**, und das
    /// ist gemessen: `pygame.font.Font.metrics` gibt fuer ein unbekanntes
    /// Zeichen die Masse von `.notdef`, `pygame.freetype.Font.get_metrics`
    /// gibt `None`. Wer eine Antwort fuer beide baut, hat eine falsche.
    pub fn hat_zeichen(&self, c: char) -> bool {
        crate::face::Sicht::neu(&self.datei).glyph(c) != 0
    }

    /// Besteht die Schrift aus **Pixeln** statt aus Kurven?
    ///
    /// Gemessen: `pygame.freetype.Font("PyGameMono-8.bdf").scalable` ist
    /// `False`, und dieselbe Schrift laesst sich weder fett noch schraeg
    /// stellen - beides gibt einen `AttributeError`. Eine Bitmapschrift
    /// hat keine Kontur, die man scheren koennte.
    pub fn ist_bitmap(&self) -> bool {
        self.datei.punkte.is_some()
    }

    /// Haben alle Zeichen denselben Vorschub?
    ///
    /// Gemessen: `PyGameMono.otf` ja, `test_sans.ttf` nein.
    pub fn feste_breite(&self) -> bool {
        match &self.datei.punkte {
            Some(b) => b.feste_breite,
            None => self
                .datei
                .tabellen()
                .map(|tt| tt.is_monospaced())
                .unwrap_or(false),
        }
    }

    /// Ob eine Groesse den einen Strike einer **Bitmapschrift** trifft -
    /// `_PGFT_GetFontSized` und FreeTypes `BDF_Size_Request`, uebertragen:
    ///
    /// * Eine Zahl (`y26 == 0`): trifft `FX6_ROUND` von ihr das von
    ///   `bsize->size`, wird nach `(x_ppem, y_ppem)` des Strikes gefragt,
    ///   sonst nach der Zahl selbst. Deshalb gehen bei `PyGameMono-8.bdf`
    ///   auch `8.4` und `7.6`.
    /// * Zwei Zahlen: wie sie sind.
    /// * `BDF_Size_Request` sieht dann **nur die Hoehe** an: gerundet aus
    ///   `FT_REQUEST_HEIGHT` muss sie `y_ppem` des Strikes sein. Die Breite
    ///   zaehlt nicht - `size=(16, 8)` geht, `size=(8, 16)` nicht.
    ///
    /// `x26`/`y26` sind `DBL_TO_FX6` der Groesse, `aufloesung` die der
    /// Schrift. Fuer eine Umrissschrift immer `true`.
    pub fn bitmap_trifft(&self, x26: i64, y26: i64, aufloesung: i64) -> bool {
        let Some(b) = &self.datei.punkte else {
            return true;
        };
        let hoehe = if y26 == 0 {
            if (x26 + 32) & -64 == (b.groesse26 + 32) & -64 {
                b.y_ppem26
            } else {
                x26
            }
        } else {
            y26
        };
        // `FT_Set_Char_Size`: unter einem Pixel wird es einer.
        let hoehe = hoehe.max(64);
        let anfrage = if aufloesung > 0 {
            (hoehe * aufloesung + 36) / 72
        } else {
            hoehe
        };
        (anfrage + 32) >> 6 == (b.y_ppem26 + 32) >> 6
    }

    /// Die eingebauten Groessen einer Bitmapschrift.
    ///
    /// Je Eintrag `(Punktgroesse, Breite, Hoehe, x_ppem, y_ppem)` - genau
    /// die fuenf Zahlen, die `freetype.Font.get_sizes()` liefert. Eine
    /// Umrissschrift gibt eine leere Liste.
    pub fn feste_groessen(&self) -> Vec<(i32, i32, i32, f64, f64)> {
        match &self.datei.punkte {
            Some(b) => vec![(
                b.punktgroesse,
                b.ppem.0 as i32,
                b.ppem.1 as i32,
                b.ppem.0 as f64,
                b.ppem.1 as f64,
            )],
            // **Auch eine Umrissschrift kann Pixelbilder mitbringen.**
            //
            // `PyGameMono.otf` hat zwei davon (8x8 und 19x19), und
            // `freetype_test` zaehlt sie: `f.fixed_sizes` ist dort 2.
            // Sie stehen in `EBLC` (oder `CBLC`, oder `bloc` bei alten
            // Apple-Schriften) - drei Namen fuer dieselbe Tabelle.
            None => self
                .datei
                .tabellen()
                .map(|tt| strichgroessen(&tt))
                .unwrap_or_default(),
        }
    }

    /// Die Kette aus `sfnt_load_face` (FreeType 2.13, `sfobjs.c` Z. 1037 ff.),
    /// uebertragen statt nachempfunden.
    ///
    /// ```c
    /// if ( face->os2.version != 0xFFFFU && face->os2.fsSelection & 256 ) {
    ///   GET_NAME( TYPOGRAPHIC_FAMILY, … );  if (!…) GET_NAME( FONT_FAMILY, … );
    ///   GET_NAME( TYPOGRAPHIC_SUBFAMILY, … ); if (!…) GET_NAME( FONT_SUBFAMILY, … );
    /// } else {
    ///   GET_NAME( WWS_FAMILY, … ); … TYPOGRAPHIC_FAMILY … FONT_FAMILY …
    ///   GET_NAME( WWS_SUBFAMILY, … ); … TYPOGRAPHIC_SUBFAMILY … FONT_SUBFAMILY …
    /// }
    /// ```
    ///
    /// Die beiden `ignore_typographic_*`-Schalter kommen aus
    /// `FT_PARAM_TAG_IGNORE_TYPOGRAPHIC_FAMILY`; SDL_ttf oeffnet die Schrift
    /// ohne Parameter, also sind sie immer aus.
    ///
    /// Dass die typographischen IDs 16/17 vorgehen, ist der Grund fuer das
    /// ueberraschende `"Arial"` / `"Narrow"` oben. Nachgemessen ueber alle
    /// 327 Schriften in `C:\Windows\Fonts` plus `freesansbold.ttf`: die Kette
    /// trifft pygames Werte 327/327; nur ID 1 bzw. 2 zu nehmen traefe 295
    /// bzw. 293.
    fn namen_holen(&self, familie: bool) -> String {
        // Eine Bitmapschrift hat keine `name`-Tabelle; ihre Namen stehen
        // als Eigenschaften in der Datei (`FAMILY_NAME`, `WEIGHT_NAME`).
        if let Some(b) = &self.datei.punkte {
            return if familie {
                b.familie.clone()
            } else {
                b.schnitt.clone()
            };
        }
        let Some(tt) = self.datei.tabellen() else {
            return String::new();
        };
        let Some(daten) = roh(&tt, b"name") else {
            return String::new();
        };
        let eintraege = namenstabelle(daten);

        // "Bit 8 of the `fsSelection' field in the `OS/2' table denotes a
        // WWS-only font face." Fehlt die Tabelle, greift in FreeType der
        // else-Zweig - `version != 0xFFFF` ist dann falsch.
        let wws_only = roh(&tt, b"OS/2")
            .filter(|d| d.len() >= 64)
            .is_some_and(|d| u16::from_be_bytes([d[62], d[63]]) & 256 != 0);

        let ids: &[u16] = match (familie, wws_only) {
            (true, true) => &[NAME_TYPO_FAMILY, NAME_FAMILY],
            (true, false) => &[NAME_WWS_FAMILY, NAME_TYPO_FAMILY, NAME_FAMILY],
            (false, true) => &[NAME_TYPO_SUBFAMILY, NAME_SUBFAMILY],
            (false, false) => &[NAME_WWS_SUBFAMILY, NAME_TYPO_SUBFAMILY, NAME_SUBFAMILY],
        };
        ids.iter()
            .find_map(|&id| name_waehlen(&eintraege, id))
            .unwrap_or_default()
    }

    /// Ist fett eingeschaltet?
    ///
    /// **Wie `TTF_GetFontStyle`: ein fetter Schnitt ist immer fett.**
    /// Gemessen: `Font("consolab.ttf", 14).get_bold()` ist bei pygame
    /// `True`, ohne dass je `set_bold` gerufen wurde, und bleibt es nach
    /// `set_bold(False)`. Hier stand der blosse Schalter - und die bekannte
    /// Grenze `sysfont` erklaerte das falsche `False` mit pygames
    /// `sysfont.py`.
    pub fn get_bold(&self) -> bool {
        self.stil.bold || self.datei.schnitt_fett
    }

    /// Schaltet fett - mit SDL_ttfs Regel (`TTF_SetFontStyle`): ist der
    /// Schnitt schon fett, wird **nichts** nachgemacht. Gemessen an
    /// `consolab.ttf` mit `set_bold(True)`: pygame `size("Hallo")` 42, die
    /// alte Fassung hier 47 - sie hatte die Fettspalte zusaetzlich
    /// verschmiert.
    pub fn set_bold(&mut self, an: bool) {
        self.set_bold_roh(an && !self.datei.schnitt_fett);
    }

    /// Schaltet das **nachgemachte** Fett ohne Ruecksicht auf den Schnitt -
    /// fuer `pygame.freetype`, das kein SDL_ttf ist.
    pub fn set_bold_roh(&mut self, an: bool) {
        if self.stil.bold != an {
            self.stil.bold = an;
            self.metriken_neu();
        }
    }

    /// Ist kursiv eingeschaltet - oder der Schnitt kursiv
    /// (`TTF_GetFontStyle`, wie bei [`Font::get_bold`])?
    pub fn get_italic(&self) -> bool {
        self.stil.italic || self.datei.schnitt_kursiv
    }

    /// Schaltet kursiv - mit SDL_ttfs Regel: ein kursiver Schnitt wird
    /// nicht noch einmal geschert.
    pub fn set_italic(&mut self, an: bool) {
        self.set_italic_roh(an && !self.datei.schnitt_kursiv);
    }

    /// Schaltet das **nachgemachte** Kursiv ohne Ruecksicht auf den
    /// Schnitt - fuer `pygame.freetype`.
    pub fn set_italic_roh(&mut self, an: bool) {
        if self.stil.italic != an {
            self.stil.italic = an;
            self.metriken_neu();
        }
    }

    /// Wird unterstrichen?
    pub fn get_underline(&self) -> bool {
        self.stil.underline
    }

    /// Schaltet den Unterstrich.
    pub fn set_underline(&mut self, an: bool) {
        if self.stil.underline != an {
            self.stil.underline = an;
            self.metriken_neu();
        }
    }

    /// Wird durchgestrichen?
    pub fn get_strikethrough(&self) -> bool {
        self.stil.strikethrough
    }

    /// Schaltet den Durchstrich.
    pub fn set_strikethrough(&mut self, an: bool) {
        if self.stil.strikethrough != an {
            self.stil.strikethrough = an;
            self.metriken_neu();
        }
    }

    /// Je Zeichen `(minx, maxx, miny, maxy, advance)` - genau die fuenf
    /// Zahlen aus `TTF_GlyphMetrics32`.
    ///
    /// `None` steht fuer ein Zeichen, das die Schrift nicht kennt; pygame
    /// liefert dort ebenfalls `None`.
    pub fn metrics(&self, text: &str) -> Vec<Option<GlyphMass>> {
        let sicht = Sicht::neu(&self.datei);
        text.chars()
            .map(|c| {
                // **Ein fehlendes Zeichen gibt das Ersatzglyph, nicht
                // `None`.**
                //
                // Gemessen an freesansbold: pygame liefert fuer U+212A
                // (Kelvin), U+4E2D, U+1F600 und U+0E01 **dieselben** Masse
                // `(0, 5, 0, 9, 6)` - das ist die Huelle von `.notdef`,
                // Glyph 0. Hier stand `glyph_index(c)?`, und das machte
                // aus jedem unbekannten Zeichen ein `None`.
                //
                // `font_test.test_metrics` prueft das ueber den Umweg
                // eines UTF-16-Bytespaars, dessen Latin-1-Lesung zwei
                // unbekannte Zeichen ergibt; dort muessen **zwei** Eintraege
                // stehen, und keiner davon `None`.
                //
                // `None` bleibt moeglich - die Schnittstelle sagt es zu -,
                // nur kommt es bei einer Schrift mit `.notdef` nicht vor.
                let gid = sicht.glyph_sdl_ttf(c);
                let b = self.bild(&sicht, gid, false);
                let vorschub = self.vorschub_sdl(&sicht, gid)
                    + if self.stil.bold {
                        i64::from(self.glyph_overhang) << 6
                    } else {
                        0
                    };
                // `TTF_GlyphMetrics32` legt die Kontur auf `maxx` und
                // `maxy`, nicht auf die andere Seite.
                Some((
                    b.m_left,
                    b.m_left + b.m_width + 2 * self.outline,
                    b.m_top - b.m_rows,
                    b.m_top + 2 * self.outline,
                    ceil26(vorschub),
                ))
            })
            .collect()
    }

    // ----- Glyphen --------------------------------------------------------

    /// Der Vorschub, den SDL_ttf (und `hb_ft`) fuer eine Glyphe sieht, in
    /// 26.6: kommt sie aus einem **Strike**, steht er in dessen sbit-Massen
    /// (`load_sbit_image`), sonst ist es der gehintete. Fehlt er im Strike,
    /// nimmt `TT_Load_Glyph` `linearHoriAdvance` mal der Skala des Strikes.
    /// Gemessen an Calibri bei 13 (Strike): `metrics("a")` gibt bei pygame
    /// den Vorschub 7, hier stand der gehintete 6.
    fn vorschub_sdl(&self, sicht: &Sicht, gid: u16) -> i64 {
        let ppem = self.ptsize.max(0) as u16;
        if self.bitmap_strikes && self.outline == 0 && self.datei.strike(gid, ppem).is_some() {
            if let Some((px, _)) = self.datei.strike_masse(gid, ppem) {
                if px != 0 || sicht.ist_cff() {
                    return px << 6;
                }
                let skala = crate::fixed::div_fix(
                    i64::from(ppem) << 6,
                    i64::from(self.datei.units_per_em.max(1)),
                );
                return (mul_fix(sicht.vorschub_einheiten(gid), skala) + 63) & -64;
            }
        }
        let ppem = u32::from(ppem);
        if let Some(v) = self.datei.gehinteter_vorschub((ppem << 6, ppem << 6), gid) {
            return v;
        }
        sicht.vorschub26(gid, self.scale)
    }

    fn bild(&self, sicht: &Sicht, gid: u16, mono: bool) -> Rc<GlyphBild> {
        let key = (gid, self.stil.schluessel(mono, self.bitmap_strikes));
        if let Some(b) = self.bilder.borrow().get(&key) {
            return Rc::clone(b);
        }
        let _ = sicht;
        // **Nur wenn sie passt und der Stil sie zulaesst.** Eine Kontur
        // (`STYLE_CONTOUR`) ersetzt das Zeichen durch seinen Rand; dafuer
        // braucht es den Umriss, nicht ein fertiges Bild. Fett, kursiv und
        // Drehung entscheidet der Aufrufer ueber den Schalter - siehe
        // [`Font::set_bitmap_strikes`].
        let strike = if self.bitmap_strikes && self.outline == 0 {
            self.datei.strike(gid, self.ptsize.max(0) as u16)
        } else {
            None
        };
        // **FreeTypes Weg, wo er gilt**: geglaettet (einfarbig rastert
        // FreeType mit `ftraster.c`, einem anderen Rasterer) und ohne Kontur
        // (die zieht `FT_Stroker`). Kursiv schert SDL_ttf den gehinteten
        // Umriss vor dem Rastern - das liefert `gehintet` als zweites Bild.
        // Fett ist kein Hindernis - das verschmiert SDL_ttf erst auf dem
        // fertigen Bild.
        let gehintet = if strike.is_none() && !mono && self.outline == 0 {
            self.datei
                .gehintet(self.ptsize.max(0) as u32, gid, self.stil.italic)
        } else {
            None
        };
        let (gerade, schief) = match &gehintet {
            Some((g, s)) => (Some(g), s.as_ref()),
            None => (None, None),
        };
        // Einfarbig rastert FreeType mit `ftraster.c`, eine Kontur zieht
        // `FT_Stroker` - siehe `crate::ftraster` und `crate::ftstroke`.
        let einfarbig = if strike.is_none() && (mono || self.outline > 0) {
            self.datei.gehintet_fertig(
                self.ptsize.max(0) as u32,
                gid,
                self.stil.italic,
                mono,
                self.outline,
            )
        } else {
            None
        };
        let b = Rc::new(rastern(
            strike.as_ref(),
            gerade,
            schief,
            einfarbig.as_ref(),
            self.datei.raster(),
            self.datei.punkte.as_ref(),
            gid,
            self.ptsize as f32,
            self.stil,
            self.glyph_overhang,
            self.height,
            mono,
            self.outline,
        ));
        self.bilder.borrow_mut().insert(key, Rc::clone(&b));
        b
    }

    // ----- Vermessen ------------------------------------------------------

    /// `TTF_Size_Internal` fuer **eine** Zeile.
    ///
    /// Die Schreibmarke `x` sammelt in 26.6 und wird **nicht** je Zeichen auf
    /// ganze Pixel gerundet. Gemessen: bei freesansbold liefert
    /// `size("x" * 11) - size("x")` genau 212 Pixel; mit Rundung waeren es
    /// 210. Die Vorschuebe selbst sind dagegen ganze Pixel, nur das Kerning
    /// bringt Bruchteile hinein.
    fn vermessen(&self, sicht: &Sicht, text: &str, mono: bool) -> Zeilenmass {
        let mut x: i64 = 0;
        let mut minx = 0i32;
        let mut maxx = 0i32;
        let mut miny = 0i32;
        let mut maxy = self.height;
        let mut prev_adv: i64 = 0;
        let mut prev_gid: Option<u16> = None;
        let mut plaetze = Vec::with_capacity(text.len());

        let ascent26 = i64::from(self.ascent) << 6;
        // `advance_if_bold` - SDL_ttf legt es **ausserhalb** von HarfBuzz drauf.
        let fett26 = if self.stil.bold {
            i64::from(self.glyph_overhang) << 6
        } else {
            0
        };

        // **Der HarfBuzz-Weg** - siehe `crate::gestalt`. Glyphenfolge (GSUB)
        // und Lagen (GPOS, auch senkrecht) kommen von dort, genau wie in
        // `TTF_Size_Internal`:
        //
        //     pos_x = x + x_offset;   pos_y = y + ascent - y_offset;
        //     x += x_advance + advance_if_bold;   y += y_advance;
        let vorschub = |gid: u16| self.vorschub_sdl(sicht, gid) as i32;
        if let Some(lagen) = sicht.datei.formen(text, self.scale, &vorschub) {
            let mut y: i64 = 0;
            for l in lagen {
                let b = self.bild(sicht, l.gid, mono);
                let pos_x = x + i64::from(l.x_offset);
                let pos_y = y + ascent26 - i64::from(l.y_offset);
                x += i64::from(l.x_advance) + fett26;
                y += i64::from(l.y_advance);
                plaetze.push(Platz {
                    gid: l.gid,
                    x26: pos_x,
                    y26: pos_y,
                });
                // Fuer die Masse zaehlt der Metrik-Kasten, nicht das Bild -
                // siehe [`crate::glyph::GlyphBild`].
                let px = floor26(pos_x) + b.m_left;
                let py = floor26(pos_y) - b.m_top;
                minx = minx.min(px);
                maxx = maxx.max(px + b.m_width);
                miny = miny.min(py);
                maxy = maxy.max(py + b.m_rows);
            }
            // "Erlaubt eine Zeichenkette aus nur einem Leerzeichen (Fehler
            // 4344)." - auf diesem Weg ist `prev_advance` in SDL_ttf immer 0.
            maxx = maxx.max(floor26(x));
        } else {
            // Ohne HarfBuzz (eine Bitmapschrift): Zeichen fuer Zeichen ueber
            // `cmap`, Vorschub und Paar-Kerning - SDL_ttfs zweiter Weg.
            for c in text.chars() {
                let gid = sicht.glyph(c);
                let b = self.bild(sicht, gid, mono);

                x += prev_adv;
                prev_adv = self.vorschub_sdl(sicht, gid) + fett26;
                if let Some(p) = prev_gid {
                    let k = sicht.kerning(p, gid);
                    if k != 0 {
                        x += mul_fix(i64::from(k), self.scale);
                    }
                }
                prev_gid = Some(gid);

                plaetze.push(Platz {
                    gid,
                    x26: x,
                    y26: ascent26,
                });

                let pos_x = floor26(x) + b.m_left;
                let pos_y = self.ascent - b.m_top;
                minx = minx.min(pos_x);
                maxx = maxx.max(pos_x + b.m_width);
                miny = miny.min(pos_y);
                maxy = maxy.max(pos_y + b.m_rows);
            }
            maxx = maxx.max(floor26(x + prev_adv));
        }

        // `TTF_Size_Internal`: die Kontur kommt **hier** dazu - `w` nur,
        // wenn es eine Breite gibt, `h` immer, und beide Startpunkte um
        // `n` nach innen.
        let fo = self.outline;
        let w = maxx - minx;
        Zeilenmass {
            w: if w != 0 { w + 2 * fo } else { 0 },
            h: maxy - miny + 2 * fo,
            xstart: if minx < 0 { -minx } else { 0 } + fo,
            ystart: if miny < 0 { -miny } else { 0 } + fo,
            plaetze,
        }
    }

    /// Breite und Hoehe des Textes - Gegenstueck zu `Font.size`.
    ///
    /// Achtung: das ist **nicht** die Groesse der Flaeche aus [`Font::render`].
    /// `size` kennt keine Zeilenumbrueche und gibt die Texthoehe zurueck, die
    /// Flaeche ist `linesize` hoch. Beides ist in pygame genau so.
    pub fn size(&self, text: &str) -> (u32, u32) {
        let sicht = Sicht::neu(&self.datei);
        let m = self.vermessen(&sicht, text, false);
        (m.w.max(0) as u32, m.h.max(0) as u32)
    }

    /// Wie pygame den Text in Zeilen zerlegt: nur `\n` trennt, und ein
    /// abschliessendes `\n` erzeugt **keine** leere Schlusszeile.
    fn zeilen(text: &str) -> Vec<&str> {
        let mut z: Vec<&str> = text.split('\n').collect();
        if z.len() > 1 && text.ends_with('\n') {
            z.pop();
        }
        z
    }

    // ----- Rendern --------------------------------------------------------

    /// Setzt Text auf eine neue Flaeche - Gegenstueck zu `Font.render`.
    ///
    /// * `antialias = true`, `background = None`: Flaeche mit `SRCALPHA`,
    ///   RGB konstant `color`, Deckung im Alphakanal.
    /// * `antialias = true` mit Hintergrund: undurchsichtige Flaeche, Farbe
    ///   zwischen Hintergrund und Vordergrund gemischt.
    /// * `antialias = false`: undurchsichtige Flaeche mit **Schluesselfarbe**;
    ///   die Schluesselfarbe ist `255 - color` je Kanal, so wie SDL_ttf sie
    ///   in die Palette schreibt. Mit Hintergrund faellt die Schluesselfarbe
    ///   weg.
    /// * Leerer Text: Flaeche `0 x get_height()`, ohne `SRCALPHA`.
    pub fn render(
        &self,
        text: &str,
        antialias: bool,
        color: impl IntoColor,
        background: Option<Color>,
    ) -> Result<Surface, FontError> {
        let fg = color.into_color();

        if text.is_empty() {
            // `font.c`: der Sonderfall legt eine Flaeche 0 x FontHeight an -
            // ohne Alphakanal, egal welche Betriebsart gewaehlt war.
            return Ok(Surface::with_flags((0, self.height.max(0) as u32), 0));
        }

        let mono = !antialias;
        let sicht = Sicht::neu(&self.datei);

        // Schritt 1: das ganze Stueck vermessen - daraus kommen Zeilenhoehe
        // und, bei nur einer Zeile, auch die Breite.
        let ganz = self.vermessen(&sicht, text, mono);
        if ganz.w <= 0 {
            return Err(FontError::OhneBreite);
        }

        let zeilen = Font::zeilen(text);
        let lineskip = self.get_linesize();
        let rowheight = ganz.h.max(lineskip);

        let breite = if zeilen.len() > 1 {
            zeilen
                .iter()
                .map(|z| self.vermessen(&sicht, z, mono).w)
                .max()
                .unwrap_or(0)
                .max(1)
        } else {
            ganz.w
        };
        let mut hoehe = rowheight + lineskip * (zeilen.len() as i32 - 1);
        // **Der Unterstrich zaehlt mit - bei `pygame.freetype`.**
        //
        // Er liegt unter der Grundlinie und ragt bei manchen Schriften
        // aus dem Kasten heraus, den die Zeichen brauchen. SDL_ttf
        // schneidet ihn dort ab; `pygame.freetype` macht den Kasten
        // groesser. Siehe `Font::unterstrich_waechst`.
        if self.unterstrich_waechst {
            let letzte = lineskip * (zeilen.len() as i32 - 1);
            if self.stil.underline {
                hoehe = hoehe.max(letzte + self.underline_top_row + self.line_thickness);
            }
            if self.stil.strikethrough {
                hoehe = hoehe.max(letzte + self.strikethrough_top_row + self.line_thickness);
            }
        }
        let hoehe = hoehe;

        let (w, h) = (breite.max(0) as usize, hoehe.max(0) as usize);
        // Ein Byte Deckung je Pixel - das entspricht genau dem, was SDL_ttf
        // in die Flaeche schreibt: bei Blended der Alphawert, bei Shaded der
        // Palettenindex, bei Solid die 0/1 der Einfarbenmaske. In allen drei
        // Faellen mit `|=`.
        let mut deckung = vec![0u8; w * h];

        for (i, zeile) in zeilen.iter().enumerate() {
            let m = if zeilen.len() == 1 {
                // Bei genau einer Zeile ist der Text (bis auf ein moegliches
                // abschliessendes \n) derselbe - trotzdem neu vermessen, denn
                // das \n hat einen eigenen Glyphen und faellt hier weg.
                self.vermessen(&sicht, zeile, mono)
            } else {
                self.vermessen(&sicht, zeile, mono)
            };
            let ystart = m.ystart + i as i32 * lineskip;

            // `TTF_Render_Wrapped_Internal`, "Control left/right/center align
            // of each bit of text":
            //
            // ```c
            // if (align == RIGHT)       xoffset = width - line_width;
            // else if (align == CENTER) xoffset = width / 2 - line_width / 2;
            // else                      xoffset = 0;
            // xoffset = SDL_max(0, xoffset);
            // ```
            //
            // Die Mitte ist **nicht** `(width - line_width) / 2`: bei
            // ungeraden Zahlen liegt der Unterschied bei einem Pixel. Gemessen
            // an pygame mit `Font(None, 24)` und `"iii\nWWWWWW"`: Flaeche 90
            // breit, `size("iii")` 12 breit, die kurze Zeile beginnt bei
            // Mitte auf Spalte 40 statt 39 - also 90/2 - 12/2 = 39 plus der
            // Spalte 1, auf der sie schon bei Links liegt.
            let xoffset = match self.align {
                Ausrichtung::Rechts => breite - m.w,
                Ausrichtung::Mitte => breite / 2 - m.w / 2,
                Ausrichtung::Links => 0,
            }
            .max(0);

            self.zeile_zeichnen(
                &sicht,
                &m,
                &mut deckung,
                w,
                h,
                m.xstart + xoffset,
                ystart,
                mono,
            );

            // Die Striche fangen bei `xoffset` an, **ohne** `xstart` - so
            // steht es in `Draw_Line(font, textbuf, xoffset, …)` und so ist es
            // gemessen: bei `align = Rechts`, Unterstrich und `"ii\nWWWWWW"`
            // liegt der Strich der ersten Zeile auf den Spalten 82 bis 89,
            // die Zeichen dagegen auf 83 bis 89.
            if self.stil.underline {
                strich(
                    &mut deckung,
                    w,
                    h,
                    xoffset,
                    ystart + self.underline_top_row,
                    m.w,
                    self.line_thickness,
                );
            }
            if self.stil.strikethrough {
                strich(
                    &mut deckung,
                    w,
                    h,
                    xoffset,
                    ystart + self.strikethrough_top_row,
                    m.w,
                    self.line_thickness,
                );
            }
        }

        Ok(self.flaeche_bauen(&deckung, w, h, fg, background, antialias))
    }

    /// Setzt Text direkt auf eine vorhandene Flaeche.
    ///
    /// In `pygame.font` gibt es das nicht - nur in `pygame.freetype`. Hier
    /// ist es drin, weil das umgebende Projekt sonst fuer jede Anzeige eine
    /// Flaeche anlegen und gleich wieder wegwerfen muesste. Das Ergebnis ist
    /// dasselbe wie `ziel.blit(&font.render(...), dest)`; zurueck kommt das
    /// beschriebene Rechteck.
    pub fn render_to(
        &self,
        ziel: &mut Surface,
        dest: (i32, i32),
        text: &str,
        antialias: bool,
        color: impl IntoColor,
        background: Option<Color>,
    ) -> Result<Rect, FontError> {
        let s = self.render(text, antialias, color, background)?;
        Ok(ziel.blit(&s, dest))
    }

    #[allow(clippy::too_many_arguments)]
    fn zeile_zeichnen(
        &self,
        sicht: &Sicht,
        m: &Zeilenmass,
        deckung: &mut [u8],
        w: usize,
        h: usize,
        xstart: i32,
        ystart: i32,
        mono: bool,
    ) {
        for p in &m.plaetze {
            let b = self.bild(sicht, p.gid, mono);
            if b.rows == 0 || b.width == 0 {
                continue;
            }
            // `Render_Line`: `x = xstart + FT_FLOOR(x) + left`,
            // `y = ystart + FT_FLOOR(y) - top`.
            let x0 = xstart + floor26(p.x26) + b.left;
            let y0 = ystart + floor26(p.y26) - b.top;
            for r in 0..b.rows {
                let y = y0 + r as i32;
                if y < 0 || y as usize >= h {
                    continue;
                }
                let zeile = y as usize * w;
                for c in 0..b.width {
                    let x = x0 + c as i32;
                    if x < 0 || x as usize >= w {
                        continue;
                    }
                    // `|=` wie in `BG_Blended_Opaque` - ueberlappende Zeichen
                    // verodern ihre Deckung, sie mischen sie nicht.
                    deckung[zeile + x as usize] |= b.deckung[r * b.width + c];
                }
            }
        }
    }

    fn flaeche_bauen(
        &self,
        deckung: &[u8],
        w: usize,
        h: usize,
        fg: Color,
        bg: Option<Color>,
        antialias: bool,
    ) -> Surface {
        let mut s = match (antialias, bg) {
            (true, None) => Surface::with_flags((w as u32, h as u32), SRCALPHA),
            _ => Surface::with_flags((w as u32, h as u32), 0),
        };

        let palette: Vec<u32> = match (antialias, bg) {
            (true, None) => Vec::new(),
            (true, Some(bg)) => palette_shaded(fg, bg),
            (false, bg) => {
                // `Create_Surface_Solid`: Index 0 ist die Gegenfarbe und wird
                // zur Schluesselfarbe, Index 1 ist die Schriftfarbe. Gibt der
                // Aufrufer einen Hintergrund an, ersetzt `font.c` Index 0 und
                // loescht die Schluesselfarbe.
                let null = bg.unwrap_or(Color::rgb(255 - fg.r, 255 - fg.g, 255 - fg.b));
                vec![
                    0xFF00_0000
                        | (u32::from(null.r) << 16)
                        | (u32::from(null.g) << 8)
                        | u32::from(null.b),
                    0xFF00_0000
                        | (u32::from(fg.r) << 16)
                        | (u32::from(fg.g) << 8)
                        | u32::from(fg.b),
                ]
            }
        };

        if antialias && bg.is_none() {
            let grund = (u32::from(fg.r) << 16) | (u32::from(fg.g) << 8) | u32::from(fg.b);
            s.mit_pixeln(|daten, breite, (ox, oy)| {
                for y in 0..h {
                    let zeile = (oy + y) * breite + ox;
                    for x in 0..w {
                        daten[zeile + x] = grund | (u32::from(deckung[y * w + x]) << 24);
                    }
                }
            });
        } else if antialias {
            s.mit_pixeln(|daten, breite, (ox, oy)| {
                for y in 0..h {
                    let zeile = (oy + y) * breite + ox;
                    for x in 0..w {
                        daten[zeile + x] = palette[deckung[y * w + x] as usize];
                    }
                }
            });
        } else {
            s.mit_pixeln(|daten, breite, (ox, oy)| {
                for y in 0..h {
                    let zeile = (oy + y) * breite + ox;
                    for x in 0..w {
                        let i = usize::from(deckung[y * w + x] != 0);
                        daten[zeile + x] = palette[i];
                    }
                }
            });
            // **Acht Bit, nicht zweiunddreissig - aber nur ohne Grund.**
            //
            // Gemessen: `render(text, farbe)` ohne Hintergrund gibt eine
            // Flaeche mit acht Bit und Schluesselfarbe, `render(text,
            // farbe, hintergrund)` eine mit **zweiunddreissig**. Die
            // Tafel ist also nicht die Betriebsart, sondern die Folge
            // davon, dass es nur zwei Farben gibt und eine davon
            // durchsichtig sein soll.
            if bg.is_none() {
                s.set_colorkey(Some(Color::rgb(255 - fg.r, 255 - fg.g, 255 - fg.b)));
                acht_bit(&mut s, &palette);
            }
        }

        s
    }
}

/// Macht aus einer fertigen Zweifarbenflaeche eine mit Tafel.
///
/// **Acht Bit, nicht zweiunddreissig.**
            //
            // `Create_Surface_Solid` legt in SDL_ttf eine Flaeche mit
            // Farbtafel an, und pygame reicht sie durch. Gemessen an
            // `pygame.freetype.Font(None, 24)` mit `antialiased = False`:
            //
            // ```text
            // surf.get_bitsize()   8
            // surf.get_flags()     4096   (SRCCOLORKEY)
            // surf.get_palette()   (Gegenfarbe, Schriftfarbe, ...)
            // ```
            //
            // Am Speicher aendert das nichts - rustgame legt ohnehin
            // Farben ab, nicht Indizes. Es aendert die **Auskunft**, und
            // die wird geprueft: `freetype_test.test_freetype_Font_render_mono`
            // vergleicht die Tiefe mit einer 8.
/// Die eingebauten Pixelgroessen aus `EBLC`, `CBLC` oder `bloc`.
///
/// `ttf-parser` liest diese Tabelle, gibt aber nur einzelne Bilder heraus -
/// die **Liste** der Groessen bleibt innen. Sie steht am Anfang der Tabelle
/// und ist einfach genug, um sie hier zu lesen:
///
/// ```text
///  0  version        u32
///  4  numSizes       u32
///  8  je Groesse 48 Byte:
///       ...
///      +44 ppemX     u8
///      +45 ppemY     u8
///      +46 bitDepth  u8
/// ```
///
/// Die Punktgroesse steht dort **nicht**; FreeType setzt sie bei einer
/// eingebauten Groesse gleich der senkrechten Pixelzahl, und genau das
/// tut diese Zeile auch.
fn strichgroessen(tt: &TtfFace<'_>) -> Vec<(i32, i32, i32, f64, f64)> {
    let roh = tt.raw_face();
    let daten = [b"EBLC", b"CBLC", b"bloc"]
        .iter()
        .find_map(|t| roh.table(Tag::from_bytes(t)));
    let Some(d) = daten else {
        return Vec::new();
    };
    if d.len() < 8 {
        return Vec::new();
    }
    let anzahl = u32::from_be_bytes([d[4], d[5], d[6], d[7]]) as usize;
    let mut aus = Vec::new();
    for i in 0..anzahl {
        let a = 8 + i * 48;
        if a + 46 > d.len() {
            break;
        }
        let (px, py) = (d[a + 44] as i32, d[a + 45] as i32);
        if px == 0 || py == 0 {
            continue;
        }
        aus.push((py, px, py, px as f64, py as f64));
    }
    aus
}

fn acht_bit(s: &mut Surface, palette: &[u32]) {
    if palette.len() < 2 {
        return;
    }
    // **`umformen`, nicht `set_format` und danach `set_palette`.** Seit die
    // Bytes im gemeldeten Format liegen, rechnet `set_format` die Pixel
    // gegen die **alte** Tafel um - hier gab es keine -, und `set_palette`
    // faerbt nicht nach. Jedes Pixel landete so auf einem Index ausserhalb
    // der Zweifarbentafel, und `render(text, False, ..)` war **leer**, in
    // jeder Groesse und fuer jeden Text. Gefunden in Runde 19, als der neue
    // Schwarzweiss-Rasterer kein einziges Bild veraenderte.
    if let Ok(f8) = rustgame_core::surface::Format::standard(8, false) {
        s.umformen(
            f8,
            Some(vec![Color::from_argb(palette[0]), Color::from_argb(palette[1])]),
        );
    }
}

// ----- name-Tabelle -------------------------------------------------------
//
// ttf-parser haette dafuer ein `name`-Modul, aber die Kiste ist hier mit
// `default-features = false` eingebunden und das Merkmal ist nicht dabei.
// Die Tabelle von Hand zu lesen ist ohnehin naeher an der Vorlage: FreeType
// waehlt seinen Eintrag nach Regeln, die ttf-parser nicht kennt.

/// name-IDs aus `ttnameid.h`.
const NAME_FAMILY: u16 = 1;
const NAME_SUBFAMILY: u16 = 2;
const NAME_TYPO_FAMILY: u16 = 16;
const NAME_TYPO_SUBFAMILY: u16 = 17;
const NAME_WWS_FAMILY: u16 = 21;
const NAME_WWS_SUBFAMILY: u16 = 22;

/// Die Rohbytes einer Tabelle.
fn roh<'a>(tt: &TtfFace<'a>, tag: &[u8; 4]) -> Option<&'a [u8]> {
    tt.raw_face().table(Tag::from_bytes(tag))
}

/// Ein Eintrag der `name`-Tabelle, so weit er hier gebraucht wird.
struct Namenseintrag<'a> {
    platform: u16,
    encoding: u16,
    sprache: u16,
    id: u16,
    bytes: &'a [u8],
}

/// Zerlegt die `name`-Tabelle (Format 0 und 1 sind im Kopf gleich).
fn namenstabelle(daten: &[u8]) -> Vec<Namenseintrag<'_>> {
    let lies = |i: usize| -> Option<u16> {
        Some(u16::from_be_bytes([
            *daten.get(i)?,
            *daten.get(i + 1)?,
        ]))
    };
    let (Some(anzahl), Some(ablage)) = (lies(2), lies(4)) else {
        return Vec::new();
    };
    let ablage = ablage as usize;
    (0..anzahl as usize)
        .filter_map(|n| {
            let p = 6 + 12 * n;
            let laenge = lies(p + 8)? as usize;
            let versatz = lies(p + 10)? as usize;
            Some(Namenseintrag {
                platform: lies(p)?,
                encoding: lies(p + 2)?,
                sprache: lies(p + 4)?,
                id: lies(p + 6)?,
                bytes: daten.get(ablage + versatz..ablage + versatz + laenge)?,
            })
        })
        .collect()
}

/// `tt_face_get_name` aus FreeTypes `sfobjs.c` (Z. 131-265), uebertragen.
///
/// Die Rangfolge ist nicht offensichtlich und war nicht zu erraten: ein
/// Windows-Eintrag gewinnt, **ausser** es gibt einen Apple-Eintrag und der
/// Windows-Eintrag ist nicht englisch. Unter den Windows-Eintraegen setzt
/// sich der erste englische durch, sonst der erste ueberhaupt.
///
/// `None` steht fuer "diese ID kommt gar nicht vor" - dann geht die Kette in
/// `Font::namen_holen` eine ID weiter. Ein gefundener, aber leerer Eintrag
/// gibt `Some("")` und beendet die Kette, genau wie FreeTypes Pruefung auf
/// den NULL-Zeiger.
fn name_waehlen(eintraege: &[Namenseintrag<'_>], id: u16) -> Option<String> {
    /// `TT_PLATFORM_APPLE_UNICODE`, `_MACINTOSH`, `_ISO`, `_MICROSOFT`.
    const APPLE_UNICODE: u16 = 0;
    const MACINTOSH: u16 = 1;
    const ISO: u16 = 2;
    const MICROSOFT: u16 = 3;

    let mut apple_roman = None;
    let mut apple_english = None;
    let mut win = None;
    let mut unicode = None;
    let mut is_english = false;

    for (n, rec) in eintraege.iter().enumerate() {
        if rec.id != id || rec.bytes.is_empty() {
            continue;
        }
        match rec.platform {
            // "We should use this field only as a last solution when nothing
            // else is available."
            APPLE_UNICODE | ISO => unicode = Some(n),
            MACINTOSH => {
                // `TT_MAC_LANGID_ENGLISH` ist 0, `TT_MAC_ID_ROMAN` ebenso.
                if rec.sprache == 0 {
                    apple_english = Some(n);
                } else if rec.encoding == 0 {
                    apple_roman = Some(n);
                }
            }
            // "we only take a non-English name when there is nothing else
            // available in the font" - und nur aus einer der drei
            // Unicode-Kodierungen (TT_MS_ID_SYMBOL_CS, _UNICODE_CS, _UCS_4).
            // In der Vorlage sind das zwei geschachtelte Pruefungen; hier
            // stehen sie in einer Zeile, weil clippy sonst widerspricht.
            MICROSOFT
                if (win.is_none() || rec.sprache & 0x3FF == 0x009)
                    && matches!(rec.encoding, 0 | 1 | 10) =>
            {
                is_english = rec.sprache & 0x3FF == 0x009;
                win = Some(n);
            }
            _ => {}
        }
    }

    let apple = apple_english.or(apple_roman);

    // "some fonts contain invalid Unicode or Macintosh formatted entries; we
    // will thus favor names encoded in Windows formats if available
    // (provided it is an English name)"
    if let Some(n) = win.filter(|_| !(apple.is_some() && !is_english)) {
        Some(ascii_aus_utf16(eintraege[n].bytes))
    } else if let Some(n) = apple {
        Some(ascii_aus_anderem(eintraege[n].bytes))
    } else {
        unicode.map(|n| ascii_aus_utf16(eintraege[n].bytes))
    }
}

/// `tt_name_ascii_from_utf16`: UTF-16BE zu ASCII.
///
/// FreeType wirft hier alles weg, was nicht zwischen 32 und 127 liegt -
/// jedes solche Zeichen wird ein `?`, und eine eingebettete Null bricht ab.
///
/// Diese Stelle ist **uebertragen, nicht gemessen**: unter den 327 Schriften
/// in `C:\Windows\Fonts` hat keine einzige ein Zeichen ausserhalb von ASCII
/// im englischen Namenseintrag, der Fall kam in der Gegenprobe also gar
/// nicht vor. Dass er trotzdem so behandelt wird, steht in
/// `tt_name_ascii_from_utf16` (`sfobjs.c`, Z. 55-88).
fn ascii_aus_utf16(b: &[u8]) -> String {
    b.as_chunks::<2>().0.iter()
        .map(|p| u16::from_be_bytes([p[0], p[1]]))
        .take_while(|&c| c != 0)
        .map(|c| if (32..=127).contains(&c) { c as u8 as char } else { '?' })
        .collect()
}

/// `tt_name_ascii_from_other`: ein Byte je Zeichen, sonst wie oben.
fn ascii_aus_anderem(b: &[u8]) -> String {
    b.iter()
        .copied()
        .take_while(|&c| c != 0)
        .map(|c| if (32..=127).contains(&c) { c as char } else { '?' })
        .collect()
}

/// `DIVIDE_BY_255_SIGNED` aus `SDL_ttf.c` - `sign` ist 1 oder 255.
#[inline]
fn div255(x: i32, sign: i32) -> i32 {
    (x + sign + (x >> 8)) >> 8
}

/// `Create_Surface_Shaded`: 256 Stufen von Hintergrund zu Vordergrund.
fn palette_shaded(fg: Color, bg: Color) -> Vec<u32> {
    let rd = i32::from(fg.r) - i32::from(bg.r);
    let gd = i32::from(fg.g) - i32::from(bg.g);
    let bd = i32::from(fg.b) - i32::from(bg.b);
    let sr = if rd >= 0 { 1 } else { 255 };
    let sg = if gd >= 0 { 1 } else { 255 };
    let sb = if bd >= 0 { 1 } else { 255 };
    (0..256)
        .map(|i| {
            let r = (i32::from(bg.r) + div255(i * rd, sr)) as u8;
            let g = (i32::from(bg.g) + div255(i * gd, sg)) as u8;
            let b = (i32::from(bg.b) + div255(i * bd, sb)) as u8;
            0xFF00_0000 | (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
        })
        .collect()
}

/// `Draw_Line`: voller Strich fuer Unterstreichung und Durchstreichung.
///
/// Der Strich **ueberschreibt**, er verodert nicht - anders als die
/// Glyphenbilder. Reicht er ueber den unteren Rand hinaus, wird er duenner
/// gemacht statt abgeschnitten.
fn strich(
    deckung: &mut [u8],
    w: usize,
    h: usize,
    spalte: i32,
    zeile: i32,
    breite: i32,
    dicke: i32,
) {
    let mut dicke = dicke;
    let ueber = zeile + dicke - h as i32;
    if ueber > 0 {
        dicke -= ueber;
    }
    if dicke <= 0 || zeile < 0 {
        return;
    }
    let breite = breite.min(w as i32).max(0) as usize;
    for i in 0..dicke {
        let y = zeile + i;
        if y < 0 || y as usize >= h {
            continue;
        }
        let start = y as usize * w + spalte.max(0) as usize;
        for x in 0..breite {
            if spalte.max(0) as usize + x >= w {
                break;
            }
            deckung[start + x] = 255;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustgame_core::color::Color;

    /// Dieselbe Schrift, die `pygame.font.Font(None, 24)` oeffnet: die
    /// Standardschrift in Punktgroesse 16 (24 mal 0.6875).
    ///
    /// Fehlt `freesansbold.ttf`, werden die Pruefungen uebersprungen statt zu
    /// scheitern - so halten es die Pruefungen in `tests/gegen_pygame.rs`
    /// auch.
    fn schrift() -> Option<Font> {
        Font::new(crate::default_font_path()?, 16).ok()
    }

    /// Die Spalten, in denen eine Bildzeile Deckung hat: `(erste, letzte)`.
    fn tinte(s: &Surface, y: i32) -> Option<(i32, i32)> {
        let mut erste = None;
        let mut letzte = 0;
        for x in 0..s.get_width() as i32 {
            if s.get_at((x, y)).a > 0 {
                erste.get_or_insert(x);
                letzte = x;
            }
        }
        erste.map(|e| (e, letzte))
    }

    /// Sollwerte aus pygame-ce 2.5.7:
    ///
    /// ```text
    /// f = pygame.font.Font(None, 24)
    /// f.name, f.style_name  ->  ('FreeSans', 'Bold')
    /// ```
    ///
    /// Der Name kommt aus name-ID 16, nicht aus ID 1 - bei freesansbold
    /// steht in beiden dasselbe, bei `ARIALN.TTF` nicht (dort ist ID 1
    /// "Arial Narrow", ID 16 "Arial", und pygame meldet "Arial").
    #[test]
    fn name_und_style_name_wie_pygame() {
        let Some(f) = schrift() else {
            eprintln!("freesansbold.ttf nicht gefunden - uebersprungen");
            return;
        };
        assert_eq!(f.name(), "FreeSans");
        assert_eq!(f.style_name(), "Bold");
    }

    /// Gemessen: `Font(None, 24).align` ist `0`, also `FONT_LEFT`.
    #[test]
    fn align_ist_anfangs_links() {
        let Some(mut f) = schrift() else { return };
        assert_eq!(f.get_align(), Ausrichtung::Links);

        // Die Ausrichtung ueberlebt eine neue Punktgroesse - `metriken_neu`
        // fasst sie nicht an.
        f.set_align(Ausrichtung::Rechts);
        f.set_point_size(24);
        assert_eq!(f.get_align(), Ausrichtung::Rechts);
    }

    /// Gemessen: `render("Hallo", True, …).get_size()` ist `(41, 18)` fuer
    /// **alle drei** Ausrichtungen. Bei einer Zeile ist die Flaeche genau so
    /// breit wie die Zeile, also bleibt `xoffset` null.
    #[test]
    fn align_laesst_eine_einzelne_zeile_in_ruhe() {
        let Some(mut f) = schrift() else { return };
        let mut groessen = Vec::new();
        for a in [Ausrichtung::Links, Ausrichtung::Mitte, Ausrichtung::Rechts] {
            f.set_align(a);
            let s = f.render("Hallo", true, Color::rgb(255, 255, 255), None).unwrap();
            groessen.push((s.get_width(), s.get_height()));
        }
        assert_eq!(groessen, vec![(41, 18); 3]);
    }

    /// Der Unterstrich ist der genaueste Pruefstein, den es hier gibt: seine
    /// Lage haengt allein an der Rechnung, nicht an der Rasterung.
    ///
    /// Gemessen mit `Font(None, 24)`, `set_underline(True)` und
    /// `"ii\nWWWWWW"` - Flaeche 90 x 36, und der Strich der ersten Zeile
    /// liegt auf Bildzeile 16:
    ///
    /// ```text
    /// align = 0 (FONT_LEFT)    Spalten  0 bis  7
    /// align = 1 (FONT_CENTER)  Spalten 41 bis 48
    /// align = 2 (FONT_RIGHT)   Spalten 82 bis 89
    /// ```
    ///
    /// Hier sind beide Mittenformeln noch einig (`90/2 - 8/2` und
    /// `(90 - 8)/2` geben 41); die naechste Pruefung trennt sie.
    #[test]
    fn align_verschiebt_den_unterstrich_wie_pygame() {
        let Some(mut f) = schrift() else { return };
        f.set_underline(true);
        let mut lagen = Vec::new();
        for a in [Ausrichtung::Links, Ausrichtung::Mitte, Ausrichtung::Rechts] {
            f.set_align(a);
            let s = f
                .render("ii\nWWWWWW", true, Color::rgb(255, 255, 255), None)
                .unwrap();
            assert_eq!((s.get_width(), s.get_height()), (90, 36));
            lagen.push(tinte(&s, 16));
        }
        assert_eq!(lagen, vec![Some((0, 7)), Some((41, 48)), Some((82, 89))]);
    }

    /// Hier trennen sich die beiden Mittenformeln - und die naheliegende
    /// verliert.
    ///
    /// Gemessen mit `"a\nWWWWWW"`: die Flaeche ist 90 breit, `f.size("a")`
    /// ist 9 breit, und der Unterstrich der ersten Zeile faengt bei
    /// `FONT_CENTER` auf Spalte **41** an:
    ///
    /// ```text
    /// align = 0   Spalten  0 bis  8
    /// align = 1   Spalten 41 bis 49      <- hier zeigt es sich
    /// align = 2   Spalten 81 bis 89
    /// ```
    ///
    /// `90/2 - 9/2` ist `45 - 4 = 41` und trifft. `(90 - 9)/2` waere `40`
    /// und laege daneben. Ganzzahldivision zweimal einzeln ist eben nicht
    /// dasselbe wie einmal auf die Differenz - genau deshalb steht die
    /// Formel oben so, wie sie in `TTF_Render_Wrapped_Internal` steht, und
    /// wurde nicht nachempfunden.
    ///
    /// Zum Vergleich `"a\nbb\nccc"`: Flaeche 27 breit, erste Zeile 9 breit,
    /// beide Formeln geben 9 - der Fehler waere dort unsichtbar geblieben.
    #[test]
    fn align_mitte_rechnet_wie_sdl_ttf() {
        let Some(mut f) = schrift() else { return };
        f.set_underline(true);

        for (text, breite, soll) in [
            ("a\nWWWWWW", 90, [(0, 8), (41, 49), (81, 89)]),
            ("a\nbb\nccc", 27, [(0, 8), (9, 17), (18, 26)]),
        ] {
            let mut lagen = Vec::new();
            for a in [Ausrichtung::Links, Ausrichtung::Mitte, Ausrichtung::Rechts] {
                f.set_align(a);
                let s = f.render(text, true, Color::rgb(255, 255, 255), None).unwrap();
                assert_eq!(s.get_width() as i32, breite, "Flaechenbreite bei {text:?}");
                lagen.push(tinte(&s, 16).unwrap());
            }
            assert_eq!(lagen, soll, "Unterstrich bei {text:?}");
        }
    }

    /// FreeType laesst nur ASCII 32 bis 127 durch, alles andere wird `?`,
    /// und eine eingebettete Null bricht ab (`tt_name_ascii_from_utf16`).
    #[test]
    fn namen_werden_wie_freetype_auf_ascii_gestutzt() {
        // "A" + U+00C4 (Ae) + "B" in UTF-16BE
        let b = [0x00, 0x41, 0x00, 0xC4, 0x00, 0x42];
        assert_eq!(ascii_aus_utf16(&b), "A?B");
        // Null bricht ab, der Rest faellt weg.
        let b = [0x00, 0x41, 0x00, 0x00, 0x00, 0x42];
        assert_eq!(ascii_aus_utf16(&b), "A");
        assert_eq!(ascii_aus_anderem(b"Ab\xC4c"), "Ab?c");
        assert_eq!(ascii_aus_anderem(b"Ab\0c"), "Ab");
    }

    /// `tt_face_get_name`: der Windows-Eintrag gewinnt - aber nicht, wenn er
    /// nicht englisch ist und daneben ein Apple-Eintrag steht.
    #[test]
    fn namensauswahl_folgt_freetypes_rangfolge() {
        let mac = |bytes: &'static [u8]| Namenseintrag {
            platform: 1,
            encoding: 0,
            sprache: 0,
            id: 1,
            bytes,
        };
        let win = |sprache: u16, bytes: &'static [u8]| Namenseintrag {
            platform: 3,
            encoding: 1,
            sprache,
            id: 1,
            bytes,
        };

        // Englischer Windows-Eintrag schlaegt den Apple-Eintrag.
        let e = [mac(b"Apple"), win(0x409, b"\0W\0i\0n")];
        assert_eq!(name_waehlen(&e, 1).as_deref(), Some("Win"));

        // Deutscher Windows-Eintrag verliert gegen den Apple-Eintrag.
        let e = [mac(b"Apple"), win(0x407, b"\0W\0i\0n")];
        assert_eq!(name_waehlen(&e, 1).as_deref(), Some("Apple"));

        // Ohne Apple-Eintrag gewinnt auch der deutsche Windows-Eintrag.
        let e = [win(0x407, b"\0W\0i\0n")];
        assert_eq!(name_waehlen(&e, 1).as_deref(), Some("Win"));

        // Gibt es die ID gar nicht, geht die Kette eine ID weiter.
        assert_eq!(name_waehlen(&e, 16), None);
    }
}

#[cfg(test)]
mod einfarbig {
    use super::*;

    /// `render(text, False, ..)` war bis Runde 19 **leer** - in jeder
    /// Groesse, fuer jeden Text (siehe `acht_bit`). Gemessen bei pygame:
    /// `Font(None, 24).render("Hi", False, weiss)` setzt 92 Pixel.
    #[test]
    fn hi_hat_92_pixel_wie_pygame() {
        let Some(pfad) = crate::default_font_path() else {
            return;
        };
        let f = Font::default_font(std::fs::read(pfad).unwrap(), 24).unwrap();
        let s = f.render("Hi", false, Color::rgb(255, 255, 255), None).unwrap();
        assert_eq!(s.get_size(), (16, 18));
        assert_eq!(s.get_bitsize(), 8);
        let mut n = 0;
        for y in 0..18 {
            for x in 0..16 {
                if s.get_at((x, y)).r == 255 {
                    n += 1;
                }
            }
        }
        assert_eq!(n, 92);
    }
}
