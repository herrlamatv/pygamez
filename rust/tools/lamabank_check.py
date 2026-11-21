# -*- coding: utf-8 -*-
"""
lamabank_check.py
=================
Kreuztest der Lama-Bank: dieselben Buchungen laufen ueber ``lamabank.py``
und ueber ``pgz-core/src/lamabank.rs`` (Beispiel ``lamabank_io``), jeweils
in einem frischen Datenordner mit derselben Ausgangs-``mem.json``.
Verglichen werden die Antworten Zeile fuer Zeile und die geschriebene
``mem.json`` Byte fuer Byte (nur die Ziffern des Zeitstempels ``_generated``
maskiert).

    cargo build -p pgz-core --release --offline --example lamabank_io
    .venv/Scripts/python.exe rust/tools/lamabank_check.py

Abgedeckt: frisches Konto, ``bet_and_pay`` (mit/ohne Gewinn, nicht gedeckt,
Einsatz 0, Zusatz-Spielstand, unbekanntes Spiel), Mindesteinsaetze und Pleite
je Spiel samt Kredit, Spielstand von vor dem Mines-/Plinko-Update (4 Spiele
im ``ledger``), Migration der alten Blackjack-/Poker-Abschnitte, Poker-
Tischstapel (escrow) inkl. Erstattung beim Laden.
"""

import json
import os
import shutil
import subprocess
import sys
import tempfile

HIER = os.path.dirname(os.path.abspath(__file__))
WURZEL = os.path.dirname(os.path.dirname(HIER))
sys.path.insert(0, HIER)
from vergleich import maskiert  # noqa: E402  (Zeitstempel `_generated`)
# Bauordner wie vergleich.py (CARGO_TARGET_DIR oder rust.output/)
EXE = os.path.join(os.path.abspath(os.environ.get("CARGO_TARGET_DIR") or os.path.join(WURZEL, "rust.output")),
                   "release", "examples", "lamabank_io.exe")
PY = os.path.join(WURZEL, ".venv", "Scripts", "python.exe")
if not os.path.exists(PY):
    PY = sys.executable

PY_SEITE = r'''
import json, os, sys
sys.path.insert(0, %(wurzel)r)
import store
store._DIR = %(daten)r
store._PATH = os.path.join(store._DIR, "mem.json")
store._LEGACY_HS_PATH = os.path.join(store._DIR, "highscores.json")
import lamabank as lb

def b(v):
    return "true" if v else "false"

for zeile in open(%(befehle)r, encoding="utf-8").read().splitlines():
    w = zeile.split()
    if not w:
        continue
    c = w[0]
    g = w[1] if len(w) > 1 else ""
    m = w[2] if len(w) > 2 else None
    if c == "load":
        lb.load(); out = "ok"
    elif c == "reset_cache":
        lb.reset_cache(); out = "ok"
    elif c in ("balance", "escrow", "refills"):
        out = str(getattr(lb, c)())
    elif c in ("ledger", "value_for", "score_for"):
        out = str(getattr(lb, c)(g))
    elif c == "min_bet":
        out = str(lb.min_bet(g, m))
    elif c == "is_broke":
        out = b(lb.is_broke(g, m))
    elif c == "debit":
        out = b(lb.debit(int(w[1]), w[2]))
    elif c == "credit":
        lb.credit(int(w[1]), w[2], save=w[3] == "1"); out = "ok"
    elif c == "bet_and_pay":
        if len(w) > 5:
            out = b(lb.bet_and_pay(int(w[1]), int(w[2]), w[3], w[4], json.loads(w[5])))
        else:
            out = b(lb.bet_and_pay(int(w[1]), int(w[2]), w[3]))
    elif c == "refill":
        out = b(lb.refill_if_broke(g, m))
    elif c == "set_escrow":
        out = str(lb.set_escrow(int(w[1])))
    elif c == "pay_from_escrow":
        out = str(lb.pay_from_escrow(int(w[1]), w[2]))
    elif c == "clear_escrow":
        out = str(lb.clear_escrow(save=w[1] == "1"))
    elif c == "settle_escrow":
        lb.settle_escrow(int(w[1]), w[2]); out = "ok"
    elif c == "set_extra":
        lb.set_extra(g, json.loads(w[2])); out = "ok"
    elif c == "get_extra":
        out = json.dumps(lb.get_extra(g), separators=(",", ":"), ensure_ascii=False)
    else:
        out = "? " + c
    print("%%s -> %%s" %% (zeile, out))
'''

FRISCH = {"mem": {"lang": "de"}}
# Spielstand von vor dem Mines-/Plinko-Update: ledger/peak kennen 4 Spiele.
ALT_4 = {"mem": {"lang": "de"},
         "casino": {"chips": 1530, "ledger": {"blackjack": 120, "poker": -40, "casino": 300, "lamacrossing": 150},
                    "peak": {"blackjack": 1200, "poker": 1000, "casino": 1500, "lamacrossing": 1150},
                    "escrow": 0, "refills": 2, "migrated": True,
                    "extra": {"lamacrossing": {"hist": [1, 2]}, "casino_roulette": [5, 17]}}}
# Noch nicht migriert: alte Abschnitte blackjack/poker + Highscores.
LEGACY = {"mem": {"lang": "de"}, "blackjack": {"chips": 1800, "best": 2100}, "poker": {"chips": 650},
          "highscores": {"poker": 1400, "casino": 900}}
# Abgebrochene Poker-Hand: escrow wird beim Laden erstattet.
ESCROW = {"mem": {"lang": "de"},
          "casino": {"chips": 400, "ledger": {}, "peak": {}, "escrow": 250, "refills": 0, "migrated": True,
                     "extra": {}}}

FAELLE = [
    ("frisch_plinko", FRISCH, """load
bet_and_pay 10 0 lamaplinko
bet_and_pay 10 25 lamaplinko
bet_and_pay 5 5 lamaplinko
bet_and_pay 0 30 lamaplinko
bet_and_pay 0 0 lamaplinko
bet_and_pay -5 0 lamaplinko
bet_and_pay 999999 0 lamaplinko
bet_and_pay 1 0 lamaplinko lamaplinko {"round":1,"hist":[1,2],"fair":{"nonce":3}}
get_extra lamaplinko
bet_and_pay 3 7 lamaplinko lamaplinko {"round":2}
bet_and_pay 2 0 lamaplinko lamamines null
get_extra lamamines
get_extra lamaplinko
bet_and_pay 999999 5 lamaplinko lamaplinko {"x":1}
get_extra lamaplinko
balance
ledger lamaplinko
value_for lamaplinko
score_for lamaplinko
bet_and_pay 7 70 zzz
ledger zzz
score_for zzz
reset_cache
balance
ledger lamaplinko
score_for lamaplinko"""),
    ("frisch_mines", FRISCH, """load
min_bet lamamines
min_bet lamaplinko
min_bet lamacrossing
min_bet poker video
min_bet casino slots
debit 10 lamamines
credit 45 lamamines 1
debit 20 lamamines
score_for lamamines
value_for lamamines
set_extra lamamines {"round":{"grid":5,"mines":3,"bet":20,"open":[3,7]},"hist":[]}
get_extra lamamines
debit 1015 lamamines
balance
is_broke lamamines
is_broke lamaplinko
refill lamaplinko
refill lamamines
refills
balance
debit 1000 lamaplinko
is_broke lamaplinko
refill lamaplinko
balance
ledger lamaplinko
score_for lamaplinko"""),
    ("alt_4_spiele", ALT_4, """load
ledger lamamines
ledger lamaplinko
score_for lamamines
score_for lamacrossing
bet_and_pay 100 970 lamamines
bet_and_pay 50 0 lamaplinko lamaplinko {"hist":[0.3]}
get_extra lamacrossing
get_extra casino_roulette
balance
refills"""),
    ("legacy", LEGACY, """load
balance
score_for blackjack
score_for poker
score_for casino
score_for lamamines
bet_and_pay 20 0 lamamines"""),
    ("escrow", ESCROW, """load
balance
escrow
set_escrow 300
pay_from_escrow 120 poker
bet_and_pay 10 40 lamaplinko
settle_escrow 500 poker
balance
escrow
set_escrow 100
reset_cache
load
balance
escrow
clear_escrow 1"""),
]


def lauf(befehle, daten, mem, rust):
    if os.path.exists(daten):
        shutil.rmtree(daten)
    os.makedirs(daten)
    with open(os.path.join(daten, "mem.json"), "w", encoding="utf-8") as f:
        json.dump(mem, f)
    if rust:
        env = dict(os.environ, PYGAMEZ_DATA_DIR=daten)
        r = subprocess.run([EXE, befehle], capture_output=True, text=True, env=env)
    else:
        code = PY_SEITE % {"wurzel": WURZEL, "daten": daten, "befehle": befehle}
        r = subprocess.run([PY, "-c", code], capture_output=True, text=True, cwd=WURZEL)
    return (r.stdout + r.stderr).strip().splitlines()


def main():
    if not os.path.exists(EXE):
        raise SystemExit("Erst bauen: cargo build -p pgz-core --release --offline --example lamabank_io")
    ordner = tempfile.mkdtemp(prefix="pgz_lamabank_")
    schlecht = 0
    for name, mem, befehle in FAELLE:
        datei = os.path.join(ordner, name + ".txt")
        with open(datei, "w", encoding="utf-8") as f:
            f.write(befehle + "\n")
        dp, dr = os.path.join(ordner, name + "_py"), os.path.join(ordner, name + "_rs")
        ap, ar = lauf(datei, dp, mem, False), lauf(datei, dr, mem, True)
        fehler = None
        for i, (x, y) in enumerate(zip(ap, ar)):
            if x != y:
                fehler = "Zeile %d: py=%r rust=%r" % (i + 1, x, y)
                break
        if fehler is None and len(ap) != len(ar):
            fehler = "%d gegen %d Zeilen (py: %s)" % (len(ap), len(ar), ap[-1:])
        if fehler is None:
            with open(os.path.join(dp, "mem.json"), "rb") as f:
                mp = maskiert(f.read().decode("utf-8"))
            with open(os.path.join(dr, "mem.json"), "rb") as f:
                mr = maskiert(f.read().decode("utf-8"))
            if mp != mr:
                fehler = "mem.json verschieden (%d gegen %d Bytes)" % (len(mp), len(mr))
        schlecht += fehler is not None
        print("%-6s %-14s %d Befehle%s" % ("ok" if fehler is None else "FEHLER", name, len(ap),
                                           "" if fehler is None else "  " + fehler), flush=True)
    print("Abweichungen: %d   (Ordner: %s)" % (schlecht, ordner))
    return 1 if schlecht else 0


if __name__ == "__main__":
    sys.exit(main())
