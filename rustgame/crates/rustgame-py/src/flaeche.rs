//! flaeche.rs
//! ==========
//! `pygame.Surface`.
//!
//! ## Warum hier noch ein geteilter Behaelter um die Flaeche liegt
//!
//! `pygame.display.set_mode()` gibt bei jedem Aufruf **dasselbe** Objekt
//! zurueck, und `rustgame_display` bildet das mit einem
//! [`rustgame_display::Screen`] ab. Damit die Anzeigeflaeche und eine
//! gewoehnliche Flaeche in Python **eine** Klasse sind, traegt jede
//! `Surface` hier denselben geteilten Behaelter. Beim Bild aus `set_mode`
//! ist es der des Anzeigesystems, sonst ein frischer.
//!
//! Die Alternative waere ein Enum mit zwei Faellen gewesen - und dann in
//! jeder der vierzig Methoden eine Fallunterscheidung.
//!
//! ## Hier stand `unsendable`, "und das ist die richtige Aussage"
//!
//! Die Begruendung: eine `Surface` teile ihren Pixelspeicher ueber
//! `Rc<RefCell<..>>`, und das gelte nicht ueber Threads hinweg; PyO3 halte
//! das Objekt deshalb an seinen Erzeuger-Thread, statt es zuzulassen und
//! spaeter zu stuerzen. Dazu der Hinweis, der Kern sehe einen Wechsel auf
//! `Arc<Mutex<..>>` vor und dann falle `unsendable` weg.
//!
//! **Der Wechsel ist vollzogen**, und ausgeloest hat ihn ein Fehler, der
//! wuerfelte: `image_test.test_load_bmp_threads` laedt Bilder in zwanzig
//! Faeden und gibt sie an den Hauptfaden weiter. Gebaut wurden sie alle;
//! beim **Wegraeumen** kam
//!
//! ```text
//! RuntimeError: PySurface is unsendable,
//!               but is being dropped on another thread
//! ```
//!
//! Mal zwei Fehler im Lauf, mal keiner - je nachdem, welcher Faden gerade
//! aufraeumte. Die alte Begruendung hatte also recht ("spaeter stuerzen")
//! und uebersah, dass es dafuer keinen eigenen Zugriff braucht: **das
//! Wegraeumen allein genuegt.**
//!
//! Siehe [`Geteilt`] dafuer, warum hier ein `RwLock` steht und im Kern ein
//! `ReentrantMutex<RefCell<..>>` - es sind zwei verschiedene Aufgaben.
//! Und PyO3 0.26 verlangt fuer eine `#[pyclass]` ohne `unsendable` nicht
//! nur `Send`, sondern **`Send + Sync`**; deshalb sind die vier Schalter
//! hier `AtomicBool` und die Sperrliste ein `Mutex`.
//!
//! ## Was am Format gemessen ist
//!
//! | | ohne `SRCALPHA` | mit `SRCALPHA` |
//! |---|---|---|
//! | `get_bitsize()` | 32 | 32 |
//! | `get_bytesize()` | 4 | 4 |
//! | `get_masks()` | `(0xff0000, 0xff00, 0xff, 0)` | `(…, 0xff000000)` |
//! | `get_shifts()` | `(16, 8, 0, 0)` | `(16, 8, 0, 24)` |
//! | `get_losses()` | `(0, 0, 0, 8)` | `(0, 0, 0, 0)` |
//! | `map_rgb((10,20,30))` | `0x0a141e` | `0x…0a141e` mit Alpha |
//!
//! Die `losses` sind der Punkt, den man ohne Messung nicht ahnt: ohne
//! Alphakanal gehen **acht Bit Alpha verloren**, und genau das steht
//! dort - keine Null.
//!
//! Das ist die **Vorgabe**, nicht mehr die einzige Moeglichkeit: seit
//! `Surface((w,h), flags, depth[, masks])` die Tiefe wirklich auswertet,
//! kann eine Flaeche auch 8, 12, 15, 16 oder 24 Bit melden und eine
//! Farbtafel tragen. Gerechnet wird weiterhin in ARGB8888 - siehe
//! [`rustgame_core::surface::Format`], wo steht, was daran gemessen ist
//! und was nur gemeldet wird.

use std::sync::atomic::{AtomicBool, Ordering as Reihe};
use std::sync::{Arc, Mutex};

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Was dasteht, wenn die Sperrliste einer Flaeche vergiftet ist.
///
/// Ein `Mutex` wird vergiftet, wenn ein Faden **waehrend** er ihn haelt
/// abbricht. Hier kann das nur passieren, wenn ohnehin schon etwas
/// zerbrochen ist - dann ist Abbrechen die richtige Antwort und nicht
/// Weitermachen mit einer halben Liste.
const SPERRE: &str = "die Sperrliste einer Flaeche ist vergiftet";

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};

use rustgame_core::color::Color;
use rustgame_core::rect::Rect as KernRect;
use rustgame_core::surface::{Format, Surface as KernSurface, SRCALPHA};

/// `pygame.SRCCOLORKEY` - die Flagge, die zu einer Schluesselfarbe gehoert.
const SRCCOLORKEY: u32 = 4096;

/// Ein Argument, bei dem **weggelassen** und **ausdrueckliches `None`**
/// nicht dasselbe sind.
///
/// PyO3 legt beide zusammen: ein Feld `Option<&Bound<PyAny>>` mit der
/// Vorgabe `None` bekommt fuer `f()` und fuer `f(None)` denselben Wert
/// `None`, weil `Option::extract_bound` bei `None` sofort abbricht. Fuer
/// pygame ist der Unterschied aber ein Fehler wert:
///
/// ```text
/// transform.invert(s)            gibt eine Flaeche
/// transform.invert(s, None)      TypeError: argument 2 must be pygame.surface.Surface, not None
/// s.fblits(liste)                gibt None
/// s.fblits(liste, None)          TypeError: The special_flags parameter must be an int
/// s.blits(liste)                 gibt die Liste
/// s.blits(liste, doreturn=None)  TypeError: 'NoneType' object cannot be interpreted as an integer
/// ```
///
/// Alles gemessen. `Weitergabe` reicht das Objekt **unveraendert**
/// durch - auch `None` - und als Vorgabe im `signature` steht
/// `Weitergabe::Fehlt`, ein Wert, den kein Aufruf erzeugen kann.
pub enum Weitergabe<'py> {
    /// Der Aufrufer hat nichts hingeschrieben.
    Fehlt,
    /// Der Aufrufer hat etwas hingeschrieben - moeglicherweise `None`.
    Da(Bound<'py, PyAny>),
}

impl<'py> FromPyObject<'py> for Weitergabe<'py> {
    fn extract_bound(o: &Bound<'py, PyAny>) -> PyResult<Self> {
        Ok(Weitergabe::Da(o.clone()))
    }
}

impl<'py> Weitergabe<'py> {
    /// Das Objekt, falls eines gereicht wurde - `None` kann darin stecken.
    pub fn als_option(&self) -> Option<&Bound<'py, PyAny>> {
        match self {
            Weitergabe::Fehlt => None,
            Weitergabe::Da(o) => Some(o),
        }
    }
}

use crate::farbe::PyColor;
use crate::frechteck::PyFRect;
use crate::rechteck::PyRect;
use crate::konstanten::misch_art;
use crate::umwandlung::{flagge, belegt, farbe_aus, punkt_aus, rect_aus_opt};

/// Der geteilte Behaelter - derselbe Typ wie [`rustgame_display::Screen`].
/// Wie die Huelle ihre Flaeche haelt.
///
/// ## Hier stand `Rc<RefCell<KernSurface>>`
///
/// Und daran hing `unsendable` an der `#[pyclass]`, und daran hing ein
/// Fehler, der wuerfelte: `image_test.test_load_bmp_threads` laedt Bilder
/// in zwanzig Faeden und gibt sie an den Hauptfaden weiter. Gebaut wurden
/// sie, aber beim **Wegraeumen** kam
///
/// ```text
/// RuntimeError: PySurface is unsendable,
///               but is being dropped on another thread
/// ```
///
/// Mal zwei Fehler, mal keiner - je nachdem, welcher Faden gerade
/// aufraeumte.
///
/// ## Warum ein `RwLock` und nicht dasselbe wie im Kern
///
/// Der Kern nimmt `Arc<ReentrantMutex<RefCell<..>>>` - zwei Stufen, weil
/// er verschachtelt sperrt (ein Blit haelt Quelle und Ziel). Hier geht das
/// **nicht**, und zwar aus einem Grund, der erst beim Uebersetzen auffiel:
///
/// [`PySurface::leihen`] muss einen Waechter herausgeben, dessen
/// Lebensdauer an `&self` haengt. Bei einer Stufe geht das - ein
/// `RwLockReadGuard<'_, T>` leiht von `&self`. Bei zwei Stufen leiht der
/// innere `Ref` vom **aeusseren Waechter** und nicht von `&self`, und der
/// ist beim Verlassen der Funktion weg. Zwei Stufen brauchen also einen
/// selbstbezueglichen Typ, und den gibt es in sicherem Rust nicht.
///
/// Ein `RwLock` trifft dabei die Semantik von `RefCell` genauer als ein
/// `Mutex`: **viele Leser, ein Schreiber.** `try_read` und `try_write`
/// scheitern da, wo `try_borrow` und `try_borrow_mut` gescheitert sind -
/// und geben denselben Fehler, den pygame dort gibt
/// (`Surfaces must not be locked during blit`). Ein `Mutex` haette aus
/// zwei gleichzeitigen **Lesern** einen Fehler gemacht, den es vorher
/// nicht gab.
pub type Geteilt = Arc<RwLock<KernSurface>>;

/// `pygame.Surface`.
#[pyclass(name = "Surface", module = "rustgame_py.surface", subclass, weakref)]
pub struct PySurface {
    pub(crate) inner: Geteilt,
    /// Die Flaeche, aus der diese hier geschnitten wurde.
    ///
    /// pygame gibt bei `get_parent()` das **Objekt** zurueck, nicht eine
    /// Kopie; deshalb wird der Verweis hier gehalten und nicht aus dem
    /// Kern nachgebaut.
    eltern: Option<Py<PySurface>>,
    /// Hat ein Programm `set_alpha(None)` gerufen?
    ///
    /// Der Kern kann das nicht sagen: eine frische Flaeche mit `SRCALPHA`
    /// und eine, deren Flaechen-Alpha ausgeschaltet wurde, haben dort
    /// beide `alpha == None`. pygame unterscheidet sie aber - gemessen:
    ///
    /// ```text
    /// Surface((4,4), SRCALPHA).get_alpha()   -> 255
    /// s.set_alpha(None); s.get_alpha()       -> None
    /// Surface((4,4)).get_alpha()             -> None
    /// ```
    ///
    /// Im Kern steckt der Unterschied im Feld `mischen`, und das ist von
    /// aussen nicht lesbar. Also fuehrt die Huelle das eine fehlende Bit
    /// selbst mit. Siehe Bericht.
    alpha_aus: AtomicBool,
    /// Ist das die Anzeigeflaeche aus `display.set_mode`?
    ///
    /// Sie ueberlebt `display.quit()` nicht: in pygame gibt danach **jede**
    /// Methode `pygame.error: Surface is not initialized` (gemessen ueber
    /// 27 Methoden; Ausnahmen sind nur `get_locked()`, das `False` liefert,
    /// und `repr`, das `<Surface(Dead Display)>` gibt). Sieben Tests in
    /// `surface_test.py` pruefen ausschliesslich das.
    ///
    /// Gesetzt wird das Feld allein in [`Self::aus_geteilt`], und das ruft
    /// nur `anzeige.rs` fuer den Schirm.
    ist_anzeige: bool,
    /// Wer diese Flaeche gesperrt hat - pygame **zaehlt** Sperren.
    ///
    /// Gemessen: dreimal `lock()` und `get_locks()` gibt `(s, s, s)`,
    /// `get_locked()` bleibt bis zum dritten `unlock()` wahr, und ein
    /// `unlock()` zuviel ist kein Fehler. `sub.lock()` sperrt **auch den
    /// Elternteil**, und dessen Liste enthaelt dann die Teilflaeche, nicht
    /// ihn selbst.
    ///
    /// **`Py<PyAny>`, nicht `Py<PySurface>`** - in der Liste steht nicht
    /// immer eine Flaeche. Gemessen: nach `pygame.PixelArray(s)` enthaelt
    /// `s.get_locks()` **das PixelArray selbst**, nicht `pa.surface`, und
    /// `pa.close()` nimmt es wieder heraus.
    ///
    /// Und zwar **schwach**, siehe [`Sperre`]: haelt die Flaeche ihr
    /// PixelArray fest, stirbt es nie.
    sperren: Mutex<Vec<Sperre>>,
    /// `RLEACCELOK` (8192): eine Schluesselfarbe oder ein Flaechen-Alpha
    /// wurde mit `RLEACCEL` gesetzt.
    ///
    /// SDL merkt sich damit nur die **Erlaubnis**. Gemessen:
    /// `set_colorkey(c, RLEACCEL)` macht aus den Flaggen 12288
    /// (`RLEACCELOK | SRCCOLORKEY`), und `RLEACCEL` selbst ist noch nicht
    /// dabei.
    /// Wurde die Flaeche ueberhaupt eingerichtet?
    ///
    /// `pygame.Surface.__new__(pygame.Surface)` **ohne** Argumente ist
    /// erlaubt und gibt ein Objekt, das sich wie eine Flaeche nach
    /// `display.quit()` verhaelt: jede Auskunft gibt
    /// `pygame.error: Surface is not initialized`, `repr` gibt
    /// `<Surface(Dead Display)>`, nur `get_locked()` antwortet mit `False`.
    /// Gemessen an zwanzig Methoden.
    ///
    /// `surface_test.test_subsurface` legt so eine Flaeche an und verlangt
    /// den Fehler; ohne dieses Feld scheiterte schon das `__new__`.
    uneingerichtet: AtomicBool,
    rle_erlaubt: AtomicBool,
    /// `RLEACCEL` (16384): SDL hat die Flaeche wirklich kodiert.
    ///
    /// Das tut es **erst, wenn sie einmal als Blitquelle gedient hat** -
    /// gemessen: nach `ziel.blit(s, (0,0))` sind die Flaggen 28672, und
    /// `mustlock()` wird wahr. Eine Kopie bekommt `RLEACCELOK` mit, aber
    /// nicht `RLEACCEL`; `convert()` ebenso (das ist der Kern von
    /// `test_solarwolf_rle_usage`).
    rle_aktiv: AtomicBool,
}

/// `pygame.RLEACCEL` - die Bitte, die Flaeche lauflaengenzukodieren.
const RLEACCEL: u32 = 16384;
/// `pygame.RLEACCELOK` - die Erlaubnis dazu.
const RLEACCELOK: u32 = 8192;

/// Ein Eintrag in der Sperrliste einer Flaeche.
///
/// **Schwach, wo es geht.** pygame legt in `surface->locklist` weiche
/// Verweise ab (`PyWeakref_NewRef`), und das ist kein Umweg, sondern die
/// Bedingung dafuer, dass ein `PixelArray` ueberhaupt sterben kann:
///
/// ```text
/// ar = pygame.PixelArray(sf)   # sf haelt ar
/// r = weakref.ref(ar)
/// del ar; gc.collect()
/// r() is None                  # nur mit weichem Verweis wahr
/// ```
///
/// Nicht jedes Objekt laesst sich weich halten - `Surface.lock()` traegt
/// die Flaeche selbst ein. Fuer die bleibt der feste Verweis, und dort
/// entscheidet weiter der Verweiszaehler: haelt ausser dieser Liste
/// niemand mehr einen Verweis, war das Objekt fuer alle anderen schon
/// fort. Ein `Drop` auf einer `#[pyclass]` darf Python nicht anfassen,
/// also wird die Frage umgedreht gestellt.
enum Sperre {
    Fest(Py<PyAny>),
    Schwach(Py<pyo3::types::PyWeakrefReference>),
}

impl Sperre {
    fn neu(_py: Python<'_>, wer: &Bound<'_, PyAny>) -> Sperre {
        match pyo3::types::PyWeakrefReference::new(wer) {
            Ok(w) => Sperre::Schwach(w.unbind()),
            Err(_) => Sperre::Fest(wer.clone().unbind()),
        }
    }

    /// Gibt es den Halter noch?
    fn lebt(&self, py: Python<'_>) -> bool {
        match self {
            Sperre::Fest(o) => o.get_refcnt(py) > 1,
            Sperre::Schwach(w) => w.bind(py).upgrade().is_some(),
        }
    }

    /// Der Halter selbst - `None`, wenn er fort ist.
    fn wer(&self, py: Python<'_>) -> Option<Py<PyAny>> {
        match self {
            Sperre::Fest(o) => Some(o.clone_ref(py)),
            Sperre::Schwach(w) => w.bind(py).upgrade().map(|o| o.unbind()),
        }
    }
}

impl PySurface {
    /// Nimmt Sperren heraus, deren Halter nicht mehr da ist.
    ///
    /// pygame loest die Sperre eines `BufferProxy`, wenn niemand ihn mehr
    /// hat - `v = None; gc.collect()` genuegt dort.
    ///
    /// Das ist keine Feinheit - ohne das Aufraeumen bliebe eine Flaeche
    /// nach dem ersten `get_buffer()` **fuer immer** gesperrt, und ein
    /// Blit auf sie waere danach ein Fehler.
    fn sperren_aufraeumen(&self, py: Python<'_>) {
        self.sperren.lock().expect(SPERRE).retain(|x| x.lebt(py));
    }

    /// Erklaert die Flaeche fuer tot - fuer `Window.destroy()`.
    ///
    /// Danach antwortet sie wie eine Flaeche nach `display.quit()`: jede
    /// Auskunft gibt `pygame.error: Surface is not initialized`. Gemessen
    /// an `window_test.test_window_surface`, das genau diesen Fehler von
    /// einem `surf.fill(...)` nach dem `destroy()` erwartet.
    pub(crate) fn toeten(&self) {
        self.uneingerichtet.store(true, Reihe::Relaxed);
    }

    /// Traegt ein fremdes Objekt als Sperre ein - fuer `PixelArray`.
    pub(crate) fn sperre_eintragen(&self, py: Python<'_>, wer: &Bound<'_, PyAny>) {
        self.sperren.lock().expect(SPERRE).push(Sperre::neu(py, wer));
    }


    /// Nimmt die **letzte** Sperre dieses Halters heraus - fuer
    /// `Surface.unlock()`.
    ///
    /// Steht keine drin, passiert nichts; gemessen ist ein `unlock()`
    /// zuviel kein Fehler.
    fn sperre_loesen_letzte(&self, py: Python<'_>, wer: &Py<PyAny>) {
        let mut liste = self.sperren.lock().expect(SPERRE);
        if let Some(i) = liste.iter().rposition(|x| {
            x.wer(py).map(|o| o.bind(py).is(wer.bind(py))).unwrap_or(false)
        }) {
            liste.remove(i);
        }
    }

    /// Nimmt es wieder heraus. Steht es nicht drin, passiert nichts -
    /// gemessen ist ein zweites `close()` kein Fehler.
    pub(crate) fn sperre_loesen(&self, py: Python<'_>, wer: &Py<PyAny>) {
        let mut liste = self.sperren.lock().expect(SPERRE);
        if let Some(i) = liste
            .iter()
            .position(|x| x.wer(py).map(|o| o.bind(py).is(wer.bind(py))).unwrap_or(false))
        {
            liste.remove(i);
        }
    }

    /// Eine frische Flaeche mit eigenem Behaelter.
    pub fn neu(s: KernSurface) -> PySurface {
        PySurface {
            inner: Arc::new(RwLock::new(s)),
            eltern: None,
            alpha_aus: AtomicBool::new(false),
            ist_anzeige: false,
            sperren: Mutex::new(Vec::new()),
            uneingerichtet: AtomicBool::new(false),
            rle_erlaubt: AtomicBool::new(false),
            rle_aktiv: AtomicBool::new(false),
        }
    }

    /// Eine frische Flaeche, die die **RLE-Erlaubnis** eines Vorbilds
    /// uebernimmt.
    ///
    /// Fuer `transform.flip`, `.scale` und `.rotate`: gemessen behalten
    /// sie `RLEACCELOK` samt Schluesselfarbe, waehrend `copy()` und
    /// `convert_alpha()` beides anders behandeln.
    ///
    /// ```text
    /// Quelle (SRCALPHA, key, RLEACCEL)   Flaggen 0x13000  key (255,0,0,255)
    /// flip / scale / rotate              Flaggen 0x13000  key (255,0,0,255)
    /// copy / convert_alpha               Flaggen 0x12000  key None
    /// ```
    ///
    /// Die Erlaubnis (`RLEACCELOK`) geht mit, die tatsaechliche Kodierung
    /// (`RLEACCEL`) nicht - die entsteht erst beim naechsten Blit.
    /// Ohne diese Zeile faellt `transform_test.test_flip_alpha`: dort wird
    /// die gedrehte Flaeche geblittet, und ohne RLE gilt ihre
    /// Schluesselfarbe wieder.
    pub fn neu_wie(s: KernSurface, vorbild: &PySurface) -> PySurface {
        let neu = PySurface::neu(s);
        neu.rle_erlaubt
            .store(vorbild.rle_erlaubt.load(Reihe::Relaxed), Reihe::Relaxed);
        neu
    }

    /// Eine Flaeche um einen schon vorhandenen Behaelter - so kommt die
    /// Anzeigeflaeche aus `set_mode` herein.
    pub fn aus_geteilt(s: Geteilt) -> PySurface {
        PySurface {
            inner: s,
            eltern: None,
            alpha_aus: AtomicBool::new(false),
            ist_anzeige: true,
            sperren: Mutex::new(Vec::new()),
            uneingerichtet: AtomicBool::new(false),
            rle_erlaubt: AtomicBool::new(false),
            rle_aktiv: AtomicBool::new(false),
        }
    }

    /// Lebt die Flaeche noch?
    ///
    /// Nur die Anzeigeflaeche kann sterben, und zwar an `display.quit()`.
    /// Siehe das Feld `ist_anzeige`.
    fn lebt(&self) -> PyResult<()> {
        if self.uneingerichtet.load(Reihe::Relaxed) {
            return Err(crate::umwandlung::PygameError::new_err(
                "Surface is not initialized",
            ));
        }
        if self.ist_anzeige && !rustgame_display::display::get_init() {
            return Err(crate::umwandlung::PygameError::new_err(
                "Surface is not initialized",
            ));
        }
        Ok(())
    }

    /// Lesender Zugriff. Scheitert statt zu stuerzen, wenn dieselbe
    /// Flaeche gerade schreibend ausgeliehen ist.
    ///
    /// **Hier sitzt auch die Lebendpruefung**, und mit Absicht: jede
    /// Methode dieser Klasse geht durch `leihen` oder `leihen_mut`, und
    /// jede von ihnen soll nach `display.quit()` scheitern. Die Pruefung
    /// vierzigmal von Hand einzutragen waere die Art Wiederholung, bei der
    /// genau eine Stelle vergessen wird.
    pub fn leihen(&self) -> PyResult<RwLockReadGuard<'_, KernSurface>> {
        self.lebt()?;
        self.inner.try_read().ok_or_else(belegt)
    }

    /// Schreibender Zugriff.
    pub fn leihen_mut(&self) -> PyResult<RwLockWriteGuard<'_, KernSurface>> {
        self.lebt()?;
        self.inner.try_write().ok_or_else(belegt)
    }

    /// Eine neue Flaeche **im Typ des Aufrufers**.
    ///
    /// pygame erhaelt die Unterklasse: `copy`, `convert`, `convert_alpha`,
    /// `premul_alpha` und `subsurface` geben alle ein Objekt der eigenen
    /// Klasse zurueck (gemessen). Gebaut wird es mit `tp_new` **ohne**
    /// `tp_init` - deshalb hier `type(self).__new__(type(self), groesse)`
    /// und kein Aufruf der Klasse selbst, der `__init__` einer Unterklasse
    /// mitausloesen wuerde.
    ///
    /// Die Groesse wird `__new__` mitgegeben, obwohl es sie seit dem
    /// Umbau von [`PySurface::new`] **nicht ansieht** - genau wie pygames
    /// `tp_new`. Sie bleibt stehen, damit ein eigenes `__new__` einer
    /// Unterklasse nichts Ueberraschendes sieht.
    ///
    /// **Und `uneingerichtet` gehoert mit in die Uebernahme.** Vorher baute
    /// `__new__` die Flaeche fertig, also war das Feld dort schon `false`;
    /// seit es nur noch den leeren Platz anlegt, steht es auf `true`, und
    /// ohne diese Zeile gaben `copy`, `convert` und `convert_alpha` auf
    /// einer **Unterklasse** eine Flaeche, die `Surface is not
    /// initialized` meldet. Sechs Tests in `surface_test.py` haben das
    /// sofort gesagt - und keine der 804 Proben, denn sie fragen alle die
    /// Klasse selbst und keine Unterklasse.
    fn wie_typ<'py>(
        slf: &Bound<'py, PySurface>,
        kern: KernSurface,
        eltern: Option<Py<PySurface>>,
    ) -> PyResult<Bound<'py, PySurface>> {
        let py = slf.py();
        let rle = slf.borrow().rle_erlaubt.load(Reihe::Relaxed);
        let groesse = kern.get_size();
        let neu = PySurface {
            inner: Arc::new(RwLock::new(kern)),
            eltern,
            alpha_aus: AtomicBool::new(slf.borrow().alpha_aus.load(Reihe::Relaxed)),
            ist_anzeige: false,
            sperren: Mutex::new(Vec::new()),
            uneingerichtet: AtomicBool::new(false),
            rle_erlaubt: AtomicBool::new(rle),
            // Eine Kopie ist noch nie Blitquelle gewesen.
            rle_aktiv: AtomicBool::new(false),
        };
        let typ = slf.get_type();
        if typ.is(py.get_type::<PySurface>()) {
            return Bound::new(py, neu);
        }
        let leer = typ
            .getattr(pyo3::intern!(py, "__new__"))?
            .call1((&typ, groesse))?;
        let leer = leer.downcast_into::<PySurface>()?;
        {
            let mut b = leer.borrow_mut();
            b.inner = neu.inner;
            b.eltern = neu.eltern;
            b.alpha_aus = neu.alpha_aus;
            b.rle_erlaubt = neu.rle_erlaubt;
            b.rle_aktiv = neu.rle_aktiv;
            b.uneingerichtet = neu.uneingerichtet;
        }
        Ok(leer)
    }

    /// Hat die Flaeche einen eigenen Alphakanal?
    fn mit_alpha(&self) -> PyResult<bool> {
        Ok(self.leihen()?.get_flags() & SRCALPHA != 0)
    }

    /// Der eigentliche Blit - mit der Mischart des **Kerns**.
    ///
    /// `blit`, `blits` und `fblits` pruefen ihre Argumente jeweils anders
    /// (drei verschiedene Fehlermeldungen fuer dieselbe Lage), rechnen
    /// danach aber gleich. Deshalb steht die Rechnung hier und nicht in
    /// einer der drei.
    ///
    /// Blittet ein Programm eine Flaeche auf sich selbst, wuerde die
    /// doppelte Ausleihe stuerzen. Dann bekommt der Kern einen flachen Klon -
    /// siehe [`blit_an`](Self::blit_an).
    ///
    /// `quellrechteck` ist der einzige Punkt, an dem die drei doch nicht
    /// gleich rechnen: `surf_blit` und `surf_blits` uebergeben pygames
    /// C-Schicht immer ein Quellrechteck, `surf_fblits` gibt `NULL`. Beim
    /// ueberlappenden Selbstblit entscheidet das ueber die Routine - siehe
    /// `Blitweg::waehlen` im Kern.
    fn blit_roh(
        &self,
        source: &PySurface,
        dest: &Bound<'_, PyAny>,
        area: Option<&Bound<'_, PyAny>>,
        flags: u32,
        quellrechteck: bool,
    ) -> PyResult<PyRect> {
        let p = punkt_aus(dest).map_err(|_| {
            pyo3::exceptions::PyTypeError::new_err("invalid destination position for blit")
        })?;
        self.blit_an(source, p, area, flags, quellrechteck)
    }

    /// Wie [`blit_roh`](Self::blit_roh), nur mit einer Stelle, die schon
    /// gelesen ist.
    ///
    /// Es gibt sie, weil `blit(q)` **ohne** `dest` erlaubt ist und dann
    /// `(0, 0)` gilt. Der Umweg ueber `blit_roh` haette dafuer ein
    /// Python-Tupel bauen muessen, nur um es gleich wieder auseinander zu
    /// nehmen - und die Fehlermeldung von `punkt_aus` koennte dabei
    /// auftauchen, wo es gar kein Argument gab.
    fn blit_an(
        &self,
        source: &PySurface,
        dest: (i32, i32),
        area: Option<&Bound<'_, PyAny>>,
        flags: u32,
        quellrechteck: bool,
    ) -> PyResult<PyRect> {
        // **Eine gesperrte Flaeche laesst sich nicht blitten** - weder als
        // Quelle noch als Ziel. Gemessen gibt beides
        // `pygame.error: Surfaces must not be locked during blit`.
        //
        // Das ist mehr als eine Formsache: `surface_test.test_get_locked`
        // prueft `get_locked()` **gegen** den Blit-Versuch, also ob die
        // beiden Auskuenfte dasselbe sagen. Eine Huelle, die `lock()`
        // mitzaehlt, aber den Blit trotzdem durchlaesst, faellt genau dort
        // auf - und sie ist auch sachlich falsch, denn das Sperren hat nur
        // den Zweck, Zugriffe abzuweisen.
        //
        // `fill` dagegen geht auf einer gesperrten Flaeche weiter, auch
        // gemessen. Die Sperre ist also nicht "kein Schreiben", sondern
        // "kein Blit".
        let gesperrt = Python::attach(|py| self.get_locked(py) || source.get_locked(py));
        if gesperrt {
            return Err(crate::umwandlung::PygameError::new_err(
                "Surfaces must not be locked during blit",
            ));
        }
        // **Eine Teilflaeche laesst sich nicht auf ihre Verwandtschaft
        // blitten** - und die Regel ist asymmetrisch. Gemessen, mit
        // `s` als Flaeche, `a` als ihrer Teilflaeche und `z` als
        // Teilflaeche von `a`:
        //
        // ```text
        // s.blit(a)   Fehler        a.blit(s)   geht
        // a.blit(z)   Fehler        z.blit(a)   Fehler
        // s.blit(z)   Fehler        z.blit(s)   geht
        // s.blit(s)   geht          s.blit(fremde Teilflaeche)  geht
        // ```
        //
        // Die Regel dahinter ist knapp: **die Quelle ist eine Teilflaeche
        // und beide teilen dieselbe Wurzel.** Ob sich die Bereiche
        // ueberschneiden, spielt keine Rolle - `s.blit(a, (10, 10))`
        // scheitert genauso.
        //
        // Dass die Meldung von Sperren redet, ist pygames Innenleben:
        // `pgSurface_Blit` sperrt die Quelle, und das Sperren einer
        // Teilflaeche sperrt die Wurzel mit; das Ziel ist dann belegt. Auf
        // der Python-Seite sieht man davon nichts - `get_locked()` bleibt
        // die ganze Zeit `False`, auch waehrend eine Teilflaeche lebt.
        // Darum steht hier die **gemessene** Bedingung und nicht die
        // erzaehlte.
        {
            let q = source.leihen()?;
            let z = self.leihen()?;
            if q.get_parent().is_some() && q.teilt_speicher(&z) {
                return Err(crate::umwandlung::PygameError::new_err(
                    "Surfaces must not be locked during blit",
                ));
            }
        }
        // **Hier wird die Quelle lauflaengenkodiert** - genauer: hier
        // merkt sie sich, dass SDL es getan haette.
        //
        // `set_colorkey(c, RLEACCEL)` gibt nur die **Erlaubnis**
        // (`RLEACCELOK`); kodiert wird erst, wenn die Flaeche das erste
        // Mal als Blitquelle dient. Gemessen:
        //
        // ```text
        // s.set_colorkey(c, RLEACCEL)   flags 0x3000   mustlock() False
        // ziel.blit(s, (0, 0))          flags 0x7000   mustlock() True
        // ```
        //
        // Die ganze Buchfuehrung dafuer stand schon (`rle_erlaubt`,
        // `rle_aktiv`, samt Vermerken, die genau das beschreiben) - nur
        // hat sie niemand umgelegt. Sechs Tests in `surface_test.py`
        // haengen daran.
        //
        // rustgame kodiert dabei **nichts**: es gibt keine
        // Lauflaengenkodierung im Kern, und sie waere hier auch ohne
        // Wirkung, weil nichts den Speicher nach Bereichen durchsucht. Was
        // nach aussen zaehlt, ist die Auskunft - `get_flags()` und
        // `mustlock()` -, und die stimmt damit. Der Unterschied ist Tempo,
        // kein Verhalten.
        if source.rle_erlaubt.load(Reihe::Relaxed) {
            source.rle_aktiv.store(true, Reihe::Relaxed);
        }
        let a = rect_aus_opt(area)?;
        // Blittet ein Programm eine Flaeche auf sich selbst, wuerde die
        // doppelte Ausleihe stuerzen - dann bekommt der Kern einen **flachen
        // Klon**. `Rc::ptr_eq` und nicht ein Vergleich der Groessen: eine
        // Teilflaeche teilt denselben Speicher und muss genauso behandelt
        // werden.
        //
        // **Flach, nicht `copy()`** - und das war ein Fehler, der sich hinter
        // einem richtigen Satz versteckte: "dann wird die Quelle vorher
        // kopiert" loest die Ausleihe, nimmt dem Kern aber die Auskunft, die
        // er braucht. Der Kern erkennt den Selbstblit an `Rc::ptr_eq` auf dem
        // Pixelpuffer, und eine echte Kopie hat einen eigenen. Damit fiel
        // pygames Sonderweg fuer den ueberlappenden Selbstblit aus, der
        // anders rundet als SDLs - siehe `Blitweg::waehlen`. Ein `Clone` teilt
        // den Puffer und laesst `RefCell` in Ruhe, weil die Ausleihe mit dem
        // `let` endet; ueber die Zwischenkopie der Zeilen wacht der Kern
        // selbst (`blit_selbst`).
        if Arc::ptr_eq(&self.inner, &source.inner) {
            let quelle = self.leihen()?.clone();
            let mut ziel = self.leihen_mut()?;
            return Ok(PyRect::neu(if quellrechteck {
                ziel.blit_ex(&quelle, dest, a, flags)
            } else {
                ziel.fblit(&quelle, dest, flags)
            }));
        }
        // **Alphakanal plus lauflaengenkodierte Schluesselfarbe: die
        // Schluesselfarbe faellt weg.** Gemessen, und zwar genau in dieser
        // einen Verbindung - rot mit Alpha 255 auf weiss, Schluessel rot:
        //
        // ```text
        // SRCALPHA, key            -> weiss   (uebersprungen)
        // SRCALPHA, key + RLEACCEL -> **rot** (gezeichnet)
        // SRCALPHA, key, Alpha 128 -> weiss
        // ohne SRCALPHA, key       -> weiss
        // ohne SRCALPHA, key+RLE   -> weiss
        // ```
        //
        // Der Grund liegt in SDL: eine Flaeche **mit** Alphakanal wird von
        // `SDL_RLESurface` in der Alpha-Spielart kodiert, und die kennt
        // keine Schluesselfarbe mehr. Wer nur "RLE ist eine Beschleunigung"
        // denkt, kommt darauf nicht - es aendert das Ergebnis.
        //
        // Die Buchfuehrung ueber RLE steht in der Huelle, die Blitwahl im
        // Kern; deshalb wird hier eine flache Kopie **ohne** Schluesselfarbe
        // weitergereicht, statt den Kern um RLE zu erweitern.
        let q = source.leihen()?;
        let ohne_schluessel = if q.has_alpha()
            && source.rle_erlaubt.load(Reihe::Relaxed)
            && q.get_colorkey().is_some()
        {
            let mut k = q.clone();
            k.set_colorkey(None::<rustgame_core::color::Color>);
            Some(k)
        } else {
            None
        };
        let quelle: &rustgame_core::surface::Surface = ohne_schluessel.as_ref().unwrap_or(&q);
        let mut ziel = self.leihen_mut()?;
        Ok(PyRect::neu(if quellrechteck {
            ziel.blit_ex(quelle, dest, a, flags)
        } else {
            ziel.fblit(quelle, dest, flags)
        }))
    }

    /// Eine Farbe **gegen diese Flaeche** aufgeloest.
    ///
    /// Der Unterschied zu `farbe_aus` ist eine blanke Zahl: die ist in
    /// pygame kein RGBA-Wort, sondern der **abgebildete Pixelwert dieser
    /// Flaeche**. Gemessen gibt `fill(5)` auf einer ARGB-Flaeche
    /// `(0, 0, 5, 0)`, nicht `(5, 5, 5, 255)` - und auf einer Flaeche mit
    /// anderen Masken etwas Drittes. Darum haengt die Auskunft am Ziel und
    /// kann nicht vorher feststehen.
    ///
    /// Es ist derselbe Helfer, den `pygame.draw` benutzt; dort steckt auch
    /// die Aufteilung der Fehlerarten - `ValueError` fuer eine Folge, die
    /// keine Farbe hergibt, `TypeError` fuer alles andere.
    fn farbe_oder_wert(&self, c: &Bound<'_, PyAny>) -> PyResult<Color> {
        let k = self.leihen()?;
        crate::zeichnen::farbe_fuer(&k, c)
    }
}

#[pymethods]
impl PySurface {
    /// `Surface((w, h)[, flags[, depth[, masks]]])`.
    ///
    /// **Vier** Argumente, nicht drei - hier standen lange nur `size`,
    /// `flags` und ein `depth`, das verworfen wurde ("rustgame rechnet
    /// ohnehin immer in ARGB8888"). Das Rechnen stimmt weiterhin; die
    /// **Auskunft** stimmte nicht: pygames
    /// `transform_test.test_scale__destination` baut
    /// `Surface((64,64), SRCALPHA, 32, (0xFF000000, 0xFF0000, 0xFF00, 0xFF))`
    /// und verlangt danach, dass `get_shifts()` etwas anderes meldet als
    /// bei der gewoehnlichen Flaeche. Seit [`Format`] traegt der Kern die
    /// Angabe mit.
    ///
    /// `depth` darf auch eine **Flaeche** sein - dann wird deren Format
    /// uebernommen (pygames "pixel format as surface").
    /// `Surface.__new__(...)` - **legt nur den leeren Platz an.**
    ///
    /// ## Hier stand die Arbeit, und das war eine benannte Grenze
    ///
    /// Vorher baute diese Methode die Flaeche fertig, und daneben stand
    /// die Grenze `Surface() ohne Groesse`: PyO3 bilde `__init__` nicht
    /// auf `tp_init` ab, also seien `Surface()` und
    /// `Surface.__new__(Surface)` nicht zu unterscheiden; eines von beiden
    /// muesse falsch sein, und gewaehlt sei die Seite mit dem Test.
    ///
    /// **Die Begruendung war falsch.** Eine PyO3-Klasse ist ein
    /// **Heap-Typ**, und bei einem Heap-Typ setzt
    /// `setattr(Klasse, "__init__", f)` den Schlitz mit (CPythons
    /// `type_setattro` ruft `fixup_slot_dispatchers`). Aufgefallen ist
    /// das beim Aufraeumen von `Sound` und `PixelArray`, wo dieselbe
    /// Begruendung eine Python-Unterklasse getragen hatte.
    ///
    /// Jetzt ist die Aufteilung dieselbe wie bei pygame, und sie ist
    /// gemessen:
    ///
    /// ```text
    /// Surface.__new__(S, (4,4)).get_size()  error: Surface is not initialized
    /// Surface()                             TypeError: function missing
    ///                                       required argument 'size' (pos 1)
    /// Surface((4,4)).get_size()             (4, 4)
    /// ```
    ///
    /// `tp_new` **sieht seine Argumente gar nicht an** - auch das ist
    /// gemessen, nicht angenommen. Nebenwirkung: eine Unterklasse darf
    /// jetzt `def __init__(self, w, h): super().__init__((w, h))`
    /// schreiben, was vorher am eifrigen `__new__` scheiterte.
    #[new]
    #[pyo3(signature = (*_args, **_kwargs))]
    fn new(_args: &Bound<'_, PyAny>, _kwargs: Option<&Bound<'_, PyAny>>) -> Self {
        let leer = PySurface::neu(KernSurface::new((0, 0)));
        leer.uneingerichtet.store(true, Reihe::Relaxed);
        leer
    }

    /// `s.__init__((9, 9))` macht aus einer 4x4 wirklich eine 9x9.
    ///
    /// Bei pygame ist `__init__` **kein Leerlauf**, sondern setzt die
    /// Flaeche neu - gemessen. Sichtbar wird das an zwei Stellen: beim
    /// Ableiten (`super().__init__(...)` landet sonst bei
    /// `object.__init__` und meldet `takes exactly one argument`, woran
    /// `surface_test.py` reihenweise scheitert) und beim Modulvergleich,
    /// wo die Probe `__init__ setzt neu` bis zuletzt die einzige
    /// Abweichung war.
    ///
    /// Der geteilte Behaelter wird **nicht** ausgetauscht, sondern sein
    /// Inhalt ersetzt: bei der Anzeigeflaeche aus `set_mode` ist es der
    /// des Anzeigesystems, und ein neuer `Rc` haette die Verbindung
    /// gekappt.
    ///
    /// Die Elternschaft faellt weg, und das ist gemessen, nicht geraten:
    ///
    /// ```text
    /// p = Surface((8,8)); sub = p.subsurface((0,0,4,4))
    /// sub.__init__((9,9)) -> sub.get_size() (9,9), sub.get_parent() None,
    ///                        p.get_size()   (8,8)
    /// ```
    /// Mit Unterstrich, weil `__init__` selbst in Python steht
    /// (`python/surface.py`): dort haengt die Pruefung auf eine Groesse,
    /// und von dort wird diese Methode gerufen. Siehe [`Self::new`].
    #[pyo3(signature = (size, flags=0, depth=None, masks=None))]
    fn _init(
        &mut self,
        size: &Bound<'_, PyAny>,
        flags: i64,
        depth: Option<&Bound<'_, PyAny>>,
        masks: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        // **Die Groesse darf Fliesskomma sein** - gemessen nimmt pygame
        // `Surface((16.0, 16.0))` an und schneidet ab.
        // `surface_test.test_get_frect` faengt genau damit an, und mit
        // einem `(u32, u32)`-Beiwert gab es dort
        // `TypeError: argument 'size': 'float' object cannot be
        // interpreted as an integer` - ein Fehler beim **Anlegen**, obwohl
        // der Test von `get_frect` handelt.
        // **Pygames Wortlaut, und pygames Fehlerart.** `versatz_aus` gibt
        // `TypeError: offset must be two numbers` - richtig fuer einen
        // Versatz, falsch hier. Gemessen antwortet pygame auf **jede**
        // schlechte Groesse (`1`, `(1,)`, `"ab"`, `(1,2,3)`, `None`) mit
        // `ValueError: size needs to be (number width, number height)`.
        //
        // Aufgefallen ist das erst, als `__init__` nach Python wanderte und
        // die Groesse hier geprueft wurde statt in `__new__` - der Satz war
        // vorher schon falsch, nur hat ihn keine Probe gelesen.
        let p = crate::umwandlung::versatz_aus(size).map_err(|_| {
            pyo3::exceptions::PyValueError::new_err(
                "size needs to be (number width, number height)",
            )
        })?;
        let size = (p.0.max(0) as u32, p.1.max(0) as u32);
        let (format, tafel) = format_aus(flagge(flags), depth, masks)?;
        let mut k = KernSurface::with_flags(size, flagge(flags));
        k.set_format(format);
        k.set_palette(tafel);
        *self.inner.try_write().ok_or_else(belegt)? = k;
        // Eine mit `__new__` ohne Groesse angelegte Flaeche wird hier
        // endlich eingerichtet - gemessen: `s.__init__((4, 4))` macht aus
        // der toten eine lebendige mit `get_size() == (4, 4)`.
        self.uneingerichtet.store(false, Reihe::Relaxed);
        self.eltern = None;
        self.alpha_aus.store(false, Reihe::Relaxed);
        self.rle_erlaubt.store(false, Reihe::Relaxed);
        self.rle_aktiv.store(false, Reihe::Relaxed);
        self.sperren.lock().expect(SPERRE).clear();
        Ok(())
    }

    // ----- Groesse und Format --------------------------------------------

    fn get_width(&self) -> PyResult<u32> {
        Ok(self.leihen()?.get_width())
    }
    fn get_height(&self) -> PyResult<u32> {
        Ok(self.leihen()?.get_height())
    }
    fn get_size(&self) -> PyResult<(u32, u32)> {
        Ok(self.leihen()?.get_size())
    }
    /// Die Flaggen der Flaeche.
    ///
    /// `SRCCOLORKEY` (4096) kommt dazu, sobald eine Schluesselfarbe
    /// gesetzt ist - gemessen: `Surface((4,4))` hat 0,
    /// `s.set_colorkey((1,2,3))` macht daraus 4096, `set_colorkey(None)`
    /// wieder 0, und zusammen mit `SRCALPHA` sind es 69632. Der Kern
    /// fuehrt die Flagge nicht mit; hier ist sie aus der Schluesselfarbe
    /// zurueckgerechnet, was denselben Wert ergibt. Siehe Bericht.
    fn get_flags(&self) -> PyResult<u32> {
        let s = self.leihen()?;
        let mut f = s.get_flags();
        if s.get_colorkey().is_some() {
            f |= SRCCOLORKEY;
        }
        // Die beiden RLE-Flaggen fuehrt die Huelle selbst mit - siehe die
        // Felder `rle_erlaubt` und `rle_aktiv`. Eine **Teilflaeche** hat
        // keine von beiden, auch wenn ihr Elternteil sie hat (gemessen:
        // `test_subsurface_rle` und `test_subsurface_rle2` pruefen genau
        // das).
        if self.eltern.is_none() {
            if self.rle_erlaubt.load(Reihe::Relaxed) {
                f |= RLEACCELOK;
            }
            if self.rle_aktiv.load(Reihe::Relaxed) {
                f |= RLEACCEL;
            }
        }
        Ok(f)
    }

    /// Bits je Pixel. 32, solange niemand eine andere Tiefe verlangt hat.
    fn get_bitsize(&self) -> PyResult<u32> {
        Ok(self.leihen()?.get_bitsize())
    }
    /// Bytes je Pixel.
    fn get_bytesize(&self) -> PyResult<u32> {
        Ok(self.leihen()?.get_bytesize())
    }
    /// Zeilenlaenge in Bytes.
    ///
    /// Bei einer Teilflaeche meldet pygame die Zeilenlaenge des
    /// **Elternpuffers**; der Kern gibt den nicht her, also steht hier die
    /// eigene Breite. Siehe Bericht.
    fn get_pitch(&self) -> PyResult<u32> {
        Ok(self.leihen()?.get_pitch())
    }

    /// `Surface._pixels_address` - die Stelle des ersten Pixels.
    ///
    /// In pygame eine Eigenschaft (`surface_getsets`), keine Methode, und
    /// sie wird von seiner eigenen Testsuite an vier Stellen gebraucht:
    /// `_check_interface_2D` und Geschwister vergleichen sie mit dem
    /// `data`-Feld der numpy-Schnittstelle.
    ///
    /// Die Zahl ist ehrlich - dort liegt wirklich der Puffer -, nur
    /// beschreibt sie rustgames Ablage: **immer vier Bytes je Pixel**.
    /// Siehe [`rustgame_core::surface::Surface::pixel_zeiger`].
    #[getter]
    fn _pixels_address(&self) -> PyResult<usize> {
        Ok(self.leihen()?.pixel_zeiger())
    }

    /// Wo Rot, Gruen, Blau und Alpha im Pixel sitzen.
    fn get_masks(&self) -> PyResult<(u32, u32, u32, u32)> {
        Ok(self.leihen()?.get_masks())
    }
    /// Um wie viel jeder Kanal verschoben ist.
    fn get_shifts(&self) -> PyResult<(u32, u32, u32, u32)> {
        Ok(self.leihen()?.get_shifts())
    }
    /// Wie viele Bit je Kanal verlorengehen. Ohne Alphakanal sind es acht.
    fn get_losses(&self) -> PyResult<(u32, u32, u32, u32)> {
        Ok(self.leihen()?.get_losses())
    }

    /// Die Farbtafel einer Palettenflaeche.
    ///
    /// Gemessen: ohne Palette gibt pygame
    /// `pygame.error: Surface has no palette to get`, mit Palette ein
    /// Tupel von 256 Farben.
    fn get_palette<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let s = self.leihen()?;
        let Some(t) = s.get_palette() else {
            return Err(crate::umwandlung::PygameError::new_err(
                "Surface has no palette to get",
            ));
        };
        // **Laenge 3**, nicht 4 - und das ist keine Kuerzung der Farbe.
        //
        // Gemessen: `s.get_palette()[1]` ist ein `Color`, dessen `repr`
        // `Color(0, 0, 85, 255)` lautet und dessen `a` 255 ist, dessen
        // `len()` aber **3** ist. pygames `Color` traegt eine einstellbare
        // Laenge (das alte `set_length`), und die Tafel benutzt sie: der
        // Alphakanal ist da, er zaehlt nur nicht mit.
        //
        // Sichtbar wird der Unterschied bei `tuple(c)` und `list(c)` - die
        // geben drei Werte -, **nicht** bei `c[:]`, das weiter vier gibt.
        // Ein Nachbau mit Laenge 4 faellt also nur auf, wenn jemand die
        // Tafel in Tupel verwandelt; genau das tut pygames Testsuite.
        PyTuple::new(py, t.iter().map(|c| PyColor::mit_laenge(*c, 3)))
    }

    /// Ein einzelner Tafeleintrag.
    /// Der Index kommt als `i64` herein, nicht als `usize`.
    ///
    /// Gemessen gibt `get_palette_at(-1)` in pygame
    /// `IndexError: index out of bounds`; ein `usize`-Beiwert machte
    /// daraus `OverflowError: can't convert negative int to unsigned` -
    /// andere Ausnahme, anderer Text, und der Text erzaehlt von PyO3
    /// statt von pygame.
    #[pyo3(signature = (index, /))]
    fn get_palette_at(&self, index: i64) -> PyResult<PyColor> {
        let s = self.leihen()?;
        let Some(t) = s.get_palette() else {
            return Err(crate::umwandlung::PygameError::new_err(
                "Surface has no palette to get",
            ));
        };
        t.get(usize::try_from(index).unwrap_or(usize::MAX))
            // Auch hier Laenge 3 - siehe `get_palette`.
            .map(|c| PyColor::mit_laenge(*c, 3))
            .ok_or_else(|| pyo3::exceptions::PyIndexError::new_err("index out of bounds"))
    }

    /// Setzt die Farbtafel - ganz oder von vorn so weit, wie die Liste reicht.
    ///
    /// Gemessen: `set_palette([(9,9,9)])` aendert **nur** den ersten
    /// Eintrag, die Tafel bleibt 256 lang. Ohne Palette gibt pygame
    /// `pygame.error: Surface colors are not indexed`.
    #[pyo3(signature = (tafel, /))]
    fn set_palette(&self, tafel: &Bound<'_, PyAny>) -> PyResult<()> {
        let mut s = self.leihen_mut()?;
        let Some(alt) = s.get_palette() else {
            return Err(crate::umwandlung::PygameError::new_err(
                // **Mit Zeilenumbruch am Ende** - gemessen hat
                // pygames Satz hier einen, und ein Programm, das den
                // Text vergleicht, sieht ihn.
                "Surface colors are not indexed\n",
            ));
        };
        let mut neu: Vec<Color> = alt.to_vec();
        for (i, f) in tafel.try_iter()?.enumerate() {
            if i >= neu.len() {
                break;
            }
            // **Eigener Wortlaut fuer die Farbtafel.** Gemessen gibt
            // `set_palette([(1, 2)])` in pygame
            // `ValueError: takes a sequence of integers of RGB`, waehrend
            // `farbe_aus` den allgemeinen Satz ueber Farbfolgen meldet.
            // Derselbe Vorgang, zwei Saetze - wie schon bei `rect.c`
            // gegen `mask.c`.
            let c = farbe_aus(&f?).map_err(|_| {
                pyo3::exceptions::PyValueError::new_err(
                    "takes a sequence of integers of RGB",
                )
            })?;
            // **Ein Alpha unter 255 wird abgewiesen, nicht stillschweigend
            // hochgesetzt.** Gemessen:
            // `set_palette([Color(1, 2, 3, 254)])` gibt
            // `ValueError: takes an alpha value of 255`.
            //
            // Hier stand `.with_alpha(255)` - also "wir richten das schon".
            // Eine Farbtafel hat aber keinen Alphakanal, und wer eine
            // durchsichtige Farbe hineinlegt, meint etwas, das es nicht
            // gibt; das soll er erfahren.
            if c.a != 255 {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "takes an alpha value of 255",
                ));
            }
            neu[i] = c;
        }
        // **Ueber `palette_wechseln`**, damit die vorhandenen Pixel ihren
        // Index behalten - siehe dort.
        s.palette_wechseln(neu);
        Ok(())
    }

    /// Setzt einen einzelnen Tafeleintrag.
    /// Siehe [`get_palette_at`](Self::get_palette_at) zum `i64`.
    #[pyo3(signature = (index, c, /))]
    fn set_palette_at(&self, index: i64, c: &Bound<'_, PyAny>) -> PyResult<()> {
        let index = usize::try_from(index).unwrap_or(usize::MAX);
        let mut s = self.leihen_mut()?;
        let Some(alt) = s.get_palette() else {
            return Err(crate::umwandlung::PygameError::new_err(
                // **Mit Zeilenumbruch am Ende** - gemessen hat
                // pygames Satz hier einen, und ein Programm, das den
                // Text vergleicht, sieht ihn.
                "Surface colors are not indexed\n",
            ));
        };
        let mut neu: Vec<Color> = alt.to_vec();
        if index >= neu.len() {
            return Err(pyo3::exceptions::PyIndexError::new_err(
                "index out of bounds",
            ));
        }
        neu[index] = farbe_aus(c)?.with_alpha(255);
        s.set_palette(Some(neu));
        Ok(())
    }

    /// Eine Farbe in die Pixeldarstellung dieser Flaeche.
    ///
    /// Ohne Alphakanal faellt das Alpha weg - gemessen:
    /// `Surface((4,4)).map_rgb((10,20,30,40))` ist `0x0a141e`, nicht
    /// `0xff0a141e`. Bei einer schmaleren Tiefe kommen die Bits **dieses**
    /// Formats heraus: `Surface((4,4), depth=16).map_rgb((255,0,0))` ist
    /// `0xf800`.
    #[pyo3(signature = (c, /))]
    fn map_rgb(&self, c: &Bound<'_, PyAny>) -> PyResult<i32> {
        let s = self.leihen()?;
        // **`map_rgb` probiert keine Farbnamen.** Gemessen:
        // `set_colorkey("x")` gibt `invalid color name`, `map_rgb("x")`
        // dagegen den langen Satz ueber die Folge - pygames `map_rgb`
        // laesst den Namensweg weg.
        let f = farbe_aus(c).map_err(|_| {
            pyo3::exceptions::PyValueError::new_err(
                crate::umwandlung::FARBE_KAPUTT,
            )
        })?;
        let v = s.map_rgb_gepackt(f);
        Ok(gepackt(v, self.mit_alpha()? && s.get_bitsize() == 32))
    }

    /// Der Rueckweg von [`Self::map_rgb`].
    #[pyo3(signature = (v, /))]
    fn unmap_rgb(&self, v: &Bound<'_, PyAny>) -> PyResult<PyColor> {
        // pygames eigener Satz; PyO3 haengte sonst `argument 'v': ` vor
        // den seinen.
        let v: i64 = v.extract().map_err(|_| {
            pyo3::exceptions::PyTypeError::new_err("unmap_rgb expects 1 number argument")
        })?;
        let s = self.leihen()?;
        let c = s.unmap_rgb_gepackt(v as u32);
        Ok(PyColor::neu(if self.mit_alpha()? {
            c
        } else {
            c.with_alpha(255)
        }))
    }

    /// Der Pixelwert, wie er im **gemeldeten** Format aussieht.
    ///
    /// Bei 32 Bit ist das der Rohwert aus dem Speicher - und zwar genau
    /// der, denn das ungenutzte vierte Byte einer Flaeche ohne Alphakanal
    /// ist echter Zustand (gemessen: `set_at((0,0), (9,8,7,6))` gefolgt von
    /// `get_at_mapped` ergibt `0x00090807`, die 6 ist weg). Bei jeder
    /// anderen Tiefe muss der Wert erst in dessen Bits gepackt werden:
    /// `Surface((4,4), depth=16)` mit Rot gefuellt meldet `0xf800`, eine
    /// Palettenflaeche den **Index**.
    #[pyo3(signature = (pos, /))]
    fn get_at_mapped(&self, pos: &Bound<'_, PyAny>) -> PyResult<i32> {
        // Ueber `stelle_aus`, damit ein `Vector2` durchkommt und ein
        // untaugliches Argument denselben Satz gibt wie bei `get_at`.
        let pos = stelle_aus(pos)?;
        let s = self.leihen()?;
        if s.get_palette().is_none() && s.get_bitsize() == 32 {
            return Ok(gepackt(s.get_at_mapped(pos), self.mit_alpha()?));
        }
        Ok(gepackt(s.gepackt_an(pos), false))
    }

    /// `get_rect(center=(x,y))` und Geschwister - pygame setzt hier jedes
    /// der virtuellen Attribute per Schluesselwort.
    #[pyo3(signature = (*args, **kwargs))]
    fn get_rect(
        &self,
        args: &Bound<'_, pyo3::types::PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<PyRect> {
        // **Kein Stellungsargument.** pygame sagt hier
        // `get_rect only accepts keyword arguments`; PyO3 sagte
        // `Surface.get_rect() takes 0 positional arguments but 1
        // was given`. Dafuer nimmt die Methode `*args` und
        // zaehlt selbst.
        if !args.is_empty() {
            return Err(pyo3::exceptions::PyTypeError::new_err(
                "get_rect only accepts keyword arguments",
            ));
        }
        let mut r = PyRect::neu(self.leihen()?.get_rect());
        r.aus_kwargs(kwargs)?;
        Ok(r)
    }

    /// Dasselbe mit Komma - `pygame.Surface.get_frect`.
    ///
    /// Nimmt dieselben Ankerwoerter wie `get_rect`, also
    /// `get_frect(center=(5.5, 5.5))`.
    #[pyo3(signature = (*args, **kwargs))]
    fn get_frect(
        &self,
        args: &Bound<'_, pyo3::types::PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<PyFRect> {
        // **Kein Stellungsargument.** pygame sagt hier
        // `get_frect only accepts keyword arguments`; PyO3 sagte
        // `Surface.get_frect() takes 0 positional arguments but 1
        // was given`. Dafuer nimmt die Methode `*args` und
        // zaehlt selbst.
        if !args.is_empty() {
            return Err(pyo3::exceptions::PyTypeError::new_err(
                "get_frect only accepts keyword arguments",
            ));
        }
        let mut r = PyFRect::from(self.leihen()?.get_frect());
        r.aus_kwargs(kwargs)?;
        Ok(r)
    }

    /// `get_buffer()` - der ganze Pixelspeicher als [`PyBufferProxy`].
    ///
    /// **Nimmt kein Argument**; gemessen gibt `s.get_buffer("2")`
    /// `TypeError: Surface.get_buffer() takes no arguments (1 given)`. Wer
    /// eine Sicht will, nimmt [`get_view`](Self::get_view).
    fn get_buffer(slf: &Bound<'_, PySurface>) -> PyResult<Py<PyAny>> {
        let py = slf.py();
        let zeiger = Py::new(
            py,
            crate::pufferzeiger::PyBufferProxy::fuer_flaeche(
                slf.clone().into_any().unbind(),
                None,
            ),
        )?;
        // **Ein offener Puffer sperrt die Flaeche** - gemessen:
        //
        // ```python
        // v = s.get_buffer(); s.get_locked()   # True
        // v = None; gc.collect(); s.get_locked()   # False
        // ```
        //
        // Hier gibt es das Python-Objekt im Gegensatz zu `PixelArray`:
        // `get_buffer` ist eine gewoehnliche Methode, also wird der Zeiger
        // selbst gebaut und kann sich eintragen. Das Loesen besorgt der
        // weiche Verweis, siehe [`Sperre`].
        // Das Eintragen selbst besorgt `wecken` - dieselbe Stelle, die es
        // auch fuer `get_view(...)` tut, nur eben **jetzt**.
        let alsany: Py<PyAny> = zeiger.clone_ref(py).into_any();
        zeiger.bind(py).borrow().wecken(alsany.bind(py))?;
        Ok(zeiger.into_any())
    }

    /// `get_view(kind='2')` - eine Sicht auf die Pixel.
    ///
    /// Elf Buchstaben, drei Fehlerarten, und die Bedingungen haengen am
    /// Pixelformat. Alles hier ist an echtem pygame abgelesen, Zeile fuer
    /// Zeile (Flaeche 3x2):
    ///
    /// ```text
    /// Tiefe   '0'      '1'      '2'   '3'   'r','g','b'   'a','A'
    ///   8     Fehler   Fehler    6    Fehler  Fehler       Fehler
    ///  16     Fehler   Fehler   12    Fehler  Fehler       Fehler
    ///  24     Fehler   Fehler   18    18        6          Fehler
    ///  32       24       24     24    18        6          Fehler
    ///  32+SRCALPHA                                           6
    /// ```
    ///
    /// Und die Saetze, jeder mit eigenem Wortlaut:
    ///
    /// ```text
    /// '0' auf 3x2 mit 8 Bit   ValueError: Surface data is not contiguous
    /// '0' auf einer Teilflaeche  dasselbe
    /// '3' auf 8 Bit           ValueError: unsupported bit depth 8 for 3D reference array
    /// 'r' auf 8 Bit           ValueError: unsupported colormasks for red reference array
    /// 'a' ohne Alphamaske     ValueError: unsupported colormasks for alpha reference array
    /// 'RGB'                   TypeError:  expected a length 1 string for argument 1
    /// 'x'                     TypeError:  unrecognized view kind 'x' for argument 1
    /// 1                       TypeError:  expected a length one string for argument 1: got 'int'
    /// ```
    ///
    /// Drei Dinge sind nicht zu erraten:
    ///
    /// * **`'0'` und `'1'` verlangen zusammenhaengenden Speicher**, also
    ///   `pitch == breite * bytesize`. Bei 3x2 mit 8 Bit ist `pitch` auf
    ///   vier Bytes aufgerundet und die Sicht darum unmoeglich; bei 8x7 mit
    ///   8 Bit geht sie. Es haengt also an der **Breite**, nicht an der
    ///   Tiefe - und eine Teilflaeche scheitert immer.
    /// * **Ein zu langes Wort und ein unbekannter Buchstabe geben
    ///   verschiedene Saetze**, und der eine sagt "length 1", der andere
    ///   "length one". Das ist pygames Wortlaut, kein Tippfehler hier.
    /// * **`'a'` fragt die Alphamaske, nicht die Tiefe.** Eine Flaeche mit
    ///   32 Bit ohne `SRCALPHA` hat keine, und dann ist es ein Fehler.
    #[pyo3(signature = (kind=None, /))]
    fn get_view(
        slf: &Bound<'_, PySurface>,
        kind: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<crate::pufferzeiger::PyBufferProxy> {
        let art: char = match kind {
            None => '2',
            Some(o) => {
                // **Bytes gehen auch.** Gemessen nimmt pygame beides:
                // `get_view("2")` und `get_view(b"2")`. Der Test sagt es
                // sogar in einem Kommentar - "Both unicode and bytes
                // strings are allowed for kind" -, und hier stand nur der
                // eine der beiden Wege.
                let t = match o.extract::<String>() {
                    Ok(t) => t,
                    Err(_) => match o.extract::<Vec<u8>>() {
                        Ok(b) if o.is_instance_of::<pyo3::types::PyBytes>() => {
                            String::from_utf8_lossy(&b).into_owned()
                        }
                        _ => {
                            let name = o.get_type().name()?.to_string();
                            return Err(pyo3::exceptions::PyTypeError::new_err(format!(
                                "expected a length one string for argument 1: got '{name}'"
                            )));
                        }
                    },
                };
                let mut z = t.chars();
                match (z.next(), z.next()) {
                    (Some(c), None) => c,
                    _ => {
                        return Err(pyo3::exceptions::PyTypeError::new_err(
                            "expected a length 1 string for argument 1",
                        ))
                    }
                }
            }
        };

        let (tiefe, masken, zusammenhaengend) = {
            let k = slf.borrow();
            let s = k.leihen()?;
            let f = s.get_format();
            let (w, _) = s.get_size();
            (
                f.bitsize,
                f.masks,
                // Zusammenhaengend heisst: keine Auffuellung am Zeilenende
                // **und** keine Teilflaeche. Das Zweite ist nicht aus dem
                // Ersten ableitbar - eine Teilflaeche, die die ganze
                // Breite einnimmt, hat denselben `pitch` und trotzdem
                // fremde Zeilen davor und danach.
                s.get_pitch() == w * f.bytesize() && s.get_parent().is_none(),
            )
        };
        // **Eine Kanalmaske muss genau ein Byte sein.**
        //
        // Hier stand nur die Frage, ob sie ungleich null ist - und das
        // laesst 16 Bit durch: RGB565 hat eine rote Maske von 0xF800, die
        // ist nicht null und trotzdem kein Byte. Eine Sicht darauf gaebe
        // ein Byte, in dem Rot und ein Stueck Gruen stehen.
        //
        // Gemessen an `surfarray_test._test_pixels_rgb`, das genau drei
        // Tiefen als Fehler verlangt - 8 Bit, 16 Bit und 16 Bit mit
        // Alphakanal -, und an der Tabelle im Kopf dieser Methode, die es
        // seit je sagt: bei 8 und 16 Bit steht dort "Fehler".
        let farbig = |m: u32, name: &str| -> PyResult<()> {
            if m == 0 || m.count_ones() != 8 || !m.trailing_zeros().is_multiple_of(8) {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "unsupported colormasks for {name} reference array"
                )));
            }
            Ok(())
        };

        match art {
            '0' | '1' => {
                if !zusammenhaengend {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "Surface data is not contiguous",
                    ));
                }
            }
            '2' => {}
            '3' => {
                if tiefe != 24 && tiefe != 32 {
                    return Err(pyo3::exceptions::PyValueError::new_err(format!(
                        "unsupported bit depth {tiefe} for 3D reference array"
                    )));
                }
            }
            // **Gross und klein**, bei allen vier. Bei `'a'`/`'A'` war das
            // schon so, bei den Farben fehlte es - und `test_get_view`
            // fragt `'R'`. Gemessen gibt `get_view('R')` auf einer 8x6 die
            // Laenge 48, also `breite * hoehe` wie bei `'r'`.
            'r' | 'R' => farbig(masken.0, "red")?,
            'g' | 'G' => farbig(masken.1, "green")?,
            'b' | 'B' => farbig(masken.2, "blue")?,
            'a' | 'A' => farbig(masken.3, "alpha")?,
            c => {
                return Err(pyo3::exceptions::PyTypeError::new_err(format!(
                    "unrecognized view kind '{c}' for argument 1"
                )))
            }
        }
        Ok(crate::pufferzeiger::PyBufferProxy::fuer_flaeche(
            slf.clone().into_any().unbind(),
            Some(art.to_string()),
        ))
    }

    /// `s.width` - dasselbe wie `get_width()`, nur als Feld.
    ///
    /// Die drei Felder sind **nur lesbar**: gemessen gibt
    /// `s.width = 3` in pygame
    /// `AttributeError: attribute 'width' of 'pygame.surface.Surface'
    /// objects is not writable`. Ein `#[setter]` waere hier also falsch -
    /// und PyO3 erzeugt ohne ihn von sich aus dieselbe Ausnahmeart.
    /// Die Rust-Namen tragen ein `feld_` davor, die Python-Namen nicht.
    ///
    /// PyO3 baut aus `#[getter] fn width` die Innenfunktion
    /// `__pymethod_get_width__` - und die gibt es schon, naemlich fuer die
    /// Methode `get_width`. Der Bau bricht dann mit
    /// `duplicate definitions with name __pymethod_get_width__` ab. Mit
    /// `#[pyo3(name = ...)]` heisst das Feld nach aussen weiter `width`,
    /// und die Kollision ist weg.
    #[getter]
    #[pyo3(name = "width")]
    fn feld_width(&self) -> PyResult<u32> {
        Ok(self.leihen()?.get_size().0)
    }
    #[getter]
    #[pyo3(name = "height")]
    fn feld_height(&self) -> PyResult<u32> {
        Ok(self.leihen()?.get_size().1)
    }
    #[getter]
    #[pyo3(name = "size")]
    fn feld_size(&self) -> PyResult<(u32, u32)> {
        Ok(self.leihen()?.get_size())
    }

    /// `set_masks` und `set_shifts` gibt es - **damit sie ablehnen**.
    ///
    /// Gemessen: beide melden
    /// `TypeError: The surface masks are read-only in SDL2` bzw.
    /// `... shifts are read-only in SDL2`. Sie einfach weglassen waere die
    /// falsche Auskunft: ein Programm bekaeme dann einen
    /// `AttributeError`, und `surface_test` prueft ausdruecklich den
    /// `TypeError`. Eine Funktion, die nur absagt, ist hier also die
    /// **genauere** Nachbildung als keine Funktion.
    #[pyo3(signature = (*_args, **_kwargs))]
    fn set_masks(&self, _args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
        Err(pyo3::exceptions::PyTypeError::new_err(
            "The surface masks are read-only in SDL2",
        ))
    }
    #[pyo3(signature = (*_args, **_kwargs))]
    fn set_shifts(&self, _args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
        Err(pyo3::exceptions::PyTypeError::new_err(
            "The surface shifts are read-only in SDL2",
        ))
    }

    /// Die Mischart der Flaeche - `1`, wenn gemischt wird, sonst `0`.
    fn get_blendmode(&self) -> PyResult<u32> {
        Ok(self.leihen()?.get_blendmode())
    }

    /// Viele Blits auf einmal, ohne Bereich und ohne Rueckgabe.
    ///
    /// Der schnelle Bruder von [`blits`](Self::blits): jeder Eintrag ist
    /// nur `(flaeche, ziel)`, die Mischart gilt fuer alle, und es kommt
    /// **immer** `None` zurueck.
    ///
    /// Gemessen: `fblits` nimmt **keine** Schluesselwortargumente -
    /// `s.fblits(blit_sequence=[...])` gibt
    /// `TypeError: Surface.fblits() takes no keyword arguments`. Deshalb
    /// steht hier kein `doreturn`, obwohl `blits` eines hat.
    ///
    /// Die Fehlerarten sind ueberraschend, deshalb hier gesammelt und
    /// jede einzeln gemessen:
    ///
    /// ```text
    /// fblits()                 ValueError: Incorrect number of parameters passed: need at least one, 2 at max
    /// fblits(None) / (1) / ("str")
    ///                          ValueError: blit_sequence should be iterator of (Surface, dest)
    /// fblits([-1]) / ([surf])  ValueError: Blit_sequence item should be a tuple of (Surface, dest)
    /// fblits(liste, 12.245)    TypeError:  The special_flags parameter must be an int
    /// fblits(liste, -999)      TypeError:  Blit failed (probably the flag used does not exist)
    /// fblits([(None, None)])   TypeError:  Source objects must be a Surface
    /// fblits([(surf, None)])   TypeError:  invalid destination position for blit
    /// ```
    ///
    /// Das fehlende Argument gibt also einen **`ValueError`**, nicht den
    /// `TypeError`, den PyO3 von sich aus erzeugt haette - deshalb steht
    /// `sequence` hier als `Option` in der Signatur und wird von Hand
    /// geprueft.
    ///
    /// **Und die Woerter sind hier nur Namen, keine Schluessel.** Der
    /// Vermerk oben sagte das schon, umgesetzt war es nicht: `fblits`
    /// nahm `special_flags=0` stillschweigend an, wo pygame
    /// `TypeError: Surface.fblits() takes no keyword arguments` gibt. Ein
    /// `/` in der Signatur gaebe PyO3s eigenen Satz ("got some
    /// positional-only arguments passed as keyword arguments"), also wird
    /// `**kwargs` genommen und der gemessene Satz selbst geworfen.
    //
    // Das `/` ist der Kniff: ohne es faengt der **benannte** Beiwert
    // `special_flags` das Wort weg, und `**kwargs` bleibt leer -
    // `fblits(liste, special_flags=0)` ginge durch. Mit `/` sind beide
    // nur-positionell, jedes Wort landet in `kwargs`, und der
    // gemessene Satz kommt heraus.
    #[pyo3(signature = (sequence=Weitergabe::Fehlt, special_flags=Weitergabe::Fehlt, /, **kwargs))]
    fn fblits(
        &self,
        sequence: Weitergabe<'_>,
        special_flags: Weitergabe<'_>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        if kwargs.map(|k| !k.is_empty()).unwrap_or(false) {
            return Err(pyo3::exceptions::PyTypeError::new_err(
                "Surface.fblits() takes no keyword arguments",
            ));
        }
        let Some(sequence) = sequence.als_option() else {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Incorrect number of parameters passed: need at least one, 2 at max",
            ));
        };
        let flags = flagge_aus(special_flags.als_option())?;
        let lauf = sequence.try_iter().map_err(|_| folge_fehler())?;
        for eintrag in lauf {
            let e = eintrag?;
            let teile = eintrag_teile(&e).ok_or_else(eintrag_fehler)?;
            if teile.len() != 2 {
                return Err(eintrag_fehler());
            }
            let quelle: PyRef<PySurface> = teile[0].extract().map_err(|_| {
                pyo3::exceptions::PyTypeError::new_err("Source objects must be a Surface")
            })?;
            self.blit_roh(&quelle, &teile[1], None, flags, false)?;
        }
        Ok(())
    }

    // ----- Einzelne Pixel -------------------------------------------------

    /// Ausserhalb der Flaeche ist es ein **`IndexError`**, keine Farbe.
    ///
    /// Gemessen: `Surface((8,8)).get_at((99, 99))` gibt
    /// `IndexError: pixel index out of range`. Der Kern liefert dort
    /// stillschweigend durchsichtiges Schwarz - fuer Rust richtig, aber
    /// ein Programm, das sich auf die Ausnahme verlaesst, liefe hier ins
    /// Leere. Dieselbe Ueberlegung wie bei `Mask.get_at`.
    #[pyo3(signature = (pos, /))]
    fn get_at(&self, pos: &Bound<'_, PyAny>) -> PyResult<PyColor> {
        let p = stelle_aus(pos)?;
        let k = self.leihen()?;
        let (w, h) = k.get_size();
        if p.0 < 0 || p.1 < 0 || p.0 >= w as i32 || p.1 >= h as i32 {
            return Err(pyo3::exceptions::PyIndexError::new_err(
                "pixel index out of range",
            ));
        }
        Ok(PyColor::neu(k.get_at(p)))
    }
    #[pyo3(signature = (pos, c, /))]
    fn set_at(&self, pos: &Bound<'_, PyAny>, c: &Bound<'_, PyAny>) -> PyResult<()> {
        let p = stelle_aus(pos)?;
        let f = self.farbe_oder_wert(c)?;
        self.leihen_mut()?.set_at(p, f);
        Ok(())
    }

    // ----- Alpha, Schluesselfarbe, Clip ------------------------------------

    /// Das Flaechen-Alpha. Siehe das Feld `alpha_aus`, warum hier eine
    /// Fallunterscheidung steht und nicht bloss ein Durchreichen.
    fn get_alpha(&self) -> PyResult<Option<u8>> {
        let s = self.leihen()?;
        if let Some(a) = s.get_alpha() {
            return Ok(Some(a));
        }
        if s.get_flags() & SRCALPHA != 0 && !self.alpha_aus.load(Reihe::Relaxed) {
            return Ok(Some(255));
        }
        Ok(None)
    }
    #[pyo3(signature = (a=None, flags=0, /))]
    fn set_alpha(&self, a: Option<&Bound<'_, PyAny>>, flags: u32) -> PyResult<()> {
        // `RLEACCEL` ist die **Erlaubnis**, nicht die Tat - gemessen:
        // `set_alpha(128, RLEACCEL)` macht aus den Flaggen 73728
        // (`SRCALPHA | RLEACCELOK`), und `mustlock()` bleibt `False`.
        if flags & RLEACCEL != 0 {
            self.rle_erlaubt.store(true, Reihe::Relaxed);
        } else {
            // **Ohne die Flagge nimmt `set_alpha` die Erlaubnis weg - und
            // die Kodierung mit.**
            //
            // Gemessen in `surface_test.test_solarwolf_rle_usage_2`: nach
            // `set_colorkey(key, RLEACCEL)` und einem Blit stehen beide
            // Flaggen; `set_alpha(90)` danach loescht **beide**. Der
            // Kommentar im Test sagt es woertlich - "should strip all RLE
            // flags".
            //
            // `set_colorkey` ohne die Flagge tut dasselbe, und das stand
            // hier schon (siehe unten). Bei `set_alpha` fehlte es - eine
            // halbe Regel, und die Haelfte war nicht zu erraten, sondern
            // stand im Nachbartest.
            self.rle_erlaubt.store(false, Reihe::Relaxed);
            self.rle_aktiv.store(false, Reihe::Relaxed);
        }
        let wert = match a {
            None => None,
            Some(x) if x.is_none() => None,
            // pygames eigener Satz - `set_alpha("x")` gibt
            // `TypeError: invalid alpha argument`, nicht den des
            // Argumentlesers.
            Some(x) => Some(x.extract::<u8>().map_err(|_| {
                pyo3::exceptions::PyTypeError::new_err("invalid alpha argument")
            })?),
        };
        self.alpha_aus.store(wert.is_none(), Reihe::Relaxed);
        self.leihen_mut()?.set_alpha(wert);
        Ok(())
    }
    /// `get_colorkey()` - ein **Tupel** `(r, g, b, a)`, keine `Color`.
    ///
    /// `surf_get_colorkey` holt den abgebildeten Pixelwert
    /// (`SDL_GetColorKey`) und rechnet ihn zurueck: bei 16 Bit also
    /// gerundet, auf einer Palettenflaeche der Tafeleintrag, und `a` ist
    /// 255, solange das Format keinen Alphakanal hat. Hier stand bis Runde
    /// 19 die gespeicherte `Color` - `==` mit einem Tupel ist dabei wahr,
    /// darum hat es keine Probe gesehen; `type(k)` und `k + (1,)` sehen es.
    fn get_colorkey<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyTuple>>> {
        let s = self.leihen()?;
        let Some(k) = s.get_colorkey() else {
            return Ok(None);
        };
        let c = s.unmap_rgb_gepackt(s.map_rgb_gepackt(k));
        let a = if s.has_alpha() { c.a } else { 255 };
        Ok(Some(PyTuple::new(py, [c.r, c.g, c.b, a])?))
    }
    /// `set_colorkey(farbe[, flags])`.
    ///
    /// Die Flagge `RLEACCEL` wird hier zur Erlaubnis, und eine
    /// Schluesselfarbe **ohne** sie nimmt die Erlaubnis wieder weg -
    /// gemessen:
    ///
    /// ```text
    /// s.set_colorkey(c, RLEACCEL)  -> flags 12288  (RLEACCELOK|SRCCOLORKEY)
    /// s.set_colorkey(c)            -> flags  4096  (nur SRCCOLORKEY)
    /// s.set_colorkey(None)         -> flags  8192  (RLEACCELOK bleibt!)
    /// ```
    ///
    /// Die letzte Zeile ist die ueberraschende: `None` loescht die
    /// Schluesselfarbe, nicht die Erlaubnis.
    #[pyo3(signature = (c=None, flags=0, /))]
    fn set_colorkey(&self, c: Option<&Bound<'_, PyAny>>, flags: u32) -> PyResult<()> {
        // `pg_MappedColorFromObj`: eine ganze Zahl ist ein **Pixelwert
        // dieser Flaeche** - auf einer 8-Bit-Flaeche also ein Tafelindex
        // (`set_colorkey(5)` meint dort `(0, 36, 85)`, nicht Schwarz).
        let wert = match c {
            None => None,
            Some(x) if x.is_none() => None,
            Some(x) => {
                // `SDL_SetColorKey` prueft auf einer Palettenflaeche den
                // Index gegen die Tafellaenge: `set_colorkey(0x123456)`
                // auf acht Bit gibt `pygame.error: Parameter 'key' is
                // invalid`, gemessen.
                if x.is_instance_of::<pyo3::types::PyInt>() {
                    let k = self.leihen()?;
                    if k.get_bytesize() == 1 {
                        let n = k.get_palette().map_or(256, |t| t.len()) as i64;
                        let v = x.extract::<i64>().unwrap_or(i64::MAX) & 0xFFFF_FFFF;
                        if v >= n {
                            return Err(crate::umwandlung::PygameError::new_err(
                                "Parameter 'key' is invalid",
                            ));
                        }
                    }
                }
                Some(self.farbe_oder_wert(x)?)
            }
        };
        if wert.is_some() {
            self.rle_erlaubt.store(flags & RLEACCEL != 0, Reihe::Relaxed);
            self.rle_aktiv.store(false, Reihe::Relaxed);
        }
        self.leihen_mut()?.set_colorkey(wert);
        Ok(())
    }
    fn get_clip(&self) -> PyResult<PyRect> {
        Ok(PyRect::neu(self.leihen()?.get_clip()))
    }
    #[pyo3(signature = (r=None, /))]
    fn set_clip(&self, r: Option<&Bound<'_, PyAny>>) -> PyResult<()> {
        // **`ValueError`, nicht `TypeError`** - gemessen gibt
        // `Surface.set_clip("x")` in pygame
        // `ValueError: invalid rectstyle object`. `rect_aus` gibt den
        // Satz aus `rect.c`, und der stimmt an anderen Stellen; hier
        // nicht, und nicht einmal die Art. Ein `except ValueError` sah
        // diesen Fehler also gar nicht.
        let geclippt = rect_aus_opt(r)
            .map_err(|_| pyo3::exceptions::PyValueError::new_err("invalid rectstyle object"))?;
        self.leihen_mut()?.set_clip(geclippt);
        Ok(())
    }

    // ----- Fuellen und Blitten ---------------------------------------------

    #[pyo3(signature = (color, rect=None, special_flags=0))]
    fn fill(
        &self,
        color: &Bound<'_, PyAny>,
        rect: Option<&Bound<'_, PyAny>>,
        special_flags: u32,
    ) -> PyResult<PyRect> {
        // Gemessen: `fill` mit BLEND_PREMULTIPLIED oder BLEND_ALPHA_SDL2
        // gibt in pygame `pygame.error: invalid blend flag for this
        // operation`. Der Kern liesse beide durch.
        if special_flags == 17 || special_flags == 18 {
            return Err(crate::umwandlung::PygameError::new_err(
                "invalid blend flag for this operation",
            ));
        }
        let c = self.farbe_oder_wert(color)?;
        // Gemessen: `fill(c, "x")` und `fill(c, (1,2))` geben
        // `ValueError: invalid rectstyle object` - **object**, waehrend
        // `subsurface` bei derselben Lage **argument** sagt. Derselbe
        // Vorgang, zwei Saetze; durchreichen waere falsch.
        let r = match rect {
            None => None,
            Some(x) if x.is_none() => None,
            Some(x) => Some(crate::rechteck::rect_aus_objekt(x).ok_or_else(|| {
                pyo3::exceptions::PyValueError::new_err("invalid rectstyle object")
            })?),
        };
        Ok(PyRect::neu(
            self.leihen_mut()?.fill_ex(c, r, misch_art(special_flags)),
        ))
    }

    /// `dest.blit(quelle, ziel[, bereich[, special_flags]])`.
    ///
    /// Blittet ein Programm eine Flaeche auf sich selbst, wuerde die
    /// doppelte Ausleihe stuerzen. Dann wird die Quelle vorher kopiert -
    /// das ist genau das, was das Programm meint.
    /// `dest` darf **fehlen** - dann ist es `(0, 0)`.
    ///
    /// Gemessen: `ziel.blit(quelle)` gibt `Rect(0, 0, w, h)`. Das ist neu
    /// in pygame-ce und `surface_test.test_blit_default_dest` prueft es.
    #[pyo3(signature = (source, dest=None, area=None, special_flags=Weitergabe::Fehlt))]
    fn blit(
        &self,
        source: &PySurface,
        dest: Option<&Bound<'_, PyAny>>,
        area: Option<&Bound<'_, PyAny>>,
        special_flags: Weitergabe<'_>,
    ) -> PyResult<PyRect> {
        let flags = flagge_aus(special_flags.als_option())?;
        match dest {
            None => self.blit_an(source, (0, 0), area, flags, true),
            Some(d) if d.is_none() => self.blit_an(source, (0, 0), area, flags, true),
            Some(d) => self.blit_roh(source, d, area, flags, true),
        }
    }

    /// Mehrere Blits auf einmal - `[(flaeche, ziel[, bereich[, flags]]), …]`.
    ///
    /// Mit `doreturn=False` gibt pygame `None` statt der Liste zurueck -
    /// und `doreturn` ist **kein** `bool` im Sinne von PyO3:
    /// `blits(liste, doreturn=0)` steht so in pygames eigener Testdatei,
    /// und eine `bool`-Signatur gab dort
    /// `TypeError: argument 'doreturn': 'int' object cannot be converted
    /// to 'PyBool'`. Geprueft wird der **Wahrheitswert**.
    ///
    /// Die Fehlerarten sind dieselben wie bei [`fblits`](Self::fblits) -
    /// bis auf zwei Meldungen, die abweichen (gemessen):
    ///
    /// ```text
    /// blits(None) / ([surf])   ValueError: blit_sequence should be iterator of (Surface, dest)
    /// blits([(None, None)])    TypeError:  Source objects must be a surface   (klein geschrieben!)
    /// ```
    ///
    /// `blits` sagt bei einem zu kurzen Eintrag also **nicht**
    /// "Blit_sequence item should be ...", sondern noch einmal denselben
    /// Satz wie fuer die ganze Folge.
    #[pyo3(signature = (sequence, doreturn=Weitergabe::Fehlt))]
    fn blits(
        &self,
        py: Python<'_>,
        sequence: &Bound<'_, PyAny>,
        doreturn: Weitergabe<'_>,
    ) -> PyResult<Py<PyAny>> {
        // `doreturn` ist eine **Zahl**, kein Wahrheitswert: pygame liest
        // sie mit `p` (int) ein, weshalb `doreturn=None`
        // `TypeError: 'NoneType' object cannot be interpreted as an
        // integer` gibt und `doreturn=0` durchgeht. Gemessen.
        let doreturn = match doreturn.als_option() {
            None => true,
            Some(d) => d.extract::<i64>()? != 0,
        };
        let mut aus = Vec::new();
        let lauf = sequence.try_iter().map_err(|_| folge_fehler())?;
        for eintrag in lauf {
            let e = eintrag?;
            let teile = eintrag_teile(&e).ok_or_else(folge_fehler)?;
            if teile.len() < 2 {
                return Err(folge_fehler());
            }
            let quelle: PyRef<PySurface> = teile[0].extract().map_err(|_| {
                pyo3::exceptions::PyTypeError::new_err("Source objects must be a surface")
            })?;
            let bereich = teile.get(2);
            let flags = flagge_aus(teile.get(3))?;
            aus.push(self.blit_roh(&quelle, &teile[1], bereich, flags, true)?);
        }
        if !doreturn {
            return Ok(py.None());
        }
        Ok(PyList::new(py, aus)?.into_any().unbind())
    }

    // ----- Kopien und Ausschnitte -------------------------------------------

    /// Eine eigenstaendige Kopie - **im Typ des Aufrufers**.
    ///
    /// `copy()` ruft **nicht** das `__copy__` der Instanz. Gemessen:
    ///
    /// ```python
    /// class S(pygame.Surface):
    ///     def __copy__(self): return "MEINS"
    /// S((4,4)).copy()      # -> <Surface(4x4x32)>, nicht "MEINS"
    /// copy.copy(S((4,4)))  # -> "MEINS"
    /// ```
    ///
    /// Bei `Mask` war es andersherum gebaut und zwei Tests sind daran
    /// gescheitert; hier ist die Richtung nachgemessen und stimmt. Der
    /// **Typ** wird dabei erhalten: `S((4,4)).copy()` einer Unterklasse
    /// ohne eigenes `__copy__` ist ein `S`.
    fn copy<'py>(slf: &Bound<'py, PySurface>) -> PyResult<Bound<'py, PySurface>> {
        let k = slf.borrow().leihen()?.copy();
        PySurface::wie_typ(slf, k, None)
    }

    /// `copy.copy(s)` - dasselbe wie [`Self::copy`].
    fn __copy__<'py>(slf: &Bound<'py, PySurface>) -> PyResult<Bound<'py, PySurface>> {
        PySurface::copy(slf)
    }

    /// `copy.deepcopy(s)`.
    ///
    /// Eine Flaeche hat keine geschachtelten Python-Objekte, an denen eine
    /// tiefe Kopie etwas anderes tun koennte als eine flache. pygame hat
    /// die Methode trotzdem (gemessen: `hasattr(s, "__deepcopy__")` ist
    /// wahr), und ohne sie versucht `copy.deepcopy` es ueber `__reduce__`
    /// und scheitert mit `cannot pickle`.
    #[pyo3(signature = (memo=None))]
    fn __deepcopy__<'py>(
        slf: &Bound<'py, PySurface>,
        memo: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Bound<'py, PySurface>> {
        let _ = memo;
        PySurface::copy(slf)
    }

    /// `convert([vorlage])`.
    ///
    /// **Ohne Anzeige und ohne Vorlage ist das ein Fehler** - gemessen:
    ///
    /// ```text
    /// convert()          error: cannot convert without format when pygame.display is not initialized
    /// convert(Surface)   geht
    /// convert(32)        geht
    /// ```
    ///
    /// rustgame hat nur ein Rechenformat, die Umwandlung ist also
    /// weiterhin eine billige Kopie. Der **Einwand** gehoert trotzdem
    /// hierher: ein Programm, das `convert()` vor `set_mode()` ruft, hat
    /// einen Fehler, und pygame sagt es ihm.
    #[pyo3(signature = (vorlage=None, /))]
    fn convert<'py>(
        slf: &Bound<'py, PySurface>,
        vorlage: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Bound<'py, PySurface>> {
        let hat_vorlage = matches!(vorlage, Some(v) if !v.is_none());
        {
            let b = slf.borrow();
            // **Zwei Lagen, zwei Meldungen** - gemessen:
            //
            // ```text
            // display nie gestartet   error: cannot convert without format
            //                                when pygame.display is not initialized
            // gestartet, kein set_mode error: No convert format has been set,
            //                                try display.set_mode()
            // ```
            //
            // Der zweite Fall fehlte hier ganz: nach `pygame.init()` ohne
            // Fenster ging `convert()` einfach durch. Es gibt dann aber
            // kein Zielformat - die Flaeche wuerde in ein Format gewandelt,
            // das niemand festgelegt hat.
            if !hat_vorlage {
                if !rustgame_display::display::get_init() {
                    return Err(crate::umwandlung::PygameError::new_err(
                        "cannot convert without format when pygame.display is not initialized",
                    ));
                }
                if rustgame_display::display::get_surface().is_none() {
                    return Err(crate::umwandlung::PygameError::new_err(
                        "No convert format has been set, try display.set_mode()",
                    ));
                }
            }
            b.lebt()?;
        }
        // **Mit Argument wird in dessen Format gewandelt.**
        //
        // pygame nimmt dort eine Tiefe (`convert(8)`), eine andere
        // Flaeche (`convert(vorbild)`) oder Masken samt Tiefe. Ohne
        // Argument gilt das Anzeigeformat. Hier wurde das Argument
        // angenommen und **nicht ausgewertet**: `convert(8)` gab eine
        // 32-Bit-Flaeche zurueck, und `test_convert_palettize` hat den
        // Unterschied an der Farbe gesehen - `(255, 50, 0)` statt
        // `(255, 36, 0)`, denn nur die zweite steht in der Farbtafel.
        let k = match vorlage {
            Some(v) if !v.is_none() => {
                let (format, palette) = zielformat(v)?;
                slf.borrow().leihen()?.convert_zu(format, palette)
            }
            _ => slf.borrow().leihen()?.convert(),
        };
        let neu = PySurface::wie_typ(slf, k, None)?;
        // `convert` behaelt die RLE-**Erlaubnis** und verliert die
        // Kodierung - genau das prueft `test_solarwolf_rle_usage`
        // (danach `RLEACCELOK` gesetzt, `RLEACCEL` nicht).
        neu.borrow().rle_aktiv.store(false, Reihe::Relaxed);
        Ok(neu)
    }

    /// `convert_alpha([vorlage])`.
    ///
    /// Ohne Anzeige ein Fehler, und mit **anderem** Wortlaut als
    /// [`Self::convert`] (gemessen:
    /// `cannot convert without pygame.display initialized`) - auch dann,
    /// wenn eine Vorlage dasteht. Die Vorlage selbst ist seit pygame-ce
    /// 2.5 veraltet und loest eine `DeprecationWarning` aus.
    #[pyo3(signature = (vorlage=None, /))]
    fn convert_alpha<'py>(
        slf: &Bound<'py, PySurface>,
        vorlage: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Bound<'py, PySurface>> {
        let py = slf.py();
        // **Erst das Argument, dann der Zustand.** Gemessen gibt
        // `Surface((2,2)).convert_alpha('x')` ohne `set_mode`
        // `TypeError: argument 1 must be pygame.surface.Surface, not str`
        // - also den Argumentfehler, nicht die Zustandsmeldung. Hier
        // stand die Reihenfolge umgekehrt, und damit war der Fehler im
        // Argument unsichtbar, solange kein Fenster offen war.
        if let Some(v) = vorlage {
            if !v.is_none() && v.downcast::<PySurface>().is_err() {
                return Err(pyo3::exceptions::PyTypeError::new_err(format!(
                    "argument 1 must be pygame.surface.Surface, not {}",
                    v.get_type()
                        .name()
                        .map(|n| n.to_string())
                        .unwrap_or_else(|_| "object".to_string())
                )));
            }
        }
        // Dieselbe Zweiteilung wie bei `convert` - nur mit anderem Wortlaut
        // im ersten Fall, gemessen:
        //
        // ```text
        // display nie gestartet   error: cannot convert without
        //                                pygame.display initialized
        // gestartet, kein set_mode error: No convert format has been set,
        //                                try display.set_mode()
        // ```
        if !rustgame_display::display::get_init() {
            return Err(crate::umwandlung::PygameError::new_err(
                "cannot convert without pygame.display initialized",
            ));
        }
        if rustgame_display::display::get_surface().is_none() {
            return Err(crate::umwandlung::PygameError::new_err(
                "No convert format has been set, try display.set_mode()",
            ));
        }
        slf.borrow().lebt()?;
        if matches!(vorlage, Some(v) if !v.is_none()) {
            // **Und es muss eine Flaeche sein.** Hier wurde die Vorlage
            // nur auf `None` geprueft und sonst nie angesehen, also kam
            // `convert_alpha("x")` stillschweigend durch; pygame gibt
            // dort einen Argumentfehler.
            if let Some(v) = vorlage {
                let _: PyRef<'_, PySurface> = v.extract()?;
            }
            pyo3::PyErr::warn(
                py,
                &py.get_type::<pyo3::exceptions::PyDeprecationWarning>(),
                std::ffi::CString::new(
                    "public method convert_alpha() with a surface argument is deprecated",
                )?
                .as_c_str(),
                1,
            )?;
        }
        let k = slf.borrow().leihen()?.convert_alpha();
        let neu = PySurface::wie_typ(slf, k, None)?;
        neu.borrow().rle_aktiv.store(false, Reihe::Relaxed);
        Ok(neu)
    }

    /// Ein Ausschnitt, der sich den Speicher mit dieser Flaeche **teilt**.
    #[pyo3(signature = (erstes, zweites=None, drittes=None, viertes=None, /))]
    fn subsurface<'py>(
        slf: &Bound<'py, PySurface>,
        erstes: &Bound<'_, PyAny>,
        zweites: Option<&Bound<'_, PyAny>>,
        drittes: Option<&Bound<'_, PyAny>>,
        viertes: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Bound<'py, PySurface>> {
        let py = slf.py();
        // **Alle vier Schreibweisen**, wie pygame sie nimmt (gemessen):
        // `subsurface(Rect)`, `subsurface((x,y,w,h))`,
        // `subsurface((x,y),(w,h))` und `subsurface(x,y,w,h)`. Hier gab es
        // nur die erste und die letzte, und alles andere einen `TypeError`
        // mit einem selbstgeschriebenen Satz. pygame sagt
        // `ValueError: invalid rectstyle argument` - und zwar auch fuer
        // `subsurface(0,0,2)`, wo man einen TypeError erwarten wuerde.
        let teile: Vec<&Bound<'_, PyAny>> = std::iter::once(erstes)
            .chain([zweites, drittes, viertes].into_iter().flatten())
            .collect();
        let r = rechteck_aus_teilen(&teile).ok_or_else(rechteckform)?;
        let ganz = slf.borrow().leihen()?.get_rect();
        // Gemessen: ein Ausschnitt, der ueber den Rand hinausragt, ist ein
        // `ValueError` - der Kern beschneidet ihn stillschweigend, was
        // hier die falsche Auskunft waere.
        if !ganz.contains(r) {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "subsurface rectangle outside surface area",
            ));
        }
        let teil = slf.borrow().leihen()?.subsurface(r);
        let eltern = Some(slf.clone().unbind().clone_ref(py));
        // Eine Teilflaeche behaelt den Typ des Elternteils (gemessen), aber
        // **keine** RLE-Flagge.
        let neu = PySurface::wie_typ(slf, teil, eltern)?;
        neu.borrow().rle_erlaubt.store(false, Reihe::Relaxed);
        neu.borrow().rle_aktiv.store(false, Reihe::Relaxed);
        Ok(neu)
    }

    /// Die Flaeche, aus der diese geschnitten wurde - `None` bei einer
    /// eigenstaendigen.
    /// Auch diese beiden fragen zuerst, ob die Flaeche noch lebt.
    ///
    /// Gemessen: `s = display.set_mode(); display.quit(); s.get_parent()`
    /// gibt `pygame.error`. Die Anzeigeflaeche ist nach `display.quit()`
    /// hinueber, und **jede** Auskunft darueber ist es auch - nicht nur
    /// die ueber Pixel. Hier fehlte die Pruefung an genau diesen zwei
    /// Stellen, und `surface_test` fragt beide einzeln ab.
    fn get_parent(&self, py: Python<'_>) -> PyResult<Option<Py<PySurface>>> {
        self.lebt()?;
        Ok(self.eltern.as_ref().map(|e| e.clone_ref(py)))
    }

    /// Die urspruengliche Flaeche ganz oben in der Kette.
    fn get_abs_parent(slf: &Bound<'_, PySurface>) -> PyResult<Py<PySurface>> {
        let py = slf.py();
        slf.borrow().lebt()?;
        let mut jetzt = slf.clone().unbind();
        loop {
            let naechst = { jetzt.bind(py).borrow().eltern.as_ref().map(|e| e.clone_ref(py)) };
            match naechst {
                None => return Ok(jetzt),
                Some(e) => jetzt = e,
            }
        }
    }

    fn get_offset(&self) -> PyResult<(i32, i32)> {
        Ok(self.leihen()?.get_offset())
    }
    fn get_abs_offset(&self) -> PyResult<(i32, i32)> {
        Ok(self.leihen()?.get_abs_offset())
    }

    /// `scroll(dx, dy[, scroll_flag])`.
    ///
    /// Der dritte Parameter fehlte. Gemessen:
    ///
    /// ```text
    /// scroll(2, 2)                    schiebt, der Rest bleibt stehen
    /// scroll(2, 2, SCROLL_ERASE)      der frei werdende Rand wird schwarz
    /// scroll(4, 0, SCROLL_REPEAT)     der Inhalt laeuft rundherum
    /// scroll(1, 1, 99)                ValueError: Invalid scroll flag
    /// scroll(dx=1, dy=1, scroll_flag=0)   geht
    /// ```
    ///
    /// `SCROLL_ERASE` ist 4, `SCROLL_REPEAT` ist 1, und 0 ist das alte
    /// Verhalten.
    #[pyo3(signature = (dx=0, dy=0, scroll_flag=0))]
    fn scroll(&self, dx: i32, dy: i32, scroll_flag: i32) -> PyResult<()> {
        const SCROLL_REPEAT: i32 = 1;
        const SCROLL_ERASE: i32 = 4;
        if !matches!(scroll_flag, 0 | SCROLL_REPEAT | SCROLL_ERASE) {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Invalid scroll flag",
            ));
        }
        let mut k = self.leihen_mut()?;
        match scroll_flag {
            SCROLL_REPEAT => k.scroll_wiederholend(dx, dy),
            SCROLL_ERASE => k.scroll_loeschend(dx, dy),
            _ => k.scroll(dx, dy),
        }
        Ok(())
    }

    /// Das kleinste Rechteck, das alle Pixel mit Alpha **ab** `min_alpha`
    /// umschliesst.
    ///
    /// Die Schwelle geht **unveraendert** an den Kern. Hier stand einmal
    /// `min_alpha - 1`, mit der Begruendung "pygame prueft `>=`, der Kern
    /// `>`". Das stimmte, als es geschrieben wurde. Spaeter wurde der Kern
    /// auf `>=` umgestellt - und die Korrektur blieb stehen. Aus der
    /// Vorgabe `1` wurde damit `0`, und `0 >= 0` ist fuer **jeden** Pixel
    /// wahr: die Huelle meldete auf einer vollstaendig durchsichtigen
    /// Flaeche die ganze Flaeche statt eines leeren Rechtecks.
    ///
    /// Der Kern hatte einen Test dafuer, die Huelle nicht - gefunden hat
    /// es erst der Modulvergleich. Eine Umrechnung zwischen zwei Schichten
    /// muss auf **beiden** Seiten einen Test haben, sonst ueberlebt sie
    /// die Schicht, die sie ausgleichen sollte.
    #[pyo3(signature = (min_alpha=None))]
    fn get_bounding_rect(&self, min_alpha: Option<&Bound<'_, PyAny>>) -> PyResult<PyRect> {
        // **`ValueError`, nicht `TypeError`** - gemessen gibt pygame
        // `ValueError: get_bounding_rect only accepts a single optional
        // min_alpha argument`, auch wenn der Wert bloss die falsche Sorte
        // hat. Das ist dieselbe Ausnahme wie bei zu vielen Argumenten.
        let min_alpha: u8 = match min_alpha {
            None => 1,
            Some(o) => o.extract().map_err(|_| {
                pyo3::exceptions::PyValueError::new_err(
                    "get_bounding_rect only accepts a single optional \
                     min_alpha argument",
                )
            })?,
        };
        let s = self.leihen()?;
        // Ohne Alphakanal **und ohne Schluesselfarbe** gibt es nichts zu
        // messen: pygame liefert dann immer die ganze Flaeche, auch bei
        // `min_alpha = 255`.
        //
        // **Die Schluesselfarbe stand hier nicht**, und der Vermerk war
        // deshalb zu kurz - er stimmte, bevor der Kern die
        // Schluesselfarbe ueberhaupt beachtete. Gemessen an einer
        // schwarzen 24-Bit-Flaeche:
        //
        // ```text
        // ohne Schluesselfarbe        -> Rect(0, 0, 70, 70)
        // nach set_colorkey((0,0,0))  -> Rect(0, 0, 0, 0)
        // ```
        //
        // Die Abkuerzung hat die zweite Zeile verschluckt, obwohl der Kern
        // sie inzwischen richtig rechnet. Genau die Sorte Fehler, die
        // dieses Projekt schon mehrfach bezahlt hat: eine Abkuerzung, die
        // einmal stimmte.
        if (s.get_flags() & SRCALPHA == 0 && s.get_colorkey().is_none()) || min_alpha == 0 {
            return Ok(PyRect::neu(s.get_rect()));
        }
        Ok(PyRect::neu(s.get_bounding_rect(min_alpha)))
    }

    /// Eine Kopie, in der die Farbe schon mit dem Alpha verrechnet ist.
    ///
    /// Die Rechnung ist `((c + 1) * a) >> 8` und steckt in
    /// [`rustgame_core::color::Color::premul_alpha`]; der Kern hat das
    /// **nicht** an der Flaeche, also wird es hier Pixel fuer Pixel
    /// angewandt - roh im Puffer, weil `set_at` auf einer Flaeche ohne
    /// Alphakanal jedes Alpha auf 255 zwingen wuerde.
    fn premul_alpha<'py>(slf: &Bound<'py, PySurface>) -> PyResult<Bound<'py, PySurface>> {
        let k = PySurface::vormultipliziert(&slf.borrow())?;
        PySurface::wie_typ(slf, k, None)
    }

    /// `premul_alpha_ip()` - dasselbe **in** der Flaeche, und sie selbst
    /// zurueck.
    ///
    /// Gemessen: der Rueckgabewert ist die Flaeche, auf der es gerufen
    /// wurde, nicht eine Kopie.
    fn premul_alpha_ip<'py>(slf: &Bound<'py, PySurface>) -> PyResult<Bound<'py, PySurface>> {
        let k = PySurface::vormultipliziert(&slf.borrow())?;
        let (w, h) = k.get_size();
        {
            // Zwei Bindungen, nicht eine: `slf.borrow()` gibt einen
            // Schein, und `leihen_mut()` leiht aus **ihm** aus. In einer
            // Zeile wird der Schein am Semikolon weggeworfen, waehrend die
            // Ausleihe noch lebt - und dann uebersetzt es nicht.
            let g = slf.borrow();
            let mut ziel = g.leihen_mut()?;
            for y in 0..h as i32 {
                for x in 0..w as i32 {
                    ziel.set_at_ungeprueft((x, y), k.get_at_mapped((x, y)));
                }
            }
        }
        Ok(slf.clone())
    }
}

impl PySurface {
    /// Der gemeinsame Teil von `premul_alpha` und `premul_alpha_ip`.
    ///
    /// Ohne Alphakanal ist es ein `ValueError` - gemessen samt Wortlaut.
    fn vormultipliziert(&self) -> PyResult<KernSurface> {
        if !self.mit_alpha()? {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "source surface to be alpha pre-multiplied must have alpha channel",
            ));
        }
        let mut k = self.leihen()?.copy();
        let (w, h) = k.get_size();
        k.mit_pixeln(|daten, stride, (ox, oy)| {
            for y in 0..h as usize {
                for x in 0..w as usize {
                    let i = (oy + y) * stride + ox + x;
                    daten[i] = Color::from_argb(daten[i]).premul_alpha().to_argb();
                }
            }
        });
        Ok(k)
    }
}

#[pymethods]
impl PySurface {

    // ----- Sperren -----------------------------------------------------------

    /// Muss die Flaeche zum Pixelzugriff gesperrt werden?
    ///
    /// Hier stand "Immer `False`: rustgame haelt seine Pixel im eigenen
    /// Speicher, es gibt keine Textur, die erst gesperrt werden muesste."
    /// Der erste Halbsatz war falsch. pygame gibt `True` in **zwei**
    /// Faellen, und keiner hat mit Texturen zu tun (beide gemessen):
    ///
    /// ```text
    /// Surface((8,8)).mustlock()                      -> False
    /// Surface((8,8)).subsurface((0,0,4,4)).mustlock() -> True
    /// s.set_colorkey(c, RLEACCEL); s.mustlock()       -> False
    /// ziel.blit(s, (0,0)); s.mustlock()               -> True
    /// ```
    ///
    /// Eine Teilflaeche ist immer sperrpflichtig, und eine wirklich
    /// lauflaengenkodierte Flaeche auch - letztere aber erst, nachdem SDL
    /// sie kodiert hat, und das geschieht beim ersten Blit.
    ///
    /// Gesperrt werden muss hier trotzdem nichts: [`Self::lock`] zaehlt
    /// nur mit. Der Rueckgabewert ist die Auskunft, die pygame gibt.
    fn mustlock(&self) -> PyResult<bool> {
        self.lebt()?;
        Ok(self.rle_aktiv.load(Reihe::Relaxed) || self.eltern.is_some())
    }

    /// Ist die Flaeche gerade gesperrt?
    ///
    /// **Ohne** Lebendpruefung: das ist die einzige Methode, die pygame
    /// auch auf der toten Anzeigeflaeche noch beantwortet (gemessen,
    /// `False`).
    fn get_locked(&self, py: Python<'_>) -> bool {
        // **Die eine Auskunft, die eine tote Flaeche noch gibt** - gemessen
        // an beiden Todesarten: `get_locked()` sagt `False`, waehrend jede
        // andere Methode `Surface is not initialized` meldet.
        if self.uneingerichtet.load(Reihe::Relaxed) {
            return false;
        }
        if self.ist_anzeige && !rustgame_display::display::get_init() {
            return false;
        }
        self.sperren_aufraeumen(py);
        !self.sperren.lock().expect(SPERRE).is_empty()
    }

    /// Sperre setzen - pygame **zaehlt** sie.
    ///
    /// Bei einer Teilflaeche wird der Elternteil mitgesperrt, und in
    /// **seiner** Liste steht dann die Teilflaeche. Gemessen:
    /// `sub.lock(); sub.get_parent().get_locks()` gibt `(sub,)`.
    fn lock(slf: &Bound<'_, PySurface>) -> PyResult<()> {
        let py = slf.py();
        slf.borrow().lebt()?;
        let ich = slf.clone().unbind();
        let eltern = slf.borrow().eltern.as_ref().map(|e| e.clone_ref(py));
        let wer = ich.into_any();
        slf.borrow()
            .sperren
            .lock()
            .expect(SPERRE)
            .push(Sperre::neu(py, wer.bind(py)));
        if let Some(e) = eltern {
            e.bind(py)
                .borrow()
                .sperren
                .lock()
                .expect(SPERRE)
                .push(Sperre::neu(py, wer.bind(py)));
        }
        Ok(())
    }

    /// Gegenstueck zu [`Self::lock`].
    ///
    /// Ein `unlock()` zuviel ist **kein** Fehler - gemessen: pygame gibt
    /// stillschweigend `None` zurueck.
    fn unlock(slf: &Bound<'_, PySurface>) -> PyResult<()> {
        let py = slf.py();
        slf.borrow().lebt()?;
        let eltern = slf.borrow().eltern.as_ref().map(|e| e.clone_ref(py));
        // **Nur die eigene Sperre, nicht die letzte.**
        //
        // Hier stand ein blankes `pop()`, und das nahm heraus, was zuletzt
        // hineingekommen war - auch einen `BufferProxy`. Gemessen:
        //
        // ```python
        // buf = sf.get_buffer()
        // sf.unlock()
        // sf.get_locked()     # True - der Puffer sperrt weiter
        // ```
        //
        // `lock()` und `unlock()` zaehlen die **ausdruecklichen** Sperren,
        // und die tragen immer die Flaeche selbst als Halter. Ein Puffer
        // oder ein `PixelArray` loest sich anders: er verschwindet.
        let ich: Py<PyAny> = slf.clone().into_any().unbind();
        slf.borrow().sperre_loesen_letzte(py, &ich);
        if let Some(e) = eltern {
            e.bind(py).borrow().sperre_loesen_letzte(py, &ich);
        }
        Ok(())
    }
    /// Wer die Flaeche gesperrt hat - ein **Tupel**, nicht eine Liste.
    ///
    /// Hier stand `Vec<u8>`, was nach Python als `b''` hinausging - ein
    /// leeres Bytesobjekt, nicht ein leeres Tupel; und es war immer leer,
    /// mit der Begruendung, rustgame habe keine Textur zu sperren. Beides
    /// war falsch. pygame zaehlt mit (gemessen: dreimal `lock()` gibt
    /// `(s, s, s)`), und `surface_test.test_get_locks` prueft genau die
    /// Zahl.
    fn get_locks<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        self.lebt()?;
        self.sperren_aufraeumen(py);
        let halter: Vec<Py<PyAny>> =
            self.sperren.lock().expect(SPERRE).iter().filter_map(|s| s.wer(py)).collect();
        PyTuple::new(py, halter)
    }

    /// `<Surface(BxHx32)>`, und mit Flaechen-Alpha `<Surface(BxHx32,
    /// global_alpha=N)>` - beides gemessen.
    fn __repr__(&self) -> PyResult<String> {
        // Gemessen: `<Surface(Dead Display)>` - fuer beide Todesarten,
        // die uneingerichtete und die nach `display.quit()`.
        if self.uneingerichtet.load(Reihe::Relaxed)
            || (self.ist_anzeige && !rustgame_display::display::get_init())
        {
            return Ok("<Surface(Dead Display)>".to_string());
        }
        let a = self.get_alpha()?;
        let s = self.leihen()?;
        let (w, h) = s.get_size();
        // **Die Schluesselfarbe steht mit drin** - gemessen:
        // `<Surface(8x8x32, colorkey=(160, 32, 240, 255))>`. Es gibt also
        // drei Formen, nicht zwei, und die Schluesselfarbe wird mit vier
        // Kanaelen geschrieben, obwohl man sie mit dreien setzt.
        // **Beides kann zusammen dastehen** - gemessen:
        // `<Surface(70x70x32, colorkey=(255, 165, 0, 255), global_alpha=200)>`.
        // Meine erste Fassung kehrte beim Schluesselwort zurueck und
        // verschwieg damit das Flaechen-Alpha; es gibt also **vier**
        // Formen, nicht drei.
        // **Die echte Tiefe**, nicht immer 32: gemessen schreibt pygame
        // `<Surface(4x4x16)>` fuer eine 16-Bit-Flaeche. Hier stand die 32
        // fest, und das war richtig, solange es nur ein Pixelformat gab.
        let tiefe = s.get_format().bitsize;
        let mut zusatz = String::new();
        if let Some(c) = s.get_colorkey() {
            zusatz.push_str(&format!(
                ", colorkey=({}, {}, {}, {})",
                c.r, c.g, c.b, c.a
            ));
        }
        if let Some(n) = a {
            zusatz.push_str(&format!(", global_alpha={n}"));
        }
        return Ok(format!("<Surface({w}x{h}x{tiefe}{zusatz})>"));
        #[allow(unreachable_code)]
        Ok(match a {
            None => format!("<Surface({w}x{h}x{tiefe})>"),
            Some(a) => format!("<Surface({w}x{h}x{tiefe}, global_alpha={a})>"),
        })
    }
}

/// `blit_sequence should be iterator of (Surface, dest)` - der Satz, den
/// `blits` und `fblits` fuer eine untaugliche Folge geben (gemessen).
fn folge_fehler() -> PyErr {
    pyo3::exceptions::PyValueError::new_err("blit_sequence should be iterator of (Surface, dest)")
}

/// Derselbe Gedanke fuer einen einzelnen Eintrag - `fblits` benutzt hier
/// einen **anderen** Satz als `blits` (gemessen).
fn eintrag_fehler() -> PyErr {
    pyo3::exceptions::PyValueError::new_err(
        // pygames Wortlaut, gemessen an `fblits("x")`: es nennt die
        // **Folge**, nicht das Glied.
        "blit_sequence should be iterator of (Surface, dest)",
    )
}

/// Die Teile eines Eintrags `(flaeche, ziel[, bereich[, flags]])`.
///
/// `None`, wenn der Eintrag gar keine Folge ist - eine Zahl etwa. Ein
/// **String** ist in Python zwar eine Folge, taugt hier aber nicht, und
/// pygame lehnt ihn ab; deshalb steht er ausdruecklich da.
fn eintrag_teile<'py>(e: &Bound<'py, PyAny>) -> Option<Vec<Bound<'py, PyAny>>> {
    if e.is_instance_of::<pyo3::types::PyString>() {
        return None;
    }
    e.try_iter().ok()?.collect::<PyResult<Vec<_>>>().ok()
}

/// `special_flags` einlesen und in die Nummerierung des Kerns bringen.
///
/// pygame nimmt hier **nur** ganze Zahlen: `12.245`, `None`, `[]`, `()`
/// und `"str"` geben alle
/// `TypeError: The special_flags parameter must be an int`, ein
/// unbekannter Wert wie `-999`
/// `TypeError: Blit failed (probably the flag used does not exist)`
/// (beides gemessen an `fblits`). Ein `bool` ist in Python eine ganze
/// Zahl und geht durch - genau wie dort.
fn flagge_aus(o: Option<&Bound<'_, PyAny>>) -> PyResult<u32> {
    let Some(o) = o else { return Ok(0) };
    if o.is_none() || o.is_instance_of::<pyo3::types::PyFloat>() {
        return Err(pyo3::exceptions::PyTypeError::new_err(
            "The special_flags parameter must be an int",
        ));
    }
    let v: i64 = o.extract().map_err(|_| {
        pyo3::exceptions::PyTypeError::new_err("The special_flags parameter must be an int")
    })?;
    if !(0..=18).contains(&v) {
        return Err(pyo3::exceptions::PyTypeError::new_err(
            "Blit failed (probably the flag used does not exist)",
        ));
    }
    // pygames BLEND_RGBA_MAX ist 16, im Kern ist es 10 - siehe
    // `konstanten.rs`.
    Ok(misch_art(v as u32))
}

/// Das Pixelformat aus `depth` und `masks`, wie pygames `surface_init`.
///
/// `depth` darf eine Zahl oder eine **Flaeche** sein (dann gilt deren
/// Format), `masks` eine Folge aus vier Zahlen. Die drei Meldungen sind
/// wortgleich gemessen:
///
/// ```text
/// Surface((4,4), depth=48)          ValueError: nonstandard bit depth given
/// Surface((4,4), SRCALPHA, 24)      ValueError: no standard masks exist for given bitdepth with alpha
/// Surface((4,4), 0, 32, ())         ValueError: masks argument must be sequence of four numbers
/// ```
fn format_aus(
    flags: u32,
    depth: Option<&Bound<'_, PyAny>>,
    masks: Option<&Bound<'_, PyAny>>,
) -> PyResult<(Format, Option<Vec<Color>>)> {
    let mit_alpha = flags & SRCALPHA != 0;
    // Eine Vorlageflaeche statt einer Tiefe.
    if let Some(d) = depth {
        if !d.is_none() {
            if let Ok(v) = d.extract::<PyRef<PySurface>>() {
                let f = v.leihen()?.get_format();
                let t = v.leihen()?.get_palette().map(|p| p.to_vec());
                return Ok((f, t));
            }
        }
    }
    let tiefe: u32 = match depth {
        None => 32,
        Some(d) if d.is_none() => 32,
        Some(d) => d
            .extract::<i64>()
            .map_err(|_| {
                pyo3::exceptions::PyTypeError::new_err(
                    "depth must be an integer or a Surface",
                )
            })?
            .clamp(0, 1 << 20) as u32,
    };
    let format = match masks {
        None => Format::standard(tiefe, mit_alpha)
            .map_err(pyo3::exceptions::PyValueError::new_err)?,
        Some(m) if m.is_none() => Format::standard(tiefe, mit_alpha)
            .map_err(pyo3::exceptions::PyValueError::new_err)?,
        Some(m) => {
            let v: Vec<u32> = m.extract().map_err(|_| {
                pyo3::exceptions::PyValueError::new_err(
                    "masks argument must be sequence of four numbers",
                )
            })?;
            if v.len() != 4 {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "masks argument must be sequence of four numbers",
                ));
            }
            // Die Tiefe wird trotzdem geprueft - `Surface((4,4), 0, 48,
            // masken)` gibt in pygame ebenfalls "nonstandard bit depth
            // given", die Masken retten sie nicht.
            Format::standard(tiefe, false).map_err(pyo3::exceptions::PyValueError::new_err)?;
            // Vier Nullen heissen "nimm die Vorgabe" - siehe
            // `Format::aus_masken`.
            Format::aus_masken(tiefe, (v[0], v[1], v[2], v[3]))
                .map_err(pyo3::exceptions::PyValueError::new_err)?
        }
    };
    // **Acht Bit mit Masken ist RGB332, keine Palettenflaeche.** SDL macht
    // aus `(8, 0xE0, 0x1C, 0x03)` `SDL_PIXELFORMAT_RGB332`: gepackt wie 16
    // Bit, ohne Tafel. Hier hing jeder 8-Bit-Flaeche die Vorgabetafel an,
    // und `map_rgb` suchte die naechste Tafelfarbe statt die Bits
    // abzuschneiden - gemessen `(72, 182, 85)` gab 53 statt 85.
    let tafel = (tiefe == 8 && format.masks == (0, 0, 0, 0))
        .then(rustgame_core::surface::palette_vorgabe);
    Ok((format, tafel))
}

/// Ein Pixelwert, wie pygame ihn nach Python gibt.
///
/// Ohne Alphakanal faellt das oberste Byte weg; mit Alphakanal steht dort
/// ein gesetztes oberstes Bit, und pygame reicht den Wert **mit
/// Vorzeichen** hinaus: `map_rgb((10,20,30,40))` auf einer Flaeche mit
/// `SRCALPHA` ist gemessen `-16116706`, nicht `4278850590`.
fn gepackt(v: u32, mit_alpha: bool) -> i32 {
    // Das Abschneiden des obersten Bytes steht jetzt im Kern, wo die
    // Masken bekannt sind - hier wuerde es auch Flaechen treffen, bei
    // denen dort ein **Farbkanal** liegt. Siehe `map_rgb_gepackt`.
    let _ = mit_alpha;
    v as i32
}

/// Das Format, in das `convert(...)` wandeln soll.
///
/// Drei Formen, alle gemessen:
///
/// * eine **Zahl** - die Tiefe (`convert(8)`),
/// * eine **Flaeche** - deren Format wird uebernommen,
/// * ein **Maskenpaar** `(masken, tiefe)`.
fn zielformat(
    o: &Bound<'_, PyAny>,
) -> PyResult<(Format, Option<Vec<rustgame_core::color::Color>>)> {
    if let Ok(f) = o.extract::<PyRef<PySurface>>() {
        let k = f.leihen()?;
        return Ok((k.get_format(), k.get_palette().map(|t| t.to_vec())));
    }
    if let Ok(tiefe) = o.extract::<u32>() {
        let format =
            Format::standard(tiefe, false).map_err(pyo3::exceptions::PyValueError::new_err)?;
        let palette = if tiefe == 8 {
            Some(rustgame_core::surface::palette_vorgabe())
        } else {
            None
        };
        return Ok((format, palette));
    }
    if let Ok((masken, tiefe)) = o.extract::<((u32, u32, u32, u32), u32)>() {
        let format =
            Format::aus_masken(tiefe, masken).map_err(pyo3::exceptions::PyValueError::new_err)?;
        return Ok((format, None));
    }
    Err(pyo3::exceptions::PyValueError::new_err(
        // pygames ganzer Satz - gemessen.
        "invalid argument specifying new format to convert to",
    ))
}

/// Eine **Pixelstelle** - zwei Zahlen, genau zwei.
///
/// Nicht zu verwechseln mit `fenster::stelle_aus`, das pygames
/// `Window(position=...)` liest und dort auch einzelne Konstanten wie
/// `WINDOWPOS_CENTERED` annimmt. Hier geht es um `get_at`/`set_at`, und
/// der Wortlaut ist ein anderer - gemessen:
///
/// ```text
/// s.get_at((1,))      -> TypeError: position must be a sequence of two numbers
/// s.get_at([1, 2, 3]) -> dasselbe
/// s.get_at(1)         -> dasselbe
/// s.get_at("xy")      -> dasselbe
/// s.get_at((1, "a"))  -> dasselbe
/// s.get_at(Vector2(1, 2)) -> Color(0, 0, 0, 255)
/// ```
///
/// Drei Dinge daran sind wichtig und keines davon offensichtlich:
///
/// * **Genau zwei.** `[1, 2, 3]` ist ein Fehler, obwohl die ersten beiden
///   Zahlen taugen. `umwandlung::punkt_aus` nimmt `>= 2` und waere hier
///   deshalb falsch - es ist fuer `draw` gebaut, wo ein `Rect` als Punkt
///   gilt.
/// * **Ein `Rect` gilt hier nicht.** Bei `draw.line` schon.
/// * **Ein `Vector2` gilt.** Also reicht `extract::<(i32, i32)>()` nicht,
///   das lehnt ihn ab.
fn stelle_aus(o: &Bound<'_, PyAny>) -> PyResult<(i32, i32)> {
    let fehler = || {
        pyo3::exceptions::PyTypeError::new_err("position must be a sequence of two numbers")
    };
    // Erst die Zahlenfolge mit genau zwei Gliedern - das deckt Tupel,
    // Liste und `Vector2` ab, weil alle drei sich als `Vec<f64>` lesen
    // lassen.
    let v = o.extract::<Vec<f64>>().map_err(|_| fehler())?;
    if v.len() != 2 {
        return Err(fehler());
    }
    Ok((v[0] as i32, v[1] as i32))
}

/// Die Meldung, die `subsurface` fuer ein untaugliches Rechteck gibt.
///
/// **`ValueError`, nicht `TypeError`** - und das ist die Ueberraschung:
/// gemessen gibt `s.subsurface(0, 0, 2)` ein
/// `ValueError: invalid rectstyle argument`, wo man nach drei statt vier
/// Zahlen einen `TypeError` erwarten wuerde. Dasselbe fuer
/// `subsurface(0, 0)` und `subsurface("x")`.
///
/// Damit hat derselbe Vorgang in diesem Nachbau schon **drei** Wortlaute,
/// je nach Modul, und alle drei sind nachgemessen:
///
/// | Stelle | Art | Satz |
/// |---|---|---|
/// | `rect.c` | `TypeError` | `Argument must be rect style object` |
/// | `mask.c` | `TypeError` | `invalid rectstyle argument` |
/// | `surface.c` (`subsurface`) | **`ValueError`** | `invalid rectstyle argument` |
///
/// Einen davon durchzureichen waere also an zwei von drei Stellen falsch.
fn rechteckform() -> PyErr {
    pyo3::exceptions::PyValueError::new_err("invalid rectstyle argument")
}

/// Baut ein Rechteck aus **1, 2 oder 4** Teilen - drei sind keine Form.
///
/// `subsurface` nimmt in pygame alle diese Formen:
///
/// ```python
/// s.subsurface(rect)              # ein rechteckartiges Objekt
/// s.subsurface((0, 0), (2, 2))    # Ecke und Groesse getrennt
/// s.subsurface(0, 0, 2, 2)        # vier Zahlen
/// ```
///
/// `None` heisst "kein Rechteck" und der Aufrufer macht daraus
/// [`rechteckform`]. Der Rueckgabetyp ist bewusst `Option` und nicht
/// `PyResult`: so steht die Entscheidung, **welcher** Fehler es wird, an
/// der Aufrufstelle - und die ist je Modul verschieden.
fn rechteck_aus_teilen(teile: &[&Bound<'_, PyAny>]) -> Option<KernRect> {
    match teile.len() {
        // Ein Teil: das uebliche rechteckartige Objekt, samt `.rect`-Feld
        // und dem Schutz gegen Ringe darin.
        1 => crate::rechteck::rect_aus_objekt(teile[0]),
        // Zwei Teile: Ecke und Groesse, jedes als Zahlenpaar.
        2 => {
            let a = teile[0].extract::<(f64, f64)>().ok()?;
            let b = teile[1].extract::<(f64, f64)>().ok()?;
            Some(KernRect::new(a.0 as i32, a.1 as i32, b.0 as i32, b.1 as i32))
        }
        4 => {
            let mut z = [0i32; 4];
            for (i, t) in teile.iter().enumerate() {
                z[i] = t.extract::<f64>().ok()? as i32;
            }
            Some(KernRect::new(z[0], z[1], z[2], z[3]))
        }
        _ => None,
    }
}
