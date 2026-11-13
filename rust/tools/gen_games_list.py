"""Schreibt pgz-games/src/lib.rs: alle 46 Spiele in ALL_GAMES-Reihenfolge,
eingetragen ist, wessen Modul schon existiert."""
import os
import sys

wurzel = sys.argv[1]
sys.path.insert(0, wurzel)
os.environ.setdefault("PYGAME_HIDE_SUPPORT_PROMPT", "1")
src = os.path.join(wurzel, "rust", "crates", "pgz-games", "src")

# Python-Modulname -> Rust-Modulname (gleich, ausser es kollidiert).
from games import ALL_GAMES

module = []
for cls in ALL_GAMES:
    mod = cls.__module__.split(".")[-1]
    module.append((mod, cls.highscore_key))

da = {f[:-3] for f in os.listdir(src) if f.endswith(".rs")}
da |= {d for d in os.listdir(src) if os.path.isfile(os.path.join(src, d, "mod.rs"))}
zeilen = [
    "//! pgz-games",
    "//! =========",
    "//! Die Spiele - je Datei eine Python-Vorlage aus `games/`.",
    "//!",
    "//! `all_games()` haelt die Reihenfolge von `games/__init__.py` (`ALL_GAMES`).",
    "//! Diese Datei wird geschrieben, nicht von Hand gepflegt.",
    "",
    "// approx_constant: Zahlen wie 6.28 oder 0.7071 stehen genau so in der",
    "// Python-Vorlage - eine genauere Konstante gaebe andere Pixel.",
    "#![allow(clippy::new_ret_no_self, clippy::too_many_arguments, clippy::approx_constant)]",
    "",
    "use pgz_engine::GameInfo;",
    "",
]
extra = sorted(m for m in da if m not in {x for x, _ in module} and m not in ("lib",))
for m in sorted({x for x, _ in module if x in da} | set(extra)):
    zeilen.append("pub mod %s;" % m)
zeilen += ["", "/// Alle uebertragenen Spiele in der Reihenfolge von `ALL_GAMES`.",
           "pub fn all_games() -> Vec<GameInfo> {", "    vec!["]
fertig = 0
for mod, key in module:
    if mod in da:
        zeilen.append("        %s::INFO," % mod)
        fertig += 1
    else:
        zeilen.append("        // %s (%s) - noch nicht uebertragen" % (mod, key))
zeilen += ["    ]", "}", ""]
with open(os.path.join(src, "lib.rs"), "w", encoding="utf-8", newline="\n") as f:
    f.write("\n".join(zeilen))
print("%d von %d Spielen eingetragen" % (fertig, len(module)))
