# -*- coding: utf-8 -*-
"""
filepick.py
===========
Die Datei-Dialoge des Spiels ("Exportieren als ...", "Importieren").

Gebraucht werden sie beim Teilen eigener Minigolf-Bahnen (siehe ``ugc.py``).
Das Fenster gehört ohnehin Tkinter - pygame zeichnet nur eingebettet hinein
(siehe den Kopf von ``main.py``) -, also stehen die Standard-Dialoge von
``tkinter.filedialog`` ohne Umwege zur Verfügung.

Alles ist gegen Fehler abgesichert: läuft kein Tk (Headless-Audit, kaputte
Anzeige), liefern die Funktionen einfach ``None`` bzw. den Downloads-Ordner,
statt das Spiel abzuschießen.

Wichtig: Ein Dialog ist modal - solange er offen ist, läuft die Spielschleife
nicht weiter. Das ist gewollt und fällt nicht auf, weil er nur aus einem
Menü heraus geöffnet wird, nie mitten im Spiel.
"""

import os


def _root():
    """Das laufende Tk-Hauptfenster (oder None, wenn es keins gibt)."""
    try:
        import tkinter
        return tkinter._default_root
    except Exception:
        return None


def available():
    """True, wenn Datei-Dialoge benutzt werden können."""
    return _root() is not None


def _types(exts):
    """Baut die filetypes-Liste für die Dialoge."""
    names = {".lamapgzmap": "PyGameZ Minigolf-Map", ".json": "JSON"}
    out = [(names.get(e, e.lstrip(".").upper()), "*" + e) for e in exts]
    out.append(("Alle Dateien", "*.*"))
    return out


def save_as(initialfile, title="", exts=(".lamapgzmap", ".json")):
    """Öffnet "Speichern unter ..." und liefert den Pfad (oder None).

    'initialfile' ist der vorgeschlagene Dateiname - beim Teilen einer Bahn
    also ihre id samt Endung, so wie es der Teilen-Dialog anzeigt.
    """
    root = _root()
    if root is None:
        return None
    try:
        from tkinter import filedialog
        path = filedialog.asksaveasfilename(
            parent=root, title=title or "Speichern unter",
            initialfile=initialfile, defaultextension=exts[0],
            filetypes=_types(exts))
    except Exception:
        return None
    return path or None


def open_file(title="", exts=(".lamapgzmap", ".json")):
    """Öffnet "Datei öffnen" und liefert den Pfad (oder None)."""
    root = _root()
    if root is None:
        return None
    try:
        from tkinter import filedialog
        path = filedialog.askopenfilename(
            parent=root, title=title or "Öffnen", filetypes=_types(exts))
    except Exception:
        return None
    return path or None


def downloads_dir():
    """Der Downloads-Ordner des Benutzers (notfalls sein Heimatverzeichnis).

    Wird angelegt, falls er fehlt. Unter Windows heißt er auch in einer
    deutschen Installation "Downloads" - der angezeigte Name "Downloads"
    ist nur eine Beschriftung, der Ordner selbst ist englisch benannt.
    """
    home = os.path.expanduser("~")
    path = os.path.join(home, "Downloads")
    if os.path.isdir(path):
        return path
    try:
        os.makedirs(path, exist_ok=True)
        return path
    except OSError:
        return home


def to_downloads(filename):
    """Vollständiger Pfad für 'filename' im Downloads-Ordner.

    Ist dort schon eine Datei desselben Namens, wird " (2)", " (3)" ...
    eingeschoben - eine bestehende Datei wird also nie überschrieben.
    """
    folder = downloads_dir()
    base, ext = os.path.splitext(filename)
    path = os.path.join(folder, filename)
    n = 2
    while os.path.exists(path) and n < 1000:
        path = os.path.join(folder, "%s (%d)%s" % (base, n, ext))
        n += 1
    return path


def short(path, keep=44):
    """Kürzt einen Pfad für die Anzeige ("...\\Downloads\\bahn.lamapgzmap")."""
    if not path:
        return ""
    if len(path) <= keep:
        return path
    parts = path.replace("/", os.sep).split(os.sep)
    tail = os.sep.join(parts[-2:]) if len(parts) >= 2 else parts[-1]
    return "..." + os.sep + tail if len(tail) < len(path) else path[-keep:]
