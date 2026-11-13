//! pacman.rs
//! =========
//! Pac-Man - originalgetreuer Klon.
//!
//! Uebertragen aus `games/pacman.py`: 28x31-Labyrinth mit Pillen, Power-
//! Pillen, Tunnel und Geisterhaus; vier Geister mit Ziel-Kachel-KI
//! (Blinky/Pinky/Inky/Clyde), Scatter/Chase-Plan, Frightened mit Kette,
//! gestaffelte Freigabe, Fruechte, Extraleben, Level, Popups.

use pgz_core::highscore;
use pgz_engine::prelude::*;

const MAZE: [&str; 31] = [
    "############################",
    "#............##............#",
    "#.####.#####.##.#####.####.#",
    "#o####.#####.##.#####.####o#",
    "#.####.#####.##.#####.####.#",
    "#..........................#",
    "#.####.##.########.##.####.#",
    "#.####.##.########.##.####.#",
    "#......##....##....##......#",
    "######.#####.##.#####.######",
    "######.#####.##.#####.######",
    "######.##..........##.######",
    "######.##.###==###.##.######",
    "######.##.#      #.##.######",
    "tttttt....#      #....tttttt",
    "######.##.#      #.##.######",
    "######.##.########.##.######",
    "######.##..........##.######",
    "######.##.########.##.######",
    "######.##.########.##.######",
    "#............##............#",
    "#.####.#####.##.#####.####.#",
    "#.####.#####.##.#####.####.#",
    "#o..##.......  .......##..o#",
    "###.##.##.########.##.##.###",
    "###.##.##.########.##.##.###",
    "#......##....##....##......#",
    "#.##########.##.##########.#",
    "#.##########.##.##########.#",
    "#..........................#",
    "############################",
];
const COLS: i32 = 28;
const ROWS: i32 = 31;
const CLEAR_TILES: [(i32, i32); 2] = [(13, 11), (14, 11)];
const PAC_START: (i32, i32) = (13, 23);
const DOOR_EXIT: (i32, i32) = (13, 11);

type Dir = (i32, i32);
const UP: Dir = (0, -1);
const LEFT: Dir = (-1, 0);
const DOWN: Dir = (0, 1);
const RIGHT: Dir = (1, 0);
const ORDER: [Dir; 4] = [UP, LEFT, DOWN, RIGHT];

const SCHEDULE: [(f64, &str); 8] =
    [(7.0, "scatter"), (20.0, "chase"), (7.0, "scatter"), (20.0, "chase"), (5.0, "scatter"), (20.0, "chase"), (5.0, "scatter"), (1e9, "chase")];

const SPD_PAC: f64 = 8.4;
const SPD_GHOST: f64 = 7.4;
const SPD_FRIGHT: f64 = 4.8;
const SPD_EYES: f64 = 16.0;
const SPD_HOUSE: f64 = 3.6;
const SPD_TUNNEL: f64 = 4.4;

const FRUIT_TABLE: [(i64, Color); 8] = [
    (100, Color::rgb(235, 60, 60)),
    (300, Color::rgb(235, 80, 110)),
    (500, Color::rgb(245, 160, 50)),
    (700, Color::rgb(220, 50, 50)),
    (1000, Color::rgb(120, 220, 120)),
    (2000, Color::rgb(120, 180, 255)),
    (3000, Color::rgb(245, 220, 80)),
    (5000, Color::rgb(200, 210, 230)),
];

struct Diff {
    key: &'static str,
    gspeed: f64,
    fright: f64,
}

const DIFFS: [Diff; 3] =
    [Diff { key: "normal", gspeed: 1.0, fright: 6.0 }, Diff { key: "hard", gspeed: 1.07, fright: 4.5 }, Diff { key: "extreme", gspeed: 1.13, fright: 3.5 }];

const COL_BG: Color = Color::rgb(0, 0, 0);
const COL_WALL: Color = Color::rgb(36, 46, 190);
const COL_WALL_HI: Color = Color::rgb(80, 110, 255);
const COL_DOOR: Color = Color::rgb(255, 180, 210);
const COL_PILL: Color = Color::rgb(250, 220, 170);
const COL_POWER: Color = Color::rgb(255, 200, 120);
const COL_PAC: Color = Color::rgb(255, 235, 50);
const COL_FRIGHT: Color = Color::rgb(36, 40, 210);
const COL_FRIGHT_END: Color = Color::rgb(245, 245, 255);

#[derive(Clone, Copy, PartialEq)]
enum State {
    Setup,
    Ready,
    Play,
    Dying,
    LevelClear,
    GameOver,
}

#[derive(Clone, Copy, PartialEq)]
enum GState {
    Maze,
    House,
    Leaving,
    Eyes,
    Entering,
}

fn add(a: (i32, i32), b: (i32, i32)) -> (i32, i32) {
    (a.0 + b.0, a.1 + b.1)
}

fn opp(d: Dir) -> Dir {
    (-d.0, -d.1)
}

fn dist2(a: (i32, i32), b: (i32, i32)) -> i64 {
    let (dx, dy) = ((a.0 - b.0) as i64, (a.1 - b.1) as i64);
    dx * dx + dy * dy
}

#[derive(Clone, Copy)]
struct Mover {
    tile: (i32, i32),
    dir: Dir,
    frac: f64,
}

#[derive(Clone, Copy)]
struct Ghost {
    m: Mover,
    name: usize,
    color: Color,
    home: (i32, i32),
    scatter: (i32, i32),
    release_at: f64,
    state: GState,
    frightened: bool,
    bob_t: f64,
}

const GHOST_NAMES: [&str; 4] = ["blinky", "pinky", "inky", "clyde"];
const GHOST_HOMES: [(i32, i32); 4] = [(13, 11), (13, 14), (11, 14), (16, 14)];
const GHOST_SCATTER: [(i32, i32); 4] = [(25, 0), (2, 0), (27, 30), (0, 30)];
const GHOST_COLORS: [Color; 4] = [Color::rgb(255, 70, 70), Color::rgb(255, 165, 220), Color::rgb(70, 220, 240), Color::rgb(255, 175, 70)];
const GHOST_RELEASE: [f64; 4] = [-1.0, 2.0, 7.0, 13.0];

impl Ghost {
    fn new(i: usize) -> Ghost {
        Ghost {
            m: Mover { tile: GHOST_HOMES[i], dir: LEFT, frac: 0.0 },
            name: i,
            color: GHOST_COLORS[i],
            home: GHOST_HOMES[i],
            scatter: GHOST_SCATTER[i],
            release_at: GHOST_RELEASE[i],
            state: if i == 0 { GState::Maze } else { GState::House },
            frightened: false,
            bob_t: py::uniform(0.0, py::TAU),
        }
    }
}

#[derive(Clone, Copy)]
struct Fruit {
    pts: i64,
    col: Color,
    t: f64,
}

#[derive(Clone)]
struct Popup {
    x: f64,
    y: f64,
    text: String,
    col: Color,
    t: f64,
    t0: f64,
}

/// `PacmanGame`
pub struct PacmanGame {
    b: GameBase,
    diff: usize,
    cell: i32,
    maze_w: i32,
    maze_h: i32,
    ox: i32,
    oy: i32,
    font: Font,
    digit: Font,
    small: Font,
    tiny: Font,
    big: Font,
    highscore: i64,
    anim_t: f64,
    popups: Vec<Popup>,
    fruit_history: Vec<Color>,
    diff_panel: Rect,
    diff_left: Rect,
    diff_right: Rect,
    start_rect: Rect,
    level: i64,
    lives: i64,
    extra_awarded: bool,
    grid: Vec<Vec<u8>>,
    pellets: Vec<Vec<u8>>,
    dots_total: i64,
    dots_eaten: i64,
    fruit: Option<Fruit>,
    fruit_thresholds: Vec<i64>,
    pac: Mover,
    pac_want: Dir,
    pac_moving: bool,
    pac_mouth: f64,
    ghosts: Vec<Ghost>,
    play_timer: f64,
    phase: usize,
    mode_timer: f64,
    fright_timer: f64,
    ghost_combo: u32,
    fright_eaten: i64,
    freeze: f64,
    state: State,
    ready_timer: f64,
    dying_timer: f64,
    levelclear_timer: f64,
}

impl PacmanGame {
    /// `PacmanGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let mut g = PacmanGame {
            b,
            diff: 0,
            cell: 6,
            maze_w: 0,
            maze_h: 0,
            ox: 0,
            oy: 0,
            font: f.clone(),
            digit: f.clone(),
            small: f.clone(),
            tiny: f.clone(),
            big: f,
            highscore: 0,
            anim_t: 0.0,
            popups: Vec::new(),
            fruit_history: Vec::new(),
            diff_panel: z,
            diff_left: z,
            diff_right: z,
            start_rect: z,
            level: 1,
            lives: 3,
            extra_awarded: false,
            grid: Vec::new(),
            pellets: Vec::new(),
            dots_total: 0,
            dots_eaten: 0,
            fruit: None,
            fruit_thresholds: Vec::new(),
            pac: Mover { tile: PAC_START, dir: LEFT, frac: 0.0 },
            pac_want: LEFT,
            pac_moving: false,
            pac_mouth: 0.2,
            ghosts: Vec::new(),
            play_timer: 0.0,
            phase: 0,
            mode_timer: 0.0,
            fright_timer: 0.0,
            ghost_combo: 0,
            fright_eaten: 0,
            freeze: 0.0,
            state: State::Setup,
            ready_timer: 0.0,
            dying_timer: 0.0,
            levelclear_timer: 0.0,
        };
        g.reset();
        Box::new(g)
    }

    fn layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        let cell = (w / COLS).min(h / (ROWS + 5));
        self.cell = 6.max(cell);
        self.maze_w = COLS * self.cell;
        self.maze_h = ROWS * self.cell;
        self.ox = py::fdiv32(w - self.maze_w, 2);
        let top = (2.2 * self.cell as f64) as i32;
        self.oy = top + 0.max(py::fdiv32(h - top - self.maze_h - 2 * self.cell, 2));
        let c = self.cell as f64;
        self.font = ui::fb(11.max((c * 1.15) as i32));
        self.digit = ui::font(11.max((c * 1.15) as i32), true, true);
        self.small = ui::f(10.max((c * 0.95) as i32));
        self.tiny = ui::f(9.max((c * 0.8) as i32));
        self.big = ui::fb(20.max((c * 2.0) as i32));
    }

    fn new_game(&mut self) {
        self.level = 1;
        self.b.score = 0;
        self.lives = 3;
        self.extra_awarded = false;
        self.b.game_over = false;
        self.fruit_history.clear();
        self.build_level();
        self.reset_positions(2.2);
    }

    fn build_level(&mut self) {
        self.grid = MAZE.iter().map(|r| r.as_bytes().to_vec()).collect();
        self.pellets = vec![vec![0u8; COLS as usize]; ROWS as usize];
        let mut total = 0;
        for r in 0..ROWS {
            for c in 0..COLS {
                if CLEAR_TILES.contains(&(c, r)) {
                    continue;
                }
                match self.grid[r as usize][c as usize] {
                    b'.' => {
                        self.pellets[r as usize][c as usize] = 1;
                        total += 1;
                    }
                    b'o' => {
                        self.pellets[r as usize][c as usize] = 2;
                        total += 1;
                    }
                    _ => {}
                }
            }
        }
        self.dots_total = total;
        self.dots_eaten = 0;
        self.fruit = None;
        self.fruit_thresholds = vec![70, 170];
    }

    fn reset_positions(&mut self, ready_time: f64) {
        self.pac = Mover { tile: PAC_START, dir: LEFT, frac: 0.0 };
        self.pac_want = LEFT;
        self.pac_moving = false;
        self.pac_mouth = 0.2;
        self.ghosts = (0..4).map(Ghost::new).collect();
        self.play_timer = 0.0;
        self.phase = 0;
        self.mode_timer = SCHEDULE[0].0;
        self.fright_timer = 0.0;
        self.ghost_combo = 0;
        self.fright_eaten = 0;
        self.freeze = 0.0;
        self.state = State::Ready;
        self.ready_timer = ready_time;
        self.dying_timer = 0.0;
    }

    fn global_mode(&self) -> &'static str {
        SCHEDULE[self.phase].1
    }

    fn fright_time(&self) -> f64 {
        1.0f64.max(DIFFS[self.diff].fright - (self.level - 1) as f64 * 0.4)
    }

    fn level_factor(&self) -> f64 {
        1.28f64.min(1.0 + (self.level - 1) as f64 * 0.035)
    }

    fn build_setup_layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let bw = 420.min(w - 60);
        let top = 150.max((h as f64 * 0.30) as i32);
        let l = cx - py::fdiv32(bw, 2);
        self.diff_panel = Rect::new(l, top, bw, 60);
        self.diff_left = Rect::new(l, top, 42, 60);
        self.diff_right = Rect::new(self.diff_panel.right() - 42, top, 42, 60);
        self.start_rect = Rect::new(cx - 95, top + 88, 190, 52);
    }

    fn cycle_diff(&mut self, step: i64) {
        self.diff = py::pmod(self.diff as i64 + step, 3) as usize;
        self.b.set_opt("pacman", "difficulty", (self.diff as i64).into());
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

    fn dir_from_key(&self, k: &str) -> Option<Dir> {
        if self.b.is_action(k, "up", None) || k == "Up" {
            Some(UP)
        } else if self.b.is_action(k, "down", None) || k == "Down" {
            Some(DOWN)
        } else if self.b.is_action(k, "left", None) || k == "Left" {
            Some(LEFT)
        } else if self.b.is_action(k, "right", None) || k == "Right" {
            Some(RIGHT)
        } else {
            None
        }
    }

    fn wrapt(t: (i32, i32)) -> (i32, i32) {
        (py::pmod32(t.0, COLS), t.1)
    }

    fn cell_at(&self, c: i32, r: i32) -> u8 {
        self.grid[r as usize][py::pmod32(c, COLS) as usize]
    }

    fn pac_open(&self, t: (i32, i32)) -> bool {
        if t.1 < 0 || t.1 >= ROWS {
            return false;
        }
        let ch = self.cell_at(t.0, t.1);
        ch != b'#' && ch != b'='
    }

    fn ghost_open(&self, g: &Ghost, t: (i32, i32)) -> bool {
        if t.1 < 0 || t.1 >= ROWS {
            return false;
        }
        match self.cell_at(t.0, t.1) {
            b'#' => false,
            b'=' => matches!(g.state, GState::Leaving | GState::Eyes | GState::Entering),
            _ => true,
        }
    }

    fn update_modes(&mut self, dt: f64) {
        if self.fright_timer > 0.0 {
            self.fright_timer -= dt;
            if self.fright_timer <= 0.0 {
                for g in self.ghosts.iter_mut() {
                    if g.state == GState::Maze {
                        g.frightened = false;
                    }
                }
            }
            return;
        }
        self.mode_timer -= dt;
        if self.mode_timer <= 0.0 && self.phase < SCHEDULE.len() - 1 {
            self.phase += 1;
            self.mode_timer = SCHEDULE[self.phase].0;
            for g in self.ghosts.iter_mut() {
                if g.state == GState::Maze && !g.frightened {
                    reverse(&mut g.m);
                }
            }
        }
    }

    fn release_ghosts(&mut self) {
        for g in self.ghosts.iter_mut() {
            if g.state == GState::House && self.play_timer >= g.release_at {
                g.state = GState::Leaving;
                g.m.tile = g.home;
                g.m.frac = 0.0;
                g.m.dir = UP;
            }
        }
    }

    fn move_pac(&mut self, dt: f64) {
        let mut dist = SPD_PAC * self.level_factor() * dt;
        while dist > 1e-9 {
            if self.pac.frac <= 1e-9 {
                if self.pac_open(add(self.pac.tile, self.pac_want)) {
                    self.pac.dir = self.pac_want;
                }
                if !self.pac_open(add(self.pac.tile, self.pac.dir)) {
                    self.pac_moving = false;
                    break;
                }
                self.pac_moving = true;
            }
            let step = dist.min(1.0 - self.pac.frac);
            self.pac.frac += step;
            dist -= step;
            if self.pac.frac >= 1.0 - 1e-9 {
                self.pac.tile = Self::wrapt(add(self.pac.tile, self.pac.dir));
                self.pac.frac = 0.0;
                self.pac_eat(self.pac.tile);
            }
        }
    }

    fn pac_eat(&mut self, tile: (i32, i32)) {
        let (c, r) = (tile.0 as usize, tile.1 as usize);
        match self.pellets[r][c] {
            1 => {
                self.pellets[r][c] = 0;
                self.b.score += 10;
                self.dots_eaten += 1;
                if self.dots_eaten % 2 == 0 {
                    self.b.play_sound("eat");
                }
            }
            2 => {
                self.pellets[r][c] = 0;
                self.b.score += 50;
                self.dots_eaten += 1;
                self.frighten();
                self.b.play_sound("powerup");
            }
            _ => {}
        }
        if self.fruit.is_some() && (tile == (13, 17) || tile == (14, 17)) {
            self.eat_fruit();
        }
    }

    fn anim_pac(&mut self, force_move: bool) {
        self.pac_mouth = if force_move { (self.anim_t * 11.0).sin().abs() * 0.92 } else { 0.18 };
    }

    fn ghost_speed(&self, g: &Ghost) -> f64 {
        match g.state {
            GState::Eyes | GState::Entering => SPD_EYES,
            GState::House | GState::Leaving => SPD_HOUSE,
            GState::Maze => {
                if g.frightened {
                    SPD_FRIGHT
                } else if self.cell_at(g.m.tile.0, g.m.tile.1) == b't' {
                    SPD_TUNNEL * self.level_factor()
                } else {
                    SPD_GHOST * self.level_factor() * DIFFS[self.diff].gspeed
                }
            }
        }
    }

    fn move_ghost(&mut self, i: usize, dt: f64) {
        let mut dist = self.ghost_speed(&self.ghosts[i]) * dt;
        while dist > 1e-9 {
            if self.ghosts[i].m.frac <= 1e-9 {
                self.ghost_decide(i);
            }
            let g = &mut self.ghosts[i];
            let step = dist.min(1.0 - g.m.frac);
            g.m.frac += step;
            dist -= step;
            if g.m.frac >= 1.0 - 1e-9 {
                g.m.tile = Self::wrapt(add(g.m.tile, g.m.dir));
                g.m.frac = 0.0;
                self.ghost_arrive(i);
            }
        }
    }

    fn ghost_decide(&mut self, i: usize) {
        let g = self.ghosts[i];
        if g.frightened && g.state == GState::Maze {
            let opts: Vec<Dir> = ORDER.iter().copied().filter(|&d| d != opp(g.m.dir) && self.ghost_open(&g, add(g.m.tile, d))).collect();
            self.ghosts[i].m.dir = if opts.is_empty() { opp(g.m.dir) } else { py::choice(&opts) };
            return;
        }
        let target = self.ghost_target(&g);
        let (mut best, mut bd) = (None, i64::MAX);
        for d in ORDER {
            if d == opp(g.m.dir) {
                continue;
            }
            let nt = add(g.m.tile, d);
            if !self.ghost_open(&g, nt) {
                continue;
            }
            let dd = dist2(Self::wrapt(nt), target);
            if dd < bd {
                bd = dd;
                best = Some(d);
            }
        }
        self.ghosts[i].m.dir = best.unwrap_or(opp(g.m.dir));
    }

    fn ghost_target(&self, g: &Ghost) -> (i32, i32) {
        match g.state {
            GState::Leaving | GState::Eyes => return DOOR_EXIT,
            GState::Entering => return g.home,
            _ => {}
        }
        if self.global_mode() == "scatter" {
            return g.scatter;
        }
        let pac = &self.pac;
        match GHOST_NAMES[g.name] {
            "blinky" => pac.tile,
            "pinky" => add(pac.tile, (pac.dir.0 * 4, pac.dir.1 * 4)),
            "inky" => {
                let p2 = add(pac.tile, (pac.dir.0 * 2, pac.dir.1 * 2));
                let bl = self.ghosts[0].m.tile;
                (2 * p2.0 - bl.0, 2 * p2.1 - bl.1)
            }
            _ => {
                if dist2(g.m.tile, pac.tile) >= 64 {
                    pac.tile
                } else {
                    g.scatter
                }
            }
        }
    }

    fn ghost_arrive(&mut self, i: usize) {
        let ft = self.fright_timer;
        let pt = self.play_timer;
        let g = &mut self.ghosts[i];
        let (c, r) = g.m.tile;
        if g.state == GState::Leaving && r == 11 && (c == 13 || c == 14) {
            g.state = GState::Maze;
            g.frightened = ft > 0.0;
        } else if g.state == GState::Eyes && r == 11 && (c == 13 || c == 14) {
            g.state = GState::Entering;
        } else if g.state == GState::Entering && g.m.tile == g.home {
            g.state = GState::House;
            g.m.dir = UP;
            g.release_at = pt + 1.0;
        }
    }

    fn frighten(&mut self) {
        self.fright_timer = self.fright_time();
        self.ghost_combo = 0;
        self.fright_eaten = 0;
        for g in self.ghosts.iter_mut() {
            if g.state == GState::Maze {
                g.frightened = true;
                reverse(&mut g.m);
            }
        }
    }

    fn pix(&self, m: &Mover, bob: f64) -> (f64, f64) {
        let c = self.cell as f64;
        let x = self.ox as f64 + (m.tile.0 as f64 + 0.5 + m.dir.0 as f64 * m.frac) * c;
        let y = self.oy as f64 + (m.tile.1 as f64 + 0.5 + m.dir.1 as f64 * m.frac + bob) * c;
        (x, y)
    }

    fn collisions(&mut self) {
        let (px, py_) = self.pix(&self.pac, 0.0);
        for i in 0..self.ghosts.len() {
            if self.ghosts[i].state != GState::Maze {
                continue;
            }
            let (gx, gy) = self.pix(&self.ghosts[i].m, 0.0);
            if py::hypot(px - gx, py_ - gy) < 0.55 * self.cell as f64 {
                if self.ghosts[i].frightened {
                    self.eat_ghost(i, gx, gy);
                } else {
                    self.pac_die();
                    return;
                }
            }
        }
    }

    fn eat_ghost(&mut self, i: usize, gx: f64, gy: f64) {
        let pts = 200 * 2i64.pow(self.ghost_combo);
        self.ghost_combo = (self.ghost_combo + 1).min(3);
        self.fright_eaten += 1;
        if self.fright_eaten >= 4 {
            self.b.ach_event("pac_ghosts", None);
        }
        self.b.score += pts;
        self.ghosts[i].state = GState::Eyes;
        self.ghosts[i].frightened = false;
        self.popup(gx, gy, pts.to_string(), rgb(120, 230, 255));
        self.freeze = 0.5;
        self.b.play_sound("point");
        self.b.rumble(80);
    }

    fn pac_die(&mut self) {
        self.state = State::Dying;
        self.dying_timer = 1.7;
        self.b.play_sound("explode");
        self.b.rumble(300);
    }

    fn after_death(&mut self) {
        self.lives -= 1;
        if self.lives <= 0 {
            self.state = State::GameOver;
            self.b.game_over = true;
            self.highscore = self.highscore.max(self.b.score);
            self.b.play_sound("gameover");
        } else {
            self.reset_positions(1.8);
        }
    }

    fn check_extra_life(&mut self) {
        if !self.extra_awarded && self.b.score >= 10000 {
            self.extra_awarded = true;
            self.lives += 1;
            self.b.play_sound("level");
        }
    }

    fn update_fruit(&mut self, dt: f64) {
        if let Some(f) = self.fruit.as_mut() {
            f.t -= dt;
            if f.t <= 0.0 {
                self.fruit = None;
            }
        } else if !self.fruit_thresholds.is_empty() && self.dots_eaten >= self.fruit_thresholds[0] {
            self.fruit_thresholds.remove(0);
            let (pts, col) = FRUIT_TABLE[((self.level - 1) as usize).min(FRUIT_TABLE.len() - 1)];
            self.fruit = Some(Fruit { pts, col, t: 9.5 });
        }
    }

    fn eat_fruit(&mut self) {
        let Some(f) = self.fruit else { return };
        self.b.score += f.pts;
        let c = self.cell as f64;
        let gx = self.ox as f64 + 13.5 * c;
        let gy = self.oy as f64 + 17.5 * c;
        self.popup(gx, gy, f.pts.to_string(), f.col);
        self.fruit_history.push(f.col);
        self.fruit = None;
        self.b.play_sound("point");
    }

    fn popup(&mut self, x: f64, y: f64, text: String, col: Color) {
        self.popups.push(Popup { x, y, text, col, t: 1.4, t0: 1.4 });
    }

    fn update_popups(&mut self, dt: f64) {
        for p in self.popups.iter_mut() {
            p.t -= dt;
            p.y -= dt * 12.0;
        }
        self.popups.retain(|p| p.t > 0.0);
    }

    fn is_wall(&self, col: i32, row: i32) -> bool {
        if row < 0 || row >= ROWS || col < 0 || col >= COLS {
            return true;
        }
        self.grid[row as usize][col as usize] == b'#'
    }

    fn draw_maze(&self, s: &mut Surface, flash: bool) {
        let c = self.cell;
        let cf = c as f64;
        let col_line = if flash { COL_WALL_HI } else { COL_WALL };
        let w = 2.max((cf * 0.16) as i32);
        let m = cf * 0.28;
        for r in 0..ROWS {
            for cc in 0..COLS {
                let ch = self.grid[r as usize][cc as usize];
                let (rx, ry) = (self.ox + cc * c, self.oy + r * c);
                if ch != b'#' {
                    if ch == b'=' {
                        draw::rect(s, COL_DOOR, (rx, ry + c / 2 - 2, c, 4), 0);
                    }
                    continue;
                }
                let (rxf, ryf) = (rx as f64, ry as f64);
                let top = !self.is_wall(cc, r - 1);
                let bot = !self.is_wall(cc, r + 1);
                let lft = !self.is_wall(cc - 1, r);
                let rgt = !self.is_wall(cc + 1, r);
                if top {
                    draw::line(s, col_line, (rxf, ryf + m), (rxf + cf, ryf + m), w);
                }
                if bot {
                    draw::line(s, col_line, (rxf, ryf + cf - m), (rxf + cf, ryf + cf - m), w);
                }
                if lft {
                    draw::line(s, col_line, (rxf + m, ryf), (rxf + m, ryf + cf), w);
                }
                if rgt {
                    draw::line(s, col_line, (rxf + cf - m, ryf), (rxf + cf - m, ryf + cf), w);
                }
                let rad = w / 2;
                if top && lft {
                    draw::circle(s, col_line, ((rxf + m) as i32, (ryf + m) as i32), rad, 0);
                }
                if top && rgt {
                    draw::circle(s, col_line, ((rxf + cf - m) as i32, (ryf + m) as i32), rad, 0);
                }
                if bot && lft {
                    draw::circle(s, col_line, ((rxf + m) as i32, (ryf + cf - m) as i32), rad, 0);
                }
                if bot && rgt {
                    draw::circle(s, col_line, ((rxf + cf - m) as i32, (ryf + cf - m) as i32), rad, 0);
                }
            }
        }
    }

    fn draw_pellets(&self, s: &mut Surface) {
        let c = self.cell;
        let pr = 1.max((c as f64 * 0.12) as i32);
        let power_on = (self.anim_t * 6.0) as i64 % 2 == 0;
        for r in 0..ROWS {
            for col in 0..COLS {
                let v = self.pellets[r as usize][col as usize];
                let (cx, cy) = (self.ox + col * c + c / 2, self.oy + r * c + c / 2);
                if v == 1 {
                    draw::circle(s, COL_PILL, (cx, cy), pr, 0);
                } else if v == 2 && power_on {
                    draw::circle(s, COL_POWER, (cx, cy), 3.max((c as f64 * 0.34) as i32), 0);
                }
            }
        }
    }

    fn fruit_icon(s: &mut Surface, cx: f64, cy: f64, r: f64, col: Color) {
        draw::circle(s, col, (cx as i32, (cy + r * 0.15) as i32), (r * 0.6) as i32, 0);
        draw::line(s, rgb(90, 200, 90), (cx as i32, (cy - r * 0.4) as i32), ((cx + r * 0.35) as i32, (cy - r * 0.7) as i32), 2);
        draw::circle(s, rgb(255, 255, 255), ((cx - r * 0.2) as i32, cy as i32), 1.max((r * 0.12) as i32), 0);
    }

    fn draw_pac(&self, s: &mut Surface) {
        let r = self.cell as f64 * 0.46;
        let (px, py_) = self.pix(&self.pac, 0.0);
        let mouth = if self.state == State::Dying {
            let prog = 1.0 - self.dying_timer / 1.7;
            if prog >= 0.98 {
                return;
            }
            0.15 + prog * (py::PI - 0.15)
        } else {
            self.pac_mouth
        };
        let ang = (self.pac.dir.1 as f64).atan2(self.pac.dir.0 as f64);
        draw::circle(s, COL_PAC, (px as i32, py_ as i32), r as i32, 0);
        if mouth > 0.03 {
            let mut pts = vec![(px, py_)];
            let steps = 12;
            for i in 0..=steps {
                let a = ang - mouth + (2.0 * mouth) * i as f64 / steps as f64;
                pts.push((px + a.cos() * r * 1.25, py_ + a.sin() * r * 1.25));
            }
            draw::polygon(s, COL_BG, &pts, 0);
        }
    }

    fn draw_ghost(&self, s: &mut Surface, g: &Ghost) {
        let r = self.cell as f64 * 0.46;
        let bob = if g.state == GState::House { 0.12 * g.bob_t.sin() } else { 0.0 };
        let (px, py_) = self.pix(&g.m, bob);
        let eyes_only = matches!(g.state, GState::Eyes | GState::Entering);
        if !eyes_only {
            let mut col = g.color;
            if g.frightened {
                let ending = self.fright_timer < 2.2f64.min(self.fright_time() * 0.4);
                col = if ending && (self.anim_t * 8.0) as i64 % 2 == 0 { COL_FRIGHT_END } else { COL_FRIGHT };
            }
            Self::ghost_body(s, px, py_, r, col);
        }
        if g.frightened && !eyes_only {
            Self::fright_face(s, px, py_, r);
        } else {
            Self::ghost_eyes(s, px, py_, r, g.m.dir);
        }
    }

    fn ghost_body(s: &mut Surface, cx: f64, cy: f64, r: f64, col: Color) {
        draw::circle(s, col, (cx as i32, (cy - r * 0.12) as i32), r as i32, 0);
        draw::rect(s, col, ((cx - r) as i32, (cy - r * 0.12) as i32, (2.0 * r) as i32, (r * 1.05) as i32), 0);
        let foot_y = cy - r * 0.12 + r * 1.05;
        let n = 4;
        let fw = 2.0 * r / n as f64;
        for i in 0..n {
            let x0 = cx - r + i as f64 * fw;
            draw::polygon(s, COL_BG, &[(x0, foot_y), (x0 + fw / 2.0, foot_y - r * 0.42), (x0 + fw, foot_y)], 0);
        }
    }

    fn ghost_eyes(s: &mut Surface, cx: f64, cy: f64, r: f64, d: Dir) {
        for sx in [-1.0, 1.0] {
            let ex = cx + sx * r * 0.42;
            let ey = cy - r * 0.18;
            draw::circle(s, rgb(255, 255, 255), (ex as i32, ey as i32), (r * 0.34) as i32, 0);
            let pupil = (ex + d.0 as f64 * r * 0.16, ey + d.1 as f64 * r * 0.16);
            draw::circle(s, rgb(40, 50, 190), (pupil.0 as i32, pupil.1 as i32), (r * 0.17) as i32, 0);
        }
    }

    fn fright_face(s: &mut Surface, cx: f64, cy: f64, r: f64) {
        for sx in [-1.0, 1.0] {
            draw::circle(s, rgb(255, 220, 230), ((cx + sx * r * 0.38) as i32, (cy - r * 0.12) as i32), 1.max((r * 0.14) as i32), 0);
        }
        let y = cy + r * 0.4;
        let pts: Vec<(f64, f64)> = (0..7)
            .map(|i| {
                let x = cx - r * 0.6 + (r * 1.2) * i as f64 / 6.0;
                (x, y + if i % 2 != 0 { r * 0.14 } else { -r * 0.14 })
            })
            .collect();
        draw::lines(s, rgb(255, 220, 230), false, &pts, 2);
    }

    fn blit_panel(&self, s: &mut Surface, rect: Rect, border: i32) {
        let p = ui::pal();
        let mut surf = pg::surface_alpha(rect.w, rect.h);
        draw::rect_r(&mut surf, pg::alpha(p.panel, 222), (0, 0, rect.w, rect.h), 0, 10);
        s.bl(&surf, (rect.x, rect.y));
        draw::rect_r(s, self.b.accent, rect, border, 10);
    }

    fn draw_hud(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        self.highscore = self.highscore.max(self.b.score);
        let lab = self.tiny.render(&t("pac.1up"), true, self.b.accent);
        let score = self.digit.render(&self.b.score.to_string(), true, p.text);
        let hlab = self.tiny.render(&t("pac.high"), true, self.b.accent);
        let himg = self.digit.render(&self.highscore.to_string(), true, p.text);
        let lv = self.small.render(&tf("pac.level", &[("n", &self.level.to_string())]), true, p.text_dim);
        let bar = Rect::new(6, 2, w - 12, lab.h() + score.h() + 8);
        self.blit_panel(s, bar, 1);
        s.bl(&lab, (14, 5));
        s.bl(&score, (14, 5 + lab.h()));
        s.blr(&hlab, hlab.get_rect().midtop_at((w / 2, 5)));
        s.blr(&himg, himg.get_rect().midtop_at((w / 2, 5 + hlab.h())));
        s.blr(&lv, lv.get_rect().topright_at((w - 14, 7)));
        let c = self.cell as f64;
        let y = h - (c * 1.2) as i32;
        for i in 0..0.max(self.lives - 1) as i32 {
            Self::life_icon(s, (16 + i * (c * 1.4) as i32) as f64, y as f64, c * 0.5);
        }
        let n = self.fruit_history.len();
        for (i, col) in self.fruit_history[n.saturating_sub(6)..].iter().enumerate() {
            Self::fruit_icon(s, (w - 16 - i as i32 * (c * 1.3) as i32) as f64, y as f64, c * 0.6, *col);
        }
    }

    fn life_icon(s: &mut Surface, x: f64, y: f64, r: f64) {
        draw::circle(s, COL_PAC, (x as i32, y as i32), r as i32, 0);
        let mut pts = vec![(x, y)];
        for i in 0..7 {
            let a = py::PI - 0.5 + 1.0 * i as f64 / 6.0;
            pts.push((x + a.cos() * r * 1.3, y + a.sin() * r * 1.3));
        }
        draw::polygon(s, COL_BG, &pts, 0);
    }

    fn draw_overlays(&self, s: &mut Surface) {
        let p = ui::pal();
        let c = self.cell as f64;
        let cx = self.ox + self.maze_w / 2;
        let cy = self.oy + (17.2 * c) as i32;
        if self.state == State::Ready {
            let img = self.font.render(&t("pac.ready"), true, self.b.accent);
            let bx = img.get_rect().center_at((cx, cy)).inflate((c * 1.8) as i32, (c * 0.9) as i32);
            self.blit_panel(s, bx, 2);
            s.blr(&img, img.get_rect().center_at((cx, cy)));
        } else if self.state == State::GameOver {
            let img = self.big.render(&t("common.game_over"), true, p.red);
            let hint = self.small.render(&t("pac.restart_hint"), true, p.text);
            let pad = (c * 0.8) as i32;
            let bw = img.w().max(hint.w()) + 2 * pad + self.cell;
            let top = cy - img.h() / 2 - pad;
            let bottom = cy + (c * 2.2) as i32 + hint.h() / 2 + pad;
            let bx = Rect::new(0, 0, bw, bottom - top).midtop_at((cx, top));
            self.blit_panel(s, bx, 2);
            s.blr(&img, img.get_rect().center_at((cx, cy)));
            s.blr(&hint, hint.get_rect().center_at((cx, cy + (c * 2.2) as i32)));
        }
    }

    fn draw_setup(&self) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let mut s = self.b.sf();
        ui::draw_background(&mut s, w, h);
        ui::draw_title_ex(&mut s, w, "PAC-MAN", Some(&t("snake.singleplayer")), 52, None, None, Some(self.b.accent));
        let d = &DIFFS[self.diff];
        draw::rect_r(&mut s, p.panel, self.diff_panel, 0, 10);
        draw::rect_r(&mut s, self.b.accent, self.diff_panel, 2, 10);
        let name = self.font.render(&format!("{}:  {}", t("pac.difficulty"), t(&format!("pac.diff.{}", d.key))), true, p.text);
        s.blr(&name, name.get_rect().center_at((self.diff_panel.centerx(), self.diff_panel.y + 22)));
        let note = self.tiny.render(&t("pac.diff_note"), true, p.text_dim);
        s.blr(&note, note.get_rect().center_at((self.diff_panel.centerx(), self.diff_panel.y + 44)));
        for (rect, sym) in [(self.diff_left, "<"), (self.diff_right, ">")] {
            let arr = self.big.render(sym, true, self.b.accent);
            s.blr(&arr, arr.get_rect().center_at(rect.center()));
        }
        ui::draw_button_ex(&mut s, self.start_rect, &t("common.start"), &self.font, true, None, None, Some(p.green));
        let hint = self.small.render(&t("pac.setup_hint"), true, p.text_dim);
        s.blr(&hint, hint.get_rect().center_at((w / 2, h - 38)));
        let ctrl = self.tiny.render(&t("pac.controls_hint"), true, p.green);
        s.blr(&ctrl, ctrl.get_rect().center_at((w / 2, h - 16)));
    }
}

/// `_reverse(e)` - Richtung umkehren (auch mitten in einer Kachel).
fn reverse(e: &mut Mover) {
    if e.frac > 1e-6 {
        e.tile = PacmanGame::wrapt(add(e.tile, e.dir));
        e.frac = 1.0 - e.frac;
    }
    e.dir = opp(e.dir);
}

impl Game for PacmanGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        self.b.score = 0;
        self.b.game_over = false;
        self.diff = self.b.opt_i64("pacman", "difficulty", 0).clamp(0, 2) as usize;
        self.layout();
        self.highscore = highscore::get("pacman");
        self.anim_t = 0.0;
        self.popups.clear();
        self.fruit_history.clear();
        self.build_setup_layout();
        self.new_game();
        self.state = State::Setup;
    }

    fn on_surface_changed(&mut self) {
        self.layout();
        self.build_setup_layout();
        self.popups.clear();
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if self.state == State::Setup {
            self.handle_setup(ev);
            return;
        }
        if ev.kind != Kind::KeyDown {
            return;
        }
        if self.state == State::GameOver {
            if ev.key_in(&["Return", "space"]) {
                self.new_game();
            } else if ev.key_in(&["s", "S"]) {
                self.state = State::Setup;
                self.b.play_sound("click");
            }
            return;
        }
        if let Some(d) = self.dir_from_key(&ev.key) {
            if matches!(self.state, State::Ready | State::Play) {
                self.pac_want = d;
                if d == opp(self.pac.dir) {
                    reverse(&mut self.pac);
                }
            }
        }
    }

    fn update(&mut self, dt: f64) {
        self.anim_t += dt;
        self.update_popups(dt);
        match self.state {
            State::Setup | State::GameOver => return,
            State::Ready => {
                self.ready_timer -= dt;
                self.anim_pac(false);
                if self.ready_timer <= 0.0 {
                    self.state = State::Play;
                }
                return;
            }
            State::LevelClear => {
                self.levelclear_timer -= dt;
                if self.levelclear_timer <= 0.0 {
                    self.level += 1;
                    self.build_level();
                    self.reset_positions(1.6);
                }
                return;
            }
            State::Dying => {
                self.dying_timer -= dt;
                if self.dying_timer <= 0.0 {
                    self.after_death();
                }
                return;
            }
            State::Play => {}
        }
        if self.freeze > 0.0 {
            self.freeze -= dt;
            return;
        }
        self.play_timer += dt;
        self.update_modes(dt);
        self.release_ghosts();
        self.move_pac(dt);
        self.anim_pac(self.pac_moving);
        for i in 0..self.ghosts.len() {
            if self.ghosts[i].state == GState::House {
                self.ghosts[i].bob_t += dt * 3.0;
            } else {
                self.move_ghost(i, dt);
            }
        }
        self.collisions();
        self.update_fruit(dt);
        self.check_extra_life();
        if self.dots_eaten >= self.dots_total {
            self.state = State::LevelClear;
            self.levelclear_timer = 1.8;
            self.b.ach_event("pacman_clear", None);
            self.b.play_sound("win");
        }
    }

    fn draw(&mut self) {
        if self.state == State::Setup {
            self.draw_setup();
            return;
        }
        let mut s = self.b.sf();
        s.fill_c(COL_BG);
        let flash = self.state == State::LevelClear && (self.anim_t * 6.0) as i64 % 2 == 0;
        self.draw_maze(&mut s, flash);
        if !flash {
            self.draw_pellets(&mut s);
        }
        if let Some(f) = self.fruit {
            let c = self.cell as f64;
            Self::fruit_icon(&mut s, self.ox as f64 + 13.5 * c, self.oy as f64 + 17.5 * c, c * 0.7, f.col);
        }
        if self.state != State::Dying {
            for g in &self.ghosts {
                self.draw_ghost(&mut s, g);
            }
        }
        self.draw_pac(&mut s);
        for p in &self.popups {
            let a = 0.max(255.min((255.0 * p.t / p.t0) as i32));
            let mut img = self.tiny.render(&p.text, true, p.col);
            img.set_alpha(Some(a as u8));
            s.blr(&img, img.get_rect().center_at((p.x as i32, p.y as i32)));
        }
        self.draw_hud(&mut s);
        self.draw_overlays(&mut s);
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "pacman",
    class_name: "PacmanGame",
    supports_multiplayer: false,
    modes: &[],
    make: PacmanGame::new,
};
