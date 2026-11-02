# -*- coding: utf-8 -*-
"""
PyGameZ selbst, Bild fuer Bild, auf beiden Engines.

Das ist die Endabnahme, die im Plan stand ("PyGameZ zweimal starten -
einmal mit echtem pygame, einmal mit rustgame - und die Bilder
vergleichen") und siebzehn Runden lang nicht gefahren wurde: gefahren
wurde stattdessen `vergleich_spiel.py`, ein Testspiel von 180 Zeilen.
Hier laeuft **jedes** Spiel aus `games.ALL_GAMES`, jedes in einem eigenen
Prozess je Engine, mit fester Uhr, festem Zufall und demselben
Eingabeskript, und jedes Bild wird als SHA-256 verglichen.

Aufruf (aus dem Projektordner `pygamez`)::

    .venv/Scripts/python.exe rustgame/tools/vergleich_pygamez.py
    .venv/Scripts/python.exe rustgame/tools/vergleich_pygamez.py --bilder 600 PongGame
    .venv/Scripts/python.exe rustgame/tools/vergleich_pygamez.py --text echt
    .venv/Scripts/python.exe rustgame/tools/vergleich_pygamez.py --zweimal

Endet mit 1, sobald ein Bild abweicht, ein Spiel abbricht oder der
Waechter anschlaegt.

Der Text
--------
Mit `--text ersatz` (die Vorgabe) bekommen **beide** Engines dieselbe
Ersatzschrift aus Rechtecken: rustgame rastert Glyphen heute mit fontdue,
pygame mit FreeType samt Hinting - die benannte Grenze
`font.metrics glyphenbox`. Ohne Ersatz waere **jedes** Bild verschieden
und jeder andere Fehler darunter begraben. `--text echt` zeigt, wie viele
Bilder dann abweichen.

Was das Messgeraet selbst festhaelt - es hat dreimal danebengelegen
---------------------------------------------------------------------
1. **Acht Spiele bauen `random.Random()` ohne Saat**, das zieht aus
   `os.urandom`. Der erste Lauf meldete zehn "abweichende" Spiele; acht
   davon waren auf **derselben** Engine nicht wiederholbar. Jetzt bekommt
   jeder ungesaete Generator eine feste Saat, dazu `PYTHONHASHSEED=0`.
   `--zweimal` prueft das: es faehrt pygame zweimal und vergleicht.
2. **Die Uhr.** `time.time`, `perf_counter`, `monotonic` und
   `pygame.time.get_ticks` gehen nur mit den Bildern weiter.
3. **Die Nutzerdaten.** PyGameZ schreibt Statistik ueber
   `store.path_for` in `store._DIR`; der Datenordner wird umgeleitet,
   bevor ein Spiel geladen ist.

Und der Waechter: jedes Spiel muss alle Bilder liefern, und sein Bild
muss sich bewegen (mindestens zwei verschiedene Bilder) - sonst ist
"gleich" nichts wert.
"""

import concurrent.futures as cf
import hashlib
import json
import math
import os
import subprocess
import sys
import tempfile
import time
import traceback
import zlib

HIER = os.path.dirname(os.path.abspath(__file__))
RUSTGAME = os.path.dirname(HIER)
ROOT = os.path.dirname(RUSTGAME)
TOOLS_PY = os.path.join(RUSTGAME, "crates", "rustgame-py", "tools")
AUSGABE = os.path.join(ROOT, "rustgame.output", "vergleich_pygamez")
PY = sys.executable

W, H = 640, 480
DT = 1.0 / 60.0


# ---------------------------------------------------------------------------
#  Arbeiter - ein Spiel, eine Engine, ein Prozess
# ---------------------------------------------------------------------------

def engine_laden(engine):
    """Holt pygame oder legt rustgame unter dem Namen `pygame` ab."""
    for k in ("SDL_VIDEODRIVER", "RUSTGAME_VIDEODRIVER", "SDL_AUDIODRIVER",
              "RUSTGAME_AUDIODRIVER"):
        os.environ[k] = "dummy"
    os.environ["PYGAME_HIDE_SUPPORT_PROMPT"] = "1"
    if engine == "rustgame":
        sys.path.insert(0, TOOLS_PY)
        import pygame_tests
        pygame_tests.umbiegen()
        sys.path.remove(TOOLS_PY)
    import pygame
    if ("rustgame" in repr(pygame)) != (engine == "rustgame"):
        raise RuntimeError("falsche Engine geladen: %r" % pygame)
    return pygame


def ersatzschrift(pygame):
    """Eine Schrift aus Rechtecken - auf beiden Engines Byte fuer Byte gleich.

    PyGameZ holt Schriften nur ueber `SysFont` und braucht `render`,
    `size` und `get_height`; die anderen Methoden sind der Vollstaendigkeit
    halber da.
    """

    class Ersatz:
        def __init__(self, name=None, size=12, bold=False, italic=False):
            self.groesse = max(4, int(size))
            self.h = self.groesse + self.groesse // 4
            self.cw = max(2, self.groesse // 2)
            self.bold, self.italic = bool(bold), bool(italic)
            self.underline = self.strikethrough = False

        def size(self, text):
            return (len(str(text)) * self.cw, self.h)

        def render(self, text, aa, color, bg=None, *rest):
            text = str(text)
            s = pygame.Surface((max(1, len(text) * self.cw), self.h), pygame.SRCALPHA)
            if bg is not None:
                s.fill(bg)
            for i, ch in enumerate(text):
                if ch != " ":
                    hh = 2 + (ord(ch) * 7) % max(1, self.h - 2)
                    pygame.draw.rect(s, color, (i * self.cw, self.h - hh, self.cw - 1, hh))
            return s

        def get_height(self):
            return self.h

        def get_linesize(self):
            return self.h + 1

        def get_ascent(self):
            return self.h * 4 // 5

        def get_descent(self):
            return self.h * 4 // 5 - self.h

        def set_bold(self, v):
            self.bold = bool(v)

        def get_bold(self):
            return self.bold

        def set_italic(self, v):
            self.italic = bool(v)

        def get_italic(self):
            return self.italic

        def set_underline(self, v):
            self.underline = bool(v)

        def get_underline(self):
            return self.underline

        def metrics(self, text):
            return [(0, self.cw, 0, self.h, self.cw) for _ in str(text)]

    pygame.font.SysFont = lambda name, size, bold=False, italic=False: Ersatz(
        name, size, bold, italic)
    pygame.font.Font = Ersatz


def eingaben(nr, InputEvent):
    """Das feste Eingabeskript - fuer jedes Spiel dasselbe."""
    E = InputEvent
    aus = []
    x = int(W / 2 + (W / 2 - 20) * math.sin(nr * 0.051))
    y = int(H / 2 + (H / 2 - 20) * math.sin(nr * 0.037 + 1.0))
    aus.append(E(E.MOUSEMOVE, pos=(x, y)))
    m = nr % 60
    if m == 10:
        aus.append(E(E.KEYDOWN, key="Return", char="\r"))
    elif m == 12:
        aus.append(E(E.KEYUP, key="Return"))
    elif m == 25:
        aus.append(E(E.KEYDOWN, key="space", char=" "))
    elif m == 28:
        aus.append(E(E.KEYUP, key="space"))
    pfeil = ("Left", "Up", "Right", "Down")[(nr // 30) % 4]
    if nr % 30 == 5:
        aus.append(E(E.KEYDOWN, key=pfeil))
    elif nr % 30 == 14:
        aus.append(E(E.KEYUP, key=pfeil))
    if nr % 45 == 20:
        aus.append(E(E.MOUSEDOWN, pos=(x, y), button=1))
    elif nr % 45 == 23:
        aus.append(E(E.MOUSEUP, pos=(x, y), button=1))
    if nr % 90 == 40:
        aus.append(E(E.KEYDOWN, key="a", char="a"))
    elif nr % 90 == 44:
        aus.append(E(E.KEYUP, key="a"))
    return aus


def arbeiter(engine, spielname, bilder, ziel, abzuege, text):
    ziel = os.path.abspath(ziel)
    pygame = engine_laden(engine)

    # Die Uhr geht nur mit den Bildern weiter.
    stand = {"bild": 0}
    import time as _time
    import random
    _time.time = _time.perf_counter = _time.monotonic = (
        lambda: 1000.0 + stand["bild"] * DT)
    pygame.time.get_ticks = lambda: int(stand["bild"] * 1000 / 60)

    # Jeder ungesaete Generator bekommt eine feste Saat, in der Reihenfolge,
    # in der er entsteht (acht Spiele bauen `random.Random()` ohne Saat).
    echt = random.Random

    class FesterZufall(echt):
        zaehler = 0

        def __init__(self, x=None):
            if x is None:
                FesterZufall.zaehler += 1
                x = 18000 + FesterZufall.zaehler
            super().__init__(x)

    random.Random = FesterZufall

    pygame.init()
    pygame.display.set_mode((1, 1))
    if text == "ersatz":
        ersatzschrift(pygame)
    os.chdir(ROOT)
    sys.path.insert(0, ROOT)

    # Die Nutzerdaten bleiben unberuehrt: der ganze Datenordner wird
    # umgeleitet, bevor ein Spiel ihn sieht.
    tmp = tempfile.mkdtemp(prefix="pgz-vergleich-")
    import store
    store._DIR = tmp
    store._PATH = os.path.join(tmp, "mem.json")
    import settings as settings_mod
    settings_mod.save_settings = lambda s: None
    for name in ("ugc", "replay"):
        m = __import__(name)
        m._PATH = os.path.join(tmp, os.path.basename(m._PATH))
    import stats
    stats.flush = lambda force=True: None
    import i18n
    i18n.init()
    from game_base import InputEvent
    from games import ALL_GAMES

    klasse = [c for c in ALL_GAMES if c.__name__ == spielname][0]
    random.seed(18)
    gs = json.loads(json.dumps(settings_mod.DEFAULTS))
    modi = getattr(klasse, "MODES", None)
    modus = modi[0][0] if modi and isinstance(modi[0], (tuple, list)) else "single"
    canvas = pygame.Surface((W, H))
    zeilen, fehler = [], None
    try:
        spiel = klasse(canvas, W, H, mode=modus, game_settings=gs)
        spiel.player_ids = [None, None]
        spiel.paused = False
        for nr in range(bilder):
            stand["bild"] = nr
            for ev in eingaben(nr, InputEvent):
                spiel.handle_event(ev)
            if not spiel.paused and not spiel.game_over:
                spiel.update(DT)
            spiel.draw()
            roh = pygame.image.tobytes(canvas, "RGB")
            zeilen.append("%d %s" % (nr, hashlib.sha256(roh).hexdigest()))
            if nr in abzuege:
                with open("%s.%d.z" % (ziel, nr), "wb") as f:
                    f.write(zlib.compress(roh, 6))
    except BaseException:  # noqa: BLE001 - auch ein Abbruch ist ein Ergebnis
        fehler = traceback.format_exc()
    with open(ziel, "w", encoding="utf-8") as f:
        f.write("\n".join(zeilen) + "\n")
        if fehler:
            f.write("FEHLER bei Bild %d\n%s" % (len(zeilen), fehler))


# ---------------------------------------------------------------------------
#  Leitung
# ---------------------------------------------------------------------------

def spielnamen():
    code = ("import os,sys;os.environ['SDL_VIDEODRIVER']='dummy';"
            "os.environ['PYGAME_HIDE_SUPPORT_PROMPT']='1';"
            "sys.path.insert(0,%r);os.chdir(%r);from games import ALL_GAMES;"
            "print(' '.join(c.__name__ for c in ALL_GAMES))" % (ROOT, ROOT))
    r = subprocess.run([PY, "-c", code], capture_output=True, text=True)
    namen = r.stdout.split()
    if not namen:
        raise RuntimeError("keine Spiele gefunden:\n" + r.stderr)
    return namen


def fahre(engine, name, bilder, ordner, text, abzuege="", nr=""):
    env = dict(os.environ, PYTHONIOENCODING="utf-8", PYTHONHASHSEED="0")
    ziel = os.path.join(ordner, "%s.%s%s" % (name, engine, nr))
    subprocess.run([PY, "-W", "ignore", os.path.abspath(__file__), "--arbeiter",
                    engine, name, str(bilder), ziel, abzuege, text],
                   env=env, capture_output=True, timeout=3600)
    return ziel


def lies(pfad):
    try:
        text = open(pfad, encoding="utf-8").read()
    except OSError:
        return [], "keine Ausgabe"
    teile = text.split("FEHLER bei Bild", 1)
    hashes = [z.split()[1] for z in teile[0].split("\n") if z.strip()]
    return hashes, ("FEHLER bei Bild" + teile[1]) if len(teile) > 1 else ""


def unterschied(a, b):
    """Zahl der verschiedenen Pixel, groesster Abstand, Kasten."""
    n, maxd = 0, 0
    x0, y0, x1, y1 = W, H, -1, -1
    for i in range(0, len(a), 3):
        if a[i:i + 3] != b[i:i + 3]:
            d = max(abs(a[i + k] - b[i + k]) for k in range(3))
            n += 1
            maxd = max(maxd, d)
            p = i // 3
            x, y = p % W, p // W
            x0, y0, x1, y1 = min(x0, x), min(y0, y), max(x1, x), max(y1, y)
    return n, maxd, (x0, y0, x1, y1)


def ein_spiel(name, bilder, ordner, text, zweimal):
    if zweimal:
        a = fahre("pygame", name, bilder, ordner, text, nr=".1")
        b = fahre("pygame", name, bilder, ordner, text, nr=".2")
    else:
        a = fahre("pygame", name, bilder, ordner, text)
        b = fahre("rustgame", name, bilder, ordner, text)
    ha, fa = lies(a)
    hb, fb = lies(b)
    probleme = []
    if fa:
        probleme.append("A-Abbruch: " + fa.strip().splitlines()[-1][:100])
    if fb:
        probleme.append("B-Abbruch: " + fb.strip().splitlines()[-1][:100])
    if len(ha) != bilder or len(hb) != bilder:
        probleme.append("nur %d/%d Bilder" % (len(ha), len(hb)))
    if len(set(ha)) < 2:
        probleme.append("WAECHTER: das Bild bewegt sich nicht")
    gleich = sum(1 for x, y in zip(ha, hb) if x == y)
    erstes = next((i for i, (x, y) in enumerate(zip(ha, hb)) if x != y), None)
    zeile = "%-22s %3d/%d gleich" % (name, gleich, bilder)
    if erstes is not None and not zweimal:
        fahre("pygame", name, bilder, ordner, text, str(erstes))
        fahre("rustgame", name, bilder, ordner, text, str(erstes))
        try:
            pa = zlib.decompress(open("%s.%d.z" % (a, erstes), "rb").read())
            pb = zlib.decompress(open("%s.%d.z" % (b, erstes), "rb").read())
            n, maxd, kasten = unterschied(pa, pb)
            zeile += "  erstes Bild %d: %d Pixel, max %d, Kasten %s" % (erstes, n, maxd, kasten)
        except OSError as e:
            zeile += "  (Abzug fehlt: %s)" % e
    elif erstes is not None:
        zeile += "  NICHT WIEDERHOLBAR ab Bild %d" % erstes
    ok = not probleme and erstes is None
    return ok, zeile + ("  " + "; ".join(probleme) if probleme else "")


def main():
    args = sys.argv[1:]
    if args and args[0] == "--arbeiter":
        engine, name, bilder, ziel, abzuege, text = args[1:7]
        arbeiter(engine, name, int(bilder), ziel,
                 {int(x) for x in abzuege.split(",") if x}, text)
        return 0

    bilder, text, zweimal, namen = 300, "ersatz", False, []
    i = 0
    while i < len(args):
        if args[i] == "--bilder":
            bilder = int(args[i + 1])
            i += 2
        elif args[i] == "--text":
            text = args[i + 1]
            i += 2
        elif args[i] == "--zweimal":
            zweimal = True
            i += 1
        else:
            namen.append(args[i])
            i += 1
    if text not in ("ersatz", "echt"):
        raise SystemExit("--text ersatz|echt")
    namen = namen or spielnamen()
    ordner = os.path.join(AUSGABE, "%s_%s" % (text, "zweimal" if zweimal else "beide"))
    os.makedirs(ordner, exist_ok=True)
    t0 = time.time()
    print("%d Spiele, %d Bilder, Text: %s%s" % (
        len(namen), bilder, text, ", pygame gegen pygame" if zweimal else ""), flush=True)
    schlecht = 0
    with cf.ThreadPoolExecutor(max_workers=6) as ex:
        for ok, zeile in ex.map(lambda n: ein_spiel(n, bilder, ordner, text, zweimal), namen):
            schlecht += not ok
            print(("   " if ok else "!! ") + zeile, flush=True)
    print("\n%d von %d Spielen Bild fuer Bild gleich (%d x %d Bilder, %.0f s)" % (
        len(namen) - schlecht, len(namen), len(namen), bilder, time.time() - t0))
    return 1 if schlecht else 0


if __name__ == "__main__":
    sys.exit(main())
