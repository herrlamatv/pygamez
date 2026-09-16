# -*- coding: utf-8 -*-
"""Headless-Audit für Wordle (games/wordle.py, games/wordle_words.py,
woordlistz/ und web/js/games/wordle*.js).

Geprüft wird:

Logik      (1) evaluate für Wortlängen 4-7 inkl. Doppelbuchstaben - feste Fälle
               und Zufallsvergleich mit einer unabhängigen Referenz,
           (2) harter Modus (hard_problem) je Länge, auch mit doppelten
               Buchstaben,
Listen     (3) je Sprache x Länge: Dateien vorhanden, genug Lösungswörter,
               nur A-Z in genau N Zeichen, sortiert, ohne Dubletten, Lösungen
               in den Rateworten, keine gesperrten Wörter (Namen/Sex/Drogen);
               Englisch/5 = Original-NYT-Liste; Web-Dateien = Textdateien,
Tageswort  (4) deterministisch, je Sprache/Länge verschieden, keine
               Wiederholung, bevor die Liste durch ist; Python == JS (Node),
Spiel      (5) Endlos: Punkte, Highscore nur bei 5 Buchstaben, Bestwerte je
               Länge, wordle_two; Abbruch über das Setup sichert die Serie,
           (6) Tageswort: Fortschritt überlebt einen Neustart, fertig = nur
               Ergebnis; Serie über Tage, Lücke setzt zurück, wordle_daily7,
           (7) Dordle/Quordle: Bretter einzeln gelöst, 7/9 Versuche,
               Tastatur-Zustände je Brett, wordle_quordle, Niederlage,
           (8) Statistik (Verteilung, Quote, Serien) + Python == JS,
           (9) Teilen: Emoji-Raster für 1/2/4 Bretter, Farbenblind, harter
               Modus, Python == JS; Zwischenablage-Rundlauf über Tk,
          (10) Setup per Tastatur/Maus, Einstellungen gespeichert, alte
               Modi ("hard"/"normal") und kaputte Einstellungen,
               Tastaturbelegung je Sprache (AZERTY für Französisch),
               Auflösungswechsel mitten im Spiel,
Layout    (11) 5 Auflösungen x Längen x Modi: alles im Bild, nichts überlappt;
               14 Sprachen: Texte passen (Setup, Spiel, Meldungen, Ergebnis),
               UI v4.2 und UI v1,
Tempo     (12) Quordle mit 7 Buchstaben bei 1280x960 während der Aufdeckung.

Aufruf aus dem Repo-Root:  python tests/audit_wordle.py
Exit-Code 0 = alle Prüfungen bestanden.
"""
import json
import os
import random
import re
import shutil
import subprocess
import sys
import tempfile
import time

os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
try:
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
except (AttributeError, OSError):
    pass

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, REPO)

import pygame
pygame.init()
pygame.display.set_mode((640, 480))

import store
# Testläufe dürfen weder mem.json noch settings.json anfassen
store._PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                           "_audit-wordle-mem.json")
import settings as settings_mod
SAVED_SETTINGS = []
settings_mod.save_settings = lambda s: SAVED_SETTINGS.append(json.loads(json.dumps(s)))
import i18n
i18n.init()
import seedrand
import ui
from game_base import InputEvent

from games import wordle as wd
from games import wordle_words as ww

sys.path.insert(0, os.path.join(REPO, "woordlistz"))
sys.dont_write_bytecode = True         # kein __pycache__ in woordlistz/ (landet in der .exe)
import build_wordlists as bw          # nur die Sperrlisten - lädt nichts herunter
sys.dont_write_bytecode = False

# Die echte Zwischenablage bleibt in Ruhe (eigener Rundlauf-Test in (9)).
_REAL_WIN_CLIPBOARD = wd._win_clipboard
wd._win_clipboard = lambda text: False

FAILS = []
RESOLUTIONS = ((480, 360), (640, 480), (800, 600), (960, 720), (1280, 960))
LANGS = [code for code, _name in i18n.AVAILABLE]


def check(ok, label, detail=""):
    print("  %-4s %s%s" % ("OK" if ok else "FAIL", label,
                           ("  -> " + str(detail)) if detail and not ok else ""))
    if not ok:
        FAILS.append(label)
    return ok


def quiet(game):
    game.events = []
    game.play_sound = lambda name: None
    game.rumble = lambda ms=0: None
    game.ach_event = lambda i, v=None: game.events.append(("ach", i, v))
    game.report_result = lambda won: game.events.append(("result", won))
    return game


def wipe_store():
    for p in (store._PATH, store._PATH + ".bak"):
        try:
            os.remove(p)
        except OSError:
            pass


def settings_for(length=5, hard=False, colorblind=False):
    gs = json.loads(json.dumps(settings_mod.DEFAULTS))
    gs["wordle"] = {"length": length, "hard": hard, "colorblind": colorblind}
    return gs


def make(mode="endless", w=640, h=480, length=5, hard=False, colorblind=False, gs=None):
    gs = gs or settings_for(length, hard, colorblind)
    return quiet(wd.WordleGame(pygame.Surface((w, h)), w, h, mode=mode, game_settings=gs))


def key(g, k, char=None):
    g.handle_event(InputEvent(InputEvent.KEYDOWN, key=k,
                              char=char if char is not None else (k if len(k) == 1 else "")))


def click(g, pos):
    g.handle_event(InputEvent(InputEvent.MOUSEDOWN, pos=(int(pos[0]), int(pos[1]))))


def settle(g, limit=4.0):
    """Aufdeckung zu Ende laufen lassen."""
    for _ in range(int(limit * 60)):
        if g.state != wd.REVEAL:
            break
        if g.game_over:
            g._tick(1 / 60)
        else:
            g.update(1 / 60)


def guess(g, word):
    for ch in word:
        key(g, ch.lower())
    key(g, "Return")
    settle(g)


def started(mode="endless", answers=None, **kw):
    """Partie starten und (optional) die Lösungswörter festlegen."""
    g = make(mode, **kw)
    g._start()
    if answers:
        g.boards = [wd.Board(a) for a in answers]
        g._clear_round()
        g.state = wd.PLAY
    return g


# ============================================================ (1)-(2) Logik

def ref_evaluate(guess_word, answer):
    """Unabhängige Referenz: Zähler statt Streichliste."""
    n = len(answer)
    out = [None] * n
    counts = {}
    for i in range(n):
        if guess_word[i] == answer[i]:
            out[i] = "correct"
        else:
            counts[answer[i]] = counts.get(answer[i], 0) + 1
    for i in range(n):
        if out[i]:
            continue
        ch = guess_word[i]
        if counts.get(ch, 0) > 0:
            out[i] = "present"
            counts[ch] -= 1
        else:
            out[i] = "absent"
    return out


C, P, A = "correct", "present", "absent"


def audit_logic():
    print("\nWordle - Logik (evaluate, harter Modus)")
    cases = [
        ("ABBE", "BABY", [P, P, C, A]),
        ("LOOL", "LOLA", [C, C, A, P]),
        ("SPEED", "ABIDE", [A, A, P, A, P]),
        ("EERIE", "THEME", [P, A, A, A, C]),
        ("ALLEE", "LLAMA", [P, C, P, A, A]),
        ("KAYAK", "KAYAK", [C] * 5),
        ("LETTER", "BETTER", [A, C, C, C, C, C]),
        ("TTTTTT", "BUTTER", [A, A, C, C, A, A]),
        ("ANNANAS", "BANANEN", [P, P, C, C, C, A, A]),
        ("OOOAAAA", "AOAOAOA", [P, C, P, P, C, P, C]),
    ]
    bad = [(gw, an, wd.evaluate(gw, an)) for gw, an, want in cases if wd.evaluate(gw, an) != want]
    check(not bad, "feste Fälle (4-7 Buchstaben, Doppelbuchstaben)", bad[:2])
    rng = random.Random(4)
    mismatch = []
    for n in ww.LENGTHS:
        for _ in range(4000):
            alpha = "ABCDE"[:rng.randint(2, 5)]
            an = "".join(rng.choice(alpha) for _ in range(n))
            gw = "".join(rng.choice(alpha) for _ in range(n))
            if wd.evaluate(gw, an) != ref_evaluate(gw, an):
                mismatch.append((gw, an))
    check(not mismatch, "16.000 Zufallsfälle = Referenz", mismatch[:3])

    # harter Modus je Länge
    probs = []
    for n in ww.LENGTHS:
        answer = "ABCDEFG"[:n]
        first = "A" + "X" * (n - 2) + "B"         # A grün, B gelb
        hist = [(first, wd.evaluate(first, answer))]
        ok_guess = "AB" + "Y" * (n - 2)
        if wd.hard_problem(ok_guess, hist) is not None:
            probs.append("%d: gültiger Versuch abgelehnt" % n)
        p = wd.hard_problem("Y" * n, hist)
        if not p or p[0] != "wd.hard_green" or p[1] != {"n": 1, "c": "A"}:
            probs.append("%d: grüne Regel %r" % (n, p))
        p = wd.hard_problem("A" + "Y" * (n - 1), hist)
        if not p or p[0] != "wd.hard_yellow" or p[1] != {"c": "B"}:
            probs.append("%d: gelbe Regel %r" % (n, p))
    # doppelte Buchstaben: zwei E gefunden -> zwei E nötig
    hist = [("EERIE", wd.evaluate("EERIE", "EMBER"))]
    if wd.hard_problem("EXXXX", hist) is None or wd.hard_problem("EXREX", hist) is not None:
        probs.append("Doppelbuchstaben-Zählung")
    check(not probs, "harter Modus für 4-7 Buchstaben (grün/gelb/doppelt)", probs[:3])


# ============================================================ (3) Listen

def read_js_list(path, n):
    text = open(path, encoding="utf-8").read()
    m = re.search(r'PG\.wordleWords\.add\("(\w+)", "([A-Z]*)", "([A-Z]*)", (\d)\)', text)
    if not m or int(m.group(4)) != n:
        return None, None
    split = lambda s: [s[i:i + n] for i in range(0, len(s), n)]
    answers = split(m.group(2))
    return answers, set(split(m.group(3))) | set(answers)


def audit_lists():
    print("\nWordle - Wortlisten (14 Sprachen x Längen 4-7)")
    base = os.path.join(REPO, "woordlistz")
    probs, small, blocked, web = [], [], [], []
    sizes = {}
    for lang in LANGS:
        adult = bw.ADULT_BLOCK["*"] | bw.ADULT_BLOCK.get(lang, set()) if lang != "en" else set()
        names = bw.NAME_BLOCK.get(lang, set())
        for n in ww.LENGTHS:
            a_name, l_name = ww.file_names(n)
            try:
                raw_a = open(os.path.join(base, lang, a_name + ".txt"), encoding="utf-8").read().split("\n")
                raw_l = open(os.path.join(base, lang, l_name + ".txt"), encoding="utf-8").read().split("\n")
            except OSError as exc:
                probs.append("%s/%d fehlt (%s)" % (lang, n, exc))
                continue
            a = [x for x in raw_a if x]
            l = [x for x in raw_l if x]
            sizes[(lang, n)] = (len(a), len(l))
            pat = re.compile("^[A-Z]{%d}$" % n)
            if not all(pat.match(x) for x in a + l):
                probs.append("%s/%d ungültige Zeichen" % (lang, n))
            if a != sorted(set(a)) or l != sorted(set(l)):
                probs.append("%s/%d unsortiert/doppelt" % (lang, n))
            if not set(a) <= set(l):
                probs.append("%s/%d Lösungen fehlen in allowed" % (lang, n))
            if len(a) < bw.ANSWER_MIN_TOTAL or len(l) < 3 * len(a) // 2:
                small.append("%s/%d %d/%d" % (lang, n, len(a), len(l)))
            hit = sorted((set(a) & (adult | names)) - bw.ANSWER_KEEP.get(lang, set()))
            if hit:
                blocked.append("%s/%d %s" % (lang, n, " ".join(hit[:4])))
            if ww.words_for(lang, n) != a or ww.allowed_for(lang, n) != set(l):
                probs.append("%s/%d wordle_words lädt anders" % (lang, n))
            js = os.path.join(REPO, "web", "js", "games", "wordle_words",
                              lang + ("" if n == 5 else str(n)) + ".js")
            ja, jl = read_js_list(js, n) if os.path.exists(js) else (None, None)
            if ja != a or jl != set(l):
                web.append("%s/%d" % (lang, n))
    check(not probs, "56 Listen vorhanden, nur A-Z, sortiert, Lösungen ⊆ Rateworte", probs[:4])
    check(not small, "mind. %d Lösungswörter je Sprache/Länge" % bw.ANSWER_MIN_TOTAL, small)
    check(not blocked, "keine gesperrten Namen/Wörter unter den Lösungen", blocked[:4])
    check(sizes.get(("en", 5), (0,))[0] == 2313, "Englisch/5 = Original-Lösungsliste (2313)",
          sizes.get(("en", 5)))
    check(not web, "Web-Dateien wordle_words/<code><n>.js = Textdateien", web[:5])
    lo = min(v[0] for v in sizes.values())
    hi = max(v[1] for v in sizes.values())
    print("       (Lösungswörter min %d, Rateworte max %d)" % (lo, hi))
    check(ww.has_length("de", 7) and ww.counts_for("de", 5)[0] > 1000, "has_length/counts_for")


# ============================================================ (4) Tageswort

def node_check(request):
    node = shutil.which("node")
    if not node:
        return None
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False, encoding="utf-8") as f:
        json.dump(request, f)
        path = f.name
    try:
        out = subprocess.run([node, os.path.join(REPO, "web", "tools", "wordle_check.js"), path],
                             capture_output=True, text=True, encoding="utf-8", timeout=300)
        if out.returncode != 0:
            print("       node:", out.stderr[-400:])
            return {}
        return json.loads(out.stdout)
    finally:
        os.remove(path)


def audit_daily_logic():
    print("\nWordle - Tageswort")
    today = seedrand.today_str()
    words = ww.words_for("de", 5)
    a1 = wd.daily_answer(words, "de", 5, today)
    wd._order_cache.clear()
    check(wd.daily_answer(list(words), "de", 5, today) == a1 and a1 in words,
          "deterministisch (auch nach neuem Mischen)")
    per_day = [wd.daily_answer(words, "de", 5, "2026-%02d-%02d" % (m, d))
               for m in range(1, 7) for d in range(1, 29)]
    check(len(set(per_day)) == len(per_day), "168 Tage ohne Wiederholung")
    distinct = {wd.daily_answer(ww.words_for(lang, n), lang, n, today)
                for lang in ("de", "en", "fr") for n in ww.LENGTHS}
    check(len(distinct) == 12, "je Sprache/Länge ein eigenes Wort")
    check(wd.seconds_to_midnight() <= 86400 and wd.format_hms(3725) == "01:02:05",
          "Countdown bis Mitternacht")

    dates = [today, "2026-01-01", "2027-03-15"]
    req = {"daily": [[lang, n, d] for lang in LANGS for n in ww.LENGTHS for d in dates],
           "counts": [[lang, n] for lang in LANGS for n in ww.LENGTHS]}
    res = node_check(req)
    if res is None:
        print("  SKIP node nicht gefunden")
        return
    py_daily = [wd.daily_answer(ww.words_for(lang, n), lang, n, d) for lang, n, d in req["daily"]]
    diff = [tuple(r) for r, a, b in zip(req["daily"], py_daily, res.get("daily", [])) if a != b]
    check(res.get("daily") and not diff, "Tageswort Python == JS (14 Sprachen x 4 Längen x 3 Tage)", diff[:3])
    py_counts = [list(ww.counts_for(lang, n)) for lang, n in req["counts"]]
    check(res.get("counts") == py_counts, "Listen im Browser gleich groß wie am PC")


# ============================================================ (5) Endlos

def audit_endless():
    print("\nWordle - Endlos")
    wipe_store()
    for length in (5, 6):
        g = started("endless", ["ABCDE"[:length] if length == 5 else "ABCDEF"], length=length)
        answer = g.boards[0].answer
        g.allowed = set(g.allowed) | {answer, "X" * length, "Y" * length}
        guess(g, "X" * length)
        guess(g, answer)
        check(g.state == wd.SOLVED and g.points == 50 and g.solved_count == 1,
              "%d Buchstaben: gelöst in 2 -> 50 Punkte" % length, (g.state, g.points))
        check(("ach", "wordle_two", None) in g.events, "%d: wordle_two ausgelöst" % length)
        if length == 5:
            check(g.score == 50 and g.show_highscore_banner, "5 Buchstaben füllen score + Banner")
        else:
            check(g.score == 0 and not g.show_highscore_banner, "6 Buchstaben: score 0, kein Banner")
        key(g, "Return")                                  # nächstes Wort
        check(g.state == wd.PLAY and not g.guesses, "Enter -> nächstes Wort")
        g.boards = [wd.Board(answer)]
        for _ in range(6):
            guess(g, "Y" * length)
        check(g.state == wd.DONE and g.game_over, "%d: 6 Fehlversuche -> Game Over" % length)
        best = wd.load_data()["best"].get(str(length))
        check(best == 50 and g.result["new_best"], "%d: Bestwert je Länge gespeichert" % length, best)
        st = wd.get_stats(wd.load_data(), "endless", g.lang, length)
        check(st["played"] == 2 and st["won"] == 1 and st["dist"][1] == 1 and st["streak"] == 0,
              "%d: Statistik (2 Partien, 1 Sieg, Verteilung)" % length, st)
        g.draw()
        key(g, "Return")                                  # nochmal
        check(g.state == wd.PLAY and not g.game_over and g.points == 0, "%d: Enter = neue Partie" % length)

    # Tastatur-Details
    g = started("endless", ["HOUSE"])
    g.allowed = set(g.allowed) | {"HOUSE"}
    for ch in "HOUS":
        key(g, ch.lower())
    key(g, "Return")
    check(g.state == wd.PLAY and g.shake > 0 and "5" in g.message, "zu kurz -> Meldung mit Länge + Wackeln")
    key(g, "e")
    key(g, "x")
    check(g.current == "HOUSE", "mehr als N Buchstaben werden ignoriert")
    key(g, "BackSpace")
    check(g.current == "HOUS", "Rücktaste löscht")
    key(g, "BackSpace")
    for ch in "QZXVJ":
        key(g, ch.lower())
    key(g, "Return")
    check(g.message == i18n.t("wd.not_a_word"), "unbekanntes Wort abgelehnt")
    g.current = ""
    click(g, g.key_rects["H"].center)
    check(g.current == "H", "Bildschirmtastatur tippt")
    click(g, g.del_rect.center)
    check(g.current == "", "Bildschirm-Rücktaste")
    key(g, "ä", char="ä")
    check(g.current == "", "Umlaute/Nicht-ASCII ignoriert")

    # Abbruch übers Regler-Symbol sichert die Serie
    wipe_store()
    g = started("endless", ["HOUSE"])
    g.allowed = set(g.allowed) | {"HOUSE"}
    import highscore
    saved = []
    orig = highscore.update_highscore
    highscore.update_highscore = lambda k, s: (saved.append((k, s)) or (s, True))
    try:
        guess(g, "HOUSE")
        click(g, g.gear_rect.center)
    finally:
        highscore.update_highscore = orig
    check(g.state == wd.SETUP and g.score == 0 and saved == [("wordle", 60)]
          and wd.load_data()["best"].get("5") == 60, "Setup mitten im Lauf sichert Highscore + Bestwert",
          (saved, wd.load_data()["best"]))

    # harter Modus im Spiel, je Länge
    probs = []
    for length in ww.LENGTHS:
        answer = "ABCDEFG"[:length]
        g = started("endless", [answer], length=length, hard=True)
        first = "A" + "X" * (length - 2) + "B"
        g.allowed = set(g.allowed) | {answer, first, "Y" * length, "AB" + "Y" * (length - 2)}
        guess(g, first)
        guess(g, "Y" * length)
        if len(g.guesses) != 1 or g.message != i18n.t("wd.hard_green", n=1, c="A"):
            probs.append("%d grün" % length)
        g.current = ""
        guess(g, "AB" + "Y" * (length - 2))
        if len(g.guesses) != 2:
            probs.append("%d gültig" % length)
    check(not probs, "harter Modus im Spiel (4-7)", probs)


# ============================================================ (6) Tageswort-Spiel

def audit_daily_game():
    print("\nWordle - Tageswort im Spiel")
    wipe_store()
    g = started("daily")
    answer = g.boards[0].answer
    check(answer == wd.daily_answer(ww.words_for(g.lang, 5), g.lang, 5), "Spiel nimmt das Tageswort")
    others = [w for w in ww.words_for(g.lang, 5) if w != answer][:3]
    guess(g, others[0])
    guess(g, others[1])
    g2 = started("daily")
    check(g2.guesses == others[:2] and len(g2.boards[0].rows) == 2 and g2.state == wd.PLAY,
          "angefangenes Tageswort nach Neustart wieder da")
    guess(g2, answer)
    check(g2.state == wd.DONE and g2.result["won"] and g2.result["tries"] == 3 and g2.score == 0
          and not g2.show_highscore_banner, "gelöst: Ergebnis, kein Highscore")
    check(("ach", "wordle_daily7", 1) in g2.events, "wordle_daily7 mit Serienwert")
    g2.draw()
    g3 = started("daily")
    check(g3.state == wd.DONE and g3.daily_view and g3.result["won"] and not g3.game_over,
          "heute schon gelöst -> nur Ergebnis + Countdown")
    g3.draw()
    key(g3, "Return")
    check(g3.state == wd.SETUP, "Enter führt (selber Tag) ins Setup")
    prefix = i18n.t("wd.daily.next", t="§").split("§")[0]
    check(g3._setup_info().startswith(prefix), "Setup zeigt Countdown", g3._setup_info())
    # Anderer Tag im Speicher -> verworfen
    data = wd.load_data()
    data["daily"]["de:6"] = {"date": "2020-01-01", "answer": "XXXXXX", "guesses": ["AAAAAA"]}
    wd.save_data(data)
    g4 = started("daily", length=6)
    check(not g4.guesses and g4.state == wd.PLAY, "alter Tagesstand wird ignoriert")
    # Serie über Tage
    data = {"stats": {}, "daily": {}, "best": {}}
    streaks = []
    for day, won in ((10, True), (11, True), (12, True), (14, True), (15, False), (16, True)):
        st = wd.record_game(data, "daily", "de", 5, won, 3, day)
        streaks.append(st["streak"])
    check(streaks == [1, 2, 3, 1, 0, 1] and st["best"] == 3, "Serie wächst nur an Folgetagen", streaks)
    check(wd.shown_streak({"streak": 4, "last_day": 20}, "daily", 21) == 4
          and wd.shown_streak({"streak": 4, "last_day": 20}, "daily", 22) == 0,
          "angezeigte Serie verfällt nach einem Tag Pause")
    # 7 Tage in Folge -> Ereignis mit 7
    wipe_store()
    data = wd.load_data()
    today = seedrand.day_index()
    data["stats"][wd.stats_key("daily", "de", 5)] = {"played": 6, "won": 6, "streak": 6, "best": 6,
                                                     "dist": [0, 0, 6, 0, 0, 0], "last_day": today - 1}
    wd.save_data(data)
    g = started("daily")
    guess(g, g.boards[0].answer)
    check(("ach", "wordle_daily7", 7) in g.events and ("ach", "wordle_two", None) in g.events,
          "7. Tag in Folge -> wordle_daily7(7) (+ wordle_two)")


# ============================================================ (7) Dordle/Quordle

def audit_multi():
    print("\nWordle - Dordle / Quordle")
    wipe_store()
    g = started("dordle", ["HOUSE", "MOUSE"])
    g.allowed = set(g.allowed) | {"HOUSE", "MOUSE", "XXXXX"}
    check(g.rows == 7 and len(g.board_pos) == 2, "Dordle: 2 Bretter, 7 Versuche")
    guess(g, "HOUSE")
    check(g.boards[0].solved and not g.boards[1].solved and g.state == wd.PLAY,
          "ein Brett gelöst, Partie läuft weiter")
    check(g.boards[1].keys.get("O") == "correct" and g.boards[1].keys.get("H") == "absent",
          "Tastatur-Zustand je Brett")
    guess(g, "XXXXX")
    check(len(g.boards[0].rows) == 1 and len(g.boards[1].rows) == 2, "gelöstes Brett bekommt keine Zeilen mehr")
    guess(g, "MOUSE")
    check(g.state == wd.DONE and g.result["won"] and g.result["tries"] == 3 and g.score == 0
          and g.game_over and not g.show_highscore_banner, "Dordle gewonnen, kein Highscore")
    g.draw()

    g = started("quordle", ["HOUSE", "MOUSE", "LOUSE", "DOUSE"])
    g.allowed = set(g.allowed) | {"HOUSE", "MOUSE", "LOUSE", "DOUSE", "XXXXX"}
    check(g.rows == 9 and len(g.board_pos) == 4, "Quordle: 4 Bretter, 9 Versuche")
    for w in ("HOUSE", "MOUSE", "LOUSE", "DOUSE"):
        guess(g, w)
    check(g.state == wd.DONE and g.result["won"] and ("ach", "wordle_quordle", None) in g.events,
          "Quordle gewonnen -> wordle_quordle")
    st = wd.get_stats(wd.load_data(), "quordle", g.lang, 5)
    check(st["dist"][3] == 1 and len(st["dist"]) == 9, "Quordle-Verteilung (4 von 9)", st["dist"])
    g.draw()

    g = started("quordle", ["HOUSE", "MOUSE", "LOUSE", "DOUSE"])
    g.allowed = set(g.allowed) | {"HOUSE", "XXXXX"}
    guess(g, "HOUSE")
    for _ in range(8):
        guess(g, "XXXXX")
    check(g.state == wd.DONE and not g.result["won"] and ("result", False) in g.events
          and ("ach", "wordle_quordle", None) not in g.events, "Quordle verloren nach 9 Versuchen")
    g._tick(2.0)
    g.draw()
    check(bool(g._done_buttons), "Ergebnis-Panel mit Knöpfen")

    # harter Modus mit mehreren Brettern: passt zu einem Brett = erlaubt
    g = started("dordle", ["ABCDE", "VWXYZ"], hard=True)
    g.allowed = set(g.allowed) | {"AQQQQ", "QQQQQ", "VQQQQ"}
    guess(g, "AQQQQ")
    guess(g, "VQQQQ")
    check(len(g.guesses) == 2, "Dordle hart: Versuch passt zu einem Brett")
    guess(g, "QQQQQ")
    check(len(g.guesses) == 2 and g.message == i18n.t("wd.hard_multi"), "Dordle hart: passt zu keinem -> abgelehnt")


def win_clipboard_read():
    """Unicode-Text der Windows-Zwischenablage (None, wenn keiner da ist)."""
    import ctypes
    user32, kernel32 = ctypes.windll.user32, ctypes.windll.kernel32
    user32.OpenClipboard.argtypes = [ctypes.c_void_p]
    user32.GetClipboardData.restype = ctypes.c_void_p
    kernel32.GlobalLock.argtypes = [ctypes.c_void_p]
    kernel32.GlobalLock.restype = ctypes.c_void_p
    kernel32.GlobalUnlock.argtypes = [ctypes.c_void_p]
    if not user32.OpenClipboard(None):
        return None
    try:
        handle = user32.GetClipboardData(13)
        ptr = kernel32.GlobalLock(handle) if handle else None
        if not ptr:
            return None
        text = ctypes.wstring_at(ptr)
        kernel32.GlobalUnlock(handle)
        return text
    finally:
        user32.CloseClipboard()


# ============================================================ (8)-(9) Statistik, Teilen

def audit_stats_share():
    print("\nWordle - Statistik und Teilen")
    data = {"stats": {"endless:de:5": {"played": "x", "won": 99, "streak": -1, "dist": [1, "a", 2]}}}
    st = wd.get_stats(data, "endless", "de", 5)
    check(st == {"played": 0, "won": 0, "streak": 0, "best": 0, "dist": [1, 0, 2, 0, 0, 0]},
          "kaputte Statistik wird bereinigt", st)

    seq = [["endless", True, 3, None], ["endless", True, 1, None], ["endless", False, 6, None],
           ["endless", True, 6, None]]
    data = {"stats": {}, "daily": {}, "best": {}}
    py_rec = [wd.record_game(data, m, "de", 5, won, tries, day) for m, won, tries, day in seq]
    check(py_rec[-1]["played"] == 4 and py_rec[-1]["won"] == 3 and py_rec[-1]["best"] == 2
          and py_rec[-1]["dist"] == [1, 0, 1, 0, 0, 1], "Verteilung, Quote, beste Serie", py_rec[-1])

    rows = [[A, P, A, A, C], [C, C, C, C, C]]
    one = wd.share_text("daily", "de", 5, [(rows, 2)], hard=True, date="2026-09-16")
    lines = one.split("\n")
    check(lines[0] == "PyGameZ Wordle 2026-09-16 · DE · 5 2/6*" and lines[1] == ""
          and lines[2] == "⬛\U0001F7E8⬛⬛\U0001F7E9" and lines[3] == "\U0001F7E9" * 5,
          "1 Brett: Kopf + Raster", lines[:4])
    cb = wd.share_text("endless", "fr", 5, [(rows, 2)], colorblind=True)
    check("\U0001F7E7" in cb and "\U0001F7E6" in cb and "\U0001F7E9" not in cb and "FR" in cb,
          "Farbenblind: Orange/Blau")
    two = wd.share_text("dordle", "en", 4, [([[C] * 4], 1), ([[A] * 4, [P] * 4], None)])
    check(two.split("\n")[0].endswith("1&X/7") and two.split("\n")[2] == "\U0001F7E9" * 4 + " " + "⬛" * 4
          and two.split("\n")[3] == "⬜" * 4 + " " + "\U0001F7E8" * 4, "2 Bretter nebeneinander", two)
    four = wd.share_text("quordle", "de", 6, [([[C] * 6], 1), ([[A] * 6, [C] * 6], 2),
                                               ([[A] * 6], None), ([[C] * 6], 1)])
    fl = four.split("\n")
    check(fl[1] == "1️⃣2️⃣" and fl[2] == "\U0001F7E5" + "1️⃣" and len(fl) == 8, "4 Bretter mit Zahlen-Emoji", fl)

    share_req = [["daily", "de", 5, [[rows, 2]], True, False, "2026-09-16"],
                 ["endless", "fr", 7, [[[[A] * 7], None]], False, True, None],
                 ["dordle", "en", 4, [[[[C] * 4], 1], [[[A] * 4, [P] * 4], None]], False, False, None],
                 ["quordle", "de", 6, [[[[C] * 6], 1], [[[A] * 6, [C] * 6], 2], [[[A] * 6], None],
                                       [[[C] * 6], 1]], True, True, None]]
    eval_req = [["LETTER", "BETTER"], ["ANNANAS", "BANANEN"], ["EERIE", "THEME"], ["ABBE", "BABY"]]
    hard_req = [["YYYYY", [["AXXXB", wd.evaluate("AXXXB", "ABCDE")]]],
                ["AYYYY", [["AXXXB", wd.evaluate("AXXXB", "ABCDE")]]],
                ["ABYYYY", [["AXXXXB", wd.evaluate("AXXXXB", "ABCDEF")]]]]
    res = node_check({"share": share_req, "record": seq, "evaluate": eval_req, "hard": hard_req})
    if res is None:
        print("  SKIP node nicht gefunden")
    else:
        py_share = [wd.share_text(m, l, n, [tuple(b) for b in boards], h, c, d)
                    for m, l, n, boards, h, c, d in share_req]
        check(res.get("share") == py_share, "Teilen-Text Python == JS")
        check(res.get("record") == py_rec, "Statistik Python == JS")
        check(res.get("evaluate") == [wd.evaluate(a, b) for a, b in eval_req], "evaluate Python == JS")
        py_hard = [list(p) if p else None for p in (wd.hard_problem(gw, [tuple(h) for h in hist])
                                                     for gw, hist in hard_req)]
        check(res.get("hard") == py_hard, "harter Modus Python == JS")

    # Zwischenablage: Rundlauf über ein echtes (verstecktes) Tk-Fenster und
    # die Win32-API - der vorherige Inhalt wird danach wiederhergestellt.
    # Nur auf Wunsch (PGZ_CLIPBOARD_TEST=1): die Zwischenablage ist systemweit -
    # parallel laufende Programme (z.B. der Web-Smoke-Test) machen den Test
    # sonst unzuverlässig, und der Nutzer soll nicht auf fremden Text stoßen.
    win = sys.platform == "win32"
    if os.environ.get("PGZ_CLIPBOARD_TEST") != "1":
        print("  SKIP Zwischenablage-Rundlauf (PGZ_CLIPBOARD_TEST=1 zum Einschalten)")
        win = None
    before = win_clipboard_read() if win else None
    try:
        if win is None:
            raise RuntimeError("übersprungen")
        import tkinter
        root = tkinter.Tk()
        root.withdraw()
        text = wd.share_text("quordle", "de", 5, [([[C] * 5], 1)] * 4)
        ok = wd.copy_to_clipboard(text, tk_root=root)
        root.update()
        back = win_clipboard_read() if win else root.clipboard_get()
        # Windows speichert Zeilenumbrüche als CRLF - der Inhalt bleibt gleich.
        check(ok and (back or "").replace("\r\n", "\n") == text,
              "Emoji-Raster landet unverändert in der Zwischenablage (Tk)")
        if win:
            check(_REAL_WIN_CLIPBOARD("PyGameZ ⬛🟩") and win_clipboard_read() == "PyGameZ ⬛🟩",
                  "Win32-Fallback schreibt Unicode")
        root.destroy()
    except Exception as exc:                   # kein Display -> nur Headless-Pfad prüfen
        print("  SKIP Tk-Zwischenablage:", exc)
    if win and before is not None:
        _REAL_WIN_CLIPBOARD(before)
    g = started("endless", ["HOUSE"])
    g.state = wd.SOLVED
    g.boards[0].apply("HOUSE", [C] * 5)
    key(g, "c")
    check(g.message == i18n.t("wd.copy_failed"), "ohne Zwischenablage: Meldung statt Absturz")


# ============================================================ (10) Setup

def audit_setup():
    print("\nWordle - Setup, Einstellungen, Tastaturen")
    wipe_store()
    SAVED_SETTINGS.clear()
    g = make("endless")
    check(g.state == wd.SETUP and g.setup_focus == 3, "startet im Setup")
    key(g, "7")
    check(g.length == 7 and SAVED_SETTINGS and SAVED_SETTINGS[-1]["wordle"]["length"] == 7,
          "Taste 7 -> Länge gespeichert")
    key(g, "Left")
    check(g.length == 6, "Pfeil links -> 6")
    click(g, g.su_seg[0].center)
    check(g.length == 4, "Klick auf 4")
    key(g, "h")
    check(g.hard and SAVED_SETTINGS[-1]["wordle"]["hard"] is True, "H -> harter Modus gespeichert")
    click(g, g.su_cb.center)
    check(g.colorblind and SAVED_SETTINGS[-1]["wordle"]["colorblind"] is True
          and g.palette()["correct"] == wd.PALETTES[True]["correct"], "Farbenblind per Klick")
    for _ in range(20):
        g.draw()
    click(g, g.su_start.center)
    check(g.state == wd.PLAY and g.length == 4 and len(g.boards[0].answer) == 4, "Start mit 4 Buchstaben")

    gs = settings_for()
    gs["wordle"] = {"length": 12, "hard": "ja", "colorblind": None}
    g = make("endless", gs=gs)
    check(g.length == 5 and g.hard is False and g.colorblind is False, "kaputte Einstellungen -> Standard")
    SAVED_SETTINGS.clear()
    g = make("hard")
    check(g.game_mode == "endless" and g.hard and SAVED_SETTINGS and SAVED_SETTINGS[-1]["wordle"]["hard"],
          "alter Modus 'hard' -> Endlos + harter Modus")
    g = make("normal")
    check(g.game_mode == "endless", "alter Modus 'normal' -> Endlos")
    check([m for m, _k in wd.WordleGame.MODES] == ["endless", "daily", "dordle", "quordle"], "4 Modi")

    lang_before = i18n.get_language()
    layouts = {}
    for code in LANGS:
        i18n.set_language(code, persist=False)
        g = make("endless")
        rows = {}
        for ch, rc in sorted(g.key_rects.items(), key=lambda kv: (kv[1].y, kv[1].x)):
            rows.setdefault(rc.y, []).append(ch)
        layouts[code] = ["".join(r) for _y, r in sorted(rows.items())]
        if "".join(sorted(g.key_rects)) != "ABCDEFGHIJKLMNOPQRSTUVWXYZ":
            layouts[code] = None
    i18n.set_language(lang_before, persist=False)
    check(layouts["fr"] == wd.KEYBOARDS["azerty"] and layouts["de"] == wd.KEYBOARDS["qwertz"]
          and layouts["en"] == wd.KEYBOARDS["qwerty"] and all(layouts.values()),
          "Tastaturen: AZERTY (fr), QWERTZ (de/cs/sl/hr), QWERTY; alle 26 Buchstaben")

    g = started("quordle", ["HOUSE", "MOUSE", "LOUSE", "DOUSE"], w=800, h=600)
    g.allowed = set(g.allowed) | {"HOUSE"}
    guess(g, "HOUSE")
    for w, h in ((480, 360), (1280, 960)):
        g.surface = pygame.Surface((w, h))
        g.width, g.height = w, h
        g.on_surface_changed()
        g.draw()
        inside = all(0 <= bx and bx + g.board_w <= w and by >= g.hud_h and by + g.board_h <= g.kb_top
                     for bx, by in g.board_pos)
        check(inside and g.boards[0].solved, "Auflösungswechsel mitten in Quordle -> %dx%d" % (w, h))


# ============================================================ (11) Layout

def overlap(rects):
    for i, a in enumerate(rects):
        for b in rects[i + 1:]:
            if a.colliderect(b):
                return (tuple(a), tuple(b))
    return None


class FitRecorder:
    """Zeichnet alle _text-Aufrufe mit und merkt sich Notschriften/Kürzungen."""

    def __init__(self, g):
        self.g = g
        self.bad = []
        orig = g._text

        def text(s, t_, px, color, max_w, bold=False, **anchor):
            f = g._fit(t_, max_w, px, bold)
            size = f.get_height()
            if not t_.replace("%", "").replace(".", "").replace(":", "").strip().isdigit():
                full = ui.font(px, bold=bold).get_height()
                if f.size(t_)[0] > max_w:
                    self.bad.append("gekürzt '%s'" % t_[:40])
                elif size < full * 0.6 or size < ui.font(9, bold=bold).get_height():
                    self.bad.append("%d->%dpx '%s'" % (px, f.get_height(), t_[:40]))
            return orig(s, t_, px, color, max_w, bold, **anchor)
        g._text = text


def audit_layout():
    print("\nWordle - Layout (5 Auflösungen x Längen x Modi, 14 Sprachen)")
    wipe_store()
    lang_before = i18n.get_language()
    theme_before = ui.theme_name()
    for w, h in RESOLUTIONS:
        geo = []
        for mode in ("endless", "daily", "dordle", "quordle"):
            for length in ww.LENGTHS:
                for code in ("de", "fr"):                # QWERTZ/AZERTY haben verschieden viele Tasten
                    i18n.set_language(code, persist=False)
                    g = make(mode, w, h, length=length)
                    setup = g.su_seg + [g.su_hard, g.su_cb, g.su_start]
                    if any(r.left < 0 or r.right > w or r.top < 0 or r.bottom > h - 22 for r in setup + [g.su_stats_rect]):
                        geo.append("Setup außerhalb %s/%d" % (mode, length))
                    if overlap(setup + [g.su_stats_rect]):
                        geo.append("Setup überlappt %s" % mode)
                    g._start()
                    keys = list(g.key_rects.values()) + [g.enter_rect, g.del_rect]
                    if any(r.left < 0 or r.right > w or r.bottom > h for r in keys) or overlap(keys):
                        geo.append("Tastatur %s/%s/%d" % (code, mode, length))
                    frames = [pygame.Rect(bx - g.board_inner, by - g.board_inner, g.board_w + 2 * g.board_inner,
                                          g.board_h + 2 * g.board_inner) for bx, by in g.board_pos]
                    if any(f.left < 0 or f.right > w or f.top < g.hud_h or f.bottom > g.kb_top for f in frames) \
                            or overlap(frames):
                        geo.append("Bretter %s/%d (%s)" % (mode, length, frames[0]))
                    min_tile = 22 if g.n_boards == 1 else (13 if g.n_boards == 2 else 9)
                    if g.tile < min_tile * min(w / 480.0, 2.0):
                        geo.append("Kacheln zu klein %s/%d: %d" % (mode, length, g.tile))
                    if g.gear_rect.right > w or g.gear_rect.bottom > g.hud_h:
                        geo.append("Regler")
        i18n.set_language(lang_before, persist=False)
        check(not geo, "%4dx%d: Setup, Tastatur und Bretter im Bild" % (w, h), "; ".join(sorted(set(geo))[:5]))

        probs = []
        for theme in ("v42", "v1"):
            ui.set_theme(theme)
            for code in LANGS:
                if theme == "v1" and code not in ("de", "fi", "pl", "hr"):
                    continue
                i18n.set_language(code, persist=False)
                for mode in ("endless", "daily", "dordle", "quordle"):
                    wipe_store()
                    g = make(mode, w, h, length=7 if mode != "endless" else 5)
                    rec = FitRecorder(g)
                    g.data["best"]["7"] = g.data["best"]["5"] = 123456
                    g.draw()                                                # Setup
                    g._start()
                    n = g.length
                    answers = [b.answer for b in g.boards]
                    others = [x for x in ww.words_for(code, n) if x not in answers][:g.rows + 2]
                    g.allowed = set(g.allowed) | set(others) | set(answers)
                    g.points, g.solved_count = 98765, 432
                    guess(g, others[0])
                    for ch in others[1][:3]:
                        key(g, ch.lower())
                    g._reject(i18n.t("wd.hard_multi") if mode in ("dordle", "quordle") else i18n.t("wd.too_short", n=n))
                    g.draw()                                                # Spiel + Meldung
                    g.current = ""
                    key(g, "Return")
                    g._say(i18n.t("wd.not_a_word"))
                    g.draw()
                    g.current = others[1]
                    g._submit()
                    for _ in range(12):
                        g.update(1 / 60)
                    g.draw()                                                # Aufdeckung
                    settle(g)
                    if mode == "endless":
                        g.current = answers[0]
                        g._submit()
                        settle(g)
                        g._tick(1.0)
                        g.draw()                                            # Gelöst-Banner
                        key(g, "Return")
                        g.boards = [wd.Board(answers[0])]
                    rng = random.Random(1)
                    for x in others[2:]:
                        if g.state == wd.DONE:
                            break
                        g.current = x
                        g._submit()
                        settle(g)
                    data = g.result["stats"] if g.result else None
                    if data:
                        data["played"], data["won"], data["best"] = 9999, 8888, 777
                        data["dist"] = [rng.randint(0, 999) for _ in data["dist"]]
                    g._tick(2.0)
                    g._say(i18n.t("wd.copied"))
                    g.draw()                                                # Ergebnis-Panel
                    panel = g._done_panel
                    btns = [rc for rc, _a in g._done_buttons]
                    if panel is None or panel.top < 0 or panel.bottom > h or \
                            any(not panel.contains(b) for b in btns) or overlap(btns):
                        probs.append("%s/%s Panel %s" % (code, mode, panel))
                    if g.show_highscore_banner and panel.bottom > h - wd.BANNER_SPACE + 4:
                        probs.append("%s Panel unter dem Highscore-Banner" % mode)
                    key(g, "space")
                    g.draw()                                                # Bretter ansehen
                    key(g, "s")
                    g.draw()                                                # Setup mit Statistik
                    probs += ["%s/%s/%s: %s" % (theme, code, mode, b) for b in rec.bad]
        i18n.set_language(lang_before, persist=False)
        ui.set_theme(theme_before)
        check(not probs, "%4dx%d: Texte passen, Panel im Bild (14 Sprachen, v4.2 + v1)" % (w, h),
              "; ".join(sorted(set(probs))[:6]))
    wipe_store()


# ============================================================ (12) Tempo

def audit_performance():
    print("\nWordle - Tempo (Quordle, 7 Buchstaben, 1280x960)")
    wipe_store()
    theme_before = ui.theme_name()
    for theme in ("v42", "v1"):
        ui.set_theme(theme)
        g = started("quordle", w=1280, h=960, length=7)
        pool = ww.words_for(g.lang, 7)
        g.allowed = set(g.allowed) | set(pool[:40])
        for _ in range(30):
            g.draw()
            g.update(1 / 60)
        times = []
        i = 0
        for frame in range(420):
            t0 = time.perf_counter()
            if g.state == wd.PLAY:
                if len(g.guesses) >= 7:
                    g.boards = [wd.Board(b.answer) for b in g.boards]
                    g._clear_round()
                g.current = pool[i % 40]
                i += 1
                g._submit()
            g.update(1 / 60)
            g.draw()
            times.append(time.perf_counter() - t0)
        times.sort()
        avg = sum(times) / len(times) * 1000
        p95 = times[int(len(times) * 0.95)] * 1000
        check(avg < 10.0 and p95 < 16.0,
              "%s: Frame im Schnitt %.2f ms, p95 %.2f ms (Budget 16,7 ms)" % (theme, avg, p95))
    ui.set_theme(theme_before)


if __name__ == "__main__":
    t0 = time.time()
    i18n.set_language("de", persist=False)
    wipe_store()
    audit_logic()
    audit_lists()
    audit_daily_logic()
    audit_endless()
    audit_daily_game()
    audit_multi()
    audit_stats_share()
    audit_setup()
    audit_layout()
    audit_performance()
    wipe_store()
    print("\n%s  (%.1f s)" % ("ALLE PRÜFUNGEN BESTANDEN" if not FAILS
                              else "%d FEHLER: %s" % (len(FAILS), FAILS),
                              time.time() - t0))
    sys.exit(1 if FAILS else 0)
