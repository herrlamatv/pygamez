# woordlistz – die Wortlisten für Wordle

Hier liegen die Wortlisten, aus denen das Wordle in PyGameZ seine Rätsel zieht –
**für alle 14 Sprachen der Spielesammlung und die Wortlängen 4 bis 7**, aufgebaut
wie beim Original:

| Datei | Inhalt |
|-------|--------|
| `<sprache>/answers.txt` | **Lösungswörter** mit 5 Buchstaben – geläufige Wörter, die als Rätsel drankommen können |
| `<sprache>/allowed.txt` | **Erlaubte Rateworte** mit 5 Buchstaben – alles, was man eintippen darf (Obermenge der Lösungswörter) |
| `<sprache>/answers4.txt`, `answers6.txt`, `answers7.txt` | Lösungswörter mit 4, 6 bzw. 7 Buchstaben |
| `<sprache>/allowed4.txt`, `allowed6.txt`, `allowed7.txt` | Rateworte mit 4, 6 bzw. 7 Buchstaben |

Die 5-Buchstaben-Listen behalten ihre alten Namen (klassisches Wordle), die
anderen Längen bekommen die Zahl angehängt. Jede Datei enthält genau ein Wort je
Zeile, groß geschrieben, alphabetisch sortiert, immer **genau N Buchstaben A–Z**.

Lösungswörter / Rateworte je Sprache und Länge:

| Sprache | 4 Buchstaben | 5 Buchstaben | 6 Buchstaben | 7 Buchstaben |
|---------|-------------:|-------------:|-------------:|-------------:|
| Deutsch (de) | 837 / 6.289 | 1.436 / 11.536 | 2.316 / 15.256 | 2.569 / 15.853 |
| English (en) | 1.500 / 4.035 | 2.313 / 14.866 | 2.600 / 15.325 | 2.600 / 19.000 |
| Français (fr) | 1.043 / 8.101 | 2.055 / 15.490 | 2.886 / 17.280 | 3.395 / 19.000 |
| Español (es) | 1.052 / 10.089 | 2.168 / 16.097 | 2.903 / 17.846 | 3.413 / 19.000 |
| Português (pt) | 816 / 8.178 | 1.583 / 18.619 | 2.019 / 19.000 | 2.193 / 19.000 |
| Polski (pl) | 1.728 / 10.563 | 3.103 / 18.538 | 3.897 / 19.000 | 4.000 / 19.000 |
| Türkçe (tr) | 1.577 / 9.447 | 3.873 / 18.750 | 4.000 / 19.000 | 4.000 / 19.000 |
| Dansk (da) | 1.337 / 5.512 | 1.969 / 10.111 | 2.290 / 14.540 | 2.215 / 17.028 |
| Norsk (no) | 1.542 / 6.004 | 2.156 / 11.069 | 2.220 / 15.399 | 2.017 / 19.000 |
| Svenska (sv) | 1.573 / 6.071 | 2.425 / 12.138 | 2.761 / 19.000 | 2.740 / 19.000 |
| Suomi (fi) | 683 / 6.019 | 1.787 / 14.127 | 1.740 / 17.435 | 1.624 / 18.646 |
| Čeština (cs) | 1.622 / 11.183 | 3.401 / 17.184 | 4.000 / 19.000 | 4.000 / 19.000 |
| Slovenščina (sl) | 1.483 / 7.969 | 3.484 / 18.741 | 4.000 / 19.000 | 4.000 / 19.000 |
| Hrvatski (hr) | 1.106 / 10.464 | 2.094 / 16.109 | 2.723 / 17.803 | 3.116 / 19.000 |
| **zusammen** | **17.899 / 109.924** | **33.847 / 213.375** | **40.355 / 244.884** | **41.882 / 260.527** |

Jede Sprache hat in jeder Länge mindestens 500 Lösungswörter (das Skript bricht
sonst mit einem Fehler ab), höchstens 4.000 Lösungswörter und 19.000 Rateworte.

## Umlaute und Akzente

Die Bildschirmtastatur im Spiel hat nur A–Z, deshalb ist jedes Wort in der
Schreibweise abgelegt, die man **ohne Sonderzeichen** tippen würde:

* Deutsch: `Ä→AE`, `Ö→OE`, `Ü→UE`, `ß→SS` – aus *BÖSE* wird `BOESE`
* Dänisch/Norwegisch: `Æ→AE`, `Ø→OE`, `Å→AA` – aus *SØLV* wird `SOELV`
* Schwedisch/Finnisch: `Å/Ä→A`, `Ö→O` – aus *TALVI* bleibt `TALVI`, aus *PÄÄSY* wird `PAASY`
* alle übrigen Sprachen: Akzent weg – `É→E`, `Ñ→N`, `Ł→L`, `Ş→S`, `Č→C`

Wörter, die dabei nicht auf eine der Längen 4–7 kommen, fallen heraus.

## Woher die Wörter kommen

| Quelle | wofür | Lizenz |
|--------|-------|--------|
| [wooorm/dictionaries](https://github.com/wooorm/dictionaries) (Hunspell, aus den LibreOffice-Wörterbüchern) | echte Wörterbuch-Einträge je Sprache | je Sprache eigene freie Lizenz (GPL/LGPL/MPL/BSD – siehe dort) |
| [hermitdave/FrequencyWords](https://github.com/hermitdave/FrequencyWords) (OpenSubtitles 2018) | gebeugte Formen aus echtem Text + wie geläufig ein Wort ist | MIT |
| [voikko/corevoikko](https://github.com/voikko/corevoikko) (`joukahainen.xml`) | Finnisch, weil es dort kein Hunspell-Wörterbuch gibt | GPL |
| [Wordle-Listen der New York Times](https://gist.github.com/cfreshman) | Englisch, 5 Buchstaben: exakt die Lösungswörter und Rateworte des Originals | – |
| [ENABLE](https://github.com/dolph/dictionary) (Scrabble-Wortliste) | Englisch, 4/6/7 Buchstaben: Rateworte; Maßstab für „englische Mitbringsel“ in den anderen Sprachen | gemeinfrei |
| [WordNet 3.0](https://wordnet.princeton.edu/) (Princeton) | Englisch, 4/6/7 Buchstaben: nur Grundformen als Lösungswörter (kein *drummed*, *bigger*) | WordNet-Lizenz (frei) |

So entstehen die Listen je Sprache und Länge:

* **Rateworte** = Wörterbuch **+** die geläufigsten Formen aus echtem Text.
  Erst diese Mischung bringt auch gebeugte Formen wie `HAUSE` oder `KOMMT`, die
  in einem Wörterbuch nur als Grundform stehen.
* **Lösungswörter** = Wörter, die **beides** sind: im Wörterbuch **und**
  geläufig. Das hält Tippfehler, Namen und Wortfetzen (`DOESN`, `WEREN`) aus den
  Rätseln heraus. Dazu kommen diese Filter:
  * **Eigennamen**: in den meisten Sprachen stehen sie groß im Wörterbuch. Im
    Deutschen, wo jedes Substantiv groß geschrieben wird, entscheiden die
    Deklinations-Flags (`Blume/Nm` → Substantiv, `Petra/S` → Name) und bei
    unklaren Einträgen der Blick in die anderen 12 Wörterbücher: steht ein Wort
    dort nur groß (`Basel`, `Kongo`, `Tokio`), ist es ein Name. Dazu eine von
    Hand gesichtete Liste klein geschriebener Namen (`NAME_BLOCK`).
  * **Wörterbuch-Flags**: Einträge, die Hunspell als verboten
    (`FORBIDDENWORD`), als reinen Wortbaustein (`NEEDAFFIX` – deutsche Verbstämme
    wie `TRINK`, `STECK`) oder als „nicht vorschlagen“ (`NOSUGGEST` – vulgär,
    Slang) markiert, werden keine Lösungswörter.
  * **Englische Mitbringsel**: Untertitel enthalten viel Englisch. Ein Wort, das
    im Englischen mehr als 25-mal so häufig ist wie in der Sprache selbst
    (`HOUSE`, `PARTY` im Polnischen), wird keine Lösung.
  * **Schimpfwörter und anstößige Wörter**: `swear.py` (alle 14 Sprachen
    gleichzeitig) und zusätzlich eine Liste von Wörtern rund um Sex, Drogen und
    Fäkalsprache (`ADULT_BLOCK`) – als Rateworte bleiben sie erlaubt.
  * **Ausnahmen** (`ANSWER_KEEP`): echte, geläufige Wörter, die ein Filter
    fälschlich treffen würde – deutsch `TASTE`, `LISTEN`, `OPERN`, französisch
    `STORE`, tschechisch `MORE` (moře), portugiesisch `SOBRE` …
* Englisch mit 4/6/7 Buchstaben (ohne Original-Liste): Lösungswörter müssen in
  Hunspell, ENABLE und WordNet stehen, geläufig sein und durch eine kleine
  Sperrliste (Ausrufe, Umgangssprache, Anstößiges); genommen werden die 1.500
  (4) bzw. 2.600 (6/7) häufigsten.

## Was sich an den 5-Buchstaben-Listen geändert hat (September 2026)

Mit dem Ausbau auf 4–7 Buchstaben kamen die oben beschriebenen Filter hinzu.
Bei den 5-Buchstaben-Listen sind dadurch **nur Wörter weggefallen, keine neuen
dazugekommen** – Englisch ist unverändert (Original-Listen):

| Sprache | Lösungswörter vorher → jetzt | Rateworte vorher → jetzt | Beispiele für entfernte Lösungswörter |
|---------|-----------------------------:|-------------------------:|---------------------------------------|
| de | 1.559 → 1.436 | 11.552 → 11.536 | Namen (`PARIS`, `RHEIN`, `KAIRO`), Verbstämme (`TRINK`, `STECK`), Englisch (`POWER`, `SOUND`), `PORNO` |
| fr | 2.112 → 2.055 | 15.490 | Englisch (`HOUSE`, `SMALL`, `TRUCK`), Namen (`JULES`, `VICHY`), `CONNE` |
| es | 2.187 → 2.168 | 16.097 | Englisch (`LIGHT`, `ITEMS`), Namen (`DIEGO`, `CUZCO`) |
| pt | 1.631 → 1.583 | 18.621 → 18.619 | Namen (`MIAMI`, `PAULO`), Englisch (`SCORE`, `LOBBY`), Präfixe (`HIPER`, `MULTI`) |
| pl | 3.222 → 3.103 | 18.538 | Englisch (`HOUSE`, `PARTY`, `HELLO`), Namen (`NOKIA`, `TEXAS`), `DILDO` |
| tr | 3.896 → 3.873 | 18.750 | Namen (`ALICE`, `TOKYO`), Englisch (`BEGIN`), `SEKSI` |
| da | 2.019 → 1.969 | 10.118 → 10.111 | Englisch (`WORLD`, `HAPPY`), Namen (`DAVID`, `BIDEN`), `PISSE` |
| no | 2.194 → 2.156 | 11.069 | Englisch (`WATER`, `HORSE`), Namen (`HILDA`, `MINSK`), `PORNO` |
| sv | 2.474 → 2.425 | 12.138 | Kraftausdrücke laut Wörterbuch (`KNULL`, `RUNKA`), Englisch (`HEAVY`), Namen (`OSKAR`) |
| fi | 1.802 → 1.787 | 14.127 | Namen (`MESSI`, `KOREA`), `PORNO`, `SEKSI` |
| cs | 3.431 → 3.401 | 17.184 | Kraftausdrücke laut Wörterbuch (`PRDET`, `SRANI`), Englisch (`JEANS`), Namen (`ANTON`) |
| sl | 3.498 → 3.484 | 18.741 | Namen (`BORIS`, `ZORAN`), Englisch (`MUSIC`), `PENIS` |
| hr | 2.104 → 2.094 | 16.109 | Englisch (`STAND`, `TOTAL`), `ELISA`, `PORNO` |

Bei den Rateworten fehlen nur Einträge, die das Wörterbuch selbst als verboten
oder als reinen Wortstamm führt (deutsch `ABHOL`, `SPAEH`; dänisch `BILET`;
portugiesisch `AGUES`). Echte Wörter, die ein Filter fälschlich getroffen hätte,
stehen in `ANSWER_KEEP` und sind geblieben.

## Neu bauen

```bat
python woordlistz/build_wordlists.py                :: alle 14 Sprachen, Längen 4-7
python woordlistz/build_wordlists.py de en          :: nur einzelne Sprachen
python woordlistz/build_wordlists.py --lengths 6 7  :: nur einzelne Längen
python woordlistz/build_wordlists.py --sample 40    :: Stichproben + entfernte Englisch-Wörter zeigen
```

Das Skript lädt die Quellen beim ersten Lauf herunter (zusammen ~320 MB) und
legt sie im Temp-Ordner ab – nicht im Projekt. Ein erneuter Lauf mit denselben
Quellen erzeugt exakt dieselben Dateien (auch mit `--lengths 5` allein). Danach:

```bat
node web/tools/build-wordlists.js               :: Web-Version nachziehen
```

Damit entstehen `web/js/games/wordle_words/<code>.js` (5 Buchstaben) und
`<code>4.js`, `<code>6.js`, `<code>7.js`; die Web-Version lädt davon immer nur
die Datei der gerade gespielten Sprache und Länge nach.

## Wer die Listen liest

* **Desktop**: `games/wordle_words.py` (`words_for(lang, length)` /
  `allowed_for(lang, length)` / `has_length(lang, length)`). In der `.exe` liegt
  dieser Ordner dank `--add-data "woordlistz;woordlistz"` mit drin – fehlt er,
  greift für 5 Buchstaben eine kurze eingebaute Notfallliste.
* **Web**: `web/js/games/wordle_words.js` mit den erzeugten Sprachdateien.
* **Tageswort**: die Lösungswörter werden je Sprache und Länge fest gemischt
  (`seedrand`, am PC und im Browser identisch) – jede Änderung an einer
  Lösungsliste verschiebt also die künftigen Tageswörter dieser Liste.
