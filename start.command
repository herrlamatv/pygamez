#!/bin/bash
# ===================================================================
#  macOS: Doppelklick im Finder startet PyGameZ im Terminal.
#  Ruft start.sh auf - das richtet beim ersten Start .venv und
#  pygame-ce selbst ein.
#
#  Blockiert macOS das Oeffnen ("nicht verifizierter Entwickler"):
#  Systemeinstellungen -> Datenschutz & Sicherheit -> "Trotzdem oeffnen".
#
#  Hinweis: nur ASCII-Zeichen und LF-Zeilenenden in dieser Datei.
# ===================================================================
cd "$(dirname "$0")" || exit 1

# "bash ./start.sh" statt "./start.sh": klappt auch, wenn start.sh ihr
# Ausfuehrungsrecht verloren hat (z. B. nach dem Entpacken einer ZIP).
bash ./start.sh "$@"
status=$?

if [ "$status" -ne 0 ]; then
    echo
    echo "PyGameZ wurde mit Fehlercode $status beendet - Meldungen oben beachten."
    read -r -p "Enter druecken zum Schliessen..." _
fi
exit "$status"
