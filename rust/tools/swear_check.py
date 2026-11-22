# -*- coding: utf-8 -*-
"""Wortfilter Rust gegen Python (swear.check) auf einem erzeugten Korpus.

    .venv/Scripts/python.exe rust/tools/swear_check.py

Korpus: jedes Klartextwort der Musterdateien (Kommentarzeile unter dem
Muster) in vielen Schreibweisen - gross, gesperrt, mit Strichen, Leetspeak,
Akzenten, Satzzeichen davor/dahinter, eingebettet in saubere Woerter - plus
saubere Namen. Ausgabe: Zahl der Abweichungen (Ziel 0).
"""
import glob
import os
import random
import subprocess
import sys

HIER = os.path.dirname(os.path.abspath(__file__))
WURZEL = os.path.dirname(os.path.dirname(HIER))
sys.path.insert(0, WURZEL)

import swear  # noqa: E402

LEET = {"a": "4", "e": "3", "i": "1", "o": "0", "s": "$", "t": "7", "g": "9"}
AKZENT = {"a": "á", "e": "é", "i": "í", "o": "ö", "u": "ü", "c": "ç"}
# Zeichen, die erst die Normalisierung zu einem ASCII-Buchstaben macht: das punktlose i (swear._norm
# ersetzt es ausdruecklich), langes s und Kelvin-K (NFKD). Die Regex-Maschinen der Fassungen setzen
# sie bei "ohne Gross-/Kleinschreibung" nicht alle von sich aus mit i/s/k gleich.
GLEICH = {"i": "ı", "s": "ſ", "k": "\u212a"}
SAUBER = ["Lama", "Tunnelblick", "Pick a game", "Grassland", "Mein Kurs", "Bahn 3", "Pass auf",
          "Classic", "Wasserfall", "Schanze", "Kurve", "Insel", "Labyrinth", "Hasenjagd",
          "Glas", "Klasse", "Sushi", "Analyse", "Assistent", "Kassel", "Cocktail", "Scunthorpe",
          "Kırmızı", "Yıldız", "Işık", "Balık", "Sıkı Fıkı"]


def woerter():
    out = []
    for p in glob.glob(os.path.join(WURZEL, "lang", "**", "swear", "*.yml"), recursive=True):
        with open(p, encoding="utf-8") as f:
            zeilen = f.read().splitlines()
        for a, b in zip(zeilen, zeilen[1:]):
            if a.strip().startswith("-") and b.strip().startswith("#"):
                w = b.strip()[1:].strip()
                if w and len(w) < 30:
                    out.append(w)
    return out


def korpus():
    r = random.Random(5)
    out = list(SAUBER)
    for w in woerter():
        out.append(w)
        out.append(w.upper())
        out.append(" ".join(w))
        out.append("-".join(w))
        out.append("".join(LEET.get(c, c) for c in w))
        out.append("".join(AKZENT.get(c, c) for c in w))
        out.append(w + "!")
        out.append("x" + w)
        out.append(w + "s")
        out.append("mein " + w + " kurs")
        out.append("".join(c * 2 for c in w))
        out.append("".join(LEET.get(c, c) if r.random() < 0.5 else c for c in w) + r.choice(["", "$", "?", "_x"]))
        for von, nach in GLEICH.items():
            if von in w:
                out.append(w.replace(von, nach))
                out.append(w.replace(von, nach).upper())
        if "i" in w:
            out.append(w.replace("i", "İ").upper())      # tuerkisch grossgeschrieben (I mit Punkt)
            out.append(w.replace("i", "ı", 1))
            out.append(" ".join(w.replace("i", "ı")))
            out.append("mein " + w.replace("i", "ı") + " kurs")
        out.append(w[:1] + "ı" + w[1:])                   # punktloses i als zusaetzlicher Buchstabe
        out.append("ı" + w)
        out.append(w + "ı")
    return out


def main():
    texte = [t.replace("\n", " ") for t in korpus()]
    py = []
    for t in texte:
        ok, code = swear.check(t)
        py.append("1" if ok else "0 " + code)
    exe = os.path.join(WURZEL, "rust.output", "release", "examples", "swear_check.exe")
    r = subprocess.run([exe], input="\n".join(texte) + "\n", capture_output=True, text=True, encoding="utf-8")
    rs = r.stdout.splitlines()
    schlecht = 0
    for t, a, b in zip(texte, py, rs):
        if a != b:
            schlecht += 1
            if schlecht <= 15:
                print("ANDERS %r: py %s / rs %s" % (t, a, b))
    print("Wortfilter: %d/%d gleich" % (len(texte) - schlecht, len(texte)))
    return 1 if schlecht or len(rs) != len(py) else 0


if __name__ == "__main__":
    sys.exit(main())
