@echo off
REM ===================================================================
REM  Richtet die Python-Spielesammlung ein:
REM   1. Installiert Python 3.13 (ueber winget), falls nicht vorhanden.
REM   2. Erstellt die virtuelle Umgebung .venv.
REM   3. Installiert die Abhaengigkeiten (pygame) aus requirements.txt.
REM  Danach die Sammlung mit start.bat starten.
REM
REM  Hinweis: KEIN "chcp 65001" und nur ASCII-Zeichen - sonst zerlegt
REM  der Batch-Parser die Datei (erstes Zeichen jeder Zeile geht weg).
REM ===================================================================
setlocal
cd /d "%~dp0"

set "PYVER=3.13"

echo ============================================
echo  Python-Spielesammlung - Einrichtung
echo ============================================
echo.

REM --- 1. Python vorhanden? -----------------------------------------
py -%PYVER% --version >nul 2>&1
if not errorlevel 1 goto have_python

echo Python %PYVER% wurde nicht gefunden - versuche Installation ueber winget...
where winget >nul 2>&1
if errorlevel 1 goto no_winget

winget install --id Python.Python.3.13 -e --accept-package-agreements --accept-source-agreements

REM winget aktualisiert PATH erst in einem neuen Fenster.
py -%PYVER% --version >nul 2>&1
if not errorlevel 1 goto have_python
echo.
echo Python wurde installiert, ist in DIESEM Fenster aber noch nicht verfuegbar.
echo Bitte ein NEUES Terminal oeffnen und install-python.bat erneut ausfuehren.
pause
exit /b 1

:no_winget
echo.
echo winget ist nicht verfuegbar. Bitte Python %PYVER% manuell installieren:
echo     https://www.python.org/downloads/
echo Wichtig: bei der Installation "Add python.exe to PATH" anhaken.
echo Danach dieses Skript erneut ausfuehren.
pause
exit /b 1

:have_python
echo Gefundene Python-Version:
py -%PYVER% --version
echo.

REM --- 2. Virtuelle Umgebung ----------------------------------------
echo === Erstelle virtuelle Umgebung .venv ===
py -%PYVER% -m venv .venv
if errorlevel 1 (
    echo Fehler beim Erstellen der virtuellen Umgebung.
    pause
    exit /b 1
)

REM --- 3. Abhaengigkeiten -------------------------------------------
echo === Installiere Abhaengigkeiten (pygame) ===
".venv\Scripts\python.exe" -m pip install --upgrade pip
".venv\Scripts\python.exe" -m pip install -r requirements.txt
if errorlevel 1 (
    echo Fehler beim Installieren der Abhaengigkeiten.
    pause
    exit /b 1
)

echo.
echo ============================================
echo  Fertig! Starte das Spiel mit:  start.bat
echo ============================================
pause
