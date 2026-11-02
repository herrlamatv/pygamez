//! zeiger_daten.rs
//! ===============
//! **Diese Datei ist erzeugt - nicht von Hand aendern.**
//!
//! Quelle: `pygame/cursors.py` der installierten pygame-ce-Fassung,
//! ausgeschnitten von `tools/gen_zeiger.py` dieser Kiste. Uebernommen
//! wird alles ab `arrow = Cursor(`: die sechs fertigen Zeiger, die fuenf
//! Zeichenkettenformen, `compile` und `load_xbm`.
//!
//! Warum uebernommen statt nachgebaut: zwei Drittel davon sind **Daten**
//! (rund 267 Zeilen Hexzahlen), und `compile`/`load_xbm` sind reines
//! Python. Was in pygame schon Python ist, wird hier uebernommen - die
//! gleiche Entscheidung wie bei `sysfont.py` und `version.py`.
//!
//! `Cursor` reicht [`crate::zeiger`] herein: `mouse.get_cursor()` muss
//! denselben Typ liefern wie `cursors.Cursor`, sonst scheitert jedes
//! `isinstance`.

/// Der uebernommene Abschnitt aus `pygame/cursors.py`.
pub const ZEIGER_PY: &str = r##"
arrow = Cursor(
    (16, 16),
    (0, 0),
    (
        0x00,
        0x00,
        0x40,
        0x00,
        0x60,
        0x00,
        0x70,
        0x00,
        0x78,
        0x00,
        0x7C,
        0x00,
        0x7E,
        0x00,
        0x7F,
        0x00,
        0x7F,
        0x80,
        0x7C,
        0x00,
        0x6C,
        0x00,
        0x46,
        0x00,
        0x06,
        0x00,
        0x03,
        0x00,
        0x03,
        0x00,
        0x00,
        0x00,
    ),
    (
        0x40,
        0x00,
        0xE0,
        0x00,
        0xF0,
        0x00,
        0xF8,
        0x00,
        0xFC,
        0x00,
        0xFE,
        0x00,
        0xFF,
        0x00,
        0xFF,
        0x80,
        0xFF,
        0xC0,
        0xFF,
        0x80,
        0xFE,
        0x00,
        0xEF,
        0x00,
        0x4F,
        0x00,
        0x07,
        0x80,
        0x07,
        0x80,
        0x03,
        0x00,
    ),
)

diamond = Cursor(
    (16, 16),
    (7, 7),
    (
        0,
        0,
        1,
        0,
        3,
        128,
        7,
        192,
        14,
        224,
        28,
        112,
        56,
        56,
        112,
        28,
        56,
        56,
        28,
        112,
        14,
        224,
        7,
        192,
        3,
        128,
        1,
        0,
        0,
        0,
        0,
        0,
    ),
    (
        1,
        0,
        3,
        128,
        7,
        192,
        15,
        224,
        31,
        240,
        62,
        248,
        124,
        124,
        248,
        62,
        124,
        124,
        62,
        248,
        31,
        240,
        15,
        224,
        7,
        192,
        3,
        128,
        1,
        0,
        0,
        0,
    ),
)

ball = Cursor(
    (16, 16),
    (7, 7),
    (
        0,
        0,
        3,
        192,
        15,
        240,
        24,
        248,
        51,
        252,
        55,
        252,
        127,
        254,
        127,
        254,
        127,
        254,
        127,
        254,
        63,
        252,
        63,
        252,
        31,
        248,
        15,
        240,
        3,
        192,
        0,
        0,
    ),
    (
        3,
        192,
        15,
        240,
        31,
        248,
        63,
        252,
        127,
        254,
        127,
        254,
        255,
        255,
        255,
        255,
        255,
        255,
        255,
        255,
        127,
        254,
        127,
        254,
        63,
        252,
        31,
        248,
        15,
        240,
        3,
        192,
    ),
)

broken_x = Cursor(
    (16, 16),
    (7, 7),
    (
        0,
        0,
        96,
        6,
        112,
        14,
        56,
        28,
        28,
        56,
        12,
        48,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        12,
        48,
        28,
        56,
        56,
        28,
        112,
        14,
        96,
        6,
        0,
        0,
    ),
    (
        224,
        7,
        240,
        15,
        248,
        31,
        124,
        62,
        62,
        124,
        30,
        120,
        14,
        112,
        0,
        0,
        0,
        0,
        14,
        112,
        30,
        120,
        62,
        124,
        124,
        62,
        248,
        31,
        240,
        15,
        224,
        7,
    ),
)

tri_left = Cursor(
    (16, 16),
    (1, 1),
    (
        0,
        0,
        96,
        0,
        120,
        0,
        62,
        0,
        63,
        128,
        31,
        224,
        31,
        248,
        15,
        254,
        15,
        254,
        7,
        128,
        7,
        128,
        3,
        128,
        3,
        128,
        1,
        128,
        1,
        128,
        0,
        0,
    ),
    (
        224,
        0,
        248,
        0,
        254,
        0,
        127,
        128,
        127,
        224,
        63,
        248,
        63,
        254,
        31,
        255,
        31,
        255,
        15,
        254,
        15,
        192,
        7,
        192,
        7,
        192,
        3,
        192,
        3,
        192,
        1,
        128,
    ),
)

tri_right = Cursor(
    (16, 16),
    (14, 1),
    (
        0,
        0,
        0,
        6,
        0,
        30,
        0,
        124,
        1,
        252,
        7,
        248,
        31,
        248,
        127,
        240,
        127,
        240,
        1,
        224,
        1,
        224,
        1,
        192,
        1,
        192,
        1,
        128,
        1,
        128,
        0,
        0,
    ),
    (
        0,
        7,
        0,
        31,
        0,
        127,
        1,
        254,
        7,
        254,
        31,
        252,
        127,
        252,
        255,
        248,
        255,
        248,
        127,
        240,
        3,
        240,
        3,
        224,
        3,
        224,
        3,
        192,
        3,
        192,
        1,
        128,
    ),
)


# Here is an example string resource cursor. To use this:
#    curs, mask = pygame.cursors.compile_cursor(pygame.cursors.thickarrow_strings, 'X', '.')
#    pygame.mouse.set_cursor((24, 24), (0, 0), curs, mask)
# Be warned, though, that cursors created from compiled strings do not support colors.

# sized 24x24
thickarrow_strings = (
    "XX                      ",
    "XXX                     ",
    "XXXX                    ",
    "XX.XX                   ",
    "XX..XX                  ",
    "XX...XX                 ",
    "XX....XX                ",
    "XX.....XX               ",
    "XX......XX              ",
    "XX.......XX             ",
    "XX........XX            ",
    "XX........XXX           ",
    "XX......XXXXX           ",
    "XX.XXX..XX              ",
    "XXXX XX..XX             ",
    "XX   XX..XX             ",
    "     XX..XX             ",
    "      XX..XX            ",
    "      XX..XX            ",
    "       XXXX             ",
    "       XX               ",
    "                        ",
    "                        ",
    "                        ",
)

# sized 24x16
sizer_x_strings = (
    "     X      X           ",
    "    XX      XX          ",
    "   X.X      X.X         ",
    "  X..X      X..X        ",
    " X...XXXXXXXX...X       ",
    "X................X      ",
    " X...XXXXXXXX...X       ",
    "  X..X      X..X        ",
    "   X.X      X.X         ",
    "    XX      XX          ",
    "     X      X           ",
    "                        ",
    "                        ",
    "                        ",
    "                        ",
    "                        ",
)

# sized 16x24
sizer_y_strings = (
    "     X          ",
    "    X.X         ",
    "   X...X        ",
    "  X.....X       ",
    " X.......X      ",
    "XXXXX.XXXXX     ",
    "    X.X         ",
    "    X.X         ",
    "    X.X         ",
    "    X.X         ",
    "    X.X         ",
    "    X.X         ",
    "    X.X         ",
    "XXXXX.XXXXX     ",
    " X.......X      ",
    "  X.....X       ",
    "   X...X        ",
    "    X.X         ",
    "     X          ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
)

# sized 24x16
sizer_xy_strings = (
    "XXXXXXXX                ",
    "X.....X                 ",
    "X....X                  ",
    "X...X                   ",
    "X..X.X                  ",
    "X.X X.X                 ",
    "XX   X.X    X           ",
    "X     X.X  XX           ",
    "       X.XX.X           ",
    "        X...X           ",
    "        X...X           ",
    "       X....X           ",
    "      X.....X           ",
    "     XXXXXXXX           ",
    "                        ",
    "                        ",
)

# sized 8x16
textmarker_strings = (
    "ooo ooo ",
    "   o    ",
    "   o    ",
    "   o    ",
    "   o    ",
    "   o    ",
    "   o    ",
    "   o    ",
    "   o    ",
    "   o    ",
    "   o    ",
    "ooo ooo ",
    "        ",
    "        ",
    "        ",
    "        ",
)


def compile(strings, black="X", white=".", xor="o"):
    """Create binary cursor data from simple strings.

    A sequence of strings can be used to create binary cursor data for the
    system cursor. This returns the binary data in the form of two tuples.
    Those can be passed as the third and fourth arguments respectively of the
    :func:`pygame.mouse.set_cursor()` function.

    If you are creating your own cursor strings, you can use any value represent
    the black and white pixels. Some system allow you to set a special toggle
    color for the system color, this is also called the xor color. If the system
    does not support xor cursors, that color will simply be black.

    The height must be divisible by 8. The width of the strings must all be equal
    and be divisible by 8. If these two conditions are not met, ``ValueError`` is
    raised.
    An example set of cursor strings looks like this

    ::

        thickarrow_strings = (               #sized 24x24
          "XX                      ",
          "XXX                     ",
          "XXXX                    ",
          "XX.XX                   ",
          "XX..XX                  ",
          "XX...XX                 ",
          "XX....XX                ",
          "XX.....XX               ",
          "XX......XX              ",
          "XX.......XX             ",
          "XX........XX            ",
          "XX........XXX           ",
          "XX......XXXXX           ",
          "XX.XXX..XX              ",
          "XXXX XX..XX             ",
          "XX   XX..XX             ",
          "     XX..XX             ",
          "      XX..XX            ",
          "      XX..XX            ",
          "       XXXX             ",
          "       XX               ",
          "                        ",
          "                        ",
          "                        ")

    """
    # first check for consistent lengths
    size = len(strings[0]), len(strings)
    if size[0] % 8 or size[1] % 8:
        raise ValueError(f"cursor string sizes must be divisible by 8 {size}")

    for s in strings[1:]:
        if len(s) != size[0]:
            raise ValueError("Cursor strings are inconsistent lengths")

    # create the data arrays.
    # this could stand a little optimizing
    maskdata = []
    filldata = []
    maskitem = fillitem = 0
    step = 8
    for s in strings:
        for c in s:
            maskitem = maskitem << 1
            fillitem = fillitem << 1
            step = step - 1
            if c == black:
                maskitem = maskitem | 1
                fillitem = fillitem | 1
            elif c == white:
                maskitem = maskitem | 1
            elif c == xor:
                fillitem = fillitem | 1

            if not step:
                maskdata.append(maskitem)
                filldata.append(fillitem)
                maskitem = fillitem = 0
                step = 8

    return tuple(filldata), tuple(maskdata)


def load_xbm(curs, mask):
    """Load cursor data from an XBM file.

    This loads cursors for a simple subset of ``XBM`` files. ``XBM`` files are
    traditionally used to store cursors on UNIX systems, they are an ASCII
    format used to represent simple images.

    Sometimes the black and white color values will be split into two separate
    ``XBM`` files. You can pass a second maskfile argument to load the two
    images into a single cursor.

    The cursorfile and maskfile arguments can either be filenames or file-like
    object with the readlines method.

    The return value cursor_args can be passed directly to the
    ``pygame.mouse.set_cursor()`` function.
    """

    def bitswap(num):
        val = 0
        for x in range(8):
            b = num & (1 << x) != 0
            val = val << 1 | b
        return val

    if hasattr(curs, "readlines"):
        curs = curs.readlines()
    else:
        with open(curs, encoding="ascii") as cursor_f:
            curs = cursor_f.readlines()

    if hasattr(mask, "readlines"):
        mask = mask.readlines()
    else:
        with open(mask, encoding="ascii") as mask_f:
            mask = mask_f.readlines()

    # avoid comments
    for i, line in enumerate(curs):
        if line.startswith("#define"):
            curs = curs[i:]
            break

    for i, line in enumerate(mask):
        if line.startswith("#define"):
            mask = mask[i:]
            break

    # load width,height
    width = int(curs[0].split()[-1])
    height = int(curs[1].split()[-1])
    # load hotspot position
    if curs[2].startswith("#define"):
        hotx = int(curs[2].split()[-1])
        hoty = int(curs[3].split()[-1])
    else:
        hotx = hoty = 0

    info = width, height, hotx, hoty

    possible_starts = ("static char", "static unsigned char")
    for i, line in enumerate(curs):
        if line.startswith(possible_starts):
            break
    data = " ".join(curs[i + 1 :]).replace("};", "").replace(",", " ")
    cursdata = tuple(bitswap(int(x, 16)) for x in data.split())
    for i, line in enumerate(mask):
        if line.startswith(possible_starts):
            break
    data = " ".join(mask[i + 1 :]).replace("};", "").replace(",", " ")
    maskdata = tuple(bitswap(int(x, 16)) for x in data.split())
    return info[:2], info[2:], cursdata, maskdata
"##;
