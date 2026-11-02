//! rustgame-sprite
//! ===============
//! Gegenstueck zu `pygame.sprite`.
//!
//! Die Vorlage ist mit 1788 Zeilen **reines Python** (`pygame/sprite.py`) -
//! kein C. Das ist der seltene Fall, in dem die Logik Zeile fuer Zeile
//! lesbar ist. Sie wurde trotzdem an echtem pygame **nachgemessen**; drei
//! Ergebnisse haben die Erwartung widerlegt und stehen unten.
//!
//! ## Die eine Entscheidung: wer haelt wen am Leben
//!
//! In pygame kennt jedes Sprite seine Gruppen (`Sprite.__g`) und jede
//! Gruppe ihre Sprites (`AbstractGroup.spritedict`) - **beide Richtungen
//! stark**. Pythons Abfallsammler loest den Zyklus auf. In Rust gibt es
//! den nicht, zwei `Rc` im Kreis waeren ein Leck.
//!
//! Hier gilt darum:
//!
//! > **Die Gruppe haelt das Sprite (`Rc`), das Sprite kennt die Gruppe nur
//! > schwach (`Weak`).**
//!
//! Begruendung, in dieser Reihenfolge:
//!
//! 1. **So wird es benutzt.** Der uebliche Ablauf ist
//!    `Group::new().add(Gegner::neu(..))` - danach faellt die oertliche
//!    Variable weg und die Gruppe ist der Besitzer. Andersherum ist
//!    "ein Sprite haelt eine Gruppe am Leben, die sonst niemand mehr
//!    kennt" ein Zustand, den kein Spiel absichtlich herbeifuehrt.
//! 2. **`kill()` bleibt richtig.** Das Sprite laeuft seine schwachen
//!    Verweise ab, hebt jeden an und streicht sich aus den Gruppen, die
//!    es noch gibt. Tote Verweise ueberspringt es - genau das, was
//!    pygame tut, wenn eine Gruppe vorher aufgeloest wurde.
//! 3. **Es ist derselbe Weg wie im Kern.** `Surface` teilt seinen
//!    Pixelspeicher ueber [`rustgame_core::surface::Shared`], also
//!    `Rc<RefCell<T>>`. Eine zweite Bauart im selben Projekt waere
//!    Ballast.
//!
//! **Der eine messbare Unterschied** steht in
//! [`Sprite::groups`]: faellt in pygame die letzte aeussere Variable einer
//! Gruppe weg, haelt das Sprite sie weiter am Leben und `groups()` zaehlt
//! sie mit (gemessen: `1`). Hier wird sie freigegeben und `groups()` zaehlt
//! sie nicht mehr (`0`). Das ist die Rechnung fuer "kein Leck"; der Test
//! `gruppe_faellt_weg_sprite_ueberlebt` nagelt es fest.
//!
//! ## Erweiterbar ohne Vererbung
//!
//! In pygame erbt man von `Sprite` und ueberschreibt `update`. Hier traegt
//! das Sprite einen Kasten mit eigenem Zustand: [`SpriteVerhalten`].
//! Beim Aktualisieren wird der Kasten **herausgenommen**, gerufen und
//! zurueckgelegt - deshalb darf `update` das Sprite ganz normal
//! veraendern, bewegen oder sogar [`Sprite::kill`] rufen, ohne dass sich
//! zwei `RefCell`-Ausleihen in die Quere kommen. Der eigene Zustand ist
//! ueber [`Sprite::mit_zustand`] wieder zu erreichen.
//!
//! ```
//! use rustgame_core::Rect;
//! use rustgame_sprite::{AbstractGroup, Group, Sprite, SpriteVerhalten};
//! use std::any::Any;
//!
//! struct Faller { tempo: i32 }
//!
//! impl SpriteVerhalten for Faller {
//!     fn update(&mut self, s: &Sprite, _args: &dyn Any) {
//!         s.bewege(0, self.tempo);
//!     }
//! }
//!
//! let s = Sprite::new();
//! s.set_rect(Some(Rect::new(0, 0, 4, 4)));
//! s.set_verhalten(Faller { tempo: 3 });
//!
//! let g = Group::new();
//! g.add(&s);
//! g.update();
//! assert_eq!(s.rect().unwrap().y, 3);
//! assert_eq!(s.mit_zustand(|f: &Faller| f.tempo), Some(3));
//! ```
//!
//! ## Was die Messung widerlegt hat
//!
//! | Frage | Erwartung | gemessen |
//! |---|---|---|
//! | Was gibt `Group.draw()` zurueck? | die veraenderten Rechtecke | **immer `[]`** - `draw` setzt `lostsprites` auf eine neue leere Liste und gibt *die* zurueck |
//! | Sind `RenderPlain`/`RenderClear`/`OrderedUpdates` Namensgleichungen? | ja, laut Auftrag | **nein** - in pygame-ce 2.5.7 sind es echte Unterklassen, die nur eine `DeprecationWarning` werfen und sonst nichts aendern (`OrderedUpdates.draw is RenderUpdates.draw` -> `True`) |
//! | Setzt `LayeredDirty.draw` im Vollbildmodus `dirty` zurueck? | ja | **nein** - nur der Weg ueber `_use_update` raeumt die Flaggen ab |
//! | Ueberlebt `LayeredUpdates.copy()` die Ebenen? | ja | **nein** - `copy()` ruft `Klasse(sprites())` ohne `layer`, und ein einfaches `Sprite`, das mit `layer=` eingehaengt wurde, hat gar kein `_layer`; alles landet auf der Vorgabeebene |
//!
//! ## Wer diese Kiste benutzt - und wer nicht
//!
//! Sie ist die **Rust-eigene** Sprite-Welt. Die Python-Huelle
//! (`rustgame-py/src/figur.rs`) baut **nicht** darauf auf, sondern
//! uebertraegt `pygame/sprite.py` noch einmal auf Python-Objekte. Der
//! Grund ist eine einzige gemessene Zeile:
//!
//! ```python
//! self.rect.x += 1
//! ```
//!
//! Sie wirkt nur, wenn `sprite.rect` **jedes Mal dasselbe Python-Objekt**
//! liefert; Python uebersetzt sie zu `tmp = self.rect; tmp.x = tmp.x + 1`
//! und schreibt nichts zurueck. Hier liegt das Rechteck als [`Rect`] im
//! Kern, und `PyRect` traegt seinen Wert im Python-Objekt - ein Zugriff
//! muesste also ein frisches `PyRect` bauen, und die Bewegung waere weg.
//! Dazu kommt, dass pygames Gruppen entenartig arbeiten: `Group.draw`
//! verlangt nur `spr.image` und `spr.rect`, kein `isinstance`.
//!
//! Das ist **keine** Doppelung, die sich zusammenlegen liesse: die eine
//! Seite haelt Rust-Werte, die andere Python-Objekte. Wer hier etwas
//! aendert, aendert nichts an `pygame.sprite` - und umgekehrt.
//!
//! ## Aufbau
//!
//! | Modul | wofuer |
//! |---|---|
//! | [`sprite`] | [`Sprite`], [`DirtySprite`], [`SpriteVerhalten`] |
//! | [`gruppe`] | [`AbstractGroup`], [`Group`], [`RenderUpdates`], [`GroupSingle`] |
//! | [`layered`] | [`LayeredUpdates`] - vier gleichlaufende Buecher |
//! | [`dirty`] | [`LayeredDirty`] - `dirty == 1` gegen `dirty == 2` |
//! | [`collide`] | die acht Kollisionsfunktionen |

// **Kein `unsafe` in dieser Kiste, und das prueft der Uebersetzer.**
//
// Hier stand der Satz lange nur in der Beschreibung - und eine Behauptung,
// die niemand nachhaelt, veraltet still. Siehe
// `rustgame-py/src/puffer.rs`: dort liegt das `unsafe` des Projekts, und
// nur dort ist der Lint ausdruecklich abgeschaltet.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod collide;
pub mod dirty;
pub mod gruppe;
pub mod layered;
pub mod sprite;

pub use collide::{
    collide_circle, collide_circle_ratio, collide_mask, collide_mask_trifft, collide_rect,
    collide_rect_ratio, groupcollide, spritecollide, spritecollideany, Kollision,
};
pub use dirty::LayeredDirty;
pub use gruppe::{
    AbstractGroup, Group, GroupSingle, GruppenHandle, Hinzufuegbar, IrgendeineGruppe, RenderClear,
    RenderPlain, RenderUpdates,
};
pub use layered::{EbenenGruppe, LayeredUpdates, NichtInGruppe, OrderedUpdates};
pub use sprite::{DirtySprite, LayerFehler, Sprite, SpriteVerhalten};

/// Weitergereicht, damit niemand nur wegen eines Rechtecks noch
/// `rustgame-core` in seine `Cargo.toml` schreiben muss.
///
/// `pygame/sprite.py` tut dasselbe (`from pygame.rect import Rect`);
/// deshalb steht `Rect` auch in `dir(pygame.sprite)`.
pub use rustgame_core::Rect;

/// Version der Vorlage, gegen die geprueft wurde.
pub const PYGAME_VERSION: &str = "2.5.7";
