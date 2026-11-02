# -*- coding: utf-8 -*-
"""Erzeugt `src/farbnamen.rs` aus `pygame.colordict.THECOLORS`.

Neu erzeugen (aus dem Projektordner `pygamez`)::

    .venv/Scripts/python.exe rustgame/crates/rustgame-core/tools/gen_farben.py

Woher die Daten kommen
----------------------
Aus dem **installierten** pygame-ce, nicht aus einer abgeschriebenen
Liste: `pygame/colordict.py` im venv ist die Quelle, ausgelesen zur
Laufzeit ueber `from pygame.colordict import THECOLORS`. Die erzeugte
Datei haelt fest, mit welcher pygame-Fassung sie entstanden ist.

Warum erzeugt und nicht getippt
-------------------------------
Es sind 665 Namen mit je drei Zahlen, also rund zweitausend Ziffern.
Ein Tippfehler faellt beim Uebersetzen **nicht** auf - `("aquamarine3",
(102, 205, 170))` statt `(102, 205, 170)` fuer `mediumaquamarine`
uebersetzt anstandslos und ist trotzdem eine falsche Farbe. Dieselbe
Begruendung wie bei `rustgame-py/tools/gen_konstanten.py`.

Die Tabelle wird **sortiert** abgelegt, damit `color::by_name` binaer
suchen kann; der Generator prueft das nach dem Schreiben nach.
"""
import os

from pygame.colordict import THECOLORS
from pygame.version import ver

HIER = os.path.dirname(os.path.abspath(__file__))
ZIEL = os.path.join(os.path.dirname(HIER), "src", "farbnamen.rs")

namen = sorted(THECOLORS)
assert namen == sorted(set(namen)), "doppelte Namen in THECOLORS"
for n in namen:
    r, g, b, a = THECOLORS[n]
    assert a == 255, "%s hat Alpha %d - die Tabelle geht von 255 aus" % (n, a)
    assert n == n.lower() and n.isalnum(), "unerwarteter Name %r" % n

zeilen = [
    "//! farbnamen.rs",
    "//! ============",
    "//! **Diese Datei ist erzeugt - nicht von Hand aendern.**",
    "//!",
    "//! Quelle: `pygame.colordict.THECOLORS` aus pygame-ce %s, ausgelesen" % ver,
    "//! von `tools/gen_farben.py` dieser Kiste. Bei %d Namen mit je drei"
    % len(namen),
    "//! Zahlen faellt ein Zahlendreher beim Uebersetzen nicht auf - deshalb",
    "//! erzeugt und nicht abgetippt.",
    "//!",
    "//! Nach Namen **sortiert**, damit [`super::by_name`] binaer suchen kann.",
    "",
    "/// Alle %d Farbnamen, die pygame kennt - `(Name, (r, g, b))`." % len(namen),
    "///",
    "/// Das Alpha fehlt mit Absicht: in `THECOLORS` ist es bei allen %d"
    % len(namen),
    "/// Eintraegen 255 (vom Generator geprueft), und eine Spalte, die nie",
    "/// etwas anderes enthaelt, laedt nur dazu ein, sie irgendwann falsch",
    "/// zu fuellen.",
    "pub(super) const TABELLE: &[(&str, (u8, u8, u8))] = &[",
]
for n in namen:
    r, g, b, _ = THECOLORS[n]
    zeilen.append('    ("%s", (%d, %d, %d)),' % (n, r, g, b))
zeilen.append("];")
zeilen.append("")

open(ZIEL, "w", encoding="utf-8", newline="\n").write("\n".join(zeilen))
print("%d Farbnamen -> %s" % (len(namen), ZIEL))
