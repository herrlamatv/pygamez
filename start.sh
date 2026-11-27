#!/usr/bin/env bash
# ===================================================================
#  Startet die Python-Spielesammlung (macOS / Linux / Git Bash).
#  Reihenfolge: lokale .venv  ->  sonst ein passendes Python suchen,
#  .venv anlegen und pygame-ce aus requirements.txt installieren.
#  Fehlt pygame in der .venv, wird es automatisch nachinstalliert.
#
#  macOS: einfach start.command im Finder doppelklicken - das ruft
#  dieses Skript auf.
#
#  Hinweis: nur ASCII-Zeichen und LF-Zeilenenden in dieser Datei.
#  Laeuft auch mit dem alten bash 3.2, das macOS mitbringt.
# ===================================================================
cd "$(dirname "$0")" || exit 1

OS="$(uname -s 2>/dev/null || echo unknown)"
case "$OS" in
    MINGW*|MSYS*|CYGWIN*) WIN=1 ;;
    *) WIN=0 ;;
esac

# --- Hilfsfunktionen -----------------------------------------------

# Ein Python taugt, wenn es mindestens 3.9 ist und Tkinter mit Tk 8.6+
# mitbringt. Apples altes /usr/bin/python3 hat nur Tk 8.5, Homebrew-
# Python ohne "python-tk" gar kein Tkinter. Es oeffnet sich dabei kein
# Fenster (TkVersion steht fest, Tk wird nicht gestartet).
py_ok() {
    "$1" -c 'import sys, tkinter; sys.exit(0 if sys.version_info >= (3, 9) and tkinter.TkVersion >= 8.6 else 1)' >/dev/null 2>&1
}

py_new_enough() {
    "$1" -c 'import sys; sys.exit(0 if sys.version_info >= (3, 9) else 1)' >/dev/null 2>&1
}

# Python der .venv (je nach System liegt es woanders).
venv_python() {
    PY=""
    if [ "$WIN" = 1 ]; then
        [ -x ".venv/Scripts/python.exe" ] && PY=".venv/Scripts/python.exe"
    else
        [ -x ".venv/bin/python" ] && PY=".venv/bin/python"
    fi
}

BOOT=""
NOTK=""

try_python() {
    p="$1"
    [ -n "$p" ] && [ -x "$p" ] || return 1
    # Ohne Xcode-Kommandozeilenwerkzeuge ist /usr/bin/python3 auf dem Mac
    # nur ein Platzhalter, der einen Installationsdialog aufpoppen laesst.
    if [ "$OS" = "Darwin" ] && [ "$p" = "/usr/bin/python3" ] \
            && ! xcode-select -p >/dev/null 2>&1; then
        return 1
    fi
    if py_ok "$p"; then
        BOOT="$p"
        return 0
    fi
    if [ -z "$NOTK" ] && py_new_enough "$p"; then
        NOTK="$p"
    fi
    return 1
}

find_python() {
    if [ "$OS" = "Darwin" ]; then
        # python.org-Installer zuerst (bringt Tk 8.6 mit), dann Homebrew.
        for p in /Library/Frameworks/Python.framework/Versions/Current/bin/python3 \
                 /opt/homebrew/bin/python3 \
                 /usr/local/bin/python3; do
            try_python "$p" && return 0
        done
    fi
    for n in python3.14 python3.13 python3.12 python3.11 python3.10 python3.9 \
             python3 python; do
        p="$(command -v "$n" 2>/dev/null)" || continue
        try_python "$p" && return 0
    done
    return 1
}

no_python() {
    echo
    echo "Es wurde kein passendes Python gefunden."
    echo "Gebraucht wird Python 3.9 oder neuer mit Tkinter (Tk 8.6 oder neuer)."
    if [ -n "$NOTK" ]; then
        echo "Gefunden, aber ohne brauchbares Tkinter: $NOTK"
    fi
    echo
    case "$OS" in
        Darwin)
            echo "macOS: Python 3.13 von https://www.python.org/downloads/macos/"
            echo "       installieren (der Installer bringt Tkinter mit) und dann"
            echo "       start.command noch einmal starten."
            echo "       Mit Homebrew geht auch:  brew install python python-tk"
            ;;
        Linux)
            echo "Linux (Debian/Ubuntu):  sudo apt install python3 python3-venv python3-tk"
            echo "Linux (Fedora):         sudo dnf install python3 python3-tkinter"
            echo "Linux (Arch):           sudo pacman -S python tk"
            ;;
        *)
            echo "Windows: start.bat benutzen oder einmalig install-python.bat ausfuehren."
            ;;
    esac
    echo
    exit 1
}

venv_fail() {
    echo
    echo "Konnte die virtuelle Umgebung .venv nicht anlegen (mit $BOOT)."
    if [ "$OS" = "Linux" ]; then
        echo "Debian/Ubuntu: sudo apt install python3-venv  - danach erneut starten."
    fi
    echo "Manuell:  $BOOT -m venv .venv"
    echo
    exit 1
}

pip_fail() {
    echo
    echo "pygame konnte nicht installiert werden (Internetverbindung?)."
    echo "Manuell versuchen:  $PY -m pip install -r requirements.txt"
    echo
    exit 1
}

# --- Fall 1: lokale .venv vorhanden --------------------------------
venv_python

# --- Fall 2: keine .venv - Python suchen und .venv anlegen ----------
if [ -z "$PY" ]; then
    find_python || no_python
    echo "Keine .venv gefunden - erstelle sie mit $BOOT"
    echo "und installiere die Abhaengigkeiten (einmalig, dauert etwas)..."
    "$BOOT" -m venv .venv || venv_fail
    venv_python
    [ -n "$PY" ] || venv_fail
    "$PY" -m pip install --upgrade pip || echo "pip-Update fehlgeschlagen - mache trotzdem weiter."
    "$PY" -m pip install -r requirements.txt || pip_fail
fi

# --- Tkinter pruefen (eine alte .venv kann ohne Tk gebaut sein) -----
if ! py_ok "$PY"; then
    echo
    echo "Das Python in .venv hat kein brauchbares Tkinter (Tk 8.6 oder neuer)."
    echo "Bitte den Ordner .venv loeschen und dieses Skript erneut starten -"
    echo "es sucht dann ein passendes Python und legt .venv neu an."
    if [ "$OS" = "Darwin" ]; then
        echo "Am einfachsten vorher Python 3.13 von https://www.python.org/downloads/macos/"
        echo "installieren (oder: brew install python-tk)."
    fi
    echo
    exit 1
fi

# --- Sicherstellen, dass pygame vorhanden ist -----------------------
if ! "$PY" -c "import pygame" >/dev/null 2>&1; then
    echo "pygame fehlt - installiere es nach..."
    "$PY" -m pip install -r requirements.txt || pip_fail
fi

exec "$PY" main.py "$@"
