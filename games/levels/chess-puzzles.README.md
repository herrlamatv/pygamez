# Schachrätsel – Quelle und Lizenz

`chess-puzzles.json` (und die Web-Kopie `web/js/games/chess_puzzles.js`) enthält
200 Rätsel aus der **Lichess-Rätseldatenbank**:

- Quelle: https://database.lichess.org/#puzzles (`lichess_db_puzzle.csv.zst`)
- Lizenz: **CC0 1.0** – gemeinfrei, Nutzung ohne Einschränkung erlaubt.
  Wir nennen Lichess trotzdem gern als Quelle – danke an lichess.org!

Erzeugt mit `devtools/build_chess_puzzles.py`: fünf Stufen (Matt in 1/2/3,
Taktik I/II) mit je 40 beliebten Rätseln, nach Wertung sortiert. Jedes Rätsel
wurde mit der eigenen Engine (`games/chess_engine.py`) geprüft: alle Züge legal,
Matt-Aufgaben enden mit Matt und das Matt in N ist erzwungen.

Format je Rätsel: `id` (Lichess-Rätsel-ID), `fen` (Stellung **vor** dem Zug des
Gegners), `moves` (UCI; der erste Zug gehört dem Gegner, danach abwechselnd
Lösung und Antwort), `rating`, `theme` (Motiv).
