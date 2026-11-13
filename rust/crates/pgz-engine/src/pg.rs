//! pg.rs
//! =====
//! Die pygame-Fassade ueber rustgame.
//!
//! Die Spiele sind aus Python uebertragen, und Python ruft pygame mit
//! **beliebigen Zahlen** auf: `pygame.draw.rect(s, c, (20, self.y, 12, 80))`
//! mit einem Komma-`self.y`, `blit(img, (x / 2, y))` und so fort. pygame
//! schneidet dabei **zur Null hin ab** (`pg_IntFromObj`, `pgRect_FromObject`,
//! gemessen in `rustgame-py`). Die Fassade nimmt deshalb jede Zahl an
//! ([`Num`]) und schneidet genauso ab - dann steht in Rust dieselbe Zeile
//! wie in Python und zeichnet dasselbe Bild.
//!
//! Die Grundtypen (`Surface`, `Rect`, `Color`) sind die von rustgame; hier
//! kommen nur die Umwandlungen und die Zeichenaufrufe in pygames Form dazu.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

pub use rustgame_core::blend::flags::{
    BLEND_ADD, BLEND_MAX, BLEND_MIN, BLEND_MULT, BLEND_PREMULTIPLIED, BLEND_RGBA_ADD,
    BLEND_RGBA_MAX, BLEND_RGBA_MIN, BLEND_RGBA_MULT, BLEND_RGBA_SUB, BLEND_RGB_ADD,
    BLEND_RGB_MAX, BLEND_RGB_MIN, BLEND_RGB_MULT, BLEND_RGB_SUB, BLEND_SUB,
};
pub use rustgame_core::{Color, FRect, Rect, Surface, SRCALPHA};

// ---------------------------------------------------------------------------
//  Zahlen, Punkte, Rechtecke, Farben
// ---------------------------------------------------------------------------

/// Alles, was pygame als Zahl annimmt.
pub trait Num: Copy {
    /// Ganze Zahl wie pygame: Komma wird zur Null hin abgeschnitten.
    fn i(self) -> i32;
    /// Als Kommazahl.
    fn f(self) -> f64;
}

macro_rules! num_ganz {
    ($($t:ty),*) => {$(
        impl Num for $t {
            #[inline] fn i(self) -> i32 { self as i32 }
            #[inline] fn f(self) -> f64 { self as f64 }
        }
    )*};
}
num_ganz!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

impl Num for f32 {
    #[inline]
    fn i(self) -> i32 {
        self as i32
    }
    #[inline]
    fn f(self) -> f64 {
        self as f64
    }
}

impl Num for f64 {
    #[inline]
    fn i(self) -> i32 {
        self as i32
    }
    #[inline]
    fn f(self) -> f64 {
        self
    }
}

/// Ein Punkt `(x, y)`.
pub trait Pt: Copy {
    /// Als ganze Zahlen (abgeschnitten).
    fn pt(self) -> (i32, i32);
    /// Als Kommazahlen.
    fn ptf(self) -> (f64, f64);
}

impl<A: Num, B: Num> Pt for (A, B) {
    #[inline]
    fn pt(self) -> (i32, i32) {
        (self.0.i(), self.1.i())
    }
    #[inline]
    fn ptf(self) -> (f64, f64) {
        (self.0.f(), self.1.f())
    }
}

impl<A: Num> Pt for [A; 2] {
    #[inline]
    fn pt(self) -> (i32, i32) {
        (self[0].i(), self[1].i())
    }
    #[inline]
    fn ptf(self) -> (f64, f64) {
        (self[0].f(), self[1].f())
    }
}

/// Alles, was als Rechteck durchgeht (`pgRect_FromObject`).
pub trait IntoRect {
    /// Das Rechteck.
    fn rect(self) -> Rect;
}

impl IntoRect for Rect {
    #[inline]
    fn rect(self) -> Rect {
        self
    }
}

impl IntoRect for &Rect {
    #[inline]
    fn rect(self) -> Rect {
        *self
    }
}

impl<A: Num, B: Num, C: Num, D: Num> IntoRect for (A, B, C, D) {
    #[inline]
    fn rect(self) -> Rect {
        Rect::new(self.0.i(), self.1.i(), self.2.i(), self.3.i())
    }
}

impl<P: Pt, Q: Pt> IntoRect for (P, Q)
where
    P: Pt,
{
    #[inline]
    fn rect(self) -> Rect {
        let (x, y) = self.0.pt();
        let (w, h) = self.1.pt();
        Rect::new(x, y, w, h)
    }
}

/// `pygame.Rect(x, y, w, h)` aus beliebigen Zahlen.
#[inline]
pub fn rect(x: impl Num, y: impl Num, w: impl Num, h: impl Num) -> Rect {
    Rect::new(x.i(), y.i(), w.i(), h.i())
}

/// Alles, was als Farbe durchgeht.
///
/// Ganzzahl-Tupel sind `i32`, weil Python dort mit gerechneten Werten
/// arbeitet (`(c, c + 6, c + 18)`); sie werden auf `0..=255` begrenzt.
pub trait Col: Copy {
    /// Die Farbe.
    fn c(self) -> Color;
}

impl Col for Color {
    #[inline]
    fn c(self) -> Color {
        self
    }
}

#[inline]
fn kanal(v: i32) -> u8 {
    v.clamp(0, 255) as u8
}

impl Col for (i32, i32, i32) {
    #[inline]
    fn c(self) -> Color {
        Color::rgb(kanal(self.0), kanal(self.1), kanal(self.2))
    }
}

impl Col for (i32, i32, i32, i32) {
    #[inline]
    fn c(self) -> Color {
        Color::rgba(kanal(self.0), kanal(self.1), kanal(self.2), kanal(self.3))
    }
}

impl Col for (u8, u8, u8) {
    #[inline]
    fn c(self) -> Color {
        Color::rgb(self.0, self.1, self.2)
    }
}

impl Col for (u8, u8, u8, u8) {
    #[inline]
    fn c(self) -> Color {
        Color::rgba(self.0, self.1, self.2, self.3)
    }
}

/// Deckende Farbe aus gerechneten Werten.
#[inline]
pub fn rgb(r: impl Num, g: impl Num, b: impl Num) -> Color {
    Color::rgb(kanal(r.i()), kanal(g.i()), kanal(b.i()))
}

/// Farbe mit Deckung aus gerechneten Werten.
#[inline]
pub fn rgba(r: impl Num, g: impl Num, b: impl Num, a: impl Num) -> Color {
    Color::rgba(kanal(r.i()), kanal(g.i()), kanal(b.i()), kanal(a.i()))
}

/// `(*farbe[:3], a)` - dieselbe Farbe mit neuer Deckung.
#[inline]
pub fn alpha(c: impl Col, a: impl Num) -> Color {
    let c = c.c();
    Color::rgba(c.r, c.g, c.b, kanal(a.i()))
}

/// `tuple(int(v * f) for v in farbe)` - jede Komponente skaliert.
#[inline]
pub fn scaled(c: impl Col, f: f64) -> Color {
    let c = c.c();
    rgb(c.r as f64 * f, c.g as f64 * f, c.b as f64 * f)
}

// ---------------------------------------------------------------------------
//  Rechtecke: pygames Schreibweise `get_rect(center=...)`
// ---------------------------------------------------------------------------

/// Die neun Anker von pygame als Bauform mit beliebigen Zahlen.
pub trait RectExt: Sized {
    /// `get_rect(center=p)`
    fn center_at(self, p: impl Pt) -> Self;
    /// `get_rect(topleft=p)`
    fn topleft_at(self, p: impl Pt) -> Self;
    /// `get_rect(topright=p)`
    fn topright_at(self, p: impl Pt) -> Self;
    /// `get_rect(bottomleft=p)`
    fn bottomleft_at(self, p: impl Pt) -> Self;
    /// `get_rect(bottomright=p)`
    fn bottomright_at(self, p: impl Pt) -> Self;
    /// `get_rect(midtop=p)`
    fn midtop_at(self, p: impl Pt) -> Self;
    /// `get_rect(midbottom=p)`
    fn midbottom_at(self, p: impl Pt) -> Self;
    /// `get_rect(midleft=p)`
    fn midleft_at(self, p: impl Pt) -> Self;
    /// `get_rect(midright=p)`
    fn midright_at(self, p: impl Pt) -> Self;
    /// `get_rect(centerx=x)`
    fn centerx_at(self, x: impl Num) -> Self;
    /// `get_rect(centery=y)`
    fn centery_at(self, y: impl Num) -> Self;
    /// `r.collidepoint(p)` mit beliebigen Zahlen.
    fn hit(&self, p: impl Pt) -> bool;
}

impl RectExt for Rect {
    fn center_at(self, p: impl Pt) -> Self {
        self.with_center(p.pt())
    }
    fn topleft_at(self, p: impl Pt) -> Self {
        self.with_topleft(p.pt())
    }
    fn topright_at(self, p: impl Pt) -> Self {
        self.with_topright(p.pt())
    }
    fn bottomleft_at(self, p: impl Pt) -> Self {
        self.with_bottomleft(p.pt())
    }
    fn bottomright_at(self, p: impl Pt) -> Self {
        self.with_bottomright(p.pt())
    }
    fn midtop_at(self, p: impl Pt) -> Self {
        self.with_midtop(p.pt())
    }
    fn midbottom_at(self, p: impl Pt) -> Self {
        self.with_midbottom(p.pt())
    }
    fn midleft_at(self, p: impl Pt) -> Self {
        self.with_midleft(p.pt())
    }
    fn midright_at(self, p: impl Pt) -> Self {
        self.with_midright(p.pt())
    }
    fn centerx_at(self, x: impl Num) -> Self {
        self.with_centerx(x.i())
    }
    fn centery_at(self, y: impl Num) -> Self {
        self.with_centery(y.i())
    }
    fn hit(&self, p: impl Pt) -> bool {
        self.collidepoint(p.pt())
    }
}

// ---------------------------------------------------------------------------
//  Flaechen
// ---------------------------------------------------------------------------

/// `pygame.Surface((w, h))` - undurchsichtig schwarz.
pub fn surface(w: impl Num, h: impl Num) -> Surface {
    Surface::new((w.i().max(0) as u32, h.i().max(0) as u32))
}

/// `pygame.Surface((w, h), pygame.SRCALPHA)` - ganz durchsichtig.
pub fn surface_alpha(w: impl Num, h: impl Num) -> Surface {
    Surface::with_flags((w.i().max(0) as u32, h.i().max(0) as u32), SRCALPHA)
}

/// Bequeme Formen fuer `blit` und `fill` mit pygames Zahlenregeln.
pub trait SurfExt {
    /// `blit(src, pos)`
    fn bl(&mut self, src: &Surface, pos: impl Pt) -> Rect;
    /// `blit(src, rect)` - linke obere Ecke des Rechtecks.
    fn blr(&mut self, src: &Surface, r: Rect) -> Rect;
    /// `blit(src, pos, area)`
    fn bl_area(&mut self, src: &Surface, pos: impl Pt, area: impl IntoRect) -> Rect;
    /// `blit(src, pos, special_flags=f)`
    fn bl_flags(&mut self, src: &Surface, pos: impl Pt, flags: u32) -> Rect;
    /// `blit(src, pos, area, special_flags=f)`
    fn bl_area_flags(&mut self, src: &Surface, pos: impl Pt, area: impl IntoRect, flags: u32)
        -> Rect;
    /// `fill(color)`
    fn fill_c(&mut self, c: impl Col) -> Rect;
    /// `fill(color, rect)`
    fn fill_r(&mut self, c: impl Col, r: impl IntoRect) -> Rect;
    /// `fill(color, rect, special_flags=f)`
    fn fill_rf(&mut self, c: impl Col, r: impl IntoRect, flags: u32) -> Rect;
    /// `fill(color, special_flags=f)` ueber die ganze Flaeche.
    fn fill_f(&mut self, c: impl Col, flags: u32) -> Rect;
    /// Breite als `i32`.
    fn w(&self) -> i32;
    /// Hoehe als `i32`.
    fn h(&self) -> i32;
    /// `set_alpha(a)`
    fn alpha_set(&mut self, a: impl Num);
}

impl SurfExt for Surface {
    #[inline]
    fn bl(&mut self, src: &Surface, pos: impl Pt) -> Rect {
        self.blit(src, pos.pt())
    }
    #[inline]
    fn blr(&mut self, src: &Surface, r: Rect) -> Rect {
        self.blit(src, (r.x, r.y))
    }
    #[inline]
    fn bl_area(&mut self, src: &Surface, pos: impl Pt, area: impl IntoRect) -> Rect {
        self.blit_area(src, pos.pt(), area.rect())
    }
    #[inline]
    fn bl_flags(&mut self, src: &Surface, pos: impl Pt, flags: u32) -> Rect {
        self.blit_ex(src, pos.pt(), None, flags)
    }
    #[inline]
    fn bl_area_flags(
        &mut self,
        src: &Surface,
        pos: impl Pt,
        area: impl IntoRect,
        flags: u32,
    ) -> Rect {
        self.blit_ex(src, pos.pt(), Some(area.rect()), flags)
    }
    #[inline]
    fn fill_c(&mut self, c: impl Col) -> Rect {
        self.fill(c.c())
    }
    #[inline]
    fn fill_r(&mut self, c: impl Col, r: impl IntoRect) -> Rect {
        self.fill_rect(c.c(), r.rect())
    }
    #[inline]
    fn fill_rf(&mut self, c: impl Col, r: impl IntoRect, flags: u32) -> Rect {
        self.fill_ex(c.c(), Some(r.rect()), flags)
    }
    #[inline]
    fn fill_f(&mut self, c: impl Col, flags: u32) -> Rect {
        self.fill_ex(c.c(), None, flags)
    }
    #[inline]
    fn w(&self) -> i32 {
        self.get_width() as i32
    }
    #[inline]
    fn h(&self) -> i32 {
        self.get_height() as i32
    }
    #[inline]
    fn alpha_set(&mut self, a: impl Num) {
        self.set_alpha(Some(a.i().clamp(0, 255) as u8));
    }
}

// ---------------------------------------------------------------------------
//  pygame.draw
// ---------------------------------------------------------------------------

/// `pygame.draw` mit pygames Aufrufform.
pub mod draw {
    use super::*;
    use rustgame_core::draw as d;

    /// `pygame.draw.rect(s, c, r, width)`
    pub fn rect(s: &mut Surface, c: impl Col, r: impl IntoRect, width: i32) -> Rect {
        d::rect(s, c.c(), r.rect(), width)
    }

    /// `pygame.draw.rect(s, c, r, width, border_radius=radius)`
    pub fn rect_r(s: &mut Surface, c: impl Col, r: impl IntoRect, width: i32, radius: i32) -> Rect {
        if radius > 0 {
            d::rect_radius(s, c.c(), r.rect(), width, radius)
        } else {
            d::rect(s, c.c(), r.rect(), width)
        }
    }

    /// `pygame.draw.rect(..., border_top_left_radius=.., ...)` - `-1` heisst
    /// "nicht gesetzt", dann gilt `radius`.
    #[allow(clippy::too_many_arguments)]
    pub fn rect_corners(
        s: &mut Surface,
        c: impl Col,
        r: impl IntoRect,
        width: i32,
        radius: i32,
        tl: i32,
        tr: i32,
        bl: i32,
        br: i32,
    ) -> Rect {
        if [tl, tr, bl, br].iter().any(|e| *e >= 0) {
            d::rect_corners(s, c.c(), r.rect(), width, radius, tl, tr, bl, br)
        } else {
            rect_r(s, c, r, width, radius)
        }
    }

    /// `pygame.draw.circle(s, c, center, radius, width)`
    pub fn circle(s: &mut Surface, c: impl Col, center: impl Pt, radius: impl Num, width: i32) -> Rect {
        d::circle(s, c.c(), center.pt(), radius.i(), width)
    }

    /// `pygame.draw.circle(..., draw_top_right=.., ...)`
    #[allow(clippy::too_many_arguments)]
    pub fn circle_q(
        s: &mut Surface,
        c: impl Col,
        center: impl Pt,
        radius: impl Num,
        width: i32,
        tr: bool,
        tl: bool,
        bl: bool,
        br: bool,
    ) -> Rect {
        let q = d::Quadranten { top_right: tr, top_left: tl, bottom_left: bl, bottom_right: br };
        d::circle_quadranten(s, c.c(), center.pt(), radius.i(), width, q)
    }

    /// `pygame.draw.aacircle(s, c, center, radius, width)`
    pub fn aacircle(s: &mut Surface, c: impl Col, center: impl Pt, radius: impl Num, width: i32) -> Rect {
        d::aacircle(s, c.c(), center.pt(), radius.i(), width)
    }

    /// `pygame.draw.ellipse(s, c, r, width)`
    pub fn ellipse(s: &mut Surface, c: impl Col, r: impl IntoRect, width: i32) -> Rect {
        d::ellipse(s, c.c(), r.rect(), width)
    }

    /// `pygame.draw.arc(s, c, r, start, stop, width)`
    pub fn arc(s: &mut Surface, c: impl Col, r: impl IntoRect, start: f64, stop: f64, width: i32) -> Rect {
        d::arc(s, c.c(), r.rect(), start, stop, width)
    }

    /// `pygame.draw.line(s, c, a, b, width)`
    pub fn line(s: &mut Surface, c: impl Col, a: impl Pt, b: impl Pt, width: i32) -> Rect {
        d::line(s, c.c(), a.pt(), b.pt(), width)
    }

    /// `pygame.draw.lines(s, c, closed, points, width)`
    pub fn lines<P: Pt>(s: &mut Surface, c: impl Col, closed: bool, pts: &[P], width: i32) -> Rect {
        let v: Vec<(i32, i32)> = pts.iter().map(|p| p.pt()).collect();
        if v.len() < 2 {
            return Rect::new(0, 0, 0, 0);
        }
        d::lines(s, c.c(), closed, &v, width)
    }

    /// `pygame.draw.polygon(s, c, points, width)`
    pub fn polygon<P: Pt>(s: &mut Surface, c: impl Col, pts: &[P], width: i32) -> Rect {
        let v: Vec<(i32, i32)> = pts.iter().map(|p| p.pt()).collect();
        if v.len() < if width != 0 { 2 } else { 3 } {
            return Rect::new(0, 0, 0, 0);
        }
        d::polygon(s, c.c(), &v, width)
    }

    /// `pygame.draw.aaline(s, c, a, b)` (pygame rechnet hier in `f32`).
    pub fn aaline(s: &mut Surface, c: impl Col, a: impl Pt, b: impl Pt) -> Rect {
        let (ax, ay) = a.ptf();
        let (bx, by) = b.ptf();
        d::aaline(s, c.c(), (ax as f32, ay as f32), (bx as f32, by as f32), 1)
    }

    /// `pygame.draw.aaline(s, c, a, b, width)`
    pub fn aaline_w(s: &mut Surface, c: impl Col, a: impl Pt, b: impl Pt, width: i32) -> Rect {
        let (ax, ay) = a.ptf();
        let (bx, by) = b.ptf();
        d::aaline(s, c.c(), (ax as f32, ay as f32), (bx as f32, by as f32), width)
    }

    /// `pygame.draw.aalines(s, c, closed, points)`
    pub fn aalines<P: Pt>(s: &mut Surface, c: impl Col, closed: bool, pts: &[P]) -> Rect {
        let v: Vec<(f32, f32)> = pts
            .iter()
            .map(|p| {
                let (x, y) = p.ptf();
                (x as f32, y as f32)
            })
            .collect();
        if v.len() < 2 {
            return Rect::new(0, 0, 0, 0);
        }
        d::aalines(s, c.c(), closed, &v)
    }
}

// ---------------------------------------------------------------------------
//  pygame.transform
// ---------------------------------------------------------------------------

/// `pygame.transform` mit beliebigen Zahlen.
pub mod transform {
    use super::*;
    use rustgame_core::transform as t;

    fn groesse(w: impl Num, h: impl Num) -> (u32, u32) {
        (w.i().max(0) as u32, h.i().max(0) as u32)
    }

    /// `pygame.transform.scale(s, (w, h))`
    pub fn scale(s: &Surface, w: impl Num, h: impl Num) -> Surface {
        t::scale(s, groesse(w, h))
    }

    /// `pygame.transform.smoothscale(s, (w, h))`
    pub fn smoothscale(s: &Surface, w: impl Num, h: impl Num) -> Surface {
        t::smoothscale(s, groesse(w, h))
    }

    /// `pygame.transform.scale_by(s, f)`
    pub fn scale_by(s: &Surface, f: f64) -> Surface {
        t::scale_by(s, f as f32, f as f32)
    }

    /// `pygame.transform.smoothscale_by(s, f)`
    pub fn smoothscale_by(s: &Surface, f: f64) -> Surface {
        t::smoothscale_by(s, f as f32, f as f32)
    }

    /// `pygame.transform.rotate(s, winkel)`
    pub fn rotate(s: &Surface, winkel: f64) -> Surface {
        t::rotate(s, winkel as f32)
    }

    /// `pygame.transform.rotozoom(s, winkel, faktor)`
    pub fn rotozoom(s: &Surface, winkel: f64, faktor: f64) -> Surface {
        t::rotozoom(s, winkel as f32, faktor as f32)
    }

    /// `pygame.transform.flip(s, x, y)`
    pub fn flip(s: &Surface, x: bool, y: bool) -> Surface {
        t::flip(s, x, y)
    }

    /// `pygame.transform.grayscale(s)`
    pub fn grayscale(s: &Surface) -> Surface {
        t::grayscale(s)
    }

    /// `pygame.transform.box_blur(s, r)`
    pub fn box_blur(s: &Surface, r: i32) -> Surface {
        t::box_blur(s, r, false).unwrap_or_else(|_| s.copy())
    }

    /// `pygame.transform.gaussian_blur(s, r)`
    pub fn gaussian_blur(s: &Surface, r: i32) -> Surface {
        t::gaussian_blur(s, r, false).unwrap_or_else(|_| s.copy())
    }
}

// ---------------------------------------------------------------------------
//  pygame.image
// ---------------------------------------------------------------------------

/// `pygame.image` - nur, was PyGameZ braucht.
pub mod image {
    use super::*;

    /// `pygame.image.frombytes(data, (w, h), "RGB"/"RGBA")`
    pub fn frombytes(daten: &[u8], w: i32, h: i32, format: &str) -> Surface {
        rustgame_core::image::frombytes(daten, (w.max(0) as u32, h.max(0) as u32), format, false)
            .unwrap_or_else(|_| super::surface(w, h))
    }

    /// `pygame.image.load(pfad)`
    pub fn load(pfad: impl AsRef<std::path::Path>) -> Option<Surface> {
        rustgame_core::image::load(pfad).ok()
    }

    /// `pygame.image.load(BytesIO(daten))`
    pub fn load_bytes(daten: &[u8]) -> Option<Surface> {
        rustgame_core::image::load_from_bytes(daten).ok()
    }

    /// `pygame.image.save(s, pfad)`
    pub fn save(s: &Surface, pfad: impl AsRef<std::path::Path>) -> bool {
        rustgame_core::image::save(s, pfad).is_ok()
    }

    /// `pygame.image.tobytes(s, fmt)`
    pub fn tobytes(s: &Surface, format: &str) -> Vec<u8> {
        rustgame_core::image::tobytes(s, format).unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
//  Zeit
// ---------------------------------------------------------------------------

thread_local! {
    /// Fester Takt fuer den Pruefstand: ist er gesetzt, liefert [`ticks`]
    /// diesen Wert statt der Uhr - dann ist jedes Bild wiederholbar.
    static FESTE_ZEIT: Cell<Option<u64>> = const { Cell::new(None) };
}

/// `pygame.time.get_ticks()` - Millisekunden seit dem Start.
pub fn ticks() -> u64 {
    FESTE_ZEIT.with(|z| z.get()).unwrap_or_else(rustgame_core::time::get_ticks)
}

/// Fester Takt (Pruefstand): `Some(ms)` friert die Uhr ein, `None` loest.
pub fn set_fixed_ticks(ms: Option<u64>) {
    FESTE_ZEIT.with(|z| z.set(ms));
}

/// Uhr starten (`pygame.init()` tut das bei pygame).
pub fn init_time() {
    rustgame_core::time::init();
}

// ---------------------------------------------------------------------------
//  pygame.font - mit Bildzwischenspeicher
// ---------------------------------------------------------------------------

/// Ein Zwischenspeicher-Schluessel: Text, Glaettung, Farbe, Hintergrund.
type TextKey = (String, bool, u32, Option<u32>);

struct FontInner {
    font: RefCell<rustgame_font::Font>,
    cache: RefCell<HashMap<TextKey, Surface>>,
    id: usize,
}

/// `pygame.font.Font` - ein Verweis, wie in Python.
///
/// **Warum ein Zwischenspeicher:** Die Spiele rendern ihren Text in jedem
/// Bild neu (`font.render(str(score), True, ui.TEXT)`). In pygame kostet
/// das SDL_ttf wenig; rustgames Text ist pixelgleich, aber mehrfach
/// langsamer. Gleicher Text, gleiche Farbe gibt dasselbe Bild - also wird
/// es nur einmal gesetzt.
///
/// **Die Falle dabei:** `render` gibt eine Flaeche zurueck, die sich den
/// Speicher mit dem Zwischenspeicher teilt. Wer in ein Textbild hinein
/// zeichnet, nimmt vorher `.copy()`.
#[derive(Clone)]
pub struct Font(Rc<FontInner>);

thread_local! {
    static FONT_ID: Cell<usize> = const { Cell::new(1) };
}

const CACHE_MAX: usize = 600;

impl Font {
    fn aus(font: rustgame_font::Font) -> Font {
        let id = FONT_ID.with(|c| {
            let v = c.get();
            c.set(v + 1);
            v
        });
        Font(Rc::new(FontInner {
            font: RefCell::new(font),
            cache: RefCell::new(HashMap::new()),
            id,
        }))
    }

    /// `pygame.font.SysFont(name, size, bold, italic)`
    pub fn sys(name: &str, size: i32, bold: bool, italic: bool) -> Font {
        let f = rustgame_font::sys_font(name, size, bold, italic)
            .or_else(|_| rustgame_font::default_font(size))
            .expect("keine Schrift gefunden - freesansbold.ttf fehlt");
        Font::aus(f)
    }

    /// `pygame.font.Font(None, size)`
    pub fn default(size: i32) -> Font {
        let f = rustgame_font::default_font(size).expect("freesansbold.ttf fehlt");
        Font::aus(f)
    }

    /// `pygame.font.Font(pfad, size)`
    pub fn from_file(pfad: impl AsRef<std::path::Path>, size: i32) -> Option<Font> {
        rustgame_font::Font::new(pfad, size).ok().map(Font::aus)
    }

    /// Eindeutige Nummer (Python: `id(font)`) fuer Zwischenspeicher.
    pub fn id(&self) -> usize {
        self.0.id
    }

    /// `font.render(text, aa, color)`
    pub fn render(&self, text: &str, aa: bool, color: impl Col) -> Surface {
        self.render_ex(text, aa, color.c(), None)
    }

    /// `font.render(text, aa, color, background)`
    pub fn render_bg(&self, text: &str, aa: bool, color: impl Col, bg: impl Col) -> Surface {
        self.render_ex(text, aa, color.c(), Some(bg.c()))
    }

    fn render_ex(&self, text: &str, aa: bool, color: Color, bg: Option<Color>) -> Surface {
        let key = (text.to_string(), aa, color.to_argb(), bg.map(|b| b.to_argb()));
        if let Some(s) = self.0.cache.borrow().get(&key) {
            return s.clone();
        }
        let img = self
            .0
            .font
            .borrow()
            .render(text, aa, color, bg)
            .unwrap_or_else(|_| Surface::with_flags((0, self.get_height().max(0) as u32), 0));
        let mut c = self.0.cache.borrow_mut();
        if c.len() >= CACHE_MAX {
            c.clear();
        }
        c.insert(key, img.clone());
        img
    }

    /// `font.size(text)`
    pub fn size(&self, text: &str) -> (i32, i32) {
        let (w, h) = self.0.font.borrow().size(text);
        (w as i32, h as i32)
    }

    /// `font.size(text)[0]`
    pub fn width(&self, text: &str) -> i32 {
        self.size(text).0
    }

    /// `font.get_height()`
    pub fn get_height(&self) -> i32 {
        self.0.font.borrow().get_height()
    }

    /// `font.get_linesize()`
    pub fn get_linesize(&self) -> i32 {
        self.0.font.borrow().get_linesize()
    }

    /// `font.get_ascent()`
    pub fn get_ascent(&self) -> i32 {
        self.0.font.borrow().get_ascent()
    }

    /// `font.get_descent()`
    pub fn get_descent(&self) -> i32 {
        self.0.font.borrow().get_descent()
    }

    /// `font.set_bold(b)` - leert den Zwischenspeicher.
    pub fn set_bold(&self, b: bool) {
        self.0.font.borrow_mut().set_bold(b);
        self.0.cache.borrow_mut().clear();
    }

    /// `font.set_italic(b)`
    pub fn set_italic(&self, b: bool) {
        self.0.font.borrow_mut().set_italic(b);
        self.0.cache.borrow_mut().clear();
    }

    /// `font.set_underline(b)`
    pub fn set_underline(&self, b: bool) {
        self.0.font.borrow_mut().set_underline(b);
        self.0.cache.borrow_mut().clear();
    }

    /// `font.get_bold()`
    pub fn get_bold(&self) -> bool {
        self.0.font.borrow().get_bold()
    }

    /// Zwischenspeicher leeren (z.B. bei Speicherdruck).
    pub fn clear_cache(&self) {
        self.0.cache.borrow_mut().clear();
    }
}

impl std::fmt::Debug for Font {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Font#{}", self.0.id)
    }
}

/// `pygame.font.init()` - Schriftordner neben der .exe bekanntgeben.
pub fn init_font() {
    rustgame_font::init();
}
