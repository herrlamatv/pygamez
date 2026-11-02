//! `LayeredUpdates` - die Gruppe mit Ebenen.
//!
//! Vorlage: `pygame/sprite.py`, Zeilen 726-1056.
//!
//! ## Vier Buecher, die gleichlaufen muessen
//!
//! pygame fuehrt hier vier Woerterbuecher beziehungsweise Listen parallel:
//!
//! | pygame | hier | Inhalt |
//! |---|---|---|
//! | `spritedict` | `eintraege` | Sprite -> zuletzt gezeichnetes Rechteck |
//! | `_spritelist` | `reihenfolge` | alle Sprites, nach Ebene sortiert |
//! | `_spritelayers` | `ebenen` | Sprite -> Ebene |
//! | `lostsprites` | `lostsprites` | Rechtecke entfernter Sprites |
//!
//! Sie werden an genau zwei Stellen fortgeschrieben (`add_internal` und
//! `remove_internal`, beide in [`crate::gruppe`]) - jede weitere Stelle
//! waere eine Gelegenheit, sie auseinanderlaufen zu lassen.
//!
//! ## Die Ebenenwahl, und warum sie ueberrascht
//!
//! `add_internal(sprite, layer)` kennt drei Faelle. Zwei sind
//! naheliegend, der dritte nicht:
//!
//! 1. **`layer` fehlt, Sprite hat `_layer`** -> die des Sprites.
//! 2. **`layer` fehlt, Sprite hat kein `_layer`** -> die Vorgabeebene,
//!    und sie wird dem Sprite als `_layer` **angeschrieben**.
//! 3. **`layer` ist angegeben** -> diese Ebene; `_layer` wird dem Sprite
//!    aber nur dann angeschrieben, wenn es das Attribut **schon hat**.
//!
//! Fall 3 heisst: ein einfaches `Sprite`, das mit `add(s, layer=5)`
//! eingehaengt wurde, hat danach **kein** `_layer` (gemessen: der Zugriff
//! wirft `AttributeError`) - ein `DirtySprite` dagegen schon, denn das
//! bringt `_layer = 0` mit auf die Welt.
//!
//! Die Folge steht in `copy()`: die Kopie verliert die Ebenen von
//! Fall-3-Sprites, weil sie bei `Klasse(sprites())` wieder in Fall 2
//! laufen. Gemessen an drei Sprites auf den Ebenen 1 und 2: die Kopie
//! meldet `layers() == [0]`. Kein Versehen dieser Uebersetzung, sondern
//! pygames Verhalten.

use rustgame_core::Rect;

use crate::gruppe::{
    gruppen_rumpf, AbstractGroup, Art, GruppenHandle, GruppenKern, Hinzufuegbar,
};
use crate::sprite::Sprite;

/// Gruppe mit Ebenen - `pygame.sprite.LayeredUpdates`.
///
/// Gezeichnet wird von der untersten Ebene zur obersten, innerhalb einer
/// Ebene in der Reihenfolge des Einhaengens. `draw` liefert wie bei
/// [`RenderUpdates`](crate::RenderUpdates) die veraenderten Bereiche.
/// Die Ebenenmethoden stehen in [`EbenenGruppe`].
#[derive(Clone)]
pub struct LayeredUpdates(GruppenHandle);
gruppen_rumpf!(LayeredUpdates, Art::Ebenen);

/// `pygame.sprite.OrderedUpdates`.
///
/// In pygame-ce 2.5.7 eine Unterklasse von `RenderUpdates`, die nichts
/// weiter tut als zu warnen - ihre eigene Warnung sagt es selbst:
/// *"OrderedUpdates is now just an alias to RenderUpdates, order of
/// sprites is now maintained in all sprite Group classes."* Nachgemessen:
/// `OrderedUpdates.draw is RenderUpdates.draw` ist `True`. Eine
/// Namensgleichung ist damit die richtige Uebersetzung.
pub type OrderedUpdates = crate::gruppe::RenderUpdates;

/// `change_layer` auf ein Sprite, das gar nicht in der Gruppe ist.
///
/// pygame laeuft dort in `list.remove(x): x not in list` - ein
/// `ValueError` mitten aus einer Hilfsmethode heraus. Hier ist es ein
/// ordentliches [`Err`]; gemessen wurde der Fall, weil der Text der
/// Methode ausdruecklich sagt "This is not checked".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NichtInGruppe;

impl std::fmt::Display for NichtInGruppe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("das Sprite steckt nicht in dieser Gruppe")
    }
}

impl std::error::Error for NichtInGruppe {}

// ---------------------------------------------------------------------------
//  Das Merkmal
// ---------------------------------------------------------------------------

/// Die Ebenenmethoden - von [`LayeredUpdates`] und
/// [`LayeredDirty`](crate::LayeredDirty) geteilt.
///
/// In pygame erbt `LayeredDirty` sie von `LayeredUpdates`. Hier ist es
/// ein Merkmal, damit beide Typen dieselben Rumpfmethoden benutzen statt
/// sie zu verdoppeln - und damit eigener Code ueber "irgendeine Gruppe
/// mit Ebenen" schreiben kann.
pub trait EbenenGruppe: AbstractGroup {
    /// `_default_layer`.
    fn default_layer(&self) -> i32 {
        self.kern().borrow().default_layer
    }

    /// `group.add(*sprites, layer=n)` - alles auf eine feste Ebene.
    fn add_mit_ebene(&self, was: impl Hinzufuegbar, layer: i32) {
        for s in was.sprites_fuer() {
            GruppenKern::add(self.kern(), &s, Some(layer));
        }
    }

    /// `group.layers()` - die vorkommenden Ebenen, aufsteigend, ohne
    /// Wiederholung.
    fn layers(&self) -> Vec<i32> {
        let k = self.kern().borrow();
        let mut v: Vec<i32> = k.ebenen.values().copied().collect();
        v.sort_unstable();
        v.dedup();
        v
    }

    /// `group.get_layer_of_sprite(sprite)`.
    ///
    /// Fuer ein Sprite, das gar nicht drin ist, liefert pygame die
    /// **Vorgabeebene** statt eines Fehlers - gemessen, und hier genauso.
    fn get_layer_of_sprite(&self, s: &Sprite) -> i32 {
        let k = self.kern().borrow();
        k.ebenen.get(&s.nummer()).copied().unwrap_or(k.default_layer)
    }

    /// `group.change_layer(sprite, neue_ebene)`.
    fn change_layer(&self, s: &Sprite, neue_ebene: i32) -> Result<(), NichtInGruppe> {
        change_layer_intern(self.kern(), s, neue_ebene)
    }

    /// `group.get_sprite(idx)` - das Sprite an dieser Stelle der
    /// Zeichenreihenfolge.
    fn get_sprite(&self, idx: usize) -> Option<Sprite> {
        self.kern().borrow().reihenfolge.get(idx).cloned()
    }

    /// `group.get_sprites_at(pos)` - alle Sprites unter diesem Punkt,
    /// unterstes zuerst.
    fn get_sprites_at(&self, pos: (i32, i32)) -> Vec<Sprite> {
        let k = self.kern().borrow();
        let punkt = Rect::from_parts(pos, (1, 1));
        let rechtecke: Vec<Rect> = k
            .reihenfolge
            .iter()
            // Ein Sprite ohne Rechteck kann nie getroffen werden; ein
            // leeres Rechteck an seiner Stelle haelt die Nummerierung
            // gerade.
            .map(|s| s.rect().unwrap_or(Rect::new(0, 0, 0, 0)))
            .collect();
        punkt
            .collidelistall(&rechtecke)
            .into_iter()
            .map(|i| k.reihenfolge[i].clone())
            .collect()
    }

    /// `group.get_sprites_from_layer(layer)` - in der Reihenfolge des
    /// Einhaengens.
    fn get_sprites_from_layer(&self, layer: i32) -> Vec<Sprite> {
        let k = self.kern().borrow();
        let mut raus = Vec::new();
        for s in &k.reihenfolge {
            let e = k.ebenen[&s.nummer()];
            if e == layer {
                raus.push(s.clone());
            } else if e > layer {
                // Die Liste ist sortiert - danach kommt nichts mehr.
                break;
            }
        }
        raus
    }

    /// `group.remove_sprites_of_layer(layer)` - nimmt sie heraus und
    /// gibt sie zurueck.
    ///
    /// Die zurueckgegebenen Sprites sind danach in keiner Gruppe mehr
    /// (gemessen: `alive()` ist `false`).
    fn remove_sprites_of_layer(&self, layer: i32) -> Vec<Sprite> {
        let sprites = self.get_sprites_from_layer(layer);
        self.remove(&sprites);
        sprites
    }

    /// `group.get_top_layer()`.
    fn get_top_layer(&self) -> Option<i32> {
        let k = self.kern().borrow();
        k.reihenfolge.last().map(|s| k.ebenen[&s.nummer()])
    }

    /// `group.get_bottom_layer()`.
    fn get_bottom_layer(&self) -> Option<i32> {
        let k = self.kern().borrow();
        k.reihenfolge.first().map(|s| k.ebenen[&s.nummer()])
    }

    /// `group.get_top_sprite()`.
    fn get_top_sprite(&self) -> Option<Sprite> {
        self.kern().borrow().reihenfolge.last().cloned()
    }

    /// `group.move_to_front(sprite)` - auf die oberste Ebene, dort ganz
    /// nach hinten.
    ///
    /// Die Ebene des Sprites **aendert sich** dabei auf die oberste;
    /// gemessen an `[f1@0, f2@1, f3@1]` -> `[f2, f3, f1]`, und `f1` liegt
    /// danach auf Ebene 1.
    fn move_to_front(&self, s: &Sprite) -> Result<(), NichtInGruppe> {
        match self.get_top_layer() {
            Some(l) => self.change_layer(s, l),
            None => Err(NichtInGruppe),
        }
    }

    /// `group.move_to_back(sprite)` - auf eine **neue** Ebene unter der
    /// bisher untersten.
    fn move_to_back(&self, s: &Sprite) -> Result<(), NichtInGruppe> {
        match self.get_bottom_layer() {
            Some(l) => self.change_layer(s, l - 1),
            None => Err(NichtInGruppe),
        }
    }

    /// `group.switch_layer(l1, l2)` - tauscht die Bewohner zweier Ebenen.
    fn switch_layer(&self, layer1: i32, layer2: i32) {
        let aus1 = self.remove_sprites_of_layer(layer1);
        for s in self.get_sprites_from_layer(layer2) {
            let _ = self.change_layer(&s, layer1);
        }
        self.add_mit_ebene(&aus1, layer2);
    }
}

impl EbenenGruppe for LayeredUpdates {}

impl LayeredUpdates {
    /// Eine leere Gruppe mit einer anderen Vorgabeebene -
    /// `LayeredUpdates(default_layer=n)`.
    pub fn mit_default_layer(default_layer: i32) -> LayeredUpdates {
        let g = LayeredUpdates::new();
        g.kern().borrow_mut().default_layer = default_layer;
        g
    }
}

/// `change_layer` - von beiden Ebenengruppen benutzt.
///
/// Der Ablauf ist der von `add_internal`, nur andersherum angefangen:
/// erst aus Liste und Ebenenbuch heraus, dann ueber dieselbe Halbierung
/// wieder hinein. `_layer` wird dem Sprite nur angeschrieben, wenn es das
/// Attribut schon hat - dieselbe Regel wie in Fall 3 des Modulkopfs.
///
/// Das Schmutzigmachen am Ende steht in pygame in
/// `LayeredDirty.change_layer`, also eine Ebene hoeher. Hier liegt es
/// unten, weil es sonst die einzige Zeile waere, fuer die sich der ganze
/// Umweg ueber eine zweite Fassung lohnen muesste.
pub(crate) fn change_layer_intern(
    g: &GruppenHandle,
    s: &Sprite,
    neue_ebene: i32,
) -> Result<(), NichtInGruppe> {
    let art = {
        let mut k = g.borrow_mut();
        let n = s.nummer();
        let Some(pos) = k.reihenfolge.iter().position(|sp| sp.nummer() == n) else {
            return Err(NichtInGruppe);
        };
        k.reihenfolge.remove(pos);
        k.ebenen.remove(&n);

        // `bisect` liest `ebenen`; das Sprite darf dabei nicht darin
        // stehen - genau wie in pygame, wo `pop` vor der Suche kommt.
        let stelle = k.bisect(neue_ebene);
        k.reihenfolge.insert(stelle, s.clone());
        if s.layer().is_some() {
            s.set_layer_intern(neue_ebene);
        }
        k.ebenen.insert(n, neue_ebene);
        k.art
    };
    if art == Art::EbenenDirty && s.dirty() == Some(0) {
        s.set_dirty(1);
    }
    Ok(())
}
