# -*- coding: utf-8 -*-
"""
build_geodash_solver.py
=======================
Beweist, dass ein Geometry-Dash-Level schaffbar ist - mit dem ECHTEN
Schritt-Code aus ``games/geodash_core.py``.

Suche
-----
Tiefensuche über Entscheidungspunkte: alle ``PERIOD`` Schritte (5 = ~21 ms)
wird entschieden, ob die Taste gedrückt bleibt oder umschaltet. Umschalten
geht nur, wenn die Taste lang genug gedrückt bzw. losgelassen war
(``MIN_HOLD``/``MIN_RELEASE`` - so tippt kein Mensch). Ein Orb reagiert nur
auf den Druck selbst (steigende Flanke), genau wie im Spiel.

Robust statt Glückstreffer
--------------------------
Gesucht wird nicht mit EINEM Zustand, sondern mit dreien gleichzeitig: der
gleichen Eingabefolge, einmal pünktlich, einmal einen Schritt zu spät und
einmal einen Schritt zu früh. Ein Zweig zählt nur, wenn alle drei überleben -
jede gefundene Lösung verträgt damit ±1 Schritt (±4 ms) Versatz.

Bereits besuchte (gerasterte) Zustände werden übersprungen, sonst explodiert
die Suche in offenen Abschnitten (Schiff, Welle).

Münzen: optional muss jede Münze spätestens zwei Blöcke hinter ihrer Position
eingesammelt sein - so sucht der Solver die Münze dort, wo sie liegt, statt
das halbe Level umsonst zu durchsuchen.
"""

import os
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)

# Den Kern direkt laden (nicht über das Paket games, das alle Spiele
# samt pygame importieren würde).
sys.path.insert(0, os.path.join(ROOT, "games"))
import geodash_core as core  # noqa: E402

PERIOD = 5
MIN_HOLD = 10
MIN_RELEASE = 10


class _Node:
    __slots__ = ("sb", "sd", "sa", "held", "run", "toggles", "last_d",
                 "last_a", "k")


def _advance(st, lv, inputs, last):
    """Rechnet einen Zustand mit einer Eingabeliste weiter. -> letzte Eingabe."""
    step = core.step
    for h in inputs:
        step(st, lv, h, h and not last)
        last = h
        if st.dead or st.won:
            break
    return last


def _targets(lv):
    """Zielhöhe (Einheiten, Mitte der Lücke) je Spalte für die Flugformen.

    Nur eine Sortier-Hilfe der Suche: Schiff, UFO und Welle probieren zuerst
    die Richtung, die zur nächsten Lücke führt. Je Spalte mit Hindernissen
    wird die freie Strecke gewählt, die der vorigen Zielhöhe am nächsten
    liegt; leere Spalten übernehmen das Ziel der nächsten Hindernis-Spalte.
    """
    top = 12 * core.B
    target = {}
    prev = 3 * core.B
    for c in sorted(c for c in lv.cols if c >= 0):
        spans = []
        for i in lv.cols[c]:
            if lv.cls[i] in (core.C_SOLID, core.C_HAZARD) and lv.ox[i] == c:
                b = lv.box[i]
                spans.append((max(0, b[1]), min(top, b[3])))
        if not spans:
            continue
        spans.sort()
        free = []
        y = 0
        for a, b in spans:
            if a > y:
                free.append((y, a))
            y = max(y, b)
        if y < top:
            free.append((y, top))
        free = [f for f in free if f[1] - f[0] >= core.B // 2]
        if not free:
            continue
        best = min(free, key=lambda f: abs((f[0] + f[1]) // 2 - prev))
        prev = (best[0] + best[1]) // 2
        target[c] = prev
    out = [3 * core.B] * (lv.length + 2)
    nxt = 3 * core.B
    for c in range(lv.length + 1, -1, -1):
        if c in target:
            nxt = target[c]
        out[c] = nxt
    return out


def solve(level, need_coins=True, max_nodes=400000, verbose=False,
          prefer_release=True, time_limit=600.0):
    """Sucht eine robuste Eingabefolge. Gibt (toggles, info) oder (None, info).

    toggles: sortierte Schritte (0-basiert), in denen die Taste umschaltet.
    """
    lv = level if isinstance(level, core.Level) else core.Level(level)
    t0 = time.time()
    deadlines = []
    if need_coins:
        for i, bit in lv.coin_bit.items():
            deadlines.append((lv.box[i][2] + 2 * core.B, bit))
        deadlines.sort()
    root = _Node()
    root.sb = core.new_state(lv)
    root.sd = core.new_state(lv)
    root.sa = core.new_state(lv)
    root.held = False
    root.run = 10 ** 6
    root.toggles = None           # verkettete Liste (schritt, rest)
    root.last_d = False
    root.last_a = False
    root.k = 0
    stack = [root]
    seen = set()
    nodes = 0
    best_x = 0
    targets = _targets(lv)
    qy = core.B // 12
    qv = 240
    limit_steps = lv.end_x // core.SPEEDS[0] + 8 * core.HZ
    while stack:
        node = stack.pop()
        nodes += 1
        if nodes > max_nodes or time.time() - t0 > time_limit:
            return None, {"nodes": nodes, "best_x": best_x,
                          "reason": "limit", "time": time.time() - t0}
        sb = node.sb
        if sb.x > best_x:
            best_x = sb.x
            if verbose and nodes % 1 == 0:
                pass
        # Kinder: (neuer Tastenzustand) - bevorzugte Wahl zuletzt auf den
        # Stapel, damit sie zuerst drankommt.
        choices = []
        can_toggle = node.run >= (MIN_HOLD if node.held else MIN_RELEASE)
        keep = node.held
        if can_toggle:
            other = not keep
            first, second = keep, other
            if prefer_release and sb.mode == core.CUBE and keep and \
                    node.run >= MIN_HOLD:
                first, second = other, keep
            elif sb.mode in (core.SHIP, core.UFO, core.WAVE):
                # Richtung zur nächsten Lücke zuerst (Blick ~0,3 s voraus)
                col = min(len(targets) - 1,
                          max(0, sb.x // core.B + 4 + sb.speed * 2))
                ahead = sb.y + core.B // 2 + sb.vy * 14
                want_up = ahead < targets[col]
                if sb.grav < 0:
                    want_up = not want_up
                first, second = want_up, not want_up
            choices = [second, first]
        else:
            choices = [keep]
        for c in choices:
            k = node.k
            toggled = c != node.held
            ch = _Node()
            ch.sb = sb.copy()
            ch.sd = node.sd.copy()
            ch.sa = node.sa.copy()
            ch.held = c
            ch.run = PERIOD if toggled else node.run + PERIOD
            ch.toggles = (k, node.toggles) if toggled else node.toggles
            ch.k = k + PERIOD
            seg = [c] * PERIOD
            last_b = node.held
            _advance(ch.sb, lv, seg, last_b)
            # einen Schritt zu spät: sieht in Schritt k noch die alte Eingabe
            ch.last_d = _advance(ch.sd, lv, [node.held] + [c] * (PERIOD - 1),
                                 node.last_d)
            # einen Schritt zu früh: hängt einen Schritt hinterher, bekommt
            # aber schon die Eingaben ab k (beim allerersten Segment nur P-1)
            n_adv = PERIOD - 1 if k == 0 else PERIOD
            ch.last_a = _advance(ch.sa, lv, [c] * n_adv, node.last_a)
            if ch.sb.dead or ch.sd.dead or ch.sa.dead:
                continue
            if ch.sb.won:
                if _finish(ch, lv, c):
                    toggles = []
                    t = ch.toggles
                    while t is not None:
                        toggles.append(t[0])
                        t = t[1]
                    toggles.reverse()
                    return toggles, {"nodes": nodes, "time": time.time() - t0,
                                     "steps": ch.sb.step, "coins": ch.sb.coins}
                continue
            if ch.k > limit_steps:
                continue
            bad = False
            for dl, bit in deadlines:
                if dl > ch.sb.x:
                    break
                # alle drei Varianten müssen die Münze haben
                if not (ch.sb.coins & ch.sd.coins & ch.sa.coins & bit):
                    bad = True
                    break
            if bad:
                continue
            key = (ch.k, ch.sb.mode, ch.sb.grav, ch.sb.speed, ch.sb.ground,
                   ch.sb.coins, len(ch.sb.used), c,
                   min(ch.run, 2 * MIN_HOLD),
                   ch.sb.y // qy, ch.sb.vy // qv,
                   ch.sd.y // qy, ch.sa.y // qy, ch.sd.grav, ch.sa.grav)
            if key in seen:
                continue
            seen.add(key)
            stack.append(ch)
    return None, {"nodes": nodes, "best_x": best_x, "reason": "exhausted",
                  "time": time.time() - t0}


def _finish(node, lv, c):
    """Nach dem Ziel der Basis: die beiden Nachbarn ebenfalls ins Ziel bringen."""
    for _ in range(4 * PERIOD):
        if not node.sd.won:
            node.last_d = _advance(node.sd, lv, [c], node.last_d)
        if not node.sa.won:
            node.last_a = _advance(node.sa, lv, [c], node.last_a)
        if node.sd.dead or node.sa.dead:
            return False
        if node.sd.won and node.sa.won:
            return True
    return False


def verify(level, toggles, shift=0, need_coins=True):
    """Spielt eine Lösung (optional um 'shift' Schritte versetzt) ab.

    Gibt (ok, endzustand) zurück: ok = Ziel erreicht (+ alle Münzen).
    """
    lv = level if isinstance(level, core.Level) else core.Level(level)
    tg = [max(0, t + shift) for t in toggles]
    s, _trace = core.run_toggles(lv, tg)
    ok = bool(s.won) and not s.dead
    if need_coins:
        ok = ok and s.coins == (1 << lv.coin_count) - 1
    return ok, s
