# -*- coding: utf-8 -*-
"""
wordle.py
=========
Wordle - errate das gesuchte Wort in wenigen Versuchen. Vier Modi:

- **Endlos**: ein Wort nach dem anderen (6 Versuche je Wort). Jedes gelöste
  Wort bringt Punkte (weniger Versuche = mehr), das erste NICHT gelöste Wort
  beendet die Partie. Nur Endlos mit 5 Buchstaben zählt für den Highscore;
  die anderen Längen haben eigene Bestwerte.
- **Tageswort**: ein Wort pro Tag, je Sprache und Länge für alle gleich
  (``seedrand`` - PC und Browser liefern dasselbe Wort). Angefangene
  Tageswörter werden gespeichert; danach gibt es Ergebnis, Statistik und den
  Countdown bis morgen. Serie = Tageswörter in Folge.
- **Dordle**: zwei Wörter gleichzeitig, 7 Versuche - jeder Rateversuch gilt
  für beide Bretter.
- **Quordle**: vier Wörter gleichzeitig, 9 Versuche. Die Tasten der
  Bildschirmtastatur zeigen die Farben je Brett geviertelt.

Setup vor jeder Partie: Wortlänge 4-7, harter Modus (gefundene Hinweise
müssen weiterverwendet werden) und eine Farbenblind-Palette (Orange/Blau
statt Grün/Gelb). Nach jeder Partie: Statistik je Sprache/Länge/Modus mit
Versuchsverteilung und "Ergebnis teilen" (Emoji-Raster in die
Zwischenablage).

Bewertung: grün = richtig (Position stimmt), gelb = im Wort (falsche
Position), grau = nicht enthalten. Doppelte Buchstaben werden korrekt
gezählt (Standard-Wordle-Algorithmus, für jede Wortlänge).

Steuerung: Buchstaben A-Z tippen, Enter = raten, Backspace = löschen, die
Bildschirmtastatur ist anklickbar (QWERTZ für de/cs/sl/hr, AZERTY für
Französisch, sonst QWERTY). Das Regler-Symbol oben rechts führt zurück ins
Setup. Nach einer Partie: Enter = nochmal, C = teilen, S = Setup,
Leertaste = Bretter ansehen.
"""

import math
import random
import sys
import time

import pygame

import i18n
import seedrand
import settings as settings_mod
import store
import ui
from game_base import Game, InputEvent
from i18n import t

from .wordle_words import DEFAULT_LENGTH, LENGTHS, allowed_for, has_length, words_for

# ----------------------------------------------------------------- Modi
MODES = [("endless", "wd.mode.endless"), ("daily", "wd.mode.daily"),
         ("dordle", "wd.mode.dordle"), ("quordle", "wd.mode.quordle")]
BOARDS = {"endless": 1, "daily": 1, "dordle": 2, "quordle": 4}
MAX_ROWS = {"endless": 6, "daily": 6, "dordle": 7, "quordle": 9}
SHARE_NAME = {"endless": "Wordle", "daily": "Wordle", "dordle": "Dordle",
              "quordle": "Quordle"}
# Frühere Modus-Schlüssel (Normal/Hart als eigene Knöpfe) -> heutiger Modus.
LEGACY_MODES = {"normal": "endless", "hard": "endless", "single": "endless"}
# Nur Endlos mit dieser Wortlänge füllt den Highscore (wie vor dem Ausbau).
SCORE_LENGTH = 5

SETUP, PLAY, REVEAL, SOLVED, DONE = "setup", "play", "reveal", "solved", "done"

# ----------------------------------------------------------------- Farben
# Kachelfarben sind die Identität des Spiels und bleiben in jedem Theme
# gleich; alles drumherum (Panels, Texte, Tasten) kommt aus ui.*.
PALETTES = {
    False: {"correct": (106, 170, 100), "present": (201, 180, 88)},
    True: {"correct": (245, 121, 58), "present": (133, 192, 249)},
}
COL_ABSENT = (58, 58, 62)
COL_LETTER = (240, 241, 246)
COL_LOSE = (225, 110, 100)

# Emoji fürs Teilen (Farbenblind: Orange/Blau wie im Original).
EMOJI = {
    False: {"correct": "\U0001F7E9", "present": "\U0001F7E8"},
    True: {"correct": "\U0001F7E7", "present": "\U0001F7E6"},
}
EMOJI_ABSENT = "⬛"
EMOJI_EMPTY = "⬜"
EMOJI_FAIL = "\U0001F7E5"
KEYCAP = "️⃣"

# ----------------------------------------------------------------- Zeiten
FLIP_TIME = 0.26          # eine Kachel dreht sich um
MESSAGE_TIME = 1.6        # Meldung bleibt stehen
SHAKE_TIME = 0.4          # abgelehnte Zeile wackelt
POP_TIME = 0.11           # getippter Buchstabe "ploppt" auf
BOUNCE_TIME = 0.5         # gelöste Zeile hüpft
PANEL_DELAY = 0.45        # Ergebnis erscheint nach der letzten Aufdeckung
# Höhe des Highscore-Banners, den main.py bei Game Over unten einblendet
# (feste 18-px-Schrift + Rand) - das Ergebnis-Panel lässt den Platz frei.
BANNER_SPACE = 50

# ----------------------------------------------------------------- Tastatur
KEYBOARDS = {
    "qwerty": ["QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"],
    "qwertz": ["QWERTZUIOP", "ASDFGHJKL", "YXCVBNM"],
    "azerty": ["AZERTYUIOP", "QSDFGHJKLM", "WXCVBN"],
}
# Tschechisch, Slowenisch und Kroatisch tippen wie Deutsch auf QWERTZ.
LANG_KEYBOARD = {"de": "qwertz", "cs": "qwertz", "sl": "qwertz",
                 "hr": "qwertz", "fr": "azerty"}

# Rangordnung der Buchstaben-Zustände (höher gewinnt in der Tastatur-Färbung)
_RANK = {None: 0, "absent": 1, "present": 2, "correct": 3}


# =====================================================================
#  Spiellogik ohne pygame (auch für Tests und als Vorlage für wordle.js)
# =====================================================================
def evaluate(guess, answer):
    """Standard-Wordle-Bewertung mit korrekter Doppelbuchstaben-Zählung.

    Erst alle Treffer an der richtigen Stelle, dann die übrigen Buchstaben
    von links nach rechts - jeder Buchstabe der Lösung wird höchstens einmal
    "verbraucht". Funktioniert für jede Wortlänge.
    """
    n = len(answer)
    result = ["absent"] * n
    rest = list(answer)
    for i in range(n):
        if guess[i] == answer[i]:
            result[i] = "correct"
            rest[i] = None
    for i in range(n):
        if result[i] == "correct":
            continue
        if guess[i] in rest:
            result[i] = "present"
            rest[rest.index(guess[i])] = None
    return result


def hard_problem(guess, history):
    """Verstößt 'guess' im harten Modus gegen einen Hinweis?

    history = [(wort, ergebnis), ...] eines Bretts. Liefert None oder
    (i18n-Schlüssel, Werte) der ersten Verletzung: grüne Buchstaben müssen
    an ihrer Stelle bleiben, grüne/gelbe mindestens so oft wieder vorkommen
    wie im Hinweis (Standard-Regel des Originals).
    """
    for word, result in history:
        for i, state in enumerate(result):
            if state == "correct" and guess[i] != word[i]:
                return "wd.hard_green", {"n": i + 1, "c": word[i]}
        need = {}
        for i, state in enumerate(result):
            if state != "absent":
                need[word[i]] = need.get(word[i], 0) + 1
        for ch, count in need.items():
            if guess.count(ch) < count:
                return "wd.hard_yellow", {"c": ch}
    return None


_order_cache = {}


def daily_answer(words, lang, length, date=None):
    """Das Tageswort für Sprache und Länge (am PC und im Browser gleich).

    Die Lösungswörter werden einmal je Sprache/Länge fest gemischt
    (seedrand, Seed aus "wordle"/Sprache/Länge); der Tag (Tage seit
    2026-01-01) wählt die Stelle in dieser Reihenfolge. So kommt kein Wort
    wieder, bevor die ganze Liste durch ist.
    """
    n = len(words)
    key = (lang, length, n)
    order = _order_cache.get(key)
    if order is None:
        order = list(range(n))
        seedrand.Rand(seedrand.seed_from("wordle", lang, length)).shuffle(order)
        _order_cache[key] = order
    return words[order[seedrand.day_index(date) % n]]


def seconds_to_midnight(now=None):
    """Sekunden bis zum nächsten lokalen Tageswechsel."""
    now = time.time() if now is None else now
    lt = time.localtime(now)
    midnight = time.mktime((lt.tm_year, lt.tm_mon, lt.tm_mday + 1,
                            0, 0, 0, 0, 0, -1))
    return max(0, int(round(midnight - now)))


def format_hms(seconds):
    seconds = max(0, int(seconds))
    return "%02d:%02d:%02d" % (seconds // 3600, seconds // 60 % 60, seconds % 60)


# ----------------------------------------------------------------- Statistik
def stats_key(mode, lang, length):
    return "%s:%s:%d" % (mode, lang, length)


def load_data():
    """mem.json-Section 'wordle' - fehlende/kaputte Teile werden ergänzt."""
    data = store.load_section("wordle")
    for key in ("stats", "daily", "best"):
        if not isinstance(data.get(key), dict):
            data[key] = {}
    return data


def save_data(data):
    store.save_section("wordle", data)


def get_stats(data, mode, lang, length):
    """Bereinigte Kopie der Statistik (Partien, Siege, Serien, Verteilung)."""
    rows = MAX_ROWS[mode]
    raw = data.get("stats", {}).get(stats_key(mode, lang, length))
    raw = raw if isinstance(raw, dict) else {}

    def num(key):
        v = raw.get(key, 0)
        return v if isinstance(v, int) and not isinstance(v, bool) and v >= 0 else 0

    dist = raw.get("dist")
    dist = [v if isinstance(v, int) and v >= 0 else 0
            for v in (dist if isinstance(dist, list) else [])][:rows]
    dist += [0] * (rows - len(dist))
    out = {"played": num("played"), "won": num("won"), "streak": num("streak"),
           "best": num("best"), "dist": dist}
    if isinstance(raw.get("last_day"), int):
        out["last_day"] = raw["last_day"]
    out["won"] = min(out["won"], out["played"])
    return out


def record_game(data, mode, lang, length, won, tries, day=None):
    """Trägt eine beendete Partie ein und liefert die neue Statistik.

    Tageswort (day = seedrand.day_index): die Serie wächst nur, wenn auch
    das Tageswort von GESTERN gelöst wurde - ein ausgelassener Tag beendet
    sie. Sonst zählt jeder Sieg in Folge.
    """
    st = get_stats(data, mode, lang, length)
    st["played"] += 1
    if won:
        st["won"] += 1
        if 1 <= tries <= len(st["dist"]):
            st["dist"][tries - 1] += 1
        if day is not None and st.get("last_day") != day - 1:
            st["streak"] = 1
        else:
            st["streak"] += 1
        st["best"] = max(st["best"], st["streak"])
        if day is not None:
            st["last_day"] = day
    else:
        st["streak"] = 0
    data.setdefault("stats", {})[stats_key(mode, lang, length)] = st
    return st


def shown_streak(st, mode, today=None):
    """Aktuelle Serie für die Anzeige (Tageswort: verfällt nach einem Tag Pause)."""
    if mode != "daily":
        return st["streak"]
    today = seedrand.day_index() if today is None else today
    if st.get("last_day") in (today, today - 1):
        return st["streak"]
    return 0


# ----------------------------------------------------------------- Teilen
def share_text(mode, lang, length, boards, hard=False, colorblind=False,
               date=None):
    """Ergebnis als Emoji-Raster zum Teilen.

    boards = [(ergebnisse, gelöst_nach), ...] - je Brett die Ergebnis-Zeilen
    bis zur Lösung und die Anzahl Versuche (None = nicht gelöst).
    """
    rows = MAX_ROWS[mode]
    palette = EMOJI[bool(colorblind)]
    head = "PyGameZ " + SHARE_NAME[mode]
    if mode == "daily":
        head += " " + (date or seedrand.today_str())
    head += " · %s · %d" % (lang.upper(), length)
    star = "*" if hard else ""

    def line(results, r):
        if r >= len(results):
            return EMOJI_EMPTY * length
        return "".join(palette.get(st, EMOJI_ABSENT) for st in results[r])

    def score(solved):
        return str(solved) if solved else "X"

    if len(boards) == 1:
        results, solved = boards[0]
        lines = ["%s %s/%d%s" % (head, score(solved), rows, star), ""]
        lines += [line(results, r) for r in range(len(results))]
    elif len(boards) == 2:
        lines = ["%s %s/%d%s" % (head, "&".join(score(s) for _, s in boards),
                                 rows, star), ""]
        used = max(len(res) for res, _ in boards)
        lines += [line(boards[0][0], r) + " " + line(boards[1][0], r)
                  for r in range(used)]
    else:
        def badge(solved):
            return (str(solved) + KEYCAP) if solved else EMOJI_FAIL
        lines = [head + star, badge(boards[0][1]) + badge(boards[1][1]),
                 badge(boards[2][1]) + badge(boards[3][1]), ""]
        for a, b in ((0, 1), (2, 3)):
            used = max(len(boards[a][0]), len(boards[b][0]))
            lines += [line(boards[a][0], r) + " " + line(boards[b][0], r)
                      for r in range(used)]
            if a == 0:
                lines.append("")
    return "\n".join(lines)


def copy_to_clipboard(text, tk_root=None):
    """Legt 'text' in die Zwischenablage (False, wenn das nicht geht).

    Das Spiel läuft eingebettet in einem Tk-Fenster - dessen Zwischenablage
    kann auch Emoji (unter Windows geht der Text direkt an das System und
    bleibt auch nach dem Beenden erhalten). Klappt das nicht, schreibt unter
    Windows die Win32-API den Text als Unicode hinein. Ohne beides (Tests,
    Headless) gibt es einfach False.
    """
    try:
        import tkinter
        root = tk_root or getattr(tkinter, "_default_root", None)
        if root is not None:
            root.clipboard_clear()
            root.clipboard_append(text)
            root.update_idletasks()
            return True
    except Exception:
        pass
    return _win_clipboard(text)


def _win_clipboard(text):
    """Unicode-Text über die Win32-Zwischenablage setzen (nur Windows)."""
    if sys.platform != "win32":
        return False
    try:
        import ctypes
        from ctypes import wintypes
        user32 = ctypes.WinDLL("user32", use_last_error=True)
        kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
        user32.OpenClipboard.argtypes = [wintypes.HWND]
        user32.SetClipboardData.argtypes = [wintypes.UINT, wintypes.HANDLE]
        user32.SetClipboardData.restype = wintypes.HANDLE
        kernel32.GlobalAlloc.argtypes = [wintypes.UINT, ctypes.c_size_t]
        kernel32.GlobalAlloc.restype = wintypes.HGLOBAL
        kernel32.GlobalLock.argtypes = [wintypes.HGLOBAL]
        kernel32.GlobalLock.restype = wintypes.LPVOID
        kernel32.GlobalUnlock.argtypes = [wintypes.HGLOBAL]
        kernel32.GlobalFree.argtypes = [wintypes.HGLOBAL]
        data = text.encode("utf-16-le") + b"\x00\x00"
        if not user32.OpenClipboard(None):
            return False
        try:
            user32.EmptyClipboard()
            handle = kernel32.GlobalAlloc(0x0042, len(data))    # GMEM_MOVEABLE|ZEROINIT
            if not handle:
                return False
            ptr = kernel32.GlobalLock(handle)
            ctypes.memmove(ptr, data, len(data))
            kernel32.GlobalUnlock(handle)
            if not user32.SetClipboardData(13, handle):          # CF_UNICODETEXT
                kernel32.GlobalFree(handle)
                return False
            return True
        finally:
            user32.CloseClipboard()
    except Exception:
        return False


# =====================================================================
#  Ein Brett (ein gesuchtes Wort)
# =====================================================================
class Board:
    """Zustand eines Bretts: Lösung, bisherige Zeilen, Tastatur-Wissen."""

    __slots__ = ("answer", "rows", "solved_at", "keys", "solve_t")

    def __init__(self, answer):
        self.answer = answer
        self.rows = []            # [(wort, ergebnis), ...] bis zur Lösung
        self.solved_at = None     # Anzahl Versuche bis zur Lösung
        self.keys = {}            # Buchstabe -> bester bekannter Zustand
        self.solve_t = None       # Animationszeit der Lösung (Hüpfen)

    @property
    def solved(self):
        return self.solved_at is not None

    def apply(self, guess, result, anim_t=0.0):
        self.rows.append((guess, result))
        for ch, st in zip(guess, result):
            if _RANK[st] > _RANK[self.keys.get(ch)]:
                self.keys[ch] = st
        if all(st == "correct" for st in result):
            self.solved_at = len(self.rows)
            self.solve_t = anim_t


# =====================================================================
#  Das Spiel
# =====================================================================
class WordleGame(Game):
    name = "Wordle"
    highscore_key = "wordle"
    supports_multiplayer = False
    MODES = MODES

    @property
    def show_highscore_banner(self):
        """Highscore gibt es nur in Endlos mit 5 Buchstaben."""
        return (getattr(self, "game_mode", "endless") == "endless"
                and getattr(self, "length", SCORE_LENGTH) == SCORE_LENGTH)

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False
        mode = LEGACY_MODES.get(self.mode, self.mode)
        self.game_mode = mode if mode in BOARDS else "endless"
        self.lang = i18n.get_language()
        if self.mode == "hard" and not self._opts()["hard"]:
            # Früherer Modus-Knopf "Hart" = Endlos mit eingeschaltetem
            # harten Modus (wird zur Einstellung).
            self._save_opt("hard", True)
        opts = self._opts()
        self.length = opts["length"]
        self.hard = opts["hard"]
        self.colorblind = opts["colorblind"]
        self.data = load_data()

        self.anim = 0.0                # Animationsuhr (läuft auch nach Game Over)
        self._updated = False
        self._last_draw = None
        self.state = SETUP
        self.setup_focus = 3           # 0 Länge, 1 hart, 2 farbenblind, 3 Start
        self.setup_t = 0.0
        self._toggle_anim = {}

        self.boards = []
        self.guesses = []
        self.current = ""
        self.reveal = None
        self.points = 0
        self.last_points = 0
        self.solved_count = 0
        self.solved_t = 0.0
        self.message = ""
        self.message_t = 0.0
        self.shake = 0.0
        self.pops = {}
        self.result = None
        self.panel_hidden = False
        self.daily_date = None
        self.daily_view = False

        self._fit_cache = {}
        self._letter_cache = {}
        self._board_cache = {}
        self._make_fonts()
        self._layout()

    def _opts(self):
        """Wordle-Einstellungen (settings.json) mit Prüfung."""
        ws = self.settings.get("wordle", {}) if isinstance(self.settings, dict) else {}
        ws = ws if isinstance(ws, dict) else {}
        length = ws.get("length", DEFAULT_LENGTH)
        if not isinstance(length, int) or length not in LENGTHS:
            length = DEFAULT_LENGTH
        return {"length": length, "hard": ws.get("hard") is True,
                "colorblind": ws.get("colorblind") is True}

    def _save_opt(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("wordle", {})[key] = value
            settings_mod.save_settings(self.settings)

    @property
    def rows(self):
        return MAX_ROWS[self.game_mode]

    @property
    def n_boards(self):
        return BOARDS[self.game_mode]

    @property
    def counts_score(self):
        return self.game_mode == "endless" and self.length == SCORE_LENGTH

    def palette(self):
        return PALETTES[bool(self.colorblind)]

    def on_surface_changed(self):
        self._fit_cache.clear()
        self._letter_cache.clear()
        self._board_cache.clear()
        self._make_fonts()
        self._layout()

    def _k(self):
        return min(self.width / 800.0, self.height / 600.0)

    def _make_fonts(self):
        k = self._k()
        self._hud = ui.font(max(12, int(17 * k)), bold=True)
        self._small = ui.font(max(11, int(15 * k)))
        self._small_b = ui.font(max(11, int(15 * k)), bold=True)
        self._tiny = ui.font(max(10, int(12 * k)))
        self._big = ui.font(max(18, int(30 * k)), bold=True)
        self._num = ui.font(max(15, int(26 * k)), bold=True)

    def _fit(self, text, max_w, px, bold=False, min_px=9):
        """Größte Schrift <= px, in der 'text' höchstens max_w breit ist."""
        key = (text, int(max_w), px, bold)
        f = self._fit_cache.get(key)
        if f is None:
            size = px
            f = ui.font(size, bold=bold)
            while size > min_px and f.size(text)[0] > max_w:
                size -= 1
                f = ui.font(size, bold=bold)
            if len(self._fit_cache) > 600:
                self._fit_cache.clear()
            self._fit_cache[key] = f
        return f

    def _text(self, s, text, px, color, max_w, bold=False, **anchor):
        """Text passend verkleinert zeichnen; anchor wie get_rect(center=...).

        Passt er selbst in der kleinsten Größe nicht, wird er mit "…"
        gekürzt statt über den Rand zu laufen.
        """
        f = self._fit(text, max_w, px, bold)
        if f.size(text)[0] > max_w > 8:
            text = self._ellipsis(f, text, max_w)
        img = f.render(text, True, color)
        rc = img.get_rect(**anchor)
        s.blit(img, rc)
        return rc

    def _fits(self, text, max_w, px, bold=False, min_px=9):
        """True, wenn 'text' spätestens in Größe min_px in max_w passt."""
        return self._fit(text, max_w, px, bold, min_px).size(text)[0] <= max_w

    def _fit_parts(self, text, max_w, px):
        """Tastenhinweis "A · B · C": passt er nicht, fallen hintere Teile weg."""
        parts = text.split(" · ")
        while len(parts) > 1 and not self._fits(" · ".join(parts), max_w, px):
            parts.pop()
        return " · ".join(parts)

    @staticmethod
    def _ellipsis(f, text, max_w):
        while len(text) > 1 and f.size(text + "…")[0] > max_w:
            text = text[:-1]
        return text.rstrip() + "…"

    # ===================================================== Layout
    def _layout(self):
        self._layout_setup()
        self._layout_play()

    def _layout_setup(self):
        W, H = self.width, self.height
        k = self._k()
        self.su_tile = max(22, int(46 * k))
        self.su_title_y = max(10, int(26 * k))
        self.su_sub_y = self.su_title_y + self.su_tile + max(6, int(12 * k))
        top = self.su_sub_y + self._small.get_height() + max(8, int(16 * k))
        footer = max(22, int(34 * k))
        bottom = H - footer
        margin = max(12, int(34 * k))
        gap = max(10, int(22 * k))
        col_w = (W - 2 * margin - gap) // 2
        left = pygame.Rect(margin, top, col_w, bottom - top)
        # Linke Spalte: Überschrift Länge, 4 Knöpfe, 2 Schalter, Start.
        label_h = self._small.get_height()
        seg_h = max(28, int(46 * k))
        row_h = max(36, int(58 * k))
        start_h = max(32, int(50 * k))
        sp = max(6, int(12 * k))
        total = label_h + 4 + seg_h + sp + row_h + sp + row_h + sp * 2 + start_h
        y = left.y + max(0, (left.h - total) // 2)
        self.su_label_y = y
        y += label_h + 4
        sg = max(4, int(8 * k))
        seg_w = (col_w - 3 * sg) // 4
        self.su_seg = [pygame.Rect(left.x + i * (seg_w + sg), y, seg_w, seg_h)
                       for i in range(4)]
        y += seg_h + sp
        self.su_hard = pygame.Rect(left.x, y, col_w, row_h)
        y += row_h + sp
        self.su_cb = pygame.Rect(left.x, y, col_w, row_h)
        y += row_h + sp * 2
        self.su_start = pygame.Rect(left.x, y, col_w, start_h)
        # Rechte Spalte: Statistik, so hoch wie die Einstellungen daneben.
        st_top = max(top, self.su_label_y - max(4, int(6 * k)))
        self.su_stats_rect = pygame.Rect(margin + col_w + gap, st_top, col_w,
                                         max(self.su_start.bottom - st_top,
                                             int(250 * k)))

    def _layout_play(self):
        W, H = self.width, self.height
        k = self._k()
        self.hud_h = max(28, int(44 * k))
        self.gear_rect = pygame.Rect(W - self.hud_h + 2, 3, self.hud_h - 6,
                                     self.hud_h - 6)
        self._build_keyboard()
        n = self.n_boards
        pad = max(8, int(20 * k)) if n == 1 else max(4, int(14 * k))
        lane = max(16, int(30 * k)) if n == 1 else max(4, int(8 * k))
        top = self.hud_h + lane
        bottom = self.kb_top - max(6, int(10 * k))
        area = pygame.Rect(pad, top, W - 2 * pad, max(40, bottom - top))
        L, R = self.length, self.rows
        options = {1: [(1, 1)], 2: [(2, 1), (1, 2)], 4: [(4, 1), (2, 2)]}[n]
        best = None
        for bc, br in options:
            bgap = max(6, int(16 * k)) if n > 1 else 0
            inner = max(3, int(7 * k)) if n > 1 else 0      # Panel-Rand je Brett
            cell_w = (area.w - (bc - 1) * bgap) / bc - 2 * inner
            cell_h = (area.h - (br - 1) * bgap) / br - 2 * inner
            tg_ratio = 0.09 if n == 1 else 0.07
            tile = int(min(cell_w / (L + (L - 1) * tg_ratio),
                           cell_h / (R + (R - 1) * tg_ratio)))
            if best is None or tile > best[0]:
                best = (tile, bc, br, bgap, inner, tg_ratio)
        tile, bc, br, bgap, inner, tg_ratio = best
        cap = max(20, int((64 if n == 1 else 52) * k))
        tile = max(6, min(tile, cap))
        tgap = max(1, int(round(tile * tg_ratio)))
        bw = L * tile + (L - 1) * tgap
        bh = R * tile + (R - 1) * tgap
        total_w = bc * (bw + 2 * inner) + (bc - 1) * bgap
        total_h = br * (bh + 2 * inner) + (br - 1) * bgap
        x0 = (W - total_w) // 2
        y0 = area.y + max(0, (area.h - total_h) // 2)
        self.tile, self.tile_gap = tile, tgap
        self.board_w, self.board_h, self.board_inner = bw, bh, inner
        self.board_pos = []
        for i in range(n):
            cx, cy = i % bc, i // bc
            self.board_pos.append((x0 + inner + cx * (bw + 2 * inner + bgap),
                                   y0 + inner + cy * (bh + 2 * inner + bgap)))
        self.grid_top = y0
        self.grid_bottom = y0 + total_h
        self.msg_y = max(self.hud_h + lane // 2 + 2, y0 - lane // 2) if n == 1 \
            else y0 + max(12, int(20 * k))
        self._board_cache.clear()
        self._letter_cache.clear()

    def _build_keyboard(self):
        W, H = self.width, self.height
        k = self._k()
        layout = KEYBOARDS[LANG_KEYBOARD.get(self.lang, "qwerty")]
        self.key_h = max(22, min(int(H * 0.078), 64))
        self.key_gap = max(2, int(5 * k))
        pad = max(6, int(12 * k))
        kw = int(self.key_h * 1.05)
        for ri, row in enumerate(layout):
            units = len(row) + (3.0 if ri == 2 else 0.0)
            items = len(row) + (2 if ri == 2 else 0)
            kw = min(kw, int((W - 2 * pad - (items - 1) * self.key_gap) / units))
        kw = max(12, kw)
        special = int(kw * 1.5)
        kb_h = 3 * self.key_h + 2 * self.key_gap
        self.kb_top = H - kb_h - max(4, int(8 * k))
        self.key_rects = {}
        self.enter_rect = self.del_rect = None
        y = self.kb_top
        for ri, row in enumerate(layout):
            row_w = len(row) * kw + (len(row) - 1) * self.key_gap
            if ri == 2:
                row_w += 2 * (special + self.key_gap)
            x = (W - row_w) // 2
            if ri == 2:
                self.enter_rect = pygame.Rect(x, y, special, self.key_h)
                x += special + self.key_gap
            for ch in row:
                self.key_rects[ch] = pygame.Rect(x, y, kw, self.key_h)
                x += kw + self.key_gap
            if ri == 2:
                self.del_rect = pygame.Rect(x, y, special, self.key_h)
            y += self.key_h + self.key_gap
        self._key_font = ui.font(max(10, min(int(self.key_h * 0.42), int(kw * 0.62))),
                                 bold=True)

    # ===================================================== Partie starten
    def _start(self):
        """Neue Partie im gewählten Modus (nach dem Setup oder "Nochmal")."""
        self.lang = i18n.get_language()
        opts = self._opts()
        self.length = opts["length"]
        self.hard = opts["hard"]
        self.colorblind = opts["colorblind"]
        if not has_length(self.lang, self.length):
            self.length = DEFAULT_LENGTH          # Liste fehlt -> klassisch
        self.words = words_for(self.lang, self.length)
        self.allowed = allowed_for(self.lang, self.length)
        self.data = load_data()
        self.game_over = False
        self.result = None
        self.panel_hidden = False
        self.daily_view = False
        self.points = 0
        self.score = 0
        self.solved_count = 0
        self._layout_play()
        if self.game_mode == "daily":
            self._start_daily()
        else:
            self._new_round()
        self.play_sound("click")

    def _new_round(self):
        """Neue Wörter ziehen (Endlos: nächstes Wort, Dordle/Quordle: neue Runde)."""
        pool = self.words
        n = self.n_boards
        answers = random.sample(pool, n) if len(pool) >= n else \
            [random.choice(pool) for _ in range(n)]
        self.boards = [Board(a) for a in answers]
        self._clear_round()
        self.state = PLAY

    def _clear_round(self):
        self.guesses = []
        self.current = ""
        self.reveal = None
        self.message = ""
        self.message_t = 0.0
        self.shake = 0.0
        self.pops = {}
        self._board_cache.clear()

    def _daily_slot(self):
        return "%s:%d" % (self.lang, self.length)

    def _start_daily(self):
        """Tageswort laden - samt bisherigem Fortschritt von heute."""
        today = seedrand.today_str()
        self.daily_date = today
        answer = daily_answer(self.words, self.lang, self.length, today)
        self.boards = [Board(answer)]
        self._clear_round()
        self.state = PLAY
        prog = self.data["daily"].get(self._daily_slot())
        if not (isinstance(prog, dict) and prog.get("date") == today
                and prog.get("answer") == answer):
            return
        board = self.boards[0]
        for guess in prog.get("guesses", [])[:self.rows]:
            if not (isinstance(guess, str) and len(guess) == self.length
                    and guess.isascii() and guess.isalpha()):
                break
            guess = guess.upper()
            self.guesses.append(guess)
            board.apply(guess, evaluate(guess, answer), -99.0)
            if board.solved:
                break
        if board.solved or len(self.guesses) >= self.rows:
            # Heute schon fertig: nur Ergebnis, Statistik und Countdown.
            st = get_stats(self.data, "daily", self.lang, self.length)
            self.daily_view = True
            self.result = {"won": board.solved, "tries": board.solved_at or 0,
                           "stats": st, "t0": self.anim - PANEL_DELAY,
                           "new_best": False}
            self.state = DONE

    def _save_daily(self, done):
        self.data["daily"][self._daily_slot()] = {
            "date": self.daily_date, "answer": self.boards[0].answer,
            "guesses": list(self.guesses), "done": bool(done)}
        # Alte Tage aufräumen (nur heutige Stände behalten).
        for slot in [s for s, p in self.data["daily"].items()
                     if not isinstance(p, dict) or p.get("date") != self.daily_date]:
            del self.data["daily"][slot]

    # ===================================================== Eingabe
    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup(event)
            return
        if event.kind == InputEvent.MOUSEDOWN and self.state in (PLAY, REVEAL, SOLVED) \
                and self.gear_rect.collidepoint(event.pos):
            self._to_setup()
            return
        if self.state == DONE:
            self._handle_done(event)
            return
        if self.state == SOLVED:
            if self._is_continue(event):
                self._new_round()
                self.play_sound("click")
            elif event.kind == InputEvent.KEYDOWN and event.key in ("c", "C"):
                self._share()
            return
        if self.state == REVEAL:
            return
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k == "BackSpace":
                self._backspace()
            elif k in ("Return", "KP_Enter"):
                self._submit()
            else:
                ch = event.char if event.char and len(event.char) == 1 else k
                if len(ch) == 1 and ch.isascii() and ch.isalpha():
                    self._type(ch.upper())
        elif event.kind == InputEvent.MOUSEDOWN:
            self._click_keyboard(event.pos)

    def _is_continue(self, event):
        return (event.kind == InputEvent.MOUSEDOWN
                or (event.kind == InputEvent.KEYDOWN
                    and event.key in ("Return", "KP_Enter", "space")))

    def _type(self, ch):
        if len(self.current) < self.length:
            self.pops[len(self.current)] = self.anim
            self.current += ch
            self.play_sound("click")

    def _backspace(self):
        if self.current:
            self.current = self.current[:-1]
            self.pops.pop(len(self.current), None)
            self.play_sound("move")

    def _click_keyboard(self, pos):
        if self.enter_rect and self.enter_rect.collidepoint(pos):
            self._submit()
            return
        if self.del_rect and self.del_rect.collidepoint(pos):
            self._backspace()
            return
        for ch, rc in self.key_rects.items():
            if rc.collidepoint(pos):
                self._type(ch)
                return

    def _submit(self):
        if len(self.current) != self.length:
            self._reject(t("wd.too_short", n=self.length))
            return
        if self.current not in self.allowed:
            self._reject(t("wd.not_a_word"))
            return
        if self.hard:
            problem = self._hard_check(self.current)
            if problem:
                self._reject(problem)
                return
        results = [None if b.solved else evaluate(self.current, b.answer)
                   for b in self.boards]
        step = self._reveal_step()
        self.reveal = {"guess": self.current, "results": results,
                       "t0": self.anim,
                       "end": self.anim + (self.length - 1) * step + FLIP_TIME + 0.06,
                       "ticks": 0}
        self.state = REVEAL
        self.play_sound("select")

    def _reveal_step(self):
        return 0.13 if self.length <= 5 else 0.105

    def _hard_check(self, guess):
        """Meldung, wenn der Versuch im harten Modus Hinweise ignoriert.

        Mehrere Bretter: der Versuch muss zu den Hinweisen MINDESTENS EINES
        noch offenen Bretts passen (alle zugleich wäre meist unmöglich).
        """
        open_boards = [b for b in self.boards if not b.solved]
        problems = [hard_problem(guess, b.rows) for b in open_boards]
        if not problems or any(p is None for p in problems):
            return ""
        if len(self.boards) > 1:
            return t("wd.hard_multi")
        key, values = problems[0]
        return t(key, **values)

    def _reject(self, text):
        """Rateversuch abgelehnt: Meldung zeigen und die Zeile wackeln lassen."""
        self._say(text)
        self.shake = SHAKE_TIME
        self.play_sound("hit")
        self.rumble(60)

    def _say(self, text, seconds=MESSAGE_TIME):
        self.message = text
        self.message_t = seconds

    def _finish_reveal(self):
        rv = self.reveal
        guess = rv["guess"]
        self.guesses.append(guess)
        solved_now = []
        for board, result in zip(self.boards, rv["results"]):
            if result is None:
                continue
            board.apply(guess, result, self.anim)
            if board.solved:
                solved_now.append(board)
        self.reveal = None
        self.current = ""
        self.pops = {}
        self._board_cache.clear()
        n = len(self.guesses)
        all_solved = all(b.solved for b in self.boards)
        if self.game_mode == "daily":
            self._save_daily(all_solved or n >= self.rows)
            save_data(self.data)
        for board in solved_now:
            if len(self.boards) > 1 and not all_solved:
                self.play_sound("point")
                bx, by = self.board_pos[self.boards.index(board)]
                ui.spawn_burst(bx + self.board_w // 2, by + self.board_h // 2,
                               self.palette()["correct"])
        if all_solved:
            self._round_won(n)
        elif n >= self.rows:
            self._round_lost(n)
        else:
            self.state = PLAY

    def _round_won(self, n):
        mode = self.game_mode
        if mode == "endless":
            pts = 10 + (self.rows - n) * 10          # weniger Versuche = mehr
            self.last_points = pts
            self.points += pts
            if self.counts_score:
                self.score = self.points
            self.solved_count += 1
            record_game(self.data, mode, self.lang, self.length, True, n)
            save_data(self.data)
            self.report_result(True)
            if n <= 2:
                self.ach_event("wordle_two")
            self.solved_t = self.anim
            self.state = SOLVED
            self.play_sound("win")
            return
        day = seedrand.day_index(self.daily_date) if mode == "daily" else None
        st = record_game(self.data, mode, self.lang, self.length, True, n, day)
        save_data(self.data)
        self.report_result(True)
        if mode == "daily":
            if n <= 2:
                self.ach_event("wordle_two")
            self.ach_event("wordle_daily7", st["streak"])
        elif mode == "quordle":
            self.ach_event("wordle_quordle")
        self._finish_round(True, n, st)
        ui.spawn_confetti(self.width, self.height, 70)
        self.play_sound("win")

    def _round_lost(self, n):
        mode = self.game_mode
        day = seedrand.day_index(self.daily_date) if mode == "daily" else None
        st = record_game(self.data, mode, self.lang, self.length, False, n, day)
        new_best = False
        if mode == "endless":
            key = str(self.length)
            best = self.data["best"].get(key, 0)
            best = best if isinstance(best, int) else 0
            if self.points > best:
                self.data["best"][key] = self.points
                new_best = best > 0 or self.points > 0
            if self.solved_count == 0:
                self.report_result(False)
        else:
            self.report_result(False)
        save_data(self.data)
        self._finish_round(False, n, st, new_best)
        self.play_sound("gameover")
        self.rumble(160)

    def _finish_round(self, won, n, st, new_best=False):
        self.result = {"won": won, "tries": n, "stats": st, "t0": self.anim,
                       "new_best": new_best}
        self.panel_hidden = False
        self.state = DONE
        # main.py sichert jetzt den Highscore (Endlos/5) bzw. zählt die Partie.
        self.game_over = True

    def _to_setup(self):
        """Zurück ins Setup - eine laufende Endlos-Serie wird vorher gesichert."""
        if self.state in (PLAY, REVEAL, SOLVED) and self.game_mode == "endless" \
                and self.points > 0:
            self._bank_run()
        self.game_over = False
        self.state = SETUP
        self.setup_t = self.anim
        self.score = 0
        self.points = 0
        self.play_sound("click")

    def _bank_run(self):
        """Abgebrochene Endlos-Serie: Bestwert und ggf. Highscore sichern."""
        key = str(self.length)
        best = self.data["best"].get(key, 0)
        if not isinstance(best, int) or self.points > best:
            self.data["best"][key] = self.points
            save_data(self.data)
        if self.counts_score and self.score > 0:
            try:
                import highscore
                import achievements
                hs, _ = highscore.update_highscore(self.highscore_key, self.score)
                achievements.on_highscore(self.highscore_key, hs)
            except Exception:
                pass

    def _handle_done(self, event):
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("Return", "KP_Enter"):
                self._done_primary()
            elif k in ("c", "C"):
                self._share()
            elif k in ("s", "S", "BackSpace"):
                self._to_setup()
            elif k == "space":
                self.panel_hidden = not self.panel_hidden
                self.play_sound("move")
        elif event.kind == InputEvent.MOUSEDOWN:
            if self.panel_hidden:
                self.panel_hidden = False
                return
            for rc, action in getattr(self, "_done_buttons", []):
                if rc.collidepoint(event.pos):
                    action()
                    return
            panel = getattr(self, "_done_panel", None)
            if panel is not None and not panel.collidepoint(event.pos):
                self.panel_hidden = True

    def _done_primary(self):
        if self.game_mode == "daily":
            if self.daily_date != seedrand.today_str():
                self.game_over = False
                self.new_round_result()
                self._start()                 # neuer Tag, neues Wort
            else:
                self._to_setup()
            return
        self.game_over = False
        self.new_round_result()
        self._start()

    def _share(self):
        boards = [([res for _, res in b.rows], b.solved_at) for b in self.boards]
        text = share_text(self.game_mode, self.lang, self.length, boards,
                          hard=self.hard, colorblind=self.colorblind,
                          date=self.daily_date)
        if copy_to_clipboard(text):
            self._say(t("wd.copied"))
            self.play_sound("powerup")
        else:
            self._say(t("wd.copy_failed"))
            self.play_sound("hit")

    # ----- Setup-Eingabe -------------------------------------------------
    def _handle_setup(self, event):
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("Return", "KP_Enter"):
                self._start()
            elif k in ("4", "5", "6", "7"):
                self._set_length(int(k))
            elif k in ("Left", "Right") or self.is_action(k, "left") \
                    or self.is_action(k, "right"):
                step = -1 if (k == "Left" or self.is_action(k, "left")) else 1
                idx = LENGTHS.index(self.length) if self.length in LENGTHS else 1
                self._set_length(LENGTHS[max(0, min(len(LENGTHS) - 1, idx + step))])
                self.setup_focus = 0
            elif k in ("h", "H"):
                self._toggle("hard")
            elif k in ("f", "F"):
                self._toggle("colorblind")
            elif k == "Up" or self.is_action(k, "up"):
                self.setup_focus = (self.setup_focus - 1) % 4
                self.play_sound("move")
            elif k == "Down" or self.is_action(k, "down"):
                self.setup_focus = (self.setup_focus + 1) % 4
                self.play_sound("move")
            elif k == "space":
                if self.setup_focus == 1:
                    self._toggle("hard")
                elif self.setup_focus == 2:
                    self._toggle("colorblind")
                elif self.setup_focus == 3:
                    self._start()
        elif event.kind == InputEvent.MOUSEDOWN:
            for i, rc in enumerate(self.su_seg):
                if rc.collidepoint(event.pos):
                    self.setup_focus = 0
                    self._set_length(LENGTHS[i])
                    return
            if self.su_hard.collidepoint(event.pos):
                self.setup_focus = 1
                self._toggle("hard")
            elif self.su_cb.collidepoint(event.pos):
                self.setup_focus = 2
                self._toggle("colorblind")
            elif self.su_start.collidepoint(event.pos):
                self.setup_focus = 3
                self._start()
        elif event.kind == InputEvent.MOUSEMOVE:
            for i, rc in enumerate((pygame.Rect(0, 0, 0, 0), self.su_hard,
                                    self.su_cb, self.su_start)):
                if rc.collidepoint(event.pos):
                    self.setup_focus = i
            if any(rc.collidepoint(event.pos) for rc in self.su_seg):
                self.setup_focus = 0

    def _set_length(self, length):
        if length == self.length or length not in LENGTHS:
            return
        self.length = length
        self._save_opt("length", length)
        self._layout_play()
        self.play_sound("click")

    def _toggle(self, key):
        value = not getattr(self, key)
        setattr(self, key, value)
        self._save_opt(key, value)
        self._board_cache.clear()
        self.play_sound("click")

    # ===================================================== Update
    def update(self, dt):
        self._updated = True
        self._tick(dt)

    def _tick(self, dt):
        self.anim += dt
        if self.message_t > 0:
            self.message_t = max(0.0, self.message_t - dt)
            if self.message_t == 0:
                self.message = ""
        if self.shake > 0:
            self.shake = max(0.0, self.shake - dt)
        rv = self.reveal
        if self.state == REVEAL and rv is not None:
            # Ein leises Klacken je umgedrehter Spalte.
            col = int((self.anim - rv["t0"]) / self._reveal_step())
            if col >= rv["ticks"] and rv["ticks"] < self.length:
                rv["ticks"] = col + 1
                self.play_sound("click")
            if self.anim >= rv["end"]:
                self._finish_reveal()

    # ===================================================== Zeichnen
    def draw(self):
        # Nach Game Over ruft main.py update() nicht mehr auf - die Uhr läuft
        # dann hier weiter, damit Ergebnis-Panel und Balken animiert bleiben.
        now = time.perf_counter()
        if not self._updated and self._last_draw is not None and self.game_over:
            self._tick(min(0.1, now - self._last_draw))
        self._updated = False
        self._last_draw = now
        s = self.surface
        if self.state == SETUP:
            self._draw_setup(s)
            return
        ui.draw_background(s, self.width, self.height, stars=False, aurora=True)
        self._draw_hud(s)
        for i, board in enumerate(self.boards):
            self._draw_board(s, i, board)
        self._draw_keyboard(s)
        if self.state == SOLVED:
            self._draw_solved(s)
        if self.state == DONE:
            if self.panel_hidden:
                self._draw_hidden_hint(s)
            else:
                self._draw_result(s)
        if self.message:
            self._draw_message(s)

    # ----- HUD -----------------------------------------------------------
    def _draw_hud(self, s):
        W, h = self.width, self.hud_h
        pygame.draw.rect(s, ui.PANEL, (0, 0, W, h))
        pygame.draw.line(s, ui.BORDER, (0, h), (W, h))
        cy = h // 2
        third = (W - self.hud_h) // 3 - 10
        mode = self.game_mode
        if mode == "endless":
            left = t("wd.score", n=self.points)
            mid = t("wd.solved_n", n=self.solved_count)
        elif mode == "daily":
            left = t("wd.hud.daily", d=self._short_date())
            st = get_stats(self.data, "daily", self.lang, self.length)
            mid = t("wd.hud.streak", n=shown_streak(st, "daily"))
        else:
            left = t("wd.mode." + mode)
            mid = t("wd.boards_solved", a=sum(b.solved for b in self.boards),
                    b=len(self.boards))
        px = self._hud.get_height()
        self._text(s, left, int(px * 0.95), self.accent, third, bold=True,
                   midleft=(12, cy))
        self._text(s, mid, int(px * 0.8), ui.TEXT_DIM, third,
                   center=((W - self.hud_h) // 2 + 6, cy))
        tries = min(len(self.guesses) + (1 if self.state in (PLAY, REVEAL) else 0),
                    self.rows)
        self._text(s, t("wd.tries", a=tries, b=self.rows), int(px * 0.8),
                   ui.TEXT_DIM, third, midright=(self.gear_rect.x - 8, cy))
        self._draw_gear(s, self.gear_rect)

    def _short_date(self):
        d = self.daily_date or seedrand.today_str()
        return "%s.%s." % (d[8:10], d[5:7])

    def _draw_gear(self, s, rc):
        """Regler-Symbol (drei Schieber) = zurück ins Setup."""
        col = ui.TEXT_DIM
        pygame.draw.rect(s, ui.BTN, rc, border_radius=max(4, rc.h // 4))
        x0, x1 = rc.x + rc.w * 0.24, rc.right - rc.w * 0.24
        th = max(1, rc.h // 14)
        for i, f in enumerate((0.3, 0.5, 0.7)):
            y = int(rc.y + rc.h * f)
            pygame.draw.line(s, col, (int(x0), y), (int(x1), y), th)
            kx = int(x0 + (x1 - x0) * (0.7, 0.3, 0.55)[i])
            pygame.draw.circle(s, col, (kx, y), max(2, rc.h // 10))

    # ----- Kacheln -------------------------------------------------------
    def _state_color(self, state):
        if state == "absent":
            return COL_ABSENT
        return self.palette().get(state)

    def _empty_fill(self):
        return ui.mix(ui.BG_BOTTOM, ui.PANEL, 0.55)

    def _letter(self, size, ch):
        key = (size, ch)
        img = self._letter_cache.get(key)
        if img is None:
            f = ui.font(max(7, int(size * 0.58)), bold=True)
            img = f.render(ch, True, COL_LETTER)
            self._letter_cache[key] = img
        return img

    def _draw_tile(self, s, x, y, ch, state, border=None, scale_y=1.0,
                   scale=1.0, dy=0, alpha_fill=None):
        size = self.tile
        fill = self._state_color(state) if state else (alpha_fill or self._empty_fill())
        w = max(1, int(size * scale))
        h = max(1, int(size * scale * scale_y))
        rc = pygame.Rect(0, 0, w, h)
        rc.center = (x + size // 2, y + size // 2 + dy)
        radius = max(2, size // 9)
        pygame.draw.rect(s, fill, rc, border_radius=radius)
        if not state:
            pygame.draw.rect(s, border or ui.BORDER, rc, max(1, size // 24),
                             border_radius=radius)
        if ch and h > 3:
            img = self._letter(size, ch)
            if scale_y < 0.99 or abs(scale - 1.0) > 0.01:
                img = pygame.transform.smoothscale(
                    img, (max(1, int(img.get_width() * scale)),
                          max(1, int(img.get_height() * scale * scale_y))))
            s.blit(img, img.get_rect(center=rc.center))

    def _tile_xy(self, bi, r, c):
        bx, by = self.board_pos[bi]
        step = self.tile + self.tile_gap
        return bx + c * step, by + r * step

    def _bouncing(self, board):
        return (board.solved and board.solve_t is not None
                and self.anim - board.solve_t < BOUNCE_TIME + self.length * 0.08)

    def _board_static(self, bi, board):
        """Fertige Zeilen und leere Kacheln eines Bretts als gecachte Fläche."""
        bouncing = self._bouncing(board)
        key = (bi, self.tile, self.tile_gap, self.length, self.rows,
               len(board.rows), board.solved_at, bouncing, self.colorblind,
               ui.theme_name(), tuple(ui.BORDER), tuple(ui.PANEL))
        surf = self._board_cache.get(bi)
        if surf is not None and surf[0] == key:
            return surf[1]
        img = pygame.Surface((self.board_w, self.board_h), pygame.SRCALPHA)
        live = None if board.solved else len(board.rows)
        step = self.tile + self.tile_gap
        dim_fill = ui.mix(ui.BG_BOTTOM, ui.PANEL, 0.3)
        for r in range(self.rows):
            if r == live:
                continue
            for c in range(self.length):
                x, y = c * step, r * step
                if r < len(board.rows):
                    if bouncing and r == board.solved_at - 1:
                        continue
                    word, res = board.rows[r]
                    self._draw_tile(img, x, y, word[c], res[c])
                elif board.solved:
                    self._draw_tile(img, x, y, "", None, border=ui.mix(dim_fill, ui.BORDER, 0.4),
                                    alpha_fill=dim_fill)
                else:
                    self._draw_tile(img, x, y, "", None)
        self._board_cache[bi] = (key, img)
        return img

    def _draw_board(self, s, bi, board):
        bx, by = self.board_pos[bi]
        if len(self.boards) > 1:
            pad = self.board_inner
            frame = pygame.Rect(bx - pad, by - pad, self.board_w + 2 * pad,
                                self.board_h + 2 * pad)
            border = None
            if board.solved:
                border = self.palette()["correct"]
            elif self.state == DONE:
                border = COL_LOSE
            ui.draw_panel(s, frame, radius=max(6, pad * 2), shadow=False,
                          border=border)
            if border is not None:
                pygame.draw.rect(s, border, frame, max(1, pad // 3),
                                 border_radius=max(6, pad * 2))
        s.blit(self._board_static(bi, board), (bx, by))
        step = self.tile + self.tile_gap
        # Hüpfende Siegerzeile
        if self._bouncing(board):
            r = board.solved_at - 1
            word, res = board.rows[r]
            for c in range(self.length):
                p = (self.anim - board.solve_t - c * 0.08) / BOUNCE_TIME
                dy = -int(math.sin(math.pi * p) * self.tile * 0.35) if 0 < p < 1 else 0
                x, y = self._tile_xy(bi, r, c)
                self._draw_tile(s, x, y, word[c], res[c], dy=dy)
        if board.solved:
            return
        r = len(board.rows)
        if r >= self.rows:
            if self.state == DONE:
                self._draw_answer_pill(s, bi, board)
            return
        rv = self.reveal
        if rv is not None and rv["results"][bi] is not None:
            # Kachel für Kachel umdrehen: erst zuklappen, dann mit Farbe auf.
            step_t = self._reveal_step()
            for c in range(self.length):
                x, y = self._tile_xy(bi, r, c)
                p = (self.anim - rv["t0"] - c * step_t) / FLIP_TIME
                if p <= 0:
                    self._draw_tile(s, x, y, rv["guess"][c], None, border=ui.TEXT_DIM)
                elif p < 0.5:
                    self._draw_tile(s, x, y, rv["guess"][c], None, border=ui.TEXT_DIM,
                                    scale_y=max(0.02, math.cos(p * math.pi)))
                else:
                    sy = max(0.02, -math.cos(p * math.pi)) if p < 1 else 1.0
                    self._draw_tile(s, x, y, rv["guess"][c], rv["results"][bi][c],
                                    scale_y=sy)
            return
        if self.state == DONE:
            # Nicht gelöst: die Lösung in der nächsten Zeile zeigen.
            for c in range(self.length):
                x, y = self._tile_xy(bi, r, c)
                self._draw_tile(s, x, y, board.answer[c], None, border=COL_LOSE,
                                alpha_fill=ui.mix(self._empty_fill(), COL_LOSE, 0.25))
            return
        shake = int(self.tile * 0.14 * math.sin(self.shake * 48)) if self.shake > 0 else 0
        for c in range(self.length):
            x, y = self._tile_xy(bi, r, c)
            ch = self.current[c] if c < len(self.current) else ""
            scale = 1.0
            if ch and c in self.pops:
                p = (self.anim - self.pops[c]) / POP_TIME
                if 0 <= p < 1:
                    scale = 1.0 + 0.12 * math.sin(math.pi * p)
            border = ui.TEXT_DIM if ch else None
            self._draw_tile(s, x + shake, y, ch, None, border=border, scale=scale)

    def _draw_answer_pill(self, s, bi, board):
        """Nicht gelöstes Brett ohne freie Zeile: Lösung als Etikett unten."""
        bx, by = self.board_pos[bi]
        px = max(10, min(int(self.tile * 0.62), self._small_b.get_height()))
        f = self._fit(board.answer, self.board_w - 8, px, True)
        img = f.render(board.answer, True, COL_LETTER)
        rc = img.get_rect(center=(bx + self.board_w // 2, by + self.board_h))
        box = rc.inflate(max(12, px), max(4, px // 3))
        pygame.draw.rect(s, COL_LOSE, box, border_radius=box.h // 2)
        s.blit(img, rc)

    # ----- Tastatur ------------------------------------------------------
    def _draw_keyboard(self, s):
        radius = max(3, self.key_h // 8)
        n = len(self.boards)
        for ch, rc in self.key_rects.items():
            pygame.draw.rect(s, ui.BTN, rc, border_radius=radius)
            states = [b.keys.get(ch) for b in self.boards]
            if n == 1:
                if states and states[0]:
                    pygame.draw.rect(s, self._state_color(states[0]), rc,
                                     border_radius=radius)
            elif n == 2:
                half = rc.w // 2
                for i, st in enumerate(states):
                    if not st:
                        continue
                    part = pygame.Rect(rc.x + i * half, rc.y,
                                       half if i == 0 else rc.w - half, rc.h)
                    kw = dict(border_top_left_radius=radius if i == 0 else 0,
                              border_bottom_left_radius=radius if i == 0 else 0,
                              border_top_right_radius=radius if i == 1 else 0,
                              border_bottom_right_radius=radius if i == 1 else 0)
                    pygame.draw.rect(s, self._state_color(st), part, **kw)
            else:
                hw, hh = rc.w // 2, rc.h // 2
                for i, st in enumerate(states[:4]):
                    if not st:
                        continue
                    cx, cy = i % 2, i // 2
                    part = pygame.Rect(rc.x + cx * hw, rc.y + cy * hh,
                                       hw if cx == 0 else rc.w - hw,
                                       hh if cy == 0 else rc.h - hh)
                    kw = dict(border_top_left_radius=radius if i == 0 else 0,
                              border_top_right_radius=radius if i == 1 else 0,
                              border_bottom_left_radius=radius if i == 2 else 0,
                              border_bottom_right_radius=radius if i == 3 else 0)
                    pygame.draw.rect(s, self._state_color(st), part, **kw)
            if n == 1 and not states[0]:
                pygame.draw.rect(s, ui.BORDER, rc, 1, border_radius=radius)
            img = self._key_font.render(ch, True, COL_LETTER)
            sh = self._key_font.render(ch, True, (0, 0, 0))
            sh.set_alpha(90)
            r = img.get_rect(center=rc.center)
            s.blit(sh, r.move(1, 1))
            s.blit(img, r)
        for rc, kind in ((self.enter_rect, "enter"), (self.del_rect, "del")):
            if rc is None:
                continue
            pygame.draw.rect(s, ui.BTN_SEL, rc, border_radius=radius)
            if kind == "enter":
                self._text(s, t("wd.enter"), max(9, self.key_h * 3 // 8), ui.TEXT,
                           rc.w - 6, bold=True, center=rc.center)
            else:
                self._draw_backspace(s, rc)

    def _draw_backspace(self, s, rc):
        """Rücktaste als gezeichnetes Symbol (Pfeil-Etikett mit x)."""
        h = max(8, int(rc.h * 0.36))
        w = int(h * 1.5)
        x0 = rc.centerx - w // 2
        y0 = rc.centery - h // 2
        pts = [(x0, rc.centery), (x0 + h // 2, y0), (x0 + w, y0),
               (x0 + w, y0 + h), (x0 + h // 2, y0 + h)]
        th = max(1, h // 8)
        pygame.draw.polygon(s, ui.TEXT, pts, th)
        cx = x0 + h // 2 + (w - h // 2) // 2
        d = max(2, h // 5)
        pygame.draw.line(s, ui.TEXT, (cx - d, rc.centery - d), (cx + d, rc.centery + d), th)
        pygame.draw.line(s, ui.TEXT, (cx - d, rc.centery + d), (cx + d, rc.centery - d), th)

    # ----- Meldungen / Banner ---------------------------------------------
    def _draw_message(self, s):
        """Kurze Meldung ("Kein Wort in der Liste", "Kopiert!")."""
        alpha = min(1.0, self.message_t / 0.35)
        f = self._fit(self.message, self.width - 40, self._small_b.get_height(), True)
        img = f.render(self.message, True, ui.TEXT)
        rc = img.get_rect(center=(self.width // 2, self.msg_y))
        if self.state == DONE and not self.panel_hidden:
            panel = getattr(self, "_done_panel", None)
            if panel is not None:
                rc.center = (self.width // 2, max(rc.h, panel.y - rc.h))
        box = rc.inflate(26, 12)
        panel = pygame.Surface(box.size, pygame.SRCALPHA)
        pygame.draw.rect(panel, (*ui.PANEL_LIGHT, int(240 * alpha)), panel.get_rect(),
                         border_radius=box.h // 2)
        s.blit(panel, box.topleft)
        pygame.draw.rect(s, ui.mix(ui.BORDER_LIGHT, self.accent, 0.3), box, 1,
                         border_radius=box.h // 2)
        img.set_alpha(int(255 * alpha))
        s.blit(img, rc)

    def _draw_solved(self, s):
        """Endlos: Wort gelöst - kleiner Banner mit Punkten über dem Brett."""
        p = (self.anim - self.solved_t - 0.35) / 0.25
        if p <= 0:
            return
        alpha = min(1.0, p)
        k = self._k()
        w = min(self.width - 24, int(440 * k))
        h = max(56, int(88 * k))
        # Nicht über die gelöste Zeile legen: steht sie oben, kommt der
        # Banner nach unten ins Brett und umgekehrt.
        board = self.boards[0]
        row_y = self._tile_xy(0, (board.solved_at or 1) - 1, 0)[1]
        if row_y + self.tile // 2 < self.grid_top + self.board_h // 2:
            y = self.grid_bottom - h
        else:
            y = self.grid_top
        rc = pygame.Rect((self.width - w) // 2, y, w, h)
        rc.y -= int((1 - alpha) * 12)
        ui.draw_panel(s, rc, accent_top=self.palette()["correct"])
        col = self.palette()["correct"]
        self._text(s, t("wd.solved", n=self.last_points), self._big.get_height(), col,
                   w - 20, bold=True, center=(rc.centerx, rc.y + h * 0.38))
        self._text(s, t("wd.next") + "   ·   " + t("wd.share_key"),
                   self._tiny.get_height(), ui.TEXT_DIM, w - 20,
                   center=(rc.centerx, rc.y + h * 0.76))

    def _draw_hidden_hint(self, s):
        text = t("wd.result.show")
        f = self._fit(text, self.width - 30, self._small.get_height())
        img = f.render(text, True, ui.TEXT)
        rc = img.get_rect(midbottom=(self.width // 2, self.kb_top - 4))
        box = rc.inflate(22, 10)
        ui.draw_panel(s, box, radius=box.h // 2, shadow=False)
        s.blit(img, rc)

    # ----- Statistik-Block (Setup + Ergebnis) ----------------------------
    def _draw_stats_block(self, s, rect, st, highlight=None, grow=1.0):
        """Vier Kennzahlen + Balkendiagramm der Versuchsverteilung in 'rect'."""
        k = self._k()
        played = st["played"]
        rate = int(round(100.0 * st["won"] / played)) if played else 0
        values = [(played, "wd.stats.played"), (rate, "wd.stats.winrate"),
                  (shown_streak(st, self.game_mode), "wd.stats.streak"),
                  (st["best"], "wd.stats.best")]
        num_px, lab_px, per_row = self._stats_metrics(rect.w, rect.h)
        col_w = rect.w // per_row
        y = rect.y
        for i, (v, key) in enumerate(values):
            cx = rect.x + col_w * (i % per_row) + col_w // 2
            cy = y + (i // per_row) * (num_px + lab_px + 6)
            self._text(s, str(v), num_px, ui.TEXT, col_w - 4, bold=True,
                       midtop=(cx, cy))
            self._text(s, t(key), lab_px, ui.TEXT_DIM, col_w - 4,
                       midtop=(cx, cy + num_px + 2))
        y += (4 // per_row) * (num_px + lab_px + 6) - 6 + max(6, int(12 * k))
        self._text(s, t("wd.stats.dist"), max(9, int(12 * k)), ui.TEXT_DIM, rect.w,
                   bold=True, midtop=(rect.centerx, y))
        y += max(9, int(12 * k)) + max(4, int(8 * k))
        first = self.n_boards - 1                 # Quordle: frühestens nach 4
        dist = st["dist"][first:]
        bars = len(dist)
        avail = rect.bottom - y
        bar_gap = max(2, int(4 * k))
        bar_h = max(6, min(int(20 * k), (avail - (bars - 1) * bar_gap) // max(1, bars)))
        label_w = max(12, int(18 * k))
        top_v = max([1] + dist)
        palette = self.palette()
        for i, v in enumerate(dist):
            tries = first + i + 1
            by = y + i * (bar_h + bar_gap)
            if by + bar_h > rect.bottom + 2:
                break
            self._text(s, str(tries), max(9, bar_h - 2), ui.TEXT_DIM, label_w,
                       bold=True, midleft=(rect.x, by + bar_h // 2))
            full = rect.w - label_w - 6
            minw = max(18, int(24 * k))
            bw = int(minw + (full - minw) * (v / top_v) * grow)
            bar = pygame.Rect(rect.x + label_w + 6, by, bw, bar_h)
            fill = palette["correct"] if tries == highlight else ui.BTN_SEL
            pygame.draw.rect(s, fill, bar, border_radius=max(2, bar_h // 4))
            self._text(s, str(v), max(9, bar_h - 3), COL_LETTER, bw - 6, bold=True,
                       midright=(bar.right - 5, bar.centery))

    def _stats_metrics(self, width, height=9999):
        """(Zahl-Größe, Beschriftungs-Größe, Kennzahlen je Zeile)."""
        k = self._k()
        num_px = max(14, min(int(26 * k), height // 7))
        lab_px = max(9, int(11 * k))
        col_w = width // 4 - 4
        labels = ("wd.stats.played", "wd.stats.winrate", "wd.stats.streak",
                  "wd.stats.best")
        wide = all(self._fits(t(key), col_w, lab_px) for key in labels)
        return num_px, lab_px, 4 if wide else 2

    def _stats_height(self, width, bars):
        """Natürliche Höhe des Statistik-Blocks (für die Panel-Größe)."""
        k = self._k()
        num_px, lab_px, per_row = self._stats_metrics(width)
        head = (4 // per_row) * (num_px + lab_px + 6) - 6 + max(6, int(12 * k))
        dist = max(9, int(12 * k)) + max(4, int(8 * k))
        return head + dist + bars * max(10, int(20 * k)) + (bars - 1) * max(2, int(4 * k))

    def _dim_surface(self, alpha):
        """Abdunkelnde Vollbild-Fläche (einmal je Größe gebaut)."""
        key = (self.width, self.height)
        if getattr(self, "_dim_key", None) != key:
            self._dim_key = key
            self._dim = pygame.Surface(key, pygame.SRCALPHA)
            self._dim.fill((6, 8, 14, 120))
        self._dim.set_alpha(int(255 * max(0.0, min(1.0, alpha))))
        return self._dim

    # ----- Ergebnis-Panel --------------------------------------------------
    def _draw_result(self, s):
        res = self.result
        if res is None:
            return
        p = (self.anim - res["t0"] - PANEL_DELAY) / 0.3
        if p <= 0:
            self._done_buttons = []
            return
        appear = min(1.0, p)
        W, H = self.width, self.height
        k = self._k()
        s.blit(self._dim_surface(appear), (0, 0))
        pw = min(W - 20, int(470 * k))
        inner = pw - 2 * max(12, int(22 * k))
        mode = self.game_mode
        sub_px = max(11, int(15 * k))
        btn_h = max(26, int(40 * k))
        hint_px = max(9, int(11 * k))
        big_px = max(18, int(30 * k))
        need = (max(8, int(16 * k)) + big_px + max(4, int(8 * k)) + sub_px
                + max(4, int(6 * k)) + max(2, int(6 * k))
                + self._stats_height(inner, self.rows - self.n_boards + 1)
                + max(4, int(8 * k)) + btn_h + max(4, int(8 * k)) + hint_px
                + max(8, int(14 * k)))
        if mode == "endless":
            need += max(10, int(13 * k)) + max(4, int(6 * k))
        if mode == "daily":
            need += sub_px + max(4, int(8 * k))
        # Endlos/5: main.py blendet unten den Highscore-Banner ein - das
        # Panel bleibt darüber.
        reserve = BANNER_SPACE if self.show_highscore_banner else 0
        ph = min(H - 16 - reserve, need)
        panel = pygame.Rect(0, 0, pw, ph)
        panel.center = (W // 2, (H - reserve) // 2 + int((1 - appear) * 20))
        self._done_panel = panel
        col = self.palette()["correct"] if res["won"] else COL_LOSE
        ui.draw_panel(s, panel, accent_top=col)
        y = panel.y + max(8, int(16 * k))
        title = t("wd.result.win") if res["won"] else t("wd.result.lose")
        if mode == "endless":
            title = t("wd.gameover")
        self._text(s, title, big_px, col, inner, bold=True, midtop=(panel.centerx, y))
        y += big_px + max(4, int(8 * k))
        if mode == "endless":
            sub = t("wd.answer_was", w=self.boards[0].answer) + "  ·  " + \
                t("wd.result.points", n=self.points)
        elif len(self.boards) == 1:
            sub = t("wd.result.in", n=res["tries"], m=self.rows) if res["won"] \
                else t("wd.answer_was", w=self.boards[0].answer)
        else:
            sub = t("wd.result.words", w=" · ".join(b.answer for b in self.boards))
        self._text(s, sub, sub_px, ui.TEXT, inner, midtop=(panel.centerx, y))
        y += sub_px + max(4, int(6 * k))
        if mode == "endless":
            best = self.data["best"].get(str(self.length), 0)
            line = t("wd.new_best") if res.get("new_best") else \
                t("wd.result.best", n=best if isinstance(best, int) else 0)
            self._text(s, line, max(10, int(13 * k)), ui.GOLD, inner,
                       midtop=(panel.centerx, y))
            y += max(10, int(13 * k)) + max(4, int(6 * k))
        y += max(2, int(6 * k))
        # Unten: Countdown, Knöpfe, Hinweis
        bottom = panel.bottom - max(8, int(14 * k)) - hint_px - max(4, int(8 * k)) - btn_h
        if mode == "daily":
            bottom -= sub_px + max(4, int(8 * k))
        grow = min(1.0, max(0.0, (self.anim - res["t0"] - PANEL_DELAY) / 0.7))
        grow = 1 - (1 - grow) ** 3
        highlight = res["tries"] if res["won"] else None
        block = pygame.Rect(panel.centerx - inner // 2, y, inner, bottom - y - max(4, int(8 * k)))
        self._draw_stats_block(s, block, res["stats"], highlight, grow)
        y = bottom
        if mode == "daily":
            if self.daily_date != seedrand.today_str():
                text = t("wd.daily.new")
            else:
                text = t("wd.daily.next", t=format_hms(seconds_to_midnight()))
            self._text(s, text, sub_px, ui.GOLD, inner, bold=True,
                       midtop=(panel.centerx, y))
            y += sub_px + max(4, int(8 * k))
        # Knöpfe
        gap = max(6, int(10 * k))
        buttons = [(t("wd.share"), self._share)]
        if mode == "daily":
            buttons.append((t("wd.setup"), self._to_setup))
        else:
            buttons.append((t("wd.again"), self._done_primary))
            buttons.append((t("wd.setup"), self._to_setup))
        bw = (inner - (len(buttons) - 1) * gap) // len(buttons)
        bfont = self._fit(max((b[0] for b in buttons), key=len), bw - 12,
                          max(11, int(16 * k)), True)
        self._done_buttons = []
        for i, (label, action) in enumerate(buttons):
            rc = pygame.Rect(panel.centerx - inner // 2 + i * (bw + gap), y, bw, btn_h)
            ui.draw_button(s, rc, label, bfont, selected=(i == len(buttons) - 2
                                                          or len(buttons) == 2 and i == 0),
                           accent=self.accent)
            self._done_buttons.append((rc, action))
        y += btn_h + max(4, int(8 * k))
        hint = t("wd.result.hint_daily") if mode == "daily" else t("wd.result.hint")
        self._text(s, self._fit_parts(hint, inner, hint_px), hint_px, ui.TEXT_FAINT, inner,
                   midtop=(panel.centerx, y))

    # ----- Setup -----------------------------------------------------------
    def _draw_setup(self, s):
        W, H = self.width, self.height
        k = self._k()
        ui.draw_background(s, W, H)
        self._draw_title_tiles(s)
        self._text(s, t("wd.mode." + self.game_mode) + "  ·  "
                   + t("wd.setup.sub." + self.game_mode),
                   self._small.get_height(), ui.TEXT_DIM, W - 30,
                   midtop=(W // 2, self.su_sub_y))
        palette = self.palette()
        lab_px = self._small.get_height()
        col_w = self.su_start.w
        self._text(s, t("wd.setup.length"), lab_px, ui.TEXT, col_w, bold=True,
                   midleft=(self.su_seg[0].x + 2, self.su_label_y + lab_px // 2))
        # Wortlänge: vier Knöpfe
        for i, rc in enumerate(self.su_seg):
            on = LENGTHS[i] == self.length
            fill = palette["correct"] if on else ui.BTN
            pygame.draw.rect(s, fill, rc, border_radius=max(4, rc.h // 5))
            if not on:
                pygame.draw.rect(s, ui.BORDER, rc, 1, border_radius=max(4, rc.h // 5))
            if self.setup_focus == 0 and on:
                pygame.draw.rect(s, ui.TEXT, rc.inflate(4, 4), 2,
                                 border_radius=max(4, rc.h // 5) + 2)
            num_px = max(14, int(rc.h * 0.5))
            self._text(s, str(LENGTHS[i]), num_px, COL_LETTER if on else ui.TEXT,
                       rc.w - 4, bold=True, center=(rc.centerx, rc.centery))
        # Schalter
        self._draw_toggle(s, self.su_hard, "hard", t("wd.setup.hard"),
                          t("wd.setup.hard_desc"), self.hard, self.setup_focus == 1)
        self._draw_toggle(s, self.su_cb, "colorblind", t("wd.setup.colorblind"),
                          t("wd.setup.colorblind_desc"), self.colorblind,
                          self.setup_focus == 2, preview=True)
        ui.draw_button(s, self.su_start, t("common.start"),
                       self._fit(t("common.start"), self.su_start.w - 20,
                                 max(13, int(20 * k)), True),
                       selected=self.setup_focus == 3, accent=self.accent)
        self._draw_setup_stats(s)
        hint = self._fit_parts(t("wd.setup.hint"), W - 20, max(10, int(13 * k)))
        ui.draw_footer(s, W, H, hint, self._fit(hint, W - 20, max(10, int(13 * k))))

    def _draw_title_tiles(self, s):
        """WORDLE als Kachelreihe, die sich beim Öffnen nacheinander umdreht."""
        word = "WORDLE"
        size = self.su_tile
        gap = max(3, size // 9)
        total = len(word) * size + (len(word) - 1) * gap
        x0 = (self.width - total) // 2
        palette = self.palette()
        states = ["correct", "absent", "present", "correct", "absent", "correct"]
        f = ui.font(max(10, int(size * 0.58)), bold=True)
        for i, ch in enumerate(word):
            p = (self.anim - self.setup_t - 0.08 * i) / FLIP_TIME
            sy = 1.0
            state = states[i]
            if p < 0.5:
                state = None
                sy = max(0.05, math.cos(max(0.0, p) * math.pi)) if p > 0 else 1.0
            elif p < 1:
                sy = max(0.05, -math.cos(p * math.pi))
            bob = math.sin(self.anim * 2.2 + i * 0.7) * size * 0.04
            rc = pygame.Rect(0, 0, size, max(1, int(size * sy)))
            rc.center = (x0 + i * (size + gap) + size // 2, self.su_title_y + size // 2 + int(bob))
            fill = (COL_ABSENT if state == "absent" else palette[state]) if state \
                else self._empty_fill()
            pygame.draw.rect(s, fill, rc, border_radius=max(3, size // 9))
            if not state:
                pygame.draw.rect(s, ui.BORDER_LIGHT, rc, 2, border_radius=max(3, size // 9))
            img = f.render(ch, True, COL_LETTER)
            if sy < 0.99:
                img = pygame.transform.smoothscale(img, (img.get_width(),
                                                         max(1, int(img.get_height() * sy))))
            s.blit(img, img.get_rect(center=rc.center))

    def _draw_toggle(self, s, rc, key, label, desc, value, focus, preview=False):
        k = self._k()
        radius = max(6, rc.h // 5)
        pygame.draw.rect(s, ui.BTN_SEL if focus else ui.BTN, rc, border_radius=radius)
        pygame.draw.rect(s, self.accent if focus else ui.BORDER, rc, 1,
                         border_radius=radius)
        # Schalter rechts (Knopf gleitet weich hin und her)
        sw_h = max(16, int(rc.h * 0.42))
        sw_w = int(sw_h * 1.9)
        sw = pygame.Rect(rc.right - sw_w - max(8, int(12 * k)),
                         rc.centery - sw_h // 2, sw_w, sw_h)
        v = self._toggle_anim.get(key, 1.0 if value else 0.0)
        v += ((1.0 if value else 0.0) - v) * 0.3
        self._toggle_anim[key] = v
        on_col = self.palette()["correct"]
        pygame.draw.rect(s, ui.mix(ui.BORDER_LIGHT, on_col, v), sw, border_radius=sw_h // 2)
        knob_r = sw_h // 2 - 2
        kx = int(sw.x + sw_h // 2 + (sw_w - sw_h) * v)
        pygame.draw.circle(s, COL_LETTER, (kx, sw.centery), knob_r)
        text_x = rc.x + max(8, int(14 * k))
        right = sw.x - max(6, int(10 * k))
        if preview:
            ps = max(8, int(sw_h * 0.8))
            for i, state in enumerate(("correct", "present")):
                pr = pygame.Rect(right - (2 - i) * (ps + 3), rc.centery - ps // 2, ps, ps)
                pygame.draw.rect(s, self.palette()[state], pr, border_radius=max(2, ps // 5))
            right -= 2 * (ps + 3) + 4
        max_w = right - text_x
        lab_px = self._small_b.get_height()
        desc_px = max(9, int(11 * k))
        if self._fits(desc, max_w, desc_px) and rc.h >= lab_px + desc_px + 8:
            self._text(s, label, lab_px, ui.TEXT, max_w, bold=True,
                       midleft=(text_x, rc.y + rc.h * 0.36))
            self._text(s, desc, desc_px, ui.TEXT_DIM, max_w,
                       midleft=(text_x, rc.y + rc.h * 0.72))
        else:
            # Zu eng (kleine Auflösung, lange Sprache): nur die Überschrift.
            self._text(s, label, lab_px, ui.TEXT, max_w, bold=True,
                       midleft=(text_x, rc.centery))

    def _draw_setup_stats(self, s):
        k = self._k()
        rect = self.su_stats_rect
        ui.draw_panel(s, rect, accent_top=self.accent)
        pad = max(8, int(16 * k))
        inner = rect.inflate(-2 * pad, -2 * pad)
        mode = self.game_mode
        head = t("wd.stats.title") + "  ·  " + t("wd.setup.letters", n=self.length)
        head_px = self._small_b.get_height()
        self._text(s, head, head_px, ui.TEXT, inner.w, bold=True,
                   midtop=(inner.centerx, inner.y))
        st = get_stats(self.data, mode, self.lang, self.length)
        info_px = max(10, int(13 * k))
        info = self._setup_info()
        block = pygame.Rect(inner.x, inner.y + head_px + max(6, int(12 * k)), inner.w,
                            inner.h - head_px - max(6, int(12 * k)) - info_px - max(6, int(10 * k)))
        self._draw_stats_block(s, block, st)
        self._text(s, info, info_px, ui.GOLD, inner.w,
                   midbottom=(inner.centerx, inner.bottom))

    def _setup_info(self):
        mode = self.game_mode
        if mode == "endless":
            best = self.data["best"].get(str(self.length), 0)
            return t("wd.result.best", n=best if isinstance(best, int) else 0)
        if mode == "daily":
            prog = self.data["daily"].get(self._daily_slot())
            today = seedrand.today_str()
            if isinstance(prog, dict) and prog.get("date") == today:
                if prog.get("done"):
                    return t("wd.daily.next", t=format_hms(seconds_to_midnight()))
                return t("wd.daily.progress", n=len(prog.get("guesses", [])))
            return t("wd.daily.ready")
        return t("wd.setup.sub." + mode)
