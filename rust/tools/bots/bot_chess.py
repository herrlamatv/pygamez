"""Bot fuer Schach.

Partie/2 Spieler: stellt im Setup zufaellig Staerke, Farbe, Uhr, Chess960 und
Drehen ein, zieht dann als Mensch (Klick-Klick, Ziehen mit der Maus oder
Pfeiltasten + Enter), waehlt bei Umwandlungen eine Figur, nutzt Hinweis,
Rueckgaengig, Drehen, Remis-Angebot und Aufgeben (mit Ja/Nein im Dialog) und
laesst mit CH_TROEDEL=1 manchmal die Uhr ablaufen. Nach dem Ende: Neue
Partie, Setup oder Knoepfe.

Raetsel: oeffnet Aufgaben (Tasten/Klick), spielt meist den richtigen Zug,
manchmal einen falschen, zeigt die Loesung, blaettert weiter, wechselt Stufen.

Aufruf: bot_chess.py <ordner> <frames>   (CH_MODE=single|multi|puzzles)
Schreibt <ordner>/ch_keys.txt. Eigener Zufall (random.Random(41)); die
Zugwahl rechnet mit einer eigenen Transpositionstabelle.
"""
import os
import random
import sys
sys.path.insert(0, "rust/tools")
sys.path.insert(0, ".")
import vergleich
from games import chess_engine as ce

sp, frames = sys.argv[1], int(sys.argv[2])
spur = []
plan = []
r = random.Random(int(os.environ.get("CH_BOTSEED", "41")))
orig = vergleich.ereignis
MODUS = os.environ.get("CH_MODE", "single")
TROEDEL = os.environ.get("CH_TROEDEL", "0") == "1"
besucht = {}
wechsel = []
enden = []
tt = {}
zustand = {"troedeln": False}


def mitte(rc):
    return "%d:%d" % rc.center


def klick(rc):
    return "@" + mitte(rc)


def zug_waehlen(game):
    pos = game.pos
    legal = list(game.legal)
    if not legal:
        return 0
    if r.random() < 0.5:
        s = ce.Search(pos, ce.LEVELS[1], tt=tt, rng=r)
        ce.run_to_end(s)
        if s.best_move in legal:
            return s.best_move
    caps = [m for m in legal if pos.is_capture(m)]
    if caps and r.random() < 0.6:
        return r.choice(caps)
    return r.choice(legal)


def zug_plan(game, m):
    frm = m & 255
    to = (m >> 8) & 255
    if m >> 20 == ce.M_CASTLE and r.random() < 0.5:
        to = (frm & 0x70) + (6 if to > frm else 2)
    a = game._sq_rect(frm)
    b = game._sq_rect(to)
    x = r.random()
    if x < 0.55:
        return [klick(a), "", klick(b)]
    if x < 0.85:
        return ["<" + mitte(a), "~%d:%d" % (a.centerx + 7, a.centery + 3), "~" + mitte(b), ">" + mitte(b)]
    # Tastatur: Cursor auf das Startfeld, Enter, Cursor aufs Ziel, Enter
    out = []
    cur = list(game.cursor)
    for ziel in (frm, to):
        dr, dc = game._sq_to_disp(ziel)
        while cur[0] != dr:
            out.append("Down" if dr > cur[0] else "Up")
            cur[0] += 1 if dr > cur[0] else -1
        while cur[1] != dc:
            out.append("Right" if dc > cur[1] else "Left")
            cur[1] += 1 if dc > cur[1] else -1
        out.append("Return")
    return out


def setup(game, n):
    if n % 8:
        return ""
    x = r.random()
    if x < 0.12:
        return r.choice(["1", "2", "3", "4", "5", "6"]) if MODUS == "single" else "Down"
    if x < 0.2:
        return "c" if MODUS == "single" else "Right"
    if x < 0.45:
        return r.choice(["Up", "Down", "Left", "Right", "Right"])
    if x < 0.55:
        rid, rects = r.choice(game.setup_rows)
        return klick(r.choice(rects))
    if x < 0.8:
        return r.choice(["Return", klick(game.start_rect)])
    return ""


def partie(game, n):
    if game.promo is not None:
        if r.random() < 0.5:
            return r.choice(["q", "n", "r", "b", "1", "4"])
        return klick(r.choice(game._promo_rects()))
    if game.dialog is not None:
        if n % 15:
            return ""
        _, yes, no = game._dialog_rects()
        return r.choice(["y", "n", "Return", "Escape", klick(yes), klick(no)])
    if plan:
        return plan.pop(0)
    if n % 6:
        return ""
    if zustand["troedeln"]:
        return ""
    if not game._human_can_move():
        x = r.random()
        if x < 0.01:
            return "f"
        return ""
    x = r.random()
    if x < 0.03:
        return r.choice(["h", "@" + mitte(game.btn_rects["hint"])])
    if x < 0.05 and len(game.moves) > 2:
        return r.choice(["u", "BackSpace", "@" + mitte(game.btn_rects["undo"])])
    if x < 0.06:
        return r.choice(["f", "@" + mitte(game.btn_rects["flip"])])
    if x < 0.07 and len(game.moves) > 6:
        return r.choice(["o", "@" + mitte(game.btn_rects["draw"])])
    if x < 0.075 and len(game.moves) > 20:
        return r.choice(["x", "@" + mitte(game.btn_rects["resign"])])
    if x < 0.08:
        return "~" + mitte(r.choice(list(game.btn_rects.values())))
    if TROEDEL and game.clock and x < 0.09:
        zustand["troedeln"] = True
        return ""
    m = zug_waehlen(game)
    if m:
        plan.extend(zug_plan(game, m))
        return plan.pop(0)
    return ""


def ende(game, n):
    zustand["troedeln"] = False
    if n % 30:
        return ""
    x = r.random()
    if x < 0.15:
        return "~" + mitte(r.choice(list(game.over_btns.values())))
    if x < 0.55:
        return r.choice(["Return", klick(game.over_btns["new"])])
    if x < 0.75:
        return r.choice(["s", klick(game.over_btns["setup"])])
    if x < 0.85:
        return r.choice(["Up", "Down", "f"])
    return ""


def raetsel_menue(game, n):
    if n % 10:
        return ""
    x = r.random()
    if x < 0.15:
        return r.choice(["1", "2", "3", "4", "5", "Tab", "Prior"])
    if x < 0.3:
        return r.choice(["Left", "Right", "Up", "Down"])
    if x < 0.4:
        return klick(r.choice(game.stage_rects))
    if x < 0.5:
        return "~" + mitte(r.choice(game.grid_rects[:8]))
    if x < 0.75:
        return r.choice(["Return", klick(game.puz_start_rect)])
    return klick(r.choice(game.grid_rects[:10]))


def raetsel(game, n):
    if game.promo is not None:
        return r.choice(["q", "1"])
    if plan:
        return plan.pop(0)
    if n % 8:
        return ""
    st = game.puz_status
    x = r.random()
    if st in ("solved", "shown_done"):
        return r.choice(["Return", "n", "@" + mitte(game.puz_btn_rects["next"]), "m"]) if x < 0.6 else ""
    if st != "play" or not game._human_can_move():
        return ""
    if x < 0.06:
        return r.choice(["h", "@" + mitte(game.puz_btn_rects["solution"])])
    if x < 0.09:
        return r.choice(["u", "@" + mitte(game.puz_btn_rects["retry"])])
    if x < 0.11:
        return r.choice(["m", "@" + mitte(game.puz_btn_rects["list"])])
    if x < 0.13:
        return "~" + mitte(r.choice(list(game.puz_btn_rects.values())))
    if x < 0.3:
        m = r.choice(game.legal)
    else:
        m = game.pos.parse_uci(game.puz_line[game.puz_step])
    if m:
        plan.extend(zug_plan(game, m))
        return plan.pop(0)
    return ""


VORLAUF = os.environ.get("CH_VORLAUF", "").split()


def bot(game, k, IE):
    n = len(spur)
    if n < len(VORLAUF):
        spur.append(VORLAUF[n])
        orig(game, VORLAUF[n], IE)
        return
    st = game.state
    if st != besucht.get("_l"):
        wechsel.append((n, st))
        if st == "over":
            enden.append((n, game.result, len(game.moves)))
        plan.clear()
    besucht["_l"] = st
    besucht[st] = besucht.get(st, 0) + 1
    if st == "setup":
        taste = setup(game, n)
    elif st == "play":
        taste = partie(game, n)
    elif st == "over":
        taste = ende(game, n)
    elif st == "puz_menu":
        taste = raetsel_menue(game, n)
    else:
        taste = raetsel(game, n)
        if game.puz_status != besucht.get("_p"):
            besucht["_p"] = game.puz_status
            besucht["p_" + game.puz_status] = besucht.get("p_" + game.puz_status, 0) + 1
    spur.append(taste)
    if taste:
        orig(game, taste, IE)


vergleich.ereignis = bot
d = vergleich.datenordner(sp, "bot_probe")
vergleich.python_lauf("chess", MODUS, int(os.environ.get("CH_SEED", "7")), frames, ["x"] * frames, d,
                      os.path.join(sp, "bot.png"))
open(os.path.join(sp, "ch_keys.txt"), "w").write(",".join(spur))
print("Ende", {k: v for k, v in besucht.items() if not k.startswith("_")})
print("Wechsel", wechsel[:40])
print("Enden", enden)
