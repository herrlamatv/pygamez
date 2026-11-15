# -*- coding: utf-8 -*-
"""Headless-Audit für Lama Crossing (Spur-für-Spur-Mutspiel mit Lama-Chips).

Geprüft wird:

Regeln   (1) Tabellen: richtige Länge (22/20/18/15), steigend, genau
             floor(0,97/(1-p)^k*100),
         (2) jede Auszahl-Strategie hat eine Quote von 96-97 %,
         (3) Monte-Carlo: Fehltritt-Häufigkeit passt zum Risiko je Stufe,
         (4) hit_test / clean_round / clean_history sind robust,
         (5) Web-Version (lamacrossing_logic.js) liefert dieselben Tabellen,
             Verkehrswerte, Ziehungen und Texte,
Bank     (6) Einsatz wird bei LOS abgebucht, Auszahlung gutgeschrieben, Bilanz
             und Höchststand nur für Lama Crossing; Bilanz übersteht load(),
         (7) Zufalls-Fehltritt: Einsatz weg, Verlauf zeigt 0, Runde gelöscht;
             auf der Straße kommt ein Raser und das Lama liegt platt da,
         (8) Verlassen mitten in der Überquerung: neuer Start macht genau dort
             weiter, die erreichten Spuren sind geschützt,
         (9) Ziel erreicht: automatische Auszahlung + Erfolge; Auszahlen ab x10
             = lcr_brave, auf Extrem ab x100 = lcr_extreme,
         (10) Pleite (unter 10 Chips) -> Kredit auf 1000,
Verkehr  (15) Hindernisse kommen auf allen Spuren mit dem Tempo der Stufe,
         (16) Losgehen in ein Hindernis = überfahren/getroffen (Einsatz weg),
         (17) wer auf einer Spur steht, ist geschützt (Straße: Autos halten an,
             sonst blenden die Hindernisse aus); nach der Runde geht's weiter,
Bedienung (11) Tasten: 1-4 Stufe, W Welt (4 Welten, Straße = Standard),
             Pfeile Einsatz; während der Runde gesperrt; Einstellungen werden
             gespeichert und vom Schema geprüft,
         (12) Klick auf die nächste Spur geht weiter,
Layout   (13) 5 Auflösungen x 14 Sprachen x 4 Welten: Leiste überlappt nicht,
             Texte passen, alle Zustände zeichnen fehlerfrei (UI v4.2 und v1),
Texte    (14) alle lcr-Schlüssel in 14 Sprachen, LamaWiki-Seite vorhanden.

Aufruf aus dem Repo-Root:  python tests/audit_lamacrossing.py
Exit-Code 0 = alle Prüfungen bestanden.
"""
import json
import math
import os
import random
import shutil
import subprocess
import sys
from fractions import Fraction

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
MEM = os.path.join(os.path.dirname(os.path.abspath(__file__)), "_audit-lamacrossing-mem.json")
store._PATH = MEM
import settings as settings_mod
SAVED = []
settings_mod.save_settings = lambda s: SAVED.append(json.loads(json.dumps(s.get("lamacrossing", {}))))
import i18n
i18n.init()
import ui

import lamabank
from game_base import InputEvent
from games import lamacrossing as LC
from games import lamacrossing_draw as D
from games import lamacrossing_logic as L

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
    if content is not None:
        with open(MEM, "w", encoding="utf-8") as f:
            json.dump(content, f)
    lamabank._data = None


class Seq:
    """Ersatz-Zufall: random() liefert vorgegebene Werte (danach 0.99 = hält)."""

    def __init__(self, values):
        self.values = list(values)

    def random(self):
        return self.values.pop(0) if self.values else 0.99

    def uniform(self, a, b):
        return (a + b) / 2

    def randrange(self, n):
        return 0


def game(W=800, H=600, events=None, settings=None):
    g = LC.LamaCrossingGame(pygame.Surface((W, H)), W, H, game_settings=settings or GS)
    g.play_sound = lambda name: None
    g.rumble = lambda ms=0: None
    g.ach_event = (lambda *a, **k: events.append(a[0])) if events is not None else (lambda *a, **k: None)
    return g


def calm(g):
    """Verkehr anhalten: keine Hindernisse, keine neuen (für reine Ablauf-Tests)."""
    for lane in g.lanes[1:]:
        lane["obs"] = []
        lane["t"] = 1e9
    return g


def key(g, k):
    g.handle_event(InputEvent(InputEvent.KEYDOWN, key=k))


def run(g, seconds, dt=1 / 60):
    for _ in range(int(seconds / dt)):
        g.update(dt)


def walk(g, n):
    """n Spuren gehen (Leertaste) und jeweils landen lassen."""
    for _ in range(n):
        key(g, "space")
        run(g, LC.HOP_T + 0.05)


def obstacle(y, v=0.0, kind=0):
    return dict(y=y, v=v, kind=kind, stop=None, fade=None)


# ================================================================ Regeln
print("\n[Regeln]")
bad = []
for d in L.DIFFS:
    num, den = L.RISK[d]
    hold = 1 - Fraction(num, den)
    want = tuple(math.floor(Fraction(97, 100) / hold ** k * 100) for k in range(1, L.steps(d) + 1))
    if L.TABLES[d] != want:
        bad.append(d)
check(not bad, "(1) Multiplikatoren = floor(0,97 / (1-p)^k * 100) je Stufe", bad)
check([L.steps(d) for d in L.DIFFS] == [22, 20, 18, 15], "(1) 22 / 20 / 18 / 15 Spuren")
check(all(all(a < b for a, b in zip(L.TABLES[d], L.TABLES[d][1:])) and L.TABLES[d][0] > 100
          for d in L.DIFFS), "(1) Tabellen steigen, schon Spur 1 lohnt sich")
check(L.TABLES["extreme"][-1] == 3178496 and L.TABLES["hard"][-1] == 59567, "(1) Extrem bis x31.784,96")
rets = [L.expected_return(d, k) for d in L.DIFFS for k in range(1, L.steps(d) + 1)]
check(all(0.96 <= r <= 0.97 + 1e-9 for r in rets), "(2) jede Auszahl-Strategie: Quote 96-97 %",
      (min(rets), max(rets)))
check(L.mult("easy", 0) == 100 and L.payout(100, "hard", 3) == 282 and L.payout(10, "easy", 1) == 10
      and L.payout(10000, "extreme", 15) == 317849600, "(2) Auszahlung abgerundet, große Zahlen exakt")
rng = random.Random(4711)
mc_bad = []
for d in L.DIFFS:
    n = 60000
    fails = sum(1 for _ in range(n) if not L.roll(rng, d))
    p = L.risk(d)
    se = math.sqrt(p * (1 - p) / n)
    if abs(fails / n - p) > 4 * se:
        mc_bad.append((d, fails / n, p))
check(not mc_bad, "(3) Monte-Carlo: Fehltritte je Stufe wie angegeben (4 x 60000 Spuren)", mc_bad)
check(L.hit_test(100, 40, 120, 10) and not L.hit_test(100, 40, 131, 10) and L.hit_test(140, 40, 120, 0.5),
      "(4) hit_test: Überlappung von Hindernis und Lama")
check(L.clean_round({"bet": 50, "diff": "hard", "step": 4}) == {"bet": 50, "diff": "hard", "step": 4}
      and L.clean_round({"bet": 50, "diff": "hard", "step": 18}) is None
      and L.clean_round({"bet": True, "diff": "hard", "step": 2}) is None
      and L.clean_round({"bet": 50, "diff": "x", "step": 2}) is None and L.clean_round("kaputt") is None,
      "(4) clean_round prüft Einsatz, Stufe und Spur")
check(L.clean_history([0, 154, -3, "x", True, 999] + [100] * 20) == [100] * L.HISTORY
      and L.clean_history(None) == [] and L.clean_history([0, 154]) == [0, 154],
      "(4) clean_history: nur gültige Werte, höchstens 12")
check([L.risk_percent(d) for d in L.DIFFS] == [8, 17, 30, 50] and L.fmt_mult(1142, ",") == "11,42"
      and L.fmt_mult(105) == "1.05", "(4) Anzeige: Risiko in %, Multiplikator mit Trennzeichen")
check(L.WORLDS == ("road", "bridge", "river", "ice") and set(L.SPEED_F) == set(L.WORLDS)
      and set(L.TRAFFIC) == set(L.DIFFS), "(4) 4 Welten, Verkehr je Stufe")

node = shutil.which("node")
if node:
    out = subprocess.run([node, os.path.join(REPO, "web", "tools", "lamacrossing_logic_dump.js")],
                         capture_output=True, text=True, encoding="utf-8")
    try:
        js = json.loads(out.stdout)
    except ValueError:
        js = None
    if check(js is not None, "(5) Web-Logik lässt sich laden", out.stderr[-300:]):
        check(js["diffs"] == list(L.DIFFS) and js["worlds"] == list(L.WORLDS)
              and js["tables"] == {d: list(v) for d, v in L.TABLES.items()}
              and js["risk"] == {d: list(v) for d, v in L.RISK.items()}
              and js["bets"] == list(L.BETS) and js["brave"] == L.BRAVE_MULT
              and js.get("extreme") == L.EXTREME_MULT and js["history"] == L.HISTORY,
              "(5) Web: gleiche Stufen, Tabellen, Risiken, Einsätze")
        check(js.get("traffic") == {d: list(v) for d, v in L.TRAFFIC.items()}
              and js.get("speed_f") == L.SPEED_F and js.get("raser_f") == L.RASER_F,
              "(5) Web: gleicher Verkehr (Tempo, Abstände, Raser)")
        same = True
        try:
            for d in L.DIFFS:
                for k in range(0, L.steps(d) + 2):
                    for bet in (10, 55, 10000):
                        if L.payout(bet, d, k) != js["payout"][d][k][str(bet)]:
                            same = False
        except (IndexError, KeyError):
            same = False
        vals = js["rolls_in"]
        rolls = [L.roll(Seq([v]), d) for d in L.DIFFS for v in vals]
        check(same and rolls == js["rolls"], "(5) Web: identische Auszahlung und Ziehung")
        hits = [L.hit_test(*a) for a in js.get("hits_in", [])]
        check(js.get("hits_in") and hits == js.get("hits"), "(5) Web: identischer Treffertest")
        check(js["fmt"] == [L.fmt_mult(m, s) for m in (100, 105, 1142, 16042) for s in (".", ",")]
              and js["pct"] == [L.risk_percent(d) for d in L.DIFFS],
              "(5) Web: identische Anzeige-Texte")
else:
    print("  --   (5) node nicht gefunden - Web-Vergleich übersprungen")

# ================================================================ Bank + Ablauf
print("\n[Bank + Ablauf]")
fresh_mem({"mem": {"lang": "de"}})
GS["lamacrossing"] = {"world": "road", "difficulty": "medium", "bet": 100}
ev = []
g = calm(game(events=ev))
check(g.state == LC.IDLE and lamabank.balance() == 1000 and g.bet == 100 and g.world == "road",
      "(6) Start: 1000 Chips, Einsatz 100, Straße")
g.rng = Seq([0.9, 0.9, 0.9])
walk(g, 3)
check(g.state == LC.WAIT and g.step == 3 and lamabank.balance() == 900, "(6) LOS bucht 100 ab, 3 Spuren geschafft")
key(g, "Return")
win = L.payout(100, "medium", 3)
check(g.state == LC.CASHED and lamabank.balance() == 900 + win and g.hist[-1] == L.mult("medium", 3),
      "(6) Enter zahlt aus (+%d)" % win)
check(lamabank.ledger("lamacrossing") == win - 100 and lamabank.score_for("lamacrossing") == 1000 + win - 100
      and lamabank.ledger("casino") == 0 and lamabank.score_for("blackjack") == 1000,
      "(6) Bilanz + Höchststand nur bei Lama Crossing")
lamabank._data = None
lamabank.load()
check(lamabank.ledger("lamacrossing") == win - 100, "(6) Bilanz übersteht load() (lamabank.GAMES)")
check("×" in g.result[1] and "1,67" in g.result[1], "(6) Ergebnis zeigt den Multiplikator (keine Auflösung mehr)",
      g.result)
run(g, LC.RESULT_T + 0.1)
check(g.state == LC.IDLE and g.step == 0, "(6) zurück ans Startufer")

calm(g)
g.rng = Seq([0.9, 0.0])
walk(g, 2)
run(g, 0.2)
check(g.state == LC.FALL and g.broken == 2 and g.round is None and g.hist[-1] == 0
      and lamabank.balance() == 900 + win - 100, "(7) Zufalls-Fehltritt auf Spur 2: Einsatz weg, Verlauf 0")
check(g.fall_kind == "hit" and g.result[0] == i18n.t("lcr.fall.road"),
      "(7) Straße: ein Raser überfährt das Lama, es liegt platt da", (g.fall_kind, g.result))
extra = lamabank.get_extra("lamacrossing")
check(extra["round"] is None and extra["hist"] == g.hist, "(7) Runde gelöscht, Verlauf gespeichert")
key(g, "space")
check(g.state == LC.FALL, "(7) Ergebnis bleibt kurz stehen")
run(g, 0.7)
key(g, "space")
check(g.state == LC.IDLE, "(7) Taste schließt das Ergebnis")

GS["lamacrossing"]["world"] = "bridge"
g = calm(game())
g.rng = Seq([0.0])
walk(g, 1)
run(g, 0.1)
check(g.state == LC.FALL and g.fall_kind == "break", "(7) Brücke: Zufall = Planke bricht (wie bisher)")
run(g, LC.RESULT_T + LC.FALL_T + 0.1)
GS["lamacrossing"]["world"] = "road"

g = calm(game())
g.rng = Seq([0.9, 0.9])
walk(g, 2)
bal = lamabank.balance()
g2 = game()
check(g2.state == LC.WAIT and g2.step == 2 and g2.round == {"bet": 100, "diff": "medium", "step": 2}
      and g2.lanes[1]["closed"] and g2.lanes[2]["closed"] and not g2.lanes[3]["closed"],
      "(8) neuer Start setzt fort, erreichte Spuren sind geschützt")
key(g2, "c")
check(lamabank.balance() == bal + L.payout(100, "medium", 2), "(8) fortgesetzte Runde zahlt richtig aus")

fresh_mem({"mem": {"lang": "de"}})
GS["lamacrossing"] = {"world": "ice", "difficulty": "extreme", "bet": 10}
ev = []
g = calm(game(events=ev))
g.rng = Seq([])
walk(g, 14)
check(g.state == LC.WAIT and g.step == 14, "(9) Extrem: 14 Spuren")
key(g, "space")
run(g, 2 * LC.HOP_T + 0.2)
full = L.payout(10, "extreme", 15)
check(g.state == LC.GOAL and lamabank.balance() == 990 + full and g.round is None,
      "(9) Ziel: automatisch %d ausgezahlt" % full)
check(set(ev) == {"lcr_other_side", "lcr_extreme", "lcr_brave"}, "(9) Erfolge am Ziel", ev)
run(g, LC.RESULT_T + 0.2)
GS["lamacrossing"]["difficulty"] = "hard"
ev = []
g = calm(game(events=ev))
g.rng = Seq([])
walk(g, 6)
key(g, "Return")
check(ev == [], "(9) x8,24 ist noch nicht nervenstark", ev)
run(g, LC.RESULT_T + 0.2)
calm(g)
walk(g, 7)
key(g, "Return")
check(ev == ["lcr_brave"], "(9) Auszahlen bei x11,77: Nervenstark", ev)
run(g, LC.RESULT_T + 0.2)
GS["lamacrossing"]["difficulty"] = "extreme"
ev = []
g = calm(game(events=ev))
g.rng = Seq([])
walk(g, 6)
key(g, "Return")
check(ev == ["lcr_brave"], "(9) Extrem x62,08: nur Nervenstark", ev)
run(g, LC.RESULT_T + 0.2)
calm(g)
walk(g, 7)
key(g, "Return")
check(ev == ["lcr_brave", "lcr_brave", "lcr_extreme"], "(9) Extrem x124,16: Drahtseilakt", ev)

fresh_mem({"mem": {"lang": "de"}})
lamabank.load()
lamabank._data["chips"] = 5
lamabank._save()
g = game()
check(g.broke, "(10) 5 Chips = pleite")
key(g, "Return")
check(not g.broke and lamabank.balance() == 1000 and lamabank.refills() == 1, "(10) Kredit auf 1000")

# ================================================================ Verkehr
print("\n[Verkehr]")
fresh_mem({"mem": {"lang": "de"}})
GS["lamacrossing"] = {"world": "road", "difficulty": "hard", "bet": 10}
g = game()
run(g, 6.0)
seen = [i for i in range(1, len(g.lanes)) if g.lanes[i]["obs"]]
v_lo, v_hi, gap_lo, gap_hi = L.TRAFFIC["hard"]
speeds_ok = all(v_lo * g.k - 1e-9 <= g.lanes[i]["v"] <= v_hi * g.k + 1e-9 for i in range(1, len(g.lanes)))
check(len(g.lanes) == L.steps("hard") + 1 and len(seen) >= L.steps("hard") // 2 and speeds_ok,
      "(15) Autos auf den Spuren, Tempo der Stufe", (len(seen), speeds_ok))
objs = [o for i in seen for o in g.lanes[i]["obs"]]
y0s = [o["y"] for o in objs]
run(g, 0.1)
check(objs and all(o["y"] > y0 for o, y0 in zip(objs, y0s)), "(15) Autos fahren nach unten")
lens = [D.obstacle_len("road", k, g.gap) for k in range(4)]
check(lens[3] > lens[0] == lens[1] == lens[2] > 0, "(15) Laster sind länger als Autos", lens)

for world in L.WORLDS:
    GS["lamacrossing"] = {"world": world, "difficulty": "medium", "bet": 10}
    fresh_mem({"mem": {"lang": "de"}})
    g = calm(game())
    g.rng = Seq([])
    walk(g, 1)
    bal = lamabank.balance()
    g.lanes[2]["obs"].append(obstacle(g._target_y()))
    key(g, "space")
    run(g, LC.HOP_T + 0.05)
    check(g.state == LC.FALL and g.fall_kind == "hit" and g.round is None and g.hist[-1] == 0
          and lamabank.balance() == bal and g.result[0] == i18n.t("lcr.hit." + world),
          "(16) %s: in ein Hindernis gelaufen -> platt, Einsatz weg" % world, (g.state, g.fall_kind, g.result))
    extra = lamabank.get_extra("lamacrossing")
    check(extra["round"] is None and extra["hist"][-1] == 0, "(16) %s: Verlust gespeichert" % world)

GS["lamacrossing"] = {"world": "road", "difficulty": "medium", "bet": 10}
fresh_mem({"mem": {"lang": "de"}})
g = calm(game())
g.rng = Seq([])
far = obstacle(g.scene.y, v=300.0 * g.k)
g.lanes[1]["obs"].append(far)
walk(g, 1)
check(g.state == LC.WAIT and g.lanes[1]["closed"] and far["stop"] is not None, "(17) Straße: Spur gesperrt, Auto bremst")
run(g, 3.0)
olen = D.obstacle_len("road", 0, g.gap)
check(g.state == LC.WAIT and abs(far["y"] - (g.path_y - g.llama_n - olen / 2 - 6 * g.k)) < 1e-6
      and far in g.lanes[1]["obs"], "(17) Auto hält vor dem Lama an, kein Treffer beim Stehen", far)
g.lanes[1]["t"] = -1
run(g, 0.5)
check(len(g.lanes[1]["obs"]) == 1, "(17) auf der gesperrten Spur kommt nichts nach")
key(g, "Return")
run(g, LC.RESULT_T + 0.1)
check(g.state == LC.IDLE and not g.lanes[1]["closed"] and far["stop"] is None, "(17) nach der Runde: Spur wieder offen")
y0 = far["y"]
run(g, 0.2)
check(far["y"] > y0, "(17) das Auto fährt weiter")

GS["lamacrossing"] = {"world": "river", "difficulty": "medium", "bet": 10}
fresh_mem({"mem": {"lang": "de"}})
g = calm(game())
g.rng = Seq([])
log = obstacle(g._horizon(), v=10.0)
g.lanes[1]["obs"].append(log)
walk(g, 1)
check(log["fade"] is not None, "(17) Fluss: Treibholz auf der erreichten Spur blendet aus")
run(g, LC.FADE_T + 0.1)
check(log not in g.lanes[1]["obs"] and g.state == LC.WAIT, "(17) Fluss: Treibholz weg, Lama sicher")

# ================================================================ Bedienung
print("\n[Bedienung]")
fresh_mem({"mem": {"lang": "de"}})
GS["lamacrossing"] = {"world": "road", "difficulty": "medium", "bet": 10}
SAVED.clear()
g = calm(game())
key(g, "3")
key(g, "w")
key(g, "Up")
key(g, "Up")
check(g.diff == "hard" and g.world == "bridge" and g.bet == 50, "(11) 3 = Schwer, W = Brücke, 2x hoch = 50")
check(SAVED and SAVED[-1] == {"world": "bridge", "difficulty": "hard", "bet": 50}, "(11) Einstellungen gespeichert",
      SAVED[-1:])
key(g, "w")
key(g, "w")
key(g, "w")
check(g.world == "road", "(11) W wechselt reihum durch alle 4 Welten")
calm(g)
g.rng = Seq([])
walk(g, 1)
key(g, "1")
key(g, "w")
key(g, "Down")
check(g.diff == "hard" and g.world == "road" and g.round["bet"] == 50, "(11) während der Runde gesperrt")
merged = settings_mod._merge_defaults({"lamacrossing": {"world": "mars", "difficulty": "hard", "bet": 7}})
check(merged["lamacrossing"] == {"world": "road", "difficulty": "hard", "bet": 10},
      "(11) Schema verwirft ungültige Werte (Standard = Straße)", merged.get("lamacrossing"))
r = g._next_tile_rect()
g.handle_event(InputEvent(InputEvent.MOUSEDOWN, pos=r.center, button=1))
run(g, LC.HOP_T + 0.05)
check(g.step == 2, "(12) Klick auf die nächste Spur geht weiter")
g.handle_event(InputEvent(InputEvent.MOUSEDOWN, pos=g.r_cash.center, button=1))
check(g.state == LC.CASHED, "(12) Klick auf AUSZAHLEN")


# ================================================================ Layout
def rects_ok(rects, W, H):
    rs = [pygame.Rect(r) for r in rects]
    for i, a in enumerate(rs):
        if a.left < 0 or a.right > W or a.top < 0 or a.bottom > H:
            return False
        for b in rs[i + 1:]:
            if a.colliderect(b):
                return False
    return True


def fits(g, rect, label):
    return min(g.f_btn.size(label)[0], g.f_tiny.size(label)[0]) <= rect.w - 4


print("\n[Layout]")
problems = []
for theme in ("v42", "v1"):
    ui.set_theme(theme)
    for code, _ in i18n.AVAILABLE:
        if theme == "v1" and code not in ("de", "fi"):
            continue
        i18n.set_language(code, persist=False)
        for (W, H) in RES:
            for world in L.WORLDS:
                tag = "%s/%s/%dx%d/%s" % (theme, code, W, H, world)
                try:
                    fresh_mem({"mem": {"lang": code}})
                    GS["lamacrossing"] = {"world": world, "difficulty": "easy", "bet": 10}
                    g = game(W, H)
                    strip = [g.r_minus, g.r_bet, g.r_plus, g.r_cash, g.r_go]
                    if not rects_ok(strip, W, H) or not rects_ok(g.r_diffs + g.r_worlds, W, H):
                        problems.append((tag, "Leiste/Knöpfe überlappen"))
                    for rect, lbl in ((g.r_go, i18n.t("lcr.go")), (g.r_go, i18n.t("lcr.step")),
                                      (g.r_cash, i18n.t("lcr.cashout") + " 31784960")):
                        if not fits(g, rect, lbl):
                            problems.append((tag, "Knopf", lbl))
                    for i, d in enumerate(L.DIFFS):
                        if not fits(g, g.r_diffs[i], "%d  %s" % (i + 1, i18n.t("lcr.diff." + d))):
                            problems.append((tag, "Stufe", d))
                    for d in L.DIFFS:
                        info = i18n.t("lcr.info", n=L.steps(d), p=L.risk_percent(d),
                                      m=L.fmt_mult(L.TABLES[d][-1], i18n.t("lcr.decimal")))
                        if g.f_tiny.size(info)[0] + 18 > W:
                            problems.append((tag, "Info", info))
                    for txt in (i18n.t("lcr.hint_wait"), i18n.t("lcr.resumed"), i18n.t("cas.not_enough")):
                        if g.f_tiny.size(txt)[0] + 18 > W:
                            problems.append((tag, "Hinweis", txt))
                    for txt in [i18n.t("lcr.lost", n=10000), "×31.784,96   ·   %s: 10000" % i18n.t("cas.bet"),
                                i18n.t("lcr.cashed", n=317849600)]:
                        if g.f_tiny.size(txt)[0] > W - 40:
                            problems.append((tag, "Ergebnis-Text", txt))
                    for txt in [i18n.t("lcr.goal"), i18n.t("lcr.fall." + world), i18n.t("lcr.hit." + world),
                                i18n.t("lcr.cashed", n=317849600)]:
                        if g.f_big.size(txt)[0] > W - 40:
                            problems.append((tag, "Ergebnis-Titel", txt))
                    if g.r_worlds[-1].right + g.margin + g.f_small.size(i18n.t("lcr.world." + world))[0] > W * 0.6:
                        problems.append((tag, "Weltname"))
                    if g.r_diffs[0].y - g.f_tiny.get_height() - 12 < g.path_y + g.tw * 0.36 + 6 * g.k + g.f_tiny.get_height():
                        problems.append((tag, "Stufen-Leiste verdeckt die Schilder"))
                    # alle Zustände zeichnen - mit laufendem Verkehr
                    run(g, 1.5)
                    g.draw()
                    calm(g)
                    g.rng = Seq([0.9, 0.9, 0.9])
                    walk(g, 2)
                    g.lanes[3]["obs"].append(obstacle(g.scene.y, v=200.0))
                    g.draw()
                    key(g, "space")
                    run(g, 0.15)
                    g.draw()
                    run(g, 0.4)
                    key(g, "Return")
                    run(g, 0.3)
                    g.draw()
                    run(g, LC.RESULT_T)
                    calm(g)
                    g.rng = Seq([0.0])
                    key(g, "space")
                    for _ in range(6):
                        run(g, 0.25)
                        g.draw()
                    run(g, LC.RESULT_T + LC.FALL_T)
                    calm(g)
                    g.rng = Seq([])
                    walk(g, 1)
                    g.lanes[2]["obs"].append(obstacle(g._target_y() - 150.0 * L.SPEED_F[world] * LC.HOP_T,
                                                      v=150.0, kind=3))
                    key(g, "space")
                    for _ in range(4):
                        run(g, 0.25)
                        g.draw()
                    if not (g.state == LC.FALL and g.fall_kind == "hit"):
                        problems.append((tag, "Treffer-Zustand", g.state))
                    run(g, LC.RESULT_T + LC.FALL_T)
                    g.state = LC.GOAL
                    g.step = L.steps("easy") + 1
                    g.result = (i18n.t("lcr.goal"), i18n.t("lcr.cashed", n=55), ui.GOLD)
                    g.draw()
                    g._to_idle()
                    g.broke = True
                    g.toast = [i18n.t("cas.not_enough"), 1.0]
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
check(not problems, "(13) 5 Auflösungen x 14 Sprachen x 4 Welten: Layout + alle Zustände", problems[:6])

# ================================================================ Texte
print("\n[Texte]")
langs = {}
for code, _ in i18n.AVAILABLE:
    sub = "" if code in ("de", "en", "fr", "es", "pt") else "lang.expansion"
    with open(os.path.join(REPO, "lang", sub, code + ".json"), encoding="utf-8") as f:
        langs[code] = json.load(f)
need = sorted(k for k in langs["de"] if k.startswith("lcr."))
need += ["lcr.world." + w for w in L.WORLDS] + ["lcr.hit." + w for w in L.WORLDS] \
    + ["lcr.fall." + w for w in L.WORLDS]
need += ["ach.%s.%s" % (a, f) for a in ("lcr_other_side", "lcr_brave", "lcr_extreme") for f in ("name", "desc")]
need += ["cas.bet", "cas.chips", "cas.record", "cas.credits", "cas.broke", "cas.broke_sub",
         "cas.broke_restart", "cas.not_enough"]
missing = [(c, k) for c in langs for k in need if not langs[c].get(k)]
check(len(set(need)) >= 40 and not missing, "(14) %d Lama-Crossing-Schlüssel in allen 14 Sprachen" % len(set(need)),
      missing[:5])
check(not any(k.startswith("lcr.reveal") for c in langs for k in langs[c]), "(14) Auflösungs-Texte entfernt")
wiki_bad = []
for code, _ in i18n.AVAILABLE:
    sub = "" if code in ("de", "en", "fr", "es", "pt") else "lang.expansion"
    with open(os.path.join(REPO, "lamawiki", sub, code + ".json"), encoding="utf-8") as f:
        pages = {p["id"]: p for p in json.load(f)["pages"]}
    if pages.get("lamacrossing", {}).get("game") != "LamaCrossingGame":
        wiki_bad.append(code)
check(not wiki_bad, "(14) LamaWiki-Seite in 14 Sprachen", wiki_bad)

for p in (MEM, MEM + ".bak", MEM + ".tmp"):
    if os.path.exists(p):
        os.remove(p)
print("\n%s - %d Fehler" % ("BESTANDEN" if not FAILS else "FEHLGESCHLAGEN", len(FAILS)))
sys.exit(1 if FAILS else 0)
