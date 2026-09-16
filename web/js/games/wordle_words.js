/*
 * wordle_words.js - Wortlisten für Wordle, je Sprache
 * (Port von games/wordle_words.py)
 * ===================================================
 * - Die eigentlichen Listen liegen in js/games/wordle_words/<code>.js und
 *   werden aus woordlistz/ der Desktop-Version erzeugt
 *   (node web/tools/build-wordlists.js). Zusammen sind das mehrere Megabyte,
 *   deshalb wird immer nur die Datei der gerade gespielten Sprache
 *   nachgeladen - per <script>-Tag, damit es auch über file:// läuft.
 * - Jede dieser Dateien meldet sich mit PG.wordleWords.add(code, ...) an. Die
 *   Wörter stehen dort ohne Trennzeichen hintereinander (je 5 Zeichen).
 * - Bis eine Liste da ist (oder wenn die Datei fehlt), greift die kurze
 *   eingebaute Notfallliste FALLBACK - so ist das Spiel nie ohne Wörter.
 *
 *   PG.wordleWords.load(code, fertig)   Liste der Sprache nachladen
 *   PG.wordleWords.isLoaded(code)       liegt sie schon bereit?
 *   PG.wordleWords.wordsFor(code)       Lösungswörter (Array)
 *   PG.wordleWords.allowedFor(code)     erlaubte Rateworte (Set)
 */
(function () {
  "use strict";

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

  const DATA = {};      // code -> {answers: [...], allowed: Set}
  const WAITING = {};   // code -> [callback, ...] (Ladevorgang läuft)

  // Ordner der Sprachdateien - aus dem Pfad DIESER Datei abgeleitet, damit es
  // sowohl aus index.html als auch aus tools/smoketest.html stimmt.
  const BASE = (function () {
    const src = (document.currentScript && document.currentScript.src) || "";
    return src ? src.replace(/wordle_words\.js(\?.*)?$/, "wordle_words/")
               : "js/games/wordle_words/";
  })();

  /** Zerlegt "ABCDEFGHIJ" in ["ABCDE", "FGHIJ"]. */
  function split5(packed) {
    const out = [];
    for (let i = 0; i + 5 <= packed.length; i += 5) out.push(packed.slice(i, i + 5));
    return out;
  }

  /** Wird von js/games/wordle_words/<code>.js aufgerufen. */
  function add(code, answers, allowed) {
    const list = split5(String(answers || ""));
    const set = new Set(split5(String(allowed || "")));
    for (const w of list) set.add(w);
    DATA[code] = { answers: list, allowed: set };
  }

  function isLoaded(code) {
    return !!DATA[code];
  }

  /** Lädt die Wortliste einer Sprache nach und ruft danach 'done' auf. */
  function load(code, done) {
    if (DATA[code]) return done && done();
    if (WAITING[code]) {
      if (done) WAITING[code].push(done);
      return;
    }
    WAITING[code] = done ? [done] : [];
    const finish = () => {
      const waiting = WAITING[code];
      delete WAITING[code];
      for (const cb of waiting) cb();
    };
    const s = document.createElement("script");
    s.src = BASE + code + ".js";
    s.onload = finish;
    s.onerror = () => {
      console.warn("[PyGameZ] Wordle-Wortliste fehlt:", s.src);
      finish();
    };
    document.head.appendChild(s);
  }

  /** Lösungswörter der Sprache (Notfallliste, solange nichts geladen ist). */
  function wordsFor(code) {
    if (DATA[code]) return DATA[code].answers;
    return (FALLBACK[code] || FALLBACK.en).filter((w) => /^[A-Z]{5}$/.test(w));
  }

  /** Erlaubte Rateworte der Sprache als Set. */
  function allowedFor(code) {
    if (DATA[code]) return DATA[code].allowed;
    return new Set(wordsFor(code));
  }

  PG.wordleWords = { add, load, isLoaded, wordsFor, allowedFor, FALLBACK };
})();
