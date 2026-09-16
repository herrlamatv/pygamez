# -*- coding: utf-8 -*-
"""
tetris_ai.py
============
Heuristik-KI für Tetris (Versus KI) im Stil von Dellacherie / El-Tetris.

- Für den aktuellen Stein (und - wenn erlaubt - den Hold- bzw. nächsten
  Stein) werden alle Endlagen durchprobiert, die ein Mensch mit "drehen,
  schieben, fallen lassen" erreicht. Gedreht wird mit den echten SRS-Kicks
  aus tetris_core, also genau so, wie es später ausgeführt wird.
- Jede Lage wird nach sechs Merkmalen bewertet (El-Tetris-Gewichte):
  Landehöhe, abgetragene Steinzellen, Zeilen- und Spaltenwechsel, Löcher,
  Brunnen. Diese Gewichte überleben allein schon Zehntausende Zeilen.
- Stufe Mittel/Schwer spielt zusätzlich auf Angriff: Solange der Stapel
  niedrig ist, hält sie die rechte Spalte als Brunnen frei und sammelt für
  einen Tetris (4 Müllzeilen) statt einzelne Zeilen abzuräumen. Wird es eng,
  zählt nur noch das Überleben.
- Die drei Stärken unterscheiden sich in Denkzeit, Tempo je Eingabe und
  Fehlerquote (wählt ab und zu nur eine der besten statt der besten Lage).

Die Web-Fassung steckt in web/js/games/tetris_core.js (gleiche Zahlen).
"""

from . import tetris_core as core

W_LAND = -4.500158825082766
W_ERODE = 3.4181268101392694
W_ROWT = -3.2178882868487753
W_COLT = -9.348695305445199
W_HOLES = -7.899265427351652
W_WELLS = -3.3855972247263626

# Angriffsspiel: Bonus/Malus nur bei niedrigem, lochfreiem Stapel.
SAFE_HEIGHT = 9
TETRIS_BONUS = 30.0
SMALL_CLEAR_MALUS = 14.0
WELL_COL_MALUS = 4.0

# Stärken: Denkzeit nach dem Erscheinen (s), Zeit je Eingabe (s),
# Fehlerquote, aus wie vielen der besten Lagen ein Fehler wählt,
# Hold erlaubt, Angriffsspiel.
LEVELS = (
    dict(think=0.55, step=0.11, error=0.22, top=4, hold=False, attack=False),
    dict(think=0.24, step=0.06, error=0.05, top=3, hold=True, attack=True),
    dict(think=0.07, step=0.03, error=0.0, top=1, hold=True, attack=True),
)

_WALLS = 1 | (1 << (core.COLS + 1))
_ROW_PAIRS = (1 << (core.COLS + 1)) - 1
_LEFT_WALL = 1
_RIGHT_WALL = 1 << (core.COLS - 1)


def evaluate(rows, lines, eroded, land_h, attack):
    """Bewertung eines Feldes NACH dem Einrasten (größer = besser)."""
    H = core.ROWS
    full = core.FULL
    t = 0
    while t < H and not rows[t]:
        t += 1
    rowt = 2 * t                     # leere Zeilen: je 2 Wechsel an den Wänden
    colt = 0
    holes = 0
    wells = 0
    cover = 0
    prev = 0
    # Angriffsspiel bei niedrigem Stapel: die rechte Spalte gilt beim Zählen
    # als Wand - ein offener Brunnen dort kostet dann nichts.
    atk = attack and H - t <= SAFE_HEIGHT
    well_cols = full & ~_RIGHT_WALL if atk else full
    wall_bit = _RIGHT_WALL if atk else 0
    col9 = 0
    for i in range(t, H):
        r = rows[i]
        w = ((r | wall_bit) << 1) | _WALLS
        rowt += ((w ^ (w >> 1)) & _ROW_PAIRS).bit_count()
        colt += (r ^ prev).bit_count()
        holes += (cover & ~r & full).bit_count()
        cover |= r
        prev = r
        wm = ~r & ((r << 1) | _LEFT_WALL) & ((r >> 1) | _RIGHT_WALL) & well_cols
        while wm:
            b = wm & -wm
            wm ^= b
            k = i
            while k < H and not rows[k] & b:
                wells += 1
                k += 1
        if r & _RIGHT_WALL:
            col9 += 1
    colt += (prev ^ full).bit_count()
    score = (W_LAND * land_h + W_ERODE * eroded + W_ROWT * rowt
             + W_COLT * colt + W_HOLES * holes + W_WELLS * wells)
    if atk:
        if lines == 4:
            score += TETRIS_BONUS
        elif lines:
            score -= SMALL_CLEAR_MALUS * (4 - lines) / 3.0
        score -= WELL_COL_MALUS * col9
    return score


def _place(rows, kind, rot, x, y):
    """Setzt den Stein in eine Kopie der Zeilen; liefert (zeilen, lines, eroded)."""
    sh = core.SHAPES[kind][rot]
    new = rows[:]
    for dy, m in sh.rows:
        new[y + dy] |= (m << x) if x >= 0 else (m >> -x)
    lines = 0
    eroded = 0
    for dy, m in sh.rows:
        if new[y + dy] == core.FULL:
            lines += 1
            eroded += bin(m).count("1")
    if lines:
        keep = [r for r in new if r != core.FULL]
        new = [0] * lines + keep
    return new, lines, lines * eroded


def _tops(rows):
    """Erste belegte Zeile je Spalte (ROWS = leer)."""
    tops = [core.ROWS] * core.COLS
    seen = 0
    for i, r in enumerate(rows):
        new = r & ~seen
        if new:
            for c in range(core.COLS):
                if new >> c & 1:
                    tops[c] = i
            seen |= r
            if seen == core.FULL:
                break
    return tops


def candidates(rows, kind, rot0, x0, y0, attack=False):
    """Alle erreichbaren Endlagen: Liste (score, dreh_folge, dx, rot, x, y)."""
    out = []
    tops = _tops(rows)
    seqs = ((),) if kind == "O" else ((), (1,), (1, 1), (-1,))
    seen = set()
    for seq in seqs:
        r, x, y = rot0, x0, y0
        ok = True
        for d in seq:
            res = core.try_rotate(rows, kind, r, x, y, d)
            if res is None:
                ok = False
                break
            r, x, y, _k = res
        if not ok:
            continue
        sh = core.SHAPES[kind][r]
        xl = x
        while not core.collides(rows, kind, r, xl - 1, y):
            xl -= 1
        xr = x
        while not core.collides(rows, kind, r, xr + 1, y):
            xr += 1
        for tx in range(xl, xr + 1):
            # Landehöhe über die Spaltenhöhen; liegen Blöcke über dem Stein
            # (Puffer fast voll), ehrlich Zeile für Zeile fallen lassen.
            ly = None
            for cx, by in sh.bottom:
                top = tops[tx + cx]
                if top <= y + by:
                    ly = core.drop_y(rows, kind, r, tx, y)
                    break
                cand = top - 1 - by
                ly = cand if ly is None or cand < ly else ly
            # S/Z/I liefern in zwei Zuständen dieselben Zellen -> nur einmal.
            key = frozenset((tx + cx, ly + cy) for cx, cy in sh.cells)
            if key in seen:
                continue
            seen.add(key)
            new, lines, eroded = _place(rows, kind, r, tx, ly)
            land_h = core.ROWS - (ly + (sh.miny + sh.maxy) / 2.0)
            score = evaluate(new, lines, eroded, land_h, attack)
            out.append((score, seq, tx - x, r, tx, ly))
    out.sort(key=lambda c: -c[0])
    return out


def _spawn_pos(rows, kind):
    """Position, an der 'kind' nach dem Spawn stünde (oder None = belegt)."""
    x, y = core.SPAWN_X, core.SPAWN_Y
    if core.collides(rows, kind, 0, x, y):
        return None
    if not core.collides(rows, kind, 0, x, y + 1):
        y += 1
    return x, y


def plan(board, level=1, rng=None, allow_hold=True):
    """Eingabefolge für den aktuellen Stein.

    Liefert eine Liste aus "hold", "cw", "ccw", "left", "right", "drop"
    (endet immer mit "drop") oder [] ohne aktiven Stein.
    """
    if not board.active:
        return []
    cfg = LEVELS[max(0, min(2, level))]
    attack = cfg["attack"]
    rows = board.rows
    best = candidates(rows, board.kind, board.rot, board.x, board.y, attack)
    use_hold = False
    if allow_hold and cfg["hold"] and not board.hold_used:
        alt = board.hold_kind or board.queue.peek(1)[0]
        pos = _spawn_pos(rows, alt) if alt != board.kind else None
        if pos is not None:
            alt_c = candidates(rows, alt, 0, pos[0], pos[1], attack)
            if alt_c and (not best or alt_c[0][0] > best[0][0] + 1.0):
                best = alt_c
                use_hold = True
    if not best:
        return ["drop"]
    pick = best[0]
    if rng is not None and cfg["error"] > 0 and rng.random() < cfg["error"]:
        pick = best[min(len(best) - 1, int(rng.random() * cfg["top"]))]
    _score, seq, dx, _r, _x, _y = pick
    acts = ["hold"] if use_hold else []
    acts += ["cw" if d > 0 else "ccw" for d in seq]
    acts += ["left" if dx < 0 else "right"] * abs(dx)
    acts.append("drop")
    return acts


def apply(board, act):
    """Führt eine Eingabe auf dem Feld aus; True bei Erfolg."""
    if act == "hold":
        return board.hold()
    if act == "cw":
        return board.rotate(1)
    if act == "ccw":
        return board.rotate(-1)
    if act == "left":
        return board.move(-1)
    if act == "right":
        return board.move(1)
    if act == "drop":
        return board.hard_drop() is not None
    return False
