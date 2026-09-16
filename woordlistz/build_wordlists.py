# -*- coding: utf-8 -*-
"""
build_wordlists.py
==================
Erzeugt die Wordle-Wortlisten in ``woordlistz/<sprache>/`` aus frei
verfügbaren Wörterbüchern und Häufigkeitslisten - für alle 14 Sprachen der
Spielesammlung.

Aufruf (aus dem Projektordner)::

    python woordlistz/build_wordlists.py            # alle Sprachen, lädt Quellen
    python woordlistz/build_wordlists.py de en      # nur einzelne Sprachen
    python woordlistz/build_wordlists.py --cache X  # anderer Ordner für die Quellen

Das Skript ist NICHT Teil des Spiels - es wird nur von Hand aufgerufen, wenn
die Listen neu gebaut werden sollen. Das Spiel liest ausschließlich die
fertigen ``.txt``-Dateien (siehe ``games/wordle_words.py``).

Quellen
-------
* **Hunspell-Wörterbücher** (github.com/wooorm/dictionaries, abgeleitet von den
  LibreOffice-/Hunspell-Wörterbüchern der jeweiligen Sprachgemeinschaft):
  liefern echte Wörterbuch-Einträge (Grundformen).
* **OpenSubtitles-Häufigkeitslisten** (github.com/hermitdave/FrequencyWords):
  liefern die gebeugten Formen aus echtem Text ("HAUSE", "MEINE", ...) und
  sagen, wie geläufig ein Wort ist.
* **Finnisch**: statt Hunspell das Vokabular des Voikko-Projekts
  (joukahainen.xml), weil es für Finnisch kein Hunspell-Wörterbuch gibt.
* **Englisch**: zusätzlich die echten Wordle-Listen (NYT-Lösungswörter und
  erlaubte Rateworte), damit Englisch exakt dem Original entspricht.

Ergebnis je Sprache
-------------------
``answers.txt``   Lösungswörter - geläufige Wörter, die auch im Wörterbuch
                  stehen (Schnittmenge), nach Häufigkeit ausgewählt.
``allowed.txt``   Alle erlaubten Rateworte - Wörterbuch + geläufige Formen aus
                  der Häufigkeitsliste. Enthält immer alle Lösungswörter.

Beide Dateien enthalten ausschließlich Wörter aus genau 5 Großbuchstaben A-Z,
alphabetisch sortiert, eine Zeile je Wort. Umlaute und Akzente werden je
Sprache nach der dort üblichen Schreibweise umgeschrieben (deutsch Ä->AE,
dänisch/norwegisch Å->AA, sonst Akzente weglassen) - die Bildschirmtastatur im
Spiel hat nur A-Z.
"""

import argparse
import os
import re
import sys
import tempfile
import unicodedata
import urllib.request
import xml.etree.ElementTree as ET

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT_DIR = os.path.join(ROOT, "woordlistz")
# Die Quelldateien sind zusammen ~280 MB - die liegen im Temp-Ordner und nicht
# im Projekt (sonst landeten sie in git und in der .exe).
CACHE_DIR = os.path.join(tempfile.gettempdir(), "pygamez-wordlist-sources")

# Reihenfolge wie i18n.AVAILABLE (de zuerst = Standardsprache).
LANGS = ["de", "en", "fr", "es", "pt", "pl", "tr", "da", "no", "sv", "fi",
         "cs", "sl", "hr"]

_FREQ = "https://raw.githubusercontent.com/hermitdave/FrequencyWords/master/content/2018"
_DIC = "https://raw.githubusercontent.com/wooorm/dictionaries/main/dictionaries"

# Sprachcode -> Verzeichnis im Wörterbuch-Repo (Norwegisch: Bokmål).
DIC_NAME = {"de": "de", "en": "en", "fr": "fr", "es": "es", "pt": "pt",
            "pl": "pl", "tr": "tr", "da": "da", "no": "nb", "sv": "sv",
            "cs": "cs", "sl": "sl", "hr": "hr"}

# Zusätzliche Quellen für einzelne Sprachen.
EXTRA_SOURCES = {
    # Finnisch hat kein Hunspell-Wörterbuch - das Voikko-Vokabular übernimmt.
    "fi_vocab": ("https://raw.githubusercontent.com/voikko/corevoikko/master/"
                 "voikko-fi/vocabulary/joukahainen.xml", "fi_vocab.xml"),
    # Die echten Wordle-Listen (New York Times) für Englisch.
    "en_answers": ("https://gist.githubusercontent.com/cfreshman/"
                   "a03ef2cba789d8cf00c08f767e0fad7b/raw/"
                   "wordle-answers-alphabetical.txt", "en_answers.txt"),
    "en_allowed": ("https://raw.githubusercontent.com/tabatkins/wordle-list/"
                   "main/words", "en_allowed.txt"),
}

# --------------------------------------------------------------- Umschrift
# Sonderbuchstaben, die eine Sprache ANDERS schreibt als "Akzent weglassen".
# Deutsch: Ä->AE (wie man es ohne Umlaute tippt). Dänisch/Norwegisch: Å->AA.
# Schwedisch/Finnisch: Å/Ä->A, Ö->O (dort übliche ASCII-Schreibweise).
SPECIAL = {
    "de": {"ä": "ae", "ö": "oe", "ü": "ue", "ß": "ss"},
    "da": {"æ": "ae", "ø": "oe", "å": "aa"},
    "no": {"æ": "ae", "ø": "oe", "å": "aa"},
    "sv": {"å": "a", "ä": "a", "ö": "o"},
    "fi": {"å": "a", "ä": "a", "ö": "o"},
}
# Buchstaben, die keine "Grundform + Akzent" sind und deshalb in JEDER Sprache
# eine eigene Umschrift brauchen (sonst fielen sie beim Entfernen der
# Akzentzeichen komplett weg).
GENERIC = {"ß": "ss", "æ": "ae", "œ": "oe", "ø": "o", "å": "a",
           "ł": "l", "đ": "d", "ð": "d", "þ": "th", "ı": "i", "ŋ": "ng"}

WORD_RE = re.compile(r"^[A-Z]{5}$")

# Wie viele Wörter der Häufigkeitsliste höchstens in die Rateworte wandern und
# wie oft ein Wort dafür mindestens vorkommen muss. Die Obergrenze hält die
# Listen in der Größenordnung des echten Wordle (~13.000 Rateworte) und wirkt
# gegen Tippfehler/Fremdwörter, die in Untertiteln herumliegen.
FREQ_TOP = 15000
FREQ_MIN = 12
# Lösungswörter: geläufig UND im Wörterbuch. Mehr als 4.000 braucht niemand
# (das echte Wordle hat gut 2.300).
ANSWER_MAX = 4000
ANSWER_MIN_COUNT = 30


def fold(word, lang):
    """Schreibt 'word' in 5 Großbuchstaben A-Z um - oder None, wenn das nicht geht."""
    special = SPECIAL.get(lang, {})
    out = []
    for ch in word.lower():
        if ch in special:
            out.append(special[ch])
        elif ch in GENERIC:
            out.append(GENERIC[ch])
        else:
            decomposed = unicodedata.normalize("NFD", ch)
            out.append("".join(c for c in decomposed
                               if not unicodedata.combining(c)))
    word = "".join(out).upper()
    return word if WORD_RE.match(word) else None


# ----------------------------------------------------------------- Quellen
def download(url, name, cache):
    """Lädt 'url' einmalig nach cache/name und liefert den Pfad."""
    path = os.path.join(cache, name)
    if os.path.exists(path) and os.path.getsize(path) > 0:
        return path
    os.makedirs(cache, exist_ok=True)
    print(f"    lade {url}")
    tmp = path + ".part"
    with urllib.request.urlopen(url, timeout=300) as src, open(tmp, "wb") as dst:
        while True:
            chunk = src.read(1 << 20)
            if not chunk:
                break
            dst.write(chunk)
    os.replace(tmp, path)
    return path


# Deklinations-Flags im deutschen Hunspell-Wörterbuch (igerman98). Ein groß
# geschriebener Eintrag MIT einem dieser Flags ist ein Substantiv
# ("Blume/Nm", "Tisch/EPSTMmij"), einer ohne ist ein Eigenname ("David/S",
# "Petra/S", "Nokia/S"). Eigennamen sollen keine Lösungswörter werden.
DE_NOUN_FLAGS = set("NEPTMRJ")


def read_dictionary(lang, cache):
    """Wörterbuch-Einträge als (alle Wörter, für Lösungen taugliche Wörter).

    Die zweite Menge lässt Eigennamen weg: in den meisten Sprachen stehen sie
    groß im Wörterbuch und fliegen schon beim Einlesen raus - im Deutschen, wo
    jedes Substantiv groß steht, entscheiden stattdessen die Deklinations-Flags.
    """
    words = set()
    good = set()
    if lang == "fi":
        url, name = EXTRA_SOURCES["fi_vocab"]
        path = download(url, name, cache)
        # <word><forms><form>sana</form></forms><classes><wclass>..</wclass>
        for _, elem in ET.iterparse(path, events=("end",)):
            if not elem.tag.endswith("word"):
                continue
            classes = [c.text or "" for c in elem.iter("wclass")]
            # pnoun_* = Eigennamen (Personen, Orte, Marken).
            proper = any(c.startswith("pnoun") for c in classes)
            for form in elem.iter("form"):
                folded = fold((form.text or "").strip(), lang)
                if folded:
                    words.add(folded)
                    if not proper:
                        good.add(folded)
            elem.clear()
        return words, good

    path = download(f"{_DIC}/{DIC_NAME[lang]}/index.dic", f"{lang}.dic", cache)
    with open(path, "r", encoding="utf-8", errors="replace") as f:
        next(f, None)                      # erste Zeile = Anzahl der Einträge
        for line in f:
            # Aufbau: "wort/FLAGS<TAB>morphologie"
            entry = line.strip().split("\t")[0]
            stem, _, flags = entry.partition("/")
            if not stem or " " in stem or "." in stem or "-" in stem:
                continue
            # Großgeschriebene Einträge sind Eigennamen - außer im Deutschen,
            # wo JEDES Substantiv groß steht (und Substantive die besten
            # Rätselwörter sind).
            if lang != "de" and stem[:1].isupper():
                continue
            folded = fold(stem, lang)
            if not folded:
                continue
            words.add(folded)
            if (lang != "de" or not stem[:1].isupper()
                    or set(flags) & DE_NOUN_FLAGS):
                good.add(folded)
    return words, good


def read_frequencies(lang, cache):
    """Häufigkeitsliste als dict wort -> Anzahl (nur 5-Buchstaben-Wörter)."""
    path = download(f"{_FREQ}/{lang}/{lang}_full.txt", f"{lang}_freq.txt", cache)
    counts = {}
    with open(path, "r", encoding="utf-8", errors="replace") as f:
        for line in f:
            parts = line.split()
            if len(parts) != 2 or not parts[1].isdigit():
                continue
            folded = fold(parts[0], lang)
            if folded:
                # Mehrere Schreibweisen können auf dasselbe Wort fallen
                # (z.B. "für"/"fur") - die Häufigkeiten addieren sich.
                counts[folded] = counts.get(folded, 0) + int(parts[1])
    return counts


def read_wordle_lists(cache):
    """Die echten englischen Wordle-Listen: (Lösungswörter, Rateworte)."""
    def lines(key):
        url, name = EXTRA_SOURCES[key]
        path = download(url, name, cache)
        with open(path, "r", encoding="utf-8", errors="replace") as f:
            return {w for w in (fold(x.strip(), "en") for x in f) if w}
    return lines("en_answers"), lines("en_allowed")


# ------------------------------------------------------------------ Filter
def clean_filter():
    """Liefert eine Funktion, die Schimpfwörter aussortiert (swear.py)."""
    sys.path.insert(0, ROOT)
    try:
        import swear
    except Exception as exc:                          # pragma: no cover
        print(f"    (swear.py nicht nutzbar: {exc} - Filter übersprungen)")
        return lambda word: True
    cache = {}

    def ok(word):
        if word not in cache:
            cache[word] = swear.is_clean(word)
        return cache[word]
    return ok


# ------------------------------------------------------------------- Bauen
def build(lang, cache):
    """Baut (Lösungswörter, Rateworte) für eine Sprache."""
    print(f"  {lang}: Quellen lesen ...")
    dictionary, solvable = read_dictionary(lang, cache)
    counts = read_frequencies(lang, cache)
    is_clean = clean_filter()

    # Rateworte: alles aus dem Wörterbuch + die geläufigsten Formen aus echtem
    # Text. Die Häufigkeitsliste bringt die gebeugten Formen mit, die in einem
    # Wörterbuch nur als Grundform stehen (HAUSE, MEINE, KOMMT ...).
    common = sorted((w for w, n in counts.items() if n >= FREQ_MIN),
                    key=lambda w: -counts[w])[:FREQ_TOP]
    allowed = set(dictionary) | set(common)

    # Lösungswörter: geläufig UND im Wörterbuch (ohne Eigennamen) - das hält
    # Tippfehler, Namen und Wortfetzen ("DOESN", "WEREN") aus den Lösungen.
    answers = sorted((w for w in solvable
                      if counts.get(w, 0) >= ANSWER_MIN_COUNT),
                     key=lambda w: -counts[w])[:ANSWER_MAX]

    if lang == "en":
        # Englisch bekommt die echten Wordle-Listen: exakt die Lösungswörter
        # der New York Times, dazu deren erlaubte Rateworte. Die
        # Untertitel-Häufigkeiten bleiben hier außen vor - sie brächten vor
        # allem Namen und Tippfehler ("RYOGA", "PEOPL") in die Rateworte.
        nyt_answers, nyt_allowed = read_wordle_lists(cache)
        answers = sorted(nyt_answers)
        allowed = set(dictionary) | nyt_allowed | nyt_answers

    answers = [w for w in answers if is_clean(w)]
    allowed = {w for w in allowed if is_clean(w)}
    allowed |= set(answers)                # Lösungswörter sind immer erlaubt
    return sorted(set(answers)), sorted(allowed)


def write_list(path, words):
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(words) + "\n")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[3])
    parser.add_argument("langs", nargs="*", default=None,
                        help=f"Sprachen (Standard: alle {len(LANGS)})")
    parser.add_argument("--cache", default=CACHE_DIR,
                        help="Ordner für die heruntergeladenen Quellen")
    parser.add_argument("--out", default=OUT_DIR, help="Zielordner")
    args = parser.parse_args(argv)

    langs = args.langs or LANGS
    unknown = [l for l in langs if l not in LANGS]
    if unknown:
        parser.error(f"unbekannte Sprache(n): {', '.join(unknown)}")

    print(f"Wortlisten bauen -> {args.out}")
    summary = []
    for lang in langs:
        answers, allowed = build(lang, args.cache)
        folder = os.path.join(args.out, lang)
        os.makedirs(folder, exist_ok=True)
        write_list(os.path.join(folder, "answers.txt"), answers)
        write_list(os.path.join(folder, "allowed.txt"), allowed)
        summary.append((lang, len(answers), len(allowed)))
        print(f"  {lang}: {len(answers)} Lösungswörter, {len(allowed)} Rateworte")

    print("\n| Sprache | Lösungswörter | Rateworte |")
    print("|---------|---------------|-----------|")
    for lang, n_ans, n_all in summary:
        print(f"| {lang} | {n_ans} | {n_all} |")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
