@echo off
REM ===================================================================
REM  Baut eine mit PyArmor geschuetzte PyGameZ-latest.exe
REM  (PyInstaller-Onefile) fuer den Ordner lastest-build\.
REM
REM  Wie pyinstall.bat, aber der Python-Code wird vor dem Packen mit
REM  PyArmor verschluesselt/obfuskiert, damit sich die .exe nicht mehr
REM  einfach dekompilieren laesst (pyinstxtractor + Decompiler liefern
REM  dann keinen lesbaren Quelltext mehr).
REM
REM  Ablauf:
REM   1. .venv + pygame + PyInstaller + PyArmor sicherstellen
REM   2. Icon aus dem Logo erzeugen (Pillow, optional)
REM   3. Spec-Datei mit allen PyInstaller-Optionen erzeugen
REM   4. pyarmor gen --pack: verschluesselt die Skripte und baut die .exe
REM   5. Ergebnis nach lastest-build\PyGameZ-latest.exe verschieben -
REM      eine dort vorhandene Datei wird ersetzt - und aufraeumen
REM
REM  Das Skript liegt in pyinstall-dev\ und wechselt beim Start selbst
REM  ins Projekt-Root (eine Ebene darueber) - einfach doppelklicken.
REM
REM  WICHTIG: Die kostenlose PyArmor-Trial kann keine grossen Skripte
REM  verarbeiten. Solche Dateien - z.B. snake.py und main.py - werden
REM  automatisch uebersprungen und bleiben unverschluesselt; das Skript
REM  listet sie am Ende auf. Mit einer PyArmor-Lizenz wird alles
REM  geschuetzt:  .venv\Scripts\pyarmor reg pyarmor-regfile-XXXX.zip
REM
REM  Hinweis: KEIN "chcp 65001" und nur ASCII-Zeichen in dieser Datei -
REM  siehe start.bat.
REM ===================================================================
setlocal enabledelayedexpansion
REM --- Ins Projekt-Root wechseln -------------------------------------
REM Dieses Skript liegt in pyinstall-dev\, gebaut wird aber eine Ebene
REM darueber (dort liegen main.py, .venv, lang\, games\ usw.).
cd /d "%~dp0.."
if not exist "main.py" goto no_project

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

"%PYEXE%" -c "import pyarmor" >nul 2>&1
if errorlevel 1 (
    echo PyArmor fehlt - installiere es...
    "%PYEXE%" -m pip install pyarmor
)
"%PYEXE%" -c "import pyarmor" >nul 2>&1
if errorlevel 1 goto no_pyarmor

REM --- Icon aus dem Logo erzeugen (optional, braucht Pillow) ---------
REM Der Dateiname steht NICHT hier drin: logo.py entscheidet ueber
REM LOGO_NUMBER, welches Logo aktuell ist (PNG bevorzugt, sonst JPG).
REM So bleibt das Icon richtig, wenn ein neues Logo eingebaut wird.
if not exist "build" mkdir build
set "ICON_OPT="
"%PYEXE%" -c "import PIL" >nul 2>&1
if errorlevel 1 "%PYEXE%" -m pip install pillow >nul 2>&1
"%PYEXE%" -c "import logo; from PIL import Image; Image.open(logo.find_logo(512)[0]).convert('RGBA').save(r'build\pygamez.ico', sizes=[(256,256),(128,128),(64,64),(48,48),(32,32),(16,16)])" >nul 2>&1
if exist "build\pygamez.ico" set "ICON_OPT=--icon "%CD%\build\pygamez.ico""
if not defined ICON_OPT echo Kein Icon erzeugt - baue ohne eigenes Icon.

REM --- Start-Wrapper erzeugen ----------------------------------------
REM PyArmor verlangt, dass das STARTskript verschluesselt ist - main.py
REM ist dafuer in der Trial-Version zu gross. Deshalb startet die .exe
REM ueber diesen winzigen Wrapper (immer verschluesselbar), der dasselbe
REM tut wie der __main__-Block von main.py.
>  build\pygamez_boot.py echo # Auto-generiert von pyinstall-pyarmor.bat - Start-Wrapper fuer main.py
>> build\pygamez_boot.py echo import main
>> build\pygamez_boot.py echo.
>> build\pygamez_boot.py echo if main._check_dependencies():
>> build\pygamez_boot.py echo     main.App().run()
>> build\pygamez_boot.py echo else:
>> build\pygamez_boot.py echo     raise SystemExit(1)

REM --- Spec-Datei mit allen PyInstaller-Optionen erzeugen ------------
REM Absolute Pfade, weil die Spec in build\ liegt und relative Angaben
REM sonst relativ zur .spec-Datei aufgeloest wuerden. --paths sorgt
REM dafuer, dass der Wrapper main.py und alle Module im Projekt findet.
"%PYEXE%" -m PyInstaller.utils.cliutils.makespec --onefile --windowed ^
    --name PyGameZ-latest ^
    --specpath "%CD%\build" ^
    --paths "%CD%" ^
    %ICON_OPT% ^
    --add-data "%CD%\lang;lang" ^
    --add-data "%CD%\lamawiki\*.json;lamawiki" ^
    --add-data "%CD%\lamawiki\lang.expansion\*.json;lamawiki\lang.expansion" ^
    --add-data "%CD%\games\levels;games\levels" ^
    --add-data "%CD%\logo;logo" ^
    "%CD%\build\pygamez_boot.py"
if not exist "build\PyGameZ-latest.spec" goto spec_failed

REM --- Trial-Version erkennen ----------------------------------------
set "TRIAL=1"
"%PYEXE%" -m pyarmor.cli --version 2>nul | findstr /i "trial" >nul 2>&1
if errorlevel 1 set "TRIAL="

REM --- Skript-Listen aufbauen ----------------------------------------
REM In der Trial-Version wird jede Datei einzeln probe-verschluesselt;
REM zu grosse Dateien werden uebersprungen (bleiben unverschluesselt).
REM Der Start-Wrapper ist winzig und wird immer verschluesselt.
set "SRCS=build\pygamez_boot.py"
set "EXCLUDES="
set "SKIPPED="
if defined TRIAL echo Trial-Version erkannt - pruefe, welche Dateien verschluesselbar sind...

for %%f in (*.py) do (
    if defined TRIAL (
        "%PYEXE%" -m pyarmor.cli gen -O build\probe "%%f" >nul 2>&1
        if errorlevel 1 (
            set "SKIPPED=!SKIPPED! %%f"
        ) else (
            set "SRCS=!SRCS! %%f"
        )
    ) else (
        set "SRCS=!SRCS! %%f"
    )
)

for %%f in (games\*.py lamawiki\*.py) do (
    if defined TRIAL (
        "%PYEXE%" -m pyarmor.cli gen -O build\probe "%%f" >nul 2>&1
        if errorlevel 1 (
            set "p=%%f"
            set "EXCLUDES=!EXCLUDES! --exclude !p:\=/!"
            set "SKIPPED=!SKIPPED! %%f"
        )
    )
)

if defined SKIPPED (
    echo.
    echo ACHTUNG - Trial-Limit: diese Dateien sind zu gross und bleiben
    echo unverschluesselt:
    echo  !SKIPPED!
    echo Mit PyArmor-Lizenz wird alles geschuetzt - siehe Kopf dieser Datei.
    echo.
)

REM --- Verschluesseln + Packen ---------------------------------------
echo Verschluessele Code und baue PyGameZ-latest.exe - das kann einige Minuten dauern...
"%PYEXE%" -m pyarmor.cli gen --pack build\PyGameZ-latest.spec -r !EXCLUDES! !SRCS! games lamawiki
if errorlevel 1 goto build_failed
if not exist "dist\PyGameZ-latest.exe" goto build_failed

REM --- Ergebnis nach lastest-build\ verschieben, aufraeumen -----------
REM Eine dort vorhandene PyGameZ-latest.exe wird ersetzt. Laeuft sie noch,
REM laesst sie sich nicht loeschen - dann bricht das Skript mit Hinweis ab,
REM statt den frischen Build in dist\ liegen zu lassen.
if not exist "lastest-build" mkdir lastest-build
if exist "lastest-build\PyGameZ-latest.exe" del /f /q "lastest-build\PyGameZ-latest.exe" >nul 2>&1
if exist "lastest-build\PyGameZ-latest.exe" goto exe_in_use
move /y "dist\PyGameZ-latest.exe" "lastest-build\PyGameZ-latest.exe" >nul
if errorlevel 1 goto exe_in_use
rmdir /s /q dist >nul 2>&1
rmdir /s /q build >nul 2>&1
if exist ".pyarmor" rmdir /s /q .pyarmor >nul 2>&1

echo.
echo ===================================================================
echo  Fertig: lastest-build\PyGameZ-latest.exe  - mit PyArmor geschuetzt
if defined SKIPPED (
    echo.
    echo  Hinweis Trial-Version: NICHT verschluesselt wurden:
    echo   !SKIPPED!
    echo  Alles andere ist verschluesselt. Fuer vollen Schutz:
    echo  PyArmor-Lizenz kaufen und registrieren, dann neu bauen.
)
echo ===================================================================
echo.
pause
exit /b 0

:no_project
echo.
echo Projektdateien nicht gefunden (main.py fehlt in "%CD%").
echo Dieses Skript muss im Ordner pyinstall-dev\ direkt im PyGameZ-
echo Projektordner liegen und von dort gestartet werden.
echo.
pause
exit /b 1

:no_python
echo.
echo Es wurde kein Python gefunden.
echo Bitte Python installieren (Add python.exe to PATH anhaken):
echo     https://www.python.org/downloads/
echo Oder einmalig install-python.bat ausfuehren.
echo.
pause
exit /b 1

:no_venv
echo.
echo Konnte keine .venv erstellen - bitte install-python.bat ausfuehren.
echo.
pause
exit /b 1

:no_pyinstaller
echo.
echo PyInstaller konnte nicht installiert werden (Internetverbindung?).
echo Manuell versuchen:  .venv\Scripts\python -m pip install pyinstaller
echo.
pause
exit /b 1

:no_pyarmor
echo.
echo PyArmor konnte nicht installiert werden (Internetverbindung?).
echo Manuell versuchen:  .venv\Scripts\python -m pip install pyarmor
echo.
pause
exit /b 1

:spec_failed
echo.
echo Konnte die Spec-Datei nicht erzeugen - Meldungen oben beachten.
echo.
pause
exit /b 1

:build_failed
echo.
echo Der Build ist fehlgeschlagen - Meldungen oben beachten.
echo Der build\-Ordner bleibt fuer die Fehlersuche erhalten.
echo.
pause
exit /b 1

:exe_in_use
echo.
echo Die alte lastest-build\PyGameZ-latest.exe liess sich nicht ersetzen.
echo Laeuft sie noch? Bitte schliessen und das Skript neu starten.
echo Der frische Build liegt solange in dist\PyGameZ-latest.exe.
echo.
pause
exit /b 1