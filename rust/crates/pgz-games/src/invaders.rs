//! invaders.rs
//! ===========
//! Space Invaders - Klassik (Alien-Block, Setup fuer Bewegung/Zielen) und
//! Arena (Gegner von allen Raendern, Waffenwechsel 1-4).
//!
//! Uebertragen aus `games/invaders.py`: Level mit Boss in jedem vierten,
//! vier Waffen, Power-Ups (Leben, Schild, Waffe), Schutzschilde in Klassik,
//! Partikel, Startbanner.

use pgz_engine::prelude::*;

const COL_PLAYER: Color = Color::rgb(110, 220, 140);
const COL_PLAYER_HIT: Color = Color::rgb(240, 240, 240);
const COL_SHIELD: Color = Color::rgb(90, 170, 220);
const COL_EBULLET: Color = Color::rgb(240, 120, 120);
const WEAPON_ORDER: [&str; 4] = ["single", "spread", "rapid", "pierce"];

fn weapon_col(w: &str) -> Color {
    match w {
        "spread" => Color::rgb(140, 235, 180),
        "rapid" => Color::rgb(240, 190, 110),
        "pierce" => Color::rgb(150, 200, 255),
        _ => Color::rgb(240, 240, 120),
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Art {
    Grunt,
    Chaser,
    Shooter,
    Drifter,
    Boss,
}

fn kind_col(k: Art) -> Color {
    match k {
        Art::Grunt => Color::rgb(235, 110, 110),
        Art::Chaser => Color::rgb(235, 150, 80),
        Art::Shooter => Color::rgb(185, 120, 235),
        Art::Drifter => Color::rgb(110, 180, 235),
        Art::Boss => Color::rgb(235, 90, 140),
    }
}

fn kind_score(k: Art) -> i64 {
    match k {
        Art::Grunt => 10,
        Art::Chaser => 15,
        Art::Shooter => 20,
        Art::Drifter => 12,
        Art::Boss => 250,
    }
}

const PLAYER_W: f64 = 40.0;
const PLAYER_H: f64 = 22.0;
const PLAYER_SPEED: f64 = 300.0;
const PLAYER_Y_OFF: f64 = 46.0;
const EBULLET_SPEED: f64 = 210.0;
const HIT_INVULN: f64 = 1.4;
const POWERUP_FALL: f64 = 70.0;
const POWERUP_CHANCE: f64 = 0.13;
const WEAPON_PICKUP_TIME: f64 = 14.0;

#[derive(Clone, Copy)]
struct Alien {
    id: u64,
    cx: f64,
    cy: f64,
    kind: Art,
    hp: i64,
    maxhp: i64,
    w: f64,
    h: f64,
    vx: f64,
    vy: f64,
    t: f64,
    shoot: f64,
    burst: f64,
}

#[derive(Clone)]
struct Bullet {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    dmg: i64,
    pierce: i64,
    rad: i32,
    hits: Vec<u64>,
    col: Color,
}

#[derive(Clone, Copy)]
struct EBullet {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    rad: i32,
}

#[derive(Clone, Copy, PartialEq)]
enum PowerKind {
    Weapon,
    Shield,
    Life,
}

#[derive(Clone, Copy)]
struct PowerUp {
    x: f64,
    y: f64,
    vy: f64,
    kind: PowerKind,
}

#[derive(Clone, Copy)]
struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    life: f64,
    max: f64,
    col: Color,
}

#[derive(Clone, Copy)]
struct Shield {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    hp: i64,
}

#[derive(Clone, Copy, Default)]
struct Held {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    action: bool,
}

/// `InvadersGame`
pub struct InvadersGame {
    b: GameBase,
    arena: bool,
    opt_move: &'static str,
    opt_aim: &'static str,
    mouse_pos: (f64, f64),
    setup: bool,
    lives: i64,
    level: i64,
    bullets: Vec<Bullet>,
    ebullets: Vec<EBullet>,
    powerups: Vec<PowerUp>,
    particles: Vec<Particle>,
    aliens: Vec<Alien>,
    shields: Vec<Shield>,
    weapon: &'static str,
    weapon_timer: f64,
    shield_timer: f64,
    invuln_timer: f64,
    cooldown: f64,
    held: Held,
    face: (f64, f64),
    px: f64,
    py: f64,
    banner: String,
    banner_t: f64,
    start_count: usize,
    free_move: bool,
    mouse_aim: bool,
    setup_sel: usize,
    setup_rects: Vec<Rect>,
    alien_dir: f64,
    alien_speed: f64,
    alien_drop: f64,
    shoot_timer: f64,
    next_id: u64,
}

impl InvadersGame {
    /// `InvadersGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let mut g = InvadersGame {
            b,
            arena: false,
            opt_move: "lr",
            opt_aim: "fixed",
            mouse_pos: (0.0, 0.0),
            setup: false,
            lives: 3,
            level: 1,
            bullets: Vec::new(),
            ebullets: Vec::new(),
            powerups: Vec::new(),
            particles: Vec::new(),
            aliens: Vec::new(),
            shields: Vec::new(),
            weapon: "single",
            weapon_timer: 0.0,
            shield_timer: 0.0,
            invuln_timer: 0.0,
            cooldown: 0.0,
            held: Held::default(),
            face: (0.0, -1.0),
            px: 0.0,
            py: 0.0,
            banner: String::new(),
            banner_t: 0.0,
            start_count: 0,
            free_move: false,
            mouse_aim: false,
            setup_sel: 0,
            setup_rects: Vec::new(),
            alien_dir: 1.0,
            alien_speed: 0.0,
            alien_drop: 0.0,
            shoot_timer: 0.0,
            next_id: 1,
        };
        g.reset();
        Box::new(g)
    }

    fn init_run(&mut self) {
        self.setup = false;
        self.b.score = 0;
        self.b.game_over = false;
        self.lives = 3;
        self.level = 1;
        self.bullets.clear();
        self.ebullets.clear();
        self.powerups.clear();
        self.particles.clear();
        self.aliens.clear();
        self.shields.clear();
        self.weapon = "single";
        self.weapon_timer = 0.0;
        self.shield_timer = 0.0;
        self.invuln_timer = 0.0;
        self.cooldown = 0.0;
        self.held = Held::default();
        self.face = (0.0, -1.0);
        self.px = self.b.width as f64 / 2.0;
        self.py = self.b.height as f64 - PLAYER_Y_OFF;
        self.banner.clear();
        self.banner_t = 0.0;
        self.start_count = 0;
    }

    fn enter_play(&mut self) {
        if self.arena {
            self.free_move = true;
            self.mouse_aim = false;
        } else {
            self.free_move = self.opt_move == "free";
            self.mouse_aim = self.opt_aim == "mouse";
        }
        self.setup = false;
        self.level = 1;
        self.start_level();
    }

    fn restart(&mut self) {
        self.init_run();
        self.enter_play();
    }

    fn build_setup(&mut self) {
        self.setup_sel = 0;
        let (bw, bh, gap) = (340, 42, 14);
        let total = 3 * (bh + gap) - gap;
        let y0 = 150.max(self.b.height / 2 - py::fdiv32(total, 2) + 20);
        let x = self.b.width / 2 - bw / 2;
        self.setup_rects = (0..3).map(|i| Rect::new(x, y0 + i * (bh + gap), bw, bh)).collect();
    }

    fn setup_event(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown {
            let k = ev.key.as_str();
            if self.b.is_action(k, "up", None) {
                self.setup_sel = (self.setup_sel + 2) % 3;
                self.b.play_sound("move");
            } else if self.b.is_action(k, "down", None) {
                self.setup_sel = (self.setup_sel + 1) % 3;
                self.b.play_sound("move");
            } else if self.b.is_action(k, "left", None) || self.b.is_action(k, "right", None) {
                self.setup_toggle();
            } else if matches!(k, "Return" | "space") || self.b.is_action(k, "action", None) {
                self.setup_activate();
            }
        } else if ev.kind == Kind::MouseMove {
            if let Some(i) = self.setup_rects.iter().position(|r| r.hit(ev.pos)) {
                self.setup_sel = i;
            }
        } else if ev.kind == Kind::MouseDown {
            if let Some(i) = self.setup_rects.iter().position(|r| r.hit(ev.pos)) {
                self.setup_sel = i;
                self.setup_activate();
            }
        }
    }

    fn setup_toggle(&mut self) {
        if self.setup_sel == 0 {
            self.opt_move = if self.opt_move == "lr" { "free" } else { "lr" };
            self.b.play_sound("select");
        } else if self.setup_sel == 1 {
            self.opt_aim = if self.opt_aim == "fixed" { "mouse" } else { "fixed" };
            self.b.play_sound("select");
        }
    }

    fn setup_activate(&mut self) {
        if self.setup_sel == 2 {
            self.b.play_sound("click");
            self.enter_play();
        } else {
            self.setup_toggle();
        }
    }

    fn level_boss(&self) -> bool {
        self.level % 4 == 0
    }

    fn start_level(&mut self) {
        self.bullets.clear();
        self.ebullets.clear();
        self.powerups.clear();
        self.aliens.clear();
        let boss = self.level_boss();
        self.banner = if boss { t("inv.boss_level") } else { tf("inv.level_start", &[("level", &self.level.to_string())]) };
        self.banner_t = 2.2;
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        if self.arena {
            self.px = w / 2.0;
            self.py = h / 2.0;
        } else {
            self.px = w / 2.0;
            self.py = h - PLAYER_Y_OFF;
        }
        if boss {
            self.spawn_boss();
        } else if self.arena {
            self.spawn_arena_wave();
        } else {
            self.spawn_classic_block();
        }
        if !self.arena {
            self.build_shields();
        }
        self.start_count = 1.max(self.aliens.len());
    }

    #[allow(clippy::too_many_arguments)]
    fn make_alien(&mut self, cx: f64, cy: f64, kind: Art, hp: i64, w: f64, h: f64, vx: f64, vy: f64) -> Alien {
        let t = py::random() * 6.28;
        let shoot = py::uniform(0.6, 2.2);
        let id = self.next_id;
        self.next_id += 1;
        Alien { id, cx, cy, kind, hp, maxhp: hp, w, h, vx, vy, t, shoot, burst: 2.4 }
    }

    fn spawn_classic_block(&mut self) {
        let rows = 6.min(3 + self.level / 2);
        let cols = 10.min(6 + self.level / 3);
        let hp = 1 + self.level / 4;
        let (gap_x, gap_y) = (16.0, 14.0);
        let block_w = cols as f64 * (30.0 + gap_x) - gap_x;
        let start_x = (self.b.width as f64 - block_w) / 2.0 + 15.0;
        let start_y = 60.0;
        for r in 0..rows {
            for c in 0..cols {
                let a = self.make_alien(start_x + c as f64 * (30.0 + gap_x), start_y + r as f64 * (22.0 + gap_y), Art::Grunt, hp, 30.0, 22.0, 0.0, 0.0);
                self.aliens.push(a);
            }
        }
        self.alien_dir = 1.0;
        self.alien_speed = (22 + self.level * 8) as f64;
        self.alien_drop = 16.0;
        self.shoot_timer = 0.0;
    }

    fn spawn_arena_wave(&mut self) {
        let count = 5 + self.level * 2;
        let hp = 1 + self.level / 3;
        let kinds = [Art::Chaser, Art::Drifter, Art::Shooter];
        for _ in 0..count {
            let (cx, cy) = self.edge_spawn();
            let kind = py::choice(&kinds);
            let spd = (40 + self.level * 5) as f64 + py::uniform(-10.0, 20.0);
            let ang = (self.py - cy).atan2(self.px - cx);
            let a = self.make_alien(cx, cy, kind, hp, 30.0, 22.0, ang.cos() * spd, ang.sin() * spd);
            self.aliens.push(a);
        }
    }

    fn spawn_boss(&mut self) {
        let (bx, by) = (self.b.width as f64 / 2.0, 90.0);
        let hp = 26 + self.level * 6;
        let mut boss = self.make_alien(bx, by, Art::Boss, hp, 90.0, 54.0, (70 + self.level * 4) as f64, 0.0);
        boss.burst = 2.4;
        self.aliens.push(boss);
        for i in 0..4 {
            if self.arena {
                let (cx, cy) = self.edge_spawn();
                let vx = py::uniform(-60.0, 60.0);
                let vy = py::uniform(30.0, 70.0);
                let a = self.make_alien(cx, cy, Art::Drifter, 1, 30.0, 22.0, vx, vy);
                self.aliens.push(a);
            } else {
                let a = self.make_alien((120 + i * 130) as f64, 170.0, Art::Grunt, 1, 30.0, 22.0, 0.0, 0.0);
                self.aliens.push(a);
            }
        }
        if !self.arena {
            self.alien_dir = 1.0;
            self.alien_speed = 30.0;
            self.alien_drop = 12.0;
            self.shoot_timer = 0.0;
        }
    }

    fn edge_spawn(&self) -> (f64, f64) {
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        match py::choice(&["top", "bottom", "left", "right"]) {
            "top" => (py::uniform(20.0, w - 20.0), -20.0),
            "bottom" => (py::uniform(20.0, w - 20.0), h + 20.0),
            "left" => (-20.0, py::uniform(40.0, h - 20.0)),
            _ => (w + 20.0, py::uniform(40.0, h - 20.0)),
        }
    }

    fn build_shields(&mut self) {
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        self.shields = (0..4).map(|i| Shield { x: (i + 1) as f64 * w / 5.0 - 26.0, y: h - 130.0, w: 52.0, h: 18.0, hp: 6 }).collect();
    }

    fn select_weapon(&mut self, weapon: &'static str) {
        if weapon == self.weapon {
            return;
        }
        self.weapon = weapon;
        self.weapon_timer = 0.0;
        self.b.play_sound("select");
    }

    fn update_player(&mut self, dt: f64) {
        let h = self.held;
        let vx = h.right as i32 - h.left as i32;
        let vy = if self.free_move { h.down as i32 - h.up as i32 } else { 0 };
        let (mut nx, mut ny) = (0.0, 0.0);
        if vx != 0 || vy != 0 {
            let mut length = py::hypot(vx as f64, vy as f64);
            if length == 0.0 {
                length = 1.0;
            }
            nx = vx as f64 / length;
            ny = vy as f64 / length;
            self.px += nx * PLAYER_SPEED * dt;
            if self.free_move {
                self.py += ny * PLAYER_SPEED * dt;
            }
        }
        if self.mouse_aim {
            let dx = self.mouse_pos.0 - self.px;
            let dy = self.mouse_pos.1 - self.py;
            let d = py::hypot(dx, dy);
            if d > 1.0 {
                self.face = (dx / d, dy / d);
            }
        } else if self.arena {
            if vx != 0 || vy != 0 {
                self.face = (nx, ny);
            }
        } else {
            self.face = (0.0, -1.0);
        }
        let (w, hh) = (self.b.width as f64, self.b.height as f64);
        self.px = (PLAYER_W / 2.0).max((w - PLAYER_W / 2.0).min(self.px));
        if self.free_move {
            self.py = (34.0 + PLAYER_H / 2.0).max((hh - PLAYER_H / 2.0).min(self.py));
        } else {
            self.py = hh - PLAYER_Y_OFF;
        }
    }

    fn fire(&mut self) {
        if self.cooldown > 0.0 {
            return;
        }
        let (fx, fy) = self.face;
        let ang = fy.atan2(fx);
        let (ox, oy) = (self.px + fx * (PLAYER_H / 2.0 + 4.0), self.py + fy * (PLAYER_H / 2.0 + 4.0));
        match self.weapon {
            "spread" => {
                for da in [-0.26, 0.0, 0.26] {
                    self.add_bullet(ox, oy, ang + da, 460.0, 1, 0, 3);
                }
                self.cooldown = 0.46;
            }
            "rapid" => {
                let a = ang + py::uniform(-0.05, 0.05);
                self.add_bullet(ox, oy, a, 560.0, 1, 0, 3);
                self.cooldown = 0.10;
            }
            "pierce" => {
                self.add_bullet(ox, oy, ang, 720.0, 2, 3, 5);
                self.cooldown = 0.55;
            }
            _ => {
                self.add_bullet(ox, oy, ang, 520.0, 1, 0, 3);
                self.cooldown = 0.28;
            }
        }
        self.b.play_sound("shoot");
    }

    #[allow(clippy::too_many_arguments)]
    fn add_bullet(&mut self, x: f64, y: f64, ang: f64, spd: f64, dmg: i64, pierce: i64, rad: i32) {
        self.bullets.push(Bullet { x, y, vx: ang.cos() * spd, vy: ang.sin() * spd, dmg, pierce, rad, hits: Vec::new(), col: weapon_col(self.weapon) });
    }

    fn update_classic(&mut self, dt: f64) {
        let w = self.b.width as f64;
        let minion_idx: Vec<usize> = (0..self.aliens.len()).filter(|&i| self.aliens[i].kind != Art::Boss).collect();
        if !minion_idx.is_empty() {
            let factor = 1.0 + (self.start_count as f64 - self.aliens.len() as f64) * 0.025;
            let dx = self.alien_dir * self.alien_speed * factor * dt;
            let left = minion_idx.iter().map(|&i| self.aliens[i].cx - self.aliens[i].w / 2.0).fold(f64::INFINITY, f64::min);
            let right = minion_idx.iter().map(|&i| self.aliens[i].cx + self.aliens[i].w / 2.0).fold(f64::NEG_INFINITY, f64::max);
            if (self.alien_dir > 0.0 && right + dx >= w - 8.0) || (self.alien_dir < 0.0 && left + dx <= 8.0) {
                self.alien_dir *= -1.0;
                for &i in &minion_idx {
                    self.aliens[i].cy += self.alien_drop;
                }
            } else {
                for &i in &minion_idx {
                    self.aliens[i].cx += dx;
                }
            }
            self.shoot_timer -= dt;
            if self.shoot_timer <= 0.0 {
                self.shoot_timer = 0.25f64.max(py::uniform(0.5, 1.3) - self.level as f64 * 0.03);
                let mut columns: Vec<(i64, usize)> = Vec::new();
                for &i in &minion_idx {
                    let a = &self.aliens[i];
                    let key = py::round(a.cx / 20.0);
                    match columns.iter_mut().find(|c| c.0 == key) {
                        None => columns.push((key, i)),
                        Some(c) => {
                            if a.cy > self.aliens[c.1].cy {
                                c.1 = i;
                            }
                        }
                    }
                }
                let si = py::choice(&columns).1;
                let sh = self.aliens[si];
                self.enemy_shot(sh.cx, sh.cy + sh.h / 2.0, 0.0, 1.0, EBULLET_SPEED);
            }
        }
        let boss_idx: Vec<usize> = (0..self.aliens.len()).filter(|&i| self.aliens[i].kind == Art::Boss).collect();
        for i in boss_idx {
            self.update_boss(i, dt);
        }
        let limit = if self.free_move { self.b.height as f64 - 40.0 } else { self.py - PLAYER_H / 2.0 };
        if minion_idx.iter().any(|&i| self.aliens[i].cy + self.aliens[i].h / 2.0 >= limit) {
            self.hit_player();
            for &i in &minion_idx {
                self.aliens[i].cy -= self.alien_drop * 3.0;
            }
        }
    }

    fn update_arena(&mut self, dt: f64) {
        for i in 0..self.aliens.len() {
            if self.aliens[i].kind == Art::Boss {
                self.update_boss(i, dt);
            } else {
                self.update_arena_enemy(i, dt);
            }
        }
    }

    fn check_contact_damage(&mut self) {
        if self.invuln_timer > 0.0 || self.shield_timer > 0.0 {
            return;
        }
        let hit = self.aliens.iter().any(|a| (a.cx - self.px).abs() < (a.w + PLAYER_W) / 2.0 * 0.6 && (a.cy - self.py).abs() < (a.h + PLAYER_H) / 2.0 * 0.6);
        if hit {
            self.hit_player();
        }
    }

    fn update_arena_enemy(&mut self, i: usize, dt: f64) {
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        let (px, py_) = (self.px, self.py);
        let level = self.level as f64;
        let mut shot = None;
        {
            let a = &mut self.aliens[i];
            a.t += dt;
            match a.kind {
                Art::Chaser => {
                    let ang = (py_ - a.cy).atan2(px - a.cx);
                    let spd = 60.0 + level * 6.0;
                    a.vx = ang.cos() * spd;
                    a.vy = ang.sin() * spd;
                }
                Art::Drifter => {
                    if a.cx < 12.0 || a.cx > w - 12.0 {
                        a.vx *= -1.0;
                    }
                    if a.cy < 34.0 || a.cy > h - 12.0 {
                        a.vy *= -1.0;
                    }
                }
                Art::Shooter => {
                    let mut dist = py::hypot(px - a.cx, py_ - a.cy);
                    if dist == 0.0 {
                        dist = 1.0;
                    }
                    let ang = (py_ - a.cy).atan2(px - a.cx);
                    let drive = if dist > 240.0 {
                        1.0
                    } else if dist < 150.0 {
                        -1.0
                    } else {
                        0.0
                    };
                    let spd = 55.0 + level * 4.0;
                    let strafe = 0.9;
                    a.vx = (ang.cos() * drive - ang.sin() * strafe) * spd;
                    a.vy = (ang.sin() * drive + ang.cos() * strafe) * spd;
                    a.shoot -= dt;
                    if a.shoot <= 0.0 {
                        a.shoot = py::uniform(1.1, 2.2);
                        shot = Some((a.cx, a.cy, ang.cos(), ang.sin()));
                    }
                }
                _ => {}
            }
        }
        if let Some((x, y, dx, dy)) = shot {
            self.enemy_shot(x, y, dx, dy, EBULLET_SPEED);
        }
        let a = &mut self.aliens[i];
        a.cx += a.vx * dt;
        a.cy += a.vy * dt;
        a.cx = (-30.0f64).max((w + 30.0).min(a.cx));
        a.cy = (-30.0f64).max((h + 30.0).min(a.cy));
    }

    fn update_boss(&mut self, i: usize, dt: f64) {
        let w = self.b.width as f64;
        let level = self.level as f64;
        let (px, py_) = (self.px, self.py);
        let b = &mut self.aliens[i];
        b.t += dt;
        b.cx += b.vx * dt;
        if b.cx < b.w / 2.0 + 8.0 || b.cx > w - b.w / 2.0 - 8.0 {
            b.vx *= -1.0;
            b.cx = (b.w / 2.0 + 8.0).max((w - b.w / 2.0 - 8.0).min(b.cx));
        }
        b.cy = 90.0 + (b.t * 1.3).sin() * 22.0;
        b.burst -= dt;
        if b.burst <= 0.0 {
            b.burst = 1.1f64.max(2.6 - level * 0.05);
            let (bx, by, bt) = (b.cx, b.cy, b.t);
            let n = 10;
            for k in 0..n {
                let ang = (k as f64 / n as f64) * py::TAU + bt;
                self.enemy_shot(bx, by, ang.cos(), ang.sin(), 150.0);
            }
            let ang = (py_ - by).atan2(px - bx);
            self.enemy_shot(bx, by, ang.cos(), ang.sin(), 260.0);
        }
    }

    fn enemy_shot(&mut self, x: f64, y: f64, dx: f64, dy: f64, speed: f64) {
        let mut length = py::hypot(dx, dy);
        if length == 0.0 {
            length = 1.0;
        }
        self.ebullets.push(EBullet { x, y, vx: dx / length * speed, vy: dy / length * speed, rad: 4 });
    }

    fn update_bullets(&mut self, dt: f64) {
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        let bullets = std::mem::take(&mut self.bullets);
        let mut alive = Vec::new();
        for mut b in bullets {
            b.x += b.vx * dt;
            b.y += b.vy * dt;
            if b.x < -20.0 || b.x > w + 20.0 || b.y < -20.0 || b.y > h + 20.0 {
                continue;
            }
            if !self.arena && self.bullet_hits_shield(b.x, b.y) {
                continue;
            }
            if self.bullet_hits_alien(&mut b) {
                if b.pierce > 0 {
                    b.pierce -= 1;
                    alive.push(b);
                }
            } else {
                alive.push(b);
            }
        }
        self.bullets = alive;
    }

    fn bullet_hits_alien(&mut self, b: &mut Bullet) -> bool {
        for i in 0..self.aliens.len() {
            let a = self.aliens[i];
            if b.hits.contains(&a.id) {
                continue;
            }
            if (b.x - a.cx).abs() <= a.w / 2.0 + b.rad as f64 && (b.y - a.cy).abs() <= a.h / 2.0 + b.rad as f64 {
                b.hits.push(a.id);
                self.aliens[i].hp -= b.dmg;
                if self.aliens[i].hp <= 0 {
                    self.kill_alien(i);
                } else {
                    self.b.play_sound("hit");
                }
                return true;
            }
        }
        false
    }

    fn kill_alien(&mut self, i: usize) {
        let a = self.aliens.remove(i);
        self.b.score += kind_score(a.kind);
        self.spawn_particles(a.cx, a.cy, kind_col(a.kind), if a.kind == Art::Boss { 18 } else { 8 });
        self.b.play_sound("explode");
        if a.kind == Art::Boss {
            self.b.ach_event("inv_boss", None);
            self.b.rumble(220);
            self.drop_powerup(a.cx, a.cy);
        } else if py::random() < POWERUP_CHANCE {
            self.drop_powerup(a.cx, a.cy);
        }
    }

    fn bullet_hits_shield(&mut self, x: f64, y: f64) -> bool {
        for sh in self.shields.iter_mut() {
            if sh.hp > 0 && sh.x <= x && x <= sh.x + sh.w && sh.y <= y && y <= sh.y + sh.h {
                sh.hp -= 1;
                return true;
            }
        }
        false
    }

    fn update_ebullets(&mut self, dt: f64) {
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        let eb = std::mem::take(&mut self.ebullets);
        let mut alive = Vec::new();
        // _hit_player leert in Python die Liste, ueber die gerade iteriert
        // wird - die Schleife endet dort; die bis dahin gesammelten Kugeln
        // bleiben (self.ebullets = alive).
        for mut b in eb {
            b.x += b.vx * dt;
            b.y += b.vy * dt;
            if b.x < -20.0 || b.x > w + 20.0 || b.y < -20.0 || b.y > h + 20.0 {
                continue;
            }
            if !self.arena && self.bullet_hits_shield(b.x, b.y) {
                continue;
            }
            if self.can_be_hit() && (b.x - self.px).abs() <= PLAYER_W / 2.0 + b.rad as f64 && (b.y - self.py).abs() <= PLAYER_H / 2.0 + b.rad as f64 {
                self.hit_player();
                break;
            }
            alive.push(b);
        }
        self.ebullets = alive;
    }

    fn can_be_hit(&self) -> bool {
        self.invuln_timer <= 0.0 && self.shield_timer <= 0.0
    }

    fn drop_powerup(&mut self, x: f64, y: f64) {
        let kind = [PowerKind::Weapon, PowerKind::Shield, PowerKind::Life][py::weighted_index(&[6.0, 4.0, 2.0])];
        self.powerups.push(PowerUp { x, y, vy: POWERUP_FALL, kind });
    }

    fn update_powerups(&mut self, dt: f64) {
        let h = self.b.height as f64;
        let ps = std::mem::take(&mut self.powerups);
        let mut alive = Vec::new();
        for mut p in ps {
            p.y += p.vy * dt;
            if p.y > h + 20.0 {
                continue;
            }
            if (p.x - self.px).abs() < PLAYER_W && (p.y - self.py).abs() < PLAYER_H {
                self.collect_powerup(&p);
                continue;
            }
            alive.push(p);
        }
        self.powerups = alive;
    }

    fn collect_powerup(&mut self, p: &PowerUp) {
        self.b.play_sound("powerup");
        match p.kind {
            PowerKind::Life => self.lives = 9.min(self.lives + 1),
            PowerKind::Shield => self.shield_timer = 6.0,
            PowerKind::Weapon => {
                self.weapon = py::choice(&["spread", "rapid", "pierce"]);
                self.weapon_timer = if self.arena { 0.0 } else { WEAPON_PICKUP_TIME };
            }
        }
    }

    fn spawn_particles(&mut self, x: f64, y: f64, col: Color, n: usize) {
        for _ in 0..n {
            let ang = py::uniform(0.0, py::TAU);
            let spd = py::uniform(40.0, 190.0);
            let life = py::uniform(0.3, 0.7);
            self.particles.push(Particle { x, y, vx: ang.cos() * spd, vy: ang.sin() * spd, life, max: life, col });
        }
    }

    fn update_particles(&mut self, dt: f64) {
        let ps = std::mem::take(&mut self.particles);
        self.particles = ps
            .into_iter()
            .filter_map(|mut p| {
                p.life -= dt;
                if p.life <= 0.0 {
                    return None;
                }
                p.x += p.vx * dt;
                p.y += p.vy * dt;
                p.vx *= 0.92;
                p.vy *= 0.92;
                Some(p)
            })
            .collect();
    }

    fn hit_player(&mut self) {
        if !self.can_be_hit() {
            return;
        }
        self.lives -= 1;
        self.invuln_timer = HIT_INVULN;
        self.ebullets.clear();
        self.b.play_sound("hit");
        self.b.rumble(180);
        self.spawn_particles(self.px, self.py, COL_PLAYER, 14);
        if self.lives <= 0 {
            self.lives = 0;
            self.b.game_over = true;
            self.b.play_sound("gameover");
        }
    }

    fn next_level(&mut self) {
        self.level += 1;
        self.b.score += 50;
        self.b.play_sound("level");
        self.start_level();
    }

    fn draw_setup(&self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        let name = pgz_core::achievements::game_name(self.b.key);
        let glow = self.b.big_font.render(&name, true, self.b.accent);
        s.blr(&glow, glow.get_rect().center_at((w / 2 + 2, 72)));
        let title = self.b.big_font.render(&name, true, p.text);
        s.blr(&title, title.get_rect().center_at((w / 2, 70)));
        let sub = self.b.font.render(&t("inv.setup_title"), true, p.text_dim);
        s.blr(&sub, sub.get_rect().center_at((w / 2, 112)));
        let rows: [(String, Option<String>); 3] = [
            (t("inv.opt_move"), Some(if self.opt_move == "free" { t("inv.opt_move_free") } else { t("inv.opt_move_lr") })),
            (t("inv.opt_aim"), Some(if self.opt_aim == "mouse" { t("inv.opt_aim_mouse") } else { t("inv.opt_aim_fixed") })),
            (t("common.start"), None),
        ];
        for (i, r) in self.setup_rects.iter().enumerate() {
            let selected = i == self.setup_sel;
            draw::rect_r(s, if selected { p.btn_sel } else { p.btn }, *r, 0, 8);
            draw::rect_r(s, if selected { self.b.accent } else { p.border }, *r, 1, 8);
            let (label, value) = &rows[i];
            match value {
                None => {
                    let img = self.b.font.render(label, true, p.text);
                    s.blr(&img, img.get_rect().center_at(r.center()));
                }
                Some(v) => {
                    let limg = self.b.font.render(label, true, p.text);
                    s.bl(&limg, (r.x + 14, r.centery() - limg.h() / 2));
                    let vimg = self.b.font.render(&format!("< {v} >"), true, if selected { p.gold } else { p.text_dim });
                    s.bl(&vimg, (r.right() - vimg.w() - 14, r.centery() - vimg.h() / 2));
                }
            }
        }
        let hint = self.b.font.render(&t("inv.setup_hint"), true, p.text_dim);
        s.blr(&hint, hint.get_rect().center_at((w / 2, self.b.height - 24)));
    }

    fn draw_alien(&self, s: &mut Surface, a: &Alien) {
        let p = ui::pal();
        let mut col = kind_col(a.kind);
        if a.maxhp > 1 {
            let g = 0.4 + 0.6 * (a.hp as f64 / a.maxhp as f64);
            col = rgb((col.r as f64 * g) as i32, (col.g as f64 * g) as i32, (col.b as f64 * g) as i32);
        }
        let x = (a.cx - a.w / 2.0) as i32;
        let y = (a.cy - a.h / 2.0) as i32;
        let (aw, ah) = (a.w as i32, a.h as i32);
        draw::rect_r(s, col, (x, y, aw, ah), 0, 6);
        let ey = y + ah / 3;
        draw::rect(s, p.bg_top, (x + 7, ey, 4, 4), 0);
        draw::rect(s, p.bg_top, (x + aw - 11, ey, 4, 4), 0);
        if a.kind == Art::Boss {
            draw::rect(s, p.panel, (x, y - 10, aw, 5), 0);
            draw::rect(s, rgb(235, 90, 140), (x, y - 10, (aw as f64 * a.hp as f64 / a.maxhp as f64) as i32, 5), 0);
        }
    }

    fn draw_powerup(&self, s: &mut Surface, pu: &PowerUp) {
        let (letter, col) = match pu.kind {
            PowerKind::Life => ("+", rgb(240, 110, 120)),
            PowerKind::Shield => ("S", rgb(90, 170, 220)),
            PowerKind::Weapon => ("W", rgb(240, 210, 120)),
        };
        let c = (pu.x as i32, pu.y as i32);
        draw::circle(s, col, c, 11, 2);
        let img = self.b.font.render(letter, true, col);
        s.blr(&img, img.get_rect().center_at(c));
    }

    fn draw_player(&self, s: &mut Surface) {
        let col = if self.invuln_timer > 0.0 && (self.invuln_timer * 12.0) as i64 % 2 == 0 { COL_PLAYER_HIT } else { COL_PLAYER };
        let (fx, fy) = self.face;
        let ang = fy.atan2(fx);
        let tip = (self.px + ang.cos() * PLAYER_H * 0.8, self.py + ang.sin() * PLAYER_H * 0.8);
        let left = (self.px + (ang + 2.5).cos() * PLAYER_W * 0.5, self.py + (ang + 2.5).sin() * PLAYER_W * 0.5);
        let right = (self.px + (ang - 2.5).cos() * PLAYER_W * 0.5, self.py + (ang - 2.5).sin() * PLAYER_W * 0.5);
        draw::polygon(s, col, &[tip, left, right], 0);
        if self.shield_timer > 0.0 {
            let r = (PLAYER_W.max(PLAYER_H) * 0.8) as i32;
            draw::circle(s, COL_SHIELD, (self.px as i32, self.py as i32), r, 2);
        }
    }

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let mut bar = pg::surface_alpha(w, 32);
        bar.fill_c(pg::alpha(p.panel, 140));
        s.bl(&bar, (0, 0));
        draw::line(s, p.border, (0, 31), (w, 31), 1);
        let pts = self.b.font.render(&tf("common.points", &[("score", &self.b.score.to_string())]), true, p.text);
        s.bl(&pts, (10, 6));
        let lvl = self.b.font.render(&tf("inv.level", &[("level", &self.level.to_string())]), true, p.text);
        s.bl(&lvl, (w / 2 - lvl.w() / 2, 6));
        let leben = self.b.font.render(&tf("inv.lives", &[("lives", &self.lives.to_string())]), true, p.text);
        s.bl(&leben, (w - leben.w() - 10, 6));
        let mut wname = t(&format!("inv.wpn_{}", self.weapon));
        if self.weapon_timer > 0.0 {
            wname += &format!("  {:.0}s", self.weapon_timer);
        }
        let wimg = self.b.font.render(&tf("inv.weapon", &[("weapon", &wname)]), true, weapon_col(self.weapon));
        s.bl(&wimg, (10, h - 26));
    }

    fn draw_banner(&self, s: &mut Surface) {
        if self.banner_t <= 0.0 {
            return;
        }
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let mut band = pg::surface_alpha(w, 96);
        band.fill_c(pg::alpha(p.panel, 150));
        band.fill_r(pg::alpha(self.b.accent, 200), (0, 0, w, 2));
        band.fill_r(pg::alpha(self.b.accent, 200), (0, 94, w, 2));
        s.bl(&band, (0, h / 2 - 62));
        let img = self.b.big_font.render(&self.banner, true, p.text);
        s.blr(&img, img.get_rect().center_at((w / 2, h / 2 - 30)));
        let hint = if self.mouse_aim {
            t("inv.hint_mouse")
        } else if self.arena {
            t("inv.hint_free")
        } else if self.free_move {
            t("inv.hint_classic_free")
        } else {
            t("inv.hint_classic")
        };
        let himg = self.b.font.render(&hint, true, p.text_dim);
        s.blr(&himg, himg.get_rect().center_at((w / 2, h / 2 + 16)));
    }

    fn held_set(&mut self, act: &str, on: bool) {
        match act {
            "up" => self.held.up = on,
            "down" => self.held.down = on,
            "left" => self.held.left = on,
            "right" => self.held.right = on,
            _ => self.held.action = on,
        }
    }
}

impl Game for InvadersGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        self.arena = self.b.mode == "free";
        self.opt_move = "lr";
        self.opt_aim = "fixed";
        self.mouse_pos = (self.b.width as f64 / 2.0, self.b.height as f64 / 4.0);
        self.init_run();
        if self.arena {
            self.enter_play();
        } else {
            self.setup = true;
            self.build_setup();
        }
    }

    fn on_surface_changed(&mut self) {
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        self.px = 16f64.max((w - 16.0).min(self.px));
        self.py = 40f64.max((h - 16.0).min(self.py));
        if self.setup {
            self.build_setup();
        }
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if self.setup {
            self.setup_event(ev);
            return;
        }
        match ev.kind {
            Kind::MouseMove => {
                self.mouse_pos = (ev.pos.0 as f64, ev.pos.1 as f64);
                return;
            }
            Kind::MouseDown => {
                self.mouse_pos = (ev.pos.0 as f64, ev.pos.1 as f64);
                if !self.b.game_over {
                    self.fire();
                }
                return;
            }
            _ => {}
        }
        let acts = ["up", "down", "left", "right", "action"];
        if ev.kind == Kind::KeyUp {
            for act in acts {
                if self.b.is_action(&ev.key, act, None) {
                    self.held_set(act, false);
                }
            }
            return;
        }
        if ev.kind != Kind::KeyDown {
            return;
        }
        if self.b.game_over {
            if ev.key_in(&["Return", "space"]) {
                self.restart();
            }
            return;
        }
        if ev.key_in(&["1", "2", "3", "4"]) {
            if self.arena {
                self.select_weapon(WEAPON_ORDER[ev.key.parse::<usize>().unwrap_or(1) - 1]);
            }
            return;
        }
        for act in acts {
            if self.b.is_action(&ev.key, act, None) {
                self.held_set(act, true);
            }
        }
    }

    fn update(&mut self, dt: f64) {
        if self.b.game_over || self.setup {
            return;
        }
        self.cooldown = 0f64.max(self.cooldown - dt);
        self.shield_timer = 0f64.max(self.shield_timer - dt);
        self.invuln_timer = 0f64.max(self.invuln_timer - dt);
        self.banner_t = 0f64.max(self.banner_t - dt);
        if self.weapon_timer > 0.0 {
            self.weapon_timer -= dt;
            if self.weapon_timer <= 0.0 {
                self.weapon = "single";
            }
        }
        self.update_player(dt);
        if self.held.action {
            self.fire();
        }
        if self.arena {
            self.update_arena(dt);
        } else {
            self.update_classic(dt);
        }
        if self.free_move {
            self.check_contact_damage();
        }
        self.update_bullets(dt);
        self.update_ebullets(dt);
        self.update_powerups(dt);
        self.update_particles(dt);
        if self.aliens.is_empty() && self.banner_t <= 0.0 {
            self.next_level();
        }
    }

    fn draw(&mut self) {
        let p = ui::pal();
        let mut s = self.b.sf();
        ui::draw_background_ex(&mut s, self.b.width, self.b.height, true, None);
        if self.setup {
            self.draw_setup(&mut s);
            return;
        }
        for sh in &self.shields {
            if sh.hp <= 0 {
                continue;
            }
            let g = sh.hp as f64 / 6.0;
            let farbe = rgb((60.0 + 30.0 * g) as i32, (110.0 + 60.0 * g) as i32, (150.0 + 70.0 * g) as i32);
            draw::rect_r(&mut s, farbe, (sh.x, sh.y, sh.w, sh.h), 0, 4);
        }
        for a in &self.aliens {
            self.draw_alien(&mut s, a);
        }
        for pu in &self.powerups {
            self.draw_powerup(&mut s, pu);
        }
        for b in &self.bullets {
            draw::circle(&mut s, b.col, (b.x as i32, b.y as i32), b.rad + 1, 0);
        }
        for b in &self.ebullets {
            draw::circle(&mut s, COL_EBULLET, (b.x as i32, b.y as i32), b.rad, 0);
        }
        for pa in &self.particles {
            let a = 0f64.max(pa.life / pa.max);
            let r = 1.max((3.0 * a) as i32);
            draw::circle(&mut s, pa.col, (pa.x as i32, pa.y as i32), r, 0);
        }
        self.draw_player(&mut s);
        if self.mouse_aim {
            let (mx, my) = (self.mouse_pos.0 as i32, self.mouse_pos.1 as i32);
            draw::circle(&mut s, p.text, (mx, my), 8, 1);
            draw::line(&mut s, p.text, (mx - 11, my), (mx - 4, my), 1);
            draw::line(&mut s, p.text, (mx + 4, my), (mx + 11, my), 1);
        }
        self.draw_hud(&mut s);
        self.draw_banner(&mut s);
        if self.b.game_over {
            self.b.draw_center_text(&t("common.game_over"), &self.b.big_font, p.red, -20);
            self.b.draw_center_text(&t("common.enter_restart"), &self.b.font, p.text, 30);
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "invaders",
    class_name: "InvadersGame",
    supports_multiplayer: false,
    modes: &[("classic", "inv.mode_classic"), ("free", "inv.mode_free")],
    make: InvadersGame::new,
};
