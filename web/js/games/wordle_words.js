/*
 * wordle_words.js - Wortlisten für Wordle, je Sprache und Wortlänge
 * (Port von games/wordle_words.py)
 * =================================================================
 * - Die eigentlichen Listen liegen in js/games/wordle_words/<code><n>.js
 *   (5 Buchstaben ohne Zahl: de.js, sonst de4.js, de6.js, de7.js) und werden
 *   aus woordlistz/ der Desktop-Version erzeugt
 *   (node web/tools/build-wordlists.js). Zusammen sind das mehrere Megabyte,
 *   deshalb wird immer nur die Datei der gerade gespielten Sprache/Länge
 *   nachgeladen - per <script>-Tag, damit es auch über file:// läuft.
 * - Jede dieser Dateien meldet sich mit PG.wordleWords.add(code, lösungen,
 *   weitere, länge) an. Die Wörter stehen dort ohne Trennzeichen
 *   hintereinander; "weitere" sind die erlaubten Rateworte, die KEINE
 *   Lösungswörter sind (die Lösungen werden beim Anmelden ergänzt).
 * - Bis eine Liste da ist (oder wenn die Datei fehlt), greift für 5 Buchstaben
 *   die kurze eingebaute Notfallliste FALLBACK - so ist das Spiel nie ohne
 *   Wörter. Für 4/6/7 gibt es keine Notfallliste (hasLength() sagt es).
 *
 *   PG.wordleWords.load(code, n, fertig)   Liste nachladen
 *   PG.wordleWords.isLoaded(code, n)       liegt sie schon bereit?
 *   PG.wordleWords.wordsFor(code, n)       Lösungswörter (Array, alphabetisch)
 *   PG.wordleWords.allowedFor(code, n)     erlaubte Rateworte (Set)
 *   PG.wordleWords.hasLength(code, n)      gibt es Lösungswörter?
 */
(function () {
  "use strict";

  const LENGTHS = [4, 5, 6, 7];
  const DEFAULT_LENGTH = 5;

  // Notfall-Lösungswörter, falls die Sprachdatei fehlt (gleiche Liste wie
  // FALLBACK in games/wordle_words.py).
  const FALLBACK = {
    de: [
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
    en: [
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
    fr: [
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
    es: [
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
    pt: [
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
  };

  const DATA = {}; // "code:n" -> {answers: [...], allowed: Set}
  const WAITING = {}; // "code:n" -> [callback, ...] (Ladevorgang läuft)
  const FAILED = {}; // "code:n" -> true (Datei fehlt)

  // Ordner der Sprachdateien - aus dem Pfad DIESER Datei abgeleitet, damit es
  // sowohl aus index.html als auch aus tools/smoketest.html stimmt.
  const BASE = (function () {
    const src = (document.currentScript && document.currentScript.src) || "";
    return src ? src.replace(/wordle_words\.js(\?.*)?$/, "wordle_words/") : "js/games/wordle_words/";
  })();

  function slot(code, n) {
    return code + ":" + (n || DEFAULT_LENGTH);
  }

  /** Zerlegt "ABCDEFGHIJ" in ["ABCDE", "FGHIJ"] (n Zeichen je Wort). */
  function split(packed, n) {
    const out = [];
    for (let i = 0; i + n <= packed.length; i += n) out.push(packed.slice(i, i + n));
    return out;
  }

  /** Wird von js/games/wordle_words/<code><n>.js aufgerufen. */
  function add(code, answers, extra, n) {
    n = n || DEFAULT_LENGTH;
    const list = split(String(answers || ""), n).sort();
    const set = new Set(split(String(extra || ""), n));
    for (const w of list) set.add(w);
    DATA[slot(code, n)] = { answers: list, allowed: set };
  }

  function isLoaded(code, n) {
    return !!DATA[slot(code, n)];
  }

  /** Lädt die Wortliste einer Sprache/Länge nach und ruft danach 'done' auf. */
  function load(code, n, done) {
    const key = slot(code, n);
    if (DATA[key] || FAILED[key]) return done && done();
    if (WAITING[key]) {
      if (done) WAITING[key].push(done);
      return;
    }
    WAITING[key] = done ? [done] : [];
    const finish = () => {
      const waiting = WAITING[key];
      delete WAITING[key];
      for (const cb of waiting) cb();
    };
    const s = document.createElement("script");
    s.src = BASE + code + (n === DEFAULT_LENGTH ? "" : String(n)) + ".js";
    s.onload = finish;
    s.onerror = () => {
      console.warn("[PyGameZ] Wordle-Wortliste fehlt:", s.src);
      FAILED[key] = true;
      finish();
    };
    document.head.appendChild(s);
  }

  /** Lösungswörter (Notfallliste für 5 Buchstaben, solange nichts geladen ist). */
  function wordsFor(code, n) {
    n = n || DEFAULT_LENGTH;
    const d = DATA[slot(code, n)];
    if (d) return d.answers;
    if (n !== DEFAULT_LENGTH) return [];
    return (FALLBACK[code] || FALLBACK.en).filter((w) => /^[A-Z]{5}$/.test(w)).sort();
  }

  /** Erlaubte Rateworte der Sprache/Länge als Set. */
  function allowedFor(code, n) {
    const d = DATA[slot(code, n)];
    if (d) return d.allowed;
    return new Set(wordsFor(code, n));
  }

  function hasLength(code, n) {
    return wordsFor(code, n).length > 0;
  }

  PG.wordleWords = { add, load, isLoaded, wordsFor, allowedFor, hasLength, FALLBACK, LENGTHS, DEFAULT_LENGTH };
})();
