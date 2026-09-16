# woordlistz – die Wortlisten für Wordle

Hier liegen die Wortlisten, aus denen das Wordle in PyGameZ seine Rätsel zieht –
**für alle 14 Sprachen der Spielesammlung**, aufgebaut wie beim Original:

| Datei | Inhalt |
|-------|--------|
| `<sprache>/answers.txt` | **Lösungswörter** – geläufige Wörter, die als Rätsel drankommen können |
| `<sprache>/allowed.txt` | **Erlaubte Rateworte** – alles, was man eintippen darf (Obermenge der Lösungswörter) |

Beide Dateien enthalten genau ein Wort je Zeile, groß geschrieben, alphabetisch
sortiert, immer **5 Buchstaben A–Z**.

| Sprache | Lösungswörter | Rateworte |
|---------|--------------:|----------:|
| Deutsch (de) | 1.559 | 11.552 |
| English (en) | 2.313 | 14.866 |
| Français (fr) | 2.112 | 15.490 |
| Español (es) | 2.187 | 16.097 |
| Português (pt) | 1.631 | 18.621 |
| Polski (pl) | 3.222 | 18.538 |
| Türkçe (tr) | 3.896 | 18.750 |
| Dansk (da) | 2.019 | 10.118 |
| Norsk (no) | 2.194 | 11.069 |
| Svenska (sv) | 2.474 | 12.138 |
| Suomi (fi) | 1.802 | 14.127 |
| Čeština (cs) | 3.431 | 17.184 |
| Slovenščina (sl) | 3.498 | 18.741 |
| Hrvatski (hr) | 2.104 | 16.109 |
| **zusammen** | **34.442** | **213.400** |

## Umlaute und Akzente

Die Bildschirmtastatur im Spiel hat nur A–Z, deshalb ist jedes Wort in der
Schreibweise abgelegt, die man **ohne Sonderzeichen** tippen würde:

* Deutsch: `Ä→AE`, `Ö→OE`, `Ü→UE`, `ß→SS` – aus *BÖSE* wird `BOESE`
* Dänisch/Norwegisch: `Æ→AE`, `Ø→OE`, `Å→AA` – aus *SØLV* wird `SOELV`
* Schwedisch/Finnisch: `Å/Ä→A`, `Ö→O` – aus *TALVI* bleibt `TALVI`, aus *PÄÄSY* wird `PAASY`
* alle übrigen Sprachen: Akzent weg – `É→E`, `Ñ→N`, `Ł→L`, `Ş→S`, `Č→C`

Wörter, die dabei nicht auf genau 5 Buchstaben kommen, fallen heraus.

## Woher die Wörter kommen

| Quelle | wofür | Lizenz |
|--------|-------|--------|
| [wooorm/dictionaries](https://github.com/wooorm/dictionaries) (Hunspell, aus den LibreOffice-Wörterbüchern) | echte Wörterbuch-Einträge je Sprache | je Sprache eigene freie Lizenz (GPL/LGPL/MPL/BSD – siehe dort) |
| [hermitdave/FrequencyWords](https://github.com/hermitdave/FrequencyWords) (OpenSubtitles 2018) | gebeugte Formen aus echtem Text + wie geläufig ein Wort ist | MIT |
| [voikko/corevoikko](https://github.com/voikko/corevoikko) (`joukahainen.xml`) | Finnisch, weil es dort kein Hunspell-Wörterbuch gibt | GPL |
| [Wordle-Listen der New York Times](https://gist.github.com/cfreshman) | Englisch: exakt die Lösungswörter und Rateworte des Originals | – |

So entstehen die beiden Listen:

* **Rateworte** = Wörterbuch **+** die geläufigsten Formen aus echtem Text.
  Erst diese Mischung bringt auch gebeugte Formen wie `HAUSE` oder `KOMMT`, die
  in einem Wörterbuch nur als Grundform stehen.
* **Lösungswörter** = Wörter, die **beides** sind: im Wörterbuch **und**
  geläufig. Das hält Tippfehler, Namen und Wortfetzen (`DOESN`, `WEREN`) aus den
  Rätseln heraus. Im Deutschen, wo jedes Substantiv groß geschrieben wird,
  entscheiden zusätzlich die Deklinations-Flags des Wörterbuchs, ob ein
  großgeschriebener Eintrag ein Substantiv (`Blume/Nm` → Lösungswort) oder ein
  Eigenname (`Petra/S` → kein Lösungswort) ist.
* Beide Listen laufen durch den Schimpfwortfilter des Projekts (`swear.py`,
  alle 14 Sprachen gleichzeitig).

## Neu bauen

```bat
python woordlistz/build_wordlists.py            :: alle 14 Sprachen
python woordlistz/build_wordlists.py de en      :: nur einzelne
```

Das Skript lädt die Quellen beim ersten Lauf herunter (zusammen ~280 MB) und
legt sie im Temp-Ordner ab – nicht im Projekt. Danach:

```bat
node web/tools/build-wordlists.js               :: Web-Version nachziehen
```

Damit entstehen `web/js/games/wordle_words/<code>.js`; die Web-Version lädt
davon immer nur die Datei der gerade eingestellten Sprache nach.

## Wer die Listen liest

* **Desktop**: `games/wordle_words.py` (`words_for(lang)` / `allowed_for(lang)`).
  In der `.exe` liegt dieser Ordner dank `--add-data "woordlistz;woordlistz"`
  mit drin – fehlt er, greift eine kurze eingebaute Notfallliste.
* **Web**: `web/js/games/wordle_words.js` mit den erzeugten Sprachdateien.
