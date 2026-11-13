"""Bot fuer Sudoku: loest ueberwiegend richtig, macht aber auch Fehler und nutzt
Notizen, Undo/Redo, Tipps, Kandidaten, Radieren (Rechtsklick) und Marker.

Aufruf: bot_sudoku.py <ordner> <frames> <modus> [setup-tasten]
Schreibt <ordner>/sud_keys.txt. Eigener Zufall (random.Random(77)).
"""
import os
import random
import sys
sys.path.insert(0, "rust/tools")
import vergleich

sp, frames, modus = sys.argv[1], int(sys.argv[2]), sys.argv[3]
vorlauf = (sys.argv[4].split() if len(sys.argv) > 4 else []) + ["Return"]
spur = []
plan = []
r = random.Random(77)
fehler_quote = float(os.environ.get("SUD_FEHLER", "0.08"))
orig = vergleich.ereignis


def mitte(game, i):
    n = game.N
    return (game.bx + (i % n) * game.cell + game.cell // 2,
            game.by + (i // n) * game.cell + game.cell // 2)


def aktion(game):
    offen = [i for i in range(game.cells)
             if not game.locked[i] and game.board[i] != game.solution[i]]
    if not offen:
        return []
    i = r.choice(offen)
    x, y = mitte(game, i)
    w = r.random()
    if w < 0.04:
        return ["u"]
    if w < 0.06:
        return ["y"]
    if w < 0.08:
        return ["n"]
    if w < 0.10:
        return ["h"]
    if w < 0.11:
        return ["c"]
    if w < 0.13:
        return ["@%d:%d:3" % (x, y)]
    if w < 0.15:
        return ["@%d:%d" % (x, y), "m"]
    if w < 0.15 + fehler_quote:
        falsch = [d for d in range(1, game.N + 1) if d != game.solution[i]]
        return ["@%d:%d" % (x, y), str(r.choice(falsch))]
    return ["@%d:%d" % (x, y), str(game.solution[i])]


def bot(game, k, IE):
    n = len(spur)
    if n < len(vorlauf):
        spur.append(vorlauf[n])
        orig(game, vorlauf[n], IE)
        return
    taste = ""
    if game.state == "play":
        if game.game_over:
            if n % 150 == 0:
                taste = "Return"
        elif plan:
            taste = plan.pop(0)
        elif n % 4 == 0:
            plan.extend(aktion(game))
            if game.note_mode and r.random() < 0.5:
                plan.insert(0, "n")
            taste = plan.pop(0) if plan else ""
    spur.append(taste)
    if taste:
        orig(game, taste, IE)


vergleich.ereignis = bot
d = vergleich.datenordner(sp, "bot_probe")
vergleich.python_lauf("sudoku", modus, 7, frames, ["x"] * frames, d, os.path.join(sp, "bot.png"))
open(os.path.join(sp, "sud_keys.txt"), "w").write(",".join(spur))
print("Ende", sum(1 for s in spur if s))
