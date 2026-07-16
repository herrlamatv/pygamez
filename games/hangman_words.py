# -*- coding: utf-8 -*-
"""
hangman_words.py
================
Ratewoerter fuer das Galgenmaennchen, je Sprache.

- Anders als bei Wordle sind die Woerter unterschiedlich lang (3-12 Buchstaben).
- Nur A-Z (keine Umlaute/Akzente), damit sie mit der A-Z-Bildschirmtastatur
  ratbar sind; deutsche Umlaute/Eszett und Akzente werden dabei bewusst als
  ASCII geschrieben (z.B. STRASSE, MACA).
- Als reines Python-Modul (keine externe Datei), damit die Listen auch in einer
  mit PyInstaller gebauten .exe sicher mitgebuendelt sind - wie wordle_words.py.
- ``words_for(lang, mode)`` liefert eine gefilterte, gross geschriebene Liste;
  ``mode`` waehlt die Laenge: "short" (3-5), "long" (7-12) oder "mixed" (3-12).
"""

WORDS = {
    "de": [
        "HAUS", "BAUM", "TISCH", "STUHL", "LAMPE", "APFEL", "BIRNE", "PFERD",
        "KATZE", "HUND", "MAUER", "WOLKE", "REGEN", "SONNE", "STERN", "BLUME",
        "GARTEN", "FENSTER", "SCHULE", "STRASSE", "KAFFEE", "MILCH", "BROT",
        "KUCHEN", "ZUCKER", "WASSER", "FEUER", "ERDE", "HIMMEL", "MEER",
        "INSEL", "BERG", "WALD", "WIESE", "FLUSS", "KLAVIER", "GITARRE",
        "TROMPETE", "ORCHESTER", "KONZERT", "THEATER", "KINO", "BUCH",
        "ZEITUNG", "BLEISTIFT", "PAPIER", "SCHERE", "MESSER", "GABEL",
        "TELLER", "FLASCHE", "SCHRANK", "SPIEGEL", "COMPUTER", "TELEFON",
        "KAMERA", "FAHRRAD", "AUTO", "SCHIFF", "FLUGZEUG", "RAKETE", "PLANET",
        "KOMET", "ROBOTER", "DRACHE", "RITTER", "SCHLOSS", "PRINZ", "ZAUBERER",
        "GESPENST", "SCHATTEN", "TRAUM", "WINTER", "SOMMER", "HERBST",
        "GEWITTER", "SCHNEE", "DONNER", "BLITZ", "NEBEL", "WIND",
    ],
    "en": [
        "HOUSE", "TREE", "TABLE", "CHAIR", "LAMP", "APPLE", "ORANGE", "HORSE",
        "CAT", "DOG", "MOUNTAIN", "RIVER", "OCEAN", "ISLAND", "FOREST",
        "GARDEN", "WINDOW", "SCHOOL", "STREET", "COFFEE", "MILK", "BREAD",
        "CAKE", "SUGAR", "WATER", "FIRE", "EARTH", "PLANET", "COMET", "GALAXY",
        "ROBOT", "DRAGON", "KNIGHT", "CASTLE", "PRINCE", "PRINCESS", "WIZARD",
        "WITCH", "GHOST", "SHADOW", "DREAM", "WINTER", "SUMMER", "AUTUMN",
        "THUNDER", "LIGHTNING", "RAINBOW", "PIANO", "GUITAR", "TRUMPET",
        "ORCHESTRA", "CONCERT", "THEATRE", "CINEMA", "BOOK", "PENCIL", "PAPER",
        "SCISSORS", "KNIFE", "SPOON", "BOTTLE", "MIRROR", "CLOCK", "RADIO",
        "COMPUTER", "TELEPHONE", "CAMERA", "BICYCLE", "AIRPLANE", "ROCKET",
        "ELEPHANT", "GIRAFFE", "DOLPHIN", "PENGUIN", "BUTTERFLY", "MONKEY",
        "RABBIT", "TIGER", "FLOWER", "BRIDGE",
    ],
    "fr": [
        "MAISON", "ARBRE", "TABLE", "CHAISE", "LAMPE", "POMME", "ORANGE",
        "CHEVAL", "CHAT", "CHIEN", "MONTAGNE", "RIVIERE", "OCEAN", "ILE",
        "FORET", "JARDIN", "FENETRE", "ECOLE", "ROUTE", "CAFE", "LAIT", "PAIN",
        "GATEAU", "SUCRE", "FEU", "TERRE", "PLANETE", "GALAXIE", "ROBOT",
        "DRAGON", "CHEVALIER", "CHATEAU", "PRINCE", "PRINCESSE", "SORCIER",
        "FANTOME", "OMBRE", "REVE", "HIVER", "ETE", "AUTOMNE", "TONNERRE",
        "ECLAIR", "PIANO", "GUITARE", "TROMPETTE", "CONCERT", "THEATRE",
        "CINEMA", "LIVRE", "CRAYON", "PAPIER", "CISEAUX", "COUTEAU",
        "BOUTEILLE", "MIROIR", "HORLOGE", "ORDINATEUR", "TELEPHONE", "CAMERA",
        "VELO", "AVION", "FUSEE", "ELEPHANT", "GIRAFE", "DAUPHIN", "PAPILLON",
        "SINGE", "LAPIN", "TIGRE", "FLEUR", "PONT",
    ],
    "es": [
        "CASA", "ARBOL", "MESA", "SILLA", "LAMPARA", "MANZANA", "NARANJA",
        "CABALLO", "GATO", "PERRO", "MONTANA", "RIO", "OCEANO", "ISLA",
        "BOSQUE", "JARDIN", "VENTANA", "ESCUELA", "CALLE", "CAFE", "LECHE",
        "PAN", "PASTEL", "AZUCAR", "FUEGO", "TIERRA", "PLANETA", "GALAXIA",
        "ROBOT", "DRAGON", "CABALLERO", "CASTILLO", "PRINCIPE", "PRINCESA",
        "MAGO", "FANTASMA", "SOMBRA", "SUENO", "INVIERNO", "VERANO", "OTONO",
        "TRUENO", "RELAMPAGO", "PIANO", "GUITARRA", "TROMPETA", "CONCIERTO",
        "TEATRO", "CINE", "LIBRO", "LAPIZ", "PAPEL", "TIJERAS", "CUCHILLO",
        "BOTELLA", "ESPEJO", "RELOJ", "ORDENADOR", "TELEFONO", "CAMARA",
        "BICICLETA", "AVION", "COHETE", "ELEFANTE", "JIRAFA", "DELFIN",
        "MARIPOSA", "MONO", "CONEJO", "TIGRE", "FLOR", "PUENTE",
    ],
    "pt": [
        "CASA", "ARVORE", "MESA", "CADEIRA", "LAMPADA", "MACA", "LARANJA",
        "CAVALO", "GATO", "CACHORRO", "MONTANHA", "RIO", "OCEANO", "ILHA",
        "FLORESTA", "JARDIM", "JANELA", "ESCOLA", "RUA", "CAFE", "LEITE",
        "PAO", "BOLO", "ACUCAR", "FOGO", "TERRA", "PLANETA", "GALAXIA",
        "ROBO", "DRAGAO", "CAVALEIRO", "CASTELO", "PRINCIPE", "PRINCESA",
        "MAGO", "FANTASMA", "SOMBRA", "SONHO", "INVERNO", "VERAO", "OUTONO",
        "TROVAO", "RELAMPAGO", "PIANO", "VIOLAO", "TROMPETE", "CONCERTO",
        "TEATRO", "CINEMA", "LIVRO", "LAPIS", "PAPEL", "TESOURA", "FACA",
        "GARRAFA", "ESPELHO", "RELOGIO", "COMPUTADOR", "TELEFONE", "CAMERA",
        "BICICLETA", "AVIAO", "FOGUETE", "ELEFANTE", "GIRAFA", "GOLFINHO",
        "BORBOLETA", "MACACO", "COELHO", "TIGRE", "FLOR", "PONTE",
    ],
}

# Laengen-Grenzen je Modus.
_RANGES = {"short": (3, 5), "long": (7, 12), "mixed": (3, 12)}


def words_for(lang, mode="mixed"):
    """Gross geschriebene, auf A-Z und Modus-Laenge gefilterte Wortliste."""
    lo, hi = _RANGES.get(mode, _RANGES["mixed"])
    raw = WORDS.get(lang) or WORDS.get("en", [])
    out, seen = [], set()
    for w in raw:
        u = w.strip().upper()
        if lo <= len(u) <= hi and u.isascii() and u.isalpha() and u not in seen:
            seen.add(u)
            out.append(u)
    if not out:                          # Notnagel: englische Liste, Laenge egal
        out = [w for w in WORDS["en"] if lo <= len(w) <= hi] or list(WORDS["en"])
    return out
