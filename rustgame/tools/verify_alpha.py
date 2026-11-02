# -*- coding: utf-8 -*-
"""
rustgame/tools/verify_alpha.py
==============================
Nagelt die **Misch-Formeln** von pygame-ce fest, bevor eine Zeile Rust
entsteht.

Warum das zuerst kommt: Der Blit ist der heisseste und meistbenutzte Pfad
(1395 Aufrufstellen im Projekt). Stimmt die Formel auch nur um eins daneben,
ist jedes spaetere Referenzbild falsch - und der Fehler faellt erst auf, wenn
schon hundert Funktionen darauf aufbauen.

Die naheliegende Formel ist **falsch**::

    out = (src*a + dst*(255-a)) / 255        # dst=10 src=200 a=1  -> 10
    pygame liefert aber                                            -> 11

**Seit dem Blit-Umbau prueft dieses Skript nur noch, es schreibt nicht
mehr.** `blend_formula.rs` enthaelt inzwischen **vier** Rechenwege - pygame
waehlt fuer denselben Aufruf `blit(..., special_flags=0)` je nach Quelle,
Schluesselfarbe und Mischmodus zwischen einer eigenen Routine und drei
SDL-Blittern, und jede rundet anders. Dieses Skript kennt nur die erste;
ein Schreibvorgang wuerde die anderen drei loeschen.

Geprueft werden mehrere Kandidaten gegen das echte pygame, und zwar
**exhaustiv** ueber alle 256^3 Kombinationen von (dst, src, alpha) - nicht
stichprobenartig. Das dauert ein paar Minuten und ist danach fuer immer
erledigt.

Aufruf (aus dem Projektordner):

    .venv/Scripts/python.exe rustgame/tools/verify_alpha.py

Das Ergebnis landet als Rust-Quelltext in
``rustgame/crates/rustgame-core/src/blend_formula.rs`` - damit kann die
Rust-Seite gar nicht erst abweichen.
"""

import os
import sys

os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
os.environ.setdefault("SDL_AUDIODRIVER", "dummy")

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

import pygame  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "rustgame", "crates", "rustgame-core", "src", "blend_formula.rs")


# ---------------------------------------------------------------------------
# Die Kandidaten
# ---------------------------------------------------------------------------

def naiv(dst, src, a):
    """Die Lehrbuch-Formel. Rundet ab und liegt deshalb oft um eins daneben."""
    return (src * a + dst * (255 - a)) // 255


def gerundet(dst, src, a):
    """Lehrbuch-Formel mit kaufmaennischer Rundung."""
    return (src * a + dst * (255 - a) + 127) // 255


def sdl_shift(dst, src, a):
    """SDLs Variante: +255 und dann durch 256 teilen."""
    return (src * a + dst * (255 - a) + 255) >> 8


def sdl_delta(dst, src, a):
    """SDLs BLIT-Makro in der Differenzform (was SDL_blit_A.c wirklich tut)."""
    t = (src - dst) * a + 0x80
    return dst + ((t + (t >> 8)) >> 8)


def pygame_256(dst, src, a):
    """Was pygame-ce wirklich rechnet.

    Eine schnelle Naeherung auf der 256er-Skala statt auf 255: die Quelle
    wiegt ``a + 1``, das Ziel ``256 - a``, und geteilt wird per Schieben.
    Das ist exakt, wo es exakt sein muss (``a = 0`` laesst das Ziel stehen,
    ``a = 255`` nimmt die Quelle) und spart die Division durch 255.
    """
    return (src * (a + 1) + dst * (256 - a)) >> 8


KANDIDATEN = [
    ("naiv", naiv),
    ("gerundet", gerundet),
    ("sdl_shift", sdl_shift),
    ("sdl_delta", sdl_delta),
    ("pygame_256", pygame_256),
]


# ---------------------------------------------------------------------------
# Was pygame wirklich tut
# ---------------------------------------------------------------------------

def pygame_tabelle():
    """Misst pygames Alpha-Blit fuer alle (dst, src, alpha).

    Statt 16,7 Mio. Einzel-Blits (viel zu langsam) wird pro Alphawert **ein**
    Blit ueber eine 256x256-Flaeche gefahren: x = Quellwert, y = Zielwert.
    Das sind 256 Blits statt 16,7 Millionen.

    Rueckgabe: ``tabelle[a][dst][src]`` als verschachtelte Listen.
    """
    breite = hoehe = 256
    tabelle = []

    # Achtung: `draw.line` mit einer Alpha-Farbe **blendet** seit pygame 2.0,
    # fuellt also nicht. Die Pruefflaechen muessen deshalb mit `fill` gebaut
    # werden - das setzt die Pixel hart. Genau hier ist die erste Messung
    # dieses Skripts schiefgegangen.
    vorlage_src = pygame.Surface((breite, hoehe), pygame.SRCALPHA)
    vorlage_dst = pygame.Surface((breite, hoehe))      # ohne SRCALPHA = undurchsichtig
    for y in range(hoehe):
        # Eine Zeile je Zielwert.
        vorlage_dst.fill((y, y, y), pygame.Rect(0, y, breite, 1))

    for a in range(256):
        src = vorlage_src
        for x in range(breite):
            # Eine Spalte je Quellwert; Alpha ist fuer die ganze Flaeche gleich.
            src.fill((x, x, x, a), pygame.Rect(x, 0, 1, hoehe))
        dst = vorlage_dst.copy()
        dst.blit(src, (0, 0))

        roh = pygame.image.tobytes(dst, "RGB")
        zeile_a = []
        for y in range(hoehe):
            basis = y * breite * 3
            zeile_a.append([roh[basis + x * 3] for x in range(breite)])
        tabelle.append(zeile_a)
        if a % 32 == 0:
            print("    Alpha %3d ..." % a, flush=True)
    return tabelle


def pruefe(tabelle):
    """Vergleicht jeden Kandidaten mit der gemessenen Tabelle."""
    ergebnis = []
    for name, fn in KANDIDATEN:
        fehler = 0
        erster = None
        for a in range(256):
            for d in range(256):
                zeile = tabelle[a][d]
                for s in range(256):
                    if fn(d, s, a) != zeile[s]:
                        fehler += 1
                        if erster is None:
                            erster = (d, s, a, fn(d, s, a), zeile[s])
        ergebnis.append((name, fehler, erster))
        if fehler == 0:
            print("  %-12s PASST (0 Abweichungen von 16.777.216)" % name)
        else:
            d, s, a, war, soll = erster
            print("  %-12s %d Abweichungen, zuerst dst=%d src=%d a=%d -> %d statt %d"
                  % (name, fehler, d, s, a, war, soll))
    return ergebnis


# ---------------------------------------------------------------------------
# Ziel-Alpha bei RGBA auf RGBA
# ---------------------------------------------------------------------------

def miss_ziel_alpha():
    """Wie entsteht der Alphakanal, wenn auf eine SRCALPHA-Flaeche geblittet wird?"""
    proben = []
    for da in (0, 1, 64, 128, 200, 254, 255):
        for sa in (0, 1, 64, 128, 200, 254, 255):
            dst = pygame.Surface((2, 2), pygame.SRCALPHA)
            src = pygame.Surface((2, 2), pygame.SRCALPHA)
            dst.fill((10, 20, 30, da))
            src.fill((200, 100, 50, sa))
            dst.blit(src, (0, 0))
            proben.append((da, sa, dst.get_at((0, 0))[3]))
    return proben


def pruefe_ziel_alpha(proben):
    def kandidat(da, sa):
        return da + sa - (da * sa) // 255

    fehler = [(da, sa, ist, kandidat(da, sa)) for da, sa, ist in proben
              if kandidat(da, sa) != ist]
    if not fehler:
        print("  dstA + srcA - dstA*srcA/255   PASST (%d Proben)" % len(proben))
        return True
    for da, sa, ist, war in fehler[:5]:
        print("  Abweichung: dstA=%d srcA=%d -> %d, Formel sagt %d" % (da, sa, ist, war))
    return False


# ---------------------------------------------------------------------------
# Rust-Datei schreiben
# ---------------------------------------------------------------------------

def schreibe_rust(sieger, ziel_alpha_ok):
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    zeilen = [
        "//! blend_formula.rs",
        "//! =================",
        "//! **Erzeugte Datei - nicht von Hand aendern.**",
        "//!",
        "//! Quelle: `rustgame/tools/verify_alpha.py`. Das Skript hat die",
        "//! Mischformel von pygame-ce %s (SDL %s) **exhaustiv** ueber alle"
        % (pygame.version.ver, ".".join(map(str, pygame.get_sdl_version()))),
        "//! 16.777.216 Kombinationen von (dst, src, alpha) geprueft.",
        "//!",
        "//! Die naheliegende Formel `(src*a + dst*(255-a)) / 255` ist **falsch**:",
        "//! bei dst=10, src=200, a=1 liefert sie 10, pygame aber 11.",
        "",
        "/// Mischt einen Farbkanal nach pygames Regel.",
        "///",
        "/// Exhaustiv gegen pygame-ce geprueft - Kandidat `%s`." % sieger,
        "#[inline(always)]",
        "pub fn blend_channel(dst: u8, src: u8, alpha: u8) -> u8 {",
    ]
    if sieger == "pygame_256":
        zeilen += [
            "    // pygame rechnet auf der 256er-Skala statt auf 255: die Quelle",
            "    // wiegt (a + 1), das Ziel (256 - a), geteilt wird per Schieben.",
            "    // Exakt an den Raendern (a=0 laesst dst stehen, a=255 nimmt src)",
            "    // und ohne Division durch 255.",
            "    let s = src as u32 * (alpha as u32 + 1);",
            "    let d = dst as u32 * (256 - alpha as u32);",
            "    ((s + d) >> 8) as u8",
        ]
    elif sieger == "sdl_delta":
        zeilen += [
            "    // SDLs BLIT-Makro in Differenzform (SDL_blit_A.c).",
            "    let t = (src as i32 - dst as i32) * alpha as i32 + 0x80;",
            "    (dst as i32 + ((t + (t >> 8)) >> 8)) as u8",
        ]
    elif sieger == "sdl_shift":
        zeilen += [
            "    let s = src as u32 * alpha as u32;",
            "    let d = dst as u32 * (255 - alpha as u32);",
            "    (((s + d + 255) >> 8) & 0xFF) as u8",
        ]
    elif sieger == "gerundet":
        zeilen += [
            "    let s = src as u32 * alpha as u32;",
            "    let d = dst as u32 * (255 - alpha as u32);",
            "    ((s + d + 127) / 255) as u8",
        ]
    else:
        zeilen += [
            "    let s = src as u32 * alpha as u32;",
            "    let d = dst as u32 * (255 - alpha as u32);",
            "    ((s + d) / 255) as u8",
        ]
    zeilen += [
        "}",
        "",
        "/// Das Ziel-Alpha, wenn auf eine Flaeche mit Alphakanal geblittet wird.",
        "#[inline(always)]",
        "pub fn blend_alpha(dst_a: u8, src_a: u8) -> u8 {",
        "    let d = dst_a as u32;",
        "    let s = src_a as u32;",
        "    (d + s - (d * s) / 255) as u8",
        "}" if ziel_alpha_ok else "    unimplemented!(\"Ziel-Alpha ungeklaert\")\n}",
        "",
        "#[cfg(test)]",
        "mod tests {",
        "    use super::*;",
        "",
        "    /// Die Messwerte, an denen die naive Formel scheitert.",
        "    #[test]",
        "    fn stolpersteine_stimmen() {",
        "        assert_eq!(blend_channel(10, 200, 1), 11, \"die naive Formel sagt 10\");",
        "        assert_eq!(blend_channel(255, 0, 128), 127);",
        "        assert_eq!(blend_channel(0, 255, 128), 128);",
        "        assert_eq!(blend_channel(10, 200, 254), 199);",
        "    }",
        "",
        "    /// Die Randfaelle muessen sauber durchreichen.",
        "    #[test]",
        "    fn raender_sind_exakt() {",
        "        for v in 0..=255u8 {",
        "            assert_eq!(blend_channel(v, 99, 0), v, \"alpha=0 laesst dst stehen\");",
        "            assert_eq!(blend_channel(99, v, 255), v, \"alpha=255 nimmt src\");",
        "        }",
        "    }",
        "",
        "    #[test]",
        "    fn ziel_alpha_rechnet_richtig() {",
        "        assert_eq!(blend_alpha(0, 0), 0);",
        "        assert_eq!(blend_alpha(255, 0), 255);",
        "        assert_eq!(blend_alpha(0, 255), 255);",
        "        assert_eq!(blend_alpha(200, 128), 228);",
        "    }",
        "}",
        "",
    ]
    with open(OUT, "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(zeilen))
    print("\ngeschrieben: %s" % os.path.relpath(OUT, ROOT))


def main():
    pygame.init()
    print("pygame-ce %s (SDL %s)" % (pygame.version.ver,
                                     ".".join(map(str, pygame.get_sdl_version()))))
    print()
    print("Messe pygames Alpha-Blit (256 Blits ueber je 256x256)...")
    tabelle = pygame_tabelle()

    print()
    print("Pruefe die Kandidaten exhaustiv (16.777.216 Kombinationen):")
    ergebnis = pruefe(tabelle)
    sieger = [name for name, fehler, _ in ergebnis if fehler == 0]

    print()
    print("Ziel-Alpha bei RGBA auf RGBA:")
    ziel_ok = pruefe_ziel_alpha(miss_ziel_alpha())

    if not sieger:
        print()
        print("FEHLER: Kein Kandidat passt. Die Formel muss von Hand gesucht werden.")
        print("Hilfreiche Stellen aus der Messung:")
        for d in (0, 10, 128, 255):
            for s in (0, 200, 255):
                for a in (1, 128, 254):
                    print("   dst=%3d src=%3d a=%3d -> %3d" % (d, s, a, tabelle[a][d][s]))
        return 1

    print()
    print("Sieger: %s" % ", ".join(sieger))

    # **Nicht mehr schreiben.**
    #
    # Dieses Skript hat `blend_formula.rs` einmal erzeugt, und das war
    # richtig, solange es *eine* Formel gab. Inzwischen sind es vier: fuer
    # `blit(..., special_flags=0)` waehlt pygame je nach Quelle,
    # Schluesselfarbe und Mischmodus zwischen einer eigenen Routine und
    # drei SDL-Blittern, und jede rundet anders. Dieses Skript kennt nur
    # die erste davon - ein Lauf wuerde die anderen drei stillschweigend
    # loeschen.
    #
    # Also prueft es nur noch. Wer die Datei neu erzeugen will, muss
    # vorher alle vier Wege hier abbilden.
    with open(OUT, "r", encoding="utf-8") as f:
        vorhanden = f.read()
    kern = "let s = src as u32 * (alpha as u32 + 1);"
    if kern in vorhanden:
        print("blend_formula.rs enthaelt die gemessene Formel - unveraendert gelassen.")
        return 0
    print("ACHTUNG: blend_formula.rs enthaelt die Formel nicht mehr.")
    print("Erwartet wurde die Zeile:")
    print("   %s" % kern)
    print("Diese Datei wird **von Hand** gepflegt (vier Rechenwege) -")
    print("bitte nachsehen, statt sie neu erzeugen zu lassen.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
