# -*- coding: utf-8 -*-
"""
lamacrash_world.py
==================
Die Höhenwelten von Lama Crash - Hintergrund der Bühne, durch den der Ballon
steigt (nur pygame-Primitive, keine Bilddateien):

    0 Wiese (ab ×1)   · 1 Wolken (ab ×1,5) · 2 Gewitter (ab ×3)
    3 Nordlicht (ab ×10) · 4 Weltall (ab ×100) · 5 Mond (nur die Landung)

**Feste Schnittstelle** (Web ``lamacrash_world.js`` als ``PG.lamaCrashWorld``,
Rust ``lamacrash_world.rs``) - Signaturen nur nach Absprache ändern:

- ``clear_cache()``
- ``sky(world, w, h)``          -> Himmelsverlauf w x h (gecacht; Wiese und
  Wolken mit Sonne oben links)
- ``layer(world, depth, w, h)`` -> senkrecht kachelbare Ebene w x h
  (SRCALPHA; depth 0 = fern .. 2 = nah), gecacht - höchstens die aktuelle und
  die nächste Welt im Speicher halten
- ``ground(w, h)``              -> Weide mit Hügeln, Zaun, Lama-Stall, Lamas
  und Startplatz (w x ground_h(h), SRCALPHA), liegt bei Höhe 0 am unteren
  Bühnenrand. Startplatz (Holzpodest mit rotem Startkreis, 17 % von h breit):
  Mitte x = w * 15 // 100, Deck-Oberkante (= Korbboden) bei Höhe 0 auf
  rect.bottom - max(16, h * 5 // 100); um ihn herum bleiben ~10 % der Breite
  frei, der Stall steht rechts (71 .. 90 %)
- ``ground_h(h)``               -> Höhe dieses Bodenstreifens (30 % von h)
- ``rise_px(alt, h)``           -> Steighöhe in Pixeln (Boden wandert so weit
  nach unten); Ebene ``depth`` verschiebt sich um rise_px * PARALLAX[depth]
- ``draw(s, rect, world, share, alt, t)`` -> ganze Bühne in rect zeichnen
  (auf rect beschnitten): Himmel, Ebenen mit Parallaxe, Boden und belebte
  Kleinigkeiten (Vögel, Flugzeuge, Regen, Blitze, Polarlicht-Bänder, Jets mit
  Kondensstreifen, Satelliten, Sternschnuppen, Funkeln, Erdkrümmung, ferne
  Planeten, wachsender Mond im Weltall); bei ``share`` > 0 wird die ganze
  Szene von world+1 mit share/255 darübergeblendet (Weltall/Mond: kein share)
  - alt = Höhe als gebrochener Takt, t = Animationszeit in Sekunden
- ``moon_scene(w, h)``          -> Mondoberfläche mit Erde am Himmel (Landung)
- ``name_key(world)``           -> Text-Schlüssel des Weltnamens
- Konstanten ``PARALLAX`` (Tempo der Ebenen 0..2), ``NAMES`` (Weltnamen)

Zufall nur über ``_hash`` (deterministisch), keine Zeit außer ``t`` - und nie
der Platzpunkt: Was zu sehen ist, hängt nur von Welt, Anteil, Höhe und Zeit ab.

Speicher: Jede Weltfläche (Himmel, Ebenen, Boden, Blitze, Polarlicht, Erde,
Mond, Planeten) gehört zur Gruppe (Welt, w, h). ``draw`` verwirft in jedem
Bild alle Gruppen außer der aktuellen und der nächsten Welt (``_keep``), und
``_cached`` hält ohnehin höchstens zwei Gruppen (die am längsten unbenutzte
fliegt). Ohne Überblendung baut ``draw`` die nächste Welt vorab - je Bild
höchstens eine Fläche, damit kein Weltwechsel ruckelt. Für die Überblendung
gibt es eine bühnengroße Zwischenfläche. Nur eine Bühnengröße gleichzeitig
zeichnen (eine zweite verdrängt die erste).
"""

import math

import pygame

from . import lamacrash_logic as L
from . import lamacrossing_draw as LCD

_cache = {}
_groups = []           # Welt-Gruppen (welt, w, h), zuletzt benutzte hinten
_GROUPS = 2            # höchstens so viele: aktuelle + nächste Welt
_SMALL_MAX = 300       # kleine Teile (Vögel, Flugzeuge, Lichter): ab hier leeren

# Tempo der Ebenen (fern .. nah) relativ zur Steighöhe
PARALLAX = (0.22, 0.5, 1.0)
NAMES = ("meadow", "clouds", "storm", "aurora", "space", "moon")

_RISE = 22             # Steigtempo: h * _RISE / 10000 Pixel je Takt
_EASE = 60             # Anfahren: Tempo wächst in den ersten _EASE Takten linear
_GROUND = 30           # Bodenstreifen in % der Bühnenhöhe
_PAD_X = 15            # Startplatz-Mitte in % der Bühnenbreite (= Korb-Mitte im Spiel)
_PAD_W = 17            # Deckbreite in % der Bühnenhöhe
_TOP = 35              # Ding 0 einer Höhenreihe steht bei Höhe 0 auf 35 % von h
_SAT_P = 0.3           # Parallaxe der Satelliten
_PLANET_P = 0.16       # Parallaxe der fernen Planeten
_EARTH_P = 0.6         # so schnell sinkt die Erdkrümmung (× rise_px)
_FLASH = 1.9           # Sekunden je Blitz-Fenster
_SHOOT = 2.6           # Sekunden je Sternschnuppen-Fenster

# Himmel je Welt: Verlauf ((‰ von oben, Farbe), ...)
_SKY = (
    ((0, (52, 128, 222)), (560, (110, 180, 242)), (1000, (196, 228, 250))),      # Wiese
    ((0, (28, 88, 196)), (600, (84, 150, 232)), (1000, (162, 206, 244))),        # Wolken
    ((0, (20, 22, 36)), (520, (42, 46, 64)), (1000, (70, 76, 94))),              # Gewitter
    ((0, (4, 5, 20)), (450, (10, 14, 46)), (850, (17, 19, 60)), (1000, (28, 22, 70))),  # Nordlicht
    ((0, (0, 0, 3)), (700, (2, 3, 12)), (1000, (6, 8, 26))),                     # Weltall
    ((0, (0, 0, 2)), (1000, (8, 8, 18))),                                        # Mond
)

# Wolken-Ebenen je (Welt, Tiefe): (Anzahl bei w = h, Breite ‰ von h min/max,
# Höhe % der Breite, Palette (Licht, Körper, Schatten), Deckkraft, nur Seiten)
_CLOUDS = {
    (0, 1): (3, 200, 320, 44, ((255, 255, 255), (236, 243, 252), (188, 207, 234)), 225, False),
    (0, 2): (2, 340, 480, 42, ((255, 255, 255), (240, 246, 254), (180, 201, 232)), 255, True),
    (1, 0): (9, 110, 210, 44, ((234, 243, 254), (210, 227, 248), (174, 199, 236)), 205, False),
    (1, 1): (5, 240, 380, 44, ((255, 255, 255), (230, 240, 252), (170, 195, 230)), 240, False),
    (1, 2): (2, 520, 760, 42, ((255, 255, 255), (238, 245, 254), (164, 189, 226)), 255, True),
    (2, 0): (4, 460, 760, 34, ((86, 92, 116), (62, 66, 88), (44, 46, 64)), 225, False),
    (2, 1): (3, 340, 540, 38, ((98, 104, 130), (54, 58, 78), (34, 36, 50)), 240, False),
    (2, 2): (2, 600, 900, 36, ((74, 78, 102), (40, 42, 58), (24, 25, 36)), 250, True),
}

# Sternen-Ebenen je (Welt, Tiefe): (Pixel je Stern, Deckkraft min/max,
# ‰ Glanzsterne, ‰ Doppelpixel)
_STARS = {
    (3, 0): (700, 30, 140, 0, 40),
    (3, 1): (5200, 90, 230, 60, 200),
    (3, 2): (60000, 200, 255, 1000, 0),
    (4, 0): (420, 40, 170, 0, 30),
    (4, 1): (3400, 110, 255, 80, 250),
    (4, 2): (24000, 210, 255, 1000, 0),
}
_STAR_COLS = ((238, 242, 255), (255, 238, 216), (206, 222, 255), (255, 250, 244))

# ferne Planeten: (Start-Takt ab Weltall-Beginn, x ‰ von w, Radius ‰ von h, Art)
_PLANETS = ((520, 150, 58, 0), (2500, 860, 32, 1), (4500, 230, 44, 2))


# ============================================================= Cache
def _wkey(world, w, h, *rest):
    """Schlüssel einer Weltfläche (Gruppe = (world, w, h))."""
    return ("w", world, w, h) + rest


def _cached(key, build):
    """Gecachte Fläche zu key.

    Weltflächen (``_wkey``) gehören zur Gruppe (welt, w, h): höchstens
    _GROUPS Gruppen bleiben, eine neue verdrängt die am längsten unbenutzte
    samt allen ihren Flächen. Kleine Teile (alle anderen Schlüssel) werden ab
    _SMALL_MAX Einträgen gemeinsam verworfen."""
    if key[0] == "w":
        g = key[1:4]
        if not _groups or _groups[-1] != g:
            if g in _groups:
                _groups.remove(g)
            else:
                while len(_groups) >= _GROUPS:
                    old = _groups.pop(0)
                    for k in [k for k in _cache if k[0] == "w" and k[1:4] == old]:
                        del _cache[k]
            _groups.append(g)
    surf = _cache.get(key)
    if surf is None:
        if key[0] != "w":
            small = [k for k in _cache if k[0] != "w"]
            if len(small) >= _SMALL_MAX:
                for k in small:
                    del _cache[k]
        surf = build()
        _cache[key] = surf
    return surf


def clear_cache():
    _cache.clear()
    del _groups[:]


def name_key(world):
    """Text-Schlüssel des Weltnamens (lcx.world_<name>)."""
    return "lcx.world_" + NAMES[_world(world)]


# ============================================================= Helfer
def _world(world):
    """Weltnummer auf 0..5 begrenzt."""
    return max(0, min(5, int(world)))


def _lerp(c1, c2, num, den):
    """Ganzzahlige Farbmischung c1 + (c2 - c1) * num / den (abgerundet)."""
    den = max(1, den)
    return tuple(a + (b - a) * num // den for a, b in zip(c1, c2))


def _hash(i):
    """Ganzzahl-Streuwert 0..2^32-1 (deterministisch, ohne Zufall/sin)."""
    v = (i * 2654435761 + 40503) % 4294967296
    v = ((v ^ (v >> 15)) * 2246822519) % 4294967296
    return v ^ (v >> 13)


def _wrap(v, p):
    """v modulo p im Bereich 0 .. p, auch für negative v (wie Python-%)."""
    return v - p * math.floor(v / p)


def _wave(x, w, sd):
    """Weiche Wellenlinie -1..1 über die Breite w (Hügel, Kanten)."""
    a = math.tau * x / max(1, w)
    return (0.55 * math.sin(a * 2 + sd[0]) + 0.30 * math.sin(a * 5 + sd[1])
            + 0.15 * math.sin(a * 11 + sd[2]))


def _grad(s, y0, hh, w, stops):
    """Senkrechter Verlauf über hh Zeilen ab y0, Breite w; stops = ((‰, Farbe),
    ...) mit RGB- oder RGBA-Farben (ganzzahlig gemischt)."""
    den = max(1, hh - 1)
    k = 0
    for y in range(hh):
        p = y * 1000 // den
        while k < len(stops) - 2 and p > stops[k + 1][0]:
            k += 1
        p0, c0 = stops[k]
        p1, c1 = stops[k + 1]
        s.fill(_lerp(c0, c1, p - p0, p1 - p0), (0, y0 + y, w, 1))


def _glow_new(r, col, amax):
    """Weicher Lichthof 2r x 2r (SRCALPHA, nicht gecacht): Deckkraft amax in der
    Mitte, nach außen quadratisch auf 0 - klein gezeichnet, weich vergrößert."""
    r = max(1, int(r))
    n = max(4, min(40, r // 2))
    s = pygame.Surface((2 * n, 2 * n), pygame.SRCALPHA)
    for i in range(n, 0, -1):
        q = n - i + 1
        pygame.draw.circle(s, (col[0], col[1], col[2], amax * q * q // (n * n)), (n, n), i)
    return pygame.transform.smoothscale(s, (2 * r, 2 * r))


def _disc(r, col):
    """Glatte Scheibe (2r+2)² (4-fach überabgetastet), Mitte (r+1, r+1)."""
    r = max(1, int(r))

    def build():
        ss = 4
        n = (2 * r + 2) * ss
        s = pygame.Surface((n, n), pygame.SRCALPHA)
        pygame.draw.circle(s, col, (n // 2, n // 2), r * ss)
        return pygame.transform.smoothscale(s, (2 * r + 2, 2 * r + 2))
    return _cached(("disc", r, col), build)


def _sparkle(r, col):
    """Glanzstern (2r+1)²: weicher Kern mit vier spitzen Strahlen."""
    r = max(1, int(r))

    def build():
        ss = 4
        n = (2 * r + 1) * ss
        c = n // 2
        s = pygame.Surface((n, n), pygame.SRCALPHA)
        g = _glow_new(max(2, r * ss * 3 // 4), col, 110)
        s.blit(g, (c - g.get_width() // 2, c - g.get_height() // 2))
        R, th = r * ss, max(1, ss * 3 // 4)
        pygame.draw.polygon(s, (col[0], col[1], col[2], 190), [(c - R, c), (c, c - th), (c + R, c), (c, c + th)])
        pygame.draw.polygon(s, (col[0], col[1], col[2], 190), [(c, c - R), (c + th, c), (c, c + R), (c - th, c)])
        pygame.draw.circle(s, (255, 255, 255, 255), (c, c), max(2, ss))
        return pygame.transform.smoothscale(s, (2 * r + 1, 2 * r + 1))
    return _cached(("spark", r, col), build)


def _light(r, col):
    """Positionslicht: Lichthof (2r)² mit hellem Kern."""
    r = max(2, int(r))

    def build():
        s = _glow_new(r, col, 150)
        core = _disc(max(1, r // 4), (min(255, col[0] + 60), min(255, col[1] + 60), min(255, col[2] + 60)))
        s.blit(core, (r - core.get_width() // 2, r - core.get_height() // 2))
        return s
    return _cached(("light", r, col), build)


def _blit_a(s, img, pos, a):
    """Blit mit Deckkraft a (0..255) - setzt die Flächen-Deckkraft zurück."""
    if a <= 0:
        return
    if a < 255:
        img.set_alpha(a)
        s.blit(img, pos)
        img.set_alpha(255)
    else:
        s.blit(img, pos)


def _wblit(dst, img, x, y):
    """Blit mit senkrechtem Umlauf (für kachelbare Ebenen)."""
    h = dst.get_height()
    dst.blit(img, (x, y))
    if y < 0:
        dst.blit(img, (x, y + h))
    if y + img.get_height() > h:
        dst.blit(img, (x, y - h))


def _mask(size, c, r):
    """Weiße Kreis-Maske size² (zum Ausstanzen mit BLEND_RGBA_MIN)."""
    m = pygame.Surface((size, size), pygame.SRCALPHA)
    pygame.draw.circle(m, (255, 255, 255, 255), c, r)
    return m


def _shade(size, lx, ly, rad, col, amax, lit=0, steps=40):
    """Kugel-Schatten size²: bis lit ‰ von rad um den Lichtpunkt (lx, ly) hell,
    dann quadratisch dunkler bis amax bei rad und darüber hinaus."""
    sh = pygame.Surface((size, size), pygame.SRCALPHA)
    sh.fill((col[0], col[1], col[2], amax))
    i0 = steps * lit // 1000
    for i in range(steps, 0, -1):
        q = max(0, i - i0)
        pygame.draw.circle(sh, (col[0], col[1], col[2], amax * q * q // ((steps - i0) * (steps - i0))),
                           (lx, ly), max(1, rad * i // steps))
    return sh


# ============================================================= Steigen
def rise_px(alt, h):
    """Steighöhe in Pixeln zur Höhe alt (gebrochener Takt): sanftes Anfahren
    (Tempo wächst in den ersten _EASE Takten linear von 0), danach
    h * _RISE / 10000 Pixel je Takt."""
    a = max(0.0, float(alt))
    d = a * a / (2 * _EASE) if a < _EASE else a - _EASE / 2
    return d * h * _RISE / 10000


def ground_h(h):
    """Höhe des Bodenstreifens (_GROUND % der Bühnenhöhe)."""
    return max(8, h * _GROUND // 100)


def _pad_top(h):
    """Deck-Oberkante (Korbboden) im Bodenstreifen, von seiner Oberkante aus:
    bei Höhe 0 liegt sie auf rect.bottom - max(16, h * 5 // 100) - dort setzt
    das Spiel den Korb ab."""
    gh = ground_h(h)
    return max(gh // 3, gh - max(16, h * 5 // 100))


# ============================================================= Himmel
def _sun(s, w, h, world):
    """Sonne mit weichem Lichthof oben links (Wiese, Wolken)."""
    cx, cy = w * 16 // 100, h * 17 // 100
    rg = max(8, h * 36 // 100)
    halo = _glow_new(rg, (255, 244, 212), 120 if world == 0 else 100)
    s.blit(halo, (cx - rg, cy - rg))
    r = max(3, h * 40 // 1000)
    inner = _glow_new(r * 3, (255, 250, 232), 210)
    s.blit(inner, (cx - r * 3, cy - r * 3))
    d = _disc(r, (255, 252, 238))
    s.blit(d, (cx - r - 1, cy - r - 1))


def sky(world, w, h):
    """Himmelsverlauf der Welt (opak), Wiese/Wolken mit Sonne."""
    world, w, h = _world(world), max(1, int(w)), max(1, int(h))

    def build():
        s = pygame.Surface((w, h))
        _grad(s, 0, h, w, _SKY[world])
        if world <= 1:
            _sun(s, w, h, world)
        return s
    return _cached(_wkey(world, w, h, "sky"), build)


# ============================================================= Wolken
def _cloud(cw, ch, seed, pal, storm=False):
    """Haufenwolke cw x ch (SRCALPHA, 2-fach überabgetastet): Puffs auf flacher
    Unterkante. Gezeichnet wird Schatten (ganzer Umriss), dann Licht und Körper
    leicht versetzt darüber - so bleibt oben links (Gewitter: oben) ein heller
    Saum und rechts/unten ein Schattenrand. pal = (Licht, Körper, Schatten);
    storm = mehr, flachere Puffs mit Türmen in der Mitte."""
    ss = 2
    W, H = max(4, cw * ss), max(4, ch * ss)
    big = pygame.Surface((W, H), pygame.SRCALPHA)
    light, body, shade = pal
    base = H * 82 // 100
    n = (7 + _hash(seed) % 5) if storm else (4 + _hash(seed) % 4)
    puffs = []
    for k in range(n):
        v = _hash(seed * 16 + k + 1)
        u = (2 * k + 1) * 1000 // (2 * n) + v % 80 - 40
        bell = max(0, 1000 - abs(u - 500) * 2)
        if storm:
            r = max(2, H * (150 + bell * 170 // 1000 + (v >> 9) % 70) // 1000)
        else:
            r = max(2, H * (170 + bell * 250 // 1000 + (v >> 9) % 60) // 1000)
        x = W * 14 // 100 + (W * 72 // 100) * u // 1000
        puffs.append((x, base - r * 45 // 100, r))
    if storm:
        for j in range(2 + _hash(seed + 5) % 2):
            v = _hash(seed * 16 + 40 + j)
            r = max(2, H * (200 + (v >> 9) % 80) // 1000)
            puffs.append((W * (32 + (v % 36)) // 100, max(r + 1, base - H * (40 + (v >> 20) % 12) // 100), r))
    lx, ly = (0, -20) if storm else (-10, -17)
    for x, y, r in puffs:
        pygame.draw.circle(big, shade, (x, y), r)
    x0, x1 = puffs[0][0], puffs[n - 1][0]
    rb = max(2, min(puffs[0][2], puffs[n - 1][2]))
    pygame.draw.rect(big, shade, (x0 - rb // 2, base - rb, x1 - x0 + rb, rb), border_radius=rb // 2)
    for x, y, r in puffs:
        pygame.draw.circle(big, light, (x + r * lx // 100, y + r * ly // 100), r * 84 // 100)
    for x, y, r in puffs:
        pygame.draw.circle(big, body, (x + r * (lx + 8) // 100, y + r * (ly + 9) // 100), r * 80 // 100)
    big.fill((0, 0, 0, 0), (0, base, W, H - base))
    return pygame.transform.smoothscale(big, (max(1, cw), max(1, ch)))


def _rain_shaft(sw, sh, seed, col):
    """Regenvorhang unter einer Gewitterwolke: schräge Schlieren, nach unten
    auslaufend (SRCALPHA sw x sh)."""
    s = pygame.Surface((max(2, sw), max(2, sh)), pygame.SRCALPHA)
    for k in range(max(2, sw // 3)):
        v = _hash(seed * 64 + k)
        x = sw * k // max(1, sw // 3) + v % 3
        pygame.draw.line(s, (col[0], col[1], col[2], 40 + (v >> 8) % 90), (x, 0), (x - sh * 18 // 100, sh),
                         1 + (v >> 16) % 2)
    fade = pygame.Surface((1, max(2, sh)), pygame.SRCALPHA)
    _grad(fade, 0, max(2, sh), 1,
          ((0, (255, 255, 255, 255)), (500, (255, 255, 255, 150)), (1000, (255, 255, 255, 0))))
    s.blit(pygame.transform.scale(fade, s.get_size()), (0, 0), special_flags=pygame.BLEND_RGBA_MULT)
    return s


def _stratus_on(s, seed, w, h, col, cnt, alpha):
    """Schichtwolken-Bänder quer über die ganze Breite (Gewitter, fern)."""
    n = max(1, cnt)
    for k in range(n):
        v = _hash(seed + 71 * k + 3)
        bh = max(4, h * (60 + v % 50) // 1000)
        yc = k * h // n + ((v >> 8) % 1000) * h // (n * 1000)
        band = pygame.Surface((w + bh * 4, bh * 3), pygame.SRCALPHA)
        m = max(4, w // max(1, bh * 2))
        for j in range(m + 3):
            u = _hash(seed + 900 + k * 97 + j)
            ew = bh * (30 + u % 30) // 10
            eh = bh * (12 + (u >> 8) % 12) // 10
            ex = (w + bh * 4) * j // (m + 2) - ew // 2
            pygame.draw.ellipse(band, col, (ex, bh * 3 // 2 - eh // 2 + ((u >> 16) % 5 - 2) * bh // 8, ew, eh))
        band.set_alpha(alpha)
        _wblit(s, band, -bh * 2, yc - bh * 3 // 2)


def _wisp(ww, wh, seed, col):
    """Federwolke ww x wh: dünne, schräg versetzte Schlieren (2-fach überabgetastet)."""
    ss = 2
    W, H = max(4, ww * ss), max(4, wh * ss)
    big = pygame.Surface((W, H), pygame.SRCALPHA)
    n = 5 + _hash(seed) % 3
    for k in range(n):
        v = _hash(seed * 8 + k + 1)
        lw = W * (34 + v % 30) // 100
        lh = max(ss, H * (16 + (v >> 8) % 14) // 100)
        x = (W - lw) * k // (n - 1)
        y = (H - lh) * k // (n - 1) + (v >> 16) % max(1, H // 6) - H // 12
        y = max(0, min(H - lh, y))
        a = 110 + (v >> 20) % 110
        pygame.draw.ellipse(big, (col[0], col[1], col[2], a // 2), (x, y, lw, lh))
        pygame.draw.ellipse(big, (col[0], col[1], col[2], a),
                            (x + lw // 6, y + lh // 4, lw * 2 // 3, max(1, lh // 2)))
    return pygame.transform.smoothscale(big, (max(1, ww), max(1, wh)))


def _clouds_on(s, seed, w, h, spec, storm):
    """Wolken einer Ebene: je eine in gleich hohen Zeilen, waagerecht nach dem
    goldenen Schnitt gestreut (bzw. abwechselnd an den Seiten); Gewitterwolken
    meist mit Regenvorhang."""
    cnt, wmin, wmax, hr, pal, alpha, sides = spec
    n = max(1, (cnt * w + h // 2) // h)
    for k in range(n):
        v = _hash(seed + 31 * k)
        cw = max(8, h * (wmin + v % (wmax - wmin + 1)) // 1000)
        ch = max(4, cw * hr // 100)
        y = k * h // n + ((v >> 10) % 1000) * h // (n * 1000)
        if sides:
            fx = (v >> 20) % 300 - 100
            if k % 2:
                fx = 1000 - fx
        else:
            fx = (k * 618 + (v >> 20) % 200) % 1000 * 12 // 10 - 100
        img = _cloud(cw, ch, v % 100000, pal, storm)
        img.set_alpha(alpha)
        x = w * fx // 1000 - cw // 2
        if storm and (v >> 28) % 3 != 0:
            # Regenvorhang unter der Wolke (hängt mit ihr in der Ebene)
            sw, sh = cw * 55 // 100, ch * 16 // 10
            shaft = _rain_shaft(sw, sh, v % 100000, pal[0])
            _wblit(s, shaft, x + cw * 26 // 100, y - ch // 2 + ch * 74 // 100)
        _wblit(s, img, x, y - ch // 2)


def _wisps_on(s, seed, w, h, col, cnt, alpha):
    """Federwolken-Schlieren einer Ebene."""
    n = max(1, (cnt * w + h // 2) // h)
    for k in range(n):
        v = _hash(seed + 53 * k + 7)
        ww = max(12, h * (420 + v % 360) // 1000)
        wh = max(4, ww * (7 + (v >> 10) % 5) // 100)
        y = k * h // n + ((v >> 14) % 1000) * h // (n * 1000)
        x = w * ((k * 618 + (v >> 24) % 150) % 1000) // 1000
        img = _wisp(ww, wh, v % 100000, col)
        img.set_alpha(alpha)
        _wblit(s, img, x - ww // 2, y - wh // 2)


# ============================================================= Sterne
def _stars_on(s, seed, n, w, h, amin, amax, big, two):
    """n Sterne auf die Ebene s (Deckkraft amin..amax); big ‰ als Glanzstern,
    two ‰ als Doppelpixel."""
    rs = max(2, h // 190)
    for i in range(n):
        v = _hash(seed + 2 * i)
        u = _hash(seed + 2 * i + 1)
        x, y = v % w, u % h
        col = _STAR_COLS[(v >> 20) % 4]
        a = amin + (u >> 12) % (amax - amin + 1)
        p = (u >> 22) % 1000
        if p < big:
            spr = _sparkle(rs + (v >> 26) % 2, col)
            spr.set_alpha(a)
            _wblit(s, spr, x - spr.get_width() // 2, y - spr.get_height() // 2)
            spr.set_alpha(255)
        elif p < big + two:
            s.fill((col[0], col[1], col[2], a * 3 // 4), (x, y, 2, 2))
        else:
            s.fill((col[0], col[1], col[2], a), (x, y, 1, 1))


def _milky(s, seed, w, h):
    """Milchstraße (schräges Leuchtband mit Sternenstaub) und zwei Nebel - ohne
    den oberen/unteren Rand zu berühren (Ebene bleibt kachelbar)."""
    q = 4
    lo = pygame.Surface((w // q + 2, h // q + 2), pygame.SRCALPHA)
    y0, y1, th = h * 62 // 100, h * 36 // 100, h * 10 // 100
    n = max(6, w // 36)
    cols = ((150, 140, 230), (116, 158, 232), (190, 150, 222))
    for k in range(n):
        v = _hash(seed + 7 * k)
        x = w * k // (n - 1) + ((v % 41) - 20) * w // 1000
        yc = y0 + (y1 - y0) * x // w + ((v >> 8) % 61 - 30) * th // 100
        r = max(2, th * (60, 80, 100, 120)[(v >> 16) % 4] // 100 // q)
        g = _glow_new(r, cols[(v >> 24) % 3], 22 + (v >> 4) % 14)
        lo.blit(g, (x // q - r, yc // q - r))
    for j, (cx, cy, col, col2) in enumerate(((w * 22 // 100, h * 25 // 100, (200, 90, 190), (120, 90, 220)),
                                             (w * 80 // 100, h * 72 // 100, (70, 160, 210), (90, 120, 220)))):
        for k in range(16):
            v = _hash(seed + 900 + j * 50 + k)
            r = max(2, h * (60 + (v % 60)) // 1000 // q)
            dx = ((v >> 8) % 240 - 120) * h // 1000 // q
            dy = ((v >> 16) % 120 - 60) * h // 1000 // q
            g = _glow_new(r, col if k % 3 else col2, 9 + (v >> 24) % 8)
            lo.blit(g, (cx // q + dx - r, cy // q + dy - r))
    s.blit(pygame.transform.smoothscale(lo, ((w // q + 2) * q, (h // q + 2) * q)), (0, 0))
    m = w * th // 70
    for i in range(m):
        v = _hash(seed + 5000 + 2 * i)
        u = _hash(seed + 5001 + 2 * i)
        x = v % w
        d = ((u % 1000) + ((u >> 10) % 1000) - 1000) * th // 1000
        y = y0 + (y1 - y0) * x // w + d
        s.fill((226, 230, 255, 40 + (u >> 20) % 130), (x, y, 1, 1))


# ============================================================= Ebenen
def _build_layer(world, depth, w, h):
    s = pygame.Surface((w, h), pygame.SRCALPHA)
    seed = 1000 * (world * 3 + depth) + 17
    if world == 2 and depth == 0:
        _stratus_on(s, seed + 300, w, h, (50, 54, 74), 3, 150)
    spec = _CLOUDS.get((world, depth))
    if spec:
        _clouds_on(s, seed, w, h, spec, world == 2)
    if world == 0 and depth == 0:
        _wisps_on(s, seed, w, h, (255, 255, 255), 3, 150)
    if world == 4 and depth == 0:
        _milky(s, seed + 400, w, h)
    if world == 3 and depth == 2:
        _wisps_on(s, seed, w, h, (120, 150, 210), 2, 56)
    st = _STARS.get((world, depth))
    if st:
        per, amin, amax, big, two = st
        _stars_on(s, seed + 100, max(1, w * h // per), w, h, amin, amax, big, two)
    return s


def layer(world, depth, w, h):
    """Kachelbare Ebene der Welt (depth 0 fern .. 2 nah), w x h, SRCALPHA:
    Wiese Federwolken + kleine Haufenwolken, Wolken drei Wolkentiefen, Gewitter
    dunkle Wolkenmassen, Nordlicht Sterne + Schlieren, Weltall Sterne mit
    Milchstraße und Nebeln; Mond leer."""
    world, w, h = _world(world), max(1, int(w)), max(1, int(h))
    depth = max(0, min(2, int(depth)))
    return _cached(_wkey(world, w, h, "layer", depth), lambda: _build_layer(world, depth, w, h))


def _tile(s, rect, img, off):
    """Kachelbare Ebene senkrecht um off nach unten verschoben."""
    lh = img.get_height()
    y = int(off) % lh
    if y:
        s.blit(img, (rect.x, rect.y), (0, lh - y, rect.w, y))
    s.blit(img, (rect.x, rect.y + y), (0, 0, rect.w, lh - y))


# ============================================================= Boden
def _barn(s, x0, yb, bw, ss):
    """Lama-Stall: rote Scheune von vorn (Tor mit X, Heuluke), Seitenwand rechts."""
    bh = bw * 50 // 100
    rf = bw * 42 // 100
    yw, yk, yr = yb - bh, yb - bh - rf * 55 // 100, yb - bh - rf
    d, dy = bw * 20 // 100, bw * 6 // 100
    red, red_d, red_s = (190, 60, 50), (158, 46, 40), (138, 40, 36)
    trim, roof, roof_d = (246, 242, 232), (96, 70, 64), (70, 50, 48)
    lw = max(1, bw // 55)
    xk0, xm, xk1 = x0 + bw * 14 // 100, x0 + bw // 2, x0 + bw * 86 // 100

    def gable(x):
        if x <= xk0:
            return yw + (yk - yw) * (x - x0) // max(1, xk0 - x0)
        if x <= xm:
            return yk + (yr - yk) * (x - xk0) // max(1, xm - xk0)
        if x <= xk1:
            return yr + (yk - yr) * (x - xm) // max(1, xk1 - xm)
        return yk + (yw - yk) * (x - xk1) // max(1, x0 + bw - xk1)

    pygame.draw.ellipse(s, (40, 96, 40, 90), (x0 - bw // 10, yb - bw // 16, bw * 13 // 10 + d, bw // 8))
    # Seitenwand und Dach nach hinten rechts
    pygame.draw.polygon(s, red_s, [(x0 + bw, yb), (x0 + bw, yw), (x0 + bw + d, yw - dy), (x0 + bw + d, yb - dy)])
    pygame.draw.polygon(s, roof_d, [(xk1, yk), (x0 + bw, yw), (x0 + bw + d, yw - dy), (xk1 + d, yk - dy)])
    pygame.draw.polygon(s, roof, [(xm, yr), (xk1, yk), (xk1 + d, yk - dy), (xm + d, yr - dy)])
    wx, wy, ws = x0 + bw + d * 3 // 10, yw + bh * 3 // 10, max(2, d * 4 // 10)
    pygame.draw.rect(s, (60, 36, 32), (wx, wy - dy // 2, ws, ws))
    pygame.draw.rect(s, trim, (wx, wy - dy // 2, ws, ws), lw)
    # Giebelwand
    pygame.draw.polygon(s, red, [(x0, yb), (x0, yw), (xk0, yk), (xm, yr), (xk1, yk), (x0 + bw, yw), (x0 + bw, yb)])
    for k in range(1, 12):
        x = x0 + bw * k // 12
        pygame.draw.line(s, red_d, (x, gable(x) + lw * 2), (x, yb), lw)
    # Dachkante + Zierleiste, Ecken
    edge = [(x0 - bw * 5 // 100, yw + lw * 3), (xk0 - lw, yk - lw * 2), (xm, yr - lw * 3),
            (xk1 + lw, yk - lw * 2), (x0 + bw * 105 // 100, yw + lw * 3)]
    pygame.draw.lines(s, roof, False, edge, max(2, bw // 14))
    pygame.draw.lines(s, trim, False, [(x0, yw), (xk0, yk), (xm, yr), (xk1, yk), (x0 + bw, yw)], lw * 2)
    pygame.draw.line(s, trim, (x0 + lw, yw), (x0 + lw, yb), lw * 2)
    pygame.draw.line(s, trim, (x0 + bw - lw, yw), (x0 + bw - lw, yb), lw * 2)
    # Tor mit weißem X
    dw, dh = bw * 40 // 100, bh * 72 // 100
    dx = x0 + (bw - dw) // 2
    pygame.draw.rect(s, red_d, (dx, yb - dh, dw, dh))
    for a, b in (((dx, yb - dh), (dx + dw // 2, yb)), ((dx + dw // 2, yb - dh), (dx, yb)),
                 ((dx + dw // 2, yb - dh), (dx + dw, yb)), ((dx + dw, yb - dh), (dx + dw // 2, yb))):
        pygame.draw.line(s, trim, a, b, lw * 2)
    pygame.draw.line(s, trim, (dx + dw // 2, yb - dh), (dx + dw // 2, yb), lw * 2)
    pygame.draw.rect(s, trim, (dx, yb - dh, dw, dh), lw * 2)
    # Heuluke mit Heu
    hs = bw * 22 // 100
    hx, hy = x0 + (bw - hs) // 2, yw - rf * 40 // 100
    pygame.draw.rect(s, (66, 38, 30), (hx, hy, hs, hs))
    pygame.draw.polygon(s, (236, 200, 92), [(hx + lw, hy + hs), (hx + hs // 5, hy + hs * 55 // 100),
                                            (hx + hs * 2 // 5, hy + hs * 75 // 100),
                                            (hx + hs * 3 // 5, hy + hs * 50 // 100),
                                            (hx + hs * 4 // 5, hy + hs * 72 // 100), (hx + hs - lw, hy + hs)])
    pygame.draw.rect(s, trim, (hx, hy, hs, hs), lw * 2)
    # Wetterfahne
    pygame.draw.line(s, (70, 64, 70), (xm, yr - lw * 3), (xm, yr - rf * 40 // 100), lw)
    pygame.draw.polygon(s, (70, 64, 70), [(xm - bw // 14, yr - rf * 34 // 100),
                                          (xm + bw // 10, yr - rf * 36 // 100),
                                          (xm + bw // 14, yr - rf * 30 // 100)])


def _bale(s, cx, cy, r, ss):
    """Rundballen Heu (Stirnseite mit Spirale)."""
    pygame.draw.ellipse(s, (40, 96, 40, 90), (cx - r * 11 // 10, cy + r * 6 // 10, r * 22 // 10, r // 2))
    pygame.draw.circle(s, (204, 164, 74), (cx, cy), r)
    pygame.draw.circle(s, (234, 200, 106), (cx - r // 10, cy - r // 10), r * 86 // 100)
    for k in (64, 42, 20):
        pygame.draw.circle(s, (198, 158, 70), (cx - r // 10, cy - r // 10), max(1, r * k // 100), max(1, ss))


def _tree(s, x, yb, size, ss):
    """Laubbaum: Stamm und runde Krone (Schatten, Körper, Licht)."""
    tw = max(ss, size // 9)
    pygame.draw.ellipse(s, (40, 96, 40, 90), (x - size // 2, yb - size // 18, size, size // 9))
    pygame.draw.rect(s, (112, 80, 54), (x - tw // 2, yb - size * 55 // 100, tw, size * 55 // 100))
    shade, body, light = (52, 118, 60), (72, 148, 70), (112, 182, 92)
    for dx, dy, r, col in ((-20, -60, 28, shade), (20, -58, 28, shade), (0, -80, 32, shade),
                           (-14, -66, 24, body), (14, -64, 23, body), (-2, -84, 26, body),
                           (-12, -88, 13, light), (-22, -70, 10, light)):
        pygame.draw.circle(s, col, (x + size * dx // 100, yb + size * dy // 100), max(1, size * r // 100))


def _windsock(s, x, yb, hh, ss):
    """Windsack (rot-weiß) am Mast - der Startplatz ist ein Flugplatz."""
    top = yb - hh
    pygame.draw.line(s, (196, 198, 206), (x, yb), (x, top), max(ss, hh // 45))
    ln = hh * 38 // 100
    h0, h1 = hh * 15 // 100, hh * 8 // 100
    for k in range(4):
        xa, xb = x + ln * k // 4, x + ln * (k + 1) // 4
        ha, hb = h0 + (h1 - h0) * k // 4, h0 + (h1 - h0) * (k + 1) // 4
        da, db = hh * 2 * k // 100, hh * 2 * (k + 1) // 100
        col = (232, 52, 76) if k % 2 == 0 else (250, 248, 240)
        pygame.draw.polygon(s, col, [(xa, top + da - ha // 2 + h0 // 2), (xb, top + db - hb // 2 + h0 // 2),
                                     (xb, top + db + hb // 2 + h0 // 2), (xa, top + da + ha // 2 + h0 // 2)])
    pygame.draw.circle(s, (230, 230, 236), (x, top), max(ss, hh // 28))


def _pad(s, cx, yd, pw, G, ss):
    """Startplatz: flaches rundes Holzpodest mit Treppe, rotem Startkreis,
    Wimpelkette und zwei Fähnchen. Deck-Mitte (Korbboden) bei (cx, yd); die
    Seitenfläche reicht bis knapp über den unteren Rand des Streifens."""
    eb = max(2, pw * 11 // 100)
    th = max(ss * 2, G * 97 // 100 - yd - eb)
    yg = yd + th + eb
    wl, wm, wd = (220, 178, 120), (174, 124, 74), (136, 94, 56)
    lw = max(1, ss)
    pygame.draw.ellipse(s, (36, 86, 36, 110),
                        (cx - pw * 64 // 100, yd + th - eb // 2, pw * 132 // 100, eb * 2 + G * 3 // 100))
    # Treppe rechts (zwei Stufen)
    for k in range(2):
        sx = cx + pw * 38 // 100 + k * pw * 11 // 100
        sy = yd + th * (k + 1) // 3
        pygame.draw.rect(s, wd, (sx, sy, pw * 17 // 100, yg - sy))
        pygame.draw.rect(s, wm, (sx, sy, pw * 15 // 100, yg - sy))
        pygame.draw.rect(s, wl, (sx, sy, pw * 15 // 100, max(ss, G * 15 // 1000)))
    # Zylinder: Unterkante, Seitenfläche mit Brettern, Deck
    pygame.draw.ellipse(s, wd, (cx - pw // 2, yd + th - eb, pw, 2 * eb))
    pygame.draw.rect(s, wm, (cx - pw // 2, yd, pw, th))
    for k in range(1, 8):
        x = cx - pw // 2 + pw * k // 8
        q = (x - cx) * 2 / pw
        e = int(eb * math.sqrt(max(0.0, 1.0 - q * q)))
        pygame.draw.line(s, wd, (x, yd + e), (x, yd + th + e), lw)
    pygame.draw.ellipse(s, wl, (cx - pw // 2, yd - eb, pw, 2 * eb))
    pygame.draw.ellipse(s, wd, (cx - pw // 2, yd - eb, pw, 2 * eb), lw)
    for k in range(1, 5):
        y = yd - eb + 2 * eb * k // 5
        q = (y - yd) / eb
        hw = int(pw / 2 * math.sqrt(max(0.0, 1.0 - q * q)))
        pygame.draw.line(s, (198, 154, 100), (cx - hw + ss * 2, y), (cx + hw - ss * 2, y), lw)
    pygame.draw.ellipse(s, (232, 52, 76),
                        (cx - pw * 30 // 100, yd - eb * 56 // 100, pw * 60 // 100, eb * 112 // 100),
                        max(ss, pw // 36))
    # Wimpelkette an der Vorderkante
    cols = ((232, 52, 76), (250, 206, 70), (250, 248, 240), (70, 140, 220))
    tw = max(ss * 2, pw * 9 // 100)
    for k in range(9):
        x = cx - pw * 44 // 100 + pw * 88 * k // 800
        q = (x - cx) * 2 / pw
        y = yd + int(eb * math.sqrt(max(0.0, 1.0 - q * q))) + lw
        pygame.draw.polygon(s, cols[k % 4], [(x - tw // 2, y), (x + tw // 2, y), (x, y + tw * 11 // 10)])
    # zwei Fähnchen am hinteren Rand
    for fx, col in ((-44, (232, 52, 76)), (44, (250, 206, 70))):
        x = cx + pw * fx // 100
        top = yd - G * 30 // 100
        pygame.draw.line(s, (236, 236, 240), (x, yd - eb // 3), (x, top), max(ss, pw // 50))
        fw = pw * 16 // 100
        pygame.draw.polygon(s, col, [(x, top), (x + fw, top + fw * 3 // 10), (x, top + fw * 6 // 10)])


def _meadow(s, W, G, ss, h):
    """Alles auf dem Bodenstreifen (W x G, ss-fach überabgetastet)."""
    def Y(p):
        return G * p // 1000

    step = max(4, W // 140)
    # Hügel: dunstig fern, grüner davor; Baumgruppen auf dem vorderen Kamm
    for base, amp, col, sd in ((440, 250, (158, 200, 192), (0.6, 1.9, 0.3)),
                               (500, 160, (120, 182, 126), (2.2, 0.4, 1.3))):
        pts = [(0, G)]
        for x in range(0, W + step, step):
            pts.append((x, Y(base) - int(Y(amp) * (0.5 + 0.5 * _wave(x, W, sd)))))
        pts.append((W + step, G))
        pygame.draw.polygon(s, col, pts)
    n_trees = max(4, W // max(1, Y(120)))
    for k in range(n_trees):
        v = _hash(71000 + k)
        x = (k * W) // n_trees + (v % max(1, W // n_trees))
        y = Y(500) - int(Y(160) * (0.5 + 0.5 * _wave(x, W, (2.2, 0.4, 1.3)))) + Y(10)
        for j in range(1 + (v >> 8) % 3):
            r = max(ss, G * (22 + (v >> (12 + j)) % 14) // 1000)
            tx = x + j * r * 15 // 10
            pygame.draw.circle(s, (70, 136, 82), (tx, y - r), r)
            pygame.draw.circle(s, (92, 160, 96), (tx - r // 4, y - r - r // 4), r * 6 // 10)
    # Wiese: Verlauf, oben leicht gewellt, mit hellem Saum
    top = []
    for x in range(0, W + step, step):
        top.append((x, Y(505) - int(Y(26) * (0.5 + 0.5 * _wave(x, W, (1.0, 2.6, 0.2))))))
    grass = pygame.Surface((W, G), pygame.SRCALPHA)
    y0 = Y(470)
    _grad(grass, y0, G - y0, W,
          ((0, (146, 208, 104, 255)), (450, (110, 184, 82, 255)), (1000, (82, 154, 62, 255))))
    mask = pygame.Surface((W, G), pygame.SRCALPHA)
    pygame.draw.polygon(mask, (255, 255, 255, 255), [(0, G)] + top + [(W + step, G)])
    grass.blit(mask, (0, 0), special_flags=pygame.BLEND_RGBA_MIN)
    s.blit(grass, (0, 0))
    pygame.draw.lines(s, (176, 226, 128), False, top, max(1, ss))
    # Grasbüschel und Blumen (vorn größer)
    gy0 = Y(540)
    n = W * (G - gy0) // (ss * ss * 230)
    flowers = ((250, 250, 240), (250, 220, 80), (240, 150, 180), (190, 160, 240))
    for i in range(n):
        v = _hash(72000 + 2 * i)
        u = _hash(72001 + 2 * i)
        x = v % W
        y = gy0 + (u % 1000) * (G - gy0) // 1000
        sz = ss * (2 + (y - gy0) * 5 // max(1, G - gy0))
        if (v >> 20) % 7 == 0:
            pygame.draw.circle(s, flowers[(u >> 12) % 4], (x, y - sz // 2), max(1, sz // 3))
        else:
            col = (78, 146, 58) if (u >> 14) & 1 else (100, 170, 72)
            for dx in (-sz // 2, 0, sz // 2):
                pygame.draw.line(s, col, (x, y), (x + dx, y - sz), max(1, ss // 2))
    pw = h * _PAD_W // 100 * ss
    cx, yd = (W // ss) * _PAD_X // 100 * ss, _pad_top(h) * ss
    free = cx + W // 10                 # bis hier bleibt es um den Startplatz frei
    bw = min(W * 18 // 100, G * 62 // 100)
    bx = W * 71 // 100
    # Trampelpfad vom Startplatz zum Stalltor
    p0, p1 = cx + pw * 60 // 100, bx + bw // 2
    top_pts, bot_pts = [], []
    for k in range(9):
        x = p0 + (p1 - p0) * k // 8
        y = Y(935) - (Y(935) - Y(872)) * k * k // 64
        half = (Y(34) - (Y(34) - Y(14)) * k // 8)
        top_pts.append((x, y - half))
        bot_pts.append((x, y + half))
    pygame.draw.polygon(s, (190, 162, 112), top_pts + bot_pts[::-1])
    pygame.draw.lines(s, (210, 184, 134), False, [(x, y + ss) for x, y in top_pts], max(1, ss))
    # Zaun quer über die Weide, erst rechts vom Startplatz
    rh = max(ss, Y(20))
    for yr in (Y(588), Y(650)):
        s.fill((190, 142, 92), (free, yr, W - free, rh))
        s.fill((140, 100, 62), (free, yr + rh, W - free, max(1, ss)))
    sp = max(8, Y(150))
    pwid = max(ss, Y(26))
    for x in range(free, W, sp):
        pygame.draw.rect(s, (160, 114, 72), (x, Y(552), pwid, Y(702) - Y(552)))
        pygame.draw.rect(s, (122, 86, 54), (x + pwid * 6 // 10, Y(552), pwid - pwid * 6 // 10, Y(702) - Y(552)))
        pygame.draw.polygon(s, (176, 128, 82), [(x, Y(552)), (x + pwid // 2, Y(534)), (x + pwid, Y(552))])
    # Büsche, kleiner Baum in der Mitte, Windsack
    for fx, fy, fr in ((47, 880, 44), (51, 905, 32), (89, 935, 40), (64, 915, 28)):
        ux, uy, ur = W * fx // 100, Y(fy), max(ss, G * fr // 1000)
        pygame.draw.circle(s, (58, 124, 62), (ux, uy - ur), ur)
        pygame.draw.circle(s, (84, 156, 74), (ux - ur // 4, uy - ur - ur // 4), ur * 7 // 10)
    _tree(s, W * 55 // 100, Y(810), G * 40 // 100, ss)
    _windsock(s, W * 31 // 100, Y(900), G * 50 // 100, ss)
    # Stall rechts (71 .. 90 %), Heuballen davor, großer Baum ganz rechts
    r = max(ss * 2, G * 6 // 100)
    _bale(s, bx - r * 16 // 10, Y(900), r, ss)
    _bale(s, bx - r * 38 // 10, Y(925), r * 9 // 10, ss)
    _barn(s, bx, Y(875), bw, ss)
    _tree(s, W * 95 // 100, Y(860), G * 60 // 100, ss)
    # Startplatz und Lamas (sehen dem Start zu)
    _pad(s, cx, yd, pw, G, ss)
    n = max(8, G * 24 // 100)
    for fx, fy, k, scarf, flip in ((36, 925, 100, (60, 150, 220), True),
                                   (43, 960, 86, (250, 196, 60), True),
                                   (60, 945, 92, (150, 90, 200), False)):
        m = max(8, n * k // 100)
        spr = LCD.llama_sprite(m, "stand", scarf)
        if flip:
            spr = pygame.transform.flip(spr, True, False)
        x, y = W * fx // 100, Y(fy)
        pygame.draw.ellipse(s, (36, 86, 36, 100), (x - m * 3 // 10, y - m // 16, m * 6 // 10, m // 8))
        s.blit(spr, (x - m // 2, y - m * 94 // 100))


def ground(w, h):
    """Bodenstreifen w x ground_h(h) (SRCALPHA, 2-fach überabgetastet): Hügel mit
    Bäumen, Weide mit Blumen, Zaun, Lama-Stall, Heuballen, Windsack, drei Lamas
    und links der Startplatz (Mitte x = w * _PAD_X // 100, Deck-Oberkante bei
    _pad_top(h) im Streifen)."""
    w, h = max(1, int(w)), max(1, int(h))

    def build():
        gh = ground_h(h)
        ss = 2
        big = pygame.Surface((w * ss, gh * ss), pygame.SRCALPHA)
        _meadow(big, w * ss, gh * ss, ss, h)
        return pygame.transform.smoothscale(big, (w, gh))
    return _cached(_wkey(0, w, h, "ground"), build)


# ============================================================= kleine Figuren
def _bird(n, frame, col, flip):
    """Vogel n x n (Flügelschlag frame 0..3), Blick nach rechts (flip = links)."""
    frame %= 4

    def build():
        ss = 4
        N = n * ss
        s = pygame.Surface((N, N), pygame.SRCALPHA)
        tip = (0.10, 0.40, 0.74, 0.40)[frame]
        elb = (0.30, 0.44, 0.60, 0.46)[frame]
        cx, cy = N * 0.52, N * 0.54
        lw = max(ss, N // 9)
        pygame.draw.lines(s, col, False, [(N * 0.03, N * tip), (N * 0.28, N * elb), (cx, cy)], lw)
        pygame.draw.lines(s, col, False, [(cx, cy), (N * 0.76, N * elb), (N * 0.97, N * tip)], lw)
        pygame.draw.ellipse(s, col, (N * 0.38, N * 0.48, N * 0.30, N * 0.13))
        pygame.draw.circle(s, col, (N * 0.70, N * 0.52), N * 0.055)
        img = pygame.transform.smoothscale(s, (n, n))
        return pygame.transform.flip(img, True, False) if flip else img
    return _cached(("bird", n, frame, col, flip), build)


def _plane(n, flip):
    """Verkehrsflugzeug von der Seite, n lang (Blick nach rechts; flip = links),
    gedämpft wie im Gewitterlicht, die Fensterreihe leuchtet."""
    def build():
        ss = 3
        N = n * ss
        H = N * 38 // 100
        s = pygame.Surface((N, H), pygame.SRCALPHA)
        body, belly, dark = (170, 176, 194), (126, 132, 152), (84, 90, 108)
        fy, fh = H * 44 // 100, H * 26 // 100
        pygame.draw.polygon(s, dark, [(N * 1 // 100, H * 52 // 100), (N * 16 // 100, H * 52 // 100),
                                      (N * 19 // 100, H * 60 // 100), (0, H * 62 // 100)])
        pygame.draw.polygon(s, (184, 56, 74), [(N * 4 // 100, H * 52 // 100), (0, H * 4 // 100),
                                               (N * 9 // 100, H * 4 // 100), (N * 25 // 100, H * 50 // 100)])
        pygame.draw.rect(s, body, (N * 4 // 100, fy, N * 86 // 100, fh), border_radius=fh // 2)
        pygame.draw.ellipse(s, body, (N * 80 // 100, fy, N * 19 // 100, fh))
        pygame.draw.rect(s, belly, (N * 10 // 100, fy + fh * 62 // 100, N * 82 // 100, fh * 38 // 100),
                         border_radius=fh // 3)
        pygame.draw.polygon(s, (40, 52, 82), [(N * 90 // 100, fy + fh * 22 // 100),
                                              (N * 95 // 100, fy + fh * 30 // 100),
                                              (N * 95 // 100, fy + fh * 44 // 100),
                                              (N * 89 // 100, fy + fh * 44 // 100)])
        for k in range(17):
            pygame.draw.circle(s, (255, 226, 150), (N * (26 + k * 37 // 10) // 100, fy + fh * 38 // 100),
                               max(1, ss * 2 // 3))
        pygame.draw.polygon(s, dark, [(N * 40 // 100, fy + fh * 70 // 100), (N * 58 // 100, fy + fh * 70 // 100),
                                      (N * 44 // 100, H - 1), (N * 36 // 100, H - 1)])
        pygame.draw.ellipse(s, (70, 74, 92), (N * 45 // 100, fy + fh, N * 13 // 100, H * 15 // 100))
        img = pygame.transform.smoothscale(s, (n, max(2, H // ss)))
        return pygame.transform.flip(img, True, False) if flip else img
    return _cached(("plane", n, flip), build)


def _jet(n, flip):
    """Schlanker Jet als dunkle Silhouette (Nachthimmel), n lang, Blick nach rechts."""
    def build():
        ss = 3
        N = n * ss
        H = N * 34 // 100
        s = pygame.Surface((N, H), pygame.SRCALPHA)
        col, top = (92, 102, 136), (176, 196, 224)
        fy, fh = H * 46 // 100, H * 18 // 100
        pygame.draw.polygon(s, col, [(N * 4 // 100, H * 50 // 100), (0, H * 6 // 100),
                                     (N * 10 // 100, H * 6 // 100), (N * 26 // 100, H * 48 // 100)])
        pygame.draw.rect(s, col, (N * 4 // 100, fy, N * 80 // 100, fh), border_radius=fh // 2)
        pygame.draw.polygon(s, col, [(N * 82 // 100, fy), (N, fy + fh * 60 // 100), (N * 82 // 100, fy + fh)])
        pygame.draw.polygon(s, col, [(N * 42 // 100, fy + fh * 60 // 100), (N * 60 // 100, fy + fh * 60 // 100),
                                     (N * 38 // 100, H - 1), (N * 30 // 100, H - 1)])
        pygame.draw.line(s, top, (N * 10 // 100, fy + ss // 2), (N * 84 // 100, fy + ss // 2), max(1, ss // 2))
        img = pygame.transform.smoothscale(s, (n, max(2, H // ss)))
        return pygame.transform.flip(img, True, False) if flip else img
    return _cached(("jet", n, flip), build)


def _contrail(n, hgt, flip):
    """Kondensstreifen n x hgt: am Flugzeug (rechts) schmal und hell, nach links
    breiter und blasser (flip = Flugzeug links)."""
    def build():
        ss = 2
        N, Hh = n * ss, max(2, hgt * ss)
        s = pygame.Surface((N, Hh), pygame.SRCALPHA)
        segs = 48
        for k in range(segs):
            f = k / (segs - 1)
            th = max(1, int(Hh * (1.0 - 0.7 * f)))
            a = int(8 + 190 * f * f)
            x0 = N * k // segs
            s.fill((206, 222, 246, a), (x0, (Hh - th) // 2, N // segs + 1, th))
        img = pygame.transform.smoothscale(s, (n, max(1, hgt)))
        return pygame.transform.flip(img, True, False) if flip else img
    return _cached(("trail", n, hgt, flip), build)


def _satellite(n):
    """Satellit (goldener Körper, zwei Solarflügel, Schüssel), schräg gedreht."""
    def build():
        ss = 3
        N = n * ss
        s = pygame.Surface((N, N), pygame.SRCALPHA)
        c = N // 2
        pw, ph = N * 30 // 100, N * 13 // 100
        bw, bh = N * 18 // 100, N * 22 // 100
        lw = max(1, ss // 2)
        pygame.draw.line(s, (150, 154, 166), (c - bw // 2 - pw // 2, c), (c + bw // 2 + pw // 2, c), max(1, ss))
        for side in (-1, 1):
            x0 = c + bw // 2 + N * 4 // 100 if side > 0 else c - bw // 2 - N * 4 // 100 - pw
            pygame.draw.rect(s, (34, 58, 132), (x0, c - ph // 2, pw, ph))
            for k in range(1, 4):
                xx = x0 + pw * k // 4
                pygame.draw.line(s, (88, 120, 200), (xx, c - ph // 2), (xx, c + ph // 2), lw)
            pygame.draw.line(s, (88, 120, 200), (x0, c), (x0 + pw, c), lw)
            pygame.draw.rect(s, (130, 150, 200), (x0, c - ph // 2, pw, ph), lw)
        pygame.draw.rect(s, (196, 156, 70), (c - bw // 2, c - bh // 2, bw, bh), border_radius=ss)
        pygame.draw.rect(s, (238, 206, 118), (c - bw // 2, c - bh // 2, bw, bh // 2), border_radius=ss)
        pygame.draw.line(s, (200, 202, 210), (c, c - bh // 2), (c, c - bh), lw)
        pygame.draw.ellipse(s, (226, 228, 236), (c - bw // 2, c - bh - bh // 3, bw, bh // 2))
        img = pygame.transform.rotate(s, 24)
        return pygame.transform.smoothscale(img, (max(1, img.get_width() // ss), max(1, img.get_height() // ss)))
    return _cached(("sat", n), build)


def _bolt(w, h, var):
    """Blitz (Variante 0..3): gezackter Hauptast mit zwei Seitenästen, violetter
    Schein um einen weißen Kern (2-fach überabgetastet)."""
    var %= 4

    def build():
        bw, bh = max(8, h * 30 // 100), max(16, h * 55 // 100)
        ss = 2
        BW, BH = bw * ss, bh * ss
        s = pygame.Surface((BW, BH), pygame.SRCALPHA)
        pts = [(BW // 2, 0)]
        x, y, k = BW // 2, 0, 0
        while y < BH * 90 // 100:
            v = _hash(21000 + var * 100 + k)
            y += BH * (5 + v % 6) // 100
            x = max(BW // 6, min(BW * 5 // 6, x + BW * ((v >> 8) % 25 - 12) // 100))
            pts.append((x, y))
            k += 1
        branches = []
        for j, start in enumerate((2, 4)):
            if start >= len(pts):
                continue
            bx, by = pts[start]
            side = 1 if (_hash(21500 + var * 10 + j) & 1) else -1
            br = [(bx, by)]
            for m in range(4):
                v = _hash(21600 + var * 40 + j * 8 + m)
                bx = max(0, min(BW - 1, bx + side * BW * (6 + v % 8) // 100))
                by += BH * (4 + (v >> 8) % 4) // 100
                br.append((bx, by))
            branches.append(br)
        g = max(2, BH // 70)
        for col, wid in (((150, 140, 255, 50), g * 3), ((186, 178, 255, 110), g * 3 // 2),
                         ((255, 255, 255, 255), max(ss, g // 2))):
            pygame.draw.lines(s, col, False, pts, wid)
            for br in branches:
                pygame.draw.lines(s, col, False, br, max(1, wid * 2 // 3))
        return pygame.transform.smoothscale(s, (bw, bh))
    return _cached(_wkey(2, w, h, "bolt", var), build)


def _bolt_glow(w, h):
    """Lichtschein in der Wolke um den Blitz."""
    return _cached(_wkey(2, w, h, "boltglow"), lambda: _glow_new(max(8, h * 34 // 100), (160, 160, 236), 120))


# ============================================================= Polarlicht
def _ribbon_h(h, kind):
    """(Strahlenhöhe, Schein darunter) eines Polarlicht-Vorhangs."""
    return max(8, h * (44 if kind == 0 else 38) // 100), max(4, h * 12 // 100)


def _ribbon(w, h, kind):
    """Polarlicht-Vorhang als Fläche (w + Streifen) x (rh + gh): senkrechte
    Strahlen mit hellem unteren Saum, oben verblassend, darunter ein schwacher
    Schein (kind 0 grün, 1 violett-rosa). Waagerecht periodisch über w, damit
    er beim Zeichnen wandern kann."""
    def build():
        rh, gh = _ribbon_h(h, kind)
        sl = _slice_w(w)
        s = pygame.Surface((w + sl, rh + gh), pygame.SRCALPHA)
        glow = pygame.Surface((1, gh), pygame.SRCALPHA)
        if kind == 0:
            gstops = ((0, (120, 250, 176, 120)), (220, (90, 226, 160, 52)), (1000, (50, 160, 130, 0)))
        else:
            gstops = ((0, (226, 128, 230, 100)), (220, (180, 96, 214, 40)), (1000, (120, 60, 190, 0)))
        _grad(glow, 0, gh, 1, gstops)
        prof = pygame.Surface((1, rh), pygame.SRCALPHA)
        if kind == 0:
            stops = ((0, (170, 80, 210, 0)), (380, (150, 110, 220, 40)), (700, (70, 230, 150, 140)),
                     (880, (140, 255, 186, 225)), (950, (215, 255, 230, 240)), (1000, (130, 252, 180, 150)))
        else:
            stops = ((0, (110, 50, 200, 0)), (450, (150, 70, 210, 50)), (800, (210, 100, 222, 135)),
                     (930, (250, 160, 236, 200)), (1000, (232, 134, 232, 120)))
        _grad(prof, 0, rh, 1, stops)
        rw = max(2, w // 200)
        glow = pygame.transform.smoothscale(glow, (rw, gh))
        p = (kind * 1.7 + 0.3, kind * 0.9 + 2.1, kind * 2.3 + 1.1, kind * 1.3 + 0.7)
        for x in range(0, w, rw):
            v = _hash(51000 + kind * 4000 + x // rw)
            a = math.tau * x / w
            i = (0.5 + 0.40 * math.sin(a * 2 + p[0]) + 0.18 * math.sin(a * 7 + p[1])
                 + 0.10 * math.sin(a * 17 + p[2]))
            i = max(0.0, min(1.0, i)) * (0.74 + (v % 26) / 100)
            hf = 0.56 + 0.30 * (0.5 + 0.5 * math.sin(a * 5 + p[3])) + (v >> 8) % 14 / 100
            hh = max(2, int(rh * min(1.0, hf)))
            jit = (v >> 16) % max(1, h // 90)
            glow.set_alpha(int(255 * i))
            s.blit(glow, (x, rh - jit))
            ray = pygame.transform.smoothscale(prof, (rw, hh))
            ray.set_alpha(max(0, min(255, int(12 + 243 * i))))
            s.blit(ray, (x, rh - hh - jit))
        s.blit(s.subsurface((0, 0, sl, rh + gh)).copy(), (w, 0))
        return s
    return _cached(_wkey(3, w, h, "ribbon", kind), build)


def _slice_w(w):
    """Streifenbreite, in der das Polarlicht gewellt gezeichnet wird (Vielfache
    von 8 blitten am schnellsten)."""
    return 8 if w >= 400 else 4


# Polarlicht-Bögen je Art: (Unterkante % von h, Bogenhöhe ‰, Scheitel ‰ von w,
# Welle 1: Höhe ‰ von h, Länge × w, Tempo; Welle 2 ebenso; Wandern × w/s)
_ARCS = ((68, 100, 400, 40, 0.9, 0.55, 18, 0.33, 0.9, 0.012),
         (47, 70, 560, 30, 1.3, 0.40, 16, 0.40, 0.7, -0.008))


def _aurora(s, rect, t):
    """Zwei wogende Polarlicht-Bögen (violett hinten und sanft pulsierend, grün
    vorn), in senkrechten Streifen versetzt gezeichnet und wandernd; das fast
    durchsichtige obere Neuntel der Strahlen wird nicht geblittet."""
    w, h = rect.w, rect.h
    sl = _slice_w(w)
    for kind in (1, 0):
        rib = _ribbon(w, h, kind)
        rh, th = _ribbon_h(h, kind)[0], rib.get_height()
        top = rh * 11 // 100
        base, arc, peak, a1, l1, o1, a2, l2, o2, drift = _ARCS[kind]
        if kind == 1:
            rib.set_alpha(int(195 + 60 * math.sin(t * 0.8 + 2.4)))
        by = rect.y + h * base // 100 - rh + top
        dx = t * w * drift
        A1, A2, B = h * a1 / 1000, h * a2 / 1000, h * arc / 1000
        px, span = w * peak / 1000, w * 0.62
        for x in range(0, w, sl):
            u = (x - px) / span
            yo = (B * u * u - B + A1 * math.sin(math.tau * x / (w * l1) + t * o1)
                  + A2 * math.sin(math.tau * x / (w * l2) - t * o2 + kind))
            sx = int(_wrap(x + dx, w))
            s.blit(rib, (rect.x + x, by + int(yo)), (sx, top, sl, th - top))
        if kind == 1:
            rib.set_alpha(255)


# ============================================================= Weltall
def _moon_r1(h):
    return max(6, h * 34 // 100)


def _moon_disc(d, seed):
    """Mondscheibe d x d: Meere, Krater mit hellem Rand, Kugelschatten zum Rand
    unten rechts (Licht oben links), glatter Rand (2-fach überabgetastet)."""
    ss = 2
    D = d * ss
    C, R = D // 2, D // 2 - ss
    big = pygame.Surface((D, D), pygame.SRCALPHA)
    pygame.draw.circle(big, (214, 214, 220), (C, C), R)
    for k in range(7):
        v = _hash(seed + k)
        ang = (v % 628) / 100
        rr = R * ((v >> 10) % 55) / 100
        mx, my = C + math.cos(ang) * rr, C + math.sin(ang) * rr
        for j in range(4):
            u = _hash(seed + 100 + k * 8 + j)
            pygame.draw.circle(big, (170, 172, 188),
                               (mx + R * ((u % 31) - 15) / 100, my + R * (((u >> 8) % 31) - 15) / 100),
                               R * (8 + (u >> 16) % 14) / 100)
    for k in range(64):
        v = _hash(seed + 500 + k)
        ang = (v % 628) / 100
        rr = R * 0.92 * math.sqrt(((v >> 10) % 1000) / 1000)
        cr = R * (1.5 + (v >> 20) % 5 + (8 if k < 3 else 3 if k < 10 else 0)) / 100
        x, y = C + math.cos(ang) * rr, C + math.sin(ang) * rr
        pygame.draw.circle(big, (234, 234, 240), (x - cr * 0.16, y - cr * 0.16), cr)
        pygame.draw.circle(big, (158, 160, 176), (x + cr * 0.06, y + cr * 0.06), cr * 0.86)
        pygame.draw.circle(big, (196, 198, 208), (x + cr * 0.24, y + cr * 0.24), cr * 0.6)
    sh = _shade(D, int(C - R * 0.38), int(C - R * 0.38), int(R * 1.56), (16, 18, 38), 190, 200)
    big.blit(sh, (0, 0))
    big.blit(_mask(D, (C, C), R), (0, 0), special_flags=pygame.BLEND_RGBA_MIN)
    return pygame.transform.smoothscale(big, (d, d))


def _moon_master(w, h):
    """Mond mit Lichthof in der größten Weltall-Größe (kleinere werden davon
    weich verkleinert)."""
    def build():
        R = _moon_r1(h)
        M = R * 27 // 10
        s = pygame.Surface((M, M), pygame.SRCALPHA)
        s.blit(_glow_new(M // 2, (176, 190, 236), 64), (0, 0))
        disc = _moon_disc(2 * R, 61000)
        s.blit(disc, (M // 2 - R, M // 2 - R))
        return s
    return _cached(_wkey(4, w, h, "moon"), build)


def _moon_geom(w, h, alt):
    """(cx, cy, r) des Monds: winzig oben rechts zu Beginn des Weltalls, zum
    Mond-Takt groß oben in der Mitte (wächst immer schneller)."""
    t4, tm = L.world_ticks()[4], L.MOON_TICK
    f = max(0.0, min(1.0, (alt - t4) / max(1, tm - t4)))
    r0, r1 = max(3.0, h * 0.018), float(_moon_r1(h))
    r = r0 * (r1 / r0) ** (f ** 1.4)
    k = (r - r0) / max(1.0, r1 - r0)
    return w * (0.80 - 0.30 * k), h * (0.20 - 0.17 * k), r


def _moon(s, rect, alt):
    """Wachsender Mond; nur die zuletzt gebrauchte Größe bleibt gecacht."""
    w, h = rect.w, rect.h
    cx, cy, r = _moon_geom(w, h, alt)
    R = _moon_r1(h)
    master = _moon_master(w, h)
    size = max(4, int(master.get_width() * r / R) // 2 * 2)
    key = _wkey(4, w, h, "moon_now")
    img = _cached(key, lambda: pygame.transform.smoothscale(master, (size, size)))
    if img.get_width() != size:
        img = master if size == master.get_width() else pygame.transform.smoothscale(master, (size, size))
        _cache[key] = img
    s.blit(img, (rect.x + int(cx) - size // 2, rect.y + int(cy) - size // 2))


def _earth(w, h):
    """Erdkrümmung (w x 56 % h): Nachtseite mit Stadtlichtern, blauer Saum über
    der Kante, Sonnenaufgang rechts."""
    def build():
        eh = max(8, h * 56 // 100)
        s = pygame.Surface((w, eh), pygame.SRCALPHA)
        g = max(4, h * 12 // 100)
        R = max(w, h) * 13 // 10
        cx, cy = w // 2, g + R

        def limb(x):
            return int(cy - math.sqrt(max(0, R * R - (x - cx) * (x - cx))))

        band = pygame.Surface((2, g), pygame.SRCALPHA)
        _grad(band, 0, g, 2, ((0, (40, 90, 220, 0)), (640, (70, 140, 255, 56)), (1000, (156, 214, 255, 200))))
        for x in range(0, w, 2):
            s.blit(band, (x, limb(x) - g))
        pygame.draw.circle(s, (8, 16, 40), (cx, cy), R)
        sb = max(3, h * 9 // 100)
        inner = pygame.Surface((2, sb), pygame.SRCALPHA)
        _grad(inner, 0, sb, 2, ((0, (60, 124, 214, 255)), (300, (26, 56, 116, 200)), (1000, (8, 16, 40, 0))))
        for x in range(0, w, 2):
            s.blit(inner, (x, limb(x)))
        dot = 2 if h >= 600 else 1
        for k in range(max(6, w // 40)):
            v = _hash(66000 + k)
            x = v % w
            y = limb(x) + sb * 6 // 10 + (v >> 12) % max(1, eh - limb(x) - sb * 6 // 10)
            for j in range(4 + (v >> 24) % 10):
                u = _hash(66500 + k * 16 + j)
                px = x + (u % 41 - 20) * h // 1000
                py = y + ((u >> 8) % 21 - 10) * h // 1000
                if py > limb(px) + sb // 2:
                    s.fill((255, 206, 120, 120 + (u >> 16) % 120), (px, py, dot, dot))
        pts = [(x, limb(x)) for x in range(0, w + 1, max(2, w // 200))]
        pygame.draw.lines(s, (176, 224, 255), False, pts, max(1, h // 320))
        sx = w * 86 // 100
        gl = _glow_new(max(4, h * 22 // 100), (255, 196, 120), 150)
        s.blit(gl, (sx - gl.get_width() // 2, limb(sx) - gl.get_height() // 2))
        sp = _sparkle(max(3, h // 60), (255, 236, 200))
        s.blit(sp, (sx - sp.get_width() // 2, limb(sx) - sp.get_height() // 2))
        return s
    return _cached(_wkey(4, w, h, "earth"), build)


def _earth_draw(s, rect, alt):
    """Erde zu Beginn des Weltalls: Kante bei 58 % von h, sinkt dann nach unten weg."""
    w, h = rect.w, rect.h
    t4 = L.world_ticks()[4]
    drop = max(-h * 0.08, (rise_px(alt, h) - rise_px(t4, h)) * _EARTH_P)
    y = rect.y + h * 58 // 100 - max(4, h * 12 // 100) + int(drop)
    if y < rect.bottom:
        s.blit(_earth(w, h), (rect.x, y))


def _planet(w, h, kind):
    """Ferner Planet (Mitte = Flächenmitte): 0 Ringplanet, 1 roter Planet,
    2 Eisriese; Licht von oben links (3-fach überabgetastet)."""
    def build():
        r = max(3, h * _PLANETS[kind][2] // 1000)
        ss = 3
        R = r * ss
        size = R * 5 if kind == 0 else R * 2 + 4 * ss
        C = size // 2
        disc = pygame.Surface((size, size), pygame.SRCALPHA)
        if kind == 0:
            bands = ((228, 198, 144), (206, 166, 114), (238, 216, 170), (192, 150, 102), (224, 192, 138))
            for k in range(11):
                disc.fill(bands[k % 5], (0, C - R + 2 * R * k // 11, size, 2 * R // 11 + 1))
        elif kind == 1:
            disc.fill((198, 102, 70))
            for k in range(6):
                v = _hash(67000 + k)
                pygame.draw.circle(disc, (154, 72, 52),
                                   (C + R * ((v % 120) - 60) // 100, C + R * (((v >> 8) % 120) - 60) // 100),
                                   R * (12 + (v >> 16) % 20) // 100)
            pygame.draw.ellipse(disc, (244, 236, 228),
                                (C - R * 45 // 100, C - R - R // 10, R * 90 // 100, R * 36 // 100))
        else:
            disc.fill((128, 200, 220))
            for k in range(5):
                disc.fill((112, 184, 210), (0, C - R + 2 * R * (2 * k + 1) // 10, size, max(1, R // 9)))
        disc.blit(_shade(size, C - R * 4 // 10, C - R * 4 // 10, R * 16 // 10, (8, 8, 24), 220), (0, 0))
        disc.blit(_mask(size, (C, C), R), (0, 0), special_flags=pygame.BLEND_RGBA_MIN)
        s = pygame.Surface((size, size), pygame.SRCALPHA)
        if kind == 0:
            ring = pygame.Surface((size, size), pygame.SRCALPHA)
            pygame.draw.ellipse(ring, (214, 194, 150, 210),
                                (C - R * 23 // 10, C - R * 55 // 100, R * 46 // 10, R * 110 // 100),
                                max(ss, R // 5))
            pygame.draw.ellipse(ring, (176, 156, 118, 160),
                                (C - R * 18 // 10, C - R * 42 // 100, R * 36 // 10, R * 84 // 100),
                                max(ss, R // 9))
            back = ring.copy()
            back.fill((0, 0, 0, 0), (0, C, size, size - C))
            ring.fill((0, 0, 0, 0), (0, 0, size, C))
            s.blit(back, (0, 0))
            s.blit(disc, (0, 0))
            s.blit(ring, (0, 0))
            s = pygame.transform.rotate(s, 16)
        else:
            s.blit(disc, (0, 0))
        return pygame.transform.smoothscale(s, (max(1, s.get_width() // ss), max(1, s.get_height() // ss)))
    return _cached(_wkey(4, w, h, "planet", kind), build)


def _planets(s, rect, rise):
    """Ferne Planeten ziehen ganz langsam von oben nach unten durchs Bild."""
    w, h = rect.w, rect.h
    t4 = L.world_ticks()[4]
    for kind, (start, fx, fr, _k) in enumerate(_PLANETS):
        r = max(3, h * fr // 1000)
        cy = rect.y - r * 2 + (rise - rise_px(t4 + start, h)) * _PLANET_P
        if cy < rect.y - r * 3 or cy > rect.bottom + r * 3:
            continue
        img = _planet(w, h, kind)
        s.blit(img, (rect.x + w * fx // 1000 - img.get_width() // 2, int(cy) - img.get_height() // 2))


# ============================================================= belebte Details
def _slots(off, h, gap):
    """Plätze i >= 0 einer Höhenreihe im Abstand gap, die gerade (mit Rand)
    sichtbar sein können; Platz i liegt bei y = h * _TOP // 100 + off - i * gap."""
    top = h * _TOP // 100 + off
    return range(max(0, int((top - 2 * h) // gap)), int((top + h) // gap) + 2)


def _cross(rect, v, d, sp, t, m):
    """x eines Dings, das waagerecht durchs Bild zieht (d = +1 nach rechts,
    -1 nach links): von außen (Rand m) hinein, hinaus und nach einer Pause
    von w/2 wieder von vorn - nur aus Zeit und Streuwert v."""
    P = rect.w + rect.w // 2 + 2 * m
    u = _wrap((v >> 4) % P + sp * t, P)
    return rect.x - m + u if d > 0 else rect.right + m - u


def _flocks(s, rect, world, t, rise):
    """Vogelschwärme: Wiese kleine lose Grüppchen, Wolken Gänse im V."""
    w, h = rect.w, rect.h
    n = max(5, h * (18 if world == 0 else 24) // 1000)
    gap = h * (50 if world == 0 else 56) // 100
    off = rise * PARALLAX[1]
    col = (46, 52, 74)
    margin = n * 8
    for i in _slots(off, h, gap):
        v = _hash(9000 + world * 500 + i)
        if v % 100 >= 80:
            continue
        y = rect.y + int(h * _TOP // 100 + off - i * gap - (v >> 8) % max(1, gap // 3))
        if y < rect.y - margin or y > rect.bottom + margin:
            continue
        d = 1 if (v >> 7) & 1 else -1
        x = _cross(rect, v, d, h * (34 + (v >> 9) % 30) / 1000, t, margin)
        nb = 3 + (v >> 20) % (3 if world == 0 else 5)
        rate = 7.0 if world == 0 else 4.2
        for k in range(nb):
            if world == 0:
                u = _hash(9400 + i * 8 + k)
                bx = x - d * k * n * 13 // 10 + (u % 9 - 4) * n // 6
                by = y + ((u >> 8) % 13 - 6) * n // 6
            else:
                rank, side = (k + 1) // 2, (1 if k % 2 else -1)
                bx = x - d * rank * n * 14 // 10
                by = y + side * rank * n * 6 // 10
            fr = int(t * rate + k * 1.37 + i) % 4
            s.blit(_bird(n, fr, col, d < 0), (int(bx) - n // 2, int(by) - n // 2))


def _lights(s, x, y, img_w, img_h, d, t, ph, r, spots):
    """Blinklichter eines Flugzeugs: spots = (Beacon, Strobe, Flügelspitze) als
    (x ‰, y ‰) bei Blick nach rechts; Positionslicht rot (nach rechts) bzw.
    grün, rotes Beacon blinkt, weißer Strobe doppelt."""
    def at(p):
        px = p[0] if d > 0 else 1000 - p[0]
        return x + img_w * px // 1000, y + img_h * p[1] // 1000

    nav = (255, 70, 70) if d > 0 else (80, 255, 120)
    q = _wrap(t * 1.1 + ph, 1.0)
    st = _wrap(t * 0.8 + ph * 0.7, 1.0)
    items = [(spots[2], nav, r)]
    if q < 0.16:
        items.append((spots[0], (255, 60, 60), r + 1))
    if st < 0.05 or 0.11 < st < 0.16:
        items.append((spots[1], (255, 255, 255), r + 2))
    for p, col, rr in items:
        lx, ly = at(p)
        s.blit(_light(rr, col), (int(lx) - max(2, rr), int(ly) - max(2, rr)))


def _planes(s, rect, t, rise):
    """Kleine Verkehrsflugzeuge mit leuchtender Fensterreihe und Blinklichtern."""
    w, h = rect.w, rect.h
    n = max(14, h * 9 // 100)
    gap = h * 60 // 100
    off = rise * PARALLAX[1]
    r = max(2, n // 16)
    for i in _slots(off, h, gap):
        v = _hash(12000 + i)
        if v % 100 >= 72:
            continue
        y = rect.y + int(h * _TOP // 100 + off - i * gap - (v >> 8) % max(1, gap // 3))
        if y < rect.y - n or y > rect.bottom + n:
            continue
        d = 1 if (v >> 6) & 1 else -1
        x = _cross(rect, v, d, w * (55 + (v >> 16) % 35) / 1000, t, n)
        img = _plane(n, d < 0)
        ix, iy = int(x) - n // 2, y - img.get_height() // 2
        s.blit(img, (ix, iy))
        _lights(s, ix, iy, n, img.get_height(), d, t, (v >> 24) / 256, r, ((520, 420), (20, 60), (370, 980)))


def _jets(s, rect, t, rise):
    """Jets mit Kondensstreifen unter dem Polarlicht."""
    w, h = rect.w, rect.h
    n = max(12, h * 6 // 100)
    gap = h * 65 // 100
    off = rise * PARALLAX[1]
    cl = max(16, w * 45 // 100)
    ch = max(2, h * 12 // 1000)
    r = max(2, n // 14)
    for i in _slots(off, h, gap):
        v = _hash(14000 + i)
        if v % 100 >= 75:
            continue
        y = rect.y + int(h * _TOP // 100 + off - i * gap - (v >> 8) % max(1, gap // 3))
        if y < rect.y - n or y > rect.bottom + n:
            continue
        d = 1 if (v >> 6) & 1 else -1
        x = _cross(rect, v, d, w * (100 + (v >> 16) % 50) / 1000, t, n + cl)
        img = _jet(n, d < 0)
        ix, iy = int(x) - n // 2, y - img.get_height() // 2
        tx = ix - cl + n // 10 if d > 0 else ix + n - n // 10
        s.blit(_contrail(cl, ch, d < 0), (tx, y + img.get_height() // 10 - ch // 2))
        s.blit(img, (ix, iy))
        _lights(s, ix, iy, n, img.get_height(), d, t, (v >> 24) / 256, r, ((540, 440), (30, 80), (340, 980)))


def _flash(t):
    """(Stärke 0..1, x ‰, y ‰, Variante) des Blitzes zur Zeit t oder None:
    in jedem _FLASH-Fenster mit 60 % ein Doppelblitz an gestreuter Stelle."""
    tt = max(0.0, t)
    k = int(tt // _FLASH)
    v = _hash(20011 + k % 1000000)
    if v % 100 >= 60:
        return None
    d = tt - k * _FLASH - ((v >> 8) % 1000) * (_FLASH - 0.5) / 1000
    if d < 0 or d >= 0.45:
        return None
    if d < 0.05:
        f = 1.0
    elif d < 0.09:
        f = 0.3
    elif d < 0.16:
        f = 0.9
    else:
        f = 0.9 * (0.45 - d) / 0.29
    return f, 150 + (v >> 12) % 700, 40 + (v >> 22) % 260, (v >> 4) % 4


def _lightning(s, rect, t, front):
    """Blitz: hinten Schein + Blitzast in den Wolken, vorn heller Aufschein
    über die ganze Bühne (additiv)."""
    fl = _flash(t)
    if fl is None:
        return
    f, fx, fy, var = fl
    w, h = rect.w, rect.h
    if front:
        g = int(34 * f)
        if g > 0:
            s.fill((g, g, g + g // 3), rect, special_flags=pygame.BLEND_RGB_ADD)
        return
    x, y = rect.x + w * fx // 1000, rect.y + h * fy // 1000
    gl = _bolt_glow(w, h)
    _blit_a(s, gl, (x - gl.get_width() // 2, y - gl.get_height() // 2), int(255 * f))
    if f > 0.5:
        b = _bolt(w, h, var)
        _blit_a(s, b, (x - b.get_width() // 2, y), min(255, int(300 * f)))


def _rain(s, rect, t, rise):
    """Schräger Regen: nahe Tropfen länger, heller, schneller; fällt beim
    Steigen zusätzlich schneller vorbei."""
    w, h = rect.w, rect.h
    n = w * h // 4200
    ln = max(5, h * 5 // 100)
    thick = 2 if h >= 600 else 1
    for i in range(n):
        v = _hash(30011 + i)
        near = v % 3 == 0
        sp = h * (130 + (v >> 8) % 70) / 100 * (1.4 if near else 1.0)
        y = _wrap((v >> 4) % 1000 * (h + ln) / 1000 + sp * t + rise * 0.6, h + ln) - ln
        ll = ln * (13 if near else 8) // 10
        x = (v >> 14) % (w + w // 3) - w // 8 - y * 0.22
        col = (136, 146, 176) if near else (92, 100, 128)
        x0, y0 = rect.x + int(x), rect.y + int(y)
        pygame.draw.line(s, col, (x0, y0), (x0 - ll * 22 // 100, y0 + ll), thick if near else 1)


def _twinkle(s, rect, world, t, rise):
    """Funkelnde Sterne (zur mittleren Ebene mitlaufend; Mond: nur am Himmel,
    nicht vor der Erde)."""
    w, h = rect.w, rect.h
    n = max(6, w * h // 24000)
    off = rise * PARALLAX[1]
    r = max(2, h // 150)
    for i in range(n):
        v = _hash(40000 + world * 1000 + i)
        u = _hash(40500 + world * 1000 + i)
        x = v % w
        if world == 5:
            y = u % max(1, h * 50 // 100)
            if abs(x - w * 76 // 100) < h * 16 // 100 and abs(y - h * 22 // 100) < h * 16 // 100:
                continue
        else:
            y = int(u % h + off) % h
        b = 0.5 + 0.5 * math.sin(t * (1.4 + (v >> 20) % 30 / 10) + (v >> 8) % 628 / 100)
        al = int(255 * b * b)
        if al < 12:
            continue
        spr = _sparkle(r + (u >> 24) % 2, _STAR_COLS[(u >> 12) % 4])
        _blit_a(s, spr, (rect.x + x - spr.get_width() // 2, rect.y + y - spr.get_height() // 2), al)


def _shooting(s, rect, t):
    """Sternschnuppe: in jedem _SHOOT-Fenster mit 55 % eine, 0,75 s lang."""
    tt = max(0.0, t)
    k = int(tt // _SHOOT)
    v = _hash(41017 + k % 1000000)
    if v % 100 >= 55:
        return
    dur = 0.75
    d = tt - k * _SHOOT - ((v >> 8) % 1000) * (_SHOOT - dur) / 1000
    if not 0 <= d < dur:
        return
    p = d / dur
    w, h = rect.w, rect.h
    x0 = rect.x + w * (100 + (v >> 12) % 800) // 1000
    y0 = rect.y + h * (40 + (v >> 20) % 350) // 1000
    dx = 1 if (v >> 3) & 1 else -1
    ang = math.radians(22 + (v >> 28) % 16)
    ux, uy = dx * math.cos(ang), math.sin(ang)
    hx, hy = x0 + ux * h * 0.45 * p, y0 + uy * h * 0.45 * p
    tail = h * 0.22 * min(1.0, p * 4)
    env = math.sin(math.pi * p)
    bg = (4, 6, 20)
    wid = 2 if h >= 400 else 1
    for j in range(5):
        c = _lerp(bg, (236, 242, 255), int(env * (1000 - j * 190)), 1000)
        f0, f1 = j / 5, (j + 1) / 5
        pygame.draw.line(s, c, (int(hx - ux * tail * f0), int(hy - uy * tail * f0)),
                         (int(hx - ux * tail * f1), int(hy - uy * tail * f1)), wid if j < 3 else 1)
    hd = _light(max(3, h // 110), (220, 230, 255))
    _blit_a(s, hd, (int(hx) - hd.get_width() // 2, int(hy) - hd.get_height() // 2), int(255 * env))


def _satellites(s, rect, t, rise):
    """Satelliten ziehen langsam schräg durchs Bild, mit rotem Blinklicht."""
    w, h = rect.w, rect.h
    n = max(10, h * 6 // 100)
    gap = h * 70 // 100
    off = rise * _SAT_P
    for i in _slots(off, h, gap):
        v = _hash(43013 + i)
        if v % 100 >= 80:
            continue
        d = 1 if (v >> 6) & 1 else -1
        x = int(_cross(rect, v, d, w * (22 + (v >> 16) % 20) / 1000, t, 2 * n))
        slope = ((v >> 24) % 40 - 20) / 100
        y = rect.y + int(h * _TOP // 100 + off - i * gap - (v >> 8) % max(1, gap // 3)
                         + (x - rect.centerx) * slope)
        if y < rect.y - n or y > rect.bottom + n:
            continue
        img = _satellite(n)
        s.blit(img, (x - img.get_width() // 2, y - img.get_height() // 2))
        if _wrap(t + (v >> 20) / 64, 1.3) < 0.14:
            lt = _light(max(2, n // 7), (255, 70, 70))
            s.blit(lt, (x - lt.get_width() // 2, y - lt.get_height() // 2))


def _details(s, rect, world, phase, alt, t, rise):
    """Belebtes je Welt; phase 0 = hinter der mittleren Ebene, 1 = hinter der
    nahen, 2 = ganz vorn."""
    if world == 0 or world == 1:
        if phase == 1:
            _flocks(s, rect, world, t, rise)
    elif world == 2:
        if phase == 0:
            _lightning(s, rect, t, False)
        elif phase == 1:
            _planes(s, rect, t, rise)
        else:
            _rain(s, rect, t, rise)
            _lightning(s, rect, t, True)
    elif world == 3:
        if phase == 0:
            _aurora(s, rect, t)
        elif phase == 1:
            _jets(s, rect, t, rise)
        else:
            _twinkle(s, rect, 3, t, rise)
    elif world == 4 and phase == 2:
        _twinkle(s, rect, 4, t, rise)
        _shooting(s, rect, t)
        _planets(s, rect, rise)
        _moon(s, rect, alt)
        _earth_draw(s, rect, alt)
        _satellites(s, rect, t, rise)


# ============================================================= Mond-Landung
def _marble(r):
    """Die Erde vom Mond aus: blaue Murmel mit Kontinenten, Wolken, Nachtseite
    rechts unten und Atmosphären-Saum (3-fach überabgetastet)."""
    ss = 3
    R = r * ss
    size = R * 2 + R
    C = size // 2
    s = pygame.Surface((size, size), pygame.SRCALPHA)
    s.blit(_glow_new(C, (110, 170, 255), 80), (0, 0))
    disc = pygame.Surface((size, size), pygame.SRCALPHA)
    pygame.draw.circle(disc, (34, 92, 198), (C, C), R)
    for k in range(5):
        v = _hash(64000 + k)
        ang, rr = (v % 628) / 100, R * (10 + (v >> 10) % 60) / 100
        mx, my = C + math.cos(ang) * rr, C + math.sin(ang) * rr
        for j in range(10):
            u = _hash(64100 + k * 16 + j)
            col = (172, 152, 100) if (u >> 24) % 4 == 0 else (86, 152, 80)
            pygame.draw.circle(disc, col, (mx + R * ((u % 51) - 25) / 100, my + R * (((u >> 8) % 41) - 20) / 100),
                               R * (4 + (u >> 16) % 9) / 100)
    for k in range(22):
        v = _hash(64500 + k)
        ang, rr = (v % 628) / 100, R * ((v >> 10) % 92) / 100
        cw, ch = R * (14 + (v >> 18) % 24) / 100, R * (3 + (v >> 24) % 4) / 100
        pygame.draw.ellipse(disc, (246, 248, 255, 200),
                            (C + math.cos(ang) * rr - cw / 2, C + math.sin(ang) * rr - ch / 2, cw, ch))
    disc.blit(_shade(size, int(C - R * 0.60), int(C - R * 0.25), int(R * 1.30), (4, 8, 26), 248, 560), (0, 0))
    disc.blit(_mask(size, (C, C), R), (0, 0), special_flags=pygame.BLEND_RGBA_MIN)
    s.blit(disc, (0, 0))
    pygame.draw.arc(s, (170, 214, 255, 190), (C - R, C - R, 2 * R, 2 * R), 1.2, 4.3, max(1, ss))
    return pygame.transform.smoothscale(s, (size // ss, size // ss))


def _crater(s, x, y, cw, ch):
    """Krater in Aufsicht schräg (Ellipse): heller Wall, dunkle Mulde,
    beschienene Innenwand rechts."""
    pygame.draw.ellipse(s, (172, 172, 180), (x - cw // 2, y - ch // 2 - max(1, ch // 8), cw, ch))
    pygame.draw.ellipse(s, (84, 84, 94), (x - cw * 42 // 100, y - ch * 36 // 100, cw * 84 // 100, ch * 76 // 100))
    pygame.draw.ellipse(s, (124, 124, 134),
                        (x - cw * 4 // 100, y - ch * 20 // 100, cw * 44 // 100, ch * 52 // 100))


def moon_scene(w, h):
    """Mondlandung: schwarzer Himmel mit Sternen, die Erde oben rechts, ferne
    Berge, graue Kraterlandschaft mit flachem Landeplatz in der Mitte unten."""
    w, h = max(1, int(w)), max(1, int(h))

    def build():
        s = pygame.Surface((w, h))
        _grad(s, 0, h, w, _SKY[5])
        bg = _SKY[5][0][1]
        for i in range(max(1, w * h // 1100)):
            v = _hash(81000 + 2 * i)
            u = _hash(81001 + 2 * i)
            col = _lerp(bg, _STAR_COLS[(v >> 20) % 4], 40 + (u >> 12) % 216, 255)
            s.fill(col, (v % w, u % (h * 62 // 100), 1 + ((u >> 24) % 9 == 0), 1 + ((u >> 24) % 9 == 0)))
        re = max(4, h * 10 // 100)
        e = _marble(re)
        s.blit(e, (w * 76 // 100 - e.get_width() // 2, h * 22 // 100 - e.get_height() // 2))
        yh = h * 60 // 100
        # ferne, runde Mondberge in zwei Reihen (Licht von links: helle Kante)
        step = max(3, w // 160)
        for amp, base, col, rim, sd in ((70, 10, (96, 96, 108), (140, 140, 152), (0.4, 2.0, 1.1)),
                                        (44, 22, (118, 118, 130), (170, 170, 180), (2.6, 0.8, 0.2))):
            pts = [(0, yh + h // 20)]
            for x in range(0, w + step, step):
                q = 0.5 + 0.5 * _wave(x, w, sd)
                pts.append((x, yh + h * base // 1000 - int(h * amp / 1000 * q * q)))
            pts.append((w + step, yh + h // 20))
            pygame.draw.polygon(s, col, pts)
            pygame.draw.lines(s, rim, False, pts[1:-1], max(1, h // 320))
        # Boden: Verlauf, leicht gewölbt
        RS = w * 2
        gh = h - yh + h // 20
        surf = pygame.Surface((w, gh), pygame.SRCALPHA)
        _grad(surf, 0, gh, w, ((0, (160, 160, 168, 255)), (400, (134, 134, 144, 255)), (1000, (96, 96, 106, 255))))
        mk = pygame.Surface((w, gh), pygame.SRCALPHA)
        pygame.draw.circle(mk, (255, 255, 255, 255), (w // 2, RS + h // 20), RS)
        surf.blit(mk, (0, 0), special_flags=pygame.BLEND_RGBA_MIN)
        s.blit(surf, (0, yh - h // 20))
        # dunklere Meere (flach in der Perspektive)
        for i in range(5):
            v = _hash(83500 + i)
            z = 0.15 + 0.7 * (v % 1000) / 1000
            mw, mh = int(w * (0.25 + 0.3 * z)), max(2, int(h * 0.04 * (0.5 + z)))
            mx = (v >> 10) % w
            my = yh + int((h - yh) * z * z) + h // 40
            pygame.draw.ellipse(s, (124, 124, 134), (mx - mw // 2, my - mh // 2, mw, mh))
        # Staub, Krater (hinten klein und flach, vorn groß), Steine
        for i in range(w * (h - yh) // 260):
            v = _hash(83000 + i)
            x, y = v % w, yh + (v >> 12) % max(1, h - yh)
            dv = (v >> 24) % 3
            s.fill(((150, 150, 158), (118, 118, 128), (170, 170, 178))[dv], (x, y, 1, 1))
        for i in range(26):
            v = _hash(84000 + i)
            z = (v % 1000) / 1000
            y = yh + int((h - yh) * (0.06 + 0.9 * z * z))
            x = v >> 10 & 0xFFFF
            x = x * w // 65536
            cw = int(w * (0.03 + 0.13 * z) * (0.6 + ((v >> 26) % 8) / 10))
            ch = max(2, int(cw * (0.16 + 0.22 * z)))
            if abs(x - w // 2) < w * 16 // 100 + cw // 2 and y > h * 70 // 100:
                continue
            _crater(s, x, y, cw, ch)
        for i in range(40):
            v = _hash(85000 + i)
            z = (v % 1000) / 1000
            y = yh + int((h - yh) * (0.05 + 0.92 * z))
            x = (v >> 10) % w
            rr = max(1, int(h * (0.002 + 0.012 * z)))
            if abs(x - w // 2) < w * 12 // 100 and y > h * 72 // 100:
                continue
            pygame.draw.ellipse(s, (66, 66, 76), (x - rr, y - rr // 2, rr * 2, rr))
            pygame.draw.ellipse(s, (176, 176, 184),
                                (x - rr * 7 // 10, y - rr * 8 // 10, rr * 12 // 10, rr * 7 // 10))
        return s
    return _cached(_wkey(5, w, h, "scene"), build)


# ============================================================= Zeichnen
def _assets(world, w, h):
    """(Schlüssel, Bauen) aller Weltflächen von world - zum Vorab-Bauen."""
    out = [(_wkey(world, w, h, "sky"), lambda: sky(world, w, h))]
    for d in range(3):
        out.append((_wkey(world, w, h, "layer", d), lambda d=d: layer(world, d, w, h)))
    if world == 0:
        out.append((_wkey(0, w, h, "ground"), lambda: ground(w, h)))
    elif world == 2:
        out.append((_wkey(2, w, h, "boltglow"), lambda: _bolt_glow(w, h)))
        for v in range(4):
            out.append((_wkey(2, w, h, "bolt", v), lambda v=v: _bolt(w, h, v)))
    elif world == 3:
        for k in range(2):
            out.append((_wkey(3, w, h, "ribbon", k), lambda k=k: _ribbon(w, h, k)))
    elif world == 4:
        out.append((_wkey(4, w, h, "earth"), lambda: _earth(w, h)))
        out.append((_wkey(4, w, h, "moon"), lambda: _moon_master(w, h)))
        for k in range(3):
            out.append((_wkey(4, w, h, "planet", k), lambda k=k: _planet(w, h, k)))
    return out


def _prebuild(world, w, h):
    """Eine noch fehlende Fläche von world bauen (höchstens eine je Bild)."""
    for key, make in _assets(world, w, h):
        if key not in _cache:
            make()
            return


def _keep(world, w, h):
    """Weltflächen aller anderen Welten und Größen verwerfen - nur world und
    world + 1 (in dieser Größe) bleiben."""
    for g in list(_groups):
        if g != (world, w, h) and g != (world + 1, w, h):
            _groups.remove(g)
            for k in [k for k in _cache if k[0] == "w" and k[1:4] == g]:
                del _cache[k]


def _buffer(w, h):
    """Bühnengroße Zwischenfläche für die Überblendung (eine, je Größe neu)."""
    buf = _cache.get(("buf",))
    if buf is None or buf.get_size() != (w, h):
        buf = pygame.Surface((w, h))
        _cache[("buf",)] = buf
    return buf


def _scene(s, rect, world, alt, t, rise, with_ground):
    """Eine Welt deckend: Himmel, Ebenen fern -> nah mit Parallaxe, dazwischen
    ihre Details, nach der nahen Ebene der Boden (solange er zu sehen ist)."""
    w, h = rect.w, rect.h
    s.blit(sky(world, w, h), rect.topleft)
    for depth in range(3):
        _tile(s, rect, layer(world, depth, w, h), rise * PARALLAX[depth])
        if depth == 2 and with_ground:
            gy = rect.bottom - ground_h(h) + int(rise)
            if gy < rect.bottom:
                s.blit(ground(w, h), (rect.x, gy))
        _details(s, rect, world, depth, alt, t, rise)


def draw(s, rect, world, share, alt, t):
    """Ganze Bühne in rect: die Welt mit Himmel, Ebenen (Parallaxe), Details und
    Boden; während der Überblendung wird die nächste Welt genauso in eine
    Zwischenfläche gezeichnet und mit share/255 darübergelegt (echtes
    Überblenden). Ohne Überblendung wird die nächste Welt vorab gebaut."""
    rect = pygame.Rect(rect)
    w, h = rect.w, rect.h
    if w < 4 or h < 4:
        return
    world = _world(world)
    _keep(world, w, h)
    clip = s.get_clip()
    s.set_clip(rect.clip(clip))
    if world >= 5:
        s.blit(moon_scene(w, h), rect.topleft)
        _twinkle(s, rect, 5, t, 0.0)
    else:
        share = max(0, min(255, int(share))) if world < 4 else 0
        rise = rise_px(alt, h)
        _scene(s, rect, world, alt, t, rise, True)
        if share:
            buf = _buffer(w, h)
            _scene(buf, buf.get_rect(), world + 1, alt, t, rise, False)
            buf.set_alpha(share)
            s.blit(buf, rect.topleft)
            buf.set_alpha(None)
        elif world < 4:
            _prebuild(world + 1, w, h)
    s.set_clip(clip)
