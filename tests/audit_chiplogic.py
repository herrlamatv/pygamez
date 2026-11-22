# -*- coding: utf-8 -*-
"""Headless-Audit der Chip-Regeln von Lama Mines, Lama Plinko und Lama Crash (ohne pygame).

Geprüft wird:

Fairness  (1) SHA-256-Prüfvektoren ("", "abc", 448 Bit, 1000 x "a", Blockgrenzen),
          (2) neue Seeds: Form, Alphabet, gleicher Startwert = gleiche Seeds,
          (3) begin / advance / new_client / verify (Prüfcode = sha256(Seed)),
          (4) clean_state Feld für Feld, (5) clean_log / record (höchstens 12,
              nur JSON-Grundtypen), (6) round_rand = seedrand aus allen vier Werten,
Statistik (7) chipstats: record, Grenzen, rtp/win_rate, clean robust,
Mines     (8) Multiplikator exakt: m_j = max(floor(97*C(N,M)/C(N-j,M)), m_(j-1)+1),
              gedeckelt bei x100.000 (ab x1.01, steigt mit jedem Edelstein); comb
              exakt und in JS-Zahlen sicher, (9) Quote jeder Auszahl-Strategie 96-97 %
              (von der Untergrenze angehoben: höchstens 99 %, gedeckelt: höchstens
              97 %), (10) Chance/Anzeige/Leiter,
          (11) make_field: Monte-Carlo (jede Zelle gleich oft Mine), Form,
          (12) clean_round / clean_history / clean_marks robust, payout_carry,
Plinko    (13) 27 Tabellen: Länge, symmetrisch, nach außen nicht fallend, schöne
              Zahlen, Quote 96,5-97,5 %, Ränder x16/x110/x1000, Mitte; der
              Erzeuger devtools/gen_plinko_tables.py liefert genau diese Tabellen,
          (14) drop_path: genau rows Würfe, Verteilung = Binomial (Monte-Carlo),
          (15) prob / expected_return / Texte / clean_hits / clean_history,
              payout_carry (Rest-Hundertstel-Übertrag für Einsätze 1/2/5) / clean_carry,
Crash     (18) Platzpunkt: Formel vollständig am 2^16-Gegenstück, exakte Quote je Ziel,
          (19) Steigkurve (Landmarken, Umkehrung tick_at), (20) Takt-Regel gegen eine
              Referenz (400 Zufallsrunden, Grenzfälle, Mond, Hälfte, Übertrag),
          (21) Monte-Carlo mit echten Seeds, (22) clean_round/Zähler/Texte/Achsen/Welten,
Web       (16) Python = JS über web/tools/fairness_dump.js,
              lamamines_logic_dump.js, lamaplinko_logic_dump.js und
              lamacrash_logic_dump.js (node; Crash: ganze Kurve, Platzpunkte,
              ganze Runden mit Hand-Ausstiegen und Mond, clean_round, Zähler, Achsen),
Rust      (17) Rust-Quelltext hat dieselben Tabellen und Konstanten (Mines, Plinko,
              Lama Crash samt Musik und Lama-Bank); Prüfwerte chiplogic_ref.json und
              lamacrash_music_ref.txt aktuell; der Gleichstand der Rechnungen läuft
              als Rust-Unit-Test:
                  cd rust
                  cargo test -p pgz-core --offline --lib -- fairness chipstats
                  cargo test -p pgz-games --offline --lib -- chiplogic lamacrash_music

Aufruf aus dem Repo-Root:  python tests/audit_chiplogic.py
Exit-Code 0 = alle Prüfungen bestanden.
"""
import importlib.util
import json
import math
import os
import random
import re
import shutil
import subprocess
import sys
from fractions import Fraction

try:
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
except (AttributeError, OSError):
    pass

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, REPO)

import fairness as F  # noqa: E402
import chipstats as C  # noqa: E402
import seedrand  # noqa: E402


def _load(name, rel):
    """Modul direkt laden (games/__init__.py zöge alle Spiele samt pygame nach)."""
    spec = importlib.util.spec_from_file_location(name, os.path.join(REPO, *rel.split("/")))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


M = _load("lamamines_logic", "games/lamamines_logic.py")
P = _load("lamaplinko_logic", "games/lamaplinko_logic.py")
GEN = _load("gen_plinko_tables", "devtools/gen_plinko_tables.py")

FAILS = []


def check(ok, label, detail=""):
    print("  %-4s %s%s" % ("OK" if ok else "FAIL", label,
                           ("  -> " + str(detail)) if detail and not ok else ""))
    if not ok:
        FAILS.append(label)
    return ok


def js_dump(script, cases):
    """Dump-Skript mit Fällen (JSON über stdin) laufen lassen -> dict oder None."""
    out = subprocess.run([NODE, os.path.join(REPO, "web", "tools", script), "-"],
                         input=json.dumps(cases), capture_output=True, text=True, encoding="utf-8")
    try:
        return json.loads(out.stdout), out.stderr
    except ValueError:
        return None, out.stderr[-400:]


NODE = shutil.which("node")
R = random.Random(2026)

# ================================================================ Fairness
print("\n[Fairness]")
VECTORS = {
    "": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    "abc": "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq":
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
    "a" * 1000: "41edece42d63e8d9bf515a9ba6932e1c20cbc9f5a5d134645adb5db1b9737ea3",
}
check(all(F.sha256_hex(t) == h for t, h in VECTORS.items()), "(1) SHA-256-Prüfvektoren (leer, abc, 448 Bit, 1000 x a)")
BOUNDARY = ["x" * n for n in (55, 56, 63, 64)] + ["Lama äöü € \U0001f999"]
check(F.commit("abc") == VECTORS["abc"] and len(F.sha256_hex("x" * 64)) == 64, "(1) commit = sha256_hex, 64 Hex-Zeichen")

rng_a, rng_b = random.Random(42), random.Random(42)
st_a, st_b = F.blank_state(rng_a), F.blank_state(rng_b)
check(st_a == st_b and list(st_a) == ["server", "client", "nonce"] and st_a["nonce"] == 0,
      "(2) gleicher Startwert -> gleicher Zustand {server, client, nonce: 0}", st_a)
r42 = random.Random(42)
want_server = "%016x%016x" % (r42.getrandbits(64), r42.getrandbits(64))
want_client = "".join(F.CLIENT_ALPHABET[r42.randrange(32)] for _ in range(8))
check(st_a["server"] == want_server and st_a["client"] == want_client,
      "(2) Server = 2 x getrandbits(64), Client = 8 x randrange(32)")
check(st_a == {"server": "1c80317fa3b1799dbdd640fb06671ad1", "client": "trqigf5c", "nonce": 0},
      "(2) Prüfwert random.Random(42) (auch im Rust-Test)", st_a)
seeds = [F.new_server_seed(R) for _ in range(300)]
clients = [F.new_client_seed(R) for _ in range(300)]
check(all(F.valid_server(s) for s in seeds) and len(set(seeds)) == 300
      and all(F.valid_client(c) and len(c) == 8 for c in clients)
      and set("".join(clients)) == set(F.CLIENT_ALPHABET) and len(F.CLIENT_ALPHABET) == 32
      and not set("lo01") & set(F.CLIENT_ALPHABET),
      "(2) Seeds: 32 Hex / 8 Zeichen aus 32er-Alphabet (ohne l, o, 0, 1), alle Zeichen kommen vor")

st = F.blank_state(random.Random(5))
rng5 = random.Random(99)
e = F.begin(st)
old = dict(st)
F.advance(st, rng5)
check(list(e) == ["n", "s", "c", "h"] and e["n"] == 0 and e["s"] == old["server"] and e["c"] == old["client"]
      and e["h"] == F.sha256_hex(old["server"]), "(3) begin: {n, s, c, h} mit h = sha256(Server-Seed)")
check(st["nonce"] == 1 and st["server"] != old["server"] and st["client"] == old["client"]
      and F.valid_server(st["server"]), "(3) advance: Nonce + 1, neuer Server-Seed, Client bleibt")
c_old = st["client"]
F.new_client(st, rng5)
check(st["client"] != c_old and F.valid_client(st["client"]) and st["nonce"] == 1, "(3) new_client: neuer Client-Seed")
check(F.verify(e) and not F.verify(dict(e, h="0" * 64)) and not F.verify(dict(e, s="f" * 32))
      and not F.verify(None) and not F.verify({"s": e["s"]}), "(3) verify erkennt falsche Prüfcodes")

valid = {"server": "ab" * 16, "client": "kamaz", "nonce": 41}
check(F.clean_state(valid, random.Random(1)) == valid, "(4) clean_state: gültiger Zustand bleibt")
rs = random.Random(3)
bad = F.clean_state({"server": "AB" * 16, "client": "kaugummi", "nonce": True}, rs)
check(F.valid_server(bad["server"]) and bad["server"] != "ab" * 16 and bad["client"] == "kaugummi" and bad["nonce"] == 0,
      "(4) clean_state: Großbuchstaben/bool -> nur diese Felder neu")
bad2 = F.clean_state({"server": "ab" * 16, "client": "l0l", "nonce": -1}, rs)
check(bad2["server"] == "ab" * 16 and F.valid_client(bad2["client"]) and bad2["client"] != "l0l" and bad2["nonce"] == 0,
      "(4) clean_state: verbotene Zeichen / negative Nonce")
check(all(F.valid_server(F.clean_state(x, rs)["server"]) for x in (None, [], "x", 5, {"server": 7}))
      and F.clean_state({"server": "ab" * 16, "client": "a" * 17, "nonce": 2 ** 60}, rs)["nonce"] == 0
      and len(F.clean_state({"client": "a" * 17}, rs)["client"]) == 8,
      "(4) clean_state: kaputte Daten -> frisch, Client > 16 Zeichen, Nonce > 2^53")

log = []
for i in range(20):
    F.record(log, e, {"i": i})
check(len(log) == F.LOG == 12 and log[0]["o"] == {"i": 8} and list(log[0]) == ["n", "s", "c", "h", "o"],
      "(5) record: neueste zuletzt, höchstens 12, {n, s, c, h, o}")
check(F.make_entry(e, 1.5)["o"] is None and F.make_entry(e, [1, [2, [3]]])["o"] == [1, [2, [3]]]
      and F.make_entry(e, [[[[[1]]]]])["o"] is None and F.make_entry(e, 2 ** 60)["o"] is None
      and F.make_entry(e, {"a": None, "b": True, "c": "x"})["o"] == {"a": None, "b": True, "c": "x"},
      "(5) Ergebnis 'o' nur aus JSON-Grundtypen (keine floats, Tiefe <= 4, ints <= 2^53)")
raw_log = [dict(x) for x in log] + ["x", None, {"n": -1, "s": e["s"], "c": e["c"], "h": e["h"]},
                                    {"n": True, "s": e["s"], "c": e["c"], "h": e["h"]},
                                    {"n": 3, "s": e["s"][:-1], "c": e["c"], "h": e["h"]},
                                    {"n": 3, "s": e["s"], "c": e["c"], "h": "z" * 64},
                                    {"n": 3, "s": e["s"], "c": e["c"], "h": "0" * 64, "o": 0.5}]
cl = F.clean_log(raw_log)
check(len(cl) == 12 and cl[-1] == {"n": 3, "s": e["s"], "c": e["c"], "h": "0" * 64, "o": None}
      and not F.verify(cl[-1]) and F.verify(cl[0]) and F.clean_log("x") == [] and F.clean_log([]) == [],
      "(5) clean_log: ungültige Einträge fallen weg, falscher Prüfcode bleibt sichtbar (✗)")
check(F.short("abcdef" * 3) == "abcdefabcdef…" and F.short("abc") == "abc", "(5) short kürzt für die Anzeige")

rr = F.round_rand("lamamines", "ab" * 16, "kaugummi", 7)
check(isinstance(rr, seedrand.Rand) and rr.state == seedrand.seed_from("lamamines", "ab" * 16, "kaugummi", 7)
      and F.entry_rand("x", {"s": "ab" * 16, "c": "kaugummi", "n": 7}).state == seedrand.seed_from("x", "ab" * 16, "kaugummi", 7),
      "(6) round_rand = seedrand.Rand(seed_from(Spiel, Server, Client, Nonce))")
states = {F.round_rand(g, s, c, n).state for g in ("lamamines", "lamaplinko") for s in ("ab" * 16, "cd" * 16)
          for c in ("kaugummi", "zebra") for n in range(5)}
check(len(states) == 40, "(6) jeder der vier Werte ändert den Zufall")

# ================================================================ Statistik
print("\n[Statistik]")
s = C.blank()
check(s == {"rounds": 0, "wins": 0, "losses": 0, "wagered": 0, "paid": 0, "net": 0, "best_win": 0,
            "best_mult": 0, "curve": [], "x": {}} and C.rtp(s) is None and C.win_rate(s) is None,
      "(7) blank: alles 0, rtp/win_rate None")
C.record(s, 100, 250, 250)
C.record(s, 100, 0, 0)
C.record(s, 50, 50, 100)
check(s["rounds"] == 3 and s["wins"] == 1 and s["losses"] == 1 and s["wagered"] == 250 and s["paid"] == 300
      and s["net"] == 50 and s["best_win"] == 150 and s["best_mult"] == 250 and s["curve"] == [150, 50, 50]
      and C.rtp(s) == 300 / 250 and C.win_rate(s) == 1 / 3,
      "(7) record: Gewinn/Verlust/Gleichstand, Summen, Bestwerte, Kurve")
s2 = C.blank()
for i in range(250):
    C.record(s2, 10, 0, 0)
check(len(s2["curve"]) == C.CURVE == 100 and s2["curve"][-1] == -2500 and s2["curve"][0] == -1510
      and s2["best_win"] == 0 and s2["losses"] == 250, "(7) Kurve höchstens 100 Werte, best_win bleibt >= 0")
check(C.clean(s) == s and C.clean(json.loads(json.dumps(s2))) == s2, "(7) clean: gültige Statistik bleibt")
dirty = {"rounds": True, "wins": -3, "losses": "7", "wagered": 100, "paid": 40, "net": 999, "best_win": 2.5,
         "best_mult": 300, "curve": [1, True, "x", 2.5, -4] + [9] * 120, "x": {"hits": [1, 2], "bad": 1.5, "n": None}}
cd = C.clean(dirty)
check(cd["rounds"] == 0 and cd["wins"] == 0 and cd["losses"] == 0 and cd["net"] == -60 and cd["best_win"] == 0
      and cd["best_mult"] == 300 and len(cd["curve"]) == 100 and cd["curve"][-1] == 9
      and cd["x"] == {"hits": [1, 2], "n": None} and C.clean("x") == C.blank() and C.clean(None) == C.blank(),
      "(7) clean: bool/negativ/Text -> 0, net neu berechnet, Kurve gekürzt, x ohne floats")

# ================================================================ Mines
print("\n[Mines]")
check(M.GRIDS == (4, 5, 6, 7) and M.HOUSE == 97 and M.MAX_MULT100 == 10_000_000 and M.DEFAULT_MINES == 3
      and M.DEFAULT_GRID == 5 and M.BETS == (10, 20, 50, 100, 200, 500, 1000, 2500, 5000, 10000)
      and all(set(M.MINE_PRESETS[g]) <= set(range(1, M.max_mines(g) + 1)) and len(M.MINE_PRESETS[g]) == 6
              and 3 in M.MINE_PRESETS[g] for g in M.GRIDS),
      "(8) Konstanten: Felder 4-7, Quote 97, Deckel x100.000, Schnellwahl je Größe")
comb_bad = [(n, k) for n in range(50) for k in range(-1, n + 2) if M.comb(n, k) != (math.comb(n, k) if 0 <= k <= n else 0)]
inter_ok = all(min(k, n - k) * math.comb(n, k) < 2 ** 53 for n in range(50) for k in range(n + 1))
check(not comb_bad and inter_ok and math.comb(49, 24) * 97 < 2 ** 53,
      "(8) comb exakt, jeder Zwischenwert und 97*C(49,24) < 2^53", comb_bad[:3])
mult_bad, rtp_bad, capped, lifted = [], [], 0, []
lo_rtp, hi_rtp = Fraction(1), Fraction(0)
for g in M.GRIDS:
    n = g * g
    for m in range(1, n):
        prev = 100
        for k in range(1, n - m + 1):
            raw = math.floor(Fraction(97 * math.comb(n, m), math.comb(n - k, m)))
            step = max(raw, prev + 1) if prev < M.MAX_MULT100 else M.MAX_MULT100
            want = min(M.MAX_MULT100, step)
            got = M.mult100(g, m, k)
            if got != want or (got <= prev and got < M.MAX_MULT100) or got < 101 or M.raw_mult100(g, m, k) != raw:
                mult_bad.append((g, m, k, got, want))
            prev = got
            er = M.expected_return(g, m, k)
            exact = Fraction(math.comb(n - k, m), math.comb(n, m)) * Fraction(got, 100)
            if er != exact:
                rtp_bad.append((g, m, k, float(er)))
            if M.is_max(g, m, k):
                capped += 1
                if er > Fraction(97, 100):
                    rtp_bad.append((g, m, k, float(er)))
            elif got > raw:
                # von der Untergrenze m_(j-1) + 1 angehoben: Quote darf leicht über 97 % liegen
                lifted.append((float(er), g, m, k))
                if er > Fraction(99, 100):
                    rtp_bad.append((g, m, k, float(er)))
            else:
                lo_rtp, hi_rtp = min(lo_rtp, er), max(hi_rtp, er)
                if not Fraction(96, 100) <= er <= Fraction(97, 100):
                    rtp_bad.append((g, m, k, float(er)))
check(not mult_bad and M.mult100(5, 3, 0) == 100 and M.mult100(5, 3, -2) == 100
      and M.mult100(5, 3, 99) == M.mult100(5, 3, 22) and M.mult100(6, 1, 1) == 101 and M.mult100(7, 1, 1) == 101,
      "(8) m_j = max(97*C(N,M)//C(N-j,M), m_(j-1)+1), Deckel x100.000: ab x1.01, steigt mit jedem Edelstein",
      mult_bad[:3])
check(not rtp_bad and capped > 0, "(9) Quote jeder Strategie %.2f-%.2f %% (%d gedeckelte Stufen <= 97 %%)"
      % (float(lo_rtp) * 100, float(hi_rtp) * 100, capped), rtp_bad[:3])
top = max(lifted) if lifted else (0, 0, 0, 0)
check(len(lifted) == 3 and top[1:] == (7, 1, 1),
      "(9) Untergrenze hebt %d Stufen an (6x6/7x7 mit 1 Mine), höchste Quote %.2f %% (7x7, 1 Mine, 1. Edelstein) <= 99 %%"
      % (len(lifted), top[0] * 100), lifted)
check([M.mult100(5, 3, k) for k in range(8)] == [100, 110, 125, 144, 167, 195, 230, 273]
      and M.payout(100, 5, 3, 2) == 125 and M.payout(10, 4, 1, 1) == 10 and M.payout(10000, 7, 24, 25) == 1_000_000_000,
      "(9) Beispiel 5x5/3 Minen, Auszahlung abgerundet, Höchstgewinn 10.000 x 100.000")
pct_bad = []
for g in M.GRIDS:
    for m in range(1, g * g):
        for k in range(0, g * g - m + 1):
            num, den = M.safe_chance(g, m, k)
            want = math.floor(Fraction(num * 1000, den) + Fraction(1, 2))
            if (num, den) != (g * g - m - k, g * g - k) or M.safe_percent(g, m, k) != want:
                pct_bad.append((g, m, k))
all_bad = [(g, m, k) for g in M.GRIDS for m in range(1, g * g) for k in range(-1, g * g - m + 2)
           if M.all_safe_percent(g, m, k) != (1000 if k <= 0 else 0 if k > g * g - m else
                                              math.floor(Fraction(math.comb(g * g - k, m) * 1000, math.comb(g * g, m))
                                                         + Fraction(1, 2)))]
check(not all_bad and M.all_safe_percent(5, 3, 1) == 880 and M.all_safe_percent(7, 24, 25) == 0
      and M.all_safe_percent(4, 1, 15) == 63, "(10) all_safe_percent: k Felder alle sicher, Zehntel-Prozent", all_bad[:3])
check(not pct_bad and M.safe_percent(5, 3, 0) == 880 and M.fmt_percent(845, ",") == "84,5" and M.fmt_percent(5) == "0.5",
      "(10) Chance (N-M-k)/(N-k) in Zehntel-Prozent, kaufmännisch gerundet", pct_bad[:3])
check(M.next_mult100(5, 3, 0) == 110 and M.next_mult100(5, 3, 22) is None and M.is_clear(5, 3, 22)
      and not M.is_clear(5, 3, 21) and M.is_max(7, 24, 25) and not M.is_max(5, 3, 5)
      and M.ladder(5, 3, 0) == [(j, M.mult100(5, 3, j)) for j in range(1, 6)] and M.ladder(5, 3, 20) == [(21, M.mult100(5, 3, 21)), (22, M.mult100(5, 3, 22))]
      and M.fmt_mult(1142, ",") == "11,42" and M.fmt_mult(10_000_000) == "100000.00"
      and M.clamp_mines(4, 0) == 1 and M.clamp_mines(4, 20) == 15 and M.cells(7) == 49 and M.gems(7, 48) == 1,
      "(10) next/is_clear/is_max/ladder/fmt_mult/clamp")

rm = random.Random(77)
hits = {g: [0] * (g * g) for g in M.GRIDS}
shape_ok = True
trials = 4000
for g in M.GRIDS:
    for _ in range(trials):
        m = rm.randint(1, g * g - 1)
        field = M.make_field(seedrand.Rand(rm.getrandbits(32)), g, m)
        if field != sorted(set(field)) or len(field) != m or not all(0 <= c < g * g for c in field):
            shape_ok = False
    # Häufigkeit je Zelle bei fester Minenzahl 3: Chi-Quadrat gegen Gleichverteilung
for g in M.GRIDS:
    n = g * g
    for _ in range(trials):
        for c in M.field_from_seed(g, 3, F.new_server_seed(rm), "kaugummi", rm.randrange(10 ** 6)):
            hits[g][c] += 1
chi_bad = []
for g in M.GRIDS:
    n = g * g
    exp = trials * 3 / n
    chi = sum((h - exp) ** 2 / exp for h in hits[g])
    # Freiheitsgrade n-1, Grenze ~ df + 5*sqrt(2 df)
    if chi > (n - 1) + 5 * math.sqrt(2 * (n - 1)):
        chi_bad.append((g, round(chi, 1)))
check(shape_ok and not chi_bad, "(11) make_field: sortiert, eindeutig, M Minen; jede Zelle gleich oft Mine (Chi², 4 x 4000)",
      chi_bad)
f1 = M.make_field(seedrand.Rand(12345), 5, 3)
order = list(range(25))
seedrand.Rand(12345).shuffle(order)
check(f1 == sorted(order[:3]) and M.field_from_seed(5, 3, "ab" * 16, "kaugummi", 7)
      == M.make_field(F.round_rand("lamamines", "ab" * 16, "kaugummi", 7), 5, 3),
      "(11) Feld = erste M der gemischten Liste, aus dem Fairness-Zufall")

srv = "ab" * 16
fld = M.field_from_seed(5, 3, srv, "kaugummi", 7)
safe = [c for c in range(25) if c not in fld]
good = {"bet": 100, "grid": 5, "mines": 3, "n": 7, "s": srv, "c": "kaugummi", "h": F.commit(srv), "open": safe[:4]}
check(M.clean_round(good) == good and list(M.clean_round(good)) == ["bet", "grid", "mines", "n", "s", "c", "h", "open"]
      and M.clean_round(dict(good, open=[])) == dict(good, open=[]),
      "(12) clean_round: gültige Runde bleibt (auch ohne aufgedecktes Feld)")
bads = [dict(good, open=safe[:3] + [fld[0]]), dict(good, open=[safe[0], safe[0]]), dict(good, open=safe[:22]),
        dict(good, open=[25]), dict(good, open=[True]), dict(good, open="x"), dict(good, bet=0), dict(good, bet=True),
        dict(good, grid=8), dict(good, mines=0), dict(good, mines=25), dict(good, h="0" * 64), dict(good, s="AB" * 16),
        dict(good, c=""), dict(good, n=-1), dict(good, n=1.0), "kaputt", None, {}]
check(all(M.clean_round(b) is None for b in bads) and M.clean_round(dict(good, open=safe[:21])) is not None,
      "(12) clean_round: Mine aufgedeckt, doppelt, schon fertig, falsche Werte -> None",
      [i for i, b in enumerate(bads) if M.clean_round(b) is not None])
check(M.clean_history([0, 154, -3, "x", True, 999, 10 ** 8] + [100] * 20) == [100] * M.HISTORY
      and M.clean_history(None) == [] and M.clean_history([0, 154]) == [0, 154],
      "(12) clean_history: nur gültige Werte, höchstens 12")
marks = M.clean_marks({"4": [3, 1, 3, True, 16, -1, "2", 0], "5": "x", "7": list(range(60)), "9": [1]})
check(marks == {"4": [0, 1, 3], "5": [], "6": [], "7": list(range(48))} and M.clean_marks(None) == {"4": [], "5": [], "6": [], "7": []},
      "(12) clean_marks: alle vier Größen, eindeutig, sortiert, höchstens N-1")
rk = random.Random(9)
MCARRY = []
for _ in range(2000):
    g = rk.choice(M.GRIDS)
    m = rk.randint(1, g * g - 1)
    MCARRY.append([rk.choice(M.BETS[:3]), g, m, rk.randint(1, g * g - m)])
carry, won, want100, carry_ok = 0, 0, 0, True
for b, g, m, k in MCARRY:
    w, carry = M.payout_carry(b, g, m, k, carry)
    won += w
    want100 += b * M.mult100(g, m, k)
    carry_ok = carry_ok and 0 <= carry <= 99
check(carry_ok and won * 100 + carry == want100 and M.payout_carry(10, 6, 1, 4, 0) == (10, 90)
      and M.payout_carry(10, 6, 1, 4, 90) == (11, 80) and M.clean_carry(99) == 99 and M.clean_carry(True) == 0
      and M.clean_carry(100) == 0, "(12) payout_carry/clean_carry: Einsatz 10 bei x1.09 verliert nichts mehr")

# ================================================================ Plinko
print("\n[Plinko]")
check(P.ROWS == tuple(range(8, 17)) and P.RISKS == ("low", "medium", "high") and P.MAX_BALLS == 50
      and P.BETS == (1, 2, 5, 10, 20, 50, 100, 200, 500, 1000) and set(P.TABLES) == set(P.RISKS)
      and all(set(P.TABLES[r]) == set(P.ROWS) for r in P.RISKS), "(13) 3 Risiken x 9 Reihenzahlen = 27 Tabellen")


def nice(v):
    if v < 1000:
        return v % 10 == 0 and v >= 10
    if v < 10000:
        return v % 100 == 0
    return v % 1000 == 0


tab_bad, quotes = [], {}
for r in P.RISKS:
    for n in P.ROWS:
        t = P.TABLES[r][n]
        half = n // 2
        right = t[half:]
        q = P.expected_return(n, r)
        quotes[(r, n)] = q
        q2 = sum(Fraction(math.comb(n, i), 2 ** n) * Fraction(m, 100) for i, m in enumerate(t))
        if (len(t) != n + 1 or any(t[i] != t[n - i] for i in range(n + 1))
                or any(a > b for a, b in zip(right, right[1:])) or not all(isinstance(v, int) and nice(v) for v in t)
                or not Fraction(965, 1000) <= q <= Fraction(975, 1000) or q != q2):
            tab_bad.append((r, n, float(q)))
check(not tab_bad, "(13) symmetrisch, nach außen nicht fallend, schöne Zahlen, Quote %.2f-%.2f %%"
      % (min(map(float, quotes.values())) * 100, max(map(float, quotes.values())) * 100), tab_bad[:3])
check(P.TABLES["low"][16][0] == 1600 and P.TABLES["medium"][16][0] == 11000 and P.TABLES["high"][16][0] == 100000
      and P.TABLES["high"][16][8] == 20 and all(P.TABLES["high"][n][n // 2] == 20 for n in P.ROWS)
      and all(50 <= P.TABLES["low"][n][n // 2] < 100 for n in P.ROWS)
      and all(any(60 <= v < 100 for v in P.TABLES["low"][n]) and any(100 <= v <= 150 for v in P.TABLES["low"][n]) for n in P.ROWS),
      "(13) Ränder bei 16 Reihen x16/x110/x1000, Mitte: Hoch x0.2, Niedrig knapp unter/über 1")
check(all(P.TABLES[r][n][0] < P.TABLES[r][n + 1][0] for r in P.RISKS for n in P.ROWS[:-1])
      and all(P.TABLES["low"][n][0] < P.TABLES["medium"][n][0] < P.TABLES["high"][n][0] for n in P.ROWS),
      "(13) Rand wächst mit den Reihen und mit dem Risiko")
gen_same = all(GEN.build(r, n)[0] == P.TABLES[r][n] for r in P.RISKS for n in P.ROWS)
check(gen_same, "(13) devtools/gen_plinko_tables.py erzeugt genau diese Tabellen")

rp = random.Random(31)
counts = {n: [0] * (n + 1) for n in (8, 12, 16)}
calls_ok = True
balls = 20000
for n in counts:
    for _ in range(balls):
        rnd = F.round_rand(P.KEY, F.new_server_seed(rp), "kaugummi", rp.randrange(10 ** 9))
        st0 = rnd.state
        path = P.drop_path(rnd, n)
        if len(path) != n or set(path) - {0, 1}:
            calls_ok = False
        # genau n random()-Aufrufe: Zustand = Startwert + n * 0x6D2B79F5
        if rnd.state != (st0 + n * 0x6D2B79F5) & 0xFFFFFFFF:
            calls_ok = False
        counts[n][P.slot_of(path)] += 1
mc_bad = []
for n, cnt in counts.items():
    chi = 0.0
    df = 0
    for i, h in enumerate(cnt):
        exp = balls * math.comb(n, i) / 2 ** n
        if exp >= 5:
            chi += (h - exp) ** 2 / exp
            df += 1
    if chi > (df - 1) + 5 * math.sqrt(2 * (df - 1)):
        mc_bad.append((n, round(chi, 1)))
check(calls_ok and not mc_bad, "(14) drop_path: genau rows Würfe 0/1, Fächer binomialverteilt (Chi², 3 x 20000)", mc_bad)
check(P.path_from_seed(12, "ab" * 16, "kaugummi", 3) == P.drop_path(F.round_rand("lamaplinko", "ab" * 16, "kaugummi", 3), 12)
      and P.slot_of([1, 0, 1, 1]) == 3 and P.is_edge(16, 0) and P.is_edge(16, 16) and not P.is_edge(16, 15),
      "(14) Pfad aus dem Fairness-Zufall, Fach = Zahl der Rechts-Sprünge")

check(all(sum(P.prob(n, i) for i in range(n + 1)) == 1 for n in P.ROWS) and P.prob(16, 0) == Fraction(1, 65536)
      and P.payout(7, 16, "high", 0) == 7000 and P.payout(5, 8, "high", 4) == 1 and P.payout(1, 16, "low", 8) == 0,
      "(15) prob summiert zu 1, Auszahlung abgerundet")
fp_bad = []
for n in P.ROWS:
    for i in range(n + 1):
        p = Fraction(math.comb(n, i) * 100, 2 ** n)
        txt = P.fmt_prob(n, i)
        digits = 2 if p >= 1 else 4
        want = math.floor(p * 10 ** digits + Fraction(1, 2))
        if txt != "%d.%0*d" % (want // 10 ** digits, digits, want % 10 ** digits):
            fp_bad.append((n, i, txt))
check(not fp_bad and P.fmt_prob(16, 0) == "0.0015" and P.fmt_prob(16, 8, ",") == "19,64",
      "(15) fmt_prob: ab 1 %% zwei, darunter vier Nachkommastellen", fp_bad[:3])
check(P.fmt_short(20) == "0.2" and P.fmt_short(1100) == "11" and P.fmt_short(100000) == "1000"
      and P.fmt_short(125, ",") == "1,25" and P.fmt_mult(20) == "0.20", "(15) fmt_short / fmt_mult")
hh = P.clean_hits({"8": [1] * 9, "9": [True] * 10, "10": [1, 2], "16": [-1, "x", 2.0] + [5] * 14, "x": 1})
check(list(hh) == [str(n) for n in P.ROWS] and hh["8"] == [1] * 9 and hh["9"] == [0] * 10 and hh["10"] == [0] * 11
      and hh["16"] == [0, 0, 0] + [5] * 14 and P.clean_hits("x") == P.blank_hits()
      and P.add_hit(P.blank_hits(), 12, 6)["12"][6] == 1,
      "(15) clean_hits: alle neun Reihenzahlen, falsche Länge/Werte -> 0")
check(P.clean_history([20, -1, True, "x", 100000] + [70] * 15) == [70] * 12 and P.clean_history(5) == [],
      "(15) clean_history")
rc = random.Random(8)
CARRY_SEQ = [[rc.choice((1, 2, 5, 7, 10)), n, rc.choice(P.RISKS), 0] for n in [rc.choice(P.ROWS) for _ in range(3000)]]
for b in CARRY_SEQ:
    b[3] = P.slot_of(P.drop_path(rc, b[1]))
carry, won, want100 = 0, 0, 0
carry_ok = True
for b, n, rk, sl in CARRY_SEQ:
    w, carry = P.payout_carry(b, n, rk, sl, carry)
    won += w
    want100 += b * P.mult100(n, rk, sl)
    if not 0 <= carry <= 99:
        carry_ok = False
check(carry_ok and won * 100 + carry == want100 and P.payout_carry(1, 12, "medium", 6, 0) == (0, 40)
      and P.payout_carry(1, 12, "medium", 6, 70) == (1, 10) and P.payout_carry(7, 16, "high", 0, 99) == (7000, 99),
      "(15) payout_carry: Rest-Hundertstel-Übertrag, über 3000 Kugeln kein Hundertstel verloren")
q_low = {b: min(float(sum(P.prob(n, i) * Fraction(b * m // 100, b) for i, m in enumerate(P.TABLES[r][n])))
                for r in P.RISKS for n in P.ROWS) for b in (1, 2, 5)}
check(P.clean_carry(42) == 42 and P.clean_carry(100) == 0 and P.clean_carry(-1) == 0 and P.clean_carry(True) == 0
      and P.clean_carry("5") == 0 and P.clean_carry(None) == 0 and q_low[1] < 0.5,
      "(15) clean_carry 0..99 (ohne Übertrag hätte Einsatz 1 nur %.0f %% Quote)" % (q_low[1] * 100))

# ================================================================ Crash
print("\n[Crash]")
X = _load("lamacrash_logic", "games/lamacrash_logic.py")
TWO32 = 2 ** 32

# (18) Platzpunkt: geschlossene Formel, vollständig am 2^16-Gegenstück, exakte Quote
check(X.crash_from_u32(0) == 100 and X.crash_from_u32(TWO32 - 1) == X.MAX_MULT100
      and X.crash_from_u32(TWO32 // 2) == 194, "(18) Platzpunkt: r=0 -> x1,00, r=max -> x100.000, Mitte x1,94")
check(X.HOUSE * TWO32 < 2 ** 53 and X.HOUSE * TWO32 * 2000 < 2 ** 53 and X.BETS[-1] * X.MAX_MULT100 < 2 ** 53,
      "(18) alle Zwischenwerte < 2^53 (JS-Zahlen exakt)")


def _crash16(r):
    """Dieselbe Formel mit 2^16 statt 2^32 (vollständig auszählbar)."""
    return min(X.MAX_MULT100, max(100, (X.HOUSE * 65536) // (65536 - r)))


cnt = {}
for r16 in range(65536):
    c16 = _crash16(r16)
    cnt[c16] = cnt.get(c16, 0) + 1
keys16 = sorted(cnt)
ge, acc = {}, 0
for v in reversed(keys16):
    acc += cnt[v]
    ge[v] = acc
bad16 = []
for T in list(range(101, 3000)) + [5000, 10000, 65000, 100000, 1000000, X.MAX_MULT100]:
    have = 0
    for v in keys16:
        if v >= T:
            have = ge[v]
            break
    if have != (X.HOUSE * 65536) // T:
        bad16.append(T)
check(not bad16, "(18) 2^16-Gegenstück: #(crash >= T) = 97*2^16 // T für alle geprüften T", bad16[:5])
rtps = [X.expected_return(T) for T in X.TARGETS] + [X.expected_return(R.randint(101, X.MAX_MULT100)) for _ in range(3000)]
check(all(Fraction(96997, 100000) <= v <= Fraction(97, 100) for v in rtps),
      "(18) Quote jedes Ziels 96,997-97,000 % (alle TARGETS + 3000 Zufallsziele, exakt)",
      (float(min(rtps)), float(max(rtps))))
check(abs(1 - X.win_count(101) / TWO32 - 0.0396) < 0.0001, "(18) rund 3,96 % platzen sofort (vor x1,01)")
check(X.fmt_chance(200) == "48.50" and X.fmt_chance(200, ",") == "48,50" and X.fmt_chance(10000) == "0.9700"
      and X.fmt_chance(X.MAX_MULT100) == "0.0010" and X.fmt_chance(101) == "96.04", "(18) fmt_chance")

# (19) Steigkurve
tab = X._table()
check(len(tab) == X.MOON_TICK + 1 and tab[0] == 100 and all(tab[i] <= tab[i + 1] for i in range(len(tab) - 1)),
      "(19) Kurve: 11521 Werte, Start x1,00, nie fallend")
check(X.mult_at(X.MOON_TICK - 1) < X.MAX_MULT100 <= X.mu_at(X.MOON_TICK) // X.MU_UNIT
      and X.mu_at(X.MOON_TICK) < 2 ** 53 and X.mult_at(10 ** 9) == X.MAX_MULT100 and X.mult_at(-5) == 100,
      "(19) MOON_TICK = erster Takt mit x100.000, mu < 2^53, Begrenzung")
check([X.tick_at(v) for v in (101, 200, 1000, 10000, 100000, X.MAX_MULT100)] == [10, 694, 2305, 4608, 6912, 11520]
      and X.world_ticks() == (0, 406, 1100, 2305, 4608, 11520), "(19) Landmarken: x2 bei 694, x10 bei 2305 … Welten")
bad = [m for m in [R.randint(100, X.MAX_MULT100) for _ in range(2000)]
       if not (X.mult_at(X.tick_at(m)) >= m and (X.tick_at(m) == 0 or X.mult_at(X.tick_at(m) - 1) < m))]
check(not bad, "(19) tick_at ist die Umkehrung von mult_at (2000 Zufallswerte)", bad[:3])
mu, ok_mu = X.MU0, True
for tk in range(1, 3000):
    mu += mu // X.GROWTH
    ok_mu = ok_mu and X.mu_at(tk) == mu
check(ok_mu, "(19) Kurve = schrittweise mu += mu // 1000")


# (20) Takt-Regel gegen eine einfache Referenz
def _ref(pairs, crash, acts):
    """Erwartetes Ergebnis ohne L.tick: Teil gewinnt bei Ziel a gdw. a <= crash; Hand-Ausstieg bei
    Takt t mit m = mult_at(t) nur, wenn m >= MIN_CASH, noch nicht geplatzt und das Auto-Ziel noch
    nicht erreicht ist (je Einsatz höchstens eine ganze und eine halbe Aktion)."""
    end_t = X.pop_tick(crash)
    out = []
    for i, (b, a) in enumerate(pairs):
        half_t = next((tk for tk, j, h in acts if j == i and h), None)
        full_t = next((tk for tk, j, h in acts if j == i and not h), None)
        auto_t = X.tick_at(a) if (a and a <= crash) else None

        def ok(tk):
            return (tk is not None and X.mult_at(tk) >= X.MIN_CASH and tk < end_t
                    and (auto_t is None or tk < auto_t))
        hm = xm = 0
        if ok(full_t):
            xm = X.mult_at(full_t)
        if ok(half_t) and (not xm or half_t < full_t):
            hm = X.mult_at(half_t)
        if not xm and auto_t is not None:
            xm = a
        if not xm and crash >= X.MAX_MULT100:
            xm = X.MAX_MULT100
        out.append((hm, xm))
    return out


def _sim(pairs, crash, acts):
    rnd = X.new_round({"n": 1, "s": "0" * 32, "c": "abc", "h": "0" * 64}, pairs)
    carry, end, todo = 0, None, sorted(acts)
    while end is None:
        while todo and todo[0][0] == rnd["t"]:
            tk, i, h = todo.pop(0)
            res = X.cash_out(rnd, i, h, carry)
            if res is not None:
                carry = res[2]
        _, carry, end = X.tick(rnd, crash, carry)
        if rnd["t"] > X.MOON_TICK + 2:
            break
    return rnd, end, carry


bad = []
for case in range(400):
    pairs = [(R.choice(X.BETS), R.choice((0,) + X.TARGETS[:30])) for _ in range(R.randint(1, 2))]
    crash = R.choice((100, 101, 102, 150, 200, 207, 350, 1000, 2500)) if case % 3 else X.crash_from_u32(R.getrandbits(32))
    crash = min(crash, 20000)
    acts = []
    for i in range(len(pairs)):
        if R.random() < 0.6:
            acts.append((R.randint(0, X.pop_tick(crash) + 5), i, False))
        if R.random() < 0.5:
            acts.append((R.randint(0, X.pop_tick(crash) + 5), i, True))
    rnd, end, carry = _sim(pairs, crash, acts)
    got = [(b["hm"], b["xm"]) for b in rnd["bets"]]
    want = _ref(pairs, crash, acts)
    if got != want or end != "pop" or rnd["t"] != X.pop_tick(crash) or X.mult_at(rnd["t"] - 1) > crash:
        bad.append((pairs, crash, acts, got, want, end))
check(not bad, "(20) Takt-Regel: 400 Zufallsrunden = Referenz (Auto bei a <= Platzpunkt, Platzen erst bei m > Platzpunkt)",
      bad[:1])
# Grenzfälle
r1, e1, _ = _sim([(100, 350)], 350, [])
r2, e2, _ = _sim([(100, 351)], 350, [])
check(r1["bets"][0]["xm"] == 350 and r1["bets"][0]["p"] == 350 and r2["bets"][0]["xm"] == 0 and e1 == e2 == "pop",
      "(20) Ziel = Platzpunkt gewinnt, Ziel = Platzpunkt + 0,01 verliert")
r3, e3, _ = _sim([(10, 0)], 100, [(0, 0, False), (5, 0, False), (9, 0, True)])
check(r3["bets"][0]["xm"] == 0 and r3["bets"][0]["hm"] == 0 and e3 == "pop" and r3["t"] == 10,
      "(20) Sofort-Platzer: unter x1,01 kein Ausstieg, Platzen bei Takt 10")
r4, e4, c4 = _sim([(10, 0), (20, 200)], X.MAX_MULT100, [(5000, 0, True)])
check(e4 == "moon" and r4["t"] == X.MOON_TICK and r4["bets"][0]["xm"] == X.MAX_MULT100 and r4["bets"][1]["xm"] == 200
      and r4["bets"][0]["hm"] == X.mult_at(5000), "(20) Mondlandung: Rest bekommt x100.000, Auto-Ziel vorher")
rnd = X.new_round({"n": 1, "s": "0" * 32, "c": "abc", "h": "0" * 64}, [(10, 0), (50, 0)])
rnd["t"] = 694
w1 = X.cash_out(rnd, 1, True, 0)
w2 = X.cash_out(rnd, 1, True, w1[2])
w3 = X.cash_out(rnd, 0, False, w1[2])
w4 = X.cash_out(rnd, 1, False, w3[2])
check(w1 == (25 * 200 // 100, 200, 0) and w2 is None and w3 == (20, 200, 0) and w4 == (50, 200, 0)
      and rnd["bets"][1]["p"] == 100 and rnd["bets"][1]["man"] == 3 and not X.flying(rnd),
      "(20) Hälfte einmal, Rest danach, man-Bits, flying", (w1, w2, w3, w4))
cr = 0
seq = []
for b, m in ((10, 101), (10, 137), (5, 199), (25, 101)):
    w, cr = X.payout_carry(b, m, cr)
    seq.append((w, cr))
check(seq == [(10, 10), (13, 80), (10, 75), (26, 0)], "(20) Rest-Übertrag über Teile und Runden", seq)

# (21) Monte-Carlo mit echten Seeds
N = 120000
srv, cli = F.new_server_seed(R), F.new_client_seed(R)
cr_list = [X.crash_point(srv, cli, n) for n in range(N)]
for T in (101, 150, 200, 1000, 10000):
    p = X.win_count(T) / TWO32
    have = sum(1 for v in cr_list if v >= T) / N
    sig = math.sqrt(p * (1 - p) / N)
    check(abs(have - p) <= 4 * sig, "(21) Monte-Carlo %d Runden: P(>= x%s) = %.4f (erwartet %.4f, 4 sigma)"
          % (N, X.fmt_mult(T, ","), have, p))
check(sum(1 for v in cr_list if v == 100) > 0 and max(cr_list) > 10000, "(21) Spannweite: Sofort-Platzer und große Flüge")

# (22) Prüfen/Laden, Zähler, Texte, Kurve
n7 = 7
while X.crash_point(srv, cli, n7) < 1000:          # Runde, die sicher über x10 fliegt
    n7 += 1
ent = F.begin({"server": srv, "client": cli, "nonce": n7})
crash7 = X.crash_point(srv, cli, n7)
good = X.new_round(ent, [(100, 0), (20, 150)])
check(X.clean_round(json.loads(json.dumps(good))) == good, "(22) clean_round: frische Runde bleibt gleich")
bad_rounds = [None, 5, dict(good, h="0" * 64), dict(good, t=-1), dict(good, t=X.MOON_TICK + 1),
              dict(good, bets=[]), dict(good, bets=[{"b": 7, "a": 0, "hm": 0, "xm": 0, "p": 0, "man": 0}]),
              dict(good, bets=[{"b": 10, "a": 100, "hm": 0, "xm": 0, "p": 0, "man": 0}]),
              dict(good, bets=[{"b": 10, "a": 0, "hm": 0, "xm": 0, "p": 0, "man": 2}]),
              dict(good, bets=[{"b": 10, "a": 0, "hm": 0, "xm": 0, "p": 0, "man": True}]),
              dict(good, t=X.pop_tick(crash7)),
              dict(good, bets=[{"b": 10, "a": 0, "hm": 0, "xm": 500000, "p": 0, "man": 2}])]
check(all(X.clean_round(b) is None for b in bad_rounds), "(22) clean_round verwirft kaputte/abgelaufene Runden",
      [i for i, b in enumerate(bad_rounds) if X.clean_round(b) is not None])
done = dict(good, t=10, bets=[{"b": 100, "a": 0, "hm": 0, "xm": 101, "p": 101, "man": 2}])
check(X.clean_round(done) == done, "(22) Runde ohne fliegendes Geld ist gültig (wird beim Laden abgeschlossen)")
x = X.blank_x()
rnd = X.new_round(ent, [(100, 0), (20, 150)])
rnd["bets"][0].update(hm=300, xm=320, p=310, man=3)
rnd["bets"][1].update(xm=150, p=30)
X.record_x(x, rnd, 330, "pop")
check(x["hits"][X.bucket_of(330)] == 1 and x["cash"] == 3 and x["half"] == 1 and x["auto"] == 1 and x["close"] == 1
      and x["top"] == 330 and x["inst"] == 0 and X.clean_x(json.loads(json.dumps(x))) == x
      and X.clean_x({"hits": [1, 2], "cash": -1, "top": True}) == X.blank_x(), "(22) Zähler: record_x / clean_x")
check(X.close_call(rnd, 330) and not X.close_call(rnd, 400) and X.best_cash(rnd) == 320,
      "(22) Haarscharf nur von Hand ab x2 und >= 95 % des Platzpunkts")
o = X.outcome(rnd, 330)
check(X.check_outcome(o, 330) and not X.check_outcome(o, 329) and not X.check_outcome(dict(o, b=[[1, 2, 3]]), 330)
      and F._plain(o), "(22) Fairness-Ergebnis: check_outcome, nur JSON-Grundtypen")
check(sum(X.bucket_prob(i) for i in range(len(X.BUCKETS))) == 1 and all(X.bucket_prob(i) > 0 for i in range(len(X.BUCKETS))),
      "(22) Verteilung der Fächer summiert sich exakt zu 1")
check(X.clean_history([50, 100, True, "x", 350, X.MAX_MULT100 + 1] + [200] * 20) == [200] * 12
      and X.clean_carry(99) == 99 and X.clean_carry(100) == 0, "(22) clean_history / clean_carry")
check([X.step_target(200, 1), X.step_target(200, -1), X.step_target(101, -1), X.step_target(X.MAX_MULT100, 1),
       X.step_target(201, -1)] == [225, 175, 101, X.MAX_MULT100, 200], "(22) step_target auf der Leiter")
check([X.fmt_short(v, ",") for v in (101, 999, 1234, 12345, 123456, X.MAX_MULT100)]
      == ["1,01", "9,99", "12,3", "123", "1,2k", "100k"] and X.fmt_mult(235, ",") == "2,35", "(22) fmt_short / fmt_mult")
check(X.chart_span(0) == (480, 200) and X.chart_ylines(200) == [125, 150, 175, 200] and X.chart_xlines(480) == [120, 240, 360, 480]
      and len(X.chart_ylines(X.MAX_MULT100)) <= 4 and len(X.chart_xlines(20000)) <= 5, "(22) Kurven-Achsen")
check([X.world_blend(v) for v in (0, 300, 380, 406, 1100, 4700)] == [(0, 0), (0, 0), (0, 181), (1, 0), (2, 0), (4, 0)]
      and [X.world_of(v) for v in (100, 149, 150, 299, 300, 999, 1000, 9999, 10000, X.MAX_MULT100)]
      == [0, 0, 1, 1, 2, 2, 3, 3, 4, 5], "(22) Höhenwelten: world_of / world_blend")

# ================================================================ Web
print("\n[Web]")
if NODE:
    st_cases = [F.blank_state(random.Random(i)) for i in range(6)]
    for x in st_cases:
        x["nonce"] = R.randrange(10 ** 9)
    ent = [F.begin(x) for x in st_cases]
    fcases = {
        "sha": list(VECTORS) + BOUNDARY + ["".join(chr(R.randrange(32, 127)) for _ in range(R.randrange(200)))
                                          for _ in range(40)],
        "begin": st_cases,
        "clean_state": st_cases + [valid],
        "logs": [raw_log, log, "x", [None]],
        "verify": ent + [dict(ent[0], h="0" * 64), None, {"s": 5}],
        "rand": [[g, x["server"], x["client"], x["nonce"], 6] for g in ("lamamines", "lamaplinko") for x in st_cases],
        "entries": [[ent[0], {"m": [1, 2], "x": 5}], [ent[1], 1.5], [ent[2], None], [ent[3], [[[[[1]]]]]]],
        "stats_clean": [s, s2, dirty, "x", None, {}],
        "stats_record": [[[100, 250, 250], [100, 0, 0], [50, 50, 100]], [[10, 0, 0]] * 250,
                         [[R.choice(M.BETS), R.randrange(0, 20000), R.randrange(0, 10 ** 6)] for _ in range(130)]],
    }
    js, err = js_dump("fairness_dump.js", fcases)
    if check(js is not None, "(16) Web-Fairness lässt sich laden", err):
        check(js["alphabet"] == F.CLIENT_ALPHABET and js["client_len"] == F.CLIENT_LEN and js["client_max"] == F.CLIENT_MAX
              and js["log"] == F.LOG and js["max_nonce"] == F.MAX_NONCE and js["curve"] == C.CURVE,
              "(16) Web: gleiche Fairness-Konstanten")
        check(js["vectors"] == [F.sha256_hex(t) for t in list(VECTORS) + ["x" * 55, "x" * 56, "x" * 63, "x" * 64, BOUNDARY[-1]]]
              and js["sha"] == [F.sha256_hex(t) for t in fcases["sha"]],
              "(16) Web: SHA-256 gleich (%d Texte inkl. Blockgrenzen und Umlaute/Emoji)" % (len(fcases["sha"]) + 9))
        sh = js["shape"]
        check(sh["server_ok"] and sh["client_ok"] and sh["nonce"] == 1 and sh["entry_ok"] and sh["keys"] == ["n", "s", "c", "h"]
              and sh["state_keys"] == ["server", "client", "nonce"] and sh["clean_bad_nonce"] == 0
              and sh["record_len"] == [12, 8] and sh["short"][1] == "abc" and sh["random_ok"],
              "(16) Web: Seeds (crypto) gültig, begin/advance/record wie Python")
        check(js["begin"] == [F.begin(x) for x in st_cases] and js["clean_state"] == fcases["clean_state"]
              and js["logs"] == [F.clean_log(x) for x in fcases["logs"]] and js["verify"] == [F.verify(x) for x in fcases["verify"]]
              and js["entries"] == [F.make_entry(a, b) for a, b in fcases["entries"]],
              "(16) Web: begin, clean_state, clean_log, verify, make_entry gleich")
        want_rand = []
        for g, sv, cv, nv, k in fcases["rand"]:
            rr = F.round_rand(g, sv, cv, nv)
            want_rand.append([rr.next_u32() for _ in range(k)])
        check(js["rand"] == want_rand, "(16) Web: Rundenzufall (round_rand) bitgleich")
        want_rec = []
        for ops in fcases["stats_record"]:
            x = C.blank()
            for w, p, m in ops:
                C.record(x, w, p, m)
            want_rec.append({"st": x, "rtp": C.rtp(x), "win_rate": C.win_rate(x)})
        check(js["stats_clean"] == [C.clean(x) for x in fcases["stats_clean"]] and js["stats_record"] == want_rec,
              "(16) Web: chipstats clean/record/rtp gleich")

    sv = [F.new_server_seed(R) for _ in range(40)]
    # Mines-Runde aus (12) - "good" ist seit dem Crash-Abschnitt (22) dessen Runde
    m_good = {"bet": 100, "grid": 5, "mines": 3, "n": 7, "s": "ab" * 16, "c": "kaugummi", "h": F.commit("ab" * 16),
              "open": safe[:4]}
    mcases = {
        "fields": [[g, R.randint(1, g * g - 1), sv[i], F.new_client_seed(R), R.randrange(10 ** 6)]
                   for i, g in enumerate(list(M.GRIDS) * 10)],
        "make": [[R.getrandbits(32), g, R.randint(1, g * g - 1)] for g in list(M.GRIDS) * 10],
        "rounds": [m_good, dict(m_good, open=[])] + bads[:16] + [dict(m_good, open=safe[:21])],
        "history": [[0, 154, -3, "x", True, 999] + [100] * 20, None, [0, 154]],
        "marks": [{"4": [3, 1, 3, True, 16, -1, "2", 0], "5": "x", "7": list(range(60)), "9": [1]}, None, {"6": [35, 0]}],
        "carry": MCARRY[:300],
    }
    js, err = js_dump("lamamines_logic_dump.js", mcases)
    if check(js is not None, "(16) Web-Mines lässt sich laden", err):
        check(js["key"] == M.KEY and js["grids"] == list(M.GRIDS) and js["bets"] == list(M.BETS) and js["house"] == M.HOUSE
              and js["max_mult100"] == M.MAX_MULT100 and js["history"] == M.HISTORY
              and js["presets"] == {str(g): list(v) for g, v in M.MINE_PRESETS.items()}
              and js["default_grid"] == M.DEFAULT_GRID and js["default_mines"] == M.DEFAULT_MINES
              and js["brave"] == M.BRAVE_MULT and js["legend"] == M.LEGEND_MULT, "(16) Web: gleiche Mines-Konstanten")
        same = exp_ok = True
        for g in M.GRIDS:
            for m in range(1, g * g):
                key = "%d:%d" % (g, m)
                ks = range(0, g * g - m + 2)
                if (js["mult"][key] != [M.mult100(g, m, k) for k in ks]
                        or js["safe"][key] != [M.safe_percent(g, m, k) for k in ks]
                        or js["allsafe"][key] != [M.all_safe_percent(g, m, k) for k in ks]
                        or js["next"][key] != [M.next_mult100(g, m, k) for k in ks]
                        or js["payout"][key] != [[M.payout(b, g, m, k) for b in (10, 55, 10000)] for k in ks]
                        or js["ladder"][key] != [[list(x) for x in M.ladder(g, m, k)] for k in (0, 1, g * g - m - 2, g * g - m)]):
                    same = False
                if any(abs(a - float(M.expected_return(g, m, k))) > 1e-12 for a, k in zip(js["expected"][key], ks)):
                    exp_ok = False
        check(same, "(16) Web: alle Multiplikatoren, Chancen, Auszahlungen, Leitern gleich (1.260 Feld/Minen-Paare)")
        check(exp_ok, "(16) Web: Erwartungswerte gleich (Gleitkomma, 1e-12)")
        check(js["fmt"] == [M.fmt_mult(v, s_) for v in (100, 105, 1142, 16042, 10_000_000) for s_ in (".", ",")]
              and js["fmtpct"] == [M.fmt_percent(v, s_) for v in (0, 5, 845, 880, 1000) for s_ in (".", ",")]
              and js["combs"] == [M.comb(n, k) for n in range(50) for k in range(-1, n + 2)]
              and js["clamp"] == [M.clamp_mines(g, m) for g, m in [(4, 0), (4, 20), (7, 48), (7, 49), (5, 3.7)]]
              and js["chance"] == [list(M.safe_chance(g, m, k)) for g, m, k in [(5, 3, 0), (5, 3, 22), (4, 15, 1)]],
              "(16) Web: Texte, comb, clamp, safe_chance gleich")
        check(js["fields"] == [M.field_from_seed(*x) for x in mcases["fields"]]
              and js["make"] == [M.make_field(seedrand.Rand(a), g, m) for a, g, m in mcases["make"]],
              "(16) Web: Minenfelder aus Seeds bitgleich (80 Felder)")
        check(js["rounds"] == [M.clean_round(x) for x in mcases["rounds"]]
              and js["clean_history"] == [M.clean_history(x) for x in mcases["history"]]
              and js["marks"] == [M.clean_marks(x) for x in mcases["marks"]],
              "(16) Web: clean_round / clean_history / clean_marks gleich")
        c_, want_mc = 0, []
        for b, g, m, k in mcases["carry"]:
            w, c_ = M.payout_carry(b, g, m, k, c_)
            want_mc.append([w, c_])
        check(js.get("carry") == want_mc and js.get("carry_clean") == [M.clean_carry(v) for v in (42, 100, -1, True, "5", None)],
              "(16) Web: Mines payout_carry / clean_carry gleich")

    pcases = {
        "paths": [[R.choice(P.ROWS), F.new_server_seed(R), F.new_client_seed(R), R.randrange(10 ** 6)] for _ in range(60)],
        "drops": [[R.getrandbits(32), R.choice(P.ROWS)] for _ in range(60)],
        "hits": [{"8": [1] * 9, "9": [True] * 10, "10": [1, 2], "16": [-1, "x", 2.5] + [5] * 14, "x": 1}, "x", P.blank_hits()],
        "history": [[20, -1, True, "x", 100000] + [70] * 15, 5, []],
        "carry": [CARRY_SEQ[:400], [[1, 12, "medium", 6]] * 5],
        "carry_clean": [42, 100, -1, True, "5", None, 0, 99],
    }
    js, err = js_dump("lamaplinko_logic_dump.js", pcases)
    if check(js is not None, "(16) Web-Plinko lässt sich laden", err):
        check(js["key"] == P.KEY and js["rows"] == list(P.ROWS) and js["risks"] == list(P.RISKS) and js["bets"] == list(P.BETS)
              and js["max_balls"] == P.MAX_BALLS and js["history"] == P.HISTORY and js["default_rows"] == P.DEFAULT_ROWS
              and js["default_risk"] == P.DEFAULT_RISK and js["edge_rows"] == P.EDGE_ROWS and js["rain_balls"] == P.RAIN_BALLS
              and js["tables"] == {r: {str(n): list(v) for n, v in P.TABLES[r].items()} for r in P.RISKS},
              "(16) Web: gleiche Plinko-Konstanten und alle 27 Tabellen")
        check(js["expected"] == {r: [float(P.expected_return(n, r)) for n in P.ROWS] for r in P.RISKS},
              "(16) Web: Quoten bitgleich (= float(Fraction))")
        check(js["payout"] == {r: {str(n): [[P.payout(b, n, r, i) for b in (1, 7, 1000)] for i in range(n + 1)] for n in P.ROWS}
                               for r in P.RISKS}
              and js["short"] == {r: {str(n): [P.fmt_short(P.mult100(n, r, i), ",") for i in range(n + 1)] for n in P.ROWS}
                                  for r in P.RISKS}
              and js["probs"] == {str(n): [[math.comb(n, i), 2 ** n] for i in range(n + 1)] for n in P.ROWS}
              and js["fmtprob"] == {str(n): [[P.fmt_prob(n, i), P.fmt_prob(n, i, ",")] for i in range(n + 1)] for n in P.ROWS}
              and js["fmt"] == [[P.fmt_mult(v, s_), P.fmt_short(v, s_)] for v in (20, 100, 125, 1100, 100000) for s_ in (".", ",")],
              "(16) Web: Auszahlungen, Wahrscheinlichkeiten und Texte gleich")
        check(js["paths"] == [P.path_from_seed(*x) for x in pcases["paths"]]
              and js["drops"] == [P.drop_path(seedrand.Rand(a), n) for a, n in pcases["drops"]],
              "(16) Web: Kugelpfade aus Seeds bitgleich (120 Kugeln)")
        check(js["hits"] == [P.clean_hits(x) for x in pcases["hits"]] and js["clean_history"] == [P.clean_history(x) for x in pcases["history"]]
              and js["blank_hits"] == P.blank_hits() and js["add_hit"] == [2] + [0] * 16
              and js["edge"] == [P.is_edge(n, i) for n, i in [(8, 0), (8, 8), (12, 6), (16, 16)]]
              and js["slot_of"] == [2, 0, 3], "(16) Web: clean_hits / clean_history / is_edge gleich")
        want_carry = []
        for seq in pcases["carry"]:
            c_, rows_ = 0, []
            for b, n, rk, sl in seq:
                w, c_ = P.payout_carry(b, n, rk, sl, c_)
                rows_.append([w, c_])
            want_carry.append(rows_)
        check(js.get("carry") == want_carry and js.get("carry_clean") == [P.clean_carry(v) for v in pcases["carry_clean"]],
              "(16) Web: payout_carry / clean_carry gleich")

    # ---- Lama Crash (eigener Zufall - die Fälle oben bleiben unverändert)
    EXC = _load("export_chiplogic_ref_web", "devtools/export_chiplogic_ref.py")
    rx = random.Random(8080)
    MX = X.MAX_MULT100
    NOSEED = {"n": 1, "s": "0" * 32, "c": "abc", "h": "0" * 64}

    def jn(v):
        """Wie nach JSON (Tupel -> Listen) - so kommen die Werte aus node zurück."""
        return json.loads(json.dumps(v))

    def _csim(sim):
        """Ganze Runde mit Hand-Ausstiegen (Schleife wie _crash_sims in devtools/export_chiplogic_ref.py)."""
        rnd_ = X.new_round(NOSEED, [tuple(p_) for p_ in sim["pairs"]])
        carry_, log_, todo, end_ = sim["carry0"], [], [list(a_) for a_ in sim["acts"]], None
        while end_ is None:
            while todo and todo[0][0] == rnd_["t"]:
                _, i_, h_ = todo.pop(0)
                res_ = X.cash_out(rnd_, i_, h_, carry_)
                log_.append(["cash", rnd_["t"], i_, h_] + (list(res_) if res_ is not None else [None, None, None]))
                if res_ is not None:
                    carry_ = res_[2]
            evs, carry_, end_ = X.tick(rnd_, sim["crash"], carry_)
            for ev in evs:
                log_.append(["ev", rnd_["t"], ev[0], ev[1], ev[2], ev[3]])
            if rnd_["t"] > X.MOON_TICK + 2:
                break
        o_ = X.outcome(rnd_, sim["crash"])
        return jn({"log": log_, "end": end_, "t": rnd_["t"], "bets": rnd_["bets"], "carry": carry_, "outcome": o_,
                   "best": X.best_cash(rnd_), "close": X.close_call(rnd_, sim["crash"]), "flying": X.flying(rnd_),
                   "stake": X.stake_total(rnd_), "paid": X.paid_total(rnd_),
                   "check": X.check_outcome(o_, sim["crash"])})

    # ganze Runden: 6 x 30 wie für Rust (eigene Startwerte) + Grenzfälle (Mond, Ziel = Deckel, Sofort-Platzer ...)
    sims = []
    for sd in range(6):
        for sm in EXC._crash_sims(X, random.Random(9100 + sd)):
            sims.append({k_: sm[k_] for k_ in ("pairs", "crash", "acts", "carry0")})
    sims += [
        {"pairs": [[10, MX], [20, 5000000]], "crash": MX, "acts": [], "carry0": 0},
        {"pairs": [[10000, 0]], "crash": MX - 1, "acts": [[X.MOON_TICK - 1, 0, False]], "carry0": 99},
        {"pairs": [[10000, 0], [10000, 0]], "crash": MX, "acts": [[5000, 0, True], [6000, 1, True], [7000, 1, False]],
         "carry0": 50},
        {"pairs": [[50, 0], [20, 101]], "crash": 100, "acts": [[tk_, 0, tk_ % 2 == 0] for tk_ in range(12)], "carry0": 7},
        {"pairs": [[20, 101]], "crash": 101, "acts": [[9, 0, True], [10, 0, False]], "carry0": 0},
        {"pairs": [[100, 0]], "crash": 350, "acts": [[0, 0, False], [694, 0, True], [694, 0, True], [700, 0, False],
                                                    [700, 0, False], [9000, 0, False]], "carry0": 0},
        {"pairs": [[2500, 300], [5000, 0]], "crash": MX, "acts": [[1000, 1, True], [X.MOON_TICK - 1, 1, False]],
         "carry0": 33},
    ]
    want_sims = [_csim(sm) for sm in sims]
    fin = [[dict(NOSEED, t=w_["t"], bets=w_["bets"]), sm["crash"], w_["end"]] for sm, w_ in zip(sims, want_sims)]
    # einzelne Ausstiege: unter x1,01, Hälfte doppelt, ganz ausgestiegen, falscher Index
    cbase = X.new_round(NOSEED, [(10, 0), (25, 200)])
    cash_in = []
    for tk_ in (0, 9, 10, 694, 5000, X.MOON_TICK):
        for i_ in (-1, 0, 1, 2):
            for half_ in (False, True):
                cash_in.append([dict(jn(cbase), t=tk_), i_, half_, rx.randrange(100)])
    r_ = dict(jn(cbase), t=800)
    r_["bets"][0].update(hm=150, man=1, p=7)
    cash_in += [[r_, 0, True, 5], [r_, 0, False, 5]]
    r_ = dict(jn(cbase), t=800)
    r_["bets"][0].update(xm=150, man=2, p=15)
    cash_in += [[r_, 0, False, 0], [r_, 0, True, 0], [r_, 1, True, 98]]
    want_cash = []
    for r_, i_, half_, c_ in cash_in:
        rc = jn(r_)
        res_ = X.cash_out(rc, i_, half_, c_)
        want_cash.append(jn([res_, rc, X.riding(rc["bets"][0]), X.flying(rc)]))
    # gespeicherte Runden: die aus (22) + Typfehler + 60 Runden mit echten Seeds und zufälligen Ausstiegen
    b0 = good["bets"][0]
    crounds = [good, done] + bad_rounds + [
        dict(good, n=True), dict(good, n=-1), dict(good, s="AB" * 16), dict(good, c=""), dict(good, t=True),
        dict(good, t=2.5), dict(good, bets="x"), dict(good, bets=[None]), dict(good, bets=[b0] * 3),
        dict(good, bets=[dict(b0, b=True)]), dict(good, bets=[dict(b0, a=100)]), dict(good, bets=[dict(b0, man=4)]),
        dict(good, bets=[dict(b0, p=-1)]), dict(good, bets=[dict(b0, a=101)]), [], "x", None, 5, {}]
    for _ in range(60):
        st_ = F.blank_state(rx)
        st_["nonce"] = rx.randrange(10 ** 6)
        ent_ = F.begin(st_)
        cr_ = X.crash_point(ent_["s"], ent_["c"], ent_["n"])
        pt_ = X.pop_tick(cr_)
        r_ = X.new_round(ent_, [(rx.choice(X.BETS), rx.choice((0,) + X.TARGETS)) for _ in range(rx.randint(1, 2))])
        r_["t"] = rx.choice((0, max(0, pt_ - 1), pt_, rx.randint(0, pt_ + 5), X.MOON_TICK))
        m_ = X.mult_at(r_["t"])
        for b_ in r_["bets"]:
            if rx.random() < 0.4:
                b_["hm"] = rx.choice((101, m_, cr_, cr_ + 1, 50))
                b_["man"] |= rx.choice((0, 1))
            if rx.random() < 0.4:
                b_["xm"] = rx.choice((101, m_, cr_, max(101, m_ - 1)))
                b_["man"] |= rx.choice((0, 2))
            if rx.random() < 0.3:
                b_["p"] = rx.choice((0, 5, b_["b"] * MX // 100 + 1, b_["b"] * MX // 100 + 2))
            if rx.random() < 0.1:
                b_["man"] = rx.choice((1, 2, 3))
        crounds.append(r_)
    rc_ = dict(NOSEED, t=2000, bets=[{"b": 100, "a": 0, "hm": 300, "xm": 320, "p": 310, "man": 3},
                                     {"b": 20, "a": 150, "hm": 0, "xm": 150, "p": 30, "man": 0}])
    o_ = X.outcome(rc_, 330)
    curve = [X.mult_at(tk_) for tk_ in range(X.MOON_TICK + 1)]
    u32_in = [0, 1, 2, 3, TWO32 // 2, TWO32 - 3, TWO32 - 2, TWO32 - 1] + [rx.getrandbits(32) for _ in range(4000)]
    for T in list(X.TARGETS) + [rx.randint(101, MX) for _ in range(300)]:
        q_ = X.HOUSE * TWO32 // T                # Stufen-Grenze: ab r = 2^32 - q_ gilt crash >= T
        u32_in += [TWO32 - q_ + d_ for d_ in (-2, -1, 0, 1, 2) if 0 <= TWO32 - q_ + d_ < TWO32]
    chart_in = list(range(0, X.MOON_TICK + 1, 37)) + [1, 2, 3, 4, 5, 479, 480, 481, 694, 11519, 11520]
    wt = X.world_ticks()
    ccases = {
        "tick_at": [50, 99, 100, 101, MX, MX + 1, 10 ** 9, 0]
        + [curve[i_] + d_ for i_ in range(0, len(curve), 7) for d_ in (-1, 0, 1)]
        + [rx.randint(0, MX + 10) for _ in range(2000)],
        "u32": u32_in,
        "seeds": [[F.new_server_seed(rx), F.new_client_seed(rx), rx.randrange(10 ** 9)] for _ in range(300)]
        + [["ab" * 16, "kaugummi", n_] for n_ in (0, 1, 7, F.MAX_NONCE)],
        "win_count": list(X.TARGETS) + [50, 100, 101, MX, MX + 1, 10 ** 9] + [rx.randint(101, MX) for _ in range(300)],
        "pop_tick": [100, 101, 102, 350, 1000, 999999, MX - 1, MX] + [X.crash_from_u32(v) for v in u32_in[:2000]],
        "expected": list(X.TARGETS) + [rx.randint(101, MX) for _ in range(300)] + [50, MX + 100],
        "payout": [[[rx.choice(X.BETS + (1, 5, 13, 1250)), rx.randint(101, MX)] for _ in range(300)] for _ in range(3)],
        "sims": sims,
        "cash": cash_in,
        "rounds": [jn(v) for v in crounds],
        "x_clean": [{"hits": [1, 2], "cash": -1, "top": True}, {"hits": [3] * 7, "moon": 2, "close": "x"}, None, "x", [],
                    {"hits": [1, True, -1, 2.5, "x", 5, 2 ** 53], "cash": 2 ** 53 - 1, "half": 2 ** 53, "auto": 3,
                     "inst": 0, "moon": None, "close": 1, "top": MX}, X.blank_x(), {"hits": "x", "top": 5}],
        "x_record": [fin[i_::4] for i_ in range(4)],
        "history": [[50, 100, True, "x", 350, MX + 1] + [200] * 20, None, "x", [], [100, MX, 99, 101.5],
                    list(range(90, 130))],
        "carry_clean": [42, 100, -1, True, "5", None, 0, 99, 2.5],
        "target_ok": [100, 101, MX, MX + 1, True, 150.5, "200", None, 200],
        "fmt": [100, 101, 999, 1000, 1001, 1234, 9999, 10000, 12345, 99999, 100000, 123456, 999999, 1000000,
                1234567, MX] + list(X.TARGETS) + [rx.randint(100, MX) for _ in range(300)],
        "chart": chart_in,
        "ylines": [X.chart_span(v)[1] for v in chart_in] + [1, 50, 100, 101, 150, 200, 201, 12500000, MX]
        + [rx.randint(100, 13 * 10 ** 6) for _ in range(200)],
        "xlines": [X.chart_span(v)[0] for v in chart_in] + [1, 59, 60, 300, 301, 480, 13824, 100000]
        + [rx.randint(1, 20000) for _ in range(200)],
        "blend": [0.0, 315.5, 316.0, 380.25, 405.999, 406.0, 1050.75, 1099.5, 2300.0, 4600.125, 4700.0, -3.5, 12000.5]
        + [rx.uniform(0, 11600) for _ in range(600)] + list(range(300, 420))
        + [w_ - d_ for w_ in wt for d_ in (0, 0.5, 1, 1e-9, 89.999, 90, 90.001, 0.1 + 0.2)],
        "world_of": [100, 149, 150, 299, 300, 999, 1000, 9999, 10000, 9999999, MX, 50, MX * 2]
        + [rx.randint(100, MX) for _ in range(200)],
        "step": [[v, d_] for v in (101, 150, 151, 200, 201, 9999999, MX, 50, MX + 5, 100) + X.TARGETS for d_ in (1, -1)],
        "bucket": [100, 149, 150, 199, 200, 499, 500, 999, 1000, 9999, 10000, 99999, 100000, MX, 50]
        + [rx.randint(100, MX) for _ in range(200)],
        "close": [[r_, c_] for r_, c_, _ in fin] + [[rc_, c_] for c_ in range(290, 420, 3)]
        + [[dict(rc_, bets=[dict(rc_["bets"][0], man=m_)]), 330] for m_ in range(4)],
        "check": [[w_["outcome"], sm["crash"]] for sm, w_ in zip(sims, want_sims)]
        + [[o_, 330], [o_, 329], [o_, 320], [dict(o_, x=331), 330], [dict(o_, b=[]), 330], [dict(o_, b=[[1, 2, 3]]), 330],
           [dict(o_, b=[[1, 50, 0, 0]]), 330], [dict(o_, b=[[1, 0, 331, 0]]), 330], [dict(o_, b=[[1, 0, 0, True]]), 330],
           [dict(o_, b=[[1, 0, 0, 0]] * 3), 330], ["x", 330], [None, 330], [{"x": 330}, 330], [dict(o_, b="x"), 330],
           [dict(o_, x="330"), 330], [dict(o_, b=[[1, 0, 0, 2.5]]), 330], [dict(o_, b=[[1, -5, 0, 0]]), 330]],
    }
    ccases["chance"] = ccases["win_count"]
    ccases["music_m"] = [0, 99, 100, 149, 150, 199, 200, 229, 230, 299, 300, 999, 1000, 9999, 10000, 99999, 100000,
                         1000000, 9999999, MX] + list(range(100, 10_500_001, 9973))
    js, err = js_dump("lamacrash_logic_dump.js", ccases)
    if check(js is not None, "(16) Web-Crash lässt sich laden", err):
        check(js["consts"] == {"key": X.KEY, "house": X.HOUSE, "max": MX, "min_cash": X.MIN_CASH, "bets": list(X.BETS),
                               "max_bets": X.MAX_BETS, "history": X.HISTORY, "tps": X.TPS, "mu0": X.MU0,
                               "growth": X.GROWTH, "mu_unit": X.MU_UNIT, "moon_tick": X.MOON_TICK,
                               "speed_turbo": X.SPEED_TURBO, "speed_ff": X.SPEED_FF, "targets": list(X.TARGETS),
                               "default_target": X.DEFAULT_TARGET, "worlds": list(X.WORLDS),
                               "world_at": list(X.WORLD_AT), "blend_ticks": X.BLEND_TICKS, "brave": X.BRAVE_MULT,
                               "space": X.SPACE_MULT, "close_min": X.CLOSE_MIN, "close_pct": X.CLOSE_PCT,
                               "buckets": list(X.BUCKETS), "chart_min_ticks": X.CHART_MIN_TICKS,
                               "y_steps": list(X._Y_STEPS), "x_steps": list(X._X_STEPS), "two32": X.TWO32},
              "(16) Web: gleiche Crash-Konstanten (Quote, Deckel, Kurve, Ziel-Leiter, Welten, Fächer, Achsen)")
        check(js["curve"] == curve and js["mu"] == [X.mu_at(tk_) for tk_ in range(X.MOON_TICK + 1)]
              and js["clamp"] == [[X.mult_at(v), X.mu_at(v)] for v in (-5, -1, 0, 0.9, 1.5, 11519.99, 11520, 11521, 1e9)]
              and js["tick_at"] == [X.tick_at(v) for v in ccases["tick_at"]],
              "(16) Web: ganze Steigkurve bitgleich (11.521 Takte, mu und m100, Begrenzung) + tick_at (%d Werte)"
              % len(ccases["tick_at"]))
        check(js["u32"] == [X.crash_from_u32(v) for v in u32_in] and js["seeds"] == [X.crash_point(*v) for v in ccases["seeds"]],
              "(16) Web: Platzpunkt bitgleich (%d Zufallszahlen inkl. Stufen-Grenzen, %d echte Seeds)"
              % (len(u32_in), len(ccases["seeds"])))
        check(js["win_count"] == [X.win_count(v) for v in ccases["win_count"]]
              and js["chance"] == [[X.fmt_chance(v), X.fmt_chance(v, ",")] for v in ccases["win_count"]]
              and js["pop_tick"] == [X.pop_tick(v) for v in ccases["pop_tick"]]
              and all(Fraction(*a_[0]) == X.expected_return(v) and a_[1] == float(X.expected_return(v))
                      for a_, v in zip(js["expected"], ccases["expected"])),
              "(16) Web: win_count / fmt_chance / pop_tick / expected_return (exakt) gleich")
        bad_s = [i_ for i_, (a_, b_) in enumerate(zip(js["sims"], want_sims)) if a_ != b_]
        n_moon = sum(1 for w_ in want_sims if w_["end"] == "moon")
        n_hand = sum(1 for w_ in want_sims for ln in w_["log"] if ln[0] == "cash" and ln[4] is not None)
        n_auto = sum(1 for w_ in want_sims for ln in w_["log"] if ln[0] == "ev" and ln[3] == "auto")
        check(not bad_s and len(js["sims"]) == len(sims),
              "(16) Web: %d ganze Runden gleich (%d Mondlandungen, %d Hand-Ausstiege, %d Auto-Ziele; Takt-Regel, "
              "Hälften, Übertrag, Ergebnis)" % (len(sims), n_moon, n_hand, n_auto), bad_s[:3])
        check(js["cash"] == want_cash and js["half"] == [[X.half_stake(b), X.rest_stake(b)] for b in list(X.BETS) + [1, 3, 7]]
              and js["new_round"] == jn(X.new_round({"n": 5, "s": "ab" * 16, "c": "kaugummi", "h": "0" * 64},
                                                    [(10, 0), (20, 150)]))
              and list(js["new_round"]) == ["n", "s", "c", "h", "t", "bets"]
              and list(js["new_round"]["bets"][0]) == ["b", "a", "hm", "xm", "p", "man"],
              "(16) Web: cash_out-Einzelfälle (%d), Hälften, new_round samt Schlüsselreihenfolge gleich" % len(cash_in))
        want_rounds = [jn(X.clean_round(v)) for v in ccases["rounds"]]
        check(js["rounds"] == want_rounds
              and all(list(a_) == list(b_) and [list(x_) for x_ in a_["bets"]] == [list(x_) for x_ in b_["bets"]]
                      for a_, b_ in zip(js["rounds"], want_rounds) if a_ and b_),
              "(16) Web: clean_round gleich (%d Rohdaten, davon %d gültig; Schlüsselreihenfolge)"
              % (len(want_rounds), sum(1 for v in want_rounds if v)))
        want_x = []
        for seq in ccases["x_record"]:
            x_ = X.blank_x()
            for r_, c_, e_ in seq:
                X.record_x(x_, r_, c_, e_)
            want_x.append(x_)
        check(js["x_record"] == want_x and js["x_clean"] == [X.clean_x(v) for v in ccases["x_clean"]]
              and js["blank_x"] == X.blank_x() and list(js["blank_x"]) == list(X.blank_x()),
              "(16) Web: record_x (%d Runden) / clean_x / blank_x gleich" % len(fin))
        want_pay = []
        for seq in ccases["payout"]:
            c_, rows_ = 0, []
            for b_, m_ in seq:
                w_, c_ = X.payout_carry(b_, m_, c_)
                rows_.append([w_, c_])
            want_pay.append(rows_)
        check(js["history"] == [X.clean_history(v) for v in ccases["history"]]
              and js["carry_clean"] == [X.clean_carry(v) for v in ccases["carry_clean"]]
              and js["target_ok"] == [X.target_ok(v) for v in ccases["target_ok"]] and js["payout"] == want_pay,
              "(16) Web: clean_history / clean_carry / target_ok / payout_carry gleich")
        check(js["fmt"] == [[X.fmt_mult(v), X.fmt_mult(v, ","), X.fmt_short(v), X.fmt_short(v, ",")] for v in ccases["fmt"]],
              "(16) Web: fmt_mult / fmt_short gleich (%d Werte)" % len(ccases["fmt"]))
        check(js["chart"] == [list(X.chart_span(v)) for v in chart_in]
              and js["ylines"] == [X.chart_ylines(v) for v in ccases["ylines"]]
              and js["xlines"] == [X.chart_xlines(v) for v in ccases["xlines"]],
              "(16) Web: Kurven-Achsen chart_span / chart_ylines / chart_xlines gleich")
        check(js["blend"] == [list(X.world_blend(v)) for v in ccases["blend"]]
              and js["world_of"] == [X.world_of(v) for v in ccases["world_of"]] and js["world_ticks"] == list(wt),
              "(16) Web: world_blend (Gleitkomma, %d Takte) / world_of / world_ticks gleich" % len(ccases["blend"]))
        check(js["step"] == [X.step_target(v, d_) for v, d_ in ccases["step"]]
              and js["bucket"] == [X.bucket_of(v) for v in ccases["bucket"]]
              and [Fraction(*v) for v in js["bucket_prob"]] == [X.bucket_prob(i_) for i_ in range(len(X.BUCKETS))]
              and js["bucket_prob_f"] == [float(X.bucket_prob(i_)) for i_ in range(len(X.BUCKETS))],
              "(16) Web: step_target / bucket_of / bucket_prob (exakt) gleich")
        check(js["close"] == [X.close_call(r_, c_) for r_, c_ in ccases["close"]]
              and js["check"] == [X.check_outcome(jn(o2), c_) for o2, c_ in ccases["check"]],
              "(16) Web: close_call (%d) / check_outcome (%d) gleich" % (len(ccases["close"]), len(ccases["check"])))

        # Musik: games/lamacrash_music.py ohne pygame laden - compose()/squeak() brauchen
        # kein Audio, ein Platzhalter für "audio" reicht (gerendert wird hier nichts)
        def _py_music():
            import types
            had_audio = "audio" in sys.modules
            if not had_audio:
                sys.modules["audio"] = types.ModuleType("audio")
            pkg = types.ModuleType("lcx_web_pkg")
            pkg.__path__ = [os.path.join(REPO, "games")]
            sys.modules["lcx_web_pkg"] = pkg
            try:
                for name in ("lamacrash_logic", "lamacrash_music"):
                    spec = importlib.util.spec_from_file_location("lcx_web_pkg." + name,
                                                                  os.path.join(REPO, "games", name + ".py"))
                    mod = importlib.util.module_from_spec(spec)
                    sys.modules["lcx_web_pkg." + name] = mod
                    spec.loader.exec_module(mod)
                return sys.modules["lcx_web_pkg.lamacrash_music"]
            finally:
                if not had_audio:
                    del sys.modules["audio"]

        MUS = _py_music()
        jm = js["music"]
        want_songs = [jn(MUS.compose(s)) for s in range(MUS.STAGES)]
        n_notes = n_exact = 0
        same_mu = jm["stages"] == MUS.STAGES and len(jm["compose"]) == len(want_songs)
        for a_, b_ in zip(jm["compose"], want_songs):
            same_mu = same_mu and a_["bars"] == b_["bars"] and a_["bpm"] == b_["bpm"] and a_["length"] == b_["length"]
            same_mu = same_mu and len(a_["voices"]) == len(b_["voices"])
            for va, vb in zip(a_["voices"], b_["voices"]):
                same_mu = same_mu and va["vol"] == vb["vol"] and len(va["notes"]) == len(vb["notes"])
                for na, nb in zip(va["notes"], vb["notes"]):
                    n_notes += 1
                    n_exact += na == nb
                    # Frequenz: 440 * 2^(x/12) - Math.pow und C-pow dürfen im letzten Bit abweichen
                    same_mu = same_mu and na[0] == nb[0] and na[2:] == nb[2:] and abs(na[1] - nb[1]) <= 1e-12 * nb[1]
        check(same_mu and jm["bpm"] == [MUS.bpm(s) for s in range(6)] and jm["beat"] == [MUS.beat_seconds(s) for s in range(6)]
              and jm["stage_of"] == [MUS.stage_of(m_) for m_ in ccases["music_m"]]
              and jm["squeak"] == [jn(MUS.squeak(m_)) for m_ in ccases["music_m"]],
              "(16) Web: Lama-Crash-Musik gleich (%d Noten aller 6 Stufen, davon %d bitgleich; Tempi, Stufen, Quietschen)"
              % (n_notes, n_exact))
else:
    print("  --   (16) node nicht gefunden - Web-Vergleich übersprungen")

# ================================================================ Rust (Quelltext)
print("\n[Rust]")
RS = os.path.join(REPO, "rust", "crates")


def _read(*parts):
    p = os.path.join(RS, *parts)
    if not os.path.exists(p):
        return None
    with open(p, encoding="utf-8") as f:
        return f.read()


prs = _read("pgz-games", "src", "lamaplinko_logic.rs")
mrs = _read("pgz-games", "src", "lamamines_logic.rs")
frs = _read("pgz-core", "src", "fairness.rs")
crs = _read("pgz-core", "src", "chipstats.rs")
if check(None not in (prs, mrs, frs, crs), "(17) Rust-Module vorhanden (fairness, chipstats, lamamines_logic, lamaplinko_logic)"):
    rs_tab = {}
    for r, n, body in re.findall(r"const T_(LOW|MEDIUM|HIGH)_(\d+): \[i64; \d+\] =\s*\[([^\]]*)\]", prs):
        rs_tab[(r.lower(), int(n))] = tuple(int(v) for v in body.replace("_", "").replace("\n", " ").split(",") if v.strip())
    check(rs_tab == {(r, n): P.TABLES[r][n] for r in P.RISKS for n in P.ROWS}, "(17) Rust: alle 27 Plinko-Tabellen gleich")
    EXP = _load("export_chiplogic_ref", "devtools/export_chiplogic_ref.py")
    ref_now = None
    if os.path.exists(EXP.OUT):
        with open(EXP.OUT, encoding="utf-8") as f:
            ref_now = f.read()
    check(ref_now == EXP.text(), "(17) Rust-Prüfwerte data/chiplogic_ref.json passen zur Python-Fassung "
          "(sonst: python devtools/export_chiplogic_ref.py)")
    check("pub const MAX_MULT100: i64 = 10_000_000;" in mrs and "pub const HOUSE: i64 = 97;" in mrs
          and 'pub const CLIENT_ALPHABET: &str = "%s";' % F.CLIENT_ALPHABET in frs and "pub const CURVE: usize = 100;" in crs
          and "0x428a2f98" in frs and "pub const LOG: usize = 12;" in frs,
          "(17) Rust: Konstanten (Deckel, Quote, Alphabet, Log, Kurve, SHA-256-Tabelle)")
    print("  --   (17) Rechen-Gleichstand: cd rust; cargo test -p pgz-core --offline --lib -- fairness chipstats;"
          " cargo test -p pgz-games --offline --lib -- chiplogic")

# Lama Crash: Konstanten und Tabellen im Rust-Quelltext gegen die Python-Fassung
xrs = _read("pgz-games", "src", "lamacrash_logic.rs")
urs = _read("pgz-games", "src", "lamacrash_music.rs")
lbrs = _read("pgz-core", "src", "lamabank.rs")
if check(None not in (xrs, urs, lbrs), "(17) Rust-Module vorhanden (lamacrash_logic, lamacrash_music, lamabank)"):
    import ast

    def _rs_const(src, name, env):
        """Wert der Rust-Konstante ``name`` (``const NAME: TYP = WERT;`` am
        Zeilenanfang): Zahlen mit ``_``, Listen, Tupel, Texte, ``a * b`` und
        Namen schon gelesener Konstanten (env). None = nicht gefunden."""
        m = re.search(r"(?m)^(?:pub )?const %s: [^=]+=\s*(.*?);\s*$" % name, src, re.S)
        if not m:
            return None
        txt = re.sub(r"(?<=\d)_(?=\d)", "", m.group(1).strip()).lstrip("&")
        for k_, v_ in env.items():
            txt = re.sub(r"\b%s\b" % k_, repr(v_), txt)
        prod = re.fullmatch(r"(\d+) \* (\d+)", txt)
        if prod:
            return int(prod.group(1)) * int(prod.group(2))
        try:
            return ast.literal_eval(txt)
        except (ValueError, SyntaxError):
            return None

    def _py_assign(rel, names):
        """Literale Zuweisungen eines Python-Moduls (ohne es zu importieren)."""
        with open(os.path.join(REPO, *rel.split("/")), encoding="utf-8") as f:
            tree = ast.parse(f.read())
        out = {}
        for node in tree.body:
            if isinstance(node, ast.Assign) and isinstance(node.targets[0], ast.Name) \
                    and node.targets[0].id in names:
                out[node.targets[0].id] = ast.literal_eval(node.value)
        return out

    def _plain_json(v):
        return json.loads(json.dumps(v))

    want_x = {"KEY": X.KEY, "HOUSE": X.HOUSE, "MAX_MULT100": X.MAX_MULT100, "MIN_CASH": X.MIN_CASH,
              "BETS": X.BETS, "MAX_BETS": X.MAX_BETS, "HISTORY": X.HISTORY, "TPS": X.TPS, "MU0": X.MU0,
              "GROWTH": X.GROWTH, "MU_UNIT": X.MU_UNIT, "MOON_TICK": X.MOON_TICK, "SPEED_TURBO": X.SPEED_TURBO,
              "SPEED_FF": X.SPEED_FF, "TARGETS": X.TARGETS, "DEFAULT_TARGET": X.DEFAULT_TARGET,
              "WORLDS": X.WORLDS, "WORLD_AT": X.WORLD_AT, "BLEND_TICKS": X.BLEND_TICKS,
              "BRAVE_MULT": X.BRAVE_MULT, "SPACE_MULT": X.SPACE_MULT, "CLOSE_MIN": X.CLOSE_MIN,
              "CLOSE_PCT": X.CLOSE_PCT, "BUCKETS": X.BUCKETS, "CHART_MIN_TICKS": X.CHART_MIN_TICKS,
              "Y_STEPS": X._Y_STEPS, "X_STEPS": X._X_STEPS, "TWO32": X.TWO32, "X_COUNTERS": X._X_COUNTERS}
    got_x, env_x = {}, {}
    for name_ in want_x:
        got_x[name_] = _rs_const(xrs, name_, env_x)
        if isinstance(got_x[name_], int):
            env_x[name_] = got_x[name_]
    bad_x = [n_ for n_ in want_x if _plain_json(got_x[n_]) != _plain_json(want_x[n_])]
    check(not bad_x, "(17) Rust: Lama-Crash-Konstanten wie Python (%d: Quote, Deckel, Kurve, Tempo, %d Ziele, "
          "Welten, Erfolge, Fächer, Achsen, Zähler)" % (len(want_x), len(X.TARGETS)), bad_x)
    py_mu = _py_assign("games/lamacrash_music.py",
                       {"STAGES", "_BPM", "_ROOTS", "_VOICE_VOL", "_SQUEAK_AT", "_SQUEAK_BASE"})
    env_u = {}
    got_u = {}
    for py_name, rs_name in (("STAGES", "STAGES"), ("_BPM", "BPM"), ("_ROOTS", "ROOTS"), ("_VOICE_VOL", "VOICE_VOL"),
                             ("_SQUEAK_AT", "SQUEAK_AT"), ("_SQUEAK_BASE", "SQUEAK_BASE")):
        got_u[py_name] = _rs_const(urs, rs_name, env_u)
        if isinstance(got_u[py_name], int):
            env_u[rs_name] = got_u[py_name]
    bad_u = [n_ for n_ in py_mu if _plain_json(got_u.get(n_)) != _plain_json(py_mu[n_])]
    check(len(py_mu) == 6 and not bad_u, "(17) Rust: Lama-Crash-Musik - Stufen, Tempi, Grundtöne, Stimmen-"
          "Lautstärken, Quietsch-Schwellen wie Python", bad_u)
    music_ref = subprocess.run([sys.executable, os.path.join(REPO, "rust", "tools", "lamacrash_music_ref.py"), "--check"],
                               cwd=REPO, capture_output=True, text=True)
    check(music_ref.returncode == 0, "(17) Rust-Prüfwerte data/lamacrash_music_ref.txt (Noten aller 6 Stufen bitgenau) "
          "passen zur Python-Fassung (sonst: python rust/tools/lamacrash_music_ref.py)",
          (music_ref.stdout + music_ref.stderr).strip()[-200:])
    py_lb = _py_assign("lamabank.py", {"GAMES", "MIN_BET"})
    rs_games = _rs_const(lbrs, "GAMES", {})
    rs_min = _rs_const(lbrs, "MIN_BET", {})
    check(rs_games == list(py_lb.get("GAMES", ())) and rs_min == list(py_lb.get("MIN_BET", {}).items())
          and rs_games[-1] == X.KEY and dict(rs_min).get(X.KEY) == X.BETS[0],
          "(17) Rust: Lama-Bank kennt Lama Crash (GAMES-Reihenfolge, MIN_BET = kleinste Stufe %d) wie lamabank.py"
          % X.BETS[0], (rs_games, rs_min))
    check("fn chiplogic_crash_wie_python" in xrs and 'include_str!("data/chiplogic_ref.json")' in xrs
          and "fn lamacrash_music_wie_python" in urs and 'include_str!("data/lamacrash_music_ref.txt")' in urs,
          "(17) Rust-Unit-Tests für Lama Crash vorhanden (Regeln über chiplogic_ref.json, Musik über "
          "lamacrash_music_ref.txt)")
    print("  --   (17) Lama Crash rechnet: cd rust; cargo test -p pgz-games --offline --lib -- chiplogic lamacrash_music")

print("\n%s - %d Fehler" % ("BESTANDEN" if not FAILS else "FEHLGESCHLAGEN", len(FAILS)))
sys.exit(1 if FAILS else 0)
