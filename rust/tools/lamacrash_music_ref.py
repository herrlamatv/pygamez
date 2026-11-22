# -*- coding: utf-8 -*-
"""
lamacrash_music_ref.py
======================
Schreibt die Pruefwerte der Lama-Crash-Musik (``games/lamacrash_music.py``)
nach ``rust/crates/pgz-games/src/data/lamacrash_music_ref.txt``. Der
Rust-Test ``lamacrash_music_wie_python`` (``lamacrash_music.rs``) rechnet
alles nach:

    .venv/Scripts/python.exe rust/tools/lamacrash_music_ref.py           # neu schreiben
    .venv/Scripts/python.exe rust/tools/lamacrash_music_ref.py --check   # nur pruefen (Exit 1 = veraltet)
    cd rust && cargo test -p pgz-games --offline --lib -- lamacrash_music

Zeilen: je Stufe der SHA-256 der Noten aus ``compose`` (Gleitkommazahlen als
IEEE-Bits, also bitgenau), dann der SHA-256 der Quietsch-Tabelle
(``squeak``) und Klartext fuer ``stage_of``/``bpm``/``beat_seconds``.
``tests/audit_chiplogic.py`` prueft, dass die Datei aktuell ist.
"""

import hashlib
import os
import struct
import sys

os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
os.environ.setdefault("SDL_AUDIODRIVER", "dummy")
os.environ.setdefault("PYGAME_HIDE_SUPPORT_PROMPT", "1")

WURZEL = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, WURZEL)

from games import lamacrash_music as MU  # noqa: E402

OUT = os.path.join(WURZEL, "rust", "crates", "pgz-games", "src", "data", "lamacrash_music_ref.txt")


def bits(v):
    """IEEE-Bits einer Gleitkommazahl als 16 Hex-Zeichen (wie Rusts to_bits)."""
    return struct.pack(">d", float(v)).hex()


def sha(text):
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def song_text(song):
    out = "%s|%d|%d" % (bits(song["length"]), song["bars"], song["bpm"])
    for v in song["voices"]:
        out += "#" + bits(v["vol"])
        for n in v["notes"]:
            out += ";" + ",".join([bits(n[0]), bits(n[1]), bits(n[2]), n[3], bits(n[4]), bits(n[5])])
    return out


def text():
    zeilen = [sha(song_text(MU.compose(s))) for s in range(MU.STAGES)]
    sq = []
    for m in list(range(0, 10_500_001, 997)) + [199, 200, 229, 230, 10_000_000]:
        r = MU.squeak(m)
        sq.append("%d:-" % m if r is None else "%d:%s,%s" % (m, bits(r[0]), bits(r[1])))
    zeilen.append(sha(";".join(sq)))
    st = [str(MU.stage_of(m)) for m in (100, 149, 150, 300, 999, 1000, 10000, 10_000_000)]
    st += ["%d:%s" % (MU.bpm(s), bits(MU.beat_seconds(s))) for s in range(8)]
    zeilen.append(",".join(st))
    return "\n".join(zeilen) + "\n"


def main():
    neu = text()
    alt = None
    if os.path.exists(OUT):
        with open(OUT, encoding="utf-8") as f:
            alt = f.read()
    if "--check" in sys.argv:
        ok = alt == neu
        print("lamacrash_music_ref.txt %s" % ("aktuell" if ok else "VERALTET"))
        return 0 if ok else 1
    with open(OUT, "w", encoding="utf-8", newline="\n") as f:
        f.write(neu)
    print("geschrieben:", OUT)
    return 0


if __name__ == "__main__":
    sys.exit(main())
