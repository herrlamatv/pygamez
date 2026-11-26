//! airhockey.rs
//! ============
//! Air Hockey - gegen die KI oder zu zweit.
//!
//! Uebertragen aus `games/airhockey.py`: runde Schlaeger und Puck mit
//! Impulsuebertragung, Banden mit Restitution, Reibung, zwei Physik-
//! Unterschritte, Tore als Oeffnungen, Maussteuerung im Einzelspieler,
//! KI in drei Stufen, Power-Ups (XL, TOR, >>), Leuchtspur und Partikel.

use pgz_engine::prelude::*;

const COL_TABLE: Color = Color::rgb(21, 27, 44);
const COL_TABLE_DARK: Color = Color::rgb(10, 13, 22);
const COL_TABLE_LINE: Color = Color::rgb(46, 58, 88);
const COL_BORDER: Color = Color::rgb(90, 110, 150);
const COL_P1: Color = Color::rgb(120, 230, 160);
const COL_P2: Color = Color::rgb(140, 195, 255);
const COL_PUCK: Color = Color::rgb(245, 205, 90);

const PUCK_R: f64 = 12.0;
const MALLET_R: f64 = 26.0;
const MALLET_SPEED: f64 = 420.0;
const PUCK_MAX_SPEED: f64 = 980.0;
const WALL_REST: f64 = 0.92;
const FRICTION: f64 = 0.28;
const GOAL_H_FRAC: f64 = 0.34;
const POWERUP_EVERY: f64 = 9.0;
const POWERUP_DUR: f64 = 8.0;
const POWERUP_R: i32 = 15;
const FREEZE_AFTER_GOAL: f64 = 1.4;
const GOALS_CHOICES: [i64; 4] = [3, 5, 7, 10];

struct AiLevel {
    key: &'static str,
    speed: f64,
    noise: f64,
    defend: f64,
}

const AI_LEVELS: [AiLevel; 3] = [
    AiLevel { key: "easy", speed: 270.0, noise: 42.0, defend: 0.80 },
    AiLevel { key: "medium", speed: 350.0, noise: 18.0, defend: 0.82 },
    AiLevel { key: "hard", speed: 440.0, noise: 6.0, defend: 0.84 },
];

const POWERUPS: [(&str, &str, Color); 3] =
    [("big", "XL", Color::rgb(120, 230, 160)), ("shrink", "TOR", Color::rgb(240, 150, 90)), ("fast", ">>", Color::rgb(140, 195, 255))];

#[derive(Clone, Copy, Default)]
struct Mallet {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
}

/// Wirkende Power-Ups einer Seite (Restzeit je Art, `None` = aus).
#[derive(Clone, Copy, Default)]
struct Effects {
    big: Option<f64>,
    shrink: Option<f64>,
    fast: Option<f64>,
}

impl Effects {
    fn slot(&mut self, key: &str) -> &mut Option<f64> {
        match key {
            "big" => &mut self.big,
            "shrink" => &mut self.shrink,
            _ => &mut self.fast,
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Pressed {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

/// `AirHockeyGame`
pub struct AirHockeyGame {
    b: GameBase,
    winner: usize,
    diff: usize,
    win_goals: i64,
    powerups_on: bool,
    anim_t: f64,
    bg: Option<((i32, i32, Color, Color), Surface)>,
    small: Font,
    tiny: Font,
    fx0: i32,
    fy0: i32,
    fx1: i32,
    fy1: i32,
    cx: i32,
    cy: i32,
    size: (i32, i32),
    diff_panel: Rect,
    diff_left: Rect,
    diff_right: Rect,
    goals_rect: Rect,
    power_rect: Rect,
    start_rect: Rect,
    setup: bool,
    goals: [i64; 2],
    effects: [Effects; 2],
    powerup: Option<(f64, f64, usize)>,
    powerup_timer: f64,
    last_touch: Option<usize>,
    freeze: f64,
    goal_flash: f64,
    goal_flash_side: usize,
    particles: Vec<([f64; 5], Color)>,
    trail: Vec<(f64, f64)>,
    over_t: f64,
    streak: i64,
    mallets: [Mallet; 2],
    pressed: [Pressed; 2],
    mouse_mode: bool,
    mouse_pos: (f64, f64),
    puck_x: f64,
    puck_y: f64,
    puck_vx: f64,
    puck_vy: f64,
}

impl AirHockeyGame {
    /// `AirHockeyGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let mut g = AirHockeyGame {
            b,
            winner: 0,
            diff: 1,
            win_goals: 5,
            powerups_on: true,
            anim_t: 0.0,
            bg: None,
            small: f.clone(),
            tiny: f,
            fx0: 14,
            fy0: 14,
            fx1: 0,
            fy1: 0,
            cx: 0,
            cy: 0,
            size: (0, 0),
            diff_panel: z,
            diff_left: z,
            diff_right: z,
            goals_rect: z,
            power_rect: z,
            start_rect: z,
            setup: true,
            goals: [0, 0],
            effects: [Effects::default(); 2],
            powerup: None,
            powerup_timer: POWERUP_EVERY,
            last_touch: None,
            freeze: 0.0,
            goal_flash: 0.0,
            goal_flash_side: 0,
            particles: Vec::new(),
            trail: Vec::new(),
            over_t: 0.0,
            streak: 0,
            mallets: [Mallet::default(); 2],
            pressed: [Pressed::default(); 2],
            mouse_mode: false,
            mouse_pos: (0.0, 0.0),
            puck_x: 0.0,
            puck_y: 0.0,
            puck_vx: 0.0,
            puck_vy: 0.0,
        };
        g.reset();
        Box::new(g)
    }

    fn make_fonts(&mut self) {
        let h = self.b.height;
        self.b.font = ui::f(16.max(26.min(h / 26)));
        self.b.big_font = ui::fb(30.max(54.min(h / 10)));
        self.small = ui::f(13.max(20.min(h / 32)));
        self.tiny = ui::f(11.max(16.min(h / 42)));
    }

    fn apply_geometry(&mut self) {
        self.make_fonts();
        let (w, h) = (self.b.width, self.b.height);
        self.fx0 = 14;
        self.fy0 = 14;
        self.fx1 = w - 14;
        self.fy1 = h - 14;
        self.cx = w / 2;
        self.cy = h / 2;
        self.size = (w, h);
        self.build_setup_layout();
    }

    fn new_match(&mut self) {
        self.goals = [0, 0];
        self.b.score = 0;
        self.b.game_over = false;
        self.winner = 0;
        self.effects = [Effects::default(); 2];
        self.powerup = None;
        self.powerup_timer = POWERUP_EVERY;
        self.last_touch = None;
        self.freeze = 0.0;
        self.goal_flash = 0.0;
        self.goal_flash_side = 0;
        self.particles.clear();
        self.trail.clear();
        self.over_t = 0.0;
        self.streak = 0;
        let qh = self.b.height as f64 / 2.0;
        self.mallets = [
            Mallet { x: self.fx0 as f64 + (self.cx - self.fx0) as f64 * 0.35, y: qh, vx: 0.0, vy: 0.0 },
            Mallet { x: self.fx1 as f64 - (self.fx1 - self.cx) as f64 * 0.35, y: qh, vx: 0.0, vy: 0.0 },
        ];
        self.pressed = [Pressed::default(); 2];
        self.mouse_mode = false;
        self.mouse_pos = (self.mallets[0].x, self.mallets[0].y);
        self.center_puck(None);
    }

    fn center_puck(&mut self, side: Option<usize>) {
        match side {
            None => {
                self.puck_x = self.cx as f64;
                self.puck_y = self.cy as f64;
            }
            Some(s) => {
                let off = (self.cx - self.fx0) as f64 * 0.5;
                self.puck_x = if s == 0 { self.cx as f64 - off } else { self.cx as f64 + off };
                self.puck_y = self.cy as f64;
            }
        }
        self.puck_vx = 0.0;
        self.puck_vy = 0.0;
        self.trail.clear();
    }

    fn mallet_r(&self, side: usize) -> f64 {
        MALLET_R * if self.effects[side].big.is_some() { 1.35 } else { 1.0 }
    }

    fn mallet_speed(&self, side: usize) -> f64 {
        MALLET_SPEED * if self.effects[side].fast.is_some() { 1.45 } else { 1.0 }
    }

    fn goal_range(&self, side: usize) -> (f64, f64) {
        let mut h = (self.fy1 - self.fy0) as f64 * GOAL_H_FRAC;
        if self.effects[1 - side].shrink.is_some() {
            h *= 0.6;
        }
        (self.cy as f64 - h / 2.0, self.cy as f64 + h / 2.0)
    }

    fn build_setup_layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let bw = 440.min(w - 60);
        let ph = 52.max(64.min((h as f64 * 0.12) as i32));
        let y0 = 108.max((h as f64 * 0.24) as i32);
        let l = cx - py::fdiv32(bw, 2);
        self.diff_panel = Rect::new(l, y0, bw, ph);
        self.diff_left = Rect::new(l, y0, 42, ph);
        self.diff_right = Rect::new(self.diff_panel.right() - 42, y0, 42, ph);
        let bh = 38.max(46.min((h as f64 * 0.085) as i32));
        let gap = 10;
        let y = self.diff_panel.bottom() + 14;
        self.goals_rect = Rect::new(l, y, bw, bh);
        self.power_rect = Rect::new(l, y + bh + gap, bw, bh);
        self.start_rect = Rect::new(cx - 95, y + 2 * (bh + gap) + 8, 190, 50);
    }

    fn cycle_diff(&mut self, step: i64) {
        self.diff = py::pmod(self.diff as i64 + step, 3) as usize;
        self.b.set_opt("airhockey", "difficulty", (self.diff as i64).into());
        self.b.play_sound("click");
    }

    fn cycle_goals(&mut self) {
        let i = GOALS_CHOICES.iter().position(|g| *g == self.win_goals).unwrap_or(1);
        self.win_goals = GOALS_CHOICES[(i + 1) % GOALS_CHOICES.len()];
        self.b.set_opt("airhockey", "goals", self.win_goals.into());
        self.b.play_sound("click");
    }

    fn toggle_powerups(&mut self) {
        self.powerups_on = !self.powerups_on;
        self.b.set_opt("airhockey", "powerups", self.powerups_on.into());
        self.b.play_sound("select");
    }

    fn start_play(&mut self) {
        self.new_match();
        self.setup = false;
        self.b.play_sound("click");
    }

    fn handle_setup_event(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown {
            match ev.key.as_str() {
                "Left" | "a" | "A" => self.cycle_diff(-1),
                "Right" | "d" | "D" => self.cycle_diff(1),
                "t" | "T" => self.cycle_goals(),
                "p" | "P" => self.toggle_powerups(),
                "Return" | "space" => self.start_play(),
                _ => {}
            }
        } else if ev.kind == Kind::MouseDown {
            let p = ev.pos;
            if self.diff_left.hit(p) {
                self.cycle_diff(-1);
            } else if self.diff_right.hit(p) || self.diff_panel.hit(p) {
                self.cycle_diff(1);
            } else if self.goals_rect.hit(p) {
                self.cycle_goals();
            } else if self.power_rect.hit(p) {
                self.toggle_powerups();
            } else if self.start_rect.hit(p) {
                self.start_play();
            }
        }
    }

    fn key_dir(&self, i: usize) -> (i32, i32) {
        let p = self.pressed[i];
        (p.right as i32 - p.left as i32, p.down as i32 - p.up as i32)
    }

    fn move_mallets(&mut self, dt: f64) {
        if self.mouse_mode && !self.b.multiplayer {
            self.mouse_to_mallet(dt);
        } else {
            let (dx, dy) = if self.b.multiplayer {
                self.key_dir(0)
            } else {
                let (d1, d2) = (self.key_dir(0), self.key_dir(1));
                ((d1.0 + d2.0).clamp(-1, 1), (d1.1 + d2.1).clamp(-1, 1))
            };
            let sp = self.mallet_speed(0);
            self.keys_to_mallet(0, dx, dy, sp, dt);
        }
        self.clamp_mallet(0);
        if self.b.multiplayer {
            let (dx, dy) = self.key_dir(1);
            let sp = self.mallet_speed(1);
            self.keys_to_mallet(1, dx, dy, sp, dt);
        } else {
            self.ai_move(dt);
        }
        self.clamp_mallet(1);
    }

    fn keys_to_mallet(&mut self, i: usize, dx: i32, dy: i32, speed: f64, dt: f64) {
        let ln = py::hypot(dx as f64, dy as f64);
        let (tvx, tvy) = if ln > 0.0 { (dx as f64 / ln * speed, dy as f64 / ln * speed) } else { (0.0, 0.0) };
        let k = 1.0f64.min(dt * 12.0);
        let m = &mut self.mallets[i];
        m.vx += (tvx - m.vx) * k;
        m.vy += (tvy - m.vy) * k;
        m.x += m.vx * dt;
        m.y += m.vy * dt;
    }

    fn mouse_to_mallet(&mut self, dt: f64) {
        let (tx, ty) = self.mouse_pos;
        let k = 1.0f64.min(dt * 16.0);
        let m = &mut self.mallets[0];
        let nx = m.x + (tx - m.x) * k;
        let ny = m.y + (ty - m.y) * k;
        if dt > 0.0 {
            m.vx = (nx - m.x) / dt;
            m.vy = (ny - m.y) / dt;
            let sp = py::hypot(m.vx, m.vy);
            if sp > 1300.0 {
                m.vx *= 1300.0 / sp;
                m.vy *= 1300.0 / sp;
            }
        }
        m.x = nx;
        m.y = ny;
    }

    fn clamp_mallet(&mut self, side: usize) {
        let r = self.mallet_r(side);
        let (x0, x1) = if side == 0 {
            (self.fx0 as f64 + r, self.cx as f64 - r)
        } else {
            (self.cx as f64 + r, self.fx1 as f64 - r)
        };
        let (fy0, fy1) = (self.fy0 as f64, self.fy1 as f64);
        let m = &mut self.mallets[side];
        let nx = x0.max(x1.min(m.x));
        let ny = (fy0 + r).max((fy1 - r).min(m.y));
        if nx != m.x {
            m.vx = 0.0;
        }
        if ny != m.y {
            m.vy = 0.0;
        }
        m.x = nx;
        m.y = ny;
    }

    fn ai_move(&mut self, dt: f64) {
        let lvl = &AI_LEVELS[self.diff];
        let speed = lvl.speed * if self.effects[1].fast.is_some() { 1.45 } else { 1.0 };
        let r = self.mallet_r(1);
        let feld_w = (self.fx1 - self.fx0) as f64;
        let m = self.mallets[1];
        let (mut tx, mut ty);
        if self.puck_x > self.cx as f64 && self.freeze <= 0.0 {
            let ziel = (self.fx0 as f64, self.cy as f64);
            let dx = ziel.0 - self.puck_x;
            let dy = ziel.1 - self.puck_y;
            let mut ln = py::hypot(dx, dy);
            if ln == 0.0 {
                ln = 1.0;
            }
            tx = self.puck_x - dx / ln * (r * 0.5);
            ty = self.puck_y - dy / ln * (r * 0.5);
            if self.puck_x > m.x + 4.0 {
                tx = (self.fx1 as f64 - r).min(self.puck_x + r + PUCK_R + 8.0);
                ty = self.puck_y + if self.puck_y < m.y { r + PUCK_R + 10.0 } else { -(r + PUCK_R + 10.0) };
            }
        } else {
            tx = self.fx0 as f64 + feld_w * lvl.defend;
            let (gy0, gy1) = self.goal_range(1);
            ty = (gy0 + 10.0).max((gy1 - 10.0).min(self.puck_y));
        }
        tx += py::uniform(-lvl.noise, lvl.noise);
        ty += py::uniform(-lvl.noise, lvl.noise);
        let (dx, dy) = (tx - m.x, ty - m.y);
        let ln = py::hypot(dx, dy);
        let m = &mut self.mallets[1];
        if ln > 4.0 {
            m.vx = dx / ln * speed;
            m.vy = dy / ln * speed;
            m.x += m.vx * dt;
            m.y += m.vy * dt;
        } else {
            m.vx = 0.0;
            m.vy = 0.0;
        }
    }

    fn move_puck(&mut self, dt: f64) {
        let f = 0f64.max(1.0 - FRICTION * dt);
        self.puck_vx *= f;
        self.puck_vy *= f;
        self.puck_x += self.puck_vx * dt;
        self.puck_y += self.puck_vy * dt;
        let r = PUCK_R;
        let (fx0, fy0, fx1, fy1) = (self.fx0 as f64, self.fy0 as f64, self.fx1 as f64, self.fy1 as f64);
        if self.puck_y - r < fy0 {
            self.puck_y = fy0 + r;
            self.puck_vy = self.puck_vy.abs() * WALL_REST;
            self.wall_sound();
        } else if self.puck_y + r > fy1 {
            self.puck_y = fy1 - r;
            self.puck_vy = -self.puck_vy.abs() * WALL_REST;
            self.wall_sound();
        }
        let (gy0, gy1) = self.goal_range(0);
        if self.puck_x - r < fx0 {
            if gy0 < self.puck_y && self.puck_y < gy1 {
                if self.puck_x + r < fx0 {
                    self.goal_scored(1);
                    return;
                }
            } else {
                self.puck_x = fx0 + r;
                self.puck_vx = self.puck_vx.abs() * WALL_REST;
                self.wall_sound();
            }
        }
        let (gy0, gy1) = self.goal_range(1);
        if self.puck_x + r > fx1 {
            if gy0 < self.puck_y && self.puck_y < gy1 {
                if self.puck_x - r > fx1 {
                    self.goal_scored(0);
                    return;
                }
            } else {
                self.puck_x = fx1 - r;
                self.puck_vx = -self.puck_vx.abs() * WALL_REST;
                self.wall_sound();
            }
        }
        self.collide_mallet(0);
        self.collide_mallet(1);
        let sp = py::hypot(self.puck_vx, self.puck_vy);
        if sp > PUCK_MAX_SPEED {
            self.puck_vx *= PUCK_MAX_SPEED / sp;
            self.puck_vy *= PUCK_MAX_SPEED / sp;
        }
    }

    fn wall_sound(&self) {
        if py::hypot(self.puck_vx, self.puck_vy) > 120.0 {
            self.b.play_sound("bounce");
        }
    }

    fn collide_mallet(&mut self, side: usize) {
        let m = self.mallets[side];
        let r = self.mallet_r(side) + PUCK_R;
        let mut dx = self.puck_x - m.x;
        let mut dy = self.puck_y - m.y;
        let mut dist = py::hypot(dx, dy);
        if dist >= r {
            return;
        }
        if dist < 1e-6 {
            let ang = py::uniform(0.0, py::TAU);
            dx = ang.cos();
            dy = ang.sin();
            dist = 1.0;
        }
        let (nx, ny) = (dx / dist, dy / dist);
        self.puck_x = m.x + nx * (r + 0.5);
        self.puck_y = m.y + ny * (r + 0.5);
        let mut rvx = self.puck_vx - m.vx;
        let mut rvy = self.puck_vy - m.vy;
        let vn = rvx * nx + rvy * ny;
        if vn < 0.0 {
            rvx -= 1.9 * vn * nx;
            rvy -= 1.9 * vn * ny;
        }
        self.puck_vx = m.vx + rvx;
        self.puck_vy = m.vy + rvy;
        self.last_touch = Some(side);
        if py::hypot(self.puck_vx, self.puck_vy) > 520.0 {
            self.spawn_particles(self.puck_x, self.puck_y, if side == 0 { COL_P1 } else { COL_P2 }, 6);
            self.b.play_sound("hit");
            self.b.rumble(60);
        } else {
            self.b.play_sound("bounce");
        }
    }

    fn goal_scored(&mut self, by: usize) {
        self.goals[by] += 1;
        if by == 0 {
            self.b.score = self.goals[0];
        }
        self.goal_flash = 1.0;
        self.goal_flash_side = by;
        let gx = if by == 0 { self.fx1 } else { self.fx0 } as f64;
        self.spawn_particles(gx, self.puck_y, if by == 0 { COL_P1 } else { COL_P2 }, 22);
        self.b.play_sound("point");
        self.b.rumble(150);
        self.streak = if by == 0 { self.streak + 1 } else { 0 };
        if self.streak == 3 {
            self.b.ach_event("air_hattrick", None);
        }
        if self.goals[by] >= self.win_goals {
            self.b.game_over = true;
            self.winner = by;
            self.over_t = self.anim_t;
            let gewonnen = by == 0 || self.b.multiplayer;
            self.b.play_sound(if gewonnen { "win" } else { "gameover" });
            if !self.b.multiplayer {
                self.b.report_result(by == 0);
            }
            self.b.report_duel(by);
            self.b.rumble(250);
            return;
        }
        self.freeze = FREEZE_AFTER_GOAL;
        self.center_puck(Some(1 - by));
        self.last_touch = None;
    }

    fn update_powerups(&mut self, dt: f64) {
        let Some((px, py_, idx)) = self.powerup else {
            self.powerup_timer -= dt;
            if self.powerup_timer <= 0.0 {
                let x = py::uniform((self.cx - 150) as f64, (self.cx + 150) as f64);
                let y = py::uniform((self.fy0 + 70) as f64, (self.fy1 - 70) as f64);
                self.powerup = Some((x, y, py::randrange0(POWERUPS.len() as i64) as usize));
                self.powerup_timer = POWERUP_EVERY;
            }
            return;
        };
        if py::hypot(self.puck_x - px, self.puck_y - py_) < PUCK_R + POWERUP_R as f64 {
            if let Some(lt) = self.last_touch {
                *self.effects[lt].slot(POWERUPS[idx].0) = Some(POWERUP_DUR);
                self.spawn_particles(px, py_, POWERUPS[idx].2, 12);
                self.b.play_sound("powerup");
            }
            self.powerup = None;
        }
    }

    fn spawn_particles(&mut self, x: f64, y: f64, color: Color, n: usize) {
        for _ in 0..n {
            let ang = py::uniform(0.0, py::TAU);
            let spd = py::uniform(40.0, 240.0);
            let life = py::uniform(0.25, 0.6);
            self.particles.push(([x, y, ang.cos() * spd, ang.sin() * spd, life], color));
        }
    }

    fn update_particles(&mut self, dt: f64) {
        for (p, _) in self.particles.iter_mut() {
            p[0] += p[2] * dt;
            p[1] += p[3] * dt;
            p[2] *= 0.98;
            p[3] *= 0.98;
            p[4] -= dt;
        }
        self.particles.retain(|(p, _)| p[4] > 0.0);
    }

    fn bg_surface(&mut self) -> Surface {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let key = (w, h, p.bg_top, p.bg_bottom);
        if self.bg.as_ref().map(|c| c.0) != Some(key) {
            let mut surf = pg::surface(w, h);
            ui::draw_background(&mut surf, w, h);
            self.bg = Some((key, surf));
        }
        self.bg.as_ref().map(|c| c.1.clone()).expect("bg cached")
    }

    fn panel(&self, s: &mut Surface, rect: Rect, alpha: u8, border: i32, border_col: Option<Color>) {
        let p = ui::pal();
        let mut surf = pg::surface_alpha(rect.w, rect.h);
        draw::rect_r(&mut surf, pg::alpha(p.panel, alpha), (0, 0, rect.w, rect.h), 0, 12);
        s.bl(&surf, (rect.x, rect.y));
        draw::rect_r(s, border_col.unwrap_or(self.b.accent), rect, border, 12);
    }

    fn draw_table(&self, s: &mut Surface) {
        let (fx0, fy0, fx1, fy1) = (self.fx0, self.fy0, self.fx1, self.fy1);
        draw::rect_r(s, COL_TABLE, (fx0, fy0, fx1 - fx0, fy1 - fy0), 0, 18);
        let mut y = fy0 + 6;
        while y < fy1 - 6 {
            draw::rect(s, COL_TABLE_LINE, (self.cx - 2, y, 4, 12), 0);
            y += 22;
        }
        draw::circle(s, COL_TABLE_LINE, (self.cx, self.cy), 52, 3);
        draw::circle(s, COL_TABLE_LINE, (self.cx, self.cy), 5, 0);
        draw::rect_r(s, COL_BORDER, (fx0, fy0, fx1 - fx0, fy1 - fy0), 3, 18);
        let pulse = 2 + (2.0 * (self.anim_t * 4.0).sin()) as i32;
        for (side, farbe, x) in [(0usize, COL_P1, fx0), (1, COL_P2, fx1)] {
            let (gy0, gy1) = self.goal_range(side);
            let xf = x as f64;
            draw::rect(s, COL_TABLE_DARK, (xf - 5.0, gy0, 10.0, gy1 - gy0), 0);
            draw::rect_r(s, farbe, (xf - 3.0, gy0, 6.0, gy1 - gy0), 0, 3);
            let gh = (gy1 - gy0) as i32 + 20;
            let mut glow = pg::surface_alpha(16, gh);
            draw::rect_r(&mut glow, pg::alpha(farbe, 60 + pulse * 10), (0, 0, 16, gh), 0, 8);
            s.bl(&glow, (xf - 8.0, gy0 - 10.0));
        }
    }

    fn draw_trail(&self, s: &mut Surface) {
        let n = self.trail.len();
        for (i, &(x, y)) in self.trail.iter().enumerate() {
            let a = (70.0 * (i + 1) as f64 / 1.max(n) as f64) as i32;
            let r = 2.max((PUCK_R * (i + 1) as f64 / 1.max(n) as f64) as i32 - 2);
            let mut surf = pg::surface_alpha(r * 2, r * 2);
            draw::circle(&mut surf, pg::alpha(COL_PUCK, a), (r, r), r, 0);
            s.bl(&surf, (x - r as f64, y - r as f64));
        }
    }

    fn draw_puck(&self, s: &mut Surface) {
        let c = (self.puck_x as i32, self.puck_y as i32);
        draw::circle(s, COL_PUCK, c, PUCK_R as i32, 0);
        draw::circle(s, rgb(255, 240, 190), ((self.puck_x - 3.0) as i32, (self.puck_y - 3.0) as i32), 4, 0);
        draw::circle(s, rgb(120, 95, 30), c, PUCK_R as i32, 2);
    }

    fn draw_mallet(&self, s: &mut Surface, side: usize) {
        let m = self.mallets[side];
        let r = self.mallet_r(side) as i32;
        let farbe = if side == 0 { COL_P1 } else { COL_P2 };
        let dunkel = rgb(farbe.r / 2, farbe.g / 2, farbe.b / 2);
        let c = (m.x as i32, m.y as i32);
        draw::circle(s, dunkel, c, r, 0);
        draw::circle(s, farbe, c, r, 4);
        draw::circle(s, farbe, c, 4.max(r / 3), 0);
    }

    fn draw_powerup(&self, s: &mut Surface) {
        let Some((x, y, idx)) = self.powerup else { return };
        let (_, label, farbe) = POWERUPS[idx];
        let r = POWERUP_R + (2.0 * (self.anim_t * 6.0).sin()) as i32;
        let mut glow = pg::surface_alpha(r * 4, r * 4);
        draw::circle(&mut glow, pg::alpha(farbe, 50), (r * 2, r * 2), r * 2, 0);
        s.bl(&glow, (x - (r * 2) as f64, y - (r * 2) as f64));
        let c = (x as i32, y as i32);
        draw::circle(s, rgb(25, 30, 45), c, r, 0);
        draw::circle(s, farbe, c, r, 2);
        let img = self.tiny.render(label, true, farbe);
        s.blr(&img, img.get_rect().center_at(c));
    }

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let y0 = self.fy0 + 8;
        let g1 = self.b.big_font.render(&self.goals[0].to_string(), true, COL_P1);
        let g2 = self.b.big_font.render(&self.goals[1].to_string(), true, COL_P2);
        s.blr(&g1, g1.get_rect().midtop_at((self.cx - 70, y0)));
        s.blr(&g2, g2.get_rect().midtop_at((self.cx + 70, y0)));
        let mp = self.b.multiplayer;
        let links: String = if mp { self.b.pname(0).chars().take(6).collect() } else { t("ah.you") };
        let rechts: String = if mp { self.b.pname(1).chars().take(6).collect() } else { t("ah.ai") };
        let ly = y0 + self.b.big_font.get_height() + 2;
        let l1 = self.small.render(&links, true, COL_P1);
        let l2 = self.small.render(&rechts, true, COL_P2);
        s.blr(&l1, l1.get_rect().midtop_at((self.cx - 70, ly)));
        s.blr(&l2, l2.get_rect().midtop_at((self.cx + 70, ly)));
        let ziel = self.tiny.render(&tf("ah.first_to", &[("n", &self.win_goals.to_string())]), true, p.text_dim);
        s.blr(&ziel, ziel.get_rect().midtop_at((self.cx, ly + self.small.get_height() + 4)));
        for (side, basex, richtung) in [(0usize, self.fx0 + 12, 1), (1, self.fx1 - 12, -1)] {
            let mut y = self.fy0 + 8;
            let e = self.effects[side];
            // sorted(items()) - alphabetisch: big, fast, shrink
            for (key, rest) in [("big", e.big), ("fast", e.fast), ("shrink", e.shrink)] {
                let Some(restzeit) = rest else { continue };
                for (pk, label, farbe) in POWERUPS {
                    if pk != key {
                        continue;
                    }
                    let text = self.tiny.render(&format!("{label} {restzeit:2.0}s"), true, farbe);
                    let x = if richtung == 1 { basex } else { basex - text.w() };
                    s.bl(&text, (x, y));
                    y += 16;
                }
            }
        }
        if self.goal_flash > 0.0 && !self.b.game_over {
            let farbe = if self.goal_flash_side == 0 { COL_P1 } else { COL_P2 };
            let a = 0.max(255.min((255.0 * self.goal_flash) as i32));
            let mut img = self.b.big_font.render(&t("ah.goal"), true, farbe);
            img.set_alpha(Some(a as u8));
            s.blr(&img, img.get_rect().center_at((self.cx, self.cy - 60)));
        }
        if self.anim_t < 6.0 && !mp {
            let hint = self.tiny.render(&t("ah.mouse_hint"), true, p.text_dim);
            s.blr(&hint, hint.get_rect().midbottom_at((self.cx, self.fy1 - 6)));
        }
    }

    fn draw_game_over(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let mut dim = pg::surface_alpha(w, h);
        dim.fill_c(rgba(8, 10, 16, 150));
        s.bl(&dim, (0, 0));
        let (text, farbe) = if self.b.multiplayer {
            let name: String = self.b.pname(self.winner).chars().take(9).collect();
            (tf("common.player_wins_name", &[("name", &name)]), if self.winner == 0 { COL_P1 } else { COL_P2 })
        } else {
            let gewonnen = self.winner == 0;
            (if gewonnen { t("ah.win") } else { t("ah.lose") }, if gewonnen { COL_P1 } else { p.red })
        };
        self.b.draw_center_text(&text, &self.b.big_font, farbe, -40);
        self.b.draw_center_text(&format!("{} : {}", self.goals[0], self.goals[1]), &self.b.font, p.text, 4);
        let hint_col = ui::mix(p.text_dim, p.text, ui::pulse(2.4, 0.0, 1.0));
        self.b.draw_center_text(&t("ah.restart_hint"), &self.b.font, hint_col, 44);
    }

    fn draw_row(&self, s: &mut Surface, rect: Rect, label: &str, wert: &str, an: bool) {
        let p = ui::pal();
        draw::rect_r(s, if an { p.btn_sel } else { p.btn }, rect, 0, 8);
        draw::rect_r(s, if an { p.border_light } else { p.border }, rect, 1, 8);
        let lab = self.b.font.render(label, true, p.text);
        s.bl(&lab, (rect.x + 16, rect.centery() - lab.h() / 2));
        let img = self.b.font.render(&format!("< {wert} >"), true, if an { self.b.accent } else { p.text_dim });
        s.bl(&img, (rect.right() - img.w() - 16, rect.centery() - img.h() / 2));
    }

    fn draw_setup(&mut self) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let mut s = self.b.sf();
        let bg = self.bg_surface();
        s.bl(&bg, (0, 0));
        let modus = if self.b.multiplayer { t("snake.multiplayer") } else { t("snake.singleplayer") };
        ui::draw_title_ex(&mut s, w, "AIR HOCKEY", Some(&modus), 52, None, None, Some(self.b.accent));
        let lvl = &AI_LEVELS[self.diff];
        let aktiv = !self.b.multiplayer;
        self.panel(&mut s, self.diff_panel, 235, 2, Some(if aktiv { self.b.accent } else { p.border }));
        let col = if aktiv { p.text } else { p.text_dim };
        let name = self.b.font.render(&format!("{}:  {}", t("ah.difficulty"), t(&format!("ah.diff.{}", lvl.key))), true, col);
        let dp = self.diff_panel;
        s.blr(&name, name.get_rect().center_at((dp.centerx(), dp.y + (dp.h as f64 * 0.34) as i32)));
        let info = self.tiny.render(&if aktiv { t("ah.diff_note") } else { t("ah.diff_mp") }, true, p.text_dim);
        s.blr(&info, info.get_rect().center_at((dp.centerx(), dp.y + (dp.h as f64 * 0.72) as i32)));
        let arr_col = if aktiv { ui::mix(self.b.accent, p.text, ui::pulse(3.0, 0.0, 0.3)) } else { p.text_dim };
        for (r, sym) in [(self.diff_left, "<"), (self.diff_right, ">")] {
            let arr = self.b.big_font.render(sym, true, arr_col);
            s.blr(&arr, arr.get_rect().center_at(r.center()));
        }
        self.draw_row(&mut s, self.goals_rect, &t("ah.goals"), &self.win_goals.to_string(), true);
        let on = if self.powerups_on { t("common.on") } else { t("common.off") };
        self.draw_row(&mut s, self.power_rect, &t("ah.powerups"), &on, self.powerups_on);
        ui::draw_button_ex(&mut s, self.start_rect, &t("common.start"), &self.b.font, true, None, None, Some(self.b.accent));
        let h2 = self.tiny.render(&t("ah.mouse_hint"), true, p.green);
        s.blr(&h2, h2.get_rect().center_at((w / 2, self.start_rect.bottom() + 22)));
        ui::draw_footer(&mut s, w, h, &t("ah.setup_hint"));
    }
}

impl Game for AirHockeyGame {
    /// Match-Historie: Dauer zaehlt nicht im Setup (Python-Standard
    /// `getattr(self, "state", None) != "setup"`).
    fn match_active(&self) -> bool {
        !self.setup
    }
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        self.b.score = 0;
        self.b.game_over = false;
        self.winner = 0;
        self.diff = self.b.opt_i64("airhockey", "difficulty", 1).clamp(0, 2) as usize;
        let goals = self.b.opt_i64("airhockey", "goals", 5);
        self.win_goals = if GOALS_CHOICES.contains(&goals) { goals } else { 5 };
        self.powerups_on = self.b.opt_bool("airhockey", "powerups", true);
        self.anim_t = 0.0;
        self.bg = None;
        self.apply_geometry();
        self.setup = true;
        self.new_match();
    }

    fn on_surface_changed(&mut self) {
        let (old_w, old_h) = self.size;
        self.apply_geometry();
        let sx = self.b.width as f64 / 1.max(old_w) as f64;
        let sy = self.b.height as f64 / 1.max(old_h) as f64;
        for side in 0..2 {
            self.mallets[side].x *= sx;
            self.mallets[side].y *= sy;
            self.clamp_mallet(side);
        }
        self.puck_x *= sx;
        self.puck_y *= sy;
        self.mouse_pos = (self.mouse_pos.0 * sx, self.mouse_pos.1 * sy);
        if let Some((px, py_, idx)) = self.powerup {
            self.powerup = Some((px * sx, py_ * sy, idx));
        }
        self.trail.clear();
        self.bg = None;
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if self.setup {
            self.handle_setup_event(ev);
            return;
        }
        if self.b.game_over {
            if ev.kind == Kind::KeyDown {
                if ev.key_in(&["Return", "space"]) {
                    self.start_play();
                } else if ev.key_in(&["s", "S"]) {
                    self.setup = true;
                    self.b.play_sound("click");
                }
            } else if ev.kind == Kind::MouseDown && self.anim_t - self.over_t > 0.5 {
                self.start_play();
            }
            return;
        }
        if ev.kind == Kind::MouseMove {
            if !self.b.multiplayer {
                self.mouse_mode = true;
                self.mouse_pos = (ev.pos.0 as f64, ev.pos.1 as f64);
            }
            return;
        }
        let acts = ["up", "down", "left", "right"];
        if ev.kind == Kind::KeyUp {
            for (i, scheme) in ["p1", "p2"].iter().enumerate() {
                for act in acts {
                    if self.b.is_action(&ev.key, act, Some(scheme)) {
                        set_dir(&mut self.pressed[i], act, false);
                    }
                }
            }
            return;
        }
        if ev.kind != Kind::KeyDown {
            return;
        }
        let mut gedrueckt = false;
        for (i, scheme) in ["p1", "p2"].iter().enumerate() {
            for act in acts {
                if self.b.is_action(&ev.key, act, Some(scheme)) {
                    set_dir(&mut self.pressed[i], act, true);
                    gedrueckt = true;
                }
            }
        }
        if gedrueckt {
            self.mouse_mode = false;
        }
    }

    fn update(&mut self, dt: f64) {
        self.anim_t += dt;
        self.update_particles(dt);
        if self.setup || self.b.game_over {
            return;
        }
        for e in self.effects.iter_mut() {
            for slot in [&mut e.big, &mut e.shrink, &mut e.fast] {
                if let Some(v) = slot.as_mut() {
                    *v -= dt;
                    if *v <= 0.0 {
                        *slot = None;
                    }
                }
            }
        }
        if self.goal_flash > 0.0 {
            self.goal_flash -= dt;
        }
        self.move_mallets(dt);
        if self.freeze > 0.0 {
            self.freeze -= dt;
            return;
        }
        if self.powerups_on {
            self.update_powerups(dt);
        }
        for _ in 0..2 {
            self.move_puck(dt / 2.0);
            if self.freeze > 0.0 {
                break;
            }
        }
        self.trail.push((self.puck_x, self.puck_y));
        if self.trail.len() > 10 {
            self.trail.remove(0);
        }
    }

    fn draw(&mut self) {
        if self.setup {
            self.draw_setup();
            return;
        }
        let mut s = self.b.sf();
        let bg = self.bg_surface();
        s.bl(&bg, (0, 0));
        self.draw_table(&mut s);
        self.draw_powerup(&mut s);
        self.draw_trail(&mut s);
        self.draw_puck(&mut s);
        self.draw_mallet(&mut s, 0);
        self.draw_mallet(&mut s, 1);
        for (p, col) in &self.particles {
            let a = 0.max(255.min((255.0 * (p[4] / 0.6)) as i32));
            let mut surf = pg::surface_alpha(6, 6);
            draw::circle(&mut surf, pg::alpha(*col, a), (3, 3), 3, 0);
            s.bl(&surf, (p[0] - 3.0, p[1] - 3.0));
        }
        self.draw_hud(&mut s);
        if self.b.game_over {
            self.draw_game_over(&mut s);
        }
    }
}

fn set_dir(p: &mut Pressed, act: &str, on: bool) {
    match act {
        "up" => p.up = on,
        "down" => p.down = on,
        "left" => p.left = on,
        _ => p.right = on,
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "airhockey",
    class_name: "AirHockeyGame",
    supports_multiplayer: true,
    modes: &[],
    make: AirHockeyGame::new,
};
