//! charts.rs
//! =========
//! Die Diagramme (`charts.py`) liegen seit dem Mines-/Plinko-Update in der
//! Engine (`pgz_engine::charts`), weil auch Spiele sie zeichnen (Chip-
//! Statistik von Lama Mines / Lama Plinko). Dieses Modul reicht sie nur an
//! die Bildschirme weiter (`charts::bars(...)` im Fortschritt).

pub use pgz_engine::charts::*;
