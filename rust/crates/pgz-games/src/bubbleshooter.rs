//! bubbleshooter.rs
//! ================
//! Bubble Shooter - Dreiergruppen gleicher Farbe platzen lassen.
//!
//! Uebertragen aus `games/bubbleshooter.py`: versetztes Wabenraster,
//! Kanone mit Abprall, Einrasten in die naechste freie Wabe, Gruppen ab
//! drei, herabfallende lose Kugeln, nachrueckende Reihen je Modus.

use pgz_engine::prelude::*;

const BUBBLE_COLORS: [Color; 6] = [
    Color::rgb(230, 72, 72),
    Color::rgb(240, 205, 70),
    Color::rgb(96, 200, 112),
    Color::rgb(80, 140, 240),
    Color::rgb(175, 110, 235),
    Color::rgb(240, 150, 52),
];

fn mode_cfg(mode: &str) -> (usize, usize, u32) {
    match mode {
        "easy" => (4, 4, 0),
        "hard" => (6, 6, 6),
        _ => (5, 5, 8),
    }
}

const COLS: usize = 12;
const SPEED: f64 = 820.0;

#[derive(Clone, Copy)]
struct Shot {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    c: usize,
}

/// `BubbleShooterGame`
pub struct BubbleShooterGame {
    b: GameBase,
    ncolors: usize,
    start_rows: usize,
    drop_every: u32,
    over: bool,
    top: f64,
    r: f64,
    row_h: f64,
    bx: f64,
    left: f64,
    right: f64,
    cannon: (f64, f64),
    death_y: f64,
    grid: Vec<[Option<usize>; COLS]>,
    drop_offset: usize,
    shots: u32,
    shot: Option<Shot>,
    aim: f64,
    cur: usize,
    nxt: usize,
    hud: Font,
    small: Font,
    huge: Font,
}

impl BubbleShooterGame {
    /// `BubbleShooterGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let mut g = BubbleShooterGame {
            b,
            ncolors: 5,
            start_rows: 5,
            drop_every: 8,
            over: false,
            top: 0.0,
            r: 0.0,
            row_h: 0.0,
            bx: 0.0,
            left: 0.0,
            right: 0.0,
            cannon: (0.0, 0.0),
            death_y: 0.0,
            grid: Vec::new(),
            drop_offset: 0,
            shots: 0,
            shot: None,
            aim: 0.0,
            cur: 0,
            nxt: 0,
            hud: f.clone(),
            small: f.clone(),
            huge: f,
        };
        g.reset();
        Box::new(g)
    }

    fn make_fonts(&mut self) {
        self.hud = ui::fb(20);
        self.small = ui::f(15);
        self.huge = ui::fb(30.max(self.b.height / 11));
    }

    fn layout(&mut self) {
        let margin = 14.0;
        self.top = 54.0;
        let play_w = self.b.width as f64 - 2.0 * margin;
        self.r = 28.0f64.min(play_w / (2 * COLS + 1) as f64);
        self.row_h = self.r * 3f64.sqrt();
        let board_w = 2.0 * self.r * COLS as f64 + self.r;
        self.bx = (self.b.width as f64 - board_w) / 2.0;
        self.left = self.bx;
        self.right = self.bx + board_w;
        self.cannon = (self.b.width as f64 / 2.0, self.b.height as f64 - 30.0);
        self.death_y = self.cannon.1 - 2.0 * self.r - 26.0;
    }

    fn new_board(&mut self) {
        self.grid.clear();
        self.drop_offset = 0;
        for _ in 0..self.start_rows {
            let mut row = [None; COLS];
            for cell in row.iter_mut() {
                *cell = Some(py::randrange0(self.ncolors as i64) as usize);
            }
            self.grid.push(row);
        }
        self.shots = 0;
        self.shot = None;
        self.aim = -py::PI / 2.0;
        self.cur = self.pick_color();
        self.nxt = self.pick_color();
        self.over = false;
    }

    /// `_pick_color` - Pythons `list(set)` kleiner Zahlen ist aufsteigend.
    fn pick_color(&self) -> usize {
        let mut present: Vec<usize> = self.grid.iter().flat_map(|r| r.iter().flatten().copied()).collect();
        present.sort_unstable();
        present.dedup();
        if !present.is_empty() {
            return py::choice(&present);
        }
        py::randrange0(self.ncolors as i64) as usize
    }

    fn parity(&self, row: usize) -> usize {
        (row + self.drop_offset) % 2
    }

    fn center(&self, row: usize, col: usize) -> (f64, f64) {
        let x = self.bx + self.r + 2.0 * self.r * col as f64 + if self.parity(row) != 0 { self.r } else { 0.0 };
        let y = self.top + self.r + row as f64 * self.row_h;
        (x, y)
    }

    fn filled(&self, row: i64, col: i64) -> bool {
        0 <= row && (row as usize) < self.grid.len() && 0 <= col && (col as usize) < COLS && self.grid[row as usize][col as usize].is_some()
    }

    fn neighbors(&self, row: usize, col: usize) -> Vec<(usize, usize)> {
        let offs: [(i64, i64); 6] = if self.parity(row) == 0 {
            [(0, -1), (0, 1), (-1, -1), (-1, 0), (1, -1), (1, 0)]
        } else {
            [(0, -1), (0, 1), (-1, 0), (-1, 1), (1, 0), (1, 1)]
        };
        let mut out = Vec::new();
        for (dr, dc) in offs {
            let (rr, cc) = (row as i64 + dr, col as i64 + dc);
            if rr >= 0 && 0 <= cc && (cc as usize) < COLS {
                out.push((rr as usize, cc as usize));
            }
        }
        out
    }

    fn cell_from_point(&self, x: f64, y: f64) -> (usize, usize) {
        let row = py::round((y - self.top - self.r) / self.row_h).max(0) as usize;
        let p = (row + self.drop_offset) % 2;
        let col = py::round((x - self.bx - self.r - if p != 0 { self.r } else { 0.0 }) / (2.0 * self.r));
        (row, col.clamp(0, COLS as i64 - 1) as usize)
    }

    fn aim_at(&mut self, pos: (i32, i32)) {
        let dx = pos.0 as f64 - self.cannon.0;
        let mut dy = pos.1 as f64 - self.cannon.1;
        if dy > -1.0 {
            dy = -1.0;
        }
        let ang = dy.atan2(dx);
        self.aim = (-py::PI + 0.28).max((-0.28f64).min(ang));
    }

    fn shoot(&mut self) {
        if self.shot.is_some() || self.over {
            return;
        }
        self.shot = Some(Shot { x: self.cannon.0, y: self.cannon.1, vx: self.aim.cos() * SPEED, vy: self.aim.sin() * SPEED, c: self.cur });
        self.b.play_sound("move");
    }

    fn advance(&mut self, dt: f64) -> bool {
        let Some(mut sh) = self.shot else { return true };
        sh.x += sh.vx * dt;
        sh.y += sh.vy * dt;
        if sh.x <= self.left + self.r {
            sh.x = self.left + self.r;
            sh.vx = sh.vx.abs();
        } else if sh.x >= self.right - self.r {
            sh.x = self.right - self.r;
            sh.vx = -sh.vx.abs();
        }
        self.shot = Some(sh);
        if sh.y <= self.top + self.r {
            self.place(sh.x, sh.y, sh.c);
            return true;
        }
        let hit = 2.0 * self.r * 0.86;
        for row in 0..self.grid.len() {
            for col in 0..COLS {
                if self.grid[row][col].is_none() {
                    continue;
                }
                let (cx, cy) = self.center(row, col);
                if py::pow(sh.x - cx, 2.0) + py::pow(sh.y - cy, 2.0) <= hit * hit {
                    self.place(sh.x, sh.y, sh.c);
                    return true;
                }
            }
        }
        false
    }

    fn place(&mut self, x: f64, y: f64, color: usize) {
        self.shot = None;
        self.shots += 1;
        let (row, col) = self.snap_cell(x, y);
        while self.grid.len() <= row {
            self.grid.push([None; COLS]);
        }
        self.grid[row][col] = Some(color);
        self.b.play_sound("click");
        let cluster = self.same_color_cluster(row, col);
        if cluster.len() >= 3 {
            for (r, c) in &cluster {
                self.grid[*r][*c] = None;
            }
            self.b.score += cluster.len() as i64 * 10;
            self.b.play_sound("win");
            let dropped = self.drop_floating();
            if dropped > 0 {
                self.b.score += dropped as i64 * 20;
            }
            if cluster.len() + dropped >= 10 {
                self.b.ach_event("bubble_chain", None);
            }
        }
        if !self.grid.iter().any(|r| r.iter().any(Option::is_some)) {
            self.b.score += 500;
            self.b.play_sound("win");
            self.new_rows_top(self.start_rows);
        }
        if self.drop_every != 0 && self.shots % self.drop_every == 0 {
            self.new_rows_top(1);
        }
        self.cur = self.nxt;
        self.nxt = self.pick_color();
        self.check_over();
    }

    fn snap_cell(&self, x: f64, y: f64) -> (usize, usize) {
        let (r0, c0) = self.cell_from_point(x, y);
        let mut best = None;
        let mut bestd = 1e18;
        for rr in r0.saturating_sub(1)..r0 + 3 {
            for cc in 0..COLS {
                if self.filled(rr as i64, cc as i64) {
                    continue;
                }
                let attached = rr == 0 || self.neighbors(rr, cc).iter().any(|(nr, nc)| self.filled(*nr as i64, *nc as i64));
                if !attached {
                    continue;
                }
                let (cx, cy) = self.center(rr, cc);
                let d = py::pow(x - cx, 2.0) + py::pow(y - cy, 2.0);
                if d < bestd {
                    best = Some((rr, cc));
                    bestd = d;
                }
            }
        }
        best.unwrap_or((r0, c0))
    }

    fn same_color_cluster(&self, row: usize, col: usize) -> Vec<(usize, usize)> {
        let color = self.grid[row][col];
        let mut seen = vec![(row, col)];
        let mut stack = vec![(row, col)];
        while let Some((r, c)) = stack.pop() {
            for (nr, nc) in self.neighbors(r, c) {
                if !seen.contains(&(nr, nc)) && self.filled(nr as i64, nc as i64) && self.grid[nr][nc] == color {
                    seen.push((nr, nc));
                    stack.push((nr, nc));
                }
            }
        }
        seen
    }

    fn drop_floating(&mut self) -> usize {
        let mut anchored: Vec<(usize, usize)> = (0..COLS).filter(|c| self.filled(0, *c as i64)).map(|c| (0, c)).collect();
        let mut stack = anchored.clone();
        while let Some((r, c)) = stack.pop() {
            for (nr, nc) in self.neighbors(r, c) {
                if !anchored.contains(&(nr, nc)) && self.filled(nr as i64, nc as i64) {
                    anchored.push((nr, nc));
                    stack.push((nr, nc));
                }
            }
        }
        let mut dropped = 0;
        for r in 0..self.grid.len() {
            for c in 0..COLS {
                if self.filled(r as i64, c as i64) && !anchored.contains(&(r, c)) {
                    self.grid[r][c] = None;
                    dropped += 1;
                }
            }
        }
        dropped
    }

    fn new_rows_top(&mut self, n: usize) {
        for _ in 0..n {
            let mut row = [None; COLS];
            for cell in row.iter_mut() {
                *cell = Some(py::randrange0(self.ncolors as i64) as usize);
            }
            self.grid.insert(0, row);
            self.drop_offset += 1;
        }
    }

    fn check_over(&mut self) {
        for r in 0..self.grid.len() {
            for c in 0..COLS {
                if self.grid[r][c].is_some() {
                    let (_, cy) = self.center(r, c);
                    if cy + self.r >= self.death_y {
                        self.over = true;
                        self.b.game_over = true;
                        self.b.play_sound("gameover");
                        return;
                    }
                }
            }
        }
    }

    fn bubble(&self, s: &mut Surface, x: f64, y: f64, color: Color, radius: Option<f64>) {
        let rad = radius.unwrap_or(self.r) as i32;
        let (x, y) = (x as i32, y as i32);
        draw::circle(s, color, (x, y), rad, 0);
        draw::circle(s, mix(color, Color::rgb(255, 255, 255), 0.35), (x, y), rad, 2);
        draw::circle(s, mix(color, Color::rgb(255, 255, 255), 0.55), (x - rad / 3, y - rad / 3), 2.max(rad / 4), 0);
    }

    fn trace_aim(&self) -> Vec<(f64, f64)> {
        let (mut x, mut y) = self.cannon;
        let (mut vx, vy) = (self.aim.cos(), self.aim.sin());
        let mut pts = Vec::new();
        let step = self.r * 0.7;
        let lim = py::pow(2.0 * self.r * 0.86, 2.0);
        for _ in 0..140 {
            x += vx * step;
            y += vy * step;
            if x <= self.left + self.r {
                x = self.left + self.r;
                vx = vx.abs();
            } else if x >= self.right - self.r {
                x = self.right - self.r;
                vx = -vx.abs();
            }
            if y <= self.top + self.r {
                break;
            }
            let mut stop = false;
            'suche: for row in 0..self.grid.len() {
                for col in 0..COLS {
                    if self.grid[row][col].is_none() {
                        continue;
                    }
                    let (cx, cy) = self.center(row, col);
                    if py::pow(x - cx, 2.0) + py::pow(y - cy, 2.0) <= lim {
                        stop = true;
                        break 'suche;
                    }
                }
            }
            pts.push((x, y));
            if stop {
                break;
            }
        }
        pts
    }
}

impl Game for BubbleShooterGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        let (n, s, d) = mode_cfg(&self.b.mode);
        self.ncolors = n;
        self.start_rows = s;
        self.drop_every = d;
        self.b.score = 0;
        self.b.game_over = false;
        self.over = false;
        self.make_fonts();
        self.layout();
        self.new_board();
    }

    fn on_surface_changed(&mut self) {
        self.make_fonts();
        self.layout();
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if self.over {
            if ev.kind == Kind::MouseDown || ev.is_any(&["Return", "space"]) {
                self.b.game_over = false;
                self.reset();
            }
            return;
        }
        match ev.kind {
            Kind::MouseMove => self.aim_at(ev.pos),
            Kind::MouseDown if ev.button == 1 => {
                self.aim_at(ev.pos);
                self.shoot();
            }
            Kind::KeyDown => match ev.key.as_str() {
                "space" | "Up" | "w" | "W" => self.shoot(),
                "Left" | "a" | "A" => self.aim = (-py::PI + 0.28).max(self.aim - 0.08),
                "Right" | "d" | "D" => self.aim = (-0.28f64).min(self.aim + 0.08),
                _ => {}
            },
            _ => {}
        }
    }

    fn update(&mut self, dt: f64) {
        if self.over || self.shot.is_none() {
            return;
        }
        let dt = dt.min(0.05);
        let steps = 1.max((SPEED * dt / (self.r * 0.5)) as i64 + 1);
        let sdt = dt / steps as f64;
        for _ in 0..steps {
            if self.advance(sdt) {
                break;
            }
        }
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        let (w, h) = (self.b.width, self.b.height);
        ui::draw_background_ex(&mut s, w, h, false, Some(true));
        let y = self.death_y as i32;
        let mut x = self.left as i32;
        while x < self.right as i32 {
            draw::line(&mut s, Color::rgb(120, 60, 70), (x, y), (x + 8, y), 2);
            x += 16;
        }
        for r in 0..self.grid.len() {
            for c in 0..COLS {
                if let Some(col) = self.grid[r][c] {
                    let (cx, cy) = self.center(r, c);
                    self.bubble(&mut s, cx, cy, BUBBLE_COLORS[col], None);
                }
            }
        }
        if !self.over {
            for (i, (px, py_)) in self.trace_aim().iter().enumerate() {
                if i % 2 == 0 {
                    draw::circle(&mut s, mix(self.b.accent, Color::rgb(255, 255, 255), 0.2), (*px as i32, *py_ as i32), 3, 0);
                }
            }
        }
        let (cx, cy) = self.cannon;
        draw::circle(&mut s, Color::rgb(40, 46, 66), (cx as i32, cy as i32), self.r as i32 + 6, 0);
        draw::circle(&mut s, self.b.accent, (cx as i32, cy as i32), self.r as i32 + 6, 2);
        if self.shot.is_none() && !self.over {
            self.bubble(&mut s, cx, cy, BUBBLE_COLORS[self.cur], None);
        }
        if let Some(sh) = self.shot {
            self.bubble(&mut s, sh.x, sh.y, BUBBLE_COLORS[sh.c], None);
        }
        self.bubble(&mut s, self.left + self.r, cy, BUBBLE_COLORS[self.nxt], Some(self.r * 0.7));
        let img = self.hud.render(&t("bub.title"), true, self.b.accent);
        s.blr(&img, img.get_rect().midleft_at((20, 28)));
        let img = self.small.render(&tf("bub.score", &[("n", &self.b.score.to_string())]), true, ui::gold());
        s.blr(&img, img.get_rect().midright_at((w - 20, 28)));
        if self.over {
            let bw = (w - 40).min(460);
            let bh = 108;
            let rc = Rect::new((w - bw) / 2, (h - bh) / 2, bw, bh);
            let mut panel = pg::surface_alpha(bw, bh);
            panel.fill_c(Color::rgba(16, 18, 24, 238));
            s.bl(&panel, (rc.x, rc.y));
            let rot = Color::rgb(228, 96, 96);
            draw::rect_r(&mut s, rot, rc, 2, 14);
            let img = self.huge.render(&t("bub.gameover"), true, rot);
            s.blr(&img, img.get_rect().center_at((rc.centerx(), rc.y + 40)));
            let txt = format!("{}   \u{b7}   {}", tf("bub.score", &[("n", &self.b.score.to_string())]), t("common.enter_restart"));
            let img = self.small.render(&txt, true, ui::text_dim());
            s.blr(&img, img.get_rect().center_at((rc.centerx(), rc.y + 78)));
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "bubble",
    class_name: "BubbleShooterGame",
    supports_multiplayer: false,
    modes: &[("easy", "bub.mode.easy"), ("classic", "bub.mode.classic"), ("hard", "bub.mode.hard")],
    make: BubbleShooterGame::new,
};
