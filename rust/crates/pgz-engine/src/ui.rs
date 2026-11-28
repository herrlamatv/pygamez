//! ui.rs
//! =====
//! Gegenstueck zu `ui.py` - das gemeinsame UI-Werkzeug fuer alle Bildschirme
//! und Spiele: zehn Designs, Paletten, Schriften, Hintergrund, Glas-Panels,
//! Knoepfe, Titel, Fusszeile, Partikel, Uebergaenge und das Eingabefeld.
//!
//! **Zeile fuer Zeile aus Python**, auch die Reihenfolge der Zeichenaufrufe
//! und der Zufallszahlen. Auf rustgame gerendert ergibt das dieselben Pixel
//! wie `ui.py` auf pygame.
//!
//! Die Paletten und Effekt-Schalter kommen aus `pgz_core::defs` (erzeugt von
//! `tools/export_defs.py` aus `ui.py`), nichts davon ist abgetippt. Was
//! `ui.py` selbst als Rechenvorschrift enthaelt (Aurora-Tabelle, Muster,
//! Kornverteilung), steht hier als Code.
//!
//! Der Zustand (aktives Design, Zwischenspeicher, Partikel) liegt wie in
//! Python modulweit - hier je Strang in einem `thread_local`.

use std::cell::RefCell;
use std::collections::HashMap;

use pgz_core::defs::{self, Fx, MeshBlob, Rgb, Theme};

use crate::game::{InputEvent, Kind};
use crate::pg::{
    self, draw, rgb, transform, Col, Color, Font, IntoRect, Num, Pt, Rect, RectExt, SurfExt,
    Surface, BLEND_ADD, BLEND_RGBA_MAX, BLEND_RGBA_MIN, BLEND_RGBA_MULT, BLEND_RGB_ADD,
    BLEND_RGB_SUB,
};
use crate::py::{self, Random};

// ---------------------------------------------------------------------------
//  Paletten
// ---------------------------------------------------------------------------

/// Die Palette des aktiven Designs (`ui.BG_TOP`, `ui.TEXT`, ...).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct Pal {
    pub bg_top: Color,
    pub bg_bottom: Color,
    pub panel: Color,
    pub panel_light: Color,
    pub border: Color,
    pub border_light: Color,
    pub btn: Color,
    pub btn_sel: Color,
    pub accent: Color,
    pub accent2: Color,
    pub accent_soft: Color,
    pub green: Color,
    pub gold: Color,
    pub red: Color,
    pub text: Color,
    pub text_dim: Color,
    pub text_faint: Color,
}

/// Die Farben der Seitenleiste (`ui.tk_colors()`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct TkPal {
    pub sidebar: Color,
    pub header: Color,
    pub card: Color,
    pub btn: Color,
    pub btn_hover: Color,
    pub accent: Color,
    pub accent2: Color,
    pub danger: Color,
    pub danger_hover: Color,
    pub back: Color,
    pub back_hover: Color,
    pub text: Color,
    pub text_dim: Color,
    pub text_faint: Color,
    pub border: Color,
    pub green: Color,
    pub gold: Color,
    pub red: Color,
}

/// `(r, g, b)` aus den Stammdaten als Farbe.
pub const fn c3(v: Rgb) -> Color {
    Color::rgb(v.0, v.1, v.2)
}

fn pal_aus(t: &Theme) -> Pal {
    let p = &t.palette;
    Pal {
        bg_top: c3(p.bg_top),
        bg_bottom: c3(p.bg_bottom),
        panel: c3(p.panel),
        panel_light: c3(p.panel_light),
        border: c3(p.border),
        border_light: c3(p.border_light),
        btn: c3(p.btn),
        btn_sel: c3(p.btn_sel),
        accent: c3(p.accent),
        accent2: c3(p.accent2),
        accent_soft: c3(p.accent_soft),
        green: c3(p.green),
        gold: c3(p.gold),
        red: c3(p.red),
        text: c3(p.text),
        text_dim: c3(p.text_dim),
        text_faint: c3(p.text_faint),
    }
}

fn tk_aus(t: &Theme) -> TkPal {
    let k = &t.tk;
    TkPal {
        sidebar: c3(k.sidebar),
        header: c3(k.header),
        card: c3(k.card),
        btn: c3(k.btn),
        btn_hover: c3(k.btn_hover),
        accent: c3(k.accent),
        accent2: c3(k.accent2),
        danger: c3(k.danger),
        danger_hover: c3(k.danger_hover),
        back: c3(k.back),
        back_hover: c3(k.back_hover),
        text: c3(k.text),
        text_dim: c3(k.text_dim),
        text_faint: c3(k.text_faint),
        border: c3(k.border),
        green: c3(k.green),
        gold: c3(k.gold),
        red: c3(k.red),
    }
}

/// Die Aurora-Lichter von UI v3 (`ui._AURORA`):
/// (Farbe, Groesse rel. max(w,h), Tempo, Phase, x, y).
const AURORA: &[MeshBlob] = &[
    MeshBlob { color: (26, 48, 95), size: 1.10, speed: 0.11, phase: 0.0, x: 0.22, y: 0.28 },
    MeshBlob { color: (52, 34, 96), size: 0.95, speed: 0.14, phase: 2.1, x: 0.80, y: 0.30 },
    MeshBlob { color: (16, 58, 62), size: 0.85, speed: 0.08, phase: 4.2, x: 0.50, y: 0.88 },
];

// ---------------------------------------------------------------------------
//  Der Modulzustand
// ---------------------------------------------------------------------------

/// Schluessel des Glas-Zwischenspeichers (`_glass_cache` in Python).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum GlassKey {
    Ov(i32, i32, i32),
    Panel(i32, i32, i32, Color, Color),
    Shadow(i32, i32, i32, u8),
    BtnGlow(i32, i32, i32, Color),
    Strip(i32, i32, [Color; 3]),
    StripGlow(i32, i32, [Color; 3]),
    Hair(i32, Color),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TextFxKey {
    Grad(usize, String, Color, Color),
    Glow(usize, String, Color),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PartKind {
    Spark,
    Confetti,
}

#[derive(Debug, Clone, Copy)]
struct Particle {
    kind: PartKind,
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    age: f64,
    life: f64,
    color: Color,
    size: f64,
    spin: f64,
    phase: f64,
}

#[derive(Debug, Clone, Copy)]
struct Shoot {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    t0: u64,
    dur: i64,
}

struct State {
    theme: &'static Theme,
    pal: Pal,
    fx: Fx,
    tk: TkPal,
    fonts: HashMap<(i32, bool, bool), Font>,
    last_ms: u64,
    frame_dt: f64,
    bg_cache: HashMap<(i32, i32), Surface>,
    stars: Vec<(f64, f64, f64, i32)>,
    pattern_cache: HashMap<(Color, Color, i32), Surface>,
    glow_cache: HashMap<(Color, i32), Surface>,
    grain_tiles: HashMap<i32, (Surface, Surface, Surface)>,
    grain_cache: HashMap<(i32, i32), (Surface, Surface)>,
    glass: HashMap<GlassKey, (Surface, u64)>,
    glass_stamp: u64,
    glass_px: i64,
    glass_ms: i64,
    glass_left: i32,
    shoot: Option<Shoot>,
    shoot_next_ms: u64,
    celestial: HashMap<(u8, i32), Surface>,
    btn_anim: HashMap<(i32, i32, i32, i32), f64>,
    text_fx: HashMap<TextFxKey, Surface>,
    particles: Vec<Particle>,
    trans_start: Option<u64>,
    trans_dur: f64,
    fade_cache: HashMap<(i32, i32), Surface>,
}

const GLASS_MAX_ITEMS: usize = 64;
const GLASS_MAX_PX: i64 = 6_000_000;
const GLASS_MISS_BUDGET: i32 = 8;
const STAR_COUNT: usize = 90;
const PATTERN_UNIT: i32 = 17;
const GRAIN_TILE: i32 = 128;
const GRAIN_SEED: u64 = 0x6A1A55;
const MAX_PARTICLES: usize = 420;

impl State {
    fn neu() -> State {
        let theme = theme_def(defs::DEFAULT_THEME);
        State {
            theme,
            pal: pal_aus(theme),
            fx: theme.fx,
            tk: tk_aus(theme),
            fonts: HashMap::new(),
            last_ms: 0,
            frame_dt: 1.0 / 60.0,
            bg_cache: HashMap::new(),
            stars: Vec::new(),
            pattern_cache: HashMap::new(),
            glow_cache: HashMap::new(),
            grain_tiles: HashMap::new(),
            grain_cache: HashMap::new(),
            glass: HashMap::new(),
            glass_stamp: 0,
            glass_px: 0,
            glass_ms: -999,
            glass_left: GLASS_MISS_BUDGET,
            shoot: None,
            shoot_next_ms: 4000,
            celestial: HashMap::new(),
            btn_anim: HashMap::new(),
            text_fx: HashMap::new(),
            particles: Vec::new(),
            trans_start: None,
            trans_dur: 0.35,
            fade_cache: HashMap::new(),
        }
    }
}

thread_local! {
    static ST: RefCell<State> = RefCell::new(State::neu());
}

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    ST.with(|s| f(&mut s.borrow_mut()))
}

fn theme_def(name: &str) -> &'static Theme {
    defs::THEMES
        .iter()
        .find(|t| t.name == name)
        .unwrap_or_else(|| {
            defs::THEMES
                .iter()
                .find(|t| t.name == defs::DEFAULT_THEME)
                .expect("Standard-Design fehlt in defs.rs")
        })
}

// ---------------------------------------------------------------------------
//  Design umschalten und abfragen
// ---------------------------------------------------------------------------

/// Die Namen der Designs in Anzeige-Reihenfolge (`ui.THEME_NAMES`).
pub fn theme_names() -> Vec<&'static str> {
    defs::THEMES.iter().map(|t| t.name).collect()
}

/// `ui.set_theme(name)` - aktiviert ein Design und leert alle Zwischenspeicher.
pub fn set_theme(name: &str) {
    with(|s| {
        let t = theme_def(name);
        s.theme = t;
        s.pal = pal_aus(t);
        s.fx = t.fx;
        s.tk = tk_aus(t);
        s.bg_cache.clear();
        s.glow_cache.clear();
        s.text_fx.clear();
        s.fade_cache.clear();
        s.btn_anim.clear();
        s.glass.clear();
        s.glass_px = 0;
        s.grain_cache.clear();
    });
}

/// `ui.theme_name()`
pub fn theme_name() -> &'static str {
    with(|s| s.theme.name)
}

/// `ui.is_modern()` - die ruhige Modern-Familie (alles ausser classic/v2/v1).
pub fn is_modern() -> bool {
    !matches!(theme_name(), "classic" | "v2" | "v1")
}

/// Die Effekt-Schalter des aktiven Designs (`ui.fx(...)`).
pub fn fx() -> Fx {
    with(|s| s.fx)
}

/// Die Palette des aktiven Designs.
pub fn pal() -> Pal {
    with(|s| s.pal)
}

/// `ui.tk_colors()` - Farben der Seitenleiste.
pub fn tk_colors() -> TkPal {
    with(|s| s.tk)
}

macro_rules! farbe {
    ($($name:ident),*) => {$(
        #[doc = concat!("`ui.", stringify!($name), "` des aktiven Designs.")]
        pub fn $name() -> Color { with(|s| s.pal.$name) }
    )*};
}
farbe!(
    bg_top, bg_bottom, panel, panel_light, border, border_light, btn, btn_sel, accent,
    accent2, accent_soft, green, gold, red, text, text_dim, text_faint
);

/// `ui.game_color(cls_name)` ueber den Klassennamen der Python-Fassung.
pub fn game_color(class_name: &str) -> Color {
    defs::GAMES
        .iter()
        .find(|g| g.class_name == class_name)
        .map(|g| hex_to_rgb(g.color))
        .unwrap_or_else(accent)
}

/// Akzentfarbe ueber den Highscore-Schluessel (`"snake"`).
pub fn game_color_key(key: &str) -> Color {
    defs::GAMES
        .iter()
        .find(|g| g.key == key)
        .map(|g| hex_to_rgb(g.color))
        .unwrap_or_else(accent)
}

// ---------------------------------------------------------------------------
//  Schriften
// ---------------------------------------------------------------------------

/// Schriftname mit Rueckfallkette (`ui.FONT_UI`).
pub const FONT_UI: &str = "bahnschrift,segoeui,consolas";
/// Feste Zeichenbreite (`ui.FONT_MONO`).
pub const FONT_MONO: &str = "consolas,menlo,monospace";

/// `ui.font(size, bold, mono)` - gecachte Schrift.
pub fn font(size: i32, bold: bool, mono: bool) -> Font {
    let key = (size, bold, mono);
    if let Some(f) = with(|s| s.fonts.get(&key).cloned()) {
        return f;
    }
    let f = Font::sys(if mono { FONT_MONO } else { FONT_UI }, size, bold, false);
    with(|s| s.fonts.insert(key, f.clone()));
    f
}

/// `ui.font(size)`
pub fn f(size: i32) -> Font {
    font(size, false, false)
}

/// `ui.font(size, bold=True)`
pub fn fb(size: i32) -> Font {
    font(size, true, false)
}

/// `ui.font(size, mono=True)`
pub fn fm(size: i32) -> Font {
    font(size, false, true)
}

// ---------------------------------------------------------------------------
//  Farb-Helfer
// ---------------------------------------------------------------------------

/// `ui.hex_to_rgb('#rrggbb')`
pub fn hex_to_rgb(value: &str) -> Color {
    let v = value.trim_start_matches('#');
    let k = |i: usize| u8::from_str_radix(v.get(i..i + 2).unwrap_or("00"), 16).unwrap_or(0);
    Color::rgb(k(0), k(2), k(4))
}

/// `ui.mix(c1, c2, f)` - lineare Mischung, `f` in 0..1.
pub fn mix(c1: impl Col, c2: impl Col, f: f64) -> Color {
    let (a, b) = (c1.c(), c2.c());
    let f = f.clamp(0.0, 1.0);
    let m = |x: u8, y: u8| -> i32 { (x as f64 + (y as i32 - x as i32) as f64 * f) as i32 };
    rgb(m(a.r, b.r), m(a.g, b.g), m(a.b, b.b))
}

// ---------------------------------------------------------------------------
//  Bildtakt
// ---------------------------------------------------------------------------

fn tick() {
    let now = pg::ticks();
    with(|s| {
        if s.last_ms != 0 {
            s.frame_dt = ((now as f64 - s.last_ms as f64) / 1000.0).clamp(0.001, 0.05);
        }
        s.last_ms = now;
    });
}

fn frame_dt() -> f64 {
    with(|s| s.frame_dt)
}

// ---------------------------------------------------------------------------
//  Hintergrund
// ---------------------------------------------------------------------------

fn gradient(w: i32, h: i32, top: Color, bottom: Color) -> Surface {
    let mut surf = pg::surface(w, h);
    for y in 0..h {
        let f = y as f64 / (1.max(h - 1)) as f64;
        draw::line(&mut surf, mix(top, bottom, f), (0, y), (w, y), 1);
    }
    surf
}

fn zigzag_tile(main: Color, alt: Color, unit: i32) -> Surface {
    let key = (main, alt, unit);
    if let Some(t) = with(|s| s.pattern_cache.get(&key).cloned()) {
        return t;
    }
    let ss = 4;
    let u = unit * ss;
    let hu = u / 2;
    let mut block = pg::surface(6 * u, 3 * u);
    block.fill_c(alt);
    for ty in 0..3 {
        for tx in 0..3 {
            let (ox, oy) = (tx * 2 * u, ty * u);
            draw::polygon(&mut block, main, &[(ox + u, oy), (ox, oy + u), (ox + 2 * u, oy + u)], 0);
            draw::polygon(
                &mut block,
                alt,
                &[(ox + u, oy + hu), (ox + hu, oy + u), (ox + u + hu, oy + u)],
                0,
            );
            draw::polygon(&mut block, main, &[(ox, oy), (ox + hu, oy + hu), (ox, oy + hu)], 0);
            draw::polygon(
                &mut block,
                main,
                &[(ox + 2 * u, oy), (ox + u + hu, oy + hu), (ox + 2 * u, oy + hu)],
                0,
            );
        }
    }
    let small = transform::smoothscale(&block, 6 * unit, 3 * unit);
    let mut tile = pg::surface(2 * unit, unit);
    tile.bl_area(&small, (0, 0), (2 * unit, unit, 2 * unit, unit));
    with(|s| {
        if s.pattern_cache.len() > 8 {
            s.pattern_cache.clear();
        }
        s.pattern_cache.insert(key, tile.clone());
    });
    tile
}

/// `ui.draw_zigzag(surface, w, h, main, alt)`
pub fn draw_zigzag(surface: &mut Surface, w: i32, h: i32, main: Color, alt: Color) {
    let tile = zigzag_tile(main, alt, PATTERN_UNIT);
    let (tw, th) = (tile.w(), tile.h());
    let mut row = pg::surface(w, th);
    let mut x = 0;
    while x < w {
        row.bl(&tile, (x, 0));
        x += tw;
    }
    let mut y = 0;
    while y < h {
        surface.bl(&row, (0, y));
        y += th;
    }
}

fn base_background(w: i32, h: i32) -> Surface {
    if let Some(s) = with(|s| s.bg_cache.get(&(w, h)).cloned()) {
        return s;
    }
    let fxx = fx();
    let p = pal();
    let mut surf = if let Some((m, a)) = fxx.pattern {
        let mut s = pg::surface(w, h);
        draw_zigzag(&mut s, w, h, c3(m), c3(a));
        s
    } else {
        gradient(w, h, p.bg_top, p.bg_bottom)
    };
    let (sw, sh) = (8.max(w / 8), 8.max(h / 8));
    let mut shade = pg::surface_alpha(sw, sh);
    shade.fill_c(Color::rgba(0, 0, 0, fxx.vignette));
    draw::ellipse(
        &mut shade,
        Color::rgba(0, 0, 0, 0),
        (py::fdiv32(-sw, 3), py::fdiv32(-sh, 3), sw + 2 * sw / 3, sh + 2 * sh / 3),
        0,
    );
    surf.bl(&transform::smoothscale(&shade, w, h), (0, 0));
    with(|s| {
        if s.bg_cache.len() > 6 {
            s.bg_cache.clear();
        }
        s.bg_cache.insert((w, h), surf.clone());
    });
    surf
}

fn glow_surface(color: Color, size: i32) -> Surface {
    let size = 32.max((size / 32) * 32);
    let key = (color, size);
    if let Some(s) = with(|s| s.glow_cache.get(&key).cloned()) {
        return s;
    }
    let mut base = pg::surface(96, 96);
    for r in (1..=48).rev() {
        let f = py::pow(1.0 - r as f64 / 48.0, 2.0);
        draw::circle(
            &mut base,
            rgb(
                (color.r as f64 * f) as i32,
                (color.g as f64 * f) as i32,
                (color.b as f64 * f) as i32,
            ),
            (48, 48),
            r,
            0,
        );
    }
    let surf = transform::smoothscale(&base, size, size);
    with(|s| {
        if s.glow_cache.len() > 10 {
            s.glow_cache.clear();
        }
        s.glow_cache.insert(key, surf.clone());
    });
    surf
}

// ----- Filmkorn ---------------------------------------------------------

fn tri_tables(strength: i32) -> ([u8; 256], [u8; 256], [u8; 256]) {
    let mut up = [0u8; 256];
    let mut down = [0u8; 256];
    let mut alpha = [0u8; 256];
    let st = strength as f64;
    for i in 0..256 {
        let u = (i as f64 + 0.5) / 256.0;
        let x = if u < 0.5 {
            -st * (1.0 - (2.0 * u).sqrt())
        } else {
            st * (1.0 - (2.0 * (1.0 - u)).sqrt())
        };
        up[i] = py::round(x).clamp(0, 255) as u8;
        down[i] = py::round(-x).clamp(0, 255) as u8;
        alpha[i] = py::round(x.abs() * 2.2).clamp(0, 255) as u8;
    }
    (up, down, alpha)
}

fn grain_tiles(strength: i32) -> (Surface, Surface, Surface) {
    let key = 1.max(strength);
    if let Some(t) = with(|s| s.grain_tiles.get(&key).cloned()) {
        return t;
    }
    let n = (GRAIN_TILE * GRAIN_TILE) as usize;
    let raw = Random::new(GRAIN_SEED).randbytes(n);
    let (t_up, t_down, t_alpha) = tri_tables(key);
    let rgb_tile = |table: &[u8; 256]| -> Surface {
        let mut buf = Vec::with_capacity(3 * n);
        for b in &raw {
            let c = table[*b as usize];
            buf.extend_from_slice(&[c, c, c]);
        }
        pg::image::frombytes(&buf, GRAIN_TILE, GRAIN_TILE, "RGB")
    };
    let mut buf = Vec::with_capacity(4 * n);
    for b in &raw {
        let col = if *b < 128 { 0 } else { 255 };
        buf.extend_from_slice(&[col, col, col, t_alpha[*b as usize]]);
    }
    let tiles = (
        rgb_tile(&t_up),
        rgb_tile(&t_down),
        pg::image::frombytes(&buf, GRAIN_TILE, GRAIN_TILE, "RGBA"),
    );
    with(|s| {
        if s.grain_tiles.len() > 4 {
            s.grain_tiles.clear();
        }
        s.grain_tiles.insert(key, tiles.clone());
    });
    tiles
}

fn tile_fill(tile: &Surface, w: i32, h: i32) -> Surface {
    let (tw, th) = (tile.w(), tile.h());
    let mut row = pg::surface(w, th);
    let mut x = 0;
    while x < w {
        row.bl(tile, (x, 0));
        x += tw;
    }
    let mut surf = pg::surface(w, h);
    let mut y = 0;
    while y < h {
        surf.bl(&row, (0, y));
        y += th;
    }
    surf
}

fn grain_surfaces(w: i32, h: i32) -> (Surface, Surface) {
    if let Some(p) = with(|s| s.grain_cache.get(&(w, h)).cloned()) {
        return p;
    }
    let (add, sub, _rgba) = grain_tiles(fx().grain.unwrap_or(0));
    let pair = (tile_fill(&add, w, h), tile_fill(&sub, w, h));
    with(|s| {
        if s.grain_cache.len() >= 2 {
            s.grain_cache.clear();
        }
        s.grain_cache.insert((w, h), pair.clone());
    });
    pair
}

// ----- Glas-Zwischenspeicher (LRU mit Budget) -----------------------------

fn glass_get(key: &GlassKey) -> Option<Surface> {
    with(|s| {
        s.glass_stamp += 1;
        let stamp = s.glass_stamp;
        s.glass.get_mut(key).map(|e| {
            e.1 = stamp;
            e.0.clone()
        })
    })
}

fn glass_allow() -> bool {
    let now = pg::ticks() as i64;
    with(|s| {
        if now - s.glass_ms > 12 {
            s.glass_ms = now;
            s.glass_left = GLASS_MISS_BUDGET;
        }
        if s.glass_left <= 0 {
            return false;
        }
        s.glass_left -= 1;
        true
    })
}

fn glass_put(key: GlassKey, surf: Surface) -> Surface {
    with(|s| {
        s.glass_stamp += 1;
        let stamp = s.glass_stamp;
        s.glass_px += (surf.w() * surf.h()) as i64;
        if let Some(alt) = s.glass.insert(key, (surf.clone(), stamp)) {
            // Python ersetzt den Eintrag; die Pixelsumme zaehlt dort doppelt
            // (der alte Wert wird nicht abgezogen) - hier genauso.
            let _ = alt;
        }
        while !s.glass.is_empty()
            && (s.glass.len() > GLASS_MAX_ITEMS || (s.glass_px > GLASS_MAX_PX && s.glass.len() > 1))
        {
            let aeltester = s
                .glass
                .iter()
                .min_by_key(|(_, (_, st))| *st)
                .map(|(k, _)| k.clone());
            match aeltester {
                Some(k) => {
                    if let Some((old, _)) = s.glass.remove(&k) {
                        s.glass_px = 0.max(s.glass_px - (old.w() * old.h()) as i64);
                    }
                }
                None => break,
            }
        }
    });
    surf
}

fn fit_alpha(surf: Surface) -> Surface {
    surf.convert_alpha()
}

fn alpha_ramp(width: impl Num, color: Color, peak: f64, ends: f64) -> Surface {
    let mut base = pg::surface_alpha(16, 1);
    for i in 0..16 {
        let f = (py::PI * (i as f64 + 0.5) / 16.0).sin();
        base.fill_r(pg::alpha(color, (ends + (peak - ends) * f) as i32), (i, 0, 1, 1));
    }
    transform::smoothscale(&base, 2.max(width.i()), 1)
}

fn glass_overlay(w: i32, h: i32, radius: i32) -> Option<Surface> {
    let key = GlassKey::Ov(w, h, radius);
    if let Some(s) = glass_get(&key) {
        return Some(s);
    }
    if !glass_allow() {
        return None;
    }
    let mut surf = pg::surface_alpha(w, h);
    let grain = fx().grain.unwrap_or(0);
    if grain != 0 {
        let tile = grain_tiles(grain).2;
        let (tw, th) = (tile.w(), tile.h());
        let mut y = 0;
        while y < h {
            let mut x = 0;
            while x < w {
                surf.bl_flags(&tile, (x, y), BLEND_RGBA_MAX);
                x += tw;
            }
            y += th;
        }
    }
    let mut sheen = pg::surface_alpha(8, 8);
    for gy in 0..8 {
        for gx in 0..8 {
            let d = (gx + gy) as f64 / 14.0;
            sheen.fill_r(
                Color::rgba(255, 255, 255, (16.0 * (1.0 - d / 0.6).max(0.0)) as u8),
                (gx, gy, 1, 1),
            );
        }
    }
    surf.bl_flags(&transform::smoothscale(&sheen, w, h), (0, 0), BLEND_RGBA_MAX);
    surf.bl_flags(&alpha_ramp(w, Color::rgb(255, 255, 255), 42.0, 0.0), (0, 0), BLEND_RGBA_MAX);
    surf.fill_r(Color::rgba(0, 0, 0, 0), (0, h - 1, w, 1));
    surf.bl_flags(&alpha_ramp(w, Color::rgb(0, 0, 0), 40.0, 0.0), (0, h - 1), BLEND_RGBA_MAX);
    let mut mask = pg::surface_alpha(w, h);
    draw::rect_r(&mut mask, Color::rgba(255, 255, 255, 255), (0, 0, w, h), 0, radius);
    surf.bl_flags(&mask, (0, 0), BLEND_RGBA_MIN);
    Some(glass_put(key, fit_alpha(surf)))
}

fn glass_panel(w: i32, h: i32, radius: i32, color: Color, border: Color) -> Option<Surface> {
    let key = GlassKey::Panel(w, h, radius, color, border);
    if let Some(s) = glass_get(&key) {
        return Some(s);
    }
    if !glass_allow() {
        return None;
    }
    let mut surf = pg::surface_alpha(w, h);
    let a = fx().glass_alpha.unwrap_or(222) as i32;
    draw::rect_r(&mut surf, pg::alpha(color, a), (0, 0, w, h), 0, radius);
    draw::rect_r(&mut surf, pg::alpha(border, 255.min(a + 24)), (0, 0, w, h), 1, radius);
    if let Some(ov) = glass_overlay(w, h, radius) {
        surf.bl(&ov, (0, 0));
    }
    Some(glass_put(key, fit_alpha(surf)))
}

fn glass_shadow(w: i32, h: i32, radius: i32, a: u8) -> Option<Surface> {
    let key = GlassKey::Shadow(w, h, radius, a);
    if let Some(s) = glass_get(&key) {
        return Some(s);
    }
    if !glass_allow() {
        return None;
    }
    let (sw, sh) = (6.max((w + 24) / 4), 6.max((h + 24) / 4));
    let mut small = pg::surface_alpha(sw, sh);
    draw::rect_r(&mut small, Color::rgba(0, 0, 0, a), (1, 1, sw - 2, sh - 2), 0, 2.max(radius / 3));
    let surf = transform::smoothscale(&small, w + 24, h + 24);
    Some(glass_put(key, fit_alpha(surf)))
}

fn glass_glow(w: i32, h: i32, radius: i32, color: Color) -> Option<Surface> {
    let key = GlassKey::BtnGlow(w, h, radius, color);
    if let Some(s) = glass_get(&key) {
        return Some(s);
    }
    if !glass_allow() {
        return None;
    }
    let (sw, sh) = (6.max((w + 24) / 4), 6.max((h + 20) / 4));
    let mut small = pg::surface_alpha(sw, sh);
    draw::rect_r(&mut small, pg::alpha(color, 78), (1, 1, sw - 2, sh - 2), 0, 2.max((radius + 6) / 3));
    let surf = transform::smoothscale(&small, w + 24, h + 20);
    Some(glass_put(key, fit_alpha(surf)))
}

fn line_stops(accent: Option<Color>) -> Option<[Color; 3]> {
    let g3 = fx().grad3?;
    match accent {
        None => Some([c3(g3[0]), c3(g3[1]), c3(g3[2])]),
        Some(ac) => Some([
            mix(ac, Color::rgb(255, 255, 255), 0.25),
            ac,
            mix(ac, Color::rgb(236, 96, 196), 0.45),
        ]),
    }
}

fn grad_strip(w: i32, h: i32, stops: [Color; 3]) -> Option<Surface> {
    let key = GlassKey::Strip(w, h, stops);
    if let Some(s) = glass_get(&key) {
        return Some(s);
    }
    if !glass_allow() {
        return None;
    }
    let mut surf = pg::surface(w, h);
    let seg = (stops.len() - 1) as f64;
    for x in 0..w {
        let f = x as f64 / (1.max(w - 1)) as f64 * seg;
        let k = ((seg - 1.0) as i32).min(f as i32) as usize;
        draw::line(&mut surf, mix(stops[k], stops[k + 1], f - k as f64), (x, 0), (x, h), 1);
    }
    Some(glass_put(key, surf))
}

fn grad_glow(w: i32, h: i32, stops: [Color; 3]) -> Option<Surface> {
    let key = GlassKey::StripGlow(w, h, stops);
    if let Some(s) = glass_get(&key) {
        return Some(s);
    }
    if !glass_allow() {
        return None;
    }
    let sw = 4.max(w / 6);
    let mut small = pg::surface(sw, 3);
    let seg = (stops.len() - 1) as f64;
    for x in 0..sw {
        let f = x as f64 / (1.max(sw - 1)) as f64 * seg;
        let k = ((seg - 1.0) as i32).min(f as i32) as usize;
        let col = mix(stops[k], stops[k + 1], f - k as f64);
        small.fill_r(pg::scaled(col, 0.5), (x, 1, 1, 1));
    }
    let surf = transform::smoothscale(&small, w, 4.max(h * 3));
    Some(glass_put(key, surf))
}

/// `ui.draw_grad_line(surface, cx, y, w, h=3, accent=None, glow=False)`
pub fn draw_grad_line(
    surface: &mut Surface,
    cx: impl Num,
    y: impl Num,
    w: impl Num,
    h: i32,
    accent: Option<Color>,
    glow: bool,
) {
    let (w, h) = (2.max(w.i()), 1.max(h));
    let (x, y) = (cx.i() - w / 2, y.i());
    let stops = line_stops(accent);
    let strip = stops.and_then(|st| grad_strip(w, h, st));
    let Some(strip) = strip else {
        draw::rect_r(surface, accent.unwrap_or_else(self::accent), (x, y, w, h), 0, 2);
        return;
    };
    if glow {
        if let Some(g) = grad_glow(w, h, stops.unwrap_or([Color::rgb(0, 0, 0); 3])) {
            surface.bl_flags(&g, (x, y - h), BLEND_ADD);
        }
    }
    surface.bl(&strip, (x, y));
}

/// Ueberabtastung beim Stern-Zeichnen (`ui._STAR_SS`).
const STAR_SS: i32 = 4;

/// Cache-Schluessel der Sterne: (Radius, Farbe, Linienbreite).
type StarKey = (i32, (u8, u8, u8), i32);

/// `ui._star_points(cx, cy, r)` - zehn Eckpunkte, auf ganze Pixel gerundet
/// (`floor(x + 0.5)` wie in Python).
fn star_points(cx: f64, cy: f64, r: f64) -> Vec<(i32, i32)> {
    (0..10)
        .map(|i| {
            let rr = if i % 2 == 0 { r } else { r * 0.46 };
            let a = py::radians((-90 + i * 36) as f64);
            ((cx + a.cos() * rr + 0.5).floor() as i32, (cy + a.sin() * rr + 0.5).floor() as i32)
        })
        .collect()
}

/// `ui.star_surface(r, color, width=0)` - fuenfzackiger Stern als geglaettete,
/// transparente Flaeche (Kantenlaenge 2r+2), gecacht.
pub fn star_surface(r: i32, color: Color, width: i32) -> Surface {
    thread_local! {
        static CACHE: RefCell<HashMap<StarKey, Surface>> = RefCell::new(HashMap::new());
    }
    let r = r.max(2);
    let key = (r, (color.r, color.g, color.b), width);
    if let Some(s) = CACHE.with(|c| c.borrow().get(&key).cloned()) {
        return s;
    }
    let size = 2 * r + 2;
    let mut big = pg::surface_alpha(size * STAR_SS, size * STAR_SS);
    // Transparente Pixel in der Sternfarbe -> keine dunklen Kanten.
    big.fill_c(pg::rgba(color.r, color.g, color.b, 0));
    let c = (size * STAR_SS) as f64 / 2.0;
    draw::polygon(&mut big, rgb(color.r, color.g, color.b), &star_points(c, c, (r * STAR_SS) as f64), width * STAR_SS);
    let surf = transform::smoothscale(&big, size, size);
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() > 64 {
            c.clear();
        }
        c.insert(key, surf.clone());
    });
    surf
}

/// `ui.draw_star(surface, center, r, color, width=0)`
pub fn draw_star(surface: &mut Surface, center: impl Pt, r: i32, color: Color, width: i32) {
    let img = star_surface(r, color, width);
    let rr = img.get_rect().center_at(center);
    surface.bl(&img, (rr.x, rr.y));
}

/// `ui.draw_halo(surface, center, size, color=None)`
pub fn draw_halo(surface: &mut Surface, center: impl Pt, size: impl Num, color: Option<Color>) {
    if !fx().logo_halo {
        return;
    }
    let col = color.unwrap_or_else(|| pg::scaled(accent(), 0.5));
    let g = glow_surface(col, size.i());
    let r = g.get_rect().center_at(center);
    surface.bl_flags(&g, (r.x, r.y), BLEND_ADD);
}

fn draw_aurora(surface: &mut Surface, w: i32, h: i32, tsec: f64, table: &[MeshBlob], amp: (f64, f64)) {
    let (ax, ay) = amp;
    for b in table {
        let size = (b.size * w.max(h) as f64) as i32;
        let cx = ((b.x + ax * (tsec * b.speed + b.phase).sin()) * w as f64) as i32;
        let cy = ((b.y + ay * (tsec * b.speed * 0.9 + b.phase).cos()) * h as f64) as i32;
        let g = glow_surface(c3(b.color), size);
        let r = g.get_rect().center_at((cx, cy));
        surface.bl_flags(&g, (r.x, r.y), BLEND_ADD);
    }
}

fn ensure_stars() {
    with(|s| {
        if s.stars.is_empty() {
            let mut rnd = Random::new(20240);
            for _ in 0..STAR_COUNT {
                let x = rnd.random();
                let y = rnd.random();
                let d = rnd.uniform(0.25, 1.0);
                let r = rnd.choice(&[1, 1, 1, 2]);
                s.stars.push((x, y, d, r));
            }
        }
    });
}

fn draw_shooting_star(surface: &mut Surface, w: i32, h: i32) {
    let now = pg::ticks();
    let st = with(|s| s.shoot);
    let Some(st) = st else {
        if now >= with(|s| s.shoot_next_ms) {
            let ang = py::radians(py::uniform(18.0, 38.0));
            let speed = py::uniform(0.8, 1.3) * w.max(400) as f64;
            let x = py::uniform(0.15, 0.85) * w as f64;
            let y = py::uniform(0.05, 0.30) * h as f64;
            let vx = ang.cos() * speed * py::choice(&[1.0, -1.0]);
            let vy = ang.sin() * speed;
            let dur = py::randint(550, 800);
            with(|s| s.shoot = Some(Shoot { x, y, vx, vy, t0: now, dur }));
        }
        return;
    };
    let t = (now as f64 - st.t0 as f64) / st.dur as f64;
    if t >= 1.0 {
        let n = py::randint(4500, 10000);
        with(|s| {
            s.shoot = None;
            s.shoot_next_ms = now + n as u64;
        });
        return;
    }
    let sec = t * st.dur as f64 / 1000.0;
    let (x, y) = (st.x + st.vx * sec, st.y + st.vy * sec);
    let fade = 1.0 - t;
    for i in 0..9 {
        let f = fade * (1.0 - i as f64 / 9.0);
        let (px, py_) = (x - st.vx * 0.0024 * i as f64, y - st.vy * 0.0024 * i as f64);
        if 0.0 <= px && px < (w - 2) as f64 && 0.0 <= py_ && py_ < (h - 2) as f64 {
            let c = (190.0 * f) as i32;
            surface.fill_rf(
                rgb(c, c, 255.min((c as f64 * 1.2) as i32)),
                (px as i32, py_ as i32, 2, 2),
                BLEND_ADD,
            );
        }
    }
}

/// `ui.draw_background(surface, w, h)`
pub fn draw_background(surface: &mut Surface, w: i32, h: i32) {
    draw_background_ex(surface, w, h, true, None);
}

/// `ui.draw_background(surface, w, h, stars=..., aurora=...)`
pub fn draw_background_ex(surface: &mut Surface, w: i32, h: i32, stars: bool, aurora: Option<bool>) {
    tick();
    surface.bl(&base_background(w, h), (0, 0));
    let fxx = fx();
    let mesh_on = !fxx.mesh.is_empty() && aurora != Some(false);
    let aurora = aurora.unwrap_or(stars);
    let ticks = pg::ticks() as f64 / 1000.0;
    if mesh_on {
        draw_aurora(surface, w, h, ticks, fxx.mesh, fxx.mesh_drift.unwrap_or((0.07, 0.06)));
    }
    if aurora && fxx.aurora {
        draw_aurora(surface, w, h, ticks, AURORA, (0.07, 0.06));
    }
    if stars && fxx.stars {
        ensure_stars();
        let bright = fxx.star_bright;
        let liste = with(|s| s.stars.clone());
        let n = match fxx.star_count {
            Some(c) if c > 0 => (c as usize).min(liste.len()),
            _ => liste.len(),
        };
        for &(x, y, depth, r) in &liste[..n] {
            let yy = py::fpmod(y - ticks * 0.008 * depth, 1.0);
            let tw = 0.5 + 0.5 * (ticks * (0.8 + depth) + x * 40.0).sin();
            let c = ((40.0 + 70.0 * depth * tw) * bright) as i32;
            surface.fill_r(
                rgb(c, c + 6, c + 18),
                ((x * w as f64) as i32, (yy * h as f64) as i32, r, r),
            );
        }
        if fxx.shooting {
            draw_shooting_star(surface, w, h);
        }
    }
    if fxx.grain.unwrap_or(0) != 0 {
        let (g_add, g_sub) = grain_surfaces(w, h);
        surface.bl_flags(&g_add, (0, 0), BLEND_RGB_ADD);
        surface.bl_flags(&g_sub, (0, 0), BLEND_RGB_SUB);
    }
}

// ---------------------------------------------------------------------------
//  Himmelskoerper (UI v4.1)
// ---------------------------------------------------------------------------

fn render_black_hole(radius: i32) -> Surface {
    let key = (0u8, radius);
    if let Some(s) = with(|s| s.celestial.get(&key).cloned()) {
        return s;
    }
    const S: i32 = 512;
    let c = S / 2;
    let sh: i32 = 66;
    let rh: i32 = 84;
    let rd = 246;
    let mut surf = pg::surface_alpha(S, S);

    let mut haze = pg::surface_alpha(S, S);
    let mut r = rh + 120;
    while r > 4 {
        let d = (r - rh).abs() as f64 / if r > rh { 100.0 } else { 34.0 };
        let i = py::pow((1.0f64 - d).max(0.0), 2.0);
        if i > 0.004 {
            let col = (
                (255.0 * (i * 1.25).min(1.0)) as i32,
                (165.0 * i) as i32,
                (66.0 * i) as i32,
            );
            draw::circle(&mut haze, (col.0, col.1, col.2, (235.0 * i) as i32), (c, c), r, 0);
        }
        r -= 1;
    }
    let mut arc = pg::surface_alpha(S, S);
    for (cy_off, strength) in [(-rh, 1.0), (rh, 0.6)] {
        let mut r = 96;
        while r > 3 {
            let f = py::pow(1.0 - r as f64 / 96.0, 2.0) * strength;
            draw::circle(
                &mut arc,
                rgb((255.0 * f) as i32, (185.0 * f) as i32, (90.0 * f) as i32),
                (c, c + cy_off),
                r,
                0,
            );
            r -= 1;
        }
    }
    haze.bl_flags(&arc, (0, 0), BLEND_RGB_ADD);
    let small = transform::smoothscale(&haze, S / 4, S / 4);
    let haze = transform::smoothscale(&small, S, S);
    surf.bl(&haze, (0, 0));

    draw::circle(&mut surf, Color::rgba(16, 8, 6, 255), (c, c), sh, 0);
    draw::circle(&mut surf, Color::rgba(70, 28, 14, 110), (c, c), sh - 2, 5);
    draw::circle(&mut surf, Color::rgba(255, 200, 130, 80), (c, c), sh + 7, 8);
    draw::circle(&mut surf, Color::rgba(255, 228, 180, 170), (c, c), sh + 5, 5);
    draw::circle(&mut surf, Color::rgba(255, 244, 214, 235), (c, c), sh + 3, 4);

    let th = 28;
    let mut disk = pg::surface_alpha(S, S);
    for t in (1..=th).rev() {
        let i = py::pow(1.0 - t as f64 / th as f64, 0.8);
        let col = mix(Color::rgb(255, 186, 92), Color::rgb(255, 252, 234), i);
        draw::ellipse(&mut disk, pg::alpha(col, (238.0 + 17.0 * i) as i32), (c - rd, c - t, 2 * rd, 2 * t), 0);
    }
    for (fy, col) in [
        (-0.60, Color::rgb(243, 198, 128)),
        (-0.28, Color::rgb(248, 214, 154)),
        (0.26, Color::rgb(246, 206, 140)),
        (0.56, Color::rgb(240, 190, 118)),
    ] {
        let yy = c + (th as f64 * fy) as i32;
        let span = (rd as f64 * (1.0 - fy * fy).max(0.0).sqrt() * 0.94) as i32;
        draw::line(&mut disk, pg::alpha(col, 255), (c - span, yy), (c + span, yy), 2);
    }
    let mut tint = pg::surface_alpha(S, S);
    for x in 0..S {
        let dx = ((x - c).abs() as f64 / rd as f64).min(1.0);
        let g = (255.0 * (1.0 - 0.18 * dx)) as i32;
        let b = (255.0 * (1.0 - 0.36 * dx)) as i32;
        let a = (255.0 * (1.0 - py::pow(dx, 4.0)).max(0.0)) as i32;
        draw::line(&mut tint, (255, g, b, a), (x, 0), (x, S), 1);
    }
    disk.bl_flags(&tint, (0, 0), BLEND_RGBA_MULT);
    let small = transform::smoothscale(&disk, S / 2, S / 2);
    let disk = transform::smoothscale(&small, S, S);
    for div in [6, 10] {
        let band = transform::smoothscale(&disk, S / div, S / div);
        let band = transform::smoothscale(&band, S, S);
        surf.bl_flags(&band, (0, 0), BLEND_RGB_ADD);
    }
    surf.bl(&disk, (0, 0));

    let mut hot = pg::surface_alpha(S, S);
    let mut r = 104;
    while r > 3 {
        let f = py::pow(1.0 - r as f64 / 104.0, 2.0);
        draw::circle(
            &mut hot,
            rgb((255.0 * f) as i32, (235.0 * f) as i32, (180.0 * f) as i32),
            (c + sh + 18, c - 4),
            r,
            0,
        );
        r -= 1;
    }
    surf.bl_flags(&hot, (0, 0), BLEND_RGB_ADD);

    let surf = transform::rotate(&surf, 26.0);
    let img = transform::smoothscale(&surf, radius * 2, radius * 2);
    with(|s| {
        if s.celestial.len() > 8 {
            s.celestial.clear();
        }
        s.celestial.insert(key, img.clone());
    });
    img
}

fn render_saturn(radius: i32) -> Surface {
    let key = (1u8, radius);
    if let Some(s) = with(|s| s.celestial.get(&key).cloned()) {
        return s;
    }
    const S: i32 = 320;
    let c = S / 2;
    let pr = 68;
    let mut rings = pg::surface_alpha(S, S);
    let mut ell = |rx: i32, ry: i32, col: Color| {
        draw::ellipse(&mut rings, col, (c - rx, c - ry, rx * 2, ry * 2), 0);
    };
    ell(124, 38, Color::rgba(205, 183, 148, 175));
    ell(112, 34, Color::rgba(0, 0, 0, 0));
    ell(106, 32, Color::rgba(230, 208, 172, 210));
    ell(84, 24, Color::rgba(176, 155, 124, 120));
    ell(74, 22, Color::rgba(0, 0, 0, 0));

    let mut planet = pg::surface_alpha(S, S);
    let tones = [
        Color::rgb(232, 210, 170),
        Color::rgb(212, 186, 142),
        Color::rgb(226, 202, 162),
        Color::rgb(206, 180, 138),
        Color::rgb(228, 204, 164),
        Color::rgb(216, 190, 148),
    ];
    let band_h = (pr * 2) as f64 / tones.len() as f64;
    for (i, tone) in tones.iter().enumerate() {
        draw::rect(
            &mut planet,
            *tone,
            (c - pr, (c as f64 - pr as f64 + i as f64 * band_h) as i32, pr * 2, band_h as i32 + 1),
            0,
        );
    }
    let mut mask = pg::surface_alpha(S, S);
    draw::circle(&mut mask, Color::rgba(255, 255, 255, 255), (c, c), pr, 0);
    planet.bl_flags(&mask, (0, 0), BLEND_RGBA_MIN);
    let mut shade = pg::surface_alpha(S, S);
    let mut r = pr + 24;
    while r > 0 {
        let a = 160.min((150.0 * py::pow(r as f64 / pr as f64, 3.0)) as i32);
        draw::circle(&mut shade, (18, 14, 24, a), (c + 14, c + 12), r, 0);
        r -= 1;
    }
    shade.bl_flags(&mask, (0, 0), BLEND_RGBA_MIN);
    planet.bl(&shade, (0, 0));

    let mut surf = pg::surface_alpha(S, S);
    surf.bl_area(&rings, (0, 0), (0, 0, S, c));
    surf.bl(&planet, (0, 0));
    surf.bl_area(&rings, (0, c), (0, c, S, S - c));

    let surf = transform::rotate(&surf, 20.0);
    let scale = radius as f64 / pr as f64;
    let size = 2.max((surf.w() as f64 * scale) as i32);
    let img = transform::smoothscale(&surf, size, size);
    with(|s| {
        if s.celestial.len() > 8 {
            s.celestial.clear();
        }
        s.celestial.insert(key, img.clone());
    });
    img
}

/// `ui.draw_black_hole(surface, center, radius)`
pub fn draw_black_hole(surface: &mut Surface, center: impl Pt, radius: impl Num) {
    let radius = 8.max((radius.i() / 4) * 4);
    let img = render_black_hole(radius);
    let r = img.get_rect().center_at(center);
    surface.blr(&img, r);
}

/// `ui.draw_saturn(surface, center, radius)`
pub fn draw_saturn(surface: &mut Surface, center: impl Pt, radius: impl Num) {
    let radius = 6.max((radius.i() / 4) * 4);
    let img = render_saturn(radius);
    let r = img.get_rect().center_at(center);
    surface.blr(&img, r);
}

// ---------------------------------------------------------------------------
//  Panels und Knoepfe
// ---------------------------------------------------------------------------

/// `ui.draw_panel(surface, rect)`
pub fn draw_panel(surface: &mut Surface, rect: impl IntoRect) -> Rect {
    draw_panel_ex(surface, rect, None, None, None, true, None)
}

/// `ui.draw_panel(surface, rect, color, border, radius, shadow, accent_top)`
pub fn draw_panel_ex(
    surface: &mut Surface,
    rect: impl IntoRect,
    color: Option<Color>,
    border: Option<Color>,
    radius: Option<i32>,
    shadow: bool,
    accent_top: Option<Color>,
) -> Rect {
    let p = pal();
    let fxx = fx();
    let color = color.unwrap_or(p.panel);
    let border = border.unwrap_or(p.border);
    let radius = radius.unwrap_or(fxx.panel_radius);
    let r = rect.rect();
    if fxx.style == "v42" {
        if shadow && fxx.shadow_alpha != 0 {
            if let Some(sh) = glass_shadow(r.w, r.h, radius, fxx.shadow_alpha) {
                surface.bl(&sh, (r.x - 10, r.y - 7));
            }
        }
        if let Some(glass) = glass_panel(r.w, r.h, radius, color, border) {
            surface.bl(&glass, (r.x, r.y));
        } else {
            draw::rect_r(surface, color, r, 0, radius);
            draw::rect_r(surface, border, r, 1, radius);
        }
        if let Some(at) = accent_top {
            draw::rect(surface, at, (r.x + radius, r.y, r.w - 2 * radius, 2), 0);
        }
        return r;
    }
    if fxx.style == "v1" {
        draw::rect_r(surface, color, r, 0, radius);
        if let Some(at) = accent_top {
            draw::rect(surface, at, (r.x + radius, r.y, r.w - 2 * radius, 2), 0);
        }
        return r;
    }
    if shadow {
        let mut sh = pg::surface_alpha(r.w + 12, r.h + 12);
        draw::rect_r(&mut sh, Color::rgba(0, 0, 0, fxx.shadow_alpha), (4, 6, r.w, r.h), 0, radius + 2);
        surface.bl(&sh, (r.x - 2, r.y - 2));
    }
    draw::rect_r(surface, color, r, 0, radius);
    draw::rect_r(surface, border, r, 1, radius);
    if let Some(at) = accent_top {
        draw::rect(surface, at, (r.x + radius, r.y, r.w - 2 * radius, 2), 0);
    }
    r
}

fn btn_progress(key: (i32, i32, i32, i32), selected: bool) -> f64 {
    let dt = frame_dt();
    with(|s| {
        let mut v = *s.btn_anim.get(&key).unwrap_or(&0.0);
        let target = if selected { 1.0 } else { 0.0 };
        v += (target - v) * (dt * 12.0).min(1.0);
        if (v - target).abs() < 0.01 {
            v = target;
        }
        if s.btn_anim.len() > 96 {
            s.btn_anim.clear();
        }
        s.btn_anim.insert(key, v);
        v
    })
}

#[allow(clippy::too_many_arguments)]
fn blit_button_label(
    surface: &mut Surface,
    r: Rect,
    label: &str,
    fnt: &Font,
    text_col: Color,
    sub: Option<&str>,
    sub_font: Option<&Font>,
    sub_col: Color,
) {
    let img = fnt.render(label, true, text_col);
    if let (Some(sub), Some(sf)) = (sub.filter(|s| !s.is_empty()), sub_font) {
        let sub_img = sf.render(sub, true, sub_col);
        let total_h = img.h() + 2 + sub_img.h();
        let y0 = r.centery() - py::fdiv32(total_h, 2);
        surface.blr(&img, img.get_rect().midtop_at((r.centerx(), y0)));
        surface.blr(&sub_img, sub_img.get_rect().midtop_at((r.centerx(), y0 + img.h() + 2)));
    } else {
        surface.blr(&img, img.get_rect().center_at(r.center()));
    }
}

/// `ui.draw_button(surface, rect, label, fnt, selected)`
pub fn draw_button(surface: &mut Surface, rect: impl IntoRect, label: &str, fnt: &Font, selected: bool) -> Rect {
    draw_button_ex(surface, rect, label, fnt, selected, None, None, None)
}

/// `ui.draw_button(surface, rect, label, fnt, selected, sub=, sub_font=, accent=)`
#[allow(clippy::too_many_arguments)]
pub fn draw_button_ex(
    surface: &mut Surface,
    rect: impl IntoRect,
    label: &str,
    fnt: &Font,
    selected: bool,
    sub: Option<&str>,
    sub_font: Option<&Font>,
    accent: Option<Color>,
) -> Rect {
    let p = pal();
    let fxx = fx();
    let ac = accent.unwrap_or(p.accent);
    let r = rect.rect();
    let v = btn_progress((r.x, r.y, r.w, r.h), selected);
    let radius = fxx.btn_radius;

    if fxx.style == "v42" {
        if v > 0.02 {
            if let Some(mut glow) = glass_glow(r.w, r.h, radius, ac) {
                glow.alpha_set((255.0 * v) as i32);
                surface.bl(&glow, (r.x - 12, r.y - 10));
            }
        }
        let fill = mix(p.btn, mix(p.btn_sel, ac, 0.16), v);
        draw::rect_r(surface, fill, r, 0, radius);
        draw::rect_r(
            surface,
            mix(p.border, mix(ac, Color::rgb(255, 255, 255), 0.2), 0.8 * v),
            r,
            1,
            radius,
        );
        if let Some(ov) = glass_overlay(r.w, r.h, radius) {
            surface.bl(&ov, (r.x, r.y));
        }
        let stops = line_stops(accent);
        if v > 0.05 {
            if let Some(stops) = stops {
                let full = 8.max(r.w - 16);
                if let Some(strip) = grad_strip(full, 2, stops) {
                    let lw = 2.max((full as f64 * v) as i32);
                    surface.bl_area(
                        &strip,
                        (r.centerx() - lw / 2, r.bottom() - 6),
                        ((full - lw) / 2, 0, lw, 2),
                    );
                }
            }
        }
        blit_button_label(
            surface,
            r,
            label,
            fnt,
            mix(p.text_dim, p.text, v),
            sub,
            sub_font,
            mix(p.text_faint, p.text_dim, v),
        );
        return r;
    }

    if is_modern() {
        let fill = mix(p.btn, mix(p.panel_light, ac, 0.10), v);
        draw::rect_r(surface, fill, r, 0, radius);
        draw::rect_r(surface, mix(p.border, ac, 0.85 * v), r, 1, radius);
        if v > 0.05 {
            let bh = 6.max(((r.h - 14) as f64 * v) as i32);
            draw::rect_r(surface, ac, (r.x + 7, r.centery() - bh / 2, 3, bh), 0, 2);
        }
        blit_button_label(
            surface,
            r,
            label,
            fnt,
            mix(p.text_dim, p.text, v),
            sub,
            sub_font,
            mix(p.text_faint, p.text_dim, v),
        );
        return r;
    }

    if fxx.style == "v1" {
        draw::rect_r(surface, if selected { p.btn_sel } else { p.btn }, r, 0, radius);
        blit_button_label(surface, r, label, fnt, p.text, sub, sub_font, p.text_dim);
        return r;
    }

    if fxx.style == "v2" {
        if selected {
            let mut glow = pg::surface_alpha(r.w + 20, r.h + 20);
            draw::rect_r(&mut glow, pg::alpha(ac, 45), (0, 0, r.w + 20, r.h + 20), 0, 16);
            surface.bl(&glow, (r.x - 10, r.y - 10));
            draw::rect_r(surface, p.btn_sel, r, 0, radius);
            draw::rect_r(surface, ac, r, 2, radius);
            draw::rect_r(surface, ac, (r.x + 6, r.y + 8, 4, r.h - 16), 0, 2);
            let (cx, cy) = (r.right() - 18, r.centery());
            draw::polygon(surface, p.text, &[(cx - 4, cy - 6), (cx + 4, cy), (cx - 4, cy + 6)], 0);
        } else {
            draw::rect_r(surface, p.btn, r, 0, radius);
            draw::rect_r(surface, p.border, r, 1, radius);
        }
        blit_button_label(
            surface,
            r,
            label,
            fnt,
            if selected { p.text } else { p.text_dim },
            sub,
            sub_font,
            if selected { p.text_dim } else { p.text_faint },
        );
        return r;
    }

    if v > 0.02 && fxx.btn_glow {
        let glow_a = (52.0 * v * (0.7 + 0.3 * pulse(2.2, 0.35, 1.0))) as i32;
        let mut glow = pg::surface_alpha(r.w + 22, r.h + 22);
        draw::rect_r(&mut glow, pg::alpha(ac, glow_a), (0, 0, r.w + 22, r.h + 22), 0, 16);
        surface.bl(&glow, (r.x - 11, r.y - 11));
    }
    let fill = mix(p.btn, mix(p.btn_sel, ac, 0.15), v);
    draw::rect_r(surface, fill, r, 0, radius);
    draw::rect(
        surface,
        mix(fill, Color::rgb(255, 255, 255), 0.05 + 0.10 * v),
        (r.x + 10, r.y + 1, r.w - 20, 1),
        0,
    );
    draw::rect_r(surface, mix(p.border, ac, v), r, if v > 0.5 { 2 } else { 1 }, radius);
    if v > 0.05 {
        let bh = 4.max(((r.h - 16) as f64 * v) as i32);
        draw::rect_r(surface, ac, (r.x + 6, r.centery() - bh / 2, 4, bh), 0, 2);
        if fxx.btn_arrow {
            let cx = r.right() - 18 + ((1.0 - v) * 10.0) as i32 + (2.0 * pulse(2.6, 0.0, 1.0)) as i32;
            let cy = r.centery();
            draw::polygon(
                surface,
                mix(fill, p.text, v),
                &[(cx - 4, cy - 6), (cx + 4, cy), (cx - 4, cy + 6)],
                0,
            );
        }
    }
    blit_button_label(
        surface,
        r,
        label,
        fnt,
        mix(p.text_dim, p.text, v),
        sub,
        sub_font,
        mix(p.text_faint, p.text_dim, v),
    );
    r
}

// ---------------------------------------------------------------------------
//  Text-Effekte
// ---------------------------------------------------------------------------

/// `ui.grad_text(fnt, text, top, bottom)` - Text mit senkrechtem Verlauf.
pub fn grad_text(fnt: &Font, text: &str, top: Option<Color>, bottom: Option<Color>) -> Surface {
    let top = top.unwrap_or(Color::rgb(252, 253, 255));
    let bottom = bottom.unwrap_or(Color::rgb(165, 190, 235));
    let key = TextFxKey::Grad(fnt.id(), text.to_string(), top, bottom);
    if let Some(s) = with(|s| s.text_fx.get(&key).cloned()) {
        return s;
    }
    let mut img = fnt.render(text, true, Color::rgb(255, 255, 255)).convert_alpha();
    let (gw, gh) = (img.w(), img.h());
    let mut grad = pg::surface_alpha(gw, gh);
    for yy in 0..gh {
        let f = yy as f64 / 1.max(gh - 1) as f64;
        draw::line(&mut grad, pg::alpha(mix(top, bottom, f), 255), (0, yy), (gw, yy), 1);
    }
    img.bl_flags(&grad, (0, 0), BLEND_RGBA_MULT);
    with(|s| {
        if s.text_fx.len() > 48 {
            s.text_fx.clear();
        }
        s.text_fx.insert(key, img.clone());
    });
    img
}

fn text_glow(fnt: &Font, text: &str, color: Color) -> Surface {
    let key = TextFxKey::Glow(fnt.id(), text.to_string(), color);
    if let Some(s) = with(|s| s.text_fx.get(&key).cloned()) {
        return s;
    }
    let raw = fnt.render(text, true, color);
    let (w, h) = (raw.w(), raw.h());
    let mut pad = pg::surface_alpha(w + 40, h + 40);
    pad.bl(&raw, (20, 20));
    let small = transform::smoothscale(&pad, 1.max((w + 40) / 6), 1.max((h + 40) / 6));
    let img = transform::smoothscale(&small, w + 40, h + 40);
    with(|s| {
        if s.text_fx.len() > 48 {
            s.text_fx.clear();
        }
        s.text_fx.insert(key, img.clone());
    });
    img
}

/// `ui.draw_title(surface, width, title, subtitle)`
pub fn draw_title(surface: &mut Surface, width: i32, title: &str, subtitle: Option<&str>) -> i32 {
    draw_title_ex(surface, width, title, subtitle, 52, None, None, None)
}

/// `ui.draw_title(surface, width, title, subtitle, y, big, small, accent)`
#[allow(clippy::too_many_arguments)]
pub fn draw_title_ex(
    surface: &mut Surface,
    width: i32,
    title: &str,
    subtitle: Option<&str>,
    y: i32,
    big: Option<&Font>,
    small: Option<&Font>,
    accent: Option<Color>,
) -> i32 {
    let p = pal();
    let fxx = fx();
    let big = big.cloned().unwrap_or_else(|| fb(40));
    let ac = accent.unwrap_or(p.accent);
    let cx = width / 2;
    let small_f = || small.cloned().unwrap_or_else(|| f(17));
    let subtitle = subtitle.filter(|s| !s.is_empty());

    if fxx.style == "v42" {
        let img = grad_text(&big, title, Some(Color::rgb(250, 251, 255)), Some(Color::rgb(200, 208, 244)));
        let mut sh = big.render(title, true, Color::rgb(0, 0, 0));
        sh.alpha_set(120);
        surface.blr(&sh, sh.get_rect().center_at((cx + 2, y + 2)));
        surface.blr(&img, img.get_rect().center_at((cx, y)));
        let ly = y + img.h() / 2 + 10;
        let lw = (64.0f64).max((0.6 * img.w() as f64).min(240.0)) as i32;
        draw_grad_line(surface, cx, ly, lw, 3, accent, true);
        if let Some(st) = subtitle {
            let sub = small_f().render(st, true, p.text_dim);
            surface.blr(&sub, sub.get_rect().center_at((cx, ly + 24)));
        }
        return ly;
    }

    if is_modern() {
        let img = big.render(title, true, p.text);
        surface.blr(&img, img.get_rect().center_at((cx, y)));
        let lw = 56.max((img.w() / 2).min(160));
        let ly = y + img.h() / 2 + 10;
        draw::rect_r(surface, ac, (cx - lw / 2, ly, lw, 3), 0, 2);
        if let Some(st) = subtitle {
            let sub = small_f().render(st, true, p.text_dim);
            surface.blr(&sub, sub.get_rect().center_at((cx, ly + 24)));
        }
        return ly;
    }

    if fxx.style == "v1" {
        let img = big.render(title, true, p.text);
        surface.blr(&img, img.get_rect().center_at((cx, y)));
        if let Some(st) = subtitle {
            let sub = small_f().render(st, true, p.text_dim);
            surface.blr(&sub, sub.get_rect().center_at((cx, y + 42)));
        }
        return y + img.h() / 2 + 8;
    }

    if fxx.style == "v2" {
        let mut sh = big.render(title, true, Color::rgb(0, 0, 0));
        sh.alpha_set(140);
        let img = big.render(title, true, p.text);
        surface.blr(&sh, sh.get_rect().center_at((cx + 2, y + 3)));
        surface.blr(&img, img.get_rect().center_at((cx, y)));
        let lw = (img.w() + 20).min(width - 80);
        let ly = y + img.h() / 2 + 8;
        draw::rect_r(surface, ac, (cx - py::fdiv32(lw, 2), ly, lw, 3), 0, 2);
        draw::rect_r(surface, p.accent_soft, (cx - py::fdiv32(lw, 6), ly + 5, py::fdiv32(lw, 3), 2), 0, 2);
        if let Some(st) = subtitle {
            let sub = small_f().render(st, true, p.text_dim);
            surface.blr(&sub, sub.get_rect().center_at((cx, ly + 26)));
        }
        return ly;
    }

    let glow = text_glow(&big, title, pg::scaled(ac, 0.55));
    let gr = glow.get_rect().center_at((cx, y));
    surface.bl_flags(&glow, (gr.x, gr.y), BLEND_ADD);
    let mut sh = big.render(title, true, Color::rgb(0, 0, 0));
    sh.alpha_set(140);
    let img = grad_text(&big, title, None, None);
    surface.blr(&sh, sh.get_rect().center_at((cx + 2, y + 3)));
    surface.blr(&img, img.get_rect().center_at((cx, y)));
    let lw = (img.w() + 20).min(width - 80);
    let ly = y + img.h() / 2 + 8;
    draw::rect_r(surface, ac, (cx - py::fdiv32(lw, 2), ly, lw, 3), 0, 2);
    draw::rect_r(
        surface,
        mix(ac, p.accent2, 0.6),
        (cx - py::fdiv32(lw, 6), ly + 5, py::fdiv32(lw, 3), 2),
        0,
        2,
    );
    if let Some(st) = subtitle {
        let sub = small_f().render(st, true, p.text_dim);
        surface.blr(&sub, sub.get_rect().center_at((cx, ly + 26)));
    }
    ly
}

/// `ui.draw_footer(surface, width, height, text)`
pub fn draw_footer(surface: &mut Surface, width: i32, height: i32, text: &str) {
    draw_footer_ex(surface, width, height, text, None);
}

/// `ui.draw_footer(surface, width, height, text, fnt)`
pub fn draw_footer_ex(surface: &mut Surface, width: i32, height: i32, text: &str, fnt: Option<&Font>) {
    let fnt = fnt.cloned().unwrap_or_else(|| f(14));
    let p = pal();
    let fxx = fx();
    if fxx.style == "v42" {
        let img = fnt.render(text, true, p.text_faint);
        let y = height - 22;
        let lw = 40.max(width * 2 / 3);
        let key = GlassKey::Hair(lw, p.border_light);
        let mut line = glass_get(&key);
        if line.is_none() && glass_allow() {
            line = Some(glass_put(key, fit_alpha(alpha_ramp(lw, p.border_light, 150.0, 0.0))));
        }
        if let Some(l) = line {
            surface.bl(&l, (width / 2 - lw / 2, y - 12));
        }
        surface.blr(&img, img.get_rect().center_at((width / 2, y)));
        return;
    }
    if fxx.style == "v1" {
        let img = fnt.render(text, true, p.text_dim);
        surface.blr(&img, img.get_rect().center_at((width / 2, height - 24)));
        return;
    }
    let img = fnt.render(text, true, p.text_faint);
    let y = height - 22;
    draw::line(surface, p.border, (width / 6, y - 12), (width - width / 6, y - 12), 1);
    surface.blr(&img, img.get_rect().center_at((width / 2, y)));
}

/// `ui.pulse(speed, lo, hi)` - Sinus-Puls fuer Blink- und Atem-Animationen.
pub fn pulse(speed: f64, lo: f64, hi: f64) -> f64 {
    let ticks = pg::ticks() as f64 / 1000.0;
    lo + (hi - lo) * (0.5 + 0.5 * (ticks * speed).sin())
}

/// `ui.pulse(speed)` mit `lo=0.35, hi=1.0`.
pub fn pulse_d(speed: f64) -> f64 {
    pulse(speed, 0.35, 1.0)
}

// ---------------------------------------------------------------------------
//  Partikel
// ---------------------------------------------------------------------------

/// `ui.spawn_burst(x, y, color=None, n=18, speed=260)`
pub fn spawn_burst(x: f64, y: f64, color: Option<Color>, n: usize, speed: f64) {
    if !fx().sparks {
        return;
    }
    let color = color.unwrap_or_else(accent);
    for _ in 0..n {
        if with(|s| s.particles.len()) >= MAX_PARTICLES {
            break;
        }
        let ang = py::uniform(0.0, py::TAU);
        let sp = speed * py::uniform(0.25, 1.0);
        let life = py::uniform(0.35, 0.7);
        let size = py::uniform(1.5, 3.2);
        with(|s| {
            s.particles.push(Particle {
                kind: PartKind::Spark,
                x,
                y,
                vx: ang.cos() * sp,
                vy: ang.sin() * sp - 40.0,
                age: 0.0,
                life,
                color,
                size,
                spin: 0.0,
                phase: 0.0,
            })
        });
    }
}

/// `ui.spawn_confetti(w, h, n=90)`
pub fn spawn_confetti(w: f64, h: f64, n: usize) {
    let p = pal();
    let cols = [p.accent, p.accent2, p.green, p.gold, p.red, Color::rgb(240, 240, 250)];
    let n = if is_modern() { n.min(60) } else { n };
    for _ in 0..n {
        if with(|s| s.particles.len()) >= MAX_PARTICLES {
            break;
        }
        let x = py::uniform(0.0, w);
        let y = py::uniform(-h * 0.25, 0.0);
        let vx = py::uniform(-30.0, 30.0);
        let vy = py::uniform(90.0, 220.0);
        let life = py::uniform(2.0, 3.4);
        let color = py::choice(&cols);
        let spin = py::uniform(4.0, 9.0);
        let phase = py::uniform(0.0, py::TAU);
        with(|s| {
            s.particles.push(Particle {
                kind: PartKind::Confetti,
                x,
                y,
                vx,
                vy,
                age: 0.0,
                life,
                color,
                size: 0.0,
                spin,
                phase,
            })
        });
    }
}

fn draw_particles(surface: &mut Surface, dt: f64) {
    let mut liste = with(|s| std::mem::take(&mut s.particles));
    if liste.is_empty() {
        return;
    }
    let (w, h) = (surface.w() as f64, surface.h() as f64);
    let bg = bg_bottom();
    let mut alive = Vec::with_capacity(liste.len());
    for mut p in liste.drain(..) {
        p.age += dt;
        if p.age >= p.life {
            continue;
        }
        let fade = 1.0 - p.age / p.life;
        if p.kind == PartKind::Spark {
            p.vy += 420.0 * dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            if -4.0 <= p.x && p.x < w && -4.0 <= p.y && p.y < h {
                let s = 1.max((p.size * fade + 0.5) as i32);
                surface.fill_rf(
                    mix(Color::rgb(0, 0, 0), p.color, fade),
                    (p.x as i32, p.y as i32, s, s),
                    BLEND_ADD,
                );
            }
        } else {
            p.x += (p.vx + 34.0 * (p.age * 3.0 + p.phase).sin()) * dt;
            p.y += p.vy * dt;
            if p.y < h + 8.0 {
                let bw = 2 + (4.0 * (p.age * p.spin + p.phase).sin().abs()) as i32;
                let col = mix(bg, p.color, (fade * 1.8).min(1.0));
                draw::rect(surface, col, (p.x as i32, p.y as i32, bw, 5), 0);
            }
        }
        alive.push(p);
    }
    with(|s| {
        // Was waehrend des Zeichnens neu dazukam, bleibt hinten dran.
        let neu = std::mem::take(&mut s.particles);
        s.particles = alive;
        s.particles.extend(neu);
    });
}

// ---------------------------------------------------------------------------
//  Bildschirm-Uebergang
// ---------------------------------------------------------------------------

/// `ui.begin_transition(dur=None)`
pub fn begin_transition(dur: Option<f64>) {
    let dur = dur.unwrap_or_else(|| fx().trans_dur);
    let now = pg::ticks();
    with(|s| {
        if dur <= 0.0 {
            s.trans_start = None;
            return;
        }
        s.trans_start = Some(now);
        s.trans_dur = dur;
    });
}

fn fade_surface(w: i32, h: i32) -> Surface {
    if let Some(s) = with(|s| s.fade_cache.get(&(w, h)).cloned()) {
        return s;
    }
    let mut surf = pg::surface(w, h);
    surf.fill_c(Color::rgb(6, 8, 16));
    with(|s| {
        if s.fade_cache.len() > 3 {
            s.fade_cache.clear();
        }
        s.fade_cache.insert((w, h), surf.clone());
    });
    surf
}

fn draw_transition(surface: &mut Surface, w: i32, h: i32) {
    let (start, dur) = with(|s| (s.trans_start, s.trans_dur));
    let Some(start) = start else { return };
    let t = (pg::ticks() as f64 - start as f64) / 1000.0 / dur;
    if t >= 1.0 {
        with(|s| s.trans_start = None);
        return;
    }
    let mut ov = fade_surface(w, h);
    ov.alpha_set((210.0 * py::pow(1.0 - t, 1.5)) as i32);
    surface.bl(&ov, (0, 0));
    if !fx().scanline {
        return;
    }
    let ac = accent();
    let y = (h as f64 * t) as i32;
    let glow_a = (90.0 * (1.0 - t)) as i32;
    let mut line = pg::surface_alpha(w, 7);
    draw::rect(&mut line, pg::alpha(ac, glow_a), (0, 0, w, 7), 0);
    draw::rect(
        &mut line,
        pg::alpha(mix(ac, Color::rgb(255, 255, 255), 0.5), 255.min(glow_a * 2)),
        (0, 3, w, 1),
        0,
    );
    surface.bl(&line, (0, y - 3));
}

/// `ui.draw_fx(surface, w, h, dt)` - Partikel und Uebergang, einmal je Bild.
pub fn draw_fx(surface: &mut Surface, w: i32, h: i32, dt: f64) {
    draw_particles(surface, dt);
    draw_transition(surface, w, h);
}

// ---------------------------------------------------------------------------
//  Texteingabe
// ---------------------------------------------------------------------------

/// `ui.TextInput` - einzeiliges Eingabefeld mit Schreibmarke.
#[derive(Debug, Clone)]
pub struct TextInput {
    /// Hoechstlaenge in Zeichen.
    pub maxlen: usize,
    /// Erlaubte Zeichen (klein), `None` = alle druckbaren.
    pub charset: Option<&'static str>,
    /// Grauer Text im leeren Feld.
    pub placeholder: String,
    /// Schreibmarke (Zeichenindex).
    pub caret: usize,
    /// Der Inhalt.
    pub text: String,
}

impl TextInput {
    /// Erlaubte Zeichen einer Bahn-id (`TextInput.ID_CHARS`).
    pub const ID_CHARS: &'static str = "abcdefghijklmnopqrstuvwxyz0123456789-_";
    /// Tasten, die das Feld durchreicht (`TextInput.PASS_THROUGH`).
    pub const PASS_THROUGH: &'static [&'static str] =
        &["Tab", "ISO_Left_Tab", "Return", "KP_Enter", "Escape", "Up", "Down"];

    /// `TextInput(text, maxlen, charset, placeholder)`
    pub fn new(text: &str, maxlen: usize, charset: Option<&'static str>, placeholder: &str) -> TextInput {
        let mut t = TextInput { maxlen, charset, placeholder: placeholder.to_string(), caret: 0, text: String::new() };
        t.set_text(text);
        t
    }

    fn ok(&self, ch: char) -> bool {
        if (ch as u32) < 32 || ch == '\x7f' {
            return false;
        }
        match self.charset {
            None => true,
            Some(cs) => ch.to_lowercase().all(|l| cs.contains(l)),
        }
    }

    fn len(&self) -> usize {
        self.text.chars().count()
    }

    fn byte_at(&self, idx: usize) -> usize {
        self.text.char_indices().nth(idx).map(|(b, _)| b).unwrap_or(self.text.len())
    }

    /// `set_text(text)` - filtert und kuerzt wie bei der Eingabe.
    pub fn set_text(&mut self, text: &str) {
        self.text = text.chars().filter(|c| self.ok(*c)).take(self.maxlen).collect();
        self.caret = self.len();
    }

    /// `insert(ch)`
    pub fn insert(&mut self, ch: char) -> bool {
        if self.len() >= self.maxlen || !self.ok(ch) {
            return false;
        }
        let ch: String = if self.charset.is_some() { ch.to_lowercase().collect() } else { ch.to_string() };
        let b = self.byte_at(self.caret);
        self.text.insert_str(b, &ch);
        self.caret += 1;
        true
    }

    /// `handle(event)` - `true` = Ereignis verbraucht.
    pub fn handle(&mut self, ev: &InputEvent) -> bool {
        if ev.kind != Kind::KeyDown {
            return false;
        }
        let key = ev.key.as_str();
        if Self::PASS_THROUGH.contains(&key) {
            return false;
        }
        match key {
            "BackSpace" => {
                if self.caret > 0 {
                    let a = self.byte_at(self.caret - 1);
                    let b = self.byte_at(self.caret);
                    self.text.replace_range(a..b, "");
                    self.caret -= 1;
                }
                return true;
            }
            "Delete" => {
                if self.caret < self.len() {
                    let a = self.byte_at(self.caret);
                    let b = self.byte_at(self.caret + 1);
                    self.text.replace_range(a..b, "");
                }
                return true;
            }
            "Left" => {
                self.caret = self.caret.saturating_sub(1);
                return true;
            }
            "Right" => {
                self.caret = (self.caret + 1).min(self.len());
                return true;
            }
            "Home" | "KP_Home" => {
                self.caret = 0;
                return true;
            }
            "End" | "KP_End" => {
                self.caret = self.len();
                return true;
            }
            _ => {}
        }
        if let Some(ch) = ev.char.chars().next() {
            return self.insert(ch);
        }
        if key == "space" {
            return self.insert(' ');
        }
        let mut it = key.chars();
        if let (Some(c), None) = (it.next(), it.next()) {
            if !c.is_control() {
                return self.insert(c);
            }
        }
        false
    }

    /// `draw(surface, rect, fnt, focused, invalid)`
    pub fn draw(&self, surface: &mut Surface, rect: impl IntoRect, fnt: &Font, focused: bool, invalid: bool) -> Rect {
        let p = pal();
        let r = rect.rect();
        let border = if invalid { p.red } else if focused { p.accent } else { p.border };
        draw::rect_r(surface, if focused { p.panel_light } else { p.panel }, r, 0, 7);
        draw::rect_r(surface, border, r, if focused || invalid { 2 } else { 1 }, 7);
        let pad = 8;
        let inner = r.w - 2 * pad;
        let show = if self.text.is_empty() { self.placeholder.as_str() } else { self.text.as_str() };
        let col = if self.text.is_empty() { p.text_faint } else { p.text };
        let vor: String = self.text.chars().take(self.caret).collect();
        let mut off = 0;
        if !self.text.is_empty() {
            let upto = fnt.size(&vor).0;
            if upto > inner {
                off = upto - inner;
            }
        }
        let img = fnt.render(show, true, col);
        let clip = surface.get_clip();
        surface.set_clip(Some(r.inflate(-4, -4)));
        surface.bl(&img, (r.x + pad - off, r.centery() - img.h() / 2));
        if focused {
            let cx = r.x + pad - off + fnt.size(&vor).0;
            if pulse(3.0, 0.0, 1.99) as i32 == 0 {
                draw::rect(surface, p.accent, (cx, r.y + 5, 2, r.h - 10), 0);
            }
        }
        surface.set_clip(Some(clip));
        r
    }
}
