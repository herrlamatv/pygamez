# -*- coding: utf-8 -*-
"""
rust/tools/audit.py
===================
Prüfstand für die Rust-Fassung - das Gegenstück zu ``tests/arcade_casino_audit.py``
und ``web/tools/progress_check.js``.

Geprüft wird, was ein Rust-Test nicht sehen kann, weil es quer über Dateien
und Sprachen liegt:

1. **Übersetzungen**: Jeder Schlüssel, den der Rust-Quelltext über
   ``i18n::t("…")`` / ``tf("…")`` verlangt, muss in allen 14 Sprachdateien
   stehen. Fehlende Schlüssel erscheinen sonst im Klartext auf dem Bildschirm.
2. **Platzhalter**: Ein Schlüssel mit ``{name}`` muss mit ``tf`` aufgerufen
   werden, nicht mit ``t`` - sonst steht die Klammer auf dem Bildschirm.
3. **Spieleliste**: Jedes portierte Spiel muss in ``defs.rs`` stehen
   (gleicher Highscore-Schlüssel wie in Python).
4. **Erzeugte Datei**: ``defs.rs`` muss zu den Python-Quellen passen - sonst
   hat jemand ``export_defs.py`` nach einer Änderung nicht laufen lassen.
5. **Umlaute**: Im Rust-Quelltext sollen Kommentare ohne Umlaute auskommen
   (Projektkonvention: ae/oe/ue), sichtbare Texte kommen aus den Sprachdateien.

Aufruf (aus dem Projektordner):

    .venv/Scripts/python.exe rust/tools/audit.py
"""

import json
import os
import re
import sys

# Die Windows-Konsole steht standardmäßig auf cp1252 - dort zerbricht jede
# Meldung mit einem tschechischen oder türkischen Zeichen darin.
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
RUST = os.path.join(ROOT, "rust")
CRATES = os.path.join(RUST, "crates")

# Die 14 Sprachen und wo ihre Dateien liegen.
CORE_LANGS = ("de", "en", "fr", "es", "pt")
EXPANSION_LANGS = ("pl", "tr", "da", "no", "sv", "fi", "cs", "sl", "hr")

fehler = []
warnungen = []


def melde(text):
    fehler.append(text)


def warne(text):
    warnungen.append(text)


def lang_path(code):
    if code in CORE_LANGS:
        return os.path.join(ROOT, "lang", "%s.json" % code)
    return os.path.join(ROOT, "lang", "lang.expansion", "%s.json" % code)


def load_lang(code):
    with open(lang_path(code), "r", encoding="utf-8") as f:
        return json.load(f)


def rust_files():
    """Alle .rs-Dateien des Workspace (ohne erzeugte defs.rs)."""
    for base, dirs, files in os.walk(CRATES):
        dirs[:] = [d for d in dirs if d not in ("target", ".git")]
        for name in files:
            if name.endswith(".rs") and name != "defs.rs":
                yield os.path.join(base, name)


# Schlüssel aus t("…") und tf("…", …). Auch format!-Aufrufe wie
# t(&format!("snake.mode.{mode}")) werden erkannt - dort steht ein
# Platzhalter im Schlüssel, den wir als Präfix behandeln.
RE_T = re.compile(r'\bt\(\s*"([^"]+)"\s*\)')
RE_TF = re.compile(r'\btf\(\s*"([^"]+)"\s*,')
RE_T_FORMAT = re.compile(r'\bt\(&format!\(\s*"([^"]+)"')
# fill(&t("schluessel"), &[...]) füllt die Platzhalter genauso wie tf().
# Zwischen `fill(` und `t(` darf ein Zeilenumbruch samt Einrückung und der
# volle Modulpfad stehen - genau so schreibt es rustfmt bei langen Zeilen.
RE_FILL = re.compile(r'\bfill\(\s*&?\s*(?:pgz_core::i18n::)?t\(\s*"([^"]+)"')


def sammle_schluessel():
    """{schlüssel: (datei, mit_platzhaltern_aufgerufen)} über alle .rs-Dateien."""
    direkt = {}      # feste Schlüssel -> (datei, per_tf)
    praefixe = []    # (muster, datei) für dynamisch gebaute Schlüssel
    for path in rust_files():
        with open(path, "r", encoding="utf-8") as f:
            zeilen = f.readlines()
        # Kommentarzeilen enthalten Beispiele ("siehe t(\"snake.score\")"),
        # keine echten Aufrufe - die zaehlen nicht mit.
        text = "".join(z for z in zeilen if not z.lstrip().startswith(("//", "!")))
        rel = os.path.relpath(path, ROOT)
        for key in RE_T.findall(text):
            direkt.setdefault(key, (rel, False))
        for key in RE_TF.findall(text):
            direkt[key] = (rel, True)
        for key in RE_FILL.findall(text):
            direkt[key] = (rel, True)
        for muster in RE_T_FORMAT.findall(text):
            praefixe.append((muster, rel))
    return direkt, praefixe


def pruefe_uebersetzungen():
    direkt, praefixe = sammle_schluessel()
    if not direkt:
        melde("Es wurden gar keine i18n-Schlüssel gefunden - stimmt der Pfad?")
        return

    sprachen = {code: load_lang(code) for code in CORE_LANGS + EXPANSION_LANGS}
    de = sprachen["de"]

    for key, (datei, per_tf) in sorted(direkt.items()):
        fehlt = [code for code, data in sprachen.items() if key not in data]
        if fehlt:
            melde("Schlüssel %r (%s) fehlt in: %s" % (key, datei, ", ".join(sorted(fehlt))))
            continue
        # Platzhalter-Probe: {…} im Text, aber mit t() statt tf() geholt?
        hat_platzhalter = re.search(r"\{[a-zA-Z_][a-zA-Z0-9_]*\}", de[key])
        if hat_platzhalter and not per_tf:
            warne("Schlüssel %r (%s) hat Platzhalter %s, wird aber mit t() geholt"
                  % (key, datei, hat_platzhalter.group(0)))

    # Dynamische Schlüssel: Präfix muss mindestens einen Treffer haben.
    for muster, datei in praefixe:
        praefix = muster.split("{")[0]
        if not praefix:
            continue
        treffer = [k for k in de if k.startswith(praefix)]
        if not treffer:
            melde("Kein Schlüssel beginnt mit %r (%s, aus format!)" % (praefix, datei))

    print("  %d feste und %d gebaute Schlüssel geprüft"
          % (len(direkt), len(praefixe)))


def pruefe_spiele():
    """Jedes in ALL_GAMES eingetragene Spiel muss es in defs.rs geben."""
    lib = os.path.join(CRATES, "pgz-games", "src", "lib.rs")
    defs = os.path.join(CRATES, "pgz-core", "src", "defs.rs")
    with open(lib, "r", encoding="utf-8") as f:
        lib_text = f.read()
    with open(defs, "r", encoding="utf-8") as f:
        defs_text = f.read()

    alle_module = set(re.findall(r"^pub mod (\w+);", lib_text, re.M))
    # lib.rs wird von gen_games_list.py geschrieben: all_games() traegt jedes
    # Spiel als <modul>::INFO ein; Hilfsmodule (Kern, Zeichnen, Editor, ...)
    # stehen nur als "pub mod" darin und brauchen kein INFO.
    block = re.search(r"pub fn all_games\(\) -> Vec<GameInfo> \{(.*?)\n\}", lib_text, re.S)
    eintraege = set(re.findall(r"(\w+)::INFO", block.group(1))) if block else set()
    offen = re.findall(r"// (\w+) \((\w+)\) - noch nicht uebertragen", lib_text)
    if offen:
        melde("Noch nicht uebertragen: %s" % ", ".join(m for m, _k in offen))
    zuviel = eintraege - alle_module
    if zuviel:
        melde("Eintrag in all_games() ohne Modul: %s" % ", ".join(sorted(zuviel)))
    module = eintraege
    print("  lib.rs: %d Spiele in all_games(), %d Hilfsmodule"
          % (len(eintraege), len(alle_module - eintraege)))

    # Highscore-Schlüssel jedes Spiels gegen defs.rs prüfen.
    bekannte = set(re.findall(r'GameDef \{ key: "([^"]+)"', defs_text))
    for modul in sorted(module):
        path = os.path.join(CRATES, "pgz-games", "src", "%s.rs" % modul)
        if not os.path.isfile(path):
            melde("Modul %s hat keine Datei" % modul)
            continue
        with open(path, "r", encoding="utf-8") as f:
            text = f.read()
        # Der Schlüssel steht entweder direkt da (key: "snake") oder als
        # Konstante (key: KEY, mit `const KEY: &str = "connect4";` oben).
        m = re.search(r'pub const INFO: GameInfo = GameInfo \{\s*key: ([^,]+),', text)
        if not m:
            melde("%s.rs hat kein INFO mit key" % modul)
            continue
        roh = m.group(1).strip()
        if roh.startswith('"'):
            key = roh.strip('"')
        else:
            konst = re.search(r'const %s: &str = "([^"]+)"' % re.escape(roh), text)
            if not konst:
                melde("%s.rs: key ist %s, die Konstante findet sich nicht" % (modul, roh))
                continue
            key = konst.group(1)
        if key not in bekannte:
            melde("%s.rs nennt den Schlüssel %r - den kennt defs.rs nicht"
                  % (modul, key))

    print("  %d portierte Spiele geprüft (von %d insgesamt)"
          % (len(eintraege), len(bekannte)))


def pruefe_defs_aktuell():
    """defs.rs muss zu den Python-Quellen passen."""
    os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
    os.environ.setdefault("SDL_AUDIODRIVER", "dummy")
    sys.path.insert(0, ROOT)
    try:
        import achievements
        from games import ALL_GAMES
    except Exception as exc:                       # pragma: no cover
        warne("Python-Quellen nicht ladbar (%s) - defs.rs ungeprüft" % exc)
        return

    defs = os.path.join(CRATES, "pgz-core", "src", "defs.rs")
    with open(defs, "r", encoding="utf-8") as f:
        text = f.read()

    m = re.search(r"TOTAL_ACHIEVEMENTS: usize = (\d+);", text)
    if m and int(m.group(1)) != len(achievements.all_defs()):
        melde("defs.rs kennt %s Erfolge, Python hat %d - export_defs.py laufen lassen"
              % (m.group(1), len(achievements.all_defs())))
    m = re.search(r"TOTAL_GAMES: usize = (\d+);", text)
    if m and int(m.group(1)) != len(ALL_GAMES):
        melde("defs.rs kennt %s Spiele, Python hat %d - export_defs.py laufen lassen"
              % (m.group(1), len(ALL_GAMES)))
    print("  defs.rs: %d Spiele, %d Erfolge" % (len(ALL_GAMES), len(achievements.all_defs())))


def pruefe_umlaute():
    """Projektkonvention: im Rust-Quelltext keine Umlaute (ae/oe/ue schreiben)."""
    treffer = 0
    for path in rust_files():
        with open(path, "r", encoding="utf-8") as f:
            for nr, zeile in enumerate(f, 1):
                # Zeichenketten dürfen Sonderzeichen haben (z.B. Pfeile);
                # es geht nur um deutsche Umlaute in Kommentaren.
                # Zeichenketten und Zeichenliterale duerfen Sonderzeichen
                # tragen (z.B. die Akzent-Tabelle des Wortfilters); es geht
                # nur um deutsche Umlaute in Kommentaren und Bezeichnern.
                ohne_strings = re.sub(r'"[^"]*"', '""', zeile)
                ohne_strings = re.sub(r"'[^']*'", "''", ohne_strings)
                if re.search(r"[äöüÄÖÜß]", ohne_strings):
                    warne("%s:%d Umlaut im Quelltext: %s"
                          % (os.path.relpath(path, ROOT), nr, zeile.strip()[:70]))
                    treffer += 1
    print("  %d Zeilen mit Umlauten im Quelltext" % treffer)


def main():
    print("PyGameZ - Rust-Prüfstand")
    print("=" * 60)
    print("Übersetzungen:")
    pruefe_uebersetzungen()
    print("Spieleliste:")
    pruefe_spiele()
    print("Erzeugte Stammdaten:")
    pruefe_defs_aktuell()
    print("Quelltext-Konventionen:")
    pruefe_umlaute()

    print("=" * 60)
    for w in warnungen:
        print("WARNUNG: %s" % w)
    for f in fehler:
        print("FEHLER:  %s" % f)
    if fehler:
        print("\n%d Fehler, %d Warnungen" % (len(fehler), len(warnungen)))
        return 1
    print("\nAlles in Ordnung (%d Warnungen)" % len(warnungen))
    return 0


if __name__ == "__main__":
    sys.exit(main())
