# -*- coding: utf-8 -*-
"""
achievements.py
===============
Erfolge (Achievements) für die Spielesammlung.

Drei Kategorien:

- ``general``   : sammlungsweite Erfolge aus den Statistiken (stats.py),
                  z.B. "10 Partien", "5 verschiedene Spiele", "1 h Spielzeit",
                  plus einige Ereignis-Erfolge (Nachteule, Wiki geöffnet ...).
- ``milestone`` : ein Punkte-Meilenstein je Spiel ("Erreiche {n} Punkte") -
                  Name/Akzentfarbe kommen vom Spiel selbst, die Schwelle steht
                  in ``MILESTONES`` (Schlüssel = highscore_key).
- ``special``   : besondere Einzelmomente, die die Spiele selbst über
                  ``Game.ach_event("...")`` melden (Schachmatt, KNIFFEL,
                  2048-Kachel, Wordle in 2 Versuchen ...).

Freigeschaltete Erfolge liegen im Abschnitt ``achievements`` der gemeinsamen
Datei ``mem.json`` als ``{id: "YYYY-MM-DD HH:MM"}``. Ein Unlock zeigt einen
Toast oben rechts (gezeichnet von main.py über ``draw_toasts``, damit er auch
mitten im Spiel über allem liegt) und spielt ein kurzes Arpeggio.

``backfill_silent()`` prüft beim App-Start alle Bedingungen gegen die schon
vorhandenen Highscores/Statistiken und schaltet Erreichtes OHNE Toast frei -
so zählen die Bestwerte von Bestandsspielern ab dem ersten Start des Updates.
"""

import time

import pygame

import audio
import stats
import store
import ui

_SECTION = "achievements"

# ----- Definitionen -------------------------------------------------------
#
# Sammlungsweite Erfolge: (id, icon, stat, target).
#   stat   : Feld aus stats.totals() ("plays"/"distinct"/"time"/"records"/
#            "wins") oder None für reine Ereignis-Erfolge (event()).
#   target : Schwelle (Sekunden bei "time"); None bei Ereignis-Erfolgen.
# Die Anzeige-Reihenfolge entspricht dieser Liste.
GENERAL = [
    ("first_game",  "star",   "plays",    1),
    ("plays_10",    "flag",   "plays",    10),
    ("plays_50",    "flag",   "plays",    50),
    ("plays_200",   "crown",  "plays",    200),
    ("explorer_5",  "gem",    "distinct", 5),
    ("explorer_15", "gem",    "distinct", 15),
    ("explorer_all", "crown", "distinct", None),   # Ziel = Anzahl aller Spiele
    ("time_1h",     "clock",  "time",     3600),
    ("time_5h",     "clock",  "time",     18000),
    ("time_20h",    "crown",  "time",     72000),
    ("records_1",   "trophy", "records",  1),
    ("records_10",  "trophy", "records",  10),
    ("records_25",  "crown",  "records",  25),
    ("wins_10",     "medal",  "wins",     10),
    ("wins_50",     "medal",  "wins",     50),
    ("victor_5",    "medal",  "win_games", 5),
    ("night_owl",   "star",   None,       None),
    ("early_bird",  "star",   None,       None),
    ("polyglot",    "heart",  None,       None),
    ("wiki_reader", "heart",  None,       None),
    ("painter",     "heart",  None,       None),
]

# Beschreibungs-Vorlagen der zählbaren Erfolge: stat -> i18n-Schlüssel
# (gefüllt mit {n} bzw. {t}); Ereignis-Erfolge haben eigene desc-Schlüssel.
_STAT_DESC = {
    "plays": "ach.plays.desc",
    "distinct": "ach.explorer.desc",
    "time": "ach.time.desc",
    "records": "ach.records.desc",
    "wins": "ach.wins.desc",
    "win_games": "ach.victor.desc",
}

# Punkte-Meilenstein je Spiel: highscore_key -> Schwelle ("solide Leistung",
# abgeleitet aus der Punkte-Skala des Spiels). Spiele, deren score nur ein
# Siegzähler gegen die KI ist (Schach, Dame, Mühle, Reversi, Vier gewinnt,
# Tic-Tac-Toe, Panzer-Duell, Billard), fehlen bewusst - sie haben stattdessen
# einen special-Erfolg bzw. zählen über die Sieg-Statistik.
MILESTONES = {
    "snake": 500,          # ~50 Äpfel klassisch
    "pong": 5,             # 5 Punkte = gewonnenes Match
    "airhockey": 5,        # 5 Tore = gewonnenes Standard-Match
    "breakout": 8000,      # mehrere Level mit Combo
    "tetris": 10000,       # einige Mehrfach-Reihen
    "invaders": 3000,      # ~Level 4-5 inkl. Boss
    "asteroids": 4000,     # ~Welle 5-8
    "pacman": 10000,       # Extraleben-Schwelle
    "flappy": 25,          # Silber-Medaille
    "doodle": 4000,        # obere Schwierigkeits-Skala
    "2048": 10000,         # ~1024er-Kachel
    "minesweeper": 300,    # Sieg auf Fortgeschritten
    "sudoku": 2500,        # zügig und fehlerarm gelöst
    "frogger": 3000,       # ~ein volles Level
    "memory": 1500,        # 6x6 flott gelöst
    "solitaire": 500,      # gewonnene Partie mit Bonus
    "aim": 4000,           # gute Sitzung
    "blackjack": 800,      # Chips deutlich vermehrt
    "tunnel": 15000,       # lange Endlos-Fahrt
    "maze": 1500,          # ~2 Level mit Orbs
    "kniffel": 200,        # Gesamtsumme mit Bonus
    "wordle": 150,         # ~3-4 Wörter in Folge
    "trex": 600,           # lange Strecke
    "poker": 1500,         # Start-Chips ver-1,5-facht
    "simon": 12,           # 12er-Sequenz
    "slide": 4000,         # 4x4 flott gelöst
    "mastermind": 150,     # mehrere Codes in Folge
    "bubble": 1500,        # großes Feld geräumt
    "hangman": 150,        # mehrere Wörter in Folge
    "blockjump": 3000,     # einige Level geschafft
    "lamatowerdef": 10000,     # ~Welle 18-20 auf der Wiese (Klassisch)
}

# Besondere Momente: (id, icon, highscore_key, target).
#   highscore_key: verknüpft den Erfolg optisch mit einem Spiel (Farbe/Name);
#                  None = spielübergreifend.
#   target : None = event(id) schaltet direkt frei; Zahl = event(id, wert)
#            schaltet erst ab wert >= target frei (z.B. Poker-Chips).
SPECIALS = [
    ("chess_win",      "crown", "chess",       None),   # KI matt gesetzt
    ("tile_2048",      "gem",   "2048",        None),   # 2048-Kachel
    ("tetris_four",    "star",  "tetris",      None),   # 4 Reihen auf einmal
    ("kniffel_five",   "star",  "kniffel",     None),   # Kniffel gewürfelt
    ("blackjack_two",  "medal", "blackjack",   None),   # Blackjack mit 2 Karten
    ("poker_rich",     "medal", "poker",       2000),   # Chips verdoppelt
    ("wordle_two",     "gem",   "wordle",      None),   # in <= 2 Versuchen
    ("hangman_clean",  "check", "hangman",     None),   # ohne Fehlversuch
    ("sudoku_clean",   "check", "sudoku",      None),   # fehlerfrei gelöst
    ("mine_win",       "flag",  "minesweeper", None),   # Feld geräumt
    ("solitaire_win",  "check", "solitaire",   None),   # Partie gewonnen
    ("memory_perfect", "heart", "memory",      None),   # ohne Fehlpaar
    ("pacman_clear",   "flag",  "pacman",      None),   # Labyrinth leergefuttert
    ("breakout_clear", "flag",  "breakout",    None),   # alle Level geschafft
    ("frog_home",      "flag",  "frogger",     None),   # alle 5 Buchten voll
    ("aim_perfect",    "star",  "aim",         None),   # 100% Genauigkeit
    ("snake_prestige", "gem",   "snake",       None),   # Prestige I erreicht
    ("snake_comp5",    "crown", "snake",       5),      # Competitive Level 5
    ("td_boss",        "crown", "lamatowerdef",    None),   # ersten Boss besiegt
    ("td_wave20",      "flag",  "lamatowerdef",    None),   # Welle 20 geschafft
    ("td_perfect10",   "heart", "lamatowerdef",    None),   # Welle 10 ohne Verlust
    ("td_maxed",       "gem",   "lamatowerdef",    None),   # Turm voll ausgebaut
]

_unlocked = None      # {id: "YYYY-MM-DD HH:MM"} (In-Memory-Kopie)
_defs = None          # Definitionen in Anzeige-Reihenfolge
_by_id = None         # id -> Definition
_milestone_by_game = None   # highscore_key -> Milestone-Definition


# ----- Aufbau -------------------------------------------------------------

def _load_unlocked():
    global _unlocked
    if _unlocked is None:
        raw = store.load_section(_SECTION)
        _unlocked = {str(k): str(v) for k, v in raw.items()}
    return _unlocked


def _build_defs():
    """Baut die Definitionsliste auf (einmalig; braucht die Spielklassen)."""
    global _defs, _by_id, _milestone_by_game
    if _defs is not None:
        return _defs
    from games import ALL_GAMES
    by_key = {cls.highscore_key: cls for cls in ALL_GAMES}

    defs = []
    for ach_id, icon, stat, target in GENERAL:
        if ach_id == "explorer_all":
            target = len(ALL_GAMES)
        d = dict(id=ach_id, cat="general", icon=icon, stat=stat,
                 target=target, game=None, game_cls=None)
        defs.append(d)
    for cls in ALL_GAMES:
        key = cls.highscore_key
        if key in MILESTONES:
            defs.append(dict(id="score_" + key, cat="milestone", icon="star",
                             stat=None, target=MILESTONES[key], game=key,
                             game_cls=cls))
    for ach_id, icon, game_key, target in SPECIALS:
        defs.append(dict(id=ach_id, cat="special", icon=icon, stat=None,
                         target=target, game=game_key,
                         game_cls=by_key.get(game_key)))

    _defs = defs
    _by_id = {d["id"]: d for d in defs}
    _milestone_by_game = {d["game"]: d for d in defs if d["cat"] == "milestone"}
    return _defs


# ----- Abfragen (für den Erfolge-Screen) ----------------------------------

def all_defs():
    """Alle Erfolgs-Definitionen in Anzeige-Reihenfolge."""
    return list(_build_defs())


def unlocked():
    """Freigeschaltete Erfolge als dict {id: zeitstempel}."""
    return dict(_load_unlocked())


def counts():
    """(freigeschaltet, gesamt) für Fortschrittsanzeigen."""
    return len(_load_unlocked()), len(_build_defs())


def display_name(d):
    """Lokalisierter Name eines Erfolgs (Meilensteine = Spielname)."""
    from i18n import t
    if d["cat"] == "milestone":
        return d["game_cls"].name
    return t("ach.%s.name" % d["id"])


def display_desc(d):
    """Lokalisierte Beschreibung eines Erfolgs."""
    from i18n import t
    if d["cat"] == "milestone":
        return t("ach.score.desc", n=d["target"])
    if d["cat"] == "general" and d["stat"]:
        key = _STAT_DESC[d["stat"]]
        if d["stat"] == "time":
            return t(key, t="%dh" % (d["target"] // 3600))
        return t(key, n=d["target"])
    return t("ach.%s.desc" % d["id"])


def progress_of(d, totals=None, scores=None):
    """Fortschritt (aktuell, ziel) eines zählbaren Erfolgs, sonst None.

    totals/scores können vorab geladen übergeben werden (der Screen macht das
    einmal pro Aufbau, statt pro Erfolg die Dateien zu lesen).
    """
    if d["cat"] == "general" and d["stat"] and d["target"]:
        if totals is None:
            totals = stats.totals()
        return min(totals[d["stat"]], d["target"]), d["target"]
    if d["cat"] == "milestone":
        if scores is None:
            import highscore
            scores = highscore.load_highscores()
        return min(scores.get(d["game"], 0), d["target"]), d["target"]
    return None


# ----- Freischalten -------------------------------------------------------

def _save_unlocked():
    store.save_section(_SECTION, dict(_load_unlocked()))


def _unlock(d, silent=False):
    """Schaltet einen Erfolg frei (einmalig); Toast + Sound, wenn nicht silent."""
    unlocked_map = _load_unlocked()
    if d["id"] in unlocked_map:
        return False
    unlocked_map[d["id"]] = time.strftime("%Y-%m-%d %H:%M")
    _save_unlocked()
    if not silent:
        _toasts.append({"def": d, "t": 0.0, "notes": []})
    return True


def check_stats(silent=False):
    """Prüft alle statistik-basierten Erfolge (nach Partien/Siegen/Zeit)."""
    totals = stats.totals()
    for d in _build_defs():
        if d["cat"] == "general" and d["stat"] and d["target"]:
            if totals[d["stat"]] >= d["target"]:
                _unlock(d, silent)


def on_highscore(key, score, silent=False):
    """Prüft den Punkte-Meilenstein eines Spiels (nach Highscore-Update)."""
    _build_defs()
    d = _milestone_by_game.get(key)
    if d is not None and score >= d["target"]:
        _unlock(d, silent)


def on_game_started(key):
    """Wird bei jedem Partie-Start gerufen (auch Neustart nach Game Over)."""
    hour = time.localtime().tm_hour
    if 0 <= hour < 5:
        event("night_owl")
    elif 5 <= hour < 8:
        event("early_bird")
    check_stats()


def event(ach_id, value=None):
    """Löst einen Ereignis-Erfolg aus (unbekannte ids werden ignoriert).

    Erfolge mit Ziel-Wert (z.B. Poker-Chips) schalten erst frei, wenn
    ``value`` das Ziel erreicht.
    """
    _build_defs()
    d = _by_id.get(ach_id)
    if d is None:
        return
    if d["target"] is not None and d["stat"] is None and d["cat"] != "milestone":
        if value is None or value < d["target"]:
            return
    _unlock(d)


def backfill_silent():
    """Beim App-Start: Erreichtes aus Bestandsdaten OHNE Toast freischalten."""
    try:
        import highscore
        scores = highscore.load_highscores()
        check_stats(silent=True)
        for key, score in scores.items():
            on_highscore(key, score, silent=True)
    except Exception:
        # Erfolge dürfen den Start nie verhindern.
        pass


# ----- Toast-Einblendung (oben rechts, über allem) ------------------------

_toasts = []                       # wartende/aktive Einblendungen (FIFO)
_TOAST_IN, _TOAST_HOLD, _TOAST_OUT = 0.35, 3.4, 0.45
# Kleines Aufstiegs-Arpeggio beim Erscheinen (Zeitmarke, Frequenz).
_ARPEGGIO = ((0.00, 660), (0.13, 880), (0.26, 1318))


def _ease_out(x):
    x = max(0.0, min(1.0, x))
    return 1.0 - (1.0 - x) ** 3


def draw_toasts(surface, w, h, dt, settings=None):
    """Zeichnet die aktive Erfolgs-Einblendung (von main.py pro Frame gerufen)."""
    if not _toasts:
        return
    toast = _toasts[0]
    toast["t"] += dt
    tt = toast["t"]

    # Arpeggio-Töne zu ihren Zeitmarken abspielen (je genau einmal).
    for i, (at, freq) in enumerate(_ARPEGGIO):
        if tt >= at and i not in toast["notes"]:
            toast["notes"].append(i)
            audio.tone(freq, 0.22, settings, wave="square", vol=0.30)

    total = _TOAST_IN + _TOAST_HOLD + _TOAST_OUT
    if tt >= total:
        _toasts.pop(0)
        return

    d = toast["def"]
    accent = ui.game_color(d["game_cls"].__name__) if d["game_cls"] else ui.GOLD

    head_font = ui.font(12, bold=True)
    name_font = ui.font(17, bold=True)
    head = head_font.render(display_name_toast_header(), True, ui.GOLD)
    name = name_font.render(display_name(d), True, ui.TEXT)

    br = 17                                   # Badge-Radius
    pad, gap = 14, 10
    bw = min(w - 24, pad * 2 + br * 2 + gap + max(head.get_width(),
                                                  name.get_width()))
    bh = max(br * 2 + 16, head.get_height() + name.get_height() + 22)

    # Von rechts einschieben, am Ende wieder hinausschieben.
    if tt < _TOAST_IN:
        off = (1.0 - _ease_out(tt / _TOAST_IN)) * (bw + 24)
    elif tt > _TOAST_IN + _TOAST_HOLD:
        off = _ease_out((tt - _TOAST_IN - _TOAST_HOLD) / _TOAST_OUT) * (bw + 24)
    else:
        off = 0.0

    rect = pygame.Rect(int(w - 12 - bw + off), 12, bw, bh)
    ui.draw_panel(surface, rect, color=ui.PANEL_LIGHT, radius=12,
                  accent_top=ui.GOLD)
    # Dezenter Gold-Puls um das Badge, solange der Toast steht.
    cx, cy = rect.x + pad + br, rect.centery
    glow = pygame.Surface((br * 4, br * 4), pygame.SRCALPHA)
    pygame.draw.circle(glow, (*ui.GOLD, int(50 * ui.pulse(3.0))),
                       (br * 2, br * 2), br * 2)
    surface.blit(glow, (cx - br * 2, cy - br * 2))
    draw_badge(surface, (cx, cy), br, d["icon"], accent)

    tx = rect.x + pad + br * 2 + gap
    ty = rect.centery - (head.get_height() + name.get_height() + 2) // 2
    surface.blit(head, (tx, ty))
    surface.blit(name, (tx, ty + head.get_height() + 2))


def display_name_toast_header():
    from i18n import t
    return t("ach.unlocked")


# ----- Badge-Zeichnung (auch vom Erfolge-Screen genutzt) ------------------

def draw_badge(surface, center, radius, icon, color, locked=False):
    """Rundes Erfolgs-Abzeichen mit kleinem Symbol.

    locked=True zeichnet die gesperrte Variante (gedimmt, Schloss-Symbol).
    """
    cx, cy = int(center[0]), int(center[1])
    r = int(radius)
    if locked:
        fill = ui.mix(ui.PANEL, ui.PANEL_LIGHT, 0.5)
        ring = ui.BORDER_LIGHT
        icon_col = ui.TEXT_FAINT
        icon = "lock"
    else:
        fill = ui.mix(ui.PANEL_LIGHT, color, 0.28)
        ring = color
        icon_col = ui.mix(color, (255, 255, 255), 0.35)
    pygame.draw.circle(surface, fill, (cx, cy), r)
    pygame.draw.circle(surface, ring, (cx, cy), r, 2)
    _draw_icon(surface, cx, cy, max(4, int(r * 0.62)), icon, icon_col)


def _draw_icon(s, cx, cy, r, icon, col):
    """Kleine, prozedural gezeichnete Symbole (kein Font nötig)."""
    import math
    if icon == "star":
        pts = []
        for i in range(10):
            rad = r if i % 2 == 0 else r * 0.45
            a = -math.pi / 2 + i * math.pi / 5
            pts.append((cx + rad * math.cos(a), cy + rad * math.sin(a)))
        pygame.draw.polygon(s, col, pts)
    elif icon == "trophy":
        cw, chh = int(r * 1.2), int(r * 0.9)          # Becher
        cup = pygame.Rect(cx - cw // 2, cy - r + 1, cw, chh)
        pygame.draw.rect(s, col, cup, border_radius=3)
        for side in (-1, 1):                          # Henkel
            hx = cx + side * (cw // 2 + 2)
            pygame.draw.circle(s, col, (hx, cup.centery - 1), 3, 1)
        pygame.draw.rect(s, col, (cx - 2, cup.bottom, 4, max(2, r // 3)))
        pygame.draw.rect(s, col, (cx - r // 2 - 1, cy + r - max(3, r // 3),
                                  r + 2, max(3, r // 3)), border_radius=1)
    elif icon == "clock":
        pygame.draw.circle(s, col, (cx, cy), r, 2)
        pygame.draw.line(s, col, (cx, cy), (cx, cy - r + 3), 2)
        pygame.draw.line(s, col, (cx, cy), (cx + r - 4, cy + 2), 2)
    elif icon == "medal":
        pygame.draw.polygon(s, col, [(cx - r + 2, cy - r), (cx - 1, cy),
                                     (cx - r + 5, cy)])
        pygame.draw.polygon(s, col, [(cx + r - 2, cy - r), (cx + 1, cy),
                                     (cx + r - 5, cy)])
        pygame.draw.circle(s, col, (cx, cy + r // 3), max(3, int(r * 0.55)))
    elif icon == "flag":
        pygame.draw.line(s, col, (cx - r + 3, cy - r), (cx - r + 3, cy + r), 2)
        pygame.draw.polygon(s, col, [(cx - r + 4, cy - r),
                                     (cx + r - 1, cy - r // 2),
                                     (cx - r + 4, cy)])
    elif icon == "crown":
        base = cy + r // 2
        pygame.draw.polygon(s, col, [(cx - r, base), (cx - r, cy - r // 3),
                                     (cx - r // 2, cy), (cx, cy - r),
                                     (cx + r // 2, cy), (cx + r, cy - r // 3),
                                     (cx + r, base)])
    elif icon == "gem":
        pygame.draw.polygon(s, col, [(cx, cy - r), (cx + r, cy),
                                     (cx, cy + r), (cx - r, cy)])
        pygame.draw.polygon(s, ui.mix(col, (255, 255, 255), 0.4),
                            [(cx, cy - r), (cx + r // 2, cy - r // 2),
                             (cx, cy), (cx - r // 2, cy - r // 2)])
    elif icon == "heart":
        pygame.draw.circle(s, col, (cx - r // 2, cy - r // 3), r // 2 + 1)
        pygame.draw.circle(s, col, (cx + r // 2, cy - r // 3), r // 2 + 1)
        pygame.draw.polygon(s, col, [(cx - r, cy - r // 4), (cx + r, cy - r // 4),
                                     (cx, cy + r)])
    elif icon == "check":
        pygame.draw.lines(s, col, False,
                          [(cx - r + 2, cy), (cx - r // 4, cy + r - 3),
                           (cx + r - 1, cy - r + 3)], 3)
    else:  # "lock"
        body = pygame.Rect(cx - r + 2, cy - 1, 2 * r - 4, r + 1)
        pygame.draw.rect(s, col, body, border_radius=2)
        pygame.draw.circle(s, col, (cx, cy - r // 3 - 1), r // 2 + 1, 2)
