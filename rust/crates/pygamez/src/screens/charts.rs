//! charts.rs
//! =========
//! Kleine, theme-treue Diagramme fuer die Statistik-Screens - uebertragen aus
//! `charts.py`. Alle Funktionen zeichnen in ein Rechteck und geben die
//! Trefferflaechen zurueck, damit der Bildschirm Hover und Tooltips bauen
//! kann.

use std::f64::consts::PI;

use chrono::{Datelike, Duration, NaiveDate};
use pgz_core::i18n::t;
use pgz_engine::pg::{self, draw, rect, Color, Font, Rect, RectExt, SurfExt, Surface};
use pgz_engine::py;
use pgz_engine::ui::{self, mix};

fn fnt(size: i32, bold: bool) -> Font {
    ui::font(size, bold, false)
}

fn grid_color() -> Color {
    let p = ui::pal();
    mix(p.panel, p.border_light, 0.6)
}

const FALLBACK_WEEKDAYS: [&str; 7] = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
const FALLBACK_MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// `name_list(key, expected, fallback)`
pub fn name_list(key: &str, expected: usize, fallback: &[&str]) -> Vec<String> {
    let parts: Vec<String> = t(key).split(',').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect();
    if parts.len() < expected {
        return fallback.iter().map(|s| s.to_string()).collect();
    }
    parts.into_iter().take(expected).collect()
}

/// Wochentage kurz.
pub fn weekday_names() -> Vec<String> {
    name_list("cal.weekdays_short", 7, &FALLBACK_WEEKDAYS)
}

/// Monate kurz.
pub fn month_names() -> Vec<String> {
    name_list("cal.months_short", 12, &FALLBACK_MONTHS)
}

/// `bars(...)` - Balkendiagramm; Rueckgabe `[(rect, index)]`.
#[allow(clippy::too_many_arguments)]
pub fn bars(
    s: &mut Surface,
    r: Rect,
    values: &[f64],
    labels: Option<&[String]>,
    colors: Option<&[Color]>,
    color: Option<Color>,
    horizontal: bool,
    fmt: Option<&dyn Fn(f64) -> String>,
    hover: Option<usize>,
) -> Vec<(Rect, usize)> {
    let p = ui::pal();
    let mut hits = Vec::new();
    if values.is_empty() {
        return hits;
    }
    let base = color.unwrap_or(p.accent);
    let top = values.iter().cloned().fold(f64::MIN, f64::max).max(1e-6);
    let n = values.len() as i32;
    let label_size = 10;
    let f = fnt(label_size, false);
    let val_fnt = fnt(label_size, true);
    if horizontal {
        let mut label_w = 0;
        if let Some(lab) = labels.filter(|l| !l.is_empty()) {
            label_w = (r.w / 3).min(lab.iter().map(|s| f.size(s).0).max().unwrap_or(0) + 8);
        }
        let gap = 4;
        let row_h = 10.max(py::fdiv32(r.h - gap * (n - 1), n));
        for (i, &value) in values.iter().enumerate() {
            let y = r.y + i as i32 * (row_h + gap);
            let col = colors.map(|c| c[i]).unwrap_or(base);
            if let Some(lab) = labels.filter(|l| i < l.len()) {
                let img = f.render(&lab[i], true, if Some(i) == hover { p.text } else { p.text_dim });
                let ir = img.get_rect().midleft_at((r.x, y + row_h / 2));
                s.bl_area(&img, (ir.x, ir.y), (0, 0, label_w - 6, img.h()));
            }
            let bx = r.x + label_w;
            let bw = 0.max(r.right() - bx);
            let track = Rect::new(bx, y, bw, row_h);
            draw::rect_r(s, mix(p.panel, p.bg_top, 0.4), track, 0, row_h / 2);
            let fill = (bw as f64 * (value / top)) as i32;
            if fill > 1 {
                draw::rect_r(s, col, (bx, y, fill, row_h), 0, row_h / 2);
            }
            if let Some(fm) = fmt {
                let img = val_fnt.render(&fm(value), true, p.text_dim);
                s.blr(&img, img.get_rect().midright_at((r.right() - 6, y + row_h / 2)));
            }
            hits.push((track, i));
        }
        return hits;
    }
    let gap = 2.max(8.min(py::fdiv32(r.w, n * 5)));
    let bar_w = 3.max(py::fdiv32(r.w - gap * (n - 1), n));
    let label_h = if labels.is_some_and(|l| !l.is_empty()) { f.get_height() + 2 } else { 0 };
    let top_pad = val_fnt.get_height() + 2;
    let plot_h = 8.max(r.h - label_h - top_pad);
    for (i, &value) in values.iter().enumerate() {
        let x = r.x + i as i32 * (bar_w + gap);
        let h = (plot_h as f64 * (value / top)) as i32;
        let y = r.y + top_pad + (plot_h - h);
        let mut col = colors.map(|c| c[i]).unwrap_or(base);
        if Some(i) == hover {
            col = mix(col, (255, 255, 255), 0.35);
        }
        let cell = Rect::new(x, r.y + top_pad, bar_w, plot_h);
        draw::rect_r(s, mix(p.panel, p.bg_top, 0.35), cell, 0, 3);
        if h > 1 {
            draw::rect_r(s, col, (x, y, bar_w, h), 0, 3);
        }
        if let Some(lab) = labels.filter(|l| i < l.len()) {
            let img = f.render(&lab[i], true, if Some(i) == hover { p.text } else { p.text_faint });
            s.blr(&img, img.get_rect().midtop_at((x + bar_w / 2, r.bottom() - label_h + 1)));
        }
        if let Some(fm) = fmt {
            if Some(i) == hover || n <= 12 {
                let img = val_fnt.render(&fm(value), true, p.text_dim);
                s.blr(&img, img.get_rect().midbottom_at((x + bar_w / 2, y - 1)));
            }
        }
        hits.push((cell, i));
    }
    hits
}

/// `line(...)` - Linienverlauf; Rueckgabe `[(rect, index)]`.
pub fn line(s: &mut Surface, r: Rect, values: &[f64], color: Option<Color>, marks: &[usize], fmt: Option<&dyn Fn(f64) -> String>, hover: Option<usize>) -> Vec<(Rect, usize)> {
    let p = ui::pal();
    let mut hits = Vec::new();
    if values.is_empty() {
        return hits;
    }
    let col = color.unwrap_or(p.accent);
    let lo = values.iter().cloned().fold(f64::MAX, f64::min);
    let mut hi = values.iter().cloned().fold(f64::MIN, f64::max);
    if hi <= lo {
        hi = lo + 1.0;
    }
    let n = values.len();
    let f = fnt(10, false);
    for frac in [0.0, 0.5, 1.0] {
        let y = (r.bottom() as f64 - frac * r.h as f64) as i32;
        draw::line(s, grid_color(), (r.x, y), (r.right(), y), 1);
        if let Some(fm) = fmt {
            let img = f.render(&fm(lo + (hi - lo) * frac), true, p.text_faint);
            s.blr(&img, img.get_rect().bottomleft_at((r.x + 2, y - 1)));
        }
    }
    let step = if n > 1 { r.w as f64 / (1.max(n - 1)) as f64 } else { 0.0 };
    let mut points: Vec<(i32, i32)> = Vec::new();
    for (i, &value) in values.iter().enumerate() {
        let x = r.x as f64 + if n > 1 { i as f64 * step } else { r.w as f64 / 2.0 };
        let y = r.bottom() as f64 - (value - lo) / (hi - lo) * r.h as f64;
        points.push((x as i32, y as i32));
    }
    if points.len() > 1 {
        let mut poly = points.clone();
        poly.push((points[points.len() - 1].0, r.bottom()));
        poly.push((points[0].0, r.bottom()));
        let mut shade = pg::surface_alpha(r.w + 2, r.h + 2);
        let rel: Vec<(i32, i32)> = poly.iter().map(|&(px, pyy)| (px - r.x, pyy - r.y)).collect();
        draw::polygon(&mut shade, pg::alpha(col, 46), &rel, 0);
        s.bl(&shade, (r.x, r.y));
        draw::lines(s, col, false, &points, 2);
    }
    for (i, &(x, y)) in points.iter().enumerate() {
        if marks.contains(&i) {
            draw::circle(s, p.gold, (x, y), 4, 0);
            draw::circle(s, mix(p.gold, (255, 255, 255), 0.5), (x, y), 2, 0);
        } else if n <= 60 {
            draw::circle(s, col, (x, y), 2, 0);
        }
        if Some(i) == hover {
            draw::circle(s, p.text, (x, y), 5, 1);
        }
        let w = if step as i32 != 0 { step as i32 } else { 4 };
        let cell = Rect::new((x as f64 - 2f64.max(step / 2.0)) as i32, r.y, 4.max(w), r.h);
        hits.push((cell, i));
    }
    hits
}

/// `calendar_heat(...)` - Spieltage-Karte; Rueckgabe `[(rect, (datum, wert))]`.
pub fn calendar_heat(s: &mut Surface, r: Rect, days: &[(String, f64)], color: Option<Color>, hover: Option<&str>) -> Vec<(Rect, (String, f64))> {
    let p = ui::pal();
    let mut hits = Vec::new();
    let col = color.unwrap_or(p.accent);
    let today = chrono::Local::now().date_naive();
    let f = fnt(9, false);
    let label_h = f.get_height() + 2;
    let day_label_w = f.size("Mo").0 + 4;
    let grid = Rect::new(r.x + day_label_w, r.y + label_h, r.w - day_label_w, r.h - label_h);
    let cell = 6.max(16.min(py::fdiv32(grid.h, 7) - 2));
    let gap = 1.max(cell / 6);
    let per_week = cell + gap;
    let fit = 1.max(py::fdiv32(grid.w + gap, per_week));
    let weeks = fit;
    let end = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    let start = end - Duration::weeks((weeks - 1) as i64);
    let top = if days.is_empty() { 1.0 } else { days.iter().map(|d| d.1).fold(f64::MIN, f64::max) }.max(1.0);
    let mut month_seen = std::collections::BTreeSet::new();
    for w in 0..weeks {
        let week_start = start + Duration::weeks(w as i64);
        let x = grid.x + w * per_week;
        for d in 0..7 {
            let date = week_start + Duration::days(d);
            if date > today {
                continue;
            }
            let key = date.format("%Y-%m-%d").to_string();
            let value = days.iter().find(|e| e.0 == key).map(|e| e.1).unwrap_or(0.0);
            let y = grid.y + d as i32 * per_week;
            let bx = Rect::new(x, y, cell, cell);
            if value > 0.0 {
                let frac = 1f64.min(py::pow(value / top, 0.45));
                let fill_col = mix(mix(p.panel, col, 0.25), col, frac);
                draw::rect_r(s, fill_col, bx, 0, 2);
            } else {
                draw::rect_r(s, mix(p.panel, p.bg_top, 0.35), bx, 0, 2);
            }
            if date == today {
                draw::rect_r(s, p.gold, bx, 1, 2);
            }
            if hover == Some(key.as_str()) {
                draw::rect_r(s, p.text, bx, 1, 2);
            }
            hits.push((bx, (key, value)));
        }
        let m = week_start.month();
        if !month_seen.contains(&m) {
            month_seen.insert(m);
            if w == 0 || week_start.day() <= 7 {
                let img = f.render(&month_short(m), true, p.text_faint);
                s.bl(&img, (x, r.y));
            }
        }
    }
    let short = weekday_names();
    for d in [0usize, 2, 4] {
        if d < short.len() {
            let img = f.render(&short[d], true, p.text_faint);
            s.bl(&img, (r.x, grid.y + d as i32 * per_week - 1));
        }
    }
    hits
}

fn month_short(month: u32) -> String {
    let names = month_names();
    if (1..=names.len() as u32).contains(&month) {
        names[month as usize - 1].clone()
    } else {
        month.to_string()
    }
}

/// `heat_legend(...)` - wenig -> viel.
pub fn heat_legend(s: &mut Surface, r: Rect, color: Option<Color>, label_less: &str, label_more: &str) {
    let p = ui::pal();
    let col = color.unwrap_or(p.accent);
    let f = fnt(9, false);
    let mut x = r.x;
    if !label_less.is_empty() {
        let img = f.render(label_less, true, p.text_faint);
        s.blr(&img, img.get_rect().midleft_at((x, r.centery())));
        x += img.w() + 6;
    }
    let size = 10.min(r.h);
    for frac in [0.0, 0.33, 0.66, 1.0] {
        let bx = Rect::new(x, r.centery() - size / 2, size, size);
        if frac <= 0.0 {
            draw::rect_r(s, mix(p.panel, p.bg_top, 0.35), bx, 0, 2);
        } else {
            draw::rect_r(s, mix(mix(p.panel, col, 0.25), col, frac), bx, 0, 2);
        }
        x += size + 3;
    }
    if !label_more.is_empty() {
        let img = f.render(label_more, true, p.text_faint);
        s.blr(&img, img.get_rect().midleft_at((x + 3, r.centery())));
    }
}

/// `donut(...)` - Ringdiagramm.
pub fn donut(s: &mut Surface, center: (i32, i32), radius: i32, slices: &[(f64, Color)], center_text: Option<&str>, center_sub: Option<&str>) {
    let p = ui::pal();
    let (cx, cy) = center;
    let r = radius;
    let thickness = 6.max(r / 3);
    let total: f64 = slices.iter().map(|(v, _)| v.max(0.0)).sum();
    let bx = Rect::new(cx - r, cy - r, r * 2, r * 2);
    draw::circle(s, mix(p.panel, p.bg_top, 0.35), (cx, cy), r, thickness);
    if total > 0.0 {
        let mut start = -PI / 2.0;
        for &(value, col) in slices {
            if value <= 0.0 {
                continue;
            }
            let end = start + 2.0 * PI * (value / total);
            draw::arc(s, col, bx, -end, -start, thickness);
            start = end;
        }
    }
    if let Some(ct) = center_text {
        let img = fnt(12.max(r / 3), true).render(ct, true, p.text);
        s.blr(&img, img.get_rect().center_at((cx, cy - if center_sub.is_some() { 3 } else { 0 })));
        if let Some(cs) = center_sub {
            let sub = fnt(9.max(r / 5), false).render(cs, true, p.text_dim);
            s.blr(&sub, sub.get_rect().center_at((cx, cy + img.h() / 2 + 4)));
        }
    }
}

/// `clock_ring(...)` - Tageszeit-Profil; Rueckgabe `[(rect, stunde)]`.
pub fn clock_ring(s: &mut Surface, center: (i32, i32), radius: i32, hours: &[f64], color: Option<Color>, hover: Option<usize>) -> Vec<(Rect, usize)> {
    let p = ui::pal();
    let (cx, cy) = center;
    let r = radius;
    let col = color.unwrap_or(p.accent);
    let top = hours.iter().cloned().fold(0.0f64, |a, b| if b > a { b } else { a });
    let top = if hours.is_empty() { 0.0 } else { top.max(hours.iter().cloned().fold(f64::MIN, f64::max)) };
    let mut hits = Vec::new();
    let inner = 8.max(r / 3);
    draw::circle(s, grid_color(), (cx, cy), r, 1);
    draw::circle(s, grid_color(), (cx, cy), inner, 1);
    let f = fnt(9, false);
    let (cxf, cyf) = (cx as f64, cy as f64);
    for h in 0..24usize {
        let ang = -PI / 2.0 + h as f64 * PI / 12.0;
        let frac = if top > 0.0 { hours[h] / top } else { 0.0 };
        let length = inner as f64 + (r - inner) as f64 * frac;
        let (x1, y1) = (cxf + inner as f64 * ang.cos(), cyf + inner as f64 * ang.sin());
        let (x2, y2) = (cxf + length * ang.cos(), cyf + length * ang.sin());
        let spoke = if Some(h) != hover { col } else { mix(col, (255, 255, 255), 0.4) };
        if frac > 0.0 {
            draw::line(s, spoke, (x1, y1), (x2, y2), 5);
        }
        let bx = rect(0, 0, 14, 14).center_at((cxf + (r - 6) as f64 * ang.cos(), cyf + (r - 6) as f64 * ang.sin()));
        hits.push((bx, h));
    }
    for h in [0, 6, 12, 18] {
        let ang = -PI / 2.0 + h as f64 * PI / 12.0;
        let img = f.render(&h.to_string(), true, p.text_faint);
        s.blr(&img, img.get_rect().center_at((cxf + (r + 9) as f64 * ang.cos(), cyf + (r + 9) as f64 * ang.sin())));
    }
    hits
}

/// `tooltip(...)` - Hinweiskasten neben der Maus (erste Zeile fett).
pub fn tooltip(s: &mut Surface, pos: (i32, i32), lines: &[String], width_limit: Option<i32>) {
    let p = ui::pal();
    let lines: Vec<&String> = lines.iter().filter(|l| !l.is_empty()).collect();
    if lines.is_empty() {
        return;
    }
    let head = fnt(11, true);
    let body = fnt(10, false);
    let mut imgs = vec![head.render(lines[0], true, p.text)];
    for text in &lines[1..] {
        imgs.push(body.render(text, true, p.text_dim));
    }
    let w = imgs.iter().map(|i| i.w()).max().unwrap_or(0) + 16;
    let h: i32 = imgs.iter().map(|i| i.h()).sum::<i32>() + 12;
    let limit = width_limit.unwrap_or(s.w());
    let x = (pos.0 + 14).min(limit - w - 4);
    let y = 4.max(pos.1 - h - 10);
    let r = Rect::new(4.max(x), y, w, h);
    ui::draw_panel_ex(s, r, Some(p.panel_light), None, Some(8), false, None);
    let mut ty = r.y + 6;
    for img in &imgs {
        s.bl(img, (r.x + 8, ty));
        ty += img.h();
    }
}

/// Tag aus "JJJJ-MM-TT" (fuer `_pretty_date`).
pub fn parse_day(day: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(day.get(..10).unwrap_or(day), "%Y-%m-%d").ok()
}
