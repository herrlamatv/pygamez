# -*- coding: utf-8 -*-
"""
minigolf_gen.py
===============
Bahn-Generator für Minigolf.

Aus einem Seed entstehen reproduzierbare Bahnen: Kurs 7, Bahn 3 sieht bei
jedem Start und auf jedem Rechner gleich aus - gespeichert werden muss dafür
nichts. Die Tour umfasst ``TOUR_COURSES`` Kurse zu je ``HOLES_PER_ROUND``
Bahnen; zusammen mit den 18 handgebauten Bahnen aus ``minigolf.py`` ergibt das
``TOTAL_HOLES`` Bahnen.

**Passierbarkeit ist eingebaut, nicht erhofft.** Jede Bahnfamilie legt zuerst
den Weg vom Abschlag zum Loch fest und baut die Hindernisse anschließend
darum herum:

- Wandreihen lassen immer eine Lücke von mindestens ``MIN_PASS`` Einheiten
  (Ball-Durchmesser: 3,4).
- Wasser liegt ausschließlich neben dem Weg, nie quer darüber.
- Ein Wanderblock ist stets schmaler als seine Lücke, sodass links oder rechts
  immer eine Durchfahrt bleibt.
- Mühlenflügel sind kürzer als der halbe Korridor - waagerecht bleibt neben
  dem Flügel Platz, senkrecht sowieso.
- ``_sanitize`` entfernt zum Schluss alles, was auf Abschlag oder Loch liegt.

``tests/newgames_audit.py`` prüft zusätzlich **jede** erzeugte Bahn mit einem
Solver, der echte Schläge simuliert.
"""

import math
import random

# ------------------------------------------------------------ Kursmaße
# Einzige Definition der Platzmaße - minigolf.py importiert sie von hier.
CW, CH = 100.0, 160.0        # Kursgröße in Bahn-Einheiten
BORDER = 3.0                 # Bandenbreite am Rand
BALL_R = 1.7                 # Ballradius (muss zu minigolf.BR passen)

# Innenfläche des Grüns (ohne Bande)
X0, X1 = BORDER, CW - BORDER
Y0, Y1 = BORDER, CH - BORDER

HOLES_PER_ROUND = 9          # Bahnen je Runde
TOUR_COURSES = 38            # Tour-Kurse (zusätzlich zu Classic + Pro)
HANDMADE_HOLES = 18          # Classic + Pro in minigolf.py
TOTAL_HOLES = HANDMADE_HOLES + TOUR_COURSES * HOLES_PER_ROUND

MIN_PASS = 12.0              # schmalste Durchfahrt, die der Generator zulässt
_SEED = 0x9E3779B1           # Basis-Seed der Tour

# Lückenbreite je Schwierigkeitsstufe (0 = leicht ... 3 = fies)
GAPS = (24.0, 21.0, 18.0, 15.0)


# Alle Listen-Schlüssel einer Bahn, in der Reihenfolge von make_hole. Wer
# über "alle Hindernisse" laufen will (Editor, Spiegeln, Speichern), nimmt
# diese Liste - dann wird beim nächsten neuen Typ nichts vergessen.
HOLE_LISTS = ("walls", "sand", "water", "slopes", "bumpers", "movers", "mills",
              "tunnels", "ice", "boosters", "magnets", "gates", "sticky",
              "spinners", "jumps")


def make_hole(par, tee, cup, walls=(), sand=(), water=(), slopes=(),
              bumpers=(), movers=(), mills=(), tunnels=(), ice=(), boosters=(),
              magnets=(), gates=(), sticky=(), spinners=(), jumps=(),
              w=None, h=None):
    """Baut einen Bahn-Datensatz (gleiche Struktur wie die handgebauten Bahnen).

    Die sieben klassischen Typen:

    walls/sand/water : (x, y, w, h)
    slopes           : (x, y, w, h, ax, ay)          ax/ay = Beschleunigung
    bumpers          : (x, y, r)
    movers           : (x, y, w, h, dx, dy, speed)   pendelt zwischen den Enden
    mills            : (x, y, länge, arme, speed)   speed in rad/s

    Dazu die acht Typen, die es im Bahn-Editor gibt (siehe minigolf.py):

    tunnels  : (x1, y1, x2, y2, r)          Rohr-Paar, versetzt in beide Richtungen
    ice      : (x, y, w, h)                 fast reibungsfrei
    boosters : (x, y, w, h, dx, dy, boost)  Einmal-Schub beim Betreten
    magnets  : (x, y, r, force)             zieht an (force > 0) / stößt ab
    gates    : (x, y, w, h, dx, dy)         Einbahn-Tor, nur Richtung (dx, dy)
    sticky   : (x, y, w, h)                 bremst extrem
    spinners : (x, y, r, speed)             Drehscheibe, speed in rad/s
    jumps    : (x, y, w, h, dx, dy, dist)   Sprungrampe, fliegt dist Einheiten

    w/h legen die Bahngröße fest (Standard: CW x CH). Eigene Bahnen aus dem
    Editor dürfen davon abweichen, die eingebauten tun es nie.
    """
    hole = {"par": int(par), "tee": tee, "cup": cup,
            "w": float(CW if w is None else w),
            "h": float(CH if h is None else h)}
    for key, items in (("walls", walls), ("sand", sand), ("water", water),
                       ("slopes", slopes), ("bumpers", bumpers),
                       ("movers", movers), ("mills", mills),
                       ("tunnels", tunnels), ("ice", ice),
                       ("boosters", boosters), ("magnets", magnets),
                       ("gates", gates), ("sticky", sticky),
                       ("spinners", spinners), ("jumps", jumps)):
        hole[key] = [tuple(map(float, it)) for it in items]
    return hole


def normalize(hole):
    """Ergänzt fehlende Schlüssel einer Bahn (immer dasselbe dict zurück).

    Der Verträglichkeits-Riegel: ältere Replays (replay.json) und ältere
    eigene Bahnen (ugc.json) kennen die neuen Hindernis-Typen und die
    Bahngröße noch nicht. Statt überall mit ``hole.get(...)`` zu hantieren,
    läuft jede Bahn einmal hier durch - danach sind alle Schlüssel da.
    """
    if not isinstance(hole, dict):
        return make_hole(3, (CW / 2, CH - 18), (CW / 2, 22))
    hole.setdefault("par", 3)
    hole.setdefault("tee", (CW / 2, CH - 18))
    hole.setdefault("cup", (CW / 2, 22))
    for key in ("w", "h"):
        try:
            hole[key] = float(hole[key])
        except (KeyError, TypeError, ValueError):
            hole[key] = CW if key == "w" else CH
    for key in HOLE_LISTS:
        items = hole.get(key)
        hole[key] = list(items) if isinstance(items, (list, tuple)) else []
    return hole


# ---------------------------------------------------------------- Helfer

def _clampx(x, margin=6.0):
    return max(X0 + margin, min(X1 - margin, x))


def _clampy(y, margin=6.0):
    return max(Y0 + margin, min(Y1 - margin, y))


def _gap_center(side, gap):
    """Mitte der Lücke einer Wandreihe (side 0 = links, 1 = rechts)."""
    return X0 + gap / 2 if side == 0 else X1 - gap / 2


def _row(side, gap, y, h=7.0):
    """Wandreihe über die volle Breite mit Lücke links oder rechts."""
    width = (X1 - X0) - gap
    return (X0 + gap, y, width, h) if side == 0 else (X0, y, width, h)


def _sand_patch(rng, x, y, w=None, h=None):
    w = w if w is not None else rng.uniform(16, 26)
    h = h if h is not None else rng.uniform(12, 18)
    x = max(X0 + 1, min(X1 - 1 - w, x - w / 2))
    y = max(Y0 + 1, min(Y1 - 1 - h, y - h / 2))
    return (x, y, w, h)


def _scatter_sand(rng, hole, spots):
    """Streut Sandflächen an vorgegebenen Stellen (Sand blockiert nie)."""
    for (x, y) in spots:
        if rng.random() < 0.65:
            hole["sand"].append(_sand_patch(rng, x, y))


# Hindernisse, die den Ball wirklich aufhalten oder bestrafen - nur diese
# dürfen nicht auf Abschlag oder Loch liegen. Sand, Eis, Kleber und Schub
# stören dort niemanden und bleiben deshalb draußen vor.
_BLOCKING_RECTS = ("walls", "water", "gates", "jumps")


def _sanitize(hole):
    """Entfernt Hindernisse auf Abschlag/Loch und klemmt beides ins Feld.

    Es wird ausschließlich entfernt, nie hinzugefügt - der Weg kann dadurch
    nur breiter werden, niemals enger.

    Rechnet mit der Bahngröße aus dem Datensatz, damit auch die im Editor
    gebauten Bahnen abweichender Größe sauber geprüft werden.
    """
    normalize(hole)
    cw, ch = hole["w"], hole["h"]
    x0, x1 = BORDER, cw - BORDER
    y0, y1 = BORDER, ch - BORDER

    def clamp(v, lo, hi, margin):
        return max(lo + margin, min(hi - margin, v))

    hole["tee"] = (clamp(hole["tee"][0], x0, x1, BALL_R + 3),
                   clamp(hole["tee"][1], y0, y1, BALL_R + 3))
    hole["cup"] = (clamp(hole["cup"][0], x0, x1, BALL_R + 5),
                   clamp(hole["cup"][1], y0, y1, BALL_R + 5))
    pts = (hole["tee"], hole["cup"])
    clear = BALL_R + 3.0

    def rect_free(r):
        x, y, w, h = r[0], r[1], r[2], r[3]
        return all(not (x - clear < px < x + w + clear
                        and y - clear < py < y + h + clear) for (px, py) in pts)

    def circle_free(cx, cy, r):
        return all(math.hypot(cx - px, cy - py) > r + clear for (px, py) in pts)

    for key in _BLOCKING_RECTS:
        hole[key] = [r for r in hole[key] if rect_free(r)]
    hole["bumpers"] = [b for b in hole["bumpers"] if circle_free(b[0], b[1], b[2])]
    hole["mills"] = [m for m in hole["mills"] if circle_free(m[0], m[1], m[2])]
    hole["spinners"] = [s for s in hole["spinners"]
                        if circle_free(s[0], s[1], s[2])]
    # Rohre dürfen nicht am Loch enden (sonst saugt der Tunnel den Ball
    # heraus) und nicht auf dem Abschlag liegen.
    hole["tunnels"] = [t for t in hole["tunnels"]
                       if circle_free(t[0], t[1], t[4])
                       and circle_free(t[2], t[3], t[4])]
    kept = []
    for m in hole["movers"]:
        x, y, w, h, dx, dy, _sp = m
        if rect_free((x, y, w, h)) and rect_free((x + dx, y + dy, w, h)):
            kept.append(m)
    hole["movers"] = kept
    return hole


# ------------------------------------------------------------ Bahnfamilien
#
# Jede Familie bekommt (rng, tier, idx) und liefert eine fertige Bahn.
# tier 0..3 = Schwierigkeitsstufe des Kurses, idx = Bahnnummer 0..8.

def _f_straight(rng, tier, idx):
    """Gerade Bahn mit Trichter - der ruhige Auftakt jedes Kurses."""
    cx = rng.uniform(38, 62)
    gap = GAPS[tier] + 4
    ymid = rng.uniform(66, 88)
    half = (X1 - X0 - gap) / 2
    walls = [(X0, ymid, half, 8), (X1 - half, ymid, half, 8)]
    hole = make_hole(2 if tier < 2 else 3, (cx, 142.0), (cx, rng.uniform(20, 30)),
                     walls=walls)
    _scatter_sand(rng, hole, [(X0 + 14, ymid + 26), (X1 - 14, ymid + 26)])
    if tier >= 2 and rng.random() < 0.6:
        hole["bumpers"].append((cx, ymid - 22, 6.0))
    return _sanitize(hole)


def _f_gates(rng, tier, idx):
    """Wandreihen mit versetzten Lücken - der Zickzack-Klassiker."""
    n = max(1, min(4, 1 + tier + (1 if idx >= 6 else 0)))
    gap = GAPS[tier]
    if n == 1:
        ys = [92.0]
    else:
        ys = [126.0 - i * (92.0 / (n - 1)) for i in range(n)]
    side = rng.randint(0, 1)
    walls, sides = [], []
    for i, y in enumerate(ys):
        s = (side + i) % 2
        sides.append(s)
        walls.append(_row(s, gap, y))
    tee = (_gap_center(sides[0], gap), 146.0)
    top = ys[-1]
    if tier >= 2 and rng.random() < 0.5:          # Loch auf der Gegenseite
        cup_x = _gap_center(1 - sides[-1], gap)
    else:
        cup_x = _gap_center(sides[-1], gap)
    hole = make_hole(min(5, 2 + n), tee, (cup_x, max(16.0, top - 16.0)),
                     walls=walls)
    for i, y in enumerate(ys[:-1]):
        _scatter_sand(rng, hole, [(_gap_center(1 - sides[i], gap), y - 14)])
    return _sanitize(hole)


def _f_dogleg(rng, tier, idx):
    """Lange Wand quer im Feld - der Weg führt außen herum."""
    xw = rng.uniform(38, 58)
    ytop = rng.uniform(32, 44)
    ylen = rng.uniform(72, 96)
    flip = rng.random() < 0.5
    if flip:
        xw = CW - xw - 9
    walls = [(xw, ytop, 9.0, ylen)]
    # Optionaler Riegel im unteren Feld, damit der Bogen größer wird
    if tier >= 2:
        by = rng.uniform(96, 116)
        if flip:
            walls.append((X0, by, rng.uniform(22, 30), 8.0))
        else:
            w = rng.uniform(22, 30)
            walls.append((X1 - w, by, w, 8.0))
    tee_x = xw / 2 if not flip else (xw + 9 + X1) / 2
    cup_x = (xw + 9 + X1) / 2 if not flip else xw / 2
    hole = make_hole(3 if tier < 2 else 4, (_clampx(tee_x), 142.0),
                     (_clampx(cup_x), max(16.0, ytop - 14.0)), walls=walls)
    _scatter_sand(rng, hole, [(xw + 4.5, ytop + ylen + 12)])
    if tier >= 1 and rng.random() < 0.5:
        hole["bumpers"].append((_clampx(cup_x + rng.choice((-18, 18))),
                                ytop + 18, 6.0))
    return _sanitize(hole)


def _f_bumpers(rng, tier, idx):
    """Offenes Feld voller Gummipuffer - blockieren kann hier nichts."""
    cx = rng.uniform(40, 60)
    k = 3 + tier
    bumpers = []
    for _ in range(k * 4):
        if len(bumpers) >= k:
            break
        x = rng.uniform(X0 + 10, X1 - 10)
        y = rng.uniform(40, 122)
        r = rng.uniform(5, 7)
        if all(math.hypot(x - b[0], y - b[1]) > r + b[2] + 8 for b in bumpers):
            bumpers.append((x, y, r))
    hole = make_hole(3, (cx, 142.0), (_clampx(cx + rng.uniform(-14, 14)),
                                      rng.uniform(18, 28)), bumpers=bumpers)
    _scatter_sand(rng, hole, [(cx, 118.0)])
    return _sanitize(hole)


def _f_water(rng, tier, idx):
    """Teiche links und rechts einer trockenen Gasse."""
    lane = GAPS[tier] + 8
    cx = rng.uniform(34, 66)
    y0 = rng.uniform(48, 66)
    h = rng.uniform(30, 46)
    left_w = (cx - lane / 2) - X0
    right_w = X1 - (cx + lane / 2)
    water = []
    if left_w > 8:
        water.append((X0, y0, left_w, h))
    if right_w > 8:
        water.append((cx + lane / 2, y0, right_w, h))
    walls = []
    if tier >= 2:                       # zusätzliche Engstelle über dem Wasser
        gap = max(MIN_PASS + 3, GAPS[tier])
        gx = _clampx(cx + rng.uniform(-8, 8), gap / 2 + 2)
        walls = [(X0, y0 - 16, max(2.0, gx - gap / 2 - X0), 7.0),
                 (gx + gap / 2, y0 - 16, max(2.0, X1 - gx - gap / 2), 7.0)]
    hole = make_hole(3 if tier < 2 else 4, (cx, 144.0),
                     (_clampx(cx + rng.uniform(-10, 10)), rng.uniform(18, 30)),
                     walls=walls, water=water)
    _scatter_sand(rng, hole, [(cx, y0 + h + 16)])
    return _sanitize(hole)


def _f_ramp(rng, tier, idx):
    """Steigung, die zurückschiebt - hier braucht es Kraft."""
    y0 = rng.uniform(40, 56)
    h = rng.uniform(44, 60)
    x0 = rng.uniform(16, 26)
    w = rng.uniform(52, 66)
    accel = 20.0 + tier * 5.0
    cx = _clampx(x0 + w / 2 + rng.uniform(-10, 10))
    hole = make_hole(3 if tier < 2 else 4, (cx, 146.0), (cx, rng.uniform(16, 26)),
                     slopes=[(x0, y0, w, h, 0.0, accel)])
    _scatter_sand(rng, hole, [(x0 + 8, y0 + h + 14), (x0 + w - 8, y0 + h + 14)])
    if tier >= 2:
        hole["bumpers"].append((_clampx(cx + rng.choice((-24, 24))), y0 - 12, 6.0))
    return _sanitize(hole)


def _f_chicane(rng, tier, idx):
    """Zwei stehende Wände - der Ball schlängelt sich durch."""
    xa = rng.uniform(26, 36)
    xb = rng.uniform(60, 70)
    ya, la = rng.uniform(34, 46), rng.uniform(52, 66)
    yb, lb = rng.uniform(84, 96), rng.uniform(48, 62)
    walls = [(xa, ya, 8.0, la), (xb, yb, 8.0, lb)]
    hole = make_hole(4, (_clampx(xa / 2), 146.0),
                     (_clampx((xb + 8 + X1) / 2), rng.uniform(16, 26)),
                     walls=walls)
    _scatter_sand(rng, hole, [(xa + 22, ya + la + 10)])
    if tier >= 3 and rng.random() < 0.5:
        hole["bumpers"].append((_clampx((xa + xb) / 2), (ya + yb) / 2, 6.0))
    return _sanitize(hole)


def _f_mill(rng, tier, idx):
    """Korridor mit Windmühle(n) - reines Timing."""
    w = 42.0 - tier * 3.0
    xl = 50.0 - w / 2
    xr = 50.0 + w / 2
    ytop, ylen = 22.0, 100.0
    walls = [(xl - 8, ytop, 8.0, ylen), (xr, ytop, 8.0, ylen)]
    arm = w / 2 - 4.5                    # waagerecht bleibt seitlich Platz
    n = 1 if tier < 2 else 2
    mills = []
    for i in range(n):
        y = 104.0 - i * 44.0
        speed = rng.choice((-1.0, 1.0)) * rng.uniform(1.1, 1.9)
        mills.append((50.0, y, arm, 2, speed))
    hole = make_hole(3 + n, (50.0, 146.0), (50.0, 16.0), walls=walls, mills=mills)
    _scatter_sand(rng, hole, [(50.0, 130.0)])
    return _sanitize(hole)


def _f_island(rng, tier, idx):
    """Inselgrün: Wasser ringsum, ein schmaler Hals führt hinauf."""
    cx = rng.uniform(38, 62)
    cy = rng.uniform(30, 42)
    half_w, half_h = 15.0, 13.0
    neck = max(MIN_PASS + 2, min(GAPS[tier], 2 * half_w - 6))
    top = cy - half_h
    bot = cy + half_h
    water = []
    lw = (cx - half_w) - X0
    rw = X1 - (cx + half_w)
    if lw > 8:
        water.append((X0, top, lw, 2 * half_h))
    if rw > 8:
        water.append((cx + half_w, top, rw, 2 * half_h))
    blw = (cx - neck / 2) - X0
    brw = X1 - (cx + neck / 2)
    if blw > 8:
        water.append((X0, bot, blw, 15.0))
    if brw > 8:
        water.append((cx + neck / 2, bot, brw, 15.0))
    hole = make_hole(3 if tier < 2 else 4, (_clampx(cx), 146.0), (cx, cy),
                     water=water)
    _scatter_sand(rng, hole, [(cx, bot + 34)])
    if tier >= 2:
        hole["slopes"].append((cx - 26, bot + 22, 52.0, 30.0, 0.0, -10.0))
    return _sanitize(hole)


def _f_mover(rng, tier, idx):
    """Wandreihe mit Wanderblock - die Lücke ist immer irgendwo offen."""
    bw = 14.0
    gap = bw + 2 * MIN_PASS + 2.0        # selbst mittig bleibt links/rechts Platz
    gx = _clampx(rng.uniform(34, 66), gap / 2 + 3) - gap / 2
    y = rng.uniform(78, 104)
    left_w = gx - X0
    right_w = X1 - (gx + gap)
    walls = []
    if left_w > 2:
        walls.append((X0, y, left_w, 8.0))
    if right_w > 2:
        walls.append((gx + gap, y, right_w, 8.0))
    speed = 18.0 + tier * 4.0
    movers = [(gx, y, bw, 8.0, gap - bw, 0.0, speed)]
    cup_x = _clampx(gx + gap / 2 + rng.uniform(-16, 16))
    hole = make_hole(4, (_clampx(gx + gap / 2), 146.0), (cup_x, rng.uniform(18, 30)),
                     walls=walls, movers=movers)
    _scatter_sand(rng, hole, [(gx + gap / 2, y + 28)])
    if tier >= 3 and rng.random() < 0.5:
        hole["bumpers"].append((_clampx(cup_x + rng.choice((-20, 20))), y - 26, 6.0))
    return _sanitize(hole)


FAMILIES = {
    "straight": _f_straight, "gates": _f_gates, "dogleg": _f_dogleg,
    "bumpers": _f_bumpers, "water": _f_water, "ramp": _f_ramp,
    "chicane": _f_chicane, "mill": _f_mill, "island": _f_island,
    "mover": _f_mover,
}

# Welche Familien ab welcher Stufe vorkommen. Jeder Topf ist größer als die
# acht zu besetzenden Plätze eines Kurses - sonst käme jede Familie zwangsläufig
# doppelt vor und die Kurse sähen sich zu ähnlich.
_POOLS = (
    ("gates", "dogleg", "bumpers", "straight", "water", "ramp"),
    ("gates", "dogleg", "bumpers", "water", "ramp", "chicane", "island"),
    ("gates", "dogleg", "bumpers", "water", "ramp", "chicane", "mill", "island",
     "mover"),
    ("gates", "dogleg", "bumpers", "water", "ramp", "chicane", "mill", "island",
     "mover"),
)
# Der Abschluss jedes Kurses kommt aus dem "großen" Topf
_FINALE = ("mill", "island", "mover", "gates", "chicane")


def tier_of(course):
    """Schwierigkeitsstufe 0..3 eines Tour-Kurses (1..TOUR_COURSES)."""
    return max(0, min(3, (int(course) - 1) // 10))


def _family_for(course, idx, rng, used):
    if idx == 0:
        return "straight"
    tier = tier_of(course)
    pool = list(_POOLS[tier] if idx < HOLES_PER_ROUND - 1
                else _FINALE[:3 + tier])
    rng.shuffle(pool)
    for name in pool:                     # höchstens zweimal dieselbe Familie
        if used.get(name, 0) < 2:
            return name
    return pool[0]


def generate(course, idx, used=None):
    """Erzeugt Bahn ``idx`` (0..8) des Tour-Kurses ``course`` (1..TOUR_COURSES).

    Gleiche Eingaben liefern immer dieselbe Bahn.
    """
    course = max(1, min(TOUR_COURSES, int(course)))
    idx = max(0, min(HOLES_PER_ROUND - 1, int(idx)))
    rng = random.Random(_SEED + course * 9176 + idx * 131)
    name = _family_for(course, idx, rng, used or {})
    if used is not None:
        used[name] = used.get(name, 0) + 1
    hole = FAMILIES[name](rng, tier_of(course), idx)
    hole["family"] = name
    return hole


def course_holes(course):
    """Alle neun Bahnen eines Tour-Kurses, nach Par sortiert."""
    used = {}
    holes = [generate(course, i, used) for i in range(HOLES_PER_ROUND)]
    first, rest = holes[0], holes[1:]
    rest.sort(key=lambda h: h["par"])
    return [first] + rest


def course_par(course):
    """Gesamt-Par eines Tour-Kurses."""
    return sum(h["par"] for h in course_holes(course))
