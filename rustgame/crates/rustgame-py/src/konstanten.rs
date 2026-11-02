//! konstanten.rs
//! =============
//! Traegt die Konstanten ins Modul ein - `pygame.K_SPACE`, `pygame.QUIT`,
//! `pygame.SRCALPHA` und die anderen 443.
//!
//! Die Namensliste steht erzeugt in `crate::konstanten_liste`; hier
//! stehen nur die beiden Dinge, die die Liste nicht wissen kann.
//!
//! ## 1. `FULLSCREEN` ist in pygame **negativ**
//!
//! ```text
//! pygame.FULLSCREEN   -2147483648
//! rustgame (als u32)   2147483648
//! ```
//!
//! Derselbe Bitwert, aber `flags == pygame.FULLSCREEN` waere falsch und
//! `flags | pygame.FULLSCREEN` gaebe eine andere Zahl. Jede Konstante mit
//! gesetztem oberstem Bit geht deshalb als `i32` hinueber. Es ist
//! gegenwaertig genau eine - die Regel steht trotzdem allgemein da, damit
//! die naechste nicht durchrutscht.
//!
//! ## 2. Die `BLEND_*`-Werte laufen auseinander
//!
//! `rustgame-core` nummeriert seine Mischarten selbst weiter, pygame
//! folgt SDL. Ab `BLEND_RGBA_MAX` stimmen die Zahlen nicht mehr ueberein:
//!
//! | Name | pygame-ce 2.5.7 | rustgame-core |
//! |---|---|---|
//! | `BLEND_RGBA_MIN` | 9 | 9 |
//! | `BLEND_RGBA_MAX` | **16** | 10 |
//! | `BLEND_PREMULTIPLIED` | **17** | 11 |
//! | `BLEND_ALPHA_SDL2` | **18** | 12 |
//!
//! Nach Python gehen die **pygame**-Zahlen - ein Programm, das `16`
//! hinschreibt, muss dasselbe tun wie unter pygame. Uebersetzt wird beim
//! Eintritt, in [`misch_art`], und zwar an den drei Stellen, an denen
//! `special_flags` hereinkommt: `fill`, `blit`, `blits`.

use pyo3::prelude::*;

use crate::konstanten_liste::{TEXTE, ZAHLEN};

/// Die Mischarten mit den Werten, die **pygame** benutzt.
const MISCHARTEN: &[(&str, u32)] = &[
    ("BLEND_ADD", 1),
    ("BLEND_SUB", 2),
    ("BLEND_MULT", 3),
    ("BLEND_MIN", 4),
    ("BLEND_MAX", 5),
    ("BLEND_RGB_ADD", 1),
    ("BLEND_RGB_SUB", 2),
    ("BLEND_RGB_MULT", 3),
    ("BLEND_RGB_MIN", 4),
    ("BLEND_RGB_MAX", 5),
    ("BLEND_RGBA_ADD", 6),
    ("BLEND_RGBA_SUB", 7),
    ("BLEND_RGBA_MULT", 8),
    ("BLEND_RGBA_MIN", 9),
    ("BLEND_RGBA_MAX", 16),
    ("BLEND_PREMULTIPLIED", 17),
    ("BLEND_ALPHA_SDL2", 18),
];

/// Uebersetzt `special_flags` von pygames Nummerierung in die des Kerns.
///
/// Siehe Modultext. Alles unter 16 ist in beiden dasselbe.
pub fn misch_art(flags: u32) -> u32 {
    match flags {
        16 => 10, // BLEND_RGBA_MAX
        17 => 11, // BLEND_PREMULTIPLIED
        18 => 12, // BLEND_ALPHA_SDL2
        andere => andere,
    }
}

/// Traegt alle Konstanten in ein Modul ein.
///
/// Wird zweimal gerufen: einmal fuer `rustgame_py` selbst und einmal fuer
/// `rustgame_py.locals` - `from pygame.locals import *` ist die
/// gebraeuchlichste Schreibweise ueberhaupt.
pub fn eintragen(m: &Bound<'_, PyModule>) -> PyResult<()> {
    for (name, wert) in ZAHLEN.iter().chain(MISCHARTEN.iter()) {
        if *wert & 0x8000_0000 != 0 {
            // Oberstes Bit gesetzt - pygame fuehrt das als negative Zahl.
            m.add(*name, *wert as i32)?;
        } else {
            m.add(*name, *wert)?;
        }
    }
    // Und die fuenf, die Zeichenketten sind - `NULL_VIDEODRIVER` und die
    // vier `SCRAP_*`. Gemessen: `pygame.NULL_VIDEODRIVER` ist `'dummy'`.
    for (name, wert) in TEXTE.iter() {
        m.add(*name, *wert)?;
    }
    Ok(())
}
