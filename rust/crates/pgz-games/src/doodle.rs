//! doodle.rs
//! =========
//! Doodle Jump - Klon mit Extras.
//!
//! Uebertragen aus `games/doodle.py`: automatisches Springen, Traegheit,
//! Wrap-around, vier Plattform-Arten, Feder und Propeller-Hut, Monster zum
//! Abschiessen, drei Stufen, Kamera folgt dem Aufstieg.

use pgz_core::highscore;
use pgz_engine::prelude::*;

const GRAV: f64 = 1350.0;
const JUMP_V: f64 = -720.0;
const SPRING_V: f64 = -1250.0;
const PROP_V: f64 = -560.0;
const MOVE_ACC: f64 = 1500.0;
const MOVE_MAX: f64 = 430.0;
const MOVE_FRICTION: f64 = 0.86;
const SHOOT_CD: f64 = 0.28;
const PROP_TIME: f64 = 2.6;

struct Diff {
    key: &'static str,
    gap: f64,
    mv: f64,
    brittle: f64,
    monster: f64,
}

const DIFFS: [Diff; 3] = [
    Diff { key: "easy", gap: 62.0, mv: 0.12, brittle: 0.10, monster: 0.05 },
    Diff { key: "normal", gap: 74.0, mv: 0.20, brittle: 0.18, monster: 0.09 },
    Diff { key: "hard", gap: 86.0, mv: 0.28, brittle: 0.24, monster: 0.14 },
];

type C3 = (i32, i32, i32);
const COL_BG_TOP: C3 = (222, 236, 250);
const COL_BG_BOT: C3 = (238, 246, 252);
const COL_GRID: Color = Color::rgb(220, 228, 240);
const COL_DOODLE: C3 = (120, 210, 90);
const COL_DOODLE_DARK: Color = Color::rgb(70, 150, 55);

#[derive(Clone, Copy, PartialEq)]
enum Art {
    Normal,
    Move,
    Brittle,
    Vanish,
}

fn plat_color(k: Art) -> C3 {
    match k {
        Art::Normal => (110, 205, 90),
        Art::Move => (90, 160, 235),
        Art::Brittle => (190, 130, 80),
        Art::Vanish => (235, 235, 240),
    }
}

fn c3(c: C3) -> Color {
    rgb(c.0, c.1, c.2)
}

fn scaled7(c: C3) -> Color {
    rgb((c.0 as f64 * 0.7) as i32, (c.1 as f64 * 0.7) as i32, (c.2 as f64 * 0.7) as i32)
}

#[derive(Clone, Copy)]
struct Plat {
    x: f64,
    y: f64,
    w: i32,
    kind: Art,
    vx: f64,
    spring: bool,
    prop: bool,
    dead: bool,
    used: bool,
}

impl Plat {
    fn new(x: f64, y: f64, w: i32, kind: Art) -> Plat {
        Plat { x, y, w, kind, vx: 0.0, spring: false, prop: false, dead: false, used: false }
    }
}

#[derive(Clone, Copy)]
struct Monster {
    x: f64,
    y: f64,
    vx: f64,
    hp: i32,
    kind: i64,
    bob: f64,
}

#[derive(Clone, Copy)]
struct Bullet {
    x: f64,
    y: f64,
    vy: f64,
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Setup,
    Play,
    GameOver,
}

/// `DoodleGame`
pub struct DoodleGame {
    b: GameBase,
    diff: usize,
    highscore: i64,
    anim_t: f64,
    small: Font,
    tiny: Font,
    huge: Font,
    plat_w: i32,
    dr: i32,
    diff_panel: Rect,
    diff_left: Rect,
    diff_right: Rect,
    start_rect: Rect,
    cam_y: f64,
    x: f64,
    y: f64,
    py_prev: f64,
    vx: f64,
    vy: f64,
    face: i32,
    shoot_cd: f64,
    shoot_pose: f64,
    prop_t: f64,
    start_y: f64,
    max_rise: f64,
    press_l: bool,
    press_r: bool,
    platforms: Vec<Plat>,
    monsters: Vec<Monster>,
    bullets: Vec<Bullet>,
    particles: Vec<([f64; 5], C3)>,
    new_best: bool,
    over_t: f64,
    prev_kind: Option<Art>,
    gen_y: f64,
    state: State,
    ui_bg: Option<((i32, i32, Color, Color), Surface)>,
    bg: Option<((i32, i32), Surface)>,
}

impl DoodleGame {
    /// `DoodleGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let mut g = DoodleGame {
            b,
            diff: 1,
            highscore: 0,
            anim_t: 0.0,
            small: f.clone(),
            tiny: f.clone(),
            huge: f,
            plat_w: 46,
            dr: 12,
            diff_panel: z,
            diff_left: z,
            diff_right: z,
            start_rect: z,
            cam_y: 0.0,
            x: 0.0,
            y: 0.0,
            py_prev: 0.0,
            vx: 0.0,
            vy: 0.0,
            face: 1,
            shoot_cd: 0.0,
            shoot_pose: 0.0,
            prop_t: 0.0,
            start_y: 0.0,
            max_rise: 0.0,
            press_l: false,
            press_r: false,
            platforms: Vec::new(),
            monsters: Vec::new(),
            bullets: Vec::new(),
            particles: Vec::new(),
            new_best: false,
            over_t: 0.0,
            prev_kind: None,
            gen_y: 0.0,
            state: State::Setup,
            ui_bg: None,
            bg: None,
        };
        g.reset();
        Box::new(g)
    }

    fn make_fonts(&mut self) {
        let h = self.b.height;
        self.b.font = ui::f(18.max(26.min(h / 26)));
        self.b.big_font = ui::fb(32.max(52.min(h / 12)));
        self.small = ui::f(14.max(20.min(h / 32)));
        self.tiny = ui::f(12.max(16.min(h / 42)));
        self.huge = ui::fb(26.max(h / 11));
    }

    fn apply_layout(&mut self) {
        self.make_fonts();
        self.plat_w = 46.max((self.b.width as f64 * 0.16) as i32);
        self.dr = 12.max((self.b.height as f64 * 0.03) as i32);
        self.build_setup_layout();
    }

    fn new_game(&mut self) {
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        self.b.score = 0;
        self.b.game_over = false;
        self.cam_y = 0.0;
        self.x = w / 2.0;
        self.y = h * 0.7;
        self.py_prev = self.y;
        self.vx = 0.0;
        self.vy = JUMP_V;
        self.face = 1;
        self.shoot_cd = 0.0;
        self.shoot_pose = 0.0;
        self.prop_t = 0.0;
        self.start_y = self.y;
        self.max_rise = 0.0;
        self.press_l = false;
        self.press_r = false;
        self.platforms.clear();
        self.monsters.clear();
        self.bullets.clear();
        self.particles.clear();
        self.new_best = false;
        self.over_t = 0.0;
        self.prev_kind = None;
        self.platforms.push(Plat::new(w / 2.0 - self.plat_w as f64 / 2.0, h * 0.75, self.plat_w, Art::Normal));
        self.gen_y = h * 0.75;
        self.fill_platforms();
        self.state = State::Play;
    }

    fn difficulty_scale(&self) -> f64 {
        1.0f64.min(self.b.score as f64 / 4000.0)
    }

    fn build_setup_layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let bw = 440.min(w - 60);
        let ph = 56.max(72.min((h as f64 * 0.14) as i32));
        let y0 = 140.max((h as f64 * 0.34) as i32);
        self.diff_panel = Rect::new(cx - py::fdiv32(bw, 2), y0, bw, ph);
        self.diff_left = Rect::new(self.diff_panel.x, y0, 44, ph);
        self.diff_right = Rect::new(self.diff_panel.right() - 44, y0, 44, ph);
        self.start_rect = Rect::new(cx - 95, self.diff_panel.bottom() + 26, 190, 52);
    }

    fn cycle_diff(&mut self, step: i64) {
        self.diff = py::pmod(self.diff as i64 + step, DIFFS.len() as i64) as usize;
        self.b.set_opt("doodle", "difficulty", (self.diff as i64).into());
        self.b.play_sound("click");
    }

    fn handle_setup(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown {
            match ev.key.as_str() {
                "Left" | "a" | "A" => self.cycle_diff(-1),
                "Right" | "d" | "D" => self.cycle_diff(1),
                "Return" | "space" => {
                    self.new_game();
                    self.b.play_sound("click");
                }
                _ => {}
            }
        } else if ev.kind == Kind::MouseDown {
            let p = ev.pos;
            if self.diff_left.hit(p) {
                self.cycle_diff(-1);
            } else if self.diff_right.hit(p) || self.diff_panel.hit(p) {
                self.cycle_diff(1);
            } else if self.start_rect.hit(p) {
                self.new_game();
                self.b.play_sound("click");
            }
        }
    }

    fn is_left(&self, k: &str) -> bool {
        self.b.is_action(k, "left", None) || k == "Left"
    }

    fn is_right(&self, k: &str) -> bool {
        self.b.is_action(k, "right", None) || k == "Right"
    }

    fn is_shoot(&self, k: &str) -> bool {
        self.b.is_action(k, "up", None) || matches!(k, "Up" | "space" | "w" | "W")
    }

    fn shoot(&mut self) {
        if self.shoot_cd > 0.0 {
            return;
        }
        self.shoot_cd = SHOOT_CD;
        self.shoot_pose = 0.25;
        self.bullets.push(Bullet { x: self.x, y: self.y - self.dr as f64, vy: -820.0 });
        self.b.play_sound("shoot");
    }

    fn platform_collisions(&mut self) {
        let dr = self.dr as f64;
        let prev_feet = self.py_prev + dr;
        let feet = self.y + dr;
        for i in 0..self.platforms.len() {
            let p = self.platforms[i];
            if p.dead {
                continue;
            }
            if !(p.x <= self.x && self.x <= p.x + p.w as f64) {
                continue;
            }
            if !(prev_feet <= p.y + 6.0 && feet >= p.y - 2.0) {
                continue;
            }
            if p.kind == Art::Brittle {
                self.platforms[i].dead = true;
                self.sparkle(self.x, p.y, plat_color(Art::Brittle), 9);
                self.b.play_sound("hit");
                continue;
            }
            self.y = p.y - dr;
            if p.prop {
                self.prop_t = PROP_TIME;
                self.platforms[i].prop = false;
                self.b.play_sound("powerup");
                self.sparkle(self.x, p.y, (120, 200, 255), 9);
            } else if p.spring {
                self.vy = SPRING_V;
                self.b.play_sound("powerup");
                self.sparkle(self.x, p.y, (255, 220, 120), 9);
            } else {
                self.vy = JUMP_V;
                self.b.play_sound("bounce");
            }
            if p.kind == Art::Vanish && !p.used {
                self.platforms[i].used = true;
                self.platforms[i].dead = true;
            }
            break;
        }
    }

    fn update_platforms(&mut self, dt: f64) {
        let w = self.b.width as f64;
        for p in self.platforms.iter_mut() {
            if p.kind == Art::Move && !p.dead {
                p.x += p.vx * dt;
                if p.x < 0.0 || p.x + p.w as f64 > w {
                    p.vx = -p.vx;
                    p.x = 0f64.max((w - p.w as f64).min(p.x));
                }
            }
        }
    }

    fn update_monsters(&mut self, dt: f64) {
        let w = self.b.width as f64;
        let reach = (self.dr + 16) as f64;
        for i in 0..self.monsters.len() {
            let m = &mut self.monsters[i];
            m.x += m.vx * dt;
            if m.x < 20.0 || m.x > w - 20.0 {
                m.vx = -m.vx;
            }
            let (mx, my) = (m.x, m.y);
            let d = py::hypot(self.x - mx, self.y - my);
            if self.prop_t <= 0.0 && d < reach {
                self.die();
                return;
            } else if self.prop_t > 0.0 && d < reach {
                self.monsters[i].hp = 0;
                self.b.score += 200;
                self.sparkle(mx, my, (255, 120, 120), 9);
                self.b.play_sound("explode");
            }
        }
        self.monsters.retain(|m| m.hp > 0);
    }

    fn update_bullets(&mut self, dt: f64) {
        for bi in 0..self.bullets.len() {
            self.bullets[bi].y += self.bullets[bi].vy * dt;
            for mi in 0..self.monsters.len() {
                let m = self.monsters[mi];
                let b = self.bullets[bi];
                if m.hp > 0 && py::hypot(b.x - m.x, b.y - m.y) < 18.0 {
                    self.monsters[mi].hp = 0;
                    self.bullets[bi].y = -99999.0;
                    self.b.score += 200;
                    self.sparkle(m.x, m.y, (255, 120, 120), 9);
                    self.b.play_sound("explode");
                }
            }
        }
        let cam = self.cam_y;
        self.bullets.retain(|b| b.y - cam > -30.0);
        self.monsters.retain(|m| m.hp > 0);
    }

    fn camera_and_score(&mut self) {
        let h = self.b.height as f64;
        let thresh = self.cam_y + h * 0.42;
        if self.y < thresh {
            self.cam_y = self.y - h * 0.42;
        }
        let rise = self.start_y - (self.cam_y + h * 0.42);
        self.max_rise = self.max_rise.max(rise);
        self.b.score = self.b.score.max((self.max_rise / 4.0) as i64);
        if self.b.score >= 10000 {
            self.b.ach_event("doodle_high", None);
        }
    }

    fn pick_kind(&self, d: &Diff, scale: f64) -> Art {
        let r = py::random();
        let pmove = d.mv + scale * 0.12;
        let pvan = 0.06 + scale * 0.06;
        if r < pvan {
            Art::Vanish
        } else if r < pvan + pmove {
            Art::Move
        } else {
            Art::Normal
        }
    }

    fn fill_platforms(&mut self) {
        let d = &DIFFS[self.diff];
        let scale = self.difficulty_scale();
        let gap = d.gap + scale * 40.0;
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        let pw = self.plat_w;
        while self.gen_y > self.cam_y - h * 0.3 {
            let prev_y = self.gen_y;
            self.gen_y -= py::uniform(gap * 0.7, gap * 1.15);
            let x = py::uniform(0.0, w - pw as f64);
            let mut kind = self.pick_kind(d, scale);
            if kind == Art::Vanish && self.prev_kind == Some(Art::Vanish) {
                kind = Art::Normal;
            }
            self.prev_kind = Some(kind);
            let mut p = Plat::new(x, self.gen_y, pw, kind);
            if kind == Art::Move {
                let sign = py::choice(&[-1.0, 1.0]);
                p.vx = sign * py::uniform(60.0, 110.0);
            }
            if matches!(kind, Art::Normal | Art::Move) {
                let rr = py::random();
                if rr < 0.06 {
                    p.prop = true;
                } else if rr < 0.16 {
                    p.spring = true;
                }
            }
            self.platforms.push(p);
            let pbrit = d.brittle + scale * 0.10;
            if prev_y - self.gen_y > 40.0 && py::random() < pbrit {
                let bx = py::uniform(0.0, w - pw as f64);
                let by = py::uniform(self.gen_y + 14.0, prev_y - 14.0);
                self.platforms.push(Plat::new(bx, by, pw, Art::Brittle));
            }
            if py::random() < d.monster * (0.5 + scale) {
                let mx = py::uniform(40.0, w - 40.0);
                let my = self.gen_y - py::uniform(20.0, 40.0);
                let sign = py::choice(&[-1.0, 1.0]);
                let vx = sign * py::uniform(20.0, 55.0);
                let kind = py::randint(0, 2);
                let bob = py::uniform(0.0, py::TAU);
                self.monsters.push(Monster { x: mx, y: my, vx, hp: 1, kind, bob });
            }
        }
    }

    fn cull(&mut self) {
        let h = self.b.height as f64;
        let limit = self.cam_y + h + 60.0;
        let cam = self.cam_y;
        self.platforms.retain(|p| p.y < limit && !(p.dead && p.y > cam + h));
        self.monsters.retain(|m| m.y < limit);
    }

    fn die(&mut self) {
        if self.state != State::Play {
            return;
        }
        self.state = State::GameOver;
        self.b.game_over = true;
        self.new_best = self.b.score > self.highscore;
        self.highscore = self.highscore.max(self.b.score);
        self.over_t = self.anim_t;
        self.b.play_sound("gameover");
        self.b.rumble(220);
        self.sparkle(self.x, self.y, COL_DOODLE, 16);
    }

    fn sparkle(&mut self, x: f64, y: f64, col: C3, n: usize) {
        for _ in 0..n {
            let a = py::uniform(0.0, py::TAU);
            let sp = py::uniform(40.0, 160.0);
            let life = py::uniform(0.25, 0.5);
            self.particles.push(([x, y, a.cos() * sp, a.sin() * sp, life], col));
        }
    }

    fn update_particles(&mut self, dt: f64) {
        for (p, _) in self.particles.iter_mut() {
            p[0] += p[2] * dt;
            p[1] += p[3] * dt;
            p[3] += 260.0 * dt;
            p[4] -= dt;
        }
        self.particles.retain(|(p, _)| p[4] > 0.0);
    }

    fn panel(&self, s: &mut Surface, rect: Rect, alpha: u8, border: i32) {
        let p = ui::pal();
        let mut surf = pg::surface_alpha(rect.w, rect.h);
        draw::rect_r(&mut surf, pg::alpha(p.panel, alpha), (0, 0, rect.w, rect.h), 0, 12);
        s.bl(&surf, (rect.x, rect.y));
        draw::rect_r(s, self.b.accent, rect, border, 12);
    }

    fn sy(&self, wy: f64) -> i32 {
        (wy - self.cam_y) as i32
    }

    fn draw_bg(&mut self, s: &mut Surface) {
        let (w, h) = (self.b.width, self.b.height);
        if self.bg.as_ref().map(|c| c.0) != Some((w, h)) {
            let mut surf = pg::surface(w, h);
            for y in 0..h {
                let tt = y as f64 / 1.max(h) as f64;
                let m = |a: i32, b: i32| (a as f64 + (b - a) as f64 * tt) as i32;
                let c = rgb(m(COL_BG_TOP.0, COL_BG_BOT.0), m(COL_BG_TOP.1, COL_BG_BOT.1), m(COL_BG_TOP.2, COL_BG_BOT.2));
                draw::line(&mut surf, c, (0, y), (w, y), 1);
            }
            self.bg = Some(((w, h), surf));
        }
        if let Some((_, surf)) = &self.bg {
            s.bl(surf, (0, 0));
        }
        let step = 40;
        let off = py::pmod32((-self.cam_y) as i32, step);
        let mut y = -step;
        while y < h + step {
            draw::line(s, COL_GRID, (0, y + off), (w, y + off), 1);
            y += step;
        }
        let mut x = 0;
        while x < w {
            draw::line(s, COL_GRID, (x, 0), (x, h), 1);
            x += step;
        }
    }

    fn draw_plat(&self, s: &mut Surface, p: &Plat) {
        let sy = self.sy(p.y);
        let col = plat_color(p.kind);
        let rect = Rect::new(p.x as i32, sy, p.w, 14);
        draw::rect_r(s, c3(col), rect, 0, 6);
        draw::rect_r(s, scaled7(col), rect, 2, 6);
        if p.kind == Art::Brittle {
            let pw = p.w as f64;
            draw::line(s, rgb(120, 80, 50), (p.x + pw * 0.4, sy as f64), (p.x + pw * 0.5, (sy + 14) as f64), 2);
        }
        if p.spring {
            let sx = (p.x + p.w as f64 / 2.0) as i32;
            draw::rect(s, rgb(200, 200, 210), (sx - 5, sy - 10, 10, 10), 0);
            draw::line(s, rgb(120, 120, 130), (sx - 4, sy - 8), (sx + 4, sy - 4), 2);
            draw::line(s, rgb(120, 120, 130), (sx - 4, sy - 4), (sx + 4, sy - 8), 2);
        }
        if p.prop {
            let sx = (p.x + p.w as f64 / 2.0) as i32;
            draw::ellipse(s, rgb(90, 150, 230), (sx - 14, sy - 8, 28, 6), 0);
            draw::circle(s, rgb(60, 90, 160), (sx, sy - 5), 3, 0);
        }
    }

    fn draw_monster(&self, s: &mut Surface, m: &Monster) {
        let x = m.x as i32;
        let y = self.sy(m.y) + ((self.anim_t * 4.0 + m.bob).sin() * 3.0) as i32;
        let cols: [C3; 3] = [(230, 110, 130), (150, 120, 220), (235, 150, 80)];
        let col = cols[py::pmod(m.kind, 3) as usize];
        draw::circle(s, c3(col), (x, y), 16, 0);
        draw::circle(s, scaled7(col), (x, y), 16, 2);
        for sx in [-6, 6] {
            draw::circle(s, rgb(255, 255, 255), (x + sx, y - 4), 5, 0);
            draw::circle(s, rgb(30, 30, 40), (x + sx, y - 4), 2, 0);
        }
        draw::arc(s, rgb(60, 30, 40), (x - 7, y + 2, 14, 8), py::PI, py::TAU, 2);
    }

    fn draw_doodler(&self, s: &mut Surface) {
        let x = self.x as i32;
        let y = self.sy(self.y);
        let r = self.dr as f64;
        let (xf, yf) = (x as f64, y as f64);
        let aim_up = self.shoot_pose > 0.0;
        draw::ellipse(s, c3(COL_DOODLE), (xf - r, yf - r * 0.9, 2.0 * r, 1.8 * r), 0);
        draw::ellipse(s, COL_DOODLE_DARK, (xf - r, yf - r * 0.9, 2.0 * r, 1.8 * r), 2);
        draw::line(s, COL_DOODLE_DARK, (xf - r * 0.4, yf + r * 0.7), (xf - r * 0.6, yf + r * 1.2), 3);
        draw::line(s, COL_DOODLE_DARK, (xf + r * 0.4, yf + r * 0.7), (xf + r * 0.6, yf + r * 1.2), 3);
        if aim_up {
            draw::ellipse(s, rgb(90, 170, 70), (xf - 6.0, yf - r * 1.4, 12.0, 12.0), 0);
        } else {
            let nx = xf + self.face as f64 * r * 0.8;
            let left = if self.face > 0 { nx - 6.0 } else { nx - 8.0 };
            draw::ellipse(s, rgb(90, 170, 70), (left, yf - 4.0, 14.0, 12.0), 0);
        }
        for sx in [-1.0, 1.0] {
            let ex = xf + sx * r * 0.4;
            draw::circle(s, rgb(255, 255, 255), (ex as i32, (yf - r * 0.4) as i32), 5, 0);
            draw::circle(s, rgb(30, 30, 40), ((ex + self.face as f64) as i32, (yf - r * 0.4) as i32), 2, 0);
        }
        if self.prop_t > 0.0 {
            draw::rect(s, rgb(80, 110, 180), (xf - 4.0, yf - r * 1.7, 8.0, 8.0), 0);
            let w = (16.0 + 6.0 * (self.anim_t * 40.0).sin()) as i32;
            draw::ellipse(s, rgb(120, 160, 230), (xf - w as f64, yf - r * 1.8, (2 * w) as f64, 6.0), 0);
        }
        let ww = self.b.width;
        if (x as f64) < r {
            self.ghost_doodle(s, (x + ww) as f64, yf, r);
        } else if (x as f64) > ww as f64 - r {
            self.ghost_doodle(s, (x - ww) as f64, yf, r);
        }
    }

    fn ghost_doodle(&self, s: &mut Surface, x: f64, y: f64, r: f64) {
        draw::ellipse(s, c3(COL_DOODLE), ((x - r) as i32, (y - r * 0.9) as i32, (2.0 * r) as i32, (1.8 * r) as i32), 0);
    }

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let img = self.small.render(&self.b.score.to_string(), true, p.text);
        let bx = img.get_rect().inflate(24, 10).topleft_at((8, 8));
        self.panel(s, bx, 210, 1);
        s.blr(&img, img.get_rect().center_at(bx.center()));
        let best = self.tiny.render(&tf("dj.best", &[("hs", &self.highscore.to_string())]), true, p.text_dim);
        let bbox = best.get_rect().inflate(20, 10).topright_at((self.b.width - 8, 8));
        self.panel(s, bbox, 210, 1);
        s.blr(&best, best.get_rect().center_at(bbox.center()));
        if self.state == State::GameOver {
            self.draw_gameover(s);
        }
    }

    fn draw_gameover(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let mut dim = pg::surface_alpha(w, h);
        dim.fill_c(rgba(10, 12, 20, 140));
        s.bl(&dim, (0, 0));
        let (cx, cy) = (w / 2, h / 2);
        let img = self.huge.render(&t("common.game_over"), true, p.red);
        s.blr(&img, img.get_rect().center_at((cx, cy - (h as f64 * 0.16) as i32)));
        let pw = 340.min(w - 40);
        let ph = self.b.font.get_height() + self.small.get_height() + 46;
        let panel = Rect::new(cx - py::fdiv32(pw, 2), cy - py::fdiv32(ph, 2) + 10, pw, ph);
        self.panel(s, panel, 235, 2);
        let sc = self.b.font.render(&tf("common.points", &[("score", &self.b.score.to_string())]), true, p.text);
        s.blr(&sc, sc.get_rect().midtop_at((cx, panel.y + 14)));
        let best_col = if self.new_best { p.gold } else { p.text_dim };
        let bs = self.small.render(&tf("dj.best", &[("hs", &self.highscore.to_string())]), true, best_col);
        s.blr(&bs, bs.get_rect().midbottom_at((cx, panel.bottom() - 14)));
        if self.new_best {
            let rec_col = ui::mix(p.gold, p.text, ui::pulse(3.0, 0.0, 0.5));
            let rec = self.small.render(&t("trex.new_record"), true, rec_col);
            s.blr(&rec, rec.get_rect().center_at((cx, panel.y - 18)));
        }
        let hint_col = ui::mix(p.text_dim, p.text, ui::pulse(2.4, 0.0, 1.0));
        let hint = self.small.render(&t("dj.restart_hint"), true, hint_col);
        s.blr(&hint, hint.get_rect().center_at((cx, panel.bottom() + 28)));
    }

    fn draw_setup(&mut self) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let mut s = self.b.sf();
        let key = (w, h, p.bg_top, p.bg_bottom);
        if self.ui_bg.as_ref().map(|c| c.0) != Some(key) {
            let mut surf = pg::surface(w, h);
            ui::draw_background(&mut surf, w, h);
            self.ui_bg = Some((key, surf));
        }
        if let Some((_, bg)) = &self.ui_bg {
            s.bl(bg, (0, 0));
        }
        ui::draw_title_ex(&mut s, w, "DOODLE JUMP", Some(&t("snake.singleplayer")), 52, None, None, Some(self.b.accent));
        let d = &DIFFS[self.diff];
        self.panel(&mut s, self.diff_panel, 235, 2);
        let name = self.b.font.render(&format!("{}:  {}", t("dj.difficulty"), t(&format!("dj.diff.{}", d.key))), true, p.text);
        let dp = self.diff_panel;
        s.blr(&name, name.get_rect().center_at((dp.centerx(), dp.y + (dp.h as f64 * 0.36) as i32)));
        let note = self.tiny.render(&t("dj.diff_note"), true, p.text_dim);
        s.blr(&note, note.get_rect().center_at((dp.centerx(), dp.y + (dp.h as f64 * 0.72) as i32)));
        let arr_col = ui::mix(self.b.accent, p.text, ui::pulse(3.0, 0.0, 0.3));
        for (rect, sym) in [(self.diff_left, "<"), (self.diff_right, ">")] {
            let arr = self.b.big_font.render(sym, true, arr_col);
            s.blr(&arr, arr.get_rect().center_at(rect.center()));
        }
        ui::draw_button_ex(&mut s, self.start_rect, &t("common.start"), &self.b.font, true, None, None, Some(self.b.accent));
        let ctrl = self.tiny.render(&t("dj.controls_hint"), true, p.green);
        s.blr(&ctrl, ctrl.get_rect().center_at((w / 2, self.start_rect.bottom() + 24)));
        ui::draw_footer(&mut s, w, h, &t("dj.setup_hint"));
    }
}

impl Game for DoodleGame {
    /// Match-Historie: Dauer zaehlt nicht im Setup (Python-Standard
    /// `getattr(self, "state", None) != "setup"`).
    fn match_active(&self) -> bool {
        !matches!(self.state, State::Setup)
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
        self.diff = self.b.opt_i64("doodle", "difficulty", 1).clamp(0, 2) as usize;
        self.highscore = highscore::get("doodle");
        self.anim_t = 0.0;
        self.bg = None;
        self.ui_bg = None;
        self.apply_layout();
        self.new_game();
        self.state = State::Setup;
    }

    fn on_surface_changed(&mut self) {
        self.apply_layout();
        self.bg = None;
        self.ui_bg = None;
        self.x = 0f64.max((self.b.width as f64).min(self.x));
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        match self.state {
            State::Setup => {
                self.handle_setup(ev);
                return;
            }
            State::GameOver => {
                if ev.kind == Kind::KeyDown {
                    if ev.key_in(&["Return", "space"]) {
                        self.new_game();
                        self.b.play_sound("click");
                    } else if ev.key_in(&["s", "S"]) {
                        self.state = State::Setup;
                        self.b.play_sound("click");
                    }
                } else if ev.kind == Kind::MouseDown && self.anim_t - self.over_t > 0.5 {
                    self.new_game();
                    self.b.play_sound("click");
                }
                return;
            }
            State::Play => {}
        }
        if ev.kind == Kind::KeyDown {
            if self.is_left(&ev.key) {
                self.press_l = true;
            } else if self.is_right(&ev.key) {
                self.press_r = true;
            } else if self.is_shoot(&ev.key) {
                self.shoot();
            }
        } else if ev.kind == Kind::KeyUp {
            if self.is_left(&ev.key) {
                self.press_l = false;
            } else if self.is_right(&ev.key) {
                self.press_r = false;
            }
        }
    }

    fn update(&mut self, dt: f64) {
        self.anim_t += dt;
        self.update_particles(dt);
        if self.state != State::Play {
            return;
        }
        self.shoot_cd = 0f64.max(self.shoot_cd - dt);
        self.shoot_pose = 0f64.max(self.shoot_pose - dt);
        if self.press_l {
            self.vx -= MOVE_ACC * dt;
            self.face = -1;
        }
        if self.press_r {
            self.vx += MOVE_ACC * dt;
            self.face = 1;
        }
        if !self.press_l && !self.press_r {
            self.vx *= MOVE_FRICTION;
        }
        self.vx = (-MOVE_MAX).max(MOVE_MAX.min(self.vx));
        self.x += self.vx * dt;
        let w = self.b.width as f64;
        if self.x < 0.0 {
            self.x += w;
        } else if self.x > w {
            self.x -= w;
        }
        self.py_prev = self.y;
        if self.prop_t > 0.0 {
            self.prop_t -= dt;
            self.vy = PROP_V;
        } else {
            self.vy += GRAV * dt;
        }
        self.y += self.vy * dt;
        if self.vy > 0.0 {
            self.platform_collisions();
        }
        self.update_platforms(dt);
        self.update_monsters(dt);
        self.update_bullets(dt);
        self.camera_and_score();
        self.fill_platforms();
        self.cull();
        if self.y - self.cam_y > (self.b.height + self.dr) as f64 {
            self.die();
        }
    }

    fn draw(&mut self) {
        if self.state == State::Setup {
            self.draw_setup();
            return;
        }
        let mut s = self.b.sf();
        self.draw_bg(&mut s);
        for p in &self.platforms {
            if !p.dead {
                self.draw_plat(&mut s, p);
            }
        }
        for m in &self.monsters {
            self.draw_monster(&mut s, m);
        }
        for b in &self.bullets {
            let c = (b.x as i32, self.sy(b.y));
            draw::circle(&mut s, rgb(240, 250, 255), c, 5, 0);
            draw::circle(&mut s, rgb(120, 180, 230), c, 5, 1);
        }
        for (p, col) in &self.particles {
            let a = 0.max(255.min((255.0 * p[4] / 0.5) as i32));
            let mut surf = pg::surface_alpha(5, 5);
            draw::circle(&mut surf, rgba(col.0, col.1, col.2, a), (2, 2), 2, 0);
            s.bl(&surf, (p[0] - 2.0, (self.sy(p[1]) - 2) as f64));
        }
        self.draw_doodler(&mut s);
        self.draw_hud(&mut s);
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "doodle",
    class_name: "DoodleGame",
    supports_multiplayer: false,
    modes: &[],
    make: DoodleGame::new,
};
