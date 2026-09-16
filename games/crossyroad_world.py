# -*- coding: utf-8 -*-
"""
crossyroad_world.py
===================
Die Welt von Crossy Road - ganz ohne Grafik:

- Reihen-Generator: endlose Reihen aus Wiese (Bäume/Steine), Straße (Autos/
  Laster), Fluss (Stämme oder Seerosen) und Gleisen (Züge). Er verwendet
  ``seedrand`` und verbraucht die Zufallszahlen in exakt derselben Reihenfolge
  wie ``web/js/games/crossyroad_world.js`` - die Tagesstrecke ist am PC und im
  Browser Reihe für Reihe identisch. Deshalb hier: keine Mengen (Reihenfolge!),
  nur int()/floor auf positiven Zahlen, ganzzahlige Gewichte.
- Garantierter Weg: Jede Reihe merkt sich, welche Spalten man (vorwärts +
  seitwärts) erreichen kann. Wäre das nichts, räumt der Generator gezielt
  einen Baum weg bzw. legt eine Seerose dazu.
- Bewegungsformeln: Fahrzeuge, Stämme und Züge sind reine Funktionen der
  Spielzeit (keine aufsummierten Positionen) - ruckelfrei und reproduzierbar.
- Figuren-Katalog und alle Voxel-Modelle als Quader-Listen
  (x0, y0, z0, x1, y1, z1, farbe) in Kacheleinheiten: x = Spalte (nach
  rechts), y = Reihe (nach vorn/oben), z = Höhe. Figuren blicken nach +y.
  ``devtools/build_crossyroad_models.py`` schreibt dieselben Daten für das
  Web nach ``web/js/games/crossyroad_models.js``.
"""

import seedrand

# ----- Spielfeld ---------------------------------------------------------------
COLS = 9                  # begehbare Spalten 0..8
START_COL = 4             # Startspalte (Mitte)
MARGIN = 8                # Welt links/rechts außerhalb (Fahrzeuge fahren ein/aus)
LOOP = COLS + 2 * MARGIN  # Umlauflänge einer Spur
START_ROWS = 3            # Reihen 0..2: sichere Startwiese
DIFF_ROWS = 160.0         # ab hier volle Schwierigkeit

GRASS, ROAD, RIVER, RAIL = "grass", "road", "river", "rail"
KINDS = (GRASS, ROAD, RIVER, RAIL)
# Bodenhöhe je Reihenart (Kacheleinheiten): Wiese erhöht, Wasser tiefer.
LEVEL = {GRASS: 0.12, ROAD: 0.0, RAIL: 0.04, RIVER: -0.12}

CAR_LEN = 1.4
TRUCK_LEN = 2.5
N_CAR_COLORS = 6
TRAIN_CAR = 3.0           # Länge je Wagen (Lok ebenfalls)
TRAIN_SPEED = 24.0        # Spalten pro Sekunde
TRAIN_WARN = 1.0          # Sekunden Warnlicht, bevor die Lok einfährt
BIG_COIN = 5              # Wert der Riesenmünze

# ----- Figur & Regeln ------------------------------------------------------------
HOP_T = 0.13              # Dauer eines Hüpfers (s)
HOP_Z = 0.42              # Scheitelhöhe eines Hüpfers
CAR_HALF = 0.28           # halbe Trefferbreite der Figur gegen Fahrzeuge
LOG_GRACE = 0.3           # so weit neben einem Stamm zählt ein Sprung noch
EAGLE_ROWS = 3.2          # so weit hinter der Kamera holt der Adler
CREEP_LAG = 1.0           # Vorsprung der Kamera-Kriechgrenze nach einem Schritt
CREEP_BASE = 0.45         # Kriechtempo der Kamera (Reihen/s) ...
CREEP_GROW = 0.0016       # ... wächst je erreichter Reihe ...
CREEP_MAX = 0.8           # ... bis höchstens hier hin
NIGHT_FROM = 50           # erster Sonnenuntergang ab dieser Reihe
NIGHT_CYCLE = 90          # Reihen je Tag/Nacht-Zyklus

DIRS = {"up": (0, 1), "down": (0, -1), "left": (-1, 0), "right": (1, 0)}

# Figuren: (id, Preis in Münzen). Das Huhn gehört allen von Anfang an.
CHARACTERS = [
    ("chicken", 0), ("frog", 25), ("pig", 40), ("penguin", 60), ("cat", 80),
    ("fox", 100), ("llama", 125), ("robot", 150), ("ghost", 200),
    ("unicorn", 250),
]
CHAR_IDS = [c for c, _ in CHARACTERS]
PRICES = dict(CHARACTERS)
HOVER = {"ghost"}         # schwebt über dem Boden


def fmod(a, b):
    """Python-Modulo für Gleitkommazahlen (Ergebnis hat das Vorzeichen von b)."""
    return a % b


def difficulty(r):
    """0.0 am Start bis 1.0 ab Reihe DIFF_ROWS."""
    return min(1.0, r / DIFF_ROWS)


def _spread(entry, ok):
    """Seitliche Hülle: alle Spalten, die von 'entry' aus über erlaubte
    Nachbarfelder erreichbar sind (sortiert)."""
    seen = [False] * COLS
    stack = []
    for c in entry:
        if not seen[c]:
            seen[c] = True
            stack.append(c)
    while stack:
        c = stack.pop()
        for n in (c - 1, c + 1):
            if 0 <= n < COLS and not seen[n] and ok(n):
                seen[n] = True
                stack.append(n)
    return [c for c in range(COLS) if seen[c]]


def _blank(r, kind):
    """Reihe mit allen Feldern (gleiche Form wie in der JS-Fassung)."""
    return {"r": r, "kind": kind, "trees": None, "pads": None, "dir": 0,
            "speed": 0.0, "objs": [], "period": 0.0, "phase": 0.0,
            "cars": 0, "coin": None, "reach": list(range(COLS))}


def back_row(r):
    """Reihen hinter dem Start (r < 0): Wiese, weiter hinten eine Baumwand."""
    row = _blank(r, GRASS)
    trees = [0] * COLS
    for c in range(COLS):
        h = deco_hash(r, c)
        wall = r == -3 or (r == -2 and c in (0, 1, 7, 8)) or (r == -1 and c in (0, 8))
        if wall or (r < -3 and h % 100 < 55):
            trees[c] = 1 + ((h >> 8) % 3)
    row["trees"] = trees
    row["reach"] = [c for c in range(COLS) if trees[c] == 0]
    return row


def deco_hash(r, c):
    """Kleiner 32-Bit-Hash für reine Deko (identisch in JS)."""
    h = (r * 374761393 + c * 668265263) & 0xFFFFFFFF
    h = ((h ^ (h >> 13)) * 1274126177) & 0xFFFFFFFF
    return (h ^ (h >> 16)) & 0xFFFFFFFF


class World:
    """Endlose, aus einem Seed erzeugte Strecke."""

    def __init__(self, seed):
        self.seed = int(seed) & 0xFFFFFFFF
        self.rng = seedrand.Rand(self.seed)
        self.rows = {}
        self.next_row = 0
        self.prev_reach = [START_COL]
        self.last_kind = GRASS
        self.last_dir = 1
        self.last_log = False

    # ------------------------------------------------------------ Zugriff
    def row(self, r):
        if r < 0:
            got = self.rows.get(r)
            if got is None:
                got = self.rows[r] = back_row(r)
            return got
        while r >= self.next_row:
            self._zone()
        return self.rows[r]

    def kind(self, r):
        return self.row(r)["kind"]

    def rail_run(self, r):
        """Anzahl Gleise direkt hintereinander, die in Reihe r enden."""
        n = 0
        while r >= 0 and self.row(r)["kind"] == RAIL:
            n += 1
            r -= 1
        return n

    # ------------------------------------------------------------ Generator
    def _push(self, row):
        self.rows[row["r"]] = row
        self.prev_reach = row["reach"]
        self.next_row = row["r"] + 1
        if row["dir"]:
            self.last_dir = row["dir"]
        self.last_log = row["kind"] == RIVER and row["pads"] is None

    def _zone(self):
        rng = self.rng
        r0 = self.next_row
        if r0 == 0:
            for r in range(START_ROWS):
                self._grass(r, True)
            self.last_kind = GRASS
            return
        d = difficulty(r0)
        # Gewichte grass/road/river/rail - nach einer Gefahrenzone meist eine
        # Wiese zum Durchatmen, nie zweimal dieselbe Zonenart hintereinander.
        if self.last_kind == GRASS:
            w = [0, 46 + int(10 * d), 30 + int(8 * d), 12 + int(10 * d)]
        else:
            w = [58, 20 + int(10 * d), 14 + int(8 * d), 6 + int(8 * d)]
            w[KINDS.index(self.last_kind)] = 0
        if r0 < 10:
            w[3] = 0                      # erste Gleise erst nach ein paar Reihen
        kind = KINDS[rng.weighted(w)]
        early = r0 < 14                   # die ersten Zonen bleiben kurz
        if kind == GRASS:
            n = rng.randint(1, 3 - int(d * 1.5))
        elif kind == ROAD:
            n = rng.randint(1, 2 if early else 2 + int(3 * d))
        elif kind == RIVER:
            n = rng.randint(1, 2 if early else 2 + int(2 * d))
        else:
            n = 1 + rng.weighted([10, 6, int(6 * d), int(4 * d), int(4 * d)])
        for i in range(n):
            r = self.next_row
            if kind == GRASS:
                self._grass(r, False)
            elif kind == ROAD:
                self._road(r, i)
            elif kind == RIVER:
                self._river(r)
            else:
                self._rail(r)
        self.last_kind = kind

    def _coin(self, row, p):
        """Mit Wahrscheinlichkeit p eine Münze auf ein erreichbares Feld."""
        rng = self.rng
        if p > 0 and rng.random() < p:
            c = rng.choice(row["reach"])
            big = row["r"] >= 20 and rng.random() < 0.1
            row["coin"] = [c, BIG_COIN if big else 1]

    def _grass(self, r, start):
        rng = self.rng
        row = _blank(r, GRASS)
        trees = [0] * COLS
        if r > 0:
            dens = 0.12 if start else 0.12 + 0.18 * difficulty(r)
            for c in range(COLS):
                if rng.random() < dens:
                    trees[c] = rng.randint(1, 4)       # 1-3 Bäume, 4 = Stein
            if start:
                trees[START_COL] = 0
        entry = [c for c in self.prev_reach if trees[c] == 0]
        if not entry:
            c = rng.choice(self.prev_reach)
            trees[c] = 0
            entry = [c]
        row["trees"] = trees
        row["reach"] = _spread(entry, lambda c: trees[c] == 0)
        self._coin(row, 0.3 if r > 0 else 0.0)
        self._push(row)

    def _road(self, r, i):
        rng = self.rng
        d = difficulty(r)
        row = _blank(r, ROAD)
        if i > 0 and rng.random() < 0.7:
            direction = -self.last_dir
        else:
            direction = 1 if rng.random() < 0.5 else -1
        row["dir"] = direction
        row["speed"] = (1.3 + rng.random() * 1.4) * (1.0 + 0.9 * d)
        ln = TRUCK_LEN if rng.random() < 0.28 else CAR_LEN
        objs = []
        gap_min = 2.0 + 1.2 * (1.0 - d)
        x0 = x = rng.random() * 2.0
        while True:
            objs.append([x, ln, rng.randint(0, N_CAR_COLORS - 1)])
            x += ln + gap_min + rng.random() * (3.4 - 1.4 * d)
            if x + ln + gap_min > LOOP + x0:
                break
        row["objs"] = objs
        self._coin(row, 0.1)
        self._push(row)

    def _river(self, r):
        rng = self.rng
        d = difficulty(r)
        row = _blank(r, RIVER)
        if rng.random() < 0.2:
            pads = [1 if rng.random() < 0.45 else 0 for _ in range(COLS)]
            entry = [c for c in self.prev_reach if pads[c]]
            if not entry:
                c = rng.choice(self.prev_reach)
                pads[c] = 1
                entry = [c]
            row["pads"] = pads
            row["reach"] = _spread(entry, lambda c: pads[c] == 1)
            self._coin(row, 0.15)
        else:
            if self.last_log and rng.random() < 0.8:
                direction = -self.last_dir
            else:
                direction = 1 if rng.random() < 0.5 else -1
            row["dir"] = direction
            row["speed"] = (0.8 + rng.random() * 0.9) * (1.0 + 0.5 * d)
            lmax = 4 if d < 0.6 else 3
            objs = []
            x0 = x = rng.random() * 2.0
            while True:
                ln = rng.randint(2, lmax)
                room = LOOP + x0 - 1.0 - x
                if room < 2.0:
                    break
                ln = min(ln, int(room))
                objs.append([x, ln, 0])
                x += ln + 1.0 + rng.random() * (1.2 + 1.6 * d)
            row["objs"] = objs
        self._push(row)

    def _rail(self, r):
        rng = self.rng
        d = difficulty(r)
        row = _blank(r, RAIL)
        row["dir"] = 1 if rng.random() < 0.5 else -1
        period = 4.6 + rng.random() * 4.0 - 1.2 * d
        row["period"] = period
        row["phase"] = rng.random() * period
        row["cars"] = rng.randint(2, 4)
        self._coin(row, 0.08)
        self._push(row)

    # ------------------------------------------------------------ Export
    def export(self, n):
        """Die ersten n Reihen als JSON-taugliche Liste (für den Py/JS-Vergleich)."""
        out = []
        for r in range(n):
            row = self.row(r)
            out.append({k: row[k] for k in ("r", "kind", "trees", "pads", "dir",
                                             "speed", "objs", "period", "phase",
                                             "cars", "coin", "reach")})
        return out


# ----- Bewegung ------------------------------------------------------------------
def obj_x(row, obj, t):
    """Linke Kante eines Fahrzeugs/Stamms zur Zeit t (Weltspalten)."""
    return fmod(obj[0] + row["dir"] * row["speed"] * t, LOOP) - MARGIN


def train_state(row, t):
    """(warnen, lok_spitze oder None, zuglänge) eines Gleises zur Zeit t."""
    length = (row["cars"] + 1) * TRAIN_CAR
    u = fmod(t + row["phase"], row["period"])
    if u < TRAIN_WARN:
        return True, None, length
    s = u - TRAIN_WARN
    if s * TRAIN_SPEED > LOOP + length:
        return False, None, length
    dist = s * TRAIN_SPEED
    if row["dir"] > 0:
        head = -MARGIN + dist
    else:
        head = COLS + MARGIN - dist
    return True, head, length


def train_span(row, t):
    """(links, rechts) des Zuges oder None."""
    warn, head, length = train_state(row, t)
    if head is None:
        return None
    if row["dir"] > 0:
        return head - length, head
    return head, head + length


def vehicle_at(row, cx, t, half=CAR_HALF):
    """Index des Fahrzeugs, das die Figur (Mitte cx) berührt, sonst -1."""
    for i, o in enumerate(row["objs"]):
        x = obj_x(row, o, t)
        if x + 0.08 < cx + half and cx - half < x + o[1] - 0.08:
            return i
    return -1


def train_hits(row, cx, t, half=CAR_HALF):
    span = train_span(row, t)
    return span is not None and span[0] < cx + half and cx - half < span[1]


def log_at(row, cx, t, grace=LOG_GRACE):
    """(index, segment) des Stamms unter der Figurmitte cx, sonst None."""
    for i, o in enumerate(row["objs"]):
        x = obj_x(row, o, t)
        if x - grace <= cx <= x + o[1] + grace:
            seg = int(cx - x) if cx > x else 0
            seg = max(0, min(int(o[1]) - 1, seg))
            return i, seg
    return None


def creep_rate(best):
    return min(CREEP_MAX, CREEP_BASE + CREEP_GROW * max(0, best))


def night_factor(rowf):
    """0 = Tag, 1 = tiefe Nacht - abhängig von der (Kamera-)Reihe."""
    if rowf < NIGHT_FROM:
        return 0.0
    u = fmod(rowf - NIGHT_FROM, NIGHT_CYCLE)
    if u < 12:
        v = u / 12.0
    elif u < 50:
        v = 1.0
    elif u < 62:
        v = 1.0 - (u - 50) / 12.0
    else:
        v = 0.0
    return v * v * (3 - 2 * v)


# =================================================================== Modelle
# Farben
_W = (246, 246, 246)
_WD = (214, 216, 224)
_K = (34, 34, 42)
_RED = (232, 64, 64)
_OR = (246, 162, 40)
_GLASS = (160, 214, 240)
CAR_COLORS = [(232, 70, 70), (70, 130, 232), (246, 196, 58), (162, 100, 222),
              (242, 140, 48), (58, 190, 168)]
TREE_LEAVES = [(104, 186, 74), (84, 164, 66), (120, 196, 88)]


def _b(x0, y0, z0, x1, y1, z1, col):
    return (x0, y0, z0, x1, y1, z1, tuple(col))


def _chicken():
    return [
        _b(0.36, 0.42, 0.00, 0.44, 0.50, 0.14, _OR), _b(0.56, 0.42, 0.00, 0.64, 0.50, 0.14, _OR),
        _b(0.26, 0.22, 0.14, 0.74, 0.72, 0.56, _W),
        _b(0.20, 0.30, 0.24, 0.26, 0.62, 0.46, _WD), _b(0.74, 0.30, 0.24, 0.80, 0.62, 0.46, _WD),
        _b(0.36, 0.14, 0.36, 0.64, 0.22, 0.62, _WD),
        _b(0.32, 0.46, 0.56, 0.68, 0.76, 0.90, _W),
        _b(0.44, 0.52, 0.90, 0.56, 0.72, 1.02, _RED),
        _b(0.44, 0.76, 0.70, 0.56, 0.88, 0.78, _OR),
        _b(0.46, 0.76, 0.60, 0.54, 0.82, 0.70, _RED),
        _b(0.30, 0.64, 0.74, 0.32, 0.70, 0.80, _K), _b(0.68, 0.64, 0.74, 0.70, 0.70, 0.80, _K),
        _b(0.38, 0.76, 0.76, 0.42, 0.77, 0.82, _K), _b(0.58, 0.76, 0.76, 0.62, 0.77, 0.82, _K),
    ]


def _frog():
    g, gd, y, wt = (98, 198, 82), (70, 158, 64), (226, 238, 168), (250, 250, 250)
    return [
        _b(0.18, 0.18, 0.00, 0.34, 0.44, 0.14, gd), _b(0.66, 0.18, 0.00, 0.82, 0.44, 0.14, gd),
        _b(0.24, 0.62, 0.00, 0.36, 0.78, 0.10, gd), _b(0.64, 0.62, 0.00, 0.76, 0.78, 0.10, gd),
        _b(0.26, 0.22, 0.04, 0.74, 0.80, 0.40, g),
        _b(0.30, 0.80, 0.06, 0.70, 0.83, 0.24, y),
        _b(0.26, 0.58, 0.40, 0.44, 0.76, 0.56, wt), _b(0.56, 0.58, 0.40, 0.74, 0.76, 0.56, wt),
        _b(0.31, 0.76, 0.44, 0.39, 0.78, 0.52, _K), _b(0.61, 0.76, 0.44, 0.69, 0.78, 0.52, _K),
        _b(0.31, 0.62, 0.56, 0.39, 0.70, 0.58, _K), _b(0.61, 0.62, 0.56, 0.69, 0.70, 0.58, _K),
    ]


def _pig():
    p, pd = (248, 172, 186), (228, 140, 158)
    return [
        _b(0.28, 0.22, 0.00, 0.38, 0.32, 0.16, pd), _b(0.62, 0.22, 0.00, 0.72, 0.32, 0.16, pd),
        _b(0.28, 0.62, 0.00, 0.38, 0.72, 0.16, pd), _b(0.62, 0.62, 0.00, 0.72, 0.72, 0.16, pd),
        _b(0.24, 0.16, 0.16, 0.76, 0.80, 0.62, p),
        _b(0.38, 0.80, 0.28, 0.62, 0.90, 0.46, pd),
        _b(0.43, 0.90, 0.35, 0.48, 0.91, 0.40, _K), _b(0.52, 0.90, 0.35, 0.57, 0.91, 0.40, _K),
        _b(0.26, 0.60, 0.62, 0.38, 0.72, 0.74, pd), _b(0.62, 0.60, 0.62, 0.74, 0.72, 0.74, pd),
        _b(0.32, 0.80, 0.50, 0.38, 0.81, 0.56, _K), _b(0.62, 0.80, 0.50, 0.68, 0.81, 0.56, _K),
        _b(0.46, 0.10, 0.44, 0.54, 0.16, 0.54, pd),
    ]


def _penguin():
    bl = (44, 48, 62)
    return [
        _b(0.32, 0.46, 0.00, 0.46, 0.66, 0.06, _OR), _b(0.54, 0.46, 0.00, 0.68, 0.66, 0.06, _OR),
        _b(0.28, 0.30, 0.06, 0.72, 0.70, 0.82, bl),
        _b(0.34, 0.70, 0.10, 0.66, 0.73, 0.62, _W),
        _b(0.22, 0.38, 0.30, 0.28, 0.62, 0.62, bl), _b(0.72, 0.38, 0.30, 0.78, 0.62, 0.62, bl),
        _b(0.44, 0.73, 0.60, 0.56, 0.84, 0.68, _OR),
        _b(0.34, 0.70, 0.68, 0.43, 0.73, 0.77, _W), _b(0.57, 0.70, 0.68, 0.66, 0.73, 0.77, _W),
        _b(0.37, 0.73, 0.70, 0.41, 0.74, 0.75, _K), _b(0.59, 0.73, 0.70, 0.63, 0.74, 0.75, _K),
    ]


def _cat():
    o, od, pk = (242, 152, 62), (204, 112, 42), (250, 170, 184)
    return [
        _b(0.32, 0.18, 0.00, 0.40, 0.26, 0.14, od), _b(0.60, 0.18, 0.00, 0.68, 0.26, 0.14, od),
        _b(0.32, 0.50, 0.00, 0.40, 0.58, 0.14, od), _b(0.60, 0.50, 0.00, 0.68, 0.58, 0.14, od),
        _b(0.30, 0.12, 0.14, 0.70, 0.60, 0.48, o),
        _b(0.30, 0.22, 0.48, 0.70, 0.28, 0.50, od), _b(0.30, 0.36, 0.48, 0.70, 0.42, 0.50, od),
        _b(0.46, 0.04, 0.30, 0.54, 0.12, 0.86, od),
        _b(0.26, 0.52, 0.34, 0.74, 0.90, 0.76, o),
        _b(0.28, 0.66, 0.76, 0.40, 0.78, 0.90, od), _b(0.60, 0.66, 0.76, 0.72, 0.78, 0.90, od),
        _b(0.40, 0.90, 0.38, 0.60, 0.94, 0.52, _W),
        _b(0.47, 0.94, 0.48, 0.53, 0.95, 0.53, pk),
        _b(0.32, 0.90, 0.58, 0.40, 0.91, 0.66, _K), _b(0.60, 0.90, 0.58, 0.68, 0.91, 0.66, _K),
    ]


def _fox():
    f, fd, dk = (236, 122, 42), (192, 92, 32), (52, 44, 44)
    return [
        _b(0.34, 0.20, 0.00, 0.42, 0.28, 0.18, dk), _b(0.58, 0.20, 0.00, 0.66, 0.28, 0.18, dk),
        _b(0.34, 0.52, 0.00, 0.42, 0.60, 0.18, dk), _b(0.58, 0.52, 0.00, 0.66, 0.60, 0.18, dk),
        _b(0.30, 0.14, 0.18, 0.70, 0.66, 0.50, f),
        _b(0.36, 0.66, 0.20, 0.64, 0.69, 0.44, _W),
        _b(0.40, -0.12, 0.24, 0.60, 0.14, 0.48, f), _b(0.42, -0.18, 0.26, 0.58, -0.12, 0.46, _W),
        _b(0.30, 0.56, 0.40, 0.70, 0.86, 0.74, f),
        _b(0.40, 0.86, 0.42, 0.60, 1.00, 0.58, _W),
        _b(0.46, 1.00, 0.52, 0.54, 1.02, 0.58, _K),
        _b(0.30, 0.64, 0.74, 0.42, 0.74, 0.92, fd), _b(0.58, 0.64, 0.74, 0.70, 0.74, 0.92, fd),
        _b(0.34, 0.86, 0.62, 0.40, 0.87, 0.68, _K), _b(0.60, 0.86, 0.62, 0.66, 0.87, 0.68, _K),
    ]


def _llama():
    c, cd, pk, tq, br = (246, 238, 216), (216, 202, 174), (236, 92, 142), (60, 190, 192), (120, 96, 80)
    return [
        _b(0.30, 0.20, 0.00, 0.40, 0.30, 0.36, cd), _b(0.60, 0.20, 0.00, 0.70, 0.30, 0.36, cd),
        _b(0.30, 0.54, 0.00, 0.40, 0.64, 0.36, cd), _b(0.60, 0.54, 0.00, 0.70, 0.64, 0.36, cd),
        _b(0.26, 0.14, 0.34, 0.74, 0.70, 0.66, c),
        _b(0.25, 0.30, 0.62, 0.75, 0.54, 0.70, pk), _b(0.25, 0.39, 0.66, 0.75, 0.45, 0.71, tq),
        _b(0.45, 0.08, 0.50, 0.55, 0.14, 0.64, c),
        _b(0.38, 0.56, 0.64, 0.62, 0.76, 1.08, c),
        _b(0.36, 0.62, 1.04, 0.64, 0.94, 1.26, c),
        _b(0.42, 0.94, 1.06, 0.58, 1.00, 1.18, cd),
        _b(0.47, 1.00, 1.12, 0.53, 1.01, 1.16, br),
        _b(0.38, 0.66, 1.26, 0.45, 0.72, 1.40, cd), _b(0.55, 0.66, 1.26, 0.62, 0.72, 1.40, cd),
        _b(0.34, 0.84, 1.14, 0.36, 0.90, 1.20, _K), _b(0.64, 0.84, 1.14, 0.66, 0.90, 1.20, _K),
    ]


def _robot():
    m, md, cy, dk = (172, 180, 198), (122, 130, 150), (90, 230, 255), (62, 66, 82)
    return [
        _b(0.32, 0.40, 0.00, 0.44, 0.56, 0.22, dk), _b(0.56, 0.40, 0.00, 0.68, 0.56, 0.22, dk),
        _b(0.26, 0.30, 0.22, 0.74, 0.70, 0.64, m),
        _b(0.36, 0.70, 0.32, 0.64, 0.72, 0.56, md),
        _b(0.40, 0.72, 0.42, 0.48, 0.73, 0.50, _RED), _b(0.52, 0.72, 0.42, 0.60, 0.73, 0.50, cy),
        _b(0.18, 0.40, 0.28, 0.26, 0.60, 0.56, md), _b(0.74, 0.40, 0.28, 0.82, 0.60, 0.56, md),
        _b(0.30, 0.34, 0.64, 0.70, 0.72, 0.96, m),
        _b(0.34, 0.72, 0.74, 0.66, 0.74, 0.86, cy),
        _b(0.70, 0.46, 0.74, 0.72, 0.60, 0.86, cy),
        _b(0.48, 0.50, 0.96, 0.52, 0.54, 1.10, dk),
        _b(0.45, 0.47, 1.10, 0.55, 0.57, 1.18, _RED),
    ]


def _ghost():
    g, gd = (236, 240, 255), (204, 212, 240)
    return [
        _b(0.26, 0.26, 0.12, 0.38, 0.38, 0.24, gd), _b(0.62, 0.26, 0.12, 0.74, 0.38, 0.24, gd),
        _b(0.26, 0.62, 0.12, 0.38, 0.74, 0.24, gd), _b(0.62, 0.62, 0.12, 0.74, 0.74, 0.24, gd),
        _b(0.44, 0.44, 0.08, 0.56, 0.56, 0.24, gd),
        _b(0.26, 0.26, 0.24, 0.74, 0.74, 0.92, g),
        _b(0.32, 0.32, 0.92, 0.68, 0.68, 1.00, g),
        _b(0.18, 0.44, 0.46, 0.26, 0.60, 0.62, g), _b(0.74, 0.44, 0.46, 0.82, 0.60, 0.62, g),
        _b(0.34, 0.74, 0.62, 0.44, 0.76, 0.78, _K), _b(0.56, 0.74, 0.62, 0.66, 0.76, 0.78, _K),
        _b(0.44, 0.74, 0.44, 0.56, 0.76, 0.54, _K),
    ]


def _unicorn():
    wt, wd, gold = (250, 250, 252), (222, 222, 236), (250, 202, 62)
    hoof = (178, 170, 204)
    mane = [(240, 92, 122), (250, 172, 62), (122, 204, 92), (92, 152, 242), (172, 112, 232)]
    out = []
    for x0 in (0.30, 0.60):
        for y0 in (0.18, 0.54):
            out.append(_b(x0, y0, 0.00, x0 + 0.10, y0 + 0.10, 0.08, hoof))
            out.append(_b(x0, y0, 0.08, x0 + 0.10, y0 + 0.10, 0.32, wt))
    out += [
        _b(0.28, 0.12, 0.30, 0.72, 0.70, 0.64, wt),
        _b(0.36, 0.58, 0.58, 0.64, 0.80, 0.98, wt),
        _b(0.34, 0.66, 0.94, 0.66, 1.02, 1.18, wt),
        _b(0.40, 1.02, 0.96, 0.60, 1.08, 1.10, wd),
        _b(0.46, 0.86, 1.18, 0.54, 0.94, 1.40, gold),
        _b(0.34, 0.96, 1.04, 0.36, 1.00, 1.10, _K), _b(0.64, 0.96, 1.04, 0.66, 1.00, 1.10, _K),
    ]
    for i, col in enumerate(mane[:4]):
        out.append(_b(0.44, 0.50 + i * 0.05, 0.64 + i * 0.12, 0.56, 0.58 + i * 0.05, 0.78 + i * 0.12, col))
    out.append(_b(0.44, 0.02, 0.44, 0.56, 0.12, 0.62, mane[4]))
    out.append(_b(0.44, 0.00, 0.30, 0.56, 0.08, 0.46, mane[0]))
    return out


CHAR_MODELS = {
    "chicken": _chicken(), "frog": _frog(), "pig": _pig(), "penguin": _penguin(),
    "cat": _cat(), "fox": _fox(), "llama": _llama(), "robot": _robot(),
    "ghost": _ghost(), "unicorn": _unicorn(),
}
# Hauptfarbe je Figur (Federn/Partikel beim Unfall)
CHAR_COLORS = {"chicken": _W, "frog": (98, 198, 82), "pig": (248, 172, 186),
               "penguin": (44, 48, 62), "cat": (242, 152, 62), "fox": (236, 122, 42),
               "llama": (246, 238, 216), "robot": (172, 180, 198),
               "ghost": (236, 240, 255), "unicorn": (250, 250, 252)}


def _tree(kind):
    trunk = (124, 86, 54)
    if kind == 4:                                   # Stein
        return [_b(0.18, 0.24, 0.00, 0.82, 0.76, 0.30, (138, 142, 152)),
                _b(0.30, 0.34, 0.30, 0.70, 0.66, 0.44, (160, 164, 174)),
                _b(0.36, 0.40, 0.44, 0.56, 0.58, 0.50, (176, 180, 188))]
    leaf = TREE_LEAVES[kind - 1]
    top = tuple(min(255, v + 18) for v in leaf)
    if kind == 1:
        return [_b(0.40, 0.40, 0.00, 0.60, 0.60, 0.32, trunk),
                _b(0.20, 0.20, 0.32, 0.80, 0.80, 0.84, leaf)]
    if kind == 2:
        return [_b(0.40, 0.40, 0.00, 0.60, 0.60, 0.32, trunk),
                _b(0.18, 0.18, 0.32, 0.82, 0.82, 0.92, leaf),
                _b(0.30, 0.30, 0.92, 0.70, 0.70, 1.22, top)]
    return [_b(0.40, 0.40, 0.00, 0.60, 0.60, 0.40, trunk),
            _b(0.16, 0.16, 0.40, 0.84, 0.84, 1.00, leaf),
            _b(0.24, 0.24, 1.00, 0.76, 0.76, 1.42, top),
            _b(0.34, 0.34, 1.42, 0.66, 0.66, 1.72, leaf)]


def _car(color):
    col = tuple(color)
    roof = tuple(min(255, v + 26) for v in col)
    tyre = (34, 34, 40)
    return [
        _b(0.18, 0.10, 0.00, 0.46, 0.16, 0.20, tyre), _b(0.94, 0.10, 0.00, 1.22, 0.16, 0.20, tyre),
        _b(0.18, 0.84, 0.00, 0.46, 0.90, 0.20, tyre), _b(0.94, 0.84, 0.00, 1.22, 0.90, 0.20, tyre),
        _b(0.02, 0.14, 0.08, 1.38, 0.86, 0.42, col),
        _b(0.34, 0.20, 0.42, 1.04, 0.80, 0.70, roof),
        _b(1.04, 0.24, 0.46, 1.06, 0.76, 0.66, _GLASS),
        _b(0.40, 0.18, 0.47, 0.66, 0.20, 0.66, _GLASS), _b(0.72, 0.18, 0.47, 0.98, 0.20, 0.66, _GLASS),
        _b(1.38, 0.20, 0.20, 1.40, 0.34, 0.32, (255, 242, 170)),
        _b(1.38, 0.66, 0.20, 1.40, 0.80, 0.32, (255, 242, 170)),
    ]


def _truck(color):
    col = tuple(color)
    tyre = (34, 34, 40)
    return [
        _b(0.20, 0.12, 0.00, 0.50, 0.18, 0.22, tyre), _b(1.00, 0.12, 0.00, 1.30, 0.18, 0.22, tyre),
        _b(1.90, 0.12, 0.00, 2.20, 0.18, 0.22, tyre),
        _b(0.05, 0.18, 0.12, 2.45, 0.82, 0.26, (62, 62, 72)),
        _b(0.02, 0.12, 0.26, 1.70, 0.88, 1.00, (236, 236, 242)),
        _b(0.02, 0.11, 0.54, 1.70, 0.12, 0.66, col),
        _b(1.76, 0.16, 0.26, 2.48, 0.84, 0.80, col),
        _b(2.48, 0.22, 0.52, 2.50, 0.78, 0.72, _GLASS),
        _b(1.92, 0.14, 0.52, 2.32, 0.16, 0.72, _GLASS),
        _b(2.48, 0.20, 0.30, 2.50, 0.34, 0.40, (255, 242, 170)),
        _b(2.48, 0.66, 0.30, 2.50, 0.80, 0.40, (255, 242, 170)),
    ]


def _log(length):
    bark, dark, end = (150, 102, 62), (124, 84, 50), (210, 164, 112)
    out = [_b(0.0, 0.16, 0.0, length, 0.84, 0.26, bark)]
    for i in range(int(length)):
        out.append(_b(i + 0.45, 0.16, 0.26, i + 0.55, 0.84, 0.28, dark))
    out.append(_b(length, 0.24, 0.04, length + 0.02, 0.76, 0.22, end))
    return out


def _pad(flower):
    out = [_b(0.14, 0.14, 0.0, 0.86, 0.86, 0.05, (86, 178, 78)),
           _b(0.44, 0.50, 0.05, 0.56, 0.86, 0.06, (70, 150, 64))]
    if flower:
        out.append(_b(0.24, 0.24, 0.05, 0.36, 0.36, 0.14, (250, 164, 204)))
        out.append(_b(0.27, 0.27, 0.14, 0.33, 0.33, 0.17, (255, 226, 120)))
    return out


def _coin(big):
    s = 1.45 if big else 1.0
    gd, gl, sh = (236, 176, 36), (255, 218, 84), (255, 246, 190)

    def cb(x0, z0, x1, z1, y0, y1, col):
        cx, cz = 0.5, 0.42
        return _b(cx + (x0 - cx) * s, 0.5 + (y0 - 0.5) * s, cz + (z0 - cz) * s,
                  cx + (x1 - cx) * s, 0.5 + (y1 - 0.5) * s, cz + (z1 - cz) * s, col)
    return [cb(0.30, 0.28, 0.70, 0.56, 0.46, 0.54, gd),
            cb(0.38, 0.20, 0.62, 0.64, 0.46, 0.54, gd),
            cb(0.38, 0.28, 0.62, 0.56, 0.44, 0.46, gl),
            cb(0.46, 0.32, 0.54, 0.52, 0.43, 0.44, sh)]


def _engine():
    red, roof, dk, yl = (214, 58, 58), (172, 40, 44), (52, 52, 62), (250, 210, 70)
    out = [_b(0.05, 0.12, 0.00, 2.95, 0.88, 0.24, dk),
           _b(0.05, 0.14, 0.24, 2.95, 0.86, 1.02, red),
           _b(0.15, 0.20, 1.02, 2.40, 0.80, 1.12, roof),
           _b(2.95, 0.18, 0.24, 3.00, 0.82, 0.44, yl),
           _b(2.95, 0.30, 0.62, 2.97, 0.70, 0.90, _GLASS),
           _b(2.97, 0.42, 0.46, 3.00, 0.58, 0.58, (255, 246, 186))]
    for i in range(3):
        out.append(_b(0.30 + i * 0.85, 0.12, 0.56, 0.80 + i * 0.85, 0.14, 0.86, _GLASS))
    return out


def _wagon():
    body, roof, dk = (70, 122, 204), (52, 92, 164), (52, 52, 62)
    out = [_b(0.10, 0.12, 0.00, 2.90, 0.88, 0.24, dk),
           _b(0.10, 0.14, 0.24, 2.90, 0.86, 0.98, body),
           _b(0.14, 0.18, 0.98, 2.86, 0.82, 1.06, roof),
           _b(2.90, 0.44, 0.24, 3.00, 0.56, 0.34, dk)]
    for i in range(3):
        out.append(_b(0.30 + i * 0.88, 0.12, 0.52, 0.86 + i * 0.88, 0.14, 0.82, (236, 236, 190)))
    return out


def _signal(lit):
    on, off = (255, 70, 56), (96, 40, 40)
    a, b = (on, off) if lit == 1 else ((off, on) if lit == 2 else (off, off))
    return [_b(0.44, 0.10, 0.00, 0.56, 0.22, 1.10, (70, 72, 84)),
            _b(0.24, 0.08, 0.96, 0.76, 0.10, 1.30, (238, 238, 238)),
            _b(0.28, 0.06, 1.04, 0.44, 0.08, 1.20, a),
            _b(0.56, 0.06, 1.04, 0.72, 0.08, 1.20, b),
            _b(0.34, 0.08, 0.30, 0.66, 0.24, 0.44, (238, 238, 238))]


def _eagle():
    """Adler mit ausgebreiteten Flügeln, Kopf nach +y (Mitte bei x = y = 0)."""
    br, brd, wt, yl = (126, 84, 52), (96, 62, 38), (244, 244, 244), (250, 190, 40)
    return [_b(-0.30, -0.55, 0.05, 0.30, 0.45, 0.45, br),
            _b(-0.22, -0.95, 0.18, 0.22, -0.55, 0.32, wt),
            _b(-0.95, -0.25, 0.30, -0.30, 0.30, 0.42, br),
            _b(0.30, -0.25, 0.30, 0.95, 0.30, 0.42, br),
            _b(-1.65, -0.35, 0.40, -0.95, 0.20, 0.52, brd),
            _b(0.95, -0.35, 0.40, 1.65, 0.20, 0.52, brd),
            _b(-0.24, 0.40, 0.12, 0.24, 0.85, 0.55, wt),
            _b(-0.08, 0.85, 0.20, 0.08, 1.02, 0.34, yl),
            _b(-0.25, 0.62, 0.38, -0.24, 0.72, 0.46, _K),
            _b(0.24, 0.62, 0.38, 0.25, 0.72, 0.46, _K),
            _b(-0.18, -0.10, -0.08, -0.06, 0.04, 0.05, yl),
            _b(0.06, -0.10, -0.08, 0.18, 0.04, 0.05, yl)]


def static_models():
    """Alle festen Modelle nach Namen (für Sprite-Cache und Web-Export)."""
    m = {}
    for k in (1, 2, 3, 4):
        m["tree%d" % k] = _tree(k)
    for i, col in enumerate(CAR_COLORS):
        m["car%d" % i] = _car(col)
        m["truck%d" % i] = _truck(col)
    for ln in (2, 3, 4):
        m["log%d" % ln] = _log(ln)
    m["pad0"] = _pad(False)
    m["pad1"] = _pad(True)
    m["coin"] = _coin(False)
    m["bigcoin"] = _coin(True)
    m["engine"] = _engine()
    m["wagon"] = _wagon()
    for lit in (0, 1, 2):
        m["signal%d" % lit] = _signal(lit)
    m["eagle"] = _eagle()
    # im Anflug auf die Kamera zu: um die Mitte gedreht (Kopf nach vorn)
    m["eagle_down"] = [(-b[3], -b[4], b[2], -b[0], -b[1], b[5], b[6]) for b in m["eagle"]]
    return m


MODELS = static_models()


def mirror_x(boxes, length):
    """Spiegelt ein Modell an der Mitte seiner Länge (Fahrtrichtung links)."""
    return [(length - b[3], b[1], b[2], length - b[0], b[4], b[5], b[6]) for b in boxes]


def rotate(boxes, facing):
    """Dreht ein nach +y blickendes Figurmodell um die Zellmitte."""
    if facing == "up":
        return list(boxes)
    out = []
    for x0, y0, z0, x1, y1, z1, col in boxes:
        if facing == "down":
            out.append((1 - x1, 1 - y1, z0, 1 - x0, 1 - y0, z1, col))
        elif facing == "right":
            out.append((y0, 1 - x1, z0, y1, 1 - x0, z1, col))
        else:
            out.append((1 - y1, x0, z0, 1 - y0, x1, z1, col))
    return out
