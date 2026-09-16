# -*- coding: utf-8 -*-
"""
chess_draw.py
=============
Das Aussehen von Schach (``games/chess.py``) - als Mixin ``ChessDraw``.

Teure Flächen werden gecacht, damit auch 1280×960 flüssig bleibt:
- das Brett samt Rahmen und Koordinaten (je Feldgröße und Blickrichtung),
- jede Figur als fertiges Sprite (Füllung, Innenlinien, Kontur, Schatten),
- Texte der Zugliste/Seitenleiste (kleiner Text-Cache),
- halbtransparente Markierungen (letzter Zug, Schach, Zielpunkte).

Animationen (gleitende Figuren, ausblendende geschlagene Figuren, Brett
drehen) laufen über die Uhrzeit in ``draw()`` - so spielen sie auch nach
Partieende zu Ende, wenn ``update()`` nicht mehr aufgerufen wird.
"""

import math

import pygame

import ui
from i18n import t

from . import chess_engine as ce

# ------------------------------------------------- Brett-Identitätsfarben
# Generische UI-Farben (Hintergrund, Panels, Text) kommen zur Laufzeit aus
# der dynamischen ui-Palette; hier bleiben nur die Brett-/Figurenfarben.
COL_LIGHT = (236, 222, 196)      # helle Felder
COL_DARK = (160, 122, 90)        # dunkle Felder
COL_PLATE = (44, 36, 31)         # Brettrahmen
COL_COORD = (214, 198, 172)      # Koordinaten auf dem Rahmen
COL_SEL = (246, 214, 92)
COL_MOVE = (60, 150, 90)
COL_LAST = (232, 206, 80)
COL_CHECK = (232, 64, 64)
COL_HINT = (80, 170, 255)
COL_WHITE = (248, 246, 240)      # weiße Figuren
COL_BLACK = (40, 36, 44)         # schwarze Figuren
COL_OUTLINE = (22, 18, 20)

GLYPH_FILL = {1: "♟", 2: "♞", 3: "♝", 4: "♜", 5: "♛", 6: "♚"}
GLYPH_LINE = {1: "♙", 2: "♘", 3: "♗", 4: "♖", 5: "♕", 6: "♔"}
PIECE_FONT = "segoeuisymbol,dejavusans,arialunicodems,freeserif"

_THEME_LABEL = ("mate", "fork", "pin", "skewer", "discoveredAttack", "doubleCheck",
                "hangingPiece", "trappedPiece", "deflection", "attraction",
                "sacrifice", "promotion")


def _ease(k):
    return 1 - (1 - k) ** 3


class ChessDraw:
    """Zeichen-Methoden für ChessGame."""

    # ===================================================== Caches
    def _reset_caches(self):
        self._pieces = {}
        self._board_surf = None
        self._board_key = None
        self._texts = {}
        self._overlays = {}
        self._arrow = None

    def _text(self, fnt, text, color):
        key = (id(fnt), text, tuple(color))
        img = self._texts.get(key)
        if img is None:
            if len(self._texts) > 600:
                self._texts.clear()
            img = fnt.render(text, True, color)
            self._texts[key] = img
        return img

    def _fit(self, text, width, *fonts):
        """Erste Schrift, in die 'text' passt (sonst die kleinste)."""
        for f in fonts:
            if f.size(text)[0] <= width:
                return f
        return fonts[-1]

    def _clip_text(self, fnt, text, width):
        """Text notfalls mit … kürzen, damit er in 'width' passt."""
        if fnt.size(text)[0] <= width:
            return text
        while len(text) > 1 and fnt.size(text + "…")[0] > width:
            text = text[:-1]
        return text.rstrip() + "…"

    def _wrap(self, fnt, text, width):
        words = text.split()
        lines = []
        cur = ""
        for w in words:
            test = (cur + " " + w).strip()
            if fnt.size(test)[0] <= width or not cur:
                cur = test
            else:
                lines.append(cur)
                cur = w
        if cur:
            lines.append(cur)
        return lines

    def _overlay(self, size, color, alpha, radius=0):
        key = (size, color, alpha, radius)
        s = self._overlays.get(key)
        if s is None:
            s = pygame.Surface((size, size), pygame.SRCALPHA)
            pygame.draw.rect(s, (*color, alpha), (0, 0, size, size),
                             border_radius=radius)
            self._overlays[key] = s
        return s

    def _piece_sprite(self, p, size):
        key = (p, size)
        spr = self._pieces.get(key)
        if spr is not None:
            return spr
        fnt = pygame.font.SysFont(PIECE_FONT, max(8, int(size * 0.84)))
        typ = p & 7
        white = not (p >> 3)
        fill = fnt.render(GLYPH_FILL[typ], True, COL_WHITE if white else COL_BLACK)
        line = fnt.render(GLYPH_LINE[typ], True,
                          COL_OUTLINE if white else (118, 110, 122))
        edge = fnt.render(GLYPH_FILL[typ], True, COL_OUTLINE)
        shadow = fnt.render(GLYPH_FILL[typ], True, (0, 0, 0))
        spr = pygame.Surface((size, size), pygame.SRCALPHA)
        cx, cy = size // 2, size // 2 + max(1, size // 40)
        o = max(1, size // 36)
        sh = pygame.Surface(shadow.get_size(), pygame.SRCALPHA)
        sh.blit(shadow, (0, 0))
        sh.set_alpha(70)
        spr.blit(sh, sh.get_rect(center=(cx + o + 1, cy + o * 2)))
        for ox, oy in ((-o, 0), (o, 0), (0, -o), (0, o), (-o, -o), (o, o),
                       (-o, o), (o, -o)):
            spr.blit(edge, edge.get_rect(center=(cx + ox, cy + oy)))
        spr.blit(fill, fill.get_rect(center=(cx, cy)))
        spr.blit(line, line.get_rect(center=(cx, cy)))
        self._pieces[key] = spr
        return spr

    # ===================================================== Haupt-Zeichnen
    def draw(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height, stars=False, aurora=True)
        if self.state == "setup":
            self._draw_setup(s)
            return
        if self.state == "puz_menu":
            self._draw_puz_menu(s)
            return
        self._draw_board(s)
        if self.state == "puz":
            self._draw_puz_side(s)
        else:
            self._draw_side(s)
        if self.promo is not None:
            self._draw_promo(s)
        if self.dialog is not None:
            self._draw_dialog(s)
        if self.state == "over":
            self._draw_over(s)
        self._draw_toast(s)

    # ===================================================== Brett
    def _board_surface(self):
        key = (self.cell, self.frame, self.view_black, tuple(self.accent))
        if self._board_key == key and self._board_surf is not None:
            return self._board_surf
        pw = self.plate.w
        surf = pygame.Surface((pw, pw), pygame.SRCALPHA)
        rad = max(6, self.frame // 2)
        pygame.draw.rect(surf, COL_PLATE, (0, 0, pw, pw), border_radius=rad)
        pygame.draw.rect(surf, ui.mix(COL_PLATE, self.accent, 0.5), (0, 0, pw, pw),
                         1, border_radius=rad)
        f = self.frame
        c = self.cell
        for dr in range(8):
            for dc in range(8):
                sq = self._disp_to_sq(dr, dc)
                light = ((sq >> 4) + (sq & 7)) % 2 == 1
                pygame.draw.rect(surf, COL_LIGHT if light else COL_DARK,
                                 (f + dc * c, f + dr * c, c, c))
        # feine Innenkante ums Spielfeld
        pygame.draw.rect(surf, ui.mix(COL_PLATE, (0, 0, 0), 0.4),
                         (f - 1, f - 1, 8 * c + 2, 8 * c + 2), 1)
        cf = ui.font(max(9, int(f * 0.62)), bold=True)
        for i in range(8):
            file_ch = ce.FILES[7 - i] if self.view_black else ce.FILES[i]
            img = cf.render(file_ch, True, COL_COORD)
            surf.blit(img, img.get_rect(center=(f + i * c + c // 2, f + 8 * c + f // 2)))
            rank_ch = str(i + 1) if self.view_black else str(8 - i)
            img = cf.render(rank_ch, True, COL_COORD)
            surf.blit(img, img.get_rect(center=(f // 2, f + i * c + c // 2)))
        self._board_surf = surf
        self._board_key = key
        return surf

    def _draw_board(self, s):
        now = self._now()
        target = s
        flip_k = None
        if self.flip_anim is not None:
            k = (now - self.flip_anim) / 0.32
            if k >= 1:
                self.flip_anim = None
            else:
                flip_k = k
                target = pygame.Surface(self.plate.size, pygame.SRCALPHA)
        ox = self.plate.x if target is s else 0
        oy = self.plate.y if target is s else 0
        dx, dy = ox - self.plate.x, oy - self.plate.y

        def rect_of(sq):
            r = self._sq_rect(sq)
            return r.move(dx, dy)

        target.blit(self._board_surface(), (ox, oy))
        c = self.cell
        # letzter Zug
        if self.last_move:
            for sq in self.last_move:
                target.blit(self._overlay(c, COL_LAST, 92), rect_of(sq))
        # König im Schach: rotes Leuchten
        if self.check and self.state in ("play", "puz"):
            ks = self.pos.kings[self.pos.side]
            r = rect_of(ks)
            glow = self._check_glow(c)
            target.blit(glow, glow.get_rect(center=r.center))
        # Auswahl + Ziele
        human = self._human_can_move()
        if self.sel is not None and human:
            target.blit(self._overlay(c, COL_SEL, 110), rect_of(self.sel))
            b = self.pos.b
            for sq in self.targets:
                r = rect_of(sq)
                occupied = b[sq] != 0 or any((m >> 20) == ce.M_EP for m in self.targets[sq])
                target.blit(self._target_mark(c, occupied), r)
        # geschlagene Figuren blenden unter dem schlagenden Stein aus
        live_fades = []
        for fd in self.fades:
            k = (now - fd["t0"]) / 0.35
            if k < 1:
                live_fades.append(fd)
                spr = self._piece_sprite(fd["p"], c).copy()
                spr.set_alpha(int(255 * (1 - k)))
                size = int(c * (1 - 0.35 * k))
                if size > 4:
                    spr = pygame.transform.smoothscale(spr, (size, size))
                target.blit(spr, spr.get_rect(center=rect_of(fd["sq"]).center))
        self.fades = live_fades
        # Figuren (animierte Ziele auslassen)
        anim_live = []
        for a in self.anims:
            k = (now - a["t0"]) / 0.2
            if k < 1:
                anim_live.append((a, _ease(max(0.0, k))))
        hidden = {a["b"] for a, _ in anim_live}
        if self.drag is not None and self.drag.get("moved"):
            hidden.add(self.drag["sq"])
        b = self.pos.b
        for sq in ce.SQUARES:
            p = b[sq]
            if p and sq not in hidden:
                target.blit(self._piece_sprite(p, c), rect_of(sq))
        for a, k in anim_live:
            ra = rect_of(a["a"])
            rb = rect_of(a["b"])
            x = ra.x + (rb.x - ra.x) * k
            y = ra.y + (rb.y - ra.y) * k
            target.blit(self._piece_sprite(a["p"], c), (int(x), int(y)))
        if not anim_live:
            self.anims = []
        # Hinweis-Pfeil
        if self.hint_move and self.state == "play":
            self._draw_arrow(target, self.hint_move, dx, dy)
        # gezogene Figur
        if self.drag is not None and self.drag.get("moved") and human:
            p = b[self.drag["sq"]]
            if p:
                spr = self._piece_sprite(p, int(c * 1.12))
                mx, my = self.drag["pos"]
                over = self._sq_at((mx, my))
                if over is not None:
                    pygame.draw.rect(target, ui.mix(COL_SEL, (255, 255, 255), 0.3),
                                     rect_of(over), 2)
                target.blit(spr, spr.get_rect(center=(mx + dx, my + dy)))
        # Tastatur-Cursor
        if human and self.drag is None:
            dr, dc = self.cursor
            rect = pygame.Rect(self.bx + dc * c + dx, self.by + dr * c + dy, c, c)
            k = ui.pulse(3.0, 0.0, 1.0)
            col = ui.mix(self.accent, (255, 255, 255), 0.35 + 0.4 * k)
            pygame.draw.rect(target, col, rect.inflate(-2, -2), max(2, c // 22),
                             border_radius=3)
        if flip_k is not None:
            # Brett drehen: horizontal zusammen- und wieder aufklappen
            sx = abs(math.cos(flip_k * math.pi))
            w = max(2, int(self.plate.w * sx))
            img = pygame.transform.scale(target, (w, self.plate.h))
            s.blit(img, (self.plate.centerx - w // 2, self.plate.y))

    def _check_glow(self, c):
        key = ("check", c)
        g = self._overlays.get(key)
        if g is None:
            size = int(c * 1.3)
            g = pygame.Surface((size, size), pygame.SRCALPHA)
            steps = 10
            for i in range(steps, 0, -1):
                r = int(size / 2 * i / steps)
                a = int(150 * (1 - i / steps) + 30)
                pygame.draw.circle(g, (*COL_CHECK, a), (size // 2, size // 2), r)
            self._overlays[key] = g
        return g

    def _target_mark(self, c, occupied):
        key = ("target", c, occupied)
        g = self._overlays.get(key)
        if g is None:
            g = pygame.Surface((c, c), pygame.SRCALPHA)
            if occupied:
                pygame.draw.circle(g, (*COL_MOVE, 150), (c // 2, c // 2), c // 2 - 1,
                                   max(3, c // 11))
            else:
                pygame.draw.circle(g, (*COL_MOVE, 150), (c // 2, c // 2),
                                   max(4, c // 6))
            self._overlays[key] = g
        return g

    def _draw_arrow(self, target, m, dx, dy):
        key = (m, self.view_black, self.cell)
        if self._arrow is None or self._arrow[0] != key:
            bw = self.bw
            surf = pygame.Surface((bw, bw), pygame.SRCALPHA)
            frm, to = self._move_squares(m)
            ra = self._sq_rect(frm)
            rb = self._sq_rect(to)
            ax, ay = ra.centerx - self.bx, ra.centery - self.by
            bx, by = rb.centerx - self.bx, rb.centery - self.by
            ang = math.atan2(by - ay, bx - ax)
            width = max(5, self.cell // 6)
            head = max(12, int(self.cell * 0.42))
            ex = bx - math.cos(ang) * head * 0.9
            ey = by - math.sin(ang) * head * 0.9
            col = (*COL_HINT, 190)
            pygame.draw.line(surf, col, (ax, ay), (ex, ey), width)
            pygame.draw.circle(surf, col, (int(ax), int(ay)), width // 2 + 2)
            left = (bx - math.cos(ang - 0.45) * head, by - math.sin(ang - 0.45) * head)
            right = (bx - math.cos(ang + 0.45) * head, by - math.sin(ang + 0.45) * head)
            pygame.draw.polygon(surf, col, [(bx, by), left, right])
            self._arrow = (key, surf)
        target.blit(self._arrow[1], (self.bx + dx, self.by + dy))

    # ===================================================== Seitenleiste
    def _draw_side(self, s):
        top_color = 1 - self._bottom_color()
        self._draw_player(s, self.top_panel, top_color)
        self._draw_player(s, self.bot_panel, self._bottom_color())
        self._draw_status(s)
        self._draw_movelist(s, self.list_rect)
        self._draw_buttons(s)

    def _bottom_color(self):
        return ce.BLACK if self.view_black else ce.WHITE

    def _player_name(self, color):
        if self.mode == "single":
            if color == self.human_color:
                return t("chess.you")
            return "%s · %s" % (t("common.ai"), t("chess.diff." + ("lvl%d" % self.diff)))
        return t("chess.white") if color == ce.WHITE else t("chess.black")

    def _draw_player(self, s, rect, color):
        active = self.state == "play" and self.pos.side == color
        ui.draw_panel(s, rect, shadow=False,
                      accent_top=self.accent if active else None)
        pad = 8
        cy1 = rect.y + pad + self._small.get_height() // 2
        dot = max(5, self._small.get_height() // 3)
        pygame.draw.circle(s, COL_WHITE if color == ce.WHITE else COL_BLACK,
                           (rect.x + pad + dot, cy1), dot)
        pygame.draw.circle(s, ui.BORDER_LIGHT, (rect.x + pad + dot, cy1), dot, 1)
        clock_w = 0
        if self.clock is not None:
            txt = self._fmt_clock(self.clock[color])
            low = self.clock[color] < 10
            col = ui.RED if low else (ui.TEXT if active else ui.TEXT_DIM)
            img = self._text(self._clockf, txt, col)
            cr = img.get_rect(midright=(rect.right - pad, cy1))
            if active:
                bg = cr.inflate(10, 4)
                pygame.draw.rect(s, ui.mix(ui.PANEL_LIGHT, self.accent, 0.25), bg,
                                 border_radius=5)
            s.blit(img, cr)
            clock_w = cr.w + 14
        nx = rect.x + pad + dot * 2 + 6
        room = rect.right - pad - clock_w - nx
        name = self._player_name(color)
        if self.mode == "multi" and any(self.wins):
            # Siege in dieser Sitzung - nur, wenn der Platz reicht
            with_wins = "%s  ·  %d" % (name, self.wins[color])
            if self._small.size(with_wins)[0] <= room:
                name = with_wins
        name = self._clip_text(self._small, name, room)
        s.blit(self._text(self._small, name, ui.TEXT if active else ui.TEXT_DIM),
               (nx, cy1 - self._small.get_height() // 2))
        # geschlagene Figuren + Materialbilanz
        cy2 = rect.bottom - pad - self._tiny.get_height() // 2
        caps = sorted((p for i, p in enumerate(self.caps)
                       if p and (p >> 3) != color),
                      key=lambda p: -ce.ORDER_VALUE[p & 7])
        size = self._tiny.get_height() + 6
        step = max(6, int(size * 0.52))
        diff = self.pos.material(color) - self.pos.material(1 - color)
        dtxt = "+%d" % diff if diff > 0 else ""
        dimg = self._text(self._tiny, dtxt, ui.GREEN) if dtxt else None
        maxx = rect.right - pad - (dimg.get_width() + 6 if dimg else 0)
        x = rect.x + pad - 2
        for i, p in enumerate(caps):
            if i > 0 and (caps[i - 1] & 7) != (p & 7):
                x += step // 2
            if x + size > maxx:
                break
            s.blit(self._piece_sprite(p, size), (x, cy2 - size // 2))
            x += step
        if dimg:
            s.blit(dimg, dimg.get_rect(midleft=(min(x + size - step + 6, maxx + 6), cy2)))

    @staticmethod
    def _fmt_clock(sec):
        sec = max(0.0, sec)
        if sec < 10:
            return "0:%04.1f" % sec
        whole = int(math.ceil(sec))
        return "%d:%02d" % (whole // 60, whole % 60)

    def _status_text(self):
        """(Text, Farbe) der Statuszeile."""
        if self.state == "over":
            return t("chess.game_over"), ui.TEXT_DIM
        if self.promo is not None:
            return t("chess.promote"), self.accent
        if self.hint_job:
            return t("chess.hint_wait") + self._dots(), ui.TEXT_DIM
        if self.mode == "single":
            if self.pos.side != self.human_color:
                return t("chess.ai_thinks").rstrip(" ….") + self._dots(), ui.TEXT_DIM
            if self.check:
                return t("chess.check"), ui.RED
            return t("chess.your_turn"), self.accent
        who = t("chess.white") if self.pos.side == ce.WHITE else t("chess.black")
        txt = t("chess.turn", name=who)
        if self.check:
            return t("chess.check") + " " + txt, ui.RED
        return txt, self.accent

    def _dots(self):
        n = int(self._now() * 3) % 4
        return " " + "." * n

    def _draw_status(self, s):
        text, col = self._status_text()
        r = self.status_rect
        fnt = self._fit(text, r.w - 8, self._small, self._tiny)
        text = self._clip_text(fnt, text, r.w - 8)
        img = fnt.render(text, True, col)
        s.blit(img, img.get_rect(center=r.center))

    def _draw_movelist(self, s, r):
        if r.h < 20:
            return
        ui.draw_panel(s, r, shadow=False)
        fnt = self._listf
        rh = self.row_h
        pad = 6
        # Kopfzeile: "Züge" (+ Chess960-Nummer)
        head = t("chess.moves")
        if self.c960_n is not None:
            head += "  ·  Chess960 #%d" % self.c960_n
        hf = self._tiny
        head = self._clip_text(hf, head, r.w - 2 * pad)
        s.blit(self._text(hf, head, ui.TEXT_FAINT), (r.x + pad, r.y + 4))
        y0 = r.y + 6 + hf.get_height()
        visible = self._list_visible()
        total = self._list_rows()
        if self.list_follow:
            self.list_top = max(0, total - visible)
        top = max(0, min(self.list_top, max(0, total - visible)))
        num_w = fnt.size("%d." % (self.start_full + total + 90))[0] + 4
        col_w = (r.w - 2 * pad - num_w) // 2
        last = len(self.sans) - 1
        shift = self.start_side          # 1 = Stellung beginnt mit Schwarz
        prev_clip = s.get_clip()
        s.set_clip(r.inflate(-2, -2))
        for row in range(top, min(total, top + visible)):
            y = y0 + (row - top) * rh
            if row % 2 == 1:
                pygame.draw.rect(s, ui.mix(ui.PANEL, ui.PANEL_LIGHT, 0.5),
                                 (r.x + 3, y - 1, r.w - 6, rh), border_radius=3)
            s.blit(self._text(fnt, "%d." % (self.start_full + row), ui.TEXT_FAINT),
                   (r.x + pad, y))
            for ci in (0, 1):
                i = row * 2 + ci - shift
                x = r.x + pad + num_w + ci * col_w
                if i < 0:
                    s.blit(self._text(fnt, "…", ui.TEXT_FAINT), (x, y))
                    continue
                if i > last:
                    break
                cur = i == last
                if cur:
                    pygame.draw.rect(s, ui.mix(ui.PANEL_LIGHT, self.accent, 0.35),
                                     (x - 3, y - 1, col_w - 2, rh), border_radius=4)
                txt = self._clip_text(fnt, self.sans[i], col_w - 6)
                s.blit(self._text(fnt, txt, ui.TEXT if cur else ui.TEXT_DIM), (x, y))
        s.set_clip(prev_clip)
        # Scrollbalken
        if total > visible:
            track = pygame.Rect(r.right - 5, y0, 3, visible * rh)
            pygame.draw.rect(s, ui.mix(ui.PANEL, ui.BORDER_LIGHT, 0.4), track,
                             border_radius=2)
            hh = max(10, int(track.h * visible / total))
            hy = track.y + int((track.h - hh) * top / max(1, total - visible))
            pygame.draw.rect(s, self.accent, (track.x, hy, 3, hh), border_radius=2)

    def _button_enabled(self, bid):
        if self.state != "play":
            return False
        if bid == "undo":
            if self.mode == "single":
                return any(i % 2 == self.human_color for i in range(len(self.moves)))
            return bool(self.moves)
        if bid == "hint":
            return self.mode == "multi" or self.pos.side == self.human_color
        if bid == "draw":
            return bool(self.moves)
        return True

    def _draw_buttons(self, s):
        for bid, rc in self.btn_rects.items():
            self._icon_button(s, rc, bid, self._button_enabled(bid),
                              hover=self.hover == bid)
        if self.state == "play" and isinstance(self.hover, str)                 and self.hover in self.btn_rects:
            self._draw_tooltip(s, self.btn_rects[self.hover],
                               t("chess.btn." + self.hover))

    def _icon_button(self, s, rc, bid, enabled=True, hover=False):
        fill = ui.BTN_SEL if hover and enabled else ui.BTN
        pygame.draw.rect(s, fill, rc, border_radius=max(4, rc.w // 6))
        pygame.draw.rect(s, self.accent if hover and enabled else ui.BORDER, rc,
                         1, border_radius=max(4, rc.w // 6))
        col = ui.TEXT if enabled else ui.TEXT_FAINT
        if hover and enabled:
            col = ui.mix(ui.TEXT, self.accent, 0.35)
        self._icon(s, rc, bid, col)

    def _icon(self, s, rc, kind, col):
        cx, cy = rc.center
        u = rc.w / 24.0
        lw = max(2, int(2.2 * u))
        if kind in ("undo", "retry"):
            rr = pygame.Rect(0, 0, int(13 * u), int(13 * u))
            rr.center = (cx, cy + int(1 * u))
            start = 0.35 if kind == "undo" else -0.4
            pygame.draw.arc(s, col, rr, start, start + 4.6, lw)
            ang = start + 4.6
            px = rr.centerx + math.cos(ang) * rr.w / 2
            py = rr.centery - math.sin(ang) * rr.h / 2
            hs = 4.5 * u
            pygame.draw.polygon(s, col, [(px - hs, py - hs * 0.2), (px + hs, py - hs * 0.2),
                                         (px, py + hs)] if kind == "retry" else
                                [(px - hs * 0.2, py - hs), (px - hs * 0.2, py + hs),
                                 (px + hs, py)])
        elif kind in ("hint", "solution"):
            r = int(5.5 * u)
            pygame.draw.circle(s, col, (cx, cy - int(2.5 * u)), r, lw)
            pygame.draw.line(s, col, (cx - int(3 * u), cy + int(5 * u)),
                             (cx + int(3 * u), cy + int(5 * u)), lw)
            pygame.draw.line(s, col, (cx - int(2 * u), cy + int(8 * u)),
                             (cx + int(2 * u), cy + int(8 * u)), lw)
            if kind == "hint":
                for a in (-2.3, -1.57, -0.84):
                    x1 = cx + math.cos(a) * 8.5 * u
                    y1 = cy - 2.5 * u + math.sin(a) * 8.5 * u
                    x2 = cx + math.cos(a) * 10.5 * u
                    y2 = cy - 2.5 * u + math.sin(a) * 10.5 * u
                    pygame.draw.line(s, col, (x1, y1), (x2, y2), max(1, lw - 1))
        elif kind == "flip":
            for sgn in (-1, 1):
                x = cx + sgn * 4 * u
                pygame.draw.line(s, col, (x, cy - 7 * u), (x, cy + 7 * u), lw)
                tip = cy - 8 * u if sgn < 0 else cy + 8 * u
                back = cy - 3 * u if sgn < 0 else cy + 3 * u
                pygame.draw.polygon(s, col, [(x, tip), (x - 3.5 * u, back), (x + 3.5 * u, back)])
        elif kind == "draw":
            f = ui.font(max(10, int(15 * u)), bold=True)
            img = self._text(f, "½", col)
            s.blit(img, img.get_rect(center=(cx, cy)))
        elif kind == "resign":
            x = cx - 5 * u
            pygame.draw.line(s, col, (x, cy - 8 * u), (x, cy + 8 * u), lw)
            pygame.draw.polygon(s, col, [(x, cy - 8 * u), (x + 11 * u, cy - 4.5 * u),
                                         (x, cy - 1 * u)])
        elif kind == "list":
            q = 4.2 * u
            for ix in (-1, 1):
                for iy in (-1, 1):
                    rr = pygame.Rect(0, 0, int(q * 1.5), int(q * 1.5))
                    rr.center = (int(cx + ix * q), int(cy + iy * q))
                    pygame.draw.rect(s, col, rr, border_radius=max(1, int(u)))
        elif kind == "next":
            pts = [(cx - 6 * u, cy - 7 * u), (cx + 2 * u, cy), (cx - 6 * u, cy + 7 * u)]
            pygame.draw.lines(s, col, False, pts, lw + 1)
            pts = [(cx + 1 * u, cy - 7 * u), (cx + 9 * u, cy), (cx + 1 * u, cy + 7 * u)]
            pygame.draw.lines(s, col, False, pts, lw + 1)

    # ===================================================== Overlays
    def _draw_toast(self, s):
        if not self.msg:
            return
        alpha = 255 if self.msg_t > 0.4 else int(255 * max(0.0, self.msg_t) / 0.4)
        fnt = self._fit(self.msg, self.bw - 24, self._small, self._tiny)
        text = self._clip_text(fnt, self.msg, self.bw - 24)
        img = fnt.render(text, True, ui.TEXT)
        w = img.get_width() + 28
        h = img.get_height() + 12
        rect = pygame.Rect(self.board_rect.centerx - w // 2,
                           self.board_rect.bottom - h - max(8, self.cell // 4), w, h)
        box = pygame.Surface(rect.size, pygame.SRCALPHA)
        pygame.draw.rect(box, (*ui.PANEL, 235), (0, 0, w, h), border_radius=h // 2)
        pygame.draw.rect(box, (*ui.mix(ui.BORDER, self.accent, 0.5), 255), (0, 0, w, h),
                         1, border_radius=h // 2)
        box.blit(img, img.get_rect(center=(w // 2, h // 2)))
        box.set_alpha(alpha)
        s.blit(box, rect)

    def _dim_board(self, s, alpha=150):
        s.blit(self._overlay(self.plate.w, (8, 8, 12), alpha, max(6, self.frame // 2)),
               self.plate.topleft)

    def _draw_promo(self, s):
        self._dim_board(s, 160)
        rects = self._promo_rects()
        head = self._text(self._small, t("chess.promote"), ui.TEXT)
        s.blit(head, head.get_rect(midbottom=(self.board_rect.centerx, rects[0].y - 10)))
        side = self.pos.side
        for i, m in enumerate(self.promo):
            rc = rects[i]
            ui.draw_panel(s, rc, color=ui.BTN_SEL, shadow=False)
            pygame.draw.rect(s, self.accent, rc, 2, border_radius=8)
            typ = (m >> 16) & 7
            spr = self._piece_sprite(typ | (side << 3), int(rc.w * 0.9))
            s.blit(spr, spr.get_rect(center=rc.center))

    def _draw_dialog(self, s):
        kind, side = self.dialog
        self._dim_board(s, 140)
        panel, yes, no = self._dialog_rects()
        ui.draw_panel(s, panel, accent_top=self.accent)
        color = t("chess.white") if side == ce.WHITE else t("chess.black")
        if kind == "resign":
            q = t("chess.resign_q") if self.mode == "single" else \
                t("chess.resign_q_color", color=color)
            yes_t, no_t = t("chess.yes"), t("chess.no")
        else:
            q = t("chess.draw_offer_q", color=color)
            yes_t, no_t = t("chess.accept"), t("chess.decline")
        lines = self._wrap(self._small, q, panel.w - 24)[:3]
        y = panel.y + 12
        for ln in lines:
            img = self._text(self._small, ln, ui.TEXT)
            s.blit(img, img.get_rect(midtop=(panel.centerx, y)))
            y += self._small.get_height() + 2
        for rc, label, sel in ((yes, yes_t, True), (no, no_t, False)):
            fnt = self._fit(label, rc.w - 8, self._small, self._tiny)
            ui.draw_button(s, rc, self._clip_text(fnt, label, rc.w - 8), fnt,
                           selected=sel, accent=self.accent)

    def _result_texts(self):
        winner, reason = self.result
        if winner is None:
            head, col = t("common.draw"), ui.TEXT
        elif self.mode == "multi":
            head = t("chess.wins_white") if winner == ce.WHITE else t("chess.wins_black")
            col = self.accent
        elif winner == self.human_color:
            head, col = t("chess.win_you"), ui.GOLD
        else:
            head, col = t("chess.win_ai"), ui.TEXT_DIM
        loser = self.result_loser if self.result_loser is not None else 0
        sub = t("chess.reason." + reason,
                color=t("chess.white") if loser == ce.WHITE else t("chess.black"))
        if self.mode == "single" and winner == self.human_color and self.assisted:
            sub += " · " + t("chess.assisted_short")
        return head, col, sub

    def _draw_over(self, s):
        k = min(1.0, (self._now() - getattr(self, "over_t0", 0)) / 0.35)
        if k < 1:
            self.over_t0 = getattr(self, "over_t0", 0)
        head, col, sub = self._result_texts()
        r = self.over_rect
        if k < 1:
            r = r.move(0, int((1 - _ease(k)) * 24))
        box = pygame.Surface(r.size, pygame.SRCALPHA)
        pygame.draw.rect(box, (*ui.PANEL, 238), (0, 0, r.w, r.h), border_radius=12)
        pygame.draw.rect(box, (*ui.mix(ui.BORDER, self.accent, 0.6), 255),
                         (0, 0, r.w, r.h), 1, border_radius=12)
        pygame.draw.rect(box, (*self.accent, 255), (14, 0, r.w - 28, 3), border_radius=2)
        hf = self._fit(head, r.w - 20, self._huge, self._big, self._small)
        img = hf.render(head, True, col)
        box.blit(img, img.get_rect(midtop=(r.w // 2, 10)))
        sf = self._small if len(self._wrap(self._small, sub, r.w - 20)) <= 2 else self._tiny
        y = 12 + img.get_height()
        for ln in self._wrap(sf, sub, r.w - 20)[:2]:
            simg = sf.render(self._clip_text(sf, ln, r.w - 20), True, ui.TEXT_DIM)
            box.blit(simg, simg.get_rect(midtop=(r.w // 2, y)))
            y += sf.get_height()
        box.set_alpha(int(255 * _ease(k)))
        s.blit(box, r)
        if k >= 1:
            labels = {"new": t("chess.btn.new"), "setup": t("chess.btn.setup"),
                      "pgn": t("chess.btn.pgn")}
            for bid, rc in self.over_btns.items():
                label = labels[bid]
                fnt = self._fit(label, rc.w - 8, self._small, self._tiny)
                ui.draw_button(s, rc, self._clip_text(fnt, label, rc.w - 8), fnt,
                               selected=(self.hover == "over_" + bid) or bid == "new",
                               accent=self.accent)

    # ===================================================== Setup
    def _draw_setup(self, s):
        w, h = self.width, self.height
        cx = w // 2
        title = t("chess.title") if self.mode == "single" else t("chess.title_multi")
        sub = t("chess.subtitle") if self.mode == "single" else t("chess.subtitle_multi")
        ui.draw_title(s, w, title, subtitle=self._clip_text(self._small, sub, w - 30),
                      y=int(h * 0.09), big=self._huge, small=self._small,
                      accent=self.accent)
        clock_labels = [t("chess.clock.none"), "1+0", "3+2", "5+0", "10+5"]
        for ri, (rid, rects) in enumerate(self.setup_rows):
            n, cur = self._setup_values(rid)
            if rid == "diff":
                label = t("chess.lbl.diff", name=t("chess.diff." + "lvl%d" % self.diff))
                labels = [str(i + 1) for i in range(6)]
            elif rid == "color":
                label = t("chess.lbl.color")
                labels = [t("chess.white"), t("chess.black")]
            elif rid == "clock":
                label = t("chess.lbl.clock")
                labels = clock_labels
            elif rid == "c960":
                label = t("chess.lbl.c960")
                labels = [t("common.off"), t("common.on")]
            else:
                label = t("chess.lbl.flip")
                labels = [t("common.off"), t("common.on")]
            focus = ri == self.setup_focus
            lab = self._text(self._tiny, self._clip_text(self._tiny, label, rects[-1].right - rects[0].x),
                             ui.TEXT if focus else ui.TEXT_DIM)
            s.blit(lab, lab.get_rect(midbottom=(cx, rects[0].y - 2)))
            if focus:
                pygame.draw.rect(s, self.accent, (rects[0].x - 10, rects[0].y + 4, 3,
                                                  rects[0].h - 8), border_radius=2)
            for i, rc in enumerate(rects):
                text = labels[i]
                fnt = self._fit(text, rc.w - 8, self._small, self._tiny)
                ui.draw_button(s, rc, self._clip_text(fnt, text, rc.w - 6), fnt,
                               selected=(i == cur), accent=self.accent)
        start_focus = self.setup_focus >= len(self.setup_rows)
        ui.draw_button(s, self.start_rect, t("common.start"), self._big if self._big.size(t("common.start"))[0] < self.start_rect.w - 10 else self._small,
                       selected=True, accent=self.accent)
        if start_focus:
            pygame.draw.rect(s, self.accent, self.start_rect.inflate(8, 8), 2,
                             border_radius=10)
        hint = t("chess.setup_hint") if self.mode == "single" else t("chess.setup_hint_multi")
        ui.draw_footer(s, w, h, self._clip_text(self._tiny, hint, w - 20), self._tiny)

    # ===================================================== Rätsel-Auswahl
    def _draw_puz_menu(self, s):
        w, h = self.width, self.height
        total = sum(len(v) for v in self.puzzles.values())
        solved = sum(1 for v in self.puzzles.values() for p in v if p["id"] in self.puz_solved)
        ui.draw_title(s, w, t("chess.puz.title"),
                      subtitle=t("chess.puz.progress", n=solved, total=total),
                      y=int(h * 0.075), big=self._huge, small=self._small,
                      accent=self.accent)
        for i, rc in enumerate(self.stage_rects):
            name = t("chess.stage." + ("mate1", "mate2", "mate3", "tactic1", "tactic2")[i])
            n = len(self._stage_list(i))
            sub = "%d/%d" % (self._stage_solved(i), n)
            fnt = self._fit(name, rc.w - 6, self._small, self._tiny)
            ui.draw_button(s, rc, self._clip_text(fnt, name, rc.w - 6), fnt,
                           selected=i == self.puz_stage, sub=sub, sub_font=self._tiny,
                           accent=self.accent)
        lst = self._stage_list()
        if not lst:
            img = self._text(self._small, t("chess.puz.missing"), ui.RED)
            s.blit(img, img.get_rect(center=(w // 2, h // 2)))
        for i, rc in enumerate(self.grid_rects[:len(lst)]):
            p = lst[i]
            done = p["id"] in self.puz_solved
            seen = p["id"] in self.puz_seen
            cur = i == self.puz_cursor
            hov = self.hover == ("cell", i)
            fill = ui.mix(ui.BTN, ui.GREEN, 0.28) if done else (ui.BTN_SEL if hov else ui.BTN)
            pygame.draw.rect(s, fill, rc, border_radius=6)
            border = self.accent if cur else (ui.mix(ui.BORDER, ui.GREEN, 0.5) if done else ui.BORDER)
            pygame.draw.rect(s, border, rc, 2 if cur else 1, border_radius=6)
            num = self._text(self._small, str(i + 1), ui.TEXT if (cur or done) else ui.TEXT_DIM)
            s.blit(num, num.get_rect(center=(rc.centerx, rc.centery - (3 if rc.h > 34 else 0))))
            if done:
                u = max(3, rc.h // 9)
                x0, y0 = rc.right - 3 * u - 3, rc.y + 2 * u + 1
                pygame.draw.lines(s, ui.GREEN, False,
                                  [(x0 - u, y0), (x0, y0 + u), (x0 + 2 * u, y0 - u)],
                                  max(2, u // 2))
            elif seen:
                pygame.draw.circle(s, ui.TEXT_FAINT, (rc.right - 7, rc.y + 7), 3)
            if rc.h > 34:
                rt = self._text(self._tiny, str(p.get("rating", "")), ui.TEXT_FAINT)
                s.blit(rt, rt.get_rect(midbottom=(rc.centerx, rc.bottom - 2)))
        if lst:
            label = t("chess.puz.start", n=self.puz_cursor + 1)
            fnt = self._fit(label, self.puz_start_rect.w - 10, self._small, self._tiny)
            ui.draw_button(s, self.puz_start_rect, label, fnt, selected=True,
                           accent=self.accent)
        ui.draw_footer(s, w, h, self._clip_text(self._tiny, t("chess.puz.footer"), w - 20),
                       self._tiny)

    # ===================================================== Rätsel-Seitenleiste
    def _draw_puz_side(self, s):
        r = self.puz_info_rect
        ui.draw_panel(s, r, shadow=False, accent_top=self.accent)
        pad = 8
        x = r.x + pad
        wmax = r.w - 2 * pad
        y = r.y + pad
        stage_id = ("mate1", "mate2", "mate3", "tactic1", "tactic2")[self.puz_stage]
        name = t("chess.stage." + stage_id)
        hf = self._fit(name, wmax, self._big, self._small)
        s.blit(self._text(hf, name, ui.TEXT), (x, y))
        y += hf.get_height() + 2
        n = len(self._stage_list())
        num = self._text(self._tiny, t("chess.puz.number", n=self.puz_idx + 1, total=n),
                         ui.TEXT_DIM)
        rat = self._text(self._tiny, t("chess.puz.rating", rating=self.puz.get("rating", "?")),
                         ui.TEXT_FAINT)
        s.blit(num, (x, y))
        if num.get_width() + rat.get_width() + 10 <= wmax:
            s.blit(rat, rat.get_rect(topright=(r.right - pad, y)))
        else:
            y += self._tiny.get_height() + 1
            s.blit(rat, (x, y))
        y += self._tiny.get_height() + 2
        theme = self.puz.get("theme", "")
        if theme and theme != "mate" and theme in _THEME_LABEL:
            tt = t("chess.puz.theme", name=t("chess.theme." + theme))
            img = self._text(self._tiny, self._clip_text(self._tiny, tt, wmax), self.accent)
            s.blit(img, (x, y))
            y += self._tiny.get_height() + 2
        y += 6
        # Wer ist am Zug + Ziel
        color = t("chess.white") if self.puz_player == ce.WHITE else t("chess.black")
        dot = max(5, self._small.get_height() // 3)
        pygame.draw.circle(s, COL_WHITE if self.puz_player == ce.WHITE else COL_BLACK,
                           (x + dot, y + self._small.get_height() // 2), dot)
        pygame.draw.circle(s, ui.BORDER_LIGHT, (x + dot, y + self._small.get_height() // 2),
                           dot, 1)
        you = t("chess.puz.you_play", color=color)
        yf = self._fit(you, wmax - 2 * dot - 6, self._small, self._tiny)
        s.blit(self._text(yf, self._clip_text(yf, you, wmax - 2 * dot - 6), ui.TEXT),
               (x + 2 * dot + 6, y + (self._small.get_height() - yf.get_height()) // 2))
        y += self._small.get_height() + 4
        if self.puz_mate == 1:
            goal = t("chess.puz.goal_mate1")
        elif self.puz_mate:
            goal = t("chess.puz.goal_mate", n=self.puz_mate)
        else:
            goal = t("chess.puz.goal_tactic")
        for ln in self._wrap(self._tiny, goal, wmax)[:2]:
            s.blit(self._text(self._tiny, ln, ui.TEXT_DIM), (x, y))
            y += self._tiny.get_height() + 1
        y += 6
        # Fortschritt: ein Punkt je eigener Zug
        own = len(self.puz_line) // 2
        done = min(own, self.puz_step // 2)
        rr = max(4, self._tiny.get_height() // 3)
        for i in range(own):
            cxp = x + rr + i * (rr * 3)
            col = ui.GREEN if i < done else ui.mix(ui.PANEL, ui.TEXT_FAINT, 0.6)
            pygame.draw.circle(s, col, (cxp, y + rr), rr)
        y += 2 * rr + 10
        # Rückmeldung (bis zu zwei Zeilen) + Zusatzzeile
        key, fcol = self.puz_feedback
        fb = t("chess.puz.fb." + key, color=color)
        extra = ""
        if key == "solved" and self.puz_mistakes:
            extra = t("chess.puz.mistakes", n=self.puz_mistakes)
        elif self.puz_status in ("solved", "shown_done"):
            extra = t("chess.puz.next_hint")
        room = r.bottom - pad - y
        bf = self._big if self._big.get_height() * 2 + self._tiny.get_height() <= room             else self._small
        for ln in self._wrap(bf, fb, wmax)[:2]:
            ln = self._clip_text(bf, ln, wmax)
            s.blit(self._text(bf, ln, fcol or ui.TEXT), (x, y))
            y += bf.get_height() + 1
        if extra and y + self._tiny.get_height() <= r.bottom - 4:
            s.blit(self._text(self._tiny, self._clip_text(self._tiny, extra, wmax),
                              ui.TEXT_DIM), (x, y + 1))
        # Zugliste der Aufgabe
        self._draw_movelist(s, self.puz_list_rect)
        # Knöpfe: Auswahl, Nochmal, Lösung zeigen, Nächstes
        for bid, rc in self.puz_btn_rects.items():
            enabled = True
            if bid == "solution":
                enabled = self.puz_status not in ("solved", "shown", "shown_done")
            hover = self.hover == bid
            highlight = bid == "next" and self.puz_status in ("solved", "shown_done")
            self._icon_button(s, rc, bid, enabled, hover=hover or highlight)
        if isinstance(self.hover, str) and self.hover in self.puz_btn_rects:
            self._draw_tooltip(s, self.puz_btn_rects[self.hover],
                               t("chess.btn." + self.hover))

    def _draw_tooltip(self, s, anchor, text):
        """Kleines Schild über einem Symbol-Knopf (bleibt in der Seitenleiste)."""
        fnt = self._tiny
        side = self.side_rect
        text = self._clip_text(fnt, text, side.w - 16)
        img = self._text(fnt, text, ui.TEXT)
        w = img.get_width() + 14
        h = img.get_height() + 8
        x = max(side.x, min(side.right - w, anchor.centerx - w // 2))
        y = anchor.y - h - 5
        rect = pygame.Rect(x, y, w, h)
        pygame.draw.rect(s, ui.PANEL_LIGHT, rect, border_radius=6)
        pygame.draw.rect(s, ui.mix(ui.BORDER_LIGHT, self.accent, 0.5), rect, 1,
                         border_radius=6)
        s.blit(img, img.get_rect(center=rect.center))
