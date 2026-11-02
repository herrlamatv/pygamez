# -*- coding: utf-8 -*-
"""
rustgame/tools/gen_golden.py
============================
Erzeugt die **goldenen Referenzbilder** mit echtem pygame-ce.

Jeder Fall aus ``cases.py`` wird kopflos (``SDL_VIDEODRIVER=dummy``)
gezeichnet und als PNG unter ``rustgame/tests/golden/`` abgelegt, zusammen
mit einem ``manifest.json``, das pygame-Version, SDL-Version, Plattform und
je Fall die Toleranz festhaelt.

Die Rust-Seite spielt dieselben Faelle nach und vergleicht Pixel fuer Pixel.
Das ist der Beweis dafuer, dass rustgame "genau so funktioniert" - nicht
eine Behauptung, sondern eine Messung.

Aufruf (aus dem Projektordner):

    .venv/Scripts/python.exe rustgame/tools/gen_golden.py

Die Bilder gehoeren ins Repository: sie sind der Vertrag, gegen den
gearbeitet wird.
"""

import json
import os
import platform
import sys

os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
os.environ.setdefault("SDL_AUDIODRIVER", "dummy")

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

HIER = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HIER)

import pygame  # noqa: E402
from cases import FAELLE, gruppen  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(HIER))
ZIEL = os.path.join(ROOT, "rustgame", "tests", "golden")


def flagwert(name):
    """Wandelt einen Flag-Namen aus der Falltabelle in pygames Zahl."""
    if not name:
        return 0
    return getattr(pygame, name) if isinstance(name, str) else name


def muster(flaeche, w, h):
    """Ein wiedererkennbares Testmuster.

    Bewusst ohne Zufall und ohne Kantenglaettung: vier farbige Viertel plus
    ein Raster, damit Skalieren, Spiegeln und Drehen sichtbar falsch
    aussehen, wenn sie falsch sind.
    """
    hilf = pygame.Surface((w, h))
    for y in range(h):
        for x in range(w):
            r = 255 if x < w // 2 else 40
            g = 255 if y < h // 2 else 40
            b = 200 if (x + y) % 3 == 0 else 60
            hilf.set_at((x, y), (r, g, b))
    flaeche.blit(pygame.transform.scale(hilf, flaeche.get_size()), (0, 0))
    return flaeche


def fuehre_aus(fall):
    """Spielt einen Fall mit echtem pygame durch und gibt die Flaeche zurueck."""
    flags = flagwert(fall["flags"])
    s = pygame.Surface(fall["size"], flags)
    # Eine frische Flaeche ohne SRCALPHA ist schwarz, mit SRCALPHA durchsichtig.

    for op in fall["ops"]:
        name = op[0]
        if name == "fill":
            s.fill(op[1])
        elif name == "fill_rect":
            s.fill(op[1], pygame.Rect(*op[2]))
        elif name == "set_clip":
            s.set_clip(pygame.Rect(*op[1]))
        elif name == "muster":
            s = muster(s, op[1], op[2])
        elif name == "blit_gefuellt":
            farbe, ziel, flag = op[1], op[2], op[3]
            q = pygame.Surface(s.get_size(), pygame.SRCALPHA)
            q.fill(farbe)
            s.blit(q, ziel, special_flags=flagwert(flag))
        elif name == "blit_gefuellt_alpha":
            farbe, ziel, alpha = op[1], op[2], op[3]
            q = pygame.Surface(s.get_size(), pygame.SRCALPHA)
            q.fill(farbe)
            q.set_alpha(alpha)
            s.blit(q, ziel)
        elif name == "blit_muster":
            # (qw, qh), srcalpha, alpha|None, colorkey|None, (x,y), area|None, flag
            (qw, qh), srcalpha, alpha, ck, ziel, bereich, flag = op[1:]
            q = pygame.Surface((qw, qh), pygame.SRCALPHA if srcalpha else 0)
            muster(q, qw, qh)
            if alpha is not None:
                q.set_alpha(alpha)
            if ck is not None:
                q.set_colorkey(ck)
            s.blit(q, ziel,
                   pygame.Rect(*bereich) if bereich else None,
                   special_flags=flagwert(flag))
        elif name == "subsurface_fill":
            teil = s.subsurface(pygame.Rect(*op[1]))
            teil.fill(op[2])
        elif name == "scroll":
            s.scroll(op[1], op[2])
        elif name == "convert":
            s = s.convert()
        elif name == "convert_alpha":
            s = s.convert_alpha()
        elif name == "rect":
            farbe, r, breite, radius = op[1], op[2], op[3], op[4]
            if radius >= 0:
                pygame.draw.rect(s, farbe, pygame.Rect(*r), breite, border_radius=radius)
            else:
                pygame.draw.rect(s, farbe, pygame.Rect(*r), breite)
        elif name == "circle":
            pygame.draw.circle(s, op[1], op[2], op[3], op[4])
        elif name == "ellipse":
            pygame.draw.ellipse(s, op[1], pygame.Rect(*op[2]), op[3])
        elif name == "line":
            pygame.draw.line(s, op[1], op[2], op[3], op[4])
        elif name == "lines":
            pygame.draw.lines(s, op[1], op[2], op[3], op[4])
        elif name == "polygon":
            pygame.draw.polygon(s, op[1], op[2], op[3])
        elif name == "aaline":
            pygame.draw.aaline(s, op[1], op[2], op[3], op[4])
        elif name == "aalines":
            pygame.draw.aalines(s, op[1], op[2], op[3])
        elif name == "aacircle":
            pygame.draw.aacircle(s, op[1], op[2], op[3], op[4])
        elif name == "flood_fill":
            pygame.draw.flood_fill(s, op[1], op[2])
        elif name == "arc":
            pygame.draw.arc(s, op[1], pygame.Rect(*op[2]), op[3], op[4], op[5])
        elif name == "scale":
            s = pygame.transform.scale(s, op[1])
        elif name == "smoothscale":
            s = pygame.transform.smoothscale(s, op[1])
        elif name == "flip":
            s = pygame.transform.flip(s, op[1], op[2])
        elif name == "rotate":
            s = pygame.transform.rotate(s, op[1])
        elif name == "scale_by":
            s = pygame.transform.scale_by(s, (op[1], op[2]))
        elif name == "smoothscale_by":
            s = pygame.transform.smoothscale_by(s, (op[1], op[2]))
        elif name == "scale2x":
            s = pygame.transform.scale2x(s)
        elif name == "invert":
            s = pygame.transform.invert(s)
        elif name == "laplacian":
            s = pygame.transform.laplacian(s)
        elif name == "box_blur":
            s = pygame.transform.box_blur(s, op[1], op[2])
        elif name == "gaussian_blur":
            s = pygame.transform.gaussian_blur(s, op[1], op[2])
        elif name == "hsl":
            s = pygame.transform.hsl(s, op[1], op[2], op[3])
        elif name == "chop":
            s = pygame.transform.chop(s, pygame.Rect(op[1], op[2], op[3], op[4]))
        elif name == "pixelate":
            s = pygame.transform.pixelate(s, op[1])
        elif name == "solid_overlay":
            s = pygame.transform.solid_overlay(s, op[1], keep_alpha=op[2])
        elif name == "rotozoom":
            s = pygame.transform.rotozoom(s, op[1], op[2])
        elif name == "grayscale":
            s = pygame.transform.grayscale(s)
        else:
            raise SystemExit("unbekannte Operation: %s" % name)
    return s


# ---------------------------------------------------------------------------
#  Dieselben Faelle nach Rust uebersetzen
# ---------------------------------------------------------------------------

# In einen Unterordner: Cargo uebersetzt jede .rs direkt in tests/ als
# eigenen Integrationstest - die Falldatei ist aber nur ein Einschub.
RUST_ZIEL = os.path.join(ROOT, "rustgame", "crates", "rustgame-core",
                         "tests", "gen", "faelle.rs")

# Welche Operation steckt in welcher Rust-Datei? Fehlt die Umsetzung noch,
# wird der Fall im Rust-Test uebersprungen statt rot zu werden - so waechst
# die Abdeckung sichtbar, ohne dass die Testsuite blockiert.
BRAUCHT = {
    "rect": ("draw::rect", "crates/rustgame-core/src/draw/rect.rs"),
    "circle": ("draw::circle", "crates/rustgame-core/src/draw/circle.rs"),
    "ellipse": ("draw::ellipse", "crates/rustgame-core/src/draw/ellipse.rs"),
    "line": ("draw::line", "crates/rustgame-core/src/draw/line.rs"),
    "lines": ("draw::lines", "crates/rustgame-core/src/draw/line.rs"),
    "polygon": ("draw::polygon", "crates/rustgame-core/src/draw/polygon.rs"),
    "aaline": ("draw::aaline", "crates/rustgame-core/src/draw/aaline.rs"),
    "aalines": ("draw::aalines", "crates/rustgame-core/src/draw/aaline.rs"),
    "aacircle": ("draw::aacircle", "crates/rustgame-core/src/draw/aacircle.rs"),
    "flood_fill": ("draw::flood_fill", "crates/rustgame-core/src/draw/flood_fill.rs"),
    "arc": ("draw::arc", "crates/rustgame-core/src/draw/arc.rs"),
    "scale": ("transform::scale", "crates/rustgame-core/src/transform.rs"),
    "smoothscale": ("transform::smoothscale", "crates/rustgame-core/src/transform.rs"),
    "flip": ("transform::flip", "crates/rustgame-core/src/transform.rs"),
    "rotate": ("transform::rotate", "crates/rustgame-core/src/transform.rs"),
    "scale_by": ("transform::scale_by", "crates/rustgame-core/src/transform.rs"),
    "smoothscale_by": ("transform::smoothscale_by", "crates/rustgame-core/src/transform.rs"),
    "scale2x": ("transform::scale2x", "crates/rustgame-core/src/transform.rs"),
    "grayscale": ("transform::grayscale", "crates/rustgame-core/src/transform.rs"),
    "invert": ("transform::invert", "crates/rustgame-core/src/transform.rs"),
    "laplacian": ("transform::laplacian", "crates/rustgame-core/src/transform.rs"),
    "box_blur": ("transform::box_blur", "crates/rustgame-core/src/transform.rs"),
    "gaussian_blur": ("transform::gaussian_blur", "crates/rustgame-core/src/transform.rs"),
    "hsl": ("transform::hsl", "crates/rustgame-core/src/transform.rs"),
    "chop": ("transform::chop", "crates/rustgame-core/src/transform.rs"),
    "pixelate": ("transform::pixelate", "crates/rustgame-core/src/transform.rs"),
    "solid_overlay": ("transform::solid_overlay", "crates/rustgame-core/src/transform.rs"),
    "rotozoom": ("transform::rotozoom", "crates/rustgame-core/src/transform.rs"),
}

# Solange dieser Satz im Kopf einer Datei steht, gilt sie als Platzhalter.
# So muss niemand von Hand pflegen, was schon fertig ist - die Datei sagt es
# selbst, und der naechste Lauf von gen_golden.py zieht automatisch nach.
PLATZHALTER_MARKE = "**Noch nicht umgesetzt.**"


def ist_fertig(op_name):
    """Gibt es die Rust-Umsetzung dieser Operation schon?"""
    eintrag = BRAUCHT.get(op_name)
    if eintrag is None:
        return True
    pfad = os.path.join(ROOT, "rustgame", eintrag[1])
    try:
        with open(pfad, "r", encoding="utf-8") as f:
            return PLATZHALTER_MARKE not in f.read(4000)
    except OSError:
        return False


def rs_farbe(c):
    if len(c) == 3:
        return "(%du8, %du8, %du8)" % c
    return "(%du8, %du8, %du8, %du8)" % tuple(c)


def rs_flag(name):
    """Flag-Name -> Rust-Pfad."""
    if not name:
        return "0"
    return "rustgame_core::blend::flags::%s" % name


def rust_ops(fall):
    """Uebersetzt die Operationen eines Falls in Rust-Anweisungen."""
    zeilen = []
    for op in fall["ops"]:
        name = op[0]
        if name == "fill":
            zeilen.append("        s.fill(%s);" % rs_farbe(op[1]))
        elif name == "fill_rect":
            zeilen.append("        s.fill_rect(%s, Rect::new(%d, %d, %d, %d));"
                          % (rs_farbe(op[1]), *op[2]))
        elif name == "set_clip":
            zeilen.append("        s.set_clip(Some(Rect::new(%d, %d, %d, %d)));" % tuple(op[1]))
        elif name == "muster":
            # Nicht `s = muster(w, h)`! Das Python-Gegenstueck legt das
            # Muster in der angegebenen Groesse an und skaliert es *dann*
            # auf die Flaeche. Wer das weglaesst, bekommt eine Flaeche
            # falscher Groesse - und bei den Transform-Faellen ein voellig
            # anderes Bild.
            zeilen.append("        s = muster_auf(s, %d, %d);" % (op[1], op[2]))
        elif name == "blit_gefuellt":
            zeilen.append("        let q = gefuellt(s.get_size(), %s);" % rs_farbe(op[1]))
            zeilen.append("        s.blit_ex(&q, (%d, %d), None, %s);"
                          % (op[2][0], op[2][1], rs_flag(op[3])))
        elif name == "blit_gefuellt_alpha":
            zeilen.append("        let mut q = gefuellt(s.get_size(), %s);" % rs_farbe(op[1]))
            zeilen.append("        q.set_alpha(Some(%d));" % op[3])
            zeilen.append("        s.blit(&q, (%d, %d));" % tuple(op[2]))
        elif name == "blit_muster":
            (qw, qh), srcalpha, alpha, ck, ziel, bereich, flag = op[1:]
            zeilen.append("        let mut q = Surface::with_flags((%d, %d), %s);"
                          % (qw, qh, "SRCALPHA" if srcalpha else "0"))
            zeilen.append("        q = muster_auf(q, %d, %d);" % (qw, qh))
            if alpha is not None:
                zeilen.append("        q.set_alpha(Some(%d));" % alpha)
            if ck is not None:
                zeilen.append("        q.set_colorkey(Some(%s));" % rs_farbe(ck))
            zeilen.append("        s.blit_ex(&q, (%d, %d), %s, %s);"
                          % (ziel[0], ziel[1],
                             ("Some(Rect::new(%d, %d, %d, %d))" % tuple(bereich))
                             if bereich else "None",
                             rs_flag(flag)))
        elif name == "subsurface_fill":
            zeilen.append("        {")
            zeilen.append("            let mut teil = s.subsurface(Rect::new(%d, %d, %d, %d));"
                          % tuple(op[1]))
            zeilen.append("            teil.fill(%s);" % rs_farbe(op[2]))
            zeilen.append("        }")
        elif name == "scroll":
            zeilen.append("        s.scroll(%d, %d);" % (op[1], op[2]))
        elif name in ("convert", "convert_alpha"):
            zeilen.append("        s = s.%s();" % name)
        elif name in BRAUCHT:
            if not ist_fertig(name):
                return None, BRAUCHT[name][0]
            zeilen.extend(rust_zeichnen(name, op))
        else:
            raise SystemExit("keine Rust-Uebersetzung fuer: %s" % name)
    return zeilen, None


def rust_zeichnen(name, op):
    """Uebersetzt eine Zeichen- oder Transform-Operation nach Rust.

    Voll qualifizierte Pfade, weil `golden.rs` weder `draw` noch
    `transform` importiert und `Rect` dort schon belegt ist.
    """
    d = "rustgame_core::draw"
    tf = "rustgame_core::transform"
    if name == "rect":
        farbe, r, breite, radius = op[1], op[2], op[3], op[4]
        if radius >= 0:
            return ["        %s::rect_radius(&mut s, %s, Rect::new(%d, %d, %d, %d), %d, %d);"
                    % (d, rs_farbe(farbe), r[0], r[1], r[2], r[3], breite, radius)]
        return ["        %s::rect(&mut s, %s, Rect::new(%d, %d, %d, %d), %d);"
                % (d, rs_farbe(farbe), r[0], r[1], r[2], r[3], breite)]
    if name == "circle":
        return ["        %s::circle(&mut s, %s, (%d, %d), %d, %d);"
                % (d, rs_farbe(op[1]), op[2][0], op[2][1], op[3], op[4])]
    if name == "ellipse":
        r = op[2]
        return ["        %s::ellipse(&mut s, %s, Rect::new(%d, %d, %d, %d), %d);"
                % (d, rs_farbe(op[1]), r[0], r[1], r[2], r[3], op[3])]
    if name == "line":
        return ["        %s::line(&mut s, %s, (%d, %d), (%d, %d), %d);"
                % (d, rs_farbe(op[1]), op[2][0], op[2][1], op[3][0], op[3][1], op[4])]
    if name == "lines":
        punkte = ", ".join("(%d, %d)" % (p[0], p[1]) for p in op[3])
        return ["        %s::lines(&mut s, %s, %s, &[%s], %d);"
                % (d, rs_farbe(op[1]), "true" if op[2] else "false", punkte, op[4])]
    if name == "aaline":
        return ["        %s::aaline(&mut s, %s, (%r, %r), (%r, %r), %d);"
                % (d, rs_farbe(op[1]), float(op[2][0]), float(op[2][1]),
                   float(op[3][0]), float(op[3][1]), op[4])]
    if name == "aalines":
        punkte = ", ".join("(%r, %r)" % (float(p[0]), float(p[1])) for p in op[3])
        return ["        %s::aalines(&mut s, %s, %s, &[%s]);"
                % (d, rs_farbe(op[1]), "true" if op[2] else "false", punkte)]
    if name == "aacircle":
        return ["        %s::aacircle(&mut s, %s, (%d, %d), %d, %d);"
                % (d, rs_farbe(op[1]), op[2][0], op[2][1], op[3], op[4])]
    if name == "flood_fill":
        return ["        %s::flood_fill(&mut s, %s, (%d, %d));"
                % (d, rs_farbe(op[1]), op[2][0], op[2][1])]
    if name == "polygon":
        punkte = ", ".join("(%d, %d)" % (p[0], p[1]) for p in op[2])
        return ["        %s::polygon(&mut s, %s, &[%s], %d);"
                % (d, rs_farbe(op[1]), punkte, op[3])]
    if name == "arc":
        r = op[2]
        return ["        %s::arc(&mut s, %s, Rect::new(%d, %d, %d, %d), %r, %r, %d);"
                % (d, rs_farbe(op[1]), r[0], r[1], r[2], r[3],
                   float(op[3]), float(op[4]), op[5])]
    if name in ("scale", "smoothscale"):
        return ["        s = %s::%s(&s, (%d, %d));" % (tf, name, op[1][0], op[1][1])]
    if name == "flip":
        return ["        s = %s::flip(&s, %s, %s);"
                % (tf, "true" if op[1] else "false", "true" if op[2] else "false")]
    if name == "rotate":
        return ["        s = %s::rotate(&s, %r);" % (tf, float(op[1]))]
    if name in ("scale_by", "smoothscale_by"):
        return ["        s = %s::%s(&s, %r, %r);"
                % (tf, name, float(op[1]), float(op[2]))]
    if name in ("scale2x", "grayscale", "invert", "laplacian"):
        return ["        s = %s::%s(&s);" % (tf, name)]
    if name in ("box_blur", "gaussian_blur"):
        return ["        s = %s::%s(&s, %d, %s).unwrap();"
                % (tf, name, op[1], "true" if op[2] else "false")]
    if name == "hsl":
        return ["        s = %s::hsl(&s, %r, %r, %r).unwrap();"
                % (tf, float(op[1]), float(op[2]), float(op[3]))]
    if name == "chop":
        return ["        s = %s::chop(&s, Rect::new(%d, %d, %d, %d));"
                % (tf, op[1], op[2], op[3], op[4])]
    if name == "pixelate":
        return ["        s = %s::pixelate(&s, %d).unwrap();" % (tf, op[1])]
    if name == "solid_overlay":
        return ["        s = %s::solid_overlay(&s, %s, %s);"
                % (tf, rs_farbe(op[1]), "true" if op[2] else "false")]
    if name == "rotozoom":
        return ["        s = %s::rotozoom(&s, %r, %r);"
                % (tf, float(op[1]), float(op[2]))]
    raise SystemExit("keine Rust-Uebersetzung fuer: %s" % name)


def schreibe_rust():
    zeilen = [
        "// faelle.rs",
        "// =========",
        "// **Erzeugte Datei - nicht von Hand aendern.**",
        "//",
        "// Quelle: `rustgame/tools/gen_golden.py` uebersetzt `tools/cases.py`.",
        "// Dieselbe Falltabelle erzeugt die Referenzbilder mit echtem pygame",
        "// und wird hier nachgespielt - so koennen beide Seiten gar nicht",
        "// auseinanderlaufen.",
        "",
        "/// Alle Faelle: `(Name, Toleranz, Bauanweisung)`.",
        "#[allow(clippy::type_complexity)]",
        # `approx_constant` ist bei clippy ein Fehler, kein Hinweis - und
        # der Winkel 3.14159 aus `cases.py` loest ihn aus. Ihn durch
        # `std::f32::consts::PI` zu ersetzen waere falsch: **genau diese
        # Zahl** hat echtes pygame bekommen, als das Referenzbild entstand.
        # Wer sie "verbessert", vergleicht anschliessend zwei verschiedene
        # Eingaben. Also bleibt die Zahl, und die Regel geht aus.
        "#[allow(clippy::approx_constant)]",
        "fn faelle() -> Vec<(&'static str, u8, Box<dyn Fn() -> Ergebnis>)> {",
        "    vec![",
    ]
    machbar = 0
    for fall in FAELLE:
        ops, fehlt = rust_ops(fall)
        flags = "SRCALPHA" if fall["flags"] == "SRCALPHA" else "0"
        zeilen.append('        ("%s", %d, Box::new(|| {' % (fall["name"], fall["tol"]))
        if fehlt:
            zeilen.append('            Ergebnis::Fehlt("%s")' % fehlt)
        else:
            machbar += 1
            zeilen.append("            #[allow(unused_mut)]")
            zeilen.append("            let mut s = Surface::with_flags((%d, %d), %s);"
                          % (fall["size"][0], fall["size"][1], flags))
            zeilen.extend("    " + z.strip() and "            " + z.strip() for z in ops)
            zeilen.append("            Ergebnis::Fertig(s)")
        zeilen.append("        })),")
    zeilen += [
        "    ]",
        "}",
        "",
    ]
    os.makedirs(os.path.dirname(RUST_ZIEL), exist_ok=True)
    with open(RUST_ZIEL, "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(zeilen))
    return machbar


def main():
    pygame.init()
    os.makedirs(ZIEL, exist_ok=True)

    eintraege = []
    fehler = 0
    for fall in FAELLE:
        try:
            s = fuehre_aus(fall)
        except Exception as exc:                      # pragma: no cover
            print("  FEHLER in %-28s %s" % (fall["name"], exc))
            fehler += 1
            continue
        pfad = os.path.join(ZIEL, fall["name"] + ".png")
        pygame.image.save(s, pfad)
        eintraege.append({
            "name": fall["name"],
            "gruppe": fall["gruppe"],
            "size": list(s.get_size()),
            "tol": fall["tol"],
        })

    manifest = {
        "pygame": pygame.version.ver,
        "sdl": ".".join(map(str, pygame.get_sdl_version())),
        "plattform": platform.platform(),
        "faelle": eintraege,
    }
    with open(os.path.join(ZIEL, "manifest.json"), "w", encoding="utf-8") as f:
        json.dump(manifest, f, indent=2, ensure_ascii=False)
        f.write("\n")

    machbar = schreibe_rust()

    print("pygame-ce %s (SDL %s)" % (manifest["pygame"], manifest["sdl"]))
    print("%d Referenzbilder nach %s" % (len(eintraege), os.path.relpath(ZIEL, ROOT)))
    for g in gruppen():
        n = sum(1 for e in eintraege if e["gruppe"] == g)
        print("   %-14s %2d" % (g, n))
    print()
    print("Rust-Faelle: %s" % os.path.relpath(RUST_ZIEL, ROOT))
    print("   %d von %d sind mit dem heutigen Stand nachspielbar"
          % (machbar, len(FAELLE)))
    if fehler:
        print("%d Faelle konnten nicht erzeugt werden" % fehler)
    return 1 if fehler else 0


if __name__ == "__main__":
    sys.exit(main())
