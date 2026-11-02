//! pufferzeiger.rs
//! ===============
//! `pygame.bufferproxy` - der `BufferProxy`.
//!
//! ## Warum es die Klasse ueberhaupt gibt
//!
//! Sie stand auf der Ausschlussliste ("kein Puffer-Protokoll"), und das
//! war eine halbe Entscheidung: **`surface_test.py` laedt gar nicht ohne
//! sie.** Zeile 21 der Datei schreibt
//!
//! ```python
//! from pygame.bufferproxy import BufferProxy
//! ```
//!
//! auf Modulebene. Ein fehlendes Untermodul ist dort kein uebersprungener
//! Test, sondern ein `AttributeError` beim Laden - und damit sind **alle
//! 141** Tests der Datei nicht rot, sondern unsichtbar. Der Ausschluss hat
//! also nicht eine Grenze dokumentiert, sondern eine Messung verhindert.
//!
//! ## Drei Sorten Puffer, eine Klasse
//!
//! `BufferProxy` ist in pygame nicht nur der Zeiger auf eine Flaeche,
//! sondern ein **allgemeiner Ausgeber**. Er nimmt dreierlei:
//!
//! | Aufruf | woher die Beschreibung kommt |
//! |---|---|
//! | `Surface.get_buffer()` / `get_view(w)` | aus der Flaeche |
//! | `BufferProxy({...})` | aus einem Woerterbuch mit `shape`, `typestr`, `data`, `strides` |
//! | `BufferProxy(objekt)` | aus `__array_struct__`, `__array_interface__` oder dem Puffer-Protokoll |
//!
//! Und er gibt dreierlei wieder her: `memoryview(bp)` ueber das
//! Puffer-Protokoll, `bp.__array_struct__` als C-Kasten in einer Kapsel,
//! `bp.__array_interface__` als Woerterbuch.
//!
//! Das rohe Handwerk dazu steht in [`crate::puffer`] - der einzigen Datei
//! des Arbeitsbereichs mit `unsafe`. Hier steht nur, **was** die Sicht
//! beschreibt.
//!
//! ## `before` und `after`
//!
//! Zwei Rueckrufe, die pygame vorsieht und die man sich nicht ausdenkt:
//! `before(parent)` laeuft **einmal**, wenn zum ersten Mal jemand den
//! Puffer haben will; `after(parent)` laeuft, wenn der Zeiger selbst
//! stirbt. Gemessen an `bufferproxy_test.test_before` und `test_after`:
//! zweimal `__array_interface__` lesen ruft `before` genau **einmal**, und
//! `after` kommt erst nach `gc.collect()`.
//!
//! Dafuer steht unten ein `Drop` - die Stelle, an der Rust Python anfassen
//! muss, waehrend Python schon abraeumt.
//!
//! ## Die Bytereihenfolge ist Teil der Auskunft
//!
//! Gemessen: `pygame.get_sdl_byteorder()` gibt hier **1234**
//! (`SDL_LIL_ENDIAN`). Ein Pixelwort liegt also mit dem niedrigsten Byte
//! zuerst, und genau so wird hier gepackt. Auf einer Maschine mit
//! umgekehrter Reihenfolge waere `raw` byteweise gedreht - deshalb steht
//! die Reihenfolge nicht als Zahl im Quelltext, sondern kommt aus
//! [`u32::to_le_bytes`], das den Namen mitfuehrt.

use pyo3::exceptions::{PyIndexError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyTuple};

use crate::flaeche::PySurface;

/// Die Bytereihenfolge, die SDL meldet: **1234** fuer niedrigstes Byte
/// zuerst, 4321 fuer das Gegenteil.
///
/// Das sind SDLs eigene Zahlen (`SDL_LIL_ENDIAN` / `SDL_BIG_ENDIAN`), nicht
/// erfundene Kennungen - `surface_test.py` vergleicht damit und rechnet
/// Maskenwerte um.
pub const LIL_ENDIAN: u32 = 1234;
/// `pygame.BIG_ENDIAN` - die Zahl, die pygame dafuer nennt.
pub const BIG_ENDIAN: u32 = 4321;

/// `pygame.get_sdl_byteorder()`.
#[pyfunction]
pub fn get_sdl_byteorder() -> u32 {
    if cfg!(target_endian = "little") {
        LIL_ENDIAN
    } else {
        BIG_ENDIAN
    }
}

/// `pygame.bufferproxy.BufferProxy`.
///
/// Die vier Felder sind an echtem pygame abgelesen:
/// `dir(s.get_buffer())` gibt genau `length`, `parent`, `raw`, `write`,
/// und `repr` ist `<BufferProxy(24)>` - die Zahl ist die Laenge, nicht die
/// Groesse der Flaeche.
/// Woher ein Zeiger seine Beschreibung nimmt.
enum Quelle {
    /// Aus einer Flaeche, mit dem Wort aus `get_view(...)`.
    Flaeche {
        flaeche: Py<PyAny>,
        wort: Option<String>,
    },
    // `Fest(Sicht)` - eine beim Anlegen fertig gelesene Beschreibung -
    // stand hier, bis auch das Woerterbuch spaet gelesen wurde. Es war
    // der letzte, der den Fall gebaut hat; danach war er tot.
    /// Ein **Woerterbuch**, das noch nicht gelesen worden ist.
    ///
    /// Derselbe Fall wie [`Quelle::Fremd`], nur mit Angaben statt einem
    /// Objekt - und aus demselben gemessenen Grund: pygames
    /// `BufferProxy({...})` nimmt **jedes** Woerterbuch an, auch ein
    /// leeres, und jeder Einwand kommt erst beim ersten Gebrauch. Alle
    /// sieben Fehlertexte wurden so gemessen:
    ///
    /// ```text
    /// BufferProxy({})            Erzeuger: ok
    /// .length                    ValueError: required 'shape' item is missing
    /// ```
    ///
    /// Hier las der Erzeuger sofort, also kam der Einwand eine Stufe zu
    /// frueh. Gelesen wird so lange neu, bis es **einmal** gelingt -
    /// auch das gemessen: ein nachtraeglich gefuelltes Woerterbuch wird
    /// angenommen, eine Aenderung danach nicht mehr.
    Woerter(Py<PyDict>),
    /// Ein fremdes Objekt, das **noch nicht gefragt worden ist**.
    ///
    /// Das spaete Fragen ist der ganze Sinn dieses Falls: pygames
    /// `BufferProxy(objekt)` fragt beim Anlegen **nichts**. Gemessen an
    /// `base_test.test_PgObject_GetBuffer_array_interface`, das genau
    /// darauf baut:
    ///
    /// ```text
    /// o = Exporter2((2, 4), "u", 4)
    /// v = BufferProxy(o)
    /// self.assertRaises(NoDictError, o.is_dict_alive)   # noch ungefragt
    /// length = v.length                                 # jetzt gefragt
    /// self.assertTrue(o.is_dict_alive())
    /// ```
    ///
    /// Und `test_PgObject_GetBuffer_exception` von der anderen Seite:
    /// `BufferProxy(1)` **geht durch**, und erst `bp.length` gibt den
    /// `ValueError`. Wer beim Anlegen fragt, kann beides nicht.
    Fremd(Py<PyAny>),
}

#[pyclass(
    name = "BufferProxy",
    module = "rustgame_py.bufferproxy",
    subclass,
    dict,
    weakref,
    unsendable
)]
/// `pygame.BufferProxy` - ein Fenster auf fremden Speicher.
pub struct PyBufferProxy {
    quelle: Quelle,
    /// Wem der Puffer gehoert. `BufferProxy.parent` gibt genau dieses
    /// Objekt zurueck - **dasselbe**, nicht eine Kopie.
    eltern: Py<PyAny>,
    /// `before(parent)` - laeuft einmal, bevor der Puffer herausgeht.
    vorher: Option<Py<PyAny>>,
    /// `after(parent)` - laeuft, wenn dieser Zeiger stirbt.
    nachher: Option<Py<PyAny>>,
    /// Ist `before` schon gelaufen?
    geweckt: std::cell::Cell<bool>,
    /// Die Antwort auf `Quelle::Fremd`, sobald sie einmal geholt ist.
    ///
    /// Ein `RefCell`, weil `length` und `__array_interface__` nur `&self`
    /// haben - und trotzdem merken muessen, dass sie gefragt haben.
    gemerkt: std::cell::RefCell<Option<crate::puffer::Sicht>>,
}

impl Drop for PyBufferProxy {
    /// Ruft `after(parent)`, wenn es eines gibt.
    ///
    /// **Python anfassen, waehrend Python abraeumt** - das ist die Stelle,
    /// an der es sein muss. `Python::attach` ist dabei kein Wagnis: dieser
    /// `Drop` laeuft im Abbauschritt des Objekts, und da haelt CPython
    /// seine Sperre schon. Ein Fehler aus dem Rueckruf wird verworfen; in
    /// einem Destruktor gibt es niemanden, dem man ihn geben koennte, und
    /// pygame verschluckt ihn ebenso.
    fn drop(&mut self) {
        let Some(f) = self.nachher.take() else {
            return;
        };
        Python::attach(|py| {
            let _ = f.call1(py, (self.eltern.clone_ref(py),));
        });
    }
}

impl PyBufferProxy {
    /// Aus einer Flaeche - der Weg, den `get_buffer` und `get_view` gehen.
    pub fn fuer_flaeche(eltern: Py<PyAny>, sicht: Option<String>) -> PyBufferProxy {
        PyBufferProxy {
            quelle: Quelle::Flaeche {
                flaeche: Python::attach(|py| eltern.clone_ref(py)),
                wort: sicht,
            },
            eltern,
            vorher: None,
            nachher: None,
            geweckt: std::cell::Cell::new(false),
            gemerkt: std::cell::RefCell::new(None),
        }
    }

    /// Das Wort aus `get_view(...)`, falls es eines gab.
    fn wort(&self) -> Option<&str> {
        match &self.quelle {
            Quelle::Flaeche { wort, .. } => wort.as_deref(),
            _ => None,
        }
    }

    /// Der erste Zugriff: die Flaeche sperren und `before` rufen.
    ///
    /// **Ein Zeiger auf eine Flaeche sperrt sie - aber nicht immer
    /// sofort.** Gemessen an echtem pygame, und der Unterschied zwischen
    /// den beiden Wegen ist der Witz:
    ///
    /// ```text
    /// v = s.get_buffer()                          s.get_locked()  True
    /// w = s.get_view("2")                                         False
    /// n = w.length                                                True
    /// x = s.get_view("2"); x.__array_interface__                   True
    /// y = s.get_view("2"); memoryview(y)                           True
    /// jeweils nach `= None; gc.collect()`                          False
    /// ```
    ///
    /// `get_buffer()` sperrt also **beim Anlegen**, `get_view(...)` erst,
    /// wenn jemand die Laenge, die Feld-Schnittstelle oder den Puffer
    /// haben will. Wer beide gleich behandelt, faellt an einer der beiden
    /// Zeilen - `surface_test.test_get_view` prueft die zweite,
    /// `test_get_buffer` die erste.
    ///
    /// Geloest wird die Sperre von niemandem: sie haengt an einem `Weak`
    /// auf den Zettel, den dieser Zeiger haelt, und verschwindet mit ihm.
    /// Und `ich` ist dieser Zeiger selbst - **er** wird die Sperre.
    ///
    /// Gemessen gibt `sf.get_locks()` nach dem Wecken genau ihn heraus:
    ///
    /// ```text
    /// w = s.get_view("2"); s.get_locks()   ()
    /// n = w.length;        s.get_locks()   (<BufferProxy(100)>,)
    /// b = s.get_buffer();  s.get_locks()   (<BufferProxy(100)>, <BufferProxy(100)>)
    /// ```
    ///
    /// Also kein Merkzettel und kein Stellvertreter, sondern das Objekt.
    /// Deshalb nehmen `length`, `raw`, `write`, `__array_interface__`,
    /// `__array_struct__` und `__getbuffer__` ihr `slf` als `Bound` und
    /// nicht als `&self`: eine Methode mit `&self` kennt ihr eigenes
    /// Python-Objekt nicht, und ohne das gibt es hier nichts einzutragen.
    ///
    /// Geloest wird die Sperre von niemandem - sie ist ein weicher Verweis
    /// und verschwindet mit dem Zeiger. Siehe [`crate::flaeche::Sperre`].
    pub(crate) fn wecken(&self, ich: &Bound<'_, PyAny>) -> PyResult<()> {
        if self.geweckt.get() {
            return Ok(());
        }
        self.geweckt.set(true);
        let py = ich.py();
        if let Quelle::Flaeche { flaeche, .. } = &self.quelle {
            if let Ok(f) = flaeche.bind(py).downcast::<PySurface>() {
                f.borrow().sperre_eintragen(py, ich);
            }
        }
        if let Some(f) = &self.vorher {
            f.call1(py, (self.eltern.clone_ref(py),))?;
        }
        Ok(())
    }

    /// Den Puffer **holen** - und erst danach `before` rufen.
    ///
    /// Die Reihenfolge ist gemessen, und zwar an einem Woerterbuch, das
    /// sich nicht lesen laesst (`strides: None`) und an demselben mit
    /// einem `before`, das mitschreibt:
    ///
    /// ```text
    ///                        pygame            rustgame (vorher)
    /// .length  gut           before 1          before 1
    /// .length  kaputt        before 0, Fehler  before 1, Fehler
    /// ```
    ///
    /// pygames `_get_buffer` holt zuerst den Puffer und ruft `before`
    /// **nur, wenn das gelungen ist** - hier lief es umgekehrt, und ein
    /// Rueckruf feuerte fuer einen Puffer, den es nie gab. Bei einer
    /// Flaeche faellt das nicht auf, weil dort nichts zu lesen ist;
    /// dasselbe Holen steht trotzdem davor, damit die Reihenfolge nicht
    /// von der Quelle abhaengt.
    ///
    /// Der Rueckruf feuert weiterhin nur **einmal** - das haelt
    /// [`PyBufferProxy::wecken`] mit `geweckt` fest.
    fn geholt(slf: &Bound<'_, Self>) -> PyResult<()> {
        let py = slf.py();
        {
            let this = slf.borrow();
            if matches!(this.quelle, Quelle::Flaeche { .. }) {
                // Nur fragen, ob sie sich ausleihen laesst; die Leihe
                // endet hier wieder, damit `wecken` die Flaeche anfassen
                // kann.
                let f = this.flaeche(py)?;
                let g = f.borrow();
                drop(g.leihen()?);
            } else {
                this.sicht(py)?;
            }
        }
        slf.borrow().wecken(slf.as_any())
    }

    /// Die Beschreibung dieser Sicht.
    ///
    /// Bei einer festen Quelle steht sie schon da; bei einer Flaeche wird
    /// sie hier gerechnet - und zwar aus dem **echten** Speicher, nicht aus
    /// einer Kopie. Das ist der Grund, warum `memoryview(bp)[0] = 1` die
    /// Flaeche aendert und nicht ein Abbild davon.
    ///
    /// ## Die Grenze, die hier einmal stand
    ///
    /// An dieser Stelle gab es fuer jede Flaeche unter 32 Bit einen
    /// `BufferError`: rustgame legte **jede** Flaeche als ARGB8888 ab, und
    /// ein Puffer darauf haette Bytes gezeigt, die so nicht im Speicher
    /// stehen. Das war die richtige Antwort auf eine falsche Ablage.
    ///
    /// Die Ablage ist jetzt die richtige - `PixelBuf` haelt Bytes im
    /// gemeldeten Format -, und damit ist der Fehler weggefallen. Gemessen
    /// an einer Flaeche 10 x 6, `get_view("2")`:
    ///
    /// ```text
    /// Tiefe   shape     strides    typestr
    ///  8      (10, 6)   (1, 12)    |u1
    /// 16      (10, 6)   (2, 20)    <u2
    /// 24      (10, 6)   (3, 32)    <V3
    /// 32      (10, 6)   (4, 40)    <u4
    /// ```
    ///
    /// Die Schrittweiten sagen es: einen Schritt nach rechts sind `bpp`
    /// Bytes, einen nach unten `pitch` - und `pitch` ist auf vier
    /// aufgerundet, weshalb bei 8 Bit und Breite 10 nicht 10, sondern 12
    /// dasteht.
    fn sicht(&self, py: Python<'_>) -> PyResult<crate::puffer::Sicht> {
        use crate::puffer::Sicht;
        if let Quelle::Fremd(o) = &self.quelle {
            if let Some(s) = self.gemerkt.borrow().as_ref() {
                return Ok(s.clone());
            }
            let s = crate::puffer::hol_sicht(o.bind(py))?;
            *self.gemerkt.borrow_mut() = Some(s.clone());
            return Ok(s);
        }
        if let Quelle::Woerter(d) = &self.quelle {
            if let Some(s) = self.gemerkt.borrow().as_ref() {
                return Ok(s.clone());
            }
            let s = crate::puffer::dict_sicht(d.bind(py))?;
            *self.gemerkt.borrow_mut() = Some(s.clone());
            return Ok(s);
        }
        let f = self.flaeche(py)?;
        let g = f.borrow();
        let s = g.leihen()?;
        let format = s.get_format();
        let bs = format.bytesize().max(1) as usize;
        let (w, h) = s.get_size();
        let (wi, hi) = (w as isize, h as isize);
        let pitch = s.get_pitch() as isize;
        let adresse = s.pixel_zeiger();
        let solide = pitch == wi * bs as isize;
        // Der PEP-3118-Buchstabe zur Bytebreite. Gemessen an pygames
        // eigener Tabelle in `pixelarray_test`:
        // `{8: "B", 16: "=H", 24: "3x", 32: "=I"}`.
        let wort_format = match bs {
            1 => "B",
            2 => "=H",
            3 => "3x",
            _ => "=I",
        };
        let voll = |ndim: Vec<isize>, strides: Vec<isize>, item: usize, fmt: &str| Sicht {
            adresse,
            laenge: ndim.iter().product::<isize>() as usize * item,
            itemsize: item,
            format: crate::puffer::fmt(fmt),
            shape: ndim,
            strides,
            schreibbar: true,
            solide,
        };
        Ok(match self.wort() {
            // Der **ganze** Speicher, samt Auffuellung am Zeilenende.
            None => Sicht {
                adresse,
                laenge: (pitch * hi) as usize,
                itemsize: 1,
                format: crate::puffer::fmt("B"),
                shape: vec![pitch * hi],
                strides: vec![1],
                schreibbar: true,
                solide: true,
            },
            // `'0'` ist derselbe Speicher, byteweise gezaehlt.
            Some("0") => Sicht {
                adresse,
                laenge: (wi * hi) as usize * bs,
                itemsize: 1,
                format: crate::puffer::fmt("B"),
                shape: vec![wi * hi * bs as isize],
                strides: vec![1],
                schreibbar: true,
                solide,
            },
            Some("1") => voll(vec![wi * hi], vec![bs as isize], bs, wort_format),
            Some("3") => Sicht {
                adresse: adresse + rot_versatz(format),
                laenge: (wi * hi) as usize * 3,
                itemsize: 1,
                format: crate::puffer::fmt("B"),
                shape: vec![wi, hi, 3],
                strides: vec![bs as isize, pitch, farbschritt(format)],
                schreibbar: true,
                solide: false,
            },
            Some(k @ ("r" | "R" | "g" | "G" | "b" | "B" | "a" | "A")) => {
                let maske = match k.to_ascii_lowercase().as_str() {
                    "r" => format.masks.0,
                    "g" => format.masks.1,
                    "b" => format.masks.2,
                    _ => format.masks.3,
                };
                Sicht {
                    adresse: adresse + (maske.trailing_zeros() / 8) as usize,
                    laenge: (wi * hi) as usize,
                    itemsize: 1,
                    format: crate::puffer::fmt("B"),
                    shape: vec![wi, hi],
                    strides: vec![bs as isize, pitch],
                    schreibbar: true,
                    solide: false,
                }
            }
            // `'2'` und alles sonst, was `get_view` durchgelassen hat.
            _ => voll(vec![wi, hi], vec![bs as isize, pitch], bs, wort_format),
        })
    }

    /// Die Flaeche hinter `parent` - fuer andere Kisten des Moduls.
    ///
    /// `freetype.Font.render_raw_to` schreibt in eine Sicht, die aus
    /// `Surface.get_view(...)` kommt; ohne diesen Weg muesste es die
    /// Flaeche noch einmal aus `parent` herausdowncasten.
    pub(crate) fn flaeche_oeffentlich<'p>(
        &self,
        py: Python<'p>,
    ) -> PyResult<Bound<'p, PySurface>> {
        self.flaeche(py)
    }

    /// Die Flaeche hinter `parent`, ausgeliehen.
    fn flaeche<'p>(&self, py: Python<'p>) -> PyResult<Bound<'p, PySurface>> {
        let o = match &self.quelle {
            Quelle::Flaeche { flaeche, .. } => flaeche,
            _ => &self.eltern,
        };
        o.bind(py)
            .downcast::<PySurface>()
            .cloned()
            .map_err(|_| PyValueError::new_err("Buffer parent is not a Surface"))
    }

    /// Der Rohinhalt: Zeile fuer Zeile die abgebildeten Pixelwoerter,
    /// jedes in `bytesize` Bytes, jede Zeile auf `pitch` aufgefuellt.
    ///
    /// Aufgefuellt wird mit Nullen. pygame laesst dort stehen, was im
    /// Speicher lag - bei einer frisch angelegten Flaeche ebenfalls Null,
    /// bei einer Teilflaeche dagegen die Nachbarpixel. Das ist die eine
    /// Stelle, an der eine Kopie nicht dasselbe sehen kann wie ein
    /// Zeiger; sie steht hier, weil `pitch == breite * bytesize` ist,
    /// solange niemand eigene Masken setzt.
    fn bytes_lesen(&self, py: Python<'_>) -> PyResult<Vec<u8>> {
        // Kommt die Sicht nicht von einer Flaeche, dann gibt es echten
        // Speicher dahinter und nichts umzurechnen.
        if !matches!(self.quelle, Quelle::Flaeche { .. }) {
            return crate::puffer::bytes_lesen(&self.sicht(py)?);
        }
        let f = self.flaeche(py)?;
        // Zwei Bindungen, nicht eine: `f.borrow()` gibt einen Schein, und
        // `leihen()` leiht aus **ihm** aus. In einer Zeile wird der Schein
        // am Semikolon weggeworfen, waehrend die Ausleihe noch lebt.
        let g = f.borrow();
        let s = g.leihen()?;
        let (w, h) = s.get_size();
        let format = s.get_format();
        let bs = format.bytesize() as usize;

        // Die Kanalsichten geben **ein Byte je Pixel** - den Kanal, um den
        // es geht. Welches Byte das im Wort ist, sagt die Maske: ihre
        // Nullen von rechts sind die Verschiebung.
        // **Gepackt, nicht roh.** Ueberall hier steht
        // `map_rgb_gepackt(get_at(..))` und nicht `get_at_mapped(..)`.
        //
        // `get_at_mapped` gibt den Wert, wie er im Speicher liegt, und der
        // ist hier immer ARGB8888: der Kern rechnet nur in diesem einen
        // Format und quantisiert erst beim Ablegen. Was `raw` zeigen muss,
        // ist der Wert im **Format der Flaeche** - bei 8 Bit der
        // Tafelindex, bei 16 Bit das RGB565-Wort.
        //
        // Bei 24 und 32 Bit ist das **derselbe** Wert, und darum war die
        // erste Fassung falsch und sah richtig aus: sie nahm
        // `get_at_mapped` und schnitt mit `to_le_bytes()[..bytesize]` die
        // niedrigsten Bytes heraus. Bei 32 Bit stimmt das. Bei 8 Bit kam
        // aus dem roten ARGB-Wort `0xFFFF0000` das Byte `0x00` statt des
        // Tafeleintrags `0x60`, bei 16 Bit `0x0000` statt `0xF800`. Eine
        // schwarze Flaeche haette auf beiden Seiten gleich ausgesehen.
        let kanal = match self.wort() {
            Some("r") | Some("R") => Some(format.masks.0),
            Some("g") | Some("G") => Some(format.masks.1),
            Some("b") | Some("B") => Some(format.masks.2),
            Some("a") | Some("A") => Some(format.masks.3),
            _ => None,
        };
        if let Some(maske) = kanal {
            let schub = maske.trailing_zeros();
            let mut out = Vec::with_capacity(w as usize * h as usize);
            for y in 0..h as i32 {
                for x in 0..w as i32 {
                    out.push(((s.gepackt_an((x, y)) & maske) >> schub) as u8);
                }
            }
            return Ok(out);
        }

        // `'3'` gibt drei Bytes je Pixel - Rot, Gruen, Blau in der
        // Reihenfolge des Formats, ohne den vierten Kanal.
        if self.wort() == Some("3") {
            let mut out = Vec::with_capacity(3 * w as usize * h as usize);
            for y in 0..h as i32 {
                for x in 0..w as i32 {
                    let v = s.gepackt_an((x, y));
                    out.extend_from_slice(&v.to_le_bytes()[..3]);
                }
            }
            return Ok(out);
        }

        // Ohne Sicht der **ganze** Speicher, samt Auffuellung am
        // Zeilenende; mit `'0'`, `'1'` oder `'2'` dieselben Pixel **ohne**
        // sie. Bei 32 Bit ist das dasselbe, bei 8 Bit und Breite 3 nicht:
        // acht Bytes gegen sechs.
        let pitch = if self.wort().is_none() {
            s.get_pitch() as usize
        } else {
            w as usize * bs
        };
        let mut out = vec![0u8; pitch * h as usize];
        for y in 0..h as i32 {
            for x in 0..w as i32 {
                let v = s.gepackt_an((x, y));
                let ziel = y as usize * pitch + x as usize * bs;
                out[ziel..ziel + bs].copy_from_slice(&v.to_le_bytes()[..bs]);
            }
        }
        Ok(out)
    }
}

#[pymethods]
impl PyBufferProxy {
    /// `BufferProxy(woher)` - ein Woerterbuch oder ein Objekt.
    ///
    /// Gemessen an `bufferproxy_test`: das Woerterbuch traegt `shape`,
    /// `typestr`, `data` und `strides`, dazu wahlweise `parent`, `before`
    /// und `after`.
    ///
    /// Ein **Objekt** wird hier noch **nicht** gefragt - es wandert als
    /// `Quelle::Fremd` beiseite, und erst wer die Laenge, den Puffer oder
    /// die Feld-Schnittstelle will, loest es auf. Warum das so sein muss,
    /// steht bei [`Quelle::Fremd`].
    #[new]
    #[pyo3(signature = (woher=None))]
    fn new(py: Python<'_>, woher: Option<&Bound<'_, PyAny>>) -> PyResult<PyBufferProxy> {
        let Some(o) = woher else {
            return Err(PyTypeError::new_err(
                // **Die Zahl gehoert dazu.** Gemessen:
                // `BufferProxy() takes exactly 1 argument (0 given)`.
                // Hier stand "exactly one argument" ohne die Klammer -
                // naeher als PyO3s Satz, aber nicht pygames.
                "BufferProxy() takes exactly 1 argument (0 given)",
            ));
        };
        if let Ok(d) = o.downcast::<PyDict>() {
            return Self::aus_woerterbuch_intern(py, d);
        }
        Ok(PyBufferProxy {
            quelle: Quelle::Fremd(o.clone().unbind()),
            eltern: o.clone().unbind(),
            vorher: None,
            nachher: None,
            geweckt: std::cell::Cell::new(false),
            gemerkt: std::cell::RefCell::new(None),
        })
    }

    /// Was der Zyklensammler sehen muss.
    ///
    /// **Ohne das ist dieser Typ fuer `gc` unsichtbar**, und
    /// `bufferproxy_test.test_gc` prueft beides: dass ein Zeiger in
    /// `gc.get_objects()` auftaucht, und dass ein Zeiger, der sich selbst
    /// in einem eigenen Feld haelt, wieder eingesammelt wird
    /// (`v.some_attribute = v`). Ein PyO3-Typ bekommt `tp_traverse` erst,
    /// wenn er eines hat - und dann bekommt er auch die Flagge
    /// `Py_TPFLAGS_HAVE_GC`.
    fn __traverse__(&self, besuch: pyo3::PyVisit<'_>) -> Result<(), pyo3::PyTraverseError> {
        besuch.call(&self.eltern)?;
        if let Quelle::Fremd(o) = &self.quelle {
            besuch.call(o)?;
        }
        if let Some(f) = &self.vorher {
            besuch.call(f)?;
        }
        if let Some(f) = &self.nachher {
            besuch.call(f)?;
        }
        Ok(())
    }

    /// Gegenstueck: alles loslassen, was im Kreis zeigen koennte.
    ///
    /// Dass dabei auch `after` weggeht, ist **Absicht**. `__clear__` laeuft
    /// nur bei Muell, der im Kreis haengt, und dort ruft pygame den
    /// Rueckruf ebenfalls nicht mehr - Python raeumt zu diesem Zeitpunkt
    /// schon ab, und ein Aufruf koennte auf halb abgebaute Objekte treffen.
    /// Beim gewoehnlichen Sterben laeuft `tp_clear` nicht, und dann kommt
    /// `Drop` wie bisher dazu.
    fn __clear__(&mut self) {
        Python::attach(|py| {
            self.eltern = py.None();
        });
        self.vorher = None;
        self.nachher = None;
        *self.gemerkt.borrow_mut() = None;
    }

    /// `bp.__array_interface__` - die Beschreibung als Woerterbuch.
    ///
    /// **Genau fuenf Eintraege**, und in dieser Reihenfolge nachgemessen:
    /// `version`, `shape`, `typestr`, `data`, `strides`.
    ///
    /// Und alle fuenf kommen aus der **gelesenen** Beschreibung, auch bei
    /// einem Woerterbuch. Hier wurden dessen Angaben vorher
    /// durchgereicht, mit der Begruendung, sie muessten unveraendert
    /// wiederkommen - was fuer `bufferproxy_test.view_keywords` zufaellig
    /// dasselbe ergab, weil dort alle vier Schluessel dastehen. Gemessen
    /// ist es nicht dasselbe:
    ///
    /// ```text
    /// gegeben                       pygame gibt
    /// shape (3,2) typestr <u4       strides (8, 4)     <- gerechnet
    /// shape (4,)  typestr |V3       typestr <V3        <- umbenannt
    /// shape [4]   (eine Liste)      ValueError expected a tuple of ints
    /// ```
    ///
    /// `strides` fehlte also immer, wenn niemand sie hereingab, und ein
    /// `typestr` kam zurueck, das pygame so nie ausgibt. Das Rechnen ist
    /// auch verlustfrei: dreizehn Formate hin und zurueck gemessen,
    /// `|u1 |i1 <u2 >u2 <i2 <u4 >u4 <i4 <f4 <f8` kommen Zeichen fuer
    /// Zeichen wieder, und `|V3` wird bei pygame genauso zu `<V3`.
    #[getter]
    fn __array_interface__<'p>(slf: &Bound<'p, Self>) -> PyResult<Bound<'p, PyDict>> {
        let py = slf.py();
        Self::geholt(slf)?;
        let this = slf.borrow();
        let s = this.sicht(py)?;
        let aus = PyDict::new(py);
        aus.set_item("version", 3)?;
        aus.set_item("shape", PyTuple::new(py, s.shape.iter().map(|v| *v as usize))?)?;
        aus.set_item("typestr", s.typestr())?;
        aus.set_item("data", (s.adresse, !s.schreibbar))?;
        aus.set_item(
            "strides",
            PyTuple::new(py, s.strides.iter().copied())?,
        )?;
        Ok(aus)
    }

    /// `bp.__array_struct__` - dieselbe Beschreibung als C-Kasten.
    #[getter]
    fn __array_struct__(slf: &Bound<'_, Self>) -> PyResult<Py<PyAny>> {
        let py = slf.py();
        Self::geholt(slf)?;
        let sicht = slf.borrow().sicht(py)?;
        crate::puffer::feldschnittstelle(py, &sicht)
    }

    /// Der Schlitz `bf_getbuffer` - `memoryview(bp)` und alles darauf.
    ///
    /// # Sicherheit
    ///
    /// Reicht `view` unveraendert an [`crate::puffer::fuellen`] weiter; die
    /// Zusagen stehen dort.
    // Reicht nur durch - siehe `#![deny(unsafe_code)]` in `lib.rs`.
    #[allow(unsafe_code)]
    unsafe fn __getbuffer__(
        slf: PyRef<'_, Self>,
        view: *mut pyo3::ffi::Py_buffer,
        flags: std::ffi::c_int,
    ) -> PyResult<()> {
        let py = slf.py();
        let eigner = slf.into_pyobject(py)?;
        Self::geholt(&eigner)?;
        let sicht = eigner.borrow().sicht(py)?;
        unsafe { crate::puffer::fuellen(&sicht, view, flags, eigner.as_any()) }
    }

    /// Gegenstueck dazu.
    ///
    /// # Sicherheit
    ///
    /// Siehe [`crate::puffer::freigeben`].
    #[allow(unsafe_code)]
    unsafe fn __releasebuffer__(&self, view: *mut pyo3::ffi::Py_buffer) {
        unsafe { crate::puffer::freigeben(view) }
    }

    /// Wie viele Bytes der Puffer hat - und das haengt an der **Sicht**.
    ///
    /// Gemessen an `Surface((3, 2))` in vier Tiefen; die Zahlen trennen
    /// drei Rechnungen, die man sonst fuer eine halten koennte:
    ///
    /// | Sicht | Laenge | Rechnung |
    /// |---|---|---|
    /// | `get_buffer()` | 8 bei 8 Bit, 24 bei 32 Bit | `pitch * hoehe` |
    /// | `'0'`, `'1'`, `'2'` | 6 bei 8 Bit, 24 bei 32 Bit | `bytesize * breite * hoehe` |
    /// | `'3'` | 18 | `3 * breite * hoehe` |
    /// | `'r'`…`'A'` | 6 | `breite * hoehe` |
    ///
    /// Die erste Zeile ist die aufschlussreiche: bei 8 Bit und Breite 3 ist
    /// `pitch` auf vier Bytes aufgerundet, also **8** statt 6. Bei 32 Bit
    /// fallen beide Rechnungen auf 24 zusammen - wer nur dort misst, haelt
    /// sie fuer dieselbe.
    #[getter]
    fn length(slf: &Bound<'_, Self>) -> PyResult<usize> {
        Self::geholt(slf)?;
        slf.borrow().laenge_intern(slf.py())
    }


    /// **Dasselbe** Objekt, das den Puffer haelt.
    ///
    /// Und es **holt den Puffer**, obwohl es ihn nicht braucht: pygames
    /// `bufferproxy_get_parent` gibt `view.obj` heraus, und dafuer muss
    /// die Sicht da sein. Gemessen an drei Quellen:
    ///
    /// ```text
    /// Woerterbuch, gut      None       before 1
    /// Woerterbuch, kaputt   ValueError expected a tuple of ints for 'strides'
    /// fremdes Objekt, gut   das Objekt und es wird dabei gefragt
    /// fremdes Objekt, kaputt ValueError .. does not export an array buffer
    /// ```
    ///
    /// Hier gab es die Eltern ohne jedes Lesen heraus - bei einem kaputten
    /// Woerterbuch also ein `None` statt eines Fehlers, und ein fremder
    /// Ausgeber blieb ungefragt.
    #[getter]
    fn parent(slf: &Bound<'_, Self>) -> PyResult<Py<PyAny>> {
        Self::geholt(slf)?;
        Ok(slf.borrow().eltern.clone_ref(slf.py()))
    }

    /// `bp.raw` - die Bytes, **wenn sie zusammenhaengen**.
    ///
    /// pygames `bufferproxy_get_raw` prueft `pgBuffer_IsContiguous(.., 'A')`
    /// und gibt sonst `ValueError: the bytes are not contiguous`. Das `A`
    /// heisst **C oder Fortran**, und das ist nicht dasselbe wie
    /// `memoryview.c_contiguous`:
    ///
    /// ```text
    /// get_view('0')  shape (48,)     strides (1,)        C         raw: ok
    /// get_view('1')  shape (12,)     strides (4,)        C         raw: ok
    /// get_view('2')  shape (4, 3)    strides (4, 16)     **F**     raw: ok
    /// get_view('3')  shape (4, 3, 3) strides (4, 16, -1) keines    ValueError
    /// get_view('r')  shape (4, 3)    strides (4, 16)     keines    ValueError
    /// ```
    ///
    /// Bei `'2'` ist `itemsize` 4, also sind `(4, 16)` genau die
    /// Fortran-Schritte; bei `'r'` ist es 1, und dieselben Zahlen sind
    /// dann keine. Hier gab `.raw` fuer alle fuenf Bytes heraus - fuer die
    /// Kanalsichten sogar **gesammelte**, die so im Speicher nirgends
    /// stehen. Gefunden hat das die Messung des Pufferprotokolls in der
    /// Breite (Runde 16), 13 Stellen.
    #[getter]
    fn raw<'p>(slf: &Bound<'p, Self>) -> PyResult<Bound<'p, PyBytes>> {
        let py = slf.py();
        Self::geholt(slf)?;
        let sicht = slf.borrow().sicht(py)?;
        if !(sicht.c_zusammen() || sicht.f_zusammen()) {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "the bytes are not contiguous",
            ));
        }
        let daten = slf.borrow().bytes_lesen(py)?;
        Ok(PyBytes::new(py, &daten))
    }

    /// `write(buffer, offset=0)` - schreibt Bytes in die Flaeche zurueck.
    ///
    /// pygame prueft dabei zwei Dinge, und beide haben ihren eigenen
    /// Fehler: ein Versatz ausserhalb des Puffers ist ein `IndexError`,
    /// zu viele Bytes ein `ValueError`. Der Wortlaut ist abgelesen.
    /// **Eine Zeichenkette ist hier erlaubt.** pygames
    /// `bufferproxy_write` liest das Argument mit `s#`, und das nimmt
    /// `str` als UTF-8 mit: gemessen geht `b.write('abc')` dort
    /// anstandslos durch, waehrend `b.write(1)`
    /// `TypeError: a bytes-like object is required, not 'int'` gibt. Mit
    /// `&[u8]` wies PyO3 die Zeichenkette ab.
    #[pyo3(signature = (buffer, offset=0))]
    fn write(slf: &Bound<'_, Self>, buffer: &Bound<'_, PyAny>, offset: isize) -> PyResult<()> {
        let eigen: Vec<u8>;
        let buffer: &[u8] = if let Ok(t) = buffer.extract::<&str>() {
            t.as_bytes()
        } else {
            eigen = buffer.extract::<Vec<u8>>().map_err(|_| {
                pyo3::exceptions::PyTypeError::new_err(format!(
                    "a bytes-like object is required, not '{}'",
                    buffer
                        .get_type()
                        .name()
                        .map(|n| n.to_string())
                        .unwrap_or_else(|_| "object".to_string())
                ))
            })?;
            &eigen
        };
        let py = slf.py();
        Self::geholt(slf)?;
        let this = slf.borrow();
        let self_ = &*this;
        if !matches!(self_.quelle, Quelle::Flaeche { .. }) {
            return crate::puffer::bytes_schreiben(&self_.sicht(py)?, offset, buffer);
        }
        let laenge = self_.laenge_intern(py)?;
        if buffer.len() > laenge {
            return Err(PyValueError::new_err("'buffer' object length is too large"));
        }
        if offset < 0 || offset as usize + buffer.len() > laenge {
            return Err(PyIndexError::new_err("'offset' is out of range"));
        }
        let offset = offset as usize;
        let f = self_.flaeche(py)?;
        let g = f.borrow();
        let mut s = g.leihen_mut()?;
        let (w, h) = s.get_size();
        let bs = s.get_format().bytesize() as usize;
        let pitch = s.get_pitch() as usize;

        // Erst den ganzen Puffer aufbauen, dann die beschriebenen Bytes
        // hineinlegen, dann alles zurueckschreiben. Umgekehrt - Pixel fuer
        // Pixel direkt - wuerde ein Schreibvorgang, der mitten in einem
        // Pixelwort anfaengt, die anderen Bytes dieses Worts verlieren.
        let mut roh = {
            let mut out = vec![0u8; pitch * h as usize];
            for y in 0..h as i32 {
                for x in 0..w as i32 {
                    let v = s.gepackt_an((x, y));
                    let z = y as usize * pitch + x as usize * bs;
                    out[z..z + bs].copy_from_slice(&v.to_le_bytes()[..bs]);
                }
            }
            out
        };
        roh[offset..offset + buffer.len()].copy_from_slice(buffer);

        for y in 0..h as i32 {
            for x in 0..w as i32 {
                let z = y as usize * pitch + x as usize * bs;
                let mut wort = [0u8; 4];
                wort[..bs].copy_from_slice(&roh[z..z + bs]);
                let v = u32::from_le_bytes(wort);
                let farbe = s.unmap_rgb_gepackt(v);
                s.set_at((x, y), farbe);
            }
        }
        Ok(())
    }

    // **Kein `__len__`.** Gemessen gibt `len(s.get_buffer())` bei pygame
    // einen `TypeError` - sein `BufferProxy` ist keine Folge, es hat nur
    // das Feld `length`. Hier stand ein `__len__`, das auf `length`
    // zeigte, und damit gab `len()` die Zahl 16, wo pygame meckert.
    // Gefunden von der Probe `Klassendoppel`.

    /// Auch das holt den Puffer - die Laenge steht ja darin.
    ///
    /// Gemessen am mitschreibenden `before`: `repr(bp)` laesst es bei
    /// pygame feuern. Hier lief nur `laenge_intern`, und das weckt
    /// niemanden.
    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        Self::geholt(slf)?;
        // Gemessen: `<BufferProxy(24)>` - die Laenge in Klammern.
        Ok(format!(
            "<BufferProxy({})>",
            slf.borrow().laenge_intern(slf.py())?
        ))
    }

    /// Das Wort aus `get_view(...)`, falls es eines gab.
    ///
    /// Kein pygame-Feld - es steht hier, damit `flaeche.rs` die Sicht
    /// nicht getrennt mitfuehren muss. Mit Unterstrich, damit es nicht
    /// wie eines aussieht.
    #[getter]
    fn _sicht(&self) -> Option<&str> {
        self.wort()
    }
}

/// Um wie viele Bytes das **rote** Byte im Pixelwort nach hinten sitzt.
fn rot_versatz(format: rustgame_core::surface::Format) -> usize {
    (format.masks.0.trailing_zeros() / 8) as usize
}

/// Ein Schritt von Rot nach Gruen - **plus oder minus eins**.
///
/// Bei den Standardmasken `(0x00FF0000, 0xFF00, 0xFF, 0xFF000000)` liegt
/// Rot im dritten Byte und Blau im ersten; die Farben laufen im Speicher
/// also **rueckwaerts**, und `get_view("3")` bekommt die Schrittweite -1.
/// Gemessen an `surface_test.test_array_interface_masks`, das genau solche
/// gedrehten Masken durchprobiert.
fn farbschritt(format: rustgame_core::surface::Format) -> isize {
    let rot = format.masks.0.trailing_zeros() as isize;
    let blau = format.masks.2.trailing_zeros() as isize;
    if blau > rot {
        1
    } else {
        -1
    }
}


/// Was **nicht** zur Schnittstelle gehoert.
///
/// Beide Methoden standen in `#[pymethods]` und hiessen `laenge` und
/// `aus_woerterbuch` - zwei **deutsche** Namen in
/// `dir(pygame.bufferproxy.BufferProxy)`, die pygame nicht hat. Ein
/// `#[pymethods]`-Block ist die Schnittstelle; wer dort eine
/// Hilfsrechnung ablegt, veroeffentlicht sie.
impl PyBufferProxy {
    /// Dieselbe Rechnung, ohne zu wecken - fuer `__repr__` und `write`.
    ///
    /// **Mit `_intern` im Namen, und das war einmal anders.** Die Methode
    /// stand in `#[pymethods]` und hiess `laenge` - damit war sie von
    /// Python aus zu sehen, und `dir(pygame.bufferproxy.BufferProxy)` hatte
    /// zwei **deutsche** Namen, die pygame nicht hat: `laenge` und
    /// `aus_woerterbuch`. Eine Hilfsrechnung in der Schnittstelle.
    ///
    /// Gefunden hat das die Probe `Klassennamen`, die die **rufbaren**
    /// Namen aller 65 Klassen in beide Richtungen vergleicht.
    /// `api_coverage.py` meldet nur Fehlendes - ein Ueberschuss fiel dort
    /// nie auf, und drei Messwerkzeuge haben ihn uebersehen.
    fn laenge_intern(&self, py: Python<'_>) -> PyResult<usize> {
        if !matches!(self.quelle, Quelle::Flaeche { .. }) {
            return Ok(self.sicht(py)?.laenge);
        }
        let f = self.flaeche(py)?;
        let g = f.borrow();
        let s = g.leihen()?;
        let (w, h) = (s.get_size().0 as usize, s.get_size().1 as usize);
        let bs = s.get_format().bytesize() as usize;
        Ok(match self.wort() {
            None => s.get_pitch() as usize * h,
            Some("3") => 3 * w * h,
            Some("r") | Some("R") | Some("g") | Some("G") | Some("b") | Some("B")
            | Some("a") | Some("A") => w * h,
            // '0', '1', '2' - und alles sonst, was `get_view` durchgelassen
            // hat.
            _ => bs * w * h,
        })
    }

    /// Baut einen Zeiger aus einem Woerterbuch mit Feld-Angaben.
    ///
    /// Die Angaben selbst liest [`crate::puffer::dict_sicht`] - dort
    /// stehen auch pygames vierzehn Fehlertexte. Hier bleiben nur die drei
    /// Schluessel, die **nicht** zur Beschreibung gehoeren: `parent`,
    /// `before` und `after`.
    /// Mit `_intern` im Namen - siehe `laenge_intern`. Kein
    /// `#[staticmethod]` mehr: von Python aus gibt es den Namen bei pygame
    /// nicht.
    fn aus_woerterbuch_intern(
        py: Python<'_>,
        d: &Bound<'_, PyDict>,
    ) -> PyResult<PyBufferProxy> {
        // **Nichts lesen.** Siehe [`Quelle::Woerter`]: pygame nimmt jedes
        // Woerterbuch an und beschwert sich erst beim Gebrauch.
        Ok(PyBufferProxy {
            quelle: Quelle::Woerter(d.clone().unbind()),
            eltern: match d.get_item("parent")? {
                Some(v) => v.unbind(),
                None => py.None(),
            },
            vorher: d
                .get_item("before")?
                .filter(|v| !v.is_none())
                .map(|v| v.unbind()),
            nachher: d
                .get_item("after")?
                .filter(|v| !v.is_none())
                .map(|v| v.unbind()),
            geweckt: std::cell::Cell::new(false),
            gemerkt: std::cell::RefCell::new(None),
        })
    }
}