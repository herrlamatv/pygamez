# -*- coding: utf-8 -*-
"""Wird von jedem Python-Prozess geladen, dessen PYTHONPATH hierher zeigt.

Gebraucht von `tools/pygamez_audits.py`. Damit erben auch **Kindprozesse**
eines Audits dieselbe Engine und denselben umgeleiteten Datenordner wie
der Elternprozess - `tests/arcade_casino_audit.py` startet alle
`tests/audit_*.py` mit `sys.executable`, und ohne diese Datei liefen sie
auf echtem pygame und schrieben in die echte Statistik (gemessen am
25.10.2026: 1125 Duell-Eintraege in `stats.pgzlamastats`).

    PGZ_ENGINE=rustgame|pygame   PGZ_DATEN=<ordner>
"""
import os
import sys

_HIER = os.path.dirname(os.path.abspath(__file__))
_RUSTGAME = os.path.dirname(os.path.dirname(_HIER))
_ROOT = os.path.dirname(_RUSTGAME)
_TOOLS = os.path.join(_RUSTGAME, "crates", "rustgame-py", "tools")

if os.environ.get("PGZ_ENGINE") == "rustgame":
    for _k in ("SDL_VIDEODRIVER", "RUSTGAME_VIDEODRIVER", "SDL_AUDIODRIVER",
               "RUSTGAME_AUDIODRIVER"):
        os.environ.setdefault(_k, "dummy")
    sys.path.insert(0, _TOOLS)
    import pygame_tests as _pt

    _pt.umbiegen()
    sys.path.remove(_TOOLS)

if os.environ.get("PGZ_DATEN"):
    if _ROOT not in sys.path:
        sys.path.append(_ROOT)
    import store as _store

    # `stats` schreibt ueber `store.path_for`, also in `store._DIR` - die
    # Audits setzen nur `store._PATH` um. Darum der ganze Ordner.
    _store._DIR = os.environ["PGZ_DATEN"]
    _store._PATH = os.path.join(_store._DIR, "mem.json")
    for _name in ("ugc", "replay"):
        _m = __import__(_name)
        _m._PATH = os.path.join(_store._DIR, os.path.basename(_m._PATH))
