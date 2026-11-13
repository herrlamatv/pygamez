# -*- coding: utf-8 -*-
"""Geometry-Dash-Physik Rust gegen Python: alle mitgelieferten Level mit
ihren Loesungen (devtools/geodash_*proofs.json), dazu verschobene Loesungen
(Tod an anderer Stelle), Startpunkte mitten im Level und zufaellig
verunstaltete Level-Daten (normalize_level). Verglichen werden Pruefsumme,
Laenge, Endzustand und der Hash alle ``every`` Schritte.

    .venv/Scripts/python.exe rust/tools/geodash_check.py [varianten]
"""
import json
import os
import random
import subprocess
import sys

HIER = os.path.dirname(os.path.abspath(__file__))
WURZEL = os.path.dirname(os.path.dirname(HIER))
sys.path.insert(0, os.path.join(WURZEL, "games"))

import geodash_core as core  # noqa: E402


def lade(name):
    with open(os.path.join(WURZEL, name), encoding="utf-8") as f:
        return json.load(f)


def verunstalten(d, r):
    d = json.loads(json.dumps(d))
    objs = d.get("objects", [])
    for _ in range(r.randint(1, 12)):
        if not objs:
            break
        i = r.randrange(len(objs))
        o = list(objs[i])
        w = r.random()
        if w < 0.2:
            o[3 if len(o) > 3 else 1] = r.choice([5, -1, "2", 1.7, None, "x", 99999])
        elif w < 0.35:
            objs.append(list(o))
        elif w < 0.5:
            o = o[:r.randint(1, 3)]
        elif w < 0.6:
            o[0] = r.choice(["coin", "nix", 3, "color", "p_ship"])
        elif w < 0.7:
            o = o + [r.randint(-5, 300) for _ in range(r.randint(1, 6))]
        objs[i] = o
    for key in ("speed", "mode", "length"):
        if r.random() < 0.3:
            d[key] = r.choice([-3, 9, "2", 1.5, None, 5000])
    if r.random() < 0.3:
        d["bg"] = r.choice([[1, 2], "rot", [300, -4, "7"], None])
    if r.random() < 0.3:
        d["music"] = r.choice(["chip", "rock", 3])
    return d


def main():
    varianten = int(sys.argv[1]) if len(sys.argv) > 1 else 3
    r = random.Random(17)
    faelle = []
    for lvf, prf in (("games/levels/geodash.json", "devtools/geodash_proofs.json"),
                     ("games/levels/geodash_extra.json", "devtools/geodash_extra_proofs.json")):
        levels = lade(lvf)["levels"]
        proofs = lade(prf)
        for d in levels:
            tg = list(proofs.get(d["id"], {}).get("toggles", []))
            faelle.append({"level": d, "toggles": tg, "every": 97, "start": None})
            for _ in range(varianten):
                t2 = sorted(set(max(0, t + r.randint(-6, 6)) if r.random() < 0.2 else t for t in tg))
                start = r.choice([None, None, r.randint(1, max(2, core.level_length(d) - 5))])
                faelle.append({"level": d, "toggles": t2, "every": 211, "start": start})
            faelle.append({"level": verunstalten(d, r), "toggles": tg, "every": 0, "start": None})
    for d in lade("games/levels/geodash_templates.json")["levels"]:
        faelle.append({"level": d, "toggles": sorted(r.sample(range(4000), 40)), "every": 50, "start": None})

    erwartet = []
    for f in faelle:
        lv = core.Level(f["level"])
        start = core.new_state(lv, f["start"]) if f["start"] is not None else None
        s, trace = core.run_toggles(lv, f["toggles"], trace_every=f["every"], start=start)
        tr = ",".join("%d:%s" % (k, h) for k, h in trace)
        erwartet.append("%s|%d|%s|%s" % (core.content_hash(f["level"]), core.level_length(lv.data),
                                         core.state_str(s), tr))
    eingabe = "\n".join(json.dumps(f) for f in faelle) + "\n"
    exe = os.path.join(WURZEL, "rust.output", "release", "examples", "geodash_check.exe")
    r2 = subprocess.run([exe], input=eingabe, capture_output=True, text=True, encoding="utf-8")
    zeilen = r2.stdout.splitlines()
    falsch = 0
    for i, (a, b) in enumerate(zip(erwartet, zeilen + [""] * len(erwartet))):
        if a != b:
            falsch += 1
            if falsch <= 5:
                print("Fall %d (%s):\n  py: %s\n  rs: %s" % (i, faelle[i]["level"].get("id"), a[:300], b[:300]))
    if r2.stderr.strip():
        print(r2.stderr[-800:])
    print("%d/%d gleich" % (len(erwartet) - falsch, len(erwartet)))
    return 1 if falsch else 0


if __name__ == "__main__":
    sys.exit(main())
