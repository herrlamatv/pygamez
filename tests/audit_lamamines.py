# -*- coding: utf-8 -*-
"""Headless-Audit für Lama Mines (Edelstein-Suche mit Lama-Chips).

Geprüft wird:

Regeln    (1) Multiplikatoren steigen mit jedem Edelstein, nie unter x1,01,
              Deckel x100.000; die Quote jeder Auszahl-Strategie liegt bei
              96-98,5 % (gedeckelte Fälle darunter); Anzeige-Formate,
          (2) Monte-Carlo: jedes Feld ist gleich oft eine Mine, das Feld
              hängt nur an Server-Seed, Client-Seed und Rundennummer,
Bank      (3) SPIELEN bucht den Einsatz ab und speichert die Runde (Seeds,
              Nonce, Feld, Minen, Einsatz, offene Plätze); jedes Feld wird
              sofort gespeichert; Auszahlen schreibt gut (nur Lama-Mines-Bilanz),
          (4) Mine: Einsatz weg, Explosion + Wackeln, danach ist das ganze Feld
              abgedunkelt aufgedeckt (getroffene Mine hervorgehoben),
          (5) Verlassen mitten in der Runde: neuer Start macht genau dort weiter
              (Feld aus dem Seed neu berechnet),
          (6) volles Feld / Höchstgewinn: automatisch ausgezahlt + Erfolge
              lmi_clear, lmi_brave (ab x10), lmi_legend (ab x1000),
          (7) Pleite -> Kredit auf 1000,
Fairness  (8) Prüfcode steht vor der Runde fest und passt zum danach gezeigten
              Seed; Nachrechnen stimmt (auch Verluste), Manipulation fällt auf;
              neuer Client-Seed nur außerhalb einer Runde; Log höchstens 12,
Statistik (9) Runden, Gewinne, Verluste, Einsatz, Auszahlung, Edelsteine,
              Minen, abgeräumte Felder; Verlauf höchstens 12,
Auto      (10) Markierungen je Feldgröße gespeichert, Auto deckt genau diese auf,
              zahlt aus, stoppt nach N Runden, bei Gewinn-/Verlustgrenze, ohne
              Chips und mit Esc/STOPP; zu viele Markierungen werden abgelehnt,
Bedienung (11) Tasten: 1-4, Pfeile, Leertaste, Enter/C, R, A/S/F/T, Esc;
              Einstellungen gespeichert und vom Schema geprüft,
          (12) Maus: Felder, Knöpfe, Schnellwahl, Zufallsfeld, Fenster,
Layout    (13) 5 Auflösungen x 14 Sprachen (UI v4.2, dazu v1 in de/fi): Leisten
              und Knöpfe überlappen nicht, Texte passen, alle Zustände zeichnen
              fehlerfrei; Auflösungswechsel mitten in der Runde,
Texte     (14) alle lmi-Schlüssel in 14 Sprachen, Wortfilter, LamaWiki-Seite.

Aufruf aus dem Repo-Root:  python tests/audit_lamamines.py
Exit-Code 0 = alle Prüfungen bestanden. Speicherstand/Einstellungen landen nur
in einem Temp-Ordner (PGZ_AUDIT_DIR, sonst tempfile).
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
TMP = os.environ.get("PGZ_AUDIT_DIR") or tempfile.mkdtemp(prefix="pgz-lmi-")
os.makedirs(TMP, exist_ok=True)
MEM = os.path.join(TMP, "_audit-lamamines-mem.json")
store._PATH = MEM
# Ebenso tabu: Statistik, Replays und eigene Level. stats.pgzlamastats (und die
# alte highscores.json) hängen an store._DIR, replay.json/ugc.json liegen neben
# ihren Modulen - alles in einen eigenen (je Lauf frischen) Kratzordner in TMP.
store._DIR = tempfile.mkdtemp(prefix="daten-", dir=TMP)
store._LEGACY_HS_PATH = os.path.join(store._DIR, "highscores.json")
import replay
replay._PATH = os.path.join(store._DIR, "replay.json")
import ugc
ugc._PATH = os.path.join(store._DIR, "ugc.json")
import settings as settings_mod
SAVED = []
settings_mod.save_settings = lambda s: SAVED.append(json.loads(json.dumps(s.get("lamamines", {}))))
import i18n
i18n.init()
import ui
import audio
audio.tone = lambda *a, **k: None

import chipstats
import fairness
import lamabank
import swear
from game_base import InputEvent
from games import lamamines as LM
from games import lamamines_draw as D
from games import lamamines_logic as ML

GS = json.loads(json.dumps(settings_mod.DEFAULTS))
FAILS = []
RES = [(480, 360), (640, 480), (800, 600), (960, 720), (1280, 960)]


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
    if content is None:
        content = {"mem": {"lang": "de"}}
    with open(MEM, "w", encoding="utf-8") as f:
        json.dump(content, f)
    lamabank._data = None


def game(W=800, H=600, events=None, **opts):
    GS["lamamines"] = dict(settings_mod.DEFAULTS["lamamines"], **opts)
    g = LM.LamaMinesGame(pygame.Surface((W, H)), W, H, game_settings=GS)
    g.play_sound = lambda name: None
    g.rumble = lambda ms=0: None
    g.ach_event = (lambda *a, **k: events.append(a[0])) if events is not None else (lambda *a, **k: None)
    return g


def key(g, k, repeat=False):
    g.handle_event(InputEvent(InputEvent.KEYDOWN, key=k, repeat=repeat))


def click(g, pos):
    g.handle_event(InputEvent(InputEvent.MOUSEDOWN, pos=pos, button=1))


def run(g, seconds, dt=1 / 60):
    for _ in range(int(seconds / dt)):
        g.update(dt)


def safe(g, n=1):
    """Die ersten n verdeckten sicheren Plätze."""
    return [i for i in range(len(g.cells)) if not g.is_mine[i] and g.cells[i]["face"] == "hidden"][:n]


def extra():
    return lamabank.get_extra("lamamines", {})


# ================================================================ Regeln
print("\n[Regeln]")
bad_rise, bad_ret, bad_cap = [], [], []
for grid in ML.GRIDS:
    n = grid * grid
    for mines in range(1, n):
        prev = 100
        for k in range(1, n - mines + 1):
            m = ML.mult100(grid, mines, k)
            if not (m > prev or (m == prev == ML.MAX_MULT100)) or (k == 1 and m < 101):
                bad_rise.append((grid, mines, k, prev, m))
            if m > ML.MAX_MULT100:
                bad_cap.append((grid, mines, k, m))
            r = Fraction(math.comb(n - k, mines), math.comb(n, mines)) * Fraction(m, 100)
            raw = min(ML.MAX_MULT100, 97 * math.comb(n, mines) // math.comb(n - k, mines))
            if m == raw and m < ML.MAX_MULT100 and not (Fraction(96, 100) <= r <= Fraction(97, 100)):
                bad_ret.append((grid, mines, k, "rein", float(r)))
            if m != raw and not (Fraction(96, 100) <= r <= Fraction(99, 100)):
                bad_ret.append((grid, mines, k, "angehoben", float(r)))
            if r > Fraction(99, 100):
                bad_ret.append((grid, mines, k, float(r)))
            prev = m
check(not bad_rise, "(1) Multiplikator steigt mit jedem Edelstein (erster Stein mind. x1,01)", bad_rise[:4])
check(not bad_cap, "(1) Deckel x100.000", bad_cap[:4])
check(not bad_ret, "(1) Quote: Formel 96-97 %, angehobene Stufen (mind. +0,01) höchstens 99 %, "
                  "gedeckelte darunter", bad_ret[:4])
check([ML.mult100(5, 3, k) for k in range(0, 4)][:1] == [100] and ML.payout(100, 5, 3, 0) == 100
      and ML.payout(10, 7, 24, 25) == 10 * ML.MAX_MULT100 // 100, "(1) Auszahlung = Einsatz x m100 // 100")
fresh_mem()
g = game()
check(g._fmt(110) == "×1,10" and g._fmt(223100) == "×2.231,00" and g._fmt(10000000) == "×100.000"
      and g._fmt_short(13580) == "×135,8" and g._fmt_short(176540) == "×1765" and g._fmt_short(2345678) == "×23,4k"
      and g._group(1000000) == "1.000.000" and g._group(-400) == "-400",
      "(1) Anzeige: Multiplikatoren und Beträge mit Trennzeichen (de)")
check(ML.all_safe_percent(5, 3, 1) == 880 and ML.all_safe_percent(5, 3, 0) == 1000
      and ML.all_safe_percent(5, 3, 23) == 0
      and all(ML.all_safe_percent(7, m, k)
              == round(Fraction(math.comb(49 - k, m), math.comb(49, m)) * 1000 + Fraction(1, 10**9))
              for m in (1, 24, 48) for k in range(1, 49 - m + 1)),
      "(1) Chance 'alle markierten sicher' in Zehntel-Prozent (Auto-Fenster)")

counts = [0] * 25
rng = random.Random(7)
TRIES = 20000
for t in range(TRIES):
    server = fairness.new_server_seed(rng)
    for i in ML.field_from_seed(5, 3, server, "abcdefgh", t):
        counts[i] += 1
exp = TRIES * 3 / 25
sd = math.sqrt(TRIES * (3 / 25) * (22 / 25))
check(all(abs(c - exp) < 5 * sd for c in counts), "(2) Monte-Carlo: jedes Feld gleich oft Mine (20000 Felder)",
      (min(counts), max(counts), exp))
f1 = ML.field_from_seed(6, 7, "0123456789abcdef0123456789abcdef", "lama2345", 12)
f2 = ML.field_from_seed(6, 7, "0123456789abcdef0123456789abcdef", "lama2345", 12)
f3 = ML.field_from_seed(6, 7, "0123456789abcdef0123456789abcdef", "lama2345", 13)
check(f1 == f2 and f1 != f3 and len(f1) == 7 and f1 == sorted(set(f1)), "(2) Feld nur aus den Seeds (deterministisch)")

# ================================================================ Bank + Ablauf
print("\n[Bank + Ablauf]")
fresh_mem()
ev = []
g = game(events=ev, bet=100, grid=5, mines=3)
next_hash = fairness.commit(g.fair["server"])
nonce0 = g.fair["nonce"]
check(g.state == LM.IDLE and lamabank.balance() == 1000 and g.bet == 100 and g.grid == 5 and g.mines == 3,
      "(3) Start: 1000 Chips, Einsatz 100, 5x5, 3 Minen")
key(g, "space")
x = extra()
check(g.state == LM.PLAY and lamabank.balance() == 900 and isinstance(x.get("round"), dict)
      and x["round"]["bet"] == 100 and x["round"]["open"] == [] and x["round"]["h"] == next_hash
      and x["round"]["n"] == nonce0 and x["fair"]["nonce"] == nonce0 + 1,
      "(3) SPIELEN bucht 100 ab, Runde mit Prüfcode/Nonce gespeichert")
check(g.field == ML.field_from_seed(5, 3, g.round["s"], g.round["c"], g.round["n"]),
      "(3) Feld = field_from_seed(Seeds der Runde)")
cells = safe(g, 3)
for i in cells:
    g._reveal(i)
    check(extra()["round"]["open"] == cells[:cells.index(i) + 1], "(3) Feld %d sofort gespeichert" % i)
    run(g, 0.4)
win = ML.payout(100, 5, 3, 3)
key(g, "Return")
check(g.state == LM.CASHED and lamabank.balance() == 900 + win and g.hist[-1] == ML.mult100(5, 3, 3)
      and extra()["round"] is None, "(3) Enter zahlt aus (+%d), Runde gelöscht" % win)
check(lamabank.ledger("lamamines") == win - 100 and lamabank.score_for("lamamines") == 1000 + max(0, win - 100)
      and lamabank.ledger("lamacrossing") == 0 and lamabank.ledger("casino") == 0,
      "(3) Bilanz + Höchststand nur bei Lama Mines")
lamabank._data = None
lamabank.load()
check(lamabank.ledger("lamamines") == win - 100, "(3) Bilanz übersteht load()")
run(g, 1.5)
check(all(c["face"] != "hidden" for c in g.cells)
      and all(c["dim"] for i, c in enumerate(g.cells) if i not in cells)
      and not any(g.cells[i]["dim"] for i in cells), "(3) nach dem Auszahlen: Rest abgedunkelt aufgedeckt")
run(g, LM.RESULT_T + 0.2)
check(g.state == LM.IDLE and g.result is None and g.cells[cells[0]]["face"] == "gem",
      "(3) Ruhezustand, das letzte Feld bleibt sichtbar")

# Übertrag der Rest-Hundertstel (Einsatz 10 bei x1,44: 14,4 -> 14 / 14 / 15)
carry0 = g.carry
paid = []
for _ in range(3):
    run(g, LM.RESULT_T + 0.5)
    key(g, "Down")
    key(g, "Down")
    key(g, "Down")
    key(g, "Down")
    key(g, "Down")
    key(g, "space")
    for i in safe(g, 3):
        g._reveal(i)
    b = lamabank.balance()
    key(g, "Return")
    paid.append(lamabank.balance() - b)
want, c = [], carry0
for _ in range(3):
    w, c = ML.payout_carry(10, 5, 3, 3, c)
    want.append(w)
check(paid == want and g.carry == c and extra()["carry"] == c and sum(paid) == (3 * 1440 + carry0) // 100,
      "(3) Rest-Hundertstel werden übertragen (%s, Übertrag %d gespeichert)" % (paid, c), (paid, want))
run(g, LM.RESULT_T + 0.5)
for _ in range(3):
    key(g, "Up")
bal = lamabank.balance()
key(g, "space")
s0 = safe(g, 1)[0]
g._reveal(s0)
run(g, 0.4)
mine = g.field[0]
g._reveal(mine)
x = extra()
check(g.state == LM.BOOM and x["round"] is None and x["hist"][-1] == 0 and lamabank.balance() == bal - 100
      and g.hit == mine and g.cells[mine]["face"] == "boom", "(4) Mine: Einsatz weg, sofort gespeichert")
run(g, LM.FLIP_T + 0.05)
check(g.boom_t is not None and g.shake > 0 and g.sooty and g.mood[0] == "scared" and g.flashes,
      "(4) Explosion: Blitz, Wackeln, Maskottchen erschrickt")
run(g, 1.5)
check(all(c["face"] != "hidden" and c["flip"] >= 1.0 for c in g.cells)
      and all(c["dim"] for i, c in enumerate(g.cells) if i not in (mine, s0))
      and not g.cells[mine]["dim"] and g.cells[s0]["face"] == "gem",
      "(4) danach ganzes Feld abgedunkelt aufgedeckt, getroffene Mine hervorgehoben")
check(g._result_visible() and g.result[0] == i18n.t("lmi.boom"), "(4) Ergebnis-Tafel BUMM!")
key(g, "space")
check(g.state == LM.PLAY, "(4) Leertaste nach dem Ergebnis = nächste Runde")

# Fortsetzen
cells = safe(g, 4)
for i in cells:
    g._reveal(i)
    run(g, 0.3)
bal = lamabank.balance()
rnd = json.loads(json.dumps(extra()["round"]))
g2 = game()
check(g2.state == LM.PLAY and g2.round == rnd and g2.field == g.field
      and [i for i, c in enumerate(g2.cells) if c["face"] == "gem"] == sorted(cells),
      "(5) neuer Start setzt die Runde fort (Feld aus dem Seed, 4 Edelsteine offen)")
key(g2, "c")
check(lamabank.balance() == bal + ML.payout(rnd["bet"], 5, 3, 4) and g2.state == LM.CASHED,
      "(5) fortgesetzte Runde zahlt richtig aus")
bad = dict(rnd, open=rnd["open"] + [g.field[0]])
fresh_mem({"mem": {"lang": "de"}, "casino": {"migrated": True, "chips": 500, "extra": {"lamamines": {"round": bad}}}})
g3 = game()
check(g3.state == LM.IDLE and g3.round is None, "(5) manipulierte Runde (Mine als offen) wird verworfen")

# Volles Feld, Höchstgewinn, Erfolge
fresh_mem()
ev = []
g = game(events=ev, grid=4, mines=12, bet=10)
key(g, "space")
while g.state == LM.PLAY:
    g._reveal(safe(g, 1)[0])
    run(g, 0.3)
m4 = ML.mult100(4, 12, 4)
check(g.state == LM.CASHED and lamabank.balance() == 990 + ML.payout(10, 4, 12, 4) and g.hist[-1] == m4,
      "(6) volles Feld: automatisch ausgezahlt (x%s)" % (m4 / 100))
check(ev == ["lmi_clear", "lmi_brave", "lmi_legend"] if m4 >= ML.LEGEND_MULT else
      ev[:2] == ["lmi_clear", "lmi_brave"], "(6) Erfolge Schatzkammer / Tief gegraben / Diamantenlama", ev)
check(g.stats["x"].get("clears") == 1 and g.cart_fill >= 0, "(6) Statistik: 1 Feld abgeräumt")
run(g, LM.RESULT_T + 0.5)
fresh_mem()
ev = []
g = game(events=ev, grid=7, mines=24, bet=10)
key(g, "space")
n = 0
while g.state == LM.PLAY:
    g._reveal(safe(g, 1)[0])
    n += 1
check(g.state == LM.CASHED and g.last_mult == ML.MAX_MULT100 and n < 25 and ML.is_max(7, 24, n)
      and not ML.is_max(7, 24, n - 1), "(6) Höchstgewinn x100.000: automatisch ausgezahlt nach %d Steinen" % n)
run(g, 1.0)
check(g.banner is not None and g.banner[0] == i18n.t("chip.max_win") and "lmi_legend" in ev
      and "lmi_clear" not in ev, "(6) Banner HÖCHSTGEWINN, Diamantenlama", (g.banner, ev))
run(g, LM.RESULT_T + 3.5)
fresh_mem()
ev = []
g = game(events=ev, grid=5, mines=3, bet=10)
key(g, "space")
k = 0
while ML.mult100(5, 3, k + 1) < ML.BRAVE_MULT:
    g._reveal(safe(g, 1)[0])
    k += 1
key(g, "Return")
check(ev == [], "(6) unter x10 kein Erfolg", ev)
run(g, LM.RESULT_T + 0.5)
key(g, "space")
for _ in range(k + 1):
    g._reveal(safe(g, 1)[0])
key(g, "Return")
check(ev == ["lmi_brave"], "(6) ab x10 ausgezahlt: Tief gegraben", ev)

fresh_mem()
lamabank.load()
lamabank._data["chips"] = 5
lamabank._save()
g = game()
check(g.broke, "(7) 5 Chips = pleite")
key(g, "Return")
check(not g.broke and lamabank.balance() == 1000 and lamabank.refills() == 1, "(7) Kredit auf 1000")

# ================================================================ Fairness
print("\n[Fairness]")
fresh_mem()
g = game(bet=10)
shown = []
for r in range(14):
    shown.append(fairness.commit(g.fair["server"]))
    key(g, "space")
    if r % 3 == 2:
        g._reveal(g.field[0])
    else:
        g._reveal(safe(g, 1)[0])
        key(g, "Return")
    run(g, 0.2)
    g._to_idle()
log = extra()["fairlog"]
check(len(log) == 12 and [e["h"] for e in log] == shown[-12:] and [e["n"] for e in log] == list(range(2, 14)),
      "(8) Prüfcode stand vorher fest; Log hat die letzten 12 Runden")
check(all(fairness.verify(e) for e in log), "(8) sha256(Seed) = Prüfcode für alle Runden")
g._open_window("fair")
oks = []
for idx in range(len(g.fairlog)):
    g._verify(idx)
    oks.append(g.fair_checks[g.fairlog[idx]["n"]][0])
check(all(oks) and len(oks) == 12, "(8) Nachrechnen: alle 12 stimmen (Gewinne und Verluste)", oks)
lost = [e for e in g.fairlog if e["o"]["m"] == 0][0]
check(lost["o"]["open"][-1] in ML.field_from_seed(5, 3, lost["s"], lost["c"], lost["n"]),
      "(8) Verlust: zuletzt aufgedeckter Platz ist laut Seed eine Mine")
e = g.fairlog[-1]
e["o"]["m"] = e["o"]["m"] + 100 if e["o"]["m"] else 1
g.fairlog[-2]["s"] = "f" * 32
g._verify(len(g.fairlog) - 2)
g._verify(len(g.fairlog) - 1)
check(not g.fair_checks[g.fairlog[-2]["n"]][0] and not g.fair_checks[g.fairlog[-1]["n"]][0],
      "(8) Manipulierter Seed bzw. Multiplikator -> stimmt nicht")
cl = g.fair["client"]
g.draw()
click(g, g.r_fair_new.center)
check(g.fair["client"] != cl and fairness.valid_client(g.fair["client"])
      and extra()["fair"]["client"] == g.fair["client"],
      "(8) Neuer Client-Seed (gespeichert)")
key(g, "Escape")
key(g, "space")
cl = g.fair["client"]
key(g, "f")
g.draw()
click(g, g.r_fair_new.center)
check(g.fair["client"] == cl and g.toast is not None, "(8) während der Runde gesperrt")
key(g, "Escape")
check(g.window is None and g.state == LM.PLAY, "(8) Esc schließt das Fenster, Runde läuft weiter")
fresh_mem({"mem": {"lang": "de"}, "casino": {"migrated": True, "chips": 1000, "extra": {"lamamines": {
    "fair": {"server": "xyz", "client": 5, "nonce": -1}, "fairlog": [{"n": 1}, "kaputt"],
    "stats": {"rounds": "x"}, "hist": [5, -1, True, 300], "auto_marks": {"5": [1, 1, 99]}}}}})
g = game()
check(fairness.valid_server(g.fair["server"]) and fairness.valid_client(g.fair["client"]) and g.fair["nonce"] == 0
      and g.fairlog == [] and g.stats == chipstats.blank() and g.hist == [5, 300] and g.marks["5"] == [1],
      "(8) kaputter Speicherstand wird repariert")

# ================================================================ Statistik
print("\n[Statistik]")
fresh_mem()
g = game(bet=20)
paid = 0
for r in range(15):
    key(g, "space")
    if r % 5 == 0:
        g._reveal(g.field[0])
    else:
        for i in safe(g, 2):
            g._reveal(i)
        paid += ML.payout(20, 5, 3, 2)
        key(g, "Return")
    g._to_idle()
st = extra()["stats"]
check(st["rounds"] == 15 and st["losses"] == 3 and st["wins"] == 12 and st["wagered"] == 300
      and st["paid"] == paid and st["net"] == paid - 300 and st["x"]["gems"] == 24 and st["x"]["booms"] == 3
      and st["best_mult"] == ML.mult100(5, 3, 2) and len(st["curve"]) == 15,
      "(9) Runden, Gewinne, Verluste, Einsatz, Auszahlung, Edelsteine, Minen", st)
check(len(g.hist) == 12 and g.hist[-1] == ML.mult100(5, 3, 2), "(9) Verlauf: letzte 12")
key(g, "s")
check(g.window == "stats" and g.wants_escape, "(9) S öffnet die Statistik (Esc schließt)")
g.draw()
key(g, "Escape")
check(g.window is None, "(9) Esc schließt die Statistik")

# ================================================================ Auto
print("\n[Auto]")
fresh_mem()
SAVED.clear()
g = game(bet=10, grid=5, mines=3)
key(g, "a")
check(g.window == "auto", "(10) A öffnet das Auto-Fenster")
for i in (0, 6, 12):
    click(g, g.tile_r[i].center)
check(g.marks["5"] == [0, 6, 12] and extra()["auto_marks"]["5"] == [0, 6, 12], "(10) Markierungen gespeichert")
click(g, g.tile_r[6].center)
check(g.marks["5"] == [0, 12], "(10) zweiter Klick nimmt die Markierung weg")
g.kb = True
g.cursor = 6
key(g, "space")
check(g.marks["5"] == [0, 6, 12], "(10) Pfeile + Leertaste markieren auch")
click(g, g.auto_rows[0][1][0].center)
click(g, g.auto_rows[1][1][0].center)
click(g, g.auto_rows[2][1][0].center)
check(g.auto_rounds == 5 and SAVED and SAVED[-1].get("auto_rounds") == 5, "(10) Runden = 5 (gespeichert)")
bal0 = lamabank.balance()
click(g, g.r_auto_start.center)
check(g.auto is not None and g.window is None and g.wants_escape, "(10) AUTO STARTEN")
rounds = 0
last = None
for _ in range(60 * 40):
    g.update(1 / 60)
    if g.round is not None and g.round is not last:
        last = g.round
        rounds += 1
    if g.auto is None:
        break
log = g.fairlog
check(g.auto is None and rounds == 5 and len(log) == 5 and g.toast and g.toast[0] == i18n.t("chip.auto_done"),
      "(10) Auto: genau 5 Runden, dann beendet", (rounds, len(log)))
check(all(e["o"]["open"] == [0, 6, 12][:len(e["o"]["open"])] for e in log),
      "(10) Auto deckt genau die markierten Felder auf (in Reihenfolge)")
net, carry = 0, 0
for e in log:
    o = e["o"]
    if o["m"]:
        w, carry = ML.payout_carry(10, 5, 3, 3, carry)
        net += w
    net -= 10
check(lamabank.balance() == bal0 + net and all((e["o"]["m"] in (0, ML.mult100(5, 3, 3))) for e in log),
      "(10) Auto zahlt bei 3 sicheren Feldern aus (Bilanz %+d)" % net)
def next_field(g):
    """Minenfeld der nächsten Runde (Seeds stehen schon fest)."""
    return ML.field_from_seed(g.grid, g.mines, g.fair["server"], g.fair["client"], g.fair["nonce"])


def auto_until_stop(g, secs=20):
    for _ in range(int(60 * secs)):
        g.update(1 / 60)
        if g.auto is None:
            return True
    return False


# Stopp bei Verlust: Markierung auf eine Mine der nächsten Runde -> Verlust
fresh_mem()
g = game(bet=100, grid=5, mines=24)
g.marks["5"] = [next_field(g)[0]]
g.auto_loss = 100
g._start_auto()
check(auto_until_stop(g) and len(g.fairlog) == 1 and g.fairlog[0]["o"]["m"] == 0
      and g.toast[0] == i18n.t("chip.auto_hit_loss"), "(10) Stopp bei Verlust ab 100", (len(g.fairlog), g.toast))
fresh_mem()
g = game(bet=100, grid=5, mines=24)
f = next_field(g)
g.marks["5"] = [[i for i in range(25) if i not in f][0]]
g.auto_win = 100
g._start_auto()
check(auto_until_stop(g) and len(g.fairlog) == 1 and g.fairlog[0]["o"]["m"] == ML.mult100(5, 24, 1)
      and g.toast[0] == i18n.t("chip.auto_hit_win"), "(10) Stopp bei Gewinn ab 100 (volles Feld)",
      (len(g.fairlog), g.toast))
fresh_mem()
lamabank.load()
lamabank._data["chips"] = 25
lamabank._save()
g = game(bet=20, grid=5, mines=24)
g.marks["5"] = [next_field(g)[0]]
g._start_auto()
for _ in range(60 * 20):
    g.update(1 / 60)
    if g.auto is None:
        break
check(g.auto is None and lamabank.balance() < 20 and g.toast[0] in (i18n.t("chip.auto_no_chips"),),
      "(10) Auto stoppt ohne Chips", (lamabank.balance(), g.toast))
run(g, 0.5)
fresh_mem()
g = game(bet=10, grid=5, mines=3)
g.marks["5"] = [0, 1]
g._start_auto()
run(g, 0.3)
key(g, "Escape")
check(g.auto is None, "(10) Esc stoppt Auto")
run(g, 1.0)
if g.state == LM.PLAY:
    check(g.round is not None and all(c["flip"] >= 1.0 for c in g.cells), "(10) laufende Runde bleibt offen")
    if g._k() == 0:
        g._reveal(safe(g, 1)[0])
    key(g, "Return")
    check(g.state == LM.CASHED, "(10) ... und der Spieler zahlt von Hand aus")
run(g, LM.RESULT_T + 1)
check(g.state == LM.IDLE and g.round is None, "(10) danach Ruhezustand")
g.marks["5"] = list(range(23))
key(g, "a")
click(g, g.r_auto_start.center)
check(g.auto is None and g.toast and g.toast[0] == i18n.t("lmi.auto_too_many", n=22),
      "(10) zu viele Markierungen werden abgelehnt")
g.marks["5"] = []
click(g, g.r_auto_start.center)
check(g.auto is None and g.toast[0] == i18n.t("lmi.auto_need"), "(10) ohne Markierung kein Auto")
key(g, "Escape")
key(g, "4")
check(g.grid == 7 and g.marks["7"] == [] and g.marks["5"] == [], "(10) Markierungen gelten je Feldgröße")
key(g, "t")
check(g.turbo and SAVED[-1].get("turbo") is True, "(10) T = Turbo (gespeichert)")
key(g, "t")

# ================================================================ Bedienung
print("\n[Bedienung]")
fresh_mem()
SAVED.clear()
g = game(bet=10, grid=5, mines=3)
key(g, "3")
key(g, "Right")
key(g, "Right")
key(g, "Up")
key(g, "Up")
check(g.grid == 6 and g.mines == 5 and g.bet == 50, "(11) 3 = 6x6, 2x rechts = 5 Minen, 2x hoch = 50")
check(SAVED and SAVED[-1] == dict(settings_mod.DEFAULTS["lamamines"], grid=6, mines=5, bet=50),
      "(11) Einstellungen gespeichert", SAVED[-1:])
for _ in range(60):
    key(g, "Right")
check(g.mines == 35, "(11) höchstens N-1 Minen")
key(g, "1")
check(g.grid == 4 and g.mines == 15, "(11) kleineres Feld begrenzt die Minen")
for _ in range(60):
    key(g, "Left")
check(g.mines == 1, "(11) mindestens 1 Mine")
key(g, "2")
key(g, "space")
check(g.state == LM.PLAY, "(11) Leertaste = SPIELEN")
key(g, "1")
key(g, "Right")
key(g, "Down")
check(g.grid == 5 and g.round["mines"] == 1 and g.round["bet"] == 50, "(11) während der Runde gesperrt")
key(g, "Return")
check(g.state == LM.PLAY and g.toast is not None, "(11) Auszahlen erst ab dem ersten Edelstein")
g.kb = False
key(g, "space")
check(g.kb and g._k() == 0, "(11) erste Leertaste zeigt den Cursor")
g.cursor = safe(g, 1)[0]
key(g, "space")
check(g._k() == 1, "(11) Leertaste deckt das Feld unter dem Cursor auf")
c0 = g.cursor
key(g, "Right")
key(g, "Down")
check(g.cursor == min(c0 // 5 + 1, 4) * 5 + min(c0 % 5 + 1, 4), "(11) Pfeile bewegen den Cursor")
before = sum(1 for c in g.cells if c["face"] != "hidden")
key(g, "r")
check(sum(1 for c in g.cells if c["face"] != "hidden") >= before + 1 or g.state == LM.BOOM,
      "(11) R deckt ein Zufallsfeld auf")
if g.state == LM.PLAY:
    key(g, "c")
    check(g.state == LM.CASHED, "(11) C zahlt aus")
run(g, LM.RESULT_T + 0.5)
for k_, w in (("s", "stats"), ("f", "fair"), ("a", "auto")):
    key(g, k_)
    check(g.window == w and g.wants_escape, "(11) %s öffnet %s" % (k_.upper(), w))
    key(g, "Escape")
    check(g.window is None and not g.wants_escape, "(11) Esc schließt %s" % w)
g.controls = {"p1": dict(GS["controls"]["p1"], action="f", left="a", down="s"), "p2": GS["controls"]["p2"]}
key(g, "f")
check(g.window == "fair", "(11) Kurztasten roh: F öffnet Fairness auch als action-Taste belegt")
key(g, "Escape")
g.controls = {"p1": dict(GS["controls"]["p1"], action="x"), "p2": GS["controls"]["p2"]}
m0, b0 = g.mines, g.bet
key(g, "x")
key(g, "w")
key(g, "d")
check(g.state == LM.IDLE and g.window is None and g.mines == m0 and g.bet == b0,
      "(11) keine Auswertung über die Steuerung (action/WASD)")
g.controls = GS["controls"]
merged = settings_mod._merge_defaults({"lamamines": {"grid": 9, "mines": 0, "bet": 7, "turbo": 1,
                                                     "auto_rounds": 3, "auto_win": 500, "auto_loss": 99}})
check(merged["lamamines"] == dict(settings_mod.DEFAULTS["lamamines"], grid=7, mines=1, auto_win=500),
      "(11) Schema verwirft ungültige Werte", merged.get("lamamines"))
fresh_mem()
g = game(grid=7, mines=48)
check(g.mines == 48, "(11) 7x7 erlaubt 48 Minen")
GS["lamamines"] = dict(settings_mod.DEFAULTS["lamamines"], grid=4, mines=48)
g = LM.LamaMinesGame(pygame.Surface((800, 600)), 800, 600, game_settings=GS)
check(g.mines == 15, "(11) gespeicherte Minen werden auf die Feldgröße begrenzt")

# Maus
fresh_mem()
g = game(bet=10, grid=5, mines=3)
click(g, g.r_grids[3].center)
click(g, g.r_presets[4].center)
click(g, g.r_plus.center)
click(g, g.r_mminus.center)
check(g.grid == 7 and g.mines == ML.MINE_PRESETS[7][4] - 1 and g.bet == 20,
      "(12) Feldgröße, Schnellwahl, Einsatz, Minus")
click(g, g.tile_r[0].center)
check(g.state == LM.IDLE and g.toast and g.toast[0] == i18n.t("lmi.press_play"),
      "(12) Klick aufs Feld ohne Runde: Hinweis")
click(g, g.r_go.center)
check(g.state == LM.PLAY, "(12) Klick auf SPIELEN")
i = safe(g, 1)[0]
click(g, g.tile_r[i].center)
check(g._k() == 1, "(12) Klick auf ein Feld deckt es auf")
click(g, g.r_random.center)
check(g._k() == 2 or g.state == LM.BOOM, "(12) Klick auf Zufallsfeld")
if g.state == LM.PLAY:
    click(g, g.r_cash.center)
    check(g.state == LM.CASHED, "(12) Klick auf AUSZAHLEN")
run(g, 1.5)
click(g, g.r_go.center)
check(g.state == LM.PLAY, "(12) SPIELEN direkt aus dem Ergebnis")
for name, r in g.r_tools:
    if name != "turbo":
        click(g, r.center)
        check(g.window == name or (name == "auto" and g.toast), "(12) Knopf %s" % name)
        if g.window is not None:
            click(g, (2, g.height - 2))
            check(g.window is None, "(12) Klick daneben schließt %s" % name)


# ================================================================ Layout
def rects_ok(rects, bounds):
    rs = [pygame.Rect(r) for r in rects if pygame.Rect(r).w > 0]
    for i, a in enumerate(rs):
        if not bounds.contains(a):
            return False
        for b in rs[i + 1:]:
            if a.colliderect(b):
                return False
    return True


def fits(fonts, text, w):
    return min(f.size(text)[0] for f in fonts) <= w


print("\n[Layout]")
problems = []
for theme in ("v42", "v1"):
    ui.set_theme(theme)
    for code, _ in i18n.AVAILABLE:
        if theme == "v1" and code not in ("de", "fi"):
            continue
        i18n.set_language(code, persist=False)
        T = i18n.t
        for (W, H) in RES:
            tag = "%s/%s/%dx%d" % (theme, code, W, H)
            try:
                fresh_mem({"mem": {"lang": code}})
                g = game(W, H, bet=10)
                scr = pygame.Rect(0, 0, W, H)
                strip = [g.r_minus, g.r_bet, g.r_plus, g.r_random, g.r_cash, g.r_go]
                if not rects_ok(strip, scr):
                    problems.append((tag, "Leiste überlappt"))
                if not rects_ok([r for _, r in g.r_tools] + [g.board_r, g.panel_r], scr):
                    problems.append((tag, "Werkzeuge/Brett/Leiste überlappen"))
                if g.board_r.bottom > g.strip.y or g.panel_r.bottom > g.strip.y:
                    problems.append((tag, "Brett reicht in die Leiste"))
                setup = g.r_grids + [g.r_mminus, g.r_mval, g.r_mplus] + g.r_presets + g.r_pills
                if not rects_ok(setup, g.panel_r) or g.r_mascot.bottom > g.lab_grid_y \
                        or g.r_presets[0].bottom > g.ladder_lab_y or g.r_pills[0].bottom > g.hint_y:
                    problems.append((tag, "Seitenleiste überlappt"))
                auto = [r for _, row in g.auto_rows for r in row] + [g.r_auto_close, g.r_auto_clear, g.r_auto_start]
                if not rects_ok(auto, g.panel_r) or g.auto_info_y + g.f_tiny.get_height() > g.r_auto_start.y:
                    problems.append((tag, "Auto-Fenster überlappt"))
                big = [f for f in (g.f_btn, g.f_tiny, g.f_micro)]
                for rect, lbl in ((g.r_go, T("lmi.play")), (g.r_go, T("chip.auto_stop")),
                                  (g.r_cash, T("lmi.cashout") + " 10.000"), (g.r_auto_start, T("chip.auto_start")),
                                  (g.r_auto_clear, T("lmi.clear_marks"))):
                    if not fits(big, lbl, rect.w - 6):
                        problems.append((tag, "Knopf", lbl))
                if g.random_label and not fits(big, T("lmi.random"), g.r_random.w - g.r_random.h):
                    problems.append((tag, "Zufallsfeld"))
                if not fits((g.f_small, g.f_tiny), T("lmi.mines"), g.r_mminus.x - g.panel_inner[0] - 4):
                    problems.append((tag, "Minen-Beschriftung"))
                x0, w = g.panel_inner
                hw = g.panel_r.right - g.ppad - g.head_x
                for txt in (T("lmi.max_mult"), T("lmi.cur_mult")):
                    if not fits((g.f_tiny, g.f_micro), txt, hw):
                        problems.append((tag, "Kopf", txt))
                for txt in (T("lmi.payout_now", n="1.000.000"),
                            T("lmi.lost_short", n="10.000")):
                    if not fits((g.f_small, g.f_tiny, g.f_micro), txt, hw):
                        problems.append((tag, "Kopf", txt))
                if not fits((g.f_mult, g.f_big, g.f_btn), "×100.000", hw):
                    problems.append((tag, "Kopf-Multiplikator"))
                for lab in (T("lmi.next_mult"), T("lmi.safe"), T("lmi.next_win"), T("lmi.found"),
                            T("cas.bet"), T("chip.st_paid")):
                    if not fits((g.f_small, g.f_tiny, g.f_micro), lab, w - g.f_btn.size("1.000.000")[0] - 8):
                        problems.append((tag, "Info-Zeile", lab))
                for lab in (T("chip.auto_rounds"), T("chip.auto_stop_win"), T("chip.auto_stop_loss"),
                            T("lmi.marked", n=48), T("chip.auto_title"), T("lmi.ladder"), T("lmi.grid")):
                    if not fits((g.f_big, g.f_btn, g.f_tiny, g.f_micro), lab, w - g.r_auto_close.w - 4):
                        problems.append((tag, "Auto/Leiste", lab))
                for row in (1, 2):
                    r = g.auto_rows[row][1][0]
                    off_ok = fits((g.f_tiny, g.f_micro), T("chip.auto_off"), r.w - 6)
                    k_ok = fits((g.f_tiny, g.f_micro), "50k", g.auto_rows[row][1][-1].w - 6)
                    if not (off_ok and k_ok):
                        problems.append((tag, "aus-Knopf"))
                for txt in (T("lmi.hint_idle"), T("lmi.hint_play")):
                    if len(g._wrap(txt, g.f_micro, w)) > 2:
                        problems.append((tag, "Hinweis zu lang", txt))
                for txt in (T("lmi.resumed"), T("cas.not_enough"), T("lmi.press_play"), T("lmi.locked"),
                            T("chip.auto_hit_loss"), T("chip.auto_no_chips"), T("lmi.auto_too_many", n=48)):
                    if g.f_tiny.size(txt)[0] + 24 > W - 40:
                        problems.append((tag, "Hinweis", txt))
                for txt in (T("lmi.lost", n="10.000"), T("lmi.cashed", n="1.000.000.000") + "   ·   ×100.000",
                            T("lmi.boom"), T("lmi.cleared"), T("chip.max_win")):
                    if not fits((g.f_small, g.f_tiny, g.f_micro, g.f_big), txt, g.board_r.w - int(24 * g.k)):
                        problems.append((tag, "Ergebnis", txt))
                if g.tool_labels and g.tools_right > g.board_r.right:
                    problems.append((tag, "Werkzeug-Beschriftung"))
                # alle Zustände zeichnen
                g.draw()
                key(g, "space")
                run(g, 0.3)
                g.draw()
                for i in safe(g, 3):
                    g._reveal(i)
                    run(g, 0.1)
                    g.draw()
                g.mouse = g.tile_r[safe(g, 1)[0]].center
                g.kb = True
                g.draw()
                g._reveal(g.field[0])
                for _ in range(8):
                    run(g, 0.2)
                    g.draw()
                run(g, LM.RESULT_T)
                g.draw()
                key(g, "space")
                run(g, 0.3)
                g._reveal(safe(g, 1)[0])
                run(g, 0.3)
                g.on_surface_changed()
                key(g, "Return")
                for _ in range(4):
                    run(g, 0.25)
                    g.draw()
                run(g, LM.RESULT_T)
                key(g, "a")
                for i in (0, 3, 7):
                    click(g, g.tile_r[i].center)
                g.draw()
                key(g, "Return")
                for _ in range(6):
                    run(g, 0.3)
                    g.draw()
                key(g, "Escape")
                run(g, 3)
                key(g, "s")
                g.draw()
                key(g, "f")
                g.draw()
                if g.fairlog:
                    g._verify(len(g.fairlog) - 1)
                    g.draw()
                key(g, "Escape")
                # Höchstgewinn (7x7, 24 Minen)
                g._set_grid(7)
                g._set_mines(24)
                key(g, "space")
                while g.state == LM.PLAY:
                    g._reveal(safe(g, 1)[0])
                for _ in range(4):
                    run(g, 0.3)
                    g.draw()
                run(g, LM.RESULT_T + 3)
                g.broke = True
                g.toast = [T("cas.not_enough"), 1.0]
                g.draw()
                pw = min(W - 40, int(480 * max(0.8, g.k)))
                for txt in (T("cas.broke_sub"), T("cas.broke_restart", n=1000)):
                    if g.f_micro.size(txt)[0] > pw - 16:
                        problems.append((tag, "Pleite-Text", txt))
            except Exception as e:  # noqa: BLE001
                import traceback
                traceback.print_exc()
                problems.append((tag, "Ausnahme", repr(e)))
ui.set_theme("v42")
i18n.set_language("de", persist=False)
check(not problems, "(13) 5 Auflösungen x 14 Sprachen (v4.2 + v1): Layout, Texte, alle Zustände", problems[:6])

# ================================================================ Texte
print("\n[Texte]")
langs = {}
for code, _ in i18n.AVAILABLE:
    sub = "" if code in ("de", "en", "fr", "es", "pt") else "lang.expansion"
    with open(os.path.join(REPO, "lang", sub, code + ".json"), encoding="utf-8") as f:
        langs[code] = json.load(f)
src = open(os.path.join(REPO, "games", "lamamines.py"), encoding="utf-8").read()
import re
used = sorted(set(re.findall(r'"(lmi\.[a-z_]+)"', src)))
need = used + ["ach.%s.%s" % (a, f) for a in ("lmi_clear", "lmi_brave", "lmi_legend") for f in ("name", "desc")]
need += sorted(set(re.findall(r'"(chip\.[a-z_]+)"', src))) + sorted(set(re.findall(r'"(cas\.[a-z_]+)"', src)))
missing = [(c, k) for c in langs for k in need if not langs[c].get(k)]
check(len(used) >= 30 and not missing, "(14) %d Lama-Mines-Schlüssel in allen 14 Sprachen" % len(set(need)),
      missing[:5])
dirty = [(c, k) for c in langs for k in need if not swear.is_clean(langs[c].get(k, ""))]
check(not dirty, "(14) Wortfilter: alle Texte sauber", dirty[:5])
check(all(langs[c]["lmi.decimal"] in (",", ".") for c in langs), "(14) Dezimaltrennzeichen je Sprache")
wiki_bad = []
for code, _ in i18n.AVAILABLE:
    sub = "" if code in ("de", "en", "fr", "es", "pt") else "lang.expansion"
    with open(os.path.join(REPO, "lamawiki", sub, code + ".json"), encoding="utf-8") as f:
        pages = {p["id"]: p for p in json.load(f)["pages"]}
    if pages.get("lamamines", {}).get("game") != "LamaMinesGame":
        wiki_bad.append(code)
check(not wiki_bad, "(14) LamaWiki-Seite in 14 Sprachen", wiki_bad)

for p in (MEM, MEM + ".bak", MEM + ".tmp"):
    if os.path.exists(p):
        os.remove(p)
shutil.rmtree(store._DIR, ignore_errors=True)
if not os.environ.get("PGZ_AUDIT_DIR"):
    shutil.rmtree(TMP, ignore_errors=True)
D.clear_cache()
print("\n%s - %d Fehler" % ("BESTANDEN" if not FAILS else "FEHLGESCHLAGEN", len(FAILS)))
sys.exit(1 if FAILS else 0)
