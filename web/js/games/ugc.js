/*
 * ugc.js - Wortfilter und Speicher eigener Inhalte (Port von swear.py und ugc.py)
 * ==============================================================================
 * Gemeinsam genutzt von Minigolf (eigene Bahnen) und Geometry Dash (eigene
 * Level) - vorher steckte beides in minigolf_edit.js.
 *
 *   PG.swear   Wortfilter (Regex-Listen aller 14 Sprachen, unten eingebettet)
 *   PG.ugc     Registry je Art (Format, Endung, Prüfung, Aufräumen, Höchstzahl)
 *              mit Speichern/Laden/Löschen, Export als Text (Download) und
 *              Import (Dateiwahl). PG.ugc.forGame("minigolf") liefert die
 *              gewohnte Schnittstelle einer Art (loadMaps, saveMap, ...).
 *
 * Gespeichert wird unter PG.store "minigolf.ugc", aufgebaut wie ugc.json:
 *   {"v": 1, "author": "Lama", "minigolf": [ {map}, ... ], "geodash": [ {level}, ... ]}
 * (der Schlüsselname bleibt aus Kompatibilität mit vorhandenen Browser-Daten).
 * Eine exportierte Datei enthält genau EINEN Inhalt im Umschlag seiner Art:
 *   {"format": "pygamez.minigolf.map", "v": 1, "app": "PyGameZ", "exported": "...", "map": {...}}
 *   {"format": "pygamez.geodash.level", ..., "level": {...}}
 */
(function () {
  "use strict";

  const PG = window.PG;
  if (PG.ugc && PG.swear) return; // schon geladen (mehrere Spiele im Manifest)

  // ===========================================================================
  //  Wortfilter (swear.py)
  // ===========================================================================
  //
  // Je Sprache die Regex-Muster aus lang/swear/<code>.yml bzw.
  // lang/lang.expansion/swear/<code>.yml. Zum Platzsparen steht "~" für den
  // häufigen Zwischenraum-Baustein "[^\sa-zA-Z]*" (wird beim Laden ersetzt).
  // Geprüft wird IMMER gegen alle 14 Sprachen, nicht nur gegen die
  // eingestellte - sonst umginge man den Filter, indem man kurz die Sprache
  // umstellt.
  const SWEAR_DATA = {
    de: ["\\b([a4@]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*h+)(?=[^\\s]*\\b)","\\b([a4@]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*h+)\\b","\\b([s5$]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b([s5$]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[s5$]+)(?=[^\\s]*\\b)","\\b(f+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[z2]+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b(f+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*k+)(?=[^\\s]*\\b)","\\b(w+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(w+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*n+)(?=[^\\s]*\\b)","\\b(h+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*n+)(?=[^\\s]*\\b)","\\b(h+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[e3]+)\\b","\\b(n+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b([s5$]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*m+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b(m+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[t7+]+)(?=[^\\s]*\\b)","\\b([s5$]+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[i1!|]+)(?=[^\\s]*\\b)","\\b([s5$]+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[t7+]+)\\b","\\b(m+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[o0]+)\\b","\\b(k+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b(n+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)\\b","\\b(n+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(j+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*u+)(?=[^\\s]*\\b)","\\b(n+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[z2]+[^\\sa-zA-Z]*[i1!|]+)\\b","\\b(h+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(h+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[z2]+)(?=[^\\s]*\\b)","\\b([s5$]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*w+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[l1|]+)(?=[^\\s]*\\b)","\\b([s5$]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*w+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[z2]+)\\b","\\b(m+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*[i1!|]+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b([b8]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*d+)(?=[^\\s]*\\b)","\\b(d+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*u+)(?=[^\\s]*\\b)","\\b(v+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b(k+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b(k+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[z2]+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b(v+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*w+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[g9]+)(?=[^\\s]*\\b)","\\b([t7+]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*n+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)"],
    en: ["\\b(f+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*k+)(?=[^\\s]*\\b)","\\b([s5$]+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+)(?=[^\\s]*\\b)","\\b([b8]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*h+)(?=[^\\s]*\\b)","\\b(c+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[t7+]+)(?=[^\\s]*\\b)","\\b([a4@]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b([a4@]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b(d+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*k+)\\b","\\b(d+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*d+)(?=[^\\s]*\\b)","\\b(c+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*k+)\\b","\\b(p+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*y+)(?=[^\\s]*\\b)","\\b(w+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b([b8]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*d+)(?=[^\\s]*\\b)","\\b(n+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(n+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(f+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[t7+]+)(?=[^\\s]*\\b)","\\b(f+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[g9]+)\\b","\\b(r+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*d+)(?=[^\\s]*\\b)","\\b(w+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b([t7+]+[^\\sa-zA-Z]*w+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[t7+]+)\\b","\\b(m+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*f+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*k+)(?=[^\\s]*\\b)","\\b([b8]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[s5$]+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[s5$]+)\\b","\\b(r+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[e3]+)\\b","\\b(r+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[t7+]+)(?=[^\\s]*\\b)","\\b(n+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[z2]+[^\\sa-zA-Z]*[i1!|]+)\\b","\\b(h+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[o0]+)\\b","\\b(j+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*k+[\\W\\d_]*[o0]+[^\\sa-zA-Z]*f+[^\\sa-zA-Z]*f+)\\b","\\b(c+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(d+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*m+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[s5$]+)(?=[^\\s]*\\b)"],
    fr: ["\\b(m+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*n+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[e3]+)\\b","\\b([s5$]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b(c+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*d+)(?=[^\\s]*\\b)","\\b(c+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b([e3]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b([e3]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+)\\b","\\b(c+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b(f+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b([b8]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*d+)(?=[^\\s]*\\b)","\\b(n+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*q+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(n+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*q+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[e3]+)\\b","\\b([t7+]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b([b8]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b(n+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[e3]+)\\b","\\b(c+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(c+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[l1|]+)\\b","\\b([b8]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b([b8]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(n+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[z2]+[^\\sa-zA-Z]*[i1!|]+)\\b","\\b(h+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(v+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(d+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+)\\b","\\b([a4@]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b([b8]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[l1|]+)\\b","\\b([s5$]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*d+)(?=[^\\s]*\\b)","\\b([e3]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*f+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b([t7+]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*c+)(?=[^\\s]*\\b)","\\b(f+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[s5$]+[\\W\\d_]*d+[^\\sa-zA-Z]*[e3]+[\\W\\d_]*p+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)"],
    es: ["\\b(m+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(j+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(p+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[o0]+)\\b","\\b(c+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*n+)(?=[^\\s]*\\b)","\\b([g9]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[s5$]+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(p+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*j+[^\\sa-zA-Z]*[o0]+)(?=[^\\s]*\\b)","\\b(c+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(c+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[o0]+)(?=[^\\s]*\\b)","\\b(m+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*n+)(?=[^\\s]*\\b)","\\b(m+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*[a4@]+)\\b","\\b([z2]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(h+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*j+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(h+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*j+[^\\sa-zA-Z]*[o0]+[\\W\\d_]*d+[^\\sa-zA-Z]*[e3]+[\\W\\d_]*p+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[o0]+)(?=[^\\s]*\\b)","\\b([b8]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[o0]+)(?=[^\\s]*\\b)","\\b(v+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(f+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*[e3]+)\\b","\\b(c+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*j+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[s5$]+)(?=[^\\s]*\\b)","\\b(c+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[o0]+)(?=[^\\s]*\\b)","\\b([s5$]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*m+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[l1|]+)(?=[^\\s]*\\b)","\\b(r+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[o0]+)(?=[^\\s]*\\b)","\\b(n+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(n+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[z2]+[^\\sa-zA-Z]*[i1!|]+)\\b","\\b(h+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(v+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*n+)(?=[^\\s]*\\b)","\\b(m+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*m+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*n+)\\b","\\b(c+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(m+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[o0]+)(?=[^\\s]*\\b)","\\b(c+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[s5$]+)(?=[^\\s]*\\b)","\\b(m+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[o0]+)\\b"],
    pt: ["\\b(m+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(c+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*[o0]+)(?=[^\\s]*\\b)","\\b(f+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(f+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(p+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[o0]+)\\b","\\b(c+[^\\sa-zA-Z]*u+)\\b","\\b([b8]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(v+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[o0]+)\\b","\\b(c+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[o0]+)\\b","\\b(f+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*[o0]+[\\W\\d_]*d+[^\\sa-zA-Z]*[a4@]+[\\W\\d_]*p+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(c+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b([o0]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[o0]+)(?=[^\\s]*\\b)","\\b(r+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[o0]+)(?=[^\\s]*\\b)","\\b(n+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[z2]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(h+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b([e3]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[o0]+)(?=[^\\s]*\\b)","\\b([a4@]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*m+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[o0]+)(?=[^\\s]*\\b)","\\b(d+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[o0]+)(?=[^\\s]*\\b)","\\b(v+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(x+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*x+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b([b8]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b([e3]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[o0]+)(?=[^\\s]*\\b)","\\b([b8]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*u+[\\W\\d_]*n+[^\\sa-zA-Z]*[o0]+[\\W\\d_]*c+[^\\sa-zA-Z]*u+)\\b","\\b(c+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[a4@]+[\\W\\d_]*r+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b([b8]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(c+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[z2]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[o0]+)(?=[^\\s]*\\b)"],
    pl: ["\\b(k+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*w+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(c+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*j+)\\b","\\b(p+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[l1|]+)(?=[^\\s]*\\b)","\\b(j+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*c+)(?=[^\\s]*\\b)","\\b(j+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*y+)(?=[^\\s]*\\b)","\\b([s5$]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*w+[^\\sa-zA-Z]*y+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*y+[^\\sa-zA-Z]*n+)(?=[^\\s]*\\b)","\\b(c+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(d+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(p+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[z2]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[a4@]+)\\b","\\b([s5$]+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*j+)(?=[^\\s]*\\b)","\\b(k+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[s5$]+)(?=[^\\s]*\\b)","\\b(h+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*j+)\\b","\\b([z2]+[^\\sa-zA-Z]*j+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[b8]+)\\b","\\b(d+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[l1|]+)\\b","\\b([g9]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*w+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[o0]+)(?=[^\\s]*\\b)","\\b([s5$]+[^\\sa-zA-Z]*[z2]+[^\\sa-zA-Z]*m+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(d+[^\\sa-zA-Z]*[z2]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*w+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(c+[^\\sa-zA-Z]*w+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[l1|]+)\\b","\\b(p+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[l1|]+)(?=[^\\s]*\\b)","\\b(m+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[z2]+[^\\sa-zA-Z]*y+[^\\sa-zA-Z]*n+)(?=[^\\s]*\\b)","\\b(c+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*y+)(?=[^\\s]*\\b)","\\b(n+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[z2]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(h+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b([g9]+[^\\sa-zA-Z]*w+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[t7+]+)(?=[^\\s]*\\b)","\\b([s5$]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*y+[^\\sa-zA-Z]*n+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*j+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[b8]+)\\b","\\b(w+[^\\sa-zA-Z]*y+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*j+)(?=[^\\s]*\\b)","\\b(j+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(k+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*w+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[z2]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*n+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*y+)(?=[^\\s]*\\b)","\\b(r+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*c+)\\b","\\b([z2]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*j+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[s5$]+)\\b"],
    tr: ["\\b([o0]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*u+)(?=[^\\s]*\\b)","\\b([a4@]+[^\\sa-zA-Z]*m+[^\\sa-zA-Z]*k+)\\b","\\b([a4@]+[^\\sa-zA-Z]*m+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b([s5$]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*k+)\\b","\\b([s5$]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*y+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*m+)(?=[^\\s]*\\b)","\\b([s5$]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*c+)\\b","\\b(y+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*k+)(?=[^\\s]*\\b)","\\b(k+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b([g9]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[t7+]+)(?=[^\\s]*\\b)","\\b([i1!|]+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[e3]+)\\b","\\b([s5$]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*k+)(?=[^\\s]*\\b)","\\b([g9]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[z2]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[i1!|]+)(?=[^\\s]*\\b)","\\b([a4@]+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[l1|]+)\\b","\\b(n+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[z2]+[^\\sa-zA-Z]*[i1!|]+)\\b","\\b(h+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b([t7+]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[z2]+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[t7+]+)\\b","\\b(k+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*k+)(?=[^\\s]*\\b)","\\b([s5$]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*k+)(?=[^\\s]*\\b)","\\b([t7+]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*k+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[z2]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*k+)(?=[^\\s]*\\b)","\\b([s5$]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*k+)(?=[^\\s]*\\b)","\\b([a4@]+[^\\sa-zA-Z]*m+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*k+)(?=[^\\s]*\\b)","\\b([g9]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*n+)(?=[^\\s]*\\b)","\\b(y+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*k+)(?=[^\\s]*\\b)","\\b([s5$]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*f+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[z2]+)\\b","\\b(m+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[l1|]+[\\W\\d_]*h+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*f+)\\b","\\b([a4@]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*y+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*m+)(?=[^\\s]*\\b)","\\b([o0]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*u+)(?=[^\\s]*\\b)"],
    da: ["\\b([l1|]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[t7+]+)\\b","\\b(p+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*k+)\\b","\\b(f+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b(k+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b([l1|]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(r+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[l1|]+)(?=[^\\s]*\\b)","\\b([s5$]+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(m+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[l1|]+)(?=[^\\s]*\\b)","\\b(n+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)\\b","\\b(n+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[z2]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[t7+]+)(?=[^\\s]*\\b)","\\b(h+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(v+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[t7+]+)(?=[^\\s]*\\b)","\\b(m+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[o0]+)(?=[^\\s]*\\b)","\\b(k+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[g9]+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(k+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(k+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*m+[^\\sa-zA-Z]*y+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b(f+[^\\sa-zA-Z]*j+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[s5$]+)\\b","\\b([i1!|]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[t7+]+)\\b","\\b(m+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[g9]+)(?=[^\\s]*\\b)","\\b(n+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*v+)\\b","\\b([s5$]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*k+)\\b","\\b(h+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b([b8]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*m+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[s5$]+)\\b","\\b([t7+]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*m+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[e3]+)\\b"],
    no: ["\\b(f+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*n+)\\b","\\b(h+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b(d+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[t7+]+)\\b","\\b(k+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*k+)\\b","\\b(f+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b(h+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[e3]+)\\b","\\b([l1|]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(r+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[l1|]+)(?=[^\\s]*\\b)","\\b(r+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(j+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b([i1!|]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[t7+]+)\\b","\\b([s5$]+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(m+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[o0]+)\\b","\\b(n+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)\\b","\\b(n+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[t7+]+)(?=[^\\s]*\\b)","\\b(h+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(v+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[t7+]+)(?=[^\\s]*\\b)","\\b(k+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*d+)\\b","\\b(h+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*m+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[e3]+)\\b","\\b([s5$]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)\\b","\\b(p+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[s5$]+)(?=[^\\s]*\\b)","\\b(k+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*k+)\\b","\\b([t7+]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b(j+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(d+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*k+)(?=[^\\s]*\\b)","\\b(f+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(k+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)\\b","\\b([t7+]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*k+)\\b"],
    sv: ["\\b(h+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b([s5$]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+)\\b","\\b(k+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*k+)\\b","\\b(f+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(h+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[a4@]+)\\b","\\b([s5$]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*y+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(j+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b([a4@]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+)\\b","\\b([i1!|]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[t7+]+)\\b","\\b([e3]+[^\\sa-zA-Z]*f+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*n+)(?=[^\\s]*\\b)","\\b(m+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[o0]+)\\b","\\b(n+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)\\b","\\b(n+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[z2]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[t7+]+)(?=[^\\s]*\\b)","\\b(h+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(v+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[t7+]+)(?=[^\\s]*\\b)","\\b(k+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b([b8]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[e3]+)\\b","\\b(p+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[o0]+)\\b","\\b(k+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[g9]+)(?=[^\\s]*\\b)","\\b([s5$]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[l1|]+)(?=[^\\s]*\\b)","\\b(j+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[l1|]+)\\b","\\b([s5$]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*[a4@]+)\\b","\\b([t7+]+[^\\sa-zA-Z]*j+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*f+[^\\sa-zA-Z]*[s5$]+)\\b","\\b(k+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*d+)(?=[^\\s]*\\b)","\\b([s5$]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[e3]+)\\b","\\b(d+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*m+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*d+)\\b","\\b(f+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[g9]+)\\b"],
    fi: ["\\b(v+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*u+)\\b","\\b(p+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b([s5$]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(k+[^\\sa-zA-Z]*y+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(h+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(m+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*u+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*u+)\\b","\\b(h+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*m+[^\\sa-zA-Z]*[o0]+)\\b","\\b(n+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[i1!|]+)(?=[^\\s]*\\b)","\\b(n+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[i1!|]+)\\b","\\b(h+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(r+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[s5$]+)(?=[^\\s]*\\b)","\\b(v+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*n+)\\b","\\b(j+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*m+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(k+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(r+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[i1!|]+)(?=[^\\s]*\\b)","\\b(h+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[i1!|]+)\\b","\\b(m+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*m+[^\\sa-zA-Z]*u+)\\b","\\b([i1!|]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[i1!|]+)(?=[^\\s]*\\b)","\\b(k+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*y+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*m+[^\\sa-zA-Z]*m+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*n+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(k+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[i1!|]+)\\b","\\b(p+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*n+)\\b","\\b(v+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[a4@]+)\\b","\\b([l1|]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(n+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(m+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[o0]+)\\b","\\b(k+[^\\sa-zA-Z]*y+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[t7+]+)\\b"],
    cs: ["\\b(k+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(h+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[o0]+)(?=[^\\s]*\\b)","\\b([s5$]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(c+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*k+)(?=[^\\s]*\\b)","\\b(k+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[t7+]+)(?=[^\\s]*\\b)","\\b(m+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[t7+]+)(?=[^\\s]*\\b)","\\b([z2]+[^\\sa-zA-Z]*m+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*d+)\\b","\\b(d+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[l1|]+)\\b","\\b([i1!|]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[t7+]+)\\b","\\b([b8]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*c+)\\b","\\b(c+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*n+)\\b","\\b(n+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*r+)\\b","\\b(n+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(h+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b([z2]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[i1!|]+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[l1|]+)\\b","\\b([b8]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[z2]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[t7+]+)(?=[^\\s]*\\b)","\\b([b8]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[z2]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[a4@]+)\\b","\\b([s5$]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*n+)\\b","\\b(k+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[a4@]+)\\b","\\b([s5$]+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[e3]+)\\b","\\b(m+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*r+)\\b","\\b(r+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*d+)(?=[^\\s]*\\b)","\\b([s5$]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*c+)\\b","\\b(v+[^\\sa-zA-Z]*y+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*c+)(?=[^\\s]*\\b)","\\b(k+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*n+)\\b","\\b(h+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*j+[^\\sa-zA-Z]*[z2]+[^\\sa-zA-Z]*[l1|]+)(?=[^\\s]*\\b)","\\b([z2]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*y+)\\b","\\b(p+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[a4@]+)\\b"],
    sl: ["\\b(p+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[z2]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(k+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*c+)\\b","\\b(j+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*[i1!|]+)\\b","\\b(j+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*m+)\\b","\\b(d+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*k+)\\b","\\b(p+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*c+)(?=[^\\s]*\\b)","\\b(k+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(p+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*f+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*n+)(?=[^\\s]*\\b)","\\b([b8]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*k+)\\b","\\b([i1!|]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[t7+]+)\\b","\\b(d+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[l1|]+)\\b","\\b(c+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*n+)\\b","\\b(n+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[t7+]+)(?=[^\\s]*\\b)","\\b(h+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*[o0]+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)\\b","\\b([s5$]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*j+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b([g9]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[o0]+)\\b","\\b(k+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*n+)\\b","\\b([b8]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*j+)\\b","\\b(p+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(j+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[i1!|]+)(?=[^\\s]*\\b)","\\b(m+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*h+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(m+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*m+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*n+[\\W\\d_]*[s5$]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*n+)\\b","\\b([z2]+[^\\sa-zA-Z]*m+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*k+)\\b","\\b(k+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*v+)\\b","\\b(p+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[o0]+)\\b","\\b([s5$]+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*j+[^\\sa-zA-Z]*[a4@]+)\\b","\\b([t7+]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*c+)\\b"],
    hr: ["\\b(j+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*m+)\\b","\\b(j+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*[i1!|]+)\\b","\\b(p+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[z2]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(k+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*c+)\\b","\\b(k+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*[a4@]+)\\b","\\b([s5$]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*j+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b([g9]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[o0]+)\\b","\\b([s5$]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*j+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(d+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[l1|]+)\\b","\\b([i1!|]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[t7+]+)\\b","\\b(k+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*n+)\\b","\\b(c+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[g9]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*n+)\\b","\\b(c+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*j+[^\\sa-zA-Z]*[o0]+)\\b","\\b(n+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[s5$]+[^\\sa-zA-Z]*[t7+]+)(?=[^\\s]*\\b)","\\b(h+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)(?=[^\\s]*\\b)","\\b([s5$]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*v+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*j+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b(p+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*r+)\\b","\\b(d+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[z2]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*j+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b([g9]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*[z2]+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*[a4@]+)(?=[^\\s]*\\b)","\\b([s5$]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*p+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*k+)\\b","\\b(k+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*j+[^\\sa-zA-Z]*[a4@]+)\\b","\\b([b8]+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[l1|]+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(j+[^\\sa-zA-Z]*[e3]+[^\\sa-zA-Z]*[b8]+[^\\sa-zA-Z]*[o0]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[e3]+)(?=[^\\s]*\\b)","\\b(m+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[a4@]+)\\b","\\b([s5$]+[^\\sa-zA-Z]*m+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*d+)\\b","\\b(k+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*c+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*n+[^\\sa-zA-Z]*[a4@]+)\\b","\\b(p+[^\\sa-zA-Z]*[i1!|]+[^\\sa-zA-Z]*[z2]+[^\\sa-zA-Z]*d+[^\\sa-zA-Z]*u+[^\\sa-zA-Z]*n+)\\b","\\b([g9]+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*d+)\\b","\\b(d+[^\\sa-zA-Z]*r+[^\\sa-zA-Z]*k+[^\\sa-zA-Z]*[a4@]+[^\\sa-zA-Z]*[t7+]+[^\\sa-zA-Z]*[i1!|]+)\\b"],
  }; /*@@SWEAR@@*/

  const swear = (function () {
    const cache = {};
    // Länger als das wird nichts geprüft - ein Riegel gegen Regex-Laufzeit.
    const MAX_LEN = 400;

    /**
     * Vereinheitlicht den Text vor dem Vergleich: Akzente weg, Kleinschreibung,
     * "-"/"_" werden zu Leerzeichen (so fängt derselbe Filter auch Map-IDs).
     */
    function norm(text) {
      if (typeof text !== "string") return "";
      text = text.slice(0, MAX_LEN).normalize("NFKD").replace(/\p{M}/gu, "");
      text = text.replace(/-/g, " ").replace(/_/g, " ");
      return text.replace(/\s+/g, " ").trim().toLowerCase().replace(/ß/g, "ss");
    }

    /**
     * Schreibweisen, gegen die geprüft wird: immer die normalisierte Fassung -
     * und zusätzlich die ohne Leerzeichen, wenn der Text WIE GESPERRT
     * GESCHRIEBEN aussieht ("K U R W A").
     */
    function variants(text) {
      const n = norm(text);
      if (!n) return [];
      const parts = n.split(" ");
      if (parts.length >= 3 && parts.reduce((s, p) => s + p.length, 0) / parts.length <= 1.5) {
        return [n, n.replace(/ /g, "")];
      }
      return [n];
    }

    /** Kompilierte Muster einer Sprache (leer, wenn es keine gibt). */
    function patterns(code) {
      if (cache[code]) return cache[code];
      const out = [];
      for (const src of (SWEAR_DATA && SWEAR_DATA[code]) || []) {
        try {
          out.push(new RegExp(src.replace(/~/g, "[^\\sa-zA-Z]*"), "i"));
        } catch (e) {
          // kaputtes Muster: überspringen, nicht crashen
        }
      }
      cache[code] = out;
      return out;
    }

    /** [true, ""] wenn nichts gefunden wurde, sonst [false, sprachcode]. */
    function check(text) {
      const forms = variants(text);
      if (!forms.length) return [true, ""];
      for (const [code] of PG.LANGS) {
        for (const rx of patterns(code)) {
          if (forms.some((f) => rx.test(f))) return [false, code];
        }
      }
      return [true, ""];
    }

    const isClean = (text) => check(text)[0];
    // true, wenn jeder übergebene Text sauber ist (leer zählt als sauber).
    const allClean = (...texts) => texts.every((x) => !x || isClean(x));
    return { norm, variants, patterns, check, isClean, allClean };
  })();

  // ===========================================================================
  //  Eigene Inhalte speichern (ugc.py)
  // ===========================================================================
  const STORE_KEY = "minigolf.ugc";
  const VERSION = 1;
  const ID_CHARS = "abcdefghijklmnopqrstuvwxyz0123456789-_";
  const MAX_ID = 32, MAX_NAME = 28, MAX_AUTHOR = 24;

  const deepClone = (o) => JSON.parse(JSON.stringify(o));
  const headOk = (m) => !!(m && typeof m === "object" && !Array.isArray(m) && validId(m.id)
    && typeof m.name === "string" && m.name.trim());

  // Art -> Eigenschaften (wie KINDS in ugc.py). Die Spiel-Module werden erst
  // beim Aufruf nachgeschlagen - Geometry Dash lädt minigolf_gen.js nicht.
  const KINDS = {
    minigolf: {
      format: "pygamez.minigolf.map", ext: ".lamapgzmap", key: "map", maxItems: 60, naked: true,
      validate: (m) => headOk(m) && Array.isArray(m.tee) && Array.isArray(m.cup),
      normalize: (m) => PG.minigolfGen.normalize(m),
      clone: (m) => PG.minigolfGen.cloneHole(m),
    },
    geodash: {
      format: "pygamez.geodash.level", ext: ".lamapgzlevel", key: "level", maxItems: 60, naked: false,
      validate: (m) => headOk(m) && Array.isArray(m.objects),
      normalize: (m) => PG.gdCore.normalizeLevel(m),
      clone: deepClone,
    },
  };
  const GAMES = Object.keys(KINDS);
  const kind = (game) => KINDS[game] || KINDS.minigolf;

  const pad2 = (n) => String(n).padStart(2, "0");
  function stamp() {
    const d = new Date();
    return `${d.getFullYear()}-${pad2(d.getMonth() + 1)}-${pad2(d.getDate())} ${pad2(d.getHours())}:${pad2(d.getMinutes())}`;
  }

  /** "Mein Tunnel!" -> "mein-tunnel" (leer, wenn nichts bleibt). */
  function slug(text) {
    if (typeof text !== "string") return "";
    let out = text.trim().toLowerCase();
    for (const [src, dst] of [["ä", "ae"], ["ö", "oe"], ["ü", "ue"], ["ß", "ss"], ["å", "a"], ["æ", "ae"], ["ø", "oe"]]) {
      out = out.split(src).join(dst);
    }
    out = Array.from(out).map((c) => (ID_CHARS.includes(c) ? c : "-")).join("");
    out = out.replace(/-+/g, "-").replace(/^[-_]+|[-_]+$/g, "");
    return out.slice(0, MAX_ID);
  }

  function validId(text) {
    return typeof text === "string" && text.length > 0 && text.length <= MAX_ID && Array.from(text).every((c) => ID_CHARS.includes(c));
  }

  function readRaw() {
    const d = PG.store.get(STORE_KEY, {});
    return d && typeof d === "object" && !Array.isArray(d) ? d : {};
  }

  /** Schreibt alles zurück - ALLE Listen bleiben erhalten (auch unbekannte Arten). */
  function writeRaw(data) {
    const out = { v: VERSION };
    if (typeof data.author === "string" && data.author) out.author = data.author;
    const keys = GAMES.concat(Object.keys(data).filter((k) => !KINDS[k]).sort());
    for (const key of keys) {
      if (key === "v" || key === "author" || key === "_generated") continue;
      if (Array.isArray(data[key]) && data[key].length) out[key] = data[key];
    }
    PG.store.set(STORE_KEY, out);
    return true;
  }

  function loadMaps(game = "minigolf") {
    const items = readRaw()[game];
    if (!Array.isArray(items)) return [];
    const k = kind(game);
    return items.filter(k.validate).map((m) => k.normalize(m));
  }

  const get = (id, game = "minigolf") => loadMaps(game).find((m) => m.id === id) || null;
  const count = (game = "minigolf") => loadMaps(game).length;
  const maxItems = (game = "minigolf") => kind(game).maxItems;
  const isFull = (game = "minigolf") => count(game) >= maxItems(game);

  /** Freie id: hängt -2, -3 ... an, solange wanted schon vergeben ist. */
  function uniqueId(wanted, game = "minigolf", ignore) {
    const base = slug(wanted) || (game === "geodash" ? "level" : "map");
    const taken = new Set(loadMaps(game).filter((m) => m.id !== ignore).map((m) => m.id));
    if (!taken.has(base)) return base;
    for (let n = 2; n < 1000; n++) {
      const cand = base.slice(0, MAX_ID - String(n).length - 1) + "-" + n;
      if (!taken.has(cand)) return cand;
    }
    return base;
  }

  /** Speichert einen Inhalt. Gibt [ok, grund] zurück ("invalid"/"id"/"swear"/"full"). */
  function saveMap(m, game = "minigolf") {
    const k = kind(game);
    if (!k.validate(m)) return [false, "invalid"];
    if (!validId(m.id)) return [false, "id"];
    if (!swear.allClean(m.name, m.id, m.author)) return [false, "swear"];
    const data = readRaw();
    let items = Array.isArray(data[game]) ? data[game] : [];
    items = items.filter((x) => k.validate(x) && x.id !== m.id);
    if (items.length >= k.maxItems) return [false, "full"];
    m = k.clone(m);
    m.v = VERSION;
    m.edited = stamp();
    if (!m.created) m.created = m.edited;
    items.push(m);
    data[game] = items;
    return writeRaw(data) ? [true, ""] : [false, "io"];
  }

  function deleteMap(id, game = "minigolf") {
    const data = readRaw();
    const items = data[game];
    if (!Array.isArray(items)) return false;
    const keep = items.filter((x) => !x || x.id !== id);
    if (keep.length === items.length) return false;
    data[game] = keep;
    return writeRaw(data);
  }

  function lastAuthor() {
    const a = readRaw().author;
    return typeof a === "string" ? a : "";
  }

  function setLastAuthor(name) {
    if (typeof name !== "string" || !name.trim()) return false;
    name = name.trim().slice(0, MAX_AUTHOR);
    if (!swear.isClean(name)) return false;
    const data = readRaw();
    data.author = name;
    return writeRaw(data);
  }

  /** Text der Export-Datei. Gibt [ok, grund, text] zurück. */
  function exportText(m, game = "minigolf") {
    const k = kind(game);
    if (!k.validate(m)) return [false, "invalid", ""];
    if (!swear.allClean(m.name, m.id, m.author)) return [false, "swear", ""];
    const payload = { format: k.format, v: VERSION, app: "PyGameZ", exported: stamp() };
    payload[k.key] = k.clone(m);
    return [true, "", JSON.stringify(payload, null, 2)];
  }

  function detectGame(raw) {
    if (!raw || typeof raw !== "object") return null;
    return GAMES.find((g) => raw.format === KINDS[g].format) || null;
  }

  /**
   * Liest einen geteilten Inhalt ein und speichert ihn. Die Art bestimmt der
   * Umschlag; game ist die erwartete Art (null = jede). Eine nackte Map ohne
   * Umschlag geht nur bei Minigolf. Gibt [ok, grund, inhalt] zurück.
   */
  function importText(text, game = "minigolf") {
    let raw;
    try {
      raw = JSON.parse(text);
    } catch (e) {
      return [false, "io", null];
    }
    if (!raw || typeof raw !== "object" || Array.isArray(raw)) return [false, "format", null];
    let found = detectGame(raw);
    let m;
    if (!found) {
      if (game !== null && game !== "minigolf") return [false, "format", null];
      found = "minigolf";
      m = raw;
    } else {
      if (game !== null && found !== game) return [false, "format", null];
      m = raw[KINDS[found].key];
    }
    if (!m || typeof m !== "object" || Array.isArray(m)) return [false, "format", null];
    const k = KINDS[found];
    m = deepClone(m);
    m.id = slug(m.id || m.name || "");
    m.name = String(m.name || m.id || "").trim().slice(0, MAX_NAME);
    if (typeof m.author === "string") m.author = m.author.trim().slice(0, MAX_AUTHOR);
    m = k.normalize(m) || m;
    if (!k.validate(m)) return [false, "format", null];
    if (!swear.allClean(m.name, m.id, m.author)) return [false, "swear", null];
    if (isFull(found)) return [false, "full", null];
    m.id = uniqueId(m.id, found);
    const [ok, reason] = saveMap(m, found);
    return ok ? [true, "", m] : [false, reason, null];
  }

  /** Die gewohnte Schnittstelle EINER Art (ohne game-Parameter). */
  function forGame(game) {
    const k = kind(game);
    return {
      VERSION, ID_CHARS, MAX_ID, MAX_NAME, MAX_AUTHOR, EXT: k.ext, FORMAT: k.format, MAX_MAPS: k.maxItems,
      slug, validId, lastAuthor, setLastAuthor,
      uniqueId: (wanted, ignore) => uniqueId(wanted, game, ignore),
      loadMaps: () => loadMaps(game),
      get: (id) => get(id, game),
      count: () => count(game),
      isFull: () => isFull(game),
      saveMap: (m) => saveMap(m, game),
      deleteMap: (id) => deleteMap(id, game),
      exportText: (m) => exportText(m, game),
      importText: (text) => importText(text, game),
    };
  }

  PG.swear = swear;
  PG.ugc = {
    VERSION, KINDS, GAMES, ID_CHARS, MAX_ID, MAX_NAME, MAX_AUTHOR, STORE_KEY, stamp,
    kind, slug, validId, uniqueId, loadMaps, get, count, maxItems, isFull, saveMap, deleteMap,
    lastAuthor, setLastAuthor, exportText, importText, detectGame, forGame,
  };
})();
