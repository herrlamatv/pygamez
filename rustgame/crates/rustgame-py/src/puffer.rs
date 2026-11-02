//! puffer.rs
//! =========
//! Das **Puffer-Protokoll** - und die einzige Stelle im ganzen
//! Arbeitsbereich mit `unsafe`.
//!
//! ## Warum es hier nicht ohne geht
//!
//! Alles andere in rustgame kommt ohne aus, und das war lange auch fuer
//! diese Stelle die Antwort: der Plan hat "kein C-API, kein
//! Puffer-Protokoll" von Anfang an als Grenze genannt. Die Grenze kostete
//! **zweiundfuenfzig Tests** in `bufferproxy`, `surface`, `base`,
//! `pixelcopy`, `pixelarray`, `color`, `mask` und `mixer` - mehr als alle
//! anderen offenen Punkte zusammen.
//!
//! Ein Puffer-Protokoll ohne rohen Zeiger gibt es nicht. CPython fragt
//! ueber den Schlitz `bf_getbuffer` nach einer Adresse, einer Laenge und
//! zwei Zahlenfeldern (`shape`, `strides`), und es behaelt sie, bis es den
//! Puffer wieder freigibt. Wer statt dessen kopiert, beantwortet eine
//! andere Frage: `memoryview(s.get_buffer())[0] = 1` soll die Flaeche
//! aendern, nicht eine Kopie.
//!
//! Also steht die **Arbeit** hier, hinter sieben Funktionen, und jede
//! davon ist unten begruendet.
//!
//! ## Nachgezaehlt: nicht eine Datei, sondern sechs
//!
//! Hier stand "in **einer** Datei ... wer nachsehen will, braucht genau
//! diese Datei zu lesen und keine andere". Nachgezaehlt liegt `unsafe` in
//! sechs Dateien: hier (25 Stellen) und in `farbe.rs`, `maske.rs`,
//! `pixelfeld.rs`, `pufferzeiger.rs` und `ton.rs` mit je vier. Diese vier
//! sind aber immer dieselben: die beiden Schlitze `__getbuffer__` und
//! `__releasebuffer__`, die PyO3 als `unsafe fn` vorschreibt, samt ihren
//! zwei Aufrufen hierher. **Sie tun selbst nichts.**
//!
//! Der zweite Halbsatz war noch schlechter: die sechs anderen Kisten
//! truegen `#![forbid(unsafe_code)]`. **Dieser Lint stand nirgends im
//! Quelltext.** Die Aussage war wahr und voellig unbewacht - eine
//! Behauptung, die niemand nachhaelt, veraltet still. Jetzt haelt sie der
//! Uebersetzer: `forbid` in den sechs Kisten, `deny` in `rustgame-py` mit
//! fuenf einzelnen Ausnahmen, und `allow` nur hier.
//!
//! ## Die drei Zusagen, die hier eingehalten werden muessen
//!
//! 1. **Die Adresse muss den Puffer ueberleben.** rustgame legt die Pixel
//!    einer Flaeche in einem `Vec<u32>` ab, dessen Laenge nach dem Anlegen
//!    nie mehr wechselt - es gibt keinen Weg, eine Flaeche zu vergroessern.
//!    Damit bleibt die Adresse gueltig, solange die Flaeche lebt, und
//!    genau so lange haelt der Puffer sie auch fest (`view.obj`).
//! 2. **Der Halter muss am Leben bleiben.** `fuellen` legt den Eigner in
//!    `view.obj` und erhoeht seinen Verweiszaehler; CPython senkt ihn beim
//!    Freigeben. Ohne das koennte Python die Flaeche einsammeln, waehrend
//!    ein `memoryview` noch auf ihre Pixel zeigt.
//! 3. **`shape` und `strides` gehoeren dem Ausgeber.** Sie werden hier auf
//!    dem Haufen angelegt und in `freigeben` wieder eingesammelt. Ein
//!    Zeiger auf etwas auf dem Stapel waere schon beim Verlassen der
//!    Funktion falsch.
//!
//! ## Was gemessen ist
//!
//! ```text
//! s = Surface((10, 6), 0, 32); a = s.get_view("2")
//! Importer(a, PyBUF_SIMPLE).ndim      0      shape/strides None
//! Importer(a, PyBUF_ND)               BufferError
//! Importer(a, PyBUF_STRIDES).strides  (4, 40)
//! Importer(a, PyBUF_RECORDS).format   "=I"
//! Importer(a, PyBUF_C_CONTIGUOUS)     BufferError
//! Importer(a, PyBUF_F_CONTIGUOUS)     geht - shape (10,6), strides (4,40)
//! ```
//!
//! Die dritte und die letzte Zeile zusammen sind der ganze Witz der
//! zweidimensionalen Sicht: sie ist **spaltenweise** zusammenhaengend,
//! weil `shape` `(Breite, Hoehe)` ist und nicht `(Hoehe, Breite)`. Wer sie
//! als C-Feld anmeldet, bekommt von numpy ein gedrehtes Bild.
//!
//! Und die zweite Zeile ist die, auf die man nicht kommt: `PyBUF_ND` allein
//! **scheitert**, obwohl `PyBUF_SIMPLE` durchgeht. `SIMPLE` fragt nach
//! einem Stueck Speicher, und das ist die Flaeche; `ND` fragt nach einem
//! Feld ohne Schrittweiten, und dafuer muessten die Zeilen luecklos
//! aneinanderliegen - tun sie im Sinne von `(Breite, Hoehe)` nicht.

#![allow(unsafe_code)]

use std::ffi::{c_int, c_void, CStr, CString};

use pyo3::ffi;
use pyo3::prelude::*;

// ---------------------------------------------------------------------------
//  Die Flaggen, mit denen ein Verbraucher fragt
// ---------------------------------------------------------------------------

// Die `PyBUF_*`-Zahlen aus CPythons `object.h`. Sie stehen hier, weil
// PyO3 sie nicht herausgibt - und abgetippt, weil sie Teil der
// Schnittstelle sind und sich nicht aendern.
/// Nur Adresse und Laenge - kein Format, keine Form.
pub const PY_BUF_SIMPLE: c_int = 0;
/// Der Aufrufer will schreiben.
pub const PY_BUF_WRITABLE: c_int = 0x0001;
/// Der Aufrufer will `format` wissen.
pub const PY_BUF_FORMAT: c_int = 0x0004;
/// Der Aufrufer will `shape` wissen.
pub const PY_BUF_ND: c_int = 0x0008;
/// Dazu `strides` - damit auch Luecken je Zeile.
pub const PY_BUF_STRIDES: c_int = 0x0010 | PY_BUF_ND;
/// Muss zeilenweise zusammenhaengen (C-Anordnung).
pub const PY_BUF_C_CONTIGUOUS: c_int = 0x0020 | PY_BUF_STRIDES;
/// Muss spaltenweise zusammenhaengen (Fortran-Anordnung).
pub const PY_BUF_F_CONTIGUOUS: c_int = 0x0040 | PY_BUF_STRIDES;
/// Eines von beiden genuegt.
pub const PY_BUF_ANY_CONTIGUOUS: c_int = 0x0080 | PY_BUF_STRIDES;

/// Das Vorzeichen, mit dem PEP 3118 "so wie diese Maschine" meint.
///
/// pygame schreibt dafuer `=` und nicht `<` - auch auf einer Maschine mit
/// niedrigstem Byte zuerst. Gemessen an `base_test`, das `"=h"` erwartet,
/// wo die Feld-Schnittstelle `"<i2"` sagte.
pub const BUF_MEINS: char = '=';

/// Und das Vorzeichen fuer "andersherum als diese Maschine".
pub const BUF_ANDERS: char = if cfg!(target_endian = "little") { '>' } else { '<' };

/// Dieselbe Frage fuer die Feld-Schnittstelle, die `<` und `>` benutzt.
pub const PAI_MEINS: char = if cfg!(target_endian = "little") { '<' } else { '>' };

/// Und ihr Gegenstueck.
pub const PAI_ANDERS: char = if cfg!(target_endian = "little") { '>' } else { '<' };

/// `PAI_*` - die Flaggen der Feld-Schnittstelle (`__array_struct__`).
pub const PAI_CONTIGUOUS: c_int = 0x01;
// Die Flaggen von `__array_interface__` / `__array_struct__`.
/// Spaltenweise angeordnet.
pub const PAI_FORTRAN: c_int = 0x02;
/// Die Adresse liegt auf einer passenden Grenze.
pub const PAI_ALIGNED: c_int = 0x100;
/// Die Bytereihenfolge ist die der Maschine.
pub const PAI_NOTSWAPPED: c_int = 0x200;
/// Es darf geschrieben werden.
pub const PAI_WRITEABLE: c_int = 0x400;

// ---------------------------------------------------------------------------
//  Wie eine Sicht beschrieben wird
// ---------------------------------------------------------------------------

/// Alles, was ein Ausgeber ueber seinen Speicher sagen muss.
///
/// Die Felder heissen wie die von `Py_buffer`, damit beim Vergleich mit
/// CPythons Vorschrift nichts umzurechnen ist. `shape` und `strides`
/// tragen hoechstens drei Achsen - mehr hat in pygame keine Sicht.
#[derive(Clone)]
pub struct Sicht {
    /// Das erste Byte.
    pub adresse: usize,
    /// Wie viele Bytes die Sicht umfasst: `Produkt(shape) * itemsize`.
    pub laenge: usize,
    /// Wie viele Bytes ein Eintrag hat.
    pub itemsize: usize,
    /// Der Formatbuchstabe nach PEP 3118 - `"=I"`, `"B"`, `">h"`.
    ///
    /// **Eine eigene Zeichenkette, keine geliehene.** Frueher stand hier
    /// ein `&'static CStr` aus einer Tabelle, und die Tabelle musste jedes
    /// Format kennen, das je durchkommt. Das ging so lange gut, wie nur
    /// rustgames eigene Sichten hier landeten; sobald ein **fremder**
    /// Ausgeber seinen Puffer hergibt, ist die Tabelle falsch. Drei Faelle
    /// haben es gezeigt:
    ///
    /// * `"1h"` ist nach PEP 3118 dasselbe wie `"h"`, aber ein Verbraucher
    ///   bekommt es **unveraendert** zurueck - `base_test.test_newbuf`
    ///   stellt beide neben `"=h"` in dieselbe Liste und vergleicht jedes
    ///   mit sich selbst.
    /// * `">h"` ist ein Format mit gedrehter Byteordnung. Eine Tabelle
    ///   ohne Vorzeichen kann es nicht ausdruecken.
    /// * `"hh"` ist **ungueltig**, und das muss ein `ValueError` werden -
    ///   wer es auf `"B"` abbildet, verschweigt den Fehler.
    ///
    /// Die Zeichenkette gehoert damit der `Sicht`; [`fuellen`] legt eine
    /// Kopie davon auf den Haufen, weil CPython den Zeiger erst liest,
    /// wenn dieser Aufruf schon zurueck ist.
    pub format: CString,
    /// Die Form - so viele Zahlen, wie es Achsen gibt.
    pub shape: Vec<ffi::Py_ssize_t>,
    /// Die Schrittweite je Achse, in Bytes.
    pub strides: Vec<ffi::Py_ssize_t>,
    /// Darf der Aufrufer schreiben?
    pub schreibbar: bool,
    /// Liegt der Bereich als **ein** Stueck im Speicher?
    ///
    /// Nicht dasselbe wie "C-zusammenhaengend": die zweidimensionale Sicht
    /// einer ganzen Flaeche ist ein Stueck Speicher, aber ihre
    /// Schrittweiten sind spaltenweise. Siehe den Modulkopf.
    pub solide: bool,
}

impl Sicht {
    /// Wie viele Achsen - die Laenge von [`Sicht::shape`].
    pub fn ndim(&self) -> usize {
        self.shape.len()
    }

    /// Liegen die Achsen in C-Reihenfolge luecklos hintereinander?
    pub fn c_zusammen(&self) -> bool {
        let mut erwartet = self.itemsize as ffi::Py_ssize_t;
        for i in (0..self.ndim()).rev() {
            if self.strides[i] != erwartet {
                return false;
            }
            erwartet *= self.shape[i];
        }
        true
    }

    /// Und in Fortran-Reihenfolge?
    pub fn f_zusammen(&self) -> bool {
        let mut erwartet = self.itemsize as ffi::Py_ssize_t;
        for i in 0..self.ndim() {
            if self.strides[i] != erwartet {
                return false;
            }
            erwartet *= self.shape[i];
        }
        true
    }

    /// Die Feld-Schnittstelle: `typestr` wie numpy ihn schreibt.
    ///
    /// PEP 3118 und numpy nennen dasselbe verschieden: `"=I"` dort ist
    /// `"<u4"` hier (auf einer Maschine mit niedrigstem Byte zuerst).
    pub fn typestr(&self) -> String {
        let f = self.format.to_string_lossy();
        // **Das Vorzeichen kommt aus dem Format, nicht aus der Maschine.**
        // `">h"` heisst gedreht, und dann steht im `typestr` auch `>`.
        let vor = match f.chars().next() {
            Some('<') => '<',
            Some('>') | Some('!') => '>',
            _ => PAI_MEINS,
        };
        let kern = f.trim_start_matches(['=', '<', '>', '!', '@']);
        let kern = kern.strip_prefix('1').filter(|r| !r.is_empty()).unwrap_or(kern);
        match kern {
            "B" => "|u1".to_string(),
            "b" => "|i1".to_string(),
            "H" => format!("{vor}u2"),
            "h" => format!("{vor}i2"),
            "I" | "L" => format!("{vor}u4"),
            "i" | "l" => format!("{vor}i4"),
            "Q" => format!("{vor}u8"),
            "q" => format!("{vor}i8"),
            "f" => format!("{vor}f4"),
            "d" => format!("{vor}f8"),
            // `3x` und Geschwister sind Fuellbytes ohne Bedeutung - numpy
            // nennt so etwas "void" mit der Byteanzahl.
            //
            // **Und auch die tragen ein Vorzeichen**, sobald sie breiter
            // als ein Byte sind. Gemessen gibt pygame fuer die Sicht auf
            // eine 24-Bit-Flaeche `<V3`, nicht `|V3`; der senkrechte
            // Strich gilt nach `_pg_as_arrayinter_byteorder` nur bei
            // `itemsize == 1` und bei den Zeichentypen `c s p b B`.
            _ if self.itemsize == 1 => format!("|V{}", self.itemsize),
            _ => format!("{vor}V{}", self.itemsize),
        }
    }
}

// ---------------------------------------------------------------------------
//  Der Schlitz `bf_getbuffer`
// ---------------------------------------------------------------------------

/// Was zwischen `fuellen` und `freigeben` auf dem Haufen liegt.
///
/// `shape` und `strides` muessen den Aufruf ueberleben - CPython liest sie
/// erst, wenn `getbuffer` schon zurueck ist. Deshalb ein `Box`, dessen
/// Adresse in `view.internal` wandert; `freigeben` holt ihn dort wieder ab.
struct Zahlenfelder {
    shape: Vec<ffi::Py_ssize_t>,
    strides: Vec<ffi::Py_ssize_t>,
    /// Die Formatzeichenkette, damit `view.format` auf etwas Lebendiges
    /// zeigt. Sie steht **hier** und nicht in der `Sicht`, weil die `Sicht`
    /// ein kurzlebiger Wert ist und der Puffer sie ueberlebt.
    format: CString,
}

/// Fuellt ein `Py_buffer` nach CPythons Vorschrift.
///
/// # Sicherheit
///
/// `view` muss auf ein gueltiges, beschreibbares `Py_buffer` zeigen -
/// CPython uebergibt genau das. `eigner` muss ein lebendes Python-Objekt
/// sein; sein Verweiszaehler wird hier erhoeht und von CPython beim
/// Freigeben wieder gesenkt.
///
/// # Safety
///
/// `view` muss auf eine beschreibbare `Py_buffer`-Struktur zeigen, die
/// CPython fuer diesen Aufruf mitgebracht hat, oder NULL sein (dann gibt es
/// einen Fehler statt eines Zugriffs).
///
/// Dieser Abschnitt hat lange gefehlt, und die Begruendung stand daneben im
/// Fliesstext - **Clippy hat ihn nie verlangt, weil Clippy diese Kiste nie
/// gesehen hat**: sie steht ausserhalb des Workspace, und
/// `cargo clippy --workspace` geht an ihr vorbei.
///
/// Ausserdem muss `sicht.adresse` so lange gueltig bleiben, wie der Puffer
/// offen ist. Fuer rustgames Flaechen gilt das, weil ihr Pixelspeicher nach
/// dem Anlegen nie mehr die Groesse wechselt; siehe Zusage 1 im Modulkopf.
pub unsafe fn fuellen(
    sicht: &Sicht,
    view: *mut ffi::Py_buffer,
    flags: c_int,
    eigner: &Bound<'_, PyAny>,
) -> PyResult<()> {
    if view.is_null() {
        return Err(crate::umwandlung::PygameBufferError::new_err(
            "Py_buffer is NULL",
        ));
    }
    if flags & PY_BUF_WRITABLE != 0 && !sicht.schreibbar {
        return Err(crate::umwandlung::PygameBufferError::new_err(
            "Object is not writable",
        ));
    }

    let will_nd = (flags & PY_BUF_ND) == PY_BUF_ND;
    let will_strides = (flags & PY_BUF_STRIDES) == PY_BUF_STRIDES;

    // **Wer keine Achsen will, will ein Stueck Speicher.** Und das gibt es
    // nur, wenn der Bereich luecklos ist - bei einer Teilflaeche also
    // nicht. Gemessen: dort scheitern `PyBUF_SIMPLE`, `PyBUF_FORMAT` und
    // `PyBUF_WRITABLE` alle drei mit `BufferError`.
    if !will_nd && !sicht.solide {
        return Err(crate::umwandlung::PygameBufferError::new_err(
            "A contiguous buffer is required",
        ));
    }
    // Achsen ohne Schrittweiten heissen: luecklos in C-Reihenfolge.
    if will_nd && !will_strides && !sicht.c_zusammen() {
        return Err(crate::umwandlung::PygameBufferError::new_err(
            "A C-contiguous buffer is required",
        ));
    }
    if will_strides {
        let (c, f) = (sicht.c_zusammen(), sicht.f_zusammen());
        let verlangt = |m: c_int| (flags & m) == m;
        if verlangt(PY_BUF_C_CONTIGUOUS) && !c {
            return Err(crate::umwandlung::PygameBufferError::new_err(
                "A C-contiguous buffer is required",
            ));
        }
        if verlangt(PY_BUF_F_CONTIGUOUS) && !f {
            return Err(crate::umwandlung::PygameBufferError::new_err(
                "An F-contiguous buffer is required",
            ));
        }
        if verlangt(PY_BUF_ANY_CONTIGUOUS) && !c && !f {
            return Err(crate::umwandlung::PygameBufferError::new_err(
                "A contiguous buffer is required",
            ));
        }
    }

    let felder = Box::new(Zahlenfelder {
        shape: sicht.shape.clone(),
        strides: sicht.strides.clone(),
        format: sicht.format.clone(),
    });

    unsafe {
        let v = &mut *view;
        v.buf = sicht.adresse as *mut c_void;
        v.len = sicht.laenge as ffi::Py_ssize_t;
        v.itemsize = sicht.itemsize as ffi::Py_ssize_t;
        v.readonly = c_int::from(!sicht.schreibbar);
        v.suboffsets = std::ptr::null_mut();
        v.format = if flags & PY_BUF_FORMAT != 0 {
            felder.format.as_ptr() as *mut std::ffi::c_char
        } else {
            std::ptr::null_mut()
        };
        if will_nd {
            v.ndim = sicht.ndim() as c_int;
            v.shape = felder.shape.as_ptr() as *mut ffi::Py_ssize_t;
            v.strides = if will_strides {
                felder.strides.as_ptr() as *mut ffi::Py_ssize_t
            } else {
                std::ptr::null_mut()
            };
        } else {
            // **`ndim = 0`, nicht 1.** Gemessen gibt pygame fuer eine
            // Anfrage ohne `PyBUF_ND` genau null Achsen und `shape ==
            // None` - der Verbraucher hat ja nicht nach Achsen gefragt.
            v.ndim = 0;
            v.shape = std::ptr::null_mut();
            v.strides = std::ptr::null_mut();
        }
        v.internal = Box::into_raw(felder) as *mut c_void;
        // Der Halter zuletzt: bis hierher konnte noch ein Fehler kommen,
        // und dann darf kein Zaehler erhoeht worden sein.
        let o = eigner.as_ptr();
        ffi::Py_INCREF(o);
        v.obj = o;
    }
    Ok(())
}

/// Gibt frei, was [`fuellen`] auf dem Haufen angelegt hat.
///
/// # Sicherheit
///
/// # Safety
///
/// Darf nur mit einem `view` gerufen werden, das [`fuellen`] gefuellt hat -
/// CPython sorgt dafuer, indem es `bf_releasebuffer` genau einmal je
/// `bf_getbuffer` ruft. NULL ist erlaubt und tut nichts.
pub unsafe fn freigeben(view: *mut ffi::Py_buffer) {
    if view.is_null() {
        return;
    }
    unsafe {
        let v = &mut *view;
        if !v.internal.is_null() {
            drop(Box::from_raw(v.internal as *mut Zahlenfelder));
            v.internal = std::ptr::null_mut();
        }
        v.shape = std::ptr::null_mut();
        v.strides = std::ptr::null_mut();
    }
}

// ---------------------------------------------------------------------------
//  `__array_struct__`
// ---------------------------------------------------------------------------

/// Der C-Kasten, den numpy hinter `__array_struct__` erwartet.
///
/// Feld fuer Feld aus numpys `PyArrayInterface` uebernommen - die
/// Reihenfolge ist Teil des Vertrags, nicht Geschmackssache.
#[repr(C)]
struct FeldSchnittstelle {
    two: c_int,
    nd: c_int,
    typekind: std::ffi::c_char,
    itemsize: c_int,
    flags: c_int,
    shape: *mut ffi::Py_ssize_t,
    strides: *mut ffi::Py_ssize_t,
    data: *mut c_void,
    descr: *mut ffi::PyObject,
}

/// Was an der Kapsel haengt und mit ihr stirbt.
///
/// **`repr(C)` ist hier kein Schmuck.** Die Kapsel gibt einen Zeiger auf
/// den `kasten` heraus, und der muss am **Anfang** der Struktur liegen -
/// Rust darf Felder sonst umsortieren. Ohne diese Zeile las numpy die
/// Zahlenfelder als `nd` und bekam eine Null; `surface_test.test_get_view`
/// meldete `0 != 2`.
#[repr(C)]
struct Kapselinhalt {
    kasten: FeldSchnittstelle,
    shape: Vec<ffi::Py_ssize_t>,
    strides: Vec<ffi::Py_ssize_t>,
}

/// Der Zerstoerer der Kapsel - raeumt [`Kapselinhalt`] ab.
///
/// # Sicherheit
///
/// Wird nur von CPython gerufen, und nur mit der Kapsel, die
/// [`feldschnittstelle`] gebaut hat.
unsafe extern "C" fn kapsel_weg(kapsel: *mut ffi::PyObject) {
    unsafe {
        let p = ffi::PyCapsule_GetPointer(kapsel, std::ptr::null());
        if !p.is_null() {
            // Der Zeiger zeigt auf das **erste Feld** des Inhalts, also
            // auf den Kasten - und der liegt am Anfang der Struktur.
            drop(Box::from_raw(p as *mut Kapselinhalt));
        }
    }
}

/// Baut `__array_struct__` - eine Kapsel mit numpys Kasten darin.
///
/// Die Kapsel hat **keinen Namen** (`NULL`); so verlangt es numpys
/// Vereinbarung, und `pygame.tests.test_utils.arrinter` prueft genau das
/// mit `PyCapsule_IsValid(cobj, None)`.
///
/// # Sicherheit
///
/// Legt einen Kasten auf dem Haufen an und gibt ihn an die Kapsel weiter;
/// der Zerstoerer raeumt ihn wieder ab. Der Zeiger `data` muss so lange
/// gueltig bleiben, wie die Kapsel lebt - dieselbe Zusage wie bei
/// [`fuellen`].
pub fn feldschnittstelle(py: Python<'_>, sicht: &Sicht) -> PyResult<Py<PyAny>> {
    let typestr = sicht.typestr();
    let art = typestr.as_bytes()[1] as std::ffi::c_char;
    let mut flags = PAI_ALIGNED | PAI_NOTSWAPPED;
    if sicht.schreibbar {
        flags |= PAI_WRITEABLE;
    }
    if sicht.c_zusammen() {
        flags |= PAI_CONTIGUOUS;
    }
    if sicht.f_zusammen() {
        flags |= PAI_FORTRAN;
    }

    let mut inhalt = Box::new(Kapselinhalt {
        kasten: FeldSchnittstelle {
            two: 2,
            nd: sicht.ndim() as c_int,
            typekind: art,
            itemsize: sicht.itemsize as c_int,
            flags,
            shape: std::ptr::null_mut(),
            strides: std::ptr::null_mut(),
            data: sicht.adresse as *mut c_void,
            descr: std::ptr::null_mut(),
        },
        shape: sicht.shape.clone(),
        strides: sicht.strides.clone(),
    });
    inhalt.kasten.shape = inhalt.shape.as_mut_ptr();
    inhalt.kasten.strides = inhalt.strides.as_mut_ptr();

    let roh = Box::into_raw(inhalt);
    unsafe {
        // Der Kasten liegt am Anfang der Struktur - `#[repr(C)]` und das
        // erste Feld, also ist die Adresse dieselbe.
        let kapsel = ffi::PyCapsule_New(
            roh as *mut c_void,
            std::ptr::null(),
            Some(kapsel_weg),
        );
        if kapsel.is_null() {
            drop(Box::from_raw(roh));
            return Err(PyErr::fetch(py));
        }
        Ok(Py::from_owned_ptr(py, kapsel))
    }
}

// ---------------------------------------------------------------------------
//  Einen fremden Puffer lesen
// ---------------------------------------------------------------------------

/// Liest die Beschreibung eines **fremden** Puffers aus.
///
/// Der Weg fuer `BufferProxy(objekt)` und `mixer.Sound(objekt)`: CPython
/// gibt den Puffer her, hier wird er in eine [`Sicht`] uebersetzt und
/// sofort wieder freigegeben. Der Zeiger bleibt gueltig, solange das
/// Objekt lebt - dieselbe Zusage, die rustgame fuer seine eigenen Puffer
/// gibt.
///
/// # Sicherheit
///
/// `PyObject_GetBuffer` und `PyBuffer_Release` werden als Paar gerufen.
/// Das Format wird kopiert, nicht gemerkt: CPython darf seinen
/// Formatzeiger nach dem Freigeben wegwerfen.
pub fn fremde_sicht(o: &Bound<'_, PyAny>) -> PyResult<(Sicht, String)> {
    let py = o.py();
    let mut view = std::mem::MaybeUninit::<ffi::Py_buffer>::zeroed();
    let ok = unsafe {
        ffi::PyObject_GetBuffer(
            o.as_ptr(),
            view.as_mut_ptr(),
            PY_BUF_STRIDES | PY_BUF_FORMAT,
        )
    };
    if ok != 0 {
        return Err(PyErr::fetch(py));
    }
    let v = unsafe { view.assume_init() };
    let format = if v.format.is_null() {
        "B".to_string()
    } else {
        unsafe { CStr::from_ptr(v.format) }
            .to_string_lossy()
            .into_owned()
    };
    let ndim = v.ndim.max(0) as usize;
    let mut shape = Vec::with_capacity(ndim);
    let mut strides = Vec::with_capacity(ndim);
    for i in 0..ndim {
        shape.push(if v.shape.is_null() {
            v.len / v.itemsize.max(1)
        } else {
            unsafe { *v.shape.add(i) }
        });
        strides.push(if v.strides.is_null() {
            // Ohne Schrittweiten ist es C-zusammenhaengend; sie lassen
            // sich also ausrechnen.
            let mut s = v.itemsize;
            for j in (i + 1)..ndim {
                s *= unsafe { *v.shape.add(j) };
            }
            s
        } else {
            unsafe { *v.strides.add(i) }
        });
    }
    let sicht = Sicht {
        adresse: v.buf as usize,
        laenge: v.len.max(0) as usize,
        itemsize: v.itemsize.max(1) as usize,
        // Der Formatzeiger von CPython darf nicht gemerkt werden; die
        // Kennung wird **kopiert**, unveraendert.
        format: fmt(&format),
        shape,
        strides,
        schreibbar: v.readonly == 0,
        solide: true,
    };
    unsafe { ffi::PyBuffer_Release(&v as *const ffi::Py_buffer as *mut ffi::Py_buffer) };
    let sicht = Sicht {
        solide: sicht.c_zusammen() || sicht.ndim() == 0,
        ..sicht
    };
    Ok((sicht, format))
}

/// Kurz fuer "diese Zeichenkette als Format".
///
/// Ein Format hat nie ein Nullbyte in sich; faellt `CString::new` doch
/// aus, ist eine leere Zeichenkette die richtige Antwort - sie ist
/// ungueltig, und [`format_pruefen`] sagt das dann auch.
pub fn fmt(f: &str) -> CString {
    CString::new(f).unwrap_or_default()
}

/// Prueft eine Formatzeichenkette so, wie `pgObject_GetBuffer` es tut.
///
/// pygame nimmt **nicht** jedes PEP-3118-Format an, sondern nur einen
/// einzelnen Zahlentyp oder Fuellbytes. Uebernommen aus `base.c`, Zeichen
/// fuer Zeichen:
///
/// 1. wahlweise ein Vorzeichen `@ = < > !`
/// 2. wahlweise eine Ziffer - `1` immer, `2` bis `9` nur vor einem `x`
/// 3. **genau einer** aus `b B h H i I l L q Q f d x`
/// 4. danach das Ende der Zeichenkette
///
/// Die Reihenfolge ist der Grund, warum `"2h"` ausfaellt und `"2x"` nicht:
/// die Ziffer wird nur uebersprungen, wenn ein `x` folgt, und `"2h"`
/// bleibt damit auf der `2` haengen - keine Typkennung.
///
/// Gemessen an `base_test.test_bad_format`, das dreizehn Formate
/// durchprobiert: `""`, `"="`, `"1"`, `" "`, `"2h"`, `"=2h"`, `"0x"`,
/// `"11x"`, `"=!"`, `"h "`, `" h"`, `"hh"`, `"?"`.
pub fn format_pruefen(f: &str) -> PyResult<()> {
    let z: Vec<char> = f.chars().collect();
    let mut i = 0;
    if matches!(z.first(), Some('@' | '=' | '<' | '>' | '!')) {
        i += 1;
    }
    match z.get(i) {
        Some('1') => i += 1,
        Some('2'..='9') if z.get(i + 1) == Some(&'x') => i += 1,
        _ => {}
    }
    match z.get(i) {
        Some('b' | 'B' | 'h' | 'H' | 'i' | 'I' | 'l' | 'L' | 'q' | 'Q' | 'f' | 'd' | 'x') => {
            i += 1
        }
        _ => {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Unsupported array element type",
            ))
        }
    }
    if i != z.len() {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "Arrays of records are unsupported",
        ));
    }
    Ok(())
}

/// Wie viele Bytes ein Formatbuchstabe belegt.
pub fn format_groesse(f: &str) -> usize {
    let kern = f.trim_start_matches(['=', '<', '>', '!', '@']);
    // `3x` und Geschwister: die Zahl ist die Byteanzahl.
    if let Some(rest) = kern.strip_suffix('x') {
        return rest.parse::<usize>().unwrap_or(1);
    }
    match kern {
        "b" | "B" | "c" => 1,
        "h" | "H" => 2,
        "i" | "I" | "l" | "L" | "f" => 4,
        "q" | "Q" | "d" => 8,
        _ => 1,
    }
}

// ---------------------------------------------------------------------------
//  Die Lese-Seite: einen fremden Puffer annehmen
// ---------------------------------------------------------------------------

/// Baut eine Formatzeichenkette aus Typkennung, Breite und Byteordnung.
///
/// Wort fuer Wort aus `_pg_arraystruct_to_format`. Bemerkenswert daran:
/// das Vorzeichen steht **auch bei einem einzigen Byte** davor (`"=B"`,
/// nicht `"B"`), und Fuellbytes bekommen keines (`"3x"`).
fn kasten_format(typekind: char, itemsize: usize, getauscht: bool) -> PyResult<CString> {
    let vor = if getauscht { BUF_ANDERS } else { BUF_MEINS };
    let falsch = |was: &str| {
        Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Unsupported {was} size {itemsize}"
        )))
    };
    Ok(match typekind {
        'i' => match itemsize {
            1 => fmt(&format!("{vor}b")),
            2 => fmt(&format!("{vor}h")),
            4 => fmt(&format!("{vor}i")),
            8 => fmt(&format!("{vor}q")),
            _ => return falsch("signed integer"),
        },
        'u' => match itemsize {
            1 => fmt(&format!("{vor}B")),
            2 => fmt(&format!("{vor}H")),
            4 => fmt(&format!("{vor}I")),
            8 => fmt(&format!("{vor}Q")),
            _ => return falsch("unsigned integer"),
        },
        'f' => match itemsize {
            4 => fmt(&format!("{vor}f")),
            8 => fmt(&format!("{vor}d")),
            _ => return falsch("float"),
        },
        // Fuellbytes: die Anzahl ist der Zaehler, und mehr als neun gibt
        // es nicht - PEP 3118 laesst nur eine Ziffer zu.
        'V' if (1..=9).contains(&itemsize) => fmt(&format!("{itemsize}x")),
        'V' => return falsch("void"),
        _ => {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Unsupported value type '{typekind}'"
            )))
        }
    })
}

/// Liest `__array_struct__` - die Beschreibung als C-Kasten in einer Kapsel.
///
/// Das Gegenstueck zu [`feldschnittstelle`]. pygame nennt es
/// `pgGetArrayStruct` und `_pg_arraystruct_as_buffer`; die Fehlertexte sind
/// von dort uebernommen, weil `base_test` sie nicht prueft, andere
/// Ausgeber aber danach suchen koennen.
///
/// # Sicherheit
///
/// Der Zeiger kommt aus einer Kapsel, deren Gueltigkeit `PyCapsule_IsValid`
/// bestaetigt hat, und `two == 2` ist numpys Erkennungszeichen dafuer, dass
/// dahinter wirklich ein `PyArrayInterface` liegt. Gelesen wird nur,
/// solange `kapsel` lebt - also innerhalb dieses Aufrufs.
pub fn kapsel_sicht(o: &Bound<'_, PyAny>) -> PyResult<Sicht> {
    let kapsel = o.getattr("__array_struct__").map_err(|e| {
        if e.is_instance_of::<pyo3::exceptions::PyAttributeError>(o.py()) {
            pyo3::exceptions::PyValueError::new_err("no C-struct array interface")
        } else {
            e
        }
    })?;
    let ungueltig =
        || pyo3::exceptions::PyValueError::new_err("invalid array interface");
    let zeiger = unsafe {
        if ffi::PyCapsule_IsValid(kapsel.as_ptr(), std::ptr::null()) == 0 {
            return Err(ungueltig());
        }
        ffi::PyCapsule_GetPointer(kapsel.as_ptr(), std::ptr::null())
            as *const FeldSchnittstelle
    };
    if zeiger.is_null() {
        return Err(ungueltig());
    }
    let inter = unsafe { &*zeiger };
    if inter.two != 2 {
        return Err(ungueltig());
    }
    let nd = inter.nd.max(0) as usize;
    let itemsize = inter.itemsize.max(0) as usize;
    let typekind = inter.typekind as u8 as char;
    let format = kasten_format(typekind, itemsize, inter.flags & PAI_NOTSWAPPED == 0)?;

    if inter.shape.is_null() && nd > 0 {
        return Err(ungueltig());
    }
    let mut shape = Vec::with_capacity(nd);
    for i in 0..nd {
        shape.push(unsafe { *inter.shape.add(i) });
    }
    // Ohne Schrittweiten gilt die C-Reihenfolge - aber nur, wenn der
    // Ausgeber sich auch als zusammenhaengend gemeldet hat.
    let mut strides = Vec::with_capacity(nd);
    if inter.strides.is_null() {
        if inter.flags & (PAI_CONTIGUOUS | PAI_FORTRAN) == 0 {
            return Err(crate::umwandlung::PygameBufferError::new_err(
                "buffer is not contiguous, strides needed",
            ));
        }
        let mut lauf = itemsize as ffi::Py_ssize_t;
        strides = vec![0; nd];
        for i in (0..nd).rev() {
            strides[i] = lauf;
            lauf *= shape[i];
        }
    } else {
        for i in 0..nd {
            strides.push(unsafe { *inter.strides.add(i) });
        }
    }
    let laenge = shape.iter().product::<ffi::Py_ssize_t>().max(0) as usize * itemsize;
    let vor = Sicht {
        adresse: inter.data as usize,
        laenge,
        itemsize,
        format,
        shape,
        strides,
        schreibbar: inter.flags & PAI_WRITEABLE != 0,
        solide: true,
    };
    Ok(Sicht {
        solide: vor.c_zusammen(),
        ..vor
    })
}

/// Uebersetzt einen `typestr` wie `"<i2"` in ein PEP-3118-Format.
///
/// Aus `_pg_typestr_as_format`. Die Falle darin: **`i3` und `i5` bis `i9`
/// werden Fuellbytes**, weil es keinen Zahlentyp dieser Breite gibt -
/// `"<i3"` kommt als `"3x"` heraus und nicht als Fehler.
fn typestr_als_format(ts: &str) -> PyResult<(CString, usize)> {
    let z: Vec<char> = ts.chars().collect();
    let schlecht = || {
        pyo3::exceptions::PyValueError::new_err(format!("unsupported typestr {ts}"))
    };
    if z.len() != 3 {
        return Err(schlecht());
    }
    let getauscht = match z[0] {
        c if c == PAI_MEINS || c == '|' => false,
        c if c == PAI_ANDERS => true,
        _ => return Err(schlecht()),
    };
    let vor = if getauscht { BUF_ANDERS } else { BUF_MEINS };
    let breite = z[2].to_digit(10).ok_or_else(schlecht)? as usize;
    let (f, n) = match (z[1], breite) {
        ('u', 1) => ("B".to_string(), 1),
        ('i', 1) => ("b".to_string(), 1),
        ('u', 2) => (format!("{vor}H"), 2),
        ('i', 2) => (format!("{vor}h"), 2),
        ('u', 4) => (format!("{vor}I"), 4),
        ('i', 4) => (format!("{vor}i"), 4),
        ('u', 8) => (format!("{vor}Q"), 8),
        ('i', 8) => (format!("{vor}q"), 8),
        // Drei, fuenf, sechs, sieben, neun: keine Zahl, sondern Fuellbytes.
        ('u' | 'i' | 'V', n @ (1..=9)) => (format!("{n}x"), n),
        ('f', 4) => (format!("{vor}f"), 4),
        ('f', 8) => (format!("{vor}d"), 8),
        _ => return Err(schlecht()),
    };
    Ok((fmt(&f), n))
}

/// Liest ein `__array_interface__`-Woerterbuch aus.
///
/// Aus `pgDict_AsBuffer` samt seiner vier Pruefungen - und die sind
/// **wortgenau** uebernommen, weil `bufferproxy_test.test_bad_arguments`
/// die Texte einzeln vergleicht.
pub fn dict_sicht(d: &Bound<'_, pyo3::types::PyDict>) -> PyResult<Sicht> {
    use pyo3::exceptions::PyValueError;
    use pyo3::types::{PyAnyMethods, PyTuple};

    let ganzzahlen = |o: &Bound<'_, PyAny>| -> Option<Vec<ffi::Py_ssize_t>> {
        let t = o.downcast::<PyTuple>().ok()?;
        let mut aus = Vec::with_capacity(t.len());
        for e in t.iter() {
            if !e.is_instance_of::<pyo3::types::PyInt>() {
                return None;
            }
            aus.push(e.extract::<ffi::Py_ssize_t>().ok()?);
        }
        Some(aus)
    };

    let shape_o = d
        .get_item("shape")?
        .ok_or_else(|| PyValueError::new_err("required 'shape' item is missing"))?;
    let shape = ganzzahlen(&shape_o)
        .ok_or_else(|| PyValueError::new_err("expected a tuple of ints for 'shape'"))?;
    if shape.is_empty() {
        return Err(PyValueError::new_err(
            "expected 'shape' to be at least one-dimensional",
        ));
    }

    let typestr_o = d
        .get_item("typestr")?
        .ok_or_else(|| PyValueError::new_err("required 'typestr' item is missing"))?;
    let typestr: String = if let Ok(t) = typestr_o.extract::<String>() {
        t
    } else if let Ok(b) = typestr_o.extract::<Vec<u8>>() {
        String::from_utf8_lossy(&b).into_owned()
    } else {
        return Err(PyValueError::new_err("expected a string for 'typestr'"));
    };
    if typestr.chars().count() != 3 {
        return Err(PyValueError::new_err("expected 'typestr' to be length 3"));
    }

    let data_o = d
        .get_item("data")?
        .ok_or_else(|| PyValueError::new_err("required 'data' item is missing"))?;
    let data = data_o
        .downcast::<PyTuple>()
        .map_err(|_| PyValueError::new_err("expected a tuple for 'data'"))?;
    if data.len() != 2 {
        return Err(PyValueError::new_err("expected a length 2 tuple for 'data'"));
    }
    let adresse_o = data.get_item(0)?;
    if !adresse_o.is_instance_of::<pyo3::types::PyInt>() {
        return Err(PyValueError::new_err("expected an int for item 0 of 'data'"));
    }
    let adresse: usize = adresse_o.extract()?;
    let nur_lesen = data.get_item(1)?.is_truthy()?;

    let (format, itemsize) = typestr_als_format(&typestr)?;

    let strides = match d.get_item("strides")? {
        // **`None` ist kein Weglassen.** numpys Abrede sagt, `strides:
        // None` heisse C-Reihenfolge - pygame nimmt den Schluessel so
        // aber **nicht** an und gibt `ValueError: expected a tuple of
        // ints for 'strides'`. Gemessen an allen drei Faellen:
        //
        // ```text
        // ohne den Schluessel      length = 4
        // strides = None           ValueError
        // strides = (1,)           length = 4
        // ```
        //
        // Hier stand `Some(o) if !o.is_none()`, und damit war `None`
        // dasselbe wie "nicht da".
        Some(o) => {
            let v = ganzzahlen(&o).ok_or_else(|| {
                PyValueError::new_err("expected a tuple of ints for 'strides'")
            })?;
            if v.len() != shape.len() {
                return Err(PyValueError::new_err(
                    "'shape' and 'strides' are not the same length",
                ));
            }
            v
        }
        // Ohne Angabe: C-Reihenfolge.
        _ => {
            let mut aus = vec![0 as ffi::Py_ssize_t; shape.len()];
            let mut lauf = itemsize as ffi::Py_ssize_t;
            for i in (0..shape.len()).rev() {
                aus[i] = lauf;
                lauf *= shape[i];
            }
            aus
        }
    };
    let laenge = shape.iter().product::<ffi::Py_ssize_t>().max(0) as usize * itemsize;
    let vor = Sicht {
        adresse,
        laenge,
        itemsize,
        format,
        shape,
        strides,
        schreibbar: !nur_lesen,
        solide: true,
    };
    Ok(Sicht {
        solide: vor.c_zusammen(),
        ..vor
    })
}

/// `pgObject_GetBuffer` - alle drei Wege, in pygames Reihenfolge.
///
/// **Die Reihenfolge ist gemessen, nicht geraten**, und sie ist nicht die,
/// die man erwartet: erst das **Puffer-Protokoll**, dann
/// `__array_struct__`, dann `__array_interface__`. Zwei Tests haengen
/// genau daran und pruefen es von beiden Seiten:
///
/// * `base_test.test_GetView_array_struct` gibt einem Objekt
///   `__array_interface__ = property(lambda self: None)` mit - eine
///   Eigenschaft, die `None` liefert. Sie darf `__array_struct__` **nicht**
///   im Weg stehen.
/// * `test_PgObject_GetBuffer_array_interface` macht es umgekehrt:
///   `__array_struct__` liefert `None` und wird uebergangen.
///
/// Dass beides geht, liegt daran, dass jeder Weg bei einem Fehlschlag
/// **stillschweigend** aufgegeben wird - `None` ist keine Kapsel und kein
/// Woerterbuch, also kommt der naechste dran. Nur der letzte Fehlschlag
/// wird gemeldet.
///
/// Eine Ausnahme gibt es: gibt das Puffer-Protokoll einen Puffer mit
/// **ungueltigem Format** her, dann ist das ein Fehler und kein Grund,
/// weiterzusuchen. pygame macht es genauso, und
/// `base_test.test_bad_format` verlangt es.
pub fn hol_sicht(o: &Bound<'_, PyAny>) -> PyResult<Sicht> {
    let py = o.py();
    let mut weigerung: Option<PyErr> = None;
    match fremde_sicht(o) {
        Ok((sicht, roh)) => {
            format_pruefen(&roh)?;
            return Ok(sicht);
        }
        // **Eine Weigerung ist kein Fehlen.** Wer `TypeError: a bytes-like
        // object is required` sagt, hat kein Puffer-Protokoll; wer
        // `BufferError` sagt, hat eines und gibt es aus einem Grund nicht
        // her. Der Grund ist die bessere Auskunft als "exportiert keinen
        // Puffer" - bei rustgames Flaechen unter 32 Bit steht darin, warum.
        Err(e) if e.is_instance_of::<pyo3::exceptions::PyBufferError>(py) => {
            weigerung = Some(e);
        }
        Err(_) => {}
    }
    if let Ok(s) = kapsel_sicht(o) {
        return Ok(s);
    }
    if let Ok(ai) = o.getattr("__array_interface__") {
        if let Ok(d) = ai.downcast::<pyo3::types::PyDict>() {
            return dict_sicht(d);
        }
    }
    Err(weigerung.unwrap_or_else(|| {
        pyo3::exceptions::PyValueError::new_err(format!(
            "{} object does not export an array buffer",
            o.get_type()
                .name()
                .map(|n| n.to_string())
                .unwrap_or_else(|_| "?".into())
        ))
    }))
}

/// Die Bytes einer Sicht als Kopie - das, was `BufferProxy.raw` gibt.
///
/// Geht nur, wenn der Bereich **ein Stueck** ist. Gemessen an
/// `bufferproxy_test.test_raw`, das drei Faelle nebeneinanderstellt:
///
/// | shape | typestr | strides | Ergebnis |
/// |---|---|---|---|
/// | `(48,)` | `\|u1` | - | die 48 Bytes |
/// | `(3, 4)` | `\|u4` | `(4, 12)` | dieselben 48 Bytes |
/// | `(3, 4)` | `\|u1` | `(16, 4)` | `ValueError` |
///
/// Die Mitte ist die aufschlussreiche: `(4, 12)` ist **nicht**
/// C-zusammenhaengend (das waere `(16, 4)`), sondern spaltenweise - und das
/// reicht. Wer nur `c_zusammen()` prueft, gibt hier einen Fehler, wo pygame
/// Bytes gibt.
///
/// # Sicherheit
///
/// Liest `laenge` Bytes ab `adresse`. Beides kommt vom Ausgeber, und dass
/// der Bereich ihm gehoert, ist seine Zusage - dieselbe, die
/// `PyObject_GetBuffer` gibt. Die Adresse Null wird vorher abgefangen: ein
/// Woerterbuch darf `data: (0, True)` sagen, und `bufferproxy_test` tut es.
pub fn bytes_lesen(sicht: &Sicht) -> PyResult<Vec<u8>> {
    if !sicht.c_zusammen() && !sicht.f_zusammen() {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "the bytes are not contiguous",
        ));
    }
    if sicht.adresse == 0 {
        return Err(crate::umwandlung::PygameBufferError::new_err(
            "buffer data pointer is NULL",
        ));
    }
    let mut aus = vec![0u8; sicht.laenge];
    unsafe {
        std::ptr::copy_nonoverlapping(
            sicht.adresse as *const u8,
            aus.as_mut_ptr(),
            sicht.laenge,
        );
    }
    Ok(aus)
}

/// Legt Bytes in eine Sicht - das, was `BufferProxy.write` tut.
///
/// **Zwei Fehler, und sie sind nicht dieselben.** Gemessen:
///
/// ```text
/// laenge 10, write(10 Bytes, offset 0)   geht
/// laenge 10, write(11 Bytes, offset 0)   ValueError  'buffer' object length is too large
/// laenge 10, write(10 Bytes, offset 1)   IndexError  'offset' is out of range
/// laenge 10, write(5 Bytes, offset -1)   IndexError
/// laenge 10, write(5 Bytes, offset 10)   IndexError
/// ```
///
/// Also: zu **viele Bytes** ist ein `ValueError`, ein Versatz, der sie
/// hinausschiebt, ein `IndexError`. Die Reihenfolge der beiden Pruefungen
/// ist damit festgelegt und nicht beliebig.
///
/// # Sicherheit
///
/// Schreibt `daten.len()` Bytes ab `adresse + offset`, nachdem beides gegen
/// `laenge` geprueft ist. Der Bereich muss beschreibbar sein; ein
/// nur-lesbarer gibt vorher `BufferError`.
pub fn bytes_schreiben(sicht: &Sicht, offset: isize, daten: &[u8]) -> PyResult<()> {
    if !sicht.schreibbar {
        return Err(crate::umwandlung::PygameBufferError::new_err(
            "buffer is read-only",
        ));
    }
    if daten.len() > sicht.laenge {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "'buffer' object length is too large",
        ));
    }
    if offset < 0 || offset as usize + daten.len() > sicht.laenge {
        return Err(pyo3::exceptions::PyIndexError::new_err(
            "'offset' is out of range",
        ));
    }
    if !sicht.c_zusammen() && !sicht.f_zusammen() {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "the bytes are not contiguous",
        ));
    }
    if sicht.adresse == 0 {
        return Err(crate::umwandlung::PygameBufferError::new_err(
            "buffer data pointer is NULL",
        ));
    }
    unsafe {
        std::ptr::copy_nonoverlapping(
            daten.as_ptr(),
            (sicht.adresse + offset as usize) as *mut u8,
            daten.len(),
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
//  Einzelne Zahlen in einem fremden Feld
// ---------------------------------------------------------------------------

/// Ist dieses Format andersherum als diese Maschine?
pub fn getauscht(format: &str) -> bool {
    match format.chars().next() {
        Some('<') => cfg!(target_endian = "big"),
        Some('>') | Some('!') => cfg!(target_endian = "little"),
        _ => false,
    }
}

/// Was fuer eine Sorte Zahl ein Format beschreibt.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Art {
    /// `b B h H i I l L q Q` - eine ganze Zahl.
    Ganz,
    /// `f d` - eine Fliesskommazahl.
    Komma,
    /// `?` - ein Wahrheitswert.
    Wahrheit,
    /// `x`, `2x` … `9x` - Fuellbytes ohne eigene Bedeutung.
    Fuell,
    /// Alles andere.
    Fremd,
}

/// Die Sorte hinter einer Formatzeichenkette.
pub fn art(format: &str) -> Art {
    let kern = format.trim_start_matches(['=', '<', '>', '!', '@']);
    let kern = kern.strip_prefix('1').filter(|r| !r.is_empty()).unwrap_or(kern);
    let letzte = kern.chars().next_back();
    match letzte {
        Some('x') => Art::Fuell,
        Some('f' | 'd') => Art::Komma,
        Some('?') => Art::Wahrheit,
        Some('b' | 'B' | 'h' | 'H' | 'i' | 'I' | 'l' | 'L' | 'q' | 'Q') => Art::Ganz,
        _ => Art::Fremd,
    }
}

/// Liest die Zahl, die `versatz` Bytes hinter dem Anfang steht.
///
/// `itemsize` von einem bis neun Bytes - mehr laesst PEP 3118 mit einer
/// Ziffer nicht zu, und mehr als acht traegt keine Zahl. Die Byteordnung
/// kommt aus dem **Format**, nicht aus der Maschine: ein Feld mit `">I"`
/// wird hier richtig herum gelesen.
///
/// Die schiefen Breiten drei, fuenf, sechs, sieben und neun kommen von
/// Fuellbytes (`3x` und Geschwister). Sie tragen keine Zahl, die eine
/// Maschine kennt, und werden darum byteweise von unten her behandelt -
/// gemessen an `freetype_test.test_newbuf`, das `render_raw_to` gegen alle
/// zehn Fuellbreiten fahrt und in jedem Element das **erste** Byte
/// beschrieben findet.
///
/// # Sicherheit
///
/// `versatz` muss so gewaehlt sein, dass `itemsize` Bytes davor noch im
/// Bereich der Sicht liegen. Der einzige Aufrufer ist
/// [`crate::pixelkopie`], und der rechnet den Versatz aus `shape` und
/// `strides` derselben Sicht - also aus den Angaben des Ausgebers selbst.
pub fn feld_lesen(sicht: &Sicht, versatz: isize) -> u64 {
    let n = sicht.itemsize;
    if !(1..=9).contains(&n) {
        return 0;
    }
    let mut b = [0u8; 9];
    unsafe {
        std::ptr::copy_nonoverlapping(
            (sicht.adresse as *const u8).offset(versatz),
            b.as_mut_ptr(),
            n,
        );
    }
    let dreh = getauscht(&sicht.format.to_string_lossy());
    let mut wert = 0u64;
    for i in 0..n.min(8) {
        let byte = if dreh { b[n - 1 - i] } else { b[i] } as u64;
        wert |= byte << (8 * i);
    }
    wert
}

/// Legt eine Zahl `versatz` Bytes hinter dem Anfang ab.
///
/// # Sicherheit
///
/// Dieselbe Bedingung wie bei [`feld_lesen`], und zusaetzlich muss die
/// Sicht beschreibbar sein - das prueft der Aufrufer, bevor er anfaengt.
pub fn feld_schreiben(sicht: &Sicht, versatz: isize, wert: u64) {
    let n = sicht.itemsize;
    if !(1..=9).contains(&n) {
        return;
    }
    let dreh = getauscht(&sicht.format.to_string_lossy());
    let mut b = [0u8; 9];
    for i in 0..n.min(8) {
        let byte = (wert >> (8 * i)) as u8;
        if dreh {
            b[n - 1 - i] = byte;
        } else {
            b[i] = byte;
        }
    }
    unsafe {
        std::ptr::copy_nonoverlapping(
            b.as_ptr(),
            (sicht.adresse as *mut u8).offset(versatz),
            n,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Eine Sicht von Hand - ohne Python, nur die Zahlen.
    ///
    /// **Diese Datei war ein Jahr lang nicht uebersetzt.** `cargo test`
    /// wird im Workspace gerufen, und `rustgame-py` steht ausserhalb des
    /// Workspace (eigene `[workspace]`-Tabelle). Also lief hier nie ein
    /// Test, und drei Typfehler standen ungestoert im Testcode:
    /// `shape` und `strides` wurden zu `Vec<i64>` statt `Vec<isize>`, und
    /// `format` bekam ein `&CStr` statt eines `CString`. Seitdem gehoert
    /// `cargo test --release` in dieser Kiste zur Pruefliste.
    fn sicht(shape: &[isize], strides: &[isize], itemsize: usize) -> Sicht {
        Sicht {
            adresse: 0x1000,
            laenge: shape.iter().product::<isize>() as usize * itemsize,
            itemsize,
            format: fmt("=I"),
            shape: shape.to_vec(),
            strides: strides.to_vec(),
            schreibbar: true,
            solide: true,
        }
    }

    #[test]
    fn die_zweidimensionale_sicht_ist_spaltenweise_zusammen() {
        // Eine Flaeche 10 x 6 mit vier Byte je Pixel: `shape` ist
        // `(Breite, Hoehe)`, und damit laufen die Schrittweiten
        // spaltenweise.
        let s = sicht(&[10, 6], &[4, 40], 4);
        assert!(!s.c_zusammen(), "waagerecht ist sie es nicht");
        assert!(s.f_zusammen(), "spaltenweise schon");
    }

    #[test]
    fn die_eindimensionale_sicht_ist_beides() {
        let s = sicht(&[60], &[4], 4);
        assert!(s.c_zusammen());
        assert!(s.f_zusammen());
    }

    #[test]
    fn eine_teilflaeche_ist_in_keiner_richtung_zusammen() {
        // 7 x 4 aus einer Flaeche, die 10 breit ist: die Zeilen haben
        // Luecken.
        let s = sicht(&[7, 4], &[4, 40], 4);
        assert!(!s.c_zusammen());
        assert!(!s.f_zusammen());
    }

    #[test]
    fn typestr_nennt_die_bytereihenfolge() {
        assert_eq!(sicht(&[4], &[4], 4).typestr(), "<u4");
        let mut s = sicht(&[4], &[1], 1);
        s.format = c"B".into();
        // Ein Byte hat keine Reihenfolge - `|` heisst "einerlei".
        assert_eq!(s.typestr(), "|u1");
    }

    #[test]
    fn format_groesse_versteht_fuellbytes() {
        assert_eq!(format_groesse("3x"), 3);
        assert_eq!(format_groesse("=I"), 4);
        assert_eq!(format_groesse("x"), 1);
    }
}
