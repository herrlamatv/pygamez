//! chess_draw.rs
//! =============
//! Das Aussehen von Schach - uebertragen aus `games/chess_draw.py`.
//!
//! Gecacht: das Brett samt Rahmen (je Feldgroesse/Blickrichtung/Akzent),
//! jede Figur als Sprite (Fuellung, Innenlinien, Kontur, Schatten aus den
//! Unicode-Schachzeichen), halbtransparente Markierungen. Animationen laufen
//! ueber die Uhrzeit in `draw()` - wie in Python.

use std::f64::consts::PI;

use pgz_engine::prelude::*;

use crate::chess::{ChessGame, Hover, St, ANIM_TIME, STAGES};
use crate::chess_engine as ce;

type Rgb = (i32, i32, i32);

const COL_LIGHT: Rgb = (236, 222, 196);
const COL_DARK: Rgb = (160, 122, 90);
const COL_PLATE: Rgb = (44, 36, 31);
const COL_COORD: Rgb = (214, 198, 172);
const COL_SEL: Rgb = (246, 214, 92);
const COL_MOVE: Rgb = (60, 150, 90);
const COL_LAST: Rgb = (232, 206, 80);
const COL_CHECK: Rgb = (232, 64, 64);
const COL_HINT: Rgb = (80, 170, 255);
const COL_WHITE: Rgb = (248, 246, 240);
const COL_BLACK: Rgb = (40, 36, 44);
const COL_OUTLINE: Rgb = (22, 18, 20);

const GLYPH_FILL: [&str; 7] = ["", "\u{265F}", "\u{265E}", "\u{265D}", "\u{265C}", "\u{265B}", "\u{265A}"];
const GLYPH_LINE: [&str; 7] = ["", "\u{2659}", "\u{2658}", "\u{2657}", "\u{2656}", "\u{2655}", "\u{2654}"];
const PIECE_FONT: &str = "segoeuisymbol,dejavusans,arialunicodems,freeserif";

const THEME_LABEL: [&str; 12] = [
    "mate",
    "fork",
    "pin",
    "skewer",
    "discoveredAttack",
    "doubleCheck",
    "hangingPiece",
    "trappedPiece",
    "deflection",
    "attraction",
    "sacrifice",
    "promotion",
];

fn ease(k: f64) -> f64 {
    1.0 - py::pow(1.0 - k, 3.0)
}

fn rgba_of(c: impl Col, a: i32) -> Color {
    let c = c.c();
    rgba(c.r, c.g, c.b, a)
}

impl ChessGame {
    // ===================================================== Caches

    pub(crate) fn reset_caches(&mut self) {
        self.pieces.clear();
        self.board_cache = None;
        self.overlays.clear();
        self.arrow = None;
    }

    /// `_fit(text, width, *fonts)` - erste Schrift, in die der Text passt.
    fn fit<'a>(text: &str, width: i32, fonts: &[&'a Font]) -> &'a Font {
        for f in fonts {
            if f.size(text).0 <= width {
                return f;
            }
        }
        fonts[fonts.len() - 1]
    }

    /// `_clip_text(fnt, text, width)` - notfalls mit "…" kuerzen.
    fn clip_text(fnt: &Font, text: &str, width: i32) -> String {
        if fnt.size(text).0 <= width {
            return text.to_string();
        }
        let mut chars: Vec<char> = text.chars().collect();
        while chars.len() > 1 {
            let s: String = chars.iter().collect::<String>() + "\u{2026}";
            if fnt.size(&s).0 <= width {
                break;
            }
            chars.pop();
        }
        let s: String = chars.iter().collect();
        s.trim_end().to_string() + "\u{2026}"
    }

    /// `_wrap(fnt, text, width)`
    fn wrap(fnt: &Font, text: &str, width: i32) -> Vec<String> {
        let mut lines = Vec::new();
        let mut cur = String::new();
        for w in text.split_whitespace() {
            let test = format!("{cur} {w}").trim().to_string();
            if fnt.size(&test).0 <= width || cur.is_empty() {
                cur = test;
            } else {
                lines.push(std::mem::replace(&mut cur, w.to_string()));
            }
        }
        if !cur.is_empty() {
            lines.push(cur);
        }
        lines
    }

    fn overlay(&mut self, size: i32, color: Rgb, alpha: i32, radius: i32) -> Surface {
        let key = format!("ov{size},{color:?},{alpha},{radius}");
        if let Some(s) = self.overlays.get(&key) {
            return s.clone();
        }
        let mut s = pg::surface_alpha(size, size);
        draw::rect_r(&mut s, rgba_of(color, alpha), (0, 0, size, size), 0, radius);
        self.overlays.insert(key, s.clone());
        s
    }

    pub(crate) fn piece_sprite(&mut self, p: i32, size: i32) -> Surface {
        if let Some(s) = self.pieces.get(&(p, size)) {
            return s.clone();
        }
        let fnt = Font::sys(PIECE_FONT, 8.max((size as f64 * 0.84) as i32), false, false);
        let typ = (p & 7) as usize;
        let white = p >> 3 == 0;
        let fill = fnt.render(GLYPH_FILL[typ], true, if white { COL_WHITE } else { COL_BLACK });
        let line = fnt.render(GLYPH_LINE[typ], true, if white { COL_OUTLINE } else { (118, 110, 122) });
        let edge = fnt.render(GLYPH_FILL[typ], true, COL_OUTLINE);
        let shadow = fnt.render(GLYPH_FILL[typ], true, (0, 0, 0));
        let mut spr = pg::surface_alpha(size, size);
        let (cx, cy) = (size / 2, size / 2 + 1.max(size / 40));
        let o = 1.max(size / 36);
        let mut sh = pg::surface_alpha(shadow.w(), shadow.h());
        sh.bl(&shadow, (0, 0));
        sh.set_alpha(Some(70));
        spr.blr(&sh, sh.get_rect().center_at((cx + o + 1, cy + o * 2)));
        for (ox, oy) in [(-o, 0), (o, 0), (0, -o), (0, o), (-o, -o), (o, o), (-o, o), (o, -o)] {
            spr.blr(&edge, edge.get_rect().center_at((cx + ox, cy + oy)));
        }
        spr.blr(&fill, fill.get_rect().center_at((cx, cy)));
        spr.blr(&line, line.get_rect().center_at((cx, cy)));
        self.pieces.insert((p, size), spr.clone());
        spr
    }

    // ===================================================== Haupt-Zeichnen

    pub(crate) fn draw_all(&mut self) {
        let mut s = self.b.sf();
        ui::draw_background_ex(&mut s, self.b.width, self.b.height, false, Some(true));
        match self.state {
            St::Setup => {
                self.draw_setup(&mut s);
                return;
            }
            St::PuzMenu => {
                self.draw_puz_menu(&mut s);
                return;
            }
            _ => {}
        }
        self.draw_board(&mut s);
        if self.state == St::Puz {
            self.draw_puz_side(&mut s);
        } else {
            self.draw_side(&mut s);
        }
        if self.promo.is_some() {
            self.draw_promo(&mut s);
        }
        if self.dialog.is_some() {
            self.draw_dialog(&mut s);
        }
        if self.state == St::Over {
            self.draw_over(&mut s);
        }
        self.draw_toast(&mut s);
    }

    // ===================================================== Brett

    fn board_surface(&mut self) -> Surface {
        let key = (self.cell, self.frame, self.view_black, self.b.accent);
        if let Some((k, s)) = self.board_cache.as_ref() {
            if *k == key {
                return s.clone();
            }
        }
        let pw = self.plate.w;
        let mut surf = pg::surface_alpha(pw, pw);
        let rad = 6.max(self.frame / 2);
        draw::rect_r(&mut surf, COL_PLATE, (0, 0, pw, pw), 0, rad);
        draw::rect_r(&mut surf, ui::mix(COL_PLATE, self.b.accent, 0.5), (0, 0, pw, pw), 1, rad);
        let f = self.frame;
        let c = self.cell;
        for dr in 0..8 {
            for dc in 0..8 {
                let sq = self.disp_to_sq(dr, dc);
                let light = ((sq >> 4) + (sq & 7)) % 2 == 1;
                draw::rect(&mut surf, if light { COL_LIGHT } else { COL_DARK }, (f + dc * c, f + dr * c, c, c), 0);
            }
        }
        draw::rect(&mut surf, ui::mix(COL_PLATE, (0, 0, 0), 0.4), (f - 1, f - 1, 8 * c + 2, 8 * c + 2), 1);
        let cf = ui::font(9.max((f as f64 * 0.62) as i32), true, false);
        for i in 0..8 {
            let file_ch = (if self.view_black { ce::FILES[7 - i as usize] } else { ce::FILES[i as usize] } as char).to_string();
            let img = cf.render(&file_ch, true, COL_COORD);
            surf.blr(&img, img.get_rect().center_at((f + i * c + c / 2, f + 8 * c + f / 2)));
            let rank_ch = if self.view_black { (i + 1).to_string() } else { (8 - i).to_string() };
            let img = cf.render(&rank_ch, true, COL_COORD);
            surf.blr(&img, img.get_rect().center_at((f / 2, f + i * c + c / 2)));
        }
        self.board_cache = Some((key, surf.clone()));
        surf
    }

    fn draw_board(&mut self, s: &mut Surface) {
        let now = Self::now();
        let mut flip_k = None;
        if let Some(fa) = self.flip_anim {
            let k = (now - fa) / 0.32;
            if k >= 1.0 {
                self.flip_anim = None;
            } else {
                flip_k = Some(k);
            }
        }
        let mut own = flip_k.map(|_| pg::surface_alpha(self.plate.w, self.plate.h));
        let (ox, oy) = if own.is_some() { (0, 0) } else { (self.plate.x, self.plate.y) };
        let (dx, dy) = (ox - self.plate.x, oy - self.plate.y);
        let board = self.board_surface();
        let c = self.cell;
        {
            let target: &mut Surface = match own.as_mut() {
                Some(t) => t,
                None => s,
            };
            target.bl(&board, (ox, oy));
        }
        macro_rules! target {
            () => {
                match own.as_mut() {
                    Some(t) => t,
                    None => &mut *s,
                }
            };
        }
        if let Some((a, bsq)) = self.last_move {
            let ov = self.overlay(c, COL_LAST, 92, 0);
            for sq in [a, bsq] {
                let r = self.sq_rect(sq).move_(dx, dy);
                target!().blr(&ov, r);
            }
        }
        if self.check && matches!(self.state, St::Play | St::Puz) {
            let ks = self.pos.kings[self.pos.side as usize];
            let r = self.sq_rect(ks).move_(dx, dy);
            let glow = self.check_glow(c);
            target!().blr(&glow, glow.get_rect().center_at(r.center()));
        }
        let human = self.human_can_move();
        if let Some(sel) = self.sel.filter(|_| human) {
            let ov = self.overlay(c, COL_SEL, 110, 0);
            let r = self.sq_rect(sel).move_(dx, dy);
            target!().blr(&ov, r);
            for (sq, ms) in self.targets.clone() {
                let r = self.sq_rect(sq).move_(dx, dy);
                let occupied = self.pos.b[sq as usize] != 0 || ms.iter().any(|m| (m >> 20) == ce::M_EP);
                let mark = self.target_mark(c, occupied);
                target!().blr(&mark, r);
            }
        }
        let mut live = Vec::new();
        for fd in self.fades.clone() {
            let k = (now - fd.t0) / 0.35;
            if k < 1.0 {
                live.push(fd);
                let mut spr = self.piece_sprite(fd.p, c).copy();
                spr.set_alpha(Some((255.0 * (1.0 - k)) as i32 as u8));
                let size = (c as f64 * (1.0 - 0.35 * k)) as i32;
                if size > 4 {
                    spr = transform::smoothscale(&spr, size, size);
                }
                let r = self.sq_rect(fd.sq).move_(dx, dy);
                target!().blr(&spr, spr.get_rect().center_at(r.center()));
            }
        }
        self.fades = live;
        let mut anim_live = Vec::new();
        for a in &self.anims {
            let k = (now - a.t0) / ANIM_TIME;
            if k < 1.0 {
                anim_live.push((*a, ease(0f64.max(k))));
            }
        }
        let mut hidden: Vec<i32> = anim_live.iter().map(|(a, _)| a.b).collect();
        if let Some(d) = self.drag.filter(|d| d.moved) {
            hidden.push(d.sq);
        }
        for sq in ce::squares() {
            let p = self.pos.b[sq as usize];
            if p != 0 && !hidden.contains(&sq) {
                let spr = self.piece_sprite(p, c);
                let r = self.sq_rect(sq).move_(dx, dy);
                target!().blr(&spr, r);
            }
        }
        for (a, k) in &anim_live {
            let ra = self.sq_rect(a.a).move_(dx, dy);
            let rb = self.sq_rect(a.b).move_(dx, dy);
            let x = ra.x as f64 + (rb.x - ra.x) as f64 * k;
            let y = ra.y as f64 + (rb.y - ra.y) as f64 * k;
            let spr = self.piece_sprite(a.p, c);
            target!().bl(&spr, (x as i32, y as i32));
        }
        if anim_live.is_empty() {
            self.anims.clear();
        }
        if self.hint_move != 0 && self.state == St::Play {
            let arrow = self.arrow_surface(self.hint_move);
            target!().bl(&arrow, (self.bx + dx, self.by + dy));
        }
        if let Some(d) = self.drag.filter(|d| d.moved && human) {
            let p = self.pos.b[d.sq as usize];
            if p != 0 {
                let spr = self.piece_sprite(p, (c as f64 * 1.12) as i32);
                let (mx, my) = d.pos;
                if let Some(over) = self.sq_at((mx, my)) {
                    let r = self.sq_rect(over).move_(dx, dy);
                    draw::rect(target!(), ui::mix(COL_SEL, (255, 255, 255), 0.3), r, 2);
                }
                target!().blr(&spr, spr.get_rect().center_at((mx + dx, my + dy)));
            }
        }
        if human && self.drag.is_none() {
            let (dr, dc) = (self.cursor[0], self.cursor[1]);
            let rect = Rect::new(self.bx + dc * c + dx, self.by + dr * c + dy, c, c);
            let k = ui::pulse(3.0, 0.0, 1.0);
            let col = ui::mix(self.b.accent, (255, 255, 255), 0.35 + 0.4 * k);
            draw::rect_r(target!(), col, rect.inflate(-2, -2), 2.max(c / 22), 3);
        }
        if let (Some(k), Some(t)) = (flip_k, own.as_ref()) {
            let sx = (k * PI).cos().abs();
            let w = 2.max((self.plate.w as f64 * sx) as i32);
            let img = transform::scale(t, w, self.plate.h);
            s.bl(&img, (self.plate.centerx() - w / 2, self.plate.y));
        }
    }

    fn check_glow(&mut self, c: i32) -> Surface {
        let key = format!("check{c}");
        if let Some(g) = self.overlays.get(&key) {
            return g.clone();
        }
        let size = (c as f64 * 1.3) as i32;
        let mut g = pg::surface_alpha(size, size);
        let steps = 10;
        for i in (1..=steps).rev() {
            let r = (size as f64 / 2.0 * i as f64 / steps as f64) as i32;
            let a = (150.0 * (1.0 - i as f64 / steps as f64) + 30.0) as i32;
            draw::circle(&mut g, rgba_of(COL_CHECK, a), (size / 2, size / 2), r, 0);
        }
        self.overlays.insert(key, g.clone());
        g
    }

    fn target_mark(&mut self, c: i32, occupied: bool) -> Surface {
        let key = format!("target{c},{occupied}");
        if let Some(g) = self.overlays.get(&key) {
            return g.clone();
        }
        let mut g = pg::surface_alpha(c, c);
        if occupied {
            draw::circle(&mut g, rgba_of(COL_MOVE, 150), (c / 2, c / 2), c / 2 - 1, 3.max(c / 11));
        } else {
            draw::circle(&mut g, rgba_of(COL_MOVE, 150), (c / 2, c / 2), 4.max(c / 6), 0);
        }
        self.overlays.insert(key, g.clone());
        g
    }

    fn arrow_surface(&mut self, m: i32) -> Surface {
        let key = (m, self.view_black, self.cell);
        if let Some((k, s)) = self.arrow.as_ref() {
            if *k == key {
                return s.clone();
            }
        }
        let bw = self.bw;
        let mut surf = pg::surface_alpha(bw, bw);
        let (frm, to) = Self::move_squares(m);
        let ra = self.sq_rect(frm);
        let rb = self.sq_rect(to);
        let (ax, ay) = ((ra.centerx() - self.bx) as f64, (ra.centery() - self.by) as f64);
        let (bx, by) = ((rb.centerx() - self.bx) as f64, (rb.centery() - self.by) as f64);
        let ang = (by - ay).atan2(bx - ax);
        let width = 5.max(self.cell / 6);
        let head = 12.max((self.cell as f64 * 0.42) as i32) as f64;
        let ex = bx - ang.cos() * head * 0.9;
        let ey = by - ang.sin() * head * 0.9;
        let col = rgba_of(COL_HINT, 190);
        draw::line(&mut surf, col, (ax, ay), (ex, ey), width);
        draw::circle(&mut surf, col, (ax as i32, ay as i32), width / 2 + 2, 0);
        let left = (bx - (ang - 0.45).cos() * head, by - (ang - 0.45).sin() * head);
        let right = (bx - (ang + 0.45).cos() * head, by - (ang + 0.45).sin() * head);
        draw::polygon(&mut surf, col, &[(bx, by), left, right], 0);
        self.arrow = Some((key, surf.clone()));
        surf
    }

    // ===================================================== Seitenleiste

    fn draw_side(&mut self, s: &mut Surface) {
        let bottom = self.bottom_color();
        let (tp, bp) = (self.top_panel, self.bot_panel);
        self.draw_player(s, tp, 1 - bottom);
        self.draw_player(s, bp, bottom);
        self.draw_status(s);
        let lr = self.list_rect;
        self.draw_movelist(s, lr);
        self.draw_buttons(s);
    }

    fn bottom_color(&self) -> i32 {
        if self.view_black {
            ce::BLACK
        } else {
            ce::WHITE
        }
    }

    fn player_name(&self, color: i32) -> String {
        if self.b.mode == "single" {
            if color == self.human_color {
                return t("chess.you");
            }
            return format!("{} \u{00B7} {}", t("common.ai"), t(&format!("chess.diff.lvl{}", self.diff)));
        }
        let seat = if color == ce::WHITE { 0 } else { 1 };
        let base = if color == ce::WHITE { t("chess.white") } else { t("chess.black") };
        let who = self.b.pname(seat);
        if who != t(&format!("common.player{}", seat + 1)) {
            return format!("{base} \u{00B7} {who}");
        }
        base
    }

    fn draw_player(&mut self, s: &mut Surface, rect: Rect, color: i32) {
        let p = ui::pal();
        let active = self.state == St::Play && self.pos.side == color;
        ui::draw_panel_ex(s, rect, None, None, None, false, if active { Some(self.b.accent) } else { None });
        let pad = 8;
        let cy1 = rect.y + pad + self.small.get_height() / 2;
        let dot = 5.max(self.small.get_height() / 3);
        draw::circle(s, if color == ce::WHITE { COL_WHITE } else { COL_BLACK }, (rect.x + pad + dot, cy1), dot, 0);
        draw::circle(s, p.border_light, (rect.x + pad + dot, cy1), dot, 1);
        let mut clock_w = 0;
        if let Some(clock) = self.clock {
            let txt = Self::fmt_clock(clock[color as usize]);
            let low = clock[color as usize] < 10.0;
            let col = if low {
                p.red
            } else if active {
                p.text
            } else {
                p.text_dim
            };
            let img = self.clockf.render(&txt, true, col);
            let cr = img.get_rect().midright_at((rect.right() - pad, cy1));
            if active {
                let bg = cr.inflate(10, 4);
                draw::rect_r(s, ui::mix(p.panel_light, self.b.accent, 0.25), bg, 0, 5);
            }
            s.blr(&img, cr);
            clock_w = cr.w + 14;
        }
        let nx = rect.x + pad + dot * 2 + 6;
        let room = rect.right() - pad - clock_w - nx;
        let mut name = self.player_name(color);
        if self.b.mode == "multi" && self.wins.iter().any(|w| *w != 0) {
            let with_wins = format!("{}  \u{00B7}  {}", name, self.wins[color as usize]);
            if self.small.size(&with_wins).0 <= room {
                name = with_wins;
            }
        }
        let name = Self::clip_text(&self.small, &name, room);
        let img = self.small.render(&name, true, if active { p.text } else { p.text_dim });
        s.bl(&img, (nx, cy1 - self.small.get_height() / 2));
        let cy2 = rect.bottom() - pad - self.tiny.get_height() / 2;
        let mut caps: Vec<i32> = self.caps.iter().copied().filter(|p| *p != 0 && (p >> 3) != color).collect();
        caps.sort_by_key(|p| -ce::ORDER_VALUE[(p & 7) as usize]);
        let size = self.tiny.get_height() + 6;
        let step = 6.max((size as f64 * 0.52) as i32);
        let diff = self.pos.material(color) - self.pos.material(1 - color);
        let dimg = if diff > 0 { Some(self.tiny.render(&format!("+{diff}"), true, p.green)) } else { None };
        let maxx = rect.right() - pad - dimg.as_ref().map(|d| d.w() + 6).unwrap_or(0);
        let mut x = rect.x + pad - 2;
        for (i, pc) in caps.iter().enumerate() {
            if i > 0 && (caps[i - 1] & 7) != (pc & 7) {
                x += step / 2;
            }
            if x + size > maxx {
                break;
            }
            let spr = self.piece_sprite(*pc, size);
            s.bl(&spr, (x, cy2 - size / 2));
            x += step;
        }
        if let Some(d) = dimg {
            s.blr(&d, d.get_rect().midleft_at(((x + size - step + 6).min(maxx + 6), cy2)));
        }
    }

    fn fmt_clock(sec: f64) -> String {
        let sec = 0f64.max(sec);
        if sec < 10.0 {
            return format!("0:{:04.1}", sec);
        }
        let whole = sec.ceil() as i64;
        format!("{}:{:02}", whole / 60, whole % 60)
    }

    fn status_text(&self) -> (String, Color) {
        let p = ui::pal();
        if self.state == St::Over {
            return (t("chess.game_over"), p.text_dim);
        }
        if self.promo.is_some() {
            return (t("chess.promote"), self.b.accent);
        }
        if self.hint_job.is_some() {
            return (t("chess.hint_wait") + &Self::dots(), p.text_dim);
        }
        if self.b.mode == "single" {
            if self.pos.side != self.human_color {
                let base = t("chess.ai_thinks");
                return (base.trim_end_matches([' ', '\u{2026}', '.']).to_string() + &Self::dots(), p.text_dim);
            }
            if self.check {
                return (t("chess.check"), p.red);
            }
            return (t("chess.your_turn"), self.b.accent);
        }
        let who = if self.pos.side == ce::WHITE { t("chess.white") } else { t("chess.black") };
        let txt = tf("chess.turn", &[("name", &who)]);
        if self.check {
            return (t("chess.check") + " " + &txt, p.red);
        }
        (txt, self.b.accent)
    }

    fn dots() -> String {
        let n = (Self::now() * 3.0) as i64 % 4;
        " ".to_string() + &".".repeat(n as usize)
    }

    fn draw_status(&self, s: &mut Surface) {
        let (text, col) = self.status_text();
        let r = self.status_rect;
        let fnt = Self::fit(&text, r.w - 8, &[&self.small, &self.tiny]);
        let text = Self::clip_text(fnt, &text, r.w - 8);
        let img = fnt.render(&text, true, col);
        s.blr(&img, img.get_rect().center_at(r.center()));
    }

    fn draw_movelist(&mut self, s: &mut Surface, r: Rect) {
        let p = ui::pal();
        if r.h < 20 {
            return;
        }
        ui::draw_panel_ex(s, r, None, None, None, false, None);
        let fnt = self.listf.clone();
        let rh = self.row_h;
        let pad = 6;
        let mut head = t("chess.moves");
        if let Some(n) = self.c960_n {
            head += &format!("  \u{00B7}  Chess960 #{n}");
        }
        let hf = self.tiny.clone();
        let head = Self::clip_text(&hf, &head, r.w - 2 * pad);
        let img = hf.render(&head, true, p.text_faint);
        s.bl(&img, (r.x + pad, r.y + 4));
        let y0 = r.y + 6 + hf.get_height();
        let visible = self.list_visible();
        let total = self.list_rows();
        if self.list_follow {
            self.list_top = 0.max(total - visible);
        }
        let top = 0.max(self.list_top.min(0.max(total - visible)));
        let num_w = fnt.size(&format!("{}.", self.start_full as i64 + total + 90)).0 + 4;
        let col_w = py::fdiv32(r.w - 2 * pad - num_w, 2);
        let last = self.sans.len() as i64 - 1;
        let shift = self.start_side as i64;
        let prev_clip = s.get_clip();
        s.set_clip(Some(r.inflate(-2, -2)));
        for row in top..total.min(top + visible) {
            let y = y0 + (row - top) as i32 * rh;
            if row % 2 == 1 {
                draw::rect_r(s, ui::mix(p.panel, p.panel_light, 0.5), (r.x + 3, y - 1, r.w - 6, rh), 0, 3);
            }
            let img = fnt.render(&format!("{}.", self.start_full as i64 + row), true, p.text_faint);
            s.bl(&img, (r.x + pad, y));
            for ci in 0..2i64 {
                let i = row * 2 + ci - shift;
                let x = r.x + pad + num_w + ci as i32 * col_w;
                if i < 0 {
                    let img = fnt.render("\u{2026}", true, p.text_faint);
                    s.bl(&img, (x, y));
                    continue;
                }
                if i > last {
                    break;
                }
                let cur = i == last;
                if cur {
                    draw::rect_r(s, ui::mix(p.panel_light, self.b.accent, 0.35), (x - 3, y - 1, col_w - 2, rh), 0, 4);
                }
                let txt = Self::clip_text(&fnt, &self.sans[i as usize], col_w - 6);
                let img = fnt.render(&txt, true, if cur { p.text } else { p.text_dim });
                s.bl(&img, (x, y));
            }
        }
        s.set_clip(Some(prev_clip));
        if total > visible {
            let track = Rect::new(r.right() - 5, y0, 3, visible as i32 * rh);
            draw::rect_r(s, ui::mix(p.panel, p.border_light, 0.4), track, 0, 2);
            let hh = 10.max((track.h as f64 * visible as f64 / total as f64) as i32);
            let hy = track.y + ((track.h - hh) as f64 * top as f64 / 1.max(total - visible) as f64) as i32;
            draw::rect_r(s, self.b.accent, (track.x, hy, 3, hh), 0, 2);
        }
    }

    fn button_enabled(&self, bid: &str) -> bool {
        if self.state != St::Play {
            return false;
        }
        match bid {
            "undo" => {
                if self.b.mode == "single" {
                    (0..self.moves.len()).any(|i| (i % 2) as i32 == self.human_color)
                } else {
                    !self.moves.is_empty()
                }
            }
            "hint" => self.b.mode == "multi" || self.pos.side == self.human_color,
            "draw" => !self.moves.is_empty(),
            _ => true,
        }
    }

    fn draw_buttons(&self, s: &mut Surface) {
        for &(bid, rc) in &self.btn_rects {
            self.icon_button(s, rc, bid, self.button_enabled(bid), self.hover == Hover::Btn(bid));
        }
        if self.state == St::Play {
            if let Hover::Btn(h) = self.hover {
                if let Some(&(_, rc)) = self.btn_rects.iter().find(|e| e.0 == h) {
                    self.draw_tooltip(s, rc, &t(&format!("chess.btn.{h}")));
                }
            }
        }
    }

    fn icon_button(&self, s: &mut Surface, rc: Rect, bid: &str, enabled: bool, hover: bool) {
        let p = ui::pal();
        let fill = if hover && enabled { p.btn_sel } else { p.btn };
        draw::rect_r(s, fill, rc, 0, 4.max(rc.w / 6));
        draw::rect_r(s, if hover && enabled { self.b.accent } else { p.border }, rc, 1, 4.max(rc.w / 6));
        let mut col = if enabled { p.text } else { p.text_faint };
        if hover && enabled {
            col = ui::mix(p.text, self.b.accent, 0.35);
        }
        Self::icon(s, rc, bid, col);
    }

    fn icon(s: &mut Surface, rc: Rect, kind: &str, col: Color) {
        let (cx, cy) = rc.center();
        let (cxf, cyf) = (cx as f64, cy as f64);
        let u = rc.w as f64 / 24.0;
        let lw = 2.max((2.2 * u) as i32);
        match kind {
            "undo" | "retry" => {
                let rr = Rect::new(0, 0, (13.0 * u) as i32, (13.0 * u) as i32).center_at((cx, cy + (1.0 * u) as i32));
                let start = if kind == "undo" { 0.35 } else { -0.4 };
                draw::arc(s, col, rr, start, start + 4.6, lw);
                let ang: f64 = start + 4.6;
                let px = rr.centerx() as f64 + ang.cos() * rr.w as f64 / 2.0;
                let py_ = rr.centery() as f64 - ang.sin() * rr.h as f64 / 2.0;
                let hs = 4.5 * u;
                let pts = if kind == "retry" {
                    vec![(px - hs, py_ - hs * 0.2), (px + hs, py_ - hs * 0.2), (px, py_ + hs)]
                } else {
                    vec![(px - hs * 0.2, py_ - hs), (px - hs * 0.2, py_ + hs), (px + hs, py_)]
                };
                draw::polygon(s, col, &pts, 0);
            }
            "hint" | "solution" => {
                let r = (5.5 * u) as i32;
                draw::circle(s, col, (cx, cy - (2.5 * u) as i32), r, lw);
                draw::line(s, col, (cx - (3.0 * u) as i32, cy + (5.0 * u) as i32), (cx + (3.0 * u) as i32, cy + (5.0 * u) as i32), lw);
                draw::line(s, col, (cx - (2.0 * u) as i32, cy + (8.0 * u) as i32), (cx + (2.0 * u) as i32, cy + (8.0 * u) as i32), lw);
                if kind == "hint" {
                    for a in [-2.3f64, -1.57, -0.84] {
                        let x1 = cxf + a.cos() * 8.5 * u;
                        let y1 = cyf - 2.5 * u + a.sin() * 8.5 * u;
                        let x2 = cxf + a.cos() * 10.5 * u;
                        let y2 = cyf - 2.5 * u + a.sin() * 10.5 * u;
                        draw::line(s, col, (x1, y1), (x2, y2), 1.max(lw - 1));
                    }
                }
            }
            "flip" => {
                for sgn in [-1, 1] {
                    let x = cxf + (sgn * 4) as f64 * u;
                    draw::line(s, col, (x, cyf - 7.0 * u), (x, cyf + 7.0 * u), lw);
                    let tip = if sgn < 0 { cyf - 8.0 * u } else { cyf + 8.0 * u };
                    let back = if sgn < 0 { cyf - 3.0 * u } else { cyf + 3.0 * u };
                    draw::polygon(s, col, &[(x, tip), (x - 3.5 * u, back), (x + 3.5 * u, back)], 0);
                }
            }
            "draw" => {
                let f = ui::font(10.max((15.0 * u) as i32), true, false);
                let img = f.render("\u{00BD}", true, col);
                s.blr(&img, img.get_rect().center_at((cx, cy)));
            }
            "resign" => {
                let x = cxf - 5.0 * u;
                draw::line(s, col, (x, cyf - 8.0 * u), (x, cyf + 8.0 * u), lw);
                draw::polygon(s, col, &[(x, cyf - 8.0 * u), (x + 11.0 * u, cyf - 4.5 * u), (x, cyf - 1.0 * u)], 0);
            }
            "list" => {
                let q = 4.2 * u;
                for ix in [-1.0, 1.0] {
                    for iy in [-1.0, 1.0] {
                        let rr = Rect::new(0, 0, (q * 1.5) as i32, (q * 1.5) as i32).center_at(((cxf + ix * q) as i32, (cyf + iy * q) as i32));
                        draw::rect_r(s, col, rr, 0, 1.max(u as i32));
                    }
                }
            }
            "next" => {
                let pts = [(cxf - 6.0 * u, cyf - 7.0 * u), (cxf + 2.0 * u, cyf), (cxf - 6.0 * u, cyf + 7.0 * u)];
                draw::lines(s, col, false, &pts, lw + 1);
                let pts = [(cxf + 1.0 * u, cyf - 7.0 * u), (cxf + 9.0 * u, cyf), (cxf + 1.0 * u, cyf + 7.0 * u)];
                draw::lines(s, col, false, &pts, lw + 1);
            }
            _ => {}
        }
    }

    // ===================================================== Overlays

    fn draw_toast(&self, s: &mut Surface) {
        let p = ui::pal();
        let Some(msg) = self.msg.as_ref().filter(|m| !m.is_empty()) else {
            return;
        };
        let alpha = if self.msg_t > 0.4 { 255 } else { (255.0 * 0f64.max(self.msg_t) / 0.4) as i32 };
        let fnt = Self::fit(msg, self.bw - 24, &[&self.small, &self.tiny]);
        let text = Self::clip_text(fnt, msg, self.bw - 24);
        let img = fnt.render(&text, true, p.text);
        let w = img.w() + 28;
        let h = img.h() + 12;
        let rect = Rect::new(self.board_rect.centerx() - w / 2, self.board_rect.bottom() - h - 8.max(self.cell / 4), w, h);
        let mut bx = pg::surface_alpha(w, h);
        draw::rect_r(&mut bx, rgba_of(p.panel, 235), (0, 0, w, h), 0, h / 2);
        draw::rect_r(&mut bx, rgba_of(ui::mix(p.border, self.b.accent, 0.5), 255), (0, 0, w, h), 1, h / 2);
        bx.blr(&img, img.get_rect().center_at((w / 2, h / 2)));
        bx.set_alpha(Some(alpha.clamp(0, 255) as u8));
        s.blr(&bx, rect);
    }

    fn dim_board(&mut self, s: &mut Surface, alpha: i32) {
        let ov = self.overlay(self.plate.w, (8, 8, 12), alpha, 6.max(self.frame / 2));
        s.bl(&ov, (self.plate.x, self.plate.y));
    }

    fn draw_promo(&mut self, s: &mut Surface) {
        let p = ui::pal();
        self.dim_board(s, 160);
        let rects = self.promo_rects();
        let head = self.small.render(&t("chess.promote"), true, p.text);
        s.blr(&head, head.get_rect().midbottom_at((self.board_rect.centerx(), rects[0].y - 10)));
        let side = self.pos.side;
        for (i, m) in self.promo.clone().unwrap_or_default().iter().enumerate() {
            let rc = rects[i];
            ui::draw_panel_ex(s, rc, Some(p.btn_sel), None, None, false, None);
            draw::rect_r(s, self.b.accent, rc, 2, 8);
            let typ = (m >> 16) & 7;
            let spr = self.piece_sprite(typ | (side << 3), (rc.w as f64 * 0.9) as i32);
            s.blr(&spr, spr.get_rect().center_at(rc.center()));
        }
    }

    fn draw_dialog(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let Some((kind, side)) = self.dialog else {
            return;
        };
        self.dim_board(s, 140);
        let (panel, yes, no) = self.dialog_rects();
        ui::draw_panel_ex(s, panel, None, None, None, true, Some(self.b.accent));
        let color = if side == ce::WHITE { t("chess.white") } else { t("chess.black") };
        let (q, yes_t, no_t) = if kind == "resign" {
            let q = if self.b.mode == "single" { t("chess.resign_q") } else { tf("chess.resign_q_color", &[("color", &color)]) };
            (q, t("chess.yes"), t("chess.no"))
        } else {
            (tf("chess.draw_offer_q", &[("color", &color)]), t("chess.accept"), t("chess.decline"))
        };
        let lines = Self::wrap(&self.small, &q, panel.w - 24);
        let mut y = panel.y + 12;
        for ln in lines.iter().take(3) {
            let img = self.small.render(ln, true, p.text);
            s.blr(&img, img.get_rect().midtop_at((panel.centerx(), y)));
            y += self.small.get_height() + 2;
        }
        for (rc, label, sel) in [(yes, yes_t, true), (no, no_t, false)] {
            let fnt = Self::fit(&label, rc.w - 8, &[&self.small, &self.tiny]);
            let txt = Self::clip_text(fnt, &label, rc.w - 8);
            ui::draw_button_ex(s, rc, &txt, fnt, sel, None, None, Some(self.b.accent));
        }
    }

    fn result_texts(&self) -> (String, Color, String) {
        let p = ui::pal();
        let (winner, reason) = self.result.unwrap_or((None, "agreed"));
        let (head, col) = match winner {
            None => (t("common.draw"), p.text),
            Some(w) if self.b.mode == "multi" => (if w == ce::WHITE { t("chess.wins_white") } else { t("chess.wins_black") }, self.b.accent),
            Some(w) if w == self.human_color => (t("chess.win_you"), p.gold),
            Some(_) => (t("chess.win_ai"), p.text_dim),
        };
        let loser = self.result_loser.unwrap_or(0);
        let mut sub = tf(&format!("chess.reason.{reason}"), &[("color", &if loser == ce::WHITE { t("chess.white") } else { t("chess.black") })]);
        if self.b.mode == "single" && winner == Some(self.human_color) && self.assisted {
            sub += &(" \u{00B7} ".to_string() + &t("chess.assisted_short"));
        }
        (head, col, sub)
    }

    fn draw_over(&self, s: &mut Surface) {
        let p = ui::pal();
        let k = 1f64.min((Self::now() - self.over_t0) / 0.35);
        let (head, col, sub) = self.result_texts();
        let mut r = self.over_rect;
        if k < 1.0 {
            r = r.move_(0, ((1.0 - ease(k)) * 24.0) as i32);
        }
        let mut bx = pg::surface_alpha(r.w, r.h);
        draw::rect_r(&mut bx, rgba_of(p.panel, 238), (0, 0, r.w, r.h), 0, 12);
        draw::rect_r(&mut bx, rgba_of(ui::mix(p.border, self.b.accent, 0.6), 255), (0, 0, r.w, r.h), 1, 12);
        draw::rect_r(&mut bx, rgba_of(self.b.accent, 255), (14, 0, r.w - 28, 3), 0, 2);
        let hf = Self::fit(&head, r.w - 20, &[&self.huge, &self.big, &self.small]);
        let img = hf.render(&head, true, col);
        bx.blr(&img, img.get_rect().midtop_at((r.w / 2, 10)));
        let sf = if Self::wrap(&self.small, &sub, r.w - 20).len() <= 2 { &self.small } else { &self.tiny };
        let mut y = 12 + img.h();
        for ln in Self::wrap(sf, &sub, r.w - 20).iter().take(2) {
            let simg = sf.render(&Self::clip_text(sf, ln, r.w - 20), true, p.text_dim);
            bx.blr(&simg, simg.get_rect().midtop_at((r.w / 2, y)));
            y += sf.get_height();
        }
        bx.set_alpha(Some((255.0 * ease(k)) as i32 as u8));
        s.blr(&bx, r);
        if k >= 1.0 {
            for &(bid, rc) in &self.over_btns {
                let label = match bid {
                    "new" => t("chess.btn.new"),
                    "setup" => t("chess.btn.setup"),
                    _ => t("chess.btn.pgn"),
                };
                let fnt = Self::fit(&label, rc.w - 8, &[&self.small, &self.tiny]);
                let txt = Self::clip_text(fnt, &label, rc.w - 8);
                ui::draw_button_ex(s, rc, &txt, fnt, self.hover == Hover::Over(bid) || bid == "new", None, None, Some(self.b.accent));
            }
        }
    }

    // ===================================================== Setup

    fn draw_setup(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let single = self.b.mode == "single";
        let title = if single { t("chess.title") } else { t("chess.title_multi") };
        let sub = if single { t("chess.subtitle") } else { t("chess.subtitle_multi") };
        let sub = Self::clip_text(&self.small, &sub, w - 30);
        ui::draw_title_ex(s, w, &title, Some(&sub), (h as f64 * 0.09) as i32, Some(&self.huge), Some(&self.small), Some(self.b.accent));
        let clock_labels: Vec<String> = vec![t("chess.clock.none"), "1+0".into(), "3+2".into(), "5+0".into(), "10+5".into()];
        for (ri, (rid, rects)) in self.setup_rows.iter().enumerate() {
            let (_n, cur) = self.setup_values(rid);
            let (label, labels): (String, Vec<String>) = match *rid {
                "diff" => (tf("chess.lbl.diff", &[("name", &t(&format!("chess.diff.lvl{}", self.diff)))]), (1..=6).map(|i| i.to_string()).collect()),
                "color" => (t("chess.lbl.color"), vec![t("chess.white"), t("chess.black")]),
                "clock" => (t("chess.lbl.clock"), clock_labels.clone()),
                "c960" => (t("chess.lbl.c960"), vec![t("common.off"), t("common.on")]),
                _ => (t("chess.lbl.flip"), vec![t("common.off"), t("common.on")]),
            };
            let focus = ri == self.setup_focus;
            let lab_txt = Self::clip_text(&self.tiny, &label, rects[rects.len() - 1].right() - rects[0].x);
            let lab = self.tiny.render(&lab_txt, true, if focus { p.text } else { p.text_dim });
            s.blr(&lab, lab.get_rect().midbottom_at((cx, rects[0].y - 2)));
            if focus {
                draw::rect_r(s, self.b.accent, (rects[0].x - 10, rects[0].y + 4, 3, rects[0].h - 8), 0, 2);
            }
            for (i, rc) in rects.iter().enumerate() {
                let text = &labels[i];
                let fnt = Self::fit(text, rc.w - 8, &[&self.small, &self.tiny]);
                let txt = Self::clip_text(fnt, text, rc.w - 6);
                ui::draw_button_ex(s, *rc, &txt, fnt, i == cur, None, None, Some(self.b.accent));
            }
        }
        let start_focus = self.setup_focus >= self.setup_rows.len();
        let st = t("common.start");
        let sf = if self.big.size(&st).0 < self.start_rect.w - 10 { &self.big } else { &self.small };
        ui::draw_button_ex(s, self.start_rect, &st, sf, true, None, None, Some(self.b.accent));
        if start_focus {
            draw::rect_r(s, self.b.accent, self.start_rect.inflate(8, 8), 2, 10);
        }
        let hint = if single { t("chess.setup_hint") } else { t("chess.setup_hint_multi") };
        let hint = Self::clip_text(&self.tiny, &hint, w - 20);
        ui::draw_footer_ex(s, w, h, &hint, Some(&self.tiny));
    }

    // ===================================================== Raetsel-Auswahl

    fn draw_puz_menu(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let total: usize = self.puzzles.iter().map(|v| v.len()).sum();
        let solved: usize = self.puzzles.iter().flat_map(|v| v.iter()).filter(|pz| self.puz_solved.contains(&pz.id)).count();
        let sub = tf("chess.puz.progress", &[("n", &solved.to_string()), ("total", &total.to_string())]);
        ui::draw_title_ex(s, w, &t("chess.puz.title"), Some(&sub), (h as f64 * 0.075) as i32, Some(&self.huge), Some(&self.small), Some(self.b.accent));
        for (i, rc) in self.stage_rects.iter().enumerate() {
            let name = t(&format!("chess.stage.{}", STAGES[i]));
            let n = self.stage_list(Some(i)).len();
            let sub = format!("{}/{}", self.stage_solved(i), n);
            let fnt = Self::fit(&name, rc.w - 6, &[&self.small, &self.tiny]);
            let txt = Self::clip_text(fnt, &name, rc.w - 6);
            ui::draw_button_ex(s, *rc, &txt, fnt, i == self.puz_stage, Some(&sub), Some(&self.tiny), Some(self.b.accent));
        }
        let lst = self.stage_list(None);
        if lst.is_empty() {
            let img = self.small.render(&t("chess.puz.missing"), true, p.red);
            s.blr(&img, img.get_rect().center_at((w / 2, h / 2)));
        }
        for (i, rc) in self.grid_rects.iter().take(lst.len()).enumerate() {
            let pz = &lst[i];
            let done = self.puz_solved.contains(&pz.id);
            let seen = self.puz_seen.contains(&pz.id);
            let cur = i == self.puz_cursor;
            let hov = self.hover == Hover::Cell(i);
            let fill = if done {
                ui::mix(p.btn, p.green, 0.28)
            } else if hov {
                p.btn_sel
            } else {
                p.btn
            };
            draw::rect_r(s, fill, *rc, 0, 6);
            let border = if cur {
                self.b.accent
            } else if done {
                ui::mix(p.border, p.green, 0.5)
            } else {
                p.border
            };
            draw::rect_r(s, border, *rc, if cur { 2 } else { 1 }, 6);
            let num = self.small.render(&(i + 1).to_string(), true, if cur || done { p.text } else { p.text_dim });
            s.blr(&num, num.get_rect().center_at((rc.centerx(), rc.centery() - if rc.h > 34 { 3 } else { 0 })));
            if done {
                let u = 3.max(rc.h / 9);
                let (x0, y0) = (rc.right() - 3 * u - 3, rc.y + 2 * u + 1);
                draw::lines(s, p.green, false, &[(x0 - u, y0), (x0, y0 + u), (x0 + 2 * u, y0 - u)], 2.max(u / 2));
            } else if seen {
                draw::circle(s, p.text_faint, (rc.right() - 7, rc.y + 7), 3, 0);
            }
            if rc.h > 34 {
                let rating = match &pz.rating {
                    Some(serde_json::Value::String(st)) => st.clone(),
                    Some(v) => v.to_string(),
                    None => String::new(),
                };
                let rt = self.tiny.render(&rating, true, p.text_faint);
                s.blr(&rt, rt.get_rect().midbottom_at((rc.centerx(), rc.bottom() - 2)));
            }
        }
        if !lst.is_empty() {
            let label = tf("chess.puz.start", &[("n", &(self.puz_cursor + 1).to_string())]);
            let fnt = Self::fit(&label, self.puz_start_rect.w - 10, &[&self.small, &self.tiny]);
            ui::draw_button_ex(s, self.puz_start_rect, &label, fnt, true, None, None, Some(self.b.accent));
        }
        let foot = Self::clip_text(&self.tiny, &t("chess.puz.footer"), w - 20);
        ui::draw_footer_ex(s, w, h, &foot, Some(&self.tiny));
    }

    // ===================================================== Raetsel-Seitenleiste

    fn draw_puz_side(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let r = self.puz_info_rect;
        ui::draw_panel_ex(s, r, None, None, None, false, Some(self.b.accent));
        let pad = 8;
        let x = r.x + pad;
        let wmax = r.w - 2 * pad;
        let mut y = r.y + pad;
        let name = t(&format!("chess.stage.{}", STAGES[self.puz_stage]));
        let hf = Self::fit(&name, wmax, &[&self.big, &self.small]).clone();
        let img = hf.render(&name, true, p.text);
        s.bl(&img, (x, y));
        y += hf.get_height() + 2;
        let n = self.stage_list(None).len();
        let num = self.tiny.render(&tf("chess.puz.number", &[("n", &(self.puz_idx + 1).to_string()), ("total", &n.to_string())]), true, p.text_dim);
        let rating = match self.puz.as_ref().and_then(|pz| pz.rating.clone()) {
            Some(serde_json::Value::String(st)) => st,
            Some(v) => v.to_string(),
            None => "?".into(),
        };
        let rat = self.tiny.render(&tf("chess.puz.rating", &[("rating", &rating)]), true, p.text_faint);
        s.bl(&num, (x, y));
        if num.w() + rat.w() + 10 <= wmax {
            s.blr(&rat, rat.get_rect().topright_at((r.right() - pad, y)));
        } else {
            y += self.tiny.get_height() + 1;
            s.bl(&rat, (x, y));
        }
        y += self.tiny.get_height() + 2;
        let theme = self.puz.as_ref().map(|pz| pz.theme.clone()).unwrap_or_default();
        if !theme.is_empty() && theme != "mate" && THEME_LABEL.contains(&theme.as_str()) {
            let tt = tf("chess.puz.theme", &[("name", &t(&format!("chess.theme.{theme}")))]);
            let img = self.tiny.render(&Self::clip_text(&self.tiny, &tt, wmax), true, self.b.accent);
            s.bl(&img, (x, y));
            y += self.tiny.get_height() + 2;
        }
        y += 6;
        let color = if self.puz_player == ce::WHITE { t("chess.white") } else { t("chess.black") };
        let dot = 5.max(self.small.get_height() / 3);
        let sh = self.small.get_height();
        draw::circle(s, if self.puz_player == ce::WHITE { COL_WHITE } else { COL_BLACK }, (x + dot, y + sh / 2), dot, 0);
        draw::circle(s, p.border_light, (x + dot, y + sh / 2), dot, 1);
        let you = tf("chess.puz.you_play", &[("color", &color)]);
        let yf = Self::fit(&you, wmax - 2 * dot - 6, &[&self.small, &self.tiny]);
        let img = yf.render(&Self::clip_text(yf, &you, wmax - 2 * dot - 6), true, p.text);
        s.bl(&img, (x + 2 * dot + 6, y + py::fdiv32(sh - yf.get_height(), 2)));
        y += sh + 4;
        let goal = if self.puz_mate == 1 {
            t("chess.puz.goal_mate1")
        } else if self.puz_mate != 0 {
            tf("chess.puz.goal_mate", &[("n", &self.puz_mate.to_string())])
        } else {
            t("chess.puz.goal_tactic")
        };
        for ln in Self::wrap(&self.tiny, &goal, wmax).iter().take(2) {
            let img = self.tiny.render(ln, true, p.text_dim);
            s.bl(&img, (x, y));
            y += self.tiny.get_height() + 1;
        }
        y += 6;
        let own = self.puz_line.len() / 2;
        let done = own.min(self.puz_step / 2);
        let rr = 4.max(self.tiny.get_height() / 3);
        for i in 0..own {
            let cxp = x + rr + i as i32 * (rr * 3);
            let col = if i < done { p.green } else { ui::mix(p.panel, p.text_faint, 0.6) };
            draw::circle(s, col, (cxp, y + rr), rr, 0);
        }
        y += 2 * rr + 10;
        let (key, fcol) = self.puz_feedback;
        let fb = tf(&format!("chess.puz.fb.{key}"), &[("color", &color)]);
        let mut extra = String::new();
        if key == "solved" && self.puz_mistakes != 0 {
            extra = tf("chess.puz.mistakes", &[("n", &self.puz_mistakes.to_string())]);
        } else if self.puz_status == "solved" || self.puz_status == "shown_done" {
            extra = t("chess.puz.next_hint");
        }
        let room = r.bottom() - pad - y;
        let bf = if self.big.get_height() * 2 + self.tiny.get_height() <= room { self.big.clone() } else { self.small.clone() };
        for ln in Self::wrap(&bf, &fb, wmax).iter().take(2) {
            let ln = Self::clip_text(&bf, ln, wmax);
            let img = bf.render(&ln, true, fcol.unwrap_or(p.text));
            s.bl(&img, (x, y));
            y += bf.get_height() + 1;
        }
        if !extra.is_empty() && y + self.tiny.get_height() <= r.bottom() - 4 {
            let img = self.tiny.render(&Self::clip_text(&self.tiny, &extra, wmax), true, p.text_dim);
            s.bl(&img, (x, y + 1));
        }
        let plr = self.puz_list_rect;
        self.draw_movelist(s, plr);
        for &(bid, rc) in &self.puz_btn_rects {
            let mut enabled = true;
            if bid == "solution" {
                enabled = !["solved", "shown", "shown_done"].contains(&self.puz_status);
            }
            let hover = self.hover == Hover::Btn(bid);
            let highlight = bid == "next" && (self.puz_status == "solved" || self.puz_status == "shown_done");
            self.icon_button(s, rc, bid, enabled, hover || highlight);
        }
        if let Hover::Btn(h) = self.hover {
            if let Some(&(_, rc)) = self.puz_btn_rects.iter().find(|e| e.0 == h) {
                self.draw_tooltip(s, rc, &t(&format!("chess.btn.{h}")));
            }
        }
    }

    fn draw_tooltip(&self, s: &mut Surface, anchor: Rect, text: &str) {
        let p = ui::pal();
        let fnt = &self.tiny;
        let side = self.side_rect;
        let text = Self::clip_text(fnt, text, side.w - 16);
        let img = fnt.render(&text, true, p.text);
        let w = img.w() + 14;
        let h = img.h() + 8;
        let x = side.x.max((side.right() - w).min(anchor.centerx() - w / 2));
        let y = anchor.y - h - 5;
        let rect = Rect::new(x, y, w, h);
        draw::rect_r(s, p.panel_light, rect, 0, 6);
        draw::rect_r(s, ui::mix(p.border_light, self.b.accent, 0.5), rect, 1, 6);
        s.blr(&img, img.get_rect().center_at(rect.center()));
    }
}
