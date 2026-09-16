# -*- coding: utf-8 -*-
"""
geodash_core.py
===============
Der Physik-Kern von Geometry Dash - ohne pygame, ohne Zeichnen, ohne Zufall.

Alles, was über Leben und Tod entscheidet, steht hier: Spielformen (Würfel,
Schiff, Ball, UFO, Welle), Schwerkraft, Sprungpads, Orbs, Portale, Blöcke,
Stacheln, Gruben und Münzen. Das Spiel (geodash.py), der Editor, der Level-
Solver (devtools/build_geodash_levels.py) und der Test benutzen genau diesen
Code - ein Level, das der Solver schafft, ist also auch im Spiel schaffbar.

Bitgleich mit ``web/js/games/geodash_core.js``
----------------------------------------------
Die Browser-Fassung muss aus denselben Eingaben exakt denselben Zustand
berechnen (der Test vergleicht Zustands-Hashes). Deshalb gilt hier strikt:

- Nur GANZZAHLEN. Positionen und Tempo sind Festkomma-Werte: ein Block ist
  ``B`` = 36000 Einheiten breit, ein Schritt dauert 1/240 Sekunde.
- Nur ``+``, ``-``, ``*`` und Vergleiche. Keine Trigonometrie, kein ``%``,
  kein ``/``, kein ``round`` - deren Semantik unterscheidet sich zwischen
  Python und JavaScript (Rundung, negative Zahlen, Fließkomma).
- Die einzige Division ist ``fdiv`` (Spalte eines Punktes): abrundende
  Ganzzahl-Division durch eine positive Konstante. In JS ist das
  ``Math.floor(a / b)`` - für |a| < 2^40 exakt dasselbe wie Pythons ``//``.
- Reihenfolgen sind festgelegt: Objekte werden in Listen-Reihenfolge
  geprüft, Spalten von links nach rechts.

Zeit
----
Die Physik läuft in festen Schritten (240 pro Sekunde). Das Spiel sammelt die
vergangene Zeit in einem Akkumulator und rechnet so viele Schritte, wie
hineinpassen; jede Eingabe wird dem Schritt zugeordnet, in dem sie passiert
ist. So springt der Würfel bei 15 FPS genau dort ab, wo er es bei 144 FPS
täte.

Ein Level (JSON, siehe ``normalize_level``)::

    {"id": "lama-launch", "name": "Lama Launch", "speed": 1, "mode": 0,
     "length": 220, "bg": [40, 110, 255], "ground": [20, 70, 200],
     "music": "drive", "objects": [["block", 12, 0, 0], ["spike", 20, 0, 0],
     ["p_ship", 80, 3, 0, 10], ["color", 90, 0, 0, 0, 200, 60, 255, 8], ...]}

x/y sind Rasterfelder (Blöcke), y = 0 ist die erste Reihe über dem Boden.
"""

# ---------------------------------------------------------------------------
#  Maßstab und Zeit
# ---------------------------------------------------------------------------

B = 36000                    # Einheiten je Block (Festkomma)
HZ = 240                     # Physik-Schritte je Sekunde
HALF = B // 2

# Vorwärtstempo je Stufe in Einheiten/Schritt: 0,5x · 1x · 2x · 3x
# (8,4 · 10,4 · 12,96 · 15,6 Blöcke pro Sekunde, wie im Original).
SPEEDS = (1260, 1560, 1944, 2340)
SPEED_LABELS = ("0.5x", "1x", "2x", "3x")

# Spielformen
CUBE, SHIP, BALL, UFO, WAVE = 0, 1, 2, 3, 4
MODE_NAMES = ("cube", "ship", "ball", "ufo", "wave")

# Senkrechte Physik je Spielform (Einheiten/Schritt bzw. /Schritt^2).
GRAVITY = (66, 0, 52, 44, 0)
FALL_MAX = (4400, 0, 3800, 3000, 0)
CUBE_JUMP = 3300             # Würfel: ~2,25 Blöcke hoch, ~0,42 s in der Luft
UFO_JUMP = 2150              # UFO: ein Flügelschlag, ~1,4 Blöcke
BALL_PUSH = 1300             # Ball: Anstoß beim Umschalten der Schwerkraft
SHIP_UP = 40                 # Schiff: Auftrieb bei gehaltener Taste
SHIP_DOWN = 36               #         Absinken beim Loslassen
SHIP_MAX_UP = 1700
SHIP_MAX_DOWN = 1900

# Orbs (auf Klick) und Pads (bei Berührung): (gelb, pink, blau) je Spielform.
# Blau dreht die Schwerkraft und schubst zum neuen "Boden".
ORB_V = ((3450, 2500, 1500), (2400, 1700, 1200), (3000, 2200, 1500),
         (2600, 1900, 1400), (0, 0, 0))
PAD_V = ((4500, 2700, 2600), (2800, 1900, 1600), (3800, 2400, 2400),
         (3400, 2300, 2200), (0, 0, 0))

# Standardhöhe des Korridors (Boden bis Decke) für Schiff/Ball/UFO/Welle.
CORRIDOR = 10 * B

BUFFER = 20                  # Klick-Puffer: ~83 ms vor dem Orb zählt noch
SNAP = 9000                  # Toleranz (1/4 Block), um auf Kanten zu landen
INNER = 10800                # innere Hitbox (Tod in Blöcken): 0,3 Block
INNER_OFF = (B - INNER) // 2
WAVE_SIZE = 12000            # die Welle ist klein: 1/3 Block
WAVE_OFF = (B - WAVE_SIZE) // 2
FALL_DEATH = -2 * B          # tiefer = in die Grube gefallen
MAX_Y = 48 * B               # höher = davongeflogen

MAX_LENGTH = 4000            # Blöcke
MAX_ROW = 40
START_X = 0                  # Startposition (linke Kante) in Einheiten
MIN_LENGTH = 40


def fdiv(a, b):
    """Abrundende Ganzzahl-Division (b > 0) - JS: Math.floor(a / b)."""
    return a // b


# ---------------------------------------------------------------------------
#  Objektarten
# ---------------------------------------------------------------------------

# Klassen (bestimmen, was bei Berührung passiert)
C_SOLID, C_HAZARD, C_PIT, C_PAD, C_ORB, C_MODE, C_GRAV, C_SPEED, C_COIN, \
    C_TRIGGER = range(10)

# (name, klasse, wert, hitbox bei rot 0 in Blockbruchteilen x0,y0,x1,y1 *100)
# "wert": Spielform/Stufe/Farbe des Pads usw.
KINDS = (
    ("block", C_SOLID, 0, (0, 0, 100, 100)),
    ("half", C_SOLID, 0, (0, 0, 100, 50)),
    ("spike", C_HAZARD, 0, (40, 22, 60, 64)),
    ("spike_s", C_HAZARD, 0, (40, 4, 60, 32)),
    ("pit", C_PIT, 0, (0, 0, 100, 100)),
    ("pad_y", C_PAD, 0, (8, 0, 92, 22)),
    ("pad_p", C_PAD, 1, (8, 0, 92, 22)),
    ("pad_b", C_PAD, 2, (8, 0, 92, 22)),
    ("orb_y", C_ORB, 0, (-12, -12, 112, 112)),
    ("orb_p", C_ORB, 1, (-12, -12, 112, 112)),
    ("orb_b", C_ORB, 2, (-12, -12, 112, 112)),
    ("p_cube", C_MODE, CUBE, (15, -100, 85, 200)),
    ("p_ship", C_MODE, SHIP, (15, -100, 85, 200)),
    ("p_ball", C_MODE, BALL, (15, -100, 85, 200)),
    ("p_ufo", C_MODE, UFO, (15, -100, 85, 200)),
    ("p_wave", C_MODE, WAVE, (15, -100, 85, 200)),
    ("g_norm", C_GRAV, 1, (15, -100, 85, 200)),
    ("g_flip", C_GRAV, -1, (15, -100, 85, 200)),
    ("s_slow", C_SPEED, 0, (0, -100, 100, 200)),
    ("s_norm", C_SPEED, 1, (0, -100, 100, 200)),
    ("s_fast", C_SPEED, 2, (0, -100, 100, 200)),
    ("s_vfast", C_SPEED, 3, (0, -100, 100, 200)),
    ("coin", C_COIN, 0, (10, 10, 90, 90)),
    ("color", C_TRIGGER, 0, (0, 0, 0, 0)),
)
KIND_NAMES = tuple(k[0] for k in KINDS)
KIND_ID = {k[0]: i for i, k in enumerate(KINDS)}
# Welche Arten sich drehen lassen (0-3) - der Rest kennt nur 0 (oder 0/2).
ROTATE_ALL = ("half", "spike", "spike_s")
ROTATE_FLIP = ("pad_y", "pad_p", "pad_b")
# Zusatzwerte hinter [kind, x, y, rot]: Anzahl und Grenzen
#   Portale der Spielformen: Korridorhöhe in Blöcken (0 = Standard)
#   Farb-Trigger: Ziel (0 Hintergrund, 1 Boden), r, g, b, Dauer in Blöcken
PARAMS = {
    "p_ship": ((0, 0, 30),), "p_ball": ((0, 0, 30),),
    "p_ufo": ((0, 0, 30),), "p_wave": ((0, 0, 30),),
    "color": ((0, 0, 1), (255, 0, 255), (255, 0, 255), (255, 0, 255),
              (6, 0, 60)),
}
MAX_COINS = 3


def _rotate_box(box, rot):
    """Dreht eine Hitbox (Einheiten, relativ zur Zelle) rot-mal um 90° im
    Uhrzeigersinn um die Zellmitte: (x, y) -> (y, B - x)."""
    x0, y0, x1, y1 = box
    for _ in range(rot):
        x0, y0, x1, y1 = y0, B - x1, y1, B - x0
    return (x0, y0, x1, y1)


def _hitbox_table():
    out = []
    for name, _cls, _val, (a, b, c, d) in KINDS:
        base = (a * B // 100, b * B // 100, c * B // 100, d * B // 100)
        out.append(tuple(_rotate_box(base, r) for r in range(4)))
    return tuple(out)


HITBOX = _hitbox_table()


# ---------------------------------------------------------------------------
#  Level-Daten prüfen
# ---------------------------------------------------------------------------

DEFAULT_BG = (40, 110, 255)
DEFAULT_GROUND = (20, 70, 200)
MUSIC_STYLES = ("drive", "chip", "dream", "dark", "none")


def _int(v, lo, hi, default):
    try:
        if isinstance(v, bool):
            return default
        n = int(v)
    except (TypeError, ValueError):
        return default
    return max(lo, min(hi, n))


def _color(v, default):
    if isinstance(v, (list, tuple)) and len(v) >= 3:
        return [_int(v[0], 0, 255, 0), _int(v[1], 0, 255, 0),
                _int(v[2], 0, 255, 0)]
    return list(default)


def normalize_object(o):
    """Ein Objekt [kind, x, y, rot, ...] prüfen - None, wenn unbrauchbar."""
    if not isinstance(o, (list, tuple)) or len(o) < 3:
        return None
    kind = o[0]
    if kind not in KIND_ID:
        return None
    x = _int(o[1], 0, MAX_LENGTH, -1)
    y = _int(o[2], 0, MAX_ROW, -1)
    if x < 0 or y < 0:
        return None
    rot = _int(o[3] if len(o) > 3 else 0, 0, 3, 0)
    if kind in ROTATE_FLIP:
        rot = 2 if rot in (1, 2) else 0
    elif kind not in ROTATE_ALL:
        rot = 0
    if kind == "pit":
        y = 0
    out = [kind, x, y, rot]
    for i, (default, lo, hi) in enumerate(PARAMS.get(kind, ())):
        raw = o[4 + i] if len(o) > 4 + i else default
        out.append(_int(raw, lo, hi, default))
    return out


def sort_key(o):
    return (o[1], o[2], KIND_ID[o[0]], o[3])


def normalize_level(d):
    """Macht aus einem Level-Dict (JSON, Editor, Import) ein sauberes Dict.

    Unbekannte Objekte fallen heraus, Zahlen werden begrenzt, die Objekte
    stehen danach in fester Reihenfolge (x, y, Art) - die Physik prüft in
    dieser Reihenfolge, Python und JavaScript also gleich. Kopfdaten (id,
    name, author, verified ...) bleiben erhalten.
    """
    d = dict(d) if isinstance(d, dict) else {}
    objs = []
    coins = 0
    seen = set()
    for o in d.get("objects") or ():
        n = normalize_object(o)
        if n is None:
            continue
        if n[0] == "coin":
            if coins >= MAX_COINS:
                continue
            coins += 1
        key = tuple(n)
        if key in seen:                  # doppelt gesetzt: einmal reicht
            continue
        seen.add(key)
        objs.append(n)
    objs.sort(key=sort_key)
    d["objects"] = objs
    d["speed"] = _int(d.get("speed", 1), 0, 3, 1)
    d["mode"] = _int(d.get("mode", CUBE), 0, 4, CUBE)
    d["bg"] = _color(d.get("bg"), DEFAULT_BG)
    d["ground"] = _color(d.get("ground"), DEFAULT_GROUND)
    if d.get("music") not in MUSIC_STYLES:
        d["music"] = "drive"
    d["length"] = _int(d.get("length", 0), 0, MAX_LENGTH + 20, 0)
    return d


def level_length(d):
    """Länge in Blöcken: angegeben oder hinter dem letzten Objekt + Auslauf."""
    last = 0
    for o in d.get("objects") or ():
        if o[0] != "color":
            last = max(last, o[1])
    return max(MIN_LENGTH, int(d.get("length") or 0), last + 12)


def content_hash(d):
    """Prüfsumme über alles Spielrelevante (für das Verifiziert-Abzeichen).

    Ändert sich ein Objekt, das Starttempo oder die Startform, passt die
    Prüfsumme nicht mehr - das Level gilt dann wieder als unverifiziert.
    Farben, Name und Musik zählen nicht.
    """
    d = normalize_level(d)
    parts = ["%d|%d|%d" % (d["speed"], d["mode"], level_length(d))]
    for o in d["objects"]:
        if o[0] != "color":
            parts.append(",".join(str(v) for v in o))
    return "%08x" % fnv1a(";".join(parts))


def fnv1a(text):
    """32-Bit-FNV-1a über die Zeichencodes (nur ASCII verwenden)."""
    h = 0x811C9DC5
    for ch in text:
        h ^= ord(ch) & 0xFF
        h = (h * 0x01000193) & 0xFFFFFFFF
    return h


# ---------------------------------------------------------------------------
#  Kompiliertes Level
# ---------------------------------------------------------------------------

class Level:
    """Ein Level in der Form, die die Physik schnell abfragen kann.

    Jedes Objekt i hat: Art (kind[i]), Klasse (cls[i]), Wert (val[i]),
    Hitbox in Welt-Einheiten (box[i]) und Zusatzwerte (par[i]). ``cols``
    ordnet jeder Spalte die Objekte zu, deren Hitbox sie berührt.
    """

    __slots__ = ("data", "kind", "cls", "val", "box", "par", "ox", "oy",
                 "rot", "cols", "pits", "end_x", "length", "speed", "mode",
                 "coin_bit", "coin_count", "triggers", "n", "cache")

    def __init__(self, d):
        d = normalize_level(d)
        self.data = d
        self.kind, self.cls, self.val, self.box, self.par = [], [], [], [], []
        self.ox, self.oy, self.rot = [], [], []
        self.cols = {}
        self.pits = set()
        self.coin_bit = {}
        self.triggers = []
        self.cache = {}              # für Zeichen-Hilfen (nicht Physik)
        coins = 0
        for i, o in enumerate(d["objects"]):
            k = KIND_ID[o[0]]
            name, cls, val, _ = KINDS[k]
            cx, cy, rot = o[1] * B, o[2] * B, o[3]
            hb = HITBOX[k][rot]
            box = (cx + hb[0], cy + hb[1], cx + hb[2], cy + hb[3])
            self.kind.append(k)
            self.cls.append(cls)
            self.val.append(val)
            self.box.append(box)
            self.par.append(tuple(o[4:]))
            self.ox.append(o[1])
            self.oy.append(o[2])
            self.rot.append(rot)
            if cls == C_PIT:
                self.pits.add(o[1])
                continue
            if cls == C_TRIGGER:
                self.triggers.append(i)
                continue
            if cls == C_COIN:
                self.coin_bit[i] = 1 << coins
                coins += 1
            c0 = fdiv(box[0], B)
            c1 = fdiv(box[2] - 1, B)
            for c in range(c0, c1 + 1):
                lst = self.cols.get(c)
                if lst is None:
                    lst = []
                    self.cols[c] = lst
                lst.append(i)
        self.n = len(d["objects"])
        self.coin_count = coins
        self.length = level_length(d)
        self.end_x = self.length * B
        self.speed = d["speed"]
        self.mode = d["mode"]


# ---------------------------------------------------------------------------
#  Zustand
# ---------------------------------------------------------------------------

class State:
    """Alles, was einen Versuch ausmacht - flach, damit Kopieren billig ist.

    ``used`` ist ein frozenset der Objekte, die schon ausgelöst haben (Pads,
    Orbs, Portale, Münzen) - so feuert nichts doppelt, und ein Checkpoint
    ist eine einfache Kopie.
    """

    __slots__ = ("step", "x", "y", "vy", "mode", "grav", "speed", "ground",
                 "ceil", "buf", "dead", "won", "coins", "used")

    def copy(self):
        s = State.__new__(State)
        s.step, s.x, s.y, s.vy = self.step, self.x, self.y, self.vy
        s.mode, s.grav, s.speed = self.mode, self.grav, self.speed
        s.ground, s.ceil, s.buf = self.ground, self.ceil, self.buf
        s.dead, s.won, s.coins, s.used = self.dead, self.won, self.coins, self.used
        return s


def new_state(lv, start_block=None):
    """Startzustand - am Levelanfang oder (Editor: "ab Kamera") ab Block x.

    Beim Start mitten im Level gelten alle Portale davor als durchflogen:
    Spielform, Schwerkraft, Tempo und Korridor stimmen also. Der Spieler
    steht auf dem Boden bzw. dem Block-Stapel an dieser Stelle.
    """
    s = State()
    s.step = 0
    s.x = START_X
    s.y = 0
    s.vy = 0
    s.mode = lv.mode
    s.grav = 1
    s.speed = lv.speed
    s.ground = 1
    s.ceil = CORRIDOR if lv.mode != CUBE else 0
    s.buf = 0
    s.dead = 0
    s.won = 0
    s.coins = 0
    s.used = frozenset()
    if start_block is None or start_block <= 0:
        return s
    s.x = start_block * B
    for i in range(lv.n):
        if lv.ox[i] >= start_block:
            break
        cls = lv.cls[i]
        if cls == C_MODE:
            _set_mode(s, lv.val[i], lv.par[i])
        elif cls == C_GRAV:
            s.grav = lv.val[i]
        elif cls == C_SPEED:
            s.speed = lv.val[i]
    s.vy = 0
    if s.mode == CUBE or s.mode == BALL:
        if s.grav > 0:
            s.y = _stack_top(lv, start_block)
        else:
            s.y = max(0, _stack_bottom(lv, start_block) - B)
    else:
        s.y = (s.ceil - B) // 2 if s.ceil > 0 else 3 * B
        s.ground = 0
    return s


def _stack_top(lv, col):
    """Oberkante des höchsten zusammenhängenden Stapels ab dem Boden."""
    top = 0
    changed = True
    while changed:
        changed = False
        for i in lv.cols.get(col, ()):
            if lv.cls[i] == C_SOLID and lv.box[i][1] <= top < lv.box[i][3]:
                top = lv.box[i][3]
                changed = True
    return top


def _stack_bottom(lv, col):
    best = MAX_Y
    for i in lv.cols.get(col, ()):
        if lv.cls[i] == C_SOLID and lv.box[i][1] >= B and lv.box[i][1] < best:
            best = lv.box[i][1]
    return best if best < MAX_Y else 6 * B


def _set_mode(s, mode, par):
    """Spielform wechseln (Portal) - samt Korridor und Tempo-Grenzen."""
    s.mode = mode
    if mode == CUBE:
        s.ceil = 0
    else:
        h = par[0] if par else 0
        s.ceil = h * B if h > 0 else CORRIDOR
    if mode == SHIP:
        v = s.vy * s.grav
        if v > SHIP_MAX_UP:
            v = SHIP_MAX_UP
        if v < -SHIP_MAX_DOWN:
            v = -SHIP_MAX_DOWN
        s.vy = v * s.grav
    s.ground = 0


# ---------------------------------------------------------------------------
#  Ein Physik-Schritt
# ---------------------------------------------------------------------------

def _floor_under(lv, x0, x1):
    """Trägt der Boden zwischen x0 und x1? (Nur ganz über Gruben nicht.)"""
    if not lv.pits:
        return True
    for c in range(fdiv(x0, B), fdiv(x1 - 1, B) + 1):
        if c not in lv.pits:
            return True
    return False


def _first_orb(s, lv, x0, y0, x1, y1):
    """Erstes noch unbenutztes Orb, das die Box berührt (oder -1)."""
    for c in range(fdiv(x0, B), fdiv(x1 - 1, B) + 1):
        for i in lv.cols.get(c, ()):
            if lv.cls[i] != C_ORB or i in s.used:
                continue
            b = lv.box[i]
            if b[0] < x1 and b[2] > x0 and b[1] < y1 and b[3] > y0:
                return i
    return -1


def _use(s, i):
    s.used = s.used | {i}


def step(s, lv, held, pressed):
    """Rechnet genau einen Schritt (1/240 s).

    held    : Taste/Maus gerade gedrückt?
    pressed : in diesem Schritt NEU gedrückt (steigende Flanke)?
    """
    if s.dead or s.won:
        return
    s.step += 1
    if pressed:
        s.buf = BUFFER
    elif not held:
        s.buf = 0
    mode = s.mode
    size = WAVE_SIZE if mode == WAVE else B
    off = WAVE_OFF if mode == WAVE else 0

    # --- Klick: Orb, UFO-Flügelschlag, Ball-Umschalten ---------------------
    if s.buf > 0:
        oi = _first_orb(s, lv, s.x + off, s.y + off, s.x + off + size,
                        s.y + off + size)
        if oi >= 0:
            _use(s, oi)
            which = lv.val[oi]
            if which == 2:
                s.grav = -s.grav
                s.vy = -s.grav * ORB_V[mode][2]
            elif mode != WAVE:
                s.vy = s.grav * ORB_V[mode][which]
            s.ground = 0
            s.buf = 0
        elif mode == UFO:
            s.vy = s.grav * UFO_JUMP
            s.ground = 0
            s.buf = 0
        elif mode == BALL and s.ground:
            s.grav = -s.grav
            s.vy = -s.grav * BALL_PUSH
            s.ground = 0
            s.buf = 0
    g = s.grav
    if mode == CUBE and s.ground and (held or pressed):
        s.vy = CUBE_JUMP * g
        s.ground = 0

    # --- Senkrechte Bewegung (v = Tempo gegen die Schwerkraft) -------------
    v = s.vy * g
    if mode == SHIP:
        v += SHIP_UP if held else -SHIP_DOWN
        if v > SHIP_MAX_UP:
            v = SHIP_MAX_UP
        if v < -SHIP_MAX_DOWN:
            v = -SHIP_MAX_DOWN
    elif mode == WAVE:
        v = SPEEDS[s.speed] if held else -SPEEDS[s.speed]
    else:
        v -= GRAVITY[mode]
        if v < -FALL_MAX[mode]:
            v = -FALL_MAX[mode]
    s.vy = v * g
    prev0 = s.y + off
    prev1 = prev0 + size
    s.y += s.vy
    s.x += SPEEDS[s.speed]
    s.ground = 0
    px0 = s.x + off
    px1 = px0 + size

    # --- Boden (außer über Gruben) ----------------------------------------
    py0 = s.y + off
    if py0 < 0 and _floor_under(lv, px0, px1):
        if prev0 >= -SNAP:
            s.y = -off
            if s.vy < 0:
                s.vy = 0
            if g > 0:
                s.ground = 1
        else:
            s.dead = 1                   # seitlich gegen die Bodenkante
            return
    # --- Decke des Korridors -----------------------------------------------
    if s.ceil > 0 and s.y + off + size > s.ceil:
        s.y = s.ceil - size - off
        if s.vy > 0:
            s.vy = 0
        if g < 0:
            s.ground = 1

    # --- Blöcke --------------------------------------------------------------
    py0 = s.y + off
    py1 = py0 + size
    c0 = fdiv(px0, B)
    c1 = fdiv(px1 - 1, B)
    land = None
    bump = None
    hit = False
    for c in range(c0, c1 + 1):
        for i in lv.cols.get(c, ()):
            if lv.cls[i] != C_SOLID:
                continue
            b = lv.box[i]
            if not (b[0] < px1 and b[2] > px0 and b[1] < py1 and b[3] > py0):
                continue
            if mode == WAVE:
                s.dead = 1
                return
            hit = True
            if g > 0:
                if s.vy <= 0 and prev0 >= b[3] - SNAP:
                    if land is None or b[3] > land:
                        land = b[3]
                elif mode != CUBE and s.vy > 0 and prev1 <= b[1] + SNAP:
                    if bump is None or b[1] < bump:
                        bump = b[1]
            else:
                if s.vy >= 0 and prev1 <= b[1] + SNAP:
                    if land is None or b[1] < land:
                        land = b[1]
                elif mode != CUBE and s.vy < 0 and prev0 >= b[3] - SNAP:
                    if bump is None or b[3] > bump:
                        bump = b[3]
    if hit:
        if land is not None:
            s.y = land if g > 0 else land - B
            s.vy = 0
            s.ground = 1
        if bump is not None:
            s.y = bump - B if g > 0 else bump
            s.vy = 0
        # Tod nur, wenn die INNERE Hitbox im Block steckt (wie im Original:
        # Kanten streifen ist erlaubt, frontal hineinfahren nicht).
        ix0 = s.x + INNER_OFF
        ix1 = ix0 + INNER
        iy0 = s.y + INNER_OFF
        iy1 = iy0 + INNER
        for c in range(fdiv(ix0, B), fdiv(ix1 - 1, B) + 1):
            for i in lv.cols.get(c, ()):
                if lv.cls[i] != C_SOLID:
                    continue
                b = lv.box[i]
                if b[0] < ix1 and b[2] > ix0 and b[1] < iy1 and b[3] > iy0:
                    s.dead = 1
                    return

    # --- Auslöser und Gefahren -------------------------------------------------
    py0 = s.y + off
    py1 = py0 + size
    for c in range(fdiv(px0, B), fdiv(px1 - 1, B) + 1):
        for i in lv.cols.get(c, ()):
            cls = lv.cls[i]
            if cls == C_SOLID:
                continue
            b = lv.box[i]
            if not (b[0] < px1 and b[2] > px0 and b[1] < py1 and b[3] > py0):
                continue
            if cls == C_HAZARD:
                s.dead = 1
                return
            if cls == C_ORB or i in s.used:
                continue
            if cls == C_PAD:
                _use(s, i)
                which = lv.val[i]
                if which == 2:
                    s.grav = -s.grav
                    s.vy = -s.grav * PAD_V[s.mode][2]
                elif s.mode != WAVE:
                    s.vy = s.grav * PAD_V[s.mode][which]
                s.ground = 0
            elif cls == C_MODE:
                _use(s, i)
                if lv.val[i] != s.mode:
                    _set_mode(s, lv.val[i], lv.par[i])
                else:
                    _set_mode(s, s.mode, lv.par[i])
            elif cls == C_GRAV:
                _use(s, i)
                if lv.val[i] != s.grav:
                    s.grav = lv.val[i]
                    s.vy = 0
                    s.ground = 0
            elif cls == C_SPEED:
                _use(s, i)
                s.speed = lv.val[i]
            elif cls == C_COIN:
                _use(s, i)
                s.coins |= lv.coin_bit[i]

    if s.y < FALL_DEATH or s.y > MAX_Y:
        s.dead = 1
        return
    if s.x >= lv.end_x:
        s.won = 1
    if s.buf > 0:
        s.buf -= 1


# ---------------------------------------------------------------------------
#  Hilfen für Spiel, Solver und Test
# ---------------------------------------------------------------------------

def progress(s, lv):
    """Fortschritt in ganzen Prozent (0-100)."""
    if s.won:
        return 100
    return max(0, min(99, (s.x - START_X) * 100 // (lv.end_x - START_X)))


def state_str(s):
    """Kanonische Textform des Zustands (identisch zur JS-Fassung)."""
    return "%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%s" % (
        s.step, s.x, s.y, s.vy, s.mode, s.grav, s.speed, s.ground, s.ceil,
        s.buf, s.dead, s.won, s.coins, ".".join(str(i) for i in sorted(s.used)))


def state_hash(s):
    return "%08x" % fnv1a(state_str(s))


def held_at(toggles, k):
    """Ist die Taste in Schritt k gedrückt? (toggles: sortierte Umschalt-Schritte)"""
    n = 0
    for t in toggles:
        if t <= k:
            n += 1
        else:
            break
    return n & 1 == 1


def run_toggles(lv, toggles, max_steps=None, trace_every=0, start=None):
    """Spielt eine Eingabefolge ab: die Taste wechselt in den Schritten aus
    'toggles' (0-basiert) zwischen losgelassen und gedrückt.

    Gibt (endzustand, [(schritt, hash), ...]) zurück - die Zwischen-Hashes
    alle 'trace_every' Schritte vergleicht der Test mit Node.
    """
    s = start.copy() if start is not None else new_state(lv)
    limit = max_steps if max_steps is not None else \
        (lv.end_x // SPEEDS[0]) + 4 * HZ
    toggles = sorted(toggles)
    ti = 0
    held = False
    trace = []
    k = 0
    while k < limit and not s.dead and not s.won:
        was = held
        while ti < len(toggles) and toggles[ti] <= k:
            held = not held
            ti += 1
        step(s, lv, held, held and not was)
        k += 1
        if trace_every and k % trace_every == 0:
            trace.append((k, state_hash(s)))
    trace.append((k, state_hash(s)))
    return s, trace
