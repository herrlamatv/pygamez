//! geodash_core.rs
//! ===============
//! Der Physik-Kern von Geometry Dash - ohne Zeichnen, ohne Zufall.
//! Uebertragen aus `games/geodash_core.py`.
//!
//! Nur Ganzzahlen (Festkomma: ein Block = [`B`] Einheiten, ein Schritt =
//! 1/240 s), nur `+ - *`, Vergleiche und die abrundende Division [`fdiv`].
//! Objekte werden in Listen-Reihenfolge geprueft, Spalten von links nach
//! rechts - damit rechnen Python, JavaScript und Rust bitgleich.
//!
//! Ein Level ist ein JSON-Objekt (siehe [`normalize_level`]) mit
//! `objects = [[kind, x, y, rot, ...], ...]`.

use std::cell::OnceCell;
use std::collections::{BTreeSet, HashMap, HashSet};

use serde_json::{json, Map, Value};

/// Einheiten je Block.
pub const B: i64 = 36000;
/// Physik-Schritte je Sekunde.
pub const HZ: i64 = 240;
/// Halber Block.
pub const HALF: i64 = B / 2;
/// Vorwaertstempo je Stufe (Einheiten/Schritt).
pub const SPEEDS: [i64; 4] = [1260, 1560, 1944, 2340];
/// Anzeigenamen der Tempi.
pub const SPEED_LABELS: [&str; 4] = ["0.5x", "1x", "2x", "3x"];

/// Wuerfel.
pub const CUBE: i64 = 0;
/// Schiff.
pub const SHIP: i64 = 1;
/// Ball.
pub const BALL: i64 = 2;
/// UFO.
pub const UFO: i64 = 3;
/// Welle.
pub const WAVE: i64 = 4;
/// Namen der Spielformen.
pub const MODE_NAMES: [&str; 5] = ["cube", "ship", "ball", "ufo", "wave"];

const GRAVITY: [i64; 5] = [66, 0, 52, 44, 0];
const FALL_MAX: [i64; 5] = [4400, 0, 3800, 3000, 0];
const CUBE_JUMP: i64 = 3300;
const UFO_JUMP: i64 = 2150;
const BALL_PUSH: i64 = 1300;
const SHIP_UP: i64 = 40;
const SHIP_DOWN: i64 = 36;
const SHIP_MAX_UP: i64 = 1700;
const SHIP_MAX_DOWN: i64 = 1900;
const ORB_V: [[i64; 3]; 5] = [[3450, 2500, 1500], [2400, 1700, 1200], [3000, 2200, 1500], [2600, 1900, 1400], [0, 0, 0]];
const PAD_V: [[i64; 3]; 5] = [[4500, 2700, 2600], [2800, 1900, 1600], [3800, 2400, 2400], [3400, 2300, 2200], [0, 0, 0]];
/// Standardhoehe des Korridors.
pub const CORRIDOR: i64 = 10 * B;
const BUFFER: i64 = 20;
const SNAP: i64 = 9000;
const INNER: i64 = 10800;
const INNER_OFF: i64 = (B - INNER) / 2;
/// Groesse der Welle.
pub const WAVE_SIZE: i64 = 12000;
/// Versatz der Welle.
pub const WAVE_OFF: i64 = (B - WAVE_SIZE) / 2;
const FALL_DEATH: i64 = -2 * B;
/// Hoechste Hoehe.
pub const MAX_Y: i64 = 48 * B;
/// Laengstes Level in Bloecken.
pub const MAX_LENGTH: i64 = 4000;
/// Hoechste Reihe.
pub const MAX_ROW: i64 = 40;
/// Startposition.
pub const START_X: i64 = 0;
/// Kuerzestes Level.
pub const MIN_LENGTH: i64 = 40;

/// Abrundende Ganzzahl-Division (b > 0).
#[inline]
pub fn fdiv(a: i64, b: i64) -> i64 {
    a.div_euclid(b)
}

/// Klassen.
pub const C_SOLID: i64 = 0;
/// Gefahr.
pub const C_HAZARD: i64 = 1;
/// Grube.
pub const C_PIT: i64 = 2;
/// Sprungpad.
pub const C_PAD: i64 = 3;
/// Orb.
pub const C_ORB: i64 = 4;
/// Spielform-Portal.
pub const C_MODE: i64 = 5;
/// Schwerkraft-Portal.
pub const C_GRAV: i64 = 6;
/// Tempo-Portal.
pub const C_SPEED: i64 = 7;
/// Muenze.
pub const C_COIN: i64 = 8;
/// Farb-Ausloeser.
pub const C_TRIGGER: i64 = 9;

/// (name, klasse, wert, hitbox x0,y0,x1,y1 in 1/100 Block).
pub const KINDS: [(&str, i64, i64, [i64; 4]); 24] = [
    ("block", C_SOLID, 0, [0, 0, 100, 100]),
    ("half", C_SOLID, 0, [0, 0, 100, 50]),
    ("spike", C_HAZARD, 0, [40, 22, 60, 64]),
    ("spike_s", C_HAZARD, 0, [40, 4, 60, 32]),
    ("pit", C_PIT, 0, [0, 0, 100, 100]),
    ("pad_y", C_PAD, 0, [8, 0, 92, 22]),
    ("pad_p", C_PAD, 1, [8, 0, 92, 22]),
    ("pad_b", C_PAD, 2, [8, 0, 92, 22]),
    ("orb_y", C_ORB, 0, [-12, -12, 112, 112]),
    ("orb_p", C_ORB, 1, [-12, -12, 112, 112]),
    ("orb_b", C_ORB, 2, [-12, -12, 112, 112]),
    ("p_cube", C_MODE, CUBE, [15, -100, 85, 200]),
    ("p_ship", C_MODE, SHIP, [15, -100, 85, 200]),
    ("p_ball", C_MODE, BALL, [15, -100, 85, 200]),
    ("p_ufo", C_MODE, UFO, [15, -100, 85, 200]),
    ("p_wave", C_MODE, WAVE, [15, -100, 85, 200]),
    ("g_norm", C_GRAV, 1, [15, -100, 85, 200]),
    ("g_flip", C_GRAV, -1, [15, -100, 85, 200]),
    ("s_slow", C_SPEED, 0, [0, -100, 100, 200]),
    ("s_norm", C_SPEED, 1, [0, -100, 100, 200]),
    ("s_fast", C_SPEED, 2, [0, -100, 100, 200]),
    ("s_vfast", C_SPEED, 3, [0, -100, 100, 200]),
    ("coin", C_COIN, 0, [10, 10, 90, 90]),
    ("color", C_TRIGGER, 0, [0, 0, 0, 0]),
];

/// `KIND_ID[name]`
pub fn kind_id(name: &str) -> Option<usize> {
    KINDS.iter().position(|k| k.0 == name)
}

/// Arten, die sich in 90-Grad-Schritten drehen lassen.
pub const ROTATE_ALL: [&str; 3] = ["half", "spike", "spike_s"];
/// Arten, die sich nur umdrehen lassen (0/2).
pub const ROTATE_FLIP: [&str; 3] = ["pad_y", "pad_p", "pad_b"];

/// Zusatzwerte hinter [kind, x, y, rot]: (default, lo, hi).
pub fn params(kind: &str) -> &'static [(i64, i64, i64)] {
    match kind {
        "p_ship" | "p_ball" | "p_ufo" | "p_wave" => &[(0, 0, 30)],
        "color" => &[(0, 0, 1), (255, 0, 255), (255, 0, 255), (255, 0, 255), (6, 0, 60)],
        _ => &[],
    }
}

/// Hoechstzahl Muenzen.
pub const MAX_COINS: usize = 3;

fn rotate_box(bx: [i64; 4], rot: usize) -> [i64; 4] {
    let [mut x0, mut y0, mut x1, mut y1] = bx;
    for _ in 0..rot {
        (x0, y0, x1, y1) = (y0, B - x1, y1, B - x0);
    }
    [x0, y0, x1, y1]
}

/// `HITBOX[kind][rot]`
pub fn hitbox(k: usize, rot: usize) -> [i64; 4] {
    let [a, b, c, d] = KINDS[k].3;
    let base = [fdiv(a * B, 100), fdiv(b * B, 100), fdiv(c * B, 100), fdiv(d * B, 100)];
    rotate_box(base, rot)
}

// ---------------------------------------------------------------------------
//  Level-Daten pruefen
// ---------------------------------------------------------------------------

/// Standard-Hintergrund.
pub const DEFAULT_BG: [i64; 3] = [40, 110, 255];
/// Standard-Boden.
pub const DEFAULT_GROUND: [i64; 3] = [20, 70, 200];
/// Musikstile.
pub const MUSIC_STYLES: [&str; 5] = ["drive", "chip", "dream", "dark", "none"];

/// `_int(v, lo, hi, default)` - Pythons `int(v)` mit Grenzen.
pub fn int_of(v: Option<&Value>, lo: i64, hi: i64, default: i64) -> i64 {
    let n = match v {
        Some(Value::Number(n)) => match n.as_i64() {
            Some(i) => Some(i),
            None => n.as_f64().map(|f| f.trunc() as i64),
        },
        Some(Value::String(s)) => s.trim().parse::<i64>().ok(),
        _ => None,
    };
    match n {
        Some(n) => lo.max(hi.min(n)),
        None => default,
    }
}

fn color_of(v: Option<&Value>, default: [i64; 3]) -> Value {
    match v.and_then(|x| x.as_array()) {
        Some(a) if a.len() >= 3 => json!([int_of(a.first(), 0, 255, 0), int_of(a.get(1), 0, 255, 0), int_of(a.get(2), 0, 255, 0)]),
        _ => json!(default),
    }
}

/// Ein Objekt: Art und Zahlen [x, y, rot, zusatz...].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Obj {
    /// Art.
    pub kind: &'static str,
    /// x, y, rot, Zusatzwerte.
    pub v: Vec<i64>,
}

impl Obj {
    /// Als JSON `[kind, x, y, rot, ...]`.
    pub fn to_json(&self) -> Value {
        let mut a = vec![json!(self.kind)];
        a.extend(self.v.iter().map(|x| json!(x)));
        Value::Array(a)
    }
    /// x.
    pub fn x(&self) -> i64 {
        self.v[0]
    }
    /// y.
    pub fn y(&self) -> i64 {
        self.v[1]
    }
    /// rot.
    pub fn rot(&self) -> i64 {
        self.v[2]
    }
}

/// `normalize_object(o)`
pub fn normalize_object(o: &Value) -> Option<Obj> {
    let a = o.as_array()?;
    if a.len() < 3 {
        return None;
    }
    let name = a[0].as_str()?;
    let k = kind_id(name)?;
    let kind = KINDS[k].0;
    let x = int_of(a.get(1), 0, MAX_LENGTH, -1);
    let mut y = int_of(a.get(2), 0, MAX_ROW, -1);
    if x < 0 || y < 0 {
        return None;
    }
    let mut rot = int_of(if a.len() > 3 { a.get(3) } else { None }.or(Some(&json!(0))), 0, 3, 0);
    if ROTATE_FLIP.contains(&kind) {
        rot = if rot == 1 || rot == 2 { 2 } else { 0 };
    } else if !ROTATE_ALL.contains(&kind) {
        rot = 0;
    }
    if kind == "pit" {
        y = 0;
    }
    let mut v = vec![x, y, rot];
    for (i, (default, lo, hi)) in params(kind).iter().enumerate() {
        let raw = if a.len() > 4 + i { a.get(4 + i).cloned() } else { Some(json!(default)) };
        v.push(int_of(raw.as_ref(), *lo, *hi, *default));
    }
    Some(Obj { kind, v })
}

fn sort_key(o: &Obj) -> (i64, i64, usize, i64) {
    (o.v[0], o.v[1], kind_id(o.kind).unwrap_or(0), o.v[2])
}

/// Objekte eines (normalisierten) Levels lesen.
pub fn objects_of(d: &Map<String, Value>) -> Vec<Obj> {
    d.get("objects").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(normalize_object).collect()).unwrap_or_default()
}

/// `normalize_level(d)` - sauberes Level-Dict (Kopfdaten bleiben).
pub fn normalize_level(d: &Map<String, Value>) -> Map<String, Value> {
    let mut d = d.clone();
    let mut objs: Vec<Obj> = Vec::new();
    let mut coins = 0;
    let mut seen: HashSet<Obj> = HashSet::new();
    if let Some(Value::Array(list)) = d.get("objects") {
        for o in list {
            let Some(n) = normalize_object(o) else {
                continue;
            };
            if n.kind == "coin" {
                if coins >= MAX_COINS {
                    continue;
                }
                coins += 1;
            }
            if seen.contains(&n) {
                continue;
            }
            seen.insert(n.clone());
            objs.push(n);
        }
    }
    objs.sort_by_key(sort_key);
    d.insert("objects".into(), Value::Array(objs.iter().map(|o| o.to_json()).collect()));
    let speed = int_of(Some(d.get("speed").unwrap_or(&json!(1))), 0, 3, 1);
    d.insert("speed".into(), json!(speed));
    let mode = int_of(Some(d.get("mode").unwrap_or(&json!(CUBE))), 0, 4, CUBE);
    d.insert("mode".into(), json!(mode));
    let bg = color_of(d.get("bg"), DEFAULT_BG);
    d.insert("bg".into(), bg);
    let ground = color_of(d.get("ground"), DEFAULT_GROUND);
    d.insert("ground".into(), ground);
    let music_ok = d.get("music").and_then(|v| v.as_str()).is_some_and(|m| MUSIC_STYLES.contains(&m));
    if !music_ok {
        d.insert("music".into(), json!("drive"));
    }
    let length = int_of(Some(d.get("length").unwrap_or(&json!(0))), 0, MAX_LENGTH + 20, 0);
    d.insert("length".into(), json!(length));
    d
}

/// `level_length(d)`
pub fn level_length(d: &Map<String, Value>) -> i64 {
    let mut last = 0;
    for o in objects_of(d) {
        if o.kind != "color" {
            last = last.max(o.x());
        }
    }
    let given = d.get("length").and_then(|v| v.as_f64()).map(|f| f as i64).unwrap_or(0);
    MIN_LENGTH.max(given).max(last + 12)
}

/// `content_hash(d)` - Pruefsumme ueber alles Spielrelevante.
pub fn content_hash(d: &Map<String, Value>) -> String {
    let d = normalize_level(d);
    let mut parts = vec![format!("{}|{}|{}", d["speed"], d["mode"], level_length(&d))];
    for o in objects_of(&d) {
        if o.kind != "color" {
            let mut s = o.kind.to_string();
            for v in &o.v {
                s += &format!(",{v}");
            }
            parts.push(s);
        }
    }
    format!("{:08x}", fnv1a(&parts.join(";")))
}

/// `fnv1a(text)` - 32 Bit.
pub fn fnv1a(text: &str) -> u32 {
    let mut h: u32 = 0x811C9DC5;
    for ch in text.chars() {
        h ^= (ch as u32) & 0xFF;
        h = h.wrapping_mul(0x01000193);
    }
    h
}

// ---------------------------------------------------------------------------
//  Kompiliertes Level
// ---------------------------------------------------------------------------

/// Ein Level in der Form, die die Physik schnell abfragen kann.
#[derive(Clone, Debug)]
pub struct Level {
    /// Das normalisierte Level-Dict.
    pub data: Map<String, Value>,
    /// Objekte (normalisiert, sortiert).
    pub objs: Vec<Obj>,
    /// Art-Index je Objekt.
    pub kind: Vec<usize>,
    /// Klasse je Objekt.
    pub cls: Vec<i64>,
    /// Wert je Objekt.
    pub val: Vec<i64>,
    /// Hitbox in Welt-Einheiten.
    pub bx: Vec<[i64; 4]>,
    /// Zusatzwerte.
    pub par: Vec<Vec<i64>>,
    /// Zelle x.
    pub ox: Vec<i64>,
    /// Zelle y.
    pub oy: Vec<i64>,
    /// Drehung.
    pub rot: Vec<i64>,
    /// Spalte -> Objekte.
    pub cols: HashMap<i64, Vec<usize>>,
    /// Gruben-Spalten.
    pub pits: HashSet<i64>,
    /// Muenze -> Bit.
    pub coin_bit: HashMap<usize, i64>,
    /// Farb-Ausloeser.
    pub triggers: Vec<usize>,
    /// Anzahl Objekte.
    pub n: usize,
    /// Anzahl Muenzen.
    pub coin_count: i64,
    /// Laenge in Bloecken.
    pub length: i64,
    /// Ziel-x.
    pub end_x: i64,
    /// Starttempo.
    pub speed: i64,
    /// Startform.
    pub mode: i64,
    /// Zeichen-Cache: Spalte -> Objekte (`lv.cache["draw_cols"]`).
    pub draw_cols: OnceCell<HashMap<i64, Vec<usize>>>,
    /// Zeichen-Cache: Kanten-Masken der Bloecke (`lv.cache["masks"]`).
    pub masks: OnceCell<HashMap<usize, i64>>,
}

impl Level {
    /// `Level(d)`
    pub fn new(d: &Map<String, Value>) -> Level {
        let d = normalize_level(d);
        let objs = objects_of(&d);
        let mut lv = Level {
            data: d.clone(),
            objs: objs.clone(),
            kind: Vec::new(),
            cls: Vec::new(),
            val: Vec::new(),
            bx: Vec::new(),
            par: Vec::new(),
            ox: Vec::new(),
            oy: Vec::new(),
            rot: Vec::new(),
            cols: HashMap::new(),
            pits: HashSet::new(),
            coin_bit: HashMap::new(),
            triggers: Vec::new(),
            n: objs.len(),
            coin_count: 0,
            length: 0,
            end_x: 0,
            speed: d["speed"].as_i64().unwrap_or(1),
            mode: d["mode"].as_i64().unwrap_or(CUBE),
            draw_cols: OnceCell::new(),
            masks: OnceCell::new(),
        };
        let mut coins = 0;
        for (i, o) in objs.iter().enumerate() {
            let k = kind_id(o.kind).unwrap_or(0);
            let (_, cls, val, _) = KINDS[k];
            let (cx, cy, rot) = (o.x() * B, o.y() * B, o.rot());
            let hb = hitbox(k, rot as usize);
            let bx = [cx + hb[0], cy + hb[1], cx + hb[2], cy + hb[3]];
            lv.kind.push(k);
            lv.cls.push(cls);
            lv.val.push(val);
            lv.bx.push(bx);
            lv.par.push(o.v[3..].to_vec());
            lv.ox.push(o.x());
            lv.oy.push(o.y());
            lv.rot.push(rot);
            if cls == C_PIT {
                lv.pits.insert(o.x());
                continue;
            }
            if cls == C_TRIGGER {
                lv.triggers.push(i);
                continue;
            }
            if cls == C_COIN {
                lv.coin_bit.insert(i, 1 << coins);
                coins += 1;
            }
            let c0 = fdiv(bx[0], B);
            let c1 = fdiv(bx[2] - 1, B);
            for c in c0..=c1 {
                lv.cols.entry(c).or_default().push(i);
            }
        }
        lv.coin_count = coins;
        lv.length = level_length(&d);
        lv.end_x = lv.length * B;
        lv
    }

    fn col(&self, c: i64) -> &[usize] {
        self.cols.get(&c).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

// ---------------------------------------------------------------------------
//  Zustand
// ---------------------------------------------------------------------------

/// Alles, was einen Versuch ausmacht.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    /// Schritt.
    pub step: i64,
    /// x (Einheiten).
    pub x: i64,
    /// y (Einheiten).
    pub y: i64,
    /// Senkrechtes Tempo.
    pub vy: i64,
    /// Spielform.
    pub mode: i64,
    /// Schwerkraft (+1/-1).
    pub grav: i64,
    /// Tempostufe.
    pub speed: i64,
    /// Steht auf dem Boden.
    pub ground: i64,
    /// Decke (0 = keine).
    pub ceil: i64,
    /// Klick-Puffer.
    pub buf: i64,
    /// Tot.
    pub dead: i64,
    /// Geschafft.
    pub won: i64,
    /// Muenzen (Bits).
    pub coins: i64,
    /// Ausgeloeste Objekte.
    pub used: BTreeSet<usize>,
}

/// `new_state(lv, start_block)`
pub fn new_state(lv: &Level, start_block: Option<i64>) -> State {
    let mut s = State {
        step: 0,
        x: START_X,
        y: 0,
        vy: 0,
        mode: lv.mode,
        grav: 1,
        speed: lv.speed,
        ground: 1,
        ceil: if lv.mode != CUBE { CORRIDOR } else { 0 },
        buf: 0,
        dead: 0,
        won: 0,
        coins: 0,
        used: BTreeSet::new(),
    };
    let Some(start) = start_block.filter(|b| *b > 0) else {
        return s;
    };
    s.x = start * B;
    for i in 0..lv.n {
        if lv.ox[i] >= start {
            break;
        }
        match lv.cls[i] {
            C_MODE => set_mode(&mut s, lv.val[i], &lv.par[i]),
            C_GRAV => s.grav = lv.val[i],
            C_SPEED => s.speed = lv.val[i],
            _ => {}
        }
    }
    s.vy = 0;
    if s.mode == CUBE || s.mode == BALL {
        if s.grav > 0 {
            s.y = stack_top(lv, start);
        } else {
            s.y = 0.max(stack_bottom(lv, start) - B);
        }
    } else {
        s.y = if s.ceil > 0 { fdiv(s.ceil - B, 2) } else { 3 * B };
        s.ground = 0;
    }
    s
}

fn stack_top(lv: &Level, col: i64) -> i64 {
    let mut top = 0;
    let mut changed = true;
    while changed {
        changed = false;
        for &i in lv.col(col) {
            if lv.cls[i] == C_SOLID && lv.bx[i][1] <= top && top < lv.bx[i][3] {
                top = lv.bx[i][3];
                changed = true;
            }
        }
    }
    top
}

fn stack_bottom(lv: &Level, col: i64) -> i64 {
    let mut best = MAX_Y;
    for &i in lv.col(col) {
        if lv.cls[i] == C_SOLID && lv.bx[i][1] >= B && lv.bx[i][1] < best {
            best = lv.bx[i][1];
        }
    }
    if best < MAX_Y {
        best
    } else {
        6 * B
    }
}

fn set_mode(s: &mut State, mode: i64, par: &[i64]) {
    s.mode = mode;
    if mode == CUBE {
        s.ceil = 0;
    } else {
        let h = par.first().copied().unwrap_or(0);
        s.ceil = if h > 0 { h * B } else { CORRIDOR };
    }
    if mode == SHIP {
        let mut v = s.vy * s.grav;
        if v > SHIP_MAX_UP {
            v = SHIP_MAX_UP;
        }
        if v < -SHIP_MAX_DOWN {
            v = -SHIP_MAX_DOWN;
        }
        s.vy = v * s.grav;
    }
    s.ground = 0;
}

fn floor_under(lv: &Level, x0: i64, x1: i64) -> bool {
    if lv.pits.is_empty() {
        return true;
    }
    (fdiv(x0, B)..=fdiv(x1 - 1, B)).any(|c| !lv.pits.contains(&c))
}

fn first_orb(s: &State, lv: &Level, x0: i64, y0: i64, x1: i64, y1: i64) -> Option<usize> {
    for c in fdiv(x0, B)..=fdiv(x1 - 1, B) {
        for &i in lv.col(c) {
            if lv.cls[i] != C_ORB || s.used.contains(&i) {
                continue;
            }
            let b = lv.bx[i];
            if b[0] < x1 && b[2] > x0 && b[1] < y1 && b[3] > y0 {
                return Some(i);
            }
        }
    }
    None
}

/// `step(s, lv, held, pressed)` - genau ein Schritt (1/240 s).
pub fn step(s: &mut State, lv: &Level, held: bool, pressed: bool) {
    if s.dead != 0 || s.won != 0 {
        return;
    }
    s.step += 1;
    if pressed {
        s.buf = BUFFER;
    } else if !held {
        s.buf = 0;
    }
    let mode = s.mode;
    let mu = mode as usize;
    let size = if mode == WAVE { WAVE_SIZE } else { B };
    let off = if mode == WAVE { WAVE_OFF } else { 0 };
    if s.buf > 0 {
        if let Some(oi) = first_orb(s, lv, s.x + off, s.y + off, s.x + off + size, s.y + off + size) {
            s.used.insert(oi);
            let which = lv.val[oi];
            if which == 2 {
                s.grav = -s.grav;
                s.vy = -s.grav * ORB_V[mu][2];
            } else if mode != WAVE {
                s.vy = s.grav * ORB_V[mu][which as usize];
            }
            s.ground = 0;
            s.buf = 0;
        } else if mode == UFO {
            s.vy = s.grav * UFO_JUMP;
            s.ground = 0;
            s.buf = 0;
        } else if mode == BALL && s.ground != 0 {
            s.grav = -s.grav;
            s.vy = -s.grav * BALL_PUSH;
            s.ground = 0;
            s.buf = 0;
        }
    }
    let g = s.grav;
    if mode == CUBE && s.ground != 0 && (held || pressed) {
        s.vy = CUBE_JUMP * g;
        s.ground = 0;
    }
    let mut v = s.vy * g;
    if mode == SHIP {
        v += if held { SHIP_UP } else { -SHIP_DOWN };
        if v > SHIP_MAX_UP {
            v = SHIP_MAX_UP;
        }
        if v < -SHIP_MAX_DOWN {
            v = -SHIP_MAX_DOWN;
        }
    } else if mode == WAVE {
        v = if held { SPEEDS[s.speed as usize] } else { -SPEEDS[s.speed as usize] };
    } else {
        v -= GRAVITY[mu];
        if v < -FALL_MAX[mu] {
            v = -FALL_MAX[mu];
        }
    }
    s.vy = v * g;
    let prev0 = s.y + off;
    let prev1 = prev0 + size;
    s.y += s.vy;
    s.x += SPEEDS[s.speed as usize];
    s.ground = 0;
    let px0 = s.x + off;
    let px1 = px0 + size;
    let py0 = s.y + off;
    if py0 < 0 && floor_under(lv, px0, px1) {
        if prev0 >= -SNAP {
            s.y = -off;
            if s.vy < 0 {
                s.vy = 0;
            }
            if g > 0 {
                s.ground = 1;
            }
        } else {
            s.dead = 1;
            return;
        }
    }
    if s.ceil > 0 && s.y + off + size > s.ceil {
        s.y = s.ceil - size - off;
        if s.vy > 0 {
            s.vy = 0;
        }
        if g < 0 {
            s.ground = 1;
        }
    }
    let py0 = s.y + off;
    let py1 = py0 + size;
    let (c0, c1) = (fdiv(px0, B), fdiv(px1 - 1, B));
    let mut land: Option<i64> = None;
    let mut bump: Option<i64> = None;
    let mut hit = false;
    for c in c0..=c1 {
        for &i in lv.col(c) {
            if lv.cls[i] != C_SOLID {
                continue;
            }
            let b = lv.bx[i];
            if !(b[0] < px1 && b[2] > px0 && b[1] < py1 && b[3] > py0) {
                continue;
            }
            if mode == WAVE {
                s.dead = 1;
                return;
            }
            hit = true;
            if g > 0 {
                if s.vy <= 0 && prev0 >= b[3] - SNAP {
                    if land.is_none_or(|l| b[3] > l) {
                        land = Some(b[3]);
                    }
                } else if mode != CUBE && s.vy > 0 && prev1 <= b[1] + SNAP && bump.is_none_or(|bb| b[1] < bb) {
                    bump = Some(b[1]);
                }
            } else if s.vy >= 0 && prev1 <= b[1] + SNAP {
                if land.is_none_or(|l| b[1] < l) {
                    land = Some(b[1]);
                }
            } else if mode != CUBE && s.vy < 0 && prev0 >= b[3] - SNAP && bump.is_none_or(|bb| b[3] > bb) {
                bump = Some(b[3]);
            }
        }
    }
    if hit {
        if let Some(l) = land {
            s.y = if g > 0 { l } else { l - B };
            s.vy = 0;
            s.ground = 1;
        }
        if let Some(bb) = bump {
            s.y = if g > 0 { bb - B } else { bb };
            s.vy = 0;
        }
        let ix0 = s.x + INNER_OFF;
        let ix1 = ix0 + INNER;
        let iy0 = s.y + INNER_OFF;
        let iy1 = iy0 + INNER;
        for c in fdiv(ix0, B)..=fdiv(ix1 - 1, B) {
            for &i in lv.col(c) {
                if lv.cls[i] != C_SOLID {
                    continue;
                }
                let b = lv.bx[i];
                if b[0] < ix1 && b[2] > ix0 && b[1] < iy1 && b[3] > iy0 {
                    s.dead = 1;
                    return;
                }
            }
        }
    }
    let py0 = s.y + off;
    let py1 = py0 + size;
    for c in fdiv(px0, B)..=fdiv(px1 - 1, B) {
        for &i in lv.col(c) {
            let cls = lv.cls[i];
            if cls == C_SOLID {
                continue;
            }
            let b = lv.bx[i];
            if !(b[0] < px1 && b[2] > px0 && b[1] < py1 && b[3] > py0) {
                continue;
            }
            if cls == C_HAZARD {
                s.dead = 1;
                return;
            }
            if cls == C_ORB || s.used.contains(&i) {
                continue;
            }
            match cls {
                C_PAD => {
                    s.used.insert(i);
                    let which = lv.val[i];
                    if which == 2 {
                        s.grav = -s.grav;
                        s.vy = -s.grav * PAD_V[s.mode as usize][2];
                    } else if s.mode != WAVE {
                        s.vy = s.grav * PAD_V[s.mode as usize][which as usize];
                    }
                    s.ground = 0;
                }
                C_MODE => {
                    s.used.insert(i);
                    let m = if lv.val[i] != s.mode { lv.val[i] } else { s.mode };
                    set_mode(s, m, &lv.par[i]);
                }
                C_GRAV => {
                    s.used.insert(i);
                    if lv.val[i] != s.grav {
                        s.grav = lv.val[i];
                        s.vy = 0;
                        s.ground = 0;
                    }
                }
                C_SPEED => {
                    s.used.insert(i);
                    s.speed = lv.val[i];
                }
                C_COIN => {
                    s.used.insert(i);
                    s.coins |= lv.coin_bit.get(&i).copied().unwrap_or(0);
                }
                _ => {}
            }
        }
    }
    if s.y < FALL_DEATH || s.y > MAX_Y {
        s.dead = 1;
        return;
    }
    if s.x >= lv.end_x {
        s.won = 1;
    }
    if s.buf > 0 {
        s.buf -= 1;
    }
}

/// `progress(s, lv)` - Fortschritt in ganzen Prozent.
pub fn progress(s: &State, lv: &Level) -> i64 {
    if s.won != 0 {
        return 100;
    }
    0.max(99.min(fdiv((s.x - START_X) * 100, lv.end_x - START_X)))
}

/// `state_str(s)` - kanonische Textform.
pub fn state_str(s: &State) -> String {
    let used: Vec<String> = s.used.iter().map(|i| i.to_string()).collect();
    format!(
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        s.step,
        s.x,
        s.y,
        s.vy,
        s.mode,
        s.grav,
        s.speed,
        s.ground,
        s.ceil,
        s.buf,
        s.dead,
        s.won,
        s.coins,
        used.join(".")
    )
}

/// `state_hash(s)`
pub fn state_hash(s: &State) -> String {
    format!("{:08x}", fnv1a(&state_str(s)))
}

/// `held_at(toggles, k)`
pub fn held_at(toggles: &[i64], k: i64) -> bool {
    let mut n = 0;
    for &t in toggles {
        if t <= k {
            n += 1;
        } else {
            break;
        }
    }
    n & 1 == 1
}

/// `run_toggles(lv, toggles, max_steps, trace_every, start)`
pub fn run_toggles(lv: &Level, toggles: &[i64], max_steps: Option<i64>, trace_every: i64, start: Option<&State>) -> (State, Vec<(i64, String)>) {
    let mut s = start.cloned().unwrap_or_else(|| new_state(lv, None));
    let limit = max_steps.unwrap_or(fdiv(lv.end_x, SPEEDS[0]) + 4 * HZ);
    let mut toggles = toggles.to_vec();
    toggles.sort_unstable();
    let mut ti = 0;
    let mut held = false;
    let mut trace = Vec::new();
    let mut k = 0;
    while k < limit && s.dead == 0 && s.won == 0 {
        let was = held;
        while ti < toggles.len() && toggles[ti] <= k {
            held = !held;
            ti += 1;
        }
        step(&mut s, lv, held, held && !was);
        k += 1;
        if trace_every != 0 && k % trace_every == 0 {
            trace.push((k, state_hash(&s)));
        }
    }
    trace.push((k, state_hash(&s)));
    (s, trace)
}
