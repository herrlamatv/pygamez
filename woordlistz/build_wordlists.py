# -*- coding: utf-8 -*-
"""
build_wordlists.py
==================
Erzeugt die Wordle-Wortlisten in ``woordlistz/<sprache>/`` aus frei
verfügbaren Wörterbüchern und Häufigkeitslisten - für alle 14 Sprachen der
Spielesammlung und die Wortlängen 4 bis 7.

Aufruf (aus dem Projektordner)::

    python woordlistz/build_wordlists.py                # alles, lädt Quellen
    python woordlistz/build_wordlists.py de en          # nur einzelne Sprachen
    python woordlistz/build_wordlists.py --lengths 6 7  # nur einzelne Längen
    python woordlistz/build_wordlists.py --cache X      # anderer Quellen-Ordner
    python woordlistz/build_wordlists.py --sample 12    # Stichproben ausgeben

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
* **Englisch, 5 Buchstaben**: die echten Wordle-Listen (NYT-Lösungswörter und
  erlaubte Rateworte), damit Englisch exakt dem Original entspricht.
* **Englisch, 4/6/7 Buchstaben**: dafür gibt es keine Original-Listen. Die
  Rateworte kommen aus der gemeinfreien Scrabble-Liste ENABLE (enthält auch
  alle gebeugten Formen), die Lösungswörter müssen zusätzlich im Hunspell-
  Wörterbuch stehen UND in echtem Text geläufig sein - und laufen durch eine
  kleine Sperrliste (Ausrufe, Abkürzungen, Umgangssprache).

Ergebnis je Sprache und Länge
-----------------------------
``answers.txt`` / ``answers<N>.txt``
                  Lösungswörter - geläufige Wörter, die auch im Wörterbuch
                  stehen (Schnittmenge), nach Häufigkeit ausgewählt.
``allowed.txt`` / ``allowed<N>.txt``
                  Alle erlaubten Rateworte - Wörterbuch + geläufige Formen aus
                  der Häufigkeitsliste. Enthält immer alle Lösungswörter.

Die 5-Buchstaben-Listen behalten ihre alten Namen (``answers.txt``), die
übrigen Längen bekommen die Länge angehängt (``answers4.txt``,
``allowed7.txt`` ...). Alle Dateien enthalten ausschließlich Wörter aus genau
N Großbuchstaben A-Z, alphabetisch sortiert, eine Zeile je Wort. Umlaute und
Akzente werden je Sprache nach der dort üblichen Schreibweise umgeschrieben
(deutsch Ä->AE, dänisch/norwegisch Å->AA, sonst Akzente weglassen) - die
Bildschirmtastatur im Spiel hat nur A-Z.
"""

import argparse
import os
import random
import re
import sys
import tarfile
import tempfile
import unicodedata
import urllib.request
import xml.etree.ElementTree as ET

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT_DIR = os.path.join(ROOT, "woordlistz")
# Die Quelldateien sind zusammen ~320 MB - die liegen im Temp-Ordner und nicht
# im Projekt (sonst landeten sie in git und in der .exe).
CACHE_DIR = os.path.join(tempfile.gettempdir(), "pygamez-wordlist-sources")

# Reihenfolge wie i18n.AVAILABLE (de zuerst = Standardsprache).
LANGS = ["de", "en", "fr", "es", "pt", "pl", "tr", "da", "no", "sv", "fi",
         "cs", "sl", "hr"]
# Wortlängen, die das Spiel anbietet (5 = klassisches Wordle).
LENGTHS = (4, 5, 6, 7)

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
    # ENABLE (gemeinfrei): die Wortliste vieler Scrabble-artiger Spiele -
    # Englisch in allen Längen samt gebeugter Formen, ohne Namen/Abkürzungen.
    "en_enable": ("https://raw.githubusercontent.com/dolph/dictionary/"
                  "master/enable1.txt", "en_enable1.txt"),
    # WordNet 3.0 (Princeton, freie Lizenz): nur die Grundformen - hält
    # "drummed"/"bigger"/"keeping" aus den englischen Lösungswörtern.
    "en_wordnet": ("https://wordnetcode.princeton.edu/3.0/WNdb-3.0.tar.gz",
                   "en_wordnet30.tar.gz"),
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

WORD_RE = re.compile(r"^[A-Z]+$")

# Wie viele Wörter der Häufigkeitsliste höchstens in die Rateworte wandern und
# wie oft ein Wort dafür mindestens vorkommen muss (je Länge). Die Obergrenze
# hält die Listen in der Größenordnung des echten Wordle (~13.000 Rateworte)
# und wirkt gegen Tippfehler/Fremdwörter, die in Untertiteln herumliegen.
FREQ_TOP = 15000
FREQ_MIN = 12
# Lösungswörter: geläufig UND im Wörterbuch. Mehr als 4.000 braucht niemand
# (das echte Wordle hat gut 2.300).
ANSWER_MAX = 4000
ANSWER_MIN_COUNT = 30
# So viele Lösungswörter muss jede Sprache in jeder Länge mindestens haben -
# sonst kämen die Rätsel zu schnell wieder (das Skript meldet einen Fehler).
ANSWER_MIN_TOTAL = 500
# Obergrenze der Rateworte je Länge. Lange Wörter gibt es in stark gebeugten
# Sprachen in riesiger Zahl; mehr als ~19.000 bringen beim Raten kaum etwas,
# machen die Web-Dateien aber groß. Gekürzt wird nach Häufigkeit - Einträge
# des Wörterbuchs, die in echtem Text nie vorkommen, fallen zuerst weg.
ALLOWED_MAX = 19000

# Wörter, die zwar im Wörterbuch stehen und in Untertiteln häufig sind, als
# Lösung aber nichts taugen: Ausrufe, Lautmalerei, Umgangssprache,
# Abkürzungen. Als Rateworte bleiben sie erlaubt.
ANSWER_BLOCK = {
    "en": {
        4: set("""ANAL ANUS BARF BONG BOOB BUTT COKE CRAP DAGO DAMN DOPE DORK
            DRUG DYKE FART GIMP GOOK HASH HELL HICK HUMP HUNK KIKE KNOB LAME
            MICK NERD NUDE OKAY PIMP POOP PORN PUKE PUSS PUTZ SCUM SEXY SHAG
            SLAG SLUT SPIC SUCK TOSS TURD TUSH WEED YEAH NOPE WHOA OOPS PHEW
            BOOZE HOMO NAZI TITS WANK JIZZ ARSE SEXT PERV COON SPAZ SMUT
            DOPY LULU TOKE SCAG PEEN SNOG""".split()),
        6: set("""BOTTOM BREAST BUGGER CONDOM COOLIE CRAPPY CRETIN DIMWIT
            EUNUCH FLOOZY HARLOT HEROIN HOOKER HOOKUP INCEST JUNKIE MOLEST
            NIPPLE ORGASM SEDUCE SEXISM SEXIST SEXUAL SODOMY STONER TAMPON
            URINAL UTERUS VAGINA WHITEY RETARD WANKER BIMBO BONER PECKER
            HORNY KINKY SKANKY SCREWY SHUCKS WEENIE HICKEY WHOOPS YIPPEE
            HOORAY DAMMIT BOOBIE TITTIE PISSED FARTED HUMPED""".split()),
        7: set("""BLOWJOB BUTTOCK COLORED CRACKER CRAPPER CUCKOLD GODDAMN
            HOMEBOY JACKASS LESBIAN PERVERT REDNECK REDSKIN SCHMUCK SCROTUM
            SUICIDE TROLLOP WETBACK ALRIGHT DUMBASS BITCHES BASTARD HOOKERS
            WHOREDOM SHITTER PISSING ORGASMS STRIPPER CONDOMS BOOBIES
            HOOTERS DICKISH""".split()),
    },
}
# Das Gleiche für alle anderen Sprachen, unabhängig von der Länge: Wörter, die
# swear.py nicht als Schimpfwort führt, die als Rätsel-Lösung in einer
# Spielesammlung für jedes Alter aber nichts verloren haben (Sex, Drogen,
# Fäkal- und Kindersprache, abwertende Wörter). "*" gilt für jede Sprache
# außer Englisch, das seine eigene Liste oben hat. Wörter, die in einer
# anderen Sprache harmlos sind (tschechisch BUNDA = Jacke, schwedisch KAKA =
# Kuchen, norwegisch GODE = gute), stehen nur bei der Sprache, in der sie
# anstößig sind.
ADULT_BLOCK = {
    "*": """ANAL ANUS CACA PIPI POPO SEXY SEKS SEKSI PENIS VAGINA DILDO PORNO
        PORNOS ORGIE ORGASM ORGASME ORGASMO EROTIK EROTIKA ONANI ONANIE LOLITA
        PETTING KONDOM CONDOM TAMPON SPERM SPERMA SPERME ESPERMA KOKAIN
        COCAINA HEROIN LESBE LEZBA TISS TISSI""",
    "de": "PUFF SCHWUL",
    "fr": "BITE CONNE GODE GOUINE PEDE PISSE TRAVELO ZIZI",
    "es": "BURDEL CHOCHO CONO CULO PENE PITO POLLAS SEMEN TETA",
    "pt": "BUNDA BUNDAO CHOCHO CONO PEITOS PITO SEMEN",
    "pl": "BURDEL CIOTA CIPKA FIUT PEDZIO SIKI SUKA ZBOK",
    "tr": "KAKA SEMEN SIKI",
    "no": "PISSE",
    "sv": "BAJS",
    "fi": "KAKKA PIPPELI",
    "hr": "CIPKA DUPE GUZA JEBO SISE",
}
ADULT_BLOCK = {lang: set(words.split()) for lang, words in ADULT_BLOCK.items()}

# Eigennamen, die in den Wörterbüchern KLEIN stehen (Vornamen, Marken, Orte)
# und deshalb durch die Großschreib-Regel rutschen. Gefunden als Wörter, die
# in mindestens vier anderen Wörterbüchern nur groß vorkommen - von Hand
# gesichtet, echte Wörter ("POISSON", "PRAIA", "MERAK") sind hier NICHT drin.
NAME_BLOCK = {
    "fr": """DORIS JULES BRITISH ROBERT CLAUDE SALOME DAPHNE RAINER LARSEN
        DUNDEE BAXTER COLOMBO APOLLON FARADAY LEONARD FEDERER MOLIERE BENIN
        BLAIR CAMUS GOGOL MOORE TOMMY VICHY""",
    "es": """JANE ARGOS CELIA CUZCO DELOS DIEGO GUIDO TOURS CAMILO GARCIA
        HELENA LUGANO ORTEGA TERESA CARACAS CLAUDIA ENRIQUE GAETANO GALILEO
        HERODES SALOMON""",
    "pt": """PETE ZOLA ALGER BELEM CHILE DALAI IRENE JESSE LIVIA MAGDA MARIO
        MIAMI NEPAL OMAHA PALAU PAULO PETRA RODOS SOFIA ZAIRE ANGOLA BRONTE
        ELVIRA EMILIO HELENE ISABEL MIGUEL NANTES OFELIA RAQUEL SILVIA TORINO
        TREVOR ALABAMA ALGARVE ANTONIO DOPPLER JAMAICA MARCONI MATILDA
        MUSTAFA ONTARIO RICARDO ROBERTO SANTANA SENEGAL SUMATRA VANESSA
        VICENTE""",
    "pl": """BACH BOHR CLIO HERA JUNG SONY ZEUS ABDUL ARIEL BAUER BOSCH BUICK
        DACIA DEBRA FELIX HADES KODAK KONGO KREML LEMAN LENNY MACAO MAREK
        NIKKI NOKIA RINGO ROSIE SAMOS TEXAS WANDA ALBERT AUSTIN BOGDAN CHANEL
        DAKOTA DAMIEN DANIEL DONALD FABIAN HARLEY HELMUT JACOBS KANADA MENDEL
        MORRIS NEPTUN PIETRO RWANDA SUSSEX ARIZONA AUGUSTA BENTLEY BERNARD
        FERRARI HALIFAX HYUNDAI LAMBERT LIBERIA PANDORA PICASSO THOMSON""",
    "tr": """AMAL BLUM EMIL ERIS IRAK KAIN ALICE ALLAH ALLAN ATARI DENIS ELLEN
        KAYLA MALAK MEHDI SINAI SUSAN TARIK TOKYO VINCI BALKAN BRAHMA MARLEY
        PALLAS URSULA SABRINA""",
    "da": """ARNE JENS BIDEN DARIO DAVID KLAUS SARAH SKYPE TIMES CAESAR GRAMMY
        LUCIUS MAGNUS NORMAN SABINA SHARON""",
    "no": """HOFF PHIL RUDI TESS HILDA MINSK REGINA STELLA TURING""",
    "sv": """ERNA KAJA LARA LARS SVEN TURK ESSEN HELGA KAREN OSKAR AFRIKA
        EUROPA EUROPE HAROLD MULLER HARVARD OLYMPIA""",
    "fi": """JANIS JANNE KOREA MESSI PAULA POLLY ARABIA BOSNIA ITALIA SERBIA
        ALBANIA GEORGIA VIETNAM""",
    "cs": """ABBA ANTON JASON SELMA TURIN AMELIE FORBES PACKARD""",
    "sl": """RIGA UTAH ZITA BORIS BRUCE DAVIS HENRI HIRAM LOCKE VOGEL ZORAN
        SELENA STEVEN TIRANA ROMANOV""",
    "hr": """ELISA PLATON WINDOWS""",
}
NAME_BLOCK = {lang: set(words.split()) for lang, words in NAME_BLOCK.items()}

# Echte, geläufige Wörter der jeweiligen Sprache, die einer der Filter sonst
# fälschlich aussortieren würde - von Hand gesichtet:
# * gleich geschriebene englische Wörter (siehe english_leaks): deutsch
#   TASTE/HANG/LAST/LISTEN, französisch STORE/BRIBE/RIDE, tschechisch MORE
#   (moře), dänisch/norwegisch FROM/LAST/REST, türkisch CARE (çare) ...
# * Einträge, die das Wörterbuch nur als Wortbaustein (NEEDAFFIX: deutsch
#   OPERN, GREIF, STREU) oder "nicht vorschlagen" (NOSUGGEST: portugiesisch
#   SOBRE, EXTRA) führt, und deutsche Substantive, die woanders als Name
#   groß stehen (HESSE, PRIME).
# Sie werden trotzdem nur Lösungswort, wenn sie auch im Wörterbuch stehen und
# in echtem Text geläufig genug sind.
ANSWER_KEEP = {
    "de": """HANG HELL HOLD LAST MADE MOST PART SOLD TALK
        APART FINAL HUMAN ORDER RINGS SMART SOLID START TASTE THESE VITAL
        ADELS BOOTS GREIF HASEL HESSE OPERN PRIME QUELL STREU TURBO VOLKS
        WERKS WOLFS
        ARTIST CHARGE FORMAL GENIUS LETTER LIQUID LISTEN POLICE RELIEF REPORT
        ROTTEN WANDER
        BILLION FORTUNE HOLDING PASSION""",
    "fr": """BEAT BORE CASE GAVE LAND LEGS LIED ONCE PLOT REAL RIDE TORE
        BRIBE GRIEF LUNCH NOISE PLAIN PLANT PROBE SLAVE STORE TRUST
        CORNER DECADE LABOUR LONGER MASTER NOTICE OFFICE REPORT SINGLE
        SOVIET
        HOLDING MEETING""",
    "es": """PLUS FELON NOVEL MASTER SOVIET HOLDING""",
    "pt": """CUTE FILE TEAR TIME
        EXTRA LABOR MENTO SHORT SOBRE SUPER USUAL
        CHARGE""",
    "pl": """BOSS BURY GRAB HOLD HURT IDEA LAND LIFT PITY REST SORT SPOT TRAP
        CRAWL HABIT PIANO RUMOR TRUST
        AGENDA CHORUS DESIGN INFANT KILLER MANUAL PARDON POSTER PROFIT RELIEF
        BOWLING DANCING HOLDING""",
    "tr": """BUST CARE FILE FIRE HAIL MEAL NAME PILE SLIP SORT STEP STOP TALK
        MAJOR START
        BITTER MASTER POLICE ROMANS
        HOLDING""",
    "da": """BACK FILE FROM KIND LAST LIKE LINE LIST MADE MORE MOST PART RATE
        REST SAME SPOT STEP TOLD TRUE
        EVENT FRONT HUMAN LEGAL SERVE TASTE TRUST VISIT
        ARREST ARTIST BORING MOMENT PARDON POLICE PROPER
        HOLDING""",
    "no": """FELL FROM HAVE HOPE LAST LINE LIST LOVE MAKE MORE MOST NOTE PART
        REST RIDE SAME TURN
        FRONT HAVEN HUMAN SERVE STILL TASTE TRUST
        FASTER MOMENT SPIRIT""",
    "sv": """AREA BACK BALL BEST FLAT FROM HARM KIND LOTS MESS MUST PART POEM
        REAL RUIN
        DRESS EVENT FAVOR FRONT GIVEN HUMAN LEGAL PLANT SERVE STARE TOAST
        TRUCK TRUST VISIT
        ANIMAL ARREST ASSIST MOMENT PARDON PROPER
        INITIAL""",
    "fi": """HOME INFO INTO SAKE SIDE DESIGN SENIOR""",
    "cs": """BOSS DUTY EXIT FAIR HOLD HOLY IDEA JURY MISS MORE RING SOFA SPOT
        STEP
        CIVIL HABIT NOTES ORBIT RELAX START TRUST
        NORMAL POLICE RELIEF REPORT
        BENEFIT HOLDING JUSTICE PARKING""",
    "sl": """GLAD LAST LOVE STEP
        FRONT HUMAN PIANO RIVAL START
        MOMENT POLICE RELIEF SPIRIT
        HOLDING""",
    "hr": """GRAB MORE PLOT STEP
        FLUID FRONT HUMAN START TRUST
        AGENDA ARTIST EDITOR MOMENT SPIRIT
        HOLDING""",
}
ANSWER_KEEP = {lang: set(words.split()) for lang, words in ANSWER_KEEP.items()}

# Englisch ohne Original-Liste: nur die geläufigsten Lösungswörter (das
# echte Wordle kommt mit ~2.300 Wörtern aus; darunter wird es obskur).
ANSWER_MAX_LANG = {("en", 4): 1500, ("en", 6): 2600, ("en", 7): 2600}


def fold(word, lang, lengths=LENGTHS):
    """Schreibt 'word' in Großbuchstaben A-Z um - oder None.

    None heißt: nach der Umschrift bleiben andere Zeichen als A-Z übrig oder
    die Länge gehört nicht zu 'lengths'.
    """
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
    if len(word) not in lengths or not WORD_RE.match(word):
        return None
    return word


def file_names(length):
    """(Lösungswörter-Datei, Rateworte-Datei) für eine Wortlänge."""
    if length == 5:
        return "answers.txt", "allowed.txt"
    return f"answers{length}.txt", f"allowed{length}.txt"


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
# Deutsche Wörter, die woanders zufällig als Name groß stehen (Paul Klee ...).
DE_NOT_NAMES = {"KLEE", "FRIEDE", "LEINE", "HAUCH", "OSTEN", "NORDEN",
                "WESTEN", "SUEDEN", "HANDEL", "KOHL", "LAUS", "BAYER",
                "UNGAR", "TEDDY", "SEIDEL", "COGNAC", "MARS", "VENUS"}
# Endungen, an denen man im Deutschen eine vollständige Wortform erkennt
# (Mehrzahl, Genitiv, gebeugtes Verb) - im Gegensatz zum nackten Stamm.
DE_WORD_ENDINGS = ("EN", "ER", "ES", "E")


class AffixInfo:
    """Die paar Angaben aus der .aff-Datei, die für die Listen zählen.

    Hunspell markiert Einträge mit Sonder-Flags, die man beim Einlesen der
    .dic kennen muss:

    * FORBIDDENWORD  - ausdrücklich verbotene Schreibweise -> kein Wort.
    * NEEDAFFIX      - Stamm, der nur mit Endung ein Wort ist -> kein Wort.
    * NOSUGGEST      - gültig, aber nie vorschlagen (vulgär, Slang, fremde
      Schreibweisen) -> als Ratewort erlaubt, als Lösung nicht. Das fängt
      z.B. tschechische und schwedische Kraftausdrücke, die der
      Schimpfwortfilter nicht kennt.

    ONLYINCOMPOUND ("nur in Zusammensetzungen") wird bewusst NICHT
    ausgewertet: Deutsch und Schwedisch markieren damit auch ganz normale
    Wörter ("apfel/Sozm" neben "Apfel/Smij", "beige/XZ").

    Flags können einzelne Zeichen, Zeichenpaare (``FLAG long``), Zahlen
    (``FLAG num``) oder Verweise auf eine Alias-Tabelle (``AF``) sein.
    """

    def __init__(self, path=None):
        self.kind = "char"
        self.aliases = []
        self.skip = set()
        self.needaffix = set()
        self.nosuggest = set()
        if not path:
            return
        named = {}
        with open(path, "r", encoding="utf-8", errors="replace") as f:
            for line in f:
                parts = line.split()
                if len(parts) < 2:
                    continue
                key, value = parts[0], parts[1]
                if key == "FLAG":
                    self.kind = value.lower()
                elif key in ("FORBIDDENWORD", "NEEDAFFIX", "NOSUGGEST"):
                    named[key] = value
                elif key == "AF":
                    # Erste AF-Zeile = Anzahl der Einträge, danach die Aliase.
                    if value.isdigit() and not self.aliases and "count" not in named:
                        named["count"] = value
                    else:
                        self.aliases.append(value)
        for key in ("FORBIDDENWORD", "NEEDAFFIX"):
            if key in named:
                self.skip.add(named[key])
        self.needaffix = {named["NEEDAFFIX"]} if "NEEDAFFIX" in named else set()
        if "NOSUGGEST" in named:
            self.nosuggest.add(named["NOSUGGEST"])

    def flags(self, text):
        """Zerlegt den Flag-Teil eines .dic-Eintrags in einzelne Flags."""
        if self.aliases and text.isdigit():
            idx = int(text) - 1
            text = self.aliases[idx] if 0 <= idx < len(self.aliases) else ""
        if self.kind == "long":
            return {text[i:i + 2] for i in range(0, len(text) - 1, 2)}
        if self.kind == "num":
            return {p for p in text.split(",") if p}
        return set(text)


def read_dictionary(lang, cache, lengths=LENGTHS):
    """Wörterbuch-Einträge als (alle Wörter, für Lösungen taugliche Wörter).

    Die zweite Menge lässt Eigennamen weg: in den meisten Sprachen stehen sie
    groß im Wörterbuch und fliegen schon beim Einlesen raus - im Deutschen, wo
    jedes Substantiv groß steht, entscheiden stattdessen die Deklinations-Flags.
    Beide Mengen enthalten Wörter aller gewünschten Längen gemischt.
    """
    words = set()
    good = set()
    keep = ANSWER_KEEP.get(lang, set())
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
                folded = fold((form.text or "").strip(), lang, lengths)
                if folded:
                    words.add(folded)
                    if not proper:
                        good.add(folded)
            elem.clear()
        good |= keep & words
        return words, good

    aff = AffixInfo(download(f"{_DIC}/{DIC_NAME[lang]}/index.aff",
                             f"{lang}.aff", cache))
    path = download(f"{_DIC}/{DIC_NAME[lang]}/index.dic", f"{lang}.dic", cache)
    de_affix = set()                       # nur Deutsch, siehe unten
    de_caps = {}
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
            folded = fold(stem, lang, lengths)
            if not folded:
                continue
            flag_set = aff.flags(flags)
            if folded in keep:
                # Von Hand geprüftes echtes Wort (siehe ANSWER_KEEP).
                words.add(folded)
                good.add(folded)
                continue
            # Verbotene Formen und Stämme ohne Endung: für sich kein Wort.
            if flag_set & aff.skip:
                if lang == "de" and flag_set & aff.needaffix:
                    # igerman98 führt hier Verbstämme ("brech", "trink") UND
                    # Mehrzahl-/Fugenformen von Substantiven ("augen",
                    # "noten") - letztere werden unten gerettet.
                    de_affix.add(folded)
                continue
            words.add(folded)
            if flag_set & aff.nosuggest and not (lang == "de"
                                                 and {"i", "j"} <= flag_set):
                # Gültig, aber keine Lösung: vulgär, alte Rechtschreibung
                # ("nochmal"). igerman98 setzt das Flag allerdings auch bei
                # Substantiven wie "Song/Smnij" oder "Subjekt/EPSmnij" -
                # die erkennt man daran, dass sie Zusammensetzungen beginnen.
                continue
            if lang != "de":
                good.add(folded)
            elif stem[:1].isupper():
                de_caps.setdefault(folded, set()).update(flag_set)
            elif "o" not in flag_set:
                # Kleingeschriebene Einträge mit "o" (nur in
                # Zusammensetzungen) sind Doppel großgeschriebener Wörter
                # ("apfel/Sozm" neben "Apfel/Smij", aber auch "basel/Sozm"
                # neben "Basel/Sm") - über die entscheidet der große Eintrag.
                good.add(folded)
    # Deutsch: großgeschriebene Einträge. Mit Deklinations-Flags ("Blume/Nm")
    # oder als Anfang von Zusammensetzungen ("Apfel/Smij") ist es ein
    # Substantiv. Nur "S" ("Petra/S") heißt Eigenname. Dazwischen liegen
    # Einträge wie "Album/Sm" und "Basel/Sm", die sich in den Flags nicht
    # unterscheiden - dort hilft der Blick in die anderen Wörterbücher: steht
    # das Wort dort nur groß ("Basel", "Kongo", "Japan"), ist es ein Name.
    if de_caps:
        names = foreign_names(cache, lengths, "de")
        for word, flag_set in de_caps.items():
            if flag_set & DE_NOUN_FLAGS or flag_set & {"i", "j"}:
                good.add(word)
            elif "m" in flag_set and word not in names:
                good.add(word)
    # Deutsch: Formen mit Beugungs-Endung sind echte Wörter (AUGEN, OHREN,
    # TAGES, ZEIGE) - nackte Verbstämme wie BRECH, TRINK, STECK nicht.
    for word in de_affix:
        if word.endswith(DE_WORD_ENDINGS):
            words.add(word)
            good.add(word)
    return words, good


_names_cache = {}


def foreign_names(cache, lengths=LENGTHS, lang="de"):
    """Eigennamen aus den Wörterbüchern der ANDEREN Sprachen (ohne 'lang').

    Außer im Deutschen schreibt ein Hunspell-Wörterbuch nur Namen groß. Ein
    Wort, das in mindestens zwei davon groß steht - und öfter groß als klein
    ("Basel", "Kongo", "Japan", "Sudan") -, gilt als Name. Ein einzelnes
    Wörterbuch reicht nicht: das tschechische kennt z.B. viele deutsche
    Nachnamen ("Hecht", "Mittag"), die im Deutschen ganz normale Wörter sind.
    """
    key = (lang, tuple(lengths))
    if key in _names_cache:
        return _names_cache[key]
    caps, lower = {}, {}
    for other in DIC_NAME:
        if other == lang or other == "de":     # Deutsch schreibt alles groß
            continue
        path = download(f"{_DIC}/{DIC_NAME[other]}/index.dic", f"{other}.dic",
                        cache)
        with open(path, "r", encoding="utf-8", errors="replace") as f:
            next(f, None)
            for line in f:
                stem = line.strip().split("\t")[0].partition("/")[0]
                folded = fold(stem, other, lengths) if stem else None
                if not folded:
                    continue
                seen = caps if stem[:1].isupper() else lower
                seen.setdefault(folded, set()).add(other)
    keep = DE_NOT_NAMES if lang == "de" else set()
    _names_cache[key] = {w for w, where in caps.items()
                         if len(where) >= 2 and len(where) > len(lower.get(w, ()))
                         and w not in keep}
    return _names_cache[key]


def read_wordnet(cache, lengths=LENGTHS):
    """Grundformen (Lemmata) aus WordNet 3.0 - nur Englisch.

    WordNet führt jedes Wort genau einmal in seiner Grundform ("drum", "keep",
    "morning", "wedding") und kennt keine reinen Beugungsformen ("drummed",
    "keeping" als Verbform, "bigger"). Als Filter für englische Lösungswörter
    hält das Vergangenheitsformen, Plurale und Steigerungen fern.
    """
    url, name = EXTRA_SOURCES["en_wordnet"]
    path = download(url, name, cache)
    lemmas = set()
    with tarfile.open(path, "r:gz") as tar:
        for member in tar.getmembers():
            base = member.name.rsplit("/", 1)[-1]
            if base not in ("index.noun", "index.verb", "index.adj", "index.adv"):
                continue
            data = tar.extractfile(member).read().decode("utf-8", "replace")
            for line in data.splitlines():
                if not line or line.startswith(" "):
                    continue              # Lizenz-Kopf beginnt mit Leerzeichen
                folded = fold(line.split(" ", 1)[0], "en", lengths)
                if folded:
                    lemmas.add(folded)
    return lemmas


_freq_totals = {}


def read_frequencies(lang, cache, lengths=LENGTHS):
    """Häufigkeitsliste als dict wort -> Anzahl (nur die gewünschten Längen).

    Nebenbei wird die Summe ALLER Wörter der Liste gemerkt (frequency_total) -
    unabhängig von den gewählten Längen, damit relative Häufigkeiten bei
    ``--lengths 5`` dieselben sind wie bei einem vollen Lauf.
    """
    path = download(f"{_FREQ}/{lang}/{lang}_full.txt", f"{lang}_freq.txt", cache)
    counts = {}
    total = 0
    with open(path, "r", encoding="utf-8", errors="replace") as f:
        for line in f:
            parts = line.split()
            if len(parts) != 2 or not parts[1].isdigit():
                continue
            total += int(parts[1])
            folded = fold(parts[0], lang, lengths)
            if folded:
                # Mehrere Schreibweisen können auf dasselbe Wort fallen
                # (z.B. "für"/"fur") - die Häufigkeiten addieren sich.
                counts[folded] = counts.get(folded, 0) + int(parts[1])
    _freq_totals[lang] = max(1, total)
    return counts


def frequency_total(lang):
    """Summe aller Wörter der Häufigkeitsliste (nach read_frequencies)."""
    return _freq_totals.get(lang, 1)


def read_list(key, cache, lang="en", lengths=LENGTHS):
    """Eine Wortliste aus EXTRA_SOURCES (ein Wort je Zeile) als Menge."""
    url, name = EXTRA_SOURCES[key]
    path = download(url, name, cache)
    with open(path, "r", encoding="utf-8", errors="replace") as f:
        return {w for w in (fold(x.strip(), lang, lengths) for x in f) if w}


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
_english = {}


def english_reference(cache, lengths=LENGTHS):
    """(ENABLE-Wörter, englische Häufigkeiten) - einmal geladen, geteilt.

    Untertitel und manche Wörterbücher enthalten englische Wörter ("GOOD",
    "SHOT", "YARD" im Französischen). Die taugen nicht als Lösung, wenn sie
    im Englischen viel geläufiger sind als in der Sprache selbst.
    """
    key = tuple(lengths)
    if key not in _english:
        enable = read_list("en_enable", cache, lengths=lengths)
        counts = read_frequencies("en", cache, lengths)
        _english[key] = (enable, counts, frequency_total("en"))
    return _english[key]


def english_leaks(src, words):
    """Wörter aus 'words', die eher Englisch als Sprache 'src.lang' sind.

    Maßstab ist die relative Häufigkeit in den Untertiteln: steht ein Wort in
    der englischen ENABLE-Liste und kommt im Englischen mehr als 25-mal so
    oft vor wie in der eigenen Sprache, ist es ein Mitbringsel. (Lehnwörter
    wie PIANO, DRAMA, PIZZA sind in beiden Sprachen ähnlich häufig und bleiben.)
    Gleich geschriebene echte Wörter der Sprache (TASTE, STORE, MORE ...)
    stehen in ANSWER_KEEP und werden nicht angefasst.
    """
    if src.english is None:
        return set()
    enable, en_counts, en_total = src.english
    total = frequency_total(src.lang)
    keep = ANSWER_KEEP.get(src.lang, set())
    leaks = set()
    for w in words:
        if w in enable and w not in keep:
            own = src.counts.get(w, 0) / total
            other = en_counts.get(w, 0) / en_total
            if other > 25 * own:
                leaks.add(w)
    return leaks


class Sources:
    """Alle Quellen EINER Sprache - einmal gelesen, für alle Längen genutzt."""

    def __init__(self, lang, cache, lengths):
        print(f"  {lang}: Quellen lesen ...")
        self.lang = lang
        self.cache = cache
        self.dictionary, self.solvable = read_dictionary(lang, cache, lengths)
        self.counts = read_frequencies(lang, cache, lengths)
        self.is_clean = clean_filter()
        self.english = None
        self.leaks = {}                    # Länge -> entfernte Englisch-Wörter
        if lang != "en":
            self.english = english_reference(cache, lengths)
        if lang == "en":
            self.nyt_answers = read_list("en_answers", cache, lengths=(5,))
            self.nyt_allowed = read_list("en_allowed", cache, lengths=(5,))
            self.enable = read_list("en_enable", cache, lengths=lengths)
            self.wordnet = read_wordnet(cache, lengths=lengths)


def build(src, length):
    """Baut (Lösungswörter, Rateworte) einer Sprache für eine Wortlänge."""
    lang, counts = src.lang, src.counts
    dictionary = {w for w in src.dictionary if len(w) == length}
    solvable = {w for w in src.solvable if len(w) == length}
    freq = {w: n for w, n in counts.items() if len(w) == length}

    # Rateworte: alles aus dem Wörterbuch + die geläufigsten Formen aus echtem
    # Text. Die Häufigkeitsliste bringt die gebeugten Formen mit, die in einem
    # Wörterbuch nur als Grundform stehen (HAUSE, MEINE, KOMMT ...).
    common = sorted((w for w, n in freq.items() if n >= FREQ_MIN),
                    key=lambda w: -freq[w])[:FREQ_TOP]
    allowed = set(dictionary) | set(common)

    # Lösungswörter: geläufig UND im Wörterbuch (ohne Eigennamen) - das hält
    # Tippfehler, Namen und Wortfetzen ("DOESN", "WEREN") aus den Lösungen.
    answers = sorted((w for w in solvable
                      if freq.get(w, 0) >= ANSWER_MIN_COUNT),
                     key=lambda w: -freq[w])

    if lang == "en" and length == 5:
        # Englisch bekommt die echten Wordle-Listen: exakt die Lösungswörter
        # der New York Times, dazu deren erlaubte Rateworte. Die
        # Untertitel-Häufigkeiten bleiben hier außen vor - sie brächten vor
        # allem Namen und Tippfehler ("RYOGA", "PEOPL") in die Rateworte.
        answers = sorted(src.nyt_answers)
        allowed = set(dictionary) | src.nyt_allowed | src.nyt_answers
    elif lang == "en":
        # Andere Längen: keine Original-Listen. Rateworte = Wörterbuch +
        # ENABLE (echte Wörter samt Beugungen, ohne Namen und Tippfehler aus
        # den Untertiteln). Lösungswörter müssen zusätzlich in ENABLE stehen
        # (wirft Abkürzungen und Umgangssprache des Wörterbuchs raus) und in
        # WordNet als Grundform vorkommen (keine "drummed"/"bigger"/"cats").
        enable = {w for w in src.enable if len(w) == length}
        allowed = set(dictionary) | enable
        answers = [w for w in answers if w in enable and w in src.wordnet]

    block = ANSWER_BLOCK.get(lang, {}).get(length, set()) | NAME_BLOCK.get(lang, set())
    if lang != "en":
        block = block | ADULT_BLOCK["*"] | ADULT_BLOCK.get(lang, set())
    answers = [w for w in answers if w not in block and src.is_clean(w)]
    leaks = english_leaks(src, answers)
    if leaks:
        answers = [w for w in answers if w not in leaks]
        src.leaks[length] = sorted(leaks)
    if not (lang == "en" and length == 5):
        # Die häufigsten zuerst - die Obergrenze schneidet die seltenen ab.
        answers = sorted(answers, key=lambda w: -freq.get(w, 0))
        answers = answers[:ANSWER_MAX_LANG.get((lang, length), ANSWER_MAX)]
    allowed = {w for w in allowed if src.is_clean(w)}
    allowed |= set(answers)                # Lösungswörter sind immer erlaubt

    if len(allowed) > ALLOWED_MAX:
        # Zu viele Rateworte (lange Wörter in stark gebeugten Sprachen):
        # die Lösungswörter bleiben, dann nach Häufigkeit auffüllen.
        keep = set(answers)
        rest = sorted(allowed - keep, key=lambda w: (-freq.get(w, 0), w))
        allowed = keep | set(rest[:ALLOWED_MAX - len(keep)])
    return sorted(set(answers)), sorted(allowed)


def write_list(path, words):
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(words) + "\n")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[3])
    parser.add_argument("langs", nargs="*", default=None,
                        help=f"Sprachen (Standard: alle {len(LANGS)})")
    parser.add_argument("--lengths", nargs="+", type=int, default=list(LENGTHS),
                        choices=LENGTHS, help="Wortlängen (Standard: 4 5 6 7)")
    parser.add_argument("--cache", default=CACHE_DIR,
                        help="Ordner für die heruntergeladenen Quellen")
    parser.add_argument("--out", default=OUT_DIR, help="Zielordner")
    parser.add_argument("--sample", type=int, default=0,
                        help="je Sprache/Länge so viele zufällige Lösungswörter "
                             "zur Sichtprüfung ausgeben")
    args = parser.parse_args(argv)

    langs = args.langs or LANGS
    unknown = [l for l in langs if l not in LANGS]
    if unknown:
        parser.error(f"unbekannte Sprache(n): {', '.join(unknown)}")
    lengths = sorted(set(args.lengths))

    print(f"Wortlisten bauen -> {args.out} (Längen {', '.join(map(str, lengths))})")
    summary = []
    too_small = []
    for lang in langs:
        src = Sources(lang, args.cache, lengths)
        folder = os.path.join(args.out, lang)
        os.makedirs(folder, exist_ok=True)
        row = []
        for length in lengths:
            answers, allowed = build(src, length)
            ans_name, all_name = file_names(length)
            write_list(os.path.join(folder, ans_name), answers)
            write_list(os.path.join(folder, all_name), allowed)
            row.append((len(answers), len(allowed)))
            print(f"  {lang}/{length}: {len(answers)} Lösungswörter, "
                  f"{len(allowed)} Rateworte")
            if len(answers) < ANSWER_MIN_TOTAL:
                too_small.append(f"{lang}/{length} ({len(answers)})")
            if args.sample:
                if src.leaks.get(length):
                    print("      englisch, keine Lösung: "
                          + " ".join(src.leaks[length]))
                rng = random.Random(f"{lang}{length}")
                pick = sorted(rng.sample(answers, min(args.sample, len(answers))))
                print("      Stichprobe: " + " ".join(pick))
        summary.append((lang, row))

    head = " | ".join(f"{n} Buchstaben" for n in lengths)
    print(f"\n| Sprache | {head} |")
    print("|---------|" + "|".join("---" for _ in lengths) + "|")
    for lang, row in summary:
        cells = " | ".join(f"{a} / {b}" for a, b in row)
        print(f"| {lang} | {cells} |")
    if too_small:
        print(f"\nFEHLER: zu wenige Lösungswörter (< {ANSWER_MIN_TOTAL}): "
              + ", ".join(too_small))
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
