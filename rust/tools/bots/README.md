# Bots für den Pixelvergleich

Jeder Bot spielt **im Python-Lauf** von `rust/tools/vergleich.py` mit (er ersetzt
`vergleich.ereignis`), entscheidet aus dem Spielzustand und schreibt seine
Eingaben als Ereignisliste. Diese Liste spielt danach der echte Vergleich
gegen Rust ab:

    .venv\Scripts\python.exe rust\tools\bots\bot_pacman.py <ordner> 4000 pm_keys.txt Return
    .venv\Scripts\python.exe rust\tools\vergleich.py pacman --frames 4000 --keys-file <ordner>\pm_keys.txt

Aufruf aus dem pygamez-Wurzelordner. Die Datenordner landen im angegebenen
Ordner (nie die echten Spielstände).
