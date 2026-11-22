# -*- coding: utf-8 -*-
"""
export_chiplogic_ref.py
=======================
Schreibt die Prüfwerte der Chip-Regeln (fairness, chipstats, Lama Mines,
Lama Plinko, Lama Crash) aus der Python-Fassung nach
``rust/crates/pgz-games/src/data/chiplogic_ref.json``. Die Rust-Unit-Tests
(``lamamines_logic.rs`` / ``lamaplinko_logic.rs`` / ``lamacrash_logic.rs``) lesen die Datei per
``include_str!`` und rechnen alles nach:

    python devtools/export_chiplogic_ref.py           # Datei neu schreiben
    python devtools/export_chiplogic_ref.py --check   # nur prüfen (Exit 1 = veraltet)
    cd rust && cargo test -p pgz-games --offline --lib -- chiplogic

``tests/audit_chiplogic.py`` prüft, dass die Datei zur aktuellen
Python-Fassung passt. Gleitkommazahlen stehen als Text (repr) darin.
"""

import importlib.util
import json
import os
import random
import sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, REPO)

import chipstats as C  # noqa: E402
import fairness as F  # noqa: E402

OUT = os.path.join(REPO, "rust", "crates", "pgz-games", "src", "data", "chiplogic_ref.json")


def _load(name, rel):
    spec = importlib.util.spec_from_file_location(name, os.path.join(REPO, *rel.split("/")))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def _compact(x):
    return json.dumps(x, separators=(",", ":"), ensure_ascii=False)


def build():
    """Alle Fälle samt erwarteten Ergebnissen (Python-Fassung)."""
    M = _load("lamamines_logic", "games/lamamines_logic.py")
    P = _load("lamaplinko_logic", "games/lamaplinko_logic.py")
    digest = lambda x: F.sha256_hex(_compact(x))  # noqa: E731
    out = {}

    # ---- Fairness: Seeds aus random.Random wie py::Random
    r = random.Random(42)
    st = F.blank_state(r)
    blank = dict(st)
    e = F.begin(st)
    F.advance(st, r)
    F.new_client(st, r)
    r3 = random.Random(3)
    cs_in = [{"server": "AB" * 16, "client": "kamaz", "nonce": True}, None,
             {"server": "ab" * 16, "client": "l0l", "nonce": -1},
             {"server": "ab" * 16, "client": "kaugummi", "nonce": 41}]
    out["fair"] = {"seed": 42, "blank": blank, "begin": e, "after": st,
                   "clean_seed": 3, "clean_in": cs_in, "clean_out": [F.clean_state(x, r3) for x in cs_in]}
    rr = F.round_rand("lamamines", "ab" * 16, "kaugummi", 7)
    out["rand"] = {"game": "lamamines", "s": "ab" * 16, "c": "kaugummi", "n": 7,
                   "u32": [rr.next_u32() for _ in range(5)]}
    raw_log = [F.make_entry(e, {"i": i}) for i in range(14)] + [
        "x", None, {"n": -1, "s": e["s"], "c": e["c"], "h": e["h"]},
        {"n": 3, "s": e["s"], "c": e["c"], "h": "0" * 64, "o": 0.5},
        {"n": 4, "s": e["s"], "c": e["c"], "h": e["h"], "o": [1, [2, [3, [4]]]]},
        {"n": 5, "s": e["s"], "c": e["c"], "h": e["h"], "o": [[[[[1]]]]]},
        {"n": True, "s": e["s"], "c": e["c"], "h": e["h"]}]
    out["log"] = {"in": raw_log, "out": F.clean_log(raw_log), "verify": [F.verify(x) for x in F.clean_log(raw_log)]}
    out["sha"] = {t: F.sha256_hex(t) for t in ["", "abc", "x" * 55, "x" * 56, "x" * 64, "a" * 1000,
                                               "Lama äöü € \U0001f999"]}

    # ---- Statistik
    s = C.blank()
    ops = [[100, 250, 250], [100, 0, 0], [50, 50, 100]] + [[10, 0, 0]] * 120
    for w, p, m in ops:
        C.record(s, w, p, m)
    dirty = {"rounds": True, "wins": -3, "losses": "7", "wagered": 100, "paid": 40, "net": 999, "best_win": 2.5,
             "best_mult": 300, "curve": [1, True, "x", 2.5, -4] + [9] * 120,
             "x": {"hits": [1, 2], "bad": 1.5, "n": None}}
    out["stats"] = {"ops": ops, "after": s, "rtp": repr(C.rtp(s)), "win_rate": repr(C.win_rate(s)),
                    "clean_in": [dirty, "x", None, s], "clean_out": [C.clean(x) for x in (dirty, "x", None, s)]}

    # ---- Mines
    mult, safe, pay, allsafe = [], [], [], []
    for g in M.GRIDS:
        for m in range(1, g * g):
            ks = range(0, g * g - m + 2)
            mult.append([M.mult100(g, m, k) for k in ks])
            safe.append([M.safe_percent(g, m, k) for k in ks])
            pay.append([M.payout(55, g, m, k) for k in ks])
            allsafe.append([M.all_safe_percent(g, m, k) for k in ks])
    R = random.Random(2026)
    fields = []
    for g in list(M.GRIDS) * 3:
        m = R.randint(1, g * g - 1)
        sv, cv, nv = F.new_server_seed(R), F.new_client_seed(R), R.randrange(10 ** 6)
        fields.append([g, m, sv, cv, nv, M.field_from_seed(g, m, sv, cv, nv)])
    srv = "ab" * 16
    fld = M.field_from_seed(5, 3, srv, "kaugummi", 7)
    sc = [c for c in range(25) if c not in fld]
    good = {"bet": 100, "grid": 5, "mines": 3, "n": 7, "s": srv, "c": "kaugummi", "h": F.commit(srv), "open": sc[:4]}
    rounds = [good, dict(good, open=[]), dict(good, open=sc[:3] + [fld[0]]), dict(good, open=[sc[0]] * 2),
              dict(good, open=sc[:22]), dict(good, open=[True]), dict(good, bet=True), dict(good, grid=8),
              dict(good, mines=25), dict(good, h="0" * 64), dict(good, n=1.0), dict(good, open=sc[:21]), "x"]
    marks = [{"4": [3, 1, 3, True, 16, -1, "2", 0], "5": "x", "7": list(range(60)), "9": [1]}, None]
    exp_cases = [[5, 3, 2], [7, 1, 1], [6, 2, 2], [7, 24, 25], [4, 15, 1]]
    out["mines"] = {
        "digest_mult": digest(mult), "digest_safe": digest(safe), "digest_pay55": digest(pay), "digest_allsafe": digest(allsafe),
        "mult_5_3": [M.mult100(5, 3, k) for k in range(8)], "mult_7_1": [M.mult100(7, 1, k) for k in range(6)],
        "fields": fields, "rounds_in": rounds, "rounds_out": [M.clean_round(x) for x in rounds],
        "marks_in": marks, "marks_out": [M.clean_marks(x) for x in marks],
        "history_in": [[0, 154, -3, "x", True, 999, 10 ** 8] + [100] * 20, None],
        "history_out": [M.clean_history(x) for x in ([0, 154, -3, "x", True, 999, 10 ** 8] + [100] * 20, None)],
        # Gleitkomma-Weg wie JS/Rust: comb/comb * m / 100
        "expected": [[g, m, k, repr((M.comb(g * g - k, m) / M.comb(g * g, m)) * M.mult100(g, m, k) / 100)]
                     for g, m, k in exp_cases],
        "ladder": [[list(x) for x in M.ladder(5, 3, k)] for k in (0, 20, 22)],
        "text": [M.fmt_mult(1142, ","), M.fmt_mult(10_000_000), M.fmt_percent(845, ","), M.fmt_percent(5)],
        "carry": _mines_carry(M, R),
    }

    # ---- Plinko
    paths = []
    for _ in range(12):
        n = R.choice(P.ROWS)
        sv, cv, nv = F.new_server_seed(R), F.new_client_seed(R), R.randrange(10 ** 6)
        paths.append([n, sv, cv, nv, P.path_from_seed(n, sv, cv, nv)])
    hits = [{"8": [1] * 9, "9": [True] * 10, "10": [1, 2], "16": [-1, "x", 2.5] + [5] * 14, "x": 1}, "x"]
    out["plinko"] = {
        "digest_tables": digest({r: {str(n): list(P.TABLES[r][n]) for n in P.ROWS} for r in P.RISKS}),
        "digest_prob": digest({str(n): [[P.fmt_prob(n, i), P.fmt_prob(n, i, ",")] for i in range(n + 1)]
                               for n in P.ROWS}),
        "digest_short": digest({r: {str(n): [P.fmt_short(P.mult100(n, r, i), ",") for i in range(n + 1)]
                                    for n in P.ROWS} for r in P.RISKS}),
        "expected": {r: [repr(float(P.expected_return(n, r))) for n in P.ROWS] for r in P.RISKS},
        "paths": paths, "hits_in": hits, "hits_out": [P.clean_hits(x) for x in hits],
        "history_in": [[20, -1, True, "x", 100000, 2000000] + [70] * 15, 5],
        "history_out": [P.clean_history(x) for x in ([20, -1, True, "x", 100000, 2000000] + [70] * 15, 5)],
        "payout": [P.payout(7, 16, "high", 0), P.payout(5, 8, "high", 4), P.payout(1, 16, "low", 8)],
        "carry": _carry(P, R),
        "carry_clean_in": [42, 100, -1, True, "5", None],
        "carry_clean_out": [P.clean_carry(v) for v in (42, 100, -1, True, "5", None)],
    }
    # ---- Crash (eigener Zufall - die Werte von Mines/Plinko bleiben unverändert)
    X = _load("lamacrash_logic", "games/lamacrash_logic.py")
    RC = random.Random(5050)
    two32 = 2 ** 32
    curve = [X.mult_at(tk) for tk in range(X.MOON_TICK + 1)]
    seeds = []
    for _ in range(24):
        sv, cv, nv = F.new_server_seed(RC), F.new_client_seed(RC), RC.randrange(10 ** 9)
        seeds.append([sv, cv, nv, X.crash_point(sv, cv, nv)])
    us = [0, 1, 2, 171798691, two32 // 2, two32 - 2, two32 - 1] + [RC.getrandbits(32) for _ in range(40)]
    sims = _crash_sims(X, RC)
    xs = X.blank_x()
    for sim in sims[:12]:
        rnd = {"n": 1, "s": "0" * 32, "c": "abc", "h": "0" * 64, "t": sim["t"], "bets": sim["bets"]}
        X.record_x(xs, rnd, sim["crash"], sim["end"])
    st = F.blank_state(RC)
    st["nonce"] = 17
    ent = F.begin(st)
    good = X.new_round(ent, [(100, 0), (20, 150)])
    crash_g = X.crash_point(ent["s"], ent["c"], ent["n"])
    rounds_in = [good, dict(good, t=10), dict(good, t=X.pop_tick(crash_g)), dict(good, h="0" * 64),
                 dict(good, bets=[{"b": 10, "a": 100, "hm": 0, "xm": 0, "p": 0, "man": 0}]),
                 dict(good, bets=[{"b": 10, "a": 0, "hm": 0, "xm": 0, "p": 0, "man": 2}]),
                 dict(good, bets=[{"b": 10, "a": 0, "hm": 0, "xm": 0, "p": 0, "man": True}]),
                 dict(good, t=-1), dict(good, bets=[]), None, "x"]
    out["crash"] = {
        "consts": [X.HOUSE, X.MAX_MULT100, X.MIN_CASH, X.MU0, X.GROWTH, X.MU_UNIT, X.MOON_TICK,
                   X.SPEED_TURBO, X.SPEED_FF, X.HISTORY, X.DEFAULT_TARGET, X.BLEND_TICKS, X.CHART_MIN_TICKS,
                   X.BRAVE_MULT, X.SPACE_MULT, X.CLOSE_MIN, X.CLOSE_PCT, X.MAX_BETS, X.TPS],
        "bets": list(X.BETS), "targets": list(X.TARGETS), "world_at": list(X.WORLD_AT),
        "buckets": list(X.BUCKETS),
        "digest_curve": digest(curve),
        "curve": [[tk, curve[tk], X.mu_at(tk)] for tk in (0, 1, 9, 10, 693, 694, 2305, 4608, 6912, 11519, 11520)],
        "tick_at": [[m, X.tick_at(m)] for m in (50, 100, 101, 150, 200, 777, 1000, 123456,
                                               X.MAX_MULT100, X.MAX_MULT100 + 1)],
        "u32": [[u, X.crash_from_u32(u)] for u in us],
        "seeds": seeds,
        "win_count": [[tv, X.win_count(tv)] for tv in X.TARGETS],
        "chance": [[tv, X.fmt_chance(tv), X.fmt_chance(tv, ",")] for tv in (101, 150, 200, 777, 10000, 123456,
                                                                           X.MAX_MULT100)],
        "pop_tick": [[c, X.pop_tick(c)] for c in (100, 101, 102, 350, 1000, 999999, X.MAX_MULT100)],
        "sims": sims,
        "x": xs,
        "x_clean_in": [{"hits": [1, 2], "cash": -1, "top": True}, {"hits": [3] * 7, "moon": 2, "close": "x"}, None],
        "x_clean_out": [X.clean_x(v) for v in ({"hits": [1, 2], "cash": -1, "top": True},
                                              {"hits": [3] * 7, "moon": 2, "close": "x"}, None)],
        "rounds_in": rounds_in,
        "rounds_out": [X.clean_round(v) for v in rounds_in],
        "history_in": [50, 100, True, "x", 350, X.MAX_MULT100 + 1] + [200] * 20,
        "history_out": X.clean_history([50, 100, True, "x", 350, X.MAX_MULT100 + 1] + [200] * 20),
        "fmt": [[v, X.fmt_mult(v), X.fmt_mult(v, ","), X.fmt_short(v), X.fmt_short(v, ",")]
                for v in (100, 101, 999, 1000, 1234, 9999, 10000, 12345, 99999, 100000, 123456, 999999,
                          1000000, X.MAX_MULT100)],
        "chart": [[tk, list(X.chart_span(tk)), X.chart_ylines(X.chart_span(tk)[1]),
                   X.chart_xlines(X.chart_span(tk)[0])] for tk in (0, 50, 480, 694, 1500, 3000, 6000, 11520)],
        "blend": [[repr(f), list(X.world_blend(f))] for f in (0.0, 315.5, 316.0, 380.25, 405.999, 406.0,
                                                             1050.75, 1099.5, 2300.0, 4600.125, 4700.0)],
        "world_of": [[v, X.world_of(v)] for v in (100, 149, 150, 299, 300, 999, 1000, 9999, 10000, 9999999,
                                                  X.MAX_MULT100)],
        "step": [[v, d, X.step_target(v, d)] for v in (101, 150, 151, 200, 201, 9999999, X.MAX_MULT100)
                 for d in (1, -1)],
        "bucket": [[v, X.bucket_of(v)] for v in (100, 149, 150, 199, 200, 499, 500, 999, 1000, 9999, 10000,
                                                 99999, 100000, X.MAX_MULT100)],
        "bucket_prob": [[X.bucket_prob(i).numerator, X.bucket_prob(i).denominator] for i in range(len(X.BUCKETS))],
    }
    return out


def _crash_sims(X, R):
    """Ganze Runden mit Hand-Ausstiegen: Protokoll je Takt (Ausstiege und Ereignisse)."""
    out = []
    for case in range(30):
        n = R.randint(1, 2)
        pairs = [[R.choice(X.BETS), R.choice((0,) + X.TARGETS[:28])] for _ in range(n)]
        crash = min(X.crash_from_u32(R.getrandbits(32)), 30000) if case < 28 else X.MAX_MULT100
        end_t = X.pop_tick(crash)
        acts = sorted([[R.randint(0, min(end_t + 3, 4000)), R.randrange(n), R.random() < 0.5]
                       for _ in range(R.randint(0, 3))])
        rnd = X.new_round({"n": 1, "s": "0" * 32, "c": "abc", "h": "0" * 64}, [tuple(p) for p in pairs])
        carry = R.randrange(100)
        carry0 = carry
        log, todo, end = [], list(acts), None
        while end is None:
            while todo and todo[0][0] == rnd["t"]:
                _, i, h = todo.pop(0)
                res = X.cash_out(rnd, i, h, carry)
                log.append(["cash", rnd["t"], i, h] + (list(res) if res is not None else [None, None, None]))
                if res is not None:
                    carry = res[2]
            evs, carry, end = X.tick(rnd, crash, carry)
            for ev in evs:
                log.append(["ev", rnd["t"], ev[0], ev[1], ev[2], ev[3]])
        out.append({"pairs": pairs, "crash": crash, "acts": acts, "carry0": carry0, "log": log, "end": end,
                    "t": rnd["t"], "bets": rnd["bets"], "carry": carry, "outcome": X.outcome(rnd, crash),
                    "best": X.best_cash(rnd), "close": X.close_call(rnd, crash)})
    return out


def _mines_carry(M, R):
    """Auszahlungen mit Übertrag: [[bet, grid, mines, k, win, carry], ...]."""
    out, carry = [], 0
    for _ in range(30):
        g = R.choice(M.GRIDS)
        m = R.randint(1, g * g - 1)
        k = R.randint(1, g * g - m)
        b = R.choice(M.BETS)
        win, carry = M.payout_carry(b, g, m, k, carry)
        out.append([b, g, m, k, win, carry])
    return out


def _carry(P, R):
    """Kugelfolge mit Übertrag: [[bet, rows, risk, slot, win, carry], ...]."""
    out, carry = [], 0
    for _ in range(40):
        b, n, rk = R.choice((1, 2, 5, 7)), R.choice(P.ROWS), R.choice(P.RISKS)
        sl = R.randint(0, n)
        win, carry = P.payout_carry(b, n, rk, sl, carry)
        out.append([b, n, rk, sl, win, carry])
    return out


def text():
    return json.dumps(build(), ensure_ascii=False, indent=1) + "\n"


def main():
    new = text()
    old = None
    if os.path.exists(OUT):
        with open(OUT, encoding="utf-8") as f:
            old = f.read()
    if "--check" in sys.argv:
        ok = old == new
        print("chiplogic_ref.json %s" % ("aktuell" if ok else "VERALTET"))
        return 0 if ok else 1
    with open(OUT, "w", encoding="utf-8", newline="\n") as f:
        f.write(new)
    print("geschrieben:", OUT)
    return 0


if __name__ == "__main__":
    sys.exit(main())
