# -*- coding: utf-8 -*-
"""
ui.py
=====
Gemeinsames UI-Toolkit für alle Pygame-Screens (Menüs, Overlays, Spiele).

Seit dem UI-Update gibt es SIEBEN wählbare Designs ("Themes"):

- "v41"     (Standard, "UI v4.1"): wie "modern", aber lebendiger - leicht
  blau-violett getönte Palette, dezentes Sternenfeld und auf dem
  Startbildschirm ein Saturn und ein Schwarzes Loch (M87-Stil: dunkler
  Kern, glühender orangener Ring) im Hintergrund.
- "v411"    ("UI v4.1.1"): wie v4.1, aber statt Sternenhimmel ein
  gekacheltes Zickzack-Muster in Schwarz/Anthrazit als Hintergrund.
- "v412"    ("UI v4.1.2"): dasselbe Muster in den Blautönen der Palette
  (Akzentblau + dunkleres Blau).
- "v413"    ("UI v4.1.3"): dasselbe Muster im Indigo-Akzent von UI v4
  auf Schwarz.
- "v414"    ("UI v4.1.4"): dasselbe Muster im Graphit-Ton von UI v4
  auf Schwarz.
- "modern"  ("UI v4"): ruhiges, aufgeräumtes Graphit-Design mit einem
  einzelnen Indigo-Akzent. Flache Panels mit Haarlinien-Rand, dezente
  Hover-Übergänge, keine Deko-Effekte - wirkt clean und professionell.
- "classic" ("UI v3"): der bisherige Look - dunkler Farbverlauf mit
  "Aurora"-Lichtern, Parallax-Sternenfeld inkl. Sternschnuppen,
  Glow-Buttons, Verlaufstitel mit Leuchten, Funken und Scanlinien-Übergang.

Umgeschaltet wird über set_theme("v41"/"v411"/...) - im Spiel über den
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
    stars=True, star_bright=1.0,              # Sternenfeld (+ Helligkeit)
    aurora=True, shooting=True,               # Aurora-Lichter/Sternschnuppen
    celestial=False,                          # Saturn + Schwarzes Loch (v4.1)
    pattern=None,                             # Kachel-Muster (v4.1.x)
    vignette=70,                              # Vignetten-Stärke (Alpha)
    title_glow=True, title_grad=True,         # Titel: Glow + Verlaufstext
    btn_glow=True, btn_arrow=True,            # Buttons: Außen-Glow + Pfeil
    sparks=True,                              # Funken-Partikel bei Klicks
    scanline=True, trans_dur=0.35,            # Übergang: Fade + Scanlinie
    panel_radius=12, btn_radius=10,           # Eckenrundungen
    shadow_alpha=90,                          # Panel-Schlagschatten
    menu_bob=6,                               # Logo-Schweben (Amplitude px)
)

_MODERN_FX = dict(
    stars=False, star_bright=0.0,
    aurora=False, shooting=False,
    celestial=False,
    pattern=None,
    vignette=42,
    title_glow=False, title_grad=False,
    btn_glow=False, btn_arrow=False,
    sparks=False,
    scanline=False, trans_dur=0.22,
    panel_radius=10, btn_radius=8,
    shadow_alpha=55,
    menu_bob=0,
)

# UI v4.1: die cleane Modern-Optik, aber mit etwas Leben im Hintergrund -
# dezente Sterne, sanftes Logo-Schweben und Saturn + Schwarzes Loch auf dem
# Startbildschirm. Alle Bedien-Elemente bleiben wie im Modern-Theme.
_V41_FX = dict(
    stars=True, star_bright=0.55,
    aurora=False, shooting=False,
    celestial=True,
    pattern=None,
    vignette=48,
    title_glow=False, title_grad=False,
    btn_glow=False, btn_arrow=False,
    sparks=False,
    scanline=False, trans_dur=0.22,
    panel_radius=10, btn_radius=8,
    shadow_alpha=55,
    menu_bob=3,
)

# Palette v4.1: wie Modern, nur mit einem Hauch Blau-Violett im Hintergrund
# und leicht getönten Panels - weniger monoton, aber genauso ruhig.
_V41_COLORS = dict(
    BG_TOP=(16, 18, 28),
    BG_BOTTOM=(22, 25, 36),
    PANEL=(28, 31, 42),
    PANEL_LIGHT=(37, 41, 55),
    BORDER=(46, 51, 66),
    BORDER_LIGHT=(66, 72, 92),
    BTN=(33, 37, 49),
    BTN_SEL=(48, 57, 80),
    ACCENT=(91, 141, 239),
    ACCENT2=(129, 155, 255),
    ACCENT_SOFT=(64, 94, 156),
    GREEN=(88, 190, 132),
    GOLD=(229, 196, 106),
    RED=(224, 108, 108),
    TEXT=(233, 235, 241),
    TEXT_DIM=(150, 157, 172),
    TEXT_FAINT=(100, 106, 122),
)

# UI v4.1.1 bis v4.1.4: identisch zu v4.1 - nur der Hintergrund ist ein
# gekacheltes Zickzack-Muster statt Verlauf + Sternenhimmel. Die Sterne
# entfallen deshalb (auf dem Muster wären sie nur Bildrauschen), Saturn und
# Schwarzes Loch bleiben als Erkennungszeichen der v4.1-Familie.
#
# Musterfarben (main, alt): "main" ist die dominante Farbe (im CSS #000000),
# "alt" die zurückhaltendere Grundfarbe (im CSS #424242).
_V411_PATTERN = ((0, 0, 0), (66, 66, 66))            # Schwarz + Anthrazit
_V412_PATTERN = ((91, 141, 239), (64, 94, 156))      # ACCENT + ACCENT_SOFT
# v4.1.3/v4.1.4 nehmen die beiden Farben, die UI v4 ausmachen - den Indigo-
# Akzent und den neutralen Graphit-Ton - jeweils auf Schwarz.
_V413_PATTERN = ((91, 141, 239), (0, 0, 0))          # v4-Indigo + Schwarz
_V414_PATTERN = ((37, 41, 52), (0, 0, 0))            # v4-Graphit + Schwarz


def _pattern_fx(pattern):
    """v4.1-Effekte, aber mit Muster-Hintergrund statt Sternenfeld.

    Die Vignette fällt etwas kräftiger aus als in v4.1: das Muster ist
    unruhiger als ein Verlauf, und der dunklere Rahmen hält Reiter,
    Fußzeile & Co. lesbar - die Musterfarben selbst bleiben unangetastet.
    """
    d = dict(_V41_FX)
    d.update(stars=False, star_bright=0.0, vignette=64, pattern=pattern)
    return d


_V411_FX = _pattern_fx(_V411_PATTERN)
_V412_FX = _pattern_fx(_V412_PATTERN)
_V413_FX = _pattern_fx(_V413_PATTERN)
_V414_FX = _pattern_fx(_V414_PATTERN)

# Paletten: bewusst 1:1 die von v4.1 - der einzige Unterschied ist der
# Hintergrund. So bleiben Panels, Buttons und Spiele-Screens exakt gleich.
_V411_COLORS = dict(_V41_COLORS)
_V412_COLORS = dict(_V41_COLORS)
_V413_COLORS = dict(_V41_COLORS)
_V414_COLORS = dict(_V41_COLORS)


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

_TK_V41 = dict(
    SIDEBAR="#13161f", HEADER="#0f1219", CARD="#1a1e2a",
    BTN="#1f2431", BTN_HOVER="#293040",
    ACCENT="#5b8def", ACCENT2="#819bff",
    DANGER="#7c3540", DANGER_HOVER="#934250",
    BACK="#3c5a4b", BACK_HOVER="#4b7060",
    TEXT="#e9ebf1", TEXT_DIM="#969dac", TEXT_FAINT="#646a7a",
    BORDER="#2e3342", GREEN="#58be84", GOLD="#e5c46a", RED="#e06c6c",
)

# Die Muster-Themes teilen sich die Sidebar-Farben mit v4.1 (nur der
# Pygame-Hintergrund unterscheidet sich).
_TK_V411 = dict(_TK_V41)
_TK_V412 = dict(_TK_V41)
_TK_V413 = dict(_TK_V41)
_TK_V414 = dict(_TK_V41)

THEMES = {
    "v41": (_V41_COLORS, _V41_FX, _TK_V41),
    "v411": (_V411_COLORS, _V411_FX, _TK_V411),
    "v412": (_V412_COLORS, _V412_FX, _TK_V412),
    "v413": (_V413_COLORS, _V413_FX, _TK_V413),
    "v414": (_V414_COLORS, _V414_FX, _TK_V414),
    "modern": (_MODERN_COLORS, _MODERN_FX, _TK_MODERN),
    "classic": (_CLASSIC_COLORS, _CLASSIC_FX, _TK_CLASSIC),
}
THEME_NAMES = ("v41", "v411", "v412", "v413", "v414",
               "modern", "classic")
DEFAULT_THEME = "v41"

_theme = DEFAULT_THEME
_fx = _V41_FX
_tk = _TK_V41

# Die Palette des aktiven Themes liegt in den Modul-Globals (BG_TOP, ACCENT,
# ...), damit ALLE bestehenden ui.<NAME>-Zugriffe unverändert funktionieren.
globals().update(_V41_COLORS)


def set_theme(name):
    """Aktiviert ein Theme (Name aus THEME_NAMES) und leert alle Caches."""
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
    """Name des aktiven Themes (z.B. 'v41', 'v411', 'modern')."""
    return _theme


def is_modern():
    """True in der aufgeräumten Modern-Familie (UI v4, v4.1 und v4.1.x).

    Steuert die "cleanen" Zeichenpfade (flache Buttons/Titel/Panels ohne
    Glow und Puls). Was sich v4.1 zusätzlich gönnt (Sterne, Saturn,
    Schwarzes Loch), regeln die fx-Schalter des Themes.
    """
    return _theme != "classic"


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
    "BlockJumpGame": "#8fd14f", "LamaTowerDefenseGame": "#e2725b",
    "MiniGolfGame": "#4fd17a", "PinballGame": "#7f5af0",
    "BowlingGame": "#4a7de0",
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


# ---------------------------------------------------------------------------
#  Zickzack-Muster (UI v4.1.1 bis v4.1.4)
# ---------------------------------------------------------------------------
#  Pygame-Nachbau dieses CSS-Musters:
#
#      background:
#        conic-gradient(from 135deg,MAIN 90deg,#0000 0) 17px calc(17px/2),
#        conic-gradient(from 135deg,ALT  90deg,#0000 0),
#        conic-gradient(from 135deg at 50% 0,MAIN 90deg,#0000 0) ALT;
#      background-size: 34px 17px;
#
#  Jeder conic-gradient (von 135° über 90°) ist ein nach unten zeigendes
#  Dreieck ab seinem Mittelpunkt; gezeichnet wird in der CSS-Reihenfolge
#  (Grundfarbe zuerst, oberste Ebene zuletzt):
#
#    1. Grundfläche ALT
#    2. großes MAIN-Dreieck   (17,0) -> (0,17) / (34,17)      [3. Ebene]
#    3. kleines ALT-Dreieck   (17,8.5) -> (8.5,17) / (25.5,17)[2. Ebene]
#    4. MAIN-Dreieck um 17/8.5 versetzt -> zwei Hälften an den
#       seitlichen Rändern                                    [1. Ebene]

_PATTERN_UNIT = 17          # CSS: background-size 34px 17px
_pattern_cache = {}         # (main, alt, unit) -> Kachel-Surface


def _zigzag_tile(main, alt, unit=_PATTERN_UNIT):
    """Eine Kachel (2*unit x unit) des Zickzack-Musters, gecacht.

    Für weiche Diagonalen wird 4-fach vergrößert gezeichnet - und zwar
    gleich ein 3x3-Block, damit beim Verkleinern auch die Ränder korrekte
    Nachbarpixel haben. Danach wird die mittlere Kachel herausgeschnitten.
    """
    key = (main, alt, unit)
    tile = _pattern_cache.get(key)
    if tile is not None:
        return tile

    S = 4                                  # Supersampling
    u = unit * S                           # Kachelhöhe groß
    hu = u // 2
    block = pygame.Surface((6 * u, 3 * u))
    block.fill(alt)
    for ty in range(3):
        for tx in range(3):
            ox, oy = tx * 2 * u, ty * u
            # großes MAIN-Dreieck (Spitze oben Mitte -> untere Ecken)
            pygame.draw.polygon(block, main, [(ox + u, oy), (ox, oy + u),
                                              (ox + 2 * u, oy + u)])
            # kleines ALT-Dreieck ab der Kachelmitte
            pygame.draw.polygon(block, alt, [(ox + u, oy + hu),
                                             (ox + hu, oy + u),
                                             (ox + u + hu, oy + u)])
            # versetzte MAIN-Dreiecke an linkem und rechtem Rand
            pygame.draw.polygon(block, main, [(ox, oy), (ox + hu, oy + hu),
                                              (ox, oy + hu)])
            pygame.draw.polygon(block, main, [(ox + 2 * u, oy),
                                              (ox + u + hu, oy + hu),
                                              (ox + 2 * u, oy + hu)])
    small = pygame.transform.smoothscale(block, (6 * unit, 3 * unit))
    tile = pygame.Surface((2 * unit, unit))
    tile.blit(small, (0, 0), (2 * unit, unit, 2 * unit, unit))
    if len(_pattern_cache) > 8:
        _pattern_cache.clear()
    _pattern_cache[key] = tile
    return tile


def draw_zigzag(surface, w, h, main, alt, unit=_PATTERN_UNIT):
    """Füllt (0,0,w,h) mit dem gekachelten Zickzack-Muster.

    Erst wird eine komplette Zeile zusammengesetzt, die dann nur noch
    untereinander geblittet wird - das spart bei großen Flächen hunderte
    Einzel-Blits.
    """
    tile = _zigzag_tile(main, alt, unit)
    tw, th = tile.get_size()
    row = pygame.Surface((w, th))
    for x in range(0, w, tw):
        row.blit(tile, (x, 0))
    for y in range(0, h, th):
        surface.blit(row, (0, y))


def _base_background(w, h):
    """Hintergrundfläche + Vignette, pro Größe nur einmal berechnet.

    Normalfall ist ein vertikaler Verlauf; Themes mit fx('pattern')
    (UI v4.1.1 bis v4.1.4) bekommen stattdessen das Zickzack-Muster.
    """
    key = (w, h)
    surf = _bg_cache.get(key)
    if surf is None:
        pat = _fx.get("pattern")
        if pat:
            surf = pygame.Surface((w, h))
            draw_zigzag(surf, w, h, pat[0], pat[1])
        else:
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
    bright = _fx["star_bright"]
    for x, y, depth, r in _stars:
        # Langsame Drift nach oben + leichtes Funkeln über Sinus.
        yy = (y - ticks * 0.008 * depth) % 1.0
        tw = 0.5 + 0.5 * math.sin(ticks * (0.8 + depth) + x * 40.0)
        c = int((40 + 70 * depth * tw) * bright)
        surface.fill((c, c + 6, c + 18),
                     (int(x * w), int(yy * h), r, r))
    if _fx["shooting"]:
        _draw_shooting_star(surface, w, h)


# ---------------------------------------------------------------------------
#  Himmelskörper für UI v4.1: Saturn und ein Schwarzes Loch (M87-Stil).
#  Beide werden einmalig prozedural gerendert (klein zeichnen, weichzeichnen,
#  skalieren) und dann pro Frame nur noch geblittet - praktisch gratis.
# ---------------------------------------------------------------------------

_celestial_cache = {}


def _render_black_hole(radius):
    """Schwarzes Loch im "Interstellar"-Stil (Gargantua):

    - dunkle Kugel (Schatten) mit dünnem hellem Photonenring,
    - eine flache, glühend helle Akkretionsscheibe, die VOR der Kugel
      quer durchläuft (weiß-gelb innen, orange zu den Spitzen),
    - gelinstes Scheibenlicht als Glut-Bogen über und unter der Kugel,
    - Doppler-Hotspot (die auf uns zu rotierende Seite strahlt weiß).

    Wird einmal groß (512 px) gerendert, leicht gekippt und gecacht.
    """
    key = ("bh", int(radius))
    img = _celestial_cache.get(key)
    if img is not None:
        return img

    S = 512
    c = S // 2
    sh = 66            # Schatten-Radius
    rh = 84            # Radius des gelinsten Halo-Rings (dicht an der Kugel)
    Rd = 246           # Scheibenradius (bis fast an den Rand)

    surf = pygame.Surface((S, S), pygame.SRCALPHA)

    # --- 1) Warme Glut + Halo-Ring (wird stark weichgezeichnet) ----------
    haze = pygame.Surface((S, S), pygame.SRCALPHA)
    for r in range(rh + 120, 4, -1):
        d = abs(r - rh) / (100.0 if r > rh else 34.0)
        i = max(0.0, 1.0 - d) ** 2
        if i <= 0.004:
            continue
        col = (int(255 * min(1.0, i * 1.25)), int(165 * i), int(66 * i))
        pygame.draw.circle(haze, (*col, int(235 * i)), (c, c), r)
    # Gelinstes Licht: Bogen über der Kugel deutlich, unter ihr schwächer.
    arc = pygame.Surface((S, S), pygame.SRCALPHA)
    for cy_off, strength in ((-rh, 1.0), (rh, 0.6)):
        for r in range(96, 3, -1):
            f = (1.0 - r / 96.0) ** 2 * strength
            pygame.draw.circle(arc, (int(255 * f), int(185 * f), int(90 * f)),
                               (c, c + cy_off), r)
    haze.blit(arc, (0, 0), special_flags=pygame.BLEND_RGB_ADD)
    small = pygame.transform.smoothscale(haze, (S // 4, S // 4))
    haze = pygame.transform.smoothscale(small, (S, S))
    surf.blit(haze, (0, 0))

    # --- 2) Schatten-Kugel (dunkelbraun) + Photonenring ------------------
    pygame.draw.circle(surf, (16, 8, 6, 255), (c, c), sh)
    pygame.draw.circle(surf, (70, 28, 14, 110), (c, c), sh - 2, width=5)
    # Photonenring mit weichen Kanten (gestaffelte Alpha-Ringe gegen Treppen).
    pygame.draw.circle(surf, (255, 200, 130, 80), (c, c), sh + 7, width=8)
    pygame.draw.circle(surf, (255, 228, 180, 170), (c, c), sh + 5, width=5)
    pygame.draw.circle(surf, (255, 244, 214, 235), (c, c), sh + 3, width=4)

    # --- 3) Akkretionsscheibe: weißglühendes Band VOR der Kugel ----------
    TH = 28            # halbe Dicke in der Mitte
    disk = pygame.Surface((S, S), pygame.SRCALPHA)
    for t in range(TH, 0, -1):
        i = (1.0 - t / TH) ** 0.8          # breiter heller Kern
        col = mix((255, 186, 92), (255, 252, 234), i)
        pygame.draw.ellipse(disk, (*col, int(238 + 17 * i)),
                            (c - Rd, c - t, 2 * Rd, 2 * t))
    # Hauchzarte Wirbel-Bahnen: OPAKE, nur leicht dunklere Linien (pygame
    # ersetzt Pixel statt zu mischen - halbtransparente Linien würden Löcher
    # stanzen). Länge an die Ellipsenhöhe angepasst, damit nichts übersteht.
    for fy, col in ((-0.60, (243, 198, 128)), (-0.28, (248, 214, 154)),
                    (0.26, (246, 206, 140)), (0.56, (240, 190, 118))):
        yy = c + int(TH * fy)
        span = int(Rd * math.sqrt(max(0.0, 1.0 - fy * fy)) * 0.94)
        pygame.draw.line(disk, (*col, 255), (c - span, yy), (c + span, yy), 2)
    # Radiale Tönung: Mitte heiß-weiß, zu den Spitzen warmes Orange; erst
    # ganz außen sanft ausblenden -> das Band leuchtet über die volle Länge.
    tint = pygame.Surface((S, S), pygame.SRCALPHA)
    for x in range(S):
        dx = min(1.0, abs(x - c) / Rd)
        g = int(255 * (1.0 - 0.18 * dx))
        b = int(255 * (1.0 - 0.36 * dx))
        a = int(255 * max(0.0, 1.0 - dx ** 4.0))
        pygame.draw.line(tint, (255, g, b, a), (x, 0), (x, S))
    disk.blit(tint, (0, 0), special_flags=pygame.BLEND_RGBA_MULT)
    # Leicht weichzeichnen, damit das Band glüht statt hart zu wirken.
    small = pygame.transform.smoothscale(disk, (S // 2, S // 2))
    disk = pygame.transform.smoothscale(small, (S, S))
    # Erst zwei verwaschene Kopien ADDITIV (Leuchten um das ganze Band) ...
    for div in (6, 10):
        band = pygame.transform.smoothscale(disk, (S // div, S // div))
        band = pygame.transform.smoothscale(band, (S, S))
        surf.blit(band, (0, 0), special_flags=pygame.BLEND_RGB_ADD)
    # ... dann die Scheibe selbst darüber.
    surf.blit(disk, (0, 0))

    # --- 4) Doppler-Hotspot: rechte Seite strahlt weiß-gelb --------------
    hot = pygame.Surface((S, S), pygame.SRCALPHA)
    for r in range(104, 3, -1):
        f = (1.0 - r / 104.0) ** 2
        pygame.draw.circle(hot, (int(255 * f), int(235 * f), int(180 * f)),
                           (c + sh + 18, c - 4), r)
    surf.blit(hot, (0, 0), special_flags=pygame.BLEND_RGB_ADD)

    # --- 5) Kippen (Scheibe steigt nach rechts an) + Zielgröße -----------
    surf = pygame.transform.rotate(surf, 26)
    img = pygame.transform.smoothscale(surf, (radius * 2, radius * 2))
    if len(_celestial_cache) > 8:
        _celestial_cache.clear()
    _celestial_cache[key] = img
    return img


def _render_saturn(radius):
    """Saturn mit Bändern, Kugel-Schattierung und gekippten Ringen.

    radius = gewünschter Planeten-Radius; die Ringe stehen seitlich über.
    """
    key = ("saturn", int(radius))
    img = _celestial_cache.get(key)
    if img is not None:
        return img

    S = 320            # groß rendern -> bleibt auch hochskaliert glatt
    c = S // 2
    pr = 68            # Planeten-Radius auf der Arbeitsfläche

    # Ring-Ebene: Ellipsen von außen nach innen; transparente Ellipsen
    # "radieren" (pygame.draw ersetzt Pixel), so entstehen die Teilungen.
    rings = pygame.Surface((S, S), pygame.SRCALPHA)

    def ell(rx, ry, col):
        pygame.draw.ellipse(rings, col, (c - rx, c - ry, rx * 2, ry * 2))

    ell(124, 38, (205, 183, 148, 175))   # A-Ring
    ell(112, 34, (0, 0, 0, 0))           # Cassini-Teilung
    ell(106, 32, (230, 208, 172, 210))   # B-Ring (am hellsten)
    ell(84, 24, (176, 155, 124, 120))    # C-Ring (blass)
    ell(74, 22, (0, 0, 0, 0))            # Loch innen

    # Planet: sandfarbene Bänder, rund maskiert, zum Rand hin abgedunkelt.
    planet = pygame.Surface((S, S), pygame.SRCALPHA)
    tones = ((232, 210, 170), (212, 186, 142), (226, 202, 162),
             (206, 180, 138), (228, 204, 164), (216, 190, 148))
    band_h = (pr * 2) / len(tones)
    for i, tone in enumerate(tones):
        pygame.draw.rect(planet, tone,
                         (c - pr, int(c - pr + i * band_h),
                          pr * 2, int(band_h) + 1))
    mask = pygame.Surface((S, S), pygame.SRCALPHA)
    pygame.draw.circle(mask, (255, 255, 255, 255), (c, c), pr)
    planet.blit(mask, (0, 0), special_flags=pygame.BLEND_RGBA_MIN)
    shade = pygame.Surface((S, S), pygame.SRCALPHA)
    for r in range(pr + 24, 0, -1):
        a = min(160, int(150 * (r / pr) ** 3))
        pygame.draw.circle(shade, (18, 14, 24, a), (c + 14, c + 12), r)
    shade.blit(mask, (0, 0), special_flags=pygame.BLEND_RGBA_MIN)
    planet.blit(shade, (0, 0))

    # Zusammensetzen: ferne Ringhälfte, Planet, nahe Ringhälfte darüber.
    surf = pygame.Surface((S, S), pygame.SRCALPHA)
    surf.blit(rings, (0, 0), area=pygame.Rect(0, 0, S, c))
    surf.blit(planet, (0, 0))
    surf.blit(rings, (0, c), area=pygame.Rect(0, c, S, S - c))

    # Leicht kippen und auf die Zielgröße bringen.
    surf = pygame.transform.rotate(surf, 20)
    scale = radius / float(pr)
    size = max(2, int(surf.get_width() * scale))
    img = pygame.transform.smoothscale(surf, (size, size))
    if len(_celestial_cache) > 8:
        _celestial_cache.clear()
    _celestial_cache[key] = img
    return img


def draw_black_hole(surface, center, radius):
    """Blittet das (gecachte) Schwarze Loch zentriert an 'center'."""
    radius = max(8, (int(radius) // 4) * 4)   # Größen bündeln -> Cache klein
    img = _render_black_hole(radius)
    surface.blit(img, img.get_rect(center=center))


def draw_saturn(surface, center, radius):
    """Blittet den (gecachten) Saturn zentriert an 'center' (Planetenmitte)."""
    radius = max(6, (int(radius) // 4) * 4)
    img = _render_saturn(radius)
    surface.blit(img, img.get_rect(center=center))


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


# ---------------------------------------------------------------------------
#  Texteingabe
# ---------------------------------------------------------------------------
#
# Das erste echte Eingabefeld des Projekts - gebraucht für Namen und ids der
# eigenen Minigolf-Bahnen (siehe games/minigolf_edit.py).
#
# Besonderheit dieser Oberfläche: Tastendrücke kommen NICHT aus pygame,
# sondern als Tkinter-Ereignisse (siehe Kopf von main.py). Deshalb wird
# vorrangig InputEvent.char ausgewertet - das ist das tatsächlich getippte
# Zeichen inklusive Umlauten und Groß-/Kleinschreibung. Fehlt es (ältere
# Aufrufer, Gamepad), springt der keysym-Notnagel ein.

class TextInput:
    """Einzeiliges Eingabefeld mit Schreibmarke.

    Verwendung::

        self.feld = ui.TextInput(maxlen=28, placeholder=t("golf.ugc.name"))
        ...
        if self.feld.handle(event):      # True = Ereignis verbraucht
            return
        self.feld.draw(surface, rect, font, focused=True)
        name = self.feld.text

    'charset' begrenzt die erlaubten Zeichen (z.B. ``TextInput.ID_CHARS``
    für Bahn-ids, die keine Leerzeichen enthalten dürfen).
    """

    # Erlaubte Zeichen einer Bahn-id - klein und ohne Leerzeichen.
    ID_CHARS = "abcdefghijklmnopqrstuvwxyz0123456789-_"

    # Tasten, die ein Textfeld nicht selbst verarbeitet (der Aufrufer soll
    # sie sehen: Tab wechselt das Feld, Return/Escape schließen den Dialog).
    PASS_THROUGH = ("Tab", "ISO_Left_Tab", "Return", "KP_Enter", "Escape",
                    "Up", "Down")

    def __init__(self, text="", maxlen=28, charset=None, placeholder=""):
        self.maxlen = int(maxlen)
        self.charset = charset
        self.placeholder = placeholder
        self.caret = 0
        self.text = ""
        self.set_text(text)

    # ----- Inhalt -------------------------------------------------------
    def set_text(self, text):
        """Setzt den Inhalt (filtert und kürzt wie bei der Eingabe)."""
        text = "" if text is None else str(text)
        self.text = "".join(c for c in text if self._ok(c))[:self.maxlen]
        self.caret = len(self.text)

    def _ok(self, ch):
        """Darf 'ch' ins Feld?"""
        if not ch or ord(ch) < 32 or ch == "\x7f":
            return False
        return self.charset is None or ch.lower() in self.charset

    def insert(self, ch):
        if len(self.text) >= self.maxlen or not self._ok(ch):
            return False
        if self.charset is not None:
            ch = ch.lower()
        self.text = self.text[:self.caret] + ch + self.text[self.caret:]
        self.caret += 1
        return True

    # ----- Eingabe ------------------------------------------------------
    def handle(self, event):
        """Verarbeitet ein KEYDOWN-InputEvent. True = verbraucht."""
        from game_base import InputEvent
        if event.kind != InputEvent.KEYDOWN:
            return False
        key = event.key
        if key in self.PASS_THROUGH:
            return False
        if key == "BackSpace":
            if self.caret > 0:
                self.text = self.text[:self.caret - 1] + self.text[self.caret:]
                self.caret -= 1
            return True
        if key == "Delete":
            self.text = self.text[:self.caret] + self.text[self.caret + 1:]
            return True
        if key == "Left":
            self.caret = max(0, self.caret - 1)
            return True
        if key == "Right":
            self.caret = min(len(self.text), self.caret + 1)
            return True
        if key in ("Home", "KP_Home"):
            self.caret = 0
            return True
        if key in ("End", "KP_End"):
            self.caret = len(self.text)
            return True
        # Der Normalfall: das getippte Zeichen einfügen.
        ch = getattr(event, "char", None)
        if ch:
            return self.insert(ch)
        # Notnagel ohne char-Kanal: einzelne keysyms sind das Zeichen selbst,
        # "space" ist ausgeschrieben (wie in lamawiki.py).
        if key == "space":
            return self.insert(" ")
        if isinstance(key, str) and len(key) == 1 and key.isprintable():
            return self.insert(key)
        return False

    # ----- Zeichnen -----------------------------------------------------
    def draw(self, surface, rect, fnt, focused=False, invalid=False):
        """Zeichnet das Feld. 'invalid' färbt den Rand rot."""
        r = pygame.Rect(rect)
        border = RED if invalid else (ACCENT if focused else BORDER)
        pygame.draw.rect(surface, PANEL_LIGHT if focused else PANEL, r,
                         border_radius=7)
        pygame.draw.rect(surface, border, r, 2 if (focused or invalid) else 1,
                         border_radius=7)
        pad = 8
        inner = r.w - 2 * pad
        show = self.text if self.text else self.placeholder
        col = TEXT if self.text else TEXT_FAINT
        # Bei langem Text nach links schieben, damit die Schreibmarke im Bild
        # bleibt - das Feld ist schmal, Namen dürfen trotzdem lang sein.
        off = 0
        if self.text:
            upto = fnt.size(self.text[:self.caret])[0]
            if upto > inner:
                off = upto - inner
        img = fnt.render(show, True, col)
        clip = surface.get_clip()
        surface.set_clip(r.inflate(-4, -4))
        surface.blit(img, (r.x + pad - off, r.centery - img.get_height() // 2))
        if focused:
            cx = r.x + pad - off + fnt.size(self.text[:self.caret])[0]
            if int(pulse(3.0, 0.0, 1.99)) == 0:      # blinkt ~1.5x je Sekunde
                pygame.draw.rect(surface, ACCENT,
                                 (cx, r.y + 5, 2, r.h - 10))
        surface.set_clip(clip)
        return r
