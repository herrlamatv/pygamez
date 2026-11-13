//! aimtrainer.rs
//! =============
//! Aim Trainer - chilliges 3D-Zielschiessen (Einzelspieler).
//!
//! Uebertragen aus `games/aimtrainer.py`: FPS-Blick ueber relative
//! Mausbewegung (Pointer-Capture), vier Modi (precision/reflex/moving/chill),
//! drei Themes (space/neon/range) mit Software-3D (Kamera-Basis, Projektion,
//! Near-Clip, Painter-Sortierung, Nebel, Billboards) und Motion Blur.

use std::f64::consts::TAU;

use pgz_engine::prelude::*;
use serde_json::Value;

type V3 = (f64, f64, f64);
type Rgb = (i32, i32, i32);

const CAM_POS: V3 = (0.0, 1.6, 0.0);
const NEAR: f64 = 0.12;
const FOV_MUL: f64 = 1.04;
const DEG_PER_PX: f64 = 0.12;
const COOLDOWN: f64 = 0.18;
const FORGIVE_REL: f64 = 1.15;
const FORGIVE_ABS: f64 = 0.004;

fn pitch_clamp() -> f64 {
    py::radians(60.0)
}

#[derive(Clone, Copy)]
struct ModeCfg {
    duration: Option<f64>,
    targets: Option<i64>,
    simul: usize,
    radius: f64,
    paths: bool,
    speed: f64,
    yaw: f64,
    pmin: f64,
    pmax: f64,
    dmin: f64,
    dmax: f64,
    respawn: (f64, f64),
    relative: bool,
}

const MODE_KEYS: [&str; 4] = ["precision", "reflex", "moving", "chill"];

fn mode_cfg(mode: &str) -> ModeCfg {
    match mode {
        "reflex" => ModeCfg {
            duration: None,
            targets: Some(30),
            simul: 1,
            radius: 0.50,
            paths: false,
            speed: 1.0,
            yaw: 60.0,
            pmin: -15.0,
            pmax: 30.0,
            dmin: 6.0,
            dmax: 14.0,
            respawn: (0.4, 0.9),
            relative: true,
        },
        "moving" => ModeCfg {
            duration: Some(60.0),
            targets: None,
            simul: 2,
            radius: 0.50,
            paths: true,
            speed: 1.0,
            yaw: 70.0,
            pmin: -5.0,
            pmax: 30.0,
            dmin: 7.0,
            dmax: 16.0,
            respawn: (0.3, 0.3),
            relative: false,
        },
        "chill" => ModeCfg {
            duration: None,
            targets: None,
            simul: 3,
            radius: 0.55,
            paths: true,
            speed: 0.6,
            yaw: 90.0,
            pmin: -10.0,
            pmax: 35.0,
            dmin: 6.0,
            dmax: 18.0,
            respawn: (0.3, 0.3),
            relative: false,
        },
        _ => ModeCfg {
            duration: Some(60.0),
            targets: None,
            simul: 3,
            radius: 0.45,
            paths: false,
            speed: 1.0,
            yaw: 70.0,
            pmin: -10.0,
            pmax: 35.0,
            dmin: 8.0,
            dmax: 20.0,
            respawn: (0.0, 0.0),
            relative: false,
        },
    }
}

struct Theme {
    sky: (Rgb, Rgb),
    fog: Rgb,
    fog_start: f64,
    fog_end: f64,
    body: Rgb,
    ring: Rgb,
    dot: Rgb,
    glow: Option<Rgb>,
    accent: Rgb,
}

const THEME_KEYS: [&str; 3] = ["space", "neon", "range"];
const THEMES: [Theme; 3] = [
    Theme {
        sky: ((4, 5, 12), (10, 8, 24)),
        fog: (10, 10, 22),
        fog_start: 26.0,
        fog_end: 60.0,
        body: (120, 200, 255),
        ring: (225, 240, 255),
        dot: (255, 255, 255),
        glow: Some((90, 160, 255)),
        accent: (240, 245, 255),
    },
    Theme {
        sky: ((18, 8, 31), (58, 22, 80)),
        fog: (58, 22, 80),
        fog_start: 10.0,
        fog_end: 30.0,
        body: (0, 240, 200),
        ring: (255, 255, 255),
        dot: (20, 20, 40),
        glow: Some((0, 255, 220)),
        accent: (255, 230, 120),
    },
    Theme {
        sky: ((30, 32, 36), (30, 32, 36)),
        fog: (30, 32, 36),
        fog_start: 14.0,
        fog_end: 40.0,
        body: (230, 70, 70),
        ring: (240, 240, 240),
        dot: (230, 70, 70),
        glow: None,
        accent: (250, 250, 250),
    },
];

/// `_ease_out_back` - kleiner Ueberschwinger am Ende.
fn ease_out_back(x: f64) -> f64 {
    let (c1, c3) = (1.70158, 2.70158);
    let x = 0f64.max(1f64.min(x));
    1.0 + c3 * py::pow(x - 1.0, 3.0) + c1 * py::pow(x - 1.0, 2.0)
}

fn dir_from(yaw: f64, pitch: f64) -> V3 {
    let cp = pitch.cos();
    (yaw.sin() * cp, pitch.sin(), yaw.cos() * cp)
}

fn norm_dir(p: V3) -> V3 {
    let d = (p.0 - CAM_POS.0, p.1 - CAM_POS.1, p.2 - CAM_POS.2);
    let mut n = (py::pow(d.0, 2.0) + py::pow(d.1, 2.0) + py::pow(d.2, 2.0)).sqrt();
    if n == 0.0 {
        n = 1.0;
    }
    (d.0 / n, d.1 / n, d.2 / n)
}

fn clip_near(pts: &[V3]) -> Vec<V3> {
    let mut out = Vec::new();
    let n = pts.len();
    for i in 0..n {
        let (a, b) = (pts[i], pts[(i + 1) % n]);
        let (da, db) = (a.2 - NEAR, b.2 - NEAR);
        if da >= 0.0 {
            out.push(a);
        }
        if (da >= 0.0) != (db >= 0.0) {
            let tt = da / (da - db);
            out.push((a.0 + (b.0 - a.0) * tt, a.1 + (b.1 - a.1) * tt, NEAR));
        }
    }
    out
}

fn clip_seg(a: V3, b: V3) -> Option<(V3, V3)> {
    let (da, db) = (a.2 - NEAR, b.2 - NEAR);
    if da < 0.0 && db < 0.0 {
        return None;
    }
    if da < 0.0 || db < 0.0 {
        let tt = da / (da - db);
        let m = (a.0 + (b.0 - a.0) * tt, a.1 + (b.1 - a.1) * tt, NEAR);
        return Some(if da < 0.0 { (m, b) } else { (a, m) });
    }
    Some((a, b))
}

#[derive(Clone, Copy, PartialEq)]
enum TState {
    Grow,
    Alive,
    Shrink,
}

#[derive(Clone, Copy)]
enum Path {
    Strafe { amp: f64, om: f64, ax: V3 },
    Orbit { rad: f64, om: f64 },
    Bob { amp: f64, om: f64, drift: f64 },
}

struct Target {
    base: V3,
    radius: f64,
    state: TState,
    age: f64,
    life: Option<f64>,
    path: Option<Path>,
    phase: f64,
    spawn_t: f64,
}

struct Particle {
    pos: V3,
    v: V3,
    age: f64,
    life: f64,
    col: Rgb,
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Setup,
    Play,
}

/// `AimTrainerGame`
pub struct AimTrainerGame {
    b: GameBase,
    cfg: ModeCfg,
    theme_idx: usize,
    sens: f64,
    blur: f64,
    prev_frame: Option<Surface>,
    small: Font,
    tiny: Font,
    big: Font,
    huge: Font,
    yaw: f64,
    pitch: f64,
    kick: f64,
    anim_t: f64,
    go_last: Option<f64>,
    capture: bool,
    sky_cache: Option<((i32, i32, usize), Surface)>,
    bh_cache: Option<(i32, Surface, Surface)>,
    sun_cache: Option<(i32, Surface)>,
    stars: Vec<(V3, i32, f64)>,
    theme_rects: Vec<Rect>,
    sens_minus: Rect,
    sens_plus: Rect,
    sens_box: Rect,
    blur_minus: Rect,
    blur_plus: Rect,
    blur_box: Rect,
    start_rect: Rect,
    state: State,
    play_t: f64,
    time_left: Option<f64>,
    targets: Vec<Target>,
    pending: Vec<f64>,
    spawned: i64,
    hits: i64,
    shots: i64,
    misses: i64,
    combo: i64,
    max_combo: i64,
    last_hit_t: f64,
    reactions: Vec<i64>,
    last_reaction: Option<i64>,
    last_reaction_t: f64,
    cooldown: f64,
    flash: f64,
    tracer: f64,
    hitmark: f64,
    toast: Option<String>,
    toast_t: f64,
    particles: Vec<Particle>,
    popups: Vec<(String, V3, f64)>,
    acc_bonus: i64,
    basis: (V3, V3, V3),
    scx: f64,
    scy: f64,
    f: f64,
}

impl AimTrainerGame {
    /// `AimTrainerGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let fo = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let o = (0.0, 0.0, 0.0);
        let mut g = AimTrainerGame {
            b,
            cfg: mode_cfg("precision"),
            theme_idx: 0,
            sens: 1.0,
            blur: 0.0,
            prev_frame: None,
            small: fo.clone(),
            tiny: fo.clone(),
            big: fo.clone(),
            huge: fo,
            yaw: 0.0,
            pitch: 0.0,
            kick: 0.0,
            anim_t: 0.0,
            go_last: None,
            capture: false,
            sky_cache: None,
            bh_cache: None,
            sun_cache: None,
            stars: Vec::new(),
            theme_rects: Vec::new(),
            sens_minus: z,
            sens_plus: z,
            sens_box: z,
            blur_minus: z,
            blur_plus: z,
            blur_box: z,
            start_rect: z,
            state: State::Setup,
            play_t: 0.0,
            time_left: None,
            targets: Vec::new(),
            pending: Vec::new(),
            spawned: 0,
            hits: 0,
            shots: 0,
            misses: 0,
            combo: 0,
            max_combo: 0,
            last_hit_t: 0.0,
            reactions: Vec::new(),
            last_reaction: None,
            last_reaction_t: -9.0,
            cooldown: 0.0,
            flash: 0.0,
            tracer: 0.0,
            hitmark: 0.0,
            toast: None,
            toast_t: 0.0,
            particles: Vec::new(),
            popups: Vec::new(),
            acc_bonus: 0,
            basis: (o, o, o),
            scx: 0.0,
            scy: 0.0,
            f: 1.0,
        };
        g.reset();
        Box::new(g)
    }

    fn mode(&self) -> &str {
        self.b.mode.as_str()
    }

    fn th(&self) -> &'static Theme {
        &THEMES[self.theme_idx]
    }

    fn make_fonts(&mut self) {
        let h = self.b.height;
        self.small = ui::f(13.max(h / 30));
        self.tiny = ui::f(11.max(h / 36));
        self.big = ui::fb(16.max(h / 21));
        self.huge = ui::fb(26.max(h / 11));
    }

    /// `_dim` - Vollbild-Abdunklung ueber eine undurchsichtige Flaeche.
    fn dim(&self, s: &mut Surface, c: Rgb, alpha: i32) {
        let mut surf = pg::surface(self.b.width, self.b.height);
        surf.fill_c(c);
        surf.set_alpha(Some(alpha.clamp(0, 255) as u8));
        s.bl(&surf, (0, 0));
    }

    fn aim_settings(&self) -> (usize, f64, f64) {
        let theme = match self.b.opt_value("aim", "theme") {
            Some(Value::String(st)) => THEME_KEYS.iter().position(|k| *k == st).unwrap_or(0),
            _ => 0,
        };
        let sens = match self.b.opt_float_py("aim", "sens", 1.0) {
            Some(v) => 0.5f64.max(2.0f64.min(v)),
            None => 1.0,
        };
        let blur = match self.b.opt_float_py("aim", "blur", 0.0) {
            Some(v) => 0.0f64.max(0.8f64.min(v)),
            None => 0.0,
        };
        (theme, sens, blur)
    }

    fn save_aim(&mut self, key: &str, value: Value) {
        self.b.set_opt("aim", key, value);
    }

    fn make_stars(n: usize) -> Vec<(V3, i32, f64)> {
        let mut rnd = py::Random::new(4021);
        let mut stars = Vec::with_capacity(n);
        for _ in 0..n {
            let z = rnd.uniform(-1.0, 1.0);
            let az = rnd.uniform(0.0, TAU);
            let xy = 0f64.max(1.0 - z * z).sqrt();
            let size = rnd.choice(&[1, 1, 2]);
            let ph = rnd.uniform(0.0, TAU);
            stars.push(((az.cos() * xy, z, az.sin() * xy), size, ph));
        }
        stars
    }

    // ===== Setup-Screen

    fn build_setup_layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let bw = 120.min(py::fdiv32(w - 80, 3) - 10);
        let total = 3 * bw + 2 * 12;
        let y0 = (h as f64 * 0.28) as i32;
        self.theme_rects = (0..3).map(|i| Rect::new(cx - py::fdiv32(total, 2) + i * (bw + 12), y0, bw, 44)).collect();
        let rx = cx + 24;
        let sy = y0 + 58;
        self.sens_minus = Rect::new(rx - 60, sy, 44, 40);
        self.sens_plus = Rect::new(rx + 116, sy, 44, 40);
        self.sens_box = Rect::new(rx - 8, sy, 116, 40);
        let by = sy + 52;
        self.blur_minus = Rect::new(rx - 60, by, 44, 40);
        self.blur_plus = Rect::new(rx + 116, by, 44, 40);
        self.blur_box = Rect::new(rx - 8, by, 116, 40);
        self.start_rect = Rect::new(cx - 95, by + 54, 190, 46);
    }

    fn handle_setup(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown {
            match ev.key.as_str() {
                k @ ("1" | "2" | "3") => self.set_theme(k.parse::<usize>().unwrap_or(1) - 1),
                "Left" | "a" | "A" => self.set_theme((self.theme_idx + 2) % 3),
                "Right" | "d" | "D" => self.set_theme((self.theme_idx + 1) % 3),
                "plus" | "equal" | "KP_Add" => self.change_sens(0.1),
                "minus" | "KP_Subtract" => self.change_sens(-0.1),
                "b" | "B" => self.cycle_blur(),
                "Return" | "space" => self.start_run(),
                _ => {}
            }
        } else if ev.kind == Kind::MouseDown {
            if let Some(i) = self.theme_rects.iter().position(|r| r.hit(ev.pos)) {
                self.set_theme(i);
                return;
            }
            if self.sens_minus.hit(ev.pos) {
                self.change_sens(-0.1);
            } else if self.sens_plus.hit(ev.pos) {
                self.change_sens(0.1);
            } else if self.blur_minus.hit(ev.pos) {
                self.change_blur(-0.1);
            } else if self.blur_plus.hit(ev.pos) {
                self.change_blur(0.1);
            } else if self.start_rect.hit(ev.pos) {
                self.start_run();
            }
        }
    }

    fn set_theme(&mut self, idx: usize) {
        self.theme_idx = idx % 3;
        self.save_aim("theme", Value::from(THEME_KEYS[self.theme_idx]));
        self.sky_cache = None;
        self.b.play_sound("click");
    }

    fn change_sens(&mut self, delta: f64) {
        self.sens = py::round_to(0.5f64.max(2.0f64.min(self.sens + delta)), 1);
        self.save_aim("sens", Value::from(self.sens));
        self.toast = Some(tf("aim.sens_toast", &[("v", &format!("{:.1}", self.sens))]));
        self.toast_t = 1.0;
        self.b.play_sound("select");
    }

    fn change_blur(&mut self, delta: f64) {
        self.blur = py::round_to(0.0f64.max(0.8f64.min(self.blur + delta)), 1);
        self.save_aim("blur", Value::from(self.blur));
        if self.blur <= 0.0 {
            self.prev_frame = None;
        }
        self.b.play_sound("select");
    }

    fn cycle_blur(&mut self) {
        self.blur = if self.blur >= 0.79 { 0.0 } else { py::round_to(self.blur + 0.1, 1) };
        self.save_aim("blur", Value::from(self.blur));
        if self.blur <= 0.0 {
            self.prev_frame = None;
        }
        self.b.play_sound("select");
    }

    fn blur_label(&self) -> String {
        if self.blur <= 0.0 {
            t("common.off")
        } else {
            format!("{}%", (self.blur * 100.0) as i64)
        }
    }

    fn apply_blur(&mut self, s: &mut Surface) {
        if self.blur <= 0.01 {
            self.prev_frame = None;
            return;
        }
        match self.prev_frame.as_mut() {
            Some(prev) if prev.w() == s.w() && prev.h() == s.h() => {
                prev.set_alpha(Some((self.blur * 230.0) as u8));
                s.bl(prev, (0, 0));
                self.prev_frame = Some(s.copy());
            }
            _ => self.prev_frame = Some(s.copy()),
        }
    }

    // ===== Lauf starten/beenden

    fn start_run(&mut self) {
        self.b.score = 0;
        self.b.game_over = false;
        self.play_t = 0.0;
        self.time_left = self.cfg.duration;
        self.targets.clear();
        self.pending.clear();
        self.spawned = 0;
        self.hits = 0;
        self.shots = 0;
        self.misses = 0;
        self.combo = 0;
        self.max_combo = 0;
        self.last_hit_t = 0.0;
        self.reactions.clear();
        self.last_reaction = None;
        self.last_reaction_t = -9.0;
        self.cooldown = 0.0;
        self.flash = 0.0;
        self.tracer = 0.0;
        self.hitmark = 0.0;
        self.toast = None;
        self.toast_t = 0.0;
        self.particles.clear();
        self.popups.clear();
        self.go_last = None;
        for _ in 0..self.cfg.simul {
            self.spawn_target();
        }
        self.state = State::Play;
        self.capture = true;
        self.b.play_sound("level");
    }

    fn finish_run(&mut self) {
        if self.mode() == "precision" && self.shots > 0 {
            let bonus = (py::pow(self.hits as f64 / self.shots as f64, 2.0) * 1500.0) as i64;
            self.acc_bonus = bonus;
            self.b.score += bonus;
            if bonus != 0 {
                self.b.play_sound("point");
            }
        } else {
            self.acc_bonus = 0;
        }
        if self.shots >= 20 && self.hits == self.shots {
            self.b.ach_event("aim_perfect", None);
        }
        self.b.game_over = true;
        self.capture = false;
        self.b.play_sound("win");
    }

    // ===== Targets

    fn spawn_target(&mut self) {
        let cfg = self.cfg;
        if let Some(n) = cfg.targets {
            if self.spawned >= n {
                return;
            }
        }
        let base_yaw = if cfg.relative { self.yaw } else { 0.0 };
        let mut pos = CAM_POS;
        let mut yw = 0.0;
        for _ in 0..20 {
            yw = base_yaw + py::radians(py::uniform(-cfg.yaw, cfg.yaw));
            let pt = py::radians(py::uniform(cfg.pmin, cfg.pmax));
            let dist = py::uniform(cfg.dmin, cfg.dmax);
            let d = dir_from(yw, pt);
            pos = (CAM_POS.0 + d.0 * dist, CAM_POS.1 + d.1 * dist, CAM_POS.2 + d.2 * dist);
            if self.targets.iter().all(|o| self.angle_between(pos, o) > py::radians(15.0)) {
                break;
            }
        }
        let mut path = None;
        if cfg.paths {
            let sp = cfg.speed;
            let kind = py::weighted_index(&[0.45, 0.30, 0.25]);
            path = Some(match kind {
                0 => {
                    let amp = py::uniform(2.5, 5.0);
                    let om = py::uniform(1.5, 3.5) / amp * sp;
                    Path::Strafe { amp, om, ax: (yw.cos(), 0.0, -yw.sin()) }
                }
                1 => {
                    let rad = py::uniform(1.5, 3.0);
                    let om = py::uniform(0.5, 1.2) * sp;
                    Path::Orbit { rad, om }
                }
                _ => {
                    let amp = py::uniform(0.8, 1.6);
                    let om = py::uniform(1.0, 2.0) * sp;
                    let drift = py::uniform(-0.4, 0.4) * sp;
                    Path::Bob { amp, om, drift }
                }
            });
        }
        let mut life = None;
        if self.mode() == "reflex" {
            let i = self.spawned.min(29);
            life = Some(2.0 - 0.8 * i as f64 / 29.0);
        }
        let phase = py::uniform(0.0, TAU);
        self.targets.push(Target { base: pos, radius: cfg.radius, state: TState::Grow, age: 0.0, life, path, phase, spawn_t: self.play_t });
        self.spawned += 1;
    }

    fn angle_between(&self, pos_a: V3, target_b: &Target) -> f64 {
        let pb = self.target_pos(target_b);
        let va = norm_dir(pos_a);
        let vb = norm_dir(pb);
        let d = (-1f64).max(1f64.min(va.0 * vb.0 + va.1 * vb.1 + va.2 * vb.2));
        d.acos()
    }

    fn target_pos(&self, tg: &Target) -> V3 {
        let b = tg.base;
        let Some(p) = tg.path else {
            return b;
        };
        let tt = self.play_t + tg.phase;
        match p {
            Path::Strafe { amp, om, ax } => {
                let off = amp * (om * tt).sin();
                (b.0 + ax.0 * off, b.1, b.2 + ax.2 * off)
            }
            Path::Orbit { rad, om } => (b.0 + rad * (om * tt).cos(), b.1, b.2 + rad * (om * tt).sin()),
            Path::Bob { amp, om, drift } => {
                let dx = (-2.5f64).max(2.5f64.min(drift * (py::fpmod(tt, 12.0) - 6.0)));
                (b.0 + dx, b.1 + amp * (om * tt).sin(), b.2)
            }
        }
    }

    fn target_scale(tg: &Target) -> f64 {
        match tg.state {
            TState::Grow => ease_out_back(tg.age / 0.25),
            TState::Shrink => 0f64.max(1.0 - tg.age / 0.18),
            TState::Alive => {
                if let Some(life) = tg.life {
                    let frac = tg.age / life;
                    if frac > 0.8 {
                        return 0.3f64.max(1.0 - (frac - 0.8) / 0.2 * 0.7);
                    }
                }
                1.0
            }
        }
    }

    fn update_targets(&mut self, dt: f64) {
        let cfg = self.cfg;
        let alt = std::mem::take(&mut self.targets);
        let mut alive = Vec::new();
        for mut tg in alt {
            tg.age += dt;
            if tg.state == TState::Grow && tg.age >= 0.25 {
                tg.state = TState::Alive;
                tg.age = 0.25;
            }
            if tg.state == TState::Shrink {
                if tg.age >= 0.18 {
                    continue;
                }
            } else if let Some(life) = tg.life {
                if tg.age >= life {
                    self.misses += 1;
                    self.b.play_sound("move");
                    self.pending.push(py::uniform(cfg.respawn.0, cfg.respawn.1));
                    continue;
                }
            }
            alive.push(tg);
        }
        self.targets = alive;

        let alt = std::mem::take(&mut self.pending);
        let mut rest = Vec::new();
        for mut timer in alt {
            timer -= dt;
            if timer <= 0.0 {
                self.spawn_target();
            } else {
                rest.push(timer);
            }
        }
        self.pending = rest;

        if self.mode() == "reflex" && self.spawned >= cfg.targets.unwrap_or(0) && self.targets.is_empty() && self.pending.is_empty() && !self.b.game_over {
            self.finish_run();
        }
    }

    // ===== Kamera

    fn apply_look(&mut self, rel: (i32, i32)) {
        let k = py::radians(DEG_PER_PX) * self.sens;
        self.yaw = py::fpmod(self.yaw + rel.0 as f64 * k, TAU);
        self.pitch = (-pitch_clamp()).max(pitch_clamp().min(self.pitch - rel.1 as f64 * k));
    }

    fn view_basis(yaw: f64, pitch: f64, kick: f64) -> (V3, V3, V3) {
        let f = dir_from(yaw, pitch + py::radians(0.6) * kick);
        let mut rl = py::hypot(f.0, f.2);
        if rl == 0.0 {
            rl = 1.0;
        }
        let r = (f.2 / rl, 0.0, -f.0 / rl);
        let u = (f.1 * r.2 - f.2 * r.1, f.2 * r.0 - f.0 * r.2, f.0 * r.1 - f.1 * r.0);
        (r, u, f)
    }

    fn to_cam(&self, p: V3) -> V3 {
        let (r, u, f) = self.basis;
        let dx = p.0 - CAM_POS.0;
        let dy = p.1 - CAM_POS.1;
        let dz = p.2 - CAM_POS.2;
        (dx * r.0 + dy * r.1 + dz * r.2, dx * u.0 + dy * u.1 + dz * u.2, dx * f.0 + dy * f.1 + dz * f.2)
    }

    fn proj(&self, c: V3) -> (f64, f64) {
        let k = self.f / c.2;
        (self.scx + c.0 * k, self.scy - c.1 * k)
    }

    fn fog_color(&self, col: Rgb, depth: f64) -> Rgb {
        let th = self.th();
        if depth <= th.fog_start {
            return col;
        }
        let f = 1f64.min((depth - th.fog_start) / (th.fog_end - th.fog_start));
        let fog = th.fog;
        (
            (col.0 as f64 + (fog.0 - col.0) as f64 * f) as i32,
            (col.1 as f64 + (fog.1 - col.1) as f64 * f) as i32,
            (col.2 as f64 + (fog.2 - col.2) as f64 * f) as i32,
        )
    }

    /// Billboard-Projektion: (sx, sy, k, z) oder `None` hinter der Kamera.
    fn bill(&self, p: V3) -> Option<(f64, f64, f64, f64)> {
        let c = self.to_cam(p);
        if c.2 < NEAR {
            return None;
        }
        let k = self.f / c.2;
        Some((self.scx + c.0 * k, self.scy - c.1 * k, k, c.2))
    }

    // ===== Schiessen

    fn shoot(&mut self) {
        if self.cooldown > 0.0 || self.b.game_over {
            return;
        }
        self.cooldown = COOLDOWN;
        self.shots += 1;
        self.flash = 0.06;
        self.tracer = 0.08;
        self.kick = 1.0;
        self.b.play_sound("shoot");
        let f = dir_from(self.yaw, self.pitch);
        let mut best: Option<(f64, usize, V3, f64)> = None;
        for (i, tg) in self.targets.iter().enumerate() {
            if tg.state == TState::Shrink {
                continue;
            }
            if tg.state == TState::Grow && Self::target_scale(tg) < 0.5 {
                continue;
            }
            let pos = self.target_pos(tg);
            let d = norm_dir(pos);
            let dist = py::hypot_n(&[pos.0 - CAM_POS.0, pos.1 - CAM_POS.1, pos.2 - CAM_POS.2]);
            let ang = (-1f64).max(1f64.min(f.0 * d.0 + f.1 * d.1 + f.2 * d.2)).acos();
            let ar = ((tg.radius * Self::target_scale(tg)) / dist).atan();
            if ang <= ar * FORGIVE_REL + FORGIVE_ABS && best.is_none_or(|b| ang < b.0) {
                best = Some((ang, i, pos, dist));
            }
        }
        match best {
            Some((_, i, pos, dist)) => self.register_hit(i, pos, dist),
            None => self.register_miss(),
        }
    }

    fn register_hit(&mut self, ti: usize, pos: V3, dist: f64) {
        self.hits += 1;
        self.combo += 1;
        self.max_combo = self.max_combo.max(self.combo);
        self.last_hit_t = self.play_t;
        self.hitmark = 0.1;
        let mult = 1.0 + 0.25 * self.combo.min(12) as f64;
        let pts = match self.mode() {
            "precision" => 100 + ((dist - 8.0) * 6.0) as i64,
            "reflex" => {
                let reaction = ((self.play_t - self.targets[ti].spawn_t) * 1000.0) as i64;
                self.reactions.push(reaction);
                self.last_reaction = Some(reaction);
                self.last_reaction_t = self.play_t;
                50.max(1000 - reaction)
            }
            "moving" => {
                if self.combo == 5 || self.combo == 10 {
                    self.b.play_sound("point");
                } else if self.combo == 12 {
                    self.b.play_sound("level");
                }
                (80.0 * mult) as i64
            }
            _ => (50.0 * mult) as i64,
        };
        self.b.score += pts;
        self.b.play_sound("hit");
        self.b.rumble(60);
        self.targets[ti].state = TState::Shrink;
        self.targets[ti].age = 0.0;
        self.pending.push(py::uniform(self.cfg.respawn.0, self.cfg.respawn.1));
        let body = self.th().body;
        for _ in 0..10 {
            let a = py::uniform(0.0, TAU);
            let b = py::uniform(-1.0, 1.0);
            let sp = py::uniform(2.0, 5.0);
            let v = (a.cos() * sp, b * sp, a.sin() * sp);
            let life = py::uniform(0.3, 0.55);
            self.particles.push(Particle { pos, v, age: 0.0, life, col: body });
        }
        self.popups.push((format!("+{pts}"), pos, 0.0));
    }

    fn register_miss(&mut self) {
        self.misses += 1;
        if self.mode() == "moving" {
            self.combo = 0;
        }
    }

    // ===== Zeichnen: Hintergrund / Themes

    fn sky(&mut self) -> Surface {
        let key = (self.b.width, self.b.height, self.theme_idx);
        if let Some((k, s)) = &self.sky_cache {
            if *k == key {
                return s.clone();
            }
        }
        let (top, bottom) = self.th().sky;
        let (w, h) = (self.b.width, self.b.height);
        let mut surf = pg::surface(w, h);
        for y in 0..h {
            let f = y as f64 / 1.max(h - 1) as f64;
            let c = (
                (top.0 as f64 + (bottom.0 - top.0) as f64 * f) as i32,
                (top.1 as f64 + (bottom.1 - top.1) as f64 * f) as i32,
                (top.2 as f64 + (bottom.2 - top.2) as f64 * f) as i32,
            );
            surf.fill_r(c, Rect::new(0, y, w, 1));
        }
        self.sky_cache = Some((key, surf.clone()));
        surf
    }

    fn draw_background(&mut self, s: &mut Surface) {
        let sky = self.sky();
        s.bl(&sky, (0, 0));
        match self.theme_idx {
            0 => {
                self.draw_stars(s, 220, (200, 210, 235));
                self.draw_planet(s);
                self.draw_blackhole(s);
            }
            1 => {
                self.draw_stars(s, 120, (255, 200, 240));
                self.draw_neon_sun(s);
                self.draw_neon_grid(s);
            }
            _ => self.draw_range_hall(s),
        }
    }

    fn draw_stars(&self, s: &mut Surface, count: usize, tint: Rgb) {
        let (r, u, f) = self.basis;
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        for &(d, size, ph) in self.stars.iter().take(count) {
            let cz = d.0 * f.0 + d.1 * f.1 + d.2 * f.2;
            if cz < 0.05 {
                continue;
            }
            let cx = d.0 * r.0 + d.1 * r.1 + d.2 * r.2;
            let cy = d.0 * u.0 + d.1 * u.1 + d.2 * u.2;
            let k = self.f / cz;
            let sx = self.scx + cx * k;
            let sy = self.scy - cy * k;
            if 0.0 <= sx && sx < w && 0.0 <= sy && sy < h {
                let b = 0.55 + 0.45 * (self.anim_t * 1.7 + ph).sin();
                let c = (
                    (tint.0 as f64 * b * 0.6 + 60.0) as i32,
                    (tint.1 as f64 * b * 0.6 + 60.0) as i32,
                    (tint.2 as f64 * b * 0.6 + 60.0) as i32,
                );
                s.fill_r(c, Rect::new(sx as i32, sy as i32, size, size));
            }
        }
    }

    fn blackhole_sprite(&mut self) -> (Surface, Surface) {
        let big_r = 24.max((self.f * 3.2 / 45.0) as i32);
        if let Some((r, a, b)) = &self.bh_cache {
            if *r == big_r {
                return (a.clone(), b.clone());
            }
        }
        let rf = big_r as f64;
        let (w, h) = ((rf * 6.8) as i32, (rf * 3.4) as i32);
        let (cx, cy) = (w / 2, h / 2);
        let mut surf = pg::surface_alpha(w, h);
        for (rad, alpha) in [((rf * 1.6) as i32, 22), ((rf * 1.3) as i32, 36)] {
            draw::circle(&mut surf, rgba(255, 240, 220, alpha), (cx, cy), rad, 0);
        }
        for (grow, alpha) in [(1.15, 60), (1.0, 190)] {
            let band = Rect::new(0, 0, (rf * 3.4 * 2.0 * grow) as i32, (rf * 0.44 * 2.0 * grow) as i32).center_at((cx, cy));
            draw::ellipse(&mut surf, rgba(255, 236, 210, alpha), band, 0);
        }
        let inner = Rect::new(0, 0, (rf * 1.7 * 2.0) as i32, (rf * 0.30 * 2.0) as i32).center_at((cx, cy));
        draw::ellipse(&mut surf, rgba(0, 0, 0, 0), inner, 0);
        draw::circle(&mut surf, rgba(2, 2, 4, 255), (cx, cy), big_r, 0);
        let rs = (rf * 6.4) as i32;
        let mut ring = pg::surface_alpha(rs, rs);
        let rr = Rect::new(0, 0, (rf * 3.0 * 2.0) as i32, (rf * 0.9 * 2.0) as i32).center_at((ring.w() / 2, ring.h() / 2));
        draw::ellipse(&mut ring, rgba(255, 244, 224, 255), rr, 2.max(big_r / 22));
        let ring = transform::rotozoom(&ring, 18.0, 1.0);
        let mut shimmer = pg::surface_alpha(w, h);
        shimmer.blr(&ring, ring.get_rect().center_at((cx, cy)));
        draw::circle(&mut shimmer, rgba(255, 244, 224, 220), (cx, cy), (rf * 1.06) as i32, 1.max(big_r / 30));
        surf.bl(&shimmer, (0, 0));
        self.bh_cache = Some((big_r, surf.clone(), shimmer.clone()));
        (surf, shimmer)
    }

    fn draw_blackhole(&mut self, s: &mut Surface) {
        let d = dir_from(py::radians(40.0), py::radians(8.0));
        let pos = (CAM_POS.0 + d.0 * 45.0, CAM_POS.1 + d.1 * 45.0, CAM_POS.2 + d.2 * 45.0);
        let Some(b) = self.bill(pos) else {
            return;
        };
        let (sprite, mut shimmer) = self.blackhole_sprite();
        let rect = sprite.get_rect().center_at((b.0 as i32, b.1 as i32));
        if rect.right() < 0 || rect.x > self.b.width || rect.bottom() < 0 || rect.y > self.b.height {
            return;
        }
        s.blr(&sprite, rect);
        let a = (35.0 + 30.0 * (self.anim_t * 0.8).sin()) as i32;
        shimmer.set_alpha(Some(a.clamp(0, 255) as u8));
        s.bl_flags(&shimmer, (rect.x, rect.y), BLEND_RGBA_ADD);
        shimmer.set_alpha(Some(255));
    }

    fn draw_planet(&self, s: &mut Surface) {
        let d = dir_from(py::radians(-70.0), py::radians(4.0));
        let pos = (CAM_POS.0 + d.0 * 40.0, CAM_POS.1 + d.1 * 40.0, CAM_POS.2 + d.2 * 40.0);
        let Some(b) = self.bill(pos) else {
            return;
        };
        let r = 8.max((self.f * 0.6 / 40.0) as i32);
        let (x, y) = (b.0 as i32, b.1 as i32);
        draw::circle(s, rgb(30, 26, 40), (x, y), r, 0);
        draw::circle(s, rgb(180, 150, 220), (x - py::fdiv32(r, 4), y - py::fdiv32(r, 4)), r, 2);
        draw::circle(s, rgb(52, 44, 66), (x + py::fdiv32(r, 5), y + py::fdiv32(r, 5)), (r as f64 * 0.82) as i32, 0);
    }

    fn draw_neon_sun(&mut self, s: &mut Surface) {
        let d = dir_from(0.0, py::radians(10.0));
        let pos = (CAM_POS.0 + d.0 * 50.0, CAM_POS.1 + d.1 * 50.0, CAM_POS.2 + d.2 * 50.0);
        let Some(b) = self.bill(pos) else {
            return;
        };
        let r = 30.max((self.f * 7.0 / 50.0) as i32);
        if self.sun_cache.as_ref().is_none_or(|c| c.0 != r) {
            let mut surf = pg::surface_alpha(2 * r, 2 * r);
            for yy in 0..2 * r {
                let f = yy as f64 / (2 * r - 1) as f64;
                let col = rgba(255, (120.0 + 70.0 * f) as i32, (180.0 - 90.0 * f) as i32, 255);
                draw::line(&mut surf, col, (0, yy), (2 * r, yy), 1);
            }
            let mut mask = pg::surface_alpha(2 * r, 2 * r);
            draw::circle(&mut mask, rgba(255, 255, 255, 255), (r, r), r, 0);
            surf.bl_flags(&mask, (0, 0), BLEND_RGBA_MIN);
            for i in 0..4 {
                let yy = r + (r as f64 * (0.15 + i as f64 * 0.22)) as i32;
                let hh = 2.max((r as f64 * 0.06 + (i * 2) as f64) as i32);
                draw::rect(&mut surf, rgba(0, 0, 0, 0), Rect::new(0, yy, 2 * r, hh), 0);
            }
            self.sun_cache = Some((r, surf));
        }
        if let Some((_, surf)) = &self.sun_cache {
            s.blr(surf, surf.get_rect().center_at((b.0 as i32, b.1 as i32)));
        }
    }

    fn draw_neon_grid(&self, s: &mut Surface) {
        let col_main = (255, 60, 200);
        let col_hi = (255, 120, 220);
        for (i, gz) in (-40..42).step_by(2).enumerate() {
            let gz = gz as f64;
            if let Some(seg) = clip_seg(self.to_cam((-40.0, 0.0, gz)), self.to_cam((40.0, 0.0, gz))) {
                self.grid_line(s, seg, if i % 5 == 0 { col_hi } else { col_main });
            }
        }
        for (i, gx) in (-40..42).step_by(2).enumerate() {
            let gx = gx as f64;
            if let Some(seg) = clip_seg(self.to_cam((gx, 0.0, -40.0)), self.to_cam((gx, 0.0, 40.0))) {
                self.grid_line(s, seg, if i % 5 == 0 { col_hi } else { col_main });
            }
        }
    }

    fn grid_line(&self, s: &mut Surface, seg: (V3, V3), col: Rgb) {
        let (a, b) = seg;
        let depth = (a.2 + b.2) / 2.0;
        if depth > self.th().fog_end {
            return;
        }
        let (pa, pb) = (self.proj(a), self.proj(b));
        let w = self.b.width as f64;
        if (pa.0 < -w && pb.0 < -w) || (pa.0 > 2.0 * w && pb.0 > 2.0 * w) {
            return;
        }
        draw::line(s, self.fog_color(col, depth), (pa.0 as i32, pa.1 as i32), (pb.0 as i32, pb.1 as i32), 1);
    }

    fn add_quad(&self, items: &mut Vec<(f64, Vec<(f64, f64)>, Rgb)>, pts: [V3; 4], col: Rgb) {
        let cam: Vec<V3> = pts.iter().map(|p| self.to_cam(*p)).collect();
        if cam.iter().all(|c| c.2 <= NEAR) {
            return;
        }
        let cam = clip_near(&cam);
        if cam.len() < 3 {
            return;
        }
        let zs: Vec<f64> = cam.iter().map(|c| c.2).collect();
        let depth = py::sum(&zs) / cam.len() as f64;
        let proj: Vec<(f64, f64)> = cam.iter().map(|c| self.proj(*c)).collect();
        items.push((depth, proj, self.fog_color(col, depth)));
    }

    fn draw_range_hall(&self, s: &mut Surface) {
        let mut items = Vec::new();
        let (l, ht) = (22.0f64, 6.0f64);
        for i in 0..4 {
            let x0 = -l + i as f64 * (l / 2.0);
            self.add_quad(&mut items, [(x0, 0.0, -l), (x0 + l / 2.0, 0.0, -l), (x0 + l / 2.0, 0.0, 0.0), (x0, 0.0, 0.0)], (40, 42, 46));
            self.add_quad(&mut items, [(x0, 0.0, 0.0), (x0 + l / 2.0, 0.0, 0.0), (x0 + l / 2.0, 0.0, l), (x0, 0.0, l)], (43, 45, 49));
            self.add_quad(&mut items, [(x0, ht, -l), (x0 + l / 2.0, ht, -l), (x0 + l / 2.0, ht, l), (x0, ht, l)], (34, 36, 42));
        }
        for i in 0..4 {
            let z0 = -l + i as f64 * (l / 2.0);
            let shade = if i % 2 == 0 { 0.9 } else { 0.75 };
            let col = ((52.0 * shade) as i32, (54.0 * shade) as i32, (60.0 * shade) as i32);
            self.add_quad(&mut items, [(-l, 0.0, z0), (-l, 0.0, z0 + l / 2.0), (-l, ht, z0 + l / 2.0), (-l, ht, z0)], col);
            self.add_quad(&mut items, [(l, 0.0, z0), (l, 0.0, z0 + l / 2.0), (l, ht, z0 + l / 2.0), (l, ht, z0)], col);
            let x0 = -l + i as f64 * (l / 2.0);
            self.add_quad(&mut items, [(x0, 0.0, -l), (x0 + l / 2.0, 0.0, -l), (x0 + l / 2.0, ht, -l), (x0, ht, -l)], col);
            self.add_quad(&mut items, [(x0, 0.0, l), (x0 + l / 2.0, 0.0, l), (x0 + l / 2.0, ht, l), (x0, ht, l)], col);
        }
        items.sort_by(|a, b| (-a.0).partial_cmp(&-b.0).unwrap_or(std::cmp::Ordering::Equal));
        for (_d, pts, col) in &items {
            let ip: Vec<(i32, i32)> = pts.iter().map(|p| (p.0 as i32, p.1 as i32)).collect();
            draw::polygon(s, *col, &ip, 0);
        }
        for gx in (-10..11).step_by(5) {
            let x = gx as f64 / 2.0;
            if let Some(seg) = clip_seg(self.to_cam((x, 0.01, -l)), self.to_cam((x, 0.01, l))) {
                self.grid_line(s, seg, (66, 70, 78));
            }
        }
        for lx in [-8.0, 0.0, 8.0] {
            for lz in [-16.0, -6.0, 6.0, 16.0] {
                let Some(b) = self.bill((lx, ht - 0.1, lz)) else {
                    continue;
                };
                let wpx = 2.max((b.2 * 0.5) as i32);
                draw::rect(s, rgb(235, 235, 215), Rect::new(b.0 as i32 - wpx, b.1 as i32 - py::fdiv32(wpx, 3), 2 * wpx, 2.max(py::fdiv32(wpx, 2))), 0);
            }
        }
    }

    // ===== Targets & Effekte

    fn draw_targets(&self, s: &mut Surface) {
        let th = self.th();
        let mut drawlist = Vec::new();
        for tg in &self.targets {
            let pos = self.target_pos(tg);
            let Some(b) = self.bill(pos) else {
                continue;
            };
            let scale = Self::target_scale(tg);
            let pulse = 1.0 + 0.04 * (self.anim_t * 2.0 + tg.phase).sin();
            let rpx = b.2 * tg.radius * scale * pulse;
            if rpx < 1.0 {
                continue;
            }
            drawlist.push((b.3, b.0, b.1, rpx));
        }
        drawlist.sort_by(|a, b| (-a.0).partial_cmp(&-b.0).unwrap_or(std::cmp::Ordering::Equal));
        for (depth, sx, sy, rpx) in drawlist {
            let (x, y, r) = (sx as i32, sy as i32, rpx as i32);
            if x < -r || x > self.b.width + r || y < -r || y > self.b.height + r {
                continue;
            }
            if let Some(gl) = th.glow {
                let mut glow = pg::surface_alpha(3 * r + 2, 3 * r + 2);
                let c = (glow.w() / 2, glow.h() / 2);
                draw::circle(&mut glow, rgba(gl.0, gl.1, gl.2, 60), c, (r as f64 * 1.5) as i32, 0);
                let gr = glow.get_rect().center_at((x, y));
                s.bl_flags(&glow, (gr.x, gr.y), BLEND_RGBA_ADD);
            }
            let body = self.fog_color(th.body, depth);
            let ring = self.fog_color(th.ring, depth);
            let dot = self.fog_color(th.dot, depth);
            draw::circle(s, body, (x, y), r, 0);
            if r >= 5 {
                draw::circle(s, ring, (x, y), (r as f64 * 0.65) as i32, 2.max(r / 5));
            }
            draw::circle(s, dot, (x, y), 1.max((r as f64 * 0.28) as i32), 0);
        }
    }

    fn draw_particles(&self, s: &mut Surface) {
        for p in &self.particles {
            let Some(b) = self.bill(p.pos) else {
                continue;
            };
            let f = 1.0 - p.age / p.life;
            let r = 1.max((b.2 * 0.06 * f) as i32);
            draw::circle(s, self.fog_color(p.col, b.3), (b.0 as i32, b.1 as i32), r, 0);
        }
    }

    fn draw_popups(&self, s: &mut Surface) {
        for (txt, pos, age) in &self.popups {
            let Some(b) = self.bill(*pos) else {
                continue;
            };
            let mut img = self.small.render(txt, true, self.th().accent);
            img.set_alpha(Some((255.0 * (1.0 - age / 0.9)) as i32 as u8));
            s.blr(&img, img.get_rect().center_at((b.0 as i32, (b.1 - age * 46.0) as i32)));
        }
    }

    fn draw_tracer_flash(&self, s: &mut Surface) {
        let (gx, gy) = ((self.b.width as f64 * 0.54) as i32, (self.b.height as f64 * 0.98) as i32);
        let (cx, cy) = (self.scx as i32, self.scy as i32);
        let ac = self.th().accent;
        if self.tracer > 0.0 {
            let f = self.tracer / 0.08;
            let dim = ((ac.0 as f64 * 0.45 * f) as i32, (ac.1 as f64 * 0.45 * f) as i32, (ac.2 as f64 * 0.45 * f) as i32);
            let bright = ((ac.0 as f64 * f) as i32, (ac.1 as f64 * f) as i32, (ac.2 as f64 * f) as i32);
            draw::line(s, dim, (gx, gy), (cx, cy), 3);
            draw::line(s, bright, (gx, gy), (cx, cy), 1);
        }
        if self.flash > 0.0 {
            let r = 12.0;
            draw::circle(s, rgb(255, 240, 200), (gx, gy - 4), 6, 0);
            for a in [0.0, 90.0, 45.0, 135.0] {
                let rad = py::radians(a);
                let (gxf, gyf) = (gx as f64, gy as f64);
                draw::line(
                    s,
                    rgb(255, 220, 160),
                    (gxf - rad.cos() * r, gyf - 4.0 - rad.sin() * r),
                    (gxf + rad.cos() * r, gyf - 4.0 + rad.sin() * r),
                    2,
                );
            }
        }
    }

    fn draw_crosshair(&self, s: &mut Surface) {
        let (cx, cy) = (self.scx as i32, self.scy as i32);
        let col = if self.hitmark > 0.0 { (255, 255, 255) } else { self.th().accent };
        let ext = (4.0 * self.kick) as i32;
        let (gap, arm) = (5, 9 + ext);
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            draw::line(s, col, (cx + dx * gap, cy + dy * gap), (cx + dx * (gap + arm), cy + dy * (gap + arm)), 2);
        }
        draw::circle(s, col, (cx, cy), 2, 0);
        let mut ring = pg::surface_alpha(30, 30);
        draw::circle(&mut ring, rgba(col.0, col.1, col.2, 120), (15, 15), 14, 1);
        s.bl(&ring, (cx - 15, cy - 15));
    }

    // ===== HUD / Ergebnis

    fn fmt_mmss(sec: f64) -> String {
        let sec = sec as i64;
        format!("{:02}:{:02}", py::fdiv(sec, 60), py::pmod(sec, 60))
    }

    fn acc(&self) -> i64 {
        if self.shots != 0 {
            (self.hits as f64 / self.shots as f64 * 100.0) as i64
        } else {
            100
        }
    }

    fn mult_of(c: i64) -> f64 {
        1.0 + 0.25 * c.min(12) as f64
    }

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        let col = self.th().accent;
        let img = self.big.render(&tf("common.points", &[("score", &self.b.score.to_string())]), true, col);
        s.bl(&img, (14, 10));
        let acc = self.acc();
        let tl = || format!("{:.1}", 0f64.max(self.time_left.unwrap_or(0.0)));
        let mut right: Vec<String> = match self.mode() {
            "precision" => vec![tl(), format!("{}/{} · {}%", self.hits, self.shots, acc)],
            "reflex" => {
                let mut v = vec![tf("aim.targets", &[("n", &self.spawned.min(30).to_string()), ("m", "30")])];
                if !self.reactions.is_empty() {
                    let avg = py::fdiv(self.reactions.iter().sum::<i64>(), self.reactions.len() as i64);
                    v.push(tf("aim.reaction_avg", &[("ms", &avg.to_string())]));
                }
                v
            }
            "moving" => vec![tl(), tf("aim.combo", &[("m", &format!("{:.2}", Self::mult_of(self.combo)))]), format!("{}/{}", self.hits, self.shots)],
            _ => vec![tf("aim.hits", &[("n", &self.hits.to_string())]), tf("aim.session_time", &[("t", &Self::fmt_mmss(self.play_t))])],
        };
        if !MODE_KEYS[..3].contains(&self.mode()) && self.combo > 1 {
            right.push(tf("aim.combo", &[("m", &format!("{:.2}", Self::mult_of(self.combo)))]));
        }
        let mut y = 12;
        for line in &right {
            let img = self.small.render(line, true, col);
            s.blr(&img, img.get_rect().topright_at((w - 14, y)));
            y += 22;
        }
        if self.mode() == "reflex" {
            if let Some(ms) = self.last_reaction {
                if self.play_t - self.last_reaction_t < 0.8 {
                    let c = if ms < 400 {
                        p.green
                    } else if ms < 700 {
                        p.text
                    } else {
                        p.gold
                    };
                    let img = self.big.render(&format!("{ms} ms"), true, c);
                    s.blr(&img, img.get_rect().center_at((w / 2, (self.b.height as f64 * 0.72) as i32)));
                }
            }
        }
        if self.mode() == "chill" && !self.b.game_over {
            let img = self.tiny.render(&t("aim.end_hint"), true, col);
            s.blr(&img, img.get_rect().midbottom_at((w / 2, self.b.height - 6)));
        }
        if let Some(toast) = &self.toast {
            if !toast.is_empty() && self.toast_t > 0.0 {
                let img = self.small.render(toast, true, p.text);
                s.blr(&img, img.get_rect().midtop_at((w / 2, 12)));
            }
        }
    }

    fn draw_result(&self, s: &mut Surface) {
        let p = ui::pal();
        self.dim(s, (0, 0, 0), 140);
        let cx = self.b.width / 2;
        let cy = self.b.height / 2 - 40;
        let acc = self.acc();
        let head = self.huge.render(&t("aim.result_title"), true, self.th().accent);
        let mut lines = vec![tf("common.points", &[("score", &self.b.score.to_string())])];
        let max_combo = py::fstr(Self::mult_of(self.max_combo));
        match self.mode() {
            "precision" => {
                lines.push(format!("{}   ·   {}", tf("aim.hits", &[("n", &self.hits.to_string())]), tf("aim.misses", &[("n", &self.misses.to_string())])));
                lines.push(tf("aim.accuracy", &[("p", &acc.to_string())]));
                lines.push(tf("aim.bonus_accuracy", &[("n", &self.acc_bonus.to_string())]));
            }
            "reflex" => {
                let (avg, best) = if self.reactions.is_empty() {
                    (0, 0)
                } else {
                    (py::fdiv(self.reactions.iter().sum::<i64>(), self.reactions.len() as i64), *self.reactions.iter().min().unwrap_or(&0))
                };
                lines.push(format!("{} / 30", tf("aim.hits", &[("n", &self.hits.to_string())])));
                lines.push(tf("aim.reaction_avg", &[("ms", &avg.to_string())]));
                lines.push(tf("aim.reaction_best", &[("ms", &best.to_string())]));
            }
            "moving" => {
                lines.push(tf("aim.accuracy", &[("p", &acc.to_string())]));
                lines.push(tf("aim.max_combo", &[("m", &max_combo)]));
            }
            _ => {
                lines.push(tf("aim.hits", &[("n", &self.hits.to_string())]));
                lines.push(tf("aim.max_combo", &[("m", &max_combo)]));
                lines.push(tf("aim.session_time", &[("t", &Self::fmt_mmss(self.play_t))]));
            }
        }
        let imgs: Vec<Surface> = lines.iter().map(|l| self.b.font.render(l, true, p.text)).collect();
        let hint = self.small.render(&t("common.enter_restart"), true, p.text_dim);
        let top = cy - 70 - head.h() / 2 - 22;
        let bottom = cy - 24 + 32 * imgs.len() as i32 + 10 + hint.h() / 2 + 22;
        let widest = imgs.iter().map(|i| i.w()).max().unwrap_or(0);
        let pw = (self.b.width - 40).min(400.max(head.w() + 80).max(hint.w() + 60).max(widest + 60));
        let panel = Rect::new(cx - py::fdiv32(pw, 2), top, pw, bottom - top);
        draw::rect_r(s, p.panel, panel, 0, 14);
        draw::rect_r(s, p.border_light, panel, 1, 14);
        s.blr(&head, head.get_rect().center_at((cx, cy - 70)));
        let mut y = cy - 24;
        for img in &imgs {
            s.blr(img, img.get_rect().center_at((cx, y)));
            y += 32;
        }
        s.blr(&hint, hint.get_rect().center_at((cx, y + 10)));
    }

    fn draw_setup(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let yaw = py::fpmod(self.anim_t * py::radians(6.0), TAU);
        self.basis = Self::view_basis(yaw, py::radians(4.0), self.kick);
        self.draw_background(s);
        self.apply_blur(s);
        self.dim(s, (5, 6, 12), 120);
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let accent = self.th().accent;
        let title = self.huge.render("AIM TRAINER", true, accent);
        s.blr(&title, title.get_rect().center_at((cx, (h as f64 * 0.13) as i32)));
        let mode_lbl = if MODE_KEYS.contains(&self.mode()) { t(&format!("aim.mode.{}", self.mode())) } else { self.mode().to_string() };
        let sub = self.small.render(&format!("{}   -   {}", mode_lbl, t("aim.subtitle")), true, p.text_dim);
        s.blr(&sub, sub.get_rect().center_at((cx, (h as f64 * 0.20) as i32)));
        let lbl = self.small.render(&t("aim.setup_theme"), true, p.text_dim);
        s.blr(&lbl, lbl.get_rect().midbottom_at((cx, self.theme_rects[0].y - 8)));
        for (i, r) in self.theme_rects.iter().enumerate() {
            let on = i == self.theme_idx;
            draw::rect_r(s, if on { p.btn_sel } else { p.btn }, *r, 0, 8);
            if on {
                draw::rect_r(s, accent, *r, 2, 8);
            } else {
                draw::rect_r(s, p.border, *r, 1, 8);
            }
            let img = self.small.render(&t(&format!("aim.theme.{}", THEME_KEYS[i])), true, if on { p.text } else { p.text_dim });
            s.blr(&img, img.get_rect().center_at(r.center()));
        }
        let rows = [
            ("aim.setup_sens", self.sens_minus, self.sens_plus, self.sens_box, format!("{:.1}", self.sens)),
            ("aim.setup_blur", self.blur_minus, self.blur_plus, self.blur_box, self.blur_label()),
        ];
        for (label_key, minus, plus, bx, value) in rows {
            let lbl = self.small.render(&t(label_key), true, p.text_dim);
            s.blr(&lbl, lbl.get_rect().midright_at((minus.x - 16, minus.centery())));
            for (r, sym) in [(minus, "-"), (plus, "+")] {
                draw::rect_r(s, p.btn, r, 0, 8);
                draw::rect_r(s, p.border, r, 1, 8);
                let img = self.big.render(sym, true, p.text);
                s.blr(&img, img.get_rect().center_at(r.center()));
            }
            let img = self.big.render(&value, true, accent);
            s.blr(&img, img.get_rect().center_at(bx.center()));
        }
        draw::rect_r(s, p.btn_sel, self.start_rect, 0, 10);
        draw::rect_r(s, accent, self.start_rect, 2, 10);
        let st = self.b.font.render(&t("common.start"), true, p.text);
        s.blr(&st, st.get_rect().center_at(self.start_rect.center()));
        let hint = self.tiny.render(&t("aim.setup_hint"), true, p.text_faint);
        s.blr(&hint, hint.get_rect().center_at((cx, h - 14)));
    }
}

impl Game for AimTrainerGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        self.b.score = 0;
        self.b.game_over = false;
        self.cfg = mode_cfg(&self.b.mode);
        let (theme, sens, blur) = self.aim_settings();
        self.theme_idx = theme;
        self.sens = sens;
        self.blur = blur;
        self.prev_frame = None;
        self.make_fonts();
        self.yaw = 0.0;
        self.pitch = 0.0;
        self.kick = 0.0;
        self.anim_t = 0.0;
        self.go_last = None;
        self.capture = false;
        self.sky_cache = None;
        self.bh_cache = None;
        self.sun_cache = None;
        self.stars = Self::make_stars(220);
        self.build_setup_layout();
        self.state = State::Setup;
    }

    fn on_surface_changed(&mut self) {
        self.make_fonts();
        self.sky_cache = None;
        self.bh_cache = None;
        self.sun_cache = None;
        self.prev_frame = None;
        self.build_setup_layout();
    }

    fn capture_mouse(&self) -> bool {
        self.capture
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if self.state == State::Setup {
            self.handle_setup(ev);
            return;
        }
        if self.b.game_over {
            if ev.kind == Kind::KeyDown && ev.key_in(&["Return", "space"]) {
                self.reset();
            }
            return;
        }
        if ev.kind == Kind::MouseRel {
            self.apply_look(ev.rel);
        } else if ev.kind == Kind::MouseDown && ev.button == 1 {
            self.shoot();
        } else if ev.kind == Kind::KeyDown {
            match ev.key.as_str() {
                "plus" | "equal" | "KP_Add" => self.change_sens(0.1),
                "minus" | "KP_Subtract" => self.change_sens(-0.1),
                "e" | "E" if self.mode() == "chill" => self.finish_run(),
                _ => {}
            }
        }
    }

    fn update(&mut self, dt: f64) {
        self.anim_t += dt;
        if self.state != State::Play || self.b.game_over {
            return;
        }
        self.play_t += dt;
        self.cooldown = 0f64.max(self.cooldown - dt);
        self.flash = 0f64.max(self.flash - dt);
        self.tracer = 0f64.max(self.tracer - dt);
        self.hitmark = 0f64.max(self.hitmark - dt);
        self.kick -= self.kick * 1f64.min(dt * 9.0);
        if self.toast_t > 0.0 {
            self.toast_t -= dt;
        }
        self.update_targets(dt);
        if self.mode() == "chill" && self.combo > 0 && self.play_t - self.last_hit_t > 5.0 {
            self.combo = 0;
        }
        for p in self.particles.iter_mut() {
            p.age += dt;
            if p.age < p.life {
                p.pos.0 += p.v.0 * dt;
                p.pos.1 += p.v.1 * dt;
                p.pos.2 += p.v.2 * dt;
            }
        }
        self.particles.retain(|p| p.age < p.life);
        self.popups = std::mem::take(&mut self.popups).into_iter().filter(|(_, _, age)| age + dt < 0.9).map(|(t, p, age)| (t, p, age + dt)).collect();
        if let Some(tl) = self.time_left.as_mut() {
            *tl -= dt;
            if *tl <= 0.0 {
                *tl = 0.0;
                self.finish_run();
            }
        }
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        self.scx = self.b.width as f64 / 2.0;
        self.scy = self.b.height as f64 / 2.0;
        self.f = self.b.height as f64 * FOV_MUL;
        if self.state == State::Setup {
            self.draw_setup(&mut s);
            return;
        }
        if self.b.game_over {
            let now = py::perf_counter();
            if let Some(last) = self.go_last {
                let gdt = 0.05f64.min(now - last);
                self.anim_t += gdt;
                self.yaw = py::fpmod(self.yaw + py::radians(8.0) * gdt, TAU);
            }
            self.go_last = Some(now);
        }
        self.basis = Self::view_basis(self.yaw, self.pitch, self.kick);
        self.draw_background(&mut s);
        self.draw_targets(&mut s);
        self.draw_particles(&mut s);
        self.draw_popups(&mut s);
        self.draw_tracer_flash(&mut s);
        self.apply_blur(&mut s);
        self.draw_crosshair(&mut s);
        self.draw_hud(&mut s);
        if self.b.game_over {
            self.draw_result(&mut s);
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "aim",
    class_name: "AimTrainerGame",
    supports_multiplayer: false,
    modes: &[
        ("precision", "aim.mode.precision"),
        ("reflex", "aim.mode.reflex"),
        ("moving", "aim.mode.moving"),
        ("chill", "aim.mode.chill"),
    ],
    make: AimTrainerGame::new,
};
