//! `pygame.gfxdraw` - die Grundformen aus SDL_gfx.
//!
//! Uebertragen aus `src_c/SDL_gfx/SDL_gfxPrimitives.c` (pygame-ce 2.5.7),
//! nicht aus Messreihen rekonstruiert. Das ist wichtiger, als es klingt:
//! die Formen hier sind **nicht** dieselben wie in `pygame.draw`. Ein
//! `gfxdraw.circle` und ein `draw.circle` mit demselben Radius zeichnen
//! verschiedene Pixel, weil zwei verschiedene Rasterer dahinterstehen.
//!
//! ## Die Mischung
//!
//! Die Farbe kommt als `0xRRGGBBAA` herein, und das **niederste Byte ist
//! das Alpha**. Bei `255` wird geschrieben, sonst gemischt:
//!
//! ```c
//! R = ((dc & Rmask) + ((((color & Rmask) - (dc & Rmask)) >> Rshift) * alpha >> 8) << Rshift) & Rmask
//! ```
//!
//! Die Rechnung laeuft in C auf **vorzeichenlosen** Zahlen und laeuft bei
//! `s < d` absichtlich ueber; in Zweierkomplement-Arithmetik kommt
//! dasselbe heraus wie mit Vorzeichen, deshalb steht hier die lesbare
//! Form `d + ((s - d) * a >> 8)`.
//!
//! ## Die Gewichtung
//!
//! Fuer die kantengeglaetteten Formen gibt es `pixelColorWeight`: es
//! **multipliziert** das Gewicht ins Alpha, statt es zu ersetzen -
//! `a = (a * weight) >> 8`. Eine Farbe mit Alpha 128 und Gewicht 128
//! zeichnet also mit 64, nicht mit 128.

use crate::color::Color;
use crate::surface::Surface;

/// Ein Pixel mit Mischung - `_putPixelAlpha`.
pub fn pixel(s: &mut Surface, x: i32, y: i32, c: Color) {
    if !s.get_clip().collidepoint((x, y)) {
        return;
    }
    if c.a == 255 {
        s.set_at((x, y), c);
        return;
    }
    let d = s.get_at((x, y));
    let misch = |dk: u8, sk: u8| -> u8 {
        (dk as i32 + (((sk as i32 - dk as i32) * c.a as i32) >> 8)) as u8
    };
    let a = if s.has_alpha() {
        misch(d.a, c.a)
    } else {
        255
    };
    s.set_at((x, y), Color::rgba(misch(d.r, c.r), misch(d.g, c.g), misch(d.b, c.b), a));
}

/// Ein Pixel mit zusaetzlichem Gewicht - `pixelColorWeight`.
///
/// Das Gewicht geht **ins Alpha**, es ersetzt es nicht.
pub fn pixel_gewichtet(s: &mut Surface, x: i32, y: i32, c: Color, gewicht: u32) {
    let a = ((c.a as u32 * gewicht) >> 8).min(255) as u8;
    pixel(s, x, y, Color::rgba(c.r, c.g, c.b, a));
}

/// Eine waagerechte Linie - `hlineColor`.
pub fn hline(s: &mut Surface, x1: i32, x2: i32, y: i32, c: Color) {
    let (a, b) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
    for x in a..=b {
        pixel(s, x, y, c);
    }
}

/// Eine senkrechte Linie - `vlineColor`.
pub fn vline(s: &mut Surface, x: i32, y1: i32, y2: i32, c: Color) {
    let (a, b) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
    for y in a..=b {
        pixel(s, x, y, c);
    }
}

/// Ein Rechteckumriss - `rectangleColor`.
pub fn rectangle(s: &mut Surface, x1: i32, y1: i32, x2: i32, y2: i32, c: Color) {
    let (l, r) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
    let (o, u) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
    hline(s, l, r, o, c);
    if u != o {
        hline(s, l, r, u, c);
    }
    if u - o > 1 {
        vline(s, l, o + 1, u - 1, c);
        if r != l {
            vline(s, r, o + 1, u - 1, c);
        }
    }
}

/// Ein gefuelltes Rechteck - `boxColor`.
pub fn box_(s: &mut Surface, x1: i32, y1: i32, x2: i32, y2: i32, c: Color) {
    let (o, u) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
    for y in o..=u {
        hline(s, x1, x2, y, c);
    }
}

/// Eine Linie - `lineColor`.
///
/// Der Schrittzaehler laeuft ueber die **laengere** Achse; die kuerzere
/// kommt ueber einen aufsummierten Fehler dazu. Zeilengetreu aus der
/// Vorlage, samt des `+ 1` auf beiden Deltas.
pub fn line(s: &mut Surface, x1: i32, y1: i32, x2: i32, y2: i32, c: Color) {
    if x1 == x2 {
        return vline(s, x1, y1, y2, c);
    }
    if y1 == y2 {
        return hline(s, x1, x2, y1, c);
    }
    let sx = if x2 >= x1 { 1 } else { -1 };
    let sy = if y2 >= y1 { 1 } else { -1 };
    let mut dx = (x2 - x1).abs() + 1;
    let mut dy = (y2 - y1).abs() + 1;
    // Der Hauptschritt geht auf der laengeren Achse.
    let (mut haupt, mut neben) = ((sx, 0), (0, sy));
    if dx < dy {
        std::mem::swap(&mut dx, &mut dy);
        std::mem::swap(&mut haupt, &mut neben);
    }
    let (mut x, mut y) = (x1, y1);
    let mut fehler = 0;
    for _ in 0..dx {
        pixel(s, x, y, c);
        fehler += dy;
        if fehler >= dx {
            fehler -= dx;
            x += neben.0;
            y += neben.1;
        }
        x += haupt.0;
        y += haupt.1;
    }
}

/// Eine kantengeglaettete Linie - `_aalineColor`, Wu-Verfahren.
///
/// `draw_endpoint` sagt, ob der letzte Punkt mitgezeichnet wird:
/// `aaline` zeichnet ihn, ein Polygonzug laesst ihn weg, damit die Ecken
/// nicht doppelt gedeckt werden.
pub fn aaline(s: &mut Surface, x1: i32, y1: i32, x2: i32, y2: i32, c: Color, endpunkt: bool) {
    const AABITS: u32 = 8;
    // **Immer von oben nach unten.** Die Vorlage tauscht die beiden
    // Punkte, wenn der erste tiefer liegt - der Fehlerzaehler laeuft
    // sonst rueckwaerts. Der **urspruengliche** Anfangspunkt wird
    // trotzdem gesetzt (`pixelColorNolock(dst, x1, y1)`), also bleiben
    // beide Paare gebraucht.
    let (mut xx0, yy0, xx1, yy1) = if y1 > y2 {
        (x2, y2, x1, y1)
    } else {
        (x1, y1, x2, y2)
    };
    let mut dx = xx1 - xx0;
    let dy = yy1 - yy0;
    if dx == 0 {
        if endpunkt {
            return vline(s, x1, y1, y2, c);
        }
        if dy > 0 {
            return vline(s, x1, yy0, yy0 + dy, c);
        }
        return pixel(s, x1, y1, c);
    }
    if dy == 0 {
        if endpunkt {
            return hline(s, x1, x2, y1, c);
        }
        return hline(s, xx0, xx0 + dx, y1, c);
    }
    if dx == dy && endpunkt {
        return line(s, x1, y1, x2, y2, c);
    }
    let xdir: i32 = if dx >= 0 {
        1
    } else {
        dx = -dx;
        -1
    };
    let mut erracc: u32 = 0;
    let intshift = 32 - AABITS;
    pixel(s, x1, y1, c);
    if dy > dx {
        let erradj = (((dx as u32) << 16) / dy as u32) << 16;
        let mut x0pxdir = xx0 + xdir;
        let mut n = dy;
        let mut yy = yy0;
        loop {
            n -= 1;
            if n == 0 {
                break;
            }
            let vorher = erracc;
            erracc = erracc.wrapping_add(erradj);
            if erracc <= vorher {
                xx0 = x0pxdir;
                x0pxdir += xdir;
            }
            yy += 1;
            let g = (erracc >> intshift) & 255;
            pixel_gewichtet(s, xx0, yy, c, 255 - g);
            pixel_gewichtet(s, x0pxdir, yy, c, g);
        }
    } else {
        let erradj = (((dy as u32) << 16) / dx as u32) << 16;
        let mut y0p1 = yy0 + 1;
        let mut n = dx;
        let mut xx = xx0;
        let mut yy = yy0;
        loop {
            n -= 1;
            if n == 0 {
                break;
            }
            let vorher = erracc;
            erracc = erracc.wrapping_add(erradj);
            if erracc <= vorher {
                yy = y0p1;
                y0p1 += 1;
            }
            xx += xdir;
            let g = (erracc >> intshift) & 255;
            pixel_gewichtet(s, xx, yy, c, 255 - g);
            pixel_gewichtet(s, xx, y0p1, c, g);
        }
    }
    if endpunkt {
        pixel(s, x2, y2, c);
    }
}

/// Ein Kreisumriss - `circleColor`, Mittelpunktverfahren.
pub fn circle(s: &mut Surface, x: i32, y: i32, rad: i32, c: Color) {
    if rad < 0 {
        return;
    }
    if rad == 0 {
        return pixel(s, x, y, c);
    }
    let (mut cx, mut cy) = (0i32, rad);
    let (mut df, mut d_e, mut d_se) = (1 - rad, 3, -2 * rad + 5);
    loop {
        let (ypcy, ymcy) = (y + cy, y - cy);
        if cx > 0 {
            let (xpcx, xmcx) = (x + cx, x - cx);
            pixel(s, xmcx, ypcy, c);
            pixel(s, xpcx, ypcy, c);
            pixel(s, xmcx, ymcy, c);
            pixel(s, xpcx, ymcy, c);
        } else {
            pixel(s, x, ymcy, c);
            pixel(s, x, ypcy, c);
        }
        let (xpcy, xmcy) = (x + cy, x - cy);
        if cx > 0 && cx != cy {
            let (ypcx, ymcx) = (y + cx, y - cx);
            pixel(s, xmcy, ypcx, c);
            pixel(s, xpcy, ypcx, c);
            pixel(s, xmcy, ymcx, c);
            pixel(s, xpcy, ymcx, c);
        } else if cx == 0 {
            pixel(s, xmcy, y, c);
            pixel(s, xpcy, y, c);
        }
        if df < 0 {
            df += d_e;
            d_e += 2;
            d_se += 2;
        } else {
            df += d_se;
            d_e += 2;
            d_se += 4;
            cy -= 1;
        }
        cx += 1;
        if cx > cy {
            break;
        }
    }
}

/// Eine gefuellte Kreisflaeche - `filledCircleColor`.
///
/// Die beiden `ocx`/`ocy`-Merker sind kein Beiwerk: ohne sie wird jede
/// Zeile mehrfach gezeichnet, und bei durchsichtiger Farbe sieht man das
/// sofort.
pub fn filled_circle(s: &mut Surface, x: i32, y: i32, rad: i32, c: Color) {
    if rad < 0 {
        return;
    }
    if rad == 0 {
        return pixel(s, x, y, c);
    }
    let (mut cx, mut cy) = (0i32, rad);
    let (mut ocx, mut ocy) = (i32::MIN, i32::MIN);
    let (mut df, mut d_e, mut d_se) = (1 - rad, 3, -2 * rad + 5);
    loop {
        let (xpcx, xmcx) = (x + cx, x - cx);
        let (xpcy, xmcy) = (x + cy, x - cy);
        if ocy != cy {
            if cy > 0 {
                hline(s, xmcx, xpcx, y + cy, c);
                hline(s, xmcx, xpcx, y - cy, c);
            } else {
                hline(s, xmcx, xpcx, y, c);
            }
            ocy = cy;
        }
        if ocx != cx {
            if cx != cy {
                if cx > 0 {
                    hline(s, xmcy, xpcy, y - cx, c);
                    hline(s, xmcy, xpcy, y + cx, c);
                } else {
                    hline(s, xmcy, xpcy, y, c);
                }
            }
            ocx = cx;
        }
        if df < 0 {
            df += d_e;
            d_e += 2;
            d_se += 2;
        } else {
            df += d_se;
            d_e += 2;
            d_se += 4;
            cy -= 1;
        }
        cx += 1;
        if cx > cy {
            break;
        }
    }
}

/// Ein Kreisbogen - `arcColor`.
///
/// Der Kreis wird in **acht Oktanten** geteilt, und eine Bitmaske sagt,
/// in welchen gezeichnet wird. Die Maske wird waehrend des Laufs
/// umgeschaltet, sobald `cx` den Wert erreicht, an dem der Anfangs- oder
/// Endwinkel liegt - deshalb steht die Umschaltung **am Ende** jedes
/// Durchlaufs und nicht davor.
pub fn arc(s: &mut Surface, x: i32, y: i32, rad: i32, start: i32, ende: i32, c: Color) {
    if rad < 0 {
        return;
    }
    if rad == 0 {
        return pixel(s, x, y, c);
    }
    let mut start = start % 360;
    let mut ende = ende % 360;
    while start < 0 {
        start += 360;
    }
    while ende < 0 {
        ende += 360;
    }
    start %= 360;
    ende %= 360;
    let startoct = start / 45;
    let endoct = ende / 45;
    let mut oct = startoct - 1;
    let mut drawoct: u32 = 0;
    let (mut stop_start, mut stop_end) = (0i32, 0i32);

    // Welcher Wert von `cx` gehoert zu diesem Winkel in diesem Oktanten?
    let grenze = |grad: f64, oct: i32| -> f64 {
        let b = grad * std::f64::consts::PI / 180.0;
        match oct {
            0 | 3 => b.sin(),
            1 | 6 => b.cos(),
            2 | 5 => -b.cos(),
            _ => -b.sin(),
        }
    };

    loop {
        oct = (oct + 1) % 8;
        if oct == startoct {
            stop_start = (grenze(start as f64, oct) * rad as f64) as i32;
            if oct % 2 != 0 {
                drawoct |= 1 << oct;
            } else {
                drawoct &= 255 - (1 << oct);
            }
        }
        if oct == endoct {
            stop_end = (grenze(ende as f64, oct) * rad as f64) as i32;
            if startoct == endoct {
                if start > ende {
                    drawoct = 255;
                } else {
                    drawoct &= 255 - (1 << oct);
                }
            } else if oct % 2 != 0 {
                drawoct &= 255 - (1 << oct);
            } else {
                drawoct |= 1 << oct;
            }
        } else if oct != startoct {
            drawoct |= 1 << oct;
        }
        if oct == endoct {
            break;
        }
    }

    let (mut cx, mut cy) = (0i32, rad);
    let (mut df, mut d_e, mut d_se) = (1 - rad, 3, -2 * rad + 5);
    loop {
        let (ypcy, ymcy) = (y + cy, y - cy);
        if cx > 0 {
            let (xpcx, xmcx) = (x + cx, x - cx);
            if drawoct & 4 != 0 {
                pixel(s, xmcx, ypcy, c);
            }
            if drawoct & 2 != 0 {
                pixel(s, xpcx, ypcy, c);
            }
            if drawoct & 32 != 0 {
                pixel(s, xmcx, ymcy, c);
            }
            if drawoct & 64 != 0 {
                pixel(s, xpcx, ymcy, c);
            }
        } else {
            if drawoct & 6 != 0 {
                pixel(s, x, ypcy, c);
            }
            if drawoct & 96 != 0 {
                pixel(s, x, ymcy, c);
            }
        }
        let (xpcy, xmcy) = (x + cy, x - cy);
        if cx > 0 && cx != cy {
            let (ypcx, ymcx) = (y + cx, y - cx);
            if drawoct & 8 != 0 {
                pixel(s, xmcy, ypcx, c);
            }
            if drawoct & 1 != 0 {
                pixel(s, xpcy, ypcx, c);
            }
            if drawoct & 16 != 0 {
                pixel(s, xmcy, ymcx, c);
            }
            if drawoct & 128 != 0 {
                pixel(s, xpcy, ymcx, c);
            }
        } else if cx == 0 {
            if drawoct & 24 != 0 {
                pixel(s, xmcy, y, c);
            }
            if drawoct & 129 != 0 {
                pixel(s, xpcy, y, c);
            }
        }

        // Erst zeichnen, dann umschalten.
        if stop_start == cx {
            if drawoct & (1 << startoct) != 0 {
                drawoct &= 255 - (1 << startoct);
            } else {
                drawoct |= 1 << startoct;
            }
        }
        if stop_end == cx {
            if drawoct & (1 << endoct) != 0 {
                drawoct &= 255 - (1 << endoct);
            } else {
                drawoct |= 1 << endoct;
            }
        }

        if df < 0 {
            df += d_e;
            d_e += 2;
            d_se += 2;
        } else {
            df += d_se;
            d_e += 2;
            d_se += 4;
            cy -= 1;
        }
        cx += 1;
        if cx > cy {
            break;
        }
    }
}

/// Ein Ellipsenumriss - `ellipseColor`.
pub fn ellipse(s: &mut Surface, x: i32, y: i32, rx: i32, ry: i32, c: Color) {
    if rx < 0 || ry < 0 {
        return;
    }
    if rx == 0 {
        return vline(s, x, y - ry, y + ry, c);
    }
    if ry == 0 {
        return hline(s, x - rx, x + rx, y, c);
    }
    let (mut oh, mut oi, mut oj, mut ok) = (0xFFFFi32, 0xFFFFi32, 0xFFFFi32, 0xFFFFi32);
    if rx > ry {
        let (mut ix, mut iy) = (0i32, rx * 64);
        loop {
            let h = (ix + 32) >> 6;
            let i = (iy + 32) >> 6;
            let j = (h * ry) / rx;
            let k = (i * ry) / rx;
            if ((ok != k) && (oj != k)) || ((oj != j) && (ok != j)) || (k != j) {
                let (xph, xmh) = (x + h, x - h);
                if k > 0 {
                    pixel(s, xmh, y + k, c);
                    pixel(s, xph, y + k, c);
                    pixel(s, xmh, y - k, c);
                    pixel(s, xph, y - k, c);
                } else {
                    pixel(s, xmh, y, c);
                    pixel(s, xph, y, c);
                }
                ok = k;
                let (xpi, xmi) = (x + i, x - i);
                if j > 0 {
                    pixel(s, xmi, y + j, c);
                    pixel(s, xpi, y + j, c);
                    pixel(s, xmi, y - j, c);
                    pixel(s, xpi, y - j, c);
                } else {
                    pixel(s, xmi, y, c);
                    pixel(s, xpi, y, c);
                }
                oj = j;
            }
            ix += iy / rx;
            iy -= ix / rx;
            if i <= h {
                break;
            }
        }
    } else {
        let (mut ix, mut iy) = (0i32, ry * 64);
        loop {
            let h = (ix + 32) >> 6;
            let i = (iy + 32) >> 6;
            let j = (h * rx) / ry;
            let k = (i * rx) / ry;
            if ((oi != i) && (oh != i)) || ((oh != h) && (oi != h) && (i != h)) {
                let (xmj, xpj) = (x - j, x + j);
                if i > 0 {
                    pixel(s, xmj, y + i, c);
                    pixel(s, xpj, y + i, c);
                    pixel(s, xmj, y - i, c);
                    pixel(s, xpj, y - i, c);
                } else {
                    pixel(s, xmj, y, c);
                    pixel(s, xpj, y, c);
                }
                oi = i;
                let (xmk, xpk) = (x - k, x + k);
                if h > 0 {
                    pixel(s, xmk, y + h, c);
                    pixel(s, xpk, y + h, c);
                    pixel(s, xmk, y - h, c);
                    pixel(s, xpk, y - h, c);
                } else {
                    pixel(s, xmk, y, c);
                    pixel(s, xpk, y, c);
                }
                oh = h;
            }
            ix += iy / ry;
            iy -= ix / ry;
            if i <= h {
                break;
            }
        }
    }
}

/// Eine gefuellte Ellipse - `filledEllipseColor`.
pub fn filled_ellipse(s: &mut Surface, x: i32, y: i32, rx: i32, ry: i32, c: Color) {
    if rx < 0 || ry < 0 {
        return;
    }
    if rx == 0 {
        return vline(s, x, y - ry, y + ry, c);
    }
    if ry == 0 {
        return hline(s, x - rx, x + rx, y, c);
    }
    let (mut oh, mut oi, mut oj, mut ok) = (0xFFFFi32, 0xFFFFi32, 0xFFFFi32, 0xFFFFi32);
    if rx > ry {
        let (mut ix, mut iy) = (0i32, rx * 64);
        loop {
            let h = (ix + 32) >> 6;
            let i = (iy + 32) >> 6;
            let j = (h * ry) / rx;
            let k = (i * ry) / rx;
            if (ok != k) && (oj != k) {
                let (xph, xmh) = (x + h, x - h);
                if k > 0 {
                    hline(s, xmh, xph, y + k, c);
                    hline(s, xmh, xph, y - k, c);
                } else {
                    hline(s, xmh, xph, y, c);
                }
                ok = k;
            }
            if (oj != j) && (ok != j) && (k != j) {
                let (xmi, xpi) = (x - i, x + i);
                if j > 0 {
                    hline(s, xmi, xpi, y + j, c);
                    hline(s, xmi, xpi, y - j, c);
                } else {
                    hline(s, xmi, xpi, y, c);
                }
                oj = j;
            }
            ix += iy / rx;
            iy -= ix / rx;
            if i <= h {
                break;
            }
        }
    } else {
        let (mut ix, mut iy) = (0i32, ry * 64);
        loop {
            let h = (ix + 32) >> 6;
            let i = (iy + 32) >> 6;
            let j = (h * rx) / ry;
            let k = (i * rx) / ry;
            if (oi != i) && (oh != i) {
                let (xmj, xpj) = (x - j, x + j);
                if i > 0 {
                    hline(s, xmj, xpj, y + i, c);
                    hline(s, xmj, xpj, y - i, c);
                } else {
                    hline(s, xmj, xpj, y, c);
                }
                oi = i;
            }
            if (oh != h) && (oi != h) && (i != h) {
                let (xmk, xpk) = (x - k, x + k);
                if h > 0 {
                    hline(s, xmk, xpk, y + h, c);
                    hline(s, xmk, xpk, y - h, c);
                } else {
                    hline(s, xmk, xpk, y, c);
                }
                oh = h;
            }
            ix += iy / ry;
            iy -= ix / ry;
            if i <= h {
                break;
            }
        }
    }
}

/// Eine kantengeglaettete Ellipse - `aaellipseColor`.
pub fn aaellipse(s: &mut Surface, x: i32, y: i32, rx: i32, ry: i32, c: Color) {
    if rx < 0 || ry < 0 {
        return;
    }
    if rx == 0 {
        return vline(s, x, y - ry, y + ry, c);
    }
    if ry == 0 {
        return hline(s, x - rx, x + rx, y, c);
    }
    let a2 = rx * rx;
    let b2 = ry * ry;
    let ds = 2 * a2;
    let dt = 2 * b2;
    let xc2 = 2 * x;
    let yc2 = 2 * y;
    let sab = ((a2 + b2) as f64).sqrt();
    let dxt = (a2 as f64 / sab).round() as i32;
    let mut t = 0i32;
    let mut sv = -2 * a2 * ry;
    let mut d = 0i32;
    let mut xp = x;
    let mut yp = y - ry;

    pixel(s, xp, yp, c);
    pixel(s, xc2 - xp, yp, c);
    pixel(s, xp, yc2 - yp, c);
    pixel(s, xc2 - xp, yc2 - yp, c);

    for _ in 1..=dxt {
        xp -= 1;
        d += t - b2;
        let ys;
        if d >= 0 {
            ys = yp - 1;
        } else if (d - sv - a2) > 0 {
            if (2 * d - sv - a2) >= 0 {
                ys = yp + 1;
            } else {
                ys = yp;
                yp += 1;
                d -= sv + a2;
                sv += ds;
            }
        } else {
            yp += 1;
            ys = yp + 1;
            d -= sv + a2;
            sv += ds;
        }
        t -= dt;
        let cp = if sv != 0 {
            ((d.abs() as f32) / (sv.abs() as f32)).min(1.0)
        } else {
            1.0
        };
        let g = (cp * 255.0) as u32;
        let ig = 255 - g;
        let xx = xc2 - xp;
        pixel_gewichtet(s, xp, yp, c, ig);
        pixel_gewichtet(s, xx, yp, c, ig);
        pixel_gewichtet(s, xp, ys, c, g);
        pixel_gewichtet(s, xx, ys, c, g);
        let yy = yc2 - yp;
        pixel_gewichtet(s, xp, yy, c, ig);
        pixel_gewichtet(s, xx, yy, c, ig);
        let yy = yc2 - ys;
        pixel_gewichtet(s, xp, yy, c, g);
        pixel_gewichtet(s, xx, yy, c, g);
    }

    let dyt = (b2 as f64 / sab).round() as i32;
    for _ in 1..=dyt {
        yp += 1;
        d -= sv + a2;
        let xs;
        if d <= 0 {
            xs = xp + 1;
        } else if (d + t - b2) < 0 {
            if (2 * d + t - b2) <= 0 {
                xs = xp - 1;
            } else {
                xs = xp;
                xp -= 1;
                d += t - b2;
                t -= dt;
            }
        } else {
            xp -= 1;
            xs = xp - 1;
            d += t - b2;
            t -= dt;
        }
        sv += ds;
        let cp = if t != 0 {
            ((d.abs() as f32) / (t.abs() as f32)).min(1.0)
        } else {
            1.0
        };
        let g = (cp * 255.0) as u32;
        let ig = 255 - g;
        let xx = xc2 - xp;
        let yy = yc2 - yp;
        pixel_gewichtet(s, xp, yp, c, ig);
        pixel_gewichtet(s, xx, yp, c, ig);
        pixel_gewichtet(s, xp, yy, c, ig);
        pixel_gewichtet(s, xx, yy, c, ig);
        let xx = xc2 - xs;
        pixel_gewichtet(s, xs, yp, c, g);
        pixel_gewichtet(s, xx, yp, c, g);
        pixel_gewichtet(s, xs, yy, c, g);
        pixel_gewichtet(s, xx, yy, c, g);
    }
}

/// Ein kantengeglaetteter Kreis - in SDL_gfx schlicht eine Ellipse.
pub fn aacircle(s: &mut Surface, x: i32, y: i32, rad: i32, c: Color) {
    aaellipse(s, x, y, rad, rad, c)
}

/// Ein gefuelltes Vieleck - `filledPolygonColor`, Abtastzeilenverfahren.
///
/// Die Schnittpunkte werden in **65536stel** gerechnet und erst beim
/// Zeichnen gerundet; das `+ 1` und `- 1` an den Enden ist Absicht und
/// steht so in der Vorlage.
pub fn filled_polygon(s: &mut Surface, punkte: &[(i32, i32)], c: Color) {
    let n = punkte.len();
    if n < 3 {
        return;
    }
    let miny = punkte.iter().map(|p| p.1).min().unwrap();
    let maxy = punkte.iter().map(|p| p.1).max().unwrap();
    let mut schnitte: Vec<i32> = Vec::with_capacity(n);
    for y in miny..=maxy {
        schnitte.clear();
        for i in 0..n {
            let (ind1, ind2) = if i == 0 { (n - 1, 0) } else { (i - 1, i) };
            let (mut y1, mut y2) = (punkte[ind1].1, punkte[ind2].1);
            let (x1, x2);
            if y1 < y2 {
                x1 = punkte[ind1].0;
                x2 = punkte[ind2].0;
            } else if y1 > y2 {
                y2 = punkte[ind1].1;
                y1 = punkte[ind2].1;
                x2 = punkte[ind1].0;
                x1 = punkte[ind2].0;
            } else {
                continue;
            }
            if ((y >= y1) && (y < y2)) || ((y == maxy) && (y > y1) && (y <= y2)) {
                schnitte.push(((65536 * (y - y1)) / (y2 - y1)) * (x2 - x1) + (65536 * x1));
            }
        }
        schnitte.sort_unstable();
        let mut i = 0;
        while i + 1 < schnitte.len() {
            let xa = schnitte[i] + 1;
            let xa = (xa >> 16) + ((xa & 32768) >> 15);
            let xb = schnitte[i + 1] - 1;
            let xb = (xb >> 16) + ((xb & 32768) >> 15);
            hline(s, xa, xb, y, c);
            i += 2;
        }
    }
}

/// Ein Vieleck als Umriss - `polygonColor`.
pub fn polygon(s: &mut Surface, punkte: &[(i32, i32)], c: Color) {
    if punkte.len() < 3 {
        return;
    }
    for i in 0..punkte.len() {
        let a = punkte[i];
        let b = punkte[(i + 1) % punkte.len()];
        line(s, a.0, a.1, b.0, b.1, c);
    }
}

/// Ein kantengeglaetteter Umriss - `aapolygonColor`.
///
/// Der Endpunkt jeder Kante bleibt **weg**: sonst deckt die naechste
/// Kante ihn ein zweites Mal, und bei durchsichtiger Farbe waeren die
/// Ecken dunkler als der Rest.
pub fn aapolygon(s: &mut Surface, punkte: &[(i32, i32)], c: Color) {
    if punkte.len() < 3 {
        return;
    }
    for i in 0..punkte.len() {
        let a = punkte[i];
        let b = punkte[(i + 1) % punkte.len()];
        aaline(s, a.0, a.1, b.0, b.1, c, false);
    }
}

/// Ein Kreisausschnitt - `_pieColor`.
///
/// Die Schrittweite ist `3 / r` im Bogenmass, also grob drei Pixel je
/// Stueck; der Rand wird als Vieleck gezeichnet.
///
/// Acht Argumente sind eins mehr, als clippy gern sieht - aber genau die
/// acht, die `_pieColor` hat. Sie zu buendeln hiesse, die Vorlage beim
/// Vergleichen unleserlich zu machen.
#[allow(clippy::too_many_arguments)]
pub fn pie(s: &mut Surface, x: i32, y: i32, rad: i32, start: i32, ende: i32, c: Color, gefuellt: bool) {
    if rad < 0 {
        return;
    }
    let start = start % 360;
    let ende = ende % 360;
    if rad == 0 {
        return pixel(s, x, y, c);
    }
    let dr = rad as f64;
    let schritt = 3.0 / dr;
    let sw = start as f64 * (2.0 * std::f64::consts::PI / 360.0);
    let mut ew = ende as f64 * (2.0 * std::f64::consts::PI / 360.0);
    if start > ende {
        ew += 2.0 * std::f64::consts::PI;
    }
    let mut anzahl = 2usize;
    let mut winkel = sw;
    while winkel < ew {
        winkel += schritt;
        anzahl += 1;
    }
    let mut punkte: Vec<(i32, i32)> = Vec::with_capacity(anzahl);
    punkte.push((x, y));
    punkte.push((x + (dr * sw.cos()) as i32, y + (dr * sw.sin()) as i32));
    if anzahl < 3 {
        let a = punkte[0];
        let b = punkte[1];
        return line(s, a.0, a.1, b.0, b.1, c);
    }
    let mut winkel = sw;
    while winkel < ew {
        winkel += schritt;
        if winkel > ew {
            winkel = ew;
        }
        punkte.push((x + (dr * winkel.cos()) as i32, y + (dr * winkel.sin()) as i32));
    }
    if gefuellt {
        filled_polygon(s, &punkte, c);
    } else {
        polygon(s, &punkte, c);
    }
}

/// Eine Bezierkurve - `bezierColor`.
pub fn bezier(s: &mut Surface, punkte: &[(i32, i32)], schritte: i32, c: Color) {
    let n = punkte.len();
    if n < 3 || schritte < 2 {
        return;
    }
    let mut xs: Vec<f64> = punkte.iter().map(|p| p.0 as f64).collect();
    let mut ys: Vec<f64> = punkte.iter().map(|p| p.1 as f64).collect();
    xs.push(punkte[0].0 as f64);
    ys.push(punkte[0].1 as f64);
    let gesamt = schritte * n as i32;
    let mut x1 = auswerten(&xs, n + 1, 0, gesamt).round() as i32;
    let mut y1 = auswerten(&ys, n + 1, 0, gesamt).round() as i32;
    for i in 1..=gesamt {
        // **`n`, nicht `n + 1`** - so steht es in der Vorlage, und es ist
        // kein Tippfehler: der Anfangspunkt wird mit dem geschlossenen
        // Zug gerechnet, die Kurve selbst ohne ihn.
        let x2 = auswerten(&xs, n, i, gesamt) as i32;
        let y2 = auswerten(&ys, n, i, gesamt) as i32;
        line(s, x1, y1, x2, y2, c);
        x1 = x2;
        y1 = y2;
    }
}

/// `_evaluateBezier` - der Bernstein-Ansatz aus der Vorlage.
fn auswerten(daten: &[f64], anzahl: usize, t: i32, gesamt: i32) -> f64 {
    if t < 0 {
        return daten[0];
    }
    if t >= gesamt {
        return daten[anzahl - 1];
    }
    let mu = t as f64 / gesamt as f64;
    let n = anzahl - 1;
    let mut ergebnis = 0.0f64;
    let mut muk = 1.0f64;
    let mut munk = (1.0 - mu).powi(n as i32);
    if munk <= 0.0 {
        return daten[anzahl - 1];
    }
    // `k` steht hier nicht nur fuer den Wert, sondern auch in der
    // Binomialrechnung darunter - eine Schleife ueber die Werte allein
    // taete es also nicht.
    #[allow(clippy::needless_range_loop)]
    for k in 0..=n {
        let mut nn = n as i64;
        let mut kn = k as i64;
        let mut nkn = (n - k) as i64;
        let mut blend = muk * munk;
        muk *= mu;
        munk /= 1.0 - mu;
        while nn >= 1 {
            blend *= nn as f64;
            nn -= 1;
            if kn > 1 {
                blend /= kn as f64;
                kn -= 1;
            }
            if nkn > 1 {
                blend /= nkn as f64;
                nkn -= 1;
            }
        }
        ergebnis += daten[k] * blend;
    }
    ergebnis
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;

    fn flaeche() -> Surface {
        let mut s = Surface::new((21, 21));
        s.fill(Color::rgb(0, 0, 0));
        s
    }

    /// Ein Pixel mit halbem Alpha mischt `d + ((s - d) * a >> 8)`.
    #[test]
    fn mischung_wie_sdl_gfx() {
        let mut s = flaeche();
        s.fill(Color::rgb(200, 200, 200));
        pixel(&mut s, 1, 1, Color::rgba(0, 0, 0, 128));
        // 200 + ((0 - 200) * 128 >> 8) = 200 - 100 = 100
        assert_eq!(s.get_at((1, 1)).r, 100);
    }

    /// Das Gewicht geht **ins** Alpha, es ersetzt es nicht.
    #[test]
    fn gewicht_multipliziert_das_alpha() {
        let mut s = flaeche();
        s.fill(Color::rgb(0, 0, 0));
        // Alpha 128, Gewicht 128 -> wirksam 64.
        pixel_gewichtet(&mut s, 1, 1, Color::rgba(255, 255, 255, 128), 128);
        assert_eq!(s.get_at((1, 1)).r, 63, "255 * 64 >> 8");
    }

    /// Der Kreis ist symmetrisch und beruehrt die vier Achsenpunkte.
    #[test]
    fn kreis_trifft_die_achsen() {
        let mut s = flaeche();
        circle(&mut s, 10, 10, 5, Color::rgb(255, 255, 255));
        for p in [(15, 10), (5, 10), (10, 15), (10, 5)] {
            assert_eq!(s.get_at(p).r, 255, "{p:?}");
        }
        assert_eq!(s.get_at((10, 10)).r, 0, "innen bleibt leer");
    }

    /// Die gefuellte Scheibe deckt die Mitte.
    #[test]
    fn scheibe_deckt_die_mitte() {
        let mut s = flaeche();
        filled_circle(&mut s, 10, 10, 5, Color::rgb(255, 255, 255));
        assert_eq!(s.get_at((10, 10)).r, 255);
        assert_eq!(s.get_at((10, 16)).r, 0, "ausserhalb bleibt leer");
    }

    /// Das gefuellte Dreieck deckt seinen Schwerpunkt.
    #[test]
    fn dreieck_deckt_seinen_schwerpunkt() {
        let mut s = flaeche();
        filled_polygon(&mut s, &[(2, 2), (18, 2), (10, 18)], Color::rgb(255, 255, 255));
        assert_eq!(s.get_at((10, 6)).r, 255);
        assert_eq!(s.get_at((3, 16)).r, 0);
    }

    /// Eine waagerechte Linie trifft genau ihre Pixel.
    #[test]
    fn linie_trifft_ihre_pixel() {
        let mut s = flaeche();
        line(&mut s, 2, 3, 8, 3, Color::rgb(255, 255, 255));
        for x in 2..=8 {
            assert_eq!(s.get_at((x, 3)).r, 255, "x={x}");
        }
        assert_eq!(s.get_at((9, 3)).r, 0);
    }
}
