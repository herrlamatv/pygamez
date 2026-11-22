# -*- coding: utf-8 -*-
"""Headless-Audit für Lama Plinko (Kugel-Spiel mit Lama-Chips).

Geprüft wird:

Regeln   (1) 27 Tabellen: symmetrisch, nach außen nicht fallend, exakte Quote
             96,5-97,5 %, Rand bei 16 Reihen x16 / x110 / x1000,
         (2) Monte-Carlo: Fächer aus dem Fairness-Zufall folgen der Binomialverteilung,
         (3) Rest-Hundertstel-Übertrag: Einsatz 1 verliert keinen Hundertstel,
Bank     (4) Abwurf bucht Einsatz + Gewinn sofort (ein Speichern), angezeigt wird
             erst bei der Landung (pending, angezeigte Chips),
         (5) Verlauf, Fairness-Log, Statistik, Trefferzähler und Übertrag werden
             beim Abwurf gespeichert und überstehen einen Neustart,
         (6) Anzeige (Verlauf, Statistik, Log) erst bei der Landung, in Wurf-Reihenfolge,
         (7) Erfolge: Rand ab 12 Reihen, Jackpot (16 Reihen, Hoch), Regen (25 Kugeln),
             on_exit holt Erfolge fliegender Kugeln nach,
         (8) Fairness: Prüfcode vorab = sha256(Seed), Nachrechnen stimmt, neuer
             Client-Seed (gesperrt, solange Kugeln fliegen),
         (9) Pleite -> Kredit (erst wenn alle Kugeln gelandet sind),
         (10) nicht genug Chips, höchstens 50 Kugeln,
Bedienung (11) Tasten: Pfeile Reihen/Einsatz, 1-3 Risiko (gesperrt im Flug), T Turbo,
             A/S/F Fenster, Esc, Einstellungen gespeichert + vom Schema geprüft,
         (12) Leertaste halten = Dauerfeuer (Takt = Flugzeit / 30): bei 8, 12 und 16
             Reihen, mit Turbo und bei 30/60/144 Bildern/s sicher >= 25 Kugeln gleichzeitig,
         (13) Maus: WERFEN, Risiko, Reihen, Fenster-Knöpfe, Fach-Tooltip,
         (14) Auto-Modus: Anzahl, Stopp bei Gewinn/Verlust, keine Chips, STOPP/Esc,
         (15) Turbo halbiert die Flugzeit, Auflösungswechsel mit fliegenden Kugeln,
         (16) Flugbahn: reine Funktion der Zeit, gleicher Startwert = gleiche Kugel,
Layout   (17) 5 Auflösungen x 14 Sprachen (UI v4.2, v1 für de/fi): Leiste und
             Seitenleisten überlappen nicht, Texte passen, alle Zustände und
             Fenster zeichnen fehlerfrei,
Texte    (18) alle lpl-Schlüssel in 14 Sprachen, Wortfilter, Erfolgsnamen, LamaWiki.

Aufruf aus dem Repo-Root:  python tests/audit_lamaplinko.py
Exit-Code 0 = alle Prüfungen bestanden.
"""
import json
import math
import os
import random
import shutil
import sys
import tempfile
from fractions import Fraction

os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
os.environ.setdefault("SDL_AUDIODRIVER", "dummy")
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
TMP = tempfile.mkdtemp(prefix="pgz-audit-lamaplinko-")
MEM = os.path.join(TMP, "mem.json")
store._PATH = MEM
# Ebenso tabu: Statistik, Replays und eigene Level. stats.pgzlamastats (und die
# alte highscores.json) hängen an store._DIR, replay.json/ugc.json liegen neben
# ihren Modulen - alles in den Kratzordner umlenken.
store._DIR = TMP
store._LEGACY_HS_PATH = os.path.join(TMP, "highscores.json")
import replay
replay._PATH = os.path.join(TMP, "replay.json")
import ugc
ugc._PATH = os.path.join(TMP, "ugc.json")
import settings as settings_mod
SAVED = []
settings_mod.save_settings = lambda s: SAVED.append(json.loads(json.dumps(s.get("lamaplinko", {}))))
import i18n
i18n.init()
import ui

import chipstats
import fairness
import lamabank
import swear
from game_base import InputEvent
from games import lamaplinko as LP
from games import lamaplinko_draw as D
from games import lamaplinko_logic as L

GS = json.loads(json.dumps(settings_mod.DEFAULTS))
FAILS = []
RES = [(480, 360), (640, 480), (800, 600), (960, 720), (1280, 960)]
DT = 1 / 60


def check(ok, label, detail=""):
    print("  %-4s %s%s" % ("OK" if ok else "FAIL", label,
                           ("  -> " + str(detail)) if detail and not ok else ""))
    if not ok:
        FAILS.append(label)
    return ok


def fresh_mem(content=None):
    for p in (MEM, MEM + ".bak", MEM + ".tmp"):
        if os.path.exists(p):
            os.remove(p)
    if content is not None:
        with open(MEM, "w", encoding="utf-8") as f:
            json.dump(content, f)
    lamabank._data = None


SAID = []


def game(W=800, H=600, events=None, **opt):
    GS["lamaplinko"] = dict(settings_mod.DEFAULTS["lamaplinko"], **opt)
    g = LP.LamaPlinkoGame(pygame.Surface((W, H)), W, H, game_settings=GS)
    orig_say = g._say
    g._say = lambda text, secs=1.8: (SAID.append(text), orig_say(text, secs))
    g.play_sound = lambda name: None
    g.rumble = lambda ms=0: None
    g.ach_event = (lambda *a, **k: events.append(a[0])) if events is not None else (lambda *a, **k: None)
    g.toast = None
    return g


def key(g, k, repeat=False):
    g.handle_event(InputEvent(InputEvent.KEYDOWN, key=k, repeat=repeat))


def key_up(g, k):
    g.handle_event(InputEvent(InputEvent.KEYUP, key=k))


def click(g, pos, button=1):
    g.handle_event(InputEvent(InputEvent.MOUSEDOWN, pos=pos, button=button))
    g.handle_event(InputEvent(InputEvent.MOUSEUP, pos=pos, button=button))


def run(g, seconds):
    for _ in range(int(round(seconds / DT))):
        g.update(DT)


def land_all(g):
    run(g, LP.flight_time(g.rows) + 0.1)


class forced_path:
    """Erzwingt den Weg der nächsten Kugeln (z.B. ganz links = äußerstes Fach)."""

    def __init__(self, fn):
        self.fn = fn

    def __enter__(self):
        self.orig = L.drop_path
        L.drop_path = lambda rand, rows: self.fn(rows)

    def __exit__(self, *a):
        L.drop_path = self.orig


def left_edge(rows):
    return [0] * rows


def middle(rows):
    return [i % 2 for i in range(rows)]


# ================================================================ Regeln
print("\n[Regeln]")
bad = []
for risk in L.RISKS:
    for rows in L.ROWS:
        tb = L.table(rows, risk)
        half = rows // 2
        ok = len(tb) == rows + 1 and tuple(reversed(tb)) == tb \
            and all(tb[i] >= tb[i + 1] for i in range(half)) \
            and Fraction(965, 1000) <= L.expected_return(rows, risk) <= Fraction(975, 1000)
        if not ok:
            bad.append((risk, rows))
check(len(L.TABLES) == 3 and all(len(L.TABLES[r]) == 9 for r in L.RISKS) and not bad,
      "(1) 27 Tabellen: symmetrisch, nach außen nicht fallend, Quote 96,5-97,5 %", bad)
check([L.mult100(16, r, 0) for r in L.RISKS] == [1600, 11000, 100000], "(1) Rand bei 16 Reihen x16 / x110 / x1000")
counts = [0] * 13
N = 20000
for nonce in range(N):
    counts[L.slot_of(L.path_from_seed(12, "0123456789abcdef0123456789abcdef", "audit234", nonce))] += 1
mc_bad = []
for i, c in enumerate(counts):
    p = float(L.prob(12, i))
    se = math.sqrt(p * (1 - p) / N)
    if abs(c / N - p) > 5 * se + 1e-4:
        mc_bad.append((i, c / N, p))
check(not mc_bad, "(2) Monte-Carlo: 20000 Kugeln (12 Reihen) folgen der Binomialverteilung", mc_bad)
carry, paid, want = 0, 0, 0
rng = random.Random(7)
for _ in range(5000):
    rows, risk = rng.choice(L.ROWS), rng.choice(L.RISKS)
    slot = rng.randint(0, rows)
    win, carry = L.payout_carry(1, rows, risk, slot, carry)
    paid += win
    want += L.mult100(rows, risk, slot)
check(paid * 100 + carry == want and 0 <= carry < 100, "(3) Übertrag: bei Einsatz 1 geht kein Hundertstel verloren",
      (paid, carry, want))

# ================================================================ Bank + Ablauf
print("\n[Bank + Ablauf]")
fresh_mem({"mem": {"lang": "de"}})
ev = []
g = game(events=ev, rows=12, risk="medium", bet=10)
check(g.rows == 12 and g.risk == "medium" and g.bet == 10 and lamabank.balance() == 1000 and not g.balls,
      "(4) Start: 12 Reihen, Mittel, Einsatz 10, 1000 Chips")
nxt = fairness.commit(g.fair["server"])
nonce0 = g.fair["nonce"]
with forced_path(middle):
    key(g, "space")
key_up(g, "space")
b = g.balls[0]
check(len(g.balls) == 1 and b["slot"] == 6 and b["win"] == 10 * L.mult100(12, "medium", 6) // 100,
      "(4) Leertaste wirft eine Kugel (Fach 6 = x0,4)")
check(lamabank.balance() == 1000 - 10 + b["win"] and g.pending == b["win"],
      "(4) Einsatz und Gewinn sofort gebucht, Gewinn noch 'unterwegs'")
run(g, 1.0)
check(abs(g.shown_chips - (1000 - 10)) < 0.6, "(4) angezeigt: nur der Einsatz ist weg", g.shown_chips)
extra = lamabank.get_extra("lamaplinko")
check(extra["hist"] == [40] and extra["fairlog"][-1]["o"]["slot"] == 6 and extra["stats"]["rounds"] == 1
      and extra["stats"]["x"]["hits"]["12"][6] == 1 and extra["fair"]["nonce"] == nonce0 + 1
      and extra["carry"] == 0, "(5) beim Abwurf gespeichert: Verlauf, Log, Statistik, Treffer, Nonce, Übertrag")
check(extra["fairlog"][-1]["h"] == nxt and extra["fairlog"][-1]["n"] == nonce0,
      "(8) der vorab gezeigte Prüfcode gehört genau zu dieser Kugel")
check(g.hist_shown == [] and g.stats_shown["rounds"] == 0 and g.fairlog_shown == [],
      "(6) Anzeige noch leer, solange die Kugel fliegt")
while g.balls:
    g.update(DT)
check(not g.balls and g.pending == 0 and g.hist_shown == [40] and g.stats_shown["rounds"] == 1
      and g.fairlog_shown[-1]["n"] == nonce0 and g.slot_bump[6] > 0,
      "(6) Landung: Verlauf, Statistik, Log; das Fach wippt")
run(g, 1.0)
check(g.shown_chips == lamabank.balance(), "(4) nach der Landung zeigt der Kontostand den Gewinn")
check(lamabank.ledger("lamaplinko") == b["win"] - 10 and lamabank.ledger("casino") == 0,
      "(4) Bilanz nur bei Lama Plinko")
lamabank._data = None
lamabank.load()
check(lamabank.ledger("lamaplinko") == b["win"] - 10, "(5) Bilanz übersteht load() (lamabank.GAMES)")
g2 = game()
check(g2.hist == [40] and g2.hist_shown == [40] and g2.stats_shown["rounds"] == 1
      and g2.fair["nonce"] == nonce0 + 1 and len(g2.fairlog_shown) == 1,
      "(5) Neustart: alles wieder da")

# Reihenfolge und Übertrag bei Einsatz 1
fresh_mem({"mem": {"lang": "de"}})
g = game(rows=8, risk="low", bet=1)
for _ in range(5):
    g._drop()
    run(g, 0.05)
order = [x["log"]["n"] for x in g.balls]
land_all(g)
check([e["n"] for e in g.fairlog_shown] == order and len(g.hist_shown) == 5,
      "(6) Landung in Wurf-Reihenfolge")
want = sum(e["o"]["m"] for e in g.fairlog)
check(sum(e["o"]["win"] for e in g.fairlog) * 100 + g.carry == want,
      "(3) Spiel nutzt den Übertrag (Einsatz 1, 5 Kugeln)", (g.carry, want))

# Erfolge
fresh_mem({"mem": {"lang": "de"}})
ev = []
g = game(events=ev, rows=11, risk="high", bet=1)
with forced_path(left_edge):
    g._drop()
land_all(g)
check(ev == [], "(7) 11 Reihen, äußerstes Fach: noch kein Randtreffer", ev)
g = game(events=ev, rows=12, risk="low", bet=1)
with forced_path(left_edge):
    g._drop()
run(g, 0.5)
check(ev == [], "(7) Erfolg erst bei der Landung", ev)
land_all(g)
check(ev == ["lpl_edge"] and g.banner is None, "(7) 12 Reihen: Randtreffer", ev)
ev.clear()
g = game(events=ev, rows=16, risk="high", bet=1)
bal = lamabank.balance()
with forced_path(left_edge):
    g._drop()
land_all(g)
check(ev == ["lpl_edge", "lpl_jackpot"] and lamabank.balance() == bal - 1 + 1000 and g.banner is not None
      and g.banner["title"] == i18n.t("cas.s.jackpot"), "(7) 16 Reihen, Hoch: Jackpot x1000 mit Banner", ev)
ev.clear()
g = game(events=ev, rows=16, risk="high", bet=1)
with forced_path(left_edge):
    g._drop()
    g._drop()
g.on_exit()
check(ev == ["lpl_edge", "lpl_jackpot", "lpl_edge", "lpl_jackpot"] and not g.balls,
      "(7) on_exit holt Erfolge fliegender Kugeln nach", ev)
ev.clear()
g = game(events=ev, rows=16, risk="low", bet=1)
for _ in range(24):
    g._drop()
check("lpl_rain" not in ev, "(7) 24 Kugeln: noch kein Regen")
g._drop()
check(ev.count("lpl_rain") == 1 and len(g.balls) == 25, "(7) 25 Kugeln gleichzeitig: Wollknäuel-Regen")
land_all(g)

# Fairness
fresh_mem({"mem": {"lang": "de"}})
g = game(rows=14, risk="medium", bet=2)
for _ in range(15):
    g._drop()
    run(g, 0.03)
client = g.fair["client"]
key(g, "f")
check(g.panel == "fair" and g.wants_escape, "(8) F öffnet das Fairness-Fenster")
g._new_client()
check(g.fair["client"] == client, "(8) neuer Client-Seed gesperrt, solange Kugeln fliegen")
land_all(g)
check(len(g.fairlog_shown) == fairness.LOG and all(fairness.verify(e) for e in g.fairlog_shown),
      "(8) Log: letzte 12 Kugeln, Prüfcode = sha256(Seed)")
ok = True
for e in g.fairlog_shown:
    path = L.path_from_seed(e["o"]["rows"], e["s"], e["c"], e["n"])
    ok = ok and L.slot_of(path) == e["o"]["slot"] and L.mult100(14, "medium", e["o"]["slot"]) == e["o"]["m"]
check(ok, "(8) Ergebnis jeder Kugel lässt sich aus Seed, Client-Seed und Nr. nachrechnen")
key(g, "Return")
sel = g._fair_entry()
check(g.fair_checks.get(sel["n"], (0, 0))[:2] == (True, True) and sel["n"] == g.fairlog_shown[-1]["n"],
      "(8) Nachrechnen (Enter) der neuesten Kugel: stimmt")
key(g, "Down")
key(g, "Return")
check(g.fair_sel == 1 and g.fair_checks.get(g._fair_entry()["n"], (0, 0))[:2] == (True, True),
      "(8) Pfeil wählt die nächste Kugel, auch sie stimmt")
bad_e = dict(g.fairlog_shown[0])
bad_e["h"] = "0" * 64
g._verify(bad_e)
check(g.fair_checks[bad_e["n"]][0] is False, "(8) falscher Prüfcode wird erkannt")
key(g, "c")
check(g.fair["client"] != client and fairness.valid_client(g.fair["client"])
      and lamabank.get_extra("lamaplinko")["fair"]["client"] == g.fair["client"],
      "(8) C = neuer Client-Seed (gespeichert)")
key(g, "Escape")
check(g.panel is None and not g.wants_escape, "(8) Esc schließt das Fenster")

# Pleite
fresh_mem({"mem": {"lang": "de"}})
lamabank.load()
lamabank._data["chips"] = 0
lamabank._save()
g = game()
check(g.broke, "(9) 0 Chips = pleite")
key(g, "Return")
check(not g.broke and lamabank.balance() == 1000 and lamabank.refills() == 1, "(9) Kredit auf 1000")
fresh_mem({"mem": {"lang": "de"}})
lamabank.load()
lamabank._data["chips"] = 2
lamabank._save()
g = game(rows=8, risk="high", bet=1)
with forced_path(middle):
    g._drop()
    g._drop()
check(not g.broke and lamabank.balance() == 2 - 2 + 2 * (L.mult100(8, "high", 4) // 100), "(9) letzte Chips geworfen")
check(not g._drop(quiet=True), "(10) ohne Chips keine weitere Kugel")
land_all(g)
check(g.broke, "(9) erst nach der Landung: pleite")
fresh_mem({"mem": {"lang": "de"}})
g = game(rows=16, bet=1)
for _ in range(60):
    g._drop(quiet=True)
check(len(g.balls) == L.MAX_BALLS == 50, "(10) höchstens 50 Kugeln gleichzeitig", len(g.balls))
g.bet = 1000
lamabank._data["chips"] = 500
check(not g._drop(), "(10) Einsatz über dem Guthaben: kein Wurf")
land_all(g)

# ================================================================ Bedienung
print("\n[Bedienung]")
fresh_mem({"mem": {"lang": "de"}})
SAVED.clear()
g = game(rows=12, risk="medium", bet=10)
key(g, "Right")
key(g, "Right")
key(g, "3")
key(g, "Up")
check(g.rows == 14 and g.risk == "high" and g.bet == 20, "(11) Pfeile = Reihen/Einsatz, 3 = Hoch")
check(SAVED and SAVED[-1]["rows"] == 14 and SAVED[-1]["risk"] == "high" and SAVED[-1]["bet"] == 20,
      "(11) Einstellungen gespeichert", SAVED[-1:])
g._drop()
key(g, "Left")
key(g, "1")
check(g.rows == 14 and g.risk == "high" and len(g.slot_rects) == 15, "(11) Reihen + Risiko gesperrt, solange Kugeln fliegen")
key(g, "Down")
check(g.bet == 10, "(11) Einsatz je Kugel bleibt im Flug änderbar")
land_all(g)
key(g, "t")
check(g.turbo and SAVED[-1]["turbo"] is True, "(11) T = Turbo (gespeichert)")
key(g, "t")
for k_, name in (("a", "auto"), ("s", "stats"), ("f", "fair")):
    key(g, k_)
    ok = g.panel == name
    key(g, k_)
    check(ok and g.panel is None, "(11) %s öffnet und schließt %s" % (k_.upper(), name))
check(g.controls["p1"]["left"] == "a" and g.controls["p1"]["down"] == "s",
      "(11) ... obwohl A/S bei der Standardbelegung (WASD) Richtungstasten sind")
g.controls = {"p1": {"action": "a", "left": "f", "right": "d"}, "p2": {"action": "x"}}
n0 = len(g.balls)
key(g, "a")
ok_a = g.panel == "auto"
key(g, "a")
key(g, "x")
key(g, "d")
check(ok_a and g.panel is None and len(g.balls) == n0 and g.rows == 14,
      "(11) Tasten roh: belegte p1/p2-Tasten ändern nichts (A bleibt Auto, keine Würfe/Reihen)")
g.controls = settings_mod.DEFAULT_CONTROLS
land_all(g)
merged = settings_mod._merge_defaults({"lamaplinko": {"rows": 40, "risk": "extreme", "bet": 3, "turbo": 1,
                                                      "auto_balls": 7, "auto_win": 100, "auto_loss": 9}})
check(merged["lamaplinko"] == {"rows": 16, "risk": "medium", "bet": 10, "turbo": False, "auto_balls": 25,
                               "auto_win": 100, "auto_loss": 0},
      "(11) Schema begrenzt/verwirft ungültige Werte", merged.get("lamaplinko"))

# Dauerfeuer
fresh_mem({"mem": {"lang": "de"}})
g = game(rows=16, bet=1)
key(g, "space")
check(len(g.balls) == 1, "(12) Druck = sofort eine Kugel")
for _ in range(10):
    key(g, "space", repeat=True)
check(len(g.balls) == 1, "(12) Tastenwiederholung wirft nicht selbst")
run(g, 1.0)
n_hold = len(g.balls)
key_up(g, "space")
run(g, 0.5)
want = 1 + int((1.0 - LP.HOLD_DELAY) / LP.fire_rate(16)) + 1
check(abs(n_hold - want) <= 1 and len(g.balls) == n_hold, "(12) gehalten: Dauerfeuer im Takt, Loslassen stoppt",
      (n_hold, want))
land_all(g)
check(abs(LP.fire_rate(16) - 2.30 / 30) < 1e-9 and abs(LP.fire_rate(8) - 1.62 / 30) < 1e-9
      and abs(LP.flight_time(16) - 2.30) < 1e-9 and abs(LP.flight_time(8) - 1.62) < 1e-9,
      "(12) Takt = Flugzeit / 30 (16 Reihen 0,077 s, 8 Reihen 0,054 s)")
for rows, turbo, fps in ((16, False, 60), (8, False, 60), (16, True, 60), (16, False, 30), (12, False, 144)):
    fresh_mem({"mem": {"lang": "de"}})
    ev = []
    g = game(events=ev, rows=rows, bet=1, turbo=turbo)
    key(g, "space")
    peak = 0
    for _ in range(int(4.0 * fps)):
        g.update(1.0 / fps)
        peak = max(peak, len(g.balls))
    key_up(g, "space")
    check(L.RAIN_BALLS <= peak <= L.MAX_BALLS and "lpl_rain" in ev,
          "(12) Leertaste halten, %d Reihen%s, %d Bilder/s: %d Kugeln gleichzeitig -> Wollknäuel-Regen"
          % (rows, " + Turbo" if turbo else "", fps, peak), (peak, ev[:3]))
    land_all(g)

# Maus
fresh_mem({"mem": {"lang": "de"}})
g = game(rows=12, risk="medium", bet=10)
click(g, g.r_risks[0].center)
click(g, g.r_rows_p.center)
click(g, g.r_plus.center)
check(g.risk == "low" and g.rows == 13 and g.bet == 20, "(13) Klick: Risiko, Reihen, Einsatz")
click(g, g.r_drop.center)
check(len(g.balls) == 1 and not g.hold, "(13) Klick auf WERFEN")
for i, name in enumerate(("auto", "stats", "fair")):
    click(g, g.r_icons[i].center)
    ok = g.panel == name
    click(g, g.r_close.center)
    check(ok and g.panel is None, "(13) Knopf %s öffnet, X schließt" % name)
click(g, g.r_icons[3].center)
check(g.turbo, "(13) Knopf Turbo")
g.mouse = g.slot_rects[0].center
g.draw()
check(g._hover_slot() == 0, "(13) Maus über Fach 0: Tooltip")
land_all(g)

# Auto
fresh_mem({"mem": {"lang": "de"}})
g = game(rows=8, risk="low", bet=10, auto_balls=10, turbo=False)
key(g, "a")
key(g, "Return")
check(g.auto is not None and g.panel is None and g.wants_escape, "(14) Auto startet (Enter im Fenster)")
key(g, "Up")
check(g.bet == 10, "(14) Einsatz im Auto gesperrt")
SAID.clear()
run(g, 12)
check(g.auto is None and g.stats_shown["rounds"] == 10 and not g.balls
      and SAID == [i18n.t("chip.auto_done")], "(14) genau 10 Kugeln, dann Hinweis", (g.stats_shown["rounds"], SAID))
GS["lamaplinko"]["auto_win"] = 100
g = game(rows=16, risk="high", bet=10, auto_balls=100, auto_win=100)
with forced_path(left_edge):
    g._start_auto()
    run(g, 0.05)
check(g.auto is None and len(g.balls) == 1, "(14) Gewinnziel: stoppt sofort nach der Kugel, die es erreicht")
check(g.toast is None or g.toast[0] != i18n.t("chip.auto_hit_win"), "(14) Hinweis erst bei der Landung (kein Verraten)")
land_all(g)
check(g.toast and g.toast[0] == i18n.t("chip.auto_hit_win"), "(14) Landung: 'Gewinnziel erreicht'")
fresh_mem({"mem": {"lang": "de"}})
g = game(rows=16, risk="high", bet=50, auto_balls=100, auto_win=0, auto_loss=500)
with forced_path(middle):
    g._start_auto()
    run(g, 6)
check(g.auto is None and g.stats["rounds"] == 13,
      "(14) Verlustgrenze 500 bei Einsatz 50 (x0,2 = -40 je Kugel): Stopp nach 13 Kugeln", g.stats["rounds"])
fresh_mem({"mem": {"lang": "de"}})
lamabank.load()
lamabank._data["chips"] = 30
lamabank._save()
g = game(rows=8, risk="high", bet=10, auto_balls=100, auto_loss=0)
SAID.clear()
with forced_path(middle):
    g._start_auto()
    run(g, 8)
check(g.auto is None and not g.balls and g.stats["rounds"] == 3 and SAID == [i18n.t("chip.auto_no_chips")]
      and lamabank.balance() == 6 and g.bet == 5, "(14) zu wenig Chips: Auto stoppt (erst wenn alles gelandet ist)", SAID)
fresh_mem({"mem": {"lang": "de"}})
g = game(rows=8, bet=1, auto_balls=1000)
g._start_auto()
run(g, 1.0)
key(g, "Escape")
check(g.auto is None and g.balls, "(14) Esc stoppt das Auto, fliegende Kugeln landen noch")
g._start_auto()
run(g, 0.5)
key(g, "space")
check(g.auto is None, "(14) Leertaste/STOPP stoppt das Auto")
land_all(g)

# Turbo + Auflösung
fresh_mem({"mem": {"lang": "de"}})
g = game(rows=16, bet=1, turbo=True)
g._drop()
run(g, LP.flight_time(16) / 2 + 0.05)
check(not g.balls, "(15) Turbo: Landung nach der halben Flugzeit")
g = game(rows=16, bet=1, turbo=False)
n_hist = len(g.hist_shown)
for _ in range(10):
    g._drop()
    run(g, 0.1)
before = [x["t"] for x in g.balls]
g.surface = pygame.Surface((480, 360))
g.width, g.height = 480, 360
g.on_surface_changed()
g.draw()
pos_ok = all(g.board_box.inflate(40, 80).collidepoint(g._ball_pos(x, x["t"])) for x in g.balls)
check([x["t"] for x in g.balls] == before and pos_ok, "(15) Auflösungswechsel mit fliegenden Kugeln")
land_all(g)
check(not g.balls and len(g.hist_shown) == min(L.HISTORY, n_hist + 10), "(15) danach landen alle normal")

# Flugbahn
g = game(rows=12)
g.rng = random.Random(42)
g._drop()
b1 = g.balls[-1]
g2 = game(rows=12)
g2.rng = random.Random(42)
g2.fair = dict(g.fair)
g2.fair["nonce"] -= 1
g2.fair["server"] = b1["log"]["s"]
g2.fair["client"] = b1["log"]["c"]
g2.rng = random.Random(42)
g2._drop()
b2 = g2.balls[-1]
check(b1["path"] == b2["path"] and b1["jit"] == b2["jit"] and b1["hb"] == b2["hb"],
      "(16) gleicher Startwert + gleiche Seeds = gleiche Kugel (Rust/Web-Vergleich)")
p0 = g._ball_pos(b1, 0.0)
st = LP.seg_time(12)
ends_ok = True
for r in range(1, 12):
    x, y = g._ball_pos(b1, LP.DROP_T + r * st - 1e-9)
    cx, cy = g._contact(r, b1["pre"][r], b1["jit"][r])
    ends_ok = ends_ok and abs(x - cx) < 0.05 and abs(y - cy) < 0.05
check(abs(p0[0] - g.mouth[0]) < 1e-9 and ends_ok, "(16) Bogen endet genau auf dem nächsten Stift")
land_all(g)
land_all(g2)


# ================================================================ Layout
def inside(r, area):
    return area.contains(pygame.Rect(r))


def no_overlap(rects):
    rs = [pygame.Rect(r) for r in rects]
    for i, a in enumerate(rs):
        for b in rs[i + 1:]:
            if a.colliderect(b):
                return False
    return True


print("\n[Layout]")
problems = []
for theme in ("v42", "v1"):
    ui.set_theme(theme)
    for code, _ in i18n.AVAILABLE:
        if theme == "v1" and code not in ("de", "fi"):
            continue
        i18n.set_language(code, persist=False)
        for (W, H) in RES:
            tag = "%s/%s/%dx%d" % (theme, code, W, H)
            try:
                fresh_mem({"mem": {"lang": code}})
                g = game(W, H, rows=16, risk="high", bet=10)
                screen = pygame.Rect(0, 0, W, H)
                strip = [g.r_minus, g.r_bet, g.r_plus, g.r_drop] + g.r_icons
                if not no_overlap(strip) or not all(inside(r, g.strip) for r in strip):
                    problems.append((tag, "Leiste überlappt"))
                side = g.r_risks + [g.r_rows_m, g.r_rows_v, g.r_rows_p]
                if not no_overlap(side) or not all(inside(r, g.left) for r in side):
                    problems.append((tag, "linke Leiste"))
                for rows in (8, 16):
                    g.rows = rows
                    g._board_layout()
                    bb = g.board_box
                    ledge_l = g.cx - g.gap * 0.55 - g.llama_n * 1.05
                    if bb.left < g.left.right or bb.right > g.right.left or g.slot_rects[-1].bottom + 3 > g.strip.y \
                            or ledge_l < g.left.right or g.foot_y - g.llama_n * D.LLAMA_PIVOT[1] < g.hud_h:
                        problems.append((tag, "Brett", rows, bb, g.left.right, g.right.left))
                g.rows = 16
                g._board_layout()
                for lbl, r in ((i18n.t("lpl.drop"), g.r_drop), (i18n.t("chip.auto_stop"), g.r_drop)):
                    if min(g.f_btn.size(lbl)[0], g.f_tiny.size(lbl)[0]) > r.w - 4:
                        problems.append((tag, "Knopf", lbl))
                for i, risk in enumerate(L.RISKS):
                    lbl = "%d  %s" % (i + 1, i18n.t("lpl.risk." + risk))
                    if g.f_tiny.size(lbl)[0] > g.r_risks[i].w - 4:
                        problems.append((tag, "Risiko", lbl))
                info_w = g.left.w - 2 * max(5, int(9 * g.k))
                for txt in (i18n.t("lpl.max", m="1000"), i18n.t("lpl.rtp", p="96,99")):
                    if g.f_tiny.size(txt)[0] > info_w * 1.25:
                        problems.append((tag, "Info", txt))
                for txt in (i18n.t("cas.not_enough"), i18n.t("lpl.locked"), i18n.t("chip.auto_hit_win"),
                            i18n.t("chip.auto_no_chips"), i18n.t("lpl.max_balls", n=50)):
                    if g.f_tiny.size(txt)[0] + 24 > W:
                        problems.append((tag, "Hinweis", txt))
                if g.right.bottom > g.strip.y or g.left.bottom > g.strip.y:
                    problems.append((tag, "Seitenleisten"))
                # alle Zustände zeichnen
                g.draw()
                for _ in range(12):
                    g._drop(quiet=True)
                    run(g, 0.07)
                g.draw()
                g.mouse = g.slot_rects[0].center
                g.draw()
                g.mouse = g.r_icons[1].center
                g.draw()
                run(g, LP.flight_time(16) - 0.6)
                g.draw()
                with forced_path(left_edge):
                    g._drop()
                land_all(g)
                run(g, 0.2)
                g.draw()
                for name in ("auto", "stats", "fair"):
                    g._open(name)
                    if not inside(g.pan, screen):
                        problems.append((tag, "Fenster", name))
                    g.mouse = g.pan.center
                    g.draw()
                    if name == "fair":
                        key(g, "Return")
                        g.draw()
                        geo = g._fair_geom()
                        if geo["vis"] < 4 or geo["r_ver"].y < geo["preview"].bottom + 4:
                            problems.append((tag, "Fairness-Fenster eng", geo["vis"]))
                    if name == "auto":
                        (rows_, turbo, sum_y, start), _ = g._auto_rects()
                        if start.bottom > g.pan.bottom or start.y < sum_y + g.f_small.get_height():
                            problems.append((tag, "Auto-Fenster"))
                        for ri, (ly, rects) in enumerate(rows_):
                            for r, v in rects:
                                lbl = str(v) if (ri == 0 or v) else i18n.t("chip.auto_off")
                                if g.f_tiny.size(lbl)[0] > r.w - 4:
                                    problems.append((tag, "Auto-Wahl", lbl))
                    g._open(name)
                g._start_auto()
                run(g, 1.0)
                g._open("auto")
                g.draw()
                g._open("auto")
                g._stop_auto(None)
                land_all(g)
                g.toast = [i18n.t("lpl.hint"), 1.0]
                g.draw()
                g.broke = True
                g.draw()
                pw = min(W - 40, int(480 * max(0.8, g.k)))
                for txt in (i18n.t("cas.broke_sub"), i18n.t("cas.broke_restart", n=1000)):
                    if g.f_tiny.size(txt)[0] > pw - 16:
                        problems.append((tag, "Pleite-Text", txt))
            except Exception as e:  # noqa: BLE001
                import traceback
                traceback.print_exc()
                problems.append((tag, "Ausnahme", repr(e)))
ui.set_theme("v42")
i18n.set_language("de", persist=False)
check(not problems, "(17) 5 Auflösungen x 14 Sprachen (+ UI v1): Layout + alle Zustände", problems[:6])

# ================================================================ Texte
print("\n[Texte]")
langs = {}
for code, _ in i18n.AVAILABLE:
    sub = "" if code in ("de", "en", "fr", "es", "pt") else "lang.expansion"
    with open(os.path.join(REPO, "lang", sub, code + ".json"), encoding="utf-8") as f:
        langs[code] = json.load(f)
src = open(os.path.join(REPO, "games", "lamaplinko.py"), encoding="utf-8").read()
import re
used = sorted(set(k for k in re.findall(r'"((?:lpl|chip|cas)\.[a-z_.]+)"', src) if not k.endswith(".")))
used += ["lpl.risk." + r for r in L.RISKS]
used += ["ach.%s.%s" % (a, f) for a in ("lpl_edge", "lpl_rain", "lpl_jackpot") for f in ("name", "desc")]
missing = [(c, k) for c in langs for k in used if not langs[c].get(k)]
check(len(used) >= 40 and not missing, "(18) %d benutzte Schlüssel in allen 14 Sprachen" % len(set(used)),
      missing[:5])
own = sorted(k for k in langs["de"] if k.startswith("lpl."))
dirty = [(c, k) for c in langs for k in own if not swear.is_clean(langs[c].get(k, ""))]
check(own and not dirty, "(18) %d lpl-Texte bestehen den Wortfilter" % len(own), dirty[:5])
wiki_missing = []
for code, _ in i18n.AVAILABLE:
    sub = "" if code in ("de", "en", "fr", "es", "pt") else "lang.expansion"
    with open(os.path.join(REPO, "lamawiki", sub, code + ".json"), encoding="utf-8") as f:
        pages = {p["id"]: p for p in json.load(f)["pages"]}
    if pages.get("lamaplinko", {}).get("game") != "LamaPlinkoGame":
        wiki_missing.append(code)
if len(wiki_missing) == len(langs):
    print("  --   (18) LamaWiki-Seite noch nicht eingespielt (Agent WIKI-PLINKO) - übersprungen")
else:
    check(not wiki_missing, "(18) LamaWiki-Seite in 14 Sprachen", wiki_missing)

shutil.rmtree(TMP, ignore_errors=True)
print("\n%s - %d Fehler" % ("BESTANDEN" if not FAILS else "FEHLGESCHLAGEN", len(FAILS)))
sys.exit(1 if FAILS else 0)
