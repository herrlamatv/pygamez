"""Bot fuer Pac-Man: BFS zur naechsten Pille (Python-Lauf), Spur als Ereignisse."""
import os
import sys
from collections import deque
sys.path.insert(0, "rust/tools")
import vergleich

sp, frames, out, vorab = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4].split(",")
spur = []
orig = vergleich.ereignis
NAMES = {(0, -1): "Up", (0, 1): "Down", (-1, 0): "Left", (1, 0): "Right"}
warte = [0]


def schritt(g):
    start = g.pac.tile
    gefahr = set()
    for gh in g.ghosts:
        if gh.state == "maze" and not gh.frightened:
            gx, gy = gh.tile
            for dx in (-1, 0, 1):
                for dy in (-1, 0, 1):
                    gefahr.add(((gx + dx) % 28, gy + dy))
    prev = {start: None}
    q = deque([start])
    while q:
        t = q.popleft()
        c, r = t
        if t != start and g.pellets[r][c]:
            while prev[t] != start:
                t = prev[t]
            return (t[0] - start[0], t[1] - start[1])
        for d in ((0, -1), (-1, 0), (0, 1), (1, 0)):
            n = ((c + d[0]) % 28, r + d[1])
            if n not in prev and g._pac_open((c + d[0], r + d[1])) and n not in gefahr:
                prev[n] = t
                q.append(n)
    return None


def bot(game, k, IE):
    g = game
    e = ""
    if len(spur) < len(vorab):
        e = vorab[len(spur)]
    elif g.state == "gameover":
        warte[0] += 1
        if warte[0] > 40:
            warte[0] = 0
            e = "Return"
    elif g.state in ("play", "ready"):
        d = schritt(g)
        if d is not None:
            if abs(d[0]) > 1:
                d = (-1 if d[0] > 0 else 1, 0)
            if d != g.pac.want:
                e = NAMES.get(d, "")
    spur.append(e)
    if e:
        orig(game, e, IE)


vergleich.ereignis = bot
d = vergleich.datenordner(sp, "bot_probe")
vergleich.python_lauf("pacman", "single", 7, frames, ["x"] * frames, d, os.path.join(sp, "bot.png"))
open(os.path.join(sp, out), "w").write(",".join(spur))
print("fertig")
