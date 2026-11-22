# -*- coding: utf-8 -*-
"""
lamacrash_draw.py
=================
Zeichen-Bausteine von Lama Crash (nur pygame-Primitive, keine Bilddateien).
Alles Teure wird einmal 2- bis 4-fach überabgetastet gezeichnet, weich
verkleinert und gecacht; die ``draw_*``-Funktionen setzen jedes Bild nur noch
gecachte Teile und billige Primitive zusammen.

**Feste Schnittstelle** (Web ``lamacrash_draw.js`` als ``PG.lamaCrashDraw``
in camelCase, Rust ``lamacrash_draw.rs`` gleichnamig) - Signaturen nur nach
Absprache ändern:

- ``clear_cache()``
- ``infl_level(m100)``        -> Aufblas-Stufe 0 .. INFL_MAX des Ballons
- ``balloon_size(base, level)`` -> (w, h) der Ballon-Fläche:
  ``w = max(12, base * (100 + 4 * level) // 100)``,
  ``h = w * (118 - level) // 100 + base * 9 // 25`` (die Hülle wird breiter
  und runder, die Seile darunter bleiben gleich lang)
- ``balloon(base, level, stripe)`` -> Hülle (Bahnen RUBBER / stripe, je
  praller desto runder, heller und glänzender; eine zu RUBBER ähnliche stripe-
  Farbe wie die rote Spielfarbe wird zu Creme-Rosa aufgehellt, ``_stripe_col``)
  + Schürze + Brenner-Gestell +
  4 Seile (gecacht); Fläche ``balloon_size``; Ankerpunkt = Unterkante Mitte =
  Korbrand-Mitte, die Seile enden an der Unterkante bei ``w/2 ± basket_w(base)/2``
- ``basket_w(base)``          -> Korbbreite zur Grundbreite
- ``basket(w, front)``        -> Korb w x (w*2//3): front=False Rückwand +
  Innenraum (hinter den Lamas), front=True Vorderwand mit Flechtmuster und
  Rand (vor den Lamas); beide an derselben Stelle (oben links = Korbrand)
- ``burner_flame(w, step)``   -> Brenner-Düse + Flamme w x w*3//2 (step 0..5),
  Anker Unterkante Mitte
- ``parachute(w, col)``       -> Fallschirm w x w (Kappe oben, Leinen laufen
  unten Mitte zusammen = Anker für das Lama)
- ``money_sack(n, level)``    -> Geldsack n x n (level 0..2 = voller), Anker Mitte
- ``shreds(w, stripe)``       -> Tupel aus 8 Gummifetzen (kleine Flächen)
- ``pop_star(r, step)``       -> Knall-Stern 2r x 2r (step 0..5: wächst, dann
  verblasst er), ohne Text
- ``pill_color(m100)``        -> (Hintergrund, Schrift) einer Verlaufs-Pille:
  rot unter ×2, grün ab ×2, gold ab ×10, violett ab ×100
- ``pill(w, h, m100, label, font_size)`` -> Verlaufs-Pille (gecacht)
- ``icon(name, size, col)``   -> Piktogramm size x size: auto, stats, fair,
  turbo, music, mute, close, left, right, check, cross, half, ff, play,
  balloon, two
- ``draw_grid(s, rect, ylines, xlines, font_size)`` -> Hilfslinien der Kurve
  (nicht gecacht; ylines = [(y_px, Text)], xlines = [(x_px, Text)], absolute
  Koordinaten in rect)
- ``draw_curve(s, pts, col, base_y, width)`` -> Kurve mit Leuchten und
  halbdurchsichtiger Verlaufsfläche bis base_y (nicht gecacht; pts = absolute
  Punkte, auch Kommazahlen, x steigend)
- ``glow(r, col)``            -> weicher Lichthof (2r+2)² auf Schwarz zum
  additiven Blitten (BLEND_RGB_ADD)
- ``curve_preview(w, h, crash100, marks)`` -> Mini-Kurve fürs Fairness-Fenster
  (bis zum Platzpunkt, marks = Ausstiege m100)
- ``flag(n, col)``            -> Fahne n x n für die Mondlandung (Stangenfuß unten)
- ``llama_size(base)``        -> Größe der Lamas im Korb
- ``draw_rig(s, cx, rim_y, base, level, stripe, riders, flame)`` -> ganzer
  Ballon (Hülle + Seile über dem Korbrand, Rückwand, Lamas, Vorderwand,
  Brenner); riders = [(scarf, pose) oder None] mit 1 Eintrag (Lama mittig)
  oder 2 Einträgen (Platz 0 bei cx - basket_w//4, Platz 1 bei cx + basket_w//4;
  None = schon abgesprungen), flame = 0..5 oder -1 (aus). Ein Lama-Sprite
  (n = llama_size) liegt mit der linken oberen Ecke bei
  ``(cx + dx - n // 2, _rider_foot(rim_y, base) - n)`` - wie ``draw_jumper`` /
  ``draw_faller`` bei Mitte ``(cx + dx, _rider_foot(rim_y, base) - n + n // 2)``.
  Gibt das Rechteck der Hülle (mit Schürze, ohne Seile) zurück.
- ``draw_jumper(s, cx, cy, n, scarf, chute, open_f)`` -> Lama am Fallschirm
  (cx, cy = Mitte des Lama-Sprites, open_f 0..1 = Schirm geht auf)
- ``draw_sack(s, cx, cy, n, chute, open_f)`` -> Geldsack am Mini-Schirm
  (cx, cy = Mitte des Sacks)
- ``draw_faller(s, cx, cy, n, scarf, step)`` -> purzelndes Lama (step 0..7)

Alle Formen hängen nur von Größen, ganzzahligen Werten und Farben ab - kein
Zufall, keine Zeit, und **nie vom Platzpunkt der laufenden Runde** (der Ballon
kennt nur die Aufblas-Stufe). Damit sind Web- und Rust-Fassung gleich baubar.
Farben werden ganzzahlig gemischt (``_lerp_i``); Streuung nur über ``_hash``.
"""

import math

import pygame

import ui

from . import lamacrossing_draw as LCD

_cache = {}     # große Teile: Ballon, Korb, Sprites, Arbeitsfläche
_small = {}     # Texte und Pillen (wechseln oft, eigener kleiner Speicher)

# Aufblas-Stufen: ab diesen Multiplikatoren (x100) wird der Ballon praller
INFL_AT = (100, 110, 125, 150, 200, 300, 500, 1000, 3000, 10000, 100000, 1000000)
INFL_MAX = len(INFL_AT) - 1

RUBBER = (232, 52, 76)
RUBBER_D = (150, 26, 46)
RUBBER_DD = (84, 14, 30)
RUBBER_L = (255, 226, 232)
WICKER = (176, 122, 62)
WICKER_D = (112, 72, 34)
WICKER_DD = (70, 42, 20)
WICKER_L = (222, 176, 108)
LEATHER = (98, 56, 34)
LEATHER_L = (160, 104, 64)
ROPE = (90, 70, 56)
METAL = (138, 144, 162)
METAL_L = (216, 222, 234)
METAL_D = (62, 66, 80)
GOLD = (232, 190, 92)
GOLD_L = (255, 236, 160)
GOLD_D = (160, 112, 34)
BURLAP = (196, 150, 88)
BURLAP_L = (230, 196, 134)
BURLAP_D = (128, 88, 46)
WHITE = (255, 255, 255)
BLACK = (0, 0, 0)

# Sichtbare Bahnen der Hülle (Vorderseite)
GORES = 8
# Lamas im Korb: diese Sprite-Zeile (in %) liegt auf der vorderen Korbkante
RIDER_RIM = 48
# Saum des Fallschirms in Prozent der Höhe (Leinen beginnen hier)
CHUTE_HEM = 38
# Flackern der Brennerflamme: (Höhe %, Spitze seitlich %, Breite %, Zunge -1/0/1)
FLAME_STEPS = ((100, 0, 100, 0), (93, -6, 97, 1), (97, 5, 103, -1),
               (89, -3, 95, 1), (100, 7, 99, -1), (95, -5, 104, 0))
# Knall-Stern: Größe (%) und Deckkraft je Schritt
POP_SCALE = (52, 84, 100, 100, 97, 93)
POP_ALPHA = (255, 255, 255, 215, 140, 64)


# ============================================================= Grundlagen
def _cached(key, build, store=None):
    """Fläche aus dem Speicher holen oder einmal bauen."""
    if store is None:
        store = _cache
    surf = store.get(key)
    if surf is None:
        if len(store) > 400:
            store.clear()
        surf = build()
        store[key] = surf
    return surf


def clear_cache():
    _cache.clear()
    _small.clear()


def _ri(v):
    """Kaufmännisch runden (x.5 nach oben), in Python, JS und Rust gleich."""
    return int(math.floor(v + 0.5))


def _lerp_i(c1, c2, num, den):
    """Ganzzahlige Farbmischung c1 + (c2 - c1) * num / den (abgerundet)."""
    den = max(1, den)
    return tuple(a + (b - a) * num // den for a, b in zip(c1, c2))


def _rgb(col):
    return int(col[0]), int(col[1]), int(col[2])


def _rgba(col, a):
    return col[0], col[1], col[2], max(0, min(255, int(a)))


def _hash(i):
    """Ganzzahl-Streuwert 0..2^32-1 (deterministisch, ohne Zufall/sin)."""
    v = (i * 2654435761 + 40503) % 4294967296
    v = ((v ^ (v >> 15)) * 2246822519) % 4294967296
    return v ^ (v >> 13)


def _layer(w, h):
    """Leere, durchsichtige Fläche."""
    return pygame.Surface((max(1, int(w)), max(1, int(h))), pygame.SRCALPHA)


def _mask(w, h, pts):
    """Stanzmaske für BLEND_RGBA_MIN: innen weiß und deckend, außen durchsichtig."""
    m = _layer(w, h)
    m.fill((255, 255, 255, 0))
    pygame.draw.polygon(m, (255, 255, 255, 255), pts)
    return m


def _stroke(s, col, pts, lw):
    """Dicke Linie mit runden Gelenken und Enden."""
    lw = max(1, int(lw))
    pygame.draw.lines(s, col, False, pts, lw)
    if lw > 2:
        for p in pts:
            pygame.draw.circle(s, col, p, lw / 2.0)


def _radial(s, col, c, r_in, r_out, a_in, a_out, steps, power=1.0):
    """Stufiger Radialverlauf (Alpha ersetzen): von außen (r_out, a_out) nach
    innen (r_in, a_in); jeder kleinere Kreis überschreibt die Mitte."""
    for i in range(steps, -1, -1):
        f = i / steps
        a = a_in + (a_out - a_in) * f ** power
        pygame.draw.circle(s, _rgba(col, a), c, max(1, int(r_in + (r_out - r_in) * f)))


def _ellipse_pts(cx, cy, rx, ry, rot=0.0, n=24):
    """Geschlossene, um rot gedrehte Ellipse als Punktliste."""
    ca, sa = math.cos(rot), math.sin(rot)
    pts = []
    for i in range(n):
        a = math.tau * i / n
        x, y = rx * math.cos(a), ry * math.sin(a)
        pts.append((cx + x * ca - y * sa, cy + x * sa + y * ca))
    return pts


def _arc_band(cx, cy, r_out, thick, a0, a1, n=14):
    """Sichel entlang eines Kreisbogens (Winkel im Bild, y nach unten),
    in der Mitte thick dick, an den Enden spitz."""
    outer, inner = [], []
    for i in range(n + 1):
        f = i / n
        a = a0 + (a1 - a0) * f
        t = thick * math.sin(math.pi * f)
        ca, sa = math.cos(a), math.sin(a)
        outer.append((cx + r_out * ca, cy + r_out * sa))
        inner.append((cx + (r_out - t) * ca, cy + (r_out - t) * sa))
    return outer + inner[::-1]


def _sparkle(s, col, x, y, r):
    """Vierzackiges Funkeln."""
    q = r * 0.26
    pygame.draw.polygon(s, col, [(x, y - r), (x + q, y - q), (x + r, y), (x + q, y + q),
                                 (x, y + r), (x - q, y + q), (x - r, y), (x - q, y - q)])


def _burst(cx, cy, r_out, r_in, spikes, rot, seed, jit=0.22):
    """Gezackter Comic-Stern: Zackenlängen über _hash leicht unregelmäßig."""
    pts = []
    for j in range(spikes * 2):
        a = rot + math.pi * j / spikes
        if j % 2 == 0:
            rr = r_out * (1.0 - jit * (_hash(seed + j) % 100) / 100.0)
        else:
            rr = r_in * (1.0 - 0.12 * (_hash(seed + j * 7) % 100) / 100.0)
        pts.append((cx + math.cos(a) * rr, cy + math.sin(a) * rr))
    return pts


def _smooth(big, w, h):
    return pygame.transform.smoothscale(big, (max(1, int(w)), max(1, int(h))))


# ============================================================= Größen
def infl_level(m100):
    """Aufblas-Stufe 0 .. INFL_MAX zum Multiplikator."""
    lvl = 0
    for i, lo in enumerate(INFL_AT):
        if m100 >= lo:
            lvl = i
    return lvl


def basket_w(base):
    """Korbbreite zur Grundbreite des Ballons."""
    return max(8, base * 9 // 20)


def llama_size(base):
    """Größe der Lamas im Korb (Lama-Sprite n x n)."""
    return max(8, basket_w(base) * 3 // 4)


def balloon_size(base, level):
    """(w, h) der Ballon-Fläche: 4 % breiter je Aufblas-Stufe, die Hülle wird
    dabei runder (h wächst langsamer), die Seilzone (base * 9 // 25) bleibt."""
    level = max(0, min(INFL_MAX, int(level)))
    w = max(12, base * (100 + 4 * level) // 100)
    return w, w * (118 - level) // 100 + base * 9 // 25


def _basket_dims(w):
    """(Höhe, Randwulst, Tiefe der Vorderkante) des Korbs der Breite w."""
    h = max(4, w * 2 // 3)
    t = max(2, h * 14 // 100)
    e = max(t * 2, h * 30 // 100)
    return h, t, e


def _rig(base):
    """Maße unter der Hülle in Pixeln über dem Korbrand:
    (Schürzen-Unterkante, Schürzenhöhe, halbe Halsbreite, halbe Mundbreite,
    Brennerrahmen, Brennerhöhe, halbe Brennerbreite, Flammenbreite)."""
    bw = basket_w(base)
    n = llama_size(base)
    bh, t, e = _basket_dims(bw)
    ears = n * RIDER_RIM // 100 - (e - t // 2)   # so weit ragen die Ohren über den Rand
    frame = max(3, ears + max(2, base // 30))
    burner_h = max(3, base // 20)
    burner_hw = max(2, bw * 11 // 100)
    mouth = max(frame + burner_h + 4, base * 9 // 25)
    skirt = max(2, base * 7 // 100)
    neck_hw = max(2, bw * 34 // 100)
    mouth_hw = max(3, bw * 40 // 100)
    flame_w = max(4, bw * 9 // 25)
    return mouth, skirt, neck_hw, mouth_hw, frame, burner_h, burner_hw, flame_w


def _rider_foot(rim_y, base):
    """y der Lama-Füße im Korb: Sprite-Zeile RIDER_RIM % liegt auf der Vorderkante."""
    n = llama_size(base)
    bh, t, e = _basket_dims(basket_w(base))
    return int(rim_y) + e - t // 2 + n - n * RIDER_RIM // 100


# ============================================================= Ballon
def _profile(R, L, m, level):
    """Halbe Hüllenbreite als Liste (dy, r): vom Pol (dy = -R) über den Äquator
    (dy = 0) bis zum Hals (dy = L, r = m). Schlaff = Tropfen mit leicht hohlem
    Kegel, prall = fast eine Kugel mit kurzem Hals."""
    f = max(0, min(INFL_MAX, level)) / INFL_MAX
    bulge = 0.92 * f ** 1.15
    p = 1.32 - 0.36 * f
    out = []
    for i in range(18):
        a = math.pi / 2 * i / 18
        out.append((-R * math.cos(a), R * math.sin(a)))
    for i in range(23):
        u = i / 22
        g0 = max(0.0, math.cos(u * math.pi / 2)) ** p
        g1 = math.sqrt(max(0.0, 1.0 - u * u))
        out.append((L * u, m + (R - m) * (g0 + (g1 - g0) * bulge)))
    return out


def _gore_bounds():
    """Bahnkanten als Längengrade -pi/2 .. pi/2, um eine halbe Bahn gedreht
    (vorn in der Mitte liegt eine rote Bahn)."""
    out = [-math.pi / 2]
    for k in range(GORES):
        out.append(-math.pi / 2 + (k + 0.5) * math.pi / GORES)
    out.append(math.pi / 2)
    return out


def _stripe_col(stripe):
    """Bahnfarbe mit Kontrast zu RUBBER: zu ähnliche Farben (Summe der
    Kanal-Abstände < 150, z. B. die rote Spielfarbe) werden zu Creme-Rosa."""
    c = _rgb(stripe)
    if abs(c[0] - RUBBER[0]) + abs(c[1] - RUBBER[1]) + abs(c[2] - RUBBER[2]) < 150:
        return _lerp_i(c, (255, 246, 232), 82, 100)
    return c


def balloon(base, level, stripe):
    """Hülle + Schürze + Brenner-Gestell + Seile (gecacht), Fläche balloon_size."""
    level = max(0, min(INFL_MAX, int(level)))
    stripe = _stripe_col(stripe)
    return _cached(("balloon", int(base), level, stripe),
                   lambda: _build_balloon(int(base), level, stripe))


def _build_balloon(base, level, stripe):
    w, h = balloon_size(base, level)
    mouth, skirt, neck_hw, mouth_hw, frame, burner_h, burner_hw, _fw = _rig(base)
    bw = basket_w(base)
    ss = 3
    W, H = w * ss, h * ss
    cx = W / 2.0
    R = (w - 2) * ss / 2.0
    y_eq = ss + R
    y_mouth = (h - mouth) * ss
    y_neck = y_mouth - skirt * ss
    L = max(R * 0.5, y_neck - y_eq)
    m = neck_hw * ss
    mb = mouth_hw * ss
    prof = _profile(R, L, m, level)
    right = [(cx + r, y_eq + dy) for dy, r in prof]
    left = [(cx - r, y_eq + dy) for dy, r in prof]
    sil = right + left[::-1]
    lift = level * 2                                   # dünner Gummi -> heller
    red = _lerp_i(RUBBER, RUBBER_L, lift, 100)
    st = _lerp_i(stripe, WHITE, lift, 100)
    outline = _lerp_i(RUBBER_DD, BLACK, 25, 100)
    bounds = _gore_bounds()

    # --- Hülle: Bahnen
    env = _layer(W, H)
    for j in range(len(bounds) - 1):
        s0, s1 = math.sin(bounds[j]), math.sin(bounds[j + 1])
        poly = [(cx + r * s0, y_eq + dy) for dy, r in prof]
        poly += [(cx + r * s1, y_eq + dy) for dy, r in reversed(prof)]
        pygame.draw.polygon(env, red if j % 2 == 0 else st, poly)
    # Nähte (bei praller Hülle glänzen sie gespannt)
    seam = _layer(W, H)
    sw = max(2, ss * 2 // 3)
    for b in bounds[1:-1]:
        sb = math.sin(b)
        line = [(cx + r * sb, y_eq + dy) for dy, r in prof[2:]]
        pygame.draw.lines(seam, (70, 10, 28, 84), False, line, sw)
        if level >= 3:
            pygame.draw.lines(seam, (255, 240, 244, min(150, 14 + level * 12)), False,
                              [(x - sw, y) for x, y in line[3:]], max(1, sw // 2))
    # Falten der noch schlaffen Hülle (nur Stufe 0..2)
    for j in range(3 - min(3, level)):
        phi = (-0.42, 0.18, 0.62)[j]
        pts = []
        for i, (dy, r) in enumerate(prof):
            if dy >= L * 0.66:
                wob = 0.05 * math.sin(i * 1.7 + j)
                pts.append((cx + r * math.sin(phi + wob), y_eq + dy))
        if len(pts) >= 2:
            pygame.draw.lines(seam, (60, 8, 24, 70), False, pts, ss)
    env.blit(seam, (0, 0))
    # Schatten: Licht von links oben, rechts unten dunkel
    shade = _layer(W, H)
    _radial(shade, (58, 6, 24), (cx - R * 0.36, y_eq - R * 0.42), R * 0.10, R * 2.40,
            0, 196 - level * 6, 30, 1.45)
    env.blit(shade, (0, 0))
    # Gegenlicht am rechten/unteren Rand (Gummi spiegelt)
    bounce = _layer(W, H)
    pygame.draw.polygon(bounce, (255, 150, 160, 40 + level * 4), sil)
    pygame.draw.polygon(bounce, (0, 0, 0, 0), [(x - R * 0.075, y - R * 0.05) for x, y in sil])
    env.blit(bounce, (0, 0))
    # Glanz
    hl = _layer(W, H)
    amax = 54 + level * 9
    for i in range(10, 0, -1):
        f = i / 10
        a = amax * ((11 - i) / 10) ** 1.6
        pygame.draw.polygon(hl, (255, 240, 244, int(a)),
                            _ellipse_pts(cx - R * 0.40, y_eq - R * 0.46, R * 0.52 * f, R * 0.40 * f,
                                         -0.62, 28))
    env.blit(hl, (0, 0))
    hl = _layer(W, H)                       # scharfe Glanzformen darüber
    pygame.draw.polygon(hl, (255, 255, 255, 168 + level * 7),
                        _arc_band(cx, y_eq, R * 0.88, R * (0.085 + 0.005 * level),
                                  math.pi * 1.10, math.pi * (1.34 + 0.008 * level), 16))
    pa = math.pi * (1.40 + 0.006 * level)
    dot = (cx + R * 0.82 * math.cos(pa), y_eq + R * 0.82 * math.sin(pa))
    pygame.draw.circle(hl, (255, 255, 255, 190 + level * 5), dot, R * (0.030 + 0.0015 * level))
    if level >= 4:
        # gespannte Haut: Randlicht links unten am Bauch
        k = level - 3
        rim = []
        inner = []
        sel = [(dy, r) for dy, r in prof if 0 <= dy <= L * (0.40 + 0.03 * k)]
        for i, (dy, r) in enumerate(sel):
            t = math.sin(math.pi * i / max(1, len(sel) - 1))
            rim.append((cx - r * 0.955, y_eq + dy))
            inner.append((cx - r * (0.955 - (0.03 + 0.004 * k) * t), y_eq + dy))
        if len(rim) >= 2:
            pygame.draw.polygon(hl, (255, 236, 240, 50 + 14 * k), rim + inner[::-1])
    if level >= 6:
        # Spannungs-Streifen neben dem Glanz
        for j in range(1 + (level - 6) // 2):
            pygame.draw.polygon(hl, (255, 255, 255, 70 + 10 * j),
                                _arc_band(cx, y_eq, R * (0.74 - 0.07 * j), R * 0.022,
                                          math.pi * (1.17 + 0.02 * j), math.pi * (1.31 - 0.01 * j), 10))
    if level >= 8:
        # zweiter Lichtreflex rechts oben
        pygame.draw.polygon(hl, (255, 250, 252, 60 + 12 * (level - 8)),
                            _arc_band(cx, y_eq, R * 0.86, R * 0.05, math.pi * 1.66, math.pi * 1.80, 10))
    if level >= 10:
        _sparkle(hl, (255, 255, 255, 230), dot[0] + R * 0.09, dot[1] + R * 0.05, R * 0.085)
    env.blit(hl, (0, 0))
    env.blit(_mask(W, H, sil), (0, 0), special_flags=pygame.BLEND_RGBA_MIN)
    olw = max(2, ss * 4 // 3)
    pygame.draw.lines(env, outline, False, right, olw)
    pygame.draw.lines(env, outline, False, left, olw)
    big = env                               # Schürze, Seile, Brenner kommen dazu

    # --- Schürze (Bahnen setzen sich dunkler fort) + Ring am Hals
    yn = y_neck - ss

    def skx(y, sb):
        t = (y - yn) / max(1.0, y_mouth - yn)
        return cx + (m + (mb - m) * t) * sb
    for j in range(len(bounds) - 1):
        s0, s1 = math.sin(bounds[j]), math.sin(bounds[j + 1])
        mid = (s0 + s1) / 2
        col = _lerp_i(red if j % 2 == 0 else st, (40, 8, 20), 28 + int(24 * (mid + 1)), 100)
        pygame.draw.polygon(big, col, [(skx(yn, s0), yn), (skx(y_mouth, s0), y_mouth),
                                       (skx(y_mouth, s1), y_mouth), (skx(yn, s1), yn)])
    pygame.draw.line(big, outline, (cx - m, yn), (cx - mb, y_mouth), olw)
    pygame.draw.line(big, outline, (cx + m, yn), (cx + mb, y_mouth), olw)
    ring = max(ss * 2, skirt * ss // 3)
    pygame.draw.rect(big, outline, (int(cx - m - ss), int(yn - ring // 2), int(2 * m + 2 * ss), ring),
                     border_radius=ring // 2)
    pygame.draw.line(big, _lerp_i(red, WHITE, 30, 100), (cx - m * 0.75, yn - ring // 4),
                     (cx - m * 0.15, yn - ring // 4), max(1, ss // 2))
    # Mundöffnung (dunkel) mit hellem Saum vorn
    ry = max(ss * 1.5, mb * 0.2)
    pygame.draw.ellipse(big, (34, 10, 18), (cx - mb, y_mouth - ry, 2 * mb, 2 * ry))
    hem = [(cx + mb * math.cos(math.pi * i / 12), y_mouth + ry * math.sin(math.pi * i / 12))
           for i in range(13)]
    _stroke(big, _lerp_i(red, (40, 8, 20), 20, 100), hem, ss)

    # --- Seile, Brennerrahmen, Brenner
    rlw = max(ss, ss * base // 100)
    yb = H + ss
    cl, cr = cx - bw * ss / 2.0, cx + bw * ss / 2.0
    il = (cx - mb * 0.40, y_mouth + ry * 0.7)
    ir = (cx + mb * 0.40, y_mouth + ry * 0.7)
    rope_d = _lerp_i(ROPE, BLACK, 35, 100)
    pygame.draw.line(big, rope_d, il, (cl, yb), rlw)
    pygame.draw.line(big, rope_d, ir, (cr, yb), rlw)
    yf = H - frame * ss
    tf = (yf - il[1]) / max(1.0, yb - il[1])
    xl = il[0] + (cl - il[0]) * tf
    xr = ir[0] + (cr - ir[0]) * tf
    bar = max(ss, ss * base // 75)
    pygame.draw.rect(big, METAL_D, (int(xl), int(yf - bar / 2), int(xr - xl), bar),
                     border_radius=max(1, bar // 2))
    pygame.draw.line(big, METAL, (xl + bar, yf - bar / 4), (xr - bar, yf - bar / 4), max(1, bar // 3))
    bhw = burner_hw * ss
    bt = yf - burner_h * ss
    body = pygame.Rect(int(cx - bhw), int(bt), int(2 * bhw), int(yf - bt + bar // 2))
    pygame.draw.rect(big, METAL_D, body, border_radius=ss * 2)
    pygame.draw.rect(big, METAL, (body.x + ss, body.y + ss, body.w * 11 // 20, body.h - ss * 2),
                     border_radius=ss)
    pygame.draw.line(big, METAL_L, (body.x + ss * 2, body.y + ss * 2), (body.x + ss * 2, body.bottom - ss * 2),
                     max(1, ss * 2 // 3))
    for fy in (0.38, 0.68):
        yy = body.y + body.h * fy
        pygame.draw.line(big, METAL_D, (body.x + ss, yy), (body.right - ss, yy), max(1, ss * 2 // 3))
    pygame.draw.ellipse(big, (28, 28, 36), (body.x + ss, body.y - ss, body.w - 2 * ss, ss * 2))
    pygame.draw.line(big, ROPE, (cx - mb, y_mouth), (cl, yb), rlw)
    pygame.draw.line(big, ROPE, (cx + mb, y_mouth), (cr, yb), rlw)
    return _smooth(big, w, h)


# ============================================================= Korb
def basket(w, front):
    """Korb w x (w*2//3): front=False Rückwand + Innenraum, front=True
    Vorderwand mit Flechtmuster, Randwulst und Eckpfosten."""
    w = max(8, int(w))
    front = bool(front)

    def build():
        bh, t, e = _basket_dims(w)
        ss = 3
        big = _layer(w * ss, bh * ss)
        if front:
            _basket_front(big, w * ss, bh * ss, t * ss, e * ss, ss)
        else:
            _basket_back(big, w * ss, bh * ss, t * ss, e * ss, ss)
        return _smooth(big, w, bh)
    return _cached(("basket", w, front), build)


def _basket_back(s, W, H, T, E, ss):
    """Innenseite der Rückwand (dunkles Geflecht) + hinterer Randwulst."""
    x0, x1 = int(T * 0.5), int(W - T * 0.5)
    y0, y1 = int(T * 0.5), int(min(H, E + T))
    for y in range(y0, y1):
        s.fill(_lerp_i((100, 66, 36), (44, 26, 12), y - y0, y1 - y0), (x0, y, x1 - x0, 1))
    step = max(ss * 3, T)
    for y in range(y0 + step // 2, y1, step):
        pygame.draw.line(s, (36, 22, 10), (x0, y), (x1, y), max(1, ss // 2))
        off = (y // step) % 2 * step
        for x in range(x0 + off, x1, step * 2):
            pygame.draw.line(s, (128, 88, 50), (x + ss, y - step // 2 + ss), (x + step - ss, y - step // 2 + ss),
                             max(1, ss // 2))
    rim = _lerp_i(LEATHER, BLACK, 28, 100)
    pygame.draw.rect(s, rim, (0, 0, W, T), border_radius=T // 2)
    pygame.draw.line(s, LEATHER, (T // 2, T // 3), (W - T // 2, T // 3), max(1, ss // 2))


def _basket_front(s, W, H, T, E, ss):
    """Vorderwand: Ziegel-Flechtmuster, Schatten rechts, Bodenleiste, Rand vorn."""
    inset = W * 5 // 100
    rad = max(ss * 2, (H - E) // 4)
    wall = [(0, E), (W, E)]
    for i in range(7):
        a = math.pi / 2 * i / 6
        wall.append((W - inset - rad + rad * math.cos(a), H - rad + rad * math.sin(a)))
    for i in range(7):
        a = math.pi / 2 + math.pi / 2 * i / 6
        wall.append((inset + rad + rad * math.cos(a), H - rad + rad * math.sin(a)))
    lay = _layer(W, H)
    lay.fill(WICKER_DD)
    # Geflecht: Reihen versetzter, gewölbter Stäbe
    top = min(E + T // 2, H - ss * 2)
    rows = max(1, _ri((H - top) / (ss * 5.5)))
    rh = max(float(ss), (H - top) / rows)
    sw = rh * 1.9
    gap = max(1, ss * 2 // 3)
    for r in range(rows):
        y = top + r * rh
        off = (r % 2) * sw / 2
        x = -sw + off
        while x < W + sw:
            box = (x + gap, y + gap, sw - 2 * gap, rh - 2 * gap)
            pygame.draw.ellipse(lay, WICKER, box)
            pygame.draw.ellipse(lay, WICKER_L, (x + gap * 2, y + gap * 1.5, sw - 4 * gap, (rh - 2 * gap) * 0.45))
            x += sw
    # Schatten: rechts und unten dunkler, links Lichtkante
    sh = _layer(W, H)
    bands = 12
    for i in range(bands):
        x = W * (40 + 60 * i // bands) // 100
        sh.fill((30, 14, 4, 8 + 6 * i), (x, 0, W - x, H))
    for i in range(6):
        y = int(H - (H - E) * (6 - i) / 14)
        sh.fill((30, 14, 4, 10 + 6 * i), (0, y, W, H - y))
    sh.fill((20, 10, 2, 90), (0, E + T // 2, W, max(ss, T // 2)))      # Schatten unter dem Wulst
    lay.blit(sh, (0, 0))
    pygame.draw.line(lay, WICKER_L, (ss, E), (inset + ss, H - rad), max(1, ss))
    pygame.draw.rect(lay, WICKER_DD, (0, int(H - T * 0.8), W, int(T * 0.8) + 1))
    pygame.draw.line(lay, WICKER_D, (0, int(H - T * 0.8)), (W, int(H - T * 0.8)), max(1, ss // 2))
    lay.blit(_mask(W, H, wall), (0, 0), special_flags=pygame.BLEND_RGBA_MIN)
    s.blit(lay, (0, 0))
    pygame.draw.lines(s, WICKER_DD, False, wall[1:] + [wall[0]], max(1, ss * 2 // 3))
    # Eckpfosten (verbinden hinteren und vorderen Rand) mit Seil-Ringen
    pw = int(T * 0.9)
    for x in (0, W - pw):
        pygame.draw.rect(s, LEATHER, (x, 0, pw, E + T // 2), border_radius=max(1, pw // 3))
        pygame.draw.line(s, LEATHER_L, (x + pw // 3, ss), (x + pw // 3, E), max(1, ss // 2))
        pygame.draw.circle(s, METAL_D, (x + pw / 2, T * 0.45), T * 0.36)
        pygame.draw.circle(s, METAL_L, (x + pw / 2, T * 0.45), T * 0.36, max(1, ss * 2 // 3))
    # vorderer Randwulst
    pygame.draw.rect(s, _lerp_i(LEATHER, BLACK, 30, 100), (0, E - T // 2 + ss // 2, W, T), border_radius=T // 2)
    pygame.draw.rect(s, LEATHER, (0, E - T // 2, W, T - ss // 2), border_radius=T // 2)
    pygame.draw.line(s, LEATHER_L, (T // 2, E - T // 4), (W - T // 2, E - T // 4), max(1, ss * 2 // 3))
    # Nähte am Wulst
    step = max(ss * 4, T * 2)
    for x in range(T, W - T, step):
        pygame.draw.line(s, _lerp_i(LEATHER, BLACK, 40, 100), (x, E - T // 2 + ss), (x - ss, E + T // 2 - ss),
                         max(1, ss // 2))


# ============================================================= Brenner
def _flame_pts(cx, by, hw, fh, sway, n=14):
    """Flammenumriss: schmal an der Düse, bauchig, oben spitz (Spitze um sway versetzt)."""
    right, left = [], []
    for i in range(n + 1):
        u = i / n
        wdt = hw * math.sqrt(min(1.0, u * 5.0 + 0.04)) * (1.0 - u) ** 1.15
        x = cx + sway * u * u
        y = by - fh * u
        right.append((x + wdt, y))
        left.append((x - wdt, y))
    return right + left[::-1]


def burner_flame(w, step):
    """Düse + Flamme w x w*3//2 (blauer Kern, gelb/orange außen), 6 Flacker-Stufen.

    Anker: Unterkante Mitte (= Oberkante des Brenners im Gestell)."""
    w = max(4, int(w))
    step = int(step) % 6

    def build():
        fh_img = w * 3 // 2
        ss = 4
        W, H = w * ss, fh_img * ss
        big = _layer(W, H)
        cx = W / 2.0
        hp, sway_p, wp, tongue = FLAME_STEPS[step]
        noz = max(ss * 2, H * 9 // 100)
        by = H - noz
        fh = (by - ss) * hp / 100.0
        hw = W * 0.36 * wp / 100.0
        sway = W * sway_p / 100.0
        pygame.draw.polygon(big, (255, 140, 40, 64), _flame_pts(cx, by, hw * 1.32, fh, sway))
        if tongue:
            pygame.draw.polygon(big, (255, 150, 44, 220),
                                _flame_pts(cx + tongue * hw * 0.55, by - fh * 0.10, hw * 0.34, fh * 0.42,
                                           tongue * W * 0.10, 10))
        pygame.draw.polygon(big, (255, 118, 30, 238), _flame_pts(cx, by, hw, fh * 0.94, sway))
        pygame.draw.polygon(big, (255, 192, 58, 250), _flame_pts(cx, by, hw * 0.70, fh * 0.74, sway * 0.8))
        pygame.draw.polygon(big, (255, 242, 176), _flame_pts(cx, by, hw * 0.42, fh * 0.50, sway * 0.5))
        pygame.draw.polygon(big, (96, 162, 255, 236), _flame_pts(cx, by, hw * 0.32, fh * 0.22, 0, 8))
        pygame.draw.polygon(big, (220, 238, 255), _flame_pts(cx, by, hw * 0.14, fh * 0.12, 0, 6))
        # Düse
        nw = W * 0.40
        pygame.draw.rect(big, METAL_D, (int(cx - nw / 2), int(by), int(nw), int(noz)), border_radius=ss)
        pygame.draw.rect(big, METAL, (int(cx - nw / 2 + ss), int(by + ss // 2), int(nw * 0.45), int(noz - ss)),
                         border_radius=ss // 2)
        pygame.draw.ellipse(big, (24, 24, 30), (cx - nw / 2 + ss, by - ss, nw - 2 * ss, ss * 2))
        return _smooth(big, w, fh_img)
    return _cached(("flame", w, step), build)


# ============================================================= Fallschirm
def parachute(w, col):
    """Fallschirm w x w: Kuppel mit Bahnen in col und Weiß, gewellter Saum,
    Leinen laufen unten in der Mitte zusammen (Anker)."""
    w = max(6, int(w))
    col = _rgb(col)

    def build():
        ss = 3
        N = w * ss
        big = _layer(N, N)
        cx = N / 2.0
        rx = N * 0.47
        top = N * 0.05
        hem = N * CHUTE_HEM / 100.0
        ry = hem - top
        conv = (cx, N - ss * 0.5)
        np_ = 6
        bounds = [-math.pi / 2 + k * math.pi / np_ for k in range(np_ + 1)]
        white = (250, 248, 242)
        dark = _lerp_i(col, BLACK, 45, 100)

        def mer(sb, th):
            return cx + rx * sb * math.sin(th), hem - ry * math.cos(th)
        sil_top = []
        for j in range(np_):
            s0, s1 = math.sin(bounds[j]), math.sin(bounds[j + 1])
            x0, x1 = cx + rx * s0, cx + rx * s1
            scal = rx * (s1 - s0) * 0.16
            pts = [mer(s0, math.pi / 2 * i / 10) for i in range(11)]
            edge = [(x0 + (x1 - x0) * i / 8, hem - scal * math.sin(math.pi * i / 8)) for i in range(1, 8)]
            pts += edge
            pts += [mer(s1, math.pi / 2 * i / 10) for i in range(10, -1, -1)]
            mid = (s0 + s1) / 2
            base_col = col if j % 2 == 0 else white
            pygame.draw.polygon(big, _lerp_i(base_col, BLACK, int(8 + 16 * (mid + 1)), 100), pts)
            sil_top.append((x0, hem))
            sil_top += edge
        sil_top.append((cx + rx, hem))
        # Glanz links oben, Bahnkanten
        hl = _layer(N, N)
        band = _arc_band(0, 0, 0.84, 0.16, math.pi * 1.14, math.pi * 1.40, 12)
        pygame.draw.polygon(hl, (255, 255, 255, 96), [(cx + rx * x, hem + ry * y) for x, y in band])
        big.blit(hl, (0, 0))
        for b in bounds[1:-1]:
            sb = math.sin(b)
            pygame.draw.lines(big, _lerp_i(dark, BLACK, 20, 100), False,
                              [mer(sb, math.pi / 2 * i / 10) for i in range(1, 11)], max(1, ss // 2))
        outline = [mer(-1.0, math.pi / 2 * i / 10) for i in range(10, -1, -1)]
        outline += [mer(1.0, math.pi / 2 * i / 10) for i in range(1, 11)]
        pygame.draw.lines(big, dark, False, outline, max(2, ss))
        pygame.draw.lines(big, dark, False, sil_top, max(1, ss * 2 // 3))
        # Leinen
        llw = max(2, ss * w // 70)
        for b in bounds:
            pygame.draw.line(big, (74, 70, 82), (cx + rx * math.sin(b), hem), conv, llw)
        pygame.draw.circle(big, (74, 70, 82), conv, llw)
        return _smooth(big, w, w)
    return _cached(("chute", w, col), build)


def _chute_stage(cw, col, k):
    """Schirm in Öffnungsstufe k (0..8): erst ein schmaler Schlauch an langen
    Leinen, dann bläht er sich auf; Fläche cw x cw, Anker unten Mitte."""
    col = _rgb(col)
    if k >= 8:
        return parachute(cw, col)

    def build():
        full = parachute(cw, col)
        hem = cw * CHUTE_HEM // 100
        f = k / 8.0
        sw = max(2, int(cw * (0.16 + 0.80 * f)))
        sh = max(2, int((hem + 1) * (0.82 + 0.18 * f)))
        can = _smooth(full.subsurface((0, 0, cw, hem + 1)), sw, sh)
        out = _layer(cw, cw)
        x0 = (cw - sw) // 2
        out.blit(can, (x0, hem + 1 - sh))
        conv = (cw / 2.0, cw - 1)
        for fx in (0.06, 0.36, 0.64, 0.94):
            pygame.draw.aaline(out, (74, 70, 82), (x0 + sw * fx, hem), conv)
        return out
    return _cached(("chute_k", cw, col, k), build)


# ============================================================= Geldsack
def money_sack(n, level):
    """Geldsack n x n (Anker Mitte): Jute mit Zipfel, Kordel und goldener Münze;
    level 0..2 = voller (2: Münzen schauen oben heraus)."""
    n = max(6, int(n))
    level = max(0, min(2, int(level)))

    def build():
        ss = 3
        N = n * ss
        big = _layer(N, N)
        cx = N / 2.0
        brx = N * (0.31 + 0.045 * level)
        bry = N * (0.25 + 0.025 * level)
        bcy = N * 0.95 - bry
        neck_y = N * 0.34
        nhw = N * 0.085
        right = []
        for i in range(10):
            t = i / 10
            right.append((cx + nhw + (brx - nhw) * math.sin(t * math.pi / 2) ** 1.3,
                          neck_y + (bcy - neck_y) * t))
        for i in range(11):
            a = math.pi / 2 * i / 10
            right.append((cx + brx * math.cos(a), bcy + bry * math.sin(a)))
        body = right + [(2 * cx - x, y) for x, y in reversed(right)]
        # Münzen oben (nur voller Sack), dahinter der Zipfel
        if level >= 2:
            for dx, dy, rot in ((-0.07, 0.15, -0.3), (0.06, 0.12, 0.25), (0.0, 0.08, 0.0)):
                pts = _ellipse_pts(cx + N * dx, N * dy, N * 0.075, N * 0.05, rot, 16)
                pygame.draw.polygon(big, GOLD_D, [(x, y + ss) for x, y in pts])
                pygame.draw.polygon(big, GOLD, pts)
                pygame.draw.line(big, GOLD_L, (cx + N * (dx - 0.04), N * (dy - 0.015)),
                                 (cx + N * (dx + 0.02), N * (dy - 0.03)), max(1, ss))
        tuft = [(cx - nhw, neck_y), (cx - N * 0.17, N * 0.13), (cx - N * 0.09, N * 0.19), (cx - N * 0.04, N * 0.10),
                (cx + N * 0.03, N * 0.18), (cx + N * 0.10, N * 0.11), (cx + N * 0.16, N * 0.17), (cx + nhw, neck_y)]
        pygame.draw.polygon(big, BURLAP_D, tuft)
        pygame.draw.polygon(big, BURLAP, [(x + (cx - x) * 0.18, y + (neck_y - y) * 0.15) for x, y in tuft])
        # Körper mit Schatten und Jute-Muster
        lay = _layer(N, N)
        pygame.draw.polygon(lay, BURLAP, body)
        tex = _layer(N, N)
        step = max(ss * 2, N // 14)
        for i in range(-N, N, step):
            pygame.draw.line(tex, (110, 74, 38, 46), (i, 0), (i + N, N), max(1, ss // 2))
            pygame.draw.line(tex, (250, 220, 170, 30), (i + N, 0), (i, N), max(1, ss // 2))
        lay.blit(tex, (0, 0))
        sh = _layer(N, N)
        _radial(sh, (80, 46, 16), (cx - N * 0.14, bcy - bry * 0.55), N * 0.08, N * 0.80, 0, 150, 16, 1.4)
        pygame.draw.polygon(sh, (255, 236, 196, 70),
                            _arc_band(cx, bcy, brx * 0.86, brx * 0.14, math.pi * 1.05, math.pi * 1.35, 10))
        lay.blit(sh, (0, 0))
        lay.blit(_mask(N, N, body), (0, 0), special_flags=pygame.BLEND_RGBA_MIN)
        big.blit(lay, (0, 0))
        pygame.draw.lines(big, _lerp_i(BURLAP_D, BLACK, 30, 100), True, body, max(2, ss))
        # Kordel mit Schleife
        cord = (124, 82, 46)
        pygame.draw.rect(big, _lerp_i(cord, BLACK, 35, 100),
                         (int(cx - nhw * 1.45), int(neck_y - N * 0.035), int(nhw * 2.9), int(N * 0.07)),
                         border_radius=int(N * 0.03))
        pygame.draw.rect(big, cord, (int(cx - nhw * 1.35), int(neck_y - N * 0.03), int(nhw * 2.7), int(N * 0.05)),
                         border_radius=int(N * 0.025))
        lw = max(ss, N // 30)
        pygame.draw.ellipse(big, cord, (cx + nhw * 1.1, neck_y - N * 0.09, N * 0.10, N * 0.08), lw)
        _stroke(big, cord, [(cx + nhw * 1.2, neck_y), (cx + nhw * 1.9, neck_y + N * 0.07),
                            (cx + nhw * 1.7, neck_y + N * 0.12)], lw)
        # goldene Münze als Wappen
        r = N * (0.13 + 0.012 * level)
        mc = (cx, bcy - bry * 0.08)
        pygame.draw.circle(big, _lerp_i(GOLD_D, BLACK, 25, 100), (mc[0], mc[1] + ss), r)
        pygame.draw.circle(big, GOLD_D, mc, r)
        pygame.draw.circle(big, GOLD, mc, r * 0.84)
        pygame.draw.circle(big, _lerp_i(GOLD, GOLD_D, 1, 3), mc, r * 0.64, max(1, ss // 2 + 1))
        star = []
        for j in range(10):
            a = -math.pi / 2 + math.pi * j / 5
            rr = r * (0.46 if j % 2 == 0 else 0.19)
            star.append((mc[0] + math.cos(a) * rr, mc[1] + math.sin(a) * rr))
        pygame.draw.polygon(big, GOLD_D, star)
        pygame.draw.polygon(big, GOLD_L, _arc_band(mc[0], mc[1], r * 0.80, r * 0.16, math.pi * 1.05, math.pi * 1.45, 8))
        if level >= 2:
            _sparkle(big, WHITE, cx + N * 0.20, N * 0.08, N * 0.07)
        return _smooth(big, n, n)
    return _cached(("sack", n, level), build)


# ============================================================= Platzen
def _shred_poly(S, v, flap):
    """Umriss eines Gummifetzens in einer Fläche S x S (Streuwert v).

    flap=False: gebogener Streifen mit ausgefransten Enden, sonst ein
    gezackter Lappen."""
    c = S / 2.0
    rot = (v >> 5) % 360 * math.pi / 180
    ca, sa = math.cos(rot), math.sin(rot)
    pts = []
    if flap:
        k = 11
        for j in range(k):
            a = math.tau * j / k
            rr = c * (0.62 + ((v >> (j * 3 % 27)) % 34) / 100.0)
            if j % 4 == 1:
                rr *= 0.62                                      # Riss-Kerbe
            pts.append((math.cos(a) * rr, math.sin(a) * rr * 0.74))
    else:
        ln = S * (0.84 + (v % 12) / 100.0)
        th = S * (0.20 + ((v >> 4) % 12) / 100.0)
        bend = ((v >> 9) % 100) / 100.0 * 1.2 - 0.6
        top, bot = [], []
        for i in range(9):
            t = i / 8.0 - 0.5
            x, y = t * ln, bend * (t * t - 0.125) * ln
            d = 2.0 * bend * t
            nl = math.hypot(1.0, d)
            hw = th * (0.62 + 0.38 * math.cos(t * math.pi)) * (0.86 + 0.14 * ((v >> (i + 2)) % 2)) / 2
            top.append((x + d / nl * hw, y - hw / nl))
            bot.append((x - d / nl * hw, y + hw / nl))
        # ausgefranste Enden (Zickzack)
        def tear(a, b, sign, seed):
            out = []
            for j in (1, 2, 3):
                f = j / 4.0
                jag = th * (0.30 if j % 2 else -0.12) * (0.7 + 0.3 * ((seed >> j) % 2))
                out.append((a[0] + (b[0] - a[0]) * f + sign * jag, a[1] + (b[1] - a[1]) * f))
            return out
        pts = top + tear(top[-1], bot[-1], 1, v >> 3) + bot[::-1] + tear(bot[0], top[0], -1, v >> 7)
    return [(c + x * ca - y * sa, c + x * sa + y * ca) for x, y in pts]


def shreds(w, stripe):
    """8 Gummifetzen (abwechselnd RUBBER / stripe wie die Hülle): gebogene
    Streifen und gezackte Lappen, Größe ~ w/5 .. w/3."""
    w = max(8, int(w))
    stripe = _stripe_col(stripe)

    def build():
        out = []
        for i in range(8):
            v = _hash(i * 7 + 3)
            sz = max(5, w * (20 + v % 13) // 100)
            ss = 3
            S = sz * ss
            big = _layer(S, S)
            col = RUBBER if i % 2 == 0 else stripe
            pts = _shred_poly(S, v, i % 3 == 2)
            pygame.draw.polygon(big, _lerp_i(col, BLACK, 50, 100), pts)
            c = S / 2.0
            inner = [(c + (x - c) * 0.86 - S * 0.02, c + (y - c) * 0.80 - S * 0.03) for x, y in pts]
            pygame.draw.polygon(big, col, inner)
            # Glanz: helle Linie nahe der Oberkante
            gl = [(c + (x - c) * 0.55 - S * 0.03, c + (y - c) * 0.45 - S * 0.06) for x, y in inner[1:len(inner) // 3]]
            if len(gl) >= 2:
                _stroke(big, _lerp_i(col, WHITE, 60, 100), gl, max(ss, S // 14))
            out.append(_smooth(big, sz, sz))
        return tuple(out)
    return _cached(("shreds", w, stripe), build)


def pop_star(r, step):
    """Comic-Knall 2r x 2r (ohne Text): weißer Kern, gelb, orange, dunkler Rand.
    step 0..5: wächst bis Schritt 2, dann wird er hohl und verblasst."""
    r = max(4, int(r))
    step = int(step) % 6

    def build():
        ss = 3
        S = 2 * r * ss
        big = _layer(S, S)
        c = S / 2.0
        sc = POP_SCALE[step] / 100.0 * 0.86
        a = POP_ALPHA[step]
        rot = 0.12 * step
        R = c * sc
        # dunkler Rand verblasst schneller als der Stern (sonst bleibt ein dunkler Ring)
        pygame.draw.polygon(big, (120, 28, 18, a * a // 255), _burst(c, c, R * 1.06, R * 0.58, 14, rot, 11))
        pygame.draw.polygon(big, (255, 112, 36, a), _burst(c, c, R, R * 0.54, 14, rot, 11))
        pygame.draw.polygon(big, (255, 212, 62, a), _burst(c, c, R * 0.74, R * 0.42, 14, rot + 0.11, 23))
        pygame.draw.polygon(big, (255, 252, 230, a), _burst(c, c, R * 0.44, R * 0.32, 10, rot + 0.2, 37, 0.15))
        if step >= 4:
            # hohl werden: Mitte ausstanzen
            pygame.draw.polygon(big, (0, 0, 0, 0), _burst(c, c, R * (0.30 if step == 4 else 0.56),
                                                         R * (0.24 if step == 4 else 0.44), 10, rot, 41, 0.1))
        if 1 <= step <= 3:
            # Speed-Linien außen herum
            lw = max(ss, S // 60)
            for j in range(10):
                ang = math.tau * j / 10 + 0.3 + step * 0.05
                r0 = c * (0.90 if step < 3 else 0.93)
                r1 = c * 0.99
                pygame.draw.line(big, (255, 236, 150, a), (c + math.cos(ang) * r0, c + math.sin(ang) * r0),
                                 (c + math.cos(ang) * r1, c + math.sin(ang) * r1), lw)
        return _smooth(big, 2 * r, 2 * r)
    return _cached(("pop", r, step), build)


# ============================================================= Verlauf (Pillen)
def pill_color(m100):
    """(Hintergrund, Schrift) einer Verlaufs-Pille: rot < ×2, grün ab ×2,
    gold ab ×10, violett ab ×100."""
    if m100 >= 10000:
        return (142, 70, 222), (255, 255, 255)
    if m100 >= 1000:
        return (222, 168, 48), (46, 28, 6)
    if m100 >= 200:
        return (36, 148, 100), (240, 255, 246)
    return (168, 44, 60), (255, 236, 238)


def _fit_font(text, max_w, size, min_size=7):
    """Größte fette Schrift (ab size abwärts), in der text in max_w passt."""
    while size > min_size and ui.font(size, bold=True).size(text)[0] > max_w:
        size -= 1
    return ui.font(size, bold=True)


def pill(w, h, m100, label, font_size):
    """Verlaufs-Pille w x h: Verlauf, Unterkante, Glanzlinie, Beschriftung."""
    w, h = max(6, int(w)), max(6, int(h))

    def build():
        bg, fg = pill_color(m100)
        ss = 2
        W, H = w * ss, h * ss
        big = _layer(W, H)
        rad = H // 2
        pygame.draw.rect(big, _lerp_i(bg, BLACK, 50, 100), (0, ss, W, H - ss), border_radius=rad)
        body = _layer(W, H - ss)
        hb = H - ss
        top, bot = _lerp_i(bg, WHITE, 28, 100), _lerp_i(bg, BLACK, 10, 100)
        for y in range(hb):
            body.fill(_lerp_i(top, bot, y, hb - 1), (0, y, W, 1))
        mk = _layer(W, hb)
        mk.fill((255, 255, 255, 0))
        pygame.draw.rect(mk, (255, 255, 255, 255), (0, 0, W, hb), border_radius=rad)
        body.blit(mk, (0, 0), special_flags=pygame.BLEND_RGBA_MIN)
        big.blit(body, (0, 0))
        pygame.draw.line(big, _lerp_i(bg, WHITE, 52, 100), (rad * 0.8, ss * 1.5), (W - rad * 0.8, ss * 1.5), ss)
        if m100 >= 1000:
            pygame.draw.rect(big, _lerp_i(bg, WHITE, 60, 100), (0, 0, W, H - ss), max(1, ss // 2),
                             border_radius=rad)
        img = _smooth(big, w, h)
        fnt = _fit_font(label, w - max(4, h // 2), font_size)
        txt = fnt.render(label, True, fg)
        img.blit(txt, txt.get_rect(center=(w // 2, (h - 1) // 2)))
        return img
    return _cached(("pill", w, h, int(m100), label, int(font_size)), build, _small)


# ============================================================= Piktogramme
def _note(s, col, hx, hy, n, stem_top, lw):
    """Notenkopf (schräge Ellipse) mit Hals bis stem_top; liefert x des Halses."""
    pygame.draw.polygon(s, col, _ellipse_pts(hx, hy, n * 0.145, n * 0.105, -0.42, 20))
    sx = hx + n * 0.118
    pygame.draw.line(s, col, (sx, hy - n * 0.02), (sx, stem_top), max(1, int(lw * 0.8)))
    return sx


def _mini_balloon(cx, top, R):
    """Umriss eines kleinen Ballons (Pol oben bei top) + y und halbe Breite des Munds."""
    yeq = top + R
    L = R * 1.15
    m = R * 0.26
    right = []
    for i in range(9):
        a = math.pi / 2 * i / 8
        right.append((cx + R * math.sin(a), yeq - R * math.cos(a)))
    for i in range(1, 9):
        u = i / 8
        right.append((cx + m + (R - m) * math.cos(u * math.pi / 2) ** 1.2, yeq + L * u))
    return right + [(2 * cx - x, y) for x, y in reversed(right)], yeq + L, m


def _icon_balloon(s, col, cx, top, R, n, lw, ss):
    """Kleiner Ballon mit ausgespartem Glanz, Seilen und Korb (für Piktogramme)."""
    sil, ym, m = _mini_balloon(cx, top, R)
    pygame.draw.polygon(s, col, sil)
    pygame.draw.polygon(s, (0, 0, 0, 0), _arc_band(cx, top + R, R * 0.76, R * 0.22,
                                                   math.pi * 1.10, math.pi * 1.40, 10))
    bw = R * 0.70
    by = ym + R * 0.34
    rl = max(ss, lw // 2)
    pygame.draw.line(s, col, (cx - m, ym), (cx - bw / 2, by), rl)
    pygame.draw.line(s, col, (cx + m, ym), (cx + bw / 2, by), rl)
    pygame.draw.rect(s, col, (int(cx - bw / 2), int(by), int(bw), int(R * 0.46)), border_radius=max(1, ss * 2))
    return sil, by + R * 0.46


def icon(name, size, col):
    """Piktogramm size x size (4-fach überabgetastet): auto, stats, fair, turbo,
    music, mute, close, left, right, check, cross, half, ff, play, balloon, two."""
    size = max(4, int(size))
    col = _rgb(col)

    def build():
        ss = 4
        n = size * ss
        s = _layer(n, n)
        lw = max(ss, n // 9)
        if name == "auto":
            # Kreispfeil mit Spitze + Play-Dreieck in der Mitte
            c, r = (n // 2, n // 2), n * 2 // 5
            pygame.draw.arc(s, col, (c[0] - r, c[1] - r, 2 * r, 2 * r), 0.55, 5.75, lw)
            tip = (c[0] + r * math.cos(0.55), c[1] - r * math.sin(0.55))
            pygame.draw.polygon(s, col, [(tip[0] - n * 0.16, tip[1] - n * 0.02),
                                         (tip[0] + n * 0.10, tip[1] - n * 0.06),
                                         (tip[0] - n * 0.02, tip[1] + n * 0.18)])
            pygame.draw.polygon(s, col, [(n * 0.42, n * 0.34), (n * 0.66, n * 0.5), (n * 0.42, n * 0.66)])
        elif name == "stats":
            bw = n // 5
            for i, hh in enumerate((0.38, 0.62, 0.86)):
                x = n * 0.14 + i * (bw + n * 0.09)
                pygame.draw.rect(s, col, (int(x), int(n * (0.92 - hh)), bw, int(n * hh)),
                                 border_radius=max(1, ss))
        elif name == "fair":
            pts = [(n * 0.5, n * 0.06), (n * 0.88, n * 0.2), (n * 0.82, n * 0.6), (n * 0.5, n * 0.95),
                   (n * 0.18, n * 0.6), (n * 0.12, n * 0.2)]
            pygame.draw.polygon(s, col, pts, lw)
            pygame.draw.lines(s, col, False, [(n * 0.32, n * 0.5), (n * 0.46, n * 0.64),
                                              (n * 0.7, n * 0.36)], lw)
        elif name == "turbo":
            pygame.draw.polygon(s, col, [(n * 0.58, n * 0.04), (n * 0.2, n * 0.56), (n * 0.48, n * 0.56),
                                         (n * 0.38, n * 0.96), (n * 0.82, n * 0.4), (n * 0.53, n * 0.4)])
        elif name == "music":
            # zwei Achtelnoten mit Balken (♫)
            x1 = _note(s, col, n * 0.30, n * 0.75, n, n * 0.22, lw)
            x2 = _note(s, col, n * 0.71, n * 0.66, n, n * 0.13, lw)
            hl = max(ss, int(lw * 0.8)) / 2
            bt = n * 0.13
            pygame.draw.polygon(s, col, [(x1 - hl, n * 0.22), (x2 + hl, n * 0.13),
                                         (x2 + hl, n * 0.13 + bt), (x1 - hl, n * 0.22 + bt)])
        elif name == "mute":
            # Achtelnote (♪), schräg durchgestrichen
            x1 = _note(s, col, n * 0.40, n * 0.72, n, n * 0.15, lw)
            _stroke(s, col, [(x1, n * 0.15), (x1 + n * 0.13, n * 0.25), (x1 + n * 0.19, n * 0.38),
                             (x1 + n * 0.14, n * 0.50)], max(ss, int(lw * 0.8)))
            a, b = (n * 0.15, n * 0.13), (n * 0.86, n * 0.87)
            _stroke(s, (0, 0, 0, 0), [a, b], int(lw * 2.4))
            _stroke(s, col, [a, b], lw)
        elif name == "close":
            pygame.draw.line(s, col, (n * 0.22, n * 0.22), (n * 0.78, n * 0.78), lw)
            pygame.draw.line(s, col, (n * 0.78, n * 0.22), (n * 0.22, n * 0.78), lw)
        elif name in ("left", "right"):
            pts = [(n * 0.66, n * 0.18), (n * 0.30, n * 0.5), (n * 0.66, n * 0.82)]
            if name == "right":
                pts = [(n - x, y) for x, y in pts]
            pygame.draw.polygon(s, col, pts)
        elif name == "check":
            pygame.draw.lines(s, col, False, [(n * 0.16, n * 0.52), (n * 0.4, n * 0.76),
                                              (n * 0.84, n * 0.26)], max(ss, n // 7))
        elif name == "cross":
            w2 = max(ss, n // 7)
            pygame.draw.line(s, col, (n * 0.2, n * 0.2), (n * 0.8, n * 0.8), w2)
            pygame.draw.line(s, col, (n * 0.8, n * 0.2), (n * 0.2, n * 0.8), w2)
        elif name == "half":
            # "½" aus Strichen: 1, Bruchstrich, 2
            lw2 = max(ss, int(lw * 0.9))
            _stroke(s, col, [(n * 0.16, n * 0.22), (n * 0.30, n * 0.11), (n * 0.30, n * 0.50)], lw2)
            _stroke(s, col, [(n * 0.74, n * 0.10), (n * 0.27, n * 0.90)], lw2)
            c2x, c2y, r2 = n * 0.69, n * 0.63, n * 0.13
            pts = [(c2x + r2 * math.cos(a), c2y + r2 * math.sin(a))
                   for a in [math.pi * (1.06 + 1.12 * i / 10) for i in range(11)]]
            pts += [(n * 0.55, n * 0.90), (n * 0.85, n * 0.90)]
            _stroke(s, col, pts, lw2)
        elif name == "ff":
            pygame.draw.polygon(s, col, [(n * 0.10, n * 0.20), (n * 0.50, n * 0.50), (n * 0.10, n * 0.80)])
            pygame.draw.polygon(s, col, [(n * 0.48, n * 0.20), (n * 0.88, n * 0.50), (n * 0.48, n * 0.80)])
        elif name == "play":
            pygame.draw.polygon(s, col, [(n * 0.26, n * 0.14), (n * 0.84, n * 0.50), (n * 0.26, n * 0.86)])
        elif name == "balloon":
            _icon_balloon(s, col, n * 0.5, n * 0.05, n * 0.31, n, lw, ss)
        elif name == "two":
            # zwei Ballone: hinten links kleiner, vorn rechts größer (mit Lücke)
            _icon_balloon(s, col, n * 0.33, n * 0.04, n * 0.21, n, lw, ss)
            sil = _mini_balloon(n * 0.66, n * 0.16, n * 0.245)[0]
            pygame.draw.polygon(s, (0, 0, 0, 0), sil)
            pygame.draw.lines(s, (0, 0, 0, 0), True, sil, max(ss, lw))
            _icon_balloon(s, col, n * 0.66, n * 0.16, n * 0.245, n, lw, ss)
        return _smooth(s, size, size)
    return _cached(("icon", name, size, col), build, _small)


# ============================================================= Kurve
def _label(text, size):
    """Beschriftung mit weichem Schatten (gecacht)."""
    def build():
        f = ui.font(size, bold=True)
        fg = f.render(text, True, (238, 240, 252))
        sh = f.render(text, True, (0, 0, 0))
        sh.set_alpha(150)
        out = _layer(fg.get_width() + 1, fg.get_height() + 1)
        out.blit(sh, (1, 1))
        out.blit(fg, (0, 0))
        return out
    return _cached(("label", text, int(size)), build, _small)


def _grid_strip(length, vertical):
    """Hilfslinie (1 px, gestrichelt, halbdurchsichtig) als gecachter Streifen."""
    def build():
        s = _layer(1, length) if vertical else _layer(length, 1)
        for i in range(0, length, 8):
            if vertical:
                s.fill((255, 255, 255, 44), (0, i, 1, 5))
            else:
                s.fill((255, 255, 255, 44), (i, 0, 5, 1))
        return s
    return _cached(("gstrip", int(length), bool(vertical)), build, _small)


def draw_grid(s, rect, ylines, xlines, font_size):
    """Hilfslinien: waagerecht bei ylines (Text links über der Linie), senkrecht
    bei xlines (Text unten mittig). Nur gecachte Streifen und Texte."""
    rect = pygame.Rect(rect)
    if rect.w <= 0 or rect.h <= 0:
        return
    pad = max(2, int(font_size) // 3)
    if ylines:
        hl = _grid_strip(rect.w, False)
        for y, label in ylines:
            y = int(y)
            s.blit(hl, (rect.x, y))
            if label:
                img = _label(label, font_size)
                s.blit(img, (rect.x + pad, y - img.get_height()))
    for x, label in xlines:
        x = int(x)
        img = _label(label, font_size) if label else None
        bottom = rect.bottom - (img.get_height() + pad if img else 0)
        if bottom > rect.y:
            s.blit(_grid_strip(rect.h, True), (x, rect.y), (0, 0, 1, bottom - rect.y))
        if img:
            s.blit(img, (x - img.get_width() // 2, rect.bottom - img.get_height() - 1))


def _ribbon(p, hw):
    """Band der halben Breite hw um die Punktfolge p (glatte dicke Linie)."""
    left, right = [], []
    n = len(p)
    for i in range(n):
        xa, ya = p[max(0, i - 1)]
        xb, yb = p[min(n - 1, i + 1)]
        dx, dy = xb - xa, yb - ya
        d = math.hypot(dx, dy) or 1.0
        nx, ny = -dy / d * hw, dx / d * hw
        x, y = p[i]
        left.append((x + nx, y + ny))
        right.append((x - nx, y - ny))
    return left + right[::-1]


def _work(w, h):
    """Wiederverwendete Arbeitsfläche für die Kurvenfläche (wächst nur)."""
    cur = _cache.get(("work",))
    if cur is None or cur.get_width() < w or cur.get_height() < h:
        ow, oh = (cur.get_width(), cur.get_height()) if cur is not None else (0, 0)
        cur = _layer((max(w, ow) + 63) // 64 * 64, (max(h, oh) + 63) // 64 * 64)
        _cache[("work",)] = cur
    return cur


def draw_curve(s, pts, col, base_y, width):
    """Multiplikator-Kurve: Verlaufsfläche bis base_y (oben dichter), breites
    Leuchten, kräftige Linie mit Glanz und hellem Kopf."""
    if len(pts) < 2:
        return
    col = _rgb(col)
    width = max(1, int(width))
    by = _ri(base_y)
    step = max(1, (len(pts) + 239) // 240)          # höchstens ~240 Stützpunkte
    sel = list(pts[::step])
    if (len(pts) - 1) % step:
        sel.append(pts[-1])
    p = [(_ri(x), _ri(y)) for x, y in sel]
    sw, sh = s.get_size()
    g = width * 2 + 3
    xs = [q[0] for q in p]
    ys = [q[1] for q in p]
    ymin = min(ys)
    left, right = max(0, min(xs) - g), min(sw, max(xs) + g + 1)
    top, bottom = max(0, min(ymin, by) - g), min(sh, max(max(ys), by) + 1)
    light = _lerp_i(col, WHITE, 40, 100)
    if right > left and bottom > top:
        bw, bh = right - left, bottom - top
        work = _work(bw, bh)
        work.fill((0, 0, 0, 0), (0, 0, bw, bh))
        loc = [(x - left, y - top) for x, y in p]
        if by > ymin:
            pygame.draw.polygon(work, (col[0], col[1], col[2], 255),
                                loc + [(loc[-1][0], by - top), (loc[0][0], by - top)])
            # Verlauf: Alpha bandweise multiplizieren (oben ~70, unten 0)
            y0 = max(0, ymin - top)
            span = max(1, by - top - y0)
            bands = 24
            for k in range(bands):
                ya = y0 + span * k // bands
                yb = y0 + span * (k + 1) // bands if k < bands - 1 else bh
                a = 70 * (bands - k) * (bands - k) // (bands * bands)
                work.fill((255, 255, 255, a), (0, ya, bw, yb - ya), special_flags=pygame.BLEND_RGBA_MULT)
        st = max(1, (len(loc) + 95) // 96)              # Leuchten: höchstens ~96 Punkte
        gl = loc[::st]
        if (len(loc) - 1) % st:
            gl.append(loc[-1])
        for hw, a in ((width * 2.0, 70), (width * 1.0 + 1, 120)):
            c = (light[0], light[1], light[2], a)
            pygame.draw.polygon(work, c, _ribbon(gl, hw))
            pygame.draw.circle(work, c, gl[0], hw)
            pygame.draw.circle(work, c, gl[-1], hw)
        s.blit(work, (left, top), (0, 0, bw, bh))
    pygame.draw.lines(s, col, False, p, width)
    if width >= 3:
        r = width / 2.0
        for q in p:
            pygame.draw.circle(s, col, q, r)
    if width >= 3:
        pygame.draw.lines(s, light, False, [(x, y - width // 4) for x, y in p], max(1, width // 3))
    end = p[-1]
    pygame.draw.circle(s, col, end, width * 0.95 + 1)
    pygame.draw.circle(s, _lerp_i(col, WHITE, 75, 100), end, max(1.0, width * 0.5))


def glow(r, col):
    """Lichthof mit Radius r auf schwarzem Grund - zum additiven Blitten
    (``special_flags=pygame.BLEND_RGB_ADD``)."""
    r = max(1, int(r))
    col = _rgb(col)

    def build():
        s = pygame.Surface((2 * r + 2, 2 * r + 2))
        s.fill((0, 0, 0))
        steps = 12
        for i in range(steps):
            f = (i + 1) / steps
            a = f * f
            pygame.draw.circle(s, (int(col[0] * a), int(col[1] * a), int(col[2] * a)), (r + 1, r + 1),
                               max(1, int(r * (1.0 - i / steps))))
        return s
    return _cached(("glow", r, col), build)


def curve_preview(w, h, crash100, marks):
    """Mini-Kurve w x h fürs Fairness-Fenster: von ×1 bis zum Platzpunkt
    (Knall-Stern bzw. goldener Stern bei der Mondlandung), grüne Punkte bei
    den Ausstiegen marks (m100), dezente Achsen."""
    from . import lamacrash_logic as L
    w, h = max(16, int(w)), max(12, int(h))
    crash100 = max(100, min(L.MAX_MULT100, int(crash100)))
    marks = tuple(int(v) for v in marks)

    def build():
        ss = 2
        W, H = w * ss, h * ss
        big = _layer(W, H)
        pad = max(ss * 2, min(W, H) // 12)
        x0, y0, x1, y1 = pad, pad + ss, W - pad * 2, H - pad
        moon = crash100 >= L.MAX_MULT100
        T = max(1, L.pop_tick(crash100))
        xs = max(T, L.TPS * 4)
        top = max(crash100, 150)

        def px(t):
            return x0 + (x1 - x0) * t // xs

        def py(m):
            return y1 - (y1 - y0) * (min(m, crash100) - 100) // (top - 100)
        ax = (255, 255, 255, 70)
        pygame.draw.line(big, ax, (x0, y0 - pad // 2), (x0, y1), ss)
        pygame.draw.line(big, ax, (x0, y1), (W - pad // 2, y1), ss)
        yc = py(crash100)
        for x in range(x0 + ss * 2, x1, ss * 6):
            pygame.draw.line(big, (255, 255, 255, 46), (x, yc), (min(x1, x + ss * 3), yc), max(1, ss // 2))
        k = max(8, W // (3 * ss))
        pts = [(px(T * i // k), py(L.mult_at(T * i // k))) for i in range(k + 1)]
        line = GOLD if moon else (255, 96, 112)
        pygame.draw.polygon(big, (line[0], line[1], line[2], 56), pts + [(pts[-1][0], y1), (pts[0][0], y1)])
        pygame.draw.lines(big, line, False, pts, max(ss, H // 40))
        rm = max(ss * 2, H // 22)
        for mv in marks:
            if L.MIN_CASH <= mv <= crash100:
                c = (px(L.tick_at(mv)), py(mv))
                pygame.draw.circle(big, (12, 52, 30), c, rm + ss)
                pygame.draw.circle(big, (64, 214, 122), c, rm)
                pygame.draw.circle(big, (214, 255, 230), (c[0] - rm // 3, c[1] - rm // 3), max(1, rm // 3))
        ex, ey = pts[-1]
        rs = max(ss * 4, min(W, H) // 9)
        if moon:
            pygame.draw.polygon(big, GOLD_D, _burst(ex, ey, rs * 1.1, rs * 0.5, 5, -math.pi / 2, 5, 0.0))
            pygame.draw.polygon(big, GOLD, _burst(ex, ey, rs * 0.9, rs * 0.4, 5, -math.pi / 2, 5, 0.0))
        else:
            pygame.draw.polygon(big, (120, 28, 18), _burst(ex, ey, rs * 1.08, rs * 0.56, 9, 0.2, 3))
            pygame.draw.polygon(big, (255, 112, 36), _burst(ex, ey, rs, rs * 0.52, 9, 0.2, 3))
            pygame.draw.polygon(big, (255, 226, 90), _burst(ex, ey, rs * 0.62, rs * 0.36, 9, 0.35, 9))
            pygame.draw.circle(big, (255, 252, 230), (ex, ey), rs * 0.26)
        return _smooth(big, w, h)
    return _cached(("preview", w, h, crash100, marks), build, _small)


# ============================================================= Mondlandung
def flag(n, col):
    """Fahne n x n: Stange mit Querstab (auf dem Mond weht kein Wind), Tuch in
    col mit Falten und weißem Lama im Abzeichen, Staubhügel am Fuß (unten)."""
    n = max(8, int(n))
    col = _rgb(col)

    def build():
        ss = 3
        N = n * ss
        big = _layer(N, N)
        px = N * 0.20
        pw = max(ss, N // 22)
        foot = N * 0.94
        pygame.draw.ellipse(big, (92, 92, 104), (px - N * 0.18, foot - N * 0.05, N * 0.36, N * 0.10))
        pygame.draw.ellipse(big, (150, 150, 162), (px - N * 0.15, foot - N * 0.065, N * 0.28, N * 0.07))
        pygame.draw.rect(big, (112, 116, 130), (int(px - pw / 2), int(N * 0.06), pw, int(foot - N * 0.06)))
        pygame.draw.rect(big, (232, 234, 242), (int(px - pw / 2), int(N * 0.06), max(1, pw // 3),
                                                int(foot - N * 0.06)))
        fx0, fx1 = px + pw / 2, N * 0.93
        fy0 = N * 0.10
        fh = N * 0.40
        top_e, bot_e = [], []
        for i in range(17):
            f = i / 16
            x = fx0 + (fx1 - fx0) * f
            wave = math.sin(f * math.pi * 3.0) * N * 0.02 * f
            top_e.append((x, fy0 + ss + wave * 0.25))
            bot_e.append((x, fy0 + fh + wave - N * 0.03 * f))
        cloth = top_e + bot_e[::-1]
        lay = _layer(N, N)
        pygame.draw.polygon(lay, col, cloth)
        folds = _layer(N, N)
        for x in range(int(fx0), int(fx1) + 1, ss):
            f = (x - fx0) / max(1.0, fx1 - fx0)
            a = int(46 * math.cos(f * math.pi * 3.0) * min(1.0, f * 4.0))
            folds.fill((255, 255, 255, a) if a > 0 else (0, 0, 0, -a), (x, 0, ss, N))
        lay.blit(folds, (0, 0))
        # Abzeichen: dunklerer Kreis mit weißem Lama
        bc = (fx0 + (fx1 - fx0) * 0.5, fy0 + fh * 0.50)
        br = fh * 0.34
        pygame.draw.circle(lay, _lerp_i(col, BLACK, 30, 100), bc, br)
        e = max(8, int(br * 1.6))
        lam = _layer(e, e)
        LCD._draw_llama(lam, e, "stand", WHITE)
        lam.fill((255, 255, 255, 0), special_flags=pygame.BLEND_RGBA_MAX)
        lay.blit(lam, (int(bc[0] - e * 0.55), int(bc[1] - e * 0.52)))
        lay.blit(_mask(N, N, cloth), (0, 0), special_flags=pygame.BLEND_RGBA_MIN)
        big.blit(lay, (0, 0))
        pygame.draw.lines(big, _lerp_i(col, BLACK, 45, 100), False, bot_e, max(1, ss * 2 // 3))
        pygame.draw.line(big, (206, 208, 220), (px, fy0), (fx1 + ss, fy0), max(ss, pw * 2 // 3))
        pygame.draw.circle(big, (244, 244, 250), (px, N * 0.06), pw * 0.9)
        return _smooth(big, n, n)
    return _cached(("flag", n, col), build)


# ============================================================= Zusammengesetztes
def draw_rig(s, cx, rim_y, base, level, stripe, riders, flame):
    """Ganzer Ballon über dem Korbrand (cx, rim_y): Hülle + Seile, Flamme,
    Rückwand, Lamas, Vorderwand. Gibt das Rechteck der Hülle zurück."""
    cx, rim_y, base = int(cx), int(rim_y), int(base)
    img = balloon(base, level, stripe)
    w, h = img.get_size()
    x0, y0 = cx - w // 2, rim_y + 1 - h
    s.blit(img, (x0, y0))
    mouth, _sk, _nh, mouth_hw, frame, burner_h, _bhw, flame_w = _rig(base)
    if flame is not None and flame >= 0:
        g = glow(max(3, mouth_hw * 5 // 4), (84, 46, 12) if flame % 2 == 0 else (70, 38, 10))
        s.blit(g, (cx - g.get_width() // 2, rim_y - mouth - g.get_height() // 2),
               special_flags=pygame.BLEND_RGB_ADD)
        fl = burner_flame(flame_w, flame)
        s.blit(fl, (cx - fl.get_width() // 2, rim_y - frame - burner_h + 1 - fl.get_height()))
    bw = basket_w(base)
    bx = cx - bw // 2
    s.blit(basket(bw, False), (bx, rim_y))
    n = llama_size(base)
    foot = _rider_foot(rim_y, base)
    single = len(riders) == 1
    for i, rider in enumerate(riders[:2]):
        if rider is None:
            continue
        scarf, pose = rider
        dx = 0 if single else (-bw // 4 if i == 0 else bw // 4)
        spr = LCD.llama_sprite(n, pose, _rgb(scarf))
        s.blit(spr, (cx + dx - n // 2, foot - n))
    s.blit(basket(bw, True), (bx, rim_y))
    return pygame.Rect(x0, y0, w, h - mouth)


def _open_stage(open_f):
    """Öffnungsstufe 0..8 des Schirms."""
    return max(0, min(8, int(float(open_f) * 8 + 1e-6)))


def _jumper_img(n, scarf, chute, k):
    """Lama mit Gurt am Schirm (Stufe k) - Fläche und Lage der Sprite-Ecke."""
    cw = max(6, n * 3 // 2)
    W = max(cw, n) + 2
    hx = W // 2
    lx = hx - n * 49 // 100
    ly = cw - 1 - n * 40 // 100

    def build():
        out = _layer(W, ly + n + 1)
        can = _chute_stage(cw, chute, k)
        out.blit(can, (hx - cw // 2, 0))
        spr = LCD.llama_sprite(n, "hop" if k < 4 else "stand", scarf)
        out.blit(spr, (lx, ly))
        strap = max(1, n // 16)
        pygame.draw.line(out, (54, 48, 60), (hx, ly + n * 40 // 100), (hx - n * 3 // 100, ly + n * 68 // 100), strap)
        pygame.draw.circle(out, METAL_L, (hx, ly + n * 41 // 100), max(1, strap))
        return out
    return _cached(("jumper", n, scarf, chute, k), build), lx, ly


def draw_jumper(s, cx, cy, n, scarf, chute, open_f):
    """Lama am Fallschirm; (cx, cy) = Mitte des Lama-Sprites, open_f 0..1."""
    n = max(4, int(n))
    img, lx, ly = _jumper_img(n, _rgb(scarf), _rgb(chute), _open_stage(open_f))
    s.blit(img, (int(cx) - lx - n // 2, int(cy) - ly - n // 2))


def draw_sack(s, cx, cy, n, chute, open_f):
    """Geldsack am Mini-Schirm; (cx, cy) = Mitte des Sacks, open_f 0..1."""
    n = max(6, int(n))
    chute = _rgb(chute)
    k = _open_stage(open_f)
    cw = max(6, n * 13 // 10)
    W = max(cw, n) + 2
    hx = W // 2
    hook = n * 10 // 100
    sx, sy = hx - n // 2, cw - 1 - hook

    def build():
        out = _layer(W, sy + n + 1)
        out.blit(_chute_stage(cw, chute, k), (hx - cw // 2, 0))
        out.blit(money_sack(n, 1), (sx, sy))
        return out
    img = _cached(("sackchute", n, chute, k), build)
    s.blit(img, (int(cx) - sx - n // 2, int(cy) - sy - n // 2))


def draw_faller(s, cx, cy, n, scarf, step):
    """Purzelndes Lama (Pose "fall"), in 8 Stufen zu 45° gedreht, Mitte (cx, cy)."""
    n = max(4, int(n))
    step = int(step) % 8
    scarf = _rgb(scarf)

    def build():
        big = _layer(n * 2, n * 2)
        LCD._draw_llama(big, n * 2, "fall", scarf)
        if step:
            big = pygame.transform.rotate(big, -45 * step)
        return _smooth(big, big.get_width() // 2, big.get_height() // 2)
    img = _cached(("faller", n, scarf, step), build)
    s.blit(img, (int(cx) - img.get_width() // 2, int(cy) - img.get_height() // 2))
