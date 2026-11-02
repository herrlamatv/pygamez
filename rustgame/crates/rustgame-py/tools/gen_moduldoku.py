# -*- coding: utf-8 -*-
"""Erzeugt `src/moduldoku.rs` - die Modul-Docstrings aus **echtem** pygame.

## Warum erzeugt und nicht geschrieben

`__doc__` ist Schnittstelle: `help(pygame.draw)`, `pydoc`, jede
Entwicklungsumgebung zeigt ihn. Nach der Regel dieses Projekts gehoert
alles, was nach aussen sichtbar ist, pygame - und nach der anderen Regel
gehoeren die Doku-Kommentare im Quelltext dem, der sie liest, also auf
Deutsch. PyO3 verknuepft beides: der Rust-Kommentar **wird** zum
`__doc__`.

Fuer **Module** laesst sich das trennen, weil `__doc__` an einem Modul
schreibbar ist (an einer Methode nicht - `method_descriptor` ist
unveraenderlich). Gemessen fehlte er bei **36** von 60 Modulen ganz;
`help(rustgame_py.draw)` zeigte nichts.

Also: der deutsche Vermerk bleibt im Quelltext, und beim Modulaufbau
wird pygames Satz darueber gelegt - aus dieser erzeugten Tabelle, damit
niemand 60 Saetze von Hand pflegt.

## Fahren

```bash
.venv/Scripts/python.exe rustgame/crates/rustgame-py/tools/gen_moduldoku.py
```

Schreibt `crates/rustgame-py/src/moduldoku.rs`. Die Datei ist
eingecheckt - derselbe Vertrag wie bei `gen_konstanten.py` und
`gen_golden.py`: erzeugt, aber nicht beim Bauen.
"""
import importlib
import os
import pkgutil
import sys

os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
os.environ.setdefault("SDL_AUDIODRIVER", "dummy")
os.environ.setdefault("PYGAME_HIDE_SUPPORT_PROMPT", "1")

HIER = os.path.dirname(os.path.abspath(__file__))
ZIEL = os.path.join(os.path.dirname(HIER), "src", "moduldoku.rs")


def rust_text(s):
    """Eine Rust-Zeichenkette - `r#"..."#` wo moeglich, sonst entwertet."""
    if '"#' not in s:
        return 'r#"%s"#' % s
    return '"%s"' % s.replace("\\", "\\\\").replace('"', '\\"').replace(
        "\n", "\\n")


def main():
    import pygame

    namen = sorted(m.name for m in pkgutil.iter_modules(pygame.__path__)
                   if m.name != "tests")
    zeilen = []
    for n in [""] + namen:
        voll = "pygame" + ("." + n if n else "")
        try:
            m = importlib.import_module(voll)
        except Exception:                            # noqa: BLE001
            continue
        d = getattr(m, "__doc__", None)
        if not (isinstance(d, str) and d.strip()):
            continue
        zeilen.append((n or "", d))

    # Die Klassen dazu - an einer Klasse ist `__doc__` ebenfalls
    # schreibbar (gemessen), an ihren Methoden nicht.
    klassen = []
    gesehen = set()
    for n in [""] + namen:
        voll = "pygame" + ("." + n if n else "")
        try:
            m = importlib.import_module(voll)
        except Exception:                            # noqa: BLE001
            continue
        for a in sorted(dir(m)):
            if a.startswith("_"):
                continue
            try:
                w = getattr(m, a)
            except Exception:                        # noqa: BLE001
                continue
            if not isinstance(w, type):
                continue
            d = getattr(w, "__doc__", None)
            if not (isinstance(d, str) and d.strip()):
                continue
            if (n, a) in gesehen:
                continue
            gesehen.add((n, a))
            klassen.append((n or "", a, d))

    with open(ZIEL, "w", encoding="utf-8", newline="\n") as f:
        f.write('''//! moduldoku.rs
//! ============
//! Die `__doc__`-Saetze aller Module - **aus echtem pygame erzeugt.**
//!
//! Erzeugt von `tools/gen_moduldoku.py`; nicht von Hand aendern. Warum es
//! diese Tabelle gibt, steht dort: `__doc__` ist Schnittstelle und gehoert
//! damit pygame, waehrend die Doku-Kommentare im Quelltext deutsch
//! bleiben. An einem **Modul** ist `__doc__` schreibbar, an einer Methode
//! nicht - deshalb nur Module.
//!
//! Gemessen fehlte der Satz bei **36** von 60 Modulen ganz
//! (`help(rustgame_py.draw)` zeigte nichts), und bei den eingebetteten
//! Python-Modulen stand deutsche Prosa darin.

/// `(Untermodul, Satz)` - leerer Name heisst das Wurzelmodul.
///
/// Zeilenzahl: siehe unten; die Tabelle ist erzeugt.
pub const MODULDOKU: &[(&str, &str)] = &[
''')
        for n, d in zeilen:
            f.write("    (%s, %s),\n" % (rust_text(n), rust_text(d)))
        f.write("];\n")
        f.write('''
/// `(Untermodul, Klasse, Satz)` - leerer Modulname heisst das Wurzelmodul.
///
/// Dieselbe Sache fuer die Klassen: `help(pygame.Surface)` zeigt
/// `Surface(size, flags=0, depth=0, masks=None) -> Surface` und darunter
/// den Satz. Eine Klasse kann unter mehreren Namen stehen
/// (`Channel`, `mixer.Channel`, `mixer.ChannelType` sind **ein** Objekt);
/// die Tabelle fuehrt jeden Weg, und derselbe Satz zweimal zu setzen
/// kostet nichts.
pub const KLASSENDOKU: &[(&str, &str, &str)] = &[
''')
        for n, k, d in klassen:
            f.write("    (%s, %s, %s),\n"
                    % (rust_text(n), rust_text(k), rust_text(d)))
        f.write("];\n")
    print("%d Modul- und %d Klassen-Docstrings -> %s"
          % (len(zeilen), len(klassen), ZIEL))
    return 0


if __name__ == "__main__":
    sys.exit(main())
