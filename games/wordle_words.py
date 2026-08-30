# -*- coding: utf-8 -*-
"""
wordle_words.py
===============
Kuratierte Lösungswörter für das Wordle-Spiel, je Sprache.

- Alle Wörter sind genau 5 Buchstaben lang und verwenden nur A-Z (keine
  Umlaute/Akzente), damit sie mit einer schlichten A-Z-Bildschirmtastatur
  eingegeben werden können.
- Als reines Python-Modul (keine externe Datei), damit die Listen auch in einem
  mit PyInstaller gebauten .exe sicher mitgebündelt sind - genauso wie
  maze_gen.py / sudoku_gen.py.
- ``words_for(lang)`` liefert eine gefilterte, groß geschriebene Liste; ein
  robuster Filter wirft stray Einträge (falsche Länge/Zeichen) heraus, statt
  das Spiel zu stören. Fehlt eine Sprache, wird auf Englisch zurückgegriffen.
"""

WORDS = {
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
        "EISIG", "WINDE", "FEUER", "ASCHE", "KOHLE", "RAUCH", "DAMPF", "FUNKE",
        "LICHT", "HELLE", "GRAUE", "BLAUE", "MILCH", "HAFER", "KRAUT", "SPECK",
        "WURST", "STEAK", "GRILL", "HERDE", "SALAT", "PIZZA", "KAKAO", "MOKKA",
        "LATTE", "WODKA",
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
        "MAPLE", "BIRCH", "CEDAR", "ROCKS", "SANDY", "FIELD", "VALLEY", "MOUNT",
        "CANDY", "MONEY", "MUSIC", "PIANO", "DRAMA", "STAGE", "NOVEL", "STORY",
        "WORDS", "LINES",
    ],
    "fr": [
        "TABLE", "LIVRE", "PORTE", "ARBRE", "FLEUR", "PLAGE", "NUAGE", "ORAGE",
        "PLUIE", "NEIGE", "TERRE", "MONDE", "ROUTE", "VILLE", "OCEAN", "GRAIN",
        "POMME", "POIRE", "MELON", "SUCRE", "PIZZA", "VERRE", "NAPPE", "VESTE",
        "GANTS", "BAGUE", "PERLE", "ACIER", "CRAIE", "LIGNE", "POINT", "CARRE",
        "COEUR", "TIGRE", "ZEBRE", "PANDA", "AIGLE", "HERBE", "EPINE", "FRUIT",
        "VIGNE", "CHIEN", "LOUPS", "RENARD", "SOURIS", "CHATS", "BLEUE", "VERTE",
        "NOIRE", "ROUGE", "JAUNE", "BRUNE", "NUITS", "MATIN", "MIDIS", "HIVER",
        "LUNDI", "MARDI", "AMOUR", "AMIES", "PERES", "MERES", "ENFANT", "HEURE",
        "ANNEE", "MOISI", "PLACE", "SALLE", "MAINS", "PIEDS", "TETES", "DENTS",
        "YEUXX", "NEZZZ", "JOUES", "LEVRE", "LANGUE", "GORGE", "DOIGT", "POUCE",
        "GENOU", "CHEVILLE", "EPAULE", "TALON", "MUSIC", "PIANO", "DANSE", "CHANT",
        "SCENE", "DRAME", "ROMAN", "CONTE", "MOTSS", "PHRASE",
    ],
    "es": [
        "SILLA", "LIBRO", "ARBOL", "PLAYA", "NIEVE", "MUNDO", "GRANO", "LIMON",
        "MELON", "FRESA", "PASTA", "PIZZA", "PLATO", "FALDA", "PERLA", "ACERO",
        "PAPEL", "LINEA", "PUNTO", "CARRO", "SUENO", "OVEJA", "PERRO", "TIGRE",
        "CEBRA", "PANDA", "TRIGO", "FRUTA", "PARRA", "NOCHE", "TARDE", "LUNES",
        "VERDE", "NEGRO", "AMIGO", "PADRE", "MADRE", "NINOS", "FELIZ", "LENTO",
        "DULCE", "CIELO", "FUEGO", "CALOR", "RITMO", "PIANO", "CANTO", "BAILE",
        "DRAMA", "TEXTO", "GATOS", "PATOS", "OSITO", "LOBOS", "PECES", "AVES",
        "FLORE", "HOJAS", "RAICES", "PIEDRA", "MONTE", "VALLE", "CAMPO", "PRADO",
        "NUBES", "SOLES", "ESTRELLA", "MARES", "OLASS", "ARENA", "ROCAS", "BARCO",
        "COCHE", "TRENE", "AVION", "PUENTE", "CALLE", "PLAZA", "TORRE", "MUROS",
        "TECHO", "SUELO", "MESAS", "CAMAS", "SOFAS", "LAMPARA", "VELAS", "FUENTE",
        "JARDIN", "ROSAS", "TULIPAN",
    ],
    "pt": [
        "LIVRO", "PORTA", "PRAIA", "NUVEM", "CHUVA", "TERRA", "MUNDO", "LIMAO",
        "MELAO", "FRUTA", "PASTA", "PIZZA", "PRATO", "PAPEL", "LINHA", "PONTO",
        "CARRO", "SONHO", "TIGRE", "ZEBRA", "PANDA", "TRIGO", "VERDE", "PRETO",
        "NOITE", "TARDE", "MANHA", "VENTO", "CALOR", "RITMO", "PIANO", "VIOLA",
        "CANTO", "DANCA", "DRAMA", "CAMPO", "MONTE", "AMIGO", "PONTE", "FESTA",
        "LEITE", "PEIXE", "CARNE", "ARROZ", "SALSA", "MOLHO", "VINHO", "MASSA",
        "FORNO", "FOGAO", "GATOS", "PATOS", "LOBOS", "AVESS", "FLORE", "FOLHA",
        "RAIZES", "PEDRA", "VALES", "PRADO", "NUVEN", "SOLIS", "MARES", "AREIA",
        "ROCHA", "BARCO", "COMBOIO", "AVIAO", "RUAS", "PRACA", "TORRE", "MUROS",
        "TETO", "CHAO", "CAMAS", "SOFAS", "VELAS", "JARDIM", "ROSAS", "RELVA",
        "OCEANO", "RIOSS", "LAGOA", "ILHAS", "AGUAS", "GELOO", "NEVOA", "TROVAO",
    ],
}


def words_for(lang):
    """Groß geschriebene, auf gültige 5-Buchstaben-A-Z-Wörter gefilterte Liste."""
    raw = WORDS.get(lang) or WORDS.get("en", [])
    out = []
    seen = set()
    for w in raw:
        u = w.strip().upper()
        if len(u) == 5 and u.isascii() and u.isalpha() and u not in seen:
            seen.add(u)
            out.append(u)
    if not out:
        out = [w for w in WORDS["en"] if len(w) == 5]
    return out
