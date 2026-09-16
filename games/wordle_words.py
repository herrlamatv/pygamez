# -*- coding: utf-8 -*-
"""
wordle_words.py
===============
Wortlisten für das Wordle-Spiel, je Sprache.

- Die Listen liegen als Textdateien im Ordner ``woordlistz/`` im Projektordner::

      woordlistz/<sprache>/answers.txt   Lösungswörter (geläufige Wörter)
      woordlistz/<sprache>/allowed.txt   alle erlaubten Rateworte (Obermenge)

  Erzeugt werden sie von ``woordlistz/build_wordlists.py`` aus echten
  Wörterbüchern und Häufigkeitslisten - siehe ``woordlistz/README.md``.
- Alle Wörter sind genau 5 Buchstaben lang und verwenden nur A-Z (keine
  Umlaute/Akzente), damit sie mit einer schlichten A-Z-Bildschirmtastatur
  eingegeben werden können. Umlaute und Akzente sind je Sprache umgeschrieben
  (deutsch Ä->AE, dänisch Å->AA, sonst Akzent weg).
- ``words_for(lang)`` liefert die Lösungswörter, ``allowed_for(lang)`` die
  Menge aller erlaubten Rateworte. Beides wird einmal je Sprache geladen und
  gecacht; ein robuster Filter wirft stray Einträge (falsche Länge/Zeichen)
  heraus, statt das Spiel zu stören.
- Fehlt der Ordner (oder eine Sprache darin), greift die eingebaute
  Notfallliste ``FALLBACK`` - so läuft das Spiel auch dann, wenn eine ältere
  .exe ohne die Wortlisten gebaut wurde.
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

_cache = {}          # sprache -> (lösungswörter, erlaubte wörter)


def _valid(word):
    """True für genau 5 Großbuchstaben A-Z."""
    return len(word) == 5 and word.isascii() and word.isalpha()


def _read_list(lang, name):
    """Liest woordlistz/<lang>/<name>.txt (leere Liste, wenn es sie nicht gibt)."""
    for base in _BASES:
        path = os.path.join(base, lang, name + ".txt")
        try:
            with open(path, "r", encoding="utf-8") as f:
                out = []
                seen = set()
                for line in f:
                    word = line.strip().upper()
                    if _valid(word) and word not in seen:
                        seen.add(word)
                        out.append(word)
            if out:
                return out
        except OSError:
            continue
    return []


def _load(lang):
    """(Lösungswörter, Menge der erlaubten Rateworte) für eine Sprache."""
    if lang in _cache:
        return _cache[lang]
    answers = _read_list(lang, "answers")
    allowed = set(_read_list(lang, "allowed"))
    if not answers:
        # Keine Dateien: Notfallliste dieser Sprache, sonst die englische.
        answers = [w for w in FALLBACK.get(lang, FALLBACK["en"]) if _valid(w)]
    allowed.update(answers)
    _cache[lang] = (answers, allowed)
    return _cache[lang]


def words_for(lang):
    """Lösungswörter der Sprache (groß geschrieben, nur A-Z, ohne Dubletten)."""
    return _load(lang)[0]


def allowed_for(lang):
    """Menge aller erlaubten Rateworte - enthält immer die Lösungswörter."""
    return _load(lang)[1]


def is_allowed(lang, word):
    """True, wenn 'word' als Rateversuch zugelassen ist."""
    return word.upper() in allowed_for(lang)


def counts_for(lang):
    """(Anzahl Lösungswörter, Anzahl Rateworte) - für Anzeige/Tests."""
    answers, allowed = _load(lang)
    return len(answers), len(allowed)
