//! rustgame-core
//! =============
//! Der Kern von **rustgame** - einer Nachbildung von pygame-ce 2.5.7 in Rust.
//!
//! Hier steckt alles, was ohne Fenster, ohne SDL und ohne Betriebssystem
//! auskommt: Flaechen, Rechtecke, Farben, das Zeichnen und die Mischarten.
//! Dadurch laeuft die gesamte Testsuite kopflos und schnell - genau wie die
//! pygame-Referenz, gegen die geprueft wird.
//!
//! ## Warum das genau wie pygame rechnet
//!
//! Die Mischformel wurde nicht geraten, sondern **gemessen**:
//! `rustgame/tools/verify_alpha.py` hat alle 16.777.216 Kombinationen von
//! (Ziel, Quelle, Alpha) durch echtes pygame geschickt und gegen mehrere
//! Kandidaten geprueft. Die Lehrbuchformel liegt in 8,1 Millionen Faellen
//! daneben, SDLs eigenes Makro in 3,3 Millionen. Richtig ist:
//!
//! ```text
//! out = (src * (alpha + 1) + dst * (256 - alpha)) >> 8
//! ```
//!
//! Siehe [`blend_formula`] - die Datei wird von dem Skript erzeugt.
//!
//! ## Aufbau
//!
//! | Modul | wofuer |
//! |---|---|
//! | [`surface`] | `pygame.Surface`: Pixelpuffer, blit, fill, clip, subsurface |
//! | [`rect`] | `pygame.Rect`: alle Anker, Kollisionen, Schnitt/Vereinigung |
//! | [`color`] | `pygame.Color` und alles, was als Farbe durchgeht |
//! | [`blend`] | die 17 `BLEND_*`-Mischarten |
//! | [`blend_formula`] | die gemessene Grundformel (erzeugt) |

// **Kein `unsafe` in dieser Kiste, und das prueft der Uebersetzer.**
//
// Hier stand der Satz lange nur in der Beschreibung - und eine Behauptung,
// die niemand nachhaelt, veraltet still. Siehe
// `rustgame-py/src/puffer.rs`: dort liegt das `unsafe` des Projekts, und
// nur dort ist der Lint ausdruecklich abgeschaltet.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod blend;
pub mod blend_formula;
pub mod color;
pub mod draw;
pub mod formate;
mod gifsdl;
pub mod gfx;
pub mod frect;
pub mod image;
pub mod rect;
pub mod surface;
pub mod time;
pub mod transform;

pub use color::{Color, IntoColor};
pub use frect::FRect;
pub use rect::Rect;
pub use surface::{Surface, SRCALPHA};

/// Version der Vorlage, gegen die geprueft wird.
pub const PYGAME_VERSION: &str = "2.5.7";
