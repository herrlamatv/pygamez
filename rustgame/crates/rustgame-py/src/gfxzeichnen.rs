//! gfxzeichnen.rs
//! ==============
//! `pygame.gfxdraw` - die Grundformen aus SDL_gfx.
//!
//! **Nicht dasselbe wie `pygame.draw`**, und das ist der springende
//! Punkt: hinter `draw` steht `draw.c`, hinter `gfxdraw` steht
//! `SDL_gfxPrimitives.c`. Ein Kreis mit demselben Mittelpunkt und
//! demselben Radius deckt in beiden verschiedene Pixel. Wer `gfxdraw`
//! auf `draw` abbildet, baut eine Funktion, die zwar laeuft und immer
//! knapp daneben zeichnet.
//!
//! Alle Aufrufe sind **rein positionell** (`/` am Ende der Signatur) -
//! gemessen an pygames eigenen Signaturen:
//!
//! ```text
//! pixel(surface, x, y, color, /)
//! circle(surface, x, y, r, color, /)
//! aapolygon(surface, points, color, /)
//! ```

use pyo3::prelude::*;
use pyo3::types::PyAny;

use rustgame_core::gfx;

use crate::flaeche::PySurface;
use crate::umwandlung::farbe_aus;

/// Eine Punktliste aus einer beliebigen Folge.
fn punkte(o: &Bound<'_, PyAny>) -> PyResult<Vec<(i32, i32)>> {
    let mut aus = Vec::new();
    for e in o.try_iter()? {
        let e = e?;
        let p = crate::umwandlung::versatz_aus(&e)?;
        aus.push(p);
    }
    Ok(aus)
}

macro_rules! zeichnen {
    ($name:ident, $py:literal, $kern:path, ($($arg:ident : $typ:ty),*)) => {
        #[pyfunction]
        // **Der Name, den Python sieht, steht im Makroaufruf.** Ohne
        // `#[pyo3(name = ..)]` traegt die Funktion ihren **Rust**-Namen in
        // `__name__` - `gfx_aacircle` statt `aacircle` -, und der steht
        // dann in jeder Fehlermeldung ueber ein Schluesselwort und in
        // jedem Rueckverfolgungsauszug. Gefunden hat das die Probe
        // `Funktionsnamen`; vorher trugen **258 von 390** Funktionen den
        // falschen Namen.
        #[pyo3(name = $py)]
        #[pyo3(signature = (surface, $($arg,)* color, /))]
        /// Eine Grundform aus SDL_gfx - siehe den Modulkopf.
        pub fn $name(
            surface: &PySurface,
            $($arg: $typ,)*
            color: &Bound<'_, PyAny>,
        ) -> PyResult<()> {
            let c = farbe_aus(color)?;
            let mut s = surface.leihen_mut()?;
            $kern(&mut s, $($arg,)* c);
            Ok(())
        }
    };
}

zeichnen!(gfx_pixel, "pixel", gfx::pixel, (x: i32, y: i32));
zeichnen!(gfx_hline, "hline", gfx::hline, (x1: i32, x2: i32, y: i32));
zeichnen!(gfx_vline, "vline", gfx::vline, (x: i32, y1: i32, y2: i32));
zeichnen!(gfx_line, "line", gfx::line, (x1: i32, y1: i32, x2: i32, y2: i32));
zeichnen!(gfx_circle, "circle", gfx::circle, (x: i32, y: i32, r: i32));
zeichnen!(gfx_aacircle, "aacircle", gfx::aacircle, (x: i32, y: i32, r: i32));
zeichnen!(gfx_filled_circle, "filled_circle", gfx::filled_circle, (x: i32, y: i32, r: i32));
zeichnen!(gfx_ellipse, "ellipse", gfx::ellipse, (x: i32, y: i32, rx: i32, ry: i32));
zeichnen!(gfx_aaellipse, "aaellipse", gfx::aaellipse, (x: i32, y: i32, rx: i32, ry: i32));
zeichnen!(
    gfx_filled_ellipse,
    "filled_ellipse",
    gfx::filled_ellipse,
    (x: i32, y: i32, rx: i32, ry: i32)
);
zeichnen!(
    gfx_arc,
    "arc",
    gfx::arc,
    (x: i32, y: i32, r: i32, start_angle: i32, stop_angle: i32)
);

/// `gfxdraw.rectangle(surface, rect, color)` - **ein Rechteck**, keine
/// vier Zahlen. Gemessen an der Signatur.
#[pyfunction]
#[pyo3(signature = (surface, rect, color, /))]
#[pyo3(name = "rectangle")]
pub fn gfx_rectangle(
    surface: &PySurface,
    rect: &Bound<'_, PyAny>,
    color: &Bound<'_, PyAny>,
) -> PyResult<()> {
    let r = crate::umwandlung::rect_aus(rect)?;
    let c = farbe_aus(color)?;
    let mut s = surface.leihen_mut()?;
    gfx::rectangle(&mut s, r.x, r.y, r.x + r.w - 1, r.y + r.h - 1, c);
    Ok(())
}

/// `gfxdraw.box(surface, rect, color)` - gefuellt.
#[pyfunction]
#[pyo3(signature = (surface, rect, color, /))]
#[pyo3(name = "box")]
pub fn gfx_box(
    surface: &PySurface,
    rect: &Bound<'_, PyAny>,
    color: &Bound<'_, PyAny>,
) -> PyResult<()> {
    let r = crate::umwandlung::rect_aus(rect)?;
    let c = farbe_aus(color)?;
    let mut s = surface.leihen_mut()?;
    gfx::box_(&mut s, r.x, r.y, r.x + r.w - 1, r.y + r.h - 1, c);
    Ok(())
}

/// Ein Kreisausschnitt als Umriss.
#[pyfunction]
#[pyo3(signature = (surface, x, y, r, start_angle, stop_angle, color, /))]
#[allow(clippy::too_many_arguments)]
#[pyo3(name = "pie")]
pub fn gfx_pie(
    surface: &PySurface,
    x: i32,
    y: i32,
    r: i32,
    start_angle: i32,
    stop_angle: i32,
    color: &Bound<'_, PyAny>,
) -> PyResult<()> {
    let c = farbe_aus(color)?;
    let mut s = surface.leihen_mut()?;
    gfx::pie(&mut s, x, y, r, start_angle, stop_angle, c, false);
    Ok(())
}

macro_rules! dreieck {
    ($name:ident, $py:literal, $wie:expr) => {
        #[pyfunction]
        // Der Python-Name steht im Aufruf - siehe `zeichnen!`.
        #[pyo3(name = $py)]
        #[pyo3(signature = (surface, x1, y1, x2, y2, x3, y3, color, /))]
        #[allow(clippy::too_many_arguments)]
        /// Eine Grundform aus SDL_gfx - siehe den Modulkopf.
        pub fn $name(
            surface: &PySurface,
            x1: i32,
            y1: i32,
            x2: i32,
            y2: i32,
            x3: i32,
            y3: i32,
            color: &Bound<'_, PyAny>,
        ) -> PyResult<()> {
            let c = farbe_aus(color)?;
            let p = [(x1, y1), (x2, y2), (x3, y3)];
            let mut s = surface.leihen_mut()?;
            $wie(&mut s, &p, c);
            Ok(())
        }
    };
}

dreieck!(gfx_trigon, "trigon", gfx::polygon);
dreieck!(gfx_aatrigon, "aatrigon", gfx::aapolygon);
dreieck!(gfx_filled_trigon, "filled_trigon", gfx::filled_polygon);

macro_rules! vieleck {
    ($name:ident, $py:literal, $wie:expr) => {
        #[pyfunction]
        // Der Python-Name steht im Aufruf - siehe `zeichnen!`.
        #[pyo3(name = $py)]
        #[pyo3(signature = (surface, points, color, /))]
        /// Eine Grundform aus SDL_gfx - siehe den Modulkopf.
        pub fn $name(
            surface: &PySurface,
            points: &Bound<'_, PyAny>,
            color: &Bound<'_, PyAny>,
        ) -> PyResult<()> {
            let p = punkte(points)?;
            let c = farbe_aus(color)?;
            let mut s = surface.leihen_mut()?;
            $wie(&mut s, &p, c);
            Ok(())
        }
    };
}

vieleck!(gfx_polygon, "polygon", gfx::polygon);
vieleck!(gfx_aapolygon, "aapolygon", gfx::aapolygon);
vieleck!(gfx_filled_polygon, "filled_polygon", gfx::filled_polygon);

/// Eine Bezierkurve - `bezier(surface, points, steps, color)`.
#[pyfunction]
#[pyo3(signature = (surface, points, steps, color, /))]
#[pyo3(name = "bezier")]
pub fn gfx_bezier(
    surface: &PySurface,
    points: &Bound<'_, PyAny>,
    steps: i32,
    color: &Bound<'_, PyAny>,
) -> PyResult<()> {
    let p = punkte(points)?;
    let c = farbe_aus(color)?;
    let mut s = surface.leihen_mut()?;
    gfx::bezier(&mut s, &p, steps, c);
    Ok(())
}

/// Ein Vieleck, gefuellt mit einer anderen Flaeche.
///
/// `tx`/`ty` verschieben die Vorlage unter dem Vieleck. Gezeichnet wird
/// zeilenweise wie beim gefuellten Vieleck, nur dass die Farbe aus der
/// Vorlage kommt - und die **wiederholt sich**, wenn das Vieleck groesser
/// ist als sie.
#[pyfunction]
#[pyo3(signature = (surface, points, texture, tx, ty, /))]
#[pyo3(name = "textured_polygon")]
pub fn gfx_textured_polygon(
    surface: &PySurface,
    points: &Bound<'_, PyAny>,
    texture: &PySurface,
    tx: i32,
    ty: i32,
) -> PyResult<()> {
    let p = punkte(points)?;
    if p.len() < 3 {
        return Ok(());
    }
    let vorlage = texture.leihen()?;
    let (tw, th) = vorlage.get_size();
    if tw == 0 || th == 0 {
        return Ok(());
    }
    // **Eine Vorlage mit Alphakanal geht nicht auf eine 8-Bit-Flaeche.**
    // Gemessen:
    //
    // ```text
    // Ziel 8 Bit, Vorlage SRCALPHA 32  ValueError: Per-byte alpha texture unsupported for 8 bit surfaces
    // Ziel 8 Bit, Vorlage 32 ohne Alpha  geht
    // Ziel 8 Bit, Vorlage 8 Bit          geht
    // ```
    //
    // Es liegt also an der **Vorlage**, nicht am Ziel allein - und die
    // Pruefung steht vor dem ersten Pixel, nicht mittendrin.
    {
        let ziel = surface.leihen()?;
        if ziel.get_bitsize() == 8 && vorlage.has_alpha() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Per-byte alpha texture unsupported for 8 bit surfaces",
            ));
        }
    }
    // Erst die abzudeckenden Zeilen sammeln, dann zeichnen - sonst
    // waeren Vorlage und Ziel zugleich ausgeliehen, und bei
    // `textured_polygon(s, p, s, 0, 0)` ist es dieselbe Flaeche.
    let mut streifen: Vec<(i32, i32, i32)> = Vec::new();
    let miny = p.iter().map(|q| q.1).min().unwrap();
    let maxy = p.iter().map(|q| q.1).max().unwrap();
    let n = p.len();
    for y in miny..=maxy {
        let mut schnitte: Vec<i32> = Vec::new();
        for i in 0..n {
            let (i1, i2) = if i == 0 { (n - 1, 0) } else { (i - 1, i) };
            let (mut y1, mut y2) = (p[i1].1, p[i2].1);
            let (x1, x2);
            if y1 < y2 {
                x1 = p[i1].0;
                x2 = p[i2].0;
            } else if y1 > y2 {
                y2 = p[i1].1;
                y1 = p[i2].1;
                x2 = p[i1].0;
                x1 = p[i2].0;
            } else {
                continue;
            }
            if ((y >= y1) && (y < y2)) || ((y == maxy) && (y > y1) && (y <= y2)) {
                schnitte.push(((65536 * (y - y1)) / (y2 - y1)) * (x2 - x1) + (65536 * x1));
            }
        }
        schnitte.sort_unstable();
        let mut i = 0;
        while i + 1 < schnitte.len() {
            let xa = schnitte[i] + 1;
            let xa = (xa >> 16) + ((xa & 32768) >> 15);
            let xb = schnitte[i + 1] - 1;
            let xb = (xb >> 16) + ((xb & 32768) >> 15);
            streifen.push((xa, xb, y));
            i += 2;
        }
    }
    let mut farben: Vec<(i32, i32, rustgame_core::color::Color)> = Vec::new();
    for (xa, xb, y) in streifen {
        for x in xa..=xb {
            let qx = (x - tx).rem_euclid(tw as i32);
            let qy = (y - ty).rem_euclid(th as i32);
            farben.push((x, y, vorlage.get_at((qx, qy))));
        }
    }
    drop(vorlage);
    let mut s = surface.leihen_mut()?;
    for (x, y, c) in farben {
        gfx::pixel(&mut s, x, y, c);
    }
    Ok(())
}
