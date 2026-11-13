//! tetris_core.rs
//! ==============
//! Das Regelwerk von Tetris ohne Grafik - uebertragen aus
//! `games/tetris_core.py`.
//!
//! 10 x 20 Feld mit 20 verborgenen Pufferzeilen, jede Zeile eine 10-Bit-
//! Maske; 7-Bag ueber seedrand, SRS mit echten Kicks, Lock Delay mit
//! Extended Placement, T-Spins, Back-to-Back, Combo, Perfect Clear,
//! Guideline-Gravitation und Angriffstabelle mit Aufrechnen.

use std::sync::OnceLock;

use pgz_core::seedrand::Rand;

/// Spalten.
pub const COLS: i32 = 10;
/// Sichtbare Zeilen.
pub const VISIBLE: i32 = 20;
/// Verborgene Pufferzeilen.
pub const BUFFER: i32 = 20;
/// Alle Zeilen.
pub const ROWS: i32 = VISIBLE + BUFFER;
/// Volle Zeile.
pub const FULL: u32 = (1 << COLS) - 1;
/// Steinarten.
pub const KINDS: [char; 7] = ['I', 'J', 'L', 'O', 'S', 'T', 'Z'];

const SPAWN_X: i32 = 3;
const SPAWN_Y: i32 = BUFFER - 2;
/// Sekunden bis zum Einrasten.
pub const LOCK_DELAY: f64 = 0.5;
const MAX_RESETS: i32 = 15;
const SOFT_FACTOR: f64 = 20.0;
const SOFT_MIN_RATE: f64 = 20.0;
/// Muell steigt fruehestens nach so vielen Sekunden.
pub const GARBAGE_DELAY: f64 = 0.5;
const GARBAGE_CAP: i64 = 8;

/// Index einer Steinart in `KINDS`.
pub fn kind_index(k: char) -> usize {
    KINDS.iter().position(|c| *c == k).unwrap_or(0)
}

/// Vorberechnete Daten eines Steins in einem Drehzustand.
#[derive(Clone, Debug)]
pub struct Shape {
    /// Zellen (sortiert nach (y, x)).
    pub cells: Vec<(i32, i32)>,
    /// Ausdehnung.
    pub minx: i32,
    /// Ausdehnung.
    pub maxx: i32,
    /// Ausdehnung.
    pub miny: i32,
    /// Ausdehnung.
    pub maxy: i32,
    /// (dy, Maske bei x = 0) je belegter Box-Zeile.
    pub rows: Vec<(i32, u32)>,
    /// Unterste belegte Box-Zeile je Box-Spalte.
    pub bottom: Vec<(i32, i32)>,
}

impl Shape {
    fn new(cells: &[(i32, i32)]) -> Shape {
        let mut sorted = cells.to_vec();
        sorted.sort_by_key(|c| (c.1, c.0));
        let xs: Vec<i32> = cells.iter().map(|c| c.0).collect();
        let ys: Vec<i32> = cells.iter().map(|c| c.1).collect();
        let mut masks: Vec<(i32, u32)> = Vec::new();
        for &(cx, cy) in cells {
            match masks.iter_mut().find(|(y, _)| *y == cy) {
                Some(e) => e.1 |= 1 << cx,
                None => masks.push((cy, 1 << cx)),
            }
        }
        masks.sort_by_key(|m| m.0);
        let mut bottom: Vec<(i32, i32)> = Vec::new();
        for &(cx, cy) in cells {
            match bottom.iter_mut().find(|(x, _)| *x == cx) {
                Some(e) => e.1 = e.1.max(cy),
                None => bottom.push((cx, cy)),
            }
        }
        bottom.sort_by_key(|b| b.0);
        Shape {
            cells: sorted,
            minx: *xs.iter().min().unwrap_or(&0),
            maxx: *xs.iter().max().unwrap_or(&0),
            miny: *ys.iter().min().unwrap_or(&0),
            maxy: *ys.iter().max().unwrap_or(&0),
            rows: masks,
            bottom,
        }
    }
}

fn base_cells(k: char) -> [(i32, i32); 4] {
    match k {
        'I' => [(0, 1), (1, 1), (2, 1), (3, 1)],
        'J' => [(0, 0), (0, 1), (1, 1), (2, 1)],
        'L' => [(2, 0), (0, 1), (1, 1), (2, 1)],
        'O' => [(1, 0), (2, 0), (1, 1), (2, 1)],
        'S' => [(1, 0), (2, 0), (0, 1), (1, 1)],
        'T' => [(1, 0), (0, 1), (1, 1), (2, 1)],
        _ => [(0, 0), (1, 0), (1, 1), (2, 1)],
    }
}

/// `SHAPES[kind][rot]`
pub fn shape(kind: char, rot: i32) -> &'static Shape {
    static S: OnceLock<Vec<Vec<Shape>>> = OnceLock::new();
    let all = S.get_or_init(|| {
        KINDS
            .iter()
            .map(|&k| {
                let base = base_cells(k);
                if k == 'O' {
                    let s = Shape::new(&base);
                    return vec![s.clone(), s.clone(), s.clone(), s];
                }
                let n = if k == 'I' { 4 } else { 3 };
                let mut states: Vec<Vec<(i32, i32)>> = vec![base.to_vec()];
                for _ in 0..3 {
                    let last = states.last().cloned().unwrap_or_default();
                    states.push(last.iter().map(|&(x, y)| (n - 1 - y, x)).collect());
                }
                states.iter().map(|s| Shape::new(s)).collect()
            })
            .collect()
    });
    &all[kind_index(kind)][rot.rem_euclid(4) as usize]
}

fn kicks(kind: char, from: i32, to: i32) -> [(i32, i32); 5] {
    if kind == 'I' {
        match (from, to) {
            (0, 1) => [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
            (1, 0) => [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
            (1, 2) => [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
            (2, 1) => [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
            (2, 3) => [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
            (3, 2) => [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
            (3, 0) => [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
            _ => [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
        }
    } else {
        match (from, to) {
            (0, 1) => [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
            (1, 0) => [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
            (1, 2) => [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
            (2, 1) => [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
            (2, 3) => [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
            (3, 2) => [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
            (3, 0) => [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
            _ => [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
        }
    }
}

fn t_front(rot: i32) -> (usize, usize) {
    match rot {
        0 => (0, 1),
        1 => (1, 2),
        2 => (2, 3),
        _ => (3, 0),
    }
}

fn score_table(spin: &str, lines: i64) -> i64 {
    match (spin, lines) {
        ("", 1) => 100,
        ("", 2) => 300,
        ("", 3) => 500,
        ("", 4) => 800,
        ("mini", 0) => 100,
        ("mini", 1) => 200,
        ("mini", 2) => 400,
        ("full", 0) => 400,
        ("full", 1) => 800,
        ("full", 2) => 1200,
        ("full", 3) => 1600,
        _ => 0,
    }
}

fn attack_table(spin: &str, lines: i64) -> i64 {
    match (spin, lines) {
        ("", 2) => 1,
        ("", 3) => 2,
        ("", 4) => 4,
        ("mini", 2) => 1,
        ("full", 1) => 2,
        ("full", 2) => 4,
        ("full", 3) => 6,
        _ => 0,
    }
}

fn pc_points(lines: i64) -> i64 {
    match lines {
        1 => 800,
        2 => 1200,
        3 => 1800,
        _ => 2000,
    }
}

const COMBO_ATTACK: [i64; 12] = [0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 4, 5];

/// `gravity_interval(level)` - Sekunden je Zeile Fall.
pub fn gravity_interval(level: i64) -> f64 {
    let lv = 1.max(level.min(20));
    let base = 0.8 - (lv - 1) as f64 * 0.007;
    let mut value = 1.0;
    for _ in 0..lv - 1 {
        value *= base;
    }
    value
}

/// `soft_interval(level)`
pub fn soft_interval(level: i64) -> f64 {
    let g = gravity_interval(level);
    (g / SOFT_FACTOR).min(1.0 / SOFT_MIN_RATE)
}

/// `collides(rows, kind, rot, x, y)`
pub fn collides(rows: &[u32], kind: char, rot: i32, x: i32, y: i32) -> bool {
    let sh = shape(kind, rot);
    if x + sh.minx < 0 || x + sh.maxx >= COLS || y + sh.maxy >= ROWS || y + sh.miny < 0 {
        return true;
    }
    for &(dy, m) in &sh.rows {
        let mm = if x >= 0 { m << x } else { m >> -x };
        if rows[(y + dy) as usize] & mm != 0 {
            return true;
        }
    }
    false
}

/// `try_rotate(rows, kind, rot, x, y, d)` - (Zustand, x, y, Kick-Index).
pub fn try_rotate(rows: &[u32], kind: char, rot: i32, x: i32, y: i32, d: i32) -> Option<(i32, i32, i32, i32)> {
    let nrot = (rot + d).rem_euclid(4);
    if kind == 'O' {
        return Some((nrot, x, y, 0));
    }
    for (i, (kx, ky)) in kicks(kind, rot, nrot).iter().enumerate() {
        let (nx, ny) = (x + kx, y - ky);
        if !collides(rows, kind, nrot, nx, ny) {
            return Some((nrot, nx, ny, i as i32));
        }
    }
    None
}

/// `drop_y(rows, kind, rot, x, y)`
pub fn drop_y(rows: &[u32], kind: char, rot: i32, x: i32, mut y: i32) -> i32 {
    while !collides(rows, kind, rot, x, y + 1) {
        y += 1;
    }
    y
}

/// Steinfolge: 7-Bag oder (im Replay) die aufgezeichnete Vorschau.
pub enum Queue {
    /// `PieceQueue(seed)`
    Bag {
        /// Generator.
        rng: Rand,
        /// Wartende Steine.
        items: Vec<char>,
    },
    /// `_RepQueue`
    Rep(Vec<char>),
}

impl Queue {
    /// `PieceQueue(seed)`
    pub fn bag(seed: u32) -> Queue {
        Queue::Bag { rng: Rand::new(seed), items: Vec::new() }
    }

    fn fill(&mut self, n: usize) {
        if let Queue::Bag { rng, items } = self {
            while items.len() < n {
                let mut bag = KINDS.to_vec();
                rng.shuffle(&mut bag);
                items.extend(bag);
            }
        }
    }

    /// `pop()`
    pub fn pop(&mut self) -> char {
        match self {
            Queue::Bag { .. } => {
                self.fill(8);
                if let Queue::Bag { items, .. } = self {
                    return items.remove(0);
                }
                'I'
            }
            Queue::Rep(k) => *k.first().unwrap_or(&'I'),
        }
    }

    /// `peek(n)`
    pub fn peek(&mut self, n: usize) -> Vec<char> {
        match self {
            Queue::Bag { .. } => {
                self.fill(n.max(8));
                if let Queue::Bag { items, .. } = self {
                    return items[..n].to_vec();
                }
                Vec::new()
            }
            Queue::Rep(k) => k.iter().take(n).copied().collect(),
        }
    }
}

/// Ergebnis eines eingerasteten Steins.
#[derive(Clone, Debug)]
pub struct LockInfo {
    /// Steinart.
    pub kind: char,
    /// Abgeraeumte Zeilen.
    pub lines: i64,
    /// T-Spin: "mini" / "full".
    pub spin: Option<&'static str>,
    /// Back-to-Back-Bonus.
    pub b2b: bool,
    /// Combo-Zaehler.
    pub combo: i64,
    /// Perfect Clear.
    pub pc: bool,
    /// Punkte.
    pub points: i64,
    /// Angriff.
    pub attack: i64,
    /// Gesendet (nach Aufrechnen).
    pub sent: i64,
    /// Aufgerechnet.
    pub cancelled: i64,
    /// Abgeraeumte Zeilen (alte Indizes).
    pub rows: Vec<i32>,
    /// Inhalt der abgeraeumten Zeilen.
    pub cleared: Vec<[Option<char>; 10]>,
    /// Zellen des Steins.
    pub cells: Vec<(i32, i32)>,
    /// Hard-Drop-Hoehe.
    pub drop: i32,
}

/// Ereignisse fuer die Oberflaeche.
#[derive(Clone, Debug)]
pub enum Event {
    /// Eingerastet.
    Lock(LockInfo),
    /// Neues Level.
    Level(i64),
    /// Muell gestiegen.
    Garbage(i64),
    /// Hold benutzt.
    Hold(char),
    /// Feld verloren.
    Dead(&'static str),
}

/// Ein Spielfeld.
pub struct Board {
    /// Zeilenmasken.
    pub rows: Vec<u32>,
    /// Steinart je Zelle.
    pub cells: Vec<[Option<char>; 10]>,
    /// Steinfolge.
    pub queue: Queue,
    grng: Rand,
    /// Hold.
    pub hold_kind: Option<char>,
    /// Hold schon benutzt.
    pub hold_used: bool,
    start_level: i64,
    /// Level.
    pub level: i64,
    fixed_level: bool,
    /// Zeilen.
    pub lines: i64,
    /// Punkte.
    pub score: i64,
    /// Steine.
    pub pieces: i64,
    /// Combo.
    pub combo: i64,
    /// Back-to-Back.
    pub b2b: bool,
    /// Anzahl Tetris.
    pub tetrises: i64,
    /// Anzahl T-Spins.
    pub tspins: i64,
    /// Gesendeter Angriff.
    pub attack_sent: i64,
    /// Empfangener Muell.
    pub garbage_received: i64,
    /// Wartender Muell: (Zeilen, Lochspalte, Alter).
    pub pending: Vec<(i64, i64, f64)>,
    /// Verloren.
    pub dead: bool,
    /// Grund.
    pub dead_reason: Option<&'static str>,
    /// Ereignisse.
    pub events: Vec<Event>,
    /// Feldversion.
    pub version: i64,
    /// Steinnummer.
    pub piece_id: i64,
    /// Aktiver Stein vorhanden.
    pub active: bool,
    /// Steinart.
    pub kind: char,
    /// Drehung.
    pub rot: i32,
    /// Position.
    pub x: i32,
    /// Position.
    pub y: i32,
    last_rot: bool,
    last_kick: i32,
    /// Lock-Timer.
    pub lock_timer: f64,
    lock_resets: i32,
    fall_acc: f64,
    /// Tiefster Stand.
    pub lowest: i32,
}

impl Board {
    /// `Board(seed, level, fixed_level)`
    pub fn new(seed: u32, level: i64, fixed_level: bool) -> Board {
        let start = 1.max(level);
        let mut b = Board {
            rows: vec![0; ROWS as usize],
            cells: vec![[None; 10]; ROWS as usize],
            queue: Queue::bag(seed),
            grng: Rand::new(seed ^ 0x5F37_59DF),
            hold_kind: None,
            hold_used: false,
            start_level: start,
            level: start,
            fixed_level,
            lines: 0,
            score: 0,
            pieces: 0,
            combo: -1,
            b2b: false,
            tetrises: 0,
            tspins: 0,
            attack_sent: 0,
            garbage_received: 0,
            pending: Vec::new(),
            dead: false,
            dead_reason: None,
            events: Vec::new(),
            version: 0,
            piece_id: 0,
            active: false,
            kind: 'I',
            rot: 0,
            x: 0,
            y: 0,
            last_rot: false,
            last_kick: -1,
            lock_timer: 0.0,
            lock_resets: 0,
            fall_acc: 0.0,
            lowest: 0,
        };
        b.spawn(None);
        b
    }

    /// `spawn(kind)`
    pub fn spawn(&mut self, kind: Option<char>) -> bool {
        self.kind = match kind {
            Some(k) => k,
            None => self.queue.pop(),
        };
        self.rot = 0;
        self.x = SPAWN_X;
        self.y = SPAWN_Y;
        self.piece_id += 1;
        self.last_rot = false;
        self.last_kick = -1;
        self.lock_timer = 0.0;
        self.lock_resets = 0;
        self.fall_acc = 0.0;
        if collides(&self.rows, self.kind, 0, self.x, self.y) {
            self.active = false;
            self.die("blockout");
            return false;
        }
        self.active = true;
        if !collides(&self.rows, self.kind, 0, self.x, self.y + 1) {
            self.y += 1;
        }
        self.lowest = self.y;
        true
    }

    fn die(&mut self, reason: &'static str) {
        if !self.dead {
            self.dead = true;
            self.dead_reason = Some(reason);
            self.active = false;
            self.events.push(Event::Dead(reason));
        }
    }

    /// `cells_of(kind, rot, x, y)` fuer den aktiven Stein (y optional).
    pub fn cells_at(&self, y: Option<i32>) -> Vec<(i32, i32)> {
        let yy = y.unwrap_or(self.y);
        shape(self.kind, self.rot).cells.iter().map(|&(cx, cy)| (self.x + cx, yy + cy)).collect()
    }

    /// `ghost_y()`
    pub fn ghost_y(&self) -> i32 {
        drop_y(&self.rows, self.kind, self.rot, self.x, self.y)
    }

    /// `grounded()`
    pub fn grounded(&self) -> bool {
        collides(&self.rows, self.kind, self.rot, self.x, self.y + 1)
    }

    /// `pending_lines()`
    pub fn pending_lines(&self) -> i64 {
        self.pending.iter().map(|p| p.0).sum()
    }

    /// `stack_height()`
    pub fn stack_height(&self) -> i32 {
        for (i, r) in self.rows.iter().enumerate() {
            if *r != 0 {
                return ROWS - i as i32;
            }
        }
        0
    }

    /// `move(dx)`
    pub fn mv(&mut self, dx: i32) -> bool {
        if !self.active || collides(&self.rows, self.kind, self.rot, self.x + dx, self.y) {
            return false;
        }
        self.x += dx;
        self.last_rot = false;
        self.manipulated();
        true
    }

    /// `rotate(d)`
    pub fn rotate(&mut self, d: i32) -> bool {
        if !self.active {
            return false;
        }
        let Some((r, x, y, k)) = try_rotate(&self.rows, self.kind, self.rot, self.x, self.y, d) else {
            return false;
        };
        self.rot = r;
        self.x = x;
        self.y = y;
        self.last_kick = k;
        self.last_rot = true;
        self.manipulated();
        true
    }

    fn manipulated(&mut self) {
        if self.y > self.lowest {
            self.lowest = self.y;
            self.lock_resets = 0;
        }
        let on_ground = self.grounded();
        if on_ground || self.lock_timer > 0.0 {
            if self.lock_resets < MAX_RESETS {
                self.lock_resets += 1;
                self.lock_timer = 0.0;
            } else if on_ground {
                self.lock_timer = LOCK_DELAY;
            }
        }
    }

    /// `soft_step()`
    pub fn soft_step(&mut self) -> bool {
        if !self.active || self.grounded() {
            return false;
        }
        self.y += 1;
        self.score += 1;
        self.last_rot = false;
        self.fall_acc = 0.0;
        if self.y > self.lowest {
            self.lowest = self.y;
            self.lock_resets = 0;
        }
        true
    }

    /// `hard_drop()`
    pub fn hard_drop(&mut self) -> Option<LockInfo> {
        if !self.active {
            return None;
        }
        let gy = self.ghost_y();
        let d = gy - self.y;
        if d > 0 {
            self.y = gy;
            self.last_rot = false;
        }
        self.score += 2 * d as i64;
        self.lock(d)
    }

    /// `hold()`
    pub fn hold(&mut self) -> bool {
        if !self.active || self.hold_used {
            return false;
        }
        let cur = self.kind;
        let swap = self.hold_kind;
        self.hold_kind = Some(cur);
        self.spawn(swap);
        self.hold_used = true;
        self.events.push(Event::Hold(cur));
        true
    }

    /// `tick(dt, soft)`
    pub fn tick(&mut self, dt: f64, soft: bool) {
        for p in self.pending.iter_mut() {
            p.2 += dt;
        }
        if !self.active {
            return;
        }
        let interval = if soft { soft_interval(self.level) } else { gravity_interval(self.level) };
        if !self.grounded() {
            self.fall_acc += dt / interval;
            let mut n = self.fall_acc as i64;
            if n > 0 {
                self.fall_acc -= n as f64;
                let mut moved = 0;
                while n > 0 && !self.grounded() {
                    self.y += 1;
                    n -= 1;
                    moved += 1;
                }
                if moved != 0 {
                    self.last_rot = false;
                    if soft {
                        self.score += moved;
                    }
                    if self.y > self.lowest {
                        self.lowest = self.y;
                        self.lock_resets = 0;
                    }
                }
            }
        }
        if self.grounded() {
            self.fall_acc = 0.0;
            self.lock_timer += dt;
            if self.lock_timer >= LOCK_DELAY {
                self.lock(0);
            }
        } else {
            self.lock_timer = 0.0;
        }
    }

    fn occupied(&self, x: i32, y: i32) -> bool {
        if x < 0 || x >= COLS || y < 0 || y >= ROWS {
            return true;
        }
        (self.rows[y as usize] >> x) & 1 != 0
    }

    fn spin_type(&self) -> Option<&'static str> {
        if self.kind != 'T' || !self.last_rot {
            return None;
        }
        let (cx, cy) = (self.x + 1, self.y + 1);
        let occ = [self.occupied(cx - 1, cy - 1), self.occupied(cx + 1, cy - 1), self.occupied(cx + 1, cy + 1), self.occupied(cx - 1, cy + 1)];
        if occ.iter().filter(|o| **o).count() < 3 {
            return None;
        }
        let (a, b) = t_front(self.rot);
        if (occ[a] && occ[b]) || self.last_kick == 4 {
            return Some("full");
        }
        Some("mini")
    }

    fn lock(&mut self, drop: i32) -> Option<LockInfo> {
        let kind = self.kind;
        let spin = self.spin_type();
        let cells = self.cells_at(None);
        for &(px, py_) in &cells {
            self.rows[py_ as usize] |= 1 << px;
            self.cells[py_ as usize][px as usize] = Some(kind);
        }
        self.version += 1;
        self.pieces += 1;
        self.hold_used = false;
        self.active = false;
        if cells.iter().all(|c| c.1 < BUFFER) {
            self.die("lockout");
            return None;
        }
        let mut full: Vec<i32> = cells.iter().map(|c| c.1).filter(|py_| self.rows[*py_ as usize] == FULL).collect();
        full.sort_unstable();
        full.dedup();
        let lines = full.len() as i64;
        let cleared: Vec<[Option<char>; 10]> = full.iter().map(|r| self.cells[*r as usize]).collect();
        if lines != 0 {
            let keep: Vec<usize> = (0..ROWS as usize).filter(|i| self.rows[*i] != FULL).collect();
            let mut nr = vec![0u32; lines as usize];
            nr.extend(keep.iter().map(|i| self.rows[*i]));
            let mut nc = vec![[None; 10]; lines as usize];
            nc.extend(keep.iter().map(|i| self.cells[*i]));
            self.rows = nr;
            self.cells = nc;
        }
        let pc = lines > 0 && !self.rows.iter().any(|r| *r != 0);
        let difficult = lines == 4 || (spin.is_some() && lines > 0);
        let mut b2b_bonus = false;
        if lines != 0 {
            if difficult {
                b2b_bonus = self.b2b;
                self.b2b = true;
            } else {
                self.b2b = false;
            }
            self.combo += 1;
        } else {
            self.combo = -1;
        }
        let sp = spin.unwrap_or("");
        let level = self.level;
        let mut base = score_table(sp, lines);
        if b2b_bonus {
            base = base * 3 / 2;
        }
        let mut points = base * level;
        if self.combo > 0 {
            points += 50 * self.combo * level;
        }
        if pc {
            let bonus = if b2b_bonus && lines == 4 { 3200 } else { pc_points(lines) };
            points += bonus * level;
        }
        self.score += points;
        let mut attack = attack_table(sp, lines);
        if lines != 0 {
            if b2b_bonus {
                attack += 1;
            }
            attack += COMBO_ATTACK[(self.combo as usize).min(COMBO_ATTACK.len() - 1)];
            if pc {
                attack += 10;
            }
        }
        let mut cancelled = 0;
        let mut sent = attack;
        while sent > 0 && !self.pending.is_empty() {
            let take = sent.min(self.pending[0].0);
            self.pending[0].0 -= take;
            sent -= take;
            cancelled += take;
            if self.pending[0].0 <= 0 {
                self.pending.remove(0);
            }
        }
        self.attack_sent += sent;
        self.lines += lines;
        if lines == 4 {
            self.tetrises += 1;
        }
        if spin == Some("full") && lines != 0 {
            self.tspins += 1;
        }
        let mut level_up = false;
        if !self.fixed_level {
            let new_level = self.start_level.max(1 + self.lines / 10);
            if new_level > self.level {
                self.level = new_level;
                level_up = true;
            }
        }
        let info = LockInfo { kind, lines, spin, b2b: b2b_bonus, combo: self.combo, pc, points, attack, sent, cancelled, rows: full, cleared, cells, drop };
        self.events.push(Event::Lock(info.clone()));
        if level_up {
            self.events.push(Event::Level(self.level));
        }
        if lines == 0 {
            self.raise_garbage();
        }
        if !self.dead {
            self.spawn(None);
        }
        Some(info)
    }

    /// `receive(lines)`
    pub fn receive(&mut self, lines: i64) {
        if lines <= 0 || self.dead {
            return;
        }
        let hole = self.grng.randint(0, (COLS - 1) as i64);
        self.pending.push((lines, hole, 0.0));
    }

    fn raise_garbage(&mut self) {
        let mut total = 0;
        let mut overflow = false;
        while !self.pending.is_empty() && total < GARBAGE_CAP {
            if self.pending[0].2 < GARBAGE_DELAY {
                break;
            }
            let n = self.pending[0].0.min(GARBAGE_CAP - total);
            self.pending[0].0 -= n;
            let hole = self.pending[0].1;
            if self.pending[0].0 <= 0 {
                self.pending.remove(0);
            }
            for _ in 0..n {
                if self.rows[0] != 0 {
                    overflow = true;
                }
                self.rows.remove(0);
                self.cells.remove(0);
                self.rows.push(FULL & !(1 << hole));
                let mut row = [Some('G'); 10];
                row[hole as usize] = None;
                self.cells.push(row);
            }
            total += n;
        }
        if total != 0 {
            self.version += 1;
            self.garbage_received += total;
            self.events.push(Event::Garbage(total));
            if overflow {
                self.die("topout");
            }
        }
    }
}
