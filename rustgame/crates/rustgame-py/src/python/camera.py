"""`pygame.camera` - Bilder von einer Kamera.

**Hier stand einmal "es gibt hier keine Kamera".** Die Liste der Geraete
war leer, `Camera(...)` warf einen Fehler, und die Begruendung daneben
lautete: ein Kamerabild brauche den Geraetezugriff des Betriebssystems
(DirectShow, V4L2), und der ginge nur ueber FFI - der Arbeitsbereich habe
kein `unsafe`.

Der erste Teil ist wahr, der zweite war eine Verwechslung, und zwar
dieselbe wie bei `system.get_total_ram` und bei `pygame.midi`: **`unsafe`
braucht irgendwer, aber nicht dieses Programm.** Die Kiste `nokhwa`
spricht unter Windows mit Media Foundation, unter Linux mit V4L2 und unter
macOS mit AVFoundation, hinter einer sicheren Schnittstelle.

Ausgeloest hat es eine einzige Probe: `pygame.camera.list_cameras()` gab
auf dieser Maschine `['USB  Live camera']`, und `get_image()` ein Bild
davon. Es gab also eine Kamera, und pygame benutzte sie.

## Was gemessen ist

Alles hier stammt aus echtem pygame-ce 2.5.7 mit dieser Kamera:

```text
get_backends()                  ['_camera (MSMF)', 'OpenCV']
list_cameras()                  ['USB  Live camera']   (str, nicht int)
Camera(77)                      TypeError: bad argument type for
                                built-in operation
Camera("gibtesnicht")           ValueError: Couldn't find a camera
                                with that name
get_size() vor start()          der **Wunsch** (320, 240)
get_size() nach start()         die Wirklichkeit (320, 240)
query_image() vor start()       error: Camera needs to be started
                                to read data
get_raw() vor start()           derselbe Fehler
get_controls()                  (False, False, -1)
set_controls(hflip=True)        (True, False, -1)
set_controls(brightness=1.5)    TypeError: 'float' object cannot be
                                interpreted as an integer
get_image()                     (320, 240), 32 Bit,
                                Masken (0xff0000, 0xff00, 0xff, 0)
get_image(zu_klein)             ValueError: Destination surface not the
                                correct width or height.
get_raw()                       None  (nur unter MSMF)
colorspace(surf, "YUV")         eine Flaeche
colorspace(surf, "RGB")         ValueError: Incorrect colorspace value
stop(); stop()                  beides None
```

Vier Messungen sind ueberraschend:

* **`get_size()` sagt vor dem Start etwas anderes als danach.** Vorher
  nennt pygame den Wunsch, nachher die Wirklichkeit:
  `Camera(name, (1,1))` gibt vor `start()` `(1, 1)` und danach
  `(320, 240)`. Beim ersten Messen hatte ich gestartet und nur die zweite
  Zeile gesehen, beim zweiten nicht gestartet und nur die erste - zwei
  richtige Messungen, die zusammen widerspruechlich aussahen. **Zwei
  Messungen, die sich widersprechen, messen meist zwei Zustaende.**
* **Die Groesse ist nur ein Wunsch.** Was das Geraet nicht kann, bekommt
  man nicht: `(99999, 99999)` wird zu dem, was naeher liegt. Wer hier
  einen Fehler wirft, verhaelt sich anders als pygame.
* **`format` wird nicht geprueft.** `"YUV"`, `"RGBA"`, sogar `"quatsch"`
  kommen durch. Das Umrechnen macht `colorspace` - und die nimmt
  umgekehrt **nur** `"YUV"` und `"HSV"`, nicht einmal `"RGB"`.
* **`query_image()` ist unter Windows immer `False`.** Siehe dort.

## Drei Stellen, an denen hier **mehr** steht als bei pygame

Unter pygames MSMF-Stueck tun drei Aufrufe nichts:

```text
Aufruf              pygame (MSMF)   hier
query_image()       immer False     True, solange sie laeuft
get_raw()           immer None      die Bytes des Geraets
get_controls()[2]   immer -1        der Reglerwert, sonst -1
```

Dasselbe pygame gibt unter Linux bei `get_raw()` die Bytes des
V4L2-Puffers heraus - die Luecke ist also keine Absicht, sondern ein
nicht umgesetzter Aufruf. Bei `query_image` wiegt sie schwer, denn
pygames **eigenes** Doku-Muster

```python
while not cam.query_image():
    pass
bild = cam.get_image()
```

haengt unter Windows fuer immer: gemessen war der Aufruf vierzigmal in
Folge `False`. Eine Luecke nachzubauen, waere hier die schlechtere
Antwort - und `-1` bei der Helligkeit heisst "weiss nicht", nicht
"dunkel". Alle drei stehen in den benannten Grenzen, damit die Messung
den Unterschied nicht fuer einen Fehler haelt.
"""

# Mit Unterstrich, damit der Name nicht in `dir()` landet: pygames
# `camera.py` hat ihn dort nicht, und `__dir__` zaehlt jetzt den
# eigenen Namensraum mit.
import operator as _operator

# **Vier mitimportierte Namen.** `pygame/camera.py` schreibt
# `import os, sys, platform, warnings`; sie stehen damit in
# `dir(pygame.camera)`. Gebraucht werden sie hier nicht - die Geraetearbeit
# steckt in `rustgame-display/src/kamera.rs` -, aber `dir()` ist die
# Schnittstelle.
import os                                            # noqa: F401
import platform                                      # noqa: F401
import sys                                           # noqa: F401
import warnings                                      # noqa: F401
# **`ABC` und `abstractmethod` stehen hier, weil sie bei pygame hier
# stehen.** `pygame/camera.py` schreibt `from abc import ABC,
# abstractmethod` und baut damit seine Basisklasse `AbstractCamera`. Die
# beiden Namen sind damit Teil von `dir(pygame.camera)` - und ein Name
# zuviel ist derselbe Fehler wie einer zuwenig.
from abc import ABC, abstractmethod                  # noqa: F401

__all__ = [
    "Camera",
    "error",
    "AbstractCamera",
    "colorspace",
    "get_backends",
    "init",
    "list_cameras",
    "quit",
]


def _pg():
    import rustgame_py

    return rustgame_py


def __dir__():
    """`__all__` **und** was das Modul selbst eingefuehrt hat.

    Vorher stand hier `sorted(__all__)`, und damit war `dir()` eine von
    Hand gepflegte Liste - genau die Sorte, an der dieses Projekt schon
    neun blinde Flecken hatte. Ein `import` am Kopf des Moduls stand dann
    im Woerterbuch, aber nicht in `dir()`.

    `globals()` ist die Antwort, die sich selbst nachfuehrt.
    """
    return sorted(set(__all__) | {n for n in globals() if not n.startswith("_")})


def __getattr__(name):
    if name == "error":
        return _pg().error
    raise AttributeError(
        "module %r has no attribute %r" % (__name__, name)
    )


_gestartet = False


def _pre_init_placeholder():
    """Der Fehler, den jede Kameraarbeit **vor** `init()` gibt.

    Wortgleich aus `pygame/camera.py` uebertragen, samt dem zweiten Zweig:
    ist das Modul hochgefahren und die Namen trotzdem nicht ausgetauscht,
    dann stimmt etwas nicht, und ein `NotImplementedError` sagt das
    deutlicher als ein stiller Erfolg.
    """
    if not _gestartet:
        raise _pg().error("pygame.camera is not initialized")
    raise NotImplementedError()


def _pre_init_placeholder_varargs(*_, **__):
    """Dasselbe, aber als Methode mit beliebigen Argumenten."""
    _pre_init_placeholder()


def init(backend=None):
    """Faehrt das Kameramodul hoch und **tauscht die Namen aus**.

    `backend` nimmt pygame an und benutzt es, um zwischen `_camera` und
    OpenCV zu waehlen. Hier gibt es einen Weg; der Name wird angenommen
    und nicht gebraucht.
    """
    # **Ein unbekannter Name ist ein `ValueError`** - gemessen gibt
    # `camera.init("argname")` in pygame `ValueError: unrecognized
    # backend name`. Hier wurde jeder Name angenommen, und ein Tippfehler
    # blieb unbemerkt. pygame kennt `_camera` und `OpenCV`; hier gibt es
    # den ersten Weg, und der zweite wird genannt, damit ein Programm,
    # das ihn anfordert, denselben Fehler sieht wie bei pygame ohne
    # `cv2` - naemlich keinen, solange der Name bekannt ist.
    if backend is not None and str(backend).lower() not in (
        "_camera", "opencv", "opencv-mac", "videocapture"
    ):
        raise ValueError("unrecognized backend name")
    global _gestartet, Camera, list_cameras
    _gestartet = True
    Camera = _EchteKamera
    list_cameras = _echte_liste


def quit():                                          # noqa: A001
    """Faehrt es herunter - und **setzt den Platzhalter zurueck**.

    Gemessen: nach `quit()` ist `camera.Camera` wieder
    `_PreInitPlaceholderCamera`, und `list_cameras()` gibt den Fehler.
    """
    global _gestartet, Camera, list_cameras
    _gestartet = False
    Camera = _PreInitPlaceholderCamera
    list_cameras = _pre_init_placeholder


def get_backends():
    """Welche Wege es zu einer Kamera gibt.

    pygame gibt auf dieser Maschine `['_camera (MSMF)', 'OpenCV']` - der
    zweite Eintrag steht nur da, weil `cv2` in derselben Umgebung liegt
    und pygame dafuer ein zweites, reines Python-Stueck hat. Hier gibt es
    den einen Weg, und der heisst wie pygames erster.
    """
    return _pg()._kamera_rueckwaende()


def list_cameras():
    """Die gefundenen Kameras, als Namen - **nach `init()`**.

    Vor `init()` steht unter `list_cameras` der Platzhalter; `init()`
    tauscht ihn gegen diese Funktion. Siehe
    `_PreInitPlaceholderCamera`.

    **Leer ist keine Ausnahme.** Ein Rechner ohne Kamera bekommt eine
    leere Liste und keinen Fehler - genau wie bei pygame, und
    `camera_test.py` rechnet damit: seine Tests ueberspringen sich selbst,
    wenn die Liste leer ist.
    """
    return _pg()._kamera_geraete()


def colorspace(surface, color, dest_surface=None, /):
    """Rechnet eine Flaeche in einen anderen Farbraum um.

    Die einzige Funktion dieses Moduls, die ohne Geraet etwas tut.

    ## Die drei Namen sind pygames

    `color` fuer einen **Farbraum** liest sich falsch, steht aber so in
    `camera.pyi` - und der Name eines Parameters ist Schnittstelle, keine
    Geschmacksfrage. Hier hiess er `color_space`, und damit ging
    `colorspace(s, color_space="YUV")`, was pygame ablehnt. Der Schraegstrich
    macht alle drei stellungsgebunden, so wie pygames Cython-Fassung sie
    nimmt.

    **Es gibt genau zwei Raeume: `"YUV"` und `"HSV"`.** Hier stand einmal
    `"RGB"` daneben, mit dem Vermerk, es lasse die Farben stehen - und
    gemessen gibt pygame dafuer `ValueError: Incorrect colorspace value`.
    Der Vermerk war nie gemessen worden; die Probe `camera.colorspace` hat
    ihn beim ersten Lauf umgeworfen. Auch Kleinschreibung zaehlt nicht:
    `"yuv"` ist derselbe Fehler.
    """
    pg = _pg()
    color_space = color
    if color_space not in ("YUV", "HSV"):
        raise ValueError("Incorrect colorspace value")
    ziel = dest_surface if dest_surface is not None else surface.copy()
    b, h = surface.get_size()
    for y in range(h):
        for x in range(b):
            c = surface.get_at((x, y))
            r, g, bl = c.r / 255.0, c.g / 255.0, c.b / 255.0
            if color_space == "YUV":
                yy = 0.299 * r + 0.587 * g + 0.114 * bl
                u = 0.492 * (bl - yy) + 0.5
                v = 0.877 * (r - yy) + 0.5
                neu = (yy, u, v)
            else:
                gross, klein = max(r, g, bl), min(r, g, bl)
                spanne = gross - klein
                if spanne == 0:
                    ton = 0.0
                elif gross == r:
                    ton = ((g - bl) / spanne % 6) / 6.0
                elif gross == g:
                    ton = ((bl - r) / spanne + 2) / 6.0
                else:
                    ton = ((r - g) / spanne + 4) / 6.0
                saettigung = 0.0 if gross == 0 else spanne / gross
                neu = (ton, saettigung, gross)
            ziel.set_at(
                (x, y),
                pg.Color(*[max(0, min(255, int(v * 255))) for v in neu], c.a),
            )
    return ziel


class AbstractCamera(ABC):
    """Die Form, die eine Kamera hat.

    Eine abstrakte Basisklasse tut ihre Arbeit, indem sie abstrakt ist:
    diese hier hat dieselben sieben Pflichtmethoden wie pygames, und davon
    laesst sich erben. Genau das tut `Camera` darunter.
    """

    # **Auch `__init__` ist abstrakt** - gemessen:
    # `pygame.camera.AbstractCamera.__abstractmethods__` nennt sieben
    # Namen, und der erste ist `__init__`. Ohne ihn gab
    # `AbstractCamera()` hier `TypeError: AbstractCamera() takes no
    # arguments` statt pygames `Can't instantiate abstract class
    # AbstractCamera without an implementation for abstract methods
    # '__init__', 'get_image', ...`. Gefunden hat das die Frage, was
    # eine Klasse sagt, wenn man sie **falsch** ruft.
    @abstractmethod
    def __init__(self, device=0, size=(320, 200), format="RGB"):
        ...

    @abstractmethod
    def start(self):
        ...

    @abstractmethod
    def stop(self):
        ...

    # **`get_controls` und `set_controls` stehen hier nicht**, und das ist
    # gemessen: `dir(pygame.camera.AbstractCamera)` hat genau sechs Namen -
    # `start`, `stop`, `get_size`, `query_image`, `get_image`, `get_raw`.
    # Die zwei Regler fehlen in pygames Formbeschreibung, obwohl das echte
    # Kamerastueck sie hat.
    #
    # Hier standen sie, und damit war die abstrakte Klasse **strenger** als
    # pygames: ein eigenes Kamerastueck ohne Regler liess sich nicht einmal
    # anlegen. Gefunden hat das die Probe `Klassennamen`, die die
    # **rufbaren** Namen aller 65 Klassen in beide Richtungen vergleicht -
    # `api_coverage.py` meldet nur Fehlendes, nie Ueberschuss.

    @abstractmethod
    def get_size(self):
        ...

    @abstractmethod
    def query_image(self):
        ...

    @abstractmethod
    def get_image(self, surface=None):
        ...

    @abstractmethod
    def get_raw(self):
        ...


class Camera:
    """Eine Kamera.

    **Erbt nicht von `AbstractCamera`** - und das ist gemessen: pygames
    `_camera.Camera` hat `object` als einzige Basis, obwohl `camera.py`
    daneben eine abstrakte Basisklasse definiert. Hier stand
    `class Camera(AbstractCamera)`, was besser aussieht und `isinstance`
    anders beantwortet. Gefunden hat es eine Probe, die die **Basisklassen**
    aller 65 Klassen vergleicht - dieselbe Probe, die `ScancodeWrapper` als
    Nicht-Tupel entlarvt haette, wenn sie frueher da gewesen waere.

    Die abstrakte Klasse bleibt stehen: sie ist bei pygame die
    Formbeschreibung, die ein eigenes Kamerastueck erfuellen soll, und
    `dir(pygame.camera)` zeigt sie.

    Gebaut wird sie ohne Strom: der Konstruktor sucht das Geraet, waehlt
    das Format und fragt die Groesse ab; lesen laesst sich erst nach
    `start()`. Diese Reihenfolge hat pygame auch, und sie ist messbar -
    `get_size()` antwortet vor `start()`, `query_image()` nicht.
    """

    def __init__(self, device=0, size=(640, 480), format="RGB"):
        # **Ein `int` ist hier ein TypeError, kein Geraet.** pygames
        # C-Argumentleser nimmt nur eine Zeichenkette; `Camera(0)` gibt
        # gemessen `TypeError: bad argument type for built-in operation`.
        # Eine Nummer freundlich zu dulden, hiesse, Programme zu bedienen,
        # die unter pygame nicht laufen.
        if not isinstance(device, str):
            raise TypeError("bad argument type for built-in operation")
        w, h = (int(size[0]), int(size[1])) if size else (640, 480)
        # `format` wird angenommen und nicht geprueft - gemessen laesst
        # pygame jede Zeichenkette durch.
        self._format = format
        self._kam = _pg()._Kamera(device, w, h)
        self.device_name = device

    # ----------------------------------------------------------- Strom
    def start(self):
        """Faengt an. Zweimal starten ist kein Fehler."""
        self._kam.starten()

    def stop(self):
        """Hoert auf. Zweimal aufhoeren ist kein Fehler - gemessen gibt
        pygames `stop()` beide Male `None`."""
        self._kam.stoppen()

    def _laeuft(self):
        if not self._kam.laeuft:
            raise _pg().error("Camera needs to be started to read data")

    # ----------------------------------------------------------- Regler
    def get_controls(self):
        """`(hflip, vflip, brightness)`.

        **`-1` bei der Helligkeit heisst "weiss nicht", nicht "dunkel".**
        pygame gibt auf dieser Maschine `-1`, weil sein MSMF-Stueck den
        Regler ueberhaupt nicht abfragt. Hier wird er abgefragt; hat das
        Geraet keinen, steht wieder `-1` da.
        """
        return (self._kam.hflip, self._kam.vflip, self._kam.helligkeit)

    def set_controls(self, **kwargs):
        """Stellt Regler und gibt danach `get_controls()` zurueck.

        Ohne Argumente stellt sie nichts und sagt nur, wie es steht -
        gemessen tut pygames `set_controls()` genau das.

        Die Helligkeit muss eine **ganze** Zahl sein: pygame gibt bei
        `brightness=1.5` gemessen `TypeError: 'float' object cannot be
        interpreted as an integer`. Das macht hier `_operator.index`, und
        zwar mit derselben Meldung.
        """
        if "hflip" in kwargs:
            self._kam.hflip = bool(kwargs["hflip"])
        if "vflip" in kwargs:
            self._kam.vflip = bool(kwargs["vflip"])
        if "brightness" in kwargs:
            self._kam.helligkeit_setzen(_operator.index(kwargs["brightness"]))
        return self.get_controls()

    def get_size(self):
        """Die Groesse, die wirklich herauskommt - **nicht die gewuenschte.**

        Geht auch vor `start()`; das Format steht schon fest, sobald das
        Geraet gefunden ist.
        """
        return self._kam.groesse()

    # ----------------------------------------------------------- Bilder
    def query_image(self):
        """Liegt ein Bild bereit?

        Solange die Kamera laeuft: ja. **pygame gibt hier unter Windows
        immer `False`** - gemessen vierzigmal in Folge -, womit sein
        eigenes Doku-Muster `while not cam.query_image(): pass` fuer immer
        haengt. Siehe den Modulkopf.
        """
        self._laeuft()
        return True

    def get_image(self, surface=None):
        """Ein Bild als Flaeche - 32 Bit, Masken `(0xff0000, 0xff00, 0xff, 0)`.

        Mit `surface` wird **in diese** gezeichnet und sie selbst
        zurueckgegeben; passt die Groesse nicht, ist das gemessen ein
        `ValueError` mit pygames Wortlaut.
        """
        self._laeuft()
        bild = self._kam.bild()
        if surface is None:
            return bild
        if surface.get_size() != bild.get_size():
            raise ValueError(
                "Destination surface not the correct width or height."
            )
        surface.blit(bild, (0, 0))
        return surface

    def get_raw(self):
        """Die Bytes, wie das Geraet sie liefert - **undekodiert**.

        Je nach Kamera ist das MJPEG, YUYV oder NV12; `get_image()` rechnet
        daraus ein Bild. pygames MSMF-Stueck gibt hier `None`, sein
        V4L2-Stueck die Bytes. Hier gibt es sie ueberall.
        """
        self._laeuft()
        return self._kam.roh()


class _PreInitPlaceholderCamera(AbstractCamera):  # noqa: F821
    """Was `camera.Camera` **vor** `init()` ist.

    ## Der Fund, den keine Probe hatte

    Hier gab es diesen Platzhalter nicht: `camera.Camera` war von Anfang an
    die echte Klasse, und `list_cameras()` arbeitete ohne `init()`.
    Gemessen gibt pygame dort:

    ```text
    camera.list_cameras()   error: pygame.camera is not initialized
    camera.Camera("x")      error: pygame.camera is not initialized
    camera.get_backends()   ['_camera (MSMF)', 'OpenCV']   -  geht ohne init
    ```

    `init()` tauscht `Camera` und `list_cameras` gegen die echten aus,
    `quit()` setzt den Platzhalter **zurueck** - beides gemessen.

    Gefunden hat das eine Probe, die die **Basisklassen** aller Klassen
    vergleicht: pygames `camera.Camera` erbt von `AbstractCamera`, sein
    `_camera.Camera` nicht. Das waren nicht zwei Meinungen ueber eine
    Klasse, sondern **zwei Klassen** - und die eine fehlte hier ganz.
    """

    def __init__(self, *args, **kwargs):
        _pre_init_placeholder()

    start = _pre_init_placeholder_varargs
    stop = _pre_init_placeholder_varargs
    get_controls = _pre_init_placeholder_varargs
    set_controls = _pre_init_placeholder_varargs
    get_size = _pre_init_placeholder_varargs
    query_image = _pre_init_placeholder_varargs
    get_image = _pre_init_placeholder_varargs
    get_raw = _pre_init_placeholder_varargs


# **Die echte Klasse heisst `Camera`** (`_camera.Camera.__name__` ist bei
# pygame genau das), steht aber vor `init()` nicht unter diesem Namen. Also
# wird sie hier gesichert, und `Camera` zeigt auf den Platzhalter - wie in
# `pygame/camera.py`, Zeile 73.
_EchteKamera = Camera
Camera = _PreInitPlaceholderCamera
# **Dasselbe mit `list_cameras`**, und aus demselben Grund: die echte
# Funktion muss `list_cameras` **heissen** (`_camera.list_cameras.__name__`
# ist bei pygame genau das), steht aber vor `init()` nicht unter diesem
# Namen. Hier hiess sie `_echte_liste`, und damit trug sie den falschen
# Namen in `__name__` - gefunden von der Probe `Funktionsnamen`.
_echte_liste = list_cameras
list_cameras = _pre_init_placeholder
