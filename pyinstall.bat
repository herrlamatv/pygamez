@echo off
REM ===================================================================
REM  Baut eine eigenstaendige PyGameZ.exe mit PyInstaller.
REM
REM  - nutzt die lokale .venv (wird bei Bedarf wie in start.bat erstellt)
REM  - installiert PyInstaller (und Pillow fuer das Icon) automatisch nach
REM  - packt alle Daten mit ein: lang\, lamawiki\*.json, games\levels\, logo\
REM  - Ergebnis: builds\PyGameZ.exe  (alles in EINER Datei)
REM
REM  Die .exe laeuft ohne installiertes Python. Einstellungen und
REM  Highscores (settings.json, mem.json, mem-ngb.json) legt sie beim
REM  Spielen neben der .exe an.
REM
REM  Hinweis: KEIN "chcp 65001" und nur ASCII-Zeichen in dieser Datei -
REM  siehe start.bat.
REM ===================================================================
setlocal
cd /d "%~dp0"

REM --- Python finden: lokale .venv -> sonst anlegen ------------------
if exist ".venv\Scripts\python.exe" (
    set "PYEXE=.venv\Scripts\python.exe"
    goto have_python
)

set "BOOT="
py -3 --version >nul 2>&1
if not errorlevel 1 set "BOOT=py -3"
if not defined BOOT (
    python --version >nul 2>&1
    if not errorlevel 1 set "BOOT=python"
)
if not defined BOOT goto no_python

echo Keine .venv gefunden - erstelle sie und installiere Abhaengigkeiten...
%BOOT% -m venv .venv
if not exist ".venv\Scripts\python.exe" goto no_venv
set "PYEXE=.venv\Scripts\python.exe"
"%PYEXE%" -m pip install --upgrade pip
"%PYEXE%" -m pip install -r requirements.txt

:have_python
REM --- Abhaengigkeiten sicherstellen ---------------------------------
"%PYEXE%" -c "import pygame" >nul 2>&1
if errorlevel 1 (
    echo pygame fehlt - installiere Abhaengigkeiten...
    "%PYEXE%" -m pip install -r requirements.txt
)

"%PYEXE%" -c "import PyInstaller" >nul 2>&1
if errorlevel 1 (
    echo PyInstaller fehlt - installiere es...
    "%PYEXE%" -m pip install pyinstaller
)
"%PYEXE%" -c "import PyInstaller" >nul 2>&1
if errorlevel 1 goto no_pyinstaller

REM --- Icon aus dem Logo erzeugen (optional, braucht Pillow) ---------
if not exist "build" mkdir build
set "ICON_OPT="
"%PYEXE%" -c "import PIL" >nul 2>&1
if errorlevel 1 "%PYEXE%" -m pip install pillow >nul 2>&1
"%PYEXE%" -c "from PIL import Image; Image.open(r'logo\pygamez2-512.png').save(r'build\pygamez.ico', sizes=[(256,256),(128,128),(64,64),(48,48),(32,32),(16,16)])" >nul 2>&1
if exist "build\pygamez.ico" set "ICON_OPT=--icon "%CD%\build\pygamez.ico""
if not defined ICON_OPT echo Kein Icon erzeugt (Pillow fehlt) - baue ohne eigenes Icon.

REM --- Build ---------------------------------------------------------
REM Absolute Pfade, weil --specpath sonst relative --add-data-Angaben
REM relativ zur .spec-Datei aufloesen wuerde.
echo.
echo Baue PyGameZ.exe - das kann einige Minuten dauern...
"%PYEXE%" -m PyInstaller --noconfirm --clean --onefile --windowed ^
    --name PyGameZ ^
    --distpath "%CD%\builds" ^
    --workpath "%CD%\build\work" ^
    --specpath "%CD%\build" ^
    %ICON_OPT% ^
    --add-data "%CD%\lang;lang" ^
    --add-data "%CD%\lamawiki\*.json;lamawiki" ^
    --add-data "%CD%\games\levels;games\levels" ^
    --add-data "%CD%\logo;logo" ^
    "%CD%\main.py"
if errorlevel 1 goto build_failed
if not exist "builds\PyGameZ.exe" goto build_failed

REM --- Aufraeumen: temporaere Build-Dateien entfernen ----------------
rmdir /s /q build >nul 2>&1

echo.
echo ===================================================================
echo  Fertig: builds\PyGameZ.exe
echo  Die Datei enthaelt alles (Python, pygame, Spiele, Sprachen, Wiki)
echo  und laeuft ohne installiertes Python. settings.json und mem.json
echo  werden beim Spielen neben der .exe angelegt.
echo ===================================================================
echo.
pause
goto :eof

:no_python
echo.
echo Es wurde kein Python gefunden.
echo Bitte Python installieren (Add python.exe to PATH anhaken):
echo     https://www.python.org/downloads/
echo Oder einmalig install-python.bat ausfuehren.
echo.
pause
goto :eof

:no_venv
echo.
echo Konnte keine .venv erstellen - bitte install-python.bat ausfuehren.
echo.
pause
goto :eof

:no_pyinstaller
echo.
echo PyInstaller konnte nicht installiert werden (Internetverbindung?).
echo Manuell versuchen:  .venv\Scripts\python -m pip install pyinstaller
echo.
pause
goto :eof

:build_failed
echo.
echo Der Build ist fehlgeschlagen - Meldungen oben beachten.
echo Der build\-Ordner bleibt fuer die Fehlersuche erhalten.
echo.
pause
goto :eof
