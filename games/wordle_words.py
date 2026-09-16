# -*- coding: utf-8 -*-
"""
wordle_words.py
===============
Wortlisten für das Wordle-Spiel, je Sprache und Wortlänge (4 bis 7).

- Die Listen liegen als Textdateien im Ordner ``woordlistz/`` im Projektordner::

      woordlistz/<sprache>/answers.txt    Lösungswörter mit 5 Buchstaben
      woordlistz/<sprache>/allowed.txt    alle erlaubten Rateworte (Obermenge)
      woordlistz/<sprache>/answers4.txt   ... und dasselbe für 4, 6 und 7
      woordlistz/<sprache>/allowed4.txt       Buchstaben

  Erzeugt werden sie von ``woordlistz/build_wordlists.py`` aus echten
  Wörterbüchern und Häufigkeitslisten - siehe ``woordlistz/README.md``.
- Alle Wörter verwenden nur A-Z (keine Umlaute/Akzente), damit sie mit einer
  schlichten A-Z-Bildschirmtastatur eingegeben werden können. Umlaute und
  Akzente sind je Sprache umgeschrieben (deutsch Ä->AE, dänisch Å->AA, sonst
  Akzent weg).
- ``words_for(lang, length)`` liefert die Lösungswörter (alphabetisch - das
  Tageswort hängt an dieser Reihenfolge, die Web-Version sortiert genauso),
  ``allowed_for(lang, length)`` die Menge aller erlaubten Rateworte. Beides
  wird einmal je Sprache und Länge geladen und gecacht; ein robuster Filter
  wirft stray Einträge (falsche Länge/Zeichen) heraus, statt das Spiel zu
  stören.
- Fehlt der Ordner (oder eine Sprache darin), greift für 5 Buchstaben die
  eingebaute Notfallliste ``FALLBACK`` - so läuft das Spiel auch dann, wenn
  eine ältere .exe ohne die Wortlisten gebaut wurde. Für die anderen Längen
  gibt es keine Notfallliste; ``has_length()`` sagt dem Spiel, ob es die
  Länge anbieten kann.
"""

import os
import sys

# Ordner mit den Wortlisten: normalerweise <projekt>/woordlistz. In einer mit
# PyInstaller gebauten .exe liegt alles entpackt in sys._MEIPASS - dort steht
# woordlistz\ dank --add-data direkt daneben.
_HERE = os.path.dirname(os.path.abspath(__file__))
_BASES = [os.path.join(os.path.dirname(_HERE), "woordlistz")]
if getattr(sys, "_MEIPASS", None):
    _BASES.insert(0, os.path.join(sys._MEIPASS, "woordlistz"))

# Wortlängen, die das Spiel anbietet (5 = klassisches Wordle).
LENGTHS = (4, 5, 6, 7)
DEFAULT_LENGTH = 5

# Notfall-Lösungswörter, falls die Dateien fehlen (kurze kuratierte Listen).
FALLBACK = {
    "de": [
        "HAUSE", "TISCH", "STUHL", "LAMPE", "APFEL", "BIRNE", "PFERD", "KATZE",
        "MAUER", "WOLKE", "REGEN", "SONNE", "STERN", "BLUME", "BLATT", "BAUER",
        "TASSE", "KANNE", "GABEL", "HONIG", "ROSEN", "TULPE", "HECKE", "BUSCH",
        "FLUSS", "INSEL", "STADT", "BERGE", "WIESE", "BUCHE", "EICHE", "TANNE",
        "LINDE", "AHORN", "ZWEIG", "KRONE", "RINDE", "BEERE", "GURKE", "PILZE",
        "BOHNE", "ERBSE", "LINSE", "NUDEL", "SUPPE", "TORTE", "KEKSE", "SAHNE",
        "QUARK", "ESSIG", "CHILI", "CURRY", "REISE", "NADEL", "FADEN", "WOLLE",
        "STOFF", "HOSEN", "JACKE", "SOCKE", "RINGE", "KETTE", "PERLE", "EISEN",
        "STAHL", "STEIN", "FARBE", "SEITE", "ZEILE", "WORTE", "BRIEF", "KARTE",
        "STIFT", "TINTE", "TAFEL", "KREIS", "ECKEN", "KANTE", "LINIE", "PUNKT",
        "SUMME", "REGEL", "PROBE", "MONDE", "NEBEL", "STURM", "BLITZ", "FROST",
        "WINDE", "FEUER", "ASCHE", "KOHLE", "RAUCH", "DAMPF", "FUNKE", "LICHT",
        "MILCH", "HAFER", "KRAUT", "SPECK", "WURST", "STEAK", "GRILL", "HERDE",
        "SALAT", "PIZZA", "KAKAO", "MOKKA", "LATTE",
    ],
    "en": [
        "APPLE", "BREAD", "CHAIR", "TABLE", "HOUSE", "MOUSE", "LIGHT", "NIGHT",
        "WATER", "EARTH", "PLANT", "STONE", "RIVER", "OCEAN", "BEACH", "CLOUD",
        "STORM", "SUNNY", "HAPPY", "ANGRY", "QUIET", "BRAVE", "SMART", "QUICK",
        "SWEET", "SPICY", "FRESH", "GREEN", "BROWN", "BLACK", "WHITE", "GRAPE",
        "LEMON", "MELON", "PEACH", "BERRY", "HONEY", "SUGAR", "FLOUR", "DOUGH",
        "PASTA", "PIZZA", "SALAD", "JUICE", "DRINK", "GLASS", "PLATE", "SPOON",
        "KNIFE", "CLOTH", "SHIRT", "PANTS", "SHOES", "SOCKS", "DRESS", "SCARF",
        "GLOVE", "WATCH", "RINGS", "CHAIN", "PEARL", "METAL", "STEEL", "BRICK",
        "PAPER", "PAINT", "BRUSH", "CHALK", "BOARD", "POINT", "ANGLE", "ROUND",
        "HEART", "SMILE", "LAUGH", "DREAM", "SLEEP", "AWAKE", "HORSE", "SHEEP",
        "GOOSE", "TIGER", "ZEBRA", "PANDA", "KOALA", "SNAKE", "EAGLE", "ROBIN",
        "WHALE", "SHARK", "TROUT", "GRASS", "BLOOM", "PETAL", "THORN", "FRUIT",
        "MAPLE", "BIRCH", "CEDAR", "ROCKS", "SANDY", "FIELD", "CANDY", "MONEY",
        "MUSIC", "PIANO", "DRAMA", "STAGE", "NOVEL", "STORY", "WORDS", "LINES",
    ],
    "fr": [
        "TABLE", "LIVRE", "PORTE", "ARBRE", "FLEUR", "PLAGE", "NUAGE", "ORAGE",
        "PLUIE", "NEIGE", "TERRE", "MONDE", "ROUTE", "VILLE", "OCEAN", "GRAIN",
        "POMME", "POIRE", "MELON", "SUCRE", "PIZZA", "VERRE", "NAPPE", "VESTE",
        "GANTS", "BAGUE", "PERLE", "ACIER", "CRAIE", "LIGNE", "POINT", "CARRE",
        "COEUR", "TIGRE", "ZEBRE", "PANDA", "AIGLE", "HERBE", "EPINE", "FRUIT",
        "VIGNE", "CHIEN", "LOUPS", "CHATS", "BLEUE", "VERTE", "NOIRE", "ROUGE",
        "JAUNE", "BRUNE", "NUITS", "MATIN", "HIVER", "LUNDI", "MARDI", "AMOUR",
        "AMIES", "PERES", "MERES", "HEURE", "ANNEE", "PLACE", "SALLE", "MAINS",
        "PIEDS", "TETES", "DENTS", "JOUES", "LEVRE", "GORGE", "DOIGT", "POUCE",
        "GENOU", "TALON", "PIANO", "DANSE", "CHANT", "SCENE", "DRAME", "ROMAN",
        "CONTE",
    ],
    "es": [
        "SILLA", "LIBRO", "ARBOL", "PLAYA", "NIEVE", "MUNDO", "GRANO", "LIMON",
        "MELON", "FRESA", "PASTA", "PIZZA", "PLATO", "FALDA", "PERLA", "ACERO",
        "PAPEL", "LINEA", "PUNTO", "CARRO", "SUENO", "OVEJA", "PERRO", "TIGRE",
        "CEBRA", "PANDA", "TRIGO", "FRUTA", "PARRA", "NOCHE", "TARDE", "LUNES",
        "VERDE", "NEGRO", "AMIGO", "PADRE", "MADRE", "NINOS", "FELIZ", "LENTO",
        "DULCE", "CIELO", "FUEGO", "CALOR", "RITMO", "PIANO", "CANTO", "BAILE",
        "DRAMA", "TEXTO", "GATOS", "PATOS", "OSITO", "LOBOS", "PECES", "HOJAS",
        "MONTE", "VALLE", "CAMPO", "PRADO", "NUBES", "SOLES", "MARES", "ARENA",
        "ROCAS", "BARCO", "COCHE", "AVION", "CALLE", "PLAZA", "TORRE", "MUROS",
        "TECHO", "SUELO", "MESAS", "CAMAS", "SOFAS", "VELAS", "ROSAS",
    ],
    "pt": [
        "LIVRO", "PORTA", "PRAIA", "NUVEM", "CHUVA", "TERRA", "MUNDO", "LIMAO",
        "MELAO", "FRUTA", "PASTA", "PIZZA", "PRATO", "PAPEL", "LINHA", "PONTO",
        "CARRO", "SONHO", "TIGRE", "ZEBRA", "PANDA", "TRIGO", "VERDE", "PRETO",
        "NOITE", "TARDE", "MANHA", "VENTO", "CALOR", "RITMO", "PIANO", "VIOLA",
        "CANTO", "DANCA", "DRAMA", "CAMPO", "MONTE", "AMIGO", "PONTE", "FESTA",
        "LEITE", "PEIXE", "CARNE", "ARROZ", "SALSA", "MOLHO", "VINHO", "MASSA",
        "FORNO", "FOGAO", "GATOS", "PATOS", "LOBOS", "FOLHA", "PEDRA", "VALES",
        "PRADO", "MARES", "AREIA", "ROCHA", "BARCO", "AVIAO", "PRACA", "TORRE",
        "MUROS", "CAMAS", "SOFAS", "VELAS", "ROSAS", "RELVA", "LAGOA", "ILHAS",
        "AGUAS", "NEVOA",
    ],
}

_cache = {}          # (sprache, länge) -> (lösungswörter, erlaubte wörter)


def _valid(word, length=DEFAULT_LENGTH):
    """True für genau 'length' Großbuchstaben A-Z."""
    return len(word) == length and word.isascii() and word.isalpha()


def file_names(length):
    """(Lösungswörter, Rateworte) als Dateinamen ohne Endung.

    5 Buchstaben behalten die alten Namen (answers/allowed), die anderen
    Längen bekommen die Zahl angehängt (answers4, allowed7 ...).
    """
    if length == DEFAULT_LENGTH:
        return "answers", "allowed"
    return f"answers{length}", f"allowed{length}"


def _read_list(lang, name, length):
    """Liest woordlistz/<lang>/<name>.txt (leere Liste, wenn es sie nicht gibt)."""
    for base in _BASES:
        path = os.path.join(base, lang, name + ".txt")
        try:
            with open(path, "r", encoding="utf-8") as f:
                out = []
                seen = set()
                for line in f:
                    word = line.strip().upper()
                    if _valid(word, length) and word not in seen:
                        seen.add(word)
                        out.append(word)
            if out:
                return out
        except OSError:
            continue
    return []


def _load(lang, length=DEFAULT_LENGTH):
    """(Lösungswörter, Menge der erlaubten Rateworte) einer Sprache/Länge."""
    key = (lang, length)
    if key in _cache:
        return _cache[key]
    ans_name, all_name = file_names(length)
    answers = _read_list(lang, ans_name, length)
    allowed = set(_read_list(lang, all_name, length))
    if not answers and length == DEFAULT_LENGTH:
        # Keine Dateien: Notfallliste dieser Sprache, sonst die englische.
        answers = [w for w in FALLBACK.get(lang, FALLBACK["en"]) if _valid(w)]
    # Sortiert, damit das Tageswort (Index in dieser Liste) am PC und im
    # Browser dasselbe ist - die Dateien sind es ohnehin schon.
    answers = sorted(answers)
    allowed.update(answers)
    _cache[key] = (answers, allowed)
    return _cache[key]


def words_for(lang, length=DEFAULT_LENGTH):
    """Lösungswörter (groß geschrieben, nur A-Z, alphabetisch, ohne Dubletten)."""
    return _load(lang, length)[0]


def allowed_for(lang, length=DEFAULT_LENGTH):
    """Menge aller erlaubten Rateworte - enthält immer die Lösungswörter."""
    return _load(lang, length)[1]


def has_length(lang, length):
    """True, wenn es für Sprache und Länge Lösungswörter gibt."""
    return bool(words_for(lang, length))


def is_allowed(lang, word):
    """True, wenn 'word' als Rateversuch zugelassen ist (Länge = Wortlänge)."""
    word = word.upper()
    return word in allowed_for(lang, len(word))


def counts_for(lang, length=DEFAULT_LENGTH):
    """(Anzahl Lösungswörter, Anzahl Rateworte) - für Anzeige/Tests."""
    answers, allowed = _load(lang, length)
    return len(answers), len(allowed)
