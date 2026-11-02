//! geometrie.rs
//! ============
//! `pygame.geometry` - `Circle` und `Line`.
//!
//! Zwei Formen, die pygame-ce seit 2.5 mitbringt. Sie standen in keiner
//! Zahl dieses Projekts, weil weder die Abdeckungstabelle noch der
//! Testlaeufer sie kannten - `geometry_test.py` hat **145** Tests.
//!
//! ## Was ueberrascht hat
//!
//! * **`top`, `bottom`, `left`, `right` sind Punkte**, keine Zahlen:
//!   `Circle(1, 2, 3).top` gibt `(1.0, -1.0)`.
//! * **`as_rect` schneidet zur Null hin ab**, wie `Rect` es mit
//!   Fliesskomma immer tut: `Circle(2.5, 3.5, 4.2).as_rect()` gibt
//!   `Rect(-1, 0, 8, 8)`, nicht `Rect(-2, -1, 8, 8)`.
//! * **Alles ist Fliesskomma.** `Circle(1, 2, 3).x` ist `1.0`, und die
//!   Darstellung schreibt `Circle((1.0, 2.0), 3.0)`.

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyList, PyTuple};

use rustgame_math::geometry::{Circle, Line};

use crate::flaeche::Weitergabe;
use crate::frechteck::PyFRect;
use crate::rechteck::PyRect;

/// Der Satz, den pygame beim Anlegen eines Kreises gibt - fuer **jeden**
/// Einwand, auch den ueber einen negativen Radius.
const KREIS_ARGUMENTE: &str = concat!(
    "Arguments must be a Circle, a sequence of length 3 or 2, or an object ",
    "with an attribute called 'circle', all with corresponding nonnegative ",
    "radius argument"
);

/// Dasselbe fuer die Strecke.
const STRECKE_ARGUMENTE: &str =
    "Invalid line end points, expected 4 numbers or 2 sequences of 2 numbers";

/// Ein Drehwinkel - `Invalid angle argument, must be numeric`.
///
/// Gemessen an `Circle.rotate('x')` und `rotate_ip('x')`; vorher kam
/// PyO3s Satz, weil das Argument als `f64` deklariert war.
fn winkel(o: &Bound<'_, PyAny>) -> PyResult<f64> {
    gleitzahl(o)
        .ok_or_else(|| PyTypeError::new_err("Invalid angle argument, must be numeric"))
}

/// Der Satz fuer einen untauglichen Drehpunkt.
const DREHPUNKT: &str = "Invalid rotation point argument, must be a sequence of 2 numbers";

/// Zwei Zahlen aus einem Objekt - `(x, y)`, eine Liste oder ein Vektor.
///
/// **`two`, nicht `2`.** pygames `geometry`-Modul schreibt die Zahl an
/// dieser Stelle aus: `Invalid point argument, must be a sequence of two
/// numbers` - gemessen an `Circle.collidelist('x')` und
/// `collideswith('x')`. Die **Felder** sagen etwas anderes, siehe
/// [`punkt_feld`], und die meisten **Methoden** haben einen eigenen Satz,
/// siehe [`punkt_satz`].
fn punkt(o: &Bound<'_, PyAny>) -> PyResult<(f64, f64)> {
    let fehler =
        || PyTypeError::new_err("Invalid point argument, must be a sequence of two numbers");
    if o.len().map(|n| n != 2).unwrap_or(true) {
        return Err(fehler());
    }
    Ok((
        o.get_item(0).map_err(|_| fehler())?.extract().map_err(|_| fehler())?,
        o.get_item(1).map_err(|_| fehler())?.extract().map_err(|_| fehler())?,
    ))
}

/// Ein Punkt mit **dem Satz dieser Stelle**.
///
/// pygame hat fuer fast jede Methode einen eigenen: `move requires a pair
/// of numbers`, `project requires a sequence of two numbers`,
/// `Circle.collidepoint requires a point or PointLike object`. Hier gab
/// ein gemeinsamer Helfer ueberall denselben Satz - gemessen waren es
/// **vierzehn** Stellen.
fn punkt_satz(o: &Bound<'_, PyAny>, satz: &str) -> PyResult<(f64, f64)> {
    punkt(o).map_err(|_| PyTypeError::new_err(satz.to_string()))
}

/// Der blosse Typname eines Werts - fuer pygames Fehlersaetze.
fn typname(o: &Bound<'_, PyAny>) -> String {
    o.get_type()
        .name()
        .map(|n| n.to_string())
        .unwrap_or_else(|_| "object".to_string())
}

/// Eine Zahl, **wie pygames `pg_DoubleFromObj` sie liest**.
///
/// Ein `float` wird unmittelbar genommen, ein `int` ueber `PyLong_AsLong`
/// - und ein `long` ist unter Windows **32 Bit**. Gemessen heisst das:
///
/// ```text
/// c.x = 2**31-1   ->  2147483647.0
/// c.x = 2**31     ->  TypeError: Expected a number, got 'int'
/// c.x = 1e30      ->  1e+30            (ein float, also kein long)
/// c.x = True      ->  1.0
/// ```
///
/// Hier stand `v: f64`, und PyO3 nimmt jede Ganzzahl an: `c.x = 10**30`
/// wurde stillschweigend `1e+30`, wo pygame zurueckweist. Die **Groesse**
/// der Zahl ist dabei nicht der Punkt - der **Typ** ist es, und eine zu
/// grosse Ganzzahl ist fuer pygame keine Zahl mehr.
fn gleitzahl(o: &Bound<'_, PyAny>) -> Option<f64> {
    if o.is_instance_of::<pyo3::types::PyBool>() {
        return o.extract::<bool>().ok().map(f64::from);
    }
    if o.is_instance_of::<pyo3::types::PyInt>() {
        return o.extract::<i32>().ok().map(f64::from);
    }
    if o.is_instance_of::<pyo3::types::PyFloat>() {
        return o.extract::<f64>().ok();
    }
    None
}

/// `Expected a number, got 'list'` - der Satz von `Circle.x`, `y`, `r`
/// und `radius`.
fn zahl_mit_typ(o: &Bound<'_, PyAny>) -> PyResult<f64> {
    gleitzahl(o)
        .ok_or_else(|| PyTypeError::new_err(format!("Expected a number, got '{}'", typname(o))))
}

/// `Invalid type for area, must be numeric` - der Satz der **abgeleiteten**
/// Masse (`d`, `diameter`, `area`, `circumference`, `r_sqr`).
///
/// pygame nennt hier den Namen des Masses und **nicht** den Typ des
/// Werts; bei `r_sqr` heisst es `radius squared`. Alles gemessen.
fn zahl_fuer(o: &Bound<'_, PyAny>, was: &str) -> PyResult<f64> {
    gleitzahl(o)
        .ok_or_else(|| PyTypeError::new_err(format!("Invalid type for {was}, must be numeric")))
}

/// `Expected a number` - **ohne** Typnamen, der Satz von `Line.ax` und
/// seinen drei Geschwistern. pygames `Line` ist knapper als sein `Circle`,
/// und das ist gemessen, nicht geraten.
fn zahl_blank(o: &Bound<'_, PyAny>) -> PyResult<f64> {
    gleitzahl(o).ok_or_else(|| PyTypeError::new_err("Expected a number"))
}

/// Ein Punkt **als Feldwert** - `Expected a sequence of 2 numbers`.
///
/// Denselben Satz gibt pygame fuer `Circle.center`, `top`, `bottom`,
/// `left`, `right` und `Line.a`/`b`, und zwar fuer jeden Wert: `None`,
/// `5`, `(1,)` und `(1,2,3)` eingeschlossen. Nicht derselbe wie der von
/// [`punkt`]: pygames **Methoden** haben eigene Saetze
/// (`move requires a pair of numbers`), seine **Felder** diesen hier.
fn punkt_feld(o: &Bound<'_, PyAny>) -> PyResult<(f64, f64)> {
    let fehler = || PyTypeError::new_err("Expected a sequence of 2 numbers");
    if o.len().map(|n| n != 2).unwrap_or(true) {
        return Err(fehler());
    }
    Ok((
        gleitzahl(&o.get_item(0).map_err(|_| fehler())?).ok_or_else(fehler)?,
        gleitzahl(&o.get_item(1).map_err(|_| fehler())?).ok_or_else(fehler)?,
    ))
}

/// Ein Kreis - `pygame.geometry.Circle`.
///
/// Vier Bauweisen, alle gemessen:
///
/// ```python
/// Circle(x, y, r)        Circle((x, y), r)
/// Circle((x, y, r))      Circle(anderer_kreis)
/// ```
#[pyclass(name = "Circle", module = "rustgame_py.geometry", subclass)]
#[derive(Clone)]
pub struct PyCircle {
    pub(crate) inner: Circle,
}

impl PyCircle {
    fn neu(c: Circle) -> PyCircle {
        PyCircle { inner: c }
    }

    /// Ein Kreis **vom Typ des Vorbilds**.
    ///
    /// `geometry_test.test_move_return_type` leitet ab und prueft
    /// `assertIsInstance(cs.move(1, 1), CircleSub)`. Ein schlichtes
    /// `PyCircle` waere immer die Basisklasse - dieselbe Lage wie bei
    /// `Surface.copy()`, nur dass hier der gewoehnliche Aufruf genuegt:
    /// eine Unterklasse erbt `__init__` und nimmt `(x, y, r)`.
    fn wie_typ<'py>(slf: &Bound<'py, PyCircle>, c: Circle) -> PyResult<Bound<'py, PyAny>> {
        let py = slf.py();
        let typ = slf.get_type();
        if typ.is(py.get_type::<PyCircle>()) {
            return Ok(Bound::new(py, PyCircle::neu(c))?.into_any());
        }
        typ.call1((c.x, c.y, c.r))
    }

    /// Ein Kreis aus einem beliebigen Argument - fuer `collideswith`
    /// und die Bauweise mit einem einzigen Wert.
    fn aus(o: &Bound<'_, PyAny>) -> PyResult<Circle> {
        if let Ok(c) = o.extract::<PyRef<PyCircle>>() {
            return Ok(c.inner);
        }
        // **Alles mit einem Feld `circle` zaehlt als Kreis** - dieselbe
        // Nachsicht, die `rect_aus` fuer `rect` kennt. Es darf eine
        // Eigenschaft, ein Feld oder eine Methode sein.
        if let Ok(feld) = o.getattr("circle") {
            let wert = if feld.is_callable() {
                feld.call0()?
            } else {
                feld
            };
            return PyCircle::aus(&wert);
        }
        let laenge = o.len().map_err(|_| PyTypeError::new_err(KREIS_ARGUMENTE))?;
        if laenge == 3 {
            return Ok(Circle::new(
                o.get_item(0)?.extract()?,
                o.get_item(1)?.extract()?,
                o.get_item(2)?.extract()?,
            ));
        }
        if laenge == 2 {
            // `((x, y), r)`
            let mitte = punkt(&o.get_item(0)?)?;
            return Ok(Circle::new(mitte.0, mitte.1, o.get_item(1)?.extract()?));
        }
        Err(PyTypeError::new_err(KREIS_ARGUMENTE))
    }

    /// Der Radius darf nicht negativ sein - **null aber schon**.
    ///
    /// Gemessen: `Circle(1, 2, 0)` geht durch, `Circle(1, 2, -1)` nicht.
    /// Und die Fehlerart haengt am Weg:
    ///
    /// ```text
    /// Circle(1, 2, -1)   TypeError:  Arguments must be a Circle, a sequence of length 3 or 2
    /// c.r = -1           ValueError: Radius must be nonnegative
    /// c.d = -1           ValueError: Invalid diameter value, must be nonnegative
    /// c.area = -1        ValueError: Invalid area value, must be nonnegative
    /// c.r_sqr = -1       ValueError: Invalid radius squared value, must be nonnegative
    /// ```
    ///
    /// Beim Anlegen ist also **jeder** Einwand ein `TypeError`, auch der
    /// ueber den Wert; beim Setzen nennt jede Eigenschaft ihren eigenen
    /// Satz. Ein gemeinsamer Pruefer waere kuerzer und an fuenf Stellen
    /// falsch.
    fn radius_pruefen(r: f64, satz: &str) -> PyResult<f64> {
        if r < 0.0 || r.is_nan() {
            return Err(PyValueError::new_err(satz.to_string()));
        }
        Ok(r)
    }
}

#[pymethods]
impl PyCircle {
    /// **Ohne jedes Argument der Nullzustand** - und das ist `tp_new`.
    ///
    /// pygame sieht die Argumente von `__new__` nicht an und legt das
    /// Objekt leer an; gelesen wird in `tp_init`. Gemessen:
    ///
    /// ```text
    /// Circle.__new__(Circle)         Circle((0.0, 0.0), 1.0)
    /// Circle.__new__(Circle, 1,2,3)  dasselbe - die Argumente zaehlen nicht
    /// Circle()                       TypeError: Arguments must be a Circle, …
    /// Line.__new__(Line)             Line((0.0, 0.0), (0.0, 0.0))
    /// ```
    ///
    /// Hier warf `__new__` denselben Fehler wie der Erzeuger, weil PyO3
    /// nur `#[new]` kennt. Getrennt wird es wie bei `Surface` und
    /// `Color`: dieses `#[new]` gibt den Nullzustand, und
    /// `python/nullbau.py` haengt ein `__init__` an, das zu `_init`
    /// weiterreicht. Gefunden hat den Unterschied `tools/halbfertig.py`.
    #[new]
    // **`*args` und drei Schluesselwoerter.** `geometry.pyi` nennt
    // `x`, `y`, `r`, und gemessen nimmt pygame
    // `Circle(x=1, y=2, r=3)` an. Mit `*args` allein kam dort
    // `TypeError: Circle.__new__() got an unexpected keyword
    // argument 'x'`; die Zaehlung der stellungsweisen Faelle braucht
    // `*args` aber weiter, weil der Einwand fuer **jede** falsche Form
    // derselbe Satz ist.
    #[pyo3(signature = (*args, x=None, y=None, r=None))]
    fn new(
        args: &Bound<'_, PyTuple>,
        x: Option<f64>,
        y: Option<f64>,
        r: Option<f64>,
    ) -> PyResult<PyCircle> {
        if args.is_empty() && x.is_none() && y.is_none() && r.is_none() {
            // Der Nullzustand - siehe den Vermerk oben. Der Einwand
            // gegen `Circle()` faellt in `_init`.
            return Ok(PyCircle::neu(Circle::new(0.0, 0.0, 1.0)));
        }
        if args.is_empty() {
            if let (Some(x), Some(y), Some(r)) = (x, y, r) {
                let c = Circle::new(x, y, r);
                if c.r < 0.0 || c.r.is_nan() {
                    return Err(PyTypeError::new_err(KREIS_ARGUMENTE));
                }
                return Ok(PyCircle::neu(c));
            }
        }
        let c = match args.len() {
            1 => PyCircle::aus(&args.get_item(0)?)?,
            2 => {
                let mitte = punkt(&args.get_item(0)?)?;
                Circle::new(mitte.0, mitte.1, args.get_item(1)?.extract()?)
            }
            3 => Circle::new(
                args.get_item(0)?.extract()?,
                args.get_item(1)?.extract()?,
                args.get_item(2)?.extract()?,
            ),
            _ => {
                return Err(PyTypeError::new_err(KREIS_ARGUMENTE))
            }
        };
        // Auch der Wert-Einwand ist hier ein `TypeError` - gemessen.
        if c.r < 0.0 || c.r.is_nan() {
            return Err(PyTypeError::new_err(KREIS_ARGUMENTE));
        }
        Ok(PyCircle::neu(c))
    }

    /// Was `tp_init` tut: dieselben Argumente, aber in dieses Objekt.
    ///
    /// Gerufen wird es von `python/nullbau.py`; der Unterstrichname
    /// taucht in keinem `dir()` auf (dort zaehlen nur Namen ohne) und in
    /// keiner Meldung, weil der Satz aus `new` kommt.
    #[pyo3(name = "_init")]
    #[pyo3(signature = (*args, x=None, y=None, r=None))]
    fn eigen_init(
        &mut self,
        args: &Bound<'_, PyTuple>,
        x: Option<f64>,
        y: Option<f64>,
        r: Option<f64>,
    ) -> PyResult<()> {
        // **Nicht ueber `new`**, sonst gibt ein leeres `Circle()` den
        // Nullzustand statt des Fehlers. Die Argumente werden hier mit
        // einer Pflichtstelle gelesen - dafuer steht `KREIS_ARGUMENTE`.
        if args.is_empty() && x.is_none() && y.is_none() && r.is_none() {
            return Err(PyTypeError::new_err(KREIS_ARGUMENTE));
        }
        self.inner = PyCircle::new(args, x, y, r)?.inner;
        Ok(())
    }

    // ----- Felder ---------------------------------------------------------

    #[getter]
    fn get_x(&self) -> f64 {
        self.inner.x
    }
    #[setter]
    fn set_x(&mut self, o: &Bound<'_, PyAny>) -> PyResult<()> {
        self.inner.x = zahl_mit_typ(o)?;
        Ok(())
    }
    #[getter]
    fn get_y(&self) -> f64 {
        self.inner.y
    }
    #[setter]
    fn set_y(&mut self, o: &Bound<'_, PyAny>) -> PyResult<()> {
        self.inner.y = zahl_mit_typ(o)?;
        Ok(())
    }
    #[getter]
    fn get_r(&self) -> f64 {
        self.inner.r
    }
    #[setter]
    fn set_r(&mut self, o: &Bound<'_, PyAny>) -> PyResult<()> {
        let v = zahl_mit_typ(o)?;
        self.inner.r = PyCircle::radius_pruefen(v, "Radius must be nonnegative")?;
        Ok(())
    }
    /// `radius` ist der lange Name von `r`.
    #[getter]
    fn get_radius(&self) -> f64 {
        self.inner.r
    }
    #[setter]
    fn set_radius(&mut self, o: &Bound<'_, PyAny>) -> PyResult<()> {
        self.set_r(o)
    }
    #[getter]
    fn get_d(&self) -> f64 {
        self.inner.diameter()
    }
    #[setter]
    fn set_d(&mut self, o: &Bound<'_, PyAny>) -> PyResult<()> {
        let v = zahl_fuer(o, "diameter")?;
        PyCircle::radius_pruefen(v, "Invalid diameter value, must be nonnegative")?;
        self.inner.r = v / 2.0;
        Ok(())
    }
    #[getter]
    fn get_diameter(&self) -> f64 {
        self.inner.diameter()
    }
    #[setter]
    fn set_diameter(&mut self, o: &Bound<'_, PyAny>) -> PyResult<()> {
        self.set_d(o)
    }
    #[getter]
    fn get_r_sqr(&self) -> f64 {
        self.inner.r_sqr()
    }
    #[setter]
    fn set_r_sqr(&mut self, o: &Bound<'_, PyAny>) -> PyResult<()> {
        let v = zahl_fuer(o, "radius squared")?;
        PyCircle::radius_pruefen(v, "Invalid radius squared value, must be nonnegative")?;
        self.inner.r = v.sqrt();
        Ok(())
    }
    #[getter]
    fn get_area(&self) -> f64 {
        self.inner.area()
    }
    #[setter]
    fn set_area(&mut self, o: &Bound<'_, PyAny>) -> PyResult<()> {
        let v = zahl_fuer(o, "area")?;
        PyCircle::radius_pruefen(v, "Invalid area value, must be nonnegative")?;
        self.inner.r = (v / std::f64::consts::PI).sqrt();
        Ok(())
    }
    #[getter]
    fn get_circumference(&self) -> f64 {
        self.inner.circumference()
    }
    #[setter]
    fn set_circumference(&mut self, o: &Bound<'_, PyAny>) -> PyResult<()> {
        let v = zahl_fuer(o, "circumference")?;
        PyCircle::radius_pruefen(v, "Invalid circumference value, must be nonnegative")?;
        self.inner.r = v / (2.0 * std::f64::consts::PI);
        Ok(())
    }
    #[getter]
    fn get_center(&self) -> (f64, f64) {
        self.inner.center()
    }
    #[setter]
    fn set_center(&mut self, o: &Bound<'_, PyAny>) -> PyResult<()> {
        let p = punkt_feld(o)?;
        self.inner.x = p.0;
        self.inner.y = p.1;
        Ok(())
    }

    /// Die vier Randpunkte - **Punkte**, keine Zahlen.
    #[getter]
    fn get_top(&self) -> (f64, f64) {
        self.inner.top()
    }
    #[setter]
    fn set_top(&mut self, o: &Bound<'_, PyAny>) -> PyResult<()> {
        let p = punkt_feld(o)?;
        self.inner.x = p.0;
        self.inner.y = p.1 + self.inner.r;
        Ok(())
    }
    #[getter]
    fn get_bottom(&self) -> (f64, f64) {
        self.inner.bottom()
    }
    #[setter]
    fn set_bottom(&mut self, o: &Bound<'_, PyAny>) -> PyResult<()> {
        let p = punkt_feld(o)?;
        self.inner.x = p.0;
        self.inner.y = p.1 - self.inner.r;
        Ok(())
    }
    #[getter]
    fn get_left(&self) -> (f64, f64) {
        self.inner.left()
    }
    #[setter]
    fn set_left(&mut self, o: &Bound<'_, PyAny>) -> PyResult<()> {
        let p = punkt_feld(o)?;
        self.inner.x = p.0 + self.inner.r;
        self.inner.y = p.1;
        Ok(())
    }
    #[getter]
    fn get_right(&self) -> (f64, f64) {
        self.inner.right()
    }
    #[setter]
    fn set_right(&mut self, o: &Bound<'_, PyAny>) -> PyResult<()> {
        let p = punkt_feld(o)?;
        self.inner.x = p.0 - self.inner.r;
        self.inner.y = p.1;
        Ok(())
    }

    // ----- Umwandeln ------------------------------------------------------

    /// Das umschliessende Rechteck - **abgeschnitten**, nicht gerundet.
    fn as_rect(&self) -> PyRect {
        let c = &self.inner;
        PyRect::neu(rustgame_core::rect::Rect::new(
            (c.x - c.r) as i32,
            (c.y - c.r) as i32,
            c.diameter() as i32,
            c.diameter() as i32,
        ))
    }

    /// Dasselbe mit Nachkommastellen.
    fn as_frect(&self) -> PyFRect {
        let c = &self.inner;
        PyFRect(rustgame_core::frect::FRect::new(
            (c.x - c.r) as f32,
            (c.y - c.r) as f32,
            c.diameter() as f32,
            c.diameter() as f32,
        ))
    }

    fn copy(&self) -> PyCircle {
        PyCircle::neu(self.inner)
    }

    // ----- Beruehrungen ---------------------------------------------------

    #[pyo3(signature = (*args))]
    fn collidepoint(&self, args: &Bound<'_, PyTuple>) -> PyResult<bool> {
        // pygame nennt hier die Methode **und** `PointLike` - gemessen
        // `Circle.collidepoint requires a point or PointLike object`.
        const SATZ: &str = "Circle.collidepoint requires a point or PointLike object";
        let p = match args.len() {
            1 => punkt_satz(&args.get_item(0)?, SATZ)?,
            2 => (
                args.get_item(0)?.extract().map_err(|_| PyTypeError::new_err(SATZ))?,
                args.get_item(1)?.extract().map_err(|_| PyTypeError::new_err(SATZ))?,
            ),
            _ => return Err(PyTypeError::new_err(SATZ)),
        };
        Ok(self.inner.collidepoint(p.0, p.1))
    }

    #[pyo3(signature = (*args))]
    fn collidecircle(&self, args: &Bound<'_, PyTuple>) -> PyResult<bool> {
        // **Nicht der Satz des Erzeugers.** Gemessen gibt pygame hier
        // `A CircleType object was expected`, waehrend `Circle('x')` den
        // langen Satz ueber die Bauweisen gibt.
        let c = kreis_aus_argumenten(args)
            .map_err(|_| PyTypeError::new_err("A CircleType object was expected"))?;
        Ok(self.inner.collidecircle(&c))
    }

    #[pyo3(signature = (*args))]
    fn colliderect(&self, args: &Bound<'_, PyTuple>) -> PyResult<bool> {
        // `RectType`, nicht `Rect`, und ohne das Wort `argument` -
        // gemessen.
        let r = rechteck_aus_argumenten(args).map_err(|_| {
            PyTypeError::new_err("Invalid rect, must be RectType or sequence of 4 numbers")
        })?;
        Ok(self.inner.colliderect(r.0, r.1, r.2, r.3))
    }

    /// Was auch immer: Punkt, Kreis oder Rechteck.
    #[pyo3(signature = (o, /))]
    fn collideswith(&self, o: &Bound<'_, PyAny>) -> PyResult<bool> {
        if let Ok(c) = o.extract::<PyRef<PyCircle>>() {
            return Ok(self.inner.collidecircle(&c.inner));
        }
        if let Some(t) = rechteck_von(o) {
            return Ok(self.inner.colliderect(t.0, t.1, t.2, t.3));
        }
        let p = punkt(o)?;
        Ok(self.inner.collidepoint(p.0, p.1))
    }

    /// Die Stelle des ersten Treffers - oder `-1`.
    #[pyo3(signature = (liste, /))]
    fn collidelist(&self, liste: &Bound<'_, PyAny>) -> PyResult<i64> {
        for (n, e) in liste.try_iter()?.enumerate() {
            if self.collideswith(&e?)? {
                return Ok(n as i64);
            }
        }
        Ok(-1)
    }

    /// Alle Stellen, an denen es trifft.
    #[pyo3(signature = (liste, /))]
    fn collidelistall(&self, py: Python<'_>, liste: &Bound<'_, PyAny>) -> PyResult<Py<PyList>> {
        let aus = PyList::empty(py);
        for (n, e) in liste.try_iter()?.enumerate() {
            if self.collideswith(&e?)? {
                aus.append(n)?;
            }
        }
        Ok(aus.unbind())
    }

    /// Liegt das andere **ganz** im Kreis?
    #[pyo3(signature = (o, /))]
    fn contains(&self, o: &Bound<'_, PyAny>) -> PyResult<bool> {
        if let Ok(c) = o.extract::<PyRef<PyCircle>>() {
            return Ok(self.inner.contains_circle(&c.inner));
        }
        if let Some(t) = rechteck_von(o) {
            return Ok(self.inner.contains_rect(t.0, t.1, t.2, t.3));
        }
        // pygame nennt **alle drei** Moeglichkeiten, samt seinem eigenen
        // Schreibfehler `Frect`: `Invalid shape argument, must be a
        // Circle, Rect / Frect or a coordinate`. Gemessen, Zeichen fuer
        // Zeichen.
        let p = punkt_satz(
            o,
            "Invalid shape argument, must be a Circle, Rect / Frect or a coordinate",
        )?;
        Ok(self.inner.collidepoint(p.0, p.1))
    }

    /// Die Schnittpunkte mit einem anderen Kreis.
    #[pyo3(signature = (o, /))]
    fn intersect(&self, py: Python<'_>, o: &Bound<'_, PyAny>) -> PyResult<Py<PyList>> {
        let Ok(c) = o.extract::<PyRef<PyCircle>>() else {
            // Gemessen nennt die Meldung den **bekommenen** Typ:
            // `Argument must be a CircleType, got int`.
            return Err(PyTypeError::new_err(format!(
                "Argument must be a CircleType, got {}",
                o.get_type().name()?
            )));
        };
        let aus = PyList::empty(py);
        for p in self.inner.intersect_circle(&c.inner) {
            aus.append(p)?;
        }
        Ok(aus.unbind())
    }

    // ----- Verschieben und Drehen -----------------------------------------

    #[pyo3(signature = (*args))]
    fn r#move<'py>(
        slf: &Bound<'py, PyCircle>,
        args: &Bound<'_, PyTuple>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let d = versatz(args, "move")?;
        let c = slf.borrow().inner.move_by(d.0, d.1);
        PyCircle::wie_typ(slf, c)
    }

    #[pyo3(signature = (*args))]
    fn move_ip(&mut self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let d = versatz(args, "move_ip")?;
        self.inner = self.inner.move_by(d.0, d.1);
        Ok(())
    }

    /// Gedreht - und **ein ausdrueckliches `None` ist kein Weglassen**.
    ///
    /// Gemessen: `c.rotate(1, None)` gibt
    /// `TypeError: Invalid rotation point argument, ...`, waehrend
    /// `c.rotate(1)` um den eigenen Mittelpunkt dreht. In PyO3 kommt
    /// beides als `Option::None` an; der Unterschied laesst sich nur
    /// ueber ein `Bound<PyAny>` sehen, das dann auf `is_none()` geprueft
    /// wird. Derselbe Stolperstein wie bei `Event`-Feldern und bei
    /// `transform`s `dest_surface`.
    #[pyo3(signature = (grad, um=Weitergabe::Fehlt, /))]
    fn rotate<'py>(
        slf: &Bound<'py, PyCircle>,
        grad: &Bound<'_, PyAny>,
        um: Weitergabe<'_>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let grad = winkel(grad)?;
        let mitte = slf.borrow().inner.center();
        let p = drehpunkt(um.als_option(), mitte)?;
        let c = slf.borrow().inner.rotate(grad, p);
        PyCircle::wie_typ(slf, c)
    }

    #[pyo3(signature = (grad, um=Weitergabe::Fehlt, /))]
    fn rotate_ip(&mut self, grad: &Bound<'_, PyAny>, um: Weitergabe<'_>) -> PyResult<()> {
        let grad = winkel(grad)?;
        let p = drehpunkt(um.als_option(), self.inner.center())?;
        self.inner = self.inner.rotate(grad, p);
        Ok(())
    }

    /// Setzt alles neu - dieselben Bauweisen wie beim Anlegen.
    /// Setzt alles neu - dieselben Bauweisen wie beim Anlegen.
    ///
    /// **Erst auswerten, dann schreiben.** `c.update(c)` ist erlaubt und
    /// geht mit `&mut self` nicht: das Auswerten braeuchte den Kreis noch
    /// einmal lesend, und `RefCell` laesst das nicht zu. Mit `&Bound` ist
    /// die Auswertung fertig, bevor ueberhaupt jemand schreibend
    /// zugreift.
    #[pyo3(signature = (*args))]
    fn update(slf: &Bound<'_, PyCircle>, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        if args.is_empty() {
            return Err(PyTypeError::new_err(
                "Circle.update requires a circle or CircleLike object",
            ));
        }
        // **Der Satz von `update`, nicht der des Erzeugers.** Gemessen
        // gibt `c.update('x')` `Circle.update requires a circle or
        // CircleLike object`, waehrend `Circle('x')` den langen Satz
        // ueber die Bauweisen gibt - derselbe Weg, zwei Saetze.
        let neu = PyCircle::new(args, None, None, None).map_err(|_| {
            PyTypeError::new_err("Circle.update requires a circle or CircleLike object")
        })?;
        slf.borrow_mut().inner = neu.inner;
        Ok(())
    }

    // ----- Darstellung ----------------------------------------------------

    fn __repr__(&self) -> String {
        let c = &self.inner;
        format!("Circle(({:?}, {:?}), {:?})", c.x, c.y, c.r)
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    fn __eq__(&self, o: &Bound<'_, PyAny>) -> bool {
        o.extract::<PyRef<PyCircle>>()
            .map(|c| c.inner == self.inner)
            .unwrap_or(false)
    }

    fn __copy__(&self) -> PyCircle {
        self.copy()
    }
}

/// Ein Rechteck aus irgendetwas Rechteckigem - `Rect`, `FRect` oder
/// ein Viertupel.
///
/// `PyRect` und `PyFRect` geben ihr Kernrechteck verschieden heraus: das
/// eine ueber `rect_aus_objekt` (es nimmt gleich jede Rechteckform an),
/// das andere ueber `wert()`. Die Reihenfolge ist wichtig - ein `FRect`
/// wuerde sonst ueber den ganzzahligen Weg laufen und seine
/// Nachkommastellen verlieren.
fn rechteck_von(o: &Bound<'_, PyAny>) -> Option<(f64, f64, f64, f64)> {
    if let Ok(r) = o.extract::<PyRef<PyFRect>>() {
        let t = r.wert();
        return Some((t.x as f64, t.y as f64, t.w as f64, t.h as f64));
    }
    if o.extract::<PyRef<PyRect>>().is_ok() {
        let t = crate::rechteck::rect_aus_objekt(o)?;
        return Some((t.x as f64, t.y as f64, t.w as f64, t.h as f64));
    }
    None
}

/// Ein Kreis aus `(*args)` - `(c)`, `((x,y), r)` oder `(x, y, r)`.
fn kreis_aus_argumenten(args: &Bound<'_, PyTuple>) -> PyResult<Circle> {
    match args.len() {
        1 => PyCircle::aus(&args.get_item(0)?),
        2 => {
            let m = punkt(&args.get_item(0)?)?;
            Ok(Circle::new(m.0, m.1, args.get_item(1)?.extract()?))
        }
        3 => Ok(Circle::new(
            args.get_item(0)?.extract()?,
            args.get_item(1)?.extract()?,
            args.get_item(2)?.extract()?,
        )),
        _ => Err(PyTypeError::new_err("expected a Circle")),
    }
}

/// Ein Rechteck aus `(*args)` - ein `Rect` oder vier Zahlen.
fn rechteck_aus_argumenten(args: &Bound<'_, PyTuple>) -> PyResult<(f64, f64, f64, f64)> {
    if args.len() == 1 {
        let o = args.get_item(0)?;
        if let Some(t) = rechteck_von(&o) {
            return Ok(t);
        }
        if o.len().map(|n| n == 4).unwrap_or(false) {
            return Ok((
                o.get_item(0)?.extract()?,
                o.get_item(1)?.extract()?,
                o.get_item(2)?.extract()?,
                o.get_item(3)?.extract()?,
            ));
        }
    }
    if args.len() == 4 {
        return Ok((
            args.get_item(0)?.extract()?,
            args.get_item(1)?.extract()?,
            args.get_item(2)?.extract()?,
            args.get_item(3)?.extract()?,
        ));
    }
    // **Zwei Punkte** gehen auch: `colliderect((1, 2), (3, 4))` meint
    // Ecke und Groesse. Gemessen.
    if args.len() == 2 {
        let a = punkt(&args.get_item(0)?)?;
        let b = punkt(&args.get_item(1)?)?;
        return Ok((a.0, a.1, b.0, b.1));
    }
    Err(PyTypeError::new_err(
        "Invalid rect argument, must be a Rect or a sequence of 4 numbers",
    ))
}

/// Der Drehpunkt: weggelassen heisst "der eigene Mittelpunkt",
/// ausdrueckliches `None` ist ein **Fehler**.
fn drehpunkt(um: Option<&Bound<'_, PyAny>>, mitte: (f64, f64)) -> PyResult<(f64, f64)> {
    match um {
        None => Ok(mitte),
        Some(o) if o.is_none() => Err(PyTypeError::new_err(DREHPUNKT)),
        Some(o) => punkt(o).map_err(|_| PyTypeError::new_err(DREHPUNKT)),
    }
}

/// Ein Versatz aus `(*args)` - `(dx, dy)` oder `((dx, dy),)`.
fn versatz(args: &Bound<'_, PyTuple>, wie: &str) -> PyResult<(f64, f64)> {
    // **Der Satz nennt die Methode**, und zwar die gerufene: gemessen
    // sagt `move('x')` `move requires a pair of numbers` und
    // `move_ip('x')` `move_ip requires a pair of numbers`. Darum der
    // Name als Argument und nicht ein fester Satz.
    let satz = format!("{wie} requires a pair of numbers");
    let fehler = || PyTypeError::new_err(satz.clone());
    match args.len() {
        1 => punkt_satz(&args.get_item(0)?, &satz),
        2 => Ok((
            args.get_item(0)?.extract().map_err(|_| fehler())?,
            args.get_item(1)?.extract().map_err(|_| fehler())?,
        )),
        _ => Err(fehler()),
    }
}

/// Eine Strecke - `pygame.geometry.Line`.
#[pyclass(name = "Line", module = "rustgame_py.geometry", subclass)]
#[derive(Clone)]
pub struct PyLine {
    pub(crate) inner: Line,
}

impl PyLine {
    fn neu(l: Line) -> PyLine {
        PyLine { inner: l }
    }
}

#[pymethods]
impl PyLine {
    /// **Ohne jedes Argument der Nullzustand** - siehe `PyCircle::new`.
    #[new]
    // Dieselbe Lage wie beim Kreis: `line.pyi` nennt `ax`, `ay`,
    // `bx`, `by`, und pygame nimmt sie als Schluesselwoerter an.
    #[pyo3(signature = (*args, ax=None, ay=None, bx=None, by=None))]
    fn new(
        args: &Bound<'_, PyTuple>,
        ax: Option<f64>,
        ay: Option<f64>,
        bx: Option<f64>,
        by: Option<f64>,
    ) -> PyResult<PyLine> {
        if args.is_empty()
            && ax.is_none()
            && ay.is_none()
            && bx.is_none()
            && by.is_none()
        {
            // Der Nullzustand; der Einwand gegen `Line()` faellt in
            // `_init`.
            return Ok(PyLine::neu(Line::new(0.0, 0.0, 0.0, 0.0)));
        }
        if args.is_empty() {
            if let (Some(ax), Some(ay), Some(bx), Some(by)) = (ax, ay, bx, by) {
                return Ok(PyLine::neu(Line::new(ax, ay, bx, by)));
            }
        }
        let l = match args.len() {
            1 => {
                let o = args.get_item(0)?;
                if let Ok(l) = o.extract::<PyRef<PyLine>>() {
                    l.inner
                } else if o.len().map(|n| n == 4).unwrap_or(false) {
                    Line::new(
                        o.get_item(0)?.extract()?,
                        o.get_item(1)?.extract()?,
                        o.get_item(2)?.extract()?,
                        o.get_item(3)?.extract()?,
                    )
                } else if o.len().map(|n| n == 2).unwrap_or(false) {
                    let a = punkt(&o.get_item(0)?)?;
                    let b = punkt(&o.get_item(1)?)?;
                    Line::new(a.0, a.1, b.0, b.1)
                } else if let Ok(feld) = o.getattr("line") {
                    // Wie beim Kreis: ein Feld, eine Eigenschaft oder eine
                    // Methode namens `line` genuegt.
                    let wert = if feld.is_callable() {
                        feld.call0()?
                    } else {
                        feld
                    };
                    let p = PyLine::new(&PyTuple::new(o.py(), [wert])?, None, None, None, None)?;
                    p.inner
                } else {
                    return Err(PyTypeError::new_err(STRECKE_ARGUMENTE));
                }
            }
            2 => {
                let a = punkt(&args.get_item(0)?)?;
                let b = punkt(&args.get_item(1)?)?;
                Line::new(a.0, a.1, b.0, b.1)
            }
            4 => Line::new(
                args.get_item(0)?.extract()?,
                args.get_item(1)?.extract()?,
                args.get_item(2)?.extract()?,
                args.get_item(3)?.extract()?,
            ),
            _ => {
                return Err(PyTypeError::new_err(STRECKE_ARGUMENTE))
            }
        };
        Ok(PyLine::neu(l))
    }

    /// Was `tp_init` tut - siehe `PyCircle::eigen_init`.
    #[pyo3(name = "_init")]
    #[pyo3(signature = (*args, ax=None, ay=None, bx=None, by=None))]
    fn eigen_init(
        &mut self,
        args: &Bound<'_, PyTuple>,
        ax: Option<f64>,
        ay: Option<f64>,
        bx: Option<f64>,
        by: Option<f64>,
    ) -> PyResult<()> {
        if args.is_empty()
            && ax.is_none()
            && ay.is_none()
            && bx.is_none()
            && by.is_none()
        {
            return Err(PyTypeError::new_err(STRECKE_ARGUMENTE));
        }
        self.inner = PyLine::new(args, ax, ay, bx, by)?.inner;
        Ok(())
    }

    #[getter]
    fn get_ax(&self) -> f64 {
        self.inner.ax
    }
    #[setter]
    fn set_ax(&mut self, o: &Bound<'_, PyAny>) -> PyResult<()> {
        self.inner.ax = zahl_blank(o)?;
        Ok(())
    }
    #[getter]
    fn get_ay(&self) -> f64 {
        self.inner.ay
    }
    #[setter]
    fn set_ay(&mut self, o: &Bound<'_, PyAny>) -> PyResult<()> {
        self.inner.ay = zahl_blank(o)?;
        Ok(())
    }
    #[getter]
    fn get_bx(&self) -> f64 {
        self.inner.bx
    }
    #[setter]
    fn set_bx(&mut self, o: &Bound<'_, PyAny>) -> PyResult<()> {
        self.inner.bx = zahl_blank(o)?;
        Ok(())
    }
    #[getter]
    fn get_by(&self) -> f64 {
        self.inner.by
    }
    #[setter]
    fn set_by(&mut self, o: &Bound<'_, PyAny>) -> PyResult<()> {
        self.inner.by = zahl_blank(o)?;
        Ok(())
    }
    #[getter]
    fn get_a(&self) -> (f64, f64) {
        (self.inner.ax, self.inner.ay)
    }
    #[setter]
    fn set_a(&mut self, o: &Bound<'_, PyAny>) -> PyResult<()> {
        let p = punkt_feld(o)?;
        self.inner.ax = p.0;
        self.inner.ay = p.1;
        Ok(())
    }
    #[getter]
    fn get_b(&self) -> (f64, f64) {
        (self.inner.bx, self.inner.by)
    }
    #[setter]
    fn set_b(&mut self, o: &Bound<'_, PyAny>) -> PyResult<()> {
        let p = punkt_feld(o)?;
        self.inner.bx = p.0;
        self.inner.by = p.1;
        Ok(())
    }
    #[getter]
    fn get_length(&self) -> f64 {
        self.inner.length()
    }

    fn copy(&self) -> PyLine {
        PyLine::neu(self.inner)
    }
    fn __copy__(&self) -> PyLine {
        self.copy()
    }

    fn flip_ab(&self) -> PyLine {
        PyLine::neu(self.inner.flip_ab())
    }
    fn flip_ab_ip(&mut self) {
        self.inner = self.inner.flip_ab();
    }

    #[pyo3(signature = (*args))]
    fn r#move(&self, args: &Bound<'_, PyTuple>) -> PyResult<PyLine> {
        let d = versatz(args, "move")?;
        Ok(PyLine::neu(self.inner.move_by(d.0, d.1)))
    }

    #[pyo3(signature = (*args))]
    fn move_ip(&mut self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let d = versatz(args, "move_ip")?;
        self.inner = self.inner.move_by(d.0, d.1);
        Ok(())
    }

    /// Streckt die Strecke - `scale((faktor, ursprung))`.
    ///
    /// Gemessen gibt ein einzelner Wert
    /// `TypeError: scale requires a sequence of two numbers`.
    #[pyo3(signature = (*args))]
    fn scale(&self, args: &Bound<'_, PyTuple>) -> PyResult<PyLine> {
        let (f, u) = streckung(args, "scale")?;
        Ok(PyLine::neu(self.inner.scale(f, u)))
    }

    #[pyo3(signature = (*args))]
    fn scale_ip(&mut self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let (f, u) = streckung(args, "scale_ip")?;
        self.inner = self.inner.scale(f, u);
        Ok(())
    }

    /// Der Fusspunkt eines Punktes auf der Strecke.
    #[pyo3(signature = (punktobjekt, clamp=true))]
    fn project(&self, punktobjekt: &Bound<'_, PyAny>, clamp: bool) -> PyResult<(f64, f64)> {
        let p = punkt_satz(punktobjekt, "project requires a sequence of two numbers")?;
        Ok(self.inner.project(p.0, p.1, clamp))
    }

    #[pyo3(signature = (*args))]
    fn update(&mut self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        // **Leer ist hier ein Fehler**, seit `new` ohne Argumente den
        // Nullzustand gibt (siehe dort). Gefunden hat das pygames
        // eigener `geometry_test.test_meth_update`, der genau
        // `line.update()` verlangt - eine Stunde nach der Aenderung, und
        // der einzige rote Test von 2364.
        if args.is_empty() {
            return Err(PyTypeError::new_err(
                "Line.update requires a line or LineLike object",
            ));
        }
        // Wie bei `Circle.update`: eigener Satz, gemessen.
        let l = PyLine::new(args, None, None, None, None).map_err(|_| {
            PyTypeError::new_err("Line.update requires a line or LineLike object")
        })?;
        self.inner = l.inner;
        Ok(())
    }

    fn __repr__(&self) -> String {
        let l = &self.inner;
        format!(
            "Line(({:?}, {:?}), ({:?}, {:?}))",
            l.ax, l.ay, l.bx, l.by
        )
    }
    fn __str__(&self) -> String {
        self.__repr__()
    }
    fn __eq__(&self, o: &Bound<'_, PyAny>) -> bool {
        o.extract::<PyRef<PyLine>>()
            .map(|l| l.inner == self.inner)
            .unwrap_or(false)
    }
}

/// Faktor und Ursprung fuer `Line.scale`.
fn streckung(args: &Bound<'_, PyTuple>, wie: &str) -> PyResult<(f64, f64)> {
    // **Der Satz nennt die gerufene Methode** - `scale` oder `scale_ip`,
    // gemessen. Darum der Name als Argument und kein `map_err` am
    // Aufrufer: ein `map_err` um den ganzen Aufruf haette auch den
    // `ValueError` unten verschluckt (`scale_ip(0, 0.5)` gibt
    // `Can only scale by a positive non zero number`), und
    // `geometry_test.test_meth_scale_ip` hat genau das gemeldet.
    // *Ein Fehler, den man gegen einen anderen tauscht, ist kein
    // behobener Fehler.*
    let satz = format!("{wie} requires a sequence of two numbers");
    let fehler = || PyTypeError::new_err(satz.clone());
    match args.len() {
        1 => {
            let o = args.get_item(0)?;
            if o.len().map(|n| n != 2).unwrap_or(true) {
                return Err(fehler());
            }
            Ok((
                o.get_item(0)?.extract().map_err(|_| fehler())?,
                o.get_item(1)?.extract().map_err(|_| fehler())?,
            ))
        }
        2 => Ok((
            args.get_item(0)?.extract().map_err(|_| fehler())?,
            args.get_item(1)?.extract().map_err(|_| fehler())?,
        )),
        _ => Err(fehler()),
    }
    .and_then(|(f, u)| {
        // Gemessen: `Line(0,0,1,1).scale(0, 0)` gibt
        // `ValueError: Can only scale by a positive non zero number`.
        if f <= 0.0 {
            return Err(PyValueError::new_err(
                "Can only scale by a positive non zero number",
            ));
        }
        // Gemessen: `scale(2, -0.1)` gibt ebenfalls einen `ValueError` -
        // der Ursprung ist ein Anteil der Strecke und liegt zwischen den
        // Endpunkten.
        if !(0.0..=1.0).contains(&u) {
            return Err(PyValueError::new_err(
                "Origin must be between 0 and 1",
            ));
        }
        Ok((f, u))
    })
}
