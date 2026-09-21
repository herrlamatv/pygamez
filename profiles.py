# -*- coding: utf-8 -*-
"""
profiles.py
===========
Spielerprofile - mehrere Spieler an EINEM Gerät.

Jedes Profil hat seine eigenen Highscores, Erfolge, Statistiken, Lama-Chips
und Spielstände. Technisch liegt das alles in ``mem.json`` unter
``players/<id>`` (siehe store.py) bzw. in ``stats.pgzlamastats``; gemeinsam
bleiben nur die Geräte-Einstellungen (``settings.json``: Auflösung, Ton,
Tastenbelegung, Design) und die Sprache der Oberfläche.

Dieses Modul kennt die Regeln drumherum:

- Anlegen (höchstens ``store.MAX_PROFILES``), Umbenennen, Löschen, Wechseln,
- Namensprüfung (Länge + Wortfilter in allen 14 Sprachen, wie bei den
  selbstgebauten Bahnen in ugc.py),
- Aussehen: ein Symbol (``ICONS``) und eine Farbe (``COLORS``) je Profil,
- den Gast-Platz für Spieler 2 im lokalen Mehrspieler (id ``GUEST``).

Wichtig: Dieses Modul kommt OHNE pygame aus - main.py prüft die
Abhängigkeiten, bevor pygame zum ersten Mal importiert wird.
"""

import store

# Profil-Platzhalter für "kein Profil" (Spieler 2 spielt als Gast).
GUEST = "guest"

# Auswählbare Symbole (gezeichnet von achievements.draw_badge/_draw_icon).
ICONS = ("llama", "star", "crown", "gem", "heart", "rocket",
         "moon", "flame", "dice", "target", "medal", "check")

# Auswählbare Profilfarben (Index = "color" in den Kopfdaten).
COLORS = ("#6ecd8c", "#589cff", "#f0a05a", "#b07fe8", "#5ad4d4", "#f5cd64",
          "#e05a7d", "#4fd17a", "#c86ad8", "#e2725b", "#35e2ff", "#b9f23a")

# Längster erlaubter Profilname (Anzeige in Sidebar/HUD bleibt lesbar).
MAX_NAME = 14

# Fehlercodes der Namensprüfung (Oberfläche zeigt t("profile.err_<code>")).
ERR_EMPTY = "empty"
ERR_LONG = "long"
ERR_DIRTY = "dirty"
ERR_DUP = "dup"
ERR_FULL = "full"


# ----- Abfragen -----------------------------------------------------------

def active_id():
    """id des aktiven Profils."""
    return store.active_id()


def all_profiles():
    """[(id, meta), ...] in Anzeige-Reihenfolge."""
    return store.all_profiles()


def count():
    return len(store.profile_ids())


def meta(pid=None):
    """Kopfdaten eines Profils (Kopie)."""
    return store.profile_meta(pid)


def name_of(pid=None):
    """Anzeigename eines Profils (leerer Name -> "Spieler N")."""
    if pid == GUEST:
        from i18n import t
        return t("profile.guest")
    entry = store.profile_meta(pid)
    name = (entry.get("name") or "").strip()
    if name:
        return name
    from i18n import t
    ids = store.profile_ids()
    pid = pid or store.active_id()
    n = (ids.index(pid) + 1) if pid in ids else 1
    return t("profile.player_n", n=n)


def icon_of(pid=None):
    if pid == GUEST:
        return "person"
    icon = store.profile_meta(pid).get("icon") or "llama"
    return icon if icon in ICONS or icon == "person" else "llama"


def color_hex(pid=None):
    if pid == GUEST:
        return "#9aa6bf"
    idx = store.profile_meta(pid).get("color", 0)
    return COLORS[idx % len(COLORS)]


def color_rgb(pid=None):
    """Profilfarbe als (r, g, b) - ohne ui/pygame, damit früh nutzbar."""
    value = color_hex(pid).lstrip("#")
    return tuple(int(value[i:i + 2], 16) for i in (0, 2, 4))


def level_of(pid=None):
    """(Level, Punkte) des Profils aus der Statistik-Datei."""
    import stats
    return stats.level_of(pid or store.active_id())


# ----- Namen prüfen -------------------------------------------------------

def check_name(name, pid=None):
    """Prüft einen Profilnamen; "" = in Ordnung, sonst ein ERR_-Code.

    'pid' ist das Profil, das umbenannt wird (sein eigener Name gilt nicht
    als Dublette).
    """
    name = (name or "").strip()
    if not name:
        return ERR_EMPTY
    if len(name) > MAX_NAME:
        return ERR_LONG
    import swear
    if not swear.all_clean(name):
        return ERR_DIRTY
    lower = name.casefold()
    for other, entry in store.all_profiles():
        if other == pid:
            continue
        if (entry.get("name") or "").strip().casefold() == lower:
            return ERR_DUP
    return ""


# ----- Ändern -------------------------------------------------------------

def create(name, icon="llama", color=0):
    """Legt ein Profil an. Gibt (id, "") oder (None, ERR_-Code) zurück."""
    err = check_name(name)
    if err:
        return None, err
    if count() >= store.MAX_PROFILES:
        return None, ERR_FULL
    pid = store.create_profile(name=name.strip(), icon=icon, color=int(color))
    if pid is None:
        return None, ERR_FULL
    # Erfolg: "Gesellschaftsspieler" (zweites Profil angelegt).
    try:
        import achievements
        achievements.event("profile_second", count())
        achievements.event("profile_family", count())
    except Exception:
        pass
    return pid, ""


def rename(pid, name):
    """Benennt ein Profil um. Gibt "" oder einen ERR_-Code zurück."""
    err = check_name(name, pid=pid)
    if err:
        return err
    store.set_meta(pid, name=name.strip())
    return ""


def set_look(pid, icon=None, color=None):
    """Ändert Symbol und/oder Farbe eines Profils."""
    fields = {}
    if icon is not None:
        fields["icon"] = icon
    if color is not None:
        fields["color"] = int(color)
    if fields:
        store.set_meta(pid, **fields)


def delete(pid):
    """Löscht ein Profil samt Daten (das letzte Profil bleibt bestehen)."""
    if count() <= 1:
        return False
    import stats
    stats.flush()
    stats.drop_profile(pid)
    return store.delete_profile(pid)


def switch(pid):
    """Wechselt das aktive Profil (schreibt vorher alles Offene weg)."""
    if pid == store.active_id():
        return False
    import stats
    stats.flush()
    ok = store.set_active(pid)
    if ok:
        store.touch(pid)
    return ok


def touch(pid=None):
    """Merkt sich den letzten Besuch eines Profils."""
    store.touch(pid)
