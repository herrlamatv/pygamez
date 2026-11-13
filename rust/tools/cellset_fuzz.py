"""Vergleicht py::CellSet (Rust) mit CPythons set aus Zellen-Tupeln.

    .venv/Scripts/python.exe rust/tools/cellset_fuzz.py [runden]

Erzeugt zufaellige add/discard-Folgen, laesst sie von
`cargo run --example cellset_fuzz` nachspielen und vergleicht nach jeder
Runde die Iterationsreihenfolge.
"""
import os
import random
import subprocess
import sys

HIER = os.path.dirname(os.path.abspath(__file__))
RUST = os.path.dirname(HIER)

runden = int(sys.argv[1]) if len(sys.argv) > 1 else 2000
r = random.Random(11)
befehle = []
erwartet = []
for _ in range(runden):
    befehle.append("n")
    s = set()
    for _ in range(r.randint(1, 60)):
        x, y = r.randrange(32), r.randrange(24)
        if s and r.random() < 0.3:
            x, y = r.choice(sorted(s)) if r.random() < 0.7 else (x, y)
            s.discard((x, y))
            befehle.append("d %d %d" % (x, y))
        else:
            s.add((x, y))
            befehle.append("a %d %d" % (x, y))
    befehle.append("p")
    erwartet.append(" ".join("%d,%d" % c for c in s))

env = dict(os.environ)
env["PATH"] = os.path.expanduser("~/.cargo/bin") + os.pathsep + env.get("PATH", "")
res = subprocess.run(["cargo", "run", "-q", "--release", "--offline", "-p", "pgz-engine",
                      "--example", "cellset_fuzz"], cwd=RUST, input="\n".join(befehle) + "\n",
                     capture_output=True, text=True, env=env)
zeilen = res.stdout.splitlines()
if len(zeilen) != len(erwartet):
    print("Fehler: %d Zeilen statt %d\n%s" % (len(zeilen), len(erwartet), res.stderr[-2000:]))
    sys.exit(1)
schlecht = sum(1 for a, b in zip(zeilen, erwartet) if a != b)
print("CellSet: %d/%d Runden gleich" % (runden - schlecht, runden))
sys.exit(1 if schlecht else 0)
