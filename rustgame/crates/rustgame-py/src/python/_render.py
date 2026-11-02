"""`pygame._render` - `Renderer`, `Texture` und `Image`.

**Hier ist der Renderer schon einer.** In pygame steht hinter diesen drei
Klassen SDLs beschleunigter Zeichenweg: eine `Texture` liegt auf der
Grafikkarte, `Renderer.clear()` ist ein GL-Aufruf, und `to_surface()`
holt das Bild zurueck in den Hauptspeicher. rustgame hat keine
Grafikkarte im Spiel - jede Flaeche liegt ohnehin im Hauptspeicher.

Damit faellt die ganze Klasse weg, aber nicht ihre Bedeutung: `Renderer`
ist hier eine **Flaeche mit Zeichenfarbe**, `Texture` eine Flaeche mit
Faerbung und Alpha, `Image` ein Ausschnitt daraus. Was die Tests pruefen,
sind Pixel - und die stimmen.

## Was gemessen ist

| Aufruf | pygame-ce 2.5.7 |
|---|---|
| `Renderer(win).draw_color` | `Color(0, 0, 0, 0)` - **nicht** Schwarz |
| `to_surface()` | neue Flaeche in Fenstergroesse, Alpha 255 |
| `to_surface(kleiner)` | `ValueError` |
| `to_surface(area=Rect(5,5,20,20))` | Flaeche 20x20, Inhalt **verschoben** |
| `draw_rect(Rect(10,10,11,11))` | Umriss bis x = 20 einschliesslich |
| `logical_size` | `(0, 0)`, bis jemand es setzt |
| `coordinates_to_window((10,10))` | bei `logical_size (10,10)`: `(100,100)` |
| `get_viewport()` ohne Angabe | `Rect(0, 0, 100, 100)` |
| `Texture(r, (80,60)).alpha` | `255` |
| `Texture(...).color` | `Color(255,255,255,255)`, Alpha bleibt 255 |
| `Texture(..., depth=33)` | `ValueError` |
| `static` **und** `target` | `ValueError` - genau eine Art |
| `texture.width = 100` | `AttributeError` |
| Dreieck mit Eckfarben | `Ergebnis = Textur * Eckfarbe / 255` |

Die letzte Zeile ist die ueberraschende: bei `BLENDMODE_NONE` mischt SDL
**nicht** mit dem Alpha der Eckfarben. Gemessen an
`render_test.test_draw_triangle`, wo die Eckfarbe `(100,100,100,100)`
lautet und das Ergebnis genau `Textur * 100/255` ist - das vierte Byte
bleibt liegen. Wer hier blendet, kommt auf ein Drittel der Helligkeit.
"""

# **`import math as _math`**, nicht `import math`. Gemessen hat
# `dir(pygame._render)` genau drei Namen: `Image`, `Renderer`, `Texture`.
# Ein `import math` am Kopf macht daraus vier, und **ein Name zuviel ist
# derselbe Fehler wie einer zuwenig**. Mit dem Unterstrich ist der Name
# nicht Teil der Schnittstelle - dieselbe Loesung, die pygames eigenes
# `typing.py` mit einem `del` am Dateiende waehlt.
import math as _math

__all__ = ["Renderer", "Texture", "Image"]


def _pg():
    """Das Wurzelmodul - erst beim Aufruf, nicht beim Laden.

    Dieser Text laeuft waehrend der Einrichtung von `rustgame_py`; ein
    `import` ganz oben liefe dabei in einen Ring.
    """
    import rustgame_py

    return rustgame_py


# Die fuenf Mischarten, die SDL kennt. `BLENDMODE_MUL + 1` ist keine -
# `test_draw_blend_mode` prueft genau das.
_MISCHARTEN = (0, 1, 2, 4, 8)


def _farbe(wert):
    """Was auch immer hereinkommt, als `Color`."""
    pg = _pg()
    if isinstance(wert, pg.Color):
        return pg.Color(wert.r, wert.g, wert.b, wert.a)
    return pg.Color(wert)


def _rechteck(wert):
    """Was auch immer hereinkommt, als `Rect` - oder `None`."""
    if wert is None:
        return None
    pg = _pg()
    if isinstance(wert, pg.Rect):
        return pg.Rect(wert)
    return pg.Rect(wert)


class Renderer:
    """Der Zeichner - hier eine Flaeche mit Zustand."""

    @classmethod
    def from_window(cls, window):
        """Ein Zeichner fuer ein **vorhandenes** Fenster.

        In pygame holt das den Zeichner, den SDL dem Fenster schon
        mitgegeben hat, und scheitert mit `pygame.error`, wenn es keinen
        gibt. Hier gibt es keinen vorgefertigten - ein Zeichner ist eine
        Flaeche mit Zustand -, also wird einer gebaut. Das Ergebnis ist
        dasselbe: ein Zeichner, der auf dieses Fenster malt.
        """
        return cls(window)

    def __init__(
        self, window, index=-1, accelerated=-1, vsync=False, target_texture=False
    ):
        pg = _pg()
        # **Erst pruefen, dann lesen.** `Renderer('x')` gab hier
        # `AttributeError: 'str' object has no attribute 'size'`, weil die
        # Groesse unmittelbar gefragt wurde; pygame gibt einen `TypeError`
        # und nennt den erwarteten Typ (gemessen an
        # `pygame._sdl2.video.Renderer`):
        #
        #     TypeError: Argument 'window' has incorrect type
        #                (expected pygame.window.Window, got str)
        #
        # Die Art ist das Wichtige: ein `except TypeError` fing den Fall
        # vorher nicht.
        if not hasattr(window, "size"):
            raise TypeError(
                "Argument 'window' has incorrect type (expected "
                "pygame.window.Window, got %s)" % type(window).__name__
            )
        self._window = window
        b, h = window.size
        self._flaeche = pg.Surface((b, h))
        self._flaeche.fill((0, 0, 0))
        self._draw_color = pg.Color(0, 0, 0, 0)
        self._blend = 0
        self._ziel = None
        self._logical = (0, 0)
        self._scale = (1.0, 1.0)
        self._viewport = None
        self._vsync = bool(vsync)

    # ----- Zustand ------------------------------------------------------

    @property
    def draw_color(self):
        return _farbe(self._draw_color)

    @draw_color.setter
    def draw_color(self, wert):
        self._draw_color = _farbe(wert)

    @property
    def draw_blend_mode(self):
        return self._blend

    @draw_blend_mode.setter
    def draw_blend_mode(self, wert):
        if wert not in _MISCHARTEN:
            raise _pg().error("unsupported blend mode")
        self._blend = wert

    @property
    def logical_size(self):
        return self._logical

    @logical_size.setter
    def logical_size(self, wert):
        b, h = wert
        self._logical = (int(b), int(h))

    @property
    def scale(self):
        return self._scale

    @scale.setter
    def scale(self, wert):
        b, h = wert
        self._scale = (float(b), float(h))

    @property
    def target(self):
        return self._ziel

    @target.setter
    def target(self, wert):
        if wert is not None and not isinstance(wert, Texture):
            raise TypeError("target must be a Texture or None")
        self._ziel = wert

    def get_viewport(self):
        """Der Sichtausschnitt - ohne Angabe die ganze Flaeche.

        In **logischen** Masszahlen, wenn eine logische Groesse gesetzt
        ist: `video_test` setzt bei einem 800x600-Fenster die logische
        Groesse 1920x1080 und erwartet genau die zurueck.
        """
        pg = _pg()
        if self._viewport is not None:
            return pg.Rect(self._viewport)
        b, h = self._logical if self._logical != (0, 0) else self._ziel_flaeche().size
        return pg.Rect(0, 0, b, h)

    def set_viewport(self, rect):
        self._viewport = _rechteck(rect)

    def compose_custom_blend_mode(self, color_mode, alpha_mode):
        """SDLs Zahlenformel fuer eine eigene Mischart.

        Reine Bitrechnung, kein Geraet noetig - `SDL_ComposeCustomBlendMode`
        packt sechs Felder in ein Wort.
        """
        (sc, dc, oc) = color_mode
        (sa, da, oa) = alpha_mode
        return (
            (oc << 4)
            | (sc << 8)
            | (dc << 12)
            | (oa << 16)
            | (sa << 20)
            | (da << 24)
        )

    # ----- Umrechnung Fenster <-> logische Masse -------------------------

    def _massstab(self):
        b, h = self._ziel_flaeche().size
        lb, lh = self._logical
        if lb <= 0 or lh <= 0:
            return (1.0, 1.0)
        return (b / lb, h / lh)

    def coordinates_to_window(self, coordinate):
        mx, my = self._massstab()
        x, y = coordinate
        return (x * mx, y * my)

    def coordinates_from_window(self, coordinate):
        mx, my = self._massstab()
        x, y = coordinate
        return (x / mx, y / my)

    # ----- Zeichnen ------------------------------------------------------

    def _ziel_flaeche(self):
        """Wohin gezeichnet wird - die Zieltextur oder der Bildspeicher."""
        if self._ziel is not None:
            return self._ziel._flaeche
        return self._flaeche

    def clear(self):
        self._ziel_flaeche().fill(self._draw_color)

    def draw_point(self, point):
        f = self._ziel_flaeche()
        x, y = point
        f.set_at((int(x), int(y)), self._draw_color)

    def draw_line(self, p1, p2):
        _pg().draw.line(self._ziel_flaeche(), self._draw_color, p1, p2)

    def draw_rect(self, rect):
        _pg().draw.rect(self._ziel_flaeche(), self._draw_color, _rechteck(rect), 1)

    def fill_rect(self, rect):
        _pg().draw.rect(self._ziel_flaeche(), self._draw_color, _rechteck(rect))

    def draw_triangle(self, p1, p2, p3):
        _pg().draw.polygon(self._ziel_flaeche(), self._draw_color, [p1, p2, p3], 1)

    def fill_triangle(self, p1, p2, p3):
        _pg().draw.polygon(self._ziel_flaeche(), self._draw_color, [p1, p2, p3])

    def draw_quad(self, p1, p2, p3, p4):
        _pg().draw.polygon(
            self._ziel_flaeche(), self._draw_color, [p1, p2, p3, p4], 1
        )

    def fill_quad(self, p1, p2, p3, p4):
        _pg().draw.polygon(self._ziel_flaeche(), self._draw_color, [p1, p2, p3, p4])

    def blit(self, source, dest=None, area=None, special_flags=0):
        """Zeichnet, was sich zeichnen laesst.

        `Texture` und `Image` kennt der Renderer selbst; alles andere wird
        **gefragt**: `source.draw(area, dest)`. Genau so haengt sich
        `pygame.sprite` an den Renderer, und `render_test.test_blit`
        prueft, dass ein beliebiges Objekt mit `draw` durchkommt.
        """
        if isinstance(source, Texture):
            source.draw(srcrect=area, dstrect=dest)
        elif isinstance(source, Image):
            source.draw(srcrect=area, dstrect=dest)
        else:
            source.draw(area, dest)
        return _rechteck(dest)

    def present(self):
        """Zeigt das Bild - hier im Fenster, wenn es eines gibt."""
        try:
            flaeche = self._window.get_surface()
        except Exception:
            return
        flaeche.blit(self._flaeche, (0, 0))
        self._window.flip()

    def to_surface(self, surface=None, area=None):
        """Das Bild als Flaeche.

        Ohne Flaeche kommt eine neue in der Groesse des Ausschnitts; mit
        einer muss sie **mindestens** so gross sein, sonst `ValueError`.
        Der Ausschnitt ist die Stelle in der **Quelle**: gemessen landet
        bei `area=Rect(7,7,40,40)` das Pixel `(10,10)` auf `(3,3)`.
        """
        pg = _pg()
        quelle = self._flaeche
        r = _rechteck(area)
        if r is None:
            r = pg.Rect(0, 0, *quelle.size)
        r = r.clip(pg.Rect(0, 0, *quelle.size))
        if surface is None:
            neu = pg.Surface((r.width, r.height))
            neu.blit(quelle, (0, 0), r)
            return neu
        if surface.width < r.width or surface.height < r.height:
            raise ValueError("the surface is too small")
        surface.blit(quelle, (0, 0), r)
        return surface


class Texture:
    """Ein Bild im Speicher des Zeichners - hier schlicht eine Flaeche."""

    def __init__(
        self,
        renderer,
        size,
        depth=0,
        static=False,
        streaming=False,
        target=False,
        scale_quality=None,
    ):
        """**Sieben Argumente, nicht sechs.**

        `scale_quality` fehlte hier, und gemessen nimmt pygames `Texture`
        es an: `Texture(*17 Argumente)` sagt dort `function takes at most
        7 arguments`, hier waren es sechs. Gefunden hat das die Probe
        `Argumentfehler`, die aus beiden Fehlertexten die Stelligkeit
        liest und vergleicht.

        SDL waehlt damit, wie beim Strecken geglaettet wird
        (`SCALEQUALITY_NEAREST`, `_LINEAR`, `_BEST`). Ein **Feld** dazu
        gibt es bei pygame nicht - die Probe `Klassennamen` vergleicht
        alle oeffentlichen Namen der Klasse, und `scale_quality` ist
        keiner davon. Der Wert wird also angenommen und beiseitegelegt,
        so wie pygame ihn an SDL weitergibt und danach vergisst.
        """
        pg = _pg()
        self._scale_quality = scale_quality
        if depth not in (0, 8, 12, 15, 16, 24, 32):
            raise ValueError("unsupported depth")
        # **Genau eine Art.** SDL kennt `STATIC`, `STREAMING` und `TARGET`
        # als sich ausschliessende Zugriffsarten; zwei davon zugleich gibt
        # gemessen einen `ValueError`.
        if sum(bool(x) for x in (static, streaming, target)) > 1:
            raise ValueError(
                "only one of static, streaming, or target can be true"
            )
        b, h = size
        b, h = int(b), int(h)
        if b <= 0 or h <= 0:
            raise ValueError("size must be positive")
        self._renderer = renderer
        self._flaeche = pg.Surface((b, h), pg.SRCALPHA)
        self._color = pg.Color(255, 255, 255, 255)
        self._alpha = 255
        self._blend = 0
        self.static = bool(static)
        self.streaming = bool(streaming)
        self.target = bool(target)

    # ----- Auskunft ------------------------------------------------------

    @property
    def renderer(self):
        return self._renderer

    @property
    def width(self):
        return self._flaeche.width

    @property
    def height(self):
        return self._flaeche.height

    @property
    def alpha(self):
        return self._alpha

    @alpha.setter
    def alpha(self, wert):
        self._alpha = max(0, min(255, int(wert)))

    @property
    def blend_mode(self):
        return self._blend

    @blend_mode.setter
    def blend_mode(self, wert):
        if wert not in _MISCHARTEN:
            raise _pg().error("unsupported blend mode")
        self._blend = wert

    @property
    def color(self):
        """Die Faerbung - **immer mit Alpha 255**.

        Gemessen: nach `texture.color = Color(100,110,120,130)` gibt
        `texture.color` die Farbe `(100,110,120,255)`. Das Alpha steht
        getrennt in `texture.alpha`; SDL hat dafuer zwei Aufrufe.
        """
        return _pg().Color(self._color.r, self._color.g, self._color.b, 255)

    @color.setter
    def color(self, wert):
        c = _farbe(wert)
        self._color = _pg().Color(c.r, c.g, c.b, 255)

    def get_rect(self, **kwargs):
        r = _pg().Rect(0, 0, self.width, self.height)
        for name, wert in kwargs.items():
            setattr(r, name, wert)
        return r

    @classmethod
    def from_surface(cls, renderer, surface):
        t = cls(renderer, surface.size)
        t._flaeche.blit(surface, (0, 0))
        # Eine Quellflaeche **mit** Alphakanal bringt ihre Mischart mit -
        # so macht es `SDL_CreateTextureFromSurface` auch.
        if surface.get_flags() & _pg().SRCALPHA:
            t._blend = 1
        return t

    def update(self, surface, area=None):
        """Legt eine Flaeche in die Textur - ohne Groessenaenderung."""
        pg = _pg()
        r = _rechteck(area)
        eigen = pg.Rect(0, 0, self.width, self.height)
        if r is None:
            r = eigen
        r = r.clip(eigen)
        if r.width <= 0 or r.height <= 0:
            return
        teil = pg.Surface((r.width, r.height), pg.SRCALPHA)
        teil.blit(surface, (0, 0))
        self._flaeche.blit(teil, r.topleft)

    # ----- Zeichnen ------------------------------------------------------

    def _gefaerbt(self, flaeche):
        """Die Flaeche mit Faerbung und Alpha - eine Kopie."""
        pg = _pg()
        c = self._color
        k = flaeche
        if (c.r, c.g, c.b) != (255, 255, 255):
            k = flaeche.copy()
            k.fill((c.r, c.g, c.b, 255), special_flags=pg.BLEND_RGB_MULT)
        if self._alpha != 255:
            if k is flaeche:
                k = flaeche.copy()
            k.fill((255, 255, 255, self._alpha), special_flags=pg.BLEND_RGBA_MULT)
        return k

    def draw(
        self,
        srcrect=None,
        dstrect=None,
        angle=0.0,
        origin=None,
        flip_x=False,
        flip_y=False,
    ):
        """Zeichnet die Textur auf den Zeichner.

        `angle` dreht **im Uhrzeigersinn** - SDLs Richtung, die der von
        `pygame.transform.rotate` entgegengesetzt ist.
        """
        pg = _pg()
        ziel = self._renderer._ziel_flaeche()
        eigen = pg.Rect(0, 0, self.width, self.height)
        q = _rechteck(srcrect)
        q = eigen if q is None else q.clip(eigen)
        if q.width <= 0 or q.height <= 0:
            return
        z = _rechteck(dstrect)
        if z is None:
            z = pg.Rect(0, 0, *ziel.size)
        teil = self._gefaerbt(self._flaeche.subsurface(q))
        if (teil.width, teil.height) != (z.width, z.height):
            teil = pg.transform.scale(teil, (z.width, z.height))
        if flip_x or flip_y:
            teil = pg.transform.flip(teil, flip_x, flip_y)
        if angle:
            gedreht = pg.transform.rotate(teil, -angle)
            # Der Drehpunkt liegt ohne Angabe in der **Mitte** des Ziels.
            if origin is None:
                mitte = z.center
            else:
                mitte = (z.x + origin[0], z.y + origin[1])
            r = gedreht.get_rect(center=mitte)
            ziel.blit(gedreht, r.topleft)
        else:
            ziel.blit(teil, z.topleft)

    def draw_triangle(
        self, p1, p2, p3, uv1=(0, 0), uv2=(0, 1), uv3=(1, 1),
        color1=None, color2=None, color3=None,
    ):
        """Ein Dreieck mit Texturkoordinaten und Eckfarben."""
        weiss = (255, 255, 255, 255)
        _gouraud(
            self,
            (p1, p2, p3),
            (uv1, uv2, uv3),
            (color1 or weiss, color2 or weiss, color3 or weiss),
        )

    def draw_quad(
        self, p1, p2, p3, p4,
        uv1=(0, 0), uv2=(0, 1), uv3=(1, 1), uv4=(1, 0),
        color1=None, color2=None, color3=None, color4=None,
    ):
        """Ein Viereck - **zwei Dreiecke**, genau wie bei SDL."""
        weiss = (255, 255, 255, 255)
        c1, c2 = color1 or weiss, color2 or weiss
        c3, c4 = color3 or weiss, color4 or weiss
        _gouraud(self, (p1, p2, p3), (uv1, uv2, uv3), (c1, c2, c3))
        _gouraud(self, (p1, p3, p4), (uv1, uv3, uv4), (c1, c3, c4))


def _gouraud(textur, punkte, uvs, farben):
    """Ein texturiertes Dreieck mit Farbverlauf an den Ecken.

    Die Formel ist die gemessene und **nicht** die naheliegende: bei
    `BLENDMODE_NONE` ist das Ergebnis `Textur * Eckfarbe / 255`, und das
    Alpha der Eckfarbe bleibt **liegen**. Nachgerechnet an
    `render_test.test_draw_triangle`: an der Stelle `(30, 50)` liegen die
    Gewichte bei genau `(0.5, 0.5, 0)`, die Eckfarbe ist damit
    `(90, 80, 70)`, und das Ergebnis `(28, 38, 45)` ist Punkt fuer Punkt
    `(80,120,160) * (90,80,70) / 255`.
    """
    pg = _pg()
    ziel = textur._renderer._ziel_flaeche()
    quelle = textur._flaeche
    tb, th = quelle.width, quelle.height
    zb, zh = ziel.size

    (x0, y0), (x1, y1), (x2, y2) = [(float(p[0]), float(p[1])) for p in punkte]
    flaeche2 = (x1 - x0) * (y2 - y0) - (y1 - y0) * (x2 - x0)
    if flaeche2 == 0:
        return
    links = max(0, int(_math.floor(min(x0, x1, x2))))
    rechts = min(zb - 1, int(_math.ceil(max(x0, x1, x2))))
    oben = max(0, int(_math.floor(min(y0, y1, y2))))
    unten = min(zh - 1, int(_math.ceil(max(y0, y1, y2))))

    tf = textur._color
    ta = textur._alpha
    for y in range(oben, unten + 1):
        for x in range(links, rechts + 1):
            px, py = x + 0.5, y + 0.5
            w0 = ((x1 - px) * (y2 - py) - (y1 - py) * (x2 - px)) / flaeche2
            w1 = ((x2 - px) * (y0 - py) - (y2 - py) * (x0 - px)) / flaeche2
            w2 = 1.0 - w0 - w1
            if w0 < 0 or w1 < 0 or w2 < 0:
                continue
            u = w0 * uvs[0][0] + w1 * uvs[1][0] + w2 * uvs[2][0]
            v = w0 * uvs[0][1] + w1 * uvs[1][1] + w2 * uvs[2][1]
            sx = min(tb - 1, max(0, int(u * tb)))
            sy = min(th - 1, max(0, int(v * th)))
            t = quelle.get_at((sx, sy))
            werte = []
            for i in range(4):
                c = w0 * farben[0][i] + w1 * farben[1][i] + w2 * farben[2][i]
                m = tf.r if i == 0 else tf.g if i == 1 else tf.b if i == 2 else ta
                werte.append(int(t[i] * c / 255.0 * m / 255.0))
            ziel.set_at((x, y), pg.Color(*[min(255, max(0, w)) for w in werte]))


class Image:
    """Ein Ausschnitt einer Textur mit eigener Faerbung."""

    def __init__(self, texture_or_image, srcrect=None):
        pg = _pg()
        if isinstance(texture_or_image, Image):
            vorbild = texture_or_image
            self._texture = vorbild._texture
            grund = pg.Rect(vorbild.srcrect)
        else:
            # Wie bei `Renderer`: pygame nennt den erwarteten Typ und gibt
            # einen `TypeError` (gemessen an `pygame._sdl2.video.Image`:
            # `Cannot convert str to pygame._sdl2.video.Texture`), hier
            # kam ein `AttributeError` aus der naechsten Zeile.
            if not hasattr(texture_or_image, "width"):
                raise TypeError(
                    "Cannot convert %s to pygame._sdl2.video.Texture"
                    % type(texture_or_image).__name__
                )
            self._texture = texture_or_image
            grund = pg.Rect(0, 0, self._texture.width, self._texture.height)
        r = _rechteck(srcrect)
        self._srcrect = grund if r is None else r
        self._color = pg.Color(255, 255, 255, 255)
        self._alpha = 255.0
        self._blend_mode = 0
        self._angle = 0.0
        self._origin = None
        self._flip_x = False
        self._flip_y = False

    # **Eigenschaften, keine blanken Felder.**
    #
    # Die sieben standen als gewoehnliche Attribute in `__init__`, und von
    # aussen tat das dasselbe: `img.alpha = 128` wirkte, `img.angle` gab
    # den Winkel. Nur **`dir(Image)`** zeigte sie nicht - ein Attribut
    # entsteht erst mit dem Objekt, die Klasse weiss nichts davon.
    # pygames `Image` ist eine Cython-Klasse, dort sind es echte
    # Eigenschaften, und `api_coverage.py` fragt die **Klasse**.
    #
    # Das ist keine Buchhalterei: wer `hasattr(Image, "angle")` prueft -
    # und das tut jeder, der eine Schnittstelle abtastet -, bekam `False`.
    # Die Typen sind dabei mitgemessen: `alpha` und `angle` sind
    # **Fliesskomma** (pygame gibt `255.0` und `0.0`), `blend_mode` eine
    # ganze Zahl, `color` eine `Color`, `flip_x`/`flip_y` Wahrheitswerte,
    # `origin` entweder `None` oder ein Punkt.

    @property
    def color(self):
        return self._color

    @color.setter
    def color(self, wert):
        self._color = _pg().Color(wert)

    @property
    def alpha(self):
        return self._alpha

    @alpha.setter
    def alpha(self, wert):
        self._alpha = float(wert)

    @property
    def blend_mode(self):
        return self._blend_mode

    @blend_mode.setter
    def blend_mode(self, wert):
        self._blend_mode = int(wert)

    @property
    def angle(self):
        return self._angle

    @angle.setter
    def angle(self, wert):
        self._angle = float(wert)

    @property
    def origin(self):
        return self._origin

    @origin.setter
    def origin(self, wert):
        self._origin = None if wert is None else (wert[0], wert[1])

    @property
    def flip_x(self):
        return self._flip_x

    @flip_x.setter
    def flip_x(self, wert):
        self._flip_x = bool(wert)

    @property
    def flip_y(self):
        return self._flip_y

    @flip_y.setter
    def flip_y(self, wert):
        self._flip_y = bool(wert)

    @property
    def texture(self):
        return self._texture

    @property
    def srcrect(self):
        return _pg().Rect(self._srcrect)

    @srcrect.setter
    def srcrect(self, wert):
        self._srcrect = _rechteck(wert)

    def get_rect(self):
        """Das Rechteck des Ausschnitts - **ohne Schluesselwoerter.**

        `Texture.get_rect` nimmt sie (`**kwargs`, so steht es in pygames
        `video.pyi`), `Image.get_rect` **nicht**: dort schreibt derselbe
        Stub `def get_rect(self) -> Rect`. Hier nahmen beide sie, und
        damit ging `img.get_rect(center=(1, 1))`, was bei pygame
        scheitert.
        """
        return _pg().Rect(0, 0, self._srcrect.width, self._srcrect.height)

    def draw(self, srcrect=None, dstrect=None):
        """Zeichnet den Ausschnitt - die eigene Faerbung geht mit ein."""
        pg = _pg()
        r = _rechteck(srcrect)
        if r is None:
            r = pg.Rect(self._srcrect)
        else:
            r = pg.Rect(
                self._srcrect.x + r.x, self._srcrect.y + r.y, r.width, r.height
            )
        alte_farbe, altes_alpha = self._texture.color, self._texture.alpha
        self._texture.color = self.color
        self._texture.alpha = self.alpha
        try:
            self._texture.draw(
                srcrect=r,
                dstrect=dstrect,
                angle=self.angle,
                origin=self.origin,
                flip_x=self.flip_x,
                flip_y=self.flip_y,
            )
        finally:
            self._texture.color = alte_farbe
            self._texture.alpha = altes_alpha
