# -*- coding: utf-8 -*-
"""
lamabank.py
===========
Die **Lama-Bank** - das gemeinsame Chip-Konto von Blackjack, Poker und Casino.

Alle drei Spiele setzen dieselben "Lama-Chips". Gespeichert wird im Abschnitt
``casino`` der gemeinsamen ``mem.json``:

    {
      "chips":    1234,                               # Kontostand
      "ledger":   {"blackjack": 0, "poker": 0, "casino": 0},   # Netto-Bilanz je Spiel
      "peak":     {"blackjack": 1000, ...},           # Höchststand je Spiel
      "escrow":   0,                                  # Poker-Tischstapel (unterwegs)
      "refills":  0,                                  # genommene Bank-Kredite
      "migrated": true,                               # Altbestände übernommen?
      "extra":    {...}                               # kleine Spielstände (Verlauf ...)
    }

Regeln:

- **Einsatz sofort abbuchen + speichern.** Wer eine laufende Hand/Drehung
  verlässt, bekommt seinen Einsatz NICHT zurück (früher lagen Chips bis zur
  Auszahlung nur im Speicher).
- **Poker-Tischstapel = escrow.** Beim Setzen an den Tisch wandert der Stapel
  vom Konto ins escrow; nur was wirklich in den Pot geht, wird abgebucht. Bricht
  die Hand ab (Menü, Absturz), erstattet ``load()`` den Stapel beim nächsten
  Start.
- **Je Spiel eine eigene Bilanz** (``ledger`` = Gewinne minus Einsätze in
  diesem Spiel). Der Highscore eines Spiels ist der Höchststand von
  ``START_CHIPS + Bilanz`` - bzw. der übernommene alte Bestwert, falls der
  höher war. Ein Slot-Gewinn hebt also keinen Blackjack-Highscore, und
  Bank-Kredite zählen nirgends mit.
- **Pleite** = weniger als der Mindesteinsatz des gerade geöffneten Spiels.
  Dann gibt die Bank einen Kredit: das Konto wird wieder auf ``START_CHIPS``
  aufgefüllt (``refills`` zählt mit).

Migration (einmalig): Konto = max(Blackjack-Chips, Poker-Chips) der alten
Abschnitte (sonst 1000), ``peak`` = alter Bestwert bzw. vorhandener Highscore.
Die Abschnitte ``blackjack``/``poker`` bleiben unangetastet liegen, der
Abschnitt ``highscores`` wird nur gelesen, nie geschrieben.
"""

import store

SECTION = "casino"
START_CHIPS = 1000
GAMES = ("blackjack", "poker", "casino")

# Mindesteinsatz je Spiel bzw. "spiel:modus" (Pleite-Regel).
#   Blackjack: kleinster Chip 10 · Poker: Big Blind 20 (Hold'em und Draw),
#   Video Poker 10 · Roulette: 1er-Chip · Lama-Slot: 10 Linien x 1
MIN_BET = {
    "blackjack": 10,
    "poker": 20,
    "poker:video": 10,
    "casino": 1,
    "casino:slots": 10,
}

_data = None          # zuletzt geladener Stand (Cache - draw() liest pro Frame)


# ===================================================== Laden / Speichern
def _int(value, default=0, lo=None):
    """Robuste Ganzzahl aus JSON (bool zählt nicht als Zahl)."""
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        return default
    v = int(value)
    return v if lo is None else max(lo, v)


def _blank():
    return {"chips": START_CHIPS,
            "ledger": {g: 0 for g in GAMES},
            "peak": {g: START_CHIPS for g in GAMES},
            "escrow": 0, "refills": 0, "migrated": False, "extra": {}}


def _clean(raw):
    """Bringt einen (evtl. beschädigten) Abschnitt in Form."""
    out = _blank()
    if not isinstance(raw, dict):
        return out
    out["chips"] = _int(raw.get("chips"), START_CHIPS, lo=0)
    for field, default in (("ledger", 0), ("peak", START_CHIPS)):
        src = raw.get(field) if isinstance(raw.get(field), dict) else {}
        for g in GAMES:
            out[field][g] = _int(src.get(g), default)
    out["escrow"] = _int(raw.get("escrow"), 0, lo=0)
    out["refills"] = _int(raw.get("refills"), 0, lo=0)
    out["migrated"] = raw.get("migrated") is True
    if isinstance(raw.get("extra"), dict):
        out["extra"] = raw["extra"]
    return out


def _migrate(full):
    """Übernimmt Chips und Bestwerte aus den alten Blackjack-/Poker-Abschnitten.

    'full' ist der komplette mem.json-Inhalt; gelesen wird nur.
    """
    data = _blank()
    hs = full.get("highscores") if isinstance(full.get("highscores"), dict) else {}
    chips = []
    for game in ("blackjack", "poker"):
        old = full.get(game) if isinstance(full.get(game), dict) else {}
        c = _int(old.get("chips"), None, lo=0)
        if c is not None:
            chips.append(c)
        best = max(_int(old.get("best"), 0), _int(hs.get(game), 0))
        data["peak"][game] = max(START_CHIPS, best)
    data["peak"]["casino"] = max(START_CHIPS, _int(hs.get("casino"), 0))
    data["chips"] = max(chips) if chips else START_CHIPS
    data["migrated"] = True
    return data


def _save():
    if _data is not None:
        store.save_section(SECTION, _data)


def load():
    """Liest das Konto frisch aus mem.json (Migration + escrow-Erstattung).

    Aufrufen beim Start/Reset eines Chip-Spiels. Liefert den Stand als dict.
    """
    global _data
    full = store.load()
    raw = full.get(SECTION)
    changed = False
    if isinstance(raw, dict) and raw.get("migrated") is True:
        _data = _clean(raw)
        changed = _data != raw
    else:
        _data = _migrate(full)
        changed = True
    if _data["escrow"] > 0:
        # Abgebrochene Poker-Hand: der Tischstapel kommt zurück aufs Konto.
        _data["chips"] += _data["escrow"]
        _data["escrow"] = 0
        changed = True
    if changed:
        _save()
    return _data


def _d():
    return _data if _data is not None else load()


# ===================================================== Abfragen
def balance():
    """Aktueller Kontostand (ohne escrow)."""
    return _d()["chips"]


def escrow():
    """Chips, die gerade als Poker-Tischstapel unterwegs sind."""
    return _d()["escrow"]


def ledger(game):
    """Netto-Bilanz eines Spiels (Gewinne minus Einsätze)."""
    return _d()["ledger"].get(game, 0)


def value_for(game):
    """Bilanz-Stand eines Spiels: START_CHIPS + Bilanz (für Erfolge)."""
    return START_CHIPS + ledger(game)


def score_for(game):
    """Highscore eines Spiels: Höchststand von START_CHIPS + Bilanz."""
    d = _d()
    return max(d["peak"].get(game, START_CHIPS), value_for(game))


def refills():
    """Anzahl der bisher genommenen Bank-Kredite."""
    return _d()["refills"]


def min_bet(game, mode=None):
    """Mindesteinsatz eines Spiels (optional je Modus)."""
    if mode is not None and f"{game}:{mode}" in MIN_BET:
        return MIN_BET[f"{game}:{mode}"]
    return MIN_BET.get(game, 1)


def is_broke(game, mode=None):
    """True, wenn das Konto unter dem Mindesteinsatz liegt."""
    return balance() < min_bet(game, mode)


def get_extra(key, default=None):
    """Kleiner Zusatz-Spielstand (z.B. Roulette-Verlauf, offene Freispiele)."""
    return _d()["extra"].get(key, default)


# ===================================================== Buchungen
def debit(n, game):
    """Bucht einen Einsatz ab und speichert sofort. False = nicht gedeckt."""
    d = _d()
    n = int(n)
    if n <= 0:
        return True
    if n > d["chips"]:
        return False
    d["chips"] -= n
    d["ledger"][game] = d["ledger"].get(game, 0) - n
    _save()
    return True


def credit(n, game, save=True):
    """Schreibt einen Gewinn gut (zählt zur Bilanz und zum Höchststand)."""
    d = _d()
    n = int(n)
    if n <= 0:
        return
    d["chips"] += n
    d["ledger"][game] = d["ledger"].get(game, 0) + n
    d["peak"][game] = max(d["peak"].get(game, START_CHIPS),
                          START_CHIPS + d["ledger"][game])
    if save:
        _save()


def refill_if_broke(game, mode=None):
    """Bank-Kredit bei Pleite: Konto zurück auf START_CHIPS (zählt nicht zur Bilanz)."""
    d = _d()
    if d["chips"] >= min_bet(game, mode):
        return False
    d["chips"] = START_CHIPS
    d["refills"] += 1
    _save()
    return True


def set_escrow(n, game="poker"):
    """Legt n Chips als Tischstapel ins escrow (kein Einsatz, keine Bilanz)."""
    d = _d()
    n = max(0, min(int(n), d["chips"]))
    d["chips"] -= n
    d["escrow"] += n
    _save()
    return n


def pay_from_escrow(n, game="poker"):
    """Chips aus dem Tischstapel in den Pot: gilt als Einsatz (Bilanz sinkt)."""
    d = _d()
    n = max(0, min(int(n), d["escrow"]))
    if n <= 0:
        return 0
    d["escrow"] -= n
    d["ledger"][game] = d["ledger"].get(game, 0) - n
    _save()
    return n


def clear_escrow(save=True):
    """Tischstapel zurück aufs Konto (Hand vorbei oder Tisch verlassen)."""
    d = _d()
    if d["escrow"] <= 0:
        return 0
    n = d["escrow"]
    d["chips"] += n
    d["escrow"] = 0
    if save:
        _save()
    return n


def settle_escrow(win, game="poker"):
    """Hand-Ende: Tischstapel zurück + Pot-Gewinn gutschreiben (ein Speichern)."""
    clear_escrow(save=False)
    if win > 0:
        credit(win, game, save=False)
    _save()


def set_extra(key, value):
    """Speichert einen kleinen Zusatz-Spielstand (JSON-fähig)."""
    d = _d()
    d["extra"][key] = value
    _save()
