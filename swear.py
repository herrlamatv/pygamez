# -*- coding: utf-8 -*-
"""
swear.py
========
Wortfilter für alles, was ein Spieler selbst eintippt und weitergibt.

Gebraucht wird er bei den eigenen Minigolf-Bahnen (siehe ``ugc.py``): Map-Name,
Map-ID und Creator-Name landen in einer Datei, die man verschickt - also darf
dort nichts Beleidigendes stehen.

Die Listen
----------
Je Sprache eine Datei mit Regex-Mustern:

    lang/swear/<code>.yml                 (de, en, es, fr, pt)
    lang/lang.expansion/swear/<code>.yml  (pl, tr, da, no, sv, fi, cs, sl, hr)

Aufbau einer Datei - je Eintrag zwei Zeilen, erst das Muster, darunter das
gemeinte Wort als Kommentar::

    - \\b(f+[\\W\\d_]*[a4]+[\\W\\d_]*g+)(?=[^\\s]*\\b)
    # fag

Mehr Syntax gibt es nicht: leere Zeilen und ``#``-Zeilen werden übersprungen,
alles hinter ``- `` ist ein Regex. Damit braucht es keine YAML-Bibliothek -
das Projekt hängt weiterhin nur an pygame-ce. Ungültige Muster werden still
übersprungen, eine kaputte Zeile legt also nichts lahm.

Geprüft wird IMMER gegen alle 14 Sprachen, nicht nur gegen die eingestellte -
sonst umginge man den Filter, indem man kurz die Sprache umstellt.

Verwendung::

    import swear
    if not swear.is_clean(name):
        ...   # Eingabe ablehnen
"""

import os
import re
import unicodedata

import i18n

# Bundled-Daten liegen neben dem Modul - auch in der PyInstaller-.exe, weil
# __file__ dort in den Entpack-Ordner zeigt (anders als bei mem.json & Co.,
# die BESCHRIEBEN werden und deshalb neben die .exe müssen).
_DIR = os.path.dirname(os.path.abspath(__file__))
_LANG_DIR = os.path.join(_DIR, "lang")
_DIRS = (os.path.join(_LANG_DIR, "swear"),
         os.path.join(_LANG_DIR, "lang.expansion", "swear"))

# Kompilierte Muster je Sprachcode (erst beim ersten Zugriff geladen).
_cache = {}

# Länger als das wird nichts geprüft - ein Riegel gegen Regex-Laufzeit bei
# absurd langen Eingaben (die Felder begrenzen ohnehin auf ~32 Zeichen).
MAX_LEN = 400


def _norm(text):
    """Vereinheitlicht den Text vor dem Vergleich.

    Akzente weg (``Pícá`` -> ``pica``), Kleinschreibung, und ``-``/``_``
    werden zu Leerzeichen - so fängt derselbe Filter auch Map-IDs wie
    ``mein-böses-wort``, die ja keine Leerzeichen enthalten dürfen.
    """
    if not isinstance(text, str):
        return ""
    text = text[:MAX_LEN]
    text = unicodedata.normalize("NFKD", text)
    text = "".join(c for c in text if not unicodedata.combining(c))
    text = text.replace("-", " ").replace("_", " ")
    return re.sub(r"\s+", " ", text).strip().casefold()


def _variants(text):
    """Schreibweisen, gegen die geprüft wird.

    Immer die normalisierte Fassung - und zusätzlich die ohne Leerzeichen,
    wenn der Text WIE GESPERRT GESCHRIEBEN aussieht ("K U R W A"). Die Muster
    selbst dürfen keine Leerzeichen überspringen, sonst liefen sie über
    Wortgrenzen ("45 s" wäre "ass"); diese Zusatzprüfung holt genau den
    Trick zurück, ohne saubere Namen wie "Pick a game" zu treffen.
    """
    norm = _norm(text)
    if not norm:
        return ()
    parts = norm.split(" ")
    if len(parts) >= 3 and sum(len(p) for p in parts) / len(parts) <= 1.5:
        return (norm, norm.replace(" ", ""))
    return (norm,)


def _parse(path):
    """Liest eine Musterdatei (Liste kompilierter Regexe, leer bei Fehler)."""
    out = []
    try:
        with open(path, "r", encoding="utf-8") as f:
            lines = f.read().splitlines()
    except OSError:
        return out
    for line in lines:
        line = line.strip()
        if not line or line.startswith("#"):
            continue          # Leerzeile oder Klartext-Kommentar
        if line.startswith("- "):
            line = line[2:].strip()
        elif line.startswith("-"):
            line = line[1:].strip()
        else:
            continue          # alles andere ist kein Eintrag
        if len(line) >= 2 and line[0] == line[-1] and line[0] in "\"'":
            line = line[1:-1]          # notfalls auch "..." / '...'
        try:
            out.append(re.compile(line, re.IGNORECASE | re.UNICODE))
        except re.error:
            continue          # kaputtes Muster: überspringen, nicht crashen
    return out


def patterns(code):
    """Kompilierte Muster einer Sprache (leer, wenn es keine Datei gibt)."""
    if code in _cache:
        return _cache[code]
    found = []
    for base in _DIRS:
        path = os.path.join(base, "%s.yml" % code)
        if os.path.isfile(path):
            found = _parse(path)
            break
    _cache[code] = found
    return found


def codes():
    """Alle Sprachcodes, gegen die geprüft wird (= die 14 der Oberfläche)."""
    return [code for code, _ in i18n.AVAILABLE]


def check(text):
    """Prüft 'text' gegen alle Sprachen.

    Liefert ``(True, "")`` wenn nichts gefunden wurde, sonst
    ``(False, sprachcode)`` - der Code sagt nur, in welcher Liste der Treffer
    stand (für Fehlersuche; die Meldung an den Spieler ist immer dieselbe).
    """
    forms = _variants(text)
    if not forms:
        return True, ""
    for code in codes():
        for rx in patterns(code):
            if any(rx.search(form) for form in forms):
                return False, code
    return True, ""


def is_clean(text):
    """True, wenn 'text' in keiner der 14 Listen anschlägt."""
    return check(text)[0]


def all_clean(*texts):
    """True, wenn jeder übergebene Text sauber ist (None/'' zählen als sauber)."""
    return all(is_clean(x) for x in texts if x)


def reload():
    """Verwirft den Cache (für Tests und zum Nachladen geänderter Listen)."""
    _cache.clear()
