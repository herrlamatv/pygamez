# -*- coding: utf-8 -*-
"""
ui.py
=====
Gemeinsames UI-Toolkit für alle Pygame-Screens (Menüs, Overlays, Spiele).

Seit dem UI-Update gibt es ZWEI wählbare Designs ("Themes"):

- "modern"  (Standard): ruhiges, aufgeräumtes Graphit-Design mit einem
  einzelnen Indigo-Akzent. Flache Panels mit Haarlinien-Rand, dezente
  Hover-Übergänge, keine Deko-Effekte - wirkt clean und professionell.
- "classic": der bisherige Look - dunkler Farbverlauf mit "Aurora"-Lichtern,
  Parallax-Sternenfeld inkl. Sternschnuppen, Glow-Buttons, Verlaufstitel
  mit Leuchten, Funken-Partikel und Scanlinien-Übergang.

Umgeschaltet wird über set_theme("modern"/"classic") - im Spiel über den
Reiter "Erscheinungsbild" im Options-Screen. Da ALLE Module die Farben nur
über ui.<NAME> (dynamisch) lesen, wirkt der Wechsel sofort überall.

Alles rein in Software gerendert (SDL-dummy), deshalb werden teure
Flächen (Verläufe, Glows, Vignette, Text-Verläufe) gecacht; set_theme()
leert die Caches.

Verwendung (in draw()):
    import ui
    ui.draw_background(surface, w, h)          # Hintergrund im aktiven Theme
    ui.draw_title(surface, w, "TITEL", sub)    # Kopfzeile mit Akzentlinie
    ui.draw_button(surface, rect, "Start", font, selected=True)
    ui.draw_panel(surface, rect)               # Karten-Panel

Effekte (einmal auslösen, main.py zeichnet sie pro Frame über draw_fx):
    ui.spawn_burst(x, y, color)                # Funken (nur im Classic-Theme)
    ui.spawn_confetti(w, h)                    # Konfetti-Regen (Rekord!)
    ui.begin_transition()                      # weicher Screen-Übergang
"""

import math
import random

import pygame

# ---------------------------------------------------------------------------
#  Themes: Farbpaletten + Effekt-Schalter
# ---------------------------------------------------------------------------
#  Jedes Theme besteht aus einer Farbpalette (wird per set_theme() in die
#  Modul-Globals BG_TOP, PANEL, ACCENT, ... geschrieben) und einem fx-Dict
#  mit Schaltern/Werten für die Deko-Effekte.

_CLASSIC_COLORS = dict(
    BG_TOP=(14, 17, 29),        # Hintergrund-Verlauf oben
    BG_BOTTOM=(24, 28, 44),     # Hintergrund-Verlauf unten
    PANEL=(30, 35, 52),         # Panel-Fläche
    PANEL_LIGHT=(38, 44, 64),   # hellere Panel-Variante / Hover
    BORDER=(52, 60, 86),        # Panel-/Button-Rand
    BORDER_LIGHT=(74, 84, 116),
    BTN=(40, 46, 66),           # Button normal
    BTN_SEL=(56, 88, 152),      # Button ausgewählt
    ACCENT=(88, 156, 255),      # Primär-Akzent (kühles Blau)
    ACCENT2=(155, 110, 255),    # Zweit-Akzent (Violett, für Verläufe)
    ACCENT_SOFT=(66, 110, 180),
    GREEN=(110, 205, 140),      # Erfolg / "AN"
    GOLD=(245, 205, 100),       # Highscore / Tastenwerte
    RED=(225, 95, 95),          # Gefahr / "AUS"
    TEXT=(235, 238, 245),       # Haupttext
    TEXT_DIM=(150, 158, 178),   # Nebentext
    TEXT_FAINT=(95, 102, 124),  # Fußnoten
)

_MODERN_COLORS = dict(
    BG_TOP=(17, 19, 24),        # neutrales Graphit, kaum Verlauf
    BG_BOTTOM=(22, 25, 32),
    PANEL=(28, 31, 40),
    PANEL_LIGHT=(37, 41, 52),
    BORDER=(45, 50, 62),
    BORDER_LIGHT=(64, 70, 86),
    BTN=(33, 37, 47),
    BTN_SEL=(47, 56, 76),
    ACCENT=(91, 141, 239),      # Indigo-Blau, einziger Schmuck-Akzent
    ACCENT2=(129, 155, 255),
    ACCENT_SOFT=(64, 94, 156),
    GREEN=(88, 190, 132),
    GOLD=(229, 196, 106),
    RED=(224, 108, 108),
    TEXT=(233, 235, 241),
    TEXT_DIM=(150, 157, 172),
    TEXT_FAINT=(100, 106, 122),
)

_CLASSIC_FX = dict(
    stars=True, aurora=True, shooting=True,   # Sternenfeld/Aurora/Schnuppen
    vignette=70,                              # Vignetten-Stärke (Alpha)
    title_glow=True, title_grad=True,         # Titel: Glow + Verlaufstext
    btn_glow=True, btn_arrow=True,            # Buttons: Außen-Glow + Pfeil
    sparks=True,                              # Funken-Partikel bei Klicks
    scanline=True, trans_dur=0.35,            # Übergang: Fade + Scanlinie
    panel_radius=12, btn_radius=10,           # Eckenrundungen
    shadow_alpha=90,                          # Panel-Schlagschatten
)

_MODERN_FX = dict(
    stars=False, aurora=False, shooting=False,
    vignette=42,
    title_glow=False, title_grad=False,
    btn_glow=False, btn_arrow=False,
    sparks=False,
    scanline=False, trans_dur=0.22,
    panel_radius=10, btn_radius=8,
    shadow_alpha=55,
)

# Passende Farbwerte für die Tkinter-Seite (Sidebar) als Hex-Strings.
_TK_CLASSIC = dict(
    SIDEBAR="#12151f", HEADER="#0c0f18", CARD="#1a2030",
    BTN="#1f2636", BTN_HOVER="#2c3650",
    ACCENT="#589cff", ACCENT2="#9b6eff",
    DANGER="#8e3540", DANGER_HOVER="#ab414e",
    BACK="#4b6b5a", BACK_HOVER="#5f8a73",
    TEXT="#e9edf5", TEXT_DIM="#98a2b8", TEXT_FAINT="#5f6680",
    BORDER="#2a3147", GREEN="#6ecd8c", GOLD="#f5cd64", RED="#e15f5f",
)

_TK_MODERN = dict(
    SIDEBAR="#14161c", HEADER="#101217", CARD="#1b1e27",
    BTN="#20242e", BTN_HOVER="#2a2f3b",
    ACCENT="#5b8def", ACCENT2="#819bff",
    DANGER="#7c3540", DANGER_HOVER="#934250",
    BACK="#3c5a4b", BACK_HOVER="#4b7060",
    TEXT="#e9ebf1", TEXT_DIM="#969dac", TEXT_FAINT="#646a7a",
    BORDER="#2d323e", GREEN="#58be84", GOLD="#e5c46a", RED="#e06c6c",
)

THEMES = {
    "modern": (_MODERN_COLORS, _MODERN_FX, _TK_MODERN),
    "classic": (_CLASSIC_COLORS, _CLASSIC_FX, _TK_CLASSIC),
}
THEME_NAMES = ("modern", "classic")
DEFAULT_THEME = "modern"

_theme = DEFAULT_THEME
_fx = _MODERN_FX
_tk = _TK_MODERN

# Die Palette des aktiven Themes liegt in den Modul-Globals (BG_TOP, ACCENT,
# ...), damit ALLE bestehenden ui.<NAME>-Zugriffe unverändert funktionieren.
globals().update(_MODERN_COLORS)


def set_theme(name):
    """Aktiviert ein Theme ("modern"/"classic") und leert alle Render-Caches."""
    global _theme, _fx, _tk
    if name not in THEMES:
        name = DEFAULT_THEME
    colors, fx, tkcols = THEMES[name]
    _theme = name
    _fx = fx
    _tk = tkcols
    globals().update(colors)
    # Gecachte Flächen basieren auf der alten Palette -> wegwerfen.
    _bg_cache.clear()
    _glow_cache.clear()
    _text_fx_cache.clear()
    _fade_cache.clear()
    _btn_anim.clear()


def theme_name():
    """Name des aktiven Themes ('modern' oder 'classic')."""
    return _theme


def is_modern():
    """True im aufgeräumten Standard-Design (ohne Deko-Effekte)."""
    return _theme == "modern"


def fx(key):
    """Effekt-Schalter/-Wert des aktiven Themes (z.B. fx('stars'))."""
    return _fx[key]


def tk_colors():
    """Hex-Farbpalette des aktiven Themes für die Tkinter-Sidebar."""
    return dict(_tk)


# Eigene Akzentfarbe je Spiel - genutzt von der Tkinter-Sidebar (hex) UND
# von den Pygame-Screens (über game_color() als RGB-Tupel).
GAME_COLORS = {
    "SnakeGame": "#6ecd8c", "PongGame": "#589cff", "TicTacToeGame": "#f0a05a",
    "BreakoutGame": "#e15f5f", "TetrisGame": "#b07fe8", "InvadersGame": "#5ad4d4",
    "Game2048": "#f5cd64", "AirHockeyGame": "#6fe0d0", "MinesweeperGame": "#f08fb0",
    "AsteroidsGame": "#b9c2d9", "PacmanGame": "#ffd83b",
    "FlappyGame": "#f5c518", "DoodleGame": "#78d25a",
    "SudokuGame": "#c77dba", "FroggerGame": "#4caf6d",
    "MemoryGame": "#8f7ef2", "SolitaireGame": "#2fa77c",
    "AimTrainerGame": "#e05ad4",
    "ConnectFourGame": "#ff8f2e", "TankDuelGame": "#a8b545",
    "BlackjackGame": "#c8384f", "TunnelRacerGame": "#35e2ff",
    "LabyrinthGame": "#b07a4a",
    "ReversiGame": "#3fbf8f", "KniffelGame": "#e8b04b", "WordleGame": "#6aaa64",
    "TRexRunnerGame": "#8ea3b0", "DameGame": "#d87842", "PokerGame": "#e8c45c",
    "ChessGame": "#c9a24b", "MuehleGame": "#7fae8f", "SimonGame": "#e05a7d",
    "BilliardGame": "#2f9e6a",
    "SlidingPuzzleGame": "#5ac0e0", "MastermindGame": "#c86ad8",
    "BubbleShooterGame": "#ff7aa8", "HangmanGame": "#d89a4a",
    "BlockJumpGame": "#8fd14f",
}

# Schriftname mit Fallback-Kette (SysFont probiert der Reihe nach durch).
FONT_UI = "bahnschrift,segoeui,consolas"
FONT_MONO = "consolas,menlo,monospace"

_font_cache = {}


def font(size, bold=False, mono=False):
    """Gecachte Schrift; UI-Schrift (Bahnschrift/Segoe) oder Monospace."""
    key = (size, bold, mono)
    f = _font_cache.get(key)
    if f is None:
        f = pygame.font.SysFont(FONT_MONO if mono else FONT_UI, size, bold=bold)
        _font_cache[key] = f
    return f


# ---------------------------------------------------------------------------
#  Farb-Helfer
# ---------------------------------------------------------------------------

def hex_to_rgb(value):
    """'#rrggbb' -> (r, g, b)."""
    v = value.lstrip("#")
    return tuple(int(v[i:i + 2], 16) for i in (0, 2, 4))


def game_color(cls_name, default=None):
    """Akzentfarbe eines Spiels als RGB-Tupel (Fallback: Primär-Akzent)."""
    h = GAME_COLORS.get(cls_name)
    return hex_to_rgb(h) if h else (default or ACCENT)


def mix(c1, c2, f):
    """Lineare Mischung zweier RGB-Farben, f in 0..1."""
    f = max(0.0, min(1.0, f))
    return (int(c1[0] + (c2[0] - c1[0]) * f),
            int(c1[1] + (c2[1] - c1[1]) * f),
            int(c1[2] + (c2[2] - c1[2]) * f))


# ---------------------------------------------------------------------------
#  Frame-Takt: draw_background() misst die Zeit seit dem letzten Frame,
#  damit Hover-/Auswahl-Animationen unabhängig von den FPS gleich schnell laufen.
# ---------------------------------------------------------------------------

_last_ms = [0]
_frame_dt = 1 / 60.0


def _tick():
    global _frame_dt
    now = pygame.time.get_ticks()
    if _last_ms[0]:
        _frame_dt = max(0.001, min(0.05, (now - _last_ms[0]) / 1000.0))
    _last_ms[0] = now


# ---------------------------------------------------------------------------
#  Hintergrund: Verlauf + Vignette (gecacht). Im Classic-Theme zusätzlich
#  Aurora-Lichter und Sternenfeld mit Parallax-Drift + Sternschnuppen;
#  das Modern-Theme bleibt bewusst ruhig und statisch.
# ---------------------------------------------------------------------------

_bg_cache = {}       # (w, h) -> Surface mit Verlauf + Vignette
_stars = []          # [x, y, tiefe, radius] in 0..1-Koordinaten
_STAR_COUNT = 90


def _gradient(w, h, top, bottom):
    """Vertikaler Farbverlauf als Surface (zeilenweise, einmalig erzeugt)."""
    surf = pygame.Surface((w, h))
    for y in range(h):
        f = y / max(1, h - 1)
        pygame.draw.line(surf, mix(top, bottom, f), (0, y), (w, y))
    return surf


def _base_background(w, h):
    """Verlauf + Vignette, pro Größe nur einmal berechnet."""
    key = (w, h)
    surf = _bg_cache.get(key)
    if surf is None:
        surf = _gradient(w, h, BG_TOP, BG_BOTTOM)
        # Weiche Vignette: klein zeichnen und hochskalieren -> sanfter Verlauf
        # statt harter Ellipsen-Kante (smoothscale interpoliert die Ränder).
        sw, sh = max(8, w // 8), max(8, h // 8)
        shade = pygame.Surface((sw, sh), pygame.SRCALPHA)
        shade.fill((0, 0, 0, _fx["vignette"]))
        pygame.draw.ellipse(shade, (0, 0, 0, 0),
                            (-sw // 3, -sh // 3, sw + 2 * sw // 3, sh + 2 * sh // 3))
        surf.blit(pygame.transform.smoothscale(shade, (w, h)), (0, 0))
        # Cache klein halten (Auto-Auflösung erzeugt viele Größen beim Resize).
        if len(_bg_cache) > 6:
            _bg_cache.clear()
        _bg_cache[key] = surf
    return surf


_glow_cache = {}     # (farbe, größe) -> weiche runde Licht-Fläche (additiv)


def _glow_surface(color, size):
    """Weicher runder Licht-Fleck auf Schwarz - für additives Blitten.

    Wird klein (96px) mit quadratischem Abfall gezeichnet und dann
    hochskaliert -> butterweicher Verlauf ohne Banding, fast gratis.
    """
    size = max(32, (int(size) // 32) * 32)   # Größen bündeln -> Cache klein
    key = (color, size)
    surf = _glow_cache.get(key)
    if surf is None:
        base = pygame.Surface((96, 96))
        for r in range(48, 0, -1):
            f = (1.0 - r / 48.0) ** 2
            pygame.draw.circle(base, (int(color[0] * f), int(color[1] * f),
                                      int(color[2] * f)), (48, 48), r)
        surf = pygame.transform.smoothscale(base, (size, size))
        if len(_glow_cache) > 10:
            _glow_cache.clear()
        _glow_cache[key] = surf
    return surf


# Aurora-Lichter: (Farbe(max. Helligkeit), Größe rel. zu max(w,h),
#                  Drift-Tempo, Phase, Grundposition x/y rel.)
_AURORA = (
    ((26, 48, 95), 1.10, 0.11, 0.0, 0.22, 0.28),
    ((52, 34, 96), 0.95, 0.14, 2.1, 0.80, 0.30),
    ((16, 58, 62), 0.85, 0.08, 4.2, 0.50, 0.88),
)


def _draw_aurora(surface, w, h, tsec):
    """Langsam driftende, additive Licht-Flecken hinter allem."""
    for color, size_f, spd, ph, fx_, fy in _AURORA:
        size = int(size_f * max(w, h))
        cx = int((fx_ + 0.07 * math.sin(tsec * spd + ph)) * w)
        cy = int((fy + 0.06 * math.cos(tsec * spd * 0.9 + ph)) * h)
        g = _glow_surface(color, size)
        surface.blit(g, g.get_rect(center=(cx, cy)),
                     special_flags=pygame.BLEND_ADD)


def _ensure_stars():
    if not _stars:
        rnd = random.Random(20240)   # fester Seed -> ruhiges, stabiles Bild
        for _ in range(_STAR_COUNT):
            _stars.append([rnd.random(), rnd.random(),
                           rnd.uniform(0.25, 1.0), rnd.choice((1, 1, 1, 2))])


_shoot = {"active": None, "next_ms": 4000}


def _draw_shooting_star(surface, w, h):
    """Alle paar Sekunden zieht eine Sternschnuppe mit Leuchtspur durch."""
    now = pygame.time.get_ticks()
    st = _shoot["active"]
    if st is None:
        if now >= _shoot["next_ms"]:
            ang = math.radians(random.uniform(18, 38))
            speed = random.uniform(0.8, 1.3) * max(w, 400)
            _shoot["active"] = dict(
                x=random.uniform(0.15, 0.85) * w, y=random.uniform(0.05, 0.30) * h,
                vx=math.cos(ang) * speed * random.choice((1, -1)),
                vy=math.sin(ang) * speed, t0=now, dur=random.randint(550, 800))
        return
    t = (now - st["t0"]) / st["dur"]
    if t >= 1.0:
        _shoot["active"] = None
        _shoot["next_ms"] = now + random.randint(4500, 10000)
        return
    sec = t * st["dur"] / 1000.0
    x, y = st["x"] + st["vx"] * sec, st["y"] + st["vy"] * sec
    fade = 1.0 - t
    for i in range(9):     # Leuchtspur: kleiner werdende additive Punkte
        f = fade * (1.0 - i / 9.0)
        px, py = x - st["vx"] * 0.0024 * i, y - st["vy"] * 0.0024 * i
        if 0 <= px < w - 2 and 0 <= py < h - 2:
            c = int(190 * f)
            surface.fill((c, c, min(255, int(c * 1.2))),
                         (int(px), int(py), 2, 2),
                         special_flags=pygame.BLEND_ADD)


def draw_background(surface, w, h, stars=True, aurora=None):
    """Zeichnet den Standard-Hintergrund des aktiven Themes.

    Modern : ruhiger Verlauf + dezente Vignette, keine Bewegung.
    Classic: zusätzlich Aurora-Lichter, Sternenfeld und Sternschnuppen.
    stars=False (z.B. Options-Screen) -> auch im Classic-Theme ruhig.
    """
    _tick()
    surface.blit(_base_background(w, h), (0, 0))
    if aurora is None:
        aurora = stars
    ticks = pygame.time.get_ticks() / 1000.0
    if aurora and _fx["aurora"]:
        _draw_aurora(surface, w, h, ticks)
    if not stars or not _fx["stars"]:
        return
    _ensure_stars()
    for x, y, depth, r in _stars:
        # Langsame Drift nach oben + leichtes Funkeln über Sinus.
        yy = (y - ticks * 0.008 * depth) % 1.0
        tw = 0.5 + 0.5 * math.sin(ticks * (0.8 + depth) + x * 40.0)
        c = int(40 + 70 * depth * tw)
        surface.fill((c, c + 6, c + 18),
                     (int(x * w), int(yy * h), r, r))
    if _fx["shooting"]:
        _draw_shooting_star(surface, w, h)


# ---------------------------------------------------------------------------
#  Panels & Buttons
# ---------------------------------------------------------------------------

def draw_panel(surface, rect, color=None, border=None, radius=None,
               shadow=True, accent_top=None):
    """Abgerundetes Panel mit Rand und weichem Schlagschatten.

    color/border/radius=None -> Werte des aktiven Themes.
    accent_top: optionale Farbe für eine dezente 2px-Lichtkante oben.
    """
    color = color if color is not None else PANEL
    border = border if border is not None else BORDER
    radius = radius if radius is not None else _fx["panel_radius"]
    r = pygame.Rect(rect)
    if shadow:
        sh = pygame.Surface((r.w + 12, r.h + 12), pygame.SRCALPHA)
        pygame.draw.rect(sh, (0, 0, 0, _fx["shadow_alpha"]), (4, 6, r.w, r.h),
                         border_radius=radius + 2)
        surface.blit(sh, (r.x - 2, r.y - 2))
    pygame.draw.rect(surface, color, r, border_radius=radius)
    pygame.draw.rect(surface, border, r, width=1, border_radius=radius)
    if accent_top:
        pygame.draw.rect(surface, accent_top,
                         (r.x + radius, r.y, r.w - 2 * radius, 2))
    return r


_btn_anim = {}       # (x, y, w, h) -> Auswahl-Fortschritt 0..1


def _btn_progress(key, selected):
    """Weicher Übergang des Auswahl-Zustands (statt hartem Umschalten)."""
    v = _btn_anim.get(key, 0.0)
    target = 1.0 if selected else 0.0
    v += (target - v) * min(1.0, _frame_dt * 12.0)
    if abs(v - target) < 0.01:
        v = target
    if len(_btn_anim) > 96:      # Screens wechseln -> alte Keys wegwerfen
        _btn_anim.clear()
    _btn_anim[key] = v
    return v


def _blit_button_label(surface, r, label, fnt, text_col, sub, sub_font, sub_col):
    """Zentriert Label (und optional Unterzeile) auf dem Button."""
    img = fnt.render(label, True, text_col)
    if sub and sub_font:
        sub_img = sub_font.render(sub, True, sub_col)
        total_h = img.get_height() + 2 + sub_img.get_height()
        y0 = r.centery - total_h // 2
        surface.blit(img, img.get_rect(midtop=(r.centerx, y0)))
        surface.blit(sub_img, sub_img.get_rect(
            midtop=(r.centerx, y0 + img.get_height() + 2)))
    else:
        surface.blit(img, img.get_rect(center=r.center))


def draw_button(surface, rect, label, fnt, selected=False, icon=None,
                sub=None, sub_font=None, accent=None):
    """Menü-Button im aktiven Theme.

    Modern : flache Fläche mit Haarlinien-Rand; bei Auswahl Akzent-Rand,
             hellere Füllung und ein schmaler Akzentbalken links - ohne
             Glow, Pfeil oder Puls-Animationen.
    Classic: weich animierte Auswahl mit Glow, Akzentbalken + Pfeil.
    """
    ac = accent or ACCENT
    r = pygame.Rect(rect)
    v = _btn_progress((r.x, r.y, r.w, r.h), selected)
    radius = _fx["btn_radius"]

    if is_modern():
        fill = mix(BTN, mix(PANEL_LIGHT, ac, 0.10), v)
        pygame.draw.rect(surface, fill, r, border_radius=radius)
        pygame.draw.rect(surface, mix(BORDER, ac, 0.85 * v), r,
                         width=1, border_radius=radius)
        if v > 0.05:
            bh = max(6, int((r.h - 14) * v))
            pygame.draw.rect(surface, ac,
                             (r.x + 7, r.centery - bh // 2, 3, bh),
                             border_radius=2)
        _blit_button_label(surface, r, label, fnt, mix(TEXT_DIM, TEXT, v),
                           sub, sub_font, mix(TEXT_FAINT, TEXT_DIM, v))
        return r

    if v > 0.02 and _fx["btn_glow"]:
        # Weicher Außen-Glow, pulsiert leicht
        glow_a = int(52 * v * (0.7 + 0.3 * pulse(2.2)))
        glow = pygame.Surface((r.w + 22, r.h + 22), pygame.SRCALPHA)
        pygame.draw.rect(glow, (*ac, glow_a), (0, 0, r.w + 22, r.h + 22),
                         border_radius=16)
        surface.blit(glow, (r.x - 11, r.y - 11))

    fill = mix(BTN, mix(BTN_SEL, ac, 0.15), v)
    pygame.draw.rect(surface, fill, r, border_radius=radius)
    # dezente Lichtkante oben (wirkt "erhaben")
    pygame.draw.rect(surface, mix(fill, (255, 255, 255), 0.05 + 0.10 * v),
                     (r.x + 10, r.y + 1, r.w - 20, 1))
    pygame.draw.rect(surface, mix(BORDER, ac, v), r,
                     width=2 if v > 0.5 else 1, border_radius=radius)

    if v > 0.05:
        # Akzentbalken links wächst mit der Auswahl
        bh = max(4, int((r.h - 16) * v))
        pygame.draw.rect(surface, ac, (r.x + 6, r.centery - bh // 2, 4, bh),
                         border_radius=2)
        if _fx["btn_arrow"]:
            # Pfeil-Marker rechts gleitet ein und "atmet" leicht
            cx = r.right - 18 + int((1.0 - v) * 10) + int(2 * pulse(2.6, lo=0.0))
            cy = r.centery
            pygame.draw.polygon(surface, mix(fill, TEXT, v),
                                [(cx - 4, cy - 6), (cx + 4, cy), (cx - 4, cy + 6)])

    _blit_button_label(surface, r, label, fnt, mix(TEXT_DIM, TEXT, v),
                       sub, sub_font, mix(TEXT_FAINT, TEXT_DIM, v))
    return r


# ---------------------------------------------------------------------------
#  Text-Effekte: Farbverlauf + weicher Glow (beides gecacht)
# ---------------------------------------------------------------------------

_text_fx_cache = {}


def grad_text(fnt, text, top=None, bottom=None):
    """Rendert Text mit vertikalem Farbverlauf (Standard: Weiß -> kühles Blau)."""
    top = top or (252, 253, 255)
    bottom = bottom or (165, 190, 235)
    key = ("grad", id(fnt), text, top, bottom)
    img = _text_fx_cache.get(key)
    if img is None:
        img = fnt.render(text, True, (255, 255, 255)).convert_alpha()
        gw, gh = img.get_size()
        grad = pygame.Surface((gw, gh), pygame.SRCALPHA)
        for yy in range(gh):
            f = yy / max(1, gh - 1)
            pygame.draw.line(grad, (*mix(top, bottom, f), 255),
                             (0, yy), (gw, yy))
        img.blit(grad, (0, 0), special_flags=pygame.BLEND_RGBA_MULT)
        if len(_text_fx_cache) > 48:
            _text_fx_cache.clear()
        _text_fx_cache[key] = img
    return img


def _text_glow(fnt, text, color):
    """Weich verschwommene Text-Kopie in 'color' (für additiven Glow)."""
    key = ("glow", id(fnt), text, color)
    img = _text_fx_cache.get(key)
    if img is None:
        raw = fnt.render(text, True, color)
        w, h = raw.get_size()
        pad = pygame.Surface((w + 40, h + 40), pygame.SRCALPHA)
        pad.blit(raw, (20, 20))
        small = pygame.transform.smoothscale(pad, (max(1, (w + 40) // 6),
                                                   max(1, (h + 40) // 6)))
        img = pygame.transform.smoothscale(small, (w + 40, h + 40))
        if len(_text_fx_cache) > 48:
            _text_fx_cache.clear()
        _text_fx_cache[key] = img
    return img


def draw_title(surface, width, title, subtitle=None, y=52, big=None,
               small=None, accent=None):
    """Zentrierter Titel im aktiven Theme.

    Modern : klarer Titel in Textfarbe + kurze Akzentlinie darunter.
    Classic: Glow + Schatten + Verlaufstext + doppelter Akzent-Unterstrich.
    """
    big = big or font(40, bold=True)
    ac = accent or ACCENT
    cx = width // 2

    if is_modern():
        img = big.render(title, True, TEXT)
        surface.blit(img, img.get_rect(center=(cx, y)))
        # Kurze, ruhige Akzentlinie unter dem Titel.
        lw = max(56, min(img.get_width() // 2, 160))
        ly = y + img.get_height() // 2 + 10
        pygame.draw.rect(surface, ac, (cx - lw // 2, ly, lw, 3),
                         border_radius=2)
        if subtitle:
            small = small or font(17)
            sub = small.render(subtitle, True, TEXT_DIM)
            surface.blit(sub, sub.get_rect(center=(cx, ly + 24)))
        return ly

    glow = _text_glow(big, title, tuple(int(c * 0.55) for c in ac))
    surface.blit(glow, glow.get_rect(center=(cx, y)),
                 special_flags=pygame.BLEND_ADD)

    sh = big.render(title, True, (0, 0, 0))
    sh.set_alpha(140)
    img = grad_text(big, title)
    surface.blit(sh, sh.get_rect(center=(cx + 2, y + 3)))
    surface.blit(img, img.get_rect(center=(cx, y)))

    # Akzentlinie unter dem Titel (kurz, mittig) + weicher Zweitstrich
    lw = min(img.get_width() + 20, width - 80)
    ly = y + img.get_height() // 2 + 8
    pygame.draw.rect(surface, ac, (cx - lw // 2, ly, lw, 3), border_radius=2)
    pygame.draw.rect(surface, mix(ac, ACCENT2, 0.6),
                     (cx - lw // 6, ly + 5, lw // 3, 2), border_radius=2)
    if subtitle:
        small = small or font(17)
        sub = small.render(subtitle, True, TEXT_DIM)
        surface.blit(sub, sub.get_rect(center=(cx, ly + 26)))
    return ly


def draw_footer(surface, width, height, text, fnt=None):
    """Fußzeile: dezente Trennlinie + Hinweistext unten."""
    fnt = fnt or font(14)
    img = fnt.render(text, True, TEXT_FAINT)
    y = height - 22
    pygame.draw.line(surface, BORDER, (width // 6, y - 12),
                     (width - width // 6, y - 12))
    surface.blit(img, img.get_rect(center=(width // 2, y)))


def pulse(speed=2.0, lo=0.35, hi=1.0):
    """Sinus-Puls 0..1 für sanfte Blink-/Atem-Animationen."""
    ticks = pygame.time.get_ticks() / 1000.0
    return lo + (hi - lo) * (0.5 + 0.5 * math.sin(ticks * speed))


# ---------------------------------------------------------------------------
#  Partikel: Funken (Klick/Start) und Konfetti (neuer Rekord).
#  Menüs/Screens SPAWNEN nur - gezeichnet wird zentral in main.py (draw_fx),
#  damit die Effekte auch über Screen-Wechsel hinweg weiterlaufen.
#  Im Modern-Theme sind die Deko-Funken aus; Konfetti (Rekord-Feedback) bleibt.
# ---------------------------------------------------------------------------

_particles = []
_MAX_PARTICLES = 420


def spawn_burst(x, y, color=None, n=18, speed=260):
    """Kleine Funken-Explosion, z.B. wenn ein Menüpunkt aktiviert wird."""
    if not _fx["sparks"]:
        return
    color = color or ACCENT
    for _ in range(n):
        if len(_particles) >= _MAX_PARTICLES:
            break
        ang = random.uniform(0, math.tau)
        sp = speed * random.uniform(0.25, 1.0)
        _particles.append(dict(
            kind="spark", x=x, y=y,
            vx=math.cos(ang) * sp, vy=math.sin(ang) * sp - 40,
            age=0.0, life=random.uniform(0.35, 0.7),
            color=color, size=random.uniform(1.5, 3.2)))


def spawn_confetti(w, h, n=90):
    """Konfetti-Regen von oben (neuer Highscore!)."""
    cols = [ACCENT, ACCENT2, GREEN, GOLD, RED, (240, 240, 250)]
    if is_modern():
        n = min(n, 60)   # etwas zurückhaltender, aber Feiern bleibt erlaubt
    for _ in range(n):
        if len(_particles) >= _MAX_PARTICLES:
            break
        _particles.append(dict(
            kind="confetti", x=random.uniform(0, w),
            y=random.uniform(-h * 0.25, 0),
            vx=random.uniform(-30, 30), vy=random.uniform(90, 220),
            age=0.0, life=random.uniform(2.0, 3.4),
            color=random.choice(cols),
            spin=random.uniform(4.0, 9.0), phase=random.uniform(0, math.tau)))


def _draw_particles(surface, dt):
    if not _particles:
        return
    w, h = surface.get_size()
    alive = []
    for p in _particles:
        p["age"] += dt
        if p["age"] >= p["life"]:
            continue
        fade = 1.0 - p["age"] / p["life"]
        if p["kind"] == "spark":
            p["vy"] += 420 * dt
            p["x"] += p["vx"] * dt
            p["y"] += p["vy"] * dt
            if -4 <= p["x"] < w and -4 <= p["y"] < h:
                s = max(1, int(p["size"] * fade + 0.5))
                surface.fill(mix((0, 0, 0), p["color"], fade),
                             (int(p["x"]), int(p["y"]), s, s),
                             special_flags=pygame.BLEND_ADD)
        else:   # confetti: taumelt, "Rotation" über oszillierende Breite
            p["x"] += (p["vx"] + 34 * math.sin(p["age"] * 3.0 + p["phase"])) * dt
            p["y"] += p["vy"] * dt
            if p["y"] < h + 8:
                bw = 2 + int(4 * abs(math.sin(p["age"] * p["spin"] + p["phase"])))
                col = mix(BG_BOTTOM, p["color"], min(1.0, fade * 1.8))
                pygame.draw.rect(surface, col,
                                 (int(p["x"]), int(p["y"]), bw, 5))
        alive.append(p)
    _particles[:] = alive


# ---------------------------------------------------------------------------
#  Screen-Übergang: kurzes Aufblenden aus Dunkel; im Classic-Theme läuft
#  zusätzlich eine Akzent-Scanlinie durch.
# ---------------------------------------------------------------------------

_trans = {"start": None, "dur": 0.35}
_fade_cache = {}


def begin_transition(dur=None):
    """Startet den Übergangs-Effekt (beim nächsten draw_fx sichtbar)."""
    _trans["start"] = pygame.time.get_ticks()
    _trans["dur"] = dur if dur is not None else _fx["trans_dur"]


def _fade_surface(w, h):
    key = (w, h)
    surf = _fade_cache.get(key)
    if surf is None:
        if len(_fade_cache) > 3:
            _fade_cache.clear()
        surf = pygame.Surface((w, h))
        surf.fill((6, 8, 16))
        _fade_cache[key] = surf
    return surf


def _draw_transition(surface, w, h):
    if _trans["start"] is None:
        return
    t = (pygame.time.get_ticks() - _trans["start"]) / 1000.0 / _trans["dur"]
    if t >= 1.0:
        _trans["start"] = None
        return
    # Dunkel-Overlay blendet aus ...
    ov = _fade_surface(w, h)
    ov.set_alpha(int(210 * (1.0 - t) ** 1.5))
    surface.blit(ov, (0, 0))
    if not _fx["scanline"]:
        return
    # ... während eine Akzent-Scanlinie nach unten durchläuft (Classic).
    y = int(h * t)
    glow_a = int(90 * (1.0 - t))
    line = pygame.Surface((w, 7), pygame.SRCALPHA)
    pygame.draw.rect(line, (*ACCENT, glow_a), (0, 0, w, 7))
    pygame.draw.rect(line, (*mix(ACCENT, (255, 255, 255), 0.5),
                            min(255, glow_a * 2)), (0, 3, w, 1))
    surface.blit(line, (0, y - 3))


def draw_fx(surface, w, h, dt):
    """Zeichnet alle globalen Effekte (Partikel + Übergang). 1x pro Frame."""
    _draw_particles(surface, dt)
    _draw_transition(surface, w, h)
