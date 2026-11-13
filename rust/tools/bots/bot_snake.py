"""Bot fuer Snake: lenkt gierig zum naechsten Apfel (weicht Waenden,
Hindernissen und Koerpern aus), boostet ab und zu, drueckt P (Prestige),
startet nach Game Over neu. Im Setup: Modi, 2D/3D, Schalter, Apfelzahl,
Personalisierung (C) und 3D-Kamera-Menue (K).

Aufruf: bot_snake.py <ordner> <frames> [setup-tasten]   (SN_MODE=single|multi)
SN_MENUE=1 besucht im Setup zuerst die Menues. Schreibt <ordner>/sn_keys.txt.
Eigener Zufall (random.Random(37)).
"""
import os
import random
import sys
sys.path.insert(0, "rust/tools")
import vergleich

sp, frames = sys.argv[1], int(sys.argv[2])
vorlauf = sys.argv[3].split() if len(sys.argv) > 3 else []
spur = []
plan = []
r = random.Random(37)
orig = vergleich.ereignis
MENUE = os.environ.get("SN_MENUE") == "1"
besucht = {}
wechsel = []
DIRS = {(0, -1): "Up", (0, 1): "Down", (-1, 0): "Left", (1, 0): "Right"}
DIRS2 = {(0, -1): "w", (0, 1): "s", (-1, 0): "a", (1, 0): "d"}


def frei(game, c, ignore_tail=True):
    x, y = c
    if game.wrap_active:
        x %= game.cols
        y %= game.rows
    elif not (0 <= x < game.cols and 0 <= y < game.rows):
        return False
    if (x, y) in game.obstacles:
        return False
    for sn in game.snakes:
        body = sn.body[1:] if ignore_tail else sn.body
        if (x, y) in body:
            return False
    return True


def wahl(game, sn):
    hx, hy = sn.body[-1]
    ziele = list(game.foods) + ([game.golden] if game.golden else []) + list(game.specials)
    best, bestd = None, None
    for d in DIRS:
        if d == (-sn.direction[0], -sn.direction[1]):
            continue
        n = (hx + d[0], hy + d[1])
        if n in game.portals:
            n = game.portals[n]
            n = (n[0] + d[0], n[1] + d[1])
        if not frei(game, n):
            continue
        dist = min((abs(n[0] - z[0]) + abs(n[1] - z[1]) for z in ziele), default=0)
        dist += r.random() * 0.5
        if best is None or dist < bestd:
            best, bestd = d, dist
    return best


def bot(game, k, IE):
    n = len(spur)
    if n < len(vorlauf):
        spur.append(vorlauf[n])
        orig(game, vorlauf[n], IE)
        return
    taste = ""
    st = game.state if not (game.state == "play" and game.game_over) else "over"
    if st != besucht.get("_l"):
        wechsel.append((n, st))
    besucht["_l"] = st
    besucht[st] = besucht.get(st, 0) + 1
    if game.state == "play" and getattr(game, "slot", None) is not None:
        if besucht.get("slot", 0) == 0:
            wechsel.append((n, "slot"))
        besucht["slot"] = besucht.get("slot", 0) + 1
    if plan:
        taste = plan.pop(0)
    elif st == "setup":
        if n % 8 == 0:
            x = r.random()
            if MENUE and besucht.get("personalize", 0) == 0 and x < 0.5:
                taste = r.choice(["c", "@%d:%d" % game.brush_rect.center])
            elif MENUE and besucht.get("cam3d", 0) == 0 and game.view3d_active and x < 0.5:
                taste = r.choice(["k", "@%d:%d" % game.wrap_rect.center])
            elif x < 0.35:
                taste = r.choice(["Return", "space", "@%d:%d" % game.start_rect.center])
            elif x < 0.55:
                taste = r.choice(["Left", "Right", "1", "2", "3", "4", "5", "6",
                                  "@%d:%d" % game.mode_left.center, "@%d:%d" % game.mode_panel.center])
            elif x < 0.75:
                taste = r.choice(["v", "w", "b", "f", "h", "@%d:%d" % game.view_rect.center,
                                  "@%d:%d" % game.apples_rect.center, "@%d:%d" % game.bonus_rect.center])
            else:
                taste = r.choice(["Return", "space"])
    elif st == "personalize":
        if n % 6 == 0:
            m = game._ngb_menu
            x = r.random()
            if x < 0.15:
                taste = r.choice(["Escape", "Return", "@%d:%d" % m.done_rect.center])
            elif x < 0.35:
                taste = "@%d:%d" % r.choice(list(m.tab_rects.values())).center
            elif x < 0.55:
                taste = r.choice(["Tab", "1", "2", "3", "4", "5", "6", "7"])
            elif x < 0.70:
                tiles = m.head_tiles if m.tab == "head" else m.grid_tiles
                taste = "@%d:%d" % r.choice(tiles)[0].center
            elif x < 0.92:
                sl = (m._sliders_head if m.tab == "head" else m._sliders_grid)
                if m.tab == "banner":
                    sl = m.banner_slider
                    if r.random() < 0.3:
                        taste = "@%d:%d" % m.banner_toggle.center
                if not taste:
                    s = r.choice(list(sl.values()))
                    t = s["track"]
                    taste = r.choice(["@%d:%d" % s["minus"].center, "@%d:%d" % s["plus"].center,
                                      "@%d:%d" % (t.left + r.randrange(t.width), t.centery)])
            else:
                taste = "@%d:%d" % r.choice(list(m.grid_ab.values())).center
    elif st == "cam3d":
        if n % 6 == 0:
            x = r.random()
            if x < 0.15:
                taste = r.choice(["Escape", "k", "@%d:%d" % game.cam_back_rect.center])
            elif x < 0.6:
                taste = r.choice(["Left", "Right", "Up", "Down"])
            else:
                taste = "@%d:%d" % r.choice([game.cam_smooth_rect, game.cam_turn_rect, game.cam_fov_minus,
                                             game.cam_fov_plus, game.cam_height_minus,
                                             game.cam_height_plus]).center
    elif st == "over":
        if n % 50 == 0:
            taste = r.choice(["Return", "space", "p", "Return"])
    elif st == "play" and game.slot is None:
        x = r.random()
        if x < 0.01:
            taste = "p"
        elif x < 0.03:
            plan.extend(["+space"] + [""] * r.randint(10, 60) + ["-space"])
            taste = plan.pop(0)
        else:
            for i, sn in enumerate(game.snakes):
                if not sn.alive or taste:
                    continue
                d = wahl(game, sn)
                if d is None or d == sn.next_direction:
                    continue
                if game.view3d_active:
                    if sn.turn_queue:
                        continue
                    dx, dy = sn.direction
                    if d == (dy, -dx):
                        taste = "Left"
                    elif d == (-dy, dx):
                        taste = "Right"
                elif game.multiplayer:
                    taste = (DIRS2 if i == 0 else DIRS)[d]
                else:
                    taste = DIRS[d]
    spur.append(taste)
    if taste:
        orig(game, taste, IE)


vergleich.ereignis = bot
d = vergleich.datenordner(sp, "bot_probe")
vergleich.python_lauf("snake", os.environ.get("SN_MODE", "single"), 7, frames, ["x"] * frames, d,
                      os.path.join(sp, "bot.png"))
open(os.path.join(sp, "sn_keys.txt"), "w").write(",".join(spur))
print("Ende", {k: v for k, v in besucht.items() if k != "_l"})
print("Wechsel", wechsel[:30])
