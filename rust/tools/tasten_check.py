# -*- coding: utf-8 -*-
"""
tasten_check.py
===============
Prueft die Tastatur im ECHTEN Fenster gegen Tk - der Pruefstand reicht
Tastennamen direkt an die App und umgeht dabei ``input.rs`` (Windows-Taste ->
Tk-keysym + Zeichen). Hier gehen dieselben Windows-Nachrichten (WM_KEYDOWN /
WM_KEYUP per PostMessage, Zeichen wie beim Tippen ueber TranslateMessage)

* an ein Tk-Fenster (so wie es main.py benutzt: ``event.keysym`` /
  ``event.char``) - das ist die Vorlage,
* an ``pygamez.exe`` mit ``PYGAMEZ_TASTEN_PROBE``: ein verstecktes Fenster
  mit der Tastatur-Fensterprozedur der App und derselben Nachrichtenschleife
  wie minifb; ``input.rs`` uebersetzt, nichts wird ausgeloest.

Beide Fenster sind unsichtbar: nichts wird an Windows "getippt", kein
Fokus geholt, echte Tastendruecke stoeren nicht. Es gilt das Tastaturlayout
des Rechners. Tasten mit Umschalt/AltGr und Tottasten (^, ´) lassen sich so
nicht nachstellen und fehlen.

    .venv/Scripts/python.exe rust/tools/tasten_check.py
"""

import ctypes
import os
import shutil
import subprocess
import sys
import tempfile
import time
from ctypes import wintypes

HIER = os.path.dirname(os.path.abspath(__file__))
WURZEL = os.path.dirname(os.path.dirname(HIER))
EXE = os.path.join(WURZEL, "rust.output", "release", "pygamez.exe")

user32 = ctypes.WinDLL("user32", use_last_error=True)
user32.PostMessageW.argtypes = [wintypes.HWND, wintypes.UINT, wintypes.WPARAM, wintypes.LPARAM]
user32.MapVirtualKeyW.argtypes = [wintypes.UINT, wintypes.UINT]
WM_KEYDOWN, WM_KEYUP, WM_CLOSE, WM_SETFOCUS = 0x100, 0x101, 0x10, 0x7

# (Name, virtueller Tastencode, erweitert?)
TASTEN = ([(chr(c), c, False) for c in range(ord("A"), ord("Z") + 1)]
          + [(chr(c), c, False) for c in range(ord("0"), ord("9") + 1)]
          + [("Leertaste", 0x20, False), ("Enter", 0x0D, False), ("Esc", 0x1B, False),
             ("Rueck", 0x08, False), ("Tab", 0x09, False),
             ("Links", 0x25, True), ("Hoch", 0x26, True), ("Rechts", 0x27, True), ("Runter", 0x28, True),
             ("BildHoch", 0x21, True), ("BildRunter", 0x22, True), ("Pos1", 0x24, True), ("Ende", 0x23, True),
             ("Einfg", 0x2D, True), ("Entf", 0x2E, True)]
          + [("F%d" % i, 0x6F + i, False) for i in (1, 2, 3, 4, 5, 6, 7, 8, 9, 12)]
          + [("Num%d" % i, 0x60 + i, False) for i in range(10)]
          + [("Num*", 0x6A, False), ("Num+", 0x6B, False), ("Num-", 0x6D, False), ("Num,", 0x6E, False),
             ("Num/", 0x6F, True), ("NumEnter", 0x0D, True)]
          # Satzzeichen-Tasten (deutsch: ue + , - . # oe ss ae <)
          + [("OEM_1", 0xBA, False), ("OEM_PLUS", 0xBB, False), ("OEM_COMMA", 0xBC, False),
             ("OEM_MINUS", 0xBD, False), ("OEM_PERIOD", 0xBE, False), ("OEM_2", 0xBF, False),
             ("OEM_3", 0xC0, False), ("OEM_4", 0xDB, False), ("OEM_7", 0xDE, False), ("OEM_102", 0xE2, False)]
          + [("ShiftL", 0xA0, False), ("ShiftR", 0xA1, False), ("StrgL", 0xA2, False), ("StrgR", 0xA3, True)])

# Fuer Umschalt/Strg nimmt Windows im WM_KEYDOWN den allgemeinen Code.
ALLGEMEIN = {0xA0: 0x10, 0xA1: 0x10, 0xA2: 0x11, 0xA3: 0x11}


def lparam(vk, ext, los):
    scan = user32.MapVirtualKeyW(vk, 0)
    if vk == 0xA1:
        scan = 0x36
    v = 1 | (scan << 16) | ((1 << 24) if ext else 0)
    if los:
        v |= (1 << 30) | (1 << 31)
    return v


def modifikatoren_frei(hoechstens=3.0):
    """Wartet, bis niemand Umschalt/Strg/Alt/Win gedrueckt haelt: Windows
    uebersetzt Zeichen mit dem ECHTEN Tastenstand (wer gerade tippt, wuerde
    sonst aus "-" ein "_" machen)."""
    ende = time.time() + hoechstens
    while time.time() < ende:
        if not any(user32.GetAsyncKeyState(vk) & 0x8000 for vk in (0x10, 0x11, 0x12, 0x5B, 0x5C)):
            return
        time.sleep(0.02)


def senden(hwnd, vk, ext, los, warten):
    user32.PostMessageW(hwnd, WM_KEYUP if los else WM_KEYDOWN, ALLGEMEIN.get(vk, vk), lparam(vk, ext, los))
    warten()


def tk_vorlage(tasten):
    """Was Tk (wie in main.py) aus den Nachrichten macht."""
    import tkinter as tk
    root = tk.Tk()
    # Zurueckgezogen: nie sichtbar, nie aktiv - echte Tasten (wer gerade am
    # Rechner tippt) koennen den Vergleich nicht stoeren.
    root.withdraw()
    out = []
    # Wie main.py (_bind_events): am Hauptfenster, nicht an "all" - sonst
    # gewinnt Tks eigene Tab-Bindung.
    root.bind("<KeyPress>", lambda e: out.append(("druck", e.keysym, ",".join("%x" % ord(c) for c in e.char))))
    root.bind("<KeyRelease>", lambda e: out.append(("los", e.keysym, "")))
    root.update()
    hwnd = root.winfo_id()
    user32.PostMessageW(hwnd, WM_SETFOCUS, 0, 0)
    root.focus_set()
    root.update()

    def warten():
        ende = time.time() + 0.02
        while time.time() < ende:
            root.update()
    ergebnis = []
    for name, vk, ext in tasten:
        # Tks innerer Fokus (nur per WM_SETFOCUS behauptet) haelt nicht immer,
        # wenn auf dem Rechner gerade gearbeitet wird: vor jeder Taste neu
        # setzen und notfalls wiederholen.
        for _versuch in range(5):
            out.clear()
            user32.PostMessageW(hwnd, WM_SETFOCUS, 0, 0)
            root.focus_set()
            warten()
            modifikatoren_frei()
            senden(hwnd, vk, ext, False, warten)
            senden(hwnd, vk, ext, True, warten)
            if out:
                break
        ergebnis.append(list(out))
    root.destroy()
    return ergebnis


def rust_ergebnis(tmp, tasten):
    """pygamez.exe mit PYGAMEZ_TASTEN_PROBE: nur ein verstecktes Fenster mit
    der Tastatur-Fensterprozedur der App (kein Spielfenster, kein Fokus).
    Rueckgabe je Taste: [(druck|los, keysym, zeichen), ...]."""
    log = os.path.join(tmp, "tasten.txt")
    for alt in (log, log + ".hwnd"):
        if os.path.exists(alt):
            os.remove(alt)
    env = dict(os.environ, PYGAMEZ_DATA_DIR=tmp, PYGAMEZ_TASTEN_PROBE=log)
    for k in ("PYGAMEZ_SHOT", "PYGAMEZ_VOLLBILD_PROBE"):
        env.pop(k, None)
    proc = subprocess.Popen([EXE], env=env)
    hwnd = None
    for _ in range(100):
        time.sleep(0.05)
        if os.path.exists(log + ".hwnd"):
            with open(log + ".hwnd", encoding="utf-8") as f:
                text = f.read().strip()
            if text:
                hwnd = int(text)
                break
    if not hwnd:
        proc.kill()
        raise SystemExit("FEHLER: verstecktes Rust-Fenster meldet sich nicht")
    # Je Taste: warten, bis niemand Modifikatoren haelt, dann Druck +
    # Loslassen (Rust verarbeitet die Nachrichten der Reihe nach).
    for name, vk, ext in tasten:
        modifikatoren_frei()
        senden(hwnd, vk, ext, False, lambda: time.sleep(0.005))
        senden(hwnd, vk, ext, True, lambda: time.sleep(0.005))
    user32.PostMessageW(hwnd, 0x8001, 0, 0)   # Ende
    proc.wait(timeout=30)
    with open(log, encoding="utf-8") as f:
        zeilen = [z.split("\t") for z in f.read().splitlines() if z.strip()]
    # Jede Taste ergibt genau Druck + Loslassen.
    return [[(z[0], z[1], z[2] if z[0] == "druck" else "") for z in zeilen[2 * i:2 * i + 2]]
            for i in range(len(tasten))]


def vergleichen(tasten, tk, rs):
    """Liste der abweichenden (Taste, Tk, Rust)."""
    falsch = []
    for (name, vk, ext), tk_ev, rs_ev in zip(tasten, tk, rs):
        tk_cmp = [tuple(e) for e in tk_ev]
        if name == "ShiftR":
            # Grenze des Tests: Tk liest links/rechts aus dem Echtzeit-
            # Tastenstand, den gepostete Nachrichten nicht setzen (Tk meldet
            # dann Shift_L). Beim echten Tippen sagen beide Shift_R.
            tk_cmp = [(a, "Shift_R", c) for a, _k, c in tk_cmp]
        if rs_ev != tk_cmp:
            falsch.append(((name, vk, ext), tk_cmp, rs_ev))
    return falsch


def main():
    if not os.path.exists(EXE):
        raise SystemExit("Erst bauen: cargo build -p pygamez --release --offline")
    tmp = tempfile.mkdtemp(prefix="pgz_tasten_")
    tasten = list(TASTEN)
    falsch = []
    try:
        # Wer gerade am Rechner tippt, kann einzelne Tasten verfaelschen (der
        # echte Tastenstand fliesst in die Zeichen ein): Abweichungen werden
        # bis zu zweimal neu geprueft - ein echter Fehler weicht jedes Mal ab.
        for runde in range(3):
            falsch = vergleichen(tasten, tk_vorlage(tasten), rust_ergebnis(tmp, tasten))
            if not falsch:
                break
            tasten = [t for t, _a, _b in falsch]
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    for (name, _vk, _e), tk_cmp, rs_ev in falsch:
        print("FEHLER %-10s Tk: %-40s Rust: %s" % (name, tk_cmp, rs_ev))
    print("Tasten geprueft: %d, Abweichungen: %d" % (len(TASTEN), len(falsch)))
    print("Tastatur:", "wie Tk" if not falsch else "%d Abweichungen" % len(falsch))
    return 1 if falsch else 0


if __name__ == "__main__":
    sys.exit(main())
