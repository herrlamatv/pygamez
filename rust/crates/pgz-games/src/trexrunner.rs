//! trexrunner.rs
//! =============
//! T-Rex Runner - der endlose Wuestenlauf.
//!
//! Uebertragen aus `games/trexrunner.py`: variable Sprunghoehe, Ducken,
//! Kakteen und Flugsaurier auf drei Hoehen, Tag/Nacht-Wechsel, Wolken,
//! scrollender Boden; Schwierigkeit, Figur und Nacht bleiben gespeichert.

use pgz_core::highscore;
use pgz_engine::prelude::*;

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct Pal {
    sky: (i32, i32, i32),
    ground: (i32, i32, i32),
    line: (i32, i32, i32),
    obj: (i32, i32, i32),
    cloud: (i32, i32, i32),
    star: (i32, i32, i32),
    text: (i32, i32, i32),
    dim: (i32, i32, i32),
}

const DAY: Pal = Pal {
    sky: (244, 246, 250),
    ground: (84, 84, 92),
    line: (120, 120, 130),
    obj: (74, 78, 90),
    cloud: (210, 214, 224),
    star: (244, 246, 250),
    text: (60, 64, 78),
    dim: (120, 126, 140),
};
const NIGHT: Pal = Pal {
    sky: (24, 26, 42),
    ground: (150, 154, 168),
    line: (110, 116, 140),
    obj: (198, 202, 216),
    cloud: (70, 76, 104),
    star: (250, 250, 210),
    text: (226, 230, 242),
    dim: (150, 156, 178),
};

const SKINS: [(i32, i32, i32); 4] = [(94, 106, 122), (86, 200, 130), (240, 150, 70), (110, 160, 240)];
const DIFFS: [&str; 3] = ["chill", "normal", "hardcore"];

fn diff_params(d: &str) -> (f64, f64, i64) {
    match d {
        "chill" => (300.0, 5.0, 250),
        "hardcore" => (470.0, 12.0, 60),
        _ => (380.0, 8.0, 150),
    }
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Ready,
    Run,
    Over,
}

#[derive(Clone, Copy, PartialEq)]
enum Bird {
    High,
    Mid,
    Low,
}

#[derive(Clone)]
struct Obstacle {
    x: f64,
    w: i32,
    h: i32,
    bird: bool,
    oy: i32,
    flap: f64,
    spikes: i32,
}

/// `TRexRunnerGame`
pub struct TRexRunnerGame {
    b: GameBase,
    diff: &'static str,
    skin: usize,
    night_on: bool,
    big: Font,
    mid: Font,
    small: Font,
    tiny: Font,
    mono: Font,
    best: i64,
    state: State,
    scale: f64,
    gy: i32,
    dino_x: i32,
    dino_w: i32,
    dino_h: i32,
    duck_w: i32,
    duck_h: i32,
    speed: f64,
    base_speed: f64,
    accel: f64,
    max_speed: f64,
    dist: f64,
    next_milestone: i64,
    dy: f64,
    vy: f64,
    on_ground: bool,
    ducking: bool,
    jump_held: bool,
    duck_held: bool,
    run_frame: f64,
    dead_t: f64,
    blink: f64,
    obstacles: Vec<Obstacle>,
    spawn_gap: f64,
    clouds: Vec<[f64; 3]>,
    stars: Vec<(f64, f64, i32)>,
    ground_bumps: Vec<f64>,
    night: f64,
    night_target: f64,
    next_flip: i64,
}

impl TRexRunnerGame {
    /// `TRexRunnerGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let mut g = TRexRunnerGame {
            b,
            diff: "normal",
            skin: 1,
            night_on: true,
            big: f.clone(),
            mid: f.clone(),
            small: f.clone(),
            tiny: f.clone(),
            mono: f,
            best: 0,
            state: State::Ready,
            scale: 1.0,
            gy: 0,
            dino_x: 0,
            dino_w: 0,
            dino_h: 0,
            duck_w: 0,
            duck_h: 0,
            speed: 0.0,
            base_speed: 1.0,
            accel: 0.0,
            max_speed: 0.0,
            dist: 0.0,
            next_milestone: 100,
            dy: 0.0,
            vy: 0.0,
            on_ground: true,
            ducking: false,
            jump_held: false,
            duck_held: false,
            run_frame: 0.0,
            dead_t: 0.0,
            blink: 0.0,
            obstacles: Vec::new(),
            spawn_gap: 0.0,
            clouds: Vec::new(),
            stars: Vec::new(),
            ground_bumps: Vec::new(),
            night: 0.0,
            night_target: 0.0,
            next_flip: 300,
        };
        g.reset();
        Box::new(g)
    }

    fn make_fonts(&mut self) {
        let h = self.b.height;
        self.big = ui::fb(26.max(h / 12));
        self.mid = ui::fb(16.max(h / 24));
        self.small = ui::f(12.max(h / 32));
        self.tiny = ui::f(11.max(h / 36));
        self.mono = ui::font(14.max(h / 26), true, true);
    }

    fn layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        self.scale = h as f64 / 480.0;
        self.gy = (h as f64 * 0.80) as i32;
        self.dino_x = (w as f64 * 0.16) as i32;
        self.dino_w = (44.0 * self.scale) as i32;
        self.dino_h = (48.0 * self.scale) as i32;
        self.duck_w = (60.0 * self.scale) as i32;
        self.duck_h = (28.0 * self.scale) as i32;
    }

    fn make_stars(&self) -> Vec<(f64, f64, i32)> {
        (0..46)
            .map(|_| {
                let x = py::uniform(0.0, self.b.width as f64);
                let y = py::uniform(0.0, self.gy as f64 * 0.7);
                (x, y, py::choice(&[1, 1, 2]))
            })
            .collect()
    }

    fn new_run(&mut self) {
        let (base, accel, _) = diff_params(self.diff);
        self.speed = base * self.scale;
        self.base_speed = base * self.scale;
        self.accel = accel * self.scale;
        self.max_speed = 900.0 * self.scale;
        self.dist = 0.0;
        self.b.score = 0;
        self.next_milestone = 100;
        self.dy = 0.0;
        self.vy = 0.0;
        self.on_ground = true;
        self.ducking = false;
        self.jump_held = false;
        self.duck_held = false;
        self.run_frame = 0.0;
        self.dead_t = 0.0;
        self.blink = 0.0;
        self.obstacles.clear();
        self.spawn_gap = self.rand_gap() * 0.6;
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        self.clouds = (0..3)
            .map(|_| {
                let x = py::uniform(0.0, w);
                let y = py::uniform(h * 0.12, h * 0.42);
                [x, y, py::uniform(0.25, 0.5)]
            })
            .collect();
        self.stars = self.make_stars();
        self.ground_bumps = (0..24).map(|_| py::uniform(0.0, w)).collect();
        self.night = 0.0;
        self.night_target = 0.0;
        self.next_flip = 300;
    }

    fn rand_gap(&self) -> f64 {
        let base = py::uniform(320.0, 560.0) * self.scale;
        base * py::pow(self.speed / self.base_speed, 0.5)
    }

    fn save_setting(&mut self, key: &str, value: serde_json::Value) {
        self.b.set_opt("trex", key, value);
    }

    fn is_jump_key(&self, k: &str) -> bool {
        matches!(k, "space" | "Up" | "w" | "W") || self.b.is_action(k, "action", None) || self.b.is_action(k, "up", None)
    }

    fn is_duck_key(&self, k: &str) -> bool {
        matches!(k, "Down" | "s" | "S") || self.b.is_action(k, "down", None)
    }

    fn handle_ready(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown {
            let k = ev.key.as_str();
            if matches!(k, "1" | "2" | "3") {
                self.diff = DIFFS[k.parse::<usize>().unwrap_or(2) - 1];
                self.save_setting("difficulty", self.diff.into());
                self.new_run();
                self.b.play_sound("click");
            } else if matches!(k, "n" | "N") {
                self.night_on = !self.night_on;
                self.save_setting("night", self.night_on.into());
                self.b.play_sound("click");
            } else if matches!(k, "c" | "C") {
                self.skin = (self.skin + 1) % SKINS.len();
                self.save_setting("skin", (self.skin as i64).into());
                self.b.play_sound("click");
            } else if matches!(k, "space" | "Up" | "Return" | "w" | "W") || self.b.is_action(k, "action", None) {
                self.start_running();
            }
        } else if ev.kind == Kind::MouseDown {
            self.start_running();
        }
    }

    fn start_running(&mut self) {
        self.state = State::Run;
        self.jump_held = true;
        self.try_jump();
        self.b.play_sound("select");
    }

    fn try_jump(&mut self) {
        if self.on_ground && self.state == State::Run {
            self.vy = -620.0 * self.scale;
            self.on_ground = false;
            self.ducking = false;
            self.b.play_sound("bounce");
        }
    }

    fn update_dino(&mut self, dt: f64) {
        if self.on_ground {
            self.ducking = self.duck_held;
        } else {
            let mut g = 1800.0 * self.scale;
            if self.jump_held && self.vy < 0.0 {
                g = 1150.0 * self.scale;
            }
            self.vy += g * dt;
            if self.duck_held {
                self.vy += 2600.0 * self.scale * dt;
            }
            self.dy -= self.vy * dt;
            if self.dy <= 0.0 {
                self.dy = 0.0;
                self.vy = 0.0;
                self.on_ground = true;
                self.ducking = self.duck_held;
            }
        }
    }

    fn dino_rect(&self) -> Rect {
        let (w, h) = if self.ducking && self.on_ground { (self.duck_w, self.duck_h) } else { (self.dino_w, self.dino_h) };
        Rect::new(self.dino_x, self.gy - self.dy as i32 - h, w, h)
    }

    fn update_obstacles(&mut self, dt: f64) {
        for o in self.obstacles.iter_mut() {
            o.x -= self.speed * dt;
            if o.bird {
                o.flap += dt * 8.0;
            }
        }
        self.obstacles.retain(|o| o.x + o.w as f64 > -4.0);
        self.spawn_gap -= self.speed * dt;
        if self.spawn_gap <= 0.0 {
            self.spawn();
            self.spawn_gap = self.rand_gap();
        }
        let sc = self.scale;
        let dr = self.dino_rect().inflate((-8.0 * sc) as i32, (-8.0 * sc) as i32);
        let hit = self.obstacles.iter().any(|o| {
            let r = Rect::new(o.x as i32, o.oy, o.w, o.h).inflate((-5.0 * sc) as i32, (-6.0 * sc) as i32);
            dr.colliderect(r)
        });
        if hit {
            self.die();
        }
    }

    fn spawn(&mut self) {
        let (_, _, bird_at) = diff_params(self.diff);
        let allow_bird = self.b.score >= bird_at;
        let sc = self.scale;
        let x = self.b.width as f64 + 20.0 * sc;
        if allow_bird && py::random() < 0.28 {
            let kind = py::choice(&[Bird::High, Bird::Mid, Bird::Low]);
            let w = (46.0 * sc) as i32;
            let h = (30.0 * sc) as i32;
            let off = match kind {
                Bird::High => 92.0,
                Bird::Mid => 58.0,
                Bird::Low => 26.0,
            };
            let oy = self.gy - (off * sc) as i32 - h;
            self.obstacles.push(Obstacle { x, w, h, bird: true, oy, flap: 0.0, spikes: 1 });
        } else {
            let variant = py::random();
            let (w, h, spikes);
            if variant < 0.4 {
                w = (18.0 * sc) as i32;
                h = (36.0 * sc) as i32;
                spikes = 1;
            } else if variant < 0.72 {
                w = (26.0 * sc) as i32;
                h = (50.0 * sc) as i32;
                spikes = 1;
            } else {
                spikes = py::choice(&[2, 3]);
                w = ((14 * spikes + 6) as f64 * sc) as i32;
                h = (40.0 * sc) as i32;
            }
            let oy = self.gy - h;
            self.obstacles.push(Obstacle { x, w, h, bird: false, oy, flap: 0.0, spikes });
        }
    }

    fn update_night(&mut self, dt: f64) {
        if !self.night_on {
            self.night += (0.0 - self.night) * 1.0f64.min(dt * 3.0);
            return;
        }
        if self.state == State::Run && self.b.score >= self.next_flip {
            self.night_target = 1.0 - self.night_target;
            self.next_flip += 250;
            if self.night_target > 0.0 {
                self.b.ach_event("trex_night", None);
            }
        }
        self.night += (self.night_target - self.night) * 1.0f64.min(dt * 1.4);
    }

    fn die(&mut self) {
        self.state = State::Over;
        self.b.game_over = true;
        self.dead_t = 0.0;
        self.ducking = false;
        if self.b.score > self.best {
            self.best = self.b.score;
        }
        self.b.play_sound("gameover");
    }

    fn pal(&self, key: fn(&Pal) -> (i32, i32, i32)) -> Color {
        let (d, n) = (key(&DAY), key(&NIGHT));
        let f = self.night;
        let mix = |a: i32, b: i32| (a as f64 + (b - a) as f64 * f) as i32;
        rgb(mix(d.0, n.0), mix(d.1, n.1), mix(d.2, n.2))
    }

    fn draw_sky(&self, s: &mut Surface) {
        let (w, h) = (self.b.width, self.b.height);
        let cx = (w as f64 * 0.82) as i32;
        let cy = (h as f64 * 0.20) as i32;
        let r = (24.0 * self.scale) as i32;
        if self.night > 0.35 {
            let star_col = self.pal(|p| p.star);
            for &(sx, sy, sr) in &self.stars {
                s.fill_r(star_col, (sx as i32, sy as i32, sr, sr));
            }
            draw::circle(s, rgb(232, 234, 220), (cx, cy), r, 0);
            draw::circle(s, self.pal(|p| p.sky), (cx + (9.0 * self.scale) as i32, cy - (6.0 * self.scale) as i32), r, 0);
        } else {
            draw::circle(s, rgb(250, 214, 120), (cx, cy), r, 0);
            draw::circle(s, rgb(250, 224, 150), (cx, cy), (r as f64 * 1.5) as i32, 2);
        }
        let col = self.pal(|p| p.cloud);
        for c in &self.clouds {
            self.draw_cloud(s, c[0] as i32, c[1] as i32, col);
        }
    }

    fn draw_cloud(&self, s: &mut Surface, x: i32, y: i32, col: Color) {
        let u = (9.0 * self.scale) as i32;
        draw::ellipse(s, col, (x, y, u * 5, u * 2), 0);
        draw::ellipse(s, col, (x + u, y - u, u * 3, u * 2), 0);
    }

    fn draw_ground(&self, s: &mut Surface) {
        let gy = self.gy;
        let w = self.b.width;
        let sc = self.scale;
        draw::line(s, self.pal(|p| p.line), (0, gy), (w, gy), 2.max((2.0 * sc) as i32));
        let off = if self.state != State::Ready { py::fpmod(self.dist * 0.5, w as f64) } else { 0.0 };
        let col = self.pal(|p| p.line);
        for &bx in &self.ground_bumps {
            let x = py::fpmod(bx - off, w as f64) as i32;
            let yy = gy + (6.0 * sc) as i32;
            draw::line(s, col, (x, yy), (x + (10.0 * sc) as i32, yy), 1.max(sc as i32));
        }
    }

    fn draw_dino(&self, s: &mut Surface) {
        let r = self.dino_rect();
        let mut body = SKINS[self.skin];
        if self.night > 0.5 {
            let lift = |c: i32| 255.min((c as f64 + 60.0 * (self.night - 0.5) * 2.0) as i32);
            body = (lift(body.0), lift(body.1), lift(body.2));
        }
        let dark = rgb((body.0 as f64 * 0.6) as i32, (body.1 as f64 * 0.6) as i32, (body.2 as f64 * 0.6) as i32);
        let body = rgb(body.0, body.1, body.2);
        let u = self.scale;
        let iu = |v: f64| (v * u) as i32;
        if self.ducking && self.on_ground {
            draw::rect_r(s, body, r, 0, iu(6.0));
            let eye = (r.right() - iu(8.0), r.y + iu(7.0));
            draw::circle(s, rgb(250, 250, 250), eye, 2.max(iu(3.0)), 0);
            draw::circle(s, rgb(20, 20, 20), eye, 1.max(iu(1.5)), 0);
            let f = self.run_frame as i64 % 2;
            for i in 0..2i64 {
                let lx = r.x + (((10 + i * 22) as f64) * u) as i32;
                let ly = r.bottom();
                let dh = ((if (i + f) % 2 != 0 { 6.0 } else { 3.0 }) * u) as i32;
                draw::rect(s, dark, (lx, ly, iu(5.0), dh), 0);
            }
            return;
        }
        let head = Rect::new(r.right() - iu(20.0), r.y, iu(20.0), iu(18.0));
        draw::rect_r(s, body, (r.x + iu(6.0), r.y + iu(10.0), r.w - iu(10.0), r.h - iu(18.0)), 0, iu(5.0));
        draw::rect_r(s, body, head, 0, iu(4.0));
        draw::polygon(
            s,
            body,
            &[(r.x + iu(6.0), r.y + iu(14.0)), (r.x - iu(8.0), r.y + iu(8.0)), (r.x + iu(6.0), r.y + iu(24.0))],
            0,
        );
        let eye = (head.right() - iu(6.0), head.y + iu(6.0));
        draw::circle(s, rgb(250, 250, 250), eye, 2.max(iu(3.0)), 0);
        draw::circle(s, rgb(20, 20, 20), eye, 1.max(iu(1.5)), 0);
        let f = if self.on_ground && self.state == State::Run { self.run_frame as i64 % 2 } else { 0 };
        for i in 0..2i64 {
            let lx = r.x + (((10 + i * 14) as f64) * u) as i32;
            let ly = r.bottom() - iu(10.0);
            let dh = ((if (i + f) % 2 != 0 { 10.0 } else { 5.0 }) * u) as i32;
            draw::rect(s, dark, (lx, ly, iu(6.0), dh), 0);
        }
    }

    fn draw_obstacles(&self, s: &mut Surface) {
        let col = self.pal(|p| p.obj);
        let u = self.scale;
        let iu = |v: f64| (v * u) as i32;
        for o in &self.obstacles {
            let (x, oy, w, h) = (o.x as i32, o.oy, o.w, o.h);
            if o.bird {
                let cy = oy + py::fdiv32(h, 2);
                draw::ellipse(s, col, (x + iu(8.0), cy - iu(5.0), iu(28.0), iu(12.0)), 0);
                draw::circle(s, col, (x + w - iu(4.0), cy), iu(6.0), 0);
                draw::polygon(s, col, &[(x + w, cy - iu(2.0)), (x + w + iu(8.0), cy), (x + w, cy + iu(2.0))], 0);
                let up = py::fpmod(o.flap, 2.0) < 1.0;
                let tip = if up { cy - iu(16.0) } else { cy + iu(16.0) };
                draw::polygon(s, col, &[(x + iu(10.0), cy), (x + iu(24.0), cy), (x + iu(14.0), tip)], 0);
            } else {
                let spikes = o.spikes;
                let cw = py::fdiv32(w, spikes);
                for i in 0..spikes {
                    let bx = x + i * cw;
                    let c2 = py::fdiv32(cw, 2);
                    draw::rect(s, col, (bx + c2 - iu(3.0), oy, iu(6.0), h), 0);
                    let ay = oy + py::fdiv32(h, 3);
                    draw::rect(s, col, (bx + c2 - iu(9.0), ay, iu(6.0), iu(3.0)), 0);
                    draw::rect(s, col, (bx + c2 - iu(9.0), ay - iu(8.0), iu(3.0), iu(10.0)), 0);
                    draw::rect(s, col, (bx + c2 + iu(3.0), ay + iu(4.0), iu(6.0), iu(3.0)), 0);
                    draw::rect(s, col, (bx + c2 + iu(6.0), ay - iu(4.0), iu(3.0), iu(10.0)), 0);
                }
            }
        }
    }

    fn draw_hud(&self, s: &mut Surface) {
        let w = self.b.width;
        let cur = format!("{:05}", self.b.score);
        let hi = format!("{:05}", self.best);
        let blink_on = self.blink > 0.0 && (self.blink * 8.0) as i64 % 2 == 0;
        let col = self.pal(|p| p.dim);
        let txt = self.mono.render(&format!("HI {hi}"), true, col);
        s.bl(&txt, (w - txt.w() - (90.0 * self.scale) as i32, 12));
        if !blink_on {
            let cimg = self.mono.render(&cur, true, self.pal(|p| p.text));
            s.bl(&cimg, (w - cimg.w() - 14, 12));
        }
    }

    fn draw_ready(&self, s: &mut Surface) {
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let at = |f: f64| (h as f64 * f) as i32;
        let title = self.big.render(&t("trex.title"), true, self.pal(|p| p.text));
        s.blr(&title, title.get_rect().center_at((cx, at(0.24))));
        if (pg::ticks() as f64 / 500.0) as i64 % 2 == 0 {
            let ps = self.mid.render(&t("trex.press_start"), true, self.pal(|p| p.text));
            s.blr(&ps, ps.get_rect().center_at((cx, at(0.36))));
        }
        let ctrl = self.small.render(&t("trex.controls"), true, self.pal(|p| p.dim));
        s.blr(&ctrl, ctrl.get_rect().center_at((cx, at(0.44))));
        let onoff = if self.night_on { t("common.on") } else { t("common.off") };
        let opts = [
            format!("[1-3] {}: {}", t("trex.difficulty"), t(&format!("trex.diff.{}", self.diff))),
            format!("[N] {}: {onoff}", t("trex.night")),
            format!("[C] {}: {}", t("trex.skin"), self.skin + 1),
        ]
        .join("  \u{b7}  ");
        let oimg = self.tiny.render(&opts, true, self.pal(|p| p.dim));
        s.blr(&oimg, oimg.get_rect().center_at((cx, at(0.52))));
    }

    fn draw_over(&self, s: &mut Surface) {
        let cx = self.b.width / 2;
        let cy = (self.b.height as f64 * 0.34) as i32;
        let go = self.big.render(&t("common.game_over"), true, self.pal(|p| p.text));
        s.blr(&go, go.get_rect().center_at((cx, cy)));
        if self.b.score >= self.best && self.b.score > 0 {
            let rec = self.small.render(&t("trex.new_record"), true, ui::gold());
            s.blr(&rec, rec.get_rect().center_at((cx, cy + (30.0 * self.scale) as i32)));
        }
        if (self.dead_t * 2.0) as i64 % 2 == 0 {
            let hint = self.small.render(&t("common.enter_restart"), true, self.pal(|p| p.dim));
            s.blr(&hint, hint.get_rect().center_at((cx, cy + (58.0 * self.scale) as i32)));
        }
    }
}

impl Game for TRexRunnerGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        self.b.game_over = false;
        self.b.score = 0;
        let d = self.b.opt_str("trex", "difficulty", "normal");
        self.diff = DIFFS.iter().copied().find(|x| *x == d).unwrap_or("normal");
        self.skin = self.b.opt_i64("trex", "skin", 1).clamp(0, SKINS.len() as i64 - 1) as usize;
        self.night_on = self.b.opt_bool("trex", "night", true);
        self.make_fonts();
        self.best = highscore::get("trex");
        self.layout();
        self.new_run();
        self.state = State::Ready;
    }

    fn on_surface_changed(&mut self) {
        self.make_fonts();
        self.layout();
        self.stars = self.make_stars();
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        match self.state {
            State::Ready => {
                self.handle_ready(ev);
                return;
            }
            State::Over => {
                if (ev.kind == Kind::KeyDown && ev.key_in(&["Return", "space", "Up"])) || ev.kind == Kind::MouseDown {
                    self.new_run();
                    self.state = State::Run;
                    self.b.play_sound("select");
                }
                return;
            }
            State::Run => {}
        }
        match ev.kind {
            Kind::KeyDown => {
                let k = ev.key.as_str();
                if self.is_jump_key(k) {
                    self.jump_held = true;
                    self.try_jump();
                } else if self.is_duck_key(k) {
                    self.duck_held = true;
                }
            }
            Kind::KeyUp => {
                let k = ev.key.as_str();
                if self.is_jump_key(k) {
                    self.jump_held = false;
                } else if self.is_duck_key(k) {
                    self.duck_held = false;
                }
            }
            Kind::MouseDown => {
                self.jump_held = true;
                self.try_jump();
            }
            Kind::MouseUp => self.jump_held = false,
            _ => {}
        }
    }

    fn update(&mut self, dt: f64) {
        let dt = dt.min(0.05);
        if self.blink > 0.0 {
            self.blink -= dt;
        }
        let fac = if self.state != State::Run { 0.4 } else { 1.0 };
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        for i in 0..self.clouds.len() {
            let c = &mut self.clouds[i];
            c[0] -= self.base_speed * c[2] * dt * fac;
            if c[0] < -60.0 * self.scale {
                c[0] = w + py::uniform(20.0, 120.0) * self.scale;
                c[1] = py::uniform(h * 0.12, h * 0.42);
            }
        }
        if self.state == State::Over {
            self.dead_t += dt;
            self.update_night(dt);
            return;
        }
        if self.state != State::Run {
            self.run_frame += dt * 6.0;
            self.update_night(dt);
            return;
        }
        self.speed = self.max_speed.min(self.speed + self.accel * dt);
        self.dist += self.speed * dt;
        let old = self.b.score;
        self.b.score = (self.dist / (10.0 * self.scale)) as i64;
        if py::fdiv(self.b.score, 100) > py::fdiv(old, 100) && self.b.score >= self.next_milestone {
            self.next_milestone += 100;
            self.blink = 0.6;
            self.b.play_sound("point");
        }
        self.update_dino(dt);
        self.update_obstacles(dt);
        self.update_night(dt);
        self.run_frame += dt * (6.0 + self.speed / (120.0 * self.scale));
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        s.fill_c(self.pal(|p| p.sky));
        self.draw_sky(&mut s);
        self.draw_ground(&mut s);
        self.draw_obstacles(&mut s);
        self.draw_dino(&mut s);
        self.draw_hud(&mut s);
        match self.state {
            State::Ready => self.draw_ready(&mut s),
            State::Over => self.draw_over(&mut s),
            State::Run => {}
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "trex",
    class_name: "TRexRunnerGame",
    supports_multiplayer: false,
    modes: &[],
    make: TRexRunnerGame::new,
};
