# -*- coding: utf-8 -*-
"""Erzeugt `src/zeiger_daten.rs` aus pygames eigener `cursors.py`.

Neu erzeugen (aus dem Projektordner `pygamez`)::

    .venv/Scripts/python.exe rustgame/crates/rustgame-py/tools/gen_zeiger.py

Warum erzeugt und nicht abgetippt
---------------------------------
`pygame/cursors.py` ist zu zwei Dritteln **Daten**: sechs fertige Zeiger
als Hexlisten (`arrow`, `diamond`, `ball`, `broken_x`, `tri_left`,
`tri_right`, zusammen rund 430 Zeilen) plus fuenf Zeichenkettenformen
(`thickarrow_strings` und Geschwister). Dazu zwei Funktionen, `compile`
und `load_xbm`, beide reines Python.

Dieses Projekt uebertraegt, statt nachzubauen - und wo die Vorlage schon
Python ist, heisst uebertragen: **uebernehmen**. Ein Generator macht das
nachpruefbar: er schneidet den Abschnitt aus der installierten Datei
heraus, statt dass jemand 430 Zeilen Hexzahlen mit der Hand kopiert und
sich dabei bei einer vertippt.

Was **nicht** uebernommen wird, steht unten bei `WEGLASSEN`.
"""

import io
import os
import re

HIER = os.path.dirname(os.path.abspath(__file__))
ZIEL = os.path.join(os.path.dirname(HIER), "src", "zeiger_daten.rs")

import pygame.cursors  # noqa: E402

QUELLE = pygame.cursors.__file__

with io.open(QUELLE, encoding="utf-8") as f:
    zeilen = f.readlines()


def zeile_von(muster):
    """Nummer der ersten Zeile, die mit `muster` anfaengt."""
    for i, z in enumerate(zeilen):
        if z.startswith(muster):
            return i
    raise SystemExit("nicht gefunden: %r" % muster)


# Alles ab `arrow = Cursor(` bis zum Dateiende: die sechs Zeiger, die
# fuenf Zeichenkettenformen, `compile` und `load_xbm`.
#
# WEGLASSEN - und jedes Mal mit Grund:
#   * `import pygame`        - hier gibt es kein `pygame`.
#   * `_cursor_id_table`     - die Zuordnung Zahl -> Name steht in
#                              `zeiger.rs` und kommt dort aus der
#                              erzeugten Konstantenliste; zwei Abschriften
#                              waeren eine zu viel.
#   * `class Cursor`         - die Klasse ist `zeiger::PyCursor`, sonst
#                              waere `mouse.get_cursor()` ein anderer Typ
#                              als `cursors.Cursor`.
#   * `set_cursor`/`get_cursor` - die haengt pygame von hier aus an
#                              `pygame.mouse`; bei uns stehen sie schon
#                              dort.
anfang = zeile_von("arrow = Cursor(")
abschnitt = "".join(zeilen[anfang:])

# `__all__` traegt Namen, die es hier nicht gibt - raus damit.
abschnitt = re.sub(r"^__all__ = .*?\]\n", "", abschnitt, flags=re.S | re.M)

# Ein Rust-Rohstring vertraegt Anfuehrungszeichen und Zeilenumbrueche,
# solange sein Abschluss im Text nicht vorkommt. `"#` **kommt** vor -
# in der Doku von `compile` steht `"""#sized 24x24`. Also zwei Rauten.
assert '"##' not in abschnitt, "Rohstring-Abschluss kommt im Text vor"

kopf = '''//! zeiger_daten.rs
//! ===============
//! **Diese Datei ist erzeugt - nicht von Hand aendern.**
//!
//! Quelle: `pygame/cursors.py` der installierten pygame-ce-Fassung,
//! ausgeschnitten von `tools/gen_zeiger.py` dieser Kiste. Uebernommen
//! wird alles ab `arrow = Cursor(`: die sechs fertigen Zeiger, die fuenf
//! Zeichenkettenformen, `compile` und `load_xbm`.
//!
//! Warum uebernommen statt nachgebaut: zwei Drittel davon sind **Daten**
//! (rund %d Zeilen Hexzahlen), und `compile`/`load_xbm` sind reines
//! Python. Was in pygame schon Python ist, wird hier uebernommen - die
//! gleiche Entscheidung wie bei `sysfont.py` und `version.py`.
//!
//! `Cursor` reicht [`crate::zeiger`] herein: `mouse.get_cursor()` muss
//! denselben Typ liefern wie `cursors.Cursor`, sonst scheitert jedes
//! `isinstance`.

/// Der uebernommene Abschnitt aus `pygame/cursors.py`.
pub const ZEIGER_PY: &str = r##"
%s"##;
''' % (anfang, abschnitt)

with io.open(ZIEL, "w", encoding="utf-8", newline="\n") as f:
    f.write(kopf)

print("%d Zeilen aus %s -> %s" % (abschnitt.count("\n"), QUELLE, ZIEL))
