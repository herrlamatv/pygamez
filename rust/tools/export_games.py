# -*- coding: utf-8 -*-
"""
export_games.py
===============
Erzeugt die Datentabellen der Spiele (Wortlisten, Level, Bahnen ...) aus den
Python-Quellen als Rust-Module unter ``crates/pgz-games/src/data/``.

Stammdaten werden **erzeugt, nicht abgetippt** - sonst laufen die Fassungen
auseinander. Nach jeder Aenderung an einer der gelesenen Python-Dateien:

    .venv/Scripts/python.exe rust/tools/export_games.py
"""

import os
import sys

HIER = os.path.dirname(os.path.abspath(__file__))
RUST = os.path.dirname(HIER)
WURZEL = os.path.dirname(RUST)
OUT = os.path.join(RUST, "crates", "pgz-games", "src", "data")
sys.path.insert(0, WURZEL)
os.environ.setdefault("PYGAME_HIDE_SUPPORT_PROMPT", "1")


def rs_str(s):
    return '"%s"' % str(s).replace("\\", "\\\\").replace('"', '\\"')


def schreiben(name, zeilen, quelle):
    os.makedirs(OUT, exist_ok=True)
    kopf = ["//! %s.rs - ERZEUGT von rust/tools/export_games.py aus %s." % (name, quelle),
            "//! Nicht von Hand aendern.", "", "#![allow(missing_docs)]", ""]
    with open(os.path.join(OUT, name + ".rs"), "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(kopf + zeilen) + "\n")
    print("geschrieben:", name)


def hangman():
    from games import hangman_words as hw
    z = ["/// `hangman_words.WORDS` - je Sprache die Rohliste.",
         "pub const WORDS: &[(&str, &[&str])] = &["]
    for lang, words in hw.WORDS.items():
        z.append("    (%s, &[%s])," % (rs_str(lang), ", ".join(rs_str(w) for w in words)))
    z.append("];")
    z.append("")
    z.append("/// `hangman_words._RANGES`")
    z.append("pub const RANGES: &[(&str, usize, usize)] = &[")
    for mode, (lo, hi) in hw._RANGES.items():
        z.append("    (%s, %d, %d)," % (rs_str(mode), lo, hi))
    z.append("];")
    schreiben("hangman_words", z, "games/hangman_words.py")


AUFGABEN = [hangman]


def main():
    namen = []
    for a in AUFGABEN:
        a()
        namen.append(a.__name__)
    # mod.rs mit allen erzeugten Modulen.
    mods = sorted(f[:-3] for f in os.listdir(OUT) if f.endswith(".rs") and f != "mod.rs")
    with open(os.path.join(OUT, "mod.rs"), "w", encoding="utf-8", newline="\n") as f:
        f.write("//! Erzeugte Datentabellen der Spiele (rust/tools/export_games.py).\n\n")
        for m in mods:
            f.write("pub mod %s;\n" % m)


if __name__ == "__main__":
    main()
