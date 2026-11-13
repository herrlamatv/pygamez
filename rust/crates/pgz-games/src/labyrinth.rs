//! labyrinth.rs
//! ============
//! 3D-Labyrinth - Ego-Raycaster im Wolfenstein-Stil.
//!
//! Uebertragen aus `games/labyrinth.py`: 50 seed-generierte Level
//! (`maze_gen`), Ego-Ansicht mit DDA-Raycaster und Sprites, alternativ
//! Draufsicht; Orbs einsammeln, Ausgang finden, Fortschritt im Profil.

use std::collections::HashSet;

use crate::maze_gen::{self, LEVELS};
use pgz_core::store;
use pgz_engine::prelude::*;
use serde_json::{Map, Value};

const MOVE_SPEED: f64 = 3.0;
const PLAYER_R: f64 = 0.25;
const TURN_KEY: f64 = 90.0;
const DEG_PER_PX: f64 = 0.12;
const FOG_DIST: f64 = 14.0;

const COL_BG: (i32, i32, i32) = (16, 14, 22);
const COL_CEIL_TOP: Color = Color::rgb(30, 34, 52);
const COL_CEIL_BOT: Color = Color::rgb(20, 22, 34);
const COL_FLOOR: Color = Color::rgb(34, 30, 40);
const COL_WALL: (i32, i32, i32) = (90, 110, 170);
const COL_EXIT: (i32, i32, i32) = (120, 255, 170);
const COL_ORB: (i32, i32, i32) = (120, 200, 255);

fn c3(c: (i32, i32, i32)) -> Color {
    rgb(c.0, c.1, c.2)
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Setup,
    Play,
    Finish,
}

/// `LabyrinthGame`
pub struct LabyrinthGame {
    b: GameBase,
    view: &'static str,
    sens: f64,
    cursor: i32,
    small: Font,
    tiny: Font,
    big: Font,
    huge: Font,
    lv_font: Font,
    capture: bool,
    show_map: bool,
    solved: Vec<i32>,
    level: i32,
    state: State,
    view_rect: Rect,
    sens_minus: Rect,
    sens_plus: Rect,
    sens_box: Rect,
    lv_cell: i32,
    lv_x: i32,
    lv_y: i32,
    start_rect: Rect,
    grid: Vec<Vec<u8>>,
    n: i32,
    exit_pos: (i32, i32),
    orbs: Vec<(i32, i32)>,
    orbs_total: usize,
    par: f64,
    px: f64,
    py: f64,
    yaw: f64,
    elapsed: f64,
    got: i64,
    keys: HashSet<&'static str>,
    time_bonus: i64,
    orb_bonus: i64,
    level_score: i64,
}

impl LabyrinthGame {
    /// `LabyrinthGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let mut g = LabyrinthGame {
            b,
            view: "ego",
            sens: 1.0,
            cursor: 1,
            small: f.clone(),
            tiny: f.clone(),
            big: f.clone(),
            huge: f.clone(),
            lv_font: f,
            capture: false,
            show_map: false,
            solved: Vec::new(),
            level: 1,
            state: State::Setup,
            view_rect: z,
            sens_minus: z,
            sens_plus: z,
            sens_box: z,
            lv_cell: 18,
            lv_x: 0,
            lv_y: 0,
            start_rect: z,
            grid: Vec::new(),
            n: 0,
            exit_pos: (1, 1),
            orbs: Vec::new(),
            orbs_total: 0,
            par: 0.0,
            px: 1.5,
            py: 1.5,
            yaw: 0.0,
            elapsed: 0.0,
            got: 0,
            keys: HashSet::new(),
            time_bonus: 0,
            orb_bonus: 0,
            level_score: 0,
        };
        g.reset();
        Box::new(g)
    }

    fn make_fonts(&mut self) {
        let h = self.b.height;
        self.small = ui::f(13.max(h / 30));
        self.tiny = ui::f(11.max(h / 36));
        self.big = ui::fb(16.max(h / 21));
        self.huge = ui::fb(26.max(h / 11));
    }

    fn dim(&self, s: &mut Surface, c: (i32, i32, i32), alpha: u8) {
        let mut surf = pg::surface(self.b.width, self.b.height);
        surf.fill_c(c3(c));
        surf.set_alpha(Some(alpha));
        s.bl(&surf, (0, 0));
    }

    fn save_setting(&mut self, key: &str, value: Value) {
        self.b.set_opt("maze", key, value);
    }

    fn load_solved(&mut self) {
        let data = store::load_section("maze");
        let mut out: Vec<i32> = Vec::new();
        if let Some(Value::Array(lst)) = data.get("solved") {
            for v in lst {
                // isinstance(v, int) - auch True/False zaehlen in Python als int.
                let n = match v {
                    Value::Number(n) if n.is_i64() || n.is_u64() => n.as_i64(),
                    Value::Bool(b) => Some(*b as i64),
                    _ => None,
                };
                if let Some(n) = n {
                    if (1..=LEVELS as i64).contains(&n) && !out.contains(&(n as i32)) {
                        out.push(n as i32);
                    }
                }
            }
        }
        out.sort();
        self.solved = out;
    }

    fn mark_solved(&mut self, n: i32) {
        if !self.solved.contains(&n) {
            self.solved.push(n);
            self.solved.sort();
            let mut sec = Map::new();
            sec.insert("solved".into(), Value::from(self.solved.clone()));
            store::save_section("maze", sec);
        }
    }

    fn build_setup_layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let y0 = (h as f64 * 0.22) as i32;
        let rx = cx + 24;
        self.view_rect = Rect::new(rx - 60, y0, 220, 40);
        self.sens_minus = Rect::new(rx - 60, y0 + 52, 44, 40);
        self.sens_plus = Rect::new(rx + 116, y0 + 52, 44, 40);
        self.sens_box = Rect::new(rx - 8, y0 + 52, 116, 40);
        let top = y0 + 110;
        let avail_h = h - top - 96;
        let cell = 18.max(py::fdiv32(w - 80, 10).min(py::fdiv32(avail_h, 5)));
        self.lv_cell = cell;
        self.lv_x = cx - cell * 5;
        self.lv_y = top;
        self.lv_font = ui::f(9.max(py::fdiv32(cell * 2, 5)));
        self.start_rect = Rect::new(cx - 95, top + 5 * cell + 12, 190, 44);
    }

    fn level_at(&self, pos: (i32, i32)) -> Option<i32> {
        let c = py::fdiv32(pos.0 - self.lv_x, self.lv_cell);
        let r = py::fdiv32(pos.1 - self.lv_y, self.lv_cell);
        if (0..10).contains(&c) && (0..5).contains(&r) {
            Some(r * 10 + c + 1)
        } else {
            None
        }
    }

    fn handle_setup(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown {
            let k = ev.key.as_str();
            match k {
                "v" | "V" => self.toggle_view(),
                "Left" | "a" | "A" => {
                    self.cursor = py::pmod32(self.cursor - 2, LEVELS) + 1;
                    self.b.play_sound("move");
                }
                "Right" | "d" | "D" => {
                    self.cursor = py::pmod32(self.cursor, LEVELS) + 1;
                    self.b.play_sound("move");
                }
                "Up" | "w" | "W" => {
                    self.cursor = py::pmod32(self.cursor - 11, LEVELS) + 1;
                    self.b.play_sound("move");
                }
                "Down" | "s" | "S" => {
                    self.cursor = py::pmod32(self.cursor + 9, LEVELS) + 1;
                    self.b.play_sound("move");
                }
                "Return" | "space" => self.start_level(self.cursor),
                _ => {}
            }
        } else if ev.kind == Kind::MouseDown {
            if self.view_rect.hit(ev.pos) {
                self.toggle_view();
                return;
            }
            if self.sens_minus.hit(ev.pos) {
                self.sens = py::round_to(0.5f64.max(self.sens - 0.1), 1);
                self.save_setting("sens", self.sens.into());
                self.b.play_sound("select");
                return;
            }
            if self.sens_plus.hit(ev.pos) {
                self.sens = py::round_to(2.0f64.min(self.sens + 0.1), 1);
                self.save_setting("sens", self.sens.into());
                self.b.play_sound("select");
                return;
            }
            if let Some(lv) = self.level_at(ev.pos) {
                self.start_level(lv);
                return;
            }
            if self.start_rect.hit(ev.pos) {
                self.start_level(self.cursor);
            }
        }
    }

    fn toggle_view(&mut self) {
        self.view = if self.view == "ego" { "top" } else { "ego" };
        self.save_setting("view", self.view.into());
        self.b.play_sound("select");
    }

    fn start_level(&mut self, level: i32) {
        self.level = level.clamp(1, LEVELS);
        self.cursor = self.level;
        self.save_setting("last_level", (self.level as i64).into());
        let (cells, orbs, par) = maze_gen::level_params(self.level);
        let mut rng = py::Random::new((4400 + self.level) as u64);
        self.grid = maze_gen::generate(cells, &mut rng);
        self.n = self.grid.len() as i32;
        let (exit_pos, dist) = maze_gen::far_exit(&self.grid);
        self.exit_pos = exit_pos;
        self.orbs = maze_gen::place_orbs(&dist, exit_pos, orbs, &mut rng);
        self.orbs_total = self.orbs.len();
        self.par = par;
        self.px = 1.5;
        self.py = 1.5;
        self.yaw = 0.0;
        self.elapsed = 0.0;
        self.got = 0;
        self.keys.clear();
        self.show_map = false;
        self.state = State::Play;
        self.capture = self.view == "ego";
        self.b.play_sound("level");
    }

    fn wall(&self, x: f64, y: f64) -> bool {
        let (ix, iy) = (x as i32, y as i32);
        if 0 <= ix && ix < self.n && 0 <= iy && iy < self.n {
            return self.grid[iy as usize][ix as usize] == 1;
        }
        true
    }

    fn blocked(&self, x: f64, y: f64) -> bool {
        let r = PLAYER_R;
        self.wall(x - r, y - r) || self.wall(x + r, y - r) || self.wall(x - r, y + r) || self.wall(x + r, y + r)
    }

    fn mv(&mut self, dx: f64, dy: f64, dt: f64) {
        let l = py::hypot(dx, dy);
        if l < 1e-9 {
            return;
        }
        let (dx, dy) = (dx / l * MOVE_SPEED * dt, dy / l * MOVE_SPEED * dt);
        let nx = self.px + dx;
        if !self.blocked(nx, self.py) {
            self.px = nx;
        }
        let ny = self.py + dy;
        if !self.blocked(self.px, ny) {
            self.py = ny;
        }
    }

    fn finish(&mut self) {
        self.capture = false;
        self.time_bonus = 0.max(((self.par - self.elapsed) * 8.0) as i64);
        self.orb_bonus = self.got * 100;
        self.level_score = 500 + self.orb_bonus + self.time_bonus;
        self.b.score += self.level_score;
        self.mark_solved(self.level);
        self.state = State::Finish;
        if self.elapsed < 30.0 {
            self.b.ach_event("maze_swift", None);
        }
        self.b.report_result(true);
        self.b.play_sound("win");
    }

    fn draw_ego(&self, s: &mut Surface) {
        let (w, h) = (self.b.width, self.b.height);
        let half = h / 2;
        draw::rect(s, COL_CEIL_TOP, (0, 0, w, half / 2), 0);
        draw::rect(s, COL_CEIL_BOT, (0, half / 2, w, half - half / 2), 0);
        draw::rect(s, COL_FLOOR, (0, half, w, h - half), 0);

        let a = py::radians(self.yaw);
        let (dirx, diry) = (a.cos(), a.sin());
        let (planex, planey) = (-diry * 0.66, dirx * 0.66);
        let step_px = 2;
        let ncols = (w / step_px + 1) as usize;
        let mut zbuf = vec![1e9f64; ncols];
        let tick = pg::ticks() as f64 / 1000.0;
        let exit_pulse = 0.75 + 0.25 * (tick * 4.0).sin();

        for (col, zb) in zbuf.iter_mut().enumerate() {
            let x = col as i32 * step_px;
            let camx = 2.0 * x as f64 / 1.max(w) as f64 - 1.0;
            let rdx = dirx + planex * camx;
            let rdy = diry + planey * camx;
            let (mut mapx, mut mapy) = (self.px as i32, self.py as i32);
            let ddx = if rdx != 0.0 { (1.0 / rdx).abs() } else { 1e30 };
            let ddy = if rdy != 0.0 { (1.0 / rdy).abs() } else { 1e30 };
            let (stepx, mut sdx) = if rdx < 0.0 { (-1, (self.px - mapx as f64) * ddx) } else { (1, (mapx as f64 + 1.0 - self.px) * ddx) };
            let (stepy, mut sdy) = if rdy < 0.0 { (-1, (self.py - mapy as f64) * ddy) } else { (1, (mapy as f64 + 1.0 - self.py) * ddy) };
            let mut side = 0;
            let mut hit = false;
            for _ in 0..4 * self.n {
                if sdx < sdy {
                    sdx += ddx;
                    mapx += stepx;
                    side = 0;
                } else {
                    sdy += ddy;
                    mapy += stepy;
                    side = 1;
                }
                if mapx < 0 || mapy < 0 || mapx >= self.n || mapy >= self.n {
                    hit = true;
                    break;
                }
                if self.grid[mapy as usize][mapx as usize] == 1 {
                    hit = true;
                    break;
                }
            }
            if !hit {
                continue;
            }
            let perp = if side == 0 { sdx - ddx } else { sdy - ddy };
            let perp = 0.02f64.max(perp);
            *zb = perp;
            let line_h = (h as f64 / perp) as i32;
            let y0 = 0.max(half - py::fdiv32(line_h, 2));
            let y1 = h.min(half + py::fdiv32(line_h, 2));
            let mut col_rgb = COL_WALL;
            if (mapx, mapy) != self.exit_pos {
                let (ex, ey) = self.exit_pos;
                if (mapx - ex).abs() + (mapy - ey).abs() == 1 {
                    let f = |c: i32| (c as f64 * exit_pulse) as i32;
                    col_rgb = (f(COL_EXIT.0), f(COL_EXIT.1), f(COL_EXIT.2));
                }
            }
            let shade = if side == 1 { 0.8 } else { 1.0 };
            let fog = 1.0f64.min(perp / FOG_DIST);
            let mix = |c: i32, bg: i32| ((c as f64 * shade) * (1.0 - fog) + bg as f64 * fog) as i32;
            let farbe = rgb(mix(col_rgb.0, COL_BG.0), mix(col_rgb.1, COL_BG.1), mix(col_rgb.2, COL_BG.2));
            draw::rect(s, farbe, (x, y0, step_px, y1 - y0), 0);
        }

        let mut sprites: Vec<(f64, f64, (i32, i32, i32), f64)> =
            self.orbs.iter().map(|o| (o.0 as f64 + 0.5, o.1 as f64 + 0.5, COL_ORB, 0.30)).collect();
        sprites.push((self.exit_pos.0 as f64 + 0.5, self.exit_pos.1 as f64 + 0.5, COL_EXIT, 0.45));
        let det = planex * diry - dirx * planey;
        let inv = 1.0 / if det != 0.0 { det } else { 1e-9 };
        let mut order: Vec<(f64, f64, (i32, i32, i32), f64)> = Vec::new();
        for (sx, sy, colr, size) in sprites {
            let (rx, ry) = (sx - self.px, sy - self.py);
            let tx = inv * (diry * rx - dirx * ry);
            let ty = inv * (-planey * rx + planex * ry);
            if ty <= 0.1 {
                continue;
            }
            order.push((ty, tx, colr, size));
        }
        order.sort_by(|a, b| (-a.0).partial_cmp(&(-b.0)).unwrap_or(std::cmp::Ordering::Equal));
        for (ty, tx, colr, size) in order {
            let sx_px = ((w as f64 / 2.0) * (1.0 + tx / ty)) as i32;
            let r_px = 2.max((h as f64 * size / ty) as i32);
            let mut free = 0;
            for off in [py::fdiv32(-r_px, 2), 0, py::fdiv32(r_px, 2)] {
                let c = py::fdiv32(sx_px + off, step_px);
                if 0 <= c && (c as usize) < ncols && zbuf[c as usize] > ty {
                    free += 1;
                }
            }
            if free < 2 {
                continue;
            }
            let fog = 1.0f64.min(ty / FOG_DIST);
            let m = |c: i32, bg: i32| (c as f64 * (1.0 - fog) + bg as f64 * fog) as i32;
            let cc = (m(colr.0, COL_BG.0), m(colr.1, COL_BG.1), m(colr.2, COL_BG.2));
            let cy = half + (h as f64 * 0.12 / ty) as i32;
            let pulse = 1.0 + 0.12 * (tick * 5.0 + tx).sin();
            let rr = 2.max(py::fdiv32((r_px as f64 * pulse) as i32, 2));
            draw::circle(s, c3(cc), (sx_px, cy), rr, 0);
            let hell = rgb(255.min(cc.0 + 60), 255.min(cc.1 + 60), 255.min(cc.2 + 60));
            draw::circle(s, hell, (sx_px, cy), rr, 1.max(rr / 4));
        }
    }

    fn draw_minimap(&self, s: &mut Surface) {
        let ts = 2.max(6.min(180 / self.n));
        let size = ts * self.n;
        let mut mm = pg::surface_alpha(size, size);
        mm.fill_c(rgba(10, 10, 16, 185));
        for y in 0..self.n {
            for x in 0..self.n {
                if self.grid[y as usize][x as usize] == 1 {
                    mm.fill_r(rgba(90, 100, 140, 220), (x * ts, y * ts, ts, ts));
                }
            }
        }
        let (ex, ey) = self.exit_pos;
        mm.fill_r(pg::alpha(c3(COL_EXIT), 255), (ex * ts, ey * ts, ts, ts));
        for o in &self.orbs {
            mm.fill_r(pg::alpha(c3(COL_ORB), 255), (o.0 * ts, o.1 * ts, ts, ts));
        }
        let (px, py_) = ((self.px * ts as f64) as i32, (self.py * ts as f64) as i32);
        draw::circle(&mut mm, rgb(255, 220, 120), (px, py_), 2.max(ts / 2), 0);
        let a = py::radians(self.yaw);
        let ende = (px + (a.cos() * ts as f64 * 2.0) as i32, py_ + (a.sin() * ts as f64 * 2.0) as i32);
        draw::line(&mut mm, rgb(255, 220, 120), (px, py_), ende, 1);
        s.bl(&mm, (self.b.width - size - 12, 78));
    }

    fn draw_top(&self, s: &mut Surface) {
        let (w, h) = (self.b.width, self.b.height);
        s.fill_c(c3(COL_BG));
        let avail = (w - 20).min(h - 56);
        let mut ts = py::fdiv32(avail, self.n);
        let tick = pg::ticks() as f64 / 1000.0;
        let (ox, oy, x0, y0, x1, y1);
        if ts >= 5 {
            ox = py::fdiv32(w - ts * self.n, 2);
            oy = 44 + py::fdiv32(h - 44 - ts * self.n, 2);
            (x0, y0, x1, y1) = (0, 0, self.n, self.n);
        } else {
            ts = 22;
            ox = w / 2 - (self.px * ts as f64) as i32;
            oy = h / 2 - (self.py * ts as f64) as i32;
            x0 = 0.max(self.px as i32 - w / (2 * ts) - 2);
            x1 = self.n.min(self.px as i32 + w / (2 * ts) + 3);
            y0 = 0.max(self.py as i32 - h / (2 * ts) - 2);
            y1 = self.n.min(self.py as i32 + h / (2 * ts) + 3);
        }
        for y in y0..y1 {
            for x in x0..x1 {
                let r = Rect::new(ox + x * ts, oy + y * ts, ts, ts);
                if self.grid[y as usize][x as usize] == 1 {
                    draw::rect(s, rgb(52, 62, 96), r, 0);
                    draw::rect(s, rgb(34, 40, 64), r, 1);
                }
            }
        }
        let (ex, ey) = self.exit_pos;
        let pulse = 0.7 + 0.3 * (tick * 4.0).sin();
        let p = |c: i32| (c as f64 * pulse) as i32;
        draw::rect(s, rgb(p(COL_EXIT.0), p(COL_EXIT.1), p(COL_EXIT.2)), (ox + ex * ts, oy + ey * ts, ts, ts), 0);
        for o in &self.orbs {
            let cx = ox + ((o.0 as f64 + 0.5) * ts as f64) as i32;
            let cy = oy + ((o.1 as f64 + 0.5) * ts as f64) as i32;
            let rr = 2.max((ts as f64 * 0.28 * (1.0 + 0.15 * (tick * 5.0).sin())) as i32);
            draw::circle(s, c3(COL_ORB), (cx, cy), rr, 0);
        }
        let px = ox as f64 + self.px * ts as f64;
        let py_ = oy as f64 + self.py * ts as f64;
        draw::circle(s, rgb(255, 220, 120), (px as i32, py_ as i32), 3.max((ts as f64 * 0.3) as i32), 0);
    }

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        let img = self.big.render(&tf("common.points", &[("score", &self.b.score.to_string())]), true, self.b.accent);
        s.bl(&img, (14, 8));
        let lines = [
            tf("maze.level", &[("n", &self.level.to_string())]),
            tf("maze.orbs", &[("n", &self.got.to_string()), ("m", &self.orbs_total.to_string())]),
            tf("maze.time", &[("s", &(self.elapsed as i64).to_string())]),
        ];
        let mut y = 10;
        for line in &lines {
            let img = self.small.render(line, true, p.text_dim);
            s.blr(&img, img.get_rect().topright_at((w - 14, y)));
            y += 20;
        }
        if self.view == "ego" && self.state == State::Play && !self.show_map {
            let hint = self.tiny.render(&t("maze.map_hint"), true, p.text_faint);
            s.blr(&hint, hint.get_rect().midbottom_at((w / 2, self.b.height - 6)));
        }
    }

    fn draw_finish(&self, s: &mut Surface) {
        let p = ui::pal();
        self.dim(s, (8, 14, 12), 190);
        let (cx, cy) = (self.b.width / 2, self.b.height / 2);
        let head = self.huge.render(&tf("maze.level_done", &[("n", &self.level.to_string())]), true, c3(COL_EXIT));
        let lines = [
            tf("maze.base", &[("n", "500")]),
            tf("maze.orb_bonus", &[("n", &self.orb_bonus.to_string())]),
            tf("maze.time_bonus", &[("n", &self.time_bonus.to_string())]),
            tf("common.points", &[("score", &self.b.score.to_string())]),
        ];
        let imgs: Vec<Surface> = lines.iter().map(|l| self.b.font.render(l, true, p.text)).collect();
        let hint = self.small.render(&t("maze.next"), true, p.text_dim);
        let top = cy - 70 - head.h() / 2 - 22;
        let bottom = cy - 20 + 30 * imgs.len() as i32 + 12 + hint.h() / 2 + 22;
        let widest = imgs.iter().map(|i| i.w()).max().unwrap_or(0);
        let pw = (self.b.width - 40).min(400.max(head.w() + 80).max(hint.w() + 60).max(widest + 60));
        let panel = Rect::new(cx - py::fdiv32(pw, 2), top, pw, bottom - top);
        draw::rect_r(s, p.panel, panel, 0, 14);
        draw::rect_r(s, p.border_light, panel, 1, 14);
        s.blr(&head, head.get_rect().center_at((cx, cy - 70)));
        let mut y = cy - 20;
        for img in &imgs {
            s.blr(img, img.get_rect().center_at((cx, y)));
            y += 30;
        }
        s.blr(&hint, hint.get_rect().center_at((cx, y + 12)));
    }

    fn draw_setup(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        ui::draw_background(s, w, h);
        let cx = w / 2;
        let title = self.huge.render(&t("maze.title"), true, self.b.accent);
        s.blr(&title, title.get_rect().center_at((cx, (h as f64 * 0.09) as i32)));
        let sub = self.small.render(&t("maze.subtitle"), true, p.text_dim);
        s.blr(&sub, sub.get_rect().center_at((cx, (h as f64 * 0.15) as i32)));

        let lbl = self.small.render(&t("maze.view"), true, p.text_dim);
        s.blr(&lbl, lbl.get_rect().midright_at((self.view_rect.x - 16, self.view_rect.centery())));
        draw::rect_r(s, p.btn_sel, self.view_rect, 0, 8);
        draw::rect_r(s, p.border, self.view_rect, 1, 8);
        let img = self.small.render(&format!("{}  [V]", t(&format!("maze.view.{}", self.view))), true, p.text);
        s.blr(&img, img.get_rect().center_at(self.view_rect.center()));

        let lbl = self.small.render(&t("maze.sens"), true, p.text_dim);
        s.blr(&lbl, lbl.get_rect().midright_at((self.sens_minus.x - 16, self.sens_minus.centery())));
        for (r, sym) in [(self.sens_minus, "-"), (self.sens_plus, "+")] {
            draw::rect_r(s, p.btn, r, 0, 8);
            draw::rect_r(s, p.border, r, 1, 8);
            let img = self.big.render(sym, true, p.text);
            s.blr(&img, img.get_rect().center_at(r.center()));
        }
        let img = self.big.render(&format!("{:.1}x", self.sens), true, self.b.accent);
        s.blr(&img, img.get_rect().center_at(self.sens_box.center()));

        let done_fill = ui::mix(p.btn, self.b.accent, 0.25);
        let done_text = ui::mix(self.b.accent, p.text, 0.35);
        for n in 1..=LEVELS {
            let i = n - 1;
            let x = self.lv_x + (i % 10) * self.lv_cell;
            let y = self.lv_y + (i / 10) * self.lv_cell;
            let cell = Rect::new(x + 1, y + 1, self.lv_cell - 2, self.lv_cell - 2);
            let done = self.solved.contains(&n);
            draw::rect_r(s, if done { done_fill } else { p.btn }, cell, 0, 4);
            if n == self.cursor {
                draw::rect_r(s, self.b.accent, cell, 2, 4);
            }
            let num = self.lv_font.render(&n.to_string(), true, if done { done_text } else { p.text_dim });
            s.blr(&num, num.get_rect().center_at(cell.center()));
        }
        let prog = self.small.render(
            &tf("maze.progress", &[("n", &self.solved.len().to_string()), ("m", &LEVELS.to_string())]),
            true,
            p.text_dim,
        );
        s.blr(&prog, prog.get_rect().center_at((cx, self.lv_y + 5 * self.lv_cell + 34)));
        draw::rect_r(s, p.btn_sel, self.start_rect, 0, 10);
        draw::rect_r(s, self.b.accent, self.start_rect, 2, 10);
        let st = self.b.font.render(&t("common.start"), true, p.text);
        s.blr(&st, st.get_rect().center_at(self.start_rect.center()));
    }

    fn key_action(&self, k: &str) -> Vec<&'static str> {
        ["up", "down", "left", "right"]
            .into_iter()
            .filter(|act| {
                let cap = match *act {
                    "up" => "Up",
                    "down" => "Down",
                    "left" => "Left",
                    _ => "Right",
                };
                self.b.is_action(k, act, None) || k == cap
            })
            .collect()
    }
}

impl Game for LabyrinthGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        self.b.score = 0;
        self.b.game_over = false;
        let v = self.b.opt_str("maze", "view", "ego");
        self.view = if v == "top" { "top" } else { "ego" };
        self.sens = match self.b.opt_value("maze", "sens") {
            None => 1.0,
            Some(Value::Number(n)) => n.as_f64().unwrap_or(1.0).clamp(0.5, 2.0),
            Some(Value::Bool(b)) => (b as i64 as f64).clamp(0.5, 2.0),
            Some(Value::String(st)) => st.trim().parse::<f64>().map(|f| f.clamp(0.5, 2.0)).unwrap_or(1.0),
            Some(_) => 1.0,
        };
        self.cursor = match self.b.opt_value("maze", "last_level") {
            None => 1,
            Some(Value::Number(n)) => n.as_f64().map(|f| (f as i64).clamp(1, LEVELS as i64) as i32).unwrap_or(1),
            Some(Value::Bool(b)) => (b as i32).clamp(1, LEVELS),
            Some(Value::String(st)) => st.trim().parse::<i64>().map(|v| v.clamp(1, LEVELS as i64) as i32).unwrap_or(1),
            Some(_) => 1,
        };
        self.make_fonts();
        self.capture = false;
        self.show_map = false;
        self.load_solved();
        self.level = 1;
        self.build_setup_layout();
        self.state = State::Setup;
    }

    fn on_surface_changed(&mut self) {
        self.make_fonts();
        self.build_setup_layout();
    }

    fn capture_mouse(&self) -> bool {
        self.capture
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        match self.state {
            State::Setup => {
                self.handle_setup(ev);
                return;
            }
            State::Finish => {
                if ev.kind == Kind::KeyDown {
                    if ev.key_in(&["Return", "space"]) {
                        self.start_level(LEVELS.min(self.level + 1));
                    } else if ev.key_in(&["s", "S"]) {
                        self.state = State::Setup;
                        self.capture = false;
                        self.build_setup_layout();
                    }
                }
                return;
            }
            State::Play => {}
        }
        match ev.kind {
            Kind::KeyDown => {
                let k = ev.key.as_str();
                for act in self.key_action(k) {
                    self.keys.insert(act);
                }
                if matches!(k, "q" | "Q") {
                    self.keys.insert("turn_l");
                } else if matches!(k, "e" | "E") {
                    self.keys.insert("turn_r");
                } else if matches!(k, "m" | "M") && self.view == "ego" {
                    self.show_map = !self.show_map;
                    self.b.play_sound("select");
                }
            }
            Kind::KeyUp => {
                let k = ev.key.as_str();
                for act in self.key_action(k) {
                    self.keys.remove(act);
                }
                if matches!(k, "q" | "Q") {
                    self.keys.remove("turn_l");
                } else if matches!(k, "e" | "E") {
                    self.keys.remove("turn_r");
                }
            }
            Kind::MouseRel if self.view == "ego" => {
                self.yaw += ev.rel.0 as f64 * DEG_PER_PX * self.sens;
            }
            _ => {}
        }
    }

    fn update(&mut self, dt: f64) {
        if self.state != State::Play {
            return;
        }
        self.elapsed += dt;
        if self.keys.contains("turn_l") {
            self.yaw -= TURN_KEY * dt;
        }
        if self.keys.contains("turn_r") {
            self.yaw += TURN_KEY * dt;
        }
        let on = |k: &str| if self.keys.contains(k) { 1 } else { 0 };
        let fwd = on("up") - on("down");
        let side = on("right") - on("left");
        if self.view == "ego" {
            let a = py::radians(self.yaw);
            let dx = a.cos() * fwd as f64 + (a + py::PI / 2.0).cos() * side as f64;
            let dy = a.sin() * fwd as f64 + (a + py::PI / 2.0).sin() * side as f64;
            if fwd != 0 || side != 0 {
                self.mv(dx, dy, dt);
            }
        } else if fwd != 0 || side != 0 {
            self.mv(side as f64, -fwd as f64, dt);
        }
        if let Some(i) = self.orbs.iter().position(|o| py::hypot(self.px - (o.0 as f64 + 0.5), self.py - (o.1 as f64 + 0.5)) < 0.5) {
            self.orbs.remove(i);
            self.got += 1;
            self.b.play_sound("powerup");
        }
        if (self.px as i32, self.py as i32) == self.exit_pos {
            self.finish();
        }
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        if self.state == State::Setup {
            self.draw_setup(&mut s);
            return;
        }
        if self.view == "ego" {
            self.draw_ego(&mut s);
            if self.show_map {
                self.draw_minimap(&mut s);
            }
        } else {
            self.draw_top(&mut s);
        }
        self.draw_hud(&mut s);
        if self.state == State::Finish {
            self.draw_finish(&mut s);
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "maze",
    class_name: "LabyrinthGame",
    supports_multiplayer: false,
    modes: &[],
    make: LabyrinthGame::new,
};
