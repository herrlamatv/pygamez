//! breakout.rs
//! ===========
//! Breakout / Brick-Breaker - Deluxe-Version.
//!
//! Uebertragen aus `games/breakout.py`: Steinsorten (Normal/Stahl/Bombe/Gold),
//! zwoelf Power-ups, Combo-Multiplikator, Partikel, Ball-Spuren, Screen-Shake,
//! 25 Level-Muster plus Aufbau "Voll".

use std::f64::consts::{PI, TAU};

use pgz_engine::prelude::*;

const STR_COLORS: [(i32, i32, i32); 3] = [(120, 205, 120), (235, 185, 80), (230, 95, 95)];

fn str_color(n: i32) -> (i32, i32, i32) {
    if (1..=3).contains(&n) {
        STR_COLORS[(n - 1) as usize]
    } else {
        (200, 200, 200)
    }
}

const BALL_COLORS: [(i32, i32, i32); 7] = [
    (255, 230, 120),
    (240, 240, 240),
    (110, 230, 230),
    (255, 120, 200),
    (120, 240, 140),
    (255, 160, 70),
    (185, 130, 255),
];

#[derive(Clone, Copy)]
struct Diff {
    lives: i32,
    paddle: i32,
    ball_speed: f64,
    drop: f64,
    bad: f64,
    hard_bonus: i32,
}

const DIFF_ORDER: [&str; 3] = ["Easy", "Medium", "Hard"];
const DIFFICULTIES: [Diff; 3] = [
    Diff { lives: 5, paddle: 140, ball_speed: 300.0, drop: 0.42, bad: 0.12, hard_bonus: 0 },
    Diff { lives: 3, paddle: 110, ball_speed: 365.0, drop: 0.32, bad: 0.28, hard_bonus: 0 },
    Diff { lives: 2, paddle: 92, ball_speed: 440.0, drop: 0.26, bad: 0.42, hard_bonus: 1 },
];

#[derive(Clone, Copy)]
struct LevelDef {
    tag: &'static str,
    pat: &'static str,
    rows: i32,
    cols: i32,
    base: i32,
    drop: f64,
    spec: f64,
}

const fn ld(tag: &'static str, pat: &'static str, rows: i32, cols: i32, base: i32, drop: f64, spec: f64) -> LevelDef {
    LevelDef { tag, pat, rows, cols, base, drop, spec }
}

const LEVEL_DEFS: [LevelDef; 25] = [
    ld("Normal", "full", 3, 11, 1, 1.0, 0.05),
    ld("Normal", "checker", 4, 11, 1, 1.0, 0.06),
    ld("Spass", "full", 6, 16, 1, 1.7, 0.10),
    ld("Normal", "pyramid", 6, 11, 1, 1.0, 0.08),
    ld("Schwer", "border", 6, 13, 2, 0.8, 0.12),
    ld("Spass", "full", 8, 20, 1, 2.0, 0.14),
    ld("Normal", "columns", 6, 11, 1, 1.0, 0.08),
    ld("Schwer", "checker", 6, 11, 2, 0.8, 0.12),
    ld("Spass", "waves", 8, 18, 1, 1.8, 0.12),
    ld("Normal", "rows", 7, 11, 1, 1.0, 0.08),
    ld("Schwer", "cross", 8, 13, 2, 0.8, 0.15),
    ld("Spass", "heart", 8, 15, 1, 2.2, 0.10),
    ld("Normal", "pyramid", 8, 13, 1, 1.0, 0.10),
    ld("Schwer", "rings", 8, 15, 2, 0.7, 0.18),
    ld("Spass", "checker", 8, 18, 1, 2.0, 0.12),
    ld("Normal", "diamond", 8, 13, 2, 1.0, 0.12),
    ld("Schwer", "border", 8, 15, 3, 0.6, 0.20),
    ld("Spass", "random", 8, 20, 1, 2.4, 0.16),
    ld("Schwer", "waves", 8, 15, 3, 0.6, 0.20),
    ld("Normal", "heart", 9, 17, 2, 1.0, 0.12),
    ld("Schwer", "cross", 9, 15, 3, 0.6, 0.22),
    ld("Spass", "full", 9, 22, 1, 2.6, 0.18),
    ld("Schwer", "rings", 9, 17, 3, 0.6, 0.24),
    ld("Schwer", "full", 9, 13, 3, 0.55, 0.26),
    ld("Schwer", "full", 9, 11, 3, 0.5, 0.30),
];
const NUM_LEVELS: usize = LEVEL_DEFS.len();

const PADDLE_H: i32 = 16;
const BALL_R: i32 = 8;
const BALL_RF: f64 = BALL_R as f64;
const BALL_MIN_SPEED: f64 = 200.0;
const BALL_MAX_SPEED: f64 = 760.0;
const PADDLE_MIN: i32 = 60;
const PADDLE_MAX: i32 = 220;
const MAX_BALLS: usize = 20;
const POWERUP_FALL: f64 = 165.0;
const LASER_SPEED: f64 = 620.0;
const LASER_INTERVAL: f64 = 0.28;

/// Effekte in Python-dict-Reihenfolge: laser, fire, sticky, shield.
const FX_KEYS: [&str; 4] = ["laser", "fire", "sticky", "shield"];
const FX_DUR: [f64; 4] = [9.0, 7.0, 11.0, 10.0];
const LASER: usize = 0;
const FIRE: usize = 1;
const STICKY: usize = 2;
const SHIELD: usize = 3;

#[derive(Clone, Copy, PartialEq)]
enum State {
    Setup,
    Play,
    Pause,
    Over,
}

#[derive(Clone)]
struct Ball {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    trail: Vec<(f64, f64)>,
    stuck: bool,
    stuck_off: f64,
}

impl Ball {
    fn new(x: f64, y: f64, vx: f64, vy: f64) -> Ball {
        Ball { x, y, vx, vy, trail: Vec::new(), stuck: false, stuck_off: 0.0 }
    }

    fn speed(&self) -> f64 {
        py::hypot(self.vx, self.vy)
    }

    fn scale_speed(&mut self, faktor: f64) {
        let mut s = self.speed();
        if s == 0.0 {
            s = 1.0;
        }
        let neu = BALL_MIN_SPEED.max(BALL_MAX_SPEED.min(s * faktor));
        self.vx *= neu / s;
        self.vy *= neu / s;
    }
}

struct PowerUp {
    kind: &'static str,
    y: f64,
    rect: Rect,
    phase: f64,
}

impl PowerUp {
    fn new(x: f64, y: f64, kind: &'static str) -> PowerUp {
        let rect = Rect::new(x as i32 - 17, y as i32 - 11, 34, 22);
        let phase = py::uniform(0.0, TAU);
        PowerUp { kind, y, rect, phase }
    }
}

/// `PowerUp.INFO`: (Beschriftung, Farbe, gut?)
fn pu_info(kind: &str) -> (&'static str, (i32, i32, i32), bool) {
    match kind {
        "multi" => ("x2", (110, 230, 230), true),
        "spread" => ("x3", (110, 170, 255), true),
        "speed" => (">>", (255, 150, 70), false),
        "slow" => ("<<", (185, 130, 255), true),
        "wide" => ("W", (120, 240, 140), true),
        "shrink" => ("S", (230, 95, 95), false),
        "life" => ("+", (255, 120, 160), true),
        "laser" => ("L", (255, 90, 90), true),
        "fire" => ("F", (255, 170, 60), true),
        "sticky" => ("G", (150, 255, 150), true),
        "shield" => ("U", (120, 200, 255), true),
        _ => ("$", (255, 220, 90), true),
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Kind2 {
    Normal,
    Steel,
    Bomb,
    Gold,
}

#[derive(Clone)]
struct Brick {
    id: u32,
    rect: Rect,
    strength: i32,
    kind: Kind2,
}

impl Brick {
    fn breakable(&self) -> bool {
        self.kind != Kind2::Steel
    }
}

struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    life: f64,
    color: (i32, i32, i32),
    size: f64,
}

struct Floater {
    x: f64,
    y: f64,
    text: String,
    color: (i32, i32, i32),
    life: f64,
}

/// `BreakoutGame`
pub struct BreakoutGame {
    b: GameBase,
    state: State,
    diff_idx: usize,
    color_index: usize,
    start_level_index: usize,
    level_full: bool,
    tiny: Font,
    small: Font,
    mid: Font,
    particles: Vec<Particle>,
    floaters: Vec<Floater>,
    lasers: Vec<[f64; 2]>,
    shake: f64,
    anim_t: f64,
    scene: Option<Surface>,
    stars: Vec<(i32, i32, f64, f64)>,
    diff_rects: Vec<Rect>,
    color_rects: Vec<Rect>,
    start_rect: Rect,
    mode_rect: Rect,
    level_up_rect: Rect,
    level_down_rect: Rect,
    cfg: Diff,
    ball_color: (i32, i32, i32),
    lives: i32,
    level_index: usize,
    paddle_w: i32,
    won: bool,
    combo: i32,
    mult: i32,
    level_def: LevelDef,
    bricks: Vec<Brick>,
    next_id: u32,
    powerups: Vec<PowerUp>,
    fx: [f64; 4],
    laser_cd: f64,
    intro_timer: f64,
    paddle_x: f64,
    paddle_y: i32,
    move_dir: i32,
    balls: Vec<Ball>,
    ball_haengt: bool,
}

impl BreakoutGame {
    /// `BreakoutGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let mut g = BreakoutGame {
            b,
            state: State::Setup,
            diff_idx: 1,
            color_index: 0,
            start_level_index: 0,
            level_full: false,
            tiny: f.clone(),
            small: f.clone(),
            mid: f,
            particles: Vec::new(),
            floaters: Vec::new(),
            lasers: Vec::new(),
            shake: 0.0,
            anim_t: 0.0,
            scene: None,
            stars: Vec::new(),
            diff_rects: Vec::new(),
            color_rects: Vec::new(),
            start_rect: z,
            mode_rect: z,
            level_up_rect: z,
            level_down_rect: z,
            cfg: DIFFICULTIES[1],
            ball_color: BALL_COLORS[0],
            lives: 0,
            level_index: 0,
            paddle_w: 110,
            won: false,
            combo: 0,
            mult: 1,
            level_def: LEVEL_DEFS[0],
            bricks: Vec::new(),
            next_id: 0,
            powerups: Vec::new(),
            fx: [0.0; 4],
            laser_cd: 0.0,
            intro_timer: 0.0,
            paddle_x: 0.0,
            paddle_y: 0,
            move_dir: 0,
            balls: Vec::new(),
            ball_haengt: true,
        };
        g.reset();
        Box::new(g)
    }

    fn make_fonts(&mut self) {
        let h = self.b.height;
        self.tiny = ui::f(11.max(h / 36));
        self.small = ui::f(13.max(h / 30));
        self.mid = ui::fb(16.max(h / 24));
    }

    fn build_bg(&mut self) {
        let (w, h) = (self.b.width as i64, self.b.height as i64);
        self.stars = (0..70)
            .map(|_| {
                let x = py::randint(0, w) as i32;
                let y = py::randint(0, h) as i32;
                let b = py::uniform(0.4, 1.0);
                let r = py::uniform(0.5, 1.6);
                (x, y, b, r)
            })
            .collect();
    }

    fn blit_bg(&self, s: &mut Surface) {
        let p = ui::pal();
        ui::draw_background(s, self.b.width, self.b.height);
        for &(x, y, b, r) in &self.stars {
            let tw = 0.6 + 0.4 * (self.anim_t * 1.5 + x as f64).sin();
            let col = ui::mix(p.bg_top, p.text, 0.2 + 0.5 * b * tw);
            draw::circle(s, col, (x, y), 1.max(r as i32), 0);
        }
    }

    fn tag_color(&self, tag: &str) -> Color {
        let p = ui::pal();
        match tag {
            "Schwer" => p.red,
            "Spass" => p.green,
            "Voll" => p.accent,
            _ => p.text_dim,
        }
    }

    // ===== Setup-Screen

    fn build_setup_layout(&mut self) {
        let cx = self.b.width / 2;
        self.diff_rects = (0..3).map(|i| Rect::new(cx - 165 + i * 112, 132, 102, 50)).collect();
        let total = BALL_COLORS.len() as i32;
        let sw = 44;
        let start = cx - py::fdiv32(total * (sw + 6), 2);
        self.color_rects = (0..total).map(|i| Rect::new(start + i * (sw + 6), 262, sw, 44)).collect();
        self.start_rect = Rect::new(cx - 95, 352, 190, 52);
        self.mode_rect = Rect::new(cx - 165, 196, 330, 34);
        self.level_up_rect = Rect::new(self.b.width - 56, 18, 38, 26);
        self.level_down_rect = Rect::new(self.b.width - 56, 50, 38, 26);
    }

    fn panel(s: &mut Surface, rect: Rect, fill: Color, border: Option<Color>) {
        draw::rect_r(s, fill, rect, 0, 8);
        draw::rect_r(s, border.unwrap_or(ui::pal().border), rect, 1, 8);
    }

    fn draw_setup(&self) {
        let p = ui::pal();
        let mut s = self.b.sf();
        let w = self.b.width;
        self.blit_bg(&mut s);
        let title = self.b.big_font.render("BREAKOUT", true, p.text);
        let glow = self.b.big_font.render("BREAKOUT", true, self.b.accent);
        s.blr(&glow, glow.get_rect().center_at((w / 2 + 2, 58)));
        s.blr(&title, title.get_rect().center_at((w / 2, 56)));
        let sub = self.small.render(&t("bo.deluxe"), true, self.b.accent);
        s.blr(&sub, sub.get_rect().center_at((w / 2, 86)));

        let ld = self.level_def_for(self.start_level_index);
        s.bl(&self.small.render(&t("bo.startlevel"), true, p.text_dim), (w - 170, 12));
        let lvl = self.mid.render(&format!("{:>2}/{}", self.start_level_index + 1, NUM_LEVELS), true, p.text);
        s.bl(&lvl, (w - 170, 36));
        let tag = self.small.render(&t(&format!("bo.tag.{}", ld.tag)), true, self.tag_color(ld.tag));
        s.bl(&tag, (w - 170, 62));
        for (r, sym) in [(self.level_up_rect, "+"), (self.level_down_rect, "-")] {
            Self::panel(&mut s, r, p.btn, None);
            let ti = self.mid.render(sym, true, p.text);
            s.blr(&ti, ti.get_rect().center_at(r.center()));
        }

        s.bl(&self.mid.render(&t("bo.difficulty"), true, p.text_dim), (w / 2 - 165, 104));
        for (i, r) in self.diff_rects.iter().enumerate() {
            let aktiv = i == self.diff_idx;
            Self::panel(&mut s, *r, if aktiv { p.btn_sel } else { p.btn }, Some(if aktiv { self.b.accent } else { p.border }));
            let ti = self.mid.render(&t(&format!("bo.diff.{}", DIFF_ORDER[i].to_lowercase())), true, p.text);
            s.blr(&ti, ti.get_rect().center_at(r.center()));
        }

        let voll = self.level_full;
        Self::panel(&mut s, self.mode_rect, if voll { p.btn_sel } else { p.btn }, None);
        let mt = self.small.render(&t(if voll { "bo.build_full" } else { "bo.build_std" }), true, p.text);
        s.blr(&mt, mt.get_rect().center_at(self.mode_rect.center()));

        s.bl(&self.mid.render(&t("bo.ballcolor"), true, p.text_dim), (w / 2 - 165, 234));
        for (i, r) in self.color_rects.iter().enumerate() {
            draw::rect_r(&mut s, BALL_COLORS[i], *r, 0, 8);
            if i == self.color_index {
                draw::rect_r(&mut s, p.text, r.inflate(8, 8), 3, 10);
            }
        }

        let col = ui::mix(p.green, (255, 255, 255), ui::pulse(4.0, 0.0, 0.22));
        draw::rect_r(&mut s, col, self.start_rect, 0, 12);
        draw::rect_r(&mut s, p.text, self.start_rect, 2, 12);
        let st = self.mid.render(&t("common.start"), true, p.text);
        s.blr(&st, st.get_rect().center_at(self.start_rect.center()));

        let hint = self.tiny.render(&t("bo.setup_hint"), true, p.text_dim);
        s.blr(&hint, hint.get_rect().center_at((w / 2, 428)));
        let leg = self.tiny.render(&t("bo.legend"), true, p.text_faint);
        s.blr(&leg, leg.get_rect().center_at((w / 2, 448)));
    }

    fn handle_setup_event(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown {
            match ev.key.as_str() {
                k @ ("1" | "2" | "3") => self.diff_idx = k.parse::<usize>().unwrap_or(2) - 1,
                "Left" | "a" => self.color_index = (self.color_index + BALL_COLORS.len() - 1) % BALL_COLORS.len(),
                "Right" | "d" => self.color_index = (self.color_index + 1) % BALL_COLORS.len(),
                "Up" | "w" => self.start_level_index = (NUM_LEVELS - 1).min(self.start_level_index + 1),
                "Down" | "s" => self.start_level_index = self.start_level_index.saturating_sub(1),
                "m" | "M" => self.toggle_mode(),
                "Return" | "space" => {
                    self.b.play_sound("select");
                    self.start_run();
                }
                _ => {}
            }
        } else if ev.kind == Kind::MouseDown {
            let pos = ev.pos;
            for i in 0..3 {
                if self.diff_rects[i].hit(pos) {
                    self.diff_idx = i;
                    self.b.play_sound("click");
                }
            }
            for i in 0..self.color_rects.len() {
                if self.color_rects[i].hit(pos) {
                    self.color_index = i;
                    self.b.play_sound("click");
                }
            }
            if self.level_up_rect.hit(pos) {
                self.start_level_index = (NUM_LEVELS - 1).min(self.start_level_index + 1);
            } else if self.level_down_rect.hit(pos) {
                self.start_level_index = self.start_level_index.saturating_sub(1);
            }
            if self.mode_rect.hit(pos) {
                self.toggle_mode();
            }
            if self.start_rect.hit(pos) {
                self.b.play_sound("select");
                self.start_run();
            }
        }
    }

    fn toggle_mode(&mut self) {
        self.level_full = !self.level_full;
        self.b.play_sound("click");
    }

    fn level_def_for(&self, index: usize) -> LevelDef {
        if self.level_full {
            let base = 3.min(1 + (index as i32 + 1) / 7);
            return ld("Voll", "full", 9, 18, base, 1.4, 0.12);
        }
        LEVEL_DEFS[index]
    }

    // ===== Spiel starten / Level laden

    fn start_run(&mut self) {
        self.cfg = DIFFICULTIES[self.diff_idx];
        self.ball_color = BALL_COLORS[self.color_index];
        self.b.score = 0;
        self.lives = self.cfg.lives;
        self.level_index = self.start_level_index;
        self.paddle_w = self.cfg.paddle;
        self.won = false;
        self.b.game_over = false;
        self.combo = 0;
        self.mult = 1;
        self.state = State::Play;
        self.load_level();
    }

    fn load_level(&mut self) {
        self.level_def = self.level_def_for(self.level_index);
        self.bricks = self.make_level(self.level_def);
        self.powerups.clear();
        self.particles.clear();
        self.lasers.clear();
        self.floaters.clear();
        self.fx = [0.0; 4];
        self.laser_cd = 0.0;
        self.intro_timer = 2.2;
        self.combo = 0;
        self.mult = 1;
        self.reset_paddle_ball();
    }

    fn reset_paddle_ball(&mut self) {
        self.paddle_x = self.b.width as f64 / 2.0 - self.paddle_w as f64 / 2.0;
        self.paddle_y = self.b.height - 36;
        self.move_dir = 0;
        let spd = self.cfg.ball_speed;
        self.balls = vec![Ball::new(self.b.width as f64 / 2.0, (self.paddle_y - BALL_R - 1) as f64, spd * 0.5, -spd)];
        self.ball_haengt = true;
        self.combo = 0;
        self.mult = 1;
    }

    fn make_level(&mut self, d: LevelDef) -> Vec<Brick> {
        let (rows, cols, pat) = (d.rows, d.cols, d.pat);
        let base = 1.max(3.min(d.base + self.cfg.hard_bonus));
        let spec = d.spec;
        let gap = 4;
        let top = 62;
        let bw = py::fdiv32(self.b.width - (cols + 1) * gap, cols);
        let bh = 22;
        let mut bricks = Vec::new();
        for r in 0..rows {
            for c in 0..cols {
                if !brick_da(pat, r, c, rows, cols) {
                    continue;
                }
                let extra = if r >= rows - 2 && base >= 2 { 1 } else { 0 };
                let mut strength = 3.min(base + extra);
                let rect = Rect::new(gap + c * (bw + gap), top + r * (bh + gap), bw, bh);
                let mut kind = Kind2::Normal;
                if py::random() < spec {
                    let roll = py::random();
                    if roll < 0.30 {
                        kind = Kind2::Steel;
                    } else if roll < 0.65 {
                        kind = Kind2::Bomb;
                    } else {
                        kind = Kind2::Gold;
                        strength = 1;
                    }
                }
                self.next_id += 1;
                bricks.push(Brick { id: self.next_id, rect, strength, kind });
            }
        }
        bricks
    }

    // ===== Eingabe

    fn launch_or_fire(&mut self) {
        let mut released = false;
        if self.ball_haengt {
            self.ball_haengt = false;
            released = true;
        }
        for b in self.balls.iter_mut() {
            if b.stuck {
                b.stuck = false;
                released = true;
            }
        }
        if !released && self.fx[LASER] > 0.0 {
            self.fire_laser();
        }
    }

    // ===== Logik

    fn paddle_rect(&self) -> Rect {
        Rect::new(self.paddle_x as i32, self.paddle_y, self.paddle_w, PADDLE_H)
    }

    fn update_balls(&mut self, dt: f64, paddle_rect: Rect) {
        let fire = self.fx[FIRE] > 0.0;
        let shield = self.fx[SHIELD] > 0.0;
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        let balls = std::mem::take(&mut self.balls);
        let mut ueberlebende = Vec::new();
        for mut b in balls {
            if b.stuck {
                ueberlebende.push(b);
                continue;
            }
            b.trail.push((b.x, b.y));
            if b.trail.len() > 7 {
                b.trail.remove(0);
            }
            b.x += b.vx * dt;
            b.y += b.vy * dt;
            if b.x - BALL_RF <= 0.0 {
                b.x = BALL_RF;
                b.vx = b.vx.abs();
            } else if b.x + BALL_RF >= w {
                b.x = w - BALL_RF;
                b.vx = -b.vx.abs();
            }
            if b.y - BALL_RF <= 0.0 {
                b.y = BALL_RF;
                b.vy = b.vy.abs();
            }
            if shield && b.vy > 0.0 && b.y + BALL_RF >= h - 6.0 {
                b.y = h - 6.0 - BALL_RF;
                b.vy = -b.vy.abs();
                self.b.play_sound("bounce");
                self.spawn_particles(b.x, h - 6.0, (120, 200, 255), 6, 180.0);
            }
            if b.y - BALL_RF > h {
                continue;
            }
            let ball_rect = Rect::new((b.x - BALL_RF) as i32, (b.y - BALL_RF) as i32, BALL_R * 2, BALL_R * 2);
            if ball_rect.colliderect(paddle_rect) && b.vy > 0.0 {
                let half = self.paddle_w as f64 / 2.0;
                let mut treff = (b.x - (self.paddle_x + half)) / half;
                treff = (-1f64).max(1f64.min(treff));
                let speed = b.speed();
                let winkel = treff * (PI / 3.0);
                b.vx = speed * winkel.sin();
                b.vy = -(speed * winkel.cos()).abs();
                b.y = (self.paddle_y - BALL_R - 1) as f64;
                self.b.play_sound("bounce");
                self.combo = 0;
                self.mult = 1;
                if self.fx[STICKY] > 0.0 {
                    b.stuck = true;
                    b.stuck_off = treff * (half - BALL_RF);
                }
            }
            self.ball_bricks(&mut b, ball_rect, fire);
            ueberlebende.push(b);
        }
        // Waehrend der Schleife kommen keine Baelle hinzu (Power-ups erst danach).
        self.balls = ueberlebende;
    }

    fn has_brick(&self, id: u32) -> bool {
        self.bricks.iter().any(|b| b.id == id)
    }

    fn ball_bricks(&mut self, b: &mut Ball, ball_rect: Rect, fire: bool) {
        let kopie: Vec<(u32, Rect, Kind2)> = self.bricks.iter().map(|br| (br.id, br.rect, br.kind)).collect();
        for (id, rect, kind) in kopie {
            if !self.has_brick(id) {
                continue;
            }
            if !ball_rect.colliderect(rect) {
                continue;
            }
            if kind == Kind2::Steel {
                bounce(b, ball_rect, rect);
                self.b.play_sound("bounce");
                return;
            }
            if fire {
                self.hit_brick(id, true);
                continue;
            }
            bounce(b, ball_rect, rect);
            self.hit_brick(id, false);
            return;
        }
    }

    fn hit_brick(&mut self, id: u32, full: bool) {
        let Some(i) = self.bricks.iter().position(|b| b.id == id) else {
            return;
        };
        let dmg = if full { self.bricks[i].strength } else { 1 };
        self.bricks[i].strength -= dmg;
        if self.bricks[i].strength > 0 {
            self.b.score += 5;
            self.b.play_sound("eat");
            return;
        }
        let brick = self.bricks.remove(i);
        self.pop_brick(&brick, true);
    }

    fn pop_brick(&mut self, brick: &Brick, chain: bool) {
        self.combo += 1;
        self.mult = 8.min(1 + self.combo / 4);
        let (base, col) = match brick.kind {
            Kind2::Gold => (60, (255, 215, 90)),
            Kind2::Bomb => (25, (255, 130, 70)),
            _ => (10, str_color(1.max(brick.strength + 1))),
        };
        let pts = base * self.mult;
        self.b.score += pts as i64;
        let (cx, cy) = (brick.rect.centerx(), brick.rect.centery());
        self.add_floater(cx as f64, cy as f64, format!("+{pts}"), col);
        self.spawn_particles(cx as f64, cy as f64, col, 10, 180.0);
        self.b.play_sound("explode");
        self.maybe_drop((cx, cy));
        if brick.kind == Kind2::Bomb && chain {
            self.b.play_sound("hit");
            self.shake = self.shake.max(12.0);
            self.explode((cx, cy), 70.0);
        }
    }

    fn explode(&mut self, center: (i32, i32), radius: f64) {
        let (cx, cy) = center;
        self.spawn_particles(cx as f64, cy as f64, (255, 150, 60), 22, 260.0);
        let opfer: Vec<u32> = self
            .bricks
            .iter()
            .filter(|b| b.breakable() && py::hypot((b.rect.centerx() - cx) as f64, (b.rect.centery() - cy) as f64) <= radius)
            .map(|b| b.id)
            .collect();
        for id in opfer {
            if let Some(i) = self.bricks.iter().position(|b| b.id == id) {
                let b = self.bricks.remove(i);
                let chain = b.kind == Kind2::Bomb;
                self.pop_brick(&b, chain);
            }
        }
    }

    fn fire_laser(&mut self) {
        let y = (self.paddle_y - 4) as f64;
        self.lasers.push([self.paddle_x + 8.0, y]);
        self.lasers.push([self.paddle_x + self.paddle_w as f64 - 8.0, y]);
        self.b.play_sound("shoot");
    }

    fn update_lasers(&mut self, dt: f64) {
        let shots = std::mem::take(&mut self.lasers);
        let mut bleibt = Vec::new();
        for mut shot in shots {
            shot[1] -= LASER_SPEED * dt;
            if shot[1] < 0.0 {
                continue;
            }
            let r = Rect::new(shot[0] as i32 - 2, shot[1] as i32 - 10, 4, 12);
            let mut getroffen = false;
            let treffer = self.bricks.iter().find(|br| r.colliderect(br.rect)).map(|br| (br.id, br.kind));
            if let Some((id, kind)) = treffer {
                if kind != Kind2::Steel {
                    self.hit_brick(id, false);
                }
                getroffen = true;
            }
            if !getroffen {
                bleibt.push(shot);
            }
        }
        // hit_brick legt keine neuen Schuesse an.
        self.lasers = bleibt;
    }

    fn maybe_drop(&mut self, pos: (i32, i32)) {
        let chance = 0.95f64.min(self.cfg.drop * self.level_def.drop);
        if py::random() > chance {
            return;
        }
        const GUT: [&str; 10] = ["multi", "spread", "wide", "life", "slow", "laser", "fire", "sticky", "shield", "coin"];
        const SCHLECHT: [&str; 2] = ["shrink", "speed"];
        let kind = if py::random() < self.cfg.bad { py::choice(&SCHLECHT) } else { py::choice(&GUT) };
        self.powerups.push(PowerUp::new(pos.0 as f64, pos.1 as f64, kind));
    }

    fn update_powerups(&mut self, dt: f64, paddle_rect: Rect) {
        let ups = std::mem::take(&mut self.powerups);
        let mut bleibt = Vec::new();
        for mut p in ups {
            p.y += POWERUP_FALL * dt;
            p.rect.y = p.y as i32 - p.rect.h / 2;
            if p.rect.colliderect(paddle_rect) {
                self.activate(p.kind);
            } else if p.rect.y <= self.b.height {
                bleibt.push(p);
            }
        }
        self.powerups = bleibt;
    }

    fn activate(&mut self, kind: &str) {
        let (label, col, gut) = pu_info(kind);
        self.add_floater(self.paddle_x + self.paddle_w as f64 / 2.0, (self.paddle_y - 18) as f64, label.to_string(), col);
        self.b.play_sound(if gut { "point" } else { "hit" });
        match kind {
            "life" => self.lives += 1,
            "wide" => self.paddle_w = PADDLE_MAX.min(self.paddle_w + 30),
            "shrink" => self.paddle_w = PADDLE_MIN.max(self.paddle_w - 26),
            "speed" => self.balls.iter_mut().for_each(|b| b.scale_speed(1.25)),
            "slow" => self.balls.iter_mut().for_each(|b| b.scale_speed(0.80)),
            "coin" => self.b.score += 250,
            "laser" | "fire" | "sticky" | "shield" => {
                let i = FX_KEYS.iter().position(|k| *k == kind).unwrap_or(0);
                self.fx[i] = FX_DUR[i];
                if kind == "laser" {
                    self.laser_cd = 0.0;
                }
            }
            "multi" => {
                let mut neu = Vec::new();
                for b in &self.balls {
                    if b.stuck {
                        continue;
                    }
                    if self.balls.len() + neu.len() < MAX_BALLS {
                        neu.push(Ball::new(b.x, b.y, -b.vx, b.vy));
                    }
                }
                self.balls.extend(neu);
            }
            "spread" => {
                let quelle = self.balls.iter().find(|b| !b.stuck).or(self.balls.first()).map(|b| (b.x, b.y, b.speed()));
                if let Some((qx, qy, sp)) = quelle {
                    let spd = if sp != 0.0 { sp } else { self.cfg.ball_speed };
                    for da in [-0.5f64, 0.5] {
                        if self.balls.len() < MAX_BALLS {
                            self.balls.push(Ball::new(qx, qy, spd * da.sin(), -(spd * da.cos()).abs()));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // ----- Partikel / Popups

    fn spawn_particles(&mut self, x: f64, y: f64, color: (i32, i32, i32), n: usize, spread: f64) {
        for _ in 0..n {
            let ang = py::uniform(0.0, TAU);
            let spd = py::uniform(40.0, spread);
            let life = py::uniform(0.3, 0.7);
            let size = py::uniform(2.0, 4.0);
            self.particles.push(Particle { x, y, vx: ang.cos() * spd, vy: ang.sin() * spd, life, color, size });
        }
    }

    fn update_particles(&mut self, dt: f64) {
        for p in self.particles.iter_mut() {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.vy += 260.0 * dt;
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);
    }

    fn add_floater(&mut self, x: f64, y: f64, text: String, color: (i32, i32, i32)) {
        self.floaters.push(Floater { x, y, text, color, life: 0.9 });
    }

    fn update_floaters(&mut self, dt: f64) {
        for f in self.floaters.iter_mut() {
            f.y -= 34.0 * dt;
            f.life -= dt;
        }
        self.floaters.retain(|f| f.life > 0.0);
    }

    fn ende(&mut self, gewonnen: bool) {
        self.won = gewonnen;
        self.state = State::Over;
        self.b.game_over = true;
        if gewonnen {
            self.b.report_result(true);
            self.b.ach_event("breakout_clear", None);
        }
        self.b.play_sound(if gewonnen { "win" } else { "gameover" });
        self.b.rumble(220);
    }

    // ===== Zeichnen

    fn draw_paddle(&self, s: &mut Surface) {
        let p = ui::pal();
        let rect = Rect::new(self.paddle_x as i32, self.paddle_y, self.paddle_w, PADDLE_H);
        let base = if self.fx[STICKY] > 0.0 { rgb(150, 240, 160) } else { p.text };
        draw::rect_r(s, base, rect, 0, 8);
        draw::rect_r(s, self.b.accent, rect, 2, 8);
        if self.fx[LASER] > 0.0 {
            for gx in [rect.x + 8, rect.right() - 8] {
                draw::rect_r(s, rgb(255, 90, 90), Rect::new(gx - 3, rect.y - 8, 6, 8), 0, 2);
            }
        }
    }

    fn draw_brick(&self, s: &mut Surface, brick: &Brick) {
        let rect = brick.rect;
        match brick.kind {
            Kind2::Steel => {
                draw::rect_r(s, rgb(120, 128, 140), rect, 0, 4);
                draw::rect_r(s, rgb(170, 178, 190), rect, 2, 4);
                for (bx, by) in [
                    (rect.x + 5, rect.y + 5),
                    (rect.right() - 5, rect.y + 5),
                    (rect.x + 5, rect.bottom() - 5),
                    (rect.right() - 5, rect.bottom() - 5),
                ] {
                    draw::circle(s, rgb(80, 86, 96), (bx, by), 2, 0);
                }
            }
            Kind2::Gold => {
                draw::rect_r(s, rgb(245, 200, 70), rect, 0, 4);
                draw::rect_r(s, rgb(255, 240, 160), rect, 2, 4);
                let ti = self.small.render("$", true, rgb(120, 80, 10));
                s.blr(&ti, ti.get_rect().center_at(rect.center()));
            }
            Kind2::Bomb => {
                draw::rect_r(s, rgb(70, 40, 40), rect, 0, 4);
                draw::rect_r(s, rgb(230, 90, 60), rect, 2, 4);
                draw::circle(s, rgb(230, 90, 60), rect.center(), 5, 0);
            }
            Kind2::Normal => {
                draw::rect_r(s, str_color(brick.strength), rect, 0, 4);
                let mut hi = pg::surface_alpha(rect.w - 4, 4);
                hi.fill_c(rgba(255, 255, 255, 55));
                s.bl(&hi, (rect.x + 2, rect.y + 2));
                if brick.strength > 1 {
                    let n = self.small.render(&brick.strength.to_string(), true, rgb(20, 20, 20));
                    s.blr(&n, n.get_rect().center_at(rect.center()));
                }
            }
        }
    }

    fn draw_powerup(&self, s: &mut Surface, p: &PowerUp) {
        let (label, farbe, gut) = pu_info(p.kind);
        let bob = (2.0 * (self.anim_t * 6.0 + p.phase).sin()) as i32;
        let r = p.rect.move_(0, bob);
        let mut glow = pg::surface_alpha(r.w + 8, r.h + 8);
        let gr = glow.get_rect();
        draw::rect_r(&mut glow, rgba(farbe.0, farbe.1, farbe.2, 70), gr, 0, 8);
        s.bl(&glow, (r.x - 4, r.y - 4));
        draw::rect_r(s, farbe, r, 0, 6);
        draw::rect_r(s, if gut { rgb(20, 20, 30) } else { rgb(30, 10, 10) }, r, 2, 6);
        let ti = self.small.render(label, true, rgb(20, 20, 20));
        s.blr(&ti, ti.get_rect().center_at(r.center()));
    }

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        let mut bar = pg::surface_alpha(w, 34);
        bar.fill_c(rgba(p.panel.r as i32, p.panel.g as i32, p.panel.b as i32, 150));
        s.bl(&bar, (0, 0));
        draw::line(s, p.border, (0, 33), (w, 33), 1);
        s.bl(&self.b.font.render(&tf("common.points", &[("score", &self.b.score.to_string())]), true, p.text), (10, 6));
        let tag = self.level_def.tag;
        let mid = self.small.render(
            &tf(
                "bo.hud",
                &[
                    ("diff", &t(&format!("bo.diff.{}", DIFF_ORDER[self.diff_idx].to_lowercase()))),
                    ("level", &(self.level_index + 1).to_string()),
                    ("total", &NUM_LEVELS.to_string()),
                    ("tag", &t(&format!("bo.tag.{tag}"))),
                ],
            ),
            true,
            self.tag_color(tag),
        );
        s.blr(&mid, mid.get_rect().midtop_at((w / 2, 8)));
        for i in 0..self.lives.min(8) {
            let hx = w - 20 - i * 20;
            draw::circle(s, rgb(230, 80, 110), (hx - 3, 14), 4, 0);
            draw::circle(s, rgb(230, 80, 110), (hx + 3, 14), 4, 0);
            draw::polygon(s, rgb(230, 80, 110), &[(hx - 6, 15), (hx + 6, 15), (hx, 23)], 0);
        }
        if self.lives > 8 {
            s.bl(&self.tiny.render(&format!("x{}", self.lives), true, p.text), (w - 20 - 8 * 20 - 24, 8));
        }
        if self.combo > 1 {
            let ct = self.mid.render(&tf("bo.combo", &[("mult", &self.mult.to_string())]), true, p.gold);
            s.blr(&ct, ct.get_rect().midtop_at((w / 2, 38)));
        }
        let y = self.b.height - 22;
        let order = [(LASER, "L", (255, 90, 90)), (FIRE, "F", (255, 170, 60)), (STICKY, "G", (150, 255, 150)), (SHIELD, "U", (120, 200, 255))];
        let mut x = 8;
        for (k, lab, col) in order {
            if self.fx[k] <= 0.0 {
                continue;
            }
            let frac = self.fx[k] / FX_DUR[k];
            draw::rect_r(s, p.panel, Rect::new(x, y, 54, 14), 0, 4);
            draw::rect_r(s, col, Rect::new(x, y, (54.0 * frac) as i32, 14), 0, 4);
            s.bl(&self.tiny.render(lab, true, rgb(10, 10, 10)), (x + 4, y));
            x += 60;
        }
    }

    fn draw_intro(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let a = if self.intro_timer < 0.4 { 1f64.min(self.intro_timer / 0.4) } else { 1.0 };
        let alpha = (220.0 * 1f64.min(a)) as i32;
        let mut band = pg::surface_alpha(w, 90);
        band.fill_c(rgba(p.panel.r as i32, p.panel.g as i32, p.panel.b as i32, (alpha as f64 * 0.7) as i32));
        let ac = self.b.accent;
        band.fill_r(rgba(ac.r as i32, ac.g as i32, ac.b as i32, alpha), Rect::new(0, 0, w, 2));
        band.fill_r(rgba(ac.r as i32, ac.g as i32, ac.b as i32, alpha), Rect::new(0, 88, w, 2));
        s.bl(&band, (0, h / 2 - 45));
        let tag = self.level_def.tag;
        let mut big = self.b.big_font.render(&tf("bo.level_intro", &[("level", &(self.level_index + 1).to_string())]), true, p.text);
        big.set_alpha(Some(alpha as u8));
        s.blr(&big, big.get_rect().center_at((w / 2, h / 2 - 10)));
        let mut sub = self.mid.render(&format!("[{}]", t(&format!("bo.tag.{tag}"))), true, self.tag_color(tag));
        sub.set_alpha(Some(alpha as u8));
        s.blr(&sub, sub.get_rect().center_at((w / 2, h / 2 + 26)));
    }

    fn draw_overlay(&self) {
        let p = ui::pal();
        let mut s = self.b.sf();
        let mut ov = pg::surface_alpha(self.b.width, self.b.height);
        ov.fill_c(rgba(0, 0, 0, 170));
        s.bl(&ov, (0, 0));
        if self.state == State::Pause {
            self.b.draw_center_text(&t("app.pause"), &self.b.big_font, p.text, -10);
            self.b.draw_center_text(&t("bo.pause_resume"), &self.b.font, p.text_dim, 40);
        } else {
            if self.won {
                self.b.draw_center_text(&t("bo.cleared"), &self.b.big_font, p.green, -30);
            } else {
                self.b.draw_center_text(&t("common.game_over"), &self.b.big_font, p.red, -30);
            }
            self.b.draw_center_text(&tf("common.points", &[("score", &self.b.score.to_string())]), &self.b.font, p.text, 18);
            self.b.draw_center_text(&t("bo.back_to_select"), &self.b.font, p.text_dim, 52);
        }
    }

    fn draw_scene(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        self.blit_bg(s);
        if matches!(self.state, State::Play | State::Pause) && self.fx[SHIELD] > 0.0 {
            let mut netz = pg::surface_alpha(w, 6);
            let a = 120 + (80.0 * (self.anim_t * 6.0).sin()) as i32;
            netz.fill_c(rgba(110, 190, 255, 60.max(a)));
            s.bl(&netz, (0, h - 6));
        }
        for brick in &self.bricks {
            self.draw_brick(s, brick);
        }
        for pt in &self.particles {
            let a = 0.max(255.min((255.0 * (pt.life / 0.7)) as i32));
            let d = (pt.size * 2.0) as i32;
            let mut surf = pg::surface_alpha(d, d);
            let r = pt.size as i32;
            draw::circle(&mut surf, rgba(pt.color.0, pt.color.1, pt.color.2, a), (r, r), r, 0);
            s.bl(&surf, (pt.x - pt.size, pt.y - pt.size));
        }
        let bc = self.ball_color;
        for b in &self.balls {
            let n = b.trail.len();
            for (j, &(tx, ty)) in b.trail.iter().enumerate() {
                let a = (90 * (j + 1) / n) as i32;
                let mut surf = pg::surface_alpha(BALL_R * 2, BALL_R * 2);
                let rad = 2.max((BALL_RF * (j + 1) as f64 / n as f64) as i32);
                draw::circle(&mut surf, rgba(bc.0, bc.1, bc.2, a), (BALL_R, BALL_R), rad, 0);
                s.bl(&surf, (tx - BALL_RF, ty - BALL_RF));
            }
        }
        for shot in &self.lasers {
            draw::rect_r(s, rgb(255, 90, 90), Rect::new(shot[0] as i32 - 2, shot[1] as i32 - 10, 4, 12), 0, 2);
        }
        self.draw_paddle(s);
        let fire = self.fx[FIRE] > 0.0;
        for b in &self.balls {
            let col = if fire { (255, 150, 40) } else { bc };
            if fire {
                let mut glow = pg::surface_alpha(BALL_R * 4, BALL_R * 4);
                draw::circle(&mut glow, rgba(255, 120, 30, 90), (BALL_R * 2, BALL_R * 2), BALL_R * 2, 0);
                s.bl(&glow, (b.x - BALL_RF * 2.0, b.y - BALL_RF * 2.0));
            }
            draw::circle(s, col, (b.x as i32, b.y as i32), BALL_R, 0);
            draw::circle(s, rgb(255, 255, 255), ((b.x - 2.0) as i32, (b.y - 2.0) as i32), 2, 0);
        }
        for pu in &self.powerups {
            self.draw_powerup(s, pu);
        }
        for f in &self.floaters {
            let a = 0.max(255.min((255.0 * (f.life / 0.9)) as i32));
            let mut img = self.small.render(&f.text, true, f.color);
            img.set_alpha(Some(a as u8));
            s.blr(&img, img.get_rect().center_at((f.x as i32, f.y as i32)));
        }
        self.draw_hud(s);
        if self.ball_haengt && self.intro_timer <= 0.0 {
            let img = self.b.font.render(&t("bo.start_ball"), true, p.text);
            s.blr(&img, img.get_rect().center_at((w / 2, h / 2 + 60)));
        }
        if self.intro_timer > 0.0 {
            self.draw_intro(s);
        }
    }
}

fn bounce(b: &mut Ball, ball_rect: Rect, rect: Rect) {
    let ux = (ball_rect.right() - rect.x).min(rect.right() - ball_rect.x);
    let uy = (ball_rect.bottom() - rect.y).min(rect.bottom() - ball_rect.y);
    if ux < uy {
        b.vx = -b.vx;
    } else {
        b.vy = -b.vy;
    }
}

/// `_brick_da` - steht an (r, c) ein Stein?
fn brick_da(pat: &str, r: i32, c: i32, rows: i32, cols: i32) -> bool {
    let mitte = cols / 2;
    match pat {
        "full" => true,
        "checker" => (r + c) % 2 == 0,
        "pyramid" => (c - mitte).abs() <= r,
        "columns" => c % 2 == 0,
        "rows" => r % 2 == 0,
        "diamond" => (c - mitte).abs() + (r - rows / 2).abs() <= mitte.max(rows / 2) - 1,
        "border" => r == 0 || r == rows - 1 || c == 0 || c == cols - 1,
        "cross" => (c - mitte).abs() <= 1 || (r - rows / 2).abs() <= 1,
        "columns3" => c % 3 != 1,
        "waves" => {
            let band = (rows - 1) as f64 / 2.0 + (rows as f64 / 3.0) * (c as f64 * 0.9).sin();
            (r as f64 - band).abs() < 1.2
        }
        "rings" => ((c - mitte).abs() + (r - rows / 2).abs()) % 3 != 1,
        "random" => py::random() < 0.72,
        "heart" => {
            let x = (c as f64 - (cols - 1) as f64 / 2.0) / (cols as f64 / 2.6);
            let y = ((rows - 1) as f64 / 2.0 - r as f64) / (rows as f64 / 2.4) + 0.35;
            // ** ruft in CPython pow() der C-Bibliothek - powf tut dasselbe, powi nicht
            let v = py::pow(x * x + y * y - 1.0, 3.0) - x * x * py::pow(y, 3.0);
            v <= 0.0
        }
        _ => true,
    }
}

impl Game for BreakoutGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        self.b.score = 0;
        self.b.game_over = false;
        self.state = State::Setup;
        self.diff_idx = 1;
        self.color_index = 0;
        self.start_level_index = 0;
        self.level_full = false;
        self.make_fonts();
        self.particles.clear();
        self.floaters.clear();
        self.lasers.clear();
        self.shake = 0.0;
        self.anim_t = 0.0;
        self.scene = None;
        self.stars.clear();
        self.build_bg();
        self.build_setup_layout();
    }

    fn on_surface_changed(&mut self) {
        self.scene = None;
        self.make_fonts();
        self.build_bg();
        self.build_setup_layout();
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        match self.state {
            State::Setup => self.handle_setup_event(ev),
            State::Over => {
                if (ev.kind == Kind::KeyDown && ev.key_in(&["Return", "space"])) || ev.kind == Kind::MouseDown {
                    self.state = State::Setup;
                    self.b.game_over = false;
                }
            }
            State::Pause => {
                if ev.kind == Kind::KeyDown && ev.key_in(&["p", "P", "Escape", "space"]) {
                    self.state = State::Play;
                }
            }
            State::Play => {
                if ev.kind == Kind::KeyDown {
                    match ev.key.as_str() {
                        "Left" | "a" => self.move_dir = -1,
                        "Right" | "d" => self.move_dir = 1,
                        "p" | "P" | "Escape" => {
                            self.state = State::Pause;
                            self.b.play_sound("click");
                        }
                        "space" => self.launch_or_fire(),
                        _ => {}
                    }
                } else if ev.kind == Kind::MouseMove {
                    self.paddle_x = ev.pos.0 as f64 - self.paddle_w as f64 / 2.0;
                    self.move_dir = 0;
                } else if ev.kind == Kind::MouseDown {
                    self.launch_or_fire();
                }
            }
        }
    }

    fn update(&mut self, dt: f64) {
        self.anim_t += dt;
        if self.state != State::Play {
            self.update_particles(dt);
            return;
        }
        if self.intro_timer > 0.0 {
            self.intro_timer = 0f64.max(self.intro_timer - dt);
        }
        for v in self.fx.iter_mut() {
            if *v > 0.0 {
                *v = 0f64.max(*v - dt);
            }
        }
        if self.move_dir != 0 {
            self.paddle_x += self.move_dir as f64 * 560.0 * dt;
        }
        self.paddle_x = 0f64.max(((self.b.width - self.paddle_w) as f64).min(self.paddle_x));
        let paddle_rect = self.paddle_rect();
        let mitte = self.paddle_x + self.paddle_w as f64 / 2.0;
        let py_ = (self.paddle_y - BALL_R - 1) as f64;
        let haengt = self.ball_haengt;
        for b in self.balls.iter_mut() {
            if haengt || b.stuck {
                let off = if haengt { 0.0 } else { b.stuck_off };
                b.x = mitte + off;
                b.y = py_;
            }
        }
        if !self.ball_haengt {
            self.update_balls(dt, paddle_rect);
        }
        if self.fx[LASER] > 0.0 {
            self.laser_cd -= dt;
            if self.laser_cd <= 0.0 {
                self.fire_laser();
                self.laser_cd = LASER_INTERVAL;
            }
        }
        self.update_lasers(dt);
        self.update_powerups(dt, paddle_rect);
        self.update_particles(dt);
        self.update_floaters(dt);
        if self.shake > 0.0 {
            self.shake = 0f64.max(self.shake - 55.0 * dt);
        }
        if self.balls.is_empty() {
            self.lives -= 1;
            self.b.play_sound("hit");
            self.shake = 10.0;
            if self.lives <= 0 {
                self.ende(false);
            } else {
                self.reset_paddle_ball();
            }
            return;
        }
        if !self.bricks.iter().any(|b| b.breakable()) {
            if self.level_index + 1 < NUM_LEVELS {
                self.level_index += 1;
                self.b.play_sound("point");
                self.load_level();
            } else {
                self.ende(true);
            }
        }
    }

    fn draw(&mut self) {
        if self.state == State::Setup {
            self.draw_setup();
            return;
        }
        let (w, h) = (self.b.width, self.b.height);
        let mut scene = match self.scene.take() {
            Some(sc) if sc.w() == w && sc.h() == h => sc,
            _ => pg::surface(w, h),
        };
        self.draw_scene(&mut scene);
        let mut s = self.b.sf();
        if self.shake > 0.5 {
            let dx = py::uniform(-self.shake, self.shake);
            let dy = py::uniform(-self.shake, self.shake);
            s.fill_c(ui::pal().bg_bottom);
            s.bl(&scene, (dx, dy));
        } else {
            s.bl(&scene, (0, 0));
        }
        self.scene = Some(scene);
        if matches!(self.state, State::Pause | State::Over) {
            self.draw_overlay();
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "breakout",
    class_name: "BreakoutGame",
    supports_multiplayer: false,
    modes: &[],
    make: BreakoutGame::new,
};
