//! pgz-engine
//! ==========
//! Die Engine von PyGameZ, gebaut auf **rustgame** - dem pixelgleichen
//! Nachbau von pygame-ce.
//!
//! | Modul | Gegenstueck in Python |
//! |---|---|
//! | [`py`] | Pythons Rechenregeln und `random` (MT19937, bitgleich) |
//! | [`pg`] | `pygame` selbst: Surface, Rect, draw, transform, font |
//! | [`ui`] | `ui.py` |
//! | [`charts`] | `charts.py` (Diagramme: Fortschritt, Chip-Statistik) |
//! | [`audio`] | `audio.py` |
//! | [`game`] | `game_base.py` |
//!
//! Die Spiele sind Zeile fuer Zeile aus `games/*.py` uebertragen und rufen
//! nur diese Module - nie rustgame direkt. So bleibt jede Stelle, an der
//! Rust anders rechnet als Python, an **einem** Ort.

#![warn(missing_docs)]

pub mod achv;
pub mod audio;
pub mod charts;
pub mod clip;
pub mod filepick;
pub mod game;
pub mod pg;
pub mod py;
pub mod shell;
pub mod ui;

pub use game::{Game, GameBase, GameInfo, InputEvent, Kind};

/// Alles, was ein Spiel braucht, mit einem `use`.
pub mod prelude {
    pub use crate::game::{Game, GameBase, GameInfo, InputEvent, Kind};
    pub use crate::pg::{
        self, draw, rect, rgb, rgba, transform, Col, Color, Font, IntoRect, Num, Pt, Rect, RectExt,
        SurfExt, Surface, BLEND_ADD, BLEND_MAX, BLEND_MIN, BLEND_MULT, BLEND_RGBA_ADD, BLEND_RGBA_MAX,
        BLEND_RGBA_MIN, BLEND_RGBA_MULT, BLEND_RGBA_SUB, BLEND_RGB_ADD, BLEND_RGB_MAX, BLEND_RGB_MIN,
        BLEND_RGB_MULT, BLEND_RGB_SUB, BLEND_SUB, SRCALPHA,
    };
    pub use crate::py;
    pub use crate::ui::{self, mix};
    pub use crate::{achv, audio, shell};
    pub use pgz_core::i18n::{t, tf};
}
