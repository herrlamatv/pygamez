# Changelog

**🌐 Sprache / Language:** **🇩🇪 [Deutsch](#-deutsch)** · **🇬🇧 [English](#-english)**

---

<a name="-deutsch"></a>

## 🇩🇪 Deutsch

### Match-Historie & Tages-Archiv – 2026-11-26

Jedes der 50 Spiele führt jetzt eine **Match-Historie** mit „Letzte Matches“
und „Alle Matches“, und die drei Tagesmodi bekommen ein **Tages-Archiv**: Die
Tagesstrecke, das Tagesrätsel und das Tageswort der letzten 30 Tage – und jeder
Tag, den man schon gespielt hat – lassen sich mit eigenem Bestwert nachspielen.
Alles in 14 Sprachen, mit neuer LamaWiki-Seite, in beiden READMEs, in der
Web-Version, in der Rust-Fassung und auf pygamez.com.

#### Neu

**Match-Historie** (alle 50 Spiele)
- Neuer Knopf **Matches** im Vorspiel-Screen jedes Spiels.
- **Letzte Matches**: Verlaufskurve der letzten 30 Ergebnisse (Rekorde golden
  markiert, Tooltip beim Überfahren) und darunter die letzten 10 Matches.
- **Alle Matches**: alle gespeicherten Matches mit Filter nach Modus (M),
  Sortierung neueste/beste (O) und Bildlauf; darüber die Summen – Matches,
  Siege mit Quote, Bestwert, Schnitt und Spielzeit.
- Je Match: Datum und Uhrzeit, Modus, Punkte (Chip-Spiele: Gewinn bzw.
  Verlust), Sieg/Niederlage/Remis, Dauer und Details wie Einsatz, Auszahlung,
  Multiplikator, Münzen, Figur, Level, Versuche oder Gegner. Rekorde sind
  golden markiert; wer vorzeitig verlässt, bekommt einen Eintrag „abgebrochen“
  (nur mit Punkten oder ab 5 Sekunden).
- Chip-Spiele zählen jede Runde einzeln (Hand, Wurf, Dreh, Kugel,
  Überquerung), Geometry Dash jeden Level-Besuch, Labyrinth jedes gelöste Level.
- Gespeichert werden je Spiel die letzten **250** Matches in der neuen Datei
  **`matches.pgzlamamatches`** (kompakt, je Profil getrennt); die Summen laufen
  unbegrenzt weiter. Auf pygamez.com sind es standardmäßig 50 je Spiel.

**Tages-Archiv** (Crossy Road, Sudoku, Wordle)
- Nachspielbar sind die **letzten 30 Tage** und jeder Tag, der schon gespielt
  wurde – mit demselben Seed wie damals, also exakt derselben Strecke, demselben
  Rätsel bzw. demselben Wort.
- Neue Datums-Leiste im Setup (◀ Heute · 26.11. ▶ Archiv): Pfeile oder **Bild
  auf/ab** wechseln den Tag, **K** oder „Archiv“ öffnet den Kalender mit
  Bestwert, Bestzeit oder Status je Tag; ältere gespielte Tage stehen darunter.
- Je Tag eigener Bestwert (Crossy Road, mit Versuchen), eigene Bestzeit und
  Sterne (Sudoku) bzw. eigener Stand (Wordle, auch angefangen).
- Archiv-Tage ändern keine Tagesserie, keine Wordle-Statistik und keine
  Serien-Erfolge; „Heute“ bleibt der normale Tagesmodus.
- Kompakt gespeichert: eine Tageszahl statt eines Datums je Eintrag – ein Jahr
  Crossy-Archiv sind rund 4 KB, ein Jahr Wordle rund 12 KB.

#### Geändert
- Vorspiel-Screen: Optionen, Matches, Wiki und Zurück stehen in der kompakten
  Reihe nebeneinander; zu lange Beschriftungen bekommen eine Kurzform.

#### Behoben
- Desktop: Der Vorspiel-Screen erscheint wieder in der Akzentfarbe des
  gewählten Spiels (statt in der Theme-Farbe) – wie im Browser.

---

### Lama Crash – 2026-11-21

Spiel Nr. 50 ist ein Crash-Spiel mit Lama-Chips: **Lama Crash**. Ein Lama
steigt im Heißluftballon auf, und der Multiplikator wächst mit jeder Sekunde –
wer rechtzeitig aussteigt, bekommt Einsatz × Multiplikator, wer zu lange
wartet, verliert den Einsatz, sobald der Ballon platzt. Die Reise führt durch
**sechs Höhenwelten** von der Lamawiese bis zur **Mondlandung bei ×100.000**,
begleitet von Spannungsmusik, die mit jeder Welt schneller wird. Dazu kommen
**zwei Einsätze** mit eigenem Auto-Ziel, **Halbieren**, ein Auto-Modus, eine
Statistik und derselbe **Fairness-Check** wie bei Lama Mines und Lama Plinko –
jeder Platzpunkt lässt sich nachrechnen, und jedes Ausstiegsziel bringt 96,997
bis 97,000 % zurück. Außerdem **6 neue Erfolge** – jetzt **274** insgesamt.
Alles in 14 Sprachen, mit LamaWiki-Seite, in beiden READMEs, in der
Web-Version, in der Rust-Fassung und auf pygamez.com.

#### Neu

**Lama Crash** (Spiel Nr. 50, im Menü nach Lama Plinko)
- Einsatz wählen (10, 20, 50 … bis 10000 Lama-Chips), **SPIELEN** drücken: Der
  Einsatz wird abgebucht, nach einem kurzen 3-2-1 hebt der Ballon ab. Ab ×1,01
  darf man jederzeit **AUSSTEIGEN** – das Lama springt mit dem Fallschirm ab,
  der Gewinn wird sofort gutgeschrieben. Platzt der Ballon vorher, ist der noch
  fliegende Einsatz weg.
- **Steigkurve**: ×2 nach 11,6 s, ×10 nach 38,4 s, ×100 nach 76,8 s, ×100.000
  nach 192 s – ganzzahlig (`mu += mu // 1000` je Takt von 1/60 s) und in
  Python, JavaScript und Rust bitgleich (`games/lamacrash_logic.py`). **Turbo**
  (T) steigt dreimal so schnell.
- **Leerflug**: Nach dem Aussteigen fliegt der leere Ballon weiter, bis er
  platzt („Alle draußen – Ballon steigt“) – so sieht man, wie weit es gegangen
  wäre. Leertaste bzw. VORSPULEN läuft 16-mal so schnell.
- **Platzpunkt und Quote**: Für jede Runde wird eine 32-Bit-Zufallszahl r
  gezogen, der Platzpunkt ist `⌊97 · 2³² / (2³² − r)⌋` in Hundertsteln (×1,00
  bis ×100.000). Die Chance, ×x zu erreichen, ist damit 97 % / x – jedes
  Ausstiegsziel bringt 96,997 bis 97,000 % zurück, rund 3,96 % der Runden
  platzen schon bei ×1,00. Der Ballon platzt erst, wenn der Multiplikator den
  Platzpunkt überschreitet: Von Hand bei ×m auszusteigen gewinnt unter genau
  derselben Bedingung wie ein Auto-Ziel ×m. Wer ×100.000 erreicht, landet auf
  dem Mond – Höchstgewinn für alle noch fliegenden Einsätze.
- **Zwei Einsätze** (Z oder Schalter auf Karte 2): ein zweites Lama im Korb,
  jedes mit eigenem Betrag und eigenem **Auto-Ziel** (×1,01 bis ×100.000, ←/→,
  X an/aus). Ein Auto-Ziel zahlt genau beim Ziel aus – auch wenn der Ballon im
  selben Takt platzt. Jeder Einsatz lässt sich einmal **halbieren** (H für
  alle, Q/W einzeln): Die Hälfte fliegt als Geldsack am Mini-Fallschirm davon,
  der Rest steigt weiter.
- **Sechs Höhenwelten**: Lamawiese → Wolkenmeer (ab ×1,5) → Gewitterfront (ab
  ×3, Regen, Blitze, Flugzeuge) → Nordlicht (ab ×10) → Weltall (ab ×100,
  Erdkrümmung, Satelliten, Sternschnuppen, ein immer größerer Mond) →
  **Mondlandung** mit Fahne, Konfetti und Münzregen. Jede neue Welt wird beim
  Erreichen angesagt; die Welten blenden weich ineinander über, der Ballon
  bläht sich mit der Höhe immer weiter auf, und darüber läuft die
  Multiplikator-Kurve mit mitwachsenden Achsen (`games/lamacrash_world.py`,
  `games/lamacrash_draw.py` – alles aus pygame-Primitiven, ohne Bilddateien).
- **Spannungsmusik** (`games/lamacrash_music.py`): ein Stück in sechs Stufen,
  je Höhenwelt eine – von der Wiese bis ins Weltall mit jeder Welt einen
  Ganzton höher und schneller (96 bis 150 bpm), auf dem Mond eine Fanfare; M
  schaltet sie an oder aus. Ab ×2 quietscht der gedehnte Ballon.
- **Nichts verrät den Platzpunkt**: Bild, Musik und Töne hängen nur vom
  aktuellen Multiplikator ab.
- **Platzen**: Knall, Gummifetzen und Kamera-Wackeln, noch sitzende Lamas
  purzeln samt Korb aus dem Bild. Gejubelt wird beim Aussteigen: Münzregen ab
  ×3, Konfetti ab ×10, Gewinn-Banner ab ×100. Oben rechts stehen die
  Platzpunkte der letzten 12 Runden als farbige Pillen.
- **Auto-Modus** (A): 5, 10, 25, 50, 100 oder 250 Runden, Stopp bei Gewinn ab
  bzw. bei Verlust ab (aus, 100, 500, 1000, 5000, 10000 oder 50000 Chips) und
  Turbo. Jeder aktive Einsatz bekommt ein Auto-Ziel, im Leerflug wird
  automatisch vorgespult, von Hand aussteigen geht weiterhin. Wird Auto
  gestoppt, fliegt eine laufende Runde normal zu Ende.
- **Fairness-Check** (F) wie bei Lama Mines und Lama Plinko: Prüfcode vorab,
  Seed nach der Runde; „Nachrechnen“ prüft den Prüfcode, berechnet den
  Platzpunkt neu, kontrolliert die Ausstiege und zeigt ✓ oder ✗ samt
  Mini-Kurve.
- **Statistik** (S): die Kacheln der Chip-Spiele plus Max. Platzpunkt,
  Halbierungen und Haarscharf, die Bilanzkurve der letzten 100 Runden und die
  **Verteilung der Platzpunkte** in sieben Fächern als Balken gegen die
  erwartete Verteilung.
- **Lama-Bank**: dasselbe Konto wie Blackjack, Poker, Casino, Lama Crossing,
  Lama Mines und Lama Plinko. SPIELEN bucht den Gesamteinsatz und speichert die
  Runde in einem Zug (`lamabank.bet_and_pay`), jeder Ausstieg schreibt den
  Gewinn sofort gut, und ein Rest-Übertrag der Hundertstel kommt der nächsten
  Auszahlung zugute. Highscore = Höchststand von 1000 + Lama-Crash-Bilanz;
  unter 10 Chips gibt es einen Bank-Kredit auf 1000. Replays gibt es – wie bei
  allen Chip-Spielen – nicht.
- **Fortsetzen nach dem Verlassen**: Wer mitten im Flug geht, macht beim
  nächsten Start nach einem Countdown am selben Takt weiter – der Platzpunkt
  steht ja schon fest. Wer erst im Leerflug geht, dessen Runde wird sofort
  abgeschlossen.
- **Pause bei Fokusverlust**: Verliert das Fenster den Fokus, während ein
  Einsatz fliegt, pausiert das Spiel von selbst – sonst stiege der Ballon
  weiter, während man woanders ist.
- Steuerung: vor der Runde Leertaste/Enter = SPIELEN, 1/2/Tab = Karte wählen,
  ↑/↓/+/-/Mausrad = Einsatz, ←/→ = Auto-Ziel, X = Ziel an/aus, Z = zweiter
  Einsatz; im Flug Leertaste/Enter/C = alle aussteigen, 1/2 = einzeln, H =
  Hälfte, Q/W = Hälfte einzeln; danach Leertaste = Vorspulen; A/S/F/T/M =
  Auto, Statistik, Fairness, Turbo, Musik. Die Tasten sind fest und hängen
  nicht an der Steuerungs-Belegung. Einsätze, Ziele, Turbo, Musik und die
  Auto-Einstellungen stehen im Abschnitt `lamacrash` der `settings.json` (mit
  Prüfregeln in `settings.py`).
- Eigenes Sidebar-Piktogramm – ein Ballon über einer steigenden Kurve – und
  eine eigene rote Akzentfarbe; 72 neue Texte in allen 14 Sprachen.

**6 neue Erfolge** (jetzt 274)
- **Lama Crash-Stammgast** (25 Partien), ein Meilenstein von **3000** und
  Meister **10000**.
- **Haarscharf** (selten): von Hand ab ×2 aussteigen – höchstens 5 % vor dem
  Platzen. Zählt nur von Hand, nicht mit Auto-Zielen.
- **Polarlicht-Pilot** (episch): bei ×10 oder mehr aussteigen.
- **Weltraumlama** (legendär): bei ×1000 oder mehr aussteigen.
- Insgesamt jetzt 60 allgemeine Erfolge, 41 Meilensteine, 34 Meister-Stufen, 89
  besondere Momente und 50 Stammgast-Erfolge; höchstens **11010 Erfolgspunkte**
  (bisher 10680), das entspricht Level **51** (bisher 50).

**Web**
- Lama Crash läuft auch in der Web-Version – mit derselben Kurve, denselben
  Platzpunkten, Welten, Musik und Erfolgen; die Pause bei Fokusverlust gibt es
  dort ebenso.

**Rust-Fassung**
- Lama Crash gibt es auch in der Rust-.exe – bild- und datengleich mit Python.
  Damit sind es **50/50 Spiele**.

**Website**
- Auf pygamez.com ist Lama Crash dabei – mit der im Konto gespeicherten
  Lama-Bank, auch in der Handy-Version; die Spielezahl steht in allen 14
  Sprachen auf 50.

**Tests**
- Neu: `tests/audit_lamacrash.py` – Lama-Bank-Buchungen, Aussteigen, Leerflug
  und Vorspulen, zwei Einsätze und Hälfte, Auto-Ziel genau am Ziel und
  Mondlandung, Erfolge, Fortsetzen nach dem Verlassen, **Zwillingsrunden**
  (Platzpunkt ×5 und ×50 sehen und klingen bis ×4,99 Bild für Bild gleich –
  nichts verrät den Platzpunkt), Fairness-Nachrechnung, Takte unabhängig von
  der Bildrate (30/60/144 Hz), Turbo, Pause und `pause_on_blur`, Auto-Modus,
  Tasten, Maus, Pleite und Kredit sowie das Layout in 5 Auflösungen × 14
  Sprachen (UI v4.2 und v1).
- `tests/audit_chiplogic.py` prüft jetzt auch die Crash-Regeln (18)–(22): die
  Platzpunkt-Formel vollständig am 2¹⁶-Gegenstück, die exakte Quote je Ziel,
  die Steigkurve und ihre Umkehrung, die Takt-Regel gegen eine Referenz (400
  Zufallsrunden), Monte-Carlo mit echten Seeds sowie Zähler, Texte, Achsen und
  Welten. `devtools/export_chiplogic_ref.py` schreibt dafür einen neuen
  Abschnitt `crash` mit den Prüfwerten für die Rust-Unit-Tests.

#### Geändert

- **Engine**: neuer Haken `pause_on_blur` in `game_base.py` – ein Spiel kann
  (auch als property) verlangen, beim Verlust des Fensterfokus pausiert zu
  werden; `main.py` prüft das direkt nach dem Loslassen der Tasten. Lama Crash
  nutzt ihn, solange ein Einsatz fliegt. Dasselbe gibt es in der Web-Version
  (`web/js/app.js`) und in der Rust-Fassung.
- **Lama-Bank**: `lamabank.py` kennt Lama Crash (`GAMES`, Mindesteinsatz 10 in
  `MIN_BET`).

**Dokumentation**
- LamaWiki: neue Seite **Lama Crash** in allen 14 Sprachen; *Erfolge,
  Seltenheit & Level* (274 Erfolge, 50 Spiele), *Speichern & Highscores* und
  die Lama-Bank-Absätze von Blackjack, Poker, Casino, Lama Crossing, Lama Mines
  und Lama Plinko aktualisiert.
- Beide READMEs: 50 Spiele, neue Tabellenzeile, Feature-Abschnitt,
  Projektstruktur und Erfolgszahlen.

---

### Lama Mines & Lama Plinko – 2026-11-21

Zwei neue Chip-Spiele auf einmal: **Lama Mines** (Spiel Nr. 48) und **Lama
Plinko** (Spiel Nr. 49). Bei Lama Mines deckt man auf einem Feld von 4x4 bis
7x7 Edelsteine auf und meidet die Minen – jeder Edelstein hebt den
Multiplikator, bis zu **×100.000**. Bei Lama Plinko fallen goldene Wollknäuel
durch 8 bis 16 Stiftreihen in Fächer bis **×1000**, bis zu 50 gleichzeitig.
Beide haben einen **Auto-Modus**, ein **Statistik-Fenster** und einen
**Fairness-Check**: Vor jeder Runde steht ein SHA-256-Prüfcode fest, danach
wird der Seed gezeigt – jedes Ergebnis lässt sich nachrechnen. Dazu kommen
**12 neue Erfolge** – jetzt **268** insgesamt. Alles in 14 Sprachen, mit
LamaWiki-Seiten, in beiden READMEs, in der Web-Version, in der Rust-Fassung und
auf pygamez.com.

#### Neu

**Lama Mines** (Spiel Nr. 48, im Menü nach Lama Crossing)
- Einsatz wählen (10, 20, 50 … bis 10000 Lama-Chips), **SPIELEN** drücken: Der
  Einsatz wird abgebucht, das Feld liegt verdeckt bereit. Jeder aufgedeckte
  Edelstein hebt den Multiplikator, eine Mine kostet den Einsatz. Ab dem ersten
  Edelstein darf jederzeit **AUSGEZAHLT** werden; wer alle Edelsteine findet
  („ALLES MEINS!“) oder ×100.000 erreicht, bekommt automatisch ausgezahlt.
- **4 Feldgrößen** (Tasten 1–4): 4x4, 5x5 (Standard), 6x6 und 7x7, dazu **1 bis
  N−1 Minen** (←/→ oder sechs Schnellwahl-Knöpfe, Standard 3).
- **Auszahlungsquote rund 97 %**: Der Multiplikator (in Hundertsteln) nach k
  Edelsteinen ist `m_k = max(⌊97 · C(N, M) / C(N−k, M)⌋, m_(k−1) + 1)`,
  gedeckelt bei ×100.000 – ganzzahlig und in Python, JavaScript und Rust
  bitgleich (`games/lamamines_logic.py`). Er liegt also nie unter ×1,01 und
  steigt mit jedem Edelstein. Jede Strategie „nach k Edelsteinen auszahlen“
  bringt 96,2 bis 97 % zurück; nur drei Stufen mit 1 Mine auf 6x6 und 7x7
  werden von der Untergrenze angehoben (höchstens 98,9 %), und wer bis an den
  Deckel spielt, bekommt weniger. Beispiel 5x5 mit 3 Minen: ×1,10, ×1,25, ×1,44
  … bis ×2.231 für alle 22 Edelsteine.
- **Rest-Übertrag**: Was beim Abrunden einer Auszahlung an Hundertsteln
  übrig bleibt, wird zur nächsten gutgeschrieben – auch kleine Einsätze
  verlieren nichts.
- **Anzeige**: aktueller und nächster Multiplikator, die Chance auf ein sicheres
  Feld in %, der Gewinn beim nächsten Feld und eine Leiste mit den nächsten 5
  Stufen. Edelsteine in vier Farben; das Maskottchen-Lama mit Grubenhelm und
  Stirnlampe füllt seine Lore, freut sich über jeden Edelstein und steht nach
  einer Mine voller Ruß da.
- **Mine getroffen**: Explosion und Kamera-Wackeln, danach wird das **ganze
  Feld abgedunkelt aufgedeckt** – nach dem Auszahlen ebenso. Münzregen ab ×3,
  Konfetti ab ×10, beim Abräumen und beim Höchstgewinn.
- **Auto-Modus** mit Markierungen: Felder im Auto-Fenster auf dem Raster
  markieren (Klick oder Pfeile + Leertaste, Entf/Rücktaste löscht; gemerkt je
  Feldgröße), jede Runde deckt genau diese auf und zahlt aus, wenn alle sicher
  waren; 5, 10, 25, 50 oder 100 Runden. Wird Auto gestoppt, bleibt eine
  laufende Runde offen und kann von Hand weitergespielt werden.
- **Fortsetzen nach dem Verlassen**: Die Runde wird bei jedem aufgedeckten Feld
  gespeichert; beim nächsten Start wird das Feld aus dem Seed neu berechnet und
  es geht genau dort weiter.
- Steuerung: vor der Runde Leertaste/Enter = SPIELEN, 1–4 = Feldgröße, ←/→ =
  Minen, ↑/↓/+/-/Mausrad = Einsatz; in der Runde Klick oder Pfeile + Leertaste
  = Feld aufdecken, R = Zufallsfeld, Enter/C = auszahlen; Turbo (T) = alle
  Zeiten ×0,4. Feldgröße, Minen, Einsatz, Turbo und Auto-Einstellungen stehen
  im Abschnitt `lamamines` der `settings.json`.

**Lama Plinko** (Spiel Nr. 49, im Menü nach Lama Mines)
- Einsatz je Kugel wählen (1, 2, 5 … bis 1000 Lama-Chips), **WERFEN** drücken:
  Ein goldenes Wollknäuel springt Stift für Stift durch das Brett und landet in
  einem Fach mit Multiplikator. Das Lama oben am Brett nickt bei jedem Abwurf.
- **8 bis 16 Reihen** (←/→, Standard 12) und **3 Risikostufen** (1–3: Niedrig,
  Mittel, Hoch); beides ist gesperrt, solange Kugeln fliegen oder Auto läuft.
  Das äußerste Fach zahlt bei 16 Reihen ×16, ×110 bzw. ×1000, bei 8 Reihen
  ×5,3, ×13 bzw. ×32; die Mitte ×0,7, ×0,4 bzw. ×0,2.
- **27 feste Tabellen** in `games/lamaplinko_logic.py`, einmal erzeugt mit
  `devtools/gen_plinko_tables.py`: symmetrisch, nach außen nicht fallend,
  „schöne“ Zahlen und eine exakt (mit Brüchen) berechnete Quote zwischen 96,9
  und 97,05 %. Ein Rest-Übertrag der Hundertstel sorgt dafür, dass auch Einsatz
  1 bei rund 97 % bleibt.
- **Ergebnis = n Münzwürfe**: Jede Reihe ist ein Wurf links/rechts aus dem
  Fairness-Zufall, das Fach ist die Zahl der Rechts-Würfe. Die Animation folgt
  genau diesem Pfad – mit Bogensprüngen, blinkendem Stift und etwas
  kosmetischem Zittern; eine Kugel fliegt 1,62 s (8 Reihen) bis 2,30 s (16
  Reihen). Das Fach wippt beim Treffer, ein Popup zeigt den Multiplikator, der
  Ton hängt vom Wert ab; Münzregen ab ×10, ab ×100 ein Gewinn-Banner mit
  Konfetti.
- **Bis zu 50 Kugeln gleichzeitig**: Leertaste gedrückt halten = Dauerfeuer im
  Takt Flugzeit/30 – so sind rund 30 Kugeln zugleich unterwegs; Turbo halbiert
  Flugzeit und Takt. Der Einsatz lässt sich auch im Flug ändern. Die Maus über
  einem Fach zeigt Multiplikator, Wahrscheinlichkeit und den Gewinn beim
  aktuellen Einsatz.
- **Gebucht wird beim Abwurf**: Einsatz und Gewinn mit einem einzigen Speichern
  (`lamabank.bet_and_pay`), angezeigt wird erst bei der Landung. Wer mit
  fliegenden Kugeln geht, verliert nichts – auch deren Erfolge werden
  nachgeholt.
- **Auto-Modus**: 10, 25, 50, 100, 250, 500 oder 1000 Kugeln, eine alle 0,24 s
  (Turbo 0,12 s), mit festem Einsatz.
- Steuerung: Maus für alles, Leertaste/Enter/Klick auf WERFEN oder aufs Brett =
  werfen (halten = Dauerfeuer), ↑/↓/+/-/Mausrad = Einsatz, ←/→ = Reihen, 1–3 =
  Risiko. Reihen, Risiko, Einsatz, Turbo und Auto-Einstellungen stehen im
  Abschnitt `lamaplinko` der `settings.json`.

**Für beide Spiele**
- **Auto-Modus** (A): Runden- bzw. Kugelzahl, Stopp bei Gewinn ab bzw. bei
  Verlust ab (aus, 100, 500, 1000, 5000, 10000 oder 50000 Chips) und Turbo (T).
  Auto läuft, bis die Zahl erreicht ist, eine Grenze greift, die Chips nicht
  mehr reichen oder STOPP/Esc gedrückt wird.
- **Fairness-Check** (F): Vor jeder Runde – bei Plinko vor jeder Kugel – steht
  der Prüfcode fest, der SHA-256 eines geheimen Server-Seeds aus 32
  Hex-Zeichen. Der Zufall entsteht aus Server-Seed, eigenem Client-Seed (8
  Zeichen, per Knopf neu – nur außerhalb einer Runde bzw. wenn keine Kugel
  fliegt) und Rundennummer (`seedrand`, mulberry32). Nach der Runde wird der
  Seed gezeigt; der Prüfcode der nächsten Runde steht dann schon fest. Das
  Fenster zeigt die letzten 12 Einträge, „Nachrechnen“ prüft Prüfcode und
  Ergebnis neu und zeigt ✓ oder ✗ mit Mini-Vorschau. Neu dafür: `fairness.py`.
- **Statistik** (S): Kacheln für Runden, Gewonnen, Verloren, Eingesetzt,
  Ausgezahlt, Bilanz, größten Gewinn, besten Multiplikator und die eigene
  Quote, dazu eine Bilanzkurve der letzten 100 Runden. Lama Mines zählt
  zusätzlich Edelsteine, getroffene Minen und abgeräumte Felder; Lama Plinko
  zeigt die Treffer-Verteilung je Reihenzahl (←/→ wechselt) gegen die erwartete
  Binomialkurve. Neu dafür: `chipstats.py`.
- Oben rechts steht der **Verlauf der letzten 12 Runden** bzw. Kugeln.
- **Lama-Bank**: dasselbe Konto wie Blackjack, Poker, Casino und Lama Crossing
  (Abschnitt `casino` in `mem.json`); dort liegen je Profil auch laufende
  Runde, Verlauf, Fairness-Werte, Statistik, Rest-Übertrag und
  Auto-Markierungen. Highscore = Höchststand von 1000 + eigene Bilanz; bei
  Pleite gibt es einen Bank-Kredit auf 1000 (bei Plinko, sobald alle Kugeln
  gelandet sind). Replays gibt es – wie bei allen Chip-Spielen – nicht.
- Die Tasten sind fest (Pfeile, Leertaste, Enter, Ziffern und Buchstaben) und
  hängen nicht an der Steuerungs-Belegung in den Optionen; Esc schließt ein
  offenes Fenster bzw. stoppt Auto.

**12 neue Erfolge** (jetzt 268)
- **Lama Mines-Stammgast** und **Lama Plinko-Stammgast** (je 25 Partien), je ein
  Meilenstein von **3000** und Meister **10000**.
- **Schatzkammer** (selten): bei Lama Mines ein ganzes Feld abräumen.
- **Tief gegraben** (episch): bei Lama Mines mit ×10 oder mehr auszahlen.
- **Diamantenlama** (legendär): bei Lama Mines mit ×1000 oder mehr auszahlen.
- **Randtreffer** (selten): bei Lama Plinko das äußerste Fach treffen (ab 12
  Reihen).
- **Wollknäuel-Regen** (episch): 25 Kugeln gleichzeitig auf dem Brett.
- **Plinko-Legende** (legendär): das äußerste Fach bei 16 Reihen und Risiko
  Hoch treffen.
- Insgesamt jetzt 60 allgemeine Erfolge, 40 Meilensteine, 33 Meister-Stufen, 86
  besondere Momente und 49 Stammgast-Erfolge; höchstens **10680 Erfolgspunkte**
  (bisher 10020), das entspricht Level **50** (bisher 49).

**Web**
- Beide Spiele laufen auch in der Web-Version – mit denselben Formeln,
  Tabellen, Fairness-Werten und Erfolgen. SHA-256 ist dort selbst geschrieben
  (Prüfvektor „abc“ → `ba7816bf…`), neue Seeds kommen aus
  `crypto.getRandomValues`.

**Rust-Fassung**
- Lama Mines und Lama Plinko gibt es auch in der Rust-.exe – pixel- und
  datengleich mit Python, geprüft mit Vergleichsläufen in
  `rust/tools/regression.py`. SHA-256 ist auch dort selbst geschrieben, weil
  ohne zusätzliche Crates offline gebaut wird. Damit sind es **49/49 Spiele**.

**Website**
- Auf pygamez.com sind beide Spiele dabei – im Konto mit gespeicherter
  Lama-Bank, auf dem Handy mit Bildschirm-Tasten; die Spielezahl steht in allen
  14 Sprachen auf 49.

**Tests**
- Neu: `tests/audit_lamamines.py` und `tests/audit_lamaplinko.py` – Quote exakt
  (mit Brüchen) und per Monte-Carlo, Rest-Übertrag, Lama-Bank-Buchungen,
  Fortsetzen, Pleite und Kredit, Fairness-Nachrechnung, Auto-Stopps,
  Statistik-Grenzen, Dauerfeuer, Tasten, Maus und das Layout in 5 Auflösungen ×
  14 Sprachen (UI v4.2 und v1).
- Neu: `tests/audit_chiplogic.py` für die gemeinsamen Regeln – SHA-256-Prüfvektoren,
  Seeds, Fairness-Log, Statistik, Mines-Formel und Plinko-Tabellen (auch gegen
  `devtools/gen_plinko_tables.py`); Kreuztest Python = JS über Node, für Rust
  schreibt `devtools/export_chiplogic_ref.py` die Prüfwerte, die die
  Rust-Unit-Tests nachrechnen.

#### Geändert

- **Lama-Bank**: neue Funktion `bet_and_pay(bet, win, game)` bucht Einsatz und
  Gewinn in einem Zug mit **einem** Speichern, auf Wunsch samt Spielstand – für
  Spiele, deren Ergebnis beim Einsatz schon feststeht (Plinko) oder die den
  Einsatz zusammen mit der neuen Runde sichern (Mines). Sie steht ebenso in
  `web/js/games/casino_bank.js` und `rust/crates/pgz-core/src/lamabank.rs`.

**Dokumentation**
- LamaWiki: neue Seiten **Lama Mines** und **Lama Plinko** in allen 14
  Sprachen; *Erfolge, Seltenheit & Level* (268 Erfolge, 49 Spiele) und
  *Speichern & Highscores* (Lama-Bank) aktualisiert.
- Beide READMEs: 49 Spiele, zwei neue Tabellenzeilen, zwei Feature-Abschnitte,
  Projektstruktur und Erfolgszahlen.

---

### Lama Crossing – 2026-11-15

Spiel Nr. 47 ist ein neues Mut-Spiel mit Lama-Chips: **Lama Crossing**. Das Lama
überquert Spur für Spur eine Straße, eine Hängebrücke, einen Fluss oder einen
Eissee – jede geschaffte Spur hebt den Multiplikator, ausgezahlt werden kann
nach jeder Spur. Noch am selben Tag kam **Version 2.0** dazu: eine **Straße mit
Autos und Lastwagen** als neue Standard-Welt, Hindernisse in allen Welten und
längere, härtere Tabellen bis **×31.784,96**. Verlieren fühlt sich jetzt
wirklich wie Verlieren an – und nicht mehr so, als hätte es von Anfang an
festgestanden. Dazu kommen **6 neue Erfolge** – jetzt **256** insgesamt. Alles
in 14 Sprachen, mit LamaWiki-Seite, in beiden READMEs und in der Web-Version.

#### Neu

**Lama Crossing** (Spiel Nr. 47, im Menü nach Casino)
- Einsatz wählen (10, 20, 50 … bis 10000 Lama-Chips), **LOS** drücken: Der
  Einsatz wird abgebucht, das Lama betritt die erste Spur. Nach jeder
  geschafften Spur heißt es **WEITER** oder **AUSZAHLEN**; wer das andere Ende
  erreicht, bekommt automatisch ausgezahlt – mit Konfetti.
- **4 Welten** (Knöpfe oben links oder **W**): die **Straße** als Standard – von
  oben gesehen, mit Gehwegen, das Lama geht über Zebrastreifen-Stücke Spur für
  Spur hinüber –, die Hängebrücke über der Schlucht, Trittsteine im Fluss und
  Eisschollen auf dem gefrorenen See. Das Lama trägt einen Schal in der
  Akzentfarbe des Spiels.
- **Hindernisse, die man abpassen muss**: Autos und Lastwagen fahren jede Spur
  der Straße entlang, Felsbrocken stürzen durch die Hängebrücke, Treibholz
  treibt im Fluss heran, Eisblöcke rutschen über den Eissee. Wer LOS oder WEITER
  drückt, während ein Hindernis in der nächsten Spur ist, wird überfahren bzw.
  getroffen und liegt platt da – also auf eine Lücke warten. Die Spur, auf der
  das Lama steht, ist geschützt: Auf der Straße senkt sich eine Schranke und die
  Autos halten vor dem Lama, in den anderen Welten verschwinden die Hindernisse
  dort. Höhere Schwierigkeiten bringen dichteren, schnelleren Verkehr.
- **Zufallsrisiko je Spur**: Auch bei perfektem Timing kann es schiefgehen – auf
  der Straße taucht plötzlich ein Raser auf, auf der Brücke bricht die Planke,
  im Fluss rutscht der Stein weg, auf dem Eissee bricht das Eis ein.
- **4 Schwierigkeiten**: Leicht (22 Spuren, Risiko 1/12, bis ×6,57), Mittel (20,
  1/6, bis ×37,18), Schwer (18, 3/10, bis ×595,67) und Extrem (15, 1/2, bis
  ×31.784,96 – jede Spur verdoppelt: ×1,94, ×3,88, ×7,76 …).
- **Auszahlungsquote rund 97 %**: Die Multiplikatoren stehen als feste
  Ganzzahl-Tabellen in `games/lamacrossing_logic.py`
  (`m_k = floor(0.97 / (1 - p)^k * 100) / 100`). Damit bringt jede Strategie
  „bei Spur k auszahlen“ bei sauberem Timing im Schnitt dasselbe – der
  Hausvorteil liegt bei etwa 3 %.
- Oben rechts steht der **Verlauf der letzten 12 Runden**.
- **Fortsetzen nach dem Verlassen**: Einsatz, Schwierigkeit und Spur einer
  laufenden Überquerung werden sofort gespeichert; der nächste Start macht genau
  dort weiter.
- **Lama-Bank**: dasselbe Konto wie Blackjack, Poker und Casino (Abschnitt
  `casino` in `mem.json`). Highscore = Höchststand von 1000 +
  Lama-Crossing-Bilanz; unter 10 Chips gibt es einen Bank-Kredit auf 1000.
  Replays gibt es – wie bei allen Chip-Spielen – nicht.
- Steuerung: Leertaste/→/Klick auf die nächste Spur = LOS bzw. weiter (wenn die
  Spur frei ist), Enter/C = auszahlen, ↑/↓/+/-/Mausrad = Einsatz, 1–4 =
  Schwierigkeit, W = Welt. Welt, Schwierigkeit und letzter Einsatz stehen im
  Abschnitt `lamacrossing` der `settings.json`.

**6 neue Erfolge** (jetzt 256)
- **Lama Crossing-Stammgast** (25 Partien), ein Meilenstein von **3000** und
  Meister **10000**.
- **Am anderen Ufer** (selten): das Lama einmal ganz ans andere Ufer bringen.
- **Nervenstark** (episch): mit ×10 oder mehr auszahlen.
- **Drahtseilakt** (legendär): auf Extrem bei ×100 oder mehr auszahlen.
- Insgesamt jetzt 60 allgemeine Erfolge, 38 Meilensteine, 31 Meister-Stufen, 80
  besondere Momente und 47 Stammgast-Erfolge; höchstens **10020 Erfolgspunkte**
  (bisher 9690), das entspricht Level **49** (bisher 48).

**Web**
- Lama Crossing läuft auch in der Web-Version – mit denselben Tabellen, Welten
  und Erfolgen.

**Tests**
- Neu: `tests/audit_lamacrossing.py` – Tabellen und Quote je Strategie,
  Monte-Carlo-Probe der Fehltritte, Web-Logik = Python, Lama-Bank (Abbuchen,
  Auszahlen, Pleite, Kredit), Fortsetzen nach dem Verlassen, Erfolge, Tasten und
  Einstellungen sowie das Layout in 5 Auflösungen × 14 Sprachen × 4 Welten.

#### Geändert

**Lama Crossing 2.0** (noch am selben Tag)
- **Straße als neue Standard-Welt**: eine Straße voller Autos und Lastwagen, die
  das Lama Spur für Spur über Zebrastreifen-Stücke überquert.
- **Hindernisse in allen Welten**: Neben dem Verkehr auf der Straße gibt es
  Felsbrocken auf der Hängebrücke, Treibholz im Fluss und Eisblöcke auf dem
  Eissee. Das Timing entscheidet mit: Wer zu früh losgeht, verliert.
- **Verlieren fühlt sich wie Verlieren an**: Wer erwischt wird, sieht es
  kommen – das Lama wird überfahren oder getroffen und liegt platt da.
- **Nichts wirkt mehr vorherbestimmt**: Man sieht die Hindernisse kommen und
  entscheidet selbst, wann man losgeht. Die Anzeige nach dem Auszahlen, bei
  welchem Schritt es schiefgegangen wäre, ist entfernt – sie ließ Niederlagen
  wirken, als hätten sie von Anfang an festgestanden.
- **Härtere, längere Tabellen**: 22, 20, 18 und 15 Spuren mit Risiko 1/12, 1/6,
  3/10 und 1/2; auf Extrem verdoppelt jede Spur den Multiplikator – bis
  ×31.784,96.
- **Drahtseilakt** verlangt jetzt eine Auszahlung bei ×100 oder mehr auf Extrem
  statt einer ganzen Überquerung.
- Neue Texte in allen 14 Sprachen: Welt Straße, „Überfahren!“, Treffer-Meldungen
  je Welt und der Hinweis „weiter, wenn die Spur frei ist“.

**Dokumentation**
- LamaWiki: neue Seite **Lama Crossing** in allen 14 Sprachen; *Erfolge,
  Seltenheit & Level* (256 Erfolge, 47 Spiele) und *Speichern & Highscores*
  (Lama-Bank) aktualisiert.
- Beide READMEs: 47 Spiele, neue Tabellenzeile, Feature-Abschnitt,
  Projektstruktur und Erfolgszahlen.

---

### PyGameZ in Rust: eigenständige 3-MB-.exe – 2026-09-23

PyGameZ gibt es jetzt ein drittes Mal: als **Rust-Programm** in `rust/`. Kein
Python daneben, kein pygame, keine Laufzeitumgebung – eine einzige Datei von
**3,1 MB** statt der 25,7 MB großen PyInstaller-.exe. Die Spielstände sind
dieselben: wer in Python spielt und danach in Rust startet, findet seine
Highscores, Erfolge und Profile wieder.

#### Neu

**Der Unterbau ist vollständig**
- **10 UI-Designs** (v4.2 „Midnight Glass“ bis zurück zu v1) mit denselben
  Paletten und Effekt-Schaltern wie in Python – Sterne, Farbwolken, Glas-Panels,
  Zickzack-Muster, Vignette, Titel-Glow.
- **14 Sprachen**, fest in die .exe eingebacken. Daneben liegen nur die Dateien,
  die auch geschrieben werden.
- **Spielerprofile** mit Level, 14 Rängen, Symbol und Farbe; **250 Erfolge** in
  fünf Kategorien mit Seltenheitsstufen und Punkten; **Statistik** mit Partien,
  Spielzeit, Siegen, Rekorden, Tagesserie und Kalender.
- **17 Soundeffekte**, beim Start aus Frequenz, Dauer und Wellenform
  synthetisiert – genau die Werte aus `audio.py`, keine Klangdateien.
- **Wortfilter** in allen 14 Sprachen für Profilnamen.
- Bildschirme: Sprachauswahl, Menü, Vorbereitung mit Spieler-2-Wahl, Spiel mit
  Pause und Highscore-Banner, Optionen mit drei Reitern, Erfolge & Statistik,
  Profilverwaltung.

**Zehn Spiele portiert**
- **Snake** (alle fünf Modi, Boost, Goldäpfel), **Pong**, **Breakout**,
  **Tetris**, **2048**, **Minesweeper**, **Tic-Tac-Toe**, **Vier gewinnt**,
  **Flappy** und **Memory** – mit der Spiellogik, den Konstanten und den
  KI-Stufen der Python-Vorlagen.
- Die restlichen 36 Spiele folgen; die Seitenleiste zeigt den Stand als
  „10 / 46“.

**Werkzeuge**
- `rust/tools/export_defs.py` liest `games/__init__.py`, `ui.GAME_COLORS` und
  `achievements.py` aus und erzeugt daraus die Rust-Stammdaten: 46 Spiele,
  250 Erfolge, 10 Designs. Nichts wird abgetippt, nichts läuft auseinander.
- `rust/tools/audit.py` prüft, was kein Einzeltest sehen kann: ob jeder
  benutzte Übersetzungsschlüssel in allen 14 Sprachen steht, ob Platzhalter
  auch gefüllt werden, ob jedes Spiel in den Stammdaten bekannt ist und ob die
  erzeugte Datei noch zu den Python-Quellen passt.
- **Prüfstand im Programm selbst**: `PYGAMEZ_SHOT=… PYGAMEZ_GAME=all` startet
  jedes portierte Spiel nacheinander und legt je einen Screenshot ab – der
  schnellste Weg, nach einer Engine-Änderung alles zu prüfen. Dazu Schalter für
  Bildschirm, Design, Sprache und vorab gedrückte Tasten.
- `rust/ENGINE.md` ist der Portierungs-Vertrag für die nächsten Spiele, das
  Gegenstück zu `web/ENGINE.md`.

#### Geändert

- **Ein Fenster statt zwei.** Die Python-Fassung bettet pygame über
  `SDL_WINDOWID` in ein Tkinter-Fenster ein – daher läuft die Eingabe dort durch
  Tkinter. In Rust wird die Seitenleiste einfach mitgezeichnet.
- **Kein Zwischenpuffer.** Die logische Spielfläche kommt über eine Kamera mit
  Sichtfenster ins Fenster statt über eine Textur: ein Kopiervorgang je Bild
  weniger, und das Bild bleibt scharf.
- **Die Tastennamen bleiben.** `"Up"`, `"space"`, `"Return"`, `"w"` – dieselben
  Tkinter-Namen wie in Python, damit eine bestehende `settings.json` unverändert
  passt.
- **Eigene Tastenwiederholung** (450 ms, dann alle 45 ms) statt der des
  Betriebssystems – auf jedem Rechner gleich.
- Die Build-Ausgabe liegt in **`rust.output/`**, nicht im Quelltext-Ordner.

#### Behoben

- Der Wortfilter hätte in Rust beinahe harmlose Namen beanstandet: ohne
  Wortgrenzen steckt in „Pi**ck a ga**me“ ein slawisches Schimpfwort. Er prüft
  jetzt Wort für Wort und zieht nur dann den ganzen Text zusammen, wenn er
  gesperrt geschrieben ist („b-a-s-t-a-r-d“) – dieselbe Regel, die in Python
  die `\b` in den Mustern leisten.
- Der erklärende Kopf der Filterlisten (`lang/swear/*.yml`) besteht selbst aus
  `#`-Zeilen und wäre fast als Wortliste gelesen worden; gezählt werden jetzt
  nur Kommentare direkt unter einem Muster.

#### Bekannte Lücken

Gegenüber der Python-Fassung fehlen: Replays, LamaWiki, eigene Bahnen,
Lama-Bank, Prestige, Competitive, Personalisierung, Gamepad-Vibration,
Musikschleifen, Pointer-Capture und die Statistik-Diagramme. Die Statistik
selbst gibt es – nur ohne gezeichnete Kurven.

### Geometry Dash: 16 Level, Bibliothek mit 80 Leveln & Vorlagen – 2026-09-22

Geometry Dash bekommt doppelt so viele Kampagnen-Level, eine durchsuchbare
**Bibliothek mit 80 weiteren Leveln** in zehn Kategorien und **12 Vorlagen**
für eigene Level. Jedes neue Level – auch jedes der 80 – ist vom Solver mit dem
echten Spiel-Code durchgespielt worden, samt allen drei Münzen und mit einem
Schritt Versatz nach vorn und hinten. Alles in 14 Sprachen, mit überarbeiteten
Wiki-Seiten, beiden READMEs, erweitertem Audit – und die Web-Version zieht
komplett mit.

#### Neu

**Kampagne: 8 → 16 Level**
- Acht handgebaute Level füllen die Kurve von *Leicht* bis *Dämon*: **Pixel
  Sprint**, **Moss Temple**, **Magnet Fields**, **Hangar Run**, **Strobe
  City**, **Frost Relay**, **Turbine Core** und der zweite Dämon **Lama
  Eclipse**.
- Damit sind **135 Sterne** zu holen (87 Level-Sterne + 48 Münzen) statt 65.
- Die Punktreihe unter der Level-Karte ist jetzt ein **Fenster**: höchstens
  zehn Punkte, die Reihe wandert mit der Auswahl, darunter steht „Level n / m“.
  **[1]**–**[9]** springen zu Level 1–9, **[0]** zu Level 10, **[Pos1]**/
  **[Ende]** an den Anfang oder ans Ende.

**Reiter „Weitere“: 80 Level in 10 Kategorien**
- Neuer dritter Reiter neben SPIEL und LEVELS: **Einstieg, Rhythmus, Münzen,
  Schiff, UFO, Ball, Schwerkraft, Tempo, Welle, Dämon** – je acht Level.
- **Suchfeld** über Namen, id und Kategorie, Kategorie-Leiste, Zufallsknopf und
  eine Zeile „x / 80 Level · n geschafft“. Jede Zeile zeigt Schwierigkeit,
  Länge, gefundene Münzen und den eigenen Bestwert.
- Eigener Fortschritt je Level (Schlüssel `x:<id>` im Abschnitt `geodash`):
  Bestwert, Münzen und Versuche zählen, **Sterne und Erfolge bleiben der
  Kampagne** – so bleibt die Sternskala aussagekräftig.
- **[Tab]** wechselt durch alle drei Reiter, der zuletzt benutzte Reiter und die
  Kategorie werden gemerkt.

**Vorlagen für eigene Level**
- „Neu“ öffnet jetzt eine **Vorlagen-Auswahl** mit 12 Kacheln: *Leer*,
  Einstieg, Sprünge, Pads & Orbs, Schiff, Welle, Ball, UFO, Schwerkraft, Tempo,
  Münzen, Dämon.
- Eine Vorlage bringt Objekte, Farben, Musik und Starttempo mit – und weil der
  Solver jede durchgespielt hat, trägt die Kopie sofort das Häkchen
  **„verifiziert“**, bis man etwas ändert.
- **„Als Vorlage“** im Reiter *Weitere* kopiert jedes fertige Level in die
  eigenen Level; dort lässt es sich umbauen, testen, speichern und spielen.

**Erzeuger, Beweise und Tests**
- Neu: `devtools/geodash_sections.py` (20 Bausteine nach den Maßen der echten
  Physik) und `devtools/build_geodash_extra.py` – deterministisch, mit Beweisen
  in `devtools/geodash_extra_proofs.json`.
- Münzen liegen **auf der bewiesenen Bahn**: erst wird das Level ohne Münzen
  gelöst, dann werden drei Münzen auf den nachgespielten Weg gesetzt und der
  Beweis noch einmal geprüft.
- `tests/audit_geodash.py` prüft zusätzlich: Bibliothek (Desktop-JSON == Web-
  Index == Web-Volldaten), jeden Beweis bei −1/0/+1 Schritt Versatz, alle
  Kategorien, die Vorlagen samt Abzeichen, Filter und Suche, „Als Vorlage“
  sowie Kategorie-Leiste, Suchfeld und Vorlagen-Auswahl in 5 Auflösungen ×
  14 Sprachen.

**Web**
- Der Browser bekommt dieselben drei Reiter, dieselbe Bibliothek und dieselben
  Vorlagen. Die **Volldaten der 80 Level werden erst beim Öffnen des Reiters
  nachgeladen** (`geodash_extra.js`), der kleine Index kommt mit dem Spiel.

#### Geändert

- `settings.json`: `geodash.last_level` reicht jetzt bis 199, dazu die neuen
  Werte `last_tab` und `last_cat`.
- Die Wiki-Seiten **Geometry Dash** und **Eigene Level (Geometry Dash)** wurden
  in allen 14 Sprachen überarbeitet (16 Level, Bibliothek, Vorlagen).

---

### Erfolge 2.0, Statistik-Diagramme & Spielerprofile – 2026-09-21

Das größte Update für alles, was zählt: **250 Erfolge** (statt 108) mit
**Seltenheitsstufen** und **Erfolgspunkten**, daraus ein **Levelsystem** mit
14 Rängen, ein Statistik-Screen mit **fünf Reitern und echten Diagrammen** –
und **Spielerprofile**, damit mehrere Leute an einem Gerät getrennt sammeln
können. Alle Statistik-Daten liegen jetzt in einer eigenen Datei
`stats.pgzlamastats`, die sich weitergeben lässt. Alles in 14 Sprachen, mit
drei LamaWiki-Seiten, beiden READMEs, eigenem Audit – und die Web-Version zieht
komplett mit.

#### Neu

**Erfolge, Seltenheit & Level**
- **250 Erfolge** in fünf Kategorien: 60 allgemeine Ziele, 37 Punkte-Meilensteine,
  30 **Meister-Stufen** (die zweite, deutlich höhere Punktmarke je Spiel), für
  jedes der 46 Spiele einen **Stammgast**-Erfolg (25 Partien) und 77 besondere
  Momente – darunter 30 neue: Riesenschlange (Länge 50), Zu-null-Sieg im Pong,
  Hattrick im Air Hockey, Boss bei Invaders, Goldmedaille bei Flappy,
  Nachtlauf beim T-Rex, Dreifachsprung bei Dame, Zwickmühle bei Mühle,
  Bandenschuss beim Panzer-Duell, Code in drei Versuchen bei Mastermind, vier
  Geister mit einer Kraftpille, Minesweeper ohne Flaggen, zwei Tetris in Folge
  und mehr.
- Fünf **Seltenheitsstufen** mit Punktwert: gewöhnlich 10, selten 25, episch 60,
  legendär 150, mythisch 300. Die Farbe steckt im Abzeichen, im Streifen der
  Erfolgszeile und in der Einblendung; legendäre und mythische Stücke bekommen
  Konfetti.
- **Levelsystem**: Punkte ergeben das Level (Level 2 kostet 20 Punkte, jede
  weitere Stufe 8 mehr als die vorige), dazu **14 Ränge** vom Lama-Küken bis zum
  Sternenlama. Ein Aufstieg zeigt eine eigene Einblendung mit Fanfare; Level,
  Rang und XP-Balken stehen in der Sidebar und im Fortschritt-Screen.
- Beim ersten Start nach dem Update zählen alte Highscores und Statistiken
  sofort – eine einmalige Karte zeigt **dein Startlevel** samt deiner seltensten
  Erfolge, statt einer Toast-Parade.

**Statistik mit Diagrammen**
- Der Fortschritt-Screen hat jetzt **fünf Reiter**: Erfolge (mit Filter, Punkten
  und Level-Kopfzeile), Statistiken (Karten, Top-10-Balken der Spielzeit,
  Sieg/Niederlage-Ring, Tabelle), **Kalender**, **Verlauf** und **Profile**.
- **Kalender**: Spieltage als Heatmap (je dunkler, desto länger gespielt),
  aktuelle und längste Tagesserie, Wochentags-Balken und ein Tageszeit-Ring.
- **Verlauf**: alles mit Zeitstempel – Erfolge (mit Seltenheit und Punkten),
  Rekorde, Level-Aufstiege und Duelle – dazu der **Punkteverlauf je Spiel** als
  Linie mit goldenen Rekord-Marken.
- **Rückblick**: stärkster Tag, Lieblingszeit, längste Sitzung am Stück,
  Lieblingsspiel und dein seltenster Erfolg.
- Alle Diagramme haben Tooltips; die neuen Zeichen-Bausteine stehen in
  `charts.py` (Balken, Linien, Kalender, Ringe, Tageszeit, Tooltip).

**Spielerprofile**
- Bis zu **acht Profile** an einem Gerät, jedes mit eigenen Highscores,
  Erfolgen, Level, Statistiken, Lama-Chips und Spielständen. Gemeinsam bleiben
  nur Auflösung, Ton, Tastenbelegung, Design und die Sprache.
- Neuer Sidebar-Knopf **„Profile"** plus eine Karte im Kopf, die zeigt, wer
  gerade spielt (Avatar, Name, Level, XP-Balken). Anlegen, umbenennen, Symbol
  und Farbe wählen, wechseln, löschen – der Name läuft durch den Wortfilter.
- **Mehrspieler**: vor der Partie wählt Spieler 2 sein Profil (oder spielt als
  Gast); im Spiel stehen die Profilnamen statt „Spieler 1/2".
- **Duell-Bilanz** und **Geräte-Bestenliste** im Profile-Reiter: wer hat gegen
  wen wie oft gewonnen, und wer hält je Spiel den Bestwert.

**Statistik-Datei `stats.pgzlamastats`**
- Alle Statistiken (aller Profile) stehen jetzt in einer eigenen JSON-Datei
  neben dem Spiel – mit Kopfzeile wie bei Replays und Bahnen.
- **Export/Import** im Kalender-Reiter (oder mit **E** und **I**); die
  Web-Version liest und schreibt dieselbe Datei.
- Gegen unbegrenztes Wachstum sind Verlauf (4000 Einträge), Punktverläufe (250
  je Spiel) und Kalender (rund drei Jahre) gedeckelt – die Summen bleiben.

**Web-Version**
- Zieht komplett mit: Erfolge samt Seltenheit und Level, alle fünf Reiter mit
  denselben Diagrammen, Profile (eigener Speicher-Prefix je Profil) und der
  Austausch der `.pgzlamastats`-Datei.
- Die Definitionen erzeugt `devtools/export_achievements.py` direkt aus
  `achievements.py` – Desktop und Browser können nicht auseinanderlaufen; ein
  Test prüft das.

#### Geändert

- `mem.json` hat eine neue Struktur: gerätweit bleibt nur `mem` (Sprache),
  alles andere liegt unter `players/<profil-id>`. Der bisherige Spielstand
  wandert beim ersten Start automatisch in Profil 1.
- Der alte `stats`-Abschnitt der `mem.json` zieht in die neue Statistik-Datei um
  (inklusive Erfolgs-Zeitstempeln als Verlauf).
- Die Snake-Kosmetik aus `mem-ngb.json` gehört jetzt zum Profil.
- Replays und selbstgebaute Bahnen bleiben gerätweit, tragen aber den
  Profilnamen; der Bahn-Editor schlägt ihn als Autor vor.
- Neue LamaWiki-Seiten **Spielerprofile** und **Statistik & Diagramme**, die
  Seite **Erfolge** ist überarbeitet – in allen 14 Sprachen.

#### Behoben

- Die Lama-Bank las ihre Abschnitte noch an den Profilen vorbei – sie geht jetzt
  ebenfalls über den Profil-Speicher.
- Nach einem Neustart aus dem Game Over heraus zählte ein zweites Duell nicht
  mehr; der Riegel wird jetzt zusammen mit dem Sieg/Niederlage-Riegel freigegeben.

---

### Replays für Billard, Pinball, Snake & Tetris – und Teilen als Datei – 2026-09-18

Die Wiederholungen gibt es jetzt in **sechs Spielen** statt zwei: **Billard**,
**Pinball**, **Snake** und **Tetris** schneiden ihre Runden genauso mit wie
Minigolf und Bowling. Neu ist außerdem das **Teilen**: **E** schreibt eine
Aufnahme als `.lamapgzreplay`-Datei, **I** liest sie anderswo wieder ein. Alles
in 14 Sprachen, mit überarbeiteter LamaWiki-Seite, in beiden READMEs und mit
einem eigenen Audit.

#### Neu

**Billard**
- Je **Stoß** eine Sequenz: Kopfdaten sind Zielwinkel, Stärke, Spieler, Gruppen
  und der komplette Tisch; die Samples enthalten nur noch die Kugeln, die sich
  seit dem letzten Bild bewegt haben. Der Vorlauf zeigt Ziellinie und Queue wie
  beim echten Stoß, der Nachlauf das Ergebnis (Versenkt, Verfehlt, Foul, Sieg).
- Am Partieende zeigt **P** die Wiederholung. Der **Übungsmodus** endet nie –
  dort zeigt **P** zwischen zwei Stößen den bisherigen Verlauf, danach läuft die
  Partie ganz normal weiter.
- Abgespielt wird in der aufgezeichneten Ansicht (2D, 3D oder Frei); die
  Kugelbahnen liegen in Tischkoordinaten und sind davon unabhängig.

**Pinball**
- Je **Kugel** eine Sequenz: Ballbahnen (auch im Multiball), Flipperstellung,
  Drop-Targets, Rollover-Bahnen, Multiplikator, Locks, Ball-Save, Tilt und
  Punktestand. Ein ruhiges Bild kostet sechs Zahlen – der Tischzustand steht nur
  in Samples, in denen er sich wirklich geändert hat.

**Snake**
- Ein Lauf wird in **Kapitel** von zehn Sekunden geschnitten: jedes Kapitel ist
  ein Schlüsselbild (Körper, Äpfel, Punkte), die Samples danach enthalten nur
  die Änderungen – ein Schritt sind fünf Zahlen. Aufgezeichnet werden alle Modi
  samt Hindernissen, Portalen, Goldäpfeln, Competitive-Spezialäpfeln und
  Ausdauer; im Mehrspieler beide Schlangen.
- Die Wiedergabe zeigt immer die **Draufsicht** – auch für Läufe in der
  3D-Ansicht, deren Kamera an der Bildrate hängt.
- **P** öffnet sie am Game Over; im laufenden Spiel bleibt P das Prestige.

**Tetris**
- Ebenfalls in **Kapiteln**: Feld (nur geänderte Zeilen), aktiver Stein samt
  Lock-Delay, Hold, 5er-Vorschau, Punkte, Level, Combo, Back-to-Back und die
  Müll-Warteschlange. Im Versus laufen **beide Felder** mit.
- **P** öffnet die Wiederholung auf dem Ergebnis-Screen.

**Teilen als Datei**
- **E** schreibt die Aufnahme als **`.lamapgzreplay`** – im Archiv wie direkt
  nach der Runde. Die Datei ist aufgebaut wie eine Minigolf-Bahn oder ein
  Geometry-Dash-Level: ein Umschlag mit `format`, Version und genau einem
  Replay.
- **I** liest so eine Datei wieder ein. Sie landet im Reiter des passenden
  Spiels und trägt dort einen kleinen Pfeil. Doppelte Aufnahmen, fremde Dateien
  und ein volles Archiv werden je mit eigener Meldung abgewiesen.
- Ohne Datei-Dialoge (z.B. ohne Tk) landet der Export im **Downloads-Ordner**;
  der Pfad steht in der Rückmeldung.
- Neuer Erfolg **Vorführer** (ein Replay als Datei geteilt) – jetzt **108**.

**Browser-Version**
- Die Web-Fassung unter `web/` bekommt dasselbe Replay-System: `js/core/replay.js`
  (Aufnahme, Archiv, Teilen) und `js/core/replayview.js` (Archiv-Liste und
  Wiedergabe), dazu der Sidebar-Knopf **Replays** und der Schalter *Replays
  aufzeichnen* in den Einstellungen.
- Aufgezeichnet werden dieselben sechs Spiele – auch Minigolf und Bowling, die
  es im Browser bisher gar nicht mitschnitten – mit **demselben Dateiformat**:
  eine `.lamapgzreplay`-Datei vom PC läuft im Browser und umgekehrt. **E** lädt
  sie herunter, **I** liest sie wieder ein.
- Das Archiv liegt im localStorage: Platz für 5 Aufnahmen je Spiel, eine
  Aufnahme endet nach 10 Minuten, und ein voller Browser-Speicher meldet sich,
  statt still zu scheitern.
- Neuer Prüfstand `web/tools/replay_check.js` (headless Chrome): spielt je Spiel
  eine echte Partie, fährt die Aufnahme vorwärts, rückwärts und in
  Zufallssprüngen durch, vergleicht den Endstand mit dem Original und prüft
  Teilen, Einlesen und den Replay-Screen – dazu eingefrorene Aufnahmen der
  Desktop-Version (`web/tools/desktop_replays.js`), damit das Dateiformat beider
  Fassungen gleich bleibt.

#### Geändert
- Die **Reiterleiste** des Replay-Screens bricht bei sechs Spielen sauber um;
  darunter steht eine Kopfzeile mit Zähler und den Knöpfen **Teilen** und
  **Einlesen**.
- **Vor- und Nachlauf je Spiel** (`replayview.PAD`): Spiele mit Zielvorgang
  behalten ihren Vorlauf mit Ziellinie, Snake und Tetris laufen ohne Pause
  durch. Der Sequenz-Zähler heißt jetzt je Spiel Schlag, Wurf, Stoß, Ball oder
  Abschnitt.
- Der **Recorder** verwirft eine zu lange Aufnahme nicht mehr komplett, sondern
  hört auf mitzuschreiben und kennzeichnet sie als **Aufnahme gekürzt** (Grenze:
  60.000 Samples bzw. 420.000 Zahlen – gut eine halbe Stunde).
- Neuer Helfer `replay.Delta` für Samples, die nur Änderungen enthalten; Billard,
  Pinball, Snake und Tetris nutzen ihn gemeinsam.
- Die LamaWiki-Seite **Replays** beschreibt jetzt alle sechs Spiele, die Kapitel
  und das Teilen als Datei.
- Neues Audit `tests/replay_audit.py`: spielt je Spiel eine echte Partie, fährt
  die Aufnahme vorwärts, rückwärts und in Zufallssprüngen durch und vergleicht
  Feld, Körper, Kugeln und Punkte mit dem Original – dazu Export, Import und
  alle Abweisungsgründe.

### Arcade & Casino: 4 neue Spiele + 6 Ausbauten – 2026-09-16

Das bisher größte Spiele-Update: Mit **Crossy Road**, **Geometry Dash**,
**Schiffe versenken** und **Casino** wächst die Sammlung auf **46 Spiele**;
**Tetris**, **2048**, **Schach**, **Wordle** und **Sudoku** werden groß
ausgebaut, **Blackjack** und **Poker** spielen jetzt mit einem gemeinsamen
Lama-Chip-Konto. Dazu kommt ein neues technisches Fundament (Tasteneingabe,
absturzsicheres Speichern, Musik-Schleifen, ein gemeinsamer Zufallsgenerator für
PC und Browser) und **22 neue Erfolge** – jetzt **107** insgesamt. Alles in 14
Sprachen, mit LamaWiki-Seiten, in beiden READMEs und in der Web-Version.

#### Neu

**Crossy Road** (Spiel Nr. 43)
- Endlos über **Wiesen, Straßen, Flüsse und Gleise** hüpfen – im
  **isometrischen Voxel-Look** mit Squash & Stretch, Wassersplash,
  Plattdrück-Animation und Partikeln. Figuren und Bodenstreifen werden je
  Kachelgröße vorgerendert, das Zeichnen bleibt auch bei 1280x960 bei wenigen
  Millisekunden pro Bild.
- **Züge** mit Warnlicht und Klingel (später ganze Bahnhöfe mit bis zu 5
  Gleisen), Stämme und Seerosen, Autos und Laster, die mit der Strecke schneller
  werden; der **Adler** holt Trödler. Der Generator garantiert immer einen
  begehbaren Weg.
- **Tag/Nacht-Wechsel** ab Reihe 50 mit Scheinwerfern, Zuglichtern und Lichthof.
- **10 freischaltbare Figuren** (Huhn, Frosch, Schwein, Pinguin, Katze, Fuchs,
  Lama, Roboter, Geist, Einhorn), gekauft mit gesammelten Münzen im Reiter
  **Figuren**.
- Modi **Endlos** (Highscore) und **Tagesstrecke** – für alle gleich, auch in
  der Web-Version – mit eigenem Tagesbestwert (Abschnitt `crossy` in `mem.json`).
- Neue Erfolge **Neuer Look** und **Gleisakrobat** sowie ein Meilenstein von 150
  Reihen.

**Geometry Dash** (Spiel Nr. 44)
- Rhythmus-Plattformer mit **Würfel, Schiff, Ball, UFO und Welle**, Form-,
  Schwerkraft- und Tempo-Portalen, gelben/pinken/blauen Pads und Orbs, Gruben,
  Farb-Triggern und je **3 geheimen Münzen**.
- **8 eingebaute Level** von Leicht bis Dämon („Lama Inferno"). Jedes ist per
  Solver nachweislich schaffbar, samt allen Münzen.
- **Übungsmodus** mit automatischen und eigenen Checkpoints (Z/X),
  Versuchszähler, Fortschrittsbalken, Bestwerten je Level, Explosionen und
  sofortigem Neustart.
- Eigener **Soundtrack je Level** mit Beat-Puls in der Grafik (abschaltbar).
- **Präzise Eingabe**: Die Physik rechnet in Festkomma mit festem 240-Hz-Schritt,
  jeder Druck wirkt auf 1/240 s genau – bei jeder Bildrate gleich.
- **Level-Editor** im Reiter LEVELS: Palette mit 6 Gruppen, Undo/Redo, Test ab
  Start oder ab hier, Level-Einstellungen, **Teilen als `.lamapgzlevel`**, Import
  und „Verifiziert"-Abzeichen erst nach eigenem Durchlauf.
- Highscore = **Sterne gesamt** (max. 65); Bestwerte, Münzen, Versuche und
  Sprünge im Abschnitt `geodash` von `mem.json`. Neue Erfolge **Im Takt**,
  **Münzjäger** und **Dämonenbezwinger** sowie ein Meilenstein von 20 Sternen.
- Browser-Version mit identischer Physik und Editor.

**Schiffe versenken** (Spiel Nr. 45, in allen anderen Sprachen „Battleship")
- **10x10-Seeschlacht** gegen die KI – *Leicht* (zufällig), *Mittel*
  (Jagen/Zielen) und *Schwer* (Wahrscheinlichkeitskarte mit Parität) – oder
  **zu zweit** mit verdecktem Übergabe-Bildschirm.
- **Flotte per Drag & Drop** aufstellen: drehen mit R/Rechtsklick, zufällig,
  grüne/rote Vorschau mit Sperrzone; die letzte Aufstellung wird vorgeschlagen.
- **Regeln wählbar** und gespeichert: Schiffe dürfen sich berühren,
  Salven-Modus, Nochmal schießen nach Treffer.
- Radar-Sweep, animierte Wellen, Granaten im Bogen, Einschlag-Splash,
  Explosionen mit Rauch, brennende Felder, „VERSENKT!"-Enthüllung und
  Treffer-Statistik am Rundenende.
- Neue Erfolge **Unversehrt** (Sieg ohne verlorenes Schiff) und **Großadmiral**
  (Sieg gegen die schwere KI); in der Web-Version gegen die KI.

**Casino** (Spiel Nr. 46)
- **Europäisches Roulette** mit allen klassischen Wetten (Plein bis
  Manque/Passe – Klick auf Zahl, Kante oder Ecke), Chipwerten 1 bis 500,
  Wiederholen/Verdoppeln/Löschen, animiertem Kessel mit einlaufender Kugel und
  Verlauf der letzten 12 Zahlen.
- **Lama-Slot**: 5 Walzen, 10 Gewinnlinien, Lama = Wild, Goldmünzen = 10
  Freispiele mit doppelten Gewinnen, Auto-Spin, Turbo und Gewinntabelle;
  **Auszahlungsquote 96,1 %**, exakt aus den Walzenstreifen berechnet.
- **Lama-Bank** (`lamabank.py`): Blackjack, Poker und Casino teilen sich ein
  Konto mit Lama-Chips (Abschnitt `casino` in `mem.json`). Jedes Spiel führt
  seine eigene Bilanz für den Highscore; bei Pleite gibt es einen Bank-Kredit.
  Alte Chipstände werden automatisch übernommen.
- Neue Erfolge **Volltreffer** (Plein beim Roulette) und **Lama-Jackpot**
  (5 Lamas auf einer Linie) sowie ein Meilenstein von 5000.

**Tetris** – komplett überarbeitet
- Moderne **Guideline-Regeln**: SRS-Drehsystem mit Wall Kicks, Drehen in beide
  Richtungen, **Halten**, **5er-Vorschau**, **Lock Delay** und eigenes
  **DAS/ARR** (im Setup einstellbar).
- **Solo** mit drei Varianten: **Marathon** (Highscore, Startlevel 1–15),
  **Sprint 40 Zeilen** (Bestzeit) und **Ultra 2 Minuten** (Bestwert).
- Wertung nach Guideline: **T-Spins** (voll/Mini), **Back-to-Back**, **Combos**
  und **Perfect Clear** – mit Einblendungen, Lösch-Animation, Partikeln und
  Level-Up-Effekt.
- **Versus gegen die KI** (3 Stärken) und **Versus zu zweit** mit Müllzeilen,
  Aufrechnen, Warnbalken, gleicher Steinfolge und K.O.-Animation.
- 3 neue Erfolge: **Dreh dich rein** (T-Spin Double), **Blitzblank** (Perfect
  Clear) und **Sprinter** (Sprint 40 unter 2:00).
- Web-Version: alles außer „2 Spieler", gleiche Regeln und Steinfolge wie am PC.

**2048** – groß ausgebaut
- Eigener **Setup-Screen** mit Brettgrößen **3x3 bis 8x8** und den Modi
  Klassisch, Zeitangriff (3 Minuten) und Endlos.
- Flüssige **Animationen**: Kacheln gleiten, verschmelzen mit Pop und wachsen
  hinein; Punkte-Popups, Funken und neue Farben bis 131072.
- **Rückgängig** (aus / 3 pro Partie / unbegrenzt) mit Animation – wer es nutzt,
  spielt ohne Highscore.
- **Wischen** mit Maus/Touchpad; gehaltene Tasten wiederholen nicht mehr,
  Eingaben während Animationen werden gepuffert.
- Partien werden je Größe und Modus automatisch **gespeichert** und lassen sich
  fortsetzen; Bestwerte und größte Kachel je Größe/Modus (Abschnitt `g2048`).
- „Weiterspielen?" nach 2048 im Klassik-Modus, neues Endstand-Panel und der neue
  Erfolg **4096!**.

**Schach** – groß ausgebaut
- Neuer Modus **Rätsel**: 200 Aufgaben aus der freien **Lichess-Rätseldatenbank
  (CC0)** in 5 Stufen (Matt in 1/2/3, Taktik I/II) mit Fortschritt.
- **Chess960** (alle 960 Grundstellungen) und **Schachuhr** (1+0, 3+2, 5+0,
  10+5) im Setup.
- Deutlich **stärkere, ruckelfreie KI**: iterative Vertiefung,
  Transpositionstabelle, Eröffnungsbuch und neue Bewertung; sie rechnet in
  kleinen Häppchen pro Bild.
- Neue **Seitenleiste** mit Zugliste (SAN), Uhr, geschlagenen Figuren und
  Materialbilanz; Rückgängig, Hinweis-Pfeil, Remis anbieten, Aufgeben, Brett
  drehen, Drag & Drop, gleitende Figuren und Koordinaten.
- **PGN-Export** nach der Partie (Web: Download und Zwischenablage).
- Neue Erfolge **Rätselknacker** (25 Rätsel) und **Großmeister** (Sieg gegen
  Stufe 6 ohne Hilfe).

**Wordle** – groß ausgebaut
- Neue Modi **Tageswort**, **Dordle** (2 Wörter, 7 Versuche) und **Quordle**
  (4 Wörter, 9 Versuche) neben Endlos.
- Neuer **Setup-Screen**: Wortlänge **4 bis 7**, harter Modus (jetzt eine
  Einstellung statt eines eigenen Modus) und **Farbenblind-Palette**
  (Orange/Blau).
- **Tageswort** je Sprache und Länge – am PC und im Browser dasselbe – mit
  Countdown und Serie; angefangene Tageswörter werden gespeichert.
- **Statistik** je Sprache, Länge und Modus: Partien, Gewinnquote, Serien und
  Versuchsverteilung als Balkendiagramm (Abschnitt `wordle` in `mem.json`).
- **Ergebnis teilen** als Emoji-Raster in die Zwischenablage; AZERTY-Tastatur für
  Französisch, QWERTZ für Deutsch, Tschechisch, Slowenisch und Kroatisch.
- Neue Wortlisten mit **4, 6 und 7 Buchstaben** für alle 14 Sprachen (über alle
  Längen rund 134.000 Lösungswörter); bessere Lösungswörter ohne englische
  Einsprengsel, Namen und anstößige Wörter.
- Neue Erfolge **Wortgewohnheit** (7 Tageswörter in Folge) und
  **Vierfach-Genie** (Quordle gelöst).

**Sudoku** – neue Varianten
- Neue Varianten **X-Sudoku**, **Killer** (400 vorab erzeugte, eindeutig lösbare
  Level) und **Mini 6x6** – jeweils 4 Stufen × 100 Level.
- **Tages-Sudoku**: ein Rätsel pro Tag für alle (am PC und im Browser gleich),
  Stufe je Wochentag und Tagesserie.
- **Rückgängig/Wiederholen** (U/Z, Y), **Kandidaten automatisch eintragen** (C),
  Eingabe „Ziffer zuerst", **Farbmarker** (M) und ein Restziffer-Zähler in allen
  Modi.
- **Angefangene Rätsel** werden automatisch gespeichert und fortgesetzt.
- Bis zu **3 Sterne** und Bestzeit je Level, neuer Setup-Screen mit Level-Info.
- Neue Erfolge **Käfigkämpfer** und **Sternensammler** (30 Level mit 3 Sternen).

#### Geändert

**Blackjack & Poker**
- Beide spielen mit den **Lama-Chips** der Lama-Bank (Start 1000) und teilen
  sich das Konto mit dem Casino; der Highscore ist jeweils der Höchststand der
  eigenen Bilanz, Bank-Kredite zählen nicht mit.
- Einsätze werden **sofort abgebucht** – wer eine laufende Hand verlässt,
  verliert den Einsatz. Beim Poker kommt der nicht gesetzte Tischstapel beim
  nächsten Start zurück aufs Konto.
- Der Blackjack-Meilenstein liegt jetzt bei 1600 (Startkonto 1000 plus 600),
  **Chipleader** zählt die Poker-Bilanz.

**Fundament & Technik**
- **Tastatur**: Gehaltene Tasten werden sauber erkannt – Spiele mit eigener
  Tastenwiederholung (Tetris) oder Halten-Steuerung (Geometry Dash) reagieren
  präzise, und Tasten „klemmen" nicht mehr nach Pause, Alt-Tab oder einem
  Screen-Wechsel.
- **Absturzsicheres Speichern**: `mem.json` und `settings.json` werden atomar
  geschrieben, mit automatischer Sicherung (`.bak`); eine beschädigte `mem.json`
  wird aus der Sicherung gelesen, statt beim nächsten Speichern Highscores,
  Erfolge und Chips zu verlieren.
- Wer mitten im Spiel über die **Sidebar** ein anderes Spiel wählt, behält jetzt
  seinen Highscore.
- **Vorspiel-Screen**: Bei vielen Modi und kleiner Auflösung rücken Optionen,
  Wiki und Zurück in eine Reihe, die Schrift passt sich an – nichts läuft mehr
  aus dem Bild.
- Neuer gemeinsamer **Zufallsgenerator** (`seedrand.py` / `seedrand.js`):
  Tagesmodi und neue Rätsel sind am PC und im Browser bitgenau identisch.
- **Musik-Schleifen** für Spiele (`audio.py`): mehrstimmig und prozedural
  erzeugt, pausieren und enden automatisch mit ihrem Spiel – erstmals genutzt für
  die Level-Soundtracks von Geometry Dash.
- **Eigene Inhalte** (`ugc.py`, `ugc.json`) verwalten jetzt Minigolf-Bahnen und
  Geometry-Dash-Level; Minigolf bleibt voll kompatibel.
- Neue Spiel-Hooks in `game_base.py`: `on_exit()` (Spiel wird verlassen),
  `show_highscore_banner` (Modi ohne Highscore-Wertung zeigen kein Banner) und
  `key_is_free()` (feste Zusatztasten greifen nur, wenn sie keiner Aktion
  zugeordnet sind); neue Spiel-Optionen prüft `settings.SCHEMAS`, eine eigene
  Whitelist ist nicht mehr nötig.
- Werkzeuge in `devtools/` (nicht in der `.exe`): `merge_staging.py` spielt
  Übersetzungen und Wiki-Seiten in alle 14 Sprachdateien ein, dazu Build-Skripte
  für Schachrätsel, Killer-Sudokus, die Crossy-Road-Modelle und die
  Geometry-Dash-Level samt Solver.
- Tests: `tests/arcade_casino_audit.py` prüft Eingabe, Zufallsgenerator
  (Python = JS), Speichern, Musik, Sprachdateien und alle Vorspiel-Screens in
  14 Sprachen × 5 Auflösungen und startet die neuen Audits je Spiel
  (`tests/audit_*.py`).

**Dokumentation**
- LamaWiki: neue Seiten für Crossy Road, Geometry Dash (plus *Eigene Level*),
  Schiffe versenken und Casino,
  überarbeitete Seiten für Tetris, 2048, Schach, Wordle, Sudoku, Blackjack und
  Poker
  sowie die allgemeinen Seiten *Erste Schritte* (alle 20 Spiele mit
  2-Spieler-Modus), *Optionen & Steuerung* (14 Sprachen, feste Zusatztasten),
  *Speichern & Highscores* (neue Abschnitte in `mem.json`, `ugc.json`,
  automatische Sicherung) und *Erfolge & Statistiken* (107 Erfolge, 46 Spiele).
- Beide READMEs: 46 Spiele, neue Tabellenzeilen und Feature-Abschnitte,
  Mehrspieler-Liste, Bedienung, Projektstruktur und Quellenhinweis zur
  Lichess-Rätseldatenbank.

#### Behoben
- **Blackjack**: Die verdeckte Dealer-Karte wird beim Aufdecken jetzt wirklich
  umgedreht.
- **Tetris**: Ein Stein, der komplett über dem Feld einrastet, beendet jetzt das
  Spiel; Enter/Leertaste starten nach dem Game Over nicht mehr versehentlich neu.
- **Schach**: Der Siegzähler zählt jetzt deine Siege statt der Siege einer Farbe.
- **Vorspiel-Screen**: Bei 480x360 liefen Sudoku und Solitär aus dem Bild, und
  der Untertitel wurde überdeckt.
- **Tastatur**: Gehaltenes ESC schaltete die Pause flackernd um, gehaltenes F11
  den Vollbildmodus.

### Wordle: echte Wortlisten in 14 Sprachen – 2026-09-16

#### Neu
- **Echte Wortlisten für alle 14 Sprachen** im neuen Ordner `woordlistz/`:
  zusammen **34.442 Lösungswörter** und **213.400 erlaubte Rateworte** (je
  Sprache 10.000 bis 19.000). Gebaut aus den Hunspell-Wörterbüchern der
  jeweiligen Sprachgemeinschaft und aus Häufigkeitslisten echter Texte;
  Englisch übernimmt die Originallisten des Vorbilds, Finnisch das Vokabular
  des Voikko-Projekts. Vorher waren es rund 100 handverlesene Wörter in nur
  5 Sprachen.
- **Rateversuche werden geprüft** – wie beim Original muss jedes eingetippte
  Wort in der Liste stehen. Was nicht darin steht, wird abgelehnt: Die Zeile
  wackelt kurz, eine Meldung erscheint und der Versuch zählt nicht.
- **Harter Modus** als zweiter Modus im Vorspiel-Bildschirm: Gefundene Hinweise
  müssen weiterverwendet werden – grüne Buchstaben bleiben an ihrem Platz,
  gelbe müssen wieder vorkommen. Die Meldung sagt, welcher Hinweis fehlt
  („1. Buchstabe muss B sein“).
- Dieselben Listen in der **Web-Version**: Dort wird immer nur die Datei der
  eingestellten Sprache nachgeladen (59–111 KB), damit der Start schnell bleibt.
- `woordlistz/build_wordlists.py` baut die Listen jederzeit neu;
  `woordlistz/README.md` nennt alle Quellen und ihre Lizenzen.

#### Geändert
- Umlaute und Akzente stehen so in den Listen, wie man sie ohne Sonderzeichen
  tippt: deutsch `Ä→AE` und `ß→SS`, dänisch/norwegisch `Å→AA`,
  schwedisch/finnisch `Ä→A`, sonst fällt der Akzent weg.
- In den Lösungswörtern stecken keine Eigennamen mehr: Im Deutschen entscheiden
  die Deklinations-Flags des Wörterbuchs, ob ein großgeschriebenes Wort ein
  Substantiv (`Blume` → Lösungswort) oder ein Name (`Petra` → keins) ist.
- Alle Listen laufen durch den Schimpfwortfilter des Projekts (alle 14 Sprachen
  gleichzeitig).
- LamaWiki, beide READMEs und die Spieleübersicht nennen die neuen Zahlen und
  den harten Modus; `pyinstall*.bat` packt `woordlistz/` mit in die `.exe`.

### UI v4.2 „Midnight Glass“ – neues Standard-Design – 2026-09-16

#### Neu
- **UI v4.2 „Midnight Glass“** als zehntes Design im Reiter
  **Erscheinungsbild**: ein tiefer Mitternachts-Verlauf, über den drei große,
  weiche Lichter in Indigo, Türkis und Magenta langsam treiben, darüber ein
  feines Filmkorn und ein paar spärliche Sterne. Panels und Buttons sehen aus
  wie mattes Milchglas: leicht aufgehellte Flächen mit einer hellen Lichtkante
  an der Oberkante. Unter den Titeln liegt eine Verlaufslinie, die zu den Seiten
  hin ausläuft, und eine ebensolche Linie trennt die Sidebar von der
  Spielfläche.
- Dasselbe Design auch in der **Web-Version** (Einstellungen → Design) –
  Desktop und Browser sehen identisch aus.
- Alle Texturen – Verlauf, Lichter, Korn, Sterne, Glasflächen – werden
  **prozedural erzeugt** und gecacht; es kommt keine einzige Bilddatei dazu.
  Damit läuft v4.2 unverändert in der `.exe` und im Browser über `file://`.

#### Geändert
- **Neue Spieler starten mit UI v4.2** statt mit v4.1 – in der Desktop-Version
  über `settings.json`, in der Web-Version über den Browser-Speicher. Wer schon
  ein Design gewählt hat, behält es; **UI v4.1** bleibt samt seinen vier
  Muster-Varianten ganz normal wählbar.
- Name und Beschreibung der Designkarte gibt es in allen **14 Sprachen**; die
  Standard-Markierung ist von der v4.1-Karte auf die v4.2-Karte gewandert.
- LamaWiki (Optionen) und alle READMEs nennen jetzt **zehn Designs**.

### UI v1 & UI v2 sind zurück – 2026-09-15

#### Neu
- **UI v2** als achtes Design im Reiter **Erscheinungsbild**: der Look der
  allerersten `ui.py` aus dem „UI Rework“ (Commit `08739d3`) – dunkler
  Navy-Verlauf mit Sternenfeld, Buttons mit statischem Glow, Akzentbalken und
  Pfeil, Titel mit Schatten und doppelter Akzentlinie, dazu die damaligen
  Sidebar-Farben samt durchgehender Akzentlinie unter dem Kopf. Bewusst so
  schlicht wie damals: keine Aurora, keine Funken, keine Screen-Übergänge,
  kein kreisender Logo-Schmuck.
- **UI v1** als neuntes Design: der Stand vor dem „UI Rework“ (Commit
  `cb71142`) – einfarbiger dunkler Hintergrund, flache Buttons, die sich nur
  über die Farbe als ausgewählt zeigen, schlichte Titel ohne Linie und die
  alten Sidebar-Farben. Keine Sterne, keine Rahmen, keine Schatten, keine
  Übergänge; die Pause ist wieder die einfache Abdunklung mit Text.
- Beide auch in der **Web-Version** wählbar (Einstellungen → Design).

#### Geändert
- `settings.json` akzeptiert jetzt `"theme": "v1"` und `"v2"`. Name und Beschreibung der
  Designkarte gibt es in allen 14 Sprachen; LamaWiki (Optionen) und alle
  READMEs nennen jetzt neun Designs.
- `ui.fx()` nimmt einen Standardwert an, damit Themes eigene Schalter haben
  können (`logo_glow`, `menu_orbit`, `style`), ohne alle anderen anzufassen.

### Minigolf: eigene Bahnen bauen, spielen und teilen – 2026-08-30

#### Neu
- **Bahn-Editor**: Der neue Reiter **MAPS** im Minigolf-Vorbereitungsbildschirm
  führt zur eigenen Sammlung. **Neu** öffnet den Editor: links die Bahn, rechts
  eine Palette mit vier Werkzeugen und **15 Hindernis-Typen**. Rechtecke zieht
  man auf, runde Dinge setzt ein Klick, ein Rohr braucht zwei (Eingang,
  Ausgang). Auswählen, Verschieben, Löschen, **Undo/Redo** (Tasten **U**/**Y**)
  und Rasterfang (**G**) gehören dazu.
- **Acht neue Hindernisse** - und zwar in der Engine, nicht nur im Editor:
  **Rohr** (versetzt den Ball ans andere Ende, Richtung und Tempo bleiben),
  **Eis** (fast reibungsfrei), **Klebefeld** (bremst extrem), **Booster**
  (einmaliger Schub beim Betreten), **Magnet** (zieht an oder stößt ab),
  **Einbahn-Tor** (nur in Pfeilrichtung durchlässig), **Drehscheibe** (nimmt
  den Ball mit nach außen) und **Sprungrampe** (der Ball fliegt über
  Hindernisse hinweg).
- **Freie Bahngröße**: von 60x80 bis 160x240 Einheiten, in Zehnerschritten.
  Der Platz passt sich je Bahn neu ein; was beim Verkleinern herausragen würde,
  rückt automatisch mit hinein.
- **12 Vorlagen** als Startpunkt: Leer, Rohr, Insel, Windmühle, Zickzack,
  Wasser, Puffer, Rampe, Eis, Labyrinth, Magnet und Schanze. Jede lässt neben
  ihrem Kunststück immer einen normalen Weg offen - alle zwölf sind vom Solver
  nachweislich in Par spielbar.
- **Teilen**: Der Knopf fragt nach deinem Namen als Ersteller und nach dem
  Dateinamen - der ist schon mit der **id** der Bahn vorbelegt. Von dort geht
  es entweder in den gewohnten **Speichern-Dialog** oder **direkt in den
  Downloads-Ordner**. Exportiert wird immer genau eine Bahn, als
  `.lamapgzmap`-Datei (JSON darin).
- **Importieren** liest so eine Datei wieder ein (`.json` wird auch
  angenommen). Ist die **id** schon vergeben, hängt der Import automatisch
  `-2`, `-3` … an, statt eine vorhandene Bahn zu überschreiben.
- **Wortfilter**: Map-Name, id und Ersteller-Name laufen beim Speichern,
  Exportieren **und** Importieren durch eine Prüfung gegen **alle 14
  Sprachen** - Sprache umstellen hilft also nicht. Die Listen stehen als
  Regex-Muster in `lang/swear/<code>.yml` bzw.
  `lang/lang.expansion/swear/<code>.yml`, je Eintrag das Muster und darunter
  das gemeinte Wort als Kommentar. Sie halten eingestreute Sonderzeichen,
  Leetspeak und gesperrt geschriebene Wörter aus.
- **Spielen**: Im MAPS-Reiter startet **Spielen** genau die gewählte Bahn (mit
  eigenem Bestwert je Bahn); im Reiter **SPIEL** gibt es als fünfte Kurswahl
  **Eigene**, die die ganze Sammlung als Runde spielt.
- **Test** im Editor probiert die Bahn sofort aus und kehrt danach ans Bauen
  zurück - ohne Bestwert, ohne Aufzeichnung.
- **96 neue Schlüssel** in allen **14 Sprachen**, dazu eine neue LamaWiki-Seite
  „Eigene Bahnen" (ebenfalls 14x), die erweiterte Hindernis-Liste auf der
  Minigolf-Seite und beide READMEs.

#### Geändert
- Das Zeichnen des Platzes liegt jetzt in `games/minigolf_draw.py` - Spiel und
  Editor malen dieselben 15 Typen mit demselben Code, statt getrennt
  auseinanderzulaufen.
- `InputEvent` transportiert zusätzlich das getippte **Zeichen**
  (`event.char`). Ohne das ließen sich in den neuen Textfeldern weder Umlaute
  noch Großbuchstaben eingeben - der Tkinter-keysym allein reicht dafür nicht.
- Neu in `ui.py`: **`ui.TextInput`**, das erste Eingabefeld des Projekts
  (Schreibmarke, Zeichenfilter, Platzhalter) - im Editor für Name und id, im
  Teilen-Dialog für Ersteller und Dateiname.
- Spiele können über `wants_escape` melden, dass sie **ESC** gerade selbst
  brauchen. Im Editor, im MAPS-Reiter und beim Test-Spielen heißt ESC deshalb
  „Abbrechen" statt „Pause"; überall sonst bleibt es wie bisher.
- Ältere Aufnahmen in `replay.json` laufen beim Laden durch
  `minigolf_gen.normalize` - sie kennen die neuen Hindernisse und die
  Bahngröße noch nicht und werden dabei ergänzt.
- Der Vorbereitungsbildschirm hat jetzt eine Reiterzeile; der Kurs-Block rückt
  entsprechend nach unten und hat mit **Eigene** einen fünften Knopf.
- Das Headless-Audit (`tests/newgames_audit.py`) prüft die eigenen Bahnen mit:
  Speicher-Rundlauf, id-Regeln, Wortfilter (inkl. Selbsttest der Listen und
  Fehlalarm-Prüfung gegen alle vorhandenen Oberflächen-Texte), je einem
  Physik-Test für die acht neuen Hindernisse, abweichenden Bahngrößen,
  Export/Import samt abgewiesener Fremddateien, allen zwölf Vorlagen per Solver
  und der Bildschirmaufteilung in fünf Auflösungen und allen 14 Sprachen.

### Minigolf: Stärke-Sperre auf der rechten Maustaste – 2026-08-29

#### Neu
- **Stärke-Sperre**: Die **rechte Maustaste** hält die Schlagstärke fest,
  solange sie gedrückt bleibt. Der Ladebalken bleibt genau da stehen, wo er
  beim Drücken war - mit fertig geladenem Schlag wartet man auf die Lücke in
  der Windmühle oder auf den Wanderblock und puttet erst im richtigen Moment.
  Loslassen lädt ganz normal weiter.
- **Die Sperre überlebt den Schlag**: Gehalten bleibt die Kraft über Putt,
  Bahnwechsel und **R** hinweg - der nächste Linksklick lädt dann nicht bei 5%
  neu, sondern schlägt exakt mit dem gemerkten Wert. Zwei gleich starke Schläge
  hintereinander sind damit zum ersten Mal möglich. Auch eine mit Pfeil
  hoch/runter eingestellte Stärke lässt sich so festnageln; die Pfeiltasten
  selbst ändern währenddessen nichts mehr.
- **Anzeige**: Der Ladebalken wird golden, bekommt ein Vorhängeschloss, eine
  helle Haltemarke am eingefrorenen Wert und einen ruhigen Puls; im HUD steht
  „Kraft gesperrt: 62%". Ziellinie, Schlägerkopf und ein pulsender Ring um den
  Ball färben sich mit - der Blick kann auf der Bahn bleiben.
- **Prozentzahl am Ladebalken**: Die Schlagstärke steht jetzt immer als Zahl
  neben dem Balken, gesperrt oder nicht - so lässt sich ein Wert von eben
  bewusst wiederholen.
- Neuer Schlüssel `golf.lock` in allen **14 Sprachen**, dazu die Minigolf-Seite
  im LamaWiki (Steuerung + Hindernisse) und beide READMEs.

#### Geändert
- Minigolf bekommt Rechtsklicks jetzt überhaupt gemeldet
  (`wants_right_click`); im Setup und am Rundenende bleiben sie wirkungslos,
  statt als Linksklick durchzurutschen.
- Eine **Pause hebt die Sperre selbst auf**: Pausiert kommt kein Loslassen der
  rechten Maustaste mehr an - sonst bliebe die Kraft eingefroren hängen.
- Das Headless-Audit (`tests/newgames_audit.py`) prüft die Sperre mit:
  Einfrieren, Schlag, Bahnwechsel, Pause, Setup-Klicks und den HUD-Text in drei
  Auflösungen und allen 14 Sprachen.

### Minigolf: Autoziel abschaltbar – 2026-08-29

#### Neu
- **Autoziel im Setup** (Standard AN): Bisher drehte sich der Schläger vor jedem
  Schlag von selbst zum Loch - am Abschlag, nach jedem Stopp und nach jedem
  Wasser-Strafschlag. Steht *Autoziel* auf AUS, bleibt die zuletzt gewählte
  Richtung stehen und gezielt wird von da an komplett selbst. Nur am Tee einer
  neuen Bahn gibt es keine „letzte" Richtung - dort zeigt der Schläger neutral
  bahnaufwärts, damit niemand mit dem Rücken zur Bahn startet. Die Einstellung
  liegt im Abschnitt `minigolf` von `settings.json` und gilt auch beim nächsten
  Start.
- **Taste Z** schaltet das Autoziel jederzeit um - im Setup genauso wie mitten
  in der Runde, wie man es von **G** (Ziellinie) und **P** (Aufnehmen) kennt.
- Neuer Schlüssel `golf.lbl_autoaim` in allen **14 Sprachen**, dazu die
  Minigolf-Seite im LamaWiki und beide READMEs.

#### Geändert
- **Setup-Screen aufgeräumt**: Ziellinie, Autoziel und Aufnehmen stehen jetzt
  als drei AN/AUS-Paare nebeneinander in einer Zeile statt in drei
  Einzelzeilen. So passt die neue Option auch auf 480x360 - dort sind die
  Knöpfe sogar höher als vorher (40 statt 31 px).
- **Der Optionsblock wächst mit der Auflösung**: Die Schriften taten das
  schon immer, der 370-px-Block nicht - lange Beschriftungen wie
  „Ligne de visée" hätten ab 800x600 nicht mehr über ihre Schaltergruppe
  gepasst. Was an Höhe übrig bleibt, verteilt sich außerdem auf die Abstände,
  statt unten leer zu stehen.
- Das Headless-Audit (`tests/newgames_audit.py`) prüft das Autoziel jetzt mit -
  samt Setup-Screen in fünf Auflösungen und allen 14 Sprachen.

### Replays für Minigolf & Bowling – 2026-08-29

#### Neu
- **Replays**: Minigolf und Bowling zeichnen jede Runde mit. Aufgenommen wird
  nicht die Eingabe, sondern die **tatsächliche Bahn von Ball und Pins** - Bild
  für Bild in einem festen 30-Hz-Raster. Die Wiederholung sieht deshalb exakt
  aus wie die gespielte Runde und bleibt auch dann gültig, wenn sich die Physik
  später ändert. Gezeichnet wird sie vom Spiel selbst: der Replay-Screen baut
  eine normale Spielinstanz und fährt sie über `replay_begin` /`replay_seek` /
  `replay_draw` durch die Aufnahme - samt HUD, Scorekarte und Bahn-Kulisse.
- **Speichern mit P**: Am Rundenende (Minigolf) bzw. Partie-Ende (Bowling)
  startet **P** die Wiederholung sofort; **S** legt sie ins Archiv, **Esc**
  führt zurück zum Rundenende - Weiter/Nochmal funktionieren danach wie
  gewohnt. Bei Minigolf gibt es dafür zusätzlich den Knopf **Replay** in der
  Knopfreihe.
- **Replay-Archiv** über den neuen Sidebar-Knopf **Replays**: ein Reiter je
  Spiel, je Eintrag Kurs bzw. Schwierigkeit, Ergebnis, Datum und Laufzeit.
  Enter spielt ab, Entf löscht (zur Sicherheit zweimal drücken), Tab wechselt
  das Spiel. Platz ist für 20 Aufnahmen je Spiel.
- **Wiedergabe wie ein Videoplayer**: Leertaste pausiert, Links/Rechts springt
  zur vorigen/nächsten Sequenz (Schlag bzw. Wurf), Hoch/Runter regelt das Tempo
  (0,5x / 1x / 2x / 4x), ein Klick in die Fortschrittsleiste springt an eine
  beliebige Stelle. Jede Sequenz bekommt einen kurzen Vorlauf mit Ziellinie und
  einen Nachlauf mit dem Ergebnis; Einlochen, Strike und Spare sind zu hören.
  Die Bedienleiste blendet sich nach drei Sekunden aus.
- **Abschaltbar**: Der Willkommens-Screen beim ersten Start und die Optionen
  (Reiter *Allgemein*, Gruppe **Aufnahme**) haben den Schalter **Replays
  aufzeichnen**. Aus bedeutet: es wird gar nichts mitgeschnitten.
- **Zwei neue Erfolge**: *Regisseur* (erstes Replay gespeichert) und *Archivar*
  (fünf Replays im Archiv) - damit sind es 85.
- **Neue Wiki-Seite „Replays"** in allen 14 Sprachen, dazu `replay.json` auf der
  Seite *Speichern & Highscores*.

#### Geändert
- Die Aufnahmen liegen in einer **eigenen Datei `replay.json`** neben `mem.json`
  (Unterkategorie je Spiel, kompakt geschrieben). Eine Minigolf-Runde kostet je
  nach Schlagzahl 30-80 KB, eine Bowling-Partie rund 90 KB - Pins stehen nur
  dann in einem Bild, wenn sie sich bewegt haben.
- Ein Bahn-Neustart mit **F** wirft die Schläge dieser Bahn auch aus der
  laufenden Aufnahme, damit Scorekarte und Wiederholung zusammenpassen.

#### Behoben
- **Bowling stürzte am Ende einer Partie ab**: nach dem letzten Wurf stand der
  Schritt-Zähler auf 4 und damit außerhalb von `STEPS` - HUD und Reglerzeile
  liefen beim Zeichnen in einen `IndexError`, der die Game-Loop mitriss. Der
  Wert wird jetzt gedeckelt, und die Reglerzeile erscheint nur noch im
  laufenden Spiel.

### Minigolf: Weiter statt Wiederholung, Bahn-Reset mit F – 2026-08-28

#### Neu
- **Weiter-Knopf am Rundenende**: Statt immer wieder denselben Neuner-Satz zu
  spielen, führt **Weiter** zum nächsten Kurs - *Classic* → *Pro* → *Tour 1* →
  *Tour 2* → … bis Tour 38. Der Knopf nennt sein Ziel (z.B. „Weiter: Tour 12"),
  daneben stehen **Nochmal** (gleicher Kurs) und **Setup**. Tasten: Enter =
  weiter, R = nochmal, S = Setup. Bei *Random* und nach dem letzten Tour-Kurs
  entfällt der Knopf - dort gibt es ohnehin jedes Mal neue Bahnen.
- **Taste F setzt die laufende Bahn zurück**: Schläge zurück auf 0, Ball zurück
  aufs Tee - gleiche Bahn, gleicher Spieler, gleicher Kurs. Bereits gespielte
  Bahnen bleiben in der Scorekarte stehen. Wirkt auch, während der Ball rollt
  oder ein Schlag geladen ist.

#### Geändert
- Das Rundenende-Banner wächst jetzt mit den Schriftgrößen mit (Überschrift,
  Ergebnis, Bestwert, Knopfreihe, Tastenzeile) - bei 1280x960 saß der Titel
  vorher auf der Ergebniszeile.

### Fehlerbehebungen & vier neue Wiki-Seiten – 2026-08-28

#### Behoben
- **`Game.ach_event()` nahm keinen Wert entgegen**, obwohl vier Spiele einen
  übergeben: Poker (`_sync_chips`, praktisch jede Hand), Snake im Modus
  *Competitive* (jeder Level-Aufstieg), Bowling (ab 200 Punkten) und Pinball
  (ab 50.000). Der `TypeError` riss die Game-Loop mit, das Bild fror ein.
  Nebenbei waren die vier wertgebundenen Erfolge *poker_rich*, *snake_comp5*,
  *bowl_200* und *pin_high* dadurch überhaupt nicht erreichbar.
- **Die Game-Loop überlebt jetzt eine Ausnahme**: `App._loop()` plante das
  nächste Frame als letzte Anweisung - warf ein Frame einen Fehler, wurde nie
  wieder gezeichnet, obwohl das Fenster noch reagierte. Der Rumpf steckt jetzt
  in `_frame()`, geplant wird in einem `finally`.

#### Neu
- **Wiki-Seiten für Schach, Mühle, Simon und Billard** in allen 14 Sprachen.
  Diese vier Spiele hatten als einzige keine Seite - der Wiki-Knopf in ihrem
  Vorspiel-Screen landete auf der Startseite statt beim Spiel. Jede Seite hat
  drei Abschnitte (Regeln bzw. Varianten, Steuerung, KI & Punkte); damit hat
  jedes der 42 Spiele seine eigene Wiki-Seite.

### UI v4.1.1 bis v4.1.4: Zickzack-Muster – 2026-08-28

Vier neue Designs im Reiter **Erscheinungsbild** - alle sind exakt UI v4.1,
nur der Hintergrund ist ein gekacheltes **Zickzack-Muster** statt Verlauf und
Sternenfeld. Untereinander unterscheiden sie sich allein durch die zwei
Musterfarben (die erste ist jeweils die dominante).

#### Neu
- **UI v4.1.1**: **Schwarz** (#000000) auf **Anthrazit** (#424242).
- **UI v4.1.2**: das **Akzentblau** (91, 141, 239) auf dem dunkleren
  `ACCENT_SOFT`-Blau (64, 94, 156) - beide Farben gab es schon in der Palette.
- **UI v4.1.3**: der **Indigo-Akzent von UI v4** (#5b8def) auf Schwarz -
  die kontrastreichste der vier Varianten.
- **UI v4.1.4**: der **Graphit-Ton von UI v4** (#252934) auf Schwarz -
  die dezenteste.
- `ui.draw_zigzag()` ist ein 1:1-Nachbau der CSS-Vorlage aus drei
  `conic-gradient`-Ebenen (Kachel 34x17 px). Für weiche Diagonalen wird die
  Kachel 4-fach übersampelt gezeichnet, einmalig gecacht und dann zeilenweise
  geblittet - der Hintergrund kostet damit pro Auflösung nur einen Aufbau.
- Jede Karte im Options-Reiter zeigt ihr Muster in der Mini-Vorschau.

#### Geändert
- Der Reiter **Erscheinungsbild** ordnet die inzwischen sieben Designs in
  einem **Raster** an: aus ein bis drei Reihen wird die Aufteilung mit der
  größten Kartenfläche gewählt (1280x960 und 640x480 ergeben 4 + 3, ganz
  kleine Auflösungen drei Reihen). Beschreibungen richten ihre Zeilenzahl
  nach der Kartenhöhe, enden bei Platzmangel mit "..." und lassen der
  "AKTIV"-Plakette Platz.
- In den Muster-Themes entfällt das Sternenfeld (auf dem Muster wäre es nur
  Bildrauschen); **Saturn und Schwarzes Loch bleiben**. Die Vignette ist etwas
  kräftiger (64 statt 48), damit Reiter und Fußzeile über dem Muster lesbar
  bleiben.
- `settings.json` akzeptiert jetzt `"theme": "v411"` bis `"v414"`; alle
  14 Sprachen haben die neuen Namen und Beschreibungen, das Wiki einen
  eigenen Abschnitt **Erscheinungsbild**.

### Minigolf-Tour: 360 Bahnen – 2026-08-26

Minigolf wächst vom Achtzehn-Bahnen-Platz zur **Tour**: zu den 18 handgebauten
Bahnen kommen **342 erzeugte** dazu - zusammen **360 Bahnen in 40 Kursen**.

#### Neu
- **Bahn-Generator** `games/minigolf_gen.py`: 38 Tour-Kurse zu je 9 Bahnen,
  vollständig aus einem Seed erzeugt. Kurs 7, Bahn 3 sieht bei jedem Start und
  auf jedem Rechner gleich aus - gespeichert werden muss dafür nichts.
- **Zehn Bahnfamilien** über vier Schwierigkeitsstufen: gerade Bahn, Wandreihen
  mit versetzten Lücken, Dogleg, Gummipuffer-Feld, Wasserteiche, Rampe,
  Chicane, Windmühlen-Korridor, Inselgrün und Wanderblock-Schleuse. Par 2 bis 5,
  Gesamt-Par je Kurs zwischen 26 und 39.
- **Passierbarkeit ist eingebaut, nicht erhofft**: jede Familie legt zuerst den
  Weg vom Abschlag zum Loch fest und baut die Hindernisse darum herum -
  Wandlücken sind nie schmaler als 12 Einheiten (Ball-Durchmesser 3,4), Wasser
  liegt nur neben dem Weg, ein Wanderblock ist stets schmaler als seine Lücke,
  und Mühlenflügel lassen seitlich Platz.
- **Kurswahl im Setup**: vier Knöpfe (Classic / Pro / Tour / Random) und darunter
  eine Zeile mit Pfeilen zum Blättern durch die 38 Tour-Kurse samt Anzeige des
  Gesamt-Pars. Der Bestwert wird je Tour-Kurs einzeln gespeichert.
- **Random** zieht jetzt aus allen 360 Bahnen statt nur aus den 18 gebauten.
- **Aufnehmen abschaltbar**: Die Regel *nach acht Schlägen ist die Bahn vorbei*
  bleibt Standard, lässt sich aber im Setup (oder mit **P**) ausschalten - dann
  wird bis zum Einlochen weitergespielt. Der Setup-Screen hat dafür eine eigene
  AN/AUS-Zeile bekommen und passt seine Höhe jetzt an die Auflösung an, damit
  auch 480x360 alle fünf Blöcke zeigt.
- **Schlag abbrechen mit R**: Wer die Maustaste hält, die Kraft schon geladen
  hat und es sich anders überlegt, drückt **R** - der Ball bleibt liegen, der
  Schlag zählt nicht, und nach dem Loslassen lässt sich ganz normal neu
  aufladen. Ein kurzer Hinweis in der Kopfzeile bestätigt den Abbruch.

#### Geändert
- `tests/newgames_audit.py` prüft nicht mehr nur die 18 gebauten Bahnen, sondern
  **alle 342 erzeugten** dazu: freie Lage von Abschlag und Loch, Einlochbarkeit
  per Solver (simuliert echte Schläge) und Reproduzierbarkeit aus dem Seed.
  Laufzeit rund eine Minute.
- `settings.json` merkt sich den zuletzt gewählten Tour-Kurs (`minigolf.tour`);
  die Platzmaße stehen jetzt einmalig in `minigolf_gen.py`.
- Sidebar-Untertitel und beide READMEs nennen die neue Zahl: 360 Bahnen.

### Minigolf, Pinball & Bowling – 2026-08-26

Drei neue Sportspiele auf einen Schlag (Nr. 40-42): **Minigolf** mit 18
handgebauten Bahnen, ein vollwertiger **Pinball**-Automat mit Multiball und
**Bowling** mit echter Pin-Physik - alle drei mit Mehrspieler-Modus, Erfolgen,
Wiki-Seite und allen 14 Sprachen.

#### Neu
- **Minigolf**: 18 handgebaute Bahnen in drei Kursen - *Classic* (sanfter
  Einstieg), *Pro* (Inselgrün, Doppelmühle, Wanderblöcke) und *Random* (neun
  zufällig gezogene und gespiegelte Bahnen). Sand bremst, Rampen beschleunigen,
  Wasser kostet einen Strafschlag, dazu Gummipuffer, Windmühlen und
  Wanderblöcke. Gezielt wird mit der Maus, die Kraft lädt bei gedrückter
  Maustaste (alternativ Pfeile + Leertaste), **G** blendet die Ziellinie um.
  Scorekarte mit Par je Bahn, **500 Punkte extra für ein Hole-in-One**; zu zweit
  spielt jeder dieselbe Bahn nacheinander. Bestwert je Kurs in `mem.json`
  (Abschnitt `minigolf`).
- **Pinball**: drei Tische - *Classic* (drei Pop-Bumper, eine Target-Bank),
  *Space* (vier Bumper im Karo, zwei Banks) und *Lama* (offenes Feld, sechs
  Targets im Bogen). Mit Slingshots, vier **L-A-M-A**-Rollover-Bahnen, Saucer
  mit Ball-Lock und **Multiball samt Jackpot**, dazu sechs Sekunden Ball-Save,
  Nudge über die Hoch-Taste, **TILT** bei dreimal zu hastigem Anstoßen und ein
  Multiplikator bis x5. 3 oder 5 Bälle je Partie, zu zweit im Wechsel. Ein zu
  schwacher Schuss ist kein Beinbruch: der Ball rollt in die Schussbahn zurück
  und darf noch einmal. Bestwert je Tisch in `mem.json` (Abschnitt `pinball`).
- **Bowling**: zehn Frames mit vollständiger Strike-/Spare-Wertung inklusive
  Bonuswürfen im zehnten Frame (Maximum: die 300) und einer Scorecard mit X, /
  und laufender Summe. Der Wurf läuft in vier Schritten - Position, Ziel, Effet
  und Kraft - über Regler, die von allein pendeln und sich mit Links/Rechts auch
  von Hand einstellen lassen. Zehn Pins mit echter Masse werfen sich gegenseitig
  um; die Bahn ist vorn geölt, der **Hook** greift erst im hinteren Drittel.
  Bestwert je Schwierigkeit in `mem.json` (Abschnitt `bowling`).
- **9 neue Erfolge**: Punkte-Meilensteine für alle drei Spiele plus
  Hole-in-One, Runde unter Par, Multiball, 50 000 Punkte im Pinball, Turkey
  (drei Strikes in Folge) und ein 200er-Bowlingspiel - jetzt **83 insgesamt**.
- **93 Übersetzungs-Keys je Sprache** und **drei neue Wiki-Seiten** in allen
  14 Sprachen (LamaWiki: 43 Seiten); beide READMEs ergänzt.
- **Headless-Audit** `tests/newgames_audit.py`: prüft alle 18 Minigolf-Bahnen
  per Solver auf Einlochbarkeit (und freie Abschlag-/Lochpositionen), jeden
  Pinball-Tisch auf gelungenen Abschuss und ein hängerfreies Partie-Ende sowie
  die Bowling-Wertung gegen Referenzspiele (300er, lauter Spares, gemischt)
  samt Pin-Physik und Frame-Logik.

#### Geändert
- Sidebar: drei neue Piktogramme (Fahne im Loch, Flipper mit Kugel, Pin mit
  Kugel) mit eigenen Akzentfarben; die Sammlung zählt jetzt **42 Spiele**.
- `settings.json` kennt die Abschnitte `minigolf`, `pinball` und `bowling`
  (Kurs & Ziellinie, Tisch & Ballzahl, Schwierigkeit & Zielhilfe) - alle drei
  merken sich ihre Setup-Auswahl.

### Block Jump: Minecraft-Skin – 2026-08-12

Optik-Update: **Block Jump** sieht jetzt aus wie sein Vorbild - echte
Pixeltexturen auf allen Blöcken, eine Steve-Figur, die Hand im Ego-Modus,
ein Beacon-Strahl am Ziel, Pixel-Wolken und ein HUD mit Herzen.

#### Neu
- **Blocktexturen** (16×16, im Code prozedural erzeugt - keine Bilddateien):
  Gras mit grünem Überhang an den Seiten, Erde, Stein, Eichenbretter,
  Diamantblock (Ziel), Schleimblock (der frühere Sprungblock) sowie Holz für
  Leiter und Zaun. Der Software-Renderer zerlegt jede Fläche perspektivisch
  korrekt in Texel-Vierecke, die Flächenhelligkeit folgt dem Vorbild
  (oben hell, Seiten abgestuft, Unterseite dunkel).
- **Detailstufe nach Abstand** mit **Taste T**: hoch / niedrig / aus. Gleich
  eingefärbte Texel werden vorab zu Rechtecken zusammengefasst, dadurch kostet
  die neue Optik statt rund 18.000 nur noch etwa 3.000 Vierecke je Bild
  (1280×720: ~14 ms in *hoch*, ~6 ms in *niedrig*, ~3 ms mit *aus*). Die Wahl
  wird in `settings.json` gespeichert (`blockjump.textures`).
- **Steve als Spielfigur** in der Verfolgerkamera: Kopf mit Gesicht, Torso,
  Arme und Beine schwingen beim Laufen, der Kopf neigt sich mit dem Blick.
- **Hand im Ego-Modus** samt Lauf-Bob, **Beacon-Strahl** über dem Ziel und
  rotierende **Gold-Barren** anstelle der bisherigen Kristall-Coins.
- **Himmel**: Minecraft-Blau mit quadratischer **Sonne** und driftenden
  **Pixel-Wolken** weit über der Karte.
- **HUD im Spielstil**: **Herzen** für die Leben, Gold-Barren als Coin-Zähler,
  Schattenschrift und ein Fadenkreuz wie im Vorbild.

#### Geändert
- Der Sprungblock ist jetzt optisch ein **Schleimblock**, das Ziel ein
  **Diamantblock** mit Lichtstrahl. Spielverhalten, Level-Generierung und
  Physik bleiben unverändert - `tests/blockjump_audit.py` läuft weiterhin
  fehlerfrei durch alle 45 Level.

### Block Jump Bugfixes – 2026-08-12

Wartungs-Update: **Block Jump** ist jetzt tatsächlich durchspielbar - der
Leiter-Aufstieg endete bisher in praktisch jedem Level in einer Sackgasse.

#### Behoben
- **Leitern waren Sackgassen**: Das Folge-Pad wurde direkt über dem
  Leiterschacht gebaut und überschrieb die oberste Sprosse - der Spieler
  stieß beim Klettern mit dem Kopf an und kam nie oben an. Das Pad beginnt
  jetzt hinter der Stützwand, der Schacht bleibt offen (betraf 44 von 45
  geprüften Leveln, inklusive Level 1 aller drei Modi).
- **Leiter-Coin unerreichbar**: Der Coin hing einen Block vor der Leiter
  statt in der Kletterspalte - jetzt wird er beim Aufstieg eingesammelt.
- **Taste C (Maus fangen/frei) war wirkungslos**: Die Zeichenroutine
  überschrieb den Schalter in jedem Frame; der HUD-Hinweis stimmt jetzt
  wieder.
- **Unsichtbare Wände**: Zäune und Sprungblöcke kollidierten als volle
  1×1×1-Blöcke, obwohl sie viel kleiner gezeichnet werden. Zäune sind jetzt
  0,8 Blöcke hoch (bequem überspringbar, wie im README beschrieben),
  Sprungblöcke blockieren seitlich gar nicht mehr und katapultieren auch
  beim Hineinlaufen; die Pad-Fläche unter beiden wird nicht mehr weggecullt
  (sichtbare Löcher) und die 3rd-Person-Kamera bleibt nicht mehr an Zäunen
  hängen.
- **Schwer-Modus**: Bei maximaler Lücke (4 Blöcke) konnten 1 Block tiefe
  Ziel-Pads plus Aufwärtsversatz framegenau-unmögliche Sprünge erzeugen -
  große Lücken erzwingen jetzt ebene/abwärts führende, tiefe Lande-Pads.
- **Tastenbelegung**: Block Jump ignorierte die in den Optionen belegten
  Tasten (fest WASD/Pfeile) - läuft jetzt über die zentrale Belegung wie
  die übrigen Spiele (Pfeiltasten bleiben als Fallback erhalten).
- **Einstellungen gingen verloren**: Ansicht/Motion-Blur/Empfindlichkeit/
  Maus-Richtung überlebten den Neustart nicht (fehlender
  `blockjump`-Abschnitt in den Settings-Defaults).

#### Neu
- **Headless-Audit** `tests/blockjump_audit.py`: prüft alle 45 Level
  (1-15 × 3 Modi) automatisch auf offene Leiterschächte, erreichbare Coins,
  per Sprungphysik schaffbare Lücken und simuliert jeden Leiter-Aufstieg.

### Tower Defense – 2026-08-06

Spiel Nr. 39: ein komplettes **Tower Defense** mit endlosen Wellen, 4 Karten
und 3 Umfangs-Modi - inklusive Erfolgen, Wiki-Seite und Übersetzungen in alle
14 Sprachen.

#### Neu
- **Tower Defense** (`games/lamatowerdefense.py`): Gegner laufen in Wellen einen
  Pfad entlang; Türme daneben bauen, ausbauen (bis 3 Stufen) und mit **70%
  Erstattung verkaufen**. Jeder Durchbruch kostet Leben (Bosse 5), bei 0 ist
  Schluss - die Punkte zählen als Highscore.
- **4 Karten** mit eigenem Pfad und HP-Schwierigkeitsfaktor: **Wiese**,
  **Schlucht**, **Kreuzung** (der Pfad kreuzt sich - stark für Türme, dafür
  zähere Gegner) und **Spießrutenlauf** (gebaut wird nur direkt am Pfad).
  Gesperrte Karten schaltet die beste erreichte Welle frei (5/10/15); die
  Bestwelle je Karte wird gespeichert (Abschnitt `lamatowerdef` in `mem.json`).
- **Endlos-Wellen per Formel** statt Wellen-Skript: Budget, Gegner-HP und
  Beute skalieren mit der Wellen-Nummer (ab Welle 25 zusätzlich exponentiell);
  alle **8 Wellen ein Boss**, jede 5. Welle eine Themen-Welle, der
  Karten-Faktor greift über eine Anlaufkurve erst ab Welle 10 voll.
- **3 Modi** im Vorspiel-Screen: **Klassisch** (7 Türme, Hauptmodus),
  **Kompakt** (4 Türme, 2 Stufen, mehr Startgold) und **Maximal** (11 Türme,
  **A/B-Spezialisierung** auf höchster Stufe, Spezialgegner und
  Aktiv-Fähigkeiten **Meteor [Q] / Frostnova [W] / Goldsegen [E]**).
- **11 Turmtypen**: Pfeil, Kanone (Splash), Frost (bremst + enttarnt),
  Scharfschütze (Riesenreichweite, Panzerbrecher, Luft), Gift (Schaden über
  Zeit), Tesla (Kettenblitz), Banner (+20%-Aura), Mörser (Bogenschuss über
  fast das ganze Feld), Flak (Luftabwehr), Laser (Dauerstrahl mit Aufladung)
  und Goldbank (Einkommen) - jeder mit eigener A/B-Verzweigung im
  Maximal-Modus (22 Zweige, z. B. Doppelschuss/Durchschlag beim Pfeilturm).
- **11 Gegnertypen**: Läufer, Sprinter, Schwarm, Panzer, Gepanzerte
  (Schadensabzug), Regenerierer, Teiler, **Flieger** (eigene Luftroute, nur
  Luftabwehr trifft), **Getarnte** (nur bei Frost/Banner/Scharfschütze
  sichtbar), **Heiler** (Heil-Aura) und **Boss**.
- **Komfort**: Geist-Vorschau mit Reichweitenkreis, Rechtsklick bricht ab,
  **1-9** Schnellwahl, **[F]** Tempo x2, **[G]** alle Reichweiten, Mausrad
  scrollt das Baumenü, **+5% Zinsen** je Bauphase, Wellen-Bonus.
- **5 neue Erfolge**: Punkte-Meilenstein 10 000 sowie **Königsjäger** (erster
  Boss), **Wellenbrecher** (Welle 20), **Festung** (Welle 10 ohne Verlust)
  und **Endausbau** (Turm voll ausgebaut) - jetzt 74 Erfolge insgesamt.
- **76 neue Übersetzungs-Keys je Sprache** und eine **neue LamaWiki-Seite** in
  allen 14 Sprachen (das Wiki hat jetzt 40 Seiten); beide READMEs (DE/EN +
  12 weitere Sprachen) um Tabellenzeile und Feature-Details ergänzt.

#### Geändert
- Sidebar: neues gezeichnetes **Burgturm-Icon** und Terracotta-Akzentfarbe
  für Tower Defense (`ui.GAME_COLORS`); die Spieleliste zählt damit 39 Spiele.

### Erfolge & Statistiken – 2026-07-26

Das Progression-Update: Die Sammlung zählt jetzt, was du spielst - **69 Erfolge**
und dauerhafte **Spielerstatistiken** über alle 38 Spiele, erreichbar über den
neuen Sidebar-Button **„Erfolge & Statistik"** (Pokal-Symbol).

#### Neu
- **69 Erfolge** in drei Kategorien:
  - **Allgemein** (21): sammlungsweite Ziele wie gestartete Partien (1/10/50/200),
    ausprobierte Spiele (5/15/alle 38), Gesamtspielzeit (1 h/5 h/20 h), gebrochene
    Rekorde (1/10/25), Siege (10/50, in 5 verschiedenen Spielen) - plus Extras wie
    **Nachteule** (nach Mitternacht), **Frühaufsteher**, **Polyglott**
    (Sprachwechsel), **Leseratte** (Wiki geöffnet) und **Ganz mein Stil**
    (Snake-Personalisierung).
  - **Punkte-Meilensteine** (30): eine faire Ziel-Punktzahl je Spiel, abgestimmt
    auf dessen Punkteskala (z. B. Snake 500, Tetris 10 000, Flappy 25, Simon 12).
  - **Besondere Momente** (18): Schachmatt gegen die KI, KNIFFEL, die
    2048er-Kachel, Tetris-Vierfachreihe, Blackjack mit zwei Karten, Wordle in
    ≤ 2 Versuchen, Galgenmännchen ohne Fehlversuch, fehlerfreies Sudoku,
    Minesweeper- und Solitär-Sieg, perfektes Memory, leergefressenes
    Pacman-Labyrinth, alle Breakout-Level, alle Frogger-Buchten, 100 %
    Aim-Trainer-Genauigkeit, 2 000 Poker-Chips, Snake-Prestige und
    Competitive-Level 5.
- **Toast-Einblendung**: goldenes Banner mit gezeichnetem Abzeichen und kleinem
  Arpeggio oben rechts beim Freischalten - auch mitten im Spiel.
- **Erfolge-&-Statistik-Screen**: Reiter **Erfolge** mit Gesamtfortschritts-Balken,
  Kategorien, Fortschrittsanzeige je Erfolg und Freischalt-Datum; Reiter
  **Statistiken** mit Übersichtskarten (Gesamtspielzeit, Partien, Siege, Rekorde,
  ausprobierte Spiele, Erfolgs-Stand), **Lieblingsspiel** und einer nach Spielzeit
  sortierten **Pro-Spiel-Tabelle** (Partien · Zeit · Siege · Bestwert). Ein Klick
  auf eine Zeile springt direkt ins Spiel.
- **Persistente Statistiken** je Spiel: Partien (inkl. Neustarts), **aktive**
  Spielzeit (Pausen/Menüs zählen nicht), Siege/Niederlagen (Spiele mit klarem
  Ausgang, im Einzelspieler), gebrochene Rekorde, zuletzt gespielt - gespeichert
  als Abschnitte `stats`/`achievements` in `mem.json`.
- **Bestand zählt**: Beim ersten Start nach dem Update werden vorhandene
  Highscores automatisch angerechnet (ohne Toast-Feuerwerk).
- **94 neue Übersetzungs-Keys in allen 14 Sprachen** (~1 300 neue Strings) und
  eine **neue LamaWiki-Seite** „Erfolge & Statistiken" - ebenfalls in allen
  14 Sprachen (das Wiki hat jetzt 39 Seiten).

#### Geändert
- `game_base.py`: neue Hooks für alle Spiele - `report_result()` (Sieg/Niederlage,
  zählt einmal pro Partie) und `ach_event()` (besondere Momente); in über
  30 Spiele eingebaut.
- Zentrale Game-Loop (`main.py`): zählt Partie-Starts und -Neustarts sowie
  Spielzeit automatisch; die Rekord-Speicherung prüft jetzt zusätzlich
  Meilenstein-Erfolge. Statistiken werden gedrosselt geschrieben (höchstens
  alle 20 s sowie bei Partie-Start/-Ende und beim Beenden).

### Sprach-Erweiterung: 9 neue Sprachen – 2026-07-21

Die Oberfläche gibt es jetzt in **14 Sprachen**. Neu hinzugekommen sind neun
Sprachen, die – wie zuvor Spanisch und Portugiesisch – beim ersten Start hinter
dem Knopf **„Weitere Sprachen"** liegen: **Polski, Türkçe, Dansk, Norsk,
Svenska, Suomi, Čeština, Slovenščina, Hrvatski**.

#### Neu
- **9 neue UI-Sprachen**, je vollständig übersetzt (802 Texte pro Sprache):
  Polnisch, Türkisch, Dänisch, Norwegisch, Schwedisch, Finnisch, Tschechisch,
  Slowenisch, Kroatisch.
- **LamaWiki komplett mitübersetzt**: alle 38 Wiki-Seiten in jeder der neun
  neuen Sprachen.
- **Dokumentation**: für jede der neun Sprachen ein vollständiger
  README-Abschnitt in `other.readme.md` (Format wie FR/ES/PT).
- Die neuen Sprachdateien liegen gebündelt in `lang/lang.expansion/` bzw.
  `lamawiki/lang.expansion/` – die Kern-Sprachen (de/en/fr/es/pt) bleiben unberührt.

#### Geändert
- **Sprachauswahl als Raster**: Sprach- und Willkommens-Screen zeigen die nun
  14 Sprachen in einem responsiven Raster mit automatischer Schriftanpassung –
  passt sauber von 480×360 bis 1280×960 (langer Name wie „Slovenščina" läuft
  nicht mehr über).
- Loader (`i18n.py`, `lamawiki.py`) durchsuchen zusätzlich die
  `lang.expansion`-Unterordner; fehlt ein Text, wird wie gehabt auf Deutsch
  zurückgegriffen.
- Build-Skripte (`pyinstall.bat`, `pyinstall-pyarmor.bat`) packen die neuen
  `lamawiki/lang.expansion/*.json` mit in die EXE.

#### Hinweise
- Wordle und Hangman nutzen für die neuen Sprachen vorerst die englischen
  Wortlisten (eigene Listen können später ergänzt werden).

### Games Rework – 2026-07-19

Die bisher größte Überarbeitung: **alle 38 Spiele** wurden in einem Durchgang auf
einen einheitlichen Stand gebracht (Optik, Konsistenz, Übersetzungen, Bugfixes).

#### Neu
- **Einheitlicher Spiel-Look**: Alle 38 Spiele nutzen jetzt die Theme-Palette und
  -Schrift des Menüs. HUDs, Setup-Screens und Overlays folgen dem in den Optionen
  gewählten Design (v4.1 / v4 / Classic) - die Spielfelder behalten ihre
  Identitätsfarben (Filz-Grün, Pacman-Labyrinth, Tetris-Steine …).
- **Sprachabhängige Spielnamen** im Menü: Schach → *Chess/Échecs/Ajedrez/Xadrez*,
  Mühle → *Nine Men's Morris*, Vier gewinnt → *Connect Four*, Panzer-Duell →
  *Tank Duel*, 3D-Labyrinth → *3D Maze*, Dame → *Checkers*, Billard → *Billiards*,
  Galgenmännchen → *Hangman*, Schiebepuzzle → *Sliding Puzzle*.
- **Responsive überall**: Jedes Spiel übernimmt Auflösungswechsel mitten im Spiel
  sauber (Schriften, Layout, Spielfeld) - 11 ältere Spiele konnten das vorher gar
  nicht, 3 weitere nur teilweise.
- **Blackjack**: Austeil- und Aufdeck-Animationen laufen jetzt wirklich (Karten
  fliegen aus dem Schuh, Hole-Card dreht sich mit Sound) - der Code existierte,
  wurde aber nie ausgeführt.
- 13 neue Übersetzungs-Keys in **allen 5 Sprachen** (u. a. Schwierigkeitsgrade in
  Breakout/Tic-Tac-Toe, Flappy-Medaillen, Sudoku-Löschtaste).

#### Geändert
- Gemeinsame Basis (`game_base.py`): sprachabhängige Namen (`LocalizedName`),
  dokumentierte Hooks (`on_surface_changed`, `capture_mouse`, `MODES`),
  Theme-Schriften und Akzentfarbe für alle Spiele automatisch.
- Setup-Screens einheitlich über die UI-Bausteine (`draw_title`, `draw_button`,
  `draw_footer`), Layouts skalieren mit der Fensterhöhe statt fester Pixelwerte.
- Game-Over einheitlich: transluzentes Panel mit Akzent-Rahmen, pulsierender
  Hinweis; **Enter und Leertaste** (oft auch Mausklick) starten überall neu.
- Kartenspiele: gemeinsamer Tisch-Look (`make_felt`), themenfarbene Kartenrücken
  in der Akzentfarbe des jeweiligen Spiels.
- Performance: Vollflächen-Alpha-Fills pro Frame durch gecachte Overlays ersetzt
  (u. a. Aim Trainer, Tunnel Racer, Pacman, Tetris, Tanks, Sudoku, Simon);
  2048 erzeugte pro Kachel und Frame eine neue Schriftart - jetzt gecacht.
- Aufgeräumt: 15 duplizierte Akzentfarb-Konstanten, tote Variablen/Zweige und
  ungenutzte Konstanten entfernt.

#### Behoben
- **Breakout**: Bomben-Steine explodierten doppelt (doppelte Punkte/Drops);
  Feuerball übersprang Steine bzw. traf falsche; Power-Up-Fallgeschwindigkeit
  war framerate-abhängig.
- **Schach**: Maus-Cursor saß gespiegelt, wenn der Mensch Schwarz spielte;
  „Schwarz gewinnt" war im Mehrspieler praktisch unsichtbar (schwarz auf dunkel).
- **Mühle**: Schloss die KI eine Mühle, verschwand die Anzeige sofort - jetzt
  leuchtet sie ~1,2 s golden.
- **Solitär**: Undo nach dem Talon-Ziehen ließ Karten dauerhaft aufgedeckt
  (Schummel-Bug); Undo während des Ziehens konnte Stapel beschädigen.
- **Poker**: Klickflächen zum Halten in 5 Card Draw saßen unterhalb der Karten;
  Pleite im Video Poker kam ohne Sound.
- **Frogger**: Nach Game Over lief die Spiellogik weiter - der Timer „tötete"
  den Frosch in Schleife (Leben negativ, Sound-/Vibrations-Spam).
- **Snake**: Nach einem Auflösungswechsel startete das Brett mit den alten Maßen.
- **Panzer-Duell**: Pfeiltasten waren im Einzelspieler tot; Auflösungswechsel am
  Match-Ende zeichnete die alte Arena.
- **T-Rex Runner**: Umbelegte Sprungtaste blieb beim Loslassen „hängen"
  (Dauer-Niedrigsprung).
- **Minesweeper**: Rechtsklick auf den Smiley startete das Spiel neu;
  Schwierigkeitsname konnte den Minenzähler überlappen.
- **Asteroids**: Schiffe spawnten nach Auflösungswechsel außerhalb des Bildes;
  Steuerungs-Hinweis erschien nie, wenn man länger im Setup war.
- **Pacman**: Punkte-Popups saßen nach Auflösungswechsel an falschen Stellen.
- **Invaders**: Fliegende Schüsse wechselten die Farbe beim Waffenwechsel.
- **Pong**: Namens-Label lief bei langen Übersetzungen aus dem Bild.
- **Doodle Jump**: Blickrichtung des Doodlers wurde nie gespiegelt;
  Monster-Kill mit Propellerhut war stumm.
- **Tic-Tac-Toe**: Schwierigkeits-Buttons waren im Mehrspieler klickbar,
  obwohl ausgegraut (jetzt deaktiviert).
- **Dame**: „S = Setup" funktionierte entgegen dem Hinweis nur im Einzelspieler.
- **Reversi/Vier gewinnt**: Sieger-Text im Mehrspieler war fast unlesbar;
  irreführender „S = Setup"-Hinweis entfernt.
- **Aim Trainer / 3D-Labyrinth / Tunnel Racer**: Auflösungswechsel baute nur
  eine von vier Schriften neu.

---

<a name="-english"></a>

## 🇬🇧 English

### Match history & daily archive – 2026-11-26

Every one of the 50 games now keeps a **match history** with "Last matches"
and "All matches", and the three daily modes get a **daily archive**: the daily
track, daily puzzle and daily word of the last 30 days – and every day you have
already played – can be replayed with their own best score. Everything in 14
languages, with a new LamaWiki page, in both READMEs, in the web version, in
the Rust build and on pygamez.com.

#### New

**Match history** (all 50 games)
- New **Matches** button in every game's pre-game screen.
- **Last matches**: a curve of the last 30 results (records in gold, tooltip on
  hover) and below it the last 10 matches.
- **All matches**: every saved match with a mode filter (M), newest/best
  sorting (O) and scrolling; above it the totals – matches, wins with rate,
  best, average and play time.
- Per match: date and time, mode, points (chip games: profit or loss),
  win/loss/draw, duration and details such as bet, payout, multiplier, coins,
  character, level, guesses or opponent. Records are marked in gold; leaving
  early creates an "aborted" entry (only with points or after 5 seconds).
- Chip games count every round separately (hand, spin, ball, crossing),
  Geometry Dash every level visit, Labyrinth every solved level.
- The last **250** matches per game are saved in the new file
  **`matches.pgzlamamatches`** (compact, separate per profile); the totals keep
  counting. On pygamez.com it is 50 per game by default.

**Daily archive** (Crossy Road, Sudoku, Wordle)
- Replayable: the **last 30 days** and every day already played – with the same
  seed as back then, i.e. exactly the same track, puzzle or word.
- New date bar in the setup (◀ Today · 26.11. ▶ Archive): the arrows or **Page
  up/down** change the day, **K** or "Archive" opens the calendar with best
  score, best time or status per day; older played days are listed below.
- Separate best score per day (Crossy Road, with tries), best time and stars
  (Sudoku) or progress (Wordle, also unfinished).
- Archive days don't change the daily streak, Wordle statistics or streak
  achievements; "Today" stays the normal daily mode.
- Stored compactly: one day number instead of a date per entry – a year of
  Crossy archive is about 4 KB, a year of Wordle about 12 KB.

#### Changed
- Pre-game screen: Options, Matches, Wiki and Back sit side by side in the
  compact row; labels that are too long get a short form.

#### Fixed
- Desktop: the pre-game screen appears in the chosen game's accent colour
  again (instead of the theme colour) – like in the browser.

---

### Lama Crash – 2026-11-21

Game no. 50 is a crash game with llama chips: **Lama Crash**. A llama rises in
a hot-air balloon while the multiplier grows every second – cash out in time
and you get bet × multiplier, wait too long and the bet is gone the moment the
balloon pops. The trip leads through **six altitude worlds** from the llama
meadow to the **moon landing at ×100,000**, accompanied by suspense music that
speeds up with every world. On top of that come **two bets** with their own
auto target, **halving**, an auto mode, statistics and the same **fairness
check** as in Lama Mines and Lama Plinko – every pop point can be
recalculated, and every cash-out target returns 96.997 to 97.000%. Plus **6 new
achievements** – **274** in total. Everything in 14 languages, with a LamaWiki
page, in both READMEs, in the web version, in the Rust build and on
pygamez.com.

#### New

**Lama Crash** (game no. 50, in the menu after Lama Plinko)
- Pick a bet (10, 20, 50 … up to 10000 llama chips) and press **PLAY**: the bet
  is taken, and after a short 3-2-1 the balloon lifts off. From ×1.01 on you
  may **CASH OUT** at any time – the llama jumps off with a parachute and the
  win is credited at once. If the balloon pops first, the bet still in the air
  is lost.
- **Climb curve**: ×2 after 11.6 s, ×10 after 38.4 s, ×100 after 76.8 s,
  ×100,000 after 192 s – integer-only (`mu += mu // 1000` per tick of 1/60 s)
  and bit-identical in Python, JavaScript and Rust
  (`games/lamacrash_logic.py`). **Turbo** (T) climbs three times as fast.
- **Empty flight**: after cashing out the empty balloon keeps rising until it
  pops ("All out – the balloon keeps rising") – so you see how far it would
  have gone. Space or FAST FORWARD runs it 16 times as fast.
- **Pop point and return**: every round draws a 32-bit random number r, and
  the pop point is `⌊97 · 2³² / (2³² − r)⌋` in hundredths (×1.00 to
  ×100,000). So the chance of reaching ×x is 97% / x – every cash-out target
  returns 96.997 to 97.000%, and about 3.96% of rounds pop right at ×1.00. The
  balloon only pops once the multiplier exceeds the pop point: cashing out by
  hand at ×m wins under exactly the same condition as an auto target of ×m.
  Reach ×100,000 and you land on the moon – the max win for every bet still in
  the air.
- **Two bets** (Z or the switch on card 2): a second llama in the basket, each
  with its own amount and its own **auto target** (×1.01 to ×100,000, ←/→, X
  on/off). An auto target pays exactly the target – even if the balloon pops
  in the same tick. Each bet can be **halved** once (H for all, Q/W for one):
  that half floats away as a money sack on a mini parachute while the rest
  keeps climbing.
- **Six altitude worlds**: llama meadow → sea of clouds (from ×1.5) → storm
  front (from ×3, rain, lightning, planes) → northern lights (from ×10) →
  outer space (from ×100, the curve of the Earth, satellites, shooting stars,
  an ever bigger moon) → **moon landing** with a flag, confetti and a coin
  shower. Each new world is announced as you reach it; the worlds blend
  smoothly into one another, the balloon inflates more and more with altitude,
  and the multiplier curve runs on top with growing axes
  (`games/lamacrash_world.py`, `games/lamacrash_draw.py` – all drawn from
  pygame primitives, no image files).
- **Suspense music** (`games/lamacrash_music.py`): one piece in six stages,
  one per altitude world – from the meadow up to outer space a whole tone
  higher and faster with every world (96 to 150 bpm), a fanfare on the moon;
  M switches it on or off. From ×2 on the stretched balloon squeaks.
- **Nothing gives the pop point away**: picture, music and sounds depend only
  on the current multiplier.
- **Pop**: a bang, rubber shreds and camera shake; llamas still on board
  tumble out of the picture together with the basket. The celebrating happens
  when you cash out: a coin shower from ×3, confetti from ×10, a win banner
  from ×100. The top right shows the pop points of the last 12 rounds as
  coloured pills.
- **Auto mode** (A): 5, 10, 25, 50, 100 or 250 rounds, stop at a profit of or
  at a loss of (off, 100, 500, 1000, 5000, 10000 or 50000 chips) and turbo.
  Every active bet gets an auto target, an empty balloon is fast-forwarded
  automatically, and you can still cash out by hand. Stopping auto lets a
  round in the air fly on normally.
- **Fairness check** (F) as in Lama Mines and Lama Plinko: hash beforehand,
  seed after the round; "Verify" checks the hash, recalculates the pop point,
  checks the cash-outs and shows ✓ or ✗ with a mini curve.
- **Statistics** (S): the chip games' tiles plus top pop point, half cash-outs
  and close calls, the net-result curve of the last 100 rounds and the
  **distribution of pop points** in seven buckets as bars against the
  expected distribution.
- **Llama Bank**: the same account as Blackjack, Poker, Casino, Lama Crossing,
  Lama Mines and Lama Plinko. PLAY takes the total bet and saves the round in
  one go (`lamabank.bet_and_pay`), every cash-out credits the win right away,
  and a carry-over of the hundredths goes to the next payout. High score =
  peak of 1000 + Lama Crash balance; below 10 chips you get a bank loan back
  up to 1000. Like all chip games, it has no replays.
- **Resume after leaving**: leave mid-flight and the next start continues at
  the same tick after a countdown – the pop point is fixed anyway. Leave during
  the empty flight and the round is settled right away.
- **Pause on focus loss**: if the window loses focus while a bet is in the
  air, the game pauses by itself – otherwise the balloon would keep climbing
  while you are elsewhere.
- Controls: before a round Space/Enter = PLAY, 1/2/Tab = pick a card,
  ↑/↓/+/-/mouse wheel = bet, ←/→ = auto target, X = target on/off, Z = second
  bet; in flight Space/Enter/C = everyone out, 1/2 = one bet, H = half, Q/W =
  half of one bet; afterwards Space = fast-forward; A/S/F/T/M = auto,
  statistics, fairness, turbo, music. The keys are fixed and don't depend on
  the control bindings. Bets, targets, turbo, music and the auto settings are
  stored in the `lamacrash` section of `settings.json` (with validation rules
  in `settings.py`).
- Its own sidebar pictogram – a balloon above a rising curve – and its own red
  accent colour; 72 new texts in all 14 languages.

**6 new achievements** (now 274)
- **Lama Crash regular** (25 rounds), a **3000** milestone and a **10000**
  master tier.
- **Close Call** (rare): cash out by hand at ×2 or more – at most 5% before
  the pop. Only counts by hand, not with auto targets.
- **Aurora Pilot** (epic): cash out at ×10 or more.
- **Space Llama** (legendary): cash out at ×1000 or more.
- That makes 60 general achievements, 41 milestones, 34 master tiers, 89
  special moments and 50 regulars; at most **11010 achievement points** (was
  10680), which is level **51** (was 50).

**Web**
- Lama Crash also runs in the web version – with the same curve, pop points,
  worlds, music and achievements; the pause on focus loss is there as well.

**Rust build**
- Lama Crash is in the Rust .exe as well – picture- and data-identical to
  Python. That makes **50/50 games**.

**Website**
- Lama Crash is on pygamez.com – with the Llama Bank saved in your account,
  also in the phone version; the game count reads 50 in all 14 languages.

**Tests**
- New: `tests/audit_lamacrash.py` – Llama Bank bookings, cashing out, empty
  flight and fast-forward, two bets and halves, auto targets paying exactly
  the target and the moon landing, achievements, resuming after leaving,
  **twin rounds** (pop points ×5 and ×50 look and sound identical up to ×4.99,
  frame by frame – nothing gives the pop point away), fairness verification,
  ticks independent of the frame rate (30/60/144 Hz), turbo, pause and
  `pause_on_blur`, auto mode, keys, mouse, going broke and the loan, and the
  layout in 5 resolutions × 14 languages (UI v4.2 and v1).
- `tests/audit_chiplogic.py` now also checks the crash rules (18)–(22): the
  pop-point formula in full against its 2¹⁶ counterpart, the exact return per
  target, the climb curve and its inverse, the tick rule against a reference
  (400 random rounds), Monte Carlo with real seeds, and counters, texts, axes
  and worlds. For this `devtools/export_chiplogic_ref.py` writes a new `crash`
  section with the reference values for the Rust unit tests.

#### Changed

- **Engine**: new hook `pause_on_blur` in `game_base.py` – a game can ask (also
  as a property) to be paused when the window loses focus; `main.py` checks it
  right after releasing the keys. Lama Crash uses it while a bet is in the
  air. The same exists in the web version (`web/js/app.js`) and in the Rust
  build.
- **Llama Bank**: `lamabank.py` knows Lama Crash (`GAMES`, minimum bet 10 in
  `MIN_BET`).

**Documentation**
- LamaWiki: new **Lama Crash** page in all 14 languages; *Achievements, rarity
  & level* (274 achievements, 50 games), *Saving & high scores* and the Llama
  Bank paragraphs of Blackjack, Poker, Casino, Lama Crossing, Lama Mines and
  Lama Plinko updated.
- Both READMEs: 50 games, a new table row, a feature section, the project
  structure and the achievement counts.

---

### Lama Mines & Lama Plinko – 2026-11-21

Two new chip games at once: **Lama Mines** (game no. 48) and **Lama Plinko**
(game no. 49). In Lama Mines you uncover gems on a 4x4 to 7x7 field and dodge
the mines – every gem raises the multiplier, up to **×100,000**. In Lama Plinko
golden balls of yarn fall through 8 to 16 rows of pegs into slots worth up to
**×1000**, up to 50 at once. Both come with an **auto mode**, a **statistics
window** and a **fairness check**: before every round a SHA-256 hash is fixed,
afterwards the seed is revealed – every result can be recalculated. On top of
that come **12 new achievements** – **268** in total. Everything in 14
languages, with LamaWiki pages, in both READMEs, in the web version, in the Rust
build and on pygamez.com.

#### New

**Lama Mines** (game no. 48, in the menu after Lama Crossing)
- Pick a bet (10, 20, 50 … up to 10000 llama chips) and press **PLAY**: the bet
  is taken and the covered field is ready. Every gem you uncover raises the
  multiplier, a mine loses the bet. From the first gem on you can **CASH OUT**
  at any time; find every gem ("ALL MINE!") or reach ×100,000 and you are paid
  out automatically.
- **4 field sizes** (keys 1–4): 4x4, 5x5 (default), 6x6 and 7x7, with **1 to
  N−1 mines** (←/→ or six quick-pick buttons, default 3).
- **About 97% return**: the multiplier (in hundredths) after k gems is
  `m_k = max(⌊97 · C(N, M) / C(N−k, M)⌋, m_(k−1) + 1)`, capped at ×100,000 –
  integer-only and bit-identical in Python, JavaScript and Rust
  (`games/lamamines_logic.py`). So it never drops below ×1.01 and rises with
  every gem. Every "cash out after k gems" strategy returns 96.2 to 97%; only
  three steps with 1 mine on 6x6 and 7x7 are lifted by the floor (at most
  98.9%), and playing all the way to the cap returns less. Example, 5x5 with 3
  mines: ×1.10, ×1.25, ×1.44 … up to ×2,231 for all 22 gems.
- **Carry-over**: the hundredths left over when a payout is rounded down are
  added to the next one – small bets lose nothing.
- **Display**: current and next multiplier, the chance of a safe tile in %, the
  win for the next tile and a bar with the next 5 steps. Gems come in four
  colours; the mascot llama with miner's helmet and headlamp fills its mine
  cart, cheers at every gem and is covered in soot after a mine.
- **Mine hit**: an explosion and camera shake, then the **whole field is
  revealed, dimmed** – after cashing out as well. A coin shower from ×3,
  confetti from ×10, on a cleared field and on the max win.
- **Auto mode** with marks: mark tiles on the grid in the auto window (click or
  arrows + Space, Delete/Backspace clears; remembered per field size); every
  round uncovers exactly those and cashes out if all of them were safe; 5, 10,
  25, 50 or 100 rounds. Stopping auto leaves a round in progress open, so you
  can finish it by hand.
- **Resume after leaving**: the round is saved with every tile you uncover; on
  the next start the field is recalculated from the seed and the round
  continues exactly there.
- Controls: before a round Space/Enter = PLAY, 1–4 = field size, ←/→ = mines,
  ↑/↓/+/-/mouse wheel = bet; during a round click or arrows + Space = uncover a
  tile, R = random tile, Enter/C = cash out; turbo (T) = all timings ×0.4.
  Field size, mines, bet, turbo and the auto settings are stored in the
  `lamamines` section of `settings.json`.

**Lama Plinko** (game no. 49, in the menu after Lama Mines)
- Pick a bet per ball (1, 2, 5 … up to 1000 llama chips) and press **DROP**: a
  golden ball of yarn bounces down the board peg by peg and lands in a slot with
  a multiplier. The llama at the top of the board nods with every drop.
- **8 to 16 rows** (←/→, default 12) and **3 risk levels** (1–3: Low, Medium,
  High); both are locked while balls are in the air or auto is running. The
  outermost slot pays ×16, ×110 or ×1000 with 16 rows and ×5.3, ×13 or ×32 with
  8 rows; the middle pays ×0.7, ×0.4 or ×0.2.
- **27 fixed tables** in `games/lamaplinko_logic.py`, generated once with
  `devtools/gen_plinko_tables.py`: symmetric, never falling towards the edges,
  "nice" numbers and a return calculated exactly (with fractions) between 96.9
  and 97.05%. A carry-over of the hundredths keeps even a bet of 1 at about 97%.
- **Result = n coin flips**: every row is a left/right flip from the fairness
  randomness, and the slot is the number of right flips. The animation follows
  exactly that path – with arcing bounces, a flashing peg and a little cosmetic
  jitter; a ball takes 1.62 s (8 rows) to 2.30 s (16 rows). The slot bobs on a
  hit, a popup shows the multiplier and the sound depends on the value; a coin
  shower from ×10, a win banner with confetti from ×100.
- **Up to 50 balls at once**: hold Space = rapid fire at a rate of flight
  time/30 – that keeps about 30 balls in the air; turbo halves flight time and
  rate. The bet can be changed mid-flight. Hovering over a slot shows its
  multiplier, probability and the win at the current bet.
- **Booked at the drop**: bet and win with a single save
  (`lamabank.bet_and_pay`), shown only when the ball lands. Leaving with balls
  still in the air costs nothing – their achievements are caught up as well.
- **Auto mode**: 10, 25, 50, 100, 250, 500 or 1000 balls, one every 0.24 s
  (turbo 0.12 s), with a fixed bet.
- Controls: mouse for everything, Space/Enter/click on DROP or the board = drop
  (hold = rapid fire), ↑/↓/+/-/mouse wheel = bet, ←/→ = rows, 1–3 = risk. Rows,
  risk, bet, turbo and the auto settings are stored in the `lamaplinko` section
  of `settings.json`.

**In both games**
- **Auto mode** (A): number of rounds or balls, stop at a profit of or at a loss
  of (off, 100, 500, 1000, 5000, 10000 or 50000 chips) and turbo (T). Auto runs
  until the count is reached, a limit kicks in, the chips run out or STOP/Esc
  is pressed.
- **Fairness check** (F): before every round – in Plinko before every ball – the
  hash is fixed, the SHA-256 of a secret server seed of 32 hex characters. The
  randomness comes from the server seed, your own client seed (8 characters,
  new at the press of a button – only outside a round or while no ball is in
  the air) and the round number (`seedrand`, mulberry32). After the round the
  seed is revealed; by then the next round's hash is already fixed. The window
  shows the last 12 entries, and "Verify" recomputes the hash and the result
  and shows ✓ or ✗ with a mini preview. New for this: `fairness.py`.
- **Statistics** (S): tiles for rounds, won, lost, wagered, paid out, net
  result, biggest win, best multiplier and your own return, plus a net-result
  curve of the last 100 rounds. Lama Mines also counts gems found, mines hit and
  fields cleared; Lama Plinko shows the hit distribution per row count (←/→
  switches) against the expected binomial curve. New for this: `chipstats.py`.
- The top right shows the **history of the last 12 rounds** or balls.
- **Llama Bank**: the same account as Blackjack, Poker, Casino and Lama Crossing
  (section `casino` in `mem.json`); the round in progress, history, fairness
  values, statistics, carry-over and auto marks are kept there per profile as
  well. High score = peak of 1000 + the game's own balance; going broke gives
  you a bank loan back up to 1000 (in Plinko once every ball has landed). Like
  all chip games, they have no replays.
- The keys are fixed (arrows, Space, Enter, digits and letters) and don't depend
  on the control bindings in the options; Esc closes an open window or stops
  auto.

**12 new achievements** (now 268)
- **Lama Mines regular** and **Lama Plinko regular** (25 rounds each), plus a
  **3000** milestone and a **10000** master tier for each.
- **Treasure Vault** (rare): clear a whole field in Lama Mines.
- **Deep Digger** (epic): cash out at ×10 or more in Lama Mines.
- **Diamond Llama** (legendary): cash out at ×1000 or more in Lama Mines.
- **Edge Hit** (rare): hit the outermost slot in Lama Plinko (12 rows or more).
- **Yarn Storm** (epic): have 25 balls on the board at once.
- **Plinko Legend** (legendary): hit the outermost slot with 16 rows on High
  risk.
- That makes 60 general achievements, 40 milestones, 33 master tiers, 86 special
  moments and 49 regulars; at most **10680 achievement points** (was 10020),
  which is level **50** (was 49).

**Web**
- Both games also run in the web version – with the same formulas, tables,
  fairness values and achievements. SHA-256 is written from scratch there (test
  vector "abc" → `ba7816bf…`), and new seeds come from
  `crypto.getRandomValues`.

**Rust build**
- Lama Mines and Lama Plinko are in the Rust .exe as well – pixel- and
  data-identical to Python, checked with comparison runs in
  `rust/tools/regression.py`. SHA-256 is written from scratch there too, because
  the build runs offline without extra crates. That makes **49/49 games**.

**Website**
- Both games are on pygamez.com – with the Llama Bank saved in your account and
  on-screen keys on phones; the game count reads 49 in all 14 languages.

**Tests**
- New: `tests/audit_lamamines.py` and `tests/audit_lamaplinko.py` – the return
  exactly (with fractions) and by Monte Carlo, the carry-over, Llama Bank
  bookings, resuming, going broke and the loan, fairness verification, auto
  stops, statistics limits, rapid fire, keys, mouse and the layout in 5
  resolutions × 14 languages (UI v4.2 and v1).
- New: `tests/audit_chiplogic.py` for the shared rules – SHA-256 test vectors,
  seeds, fairness log, statistics, the Mines formula and the Plinko tables (also
  against `devtools/gen_plinko_tables.py`); a Python = JS cross-check via Node,
  and for Rust `devtools/export_chiplogic_ref.py` writes the reference values
  that the Rust unit tests recompute.

#### Changed

- **Llama Bank**: the new function `bet_and_pay(bet, win, game)` books bet and
  win in one go with **one** save, optionally together with the game state –
  for games whose result is already settled when the bet is placed (Plinko) or
  that store the bet together with the new round (Mines). It is in
  `web/js/games/casino_bank.js` and `rust/crates/pgz-core/src/lamabank.rs` as
  well.

**Documentation**
- LamaWiki: new **Lama Mines** and **Lama Plinko** pages in all 14 languages;
  *Achievements, rarity & level* (268 achievements, 49 games) and *Saving & high
  scores* (Llama Bank) updated.
- Both READMEs: 49 games, two new table rows, two feature sections, the project
  structure and the achievement counts.

---

### Lama Crossing – 2026-11-15

Game no. 47 is a new push-your-luck game with llama chips: **Lama Crossing**.
The llama crosses a road, a rope bridge, a river or a frozen lake lane by lane –
every lane you make raises the multiplier, and you can cash out after any lane.
**Version 2.0** followed the same day: a **road with cars and trucks** as the
new default world, obstacles in every world and longer, harder tables up to
**×31,784.96**. Losing now really feels like losing – and no longer as if it had
been settled from the start. On top of that come **6 new achievements** –
**256** in total. Everything in 14 languages, with a LamaWiki page, in both
READMEs and in the web version.

#### New

**Lama Crossing** (game no. 47, in the menu after Casino)
- Pick a bet (10, 20, 50 … up to 10000 llama chips) and press **GO**: the bet is
  taken and the llama steps into the first lane. After every lane you make it's
  **NEXT** or **CASH OUT**; reaching the other side cashes out automatically –
  with confetti.
- **4 worlds** (buttons top left or **W**): the **road** as the default – a
  street seen from above with sidewalks, where the llama crosses lane by lane
  over zebra-crossing pieces – plus a rope bridge over a canyon, stepping stones
  in a river and ice floes on a frozen lake. The llama wears a scarf in the
  game's accent colour.
- **Obstacles you have to time**: cars and trucks drive down every lane of the
  road, boulders crash through the rope bridge, driftwood floats down the river
  and ice blocks slide across the ice lake. Press GO or NEXT while an obstacle
  is in the next lane and the llama gets run over or hit and lies flat – so wait
  for a gap. The lane the llama stands in is protected: on the road a barrier
  drops and the cars stop in front of the llama, in the other worlds the
  obstacles there vanish. Higher difficulties bring denser, faster traffic.
- **Random risk per lane**: even with perfect timing it can go wrong – on the
  road a speeder suddenly appears, on the bridge the plank breaks, on the river
  the stone slips away, on the ice lake the ice breaks.
- **4 difficulties**: Easy (22 lanes, risk 1/12, up to ×6.57), Medium (20, 1/6,
  up to ×37.18), Hard (18, 3/10, up to ×595.67) and Extreme (15, 1/2, up to
  ×31,784.96 – every lane doubles: ×1.94, ×3.88, ×7.76 …).
- **About 97% return**: the multipliers are fixed integer tables in
  `games/lamacrossing_logic.py` (`m_k = floor(0.97 / (1 - p)^k * 100) / 100`),
  so with clean timing every "cash out at lane k" strategy pays the same on
  average – the house edge is about 3%.
- The top right shows the **history of the last 12 rounds**.
- **Resume after leaving**: bet, difficulty and lane of a crossing in progress
  are saved immediately; the next start continues exactly there.
- **Llama Bank**: the same account as Blackjack, Poker and Casino (section
  `casino` in `mem.json`). High score = peak of 1000 + Lama Crossing balance;
  below 10 chips you get a bank loan back up to 1000. Like all chip games, it
  has no replays.
- Controls: Space/→/click on the next lane = GO or next lane (when the lane is
  clear), Enter/C = cash out, ↑/↓/+/-/mouse wheel = bet, 1–4 = difficulty, W =
  world. World, difficulty and last bet are stored in the `lamacrossing` section
  of `settings.json`.

**6 new achievements** (now 256)
- **Lama Crossing regular** (25 rounds), a **3000** milestone and a **10000**
  master tier.
- **Other Side** (rare): get the llama all the way across.
- **Nerves of Steel** (epic): cash out at ×10 or more.
- **Tightrope Walk** (legendary): cash out at ×100 or more on Extreme.
- That makes 60 general achievements, 38 milestones, 31 master tiers, 80 special
  moments and 47 regulars; at most **10020 achievement points** (was 9690),
  which is level **49** (was 48).

**Web**
- Lama Crossing also runs in the web version – with the same tables, worlds and
  achievements.

**Tests**
- New: `tests/audit_lamacrossing.py` – tables and return per strategy, a Monte
  Carlo check of the missteps, web logic = Python, the Llama Bank (debit,
  payout, going broke, loan), resuming after leaving, achievements, keys and
  settings, plus the layout in 5 resolutions × 14 languages × 4 worlds.

#### Changed

**Lama Crossing 2.0** (the same day)
- **The road is the new default world**: a street full of cars and trucks that
  the llama crosses lane by lane over zebra-crossing pieces.
- **Obstacles in every world**: besides the traffic on the road there are
  boulders on the rope bridge, driftwood on the river and ice blocks on the ice
  lake. Timing matters – go too early and you lose.
- **Losing feels like losing**: when you get caught, you see it coming – the
  llama is run over or hit and lies flat.
- **Nothing feels predetermined any more**: you watch the obstacles coming and
  decide yourself when to go. The note after cashing out that told you at which
  step it would have gone wrong is gone – it made losses look as if they had
  been settled from the start.
- **Harder, longer tables**: 22, 20, 18 and 15 lanes with risk 1/12, 1/6, 3/10
  and 1/2; on Extreme every lane doubles the multiplier – up to ×31,784.96.
- **Tightrope Walk** now asks for a cash-out at ×100 or more on Extreme instead
  of a full crossing.
- New texts in all 14 languages: the road world, "Run over!", hit messages per
  world and the hint "go when the lane is clear".

**Documentation**
- LamaWiki: new **Lama Crossing** page in all 14 languages; *Achievements,
  rarity & level* (256 achievements, 47 games) and *Saving & high scores* (Llama
  Bank) updated.
- Both READMEs: 47 games, a new table row, a feature section, the project
  structure and the achievement counts.

---

### PyGameZ in Rust: a standalone 3 MB .exe – 2026-09-23

PyGameZ now exists a third time: as a **Rust program** in `rust/`. No Python
alongside it, no pygame, no runtime — a single file of **3.1 MB** instead of the
25.7 MB PyInstaller .exe. The save files are the same ones: play in Python, then
start the Rust build, and your high scores, achievements and profiles are all
still there.

#### New

**The groundwork is complete**
- **10 UI themes** (v4.2 "Midnight Glass" back to v1) with the same palettes and
  effect switches as in Python — stars, colour clouds, glass panels, zigzag
  patterns, vignette, title glow.
- **14 languages**, baked into the .exe. The only files next to it are the ones
  that actually get written.
- **Player profiles** with level, 14 ranks, icon and colour; **250 achievements**
  in five categories with rarity tiers and points; **statistics** covering games
  played, play time, wins, records, day streak and calendar.
- **17 sound effects**, synthesised at startup from frequency, duration and
  waveform — exactly the values from `audio.py`, no audio files.
- **Word filter** in all 14 languages for profile names.
- Screens: language picker, menu, pre-game with player 2 selection, gameplay
  with pause and high-score banner, options with three tabs, achievements &
  statistics, profile management.

**Ten games ported**
- **Snake** (all five modes, boost, golden apples), **Pong**, **Breakout**,
  **Tetris**, **2048**, **Minesweeper**, **Tic-Tac-Toe**, **Connect Four**,
  **Flappy** and **Memory** — carrying over the game logic, the constants and
  the AI levels of their Python originals.
- The remaining 36 games will follow; the sidebar shows the state as "10 / 46".

**Tools**
- `rust/tools/export_defs.py` reads `games/__init__.py`, `ui.GAME_COLORS` and
  `achievements.py` and generates the Rust master data from them: 46 games,
  250 achievements, 10 themes. Nothing is typed twice, nothing drifts apart.
- `rust/tools/audit.py` checks what no single test can see: whether every
  translation key in use exists in all 14 languages, whether placeholders
  actually get filled, whether every game is known to the master data, and
  whether the generated file still matches the Python sources.
- **A test bench inside the program**: `PYGAMEZ_SHOT=… PYGAMEZ_GAME=all` starts
  every ported game in turn and saves a screenshot of each — the fastest way to
  check everything after an engine change. Plus switches for screen, theme,
  language and keys to press beforehand.
- `rust/ENGINE.md` is the porting contract for the games still to come, the
  counterpart to `web/ENGINE.md`.

#### Changed

- **One window instead of two.** The Python version embeds pygame into a Tkinter
  window via `SDL_WINDOWID` — which is why input goes through Tkinter there. In
  Rust the sidebar is simply drawn along with everything else.
- **No intermediate buffer.** The logical play area reaches the window through a
  camera with a viewport rather than a texture: one copy per frame less, and the
  image stays sharp.
- **Key names stay the same.** `"Up"`, `"space"`, `"Return"`, `"w"` — the same
  Tkinter names as in Python, so an existing `settings.json` fits unchanged.
- **Own key repeat** (450 ms, then every 45 ms) instead of the operating
  system's — identical on every machine.
- Build output goes to **`rust.output/`**, not into the source folder.

#### Fixed

- The word filter nearly rejected harmless names in Rust: without word
  boundaries, "Pi**ck a ga**me" contains a Slavic swear word. It now checks word
  by word and only collapses the whole text when it is written spaced out
  ("b-a-s-t-a-r-d") — the same rule the `\b` in the Python patterns provide.
- The explanatory header of the filter lists (`lang/swear/*.yml`) consists of
  `#` lines itself and was almost read as a word list; only comments directly
  below a pattern now count.

#### Known gaps

Compared to the Python version these are missing: replays, LamaWiki, custom
courses, the Llama Bank, prestige, competitive mode, personalisation, gamepad
rumble, music loops, pointer capture and the statistics charts. The statistics
themselves are there — just without drawn curves.

### Geometry Dash: 16 levels, a library of 80 and templates – 2026-09-22

Geometry Dash gets twice as many campaign levels, a searchable **library of 80
more levels** in ten categories and **12 templates** for your own levels. Every
new level – all 80 of them included – was played through by the solver with the
real game code, all three coins included and with a one-step shift in both
directions. Everything in 14 languages, with reworked wiki pages, both READMEs,
an extended audit – and the web version comes along in full.

#### New

**Campaign: 8 → 16 levels**
- Eight hand-built levels fill the curve from *Easy* to *Demon*: **Pixel
  Sprint**, **Moss Temple**, **Magnet Fields**, **Hangar Run**, **Strobe
  City**, **Frost Relay**, **Turbine Core** and the second demon **Lama
  Eclipse**.
- That makes **135 stars** to collect (87 level stars + 48 coins) instead of 65.
- The dot row below the level card is a **window** now: ten dots at most, the
  row follows your selection and "Level n / m" sits underneath. **[1]**–**[9]**
  jump to levels 1–9, **[0]** to level 10, **[Home]**/**[End]** to the first or
  last one.

**"More" tab: 80 levels in 10 categories**
- A new third tab next to PLAY and LEVELS: **starter, rhythm, coins, ship, UFO,
  ball, gravity, speed, wave, demon** – eight levels each.
- A **search field** over name, id and category, a category bar, a random button
  and a line "x / 80 levels · n cleared". Every row shows difficulty, length,
  the coins you found and your own best run.
- Its own progress per level (key `x:<id>` in the `geodash` section): best runs,
  coins and attempts are kept, **stars and achievements stay with the
  campaign** – that keeps the star scale meaningful.
- **[Tab]** cycles all three tabs; the last tab and category are remembered.

**Templates for your own levels**
- "New" now opens a **template picker** with 12 tiles: *empty*, starter, jumps,
  pads & orbs, ship, wave, ball, UFO, gravity, speed, coins, demon.
- A template brings its objects, colors, music and start speed along – and since
  the solver played each one through, the copy carries the **"verified"** check
  mark right away, until you change something.
- **"As template"** in the *More* tab copies any finished level into your own
  levels, where you can rebuild, test, save and play it.

**Builders, proofs and tests**
- New: `devtools/geodash_sections.py` (20 building blocks measured against the
  real physics) and `devtools/build_geodash_extra.py` – deterministic, with
  proofs in `devtools/geodash_extra_proofs.json`.
- Coins sit **on the proven path**: the level is solved without coins first,
  then three coins are placed on the replayed trajectory and the proof is
  verified again.
- `tests/audit_geodash.py` additionally checks the library (desktop JSON == web
  index == web full data), every proof at a −1/0/+1 step shift, all categories,
  the templates including their badge, filtering and search, "as template", plus
  the category bar, the search field and the template picker across 5
  resolutions × 14 languages.

**Web**
- The browser gets the same three tabs, the same library and the same templates.
  The **full data of the 80 levels is loaded on demand** when the tab is opened
  (`geodash_extra.js`); the small index ships with the game.

#### Changed

- `settings.json`: `geodash.last_level` now goes up to 199, plus the new values
  `last_tab` and `last_cat`.
- The wiki pages **Geometry Dash** and **Your own levels (Geometry Dash)** were
  reworked in all 14 languages (16 levels, library, templates).

---

### Achievements 2.0, statistics charts & player profiles – 2026-09-21

The biggest update yet for everything that counts: **250 achievements** (up from
108) with **rarity tiers** and **achievement points**, a **level system** with 14
ranks on top, a statistics screen with **five tabs and real charts** – and
**player profiles** so that several people can collect separately on one device.
All statistics now live in their own file `stats.pgzlamastats`, which can be
handed around. Everything in 14 languages, with three LamaWiki pages, both
READMEs, its own audit – and the web version comes along in full.

#### New

**Achievements, rarity & level**
- **250 achievements** in five categories: 60 general goals, 37 score
  milestones, 30 **master tiers** (the second, much higher mark per game), a
  **regular** achievement for each of the 46 games (25 rounds) and 77 special
  moments – 30 of them new: giant snake (length 50), a shutout in Pong, an air
  hockey hat-trick, the Invaders boss, a Flappy gold medal, the T-Rex night run,
  a triple jump in draughts, a double mill in Nine Men's Morris, a bank shot in
  the tank duel, cracking Mastermind in three guesses, four ghosts on one power
  pill, Minesweeper without flags, two Tetrises in a row and more.
- Five **rarity tiers** with points: common 10, rare 25, epic 60, legendary 150,
  mythic 300. The colour shows in the badge, in the stripe of the row and in the
  pop-up; legendary and mythic ones get confetti.
- **Level system**: points make the level (level 2 costs 20 points, each further
  level 8 more than the one before), plus **14 ranks** from Llama Chick to Star
  Llama. A level-up gets its own pop-up with a fanfare; level, rank and XP bar
  appear in the sidebar and in the progress screen.
- On the first start after the update old high scores and statistics count right
  away – a one-off card shows **your starting level** and your rarest
  achievements instead of a parade of pop-ups.

**Statistics with charts**
- The progress screen now has **five tabs**: Achievements (with filter, points
  and level header), Statistics (cards, top-10 playtime bars, win/loss ring,
  table), **Calendar**, **History** and **Profiles**.
- **Calendar**: days played as a heatmap (the darker, the longer), current and
  longest daily streak, weekday bars and a time-of-day ring.
- **History**: everything with a timestamp – achievements (with rarity and
  points), records, level-ups and duels – plus each game's **score curve** as a
  line with golden record markers.
- **Recap**: best day, favourite time, longest session in one go, favourite game
  and your rarest achievement.
- All charts have tooltips; the new drawing blocks live in `charts.py` (bars,
  lines, calendar, rings, clock, tooltip).

**Player profiles**
- Up to **eight profiles** on one device, each with their own high scores,
  achievements, level, statistics, llama chips and saved games. Only resolution,
  sound, key bindings, theme and the language stay shared.
- New sidebar button **"Profiles"** plus a card at the top showing who is playing
  (avatar, name, level, XP bar). Create, rename, pick icon and colour, switch,
  delete – the name goes through the word filter.
- **Multiplayer**: before the match player 2 picks their profile (or plays as a
  guest); in game the profile names replace "Player 1/2".
- **Head-to-head record** and **device leaderboard** in the Profiles tab: who
  beat whom how often, and who holds the best score in each game.

**Statistics file `stats.pgzlamastats`**
- All statistics (of all profiles) now live in their own JSON file next to the
  game – with a header like replays and courses.
- **Export/import** on the calendar tab (or with **E** and **I**); the web
  version reads and writes the same file.
- To stop it growing forever, history (4000 entries), score curves (250 per
  game) and calendar (about three years) are capped – the totals stay.

**Web version**
- Comes along in full: achievements with rarity and level, all five tabs with
  the same charts, profiles (their own storage prefix each) and the exchange of
  the `.pgzlamastats` file.
- The definitions are generated by `devtools/export_achievements.py` straight
  from `achievements.py` – desktop and browser cannot drift apart; a test checks
  it.

#### Changed

- `mem.json` has a new structure: only `mem` (language) stays device-wide,
  everything else lives under `players/<profile id>`. Your existing save moves
  into profile 1 on the first start.
- The old `stats` section of `mem.json` moves into the new statistics file
  (including achievement timestamps as history).
- The Snake cosmetics from `mem-ngb.json` now belong to the profile.
- Replays and self-made courses stay device-wide but carry the profile name; the
  course editor suggests it as the author.
- New LamaWiki pages **Player profiles** and **Statistics & charts**, the
  **Achievements** page has been rewritten – in all 14 languages.

#### Fixed

- The llama bank still read its sections past the profiles – it now goes through
  the profile storage as well.
- After restarting straight from game over a second duel no longer counted; the
  lock is now released together with the win/loss lock.

---

### Replays for Billiards, Pinball, Snake & Tetris – and sharing as a file – 2026-09-18

Replays now cover **six games** instead of two: **Billiards**, **Pinball**,
**Snake** and **Tetris** record their rounds just like Minigolf and Bowling. New
on top of that is **sharing**: **E** writes a recording as a `.lamapgzreplay`
file, **I** reads it back in somewhere else. All of it in 14 languages, with a
reworked LamaWiki page, in both READMEs and with an audit of its own.

#### Added

**Billiards**
- One sequence per **shot**: the header holds aim, power, player, groups and the
  whole table; the samples only carry the balls that moved since the last frame.
  The run-up shows the aiming line and cue just like the real shot, the run-out
  the result (potted, no pot, foul, win).
- At the end of a frame **P** shows the replay. The **practice mode** never ends
  – there **P** shows what happened so far between two shots, and the frame
  carries on afterwards.
- Playback uses the recorded view (2D, 3D or free); the ball paths are stored in
  table coordinates and are independent of it.

**Pinball**
- One sequence per **ball**: ball paths (multiball included), flipper positions,
  drop targets, rollover lanes, multiplier, locks, ball save, tilt and score. A
  quiet frame costs six numbers – the table state only appears in samples where
  it actually changed.

**Snake**
- A run is cut into ten-second **chapters**: every chapter is a key frame (body,
  apples, score), the samples after it only carry the changes – one step is five
  numbers. All modes are recorded, including obstacles, portals, golden apples,
  competitive special apples and stamina; in multiplayer both snakes.
- Playback always shows the **top-down view** – also for runs played in the 3D
  view, whose camera depends on the frame rate.
- **P** opens it on game over; during a run P stays prestige.

**Tetris**
- Chapters as well: field (only changed rows), the active piece including lock
  delay, hold, 5-piece preview, score, level, combo, back-to-back and the
  garbage queue. In versus **both fields** are recorded.
- **P** opens the replay on the results screen.

**Sharing as a file**
- **E** writes the recording as a **`.lamapgzreplay`** file – from the archive as
  well as right after the round. The file is built like a minigolf course or a
  Geometry Dash level: an envelope with `format`, version and exactly one replay.
- **I** reads such a file back in. It lands in the tab of the matching game and
  carries a small arrow there. Duplicates, foreign files and a full archive are
  each rejected with their own message.
- Without file dialogs (e.g. without Tk) the export goes to the **downloads
  folder**; the path is named in the feedback.
- New achievement **Projectionist** (share a replay as a file) – **108** in total.

**Browser version**
- The web edition in `web/` gets the same replay system: `js/core/replay.js`
  (recording, archive, sharing) and `js/core/replayview.js` (archive list and
  playback), plus the **Replays** sidebar button and the *Record replays* switch
  in the settings.
- The same six games are recorded – including Minigolf and Bowling, which the
  browser did not record at all so far – in **the same file format**: a
  `.lamapgzreplay` file from the PC plays in the browser and vice versa. **E**
  downloads it, **I** reads it back in.
- The archive lives in localStorage: room for 5 recordings per game, a recording
  stops after 10 minutes, and a full browser storage says so instead of failing
  silently.
- New test rig `web/tools/replay_check.js` (headless Chrome): plays a real round
  of each game, runs the recording forwards, backwards and in random jumps,
  compares the final state with the original and checks sharing, importing and
  the replay screen – plus frozen recordings from the desktop version
  (`web/tools/desktop_replays.js`), so the file format of both editions stays
  the same.

#### Changed
- The **tab bar** of the replay screen wraps cleanly with six games; below it a
  header row shows the counter and the **Share** and **Import** buttons.
- **Run-up and run-out per game** (`replayview.PAD`): games with an aiming phase
  keep their run-up with the aiming line, Snake and Tetris run on without a
  pause. The sequence counter is now named stroke, roll, shot, ball or part
  depending on the game.
- The **recorder** no longer throws away an over-long recording; it stops
  writing and marks it as **Recording trimmed** (limit: 60,000 samples or
  420,000 numbers – a good half hour).
- New helper `replay.Delta` for samples that only carry changes; Billiards,
  Pinball, Snake and Tetris share it.
- The LamaWiki page **Replays** now describes all six games, the chapters and
  sharing as a file.
- New audit `tests/replay_audit.py`: plays a real round of each game, runs the
  recording forwards, backwards and in random jumps and compares field, body,
  balls and score with the original – plus export, import and every rejection
  reason.

### Arcade & Casino: 4 new games + 6 expansions – 2026-09-16

The biggest game update so far: **Crossy Road**, **Geometry Dash**,
**Battleship** and **Casino** bring the collection to **46 games**; **Tetris**,
**2048**, **Chess**, **Wordle** and **Sudoku** are greatly expanded, and
**Blackjack** and **Poker** now play with one shared llama-chip account. On top
of that comes a new technical foundation (keyboard input, crash-safe saving,
music loops, a random generator shared by PC and browser) and **22 new
achievements** – **107** in total. Everything in 14 languages, with LamaWiki
pages, in both READMEs and in the web version.

#### Added

**Crossy Road** (game no. 43)
- Hop endlessly across **grass, roads, rivers and railway tracks** – in an
  **isometric voxel look** with squash & stretch, splashes, a flattening
  animation and particles. Characters and ground strips are pre-rendered per
  tile size, so drawing stays at a few milliseconds per frame even at 1280x960.
- **Trains** with warning lights and bells (later whole stations with up to 5
  tracks), logs and lily pads, cars and trucks that speed up along the route;
  the **eagle** grabs dawdlers. The generator always guarantees a walkable path.
- **Day/night cycle** from row 50 with headlights, train lights and glow.
- **10 unlockable characters** (chicken, frog, pig, penguin, cat, fox, llama,
  robot, ghost, unicorn), bought with collected coins in the **Characters** tab.
- Modes **Endless** (high score) and **Daily Route** – the same for everyone,
  web version included – with its own daily best (section `crossy` in
  `mem.json`).
- New achievements **New Look** and **Track Jumper**, plus a 150-row milestone.

**Geometry Dash** (game no. 44)
- Rhythm platformer with **cube, ship, ball, UFO and wave**, form, gravity and
  speed portals, yellow/pink/blue pads and orbs, pits, color triggers and **3
  secret coins** per level.
- **8 built-in levels** from Easy to Demon ("Lama Inferno"). Each is proven
  beatable by a solver, including all coins.
- **Practice mode** with automatic and manual checkpoints (Z/X), attempt counter,
  progress bar, per-level bests, explosions and instant restart.
- Procedural **soundtrack per level** with a beat pulse in the visuals (can be
  turned off).
- **Precise input**: the physics runs in fixed-point math at a fixed 240 Hz step,
  and every press lands on the exact 1/240 s step – at any frame rate.
- **Level editor** in the LEVELS tab: 6-group palette, undo/redo, test from the
  start or from here, level settings, **sharing as `.lamapgzlevel`**, import and
  a "verified" badge only after beating your own level.
- High score = **total stars** (max. 65); bests, coins, attempts and jumps in the
  `geodash` section of `mem.json`. New achievements **On the Beat**, **Coin
  Hunter** and **Demon Slayer**, plus a 20-star milestone.
- Browser version with identical physics and editor.

**Battleship** (game no. 45, called "Schiffe versenken" in German)
- **10x10 naval battle** against the AI – *Easy* (random), *Medium*
  (hunt/target) and *Hard* (probability map with parity) – or **two players**
  with a hidden handover screen.
- **Drag & drop fleet deployment**: rotate with R/right click, random placement,
  green/red preview with no-go zone; your last layout is suggested again.
- **Optional rules**, saved: ships may touch, salvo mode, fire again after a hit.
- Radar sweep, animated waves, shells flying in an arc, splashes, explosions
  with smoke, burning cells, a "SUNK!" reveal and accuracy stats at the end of
  each round.
- New achievements **Flawless Fleet** (win without losing a ship) and **Fleet
  Admiral** (beat the hard AI); in the web version against the AI.

**Casino** (game no. 46)
- **European roulette** with all classic bets (straight up to low/high – click a
  number, edge or corner), chip values 1 to 500, rebet/double/clear, an animated
  wheel with a ball spiralling in and the last 12 numbers.
- **Llama Slot**: 5 reels, 10 paylines, llama = wild, gold coins = 10 free spins
  with double wins, auto spin, turbo and a paytable; **96.1% return to player**,
  calculated exactly from the reel strips.
- **Llama Bank** (`lamabank.py`): Blackjack, Poker and Casino share one account
  of llama chips (section `casino` in `mem.json`). Each game keeps its own
  balance for its high score; going broke gives you a bank loan. Old chip counts
  carry over automatically.
- New achievements **Bullseye** (straight-up win at roulette) and **Llama
  Jackpot** (5 llamas on one line), plus a 5000 milestone.

**Tetris** – completely rebuilt
- Modern **Guideline rules**: the SRS rotation system with wall kicks, rotation
  in both directions, **hold**, a **5-piece preview**, **lock delay** and custom
  **DAS/ARR** (adjustable in the setup).
- **Solo** with three variants: **Marathon** (high score, start level 1–15),
  **Sprint 40 lines** (best time) and **Ultra 2 minutes** (best score).
- Guideline scoring: **T-Spins** (full/mini), **Back-to-Back**, **combos** and
  **Perfect Clears** – with callouts, line-clear animation, particles and a
  level-up effect.
- **Versus against the AI** (3 strengths) and **2-player versus** with garbage
  lines, cancelling, a warning bar, identical piece sequence and a K.O.
  animation.
- 3 new achievements: **Spin to Win** (T-Spin Double), **Squeaky Clean** (Perfect
  Clear) and **Sprinter** (Sprint 40 under 2:00).
- Web version: everything except "2 Players", same rules and piece sequence as
  on PC.

**2048** – greatly expanded
- Its own **setup screen** with board sizes from **3x3 to 8x8** and the modes
  Classic, Time attack (3 minutes) and Endless.
- Smooth **animations**: tiles slide, merge with a pop and grow in; score
  pop-ups, sparks and new colours up to 131072.
- **Undo** (off / 3 per game / unlimited) with animation – using it takes the
  game out of the high score.
- **Swipe** with mouse/touchpad; held keys no longer repeat, and input during
  animations is buffered.
- Games are **saved** automatically per size and mode and can be resumed; best
  scores and biggest tile per size/mode (section `g2048`).
- "Keep playing?" after 2048 in Classic, a new game-over panel and the new
  achievement **4096!**.

**Chess** – greatly expanded
- New **Puzzles** mode: 200 puzzles from the free **Lichess puzzle database
  (CC0)** in 5 stages (mate in 1/2/3, tactics I/II) with progress tracking.
- **Chess960** (all 960 starting positions) and a **chess clock** (1+0, 3+2, 5+0,
  10+5) in the setup.
- A much **stronger, stutter-free AI**: iterative deepening, transposition
  table, opening book and a new evaluation; it thinks in small slices per frame.
- New **sidebar** with move list (SAN), clocks, captured pieces and material
  balance; undo, hint arrow, offer draw, resign, flip board, drag & drop,
  sliding pieces and coordinates.
- **PGN export** after the game (web: download and clipboard).
- New achievements **Puzzle Cracker** (25 puzzles) and **Grandmaster** (beat
  level 6 without assistance).

**Wordle** – greatly expanded
- New **Daily word**, **Dordle** (2 words, 7 guesses) and **Quordle** (4 words,
  9 guesses) modes alongside Endless.
- New **setup screen**: word length **4 to 7**, hard mode (now a setting instead
  of a separate mode) and a **colour-blind palette** (orange/blue).
- **Daily word** per language and length – identical on PC and in the browser –
  with countdown and streak; started daily words are saved.
- **Statistics** per language, length and mode: games, win rate, streaks and a
  guess-distribution bar chart (`wordle` section in `mem.json`).
- **Share your result** as an emoji grid via the clipboard; AZERTY keyboard for
  French, QWERTZ for German, Czech, Slovenian and Croatian.
- New **4-, 6- and 7-letter** word lists for all 14 languages (about 134,000
  answers across all lengths); cleaner answers without English leftovers, names
  or offensive words.
- New achievements **Word Habit** (7 daily words in a row) and **Quad Genius**
  (solve Quordle).

**Sudoku** – new variants
- New variants **X-Sudoku**, **Killer** (400 pre-generated, uniquely solvable
  levels) and **Mini 6x6** – 4 difficulties × 100 levels each.
- **Daily Sudoku**: one puzzle per day for everyone (identical on desktop and in
  the browser), difficulty by weekday and a daily streak.
- **Undo/redo** (U/Z, Y), **automatic candidates** (C), "digit first" input,
  **colour markers** (M) and a remaining-digit counter in all modes.
- **Puzzles in progress** are saved and resumed automatically.
- Up to **3 stars** and a best time per level, a redesigned setup screen with
  level info.
- New achievements **Cage Fighter** and **Star Collector** (30 levels with 3
  stars).

#### Changed

**Blackjack & Poker**
- Both play with the **llama chips** of the Llama Bank (start 1000) and share
  the account with the Casino; each high score is the peak of that game's own
  balance, and bank loans don't count.
- Bets are **taken immediately** – leaving a hand in progress loses the bet. In
  Poker, the table stack you haven't bet returns to your account on the next
  launch.
- The Blackjack milestone is now 1600 (starting account 1000 plus 600), and
  **Chip Leader** counts your poker balance.

**Foundation & tech**
- **Keyboard**: held keys are detected cleanly – games with their own key
  repeat (Tetris) or hold controls (Geometry Dash) respond precisely, and keys
  no longer get stuck after pausing, Alt-Tab or a screen change.
- **Crash-safe saving**: `mem.json` and `settings.json` are written atomically
  with an automatic backup (`.bak`); a damaged `mem.json` is read from the
  backup instead of losing high scores, achievements and chips on the next save.
- Switching to another game from the **sidebar** mid-game now keeps your high
  score.
- **Pre-game screen**: with many modes at small resolutions, Options, Wiki and
  Back move into one row and the font adapts – nothing runs off-screen any more.
- New shared **random generator** (`seedrand.py` / `seedrand.js`): daily modes
  and new puzzles are bit-identical on PC and in the browser.
- **Music loops** for games (`audio.py`): multi-voice and procedurally
  generated, pausing and stopping automatically with their game – first used for
  the level soundtracks in Geometry Dash.
- **Custom content** (`ugc.py`, `ugc.json`) now holds minigolf holes and Geometry
  Dash levels; minigolf stays fully compatible.
- New game hooks in `game_base.py`: `on_exit()` (the game is being left),
  `show_highscore_banner` (modes without a high score show no banner) and
  `key_is_free()` (fixed extra keys only work when not bound to an action); new
  game options are validated by `settings.SCHEMAS`, so no separate whitelist is
  needed any more.
- Tools in `devtools/` (not bundled into the `.exe`): `merge_staging.py` merges
  translations and wiki pages into all 14 language files, plus build scripts for
  the chess puzzles, Killer Sudokus, the Crossy Road models and the Geometry
  Dash levels with their solver.
- Tests: `tests/arcade_casino_audit.py` checks input, the random generator
  (Python = JS), saving, music, language files and every pre-game screen in 14
  languages × 5 resolutions, and runs the new per-game audits
  (`tests/audit_*.py`).

**Documentation**
- LamaWiki: new pages for Crossy Road, Geometry Dash (plus *Custom levels*),
  Battleship and Casino,
  reworked pages for Tetris, 2048, Chess, Wordle, Sudoku, Blackjack and Poker
  plus the general pages *Getting started* (all 20 games with a 2-player mode),
  *Options & controls* (14 languages, fixed extra keys), *Saving & high scores*
  (new sections in `mem.json`, `ugc.json`, automatic backup) and *Achievements &
  Statistics* (107 achievements, 46 games).
- Both READMEs: 46 games, new table rows and feature sections, multiplayer list,
  controls, project structure and a source note for the Lichess puzzle database.

#### Fixed
- **Blackjack**: the dealer's hole card now actually flips over when revealed.
- **Tetris**: a piece that locks entirely above the playfield now ends the game;
  Enter/Space no longer accidentally restart after game over.
- **Chess**: the win counter now counts your wins instead of wins per colour.
- **Pre-game screen**: at 480x360, Sudoku and Solitaire ran off-screen and the
  subtitle was covered.
- **Keyboard**: holding ESC toggled pause back and forth, and holding F11 did
  the same with fullscreen.

### Wordle: real word lists in 14 languages – 2026-09-16

#### Added
- **Real word lists for all 14 languages** in the new `woordlistz/` folder:
  **34,442 answers** and **213,400 accepted guesses** in total (10,000 to
  19,000 per language). Built from the Hunspell dictionaries of each language
  community and from frequency lists of real text; English uses the original
  lists of the game it is modelled on, Finnish the vocabulary of the Voikko
  project. Before there were about 100 hand-picked words in just 5 languages.
- **Guesses are checked** – as in the original, every word typed has to be in
  the list. Anything else is rejected: the row shakes briefly, a message
  appears and the guess does not count.
- **Hard mode** as a second mode on the pre-game screen: revealed hints must be
  reused – green letters stay in place, yellow ones have to appear again. The
  message says which hint is missing ("Letter 1 must be B").
- The same lists in the **web version**: only the file for the selected
  language is loaded (59–111 KB), so startup stays fast.
- `woordlistz/build_wordlists.py` rebuilds the lists at any time;
  `woordlistz/README.md` lists every source and its licence.

#### Changed
- Umlauts and accents are stored the way you would type them without special
  characters: German `Ä→AE` and `ß→SS`, Danish/Norwegian `Å→AA`,
  Swedish/Finnish `Ä→A`, otherwise the accent is dropped.
- The answers no longer contain proper nouns: in German the declension flags of
  the dictionary decide whether a capitalised word is a noun (`Blume` → answer)
  or a name (`Petra` → not).
- All lists run through the project's swear-word filter (all 14 languages at
  once).
- LamaWiki, both READMEs and the game overview mention the new numbers and hard
  mode; `pyinstall*.bat` bundles `woordlistz/` into the `.exe`.

### UI v4.2 "Midnight Glass" – new default design – 2026-09-16

#### Added
- **UI v4.2 "Midnight Glass"** as the tenth design on the **Appearance** tab: a
  deep midnight gradient with three large, soft glows in indigo, turquoise and
  magenta drifting slowly across it, topped by fine film grain and a few sparse
  stars. Panels and buttons look like frosted glass: slightly brightened
  surfaces with a bright edge along the top. Titles sit above a gradient line
  that fades out towards both ends, and a line like it separates the sidebar
  from the playfield.
- The same design in the **web version** (Settings → Design) – desktop and
  browser look identical.
- Every texture – gradient, glows, grain, stars, glass surfaces – is generated
  **procedurally** and cached; not a single image file is added. v4.2 therefore
  runs unchanged in the `.exe` and in the browser over `file://`.

#### Changed
- **New players start on UI v4.2** instead of v4.1 – via `settings.json` on the
  desktop, via browser storage on the web. Anyone who has already picked a
  design keeps it; **UI v4.1** and its four pattern variants stay selectable as
  usual.
- The design card's name and description exist in all **14 languages**; the
  default marker has moved from the v4.1 card to the v4.2 card.
- The LamaWiki (Options) and all READMEs now list **ten designs**.

### UI v1 & UI v2 are back – 2026-09-15

#### Added
- **UI v2** as the eighth design on the **Appearance** tab: the look of the
  very first `ui.py` from the "UI Rework" (commit `08739d3`) – a dark navy
  gradient with a starfield, buttons with a static glow, accent bar and arrow,
  titles with a drop shadow and a double accent line, plus that era's sidebar
  colours including the solid accent line under the header. Deliberately as
  plain as it was back then: no aurora, no sparks, no screen transitions, no
  orbiting logo decoration.
- **UI v1** as the ninth design: the state before the "UI Rework" (commit
  `cb71142`) – a plain dark background, flat buttons that show selection only
  by colour, simple titles without a line and the old sidebar colours. No
  stars, borders, shadows or transitions; pause is the simple dimmed screen
  with text again.
- Both also selectable in the **web version** (Settings → Design).

#### Changed
- `settings.json` now accepts `"theme": "v1"` and `"v2"`. The design card's name and
  description exist in all 14 languages; the LamaWiki (Options) and all
  READMEs now list nine designs.
- `ui.fx()` accepts a default value, so themes can have their own switches
  (`logo_glow`, `menu_orbit`, `style`) without touching all the others.

### Minigolf: build, play and share your own holes – 2026-08-30

#### Added
- **Hole editor**: the new **MAPS** tab on the minigolf setup screen leads to
  your own collection. **New** opens the editor: the hole on the left, a
  palette with four tools and **15 obstacle types** on the right. Rectangles
  are dragged out, round things take a single click, a pipe needs two (entrance,
  exit). Selecting, moving, deleting, **undo/redo** (keys **U**/**Y**) and grid
  snap (**G**) are all part of it.
- **Eight new obstacles** - in the engine, not just in the editor: **pipe**
  (moves the ball to the other end, keeping direction and speed), **ice**
  (almost frictionless), **sticky patch** (brakes hard), **booster** (a one-off
  shove when entered), **magnet** (pulls in or pushes away), **one-way gate**
  (only passable in the arrow's direction), **turntable** (takes the ball along
  and outwards) and **jump ramp** (the ball flies over obstacles).
- **Free hole size**: from 60x80 up to 160x240 units, in steps of ten. The
  course is re-fitted per hole; anything that would stick out when you shrink
  it moves back inside automatically.
- **12 templates** as a starting point: Empty, Pipe, Island, Windmill, Zigzag,
  Water, Bumpers, Ramp, Ice, Maze, Magnet and Jump. Each one always leaves a
  normal route open next to its trick - all twelve are provably playable within
  par according to the solver.
- **Sharing**: the button asks for your name as the creator and for the file
  name - the latter is already filled in with the **id** of the hole. From
  there it is either the usual **save dialog** or **straight into the Downloads
  folder**. Exactly one hole is exported each time, as a `.lamapgzmap` file
  (JSON inside).
- **Import** reads such a file back in (`.json` is accepted too). If the **id**
  is already taken, the import automatically appends `-2`, `-3` … instead of
  overwriting an existing hole.
- **Word filter**: hole name, id and creator name are checked when saving,
  exporting **and** importing, against **all 14 languages** - switching your
  language does not help. The lists live as regex patterns in
  `lang/swear/<code>.yml` and `lang/lang.expansion/swear/<code>.yml`, each entry
  the pattern with the word it means as a comment below it. They cope with
  injected symbols, leetspeak and s p a c e d out words.
- **Playing**: in the MAPS tab **Play** starts exactly the selected hole (with
  its own best score per hole); in the **PLAY** tab there is a fifth course
  option, **Custom**, which plays the whole collection as a round.
- **Test** in the editor tries the hole out right away and returns to building
  afterwards - no best score, no recording.
- **96 new keys** in all **14 languages**, plus a new LamaWiki page "Custom
  holes" (also 14x), the extended obstacle list on the minigolf page and both
  READMEs.

#### Changed
- Drawing the course now lives in `games/minigolf_draw.py` - game and editor
  paint the same 15 types with the same code instead of drifting apart.
- `InputEvent` now also carries the typed **character** (`event.char`).
  Without it the new text fields could take neither umlauts nor capitals - the
  Tkinter keysym alone is not enough for that.
- New in `ui.py`: **`ui.TextInput`**, the project's first text field (caret,
  character filter, placeholder) - used for name and id in the editor and for
  creator and file name in the share dialog.
- Games can now signal via `wants_escape` that they need **ESC** themselves. In
  the editor, in the MAPS tab and while test-playing, ESC therefore means
  "cancel" instead of "pause"; everywhere else it stays as before.
- Older recordings in `replay.json` run through `minigolf_gen.normalize` when
  loaded - they do not know the new obstacles and the hole size yet and get
  them filled in.
- The setup screen now has a tab row; the course block moves down accordingly
  and has a fifth button, **Custom**.
- The headless audit (`tests/newgames_audit.py`) covers custom holes with: a
  storage round trip, id rules, the word filter (including a self-test of the
  lists and a false-positive check against every existing interface text), one
  physics test per new obstacle, differing hole sizes, export/import including
  rejected foreign files, all twelve templates via the solver, and the screen
  layout in five resolutions and all 14 languages.

### Minigolf: power lock on the right mouse button – 2026-08-29

#### New
- **Power lock**: the **right mouse button** holds the shot power for as long
  as you keep it pressed. The charge bar freezes exactly where it stood when
  you pressed - with the shot fully charged you can wait for the gap in the
  windmill or for the moving block and putt at the right moment. Releasing
  charges on as usual.
- **The lock survives the stroke**: the power is held across the putt, the
  change of hole and **R** - the next left click does not restart the charge at
  5%, it putts with exactly the value you kept. Two equally strong strokes in a
  row are possible for the first time. A power dialled in with the up/down
  arrows can be pinned down the same way; the arrow keys themselves change
  nothing while the lock is held.
- **Display**: the charge bar turns gold, gains a padlock, a bright marker at
  the frozen value and a calm pulse; the HUD reads "Power locked: 62%". The aim
  line, the putter head and a pulsing ring around the ball turn gold as well -
  so your eyes can stay on the hole.
- **Percentage next to the charge bar**: the shot power is now always shown as
  a number, locked or not - which makes it easy to repeat a value on purpose.
- New key `golf.lock` in all **14 languages**, plus the Minigolf page in the
  LamaWiki (controls + obstacles) and both READMEs.

#### Changed
- Minigolf is now told about right clicks at all (`wants_right_click`); in the
  setup screen and at the end of a round they stay without effect instead of
  slipping through as a left click.
- **A pause releases the lock by itself**: while paused no release of the right
  mouse button arrives any more - otherwise the power would stay frozen.
- The headless audit (`tests/newgames_audit.py`) covers the lock as well:
  freezing, stroke, change of hole, pause, setup clicks and the HUD text in
  three resolutions and all 14 languages.

### Minigolf: auto-aim can be switched off – 2026-08-29

#### New
- **Auto-aim in the setup screen** (ON by default): until now the putter turned
  towards the cup by itself before every stroke - on the tee, after every stop
  and after every water penalty. With *Auto-aim* set to OFF the direction you
  last picked stays put, and from then on you aim entirely yourself. Only on the
  tee of a new hole there is no "last" direction - there the putter points
  neutrally up the course so nobody starts with their back to the hole. The
  setting lives in the `minigolf` section of `settings.json` and survives a
  restart.
- **Key Z** toggles the auto-aim at any time - in the setup screen just as much
  as mid-round, the way **G** (aim line) and **P** (pick-up) already work.
- New key `golf.lbl_autoaim` in all **14 languages**, plus the Minigolf page in
  the LamaWiki and both READMEs.

#### Changed
- **Tidier setup screen**: aim line, auto-aim and pick-up now sit next to each
  other as three ON/OFF pairs in a single row instead of three separate rows.
  That way the new option also fits on 480x360 - where the buttons are even
  taller than before (40 instead of 31 px).
- **The option block grows with the resolution**: the fonts always did, the
  370 px block did not - long labels such as "Ligne de visée" would no longer
  have fit above their switch group from 800x600 up. Whatever height is left
  over is spread across the gaps instead of sitting empty at the bottom.
- The headless audit (`tests/newgames_audit.py`) now covers the auto-aim as
  well - including the setup screen in five resolutions and all 14 languages.

### Replays for Minigolf & Bowling – 2026-08-29

#### New
- **Replays**: Minigolf and Bowling record every round. What is captured is not
  the input but the **actual path of ball and pins** - frame by frame in a fixed
  30 Hz grid. The replay therefore looks exactly like the round you played and
  stays valid even if the physics changes later. It is drawn by the game itself:
  the replay screen builds a normal game instance and steps it through the
  recording via `replay_begin` / `replay_seek` / `replay_draw` - including HUD,
  scorecard and the course as scenery.
- **Save with P**: at the end of a round (Minigolf) or game (Bowling), **P**
  starts the replay right away; **S** puts it into the archive and **Esc**
  returns to the round-end screen - Next/Again keep working as before. Minigolf
  also gets a **Replay** button in its button row.
- **Replay archive** via the new sidebar button **Replays**: one tab per game,
  each entry showing course or difficulty, result, date and running time. Enter
  plays, Del deletes (press twice to be safe), Tab switches the game. There is
  room for 20 recordings per game.
- **Playback like a video player**: Space pauses, Left/Right jumps to the
  previous/next sequence (shot or roll), Up/Down sets the speed (0.5x / 1x / 2x
  / 4x), and a click on the progress bar jumps anywhere. Every sequence gets a
  short run-up with the aim line and a run-out with the result; holing out,
  strikes and spares are audible. The control bar fades out after three seconds.
- **Can be switched off**: the welcome screen on the first start and the options
  (tab *General*, group **Recording**) carry the **Record replays** switch. Off
  means nothing is captured at all.
- **Two new achievements**: *Director* (first replay saved) and *Archivist*
  (five replays in the archive) - 85 in total now.
- **New wiki page "Replays"** in all 14 languages, plus `replay.json` on the
  *Saving & high scores* page.

#### Changed
- The recordings live in their **own file `replay.json`** next to `mem.json`
  (one subsection per game, written compactly). A Minigolf round costs 30-80 KB
  depending on the stroke count, a Bowling game about 90 KB - pins only appear
  in a frame when they actually moved.
- Restarting a hole with **F** now also drops that hole's strokes from the
  running recording, so scorecard and replay match.

#### Fixed
- **Bowling crashed at the end of a game**: after the final delivery the step
  counter sat at 4 and thus outside `STEPS` - the HUD and the slider row ran
  into an `IndexError` while drawing, taking the game loop down with them. The
  value is capped now, and the slider row is only drawn during play.

### Minigolf: continue instead of repeat, hole reset with F – 2026-08-28

#### Added
- **Next button at the end of a round**: instead of replaying the same set of
  nine holes, **Next** moves on to the following course - *Classic* → *Pro* →
  *Tour 1* → *Tour 2* → … up to Tour 38. The button names its target (e.g.
  "Next: Tour 12"), next to it are **Again** (same course) and **Setup**. Keys:
  Enter = continue, R = again, S = setup. With *Random* and after the last tour
  course the button is left out - those hand you new holes anyway.
- **The F key resets the current hole**: strokes back to 0, ball back on the tee
  - same hole, same player, same course. Holes you already finished stay on the
  scorecard. It also works while the ball is rolling or a shot is charged.

#### Changed
- The round-end banner now scales with the font sizes (heading, result, best,
  button row, key line) - at 1280x960 the title used to sit on the result line.

### Bug fixes & four new wiki pages – 2026-08-28

#### Fixed
- **`Game.ach_event()` did not accept a value**, although four games pass one:
  Poker (`_sync_chips`, on practically every hand), Snake in *Competitive*
  mode (every level-up), Bowling (from 200 points) and Pinball (from 50,000).
  The `TypeError` took the game loop down with it and the picture froze. On top
  of that the four value-gated achievements *poker_rich*, *snake_comp5*,
  *bowl_200* and *pin_high* were unreachable.
- **The game loop now survives an exception**: `App._loop()` scheduled the next
  frame as its last statement - if a frame threw, nothing was ever drawn again
  even though the window still responded. The body now lives in `_frame()` and
  the next frame is scheduled in a `finally`.

#### Added
- **Wiki pages for Chess, Nine Men's Morris, Simon and Billiards** in all 14
  languages. These four were the only games without a page - the wiki button on
  their pre-game screen opened the start page instead of the game. Every page
  has three sections (rules or variants, controls, AI & score), so each of the
  42 games now has its own wiki page.

### UI v4.1.1 to v4.1.4: zigzag pattern – 2026-08-28

Four new designs in the **Appearance** tab - all are exactly UI v4.1, only the
background is a tiled **zigzag pattern** instead of a gradient and starfield.
The only difference between them is the pair of pattern colours (the first one
is the dominant one).

#### Added
- **UI v4.1.1**: **black** (#000000) on **charcoal** (#424242).
- **UI v4.1.2**: the **accent blue** (91, 141, 239) on the darker
  `ACCENT_SOFT` blue (64, 94, 156) - both colours already existed in the
  palette.
- **UI v4.1.3**: **UI v4's indigo accent** (#5b8def) on black - the boldest
  of the four.
- **UI v4.1.4**: **UI v4's graphite tone** (#252934) on black - the subtlest.
- `ui.draw_zigzag()` is a 1:1 rebuild of the CSS original made of three
  `conic-gradient` layers (34x17 px tile). The tile is drawn 4x supersampled
  for smooth diagonals, cached once and then blitted row by row - so the
  background costs one build per resolution.
- Every card in the options tab shows its pattern in the mini preview.

#### Changed
- The **Appearance** tab now lays the seven designs out in a **grid**: out of
  one to three rows it picks the split with the largest card area (1280x960
  and 640x480 both give 4 + 3, very small resolutions three rows).
  Descriptions match their line count to the card height, end in "..." when
  space runs out and leave room for the "ACTIVE" badge.
- The pattern themes drop the starfield (on the pattern it would just be
  noise); **Saturn and the black hole stay**. The vignette is a little
  stronger (64 instead of 48) so tabs and footer stay readable on top of the
  pattern.
- `settings.json` now accepts `"theme": "v411"` through `"v414"`; all
  14 languages have the new names and descriptions, and the wiki has its own
  **Appearance** section.

### Minigolf Tour: 360 holes – 2026-08-26

Minigolf grows from an eighteen-hole course into a **Tour**: on top of the 18
hand-built holes come **342 generated** ones - **360 holes across 40 courses**.

#### New
- **Hole generator** `games/minigolf_gen.py`: 38 tour courses of 9 holes each,
  built entirely from a seed. Course 7, hole 3 looks the same on every start and
  on every machine - nothing has to be stored for it.
- **Ten hole families** across four difficulty tiers: straight hole, wall rows
  with offset gaps, dogleg, bumper field, water ponds, ramp, chicane, windmill
  corridor, island green and moving-block gate. Par 2 to 5, total par per course
  between 26 and 39.
- **Passability is built in, not hoped for**: every family first fixes the path
  from tee to cup and builds the obstacles around it - wall gaps are never
  narrower than 12 units (ball diameter 3.4), water only sits beside the path, a
  moving block is always narrower than its gap, and windmill arms leave room at
  the sides.
- **Course picker in the setup**: four buttons (Classic / Pro / Tour / Random)
  and below them a row with arrows to page through the 38 tour courses, showing
  the total par. The best result is stored per tour course.
- **Random** now draws from all 360 holes instead of just the 18 built ones.
- **Pick-up can be switched off**: the rule *after eight strokes the hole is
  over* stays the default but can be turned off in the setup screen (or with
  **P**) - then you keep putting until the ball drops. The setup screen gained
  its own ON/OFF row for it and now scales its height with the resolution so
  that even 480x360 shows all five blocks.
- **Cancel a shot with R**: hold the mouse button, load the power, change your
  mind - press **R** and the ball stays put, the stroke does not count, and
  after releasing you can charge again as usual. A short note in the header
  confirms the cancellation.

#### Changed
- `tests/newgames_audit.py` no longer checks only the 18 built holes but **all
  342 generated** ones as well: free tee and cup positions, being sinkable
  (verified by a solver that simulates real strokes) and reproducibility from the
  seed. Runtime about one minute.
- `settings.json` remembers the last tour course (`minigolf.tour`); the course
  dimensions now live in exactly one place, `minigolf_gen.py`.
- The subtitle and both READMEs carry the new number: 360 holes.

### Minigolf, Pinball & Bowling – 2026-08-26

Three new sports games in one go (no. 40-42): **Minigolf** with 18 hand-built
holes, a full **Pinball** machine with multiball and **Bowling** with real pin
physics - all three with a multiplayer mode, achievements, a wiki page and all
14 languages.

#### New
- **Minigolf**: 18 hand-built holes across three courses - *Classic* (a gentle
  start), *Pro* (island green, double windmill, moving blocks) and *Random*
  (nine holes drawn and mirrored at random). Sand slows you down, ramps
  accelerate, water costs a penalty stroke, plus rubber bumpers, windmills and
  moving blocks. The mouse aims, holding the left button loads the power
  (arrows + space work too), and **G** toggles the aim line. A scorecard shows
  par per hole, **500 bonus points for a hole in one**; in two-player mode each
  player takes the same hole in turn. Best result per course in `mem.json`
  (section `minigolf`).
- **Pinball**: three tables - *Classic* (three pop bumpers, one target bank),
  *Space* (four bumpers in a diamond, two banks) and *Lama* (open playfield, six
  targets in an arc). With slingshots, four **L-A-M-A** rollover lanes, a saucer
  with ball lock and **multiball including jackpot**, plus a six second ball
  save, nudging via the up key, **TILT** after three hasty nudges and a
  multiplier up to x5. 3 or 5 balls per game, alternating in two-player mode. A
  weak plunge is no disaster: the ball rolls back into the shooter lane and you
  may shoot again. Best result per table in `mem.json` (section `pinball`).
- **Bowling**: ten frames with full strike/spare scoring including the bonus
  balls of the tenth frame (maximum: a 300 game) and a scorecard with X, / and
  the running total. The delivery runs in four steps - position, aim, spin and
  power - through sliders that swing on their own and can also be set by hand
  with left/right. Ten pins with real mass knock each other over; the lane is
  oiled up front, so the **hook** only bites in the last third. Best result per
  difficulty in `mem.json` (section `bowling`).
- **9 new achievements**: score milestones for all three games plus hole in one,
  a round under par, multiball, 50,000 points in pinball, a turkey (three
  strikes in a row) and a 200 bowling game - now **83 in total**.
- **93 translation keys per language** and **three new wiki pages** in all
  14 languages (LamaWiki: 43 pages); both READMEs updated.
- **Headless audit** `tests/newgames_audit.py`: checks all 18 minigolf holes
  with a solver for being sinkable (and for free tee/cup positions), every
  pinball table for a successful launch and a game that ends without hanging,
  and the bowling scoring against reference games (a 300, all spares, mixed)
  including pin physics and frame logic.

#### Changed
- Sidebar: three new pictograms (flag in the hole, flipper with ball, pin with
  ball) with their own accent colours; the collection now counts **42 games**.
- `settings.json` knows the sections `minigolf`, `pinball` and `bowling`
  (course & aim line, table & ball count, difficulty & guide) - all three
  remember their setup choice.

### Block Jump: Minecraft skin – 2026-08-12

Visual update: **Block Jump** now looks like the game it is modelled on - real
pixel textures on every block, a Steve figure, the first-person hand, a beacon
beam at the goal, pixel clouds and a HUD with hearts.

#### New
- **Block textures** (16×16, generated procedurally in code - no image files):
  grass with its green overhang on the sides, dirt, stone, oak planks, a
  diamond block (the goal), a slime block (the former spring block) plus wood
  for ladders and fences. The software renderer splits every face into
  perspective-correct texel quads, and face brightness follows the original
  (bright top, graded sides, dark bottom).
- **Distance-based level of detail** on **key T**: high / low / off. Texels
  that end up the same colour are merged into rectangles up front, so the new
  look costs about 3,000 quads per frame instead of roughly 18,000
  (1280×720: ~14 ms on *high*, ~6 ms on *low*, ~3 ms with *off*). The choice is
  stored in `settings.json` (`blockjump.textures`).
- **Steve as the player** in the chase camera: head with a face, torso, arms
  and legs swinging while walking, the head tilting with your view.
- **First-person hand** with walk bob, a **beacon beam** above the goal and
  spinning **gold ingots** instead of the previous crystal coins.
- **Sky**: Minecraft blue with a square **sun** and drifting **pixel clouds**
  high above the map.
- **HUD in the same style**: **hearts** for lives, a gold ingot as the coin
  counter, drop-shadow text and a crosshair like the original.

#### Changed
- The spring block now looks like a **slime block** and the goal like a
  **diamond block** with a light beam. Gameplay, level generation and physics
  are unchanged - `tests/blockjump_audit.py` still passes all 45 levels.

### Block Jump bugfixes – 2026-08-12

Maintenance update: **Block Jump** is now actually beatable - the ladder
climb used to end in a dead end in practically every level.

#### Fixed
- **Ladders were dead ends**: The follow-up pad was built directly on top of
  the ladder shaft and overwrote the top rung - the player bumped their head
  while climbing and never reached the top. The pad now starts behind the
  support wall, keeping the shaft open (affected 44 of 45 audited levels,
  including level 1 of all three modes).
- **Ladder coin unreachable**: The coin floated one block in front of the
  ladder instead of inside the climbing column - it is now collected during
  the ascent.
- **The C key (capture/release mouse) did nothing**: The draw routine
  overwrote the toggle every frame; the HUD hint is accurate again.
- **Invisible walls**: Fences and spring blocks collided as full 1×1×1
  cubes even though they are drawn much smaller. Fences are now 0.8 blocks
  tall (comfortably jumpable, as the README describes), spring blocks no
  longer block sideways at all and also catapult when walked into; the pad
  surface underneath both is no longer culled away (visible holes) and the
  third-person camera no longer snags on fences.
- **Hard mode**: At the maximum gap (4 blocks), 1-block-deep target pads
  plus an upward offset could produce frame-perfect-impossible jumps -
  large gaps now force level/downhill, deep landing pads.
- **Key bindings**: Block Jump ignored the keys configured in the options
  (hardcoded WASD/arrows) - it now uses the central bindings like the other
  games (arrow keys remain as a fallback).
- **Settings were lost**: View/motion blur/sensitivity/mouse direction did
  not survive a restart (missing `blockjump` section in the settings
  defaults).

#### Added
- **Headless audit** `tests/blockjump_audit.py`: automatically checks all
  45 levels (1-15 × 3 modes) for open ladder shafts, reachable coins, gaps
  clearable by the jump physics, and simulates every ladder climb.

### Tower Defense – 2026-08-06

Game no. 39: a complete **tower defense** with endless waves, 4 maps and
3 content modes - including achievements, a wiki page and translations into
all 14 languages.

#### Added
- **Tower Defense** (`games/lamatowerdefense.py`): enemies march along a path in
  waves; build towers next to it, upgrade them (up to 3 levels) and sell at a
  **70% refund**. Every breakthrough costs lives (bosses 5), at 0 the run
  ends - your points count as the high score.
- **4 maps**, each with its own path and HP difficulty factor: **Meadow**,
  **Canyon**, **Crossroads** (the path crosses itself - strong for towers,
  but tougher enemies) and **Gauntlet** (building only right next to the
  path). Locked maps unlock via your best wave (5/10/15); the best wave per
  map is stored (section `lamatowerdef` in `mem.json`).
- **Endless waves by formula** instead of a wave script: budget, enemy HP and
  bounty scale with the wave number (exponentially after wave 25); a **boss
  every 8 waves**, a themed wave every 5th, and the map factor ramps up fully
  only from wave 10.
- **3 modes** on the pre-game screen: **Classic** (7 towers, the main mode),
  **Compact** (4 towers, 2 levels, more starting gold) and **Maximal**
  (11 towers, **A/B specialisation** at top level, special enemies and active
  abilities **Meteor [Q] / Frost Nova [W] / Gold Rush [E]**).
- **11 tower types**: Arrow, Cannon (splash), Frost (slows + reveals), Sniper
  (huge range, armor piercing, anti-air), Poison (damage over time), Tesla
  (chain lightning), Banner (+20% aura), Mortar (arcing shots across most of
  the field), Flak (anti-air), Laser (ramping beam) and Gold Bank (income) -
  each with its own A/B branch in Maximal mode (22 branches, e.g. Double
  Shot/Piercing on the arrow tower).
- **11 enemy types**: runt, sprinter, swarm, tank, armored (flat damage
  reduction), regenerator, splitter, **flyer** (own air route, only anti-air
  hits), **cloaked** (visible only near Frost/Banner/Sniper), **healer**
  (heal aura) and **boss**.
- **Comfort**: ghost preview with range circle, right-click cancels, **1-9**
  quick select, **[F]** 2x speed, **[G]** all ranges, mouse wheel scrolls the
  build bar, **+5% interest** per build phase, wave bonuses.
- **5 new achievements**: a 10,000-point milestone plus **Boss Hunter**
  (first boss), **Wavebreaker** (wave 20), **Fortress** (wave 10 without
  losses) and **Full Build** (fully upgraded tower) - 74 achievements total.
- **76 new translation keys per language** and a **new LamaWiki page** in all
  14 languages (the wiki now has 40 pages); both READMEs (DE/EN + 12 more
  languages) extended with a table row and feature details.

#### Changed
- Sidebar: new hand-drawn **castle-tower icon** and terracotta accent colour
  for Tower Defense (`ui.GAME_COLORS`); the game list now counts 39 games.

### Achievements & Statistics – 2026-07-26

The progression update: the collection now tracks what you play - **69
achievements** and persistent **player statistics** across all 38 games,
reachable via the new **"Achievements & Stats"** sidebar button (trophy icon).

#### Added
- **69 achievements** in three categories:
  - **General** (21): collection-wide goals such as games started (1/10/50/200),
    games tried (5/15/all 38), total play time (1 h/5 h/20 h), records broken
    (1/10/25), wins (10/50, in 5 different games) - plus extras like **Night
    Owl** (play after midnight), **Early Bird**, **Polyglot** (switch language),
    **Bookworm** (open the wiki) and **My Own Style** (Snake personalisation).
  - **Score milestones** (30): one fair target score per game, tuned to its
    scoring scale (e.g. Snake 500, Tetris 10,000, Flappy 25, Simon 12).
  - **Special moments** (18): checkmating the AI, a YAHTZEE, the 2048 tile, a
    Tetris quadruple line clear, a two-card Blackjack, a Wordle in ≤ 2 tries,
    Hangman without a miss, a flawless Sudoku, Minesweeper and Solitaire wins,
    a perfect Memory run, an emptied Pac-Man maze, all Breakout levels, all
    Frogger bays, 100% Aim Trainer accuracy, 2,000 poker chips, Snake prestige
    and competitive level 5.
- **Toast notification**: a golden banner with a drawn badge and a little
  arpeggio in the top-right corner on unlock - even mid-game.
- **Achievements & statistics screen**: an **Achievements** tab with an overall
  progress bar, categories, per-achievement progress and unlock dates; a
  **Statistics** tab with overview cards (total play time, games, wins,
  records, games tried, achievement count), your **favourite game** and a
  **per-game table** sorted by play time (plays · time · wins · best).
  Clicking a row jumps straight to that game.
- **Persistent statistics** per game: plays (including restarts), **active**
  play time (pauses/menus don't count), wins/losses (games with a clear
  outcome, in single-player), records broken, last played - stored as the
  `stats`/`achievements` sections of `mem.json`.
- **Existing data counts**: on the first launch after the update, your old
  high scores are credited automatically (without the toast fireworks).
- **94 new translation keys in all 14 languages** (~1,300 new strings) and a
  **new LamaWiki page** "Achievements & Statistics" - also in all 14 languages
  (the wiki now has 39 pages).

#### Changed
- `game_base.py`: new hooks for every game - `report_result()` (win/loss,
  counted once per round) and `ach_event()` (special moments); wired into
  30+ games.
- Central game loop (`main.py`): counts game starts/restarts and play time
  automatically; saving a record now also checks milestone achievements.
  Statistics are written throttled (at most every 20 s plus on game
  start/end and on quit).

### Language expansion: 9 new languages – 2026-07-21

The interface is now available in **14 languages**. Nine new ones were added
which — like Spanish and Portuguese before them — hide behind the **"More
languages"** button on first launch: **Polski, Türkçe, Dansk, Norsk, Svenska,
Suomi, Čeština, Slovenščina, Hrvatski**.

#### New
- **9 new UI languages**, each fully translated (802 strings per language):
  Polish, Turkish, Danish, Norwegian, Swedish, Finnish, Czech, Slovenian,
  Croatian.
- **LamaWiki fully translated too**: all 38 wiki pages in each of the nine new
  languages.
- **Documentation**: a complete README section for every new language in
  `other.readme.md` (same format as FR/ES/PT).
- The new language files are bundled in `lang/lang.expansion/` and
  `lamawiki/lang.expansion/` — the core languages (de/en/fr/es/pt) are untouched.

#### Changed
- **Grid language picker**: the language and welcome screens now lay out the 14
  languages in a responsive grid with automatic font sizing — fits cleanly from
  480×360 to 1280×960 (long names like "Slovenščina" no longer overflow).
- Loaders (`i18n.py`, `lamawiki.py`) also search the `lang.expansion`
  subfolders; a missing string still falls back to German.
- Build scripts (`pyinstall.bat`, `pyinstall-pyarmor.bat`) now bundle the new
  `lamawiki/lang.expansion/*.json` into the EXE.

#### Notes
- Wordle and Hangman use the English word lists for the new languages for now
  (dedicated word lists can be added later).

### Games Rework – 2026-07-19

The biggest overhaul so far: **all 38 games** were brought to a common standard
in one pass (visuals, consistency, translations, bug fixes).

#### Added
- **Unified in-game look**: all 38 games now use the menu's theme palette and
  font. HUDs, setup screens and overlays follow the design chosen in the options
  (v4.1 / v4 / Classic) - playfields keep their identity colours (felt green,
  Pac-Man maze, Tetris pieces …).
- **Language-aware game names** in the menu: Schach → *Chess/Échecs/Ajedrez/
  Xadrez*, Mühle → *Nine Men's Morris*, Vier gewinnt → *Connect Four*,
  Panzer-Duell → *Tank Duel*, 3D-Labyrinth → *3D Maze*, Dame → *Checkers*,
  Billard → *Billiards*, Galgenmännchen → *Hangman*, Schiebepuzzle → *Sliding
  Puzzle*.
- **Responsive everywhere**: every game now cleanly handles mid-game resolution
  changes (fonts, layout, playfield) - 11 older games couldn't do this at all
  before, 3 more only partially.
- **Blackjack**: deal and reveal animations actually play now (cards fly from
  the shoe, the hole card flips with sound) - the code existed but never ran.
- 13 new translation keys in **all 5 languages** (incl. Breakout/Tic-Tac-Toe
  difficulty labels, Flappy medals, Sudoku erase key).

#### Changed
- Shared base (`game_base.py`): language-aware names (`LocalizedName`),
  documented hooks (`on_surface_changed`, `capture_mouse`, `MODES`), themed
  fonts and per-game accent colour provided automatically.
- Setup screens unified on the UI building blocks (`draw_title`, `draw_button`,
  `draw_footer`); layouts scale with window height instead of fixed pixels.
- Unified game over: translucent panel with accent border, pulsing hint;
  **Enter and Space** (often mouse click too) restart everywhere.
- Card games: shared table look (`make_felt`), themed card backs in each game's
  accent colour.
- Performance: per-frame full-surface alpha fills replaced by cached overlays
  (Aim Trainer, Tunnel Racer, Pac-Man, Tetris, Tanks, Sudoku, Simon, and more);
  2048 created a new font per tile per frame - now cached.
- Cleanup: 15 duplicated accent-colour constants, dead variables/branches and
  unused constants removed.

#### Fixed
- **Breakout**: bomb bricks exploded twice (double points/drops); fireball
  skipped bricks or hit wrong ones; power-up fall speed was framerate-dependent.
- **Chess**: mouse cursor was mirrored when the human played Black; "Black wins"
  was virtually invisible in multiplayer (black on dark).
- **Nine Men's Morris**: when the AI closed a mill the highlight vanished
  instantly - it now glows gold for ~1.2 s.
- **Solitaire**: undoing a stock draw left cards permanently face-up (cheat
  bug); undo while dragging could corrupt piles.
- **Poker**: hold click areas in 5 Card Draw sat below the cards; going broke in
  Video Poker was silent.
- **Frogger**: game logic kept running after game over - the timer "killed" the
  frog in a loop (negative lives, sound/rumble spam).
- **Snake**: after a resolution change the board restarted with the old size.
- **Tank Duel**: arrow keys were dead in single-player; resizing on the match
  end screen drew the old arena.
- **T-Rex Runner**: a re-bound jump key got "stuck" on release (permanent low
  jumps).
- **Minesweeper**: right-clicking the smiley restarted the game; the difficulty
  name could overlap the mine counter.
- **Asteroids**: ships respawned off-screen after a resolution change; the
  controls hint never appeared after spending time in setup.
- **Pac-Man**: score popups sat at wrong positions after a resolution change.
- **Invaders**: bullets in flight changed colour when switching weapons.
- **Pong**: name label ran off-screen with long translations.
- **Doodle Jump**: the doodler's snout never mirrored with its direction;
  propeller-hat monster kills were silent.
- **Tic-Tac-Toe**: difficulty buttons were clickable in multiplayer despite
  being greyed out (now disabled).
- **Checkers**: "S = setup" only worked in single-player despite the hint.
- **Reversi/Connect Four**: multiplayer winner text was barely readable;
  misleading "S = setup" hint removed.
- **Aim Trainer / 3D Maze / Tunnel Racer**: resolution changes rebuilt only one
  of four fonts.
