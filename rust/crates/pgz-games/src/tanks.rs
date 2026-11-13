//! tanks.rs
//! ========
//! Panzer-Duell - 2D-Arena (1 Spieler gegen KI oder 2 Spieler lokal).
//!
//! Uebertragen aus `games/tanks.py`: Drehen/Fahren, Abpraller (einmal),
//! Schonfrist fuer eigene Schuesse, fuenf Gewinnrunden, vier Arenen,
//! Power-Ups (Schnellfeuer, Schild, Dreifach), KI in drei Staerken mit
//! Vorhalt und Ricochet-Zielen ueber Wandspiegelung.

use pgz_engine::prelude::*;

const COL_FLOOR: Color = Color::rgb(24, 28, 38);
const COL_WALL: Color = Color::rgb(72, 80, 100);
const COL_WALL_EDGE: Color = Color::rgb(100, 110, 134);
const COL_P1: Color = Color::rgb(120, 180, 90);
const COL_P2: Color = Color::rgb(200, 120, 70);
const COL_BULLET: Color = Color::rgb(240, 240, 250);

#[derive(Clone, Copy, PartialEq)]
enum Power {
    Rapid,
    Shield,
    Triple,
}

fn power_col(p: Power) -> Color {
    match p {
        Power::Rapid => Color::rgb(245, 205, 90),
        Power::Shield => Color::rgb(110, 190, 255),
        Power::Triple => Color::rgb(230, 120, 200),
    }
}

const DIFFS: [&str; 3] = ["easy", "medium", "hard"];
const ARENAS: [&str; 5] = ["random", "open", "cross", "pillars", "maze"];

fn wall_preset(key: &str) -> &'static [(f64, f64, f64, f64)] {
    match key {
        "cross" => &[(0.35, 0.475, 0.30, 0.05), (0.475, 0.35, 0.05, 0.30)],
        "pillars" => &[(0.18, 0.18, 0.14, 0.14), (0.68, 0.18, 0.14, 0.14), (0.18, 0.68, 0.14, 0.14), (0.68, 0.68, 0.14, 0.14)],
        "maze" => &[(0.0, 0.32, 0.42, 0.045), (0.58, 0.32, 0.42, 0.045), (0.30, 0.62, 0.045, 0.38), (0.655, 0.0, 0.045, 0.38)],
        _ => &[],
    }
}

const ROUNDS_TO_WIN: i64 = 5;
const BULLET_SPEED: f64 = 320.0;
const BULLET_LIFE: f64 = 6.0;
const MAX_BULLETS: usize = 4;
const COOLDOWN: f64 = 0.5;
const GRACE: f64 = 0.15;

#[derive(Clone, Copy, PartialEq)]
enum State {
    Setup,
    Countdown,
    Play,
    RoundEnd,
    MatchEnd,
}

#[derive(Clone, Copy, Default)]
struct Keys {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

impl Keys {
    fn set(&mut self, act: &str, on: bool) {
        match act {
            "up" => self.up = on,
            "down" => self.down = on,
            "left" => self.left = on,
            _ => self.right = on,
        }
    }
}

#[derive(Clone, Copy)]
struct Tank {
    x: f64,
    y: f64,
    ang: f64,
    cooldown: f64,
    shield: bool,
    rapid: f64,
    triple: i32,
    alive: bool,
    keys: Keys,
}

#[derive(Clone, Copy)]
struct Bullet {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    owner: usize,
    bounces: i32,
    age: f64,
}

#[derive(Clone, Copy)]
struct PowerUp {
    x: f64,
    y: f64,
    kind: Power,
    age: f64,
}

fn tp(p: (f64, f64)) -> (i32, i32) {
    (p.0 as i32, p.1 as i32)
}

/// `TankDuelGame`
pub struct TankDuelGame {
    b: GameBase,
    diff: usize,
    arena_sel: i64,
    small: Font,
    tiny: Font,
    huge: Font,
    rounds: [i64; 2],
    arena_cycle: Vec<&'static str>,
    setup_rows: Vec<&'static str>,
    row_rects: Vec<Rect>,
    start_rect: Rect,
    sel_row: usize,
    state: State,
    arena_key: &'static str,
    hud_h: i32,
    arena: Rect,
    sc: f64,
    inner_walls: Vec<Rect>,
    tanks: [Tank; 2],
    bullets: Vec<Bullet>,
    powerups: Vec<PowerUp>,
    power_timer: f64,
    count_t: f64,
    banner_t: f64,
    round_winner: usize,
    ai_timer: f64,
    ai_move: (f64, f64),
    ai_aim: Option<f64>,
    ai_wp: Option<(f64, f64)>,
}

impl TankDuelGame {
    /// `TankDuelGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let tank = Tank { x: 0.0, y: 0.0, ang: 0.0, cooldown: 0.0, shield: false, rapid: 0.0, triple: 0, alive: true, keys: Keys::default() };
        let mut g = TankDuelGame {
            b,
            diff: 1,
            arena_sel: -1,
            small: f.clone(),
            tiny: f.clone(),
            huge: f,
            rounds: [0, 0],
            arena_cycle: Vec::new(),
            setup_rows: Vec::new(),
            row_rects: Vec::new(),
            start_rect: z,
            sel_row: 0,
            state: State::Setup,
            arena_key: "open",
            hud_h: 34,
            arena: z,
            sc: 1.0,
            inner_walls: Vec::new(),
            tanks: [tank, tank],
            bullets: Vec::new(),
            powerups: Vec::new(),
            power_timer: 0.0,
            count_t: 0.0,
            banner_t: 0.0,
            round_winner: 0,
            ai_timer: 0.0,
            ai_move: (0.0, 0.0),
            ai_aim: None,
            ai_wp: None,
        };
        g.reset();
        Box::new(g)
    }

    fn make_fonts(&mut self) {
        let h = self.b.height;
        self.small = ui::f(13.max(22.min(h / 30)));
        self.tiny = ui::f(11.max(18.min(h / 38)));
        self.huge = ui::fb(26.max(h / 11));
    }

    fn build_setup_layout(&mut self) {
        let cx = self.b.width / 2;
        let bw = 380.min(self.b.width - 60);
        let y0 = (self.b.height as f64 * 0.30) as i32;
        let mut rows = Vec::new();
        if !self.b.multiplayer {
            rows.push("diff");
        }
        rows.push("arena");
        self.row_rects = (0..rows.len() as i32).map(|i| Rect::new(cx - py::fdiv32(bw, 2), y0 + i * 58, bw, 46)).collect();
        self.start_rect = Rect::new(cx - 95, y0 + rows.len() as i32 * 58 + 16, 190, 46);
        self.setup_rows = rows;
        self.sel_row = 0;
    }

    fn adjust_row(&mut self, row: usize, direction: i64) {
        if self.setup_rows[row] == "diff" {
            self.diff = py::pmod(self.diff as i64 + direction, 3) as usize;
            self.b.set_opt("tanks", "difficulty", (self.diff as i64).into());
        } else {
            self.arena_sel = py::pmod(self.arena_sel + 1 + direction, 5) - 1;
            self.b.set_opt("tanks", "arena", self.arena_sel.into());
        }
        self.b.play_sound("select");
    }

    fn handle_setup(&mut self, ev: &InputEvent) {
        let n = self.setup_rows.len();
        if ev.kind == Kind::KeyDown {
            match ev.key.as_str() {
                "Up" | "w" | "W" => {
                    self.sel_row = (self.sel_row + n - 1) % n;
                    self.b.play_sound("move");
                }
                "Down" | "s" | "S" => {
                    self.sel_row = (self.sel_row + 1) % n;
                    self.b.play_sound("move");
                }
                "Left" | "a" | "A" => self.adjust_row(self.sel_row, -1),
                "Right" | "d" | "D" => self.adjust_row(self.sel_row, 1),
                "Return" | "space" => self.start_match(),
                _ => {}
            }
        } else if ev.kind == Kind::MouseDown {
            if let Some(i) = self.row_rects.iter().position(|r| r.hit(ev.pos)) {
                self.sel_row = i;
                self.adjust_row(i, 1);
                return;
            }
            if self.start_rect.hit(ev.pos) {
                self.start_match();
            }
        }
    }

    fn start_match(&mut self) {
        self.rounds = [0, 0];
        self.b.game_over = false;
        self.b.new_round_result();
        self.arena_cycle = vec!["open", "cross", "pillars", "maze"];
        py::shuffle(&mut self.arena_cycle);
        self.start_round();
        self.b.play_sound("click");
    }

    fn layout_arena(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        self.hud_h = 34.max((h as f64 * 0.075) as i32);
        self.arena = Rect::new(8, self.hud_h, w - 16, h - self.hud_h - 8);
        self.sc = (w as f64 / 640.0).min(h as f64 / 480.0);
        let a = self.arena;
        self.inner_walls = wall_preset(self.arena_key)
            .iter()
            .map(|(fx, fy, fw, fh)| {
                Rect::new(
                    a.x + (fx * a.w as f64) as i32,
                    a.y + (fy * a.h as f64) as i32,
                    14.max((fw * a.w as f64) as i32),
                    14.max((fh * a.h as f64) as i32),
                )
            })
            .collect();
    }

    fn start_round(&mut self) {
        self.arena_key = if self.arena_sel >= 0 {
            ARENAS[(self.arena_sel + 1) as usize]
        } else {
            self.arena_cycle[((self.rounds[0] + self.rounds[1]) % 4) as usize]
        };
        self.layout_arena();
        let a = self.arena;
        let inset_x = (a.w as f64 * 0.15) as i32;
        let inset_y = (a.h as f64 * 0.15) as i32;
        let neu = |x: i32, y: i32, ang: f64| Tank {
            x: x as f64,
            y: y as f64,
            ang,
            cooldown: 0.0,
            shield: false,
            rapid: 0.0,
            triple: 0,
            alive: true,
            keys: Keys::default(),
        };
        self.tanks = [neu(a.x + inset_x, a.y + inset_y, py::radians(45.0)), neu(a.right() - inset_x, a.bottom() - inset_y, py::radians(225.0))];
        self.bullets.clear();
        self.powerups.clear();
        self.power_timer = py::uniform(8.0, 14.0);
        self.count_t = 2.4;
        self.banner_t = 0.0;
        self.round_winner = 0;
        self.ai_timer = 0.0;
        self.ai_move = (0.0, 0.0);
        self.state = State::Countdown;
    }

    fn set_key(&mut self, key: &str, down: bool) {
        let mapping: [(&str, usize); 2] = if self.b.multiplayer { [("p1", 0), ("p2", 1)] } else { [("p1", 0), ("p2", 0)] };
        for (p, i) in mapping {
            for act in ["up", "down", "left", "right"] {
                if self.b.key_for(p, act) == key {
                    self.tanks[i].keys.set(act, down);
                }
            }
            if self.b.key_for(p, "action") == key && down && self.state == State::Play {
                self.fire(i);
            }
        }
    }

    fn tank_r(&self) -> f64 {
        13.0 * self.sc
    }

    fn move_tank(&mut self, i: usize, dt: f64, throttle: f64, turn: f64) {
        let sc = self.sc;
        let tk = &mut self.tanks[i];
        tk.ang += py::radians(180.0) * turn * dt;
        let speed = (if throttle > 0.0 { 140.0 } else { 100.0 }) * sc * throttle;
        if speed != 0.0 {
            tk.x += tk.ang.cos() * speed * dt;
            tk.y += tk.ang.sin() * speed * dt;
        }
        self.collide_walls(i);
    }

    fn collide_walls(&mut self, i: usize) {
        let r = self.tank_r();
        let a = self.arena;
        let tk = &mut self.tanks[i];
        tk.x = (a.x as f64 + r).max((a.right() as f64 - r).min(tk.x));
        tk.y = (a.y as f64 + r).max((a.bottom() as f64 - r).min(tk.y));
        for w in &self.inner_walls {
            let nearest_x = (w.x as f64).max((w.right() as f64).min(tk.x));
            let nearest_y = (w.y as f64).max((w.bottom() as f64).min(tk.y));
            let (dx, dy) = (tk.x - nearest_x, tk.y - nearest_y);
            let d2 = dx * dx + dy * dy;
            if d2 < r * r {
                let mut d = d2.sqrt();
                if d == 0.0 {
                    d = 0.001;
                }
                let push = r - d;
                tk.x += dx / d * push;
                tk.y += dy / d * push;
            }
        }
    }

    fn fire(&mut self, owner: usize) {
        let tk = self.tanks[owner];
        if tk.cooldown > 0.0 || !tk.alive {
            return;
        }
        if self.bullets.iter().filter(|b| b.owner == owner).count() >= MAX_BULLETS {
            return;
        }
        self.tanks[owner].cooldown = if tk.rapid > 0.0 { 0.2 } else { COOLDOWN };
        let mut angles = vec![0.0];
        if tk.triple > 0 {
            angles = vec![-py::radians(12.0), 0.0, py::radians(12.0)];
            self.tanks[owner].triple -= 1;
        }
        let r = self.tank_r();
        for da in angles {
            let a = tk.ang + da;
            self.bullets.push(Bullet {
                x: tk.x + a.cos() * (r + 6.0),
                y: tk.y + a.sin() * (r + 6.0),
                vx: a.cos() * BULLET_SPEED * self.sc,
                vy: a.sin() * BULLET_SPEED * self.sc,
                owner,
                bounces: 0,
                age: 0.0,
            });
        }
        self.b.play_sound("shoot");
    }

    fn update_bullets(&mut self, dt: f64) {
        let a = self.arena;
        let mut alive = Vec::new();
        let bullets = std::mem::take(&mut self.bullets);
        for mut b in bullets {
            b.age += dt;
            if b.age > BULLET_LIFE {
                continue;
            }
            b.x += b.vx * dt;
            b.y += b.vy * dt;
            let mut bounced = false;
            if b.x < a.x as f64 || b.x > a.right() as f64 {
                b.vx = -b.vx;
                b.x = (a.x as f64).max((a.right() as f64).min(b.x));
                bounced = true;
            }
            if b.y < a.y as f64 || b.y > a.bottom() as f64 {
                b.vy = -b.vy;
                b.y = (a.y as f64).max((a.bottom() as f64).min(b.y));
                bounced = true;
            }
            if !bounced {
                for w in &self.inner_walls {
                    if w.hit((b.x, b.y)) {
                        let pen_x = (b.x - w.x as f64).min(w.right() as f64 - b.x);
                        let pen_y = (b.y - w.y as f64).min(w.bottom() as f64 - b.y);
                        if pen_x < pen_y {
                            b.vx = -b.vx;
                            b.x += (pen_x + 1.0).copysign(b.vx);
                        } else {
                            b.vy = -b.vy;
                            b.y += (pen_y + 1.0).copysign(b.vy);
                        }
                        bounced = true;
                        break;
                    }
                }
            }
            if bounced {
                b.bounces += 1;
                if b.bounces >= 2 {
                    continue;
                }
                self.b.play_sound("bounce");
            }
            let mut hit = false;
            for i in 0..2 {
                let tk = self.tanks[i];
                if !tk.alive || (i == b.owner && b.age < GRACE) {
                    continue;
                }
                if py::hypot(b.x - tk.x, b.y - tk.y) < self.tank_r() + 3.5 {
                    hit = true;
                    if tk.shield {
                        self.tanks[i].shield = false;
                        self.b.play_sound("bounce");
                    } else {
                        self.tanks[i].alive = false;
                        if b.bounces >= 1 && i != b.owner && (self.b.multiplayer || b.owner == 0) {
                            self.b.ach_event("tanks_bank", None);
                        }
                        self.round_over(1 - i);
                    }
                    break;
                }
            }
            if !hit {
                alive.push(b);
            }
        }
        self.bullets = alive;
    }

    fn update_powerups(&mut self, dt: f64) {
        for p in self.powerups.iter_mut() {
            p.age += dt;
        }
        self.powerups.retain(|p| p.age < 12.0);
        self.power_timer -= dt;
        if self.power_timer <= 0.0 && self.powerups.len() < 2 {
            self.power_timer = py::uniform(8.0, 14.0);
            let a = self.arena;
            for _ in 0..30 {
                let x = py::uniform(a.x as f64 + 60.0, a.right() as f64 - 60.0);
                let y = py::uniform(a.y as f64 + 60.0, a.bottom() as f64 - 60.0);
                if self.inner_walls.iter().any(|w| w.inflate(120, 120).hit((x, y))) {
                    continue;
                }
                if self.tanks.iter().any(|tk| py::hypot(x - tk.x, y - tk.y) < 60.0) {
                    continue;
                }
                let kind = py::choice(&[Power::Rapid, Power::Shield, Power::Triple]);
                self.powerups.push(PowerUp { x, y, kind, age: 0.0 });
                break;
            }
        }
        let r = self.tank_r() + 12.0;
        let mut i = 0;
        while i < self.powerups.len() {
            let p = self.powerups[i];
            let mut weg = false;
            for t in 0..2 {
                if !self.tanks[t].alive {
                    continue;
                }
                if py::hypot(p.x - self.tanks[t].x, p.y - self.tanks[t].y) < r {
                    match p.kind {
                        Power::Rapid => self.tanks[t].rapid = 8.0,
                        Power::Shield => self.tanks[t].shield = true,
                        Power::Triple => self.tanks[t].triple = 5,
                    }
                    weg = true;
                    self.b.play_sound("powerup");
                    break;
                }
            }
            if weg {
                self.powerups.remove(i);
            } else {
                i += 1;
            }
        }
    }

    fn clear(&self, a: (f64, f64), b: (f64, f64)) -> bool {
        let line = (a.0 as i32, a.1 as i32, b.0 as i32, b.1 as i32);
        !self.inner_walls.iter().any(|w| w.clipline(line).is_some())
    }

    fn ai_update(&mut self, dt: f64) {
        if !self.tanks[1].alive || !self.tanks[0].alive {
            return;
        }
        let intervals = [0.6, 0.35, 0.2];
        let errors = [14.0, 7.0, 3.0];
        self.ai_timer -= dt;
        if self.ai_timer <= 0.0 {
            self.ai_timer = intervals[self.diff];
            self.ai_think(errors[self.diff]);
        }
        let (throttle, turn) = self.ai_move;
        self.move_tank(1, dt, throttle, turn);
        if let Some(want) = self.ai_aim {
            let diff_a = py::fpmod(want - self.tanks[1].ang + py::PI, py::TAU) - py::PI;
            if diff_a.abs() < py::radians(8.0) {
                self.fire(1);
            }
        }
    }

    fn ai_think(&mut self, err_deg: f64) {
        let (ai, pl) = (self.tanks[1], self.tanks[0]);
        let a_pos = (ai.x, ai.y);
        let p_pos = (pl.x, pl.y);
        let dist = py::hypot(p_pos.0 - a_pos.0, p_pos.1 - a_pos.1);
        let mut aim = None;
        if self.clear(a_pos, p_pos) {
            let (mut tx, mut ty) = p_pos;
            if self.diff == 2 {
                let spd = if pl.keys.up { 140.0 * self.sc } else { 0.0 };
                tx += pl.ang.cos() * spd * dist / (BULLET_SPEED * self.sc);
                ty += pl.ang.sin() * spd * dist / (BULLET_SPEED * self.sc);
            }
            aim = Some((ty - a_pos.1).atan2(tx - a_pos.0));
        } else if self.diff == 2 {
            aim = self.ricochet_aim(a_pos, p_pos);
        }
        if let Some(a) = aim {
            aim = Some(a + py::radians(py::uniform(-err_deg, err_deg)));
        }
        self.ai_aim = aim;

        let mut dodge = None;
        if self.diff >= 1 {
            for b in &self.bullets {
                if b.owner == 1 {
                    continue;
                }
                let d = py::hypot(b.x - a_pos.0, b.y - a_pos.1);
                if d < 120.0 * self.sc {
                    dodge = Some((a_pos.1 - b.y).atan2(a_pos.0 - b.x) + py::PI / 2.0);
                    break;
                }
            }
        }
        let (target_ang, throttle);
        if let Some(d) = dodge {
            target_ang = d;
            throttle = 1.0;
        } else if self.diff == 0 {
            let a = self.arena;
            let wp = match self.ai_wp {
                Some(wp) if py::hypot(wp.0 - a_pos.0, wp.1 - a_pos.1) >= 40.0 => wp,
                _ => {
                    let x = py::uniform(a.x as f64 + 40.0, a.right() as f64 - 40.0);
                    let y = py::uniform(a.y as f64 + 40.0, a.bottom() as f64 - 40.0);
                    self.ai_wp = Some((x, y));
                    (x, y)
                }
            };
            target_ang = (wp.1 - a_pos.1).atan2(wp.0 - a_pos.0);
            throttle = 1.0;
        } else if self.diff == 1 {
            target_ang = (p_pos.1 - a_pos.1).atan2(p_pos.0 - a_pos.0);
            throttle = if dist > 350.0 * self.sc {
                1.0
            } else if dist < 200.0 * self.sc {
                -1.0
            } else {
                0.0
            };
        } else {
            target_ang = (p_pos.1 - a_pos.1).atan2(p_pos.0 - a_pos.0) + py::PI / 2.0;
            throttle = 1.0;
        }
        let steer_to = self.ai_aim.unwrap_or(target_ang);
        let diff_a = py::fpmod(steer_to - ai.ang + py::PI, py::TAU) - py::PI;
        let turn = (-1.0f64).max(1.0f64.min(diff_a * 3.0));
        self.ai_move = (throttle, turn);
    }

    fn ricochet_aim(&self, a: (f64, f64), p: (f64, f64)) -> Option<f64> {
        let ar = self.arena;
        let mut planes: Vec<(bool, f64)> = vec![(true, ar.x as f64), (true, ar.right() as f64), (false, ar.y as f64), (false, ar.bottom() as f64)];
        for w in &self.inner_walls {
            planes.extend([(true, w.x as f64), (true, w.right() as f64), (false, w.y as f64), (false, w.bottom() as f64)]);
        }
        py::shuffle(&mut planes);
        for (is_x, v) in planes {
            let mirror = if is_x { (2.0 * v - p.0, p.1) } else { (p.0, 2.0 * v - p.1) };
            let dx = mirror.0 - a.0;
            let dy = mirror.1 - a.1;
            let tt = if is_x {
                if dx == 0.0 {
                    continue;
                }
                (v - a.0) / dx
            } else {
                if dy == 0.0 {
                    continue;
                }
                (v - a.1) / dy
            };
            if !(0.05 < tt && tt < 0.95) {
                continue;
            }
            let hit = (a.0 + dx * tt, a.1 + dy * tt);
            if self.clear(a, hit) && self.clear(hit, p) {
                return Some(dy.atan2(dx));
            }
        }
        None
    }

    fn round_over(&mut self, winner: usize) {
        self.rounds[winner] += 1;
        self.round_winner = winner;
        self.b.play_sound("explode");
        if self.rounds[winner] >= ROUNDS_TO_WIN {
            self.match_over(winner);
        } else {
            self.banner_t = 1.8;
            self.state = State::RoundEnd;
            self.b.play_sound("point");
        }
    }

    fn match_over(&mut self, winner: usize) {
        self.state = State::MatchEnd;
        if !self.b.multiplayer && winner == 0 {
            self.b.score += (self.diff as i64 + 1) * 100 + (ROUNDS_TO_WIN - self.rounds[1]) * 20;
            self.b.play_sound("win");
            self.b.report_result(true);
        } else if self.b.multiplayer {
            self.b.play_sound("win");
            self.b.report_duel(winner);
        } else {
            self.b.play_sound("gameover");
            self.b.report_result(false);
        }
        self.b.game_over = true;
    }

    fn draw_tank(&self, s: &mut Surface, tk: &Tank, col: Color) {
        let r = self.tank_r();
        let (x, y, a) = (tk.x, tk.y, tk.ang);
        let (c, sn) = (a.cos(), a.sin());
        let pts: Vec<(f64, f64)> =
            [(-r, -r * 0.75), (r, -r * 0.75), (r, r * 0.75), (-r, r * 0.75)].iter().map(|(px, py_)| (x + px * c - py_ * sn, y + px * sn + py_ * c)).collect();
        draw::polygon(s, col, &pts, 0);
        let k = |f: f64| rgb((col.r as f64 * f) as i32, (col.g as f64 * f) as i32, (col.b as f64 * f) as i32);
        draw::polygon(s, k(0.6), &pts, 2);
        draw::circle(s, k(0.7), (x as i32, y as i32), (r * 0.55) as i32, 0);
        draw::line(s, rgb(230, 232, 240), (x, y), (x + c * r * 1.5, y + sn * r * 1.5), 3.max((4.0 * self.sc) as i32));
        if tk.shield {
            draw::circle(s, power_col(Power::Shield), (x as i32, y as i32), (r * 1.5) as i32, 2);
        }
        if tk.rapid > 0.0 {
            draw::circle(s, power_col(Power::Rapid), (x as i32, y as i32), (r * 1.3) as i32, 1);
        }
    }

    fn draw_powerup(&self, s: &mut Surface, p: &PowerUp) {
        let col = power_col(p.kind);
        let (x, y) = (p.x as i32, p.y as i32);
        if p.age > 9.0 && (p.age * 6.0) as i64 % 2 == 0 {
            return;
        }
        draw::circle(s, col, (x, y), 11, 0);
        draw::circle(s, rgb(20, 24, 34), (x, y), 11, 2);
        let sym = match p.kind {
            Power::Rapid => ">>",
            Power::Shield => "O",
            Power::Triple => "3x",
        };
        let img = self.tiny.render(sym, true, rgb(20, 24, 34));
        s.blr(&img, img.get_rect().center_at((x, y)));
    }

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        draw::rect(s, p.panel, (0, 0, w, self.hud_h), 0);
        draw::line(s, p.border, (0, self.hud_h), (w, self.hud_h), 1);
        let cy = self.hud_h / 2;
        let pr = 3.max(self.hud_h / 9);
        let gap = 2 * pr + 6;
        for (i, col) in [(0usize, COL_P1), (1, COL_P2)] {
            let name: String = if i == 0 {
                self.b.pname(0).chars().take(8).collect()
            } else if self.b.multiplayer {
                self.b.pname(1).chars().take(8).collect()
            } else {
                t("common.ai")
            };
            let img = self.small.render(&name, true, col);
            if i == 0 {
                s.blr(&img, img.get_rect().midleft_at((12, cy)));
                let x0 = 12 + img.w() + 12 + pr;
                for k in 0..ROUNDS_TO_WIN as i32 {
                    let x = x0 + k * gap;
                    if (k as i64) < self.rounds[i] {
                        draw::circle(s, col, (x, cy), pr, 0);
                    } else {
                        draw::circle(s, p.border_light, (x, cy), pr, 1);
                    }
                }
            } else {
                s.blr(&img, img.get_rect().midright_at((w - 12, cy)));
                let x0 = w - 12 - img.w() - 12 - pr;
                for k in 0..ROUNDS_TO_WIN as i32 {
                    let x = x0 - k * gap;
                    if (k as i64) < self.rounds[i] {
                        draw::circle(s, col, (x, cy), pr, 0);
                    } else {
                        draw::circle(s, p.border_light, (x, cy), pr, 1);
                    }
                }
            }
        }
        let mid = self.small.render(&tf("tank.first_to", &[("n", &ROUNDS_TO_WIN.to_string())]), true, p.text_dim);
        s.blr(&mid, mid.get_rect().center_at((w / 2, cy)));
    }

    fn draw_banner(&self, s: &mut Surface, text: &str) {
        let p = ui::pal();
        let w = self.b.width;
        let img = self.huge.render(text, true, self.b.accent);
        let band_h = img.h() + 28;
        let y = self.b.height / 2 - band_h / 2;
        let mut ov = pg::surface_alpha(w, band_h);
        ov.fill_c(pg::alpha(p.panel, 225));
        s.bl(&ov, (0, y));
        draw::line(s, self.b.accent, (0, y), (w, y), 2);
        draw::line(s, self.b.accent, (0, y + band_h - 1), (w, y + band_h - 1), 2);
        s.blr(&img, img.get_rect().center_at((w / 2, y + band_h / 2)));
    }

    fn draw_match_end(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let mut dim = pg::surface_alpha(w, h);
        dim.fill_c(rgba(8, 10, 16, 150));
        s.bl(&dim, (0, 0));
        let (cx, cy) = (w / 2, h / 2);
        let winner = if self.rounds[0] >= ROUNDS_TO_WIN { 0 } else { 1 };
        let head = if self.b.multiplayer {
            let name: String = self.b.pname(winner).chars().take(10).collect();
            self.huge.render(&tf("common.player_wins_name", &[("name", &name)]), true, if winner == 0 { COL_P1 } else { COL_P2 })
        } else {
            let key = if winner == 0 { "tank.match_win" } else { "common.game_over" };
            self.huge.render(&t(key), true, if winner == 0 { COL_P1 } else { p.red })
        };
        let mut rows = vec![head, self.b.font.render(&format!("{} : {}", self.rounds[0], self.rounds[1]), true, p.text)];
        if !self.b.multiplayer {
            rows.push(self.small.render(&tf("common.points", &[("score", &self.b.score.to_string())]), true, p.text_dim));
        }
        rows.push(self.small.render(&t("tank.rematch"), true, p.text_dim));
        let gap = 10;
        let total: i32 = rows.iter().map(|r| r.h()).sum::<i32>() + gap * (rows.len() as i32 - 1);
        let pw = (w - 30).min(340.max(rows.iter().map(|r| r.w()).max().unwrap_or(0) + 64));
        let panel = Rect::new(0, 0, pw, total + 48).center_at((cx, cy));
        let mut ov = pg::surface_alpha(panel.w, panel.h);
        ov.fill_c(pg::alpha(p.panel, 235));
        s.bl(&ov, (panel.x, panel.y));
        draw::rect_r(s, self.b.accent, panel, 2, 14);
        let mut yy = panel.y + 24;
        for r in &rows {
            s.blr(r, r.get_rect().midtop_at((cx, yy)));
            yy += r.h() + gap;
        }
    }

    fn draw_setup(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let name = pgz_core::achievements::game_name(self.b.key).to_uppercase();
        let title = self.huge.render(&name, true, self.b.accent);
        s.blr(&title, title.get_rect().center_at((cx, (h as f64 * 0.13) as i32)));
        let sub = self.small.render(&t("tank.subtitle"), true, p.text_dim);
        s.blr(&sub, sub.get_rect().center_at((cx, (h as f64 * 0.20) as i32)));
        for (i, r) in self.row_rects.iter().enumerate() {
            let on = i == self.sel_row;
            draw::rect_r(s, if on { p.btn_sel } else { p.btn }, *r, 0, 10);
            draw::rect_r(s, if on { self.b.accent } else { p.border }, *r, if on { 2 } else { 1 }, 10);
            let (label, value) = if self.setup_rows[i] == "diff" {
                (t("tank.difficulty"), t(&format!("tank.diff.{}", DIFFS[self.diff])))
            } else {
                (t("tank.arena"), t(&format!("tank.arena.{}", ARENAS[(self.arena_sel + 1) as usize])))
            };
            let img = self.small.render(&label, true, if on { p.text } else { p.text_dim });
            s.blr(&img, img.get_rect().midleft_at((r.x + 16, r.centery())));
            let img = self.small.render(&format!("< {value} >"), true, self.b.accent);
            s.blr(&img, img.get_rect().midright_at((r.right() - 16, r.centery())));
        }
        draw::rect_r(s, p.btn_sel, self.start_rect, 0, 10);
        draw::rect_r(s, self.b.accent, self.start_rect, 2, 10);
        let st = self.b.font.render(&t("common.start"), true, p.text);
        s.blr(&st, st.get_rect().center_at(self.start_rect.center()));
        let hint = self.tiny.render(&t("tank.setup_hint"), true, p.text_faint);
        s.blr(&hint, hint.get_rect().center_at((cx, h - 30)));
        let ctrl = self.tiny.render(&t("tank.controls_hint"), true, ui::mix(self.b.accent, p.text, 0.45));
        s.blr(&ctrl, ctrl.get_rect().center_at((cx, h - 12)));
    }
}

impl Game for TankDuelGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        self.b.score = 0;
        self.b.game_over = false;
        self.diff = self.b.opt_i64("tanks", "difficulty", 1).clamp(0, 2) as usize;
        self.arena_sel = self.b.opt_i64("tanks", "arena", -1).clamp(-1, 3);
        self.make_fonts();
        self.rounds = [0, 0];
        self.arena_cycle.clear();
        self.build_setup_layout();
        self.state = State::Setup;
    }

    fn on_surface_changed(&mut self) {
        self.make_fonts();
        self.build_setup_layout();
        if self.state != State::Setup {
            self.layout_arena();
        }
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        match self.state {
            State::Setup => {
                self.handle_setup(ev);
                return;
            }
            State::MatchEnd => {
                if ev.kind == Kind::KeyDown {
                    if ev.key_in(&["Return", "space"]) {
                        self.b.game_over = false;
                        self.start_match();
                    } else if ev.key_in(&["s", "S"]) {
                        self.b.game_over = false;
                        self.state = State::Setup;
                        self.b.play_sound("click");
                    }
                } else if ev.kind == Kind::MouseDown {
                    self.b.game_over = false;
                    self.start_match();
                }
                return;
            }
            _ => {}
        }
        if ev.kind == Kind::KeyDown {
            self.set_key(&ev.key, true);
        } else if ev.kind == Kind::KeyUp {
            self.set_key(&ev.key, false);
        }
    }

    fn update(&mut self, dt: f64) {
        match self.state {
            State::Countdown => {
                self.count_t -= dt;
                if self.count_t <= 0.0 {
                    self.state = State::Play;
                    self.b.play_sound("level");
                }
                return;
            }
            State::RoundEnd => {
                self.banner_t -= dt;
                if self.banner_t <= 0.0 {
                    self.start_round();
                }
                return;
            }
            State::Play => {}
            _ => return,
        }
        for tk in self.tanks.iter_mut() {
            tk.cooldown = 0f64.max(tk.cooldown - dt);
            tk.rapid = 0f64.max(tk.rapid - dt);
        }
        let players = if self.b.multiplayer { 2 } else { 1 };
        for i in 0..players {
            let k = self.tanks[i].keys;
            let throttle = (if k.up { 1.0 } else { 0.0 }) - (if k.down { 1.0 } else { 0.0 });
            let turn = (if k.right { 1.0 } else { 0.0 }) - (if k.left { 1.0 } else { 0.0 });
            self.move_tank(i, dt, throttle, turn);
        }
        if !self.b.multiplayer {
            self.ai_update(dt);
        }
        let (a, b) = (self.tanks[0], self.tanks[1]);
        let d = py::hypot(a.x - b.x, a.y - b.y);
        let min_d = 2.0 * self.tank_r();
        if 0.0 < d && d < min_d {
            let push = (min_d - d) / 2.0;
            let (nx, ny) = ((a.x - b.x) / d, (a.y - b.y) / d);
            self.tanks[0].x += nx * push;
            self.tanks[0].y += ny * push;
            self.tanks[1].x -= nx * push;
            self.tanks[1].y -= ny * push;
        }
        self.update_bullets(dt);
        self.update_powerups(dt);
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        let (w, h) = (self.b.width, self.b.height);
        ui::draw_background(&mut s, w, h);
        if self.state == State::Setup {
            self.draw_setup(&mut s);
            return;
        }
        draw::rect(&mut s, COL_FLOOR, self.arena, 0);
        draw::rect(&mut s, COL_WALL_EDGE, self.arena, 3);
        for wl in &self.inner_walls {
            draw::rect(&mut s, COL_WALL, *wl, 0);
            draw::rect(&mut s, COL_WALL_EDGE, *wl, 2);
        }
        for p in &self.powerups {
            self.draw_powerup(&mut s, p);
        }
        for b in &self.bullets {
            draw::circle(&mut s, COL_BULLET, tp((b.x, b.y)), 2.max((3.5 * self.sc) as i32), 0);
        }
        for (i, tk) in self.tanks.iter().enumerate() {
            if tk.alive {
                self.draw_tank(&mut s, tk, if i == 0 { COL_P1 } else { COL_P2 });
            }
        }
        self.draw_hud(&mut s);
        match self.state {
            State::Countdown => {
                let n = 1.max((self.count_t / 0.8).ceil() as i64);
                let img = self.huge.render(&n.to_string(), true, self.b.accent);
                s.blr(&img, img.get_rect().center_at((w / 2, h / 2)));
            }
            State::RoundEnd => {
                self.draw_banner(&mut s, &tf("tank.round_win", &[("n", &(self.round_winner + 1).to_string())]));
            }
            State::MatchEnd => self.draw_match_end(&mut s),
            _ => {}
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "tanks",
    class_name: "TankDuelGame",
    supports_multiplayer: true,
    modes: &[],
    make: TankDuelGame::new,
};
