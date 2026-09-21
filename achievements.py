# -*- coding: utf-8 -*-
"""
achievements.py
===============
Erfolge (Achievements) für die Spielesammlung.

Fünf Kategorien:

- ``general``   : sammlungsweite Erfolge aus den Statistiken (stats.py),
                  z.B. "10 Partien", "5 verschiedene Spiele", "1 h Spielzeit",
                  "7 Tage in Folge", plus einige Ereignis-Erfolge (Nachteule,
                  Wiki geöffnet ...).
- ``milestone`` : ein Punkte-Meilenstein je Spiel ("Erreiche {n} Punkte") -
                  Name/Akzentfarbe kommen vom Spiel selbst, die Schwelle steht
                  in ``MILESTONES`` (Schlüssel = highscore_key).
- ``master``    : die zweite, deutlich höhere Punktstufe je Spiel
                  ("{Spiel}-Meister", Schwellen in ``MASTERS``).
- ``regular``   : "{Spiel}-Stammgast" - ``REGULAR_PLAYS`` Partien in diesem
                  Spiel; rein aus der Statistik, ohne Eingriff in die Spiele.
- ``special``   : besondere Einzelmomente, die die Spiele selbst über
                  ``Game.ach_event("...")`` melden (Schachmatt, KNIFFEL,
                  2048-Kachel, Wordle in 2 Versuchen ...).

Jeder Erfolg hat eine **Seltenheitsstufe** (``RARITY``) und damit einen
Punktwert. Die Summe der Punkte ergibt das **Level** des Profils (levels.py);
Punkte, Verlauf und Freischalt-Zeitpunkte stehen in ``stats.pgzlamastats``.

Freigeschaltete Erfolge liegen im Abschnitt ``achievements`` der Datei
``mem.json`` - seit dem Profil-Update je Profil getrennt - als
``{id: "YYYY-MM-DD HH:MM"}``. Ein Unlock zeigt einen Toast oben rechts
(gezeichnet von main.py über ``draw_toasts``, damit er auch mitten im Spiel
über allem liegt) und spielt ein kurzes Arpeggio; ein Level-Aufstieg bekommt
einen eigenen Toast mit Fanfare.

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

# ----- Seltenheitsstufen --------------------------------------------------
#
# Punktwert und Farbe je Stufe. Die Farbe färbt Abzeichen-Ring, Erfolgszeile
# und Toast; die Punkte zahlen auf das Level ein (levels.py).
RARITY = {
    "common":    {"points": 10,  "color": "#9aa6bf"},
    "rare":      {"points": 25,  "color": "#589cff"},
    "epic":      {"points": 60,  "color": "#b07fe8"},
    "legendary": {"points": 150, "color": "#f5cd64"},
    "mythic":    {"points": 300, "color": "#e05a7d"},
}
# Reihenfolge von häufig nach selten (Filter/Sortierung im Fortschritt-Screen).
RARITY_ORDER = ("common", "rare", "epic", "legendary", "mythic")

# Partien für den "Stammgast"-Erfolg je Spiel.
REGULAR_PLAYS = 25

# ----- Definitionen -------------------------------------------------------
#
# Sammlungsweite Erfolge: (id, icon, stat, target, rarity).
#   stat   : Feld aus stats.totals() ("plays"/"distinct"/"time"/"records"/
#            "wins"/"win_games"/"days"/"best_streak"/"duels"/"level"/"ach")
#            oder None für reine Ereignis-Erfolge (event()).
#   target : Schwelle (Sekunden bei "time"); None bei Ereignis-Erfolgen.
# Die Anzeige-Reihenfolge entspricht dieser Liste.
GENERAL = [
    ("first_game",  "star",   "plays",    1,     "common"),
    ("plays_10",    "flag",   "plays",    10,    "common"),
    ("plays_50",    "flag",   "plays",    50,    "rare"),
    ("plays_200",   "crown",  "plays",    200,   "epic"),
    ("plays_500",   "crown",  "plays",    500,   "epic"),
    ("plays_1000",  "crown",  "plays",    1000,  "legendary"),
    ("explorer_5",  "gem",    "distinct", 5,     "common"),
    ("explorer_15", "gem",    "distinct", 15,    "rare"),
    ("explorer_25", "gem",    "distinct", 25,    "rare"),
    ("explorer_35", "gem",    "distinct", 35,    "epic"),
    ("explorer_all", "crown", "distinct", None,  "legendary"),  # Ziel = alle Spiele
    ("time_1h",     "clock",  "time",     3600,   "common"),
    ("time_5h",     "clock",  "time",     18000,  "rare"),
    ("time_20h",    "crown",  "time",     72000,  "epic"),
    ("time_50h",    "clock",  "time",     180000, "epic"),
    ("time_100h",   "crown",  "time",     360000, "legendary"),
    ("records_1",   "trophy", "records",  1,     "common"),
    ("records_10",  "trophy", "records",  10,    "rare"),
    ("records_25",  "crown",  "records",  25,    "epic"),
    ("records_50",  "trophy", "records",  50,    "epic"),
    ("records_100", "crown",  "records",  100,   "legendary"),
    ("wins_10",     "medal",  "wins",     10,    "common"),
    ("wins_50",     "medal",  "wins",     50,    "rare"),
    ("wins_100",    "medal",  "wins",     100,   "epic"),
    ("wins_250",    "crown",  "wins",     250,   "legendary"),
    ("victor_5",    "medal",  "win_games", 5,    "rare"),
    ("victor_10",   "medal",  "win_games", 10,   "rare"),
    ("victor_20",   "crown",  "win_games", 20,   "epic"),
    ("days_7",      "calendar", "days",   7,     "common"),
    ("days_30",     "calendar", "days",   30,    "rare"),
    ("days_100",    "calendar", "days",   100,   "legendary"),
    ("streak_3",    "flame",  "best_streak", 3,  "common"),
    ("streak_7",    "flame",  "best_streak", 7,  "rare"),
    ("streak_14",   "flame",  "best_streak", 14, "epic"),
    ("streak_30",   "flame",  "best_streak", 30, "legendary"),
    ("level_5",     "bolt",   "level",    5,     "common"),
    ("level_10",    "bolt",   "level",    10,    "rare"),
    ("level_25",    "bolt",   "level",    25,    "epic"),
    ("level_50",    "bolt",   "level",    50,    "mythic"),
    ("ach_50",      "star",   "ach",      50,    "rare"),
    ("ach_150",     "crown",  "ach",      150,   "epic"),
    ("duels_10",    "swords", "duels",    10,    "rare"),
    ("duels_50",    "swords", "duels",    50,    "epic"),
    ("night_owl",   "moon",   None,       None,  "common"),
    ("early_bird",  "star",   None,       None,  "common"),
    ("polyglot",    "heart",  None,       None,  "common"),
    ("wiki_reader", "heart",  None,       None,  "common"),
    ("painter",     "heart",  None,       None,  "common"),
    ("replay_first", "star",  None,       None,  "common"),
    ("replay_5",    "gem",    None,       5,     "rare"),
    ("replay_share", "heart", None,       None,  "common"),
    ("marathon",    "clock",  None,       3600,  "rare"),
    ("weekend",     "calendar", None,     None,  "common"),
    ("profile_second", "person", None,    2,     "common"),
    ("profile_family", "person", None,    4,     "rare"),
    ("statistician", "chart",  None,      None,  "common"),
    ("wrapped",     "chart",  None,       None,  "common"),
    ("first_legend", "crown",  None,      None,  "epic"),
    ("all_rarities", "gem",    None,      None,  "rare"),
    ("completionist", "rocket", None,     None,  "legendary"),
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
    "days": "ach.days.desc",
    "best_streak": "ach.streak.desc",
    "level": "ach.level.desc",
    "ach": "ach.collect.desc",
    "duels": "ach.duels.desc",
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
    "blackjack": 1600,     # Blackjack-Bilanz: Start-Chips (1000) um 600 vermehrt
    "tunnel": 15000,       # lange Endlos-Fahrt
    "maze": 1500,          # ~2 Level mit Orbs
    "kniffel": 200,        # Gesamtsumme mit Bonus
    "wordle": 150,         # ~3-4 Wörter in Folge
    "trex": 600,           # lange Strecke
    "poker": 1500,         # Poker-Bilanz: Start-Chips (1000) ver-1,5-facht
    "simon": 12,           # 12er-Sequenz
    "slide": 4000,         # 4x4 flott gelöst
    "mastermind": 150,     # mehrere Codes in Folge
    "bubble": 1500,        # großes Feld geräumt
    "hangman": 150,        # mehrere Wörter in Folge
    "blockjump": 3000,     # einige Level geschafft
    "lamatowerdef": 10000,     # ~Welle 18-20 auf der Wiese (Klassisch)
    "minigolf": 5000,      # eine Runde deutlich unter Par gespielt
    "pinball": 25000,      # solide Partie mit Multiplikator
    "bowling": 120,        # ordentliches Spiel mit ein paar Spares
    "crossy": 150,         # 150 Reihen weit gehüpft
    "geodash": 20,         # 20 Sterne (Level-Sterne + Münzen)
    "casino": 5000,        # Casino-Bilanz auf 5000 Chips gebracht
}

# Zweite Punktstufe ("{Spiel}-Meister"). Nur für Spiele, deren Punktzahl
# nach oben offen ist bzw. sich über mehrere Runden aufbaut - bei Spielen mit
# fester Rundenobergrenze (Pong, Air Hockey, Memory, Minesweeper,
# Schiebepuzzle, Solitär, Sudoku) gäbe es sonst ein unerreichbares Ziel.
MASTERS = {
    "snake": 2000,
    "breakout": 20000,
    "tetris": 30000,
    "invaders": 8000,
    "asteroids": 12000,
    "pacman": 30000,
    "flappy": 60,
    "doodle": 12000,
    "2048": 30000,
    "frogger": 8000,
    "aim": 8000,
    "blackjack": 3000,
    "tunnel": 40000,
    "maze": 4000,
    "kniffel": 300,
    "wordle": 500,
    "trex": 1500,
    "poker": 4000,
    "simon": 20,
    "mastermind": 400,
    "bubble": 4000,
    "hangman": 400,
    "blockjump": 8000,
    "lamatowerdef": 25000,
    "minigolf": 10000,
    "pinball": 75000,
    "bowling": 180,
    "crossy": 400,
    "geodash": 45,
    "casino": 15000,
}

# Besondere Momente: (id, icon, highscore_key, target, rarity).
#   highscore_key: verknüpft den Erfolg optisch mit einem Spiel (Farbe/Name);
#                  None = spielübergreifend.
#   target : None = event(id) schaltet direkt frei; Zahl = event(id, wert)
#            schaltet erst ab wert >= target frei (z.B. Poker-Chips).
SPECIALS = [
    ("chess_win",      "crown", "chess",       None,  "rare"),      # KI matt gesetzt
    ("tile_2048",      "gem",   "2048",        None,  "rare"),      # 2048-Kachel
    ("tetris_four",    "star",  "tetris",      None,  "rare"),      # 4 Reihen auf einmal
    ("kniffel_five",   "star",  "kniffel",     None,  "rare"),      # Kniffel gewürfelt
    ("blackjack_two",  "medal", "blackjack",   None,  "common"),    # Blackjack mit 2 Karten
    ("poker_rich",     "medal", "poker",       2000,  "epic"),      # Poker-Bilanz verdoppelt
    ("wordle_two",     "gem",   "wordle",      None,  "epic"),      # in <= 2 Versuchen
    ("hangman_clean",  "check", "hangman",     None,  "rare"),      # ohne Fehlversuch
    ("sudoku_clean",   "check", "sudoku",      None,  "rare"),      # fehlerfrei gelöst
    ("mine_win",       "flag",  "minesweeper", None,  "common"),    # Feld geräumt
    ("solitaire_win",  "check", "solitaire",   None,  "common"),    # Partie gewonnen
    ("memory_perfect", "heart", "memory",      None,  "epic"),      # ohne Fehlpaar
    ("pacman_clear",   "flag",  "pacman",      None,  "epic"),      # Labyrinth leergefuttert
    ("breakout_clear", "flag",  "breakout",    None,  "epic"),      # alle Level geschafft
    ("frog_home",      "flag",  "frogger",     None,  "rare"),      # alle 5 Buchten voll
    ("aim_perfect",    "star",  "aim",         None,  "epic"),      # 100% Genauigkeit
    ("snake_prestige", "gem",   "snake",       None,  "epic"),      # Prestige I erreicht
    ("snake_comp5",    "crown", "snake",       5,     "epic"),      # Competitive Level 5
    ("td_boss",        "crown", "lamatowerdef", None, "rare"),      # ersten Boss besiegt
    ("td_wave20",      "flag",  "lamatowerdef", None, "epic"),      # Welle 20 geschafft
    ("td_perfect10",   "heart", "lamatowerdef", None, "epic"),      # Welle 10 ohne Verlust
    ("td_maxed",       "gem",   "lamatowerdef", None, "rare"),      # Turm voll ausgebaut
    ("golf_ace",       "star",  "minigolf",    None,  "epic"),      # Hole-in-One
    ("golf_under_par", "check", "minigolf",    None,  "rare"),      # Runde unter Par
    ("pin_multiball",  "gem",   "pinball",     None,  "common"),    # Multiball gestartet
    ("pin_high",       "crown", "pinball",     50000, "epic"),      # 50.000 in einer Partie
    ("bowl_turkey",    "flag",  "bowling",     None,  "rare"),      # drei Strikes in Folge
    ("bowl_200",       "crown", "bowling",     200,   "legendary"), # 200er-Spiel
    ("crossy_char",    "gem",   "crossy",      None,  "common"),    # erste Figur gekauft
    ("crossy_train",   "flag",  "crossy",      None,  "rare"),      # 5 Gleise in Folge
    ("gd_first",       "star",  "geodash",     None,  "common"),    # erstes Level geschafft
    ("gd_coins",       "medal", "geodash",     None,  "rare"),      # alle 3 Münzen
    ("gd_demon",       "crown", "geodash",     None,  "legendary"), # Dämon-Level geschafft
    ("bs_flawless",    "heart", "battleship",  None,  "epic"),      # Sieg ohne Verlust
    ("bs_hard",        "crown", "battleship",  None,  "epic"),      # Sieg gegen schwere KI
    ("roulette_plein", "star",  "casino",      None,  "rare"),      # Volltreffer auf eine Zahl
    ("slots_jackpot",  "gem",   "casino",      None,  "legendary"), # 5 Lamas auf einer Linie
    ("tetris_tspin",   "star",  "tetris",      None,  "epic"),      # T-Spin Double
    ("tetris_pc",      "gem",   "tetris",      None,  "legendary"), # Perfect Clear
    ("tetris_sprint",  "clock", "tetris",      None,  "epic"),      # 40 Zeilen unter 2:00
    ("tile_4096",      "crown", "2048",        None,  "legendary"), # 4096er-Kachel
    ("chess_puzzles",  "check", "chess",       25,    "epic"),      # 25 Schachrätsel gelöst
    ("chess_master",   "crown", "chess",       None,  "legendary"), # Sieg gegen Stufe 6
    ("wordle_daily7",  "flag",  "wordle",      7,     "epic"),      # 7 Tageswörter in Folge
    ("wordle_quordle", "gem",   "wordle",      None,  "epic"),      # Quordle gelöst
    ("sudoku_killer",  "medal", "sudoku",      None,  "epic"),      # Killer-Sudoku gelöst
    ("sudoku_stars",   "star",  "sudoku",      30,    "epic"),      # 30 Level mit 3 Sternen
    # --- neu im Erfolgs-Update -------------------------------------------
    ("snake_long",     "gem",   "snake",       None,  "rare"),      # Länge 50 erreicht
    ("pong_shutout",   "medal", "pong",        None,  "rare"),      # zu Null gewonnen
    ("pong_rally",     "star",  "pong",        None,  "common"),    # 20 Ballwechsel
    ("air_hattrick",   "flag",  "airhockey",   None,  "rare"),      # 3 Tore in Folge
    ("ast_wave10",     "flag",  "asteroids",   None,  "rare"),      # Welle 10 erreicht
    ("inv_boss",       "crown", "invaders",    None,  "rare"),      # Boss besiegt
    ("doodle_high",    "rocket", "doodle",     None,  "epic"),      # 10.000 Höhe
    ("flappy_gold",    "medal", "flappy",      40,    "epic"),      # Gold-Medaille
    ("trex_night",     "moon",  "trex",        None,  "common"),    # Nachtstrecke erreicht
    ("tunnel_far",     "bolt",  "tunnel",      None,  "rare"),      # Höchstgeschwindigkeit
    ("c4_diagonal",    "star",  "connect4",    None,  "common"),    # diagonaler Sieg
    ("ttt_unbeaten",   "check", "tictactoe",   3,     "rare"),      # 3 Siege in Folge
    ("reversi_sweep",  "crown", "reversi",     None,  "epic"),      # Gegner ausgelöscht
    ("dame_multi",     "star",  "dame",        None,  "rare"),      # Dreifachsprung
    ("muehle_double",  "gem",   "muehle",      None,  "rare"),      # Zwickmühle gebaut
    ("tanks_bank",     "target", "tanks",      None,  "rare"),      # Treffer über die Bande
    ("billiard_break", "star",  "billiard",    None,  "rare"),      # Kugel beim Anstoß
    ("master_three",   "check", "mastermind",  None,  "rare"),      # Code in 3 Versuchen
    ("slide_swift",    "clock", "slide",       None,  "rare"),      # 4x4 unter 100 Zügen
    ("bubble_chain",   "gem",   "bubble",      None,  "rare"),      # Kette aus 10 Blasen
    ("maze_swift",     "clock", "maze",        None,  "rare"),      # Level unter 30 s
    ("bj_deathless",   "heart", "blockjump",   None,  "rare"),      # Level ohne Sturz
    ("simon_20",       "star",  "simon",       20,    "epic"),      # 20er-Sequenz
    ("frog_fly",       "flag",  "frogger",     None,  "common"),    # Fliege gefangen
    ("kniffel_grand",  "crown", "kniffel",     300,   "epic"),      # 300 Punkte im Block
    ("solitaire_fast", "clock", "solitaire",   None,  "epic"),      # Partie unter 3 Minuten
    ("pac_ghosts",     "star",  "pacman",      None,  "rare"),      # vier Geister am Stück
    ("mine_noflag",    "flag",  "minesweeper", None,  "epic"),      # Sieg ohne Flaggen
    ("memory_swift",   "clock", "memory",      None,  "rare"),      # 4x4 unter 30 s
    ("tetris_b2b",     "gem",   "tetris",      None,  "epic"),      # zwei Tetris in Folge
]

_unlocked = None      # {id: "YYYY-MM-DD HH:MM"} (In-Memory-Kopie)
_defs = None          # Definitionen in Anzeige-Reihenfolge
_by_id = None         # id -> Definition
_milestone_by_game = None   # highscore_key -> [Milestone-, Master-Definition]
_regular_by_game = None     # highscore_key -> Stammgast-Definition


# ----- Aufbau -------------------------------------------------------------

def _load_unlocked():
    global _unlocked
    if _unlocked is None:
        raw = store.load_section(_SECTION)
        _unlocked = {str(k): str(v) for k, v in raw.items()}
    return _unlocked


def reset_cache():
    """Vergisst die geladenen Freischaltungen (nach einem Profilwechsel)."""
    global _unlocked
    _unlocked = None
    _toasts.clear()


store.on_switch(reset_cache)


def _build_defs():
    """Baut die Definitionsliste auf (einmalig; braucht die Spielklassen)."""
    global _defs, _by_id, _milestone_by_game, _regular_by_game
    if _defs is not None:
        return _defs
    from games import ALL_GAMES
    by_key = {cls.highscore_key: cls for cls in ALL_GAMES}

    defs = []
    for ach_id, icon, stat, target, rarity in GENERAL:
        if ach_id == "explorer_all":
            target = len(ALL_GAMES)
        d = dict(id=ach_id, cat="general", icon=icon, stat=stat,
                 target=target, game=None, game_cls=None, rarity=rarity)
        defs.append(d)
    for cls in ALL_GAMES:
        key = cls.highscore_key
        if key in MILESTONES:
            defs.append(dict(id="score_" + key, cat="milestone", icon="star",
                             stat=None, target=MILESTONES[key], game=key,
                             game_cls=cls, rarity="rare"))
        if key in MASTERS:
            defs.append(dict(id="master_" + key, cat="master", icon="crown",
                             stat=None, target=MASTERS[key], game=key,
                             game_cls=cls, rarity="epic"))
    for cls in ALL_GAMES:
        key = cls.highscore_key
        defs.append(dict(id="regular_" + key, cat="regular", icon="flag",
                         stat=None, target=REGULAR_PLAYS, game=key,
                         game_cls=cls, rarity="common"))
    for ach_id, icon, game_key, target, rarity in SPECIALS:
        defs.append(dict(id=ach_id, cat="special", icon=icon, stat=None,
                         target=target, game=game_key,
                         game_cls=by_key.get(game_key), rarity=rarity))

    _defs = defs
    _by_id = {d["id"]: d for d in defs}
    _milestone_by_game = {}
    _regular_by_game = {}
    for d in defs:
        if d["cat"] in ("milestone", "master"):
            _milestone_by_game.setdefault(d["game"], []).append(d)
        elif d["cat"] == "regular":
            _regular_by_game[d["game"]] = d
    return _defs


# ----- Abfragen (für den Erfolge-Screen) ----------------------------------

def all_defs():
    """Alle Erfolgs-Definitionen in Anzeige-Reihenfolge."""
    return list(_build_defs())


def by_id(ach_id):
    """Definition eines Erfolgs (oder None)."""
    _build_defs()
    return _by_id.get(ach_id)


def unlocked():
    """Freigeschaltete Erfolge als dict {id: zeitstempel}."""
    return dict(_load_unlocked())


def counts():
    """(freigeschaltet, gesamt) für Fortschrittsanzeigen."""
    return len(_load_unlocked()), len(_build_defs())


def points_of(d):
    """Punktwert eines Erfolgs (aus seiner Seltenheitsstufe)."""
    if isinstance(d, str):
        d = by_id(d)
    if not d:
        return 0
    return RARITY.get(d.get("rarity", "common"), RARITY["common"])["points"]


def rarity_color(d):
    """Farbe der Seltenheitsstufe als (r, g, b)."""
    rarity = d.get("rarity", "common") if isinstance(d, dict) else str(d)
    value = RARITY.get(rarity, RARITY["common"])["color"].lstrip("#")
    return tuple(int(value[i:i + 2], 16) for i in (0, 2, 4))


def rarity_name(d):
    """Lokalisierter Name der Seltenheitsstufe."""
    from i18n import t
    rarity = d.get("rarity", "common") if isinstance(d, dict) else str(d)
    return t("rarity." + rarity)


def total_points():
    """Summe der Punkte aller freigeschalteten Erfolge."""
    _build_defs()
    unlocked_map = _load_unlocked()
    return sum(points_of(_by_id[i]) for i in unlocked_map if i in _by_id)


def max_points():
    """Punkte, die insgesamt zu holen sind."""
    return sum(points_of(d) for d in _build_defs())


def rarity_counts():
    """{stufe: (freigeschaltet, gesamt)} für die Übersicht."""
    unlocked_map = _load_unlocked()
    out = {r: [0, 0] for r in RARITY_ORDER}
    for d in _build_defs():
        slot = out.setdefault(d["rarity"], [0, 0])
        slot[1] += 1
        if d["id"] in unlocked_map:
            slot[0] += 1
    return {k: tuple(v) for k, v in out.items()}


def display_name(d):
    """Lokalisierter Name eines Erfolgs (Meilensteine = Spielname)."""
    from i18n import t
    if d["cat"] == "milestone":
        return d["game_cls"].name
    if d["cat"] == "master":
        return t("ach.master.name", game=d["game_cls"].name)
    if d["cat"] == "regular":
        return t("ach.regular.name", game=d["game_cls"].name)
    return t("ach.%s.name" % d["id"])


def display_desc(d):
    """Lokalisierte Beschreibung eines Erfolgs."""
    from i18n import t
    if d["cat"] in ("milestone", "master"):
        return t("ach.score.desc", n=d["target"])
    if d["cat"] == "regular":
        return t("ach.regular.desc", n=d["target"], game=d["game_cls"].name)
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
            totals = _totals()
        return min(totals.get(d["stat"], 0), d["target"]), d["target"]
    if d["cat"] in ("milestone", "master"):
        if scores is None:
            import highscore
            scores = highscore.load_highscores()
        return min(scores.get(d["game"], 0), d["target"]), d["target"]
    if d["cat"] == "regular":
        return min(stats.get(d["game"])["plays"], d["target"]), d["target"]
    return None


def _totals():
    """stats.totals() + die Erfolgs-Zahl (Erfolge auf Erfolge)."""
    totals = stats.totals()
    totals["ach"] = len(_load_unlocked())
    return totals


# ----- Freischalten -------------------------------------------------------

def _save_unlocked():
    store.save_section(_SECTION, dict(_load_unlocked()))


def _unlock(d, silent=False):
    """Schaltet einen Erfolg frei (einmalig); Toast + Sound, wenn nicht silent."""
    unlocked_map = _load_unlocked()
    if d["id"] in unlocked_map:
        return False
    stamp = time.strftime("%Y-%m-%d %H:%M")
    unlocked_map[d["id"]] = stamp
    _save_unlocked()

    # Punkte/Verlauf in der Statistik nachziehen und auf Level-Aufstieg prüfen.
    import levels
    before = stats.level_of()[0]
    stats.log_ach(d["id"], points_of(d), stamp)
    stats.set_points(total_points())
    after = stats.level_of()[0]

    if not silent:
        _toasts.append({"kind": "ach", "def": d, "t": 0.0, "notes": []})
        if d["rarity"] in ("legendary", "mythic"):
            # Seltene Stücke bekommen Konfetti - das darf man feiern.
            try:
                ui.spawn_confetti(800, 600, n=60)
            except Exception:
                pass
    if after > before:
        stats.log_level(after, stamp)
        if not silent:
            _toasts.append({"kind": "level", "level": after,
                            "title": levels.rank_title(after),
                            "color": levels.rank_color(after),
                            "t": 0.0, "notes": []})
    _check_meta(silent)
    return True


def _check_meta(silent=False):
    """Erfolge über Erfolge (Sammler, erste Legende, alle Stufen ...)."""
    unlocked_map = _load_unlocked()
    _build_defs()
    # Erste Legende bzw. ein Erfolg jeder Seltenheitsstufe.
    seen = set()
    for ach_id in unlocked_map:
        d = _by_id.get(ach_id)
        if d:
            seen.add(d["rarity"])
    if "legendary" in seen or "mythic" in seen:
        _unlock_id("first_legend", silent)
    if len(seen & set(RARITY_ORDER)) >= 4:
        _unlock_id("all_rarities", silent)
    # Alle Erfolge eines Spiels eingesammelt?
    per_game = {}
    for d in _defs:
        if d["game"]:
            slot = per_game.setdefault(d["game"], [0, 0])
            slot[1] += 1
            if d["id"] in unlocked_map:
                slot[0] += 1
    if any(got == total and total >= 3 for got, total in per_game.values()):
        _unlock_id("completionist", silent)
    # Zählbare Erfolgs-Erfolge (50/150 Erfolge).
    totals = _totals()
    for d in _defs:
        if d["cat"] == "general" and d["stat"] == "ach" and d["target"]:
            if totals["ach"] >= d["target"]:
                _unlock(d, silent)


def _unlock_id(ach_id, silent=False):
    _build_defs()
    d = _by_id.get(ach_id)
    if d is not None and d["id"] not in _load_unlocked():
        _unlock(d, silent)


def check_stats(silent=False):
    """Prüft alle statistik-basierten Erfolge (nach Partien/Siegen/Zeit)."""
    totals = _totals()
    for d in _build_defs():
        if d["cat"] == "general" and d["stat"] and d["target"]:
            if totals.get(d["stat"], 0) >= d["target"]:
                _unlock(d, silent)
        elif d["cat"] == "regular":
            if stats.get(d["game"])["plays"] >= d["target"]:
                _unlock(d, silent)


def on_highscore(key, score, silent=False):
    """Prüft die Punkte-Stufen eines Spiels (nach Highscore-Update)."""
    _build_defs()
    for d in _milestone_by_game.get(key, ()):
        if score >= d["target"]:
            _unlock(d, silent)


def on_game_started(key):
    """Wird bei jedem Partie-Start gerufen (auch Neustart nach Game Over)."""
    hour = time.localtime().tm_hour
    if 0 <= hour < 5:
        event("night_owl")
    elif 5 <= hour < 8:
        event("early_bird")
    _check_weekend()
    check_stats()


def _check_weekend():
    """"Wochenendspieler": am selben Wochenende Samstag UND Sonntag gespielt."""
    if "weekend" in _load_unlocked():
        return
    import datetime
    days = stats.days_map()
    for day in days:
        try:
            date = datetime.date.fromisoformat(day)
        except ValueError:
            continue
        if date.weekday() == 5 and (date + datetime.timedelta(days=1)).isoformat() in days:
            event("weekend")
            return


def on_playtime(seconds):
    """Marathon-Erfolg: eine Sitzung von einer Stunde am Stück."""
    if seconds >= 3600:
        event("marathon", seconds)


def event(ach_id, value=None):
    """Löst einen Ereignis-Erfolg aus (unbekannte ids werden ignoriert).

    Erfolge mit Ziel-Wert (z.B. Poker-Chips) schalten erst frei, wenn
    ``value`` das Ziel erreicht.
    """
    _build_defs()
    d = _by_id.get(ach_id)
    if d is None:
        return
    if d["target"] is not None and d["stat"] is None \
            and d["cat"] not in ("milestone", "master", "regular"):
        if value is None or value < d["target"]:
            return
    _unlock(d)


def backfill_silent():
    """Beim App-Start: Erreichtes aus Bestandsdaten OHNE Toast freischalten.

    Gibt (neue_erfolge, level, punkte) zurück - main.py zeigt daraus beim
    ersten Start nach dem Update die Karte "Dein Startlevel".
    """
    before = len(_load_unlocked())
    try:
        import highscore
        scores = highscore.load_highscores()
        check_stats(silent=True)
        for key, score in scores.items():
            on_highscore(key, score, silent=True)
        _check_meta(silent=True)
        # Punktestand IMMER neu aus den freigeschalteten Erfolgen bilden -
        # so stimmt er auch nach einem Update mit neuen Punktwerten.
        stats.set_points(total_points())
    except Exception:
        # Erfolge dürfen den Start nie verhindern.
        pass
    level, points = stats.level_of()
    return len(_load_unlocked()) - before, level, points


def rarest_unlocked(n=3):
    """Die n seltensten freigeschalteten Erfolge (für die Startkarte)."""
    _build_defs()
    unlocked_map = _load_unlocked()
    items = [(_by_id[i], ts) for i, ts in unlocked_map.items() if i in _by_id]
    items.sort(key=lambda it: (-RARITY_ORDER.index(it[0]["rarity"]), it[1]))
    return items[:n]


# ----- Toast-Einblendung (oben rechts, über allem) ------------------------

_toasts = []                       # wartende/aktive Einblendungen (FIFO)
_TOAST_IN, _TOAST_HOLD, _TOAST_OUT = 0.35, 3.4, 0.45
# Kleines Aufstiegs-Arpeggio beim Erscheinen (Zeitmarke, Frequenz).
_ARPEGGIO = ((0.00, 660), (0.13, 880), (0.26, 1318))
# Fanfare beim Level-Aufstieg (länger, mit Quinte obendrauf).
_FANFARE = ((0.00, 523), (0.12, 659), (0.24, 784), (0.36, 1047), (0.52, 1319))


def _ease_out(x):
    x = max(0.0, min(1.0, x))
    return 1.0 - (1.0 - x) ** 3


def draw_toasts(surface, w, h, dt, settings=None):
    """Zeichnet die aktive Einblendung (von main.py pro Frame gerufen)."""
    if not _toasts:
        return
    toast = _toasts[0]
    toast["t"] += dt
    tt = toast["t"]
    level_toast = toast.get("kind") == "level"

    # Töne zu ihren Zeitmarken abspielen (je genau einmal).
    notes = _FANFARE if level_toast else _ARPEGGIO
    for i, (at, freq) in enumerate(notes):
        if tt >= at and i not in toast["notes"]:
            toast["notes"].append(i)
            audio.tone(freq, 0.24 if level_toast else 0.22, settings,
                       wave="square", vol=0.30)

    hold = _TOAST_HOLD + (0.8 if level_toast else 0.0)
    total = _TOAST_IN + hold + _TOAST_OUT
    if tt >= total:
        _toasts.pop(0)
        return

    from i18n import t as _t
    if level_toast:
        accent = toast["color"]
        head_text = _t("level.up")
        name_text = _t("level.toast", n=toast["level"], rank=toast["title"])
        icon = "bolt"
        sub_text = ""
    else:
        d = toast["def"]
        accent = rarity_color(d)
        head_text = _t("ach.unlocked")
        name_text = display_name(d)
        icon = d["icon"]
        sub_text = "%s  ·  +%d" % (rarity_name(d), points_of(d))

    head_font = ui.font(12, bold=True)
    name_font = ui.font(17, bold=True)
    sub_font = ui.font(11, bold=True)
    head = head_font.render(head_text, True, ui.GOLD if not level_toast else accent)
    name = name_font.render(name_text, True, ui.TEXT)
    sub = sub_font.render(sub_text, True, accent) if sub_text else None

    br = 17                                   # Badge-Radius
    pad, gap = 14, 10
    text_w = max(head.get_width(), name.get_width(),
                 sub.get_width() if sub else 0)
    bw = min(w - 24, pad * 2 + br * 2 + gap + text_w)
    text_h = head.get_height() + name.get_height() + (
        sub.get_height() + 2 if sub else 0)
    bh = max(br * 2 + 16, text_h + 22)

    # Von rechts einschieben, am Ende wieder hinausschieben.
    if tt < _TOAST_IN:
        off = (1.0 - _ease_out(tt / _TOAST_IN)) * (bw + 24)
    elif tt > _TOAST_IN + hold:
        off = _ease_out((tt - _TOAST_IN - hold) / _TOAST_OUT) * (bw + 24)
    else:
        off = 0.0

    rect = pygame.Rect(int(w - 12 - bw + off), 12, bw, bh)
    ui.draw_panel(surface, rect, color=ui.PANEL_LIGHT, radius=12,
                  accent_top=accent)
    # Dezenter Puls in der Seltenheits-/Rangfarbe um das Badge.
    cx, cy = rect.x + pad + br, rect.centery
    glow = pygame.Surface((br * 4, br * 4), pygame.SRCALPHA)
    pygame.draw.circle(glow, (*accent, int(50 * ui.pulse(3.0))),
                       (br * 2, br * 2), br * 2)
    surface.blit(glow, (cx - br * 2, cy - br * 2))
    draw_badge(surface, (cx, cy), br, icon, accent)

    tx = rect.x + pad + br * 2 + gap
    ty = rect.centery - text_h // 2
    surface.blit(head, (tx, ty))
    ty += head.get_height() + 2
    surface.blit(name, (tx, ty))
    if sub:
        surface.blit(sub, (tx, ty + name.get_height()))


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
    elif icon == "llama":
        # Lama-Kopf: Hals, Kopf, zwei Ohren, ein Auge.
        pygame.draw.rect(s, col, (cx - r // 3, cy - r // 4, max(3, r // 2),
                                  r + r // 3), border_radius=2)
        head = pygame.Rect(cx - r, cy - r, int(r * 1.1), int(r * 0.8))
        pygame.draw.rect(s, col, head, border_radius=3)
        for dx in (-r + 2, -r // 3):
            pygame.draw.polygon(s, col, [(cx + dx, cy - r),
                                         (cx + dx + 3, cy - r - r // 2),
                                         (cx + dx + 5, cy - r)])
        pygame.draw.circle(s, ui.mix(col, (0, 0, 0), 0.6),
                           (head.centerx - 1, head.centery), max(1, r // 6))
    elif icon == "person":
        pygame.draw.circle(s, col, (cx, cy - r // 2), max(3, r // 2))
        pygame.draw.polygon(s, col, [(cx - r, cy + r), (cx - r + 2, cy + 1),
                                     (cx + r - 2, cy + 1), (cx + r, cy + r)])
    elif icon == "flame":
        pygame.draw.polygon(s, col, [(cx, cy - r), (cx + r, cy + r // 3),
                                     (cx + r // 2, cy + r), (cx - r // 2, cy + r),
                                     (cx - r, cy + r // 3)])
        pygame.draw.polygon(s, ui.mix(col, (255, 255, 255), 0.45),
                            [(cx, cy - r // 4), (cx + r // 2, cy + r // 2),
                             (cx - r // 2, cy + r // 2)])
    elif icon == "calendar":
        box = pygame.Rect(cx - r, cy - r + 2, 2 * r, 2 * r - 2)
        pygame.draw.rect(s, col, box, 2, border_radius=2)
        pygame.draw.line(s, col, (box.x, box.y + 5), (box.right, box.y + 5), 2)
        for i in range(2):
            pygame.draw.line(s, col, (cx - r // 2 + i * r, cy - r),
                             (cx - r // 2 + i * r, cy - r + 4), 2)
        pygame.draw.rect(s, col, (cx - 1, cy + 1, 3, 3))
    elif icon == "dice":
        box = pygame.Rect(cx - r, cy - r, 2 * r, 2 * r)
        pygame.draw.rect(s, col, box, 2, border_radius=3)
        for dx, dy in ((-r // 2, -r // 2), (r // 2, r // 2), (0, 0)):
            pygame.draw.circle(s, col, (cx + dx, cy + dy), max(1, r // 5))
    elif icon == "bolt":
        pygame.draw.polygon(s, col, [(cx + r // 3, cy - r), (cx - r, cy + 1),
                                     (cx, cy + 1), (cx - r // 3, cy + r),
                                     (cx + r, cy - 1), (cx, cy - 1)])
    elif icon == "moon":
        pygame.draw.circle(s, col, (cx, cy), r)
        pygame.draw.circle(s, ui.PANEL_LIGHT, (cx + r // 2, cy - r // 3), r)
    elif icon == "target":
        pygame.draw.circle(s, col, (cx, cy), r, 2)
        pygame.draw.circle(s, col, (cx, cy), max(2, r // 2), 2)
        pygame.draw.circle(s, col, (cx, cy), max(1, r // 5))
    elif icon == "swords":
        pygame.draw.line(s, col, (cx - r, cy + r), (cx + r, cy - r), 2)
        pygame.draw.line(s, col, (cx + r, cy + r), (cx - r, cy - r), 2)
        pygame.draw.circle(s, col, (cx - r, cy + r), max(2, r // 4))
        pygame.draw.circle(s, col, (cx + r, cy + r), max(2, r // 4))
    elif icon == "chart":
        base = cy + r
        for i, height in enumerate((0.45, 0.8, 0.6, 1.0)):
            bx = cx - r + i * (2 * r // 4) + 1
            bh = int(2 * r * height * 0.75)
            pygame.draw.rect(s, col, (bx, base - bh, max(2, r // 3), bh),
                             border_radius=1)
    elif icon == "rocket":
        pygame.draw.polygon(s, col, [(cx, cy - r), (cx + r // 2, cy),
                                     (cx + r // 2, cy + r // 2),
                                     (cx - r // 2, cy + r // 2),
                                     (cx - r // 2, cy)])
        pygame.draw.polygon(s, col, [(cx - r // 2, cy), (cx - r, cy + r // 2),
                                     (cx - r // 2, cy + r // 2)])
        pygame.draw.polygon(s, col, [(cx + r // 2, cy), (cx + r, cy + r // 2),
                                     (cx + r // 2, cy + r // 2)])
        pygame.draw.circle(s, ui.mix(col, (0, 0, 0), 0.5), (cx, cy - r // 4),
                           max(1, r // 4))
    else:  # "lock"
        body = pygame.Rect(cx - r + 2, cy - 1, 2 * r - 4, r + 1)
        pygame.draw.rect(s, col, body, border_radius=2)
        pygame.draw.circle(s, col, (cx, cy - r // 3 - 1), r // 2 + 1, 2)
