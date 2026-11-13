"""Bot fuer Geometry Dash.

GD_MODE=spielen: waehlt in der Auswahl ein Level (Ziffern, Pfeile, Punkte,
Karte, Normal/Uebung, Optionen), spielt es mit Vorausschau (eigene Kopie
des Zustands, 4 Physik-Schritte je Bild - eine Eingabe in Bild k wirkt ab
dem ersten Schritt von Bild k+1), drueckt mal Leertaste, mal Pfeil hoch,
mal die Maus; troedelt mit GD_TROEDEL (Uebung an/aus, Checkpoints Z/X,
Neustart R) und nutzt am Ziel Nochmal/Weiter/Auswahl.

GD_MODE=edit: LEVELS-Reiter: Vorlagen-Auswahl, Editor (Palette, Setzen,
Ziehen, Rechtsklick-Loeschen, Drehen, Undo/Redo, Raster, Werte +/-,
Einstellungs-Dialog, Name/id tippen, Speichern), Test-Spielen ab Start und
ab Kamera (verifiziert ein Level), Loeschen mit Bestaetigung. Teilen und
Importieren werden NICHT gedrueckt (Python schriebe in den echten
Downloads-Ordner bzw. oeffnete einen Dateidialog).

GD_MODE=extra: WEITERE-Reiter: Kategorien, Suche tippen, Blaettern, Zufall,
Spielen, als Vorlage kopieren (fuehrt in den Editor).

Aufruf: bot_geodash.py <ordner> <frames>   (GD_SPIELMODUS=normal|practice)
Schreibt <ordner>/gd_keys.txt. Eigener Zufall (random.Random(GD_BOTSEED)).
"""
import os
import random
import sys
sys.path.insert(0, "rust/tools")
sys.path.insert(0, ".")
import vergleich

sp, frames = sys.argv[1], int(sys.argv[2])
spur = []
plan = []
r = random.Random(int(os.environ.get("GD_BOTSEED", "61")))
orig = vergleich.ereignis
MODUS = os.environ.get("GD_MODE", "spielen")
TROEDEL = float(os.environ.get("GD_TROEDEL") or "0.0")
H = int(os.environ.get("GD_H") or "32")
LEVELS = (os.environ.get("GD_LEVELS") or "1 2 3").split()
VORLAUF = os.environ.get("GD_VORLAUF", "").split()
besucht = {}
wechsel = []
ergebnisse = []
core = None

PRESS = ["+space", "+space", "+space", "+Up", "<320:240", "+w"]
RELEASE = {"k:space": "-space", "k:up": "-Up", "k:w": "-w", "mouse": ">320:240",
           "k:return": "-Return", "k:kp_enter": "-KP_Enter"}


def pt(x, y):
    return "%d:%d" % (int(x), int(y))


def mitte(rc):
    return pt(*rc.center)


# ---------------------------------------------------------------- Spielen
def sim(s, lv, plan_fn, prev, frames_):
    s = s.copy()
    n = 0
    for f in range(frames_):
        h = plan_fn(f)
        for st in range(4):
            p = h and not prev and st == 0
            core.step(s, lv, h or p, p)
            n += 1
            if s.dead:
                return n
            if s.won:
                return 10 ** 6
        prev = h
    return n


def ki(game):
    """Haltezustand fuer das naechste Bild: (soll_halten, jetzt_gehalten, quellen)."""
    lv = game.lv
    srcs = set(game.sources)
    pressed0 = False
    for te, down, src in sorted(game.queue, key=lambda e: e[0]):
        if down:
            if src not in srcs:
                srcs.add(src)
                pressed0 = True
        else:
            srcs.discard(src)
    held0 = bool(srcs)
    s = game.run.copy()
    for st in range(4):
        p = pressed0 and st == 0
        core.step(s, lv, held0 or p, p)
        if s.dead or s.won:
            return held0, held0, srcs
    score = {}
    for d in (held0, not held0):
        best = sim(s, lv, lambda f, d=d: d, held0, H)
        for j in range(1, H, 2):
            if best >= 10 ** 6:
                break
            best = max(best, sim(s, lv, lambda f, d=d, j=j: d if f < j else (not d), held0, H))
        score[d] = best
    want = (not held0) if score[not held0] > score[held0] else held0
    if os.environ.get("GD_DEBUG") and len(spur) < int(os.environ["GD_DEBUG"]) and want != held0:
        print(len(spur), s.x / 36000.0, score, want, held0, srcs, game.queue)
    return want, held0, srcs


def spielen(game, n):
    run = game.run
    if run is None:
        return ""
    if run.dead:
        if game.sources and n % 7 == 0:
            return RELEASE.get(sorted(game.sources)[0], "-space")
        return ""
    if game.queue:
        return ""
    if TROEDEL and r.random() < TROEDEL:
        x = r.random()
        if x < 0.3:
            return "z"
        if x < 0.4:
            return "x"
        if x < 0.55:
            return "p"
        if x < 0.65:
            return "r"
        if x < 0.7 and game._test is not None:
            return "Escape"
        if x < 0.72:
            return "q" if game._test is None else "BackSpace"
    want, held0, srcs = ki(game)
    if want == held0:
        return ""
    if want:
        return r.choice(PRESS)
    return RELEASE.get(sorted(srcs)[0], "-space") if srcs else ""


def fertig(game, n):
    if game.complete_t < 0.4 or n % 12:
        return ""
    _panel, rects = game._complete_buttons()
    x = r.random()
    if MODUS == "edit" or game.cur_kind == "test":
        return r.choice(["Escape", "Escape", "Return", "@" + mitte(rects[-1][1])])
    if x < 0.35:
        return r.choice(["n", "Right", "@" + mitte(rects[-1][1])])
    if x < 0.6:
        return r.choice(["Return", "space", "@" + mitte(rects[0][1])])
    return r.choice(["Escape", "BackSpace", "@" + mitte(rects[1][1])])


# ---------------------------------------------------------------- Auswahl
def auswahl(game, n):
    if n % 9:
        return ""
    if MODUS == "edit":
        if game.setup_tab != "levels":
            return r.choice(["Tab", "@" + mitte(game.tab_rects[2])])
        return liste(game, n)
    if MODUS == "extra":
        if game.setup_tab != "extra":
            return r.choice(["Tab", "ISO_Left_Tab", "@" + mitte(game.tab_rects[1])])
        return weitere(game, n)
    if game.setup_tab != "play":
        return r.choice(["Escape", "@" + mitte(game.tab_rects[0])])
    x = r.random()
    if x < 0.25:
        return r.choice(LEVELS)
    if x < 0.35:
        return r.choice(["Left", "Right", "Home", "@" + mitte(game.arrow_rects[0]), "@" + mitte(game.arrow_rects[1])])
    if x < 0.42:
        rects = game._dot_rects()
        return "@" + mitte(r.choice(rects)) if rects else ""
    if x < 0.5:
        return r.choice(["p", "@" + mitte(game.mode_rects[0]), "@" + mitte(game.mode_rects[1])])
    if x < 0.55:
        return "@" + mitte(game.opt_rects[r.choice(["bar", "auto", "music"])])
    if game.levels[game.sel]["id"] not in [game.levels[int(k) - 1]["id"] for k in LEVELS if k.isdigit()]:
        return r.choice(LEVELS)
    return r.choice(["Return", "space", "@" + mitte(game.start_rect), "@" + mitte(game.card_rect)])


def liste(game, n):
    ll = game.lists
    if ll is None:
        return ""
    if ll.picker is not None:
        _box, rects = ll._pick_rects()
        x = r.random()
        if x < 0.5:
            return "@" + mitte(r.choice(rects))
        return r.choice(["Right", "Down", "Left", "Return", "Return"])
    if ll.share is not None:
        return r.choice(["Escape", "@" + mitte(ll.share_btn["cancel"])])
    x = r.random()
    if not ll.items or x < 0.3:
        return r.choice(["n", "@" + mitte(ll.btn_rects["new"])])
    if x < 0.45:
        return r.choice(["Up", "Down"])
    if x < 0.65:
        return r.choice(["e", "@" + mitte(ll.btn_rects["edit"])])
    if x < 0.8:
        return r.choice(["Return", "@" + mitte(ll.btn_rects["play"])])
    if x < 0.9:
        return r.choice(["Delete", "@" + mitte(ll.btn_rects["delete"])])
    if x < 0.95:
        rc = ll._row_rect(0)
        return "@" + mitte(rc)
    return ""


def weitere(game, n):
    xl = game.extra
    if xl is None:
        return ""
    if xl.search_focus:
        x = r.random()
        if len(xl.query.text) < 2 and x < 0.6:
            return r.choice(list("aeinorstwx"))
        if xl.query.text and x < 0.8:
            return "BackSpace"
        return r.choice(["Return", "Return", "Escape", "Up", "Down"])
    if not xl.view:
        if xl.query.text:
            return r.choice(["f", "@" + mitte(xl.search_rect)])
        return "@" + mitte(xl.cat_rects["all"])
    if xl.query.text and r.random() < 0.3:
        plan.extend(["BackSpace", "BackSpace", "Return"])
        return "f"
    if game._gs().get("last_cat") in (None, "all") and r.random() < 0.3:
        return "@" + mitte(xl.cat_rects[r.choice(["starter", "rhythm", "coins"])])
    if r.random() < 0.35:
        return r.choice(["Return", "@" + mitte(xl.btn_rects["play"])])
    x = r.random()
    if x < 0.2:
        key = r.choice(list(xl.cat_rects))
        return "@" + mitte(xl.cat_rects[key])
    if x < 0.27:
        return "c"
    if x < 0.33:
        return r.choice(["f", "slash", "@" + mitte(xl.search_rect)])
    if x < 0.5:
        return r.choice(["Up", "Down", "Prior", "Next", "Left", "Right"])
    if x < 0.62:
        return r.choice(["Return", "@" + mitte(xl.btn_rects["play"])])
    if x < 0.7:
        return r.choice(["r", "@" + mitte(xl.btn_rects["random"])])
    if x < 0.76:
        return r.choice(["t", "@" + mitte(xl.btn_rects["template"])])
    if x < 0.85:
        rc = xl._row_rect(r.randrange(max(1, min(xl.rows_visible, len(xl.view)))))
        return "@" + mitte(rc)
    return ""


# ---------------------------------------------------------------- Editor
def editor(game, n):
    ed = game.editor
    if n % 6:
        return ""
    if ed.settings_open:
        x = r.random()
        if x < 0.75:
            key = r.choice(list(ed.set_rows))
            return "@" + mitte(r.choice(ed.set_rows[key]))
        return r.choice(["Escape", "Return", "@" + mitte(ed.set_close)])
    if ed.focus >= 0:
        x = r.random()
        if x < 0.75:
            return r.choice(list("lamastufe") + ["space", "BackSpace", "Left", "Right", "Home", "End", "minus"])
        return r.choice(["Tab", "Return", "Escape"])
    c = ed.canvas
    x = r.random()
    if x < 0.06:
        return "@" + mitte(r.choice([ed.name_rect, ed.id_rect]))
    if x < 0.18:
        key, rc = r.choice(ed.tool_rects + ed.group_rects + ed.item_rects() * 2)
        return "@" + mitte(rc)
    if x < 0.21:
        return r.choice(["1", "2", "3", "4", "5", "6"])
    if x < 0.5:
        x0 = r.randint(c.x + 4, c.right - 4)
        y0 = r.randint(c.y + 4, c.bottom - 4)
        y0 = max(c.y + 4, min(c.bottom - 4, int(c.bottom - (r.choice([0.5, 0.5, 1.5, 2.5, 4.5, r.uniform(0, 10)]) + 1 + ed.cam_y) * ed.ts)))
        if r.random() < 0.4:
            x1 = min(c.right - 2, x0 + r.randint(10, 90))
            plan.extend(["~" + pt(x0 + 5, y0), "~" + pt(x1, y0), ">" + pt(x1, y0)])
            return "<" + pt(x0, y0)
        return "@" + pt(x0, y0)
    if x < 0.55:
        x0 = r.randint(c.x + 10, c.right - 10)
        y0 = r.randint(c.y + 10, c.bottom - 10)
        if r.random() < 0.5:
            return "@%s:3" % pt(x0, y0)
        plan.extend(["~" + pt(x0 - 30, y0 + 5), ">%s:3" % pt(x0 - 30, y0 + 5)])
        return "<%s:3" % pt(x0, y0)
    if x < 0.62:
        rects = ed._param_rects()
        if rects:
            i, minus, val, plus = r.choice(rects)
            return "@" + mitte(r.choice([minus, plus]))
        return r.choice(["r", "@" + mitte(ed.edit_rects["rotate"])])
    if x < 0.7:
        return r.choice(["Left", "Right", "a", "d", "Up", "Down", "w", "Home", "End", "@" + pt(ed.map_rect.x + r.randint(0, ed.map_rect.w), ed.map_rect.centery)])
    if x < 0.77:
        return r.choice(["u", "y", "g", "r", "Delete", "@" + mitte(ed.edit_rects["undo"]), "@" + mitte(ed.edit_rects["redo"]),
                         "@" + mitte(ed.edit_rects["grid"]), "@" + mitte(ed.edit_rects["delete"])])
    if x < 0.8:
        return "@" + mitte(ed.edit_rects["settings"])
    if x < 0.87:
        return r.choice(["s", "@" + mitte(ed.save_rect)])
    if x < 0.94:
        return r.choice(["Return", "t", "@" + mitte(ed.edit_rects["test"]), "@" + mitte(ed.edit_rects["test_here"])])
    return r.choice(["Escape", "@" + mitte(ed.back_rect)])


def bot(game, k, IE):
    global core
    if core is None:
        core = sys.modules[type(game).__module__.rsplit(".", 1)[0] + ".geodash_core"]
    n = len(spur)
    if n < len(VORLAUF):
        spur.append(VORLAUF[n])
        orig(game, VORLAUF[n], IE)
        return
    st = game.state
    zst = st + (":" + game.setup_tab if st == "select" else "")
    if zst != besucht.get("_l"):
        wechsel.append((n, zst))
        plan.clear()
        if st == "complete":
            ergebnisse.append((n, game.cur_kind, game.cur.get("id"), game.practice, game.session_attempts))
    besucht["_l"] = zst
    besucht[zst] = besucht.get(zst, 0) + 1
    if plan:
        taste = plan.pop(0)
    elif st == "select":
        taste = auswahl(game, n)
    elif st == "edit":
        taste = editor(game, n)
    elif st == "complete":
        taste = fertig(game, n)
    else:
        taste = spielen(game, n)
    spur.append(taste)
    if taste:
        orig(game, taste, IE)


vergleich.ereignis = bot
d = vergleich.datenordner(sp, "bot_probe")
vergleich.python_lauf("geodash", os.environ.get("GD_SPIELMODUS") or "normal", int(os.environ.get("GD_SEED", "7")),
                      frames, ["x"] * frames, d, os.path.join(sp, "bot.png"))
open(os.path.join(sp, "gd_keys.txt"), "w").write(",".join(spur))
print("Ende", {k: v for k, v in besucht.items() if not k.startswith("_")})
print("Wechsel", wechsel[:60])
print("Ergebnisse", ergebnisse)
