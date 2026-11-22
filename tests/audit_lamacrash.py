# -*- coding: utf-8 -*-
"""Headless-Audit von Lama Crash (games/lamacrash.py).

Geprüft wird:

Bank      (1) SPIELEN bucht den Gesamteinsatz und speichert die Runde in einem Zug,
              Fairness rückt weiter, Platzpunkt = Seeds; nichts bei zu wenig Chips,
          (2) Aussteigen: Gutschrift + Speichern, Übertrag, Leerflug (COAST),
              Vorspulen, Abschluss genau einmal (Verlauf, Log, Statistik),
          (3) zwei Einsätze, Hälfte (H, Q/W, Knopf) einmal je Einsatz, MIN_CASH,
          (4) Auto-Ziel zahlt genau das Ziel (auch im Platz-Takt), Mondlandung,
          (5) Erfolge: Polarlicht-Pilot/Weltraumlama beim Aussteigen, Haarscharf nur
              von Hand und erst beim Abschluss,
Speichern (6) Fortsetzen nach dem Verlassen am selben Takt, Abbruch ohne on_exit
              zahlt nichts doppelt, Verlassen im Leerflug schließt ab, Laden einer
              fertigen Runde schließt ab, kaputte Runde fällt weg,
Fairness  (7) Zwillingsrunden: x5 und x50 sind bis x4,99 Pixel für Pixel und Ton für
              Ton gleich (nichts verrät den Platzpunkt), Fairness-Fenster rechnet nach,
          (8) Zeit: Takte unabhängig von der Bildrate (30/60/144 Hz), Turbo x3,
              Vorspulen x16, dt-Deckel, Pause friert ein, pause_on_blur,
Bedienung (9) Auto-Modus (Ziele erzwungen, Runden, Grenzen, Stopp, Eingreifen),
          (10) Tasten roh, Wiederholung ignoriert, gesperrte Fenster im Flug,
              Einstellungen + Schema, (11) Maus, (12) Pleite -> Bank-Kredit,
Layout    (13) 5 Auflösungen x 14 Sprachen (v4.2 + v1): keine Überlappung, Texte
              passen, alle Zustände und Fenster zeichnen,
Texte     (14) alle Schlüssel in 14 Sprachen, Wortfilter, Wiki-Seite.

Aufruf aus dem Repo-Root:  python tests/audit_lamacrash.py
"""
import hashlib
import json
import os
import random
import re
import shutil
import sys
import tempfile

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
TMP = os.environ.get("PGZ_AUDIT_DIR") or tempfile.mkdtemp(prefix="pgz-lcx-")
os.makedirs(TMP, exist_ok=True)
MEM = os.path.join(TMP, "_audit-lamacrash-mem.json")
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
settings_mod.save_settings = lambda s: SAVED.append(json.loads(json.dumps(s.get("lamacrash", {}))))
import i18n
i18n.init()
import ui
import audio

SOUND = []                     # Ton-Protokoll (für den Zwillings-Test)
audio.tone = lambda *a, **k: SOUND.append(("tone",) + tuple(round(v, 3) if isinstance(v, float) else v
                                                            for v in a[:2]))
_music_play = audio.music_play
audio.music_play = lambda voices, owner, settings=None, volume=1.0: (
    SOUND.append(("music", len(voices or []), round(volume, 2))) or False)
audio.music_stop = lambda owner=None: SOUND.append(("music_stop",))
audio.music_set_volume = lambda v: SOUND.append(("music_vol", round(v, 2)))

import chipstats
import fairness
import lamabank
import swear
from game_base import InputEvent
from games import lamacrash as LC
from games import lamacrash_draw as D
from games import lamacrash_logic as L

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


def game(W=800, H=600, events=None, sounds=None, **opts):
    GS["lamacrash"] = dict(settings_mod.DEFAULTS["lamacrash"], **opts)
    g = LC.LamaCrashGame(pygame.Surface((W, H)), W, H, game_settings=GS)
    g.play_sound = (lambda name: sounds.append(("sound", name))) if sounds is not None else (lambda name: None)
    g.rumble = lambda ms=0: None
    g.ach_event = (lambda *a, **k: events.append(a[0])) if events is not None else (lambda *a, **k: None)
    return g


def key(g, k, repeat=False):
    g.handle_event(InputEvent(InputEvent.KEYDOWN, key=k, repeat=repeat))


def click(g, pos):
    g.handle_event(InputEvent(InputEvent.MOUSEDOWN, pos=pos, button=1))


def wheel(g, pos, d):
    g.handle_event(InputEvent(InputEvent.MOUSEMOVE, pos=pos))
    g.handle_event(InputEvent(InputEvent.WHEEL, delta=d))


def run(g, seconds, dt=1 / 60):
    for _ in range(int(round(seconds / dt))):
        g.update(dt)


def mem_extra():
    """Gespeicherter Stand (frisch von der Platte, aktives Profil)."""
    cas = store.load_section("casino")
    return cas["extra"]["lamacrash"], cas


def launch(g, crash=None):
    """SPIELEN, Countdown abwarten; optional einen Platzpunkt erzwingen (nur Tests)."""
    key(g, "space")
    if crash is not None:
        g._crash = crash
    while g.state == LC.COUNT:
        g.update(1 / 60)


_SEEDS = {}                    # (Platzpunkt, Nonce) -> (Server-Seed, Client-Seed)


def seed_crash(g, crash):
    """Fairness-Seeds der nächsten Runde so einsetzen, dass ihr ECHTER Platzpunkt genau
    crash ist (feste Suchfolge, kein Zufall); danach launch(g) ohne crash.

    Nötig, sobald eine Runde gespeichert und wieder geladen wird: L.clean_round rechnet
    den Platzpunkt aus den Seeds nach und verwirft Runden, deren Takt oder Ausstiege
    darüber liegen - ein nur über g._crash erzwungener Platzpunkt hält dem nicht stand
    (das Spiel selbst speichert so eine Runde nie)."""
    k = (crash, g.fair["nonce"])
    if k not in _SEEDS:
        rng = random.Random(crash)
        client = fairness.new_client_seed(rng)
        server = fairness.new_server_seed(rng)
        while L.crash_point(server, client, k[1]) != crash:
            server = fairness.new_server_seed(rng)
        _SEEDS[k] = (server, client)
    g.fair["server"], g.fair["client"] = _SEEDS[k]


def fly_to(g, m100, dt=1 / 60):
    """Fliegen, bis der Multiplikator m100 erreicht ist (oder die Runde endet)."""
    while g.state in (LC.FLY, LC.COAST) and g.round is not None and L.mult_at(g.round["t"]) < m100:
        g.update(dt)


# ================================================================ Bank
print("\n[Bank]")
fresh_mem()
SAVES = []
_orig_save = store.save_section
store.save_section = lambda sec, data: (SAVES.append(sec), _orig_save(sec, data))[1]
g = game(two=True, bet1=50, bet2=20)
SAVES.clear()
fair_before = dict(g.fair)
key(g, "space")
ex, cas = mem_extra()
check(g.state == LC.COUNT and cas["chips"] == 930 and SAVES == ["casino"] and ex["round"] is not None
      and len(ex["round"]["bets"]) == 2 and ex["round"]["t"] == 0,
      "(1) SPIELEN: 70 Chips abgebucht, Runde mit 2 Einsätzen in EINEM Speichern", (cas["chips"], SAVES))
check(ex["round"]["n"] == fair_before["nonce"] and ex["round"]["s"] == fair_before["server"]
      and ex["fair"]["nonce"] == fair_before["nonce"] + 1 and ex["fair"]["server"] != fair_before["server"]
      and g._crash == L.crash_point(ex["round"]["s"], ex["round"]["c"], ex["round"]["n"]),
      "(1) Fairness: Runde nutzt den gezeigten Prüfcode, nächster steht fest, Platzpunkt = Seeds")
store.save_section = _orig_save
fresh_mem({"mem": {"lang": "de"}, "casino": {"chips": 25, "migrated": True}})
g = game(two=True, bet1=20, bet2=20)
check(g.bets == [20, 10] or g._total() <= 25, "(1) zu wenig Chips: Einsatz 2 wird zuerst gesenkt", g.bets)
g.bets = [20, 20]
key(g, "space")
check(g.state == LC.IDLE and g.round is None and lamabank.balance() == 25, "(1) Einsatz > Konto: keine Runde")

# ================================================================ Aussteigen
print("\n[Aussteigen]")
fresh_mem()
ev = []
g = game(events=ev, bet1=100)
launch(g, crash=5000)
fly_to(g, 237)
m = L.mult_at(g.round["t"])
key(g, "space")
ex, cas = mem_extra()
check(g.state == LC.COAST and cas["chips"] == 900 + 100 * m // 100 and ex["round"]["bets"][0]["xm"] == m
      and ex["round"]["bets"][0]["man"] == 2, "(2) Aussteigen bei %s: Gutschrift + Speichern, Leerflug" % m)
check(not g.hist and not g.fairlog and g.stats["rounds"] == 0, "(2) Abschluss erst beim Platzen (nichts verraten)")
key(g, "space")
check(g.ff, "(2) Leertaste im Leerflug = Vorspulen")
t0 = g.round["t"]
g.update(1 / 60)
check(g.round is None or g.round["t"] - t0 == L.SPEED_FF, "(2) Vorspulen = 16 Takte je Bild")
run(g, 20)
check(g.state in (LC.END, LC.IDLE) and g.hist == [5000] and len(g.fairlog) == 1 and g.stats["rounds"] == 1
      and g.stats["paid"] == 100 * m // 100 and g.stats["best_mult"] == m and g.round is None,
      "(2) Platzen: Verlauf, Fairness-Log, Statistik genau einmal")
g._finish("pop")
check(len(g.fairlog) == 1 and g.stats["rounds"] == 1, "(2) Abschluss ist idempotent")
check("lcx_close" not in ev and "lcx_brave" not in ev, "(5) keine Erfolge bei x%s (Platzpunkt x50)" % L.fmt_mult(m))

# ================================================================ Zwei Einsätze & Hälfte
print("\n[Zwei Einsätze]")
fresh_mem()
g = game(two=True, bet1=50, bet2=20, auto1=False, auto2=False)
launch(g, crash=3000)
key(g, "h")
check(all(b["hm"] == 0 for b in g.round["bets"]), "(3) Hälfte unter x1,01 nicht möglich (MIN_CASH)")
fly_to(g, 150)
m1 = L.mult_at(g.round["t"])
key(g, "q")
b0 = g.round["bets"][0]
check(b0["hm"] == m1 and b0["man"] == 1 and L.riding(b0) == 25, "(3) Q = Hälfte von Einsatz 1")
key(g, "q")
check(b0["hm"] == m1 and lamabank.balance() == 930 + 25 * m1 // 100, "(3) Hälfte nur einmal je Einsatz")
fly_to(g, 200)
key(g, "h")
b1 = g.round["bets"][1]
m2 = L.mult_at(g.round["t"])
check(b1["hm"] == m2 and L.riding(b1) == 10, "(3) H = Hälfte aller ungeteilten (nur noch Einsatz 2)")
fly_to(g, 260)
key(g, "2")
check(b1["xm"] > 0 and L.riding(b1) == 0 and L.riding(b0) == 25 and g.state == LC.FLY, "(3) 2 = Einsatz 2 ganz raus")
key(g, "Return")
check(b0["xm"] > 0 and g.state == LC.COAST and b0["p"] + b1["p"] == L.paid_total(g.round),
      "(3) Enter = alle restlichen raus -> Leerflug")
paid = L.paid_total(g.round)
chips_now = lamabank.balance()
check(chips_now == 930 + paid, "(3) Konto = Einsatz weg + alle Auszahlungen", (chips_now, paid))
key(g, "space")                                  # vorspulen bis zum Platzen
run(g, 40)
check(g.last is not None and g.last["paid"] == paid and g.stats["x"]["half"] == 2 and g.stats["x"]["cash"] == 4,
      "(3) Zähler: 2 Hälften, 4 Ausstiege")

# ================================================================ Auto-Ziel & Mond
print("\n[Auto-Ziel]")
fresh_mem()
ev = []
g = game(events=ev, two=True, bet1=10, bet2=10, auto1=True, target1=350, auto2=True, target2=351)
launch(g, crash=350)
run(g, 60)
bets = g.last["rnd"]["bets"]
check(bets[0]["xm"] == 350 and bets[0]["p"] == 35 and bets[1]["xm"] == 0 and g.last["end"] == "pop",
      "(4) Ziel = Platzpunkt gewinnt genau x3,50, Ziel x3,51 verliert")
fresh_mem()
ev = []
g = game(events=ev, two=True, bet1=10, bet2=20, auto1=True, target1=1000, auto2=False)
launch(g, crash=L.MAX_MULT100)
g.turbo = True
for _ in range(80 * 60):
    g.update(1 / 60)
    if g.state == LC.END:
        break
bets = g.last["rnd"]["bets"] if g.last else []
check(g.last is not None and g.last["end"] == "moon" and bets[0]["xm"] == 1000 and bets[1]["xm"] == L.MAX_MULT100
      and bets[1]["p"] == 20 * 100000 and g.stats["x"]["moon"] == 1 and g.banner is not None,
      "(4) Mondlandung: Ziel x10 vorher, Rest x100.000 (2.000.000 Chips)", bets)
check(ev.count("lcx_brave") >= 1 and "lcx_space" in ev, "(5) Polarlicht-Pilot und Weltraumlama beim Aussteigen", ev)

# Haarscharf: nur von Hand
fresh_mem()
ev = []
g = game(events=ev, bet1=10, auto1=True, target1=300)
launch(g, crash=310)
run(g, 40)
check("lcx_close" not in ev, "(5) Haarscharf nicht mit Auto-Ziel (x3,00 bei Platzpunkt x3,10)")
fresh_mem()
ev = []
g = game(events=ev, bet1=10, auto1=False)
launch(g, crash=310)
fly_to(g, 300)
key(g, "space")
check("lcx_close" not in ev, "(5) Haarscharf erst beim Abschluss (nicht beim Aussteigen)")
run(g, 40)
check(ev.count("lcx_close") == 1 and g.stats["x"]["close"] == 1, "(5) Haarscharf von Hand bei x3,00 / x3,10", ev)

# ================================================================ Speichern
print("\n[Speichern]")
fresh_mem()
g = game(bet1=100)
seed_crash(g, 900)
launch(g)
fly_to(g, 180)
t_exit = g.round["t"]
g.on_exit()
ex, cas = mem_extra()
check(ex["round"] is not None and ex["round"]["t"] == t_exit and cas["chips"] == 900,
      "(6) Verlassen im Flug: Takt %d gespeichert, Einsatz bleibt gebucht" % t_exit)
g2 = game(bet1=100)
check(g2.state == LC.COUNT and g2.round["t"] == t_exit and g2.toast is not None, "(6) Neustart: Countdown, gleicher Takt")
run(g2, 2.0)
check(g2.state == LC.FLY and g2.round["t"] > t_exit, "(6) Fortsetzen fliegt weiter")
fly_to(g2, 250)
key(g2, "space")
pay = 100 * L.mult_at(g2.round["t"]) // 100
chips = lamabank.balance()
check(chips == 900 + pay, "(6) Ausstieg nach dem Fortsetzen")
# Abbruch ohne on_exit (Spiel "stürzt ab"): neue Instanz liest den gespeicherten Stand
g3 = game(bet1=100)
check(g3.round is None and lamabank.balance() == chips and len(g3.fairlog) == 1,
      "(6) Laden einer Runde ohne fliegendes Geld schließt sofort ab, keine Doppelzahlung")
fresh_mem()
g = game(bet1=100)
seed_crash(g, 900)
launch(g)
fly_to(g, 120)
key(g, "space")
check(g.state == LC.COAST, "(6) Leerflug vor dem Verlassen")
g.on_exit()
ex, cas = mem_extra()
check(ex["round"] is None and len(ex["fairlog"]) == 1 and ex["hist"] == [900] and ex["stats"]["rounds"] == 1,
      "(6) Verlassen im Leerflug schließt die Runde ab")
fresh_mem()
g = game(bet1=100)
seed_crash(g, 900)
launch(g)
fly_to(g, 150)
g.on_exit()
cas = store.load_section("casino")
cas["extra"]["lamacrash"]["round"]["h"] = "0" * 64
store.save_section("casino", cas)
lamabank._data = None
g = game(bet1=100)
check(g.round is None and g.state == LC.IDLE, "(6) manipulierte Runde (falscher Prüfcode) fällt weg")

# ================================================================ Fairness / Zwillinge
print("\n[Fairness]")


def twin(crash, frames):
    fresh_mem()
    SOUND.clear()
    snd = []
    g = game(sounds=snd, two=True, bet1=10, bet2=10, auto1=False, auto2=True, target2=10000)
    g.rng = random.Random(7)
    key(g, "space")
    g._crash = crash
    hashes = []
    for _ in range(frames):
        g.update(1 / 60)
        g.draw()
        hashes.append(hashlib.sha1(pygame.image.tobytes(g.surface, "RGB")).hexdigest())
    return hashes, list(SOUND) + snd, g


lim = L.tick_at(499) - 1
frames = int((lim + 1) / 1.0) + int(LC.COUNT_T * 60) - 4
h5, s5, g5 = twin(500, frames)
h50, s50, g50 = twin(5000, frames)
check(g5.state == LC.FLY and L.mult_at(g5.round["t"]) <= 499, "(7) Zwilling x5 ist noch in der Luft (bis x4,99)")
check(h5 == h50, "(7) Zwillingsrunden x5 / x50: %d Bilder Pixel für Pixel gleich" % frames,
      next((i for i, (a, b) in enumerate(zip(h5, h50)) if a != b), None))
check(s5 == s50 and len(s5) > 3, "(7) Zwillingsrunden: Töne und Musik gleich (%d Einträge)" % len(s5))
fresh_mem()
g = game(bet1=10)
for _ in range(3):
    launch(g)
    run(g, 200)
    g.turbo = True
check(len(g.fairlog) == 3, "(7) drei echte Runden im Log")
for i in range(len(g.fairlog)):
    g._verify(i)
check(all(v[0] for v in g.fair_checks.values()) and len(g.fair_checks) == 3,
      "(7) Fairness-Fenster: alle Runden nachgerechnet (Prüfcode + Platzpunkt + Ausstiege)")
bad = dict(g.fairlog[0], o=dict(g.fairlog[0]["o"], x=g.fairlog[0]["o"]["x"] + 1))
g.fairlog[0] = bad
g._verify(0)
check(not g.fair_checks[bad["n"]][0], "(7) verfälschter Platzpunkt wird erkannt")

# ================================================================ Zeit
print("\n[Zeit]")
res = []
for hz in (30, 60, 144):
    fresh_mem()
    g = game(bet1=100, auto1=True, target1=777)
    launch(g, crash=2000)
    n = 0
    while g.state == LC.FLY and n < 100000:
        g.update(1.0 / hz)
        n += 1
    res.append((g.round["bets"][0]["xm"] if g.round else None, g.round["bets"][0]["p"] if g.round else None,
                g.round["t"] if g.round else None))
check(res[0] == res[1] == res[2] and res[0][0] == 777, "(8) Auto-Ziel bei 30/60/144 Hz gleich ausgezahlt", res)
fresh_mem()
g = game(bet1=10)
launch(g, crash=L.MAX_MULT100)
t0 = g.round["t"]
run(g, 2.0)
ticks_n = g.round["t"] - t0
g.turbo = True
t0 = g.round["t"]
run(g, 2.0)
ticks_t = g.round["t"] - t0
check(abs(ticks_n - 120) <= 1 and abs(ticks_t - 360) <= 1, "(8) Normal 60 Takte/s, Turbo 180 Takte/s", (ticks_n, ticks_t))
t0 = g.round["t"]
g.update(5.0)
check(g.round["t"] - t0 <= int(0.1 * 60 * 3) + 1, "(8) ein Riesen-dt wird nicht nachgeholt (Deckel 0,1 s im Spiel)",
      g.round["t"] - t0)
check(g.pause_on_blur, "(8) pause_on_blur, solange Geld fliegt")
key(g, "space")
check(not g.pause_on_blur and g.state == LC.COAST, "(8) kein pause_on_blur im Leerflug")

# ================================================================ Auto-Modus
print("\n[Auto-Modus]")
fresh_mem()
g = game(two=True, bet1=10, bet2=10, auto1=False, auto2=False, auto_rounds=5)
key(g, "a")
check(g.window == "auto", "(9) A öffnet das Auto-Fenster")
key(g, "Return")
check(g.auto is not None and g.autos == [True, True] and g.window is None, "(9) START erzwingt Auto-Ziele")
rounds_before = g.stats["rounds"]
for _ in range(20000):
    g.update(1 / 60)
    if g.auto is None:
        break
check(g.auto is None and g.stats["rounds"] - rounds_before == 5, "(9) 5 Runden, dann Ende",
      g.stats["rounds"] - rounds_before)
fresh_mem()
g = game(bet1=10, auto1=True, target1=10000000, auto_rounds=50, auto_loss=100, turbo=True)
# feste Fairness-Seeds (keine der zehn Runden erreicht den Mond): mit zufälligen landet
# etwa jeder 10.000. Lauf vorher auf dem Mond - Gewinn statt Verlust, kein Stopp
g.seed_rng = random.Random(9)
g.fair.update(fairness.blank_state(g.seed_rng))
g._start_auto()
for _ in range(200000):
    g.update(1 / 60)
    if g.auto is None:
        break
check(g.auto is None and g.stats["rounds"] == 10 and g.toast is not None,
      "(9) Stopp bei Verlust ab 100 (10 x 10 Chips verloren)", g.stats["rounds"])
fresh_mem()
g = game(bet1=10, auto1=True, target1=100000, auto_rounds=10)
g._start_auto()
while g.state != LC.FLY:
    g.update(1 / 60)
g._crash = 5000
fly_to(g, 150)
key(g, "space")
check(g.state == LC.COAST and g.auto is not None, "(9) Eingreifen von Hand im Auto-Modus (Leertaste = aussteigen)")
key(g, "Escape")
check(g.auto is None and g.round is not None, "(9) Esc stoppt Auto, die Runde fliegt zu Ende")

# ================================================================ Tasten
print("\n[Tasten]")
fresh_mem()
SAVED.clear()
g = game(two=False, bet1=10)
key(g, "Up")
key(g, "plus")
key(g, "Right")
key(g, "x")
key(g, "z")
key(g, "2")
key(g, "Down")
check(g.bets == [50, 10] and g.targets[0] == 225 and g.autos[0] is False and g.two and g.sel == 1
      and SAVED and SAVED[-1].get("two") is True, "(10) Einsatz/Ziel/Schalter/Karte per Taste, gespeichert",
      (g.bets, g.targets, g.autos, g.two, g.sel))
key(g, "space", repeat=True)
check(g.state == LC.IDLE, "(10) gehaltene Leertaste startet keine Runde")
key(g, "space")
while g.state == LC.COUNT:
    g.update(1 / 60)
g._crash = 9000
fly_to(g, 150)
key(g, "space", repeat=True)
check(L.flying(g.round), "(10) Tastenwiederholung steigt nicht aus")
key(g, "s")
check(g.window is None and g.toast is not None, "(10) Fenster im Flug gesperrt")
key(g, "space")
key(g, "s")
check(g.window == "stats", "(10) im Leerflug darf die Statistik auf")
key(g, "Escape")
key(g, "t")
check(g.turbo, "(10) T = Turbo")
key(g, "m")
check(not g.music_on, "(10) M = Musik aus")
cfg = settings_mod._merge_defaults({"lamacrash": {"bet1": 7, "target1": 50, "two": "ja", "auto_rounds": 3, "music": False}}) \
    if hasattr(settings_mod, "_merge_defaults") else None
check(cfg is None or (cfg["lamacrash"]["bet1"] == 10 and cfg["lamacrash"]["target1"] == 101
                      and cfg["lamacrash"]["two"] is False and cfg["lamacrash"]["auto_rounds"] == 10
                      and cfg["lamacrash"]["music"] is False), "(10) Schema prüft die Einstellungen",
      cfg and cfg["lamacrash"])

# ================================================================ Maus
print("\n[Maus]")
fresh_mem()
g = game(two=False, bet1=10)
c1, c2 = g.cards
click(g, c1["plus"].center)
click(g, c1["tr"].center)
click(g, c2["rect"].center)
check(g.bets[0] == 20 and g.autos[0] and g.two, "(11) Klick: Einsatz +, Ziel ▶ schaltet Auto ein, Karte 2 an")
wheel(g, c2["tv"].center, 1)
check(g.targets[1] == L.step_target(1000, 1), "(11) Mausrad über dem Ziel ändert das Ziel")
click(g, g.r_go.center)
check(g.state == LC.COUNT, "(11) Klick auf SPIELEN")
while g.state == LC.COUNT:
    g.update(1 / 60)
g._crash = 9000
fly_to(g, 160)
click(g, c1["halfc"].center)
check(g.round["bets"][0]["hm"] > 0, "(11) Klick auf ½ der Karte")
click(g, c2["cash"].center)
check(g.round["bets"][1]["xm"] > 0, "(11) Klick auf AUSSTEIGEN der Karte")
click(g, g.r_go.center)
check(g.state == LC.COAST, "(11) Hauptknopf im Flug = alle raus")
click(g, g.r_go.center)
check(g.ff, "(11) Hauptknopf im Leerflug = Vorspulen")
for name, r in g.r_tools:
    if name in ("stats", "fair"):
        click(g, r.center)
        check(g.window == name, "(11) Knopf %s" % name)
        click(g, (2, g.height - 2))
        check(g.window is None, "(11) Klick daneben schließt %s" % name)

# ================================================================ Pleite
print("\n[Pleite]")
fresh_mem({"mem": {"lang": "de"}, "casino": {"chips": 5, "migrated": True}})
g = game(bet1=10)
check(g.broke, "(12) unter 10 Chips: Pleite-Anzeige")
key(g, "Return")
check(not g.broke and lamabank.balance() == 1000 and lamabank.refills() == 1, "(12) Enter nimmt den Bank-Kredit")


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
                g = game(W, H, two=True, bet1=10, bet2=10)
                scr = pygame.Rect(0, 0, W, H)
                if not rects_ok([g.r_total, g.r_half, g.r_go], g.strip):
                    problems.append((tag, "Leiste überlappt"))
                if not rects_ok([r for _, r in g.r_tools] + [g.stage, g.panel_r], scr):
                    problems.append((tag, "Werkzeuge/Bühne/Seitenleiste überlappen"))
                if g.stage.bottom > g.strip.y or g.panel_r.bottom > g.strip.y:
                    problems.append((tag, "Bühne reicht in die Leiste"))
                for c in g.cards:
                    if not g.panel_r.contains(c["rect"]):
                        problems.append((tag, "Karte außerhalb"))
                    if not rects_ok([c["minus"], c["bet"], c["plus"], c["auto"], c["tl"], c["tv"], c["tr"]], c["rect"]):
                        problems.append((tag, "Karte überlappt (Ruhe)"))
                    if not rects_ok([c["cash"], c["halfc"]], c["rect"]) or not c["rect"].contains(c["sw"]):
                        problems.append((tag, "Karte überlappt (Flug)"))
                if g.cards[1]["rect"].bottom > g.info_y or g.info_y > g.panel_r.bottom:
                    problems.append((tag, "Info-Kasten"))
                if not g.plot.w > 40 or not g.stage.contains(g.plot):
                    problems.append((tag, "Kurvenfläche"))
                auto = [r for _, row in g.auto_rows for r in row] + [g.r_auto_close, g.r_auto_turbo, g.r_auto_start]
                if not rects_ok(auto, g.auto_r) or g.auto_sum_y + g.f_tiny.get_height() > g.r_auto_start.y:
                    problems.append((tag, "Auto-Fenster überlappt"))
                big = (g.f_btn, g.f_tiny, g.f_micro)
                for rect, lbl in ((g.r_go, T("lcx.play")), (g.r_go, T("chip.auto_stop")),
                                  (g.r_go, T("lcx.cash_all") + " 99.999"), (g.r_go, T("lcx.ff")),
                                  (g.r_auto_start, T("chip.auto_start")),
                                  (g.cards[0]["cash"], T("lcx.cash") + " 1.000")):
                    if not fits(big, lbl, rect.w - 6 - (rect.h if rect is g.r_go else 0)):
                        problems.append((tag, "Knopf", lbl))
                if g.half_label and not fits(big, T("lcx.half_all"), g.r_half.w - g.r_half.h):
                    problems.append((tag, "Hälfte-Knopf"))
                x0, w = g.panel_inner
                for lab in (T("lcx.total"), T("lcx.chance", x="×1.000,00"), T("chip.max_win"), T("lcx.world"),
                            T("lcx.flight"), T("chip.st_paid"), T("lcx.crash_at"), T("chip.st_net")):
                    if not fits((g.f_small, g.f_tiny, g.f_micro), lab, w - g.f_btn.size("100.000")[0] - 8):
                        problems.append((tag, "Info-Zeile", lab))
                for lab in (T("lcx.bet_n", n=2) + "  ·  10.000",):
                    if not fits((g.f_tiny, g.f_micro), lab, g.cards[1]["rect"].w - g.cards[1]["sw"].w - 30):
                        problems.append((tag, "Kartentitel", lab))
                for lab in (T("chip.auto_rounds"), T("chip.auto_stop_win"), T("chip.auto_stop_loss"),
                            T("chip.auto_title")):
                    if not fits((g.f_big, g.f_btn, g.f_tiny, g.f_micro), lab, g.auto_r.w - g.r_auto_close.w - 30):
                        problems.append((tag, "Auto", lab))
                for txt in (T("lcx.hint_idle"), T("lcx.hint_fly"), T("lcx.hint_coast")):
                    if g.hint_y < g.panel_r.bottom - g.ppad and len(g._wrap(txt, g.f_micro, w)) > 2:
                        problems.append((tag, "Hinweis zu lang", txt))
                for txt in (T("lcx.resumed"), T("cas.not_enough"), T("lcx.press_play"), T("lcx.locked"),
                            T("lcx.locked_fly"), T("chip.auto_hit_loss"), T("lcx.auto_forced"), T("lcx.close")):
                    if g.f_tiny.size(txt)[0] + 24 > W - 8:
                        problems.append((tag, "Hinweis", txt))
                for txt in (T("lcx.popped"), T("lcx.moon"), T("lcx.coast"), T("lcx.win_now", n="99.999.999")):
                    if not fits((g.f_huge, g.f_mult, g.f_big, g.f_btn, g.f_tiny), txt, g.stage.w - 16):
                        problems.append((tag, "Bühne", txt))
                if g.tool_labels and g.tools_right > g.stage.right:
                    problems.append((tag, "Werkzeug-Beschriftung"))
                # alle Zustände zeichnen
                g.draw()
                key(g, "space")
                g._crash = 1200
                for _ in range(3):
                    run(g, 0.5)
                    g.draw()
                fly_to(g, 160)
                g.mouse = g.r_half.center
                g.draw()
                key(g, "q")
                fly_to(g, 300)
                g.draw()
                key(g, "space")
                g.draw()
                key(g, "space")
                for _ in range(6):
                    run(g, 0.25)
                    g.draw()
                run(g, 3)
                g.draw()
                key(g, "s")
                g.draw()
                key(g, "Escape")
                key(g, "f")
                if g.fairlog:
                    g._verify(len(g.fairlog) - 1)
                g.draw()
                key(g, "Escape")
                key(g, "a")
                g.draw()
                key(g, "Escape")
                # Mondlandung zeichnen
                key(g, "space")
                g._crash = L.MAX_MULT100
                while g.state == LC.COUNT:
                    g.update(1 / 60)
                g.round["t"] = L.MOON_TICK - 2
                run(g, 0.2)
                for _ in range(3):
                    run(g, 0.3)
                    g.draw()
                g.on_surface_changed()
                g.draw()
                g.broke = True
                g.toast = [T("cas.not_enough"), 1.0]
                g.draw()
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
src = open(os.path.join(REPO, "games", "lamacrash.py"), encoding="utf-8").read()
used = sorted(set(re.findall(r'"(lcx\.[a-z_]+)"', src)))
need = used + ["lcx.world_" + w for w in L.WORLDS] + ["lcx.music"]
need += ["ach.%s.%s" % (a, f) for a in ("lcx_close", "lcx_brave", "lcx_space") for f in ("name", "desc")]
need += sorted(set(re.findall(r'"(chip\.[a-z_]+)"', src))) + sorted(set(re.findall(r'"(cas\.[a-z_.]+)"', src)))
need = sorted(set(need))
missing = [(c, k) for c in langs for k in need if not langs[c].get(k)]
check(len(used) >= 50 and not missing, "(14) %d Lama-Crash-Schlüssel in allen 14 Sprachen" % len(need), missing[:5])
dirty = [(c, k) for c in langs for k in need if not swear.is_clean(langs[c].get(k, ""))]
check(not dirty, "(14) Wortfilter: alle Texte sauber", dirty[:5])
check(all(langs[c].get("lcx.decimal") in (",", ".") for c in langs), "(14) Dezimaltrennzeichen je Sprache")
ph = [(c, k) for c in langs for k in need
      if sorted(re.findall(r"\{[a-z]+\}", langs[c].get(k, ""))) != sorted(re.findall(r"\{[a-z]+\}", langs["de"].get(k, "")))]
check(not ph, "(14) Platzhalter je Sprache gleich", ph[:5])
wiki_bad = []
for code, _ in i18n.AVAILABLE:
    sub = "" if code in ("de", "en", "fr", "es", "pt") else "lang.expansion"
    with open(os.path.join(REPO, "lamawiki", sub, code + ".json"), encoding="utf-8") as f:
        pages = {p["id"]: p for p in json.load(f)["pages"]}
    if pages.get("lamacrash", {}).get("game") != "LamaCrashGame":
        wiki_bad.append(code)
check(not wiki_bad, "(14) LamaWiki-Seite in 14 Sprachen", wiki_bad)

for p in (MEM, MEM + ".bak", MEM + ".tmp"):
    if os.path.exists(p):
        os.remove(p)
shutil.rmtree(store._DIR, ignore_errors=True)
if not os.environ.get("PGZ_AUDIT_DIR"):
    shutil.rmtree(TMP, ignore_errors=True)
D.clear_cache()
audio.music_play = _music_play
print("\n%s - %d Fehler" % ("BESTANDEN" if not FAILS else "FEHLGESCHLAGEN", len(FAILS)))
sys.exit(1 if FAILS else 0)
