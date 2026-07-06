#!/usr/bin/env bash
# ===================================================================
#  Startet die Python-Spielesammlung (Linux / macOS / Git Bash).
#  Nutzt die virtuelle Umgebung .venv, falls vorhanden - sonst das
#  System-Python (python3).
# ===================================================================
set -e
cd "$(dirname "$0")"

if [ -x ".venv/bin/python" ]; then
    # Linux / macOS venv
    exec .venv/bin/python main.py "$@"
elif [ -x ".venv/Scripts/python.exe" ]; then
    # Windows-venv unter Git Bash
    exec .venv/Scripts/python.exe main.py "$@"
elif command -v python3 >/dev/null 2>&1; then
    echo "Keine .venv gefunden - verwende System-python3."
    exec python3 main.py "$@"
else
    echo "Keine .venv gefunden - verwende System-python."
    exec python main.py "$@"
fi
