# -*- coding: utf-8 -*-
"""
rust/tools/export_defs.py
=========================
Erzeugt ``rust/crates/pgz-core/src/defs.rs`` aus den Python-Quellen.

Damit die Rust-Fassung exakt dieselben 49 Spiele, Akzentfarben und 268
Erfolge kennt wie die Python-Fassung, werden diese Tabellen nicht von Hand
abgetippt, sondern hier ausgelesen und als Rust-Konstanten geschrieben -
genauso, wie ``devtools/export_achievements.py`` es fuer die Web-Version tut.

Aufruf (aus dem Projektordner):

    .venv/Scripts/python.exe rust/tools/export_defs.py

Nach jeder Aenderung an ``achievements.py``, ``games/__init__.py`` oder
``ui.GAME_COLORS`` neu laufen lassen; ``tests/rust_audit.py`` prueft, ob die
erzeugte Datei noch zu den Quellen passt.
"""

import json
import os
import sys

# Ohne Bildschirm laufen lassen - games/__init__.py zieht pygame herein.
os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
os.environ.setdefault("SDL_AUDIODRIVER", "dummy")

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, ROOT)

import achievements  # noqa: E402
import i18n  # noqa: E402
import ui  # noqa: E402
from games import ALL_GAMES  # noqa: E402

i18n.init()

OUT = os.path.join(ROOT, "rust", "crates", "pgz-core", "src", "defs.rs")


def rs_str(value):
    """Python-Wert als Rust-Zeichenkette (None wird zu "")."""
    if value is None:
        return '""'
    return '"%s"' % str(value).replace("\\", "\\\\").replace('"', '\\"')


def rs_opt_i64(value):
    """None -> None, Zahl -> Some(n)."""
    return "None" if value is None else "Some(%d)" % int(value)


def rs_bool(value):
    """Python-Wahrheitswert als Rust-Literal (None zaehlt als false)."""
    return "true" if value else "false"


def rs_f32(value):
    """Zahl als Rust-f32-Literal (immer mit Punkt, damit es f32 bleibt)."""
    return "%r" % float(value or 0.0)


def rs_rgb(color):
    """(r, g, b) als Rust-Tupel."""
    r, g, b = color[:3]
    return "(%d, %d, %d)" % (int(r), int(g), int(b))


# Reihenfolge der Farben in Palette (stabil, damit die Datei nicht springt).
COLOR_KEYS = ("BG_TOP", "BG_BOTTOM", "PANEL", "PANEL_LIGHT", "BORDER",
              "BORDER_LIGHT", "BTN", "BTN_SEL", "ACCENT", "ACCENT2",
              "ACCENT_SOFT", "GREEN", "GOLD", "RED", "TEXT", "TEXT_DIM",
              "TEXT_FAINT")


# Die Schluessel der Tk-Paletten (Seitenleiste), in der Reihenfolge von ui.py.
TK_KEYS = ("SIDEBAR", "HEADER", "CARD", "BTN", "BTN_HOVER", "ACCENT", "ACCENT2",
           "DANGER", "DANGER_HOVER", "BACK", "BACK_HOVER", "TEXT", "TEXT_DIM",
           "TEXT_FAINT", "BORDER", "GREEN", "GOLD", "RED")

def main():
    lines = []
    add = lines.append

    add("//! defs.rs")
    add("//! =======")
    add("//! **Erzeugte Datei - nicht von Hand aendern.**")
    add("//!")
    add("//! Quelle: `rust/tools/export_defs.py` liest `games/__init__.py`,")
    add("//! `ui.GAME_COLORS` und `achievements.py` der Python-Fassung aus. So")
    add("//! kennen beide Fassungen dieselben Spiele, Akzentfarben und Erfolge -")
    add("//! auch die, deren Spiel in Rust noch nicht portiert ist (sonst")
    add("//! verschoebe sich die Erfolgs-Gesamtzahl und damit jeder Fortschritt).")
    add("//!")
    add("//! Neu erzeugen:")
    add("//! ```text")
    add("//! .venv/Scripts/python.exe rust/tools/export_defs.py")
    add("//! ```")
    add("")
    add("/// Eine Farbe als `(rot, gruen, blau)`.")
    add("///")
    add("/// Die ganze Sammlung rechnet mit 8-Bit-Kanaelen; erst beim Zeichnen")
    add("/// wird daraus eine macroquad-Farbe (`ui::col`).")
    add("pub type Rgb = (u8, u8, u8);")
    add("")

    # ----- Spiele --------------------------------------------------------
    add("/// Ein Spiel der Sammlung, so wie die Python-Fassung es kennt.")
    add("///")
    add("/// `key` ist der Highscore-Schluessel (auch der Name des")
    add("/// Statistik-Eintrags), `class_name` der Klassenname aus der")
    add("/// Python-Fassung (Schluessel in `ui.GAME_COLORS`), `color` die")
    add("/// Akzentfarbe des Spiels und `name` der Anzeigename.")
    add("#[derive(Debug, Clone, Copy, PartialEq, Eq)]")
    add("pub struct GameDef {")
    add("    pub key: &'static str,")
    add("    pub class_name: &'static str,")
    add("    pub color: &'static str,")
    add("    pub name: &'static str,")
    add("}")
    add("")
    add("/// Alle %d Spiele in Menue-Reihenfolge." % len(ALL_GAMES))
    add("pub const GAMES: &[GameDef] = &[")
    for cls in ALL_GAMES:
        name = cls.__name__
        color = ui.GAME_COLORS.get(name, "#589cff")
        i18n.set_language("de", persist=False)
        add("    GameDef { key: %s, class_name: %s, color: %s, name: %s },"
            % (rs_str(cls.highscore_key), rs_str(name), rs_str(color),
               rs_str(cls.name)))
    add("];")
    add("")

    # ----- Sprachabhaengige Spielnamen -----------------------------------
    # Die meisten Spiele heissen ueberall gleich ("Snake"); ein paar nutzen
    # game_base.LocalizedName und heissen je Sprache anders ("Schach" /
    # "Chess" / "Echecs"). Nur diese Abweichungen wandern in die Tabelle.
    overrides = []
    for cls in ALL_GAMES:
        per_lang = []
        for code, _label in i18n.AVAILABLE:
            i18n.set_language(code, persist=False)
            per_lang.append((code, cls.name))
        i18n.set_language("de", persist=False)
        default = cls.name
        differing = [(c, n) for c, n in per_lang if n != default]
        if differing:
            overrides.append((cls.highscore_key, per_lang))
    i18n.set_language("de", persist=False)

    add("/// Die Namen eines Spiels je Sprache: `[(Sprachcode, Name)]`.")
    add("pub type NamesPerLang = &'static [(&'static str, &'static str)];")
    add("")
    add("/// Spielnamen, die sich je Sprache unterscheiden (z.B. Schach/Chess).")
    add("///")
    add("/// `(Spiel-Schluessel, Namen je Sprache)`. Spiele, die ueberall")
    add("/// gleich heissen, stehen hier nicht - fuer sie gilt `GameDef::name`.")
    add("pub const GAME_NAMES: &[(&str, NamesPerLang)] = &[")
    for key, per_lang in overrides:
        pairs = ", ".join("(%s, %s)" % (rs_str(c), rs_str(n)) for c, n in per_lang)
        add("    (%s, &[%s])," % (rs_str(key), pairs))
    add("];")
    add("")

    # ----- Beschreibungs-Schluessel der sammlungsweiten Erfolge -----------
    add("/// Zu welchem Text-Schluessel ein Statistik-Feld gehoert.")
    add("pub const STAT_DESC: &[(&str, &str)] = &[")
    for stat, key in achievements._STAT_DESC.items():
        add("    (%s, %s)," % (rs_str(stat), rs_str(key)))
    add("];")
    add("")

    # ----- Seltenheit ----------------------------------------------------
    add("/// Seltenheitsstufe eines Erfolgs: `(id, Punkte, Farbe)`.")
    add("pub const RARITY: &[(&str, i64, &str)] = &[")
    for key in achievements.RARITY_ORDER:
        entry = achievements.RARITY[key]
        add("    (%s, %d, %s),"
            % (rs_str(key), entry["points"], rs_str(entry["color"])))
    add("];")
    add("")
    add("/// Partien fuer den \"Stammgast\"-Erfolg je Spiel.")
    add("pub const REGULAR_PLAYS: i64 = %d;" % achievements.REGULAR_PLAYS)
    add("")

    # ----- Sammlungsweite Erfolge ---------------------------------------
    add("/// Sammlungsweite Erfolge: `(id, Symbol, Statistik-Feld, Ziel, Seltenheit)`.")
    add("///")
    add("/// Ein leeres Statistik-Feld bzw. `None` als Ziel kennzeichnet einen")
    add("/// reinen Ereignis-Erfolg (wird ueber `event()` freigeschaltet).")
    add("pub const GENERAL: &[(&str, &str, &str, Option<i64>, &str)] = &[")
    for ach_id, icon, stat, target, rarity in achievements.GENERAL:
        if ach_id == "explorer_all":
            target = len(ALL_GAMES)
        add("    (%s, %s, %s, %s, %s),"
            % (rs_str(ach_id), rs_str(icon), rs_str(stat),
               rs_opt_i64(target), rs_str(rarity)))
    add("];")
    add("")

    # ----- Punkte-Meilensteine ------------------------------------------
    add("/// Punkte-Meilenstein je Spiel (`score_<key>`).")
    add("pub const MILESTONES: &[(&str, i64)] = &[")
    for key, target in achievements.MILESTONES.items():
        add("    (%s, %d)," % (rs_str(key), target))
    add("];")
    add("")
    add("/// Zweite Punktstufe je Spiel (`master_<key>`).")
    add("pub const MASTERS: &[(&str, i64)] = &[")
    for key, target in achievements.MASTERS.items():
        add("    (%s, %d)," % (rs_str(key), target))
    add("];")
    add("")

    # ----- Besondere Momente --------------------------------------------
    add("/// Besondere Momente: `(id, Symbol, Spiel-Schluessel, Ziel, Seltenheit)`.")
    add("///")
    add("/// `Ziel = None` schaltet bei `event(id)` sofort frei; eine Zahl")
    add("/// verlangt `event(id, wert)` mit `wert >= Ziel`.")
    add("pub const SPECIALS: &[(&str, &str, &str, Option<i64>, &str)] = &[")
    for ach_id, icon, game_key, target, rarity in achievements.SPECIALS:
        add("    (%s, %s, %s, %s, %s),"
            % (rs_str(ach_id), rs_str(icon), rs_str(game_key),
               rs_opt_i64(target), rs_str(rarity)))
    add("];")
    add("")

    # ----- Spiel-Optionen ------------------------------------------------
    # Die Standardwerte der spielspezifischen Abschnitte aus settings.py.
    # Sie muessen genau stimmen: settings.rs verwirft beim Laden alles, was
    # hier nicht steht - genau wie settings.SCHEMAS in Python. Geraten waere
    # fatal, deshalb kommen sie aus der Quelle.
    import settings as settings_mod
    bekannt = {"theme", "sound", "volume", "haptik", "resolution",
               "auto_resolution", "fps", "controls"}
    spiel_optionen = {k: v for k, v in settings_mod.DEFAULTS.items()
                      if k not in bekannt and isinstance(v, dict)}
    add("/// Standardwerte der spielspezifischen Abschnitte aus `settings.py`.")
    add("///")
    add("/// Als JSON, weil die Werte gemischte Typen haben (Wahrheitswerte,")
    add("/// Zahlen, Text, verschachtelte Tabellen). `settings::load` liest das")
    add("/// einmal ein. **Nur was hier steht, ueberlebt das Laden** - genau")
    add("/// wie `settings.SCHEMAS` in der Python-Fassung.")
    # Reihenfolge wie DEFAULTS (nicht sortiert): so schreibt settings.py die
    # Datei, und so schreibt sie auch settings.rs.
    add("pub const GAME_DEFAULTS_JSON: &str = r##\"%s\"##;"
        % json.dumps(spiel_optionen, ensure_ascii=False))
    add("")

    # ----- UI-Designs ----------------------------------------------------
    add("/// Farbpalette eines UI-Designs (Reihenfolge wie `ui._*_COLORS`).")
    add("#[derive(Debug, Clone, Copy, PartialEq, Eq)]")
    add("pub struct Palette {")
    for key in COLOR_KEYS:
        add("    pub %s: Rgb," % key.lower())
    add("}")
    add("")

    add("/// Effekt-Schalter eines UI-Designs (Gegenstueck zu `ui._*_FX`).")
    add("///")
    add("/// Nicht jedes Design kennt jeden Schalter - fehlende stehen hier")
    add("/// auf dem ruhigen Standardwert bzw. `None`.")
    add("#[derive(Debug, Clone, Copy, PartialEq)]")
    add("pub struct Fx {")
    add("    /// Sternenfeld zeichnen, mit Helligkeit und Anzahl.")
    add("    pub stars: bool,")
    add("    pub star_bright: f64,")
    add("    pub star_count: Option<i32>,")
    add("    /// Aurora-Lichter und Sternschnuppen.")
    add("    pub aurora: bool,")
    add("    pub shooting: bool,")
    add("    /// Saturn und Schwarzes Loch (UI v4.1).")
    add("    pub celestial: bool,")
    add("    /// Zickzack-Kachelmuster als Hintergrund: `(Haupt-, Nebenfarbe)`.")
    add("    /// (Haupt-, Nebenfarbe)")
    add("    pub pattern: Option<(Rgb, Rgb)>,")
    add("    /// Staerke der Vignette (0 = aus).")
    add("    pub vignette: u8,")
    add("    /// Titel mit Schein bzw. Verlaufstext.")
    add("    pub title_glow: bool,")
    add("    pub title_grad: bool,")
    add("    /// Buttons mit Aussen-Schein bzw. Pfeil.")
    add("    pub btn_glow: bool,")
    add("    pub btn_arrow: bool,")
    add("    /// Funken-Partikel bei Klicks.")
    add("    pub sparks: bool,")
    add("    /// Scanlinie beim Bildwechsel, und wie lange der Wechsel dauert.")
    add("    pub scanline: bool,")
    add("    pub trans_dur: f64,")
    add("    /// Eckenrundungen und Schlagschatten.")
    add("    pub panel_radius: i32,")
    add("    pub btn_radius: i32,")
    add("    pub shadow_alpha: u8,")
    add("    /// Wie stark das Logo im Menue schwebt.")
    add("    pub menu_bob: i32,")
    add("    /// Eigener Zeichenpfad des Designs (\"v42\", \"v2\", \"v1\" oder \"\").")
    add("    pub style: &'static str,")
    add("    /// Glas-Panels (UI v4.2): Deckkraft.")
    add("    pub glass_alpha: Option<u8>,")
    add("    /// Filmkorn (UI v4.2): Staerke.")
    add("    pub grain: Option<i32>,")
    add("    /// Drei-Farben-Verlauf fuer Titel und Linien.")
    add("    pub grad3: Option<[Rgb; 3]>,")
    add("    /// Farbwolken des Hintergrunds (UI v4.2).")
    add("    pub mesh: &'static [MeshBlob],")
    add("    pub mesh_drift: Option<(f64, f64)>,")
    add("    /// Logo-Effekte der aelteren Designs.")
    add("    pub logo_glow: bool,")
    add("    pub logo_halo: bool,")
    add("    pub menu_orbit: bool,")
    add("}")
    add("")
    add("/// Eine Farbwolke des v4.2-Hintergrunds:")
    add("/// `(Farbe, Groesse rel. max(w,h), Tempo, Phase, x, y)` wie `ui._V42_MESH`.")
    add("#[derive(Debug, Clone, Copy, PartialEq)]")
    add("pub struct MeshBlob {")
    add("    pub color: Rgb,")
    add("    pub size: f64,")
    add("    pub speed: f64,")
    add("    pub phase: f64,")
    add("    pub x: f64,")
    add("    pub y: f64,")
    add("}")
    add("")

    add("/// Farben der Seitenleiste (Gegenstueck zu `ui._TK_*`, in Python Hex).")
    add("#[derive(Debug, Clone, Copy, PartialEq, Eq)]")
    add("pub struct TkPalette {")
    for key in TK_KEYS:
        add("    pub %s: Rgb," % key.lower())
    add("}")
    add("")
    add("/// Ein komplettes UI-Design.")
    add("#[derive(Debug, Clone, Copy, PartialEq)]")
    add("pub struct Theme {")
    add("    pub name: &'static str,")
    add("    pub palette: Palette,")
    add("    pub fx: Fx,")
    add("    pub tk: TkPalette,")
    add("}")
    add("")

    # Farbwolken je Design als eigene Konstanten (Fx verweist darauf).
    for name in ui.THEME_NAMES:
        mesh = ui.THEMES[name][1].get("mesh")
        if not mesh:
            continue
        add("const MESH_%s: &[MeshBlob] = &[" % name.upper())
        for color, size, speed, phase, mx, my in mesh:
            add("    MeshBlob { color: %s, size: %s, speed: %s, phase: %s, x: %s, y: %s },"
                % (rs_rgb(color), rs_f32(size), rs_f32(speed), rs_f32(phase),
                   rs_f32(mx), rs_f32(my)))
        add("];")
        add("")

    add("/// Alle %d UI-Designs in Anzeige-Reihenfolge." % len(ui.THEME_NAMES))
    add("pub const THEMES: &[Theme] = &[")
    for name in ui.THEME_NAMES:
        colors, fx, _tk = ui.THEMES[name]
        add("    Theme {")
        add("        name: %s," % rs_str(name))
        add("        palette: Palette {")
        for key in COLOR_KEYS:
            add("            %s: %s," % (key.lower(), rs_rgb(colors[key])))
        add("        },")
        add("        fx: Fx {")
        add("            stars: %s," % rs_bool(fx.get("stars")))
        add("            star_bright: %s," % rs_f32(fx.get("star_bright", 0.0)))
        add("            star_count: %s," % rs_opt_i64(fx.get("star_count")))
        add("            aurora: %s," % rs_bool(fx.get("aurora")))
        add("            shooting: %s," % rs_bool(fx.get("shooting")))
        add("            celestial: %s," % rs_bool(fx.get("celestial")))
        pattern = fx.get("pattern")
        add("            pattern: %s," % (
            "None" if not pattern
            else "Some((%s, %s))" % (rs_rgb(pattern[0]), rs_rgb(pattern[1]))))
        add("            vignette: %d," % int(fx.get("vignette", 0)))
        add("            title_glow: %s," % rs_bool(fx.get("title_glow")))
        add("            title_grad: %s," % rs_bool(fx.get("title_grad", True)))
        add("            btn_glow: %s," % rs_bool(fx.get("btn_glow")))
        add("            btn_arrow: %s," % rs_bool(fx.get("btn_arrow")))
        add("            sparks: %s," % rs_bool(fx.get("sparks")))
        add("            scanline: %s," % rs_bool(fx.get("scanline")))
        add("            trans_dur: %s," % rs_f32(fx.get("trans_dur", 0.25)))
        add("            panel_radius: %d," % int(fx.get("panel_radius", 10)))
        add("            btn_radius: %d," % int(fx.get("btn_radius", 8)))
        add("            shadow_alpha: %d," % int(fx.get("shadow_alpha", 0)))
        add("            menu_bob: %d," % int(fx.get("menu_bob", 0)))
        add("            style: %s," % rs_str(fx.get("style") or ""))
        add("            glass_alpha: %s," % rs_opt_i64(fx.get("glass_alpha")))
        add("            grain: %s," % rs_opt_i64(fx.get("grain")))
        grad3 = fx.get("grad3")
        add("            grad3: %s," % (
            "None" if not grad3
            else "Some([%s])" % ", ".join(rs_rgb(c) for c in grad3)))
        add("            mesh: %s," % (
            "MESH_%s" % name.upper() if fx.get("mesh") else "&[]"))
        drift = fx.get("mesh_drift")
        add("            mesh_drift: %s," % (
            "None" if not drift
            else "Some((%s, %s))" % (rs_f32(drift[0]), rs_f32(drift[1]))))
        add("            logo_glow: %s," % rs_bool(fx.get("logo_glow", True)))
        add("            logo_halo: %s," % rs_bool(fx.get("logo_halo")))
        add("            menu_orbit: %s," % rs_bool(fx.get("menu_orbit", True)))
        add("        },")
        add("        tk: TkPalette {")
        for key in TK_KEYS:
            add("            %s: %s," % (key.lower(), rs_rgb(ui.hex_to_rgb(_tk[key]))))
        add("        },")
        add("    },")
    add("];")
    add("")
    add("/// Standard-Design beim ersten Start.")
    add("pub const DEFAULT_THEME: &str = %s;" % rs_str(ui.DEFAULT_THEME))
    add("")

    # ----- Probe ---------------------------------------------------------
    total = len(achievements.all_defs())
    add("/// Gesamtzahl der Erfolge - Probe gegen die Python-Fassung.")
    add("pub const TOTAL_ACHIEVEMENTS: usize = %d;" % total)
    add("")
    add("/// Gesamtzahl der Spiele - Probe gegen die Python-Fassung.")
    add("pub const TOTAL_GAMES: usize = %d;" % len(ALL_GAMES))

    text = "\n".join(lines) + "\n"
    with open(OUT, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)
    print("geschrieben: %s" % os.path.relpath(OUT, ROOT))
    print("  %d Spiele, %d Erfolge" % (len(ALL_GAMES), total))


if __name__ == "__main__":
    main()
