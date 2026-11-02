# -*- coding: utf-8 -*-
"""Steht in einem Fehlertext Deutsch? - **ein Waechter, kein Bericht.**

Ein Fehlertext ist das, was ein Mensch zu sehen bekommt. Deutsch darin ist
derselbe Fehler wie ein deutscher Parametername, nur lauter: niemand, der
`import pygame` schreibt, erwartet

    pygame.error: Unbekanntes Klangformat - erwartet wird WAV, OGG,
                  FLAC oder MP3

Genau dieser Satz stand hier, und mit ihm 85 weitere - 32 in
`rustgame-core/src/formate.rs`, 25 in den Anzeige-Stuecken, 19 im Mixer.
Gefunden hat sie nicht dieses Werkzeug, sondern die Probe `Fehlertexte`,
und zwar an **einem** davon; die anderen 85 kamen erst heraus, als danach
gesucht wurde. *Ein Fund ist ein Hinweis auf seine Geschwister.*

Seitdem haelt dieses Werkzeug den Stand: **0**. Es endet mit 1, sobald
einer dazukommt.

## Was nicht zaehlt

* **Kommentare und Dokumentation** - die sind Dokumentation, nicht
  Schnittstelle, und bleiben deutsch.
* **Namen in `{}`** - `format!` setzt dort einen Wert ein; aus
  `takes no arguments ({anzahl} given)` wird nie etwas Deutsches.
* **`tools/`** - was dort steht, liest nur, wer die Messung fahrt.
* **Testhilfen** (`assert!(.., "kein Pixel zu viel")`) - die sieht nur,
  wer `cargo test` fahrt.

## Gebrauch

```bash
.venv/Scripts/python.exe rustgame/tools/deutsche_fehlertexte.py
```

## Was gesucht wird

Gesucht wird in **jedem** Ausdruck, der einen Fehlerwert baut:

* `Fehler::Irgendwas("...".into())` und `Fehler::Irgendwas(format!("..."))`,
* `new_err("...")` in der PyO3-Huelle,
* `raise X("...")` in der eingebetteten Python-Seite.

Testhilfen (`assert!(..., "...")`) zaehlen nicht - die sieht nur, wer die
Tests fahrt.
"""
import io
import os
import re

HIER = os.path.dirname(os.path.abspath(__file__))
W = os.path.join(os.path.dirname(HIER), "crates")

MUSTER = [
    # Fehler::Variante("..."), auch ueber format!
    re.compile(r"Fehler::[A-Za-z_]+\s*\(\s*(?:format!\s*\(\s*)?"
               r'("(?:[^"\\]|\\.)*")'),
    re.compile(r'(?:new_err|new_err::<[^>]*>)\s*\(\s*(?:format!\s*\(\s*)?'
               r'("(?:[^"\\]|\\.)*")'),
    re.compile(r'(?:NichtUnterstuetzt|Format|Bild|Ton)\s*\(\s*'
               r'("(?:[^"\\]|\\.)*")\s*\.into\(\)'),
    # **Die dritte Gestalt: `impl Display for Fehler`.**
    #
    # Die Muster darueber suchen, wo ein Fehler **gebaut** wird. Ein
    # Fehlertyp mit eigenem `Display` schreibt seinen Satz aber erst beim
    # Anzeigen hin - `write!(f, "..")` -, und davon wusste dieser Sucher
    # nichts. Gefunden hat die Luecke ein Aufruf, der sie ausloeste:
    # `freetype.Font(None, 24)` gab `freesansbold.ttf nicht gefunden -
    # RUSTGAME_DEFAULT_FONT setzen`, **nachdem** der Sucher 0 gemeldet
    # hatte.
    #
    # *Ein Sucher, der Gestalten aufzaehlt, findet nur die, die jemand
    # aufgezaehlt hat* - dieselbe Falle wie bei jedem blinden Fleck in
    # diesem Projekt, nur diesmal im Messgeraet.
    re.compile(r'write(?:ln)?!\s*\(\s*[A-Za-z_][A-Za-z_0-9]*\s*,\s*'
               r'("(?:[^"\\]|\\.)*")'),
    re.compile(r'#\[error\(\s*("(?:[^"\\]|\\.)*")'),
]
PY = re.compile(r"raise\s+[A-Za-z_.]+\s*\(\s*(f?\"(?:[^\"\\]|\\.)*\")")

# **Die zweite Liste - Woerter, die es im Englischen nicht gibt.**
#
# Runde 18: der Sucher meldete 0, und im Mixer stand `WAV ohne fmt-Block`.
# "ohne" stand in keiner der beiden Listen, und `zu wenig` traf `zu wenige`
# nicht (das `\b` danach). Ein Suchlauf mit dieser Liste fand **30**
# weitere Stellen, das Lesen danach noch sechs - 36 in allen: in den
# Bildformaten, im WAV-Leser, bei WavPack und in der Anzeige. Die Liste
# unten sind deutsche Funktionswoerter und die Wortstaemme aus diesen
# Funden; `\w*` hinter den Staemmen faengt die gebeugten Formen.
HART_ZUSATZ = (
    r"ohne|mit|fuer|bei|nach|von|vom|zum|zur|ueber|auf|aus|noch|schon|nur|"
    r"auch|aber|weder|kann|koennen|konnte|soll|sollte|hat|haben|wurde|"
    r"wuerde|waere|macht|machen|reicht|reichen|bricht|"
    r"zu wenig\w*|zu viel\w*|zu kurz\w*|zu lang\w*|mitten|Ende|"
    r"unvollstaendig\w*|unbrauchbar\w*|"
    r"Hoechstwert\w*|Kopfzeile\w*|Kopfdaten|Kennung|Farbtafel|Farben|"
    r"Farbart|Bilddaten|Zeilen|Spur|Spuren|Tonspur|Angaben|Abtastrate|"
    r"Bild|Klang|Kanal|Rechteck|Zeichen|leer|falsch|gesetzt|geladen|"
    r"gefunden|geoeffnet|geschlossen|unterstuetzt\w*|Geraet\w*|Ereignis\w*"
)

# Unmissverstaendlich deutsche Wortstuecke.
DE = re.compile(
    r"\b(nicht|kein|keine|keinen|erwartet|erwarte|erwartete|unbekannt|"
    r"unbekannte|unbekanntes|ungueltig|ungueltige|ungueltiges|muss|darf|"
    r"fehlt|fehlen|liess|liessen|gibt|wird|werden|sind|ist|zu gross|"
    r"zu klein|zu viele|zu wenig|braucht|Groesse|Laenge|Breite|Hoehe|"
    r"Flaeche|Farbe|Zahl|Datei|Dateiname|Dateiobjekt|Klangformat|"
    r"Punkt|Vektor|Speicher|Fenster|Schrift|Anzahl|Stelle|Wert|"
    r"elementweises|gleicher|einer|eine|einen|oder|und|der|die|das|"
    + HART_ZUSATZ + r")\b",
    re.I)


def main():
    treffer = []
    for ordner, _, namen in os.walk(W):
        # `tools/` zaehlt nicht: was dort steht, liest nur, wer die
        # Messung fahrt - kein Nutzer von `import pygame`.
        if ("target" in ordner or "tests" in ordner
                or ordner.rstrip(os.sep + "/").endswith("tools")):
            continue
        for n in sorted(namen):
            if not n.endswith((".rs", ".py")):
                continue
            p = os.path.join(ordner, n)
            s = io.open(p, encoding="utf-8", errors="replace").read()
            fund = []
            for m in MUSTER:
                fund += [(x.start(), x.group(1)) for x in m.finditer(s)]
            if n.endswith(".py"):
                fund += [(x.start(), x.group(1)) for x in PY.finditer(s)]
            for pos, t in fund:
                roh = t[1:-1]
                # **Was in `{}` steht, sieht niemand.** `format!` setzt
                # dort einen Wert ein; der Name der Rust-Variablen
                # (`{anzahl}`, `{groesse}`, `{stelle}`) steht nie im
                # fertigen Satz. Ohne diese Zeile meldete der Sucher vier
                # englische Saetze als deutsch.
                roh = re.sub(r"\{[^}]*\}", "{}", roh)
                if not DE.search(roh):
                    continue
                # Englische Saetze enthalten "is", "or", "and" - die
                # stehen auch in DE. Also nur, wenn ein Wort dabei ist,
                # das es im Englischen nicht gibt.
                hart = re.search(
                    r"\b(nicht|kein|keine|keinen|erwartet|erwarte|"
                    r"unbekannt|unbekannte|unbekanntes|ungueltig|muss|"
                    r"darf|fehlt|fehlen|liess|liessen|werden|sind|"
                    r"zu gross|zu klein|zu viele|zu wenig|braucht|"
                    r"Groesse|Laenge|Breite|Hoehe|Flaeche|Farbe|Zahl|"
                    r"Datei|Dateiname|Dateiobjekt|Klangformat|Punkt|"
                    r"Vektor|Speicher|Fenster|Schrift|Anzahl|Stelle|"
                    r"elementweises|gleicher|" + HART_ZUSATZ + r")\b",
                    roh, re.I)
                if not hart:
                    continue
                treffer.append([os.path.relpath(p, W).replace("\\\\", "/"),
                                s[:pos].count("\n") + 1, roh])
    print("Deutsche Fehlertexte: %d" % len(treffer))
    for p, z, t in treffer:
        print("  %-40s %5d  %s" % (p, z, t[:100]))
    if treffer:
        print()
        print("Ein Fehlertext ist Schnittstelle - er gehoert pygame.")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
