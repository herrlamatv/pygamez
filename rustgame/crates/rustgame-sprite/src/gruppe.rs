//! Gruppen - `AbstractGroup`, `Group`, `RenderUpdates`, `GroupSingle`.
//!
//! Vorlage: `pygame/sprite.py`, Zeilen 361-724 und 1397-1464.
//!
//! ## Eine Bauform fuer alle Gruppen
//!
//! pygame hat fuenf Gruppenklassen, die sich in genau vier Punkten
//! unterscheiden: `add_internal`, `remove_internal`, `sprites` und `draw`.
//! Alles andere erbt unveraendert von `AbstractGroup`. Statt das ueber
//! `dyn`-Objekte und fuenf Kerne nachzubauen, gibt es hier **einen** Kern
//! mit einem Merkmal `Art` - die vier Stellen verzweigen darueber.
//!
//! Der Gewinn ist nicht die gesparte Zeile, sondern der Rueckverweis: ein
//! Sprite haelt `Weak<RefCell<GruppenKern>>`, einen einzigen festen Typ.
//! Mit fuenf Gruppentypen muesste dort `Weak<RefCell<dyn ...>>` stehen -
//! und dann ist `Weak::upgrade` auf ein unsized Ziel jedes Mal ein
//! kleiner Kampf.
//!
//! Nach aussen bleiben die Namen getrennt: [`Group`], [`RenderUpdates`],
//! [`GroupSingle`] und (in den Nachbarmodulen) `LayeredUpdates` und
//! `LayeredDirty` sind eigene Typen, die jeweils nur das zeigen, was es
//! bei ihnen auch gibt.
//!
//! ## Die Reihenfolge
//!
//! Pythons Woerterbuecher merken sich seit 3.7 die Einfuegereihenfolge,
//! und `AbstractGroup.sprites()` ist `list(self.spritedict)`. Damit ist
//! **jede** Gruppe geordnet, auch wenn ihr Text noch "in no particular
//! order" behauptet. Gemessen:
//!
//! ```text
//! g = Group(a, b, c); g.add(d)   -> [a, b, c, d]
//! g.remove(b); g.add(b)          -> [a, c, d, b]
//! ```
//!
//! Ein `Vec` gibt genau das her, deshalb steht hier einer.

use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use rustgame_core::{Rect, Surface};

use crate::sprite::{naechste_nummer, Sprite};

/// So wird eine Gruppe herumgereicht.
pub type GruppenHandle = Rc<RefCell<GruppenKern>>;

/// Welche pygame-Klasse dieser Kern darstellt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Art {
    /// `Group` (und damit `RenderPlain`, `RenderClear`).
    Einfach,
    /// `RenderUpdates` (und damit `OrderedUpdates`).
    Aktualisierend,
    /// `GroupSingle`.
    Einzel,
    /// `LayeredUpdates`.
    Ebenen,
    /// `LayeredDirty`.
    EbenenDirty,
}

/// pygames `bool(Rect)`.
///
/// **Nicht** dasselbe wie `Rect::is_empty`: gemessen ist
/// `bool(Rect(0, 0, -3, 4))` wahr, denn pygame prueft nur `w != 0 and
/// h != 0`. An drei Stellen (`AbstractGroup.remove_internal`,
/// `AbstractGroup.clear`, `RenderUpdates.draw`) haengt genau daran, ob ein
/// Rechteck mitgeschleppt wird.
pub(crate) fn wahr(r: Rect) -> bool {
    r.w != 0 && r.h != 0
}

/// Ein Eintrag in `lostsprites`.
///
/// Warum das kein schlichtes `Rect` ist: pygame legt in
/// `LayeredUpdates.remove_internal` das **lebende** Rechteck des Sprites
/// ab, nicht eine Kopie -
///
/// ```python
/// if hasattr(sprite, "rect"):
///     self.lostsprites.append(sprite.rect)   # dasselbe Objekt
/// ```
///
/// Bewegt sich das Sprite danach, wandert der Eintrag **mit**. Gemessen
/// an einem Sprite bei `(34, 24, 8, 5)`, das nach dem Entfernen um
/// `(-3, +2)` verschoben wird: pygame meldet `(31, 26, 8, 5)`, eine
/// Kopie haette `(34, 24, 8, 5)` gemeldet.
///
/// `rustgame_core::Rect` ist bewusst `Copy` - dort steht der Grund. Die
/// Referenzsemantik lebt darum hier, an der einen Stelle, an der sie
/// beobachtbar ist. Alle anderen Eintraege sind feste Rechtecke: sie
/// kommen aus `blit`-Rueckgaben oder aus `clip` und sind auch in pygame
/// frische Objekte.
#[derive(Debug, Clone)]
pub(crate) enum Verloren {
    /// Ein festes Rechteck.
    Fest(Rect),
    /// Das Rechteck dieses Sprites, wie es **jetzt** gerade steht.
    VomSprite(Sprite),
}

impl Verloren {
    pub(crate) fn rect(&self) -> Rect {
        match self {
            Verloren::Fest(r) => *r,
            Verloren::VomSprite(s) => s.rect().unwrap_or(Rect::new(0, 0, 0, 0)),
        }
    }
}

// ---------------------------------------------------------------------------
//  Der Kern
// ---------------------------------------------------------------------------

/// Der Inhalt einer Gruppe hinter dem `RefCell`.
pub struct GruppenKern {
    pub(crate) nummer: u64,
    pub(crate) art: Art,
    /// `spritedict`: Sprite -> zuletzt gezeichnetes Rechteck.
    /// `None` ist pygames `None` beziehungsweise `_init_rect`.
    pub(crate) eintraege: Vec<(Sprite, Option<Rect>)>,
    /// Spiegel von `eintraege` fuer die Mitgliedsfrage in O(1).
    pub(crate) drin: HashSet<u64>,
    pub(crate) lostsprites: Vec<Verloren>,

    // ----- nur Ebenen / EbenenDirty -----
    /// `_spritelist` - nach Ebene sortiert, hinten oben.
    pub(crate) reihenfolge: Vec<Sprite>,
    /// `_spritelayers`.
    pub(crate) ebenen: HashMap<u64, i32>,
    /// `_default_layer`.
    pub(crate) default_layer: i32,

    // ----- nur EbenenDirty -----
    pub(crate) clip: Option<Rect>,
    pub(crate) use_update: bool,
    pub(crate) time_threshold: f64,
    pub(crate) bgd: Option<Surface>,

    // ----- nur Einzel -----
    pub(crate) einzel: Option<Sprite>,
}

impl std::fmt::Debug for GruppenKern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GruppenKern")
            .field("nummer", &self.nummer)
            .field("art", &self.art)
            .field("sprites", &self.sprites().len())
            .finish()
    }
}

impl GruppenKern {
    pub(crate) fn neu(art: Art) -> GruppenHandle {
        Rc::new(RefCell::new(GruppenKern {
            nummer: naechste_nummer(),
            art,
            eintraege: Vec::new(),
            drin: HashSet::new(),
            lostsprites: Vec::new(),
            reihenfolge: Vec::new(),
            ebenen: HashMap::new(),
            default_layer: 0,
            clip: None,
            use_update: false,
            // 1000.0 / 80.0 - gemessen als 12.5.
            time_threshold: 1000.0 / 80.0,
            bgd: None,
            einzel: None,
        }))
    }

    // ----- Lesen --------------------------------------------------------

    /// `sprites()` - die Sprites in Zeichenreihenfolge.
    pub(crate) fn sprites(&self) -> Vec<Sprite> {
        match self.art {
            Art::Einzel => self.einzel.iter().cloned().collect(),
            Art::Ebenen | Art::EbenenDirty => self.reihenfolge.clone(),
            _ => self.eintraege.iter().map(|(s, _)| s.clone()).collect(),
        }
    }

    /// `has_internal(sprite)`.
    pub(crate) fn has_internal(&self, s: &Sprite) -> bool {
        match self.art {
            // `GroupSingle.has_internal` fragt **nicht** das Woerterbuch,
            // sondern den einen Platz - das ist der Grund, warum ein
            // durch `draw` in `spritedict` gelandetes Sprite dort trotzdem
            // nicht "drin" ist.
            Art::Einzel => self.einzel.as_ref().is_some_and(|e| e.ist(s)),
            _ => self.drin.contains(&s.nummer()),
        }
    }

    /// Das zuletzt gezeichnete Rechteck eines Sprites.
    pub(crate) fn rect_von(&self, s: &Sprite) -> Option<Rect> {
        let n = s.nummer();
        self.eintraege
            .iter()
            .find(|(sp, _)| sp.nummer() == n)
            .and_then(|(_, r)| *r)
    }

    /// `spritedict[sprite] = rect` - legt den Eintrag an, wenn es ihn
    /// noch nicht gibt. Genau Pythons Zuweisungssemantik, und genau
    /// deshalb hat eine `GroupSingle` nach dem ersten `draw` ploetzlich
    /// einen Eintrag (gemessen).
    pub(crate) fn setze_rect(&mut self, s: &Sprite, r: Option<Rect>) {
        let n = s.nummer();
        if let Some(e) = self.eintraege.iter_mut().find(|(sp, _)| sp.nummer() == n) {
            e.1 = r;
        } else {
            self.eintraege.push((s.clone(), r));
            self.drin.insert(n);
        }
    }

    /// Nimmt `lostsprites` heraus und macht feste Rechtecke daraus.
    pub(crate) fn lostsprites_aufloesen(&mut self) -> Vec<Rect> {
        std::mem::take(&mut self.lostsprites)
            .iter()
            .map(Verloren::rect)
            .collect()
    }

    /// `AbstractGroup.remove_internal` - nur der Woerterbuchteil.
    fn entferne_eintrag(&mut self, s: &Sprite) {
        let n = s.nummer();
        if let Some(pos) = self.eintraege.iter().position(|(sp, _)| sp.nummer() == n) {
            if let Some(r) = self.eintraege[pos].1 {
                if wahr(r) {
                    self.lostsprites.push(Verloren::Fest(r));
                }
            }
            self.eintraege.remove(pos);
            self.drin.remove(&n);
        }
    }

    // ----- Halbe Buchungen ---------------------------------------------

    /// `group.add_internal(sprite, layer)` - nur die Gruppenseite.
    pub(crate) fn add_internal(g: &GruppenHandle, s: &Sprite, layer: Option<i32>) {
        let mut k = g.borrow_mut();
        match k.art {
            Art::Einzel => {
                // Der alte Bewohner fliegt vollstaendig raus - beide
                // Buecher. Gemessen: `x1.alive()` ist danach `False`.
                if let Some(alt) = k.einzel.take() {
                    alt.remove_internal(k.nummer);
                    k.entferne_eintrag(&alt);
                }
                k.einzel = Some(s.clone());
            }
            Art::Ebenen | Art::EbenenDirty => {
                if k.art == Art::EbenenDirty {
                    // `LayeredDirty.add_internal` verlangt ein
                    // DirtySprite (pygame: AttributeError / TypeError).
                    // Hier wird still uebersprungen, was keines ist -
                    // ein Panic mitten im Aufbau einer Szene waere die
                    // schlechtere Haelfte des Tauschs.
                    if !s.ist_dirty_sprite() {
                        return;
                    }
                    if s.dirty() == Some(0) {
                        s.set_dirty(1);
                    }
                }
                k.setze_rect(s, None);

                // Welche Ebene? Die drei Faelle aus
                // `LayeredUpdates.add_internal` - und der mittlere ist
                // der ueberraschende (siehe `layered`).
                let ebene = match layer {
                    None => match s.layer() {
                        Some(l) => l,
                        None => {
                            let l = k.default_layer;
                            s.set_layer_intern(l);
                            l
                        }
                    },
                    Some(l) => {
                        if s.layer().is_some() {
                            s.set_layer_intern(l);
                        }
                        l
                    }
                };
                k.ebenen.insert(s.nummer(), ebene);
                let stelle = k.bisect(ebene);
                k.reihenfolge.insert(stelle, s.clone());
            }
            _ => k.setze_rect(s, None),
        }
    }

    /// `group.remove_internal(sprite)` - nur die Gruppenseite.
    pub(crate) fn remove_internal(g: &GruppenHandle, s: &Sprite) {
        let mut k = g.borrow_mut();
        match k.art {
            Art::Einzel => {
                if k.einzel.as_ref().is_some_and(|e| e.ist(s)) {
                    k.einzel = None;
                }
                k.entferne_eintrag(s);
            }
            Art::Ebenen | Art::EbenenDirty => {
                let n = s.nummer();
                k.reihenfolge.retain(|sp| sp.nummer() != n);
                // Hier zaehlt **Gleichheit mit `_init_rect`**, nicht die
                // Wahrheit des Rechtecks: ein leeres, aber gezeichnetes
                // Rechteck wandert sehr wohl in `lostsprites`.
                if let Some(alt) = k.rect_von(s) {
                    k.lostsprites.push(Verloren::Fest(alt));
                }
                // **Das lebende Rechteck**, nicht sein heutiger Wert -
                // siehe `Verloren`.
                if s.rect().is_some() {
                    k.lostsprites.push(Verloren::VomSprite(s.clone()));
                }
                if let Some(pos) = k.eintraege.iter().position(|(sp, _)| sp.nummer() == n) {
                    k.eintraege.remove(pos);
                }
                k.drin.remove(&n);
                k.ebenen.remove(&n);
            }
            _ => k.entferne_eintrag(s),
        }
    }

    // ----- Ganze Buchungen ---------------------------------------------

    /// `group.add(sprite)` - beide Buecher, wenn das Sprite noch fehlt.
    pub(crate) fn add(g: &GruppenHandle, s: &Sprite, layer: Option<i32>) {
        if g.borrow().has_internal(s) {
            return;
        }
        let nummer = g.borrow().nummer;
        GruppenKern::add_internal(g, s, layer);
        // `LayeredDirty` lehnt Nicht-DirtySprites ab; dann darf auch die
        // Sprite-Seite nichts eintragen.
        if g.borrow().has_internal(s) {
            s.add_internal(nummer, g);
        }
    }

    /// `group.remove(sprite)` - beide Buecher.
    pub(crate) fn remove(g: &GruppenHandle, s: &Sprite) {
        if !g.borrow().has_internal(s) {
            return;
        }
        let nummer = g.borrow().nummer;
        GruppenKern::remove_internal(g, s);
        s.remove_internal(nummer);
    }

    // ----- Ebenen-Handwerk ---------------------------------------------

    /// Die Einfuegestelle fuer `ebene` in `reihenfolge`.
    ///
    /// Zeilengetreu aus `LayeredUpdates.add_internal`: eine Halbierung,
    /// gefolgt von einer linearen Nachsuche. Beide zusammen liefern die
    /// Stelle **hinter** allen Sprites mit `layer <= ebene` - gleiche
    /// Ebene heisst also "hinten anstellen", und das haelt die
    /// Einfuegereihenfolge innerhalb einer Ebene.
    ///
    /// Gerechnet wird in `isize`, weil `high` mit `leng - 1` bei leerer
    /// Liste auf `-1` startet.
    pub(crate) fn bisect(&self, ebene: i32) -> usize {
        let leng = self.reihenfolge.len() as isize;
        let mut low: isize = 0;
        let mut mid: isize = 0;
        let mut high: isize = leng - 1;
        while low <= high {
            mid = low + (high - low) / 2;
            if self.ebene_an(mid) <= ebene {
                low = mid + 1;
            } else {
                high = mid - 1;
            }
        }
        while mid < leng && self.ebene_an(mid) <= ebene {
            mid += 1;
        }
        mid as usize
    }

    fn ebene_an(&self, i: isize) -> i32 {
        let s = &self.reihenfolge[i as usize];
        self.ebenen[&s.nummer()]
    }

    // ----- Zeichnen -----------------------------------------------------

    /// Der gemeinsame Einstieg fuer alle `draw`-Fassungen.
    ///
    /// `flags` ist `None` nur bei [`Art::EbenenDirty`] wirklich von
    /// `Some(0)` verschieden: dort heisst `None` "nimm `sprite.blendmode`".
    pub(crate) fn zeichnen(
        &mut self,
        surface: &mut Surface,
        bgd: Option<&Surface>,
        flags: Option<u32>,
    ) -> Vec<Rect> {
        match self.art {
            Art::EbenenDirty => crate::dirty::zeichnen_dirty(self, surface, bgd, flags),
            Art::Aktualisierend => self.zeichnen_aktualisierend(surface, flags.unwrap_or(0)),
            Art::Ebenen => self.zeichnen_ebenen(surface, flags.unwrap_or(0)),
            _ => self.zeichnen_einfach(surface, flags.unwrap_or(0)),
        }
    }

    /// `AbstractGroup.draw`.
    ///
    /// **Gibt immer eine leere Liste zurueck.** Der Quelltext liest sich
    /// anders, tut aber genau das:
    ///
    /// ```python
    /// self.lostsprites = []
    /// dirty = self.lostsprites   # zeigt auf die *neue* leere Liste
    /// return dirty
    /// ```
    ///
    /// Gemessen und bestaetigt - `Group.draw()` liefert `[]`, im ersten
    /// wie im zweiten Aufruf.
    fn zeichnen_einfach(&mut self, surface: &mut Surface, flags: u32) -> Vec<Rect> {
        for s in self.sprites() {
            if let Some(r) = blit_sprite(surface, &s, None, flags) {
                self.setze_rect(&s, Some(r));
            }
        }
        self.lostsprites.clear();
        Vec::new()
    }

    /// `RenderUpdates.draw` - das alte und das neue Rechteck.
    fn zeichnen_aktualisierend(&mut self, surface: &mut Surface, flags: u32) -> Vec<Rect> {
        let mut dirty = self.lostsprites_aufloesen();
        for s in self.sprites() {
            let alt = self.rect_von(&s).filter(|r| wahr(*r));
            let Some(neu) = blit_sprite(surface, &s, None, flags) else {
                continue;
            };
            match alt {
                Some(o) if neu.colliderect(o) => dirty.push(neu.union(o)),
                Some(o) => {
                    dirty.push(neu);
                    dirty.push(o);
                }
                None => dirty.push(neu),
            }
            self.setze_rect(&s, Some(neu));
        }
        dirty
    }

    /// `LayeredUpdates.draw` - wie oben, aber das alte Rechteck zaehlt
    /// schon, wenn es ueberhaupt eines gibt (`is not _init_rect`), auch
    /// ein leeres.
    fn zeichnen_ebenen(&mut self, surface: &mut Surface, flags: u32) -> Vec<Rect> {
        let mut dirty = self.lostsprites_aufloesen();
        for s in self.sprites() {
            let alt = self.rect_von(&s);
            let Some(neu) = blit_sprite(surface, &s, None, flags) else {
                continue;
            };
            match alt {
                None => dirty.push(neu),
                Some(r) if neu.colliderect(r) => dirty.push(neu.union(r)),
                Some(r) => {
                    dirty.push(neu);
                    dirty.push(r);
                }
            }
            self.setze_rect(&s, Some(neu));
        }
        dirty
    }
}

/// Zeichnet ein Sprite an seine Stelle; `None`, wenn Bild oder Rechteck
/// fehlt.
///
/// pygame wirft hier `AttributeError` ("'NoneType' has no attribute
/// ..."). Ein Panic waere das genaue Gegenstueck, aber ein halb
/// aufgebautes Sprite soll nicht das ganze Bild kosten - es wird
/// uebersprungen und bekommt keinen Eintrag im Woerterbuch.
pub(crate) fn blit_sprite(
    surface: &mut Surface,
    s: &Sprite,
    area: Option<Rect>,
    flags: u32,
) -> Option<Rect> {
    let bild = s.image()?;
    let r = s.rect()?;
    Some(surface.blit_ex(&bild, (r.x, r.y), area, flags))
}

// ---------------------------------------------------------------------------
//  Was sich einhaengen laesst
// ---------------------------------------------------------------------------

/// Alles, was einer Gruppe uebergeben werden darf.
///
/// pygames `Group.add(*sprites)` nimmt ein Sprite, eine Liste, eine
/// andere Gruppe oder beliebig verschachtelte Mischungen davon. Das ist
/// dort Ententypisierung mit `try/except`; hier ist es ein Merkmal.
pub trait Hinzufuegbar {
    /// Die betroffenen Sprites, in der Reihenfolge, in der pygame sie
    /// abarbeiten wuerde.
    fn sprites_fuer(&self) -> Vec<Sprite>;
}

impl Hinzufuegbar for Sprite {
    fn sprites_fuer(&self) -> Vec<Sprite> {
        vec![self.clone()]
    }
}

impl Hinzufuegbar for crate::sprite::DirtySprite {
    fn sprites_fuer(&self) -> Vec<Sprite> {
        vec![self.as_sprite().clone()]
    }
}

impl Hinzufuegbar for [Sprite] {
    fn sprites_fuer(&self) -> Vec<Sprite> {
        self.to_vec()
    }
}

impl Hinzufuegbar for Vec<Sprite> {
    fn sprites_fuer(&self) -> Vec<Sprite> {
        self.clone()
    }
}

impl<const N: usize> Hinzufuegbar for [Sprite; N] {
    fn sprites_fuer(&self) -> Vec<Sprite> {
        self.to_vec()
    }
}

impl<T: Hinzufuegbar + ?Sized> Hinzufuegbar for &T {
    fn sprites_fuer(&self) -> Vec<Sprite> {
        (**self).sprites_fuer()
    }
}

// ---------------------------------------------------------------------------
//  Das Merkmal AbstractGroup
// ---------------------------------------------------------------------------

/// Was jede Gruppe kann - Gegenstueck zu `pygame.sprite.AbstractGroup`.
///
/// In pygame ist das die Basisklasse; hier ein Merkmal mit einer einzigen
/// Pflichtmethode ([`AbstractGroup::kern`]) und lauter fertigen
/// Standardmethoden. Wer eine eigene Gruppenart bauen will, wickelt einen
/// [`GruppenHandle`] ein und ist fertig.
pub trait AbstractGroup {
    /// Der geteilte Kern dieser Gruppe.
    fn kern(&self) -> &GruppenHandle;

    /// Die laufende Nummer - stimmt auch, wenn die Gruppe schon weg ist.
    fn nummer(&self) -> u64 {
        self.kern().borrow().nummer
    }

    /// Zeigen zwei Griffe auf dieselbe Gruppe?
    fn ist<G: AbstractGroup>(&self, other: &G) -> bool {
        Rc::ptr_eq(self.kern(), other.kern())
    }

    /// `group.sprites()` - in Zeichenreihenfolge.
    fn sprites(&self) -> Vec<Sprite> {
        self.kern().borrow().sprites()
    }

    /// `len(group)`.
    fn len(&self) -> usize {
        self.kern().borrow().sprites().len()
    }

    /// `bool(group)` umgedreht.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// `group.add(...)` - Sprite, Liste oder andere Gruppe.
    ///
    /// Ein Sprite, das schon drin ist, wird uebersprungen; es kann nicht
    /// doppelt in derselben Gruppe stehen (gemessen: `len` bleibt gleich,
    /// und `sprite.groups()` waechst nicht).
    fn add(&self, was: impl Hinzufuegbar) {
        for s in was.sprites_fuer() {
            GruppenKern::add(self.kern(), &s, None);
        }
    }

    /// `group.remove(...)`.
    fn remove(&self, was: impl Hinzufuegbar) {
        for s in was.sprites_fuer() {
            GruppenKern::remove(self.kern(), &s);
        }
    }

    /// `group.has(...)` - sind **alle** genannten Sprites drin?
    ///
    /// Ohne Argumente `false`, genau wie pygames `if not sprites:
    /// return False`.
    fn has(&self, was: impl Hinzufuegbar) -> bool {
        let liste = was.sprites_fuer();
        if liste.is_empty() {
            return false;
        }
        let k = self.kern().borrow();
        liste.iter().all(|s| k.has_internal(s))
    }

    /// Steckt genau dieses eine Sprite drin?
    fn has_internal(&self, s: &Sprite) -> bool {
        self.kern().borrow().has_internal(s)
    }

    /// `group.empty()` - alle Sprites raus, die Gruppe bleibt.
    fn empty(&self) {
        for s in self.sprites() {
            GruppenKern::remove(self.kern(), &s);
        }
    }

    /// `group.update()` - ruft bei jedem Sprite `update()`.
    fn update(&self) {
        self.update_mit(&());
    }

    /// `group.update(*args)`.
    ///
    /// Gemessen: die Argumente werden **unveraendert durchgereicht**,
    /// jedes Sprite sieht genau das, was der Gruppe uebergeben wurde.
    /// Hier ist das ein `&dyn Any`, das die eigene Verhaltensart wieder
    /// herunterreicht.
    fn update_mit(&self, args: &dyn Any) {
        for s in self.sprites() {
            s.update(args);
        }
    }

    /// `group.draw(surface)`.
    fn draw(&self, surface: &mut Surface) -> Vec<Rect> {
        self.kern().borrow_mut().zeichnen(surface, None, None)
    }

    /// `group.draw(surface, special_flags=...)`.
    fn draw_ex(&self, surface: &mut Surface, special_flags: u32) -> Vec<Rect> {
        self.kern()
            .borrow_mut()
            .zeichnen(surface, None, Some(special_flags))
    }

    /// `group.clear(surface, bgd)` - uebermalt die alten Stellen.
    ///
    /// Erst die verlorenen Rechtecke, dann die zuletzt gezeichneten -
    /// die Reihenfolge ist gemessen und nicht beliebig: ein entferntes
    /// Sprite muss weg sein, bevor die Gebliebenen ihre Flaeche
    /// zurueckbekommen.
    ///
    /// `lostsprites` wird dabei **nicht** geleert; das macht erst das
    /// naechste `draw`.
    fn clear(&self, surface: &mut Surface, bgd: &Surface) {
        {
            // `LayeredDirty.clear` ersetzt die geerbte Bedeutung
            // vollstaendig: es legt nur den Hintergrund ab und faerbt
            // nichts. Die Weiche steht hier statt in einer zweiten
            // Fassung, damit sie sich nicht umgehen laesst.
            let mut k = self.kern().borrow_mut();
            if k.art == Art::EbenenDirty {
                k.bgd = Some(bgd.clone());
                return;
            }
        }
        let k = self.kern().borrow();
        for v in &k.lostsprites {
            let r = v.rect();
            surface.blit_ex(bgd, (r.x, r.y), Some(r), 0);
        }
        for (_, rect) in &k.eintraege {
            if let Some(r) = rect {
                if wahr(*r) {
                    surface.blit_ex(bgd, (r.x, r.y), Some(*r), 0);
                }
            }
        }
    }

    /// Wie [`AbstractGroup::clear`], nur mit einer Funktion statt einer
    /// Flaeche - pygames `clear(surface, callable)`.
    fn clear_mit<F: FnMut(&mut Surface, Rect)>(&self, surface: &mut Surface, mut bgd: F) {
        let rechtecke: Vec<Rect> = {
            let k = self.kern().borrow();
            k.lostsprites
                .iter()
                .map(Verloren::rect)
                .chain(k.eintraege.iter().filter_map(|(_, r)| *r).filter(|r| wahr(*r)))
                .collect()
        };
        for r in rechtecke {
            bgd(surface, r);
        }
    }

    /// Die Rechtecke der zuletzt entfernten Sprites.
    fn lostsprites(&self) -> Vec<Rect> {
        self.kern().borrow().lostsprites.iter().map(Verloren::rect).collect()
    }

    /// `spritedict` als Liste - Sprite und zuletzt gezeichnetes Rechteck.
    fn spritedict(&self) -> Vec<(Sprite, Option<Rect>)> {
        self.kern().borrow().eintraege.clone()
    }
}

// ---------------------------------------------------------------------------
//  Die Typen
// ---------------------------------------------------------------------------

/// Baut einen Gruppentyp: Huelle, Konstruktoren, Merkmale.
/// Der gemeinsame Rumpf aller Gruppentypen.
///
/// Der `pub struct` steht bewusst **nicht** hier drin, sondern bei jedem
/// Typ ausgeschrieben. Ein Name, den nur der Makroaufruf kennt, ist im
/// Quelltext nicht auffindbar - weder mit `grep` noch mit
/// `tools/api_coverage.py`, das die Abdeckungstabelle aus den
/// `pub`-Zeilen liest. Fuenf ausgeschriebene Zeilen sind das wert.
macro_rules! gruppen_rumpf {
    ($name:ident, $art:expr) => {
        impl $name {
            /// Eine leere Gruppe.
            pub fn new() -> Self {
                $name(GruppenKern::neu($art))
            }

            /// Eine Gruppe mit Inhalt - `Group(*sprites)`.
            pub fn mit(was: impl Hinzufuegbar) -> Self {
                let g = Self::new();
                g.add(was);
                g
            }

            /// `group.copy()` - neue Gruppe derselben Art, dieselben
            /// Sprites.
            ///
            /// Die Sprites stecken danach in **beiden** Gruppen
            /// (gemessen: `groups()` zaehlt 2). Die Kopie entsteht wie in
            /// pygame ueber `Klasse(sprites())`, also **ohne** die
            /// Einstellungen der Vorlage - bei den Ebenengruppen kostet
            /// das die Ebenen.
            pub fn copy(&self) -> Self {
                Self::mit(self.sprites())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl AbstractGroup for $name {
            fn kern(&self) -> &GruppenHandle {
                &self.0
            }
        }

        impl Hinzufuegbar for $name {
            fn sprites_fuer(&self) -> Vec<Sprite> {
                self.sprites()
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "<{}({} sprites)>", stringify!($name), self.len())
            }
        }
    };
}

pub(crate) use gruppen_rumpf;

/// Der einfache Sprite-Behaelter - `pygame.sprite.Group`.
///
/// `draw` zeichnet alle Sprites und gibt **eine leere Liste** zurueck;
/// wer die veraenderten Bereiche braucht, nimmt [`RenderUpdates`].
#[derive(Clone)]
pub struct Group(GruppenHandle);
gruppen_rumpf!(Group, Art::Einfach);

/// Gruppe, die die veraenderten Bereiche mitfuehrt -
/// `pygame.sprite.RenderUpdates`.
///
/// `draw` liefert je Sprite entweder die Vereinigung aus altem und neuem
/// Rechteck (wenn sie sich beruehren) oder beide einzeln - genau das,
/// was `display.update` braucht.
#[derive(Clone)]
pub struct RenderUpdates(GruppenHandle);
gruppen_rumpf!(RenderUpdates, Art::Aktualisierend);

/// `pygame.sprite.RenderPlain`.
///
/// In pygame-ce 2.5.7 eine eigene Unterklasse von `Group`, die im
/// Konstruktor eine `DeprecationWarning` wirft und **sonst nichts**
/// aendert (nachgemessen: `RenderPlain.draw is AbstractGroup.draw` ist
/// `True`). Damit ist sie verhaltensgleich mit [`Group`], und eine
/// Namensgleichung ist hier die ehrliche Uebersetzung - eine zweite
/// Kopie des Codes waere schlicht falsch.
pub type RenderPlain = Group;

/// `pygame.sprite.RenderClear` - wie [`RenderPlain`] eine verhaltens-
/// gleiche, veraltete Unterklasse von `Group`.
pub type RenderClear = Group;

/// Gruppe mit genau einem Platz - `pygame.sprite.GroupSingle`.
///
/// Wer hinzukommt, wirft den Vorgaenger hinaus: der alte Bewohner ist
/// danach in **keiner** Gruppe mehr (`alive()` ist `false`), nicht nur in
/// dieser nicht.
#[derive(Clone)]
pub struct GroupSingle(GruppenHandle);
gruppen_rumpf!(GroupSingle, Art::Einzel);

impl GroupSingle {
    /// `group.sprite` - der eine Bewohner.
    pub fn sprite(&self) -> Option<Sprite> {
        self.kern().borrow().einzel.clone()
    }

    /// `group.sprite = s`.
    ///
    /// Anders als [`AbstractGroup::add`] geht das auch dann durch die
    /// volle Buchung, wenn `s` schon drinsteht - so steht es in pygames
    /// `_set_sprite`.
    pub fn set_sprite(&self, s: &Sprite) {
        let nummer = self.nummer();
        GruppenKern::add_internal(self.kern(), s, None);
        if !s.kennt_gruppe(nummer) {
            s.add_internal(nummer, self.kern());
        }
    }
}

// ---------------------------------------------------------------------------
//  Eine Gruppe, deren Art man nicht kennt
// ---------------------------------------------------------------------------

/// Eine Gruppe ohne festen Typ - das, was [`Sprite::groups`] liefert.
///
/// Ein Sprite merkt sich seine Gruppen als Kerne; welcher Typ sie einmal
/// eingewickelt hat, ist danach nicht mehr zu sehen. Fuer alles aus
/// [`AbstractGroup`] macht das keinen Unterschied.
///
/// [`Sprite::groups`]: crate::Sprite::groups
#[derive(Clone)]
pub struct IrgendeineGruppe(pub(crate) GruppenHandle);

impl AbstractGroup for IrgendeineGruppe {
    fn kern(&self) -> &GruppenHandle {
        &self.0
    }
}

impl Hinzufuegbar for IrgendeineGruppe {
    fn sprites_fuer(&self) -> Vec<Sprite> {
        self.sprites()
    }
}

impl std::fmt::Debug for IrgendeineGruppe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Group({} sprites)>", self.len())
    }
}
