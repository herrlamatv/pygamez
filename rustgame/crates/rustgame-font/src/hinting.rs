//! hinting.rs
//! ==========
//! Gehintete Umrisse fuer [`crate::ftgrays`] - ueber `skrifa`.
//!
//! ## Was SDL_ttf tut
//!
//! `Load_Glyph` ruft `FT_Load_Glyph(face, index, FT_LOAD_DEFAULT |
//! FT_LOAD_TARGET_NORMAL)`. Fuer eine TrueType-Schrift mit `fpgm` oder
//! `prep` heisst das: FreeTypes **Bytecode-Interpreter**, Fassung v40, im
//! Rueckwaertsmodus - waagerechte Anweisungen werden weitgehend
//! uebergangen, senkrechte ausgefuehrt. Gemessen an einem `H` in
//! Bahnschrift 22: bei pygame ist die Oberkante eine volle Zeile 255 und
//! der Querbalken liegt auf genau zwei vollen Zeilen; ungehintet
//! (fontdue) verteilt sich beides ueber halbe Zeilen. Alle vier Schriften,
//! die PyGameZ benutzt (Bahnschrift, Segoe UI, Consolas, Segoe UI Symbol),
//! tragen solchen Bytecode.
//!
//! ## Warum `skrifa`
//!
//! FreeTypes Interpreter sind 8.635 Zeilen C (`ttinterp.c`) plus 3.033
//! Zeilen Glyphenlader. `skrifa` (Google fontations, reines Rust,
//! `#![forbid(unsafe_code)]`) hat denselben Interpreter und gleicht ihn
//! gegen FreeTypes v40-Modus ab. Nachgemessen mit pygames **eigener**
//! `freetype.dll` (2.11.1) ueber `ctypes`: fuer Consolas fett `g` bei 14
//! sind die Segmente beider Umrisse gleich, Punkt fuer Punkt.
//!
//! ## Die Punktliste, nicht nur der Stift
//!
//! `skrifa` gibt einen gehinteten Umriss nur ueber einen Zeichenstift
//! heraus. FreeType rastert aber eine **Punktliste** (`FT_Outline`), und
//! daran haengt mehr, als man denkt: `ft_smooth_render` **verschiebt** den
//! Umriss vor dem Zerlegen, und erst danach bildet `FT_Outline_Decompose`
//! die gedachten Mittelpunkte zweier Kontrollpunkte - als Ganzzahldivision,
//! die gegen null abschneidet. Bei ungerader Summe ist `(a + b) / 2` vor
//! und nach einer Verschiebung nicht dasselbe, sobald das Vorzeichen
//! wechselt (-3/2 = -1, aber (-3 + 20)/2 = 8 und nicht -1 + 10 = 9).
//!
//! Die erste Fassung hat die Punktliste aus dem Stift-Strom **erraten**:
//! ein `quad_to`, dessen Endpunkt genau der Mittelpunkt der Nachbarn war,
//! galt als gedacht. In Consolas fett `g` liegt aber ein **echter** Punkt
//! genau dort, in der Unterlaenge bei negativem y - weggelassen und nach
//! der Verschiebung neu gebildet lag er 1/64 daneben, und ein Pixel wurde
//! 174 statt 173. Drei von 11.984 Zeichenbildern.
//!
//! Jetzt wird nicht mehr geraten: welche Punkte echt sind und welche Art
//! sie haben, aendert das Hinting nie - es verschiebt nur. Also kommen
//! Punktzahl, Arten und Konturenden aus dem **ungehinteten** Umriss
//! (`with_scaled_glyf_outline`), und die gehinteten Koordinaten werden im
//! Gleichschritt mit `read-fonts`' Zerlegung (`walk_contour`,
//! `PathContourStart::ScanBackward`) den Rohpunkten zugeordnet. Passt der
//! Strom nicht zu den Arten, gibt es `None` und der alte Weg greift.
//!
//! ## CFF (`.otf` mit PostScript-Umrissen)
//!
//! Bis Runde 19 kannte dieses Modul nur `glyf`; eine CFF-Schrift ging
//! ungehintet ueber fontdue, und gemessen war in `pygame.font` an
//! `PyGameMono.otf` **kein einziges** von 165 Bildern gleich. FreeType
//! hintet CFF bei `FT_LOAD_TARGET_NORMAL` mit dem Adobe-Hinter (`cf2`,
//! `psaux/psft.c`), und `skrifa` hat genau diesen Hinter uebertragen. Die
//! Punktliste baut [`cff_umriss`] wie FreeTypes `ps_builder` - dort gibt es
//! keine Rohpunkte, an die man sich halten koennte, der Strom **ist** die
//! Punktliste.

use std::cell::RefCell;
use std::collections::HashMap;

use skrifa::instance::{LocationRef, Size};
use skrifa::outline::{
    DrawSettings, HintingInstance, HintingOptions, OutlineGlyphFormat, OutlinePen, RohArt,
};
use skrifa::{FontRef, GlyphId, MetadataProvider};

use crate::ftgrays::{Art, Umriss};
use crate::ftraster::{self, Rasterflaggen};

/// Ein Ereignis des Zeichenstifts - in 26.6.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Strich {
    Anfang((i64, i64)),
    Strecke((i64, i64)),
    Bogen((i64, i64), (i64, i64)),
    Kubisch((i64, i64), (i64, i64), (i64, i64)),
    Schluss,
}

/// Schreibt den Stift-Strom mit.
#[derive(Default)]
struct Mitschrift(Vec<Strich>);

/// Ein Stiftwert in 26.6. `skrifa` gibt `bits / 64` als `f32` heraus -
/// das ist fuer jeden Wert unter 2^24 exakt, also auch das Zurueckrechnen.
///
/// **Abgerundet, nicht abgeschnitten.** Bei `glyf` sind die Werte schon
/// 26.6 und beides gleich. CFF rechnet in 16.16, und FreeType macht daraus
/// `x >> 10` (`ps_builder_add_point`) - fuer negative Werte ist das die
/// Abrundung; `as i64` schnitte zur Null hin ab.
fn f26(v: f32) -> i64 {
    (f64::from(v) * 64.0).floor() as i64
}

impl OutlinePen for Mitschrift {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.push(Strich::Anfang((f26(x), f26(y))));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.0.push(Strich::Strecke((f26(x), f26(y))));
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.0
            .push(Strich::Bogen((f26(cx0), f26(cy0)), (f26(x), f26(y))));
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.0.push(Strich::Kubisch(
            (f26(cx0), f26(cy0)),
            (f26(cx1), f26(cy1)),
            (f26(x), f26(y)),
        ));
    }

    fn close(&mut self) {
        self.0.push(Strich::Schluss);
    }
}

/// Was im Gleichschritt noch offen ist - `PendingState` aus `read-fonts`,
/// mit den Indizes der Rohpunkte statt ihrer Werte.
#[derive(Clone, Copy)]
enum Offen {
    Nichts,
    Quad(usize),
    Kubisch(usize),
    ZweiKubisch(usize, usize),
}

/// Liest den Stift-Strom und ordnet ihn den Rohpunkten zu.
struct Zuordnung<'a> {
    strom: &'a [Strich],
    pos: usize,
    punkte: &'a mut [(i64, i64)],
    ab: usize,
}

impl Zuordnung<'_> {
    fn naechster(&mut self) -> Option<Strich> {
        let s = self.strom.get(self.pos).copied();
        self.pos += 1;
        s
    }

    fn setze(&mut self, i: usize, p: (i64, i64)) {
        self.punkte[self.ab + i] = p;
    }

    /// Ein Rohpunkt, wie `PendingState::emit` ihn verbraucht.
    fn schritt(&mut self, offen: &mut Offen, ix: usize, art: Art) -> bool {
        match (*offen, art) {
            (Offen::Nichts, Art::Quadratisch) => *offen = Offen::Quad(ix),
            (Offen::Nichts, Art::Kubisch) => *offen = Offen::Kubisch(ix),
            (Offen::Nichts, Art::Auf) => match self.naechster() {
                Some(Strich::Strecke(p)) => self.setze(ix, p),
                _ => return false,
            },
            (Offen::Quad(q), Art::Quadratisch) => match self.naechster() {
                // Zwei Kontrollpunkte in Folge: der Endpunkt ist gedacht.
                Some(Strich::Bogen(c, _)) => {
                    self.setze(q, c);
                    *offen = Offen::Quad(ix);
                }
                _ => return false,
            },
            (Offen::Quad(q), Art::Auf) => match self.naechster() {
                Some(Strich::Bogen(c, p)) => {
                    self.setze(q, c);
                    self.setze(ix, p);
                    *offen = Offen::Nichts;
                }
                _ => return false,
            },
            (Offen::Quad(_), Art::Kubisch) => return false,
            (Offen::Kubisch(c), Art::Kubisch) => *offen = Offen::ZweiKubisch(c, ix),
            (Offen::Kubisch(_), _) => return false,
            (Offen::ZweiKubisch(_, _), Art::Quadratisch) => return false,
            (Offen::ZweiKubisch(c0, c1), Art::Kubisch) => match self.naechster() {
                Some(Strich::Kubisch(a, b, _)) => {
                    self.setze(c0, a);
                    self.setze(c1, b);
                    *offen = Offen::Kubisch(ix);
                }
                _ => return false,
            },
            (Offen::ZweiKubisch(c0, c1), Art::Auf) => match self.naechster() {
                Some(Strich::Kubisch(a, b, p)) => {
                    self.setze(c0, a);
                    self.setze(c1, b);
                    self.setze(ix, p);
                    *offen = Offen::Nichts;
                }
                _ => return false,
            },
        }
        true
    }

    /// Eine Kontur - `walk_contour` mit `ScanBackward`. `arten` sind die
    /// Arten ihrer Rohpunkte, `self.ab` ihr erster Index im Umriss.
    fn kontur(&mut self, arten: &[Art]) -> bool {
        let n = arten.len();
        if n == 0 {
            return true;
        }
        let (anfang_index, basis, anzahl) = match arten[0] {
            Art::Kubisch => return false,
            Art::Auf => (Some(0), 1, n - 1),
            Art::Quadratisch if arten[n - 1] == Art::Auf => (Some(n - 1), 0, n - 1),
            // Beide Enden Kontrollpunkte: der Anfang ist ihr gedachter
            // Mittelpunkt, und alle Punkte werden durchlaufen.
            Art::Quadratisch => (None, 0, n),
        };
        let Some(Strich::Anfang(s)) = self.naechster() else {
            return false;
        };
        if let Some(i) = anfang_index {
            self.setze(i, s);
        }
        let mut offen = Offen::Nichts;
        for k in 0..anzahl {
            let ix = (basis + k) % n;
            if !self.schritt(&mut offen, ix, arten[ix]) {
                return false;
            }
        }
        // `finish`: was offen ist, endet am Anfang.
        match offen {
            Offen::Nichts => {}
            Offen::Quad(q) => match self.naechster() {
                Some(Strich::Bogen(c, _)) => self.setze(q, c),
                _ => return false,
            },
            Offen::ZweiKubisch(c0, c1) => match self.naechster() {
                Some(Strich::Kubisch(a, b, _)) => {
                    self.setze(c0, a);
                    self.setze(c1, b);
                }
                _ => return false,
            },
            Offen::Kubisch(_) => return false,
        }
        matches!(self.naechster(), Some(Strich::Schluss))
    }
}

/// Ordnet einen ganzen Stift-Strom den Rohpunkten zu.
///
/// `None`, sobald Strom und Arten nicht zusammenpassen oder Striche
/// uebrig bleiben.
fn zuordnen(arten: &[Art], enden: &[usize], strom: &[Strich]) -> Option<Vec<(i64, i64)>> {
    let mut punkte = vec![(0i64, 0i64); arten.len()];
    let mut z = Zuordnung {
        strom,
        pos: 0,
        punkte: &mut punkte,
        ab: 0,
    };
    for &ende in enden {
        if ende < z.ab || ende >= arten.len() {
            return None;
        }
        let ab = z.ab;
        if !z.kontur(&arten[ab..=ende]) {
            return None;
        }
        z.ab = ende + 1;
    }
    (z.pos == strom.len()).then_some(punkte)
}

/// FreeTypes Punktliste einer CFF-Glyphe aus dem Stift-Strom.
///
/// Uebertragen aus FreeType 2.11.1 `psaux/psobjs.c` (`ps_builder_start_point`,
/// `ps_builder_add_point`, `ps_builder_close_contour`) in der Reihenfolge,
/// in der `psft.c` sie ruft:
///
/// * `moveTo` legt **keinen** Punkt an, es schliesst nur die laufende
///   Kontur. Der Anfangspunkt kommt mit dem ersten Strich danach.
/// * Beim Schliessen faellt der letzte Punkt weg, wenn er auf dem ersten
///   liegt und kein Kontrollpunkt ist - und eine Kontur aus einem einzigen
///   Punkt faellt ganz weg.
/// * Ein `close` des Stifts hat bei FreeType kein Gegenstueck; geschlossen
///   wird beim naechsten `moveTo` und am Ende.
fn cff_umriss(strom: &[Strich]) -> Option<Umriss> {
    let mut punkte: Vec<(i64, i64)> = Vec::new();
    let mut arten: Vec<Art> = Vec::new();
    let mut enden: Vec<usize> = Vec::new();
    let mut begonnen = false;
    let mut stift = (0i64, 0i64);

    fn schliessen(punkte: &mut Vec<(i64, i64)>, arten: &mut Vec<Art>, enden: &mut Vec<usize>) {
        let n_c = enden.len();
        let first = if n_c <= 1 { 0 } else { enden[n_c - 2] + 1 };
        if n_c > 0 && first == punkte.len() {
            enden.pop();
            return;
        }
        if punkte.len() > 1 {
            let n = punkte.len();
            if punkte[first] == punkte[n - 1] && arten[n - 1] == Art::Auf {
                punkte.pop();
                arten.pop();
            }
        }
        if n_c > 0 {
            if punkte.len().checked_sub(1) == Some(first) {
                enden.pop();
                punkte.pop();
                arten.pop();
            } else if let Some(e) = enden.last_mut() {
                *e = punkte.len().saturating_sub(1);
            }
        }
    }

    for s in strom {
        match *s {
            Strich::Anfang(p) => {
                schliessen(&mut punkte, &mut arten, &mut enden);
                begonnen = false;
                stift = p;
            }
            Strich::Strecke(p) | Strich::Kubisch(_, _, p) => {
                if !begonnen {
                    // `ps_builder_start_point` -> `ps_builder_add_contour`
                    begonnen = true;
                    if let Some(e) = enden.last_mut() {
                        *e = punkte.len().saturating_sub(1);
                    }
                    enden.push(punkte.len());
                    punkte.push(stift);
                    arten.push(Art::Auf);
                }
                if let Strich::Kubisch(a, b, _) = *s {
                    punkte.push(a);
                    arten.push(Art::Kubisch);
                    punkte.push(b);
                    arten.push(Art::Kubisch);
                }
                punkte.push(p);
                arten.push(Art::Auf);
                stift = p;
            }
            // CFF kennt keine quadratischen Boegen.
            Strich::Bogen(..) => return None,
            Strich::Schluss => {}
        }
    }
    schliessen(&mut punkte, &mut arten, &mut enden);
    Some(Umriss {
        punkte,
        arten,
        enden,
        ueberlappt: false,
    })
}

/// CFF mit zwei Achsen: `x` aus dem Strom in der Breite, `y` aus dem in der
/// Hoehe (siehe [`Hinter::umriss_mit_vorschub`]). `None`, wenn die beiden
/// Stroeme nicht Schritt fuer Schritt zusammenpassen.
fn zusammensetzen(breite: &[Strich], hoehe: &[Strich]) -> Option<Vec<Strich>> {
    if breite.len() != hoehe.len() {
        return None;
    }
    let p = |x: (i64, i64), y: (i64, i64)| (x.0, y.1);
    breite
        .iter()
        .zip(hoehe)
        .map(|(a, b)| {
            Some(match (*a, *b) {
                (Strich::Anfang(x), Strich::Anfang(y)) => Strich::Anfang(p(x, y)),
                (Strich::Strecke(x), Strich::Strecke(y)) => Strich::Strecke(p(x, y)),
                (Strich::Bogen(x0, x1), Strich::Bogen(y0, y1)) => {
                    Strich::Bogen(p(x0, y0), p(x1, y1))
                }
                (Strich::Kubisch(x0, x1, x2), Strich::Kubisch(y0, y1, y2)) => {
                    Strich::Kubisch(p(x0, y0), p(x1, y1), p(x2, y2))
                }
                (Strich::Schluss, Strich::Schluss) => Strich::Schluss,
                _ => return None,
            })
        })
        .collect()
}

/// Die Hinting-Instanzen einer Schriftdatei, je Nenngroesse (26.6, Breite
/// und Hoehe) eine.
///
/// Eine Instanz anzulegen fuehrt `fpgm` und `prep` aus - das geschieht
/// einmal je Groesse, nicht je Zeichen. `None` heisst: diese Groesse ist
/// gescheitert (ein Fehler im Bytecode), dann bleibt es beim alten Weg.
#[derive(Default)]
pub(crate) struct Hinter {
    instanzen: RefCell<HashMap<(u32, u32), Option<HintingInstance>>>,
    /// Je Groesse: laeuft der v40-Interpreter im Rueckwaertsmodus (oder
    /// gibt es gar keinen)? Dann aendert das Hinting keinen Vorschub, und
    /// [`Hinter::gehinteter_vorschub`] kehrt sofort zurueck - das ist der
    /// uebliche Fall, und er wird fuer jede Glyphe gefragt.
    ohne_gehinteten_vorschub: RefCell<HashMap<(u32, u32), bool>>,
}

impl std::fmt::Debug for Hinter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Hinter")
            .field("groessen", &self.instanzen.borrow().len())
            .finish()
    }
}

impl Hinter {
    /// Der gehintete Umriss von `gid` bei `ppem` Pixeln je Geviert - als
    /// FreeTypes Punktliste.
    ///
    /// `None`, wenn die Datei keine TrueType-Umrisse hat, die Instanz oder
    /// das Zeichnen scheitert - dann rastert der Aufrufer wie bisher.
    pub(crate) fn umriss(&self, bytes: &[u8], index: u32, ppem: u32, gid: u16) -> Option<Umriss> {
        self.umriss_mit_vorschub(bytes, index, (ppem << 6, ppem << 6), gid)
            .map(|(u, _)| u)
    }

    /// Wie [`Hinter::umriss`], dazu FreeTypes `horiAdvance` in 26.6, so
    /// wie `compute_glyph_metrics` ihn setzt - also **vor**
    /// `ft_glyphslot_grid_fit_metrics`: die Phantompunkte nach dem Hinting
    /// (im Rueckwaertsmodus von v40 die unveraenderten) oder `hdmx`.
    /// `pygame.freetype`s senkrechter Satz braucht genau diesen Wert:
    /// `vertBearingX = horiBearingX - horiAdvance / 2`.
    ///
    /// `groesse26` ist die **Nenngroesse in 26.6**, Breite und Hoehe -
    /// `pygame.freetype` kennt Bruchgroessen (`size=20.5`) und zwei Achsen
    /// (`size=(24, 12)`). Ob FreeType dafuer auf ganze Pixel rundet
    /// (`head.flags` Bit 3) oder gebrochen hintet, entscheidet skrifa
    /// genauso (`compute_hinted_scale`).
    ///
    /// **Zwei Achsen** hintet FreeType mit dem gestreckten Interpreter;
    /// der steckt im skrifa-Abzug (`HintingInstance::new_gestreckt`), auch
    /// fuer ein von `prep` abgeschaltetes Hinting. CFF setzt sich aus zwei
    /// Umrissen zusammen (siehe unten). Nur der Autohinter zeichnet bei
    /// skrifa mit einer Skala - dort gibt es `None` und den alten Weg.
    pub(crate) fn umriss_mit_vorschub(
        &self,
        bytes: &[u8],
        index: u32,
        groesse26: (u32, u32),
        gid: u16,
    ) -> Option<(Umriss, Option<i64>)> {
        // Ein 26.6-Wert ist als `f32` genau, und skrifa rechnet daraus
        // wieder `(ppem * 64) as i32` - also `scaled_w` wie FreeType.
        let groesse = Size::new(groesse26.0 as f32 / 64.0);
        let zwei = groesse26.0 != groesse26.1;
        let font = FontRef::from_index(bytes, index).ok()?;
        let glyphen = font.outline_glyphs();
        let glyph = glyphen.get(GlyphId::new(gid as u32))?;
        let cff = matches!(
            glyph.format(),
            OutlineGlyphFormat::Cff | OutlineGlyphFormat::Cff2
        );

        // **CFF mit zwei Achsen.** FreeTypes CFF-Maschine nimmt
        // `(scaleX, 0, 0, scaleY)` als inneren Satz (`cf2_font_setup`) und
        // hintet nur senkrecht, mit `y_ppem` (`cf2_getPpemY`): in
        // `cf2_glyphpath_hintPoint` ist `x = FT_MulFix(scaleX, x)` und
        // `y = cf2_hintmap_map(hintmap, y)`. Ein Punkt hat also das `x` des
        // Umrisses in der Breite und das `y` des Umrisses in der Hoehe.
        if zwei && cff {
            let mut stroeme = Vec::with_capacity(2);
            for g in [groesse26.0, groesse26.1] {
                let mut tabelle = self.instanzen.borrow_mut();
                let instanz = tabelle.entry((g, g)).or_insert_with(|| {
                    HintingInstance::new(
                        &glyphen,
                        Size::new(g as f32 / 64.0),
                        LocationRef::default(),
                        HintingOptions::default(),
                    )
                    .ok()
                });
                let instanz = instanz.as_ref()?;
                let mut strom = Mitschrift::default();
                glyph
                    .draw(DrawSettings::hinted(instanz, false), &mut strom)
                    .ok()?;
                stroeme.push(strom.0);
            }
            let strom = zusammensetzen(&stroeme[0], &stroeme[1])?;
            return Some((cff_umriss(&strom)?, None));
        }

        let mut tabelle = self.instanzen.borrow_mut();
        let instanz = tabelle.entry(groesse26).or_insert_with(|| {
            if zwei {
                HintingInstance::new_gestreckt(
                    &glyphen,
                    groesse,
                    Size::new(groesse26.1 as f32 / 64.0),
                    LocationRef::default(),
                    HintingOptions::default(),
                )
                .ok()
            } else {
                HintingInstance::new(
                    &glyphen,
                    groesse,
                    LocationRef::default(),
                    HintingOptions::default(),
                )
                .ok()
            }
        });
        let instanz = instanz.as_ref()?;
        // `scan_zustand` gibt es nur fuer TrueType-Bytecode.
        if zwei && (cff || instanz.scan_zustand().is_none()) {
            return None;
        }
        // **TrueType-Bytecode: FreeTypes Punktliste direkt** (`roh_umriss`
        // im skrifa-Abzug) - Punkte, Konturenden, der ungerundete Vorschub
        // (`horiAdvance` vor `ft_glyphslot_grid_fit_metrics`) und die Art
        // jedes Punktes **nach** dem Hinting. Bis Runde 20 kamen die Arten
        // aus dem ungehinteten Umriss, und der Stift-Strom wurde ihnen
        // zugeordnet; aber `FLIPPT`/`FLIPRGON` machen beim Hinting aus
        // Kontrollpunkten Kurvenpunkte. Courier New fett tut das bei
        // kleinen Groessen an jeder Serife - dort ging die Zuordnung schief,
        // und fast jede Glyphe fiel still auf den alten Weg.
        if let Some(r) = instanz.roh_umriss(&glyph) {
            return Some((
                Umriss {
                    punkte: r
                        .punkte
                        .iter()
                        .map(|&(x, y)| (i64::from(x), i64::from(y)))
                        .collect(),
                    arten: r
                        .arten
                        .iter()
                        .map(|a| match a {
                            RohArt::Auf => Art::Auf,
                            RohArt::Quadratisch => Art::Quadratisch,
                            RohArt::Kubisch => Art::Kubisch,
                        })
                        .collect(),
                    enden: r.enden.iter().map(|&e| usize::from(e)).collect(),
                    ueberlappt: glyph.has_overlaps().unwrap_or(false),
                },
                Some(i64::from(r.vorschub)),
            ));
        }
        let mut strom = Mitschrift::default();
        glyph
            .draw(DrawSettings::hinted(instanz, false), &mut strom)
            .ok()?;
        if cff {
            // CFF: der ungerundete Vorschub ist hier nicht nachgebaut -
            // `None` laesst `pygame.freetype`s senkrechten Satz beim alten
            // Weg.
            return Some((cff_umriss(&strom.0)?, None));
        }
        // Der Autohinter verschiebt nur Punkte: Arten und Konturenden aus
        // dem ungehinteten Umriss, der Stift-Strom wird ihnen zugeordnet.
        let (arten, enden) = glyph
            .with_scaled_glyf_outline(groesse, LocationRef::default(), None, |o| {
                let arten: Vec<Art> = o
                    .flags
                    .iter()
                    .map(|f| {
                        if f.is_on_curve() {
                            Art::Auf
                        } else if f.is_off_curve_cubic() {
                            Art::Kubisch
                        } else {
                            Art::Quadratisch
                        }
                    })
                    .collect();
                let enden: Vec<usize> = o.contours.iter().map(|e| *e as usize).collect();
                Ok((arten, enden))
            })
            .ok()?;
        let punkte = zuordnen(&arten, &enden, &strom.0)?;
        Some((
            Umriss {
                punkte,
                arten,
                enden,
                ueberlappt: glyph.has_overlaps().unwrap_or(false),
            },
            None,
        ))
    }

    /// Der **gehintete** Vorschub in 26.6, auf ganze Pixel gerundet
    /// (`ft_glyphslot_grid_fit_metrics`) - aber nur, wenn das Hinting ihn
    /// ueberhaupt aendert: `prep` hat den Rueckwaertsmodus des
    /// v40-Interpreters abgeschaltet (`INSTCTRL` Bit 3). Dann nimmt FreeType
    /// die gehinteten Phantompunkte oder `hdmx`. Sonst `None`, und der
    /// gerundete lineare Vorschub gilt. Gemessen an `symbol.ttf` bei 13:
    /// das `u` (U+F075) hat bei pygame den Vorschub 8, linear waeren es 7.
    pub(crate) fn gehinteter_vorschub(
        &self,
        bytes: &[u8],
        index: u32,
        groesse26: (u32, u32),
        gid: u16,
    ) -> Option<i64> {
        if self.ohne_gehinteten_vorschub.borrow().get(&groesse26) == Some(&true) {
            return None;
        }
        let font = FontRef::from_index(bytes, index).ok()?;
        let glyphen = font.outline_glyphs();
        if !matches!(glyphen.format(), Some(OutlineGlyphFormat::Glyf)) {
            self.ohne_gehinteten_vorschub.borrow_mut().insert(groesse26, true);
            return None;
        }
        let glyph = glyphen.get(GlyphId::new(gid as u32))?;
        let mut tabelle = self.instanzen.borrow_mut();
        let instanz = tabelle.entry(groesse26).or_insert_with(|| {
            HintingInstance::new(
                &glyphen,
                Size::new(groesse26.0 as f32 / 64.0),
                LocationRef::default(),
                HintingOptions::default(),
            )
            .ok()
        });
        let ohne = instanz
            .as_ref()
            .is_none_or(|i| i.rueckwaertsmodus() != Some(false));
        self.ohne_gehinteten_vorschub.borrow_mut().insert(groesse26, ohne);
        if ohne {
            return None;
        }
        let r = instanz.as_ref()?.roh_umriss(&glyph)?;
        Some((i64::from(r.vorschub) + 32) & -64)
    }

    /// Ob das CV-Programm (`prep`) dieser Groesse scheitert - dann scheitert
    /// bei FreeType **jede** gehintete Glyphe: `tt_size_run_prep` legt den
    /// Fehler in `size->cvt_ready`, und `tt_loader_init` gibt ihn bei jedem
    /// `FT_Load_Glyph` zurueck (gemessen an `monbaiti.ttf`: Fehler 134,
    /// `Invalid_Reference`, in jeder Groesse; skrifa scheitert dort an
    /// derselben Stelle - `CALL` einer nie definierten Funktion).
    pub(crate) fn gescheitert(&self, bytes: &[u8], index: u32, groesse26: (u32, u32)) -> bool {
        let Ok(font) = FontRef::from_index(bytes, index) else {
            return false;
        };
        let glyphen = font.outline_glyphs();
        if !matches!(glyphen.format(), Some(OutlineGlyphFormat::Glyf)) {
            return false;
        }
        let mut tabelle = self.instanzen.borrow_mut();
        tabelle
            .entry(groesse26)
            .or_insert_with(|| {
                let groesse = Size::new(groesse26.0 as f32 / 64.0);
                if groesse26.0 != groesse26.1 {
                    HintingInstance::new_gestreckt(
                        &glyphen,
                        groesse,
                        Size::new(groesse26.1 as f32 / 64.0),
                        LocationRef::default(),
                        HintingOptions::default(),
                    )
                    .ok()
                } else {
                    HintingInstance::new(
                        &glyphen,
                        groesse,
                        LocationRef::default(),
                        HintingOptions::default(),
                    )
                    .ok()
                }
            })
            .is_none()
    }

    /// Was FreeTypes TrueType-Lader fuer den **Schwarzweiss-Rasterer** an
    /// den Umriss haengt: `outline.flags` und die Scan-Marken.
    ///
    /// Nur sinnvoll nach [`Hinter::umriss`] fuer dieselbe Groesse - die
    /// Instanz muss schon stehen. Nachgebaut aus `ttgload.c` 2.11.1:
    ///
    /// * `FT_OUTLINE_HIGH_PRECISION` unter 24 Pixeln je Geviert.
    /// * Die Dropout-Flaggen aus `GS.scan_control`/`GS.scan_type`, sonst
    ///   `FT_OUTLINE_IGNORE_DROPOUTS` - **auch wenn `prep` das Hinting
    ///   abgeschaltet hat**: `TT_Load_Glyph` fragt dafuer sein eigenes
    ///   `load_flags`, und das `FT_LOAD_NO_HINTING` aus `tt_loader_init`
    ///   steht nur in der Kopie des Laders (gemessen an `cour.ttf` und
    ///   `georgia.ttf` bis 8 Pixel: `0x130`, ohne Marke).
    /// * `TT_Hint_Glyph` schreibt `scan_type << 5` in die Marke des ersten
    ///   Punktes **jeder einfachen Glyphe mit eigenen Anweisungen** - bei
    ///   einer zusammengesetzten also an den Anfang der Komponente. Die
    ///   eigenen Anweisungen der zusammengesetzten Glyphe treffen dagegen
    ///   die Marke **hinter** dem letzten Punkt (`current.outline` beginnt
    ///   dort, wo `base` endet) und damit keinen gerasterten Punkt.
    ///
    /// Gemessen gegen pygames `freetype.dll` an 13 Schriften und 13
    /// Groessen, siehe `ftraster::tests::gegen_freetype`.
    pub(crate) fn rasterflaggen(
        &self,
        bytes: &[u8],
        index: u32,
        groesse26: (u32, u32),
        gid: u16,
    ) -> Option<Rasterflaggen> {
        let font = FontRef::from_index(bytes, index).ok()?;
        // `size->metrics.y_ppem` - gerundet (`FT_Request_Metrics`).
        let ppem = (groesse26.1 + 32) >> 6;
        let mut flags = 0u32;
        if ppem < 24 {
            flags |= ftraster::OUTLINE_HIGH_PRECISION;
        }
        // CFF (`cff_slot_load`): nur `HIGH_PRECISION` und
        // `FT_OUTLINE_REVERSE_FILL` - und das zweite wertet `ftraster.c`
        // nicht aus. Keine Dropout-Flaggen, keine Marken.
        let is_cff = font
            .outline_glyphs()
            .format()
            .is_some_and(|f| matches!(f, OutlineGlyphFormat::Cff | OutlineGlyphFormat::Cff2));
        if is_cff {
            return Some(Rasterflaggen {
                flags,
                marken: Vec::new(),
            });
        }
        let tabelle = self.instanzen.borrow();
        let instanz = tabelle.get(&groesse26)?.as_ref()?;
        let (scan_control, scan_type) = instanz.scan_zustand()?;
        let mut marken = Vec::new();
        flags |= if scan_control {
            match scan_type {
                0 => ftraster::OUTLINE_INCLUDE_STUBS,
                1 => 0,
                4 => ftraster::OUTLINE_SMART_DROPOUTS | ftraster::OUTLINE_INCLUDE_STUBS,
                5 => ftraster::OUTLINE_SMART_DROPOUTS,
                _ => ftraster::OUTLINE_IGNORE_DROPOUTS,
            }
        } else {
            ftraster::OUTLINE_IGNORE_DROPOUTS
        };
        // Marken schreibt nur `TT_Hint_Glyph` - also nur gehintet.
        if instanz.is_enabled() {
            // `(Byte)tags[0] >> 5` von `scan_type << 5`: die unteren drei Bit.
            let modus = (scan_type & 7) as u8;
            let mut start = 0usize;
            marken_sammeln(&font, GlyphId::new(gid as u32), &mut start, modus, &mut marken, 0);
        }
        Some(Rasterflaggen { flags, marken })
    }
}

/// Die Scan-Marken einer Glyphe: Anfang jeder einfachen Komponente mit
/// eigenen Anweisungen, in FreeTypes Punktreihenfolge.
fn marken_sammeln(
    font: &FontRef<'_>,
    gid: GlyphId,
    start: &mut usize,
    modus: u8,
    aus: &mut Vec<(usize, u8)>,
    tiefe: u32,
) {
    use skrifa::raw::tables::glyf::Glyph;
    use skrifa::raw::TableProvider;
    // FreeTypes Grenze fuer verschachtelte Komponenten ist hoeher; eine
    // Schrift, die tiefer schachtelt, ist kaputt.
    if tiefe > 16 {
        return;
    }
    let (Ok(loca), Ok(glyf)) = (font.loca(None), font.glyf()) else {
        return;
    };
    match loca.get_glyf(gid, &glyf) {
        Ok(Some(Glyph::Simple(g))) => {
            if !g.instructions().is_empty() {
                aus.push((*start, modus));
            }
            *start += g.num_points();
        }
        Ok(Some(Glyph::Composite(g))) => {
            for k in g.components() {
                marken_sammeln(font, k.glyph.into(), start, modus, aus, tiefe + 1);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eine_kontur(arten: &[Art], strom: &[Strich]) -> Option<Vec<(i64, i64)>> {
        zuordnen(arten, &[arten.len() - 1], strom)
    }

    #[test]
    fn gedachter_mittelpunkt_wird_kein_punkt() {
        // Auf, Kontrolle, Kontrolle, Auf: der Endpunkt des ersten Bogens ist
        // gedacht und darf nirgends landen.
        let arten = [Art::Auf, Art::Quadratisch, Art::Quadratisch, Art::Auf];
        let strom = [
            Strich::Anfang((0, 0)),
            Strich::Bogen((1, 10), (3, 10)),
            Strich::Bogen((5, 10), (6, 0)),
            Strich::Schluss,
        ];
        assert_eq!(
            eine_kontur(&arten, &strom),
            Some(vec![(0, 0), (1, 10), (5, 10), (6, 0)])
        );
    }

    #[test]
    fn echter_punkt_auf_dem_mittelpunkt_bleibt() {
        // Genau der Fall aus Consolas fett `g`: der echte Punkt liegt auf dem
        // abgeschnittenen Mittelpunkt seiner Nachbarn und muss bleiben.
        let arten = [
            Art::Auf,
            Art::Quadratisch,
            Art::Auf,
            Art::Quadratisch,
            Art::Auf,
        ];
        let strom = [
            Strich::Anfang((0, 0)),
            Strich::Bogen((-1, -1), (-1, -1)),
            Strich::Bogen((-2, -1), (5, 0)),
            Strich::Schluss,
        ];
        assert_eq!(
            eine_kontur(&arten, &strom),
            Some(vec![(0, 0), (-1, -1), (-1, -1), (-2, -1), (5, 0)])
        );
    }

    #[test]
    fn beginnt_mit_kontrollpunkt() {
        // Erster Punkt Kontrolle, letzter auf der Kurve: Anfang am letzten.
        let arten = [Art::Quadratisch, Art::Auf, Art::Auf];
        let strom = [
            Strich::Anfang((9, 9)),
            Strich::Bogen((1, 1), (2, 2)),
            Strich::Schluss,
        ];
        assert_eq!(
            eine_kontur(&arten, &strom),
            Some(vec![(1, 1), (2, 2), (9, 9)])
        );
    }

    #[test]
    fn cff_schliesst_wie_ps_builder() {
        // Schlusslinie auf den Anfang: der letzte Punkt faellt weg. Zwei
        // `moveTo` hintereinander legen keine Kontur an, erst der Strich.
        let strom = [
            Strich::Anfang((0, 0)),
            Strich::Strecke((0, 64)),
            Strich::Strecke((64, 64)),
            Strich::Strecke((64, 0)),
            Strich::Strecke((0, 0)),
            Strich::Schluss,
            Strich::Anfang((5, 5)),
            Strich::Anfang((10, 10)),
            Strich::Kubisch((10, 20), (20, 20), (20, 10)),
        ];
        let u = cff_umriss(&strom).unwrap();
        assert_eq!(
            u.punkte,
            vec![(0, 0), (0, 64), (64, 64), (64, 0), (10, 10), (10, 20), (20, 20), (20, 10)]
        );
        assert_eq!(u.enden, vec![3, 7]);
        assert_eq!(u.arten[5], Art::Kubisch);
        assert_eq!(u.arten[7], Art::Auf);
    }

    #[test]
    fn f26_rundet_ab_wie_rechtsschieben() {
        // `x >> 10` aus 16.16 ist fuer negative Werte die Abrundung.
        assert_eq!(f26(-0.5 / 64.0), -1);
        assert_eq!(f26(0.5 / 64.0), 0);
        assert_eq!(f26(-2.0), -128);
    }

    #[test]
    fn strom_passt_nicht() {
        let arten = [Art::Auf, Art::Auf];
        let strom = [
            Strich::Anfang((0, 0)),
            Strich::Bogen((1, 1), (2, 2)),
            Strich::Schluss,
        ];
        assert_eq!(eine_kontur(&arten, &strom), None);
    }
}
