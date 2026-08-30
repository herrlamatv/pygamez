# -*- coding: utf-8 -*-
"""
logo.py
=======
Zentrale Logo-Verwaltung für PyGameZ.

Hier wird an EINER Stelle festgelegt, welches Logo verwendet wird - über die
Variable ``LOGO_NUMBER``. Der Dateiname folgt immer dem Schema

    pygamez{number}-{size}

also z.B. ``pygamez3-512``. Die passende Datei wird pro Größe automatisch
gesucht: zuerst PNG (bevorzugt), dann JPG (Alt-Format).

Warum zwei Wege? (PNG vs. JPG)
------------------------------
Für das Fenster-/Taskleisten-Icon braucht Tkinter ein ``tk.PhotoImage``:

* **PNG** kann Tk direkt laden - das ist der "richtige" Weg, ganz ohne Umweg,
  inklusive Transparenz.
* **JPG** kann Tk NICHT laden. Dafür greift der alte Umweg: pygame lädt das
  Bild, wir wandeln es in rohe PPM-Daten (P6) um und geben das an
  ``tk.PhotoImage``.

Für die Menü-Anzeige (eine ``pygame.Surface``) ist die Unterscheidung egal -
pygame lädt PNG UND JPG direkt.

Dadurch kann man Schritt für Schritt auf reine PNG-Logos umsteigen, ohne dass
noch vorhandene JPG-Dateien plötzlich nicht mehr funktionieren.
"""

import os

# --- Konfiguration ---------------------------------------------------------
# Welches Logo? Der Dateiname ergibt sich aus  pygamez{LOGO_NUMBER}-{size}.{ext}
LOGO_NUMBER = 3

# Verfügbare Kantenlängen (px), größte zuerst.
LOGO_SIZES = (512, 256, 128)

# Ordner (relativ zu diesem Modul), in dem die Logo-Dateien liegen.
LOGO_DIR = "logo"

# Bevorzugte Reihenfolge der Formate: PNG zuerst (direkter Tk-Weg), dann JPG.
LOGO_FORMATS = ("png", "jpg")

_BASE = os.path.dirname(os.path.abspath(__file__))


def logo_basename(size):
    """Dateiname OHNE Endung nach Schema  pygamez{number}-{size}."""
    return f"pygamez{LOGO_NUMBER}-{size}"


def logo_path(size, ext):
    """Vollständiger Pfad für eine Größe + Endung (muss nicht existieren)."""
    return os.path.join(_BASE, LOGO_DIR, f"{logo_basename(size)}.{ext}")


def find_logo(size):
    """Sucht die beste vorhandene Datei für eine Größe.

    Rückgabe: ``(pfad, ext)`` oder ``(None, None)``, wenn nichts gefunden wird.
    PNG wird JPG vorgezogen.
    """
    for ext in LOGO_FORMATS:
        pfad = logo_path(size, ext)
        if os.path.exists(pfad):
            return pfad, ext
    return None, None


def icon_photos(pygame, tk):
    """Erzeugt ``tk.PhotoImage``-Objekte für alle verfügbaren Logo-Größen.

    * PNG: direkt von Tk geladen (der richtige Weg, mit Transparenz).
    * JPG: über pygame nach PPM (P6) umgewandelt, weil Tk kein JPG kann.

    Es werden alle Größen übergeben - Tk/Windows wählt die passende für
    Titelleiste, Taskleiste und Alt+Tab selbst aus.

    Rückgabe: Liste von ``tk.PhotoImage`` (evtl. leer). Der Aufrufer MUSS die
    Referenzen halten, sonst räumt Tk die Bilder weg.
    """
    photos = []
    for size in LOGO_SIZES:
        pfad, ext = find_logo(size)
        if not pfad:
            continue
        try:
            if ext == "png":
                # Richtiger Weg: Tk lädt PNG direkt.
                photos.append(tk.PhotoImage(file=pfad))
            else:
                # Alter Weg: JPG über pygame -> rohe PPM-Daten -> Tk.
                img = pygame.image.load(pfad)
                w, h = img.get_size()
                data = b"P6 %d %d 255 " % (w, h) + pygame.image.tobytes(img, "RGB")
                photos.append(tk.PhotoImage(data=data, format="ppm"))
        except Exception:
            # Fehlt/beschädigt eine Größe, wird sie einfach übersprungen.
            pass
    return photos


def load_surface(pygame, prefer_sizes=None):
    """Lädt das Logo als ``pygame.Surface`` (für die Menü-Anzeige).

    pygame kann PNG UND JPG direkt laden, daher hier keine PPM-Umwandlung.
    Es wird die erste vorhandene Größe aus ``prefer_sizes`` genommen (für die
    Menü-Anzeige lohnt sich z.B. eine mittlere Größe zum Herunterskalieren).

    Rückgabe: ``pygame.Surface`` oder ``None``.
    """
    for size in (prefer_sizes or LOGO_SIZES):
        pfad, _ext = find_logo(size)
        if not pfad:
            continue
        try:
            return pygame.image.load(pfad)
        except Exception:
            pass
    return None
