//! draw
//! ====
//! Die Zeichenfunktionen - Gegenstueck zu `pygame.draw`.
//!
//! **Fast alle hier umgesetzten Funktionen rastern binaer**: ein Pixel
//! gehoert ganz zur Form oder gar nicht. Das ist kein Sparprogramm,
//! sondern gemessenes pygame-Verhalten. Die Ausnahme sind `aaline`,
//! `aalines` und `aacircle` - sie glaetten Kanten und erzeugen dabei
//! Zwischenwerte; ihre Mischformel steht in [`aaline`](fn@aaline) und ist **keine**
//! der vier aus [`crate::blend_formula`]. Dadurch ist in beiden Faellen
//! **pixelgenaue** Uebereinstimmung erreichbar, und `tests/golden.rs`
//! weist sie nach.
//!
//! Jede Funktion gibt das tatsaechlich beschriebene Rechteck zurueck, auf
//! das Clip-Rechteck der Flaeche beschnitten - wie in pygame.

pub mod aacircle;
pub mod aaline;
pub mod arc;
pub mod circle;
pub mod ellipse;
pub mod flood_fill;
pub mod line;
pub mod polygon;
pub mod rect;

pub use aacircle::{aacircle, aacircle_quadranten, Quadranten};
pub use aaline::{aaline, aalines};
pub use arc::arc;
pub use circle::{circle, circle_quadranten};
pub use ellipse::ellipse;
pub use flood_fill::{flood_fill, flood_fill_muster};
pub use line::{line, lines};
pub use polygon::polygon;
pub use rect::{rect, rect_corners, rect_radius};
