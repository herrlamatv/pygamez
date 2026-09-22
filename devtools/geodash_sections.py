# -*- coding: utf-8 -*-
"""
geodash_sections.py
===================
Der Sektions-Baukasten für die Geometry-Dash-Bibliothek.

Eine **Sektion** ist ein kurzes, in sich abgeschlossenes Level-Stück (eine
Stachelreihe, eine Treppe, eine Orb-Kette, ein Schiffs-Tor-Abschnitt, ein
Wellen-Tunnel ...). ``devtools/build_geodash_extra.py`` reiht Sektionen zu
ganzen Leveln aneinander; der Solver beweist danach, dass jedes davon
schaffbar ist.

Die Maße stammen aus den acht handgebauten Kampagnen-Leveln - dort ist
erprobt, welche Abstände der Würfel schafft (ein Sprung fliegt ~4,3 Blöcke
weit und 2,2 Blöcke hoch, deshalb stehen einzelne Stacheln ~10 Blöcke
auseinander, Treppenstufen sind 3 Blöcke breit und Plattformen liegen auf
Reihe 3 hinter einem Pad).

Damit sich Sektionen BELIEBIG kombinieren lassen, gelten zwei Regeln:

1. **Würfel und Ball** beginnen und enden am Boden (Reihe 0); hinter jeder
   Sektion bleibt eine Sprungweite frei.
2. **Schiff, UFO und Welle** bekommen ihre Hindernisse in einem Korridor
   (``top``); am Anfang und Ende jeder Sektion ist der Korridor frei.

Jede Sektion ist eine Funktion::

    @section("cube_spikes", family="cube", mode=core.CUBE, diff=(0, 3))
    def sec_cube_spikes(L, x, ctx):
        ...
        return {"w": breite, "coins": [(x, y), ...]}

``ctx`` trägt ``speed`` (0-3), ``hard`` (0-2), ``rng``, ``top``
(Korridorhöhe) und ``mode``. ``coins`` sind Stellen, die die erzwungene Bahn
ohnehin streift - der Erzeuger sucht sich daraus drei aus.
"""

import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, "games"))
import geodash_core as core  # noqa: E402

# Sprungweite des Würfels je Tempo in Blöcken: CUBE_JUMP/GRAVITY ergibt ~100
# Schritte Flugzeit, mal SPEEDS[...] geteilt durch B.
JUMP_W = tuple(core.SPEEDS[s] * 100 / core.B for s in range(4))


def beat_for(speed):
    """Taktbreite in Blöcken (eine Sprungweite, mindestens 3)."""
    return max(3, int(round(JUMP_W[speed])))


def space_for(speed):
    """Abstand zwischen zwei Hindernissen am Boden (Sprung + Landung).

    In den Kampagnen-Leveln stehen einzelne Stacheln bei Tempo 1 rund zehn
    Blöcke auseinander - gut zwei Sprungweiten.
    """
    return int(round(2.3 * JUMP_W[speed]))


# ---------------------------------------------------------------------------
#  Registrierung
# ---------------------------------------------------------------------------

class Section:
    """Eine Sektion mit ihren Eigenschaften (siehe Modul-Docstring)."""

    def __init__(self, key, fn, family, mode, speeds, diff, coinable):
        self.key = key
        self.fn = fn
        self.family = family
        self.mode = mode
        self.speeds = tuple(speeds)
        self.diff = tuple(diff)          # (min, max) Schwierigkeits-Bereich
        self.coinable = coinable

    def fits(self, mode, speed, diff):
        return (self.mode == mode and speed in self.speeds
                and self.diff[0] <= diff <= self.diff[1])

    def build(self, L, x, ctx):
        out = self.fn(L, x, ctx) or {}
        w = int(out.get("w", space_for(ctx["speed"])))
        coins = [(int(cx), int(cy)) for cx, cy in out.get("coins", ())]
        return w, (coins if self.coinable else [])

    def __repr__(self):                                  # pragma: no cover
        return "<Section %s>" % self.key


SECTIONS = {}


def section(key, family, mode, speeds=(1, 2, 3), diff=(0, 5), coinable=True):
    def deco(fn):
        SECTIONS[key] = Section(key, fn, family, mode, speeds, diff, coinable)
        return fn
    return deco


def pick(mode, speed, diff):
    """Alle Sektionen, die zu Form, Tempo und Schwierigkeit passen."""
    return [s for s in SECTIONS.values() if s.fits(mode, speed, diff)]


# ---------------------------------------------------------------------------
#  Würfel am Boden
# ---------------------------------------------------------------------------

@section("cube_spikes", family="spikes", mode=core.CUBE, diff=(0, 5))
def sec_cube_spikes(L, x, ctx):
    """Einzelne Stacheln im Takt - das Grundmuster jedes Levels."""
    sp = space_for(ctx["speed"])
    n = 3 + ctx["hard"]
    for i in range(n):
        L.spikes(x + i * sp, 1 + (1 if ctx["hard"] > 1 and i % 2 else 0))
    return {"w": n * sp, "coins": [(x + sp // 2, 4)]}


@section("cube_spike_row", family="spikes", mode=core.CUBE, diff=(1, 5))
def sec_cube_spike_row(L, x, ctx):
    """Zwei- und Dreierreihen - je Reihe ein sauber gesetzter Sprung."""
    sp = space_for(ctx["speed"])
    rows = 3
    for i in range(rows):
        L.spikes(x + i * sp, 2 + (1 if ctx["hard"] > 0 and i == rows - 1 else 0))
    return {"w": rows * sp, "coins": [(x + sp, 4)]}


@section("cube_stairs", family="blocks", mode=core.CUBE, diff=(0, 5))
def sec_cube_stairs(L, x, ctx):
    """Treppe hinauf (drei Blöcke je Stufe, eine Reihe höher) und hinunter."""
    h = 2 + (1 if ctx["hard"] > 0 else 0)
    step = 3 + (1 if ctx["speed"] > 1 else 0)
    cx = x
    for j in range(h):
        L.blocks(cx, 0, step, j + 1)
        cx += step
    for j in range(h - 1, 0, -1):
        L.blocks(cx, 0, step, j)
        cx += step
    return {"w": cx - x + space_for(ctx["speed"]), "coins": [(x + h * step, h + 2)]}


@section("cube_blocks", family="blocks", mode=core.CUBE, diff=(0, 5))
def sec_cube_blocks(L, x, ctx):
    """Flache Blöcke mit Stacheln dazwischen (wie im Würfel-Finale)."""
    sp = space_for(ctx["speed"])
    L.blocks(x, 0, 3, 1)
    L.spikes(x + 3, 1 + ctx["hard"])
    L.blocks(x + sp, 0, 3, 1)
    L.spikes(x + sp + 3, 1)
    return {"w": 2 * sp, "coins": [(x + 1, 3)]}


@section("cube_platform", family="pad", mode=core.CUBE, diff=(0, 5))
def sec_cube_platform(L, x, ctx):
    """Gelbes Pad auf eine Hochebene - oben wartet eine Münze."""
    sp = space_for(ctx["speed"])
    run = 8 + 2 * ctx["hard"]
    L.o("pad_y", x)
    L.blocks(x + 2, 3, run, 1)
    if ctx["hard"] > 0:
        L.spikes(x + 6, 1, 4)
    return {"w": run + sp, "coins": [(x + run - 2, 5)]}


@section("cube_pit", family="pit", mode=core.CUBE, diff=(0, 5))
def sec_cube_pit(L, x, ctx):
    """Gruben im Boden - drüberspringen."""
    sp = space_for(ctx["speed"])
    w = 3 + ctx["hard"]
    n = 2
    for i in range(n):
        L.pit(x + i * sp, w)
    return {"w": n * sp, "coins": [(x + 1, 3)]}


@section("cube_timing", family="blocks", mode=core.CUBE, diff=(2, 5))
def sec_cube_timing(L, x, ctx):
    """Stachel auf dem Block - erst rauf, sofort wieder runter."""
    sp = space_for(ctx["speed"])
    n = 2
    for i in range(n):
        bx = x + i * sp
        L.blocks(bx, 0, 3, 1)
        L.spikes(bx + 1, 1, 1)
    return {"w": n * sp, "coins": [(x + 1, 4)]}


# ---------------------------------------------------------------------------
#  Pads und Orbs
# ---------------------------------------------------------------------------

@section("pad_hop", family="pad", mode=core.CUBE, diff=(0, 5))
def sec_pad_hop(L, x, ctx):
    """Pinkes Pad, dahinter ein Stachel - der Klassiker aus Level 2."""
    sp = space_for(ctx["speed"])
    L.o("pad_p", x)
    L.spikes(x + 2, 1)
    L.spikes(x + sp, 2)
    return {"w": 2 * sp, "coins": [(x + 3, 4)]}


@section("orb_bed", family="orb", mode=core.CUBE, diff=(0, 5))
def sec_orb_bed(L, x, ctx):
    """Orb-Kette über dem Stachelbett (Level 3): klicken, um weiterzufliegen."""
    sp = space_for(ctx["speed"])
    bed = 4 + ctx["hard"]
    L.spikes(x, bed)
    L.o("orb_y", x + 1, 2)
    return {"w": bed + sp, "coins": [(x + 3, 5)]}


@section("orb_double", family="orb", mode=core.CUBE, diff=(1, 5))
def sec_orb_double(L, x, ctx):
    """Zwei Orbs hintereinander - Kette über zwei Betten."""
    sp = space_for(ctx["speed"])
    L.spikes(x, 3)
    L.o("orb_y", x + 1, 2)
    L.spikes(x + sp, 3 + ctx["hard"])
    L.o("orb_y", x + sp + 1, 3)
    return {"w": 2 * sp, "coins": [(x + sp + 2, 6)]}


@section("orb_pink", family="orb", mode=core.CUBE, diff=(2, 5))
def sec_orb_pink(L, x, ctx):
    """Pinker Orb über dem Stachelbett - kleiner Schubs, enger Takt."""
    sp = space_for(ctx["speed"])
    L.o("pad_p", x)
    L.spikes(x + 2, 1)
    L.o("orb_p", x + sp, 2)
    L.spikes(x + sp - 1, 3)
    return {"w": 2 * sp, "coins": [(x + sp, 5)]}


# ---------------------------------------------------------------------------
#  Schwerkraft (Würfel an der Decke)
# ---------------------------------------------------------------------------

@section("grav_run", family="gravity", mode=core.CUBE, diff=(1, 5))
def sec_grav_run(L, x, ctx):
    """Kopfüber unter der Decke (Muster aus Gravity Tide)."""
    sp = space_for(ctx["speed"])
    run = 28 + 6 * ctx["hard"]
    roof_row = 7
    L.o("g_flip", x + 4, 1)
    L.roof(x, run + 8, roof_row)
    L.spikes(x + 9, run - 6)                       # Boden: kein Zurück
    L.spikes(x + 14, 1, roof_row - 1, 2)
    L.spikes(x + 14 + sp, 2 if ctx["hard"] else 1, roof_row - 1, 2)
    L.spikes(x + 14 + 2 * sp, 1, roof_row - 1, 2)
    L.o("g_norm", x + run + 2, roof_row - 2)
    return {"w": run + 8 + sp, "coins": [(x + 16, roof_row - 3)]}


@section("grav_pad", family="gravity", mode=core.CUBE, diff=(2, 5))
def sec_grav_pad(L, x, ctx):
    """Blaues Pad an die Decke und ein zweites wieder zurück."""
    sp = space_for(ctx["speed"])
    run = 24 + 4 * ctx["hard"]
    roof_row = 6
    L.o("pad_b", x + 2, 0)
    L.roof(x, run + 6, roof_row)
    L.spikes(x + 5, run - 4)
    L.spikes(x + 10, 1, roof_row - 1, 2)
    L.spikes(x + 10 + sp, 2 if ctx["hard"] else 1, roof_row - 1, 2)
    L.o("pad_b", x + run + 1, roof_row - 1, 2)
    return {"w": run + 6 + sp, "coins": [(x + 12, roof_row - 3)]}


# ---------------------------------------------------------------------------
#  Ball
# ---------------------------------------------------------------------------

@section("ball_switch", family="ball", mode=core.BALL, diff=(0, 5))
def sec_ball_switch(L, x, ctx):
    """Ball: Stacheln unten und oben im Wechsel (Muster aus Gravity Tide)."""
    top = ctx.get("top", 7)
    step = 12 + 2 * (ctx["speed"] - 1)
    n = 2 + ctx["hard"]
    for i in range(n):
        bx = x + i * step
        if i % 2 == 0:
            L.spikes(bx, 3 + ctx["hard"])
        else:
            L.spikes(bx, 3 + ctx["hard"], top - 1, 2)
    return {"w": n * step + step, "coins": [(x + step // 2, top // 2)]}


@section("ball_pads", family="ball", mode=core.BALL, diff=(1, 5))
def sec_ball_pads(L, x, ctx):
    """Ball: blaue Pads drehen die Schwerkraft über dem Stachelbett."""
    top = ctx.get("top", 7)
    step = 14 + 2 * (ctx["speed"] - 1)
    L.o("pad_b", x, 0)
    L.spikes(x + 3, 5 + ctx["hard"])
    L.o("pad_b", x + step, top - 1, 2)
    L.spikes(x + step + 3, 4, top - 1, 2)
    return {"w": 2 * step, "coins": [(x + 5, top // 2)]}


# ---------------------------------------------------------------------------
#  Schiff
# ---------------------------------------------------------------------------

@section("ship_gates", family="ship", mode=core.SHIP, diff=(0, 5))
def sec_ship_gates(L, x, ctx):
    """Schiff: Tore mit wandernder Lücke (Muster aus Level 1-3)."""
    top = ctx.get("top", 10)
    gap = max(3, 5 - ctx["hard"])
    n = 3
    step = (16 if ctx["speed"] < 2 else 20) + 2 * ctx["hard"]
    rng = ctx["rng"]
    lo = max(1, min(top - gap - 1, ctx.get("gate_lo", 3)))
    mid = lo + gap // 2
    for i in range(n):
        lo = max(1, min(top - gap - 1, lo + rng.choice((-3, -2, 2, 3))))
        if i == n // 2:
            mid = lo + gap // 2
        L.gate(x + step + i * step, lo, lo + gap - 1, top)
    ctx["gate_lo"] = lo
    return {"w": (n + 1) * step + step // 2,
            "coins": [(x + step + (n // 2) * step + 4, mid)]}


@section("ship_tunnel", family="ship", mode=core.SHIP, diff=(2, 5))
def sec_ship_tunnel(L, x, ctx):
    """Schiff: flacher Zickzack-Tunnel - ruhige Hand."""
    top = ctx.get("top", 10)
    half = 3 if ctx["hard"] < 2 else 2
    run = 30
    centers = zigzag_centers(top // 2, half + 1, top - half - 2, run, 3)
    L.tunnel(x + 8, centers, half, top)
    return {"w": run + 20, "coins": [(x + 8 + run // 2, centers[run // 2])]}


# ---------------------------------------------------------------------------
#  UFO
# ---------------------------------------------------------------------------

@section("ufo_gates", family="ufo", mode=core.UFO, diff=(0, 5))
def sec_ufo_gates(L, x, ctx):
    """UFO: Tore mit Platz für ein, zwei Flügelschläge (Muster Level 4)."""
    top = ctx.get("top", 9)
    gap = max(3, 4 - ctx["hard"] // 2)
    n = 3
    step = 14 + 2 * (ctx["speed"] - 1)
    rng = ctx["rng"]
    lo = max(1, min(top - gap - 1, ctx.get("gate_lo", 2)))
    mid = lo + gap // 2
    for i in range(n):
        lo = max(1, min(top - gap - 1, lo + rng.choice((-2, -1, 1, 2))))
        if i == n // 2:
            mid = lo + gap // 2
        L.gate(x + step + i * step, lo, lo + gap - 1, top)
    ctx["gate_lo"] = lo
    return {"w": (n + 1) * step + step // 2,
            "coins": [(x + step + (n // 2) * step + 3, mid)]}


# ---------------------------------------------------------------------------
#  Welle
# ---------------------------------------------------------------------------

@section("wave_lane", family="wave", mode=core.WAVE, diff=(0, 3))
def sec_wave_lane(L, x, ctx):
    """Welle: weiter Zickzack zum Eingewöhnen."""
    top = ctx.get("top", 8)
    half = 2
    run = 30
    centers = zigzag_centers(top // 2, half + 1, top - half - 2, run, 3)
    L.tunnel(x + 8, centers, half, top)
    return {"w": run + 20, "coins": [(x + 8 + run // 3, centers[run // 3])]}


@section("wave_zigzag", family="wave", mode=core.WAVE, diff=(2, 5))
def sec_wave_zigzag(L, x, ctx):
    """Welle: enger Zickzack (Muster aus Wave Reactor)."""
    top = ctx.get("top", 8)
    half = 2 if ctx["hard"] < 2 else 1
    run = 30 + 8 * ctx["hard"]
    step = 2 if ctx["hard"] < 2 else 1
    centers = zigzag_centers(top // 2, half + 1, top - half - 2, run, step)
    L.tunnel(x + 8, centers, half, top)
    return {"w": run + 20, "coins": [(x + 8 + run // 2, centers[run // 2])]}


# ---------------------------------------------------------------------------
#  Rahmen und Übergänge
# ---------------------------------------------------------------------------

def zigzag_centers(start, lo, hi, n, run=1):
    """Mittelreihen eines Zickzacks (wie zigzag() im Baukasten)."""
    out = []
    r, d = start, 1
    while len(out) < n:
        for _ in range(run):
            out.append(r)
        if r >= hi:
            d = -1
        elif r <= lo:
            d = 1
        r += d
    return out[:n]


def intro(L, x, ctx):
    """Ruhiger Anlauf am Anfang jedes Levels."""
    return 18


def outro(L, x, ctx):
    """Auslauf: ein letzter Stachel, dann freie Bahn bis zum Ziel."""
    if ctx.get("mode", core.CUBE) == core.CUBE:
        L.spikes(x + 6, 2)
    return 20


MODE_PORTAL = ("p_cube", "p_ship", "p_ball", "p_ufo", "p_wave")
SPEED_PORTAL = ("s_slow", "s_norm", "s_fast", "s_vfast")


def transition(L, x, ctx, mode_to, speed_to, colors=None, top=9):
    """Übergang in eine neue Form bzw. ein neues Tempo.

    Vom Boden in eine Flugform genügt das Portal auf Reihe 1 (der Würfel
    läuft hinein - so machen es die Kampagnen-Level). Aus einer Flugform
    zurück führt ein Trichter (``LB.funnel``), damit niemand daran
    vorbeifliegt. Gibt die belegte Breite zurück.
    """
    from_mode = ctx.get("mode", core.CUBE)
    w = 0
    if speed_to != ctx.get("speed", 1):
        L.o(SPEED_PORTAL[speed_to], x + 2, 1)
        w = 8
    if mode_to != from_mode:
        if from_mode == core.CUBE:
            par = () if mode_to == core.CUBE else (top,)
            L.o(MODE_PORTAL[mode_to], x + w + 4, 1, 0, *par)
            w += 14
        else:
            L.funnel(x + w, 0, ctx.get("top", 9), 3, MODE_PORTAL[mode_to],
                     *(() if mode_to == core.CUBE else (top,)))
            w += 18
    if colors:
        L.colors(x + 1, colors[0], colors[1])
    return max(w, 10)
