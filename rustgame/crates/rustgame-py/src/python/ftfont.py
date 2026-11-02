"""`pygame.ftfont` - die Schnittstelle von `pygame.font` auf `freetype`.

**Dasselbe Haus, andere Tuer.** Dieses Modul bietet genau die Klasse
`Font` an, die `pygame.font` hat - nur steht dahinter das zweite
Schriftsystem. Wer `import pygame.ftfont as font` schreibt, bekommt
Unicode jenseits der Grundebene (Zeichen ueber U+FFFF) und sonst
dieselben Aufrufe.

Das ist reines Python, auch im echten pygame; uebertragen aus
`pygame/ftfont.py` (LGPL 2.1, dieselbe Lizenz wie rustgame). Der Grund
steht in `ftfont_test.py`: diese Datei faehrt **die ganze
`font_test.py`** ein zweites Mal, nur gegen `pygame.ftfont` statt gegen
`pygame.font`. Ohne dieses Modul laeuft keiner der Tests.

## Die drei Stellen, an denen es nicht einfach durchreicht

* **Die Aufloesung ersetzt die Groessenrechnung.** `pygame.font.Font(None,
  16)` rechnet die Groesse mit 0,6875 klein; hier passiert dasselbe ueber
  `resolution = get_default_resolution() * 0.6875`, und zwar **nur** bei
  der mitgelieferten Schrift.
* **Fett und kursiv sind zweierlei.** Bringt die Datei den Schnitt schon
  mit (`style_name` enthaelt "Bold"), tut `set_bold(True)` nichts - sonst
  wird `wide` beziehungsweise `oblique` gesetzt.
* **Durchgestrichen zeichnet dieses Modul selbst**, mit `draw.line`;
  `freetype` kennt keinen Strich durch die Mitte.
"""

# **`draw` steht hier, weil es bei pygame hier steht.** `pygame/ftfont.py`
# schreibt `from pygame.sysfont import match_font, get_fonts, SysFont` und
# daneben `import pygame.draw` - das Unterstreichen und Durchstreichen von
# Text zeichnet es selbst mit `draw.line`. Hier macht das der Kern, aber
# der Name gehoert zum Namensraum: ein Name zuviel ist derselbe Fehler wie
# einer zuwenig.
# **Die Namen, die `dir()` zeigen muss - unabhaengig von `__all__`.**
#
# `__dir__` las hier `__all__`, und das ging schief, sobald `lib.rs` am
# Ende `__all__` auf pygames Fassung zurechtschneidet: pygames eigenes
# `__all__` laesst `FontType` und `draw` **aus**, obwohl beide im
# Woerterbuch stehen. Damit verschwand `FontType` aus `dir()`, und die
# Probe `Untermodule gesamt` hat es sofort gemeldet.
#
# Zwei Fragen, zwei Listen: `_SICHTBAR` ist, was `dir()` zeigt;
# `__all__` ist, was `import *` bringt.
_SICHTBAR = (
    "Font",
    "FontType",
    "init",
    "quit",
    "get_default_font",
    "get_init",
    "SysFont",
    "match_font",
    "get_fonts",
    "draw",
)

__all__ = [
    "Font",
    "FontType",
    "init",
    "quit",
    "get_default_font",
    "get_init",
    "SysFont",
    "match_font",
    "get_fonts",
]


def _pg():
    import rustgame_py

    return rustgame_py


def __dir__():
    """Damit `dir()` auch die durchgereichten Namen zeigt.

    Ein Modul, das seine Namen erst beim Zugriff holt, hat sie nicht in
    seinem Woerterbuch - und `api_coverage.py` zaehlt genau das ab.

    **`draw` gehoert dazu.** `pygame/ftfont.py` schreibt `import
    pygame.draw`, weil es Unterstreichen und Durchstreichen selbst
    zeichnet; hier macht das der Kern, aber der Name ist Teil des
    Namensraums - und ein Name zuviel ist derselbe Fehler wie einer
    zuwenig.
    """
    return sorted(
        set(_SICHTBAR) | {n for n in globals() if not n.startswith("_")}
    )


def __getattr__(name):
    """Holt die durchgereichten Namen erst beim Zugriff.

    Dieser Text laeuft waehrend der Einrichtung von `rustgame_py`; ein
    `from rustgame_py import ...` ganz oben liefe in einen Ring.
    """
    if name == "draw":
        return _pg().draw
    pg = _pg()
    if name in ("init", "quit", "get_default_font"):
        return getattr(pg.freetype, name)
    if name in ("match_font", "get_fonts", "get_sdl_ttf_version"):
        return getattr(pg.font, name)
    if name == "get_default_resolution":
        return pg.freetype.get_default_resolution
    if name == "Font":
        return _font_klasse()
    if name == "FontType":
        return _font_klasse()
    raise AttributeError(
        "module %r has no attribute %r" % (__name__, name)
    )


# Hier stand `_NEUZEILE = chr(10)` - der Zeilenumbruch als eigener Name,
# weil ein Rueckstrich-n in diesem eingebetteten Text schon ein echter
# Umbruch waere. Gebraucht hat ihn nur der Umbruch zu `wraplength`, und
# den hat `pygame.ftfont` nicht. Beides ist weg; wer ihn wieder braucht,
# braucht auch diesen Vermerk.

_KLASSE = [None]


def _font_klasse():
    """Baut die Klasse beim ersten Zugriff.

    Sie erbt von `freetype.Font`, und den gibt es waehrend der
    Einrichtung noch nicht.
    """
    if _KLASSE[0] is not None:
        return _KLASSE[0]
    pg = _pg()
    ft = pg.freetype

    class Font(ft.Font):
        """Eine Schrift mit der Schnittstelle von `pygame.font.Font`."""

        __unull = "\x00"
        __bnull = b"\x00"

        @staticmethod
        def _vorbereiten(file, size):
            """Aus `(file, size)` wird `(Datei, Groesse, Aufloesung)`.

            ## Warum das nicht in `__init__` stehen darf

            pygames `ftfont.py` schreibt genau diese drei Zeilen in
            `__init__`:

            ```python
            if isinstance(bfile, bytes) and bfile == self.__default_font:
                file = None
            ```

            Der **mitgelieferte Name** steht damit fuer die mitgelieferte
            Schrift: `Font("freesansbold.ttf")` ist dasselbe wie
            `Font(None)`. Ohne diese Zeile sucht die Schriftkiste eine
            Datei dieses Namens im Arbeitsverzeichnis - und findet sie
            nicht.

            Bei pygame geht das in `__init__` auf, weil dort `tp_new` nur
            die Huelle anlegt und `tp_init` die Datei laedt. **Bei PyO3
            gibt es keinen `tp_init`-Schlitz**: `#[new]` laedt schon, und
            zwar mit den Argumenten, die der Aufrufer gegeben hat - lange
            bevor `__init__` sie umschreiben koennte. Die Umschreibung muss
            also nach `__new__`.

            Gefunden hat es `font_test.test_load_default_font_filename`
            (ueber `ftfont_test.py`), und es war ein Fehler, der
            **wuerfelte**: allein gefahren war die Datei gruen, im vollen
            Durchgang fielen zwei Tests um - je nachdem, ob ein frueher
            gelaufener Test das Arbeitsverzeichnis gewechselt hatte und
            `freesansbold.ttf` dort zufaellig lag.
            """
            size = max(size, 1)
            if isinstance(file, str):
                try:
                    bfile = pg.encode_file_path(file, ValueError)
                except ValueError:
                    bfile = ""
            else:
                bfile = file
            vorgabe = pg.encode_file_path(ft.get_default_font())
            if isinstance(bfile, bytes) and bfile == vorgabe:
                file = None
            if file is None:
                # Der Faktor 0,6875 ist derselbe wie in `pygame.font`:
                # `Font(None, 16)` ist dort 11 Pixel hoch. Hier wird er
                # ueber die Aufloesung eingeschleust, statt die Groesse
                # anzufassen. **`pygame.freetype` rechnet nicht klein** -
                # siehe `rustgame_font::default_font_voll`.
                resolution = int(ft.get_default_resolution() * 0.6875)
                if resolution == 0:
                    resolution = 1
            else:
                resolution = 0
            return file, size, resolution

        def __new__(cls, file=None, size=20):
            datei, groesse, aufl = cls._vorbereiten(file, size)
            return super().__new__(cls, datei, size=groesse, resolution=aufl)

        def __init__(self, file=None, size=20):
            datei, groesse, aufl = self._vorbereiten(file, size)
            file = datei
            super().__init__(file, size=groesse, resolution=aufl)
            self.strength = 1.0 / 12.0
            self.kerning = False
            self.origin = True
            self.pad = True
            self.ucs4 = True
            self.underline_adjustment = 1.0
            self.strikethrough = False
            schnitt = self.style_name.lower() if file is not None else ""
            self._bold = "bold" in schnitt
            self._italic = "italic" in schnitt or "slanted" in schnitt

        def render(self, text, antialias, color, bgcolor=None):
            """Zeichnet Text auf eine neue Flaeche.

            **Kein `wraplength`.** Hier stand eines, mit einem Umbruch von
            Hand dahinter - und pygames eigenes `ftfont.py` hat es nicht:

                pygame    render(self, text, antialias, color, bgcolor=None)
                vorher    render(self, text, antialias, color, bgcolor=None, wraplength=0)

            Ein Parameter zuviel ist derselbe Fehler wie einer zuwenig:
            `ftfont.Font.render(.., wraplength=5)` ging hier und scheitert
            bei pygame. Den Umbruch kennt nur `pygame.font.Font.render`,
            und der liegt in `text.rs`.
            """
            if text is None:
                text = ""
            if isinstance(text, str) and self.__unull in text:
                raise ValueError("A null character was found in the text")
            if isinstance(text, bytes) and self.__bnull in text:
                raise ValueError("A null character was found in the text")
            gemerkt = self.antialiased
            self.antialiased = bool(antialias)
            try:
                s, _ = super().render(text, color, bgcolor)
                if self.strikethrough:
                    # Den Strich durch die Mitte zieht dieses Modul selbst -
                    # `freetype` kennt ihn nicht.
                    masse = s.get_size()
                    asc = self.get_ascent()
                    anfang = (0, asc * 15 // 24)
                    ende = (masse[0], anfang[1])
                    breite = max(masse[1] // 20, 1)
                    pg.draw.line(s, color, anfang, ende, breite)
                return s
            finally:
                self.antialiased = gemerkt

        def set_bold(self, value):
            """Fett - **nachgemacht**, wenn die Datei es nicht mitbringt."""
            if not self._bold:
                self.wide = bool(value)

        def get_bold(self):
            return self.wide or self._bold

        bold = property(get_bold, set_bold)

        def set_italic(self, value):
            if not self._italic:
                self.oblique = bool(value)

        def get_italic(self):
            return self.oblique or self._italic

        italic = property(get_italic, set_italic)

        def set_underline(self, value):
            self.underline = bool(value)

        def get_underline(self):
            return self.underline

        def set_strikethrough(self, value):
            self.strikethrough = value

        def get_strikethrough(self):
            return self.strikethrough

        def metrics(self, text):
            return self.get_metrics(text)

        def get_ascent(self):
            return self.get_sized_ascender()

        def get_descent(self):
            return self.get_sized_descender()

        def get_height(self):
            return self.get_sized_ascender() - self.get_sized_descender() + 1

        def get_linesize(self):
            return self.get_sized_height()

        def size(self, text):
            return self.get_rect(text).size

        # Hier stand `_umbrechen`, der Umbruch Wort fuer Wort. Er haengt
        # an `wraplength`, und das hat `pygame.ftfont.Font.render` nicht -
        # siehe den Vermerk dort. Was keinen Aufrufer hat, bleibt nicht
        # liegen; den Umbruch macht `pygame.font.Font.render` in
        # `text.rs`.

        # **Hier stand mehr, und das war zu viel.**
        #
        # `set_linesize`, `point_size` und `align` gehoeren zu
        # `pygame.font.Font`, nicht hierher - `font_test.py` zaehlt fuer
        # `pygame.ftfont` eine feste Liste von 23 Methoden ab und
        # bemaengelt jede, die zuviel da ist. Die Tests, fuer die ich sie
        # eingebaut hatte, ueberspringen sich fuer dieses Modul ohnehin
        # selbst.

    # **`__qualname__` von Hand.** Die Klasse entsteht in einer

    # Funktion, und Python schreibt das mit:

    # `_font_klasse.<locals>.Font` statt `Font`. pygames `ftfont.Font`

    # heisst `Font`, und der Name steht in jedem

    # Rueckverfolgungsauszug - gefunden von der Probe

    # `Funktionsnamen`.

    Font.__qualname__ = "Font"

    # Der Satz aus der erzeugten Tabelle - diese Klasse entsteht erst
    # beim ersten Zugriff, und die Ueberlagerung in `lib.rs` lief da
    # schon. `FontType` ist dasselbe Objekt, also genuegt eine Zeile.
    try:
        import rustgame_py

        satz = rustgame_py._klassendoku.get("ftfont.Font")
        if satz is not None:
            Font.__doc__ = satz
    except Exception:                                # noqa: BLE001
        pass

    _KLASSE[0] = Font
    return Font


def get_init():
    """Ist das Schriftsystem eingerichtet?"""
    return _pg().freetype.get_init()


def SysFont(name, size, bold=0, italic=0, constructor=None):
    """Eine Systemschrift als `ftfont.Font`."""
    pg = _pg()
    if constructor is None:

        def constructor(fontpath, size, bold, italic):
            font = _font_klasse()(fontpath, size)
            font.set_bold(bold)
            font.set_italic(italic)
            return font

    return pg.sysfont.SysFont(name, size, bold, italic, constructor)
