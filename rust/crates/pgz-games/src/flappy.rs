//! flappy.rs
//! =========
//! Flappy Bird - Klon mit Extras.
//!
//! Uebertragen aus `games/flappy.py`: Schwerkraft-Physik mit Neigung,
//! Roehrenpaare, Muenzen und Schild, Tag/Nacht-Themen, drei Stufen, Medaillen,
//! Crash mit Kamera-Wackeln. Zufall auch beim Zeichnen (Wackeln) - in
//! derselben Reihenfolge wie Python.

use pgz_core::highscore;
use pgz_engine::prelude::*;

const GRAV: f64 = 1550.0;
const FLAP_V: f64 = -430.0;
const BIRD_X_FRAC: f64 = 0.30;
const PIPE_W_FRAC: f64 = 0.13;
const CAP_H_FRAC: f64 = 0.028;

struct Diff {
    key: &'static str,
    gap: f64,
    speed: f64,
    spacing: f64,
}

const DIFFS: [Diff; 3] = [
    Diff { key: "easy", gap: 0.30, speed: 150.0, spacing: 0.62 },
    Diff { key: "normal", gap: 0.25, speed: 190.0, spacing: 0.55 },
    Diff { key: "hard", gap: 0.21, speed: 230.0, spacing: 0.48 },
];

type C3 = (i32, i32, i32);
const THEMES: [[C3; 6]; 4] = [
    [(120, 200, 245), (200, 235, 250), (110, 205, 90), (70, 150, 60), (222, 200, 120), (180, 155, 90)],
    [(250, 190, 120), (255, 225, 170), (230, 160, 70), (180, 120, 50), (210, 180, 120), (170, 140, 90)],
    [(40, 45, 90), (80, 90, 150), (90, 120, 200), (60, 80, 150), (60, 60, 100), (40, 40, 75)],
    [(150, 120, 200), (220, 190, 235), (200, 120, 190), (150, 80, 140), (200, 170, 200), (160, 130, 165)],
];

const MEDALS: [(i64, &str, C3); 4] = [
    (70, "platin", (225, 235, 245)),
    (45, "gold", (245, 205, 70)),
    (25, "silber", (200, 205, 215)),
    (10, "bronze", (205, 140, 85)),
];

const COL_COIN: C3 = (250, 205, 70);
const COL_SHIELD: C3 = (120, 220, 255);

fn c3(c: C3) -> Color {
    rgb(c.0, c.1, c.2)
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Setup,
    Ready,
    Play,
    Dying,
    GameOver,
}

#[derive(Clone, Copy)]
struct Cloud {
    x: f64,
    y: f64,
    s: f64,
    v: f64,
}

#[derive(Clone, Copy)]
struct Pipe {
    x: f64,
    gy: f64,
    gap: f64,
    passed: bool,
}

#[derive(Clone, Copy)]
struct Pickup {
    x: f64,
    y: f64,
    coin: bool,
    taken: bool,
}

/// `FlappyGame`
pub struct FlappyGame {
    b: GameBase,
    diff: usize,
    highscore: i64,
    anim_t: f64,
    small: Font,
    tiny: Font,
    huge: Font,
    ground_h: i32,
    bird_x: f64,
    bird_r: i32,
    pipe_w: i32,
    diff_panel: Rect,
    diff_left: Rect,
    diff_right: Rect,
    start_rect: Rect,
    clouds: Vec<Cloud>,
    coins: i64,
    bird_y: f64,
    bird_v: f64,
    bird_a: f64,
    pipes: Vec<Pipe>,
    pickups: Vec<Pickup>,
    particles: Vec<([f64; 5], C3)>,
    shield: bool,
    shake: f64,
    ground_x: f64,
    dist: f64,
    wing_t: f64,
    new_best: bool,
    over_t: f64,
    state: State,
    ui_bg: Option<((i32, i32, Color, Color), Surface)>,
    sky: Option<((i32, i32, C3, C3), Surface)>,
}

impl FlappyGame {
    /// `FlappyGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let mut g = FlappyGame {
            b,
            diff: 1,
            highscore: 0,
            anim_t: 0.0,
            small: f.clone(),
            tiny: f.clone(),
            huge: f,
            ground_h: 0,
            bird_x: 0.0,
            bird_r: 9,
            pipe_w: 0,
            diff_panel: z,
            diff_left: z,
            diff_right: z,
            start_rect: z,
            clouds: Vec::new(),
            coins: 0,
            bird_y: 0.0,
            bird_v: 0.0,
            bird_a: 0.0,
            pipes: Vec::new(),
            pickups: Vec::new(),
            particles: Vec::new(),
            shield: false,
            shake: 0.0,
            ground_x: 0.0,
            dist: 0.0,
            wing_t: 0.0,
            new_best: false,
            over_t: 0.0,
            state: State::Setup,
            ui_bg: None,
            sky: None,
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
        self.huge = ui::fb(30.max(h / 9));
    }

    fn apply_layout(&mut self) {
        self.make_fonts();
        let (w, h) = (self.b.width, self.b.height);
        self.ground_h = (h as f64 * 0.12) as i32;
        self.bird_x = w as f64 * BIRD_X_FRAC;
        self.bird_r = 9.max((h as f64 * 0.028) as i32);
        self.pipe_w = (w as f64 * PIPE_W_FRAC) as i32;
        self.build_setup_layout();
    }

    /// `_new_cloud(x)` - x wird vom Aufrufer vorher gezogen (wie in Python).
    fn new_cloud(&self, x: Option<f64>) -> Cloud {
        let h = self.b.height as f64;
        let x = x.unwrap_or(self.b.width as f64);
        let y = py::uniform(h * 0.08, h * 0.5);
        let s = py::uniform(0.6, 1.4);
        let v = py::uniform(12.0, 30.0);
        Cloud { x, y, s, v }
    }

    fn new_game(&mut self) {
        self.b.score = 0;
        self.coins = 0;
        self.b.game_over = false;
        self.bird_y = self.b.height as f64 * 0.45;
        self.bird_v = 0.0;
        self.bird_a = 0.0;
        self.pipes.clear();
        self.pickups.clear();
        self.particles.clear();
        self.shield = false;
        self.shake = 0.0;
        self.ground_x = 0.0;
        self.dist = 0.0;
        self.wing_t = 0.0;
        self.new_best = false;
        self.over_t = 0.0;
        self.state = State::Ready;
    }

    fn theme(&self) -> &[C3; 6] {
        &THEMES[py::pmod(py::fdiv(self.b.score, 10), THEMES.len() as i64) as usize]
    }

    fn gap(&self) -> f64 {
        let base = DIFFS[self.diff].gap * self.b.height as f64;
        (self.bird_r as f64 * 4.5).max(base - self.b.score as f64 * 1.2)
    }

    fn speed(&self) -> f64 {
        DIFFS[self.diff].speed + self.b.score as f64 * 1.5
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
        self.b.set_opt("flappy", "difficulty", (self.diff as i64).into());
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

    fn is_flap(&self, key: &str) -> bool {
        self.b.is_action(key, "up", None) || matches!(key, "Up" | "space" | "w" | "W")
    }

    fn flap(&mut self) {
        self.bird_v = FLAP_V;
        self.wing_t = 0.0;
        self.b.play_sound("move");
        for _ in 0..4 {
            let vx = py::uniform(-60.0, -20.0);
            let vy = py::uniform(-20.0, 40.0);
            self.particles.push(([self.bird_x - self.bird_r as f64, self.bird_y, vx, vy, 0.35], (255, 255, 255)));
        }
    }

    fn spawn_pipes(&mut self) {
        let spacing = DIFFS[self.diff].spacing * self.b.width as f64;
        let need = match self.pipes.last() {
            None => true,
            Some(p) => p.x < self.b.width as f64 - spacing,
        };
        if !need {
            return;
        }
        let gap = self.gap();
        let h = self.b.height as f64;
        let top = h - self.ground_h as f64;
        let margin = h * 0.09;
        let gy = py::uniform(margin + gap / 2.0, top - margin - gap / 2.0);
        let pipe = Pipe { x: (self.b.width + self.pipe_w) as f64, gy, gap, passed: false };
        self.pipes.push(pipe);
        let n = self.pipes.len();
        let px = pipe.x + self.pipe_w as f64 / 2.0;
        if n % 3 == 0 {
            self.pickups.push(Pickup { x: px, y: gy, coin: true, taken: false });
        } else if n % 7 == 0 && !self.shield {
            self.pickups.push(Pickup { x: px, y: gy, coin: false, taken: false });
        }
    }

    fn move_pipes(&mut self, dt: f64) {
        let v = self.speed() * dt;
        let half = self.pipe_w as f64 / 2.0;
        for i in 0..self.pipes.len() {
            self.pipes[i].x -= v;
            if !self.pipes[i].passed && self.pipes[i].x + half < self.bird_x {
                self.pipes[i].passed = true;
                self.b.score += 1;
                self.b.play_sound("point");
            }
        }
        let pw = self.pipe_w as f64;
        self.pipes.retain(|p| p.x + pw > -4.0);
        for pk in self.pickups.iter_mut() {
            pk.x -= v;
        }
        self.pickups.retain(|pk| pk.x > -20.0 && !pk.taken);
    }

    fn circle_rect(cx: f64, cy: f64, r: f64, rect: Rect) -> bool {
        let nx = (rect.x as f64).max(cx.min(rect.right() as f64));
        let ny = (rect.y as f64).max(cy.min(rect.bottom() as f64));
        py::pow(cx - nx, 2.0) + py::pow(cy - ny, 2.0) < r * r
    }

    fn collisions(&mut self) {
        let (bx, by, r) = (self.bird_x, self.bird_y, self.bird_r as f64);
        let h = self.b.height;
        if by + r >= (h - self.ground_h) as f64 {
            self.hit();
            return;
        }
        for p in &self.pipes {
            let top_r = (p.x, 0, self.pipe_w, p.gy - p.gap / 2.0).rect();
            let bot_r = (p.x, p.gy + p.gap / 2.0, self.pipe_w, h).rect();
            if Self::circle_rect(bx, by, r, top_r) || Self::circle_rect(bx, by, r, bot_r) {
                self.hit();
                return;
            }
        }
        for i in 0..self.pickups.len() {
            let pk = self.pickups[i];
            if !pk.taken && py::hypot(bx - pk.x, by - pk.y) < r + 12.0 {
                self.pickups[i].taken = true;
                if pk.coin {
                    self.coins += 1;
                    self.b.score += 2;
                    self.b.play_sound("eat");
                } else {
                    self.shield = true;
                    self.b.play_sound("powerup");
                }
                self.sparkle(pk.x, pk.y, COL_COIN, 10);
            }
        }
    }

    fn hit(&mut self) {
        if self.shield {
            self.shield = false;
            self.bird_v = FLAP_V * 0.8;
            self.shake = 0.4;
            self.b.play_sound("hit");
            self.sparkle(self.bird_x, self.bird_y, COL_SHIELD, 10);
            return;
        }
        self.state = State::Dying;
        self.bird_v = FLAP_V * 0.5;
        self.shake = 0.7;
        self.b.play_sound("explode");
        self.b.rumble(250);
        self.sparkle(self.bird_x, self.bird_y, (255, 200, 90), 16);
    }

    fn finish(&mut self) {
        self.b.game_over = true;
        self.b.ach_event("flappy_gold", Some(self.b.score));
        self.new_best = self.b.score > self.highscore;
        self.highscore = self.highscore.max(self.b.score);
        self.over_t = self.anim_t;
        self.b.play_sound("gameover");
    }

    fn medal(&self) -> Option<(&'static str, C3)> {
        MEDALS.iter().find(|m| self.b.score >= m.0).map(|m| (m.1, m.2))
    }

    fn sparkle(&mut self, x: f64, y: f64, col: C3, n: usize) {
        for _ in 0..n {
            let a = py::uniform(0.0, py::TAU);
            let sp = py::uniform(40.0, 180.0);
            let life = py::uniform(0.25, 0.55);
            self.particles.push(([x, y, a.cos() * sp, a.sin() * sp, life], col));
        }
    }

    fn update_particles(&mut self, dt: f64) {
        for (p, _) in self.particles.iter_mut() {
            p[0] += p[2] * dt;
            p[1] += p[3] * dt;
            p[3] += 300.0 * dt;
            p[4] -= dt;
        }
        self.particles.retain(|(p, _)| p[4] > 0.0);
    }

    fn update_clouds(&mut self, dt: f64) {
        let f = if matches!(self.state, State::Setup | State::Ready | State::GameOver) { 0.4 } else { 1.0 };
        for i in 0..self.clouds.len() {
            let c = &mut self.clouds[i];
            c.x -= c.v * c.s * f * dt;
            if c.x < -80.0 {
                let mut neu = self.new_cloud(None);
                neu.x = (self.b.width + 60) as f64;
                self.clouds[i] = neu;
            }
        }
    }

    fn panel(&self, s: &mut Surface, rect: Rect, alpha: u8, border: i32) {
        let p = ui::pal();
        let mut surf = pg::surface_alpha(rect.w, rect.h);
        draw::rect_r(&mut surf, pg::alpha(p.panel, alpha), (0, 0, rect.w, rect.h), 0, 12);
        s.bl(&surf, (rect.x, rect.y));
        draw::rect_r(s, self.b.accent, rect, border, 12);
    }

    fn chip(&self, s: &mut Surface, text: &str, font: &Font, center: (i32, i32), color: Color) {
        let img = font.render(text, true, color);
        let bx = img.get_rect().center_at(center).inflate(24, 12);
        self.panel(s, bx, 200, 1);
        s.blr(&img, img.get_rect().center_at(bx.center()));
    }

    fn draw_sky(&mut self, s: &mut Surface) {
        let th = *self.theme();
        let (w, h) = (self.b.width, self.b.height);
        let key = (w, h, th[0], th[1]);
        if self.sky.as_ref().map(|c| c.0) != Some(key) {
            let mut surf = pg::surface(w, h);
            let (top, bot) = (th[0], th[1]);
            for y in 0..h {
                let tt = y as f64 / 1.max(h) as f64;
                let m = |a: i32, b: i32| (a as f64 + (b - a) as f64 * tt) as i32;
                draw::line(&mut surf, rgb(m(top.0, bot.0), m(top.1, bot.1), m(top.2, bot.2)), (0, y), (w, y), 1);
            }
            self.sky = Some((key, surf));
        }
        if let Some((_, surf)) = &self.sky {
            s.bl(surf, (0, 0));
        }
    }

    fn draw_clouds(&self, s: &mut Surface, ox: f64, oy: f64) {
        for c in &self.clouds {
            let (x, y, sc) = (c.x + ox, c.y + oy, c.s);
            for (dx, dy, rr) in [(0.0, 0.0, 26.0), (22.0, 6.0, 20.0), (-22.0, 6.0, 20.0), (0.0, 10.0, 22.0)] {
                let d = (rr * sc * 2.0) as i32;
                let mut surf = pg::surface_alpha(d, d);
                let m = (rr * sc) as i32;
                draw::circle(&mut surf, rgba(255, 255, 255, 150), (m, m), m, 0);
                s.bl(&surf, (x + dx * sc - rr * sc, y + dy * sc - rr * sc));
            }
        }
    }

    fn draw_pipe(&self, s: &mut Surface, p: &Pipe, ox: f64, oy: f64) {
        let th = self.theme();
        let (pipe, dark) = (c3(th[2]), c3(th[3]));
        let w = self.pipe_w as f64;
        let cap = ((self.b.height as f64 * CAP_H_FRAC) as i32 + 4) as f64;
        let x = p.x + ox;
        let top_h = p.gy - p.gap / 2.0;
        let bot_y = p.gy + p.gap / 2.0;
        draw::rect(s, pipe, (x, oy, w, top_h - cap), 0);
        draw::rect_r(s, pipe, (x - 4.0, top_h - cap + oy, w + 8.0, cap), 0, 4);
        draw::rect(s, dark, (x + w - 8.0, oy, 6.0, top_h), 0);
        let gh = (self.b.height - self.ground_h) as f64 - bot_y;
        draw::rect(s, pipe, (x, bot_y + cap + oy, w, gh), 0);
        draw::rect_r(s, pipe, (x - 4.0, bot_y + oy, w + 8.0, cap), 0, 4);
        draw::rect(s, dark, (x + w - 8.0, bot_y + oy, 6.0, gh + cap), 0);
        draw::rect(s, rgb(255, 255, 255), (x + 4.0, oy, 4.0, top_h - cap), 0);
    }

    fn draw_pickup(&self, s: &mut Surface, pk: &Pickup, ox: f64, oy: f64) {
        let (x, y) = ((pk.x + ox) as i32, (pk.y + oy) as i32);
        if pk.coin {
            let w = (10.0 + 5.0 * (self.anim_t * 6.0).sin()) as i32;
            draw::ellipse(s, c3(COL_COIN), (x - w, y - 12, 2 * w, 24), 0);
            draw::ellipse(s, rgb(255, 240, 160), (x - w + 2, y - 9, 2 * w - 4, 18), 2);
        } else {
            let mut surf = pg::surface_alpha(36, 36);
            draw::circle(&mut surf, pg::alpha(c3(COL_SHIELD), 90), (18, 18), 17, 0);
            draw::circle(&mut surf, c3(COL_SHIELD), (18, 18), 17, 2);
            s.bl(&surf, (x - 18, y - 18));
            let st = self.tiny.render("S", true, rgb(240, 250, 255));
            s.blr(&st, st.get_rect().center_at((x, y)));
        }
    }

    fn draw_ground(&self, s: &mut Surface) {
        let th = self.theme();
        let (gcol, gdark) = (c3(th[4]), c3(th[5]));
        let (w, h) = (self.b.width, self.b.height);
        let gy = h - self.ground_h;
        draw::rect(s, gcol, (0, gy, w, self.ground_h), 0);
        draw::rect(s, gdark, (0, gy, w, 5), 0);
        let step = 26;
        let off = py::pmod32(self.ground_x as i32, step);
        let mut x = -step;
        while x < w + step {
            draw::line(s, gdark, (x - off, gy + 8), (x - off + 12, h), 2);
            x += step;
        }
    }

    fn bird_surface(&self, r: f64, wing: f64) -> Surface {
        let d = (r * 3.0) as i32;
        let mut surf = pg::surface_alpha(d, d);
        let (cx, cy) = ((d / 2) as f64, (d / 2) as f64);
        draw::polygon(&mut surf, rgb(235, 180, 50), &[(cx - r, cy), (cx - r * 1.5, cy - r * 0.5), (cx - r * 1.5, cy + r * 0.5)], 0);
        draw::ellipse(&mut surf, rgb(250, 205, 60), (cx - r, cy - r * 0.85, r * 2.0, r * 1.7), 0);
        draw::ellipse(&mut surf, rgb(255, 240, 200), (cx - r * 0.4, cy - r * 0.1, r * 1.1, r * 0.9), 0);
        draw::ellipse(&mut surf, rgb(230, 180, 60), (cx - r * 0.35, cy - r * 0.15 + wing, r * 0.9, r * 0.6), 0);
        draw::circle(&mut surf, rgb(255, 255, 255), ((cx + r * 0.45) as i32, (cy - r * 0.32) as i32), (r * 0.32) as i32, 0);
        draw::circle(&mut surf, rgb(25, 25, 35), ((cx + r * 0.55) as i32, (cy - r * 0.32) as i32), (r * 0.15) as i32, 0);
        draw::polygon(
            &mut surf,
            rgb(250, 150, 40),
            &[(cx + r * 0.75, cy - r * 0.1), (cx + r * 1.35, cy), (cx + r * 0.75, cy + r * 0.25)],
            0,
        );
        surf
    }

    fn draw_bird(&self, s: &mut Surface, ox: f64, oy: f64) {
        let r = self.bird_r as f64;
        let wing = if matches!(self.state, State::Play | State::Ready) { (self.wing_t.min(0.3) * 30.0).sin() * r * 0.5 } else { 0.0 };
        let surf = self.bird_surface(r, wing);
        let ang = -py::degrees(self.bird_a);
        let rot = transform::rotozoom(&surf, ang, 1.0);
        s.blr(&rot, rot.get_rect().center_at((self.bird_x + ox, self.bird_y + oy)));
        if self.shield {
            let d = (r * 4.0) as i32;
            let mut bub = pg::surface_alpha(d, d);
            let m = (r * 2.0) as i32;
            draw::circle(&mut bub, pg::alpha(c3(COL_SHIELD), 70), (m, m), (r * 1.9) as i32, 0);
            draw::circle(&mut bub, rgb(150, 235, 255), (m, m), (r * 1.9) as i32, 2);
            s.bl(&bub, (self.bird_x + ox - r * 2.0, self.bird_y + oy - r * 2.0));
        }
    }

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        if matches!(self.state, State::Play | State::Dying) {
            let txt = self.b.score.to_string();
            let img = self.huge.render(&txt, true, rgb(245, 245, 250));
            let outline = self.huge.render(&txt, true, rgb(30, 30, 40));
            let r = img.get_rect().center_at((w / 2, (h as f64 * 0.14) as i32));
            for (dx, dy) in [(-2, 0), (2, 0), (0, -2), (0, 2)] {
                s.blr(&outline, r.move_(dx, dy));
            }
            s.blr(&img, r);
        }
        if self.state == State::Ready {
            self.chip(s, &tf("fb.best", &[("hs", &self.highscore.to_string())]), &self.small, (w / 2, (h as f64 * 0.1) as i32), p.text_dim);
            let hint_col = ui::mix(p.text_dim, p.text, ui::pulse(2.4, 0.0, 1.0));
            self.chip(s, &t("fb.tap_hint"), &self.small, (w / 2, (h as f64 * 0.38) as i32), hint_col);
        }
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
        let cx = w / 2;
        let img = self.huge.render(&t("common.game_over"), true, self.b.accent);
        s.blr(&img, img.get_rect().center_at((cx, (h as f64 * 0.18) as i32)));
        let lh = self.small.get_height() + 8;
        let pw = 360.min(w - 40);
        let ph = lh * 3 + 40;
        let panel = Rect::new(cx - py::fdiv32(pw, 2), (h as f64 * 0.32) as i32, pw, ph);
        self.panel(s, panel, 235, 2);
        let (x, mut y) = (panel.x + 20, panel.y + 18);
        let l1 = self.small.render(&tf("common.points", &[("score", &self.b.score.to_string())]), true, p.text);
        s.bl(&l1, (x, y));
        y += lh;
        let l2 = self.small.render(&tf("fb.coins", &[("n", &self.coins.to_string())]), true, p.gold);
        s.bl(&l2, (x, y));
        y += lh;
        let best_col = if self.new_best { p.gold } else { p.text_dim };
        let l3 = self.small.render(&tf("fb.best", &[("hs", &self.highscore.to_string())]), true, best_col);
        s.bl(&l3, (x, y));
        if let Some((key, col)) = self.medal() {
            let mr = 24.max(36.min(panel.h / 4));
            let (mcx, mcy) = (panel.right() - mr - 24, panel.centery() - 8);
            draw::circle(s, c3(col), (mcx, mcy), mr, 0);
            let d = |c: i32| (c as f64 * 0.7) as i32;
            draw::circle(s, rgb(d(col.0), d(col.1), d(col.2)), (mcx, mcy), mr, 3);
            self.star(s, mcx, mcy, (mr as f64 * 0.55) as i32, rgb(255, 255, 255));
            let lab = self.tiny.render(&t(&format!("fb.medal.{key}")), true, c3(col));
            s.blr(&lab, lab.get_rect().center_at((mcx, mcy + mr + 12)));
        }
        if self.new_best {
            let rec_col = ui::mix(p.gold, p.text, ui::pulse(3.0, 0.0, 0.5));
            let rec = self.small.render(&t("trex.new_record"), true, rec_col);
            s.blr(&rec, rec.get_rect().center_at((cx, panel.y - 18)));
        }
        let hint_col = ui::mix(p.text_dim, p.text, ui::pulse(2.4, 0.0, 1.0));
        let hint = self.small.render(&t("fb.restart_hint"), true, hint_col);
        s.blr(&hint, hint.get_rect().center_at((cx, panel.bottom() + 26)));
    }

    fn star(&self, s: &mut Surface, cx: i32, cy: i32, r: i32, col: Color) {
        let pts: Vec<(f64, f64)> = (0..10)
            .map(|i| {
                let rr = if i % 2 == 0 { r as f64 } else { r as f64 * 0.45 };
                let a = -py::PI / 2.0 + i as f64 * py::PI / 5.0;
                (cx as f64 + a.cos() * rr, cy as f64 + a.sin() * rr)
            })
            .collect();
        draw::polygon(s, col, &pts, 0);
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
        self.draw_clouds(&mut s, 0.0, 0.0);
        ui::draw_title_ex(&mut s, w, "FLAPPY BIRD", Some(&t("snake.singleplayer")), 52, None, None, Some(self.b.accent));
        let d = &DIFFS[self.diff];
        self.panel(&mut s, self.diff_panel, 235, 2);
        let name = self.b.font.render(&format!("{}:  {}", t("fb.difficulty"), t(&format!("fb.diff.{}", d.key))), true, p.text);
        let dp = self.diff_panel;
        s.blr(&name, name.get_rect().center_at((dp.centerx(), dp.y + (dp.h as f64 * 0.36) as i32)));
        let note = self.tiny.render(&t("fb.diff_note"), true, p.text_dim);
        s.blr(&note, note.get_rect().center_at((dp.centerx(), dp.y + (dp.h as f64 * 0.72) as i32)));
        let arr_col = ui::mix(self.b.accent, p.text, ui::pulse(3.0, 0.0, 0.3));
        for (rect, sym) in [(self.diff_left, "<"), (self.diff_right, ">")] {
            let arr = self.b.big_font.render(sym, true, arr_col);
            s.blr(&arr, arr.get_rect().center_at(rect.center()));
        }
        ui::draw_button_ex(&mut s, self.start_rect, &t("common.start"), &self.b.font, true, None, None, Some(self.b.accent));
        let ctrl = self.tiny.render(&t("fb.controls_hint"), true, p.green);
        s.blr(&ctrl, ctrl.get_rect().center_at((w / 2, self.start_rect.bottom() + 24)));
        ui::draw_footer(&mut s, w, h, &t("fb.setup_hint"));
    }
}

impl Game for FlappyGame {
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
        self.diff = self.b.opt_i64("flappy", "difficulty", 1).clamp(0, 2) as usize;
        self.highscore = highscore::get("flappy");
        self.anim_t = 0.0;
        self.sky = None;
        self.ui_bg = None;
        self.apply_layout();
        let w = self.b.width as f64;
        self.clouds = (0..5)
            .map(|_| {
                let x = py::uniform(0.0, w);
                self.new_cloud(Some(x))
            })
            .collect();
        self.new_game();
        self.state = State::Setup;
    }

    fn on_surface_changed(&mut self) {
        self.apply_layout();
        self.sky = None;
        self.ui_bg = None;
        let lo = self.bird_r as f64;
        let hi = (self.b.height - self.ground_h - self.bird_r) as f64;
        self.bird_y = lo.max(self.bird_y.min(hi));
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
            _ => {}
        }
        let flap = ev.kind == Kind::MouseDown || (ev.kind == Kind::KeyDown && self.is_flap(&ev.key));
        if flap {
            if self.state == State::Ready {
                self.state = State::Play;
            }
            if self.state == State::Play {
                self.flap();
            }
        }
    }

    fn update(&mut self, dt: f64) {
        self.anim_t += dt;
        self.wing_t += dt;
        self.update_clouds(dt);
        self.update_particles(dt);
        if self.shake > 0.0 {
            self.shake = 0.0f64.max(self.shake - dt * 2.0);
        }
        let floor = (self.b.height - self.ground_h - self.bird_r) as f64;
        match self.state {
            State::Setup | State::GameOver => return,
            State::Ready => {
                self.bird_y = self.b.height as f64 * 0.45 + (self.anim_t * 3.0).sin() * 12.0;
                return;
            }
            State::Dying => {
                self.bird_v += GRAV * dt;
                self.bird_y += self.bird_v * dt;
                self.bird_a = (-1.4f64).max(self.bird_a - dt * 4.0);
                if self.bird_y >= floor {
                    self.bird_y = floor;
                    self.state = State::GameOver;
                    self.finish();
                }
                return;
            }
            State::Play => {}
        }
        self.dist += self.speed() * dt;
        self.ground_x = py::fpmod(self.ground_x - self.speed() * dt, self.b.width as f64);
        self.bird_v += GRAV * dt;
        self.bird_y += self.bird_v * dt;
        self.bird_a = (-1.3f64).max(1.4f64.min(self.bird_v / 520.0));
        if self.bird_y < self.bird_r as f64 {
            self.bird_y = self.bird_r as f64;
            self.bird_v = self.bird_v.max(40.0);
        }
        self.spawn_pipes();
        self.move_pipes(dt);
        self.collisions();
    }

    fn draw(&mut self) {
        if self.state == State::Setup {
            self.draw_setup();
            return;
        }
        let mut s = self.b.sf();
        let (mut ox, mut oy) = (0.0, 0.0);
        if self.shake > 0.0 {
            let amp = 8.0 * self.shake;
            ox = py::uniform(-amp, amp);
            oy = py::uniform(-amp, amp);
        }
        self.draw_sky(&mut s);
        self.draw_clouds(&mut s, ox * 0.3, oy * 0.3);
        for p in &self.pipes {
            self.draw_pipe(&mut s, p, ox, oy);
        }
        for pk in &self.pickups {
            self.draw_pickup(&mut s, pk, ox, oy);
        }
        self.draw_ground(&mut s);
        for (p, col) in &self.particles {
            let a = 0.max(255.min((255.0 * p[4] / 0.55) as i32));
            let mut surf = pg::surface_alpha(5, 5);
            draw::circle(&mut surf, rgba(col.0, col.1, col.2, a), (2, 2), 2, 0);
            s.bl(&surf, (p[0] - 2.0 + ox, p[1] - 2.0 + oy));
        }
        self.draw_bird(&mut s, ox, oy);
        self.draw_hud(&mut s);
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "flappy",
    class_name: "FlappyGame",
    supports_multiplayer: false,
    modes: &[],
    make: FlappyGame::new,
};
