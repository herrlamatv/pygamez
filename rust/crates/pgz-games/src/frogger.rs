//! frogger.rs
//! ==========
//! Frogger - Klassiker mit Extras.
//!
//! Uebertragen aus `games/frogger.py`: 13 Reihen (Buchten, Fluss, Mittel-
//! streifen, Strasse, Start), Staemme und tauchende Schildkroeten, Autos und
//! Laster, Bonus-Fliege und Krokodil in den Buchten, Zeitbalken, drei Stufen,
//! Level werden schneller, Extraleben bei 10 000 Punkten.

use pgz_engine::prelude::*;

const COL_GRASS: Color = Color::rgb(36, 66, 46);
const COL_GRASS_DARK: Color = Color::rgb(28, 52, 38);
const COL_ROAD: Color = Color::rgb(30, 33, 44);
const COL_ROAD_LINE: Color = Color::rgb(70, 76, 96);
const COL_RIVER: Color = Color::rgb(24, 38, 66);
const COL_BAY: Color = Color::rgb(16, 24, 38);
const COL_BAY_DONE: Color = Color::rgb(56, 120, 80);
const COL_FROG: Color = Color::rgb(108, 205, 109);
const COL_FROG_DARK: Color = Color::rgb(58, 130, 78);
const COL_CAR: [Color; 4] = [Color::rgb(225, 95, 95), Color::rgb(245, 205, 100), Color::rgb(150, 160, 235), Color::rgb(240, 240, 250)];
const COL_TRUCK: Color = Color::rgb(150, 158, 178);
const COL_LOG: Color = Color::rgb(128, 92, 60);
const COL_LOG_DARK: Color = Color::rgb(96, 68, 44);
const COL_TURTLE: Color = Color::rgb(90, 160, 120);
const COL_TURTLE_DARK: Color = Color::rgb(60, 110, 84);
const COL_CROC: Color = Color::rgb(70, 130, 70);
const COL_FLY: Color = Color::rgb(245, 205, 100);

struct Diff {
    key: &'static str,
    speed: f64,
    gap: f64,
    timer: f64,
    croc_lv: i64,
    dive_lv: i64,
}

const DIFFS: [Diff; 3] = [
    Diff { key: "easy", speed: 0.8, gap: 3.5, timer: 45.0, croc_lv: 4, dive_lv: 3 },
    Diff { key: "normal", speed: 1.0, gap: 2.75, timer: 35.0, croc_lv: 2, dive_lv: 2 },
    Diff { key: "hard", speed: 1.25, gap: 2.0, timer: 25.0, croc_lv: 1, dive_lv: 1 },
];

#[derive(Clone, Copy, PartialEq)]
enum Art {
    Car,
    Truck,
    Log,
    Turtle,
}

const ROAD_LANES: [(Art, f64, i32); 5] = [(Art::Car, 1.6, 1), (Art::Truck, -1.1, 2), (Art::Car, 1.9, 1), (Art::Car, -2.4, 1), (Art::Truck, 1.3, 2)];
const RIVER_LANES: [(Art, f64, i32); 5] = [(Art::Log, 1.2, 3), (Art::Turtle, -1.5, 3), (Art::Log, 1.9, 4), (Art::Turtle, -1.2, 2), (Art::Log, 1.6, 2)];

const EXTRA_LIFE_AT: i64 = 10_000;
const FLY_TIME: f64 = 4.0;
const CROC_TIME: f64 = 5.0;
const DIVE_CYCLE: f64 = 10.0;

#[derive(Clone, Copy, PartialEq)]
enum Turtle {
    Up,
    Blink,
    Under,
    Rise,
}

#[derive(Clone, Copy)]
struct Ent {
    x: f64,
    w: i32,
    color: Color,
    dive: bool,
    phase: f64,
}

struct Lane {
    row: i32,
    kind: Art,
    speed: f64,
    ents: Vec<Ent>,
    span: f64,
    margin: i32,
}

#[derive(Clone, Copy)]
struct BayExtra {
    bay: usize,
    t: f64,
}

/// `FroggerGame`
pub struct FroggerGame {
    b: GameBase,
    diff_idx: usize,
    small: Font,
    tiny: Font,
    huge: Font,
    anim_t: f64,
    hud_h: i32,
    bar_h: i32,
    cell: i32,
    field_top: i32,
    diff_rects: Vec<Rect>,
    start_rect: Rect,
    setup: bool,
    lives: i64,
    level: i64,
    extra_life_given: bool,
    bays_done: [bool; 5],
    fly: Option<BayExtra>,
    croc: Option<BayExtra>,
    fly_timer: f64,
    death_t: f64,
    clear_t: f64,
    lanes: Vec<Lane>,
    frog_row: i32,
    frog_x: f64,
    best_row: i32,
    time_left: f64,
}

impl FroggerGame {
    /// `FroggerGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let mut g = FroggerGame {
            b,
            diff_idx: 1,
            small: f.clone(),
            tiny: f.clone(),
            huge: f,
            anim_t: 0.0,
            hud_h: 30,
            bar_h: 8,
            cell: 16,
            field_top: 30,
            diff_rects: Vec::new(),
            start_rect: z,
            setup: true,
            lives: 3,
            level: 1,
            extra_life_given: false,
            bays_done: [false; 5],
            fly: None,
            croc: None,
            fly_timer: 0.0,
            death_t: 0.0,
            clear_t: 0.0,
            lanes: Vec::new(),
            frog_row: 12,
            frog_x: 0.0,
            best_row: 12,
            time_left: 0.0,
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

    fn diff(&self) -> &'static Diff {
        &DIFFS[self.diff_idx]
    }

    fn layout(&mut self) {
        let h = self.b.height;
        self.hud_h = 30.max((h as f64 * 0.08) as i32);
        self.bar_h = 8.max(h / 48);
        self.cell = 16.max(py::fdiv32(h - self.hud_h - self.bar_h, 13));
        self.field_top = self.hud_h;
    }

    fn row_y(&self, row: i32) -> i32 {
        self.field_top + row * self.cell
    }

    fn build_setup_layout(&mut self) {
        let cx = self.b.width / 2;
        let bw = 420.min(self.b.width - 60);
        let y0 = (self.b.height as f64 * 0.30) as i32;
        self.diff_rects = (0..3).map(|i| Rect::new(cx - py::fdiv32(bw, 2), y0 + i * 62, bw, 52)).collect();
        self.start_rect = Rect::new(cx - 95, y0 + 3 * 62 + 14, 190, 46);
    }

    fn save_diff(&mut self) {
        self.b.set_opt("frogger", "difficulty", self.diff().key.into());
    }

    fn handle_setup(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown {
            let k = ev.key.as_str();
            match k {
                "1" | "2" | "3" => {
                    self.diff_idx = k.parse::<usize>().unwrap_or(2) - 1;
                    self.save_diff();
                    self.b.play_sound("click");
                }
                "Up" | "w" | "W" => {
                    self.diff_idx = (self.diff_idx + 2) % 3;
                    self.save_diff();
                    self.b.play_sound("move");
                }
                "Down" | "s" | "S" => {
                    self.diff_idx = (self.diff_idx + 1) % 3;
                    self.save_diff();
                    self.b.play_sound("move");
                }
                "Return" | "space" => self.new_game(),
                _ => {}
            }
        } else if ev.kind == Kind::MouseDown {
            if let Some(i) = self.diff_rects.iter().position(|r| r.hit(ev.pos)) {
                self.diff_idx = i;
                self.save_diff();
                self.b.play_sound("click");
                return;
            }
            if self.start_rect.hit(ev.pos) {
                self.new_game();
            }
        }
    }

    fn new_game(&mut self) {
        self.b.score = 0;
        self.b.game_over = false;
        self.lives = 3;
        self.level = 1;
        self.extra_life_given = false;
        self.bays_done = [false; 5];
        self.fly = None;
        self.croc = None;
        self.fly_timer = py::uniform(10.0, 16.0);
        self.death_t = 0.0;
        self.clear_t = 0.0;
        self.build_lanes();
        self.respawn(true);
        self.setup = false;
        self.b.play_sound("click");
    }

    fn speed_mult(&self) -> f64 {
        2.5f64.min(self.diff().speed * (1.0 + 0.12 * (self.level - 1) as f64))
    }

    fn build_lanes(&mut self) {
        self.lanes.clear();
        let d = self.diff();
        let mult = self.speed_mult();
        let c = self.cell;
        let w = self.b.width;
        let add_lane = |lanes: &mut Vec<Lane>, row: i32, kind: Art, base_speed: f64, w_cells: i32| {
            let speed = base_speed * mult * c as f64;
            let ew = w_cells * c;
            let gap = d.gap * c as f64 * py::uniform(0.8, 1.3);
            let margin = ew + c;
            let n = 2.max(py::ffdiv((w + 2 * margin) as f64, ew as f64 + gap) as i64 + 1);
            let span = n as f64 * (ew as f64 + gap);
            let ents = (0..n)
                .map(|i| {
                    let x = i as f64 * (ew as f64 + gap) - margin as f64;
                    let color = py::choice(&COL_CAR);
                    let phase = py::uniform(0.0, DIVE_CYCLE);
                    Ent { x, w: ew, color, dive: false, phase }
                })
                .collect();
            lanes.push(Lane { row, kind, speed, ents, span, margin });
        };
        let mut lanes = Vec::new();
        for (i, (kind, sp, wc)) in ROAD_LANES.iter().enumerate() {
            add_lane(&mut lanes, 7 + i as i32, *kind, *sp, *wc);
        }
        for (i, (kind, sp, wc)) in RIVER_LANES.iter().enumerate() {
            add_lane(&mut lanes, 1 + i as i32, *kind, *sp, *wc);
        }
        if self.level >= d.dive_lv {
            for lane in lanes.iter_mut() {
                if lane.kind == Art::Turtle {
                    for (j, e) in lane.ents.iter_mut().enumerate() {
                        e.dive = j % 2 == 0;
                    }
                }
            }
        }
        self.lanes = lanes;
    }

    fn respawn(&mut self, full: bool) {
        self.frog_row = 12;
        self.frog_x = self.b.width as f64 / 2.0;
        self.best_row = 12;
        self.time_left = self.diff().timer;
        if full {
            self.anim_t = 0.0;
        }
    }

    fn hop(&mut self, dx: i32, dy: i32) {
        let nx = self.frog_x + (dx * self.cell) as f64;
        let nrow = self.frog_row + dy;
        if !(0..=12).contains(&nrow) {
            return;
        }
        let half = self.cell as f64 * 0.5;
        if !(half <= nx && nx <= self.b.width as f64 - half) {
            return;
        }
        self.frog_x = nx;
        self.frog_row = nrow;
        self.b.play_sound("move");
        if nrow < self.best_row {
            self.b.score += 10 * (self.best_row - nrow) as i64;
            self.best_row = nrow;
        }
        if nrow == 0 {
            self.try_home();
        }
    }

    fn lane_for_row(&self, row: i32) -> Option<&Lane> {
        self.lanes.iter().find(|l| l.row == row)
    }

    fn turtle_state(&self, e: &Ent) -> Turtle {
        if !e.dive {
            return Turtle::Up;
        }
        let tt = py::fpmod(self.anim_t + e.phase, DIVE_CYCLE);
        if tt < 6.0 {
            Turtle::Up
        } else if tt < 7.0 {
            Turtle::Blink
        } else if tt < 9.0 {
            Turtle::Under
        } else {
            Turtle::Rise
        }
    }

    /// Geschwindigkeit des Traegers unter x (oder None = ertrinken).
    fn carrier_at(&self, row: i32, x: f64) -> Option<f64> {
        let lane = self.lane_for_row(row)?;
        for e in &lane.ents {
            if e.x <= x && x <= e.x + e.w as f64 {
                if lane.kind == Art::Turtle && self.turtle_state(e) == Turtle::Under {
                    return None;
                }
                return Some(lane.speed);
            }
        }
        None
    }

    fn vehicle_hit(&self, row: i32, x: f64) -> bool {
        let Some(lane) = self.lane_for_row(row) else { return false };
        let half = self.cell as f64 * 0.3;
        lane.ents.iter().any(|e| e.x < x + half && x - half < e.x + e.w as f64)
    }

    fn bay_index_at(&self, x: f64) -> Option<usize> {
        (0..5).find(|&i| {
            let cx = (i as f64 + 0.5) * self.b.width as f64 / 5.0;
            (x - cx).abs() < 0.5 * self.cell as f64
        })
    }

    fn try_home(&mut self) {
        let Some(bay) = self.bay_index_at(self.frog_x).filter(|b| !self.bays_done[*b]) else {
            self.die();
            return;
        };
        if self.croc.is_some_and(|c| c.bay == bay) {
            self.die();
            return;
        }
        self.bays_done[bay] = true;
        let bonus = 50 + 10 * 0f64.max(self.time_left) as i64;
        self.b.score += bonus;
        if self.fly.is_some_and(|f| f.bay == bay) {
            self.b.score += 200;
            self.fly = None;
            self.b.ach_event("frog_fly", None);
            self.b.play_sound("powerup");
        }
        self.b.play_sound("point");
        if self.bays_done.iter().all(|d| *d) {
            self.b.score += 1000;
            self.clear_t = 1.5;
            self.b.ach_event("frog_home", None);
            self.b.play_sound("level");
        } else {
            self.respawn(false);
        }
    }

    fn next_level(&mut self) {
        self.level += 1;
        self.bays_done = [false; 5];
        self.fly = None;
        self.croc = None;
        self.build_lanes();
        self.respawn(false);
    }

    fn update_bay_extras(&mut self, dt: f64) {
        if let Some(f) = self.fly.as_mut() {
            f.t -= dt;
            if f.t <= 0.0 {
                self.fly = None;
            }
        }
        if let Some(c) = self.croc.as_mut() {
            c.t -= dt;
            if c.t <= 0.0 {
                self.croc = None;
            }
        }
        self.fly_timer -= dt;
        if self.fly_timer <= 0.0 {
            self.fly_timer = py::uniform(10.0, 16.0);
            let free: Vec<usize> = (0..5)
                .filter(|&i| !self.bays_done[i] && !self.croc.is_some_and(|c| c.bay == i) && !self.fly.is_some_and(|f| f.bay == i))
                .collect();
            if !free.is_empty() {
                if self.level >= self.diff().croc_lv && self.croc.is_none() && py::random() < 0.45 {
                    self.croc = Some(BayExtra { bay: py::choice(&free), t: CROC_TIME });
                } else if self.fly.is_none() {
                    self.fly = Some(BayExtra { bay: py::choice(&free), t: FLY_TIME });
                }
            }
        }
    }

    fn die(&mut self) {
        self.death_t = 1.0;
        self.lives -= 1;
        self.b.play_sound("hit");
        self.b.rumble(160);
    }

    fn after_death(&mut self) {
        if self.lives <= 0 {
            self.b.game_over = true;
            self.b.play_sound("gameover");
            self.b.rumble(250);
        } else {
            self.respawn(false);
        }
    }

    fn draw_field(&self, s: &mut Surface) {
        let c = self.cell;
        let w = self.b.width;
        let y0 = self.row_y(0);
        draw::rect(s, COL_GRASS, (0, y0, w, c), 0);
        for i in 0..5 {
            let cx = (i as f64 + 0.5) * w as f64 / 5.0;
            let bay = Rect::new((cx - c as f64 * 0.6) as i32, y0 + 2, (c as f64 * 1.2) as i32, c - 4);
            draw::rect_r(s, if self.bays_done[i] { COL_BAY_DONE } else { COL_BAY }, bay, 0, 6);
            if self.bays_done[i] {
                self.draw_frog(s, cx, y0, true);
            }
        }
        draw::rect(s, COL_RIVER, (0, self.row_y(1), w, 5 * c), 0);
        draw::rect(s, COL_GRASS_DARK, (0, self.row_y(6), w, c), 0);
        draw::rect(s, COL_GRASS_DARK, (0, self.row_y(12), w, c), 0);
        draw::rect(s, COL_ROAD, (0, self.row_y(7), w, 5 * c), 0);
        for r in 8..12 {
            let y = self.row_y(r);
            let mut x = 0;
            while x < w {
                draw::rect(s, COL_ROAD_LINE, (x + c / 4, y - 1, c / 2, 2), 0);
                x += c;
            }
        }
    }

    fn draw_entities(&self, s: &mut Surface) {
        let c = self.cell;
        for lane in &self.lanes {
            let y = self.row_y(lane.row);
            for e in &lane.ents {
                let r = Rect::new(e.x as i32, y + 3, e.w, c - 6);
                match lane.kind {
                    Art::Car | Art::Truck => {
                        let col = if lane.kind == Art::Truck { COL_TRUCK } else { e.color };
                        draw::rect_r(s, col, r, 0, 5);
                        let dunkel = rgb((col.r as f64 * 0.6) as i32, (col.g as f64 * 0.6) as i32, (col.b as f64 * 0.6) as i32);
                        draw::rect_r(s, dunkel, r, 2, 5);
                        let wx = if lane.speed > 0.0 { r.right() - c / 3 } else { r.x + 4 };
                        draw::rect_r(s, rgb(200, 225, 245), (wx, r.y + 3, c / 4, r.h - 6), 0, 3);
                    }
                    Art::Log => {
                        draw::rect_r(s, COL_LOG, r, 0, c / 3);
                        draw::rect_r(s, COL_LOG_DARK, r, 2, c / 3);
                        let mut lx = r.x + c / 2;
                        while lx < r.right() - c / 3 {
                            draw::line(s, COL_LOG_DARK, (lx, r.y + 4), (lx, r.bottom() - 4), 1);
                            lx += c;
                        }
                    }
                    Art::Turtle => {
                        let st = self.turtle_state(e);
                        if st == Turtle::Under {
                            continue;
                        }
                        let mut col = COL_TURTLE;
                        if st == Turtle::Blink && (self.anim_t * 8.0) as i64 % 2 == 0 {
                            col = COL_TURTLE_DARK;
                        }
                        if st == Turtle::Rise {
                            col = COL_TURTLE_DARK;
                        }
                        let n = 1.max(e.w / c);
                        for i in 0..n {
                            let cxx = (e.x + (i as f64 + 0.5) * c as f64) as i32;
                            draw::circle(s, col, (cxx, y + c / 2), c / 2 - 3, 0);
                            draw::circle(s, COL_TURTLE_DARK, (cxx, y + c / 2), c / 2 - 3, 2);
                        }
                    }
                }
            }
        }
    }

    fn draw_bay_extras(&self, s: &mut Surface) {
        let c = self.cell;
        let y0 = self.row_y(0);
        let w = self.b.width as f64;
        if let Some(f) = self.fly {
            let cx = ((f.bay as f64 + 0.5) * w / 5.0) as i32;
            let fy = y0 + c / 2;
            draw::circle(s, COL_FLY, (cx, fy), 3.max(c / 6), 0);
            let off = 2.max(c / 8);
            for sx in [-off, off] {
                draw::circle(s, rgb(255, 240, 200), (cx + sx, fy - off), 2.max(c / 9), 1);
            }
        }
        if let Some(cr) = self.croc {
            let cx = ((cr.bay as f64 + 0.5) * w / 5.0) as i32;
            let r = Rect::new(cx - (c as f64 * 0.55) as i32, y0 + c / 4, (c as f64 * 1.1) as i32, c / 2);
            draw::rect_r(s, COL_CROC, r, 0, 4);
            draw::polygon(s, rgb(240, 240, 250), &[(r.x + 4, r.centery() - 2), (r.x + c / 3, r.y + 2), (r.x + c / 3, r.bottom() - 2)], 0);
            draw::circle(s, rgb(20, 24, 34), (r.right() - c / 4, r.y + 4), 2, 0);
        }
    }

    fn draw_frog(&self, s: &mut Surface, x: f64, y: i32, small: bool) {
        let c = self.cell;
        let r = c / 2 - if small { 4 } else { 2 };
        let (cx, cy) = (x as i32, y + c / 2);
        if !small {
            for sx in [-1, 1] {
                draw::line(s, COL_FROG_DARK, (cx + sx * r, cy), (cx + sx * (r + 3), cy + r), 3);
            }
        }
        draw::ellipse(s, COL_FROG, (cx - r, cy - r, 2 * r, 2 * r), 0);
        draw::ellipse(s, COL_FROG_DARK, (cx - r, cy - r, 2 * r, 2 * r), 2);
        let er = 2.max(r / 3);
        for sx in [-1, 1] {
            let p = (cx + py::fdiv32(sx * r, 2), cy - r + er / 2);
            draw::circle(s, rgb(240, 240, 250), p, er, 0);
            draw::circle(s, rgb(20, 24, 34), p, 1.max(er / 2), 0);
        }
    }

    fn draw_death(&self, s: &mut Surface) {
        let p = ui::pal();
        let c = self.cell;
        let cx = self.frog_x as i32;
        let cy = self.row_y(self.frog_row) + c / 2;
        let f = 1.0 - self.death_t;
        let n = 8;
        for i in 0..n {
            let a = i as f64 / n as f64 * py::TAU;
            let d = f * c as f64 * 0.9;
            let (px, py_) = (cx as f64 + a.cos() * d, cy as f64 + a.sin() * d);
            draw::circle(s, COL_FROG_DARK, (px as i32, py_ as i32), 1.max((4.0 * (1.0 - f)) as i32), 0);
        }
        let img = self.small.render("+", true, p.red);
        s.blr(&img, img.get_rect().center_at((cx, cy)));
    }

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        draw::rect(s, p.panel, (0, 0, w, self.hud_h), 0);
        draw::line(s, p.border, (0, self.hud_h), (w, self.hud_h), 1);
        let cy = self.hud_h / 2;
        let img = self.small.render(&tf("common.points", &[("score", &self.b.score.to_string())]), true, p.text);
        s.blr(&img, img.get_rect().midleft_at((12, cy)));
        let lvl = self.small.render(&tf("frog.level", &[("n", &self.level.to_string())]), true, self.b.accent);
        s.blr(&lvl, lvl.get_rect().center_at((w / 2, cy)));
        for i in 0..self.lives.max(0) as i32 {
            let fx = w - 20 - i * (self.cell / 2 + 8);
            self.draw_frog(s, fx as f64, cy - self.cell / 2, true);
        }
        let frac = 0f64.max(self.time_left / self.diff().timer);
        let bar = Rect::new(0, h - self.bar_h, (w as f64 * frac) as i32, self.bar_h);
        draw::rect(s, if frac > 0.25 { p.green } else { p.red }, bar, 0);
    }

    fn draw_banner(&self, s: &mut Surface, text: &str) {
        let p = ui::pal();
        let img = self.huge.render(text, true, self.b.accent);
        let bg = img.get_rect().center_at((self.b.width / 2, self.b.height / 2)).inflate(48, 26);
        let mut ov = pg::surface_alpha(bg.w, bg.h);
        ov.fill_c(pg::alpha(p.panel, 225));
        s.bl(&ov, (bg.x, bg.y));
        draw::rect_r(s, self.b.accent, bg, 2, 12);
        s.blr(&img, img.get_rect().center_at(bg.center()));
    }

    fn draw_gameover(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let mut dim = pg::surface_alpha(w, h);
        dim.fill_c(rgba(8, 10, 16, 150));
        s.bl(&dim, (0, 0));
        let (cx, cy) = (w / 2, h / 2);
        let rows = [
            self.huge.render(&t("common.game_over"), true, p.red),
            self.b.font.render(&tf("common.points", &[("score", &self.b.score.to_string())]), true, p.text),
            self.small.render(&t("frog.retry"), true, p.text_dim),
        ];
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

    fn draw_setup(&self) {
        let p = ui::pal();
        let mut s = self.b.sf();
        let (w, h) = (self.b.width, self.b.height);
        ui::draw_background(&mut s, w, h);
        let name = pgz_core::achievements::game_name(self.b.key).to_uppercase();
        let title = self.huge.render(&name, true, self.b.accent);
        s.blr(&title, title.get_rect().center_at((w / 2, (h as f64 * 0.14) as i32)));
        let sub = self.small.render(&t("frog.subtitle"), true, p.text_dim);
        s.blr(&sub, sub.get_rect().center_at((w / 2, (h as f64 * 0.20) as i32)));
        for (i, r) in self.diff_rects.iter().enumerate() {
            let on = i == self.diff_idx;
            draw::rect_r(&mut s, if on { p.btn_sel } else { p.btn }, *r, 0, 10);
            draw::rect_r(&mut s, if on { self.b.accent } else { p.border }, *r, if on { 2 } else { 1 }, 10);
            let name = self.b.font.render(&t(&format!("frog.diff.{}", DIFFS[i].key)), true, if on { p.text } else { p.text_dim });
            s.blr(&name, name.get_rect().midleft_at((r.x + 18, r.y + 18)));
            let desc = self.tiny.render(&t(&format!("frog.diff_desc.{}", DIFFS[i].key)), true, p.text_dim);
            s.blr(&desc, desc.get_rect().midleft_at((r.x + 18, r.bottom() - 15)));
        }
        draw::rect_r(&mut s, p.btn_sel, self.start_rect, 0, 10);
        draw::rect_r(&mut s, self.b.accent, self.start_rect, 2, 10);
        let st = self.b.font.render(&t("common.start"), true, p.text);
        s.blr(&st, st.get_rect().center_at(self.start_rect.center()));
        let hint = self.tiny.render(&t("frog.setup_hint"), true, p.text_faint);
        s.blr(&hint, hint.get_rect().center_at((w / 2, h - 30)));
        let ctrl = self.tiny.render(&t("frog.hint"), true, ui::mix(self.b.accent, p.text, 0.45));
        s.blr(&ctrl, ctrl.get_rect().center_at((w / 2, h - 12)));
    }
}

impl Game for FroggerGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        self.b.score = 0;
        self.b.game_over = false;
        let key = self.b.opt_str("frogger", "difficulty", "normal");
        self.diff_idx = DIFFS.iter().position(|d| d.key == key).unwrap_or(1);
        self.make_fonts();
        self.anim_t = 0.0;
        self.layout();
        self.build_setup_layout();
        self.setup = true;
    }

    fn on_surface_changed(&mut self) {
        self.make_fonts();
        self.layout();
        self.build_setup_layout();
        if !self.setup {
            self.build_lanes();
            self.respawn(false);
        }
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if self.setup {
            self.handle_setup(ev);
            return;
        }
        if self.b.game_over {
            if ev.kind == Kind::KeyDown {
                if ev.key_in(&["Return", "space"]) {
                    self.new_game();
                } else if ev.key_in(&["s", "S"]) {
                    self.b.game_over = false;
                    self.setup = true;
                    self.b.play_sound("click");
                }
            }
            return;
        }
        if ev.kind != Kind::KeyDown {
            return;
        }
        let k = ev.key.as_str();
        if matches!(k, "r" | "R") {
            self.new_game();
            return;
        }
        if matches!(k, "s" | "S") && !self.b.is_action(k, "down", None) {
            self.setup = true;
            self.b.play_sound("click");
            return;
        }
        if self.death_t > 0.0 || self.clear_t > 0.0 {
            return;
        }
        if self.b.is_action(k, "up", None) || k == "Up" {
            self.hop(0, -1);
        } else if self.b.is_action(k, "down", None) || k == "Down" {
            self.hop(0, 1);
        } else if self.b.is_action(k, "left", None) || k == "Left" {
            self.hop(-1, 0);
        } else if self.b.is_action(k, "right", None) || k == "Right" {
            self.hop(1, 0);
        }
    }

    fn update(&mut self, dt: f64) {
        self.anim_t += dt;
        if self.setup {
            return;
        }
        let w = self.b.width as f64;
        for lane in self.lanes.iter_mut() {
            for e in lane.ents.iter_mut() {
                e.x += lane.speed * dt;
                if lane.speed > 0.0 && e.x > w + lane.margin as f64 {
                    e.x -= lane.span;
                } else if lane.speed < 0.0 && e.x + (e.w as f64) < -(lane.margin as f64) {
                    e.x += lane.span;
                }
            }
        }
        if self.b.game_over {
            return;
        }
        if self.clear_t > 0.0 {
            self.clear_t -= dt;
            if self.clear_t <= 0.0 {
                self.next_level();
            }
            return;
        }
        if self.death_t > 0.0 {
            self.death_t -= dt;
            if self.death_t <= 0.0 {
                self.after_death();
            }
            return;
        }
        if !self.extra_life_given && self.b.score >= EXTRA_LIFE_AT {
            self.extra_life_given = true;
            self.lives += 1;
            self.b.play_sound("powerup");
        }
        self.update_bay_extras(dt);
        self.time_left -= dt;
        if self.time_left <= 0.0 {
            self.die();
            return;
        }
        if (1..=5).contains(&self.frog_row) {
            let Some(speed) = self.carrier_at(self.frog_row, self.frog_x) else {
                self.die();
                return;
            };
            self.frog_x += speed * dt;
            let half = self.cell as f64 * 0.5;
            if self.frog_x < half || self.frog_x > w - half {
                self.die();
                return;
            }
        }
        if (7..=11).contains(&self.frog_row) && self.vehicle_hit(self.frog_row, self.frog_x) {
            self.die();
        }
    }

    fn draw(&mut self) {
        if self.setup {
            self.draw_setup();
            return;
        }
        let mut s = self.b.sf();
        ui::draw_background(&mut s, self.b.width, self.b.height);
        self.draw_field(&mut s);
        self.draw_entities(&mut s);
        self.draw_bay_extras(&mut s);
        if self.death_t > 0.0 {
            self.draw_death(&mut s);
        } else if self.clear_t <= 0.0 {
            self.draw_frog(&mut s, self.frog_x, self.row_y(self.frog_row), false);
        }
        self.draw_hud(&mut s);
        if self.clear_t > 0.0 {
            self.draw_banner(&mut s, &tf("frog.level_clear", &[("n", &self.level.to_string())]));
        }
        if self.b.game_over {
            self.draw_gameover(&mut s);
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "frogger",
    class_name: "FroggerGame",
    supports_multiplayer: false,
    modes: &[],
    make: FroggerGame::new,
};
