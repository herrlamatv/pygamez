# -*- coding: utf-8 -*-
"""Headless-Audit für die Wiederholungen (replay.py / replayview.py).

Geprüft wird:

Grundlage (1) replay.py kennt alle sechs Spiele, jedes hat eine Spielklasse,
              einen Vor-/Nachlauf (replayview.PAD) und eine Beschriftung
              (replayview.SEQ_KEYS),
          (2) alle neuen Sprach-Schlüssel stehen in allen 14 Sprachdateien,
          (3) Teilen: export_to schreibt eine .lamapgzreplay-Datei, read_file
              liest sie zurück, import_from legt sie ins Archiv - und weist
              Fremddateien, doppelte Aufnahmen und ein volles Archiv ab.
Billard   (4) eine Partie zeichnet je Stoß eine Sequenz auf; die Wiedergabe
              baut denselben Tischstand wieder auf (Kugeln auf 0,1 genau),
Pinball   (5) eine Partie mit drei Bällen ergibt drei Sequenzen; die
              Wiedergabe trifft Punktstand und Ballzahl,
Snake     (6) ein Lauf bis zum Tod ergibt Kapitel; die Wiedergabe trifft
              Körper, Punkte und Äpfel exakt,
Tetris    (7) ein Ultra-Lauf ergibt Kapitel; die Wiedergabe trifft Feld,
              Punkte und Vorschau exakt,
Alle      (8) Vorwärts-, Rückwärts- und Zufallssprünge durch jede Aufnahme
              zeichnen fehlerfrei (wie im Replay-Screen).

Aufruf aus dem Repo-Root:  python tests/replay_audit.py
Exit-Code 0 = alle Prüfungen bestanden.
"""
import json
import os
import random
import sys
import tempfile

os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
try:
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
except (AttributeError, OSError):
    pass

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, REPO)

import pygame
pygame.init()
pygame.display.set_mode((960, 640))

import store
store._PATH = os.path.join(tempfile.gettempdir(), "_replay-audit-mem.json")
import settings as settings_mod
settings_mod.save_settings = lambda s: None
import i18n
i18n.init()

import replay
import replayview
from games import billiard as bil
from games import pinball as pin
from games import snake as snk
from games import tetris as tet

# Das Archiv des Tests liegt im Temp-Ordner - die echte replay.json bleibt
# unberührt.
replay._PATH = os.path.join(tempfile.gettempdir(), "_replay-audit.json")
if os.path.exists(replay._PATH):
    os.remove(replay._PATH)

GS = json.loads(json.dumps(settings_mod.DEFAULTS))
W, H = 960, 640
SURF = pygame.Surface((W, H))
FAILS = []
NL = chr(10)                     # Leerzeile vor jedem Abschnitt
MAIN_LANGS = ("de", "en", "fr", "es", "pt")
EXP_LANGS = ("pl", "tr", "da", "no", "sv", "fi", "cs", "sl", "hr")


def check(ok, label, detail=""):
    print("  %-4s %s%s" % ("OK" if ok else "FAIL", label,
                           ("  -> " + detail) if detail and not ok else ""))
    if not ok:
        FAILS.append(label)
    return ok


def quiet(game):
    game.play_sound = lambda name: None
    game.rumble = lambda ms=0: None
    game.ach_event = lambda *a, **k: None
    game.report_result = lambda won: None
    return game


def make(cls, mode="single"):
    return quiet(cls(SURF, W, H, mode=mode, game_settings=GS))


# ---------------------------------------------------------------- Grundlage

def audit_basics():
    print("\n[1] Grundlage: Spiele, Vor-/Nachlauf, Beschriftungen")
    check(len(replay.GAMES) == 6, "sechs Spiele mit Aufzeichnung",
          str(replay.GAMES))
    for key in replay.GAMES:
        cls = replayview._game_class(key)
        check(cls is not None, "Spielklasse für %s" % key)
        check(key in replayview.PAD, "PAD für %s" % key)
        check(key in replayview.SEQ_KEYS, "SEQ_KEYS für %s" % key)
        for hook in ("replay_begin", "replay_seek", "replay_draw"):
            check(hasattr(cls, hook), "%s.%s" % (key, hook))
    check(replay.EXT == ".lamapgzreplay", "Endung .lamapgzreplay")


def audit_langs():
    print("\n[2] Sprachdateien: neue Schlüssel in allen 14 Sprachen")
    keys = ["replay.seq_bil", "replay.seq_pin", "replay.seq_part",
            "replay.btn_export", "replay.btn_import", "replay.export_title",
            "replay.import_title", "replay.exported", "replay.export_error",
            "replay.imported", "replay.import_dup", "replay.import_full",
            "replay.import_game", "replay.import_error", "replay.no_dialog",
            "replay.capped", "bil.replay_hint", "bil.res_pot", "bil.res_miss",
            "pin.replay_hint", "tetris.res.replay", "snake.replay_hint",
            "snake.replay_sub", "ach.replay_share.name", "ach.replay_share.desc"]
    missing = []
    for code in MAIN_LANGS + EXP_LANGS:
        sub = "" if code in MAIN_LANGS else "lang.expansion"
        path = os.path.join(REPO, "lang", sub, code + ".json")
        with open(path, "r", encoding="utf-8") as f:
            data = json.load(f)
        missing += ["%s/%s" % (code, k) for k in keys if not data.get(k)]
    check(not missing, "alle %d Schlüssel in 14 Sprachen" % len(keys),
          ", ".join(missing[:6]))
    # Wiki-Seite "replays" muss in allen Sprachen die neue Teilen-Sektion haben
    fehlt = []
    for code in MAIN_LANGS + EXP_LANGS:
        sub = "" if code in MAIN_LANGS else "lang.expansion"
        path = os.path.join(REPO, "lamawiki", sub, code + ".json")
        with open(path, "r", encoding="utf-8") as f:
            pages = json.load(f)["pages"]
        page = next((p for p in pages if p.get("id") == "replays"), None)
        if page is None or len(page.get("sections", [])) < 5 \
                or ".lamapgzreplay" not in json.dumps(page, ensure_ascii=False):
            fehlt.append(code)
    check(not fehlt, "LamaWiki-Seite 'replays' überall aktualisiert",
          ",".join(fehlt))


def audit_share(rep):
    print("\n[3] Teilen: Export / Import (.lamapgzreplay)")
    path = os.path.join(tempfile.gettempdir(),
                        replay.default_filename(rep))
    if os.path.exists(path):
        os.remove(path)
    ok, why = replay.export_to(rep, path)
    check(ok and os.path.exists(path), "export_to schreibt die Datei", why)
    with open(path, "r", encoding="utf-8") as f:
        raw = json.load(f)
    check(raw.get("format") == replay.FORMAT, "Umschlag mit format-Kennung")
    check(raw.get("replay", {}).get("id") == rep["id"], "genau ein Replay drin")

    ok, why, back = replay.read_file(path)
    check(ok and back["id"] == rep["id"], "read_file liest zurück", why)

    ok, why, imported = replay.import_from(path)
    check(ok, "import_from legt ins Archiv", why)
    check(imported.get("src") == "file", "importiertes Replay ist markiert")
    check(replay.is_saved(rep["game"], imported["id"]), "steht im Archiv")

    ok, why, _ = replay.import_from(path)
    check(not ok and why == "dup", "zweiter Import wird abgewiesen", why)

    ok, why, _ = replay.import_from(path, game="minigolf")
    check(not ok and why == "game", "falscher Reiter wird abgewiesen", why)

    fremd = os.path.join(tempfile.gettempdir(), "fremd.lamapgzreplay")
    with open(fremd, "w", encoding="utf-8") as f:
        json.dump({"format": "pygamez.minigolf.map", "map": {"id": "x"}}, f)
    ok, why, _ = replay.import_from(fremd)
    check(not ok and why == "format", "Fremddatei wird abgewiesen", why)

    kaputt = os.path.join(tempfile.gettempdir(), "kaputt.lamapgzreplay")
    with open(kaputt, "w", encoding="utf-8") as f:
        f.write("{kein json")
    ok, why, _ = replay.import_from(kaputt)
    check(not ok and why == "io", "unlesbare Datei wird abgewiesen", why)

    # Volles Archiv: MAX_PER_GAME Kopien einlegen, dann muss "full" kommen.
    game = rep["game"]
    for i in range(replay.MAX_PER_GAME + 2):
        kopie = dict(rep)
        kopie["id"] = "%s-voll-%d" % (game, i)
        replay.save_replay(kopie)
    check(replay.is_full(game), "Archiv läuft nicht über (%d)"
          % replay.count(game))
    ok, why, _ = replay.import_from(path)
    check(not ok and why == "full", "Import bei vollem Archiv: 'full'", why)
    for r in list(replay.load_game(game)):
        replay.delete_replay(game, r["id"])
    check(replay.count(game) == 0, "Archiv wieder leer")
    for p in (path, fremd, kaputt):
        if os.path.exists(p):
            os.remove(p)


# ------------------------------------------------------- Wiedergabe-Fahrten

def playback(key, rep, label, final_check=None):
    """Fährt eine Aufnahme wie der Replay-Screen komplett durch."""
    cls = replayview._game_class(key)
    inst = make(cls)
    try:
        inst.replay_begin(rep)
    except Exception as exc:                       # noqa: BLE001
        return check(False, "%s: replay_begin" % label, repr(exc))
    scenes = rep["scenes"]
    try:
        for si, sc in enumerate(scenes):
            n = max(1, replay.scene_len(sc))
            for fi in range(n):
                inst.replay_seek(si, fi)
                inst.replay_draw(aiming=(fi == 0),
                                 banner=bool(sc.get("final")) and fi == n - 1)
    except Exception as exc:                       # noqa: BLE001
        return check(False, "%s: Wiedergabe vorwärts" % label, repr(exc))
    check(True, "%s: %d Sequenzen vorwärts gezeichnet" % (label, len(scenes)))

    # Rückwärts und zufällig springen (der Screen erlaubt beides).
    rnd = random.Random(7)
    try:
        for _ in range(40):
            si = rnd.randrange(len(scenes))
            n = max(1, replay.scene_len(scenes[si]))
            inst.replay_seek(si, rnd.randrange(n))
            inst.replay_draw()
        for si in range(len(scenes) - 1, -1, -1):
            n = max(1, replay.scene_len(scenes[si]))
            for fi in range(n - 1, -1, -1):
                inst.replay_seek(si, fi)
    except Exception as exc:                       # noqa: BLE001
        return check(False, "%s: Sprünge" % label, repr(exc))
    check(True, "%s: Rück- und Zufallssprünge" % label)

    if final_check is not None:
        si = len(scenes) - 1
        inst.replay_seek(si, max(0, replay.scene_len(scenes[si]) - 1))
        final_check(inst)
    return True


# ------------------------------------------------------------------ Billard

def audit_billiard():
    print("\n[4] Billard: Stöße aufzeichnen und wiedergeben")
    random.seed(4)
    g = make(bil.BilliardGame)
    g.variant = "practice"
    g._start_play()
    schuesse = 0
    for _ in range(6):
        if g.phase != "aim":
            break
        g.aim = random.uniform(-0.4, 0.4)
        g.power = 0.85
        g._strike()
        schuesse += 1
        for _ in range(1200):
            g.update(1 / 60.0)
            if g.phase != "rolling":
                break
    g._rec_snapshot()
    rep = g.replay
    if not check(rep is not None, "Aufnahme entstanden"):
        return
    check(len(rep["scenes"]) == schuesse,
          "je Stoß eine Sequenz (%d)" % len(rep["scenes"]))
    soll = [(round(b.x, 1), round(b.y, 1), b.potted) for b in g.balls]

    def ende(inst):
        ist = [(round(b.x, 1), round(b.y, 1), b.potted) for b in inst.balls]
        check(ist == soll, "Tischstand am Ende identisch",
              "%s != %s" % (ist[:3], soll[:3]))

    playback("billiard", rep, "Billard", ende)
    return rep


# ------------------------------------------------------------------ Pinball

def audit_pinball():
    print("\n[5] Pinball: Bälle aufzeichnen und wiedergeben")
    random.seed(11)
    g = make(pin.PinballGame)
    g.ball_count = 3
    g._start_play()
    for _ in range(40000):
        if g.state != pin.PLAY:
            break
        if g.phase == "launch":
            g.plunger = 1.0
            g._launch()
        if random.random() < 0.02:
            g.flip_l_up = not g.flip_l_up
        if random.random() < 0.02:
            g.flip_r_up = not g.flip_r_up
        g.update(1 / 60.0)
    rep = g.replay
    if not check(rep is not None, "Aufnahme entstanden"):
        return
    check(g.state == pin.OVER, "Partie endet von allein")
    check(len(rep["scenes"]) == 3, "drei Sequenzen (je Ball eine): %d"
          % len(rep["scenes"]))
    soll = g.scores[0]

    def ende(inst):
        check(inst.scores[0] == soll, "Punktstand am Ende identisch",
              "%s != %s" % (inst.scores[0], soll))

    playback("pinball", rep, "Pinball", ende)
    return rep


# -------------------------------------------------------------------- Snake

def audit_snake():
    print("\n[6] Snake: Lauf aufzeichnen und wiedergeben")
    random.seed(23)
    g = make(snk.SnakeGame)
    g.mode_index = snk.MODE_KEYS.index("walls")
    g.view3d = False
    g._start_play()
    g.REC_CHAPTER = 60            # Kapitel alle 2 s statt alle 10 s (Test)
    for i in range(40000):
        if g.game_over:
            break
        if i % 17 == 0:
            sn = g.snakes[0]
            sn.next_direction = snk._rotate(sn.direction,
                                            random.choice(("L", "R")))
        g.update(1 / 60.0)
    rep = g.replay
    if not check(rep is not None, "Aufnahme entstanden"):
        return
    check(g.game_over, "Lauf endet von allein")
    check(len(rep["scenes"]) >= 2, "mehrere Kapitel (%d)" % len(rep["scenes"]))
    check(all(sc.get("bodies") for sc in rep["scenes"]),
          "jedes Kapitel ist ein Schlüsselbild")
    soll_body = list(g.snakes[0].body)
    soll_score = g.snakes[0].score
    soll_apples = g.apples_total

    def ende(inst):
        check(list(inst.snakes[0].body) == soll_body,
              "Körper am Ende identisch (%d Blöcke)" % len(soll_body),
              "%d != %d" % (len(inst.snakes[0].body), len(soll_body)))
        check(inst.snakes[0].score == soll_score, "Punkte am Ende identisch")
        check(inst.apples_total == soll_apples, "Äpfel am Ende identisch")
        check(not inst.snakes[0].alive, "Schlange ist im letzten Bild tot")

    playback("snake", rep, "Snake", ende)
    return rep


# ------------------------------------------------------------------- Tetris

def audit_tetris():
    print("\n[7] Tetris: Ultra-Lauf aufzeichnen und wiedergeben")
    random.seed(5)
    g = make(tet.TetrisGame)
    g.variant = "ultra"
    g._start()
    g.REC_CHAPTER = 60            # Kapitel alle 2 s statt alle 10 s (Test)
    for i in range(40000):
        if g.state in (tet.FINISH, tet.OVER):
            break
        b = g.boards[0]
        if g.state == tet.PLAY and b.active and i % 11 == 0:
            if random.random() < 0.5:
                b.move(random.choice((-1, 1)))
            else:
                b.rotate(1)
        if g.state == tet.PLAY and i % 23 == 0 and b.active:
            g._hard_drop(0)
        g.update(1 / 60.0)
    rep = g.replay
    if not check(rep is not None, "Aufnahme entstanden"):
        return
    check(len(rep["scenes"]) >= 2, "mehrere Kapitel (%d)" % len(rep["scenes"]))
    check(all(sc.get("boards") for sc in rep["scenes"]),
          "jedes Kapitel ist ein Schlüsselbild")
    soll_rows = list(g.boards[0].rows)
    soll_score = g.boards[0].score
    soll_next = list(g.boards[0].queue.peek(5))

    def ende(inst):
        check(list(inst.boards[0].rows) == soll_rows, "Feld am Ende identisch")
        check(inst.boards[0].score == soll_score, "Punkte am Ende identisch")
        check(list(inst.boards[0].queue.peek(5)) == soll_next,
              "Vorschau am Ende identisch")

    playback("tetris", rep, "Tetris", ende)
    return rep


def audit_multi():
    """Mehrspieler: zwei Schlangen bzw. zwei Tetris-Felder in einer Aufnahme."""
    print(NL + "[8] Mehrspieler: Snake zu zweit und Tetris gegen die KI")
    random.seed(31)
    g = make(snk.SnakeGame, mode="multi")
    g._start_play()
    for i in range(40000):
        if g.game_over:
            break
        if i % 13 == 0:
            for sn in g.snakes:
                sn.next_direction = snk._rotate(sn.direction,
                                                random.choice(("L", "R")))
        g.update(1 / 60.0)
    rep = g.replay
    if check(rep is not None, "Snake (2 Spieler): Aufnahme entstanden"):
        check(rep["meta"].get("players") == 2, "zwei Schlangen in den Kopfdaten")
        soll = [list(sn.body) for sn in g.snakes]

        def ende(inst):
            check([list(sn.body) for sn in inst.snakes] == soll,
                  "beide Körper am Ende identisch")

        playback("snake", rep, "Snake (2 Spieler)", ende)

    random.seed(37)
    g = make(tet.TetrisGame, mode="versus_ai")
    g._start()
    for i in range(60000):
        if g.state in (tet.FINISH, tet.OVER):
            break
        b0 = g.boards[0]
        if g.state == tet.PLAY and b0.active and i % 19 == 0:
            g._hard_drop(0)
        g.update(1 / 60.0)
    rep = g.replay
    if check(rep is not None, "Tetris (Versus): Aufnahme entstanden"):
        check(rep["meta"].get("boards") == 2, "zwei Felder in den Kopfdaten")
        soll = [list(b.rows) for b in g.boards]

        def ende(inst):
            check([list(b.rows) for b in inst.boards] == soll,
                  "beide Felder am Ende identisch")

        playback("tetris", rep, "Tetris (Versus)", ende)


def main():
    audit_basics()
    audit_langs()
    reps = [audit_billiard(), audit_pinball(), audit_snake(), audit_tetris()]
    audit_multi()
    reps = [r for r in reps if r]
    if reps:
        audit_share(reps[0])
    print("\n" + "=" * 62)
    if FAILS:
        print("FEHLGESCHLAGEN (%d): %s" % (len(FAILS), ", ".join(FAILS[:10])))
        return 1
    print("Alle Prüfungen bestanden.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
