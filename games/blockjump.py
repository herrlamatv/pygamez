# -*- coding: utf-8 -*-
"""
blockjump.py
============
Block Jump - ein 3D-Jump'n'Run im Minecraft-Stil.

- Voll in 3D (Software-Renderer, gleiche Pipeline wie Snakes 3D-Modus): eine
  Welt aus Wuerfel-Bloecken, auf die man springt. Distanz-Nebel, Himmels-Verlauf
  und optionaler Motion-Blur sorgen fuer den "hochwertigen" Look.
- **Minecraft-Skin**: alle Bloecke tragen echte 16x16-Pixeltexturen (Gras, Erde,
  Stein, Eichenbretter, Diamant, Schleim, Holz), die perspektivisch korrekt in
  Texel-Vierecke zerlegt werden. Der Detailgrad haengt vom Abstand ab
  (Taste **T**: hoch / niedrig / aus), damit der Software-Renderer schnell bleibt.
- Dazu im gleichen Stil: **Steve-Spielfigur** mit Kopf/Gesicht, Armen, Beinen und
  Laufanimation (3rd Person), die **Hand im Ego-Modus**, ein **Beacon-Strahl** am
  Ziel, rotierende **Gold-Barren** als Coins, quadratische **Sonne**, driftende
  **Pixel-Wolken** und ein HUD mit **Herzen** und Schattenschrift.
- Blocktypen: Gras/Erde/Stein/Holz (fest, zum Draufspringen), **Leitern**
  (kletterbar), **Zaeune** (kleine Bloecke - blockieren, aber ueberspringbar),
  **Schleimbloecke** (katapultieren nach oben), ein **Ziel** und schwebende **Coins**.
- Kamera **standardmaessig 1st-Person wie Minecraft** (Mouselook mit Pointer-
  Capture); **V** schaltet auf eine 3rd-Person-Verfolgerkamera um.
- Steuerung: **WASD/Pfeile** laufen (relativ zur Blickrichtung), **Leertaste**
  springen, **Maus** umsehen. An Leitern klettert **W/S** hoch/runter.
  **V** Ansicht, **T** Texturen, **B** Motion-Blur, **C** Maus fangen/frei,
  **+/-** Empfindlichkeit.
- Seed-generierte Parkour-Level werden pro Level schwerer; das Ziel erreichen gibt
  Punkte + Zeitbonus, ein Absturz kostet ein Leben (Start mit 3). Bei 0 Leben ist
  Schluss - die Punktesumme ist der Highscore.
"""

import math
import random

import pygame

import ui
import settings as settings_mod
from game_base import Game, InputEvent
from i18n import t

# ---------------------------------------------------------------------------
#  Renderer-Konstanten (aus dem 3D-Modus von snake.py uebernommen)
# ---------------------------------------------------------------------------
NEAR = 0.12
FOV_MUL = 1.0
FOG_START = 16.0
FOG_END = 46.0
DEG_PER_PX = 0.12                 # Grad Drehung je Maus-Pixel bei sens = 1.0
PITCH_CLAMP = math.radians(84)

# Minecraft-Himmel: kraeftiges Blau oben, heller Dunst am Horizont.
COL_SKY_TOP = (110, 160, 250)
COL_SKY_HOR = (178, 209, 252)
COL_FOG = (198, 222, 252)
COL_SUN = (252, 250, 208)

# ---------------------------------------------------------------------------
#  Physik
# ---------------------------------------------------------------------------
GRAVITY = 20.0
JUMP_VEL = 7.7
MOVE_SPEED = 4.7
CLIMB_SPEED = 3.3
SPRING_VEL = 12.6
EYE_H = 1.62
PLAYER_H = 1.7
HALF_W = 0.3
VY_MIN, VY_MAX = -16.0, 14.5
EPS = 1e-4

# ---------------------------------------------------------------------------
#  Blocktypen
# ---------------------------------------------------------------------------
EMPTY, GRASS, DIRT, STONE, PLANK, LADDER, FENCE, SPRING, GOAL = range(9)
SOLID = {GRASS, DIRT, STONE, PLANK, GOAL}   # volle 1x1x1-Bloecke
FENCE_H = 0.8    # Zaun: niedriges Hindernis, per Sprung ueberwindbar
SPRING_H = 0.78  # Schleimblock: nur Landeflaeche (blockiert nie seitlich)

READY, PLAY, CLEAR, GAMEOVER = "ready", "play", "clear", "gameover"

SEED_BASE = {"easy": 1000, "normal": 2000, "hard": 3000}

TEX_MODES = ["off", "low", "high"]          # Index = self.tex_mode


# ===========================================================================
#  Pixel-Texturen im Minecraft-Stil
#  ------------------------------------------------------------------------
#  Jede Textur ist ein quadratisches Raster aus RGB-Tupeln (oder None fuer
#  "durchsichtig", z.B. beim Gold-Barren). Erzeugt wird alles prozedural mit
#  festen Seeds - so bleibt das Bild ueber alle Frames/Starts identisch, ohne
#  dass Bilddateien mitgeliefert werden muessen.
# ===========================================================================

def _mul(c, k):
    return (max(0, min(255, int(c[0] * k))),
            max(0, min(255, int(c[1] * k))),
            max(0, min(255, int(c[2] * k))))


def _blocks(size, seed, cols, weights, spec=(), spec_p=0.05, var=0.05, cell=4):
    """Texturen im Minecraft-Stil: grosse Farbfelder + wenige Einzel-Sprenkel.

    Echte MC-Texturen sind palettenbasiert und flaechig - genau das brauchen
    wir auch fuer die Geschwindigkeit: gleichfarbige Nachbarn fasst der
    Renderer spaeter zu einem einzigen Viereck zusammen (siehe ``_rects``).
    """
    rng = random.Random(seed)
    rows = [[None] * size for _ in range(size)]
    for by in range(0, size, cell):
        for bx in range(0, size, cell):
            base = rng.choices(cols, weights)[0]
            for sy in range(0, cell, 2):          # feine Abstufung je 2x2
                for sx in range(0, cell, 2):
                    c = _mul(base, rng.uniform(1 - var, 1 + var))
                    for y in range(sy, sy + 2):
                        for x in range(sx, sx + 2):
                            rows[by + y][bx + x] = c
    if spec:
        for y in range(size):
            for x in range(size):
                if rng.random() < spec_p:
                    rows[y][x] = rng.choice(spec)
    return tuple(tuple(r) for r in rows)


def _t_grass_top():
    return _blocks(16, 11, ((116, 172, 78), (105, 160, 70), (127, 182, 87)),
                   (5, 3, 2), ((97, 150, 64), (135, 190, 95)), 0.06)


def _t_dirt():
    return _blocks(16, 12, ((134, 96, 67), (123, 88, 60), (145, 105, 74)),
                   (5, 3, 2), ((113, 80, 55), (152, 112, 79)), 0.06)


def _t_grass_side():
    """Erde mit gruenem Ueberhang oben - das MC-Erkennungsmerkmal schlechthin."""
    dirt = _t_dirt()
    green, gdark = (116, 172, 78), (97, 150, 62)
    edge = (4, 4, 3, 3, 4, 4, 3, 3, 2, 2, 4, 4, 3, 3, 4, 4)
    rows = []
    for y in range(16):
        row = []
        for x in range(16):
            e = edge[x]
            if y < e - 1:
                row.append(green)
            elif y < e:
                row.append(gdark)
            else:
                row.append(dirt[y][x])
        rows.append(tuple(row))
    return tuple(rows)


def _t_stone():
    return _blocks(16, 14, ((127, 127, 127), (118, 118, 118), (136, 136, 136)),
                   (5, 3, 2), ((109, 109, 109), (145, 145, 145)), 0.06)


def _t_planks():
    """Eichenbretter: warmes Holz, dunkle Fugen, feine Maserung."""
    base, seam, grain, light = ((162, 130, 78), (124, 97, 55),
                                (146, 116, 68), (172, 141, 88))
    rng = random.Random(15)
    rows = []
    for y in range(16):
        band = (light if y in (0, 6, 12) else
                (grain if y in (3, 4, 9, 15) else base))
        row = []
        for x in range(16):
            if y in (5, 11):                      # waagrechte Brettfuge
                c = seam
            elif (y < 5 and x == 9) or (5 < y < 11 and x == 3) or (y > 11 and x == 12):
                c = seam                          # senkrechte Stossfuge
            elif rng.random() < 0.06:
                c = grain
            else:
                c = band
            row.append(c)
        rows.append(tuple(row))
    return tuple(rows)


def _t_wood():
    """Stamm-/Balkenholz fuer Leiter und Zaun (senkrechte Maserung)."""
    base, dark, light = (150, 116, 66), (120, 90, 50), (168, 132, 80)
    cols = tuple(dark if x in (0, 7, 8, 15) else (light if x in (3, 11) else base)
                 for x in range(16))
    rng = random.Random(16)
    rows = []
    for y in range(16):
        rows.append(tuple(dark if rng.random() < 0.04 else cols[x]
                          for x in range(16)))
    return tuple(rows)


def _t_diamond():
    """Diamantblock als Ziel: tuerkise Kristalle auf hellem Grund."""
    base, cry, hi, dk = (110, 222, 216), (78, 200, 198), (196, 248, 244), (58, 168, 168)
    spots = ((3, 3), (11, 4), (6, 10), (13, 12))
    rows = []
    for y in range(16):
        row = []
        for x in range(16):
            c = base
            for sx, sy in spots:
                d = max(abs(x - sx), abs(y - sy))
                if d == 0:
                    c = hi
                elif d == 1:
                    c = cry
                elif d == 2 and c is base:
                    c = dk
            row.append(c)
        rows.append(tuple(row))
    return tuple(rows)


def _t_slime():
    """Schleimblock: gruen, mit dunklem Rahmen und hellem Kern."""
    base, edge, core, hi = (114, 196, 94), (86, 158, 70), (146, 224, 126), (176, 240, 158)
    rows = []
    for y in range(16):
        row = []
        for x in range(16):
            b = min(x, y, 15 - x, 15 - y)
            if b < 1:
                c = edge
            elif b < 3:
                c = base
            elif b < 6:
                c = core
            else:
                c = hi
            row.append(c)
        rows.append(tuple(row))
    return tuple(rows)


def _t_beacon():
    """Lichtstrahl des Ziels - fast weiss mit tuerkisem Schimmer."""
    a, b = (226, 255, 246), (186, 246, 232)
    return tuple(tuple(a if (x + y) % 3 else b for x in range(8)) for y in range(8))


# ----- Zeichen-Art (Steve, Gold-Barren) -----------------------------------

def _art(rows, pal, noise=0.0, seed=0):
    rng = random.Random(seed)
    out = []
    for r in rows:
        line = []
        for ch in r:
            c = pal.get(ch)
            if c is None:
                line.append(None)
            else:
                line.append(_mul(c, rng.uniform(1 - noise, 1 + noise)) if noise else c)
        out.append(tuple(line))
    return tuple(out)


SKIN = {
    "H": (74, 48, 30),      # Haare
    "h": (60, 38, 23),      # Haare dunkel
    "S": (233, 189, 148),   # Haut
    "s": (204, 160, 122),   # Haut Schatten
    "W": (245, 245, 245),   # Augenweiss
    "I": (62, 92, 190),     # Iris
    "M": (150, 96, 78),     # Mund
    "C": (0, 172, 172),     # Shirt
    "c": (0, 142, 145),     # Shirt dunkel
    "B": (58, 63, 138),     # Hose
    "b": (46, 50, 116),     # Hose dunkel
    "G": (88, 76, 64),      # Schuh
    "K": (52, 44, 36),      # Guertel/Kragen
}

_FACE = ("HHHHHHHH",
         "HHHHHHHH",
         "HSSSSSSH",
         "SWIssIWS",
         "SSSsSSSS",
         "SSMMMMSS",
         "hSSSSSSh",
         "SSssssSS")

_HEAD_SIDE = ("HHHHHHHH",
              "HHHHHHHH",
              "HHHSSSSS",
              "HHSSSSSs",
              "HSSSSSSS",
              "SSSSSSSs",
              "SSSSSSSS",
              "SSsSSsSS")

_HEAD_BACK = ("HHHHHHHH",
              "HHHHHHHH",
              "HHHHHHHH",
              "HHHHHHHH",
              "HHHhhHHH",
              "HHHHHHHH",
              "hHHHHHHh",
              "SSssssSS")

_HEAD_TOP = ("HHHHHHHH",
             "HhHHHHhH",
             "HHHHhHHH",
             "HHhHHHHH",
             "HHHHHhHH",
             "HhHHHHHH",
             "HHHHhHHH",
             "HHHHHHHH")

_BODY_FRONT = ("SSKKKKSS",
               "CCCCCCCC",
               "CCCcCCCC",
               "CCCCCcCC",
               "CcCCCCCC",
               "CCCCCCcC",
               "cCCCCCCc",
               "KKKKKKKK")

_BODY_SIDE = ("KKKKKKKK",
              "CCCCCCCC",
              "CCcCCCCC",
              "CCCCCCcC",
              "CcCCCCCC",
              "CCCCcCCC",
              "cCCCCCCc",
              "KKKKKKKK")

_ARM = ("CCCCCCCC",
        "CCcCCCCC",
        "CCCCCcCC",
        "cCCCCCCc",
        "CCCCcCCC",
        "SSSSSSSS",
        "SsSSSSsS",
        "SSSssSSS")

_LEG = ("BBBBBBBB",
        "BBbBBBBB",
        "BBBBBbBB",
        "bBBBBBBb",
        "BBBBbBBB",
        "BBbBBBBB",
        "GGGGGGGG",
        "GGGGGGGG")

_INGOT = ("        ",
          "  ####  ",
          " ###### ",
          "########",
          "########",
          " ###### ",
          "  ####  ",
          "        ")


def _t_ingot():
    """Gold-Barren als schwebendes Item (mit durchsichtigem Rand)."""
    gold, hi, dk = (247, 214, 88), (255, 242, 170), (198, 152, 44)
    rows = []
    for y, line in enumerate(_INGOT):
        row = []
        for x, ch in enumerate(line):
            if ch == " ":
                row.append(None)
            elif y <= 2:
                row.append(hi)
            elif y >= 5:
                row.append(dk)
            else:
                row.append(gold)
        rows.append(tuple(row))
    return tuple(rows)


TEX = {
    "grass_top": _t_grass_top(),
    "grass_side": _t_grass_side(),
    "dirt": _t_dirt(),
    "stone": _t_stone(),
    "planks": _t_planks(),
    "wood": _t_wood(),
    "diamond": _t_diamond(),
    "slime": _t_slime(),
    "beacon": _t_beacon(),
    "ingot": _t_ingot(),
    "face": _art(_FACE, SKIN, 0.0, 31),
    "head_side": _art(_HEAD_SIDE, SKIN, 0.0, 32),
    "head_back": _art(_HEAD_BACK, SKIN, 0.0, 33),
    "head_top": _art(_HEAD_TOP, SKIN, 0.0, 34),
    "body_front": _art(_BODY_FRONT, SKIN, 0.0, 35),
    "body_side": _art(_BODY_SIDE, SKIN, 0.0, 36),
    "arm": _art(_ARM, SKIN, 0.0, 37),
    "leg": _art(_LEG, SKIN, 0.0, 38),
}

# Blocktyp -> (oben, Seite, unten)
BLOCK_TEX = {
    GRASS: ("grass_top", "grass_side", "dirt"),
    DIRT: ("dirt", "dirt", "dirt"),
    STONE: ("stone", "stone", "stone"),
    PLANK: ("planks", "planks", "planks"),
    GOAL: ("diamond", "diamond", "diamond"),
}

_MIP = {}      # (name, n) -> n x n Raster (Grundfarben)
_RECTS = {}    # (name, n) -> zusammengefasste Farbrechtecke
_GRID = {}     # (name, n, shade_i, fog_i) -> eingefaerbte Rechtecke
_AVG = {}      # name -> Durchschnittsfarbe (fuer den Modus "Texturen aus")
_OPAQUE = {}   # name -> Textur ohne durchsichtige Texel?

_QUANT = 12    # Farbraster: gleiche Nachbartoene lassen sich zusammenfassen


def _mip(name, n):
    """Verkleinert eine Textur auf n x n (Mittelwert + Kontrast + Farbraster)."""
    key = (name, n)
    m = _MIP.get(key)
    if m is not None:
        return m
    src = TEX[name]
    size = len(src)
    n = min(n, size)
    step = size // n
    rows = []
    for j in range(n):
        row = []
        for i in range(n):
            r = g = b = cnt = 0
            for y in range(j * step, (j + 1) * step):
                for x in range(i * step, (i + 1) * step):
                    c = src[y][x]
                    if c is None:
                        continue
                    r += c[0]; g += c[1]; b += c[2]; cnt += 1
            if cnt * 2 < step * step:
                row.append(None)                       # ueberwiegend durchsichtig
            else:
                row.append((r // cnt, g // cnt, b // cnt))
        rows.append(row)
    if n < size:                                       # Kontrast nachschaerfen
        vals = [c for row in rows for c in row if c]
        if vals:
            mr = sum(c[0] for c in vals) / len(vals)
            mg = sum(c[1] for c in vals) / len(vals)
            mb = sum(c[2] for c in vals) / len(vals)
            f = 1.08
            rows = [[None if c is None else
                     (max(0, min(255, int(mr + (c[0] - mr) * f))),
                      max(0, min(255, int(mg + (c[1] - mg) * f))),
                      max(0, min(255, int(mb + (c[2] - mb) * f))))
                     for c in row] for row in rows]
    q, h = _QUANT, _QUANT // 2                         # aufs Farbraster runden
    rows = [[None if c is None else
             (min(255, c[0] // q * q + h), min(255, c[1] // q * q + h),
              min(255, c[2] // q * q + h))
             for c in row] for row in rows]
    m = tuple(tuple(row) for row in rows)
    _MIP[key] = m
    return m


def _rects(name, n):
    """Fasst gleichfarbige Texel zu Rechtecken zusammen (spart Polygone).

    Die Zerlegung haengt nur von der Textur ab, nicht von Licht/Nebel - sie
    wird also einmal berechnet und fuer alle Flaechen wiederverwendet. Weil
    die Rechteckkanten im Weltraum Geraden sind, bleibt die Perspektive exakt.
    """
    key = (name, n)
    r = _RECTS.get(key)
    if r is not None:
        return r
    grid = _mip(name, n)
    n = len(grid)
    used = [[False] * n for _ in range(n)]
    out = []
    for j in range(n):
        for i in range(n):
            if used[j][i]:
                continue
            c = grid[j][i]
            used[j][i] = True
            if c is None:
                continue
            i1 = i
            while i1 + 1 < n and not used[j][i1 + 1] and grid[j][i1 + 1] == c:
                i1 += 1
                used[j][i1] = True
            j1 = j
            while (j1 + 1 < n
                   and all(not used[j1 + 1][x] and grid[j1 + 1][x] == c
                           for x in range(i, i1 + 1))):
                j1 += 1
                for x in range(i, i1 + 1):
                    used[j1][x] = True
            out.append((i, j, i1 + 1, j1 + 1, c))
    r = tuple(out)
    _RECTS[key] = r
    return r


def _avg_col(name):
    a = _AVG.get(name)
    if a is None:
        src = TEX[name]
        vals = [c for row in src for c in row if c]
        a = (sum(c[0] for c in vals) // len(vals), sum(c[1] for c in vals) // len(vals),
             sum(c[2] for c in vals) // len(vals)) if vals else (128, 128, 128)
        _AVG[name] = a
    return a


def _opaque(name):
    """True, wenn die Textur keine durchsichtigen Texel hat."""
    o = _OPAQUE.get(name)
    if o is None:
        o = all(c is not None for row in TEX[name] for c in row)
        _OPAQUE[name] = o
    return o


def _shade_col(col, k):
    return (min(255, int(col[0] * k)), min(255, int(col[1] * k)),
            min(255, int(col[2] * k)))


def _fog_col(col, ft):
    if ft <= 0:
        return col
    return (int(col[0] + (COL_FOG[0] - col[0]) * ft),
            int(col[1] + (COL_FOG[1] - col[1]) * ft),
            int(col[2] + (COL_FOG[2] - col[2]) * ft))


def _tex_grid(name, n, shade, ft):
    """Farbrechtecke (i0, j0, i1, j1, col) inklusive Licht und Nebel.

    Shade/Nebel werden quantisiert, damit der Cache klein bleibt und die
    Farbrechnung pro Flaeche entfaellt.
    """
    si = int(shade * 24 + 0.5)
    fi = int(ft * 12 + 0.5)
    key = (name, n, si, fi)
    g = _GRID.get(key)
    if g is None:
        k, f = si / 24.0, fi / 12.0
        g = tuple((i0, j0, i1, j1, _fog_col(_shade_col(c, k), f))
                  for i0, j0, i1, j1, c in _rects(name, n))
        _GRID[key] = g
    return g


# Einfarbige Ersatzfarben fuer die feine Holz-/Schleimgeometrie (und den
# Modus "Texturen aus"); die Blockfarben liefert _avg_col() direkt.
COL_LADDER = _avg_col("wood")
COL_FENCE = _avg_col("wood")
COL_SLIME = _avg_col("slime")

# Pixel-Wolken (0/1-Maske, in Kacheln zu je 8 Bloecken)
_CLOUD_MAP = ("01111000",
              "01111100",
              "00111000",
              "00000000",
              "00011110",
              "00111110",
              "00011100",
              "00000000")

_HEART_ART = ("  ##  ##  ",
              " #rr##rr# ",
              "#rrrrrrrr#",
              "#rrrrrrrr#",
              "#rrrrrrrr#",
              " #rrrrrr# ",
              "  #rrrr#  ",
              "   #rr#   ",
              "    ##    ")

_HEART_PAL = {"#": (54, 16, 16), "r": (226, 58, 52), " ": None}
_HEART_EMPTY_PAL = {"#": (34, 34, 38), "r": (72, 72, 78), " ": None}


def _dir_from(yaw, pitch):
    cp = math.cos(pitch)
    return (math.sin(yaw) * cp, math.sin(pitch), math.cos(yaw) * cp)


def _lerp3(a, b, f):
    return (a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f,
            a[2] + (b[2] - a[2]) * f)


class BlockJumpGame(Game):
    name = "Block Jump"
    highscore_key = "blockjump"
    supports_multiplayer = False

    MODES = [("easy", "blj.mode.easy"), ("normal", "blj.mode.normal"),
             ("hard", "blj.mode.hard")]

    # ===================================================== Aufbau / Reset
    def reset(self):
        mode = self.mode if self.mode in SEED_BASE else "normal"
        self.mode = mode
        cfg = self.settings.get("blockjump", {}) if isinstance(self.settings, dict) else {}
        self.blur = max(0.0, min(0.8, float(cfg.get("blur", 0.35))))
        self.view = cfg.get("view", "first")
        if self.view not in ("first", "third"):
            self.view = "first"
        self.sens = max(0.4, min(2.5, float(cfg.get("sens", 1.0))))
        # Maus-Richtung: Standard normal (Maus rechts -> Blick rechts). Wer die
        # frühere/klassische Belegung will, kann invertiert einstellen (Taste I).
        self.invert = bool(cfg.get("mouse_invert", False))
        # Texturdetail: 2 = hoch (bis 8x8 Texel je Flaeche), 1 = niedrig (4x4),
        # 0 = aus (einfarbige Flaechen wie im alten Look).
        tex = cfg.get("textures", "high")
        self.tex_mode = TEX_MODES.index(tex) if tex in TEX_MODES else 2
        self._make_fonts()
        self._sky_cache = None
        self._prev_frame = None
        self.anim = 0.0
        self.walk = 0.0
        self.held = set()
        self.capture_mouse = False
        self._want_capture = True
        self._new_run()

    def _make_fonts(self):
        self._hud = ui.font(20, bold=True)
        self._small = ui.font(15)
        self._huge = ui.font(max(30, self.height // 11), bold=True)
        self._mid = ui.font(22, bold=True)

    def on_surface_changed(self):
        self._make_fonts()
        self._sky_cache = None
        self._prev_frame = None

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("blockjump", {})[key] = value
            settings_mod.save_settings(self.settings)

    def _new_run(self):
        self.lives = 3
        self.score = 0
        self.game_over = False
        self.level = 1
        self.yaw = 0.0
        self.pitch = 0.0
        self._build_level(self.level)
        self.state = READY

    # ===================================================== Level-Generierung
    def _build_level(self, level):
        rng = random.Random(SEED_BASE.get(self.mode, 2000) + level * 7919)
        self.world = {}
        self.coins = []
        hard = self.mode == "hard"
        easy = self.mode == "easy"

        n_plat = 7 + level * 2 + (2 if hard else 0)
        self._plat_count = n_plat
        cx, cy, cz = 0, 0, 0
        self._min_y = 0
        self._pad(cx, cz, cy, 1, 1, GRASS)
        self.spawn = (cx + 0.5, cy + 1.0, cz + 0.5)

        feats = (["jump", "jump", "ladder", "spring", "fence"]
                 if not easy else ["jump", "jump", "jump", "ladder", "spring"])
        for i in range(n_plat):
            last = (i == n_plat - 1)
            feat = "jump" if last else rng.choice(feats)
            hw = 1
            hd = 1 if not hard else rng.choice([0, 1, 1])
            top_type = rng.choice([GRASS, GRASS, STONE, PLANK])

            if feat == "ladder":
                climb = rng.choice([3, 4] if easy else [3, 4, 5])
                # Leiter-Saeule mit Stuetzwand dahinter
                lad_z = cz + 1
                for yy in range(cy + 1, cy + climb + 1):
                    self.world[(cx, yy, lad_z)] = LADDER
                    self.world[(cx, yy, lad_z + 1)] = STONE
                # Coin mittig in der Kletterspalte
                self.coins.append((cx + 0.5, cy + climb / 2.0 + 1.4, lad_z + 0.5))
                cy = cy + climb
                # Pad beginnt HINTER der Stuetzwand, damit der Leiterschacht
                # (z = lad_z) nach oben offen bleibt
                cz = cz + 2 + hd
                self._pad(cx, cz, cy, hw, hd, top_type)

            elif feat == "spring":
                # erhoehter Schleimblock auf dem aktuellen Pad ...
                self.world[(cx, cy + 1, cz)] = SPRING
                # ... naechstes Pad deutlich hoeher (per Katapult erreichbar)
                lift = rng.choice([3, 4])
                dz = rng.choice([2, 3])
                cy = cy + lift
                cz = cz + dz
                self._pad(cx, cz, cy, hw, hd, top_type)
                self.coins.append((cx + 0.5, cy - lift / 2.0 + 2.0, cz - dz / 2.0))

            else:  # jump / fence
                dz = rng.choice([2, 3] if easy else [3, 4])
                dxr = [-1, 0, 1] if easy else [-2, -1, 0, 1, 2]
                dx = rng.choice(dxr)
                dyr = [-1, 0] if easy else [-2, -1, 0, 1]
                dy = rng.choice(dyr)
                if hard and dz >= 4:
                    # Max-Luecke entschaerfen: nie bergauf und immer ein
                    # tiefes Lande-Pad (max. Sprungweite ~3.6 Bloecke flach,
                    # nur ~2.8 bei dy = +1)
                    dy = min(dy, 0)
                    hd = 1
                mx, my, mz = cx, cy, cz
                cx = cx + dx
                cy = max(self._min_y - 3, cy + dy)
                cz = cz + dz
                self._pad(cx, cz, cy, hw, hd, top_type)
                # Coin ueber der Luecke
                self.coins.append(((mx + cx) / 2.0 + 0.5,
                                   (my + cy) / 2.0 + 1.7,
                                   (mz + cz) / 2.0 + 0.5))
                if feat == "fence" and not last:
                    self.world[(cx, cy + 1, cz)] = FENCE
            self._min_y = min(self._min_y, cy)

        # Ziel-Beacon auf dem letzten Pad
        self.world[(cx, cy + 1, cz)] = GOAL
        self.goal = (cx + 0.5, cy + 1.0, cz + 0.5)
        self.death_y = self._min_y - 7.0
        # Wolkendecke weit ueber dem hoechsten Block (wie in Minecraft)
        self._cloud_y = max(p[1] for p in self.world) + 46.0

        # Spieler setzen
        self.px, self.py, self.pz = self.spawn
        self.vx = self.vy = self.vz = 0.0
        self.on_ground = True
        self.on_ladder = False
        self.checkpoint = self.spawn
        self.level_time = 0.0
        self.coins_level = 0
        self._clear_t = 0.0
        self._cam_pos = (self.px, self.py + EYE_H, self.pz)
        self._cam_look = (self.px, self.py + EYE_H, self.pz + 1.0)

    def _pad(self, cx, cz, y, hw, hd, typ):
        for x in range(cx - hw, cx + hw + 1):
            for z in range(cz - hd, cz + hd + 1):
                self.world[(x, y, z)] = typ

    # ===================================================== Blockabfragen
    def _is_solid(self, x, y, z):
        return self.world.get((int(x), int(y), int(z)), EMPTY) in SOLID

    def _col_h(self, x, y, z, axis, delta):
        """Kollisionshoehe der Zelle fuer diese Bewegung (None = frei)."""
        typ = self.world.get((int(x), int(y), int(z)), EMPTY)
        if typ in SOLID:
            return 1.0
        if typ == FENCE:
            return FENCE_H
        if typ == SPRING and axis == 1 and delta < 0:
            return SPRING_H          # nur Landung von oben
        return None

    def _cell(self, x, y, z):
        return self.world.get((int(x), int(y), int(z)), EMPTY)

    # ===================================================== Eingabe
    _ARROWS = {"Up": "up", "Down": "down", "Left": "left", "Right": "right"}

    def _move_acts(self, key):
        """Bewegungs-Aktionen, die 'key' laut Belegung ausloest."""
        k = key.lower() if len(key) == 1 else key   # "W" (Shift) == "w"
        acts = {a for a in ("up", "down", "left", "right") if self.is_action(k, a)}
        if key in self._ARROWS:                     # Pfeiltasten-Fallback
            acts.add(self._ARROWS[key])
        return acts

    def handle_event(self, event):
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if self.state in (READY, GAMEOVER):
                if k in ("Return", "space"):
                    self._start_or_restart()
                return
            self.held |= self._move_acts(k)
            if k == "space" or self.is_action(k, "action"):
                self._jump()
            elif k in ("v", "V"):
                self._toggle_view()
            elif k in ("b", "B"):
                self._cycle_blur()
            elif k in ("t", "T"):
                self._cycle_tex()
            elif k in ("c", "C"):
                self._want_capture = not self._want_capture
            elif k in ("i", "I"):
                self._toggle_invert()
            elif k in ("plus", "KP_Add", "equal"):
                self._change_sens(0.1)
            elif k in ("minus", "KP_Subtract"):
                self._change_sens(-0.1)
        elif event.kind == InputEvent.KEYUP:
            self.held -= self._move_acts(event.key)
        elif event.kind == InputEvent.MOUSEDOWN:
            if self.state in (READY, GAMEOVER):
                self._start_or_restart()
            elif self.state == PLAY and not self.capture_mouse:
                self._want_capture = True
                self.capture_mouse = True
        elif event.kind == InputEvent.MOUSEREL and self.state == PLAY:
            self._apply_look(event.rel)

    def _start_or_restart(self):
        if self.state == GAMEOVER:
            self.game_over = False
            self._new_run()
        self.held.clear()
        self.state = PLAY
        self.level_time = 0.0
        self._want_capture = True
        self.capture_mouse = True
        self.play_sound("click")

    def _apply_look(self, rel):
        k = math.radians(DEG_PER_PX) * self.sens
        # s = -1 -> normal (Maus rechts = Blick rechts, Maus hoch = Blick hoch);
        # s = +1 -> invertiert (beide Achsen umgekehrt).
        s = 1.0 if self.invert else -1.0
        self.yaw = (self.yaw + rel[0] * k * s) % math.tau
        self.pitch = max(-PITCH_CLAMP, min(PITCH_CLAMP, self.pitch + rel[1] * k * s))

    def _toggle_view(self):
        self.view = "third" if self.view == "first" else "first"
        self._save_setting("view", self.view)

    def _cycle_blur(self):
        self.blur = 0.0 if self.blur >= 0.79 else round(self.blur + 0.2, 2)
        self._save_setting("blur", self.blur)

    def _cycle_tex(self):
        self.tex_mode = (self.tex_mode - 1) % 3      # hoch -> niedrig -> aus
        self._save_setting("textures", TEX_MODES[self.tex_mode])
        self.play_sound("click")

    def _toggle_invert(self):
        self.invert = not self.invert
        self._save_setting("mouse_invert", self.invert)
        self.play_sound("click")

    def _change_sens(self, d):
        self.sens = round(max(0.4, min(2.5, self.sens + d)), 1)
        self._save_setting("sens", self.sens)

    def _jump(self):
        if self.state != PLAY:
            return
        if self.on_ground:
            self.vy = JUMP_VEL
            self.on_ground = False
            self.play_sound("click")
        elif self.on_ladder:
            self.vy = JUMP_VEL * 0.85
            self.on_ladder = False
            f = _dir_from(self.yaw, 0.0)
            self.vx = -f[0] * MOVE_SPEED
            self.vz = -f[2] * MOVE_SPEED
            self.play_sound("click")

    # ===================================================== Update / Physik
    def update(self, dt):
        self.anim += dt
        if self.state == CLEAR:
            self._clear_t -= dt
            if self._clear_t <= 0:
                self.level += 1
                self._build_level(self.level)
                self.state = PLAY
            return
        if self.state != PLAY:
            return
        self.level_time += dt
        self._physics(min(dt, 0.05))

    def _held_axis(self, pos, neg):
        p = 1 if self.held & pos else 0
        n = 1 if self.held & neg else 0
        return p - n

    def _physics(self, dt):
        fwd = self._held_axis({"up"}, {"down"})
        strafe = self._held_axis({"right"}, {"left"})

        self.on_ladder = self._in_ladder()
        f = _dir_from(self.yaw, 0.0)
        fx, fz = f[0], f[2]
        rx, rz = -fz, fx                       # Kamera-Rechtsvektor (Bildschirm-rechts)

        if self.on_ladder:
            self.vy = CLIMB_SPEED * fwd
            mvx = rx * strafe
            mvz = rz * strafe
            ln = math.hypot(mvx, mvz)
            if ln > 1e-6:
                self.vx = mvx / ln * MOVE_SPEED * 0.7
                self.vz = mvz / ln * MOVE_SPEED * 0.7
            else:
                self.vx = self.vz = 0.0
        else:
            mvx = fx * fwd + rx * strafe
            mvz = fz * fwd + rz * strafe
            ln = math.hypot(mvx, mvz)
            if ln > 1e-6:
                self.vx = mvx / ln * MOVE_SPEED
                self.vz = mvz / ln * MOVE_SPEED
            else:
                self.vx = self.vz = 0.0
            self.vy -= GRAVITY * dt
        self.vy = max(VY_MIN, min(VY_MAX, self.vy))

        # Schrittzaehler fuer die Lauf-/Handanimation
        self.walk += dt * 9.0 * min(1.0, math.hypot(self.vx, self.vz) / MOVE_SPEED)

        self.on_ground = False
        self._land_cells = []
        self._move_axis(0, self.vx * dt)
        self._move_axis(2, self.vz * dt)
        self._move_axis(1, self.vy * dt)

        if self.on_ground:
            bounced = False
            for c in self._land_cells:
                if self.world.get(c) == SPRING:
                    self.vy = SPRING_VEL
                    self.on_ground = False
                    bounced = True
                    self.play_sound("move")
                    break
            if not bounced:
                self.checkpoint = (self.px, self.py, self.pz)

        self._collect_coins()
        if self._at_goal():
            self._level_clear()
            return
        if self.py < self.death_y:
            self._die()

    def _aabb(self):
        return ((self.px - HALF_W, self.py, self.pz - HALF_W),
                (self.px + HALF_W, self.py + PLAYER_H, self.pz + HALF_W))

    def _move_axis(self, axis, delta):
        if axis == 0:
            self.px += delta
        elif axis == 1:
            self.py += delta
        else:
            self.pz += delta
        mn, mx = self._aabb()
        xr = range(int(math.floor(mn[0] + EPS)), int(math.floor(mx[0] - EPS)) + 1)
        yr = range(int(math.floor(mn[1] + EPS)), int(math.floor(mx[1] - EPS)) + 1)
        zr = range(int(math.floor(mn[2] + EPS)), int(math.floor(mx[2] - EPS)) + 1)
        hits = []
        for x in xr:
            for y in yr:
                for z in zr:
                    h = self._col_h(x, y, z, axis, delta)
                    if h is not None and mn[1] + EPS < y + h:
                        hits.append((x, y, z, h))
        if not hits:
            return
        if axis == 0:
            if delta > 0:
                self.px = min(h[0] for h in hits) - HALF_W - EPS
            elif delta < 0:
                self.px = max(h[0] for h in hits) + 1 + HALF_W + EPS
            self.vx = 0.0
        elif axis == 2:
            if delta > 0:
                self.pz = min(h[2] for h in hits) - HALF_W - EPS
            elif delta < 0:
                self.pz = max(h[2] for h in hits) + 1 + HALF_W + EPS
            self.vz = 0.0
        else:
            if delta > 0:                        # Kopf stoesst an die Decke
                self.py = min(h[1] for h in hits) - PLAYER_H - EPS
                self.vy = 0.0
            elif delta < 0:                      # Landung auf dem Boden
                top = max(h[1] + h[3] for h in hits)
                self.py = top + EPS
                self.vy = 0.0
                self.on_ground = True
                self._land_cells = [(h[0], h[1], h[2]) for h in hits
                                    if h[1] + h[3] == top]

    def _in_ladder(self):
        mn, mx = self._aabb()
        for x in range(int(math.floor(mn[0] + EPS)), int(math.floor(mx[0] - EPS)) + 1):
            for y in range(int(math.floor(mn[1] + EPS)), int(math.floor(mx[1] - EPS)) + 1):
                for z in range(int(math.floor(mn[2] + EPS)), int(math.floor(mx[2] - EPS)) + 1):
                    if self._cell(x, y, z) == LADDER:
                        return True
        return False

    def _collect_coins(self):
        remaining = []
        for (cx, cy, cz) in self.coins:
            if (abs(self.px - cx) < 0.85 and abs(self.pz - cz) < 0.85
                    and abs((self.py + 0.9) - cy) < 1.2):
                self.score += 50
                self.coins_level += 1
                self.play_sound("click")
            else:
                remaining.append((cx, cy, cz))
        self.coins = remaining

    def _at_goal(self):
        gx, gy, gz = self.goal
        return (abs(self.px - gx) < 1.3 and abs(self.pz - gz) < 1.3
                and self.py > gy - 1.6)

    def _level_clear(self):
        par = self._plat_count * 3.2
        bonus = max(0, int((par - self.level_time) * 15))
        self.score += 1000 + bonus
        self._last_bonus = bonus
        self.state = CLEAR
        self._clear_t = 1.8
        self.play_sound("win")

    def _die(self):
        self.lives -= 1
        self.play_sound("gameover")
        if self.lives <= 0:
            self.state = GAMEOVER
            self.game_over = True
            self.capture_mouse = False
        else:
            self.px, self.py, self.pz = self.checkpoint
            self.vx = self.vy = self.vz = 0.0
            self.on_ground = True

    # ===================================================== Kamera
    def _update_cam(self):
        head = (self.px, self.py + EYE_H, self.pz)
        f = _dir_from(self.yaw, self.pitch)
        if self.view == "first":
            self._cam_pos = head
            self._cam_look = (head[0] + f[0], head[1] + f[1], head[2] + f[2])
        else:
            look_h = self.py + 1.1
            target = (self.px, look_h, self.pz)
            dist = 4.4
            want = (self.px - f[0] * dist, look_h - f[1] * dist + 0.5,
                    self.pz - f[2] * dist)
            self._cam_pos = self._cam_collide(target, want)
            self._cam_look = target

    def _cam_collide(self, a, b):
        dx, dy, dz = b[0] - a[0], b[1] - a[1], b[2] - a[2]
        steps = 7
        for i in range(1, steps + 1):
            tt = i / steps
            p = (a[0] + dx * tt, a[1] + dy * tt, a[2] + dz * tt)
            if self._is_solid(math.floor(p[0]), math.floor(p[1]), math.floor(p[2])):
                tt = max(0.0, (i - 1) / steps) * 0.92
                return (a[0] + dx * tt, a[1] + dy * tt, a[2] + dz * tt)
        return b

    # ===================================================== 3D-Renderer
    def _view_basis(self):
        cx, cy, cz = self._cam_pos
        fx = self._cam_look[0] - cx
        fy = self._cam_look[1] - cy
        fz = self._cam_look[2] - cz
        fl = math.sqrt(fx * fx + fy * fy + fz * fz) or 1.0
        f = (fx / fl, fy / fl, fz / fl)
        rl = math.hypot(f[2], f[0]) or 1.0
        r = (-f[2] / rl, 0.0, f[0] / rl)
        u = (r[1] * f[2] - r[2] * f[1],
             r[2] * f[0] - r[0] * f[2],
             r[0] * f[1] - r[1] * f[0])
        return r, u, f

    def _to_cam(self, p):
        r, u, f = self._basis
        dx = p[0] - self._cam_pos[0]
        dy = p[1] - self._cam_pos[1]
        dz = p[2] - self._cam_pos[2]
        return (dx * r[0] + dz * r[2],
                dx * u[0] + dy * u[1] + dz * u[2],
                dx * f[0] + dy * f[1] + dz * f[2])

    def _proj(self, c):
        k = self._f / c[2]
        return (self._scx + c[0] * k, self._scy - c[1] * k)

    @staticmethod
    def _clip_near(pts):
        out = []
        n = len(pts)
        for i in range(n):
            a, b = pts[i], pts[(i + 1) % n]
            a_in, b_in = a[2] >= NEAR, b[2] >= NEAR
            if a_in:
                out.append(a)
            if a_in != b_in:
                tt = (NEAR - a[2]) / (b[2] - a[2])
                out.append((a[0] + (b[0] - a[0]) * tt,
                            a[1] + (b[1] - a[1]) * tt, NEAR))
        return out

    @staticmethod
    def _fog(col, depth):
        return _fog_col(col, max(0.0, min(1.0, (depth - FOG_START)
                                          / (FOG_END - FOG_START))))

    def _tex_n(self, depth, native, scale=1.0, fine=False):
        """Wieviele Texel je Kante bei diesem Abstand? (Detailstufe/LOD)"""
        if not self.tex_mode:
            return 1
        lim = native if self.tex_mode == 2 else min(4, native)
        step = (7.0 if fine else 16.0) if self.tex_mode == 2 else 26.0
        px = self._f * scale / max(depth, 0.25)
        n = 1
        while n < lim and px >= n * 2 * step:
            n *= 2
        return n

    def _add_poly(self, items, world_pts, color, shade=1.0, outline=None,
                  fog=True):
        """Einfarbige Flaeche in die Zeichenliste legen."""
        cs = [self._to_cam(p) for p in world_pts]
        if all(c[2] < NEAR for c in cs):
            return
        cs = self._clip_near(cs)
        if len(cs) < 3:
            return
        depth = sum(c[2] for c in cs) / len(cs)
        if fog and depth > FOG_END + 6:
            return
        pts = [self._proj(c) for c in cs]
        if (all(p[0] < -40 for p in pts) or all(p[0] > self.width + 40 for p in pts)
                or all(p[1] < -40 for p in pts) or all(p[1] > self.height + 40 for p in pts)):
            return
        col = _shade_col(color, shade)
        if fog:
            col = self._fog(col, depth)
        items.append((depth, ((pts, col),),
                      (pts, outline) if outline else None))

    def _add_face(self, items, quad, tex, shade=1.0, outline=None,
                  scale=1.0, fine=False):
        """Texturiertes Viereck.

        ``quad`` = (p00, p10, p11, p01) im Weltraum; p00 ist die linke obere
        Texturecke, p00->p10 die u-Achse, p00->p01 die v-Achse. Die Flaeche
        wird je nach Abstand in n x n Texel-Vierecke zerlegt (LOD).
        """
        if self.tex_mode == 0:
            self._add_poly(items, quad, _avg_col(tex), shade, outline)
            return
        ca, cb, cc, cd = (self._to_cam(p) for p in quad)
        zs = (ca[2], cb[2], cc[2], cd[2])
        if max(zs) < NEAR:
            return
        depth = (zs[0] + zs[1] + zs[2] + zs[3]) * 0.25
        if depth > FOG_END + 6:
            return
        n = self._tex_n(depth, len(TEX[tex]), scale, fine)
        if n <= 1:
            self._add_poly(items, quad, _avg_col(tex), shade, outline)
            return
        ft = max(0.0, min(1.0, (depth - FOG_START) / (FOG_END - FOG_START)))
        rects = _tex_grid(tex, n, shade, ft)
        # Bilinear im Kameraraum: die Sicht-Transformation ist affin, das
        # Interpolieren dort ist also identisch zum Weltraum - aber billiger.
        quads = []
        if min(zs) >= NEAR:
            # Gitterpunkte in einem Rutsch interpolieren UND projizieren
            f, scx, scy = self._f, self._scx, self._scy
            ax, ay, az = ca
            dlx, dly, dlz = cd[0] - ax, cd[1] - ay, cd[2] - az
            bx, by, bz = cb
            drx, dry, drz = cc[0] - bx, cc[1] - by, cc[2] - bz
            pr = []
            for j in range(n + 1):
                fj = j / n
                lx, ly, lz = ax + dlx * fj, ay + dly * fj, az + dlz * fj
                ex, ey, ez = (bx + drx * fj) - lx, (by + dry * fj) - ly, \
                             (bz + drz * fj) - lz
                row = []
                for i in range(n + 1):
                    fi = i / n
                    k = f / (lz + ez * fi)
                    row.append((scx + (lx + ex * fi) * k, scy - (ly + ey * fi) * k))
                pr.append(row)
            xs = (pr[0][0][0], pr[0][n][0], pr[n][0][0], pr[n][n][0])
            ys = (pr[0][0][1], pr[0][n][1], pr[n][0][1], pr[n][n][1])
            if (max(xs) < -40 or min(xs) > self.width + 40
                    or max(ys) < -40 or min(ys) > self.height + 40):
                return
            if _opaque(tex):
                # Grundfarbe unterlegen: sonst blitzt zwischen zwei Texel-
                # Vierecken gelegentlich der Hintergrund als heller Strich durch
                quads.append(((pr[0][0], pr[0][n], pr[n][n], pr[n][0]),
                              _tex_grid(tex, 1, shade, ft)[0][4]))
            for i0, j0, i1, j1, col in rects:
                r0, r1 = pr[j0], pr[j1]
                quads.append(((r0[i0], r0[i1], r1[i1], r1[i0]), col))
        else:                                   # Flaeche schneidet die Nahebene
            left = [_lerp3(ca, cd, j / n) for j in range(n + 1)]
            right = [_lerp3(cb, cc, j / n) for j in range(n + 1)]
            rows = [[_lerp3(left[j], right[j], i / n) for i in range(n + 1)]
                    for j in range(n + 1)]
            if _opaque(tex):
                base = self._clip_near((ca, cb, cc, cd))
                if len(base) >= 3:
                    quads.append(([self._proj(c) for c in base],
                                  _tex_grid(tex, 1, shade, ft)[0][4]))
            for i0, j0, i1, j1, col in rects:
                r0, r1 = rows[j0], rows[j1]
                cell = self._clip_near((r0[i0], r0[i1], r1[i1], r1[i0]))
                if len(cell) >= 3:
                    quads.append(([self._proj(c) for c in cell], col))
        if quads:
            items.append((depth, quads, (
                [self._proj(c) for c in self._clip_near((ca, cb, cc, cd))], outline)
                if outline else None))

    def _cull(self, cx, cy, cz):
        """Grobes Frustum-Cull anhand des Wuerfelzentrums (spart Flaechen-Arbeit)."""
        c = self._to_cam((cx, cy, cz))
        if c[2] < -1.7 or c[2] > FOG_END + 2:
            return True
        if abs(c[0]) > c[2] * 1.9 + 3.0:
            return True
        return False

    # ----- Bausteine: Quader (achsenparallel) ------------------------------
    def _add_prism(self, items, x0, x1, y0, y1, z0, z1, top, side, outline=None,
                   tex=None, fine=True, lit=None):
        """Achsenparalleler Quader; ``tex`` = (oben, Seite, unten) oder None.

        ``lit`` ueberschreibt die Flaechenhelligkeit (fuer selbstleuchtende
        Dinge wie den Beacon-Strahl).
        """
        px, py, pz = self._cam_pos
        sc = max(x1 - x0, y1 - y0, z1 - z0)
        sh = (lambda k: lit if lit is not None else k)
        if py > y1:
            q = ((x0, y1, z0), (x1, y1, z0), (x1, y1, z1), (x0, y1, z1))
            if tex:
                self._add_face(items, q, tex[0], sh(1.0), outline, sc, fine)
            else:
                self._add_poly(items, q, top, sh(1.0), outline)
        elif py < y0:
            q = ((x0, y0, z1), (x1, y0, z1), (x1, y0, z0), (x0, y0, z0))
            if tex:
                self._add_face(items, q, tex[2], sh(0.50), outline, sc, fine)
            else:
                self._add_poly(items, q, side, sh(0.50), outline)
        if px < x0:
            q = ((x0, y1, z1), (x0, y1, z0), (x0, y0, z0), (x0, y0, z1))
            if tex:
                self._add_face(items, q, tex[1], sh(0.62), outline, sc, fine)
            else:
                self._add_poly(items, q, side, sh(0.62), outline)
        elif px > x1:
            q = ((x1, y1, z0), (x1, y1, z1), (x1, y0, z1), (x1, y0, z0))
            if tex:
                self._add_face(items, q, tex[1], sh(0.62), outline, sc, fine)
            else:
                self._add_poly(items, q, side, sh(0.62), outline)
        if pz < z0:
            q = ((x1, y1, z0), (x0, y1, z0), (x0, y0, z0), (x1, y0, z0))
            if tex:
                self._add_face(items, q, tex[1], sh(0.80), outline, sc, fine)
            else:
                self._add_poly(items, q, side, sh(0.80), outline)
        elif pz > z1:
            q = ((x0, y1, z1), (x1, y1, z1), (x1, y0, z1), (x0, y0, z1))
            if tex:
                self._add_face(items, q, tex[1], sh(0.80), outline, sc, fine)
            else:
                self._add_poly(items, q, side, sh(0.80), outline)

    def _add_cube(self, items, bx, by, bz, tex, outline):
        """Voxel bei (bx,by,bz); nur sichtbare Flaechen, deren Nachbar leer ist."""
        px, py, pz = self._cam_pos
        x0, x1 = bx, bx + 1
        y0, y1 = by, by + 1
        z0, z1 = bz, bz + 1
        if py > y1 and not self._is_solid(bx, by + 1, bz):
            self._add_face(items, ((x0, y1, z0), (x1, y1, z0), (x1, y1, z1), (x0, y1, z1)),
                           tex[0], 1.0, outline)
        elif py < y0 and not self._is_solid(bx, by - 1, bz):
            self._add_face(items, ((x0, y0, z1), (x1, y0, z1), (x1, y0, z0), (x0, y0, z0)),
                           tex[2], 0.50, outline)
        if px < x0 and not self._is_solid(bx - 1, by, bz):
            self._add_face(items, ((x0, y1, z1), (x0, y1, z0), (x0, y0, z0), (x0, y0, z1)),
                           tex[1], 0.62, outline)
        elif px > x1 and not self._is_solid(bx + 1, by, bz):
            self._add_face(items, ((x1, y1, z0), (x1, y1, z1), (x1, y0, z1), (x1, y0, z0)),
                           tex[1], 0.62, outline)
        if pz < z0 and not self._is_solid(bx, by, bz - 1):
            self._add_face(items, ((x1, y1, z0), (x0, y1, z0), (x0, y0, z0), (x1, y0, z0)),
                           tex[1], 0.80, outline)
        elif pz > z1 and not self._is_solid(bx, by, bz + 1):
            self._add_face(items, ((x0, y1, z1), (x1, y1, z1), (x1, y0, z1), (x0, y0, z1)),
                           tex[1], 0.80, outline)

    # ----- Bausteine: gedrehte Quader (Spielfigur) -------------------------
    def _add_obox(self, items, base, yaw, swing, lo, hi, tex_front, tex_side,
                  tex_top, tex_bottom=None, tex_back=None):
        """Um die Y-Achse (yaw) und um die lokale X-Achse (swing) gedrehter Quader.

        ``lo``/``hi`` sind die lokalen Grenzen (x = rechts, y = hoch, z = vorne)
        relativ zum Drehpunkt ``base``; damit schwingen Arme und Beine sauber
        um Schulter bzw. Huefte.
        """
        cs, sn = math.cos(swing), math.sin(swing)
        fx, fz = math.sin(yaw), math.cos(yaw)
        rx, rz = -fz, fx
        bx, by, bz = base

        def w(p):
            x, y, z = p
            yy = y * cs - z * sn
            zz = y * sn + z * cs
            return (bx + x * rx + zz * fx, by + yy, bz + x * rz + zz * fz)

        x0, y0, z0 = lo
        x1, y1, z1 = hi
        # 8 Ecken (lokal) -> Welt
        p = {}
        for xi, xv in ((0, x0), (1, x1)):
            for yi, yv in ((0, y0), (1, y1)):
                for zi, zv in ((0, z0), (1, z1)):
                    p[(xi, yi, zi)] = w((xv, yv, zv))
        cam = self._cam_pos
        cen = w(((x0 + x1) / 2, (y0 + y1) / 2, (z0 + z1) / 2))

        def face(quad, tex, shade):
            if tex is None:
                return
            a, b, c = quad[0], quad[1], quad[3]
            ux, uy, uz = b[0] - a[0], b[1] - a[1], b[2] - a[2]
            vx, vy, vz = c[0] - a[0], c[1] - a[1], c[2] - a[2]
            nx = uy * vz - uz * vy
            ny = uz * vx - ux * vz
            nz = ux * vy - uy * vx
            mid = ((quad[0][0] + quad[2][0]) / 2, (quad[0][1] + quad[2][1]) / 2,
                   (quad[0][2] + quad[2][2]) / 2)
            if (nx * (mid[0] - cen[0]) + ny * (mid[1] - cen[1])
                    + nz * (mid[2] - cen[2])) < 0:
                nx, ny, nz = -nx, -ny, -nz
            if (nx * (cam[0] - mid[0]) + ny * (cam[1] - mid[1])
                    + nz * (cam[2] - mid[2])) <= 0:
                return                                   # Rueckseite
            self._add_face(items, quad, tex, shade, None, max(x1 - x0, y1 - y0), True)

        face((p[(0, 1, 1)], p[(1, 1, 1)], p[(1, 0, 1)], p[(0, 0, 1)]), tex_front, 0.94)
        face((p[(1, 1, 0)], p[(0, 1, 0)], p[(0, 0, 0)], p[(1, 0, 0)]),
             tex_back or tex_front, 0.66)
        face((p[(1, 1, 1)], p[(1, 1, 0)], p[(1, 0, 0)], p[(1, 0, 1)]), tex_side, 0.78)
        face((p[(0, 1, 0)], p[(0, 1, 1)], p[(0, 0, 1)], p[(0, 0, 0)]), tex_side, 0.72)
        face((p[(0, 1, 0)], p[(1, 1, 0)], p[(1, 1, 1)], p[(0, 1, 1)]), tex_top, 1.0)
        face((p[(0, 0, 1)], p[(1, 0, 1)], p[(1, 0, 0)], p[(0, 0, 0)]),
             tex_bottom or tex_side, 0.45)

    def _add_sprite(self, items, center, size, tex, spin):
        """Frei rotierendes Item-Sprite (Gold-Barren) mit durchsichtigem Rand."""
        cx, cy, cz = center
        h = size * 0.5
        rx, rz = math.cos(spin) * h, math.sin(spin) * h
        quad = ((cx - rx, cy + h, cz - rz), (cx + rx, cy + h, cz + rz),
                (cx + rx, cy - h, cz + rz), (cx - rx, cy - h, cz - rz))
        self._add_face(items, quad, tex, 1.0, None, size, True)

    # ----- Weltobjekte ----------------------------------------------------
    def _add_block(self, items, pos, typ):
        bx, by, bz = pos
        if self._cull(bx + 0.5, by + 0.5, bz + 0.5):
            return
        line = (30, 34, 40) if self.tex_mode == 0 else None
        if typ in BLOCK_TEX:
            self._add_cube(items, bx, by, bz, BLOCK_TEX[typ], line)
        elif typ == LADDER:
            wood = ("wood", "wood", "wood")
            for sx in (0.14, 0.66):             # zwei Holme
                self._add_prism(items, bx + sx, bx + sx + 0.2, by, by + 1,
                                bz + 0.06, bz + 0.24, COL_LADDER,
                                _shade_col(COL_LADDER, 0.8), line, wood)
            for ry in range(4):                 # Sprossen
                yy = by + 0.12 + ry * 0.26
                self._add_prism(items, bx + 0.1, bx + 0.9, yy, yy + 0.09,
                                bz + 0.03, bz + 0.27, _shade_col(COL_LADDER, 1.15),
                                COL_LADDER, None, wood)
        elif typ == FENCE:
            wood = ("wood", "wood", "wood")
            self._add_prism(items, bx + 0.38, bx + 0.62, by, by + 1,
                            bz + 0.38, bz + 0.62, COL_FENCE,
                            _shade_col(COL_FENCE, 0.8), line, wood)
            for yy in (by + 0.34, by + 0.66):   # zwei Querbalken je Achse
                self._add_prism(items, bx + 0.02, bx + 0.98, yy, yy + 0.13,
                                bz + 0.44, bz + 0.56, COL_FENCE,
                                _shade_col(COL_FENCE, 0.85), None, wood)
                self._add_prism(items, bx + 0.44, bx + 0.56, yy, yy + 0.13,
                                bz + 0.02, bz + 0.98, COL_FENCE,
                                _shade_col(COL_FENCE, 0.85), None, wood)
        elif typ == SPRING:
            slime = ("slime", "slime", "slime")
            self._add_prism(items, bx + 0.02, bx + 0.98, by, by + SPRING_H,
                            bz + 0.02, bz + 0.98, COL_SLIME,
                            _shade_col(COL_SLIME, 0.85), line, slime, fine=False)

    def _add_goal_beam(self, items):
        """Beacon-Strahl ueber dem Ziel."""
        gx, gy, gz = self.goal
        p = 0.5 + 0.5 * math.sin(self.anim * 3.0)
        w = 0.24 + 0.03 * p
        beam = ("beacon", "beacon", "beacon")
        self._add_prism(items, gx - w, gx + w, gy + 0.05, gy + 18.0,
                        gz - w, gz + w, (240, 255, 250), (215, 248, 240),
                        None, beam, fine=False, lit=1.0)

    def _add_player(self, items):
        """Steve: Kopf, Koerper, Arme, Beine - mit Laufanimation."""
        px, py, pz = self.px, self.py, self.pz
        u = PLAYER_H / 32.0                  # ein Skin-Pixel in Weltmass
        sp = min(1.0, math.hypot(self.vx, self.vz) / MOVE_SPEED)
        if self.on_ground:
            swing = math.sin(self.walk) * 0.85 * sp
        else:
            swing = 0.45 if self.vy > 0 else 0.22
        yaw = self.yaw
        head_y = py + 24 * u
        arm_y = py + 23.5 * u
        hip_y = py + 12 * u

        # Beine (schwingen gegengleich)
        for side, sgn in ((-1, 1), (1, -1)):
            self._add_obox(items, (px, hip_y, pz), yaw, swing * sgn,
                           (side * 4 * u - 2 * u, -12 * u, -2 * u),
                           (side * 4 * u + 2 * u, 0.0, 2 * u),
                           "leg", "leg", "leg", "leg")
        # Torso
        self._add_obox(items, (px, py, pz), yaw, 0.0,
                       (-4 * u, 12 * u, -2 * u), (4 * u, 24 * u, 2 * u),
                       "body_front", "body_side", "body_side", "body_side",
                       "body_side")
        # Arme
        for side, sgn in ((-1, -1), (1, 1)):
            self._add_obox(items, (px, arm_y, pz), yaw, swing * sgn,
                           (side * 6 * u - 2 * u, -12 * u, -2 * u),
                           (side * 6 * u + 2 * u, 0.0, 2 * u),
                           "arm", "arm", "arm", "arm")
        # Kopf (neigt sich mit dem Blick)
        self._add_obox(items, (px, head_y, pz), yaw, -self.pitch * 0.7,
                       (-4 * u, 0.0, -4 * u), (4 * u, 8 * u, 4 * u),
                       "face", "head_side", "head_top", "head_top", "head_back")

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        self.capture_mouse = (self._want_capture and self.state == PLAY
                              and not getattr(self, "paused", False))
        self._update_cam()
        self._scx = self.width / 2
        self._scy = self.height * 0.5
        self._f = self.height * FOV_MUL
        self._basis = self._view_basis()

        self._draw_sky(s)
        self._draw_sun_and_clouds(s)

        items = []
        for pos, typ in self.world.items():
            self._add_block(items, pos, typ)
        for coin in self.coins:
            cy = coin[1] + 0.12 * math.sin(self.anim * 2 + coin[0])
            self._add_sprite(items, (coin[0], cy, coin[2]), 0.45, "ingot",
                             self.anim * 1.9 + coin[0] + coin[2])
        self._add_goal_beam(items)
        if self.view == "third":
            self._add_player(items)

        items.sort(key=lambda it: -it[0])
        poly = pygame.draw.polygon
        for _, quads, outline in items:
            for pts, col in quads:
                poly(s, col, pts)
            if outline:
                poly(s, outline[1], outline[0], 1)

        self._apply_blur(s)
        if self.view == "first" and self.state != GAMEOVER:
            self._draw_hand(s)
            if self.state == PLAY:
                self._draw_crosshair(s)
        self._draw_hud(s)
        if self.state == READY:
            self._banner(s, t("blj.ready"), self.accent, t("blj.controls"))
        elif self.state == CLEAR:
            self._banner(s, t("blj.clear", n=self.level),
                         (120, 235, 170),
                         t("blj.clear_sub", pts=1000 + getattr(self, "_last_bonus", 0)))
        elif self.state == GAMEOVER:
            self._banner(s, t("blj.gameover"), (232, 96, 96),
                         t("common.points", score=self.score) + "   ·   "
                         + t("common.enter_restart"))

    # ----- Himmel, Sonne, Wolken -----------------------------------------
    def _draw_sky(self, s):
        if self._sky_cache is None or self._sky_cache[0] != (self.width, self.height):
            surf = pygame.Surface((self.width, self.height))
            hor = int(self.height * 0.52)
            haze = int(self.height * 0.66)
            for y in range(self.height):
                if y < hor:
                    tt = y / max(1, hor)
                    c = [int(a + (b - a) * tt) for a, b in zip(COL_SKY_TOP, COL_SKY_HOR)]
                elif y < haze:
                    tt = (y - hor) / max(1, haze - hor)
                    c = [int(a + (b - a) * tt) for a, b in zip(COL_SKY_HOR, COL_FOG)]
                else:
                    c = COL_FOG
                pygame.draw.line(surf, c, (0, y), (self.width, y))
            self._sky_cache = ((self.width, self.height), surf)
        s.blit(self._sky_cache[1], (0, 0))

    def _draw_sun_and_clouds(self, s):
        """Quadratische Sonne + driftende Pixel-Wolken (immer hinter der Welt)."""
        sky = []
        cx, cy, cz = self._cam_pos
        # Sonne: feste Weltrichtung, weit weg, ohne Nebel
        d, r = 260.0, 19.0
        sv = (0.52, 0.66, 0.54)
        sl = math.sqrt(sum(v * v for v in sv))
        sv = tuple(v / sl for v in sv)
        sc = (cx + sv[0] * d, cy + sv[1] * d, cz + sv[2] * d)
        # Sonnenscheibe als Billboard zur Kamera: bleibt ein sauberes Quadrat
        ax, ay, _fw = self._basis
        for k, col in ((1.7, ui.mix(COL_SUN, COL_SKY_HOR, 0.6)), (1.0, COL_SUN)):
            q = tuple((sc[0] + ax[0] * r * k * sx + ay[0] * r * k * sy,
                       sc[1] + ax[1] * r * k * sx + ay[1] * r * k * sy,
                       sc[2] + ax[2] * r * k * sx + ay[2] * r * k * sy)
                      for sx, sy in ((-1, 1), (1, 1), (1, -1), (-1, -1)))
            self._add_poly(sky, q, col, 1.0, None, fog=False)

        # Wolkenfeld: 8x8-Maske, Kacheln zu 8 Bloecken, driftet langsam in x
        cyl = getattr(self, "_cloud_y", 50.0)
        if cy < cyl - 1.0:
            T = 7.0
            drift = (self.anim * 0.35) % (T * 8)
            tx0 = int(math.floor((cx - drift) / T))
            tz0 = int(math.floor(cz / T))
            span = 11
            for tz in range(tz0 - span, tz0 + span + 1):
                for tx in range(tx0 - span, tx0 + span + 1):
                    if _CLOUD_MAP[tz % 8][tx % 8] == "0":
                        continue
                    x0 = tx * T + drift
                    z0 = tz * T
                    mx, mz = x0 + T * 0.5, z0 + T * 0.5
                    dist = math.hypot(mx - cx, mz - cz)
                    if dist > span * T:
                        continue
                    # eigenes Cull: die Wolken liegen weit ausserhalb der
                    # Nebelgrenze, _cull() wuerde sie alle verwerfen
                    c = self._to_cam((mx, cyl, mz))
                    if c[2] < 0.5 or abs(c[0]) > c[2] * 2.0 + T:
                        continue
                    ft = min(1.0, (dist / (span * T)) ** 2.6)
                    col = ui.mix((250, 251, 255), COL_FOG, ft)
                    self._add_poly(sky, ((x0, cyl, z0), (x0 + T, cyl, z0),
                                         (x0 + T, cyl, z0 + T), (x0, cyl, z0 + T)),
                                   col, 0.94, None, fog=False)
        sky.sort(key=lambda it: -it[0])
        for _, quads, _o in sky:
            for pts, col in quads:
                pygame.draw.polygon(s, col, pts)

    def _apply_blur(self, s):
        if self.blur <= 0.01:
            self._prev_frame = None
            return
        if self._prev_frame is None or self._prev_frame.get_size() != s.get_size():
            self._prev_frame = s.copy()
            return
        self._prev_frame.set_alpha(int(self.blur * 230))
        s.blit(self._prev_frame, (0, 0))
        self._prev_frame = s.copy()

    # ----- Ego-Hand -------------------------------------------------------
    def _draw_hand(self, s):
        """Minecraft-Hand unten rechts, mit Lauf-Bob."""
        w, h = self.width, self.height
        sp = min(1.0, math.hypot(self.vx, self.vz) / MOVE_SPEED)
        bx = math.sin(self.walk) * 0.020 * w * sp
        by = abs(math.cos(self.walk)) * 0.030 * h * sp
        if not self.on_ground:
            by -= 0.02 * h
        ax, ay = w * 0.97 + bx, h * 1.16 + by      # Schulter (ausserhalb)
        fx, fy = w * 0.79 + bx, h * 0.78 + by      # Faust
        dx, dy = fx - ax, fy - ay
        ln = math.hypot(dx, dy) or 1.0
        dx, dy = dx / ln, dy / ln
        nx, ny = -dy, dx
        hw = h * 0.055                              # halbe Armbreite
        top = (fx, fy)

        def seg(t0, t1, tex, shade):
            p0 = (ax + dx * ln * t0, ay + dy * ln * t0)
            p1 = (ax + dx * ln * t1, ay + dy * ln * t1)
            quad = ((p1[0] + nx * hw, p1[1] + ny * hw),
                    (p1[0] - nx * hw, p1[1] - ny * hw),
                    (p0[0] - nx * hw, p0[1] - ny * hw),
                    (p0[0] + nx * hw, p0[1] + ny * hw))
            self._blit_tex_quad(s, quad, tex, shade)

        seg(0.0, 1.0, "arm", 0.98)
        # schmale Schattenkante fuer die Tiefe
        edge = ((fx - nx * hw, fy - ny * hw),
                (fx - nx * hw - dx * h * 0.02, fy - ny * hw - dy * h * 0.02),
                (ax - nx * hw - dx * h * 0.02, ay - ny * hw - dy * h * 0.02),
                (ax - nx * hw, ay - ny * hw))
        self._blit_tex_quad(s, edge, "arm", 0.62)
        # Handruecken (Stirnflaeche der Faust)
        cap = ((top[0] + nx * hw, top[1] + ny * hw),
               (top[0] - nx * hw, top[1] - ny * hw),
               (top[0] - nx * hw + dx * -h * 0.035, top[1] - ny * hw + dy * -h * 0.035),
               (top[0] + nx * hw + dx * -h * 0.035, top[1] + ny * hw + dy * -h * 0.035))
        self._blit_tex_quad(s, cap, "arm", 1.12)

    def _blit_tex_quad(self, s, quad, tex, shade=1.0, n=None):
        """Zeichnet eine Textur in ein 2D-Viereck (Bildschirmkoordinaten)."""
        if self.tex_mode == 0:
            pygame.draw.polygon(s, _shade_col(_avg_col(tex), shade), quad)
            return
        n = n or (8 if self.tex_mode == 2 else 4)
        n = min(n, len(TEX[tex]))
        a, b, c, d = quad
        left = [(a[0] + (d[0] - a[0]) * j / n, a[1] + (d[1] - a[1]) * j / n)
                for j in range(n + 1)]
        right = [(b[0] + (c[0] - b[0]) * j / n, b[1] + (c[1] - b[1]) * j / n)
                 for j in range(n + 1)]
        rows = [[(left[j][0] + (right[j][0] - left[j][0]) * i / n,
                  left[j][1] + (right[j][1] - left[j][1]) * i / n)
                 for i in range(n + 1)] for j in range(n + 1)]
        for i0, j0, i1, j1, col in _tex_grid(tex, n, shade, 0.0):
            r0, r1 = rows[j0], rows[j1]
            pygame.draw.polygon(s, col, (r0[i0], r0[i1], r1[i1], r1[i0]))

    # ----- HUD ------------------------------------------------------------
    def _draw_crosshair(self, s):
        cx, cy = self.width // 2, self.height // 2
        for col, off, ln, wd in (((26, 28, 34), 1, 10, 4), ((242, 242, 242), 0, 9, 2)):
            pygame.draw.line(s, col, (cx - ln, cy + off), (cx + ln, cy + off), wd)
            pygame.draw.line(s, col, (cx + off, cy - ln), (cx + off, cy + ln), wd)

    def _blit_art(self, s, art, pal, x, y, px):
        """Zeichnet Pixel-Art (Zeichenraster) mit px Pixel je Texel."""
        for j, row in enumerate(art):
            for i, ch in enumerate(row):
                c = pal.get(ch)
                if c:
                    pygame.draw.rect(s, c, (x + i * px, y + j * px, px, px))

    def _shadow_text(self, s, font, txt, col, pos, anchor="midleft"):
        """Minecraft-Schrift: harter dunkler Schatten unten rechts."""
        sh = font.render(txt, True, (28, 28, 32))
        img = font.render(txt, True, col)
        r = img.get_rect(**{anchor: pos})
        s.blit(sh, (r.x + 2, r.y + 2))
        s.blit(img, r)
        return r

    def _draw_hud(self, s):
        self._shadow_text(s, self._hud, t("blj.level", n=self.level), self.accent,
                          (14, 20))
        self._shadow_text(s, self._hud, t("common.points", score=self.score),
                          (245, 245, 245), (self.width // 2, 20), "center")
        # Herzen (Leben) unten links
        px = max(2, self.height // 230)
        hw = 10 * px
        y = self.height - 14 - 9 * px
        for i in range(3):
            self._blit_art(s, _HEART_ART,
                           _HEART_PAL if i < self.lives else _HEART_EMPTY_PAL,
                           14 + i * (hw + px * 2), y, px)
        # Gold-Barren + Zaehler unten rechts
        ing = _mip("ingot", 8)
        ix = self.width - 14 - 8 * px
        for j, row in enumerate(ing):
            for i, c in enumerate(row):
                if c:
                    pygame.draw.rect(s, c, (ix + i * px, y + j * px, px, px))
        self._shadow_text(s, self._hud, str(self.coins_level), ui.GOLD,
                          (ix - 8, y + 4 * px), "midright")
        if self.state == PLAY:
            mdir = t("common.dir_inverted") if self.invert else t("common.dir_normal")
            hint = t("blj.hud_hint",
                     view=(t("blj.view_1p") if self.view == "first" else t("blj.view_3p")),
                     tex=t("blj.tex_" + TEX_MODES[self.tex_mode]),
                     dir=mdir)
            lines = [hint]
            if self._small.size(hint)[0] > self.width - 28:
                parts = hint.split(" · ")           # bei schmalem Fenster 2-zeilig
                h = (len(parts) + 1) // 2
                lines = [" · ".join(parts[:h]), " · ".join(parts[h:])]
            y = self.height - 10 - (len(lines) - 1) * (self._small.get_height() + 2)
            for ln in lines:
                self._shadow_text(s, self._small, ln, (226, 226, 230),
                                  (self.width // 2, y), "midbottom")
                y += self._small.get_height() + 2

    def _banner(self, s, title, color, sub):
        w = min(self.width - 40, 620)
        h = 124
        rc = pygame.Rect((self.width - w) // 2, (self.height - h) // 2, w, h)
        panel = pygame.Surface((w, h), pygame.SRCALPHA)
        panel.fill((16, 16, 20, 226))
        s.blit(panel, rc.topleft)
        # Doppelrahmen im GUI-Stil von Minecraft
        pygame.draw.rect(s, (78, 78, 86), rc, 4)
        pygame.draw.rect(s, color, rc.inflate(-8, -8), 2)
        self._shadow_text(s, self._huge, title, color, (rc.centerx, rc.y + 44), "center")
        self._shadow_text(s, self._small, sub, (232, 232, 236),
                          (rc.centerx, rc.y + 88), "center")
