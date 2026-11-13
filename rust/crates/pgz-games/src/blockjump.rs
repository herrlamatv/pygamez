//! blockjump.rs
//! ============
//! Block Jump - ein 3D-Jump'n'Run im Minecraft-Stil (Software-Renderer).
//!
//! Uebertragen aus `games/blockjump.py`: prozedurale 16x16-Pixeltexturen mit
//! festen Seeds (CPython-Zufall, bitgleich), Detailstufen, Steve-Figur,
//! Ego-Hand, Beacon-Strahl, Gold-Barren, quadratische Sonne, Pixel-Wolken,
//! Motion Blur und seed-generierte Parkour-Level.

use std::cell::RefCell;
use std::collections::HashMap;
use std::f64::consts::TAU;
use std::sync::OnceLock;

use pgz_engine::prelude::*;
use serde_json::{json, Value};

const NEAR: f64 = 0.12;
const FOV_MUL: f64 = 1.0;
const FOG_START: f64 = 16.0;
const FOG_END: f64 = 46.0;
const DEG_PER_PX: f64 = 0.12;

type Rgb = (i32, i32, i32);
type V3 = (f64, f64, f64);
type Tex = Vec<Vec<Option<Rgb>>>;

const COL_SKY_TOP: Rgb = (110, 160, 250);
const COL_SKY_HOR: Rgb = (178, 209, 252);
const COL_FOG: Rgb = (198, 222, 252);
const COL_SUN: Rgb = (252, 250, 208);

const GRAVITY: f64 = 20.0;
const JUMP_VEL: f64 = 7.7;
const MOVE_SPEED: f64 = 4.7;
const CLIMB_SPEED: f64 = 3.3;
const SPRING_VEL: f64 = 12.6;
const EYE_H: f64 = 1.62;
const PLAYER_H: f64 = 1.7;
const HALF_W: f64 = 0.3;
const VY_MIN: f64 = -16.0;
const VY_MAX: f64 = 14.5;
const EPS: f64 = 1e-4;

const EMPTY: u8 = 0;
const GRASS: u8 = 1;
const DIRT: u8 = 2;
const STONE: u8 = 3;
const PLANK: u8 = 4;
const LADDER: u8 = 5;
const FENCE: u8 = 6;
const SPRING: u8 = 7;
const GOAL: u8 = 8;
const FENCE_H: f64 = 0.8;
const SPRING_H: f64 = 0.78;

fn is_solid_typ(t: u8) -> bool {
    matches!(t, GRASS | DIRT | STONE | PLANK | GOAL)
}

fn block_tex(t: u8) -> Option<[&'static str; 3]> {
    Some(match t {
        GRASS => ["grass_top", "grass_side", "dirt"],
        DIRT => ["dirt", "dirt", "dirt"],
        STONE => ["stone", "stone", "stone"],
        PLANK => ["planks", "planks", "planks"],
        GOAL => ["diamond", "diamond", "diamond"],
        _ => return None,
    })
}

const TEX_MODES: [&str; 3] = ["off", "low", "high"];

fn seed_base(mode: &str) -> u64 {
    match mode {
        "easy" => 1000,
        "hard" => 3000,
        _ => 2000,
    }
}

// ===========================================================================
//  Pixel-Texturen
// ===========================================================================

fn mul(c: Rgb, k: f64) -> Rgb {
    (
        0.max(255.min((c.0 as f64 * k) as i32)),
        0.max(255.min((c.1 as f64 * k) as i32)),
        0.max(255.min((c.2 as f64 * k) as i32)),
    )
}

#[allow(clippy::too_many_arguments)]
fn blocks(size: usize, seed: u64, cols: &[Rgb], weights: &[f64], spec: &[Rgb], spec_p: f64, var: f64, cell: usize) -> Tex {
    let mut rng = py::Random::new(seed);
    let mut rows: Tex = vec![vec![None; size]; size];
    for by in (0..size).step_by(cell) {
        for bx in (0..size).step_by(cell) {
            let base = cols[rng.weighted_index(weights)];
            for sy in (0..cell).step_by(2) {
                for sx in (0..cell).step_by(2) {
                    let c = mul(base, rng.uniform(1.0 - var, 1.0 + var));
                    for y in sy..sy + 2 {
                        for x in sx..sx + 2 {
                            rows[by + y][bx + x] = Some(c);
                        }
                    }
                }
            }
        }
    }
    if !spec.is_empty() {
        for row in rows.iter_mut() {
            for px in row.iter_mut() {
                if rng.random() < spec_p {
                    *px = Some(spec[rng.choice_index(spec.len())]);
                }
            }
        }
    }
    rows
}

fn t_dirt() -> Tex {
    blocks(16, 12, &[(134, 96, 67), (123, 88, 60), (145, 105, 74)], &[5.0, 3.0, 2.0], &[(113, 80, 55), (152, 112, 79)], 0.06, 0.05, 4)
}

fn t_grass_side() -> Tex {
    let dirt = t_dirt();
    let (green, gdark) = ((116, 172, 78), (97, 150, 62));
    let edge = [4, 4, 3, 3, 4, 4, 3, 3, 2, 2, 4, 4, 3, 3, 4, 4];
    (0..16)
        .map(|y| {
            (0..16)
                .map(|x| {
                    let e = edge[x];
                    if (y as i32) < e - 1 {
                        Some(green)
                    } else if (y as i32) < e {
                        Some(gdark)
                    } else {
                        dirt[y][x]
                    }
                })
                .collect()
        })
        .collect()
}

fn t_planks() -> Tex {
    let (base, seam, grain, light) = ((162, 130, 78), (124, 97, 55), (146, 116, 68), (172, 141, 88));
    let mut rng = py::Random::new(15);
    let mut rows = Vec::new();
    for y in 0..16 {
        let band = if [0, 6, 12].contains(&y) {
            light
        } else if [3, 4, 9, 15].contains(&y) {
            grain
        } else {
            base
        };
        let mut row = Vec::new();
        for x in 0..16 {
            let c = if y == 5 || y == 11 {
                seam
            } else if (y < 5 && x == 9) || (5 < y && y < 11 && x == 3) || (y > 11 && x == 12) {
                seam
            } else if rng.random() < 0.06 {
                grain
            } else {
                band
            };
            row.push(Some(c));
        }
        rows.push(row);
    }
    rows
}

fn t_wood() -> Tex {
    let (base, dark, light) = ((150, 116, 66), (120, 90, 50), (168, 132, 80));
    let cols: Vec<Rgb> = (0..16)
        .map(|x| {
            if [0, 7, 8, 15].contains(&x) {
                dark
            } else if x == 3 || x == 11 {
                light
            } else {
                base
            }
        })
        .collect();
    let mut rng = py::Random::new(16);
    (0..16).map(|_| (0..16).map(|x| Some(if rng.random() < 0.04 { dark } else { cols[x] })).collect()).collect()
}

fn t_diamond() -> Tex {
    let (base, cry, hi, dk) = ((110, 222, 216), (78, 200, 198), (196, 248, 244), (58, 168, 168));
    let spots = [(3, 3), (11, 4), (6, 10), (13, 12)];
    (0..16i32)
        .map(|y| {
            (0..16i32)
                .map(|x| {
                    // `c is base`: nur solange noch kein anderer Wert gesetzt wurde
                    let mut c = base;
                    let mut is_base = true;
                    for (sx, sy) in spots {
                        let d = (x - sx).abs().max((y - sy).abs());
                        if d == 0 {
                            c = hi;
                            is_base = false;
                        } else if d == 1 {
                            c = cry;
                            is_base = false;
                        } else if d == 2 && is_base {
                            c = dk;
                            is_base = false;
                        }
                    }
                    Some(c)
                })
                .collect()
        })
        .collect()
}

fn t_slime() -> Tex {
    let (base, edge, core, hi) = ((114, 196, 94), (86, 158, 70), (146, 224, 126), (176, 240, 158));
    (0..16i32)
        .map(|y| {
            (0..16i32)
                .map(|x| {
                    let b = x.min(y).min(15 - x).min(15 - y);
                    Some(if b < 1 {
                        edge
                    } else if b < 3 {
                        base
                    } else if b < 6 {
                        core
                    } else {
                        hi
                    })
                })
                .collect()
        })
        .collect()
}

fn t_beacon() -> Tex {
    let (a, b) = ((226, 255, 246), (186, 246, 232));
    (0..8).map(|y| (0..8).map(|x| Some(if (x + y) % 3 != 0 { a } else { b })).collect()).collect()
}

fn skin(ch: char) -> Option<Rgb> {
    Some(match ch {
        'H' => (74, 48, 30),
        'h' => (60, 38, 23),
        'S' => (233, 189, 148),
        's' => (204, 160, 122),
        'W' => (245, 245, 245),
        'I' => (62, 92, 190),
        'M' => (150, 96, 78),
        'C' => (0, 172, 172),
        'c' => (0, 142, 145),
        'B' => (58, 63, 138),
        'b' => (46, 50, 116),
        'G' => (88, 76, 64),
        'K' => (52, 44, 36),
        _ => return None,
    })
}

fn art(rows: &[&str]) -> Tex {
    rows.iter().map(|r| r.chars().map(skin).collect()).collect()
}

const FACE: [&str; 8] = ["HHHHHHHH", "HHHHHHHH", "HSSSSSSH", "SWIssIWS", "SSSsSSSS", "SSMMMMSS", "hSSSSSSh", "SSssssSS"];
const HEAD_SIDE: [&str; 8] = ["HHHHHHHH", "HHHHHHHH", "HHHSSSSS", "HHSSSSSs", "HSSSSSSS", "SSSSSSSs", "SSSSSSSS", "SSsSSsSS"];
const HEAD_BACK: [&str; 8] = ["HHHHHHHH", "HHHHHHHH", "HHHHHHHH", "HHHHHHHH", "HHHhhHHH", "HHHHHHHH", "hHHHHHHh", "SSssssSS"];
const HEAD_TOP: [&str; 8] = ["HHHHHHHH", "HhHHHHhH", "HHHHhHHH", "HHhHHHHH", "HHHHHhHH", "HhHHHHHH", "HHHHhHHH", "HHHHHHHH"];
const BODY_FRONT: [&str; 8] = ["SSKKKKSS", "CCCCCCCC", "CCCcCCCC", "CCCCCcCC", "CcCCCCCC", "CCCCCCcC", "cCCCCCCc", "KKKKKKKK"];
const BODY_SIDE: [&str; 8] = ["KKKKKKKK", "CCCCCCCC", "CCcCCCCC", "CCCCCCcC", "CcCCCCCC", "CCCCcCCC", "cCCCCCCc", "KKKKKKKK"];
const ARM: [&str; 8] = ["CCCCCCCC", "CCcCCCCC", "CCCCCcCC", "cCCCCCCc", "CCCCcCCC", "SSSSSSSS", "SsSSSSsS", "SSSssSSS"];
const LEG: [&str; 8] = ["BBBBBBBB", "BBbBBBBB", "BBBBBbBB", "bBBBBBBb", "BBBBbBBB", "BBbBBBBB", "GGGGGGGG", "GGGGGGGG"];
const INGOT: [&str; 8] = ["        ", "  ####  ", " ###### ", "########", "########", " ###### ", "  ####  ", "        "];

fn t_ingot() -> Tex {
    let (gold, hi, dk) = ((247, 214, 88), (255, 242, 170), (198, 152, 44));
    INGOT
        .iter()
        .enumerate()
        .map(|(y, line)| {
            line.chars()
                .map(|ch| {
                    if ch == ' ' {
                        None
                    } else if y <= 2 {
                        Some(hi)
                    } else if y >= 5 {
                        Some(dk)
                    } else {
                        Some(gold)
                    }
                })
                .collect()
        })
        .collect()
}

fn textures() -> &'static HashMap<&'static str, Tex> {
    static T: OnceLock<HashMap<&'static str, Tex>> = OnceLock::new();
    T.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("grass_top", blocks(16, 11, &[(116, 172, 78), (105, 160, 70), (127, 182, 87)], &[5.0, 3.0, 2.0], &[(97, 150, 64), (135, 190, 95)], 0.06, 0.05, 4));
        m.insert("grass_side", t_grass_side());
        m.insert("dirt", t_dirt());
        m.insert("stone", blocks(16, 14, &[(127, 127, 127), (118, 118, 118), (136, 136, 136)], &[5.0, 3.0, 2.0], &[(109, 109, 109), (145, 145, 145)], 0.06, 0.05, 4));
        m.insert("planks", t_planks());
        m.insert("wood", t_wood());
        m.insert("diamond", t_diamond());
        m.insert("slime", t_slime());
        m.insert("beacon", t_beacon());
        m.insert("ingot", t_ingot());
        m.insert("face", art(&FACE));
        m.insert("head_side", art(&HEAD_SIDE));
        m.insert("head_back", art(&HEAD_BACK));
        m.insert("head_top", art(&HEAD_TOP));
        m.insert("body_front", art(&BODY_FRONT));
        m.insert("body_side", art(&BODY_SIDE));
        m.insert("arm", art(&ARM));
        m.insert("leg", art(&LEG));
        m
    })
}

fn tex(name: &str) -> &'static Tex {
    &textures()[name]
}

const QUANT: i32 = 12;

type Rects = Vec<(usize, usize, usize, usize, Rgb)>;

thread_local! {
    static MIP: RefCell<HashMap<(String, usize), Vec<Vec<Option<Rgb>>>>> = RefCell::new(HashMap::new());
    static RECTS: RefCell<HashMap<(String, usize), Rects>> = RefCell::new(HashMap::new());
    static GRID: RefCell<HashMap<(String, usize, i32, i32), Rects>> = RefCell::new(HashMap::new());
}

fn mip(name: &str, n: usize) -> Vec<Vec<Option<Rgb>>> {
    if let Some(m) = MIP.with(|c| c.borrow().get(&(name.to_string(), n)).cloned()) {
        return m;
    }
    let src = tex(name);
    let size = src.len();
    let nn = n.min(size);
    let step = size / nn;
    let mut rows: Vec<Vec<Option<Rgb>>> = Vec::new();
    for j in 0..nn {
        let mut row = Vec::new();
        for i in 0..nn {
            let (mut r, mut g, mut b, mut cnt) = (0i32, 0i32, 0i32, 0i32);
            for y in j * step..(j + 1) * step {
                for x in i * step..(i + 1) * step {
                    if let Some(c) = src[y][x] {
                        r += c.0;
                        g += c.1;
                        b += c.2;
                        cnt += 1;
                    }
                }
            }
            if cnt * 2 < (step * step) as i32 {
                row.push(None);
            } else {
                row.push(Some((r / cnt, g / cnt, b / cnt)));
            }
        }
        rows.push(row);
    }
    if nn < size {
        let vals: Vec<Rgb> = rows.iter().flatten().flatten().copied().collect();
        if !vals.is_empty() {
            let len = vals.len() as f64;
            let mr = vals.iter().map(|c| c.0 as i64).sum::<i64>() as f64 / len;
            let mg = vals.iter().map(|c| c.1 as i64).sum::<i64>() as f64 / len;
            let mb = vals.iter().map(|c| c.2 as i64).sum::<i64>() as f64 / len;
            let f = 1.08;
            for row in rows.iter_mut() {
                for c in row.iter_mut().flatten() {
                    *c = (
                        0.max(255.min((mr + (c.0 as f64 - mr) * f) as i32)),
                        0.max(255.min((mg + (c.1 as f64 - mg) * f) as i32)),
                        0.max(255.min((mb + (c.2 as f64 - mb) * f) as i32)),
                    );
                }
            }
        }
    }
    let (q, h) = (QUANT, QUANT / 2);
    for row in rows.iter_mut() {
        for c in row.iter_mut().flatten() {
            *c = (255.min(c.0 / q * q + h), 255.min(c.1 / q * q + h), 255.min(c.2 / q * q + h));
        }
    }
    MIP.with(|c| c.borrow_mut().insert((name.to_string(), n), rows.clone()));
    rows
}

fn rects(name: &str, n: usize) -> Rects {
    if let Some(r) = RECTS.with(|c| c.borrow().get(&(name.to_string(), n)).cloned()) {
        return r;
    }
    let grid = mip(name, n);
    let n = grid.len();
    let mut used = vec![vec![false; n]; n];
    let mut out = Vec::new();
    for j in 0..n {
        for i in 0..n {
            if used[j][i] {
                continue;
            }
            let c = grid[j][i];
            used[j][i] = true;
            let Some(col) = c else {
                continue;
            };
            let mut i1 = i;
            while i1 + 1 < n && !used[j][i1 + 1] && grid[j][i1 + 1] == c {
                i1 += 1;
                used[j][i1] = true;
            }
            let mut j1 = j;
            while j1 + 1 < n && (i..=i1).all(|x| !used[j1 + 1][x] && grid[j1 + 1][x] == c) {
                j1 += 1;
                for x in i..=i1 {
                    used[j1][x] = true;
                }
            }
            out.push((i, j, i1 + 1, j1 + 1, col));
        }
    }
    RECTS.with(|c| c.borrow_mut().insert((name.to_string(), n), out.clone()));
    out
}

fn avg_col(name: &str) -> Rgb {
    let vals: Vec<Rgb> = tex(name).iter().flatten().flatten().copied().collect();
    if vals.is_empty() {
        return (128, 128, 128);
    }
    let n = vals.len() as i32;
    (vals.iter().map(|c| c.0).sum::<i32>() / n, vals.iter().map(|c| c.1).sum::<i32>() / n, vals.iter().map(|c| c.2).sum::<i32>() / n)
}

fn opaque(name: &str) -> bool {
    tex(name).iter().flatten().all(|c| c.is_some())
}

fn shade_col(col: Rgb, k: f64) -> Rgb {
    (255.min((col.0 as f64 * k) as i32), 255.min((col.1 as f64 * k) as i32), 255.min((col.2 as f64 * k) as i32))
}

fn fog_col(col: Rgb, ft: f64) -> Rgb {
    if ft <= 0.0 {
        return col;
    }
    (
        (col.0 as f64 + (COL_FOG.0 - col.0) as f64 * ft) as i32,
        (col.1 as f64 + (COL_FOG.1 - col.1) as f64 * ft) as i32,
        (col.2 as f64 + (COL_FOG.2 - col.2) as f64 * ft) as i32,
    )
}

fn tex_grid(name: &str, n: usize, shade: f64, ft: f64) -> Rects {
    let si = (shade * 24.0 + 0.5) as i32;
    let fi = (ft * 12.0 + 0.5) as i32;
    let key = (name.to_string(), n, si, fi);
    if let Some(g) = GRID.with(|c| c.borrow().get(&key).cloned()) {
        return g;
    }
    let (k, f) = (si as f64 / 24.0, fi as f64 / 12.0);
    let g: Rects = rects(name, n).into_iter().map(|(i0, j0, i1, j1, c)| (i0, j0, i1, j1, fog_col(shade_col(c, k), f))).collect();
    GRID.with(|c| c.borrow_mut().insert(key, g.clone()));
    g
}

const CLOUD_MAP: [&str; 8] = ["01111000", "01111100", "00111000", "00000000", "00011110", "00111110", "00011100", "00000000"];
const HEART_ART: [&str; 9] = ["  ##  ##  ", " #rr##rr# ", "#rrrrrrrr#", "#rrrrrrrr#", "#rrrrrrrr#", " #rrrrrr# ", "  #rrrr#  ", "   #rr#   ", "    ##    "];

fn dir_from(yaw: f64, pitch: f64) -> V3 {
    let cp = pitch.cos();
    (yaw.sin() * cp, pitch.sin(), yaw.cos() * cp)
}

fn lerp3(a: V3, b: V3, f: f64) -> V3 {
    (a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f, a.2 + (b.2 - a.2) * f)
}

fn clip_near(pts: &[V3]) -> Vec<V3> {
    let mut out = Vec::new();
    let n = pts.len();
    for i in 0..n {
        let (a, b) = (pts[i], pts[(i + 1) % n]);
        let (a_in, b_in) = (a.2 >= NEAR, b.2 >= NEAR);
        if a_in {
            out.push(a);
        }
        if a_in != b_in {
            let tt = (NEAR - a.2) / (b.2 - a.2);
            out.push((a.0 + (b.0 - a.0) * tt, a.1 + (b.1 - a.1) * tt, NEAR));
        }
    }
    out
}

fn fog(col: Rgb, depth: f64) -> Rgb {
    fog_col(col, 0f64.max(1f64.min((depth - FOG_START) / (FOG_END - FOG_START))))
}

/// Eine Zeichenliste: (Tiefe, Vierecke, Umriss).
type Item = (f64, Vec<(Vec<(f64, f64)>, Rgb)>, Option<(Vec<(f64, f64)>, Rgb)>);

#[derive(Clone, Copy, PartialEq)]
enum State {
    Ready,
    Play,
    Clear,
    GameOver,
}

/// `BlockJumpGame`
pub struct BlockJumpGame {
    b: GameBase,
    blur: f64,
    first_person: bool,
    sens: f64,
    invert: bool,
    tex_mode: usize,
    hud: Font,
    small: Font,
    huge: Font,
    mid: Font,
    sky_cache: Option<((i32, i32), Surface)>,
    prev_frame: Option<Surface>,
    anim: f64,
    walk: f64,
    held: Vec<&'static str>,
    capture: bool,
    want_capture: bool,
    lives: i32,
    level: i64,
    yaw: f64,
    pitch: f64,
    state: State,
    world_keys: Vec<(i32, i32, i32)>,
    world: HashMap<(i32, i32, i32), u8>,
    coins: Vec<V3>,
    plat_count: i64,
    min_y: i32,
    spawn: V3,
    goal: V3,
    death_y: f64,
    cloud_y: f64,
    px: f64,
    py: f64,
    pz: f64,
    vx: f64,
    vy: f64,
    vz: f64,
    on_ground: bool,
    on_ladder: bool,
    checkpoint: V3,
    level_time: f64,
    coins_level: i64,
    deaths_level: i64,
    clear_t: f64,
    last_bonus: i64,
    land_cells: Vec<(i32, i32, i32)>,
    cam_pos: V3,
    cam_look: V3,
    basis: (V3, V3, V3),
    scx: f64,
    scy: f64,
    f: f64,
}

impl BlockJumpGame {
    /// `BlockJumpGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let fo = b.font.clone();
        let o = (0.0, 0.0, 0.0);
        let mut g = BlockJumpGame {
            b,
            blur: 0.35,
            first_person: true,
            sens: 1.0,
            invert: false,
            tex_mode: 2,
            hud: fo.clone(),
            small: fo.clone(),
            huge: fo.clone(),
            mid: fo,
            sky_cache: None,
            prev_frame: None,
            anim: 0.0,
            walk: 0.0,
            held: Vec::new(),
            capture: false,
            want_capture: true,
            lives: 3,
            level: 1,
            yaw: 0.0,
            pitch: 0.0,
            state: State::Ready,
            world_keys: Vec::new(),
            world: HashMap::new(),
            coins: Vec::new(),
            plat_count: 0,
            min_y: 0,
            spawn: o,
            goal: o,
            death_y: -7.0,
            cloud_y: 50.0,
            px: 0.0,
            py: 0.0,
            pz: 0.0,
            vx: 0.0,
            vy: 0.0,
            vz: 0.0,
            on_ground: true,
            on_ladder: false,
            checkpoint: o,
            level_time: 0.0,
            coins_level: 0,
            deaths_level: 0,
            clear_t: 0.0,
            last_bonus: 0,
            land_cells: Vec::new(),
            cam_pos: o,
            cam_look: o,
            basis: (o, o, o),
            scx: 0.0,
            scy: 0.0,
            f: 1.0,
        };
        g.reset();
        Box::new(g)
    }

    fn make_fonts(&mut self) {
        self.hud = ui::fb(20);
        self.small = ui::f(15);
        self.huge = ui::fb(30.max(self.b.height / 11));
        self.mid = ui::fb(22);
    }

    fn save_setting(&mut self, key: &str, value: Value) {
        self.b.set_opt("blockjump", key, value);
    }

    fn new_run(&mut self) {
        self.lives = 3;
        self.b.score = 0;
        self.b.game_over = false;
        self.level = 1;
        self.yaw = 0.0;
        self.pitch = 0.0;
        let l = self.level;
        self.build_level(l);
        self.state = State::Ready;
    }

    fn set_block(&mut self, k: (i32, i32, i32), t: u8) {
        if self.world.insert(k, t).is_none() {
            self.world_keys.push(k);
        }
    }

    // ===================================================== Level-Generierung

    fn build_level(&mut self, level: i64) {
        let mode = self.b.mode.clone();
        let mut rng = py::Random::new(seed_base(&mode) + level as u64 * 7919);
        self.world.clear();
        self.world_keys.clear();
        self.coins.clear();
        let hard = mode == "hard";
        let easy = mode == "easy";
        let n_plat = 7 + level * 2 + if hard { 2 } else { 0 };
        self.plat_count = n_plat;
        let (mut cx, mut cy, mut cz) = (0i32, 0i32, 0i32);
        self.min_y = 0;
        self.pad(cx, cz, cy, 1, 1, GRASS);
        self.spawn = (cx as f64 + 0.5, cy as f64 + 1.0, cz as f64 + 0.5);
        let feats: [&str; 5] = if !easy { ["jump", "jump", "ladder", "spring", "fence"] } else { ["jump", "jump", "jump", "ladder", "spring"] };
        for i in 0..n_plat {
            let last = i == n_plat - 1;
            let feat = if last { "jump" } else { rng.choice(&feats) };
            let hw = 1;
            let mut hd = if !hard { 1 } else { rng.choice(&[0, 1, 1]) };
            let top_type = rng.choice(&[GRASS, GRASS, STONE, PLANK]);
            if feat == "ladder" {
                let climb = if easy { rng.choice(&[3, 4]) } else { rng.choice(&[3, 4, 5]) };
                let lad_z = cz + 1;
                for yy in cy + 1..cy + climb + 1 {
                    self.set_block((cx, yy, lad_z), LADDER);
                    self.set_block((cx, yy, lad_z + 1), STONE);
                }
                self.coins.push((cx as f64 + 0.5, cy as f64 + climb as f64 / 2.0 + 1.4, lad_z as f64 + 0.5));
                cy += climb;
                cz = cz + 2 + hd;
                self.pad(cx, cz, cy, hw, hd, top_type);
            } else if feat == "spring" {
                self.set_block((cx, cy + 1, cz), SPRING);
                let lift = rng.choice(&[3, 4]);
                let dz = rng.choice(&[2, 3]);
                cy += lift;
                cz += dz;
                self.pad(cx, cz, cy, hw, hd, top_type);
                self.coins.push((cx as f64 + 0.5, cy as f64 - lift as f64 / 2.0 + 2.0, cz as f64 - dz as f64 / 2.0));
            } else {
                let dz = if easy { rng.choice(&[2, 3]) } else { rng.choice(&[3, 4]) };
                let dx = if easy { rng.choice(&[-1, 0, 1]) } else { rng.choice(&[-2, -1, 0, 1, 2]) };
                let mut dy = if easy { rng.choice(&[-1, 0]) } else { rng.choice(&[-2, -1, 0, 1]) };
                if hard && dz >= 4 {
                    dy = dy.min(0);
                    hd = 1;
                }
                let (mx, my, mz) = (cx, cy, cz);
                cx += dx;
                cy = (self.min_y - 3).max(cy + dy);
                cz += dz;
                self.pad(cx, cz, cy, hw, hd, top_type);
                self.coins.push(((mx + cx) as f64 / 2.0 + 0.5, (my + cy) as f64 / 2.0 + 1.7, (mz + cz) as f64 / 2.0 + 0.5));
                if feat == "fence" && !last {
                    self.set_block((cx, cy + 1, cz), FENCE);
                }
            }
            self.min_y = self.min_y.min(cy);
        }
        self.set_block((cx, cy + 1, cz), GOAL);
        self.goal = (cx as f64 + 0.5, cy as f64 + 1.0, cz as f64 + 0.5);
        self.death_y = self.min_y as f64 - 7.0;
        self.cloud_y = self.world_keys.iter().map(|k| k.1).max().unwrap_or(0) as f64 + 46.0;
        (self.px, self.py, self.pz) = self.spawn;
        self.vx = 0.0;
        self.vy = 0.0;
        self.vz = 0.0;
        self.on_ground = true;
        self.on_ladder = false;
        self.checkpoint = self.spawn;
        self.level_time = 0.0;
        self.coins_level = 0;
        self.deaths_level = 0;
        self.clear_t = 0.0;
        self.cam_pos = (self.px, self.py + EYE_H, self.pz);
        self.cam_look = (self.px, self.py + EYE_H, self.pz + 1.0);
    }

    fn pad(&mut self, cx: i32, cz: i32, y: i32, hw: i32, hd: i32, typ: u8) {
        for x in cx - hw..cx + hw + 1 {
            for z in cz - hd..cz + hd + 1 {
                self.set_block((x, y, z), typ);
            }
        }
    }

    // ===================================================== Blockabfragen

    fn cell(&self, x: i32, y: i32, z: i32) -> u8 {
        *self.world.get(&(x, y, z)).unwrap_or(&EMPTY)
    }

    fn is_solid(&self, x: i32, y: i32, z: i32) -> bool {
        is_solid_typ(self.cell(x, y, z))
    }

    fn col_h(&self, x: i32, y: i32, z: i32, axis: u8, delta: f64) -> Option<f64> {
        let typ = self.cell(x, y, z);
        if is_solid_typ(typ) {
            return Some(1.0);
        }
        if typ == FENCE {
            return Some(FENCE_H);
        }
        if typ == SPRING && axis == 1 && delta < 0.0 {
            return Some(SPRING_H);
        }
        None
    }

    // ===================================================== Eingabe

    fn move_acts(&self, key: &str) -> Vec<&'static str> {
        let k = if key.chars().count() == 1 { key.to_lowercase() } else { key.to_string() };
        let mut acts: Vec<&'static str> = ["up", "down", "left", "right"].iter().copied().filter(|a| self.b.is_action(&k, a, None)).collect();
        let arrow = match key {
            "Up" => Some("up"),
            "Down" => Some("down"),
            "Left" => Some("left"),
            "Right" => Some("right"),
            _ => None,
        };
        if let Some(a) = arrow {
            if !acts.contains(&a) {
                acts.push(a);
            }
        }
        acts
    }

    fn start_or_restart(&mut self) {
        if self.state == State::GameOver {
            self.b.game_over = false;
            self.new_run();
        }
        self.held.clear();
        self.state = State::Play;
        self.level_time = 0.0;
        self.want_capture = true;
        self.capture = true;
        self.b.play_sound("click");
    }

    fn apply_look(&mut self, rel: (i32, i32)) {
        let k = py::radians(DEG_PER_PX) * self.sens;
        let s = if self.invert { 1.0 } else { -1.0 };
        self.yaw = py::fpmod(self.yaw + rel.0 as f64 * k * s, TAU);
        let pc = py::radians(84.0);
        self.pitch = (-pc).max(pc.min(self.pitch + rel.1 as f64 * k * s));
    }

    fn jump(&mut self) {
        if self.state != State::Play {
            return;
        }
        if self.on_ground {
            self.vy = JUMP_VEL;
            self.on_ground = false;
            self.b.play_sound("click");
        } else if self.on_ladder {
            self.vy = JUMP_VEL * 0.85;
            self.on_ladder = false;
            let f = dir_from(self.yaw, 0.0);
            self.vx = -f.0 * MOVE_SPEED;
            self.vz = -f.2 * MOVE_SPEED;
            self.b.play_sound("click");
        }
    }

    // ===================================================== Physik

    fn held_axis(&self, pos: &str, neg: &str) -> f64 {
        (self.held.contains(&pos) as i32 - self.held.contains(&neg) as i32) as f64
    }

    fn physics(&mut self, dt: f64) {
        let fwd = self.held_axis("up", "down");
        let strafe = self.held_axis("right", "left");
        self.on_ladder = self.in_ladder();
        let f = dir_from(self.yaw, 0.0);
        let (fx, fz) = (f.0, f.2);
        let (rx, rz) = (-fz, fx);
        if self.on_ladder {
            self.vy = CLIMB_SPEED * fwd;
            let (mvx, mvz) = (rx * strafe, rz * strafe);
            let ln = py::hypot(mvx, mvz);
            if ln > 1e-6 {
                self.vx = mvx / ln * MOVE_SPEED * 0.7;
                self.vz = mvz / ln * MOVE_SPEED * 0.7;
            } else {
                self.vx = 0.0;
                self.vz = 0.0;
            }
        } else {
            let mvx = fx * fwd + rx * strafe;
            let mvz = fz * fwd + rz * strafe;
            let ln = py::hypot(mvx, mvz);
            if ln > 1e-6 {
                self.vx = mvx / ln * MOVE_SPEED;
                self.vz = mvz / ln * MOVE_SPEED;
            } else {
                self.vx = 0.0;
                self.vz = 0.0;
            }
            self.vy -= GRAVITY * dt;
        }
        self.vy = VY_MIN.max(VY_MAX.min(self.vy));
        self.walk += dt * 9.0 * 1f64.min(py::hypot(self.vx, self.vz) / MOVE_SPEED);
        self.on_ground = false;
        self.land_cells.clear();
        let (dx, dz, dy) = (self.vx * dt, self.vz * dt, self.vy * dt);
        self.move_axis(0, dx);
        self.move_axis(2, dz);
        self.move_axis(1, dy);
        if self.on_ground {
            let mut bounced = false;
            for c in self.land_cells.clone() {
                if self.world.get(&c) == Some(&SPRING) {
                    self.vy = SPRING_VEL;
                    self.on_ground = false;
                    bounced = true;
                    self.b.play_sound("move");
                    break;
                }
            }
            if !bounced {
                self.checkpoint = (self.px, self.py, self.pz);
            }
        }
        self.collect_coins();
        if self.at_goal() {
            self.level_clear();
            return;
        }
        if self.py < self.death_y {
            self.die();
        }
    }

    fn aabb(&self) -> (V3, V3) {
        ((self.px - HALF_W, self.py, self.pz - HALF_W), (self.px + HALF_W, self.py + PLAYER_H, self.pz + HALF_W))
    }

    fn rng_of(lo: f64, hi: f64) -> std::ops::RangeInclusive<i32> {
        ((lo + EPS).floor() as i32)..=((hi - EPS).floor() as i32)
    }

    fn move_axis(&mut self, axis: u8, delta: f64) {
        match axis {
            0 => self.px += delta,
            1 => self.py += delta,
            _ => self.pz += delta,
        }
        let (mn, mx) = self.aabb();
        let mut hits: Vec<(i32, i32, i32, f64)> = Vec::new();
        for x in Self::rng_of(mn.0, mx.0) {
            for y in Self::rng_of(mn.1, mx.1) {
                for z in Self::rng_of(mn.2, mx.2) {
                    if let Some(h) = self.col_h(x, y, z, axis, delta) {
                        if mn.1 + EPS < y as f64 + h {
                            hits.push((x, y, z, h));
                        }
                    }
                }
            }
        }
        if hits.is_empty() {
            return;
        }
        match axis {
            0 => {
                if delta > 0.0 {
                    self.px = hits.iter().map(|h| h.0).min().unwrap_or(0) as f64 - HALF_W - EPS;
                } else if delta < 0.0 {
                    self.px = hits.iter().map(|h| h.0).max().unwrap_or(0) as f64 + 1.0 + HALF_W + EPS;
                }
                self.vx = 0.0;
            }
            2 => {
                if delta > 0.0 {
                    self.pz = hits.iter().map(|h| h.2).min().unwrap_or(0) as f64 - HALF_W - EPS;
                } else if delta < 0.0 {
                    self.pz = hits.iter().map(|h| h.2).max().unwrap_or(0) as f64 + 1.0 + HALF_W + EPS;
                }
                self.vz = 0.0;
            }
            _ => {
                if delta > 0.0 {
                    self.py = hits.iter().map(|h| h.1).min().unwrap_or(0) as f64 - PLAYER_H - EPS;
                    self.vy = 0.0;
                } else if delta < 0.0 {
                    let top = hits.iter().map(|h| h.1 as f64 + h.3).fold(f64::NEG_INFINITY, f64::max);
                    self.py = top + EPS;
                    self.vy = 0.0;
                    self.on_ground = true;
                    self.land_cells = hits.iter().filter(|h| h.1 as f64 + h.3 == top).map(|h| (h.0, h.1, h.2)).collect();
                }
            }
        }
    }

    fn in_ladder(&self) -> bool {
        let (mn, mx) = self.aabb();
        for x in Self::rng_of(mn.0, mx.0) {
            for y in Self::rng_of(mn.1, mx.1) {
                for z in Self::rng_of(mn.2, mx.2) {
                    if self.cell(x, y, z) == LADDER {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn collect_coins(&mut self) {
        let mut remaining = Vec::new();
        for &(cx, cy, cz) in &self.coins.clone() {
            if (self.px - cx).abs() < 0.85 && (self.pz - cz).abs() < 0.85 && ((self.py + 0.9) - cy).abs() < 1.2 {
                self.b.score += 50;
                self.coins_level += 1;
                self.b.play_sound("click");
            } else {
                remaining.push((cx, cy, cz));
            }
        }
        self.coins = remaining;
    }

    fn at_goal(&self) -> bool {
        let (gx, gy, gz) = self.goal;
        (self.px - gx).abs() < 1.3 && (self.pz - gz).abs() < 1.3 && self.py > gy - 1.6
    }

    fn level_clear(&mut self) {
        let par = self.plat_count as f64 * 3.2;
        let bonus = 0.max(((par - self.level_time) * 15.0) as i64);
        self.b.score += 1000 + bonus;
        self.last_bonus = bonus;
        if self.deaths_level == 0 {
            self.b.ach_event("bj_deathless", None);
        }
        self.state = State::Clear;
        self.clear_t = 1.8;
        self.b.play_sound("win");
    }

    fn die(&mut self) {
        self.lives -= 1;
        self.deaths_level += 1;
        self.b.play_sound("gameover");
        if self.lives <= 0 {
            self.state = State::GameOver;
            self.b.game_over = true;
            self.capture = false;
        } else {
            (self.px, self.py, self.pz) = self.checkpoint;
            self.vx = 0.0;
            self.vy = 0.0;
            self.vz = 0.0;
            self.on_ground = true;
        }
    }

    // ===================================================== Kamera

    fn update_cam(&mut self) {
        let head = (self.px, self.py + EYE_H, self.pz);
        let f = dir_from(self.yaw, self.pitch);
        if self.first_person {
            self.cam_pos = head;
            self.cam_look = (head.0 + f.0, head.1 + f.1, head.2 + f.2);
        } else {
            let look_h = self.py + 1.1;
            let target = (self.px, look_h, self.pz);
            let dist = 4.4;
            let want = (self.px - f.0 * dist, look_h - f.1 * dist + 0.5, self.pz - f.2 * dist);
            self.cam_pos = self.cam_collide(target, want);
            self.cam_look = target;
        }
    }

    fn cam_collide(&self, a: V3, b: V3) -> V3 {
        let (dx, dy, dz) = (b.0 - a.0, b.1 - a.1, b.2 - a.2);
        let steps = 7;
        for i in 1..=steps {
            let tt = i as f64 / steps as f64;
            let p = (a.0 + dx * tt, a.1 + dy * tt, a.2 + dz * tt);
            if self.is_solid(p.0.floor() as i32, p.1.floor() as i32, p.2.floor() as i32) {
                let tt = 0f64.max((i - 1) as f64 / steps as f64) * 0.92;
                return (a.0 + dx * tt, a.1 + dy * tt, a.2 + dz * tt);
            }
        }
        b
    }

    // ===================================================== 3D-Renderer

    fn view_basis(&self) -> (V3, V3, V3) {
        let (cx, cy, cz) = self.cam_pos;
        let (fx, fy, fz) = (self.cam_look.0 - cx, self.cam_look.1 - cy, self.cam_look.2 - cz);
        let mut fl = (fx * fx + fy * fy + fz * fz).sqrt();
        if fl == 0.0 {
            fl = 1.0;
        }
        let f = (fx / fl, fy / fl, fz / fl);
        let mut rl = py::hypot(f.2, f.0);
        if rl == 0.0 {
            rl = 1.0;
        }
        let r = (-f.2 / rl, 0.0, f.0 / rl);
        let u = (r.1 * f.2 - r.2 * f.1, r.2 * f.0 - r.0 * f.2, r.0 * f.1 - r.1 * f.0);
        (r, u, f)
    }

    fn to_cam(&self, p: V3) -> V3 {
        let (r, u, f) = self.basis;
        let dx = p.0 - self.cam_pos.0;
        let dy = p.1 - self.cam_pos.1;
        let dz = p.2 - self.cam_pos.2;
        (dx * r.0 + dz * r.2, dx * u.0 + dy * u.1 + dz * u.2, dx * f.0 + dy * f.1 + dz * f.2)
    }

    fn proj(&self, c: V3) -> (f64, f64) {
        let k = self.f / c.2;
        (self.scx + c.0 * k, self.scy - c.1 * k)
    }

    fn tex_n(&self, depth: f64, native: usize, scale: f64, fine: bool) -> usize {
        if self.tex_mode == 0 {
            return 1;
        }
        let lim = if self.tex_mode == 2 { native } else { 4.min(native) };
        let step = if self.tex_mode == 2 {
            if fine {
                7.0
            } else {
                16.0
            }
        } else {
            26.0
        };
        let px = self.f * scale / depth.max(0.25);
        let mut n = 1;
        while n < lim && px >= n as f64 * 2.0 * step {
            n *= 2;
        }
        n
    }

    fn add_poly(&self, items: &mut Vec<Item>, world_pts: &[V3], color: Rgb, shade: f64, outline: Option<Rgb>, use_fog: bool) {
        let cs: Vec<V3> = world_pts.iter().map(|p| self.to_cam(*p)).collect();
        if cs.iter().all(|c| c.2 < NEAR) {
            return;
        }
        let cs = clip_near(&cs);
        if cs.len() < 3 {
            return;
        }
        let zs: Vec<f64> = cs.iter().map(|c| c.2).collect();
        let depth = py::sum(&zs) / cs.len() as f64;
        if use_fog && depth > FOG_END + 6.0 {
            return;
        }
        let pts: Vec<(f64, f64)> = cs.iter().map(|c| self.proj(*c)).collect();
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        if pts.iter().all(|p| p.0 < -40.0) || pts.iter().all(|p| p.0 > w + 40.0) || pts.iter().all(|p| p.1 < -40.0) || pts.iter().all(|p| p.1 > h + 40.0) {
            return;
        }
        let mut col = shade_col(color, shade);
        if use_fog {
            col = fog(col, depth);
        }
        let ol = outline.map(|o| (pts.clone(), o));
        items.push((depth, vec![(pts, col)], ol));
    }

    #[allow(clippy::too_many_arguments)]
    fn add_face(&self, items: &mut Vec<Item>, quad: [V3; 4], texn: &str, shade: f64, outline: Option<Rgb>, scale: f64, fine: bool) {
        if self.tex_mode == 0 {
            self.add_poly(items, &quad, avg_col(texn), shade, outline, true);
            return;
        }
        let (ca, cb, cc, cd) = (self.to_cam(quad[0]), self.to_cam(quad[1]), self.to_cam(quad[2]), self.to_cam(quad[3]));
        let zs = [ca.2, cb.2, cc.2, cd.2];
        if zs.iter().copied().fold(f64::NEG_INFINITY, f64::max) < NEAR {
            return;
        }
        let depth = (zs[0] + zs[1] + zs[2] + zs[3]) * 0.25;
        if depth > FOG_END + 6.0 {
            return;
        }
        let n = self.tex_n(depth, tex(texn).len(), scale, fine);
        if n <= 1 {
            self.add_poly(items, &quad, avg_col(texn), shade, outline, true);
            return;
        }
        let ft = 0f64.max(1f64.min((depth - FOG_START) / (FOG_END - FOG_START)));
        let rects = tex_grid(texn, n, shade, ft);
        let mut quads: Vec<(Vec<(f64, f64)>, Rgb)> = Vec::new();
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        if zs.iter().copied().fold(f64::INFINITY, f64::min) >= NEAR {
            let (f, scx, scy) = (self.f, self.scx, self.scy);
            let (ax, ay, az) = ca;
            let (dlx, dly, dlz) = (cd.0 - ax, cd.1 - ay, cd.2 - az);
            let (bx, by, bz) = cb;
            let (drx, dry, drz) = (cc.0 - bx, cc.1 - by, cc.2 - bz);
            let mut pr: Vec<Vec<(f64, f64)>> = Vec::with_capacity(n + 1);
            for j in 0..=n {
                let fj = j as f64 / n as f64;
                let (lx, ly, lz) = (ax + dlx * fj, ay + dly * fj, az + dlz * fj);
                let (ex, ey, ez) = ((bx + drx * fj) - lx, (by + dry * fj) - ly, (bz + drz * fj) - lz);
                let row: Vec<(f64, f64)> = (0..=n)
                    .map(|i| {
                        let fi = i as f64 / n as f64;
                        let k = f / (lz + ez * fi);
                        (scx + (lx + ex * fi) * k, scy - (ly + ey * fi) * k)
                    })
                    .collect();
                pr.push(row);
            }
            let xs = [pr[0][0].0, pr[0][n].0, pr[n][0].0, pr[n][n].0];
            let ys = [pr[0][0].1, pr[0][n].1, pr[n][0].1, pr[n][n].1];
            let mx = |v: &[f64]| v.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let mn = |v: &[f64]| v.iter().copied().fold(f64::INFINITY, f64::min);
            if mx(&xs) < -40.0 || mn(&xs) > w + 40.0 || mx(&ys) < -40.0 || mn(&ys) > h + 40.0 {
                return;
            }
            if opaque(texn) {
                quads.push((vec![pr[0][0], pr[0][n], pr[n][n], pr[n][0]], tex_grid(texn, 1, shade, ft)[0].4));
            }
            for &(i0, j0, i1, j1, col) in &rects {
                let (r0, r1) = (&pr[j0], &pr[j1]);
                quads.push((vec![r0[i0], r0[i1], r1[i1], r1[i0]], col));
            }
        } else {
            let left: Vec<V3> = (0..=n).map(|j| lerp3(ca, cd, j as f64 / n as f64)).collect();
            let right: Vec<V3> = (0..=n).map(|j| lerp3(cb, cc, j as f64 / n as f64)).collect();
            let rows: Vec<Vec<V3>> = (0..=n).map(|j| (0..=n).map(|i| lerp3(left[j], right[j], i as f64 / n as f64)).collect()).collect();
            if opaque(texn) {
                let base = clip_near(&[ca, cb, cc, cd]);
                if base.len() >= 3 {
                    quads.push((base.iter().map(|c| self.proj(*c)).collect(), tex_grid(texn, 1, shade, ft)[0].4));
                }
            }
            for &(i0, j0, i1, j1, col) in &rects {
                let (r0, r1) = (&rows[j0], &rows[j1]);
                let cell = clip_near(&[r0[i0], r0[i1], r1[i1], r1[i0]]);
                if cell.len() >= 3 {
                    quads.push((cell.iter().map(|c| self.proj(*c)).collect(), col));
                }
            }
        }
        if !quads.is_empty() {
            let ol = outline.map(|o| (clip_near(&[ca, cb, cc, cd]).iter().map(|c| self.proj(*c)).collect(), o));
            items.push((depth, quads, ol));
        }
    }

    fn cull(&self, cx: f64, cy: f64, cz: f64) -> bool {
        let c = self.to_cam((cx, cy, cz));
        if c.2 < -1.7 || c.2 > FOG_END + 2.0 {
            return true;
        }
        c.0.abs() > c.2 * 1.9 + 3.0
    }

    #[allow(clippy::too_many_arguments)]
    fn add_prism(&self, items: &mut Vec<Item>, x0: f64, x1: f64, y0: f64, y1: f64, z0: f64, z1: f64, top: Rgb, side: Rgb, outline: Option<Rgb>, texs: Option<[&str; 3]>, fine: bool, lit: Option<f64>) {
        let (px, py_, pz) = self.cam_pos;
        let sc = (x1 - x0).max(y1 - y0).max(z1 - z0);
        let sh = |k: f64| lit.unwrap_or(k);
        let mut put = |q: [V3; 4], ti: usize, col: Rgb, k: f64| match texs {
            Some(t) => self.add_face(items, q, t[ti], sh(k), outline, sc, fine),
            None => self.add_poly(items, &q, col, sh(k), outline, true),
        };
        if py_ > y1 {
            put([(x0, y1, z0), (x1, y1, z0), (x1, y1, z1), (x0, y1, z1)], 0, top, 1.0);
        } else if py_ < y0 {
            put([(x0, y0, z1), (x1, y0, z1), (x1, y0, z0), (x0, y0, z0)], 2, side, 0.50);
        }
        if px < x0 {
            put([(x0, y1, z1), (x0, y1, z0), (x0, y0, z0), (x0, y0, z1)], 1, side, 0.62);
        } else if px > x1 {
            put([(x1, y1, z0), (x1, y1, z1), (x1, y0, z1), (x1, y0, z0)], 1, side, 0.62);
        }
        if pz < z0 {
            put([(x1, y1, z0), (x0, y1, z0), (x0, y0, z0), (x1, y0, z0)], 1, side, 0.80);
        } else if pz > z1 {
            put([(x0, y1, z1), (x1, y1, z1), (x1, y0, z1), (x0, y0, z1)], 1, side, 0.80);
        }
    }

    fn add_cube(&self, items: &mut Vec<Item>, bx: i32, by: i32, bz: i32, t: [&str; 3], outline: Option<Rgb>) {
        let (px, py_, pz) = self.cam_pos;
        let (x0, x1) = (bx as f64, bx as f64 + 1.0);
        let (y0, y1) = (by as f64, by as f64 + 1.0);
        let (z0, z1) = (bz as f64, bz as f64 + 1.0);
        if py_ > y1 && !self.is_solid(bx, by + 1, bz) {
            self.add_face(items, [(x0, y1, z0), (x1, y1, z0), (x1, y1, z1), (x0, y1, z1)], t[0], 1.0, outline, 1.0, false);
        } else if py_ < y0 && !self.is_solid(bx, by - 1, bz) {
            self.add_face(items, [(x0, y0, z1), (x1, y0, z1), (x1, y0, z0), (x0, y0, z0)], t[2], 0.50, outline, 1.0, false);
        }
        if px < x0 && !self.is_solid(bx - 1, by, bz) {
            self.add_face(items, [(x0, y1, z1), (x0, y1, z0), (x0, y0, z0), (x0, y0, z1)], t[1], 0.62, outline, 1.0, false);
        } else if px > x1 && !self.is_solid(bx + 1, by, bz) {
            self.add_face(items, [(x1, y1, z0), (x1, y1, z1), (x1, y0, z1), (x1, y0, z0)], t[1], 0.62, outline, 1.0, false);
        }
        if pz < z0 && !self.is_solid(bx, by, bz - 1) {
            self.add_face(items, [(x1, y1, z0), (x0, y1, z0), (x0, y0, z0), (x1, y0, z0)], t[1], 0.80, outline, 1.0, false);
        } else if pz > z1 && !self.is_solid(bx, by, bz + 1) {
            self.add_face(items, [(x0, y1, z1), (x1, y1, z1), (x1, y0, z1), (x0, y0, z1)], t[1], 0.80, outline, 1.0, false);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn add_obox(&self, items: &mut Vec<Item>, base: V3, yaw: f64, swing: f64, lo: V3, hi: V3, tex_front: &str, tex_side: &str, tex_top: &str, tex_bottom: Option<&str>, tex_back: Option<&str>) {
        let (cs, sn) = (swing.cos(), swing.sin());
        let (fx, fz) = (yaw.sin(), yaw.cos());
        let (rx, rz) = (-fz, fx);
        let (bx, by, bz) = base;
        let w = |p: V3| -> V3 {
            let (x, y, z) = p;
            let yy = y * cs - z * sn;
            let zz = y * sn + z * cs;
            (bx + x * rx + zz * fx, by + yy, bz + x * rz + zz * fz)
        };
        let (x0, y0, z0) = lo;
        let (x1, y1, z1) = hi;
        let p = |xi: usize, yi: usize, zi: usize| w(([x0, x1][xi], [y0, y1][yi], [z0, z1][zi]));
        let cam = self.cam_pos;
        let cen = w(((x0 + x1) / 2.0, (y0 + y1) / 2.0, (z0 + z1) / 2.0));
        let mut face = |quad: [V3; 4], t: &str, shade: f64| {
            let (a, b, c) = (quad[0], quad[1], quad[3]);
            let (ux, uy, uz) = (b.0 - a.0, b.1 - a.1, b.2 - a.2);
            let (vx, vy, vz) = (c.0 - a.0, c.1 - a.1, c.2 - a.2);
            let (mut nx, mut ny, mut nz) = (uy * vz - uz * vy, uz * vx - ux * vz, ux * vy - uy * vx);
            let mid = ((quad[0].0 + quad[2].0) / 2.0, (quad[0].1 + quad[2].1) / 2.0, (quad[0].2 + quad[2].2) / 2.0);
            if nx * (mid.0 - cen.0) + ny * (mid.1 - cen.1) + nz * (mid.2 - cen.2) < 0.0 {
                nx = -nx;
                ny = -ny;
                nz = -nz;
            }
            if nx * (cam.0 - mid.0) + ny * (cam.1 - mid.1) + nz * (cam.2 - mid.2) <= 0.0 {
                return;
            }
            self.add_face(items, quad, t, shade, None, (x1 - x0).max(y1 - y0), true);
        };
        face([p(0, 1, 1), p(1, 1, 1), p(1, 0, 1), p(0, 0, 1)], tex_front, 0.94);
        face([p(1, 1, 0), p(0, 1, 0), p(0, 0, 0), p(1, 0, 0)], tex_back.unwrap_or(tex_front), 0.66);
        face([p(1, 1, 1), p(1, 1, 0), p(1, 0, 0), p(1, 0, 1)], tex_side, 0.78);
        face([p(0, 1, 0), p(0, 1, 1), p(0, 0, 1), p(0, 0, 0)], tex_side, 0.72);
        face([p(0, 1, 0), p(1, 1, 0), p(1, 1, 1), p(0, 1, 1)], tex_top, 1.0);
        face([p(0, 0, 1), p(1, 0, 1), p(1, 0, 0), p(0, 0, 0)], tex_bottom.unwrap_or(tex_side), 0.45);
    }

    fn add_sprite(&self, items: &mut Vec<Item>, center: V3, size: f64, t: &str, spin: f64) {
        let (cx, cy, cz) = center;
        let h = size * 0.5;
        let (rx, rz) = (spin.cos() * h, spin.sin() * h);
        let quad = [(cx - rx, cy + h, cz - rz), (cx + rx, cy + h, cz + rz), (cx + rx, cy - h, cz + rz), (cx - rx, cy - h, cz - rz)];
        self.add_face(items, quad, t, 1.0, None, size, true);
    }

    fn add_block(&self, items: &mut Vec<Item>, pos: (i32, i32, i32), typ: u8) {
        let (bx, by, bz) = pos;
        let (fx, fy, fz) = (bx as f64, by as f64, bz as f64);
        if self.cull(fx + 0.5, fy + 0.5, fz + 0.5) {
            return;
        }
        let line = if self.tex_mode == 0 { Some((30, 34, 40)) } else { None };
        let wood = Some(["wood", "wood", "wood"]);
        let col_ladder = avg_col("wood");
        if let Some(t) = block_tex(typ) {
            self.add_cube(items, bx, by, bz, t, line);
        } else if typ == LADDER {
            for sx in [0.14, 0.66] {
                self.add_prism(items, fx + sx, fx + sx + 0.2, fy, fy + 1.0, fz + 0.06, fz + 0.24, col_ladder, shade_col(col_ladder, 0.8), line, wood, true, None);
            }
            for ry in 0..4 {
                let yy = fy + 0.12 + ry as f64 * 0.26;
                self.add_prism(items, fx + 0.1, fx + 0.9, yy, yy + 0.09, fz + 0.03, fz + 0.27, shade_col(col_ladder, 1.15), col_ladder, None, wood, true, None);
            }
        } else if typ == FENCE {
            let cf = col_ladder;
            self.add_prism(items, fx + 0.38, fx + 0.62, fy, fy + 1.0, fz + 0.38, fz + 0.62, cf, shade_col(cf, 0.8), line, wood, true, None);
            for yy in [fy + 0.34, fy + 0.66] {
                self.add_prism(items, fx + 0.02, fx + 0.98, yy, yy + 0.13, fz + 0.44, fz + 0.56, cf, shade_col(cf, 0.85), None, wood, true, None);
                self.add_prism(items, fx + 0.44, fx + 0.56, yy, yy + 0.13, fz + 0.02, fz + 0.98, cf, shade_col(cf, 0.85), None, wood, true, None);
            }
        } else if typ == SPRING {
            let cs = avg_col("slime");
            self.add_prism(items, fx + 0.02, fx + 0.98, fy, fy + SPRING_H, fz + 0.02, fz + 0.98, cs, shade_col(cs, 0.85), line, Some(["slime", "slime", "slime"]), false, None);
        }
    }

    fn add_goal_beam(&self, items: &mut Vec<Item>) {
        let (gx, gy, gz) = self.goal;
        let p = 0.5 + 0.5 * (self.anim * 3.0).sin();
        let w = 0.24 + 0.03 * p;
        self.add_prism(items, gx - w, gx + w, gy + 0.05, gy + 18.0, gz - w, gz + w, (240, 255, 250), (215, 248, 240), None, Some(["beacon", "beacon", "beacon"]), false, Some(1.0));
    }

    fn add_player(&self, items: &mut Vec<Item>) {
        let (px, py_, pz) = (self.px, self.py, self.pz);
        let u = PLAYER_H / 32.0;
        let sp = 1f64.min(py::hypot(self.vx, self.vz) / MOVE_SPEED);
        let swing = if self.on_ground {
            self.walk.sin() * 0.85 * sp
        } else if self.vy > 0.0 {
            0.45
        } else {
            0.22
        };
        let yaw = self.yaw;
        let head_y = py_ + 24.0 * u;
        let arm_y = py_ + 23.5 * u;
        let hip_y = py_ + 12.0 * u;
        for (side, sgn) in [(-1.0, 1.0), (1.0, -1.0)] {
            self.add_obox(items, (px, hip_y, pz), yaw, swing * sgn, (side * 4.0 * u - 2.0 * u, -12.0 * u, -2.0 * u), (side * 4.0 * u + 2.0 * u, 0.0, 2.0 * u), "leg", "leg", "leg", Some("leg"), None);
        }
        self.add_obox(items, (px, py_, pz), yaw, 0.0, (-4.0 * u, 12.0 * u, -2.0 * u), (4.0 * u, 24.0 * u, 2.0 * u), "body_front", "body_side", "body_side", Some("body_side"), Some("body_side"));
        for (side, sgn) in [(-1.0, -1.0), (1.0, 1.0)] {
            self.add_obox(items, (px, arm_y, pz), yaw, swing * sgn, (side * 6.0 * u - 2.0 * u, -12.0 * u, -2.0 * u), (side * 6.0 * u + 2.0 * u, 0.0, 2.0 * u), "arm", "arm", "arm", Some("arm"), None);
        }
        self.add_obox(items, (px, head_y, pz), yaw, -self.pitch * 0.7, (-4.0 * u, 0.0, -4.0 * u), (4.0 * u, 8.0 * u, 4.0 * u), "face", "head_side", "head_top", Some("head_top"), Some("head_back"));
    }

    // ===================================================== Zeichnen

    fn draw_sky(&mut self, s: &mut Surface) {
        let (w, h) = (self.b.width, self.b.height);
        if self.sky_cache.as_ref().is_none_or(|c| c.0 != (w, h)) {
            let mut surf = pg::surface(w, h);
            let hor = (h as f64 * 0.52) as i32;
            let haze = (h as f64 * 0.66) as i32;
            let lerp = |a: Rgb, b: Rgb, tt: f64| -> Rgb {
                ((a.0 as f64 + (b.0 - a.0) as f64 * tt) as i32, (a.1 as f64 + (b.1 - a.1) as f64 * tt) as i32, (a.2 as f64 + (b.2 - a.2) as f64 * tt) as i32)
            };
            for y in 0..h {
                let c = if y < hor {
                    lerp(COL_SKY_TOP, COL_SKY_HOR, y as f64 / 1.max(hor) as f64)
                } else if y < haze {
                    lerp(COL_SKY_HOR, COL_FOG, (y - hor) as f64 / 1.max(haze - hor) as f64)
                } else {
                    COL_FOG
                };
                draw::line(&mut surf, c, (0, y), (w, y), 1);
            }
            self.sky_cache = Some(((w, h), surf));
        }
        if let Some((_, sky)) = &self.sky_cache {
            s.bl(sky, (0, 0));
        }
    }

    fn draw_sun_and_clouds(&self, s: &mut Surface) {
        let mut sky: Vec<Item> = Vec::new();
        let (cx, cy, cz) = self.cam_pos;
        let (d, r) = (260.0, 19.0);
        let sv0 = (0.52f64, 0.66f64, 0.54f64);
        let sl = py::sum(&[sv0.0 * sv0.0, sv0.1 * sv0.1, sv0.2 * sv0.2]).sqrt();
        let sv = (sv0.0 / sl, sv0.1 / sl, sv0.2 / sl);
        let sc = (cx + sv.0 * d, cy + sv.1 * d, cz + sv.2 * d);
        let (ax, ay, _fw) = self.basis;
        let m = ui::mix(COL_SUN, COL_SKY_HOR, 0.6);
        for (k, col) in [(1.7, (m.r as i32, m.g as i32, m.b as i32)), (1.0, COL_SUN)] {
            let q: Vec<V3> = [(-1.0, 1.0), (1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)]
                .iter()
                .map(|&(sx, sy)| (sc.0 + ax.0 * r * k * sx + ay.0 * r * k * sy, sc.1 + ax.1 * r * k * sx + ay.1 * r * k * sy, sc.2 + ax.2 * r * k * sx + ay.2 * r * k * sy))
                .collect();
            self.add_poly(&mut sky, &q, col, 1.0, None, false);
        }
        let cyl = self.cloud_y;
        if cy < cyl - 1.0 {
            let tt = 7.0;
            let drift = py::fpmod(self.anim * 0.35, tt * 8.0);
            let tx0 = ((cx - drift) / tt).floor() as i64;
            let tz0 = (cz / tt).floor() as i64;
            let span = 11i64;
            for tz in tz0 - span..=tz0 + span {
                for tx in tx0 - span..=tx0 + span {
                    if CLOUD_MAP[py::pmod(tz, 8) as usize].as_bytes()[py::pmod(tx, 8) as usize] == b'0' {
                        continue;
                    }
                    let x0 = tx as f64 * tt + drift;
                    let z0 = tz as f64 * tt;
                    let (mx, mz) = (x0 + tt * 0.5, z0 + tt * 0.5);
                    let dist = py::hypot(mx - cx, mz - cz);
                    if dist > span as f64 * tt {
                        continue;
                    }
                    let c = self.to_cam((mx, cyl, mz));
                    if c.2 < 0.5 || c.0.abs() > c.2 * 2.0 + tt {
                        continue;
                    }
                    let ft = 1f64.min(py::pow(dist / (span as f64 * tt), 2.6));
                    let col = ui::mix((250, 251, 255), COL_FOG, ft);
                    self.add_poly(&mut sky, &[(x0, cyl, z0), (x0 + tt, cyl, z0), (x0 + tt, cyl, z0 + tt), (x0, cyl, z0 + tt)], (col.r as i32, col.g as i32, col.b as i32), 0.94, None, false);
                }
            }
        }
        sky.sort_by(|a, b| (-a.0).partial_cmp(&-b.0).unwrap_or(std::cmp::Ordering::Equal));
        for (_, quads, _) in &sky {
            for (pts, col) in quads {
                draw::polygon(s, *col, pts, 0);
            }
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

    fn draw_hand(&self, s: &mut Surface) {
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        let sp = 1f64.min(py::hypot(self.vx, self.vz) / MOVE_SPEED);
        let bx = self.walk.sin() * 0.020 * w * sp;
        let mut by = self.walk.cos().abs() * 0.030 * h * sp;
        if !self.on_ground {
            by -= 0.02 * h;
        }
        let (ax, ay) = (w * 0.97 + bx, h * 1.16 + by);
        let (fx, fy) = (w * 0.79 + bx, h * 0.78 + by);
        let (mut dx, mut dy) = (fx - ax, fy - ay);
        let mut ln = py::hypot(dx, dy);
        if ln == 0.0 {
            ln = 1.0;
        }
        dx /= ln;
        dy /= ln;
        let (nx, ny) = (-dy, dx);
        let hw = h * 0.055;
        let top = (fx, fy);
        let p0 = (ax + dx * ln * 0.0, ay + dy * ln * 0.0);
        let p1 = (ax + dx * ln * 1.0, ay + dy * ln * 1.0);
        let quad = [(p1.0 + nx * hw, p1.1 + ny * hw), (p1.0 - nx * hw, p1.1 - ny * hw), (p0.0 - nx * hw, p0.1 - ny * hw), (p0.0 + nx * hw, p0.1 + ny * hw)];
        self.blit_tex_quad(s, quad, "arm", 0.98);
        let edge = [(fx - nx * hw, fy - ny * hw), (fx - nx * hw - dx * h * 0.02, fy - ny * hw - dy * h * 0.02), (ax - nx * hw - dx * h * 0.02, ay - ny * hw - dy * h * 0.02), (ax - nx * hw, ay - ny * hw)];
        self.blit_tex_quad(s, edge, "arm", 0.62);
        let cap = [
            (top.0 + nx * hw, top.1 + ny * hw),
            (top.0 - nx * hw, top.1 - ny * hw),
            (top.0 - nx * hw + dx * -h * 0.035, top.1 - ny * hw + dy * -h * 0.035),
            (top.0 + nx * hw + dx * -h * 0.035, top.1 + ny * hw + dy * -h * 0.035),
        ];
        self.blit_tex_quad(s, cap, "arm", 1.12);
    }

    fn blit_tex_quad(&self, s: &mut Surface, quad: [(f64, f64); 4], texn: &str, shade: f64) {
        if self.tex_mode == 0 {
            draw::polygon(s, shade_col(avg_col(texn), shade), &quad, 0);
            return;
        }
        let n = (if self.tex_mode == 2 { 8 } else { 4 }).min(tex(texn).len());
        let [a, b, c, d] = quad;
        let nf = n as f64;
        let left: Vec<(f64, f64)> = (0..=n).map(|j| (a.0 + (d.0 - a.0) * j as f64 / nf, a.1 + (d.1 - a.1) * j as f64 / nf)).collect();
        let right: Vec<(f64, f64)> = (0..=n).map(|j| (b.0 + (c.0 - b.0) * j as f64 / nf, b.1 + (c.1 - b.1) * j as f64 / nf)).collect();
        let rows: Vec<Vec<(f64, f64)>> = (0..=n)
            .map(|j| (0..=n).map(|i| (left[j].0 + (right[j].0 - left[j].0) * i as f64 / nf, left[j].1 + (right[j].1 - left[j].1) * i as f64 / nf)).collect())
            .collect();
        for (i0, j0, i1, j1, col) in tex_grid(texn, n, shade, 0.0) {
            let (r0, r1) = (&rows[j0], &rows[j1]);
            draw::polygon(s, col, &[r0[i0], r0[i1], r1[i1], r1[i0]], 0);
        }
    }

    fn draw_crosshair(&self, s: &mut Surface) {
        let (cx, cy) = (self.b.width / 2, self.b.height / 2);
        for (col, off, ln, wd) in [((26, 28, 34), 1, 10, 4), ((242, 242, 242), 0, 9, 2)] {
            draw::line(s, col, (cx - ln, cy + off), (cx + ln, cy + off), wd);
            draw::line(s, col, (cx + off, cy - ln), (cx + off, cy + ln), wd);
        }
    }

    fn shadow_text(&self, s: &mut Surface, font: &Font, txt: &str, col: Color, pos: (i32, i32), anchor: &str) -> Rect {
        let sh = font.render(txt, true, rgb(28, 28, 32));
        let img = font.render(txt, true, col);
        let r0 = img.get_rect();
        let r = match anchor {
            "center" => r0.center_at(pos),
            "midright" => r0.midright_at(pos),
            "midbottom" => r0.midbottom_at(pos),
            _ => r0.midleft_at(pos),
        };
        s.bl(&sh, (r.x + 2, r.y + 2));
        s.blr(&img, r);
        r
    }

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        self.shadow_text(s, &self.hud, &tf("blj.level", &[("n", &self.level.to_string())]), self.b.accent, (14, 20), "midleft");
        self.shadow_text(s, &self.hud, &tf("common.points", &[("score", &self.b.score.to_string())]), rgb(245, 245, 245), (w / 2, 20), "center");
        let px = 2.max(h / 230);
        let hw = 10 * px;
        let y = h - 14 - 9 * px;
        for i in 0..3 {
            let full = i < self.lives;
            for (j, row) in HEART_ART.iter().enumerate() {
                for (k, ch) in row.chars().enumerate() {
                    let c = match (ch, full) {
                        ('#', true) => Some((54, 16, 16)),
                        ('r', true) => Some((226, 58, 52)),
                        ('#', false) => Some((34, 34, 38)),
                        ('r', false) => Some((72, 72, 78)),
                        _ => None,
                    };
                    if let Some(c) = c {
                        draw::rect(s, c, Rect::new(14 + i * (hw + px * 2) + k as i32 * px, y + j as i32 * px, px, px), 0);
                    }
                }
            }
        }
        let ing = mip("ingot", 8);
        let ix = w - 14 - 8 * px;
        for (j, row) in ing.iter().enumerate() {
            for (i, c) in row.iter().enumerate() {
                if let Some(c) = c {
                    draw::rect(s, *c, Rect::new(ix + i as i32 * px, y + j as i32 * px, px, px), 0);
                }
            }
        }
        self.shadow_text(s, &self.hud, &self.coins_level.to_string(), p.gold, (ix - 8, y + 4 * px), "midright");
        if self.state == State::Play {
            let mdir = if self.invert { t("common.dir_inverted") } else { t("common.dir_normal") };
            let hint = tf(
                "blj.hud_hint",
                &[
                    ("view", &t(if self.first_person { "blj.view_1p" } else { "blj.view_3p" })),
                    ("tex", &t(&format!("blj.tex_{}", TEX_MODES[self.tex_mode]))),
                    ("dir", &mdir),
                ],
            );
            let mut lines = vec![hint.clone()];
            if self.small.size(&hint).0 > w - 28 {
                let parts: Vec<&str> = hint.split(" · ").collect();
                let hh = (parts.len() + 1) / 2;
                lines = vec![parts[..hh].join(" · "), parts[hh..].join(" · ")];
            }
            let sh = self.small.get_height();
            let mut yy = h - 10 - (lines.len() as i32 - 1) * (sh + 2);
            for ln in &lines {
                self.shadow_text(s, &self.small, ln, rgb(226, 226, 230), (w / 2, yy), "midbottom");
                yy += sh + 2;
            }
        }
    }

    fn banner(&self, s: &mut Surface, title: &str, color: Color, sub: &str) {
        let w = (self.b.width - 40).min(620);
        let h = 124;
        let rc = Rect::new((self.b.width - w) / 2, (self.b.height - h) / 2, w, h);
        let mut panel = pg::surface_alpha(w, h);
        panel.fill_c(rgba(16, 16, 20, 226));
        s.bl(&panel, (rc.x, rc.y));
        draw::rect(s, rgb(78, 78, 86), rc, 4);
        draw::rect(s, color, rc.inflate(-8, -8), 2);
        self.shadow_text(s, &self.huge, title, color, (rc.centerx(), rc.y + 44), "center");
        self.shadow_text(s, &self.small, sub, rgb(232, 232, 236), (rc.centerx(), rc.y + 88), "center");
    }
}

impl Game for BlockJumpGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        if !matches!(self.b.mode.as_str(), "easy" | "normal" | "hard") {
            self.b.mode = "normal".into();
        }
        self.blur = 0f64.max(0.8f64.min(self.b.opt_float_py("blockjump", "blur", 0.35).unwrap_or(0.35)));
        self.first_person = self.b.opt_str("blockjump", "view", "first") != "third";
        self.sens = 0.4f64.max(2.5f64.min(self.b.opt_float_py("blockjump", "sens", 1.0).unwrap_or(1.0)));
        self.invert = self.b.opt_value("blockjump", "mouse_invert").is_some_and(|v| match v {
            Value::Bool(b) => b,
            Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
            Value::String(s) => !s.is_empty(),
            Value::Null => false,
            Value::Array(a) => !a.is_empty(),
            Value::Object(o) => !o.is_empty(),
        });
        let tx = self.b.opt_str("blockjump", "textures", "high");
        self.tex_mode = TEX_MODES.iter().position(|m| *m == tx).unwrap_or(2);
        self.make_fonts();
        self.sky_cache = None;
        self.prev_frame = None;
        self.anim = 0.0;
        self.walk = 0.0;
        self.held.clear();
        self.capture = false;
        self.want_capture = true;
        self.new_run();
    }

    fn on_surface_changed(&mut self) {
        self.make_fonts();
        self.sky_cache = None;
        self.prev_frame = None;
    }

    fn capture_mouse(&self) -> bool {
        self.capture
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        match ev.kind {
            Kind::KeyDown => {
                let k = ev.key.as_str();
                if matches!(self.state, State::Ready | State::GameOver) {
                    if k == "Return" || k == "space" {
                        self.start_or_restart();
                    }
                    return;
                }
                for a in self.move_acts(k) {
                    if !self.held.contains(&a) {
                        self.held.push(a);
                    }
                }
                if k == "space" || self.b.is_action(k, "action", None) {
                    self.jump();
                } else if k == "v" || k == "V" {
                    self.first_person = !self.first_person;
                    self.save_setting("view", json!(if self.first_person { "first" } else { "third" }));
                } else if k == "b" || k == "B" {
                    self.blur = if self.blur >= 0.79 { 0.0 } else { py::round_to(self.blur + 0.2, 2) };
                    self.save_setting("blur", json!(self.blur));
                } else if k == "t" || k == "T" {
                    self.tex_mode = (self.tex_mode + 2) % 3;
                    self.save_setting("textures", json!(TEX_MODES[self.tex_mode]));
                    self.b.play_sound("click");
                } else if k == "c" || k == "C" {
                    self.want_capture = !self.want_capture;
                } else if k == "i" || k == "I" {
                    self.invert = !self.invert;
                    self.save_setting("mouse_invert", json!(self.invert));
                    self.b.play_sound("click");
                } else if matches!(k, "plus" | "KP_Add" | "equal") {
                    self.sens = py::round_to(0.4f64.max(2.5f64.min(self.sens + 0.1)), 1);
                    self.save_setting("sens", json!(self.sens));
                } else if matches!(k, "minus" | "KP_Subtract") {
                    self.sens = py::round_to(0.4f64.max(2.5f64.min(self.sens - 0.1)), 1);
                    self.save_setting("sens", json!(self.sens));
                }
            }
            Kind::KeyUp => {
                let acts = self.move_acts(&ev.key);
                self.held.retain(|a| !acts.contains(a));
            }
            Kind::MouseDown => {
                if matches!(self.state, State::Ready | State::GameOver) {
                    self.start_or_restart();
                } else if self.state == State::Play && !self.capture {
                    self.want_capture = true;
                    self.capture = true;
                }
            }
            Kind::MouseRel if self.state == State::Play => self.apply_look(ev.rel),
            _ => {}
        }
    }

    fn update(&mut self, dt: f64) {
        self.anim += dt;
        if self.state == State::Clear {
            self.clear_t -= dt;
            if self.clear_t <= 0.0 {
                self.level += 1;
                let l = self.level;
                self.build_level(l);
                self.state = State::Play;
            }
            return;
        }
        if self.state != State::Play {
            return;
        }
        self.level_time += dt;
        self.physics(dt.min(0.05));
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        self.capture = self.want_capture && self.state == State::Play && !self.b.paused;
        self.update_cam();
        self.scx = self.b.width as f64 / 2.0;
        self.scy = self.b.height as f64 * 0.5;
        self.f = self.b.height as f64 * FOV_MUL;
        self.basis = self.view_basis();
        self.draw_sky(&mut s);
        self.draw_sun_and_clouds(&mut s);
        let mut items: Vec<Item> = Vec::new();
        for k in &self.world_keys {
            let typ = self.world[k];
            self.add_block(&mut items, *k, typ);
        }
        for &coin in &self.coins {
            let cy = coin.1 + 0.12 * (self.anim * 2.0 + coin.0).sin();
            self.add_sprite(&mut items, (coin.0, cy, coin.2), 0.45, "ingot", self.anim * 1.9 + coin.0 + coin.2);
        }
        self.add_goal_beam(&mut items);
        if !self.first_person {
            self.add_player(&mut items);
        }
        items.sort_by(|a, b| (-a.0).partial_cmp(&-b.0).unwrap_or(std::cmp::Ordering::Equal));
        for (_, quads, outline) in &items {
            for (pts, col) in quads {
                draw::polygon(&mut s, *col, pts, 0);
            }
            if let Some((pts, col)) = outline {
                draw::polygon(&mut s, *col, pts, 1);
            }
        }
        self.apply_blur(&mut s);
        if self.first_person && self.state != State::GameOver {
            self.draw_hand(&mut s);
            if self.state == State::Play {
                self.draw_crosshair(&mut s);
            }
        }
        self.draw_hud(&mut s);
        match self.state {
            State::Ready => self.banner(&mut s, &t("blj.ready"), self.b.accent, &t("blj.controls")),
            State::Clear => self.banner(&mut s, &tf("blj.clear", &[("n", &self.level.to_string())]), rgb(120, 235, 170), &tf("blj.clear_sub", &[("pts", &(1000 + self.last_bonus).to_string())])),
            State::GameOver => {
                let sub = format!("{}   ·   {}", tf("common.points", &[("score", &self.b.score.to_string())]), t("common.enter_restart"));
                self.banner(&mut s, &t("blj.gameover"), rgb(232, 96, 96), &sub);
            }
            State::Play => {}
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "blockjump",
    class_name: "BlockJumpGame",
    supports_multiplayer: false,
    modes: &[("easy", "blj.mode.easy"), ("normal", "blj.mode.normal"), ("hard", "blj.mode.hard")],
    make: BlockJumpGame::new,
};
