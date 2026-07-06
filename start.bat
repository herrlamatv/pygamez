@echo off
REM ===================================================================
REM  Startet die Python-Spielesammlung.
REM  Reihenfolge: lokale .venv  ->  sonst System-Python (py / python).
REM  Fehlt pygame, wird es automatisch nachinstalliert.
REM
REM  Hinweis: KEIN "chcp 65001" und nur ASCII-Zeichen in dieser Datei -
REM  chcp 65001 zusammen mit Umlauten zerlegt sonst den Batch-Parser
REM  (erstes Zeichen jeder Folgezeile geht verloren).
REM ===================================================================
setlocal
cd /d "%~dp0"

REM --- Fall 1: lokale .venv vorhanden -------------------------------
if exist ".venv\Scripts\python.exe" (
    set "PYEXE=.venv\Scripts\python.exe"
    goto run
)

REM --- Fall 2: keine .venv - Boot-Python suchen ---------------------
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
if not exist ".venv\Scripts\python.exe" goto use_boot

set "PYEXE=.venv\Scripts\python.exe"
".venv\Scripts\python.exe" -m pip install --upgrade pip
".venv\Scripts\python.exe" -m pip install -r requirements.txt
goto run

:use_boot
echo Konnte keine .venv erstellen - verwende System-Python direkt.
set "PYEXE=%BOOT%"

:run
REM --- Sicherstellen, dass pygame vorhanden ist ---------------------
%PYEXE% -c "import pygame" >nul 2>&1
if not errorlevel 1 goto launch
echo pygame fehlt - installiere es nach...
%PYEXE% -m pip install -r requirements.txt

:launch
%PYEXE% main.py %*
if errorlevel 1 pause
goto :eof

:no_python
echo.
echo Es wurde kein Python gefunden.
echo Bitte Python installieren (Add python.exe to PATH anhaken):
echo     https://www.python.org/downloads/
echo Oder einmalig install-python.bat ausfuehren.
echo.
pause
