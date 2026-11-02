//! kamera.rs
//! =========
//! Die Geraeteseite von `pygame.camera` - die Huelle um
//! [`rustgame_display::kamera`].
//!
//! ## Warum so wenig hier steht
//!
//! `pygame.camera` ist bei pygame zur Haelfte Python (`camera.py`) und zur
//! Haelfte C (`_camera`). Die Teilung ist sinnvoll, und sie ist hier
//! dieselbe: `colorspace` rechnet Farbraeume um, `AbstractCamera` legt die
//! Form fest, `Camera` haelt die Regler - das ist alles Rechnerei und
//! steht in `python/camera.py`. Hierher gehoert nur, was ein Geraet
//! anfassen muss.
//!
//! Die Klasse heisst darum `_Kamera` mit Unterstrich: sie ist nicht
//! `pygame.camera.Camera`, sondern das, was die benutzt.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use rustgame_core::surface::Surface;
use rustgame_display::kamera;

use crate::flaeche::PySurface;
use crate::umwandlung::PygameError;

/// Aus einem Kamerafehler die Ausnahme, die pygame dafuer wirft.
///
/// **Zwei Sorten, nicht eine.** Gemessen wirft pygame bei einem
/// unbekannten Namen `ValueError: Couldn't find a camera with that name`
/// und bei allem Uebrigen `pygame.error`. Wer alles zu `pygame.error`
/// macht, bricht `try: ... except ValueError:` in fremdem Code.
fn als_ausnahme(f: kamera::Fehler) -> PyErr {
    match f {
        kamera::Fehler::UnbekannterName => PyValueError::new_err(f.to_string()),
        _ => PygameError::new_err(f.to_string()),
    }
}

/// Welche Wege es zu einer Kamera gibt.
#[pyfunction]
pub fn kamera_rueckwaende() -> Vec<String> {
    kamera::rueckwaende()
}

/// Die Namen der gefundenen Kameras - leer, wenn keine da ist.
#[pyfunction]
pub fn kamera_geraete() -> Vec<String> {
    kamera::geraete().into_iter().map(|g| g.name).collect()
}

/// Eine offene Kamera.
#[pyclass(name = "_Kamera", module = "rustgame_py.camera", unsendable)]
pub struct PyKamera {
    inner: kamera::Kamera,
}

#[pymethods]
impl PyKamera {
    #[new]
    #[pyo3(signature = (name, w=640, h=480))]
    fn new(name: &str, w: u32, h: u32) -> PyResult<PyKamera> {
        Ok(PyKamera {
            inner: kamera::Kamera::neu(name, w, h).map_err(als_ausnahme)?,
        })
    }

    /// Vor dem Start der Wunsch, danach die Wirklichkeit - siehe
    /// [`rustgame_display::kamera::Kamera::groesse`].
    fn groesse(&self) -> (u32, u32) {
        self.inner.groesse()
    }

    fn starten(&mut self) -> PyResult<()> {
        self.inner.starten().map_err(als_ausnahme)
    }

    fn stoppen(&mut self) {
        self.inner.stoppen();
    }

    #[getter]
    fn laeuft(&self) -> bool {
        self.inner.laeuft()
    }

    #[getter]
    fn hflip(&self) -> bool {
        self.inner.hflip
    }

    #[setter]
    fn set_hflip(&mut self, an: bool) {
        self.inner.hflip = an;
    }

    #[getter]
    fn vflip(&self) -> bool {
        self.inner.vflip
    }

    #[setter]
    fn set_vflip(&mut self, an: bool) {
        self.inner.vflip = an;
    }

    #[getter]
    fn helligkeit(&self) -> i32 {
        self.inner.helligkeit()
    }

    fn helligkeit_setzen(&mut self, wert: i32) -> bool {
        self.inner.helligkeit_setzen(wert)
    }

    /// Ein Bild als Flaeche - **32 Bit**, ohne Alphakanal.
    ///
    /// ## Warum nicht `image::frombytes`
    ///
    /// Das Geraet liefert drei Byte je Pixel, und `frombytes` kann die
    /// direkt nehmen. Dann kommt aber eine **24-Bit**-Flaeche heraus, und
    /// gemessen gibt pygames `get_image()` eine mit 32:
    ///
    /// ```text
    /// pygame            (320, 240), 32 Bit, Masken (ff0000, ff00, ff, 0)
    /// frombytes "RGB"   (320, 240), 24 Bit, Masken (ff0000, ff00, ff, 0)
    /// ```
    ///
    /// **Die Masken waren gleich, die Tiefe nicht** - genau die Sorte
    /// Abweichung, die eine Probe findet und ein Blick nicht.
    ///
    /// Auch `"RGBX"` hilft nicht: `frombytes` nimmt fuer alles ohne
    /// Alphakanal 24 Bit, und das ist dort richtig - `image.frombytes`
    /// gibt bei pygame ebenfalls 24. Die Kamera ist der andere Fall, also
    /// wird die Flaeche hier selbst gebaut: `Surface::new` ist XRGB8888.
    fn bild(&mut self) -> PyResult<PySurface> {
        let b = self.inner.bild().map_err(als_ausnahme)?;
        let (w, h) = (b.w as usize, b.h as usize);
        if b.rgb.len() < w * h * 3 {
            return Err(PygameError::new_err("camera frame too short"));
        }
        let mut s = Surface::new((b.w, b.h));
        s.mit_pixeln(|puffer, zeilenbreite, (ox, oy)| {
            for y in 0..h {
                let q = y * w * 3;
                let z = (oy + y) * zeilenbreite + ox;
                for x in 0..w {
                    let p = q + x * 3;
                    puffer[z + x] = 0xff00_0000
                        | u32::from(b.rgb[p]) << 16
                        | u32::from(b.rgb[p + 1]) << 8
                        | u32::from(b.rgb[p + 2]);
                }
            }
        });
        Ok(PySurface::neu(s))
    }

    /// Die rohen Bytes des Geraets - `None`, solange nichts laeuft.
    fn roh<'p>(&mut self, py: Python<'p>) -> Option<Bound<'p, PyBytes>> {
        self.inner.roh().map(|b| PyBytes::new(py, &b))
    }
}
