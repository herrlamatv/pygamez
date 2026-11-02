# -*- coding: utf-8 -*-
"""Startet ein **unveraendertes** pygame-Programm auf rustgame.

    python rustgame/tools/starten.py mein_spiel.py [argumente ...]
    python rustgame/tools/starten.py -m paket.modul [argumente ...]

Das Programm schreibt weiter `import pygame`, `from pygame.locals import *`
oder `import pygame.freetype` und bekommt rustgame. Ein echtes pygame darf
installiert sein oder fehlen; geladen wird es nicht.

## Warum ein eigenes Werkzeug

`import rustgame_py as pygame` genuegt nur, solange ein Programm allein
ueber diesen einen Namen geht. Die drei Zeilen oben suchen `pygame`,
`pygame.locals` und `pygame.freetype` in `sys.modules` - und finden dort
ohne Umleitung nichts oder das echte pygame.

Umgebogen wird mit derselben Funktion, unter der pygames eigene
Testsuite, die 46 Spiele von PyGameZ und ihre Audits gemessen sind:
`pygame_tests.umbiegen()`. Eine zweite Fassung davon waere eine
ungemessene.

Die Huelle muss gebaut sein: `tools/einbauen.py`.
"""
import os
import runpy
import sys

HIER = os.path.dirname(os.path.abspath(__file__))
WERKZEUG = os.path.join(os.path.dirname(HIER), "crates", "rustgame-py", "tools")


def main():
    argv = sys.argv[1:]
    if not argv or argv[0] in ("-h", "--help") or argv == ["-m"]:
        print(__doc__.strip())
        return 0 if argv and argv[0] != "-m" else 2
    # `pygame_tests` stellt beim Laden Fenster und Ton auf `dummy` - fuer
    # die Testsuite richtig, hier nicht: das Programm soll sein Fenster
    # bekommen. Gemessen beim ersten Lauf: `display.get_driver()` sagte
    # `dummy`, obwohl die Umgebung nichts davon enthielt.
    namen = ("SDL_VIDEODRIVER", "RUSTGAME_VIDEODRIVER", "SDL_AUDIODRIVER",
             "RUSTGAME_AUDIODRIVER", "PYGAME_HIDE_SUPPORT_PROMPT")
    vorher = {k: os.environ.get(k) for k in namen}
    sys.path.insert(0, WERKZEUG)
    import pygame_tests

    sys.path.remove(WERKZEUG)
    for k, v in vorher.items():
        if v is None:
            os.environ.pop(k, None)
        else:
            os.environ[k] = v
    pygame_tests.umbiegen()
    # Der Suchpfad soll aussehen wie bei `python mein_spiel.py`: vorn der
    # Ordner des Programms (bei `-m` das Arbeitsverzeichnis), nicht dieser.
    sys.path[:] = [p for p in sys.path if os.path.normcase(os.path.abspath(p or ".")) !=
                   os.path.normcase(HIER)]
    if argv[0] == "-m":
        sys.argv = argv[1:]
        sys.path.insert(0, os.getcwd())
        runpy.run_module(argv[1], run_name="__main__", alter_sys=True)
    else:
        ziel = os.path.abspath(argv[0])
        sys.argv = argv
        sys.path.insert(0, os.path.dirname(ziel))
        runpy.run_path(ziel, run_name="__main__")
    return 0


if __name__ == "__main__":
    sys.exit(main())
