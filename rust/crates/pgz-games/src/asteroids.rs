//! asteroids.rs
//! ============
//! Asteroids - Einzelspieler oder Koop-Duell (zwei Schiffe gleichzeitig).
//!
//! Uebertragen aus `games/asteroids.py`: Traegheitsphysik mit Wrap-Around,
//! zackige rotierende Brocken in drei Groessen, Wellen, UFO mit Zielfehler,
//! Power-Ups (Schild, Dreifach, Schnellfeuer), Hyperraum mit Restrisiko,
//! Extraleben, Partikel und Kamera-Wackeln.

use pgz_engine::prelude::*;

const COL_BG: (i32, i32, i32) = (8, 10, 20);
const COL_TEXT: Color = Color::rgb(232, 234, 240);
const COL_DIM: Color = Color::rgb(140, 148, 168);
const COL_OK: Color = Color::rgb(120, 200, 150);
const COL_BAD: Color = Color::rgb(245, 110, 110);
const COL_ROCK: Color = Color::rgb(205, 210, 225);
const COL_ROCK_FILL: Color = Color::rgb(26, 30, 44);
const COL_P1: Color = Color::rgb(120, 230, 160);
const COL_P2: Color = Color::rgb(140, 195, 255);
const COL_BULLET: Color = Color::rgb(245, 245, 250);
const COL_UFO: Color = Color::rgb(240, 150, 90);
const COL_UFO_SHOT: Color = Color::rgb(250, 110, 110);

const SHIP_R: f64 = 12.0;
const SHIP_TURN: f64 = 3.9;
const SHIP_THRUST: f64 = 300.0;
const SHIP_DAMP: f64 = 0.35;
const SHIP_MAX: f64 = 430.0;
const INVULN_T: f64 = 2.5;
const RESPAWN_T: f64 = 1.8;
const HYPER_CD: f64 = 4.0;
const HYPER_RISK: f64 = 0.12;
const EXTRA_LIFE_EVERY: i64 = 5000;
const BULLET_SPEED: f64 = 500.0;
const BULLET_LIFE: f64 = 0.95;
const SHOT_CD: f64 = 0.26;
const MAX_BULLETS: usize = 4;

fn rock_def(size: i32) -> (f64, f64, f64, i64) {
    match size {
        3 => (44.0, 40.0, 90.0, 20),
        2 => (26.0, 60.0, 130.0, 50),
        _ => (14.0, 90.0, 180.0, 100),
    }
}

const UFO_R: f64 = 16.0;
const UFO_SHOT_SPEED: f64 = 320.0;
const UFO_POINTS: i64 = 200;
const POWERUP_KINDS: [(&str, &str, Color, f64); 3] = [
    ("shield", "S", Color::rgb(110, 220, 220), 6.0),
    ("triple", "T", Color::rgb(180, 140, 255), 8.0),
    ("rapid", "R", Color::rgb(245, 205, 90), 8.0),
];
const POWERUP_DROP: f64 = 0.18;
const POWERUP_LIFE: f64 = 9.0;

struct Diff {
    key: &'static str,
    rocks: i64,
    speed: f64,
    ufo_every: f64,
    ufo_aim: f64,
}

const DIFFS: [Diff; 3] = [
    Diff { key: "easy", rocks: 3, speed: 0.75, ufo_every: 30.0, ufo_aim: 0.50 },
    Diff { key: "medium", rocks: 4, speed: 1.00, ufo_every: 24.0, ufo_aim: 0.28 },
    Diff { key: "hard", rocks: 5, speed: 1.30, ufo_every: 18.0, ufo_aim: 0.12 },
];

#[derive(Clone, Copy)]
struct Ship {
    home: (f64, f64),
    color: Color,
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    angle: f64,
    lives: i64,
    score: i64,
    next_extra: i64,
    alive: bool,
    invuln: f64,
    respawn: f64,
    cool: f64,
    hyper_cd: f64,
    powers: [Option<f64>; 3],
    thrusting: bool,
}

impl Ship {
    fn new(x: f64, y: f64, color: Color) -> Ship {
        Ship {
            home: (x, y),
            color,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            angle: -py::PI / 2.0,
            lives: 3,
            score: 0,
            next_extra: EXTRA_LIFE_EVERY,
            alive: true,
            invuln: INVULN_T,
            respawn: 0.0,
            cool: 0.0,
            hyper_cd: 0.0,
            powers: [None; 3],
            thrusting: false,
        }
    }
    fn has(&self, k: usize) -> bool {
        self.powers[k].is_some()
    }
}

const SHIELD: usize = 0;
const TRIPLE: usize = 1;
const RAPID: usize = 2;

#[derive(Clone)]
struct Rock {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    size: i32,
    r: f64,
    shape: Vec<f64>,
    rot: f64,
    spin: f64,
}

#[derive(Clone, Copy)]
struct Shot {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    life: f64,
    owner: usize,
}

#[derive(Clone, Copy)]
struct Item {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    kind: usize,
    life: f64,
}

#[derive(Clone, Copy)]
struct Ufo {
    x: f64,
    y: f64,
    base_y: f64,
    dir: f64,
    t: f64,
    shoot: f64,
}

#[derive(Clone, Copy, Default)]
struct Pressed {
    left: bool,
    right: bool,
    up: bool,
}

/// `AsteroidsGame`
pub struct AsteroidsGame {
    b: GameBase,
    winner: Option<usize>,
    diff: usize,
    ufos_on: bool,
    powerups_on: bool,
    small: Font,
    tiny: Font,
    tiny_mono: Font,
    anim_t: f64,
    stars: Vec<(i32, i32, i32, i32)>,
    diff_panel: Rect,
    diff_left: Rect,
    diff_right: Rect,
    ufo_rect: Rect,
    power_rect: Rect,
    start_rect: Rect,
    setup: bool,
    ships: Vec<Ship>,
    rocks: Vec<Rock>,
    bullets: Vec<Shot>,
    ufo_shots: Vec<Shot>,
    items: Vec<Item>,
    particles: Vec<([f64; 5], (i32, i32, i32))>,
    ufo: Option<Ufo>,
    ufo_timer: f64,
    wave: i64,
    banner_t: f64,
    pending_wave: bool,
    flash_msg: Option<(String, Color, f64)>,
    shake: f64,
    ending: f64,
    pressed: [Pressed; 2],
    hint_t: f64,
}

impl AsteroidsGame {
    /// `AsteroidsGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let mut g = AsteroidsGame {
            b,
            winner: None,
            diff: 1,
            ufos_on: true,
            powerups_on: true,
            small: f.clone(),
            tiny: f.clone(),
            tiny_mono: f,
            anim_t: 0.0,
            stars: Vec::new(),
            diff_panel: z,
            diff_left: z,
            diff_right: z,
            ufo_rect: z,
            power_rect: z,
            start_rect: z,
            setup: true,
            ships: Vec::new(),
            rocks: Vec::new(),
            bullets: Vec::new(),
            ufo_shots: Vec::new(),
            items: Vec::new(),
            particles: Vec::new(),
            ufo: None,
            ufo_timer: 0.0,
            wave: 1,
            banner_t: 0.0,
            pending_wave: true,
            flash_msg: None,
            shake: 0.0,
            ending: 0.0,
            pressed: [Pressed::default(); 2],
            hint_t: 0.0,
        };
        g.reset();
        Box::new(g)
    }

    fn make_fonts(&mut self) {
        let h = self.b.height as f64;
        let sz = |f: f64, lo: i32, hi: i32| lo.max(hi.min((h * f) as i32));
        self.b.font = ui::f(sz(0.040, 16, 26));
        self.b.big_font = ui::fb(sz(0.088, 30, 56));
        self.small = ui::f(sz(0.030, 13, 20));
        self.tiny = ui::f(sz(0.024, 11, 17));
        self.tiny_mono = ui::font(sz(0.024, 11, 17), false, true);
    }

    fn make_stars(&mut self) {
        let (w, h) = (self.b.width as i64, self.b.height as i64);
        self.stars = (0..70)
            .map(|_| {
                let x = py::randrange0(w) as i32;
                let y = py::randrange0(h) as i32;
                let gr = py::choice(&[1, 1, 2]);
                let hell = py::randint(60, 150) as i32;
                (x, y, gr, hell)
            })
            .collect();
    }

    fn start_run(&mut self) {
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        self.b.score = 0;
        self.b.game_over = false;
        self.winner = None;
        self.ships = if self.b.multiplayer {
            vec![Ship::new(w * 0.35, h / 2.0, COL_P1), Ship::new(w * 0.65, h / 2.0, COL_P2)]
        } else {
            vec![Ship::new(w / 2.0, h / 2.0, COL_P1)]
        };
        self.rocks.clear();
        self.bullets.clear();
        self.ufo_shots.clear();
        self.items.clear();
        self.particles.clear();
        self.ufo = None;
        self.ufo_timer = DIFFS[self.diff].ufo_every * py::uniform(0.7, 1.1);
        self.wave = 1;
        self.banner_t = 1.2;
        self.pending_wave = true;
        self.flash_msg = None;
        self.shake = 0.0;
        self.ending = 0.0;
        self.pressed = [Pressed::default(); 2];
        self.hint_t = 0.0;
    }

    fn build_setup_layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let bw = 420.min(w - 60);
        let top = 108.max((h as f64 * 0.22) as i32);
        let l = cx - py::fdiv32(bw, 2);
        self.diff_panel = Rect::new(l, top, bw, 56);
        self.diff_left = Rect::new(l, top, 40, 56);
        self.diff_right = Rect::new(self.diff_panel.right() - 40, top, 40, 56);
        let (bh, gap) = (42, 10);
        let y0 = top + 78;
        self.ufo_rect = Rect::new(l, y0, bw, bh);
        self.power_rect = Rect::new(l, y0 + (bh + gap), bw, bh);
        self.start_rect = Rect::new(cx - 95, y0 + 2 * (bh + gap) + 8, 190, 50);
    }

    fn cycle_diff(&mut self, step: i64) {
        self.diff = py::pmod(self.diff as i64 + step, 3) as usize;
        self.b.set_opt("asteroids", "difficulty", (self.diff as i64).into());
        self.b.play_sound("click");
    }

    fn toggle_ufos(&mut self) {
        self.ufos_on = !self.ufos_on;
        self.b.set_opt("asteroids", "ufos", self.ufos_on.into());
        self.b.play_sound("select");
    }

    fn toggle_powerups(&mut self) {
        self.powerups_on = !self.powerups_on;
        self.b.set_opt("asteroids", "powerups", self.powerups_on.into());
        self.b.play_sound("select");
    }

    fn start_play(&mut self) {
        self.start_run();
        self.setup = false;
        self.b.play_sound("click");
    }

    fn handle_setup_event(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown {
            match ev.key.as_str() {
                "Left" | "a" | "A" => self.cycle_diff(-1),
                "Right" | "d" | "D" => self.cycle_diff(1),
                "u" | "U" => self.toggle_ufos(),
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
            } else if self.ufo_rect.hit(p) {
                self.toggle_ufos();
            } else if self.power_rect.hit(p) {
                self.toggle_powerups();
            } else if self.start_rect.hit(p) {
                self.start_play();
            }
        }
    }

    fn scheme_ship(&self, scheme: usize) -> usize {
        if self.b.multiplayer && scheme == 1 {
            1
        } else {
            0
        }
    }

    fn ship_input(&self, i: usize) -> Pressed {
        if self.b.multiplayer {
            return self.pressed[i];
        }
        let (a, b) = (self.pressed[0], self.pressed[1]);
        Pressed { left: a.left || b.left, right: a.right || b.right, up: a.up || b.up }
    }

    fn try_shoot(&mut self, si: usize) {
        let sh = self.ships[si];
        if !sh.alive || sh.cool > 0.0 {
            return;
        }
        let max_b = if sh.has(RAPID) { 8 } else { MAX_BULLETS };
        if self.bullets.iter().filter(|b| b.owner == si).count() >= max_b {
            return;
        }
        let winkel = if sh.has(TRIPLE) { vec![sh.angle - 0.26, sh.angle, sh.angle + 0.26] } else { vec![sh.angle] };
        for a in winkel {
            self.bullets.push(Shot {
                x: sh.x + a.cos() * 14.0,
                y: sh.y + a.sin() * 14.0,
                vx: a.cos() * BULLET_SPEED + sh.vx,
                vy: a.sin() * BULLET_SPEED + sh.vy,
                life: BULLET_LIFE,
                owner: si,
            });
        }
        self.ships[si].cool = if sh.has(RAPID) { 0.12 } else { SHOT_CD };
        self.b.play_sound("shoot");
    }

    fn hyperspace(&mut self, si: usize) {
        let sh = self.ships[si];
        if !sh.alive || sh.hyper_cd > 0.0 {
            return;
        }
        self.spawn_particles(sh.x, sh.y, c3(sh.color), 10);
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        let s = &mut self.ships[si];
        s.x = py::uniform(40.0, w - 40.0);
        s.y = py::uniform(40.0, h - 40.0);
        s.vx *= 0.25;
        s.vy *= 0.25;
        s.hyper_cd = HYPER_CD;
        let (x, y, c) = (s.x, s.y, s.color);
        self.b.play_sound("rotate");
        if py::random() < HYPER_RISK {
            self.kill_ship(si);
        } else {
            self.spawn_particles(x, y, c3(c), 10);
        }
    }

    fn wrap(&self, mut x: f64, mut y: f64, m: f64) -> (f64, f64) {
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        if x < -m {
            x += w + 2.0 * m;
        } else if x > w + m {
            x -= w + 2.0 * m;
        }
        if y < -m {
            y += h + 2.0 * m;
        } else if y > h + m {
            y -= h + 2.0 * m;
        }
        (x, y)
    }

    fn update_ships(&mut self, dt: f64) {
        for i in 0..self.ships.len() {
            {
                let sh = &mut self.ships[i];
                sh.cool = 0f64.max(sh.cool - dt);
                sh.hyper_cd = 0f64.max(sh.hyper_cd - dt);
                for p in sh.powers.iter_mut() {
                    if let Some(v) = p.as_mut() {
                        *v -= dt;
                        if *v <= 0.0 {
                            *p = None;
                        }
                    }
                }
            }
            if !self.ships[i].alive {
                if self.ships[i].lives > 0 {
                    self.ships[i].respawn -= dt;
                    let home = self.ships[i].home;
                    if self.ships[i].respawn <= 0.0 && self.area_clear(home, 130.0) {
                        let sh = &mut self.ships[i];
                        sh.x = home.0;
                        sh.y = home.1;
                        sh.vx = 0.0;
                        sh.vy = 0.0;
                        sh.angle = -py::PI / 2.0;
                        sh.alive = true;
                        sh.invuln = INVULN_T;
                    }
                }
                continue;
            }
            let tasten = self.ship_input(i);
            let sh = &mut self.ships[i];
            sh.invuln = 0f64.max(sh.invuln - dt);
            if tasten.left {
                sh.angle -= SHIP_TURN * dt;
            }
            if tasten.right {
                sh.angle += SHIP_TURN * dt;
            }
            sh.thrusting = tasten.up;
            if sh.thrusting {
                sh.vx += sh.angle.cos() * SHIP_THRUST * dt;
                sh.vy += sh.angle.sin() * SHIP_THRUST * dt;
                if py::random() < dt * 40.0 {
                    let ba = sh.angle + py::PI + py::uniform(-0.4, 0.4);
                    let life = py::uniform(0.15, 0.3);
                    let p = [sh.x + ba.cos() * 12.0, sh.y + ba.sin() * 12.0, ba.cos() * 90.0 + sh.vx * 0.4, ba.sin() * 90.0 + sh.vy * 0.4, life];
                    self.particles.push((p, (250, 180, 90)));
                }
            }
            let sh = &mut self.ships[i];
            let f = 0f64.max(1.0 - SHIP_DAMP * dt);
            sh.vx *= f;
            sh.vy *= f;
            let sp = py::hypot(sh.vx, sh.vy);
            if sp > SHIP_MAX {
                sh.vx *= SHIP_MAX / sp;
                sh.vy *= SHIP_MAX / sp;
            }
            let (nx, ny) = (sh.x + sh.vx * dt, sh.y + sh.vy * dt);
            let (wx, wy) = self.wrap(nx, ny, SHIP_R);
            self.ships[i].x = wx;
            self.ships[i].y = wy;
        }
    }

    fn area_clear(&self, pos: (f64, f64), radius: f64) -> bool {
        self.rocks.iter().all(|r| py::hypot(r.x - pos.0, r.y - pos.1) > radius + r.r)
    }

    fn kill_ship(&mut self, si: usize) {
        if !self.ships[si].alive {
            return;
        }
        let sh = &mut self.ships[si];
        sh.alive = false;
        sh.lives -= 1;
        sh.respawn = RESPAWN_T;
        sh.powers = [None; 3];
        let (x, y, c) = (sh.x, sh.y, sh.color);
        self.shake = 0.55;
        self.spawn_particles(x, y, c3(c), 26);
        self.b.play_sound("hit");
        self.b.play_sound("explode");
        self.b.rumble(220);
        if self.ships.iter().all(|s| s.lives <= 0 && !s.alive) {
            self.ending = 1.4;
        }
    }

    fn finish(&mut self) {
        self.b.game_over = true;
        self.b.score = self.ships.iter().map(|s| s.score).max().unwrap_or(0);
        if self.b.multiplayer {
            let (s0, s1) = (self.ships[0].score, self.ships[1].score);
            self.winner = if s0 > s1 {
                Some(0)
            } else if s1 > s0 {
                Some(1)
            } else {
                None
            };
            if let Some(w) = self.winner {
                self.b.report_duel(w);
            }
            self.b.play_sound("win");
        } else {
            self.b.play_sound("gameover");
        }
        self.b.rumble(250);
    }

    fn add_points(&mut self, si: usize, pts: i64) {
        self.ships[si].score += pts;
        self.b.score = self.ships.iter().map(|s| s.score).max().unwrap_or(0);
        let sh = &mut self.ships[si];
        if sh.score >= sh.next_extra {
            sh.next_extra += EXTRA_LIFE_EVERY;
            sh.lives += 1;
            self.flash_msg = Some((t("ast.extra_life"), sh.color, 2.0));
            self.b.play_sound("powerup");
        }
    }

    fn spawn_wave(&mut self) {
        let anzahl = 10.min(DIFFS[self.diff].rocks + self.wave - 1);
        for _ in 0..anzahl {
            self.spawn_rock(3, None);
        }
    }

    fn spawn_rock(&mut self, size: i32, pos: Option<(f64, f64)>) {
        let (r, v0, v1, _) = rock_def(size);
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        let pos = match pos {
            Some(p) => p,
            None => {
                let (mut x, mut y) = (0.0, 0.0);
                for _ in 0..60 {
                    if py::random() < 0.5 {
                        x = py::choice(&[-r, w + r]);
                        y = py::uniform(0.0, h);
                    } else {
                        x = py::uniform(0.0, w);
                        y = py::choice(&[-r, h + r]);
                    }
                    if self.ships.iter().all(|s| py::hypot(x - s.x, y - s.y) > 150.0) {
                        break;
                    }
                }
                (x, y)
            }
        };
        let ang = py::uniform(0.0, py::TAU);
        let spd = py::uniform(v0, v1) * DIFFS[self.diff].speed;
        let shape: Vec<f64> = (0..11).map(|_| py::uniform(0.72, 1.12)).collect();
        let rot = py::uniform(0.0, py::TAU);
        let spin = py::uniform(-1.6, 1.6);
        self.rocks.push(Rock { x: pos.0, y: pos.1, vx: ang.cos() * spd, vy: ang.sin() * spd, size, r, shape, rot, spin });
    }

    fn break_rock(&mut self, ri: usize, ship: Option<usize>) {
        let pts = rock_def(self.rocks[ri].size).3;
        if let Some(si) = ship {
            self.add_points(si, pts);
        }
        let rock = self.rocks.remove(ri);
        self.spawn_particles(rock.x, rock.y, (200, 205, 220), 6 + rock.size as usize * 4);
        self.b.play_sound(if rock.size == 3 { "explode" } else { "hit" });
        if rock.size > 1 {
            for _ in 0..2 {
                self.spawn_rock(rock.size - 1, Some((rock.x, rock.y)));
            }
        }
        if self.powerups_on && self.items.len() < 2 && py::random() < POWERUP_DROP {
            let kind = py::randrange0(POWERUP_KINDS.len() as i64) as usize;
            let ang = py::uniform(0.0, py::TAU);
            self.items.push(Item { x: rock.x, y: rock.y, vx: ang.cos() * 30.0, vy: ang.sin() * 30.0, kind, life: POWERUP_LIFE });
        }
    }

    fn move_shots(&self, shots: &mut Vec<Shot>, dt: f64) {
        let mut rest = Vec::with_capacity(shots.len());
        for mut b in shots.drain(..) {
            b.life -= dt;
            if b.life <= 0.0 {
                continue;
            }
            let (x, y) = self.wrap(b.x + b.vx * dt, b.y + b.vy * dt, 4.0);
            b.x = x;
            b.y = y;
            rest.push(b);
        }
        *shots = rest;
    }

    fn update_bullets(&mut self, dt: f64) {
        let mut b = std::mem::take(&mut self.bullets);
        self.move_shots(&mut b, dt);
        self.bullets = b;
        let mut u = std::mem::take(&mut self.ufo_shots);
        self.move_shots(&mut u, dt);
        self.ufo_shots = u;
    }

    fn update_items(&mut self, dt: f64) {
        let items = std::mem::take(&mut self.items);
        let mut rest = Vec::new();
        for mut it in items {
            it.life -= dt;
            if it.life <= 0.0 {
                continue;
            }
            let (x, y) = self.wrap(it.x + it.vx * dt, it.y + it.vy * dt, 14.0);
            it.x = x;
            it.y = y;
            rest.push(it);
        }
        self.items = rest;
    }

    fn update_ufo(&mut self, dt: f64) {
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        let Some(mut u) = self.ufo else {
            self.ufo_timer -= dt;
            if self.ufo_timer <= 0.0 && self.banner_t <= 0.0 {
                let richtung = py::choice(&[-1.0, 1.0]);
                let base_y = py::uniform(60.0, h - 60.0);
                let shoot = py::uniform(0.8, 1.4);
                self.ufo = Some(Ufo { x: if richtung > 0.0 { -UFO_R } else { w + UFO_R }, y: base_y, base_y, dir: richtung, t: 0.0, shoot });
                self.b.play_sound("move");
            }
            return;
        };
        u.t += dt;
        u.x += u.dir * (95.0 + self.wave as f64 * 4.0) * dt;
        u.y = u.base_y + (u.t * 2.2).sin() * 40.0;
        if (u.dir > 0.0 && u.x > w + UFO_R) || (u.dir < 0.0 && u.x < -UFO_R) {
            self.ufo = None;
            self.ufo_timer = DIFFS[self.diff].ufo_every * py::uniform(0.8, 1.2);
            return;
        }
        u.shoot -= dt;
        if u.shoot <= 0.0 {
            u.shoot = 1.3;
            let ziele: Vec<usize> = (0..self.ships.len()).filter(|&i| self.ships[i].alive).collect();
            if !ziele.is_empty() {
                let z = self.ships[py::choice(&ziele)];
                let mut a = (z.y - u.y).atan2(z.x - u.x);
                a += py::uniform(-1.0, 1.0) * DIFFS[self.diff].ufo_aim;
                self.ufo_shots.push(Shot { x: u.x, y: u.y, vx: a.cos() * UFO_SHOT_SPEED, vy: a.sin() * UFO_SHOT_SPEED, life: 1.6, owner: 9 });
                self.b.play_sound("shoot");
            }
        }
        self.ufo = Some(u);
    }

    fn collisions(&mut self) {
        // Spielerschuesse gegen Brocken / UFO (Kopie der Liste wie in Python)
        let snapshot = self.bullets.clone();
        let mut alive: Vec<bool> = vec![true; snapshot.len()];
        for (bi, b) in snapshot.iter().enumerate() {
            if let Some(ri) = self.rocks.iter().position(|r| py::hypot(b.x - r.x, b.y - r.y) < r.r) {
                alive[bi] = false;
                self.break_rock(ri, Some(b.owner));
                continue;
            }
            if let Some(u) = self.ufo {
                if py::hypot(b.x - u.x, b.y - u.y) < UFO_R + 2.0 {
                    alive[bi] = false;
                    self.spawn_particles(u.x, u.y, (240, 150, 90), 18);
                    self.add_points(b.owner, UFO_POINTS);
                    self.ufo = None;
                    self.ufo_timer = DIFFS[self.diff].ufo_every * py::uniform(0.9, 1.3);
                    self.b.play_sound("point");
                    self.b.play_sound("explode");
                }
            }
        }
        let mut k = 0;
        self.bullets.retain(|_| {
            let keep = alive[k];
            k += 1;
            keep
        });

        for si in 0..self.ships.len() {
            if !self.ships[si].alive {
                continue;
            }
            let sh = self.ships[si];
            let verwundbar = sh.invuln <= 0.0 && !sh.has(SHIELD);
            if verwundbar && self.rocks.iter().any(|r| py::hypot(sh.x - r.x, sh.y - r.y) < r.r * 0.9 + SHIP_R) {
                self.kill_ship(si);
            }
            if !self.ships[si].alive {
                continue;
            }
            if verwundbar {
                if let Some(bi) = self.ufo_shots.iter().position(|b| py::hypot(sh.x - b.x, sh.y - b.y) < SHIP_R + 3.0) {
                    self.ufo_shots.remove(bi);
                    self.kill_ship(si);
                }
            }
            if !self.ships[si].alive {
                continue;
            }
            if verwundbar {
                if let Some(u) = self.ufo {
                    if py::hypot(sh.x - u.x, sh.y - u.y) < UFO_R + SHIP_R {
                        self.kill_ship(si);
                        continue;
                    }
                }
            }
            let items = self.items.clone();
            for it in items {
                if py::hypot(sh.x - it.x, sh.y - it.y) < SHIP_R + 14.0 {
                    let (_, _, farbe, dauer) = POWERUP_KINDS[it.kind];
                    self.ships[si].powers[it.kind] = Some(dauer);
                    if let Some(pos) = self.items.iter().position(|x| x.x == it.x && x.y == it.y && x.kind == it.kind && x.life == it.life) {
                        self.items.remove(pos);
                    }
                    self.spawn_particles(it.x, it.y, c3(farbe), 10);
                    self.b.play_sound("powerup");
                }
            }
        }
    }

    fn spawn_particles(&mut self, x: f64, y: f64, c: (i32, i32, i32), n: usize) {
        for _ in 0..n {
            let ang = py::uniform(0.0, py::TAU);
            let spd = py::uniform(40.0, 260.0);
            let life = py::uniform(0.25, 0.7);
            self.particles.push(([x, y, ang.cos() * spd, ang.sin() * spd, life], c));
        }
    }

    fn update_particles(&mut self, dt: f64) {
        for (p, _) in self.particles.iter_mut() {
            p[0] += p[2] * dt;
            p[1] += p[3] * dt;
            p[2] *= 0.985;
            p[3] *= 0.985;
            p[4] -= dt;
        }
        self.particles.retain(|(p, _)| p[4] > 0.0);
    }

    fn draw_rock(&self, s: &mut Surface, r: &Rock, ox: f64, oy: f64) {
        let n = r.shape.len() as f64;
        let pts: Vec<(f64, f64)> = r
            .shape
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let a = r.rot + i as f64 * py::TAU / n;
                (r.x + a.cos() * r.r * f + ox, r.y + a.sin() * r.r * f + oy)
            })
            .collect();
        draw::polygon(s, COL_ROCK_FILL, &pts, 0);
        draw::polygon(s, COL_ROCK, &pts, 2);
    }

    fn draw_ship(&self, s: &mut Surface, sh: &Ship, ox: f64, oy: f64) {
        if !sh.alive {
            return;
        }
        if sh.invuln > 0.0 && (self.anim_t * 12.0) as i64 % 2 == 0 {
            return;
        }
        let a = sh.angle;
        let (x, y) = (sh.x + ox, sh.y + oy);
        let nase = (x + a.cos() * 16.0, y + a.sin() * 16.0);
        let hl = (x + (a + 2.5).cos() * 13.0, y + (a + 2.5).sin() * 13.0);
        let hr = (x + (a - 2.5).cos() * 13.0, y + (a - 2.5).sin() * 13.0);
        let heck = (x + (a + py::PI).cos() * 6.0, y + (a + py::PI).sin() * 6.0);
        let dunkel = rgb(sh.color.r / 3, sh.color.g / 3, sh.color.b / 3);
        draw::polygon(s, dunkel, &[nase, hl, heck, hr], 0);
        draw::polygon(s, sh.color, &[nase, hl, heck, hr], 2);
        if sh.thrusting && py::random() < 0.85 {
            let fl = 10.0 + py::uniform(0.0, 8.0);
            let fx = (x + (a + py::PI).cos() * (8.0 + fl), y + (a + py::PI).sin() * (8.0 + fl));
            draw::polygon(s, rgb(250, 180, 90), &[hl, fx, hr], 0);
        }
        if sh.has(SHIELD) {
            let r = 20 + (2.0 * (self.anim_t * 8.0).sin()) as i32;
            draw::circle(s, rgb(110, 220, 220), (x as i32, y as i32), r, 2);
        }
    }

    fn draw_ufo(&self, s: &mut Surface, u: &Ufo, ox: f64, oy: f64) {
        let (x, y) = ((u.x + ox) as i32, (u.y + oy) as i32);
        draw::ellipse(s, rgb(40, 30, 26), (x - 18, y - 6, 36, 14), 0);
        draw::ellipse(s, COL_UFO, (x - 18, y - 6, 36, 14), 2);
        draw::arc(s, COL_UFO, (x - 9, y - 14, 18, 16), 0.0, py::PI, 2);
        draw::line(s, COL_UFO, (x - 18, y + 1), (x + 18, y + 1), 1);
    }

    fn draw_item(&self, s: &mut Surface, it: &Item, ox: f64, oy: f64) {
        let (_, sym, farbe, _) = POWERUP_KINDS[it.kind];
        if it.life < 2.5 && (self.anim_t * 6.0) as i64 % 2 == 0 {
            return;
        }
        let (x, y) = ((it.x + ox) as i32, (it.y + oy) as i32);
        let r = 12 + (1.5 * (self.anim_t * 5.0).sin()) as i32;
        draw::circle(s, rgb(22, 27, 40), (x, y), r, 0);
        draw::circle(s, farbe, (x, y), r, 2);
        let img = self.tiny.render(sym, true, farbe);
        s.blr(&img, img.get_rect().center_at((x, y)));
    }

    fn draw_hud(&self, s: &mut Surface) {
        let (w, h) = (self.b.width, self.b.height);
        for (i, sh) in self.ships.iter().enumerate() {
            let rechts = i == 1;
            let x = if rechts { w - 12 } else { 12 };
            let img = self.b.font.render(&sh.score.to_string(), true, sh.color);
            s.bl(&img, (if rechts { x - img.w() } else { x }, 8));
            for lv in 0..sh.lives.max(0) as i32 {
                let lx = if rechts { x - 14 - lv * 16 } else { x + 6 + lv * 16 };
                draw::polygon(s, sh.color, &[(lx + 5, 38), (lx, 50), (lx + 5, 47), (lx + 10, 50)], 1);
            }
            let mut y = 58;
            for (k, (_, sym, farbe, _)) in POWERUP_KINDS.iter().enumerate() {
                if let Some(rest) = sh.powers[k] {
                    let txt = self.tiny_mono.render(&format!("{sym} {:2}s", rest.ceil() as i64), true, *farbe);
                    s.bl(&txt, (if rechts { x - txt.w() } else { x }, y));
                    y += 15;
                }
            }
            if sh.alive {
                let txt = if sh.hyper_cd <= 0.0 {
                    self.tiny.render(&t("ast.hyper"), true, COL_OK)
                } else {
                    self.tiny_mono.render(&format!("{} {}s", t("ast.hyper"), sh.hyper_cd.ceil() as i64), true, COL_DIM)
                };
                s.bl(&txt, (if rechts { x - txt.w() } else { x }, y));
            }
        }
        let wimg = self.small.render(&tf("ast.wave", &[("n", &self.wave.to_string())]), true, self.b.accent);
        s.blr(&wimg, wimg.get_rect().midtop_at((w / 2, 8)));
        if self.banner_t > 0.0 && !self.b.game_over {
            let mut img = self.b.big_font.render(&tf("ast.wave", &[("n", &self.wave.to_string())]), true, self.b.accent);
            img.set_alpha(Some(0.max(255.min((255.0 * self.banner_t) as i32)) as u8));
            s.blr(&img, img.get_rect().center_at((w / 2, h / 2 - 40)));
        }
        if let Some((text, farbe, _)) = &self.flash_msg {
            let img = self.b.font.render(text, true, *farbe);
            s.blr(&img, img.get_rect().midtop_at((w / 2, 40)));
        }
        if self.hint_t < 6.0 {
            let hint = self.tiny.render(&t("ast.controls_hint"), true, COL_DIM);
            s.blr(&hint, hint.get_rect().midbottom_at((w / 2, h - 8)));
        }
    }

    fn draw_game_over(&self, s: &mut Surface) {
        let (w, h) = (self.b.width, self.b.height);
        let mut ov = pg::surface_alpha(w, h);
        ov.fill_c(rgba(0, 0, 0, 150));
        s.bl(&ov, (0, 0));
        let b = &self.b;
        if b.multiplayer {
            let (text, farbe) = match self.winner {
                None => (t("common.draw"), COL_TEXT),
                Some(wn) => {
                    let name: String = b.pname(wn).chars().take(10).collect();
                    (tf("common.player_wins_name", &[("name", &name)]), self.ships[wn].color)
                }
            };
            b.draw_center_text(&text, &b.big_font, farbe, -50);
            let n0: String = b.pname(0).chars().take(8).collect();
            let n1: String = b.pname(1).chars().take(8).collect();
            let stand = format!("{n0}: {}    {n1}: {}", self.ships[0].score, self.ships[1].score);
            b.draw_center_text(&stand, &b.font, COL_TEXT, -6);
        } else {
            b.draw_center_text(&t("common.game_over"), &b.big_font, COL_BAD, -50);
            b.draw_center_text(&tf("common.points", &[("score", &self.ships[0].score.to_string())]), &b.font, COL_TEXT, -6);
        }
        b.draw_center_text(&tf("ast.wave_reached", &[("n", &self.wave.to_string())]), &b.font, COL_DIM, 24);
        b.draw_center_text(&t("ah.restart_hint"), &b.font, COL_TEXT, 56);
    }

    fn draw_row(&self, s: &mut Surface, rect: Rect, label: &str, wert: &str, an: bool) {
        let p = ui::pal();
        draw::rect_r(s, if an { p.btn_sel } else { p.btn }, rect, 0, 8);
        draw::rect_r(s, if an { self.b.accent } else { p.border }, rect, 1, 8);
        let lab = self.b.font.render(label, true, p.text);
        s.bl(&lab, (rect.x + 16, rect.centery() - lab.h() / 2));
        let img = self.b.font.render(&format!("< {wert} >"), true, if an { self.b.accent } else { p.text_dim });
        s.bl(&img, (rect.right() - img.w() - 16, rect.centery() - img.h() / 2));
    }

    fn draw_setup(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        ui::draw_background_ex(s, w, h, true, None);
        let modus = if self.b.multiplayer { t("snake.multiplayer") } else { t("snake.singleplayer") };
        ui::draw_title_ex(s, w, "ASTEROIDS", Some(&modus), 52, None, None, Some(self.b.accent));
        let d = &DIFFS[self.diff];
        draw::rect_r(s, p.panel, self.diff_panel, 0, 10);
        draw::rect_r(s, self.b.accent, self.diff_panel, 2, 10);
        let name = self.b.font.render(&format!("{}:  {}", t("ah.difficulty"), t(&format!("ah.diff.{}", d.key))), true, p.text);
        s.blr(&name, name.get_rect().center_at((self.diff_panel.centerx(), self.diff_panel.y + 19)));
        let info = self.tiny.render(&t("ast.diff_note"), true, p.text_dim);
        s.blr(&info, info.get_rect().center_at((self.diff_panel.centerx(), self.diff_panel.y + 41)));
        for (r, sym) in [(self.diff_left, "<"), (self.diff_right, ">")] {
            let arr = self.b.big_font.render(sym, true, self.b.accent);
            s.blr(&arr, arr.get_rect().center_at(r.center()));
        }
        let onoff = |b: bool| if b { t("common.on") } else { t("common.off") };
        self.draw_row(s, self.ufo_rect, &t("ast.ufos"), &onoff(self.ufos_on), self.ufos_on);
        self.draw_row(s, self.power_rect, &t("ah.powerups"), &onoff(self.powerups_on), self.powerups_on);
        ui::draw_button_ex(s, self.start_rect, &t("common.start"), &self.b.font, true, None, None, Some(p.green));
        let hint = self.small.render(&t("ast.setup_hint"), true, p.text_dim);
        s.blr(&hint, hint.get_rect().center_at((w / 2, h - 38)));
        let h2 = self.tiny.render(&t("ast.controls_hint"), true, p.green);
        s.blr(&h2, h2.get_rect().center_at((w / 2, h - 16)));
    }
}

impl Game for AsteroidsGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        self.b.score = 0;
        self.b.game_over = false;
        self.winner = None;
        self.diff = self.b.opt_i64("asteroids", "difficulty", 1).clamp(0, 2) as usize;
        self.ufos_on = self.b.opt_bool("asteroids", "ufos", true);
        self.powerups_on = self.b.opt_bool("asteroids", "powerups", true);
        self.make_fonts();
        self.anim_t = 0.0;
        self.make_stars();
        self.build_setup_layout();
        self.setup = true;
        self.start_run();
    }

    fn on_surface_changed(&mut self) {
        self.make_fonts();
        self.make_stars();
        self.build_setup_layout();
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        if self.b.multiplayer && self.ships.len() == 2 {
            self.ships[0].home = (w * 0.35, h / 2.0);
            self.ships[1].home = (w * 0.65, h / 2.0);
        } else {
            for sh in self.ships.iter_mut() {
                sh.home = (w / 2.0, h / 2.0);
            }
        }
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if self.setup {
            self.handle_setup_event(ev);
            return;
        }
        let acts = ["left", "right", "up"];
        let schemes = ["p1", "p2"];
        if ev.kind == Kind::KeyUp {
            for (si, scheme) in schemes.iter().enumerate() {
                for act in acts {
                    if self.b.is_action(&ev.key, act, Some(scheme)) {
                        set_act(&mut self.pressed[si], act, false);
                    }
                }
            }
            return;
        }
        if ev.kind != Kind::KeyDown {
            return;
        }
        if self.b.game_over {
            if ev.key_in(&["Return", "space"]) {
                self.start_play();
            } else if ev.key_in(&["s", "S"]) {
                self.setup = true;
                self.b.play_sound("click");
            }
            return;
        }
        for (si, scheme) in schemes.iter().enumerate() {
            for act in acts {
                if self.b.is_action(&ev.key, act, Some(scheme)) {
                    set_act(&mut self.pressed[si], act, true);
                }
            }
            if self.b.is_action(&ev.key, "action", Some(scheme)) {
                let ship = self.scheme_ship(si);
                self.try_shoot(ship);
            }
            if self.b.is_action(&ev.key, "down", Some(scheme)) {
                let ship = self.scheme_ship(si);
                self.hyperspace(ship);
            }
        }
    }

    fn update(&mut self, dt: f64) {
        self.anim_t += dt;
        self.update_particles(dt);
        if self.shake > 0.0 {
            self.shake = 0f64.max(self.shake - dt * 1.6);
        }
        if self.setup || self.b.game_over {
            return;
        }
        self.hint_t += dt;
        if let Some((text, farbe, rest)) = self.flash_msg.take() {
            if rest > dt {
                self.flash_msg = Some((text, farbe, rest - dt));
            }
        }
        if self.banner_t > 0.0 {
            self.banner_t -= dt;
            if self.banner_t <= 0.0 && self.pending_wave {
                self.spawn_wave();
                self.pending_wave = false;
            }
        }
        self.update_ships(dt);
        self.update_bullets(dt);
        for i in 0..self.rocks.len() {
            let r = &self.rocks[i];
            let (x, y) = self.wrap(r.x + r.vx * dt, r.y + r.vy * dt, r.r);
            let r = &mut self.rocks[i];
            r.x = x;
            r.y = y;
            r.rot += r.spin * dt;
        }
        self.update_items(dt);
        if self.ufos_on {
            self.update_ufo(dt);
        }
        self.collisions();
        if self.rocks.is_empty() && !self.pending_wave && self.ufo.is_none() && self.banner_t <= 0.0 && self.ending <= 0.0 {
            self.wave += 1;
            self.banner_t = 2.0;
            self.pending_wave = true;
            self.b.play_sound("level");
            if self.wave >= 10 {
                self.b.ach_event("ast_wave10", None);
            }
        }
        if self.ending > 0.0 {
            self.ending -= dt;
            if self.ending <= 0.0 {
                self.finish();
            }
        }
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        if self.setup {
            self.draw_setup(&mut s);
            return;
        }
        s.fill_c(rgb(COL_BG.0, COL_BG.1, COL_BG.2));
        let (mut ox, mut oy) = (0.0, 0.0);
        if self.shake > 0.0 {
            let amp = 10.0 * self.shake;
            ox = py::uniform(-amp, amp);
            oy = py::uniform(-amp, amp);
        }
        for &(x, y, gr, hell) in &self.stars {
            let tw = hell + (30.0 * (self.anim_t * 1.5 + x as f64).sin()) as i32;
            let c = 40.max(200.min(tw));
            draw::circle(&mut s, rgb(c, c, 255.min(c + 25)), (x, y), gr, 0);
        }
        for (p, col) in &self.particles {
            let a = 0f64.max(1f64.min(p[4] / 0.7));
            let m = |c: i32, bg: i32| (c as f64 * a + bg as f64 * (1.0 - a)) as i32;
            draw::circle(&mut s, rgb(m(col.0, COL_BG.0), m(col.1, COL_BG.1), m(col.2, COL_BG.2)), ((p[0] + ox) as i32, (p[1] + oy) as i32), 2, 0);
        }
        for r in &self.rocks {
            self.draw_rock(&mut s, r, ox, oy);
        }
        for it in &self.items {
            self.draw_item(&mut s, it, ox, oy);
        }
        if let Some(u) = &self.ufo {
            self.draw_ufo(&mut s, u, ox, oy);
        }
        for b in &self.bullets {
            draw::circle(&mut s, COL_BULLET, ((b.x + ox) as i32, (b.y + oy) as i32), 2, 0);
        }
        for b in &self.ufo_shots {
            draw::circle(&mut s, COL_UFO_SHOT, ((b.x + ox) as i32, (b.y + oy) as i32), 3, 0);
        }
        for sh in &self.ships {
            self.draw_ship(&mut s, sh, ox, oy);
        }
        self.draw_hud(&mut s);
        if self.b.game_over {
            self.draw_game_over(&mut s);
        }
    }
}

fn set_act(p: &mut Pressed, act: &str, on: bool) {
    match act {
        "left" => p.left = on,
        "right" => p.right = on,
        _ => p.up = on,
    }
}

fn c3(c: Color) -> (i32, i32, i32) {
    (c.r as i32, c.g as i32, c.b as i32)
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "asteroids",
    class_name: "AsteroidsGame",
    supports_multiplayer: true,
    modes: &[],
    make: AsteroidsGame::new,
};
