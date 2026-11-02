# -*- coding: utf-8 -*-
"""
rustgame/tools/fuzz_ops.py
==========================
**Differenzielles Fuzzing** gegen echtes pygame-ce.

Die goldenen Referenzbilder (``gen_golden.py``) pruefen je *eine* Sache:
ein Rechteck, ein Kreis, eine Mischart. Fehler entstehen aber selten in
einer einzelnen Funktion - sie entstehen im **Zusammenspiel**:

    ein Blit mit Schluesselfarbe auf eine beschnittene Teilflaeche,
    deren Elternteil ein Flaechen-Alpha traegt

So etwas schreibt niemand von Hand als Testfall auf. Also wuerfeln wir es:
aus einem Startwert entsteht eine Folge von Operationen, echtes pygame
fuehrt sie aus, und **nur der Hash des Ergebnisses** wird abgelegt. Rust
spielt dieselbe Folge nach und muss denselben Hash liefern.

Das ist der Grund, warum hier kein einziges Bild im Verzeichnis landet:
500 Faelle waeren 500 PNGs. Ein Hash ist 16 Zeichen.

Benutzt wird **dasselbe Vokabular** wie in ``cases.py`` - der Fuzzer ruft
``gen_golden.fuehre_aus`` und ``gen_golden.rust_ops`` auf. Es gibt also
keinen zweiten Interpreter, der auseinanderlaufen koennte, und jede spaeter
ergaenzte Operation steht dem Fuzzer sofort zur Verfuegung.

Aufruf::

    .venv/Scripts/python.exe rustgame/tools/fuzz_ops.py            # 600 Faelle
    .venv/Scripts/python.exe rustgame/tools/fuzz_ops.py --anzahl 2000
    .venv/Scripts/python.exe rustgame/tools/fuzz_ops.py --zeige 137

``--zeige`` schreibt das *erwartete* Bild eines einzelnen Startwerts nach
``rustgame.output/fuzz-diff/`` und druckt die Operationsfolge aus - der Weg,
wenn der Rust-Test einen Fall meldet.
"""

import argparse
import os
import random
import sys

HIER = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HIER)

os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
os.environ.setdefault("PYGAME_HIDE_SUPPORT_PROMPT", "1")

import pygame  # noqa: E402

import gen_golden as gg  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(HIER))
RUST_ZIEL = os.path.join(ROOT, "rustgame", "crates", "rustgame-core",
                         "tests", "gen", "fuzz.rs")


# ---------------------------------------------------------------------------
#  Der Hash
# ---------------------------------------------------------------------------
#
# FNV-1a, 64 Bit. Bewusst **nicht** SHA-256: das muesste auf der Rust-Seite
# nachgebaut oder als Fremdkiste eingebunden werden, und fuer "sind diese
# zwei Bilder gleich?" ist es Kanonen auf Spatzen. Bei 64 Bit und ein paar
# tausend Faellen ist ein zufaelliges Zusammentreffen praktisch
# ausgeschlossen - und ein *gezielter* Angreifer existiert hier nicht.

FNV_BASIS = 0xCBF29CE484222325
FNV_PRIM = 0x100000001B3


def fnv1a(daten):
    h = FNV_BASIS
    for b in daten:
        h = ((h ^ b) * FNV_PRIM) & 0xFFFFFFFFFFFFFFFF
    return h


# ---------------------------------------------------------------------------
#  Die Wuerfel
# ---------------------------------------------------------------------------

FLAGNAMEN = [
    0, "BLEND_RGB_ADD", "BLEND_RGB_SUB", "BLEND_RGB_MULT",
    "BLEND_RGB_MIN", "BLEND_RGB_MAX", "BLEND_RGBA_ADD", "BLEND_RGBA_SUB",
    "BLEND_RGBA_MULT", "BLEND_RGBA_MIN", "BLEND_RGBA_MAX",
]


def farbe(r, undurchsichtig=False):
    """Eine Zufallsfarbe. Alpha oft, aber nicht immer, voll."""
    a = 255 if undurchsichtig or r.random() < 0.4 else r.randrange(256)
    return (r.randrange(256), r.randrange(256), r.randrange(256), a)


def rechteck_in(r, w, h):
    """Ein Rechteck, das **sicher** ganz in der Flaeche liegt.

    Fuer `subsurface` ist das Pflicht - pygame wirft sonst eine Ausnahme,
    und ein Fuzzer, der seine eigenen Faelle sprengt, prueft nichts.
    """
    x = r.randrange(w)
    y = r.randrange(h)
    return (x, y, r.randrange(1, w - x + 1), r.randrange(1, h - y + 1))


def rechteck_frei(r, w, h):
    """Ein Rechteck, das auch ueber den Rand hinausragen darf."""
    x = r.randrange(-w // 2, w + 2)
    y = r.randrange(-h // 2, h + 2)
    return (x, y, r.randrange(0, w + 4), r.randrange(0, h + 4))


def punkt_frei(r, w, h):
    return (r.randrange(-4, w + 4), r.randrange(-4, h + 4))


def erzeuge(seed):
    """Baut aus einem Startwert eine Flaechengroesse und eine Op-Folge."""
    r = random.Random(seed)
    w = r.randrange(6, 33)
    h = r.randrange(6, 33)
    srcalpha = r.random() < 0.5
    ops = [("fill", farbe(r, undurchsichtig=not srcalpha))]

    for _ in range(r.randrange(2, 9)):
        wahl = r.random()

        if wahl < 0.10:
            ops.append(("fill_rect", farbe(r), rechteck_frei(r, w, h)))
        elif wahl < 0.18:
            # Ein Clip-Rechteck bleibt fuer alles Folgende stehen - genau
            # das macht es interessant.
            ops.append(("set_clip", rechteck_in(r, w, h)))
        elif wahl < 0.28:
            ops.append(("subsurface_fill", rechteck_in(r, w, h), farbe(r)))
        elif wahl < 0.34:
            ops.append(("scroll", r.randrange(-w, w + 1), r.randrange(-h, h + 1)))
        elif wahl < 0.52:
            # Der interessanteste Fall: Blit mit allem, was schiefgehen kann.
            qw, qh = r.randrange(2, w + 3), r.randrange(2, h + 3)
            q_alpha = r.random() < 0.5
            ops.append((
                "blit_muster",
                (qw, qh),
                q_alpha,
                r.randrange(256) if r.random() < 0.4 else None,
                farbe(r, undurchsichtig=True)[:3] if r.random() < 0.3 else None,
                punkt_frei(r, w, h),
                rechteck_frei(r, qw, qh) if r.random() < 0.3 else None,
                FLAGNAMEN[r.randrange(len(FLAGNAMEN))],
            ))
        elif wahl < 0.66:
            ops.append(("rect", farbe(r), rechteck_frei(r, w, h),
                        r.randrange(0, 5),
                        r.randrange(-1, 7)))
        elif wahl < 0.78:
            ops.append(("circle", farbe(r), punkt_frei(r, w, h),
                        r.randrange(0, max(w, h)), r.randrange(0, 5)))
        elif wahl < 0.86:
            ops.append(("ellipse", farbe(r), rechteck_frei(r, w, h),
                        r.randrange(0, 5)))
        elif wahl < 0.93:
            ops.append(("line", farbe(r), punkt_frei(r, w, h),
                        punkt_frei(r, w, h), r.randrange(1, 6)))
        else:
            n = r.randrange(3, 7)
            ops.append(("polygon", farbe(r),
                        [punkt_frei(r, w, h) for _ in range(n)],
                        r.randrange(0, 4)))

    # Formatwechsel und Verwandeln **zum Schluss**.
    #
    # Warum nicht mittendrin? Beides aendert Groesse oder Flags der
    # Flaeche, und die folgenden Operationen wuerfeln ihre Koordinaten
    # noch nach der alten Groesse. `subsurface` verlangt aber ein
    # Rechteck, das wirklich hineinpasst - pygame wirft sonst, und ein
    # Fuzzer, der seine eigenen Faelle sprengt, prueft nichts. Am Ende
    # der Folge ist beides gefahrlos und trifft trotzdem genau die
    # Auswahlkette des Blits, um die es geht.
    if r.random() < 0.25:
        ops.append((r.choice(["convert", "convert_alpha"]),))
    if r.random() < 0.30:
        w2 = r.randrange(2, w + 6)
        h2 = r.randrange(2, h + 6)
        ops.append(r.choice([
            ("scale", (w2, h2)),
            ("flip", r.random() < 0.5, r.random() < 0.5),
            ("rotate", float(r.randrange(-400, 401))),
            ("scale2x",),
            ("grayscale",),
        ]))

    return {
        "name": "fuzz_%d" % seed,
        "size": (w, h),
        "flags": "SRCALPHA" if srcalpha else 0,
        # `smoothscale` ist nicht einmal mit sich selbst pixelgleich
        # (pygames GENERIC gegen SSE2: bis zu 2 Stufen). Da der Fuzzer
        # ueber Hashes vergleicht und nicht ueber Pixel, waere jede
        # Toleranz wirkungslos - solche Faelle bleiben deshalb draussen.
        "tol": 0,
        "ops": ops,
        "gruppe": "fuzz",
    }


# ---------------------------------------------------------------------------
#  Ausfuehren und uebersetzen
# ---------------------------------------------------------------------------

def hash_von(flaeche):
    return fnv1a(pygame.image.tobytes(flaeche, "RGBA"))


def bekannt_abweichend(fall):
    """Trifft dieser Fall die eine Stelle, die noch offen ist?

    `scale2x` vergleicht **ganze 32-Bit-Woerter**, also einschliesslich
    des vierten Bytes. Auf einer Flaeche ohne Alphakanal ist dieses Byte
    normalerweise null, kann aber von einer `BLEND_RGBA_*`-Mischart
    beschrieben worden sein - sichtbar ist es nie, `get_at` und `tobytes`
    maskieren es weg.

    Was genau pygame dort ablegt, liess sich **nicht** eindeutig messen:
    zwei Versuchsreihen widersprechen einander (einmal liest pygame das
    Quell-Alpha einer Flaeche ohne Alphakanal als 255, einmal roh als 0).
    Solange das nicht geklaert ist, waere jede Umsetzung geraten - und
    Raten ist hier die eine Sache, die nicht passieren darf.

    Deshalb werden genau diese Faelle gezaehlt und benannt, statt den
    Test dauerhaft rot zu lassen (was jede **neue** Abweichung verdecken
    wuerde). Wer die Frage klaert, nimmt diese Funktion wieder heraus.
    """
    namen = [op[0] for op in fall["ops"]]
    if "scale2x" not in namen:
        return False
    return any(
        op[0] == "blit_muster" and str(op[-1]).startswith("BLEND_RGBA")
        for op in fall["ops"]
    )


def schreibe_rust(faelle):
    os.makedirs(os.path.dirname(RUST_ZIEL), exist_ok=True)
    aus = ['''// ERZEUGT von rustgame/tools/fuzz_ops.py - nicht von Hand aendern.
//
// Jeder Fall ist eine gewuerfelte Folge von Operationen, die **echtes
// pygame-ce** bereits ausgefuehrt hat; hinterlegt ist nur der FNV-1a-Hash
// des Ergebnisses. Weicht der Hash ab, stimmt irgendwo ein Pixel nicht -
// welcher, verraet:
//
//     .venv/Scripts/python.exe rustgame/tools/fuzz_ops.py --zeige <nummer>
//
// Neu wuerfeln (mehr Faelle, andere Folgen):
//     .venv/Scripts/python.exe rustgame/tools/fuzz_ops.py --anzahl 2000

#[allow(clippy::type_complexity)]
fn fuzz_faelle() -> Vec<(u64, bool, fn() -> FuzzErgebnis)> {
    // `bool` = bekannte, benannte Abweichung (siehe `bekannt_abweichend`
    // in fuzz_ops.py). Solche Faelle werden gezaehlt, nicht gewertet.
    let mut v: Vec<(u64, bool, fn() -> FuzzErgebnis)> = Vec::new();
''']
    machbar = 0
    for f, seed, h in faelle:
        zeilen, fehlt = gg.rust_ops(f)
        if zeilen is None:
            aus.append("    // %d uebersprungen: %s braucht es noch nicht\n"
                       % (seed, fehlt))
            continue
        machbar += 1
        flagwert = "SRCALPHA" if f["flags"] == "SRCALPHA" else "0"
        aus.append("    v.push((%d, %s, || {\n"
                   % (seed, "true" if bekannt_abweichend(f) else "false"))
        aus.append("        #[allow(unused_mut)]\n")
        aus.append("        let mut s = Surface::with_flags((%d, %d), %s);\n"
                   % (f["size"][0], f["size"][1], flagwert))
        for z in zeilen:
            aus.append("    " + z + "\n")
        aus.append("        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), "
                   "erwartet: 0x%016X, flaeche: s }\n" % h)
        aus.append("    }));\n")
    aus.append("    v\n}\n")
    with open(RUST_ZIEL, "w", encoding="utf-8") as datei:
        datei.write("".join(aus))
    return machbar


def zeige(seed):
    """Schreibt das erwartete Bild eines Falls und druckt seine Folge."""
    f = erzeuge(seed)
    s = gg.fuehre_aus(f)
    ziel = os.path.join(ROOT, "rustgame.output", "fuzz-diff")
    os.makedirs(ziel, exist_ok=True)
    pfad = os.path.join(ziel, "fuzz_%d_erwartet.png" % seed)
    pygame.image.save(s, pfad)
    print("Fall %d: %dx%d, flags=%s" % (seed, f["size"][0], f["size"][1], f["flags"]))
    for op in f["ops"]:
        print("   %s" % (op,))
    print("Hash    0x%016X" % hash_von(s))
    print("Bild    %s" % os.path.relpath(pfad, ROOT))


def main():
    p = argparse.ArgumentParser(description=__doc__)
    # Die eingecheckte Groesse ist bewusst klein.
    #
    # Jeder Fall wird als eigener Rust-Abschluss erzeugt, und die Kiste
    # uebersetzt mit `opt-level = 3` (der Blit-Kern braucht das). Bei 5000
    # Faellen dauert allein das Uebersetzen des Tests eine halbe Stunde -
    # untragbar fuer etwas, das bei jedem `cargo test` mitlaeuft.
    #
    # 600 Faelle uebersetzen in Sekunden und decken die Mischarten,
    # Teilflaechen und Formatwechsel bereits dicht ab. Fuer einen tiefen
    # Durchlauf von Hand: `--anzahl 5000`.
    p.add_argument("--anzahl", type=int, default=600)
    p.add_argument("--zeige", type=int, default=None)
    p.add_argument("--zerlege", type=int, default=None,
                   help="erzeugt statt aller Faelle nur die Praefixe EINES "
                        "Falls - dann sagt der Rust-Test, welche Operation "
                        "kippt, statt nur welcher Fall")
    args = p.parse_args()

    pygame.init()
    pygame.display.set_mode((1, 1))

    if args.zeige is not None:
        zeige(args.zeige)
        return 0

    if args.zerlege is not None:
        voll = erzeuge(args.zerlege)
        faelle = []
        for i in range(1, len(voll["ops"]) + 1):
            teil = dict(voll, ops=voll["ops"][:i])
            faelle.append((teil, i, hash_von(gg.fuehre_aus(teil))))
        schreibe_rust(faelle)
        print("Fall %d in %d Praefixe zerlegt -> %s"
              % (args.zerlege, len(faelle), os.path.relpath(RUST_ZIEL, ROOT)))
        print("Jetzt:  cargo test --test fuzz")
        print("Die kleinste Nummer, die faellt, ist die schuldige Operation:")
        for i, op in enumerate(voll["ops"], 1):
            print("   %2d  %s" % (i, op))
        return 0

    faelle = []
    kaputt = []
    for seed in range(args.anzahl):
        f = erzeuge(seed)
        try:
            s = gg.fuehre_aus(f)
        except Exception as e:                      # noqa: BLE001
            kaputt.append((seed, e))
            continue
        faelle.append((f, seed, hash_von(s)))

    machbar = schreibe_rust(faelle)
    print("%d Faelle gewuerfelt und mit echtem pygame ausgefuehrt" % len(faelle))
    print("   %d davon sind mit dem heutigen Stand nachspielbar" % machbar)
    print("   -> %s" % os.path.relpath(RUST_ZIEL, ROOT))
    if kaputt:
        print("%d Faelle hat pygame selbst abgelehnt:" % len(kaputt))
        for seed, e in kaputt[:5]:
            print("   %d: %s" % (seed, e))
    return 0


if __name__ == "__main__":
    sys.exit(main())
