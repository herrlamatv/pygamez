@echo off
REM ===================================================================
REM  Startet die Python-Spielesammlung.
REM  Nutzt die virtuelle Umgebung .venv, falls vorhanden - sonst das
REM  System-Python. Zum Einrichten vorher install-python.bat ausfuehren.
REM ===================================================================
setlocal
cd /d "%~dp0"

if exist ".venv\Scripts\python.exe" (
    ".venv\Scripts\python.exe" main.py %*
) else (
    echo Keine .venv gefunden - verwende System-Python.
    echo (Tipp: einmalig install-python.bat ausfuehren.)
    python main.py %*
)

REM Bei Fehler das Fenster offen halten, damit man die Meldung sieht.
if errorlevel 1 pause
endlocal
