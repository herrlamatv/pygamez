# -*- coding: utf-8 -*-
"""
export_chiplogic_ref.py
=======================
Schreibt die Prüfwerte der Chip-Regeln (fairness, chipstats, Lama Mines,
Lama Plinko) aus der Python-Fassung nach
``rust/crates/pgz-games/src/data/chiplogic_ref.json``. Die Rust-Unit-Tests
(``lamamines_logic.rs`` / ``lamaplinko_logic.rs``) lesen die Datei per
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
