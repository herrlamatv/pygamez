# -*- coding: utf-8 -*-
"""Headless-Audit für das Casino (Roulette + Lama-Slot) und die Lama-Bank.

Geprüft wird:

Roulette   (1) Kessel: 37 Fächer, echte Reihenfolge, 18 rot / 18 schwarz,
               Nachbarn wechseln die Farbe,
           (2) Treffererkennung: jede Wettart an Zahl, Kante und Ecke (Plein,
               Cheval, Transversale inkl. Trio, Carré inkl. 0-1-2-3, Sixain,
               Kolonne, Dutzend, einfache Chancen); vollständige Anzahl aller
               Wetten; jeder Chip-Anker trifft wieder seine Wette,
           (3) Auszahlung je Wettart (35/17/11/8/5/2/1 zu 1) und für jede
               Wette der Erwartungswert 36/37 des Einsatzes,
Lama-Slot  (4) Linienauswertung (Wild, Scatter, bester Gewinn),
           (5) Auszahlungsquote EXAKT aus den Walzenstreifen (~96 %) und die
               angezeigte Quote stimmt, (6) Monte-Carlo-Gegenprobe inkl.
               Freispielen (innerhalb von 4 Standardfehlern),
           (7) Web-Version (casino_logic.js) liefert dieselben Tabellen,
               Treffer, Auswertungen und dieselbe Quote,
Lama-Bank  (8) Migration aus Blackjack/Poker, Abschnitt "highscores" bleibt
               unverändert, (9) Bilanzen trennen die Spiele, Kredite zählen
               nicht, (10) Pleite-Regel je Mindesteinsatz,
Spiele     (11) Einsatz nach Abbruch weg (Blackjack, Roulette, Slot),
           (12) Poker-Tischstapel (escrow) wird erstattet, Pot-Anteil nicht,
           (13) Blackjack deckt die Dealer-Karte wirklich auf,
           (14) Erfolge roulette_plein / slots_jackpot / poker_rich,
           (15) Freispiele überleben das Verlassen, Einstellungen werden
               gespeichert,
Layout     (16) 5 Auflösungen x 14 Sprachen x beide Modi: Texte passen,
               Bedienelemente überlappen nicht, alle Zustände zeichnen
               fehlerfrei (UI v4.2 und v1),
Tempo      (17) 1280x960: Frame-Zeit beim Drehen,
Texte      (18) alle Casino-Schlüssel in 14 Sprachen, Wiki-Seiten vorhanden.

Aufruf aus dem Repo-Root:  python tests/audit_casino.py
Exit-Code 0 = alle Prüfungen bestanden.
"""
import json
import math
import os
import random
import shutil
import subprocess
import sys
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
MEM = os.path.join(os.path.dirname(os.path.abspath(__file__)), "_audit-casino-mem.json")
store._PATH = MEM
import settings as settings_mod
SAVED = []
settings_mod.save_settings = lambda s: SAVED.append(json.loads(json.dumps(s.get("casino", {}))))
import i18n
i18n.init()
import ui

import lamabank
from game_base import InputEvent
from games import blackjack as bjm
from games import casino as cas
from games import casino_logic as L
from games import poker as pkm

GS = json.loads(json.dumps(settings_mod.DEFAULTS))
FAILS = []
RES = [(480, 360), (640, 480), (800, 600), (960, 720), (1280, 960)]


def check(ok, label, detail=""):
    print("  %-4s %s%s" % ("OK" if ok else "FAIL", label,
                           ("  -> " + str(detail)) if detail and not ok else ""))
    if not ok:
        FAILS.append(label)
    return ok


def quiet(game, events=None):
    game.play_sound = lambda name: None
    game.rumble = lambda ms=0: None
    game.ach_event = (lambda *a, **k: events.append(a)) if events is not None \
        else (lambda *a, **k: None)
    game.report_result = lambda won: None
    return game


def fresh_mem(content=None):
    for p in (MEM, MEM + ".bak", MEM + ".tmp"):
        if os.path.exists(p):
            os.remove(p)
    if content is not None:
        with open(MEM, "w", encoding="utf-8") as f:
            json.dump(content, f)
    lamabank._data = None


class FixedRng:
    """Ersatz-Zufall: randrange liefert vorgegebene Werte der Reihe nach."""

    def __init__(self, values):
        self.values = list(values)

    def randrange(self, n):
        return self.values.pop(0) % n

    def random(self):
        return 0.5

    def uniform(self, a, b):
        return (a + b) / 2


def run(game, seconds, dt=1 / 60):
    for _ in range(int(seconds / dt)):
        game.update(dt)


# ================================================================ Roulette
print("\n[Roulette]")
order = L.WHEEL_ORDER
check(sorted(order) == list(range(37)), "(1) Kessel hat 37 verschiedene Fächer")
check(order[:5] == (0, 32, 15, 19, 4) and order[-1] == 26, "(1) europäische Reihenfolge")
check(len(L.RED_NUMBERS) == 18, "(1) 18 rote Zahlen")
alternate = all(L.color_of(order[i]) != L.color_of(order[i + 1]) for i in range(1, 36))
check(alternate and L.color_of(order[1]) != L.color_of(order[36]),
      "(1) Nachbarfächer wechseln die Farbe")

cases = [
    ((6.5, 1.5), "plein:17"),
    ((0.4, 1.5), "plein:0"),
    ((7.0, 1.5), "cheval:17-20"),
    ((6.5, 1.0), "cheval:17-18"),
    ((6.5, 2.0), "cheval:16-17"),
    ((1.0, 0.5), "cheval:0-3"),
    ((0.9, 2.5), "cheval:0-1"),
    ((2.0, 2.0), "carre:1-2-4-5"),
    ((1.5, 3.0 - 0.01), "trans:1-2-3"),
    ((6.5, 0.01), "trans:16-17-18"),
    ((1.0, 2.0), "trans:0-1-2"),
    ((1.0, 1.0), "trans:0-2-3"),
    ((1.0, 2.99), "carre:0-1-2-3"),
    ((2.0, 2.99), "sixain:1-2-3-4-5-6"),
    ((12.9, 2.99), "trans:34-35-36"),
    ((13.5, 0.5), "column:0"),
    ((13.5, 2.5), "column:2"),
    ((5.0, 3.3), "dozen:1"),
    ((1.5, 4.2), "low"), ((3.5, 4.2), "even"), ((5.5, 4.2), "red"),
    ((7.5, 4.2), "black"), ((9.5, 4.2), "odd"), ((11.5, 4.2), "high"),
]
bad = [(pos, want, L.hit_test(*pos)) for pos, want in cases if L.hit_test(*pos) != want]
check(not bad, "(2) Klick auf Zahl/Kante/Ecke trifft die richtige Wette", bad)
check(L.hit_test(0.5, 4.2) is None and L.hit_test(13.5, 4.2) is None,
      "(2) Leerflächen neben den Außenfeldern treffen nichts")
keys = set()
steps = 100
for i in range(int(L.TABLE_W * steps)):
    for j in range(int(L.TABLE_H * steps)):
        k = L.hit_test(i / steps + 1e-7, j / steps + 1e-7)
        if k:
            keys.add(k)
kinds = {}
for k in keys:
    kinds[L.bet_kind(k)] = kinds.get(L.bet_kind(k), 0) + 1
want = {"plein": 37, "cheval": 60, "trans": 14, "carre": 23, "sixain": 11, "column": 3,
        "dozen": 3, "red": 1, "black": 1, "even": 1, "odd": 1, "low": 1, "high": 1}
check(kinds == want, "(2) alle 157 Wetten des europäischen Tischs erreichbar", kinds)
anchor_bad = []
for k in keys:
    ax, ay = L.anchor(k)
    if not any(L.hit_test(ax + dx, ay + dy) == k for dx in (0, 0.01, -0.01) for dy in (0, 0.01, -0.01)):
        anchor_bad.append(k)
check(not anchor_bad and all(L.valid_key(k) for k in keys),
      "(2) Chip-Anker liegt auf seiner Wette, alle Schlüssel gültig", anchor_bad[:5])

pay_bad = []
for k in sorted(keys):
    nums = L.bet_numbers(k)
    mult = L.payout_multiplier(k)
    if mult != 36 // len(nums) - 1:
        pay_bad.append((k, mult))
    total = 0
    for n in range(37):
        payout, winners = L.settle({k: 10}, n)
        expect = 10 * (mult + 1) if n in nums else 0
        if payout != expect or (k in winners) != (n in nums):
            pay_bad.append((k, n, payout))
        total += payout
    if total != 360:
        pay_bad.append((k, "ev", total))
check(not pay_bad, "(3) Auszahlung je Wette korrekt, Erwartungswert 36/37", pay_bad[:5])
check([L.PAYOUT[x] for x in ("plein", "cheval", "trans", "carre", "sixain", "column", "dozen", "red")]
      == [35, 17, 11, 8, 5, 2, 2, 1], "(3) Auszahlungstabelle 35/17/11/8/5/2/2/1")
payout, winners = L.settle({"plein:17": 5, "red": 10, "black": 10, "cheval:17-20": 2}, 17)
check(payout == 5 * 36 + 20 + 2 * 18 and sorted(winners) == ["black", "cheval:17-20", "plein:17"],
      "(3) Mehrere Wetten gleichzeitig abgerechnet", (payout, winners))

# ================================================================ Slot
print("\n[Lama-Slot]")
check(all(len(s) == 32 and set(s) <= set(L.SYMBOLS) for s in L.STRIPS),
      "(4) 5 Walzenstreifen mit je 32 gültigen Symbolen")
check(len(L.LINES) == 10 and len({tuple(x) for x in L.LINES}) == 10, "(4) 10 verschiedene Gewinnlinien")
lr = L.line_result
check(lr(["lama"] * 5) == ("lama", 5, 750), "(4) 5 Lamas = Jackpot 750")
check(lr(["lama", "lama", "seven", "seven", "cherry"]) == ("seven", 4, 30), "(4) Wild ergänzt Symbole")
check(lr(["lama", "lama", "lama", "cherry", "cherry"]) == ("lama", 3, 20),
      "(4) bester Gewinn zählt (3 Lamas vor 5 Kirschen)")
check(lr(["lama", "coin", "lama", "lama", "lama"]) == (None, 0, 0), "(4) Münze wird nie ersetzt")
check(lr(["bell", "bell", "gem", "bell", "bell"]) == (None, 0, 0), "(4) nur zusammenhängend von links")
res = L.evaluate([["coin", "x", "y"], ["a", "coin", "b"], ["c", "d", "coin"], ["cherry"] * 3, ["cherry"] * 3], 2)
check(res["scatter"] == 3 and res["free_spins"] == 10 and res["scatter_win"] == 2 * 2 * 10,
      "(4) 3 Münzen = Scatter-Gewinn + 10 Freispiele", res)
res2 = L.evaluate([["lama"] * 3] * 5, 1, free=True)
check(res2["jackpot"] and res2["total"] == 10 * 750 * 2, "(4) Freispiele zahlen doppelt, Jackpot erkannt")

t0 = time.perf_counter()
ex = L.exact_rtp()
check(0.955 <= ex["rtp"] <= 0.965, "(5) exakte Auszahlungsquote %.4f liegt bei ~96 %%" % ex["rtp"])
check(L.RTP_TEXT == "%.1f" % (ex["rtp"] * 100), "(5) angezeigte Quote %s stimmt" % L.RTP_TEXT)
print("       exakt: Linien %.4f + Scatter %.4f, Freispiel-Auslösung 1/%.0f, Jackpot/Linie 1/%.0f (%.2fs)"
      % (ex["line"], ex["scatter"], 1 / ex["trigger"], 1 / ex["jackpot_line"], time.perf_counter() - t0))

# Monte-Carlo inkl. Freispiel-Runden (gleiche Logik wie das Spiel)
rng = random.Random(20260916)
N = 120000
returns = []
t0 = time.perf_counter()
for _ in range(N):
    w = L.evaluate(L.window(L.random_stops(rng)), 1)
    total = w["total"]
    free = w["free_spins"]
    while free > 0:
        free -= 1
        f = L.evaluate(L.window(L.random_stops(rng)), 1, free=True)
        total += f["total"]
        free += f["free_spins"]
    returns.append(total / 10.0)
mean = sum(returns) / N
var = sum((x - mean) ** 2 for x in returns) / (N - 1)
se = math.sqrt(var / N)
check(abs(mean - ex["rtp"]) < 4 * se,
      "(6) Monte-Carlo %.4f ± %.4f passt zur exakten Quote (%d Drehungen, %.1fs)"
      % (mean, se, N, time.perf_counter() - t0), (mean, ex["rtp"], se))

node = shutil.which("node")
if node:
    out = subprocess.run([node, os.path.join(REPO, "web", "tools", "casino_logic_dump.js")],
                         capture_output=True, text=True, encoding="utf-8")
    try:
        js = json.loads(out.stdout)
    except ValueError:
        js = None
    if check(js is not None, "(7) Web-Logik lässt sich laden", out.stderr[-300:]):
        check(list(L.WHEEL_ORDER) == js["wheel"] and [list(s) for s in L.STRIPS] == js["strips"]
              and {k: list(v) for k, v in L.PAYS.items()} == js["pays"]
              and [list(x) for x in L.LINES] == js["lines"] and L.RTP_TEXT == js["rtp_text"]
              and {str(k): v for k, v in L.SCATTER_PAYS.items()} == js["scatter_pays"],
              "(7) Web: gleiche Kesselreihenfolge, Walzen, Linien und Gewinntabelle")
        grid = []
        x = 0.013
        while x < L.TABLE_W:
            y = 0.011
            while y < L.TABLE_H:
                grid.append(L.hit_test(x, y))
                y += js["grid_step"]
            x += js["grid_step"]
        check(grid == js["grid"], "(7) Web: identische Treffererkennung (%d Punkte)" % len(grid))
        check(all(list(L.anchor(k)) == v for k, v in js["anchors"].items()), "(7) Web: identische Chip-Anker")
        ev_bad = 0
        for i, (stops, total, nlines, sc, jp) in enumerate(js["evals"]):
            r = L.evaluate(L.window(stops), 2, i % 5 == 0)
            ev_bad += (r["total"], len(r["lines"]), r["scatter"], r["jackpot"]) != (total, nlines, sc, jp)
        check(ev_bad == 0, "(7) Web: identische Auswertung von 400 Walzenbildern")
        check(abs(js["rtp"]["rtp"] - ex["rtp"]) < 1e-12, "(7) Web: identische exakte Quote")
else:
    print("  --   (7) node nicht gefunden - Web-Vergleich übersprungen")

# ================================================================ Lama-Bank
print("\n[Lama-Bank]")
hs = {"blackjack": 6000, "poker": 1400, "casino": 0, "snake": 99}
fresh_mem({"mem": {"lang": "de"}, "highscores": dict(hs),
           "blackjack": {"chips": 3200, "best": 5000}, "poker": {"chips": 800, "best": 1500}})
d = lamabank.load()
raw = json.load(open(MEM, encoding="utf-8"))
check(d["chips"] == 3200 and d["migrated"], "(8) Migration: Konto = max(Blackjack, Poker)")
check(lamabank.score_for("blackjack") == 6000 and lamabank.score_for("poker") == 1500
      and lamabank.score_for("casino") == 1000, "(8) Migration: Bestwerte übernommen (Highscore falls höher)")
check(raw["highscores"] == hs and raw["blackjack"] == {"chips": 3200, "best": 5000}
      and raw["poker"] == {"chips": 800, "best": 1500}, "(8) highscores/alte Abschnitte unverändert")
lamabank.load()
check(json.load(open(MEM, encoding="utf-8"))["highscores"] == hs and lamabank.balance() == 3200,
      "(8) erneutes Laden migriert nicht nochmal")
fresh_mem({"highscores": {}})
lamabank.load()
check(lamabank.balance() == 1000 and lamabank.score_for("blackjack") == 1000, "(8) ohne Altdaten: Start 1000")

fresh_mem()
lamabank.load()
lamabank.debit(100, "casino")
lamabank.credit(4100, "casino")
check(lamabank.ledger("casino") == 4000 and lamabank.score_for("casino") == 5000
      and lamabank.score_for("blackjack") == 1000 and lamabank.value_for("poker") == 1000,
      "(9) Casino-Gewinn hebt nur den Casino-Highscore")
lamabank.debit(3000, "blackjack")
check(lamabank.score_for("blackjack") == 1000 and lamabank.score_for("casino") == 5000,
      "(9) Verluste senken den Höchststand nicht, Bilanzen getrennt")
before = dict(lamabank._data["ledger"]), dict(lamabank._data["peak"])
lamabank._data["chips"] = 5
lamabank.refill_if_broke("casino", "slots")
check(lamabank.balance() == 1000 and lamabank.refills() == 1
      and (lamabank._data["ledger"], lamabank._data["peak"]) == before, "(9) Bank-Kredit zählt nicht zur Bilanz")
check(not lamabank.debit(5000, "poker") and lamabank.balance() == 1000, "(9) ungedeckter Einsatz wird abgelehnt")

lamabank._data["chips"] = 15
check(not lamabank.is_broke("blackjack") and lamabank.is_broke("poker", "holdem")
      and lamabank.is_broke("poker", "draw") and not lamabank.is_broke("poker", "video")
      and not lamabank.is_broke("casino", "roulette") and not lamabank.is_broke("casino", "slots"),
      "(10) Pleite je Mindesteinsatz (Blackjack 10, Poker 20, Video 10, Roulette 1, Slot 10)")
check(not lamabank.refill_if_broke("blackjack") and lamabank.refill_if_broke("poker", "holdem")
      and lamabank.balance() == 1000, "(10) Kredit nur bei Pleite im geöffneten Spiel")
lamabank._data["chips"] = 9
check(lamabank.is_broke("casino", "slots") and not lamabank.is_broke("casino", "roulette"),
      "(10) 9 Chips: am Slot pleite, am Roulette nicht")

# ================================================================ Spiele
print("\n[Spiele]")
SURF = pygame.Surface((800, 600))

# Blackjack: Einsatz nach Abbruch weg + Dealer-Karte aufgedeckt
fresh_mem()
g = quiet(bjm.BlackjackGame(SURF, 800, 600, game_settings=GS))
g.bet = 50
g._start_deal()
check(lamabank.balance() == 950, "(11) Blackjack bucht den Einsatz beim Geben ab")
lamabank._data = None
g2 = quiet(bjm.BlackjackGame(SURF, 800, 600, game_settings=GS))
check(g2.chips == 950 and lamabank.ledger("blackjack") == -50, "(11) Blackjack: Hand verlassen = Einsatz weg")
flips = []
for seed in range(40):
    random.seed(seed)
    fresh_mem()
    g = quiet(bjm.BlackjackGame(SURF, 800, 600, game_settings=GS))
    g.bet = 10
    g._start_deal()
    run(g, 1.5)
    if g.state == bjm.PLAYER:
        g._stand()
        run(g, 0.4)
    flips.append(all(c.face_up for c in g.dealer) and g.renderer.get(g.dealer[1], 40, 60)
                 is not g.renderer.back(40, 60))
check(all(flips), "(13) Blackjack: verdeckte Dealer-Karte wird beim Aufdecken umgedreht (40 Runden)")

# Poker: escrow + Pot-Anteil
fresh_mem()
events = []
p = quiet(pkm.PokerGame(SURF, 800, 600, mode="holdem", game_settings=GS), events)
p.n_opponents = 2
p.dealer = 1                      # Mensch sitzt im Big Blind
p._start_hand()
me = p.players[0]
posted = 1000 - me.stack
check(posted > 0 and lamabank.escrow() == me.stack and lamabank.balance() == 0,
      "(12) Poker: Tischstapel liegt im escrow, Blind abgebucht (%d)" % posted)
lamabank._data = None
p2 = quiet(pkm.PokerGame(SURF, 800, 600, mode="holdem", game_settings=GS))
check(p2.chips == 1000 - posted and lamabank.escrow() == 0 and lamabank.ledger("poker") == -posted,
      "(12) Poker: nächster Start erstattet den Stapel, der Pot-Anteil bleibt weg")
p2._start_hand()
p2.on_exit()
check(lamabank.escrow() == 0 and lamabank.balance() == p2.players[0].stack,
      "(12) Poker: Tisch verlassen legt den Reststapel sofort zurück")
fresh_mem()
events = []
p = quiet(pkm.PokerGame(SURF, 800, 600, mode="holdem", game_settings=GS), events)
random.seed(3)
for _ in range(400):
    if p.phase in (pkm.PREHAND,):
        p._start_hand()
    elif p.phase == pkm.ACTING:
        if p.players[p.turn].is_human:
            p._do_action("call" if p._to_call(p.players[0]) else "check")
        else:
            p.update(1.0)
    elif p.phase == pkm.SHOWDOWN:
        p._next_after_showdown()
    elif p.phase == pkm.BROKE:
        break
total_table = lamabank.balance() + lamabank.escrow()
check(lamabank.ledger("poker") == total_table - 1000,
      "(12) Poker: Bilanz = Konto + Tischstapel - 1000 nach vielen Händen", (lamabank.ledger("poker"), total_table))
rich = [e for e in events if e and e[0] == "poker_rich"]
check(bool(rich), "(14) poker_rich wird nach jeder Hand gemeldet")
fresh_mem()
lamabank.load()
lamabank.credit(3000, "casino")
events = []
p = quiet(pkm.PokerGame(SURF, 800, 600, mode="video", game_settings=GS), events)
p._video_deal()
p._video_draw()
check(events and events[-1][0] == "poker_rich" and events[-1][1] == 1000 + lamabank.ledger("poker")
      and events[-1][1] < 2000, "(14) poker_rich ignoriert Casino-Gewinne", events[-1:])

# Roulette: Einsatz, Plein-Erfolg, Abbruch
fresh_mem()
events = []
g = quiet(cas.CasinoGame(SURF, 800, 600, mode="roulette", game_settings=GS), events)
g.rng = FixedRng([17])
g._r_place("plein:17")
g._r_place("red")
g._r_place("red")
stake = g._r_total()
g._r_spin()
check(lamabank.balance() == 1000 - stake + 5 * 36 and g.pending == 180,
      "(11) Roulette: Einsatz abgebucht, Gewinn gutgeschrieben aber zurückgehalten")
shown_during = int(g._target_chips())
lamabank._data = None
g2 = quiet(cas.CasinoGame(SURF, 800, 600, mode="roulette", game_settings=GS))
check(g2.shown_chips == 1000 - stake + 180 and shown_during == 1000 - stake,
      "(11) Roulette: Abbruch mitten im Dreh - Einsatz weg, Ergebnis bleibt")
run(g, 6.5)
check(g.r_phase == "result" and ("roulette_plein",) in events and g.r_history[0] == 17,
      "(14) Roulette: Plein-Treffer meldet roulette_plein, Verlauf gespeichert")
diff = (g.r_ball[0] - g.r_rot - g.r_wheel.pocket_angle(17)) % math.tau
check(abs(g.r_ball[1] - g.r_wheel.pocket_r) < 1e-6 and min(diff, math.tau - diff) < 1e-6,
      "(2) Kugel liegt exakt im gezogenen Fach")
check(g.score == lamabank.score_for("casino") == 1000 - stake + 180, "(9) Casino-Highscore folgt der Bilanz")
ok_ball = True
for n in (0, 5, 26, 32):
    g.rng = FixedRng([n])
    g._r_new_round()
    g._r_place("plein:%d" % n)
    g._r_spin()
    run(g, 6.0)
    diff = (g.r_ball[0] - g.r_rot - g.r_wheel.pocket_angle(n)) % math.tau
    ok_ball = ok_ball and min(diff, math.tau - diff) < 1e-6 and g.r_number == n
check(ok_ball, "(2) Kugel landet für beliebige Zahlen im richtigen Fach")
g._r_new_round()
g._r_select_chip(100)
check(SAVED and SAVED[-1].get("roulette_chip") == 100, "(15) Chipwert wird in den Einstellungen gespeichert")

# Slot: Einsatz, Jackpot, Freispiele
fresh_mem()
events = []
g = quiet(cas.CasinoGame(SURF, 800, 600, mode="slots", game_settings=GS), events)
jack = [(s.index("lama") - 1) % 32 for s in L.STRIPS]
g.rng = FixedRng(jack)
g._s_spin()
win = L.evaluate(L.window(jack), 1)["total"]
check(lamabank.balance() == 1000 - 10 + win and g.pending == win, "(11) Slot: Einsatz abgebucht, Gewinn zurückgehalten")
run(g, 3.0)
check(("slots_jackpot",) in events and g.pending == 0, "(14) Slot: 5 Lamas melden slots_jackpot")
fresh_mem()
g = quiet(cas.CasinoGame(SURF, 800, 600, mode="slots", game_settings=GS))
coins = [s.index("coin") for s in L.STRIPS[:3]] + [0, 0]
g.rng = FixedRng(coins)
g._s_spin()
check(g.s_free == 10 and lamabank.get_extra("slots_free", {}).get("left") == 10,
      "(15) Slot: 3 Münzen = 10 Freispiele, sofort gespeichert")
lamabank._data = None
g2 = quiet(cas.CasinoGame(SURF, 800, 600, mode="slots", game_settings=GS))
check(g2.s_free == 10 and g2.s_fs_active and g2.s_free_bet == 1, "(15) Slot: Freispiele laufen nach dem Verlassen weiter")
bal = lamabank.balance()
g2.s_banners = []
g2.rng = random.Random(1)
g2._s_spin()
check(lamabank.balance() >= bal and g2.s_free == 9, "(15) Freispiel kostet nichts")
spins = 0
while (g2.s_free > 0 or g2.s_fs_active or g2.s_phase == "spin" or g2.s_banners) and spins < 3000:
    g2.update(1 / 20)
    spins += 1
check(g2.s_free == 0 and not g2.s_fs_active and lamabank.get_extra("slots_free") == {},
      "(15) Freispiel-Runde endet sauber")
g2._s_toggle_turbo()
g2._s_set_bet(1)
check(SAVED[-1].get("turbo") is True and SAVED[-1].get("line_bet") == 2, "(15) Turbo und Linien-Einsatz gespeichert")
fresh_mem()
lamabank.load()
lamabank._data["chips"] = 9
lamabank._save()
g = quiet(cas.CasinoGame(SURF, 800, 600, mode="slots", game_settings=GS))
check(g.broke, "(10) Slot mit 9 Chips zeigt PLEITE")
g.handle_event(InputEvent(InputEvent.KEYDOWN, key="Return"))
check(not g.broke and lamabank.balance() == 1000 and lamabank.refills() == 1, "(10) Enter nimmt den Bank-Kredit")
lamabank._data["chips"] = 9
lamabank._save()
g = quiet(cas.CasinoGame(SURF, 800, 600, mode="roulette", game_settings=GS))
check(not g.broke, "(10) Roulette mit 9 Chips ist nicht pleite")


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


def btn_fits(game, rect, label):
    w = min(game.f_btn.size(label)[0], game.f_tiny.size(label)[0])
    return w <= rect.w - 4


print("\n[Layout]")
t0 = time.perf_counter()
problems = []
for theme in ("v42", "v1"):
    ui.set_theme(theme)
    for code, _ in i18n.AVAILABLE:
        i18n.set_language(code, persist=False)
        langs = [code] if theme == "v42" else (["de", "fi"] if code in ("de", "fi") else [])
        if not langs:
            continue
        for (W, H) in RES:
            s = pygame.Surface((W, H))
            tag = "%s/%s/%dx%d" % (theme, code, W, H)
            try:
                fresh_mem()
                g = quiet(cas.CasinoGame(s, W, H, mode="roulette", game_settings=GS))
                hud_w = (g.margin * 4 + int(g.hud_h * 0.52) + 10 + g.f_big.size(i18n.t("cas.chips") + ": 100000")[0]
                         + min(g.f_small.size(i18n.t("cas.record") + ": 100000   ·   " + i18n.t("cas.credits", n=12))[0],
                               g.f_tiny.size(i18n.t("cas.record") + ": 100000   ·   " + i18n.t("cas.credits", n=12))[0]))
                if hud_w > W:
                    problems.append((tag, "HUD", hud_w))
                strip = [r for _, r in g.r_chip_rects] + list(g.r_btns.values())
                if not rects_ok(strip, W, H):
                    problems.append((tag, "Roulette-Leiste überlappt"))
                for key, r in g.r_btns.items():
                    if not btn_fits(g, r, g._r_btn_label(key)):
                        problems.append((tag, "Button", key))
                cell_w = g.r_uw * 2 - 6
                for key in ("cas.r.even", "cas.r.odd"):
                    lbl = ui.font(max(8, int(ui.font(max(9, int(min(g.r_uh * 0.34, g.r_uw * 0.30))), bold=True)
                                            .get_height() * 0.72)), bold=True).size(i18n.t(key))[0]
                    if lbl > cell_w + 2:
                        problems.append((tag, key, lbl, cell_w))
                p = g.r_panel
                if g.r_table_rect.right > W or g.r_table_rect.bottom > g.strip.y or p.w < 120:
                    problems.append((tag, "Tisch/Panel", g.r_table_rect, p))
                small9 = ui.font(max(9, g.f_tiny.get_height() - 3))
                for txt in (i18n.t("cas.r.place_hint"), i18n.t("cas.r.skip_hint"),
                            "%s 25-26-27-28-29-30 · 5:1 · 500" % i18n.t("cas.r.sixain"),
                            i18n.t("cas.r.dozen") + " 13–24 · 2:1 · 1000", i18n.t("cas.r.low") + " 1–18 · 1:1 · 100"):
                    if min(g.f_tiny.size(txt)[0], small9.size(txt)[0], ui.font(10).size(txt)[0]) > p.w + 2:
                        problems.append((tag, "Panel-Text", txt))
                for txt in (i18n.t("cas.r.place_bets"), i18n.t("cas.r.no_more"), i18n.t("cas.win") + ": +18000"):
                    if min(g.f_big.size(txt)[0], g.f_small.size(txt)[0]) > p.w:
                        problems.append((tag, "Meldung", txt))
                for n in (0, 17, 36):
                    desc = g._r_number_text(n)
                    if g.f_tiny.size(desc)[0] > p.w - 2 * max(14, min(int(34 * g.k), 40)) - 10:
                        problems.append((tag, "Zahlentext", desc))
                # Zustände zeichnen
                g.r_hover = "sixain:1-2-3-4-5-6"
                g._r_place("plein:17")
                g._r_place("dozen:2")
                g.draw()
                g.rng = FixedRng([17])
                g._r_spin()
                run(g, 2.0)
                g.draw()
                run(g, 4.0)
                g.draw()
                pw = min(W - 40, int(480 * max(0.8, g.k)))
                for txt in (i18n.t("cas.broke_sub"), i18n.t("cas.broke_restart", n=1000)):
                    if g.f_tiny.size(txt)[0] > pw - 16:
                        problems.append((tag, "Pleite-Text", txt))
                if g.f_huge.size(i18n.t("cas.broke"))[0] > pw:
                    problems.append((tag, "PLEITE", i18n.t("cas.broke")))
                g.broke = True
                g.toast = [i18n.t("cas.not_enough"), 1.0]
                g.draw()

                fresh_mem()
                g = quiet(cas.CasinoGame(s, W, H, mode="slots", game_settings=GS))
                strip = [g.s_minus, g.s_bet_box, g.s_plus] + list(g.s_btns.values())
                if not rects_ok(strip, W, H):
                    problems.append((tag, "Slot-Leiste überlappt"))
                labels = {"pay": i18n.t("cas.s.paytable"), "turbo": i18n.t("cas.s.turbo"),
                          "auto": i18n.t("cas.s.auto") + " 25", "spin": i18n.t("cas.spin")}
                for key, lbl in labels.items():
                    if not btn_fits(g, g.s_btns[key], lbl):
                        problems.append((tag, "Slot-Button", key, lbl))
                if not btn_fits(g, g.s_btns["spin"], i18n.t("cas.s.stop")):
                    problems.append((tag, "Slot-Button", "stop"))
                if g.f_tiny.size(i18n.t("cas.s.line_bet"))[0] > g.s_bet_box.w:
                    problems.append((tag, "Linien-Einsatz"))
                if not pygame.Rect(0, g.hud_h, W, g.strip.y - g.hud_h).contains(g.s_cab):
                    problems.append((tag, "Automat außerhalb", g.s_cab))
                if any(r.bottom > g.strip.y for r in [o[1] for o in g.s_auto_opts]) or g.s_auto_opts[-1][1].top < g.hud_h:
                    problems.append((tag, "Auto-Menü"))
                iw = g.s_info.w - 8
                for txt in (i18n.t("cas.s.hint"), i18n.t("cas.s.line_win", line=10, n=5, sym=i18n.t("cas.sym.clover"), win=1500),
                            i18n.t("cas.s.scatter_win", n=5, win=5000), i18n.t("cas.s.free_total", n=12000),
                            i18n.t("cas.s.auto_left", n=25), i18n.t("cas.s.good_luck")):
                    if g.f_tiny.size(txt)[0] > iw:
                        problems.append((tag, "Info-Zeile", txt))
                title = i18n.t("cas.s.free_title") + "  10"
                if ui.font(max(11, int(g.s_title_band.h * 0.52)), bold=True).size(title)[0] > g.s_title_band.w - 10:
                    problems.append((tag, "Titel", title))
                g.draw()
                g.rng = FixedRng(coins)
                g._s_spin()
                run(g, 0.5)
                g.draw()
                run(g, 3.0)
                g.draw()
                g.s_banners = []
                g._s_banner(i18n.t("cas.s.free_end"), "+12345", (255, 214, 90), 2.0, "win", 12345)
                g.s_banners[0]["t"] = 1.0
                g.draw()
                g.s_paytable = True
                g.draw()
                pay = g.s_pay_cache[1]
                for txt in (i18n.t("cas.s.wild_info"), i18n.t("cas.s.scatter_info"), i18n.t("cas.s.rtp", p=L.RTP_TEXT)):
                    if ui.font(max(9, g.f_tiny.get_height() - 3)).size(txt)[0] > pay.get_width() - 12:
                        problems.append((tag, "Gewinntabelle", txt))
                if ui.font(max(12, int(15 * g.k))).size(i18n.t("cas.s.paytable_title"))[0] > pay.get_width():
                    problems.append((tag, "Gewinntabelle-Titel"))
                g.s_paytable = False
                g.s_auto_menu = True
                g.draw()
                g.on_surface_changed()
            except Exception as exc:  # noqa: BLE001
                import traceback
                traceback.print_exc()
                problems.append((tag, "Ausnahme", repr(exc)))
ui.set_theme("v42")
i18n.set_language("de", persist=False)
check(not problems, "(16) Layout 5 Auflösungen x 14 Sprachen x 2 Modi (+ UI v1) (%.1fs)"
      % (time.perf_counter() - t0), problems[:12])

# Auflösungswechsel mitten im Dreh
fresh_mem()
s = pygame.Surface((800, 600))
g = quiet(cas.CasinoGame(s, 800, 600, mode="roulette", game_settings=GS))
g._r_place("red")
g._r_spin()
run(g, 1.0)
g.surface, g.width, g.height = pygame.Surface((480, 360)), 480, 360
g.on_surface_changed()
run(g, 6.0)
g.draw()
check(g.r_phase == "result" and g.r_uw < 40, "(16) Auflösungswechsel mitten im Roulette-Dreh")

# ================================================================ Tempo
print("\n[Tempo]")
for mode in ("roulette", "slots"):
    fresh_mem()
    s = pygame.Surface((1280, 960))
    g = quiet(cas.CasinoGame(s, 1280, 960, mode=mode, game_settings=GS))
    if mode == "roulette":
        for k in ("plein:17", "red", "dozen:1", "carre:1-2-4-5", "column:0", "cheval:8-11"):
            g._r_place(k)
        g.r_hover = "sixain:1-2-3-4-5-6"
        g._r_spin()
    else:
        g._s_spin()
    times = []
    for i in range(300):
        t1 = time.perf_counter()
        g.update(1 / 60)
        g.draw()
        ui.draw_fx(s, 1280, 960, 1 / 60)
        times.append(time.perf_counter() - t1)
        if mode == "slots" and g.s_phase != "spin" and i % 60 == 0:
            g.s_banners = []
            g._s_spin()
    times.sort()
    avg = sum(times) / len(times) * 1000
    p95 = times[int(len(times) * 0.95)] * 1000
    check(avg < 8.0 and p95 < 14.0, "(17) %s 1280x960: Ø %.2f ms, p95 %.2f ms pro Frame" % (mode, avg, p95))

# ================================================================ Texte
print("\n[Texte]")
langs = {}
for code, _ in i18n.AVAILABLE:
    sub = "" if code in ("de", "en", "fr", "es", "pt") else "lang.expansion"
    with open(os.path.join(REPO, "lang", sub, code + ".json"), encoding="utf-8") as f:
        langs[code] = json.load(f)
need = sorted(k for k in langs["de"] if k.startswith("cas."))
extra = ["ach.roulette_plein.name", "ach.roulette_plein.desc", "ach.slots_jackpot.name",
         "ach.slots_jackpot.desc", "ach.poker_rich.desc", "bj.chips", "poker.chips"]
missing = [(c, k) for c in langs for k in need + extra if not langs[c].get(k)]
check(len(need) >= 68 and not missing, "(18) %d Casino-Schlüssel in allen 14 Sprachen" % len(need), missing[:5])
wiki_bad = []
for code, _ in i18n.AVAILABLE:
    sub = "" if code in ("de", "en", "fr", "es", "pt") else "lang.expansion"
    with open(os.path.join(REPO, "lamawiki", sub, code + ".json"), encoding="utf-8") as f:
        pages = {p["id"]: p for p in json.load(f)["pages"]}
    if "casino" not in pages or pages["casino"].get("game") != "CasinoGame":
        wiki_bad.append((code, "casino"))
    for pid in ("blackjack", "poker"):
        text = json.dumps(pages.get(pid, {}), ensure_ascii=False)
        if "500" in text.replace("5000", "") and pid == "blackjack":
            wiki_bad.append((code, pid, "alte Startchips"))
check(not wiki_bad, "(18) LamaWiki: Casino-Seite in 14 Sprachen, Blackjack/Poker aktualisiert", wiki_bad)

for p in (MEM, MEM + ".bak", MEM + ".tmp"):
    if os.path.exists(p):
        os.remove(p)
print("\n%s  (%d Fehler)" % ("ALLES OK" if not FAILS else "FEHLGESCHLAGEN", len(FAILS)))
for f in FAILS:
    print("   -", f)
sys.exit(1 if FAILS else 0)
