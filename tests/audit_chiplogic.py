# -*- coding: utf-8 -*-
"""Headless-Audit der Chip-Regeln von Lama Mines und Lama Plinko (ohne pygame).

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
Web       (16) Python = JS über web/tools/fairness_dump.js,
              lamamines_logic_dump.js und lamaplinko_logic_dump.js (node),
Rust      (17) Rust-Quelltext hat dieselben Tabellen und Konstanten; der
              Gleichstand der Rechnungen läuft als Rust-Unit-Test:
                  cd rust
                  cargo test -p pgz-core --offline --lib -- fairness chipstats
                  cargo test -p pgz-games --offline --lib -- chiplogic

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
    mcases = {
        "fields": [[g, R.randint(1, g * g - 1), sv[i], F.new_client_seed(R), R.randrange(10 ** 6)]
                   for i, g in enumerate(list(M.GRIDS) * 10)],
        "make": [[R.getrandbits(32), g, R.randint(1, g * g - 1)] for g in list(M.GRIDS) * 10],
        "rounds": [good, dict(good, open=[])] + bads[:16] + [dict(good, open=safe[:21])],
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

print("\n%s - %d Fehler" % ("BESTANDEN" if not FAILS else "FEHLGESCHLAGEN", len(FAILS)))
sys.exit(1 if FAILS else 0)
