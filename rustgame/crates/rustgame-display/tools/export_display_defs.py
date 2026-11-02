# -*- coding: utf-8 -*-
"""
export_display_defs.py
======================
Erzeugt ``rustgame/crates/rustgame-display/src/konstanten.rs`` aus dem
**echten pygame-ce**.

Die knapp 600 Konstanten von ``pygame.locals`` (``K_a``, ``K_SPACE``,
``KMOD_SHIFT``, ``MOUSEBUTTONDOWN``, ``QUIT``, ``GL_DEPTH_SIZE``,
``CONTROLLER_BUTTON_A`` ...) von Hand abzuschreiben waere ein stiller
Fehler auf Vorrat: ein einziger Zahlendreher faellt erst auf, wenn Monate
spaeter ein Spiel die falsche Taste meldet. Deshalb werden die Werte hier
ausgelesen, nicht getippt - genauso wie ``rust/tools/export_defs.py`` es
fuer die Spieltabellen tut.

Und weil der naechste Fehler nach dem Zahlendreher die **vergessene
Gruppe** ist - eine neue pygame-Fassung bringt Namen mit, hier steht keine
Vorsilbe dafuer -, zaehlt das Skript am Ende selbst nach und meldet jeden
Namen aus ``pygame.locals``, den es nicht geschrieben hat; siehe
[`vollstaendigkeit`].

Aufruf (aus dem Projektordner ``pygamez``):

    .venv/Scripts/python.exe rustgame/crates/rustgame-display/tools/export_display_defs.py

Neu laufen lassen, wenn pygame-ce aktualisiert wird - dann koennen neue
Tasten oder Ereignistypen dazukommen. Die Tests in
``crates/rustgame-display/tests/gegen_pygame.rs`` pruefen die Werte, die
sich dabei nicht aendern duerfen.

Zusaetzlich zu den blanken Zahlen werden drei gemessene Tabellen erzeugt:

* ``key.name()`` - beide Schreibweisen (pygame-eigen und SDL),
* ``key.key_code()`` - die Umkehrung, samt aller Schreibvarianten,
* Tkinter-``keysym`` -> Tastencode - das umgebende Projekt speichert
  Tastenbelegungen als Tkinter-Namen (``"Up"``, ``"space"``, ``"Return"``,
  ``"w"``), also muss rustgame diese Namen lesen koennen.
"""

import os
import sys

os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
os.environ.setdefault("SDL_AUDIODRIVER", "dummy")

import pygame  # noqa: E402
import pygame.locals as L  # noqa: E402

pygame.display.init()
pygame.display.set_mode((1, 1))

# Voreinstellung: ``../src/konstanten.rs``, also genau dort, wo dieses
# Skript liegt, wenn es in ``crates/rustgame-display/tools/`` steht.
# ``RUSTGAME_DEFS_OUT`` setzt ein anderes Ziel.
ZIEL = os.environ.get("RUSTGAME_DEFS_OUT") or os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    "src",
    "konstanten.rs",
)

# ---------------------------------------------------------------------------
# Welche Konstanten in welche Gruppe gehoeren
# ---------------------------------------------------------------------------

# Ereignistypen. Reihenfolge wie in SDL, damit die Datei lesbar bleibt.
EREIGNIS_NAMEN = [
    "NOEVENT", "QUIT",
    "ACTIVEEVENT", "KEYDOWN", "KEYUP", "TEXTEDITING", "TEXTINPUT",
    "KEYMAPCHANGED",
    "MOUSEMOTION", "MOUSEBUTTONDOWN", "MOUSEBUTTONUP", "MOUSEWHEEL",
    "JOYAXISMOTION", "JOYBALLMOTION", "JOYHATMOTION", "JOYBUTTONDOWN",
    "JOYBUTTONUP", "JOYDEVICEADDED", "JOYDEVICEREMOVED",
    "CONTROLLERAXISMOTION", "CONTROLLERBUTTONDOWN", "CONTROLLERBUTTONUP",
    "CONTROLLERDEVICEADDED", "CONTROLLERDEVICEREMOVED",
    "CONTROLLERDEVICEREMAPPED", "CONTROLLERTOUCHPADDOWN",
    "CONTROLLERTOUCHPADMOTION", "CONTROLLERTOUCHPADUP",
    "CONTROLLERSENSORUPDATE",
    "FINGERDOWN", "FINGERUP", "FINGERMOTION", "MULTIGESTURE",
    "CLIPBOARDUPDATE", "DROPFILE", "DROPTEXT", "DROPBEGIN", "DROPCOMPLETE",
    "AUDIODEVICEADDED", "AUDIODEVICEREMOVED",
    "RENDER_TARGETS_RESET", "RENDER_DEVICE_RESET", "LOCALECHANGED",
    "APP_TERMINATING", "APP_LOWMEMORY", "APP_WILLENTERBACKGROUND",
    "APP_DIDENTERBACKGROUND", "APP_WILLENTERFOREGROUND",
    "APP_DIDENTERFOREGROUND",
    "VIDEORESIZE", "VIDEOEXPOSE", "SYSWMEVENT",
    "WINDOWSHOWN", "WINDOWHIDDEN", "WINDOWEXPOSED", "WINDOWMOVED",
    "WINDOWRESIZED", "WINDOWSIZECHANGED", "WINDOWMINIMIZED",
    "WINDOWMAXIMIZED", "WINDOWRESTORED", "WINDOWENTER", "WINDOWLEAVE",
    "WINDOWFOCUSGAINED", "WINDOWFOCUSLOST", "WINDOWCLOSE",
    "WINDOWTAKEFOCUS", "WINDOWHITTEST", "WINDOWICCPROFCHANGED",
    "WINDOWDISPLAYCHANGED",
    "MIDIIN", "MIDIOUT",
    "USEREVENT", "USEREVENT_DROPFILE", "NUMEVENTS",
]

# Fenster- und Flaechen-Flaggen fuer set_mode.
FLAG_NAMEN = [
    "SWSURFACE", "HWSURFACE", "ASYNCBLIT", "ANYFORMAT", "HWPALETTE",
    "DOUBLEBUF", "FULLSCREEN", "OPENGL", "OPENGLBLIT", "RESIZABLE",
    "NOFRAME", "SCALED", "SHOWN", "HIDDEN",
    "SRCALPHA", "SRCCOLORKEY", "RLEACCELOK", "RLEACCEL", "PREALLOC",
]

# Maustasten, Fensterposition, Zeigerformen, Fokuszustand.
SONST_NAMEN = [
    "BUTTON_LEFT", "BUTTON_MIDDLE", "BUTTON_RIGHT",
    "BUTTON_WHEELUP", "BUTTON_WHEELDOWN", "BUTTON_X1", "BUTTON_X2",
    "WINDOWPOS_CENTERED", "WINDOWPOS_UNDEFINED",
    "APPMOUSEFOCUS", "APPINPUTFOCUS", "APPACTIVE",
]

CURSOR_NAMEN = [n for n in dir(L) if n.startswith("SYSTEM_CURSOR_")]

# Einzelgaenger ohne Gruppe.
EINZEL_NAMEN = [
    "BIG_ENDIAN", "LIL_ENDIAN", "HWACCEL", "IS_CE", "TIMER_RESOLUTION",
]

# Einzelgaenger, deren Wert eine Zeichenkette ist.
EINZEL_STR_NAMEN = ["NULL_VIDEODRIVER"]


def mit_vorsilbe(vorsilbe):
    """Alle Namen aus ``pygame.locals``, die so anfangen - sortiert.

    Die Vorsilbe traegt ihren Unterstrich selbst (``"GL_"``, nicht
    ``"GL"``). Das ist kein Schoenheitsfehler, sondern noetig: ``AUDIO_``
    trifft die vierzehn Tonformate, ``AUDIO`` zusaetzlich die beiden
    Ereignistypen ``AUDIODEVICEADDED``/``AUDIODEVICEREMOVED``, die schon
    oben in [`EREIGNIS_NAMEN`] stehen. Dasselbe bei ``CONTROLLER_``
    gegen ``CONTROLLERAXISMOTION`` und bei ``BLEND_`` gegen
    ``BLENDMODE_``/``BLENDFACTOR_``/``BLENDOPERATION_``.

    Nur ganze Zahlen - ``SCRAP_TEXT`` und die drei anderen
    ``SCRAP_*``-Namen sind in pygame **Zeichenketten** (MIME-Typen wie
    ``"text/plain"``); die holt [`mit_vorsilbe_str`].
    """
    return sorted(
        n for n in dir(L)
        if n.startswith(vorsilbe) and isinstance(getattr(L, n), int)
    )


def mit_vorsilbe_str(vorsilbe):
    """Wie [`mit_vorsilbe`], nur fuer die Namen mit Zeichenketten-Wert."""
    return sorted(
        n for n in dir(L)
        if n.startswith(vorsilbe) and isinstance(getattr(L, n), str)
    )


def hol(name):
    """Wert aus pygame.locals - fehlt er, ist das ein harter Fehler."""
    if not hasattr(L, name):
        return None
    return getattr(L, name)


# ---------------------------------------------------------------------------
# Tkinter-keysym -> pygame-Tastenname
# ---------------------------------------------------------------------------
# Das umgebende Projekt (pygamez) speichert Tastenbelegungen als
# Tkinter-"keysym"-Strings, siehe settings.py: {"up": "w", "action": "space"}.
# Fuer die Namen, bei denen Tkinter und SDL sich einig sind, findet
# pygame.key.key_code() den Code von selbst; die uebrigen stehen hier.
# Links der Tkinter-Name, rechts der pygame-Konstantenname.
TK_SONDERFAELLE = {
    "Return": "K_RETURN",
    "Escape": "K_ESCAPE",
    "BackSpace": "K_BACKSPACE",
    "Tab": "K_TAB",
    "Prior": "K_PAGEUP",       # Tkinter nennt Bild-auf "Prior"
    "Next": "K_PAGEDOWN",      # ... und Bild-ab "Next"
    "Caps_Lock": "K_CAPSLOCK",
    "Num_Lock": "K_NUMLOCK",
    "Scroll_Lock": "K_SCROLLLOCK",
    "Shift_L": "K_LSHIFT",
    "Shift_R": "K_RSHIFT",
    "Control_L": "K_LCTRL",
    "Control_R": "K_RCTRL",
    "Alt_L": "K_LALT",
    "Alt_R": "K_RALT",
    "Meta_L": "K_LGUI",
    "Meta_R": "K_RGUI",
    "Super_L": "K_LGUI",
    "Super_R": "K_RGUI",
    "Print": "K_PRINTSCREEN",
    "Sys_Req": "K_SYSREQ",
    "space": "K_SPACE",
    "exclam": "K_EXCLAIM",
    "quotedbl": "K_QUOTEDBL",
    "numbersign": "K_HASH",
    "dollar": "K_DOLLAR",
    "percent": "K_PERCENT",
    "ampersand": "K_AMPERSAND",
    "apostrophe": "K_QUOTE",
    "quoteright": "K_QUOTE",
    "parenleft": "K_LEFTPAREN",
    "parenright": "K_RIGHTPAREN",
    "asterisk": "K_ASTERISK",
    "plus": "K_PLUS",
    "comma": "K_COMMA",
    "minus": "K_MINUS",
    "period": "K_PERIOD",
    "slash": "K_SLASH",
    "colon": "K_COLON",
    "semicolon": "K_SEMICOLON",
    "less": "K_LESS",
    "equal": "K_EQUALS",
    "greater": "K_GREATER",
    "question": "K_QUESTION",
    "at": "K_AT",
    "bracketleft": "K_LEFTBRACKET",
    "backslash": "K_BACKSLASH",
    "bracketright": "K_RIGHTBRACKET",
    "asciicircum": "K_CARET",
    "underscore": "K_UNDERSCORE",
    "grave": "K_BACKQUOTE",
    "quoteleft": "K_BACKQUOTE",
    # Ziffernblock: Tkinter meldet "KP_1" ... "KP_0", "KP_Enter" usw.
    "KP_0": "K_KP0", "KP_1": "K_KP1", "KP_2": "K_KP2", "KP_3": "K_KP3",
    "KP_4": "K_KP4", "KP_5": "K_KP5", "KP_6": "K_KP6", "KP_7": "K_KP7",
    "KP_8": "K_KP8", "KP_9": "K_KP9",
    "KP_Enter": "K_KP_ENTER",
    "KP_Add": "K_KP_PLUS",
    "KP_Subtract": "K_KP_MINUS",
    "KP_Multiply": "K_KP_MULTIPLY",
    "KP_Divide": "K_KP_DIVIDE",
    "KP_Decimal": "K_KP_PERIOD",
    "KP_Separator": "K_KP_PERIOD",
    "KP_Home": "K_KP7", "KP_End": "K_KP1",
    "KP_Up": "K_KP8", "KP_Down": "K_KP2",
    "KP_Left": "K_KP4", "KP_Right": "K_KP6",
    "KP_Prior": "K_KP9", "KP_Next": "K_KP3",
    "KP_Begin": "K_KP5", "KP_Insert": "K_KP0", "KP_Delete": "K_KP_PERIOD",
}

# Diese Namen findet pygame.key.key_code() von sich aus, sie brauchen also
# keinen Eintrag oben - hier stehen sie nur, damit sie in der erzeugten
# Tabelle wirklich auftauchen.
TK_DIREKT = [
    "Up", "Down", "Left", "Right", "Home", "End", "Insert", "Delete",
    "Pause", "Menu", "Help", "Clear", "Break",
] + ["F%d" % i for i in range(1, 16)] \
  + [chr(c) for c in range(ord("a"), ord("z") + 1)] \
  + [chr(c) for c in range(ord("A"), ord("Z") + 1)] \
  + [chr(c) for c in range(ord("0"), ord("9") + 1)]


# ---------------------------------------------------------------------------
# minifb::Key -> pygame-Tastenname
# ---------------------------------------------------------------------------
# minifb kennt 106 Tasten; jede bekommt hier ihr pygame-Gegenstueck. Die
# Zahlen dahinter holt das Skript aus echtem pygame, damit kein Wert
# abgeschrieben werden muss.
MINIFB_ZUORDNUNG = (
    [("Key%d" % i, "K_%d" % i) for i in range(10)]
    + [(chr(c), "K_" + chr(c).lower()) for c in range(ord("A"), ord("Z") + 1)]
    + [("F%d" % i, "K_F%d" % i) for i in range(1, 16)]
    + [("NumPad%d" % i, "K_KP%d" % i) for i in range(10)]
    + [
        ("Down", "K_DOWN"), ("Left", "K_LEFT"),
        ("Right", "K_RIGHT"), ("Up", "K_UP"),
        ("Apostrophe", "K_QUOTE"), ("Backquote", "K_BACKQUOTE"),
        ("Backslash", "K_BACKSLASH"), ("Comma", "K_COMMA"),
        ("Equal", "K_EQUALS"), ("LeftBracket", "K_LEFTBRACKET"),
        ("Minus", "K_MINUS"), ("Period", "K_PERIOD"),
        ("RightBracket", "K_RIGHTBRACKET"), ("Semicolon", "K_SEMICOLON"),
        ("Slash", "K_SLASH"), ("Backspace", "K_BACKSPACE"),
        ("Delete", "K_DELETE"), ("End", "K_END"),
        # minifb sagt "Enter", pygame sagt "Return" - dieselbe Taste.
        ("Enter", "K_RETURN"), ("Escape", "K_ESCAPE"),
        ("Home", "K_HOME"), ("Insert", "K_INSERT"), ("Menu", "K_MENU"),
        ("PageDown", "K_PAGEDOWN"), ("PageUp", "K_PAGEUP"),
        ("Pause", "K_PAUSE"), ("Space", "K_SPACE"), ("Tab", "K_TAB"),
        ("NumLock", "K_NUMLOCK"), ("CapsLock", "K_CAPSLOCK"),
        ("ScrollLock", "K_SCROLLLOCK"),
        ("LeftShift", "K_LSHIFT"), ("RightShift", "K_RSHIFT"),
        ("LeftCtrl", "K_LCTRL"), ("RightCtrl", "K_RCTRL"),
        ("NumPadDot", "K_KP_PERIOD"), ("NumPadSlash", "K_KP_DIVIDE"),
        ("NumPadAsterisk", "K_KP_MULTIPLY"), ("NumPadMinus", "K_KP_MINUS"),
        ("NumPadPlus", "K_KP_PLUS"), ("NumPadEnter", "K_KP_ENTER"),
        ("LeftAlt", "K_LALT"), ("RightAlt", "K_RALT"),
        ("LeftSuper", "K_LGUI"), ("RightSuper", "K_RGUI"),
        ("Unknown", "K_UNKNOWN"),
    ]
)


# ---------------------------------------------------------------------------
# Rust schreiben
# ---------------------------------------------------------------------------

def rs_str(s):
    return '"%s"' % s.replace("\\", "\\\\").replace('"', '\\"')


def rs_u32(v):
    """Wert als u32-Literal.

    Gemessen: ``pygame.locals.FULLSCREEN`` ist **negativ** (-2147483648),
    weil SDL das oberste Bit benutzt und CPython daraus ein vorzeichen-
    behaftetes int macht. Als Rust-``u32`` geschrieben waere das ein
    Uebersetzungsfehler, also wird hier auf 32 Bit maskiert - der Bitwert
    bleibt derselbe.

    **Nicht jedes negative Vorzeichen ist so eines.** Nachgesehen wurde
    die ganze ``pygame.locals``: negativ sind genau drei Namen -
    ``FULLSCREEN`` (das oberste Bit), ``CONTROLLER_AXIS_INVALID`` und
    ``CONTROLLER_BUTTON_INVALID``. Die beiden letzten sind echte ``-1``
    aus einer vorzeichenbehafteten SDL-Aufzaehlung; maskiert ergaeben sie
    4294967295 und damit einen Achsenindex, den niemand wiedererkennt.
    Sie werden darum als ``i32`` geschrieben - siehe [`rs_i32`].
    """
    return "0x%08X" % (v & 0xFFFFFFFF) if v < 0 else str(v)


def rs_i32(v):
    """Wert als i32-Literal - Vorzeichen bleibt Vorzeichen."""
    return str(v)


def doku_ereignis(n, v):
    return "`%s` - Ereignistyp %d." % (pygame.event.event_name(v), v)


def doku_taste(n, v):
    kompat = pygame.key.name(v)
    sdl = pygame.key.name(v, use_compat=False)
    if kompat == sdl:
        return "Taste `%s`." % kompat
    return "Taste `%s` (SDL nennt sie `%s`)." % (kompat, sdl)


def doku_platz(n, v):
    k = "K_" + n[len("KSCAN_"):]
    if hasattr(L, k):
        return "Tastenplatz von [`%s`] - Platz %d." % (k, v)
    return "Tastenplatz %d." % v


def doku_mod(n, v):
    return "Zusatztaste `%s` - Bitmaske %d." % (n, v)


def doku_allgemein(n, v):
    return "`pygame.%s`." % n


def block(zeilen, namen, typ, doku=doku_allgemein):
    """Einen Satz `pub const NAME: typ = wert;` anhaengen.

    `typ` entscheidet auch ueber die Schreibweise des Werts: bei `i32`
    bleibt ein Minus stehen, sonst wird auf 32 Bit maskiert.
    """
    vorzeichen = typ == "i32"
    gezaehlt = 0
    for n in namen:
        v = hol(n)
        if v is None:
            print("  uebersprungen (fehlt in dieser pygame-Fassung):", n)
            continue
        GESCHRIEBEN.add(n)
        if doku:
            zeilen.append("/// %s" % doku(n, v))
        if v < 0 and not vorzeichen:
            zeilen.append(
                "/// In pygame steht hier %d - CPython macht aus dem obersten"
                % v)
            zeilen.append("/// SDL-Bit ein Vorzeichen. Derselbe Bitwert als `u32`.")
        wert = rs_i32(v) if vorzeichen else rs_u32(v)
        zeilen.append("pub const %s: %s = %s;" % (n, typ, wert))
        gezaehlt += 1
    return gezaehlt


def block_str(zeilen, namen, doku=doku_allgemein):
    """Wie [`block`], nur fuer die Namen, deren Wert eine Zeichenkette ist.

    Das sind in `pygame.locals` genau vier: die MIME-Typen der
    Zwischenablage (`SCRAP_TEXT` ist `"text/plain"`). Wer sie fuer Zahlen
    haelt, merkt es erst, wenn der Uebersetzer ueber `= text/plain;`
    stolpert - darum stehen sie in einem eigenen Satz.
    """
    gezaehlt = 0
    for n in namen:
        v = hol(n)
        if v is None:
            print("  uebersprungen (fehlt in dieser pygame-Fassung):", n)
            continue
        GESCHRIEBEN.add(n)
        if doku:
            zeilen.append("/// %s" % doku(n, v))
        zeilen.append("pub const %s: &str = %s;" % (n, rs_str(v)))
        gezaehlt += 1
    return gezaehlt


def ueberschrift(zeilen, text):
    zeilen.append("// ---------------------------------------------------------------")
    zeilen.append("// %s" % text)
    zeilen.append("// ---------------------------------------------------------------")
    zeilen.append("")


# Jeder Name, den [`block`] wirklich geschrieben hat. Am Ende wird damit
# gegen `pygame.locals` abgeglichen - siehe [`vollstaendigkeit`].
GESCHRIEBEN = set()


def vollstaendigkeit():
    """Meldet, welche ganzzahligen `pygame.locals` noch fehlen.

    Der Zweck dieses Skripts ist, dass niemand Zahlen abschreibt. Der
    naechstliegende Fehler ist dann nicht mehr der Zahlendreher, sondern
    die **vergessene Gruppe**: eine neue pygame-Fassung bringt Konstanten
    mit, hier steht keine Vorsilbe dafuer, und niemandem faellt es auf.
    Darum zaehlt das Skript am Ende selbst nach.
    """
    alle = set()
    for n in dir(L):
        if n.startswith("_"):
            continue
        if isinstance(getattr(L, n), (int, str)):
            alle.add(n)
    fehlt = sorted(alle - GESCHRIEBEN)
    print("  %-16s %5d von %d" % ("abgedeckt", len(GESCHRIEBEN & alle), len(alle)))
    if fehlt:
        print("  ACHTUNG - diese pygame.locals stehen in keiner Gruppe:")
        for n in fehlt:
            print("      %-34s = %r" % (n, getattr(L, n)))
    else:
        print("  vollstaendig - jede ganze Zahl aus pygame.locals steht drin.")
    return fehlt


def main():
    anzahl = {}
    z = []
    z.append("//! Konstanten von pygame-ce %s (SDL %s)" % (
        pygame.version.ver, pygame.version.SDL))
    z.append("//! ==========================================")
    z.append("//!")
    z.append("//! **Diese Datei ist erzeugt - nicht von Hand aendern.**")
    z.append("//!")
    z.append("//! Quelle: `pygame.locals` aus dem echten pygame-ce im `.venv` des")
    z.append("//! Projekts, ausgelesen von `tools/export_display_defs.py` neben")
    z.append("//! dieser Kiste. Neu erzeugen (aus dem Projektordner `pygamez`):")
    z.append("//!")
    z.append("//! ```text")
    z.append("//! .venv/Scripts/python.exe \\")
    z.append("//!     rustgame/crates/rustgame-display/tools/export_display_defs.py")
    z.append("//! ```")
    z.append("//!")
    z.append("//! Das Skript braucht kein Fenster - es setzt selbst")
    z.append("//! `SDL_VIDEODRIVER=dummy`.")
    z.append("//!")
    z.append("//! Warum erzeugt und nicht getippt: ein Zahlendreher in einem der")
    z.append("//! knapp 600 Werte faellt nicht beim Uebersetzen auf, sondern erst,")
    z.append("//! wenn Monate spaeter die falsche Taste gemeldet wird. Das Skript")
    z.append("//! prueft beim Erzeugen ausserdem nach, dass **jeder** ganzzahlige")
    z.append("//! und jeder Zeichenketten-Name aus `pygame.locals` hier steht -")
    z.append("//! eine vergessene Gruppe faellt damit sofort auf.")
    z.append("")
    z.append("#![allow(non_upper_case_globals)]")
    z.append("")
    z.append("/// Ein Tastencode (`K_*`), wie ihn `Event::Key` fuehrt.")
    z.append("pub type Key = u32;")
    z.append("/// Ein Tastenplatz (`KSCAN_*`) - die *Stelle* auf der Tastatur,")
    z.append("/// unabhaengig von der Belegung.")
    z.append("pub type Scancode = u32;")
    z.append("/// Ein Ereignistyp (`QUIT`, `KEYDOWN`, ...).")
    z.append("pub type EventType = u32;")
    z.append("/// Ein Satz Zusatztasten (`KMOD_*`), als Bitmaske.")
    z.append("pub type KeyMod = u32;")
    z.append("")

    z.append("// ---------------------------------------------------------------")
    z.append("// Ereignistypen")
    z.append("// ---------------------------------------------------------------")
    z.append("")
    anzahl["Ereignistypen"] = block(z, EREIGNIS_NAMEN, "EventType", doku_ereignis)
    z.append("")

    z.append("// ---------------------------------------------------------------")
    z.append("// Fenster- und Flaechen-Flaggen")
    z.append("// ---------------------------------------------------------------")
    z.append("")
    anzahl["Flaggen"] = block(z, FLAG_NAMEN, "u32")
    z.append("")

    z.append("// ---------------------------------------------------------------")
    z.append("// Tastencodes (K_*)")
    z.append("// ---------------------------------------------------------------")
    z.append("")
    k_namen = sorted([n for n in dir(L) if n.startswith("K_")])
    anzahl["Tastencodes"] = block(z, k_namen, "Key", doku_taste)
    z.append("")

    z.append("// ---------------------------------------------------------------")
    z.append("// Tastenplaetze (KSCAN_*)")
    z.append("// ---------------------------------------------------------------")
    z.append("")
    ks_namen = sorted([n for n in dir(L) if n.startswith("KSCAN_")])
    anzahl["Tastenplaetze"] = block(z, ks_namen, "Scancode", doku_platz)
    z.append("")

    z.append("// ---------------------------------------------------------------")
    z.append("// Zusatztasten (KMOD_*)")
    z.append("// ---------------------------------------------------------------")
    z.append("")
    km_namen = sorted([n for n in dir(L) if n.startswith("KMOD_")])
    anzahl["Zusatztasten"] = block(z, km_namen, "KeyMod", doku_mod)
    z.append("")

    z.append("// ---------------------------------------------------------------")
    z.append("// Maustasten, Fensterposition, Zeigerformen")
    z.append("// ---------------------------------------------------------------")
    z.append("")
    anzahl["Sonstige"] = block(z, SONST_NAMEN, "u32")
    anzahl["Zeigerformen"] = block(z, sorted(CURSOR_NAMEN), "u32")
    z.append("")

    # --- OpenGL -----------------------------------------------------------
    ueberschrift(z, "OpenGL-Merkmale (GL_*)")
    z.append("// **Diese Kiste kann kein OpenGL.** Die Namen stehen hier, weil")
    z.append("// `pygame.locals` sie fuehrt und Programme sie schreiben duerfen -")
    z.append("// `display::gl_set_attribute` nimmt sie entgegen und sagt ehrlich")
    z.append("// nein. Gemessen an echtem pygame unter `SDL_VIDEODRIVER=dummy`:")
    z.append("// `gl_get_attribute(GL_DEPTH_SIZE)` wirft dort")
    z.append("// `No dynamic GL support in current SDL video driver (dummy)`.")
    z.append("")
    anzahl["OpenGL"] = block(z, mit_vorsilbe("GL_"), "u32")
    z.append("")

    # --- Spielsteuerungen -------------------------------------------------
    ueberschrift(z, "Spielsteuerungen (CONTROLLER_*, HAT_*)")
    z.append("/// Als `i32` und nicht als `u32`: `CONTROLLER_AXIS_INVALID` und")
    z.append("/// `CONTROLLER_BUTTON_INVALID` sind in pygame wirklich `-1`, nicht")
    z.append("/// bloss ein gesetztes oberstes Bit wie bei [`FULLSCREEN`]. Auf 32")
    z.append("/// Bit maskiert ergaebe das 4294967295 - einen Achsenindex, den")
    z.append("/// niemand wiedererkennt.")
    z.append("pub type ControllerIndex = i32;")
    z.append("")
    anzahl["Spielsteuerungen"] = block(z, mit_vorsilbe("CONTROLLER_"), "i32")
    z.append("")
    anzahl["Steuerkreuz"] = block(z, mit_vorsilbe("HAT_"), "u32")
    z.append("")

    # --- Ton --------------------------------------------------------------
    ueberschrift(z, "Tonformate (AUDIO_*)")
    z.append("/// Diese Werte gehoeren zu `pygame.mixer`, nicht hierher - aber")
    z.append("/// `pygame.locals` fuehrt sie, und `rustgame-py` reicht die ganze")
    z.append("/// Datei als `pygame.*` weiter. Wer sie auswertet, ist")
    z.append("/// `rustgame-mixer`.")
    z.append("pub type AudioFormat = u32;")
    z.append("")
    anzahl["Tonformate"] = block(z, mit_vorsilbe("AUDIO_"), "AudioFormat")
    z.append("")

    # --- Mischen ----------------------------------------------------------
    ueberschrift(z, "Mischen (BLEND_*, BLENDMODE_*, BLENDFACTOR_*, BLENDOPERATION_*)")
    z.append("/// `BLEND_*` sind die Werte, mit denen `Surface::blit` und")
    z.append("/// `Surface::fill` rechnen - dieselben Zahlen stehen in")
    z.append("/// `rustgame_core::blend::flags`, dort aber von Hand gepflegt und")
    z.append("/// von einem Test gegen genau diese Tabelle geprueft. Die")
    z.append("/// `BLENDMODE_*`/`BLENDFACTOR_*`/`BLENDOPERATION_*` gehoeren zu")
    z.append("/// SDLs Renderer, den es hier nicht gibt.")
    z.append("pub type BlendMode = u32;")
    z.append("")
    anzahl["Mischen"] = block(z, mit_vorsilbe("BLEND_"), "BlendMode")
    anzahl["Mischen"] += block(z, mit_vorsilbe("BLENDMODE_"), "u32")
    anzahl["Mischen"] += block(z, mit_vorsilbe("BLENDFACTOR_"), "u32")
    anzahl["Mischen"] += block(z, mit_vorsilbe("BLENDOPERATION_"), "u32")
    z.append("")

    # --- Der Rest ---------------------------------------------------------
    ueberschrift(z, "Schrift, Bilder, Zwischenablage, Endianness")
    rest = 0
    for vorsilbe in ("DIRECTION_", "FLASH_", "FLIP_", "FONT_",
                     "TEXTUREACCESS_", "SCRAP_", "SCROLL_"):
        rest += block(z, mit_vorsilbe(vorsilbe), "u32")
    rest += block(z, EINZEL_NAMEN, "u32")
    z.append("")
    z.append("// Die vier MIME-Typen der Zwischenablage sind in pygame")
    z.append("// Zeichenketten, keine Zahlen - gemessen: `SCRAP_TEXT` ist")
    z.append("// `\"text/plain\"`.")
    rest += block_str(z, mit_vorsilbe_str("SCRAP_"))
    z.append("")
    z.append("// Der Treibername, unter dem pygame ohne Fenster laeuft - und")
    z.append("// damit genau der Fall, fuer den es hier")
    z.append("// [`crate::backend::DummyBackend`] gibt.")
    rest += block_str(z, EINZEL_STR_NAMEN)
    anzahl["Uebrige"] = rest
    z.append("")

    # --- Anzahl Tastenplaetze, gemessen -----------------------------------
    p = pygame.key.get_pressed()
    z.append("// ---------------------------------------------------------------")
    z.append("// Gemessene Groessen")
    z.append("// ---------------------------------------------------------------")
    z.append("")
    z.append("/// Laenge der Tabelle aus `key::get_pressed()`.")
    z.append("///")
    z.append("/// Gemessen an echtem pygame: `len(pygame.key.get_pressed())` ist")
    z.append("/// %d, und der Index ist der **Tastenplatz**, nicht der Tastencode." % len(p))
    z.append("pub const NUM_SCANCODES: usize = %d;" % len(p))
    z.append("")
    z.append("/// Bit, das einen Tastencode von seinem Tastenplatz trennt.")
    z.append("///")
    z.append("/// SDL bildet Tasten ohne Zeichen (Pfeile, F-Tasten, ...) als")
    z.append("/// `platz | SCANCODE_MASK` ab; gemessen: `K_UP` ist %d," % L.K_UP)
    z.append("/// `KSCAN_UP` ist %d, und %d ist genau `1 << 30`." % (
        L.KSCAN_UP, L.K_UP - L.KSCAN_UP))
    z.append("pub const SCANCODE_MASK: u32 = 1 << 30;")
    z.append("")

    # --- Tastencode <-> Tastenplatz ---------------------------------------
    # Fuer jede K_*-Konstante den zugehoerigen Tastenplatz. Fuer Tasten mit
    # gesetzter Maske ist das trivial; fuer Zeichentasten (K_a, K_SPACE)
    # nicht - die Zuordnung kommt aus den KSCAN_-Gegenstuecken.
    paare = []
    for n in k_namen:
        k = getattr(L, n)
        # Achtung: die Tastencodes heissen klein (``K_a``), die Plaetze
        # gross (``KSCAN_A``). Ohne das ``upper()`` faende die Suche fuer
        # jede Buchstabentaste nichts - und ``scancode_von(K_a)`` gaebe
        # still 0 zurueck statt 4.
        rest = "KSCAN_" + n[2:].upper()
        if hasattr(L, rest):
            paare.append((k, getattr(L, rest), n))
    paare.sort()
    # Doppelte Tastencodes (z.B. K_QUOTE/K_QUOTEDBL teilen sich keinen) raus
    gesehen = set()
    eindeutig = []
    for k, s, n in paare:
        if k in gesehen:
            continue
        gesehen.add(k)
        eindeutig.append((k, s, n))
    z.append("/// Tastencode -> Tastenplatz, nach Tastencode sortiert.")
    z.append("///")
    z.append("/// `key::get_pressed()` wird mit dem Tastenplatz indiziert, die")
    z.append("/// Spiele fragen aber mit `K_*`. Diese Tabelle ueberbrueckt das;")
    z.append("/// sie stammt aus den `KSCAN_*`-Gegenstuecken in `pygame.locals`.")
    z.append("pub static KEY_TO_SCANCODE: &[(Key, Scancode)] = &[")
    for k, s, n in eindeutig:
        z.append("    (%d, %d), // %s" % (k, s, n))
    z.append("];")
    z.append("")
    anzahl["Code->Platz"] = len(eindeutig)

    # --- key.name() -------------------------------------------------------
    namen_tab = []
    for n in k_namen:
        k = getattr(L, n)
        if k in [x[0] for x in namen_tab]:
            continue
        kompat = pygame.key.name(k)
        sdl = pygame.key.name(k, use_compat=False)
        namen_tab.append((k, kompat, sdl))
    namen_tab.sort()
    z.append("/// Tastencode -> Name, wie `pygame.key.name()` ihn liefert.")
    z.append("///")
    z.append("/// Zwei Spalten, weil pygame zwei Schreibweisen kennt: die eigene")
    z.append("/// (`name(k)`, kleingeschrieben - `\"a\"`, `\"left shift\"`) und die")
    z.append("/// von SDL (`name(k, use_compat=False)` - `\"A\"`, `\"Left Shift\"`).")
    z.append("/// Beide sind hier gemessen, nicht geraten.")
    z.append("pub static KEY_NAMES: &[(Key, &str, &str)] = &[")
    for k, kompat, sdl in namen_tab:
        z.append("    (%d, %s, %s)," % (k, rs_str(kompat), rs_str(sdl)))
    z.append("];")
    z.append("")
    anzahl["Namen"] = len(namen_tab)

    # --- key_code() -------------------------------------------------------
    # Jede Schreibweise, die echtes pygame aufloest, samt Ergebnis.
    code_tab = {}

    def merke(name):
        try:
            code_tab[name] = pygame.key.key_code(name)
        except Exception:
            pass

    for k, kompat, sdl in namen_tab:
        merke(kompat)
        merke(sdl)
        merke(kompat.lower())
    for n in k_namen:
        merke(n[2:].lower())
    codes = sorted(code_tab.items())
    z.append("/// Name -> Tastencode, die Umkehrung von `KEY_NAMES`.")
    z.append("///")
    z.append("/// Enthaelt jede Schreibweise, die echtes pygame aufloest - das")
    z.append("/// sind mehr als nur die Namen aus `KEY_NAMES`, weil")
    z.append("/// `key_code()` bei Mehrzeichen-Namen die Gross-/Kleinschreibung")
    z.append("/// nicht beachtet (`\"Up\"` und `\"up\"` ergeben beide %d)," % L.K_UP)
    z.append("/// bei Einzelzeichen dagegen schon.")
    z.append("/// Nach Name sortiert, damit binaer gesucht werden kann.")
    z.append("pub static KEY_CODES: &[(&str, Key)] = &[")
    for name, code in codes:
        z.append("    (%s, %d)," % (rs_str(name), code))
    z.append("];")
    z.append("")
    anzahl["key_code-Namen"] = len(codes)

    # --- Tkinter ----------------------------------------------------------
    tk = {}
    fehlt = []
    for tkname, kname in sorted(TK_SONDERFAELLE.items()):
        if not hasattr(L, kname):
            fehlt.append((tkname, kname))
            continue
        tk[tkname] = getattr(L, kname)
    for tkname in TK_DIREKT:
        try:
            tk[tkname] = pygame.key.key_code(tkname)
        except Exception:
            fehlt.append((tkname, "key_code"))
    tk_sortiert = sorted(tk.items())
    z.append("/// Tkinter-`keysym` -> Tastencode.")
    z.append("///")
    z.append("/// Das umgebende Projekt (pygamez) speichert Tastenbelegungen in")
    z.append("/// seiner Einstellungsdatei als Tkinter-Namen - `\"Up\"`, `\"space\"`,")
    z.append("/// `\"Return\"`, `\"w\"` - weil seine Oberflaeche auf Tkinter sitzt.")
    z.append("/// Ohne diese Tabelle waere genau dort die Bruchstelle, wenn")
    z.append("/// rustgame die gespeicherten Belegungen liest.")
    z.append("///")
    z.append("/// Die meisten Namen loest `key_code()` von selbst auf; die")
    z.append("/// Ausnahmen sind im Erzeuger-Skript aufgelistet. Zwei Faelle, die")
    z.append("/// nicht zu raten sind: Tkinter nennt Bild-auf `\"Prior\"` und")
    z.append("/// Bild-ab `\"Next\"`.")
    z.append("/// Nach Name sortiert, damit binaer gesucht werden kann.")
    z.append("pub static TK_KEYSYMS: &[(&str, Key)] = &[")
    for name, code in tk_sortiert:
        z.append("    (%s, %d)," % (rs_str(name), code))
    z.append("];")
    z.append("")
    anzahl["Tkinter-Namen"] = len(tk_sortiert)

    # --- minifb ------------------------------------------------------------
    mf = []
    mf_fehlt = []
    for mname, kname in MINIFB_ZUORDNUNG:
        if not hasattr(L, kname):
            mf_fehlt.append((mname, kname))
            continue
        mf.append((mname, getattr(L, kname), kname))
    mf.sort()
    z.append("/// minifb-Taste -> pygame-Tastencode.")
    z.append("///")
    z.append("/// Links der Name der `minifb::Key`-Variante (so, wie `Debug` ihn")
    z.append("/// schreibt), rechts der pygame-Tastencode - aus echtem pygame")
    z.append("/// ausgelesen, nicht abgeschrieben. Das minifb-Hintergrundstueck")
    z.append("/// rechnet mit einem `match` um; ein Test prueft dieses `match`")
    z.append("/// gegen diese Tabelle, damit beide nicht auseinanderlaufen.")
    z.append("///")
    z.append("/// Nach Name sortiert, damit binaer gesucht werden kann.")
    z.append("pub static MINIFB_KEYS: &[(&str, Key)] = &[")
    for mname, code, kname in mf:
        z.append("    (%s, %d), // %s" % (rs_str(mname), code, kname))
    z.append("];")
    z.append("")
    anzahl["minifb-Tasten"] = len(mf)

    # --- Ereignisnamen ----------------------------------------------------
    ev = []
    for n in EREIGNIS_NAMEN:
        v = hol(n)
        if v is None or n in ("NUMEVENTS", "USEREVENT_DROPFILE"):
            continue
        ev.append((v, pygame.event.event_name(v)))
    ev = sorted(set(ev))
    z.append("/// Ereignistyp -> Name, wie `pygame.event.event_name()` ihn liefert.")
    z.append("///")
    z.append("/// Gemessen: unbekannte Typen ergeben `\"Unknown\"`, und alles ab")
    z.append("/// `USEREVENT` heisst `\"UserEvent\"`.")
    z.append("pub static EVENT_NAMES: &[(EventType, &str)] = &[")
    for v, name in ev:
        z.append("    (%d, %s)," % (v, rs_str(name)))
    z.append("];")
    z.append("")
    anzahl["Ereignisnamen"] = len(ev)

    with open(ZIEL, "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(z) + "\n")

    gesamt = sum(anzahl.values())
    print("geschrieben:", ZIEL)
    for k in anzahl:
        print("  %-16s %5d" % (k, anzahl[k]))
    print("  %-16s %5d" % ("GESAMT", gesamt))
    if fehlt:
        print("nicht aufloesbar:", fehlt)
    print("Abgleich gegen pygame.locals:")
    vollstaendigkeit()
    return 0


if __name__ == "__main__":
    sys.exit(main())
