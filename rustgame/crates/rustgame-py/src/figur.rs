//! figur.rs
//! ========
//! `pygame.sprite` - Spielfiguren und ihre Gruppen.
//!
//! ## Warum hier **nicht** `rustgame_sprite` eingewickelt wird
//!
//! Die Kiste `rustgame_sprite` ist fertig und wird von Rust-Spielen
//! benutzt. Fuer die Python-Huelle taugt sie trotzdem nicht als Unterbau,
//! und der Grund ist eine einzige, gemessene Zeile:
//!
//! ```python
//! self.rect.x += 1
//! ```
//!
//! Das ist die haeufigste Zeile in pygame-Spielen ueberhaupt. Sie wirkt
//! nur, wenn `sprite.rect` **jedes Mal dasselbe Python-Objekt** liefert -
//! Python uebersetzt sie zu `tmp = self.rect; tmp.x = tmp.x + 1`, ohne
//! Rueckschreiben. In der Kiste liegt das Rechteck als
//! [`rustgame_core::Rect`] im Sprite-Kern, und `PyRect` traegt seinen Wert
//! (siehe Kopf von `rechteck.rs`, `#[derive(Clone, Copy)]`). Ein Zugriff
//! muesste also ein frisches `PyRect` bauen - und die Bewegung waere weg.
//!
//! Dasselbe gilt fuer `image`, `radius`, `mask` und jedes eigene Feld
//! einer Unterklasse. Dazu kommt, dass pygames Gruppen **entenartig**
//! arbeiten: `Group.draw` verlangt nur `spr.image` und `spr.rect`, kein
//! `isinstance`. Gemessen: `Group().add(object())` scheitert nicht am Typ,
//! sondern mit `AttributeError: 'object' object has no attribute
//! 'add_internal'`.
//!
//! Darum ist dieses Modul eine **Uebertragung von `pygame/sprite.py`**
//! (1788 Zeilen, reines Python) nach Rust: Zeile fuer Zeile dieselbe
//! Buchfuehrung, nur dass `spritedict` ein [`PyDict`] und `lostsprites`
//! eine [`PyList`] ist. Damit bleiben Objektgleichheit, Reihenfolge,
//! Hashbarkeit und die Ententypisierung genau die von pygame.
//!
//! ## Der Kniff mit dem Konstruktor
//!
//! `Group(*sprites)` und `Sprite(*groups)` brauchen **sich selbst**
//! waehrend der Erzeugung: ein Sprite wird in beide Buecher eingetragen,
//! und dafuer muss die Gruppe als Woerterbuchschluessel schon existieren.
//! PyO3 kennt nur `__new__` (`#[new]`), kein `__init__` - dort gibt es das
//! Objekt noch nicht.
//!
//! Der Ausweg steht unten in jedem `#[new]` und hat zwei Haelften:
//!
//! 1. `#[new]` ist zugleich `#[classmethod]` und bekommt damit `cls`.
//!    Ist `cls` **genau** dieser Typ, wird das Objekt hier mit
//!    [`Py::new`] gebaut, vollstaendig verdrahtet und als fertiges Objekt
//!    zurueckgegeben (`PyClassInitializer::from(Py<T>)`).
//! 2. Ist `cls` eine **Unterklasse aus Python**, darf das nicht passieren:
//!    ein fertiges Objekt haette den falschen Typ. Dann kommt ein
//!    gewoehnlicher Bauplan zurueck, und die Arbeit macht `__init__` -
//!    das CPython bei Klassen aus einer `class`-Anweisung ueber
//!    `fixup_slot_dispatchers` an `tp_init` haengt, auch wenn es aus dem
//!    Woerterbuch einer C-Klasse stammt.
//!
//! `#[new]` hinterlaesst in **beiden** Faellen ein vollstaendig
//! brauchbares Objekt (leeres `__g`, leeres `spritedict`); nur die
//! Argumente der Unterklasse haengen an `__init__`. Das ist auch der
//! Grund, warum `__init__` ueberhaupt da ist: ohne es scheitert
//! `super().__init__(gruppe)` in jeder Unterklasse an
//! `object.__init__() takes exactly one argument`.
//!
//! ## Gemessen an pygame-ce 2.5.7
//!
//! | Frage | Antwort |
//! |---|---|
//! | `Group.draw()` | immer `[]` - und **dasselbe Objekt** wie `g.lostsprites` danach |
//! | `RenderUpdates.draw()` | die **alte** `lostsprites`-Liste; `g.lostsprites` ist danach eine neue |
//! | `LayeredDirty.draw()` | eine **Kopie** (`list(local_update)`) |
//! | `LayeredUpdates.remove_internal` | legt `sprite.rect` **lebend** ab: nach `move_ip(-3, 2)` meldet es `(31, 26, 8, 5)` statt `(34, 24, 8, 5)` |
//! | `Group.remove_internal` | legt das **gezeichnete** Rechteck ab - eine Kopie, die sich nicht mitbewegt |
//! | `bool(Rect(0, 0, -3, 4))` | `True` - geprueft wird nur `w != 0 and h != 0` |
//! | `Group.has()` ohne Argumente | `False` |
//! | `OrderedUpdates is RenderUpdates` | **`False`** - echte Unterklassen, die nur warnen |
//! | `g.add(s, layer=5)` an einem einfachen `Sprite` | danach ist `hasattr(s, "_layer")` **`False`** |
//! | `sprites()` nach `remove(b); add(b)` | `b` steht **hinten** |
//! | `LayeredDirty` erster `draw` | `[clip]`, und die `dirty`-Flaggen bleiben stehen |
//! | `collide_circle` | schreibt `sprite.radius` an; `collide_circle_ratio` legt den **ungeskalierten** Radius ab |
//! | `collide_mask` | legt `sprite.mask` **nicht** ab |
//! | `spritecollide(s, liste, ...)` | nimmt jedes iterierbare Ding, nicht nur Gruppen |
//! | `groupcollide` | gibt ein echtes `dict` zurueck |

use std::collections::HashSet;

use pyo3::exceptions::{
    PyAttributeError, PyDeprecationWarning, PyIndexError, PyKeyError, PyTypeError, PyValueError,
};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple, PyType};
use pyo3::PyClass;

use rustgame_core::rect::Rect as KernRect;

use crate::flaeche::PySurface;
use crate::maske::PyMask;
use crate::rechteck::PyRect;
use crate::umwandlung::rect_aus;

// ---------------------------------------------------------------------------
//  Kleine Helfer
// ---------------------------------------------------------------------------

/// `group.spritedict` - ueber Python gelesen, damit auch eine Unterklasse
/// aus Python zaehlt, die das Woerterbuch ausgetauscht hat.
fn spritedict<'py>(g: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyDict>> {
    Ok(g.getattr("spritedict")?.downcast_into::<PyDict>()?)
}

/// `group.lostsprites`.
fn lostsprites<'py>(g: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyList>> {
    Ok(g.getattr("lostsprites")?.downcast_into::<PyList>()?)
}

/// `group._spritelist` - nur die Ebenengruppen haben es.
fn spritelist<'py>(g: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyList>> {
    Ok(g.getattr("_spritelist")?.downcast_into::<PyList>()?)
}

/// `group._spritelayers`.
fn spritelayers<'py>(g: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyDict>> {
    Ok(g.getattr("_spritelayers")?.downcast_into::<PyDict>()?)
}

/// `isinstance(x, Sprite)`.
fn ist_sprite(o: &Bound<'_, PyAny>) -> bool {
    o.downcast::<PySprite>().is_ok()
}

/// Der Klassenname fuer `__repr__`.
fn klassenname(o: &Bound<'_, PyAny>) -> PyResult<String> {
    Ok(o.get_type().name()?.to_string())
}

/// Ein `TypeError` oder `AttributeError`? Genau die beiden faengt pygame
/// in `add`, `remove` und `has` ab.
fn ist_enten_fehler(e: &PyErr, py: Python<'_>) -> bool {
    e.is_instance_of::<PyTypeError>(py) || e.is_instance_of::<PyAttributeError>(py)
}

/// `self.sprites()` als Liste - immer ueber Python, damit die Ebenen-
/// und Einzelgruppen ihre eigene Reihenfolge liefern.
fn sprites_von<'py>(g: &Bound<'py, PyAny>) -> PyResult<Vec<Bound<'py, PyAny>>> {
    g.call_method0("sprites")?.try_iter()?.collect()
}

/// `list.remove(x)` mit pygames Fehlermeldung.
fn liste_entfernen(l: &Bound<'_, PyList>, spr: &Bound<'_, PyAny>) -> PyResult<()> {
    for i in 0..l.len() {
        if l.get_item(i)?.is(spr) {
            l.del_item(i)?;
            return Ok(());
        }
    }
    Err(PyValueError::new_err("list.remove(x): x not in list"))
}

/// Die Ebene eines Sprites aus `_spritelayers`.
fn ebene_von(layers: &Bound<'_, PyDict>, spr: &Bound<'_, PyAny>) -> PyResult<i64> {
    match layers.get_item(spr)? {
        Some(v) => v.extract(),
        None => Err(PyKeyError::new_err(spr.clone().unbind())),
    }
}

/// Die Einfuegestelle fuer `layer` - Halbierung samt linearer Nachsuche,
/// zeilengetreu aus `LayeredUpdates.add_internal`.
///
/// Beide zusammen liefern die Stelle **hinter** allen Sprites mit
/// `layer <= ebene`; gleiche Ebene heisst also "hinten anstellen".
fn einfuegestelle(
    liste: &Bound<'_, PyList>,
    layers: &Bound<'_, PyDict>,
    layer: i64,
) -> PyResult<usize> {
    let leng = liste.len() as isize;
    let mut low: isize = 0;
    let mut mid: isize = 0;
    let mut high: isize = leng - 1;
    while low <= high {
        mid = low + (high - low) / 2;
        if ebene_von(layers, &liste.get_item(mid as usize)?)? <= layer {
            low = mid + 1;
        } else {
            high = mid - 1;
        }
    }
    while mid < leng && ebene_von(layers, &liste.get_item(mid as usize)?)? <= layer {
        mid += 1;
    }
    Ok(mid as usize)
}

/// Das Rechteck eines Sprites als Kernrechteck.
fn rect_von_sprite(spr: &Bound<'_, PyAny>) -> PyResult<KernRect> {
    rect_aus(&spr.getattr("rect")?)
}

// ---------------------------------------------------------------------------
//  Sprite
// ---------------------------------------------------------------------------

/// `pygame.sprite.Sprite`.
///
/// `image` und `rect` sind echte Zugriffsobjekte (Datendeskriptoren) und
/// geben **dasselbe** Python-Objekt zurueck, das gesetzt wurde - nur so
/// wirkt `self.rect.x += 1`. Alles andere (`radius`, `mask`, `_layer`,
/// eigene Felder) landet im `__dict__`, genau wie in pygames reiner
/// Python-Klasse.
#[pyclass(name = "Sprite", module = "rustgame_py.sprite", subclass, dict)]
pub struct PySprite {
    /// `Sprite.__g`: die Gruppen, in denen dieses Sprite steckt.
    /// Ein Woerterbuch, weil pygame eines nimmt - damit stimmen
    /// Reihenfolge (Einfuegereihenfolge) und Eindeutigkeit von selbst.
    g: Py<PyDict>,
    #[pyo3(get, set)]
    image: Option<Py<PyAny>>,
    #[pyo3(get, set)]
    rect: Option<Py<PyAny>>,
}

impl PySprite {
    fn leer(py: Python<'_>) -> PySprite {
        PySprite {
            g: PyDict::new(py).unbind(),
            image: None,
            rect: None,
        }
    }
}

#[pymethods]
impl PySprite {
    #[new]
    #[classmethod]
    #[pyo3(signature = (*groups))]
    fn new(
        cls: &Bound<'_, PyType>,
        groups: &Bound<'_, PyTuple>,
    ) -> PyResult<PyClassInitializer<PySprite>> {
        let py = cls.py();
        if !cls.is(py.get_type::<PySprite>()) {
            // Unterklasse: die Argumente gehoeren ihr, nicht uns.
            return Ok(PyClassInitializer::from(PySprite::leer(py)));
        }
        let obj = Py::new(py, PySprite::leer(py))?;
        if !groups.is_empty() {
            obj.bind(py).as_any().call_method1("add", groups)?;
        }
        Ok(PyClassInitializer::from(obj))
    }

    /// `Sprite.__init__(*groups)`.
    ///
    /// Gebraucht wird es von Unterklassen (`super().__init__(gruppe)`);
    /// beim Grundtyp hat `__new__` die Arbeit schon getan.
    #[pyo3(signature = (*groups))]
    fn __init__(slf: &Bound<'_, PySprite>, groups: &Bound<'_, PyTuple>) -> PyResult<()> {
        {
            let mut k = slf.borrow_mut();
            k.g = PyDict::new(slf.py()).unbind();
            k.image = None;
            k.rect = None;
        }
        if !groups.is_empty() {
            slf.as_any().call_method1("add", groups)?;
        }
        Ok(())
    }

    /// `sprite.add(*groups)` - haengt das Sprite in Gruppen ein.
    ///
    /// Was kein `_spritegroup` hat, wird als Folge von Gruppen
    /// ausgepackt; so schluckt pygame auch `sprite.add([g1, g2])`
    /// (gemessen).
    #[pyo3(signature = (*groups))]
    fn add(slf: &Bound<'_, PySprite>, groups: &Bound<'_, PyTuple>) -> PyResult<()> {
        let py = slf.py();
        let s = slf.as_any();
        let g = slf.borrow().g.clone_ref(py);
        let g = g.bind(py);
        for group in groups.iter() {
            if group.hasattr("_spritegroup")? {
                if !g.contains(&group)? {
                    group.call_method1("add_internal", (s,))?;
                    s.call_method1("add_internal", (&group,))?;
                }
            } else {
                let teile: Vec<Bound<'_, PyAny>> = group.try_iter()?.collect::<PyResult<_>>()?;
                s.call_method1("add", PyTuple::new(py, teile)?)?;
            }
        }
        Ok(())
    }

    /// `sprite.remove(*groups)`.
    #[pyo3(signature = (*groups))]
    fn remove(slf: &Bound<'_, PySprite>, groups: &Bound<'_, PyTuple>) -> PyResult<()> {
        let py = slf.py();
        let s = slf.as_any();
        let g = slf.borrow().g.clone_ref(py);
        let g = g.bind(py);
        for group in groups.iter() {
            if group.hasattr("_spritegroup")? {
                if g.contains(&group)? {
                    group.call_method1("remove_internal", (s,))?;
                    s.call_method1("remove_internal", (&group,))?;
                }
            } else {
                let teile: Vec<Bound<'_, PyAny>> = group.try_iter()?.collect::<PyResult<_>>()?;
                s.call_method1("remove", PyTuple::new(py, teile)?)?;
            }
        }
        Ok(())
    }

    /// Traegt die Gruppe nur auf der Sprite-Seite ein.
    fn add_internal(slf: &Bound<'_, PySprite>, group: &Bound<'_, PyAny>) -> PyResult<()> {
        let py = slf.py();
        let g = slf.borrow().g.clone_ref(py);
        g.bind(py).set_item(group, 0)
    }

    /// Streicht die Gruppe nur auf der Sprite-Seite.
    ///
    /// Eine Gruppe, in der das Sprite nicht steckt, gibt `KeyError` -
    /// gemessen, `del self.__g[group]`.
    fn remove_internal(slf: &Bound<'_, PySprite>, group: &Bound<'_, PyAny>) -> PyResult<()> {
        let py = slf.py();
        let g = slf.borrow().g.clone_ref(py);
        g.bind(py).del_item(group)
    }

    /// Der Haken zum Ueberschreiben - hier absichtlich leer.
    #[pyo3(signature = (*args, **kwargs))]
    fn update(
        &self,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        let _ = (args, kwargs);
        Ok(())
    }

    /// `sprite.kill()` - raus aus allen Gruppen.
    ///
    /// Das Sprite selbst bleibt heil und darf danach wieder eingehaengt
    /// werden (gemessen).
    fn kill(slf: &Bound<'_, PySprite>) -> PyResult<()> {
        let py = slf.py();
        let g = slf.borrow().g.clone_ref(py);
        let g = g.bind(py);
        let gruppen: Vec<Bound<'_, PyAny>> = g.keys().iter().collect();
        for gruppe in gruppen {
            gruppe.call_method1("remove_internal", (slf,))?;
        }
        g.clear();
        Ok(())
    }

    /// `sprite.groups()`.
    fn groups<'py>(slf: &Bound<'py, PySprite>) -> PyResult<Bound<'py, PyList>> {
        let py = slf.py();
        let g = slf.borrow().g.clone_ref(py);
        Ok(g.bind(py).keys())
    }

    /// `sprite.alive()` - steckt es in mindestens einer Gruppe?
    fn alive(slf: &Bound<'_, PySprite>) -> bool {
        let py = slf.py();
        let g = slf.borrow().g.clone_ref(py);
        !g.bind(py).is_empty()
    }

    fn __repr__(slf: &Bound<'_, PySprite>) -> PyResult<String> {
        let py = slf.py();
        let n = slf.borrow().g.bind(py).len();
        Ok(format!(
            "<{} Sprite(in {n} groups)>",
            klassenname(slf.as_any())?
        ))
    }

    /// `sprite.layer` - lesbar nur, wenn `_layer` da ist.
    ///
    /// Gemessen: ohne `_layer` wirft der Zugriff `AttributeError`, und
    /// **das ist verhaltensrelevant** - `LayeredUpdates.add_internal`
    /// entscheidet daran, ob es die Vorgabeebene anschreibt.
    #[getter]
    fn get_layer<'py>(slf: &Bound<'py, PySprite>) -> PyResult<Bound<'py, PyAny>> {
        slf.as_any().getattr("_layer")
    }

    /// `sprite.layer = wert` - nur, solange das Sprite in keiner Gruppe
    /// ist.
    #[setter]
    fn set_layer(slf: &Bound<'_, PySprite>, value: &Bound<'_, PyAny>) -> PyResult<()> {
        if !PySprite::alive(slf) {
            slf.as_any().setattr("_layer", value)
        } else {
            Err(PyAttributeError::new_err(
                "Can't set layer directly after adding to group. Use \
                 group.change_layer(sprite, new_layer) instead.",
            ))
        }
    }
}

// ---------------------------------------------------------------------------
//  DirtySprite
// ---------------------------------------------------------------------------

/// Eine Ganzzahl als Python-Objekt - fuer die blanken Faecher.
///
/// Die Sprite-Felder liegen als `Py<PyAny>`, weil pygames `sprite.py`
/// reines Python ist; ihre Vorgaben (`dirty = 1`, `blendmode = 0`)
/// brauchen deshalb ein Objekt und keine Zahl.
fn zahl(py: Python<'_>, v: i64) -> Py<PyAny> {
    // `unwrap`: eine Ganzzahl in ein Python-`int` zu wandeln kann nur an
    // fehlendem Speicher scheitern, und dann ist der Interpreter ohnehin
    // zu Ende.
    v.into_pyobject(py)
        .map(|o| o.into_any().unbind())
        .expect("int")
}

/// `pygame.sprite.DirtySprite` - ein Sprite mit `dirty`, `visible`,
/// `blendmode` und `source_rect`.
///
/// `_layer` faengt bei `0` an, und das ist kein Beiwerk: es entscheidet
/// in [`PyLayeredUpdates`], ob ein ausdrueckliches `layer=` am Sprite
/// haengen bleibt (gemessen - bei einem einfachen `Sprite` bleibt es
/// nicht haengen, bei einem `DirtySprite` schon).
#[pyclass(name = "DirtySprite", module = "rustgame_py.sprite",
          extends = PySprite, subclass)]
pub struct PyDirtySprite {
    /// `0` sauber, `1` einmal neu zeichnen, `2` jedes Bild neu zeichnen.
    ///
    /// **Ein Python-Objekt, keine Zahl** - und das ist kein Versehen.
    /// pygames `sprite.py` ist reines Python, `self.dirty = 1` also ein
    /// Fach ohne jede Pruefung: gemessen geht dort `s.dirty = 'x'`,
    /// `= 1.5`, `= []` und `= 10**30`. Mit `i64` wies PyO3 all das
    /// zurueck - bei `10**30` sogar mit `OverflowError`, wo pygame die
    /// Zahl einfach ablegt. Gelesen wird `dirty` von `LayeredDirty.draw`
    /// ueber `getattr(...).extract::<i64>()`, und genau dort wirft pygame
    /// seinen Fehler auch.
    #[pyo3(get, set)]
    dirty: Py<PyAny>,
    /// `special_flags` fuer den Blit - wie `dirty` ein blankes Fach.
    #[pyo3(get, set)]
    blendmode: Py<PyAny>,
    #[pyo3(get, set)]
    _visible: Py<PyAny>,
    #[pyo3(get, set)]
    source_rect: Option<Py<PyAny>>,
}

impl PyDirtySprite {
    fn leer(py: Python<'_>) -> PyDirtySprite {
        PyDirtySprite {
            dirty: zahl(py, 1),
            blendmode: zahl(py, 0),
            _visible: zahl(py, 1),
            source_rect: None,
        }
    }

    fn bauplan(py: Python<'_>) -> PyClassInitializer<PyDirtySprite> {
        PyClassInitializer::from(PySprite::leer(py)).add_subclass(PyDirtySprite::leer(py))
    }
}

#[pymethods]
impl PyDirtySprite {
    #[new]
    #[classmethod]
    #[pyo3(signature = (*groups))]
    fn new(
        cls: &Bound<'_, PyType>,
        groups: &Bound<'_, PyTuple>,
    ) -> PyResult<PyClassInitializer<PyDirtySprite>> {
        let py = cls.py();
        if !cls.is(py.get_type::<PyDirtySprite>()) {
            return Ok(PyDirtySprite::bauplan(py));
        }
        let obj = Py::new(py, PyDirtySprite::bauplan(py))?;
        PyDirtySprite::__init__(obj.bind(py), groups)?;
        Ok(PyClassInitializer::from(obj))
    }

    /// `DirtySprite.__init__(*groups)`.
    ///
    /// `self._layer = getattr(self, "_layer", 0)` - eine Unterklasse darf
    /// also eine andere Vorgabeebene mitbringen, und die bleibt stehen.
    #[pyo3(signature = (*groups))]
    fn __init__(slf: &Bound<'_, PyDirtySprite>, groups: &Bound<'_, PyTuple>) -> PyResult<()> {
        {
            let py = slf.py();
            let mut k = slf.borrow_mut();
            k.dirty = zahl(py, 1);
            k.blendmode = zahl(py, 0);
            k._visible = zahl(py, 1);
            k.source_rect = None;
        }
        let s = slf.as_any();
        let ebene = match s.getattr("_layer") {
            Ok(v) => v,
            Err(e) if e.is_instance_of::<PyAttributeError>(slf.py()) => {
                0i64.into_pyobject(slf.py())?.into_any()
            }
            Err(e) => return Err(e),
        };
        s.setattr("_layer", ebene)?;
        let grund = slf.as_super();
        PySprite::__init__(grund, groups)
    }

    /// `sprite.visible` - `0` versteckt, `1` sichtbar.
    #[getter]
    fn get_visible(&self, py: Python<'_>) -> Py<PyAny> {
        self._visible.clone_ref(py)
    }

    /// Macht das Sprite nebenbei schmutzig; sonst bliebe das alte Bild
    /// stehen. Gemessen: nach `visible = 0` steht `dirty` auf `1`.
    ///
    /// **Der Vergleich kann scheitern, und das gehoert dazu.** pygame
    /// schreibt hier `if self.dirty < 2: self.dirty = 1`, und nach
    /// `s.dirty = 'x'` gibt genau diese Zeile
    /// `TypeError: '<' not supported between instances of 'str' and 'int'`
    /// - gemessen. Darum wird ueber Python verglichen und nicht in Rust:
    /// derselbe Satz aus derselben Stelle.
    #[setter]
    fn set_visible(&mut self, py: Python<'_>, value: Py<PyAny>) -> PyResult<()> {
        self._visible = value;
        if self.dirty.bind(py).lt(2i64)? {
            self.dirty = zahl(py, 1);
        }
        Ok(())
    }

    fn __repr__(slf: &Bound<'_, PyDirtySprite>) -> PyResult<String> {
        let n = slf.as_any().call_method0("groups")?.len()?;
        Ok(format!(
            "<{} DirtySprite(in {n} groups)>",
            klassenname(slf.as_any())?
        ))
    }
}

// ---------------------------------------------------------------------------
//  Die gemeinsame Buchfuehrung der Gruppen
// ---------------------------------------------------------------------------

/// Ruft `self.add_internal(sprite)` beziehungsweise
/// `self.add_internal(sprite, layer)` - immer ueber Python, damit eine
/// Unterklasse (und `LayeredUpdates`) ihre eigene Fassung bekommt.
fn add_internal_rufen(
    slf: &Bound<'_, PyAny>,
    sprite: &Bound<'_, PyAny>,
    layer: Option<&Bound<'_, PyAny>>,
) -> PyResult<()> {
    match layer {
        None => slf.call_method1("add_internal", (sprite,))?,
        Some(l) => slf.call_method1("add_internal", (sprite, l))?,
    };
    Ok(())
}

/// Steckt das Sprite schon drin?
fn drin(slf: &Bound<'_, PyAny>, sprite: &Bound<'_, PyAny>) -> PyResult<bool> {
    slf.call_method1("has_internal", (sprite,))?.is_truthy()
}

/// `AbstractGroup.add` und `LayeredUpdates.add` in einem.
///
/// `layer` ist `None` fuer die einfachen Gruppen; die Ebenengruppen
/// reichen `kwargs["layer"]` durch, und zwar **auch in die Rekursion**
/// (`self.add(*sprite, **kwargs)`).
fn gruppe_add(
    slf: &Bound<'_, PyAny>,
    sprites: &Bound<'_, PyTuple>,
    kwargs: Option<&Bound<'_, PyDict>>,
    layer: Option<&Bound<'_, PyAny>>,
) -> PyResult<()> {
    let py = slf.py();
    for sprite in sprites.iter() {
        if ist_sprite(&sprite) {
            if !drin(slf, &sprite)? {
                add_internal_rufen(slf, &sprite, layer)?;
                sprite.call_method1("add_internal", (slf,))?;
            }
            continue;
        }
        // Kein Sprite: erst als Folge auspacken, und **jeder** Fehler
        // daraus faellt in denselben Rettungsweg wie in pygame.
        let versuch = (|| -> PyResult<()> {
            let teile: Vec<Bound<'_, PyAny>> = sprite.try_iter()?.collect::<PyResult<_>>()?;
            slf.call_method("add", PyTuple::new(py, teile)?, kwargs)?;
            Ok(())
        })();
        if let Err(e) = versuch {
            if !ist_enten_fehler(&e, py) {
                return Err(e);
            }
            if sprite.hasattr("_spritegroup")? {
                for spr in sprites_von(&sprite)? {
                    if !drin(slf, &spr)? {
                        add_internal_rufen(slf, &spr, layer)?;
                        spr.call_method1("add_internal", (slf,))?;
                    }
                }
            } else if !drin(slf, &sprite)? {
                add_internal_rufen(slf, &sprite, layer)?;
                sprite.call_method1("add_internal", (slf,))?;
            }
        }
    }
    Ok(())
}

/// `AbstractGroup.remove`.
fn gruppe_remove(slf: &Bound<'_, PyAny>, sprites: &Bound<'_, PyTuple>) -> PyResult<()> {
    let py = slf.py();
    for sprite in sprites.iter() {
        if ist_sprite(&sprite) {
            if drin(slf, &sprite)? {
                slf.call_method1("remove_internal", (&sprite,))?;
                sprite.call_method1("remove_internal", (slf,))?;
            }
            continue;
        }
        let versuch = (|| -> PyResult<()> {
            let teile: Vec<Bound<'_, PyAny>> = sprite.try_iter()?.collect::<PyResult<_>>()?;
            slf.call_method1("remove", PyTuple::new(py, teile)?)?;
            Ok(())
        })();
        if let Err(e) = versuch {
            if !ist_enten_fehler(&e, py) {
                return Err(e);
            }
            if sprite.hasattr("_spritegroup")? {
                for spr in sprites_von(&sprite)? {
                    if drin(slf, &spr)? {
                        slf.call_method1("remove_internal", (&spr,))?;
                        spr.call_method1("remove_internal", (slf,))?;
                    }
                }
            } else if drin(slf, &sprite)? {
                slf.call_method1("remove_internal", (&sprite,))?;
                sprite.call_method1("remove_internal", (slf,))?;
            }
        }
    }
    Ok(())
}

/// `AbstractGroup.has` - sind **alle** genannten Sprites drin?
fn gruppe_has(slf: &Bound<'_, PyAny>, sprites: &Bound<'_, PyTuple>) -> PyResult<bool> {
    let py = slf.py();
    if sprites.is_empty() {
        return Ok(false);
    }
    for sprite in sprites.iter() {
        if ist_sprite(&sprite) {
            if !drin(slf, &sprite)? {
                return Ok(false);
            }
            continue;
        }
        let versuch = (|| -> PyResult<bool> {
            let teile: Vec<Bound<'_, PyAny>> = sprite.try_iter()?.collect::<PyResult<_>>()?;
            slf.call_method1("has", PyTuple::new(py, teile)?)?.is_truthy()
        })();
        match versuch {
            Ok(true) => continue,
            Ok(false) => return Ok(false),
            Err(e) if ist_enten_fehler(&e, py) => {
                if sprite.hasattr("_spritegroup")? {
                    for spr in sprites_von(&sprite)? {
                        if !drin(slf, &spr)? {
                            return Ok(false);
                        }
                    }
                } else if !drin(slf, &sprite)? {
                    return Ok(false);
                }
            }
            Err(e) => return Err(e),
        }
    }
    Ok(true)
}

/// `AbstractGroup.remove_internal` - der Woerterbuchteil, den auch
/// `GroupSingle` ausdruecklich aufruft.
fn eintrag_streichen(slf: &Bound<'_, PyAny>, sprite: &Bound<'_, PyAny>) -> PyResult<()> {
    let sd = spritedict(slf)?;
    let alt = match sd.get_item(sprite)? {
        Some(v) => v,
        None => return Err(PyKeyError::new_err(sprite.clone().unbind())),
    };
    if alt.is_truthy()? {
        lostsprites(slf)?.append(&alt)?;
    }
    sd.del_item(sprite)
}

/// Blittet ein Sprite und gibt das Rechteck zurueck, das `blit` liefert.
fn blit_sprite<'py>(
    surface: &Bound<'py, PyAny>,
    spr: &Bound<'py, PyAny>,
    area: Option<Bound<'py, PyAny>>,
    flags: i64,
) -> PyResult<Bound<'py, PyAny>> {
    let py = surface.py();
    let bereich = match area {
        Some(a) => a,
        None => py.None().into_bound(py),
    };
    surface.call_method1(
        "blit",
        (spr.getattr("image")?, spr.getattr("rect")?, bereich, flags),
    )
}

// ---------------------------------------------------------------------------
//  AbstractGroup
// ---------------------------------------------------------------------------

/// `pygame.sprite.AbstractGroup` - der Rumpf aller Gruppen.
///
/// `spritedict` bildet Sprite auf das zuletzt **gezeichnete** Rechteck ab
/// (`None`, solange nichts gezeichnet wurde). Pythons Woerterbuecher
/// merken sich die Einfuegereihenfolge, und `sprites()` ist
/// `list(self.spritedict)` - damit ist **jede** Gruppe geordnet, auch
/// wenn pygames eigener Text noch "in no particular order" behauptet.
/// Gemessen: nach `remove(b); add(b)` steht `b` hinten.
#[pyclass(name = "AbstractGroup", module = "rustgame_py.sprite", subclass)]
pub struct PyAbstractGroup {
    /// Wie bei [`PyDirtySprite::dirty`] ein **blankes Fach**: pygame
    /// nimmt `g.spritedict = 5` an, und erst das naechste `add()`
    /// beschwert sich. Mit `Py<PyDict>` wies PyO3 schon die Zuweisung
    /// zurueck.
    #[pyo3(get, set)]
    spritedict: Py<PyAny>,
    #[pyo3(get, set)]
    lostsprites: Py<PyAny>,
}

impl PyAbstractGroup {
    fn leer(py: Python<'_>) -> PyAbstractGroup {
        PyAbstractGroup {
            spritedict: PyDict::new(py).into_any().unbind(),
            lostsprites: PyList::empty(py).into_any().unbind(),
        }
    }

    /// `AbstractGroup.__init__` - frische Buecher.
    fn zuruecksetzen(slf: &Bound<'_, PyAny>) -> PyResult<()> {
        let py = slf.py();
        slf.setattr("spritedict", PyDict::new(py))?;
        slf.setattr("lostsprites", PyList::empty(py))?;
        Ok(())
    }
}

#[pymethods]
impl PyAbstractGroup {
    #[new]
    #[classmethod]
    fn new(cls: &Bound<'_, PyType>) -> PyAbstractGroup {
        PyAbstractGroup::leer(cls.py())
    }

    fn __init__(slf: &Bound<'_, PyAbstractGroup>) -> PyResult<()> {
        PyAbstractGroup::zuruecksetzen(slf.as_any())
    }

    /// Das Erkennungszeichen, an dem `Sprite.add` eine Gruppe von einer
    /// Liste unterscheidet.
    #[classattr]
    fn _spritegroup() -> bool {
        true
    }

    /// `group.sprites()` - in Zeichenreihenfolge.
    fn sprites<'py>(slf: &Bound<'py, PyAbstractGroup>) -> PyResult<Bound<'py, PyList>> {
        Ok(spritedict(slf.as_any())?.keys())
    }

    /// `group.add_internal(sprite, layer=None)`.
    ///
    /// `layer` wird hier **nicht** ausgewertet; die Stelle gibt es nur,
    /// weil alte Unterklassen sie ueberschreiben.
    #[pyo3(signature = (sprite, layer=None))]
    fn add_internal(
        slf: &Bound<'_, PyAbstractGroup>,
        sprite: &Bound<'_, PyAny>,
        layer: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        let _ = layer;
        spritedict(slf.as_any())?.set_item(sprite, slf.py().None())
    }

    /// `group.remove_internal(sprite)`.
    ///
    /// Das zuletzt gezeichnete Rechteck wandert nach `lostsprites` -
    /// aber nur, wenn es **wahr** ist. `bool(Rect(0, 0, -3, 4))` ist
    /// dabei `True`, geprueft wird nur `w != 0 and h != 0` (gemessen).
    fn remove_internal(
        slf: &Bound<'_, PyAbstractGroup>,
        sprite: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        eintrag_streichen(slf.as_any(), sprite)
    }

    fn has_internal(slf: &Bound<'_, PyAbstractGroup>, sprite: &Bound<'_, PyAny>) -> PyResult<bool> {
        spritedict(slf.as_any())?.contains(sprite)
    }

    /// `group.copy()` - neue Gruppe derselben Art, dieselben Sprites.
    ///
    /// Die Sprites stecken danach in **beiden** Gruppen. Die Kopie
    /// entsteht ueber `Klasse(sprites())`, also **ohne** die
    /// Einstellungen der Vorlage - bei den Ebenengruppen kostet das die
    /// Ebenen (gemessen: `copy().layers()` meldet `[0]`).
    fn copy<'py>(slf: &Bound<'py, PyAbstractGroup>) -> PyResult<Bound<'py, PyAny>> {
        let sprites = slf.as_any().call_method0("sprites")?;
        slf.get_type().call1((sprites,))
    }

    fn __iter__(slf: &Bound<'_, PyAbstractGroup>) -> PyResult<Py<PyAny>> {
        Ok(slf
            .as_any()
            .call_method0("sprites")?
            .try_iter()?
            .into_any()
            .unbind())
    }

    fn __contains__(slf: &Bound<'_, PyAbstractGroup>, sprite: &Bound<'_, PyAny>) -> PyResult<bool> {
        slf.as_any()
            .call_method1("has", (sprite,))?
            .is_truthy()
    }

    /// `group.add(sprite, liste, gruppe, ...)`.
    #[pyo3(signature = (*sprites))]
    fn add(slf: &Bound<'_, PyAbstractGroup>, sprites: &Bound<'_, PyTuple>) -> PyResult<()> {
        gruppe_add(slf.as_any(), sprites, None, None)
    }

    #[pyo3(signature = (*sprites))]
    fn remove(slf: &Bound<'_, PyAbstractGroup>, sprites: &Bound<'_, PyTuple>) -> PyResult<()> {
        gruppe_remove(slf.as_any(), sprites)
    }

    /// `group.has(...)` - ohne Argumente `False` (gemessen).
    #[pyo3(signature = (*sprites))]
    fn has(slf: &Bound<'_, PyAbstractGroup>, sprites: &Bound<'_, PyTuple>) -> PyResult<bool> {
        gruppe_has(slf.as_any(), sprites)
    }

    /// `group.update(*args, **kwargs)` - reicht alles unveraendert an
    /// jedes Sprite durch (gemessen).
    #[pyo3(signature = (*args, **kwargs))]
    fn update(
        slf: &Bound<'_, PyAbstractGroup>,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        for sprite in sprites_von(slf.as_any())? {
            sprite.call_method("update", args.clone(), kwargs)?;
        }
        Ok(())
    }

    /// `group.draw(surface, bgd=None, special_flags=0)`.
    ///
    /// **Gibt immer eine leere Liste zurueck** - der Quelltext liest sich
    /// anders, tut aber genau das: `self.lostsprites = []; dirty =
    /// self.lostsprites; return dirty` zeigt auf die *neue* leere Liste.
    /// Gemessen, und `g.draw(s) is g.lostsprites` ist danach `True`.
    #[pyo3(signature = (surface, bgd=None, special_flags=0))]
    fn draw<'py>(
        slf: &Bound<'py, PyAbstractGroup>,
        surface: &Bound<'py, PyAny>,
        bgd: Option<&Bound<'py, PyAny>>,
        special_flags: i64,
    ) -> PyResult<Bound<'py, PyList>> {
        // `bgd` benutzt erst `LayeredDirty`; hier steht es nur, damit die
        // Stelle dieselbe ist.
        let _ = bgd;
        let py = slf.py();
        let s = slf.as_any();
        let sprites = sprites_von(s)?;
        let sd = spritedict(s)?;
        if surface.hasattr("blits")? {
            let mut auftrag = Vec::with_capacity(sprites.len());
            for spr in &sprites {
                auftrag.push((
                    spr.getattr("image")?,
                    spr.getattr("rect")?,
                    py.None(),
                    special_flags,
                ));
            }
            let erg = surface.call_method1("blits", (PyList::new(py, auftrag)?,))?;
            for (spr, r) in sprites.iter().zip(erg.try_iter()?) {
                sd.set_item(spr, r?)?;
            }
        } else {
            for spr in &sprites {
                let r = blit_sprite(surface, spr, None, special_flags)?;
                sd.set_item(spr, r)?;
            }
        }
        let neu = PyList::empty(py);
        s.setattr("lostsprites", &neu)?;
        Ok(neu)
    }

    /// `group.clear(surface, bgd)` - uebermalt die alten Stellen.
    ///
    /// Erst die verlorenen Rechtecke, dann die zuletzt gezeichneten; die
    /// Reihenfolge ist gemessen. `lostsprites` wird dabei **nicht**
    /// geleert - das macht erst der naechste `draw` (auch gemessen).
    fn clear(
        slf: &Bound<'_, PyAbstractGroup>,
        surface: &Bound<'_, PyAny>,
        bgd: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let s = slf.as_any();
        let verloren: Vec<Bound<'_, PyAny>> = lostsprites(s)?.iter().collect();
        let gezeichnet: Vec<Bound<'_, PyAny>> =
            spritedict(s)?.iter().map(|(_, v)| v).collect();
        if bgd.is_callable() {
            for r in verloren {
                bgd.call1((surface, r))?;
            }
            for r in gezeichnet {
                if r.is_truthy()? {
                    bgd.call1((surface, r))?;
                }
            }
        } else {
            for r in verloren {
                surface.call_method1("blit", (bgd, &r, &r))?;
            }
            for r in gezeichnet {
                if r.is_truthy()? {
                    surface.call_method1("blit", (bgd, &r, &r))?;
                }
            }
        }
        Ok(())
    }

    /// `group.empty()` - alle Sprites raus, die Gruppe bleibt.
    fn empty(slf: &Bound<'_, PyAbstractGroup>) -> PyResult<()> {
        let s = slf.as_any();
        for sprite in sprites_von(s)? {
            s.call_method1("remove_internal", (&sprite,))?;
            sprite.call_method1("remove_internal", (s,))?;
        }
        Ok(())
    }

    fn __bool__(slf: &Bound<'_, PyAbstractGroup>) -> PyResult<bool> {
        Ok(!sprites_von(slf.as_any())?.is_empty())
    }

    fn __len__(slf: &Bound<'_, PyAbstractGroup>) -> PyResult<usize> {
        Ok(sprites_von(slf.as_any())?.len())
    }

    fn __repr__(slf: &Bound<'_, PyAbstractGroup>) -> PyResult<String> {
        let n = slf.as_any().len()?;
        Ok(format!("<{}({n} sprites)>", klassenname(slf.as_any())?))
    }
}

// ---------------------------------------------------------------------------
//  Group und die drei veralteten Zweitnamen
// ---------------------------------------------------------------------------

/// `pygame.sprite.Group` - der einfache Sprite-Behaelter.
#[pyclass(name = "Group", module = "rustgame_py.sprite",
          extends = PyAbstractGroup, subclass)]
pub struct PyGroup;

#[pymethods]
impl PyGroup {
    /// `Group[Sprite]` - ein Typhinweis, kein Zugriff.
    ///
    /// Gemessen: `sprite.Group[sprite.Sprite]` gibt ein
    /// `types.GenericAlias`. Ohne diese Methode steht dort
    /// `type 'Group' is not subscriptable`, und jede Datei mit
    /// `group: Group[Player]` in einer Typangabe scheitert beim Laden.
    #[classmethod]
    fn __class_getitem__(
        cls: &Bound<'_, pyo3::types::PyType>,
        item: &Bound<'_, PyAny>,
    ) -> PyResult<Py<PyAny>> {
        let py = cls.py();
        Ok(py
            .import("types")?
            .getattr("GenericAlias")?
            .call1((cls, item))?
            .unbind())
    }
}

/// Baut eine Gruppe mit Anfangsinhalt - siehe Modulkopf zum `cls`-Kniff.
fn gruppe_erzeugen<T>(
    cls: &Bound<'_, PyType>,
    eigen: &Bound<'_, PyType>,
    bauplan: PyClassInitializer<T>,
    sprites: &Bound<'_, PyTuple>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<PyClassInitializer<T>>
where
    T: PyClass,
{
    let py = cls.py();
    if !cls.is(eigen) {
        return Ok(bauplan);
    }
    let obj = Py::new(py, bauplan)?;
    let b = obj.bind(py).as_any();
    if !sprites.is_empty() || kwargs.is_some() {
        b.call_method("add", sprites.clone(), kwargs)?;
    }
    Ok(PyClassInitializer::from(obj))
}

#[pymethods]
impl PyGroup {
    #[new]
    #[classmethod]
    #[pyo3(signature = (*sprites))]
    fn new(
        cls: &Bound<'_, PyType>,
        sprites: &Bound<'_, PyTuple>,
    ) -> PyResult<PyClassInitializer<PyGroup>> {
        let py = cls.py();
        gruppe_erzeugen(
            cls,
            &py.get_type::<PyGroup>(),
            PyClassInitializer::from(PyAbstractGroup::leer(py)).add_subclass(PyGroup),
            sprites,
            None,
        )
    }

    #[pyo3(signature = (*sprites))]
    fn __init__(slf: &Bound<'_, PyGroup>, sprites: &Bound<'_, PyTuple>) -> PyResult<()> {
        PyAbstractGroup::zuruecksetzen(slf.as_any())?;
        slf.as_any().call_method1("add", sprites.clone())?;
        Ok(())
    }
}

/// `pygame.sprite.RenderPlain` - veraltete, verhaltensgleiche
/// Unterklasse von [`PyGroup`].
///
/// **Kein Zweitname**: gemessen ist `RenderPlain is Group` in pygame-ce
/// 2.5.7 `False`. Ein `add_class` unter zwei Namen waere hier also die
/// falsche Uebersetzung - `is` ist aus Python sichtbar.
#[pyclass(name = "RenderPlain", module = "rustgame_py.sprite",
          extends = PyGroup, subclass)]
pub struct PyRenderPlain;

/// Wirft die `DeprecationWarning`, die pygame im Konstruktor wirft.
fn veraltet(py: Python<'_>, text: &'static std::ffi::CStr) -> PyResult<()> {
    PyErr::warn(py, &py.get_type::<PyDeprecationWarning>().into_any(), text, 2)
}

#[pymethods]
impl PyRenderPlain {
    #[new]
    #[classmethod]
    #[pyo3(signature = (*sprites))]
    fn new(
        cls: &Bound<'_, PyType>,
        sprites: &Bound<'_, PyTuple>,
    ) -> PyResult<PyClassInitializer<PyRenderPlain>> {
        let py = cls.py();
        veraltet(
            py,
            pyo3::ffi::c_str!(
                "This class is deprecated and will be removed in a future version."
            ),
        )?;
        gruppe_erzeugen(
            cls,
            &py.get_type::<PyRenderPlain>(),
            PyClassInitializer::from(PyAbstractGroup::leer(py))
                .add_subclass(PyGroup)
                .add_subclass(PyRenderPlain),
            sprites,
            None,
        )
    }
}

/// `pygame.sprite.RenderClear` - wie [`PyRenderPlain`].
#[pyclass(name = "RenderClear", module = "rustgame_py.sprite",
          extends = PyGroup, subclass)]
pub struct PyRenderClear;

#[pymethods]
impl PyRenderClear {
    #[new]
    #[classmethod]
    #[pyo3(signature = (*sprites))]
    fn new(
        cls: &Bound<'_, PyType>,
        sprites: &Bound<'_, PyTuple>,
    ) -> PyResult<PyClassInitializer<PyRenderClear>> {
        let py = cls.py();
        veraltet(
            py,
            pyo3::ffi::c_str!(
                "This class is deprecated and will be removed in a future version."
            ),
        )?;
        gruppe_erzeugen(
            cls,
            &py.get_type::<PyRenderClear>(),
            PyClassInitializer::from(PyAbstractGroup::leer(py))
                .add_subclass(PyGroup)
                .add_subclass(PyRenderClear),
            sprites,
            None,
        )
    }
}

// ---------------------------------------------------------------------------
//  RenderUpdates
// ---------------------------------------------------------------------------

/// `pygame.sprite.RenderUpdates` - Gruppe, die die veraenderten Bereiche
/// mitfuehrt.
///
/// `draw` liefert je Sprite entweder die Vereinigung aus altem und neuem
/// Rechteck (wenn sie sich beruehren) oder beide einzeln - genau das,
/// was `display.update` braucht.
#[pyclass(name = "RenderUpdates", module = "rustgame_py.sprite",
          extends = PyGroup, subclass)]
pub struct PyRenderUpdates;

#[pymethods]
impl PyRenderUpdates {
    #[new]
    #[classmethod]
    #[pyo3(signature = (*sprites))]
    fn new(
        cls: &Bound<'_, PyType>,
        sprites: &Bound<'_, PyTuple>,
    ) -> PyResult<PyClassInitializer<PyRenderUpdates>> {
        let py = cls.py();
        gruppe_erzeugen(
            cls,
            &py.get_type::<PyRenderUpdates>(),
            PyClassInitializer::from(PyAbstractGroup::leer(py))
                .add_subclass(PyGroup)
                .add_subclass(PyRenderUpdates),
            sprites,
            None,
        )
    }

    /// `RenderUpdates.draw` - gibt die **alte** `lostsprites`-Liste
    /// zurueck; `group.lostsprites` ist danach eine neue, leere
    /// (gemessen).
    #[pyo3(signature = (surface, bgd=None, special_flags=0))]
    fn draw<'py>(
        slf: &Bound<'py, PyRenderUpdates>,
        surface: &Bound<'py, PyAny>,
        bgd: Option<&Bound<'py, PyAny>>,
        special_flags: i64,
    ) -> PyResult<Bound<'py, PyList>> {
        let _ = bgd;
        let py = slf.py();
        let s = slf.as_any();
        let dirty = lostsprites(s)?;
        s.setattr("lostsprites", PyList::empty(py))?;
        let sd = spritedict(s)?;
        for sprite in sprites_von(s)? {
            let alt = match sd.get_item(&sprite)? {
                Some(v) => v,
                None => return Err(PyKeyError::new_err(sprite.clone().unbind())),
            };
            let neu = blit_sprite(surface, &sprite, None, special_flags)?;
            if alt.is_truthy()? {
                if neu.call_method1("colliderect", (&alt,))?.is_truthy()? {
                    dirty.append(neu.call_method1("union", (&alt,))?)?;
                } else {
                    dirty.append(&neu)?;
                    dirty.append(&alt)?;
                }
            } else {
                dirty.append(&neu)?;
            }
            sd.set_item(&sprite, &neu)?;
        }
        Ok(dirty)
    }
}

/// `pygame.sprite.OrderedUpdates`.
///
/// Warnt und tut sonst nichts - ihre eigene Warnung sagt es: *"order of
/// sprites is now maintained in all sprite Group classes"*. Nachgemessen:
/// `OrderedUpdates.draw is RenderUpdates.draw` ist `True`, aber
/// `OrderedUpdates is RenderUpdates` ist `False`.
#[pyclass(name = "OrderedUpdates", module = "rustgame_py.sprite",
          extends = PyRenderUpdates, subclass)]
pub struct PyOrderedUpdates;

#[pymethods]
impl PyOrderedUpdates {
    #[new]
    #[classmethod]
    #[pyo3(signature = (*sprites))]
    fn new(
        cls: &Bound<'_, PyType>,
        sprites: &Bound<'_, PyTuple>,
    ) -> PyResult<PyClassInitializer<PyOrderedUpdates>> {
        let py = cls.py();
        veraltet(
            py,
            pyo3::ffi::c_str!(
                "OrderedUpdates is now just an alias to RenderUpdates, order of \
                 sprites is now maintained in all sprite Group classes. This \
                 class is deprecated and will be removed in a future version."
            ),
        )?;
        gruppe_erzeugen(
            cls,
            &py.get_type::<PyOrderedUpdates>(),
            PyClassInitializer::from(PyAbstractGroup::leer(py))
                .add_subclass(PyGroup)
                .add_subclass(PyRenderUpdates)
                .add_subclass(PyOrderedUpdates),
            sprites,
            None,
        )
    }
}

// ---------------------------------------------------------------------------
//  LayeredUpdates
// ---------------------------------------------------------------------------

/// `pygame.sprite.LayeredUpdates` - Gruppe mit Ebenen.
///
/// Gezeichnet wird von der untersten Ebene zur obersten, innerhalb einer
/// Ebene in der Reihenfolge des Einhaengens.
///
/// ## Die Ebenenwahl, und warum sie ueberrascht
///
/// `add_internal(sprite, layer)` kennt drei Faelle; zwei sind
/// naheliegend, der dritte nicht:
///
/// 1. **`layer` fehlt, Sprite hat `_layer`** -> die des Sprites.
/// 2. **`layer` fehlt, Sprite hat kein `_layer`** -> die Vorgabeebene,
///    und sie wird dem Sprite **angeschrieben**.
/// 3. **`layer` ist angegeben** -> diese Ebene; `_layer` wird dem Sprite
///    aber nur dann angeschrieben, wenn es das Attribut **schon hat**.
///
/// Fall 3 heisst: ein einfaches `Sprite`, das mit `add(s, layer=5)`
/// eingehaengt wurde, hat danach **kein** `_layer` (gemessen:
/// `hasattr(s, "_layer")` ist `False`) - ein `DirtySprite` dagegen schon,
/// denn das bringt `_layer = 0` mit auf die Welt. Die Folge steht in
/// `copy()`: die Kopie verliert die Ebenen solcher Sprites, weil sie bei
/// `Klasse(sprites())` wieder in Fall 2 laufen (gemessen: `[0]`).
#[pyclass(name = "LayeredUpdates", module = "rustgame_py.sprite",
          extends = PyAbstractGroup, subclass)]
pub struct PyLayeredUpdates {
    /// `_spritelayers`: Sprite -> Ebene.
    #[pyo3(get, set)]
    _spritelayers: Py<PyDict>,
    /// `_spritelist`: alle Sprites, nach Ebene sortiert, hinten oben.
    #[pyo3(get, set)]
    _spritelist: Py<PyList>,
    #[pyo3(get, set)]
    _default_layer: i64,
}

impl PyLayeredUpdates {
    fn leer(py: Python<'_>, default_layer: i64) -> PyLayeredUpdates {
        PyLayeredUpdates {
            _spritelayers: PyDict::new(py).unbind(),
            _spritelist: PyList::empty(py).unbind(),
            _default_layer: default_layer,
        }
    }

    fn bauplan(py: Python<'_>, default_layer: i64) -> PyClassInitializer<PyLayeredUpdates> {
        PyClassInitializer::from(PyAbstractGroup::leer(py))
            .add_subclass(PyLayeredUpdates::leer(py, default_layer))
    }
}

/// `kwargs.get("default_layer", 0)`.
fn standardebene(kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<i64> {
    match kwargs {
        None => Ok(0),
        Some(k) => match k.get_item("default_layer")? {
            None => Ok(0),
            Some(v) => v.extract(),
        },
    }
}

/// `kwargs["layer"] if "layer" in kwargs else None`.
fn ebenen_wunsch<'py>(kwargs: Option<&Bound<'py, PyDict>>) -> PyResult<Option<Bound<'py, PyAny>>> {
    match kwargs {
        None => Ok(None),
        Some(k) => k.get_item("layer"),
    }
}

/// `LayeredUpdates.change_layer` - von beiden Ebenengruppen benutzt.
///
/// Der Ablauf ist der von `add_internal`, nur andersherum angefangen:
/// erst aus Liste und Ebenenbuch heraus, dann ueber dieselbe Halbierung
/// wieder hinein. `_layer` wird dem Sprite nur angeschrieben, wenn es das
/// Attribut schon hat - dieselbe Regel wie in Fall 3.
fn ebene_wechseln(
    slf: &Bound<'_, PyAny>,
    sprite: &Bound<'_, PyAny>,
    new_layer: i64,
) -> PyResult<()> {
    let liste = spritelist(slf)?;
    let layers = spritelayers(slf)?;
    liste_entfernen(&liste, sprite)?;
    // `pop` vor der Suche: die Halbierung liest `_spritelayers`, und das
    // Sprite darf dabei nicht darin stehen.
    layers.del_item(sprite)?;
    let stelle = einfuegestelle(&liste, &layers, new_layer)?;
    liste.insert(stelle, sprite)?;
    if sprite.hasattr("_layer")? {
        sprite.setattr("_layer", new_layer)?;
    }
    layers.set_item(sprite, new_layer)
}

#[pymethods]
impl PyLayeredUpdates {
    /// Der Platzhalter in `spritedict`, solange noch nichts gezeichnet
    /// wurde. pygame vergleicht ihn mit `is`, nicht mit `==`, und genau
    /// deshalb ist es **ein** Objekt in der Klasse und nicht je Gruppe
    /// eines.
    #[classattr]
    fn _init_rect() -> PyRect {
        PyRect::neu(KernRect::new(0, 0, 0, 0))
    }

    #[new]
    #[classmethod]
    #[pyo3(signature = (*sprites, **kwargs))]
    fn new(
        cls: &Bound<'_, PyType>,
        sprites: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<PyClassInitializer<PyLayeredUpdates>> {
        let py = cls.py();
        let bauplan = PyLayeredUpdates::bauplan(py, standardebene(kwargs)?);
        if !cls.is(py.get_type::<PyLayeredUpdates>()) {
            return Ok(bauplan);
        }
        let obj = Py::new(py, bauplan)?;
        if !sprites.is_empty() {
            obj.bind(py)
                .as_any()
                .call_method("add", sprites.clone(), kwargs)?;
        }
        Ok(PyClassInitializer::from(obj))
    }

    #[pyo3(signature = (*sprites, **kwargs))]
    fn __init__(
        slf: &Bound<'_, PyLayeredUpdates>,
        sprites: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        let py = slf.py();
        let s = slf.as_any();
        s.setattr("_spritelayers", PyDict::new(py))?;
        s.setattr("_spritelist", PyList::empty(py))?;
        PyAbstractGroup::zuruecksetzen(s)?;
        s.setattr("_default_layer", standardebene(kwargs)?)?;
        if !sprites.is_empty() {
            s.call_method("add", sprites.clone(), kwargs)?;
        }
        Ok(())
    }

    /// `LayeredUpdates.add_internal(sprite, layer=None)` - siehe die drei
    /// Faelle im Klassenkopf.
    #[pyo3(signature = (sprite, layer=None))]
    fn add_internal(
        slf: &Bound<'_, PyLayeredUpdates>,
        sprite: &Bound<'_, PyAny>,
        layer: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        let py = slf.py();
        let s = slf.as_any();
        let platzhalter = s.getattr("_init_rect")?;
        spritedict(s)?.set_item(sprite, platzhalter)?;

        let ebene: i64 = match layer {
            None => match sprite.getattr("layer") {
                Ok(v) => v.extract()?,
                Err(e) if e.is_instance_of::<PyAttributeError>(py) => {
                    let l: i64 = s.getattr("_default_layer")?.extract()?;
                    sprite.setattr("_layer", l)?;
                    l
                }
                Err(e) => return Err(e),
            },
            Some(l) => {
                let wert: i64 = l.extract()?;
                if sprite.hasattr("_layer")? {
                    sprite.setattr("_layer", wert)?;
                }
                wert
            }
        };

        let liste = spritelist(s)?;
        let layers = spritelayers(s)?;
        layers.set_item(sprite, ebene)?;
        let stelle = einfuegestelle(&liste, &layers, ebene)?;
        liste.insert(stelle, sprite)
    }

    /// `group.add(*sprites, layer=n)`.
    #[pyo3(signature = (*sprites, **kwargs))]
    fn add(
        slf: &Bound<'_, PyLayeredUpdates>,
        sprites: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        if sprites.is_empty() {
            return Ok(());
        }
        let layer = ebenen_wunsch(kwargs)?;
        gruppe_add(slf.as_any(), sprites, kwargs, layer.as_ref())
    }

    /// `LayeredUpdates.remove_internal(sprite)`.
    ///
    /// Hier landen **zwei** Rechtecke in `lostsprites`, und das zweite
    /// ist `sprite.rect` **selbst**, nicht eine Kopie. Gemessen an einem
    /// Sprite bei `(34, 24, 8, 5)`, das nach dem Entfernen um `(-3, +2)`
    /// verschoben wird: pygame meldet danach `(31, 26, 8, 5)`. Beim
    /// einfachen `Group` ist es eine Kopie und bleibt `(34, 24, 8, 5)`.
    fn remove_internal(
        slf: &Bound<'_, PyLayeredUpdates>,
        sprite: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let s = slf.as_any();
        liste_entfernen(&spritelist(s)?, sprite)?;
        let sd = spritedict(s)?;
        let alt = match sd.get_item(sprite)? {
            Some(v) => v,
            None => return Err(PyKeyError::new_err(sprite.clone().unbind())),
        };
        let verloren = lostsprites(s)?;
        // **Gleichheit mit `_init_rect`**, nicht die Wahrheit des
        // Rechtecks: ein leeres, aber gezeichnetes Rechteck wandert sehr
        // wohl nach `lostsprites`.
        if !alt.is(&s.getattr("_init_rect")?) {
            verloren.append(&alt)?;
        }
        if sprite.hasattr("rect")? {
            verloren.append(sprite.getattr("rect")?)?;
        }
        sd.del_item(sprite)?;
        spritelayers(s)?.del_item(sprite)
    }

    /// `sprites()` - unterstes zuerst, oberstes zuletzt. Eine **Kopie**
    /// von `_spritelist` (gemessen: `g.sprites() is not g._spritelist`).
    fn sprites<'py>(slf: &Bound<'py, PyLayeredUpdates>) -> PyResult<Bound<'py, PyList>> {
        let liste = spritelist(slf.as_any())?;
        PyList::new(slf.py(), liste.iter())
    }

    /// `LayeredUpdates.draw` - wie `RenderUpdates.draw`, aber das alte
    /// Rechteck zaehlt schon, wenn es ueberhaupt eines gibt
    /// (`is not _init_rect`), auch ein leeres.
    #[pyo3(signature = (surface, bgd=None, special_flags=0))]
    fn draw<'py>(
        slf: &Bound<'py, PyLayeredUpdates>,
        surface: &Bound<'py, PyAny>,
        bgd: Option<&Bound<'py, PyAny>>,
        special_flags: i64,
    ) -> PyResult<Bound<'py, PyList>> {
        let _ = bgd;
        let py = slf.py();
        // **Erst die Flaeche fragen, dann rechnen.** pygames
        // `LayeredUpdates.draw` beginnt mit `surface_blit = surface.blit`,
        // und das wirft bei einer leeren Gruppe genauso: gemessen gibt
        // `LayeredUpdates().draw('x')` dort
        // `AttributeError: 'str' object has no attribute 'blit'`,
        // waehrend hier **gar nichts** geschah - der Durchlauf war leer,
        // also fiel der falsche Wert nie auf. Ein stillschweigend
        // angenommenes falsches Ziel ist schlimmer als ein Fehler.
        surface.getattr("blit")?;
        let s = slf.as_any();
        let dirty = lostsprites(s)?;
        s.setattr("lostsprites", PyList::empty(py))?;
        let sd = spritedict(s)?;
        let init_rect = s.getattr("_init_rect")?;
        for spr in sprites_von(s)? {
            let rec = match sd.get_item(&spr)? {
                Some(v) => v,
                None => return Err(PyKeyError::new_err(spr.clone().unbind())),
            };
            let neu = blit_sprite(surface, &spr, None, special_flags)?;
            if rec.is(&init_rect) {
                dirty.append(&neu)?;
            } else if neu.call_method1("colliderect", (&rec,))?.is_truthy()? {
                dirty.append(neu.call_method1("union", (&rec,))?)?;
            } else {
                dirty.append(&neu)?;
                dirty.append(&rec)?;
            }
            sd.set_item(&spr, &neu)?;
        }
        Ok(dirty)
    }

    /// `get_sprites_at(pos)` - unterstes zuerst.
    ///
    /// pygame reicht die Sprites unveraendert an `Rect.collidelistall`,
    /// das in pygame-ce Objekte mit `rect`-Attribut annimmt (gemessen:
    /// `Rect(5,5,1,1).collidelistall([sprite, ...])` gibt `[0]`). Der
    /// `rect_aus` dieser Huelle kann das nicht, darum wird hier je Sprite
    /// selbst gefragt - dasselbe Ergebnis, ein Umweg weniger.
    fn get_sprites_at<'py>(
        slf: &Bound<'py, PyLayeredUpdates>,
        pos: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyList>> {
        let py = slf.py();
        // pygame reicht den Wert an `Rect.collidelistall` weiter, und der
        // Satz kommt von dort: `Argument must be rect style object` -
        // gemessen. Hier stand der Satz von `punkt_aus`.
        let punkt = crate::umwandlung::punkt_aus(pos).map_err(|_| {
            pyo3::exceptions::PyTypeError::new_err("Argument must be rect style object")
        })?;
        let feld = KernRect::from_parts(punkt, (1, 1));
        let liste = spritelist(slf.as_any())?;
        let mut treffer = Vec::new();
        for spr in liste.iter() {
            if feld.colliderect(rect_von_sprite(&spr)?) {
                treffer.push(spr);
            }
        }
        PyList::new(py, treffer)
    }

    /// `get_sprite(idx)` - das Sprite an dieser Stelle der
    /// Zeichenreihenfolge.
    /// pygame schreibt `return self._spritelist[idx]`, und ein falscher
    /// Schluessel gibt darum **Pythons** Satz:
    /// `TypeError: list indices must be integers or slices, not str`.
    /// Mit `idx: isize` kam PyO3s Satz dazwischen - gemessen.
    fn get_sprite<'py>(
        slf: &Bound<'py, PyLayeredUpdates>,
        idx: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        spritelist(slf.as_any())?.as_any().get_item(idx)
    }

    /// Nimmt eine ganze Ebene heraus und gibt sie zurueck. Die Sprites
    /// sind danach in keiner Gruppe mehr (gemessen).
    /// `layer_nr` ist **kein `i64`**: pygames Fassung vergleicht nur,
    /// und gemessen gibt `remove_sprites_of_layer('x')` dort eine leere
    /// Liste statt eines Fehlers. Mit `i64` warf PyO3 vorher.
    fn remove_sprites_of_layer<'py>(
        slf: &Bound<'py, PyLayeredUpdates>,
        layer_nr: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyList>> {
        let s = slf.as_any();
        let sprites = s
            .call_method1("get_sprites_from_layer", (layer_nr,))?
            .downcast_into::<PyList>()?;
        s.call_method1("remove", PyTuple::new(slf.py(), sprites.iter())?)?;
        Ok(sprites)
    }

    /// `layers()` - die vorkommenden Ebenen, aufsteigend, ohne
    /// Wiederholung.
    fn layers(slf: &Bound<'_, PyLayeredUpdates>) -> PyResult<Vec<i64>> {
        let layers = spritelayers(slf.as_any())?;
        let mut gesehen = HashSet::new();
        let mut v = Vec::new();
        for (_, wert) in layers.iter() {
            let l: i64 = wert.extract()?;
            if gesehen.insert(l) {
                v.push(l);
            }
        }
        v.sort_unstable();
        Ok(v)
    }

    /// `change_layer(sprite, new_layer)`.
    ///
    /// Ein Sprite, das gar nicht in der Gruppe ist, laeuft in pygame in
    /// `list.remove(x): x not in list` - ein `ValueError` mitten aus
    /// einer Hilfsmethode heraus. Gemessen, weil der Text der Methode
    /// ausdruecklich sagt "This is not checked".
    fn change_layer(
        slf: &Bound<'_, PyLayeredUpdates>,
        sprite: &Bound<'_, PyAny>,
        new_layer: i64,
    ) -> PyResult<()> {
        ebene_wechseln(slf.as_any(), sprite, new_layer)
    }

    /// Fuer ein Sprite, das gar nicht drin ist, liefert pygame die
    /// **Vorgabeebene** statt eines Fehlers (gemessen).
    fn get_layer_of_sprite(
        slf: &Bound<'_, PyLayeredUpdates>,
        sprite: &Bound<'_, PyAny>,
    ) -> PyResult<i64> {
        let s = slf.as_any();
        match spritelayers(s)?.get_item(sprite)? {
            Some(v) => v.extract(),
            None => s.getattr("_default_layer")?.extract(),
        }
    }

    fn get_top_layer(slf: &Bound<'_, PyLayeredUpdates>) -> PyResult<i64> {
        let s = slf.as_any();
        let liste = spritelist(s)?;
        if liste.is_empty() {
            return Err(PyIndexError::new_err("list index out of range"));
        }
        ebene_von(&spritelayers(s)?, &liste.get_item(liste.len() - 1)?)
    }

    fn get_bottom_layer(slf: &Bound<'_, PyLayeredUpdates>) -> PyResult<i64> {
        let s = slf.as_any();
        let liste = spritelist(s)?;
        if liste.is_empty() {
            return Err(PyIndexError::new_err("list index out of range"));
        }
        ebene_von(&spritelayers(s)?, &liste.get_item(0)?)
    }

    /// Auf die oberste Ebene, dort ganz nach hinten - die Ebene des
    /// Sprites **aendert sich** dabei (gemessen an `[f1@0, f2@1, f3@1]`).
    fn move_to_front(slf: &Bound<'_, PyLayeredUpdates>, sprite: &Bound<'_, PyAny>) -> PyResult<()> {
        let s = slf.as_any();
        let oben = s.call_method0("get_top_layer")?;
        s.call_method1("change_layer", (sprite, oben))?;
        Ok(())
    }

    /// Auf eine **neue** Ebene unter der bisher untersten.
    fn move_to_back(slf: &Bound<'_, PyLayeredUpdates>, sprite: &Bound<'_, PyAny>) -> PyResult<()> {
        let s = slf.as_any();
        let unten: i64 = s.call_method0("get_bottom_layer")?.extract()?;
        s.call_method1("change_layer", (sprite, unten - 1))?;
        Ok(())
    }

    fn get_top_sprite<'py>(slf: &Bound<'py, PyLayeredUpdates>) -> PyResult<Bound<'py, PyAny>> {
        spritelist(slf.as_any())?.as_any().get_item(-1isize)
    }

    /// Alle Sprites einer Ebene, in der Reihenfolge des Einhaengens.
    ///
    /// **Der Schluessel darf alles sein.** pygame vergleicht ihn nur
    /// (`if layer == l: ...`), und ein Vergleich mit einer Zeichenkette
    /// ist in Python kein Fehler, sondern `False`: gemessen gibt
    /// `get_sprites_from_layer('x')` dort `[]`. Mit `layer: i64` warf
    /// PyO3, bevor ueberhaupt verglichen wurde.
    ///
    /// Das Abbrechen bleibt, aber nur fuer Zahlen: `>` ist zwischen
    /// `int` und `str` kein gueltiger Vergleich, und pygames Schleife
    /// laeuft an dieser Stelle ohnehin bis zum Ende.
    fn get_sprites_from_layer<'py>(
        slf: &Bound<'py, PyLayeredUpdates>,
        layer: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyList>> {
        let s = slf.as_any();
        let liste = spritelist(s)?;
        let layers = spritelayers(s)?;
        let zahl: Option<i64> = layer.extract().ok();
        let mut raus = Vec::new();
        for spr in liste.iter() {
            let e = ebene_von(&layers, &spr)?;
            match zahl {
                Some(l) if e == l => raus.push(spr),
                Some(l) if e > l => break,
                Some(_) => {}
                // Kein Zahlwert: nichts trifft, und abbrechen darf man
                // auch nicht.
                None => {}
            }
        }
        PyList::new(slf.py(), raus)
    }

    /// Tauscht die Bewohner zweier Ebenen.
    fn switch_layer(
        slf: &Bound<'_, PyLayeredUpdates>,
        layer1_nr: i64,
        layer2_nr: i64,
    ) -> PyResult<()> {
        let py = slf.py();
        let s = slf.as_any();
        let aus1 = s
            .call_method1("remove_sprites_of_layer", (layer1_nr,))?
            .downcast_into::<PyList>()?;
        let aus2 = s
            .call_method1("get_sprites_from_layer", (layer2_nr,))?
            .downcast_into::<PyList>()?;
        for spr in aus2.iter() {
            s.call_method1("change_layer", (spr, layer1_nr))?;
        }
        let kw = PyDict::new(py);
        kw.set_item("layer", layer2_nr)?;
        s.call_method("add", PyTuple::new(py, aus1.iter())?, Some(&kw))?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
//  LayeredDirty
// ---------------------------------------------------------------------------

/// `pygame.sprite.LayeredDirty` - zeichnet nur, was sich geaendert hat.
///
/// ## Zwei Betriebsarten, und der Umschalter dazwischen
///
/// Entweder **Vollbild** (alles neu) oder **Teilbild** (nur die
/// schmutzigen Rechtecke). Welche, entscheidet `_use_update`, und das
/// setzt sich nach jedem `draw` selbst neu: hat das Zeichnen laenger
/// gedauert als `_time_threshold` (Vorgabe `1000/80 = 12,5` ms), geht es
/// zurueck auf Vollbild, sonst auf Teilbild.
///
/// **Eine frische Gruppe faengt im Vollbildmodus an.** Der erste `draw`
/// liefert deshalb genau ein Rechteck - den Clip-Bereich -, und erst der
/// zweite die einzelnen schmutzigen Stellen. Gemessen:
///
/// ```text
/// 1. draw -> [Rect(0, 0, 100, 100)]
/// 2. draw -> [Rect(0, 0, 10, 10), Rect(40, 40, 10, 10)]
/// ```
///
/// **Das Ueberraschende:** im **Vollbildmodus** werden die
/// `dirty`-Flaggen **nicht** zurueckgesetzt. Nachgemessen - nach dem
/// ersten `draw` stehen beide Sprites weiterhin auf `dirty == 1`, erst
/// nach dem zweiten auf `0`. Wer das uebersieht, baut eine Nachbildung,
/// bei der der zweite `draw` nichts mehr findet.
#[pyclass(name = "LayeredDirty", module = "rustgame_py.sprite",
          extends = PyLayeredUpdates, subclass)]
pub struct PyLayeredDirty {
    #[pyo3(get, set)]
    _clip: Option<Py<PyAny>>,
    #[pyo3(get, set)]
    _use_update: bool,
    #[pyo3(get, set)]
    _time_threshold: f64,
    #[pyo3(get, set)]
    _bgd: Option<Py<PyAny>>,
}

impl PyLayeredDirty {
    fn leer() -> PyLayeredDirty {
        PyLayeredDirty {
            _clip: None,
            _use_update: false,
            // 1000.0 / 80.0 - gemessen als 12.5.
            _time_threshold: 1000.0 / 80.0,
            _bgd: None,
        }
    }

    fn bauplan(py: Python<'_>, default_layer: i64) -> PyClassInitializer<PyLayeredDirty> {
        PyLayeredUpdates::bauplan(py, default_layer).add_subclass(PyLayeredDirty::leer())
    }

    /// Die drei kwargs, die pygame nach dem Aufbau noch nachtraegt.
    fn einstellungen(slf: &Bound<'_, PyAny>, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
        let Some(k) = kwargs else { return Ok(()) };
        for name in ["_use_update", "_time_threshold", "_default_layer"] {
            if let Some(v) = k.get_item(name)? {
                slf.setattr(name, v)?;
            }
        }
        Ok(())
    }
}

/// `_find_dirty_area`.
///
/// Sammelt je schmutzigem Sprite **zwei** Rechtecke: wo es jetzt ist und
/// wo es zuletzt gezeichnet wurde. Jedes wird vorher mit allen schon
/// gesammelten verschmolzen, die es beruehrt - in einer Schleife, weil
/// die Vereinigung neue Beruehrungen schaffen kann. Das ist pygames
/// eigener Kommentar: *"can it be done better? because that is an
/// O(n**2) algorithm in worst case"*. Hier steht derselbe Algorithmus,
/// denn **Reihenfolge und Anzahl** der Rechtecke sind der Rueckgabewert.
fn finde_dirty(
    sprites: &[Bound<'_, PyAny>],
    sd: &Bound<'_, PyDict>,
    init_rect: &Bound<'_, PyAny>,
    clip: KernRect,
    update: &mut Vec<KernRect>,
) -> PyResult<()> {
    fn verschmelze(u: &mut KernRect, update: &mut Vec<KernRect>) {
        while let Some(i) = u.collidelist(update) {
            u.union_ip(update[i]);
            update.remove(i);
        }
    }
    for spr in sprites {
        if spr.getattr("dirty")?.extract::<i64>()? <= 0 {
            continue;
        }
        let rect = rect_von_sprite(spr)?;
        let quelle = spr.getattr("source_rect")?;
        // `if spr.source_rect:` - Wahrheit, nicht `is not None`. Ein
        // leerer Ausschnitt zaehlt hier also als "keiner".
        let mut u = if quelle.is_truthy()? {
            let sr = rect_aus(&quelle)?;
            KernRect::from_parts(rect.topleft(), (sr.w, sr.h))
        } else {
            rect
        };
        verschmelze(&mut u, update);
        update.push(u.clip(clip));

        let alt = match sd.get_item(spr)? {
            Some(v) => v,
            None => return Err(PyKeyError::new_err(spr.clone().unbind())),
        };
        if !alt.is(init_rect) {
            let mut u = rect_aus(&alt)?;
            verschmelze(&mut u, update);
            update.push(u.clip(clip));
        }
    }
    Ok(())
}

/// `_draw_dirty_internal`.
fn zeichne_dirty(
    sprites: &[Bound<'_, PyAny>],
    sd: &Bound<'_, PyDict>,
    surface: &Bound<'_, PyAny>,
    special_flags: Option<i64>,
    update: &[KernRect],
) -> PyResult<()> {
    for spr in sprites {
        let flags = match special_flags {
            Some(f) => f,
            None => spr.getattr("blendmode")?.extract::<i64>()?,
        };
        let dirty: i64 = spr.getattr("dirty")?.extract()?;
        let sichtbar = spr.getattr("visible")?.is_truthy()?;
        if dirty < 1 && sichtbar {
            // Sauberes Sprite: nur die Stuecke auffrischen, die andere
            // schmutzig gemacht haben. Genau das ist der Gewinn dieser
            // Gruppe.
            let bild = spr.getattr("image")?;
            let rect = rect_von_sprite(spr)?;
            let quelle = spr.getattr("source_rect")?;
            // Hier prueft pygame `is not None`, nicht die Wahrheit -
            // anders als in `_find_dirty_area` zwei Bildschirme weiter
            // oben. Der Unterschied ist echt und bleibt stehen.
            let (spr_rect, off_x, off_y) = if quelle.is_none() {
                (rect, -rect.x, -rect.y)
            } else {
                let sr = rect_aus(&quelle)?;
                let r = KernRect::from_parts(rect.topleft(), (sr.w, sr.h));
                (r, sr.x - r.x, sr.y - r.y)
            };
            for idx in spr_rect.collidelistall(update) {
                let c = spr_rect.clip(update[idx]);
                let aus = KernRect::new(c.x + off_x, c.y + off_y, c.w, c.h);
                surface.call_method1(
                    "blit",
                    (&bild, PyRect::neu(c), PyRect::neu(aus), flags),
                )?;
            }
        } else {
            if sichtbar {
                let r = blit_sprite(surface, spr, Some(spr.getattr("source_rect")?), flags)?;
                sd.set_item(spr, r)?;
            }
            // Auch ein unsichtbares Sprite wird sauber gemeldet -
            // gemessen: nach `visible = 0` faellt `dirty` von 1 auf 0,
            // ohne dass irgendetwas gezeichnet wurde.
            if dirty == 1 {
                spr.setattr("dirty", 0)?;
            }
        }
    }
    Ok(())
}

#[pymethods]
impl PyLayeredDirty {
    #[new]
    #[classmethod]
    #[pyo3(signature = (*sprites, **kwargs))]
    fn new(
        cls: &Bound<'_, PyType>,
        sprites: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<PyClassInitializer<PyLayeredDirty>> {
        let py = cls.py();
        let bauplan = PyLayeredDirty::bauplan(py, standardebene(kwargs)?);
        if !cls.is(py.get_type::<PyLayeredDirty>()) {
            return Ok(bauplan);
        }
        let obj = Py::new(py, bauplan)?;
        let b = obj.bind(py).as_any();
        if !sprites.is_empty() {
            b.call_method("add", sprites.clone(), kwargs)?;
        }
        PyLayeredDirty::einstellungen(b, kwargs)?;
        Ok(PyClassInitializer::from(obj))
    }

    #[pyo3(signature = (*sprites, **kwargs))]
    fn __init__(
        slf: &Bound<'_, PyLayeredDirty>,
        sprites: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        PyLayeredUpdates::__init__(slf.as_super(), sprites, kwargs)?;
        {
            let mut k = slf.borrow_mut();
            k._clip = None;
            k._use_update = false;
            k._time_threshold = 1000.0 / 80.0;
            k._bgd = None;
        }
        PyLayeredDirty::einstellungen(slf.as_any(), kwargs)
    }

    /// Nimmt **nur** DirtySprites auf.
    ///
    /// Gemessen: ein einfaches `Sprite` gibt einen `AttributeError` ohne
    /// Text, weil pygame `raise AttributeError()` ohne Argument schreibt.
    #[pyo3(signature = (sprite, layer=None))]
    fn add_internal(
        slf: &Bound<'_, PyLayeredDirty>,
        sprite: &Bound<'_, PyAny>,
        layer: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        for name in ["dirty", "visible", "blendmode"] {
            if !sprite.hasattr(name)? {
                return Err(PyAttributeError::new_err(""));
            }
        }
        if sprite.downcast::<PyDirtySprite>().is_err() {
            return Err(PyTypeError::new_err(""));
        }
        if sprite.getattr("dirty")?.extract::<i64>()? == 0 {
            sprite.setattr("dirty", 1)?;
        }
        PyLayeredUpdates::add_internal(slf.as_super(), sprite, layer)
    }

    /// `LayeredDirty.draw(surface, bgd=None, special_flags=None)`.
    ///
    /// * `bgd`: ein Hintergrund, der zugleich **abgelegt** wird - danach
    ///   wirkt er auch bei allen weiteren Aufrufen. (pygames Text sagt es
    ///   andersherum, der Code tut dies.)
    /// * `special_flags`: `None` heisst "je Sprite `blendmode` nehmen",
    ///   ein Wert ueberstimmt alle.
    ///
    /// Gibt eine **Kopie** der schmutzigen Rechtecke zurueck, nicht
    /// `lostsprites` selbst (gemessen: `ld.draw(s) is ld.lostsprites` ist
    /// `False`).
    #[pyo3(signature = (surface, bgd=None, special_flags=None))]
    fn draw<'py>(
        slf: &Bound<'py, PyLayeredDirty>,
        surface: &Bound<'py, PyAny>,
        bgd: Option<&Bound<'py, PyAny>>,
        special_flags: Option<i64>,
    ) -> PyResult<Bound<'py, PyList>> {
        let py = slf.py();
        let s = slf.as_any();
        let orig_clip = surface.call_method0("get_clip")?;
        let latest_clip = match s.getattr("_clip")? {
            c if c.is_none() => orig_clip.clone(),
            c => c,
        };
        let clip_rect = rect_aus(&latest_clip)?;

        if let Some(b) = bgd {
            s.setattr("_bgd", b)?;
        }
        let local_bgd = s.getattr("_bgd")?;

        surface.call_method1("set_clip", (&latest_clip,))?;
        let start = rustgame_core::time::get_ticks_f64();

        let sprites: Vec<Bound<'py, PyAny>> = spritelist(s)?.iter().collect();
        let sd = spritedict(s)?;
        let verloren = lostsprites(s)?;

        let ergebnis: Vec<KernRect> = if s.getattr("_use_update")?.is_truthy()? {
            // pygames `local_update` **ist** `self.lostsprites`. Hier
            // wird die Liste einmal in Werte aufgeloest (die Eintraege
            // koennen auf lebende Sprite-Rechtecke zeigen) und danach
            // oertlich gefuehrt; am Ende wird sie ohnehin geleert.
            let mut update: Vec<KernRect> = verloren
                .iter()
                .map(|r| rect_aus(&r))
                .collect::<PyResult<_>>()?;

            // 1. Wo ist es schmutzig?
            let init_rect = s.getattr("_init_rect")?;
            finde_dirty(&sprites, &sd, &init_rect, clip_rect, &mut update)?;

            // 2. Dort den Hintergrund auftragen.
            if !local_bgd.is_none() {
                let flags = special_flags.unwrap_or(0);
                for r in &update {
                    let pr = PyRect::neu(*r);
                    surface.call_method1("blit", (&local_bgd, pr, PyRect::neu(*r), flags))?;
                }
            }

            // 3. Zeichnen.
            zeichne_dirty(&sprites, &sd, surface, special_flags, &update)?;
            update
        } else {
            if !local_bgd.is_none() {
                let flags = special_flags.unwrap_or(0);
                surface.call_method1("blit", (&local_bgd, (0, 0), py.None(), flags))?;
            }
            for spr in &sprites {
                if spr.getattr("visible")?.is_truthy()? {
                    let flags = match special_flags {
                        Some(f) => f,
                        None => spr.getattr("blendmode")?.extract::<i64>()?,
                    };
                    let r = blit_sprite(surface, spr, Some(spr.getattr("source_rect")?), flags)?;
                    sd.set_item(spr, r)?;
                }
            }
            // **Kein Zuruecksetzen der dirty-Flaggen** - gemessen.
            vec![clip_rect]
        };

        let ende = rustgame_core::time::get_ticks_f64();
        s.setattr(
            "_use_update",
            ende - start <= s.getattr("_time_threshold")?.extract::<f64>()?,
        )?;

        verloren.del_slice(0, verloren.len())?;
        surface.call_method1("set_clip", (&orig_clip,))?;
        PyList::new(py, ergebnis.into_iter().map(PyRect::neu))
    }

    /// `clear(surface, bgd)` - legt **nur** den Hintergrund ab.
    ///
    /// Die Flaeche wird nicht angefasst; `LayeredDirty` malt den
    /// Hintergrund erst beim naechsten `draw` an den richtigen Stellen
    /// auf. Das ersetzt die geerbte Bedeutung von `clear` vollstaendig.
    fn clear(
        slf: &Bound<'_, PyLayeredDirty>,
        surface: &Bound<'_, PyAny>,
        bgd: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let _ = surface;
        slf.as_any().setattr("_bgd", bgd)
    }

    /// Diesen Bereich beim naechsten `draw` neu zeichnen.
    ///
    /// Auf den eigenen Clip beschnitten, falls einer gesetzt ist
    /// (gemessen: `repaint_rect(0,0,100,100)` bei Clip `(10,10,30,30)`
    /// landet als `(10,10,30,30)`).
    fn repaint_rect(
        slf: &Bound<'_, PyLayeredDirty>,
        screen_rect: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let s = slf.as_any();
        let clip = s.getattr("_clip")?;
        let r = if clip.is_truthy()? {
            screen_rect.call_method1("clip", (&clip,))?
        } else {
            PyRect::neu(rect_aus(screen_rect)?).into_pyobject(slf.py())?.into_any()
        };
        lostsprites(s)?.append(r)
    }

    /// `set_clip(rect=None)` - beschraenkt das Zeichnen und faellt auf
    /// Vollbild zurueck.
    ///
    /// `None` holt das Rechteck der Fensterflaeche, genau wie pygame
    /// (gemessen: danach meldet `get_clip()` `Rect(0, 0, 100, 100)`).
    #[pyo3(signature = (screen_rect=None))]
    fn set_clip(
        slf: &Bound<'_, PyLayeredDirty>,
        screen_rect: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        let py = slf.py();
        let s = slf.as_any();
        match screen_rect {
            Some(r) if !r.is_none() => s.setattr("_clip", r)?,
            _ => {
                let fenster = py
                    .import("rustgame_py")?
                    .getattr("display")?
                    .call_method0("get_surface")?;
                s.setattr("_clip", fenster.call_method0("get_rect")?)?;
            }
        }
        s.setattr("_use_update", false)
    }

    fn get_clip<'py>(slf: &Bound<'py, PyLayeredDirty>) -> PyResult<Bound<'py, PyAny>> {
        slf.as_any().getattr("_clip")
    }

    /// Wie bei [`PyLayeredUpdates`], macht das Sprite dabei aber
    /// schmutzig.
    fn change_layer(
        slf: &Bound<'_, PyLayeredDirty>,
        sprite: &Bound<'_, PyAny>,
        new_layer: i64,
    ) -> PyResult<()> {
        ebene_wechseln(slf.as_any(), sprite, new_layer)?;
        if sprite.getattr("dirty")?.extract::<i64>()? == 0 {
            sprite.setattr("dirty", 1)?;
        }
        Ok(())
    }

    /// Der alte, falsch geschriebene Name - warnt und reicht weiter.
    fn set_timing_treshold(
        slf: &Bound<'_, PyLayeredDirty>,
        time_ms: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        veraltet(
            slf.py(),
            pyo3::ffi::c_str!(
                "This function will be removed, use set_timing_threshold function instead"
            ),
        )?;
        slf.as_any()
            .call_method1("set_timing_threshold", (time_ms,))?;
        Ok(())
    }

    /// Die Schwelle in Millisekunden.
    ///
    /// Gemessen: etwas, das keine Zahl ist, gibt `TypeError: Expected
    /// numeric value, got str instead`.
    fn set_timing_threshold(
        slf: &Bound<'_, PyLayeredDirty>,
        time_ms: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        match time_ms.extract::<f64>() {
            Ok(v) if !time_ms.is_instance_of::<pyo3::types::PyString>() => {
                slf.as_any().setattr("_time_threshold", v)
            }
            _ => Err(PyTypeError::new_err(format!(
                "Expected numeric value, got {} instead",
                klassenname(time_ms)?
            ))),
        }
    }
}

// ---------------------------------------------------------------------------
//  GroupSingle
// ---------------------------------------------------------------------------

/// `pygame.sprite.GroupSingle` - Gruppe mit genau einem Platz.
///
/// Wer hinzukommt, wirft den Vorgaenger hinaus: der alte Bewohner ist
/// danach in **keiner** Gruppe mehr (`alive()` ist `False`), nicht nur in
/// dieser nicht.
///
/// `has_internal` fragt **nicht** das Woerterbuch, sondern den einen
/// Platz - das ist der Grund, warum ein durch `draw` in `spritedict`
/// gelandeter Eintrag die Mitgliedschaft nicht veraendert.
#[pyclass(name = "GroupSingle", module = "rustgame_py.sprite",
          extends = PyAbstractGroup, subclass)]
pub struct PyGroupSingle {
    einzel: Option<Py<PyAny>>,
}

impl PyGroupSingle {
    fn bauplan(py: Python<'_>) -> PyClassInitializer<PyGroupSingle> {
        PyClassInitializer::from(PyAbstractGroup::leer(py))
            .add_subclass(PyGroupSingle { einzel: None })
    }
}

#[pymethods]
impl PyGroupSingle {
    #[new]
    #[classmethod]
    #[pyo3(signature = (sprite=None))]
    fn new(
        cls: &Bound<'_, PyType>,
        sprite: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<PyClassInitializer<PyGroupSingle>> {
        let py = cls.py();
        let bauplan = PyGroupSingle::bauplan(py);
        if !cls.is(py.get_type::<PyGroupSingle>()) {
            return Ok(bauplan);
        }
        let obj = Py::new(py, bauplan)?;
        if let Some(s) = sprite {
            if !s.is_none() {
                obj.bind(py).as_any().call_method1("add", (s,))?;
            }
        }
        Ok(PyClassInitializer::from(obj))
    }

    #[pyo3(signature = (sprite=None))]
    fn __init__(
        slf: &Bound<'_, PyGroupSingle>,
        sprite: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        PyAbstractGroup::zuruecksetzen(slf.as_any())?;
        slf.borrow_mut().einzel = None;
        if let Some(s) = sprite {
            if !s.is_none() {
                slf.as_any().call_method1("add", (s,))?;
            }
        }
        Ok(())
    }

    /// `copy()` - nimmt ausdruecklich `GroupSingle`, nicht
    /// `self.__class__`.
    fn copy<'py>(slf: &Bound<'py, PyGroupSingle>) -> PyResult<Bound<'py, PyAny>> {
        let py = slf.py();
        let einzel = slf.borrow().einzel.as_ref().map(|s| s.clone_ref(py));
        py.get_type::<PyGroupSingle>().call1((einzel,))
    }

    fn sprites<'py>(slf: &Bound<'py, PyGroupSingle>) -> PyResult<Bound<'py, PyList>> {
        let py = slf.py();
        match slf.borrow().einzel.as_ref() {
            Some(s) => PyList::new(py, [s.bind(py)]),
            None => Ok(PyList::empty(py)),
        }
    }

    #[pyo3(signature = (sprite, layer=None))]
    fn add_internal(
        slf: &Bound<'_, PyGroupSingle>,
        sprite: &Bound<'_, PyAny>,
        layer: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        let _ = layer;
        let py = slf.py();
        let alt = slf.borrow().einzel.as_ref().map(|s| s.clone_ref(py));
        if let Some(a) = alt {
            let a = a.bind(py);
            a.call_method1("remove_internal", (slf,))?;
            slf.as_any().call_method1("remove_internal", (a,))?;
        }
        slf.borrow_mut().einzel = Some(sprite.clone().unbind());
        Ok(())
    }

    fn __bool__(slf: &Bound<'_, PyGroupSingle>) -> bool {
        slf.borrow().einzel.is_some()
    }

    /// `group.sprite` - der eine Bewohner.
    #[getter]
    fn get_sprite(slf: &Bound<'_, PyGroupSingle>) -> Option<Py<PyAny>> {
        let py = slf.py();
        slf.borrow().einzel.as_ref().map(|s| s.clone_ref(py))
    }

    /// `group.sprite = s`.
    ///
    /// Anders als `add` geht das auch dann durch die volle Buchung, wenn
    /// `s` schon drinsteht - so steht es in pygames `_set_sprite`.
    #[setter]
    fn set_sprite(slf: &Bound<'_, PyGroupSingle>, sprite: &Bound<'_, PyAny>) -> PyResult<()> {
        slf.as_any().call_method1("add_internal", (sprite,))?;
        sprite.call_method1("add_internal", (slf,))?;
        Ok(())
    }

    fn remove_internal(slf: &Bound<'_, PyGroupSingle>, sprite: &Bound<'_, PyAny>) -> PyResult<()> {
        let py = slf.py();
        let ist_der = slf
            .borrow()
            .einzel
            .as_ref()
            .is_some_and(|s| s.bind(py).is(sprite));
        if ist_der {
            slf.borrow_mut().einzel = None;
        }
        if spritedict(slf.as_any())?.contains(sprite)? {
            eintrag_streichen(slf.as_any(), sprite)?;
        }
        Ok(())
    }

    fn has_internal(slf: &Bound<'_, PyGroupSingle>, sprite: &Bound<'_, PyAny>) -> bool {
        let py = slf.py();
        slf.borrow()
            .einzel
            .as_ref()
            .is_some_and(|s| s.bind(py).is(sprite))
    }

    fn __contains__(slf: &Bound<'_, PyGroupSingle>, sprite: &Bound<'_, PyAny>) -> bool {
        PyGroupSingle::has_internal(slf, sprite)
    }
}

// ---------------------------------------------------------------------------
//  Kollisionen
// ---------------------------------------------------------------------------

/// `collide_rect(left, right)` - beruehren sich die Rechtecke?
///
/// Ein Sprite ohne `rect` gibt `AttributeError` ("'NoneType' object has
/// no attribute 'colliderect'") - gemessen, und hier faellt derselbe
/// Fehler an derselben Stelle an, weil der Aufruf ueber Python geht.
#[pyfunction]
pub fn collide_rect(left: &Bound<'_, PyAny>, right: &Bound<'_, PyAny>) -> PyResult<bool> {
    left.getattr("rect")?
        .call_method1("colliderect", (right.getattr("rect")?,))?
        .is_truthy()
}

/// Der Mittelpunktsabstand im Quadrat.
fn abstand2(left: &Bound<'_, PyAny>, right: &Bound<'_, PyAny>) -> PyResult<f64> {
    let a = left.getattr("rect")?;
    let b = right.getattr("rect")?;
    let dx: f64 = a.getattr("centerx")?.extract::<f64>()? - b.getattr("centerx")?.extract::<f64>()?;
    let dy: f64 = a.getattr("centery")?.extract::<f64>()? - b.getattr("centery")?.extract::<f64>()?;
    Ok(dx * dx + dy * dy)
}

/// Der Radius eines Sprites - aus `sprite.radius`, sonst aus dem
/// Rechteck, und dann **angeschrieben**.
///
/// Der Text von pygame sagt nur "wird gerechnet"; der Code schreibt ihn
/// zusaetzlich an das Sprite. Gemessen, was das bedeutet: ein Sprite mit
/// `rect = (0,0,10,10)` hat nach dem ersten `collide_circle` den Radius
/// `7.0710678118654755`. Waechst sein Rechteck danach, bleibt der alte
/// Radius stehen und die naechste Pruefung liefert `False`, wo sie `True`
/// liefern muesste. Kein Fehler dieser Uebersetzung, sondern pygames
/// Zwischenspeicher - und er ist beobachtbar.
fn radius_von(spr: &Bound<'_, PyAny>) -> PyResult<f64> {
    let py = spr.py();
    match spr.getattr("radius") {
        Ok(v) => v.extract(),
        Err(e) if e.is_instance_of::<PyAttributeError>(py) => {
            let r = spr.getattr("rect")?;
            let w: f64 = r.getattr("width")?.extract()?;
            let h: f64 = r.getattr("height")?.extract()?;
            // "half of the diagonal" - pygames eigener Kommentar warnt,
            // dass das bei langen schmalen Rechtecken zu grosszuegig ist.
            let radius = 0.5 * (w * w + h * h).sqrt();
            spr.setattr("radius", radius)?;
            Ok(radius)
        }
        Err(e) => Err(e),
    }
}

/// `collide_circle(left, right)` - ueberlappen die umschliessenden Kreise?
///
/// **Veraendert die Sprites**: ein fehlender Radius wird gerechnet und
/// abgelegt.
#[pyfunction]
pub fn collide_circle(left: &Bound<'_, PyAny>, right: &Bound<'_, PyAny>) -> PyResult<bool> {
    let d2 = abstand2(left, right)?;
    let rl = radius_von(left)?;
    let rr = radius_von(right)?;
    Ok(d2 <= (rl + rr) * (rl + rr))
}

// Hier stand `zahl_repr`: Pythons `repr` einer `f64`, damit `__repr__`
// `ratio=1.3` schreibt und nicht `ratio=1.3000000000000000444`. Seit
// `ratio` ein Python-Objekt ist, braucht es die Umrechnung nicht mehr -
// das Objekt wird unmittelbar gefragt, und das ist auch das, was pygames
// f-String tut.

/// `collide_rect_ratio(ratio)` - wie [`collide_rect`], aber mit
/// skalierten Rechtecken.
///
/// In pygame ist das eine **Klasse**, kein Abschluss: `repr` meldet
/// `<collide_rect_ratio @c32cf0 ratio=1.3>`. Skaliert wird ueber
/// `inflate(w * ratio - w, h * ratio - h)`, und die Differenz ist eine
/// Fliesskommazahl, die `Rect.inflate` zur Null hin abschneidet - bei
/// `w = 7` und `ratio = 1.3` sind das `2.0999999999999996` -> `2`.
// **`subclass`** - pygames Fassung ist eine gewoehnliche
// Python-Klasse, und von der laesst sich erben. PyO3 verbietet
// das ohne dieses Flag (`is not an acceptable base type`).
// Gefunden hat es die Probe `Erben`, die fuer jede der 65
// Klassen fragt, ob eine Ableitung geht - acht Runden hatten
// nur gefragt, ob die Klasse **da** ist.
#[pyclass(name = "collide_rect_ratio", module = "rustgame_py.sprite", subclass)]
pub struct PyCollideRectRatio {
    /// Auch das ein blankes Fach - `cr.ratio = 'x'` geht bei pygame
    /// durch, und `cr.ratio = -7` bleibt dort ein `int` (mit `f64`
    /// stand hier `-7.0`).
    #[pyo3(get, set)]
    ratio: Py<PyAny>,
}

#[pymethods]
impl PyCollideRectRatio {
    #[new]
    fn new(ratio: Py<PyAny>) -> PyCollideRectRatio {
        PyCollideRectRatio { ratio }
    }

    fn __call__(
        slf: &Bound<'_, PyCollideRectRatio>,
        left: &Bound<'_, PyAny>,
        right: &Bound<'_, PyAny>,
    ) -> PyResult<bool> {
        let ratio: f64 = slf.borrow().ratio.extract(slf.py())?;
        let l = aufblasen(&left.getattr("rect")?, ratio)?;
        let r = aufblasen(&right.getattr("rect")?, ratio)?;
        l.call_method1("colliderect", (r,))?.is_truthy()
    }

    fn __repr__(slf: &Bound<'_, PyCollideRectRatio>) -> PyResult<String> {
        Ok(format!(
            "<{} @{:x} ratio={}>",
            klassenname(slf.as_any())?,
            slf.as_ptr() as usize & 0xFF_FFFF,
            // `str`, nicht `repr`: pygames f-String setzt den Wert
            // ueber `{self.ratio}` ein, und das ist `str()`. Fuer eine
            // Gleitzahl dasselbe, fuer `ratio = 'x'` nicht.
            slf.borrow().ratio.bind(slf.py()).str()?
        ))
    }
}

/// `rect.inflate(w * ratio - w, h * ratio - h)` mit pygames Rundung.
///
/// pygame reicht die **Fliesskommazahl** weiter; `Rect.inflate` nimmt sie
/// als C-`int` entgegen und schneidet dabei zur Null hin ab. Das `PyRect`
/// dieser Huelle nimmt nur ganze Zahlen (gemessen an der Gegenprobe:
/// `TypeError: argument 'dx': 'float' object cannot be interpreted as an
/// integer`), also wird hier abgeschnitten - `as i32` tut in Rust
/// dasselbe wie Pythons C-Wandlung. Bei `w = 7` und `ratio = 1.3` sind
/// das `2.0999999999999996` -> `2`, nicht `3`; gemessen ergibt
/// `Rect(0,0,7,7).inflate(2.0999999999999996, 2.1)` in pygame
/// `(-1, -1, 9, 9)`.
fn aufblasen<'py>(rect: &Bound<'py, PyAny>, ratio: f64) -> PyResult<Bound<'py, PyAny>> {
    let w: f64 = rect.getattr("width")?.extract()?;
    let h: f64 = rect.getattr("height")?.extract()?;
    rect.call_method1(
        "inflate",
        ((w * ratio - w) as i32, (h * ratio - h) as i32),
    )
}

/// `collide_circle_ratio(ratio)` - wie [`collide_circle`], aber die
/// Radien werden mit `ratio` gestreckt.
///
/// Abgelegt wird der **ungeskalierte** Radius (gemessen:
/// `collide_circle_ratio(2.0)` an einem `10x10`-Sprite hinterlaesst
/// `7.0710678118654755`, nicht das Doppelte).
#[pyclass(
    name = "collide_circle_ratio",
    module = "rustgame_py.sprite",
    subclass
)]
pub struct PyCollideCircleRatio {
    /// Auch das ein blankes Fach - `cr.ratio = 'x'` geht bei pygame
    /// durch, und `cr.ratio = -7` bleibt dort ein `int` (mit `f64`
    /// stand hier `-7.0`).
    #[pyo3(get, set)]
    ratio: Py<PyAny>,
}

#[pymethods]
impl PyCollideCircleRatio {
    #[new]
    fn new(ratio: Py<PyAny>) -> PyCollideCircleRatio {
        PyCollideCircleRatio { ratio }
    }

    fn __call__(
        slf: &Bound<'_, PyCollideCircleRatio>,
        left: &Bound<'_, PyAny>,
        right: &Bound<'_, PyAny>,
    ) -> PyResult<bool> {
        let ratio: f64 = slf.borrow().ratio.extract(slf.py())?;
        let d2 = abstand2(left, right)?;
        let rl = radius_von(left)? * ratio;
        let rr = radius_von(right)? * ratio;
        Ok(d2 <= (rl + rr) * (rl + rr))
    }

    fn __repr__(slf: &Bound<'_, PyCollideCircleRatio>) -> PyResult<String> {
        Ok(format!(
            "<{} @{:x} ratio={}>",
            klassenname(slf.as_any())?,
            slf.as_ptr() as usize & 0xFF_FFFF,
            // `str`, nicht `repr`: pygames f-String setzt den Wert
            // ueber `{self.ratio}` ein, und das ist `str()`. Fuer eine
            // Gleitzahl dasselbe, fuer `ratio = 'x'` nicht.
            slf.borrow().ratio.bind(slf.py()).str()?
        ))
    }
}

/// Die Maske eines Sprites - `sprite.mask`, sonst frisch aus dem Bild.
///
/// Sie wird **nicht** abgelegt (gemessen: nach `collide_mask` hat das
/// Sprite kein `mask`-Attribut). Bei jedem Aufruf entsteht also eine
/// neue - so steht es in pygame, und es ist der Grund, warum
/// `collide_mask` in Schleifen teuer ist.
fn maske_von<'py>(spr: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
    let py = spr.py();
    match spr.getattr("mask") {
        Ok(m) => Ok(m),
        Err(e) if e.is_instance_of::<PyAttributeError>(py) => {
            let bild = spr.getattr("image")?;
            let flaeche: PyRef<'_, PySurface> = bild.extract()?;
            // `pygame.mask.from_surface` mit dem Vorgabeschwellwert 127.
            let m: PyMask = crate::maske::maske_from_surface(&flaeche, 127)?;
            Ok(m.into_pyobject(py)?.into_any())
        }
        Err(e) => Err(e),
    }
}

/// `collide_mask(left, right)` - die erste ueberlappende Stelle, sonst
/// `None`.
///
/// Der Punkt liegt in den Koordinaten der **linken** Maske.
#[pyfunction]
pub fn collide_mask<'py>(
    left: &Bound<'py, PyAny>,
    right: &Bound<'py, PyAny>,
) -> PyResult<Bound<'py, PyAny>> {
    let a = rect_aus(&left.getattr("rect")?)?;
    let b = rect_aus(&right.getattr("rect")?)?;
    let links = maske_von(left)?;
    let rechts = maske_von(right)?;
    links.call_method1("overlap", (rechts, (b.x - a.x, b.y - a.y)))
}

/// Ob zwei Sprites sich beruehren - mit oder ohne eigenen Rueckruf.
fn treffen(
    collided: Option<&Bound<'_, PyAny>>,
    sprite: &Bound<'_, PyAny>,
    anderes: &Bound<'_, PyAny>,
) -> PyResult<bool> {
    match collided {
        Some(f) => f.call1((sprite, anderes))?.is_truthy(),
        None => collide_rect(sprite, anderes),
    }
}

/// `spritecollide(sprite, group, dokill, collided=None)`.
///
/// Sammelt **erst vollstaendig** und toetet **danach**: die getoeteten
/// Sprites stehen im Ergebnis, sind aber in keiner Gruppe mehr
/// (gemessen). `group` darf alles Iterierbare sein, nicht nur eine
/// Gruppe - auch das gemessen.
#[pyfunction]
#[pyo3(signature = (sprite, group, dokill, collided=None))]
pub fn spritecollide<'py>(
    sprite: &Bound<'py, PyAny>,
    group: &Bound<'py, PyAny>,
    dokill: bool,
    collided: Option<&Bound<'py, PyAny>>,
) -> PyResult<Bound<'py, PyList>> {
    let mut getroffen = Vec::new();
    for anderes in group.try_iter()? {
        let anderes = anderes?;
        if treffen(collided, sprite, &anderes)? {
            getroffen.push(anderes);
        }
    }
    if dokill {
        for g in &getroffen {
            g.call_method0("kill")?;
        }
    }
    PyList::new(sprite.py(), getroffen)
}

/// `groupcollide(groupa, groupb, dokilla, dokillb, collided=None)`.
///
/// Ruft [`spritecollide`] je a-Sprite. Mit `dokillb = True` sind die
/// b-Sprites also **schon tot**, wenn das naechste a-Sprite an die Reihe
/// kommt - das Woerterbuch enthaelt danach nur noch den ersten Treffer.
/// Gemessen an zwei mal zwei ueberlappenden Sprites:
///
/// ```text
/// groupcollide(A, B, True, True) -> {A1: [B1, B2]}
/// Gruppen danach: A = [A2], B = []
/// ```
#[pyfunction]
#[pyo3(signature = (groupa, groupb, dokilla, dokillb, collided=None))]
pub fn groupcollide<'py>(
    groupa: &Bound<'py, PyAny>,
    groupb: &Bound<'py, PyAny>,
    dokilla: bool,
    dokillb: bool,
    collided: Option<&Bound<'py, PyAny>>,
) -> PyResult<Bound<'py, PyDict>> {
    let py = groupa.py();
    let raus = PyDict::new(py);
    for a in groupa.try_iter()? {
        let a = a?;
        let treffer = spritecollide(&a, groupb, dokillb, collided)?;
        if !treffer.is_empty() {
            raus.set_item(&a, treffer)?;
        }
    }
    if dokilla {
        for (a, _) in raus.iter() {
            a.call_method0("kill")?;
        }
    }
    Ok(raus)
}

/// `spritecollideany(sprite, group, collided=None)` - irgendeines, oder
/// `None`.
///
/// "Irgendeines" ist in Wahrheit **das erste in Gruppenreihenfolge** -
/// gemessen an einer Gruppe `[nah1, nah2, weit]`, die `nah1` liefert.
#[pyfunction]
#[pyo3(signature = (sprite, group, collided=None))]
pub fn spritecollideany<'py>(
    sprite: &Bound<'py, PyAny>,
    group: &Bound<'py, PyAny>,
    collided: Option<&Bound<'py, PyAny>>,
) -> PyResult<Option<Bound<'py, PyAny>>> {
    for anderes in group.try_iter()? {
        let anderes = anderes?;
        if treffen(collided, sprite, &anderes)? {
            return Ok(Some(anderes));
        }
    }
    Ok(None)
}

// ---------------------------------------------------------------------------
//  Eintragen
// ---------------------------------------------------------------------------

/// Haengt alles in das Untermodul `rustgame_py.sprite`.
///
/// Die Reihenfolge ist die der Vererbung: PyO3 baut das Typobjekt einer
/// Unterklasse beim ersten Zugriff und braucht dafuer die Basis - sie
/// zuerst einzutragen spart einen Sonderfall, der sonst nur sehr selten
/// und dann schwer zu finden ist.
pub fn eintragen(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySprite>()?;
    m.add_class::<PyDirtySprite>()?;

    m.add_class::<PyAbstractGroup>()?;
    m.add_class::<PyGroup>()?;
    m.add_class::<PyRenderPlain>()?;
    m.add_class::<PyRenderClear>()?;
    m.add_class::<PyRenderUpdates>()?;
    m.add_class::<PyOrderedUpdates>()?;
    m.add_class::<PyLayeredUpdates>()?;
    m.add_class::<PyLayeredDirty>()?;
    m.add_class::<PyGroupSingle>()?;

    m.add_class::<PyCollideRectRatio>()?;
    m.add_class::<PyCollideCircleRatio>()?;

    m.add_function(wrap_pyfunction!(collide_rect, m)?)?;
    m.add_function(wrap_pyfunction!(collide_circle, m)?)?;
    m.add_function(wrap_pyfunction!(collide_mask, m)?)?;
    m.add_function(wrap_pyfunction!(spritecollide, m)?)?;
    m.add_function(wrap_pyfunction!(spritecollideany, m)?)?;
    m.add_function(wrap_pyfunction!(groupcollide, m)?)?;
    Ok(())
}
