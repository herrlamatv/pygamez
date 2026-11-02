# -*- coding: utf-8 -*-
"""Erzeugt `src/konstanten_liste.rs` aus `rustgame-display/src/konstanten.rs`.

Neu erzeugen (aus dem Projektordner `pygamez`)::

    .venv/Scripts/python.exe         rustgame/crates/rustgame-py/tools/gen_konstanten.py

Warum erzeugt und nicht getippt: es sind 448 Namen. Ein Tippfehler faellt
beim Uebersetzen auf (der Pfad `k::NAME` gaebe es dann nicht), ein
**vergessener** Name aber nicht - und genau der wuerde erst auffallen,
wenn Monate spaeter `pygame.K_F13` fehlt.
"""
import os
import re

ROOT = r"C:\Users\herrl\PycharmProjects\pygamez\rustgame\crates"
QUELLE = os.path.join(ROOT, "rustgame-display", "src", "konstanten.rs")
ZIEL = os.path.join(ROOT, "rustgame-py", "src", "konstanten_liste.rs")

# `&str` gehoert mit in die Typgruppe - sonst fielen die fuenf
# Zeichenketten-Konstanten schon hier heraus, und niemand merkte es.
muster = re.compile(r"^pub const ([A-Za-z0-9_]+): (&?[A-Za-z0-9_]+) =", re.M)
text = open(QUELLE, encoding="utf-8").read()
treffer = muster.findall(text)

# Zwei Namen gibt es in `pygame.locals` nicht - die bleiben draussen,
# damit die Huelle keinen Namensraum erfindet, den pygame nicht hat.
NICHT_PYGAME = {"NUM_SCANCODES", "SCANCODE_MASK"}

# `Key`, `Scancode`, `EventType`, `KeyMod`, `BlendMode` und
# `AudioFormat` sind Zweitnamen fuer `u32` - die passen ohne Zutun.
#
# `i32` nicht: davon gibt es 25 (die `CONTROLLER_AXIS_*`), und ein paar
# sind **negativ**. Sie werden hier mit `as u32` eingetragen; das
# umgekehrte Vorzeichen holt `konstanten.rs` beim Eintragen wieder
# heraus, denn dort steht schon die Regel "oberstes Bit gesetzt heisst
# negativ". Zwei Stellen mit derselben Absicht waeren eine zu viel.
#
# Diese Zeile ist der Grund, warum hier ueberhaupt jemand vorbeikam: die
# erzeugte Datei war **aelter als ihre Quelle**. Die `i32`-Konstanten
# und `FONT_LEFT`/`-CENTER`/`-RIGHT` standen in `konstanten.rs`, aber
# nicht in der Liste - und ein erzeugtes Ergebnis, das niemand neu
# erzeugt, veraltet genauso still wie eine handgepflegte Liste.
ganze = [(n, t) for n, t in treffer
         if t not in ("usize", "&str") and n not in NICHT_PYGAME]
u32er = [n for n, _ in ganze]

# **Fuenf Konstanten sind Zeichenketten**, nicht Zahlen:
# `NULL_VIDEODRIVER` und die vier `SCRAP_*`. Die fielen hier heraus,
# weil dieser Erzeuger nur eine Zahlentabelle schrieb - und damit fehlten
# sie in der Huelle ganz. Aufgefallen ist das erst, als pygames eigene
# Testsuite lief: `mouse_test.py`, `surface_test.py` und `scrap_test.py`
# brechen schon beim Laden mit
# `AttributeError: module has no attribute 'NULL_VIDEODRIVER'` ab -
# drei ganze Dateien wegen fuenf Zeilen.
texte = [n for n, t in treffer if t == "&str" and n not in NICHT_PYGAME]

zeilen = []
zeilen.append("//! konstanten_liste.rs")
zeilen.append("//! ===================")
zeilen.append("//! **Diese Datei ist erzeugt - nicht von Hand aendern.**")
zeilen.append("//!")
zeilen.append("//! Quelle: `rustgame_display::konstanten`, ausgelesen von")
zeilen.append("//! `tools/gen_konstanten.py` dieser Kiste - dieselbe Begruendung wie dort: bei")
zeilen.append("//! %d Werten faellt ein Zahlendreher nicht beim Uebersetzen auf." % len(treffer))
zeilen.append("//!")
zeilen.append("//! Abgeschrieben werden hier nur die **Namen**. Die Werte stehen als")
zeilen.append("//! Pfad `k::NAME` in der Tabelle, kommen also weiterhin aus der einen")
zeilen.append("//! erzeugten Quelle - aendert sich dort einer, wandert er von selbst")
zeilen.append("//! mit.")
zeilen.append("")
zeilen.append("use rustgame_display::konstanten as k;")
zeilen.append("")
zeilen.append("/// Alle %d Konstanten, die als Zahl nach Python gehen." % len(u32er))
zeilen.append("pub(crate) const ZAHLEN: &[(&str, u32)] = &[")
for n, t in ganze:
    if t == "i32":
        zeilen.append('    ("%s", k::%s as u32),' % (n, n))
    else:
        zeilen.append('    ("%s", k::%s),' % (n, n))
zeilen.append("];")
zeilen.append("")
zeilen.append("/// Die %d Konstanten, die als **Zeichenkette** nach Python gehen." % len(texte))
zeilen.append("pub(crate) const TEXTE: &[(&str, &str)] = &[")
for n in texte:
    zeilen.append('    ("%s", k::%s),' % (n, n))
zeilen.append("];")
zeilen.append("")

open(ZIEL, "w", encoding="utf-8", newline="\n").write("\n".join(zeilen))
print("%d Zahlen und %d Zeichenketten -> %s" % (len(u32er), len(texte), ZIEL))
