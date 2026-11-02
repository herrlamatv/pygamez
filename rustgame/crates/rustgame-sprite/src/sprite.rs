//! Das Sprite selbst - und wie eigener Zustand daran kommt.
//!
//! Vorlage: `pygame/sprite.py`, Zeilen 96-358 (`Sprite`, `DirtySprite`).
//!
//! Die Doppelbuchfuehrung zwischen Sprite und Gruppe ist hier genauso
//! aufgeteilt wie dort: `add`/`remove` sind die oeffentliche Haelfte und
//! rufen **beide** Seiten, `add_internal`/`remove_internal` tragen nur je
//! eine Seite nach und werden von der Gegenseite gerufen. Wer eine der
//! vier Stellen vergisst, bekommt ein Sprite, das sich fuer lebendig
//! haelt, waehrend keine Gruppe es kennt - der klassische Fehler bei
//! diesem Muster.

use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::{Rc, Weak};

use rustgame_core::{Rect, Surface};
use rustgame_math::Mask;

use crate::gruppe::GruppenKern;

// ---------------------------------------------------------------------------
//  Kennungen
// ---------------------------------------------------------------------------

/// Laufende Nummer fuer Sprites und Gruppen.
///
/// Gebraucht wird sie, weil ein toter [`Weak`] sich nicht mehr mit
/// `Rc::ptr_eq` vergleichen laesst und `Weak::as_ptr` eine Adresse liefert,
/// die das Betriebssystem spaeter neu vergeben darf. Eine eigene Nummer
/// kann nicht wiederverwendet werden und ist damit die einzige
/// Kennung, die auch ueber den Tod hinaus stimmt.
pub(crate) fn naechste_nummer() -> u64 {
    use std::cell::Cell;
    thread_local! {
        static ZAEHLER: Cell<u64> = const { Cell::new(1) };
    }
    ZAEHLER.with(|z| {
        let n = z.get();
        z.set(n + 1);
        n
    })
}

// ---------------------------------------------------------------------------
//  Eigenes Verhalten
// ---------------------------------------------------------------------------

/// Eigener Zustand und eigenes `update` fuer ein Sprite.
///
/// Das Gegenstueck zu "von `Sprite` erben und `update` ueberschreiben".
/// Die Gruppenverwaltung bleibt davon unberuehrt - sie kennt nur
/// [`Sprite`].
///
/// Beim Aufruf haelt niemand eine Ausleihe auf den Sprite-Kern, der Kasten
/// ist waehrenddessen herausgenommen. `s` darf also frei benutzt werden,
/// bis hin zu [`Sprite::kill`].
pub trait SpriteVerhalten: 'static {
    /// Wird von `Group::update` gerufen.
    ///
    /// `args` ist, was der Gruppe uebergeben wurde - bei `update()` ohne
    /// Argumente ist es `&()`.
    fn update(&mut self, s: &Sprite, args: &dyn Any);
}

/// Die objektsichere Fassung von [`SpriteVerhalten`] - nur intern.
///
/// Der Umweg spart dem Benutzer die beiden `as_any`-Rumpfmethoden, die
/// man sonst in jede eigene Verhaltensart schreiben muesste.
trait VerhaltenObj: 'static {
    fn update_obj(&mut self, s: &Sprite, args: &dyn Any);
    fn als_any(&self) -> &dyn Any;
    fn als_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: SpriteVerhalten> VerhaltenObj for T {
    fn update_obj(&mut self, s: &Sprite, args: &dyn Any) {
        self.update(s, args);
    }
    fn als_any(&self) -> &dyn Any {
        self
    }
    fn als_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// ---------------------------------------------------------------------------
//  DirtySprite-Zusatzdaten
// ---------------------------------------------------------------------------

/// Die fuenf Felder, die `DirtySprite` ueber `Sprite` hinaus hat.
///
/// Sie liegen hier statt in einem eigenen Typ, weil `LayeredDirty` an
/// einem beliebigen Sprite pruefen muss, ob es sie hat - in pygame ist das
/// `isinstance(sprite, DirtySprite)`, hier `dirty_daten.is_some()`.
#[derive(Debug, Clone)]
pub(crate) struct DirtyDaten {
    /// `0` sauber, `1` einmal neu zeichnen, `2` jedes Bild neu zeichnen.
    pub(crate) dirty: i32,
    /// `special_flags` fuer den Blit.
    pub(crate) blendmode: u32,
    pub(crate) visible: i32,
    pub(crate) source_rect: Option<Rect>,
}

impl Default for DirtyDaten {
    fn default() -> Self {
        DirtyDaten {
            dirty: 1,
            blendmode: 0,
            visible: 1,
            source_rect: None,
        }
    }
}

// ---------------------------------------------------------------------------
//  Der Kern
// ---------------------------------------------------------------------------

/// Der Inhalt eines Sprites hinter dem `RefCell`.
pub struct SpriteKern {
    pub(crate) nummer: u64,
    pub(crate) image: Option<Surface>,
    pub(crate) rect: Option<Rect>,
    /// `sprite.radius` - von [`crate::collide_circle`] bei Bedarf
    /// **nachgetragen**, genau wie in pygame.
    pub(crate) radius: Option<f64>,
    /// `sprite.mask` - wird nie von selbst gesetzt.
    pub(crate) mask: Option<Mask>,
    /// `sprite._layer`. `None` heisst "das Attribut gibt es nicht" -
    /// in pygame ein `AttributeError`, und der ist an zwei Stellen
    /// verhaltensrelevant (siehe [`crate::layered`]).
    pub(crate) layer: Option<i32>,
    pub(crate) dirty_daten: Option<DirtyDaten>,
    /// `Sprite.__g`: die Gruppen, in denen dieses Sprite steckt.
    /// Reihenfolge wie in pygames Woerterbuch - Einfuegereihenfolge.
    pub(crate) gruppen: Vec<(u64, Weak<RefCell<GruppenKern>>)>,
    verhalten: Option<Box<dyn VerhaltenObj>>,
}

impl std::fmt::Debug for SpriteKern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpriteKern")
            .field("nummer", &self.nummer)
            .field("rect", &self.rect)
            .field("hat_bild", &self.image.is_some())
            .field("gruppen", &self.gruppen.len())
            .finish()
    }
}

// ---------------------------------------------------------------------------
//  Sprite
// ---------------------------------------------------------------------------

/// Ein Spielobjekt - Gegenstueck zu `pygame.sprite.Sprite`.
///
/// Ein handlicher Griff auf einen geteilten Kern. `clone()` ist billig und
/// liefert **dasselbe** Sprite, nicht eine Kopie - so wie in Python ein
/// zweiter Name auf dasselbe Objekt zeigt.
#[derive(Clone)]
pub struct Sprite(pub(crate) Rc<RefCell<SpriteKern>>);

impl Sprite {
    /// Ein neues Sprite ohne Bild, ohne Rechteck, in keiner Gruppe.
    pub fn new() -> Sprite {
        Sprite(Rc::new(RefCell::new(SpriteKern {
            nummer: naechste_nummer(),
            image: None,
            rect: None,
            radius: None,
            mask: None,
            layer: None,
            dirty_daten: None,
            gruppen: Vec::new(),
            verhalten: None,
        })))
    }

    /// Ein Sprite mit Bild; das Rechteck bekommt die Groesse des Bildes.
    ///
    /// Der Kurzweg fuer das, was in pygame in jedem `__init__` steht:
    /// `self.image = ...; self.rect = self.image.get_rect()`.
    pub fn mit_bild(image: Surface) -> Sprite {
        let r = image.get_rect();
        let s = Sprite::new();
        {
            let mut k = s.0.borrow_mut();
            k.image = Some(image);
            k.rect = Some(r);
        }
        s
    }

    /// Die unveraenderliche Kennung dieses Sprites.
    pub fn nummer(&self) -> u64 {
        self.0.borrow().nummer
    }

    /// Zeigen zwei Griffe auf dasselbe Sprite?
    pub fn ist(&self, other: &Sprite) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    /// Leseausleihe auf den Kern - fuer alles, was die Zugriffe unten
    /// nicht abdecken.
    pub fn borrow(&self) -> Ref<'_, SpriteKern> {
        self.0.borrow()
    }

    /// Schreibausleihe auf den Kern.
    pub fn borrow_mut(&self) -> RefMut<'_, SpriteKern> {
        self.0.borrow_mut()
    }

    // ----- Bild und Rechteck -------------------------------------------

    /// `sprite.image` - eine Flaeche, die sich den Pixelspeicher mit dem
    /// Original teilt.
    pub fn image(&self) -> Option<Surface> {
        self.0.borrow().image.clone()
    }

    /// Setzt das Bild. `None` nimmt es weg.
    pub fn set_image(&self, image: Option<Surface>) {
        self.0.borrow_mut().image = image;
    }

    /// `sprite.rect`.
    pub fn rect(&self) -> Option<Rect> {
        self.0.borrow().rect
    }

    /// Setzt das Rechteck. Siehe den Modulkopf dazu, warum es ein
    /// **Objekt** sein muss und keine Kopie.
    pub fn set_rect(&self, rect: Option<Rect>) {
        self.0.borrow_mut().rect = rect;
    }

    /// Verschiebt das Rechteck - das Gegenstueck zu `sprite.rect.x += dx`.
    ///
    /// Ohne Rechteck passiert nichts, so wie in pygame ein `None.x` erst
    /// beim Zugriff auffaellt.
    pub fn bewege(&self, dx: i32, dy: i32) {
        if let Some(r) = self.0.borrow_mut().rect.as_mut() {
            r.move_ip(dx, dy);
        }
    }

    /// Setzt die linke obere Ecke.
    pub fn set_topleft(&self, pos: (i32, i32)) {
        if let Some(r) = self.0.borrow_mut().rect.as_mut() {
            r.set_topleft(pos);
        }
    }

    /// `sprite.radius` - von [`crate::collide_circle`] bei Bedarf gesetzt.
    pub fn radius(&self) -> Option<f64> {
        self.0.borrow().radius
    }

    /// Setzt den Radius fuer die Kreis-Kollisionspruefung.
    pub fn set_radius(&self, radius: Option<f64>) {
        self.0.borrow_mut().radius = radius;
    }

    /// `sprite.mask` - wird nie von selbst gesetzt (gemessen: nach
    /// `collide_mask` hat das Sprite **kein** `mask`-Attribut).
    pub fn mask(&self) -> Option<Mask> {
        self.0.borrow().mask.clone()
    }

    /// Setzt die Maske fuer die punktgenaue Kollisionspruefung.
    pub fn set_mask(&self, mask: Option<Mask>) {
        self.0.borrow_mut().mask = mask;
    }

    // ----- Ebene --------------------------------------------------------

    /// `sprite.layer`. `None` heisst: das Attribut gibt es nicht - in
    /// pygame wirft der Zugriff dann `AttributeError`.
    pub fn layer(&self) -> Option<i32> {
        self.0.borrow().layer
    }

    /// `sprite.layer = wert`.
    ///
    /// Geht nur, solange das Sprite in **keiner** Gruppe ist; danach wirft
    /// pygame `AttributeError` und verweist auf `group.change_layer`.
    /// Gemessen und hier als [`Err`] nachgebaut.
    pub fn set_layer(&self, wert: i32) -> Result<(), LayerFehler> {
        if self.alive() {
            return Err(LayerFehler);
        }
        self.0.borrow_mut().layer = Some(wert);
        Ok(())
    }

    /// Setzt `_layer` ohne die Pruefung - nur fuer die Gruppen.
    pub(crate) fn set_layer_intern(&self, wert: i32) {
        self.0.borrow_mut().layer = Some(wert);
    }

    // ----- DirtySprite-Felder ------------------------------------------

    /// Ist das ein [`DirtySprite`]? (`isinstance(s, DirtySprite)`)
    pub fn ist_dirty_sprite(&self) -> bool {
        self.0.borrow().dirty_daten.is_some()
    }

    /// `sprite.dirty`.
    pub fn dirty(&self) -> Option<i32> {
        self.0.borrow().dirty_daten.as_ref().map(|d| d.dirty)
    }

    /// Setzt `dirty` - **nur bei einer `DirtySprite`**, sonst wirkungslos.
    pub fn set_dirty(&self, wert: i32) {
        if let Some(d) = self.0.borrow_mut().dirty_daten.as_mut() {
            d.dirty = wert;
        }
    }

    /// `sprite.visible`.
    pub fn visible(&self) -> Option<i32> {
        self.0.borrow().dirty_daten.as_ref().map(|d| d.visible)
    }

    /// `sprite.visible = wert`.
    ///
    /// Macht das Sprite nebenbei schmutzig - sonst bliebe das alte Bild
    /// stehen. Steht in pygame in `_set_visible`: `if self.dirty < 2:
    /// self.dirty = 1`.
    pub fn set_visible(&self, wert: i32) {
        if let Some(d) = self.0.borrow_mut().dirty_daten.as_mut() {
            d.visible = wert;
            if d.dirty < 2 {
                d.dirty = 1;
            }
        }
    }

    /// `sprite.blendmode` - die `special_flags` fuer den Blit.
    pub fn blendmode(&self) -> Option<u32> {
        self.0.borrow().dirty_daten.as_ref().map(|d| d.blendmode)
    }

    /// Setzt den Mischmodus - nur bei einer `DirtySprite`.
    pub fn set_blendmode(&self, wert: u32) {
        if let Some(d) = self.0.borrow_mut().dirty_daten.as_mut() {
            d.blendmode = wert;
        }
    }

    /// `sprite.source_rect` - welcher Teil von `image` gezeichnet wird.
    pub fn source_rect(&self) -> Option<Rect> {
        self.0.borrow().dirty_daten.as_ref().and_then(|d| d.source_rect)
    }

    /// Setzt den gezeichneten Ausschnitt - nur bei einer `DirtySprite`.
    pub fn set_source_rect(&self, wert: Option<Rect>) {
        if let Some(d) = self.0.borrow_mut().dirty_daten.as_mut() {
            d.source_rect = wert;
        }
    }

    // ----- Eigener Zustand ---------------------------------------------

    /// Haengt eigenen Zustand samt `update` an das Sprite.
    ///
    /// Ein zweiter Aufruf ersetzt den alten Kasten.
    pub fn set_verhalten(&self, v: impl SpriteVerhalten) {
        self.0.borrow_mut().verhalten = Some(Box::new(v));
    }

    /// Liest im eigenen Zustand. `None`, wenn keiner da ist oder der
    /// Typ nicht passt.
    pub fn mit_zustand<T: SpriteVerhalten, R>(&self, f: impl FnOnce(&T) -> R) -> Option<R> {
        let k = self.0.borrow();
        let v = k.verhalten.as_ref()?;
        v.als_any().downcast_ref::<T>().map(f)
    }

    /// Wie [`Sprite::mit_zustand`], aber veraendernd.
    ///
    /// Waehrend `f` laeuft, ist der Sprite-Kern ausgeliehen - `f` darf
    /// also **nicht** auf dasselbe Sprite zugreifen. Wer das braucht,
    /// nimmt [`SpriteVerhalten::update`], dort ist der Kasten
    /// herausgenommen.
    pub fn mit_zustand_mut<T: SpriteVerhalten, R>(&self, f: impl FnOnce(&mut T) -> R) -> Option<R> {
        let mut k = self.0.borrow_mut();
        let v = k.verhalten.as_mut()?;
        v.als_any_mut().downcast_mut::<T>().map(f)
    }

    /// Ruft das eigene `update` - Gegenstueck zu `Sprite.update(*args)`.
    ///
    /// Ohne eigenen Zustand passiert nichts; das ist pygames leerer
    /// Rumpf.
    ///
    /// Der Kasten wird **herausgenommen**, bevor `update` laeuft, und
    /// danach zurueckgelegt. Dadurch ist der Sprite-Kern waehrend des
    /// Aufrufs frei und `update` darf das Sprite bewegen, sein Bild
    /// tauschen oder es toeten. Legt `update` selbst ein neues Verhalten
    /// ab, gewinnt das neue - der alte Kasten wird dann nicht
    /// zurueckgeschoben.
    pub fn update(&self, args: &dyn Any) {
        let kasten = self.0.borrow_mut().verhalten.take();
        if let Some(mut v) = kasten {
            v.update_obj(self, args);
            let mut k = self.0.borrow_mut();
            if k.verhalten.is_none() {
                k.verhalten = Some(v);
            }
        }
    }

    // ----- Gruppenverwaltung -------------------------------------------

    /// `sprite.add(*groups)` - haengt das Sprite in Gruppen ein.
    ///
    /// Gruppen, in denen es schon steckt, werden uebersprungen; ein
    /// Sprite kann nicht doppelt in derselben Gruppe sein (gemessen).
    pub fn add<G: crate::gruppe::AbstractGroup>(&self, gruppe: &G) {
        gruppe.add(self);
    }

    /// `sprite.remove(*groups)`.
    pub fn remove<G: crate::gruppe::AbstractGroup>(&self, gruppe: &G) {
        gruppe.remove(self);
    }

    /// Traegt die Gruppe nur auf der Sprite-Seite ein
    /// (`Sprite.add_internal`).
    pub(crate) fn add_internal(&self, nummer: u64, gruppe: &Rc<RefCell<GruppenKern>>) {
        self.0
            .borrow_mut()
            .gruppen
            .push((nummer, Rc::downgrade(gruppe)));
    }

    /// Streicht die Gruppe nur auf der Sprite-Seite
    /// (`Sprite.remove_internal`).
    pub(crate) fn remove_internal(&self, nummer: u64) {
        self.0.borrow_mut().gruppen.retain(|(n, _)| *n != nummer);
    }

    /// Steckt das Sprite in dieser Gruppe? (Sprite-Seite des Buches.)
    pub(crate) fn kennt_gruppe(&self, nummer: u64) -> bool {
        self.0.borrow().gruppen.iter().any(|(n, _)| *n == nummer)
    }

    /// `sprite.kill()` - raus aus allen Gruppen.
    ///
    /// Das Sprite selbst bleibt heil und darf danach wieder eingehaengt
    /// werden (gemessen).
    pub fn kill(&self) {
        // Erst die Liste herausnehmen: `remove_internal` der Gruppe kann
        // ueber `GroupSingle` wieder beim Sprite landen, und dann wuerde
        // eine laufende Ausleihe platzen.
        let gruppen = std::mem::take(&mut self.0.borrow_mut().gruppen);
        for (_, schwach) in gruppen {
            if let Some(g) = schwach.upgrade() {
                GruppenKern::remove_internal(&g, self);
            }
        }
    }

    /// `sprite.groups()` - die Gruppen, in denen dieses Sprite steckt.
    ///
    /// **Abweichung von pygame**, bewusst: eine Gruppe, die sonst niemand
    /// mehr haelt, ist hier weg und wird nicht mitgezaehlt. In pygame
    /// haelt das Sprite sie am Leben. Siehe Modulkopf von
    /// [`crate`].
    pub fn groups(&self) -> Vec<crate::gruppe::IrgendeineGruppe> {
        self.0
            .borrow()
            .gruppen
            .iter()
            .filter_map(|(_, w)| w.upgrade())
            .map(crate::gruppe::IrgendeineGruppe)
            .collect()
    }

    /// Anzahl der Gruppen, ohne sie anzufassen.
    pub fn anzahl_gruppen(&self) -> usize {
        self.0
            .borrow()
            .gruppen
            .iter()
            .filter(|(_, w)| w.strong_count() > 0)
            .count()
    }

    /// `sprite.alive()` - steckt es in mindestens einer Gruppe?
    pub fn alive(&self) -> bool {
        self.anzahl_gruppen() > 0
    }
}

impl Default for Sprite {
    fn default() -> Self {
        Sprite::new()
    }
}

impl PartialEq for Sprite {
    /// Gleich heisst **dasselbe** Sprite, nicht "sieht gleich aus" -
    /// wie Pythons Vorgabe fuer Objekte ohne eigenes `__eq__`.
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for Sprite {}

impl std::hash::Hash for Sprite {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.nummer().hash(state);
    }
}

impl std::fmt::Debug for Sprite {
    /// Nachbau von `Sprite.__repr__`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let art = if self.ist_dirty_sprite() {
            "DirtySprite"
        } else {
            "Sprite"
        };
        write!(f, "<{art}(in {} groups)>", self.anzahl_gruppen())
    }
}

/// `sprite.layer = wert`, nachdem das Sprite schon in einer Gruppe ist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayerFehler;

impl std::fmt::Display for LayerFehler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            "layer laesst sich nach dem Einhaengen nicht mehr direkt setzen - \
             gruppe.change_layer(sprite, neue_ebene) nehmen",
        )
    }
}

impl std::error::Error for LayerFehler {}

// ---------------------------------------------------------------------------
//  DirtySprite
// ---------------------------------------------------------------------------

/// Ein Sprite mit `dirty`, `visible`, `blendmode` und `source_rect` -
/// Gegenstueck zu `pygame.sprite.DirtySprite`.
///
/// Es ist **dasselbe** [`Sprite`], nur mit den Zusatzfeldern belegt: nach
/// `let s: Sprite = d.clone().into()` zeigen beide Griffe auf einen Kern.
/// Deshalb nimmt jede Gruppe beides entgegen, und [`LayeredDirty`] kann
/// mit [`Sprite::ist_dirty_sprite`] pruefen, was in pygame
/// `isinstance(sprite, DirtySprite)` ist.
///
/// [`LayeredDirty`]: crate::LayeredDirty
#[derive(Clone)]
pub struct DirtySprite(Sprite);

impl DirtySprite {
    /// Neues DirtySprite: `dirty = 1`, `visible = 1`, `blendmode = 0`,
    /// `source_rect = None`, `_layer = 0`.
    ///
    /// Das `_layer = 0` ist kein Beiwerk: es entscheidet in
    /// [`LayeredUpdates`](crate::LayeredUpdates), ob ein ausdrueckliches
    /// `layer=` auf dem Sprite haengen bleibt.
    pub fn new() -> DirtySprite {
        let s = Sprite::new();
        {
            let mut k = s.0.borrow_mut();
            k.dirty_daten = Some(DirtyDaten::default());
            k.layer = Some(0);
        }
        DirtySprite(s)
    }

    /// Wie [`Sprite::mit_bild`], nur als DirtySprite.
    pub fn mit_bild(image: Surface) -> DirtySprite {
        let d = DirtySprite::new();
        let r = image.get_rect();
        d.0.set_image(Some(image));
        d.0.set_rect(Some(r));
        d
    }

    /// Der Griff auf dasselbe Sprite.
    pub fn as_sprite(&self) -> &Sprite {
        &self.0
    }

    /// `dirty` - `0` sauber, `1` einmal, `2` immer.
    pub fn dirty(&self) -> i32 {
        self.0.dirty().unwrap_or(0)
    }

    /// `visible` - `0` versteckt, `1` sichtbar.
    pub fn visible(&self) -> i32 {
        self.0.visible().unwrap_or(1)
    }

    /// `blendmode` - die `special_flags` des Blits.
    pub fn blendmode(&self) -> u32 {
        self.0.blendmode().unwrap_or(0)
    }
}

impl Default for DirtySprite {
    fn default() -> Self {
        DirtySprite::new()
    }
}

impl std::ops::Deref for DirtySprite {
    type Target = Sprite;
    fn deref(&self) -> &Sprite {
        &self.0
    }
}

impl From<DirtySprite> for Sprite {
    fn from(d: DirtySprite) -> Sprite {
        d.0
    }
}

impl std::fmt::Debug for DirtySprite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}
