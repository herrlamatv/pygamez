/*
 * lamacrossing_logic.js - Regeln von Lama Crossing (1:1 aus games/lamacrossing_logic.py)
 * =======================================================================================
 * Das Lama überquert Spur für Spur eine Straße, eine Schlucht, einen Fluss oder
 * einen Eissee. Jede Spur hält mit Wahrscheinlichkeit 1 - p und hebt den
 * Multiplikator; ein Fehltritt kostet den Einsatz. Dazu kommen sichtbare
 * Hindernisse (Autos, Felsbrocken, Treibholz, Eisblöcke), denen man per Timing
 * ausweichen muss (TRAFFIC, hitTest). Auszahlen geht nach jeder geschafften Spur.
 *
 * Multiplikatoren stehen als Ganzzahlen in Hundertsteln fest in den Tabellen
 * (m_k = floor(0,97 / (1 - p)^k * 100)) - in Python, JS und Rust bitgleich.
 * Jede Strategie "bei Spur k auszahlen" bringt so ~97 % des Einsatzes zurück.
 * tests/audit_lamacrossing.py vergleicht beide Fassungen
 * (über web/tools/lamacrossing_logic_dump.js).
 */
(function () {
  "use strict";

  const DIFFS = ["easy", "medium", "hard", "extreme"];
  const WORLDS = ["road", "bridge", "river", "ice"];
  // Risiko je Schritt als Bruch (Zähler, Nenner)
  const RISK = { easy: [1, 12], medium: [1, 6], hard: [3, 10], extreme: [1, 2] };
  // Multiplikator (x100) nach k geschafften Schritten, k = 1 .. Anzahl Schritte
  const TABLES = {
    easy: [105, 115, 125, 137, 149, 163, 178, 194, 212, 231, 252, 275, 300,
      327, 357, 390, 425, 464, 506, 552, 603, 657],
    medium: [116, 139, 167, 201, 241, 289, 347, 417, 500, 600, 720, 864,
      1037, 1245, 1494, 1793, 2152, 2582, 3098, 3718],
    hard: [138, 197, 282, 403, 577, 824, 1177, 1682, 2403, 3433, 4905, 7008,
      10011, 14302, 20431, 29187, 41697, 59567],
    extreme: [194, 388, 776, 1552, 3104, 6208, 12416, 24832, 49664, 99328,
      198656, 397312, 794624, 1589248, 3178496],
  };
  // Verkehr je Stufe: [v_lo, v_hi] Tempo der Hindernisse in px/s bei 800x600
  // (wird mit k skaliert), [gap_lo, gap_hi] Abstand zweier Hindernisse einer
  // Spur in Sekunden. Jede Spur hat ein festes Tempo (kein Überholen).
  const TRAFFIC = {
    easy: [240, 340, 1.8, 3.2],
    medium: [300, 420, 1.4, 2.8],
    hard: [360, 500, 1.1, 2.3],
    extreme: [440, 620, 0.9, 1.9],
  };
  // Tempo-Faktor je Welt (Treibholz/Eisblöcke kommen vom Horizont her)
  const SPEED_F = { road: 1.0, bridge: 1.0, river: 0.4, ice: 0.4 };
  // Raser bei einem Zufalls-Fehltritt auf der Straße: v_hi * RASER_F
  const RASER_F = 1.6;
  const BETS = [10, 20, 50, 100, 200, 500, 1000, 2500, 5000, 10000];
  // Ab diesem Multiplikator (x100) gibt es den Erfolg "Nervenstark"
  const BRAVE_MULT = 1000;
  // "Drahtseilakt": auf Extrem bei mindestens x100 ausgezahlt
  const EXTREME_MULT = 10000;
  const HISTORY = 12;

  const isInt = (v) => typeof v === "number" && Number.isInteger(v);

  /** Anzahl der Schritte bis zum anderen Ufer. */
  function steps(diff) {
    return TABLES[diff].length;
  }
  /** Fehltritt-Wahrscheinlichkeit je Schritt als Gleitkommazahl. */
  function risk(diff) {
    const [num, den] = RISK[diff];
    return num / den;
  }
  /** Multiplikator (x100) nach k geschafften Schritten (k = 0 -> 100). */
  function mult(diff, k) {
    if (k <= 0) return 100;
    const table = TABLES[diff];
    return table[Math.min(k, table.length) - 1];
  }
  /** Auszahlung bei k geschafften Schritten (abgerundet auf ganze Chips). */
  function payout(bet, diff, k) {
    return Math.floor((bet * mult(diff, k)) / 100);
  }
  /** Ein Schritt: true = hält, false = Fehltritt (genau ein random()-Aufruf). */
  function roll(rng, diff) {
    return !(rng.random() < risk(diff));
  }
  /** Trifft ein Hindernis (Mitte obsY, senkrechte Länge obsLen) das Lama,
   *  dessen Mitte auf targetY liegt (reach = halbe Trefferhöhe des Lamas)? */
  function hitTest(obsY, obsLen, targetY, reach) {
    return Math.abs(obsY - targetY) < obsLen / 2 + reach;
  }
  /** Risiko je Schritt in ganzen Prozent (gerundet, für die Anzeige). */
  function riskPercent(diff) {
    const [num, den] = RISK[diff];
    return Math.floor((num * 200 + den) / (den * 2));
  }
  /** Multiplikator x100 als Text: 235 -> "2.35" (Trennzeichen je Sprache). */
  function fmtMult(m100, sep = ".") {
    return Math.floor(m100 / 100) + sep + String(m100 % 100).padStart(2, "0");
  }
  /** Erwartungswert (Anteil am Einsatz) der Strategie "bei k auszahlen". */
  function expectedReturn(diff, k) {
    const [num, den] = RISK[diff];
    let hold = 1.0;
    for (let i = 0; i < k; i++) hold *= (den - num) / den;
    return (hold * mult(diff, k)) / 100.0;
  }
  /** Gespeicherte, noch laufende Runde prüfen: {bet > 0, diff, step 1 .. Schritte-1} oder null. */
  function cleanRound(raw) {
    if (!raw || typeof raw !== "object" || Array.isArray(raw)) return null;
    const { bet, diff, step } = raw;
    if (!DIFFS.includes(diff) || !isInt(bet) || !isInt(step)) return null;
    if (bet <= 0 || step < 1 || step >= steps(diff)) return null;
    return { bet, diff, step };
  }
  /** Verlauf der letzten Runden: Liste von Multiplikatoren x100 (0 = verloren). */
  function cleanHistory(raw) {
    if (!Array.isArray(raw)) return [];
    return raw.filter((v) => isInt(v) && v >= 0).slice(-HISTORY);
  }

  PG.lamaCrossingLogic = {
    DIFFS, WORLDS, RISK, TABLES, TRAFFIC, SPEED_F, RASER_F, BETS, BRAVE_MULT, EXTREME_MULT, HISTORY,
    steps, risk, mult, payout, roll, hitTest, riskPercent, fmtMult, expectedReturn, cleanRound, cleanHistory,
  };
})();
