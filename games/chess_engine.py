# -*- coding: utf-8 -*-
"""
chess_engine.py
===============
Regeln, Notation und KI für Schach (``games/chess.py``) - ohne pygame.

Brett
    0x88-Brett (128 Felder, Index = Reihe * 16 + Linie, a1 = 0, h8 = 119).
    Ein Feld liegt genau dann auf dem Brett, wenn ``sq & 0x88 == 0`` ist -
    das spart bei jedem Schritt eines Läufers/Turms die Randprüfung.
    Figuren sind Zahlen: Typ (1 = Bauer … 6 = König) plus 8 für Schwarz.

Regeln
    Vollständig inklusive Rochade, En passant, Umwandlung, Schach/Matt/Patt,
    50-Züge-Regel, Stellungswiederholung und ungenügendem Material. Die
    Rochade ist für **Chess960** verallgemeinert: König landet immer auf g/c,
    der Turm auf f/d; alle Felder dazwischen müssen frei sein (bis auf König
    und Rochadeturm) und der König darf weder im Schach stehen noch über ein
    angegriffenes Feld ziehen. Intern ist ein Rochadezug "König schlägt
    eigenen Turm" - damit bleibt er auch dann eindeutig, wenn der König gar
    nicht zieht. Die Züge werden per Perft gegen bekannte Werte geprüft
    (``tests/audit_chess.py``).

Notation
    FEN (inkl. X-FEN/Shredder-Rochaderechte), SAN (lesen + schreiben),
    UCI und PGN-Export.

KI
    Negamax mit Alpha-Beta als **fortsetzbarer Generator**: ``Search.run()``
    rechnet so lange, bis ``slice_end`` (perf_counter) überschritten ist, und
    gibt dann per ``yield`` ab. Das Spiel ruft den Generator jedes Frame für
    einige Millisekunden auf - ohne Thread und ohne Ruckeln. Dazu kommen
    iterative Vertiefung, Transpositionstabelle (Zobrist-Schlüssel),
    Principal-Variation-Search, Null-Zug, Late-Move-Reductions, Killer- und
    History-Heuristik und eine Ruhesuche. Die Bewertung ist verjüngt
    (Mittel-/Endspiel, PeSTO-Tabellen) mit Mobilität, Bauernstruktur,
    Königssicherheit, Läuferpaar, Türmen auf offenen Linien und
    "Mop-up" für gewonnene Endspiele.

Die Web-Version (``web/js/games/chess_engine.js``) ist eine 1:1-Übertragung:
gleiche Zuggenerierungs-Reihenfolge, gleiche Bewertung, gleiche Stufen - bei
gleichem Knotenlimit finden beide denselben Zug.
"""

import random
import time

WHITE, BLACK = 0, 1
PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING = 1, 2, 3, 4, 5, 6

# Zug-Kodierung: von | nach << 8 | Umwandlung << 16 | Art << 20
M_NORMAL, M_DOUBLE, M_EP, M_CASTLE = 0, 1, 2, 3

SQUARES = tuple(r * 16 + f for r in range(8) for f in range(8))
KNIGHT_D = (33, 31, 18, 14, -14, -18, -31, -33)
KING_D = (17, 16, 15, 1, -1, -15, -16, -17)
BISHOP_D = (17, 15, -15, -17)
ROOK_D = (16, 1, -1, -16)

LETTERS = " PNBRQK"
FILES = "abcdefgh"

MATE = 30000
INF = 32000
MAX_PLY = 64


def sq_name(sq):
    return FILES[sq & 7] + str((sq >> 4) + 1)


def sq_parse(name):
    return (int(name[1]) - 1) * 16 + FILES.index(name[0])


def piece_char(p):
    ch = LETTERS[p & 7]
    return ch.lower() if p >> 3 else ch


# ----------------------------------------------------------------- Zobrist
def _zobrist_numbers(count):
    """Feste 64-Bit-Zufallszahlen (xorshift64*) - reproduzierbar."""
    x = 0x9E3779B97F4A7C15
    out = []
    mask = (1 << 64) - 1
    for _ in range(count):
        x ^= (x >> 12)
        x ^= (x << 25) & mask
        x ^= (x >> 27)
        out.append((x * 0x2545F4914F6CDD1D) & mask)
    return out


_ZN = _zobrist_numbers(16 * 128 + 1 + 32 + 8)
Z_PIECE = [_ZN[i * 128:(i + 1) * 128] for i in range(16)]
Z_SIDE = _ZN[16 * 128]
Z_CASTLE = [_ZN[16 * 128 + 1 + i * 8:16 * 128 + 1 + (i + 1) * 8] for i in range(4)]
Z_EP = _ZN[16 * 128 + 33:16 * 128 + 41]


def _castle_key(castle):
    h = 0
    for i in range(4):
        if castle[i] >= 0:
            h ^= Z_CASTLE[i][castle[i] & 7]
    return h


# ------------------------------------------------------------ Bewertung
# PeSTO-Tabellen (Rofchade, gemeinfrei veröffentlicht im Chess Programming
# Wiki). Index 0 = a8 aus Sicht von Weiß; Schwarz wird vertikal gespiegelt.
MG_VALUE = (0, 82, 337, 365, 477, 1025, 0)
EG_VALUE = (0, 94, 281, 297, 512, 936, 0)
PHASE_INC = (0, 0, 1, 1, 2, 4, 0)

_MG_TABLES = {
    PAWN: (
        0, 0, 0, 0, 0, 0, 0, 0,
        98, 134, 61, 95, 68, 126, 34, -11,
        -6, 7, 26, 31, 65, 56, 25, -20,
        -14, 13, 6, 21, 23, 12, 17, -23,
        -27, -2, -5, 12, 17, 6, 10, -25,
        -26, -4, -4, -10, 3, 3, 33, -12,
        -35, -1, -20, -23, -15, 24, 38, -22,
        0, 0, 0, 0, 0, 0, 0, 0),
    KNIGHT: (
        -167, -89, -34, -49, 61, -97, -15, -107,
        -73, -41, 72, 36, 23, 62, 7, -17,
        -47, 60, 37, 65, 84, 129, 73, 44,
        -9, 17, 19, 53, 37, 69, 18, 22,
        -13, 4, 16, 13, 28, 19, 21, -8,
        -23, -9, 12, 10, 19, 17, 25, -16,
        -29, -53, -12, -3, -1, 18, -14, -19,
        -105, -21, -58, -33, -17, -28, -19, -23),
    BISHOP: (
        -29, 4, -82, -37, -25, -42, 7, -8,
        -26, 16, -18, -13, 30, 59, 18, -47,
        -16, 37, 43, 40, 35, 50, 37, -2,
        -4, 5, 19, 50, 37, 37, 7, -2,
        -6, 13, 13, 26, 34, 12, 10, 4,
        0, 15, 15, 15, 14, 27, 18, 10,
        4, 15, 16, 0, 7, 21, 33, 1,
        -33, -3, -14, -21, -13, -12, -39, -21),
    ROOK: (
        32, 42, 32, 51, 63, 9, 31, 43,
        27, 32, 58, 62, 80, 67, 26, 44,
        -5, 19, 26, 36, 17, 45, 61, 16,
        -24, -11, 7, 26, 24, 35, -8, -20,
        -36, -26, -12, -1, 9, -7, 6, -23,
        -45, -25, -16, -17, 3, 0, -5, -33,
        -44, -16, -20, -9, -1, 11, -6, -71,
        -19, -13, 1, 17, 16, 7, -37, -26),
    QUEEN: (
        -28, 0, 29, 12, 59, 44, 43, 45,
        -24, -39, -5, 1, -16, 57, 28, 54,
        -13, -17, 7, 8, 29, 56, 47, 57,
        -27, -27, -16, -16, -1, 17, -2, 1,
        -9, -26, -9, -10, -2, -4, 3, -3,
        -14, 2, -11, -2, -5, 2, 14, 5,
        -35, -8, 11, 2, 8, 15, -3, 1,
        -1, -18, -9, 10, -15, -25, -31, -50),
    KING: (
        -65, 23, 16, -15, -56, -34, 2, 13,
        29, -1, -20, -7, -8, -4, -38, -29,
        -9, 24, 2, -16, -20, 6, 22, -22,
        -17, -20, -12, -27, -30, -25, -14, -36,
        -49, -1, -27, -39, -46, -44, -33, -51,
        -14, -14, -22, -46, -44, -30, -15, -27,
        1, 7, -8, -64, -43, -16, 9, 8,
        -15, 36, 12, -54, 8, -28, 24, 14),
}
_EG_TABLES = {
    PAWN: (
        0, 0, 0, 0, 0, 0, 0, 0,
        178, 173, 158, 134, 147, 132, 165, 187,
        94, 100, 85, 67, 56, 53, 82, 84,
        32, 24, 13, 5, -2, 4, 17, 17,
        13, 9, -3, -7, -7, -8, 3, -1,
        4, 7, -6, 1, 0, -5, -1, -8,
        13, 8, 8, 10, 13, 0, 2, -7,
        0, 0, 0, 0, 0, 0, 0, 0),
    KNIGHT: (
        -58, -38, -13, -28, -31, -27, -63, -99,
        -25, -8, -25, -2, -9, -25, -24, -52,
        -24, -20, 10, 9, -1, -9, -19, -41,
        -17, 3, 22, 22, 22, 11, 8, -18,
        -18, -6, 16, 25, 16, 17, 4, -18,
        -23, -3, -1, 15, 10, -3, -20, -22,
        -42, -20, -10, -5, -2, -20, -23, -44,
        -29, -51, -23, -15, -22, -18, -50, -64),
    BISHOP: (
        -14, -21, -11, -8, -7, -9, -17, -24,
        -8, -4, 7, -12, -3, -13, -4, -14,
        2, -8, 0, -1, -2, 6, 0, 4,
        -3, 9, 12, 9, 14, 10, 3, 2,
        -6, 3, 13, 19, 7, 10, -3, -9,
        -12, -3, 8, 10, 13, 3, -7, -15,
        -14, -18, -7, -1, 4, -9, -15, -27,
        -23, -9, -23, -5, -9, -16, -5, -17),
    ROOK: (
        13, 10, 18, 15, 12, 12, 8, 5,
        11, 13, 13, 11, -3, 3, 8, 3,
        7, 7, 7, 5, 4, -3, -5, -3,
        4, 3, 13, 1, 2, 1, -1, 2,
        3, 5, 8, 4, -5, -6, -8, -11,
        -4, 0, -5, -1, -7, -12, -8, -16,
        -6, -6, 0, 2, -9, -9, -11, -3,
        -9, 2, 3, -1, -5, -13, 4, -20),
    QUEEN: (
        -9, 22, 22, 27, 27, 19, 10, 20,
        -17, 20, 32, 41, 58, 25, 30, 0,
        -20, 6, 9, 49, 47, 35, 19, 9,
        3, 22, 24, 45, 57, 40, 57, 36,
        -18, 28, 19, 47, 31, 34, 39, 23,
        -16, -27, 15, 6, 9, 17, 10, 5,
        -22, -23, -30, -16, -16, -23, -36, -32,
        -33, -28, -22, -43, -5, -32, -20, -41),
    KING: (
        -74, -35, -18, -18, -11, 15, 4, -17,
        -12, 17, 14, 17, 17, 38, 23, 11,
        10, 17, 23, 15, 20, 45, 44, 13,
        -8, 22, 24, 27, 26, 33, 26, 3,
        -18, -4, 21, 24, 27, 23, 9, -11,
        -19, -3, 11, 21, 23, 16, 7, -9,
        -27, -11, 4, 13, 14, 4, -5, -17,
        -53, -34, -21, -11, -28, -14, -24, -43),
}


def _build_pst(tables, values):
    """Material + Feldwert je Figur (16) und 0x88-Feld (128)."""
    out = [[0] * 128 for _ in range(16)]
    for typ in range(1, 7):
        for sq in SQUARES:
            r, f = sq >> 4, sq & 7
            out[typ][sq] = values[typ] + tables[typ][(7 - r) * 8 + f]
            out[typ | 8][sq] = values[typ] + tables[typ][r * 8 + f]
    return out


PST_MG = _build_pst(_MG_TABLES, MG_VALUE)
PST_EG = _build_pst(_EG_TABLES, EG_VALUE)

# Zusatz-Terme (Zentibauern, [Mittelspiel, Endspiel])
BISHOP_PAIR = (25, 45)
DOUBLED = (8, 18)
ISOLATED = (10, 12)
PASSED_MG = (0, 5, 8, 15, 28, 50, 80, 0)       # Index = Reihe aus eigener Sicht
PASSED_EG = (0, 10, 15, 28, 50, 85, 130, 0)
ROOK_OPEN = (20, 10)
ROOK_HALF = (10, 5)
MOB_BASE = (0, 0, 4, 7, 7, 14, 0)              # "normale" Zahl erreichbarer Felder
MOB_MG = (0, 0, 4, 3, 2, 1, 0)
MOB_EG = (0, 0, 4, 4, 4, 2, 0)
SHIELD_NEAR = 0
SHIELD_FAR = 10
SHIELD_NONE = 25
SHIELD_OPEN = 15
TEMPO = 10
# Grobe Figurenwerte (Zugsortierung, Materialbilanz, Mop-up)
ORDER_VALUE = (0, 1, 3, 3, 5, 9, 20)

_PAWN_CACHE = {}


# ================================================================ Position
class Position:
    """Eine Schachstellung mit Zug ausführen/zurücknehmen (make/unmake)."""

    __slots__ = ("b", "side", "castle", "ep", "half", "full", "kings", "hash",
                 "phash", "mg", "eg", "phase", "counts", "stack", "hist",
                 "chess960")

    def __init__(self):
        self.b = [0] * 128
        self.side = WHITE
        self.castle = (-1, -1, -1, -1)   # Turmfeld je Recht: wK, wQ, bK, bQ
        self.ep = -1
        self.half = 0
        self.full = 1
        self.kings = [-1, -1]
        self.hash = 0
        self.phash = 0
        self.mg = [0, 0]
        self.eg = [0, 0]
        self.phase = 0
        self.counts = [0] * 16
        self.stack = []
        self.hist = []
        self.chess960 = False

    # ------------------------------------------------------------ Aufbau
    def copy(self):
        p = Position.__new__(Position)
        p.b = self.b[:]
        p.side = self.side
        p.castle = self.castle
        p.ep = self.ep
        p.half = self.half
        p.full = self.full
        p.kings = self.kings[:]
        p.hash = self.hash
        p.phash = self.phash
        p.mg = self.mg[:]
        p.eg = self.eg[:]
        p.phase = self.phase
        p.counts = self.counts[:]
        p.stack = []
        p.hist = self.hist[:]
        p.chess960 = self.chess960
        return p

    def _recompute(self):
        """Hash, Bewertungssummen und Zähler komplett neu berechnen."""
        b = self.b
        self.mg = [0, 0]
        self.eg = [0, 0]
        self.phase = 0
        self.counts = [0] * 16
        h = 0
        ph = 0
        for sq in SQUARES:
            p = b[sq]
            if not p:
                continue
            c = p >> 3
            self.mg[c] += PST_MG[p][sq]
            self.eg[c] += PST_EG[p][sq]
            self.phase += PHASE_INC[p & 7]
            self.counts[p] += 1
            h ^= Z_PIECE[p][sq]
            if p & 7 == PAWN:
                ph ^= Z_PIECE[p][sq]
            if p & 7 == KING:
                self.kings[c] = sq
        if self.side == BLACK:
            h ^= Z_SIDE
        h ^= _castle_key(self.castle)
        if self.ep >= 0:
            h ^= Z_EP[self.ep & 7]
        self.hash = h
        self.phash = ph
        self.hist = [h]

    # --------------------------------------------------------- Angriffe
    def attacked(self, sq, by):
        """True, wenn Feld 'sq' von Farbe 'by' angegriffen wird."""
        b = self.b
        if by == WHITE:
            s = sq - 15
            if not s & 0x88 and b[s] == 1:
                return True
            s = sq - 17
            if not s & 0x88 and b[s] == 1:
                return True
            kn, kg, bi, rk, qu = 2, 6, 3, 4, 5
        else:
            s = sq + 15
            if not s & 0x88 and b[s] == 9:
                return True
            s = sq + 17
            if not s & 0x88 and b[s] == 9:
                return True
            kn, kg, bi, rk, qu = 10, 14, 11, 12, 13
        for d in KNIGHT_D:
            s = sq + d
            if not s & 0x88 and b[s] == kn:
                return True
        for d in KING_D:
            s = sq + d
            if not s & 0x88 and b[s] == kg:
                return True
        for d in BISHOP_D:
            s = sq + d
            while not s & 0x88:
                q = b[s]
                if q:
                    if q == bi or q == qu:
                        return True
                    break
                s += d
        for d in ROOK_D:
            s = sq + d
            while not s & 0x88:
                q = b[s]
                if q:
                    if q == rk or q == qu:
                        return True
                    break
                s += d
        return False

    def in_check(self, color=None):
        c = self.side if color is None else color
        return self.attacked(self.kings[c], 1 - c)

    # --------------------------------------------------------- Zuggenerator
    def gen_moves(self, tactical=False):
        """Pseudolegale Züge. tactical=True: nur Schlagzüge + Damenumwandlung."""
        b = self.b
        us = self.side
        them = 1 - us
        moves = []
        add = moves.append
        if us == WHITE:
            fwd, start_rank, promo_rank = 16, 1, 7
        else:
            fwd, start_rank, promo_rank = -16, 6, 0
        ep = self.ep
        for sq in SQUARES:
            p = b[sq]
            if not p or (p >> 3) != us:
                continue
            typ = p & 7
            if typ == PAWN:
                to = sq + fwd
                promo = (to >> 4) == promo_rank
                if not b[to]:
                    if promo:
                        add(sq | to << 8 | QUEEN << 16)
                        if not tactical:
                            add(sq | to << 8 | ROOK << 16)
                            add(sq | to << 8 | BISHOP << 16)
                            add(sq | to << 8 | KNIGHT << 16)
                    elif not tactical:
                        add(sq | to << 8)
                        if (sq >> 4) == start_rank and not b[to + fwd]:
                            add(sq | (to + fwd) << 8 | M_DOUBLE << 20)
                for t in (to - 1, to + 1):
                    if t & 0x88:
                        continue
                    q = b[t]
                    if q and (q >> 3) == them:
                        if promo:
                            add(sq | t << 8 | QUEEN << 16)
                            if not tactical:
                                add(sq | t << 8 | ROOK << 16)
                                add(sq | t << 8 | BISHOP << 16)
                                add(sq | t << 8 | KNIGHT << 16)
                        else:
                            add(sq | t << 8)
                    elif t == ep:
                        add(sq | t << 8 | M_EP << 20)
            elif typ == KNIGHT or typ == KING:
                for d in (KNIGHT_D if typ == KNIGHT else KING_D):
                    t = sq + d
                    if t & 0x88:
                        continue
                    q = b[t]
                    if not q:
                        if not tactical:
                            add(sq | t << 8)
                    elif (q >> 3) == them:
                        add(sq | t << 8)
                if typ == KING and not tactical:
                    self._gen_castles(sq, add)
            else:
                for d in (BISHOP_D if typ == BISHOP else ROOK_D if typ == ROOK
                          else KING_D):
                    t = sq + d
                    while not t & 0x88:
                        q = b[t]
                        if not q:
                            if not tactical:
                                add(sq | t << 8)
                        else:
                            if (q >> 3) == them:
                                add(sq | t << 8)
                            break
                        t += d
        return moves

    def _gen_castles(self, k, add):
        us = self.side
        c = self.castle
        if c[us * 2] < 0 and c[us * 2 + 1] < 0:
            return
        base = 0 if us == WHITE else 112
        if (k >> 4) != (base >> 4):
            return
        them = 1 - us
        if self.attacked(k, them):
            return
        b = self.b
        rook = ROOK | (us << 3)
        for i in (0, 1):
            rsq = c[us * 2 + i]
            if rsq < 0 or b[rsq] != rook:
                continue
            kt = base + (6 if i == 0 else 2)
            rt = base + (5 if i == 0 else 3)
            lo = min(k, kt, rsq, rt)
            hi = max(k, kt, rsq, rt)
            free = True
            for s in range(lo, hi + 1):
                if b[s] and s != k and s != rsq:
                    free = False
                    break
            if not free:
                continue
            # König und Turm anheben: der Weg des Königs darf nicht
            # angegriffen sein (Endfeld inklusive).
            kp = b[k]
            b[k] = 0
            b[rsq] = 0
            safe = True
            step = 1 if kt > k else -1
            s = k
            while s != kt:
                s += step
                if self.attacked(s, them):
                    safe = False
                    break
            b[k] = kp
            b[rsq] = rook
            if safe:
                add(k | rsq << 8 | M_CASTLE << 20)

    def legal_moves(self):
        us = self.side
        res = []
        for m in self.gen_moves():
            self.make(m)
            if not self.attacked(self.kings[us], 1 - us):
                res.append(m)
            self.unmake()
        return res

    def has_legal_move(self):
        us = self.side
        for m in self.gen_moves():
            self.make(m)
            ok = not self.attacked(self.kings[us], 1 - us)
            self.unmake()
            if ok:
                return True
        return False

    def is_legal(self, m):
        us = self.side
        self.make(m)
        ok = not self.attacked(self.kings[us], 1 - us)
        self.unmake()
        return ok

    # --------------------------------------------------------- Zug ausführen
    def make(self, m):
        b = self.b
        frm = m & 255
        to = (m >> 8) & 255
        flag = m >> 20
        us = self.side
        them = 1 - us
        p = b[frm]
        typ = p & 7
        cap = 0 if flag == M_CASTLE else b[to]
        self.stack.append((m, p, cap, self.castle, self.ep, self.half,
                           self.hash, self.phash, self.mg[0], self.mg[1],
                           self.eg[0], self.eg[1], self.phase))
        h = self.hash ^ Z_SIDE
        if self.ep >= 0:
            h ^= Z_EP[self.ep & 7]
        self.ep = -1
        half = self.half + 1
        mg = self.mg
        eg = self.eg
        zp = Z_PIECE
        if flag == M_CASTLE:
            base = frm & 0x70
            if to > frm:
                kt, rt = base + 6, base + 5
            else:
                kt, rt = base + 2, base + 3
            rp = b[to]
            b[frm] = 0
            b[to] = 0
            b[kt] = p
            b[rt] = rp
            h ^= zp[p][frm] ^ zp[p][kt] ^ zp[rp][to] ^ zp[rp][rt]
            mg[us] += PST_MG[p][kt] - PST_MG[p][frm] + PST_MG[rp][rt] - PST_MG[rp][to]
            eg[us] += PST_EG[p][kt] - PST_EG[p][frm] + PST_EG[rp][rt] - PST_EG[rp][to]
            self.kings[us] = kt
        else:
            if flag == M_EP:
                cs = to - 16 if us == WHITE else to + 16
                cp = b[cs]
                b[cs] = 0
                h ^= zp[cp][cs]
                self.phash ^= zp[cp][cs]
                mg[them] -= PST_MG[cp][cs]
                eg[them] -= PST_EG[cp][cs]
                self.counts[cp] -= 1
                half = 0
            elif cap:
                h ^= zp[cap][to]
                mg[them] -= PST_MG[cap][to]
                eg[them] -= PST_EG[cap][to]
                self.phase -= PHASE_INC[cap & 7]
                self.counts[cap] -= 1
                if cap & 7 == PAWN:
                    self.phash ^= zp[cap][to]
                half = 0
            b[frm] = 0
            h ^= zp[p][frm]
            mg[us] -= PST_MG[p][frm]
            eg[us] -= PST_EG[p][frm]
            promo = (m >> 16) & 7
            if promo:
                np_ = promo | (us << 3)
                self.counts[p] -= 1
                self.counts[np_] += 1
                self.phase += PHASE_INC[promo]
                self.phash ^= zp[p][frm]
            else:
                np_ = p
                if typ == PAWN:
                    self.phash ^= zp[p][frm] ^ zp[p][to]
            b[to] = np_
            h ^= zp[np_][to]
            mg[us] += PST_MG[np_][to]
            eg[us] += PST_EG[np_][to]
            if typ == PAWN:
                half = 0
                if flag == M_DOUBLE:
                    # En-passant-Feld nur setzen, wenn ein Gegnerbauer daneben
                    # steht (sonst unterschieden sich gleiche Stellungen).
                    ep_pawn = PAWN | (them << 3)
                    if (not (to - 1) & 0x88 and b[to - 1] == ep_pawn) or \
                            (not (to + 1) & 0x88 and b[to + 1] == ep_pawn):
                        self.ep = (frm + to) >> 1
                        h ^= Z_EP[self.ep & 7]
            elif typ == KING:
                self.kings[us] = to
        c = self.castle
        if c != (-1, -1, -1, -1):
            nc = list(c)
            if typ == KING:
                nc[us * 2] = nc[us * 2 + 1] = -1
            for i in range(4):
                if nc[i] >= 0 and (nc[i] == frm or nc[i] == to):
                    nc[i] = -1
            nc = tuple(nc)
            if nc != c:
                h ^= _castle_key(c) ^ _castle_key(nc)
                self.castle = nc
        self.hash = h
        self.half = half
        if us == BLACK:
            self.full += 1
        self.side = them
        self.hist.append(h)

    def unmake(self):
        (m, p, cap, castle, ep, half, h, ph, mg0, mg1, eg0, eg1,
         phase) = self.stack.pop()
        self.hist.pop()
        b = self.b
        them = self.side
        us = 1 - them
        self.side = us
        if us == BLACK:
            self.full -= 1
        frm = m & 255
        to = (m >> 8) & 255
        flag = m >> 20
        if flag == M_CASTLE:
            base = frm & 0x70
            if to > frm:
                kt, rt = base + 6, base + 5
            else:
                kt, rt = base + 2, base + 3
            rp = b[rt]
            b[kt] = 0
            b[rt] = 0
            b[frm] = p
            b[to] = rp
            self.kings[us] = frm
        else:
            promo = (m >> 16) & 7
            if promo:
                self.counts[promo | (us << 3)] -= 1
                self.counts[p] += 1
            b[to] = cap
            b[frm] = p
            if cap:
                self.counts[cap] += 1
            if flag == M_EP:
                cp = PAWN | (them << 3)
                b[to - 16 if us == WHITE else to + 16] = cp
                self.counts[cp] += 1
            if p & 7 == KING:
                self.kings[us] = frm
        self.castle = castle
        self.ep = ep
        self.half = half
        self.hash = h
        self.phash = ph
        self.mg[0] = mg0
        self.mg[1] = mg1
        self.eg[0] = eg0
        self.eg[1] = eg1
        self.phase = phase

    def make_null(self):
        self.stack.append((0, 0, 0, self.castle, self.ep, self.half, self.hash,
                           self.phash, self.mg[0], self.mg[1], self.eg[0],
                           self.eg[1], self.phase))
        h = self.hash ^ Z_SIDE
        if self.ep >= 0:
            h ^= Z_EP[self.ep & 7]
        self.ep = -1
        self.hash = h
        # Über einen Null-Zug hinweg gibt es keine Stellungswiederholung.
        self.half = 0
        self.side = 1 - self.side
        self.hist.append(h)

    def unmake_null(self):
        (_, _, _, castle, ep, half, h, _, _, _, _, _, _) = self.stack.pop()
        self.hist.pop()
        self.side = 1 - self.side
        self.ep = ep
        self.half = half
        self.hash = h

    # --------------------------------------------------------- Zustände
    def is_capture(self, m):
        flag = m >> 20
        return flag == M_EP or (flag != M_CASTLE and self.b[(m >> 8) & 255] != 0)

    def repetitions(self):
        """Wie oft die aktuelle Stellung (seit dem letzten unumkehrbaren
        Zug) schon auf dem Brett stand - inklusive jetzt."""
        h = self.hash
        hist = self.hist
        n = 0
        i = len(hist) - 1
        stop = max(0, len(hist) - 1 - self.half)
        while i >= stop:
            if hist[i] == h:
                n += 1
            i -= 2
        return n

    def insufficient_material(self):
        """Remis: kein Matt mehr möglich (K-K, K+Leichtfigur-K, K+L-K+L
        mit Läufern gleicher Feldfarbe)."""
        c = self.counts
        if c[1] or c[9] or c[4] or c[12] or c[5] or c[13]:
            return False
        minors = c[2] + c[3] + c[10] + c[11]
        if minors <= 1:
            return True
        if c[2] or c[10]:
            return False
        colors = set()
        for sq in SQUARES:
            if self.b[sq] & 7 == BISHOP:
                colors.add(((sq >> 4) + (sq & 7)) & 1)
        return len(colors) == 1

    def can_mate(self, color):
        """Hat 'color' genug Material, um überhaupt mattsetzen zu können?
        (für Zeitüberschreitung: sonst Remis)."""
        c = self.counts
        o = color << 3
        if c[PAWN | o] or c[ROOK | o] or c[QUEEN | o]:
            return True
        return c[KNIGHT | o] + c[BISHOP | o] >= 2

    def non_pawn_material(self, color):
        c = self.counts
        o = color << 3
        return c[KNIGHT | o] + c[BISHOP | o] + c[ROOK | o] + c[QUEEN | o]

    def material(self, color):
        c = self.counts
        o = color << 3
        return (c[PAWN | o] + 3 * c[KNIGHT | o] + 3 * c[BISHOP | o]
                + 5 * c[ROOK | o] + 9 * c[QUEEN | o])

    def status(self):
        """None (läuft) oder 'checkmate'/'stalemate'/'fifty'/'threefold'/
        'material'."""
        if not self.has_legal_move():
            return "checkmate" if self.in_check() else "stalemate"
        if self.insufficient_material():
            return "material"
        if self.half >= 100:
            return "fifty"
        if self.repetitions() >= 3:
            return "threefold"
        return None

    # --------------------------------------------------------- Bewertung
    def evaluate(self):
        """Statische Bewertung aus Sicht der Seite am Zug (Zentibauern)."""
        b = self.b
        c = self.counts
        mg = self.mg[0] - self.mg[1]
        eg = self.eg[0] - self.eg[1]
        if c[3] >= 2:
            mg += BISHOP_PAIR[0]
            eg += BISHOP_PAIR[1]
        if c[11] >= 2:
            mg -= BISHOP_PAIR[0]
            eg -= BISHOP_PAIR[1]
        pe = _PAWN_CACHE.get(self.phash)
        if pe is None:
            pe = self._pawn_eval()
        mg += pe[0]
        eg += pe[1]
        wf = pe[2]
        bf = pe[3]
        # Mobilität + Türme auf offenen Linien
        for sq in SQUARES:
            p = b[sq]
            if not p:
                continue
            typ = p & 7
            if typ == PAWN or typ == KING:
                continue
            col = p >> 3
            n = 0
            if typ == KNIGHT:
                for d in KNIGHT_D:
                    t = sq + d
                    if not t & 0x88:
                        q = b[t]
                        if not q or (q >> 3) != col:
                            n += 1
            else:
                for d in (BISHOP_D if typ == BISHOP else ROOK_D if typ == ROOK
                          else KING_D):
                    t = sq + d
                    while not t & 0x88:
                        q = b[t]
                        if q:
                            if (q >> 3) != col:
                                n += 1
                            break
                        n += 1
                        t += d
            n -= MOB_BASE[typ]
            dm = n * MOB_MG[typ]
            de = n * MOB_EG[typ]
            if typ == ROOK:
                f = sq & 7
                own = wf[f] if col == WHITE else bf[f]
                if own == 0:
                    if (bf[f] if col == WHITE else wf[f]) == 0:
                        dm += ROOK_OPEN[0]
                        de += ROOK_OPEN[1]
                    else:
                        dm += ROOK_HALF[0]
                        de += ROOK_HALF[1]
            if col == WHITE:
                mg += dm
                eg += de
            else:
                mg -= dm
                eg -= de
        # Königssicherheit (Bauernschild) - zählt nur im Mittelspiel
        mg -= self._shield_penalty(WHITE, wf, bf)
        mg += self._shield_penalty(BLACK, wf, bf)
        # Mop-up: gewonnenes Endspiel ohne gegnerische Bauern -> den
        # gegnerischen König an den Rand drängen, den eigenen heranführen.
        if self.phase <= 8:
            mw = self.material(WHITE)
            mb = self.material(BLACK)
            if mw >= mb + 4 and c[9] == 0:
                eg += self._mop_up(self.kings[WHITE], self.kings[BLACK])
            elif mb >= mw + 4 and c[1] == 0:
                eg -= self._mop_up(self.kings[BLACK], self.kings[WHITE])
        phase = self.phase if self.phase < 24 else 24
        score = (mg * phase + eg * (24 - phase)) // 24
        return (score if self.side == WHITE else -score) + TEMPO

    @staticmethod
    def _mop_up(win_k, lose_k):
        lf, lr = lose_k & 7, lose_k >> 4
        center = max(3 - lf, lf - 4) + max(3 - lr, lr - 4)
        dist = abs((win_k & 7) - lf) + abs((win_k >> 4) - lr)
        return 10 * center + 4 * (14 - dist)

    def _shield_penalty(self, color, wf, bf):
        k = self.kings[color]
        r = k >> 4
        rel = r if color == WHITE else 7 - r
        if rel > 1:
            return 0
        # Ohne gegnerische Dame ist ein offener König nur halb so gefährlich.
        enemy_queen = self.counts[QUEEN | ((1 - color) << 3)]
        b = self.b
        pawn = PAWN | (color << 3)
        fwd = 16 if color == WHITE else -16
        f = k & 7
        pen = 0
        for ff in (f - 1, f, f + 1):
            if ff < 0 or ff > 7:
                continue
            s1 = (k - f + ff) + fwd
            if b[s1] == pawn:
                pen += SHIELD_NEAR
                continue
            s2 = s1 + fwd
            if not s2 & 0x88 and b[s2] == pawn:
                pen += SHIELD_FAR
                continue
            pen += SHIELD_NONE
            if wf[ff] == 0 and bf[ff] == 0:
                pen += SHIELD_OPEN
        return pen if enemy_queen else pen // 2

    def _pawn_eval(self):
        b = self.b
        wf = [0] * 8
        bf = [0] * 8
        wmin = [8] * 8          # niedrigste Reihe eines weißen Bauern je Linie
        bmax = [-1] * 8         # höchste Reihe eines schwarzen Bauern je Linie
        wps = []
        bps = []
        for sq in SQUARES:
            p = b[sq]
            if p == 1:
                f = sq & 7
                wf[f] += 1
                if (sq >> 4) < wmin[f]:
                    wmin[f] = sq >> 4
                wps.append(sq)
            elif p == 9:
                f = sq & 7
                bf[f] += 1
                if (sq >> 4) > bmax[f]:
                    bmax[f] = sq >> 4
                bps.append(sq)
        mg = 0
        eg = 0
        for f in range(8):
            if wf[f] > 1:
                mg -= DOUBLED[0] * (wf[f] - 1)
                eg -= DOUBLED[1] * (wf[f] - 1)
            if bf[f] > 1:
                mg += DOUBLED[0] * (bf[f] - 1)
                eg += DOUBLED[1] * (bf[f] - 1)
        for sq in wps:
            f, r = sq & 7, sq >> 4
            if (f == 0 or wf[f - 1] == 0) and (f == 7 or wf[f + 1] == 0):
                mg -= ISOLATED[0]
                eg -= ISOLATED[1]
            passed = True
            for ff in (f - 1, f, f + 1):
                if 0 <= ff <= 7 and bmax[ff] > r:
                    passed = False
                    break
            if passed:
                mg += PASSED_MG[r]
                eg += PASSED_EG[r]
        for sq in bps:
            f, r = sq & 7, sq >> 4
            if (f == 0 or bf[f - 1] == 0) and (f == 7 or bf[f + 1] == 0):
                mg += ISOLATED[0]
                eg += ISOLATED[1]
            passed = True
            for ff in (f - 1, f, f + 1):
                if 0 <= ff <= 7 and wmin[ff] < r:
                    passed = False
                    break
            if passed:
                mg -= PASSED_MG[7 - r]
                eg -= PASSED_EG[7 - r]
        entry = (mg, eg, wf, bf)
        if len(_PAWN_CACHE) > 50000:
            _PAWN_CACHE.clear()
        _PAWN_CACHE[self.phash] = entry
        return entry

    # --------------------------------------------------------- Notation
    def fen(self):
        rows = []
        for r in range(7, -1, -1):
            row = ""
            empty = 0
            for f in range(8):
                p = self.b[r * 16 + f]
                if not p:
                    empty += 1
                else:
                    if empty:
                        row += str(empty)
                        empty = 0
                    row += piece_char(p)
            if empty:
                row += str(empty)
            rows.append(row)
        cr = ""
        for i, ch in enumerate("KQkq"):
            rsq = self.castle[i]
            if rsq < 0:
                continue
            color = i // 2
            if self._outermost_rook(color, i % 2 == 0) == rsq:
                cr += ch
            else:
                letter = FILES[rsq & 7]
                cr += letter.upper() if color == WHITE else letter
        return "%s %s %s %s %d %d" % (
            "/".join(rows), "w" if self.side == WHITE else "b", cr or "-",
            sq_name(self.ep) if self.ep >= 0 else "-", self.half, self.full)

    def _outermost_rook(self, color, kingside):
        base = 0 if color == WHITE else 112
        k = self.kings[color]
        rook = ROOK | (color << 3)
        files = range(7, (k & 7), -1) if kingside else range(0, (k & 7))
        for f in files:
            if self.b[base + f] == rook:
                return base + f
        return -1

    def uci(self, m):
        frm = m & 255
        to = (m >> 8) & 255
        if m >> 20 == M_CASTLE and not self.chess960:
            to = (frm & 0x70) + (6 if to > frm else 2)
        promo = (m >> 16) & 7
        return sq_name(frm) + sq_name(to) + (LETTERS[promo].lower() if promo else "")

    def parse_uci(self, text):
        """UCI-Zug (e2e4, e7e8q, e1g1 oder e1h1 für Rochaden) -> Zug oder 0."""
        text = text.strip().lower()
        for m in self.legal_moves():
            u = self.uci(m)
            if u == text:
                return m
            if m >> 20 == M_CASTLE:
                frm = m & 255
                to = (m >> 8) & 255
                kt = (frm & 0x70) + (6 if to > frm else 2)
                if text in (sq_name(frm) + sq_name(to), sq_name(frm) + sq_name(kt)):
                    return m
        return 0

    def san(self, m, legal=None):
        """Standard Algebraic Notation (inkl. +/#)."""
        b = self.b
        frm = m & 255
        to = (m >> 8) & 255
        flag = m >> 20
        p = b[frm]
        typ = p & 7
        if flag == M_CASTLE:
            s = "O-O" if to > frm else "O-O-O"
        else:
            capture = flag == M_EP or b[to] != 0
            promo = (m >> 16) & 7
            if typ == PAWN:
                s = (FILES[frm & 7] + "x" if capture else "") + sq_name(to)
                if promo:
                    s += "=" + LETTERS[promo]
            else:
                if legal is None:
                    legal = self.legal_moves()
                same_file = same_rank = False
                ambiguous = False
                for o in legal:
                    of = o & 255
                    if of == frm or (o >> 8) & 255 != to or o >> 20 == M_CASTLE:
                        continue
                    if b[of] != p:
                        continue
                    ambiguous = True
                    if (of & 7) == (frm & 7):
                        same_file = True
                    if (of >> 4) == (frm >> 4):
                        same_rank = True
                dis = ""
                if ambiguous:
                    if not same_file:
                        dis = FILES[frm & 7]
                    elif not same_rank:
                        dis = str((frm >> 4) + 1)
                    else:
                        dis = sq_name(frm)
                s = LETTERS[typ] + dis + ("x" if capture else "") + sq_name(to)
        self.make(m)
        if self.in_check():
            s += "+" if self.has_legal_move() else "#"
        self.unmake()
        return s

    def parse_san(self, text):
        """SAN-Text -> Zug (oder 0, wenn nicht legal/eindeutig)."""
        t = text.strip().replace("0-0-0", "O-O-O").replace("0-0", "O-O")
        t = t.rstrip("+#!?")
        legal = self.legal_moves()
        for m in legal:
            if self.san(m, legal).rstrip("+#") == t:
                return m
        return 0


# ============================================================ Startstellungen
START_FEN = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
_KRN = ((0, 1), (0, 2), (0, 3), (0, 4), (1, 2), (1, 3), (1, 4), (2, 3), (2, 4),
        (3, 4))


def chess960_backrank(n):
    """Grundreihe (a..h) der Chess960-Stellung Nr. n (0..959, 518 = normal)."""
    n %= 960
    rank = [""] * 8
    n, b1 = divmod(n, 4)
    rank[b1 * 2 + 1] = "B"           # Läufer auf hellem Feld (b, d, f, h)
    n, b2 = divmod(n, 4)
    rank[b2 * 2] = "B"               # Läufer auf dunklem Feld (a, c, e, g)
    n, q = divmod(n, 6)
    empty = [i for i in range(8) if not rank[i]]
    rank[empty[q]] = "Q"
    empty = [i for i in range(8) if not rank[i]]
    k1, k2 = _KRN[n]
    rank[empty[k1]] = "N"
    rank[empty[k2]] = "N"
    empty = [i for i in range(8) if not rank[i]]
    for i, ch in zip(empty, "RKR"):
        rank[i] = ch
    return "".join(rank)


def chess960_fen(n):
    back = chess960_backrank(n)
    return "%s/pppppppp/8/8/8/8/PPPPPPPP/%s w KQkq - 0 1" % (back.lower(), back)


def from_fen(fen, chess960=None):
    """Baut eine Position aus einer FEN (X-FEN/Shredder-FEN erlaubt)."""
    parts = fen.split()
    pos = Position()
    rows = parts[0].split("/")
    if len(rows) != 8:
        raise ValueError("FEN: 8 Reihen erwartet")
    for i, row in enumerate(rows):
        r = 7 - i
        f = 0
        for ch in row:
            if ch.isdigit():
                f += int(ch)
            else:
                typ = LETTERS.index(ch.upper())
                if typ <= 0 or f > 7:
                    raise ValueError("FEN: ungültige Figur")
                pos.b[r * 16 + f] = typ | (8 if ch.islower() else 0)
                f += 1
        if f != 8:
            raise ValueError("FEN: Reihe hat nicht 8 Felder")
    pos.side = BLACK if len(parts) > 1 and parts[1] == "b" else WHITE
    for sq in SQUARES:
        p = pos.b[sq]
        if p & 7 == KING:
            pos.kings[p >> 3] = sq
    if pos.kings[0] < 0 or pos.kings[1] < 0:
        raise ValueError("FEN: König fehlt")
    castle = [-1, -1, -1, -1]
    std = True
    for ch in (parts[2] if len(parts) > 2 else "-"):
        if ch == "-":
            continue
        color = WHITE if ch.isupper() else BLACK
        base = 0 if color == WHITE else 112
        kf = pos.kings[color] & 7
        if ch.upper() == "K":
            rsq = pos._outermost_rook(color, True)
            if rsq >= 0:
                castle[color * 2] = rsq
        elif ch.upper() == "Q":
            rsq = pos._outermost_rook(color, False)
            if rsq >= 0:
                castle[color * 2 + 1] = rsq
        elif ch.lower() in FILES:
            f = FILES.index(ch.lower())
            if pos.b[base + f] == ROOK | (color << 3):
                castle[color * 2 + (0 if f > kf else 1)] = base + f
            std = False
    for i in range(4):
        color = i // 2
        base = 0 if color == WHITE else 112
        if castle[i] >= 0 and (pos.kings[color] >> 4) != (base >> 4):
            castle[i] = -1
        if castle[i] >= 0 and (pos.kings[color] != base + 4
                               or castle[i] not in (base, base + 7)):
            std = False
    pos.castle = tuple(castle)
    if len(parts) > 3 and parts[3] != "-":
        ep = sq_parse(parts[3])
        # Nur übernehmen, wenn wirklich ein Bauer en passant schlagen könnte.
        us = pos.side
        cap_pawn = PAWN | (us << 3)
        ok = False
        for d in ((-15, -17) if us == WHITE else (15, 17)):
            s = ep + d
            if not s & 0x88 and pos.b[s] == cap_pawn:
                ok = True
        pos.ep = ep if ok else -1
    pos.half = int(parts[4]) if len(parts) > 4 else 0
    pos.full = int(parts[5]) if len(parts) > 5 else 1
    pos.chess960 = (not std) if chess960 is None else chess960
    pos._recompute()
    return pos


def perft(pos, depth):
    if depth == 0:
        return 1
    us = pos.side
    n = 0
    for m in pos.gen_moves():
        pos.make(m)
        if not pos.attacked(pos.kings[us], 1 - us):
            n += 1 if depth == 1 else perft(pos, depth - 1)
        pos.unmake()
    return n


# ================================================================== PGN
def pgn_result_code(winner):
    """winner: 0 = Weiß, 1 = Schwarz, None = Remis, "*" = läuft."""
    if winner == "*":
        return "*"
    if winner is None:
        return "1/2-1/2"
    return "1-0" if winner == WHITE else "0-1"


def pgn_text(tags, sans, result, start_side=WHITE, start_full=1):
    """PGN aus Tags [(name, wert)], SAN-Liste und Ergebnis ("1-0" …)."""
    lines = ['[%s "%s"]' % (k, str(v).replace("\\", "\\\\").replace('"', '\\"'))
             for k, v in tags]
    tokens = []
    full = start_full
    side = start_side
    for i, s in enumerate(sans):
        if side == WHITE:
            tokens.append("%d." % full)
        elif i == 0:
            tokens.append("%d..." % full)
        tokens.append(s)
        if side == BLACK:
            full += 1
        side = 1 - side
    tokens.append(result)
    body = []
    line = ""
    for tok in tokens:
        if line and len(line) + 1 + len(tok) > 79:
            body.append(line)
            line = tok
        else:
            line = (line + " " + tok) if line else tok
    if line:
        body.append(line)
    return "\n".join(lines) + "\n\n" + "\n".join(body) + "\n"


# =========================================================== Eröffnungsbuch
# Kleine Sammlung gängiger Hauptvarianten (UCI) - sorgt für Abwechslung und
# vernünftige Eröffnungen. Wird nur aus der normalen Grundstellung benutzt.
BOOK_LINES = (
    "e2e4 e7e5 g1f3 b8c6 f1b5 a7a6 b5a4 g8f6 e1g1 f8e7 f1e1 b7b5 a4b3 d7d6",
    "e2e4 e7e5 g1f3 b8c6 f1b5 g8f6 e1g1 f6e4 d2d4 e4d6 b5c6 d7c6 d4e5 d6f5",
    "e2e4 e7e5 g1f3 b8c6 f1c4 f8c5 c2c3 g8f6 d2d3 d7d6 e1g1 e8g8",
    "e2e4 e7e5 g1f3 b8c6 f1c4 g8f6 d2d3 f8e7 e1g1 e8g8 f1e1 d7d6",
    "e2e4 e7e5 g1f3 b8c6 d2d4 e5d4 f3d4 g8f6 d4c6 b7c6 e4e5 d8e7",
    "e2e4 e7e5 g1f3 g8f6 f3e5 d7d6 e5f3 f6e4 d2d4 d6d5 f1d3 b8c6",
    "e2e4 e7e5 b1c3 g8f6 g1f3 b8c6 f1b5 f8b4 e1g1 e8g8",
    "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 a7a6 c1e3 e7e5 d4b3 c8e6",
    "e2e4 c7c5 g1f3 b8c6 d2d4 c5d4 f3d4 g8f6 b1c3 e7e5 d4b5 d7d6",
    "e2e4 c7c5 g1f3 e7e6 d2d4 c5d4 f3d4 a7a6 f1d3 g8f6 e1g1 d8c7",
    "e2e4 c7c5 g1f3 d7d6 f1b5 c8d7 b5d7 d8d7 e1g1 b8c6 c2c3 g8f6",
    "e2e4 c7c5 b1c3 b8c6 g2g3 g7g6 f1g2 f8g7 d2d3 d7d6",
    "e2e4 c7c5 c2c3 g8f6 e4e5 f6d5 d2d4 c5d4 g1f3 b8c6",
    "e2e4 e7e6 d2d4 d7d5 b1c3 g8f6 c1g5 f8e7 e4e5 f6d7 g5e7 d8e7",
    "e2e4 e7e6 d2d4 d7d5 e4e5 c7c5 c2c3 b8c6 g1f3 d8b6 a2a3 c5c4",
    "e2e4 e7e6 d2d4 d7d5 b1d2 g8f6 e4e5 f6d7 f1d3 c7c5 c2c3 b8c6",
    "e2e4 c7c6 d2d4 d7d5 b1c3 d5e4 c3e4 c8f5 e4g3 f5g6 h2h4 h7h6",
    "e2e4 c7c6 d2d4 d7d5 e4e5 c8f5 g1f3 e7e6 f1e2 c6c5 e1g1 b8c6",
    "e2e4 d7d6 d2d4 g8f6 b1c3 g7g6 g1f3 f8g7 f1e2 e8g8 e1g1 c7c6",
    "e2e4 d7d5 e4d5 d8d5 b1c3 d5a5 d2d4 g8f6 g1f3 c8f5 f1c4 e7e6",
    "e2e4 g8f6 e4e5 f6d5 d2d4 d7d6 g1f3 c8g4 f1e2 e7e6 e1g1 f8e7",
    "e2e4 g7g6 d2d4 f8g7 b1c3 d7d6 g1f3 g8f6 f1e2 e8g8 e1g1 c7c6",
    "d2d4 d7d5 c2c4 e7e6 b1c3 g8f6 c1g5 f8e7 e2e3 e8g8 g1f3 b8d7",
    "d2d4 d7d5 c2c4 c7c6 g1f3 g8f6 b1c3 d5c4 a2a4 c8f5 e2e3 e7e6",
    "d2d4 d7d5 c2c4 d5c4 g1f3 g8f6 e2e3 e7e6 f1c4 c7c5 e1g1 a7a6",
    "d2d4 d7d5 c2c4 e7e6 g1f3 g8f6 g2g3 f8e7 f1g2 e8g8 e1g1 d5c4",
    "d2d4 g8f6 c2c4 g7g6 b1c3 f8g7 e2e4 d7d6 g1f3 e8g8 f1e2 e7e5",
    "d2d4 g8f6 c2c4 e7e6 b1c3 f8b4 e2e3 e8g8 f1d3 d7d5 g1f3 c7c5",
    "d2d4 g8f6 c2c4 e7e6 g1f3 b7b6 g2g3 c8a6 b2b3 f8b4 c1d2 b4e7",
    "d2d4 g8f6 c2c4 g7g6 b1c3 d7d5 c4d5 f6d5 e2e4 d5c3 b2c3 f8g7",
    "d2d4 g8f6 c2c4 c7c5 d4d5 e7e6 b1c3 e6d5 c4d5 d7d6 e2e4 g7g6",
    "d2d4 g8f6 c2c4 e7e6 g2g3 d7d5 f1g2 f8e7 g1f3 e8g8 e1g1 d5c4",
    "d2d4 d7d5 g1f3 g8f6 c1f4 e7e6 e2e3 c7c5 c2c3 b8c6 b1d2 f8d6",
    "d2d4 g8f6 g1f3 e7e6 c1g5 c7c5 e2e3 b7b6 b1d2 c8b7",
    "d2d4 f7f5 g2g3 g8f6 f1g2 g7g6 g1f3 f8g7 e1g1 e8g8 c2c4 d7d6",
    "c2c4 e7e5 b1c3 g8f6 g1f3 b8c6 g2g3 d7d5 c4d5 f6d5 f1g2 d5b6",
    "c2c4 g8f6 b1c3 e7e6 g1f3 d7d5 d2d4 f8e7 c1f4 e8g8 e2e3 c7c5",
    "c2c4 c7c5 g1f3 b8c6 b1c3 g7g6 g2g3 f8g7 f1g2 g8f6 e1g1 e8g8",
    "g1f3 d7d5 g2g3 g8f6 f1g2 e7e6 e1g1 f8e7 d2d3 e8g8 b1d2 c7c5",
    "g1f3 g8f6 c2c4 g7g6 b1c3 f8g7 e2e4 d7d6 d2d4 e8g8 f1e2 e7e5",
)


def book_moves(history_uci):
    """Mögliche Buchzüge nach der Zugfolge 'history_uci' (Liste von UCI)."""
    n = len(history_uci)
    out = []
    for line in BOOK_LINES:
        seq = line.split()
        if len(seq) > n and seq[:n] == list(history_uci):
            out.append(seq[n])
    return out


# ================================================================== KI
# Stufen (Anfänger .. Meister): Suchtiefe, Rechenzeit (s), Knotenlimit,
# Ruhesuche, Zufallszug-Wahrscheinlichkeit, Streuung (cp: gleichwertig
# behandelte Züge an der Wurzel), Eröffnungsbuch.
LEVELS = (
    {"depth": 1, "time": 0.05, "nodes": 2500, "quiesce": False,
     "random": 1.0, "spread": 0, "book": False},
    {"depth": 1, "time": 0.15, "nodes": 2500, "quiesce": False,
     "random": 0.3, "spread": 120, "book": False},
    {"depth": 2, "time": 0.25, "nodes": 6000, "quiesce": True,
     "random": 0.1, "spread": 50, "book": True},
    {"depth": 3, "time": 0.4, "nodes": 15000, "quiesce": True,
     "random": 0.03, "spread": 20, "book": True},
    {"depth": 5, "time": 0.8, "nodes": 40000, "quiesce": True,
     "random": 0.0, "spread": 0, "book": True},
    {"depth": 32, "time": 1.4, "nodes": 120000, "quiesce": True,
     "random": 0.0, "spread": 0, "book": True},
)
HINT_LEVEL = {"depth": 32, "time": 0.9, "nodes": 60000, "quiesce": True,
              "random": 0.0, "spread": 0, "book": False}
TT_MAX = 200000


class Search:
    """Fortsetzbare Suche. Benutzung:

        s = Search(pos, LEVELS[5])
        gen = s.run()
        s.slice_end = time.perf_counter() + 0.007
        next(gen)          # ... jedes Frame, bis StopIteration
        s.best_move        # bester bisher gefundener Zug (0 = keiner)

    Die Suche arbeitet auf einer Kopie der Stellung. Wird sie abgebrochen
    (Generator einfach verwerfen oder ``gen.close()``), bleibt ``best_move``
    der beste Zug der letzten fertigen bzw. teilweise durchsuchten Tiefe.
    """

    def __init__(self, pos, level, tt=None, max_depth=None, node_limit=None,
                 rng=None):
        self.pos = pos.copy()
        self.level = level
        self.max_depth = max_depth or level["depth"]
        self.node_limit = node_limit if node_limit is not None else level["nodes"]
        self.quiesce = level["quiesce"]
        self.spread = level["spread"]
        self.tt = tt if tt is not None else {}
        self.rng = rng or random
        self.nodes = 0
        self.slice_end = float("inf")
        self.stop = False
        self.done = False
        self.best_move = 0
        self.best_score = 0
        self.depth_done = 0
        self.root_scores = {}
        self.killers = [[0, 0] for _ in range(MAX_PLY + 2)]
        self.history = [0] * (16 * 128)
        self.check_every = 63

    # ----------------------------------------------------------- Ablauf
    def run(self):
        pos = self.pos
        moves = pos.legal_moves()
        if not moves:
            self.done = True
            return
        self.best_move = moves[0]
        if len(moves) == 1 and not self.spread:
            self.done = True
            return
        # Wurzelzüge vorsortieren: Schlagzüge/Umwandlungen zuerst
        scores = [self._order_score(m, 0, 0) for m in moves]
        idx = sorted(range(len(moves)), key=scores.__getitem__, reverse=True)
        self.root_moves = [moves[i] for i in idx]
        for depth in range(1, self.max_depth + 1):
            score = yield from self._root(depth)
            if self.stop:
                break
            self.depth_done = depth
            self.best_score = score
            if abs(score) >= MATE - MAX_PLY and depth >= 2 and not self.spread:
                break
        if self.spread and self.root_scores:
            best = max(self.root_scores.values())
            good = [m for m in self.root_moves
                    if self.root_scores.get(m, -INF) >= best - self.spread]
            if good:
                self.best_move = self.rng.choice(good)
        self.done = True

    def _root(self, depth):
        pos = self.pos
        exact = self.spread > 0
        alpha = -INF
        beta = INF
        best = -INF
        best_move = 0
        scored = {}
        for i, m in enumerate(self.root_moves):
            pos.make(m)
            if i == 0 or exact:
                score = -(yield from self._negamax(depth - 1, -beta, -alpha if not exact else INF, 1, True))
            else:
                score = -(yield from self._negamax(depth - 1, -alpha - 1, -alpha, 1, True))
                if score > alpha and not self.stop:
                    score = -(yield from self._negamax(depth - 1, -beta, -alpha, 1, True))
            pos.unmake()
            if self.stop:
                break
            scored[m] = score
            if score > best:
                best = score
                best_move = m
                if not exact:
                    self.best_move = m
                    self.best_score = score
                if score > alpha and not exact:
                    alpha = score
        if not self.stop or exact:
            for m, s in scored.items():
                self.root_scores[m] = s
        if best_move:
            # Bester Zug nach vorn, Rest nach Wertung dieser Tiefe
            order = sorted(range(len(self.root_moves)),
                           key=lambda i: scored.get(self.root_moves[i], -INF - 1),
                           reverse=True)
            self.root_moves = [self.root_moves[i] for i in order]
            if exact and not self.stop:
                self.best_move = best_move
                self.best_score = best
        return best

    # ----------------------------------------------------------- Sortierung
    def _order_score(self, m, tt_move, ply):
        if m == tt_move:
            return 4000000
        b = self.pos.b
        frm = m & 255
        to = (m >> 8) & 255
        flag = m >> 20
        promo = (m >> 16) & 7
        if flag == M_EP:
            return 2000000 + 10 - 1
        if flag != M_CASTLE:
            cap = b[to]
            if cap:
                return 2000000 + ORDER_VALUE[cap & 7] * 10 - ORDER_VALUE[b[frm] & 7] + promo
        if promo:
            return 1900000 + promo
        k = self.killers[ply]
        if m == k[0]:
            return 1800000
        if m == k[1]:
            return 1700000
        return self.history[b[frm] * 128 + to]

    # ----------------------------------------------------------- Negamax
    def _negamax(self, depth, alpha, beta, ply, can_null):
        self.nodes += 1
        if self.nodes >= self.node_limit:
            self.stop = True
        elif not (self.nodes & self.check_every) and time.perf_counter() > self.slice_end:
            yield
        if self.stop:
            return 0
        pos = self.pos
        if pos.half >= 100 or pos.repetitions() >= 2 or pos.insufficient_material():
            return 0
        # Matt-Distanz-Schnitt
        if alpha < -MATE + ply:
            alpha = -MATE + ply
        if beta > MATE - ply - 1:
            beta = MATE - ply - 1
        if alpha >= beta:
            return alpha
        us = pos.side
        them = 1 - us
        in_check = pos.attacked(pos.kings[us], them)
        if in_check:
            depth += 1
        if depth <= 0 or ply >= MAX_PLY:
            if self.quiesce and ply < MAX_PLY:
                return (yield from self._quiesce(alpha, beta, ply))
            return pos.evaluate()
        pv = beta - alpha > 1
        key = pos.hash
        entry = self.tt.get(key)
        tt_move = 0
        if entry is not None:
            e_depth, e_flag, e_score, tt_move = entry
            if not pv and e_depth >= depth:
                if e_score > MATE - MAX_PLY:
                    e_score -= ply
                elif e_score < -MATE + MAX_PLY:
                    e_score += ply
                if e_flag == 0 or (e_flag == 1 and e_score >= beta) or \
                        (e_flag == 2 and e_score <= alpha):
                    return e_score
        static = 0
        if not pv and not in_check:
            static = pos.evaluate()
            # Statischer Null-Zug-Schnitt (Reverse Futility)
            if depth <= 3 and abs(beta) < MATE - MAX_PLY and \
                    static - 90 * depth >= beta:
                return static
            # Null-Zug
            if can_null and depth >= 3 and static >= beta and \
                    pos.non_pawn_material(us) > 0:
                r = 2 if depth < 6 else 3
                pos.make_null()
                score = -(yield from self._negamax(depth - 1 - r, -beta, -beta + 1,
                                                   ply + 1, False))
                pos.unmake_null()
                if self.stop:
                    return 0
                if score >= beta:
                    return beta if score >= MATE - MAX_PLY else score
        moves = pos.gen_moves()
        scores = [self._order_score(m, tt_move, ply) for m in moves]
        order = sorted(range(len(moves)), key=scores.__getitem__, reverse=True)
        b = pos.b
        best = -INF
        best_move = 0
        legal = 0
        orig_alpha = alpha
        futile = (not pv and not in_check and depth <= 2 and
                  abs(alpha) < MATE - MAX_PLY and static + 120 * depth <= alpha)
        for i in order:
            m = moves[i]
            quiet = scores[i] < 1700000
            if futile and quiet and legal > 0:
                continue
            pos.make(m)
            if pos.attacked(pos.kings[us], them):
                pos.unmake()
                continue
            legal += 1
            if legal == 1:
                score = -(yield from self._negamax(depth - 1, -beta, -alpha, ply + 1, True))
            else:
                red = 0
                if depth >= 3 and quiet and legal > 3 and not in_check:
                    red = 1 if legal < 10 or depth < 6 else 2
                score = -(yield from self._negamax(depth - 1 - red, -alpha - 1, -alpha,
                                                   ply + 1, True))
                if score > alpha and not self.stop and (red or score < beta):
                    score = -(yield from self._negamax(depth - 1, -beta, -alpha,
                                                       ply + 1, True))
            pos.unmake()
            if self.stop:
                return 0
            if score > best:
                best = score
                best_move = m
                if score > alpha:
                    alpha = score
                    if score >= beta:
                        if quiet:
                            k = self.killers[ply]
                            if k[0] != m:
                                k[1] = k[0]
                                k[0] = m
                            hi = b[m & 255] * 128 + ((m >> 8) & 255)
                            self.history[hi] += depth * depth
                            if self.history[hi] > 1000000:
                                self.history = [v // 2 for v in self.history]
                        break
        if legal == 0:
            return -MATE + ply if in_check else 0
        flag = 1 if best >= beta else (0 if best > orig_alpha else 2)
        store = best
        if store > MATE - MAX_PLY:
            store += ply
        elif store < -MATE + MAX_PLY:
            store -= ply
        if len(self.tt) >= TT_MAX:
            self.tt.clear()
        self.tt[key] = (depth, flag, store, best_move)
        return best

    def _quiesce(self, alpha, beta, ply):
        self.nodes += 1
        if self.nodes >= self.node_limit:
            self.stop = True
        elif not (self.nodes & self.check_every) and time.perf_counter() > self.slice_end:
            yield
        if self.stop:
            return 0
        pos = self.pos
        stand = pos.evaluate()
        if ply >= MAX_PLY:
            return stand
        if stand >= beta:
            return stand
        if stand > alpha:
            alpha = stand
        us = pos.side
        them = 1 - us
        b = pos.b
        moves = pos.gen_moves(True)
        scores = [self._order_score(m, 0, ply) for m in moves]
        order = sorted(range(len(moves)), key=scores.__getitem__, reverse=True)
        for i in order:
            m = moves[i]
            to = (m >> 8) & 255
            if not (m >> 16) & 7:
                cap = b[to]
                gain = MG_VALUE[cap & 7] if cap else MG_VALUE[PAWN]
                if stand + gain + 200 < alpha:
                    continue            # Delta-Schnitt: selbst der Schlag reicht nicht
            pos.make(m)
            if pos.attacked(pos.kings[us], them):
                pos.unmake()
                continue
            score = -(yield from self._quiesce(-beta, -alpha, ply + 1))
            pos.unmake()
            if self.stop:
                return 0
            if score >= beta:
                return score
            if score > alpha:
                alpha = score
        return alpha


def run_to_end(search):
    """Rechnet eine Suche ohne Abgabe komplett durch (Tests/Werkzeuge)."""
    for _ in search.run():
        pass
    return search.best_move


def pick_weak_move(pos, rng=None):
    """Anfänger-Stufe: zufälliger Zug, Schlagzüge bevorzugt."""
    rng = rng or random
    moves = pos.legal_moves()
    if not moves:
        return 0
    caps = [m for m in moves if pos.is_capture(m)]
    if caps and rng.random() < 0.6:
        return rng.choice(caps)
    return rng.choice(moves)


# ============================================================ Matt-Suche
def gives_mate(pos, m):
    """True, wenn Zug m sofort mattsetzt."""
    pos.make(m)
    mate = pos.in_check() and not pos.has_legal_move()
    pos.unmake()
    return mate


def forced_mate(pos, n):
    """Kann die Seite am Zug in höchstens n Zügen erzwungen mattsetzen?"""
    us = pos.side
    moves = []
    for m in pos.gen_moves():
        pos.make(m)
        if pos.attacked(pos.kings[us], 1 - us):
            pos.unmake()
            continue
        check = pos.in_check()
        pos.unmake()
        if n == 1 and not check:
            continue
        moves.append((0 if check else (1 if pos.is_capture(m) else 2), m))
    moves.sort(key=lambda x: x[0])
    for _, m in moves:
        pos.make(m)
        replies = pos.legal_moves()
        if not replies:
            mate = pos.in_check()
            pos.unmake()
            if mate:
                return True
            continue
        if n == 1:
            pos.unmake()
            continue
        ok = True
        for r in replies:
            pos.make(r)
            sub = forced_mate(pos, n - 1)
            pos.unmake()
            if not sub:
                ok = False
                break
        pos.unmake()
        if ok:
            return True
    return False
