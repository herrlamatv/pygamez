//! achv.rs
//! =======
//! Der zeichnende Teil von `achievements.py`: die Einblendung oben rechts
//! (`draw_toasts`), das runde Abzeichen (`draw_badge`) und seine 22
//! Symbole (`_draw_icon`).
//!
//! Die Erfolgs-*Logik* liegt grafikfrei in `pgz_core::achievements`; sie
//! meldet Freischaltungen ueber `take_toasts()`, und [`push_toasts`] reiht
//! sie hier ein.

use std::cell::RefCell;

use pgz_core::achievements::{self, Achievement, Toast};
use pgz_core::i18n::{t, tf};
use pgz_core::settings::Settings;

use crate::pg::{self, draw, Color, IntoRect, Rect, SurfExt, Surface};
use crate::ui::{self, mix};
use crate::{audio, py};

const TOAST_IN: f64 = 0.35;
const TOAST_HOLD: f64 = 3.4;
const TOAST_OUT: f64 = 0.45;
const ARPEGGIO: &[(f64, f64)] = &[(0.00, 660.0), (0.13, 880.0), (0.26, 1318.0)];
const FANFARE: &[(f64, f64)] =
    &[(0.00, 523.0), (0.12, 659.0), (0.24, 784.0), (0.36, 1047.0), (0.52, 1319.0)];

enum Art {
    Ach(&'static Achievement),
    Level(i64, String, Color),
}

struct Eintrag {
    art: Art,
    t: f64,
    notes: Vec<usize>,
}

thread_local! {
    static TOASTS: RefCell<Vec<Eintrag>> = const { RefCell::new(Vec::new()) };
}

/// Leert die Einblendungen (Python: `_toasts.clear()` beim Profilwechsel).
pub fn clear_toasts() {
    TOASTS.with(|t| t.borrow_mut().clear());
}

/// Meldet [`clear_toasts`] beim Profilwechsel an - und das Konfetti seltener
/// Erfolge (Python `_unlock`: sofort beim Freischalten, nicht erst hier).
pub fn register_switch_hook() {
    pgz_core::store::on_switch(clear_toasts);
    achievements::on_rare_unlock(rare_confetti);
}

/// Seltene Stuecke (legendaer, mythisch) bekommen wie in Python Konfetti.
fn rare_confetti() {
    ui::spawn_confetti(800.0, 600.0, 60);
}

/// Holt neue Freischaltungen aus `pgz_core` und reiht sie ein.
pub fn push_toasts() {
    for toast in achievements::take_toasts() {
        let art = match toast {
            Toast::Achievement(d) => Art::Ach(d),
            Toast::Level(level, titel) => {
                Art::Level(level, titel, ui::c3(pgz_core::levels::rank_color(level)))
            }
        };
        TOASTS.with(|t| t.borrow_mut().push(Eintrag { art, t: 0.0, notes: Vec::new() }));
    }
}

fn ease_out(x: f64) -> f64 {
    let x = x.clamp(0.0, 1.0);
    1.0 - crate::py::pow(1.0 - x, 3.0)
}

/// `achievements.draw_toasts(surface, w, h, dt, settings)`
pub fn draw_toasts(surface: &mut Surface, w: i32, _h: i32, dt: f64, settings: &Settings) {
    let mut fertig = false;
    let mut daten = None;
    TOASTS.with(|liste| {
        let mut liste = liste.borrow_mut();
        let Some(toast) = liste.first_mut() else { return };
        toast.t += dt;
        let tt = toast.t;
        let level_toast = matches!(toast.art, Art::Level(..));
        let notes = if level_toast { FANFARE } else { ARPEGGIO };
        for (i, (at, freq)) in notes.iter().enumerate() {
            if tt >= *at && !toast.notes.contains(&i) {
                toast.notes.push(i);
                audio::tone(*freq, if level_toast { 0.24 } else { 0.22 }, Some(settings), "square", 0.30);
            }
        }
        let hold = TOAST_HOLD + if level_toast { 0.8 } else { 0.0 };
        let total = TOAST_IN + hold + TOAST_OUT;
        if tt >= total {
            fertig = true;
            return;
        }
        let (accent, head, name, icon, sub) = match &toast.art {
            Art::Level(level, titel, farbe) => (
                *farbe,
                t("level.up"),
                tf("level.toast", &[("n", &level.to_string()), ("rank", titel)]),
                "bolt",
                String::new(),
            ),
            Art::Ach(d) => (
                ui::c3(d.rarity_color()),
                t("ach.unlocked"),
                d.display_name(),
                d.icon.as_str(),
                format!("{}  \u{b7}  +{}", d.rarity_name(), d.points()),
            ),
        };
        daten = Some((tt, hold, level_toast, accent, head, name, icon, sub));
    });
    if fertig {
        TOASTS.with(|l| {
            let mut l = l.borrow_mut();
            if !l.is_empty() {
                l.remove(0);
            }
        });
        return;
    }
    let Some((tt, hold, level_toast, accent, head_text, name_text, icon, sub_text)) = daten else {
        return;
    };
    let p = ui::pal();
    let head = ui::fb(12).render(&head_text, true, if level_toast { accent } else { p.gold });
    let name = ui::fb(17).render(&name_text, true, p.text);
    let sub = if sub_text.is_empty() { None } else { Some(ui::fb(11).render(&sub_text, true, accent)) };

    let br = 17;
    let (pad, gap) = (14, 10);
    let text_w = head.w().max(name.w()).max(sub.as_ref().map(|s| s.w()).unwrap_or(0));
    let bw = (w - 24).min(pad * 2 + br * 2 + gap + text_w);
    let text_h = head.h() + name.h() + sub.as_ref().map(|s| s.h() + 2).unwrap_or(0);
    let bh = (br * 2 + 16).max(text_h + 22);

    let off = if tt < TOAST_IN {
        (1.0 - ease_out(tt / TOAST_IN)) * (bw + 24) as f64
    } else if tt > TOAST_IN + hold {
        ease_out((tt - TOAST_IN - hold) / TOAST_OUT) * (bw + 24) as f64
    } else {
        0.0
    };
    let rect = Rect::new(((w - 12 - bw) as f64 + off) as i32, 12, bw, bh);
    ui::draw_panel_ex(surface, rect, Some(p.panel_light), None, Some(12), true, Some(accent));
    let (cx, cy) = (rect.x + pad + br, rect.centery());
    let mut glow = pg::surface_alpha(br * 4, br * 4);
    draw::circle(&mut glow, pg::alpha(accent, (50.0 * ui::pulse(3.0, 0.35, 1.0)) as i32), (br * 2, br * 2), br * 2, 0);
    surface.bl(&glow, (cx - br * 2, cy - br * 2));
    draw_badge(surface, (cx, cy), br, icon, accent, false);

    let tx = rect.x + pad + br * 2 + gap;
    let mut ty = rect.centery() - py::fdiv32(text_h, 2);
    surface.bl(&head, (tx, ty));
    ty += head.h() + 2;
    surface.bl(&name, (tx, ty));
    if let Some(s) = sub {
        surface.bl(&s, (tx, ty + name.h()));
    }
}

/// Laeuft gerade eine Einblendung?
pub fn toasts_active() -> bool {
    TOASTS.with(|t| !t.borrow().is_empty())
}

/// `achievements.draw_badge(surface, center, radius, icon, color, locked)`
pub fn draw_badge(surface: &mut Surface, center: (i32, i32), radius: i32, icon: &str, color: Color, locked: bool) {
    let p = ui::pal();
    let (cx, cy) = center;
    let r = radius;
    let (fill, ring, icon_col, icon) = if locked {
        (mix(p.panel, p.panel_light, 0.5), p.border_light, p.text_faint, "lock")
    } else {
        (mix(p.panel_light, color, 0.28), color, mix(color, Color::rgb(255, 255, 255), 0.35), icon)
    };
    draw::circle(surface, fill, (cx, cy), r, 0);
    draw::circle(surface, ring, (cx, cy), r, 2);
    draw_icon(surface, cx, cy, 4.max((r as f64 * 0.62) as i32), icon, icon_col);
}

fn fd(a: i32, b: i32) -> i32 {
    py::fdiv32(a, b)
}

fn draw_icon(s: &mut Surface, cx: i32, cy: i32, r: i32, icon: &str, col: Color) {
    let rf = r as f64;
    let (cxf, cyf) = (cx as f64, cy as f64);
    match icon {
        "star" => {
            let mut pts = Vec::new();
            for i in 0..10 {
                let rad = if i % 2 == 0 { rf } else { rf * 0.45 };
                let a = -py::PI / 2.0 + i as f64 * py::PI / 5.0;
                pts.push((cxf + rad * a.cos(), cyf + rad * a.sin()));
            }
            draw::polygon(s, col, &pts, 0);
        }
        "trophy" => {
            let (cw, chh) = ((rf * 1.2) as i32, (rf * 0.9) as i32);
            let cup = Rect::new(cx - fd(cw, 2), cy - r + 1, cw, chh);
            draw::rect_r(s, col, cup, 0, 3);
            for side in [-1, 1] {
                let hx = cx + side * (fd(cw, 2) + 2);
                draw::circle(s, col, (hx, cup.centery() - 1), 3, 1);
            }
            draw::rect(s, col, (cx - 2, cup.bottom(), 4, 2.max(fd(r, 3))), 0);
            draw::rect_r(s, col, (cx - fd(r, 2) - 1, cy + r - 3.max(fd(r, 3)), r + 2, 3.max(fd(r, 3))), 0, 1);
        }
        "clock" => {
            draw::circle(s, col, (cx, cy), r, 2);
            draw::line(s, col, (cx, cy), (cx, cy - r + 3), 2);
            draw::line(s, col, (cx, cy), (cx + r - 4, cy + 2), 2);
        }
        "medal" => {
            draw::polygon(s, col, &[(cx - r + 2, cy - r), (cx - 1, cy), (cx - r + 5, cy)], 0);
            draw::polygon(s, col, &[(cx + r - 2, cy - r), (cx + 1, cy), (cx + r - 5, cy)], 0);
            draw::circle(s, col, (cx, cy + fd(r, 3)), 3.max((rf * 0.55) as i32), 0);
        }
        "flag" => {
            draw::line(s, col, (cx - r + 3, cy - r), (cx - r + 3, cy + r), 2);
            draw::polygon(s, col, &[(cx - r + 4, cy - r), (cx + r - 1, cy - fd(r, 2)), (cx - r + 4, cy)], 0);
        }
        "crown" => {
            let base = cy + fd(r, 2);
            draw::polygon(
                s,
                col,
                &[
                    (cx - r, base),
                    (cx - r, cy - fd(r, 3)),
                    (cx - fd(r, 2), cy),
                    (cx, cy - r),
                    (cx + fd(r, 2), cy),
                    (cx + r, cy - fd(r, 3)),
                    (cx + r, base),
                ],
                0,
            );
        }
        "gem" => {
            draw::polygon(s, col, &[(cx, cy - r), (cx + r, cy), (cx, cy + r), (cx - r, cy)], 0);
            draw::polygon(
                s,
                mix(col, Color::rgb(255, 255, 255), 0.4),
                &[(cx, cy - r), (cx + fd(r, 2), cy - fd(r, 2)), (cx, cy), (cx - fd(r, 2), cy - fd(r, 2))],
                0,
            );
        }
        "heart" => {
            draw::circle(s, col, (cx - fd(r, 2), cy - fd(r, 3)), fd(r, 2) + 1, 0);
            draw::circle(s, col, (cx + fd(r, 2), cy - fd(r, 3)), fd(r, 2) + 1, 0);
            draw::polygon(s, col, &[(cx - r, cy - fd(r, 4)), (cx + r, cy - fd(r, 4)), (cx, cy + r)], 0);
        }
        "check" => {
            draw::lines(s, col, false, &[(cx - r + 2, cy), (cx - fd(r, 4), cy + r - 3), (cx + r - 1, cy - r + 3)], 3);
        }
        "llama" => {
            draw::rect_r(s, col, (cx - fd(r, 3), cy - fd(r, 4), 3.max(fd(r, 2)), r + fd(r, 3)), 0, 2);
            let head = Rect::new(cx - r, cy - r, (rf * 1.1) as i32, (rf * 0.8) as i32);
            draw::rect_r(s, col, head, 0, 3);
            for dx in [-r + 2, fd(-r, 3)] {
                draw::polygon(s, col, &[(cx + dx, cy - r), (cx + dx + 3, cy - r - fd(r, 2)), (cx + dx + 5, cy - r)], 0);
            }
            draw::circle(s, mix(col, Color::rgb(0, 0, 0), 0.6), (head.centerx() - 1, head.centery()), 1.max(fd(r, 6)), 0);
        }
        "person" => {
            draw::circle(s, col, (cx, cy - fd(r, 2)), 3.max(fd(r, 2)), 0);
            draw::polygon(s, col, &[(cx - r, cy + r), (cx - r + 2, cy + 1), (cx + r - 2, cy + 1), (cx + r, cy + r)], 0);
        }
        "flame" => {
            draw::polygon(
                s,
                col,
                &[(cx, cy - r), (cx + r, cy + fd(r, 3)), (cx + fd(r, 2), cy + r), (cx - fd(r, 2), cy + r), (cx - r, cy + fd(r, 3))],
                0,
            );
            draw::polygon(
                s,
                mix(col, Color::rgb(255, 255, 255), 0.45),
                &[(cx, cy - fd(r, 4)), (cx + fd(r, 2), cy + fd(r, 2)), (cx - fd(r, 2), cy + fd(r, 2))],
                0,
            );
        }
        "calendar" => {
            let bx = Rect::new(cx - r, cy - r + 2, 2 * r, 2 * r - 2);
            draw::rect_r(s, col, bx, 2, 2);
            draw::line(s, col, (bx.x, bx.y + 5), (bx.right(), bx.y + 5), 2);
            for i in 0..2 {
                draw::line(s, col, (cx - fd(r, 2) + i * r, cy - r), (cx - fd(r, 2) + i * r, cy - r + 4), 2);
            }
            draw::rect(s, col, (cx - 1, cy + 1, 3, 3), 0);
        }
        "dice" => {
            let bx = Rect::new(cx - r, cy - r, 2 * r, 2 * r);
            draw::rect_r(s, col, bx, 2, 3);
            for (dx, dy) in [(fd(-r, 2), fd(-r, 2)), (fd(r, 2), fd(r, 2)), (0, 0)] {
                draw::circle(s, col, (cx + dx, cy + dy), 1.max(fd(r, 5)), 0);
            }
        }
        "bolt" => {
            draw::polygon(
                s,
                col,
                &[
                    (cx + fd(r, 3), cy - r),
                    (cx - r, cy + 1),
                    (cx, cy + 1),
                    (cx - fd(r, 3), cy + r),
                    (cx + r, cy - 1),
                    (cx, cy - 1),
                ],
                0,
            );
        }
        "moon" => {
            draw::circle(s, col, (cx, cy), r, 0);
            draw::circle(s, ui::panel_light(), (cx + fd(r, 2), cy - fd(r, 3)), r, 0);
        }
        "target" => {
            draw::circle(s, col, (cx, cy), r, 2);
            draw::circle(s, col, (cx, cy), 2.max(fd(r, 2)), 2);
            draw::circle(s, col, (cx, cy), 1.max(fd(r, 5)), 0);
        }
        "swords" => {
            draw::line(s, col, (cx - r, cy + r), (cx + r, cy - r), 2);
            draw::line(s, col, (cx + r, cy + r), (cx - r, cy - r), 2);
            draw::circle(s, col, (cx - r, cy + r), 2.max(fd(r, 4)), 0);
            draw::circle(s, col, (cx + r, cy + r), 2.max(fd(r, 4)), 0);
        }
        "chart" => {
            let base = cy + r;
            for (i, height) in [0.45, 0.8, 0.6, 1.0].iter().enumerate() {
                let bx = cx - r + i as i32 * fd(2 * r, 4) + 1;
                let bh = (2.0 * rf * height * 0.75) as i32;
                draw::rect_r(s, col, (bx, base - bh, 2.max(fd(r, 3)), bh), 0, 1);
            }
        }
        "rocket" => {
            draw::polygon(
                s,
                col,
                &[(cx, cy - r), (cx + fd(r, 2), cy), (cx + fd(r, 2), cy + fd(r, 2)), (cx - fd(r, 2), cy + fd(r, 2)), (cx - fd(r, 2), cy)],
                0,
            );
            draw::polygon(s, col, &[(cx - fd(r, 2), cy), (cx - r, cy + fd(r, 2)), (cx - fd(r, 2), cy + fd(r, 2))], 0);
            draw::polygon(s, col, &[(cx + fd(r, 2), cy), (cx + r, cy + fd(r, 2)), (cx + fd(r, 2), cy + fd(r, 2))], 0);
            draw::circle(s, mix(col, Color::rgb(0, 0, 0), 0.5), (cx, cy - fd(r, 4)), 1.max(fd(r, 4)), 0);
        }
        _ => {
            let body = Rect::new(cx - r + 2, cy - 1, 2 * r - 4, r + 1);
            draw::rect_r(s, col, body, 0, 2);
            draw::circle(s, col, (cx, cy - fd(r, 3) - 1), fd(r, 2) + 1, 2);
        }
    }
}

/// Hilfsform fuer Bildschirme: Abzeichen mit Rechteck-Mitte.
pub fn badge_in(surface: &mut Surface, r: impl IntoRect, icon: &str, color: Color, locked: bool) {
    let r = r.rect();
    draw_badge(surface, r.center(), r.w.min(r.h) / 2, icon, color, locked);
}
