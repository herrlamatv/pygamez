//! crossyroad_world.rs
//! ===================
//! Die Welt von Crossy Road ohne Grafik - uebertragen aus
//! `games/crossyroad_world.py`.
//!
//! Reihen-Generator (seedrand, gleiche Reihenfolge der Zufallszahlen wie
//! Python und Web), garantierter Weg, Bewegungsformeln als reine Funktionen
//! der Spielzeit und die Modell-Hilfen (Drehen/Spiegeln). Die Quader-Daten
//! stehen in [`crate::crossyroad_models`].

use std::collections::HashMap;

use pgz_core::seedrand::Rand;
use pgz_engine::py;

pub use crate::crossyroad_models::B;
use crate::crossyroad_models::{CHAR_COLORS, CHAR_MODELS, MODELS};

/// Begehbare Spalten 0..8.
pub const COLS: i32 = 9;
/// Startspalte.
pub const START_COL: i32 = 4;
/// Welt links/rechts ausserhalb.
pub const MARGIN: f64 = 8.0;
/// Umlauflaenge einer Spur.
pub const LOOP: f64 = (COLS as f64) + 2.0 * MARGIN;
/// Sichere Startwiese.
pub const START_ROWS: i64 = 3;
/// Ab hier volle Schwierigkeit.
pub const DIFF_ROWS: f64 = 160.0;

/// Laenge eines Autos.
pub const CAR_LEN: f64 = 1.4;
/// Laenge eines Lasters.
pub const TRUCK_LEN: f64 = 2.5;
const N_CAR_COLORS: i64 = 6;
/// Laenge je Wagen.
pub const TRAIN_CAR: f64 = 3.0;
const TRAIN_SPEED: f64 = 24.0;
const TRAIN_WARN: f64 = 1.0;
const BIG_COIN: i32 = 5;

/// Dauer eines Huepfers.
pub const HOP_T: f64 = 0.13;
/// Scheitelhoehe eines Huepfers.
pub const HOP_Z: f64 = 0.42;
const CAR_HALF: f64 = 0.28;
const LOG_GRACE: f64 = 0.3;
/// So weit hinter der Kamera holt der Adler.
pub const EAGLE_ROWS: f64 = 3.2;
/// Vorsprung der Kriechgrenze.
pub const CREEP_LAG: f64 = 1.0;
const CREEP_BASE: f64 = 0.45;
const CREEP_GROW: f64 = 0.0016;
const CREEP_MAX: f64 = 0.8;
const NIGHT_FROM: f64 = 50.0;
const NIGHT_CYCLE: f64 = 90.0;

/// Figuren und Preise.
pub const CHARACTERS: [(&str, i64); 10] = [
    ("chicken", 0),
    ("frog", 25),
    ("pig", 40),
    ("penguin", 60),
    ("cat", 80),
    ("fox", 100),
    ("llama", 125),
    ("robot", 150),
    ("ghost", 200),
    ("unicorn", 250),
];

/// `CHAR_IDS`
pub fn char_ids() -> Vec<&'static str> {
    CHARACTERS.iter().map(|c| c.0).collect()
}

/// `PRICES[cid]`
pub fn price(cid: &str) -> i64 {
    CHARACTERS.iter().find(|c| c.0 == cid).map(|c| c.1).unwrap_or(0)
}

/// `HOVER`
pub fn hovers(cid: &str) -> bool {
    cid == "ghost"
}

/// `CHAR_COLORS[cid]`
pub fn char_color(cid: &str) -> (i32, i32, i32) {
    CHAR_COLORS.iter().find(|c| c.0 == cid).map(|c| c.1).unwrap_or((246, 246, 246))
}

/// Reihenart.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    /// Wiese.
    Grass,
    /// Strasse.
    Road,
    /// Fluss.
    River,
    /// Gleis.
    Rail,
}

const KINDS: [Kind; 4] = [Kind::Grass, Kind::Road, Kind::River, Kind::Rail];

impl Kind {
    /// `LEVEL[kind]`
    pub fn level(self) -> f64 {
        match self {
            Kind::Grass => 0.12,
            Kind::Road => 0.0,
            Kind::Rail => 0.04,
            Kind::River => -0.12,
        }
    }

    fn index(self) -> usize {
        KINDS.iter().position(|k| *k == self).unwrap_or(0)
    }
}

/// `difficulty(r)`
pub fn difficulty(r: i64) -> f64 {
    1f64.min(r as f64 / DIFF_ROWS)
}

fn spread(entry: &[i32], ok: impl Fn(i32) -> bool) -> Vec<i32> {
    let mut seen = [false; COLS as usize];
    let mut stack = Vec::new();
    for &c in entry {
        if !seen[c as usize] {
            seen[c as usize] = true;
            stack.push(c);
        }
    }
    while let Some(c) = stack.pop() {
        for n in [c - 1, c + 1] {
            if (0..COLS).contains(&n) && !seen[n as usize] && ok(n) {
                seen[n as usize] = true;
                stack.push(n);
            }
        }
    }
    (0..COLS).filter(|c| seen[*c as usize]).collect()
}

/// Eine Reihe.
#[derive(Clone, Debug)]
pub struct Row {
    /// Reihennummer.
    pub r: i64,
    /// Art.
    pub kind: Kind,
    /// Baeume je Spalte (Wiese).
    pub trees: Option<Vec<i32>>,
    /// Seerosen je Spalte (Fluss mit Seerosen).
    pub pads: Option<Vec<i32>>,
    /// Fahrtrichtung.
    pub dir: i32,
    /// Tempo.
    pub speed: f64,
    /// Fahrzeuge/Staemme: (x, Laenge, Farbe).
    pub objs: Vec<(f64, f64, i64)>,
    /// Zug-Periode.
    pub period: f64,
    /// Zug-Phase.
    pub phase: f64,
    /// Waggons.
    pub cars: i64,
    /// Muenze: (Spalte, Wert).
    pub coin: Option<(i32, i32)>,
    /// Erreichbare Spalten.
    pub reach: Vec<i32>,
}

fn blank(r: i64, kind: Kind) -> Row {
    Row { r, kind, trees: None, pads: None, dir: 0, speed: 0.0, objs: Vec::new(), period: 0.0, phase: 0.0, cars: 0, coin: None, reach: (0..COLS).collect() }
}

/// `back_row(r)` - Reihen hinter dem Start.
fn back_row(r: i64) -> Row {
    let mut row = blank(r, Kind::Grass);
    let mut trees = vec![0; COLS as usize];
    for c in 0..COLS {
        let h = deco_hash(r, c as i64);
        let wall = r == -3 || (r == -2 && [0, 1, 7, 8].contains(&c)) || (r == -1 && [0, 8].contains(&c));
        if wall || (r < -3 && h % 100 < 55) {
            trees[c as usize] = 1 + ((h >> 8) % 3) as i32;
        }
    }
    row.reach = (0..COLS).filter(|c| trees[*c as usize] == 0).collect();
    row.trees = Some(trees);
    row
}

/// `deco_hash(r, c)` - kleiner 32-Bit-Hash fuer reine Deko.
pub fn deco_hash(r: i64, c: i64) -> u64 {
    let mut h = (r.wrapping_mul(374_761_393).wrapping_add(c.wrapping_mul(668_265_263)) as u64) & 0xFFFF_FFFF;
    h = ((h ^ (h >> 13)) * 1_274_126_177) & 0xFFFF_FFFF;
    (h ^ (h >> 16)) & 0xFFFF_FFFF
}

/// Endlose, aus einem Seed erzeugte Strecke.
pub struct World {
    rng: Rand,
    rows: HashMap<i64, Row>,
    next_row: i64,
    prev_reach: Vec<i32>,
    last_kind: Kind,
    last_dir: i32,
    last_log: bool,
}

impl World {
    /// `World(seed)`
    pub fn new(seed: u64) -> World {
        World {
            rng: Rand::new((seed & 0xFFFF_FFFF) as u32),
            rows: HashMap::new(),
            next_row: 0,
            prev_reach: vec![START_COL],
            last_kind: Kind::Grass,
            last_dir: 1,
            last_log: false,
        }
    }

    /// `row(r)` - erzeugt bei Bedarf.
    pub fn row(&mut self, r: i64) -> &Row {
        if r < 0 {
            return self.rows.entry(r).or_insert_with(|| back_row(r));
        }
        while r >= self.next_row {
            self.zone();
        }
        &self.rows[&r]
    }

    /// `kind(r)`
    pub fn kind(&mut self, r: i64) -> Kind {
        self.row(r).kind
    }

    /// `rail_run(r)`
    pub fn rail_run(&mut self, mut r: i64) -> i64 {
        let mut n = 0;
        while r >= 0 && self.row(r).kind == Kind::Rail {
            n += 1;
            r -= 1;
        }
        n
    }

    fn push(&mut self, row: Row) {
        self.prev_reach = row.reach.clone();
        self.next_row = row.r + 1;
        if row.dir != 0 {
            self.last_dir = row.dir;
        }
        self.last_log = row.kind == Kind::River && row.pads.is_none();
        self.rows.insert(row.r, row);
    }

    fn zone(&mut self) {
        let r0 = self.next_row;
        if r0 == 0 {
            for r in 0..START_ROWS {
                self.grass(r, true);
            }
            self.last_kind = Kind::Grass;
            return;
        }
        let d = difficulty(r0);
        let mut w: [i64; 4] = if self.last_kind == Kind::Grass {
            [0, 46 + (10.0 * d) as i64, 30 + (8.0 * d) as i64, 12 + (10.0 * d) as i64]
        } else {
            let mut w = [58, 20 + (10.0 * d) as i64, 14 + (8.0 * d) as i64, 6 + (8.0 * d) as i64];
            w[self.last_kind.index()] = 0;
            w
        };
        if r0 < 10 {
            w[3] = 0;
        }
        let wf: Vec<f64> = w.iter().map(|v| *v as f64).collect();
        let kind = KINDS[self.rng.weighted(&wf)];
        let early = r0 < 14;
        let n = match kind {
            Kind::Grass => self.rng.randint(1, 3 - (d * 1.5) as i64),
            Kind::Road => self.rng.randint(1, if early { 2 } else { 2 + (3.0 * d) as i64 }),
            Kind::River => self.rng.randint(1, if early { 2 } else { 2 + (2.0 * d) as i64 }),
            Kind::Rail => 1 + self.rng.weighted(&[10.0, 6.0, (6.0 * d) as i64 as f64, (4.0 * d) as i64 as f64, (4.0 * d) as i64 as f64]) as i64,
        };
        for i in 0..n {
            let r = self.next_row;
            match kind {
                Kind::Grass => self.grass(r, false),
                Kind::Road => self.road(r, i),
                Kind::River => self.river(r),
                Kind::Rail => self.rail(r),
            }
        }
        self.last_kind = kind;
    }

    fn coin(&mut self, row: &mut Row, p: f64) {
        if p > 0.0 && self.rng.random() < p {
            let c = *self.rng.choice(&row.reach).unwrap_or(&0);
            let big = row.r >= 20 && self.rng.random() < 0.1;
            row.coin = Some((c, if big { BIG_COIN } else { 1 }));
        }
    }

    fn grass(&mut self, r: i64, start: bool) {
        let mut row = blank(r, Kind::Grass);
        let mut trees = vec![0; COLS as usize];
        if r > 0 {
            let dens = if start { 0.12 } else { 0.12 + 0.18 * difficulty(r) };
            for t in trees.iter_mut() {
                if self.rng.random() < dens {
                    *t = self.rng.randint(1, 4) as i32;
                }
            }
            if start {
                trees[START_COL as usize] = 0;
            }
        }
        let mut entry: Vec<i32> = self.prev_reach.iter().copied().filter(|c| trees[*c as usize] == 0).collect();
        if entry.is_empty() {
            let c = *self.rng.choice(&self.prev_reach).unwrap_or(&START_COL);
            trees[c as usize] = 0;
            entry = vec![c];
        }
        let tr = trees.clone();
        row.reach = spread(&entry, |c| tr[c as usize] == 0);
        row.trees = Some(trees);
        self.coin(&mut row, if r > 0 { 0.3 } else { 0.0 });
        self.push(row);
    }

    fn road(&mut self, r: i64, i: i64) {
        let d = difficulty(r);
        let mut row = blank(r, Kind::Road);
        let direction = if i > 0 && self.rng.random() < 0.7 {
            -self.last_dir
        } else if self.rng.random() < 0.5 {
            1
        } else {
            -1
        };
        row.dir = direction;
        row.speed = (1.3 + self.rng.random() * 1.4) * (1.0 + 0.9 * d);
        let ln = if self.rng.random() < 0.28 { TRUCK_LEN } else { CAR_LEN };
        let mut objs = Vec::new();
        let gap_min = 2.0 + 1.2 * (1.0 - d);
        let x0 = self.rng.random() * 2.0;
        let mut x = x0;
        loop {
            objs.push((x, ln, self.rng.randint(0, N_CAR_COLORS - 1)));
            x += ln + gap_min + self.rng.random() * (3.4 - 1.4 * d);
            if x + ln + gap_min > LOOP + x0 {
                break;
            }
        }
        row.objs = objs;
        self.coin(&mut row, 0.1);
        self.push(row);
    }

    fn river(&mut self, r: i64) {
        let d = difficulty(r);
        let mut row = blank(r, Kind::River);
        if self.rng.random() < 0.2 {
            let mut pads: Vec<i32> = (0..COLS).map(|_| if self.rng.random() < 0.45 { 1 } else { 0 }).collect();
            let mut entry: Vec<i32> = self.prev_reach.iter().copied().filter(|c| pads[*c as usize] != 0).collect();
            if entry.is_empty() {
                let c = *self.rng.choice(&self.prev_reach).unwrap_or(&START_COL);
                pads[c as usize] = 1;
                entry = vec![c];
            }
            let pd = pads.clone();
            row.reach = spread(&entry, |c| pd[c as usize] == 1);
            row.pads = Some(pads);
            self.coin(&mut row, 0.15);
        } else {
            let direction = if self.last_log && self.rng.random() < 0.8 {
                -self.last_dir
            } else if self.rng.random() < 0.5 {
                1
            } else {
                -1
            };
            row.dir = direction;
            row.speed = (0.8 + self.rng.random() * 0.9) * (1.0 + 0.5 * d);
            let lmax = if d < 0.6 { 4 } else { 3 };
            let mut objs = Vec::new();
            let x0 = self.rng.random() * 2.0;
            let mut x = x0;
            loop {
                let mut ln = self.rng.randint(2, lmax);
                let room = LOOP + x0 - 1.0 - x;
                if room < 2.0 {
                    break;
                }
                ln = ln.min(room as i64);
                objs.push((x, ln as f64, 0));
                x += ln as f64 + 1.0 + self.rng.random() * (1.2 + 1.6 * d);
            }
            row.objs = objs;
        }
        self.push(row);
    }

    fn rail(&mut self, r: i64) {
        let d = difficulty(r);
        let mut row = blank(r, Kind::Rail);
        row.dir = if self.rng.random() < 0.5 { 1 } else { -1 };
        let period = 4.6 + self.rng.random() * 4.0 - 1.2 * d;
        row.period = period;
        row.phase = self.rng.random() * period;
        row.cars = self.rng.randint(2, 4);
        self.coin(&mut row, 0.08);
        self.push(row);
    }
}

// ----- Bewegung ------------------------------------------------------------------

/// `obj_x(row, obj, t)` - linke Kante zur Zeit t.
pub fn obj_x(row: &Row, obj: &(f64, f64, i64), t: f64) -> f64 {
    py::fpmod(obj.0 + row.dir as f64 * row.speed * t, LOOP) - MARGIN
}

/// `train_state(row, t)` - (warnen, Lok-Spitze, Zuglaenge).
pub fn train_state(row: &Row, t: f64) -> (bool, Option<f64>, f64) {
    let length = (row.cars + 1) as f64 * TRAIN_CAR;
    let u = py::fpmod(t + row.phase, row.period);
    if u < TRAIN_WARN {
        return (true, None, length);
    }
    let s = u - TRAIN_WARN;
    if s * TRAIN_SPEED > LOOP + length {
        return (false, None, length);
    }
    let dist = s * TRAIN_SPEED;
    let head = if row.dir > 0 { -MARGIN + dist } else { COLS as f64 + MARGIN - dist };
    (true, Some(head), length)
}

fn train_span(row: &Row, t: f64) -> Option<(f64, f64)> {
    let (_w, head, length) = train_state(row, t);
    let head = head?;
    if row.dir > 0 {
        return Some((head - length, head));
    }
    Some((head, head + length))
}

/// `vehicle_at(row, cx, t)`
pub fn vehicle_at(row: &Row, cx: f64, t: f64) -> i64 {
    let half = CAR_HALF;
    for (i, o) in row.objs.iter().enumerate() {
        let x = obj_x(row, o, t);
        if x + 0.08 < cx + half && cx - half < x + o.1 - 0.08 {
            return i as i64;
        }
    }
    -1
}

/// `train_hits(row, cx, t)`
pub fn train_hits(row: &Row, cx: f64, t: f64) -> bool {
    let half = CAR_HALF;
    match train_span(row, t) {
        Some(sp) => sp.0 < cx + half && cx - half < sp.1,
        None => false,
    }
}

/// `log_at(row, cx, t)` - (Index, Segment) oder `None`.
pub fn log_at(row: &Row, cx: f64, t: f64) -> Option<(usize, i64)> {
    let grace = LOG_GRACE;
    for (i, o) in row.objs.iter().enumerate() {
        let x = obj_x(row, o, t);
        if x - grace <= cx && cx <= x + o.1 + grace {
            let mut seg = if cx > x { (cx - x) as i64 } else { 0 };
            seg = 0.max((o.1 as i64 - 1).min(seg));
            return Some((i, seg));
        }
    }
    None
}

/// `creep_rate(best)`
pub fn creep_rate(best: i64) -> f64 {
    CREEP_MAX.min(CREEP_BASE + CREEP_GROW * 0.max(best) as f64)
}

/// `night_factor(rowf)` - 0 = Tag, 1 = tiefe Nacht.
pub fn night_factor(rowf: f64) -> f64 {
    if rowf < NIGHT_FROM {
        return 0.0;
    }
    let u = py::fpmod(rowf - NIGHT_FROM, NIGHT_CYCLE);
    let v = if u < 12.0 {
        u / 12.0
    } else if u < 50.0 {
        1.0
    } else if u < 62.0 {
        1.0 - (u - 50.0) / 12.0
    } else {
        0.0
    };
    v * v * (3.0 - 2.0 * v)
}

// =================================================================== Modelle

/// Figurmodell `CHAR_MODELS[cid]`.
pub fn char_model(cid: &str) -> Vec<B> {
    CHAR_MODELS.iter().find(|m| m.0 == cid).map(|m| m.1.to_vec()).unwrap_or_default()
}

/// Festes Modell `MODELS[name]`.
pub fn model(name: &str) -> Vec<B> {
    MODELS.iter().find(|m| m.0 == name).map(|m| m.1.to_vec()).unwrap_or_default()
}

/// `mirror_x(boxes, length)`
pub fn mirror_x(boxes: &[B], length: f64) -> Vec<B> {
    boxes.iter().map(|b| (length - b.3, b.1, b.2, length - b.0, b.4, b.5, b.6)).collect()
}

/// `rotate(boxes, facing)` - dreht ein nach +y blickendes Figurmodell.
pub fn rotate(boxes: &[B], facing: &str) -> Vec<B> {
    if facing == "up" {
        return boxes.to_vec();
    }
    boxes
        .iter()
        .map(|&(x0, y0, z0, x1, y1, z1, col)| match facing {
            "down" => (1.0 - x1, 1.0 - y1, z0, 1.0 - x0, 1.0 - y0, z1, col),
            "right" => (y0, 1.0 - x1, z0, y1, 1.0 - x0, z1, col),
            _ => (1.0 - y1, x0, z0, 1.0 - y0, x1, z1, col),
        })
        .collect()
}
