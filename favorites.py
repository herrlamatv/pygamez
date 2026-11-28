# -*- coding: utf-8 -*-
"""
favorites.py
============
Lieblingsspiele (Favoriten) je Profil.

Gespeichert wird als eigener Abschnitt in mem.json (beim aktiven Profil):

    "favorites": {"games": ["snake", "tetris", ...]}

Die Liste enthält die ``highscore_key`` der Spiele in der Reihenfolge, in der
sie markiert wurden; angezeigt werden Favoriten aber immer in der normalen
Spiele-Reihenfolge. Die Web-Fassung speichert genau dasselbe Objekt unter
``pygamez.<profil>.favorites`` (pygamez.com: Server-Schlüssel ``favorites``),
die Rust-Fassung ebenfalls in mem.json - die Bytes sind gleich.

Ob Favoriten in der Spieleliste der Seitenleiste oben stehen, ist dagegen
eine Einstellung (settings.json: ``"favorites": {"top": false}``).
"""

import store

SECTION = "favorites"
# Obergrenze, damit eine kaputte/fremde Datei die Liste nicht aufbläht.
MAX_GAMES = 64
_MAX_KEY_LEN = 32


def _clean(raw):
    """Liste aus dem Abschnitt prüfen: nur kurze Texte, keine Doppelten."""
    out = []
    if not isinstance(raw, list):
        return out
    for k in raw:
        if isinstance(k, str) and 0 < len(k) <= _MAX_KEY_LEN and k not in out:
            out.append(k)
        if len(out) >= MAX_GAMES:
            break
    return out


def load(profile=None):
    """Favoriten (Liste von highscore_keys) des aktiven bzw. genannten Profils."""
    return _clean(store.load_section(SECTION, profile).get("games"))


def save(keys, profile=None):
    store.save_section(SECTION, {"games": _clean(list(keys))}, profile)


def is_fav(key, favs=None):
    return key in (load() if favs is None else favs)


def toggle(key):
    """Schaltet ein Spiel um und speichert. Rückgabe: True = jetzt Favorit."""
    favs = load()
    if key in favs:
        favs.remove(key)
        now = False
    else:
        favs.append(key)
        now = True
    save(favs)
    return now


def ordered(classes, favs, top):
    """Spiele-Reihenfolge für die Seitenleiste.

    top=False: unverändert. top=True: erst die Favoriten, dann der Rest -
    beide Gruppen behalten die normale Spiele-Reihenfolge.
    """
    classes = list(classes)
    if not top or not favs:
        return classes
    fav = [c for c in classes if c.highscore_key in favs]
    rest = [c for c in classes if c.highscore_key not in favs]
    return fav + rest


def top_enabled(settings):
    """Einstellung "Favoriten oben in der Spieleliste" (Standard: aus)."""
    sect = settings.get(SECTION) if isinstance(settings, dict) else None
    return bool(sect.get("top", False)) if isinstance(sect, dict) else False
