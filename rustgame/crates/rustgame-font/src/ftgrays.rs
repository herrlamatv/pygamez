//! ftgrays.rs
//! ==========
//! FreeTypes Graustufen-Rasterer, **uebertragen** aus FreeType 2.11.1 -
//! der Fassung, die in pygame-ce 2.5.7 steckt (`freetype.get_version()`
//! sagt `(2, 11, 1)`).
//!
//! ## Warum nicht fontdue
//!
//! fontdue rechnet die Flaechendeckung ebenfalls exakt, aber auf einem
//! anderen Weg: Fliesskomma statt 24.8-Festkomma, andere Zerlegung der
//! Kurven in Strecken. Gemessen an einem `H` in Bahnschrift 22 lagen die
//! Kantenspalten bei 31 statt 32 und 49 statt 48 - eine Stufe, aber in
//! **jedem** Zeichen. Pixelgleicher Text braucht FreeTypes Rechnung,
//! nicht eine gleichwertige.
//!
//! ## Was uebertragen ist
//!
//! | C-Quelle | hier |
//! |---|---|
//! | `ftoutln.c` `FT_Outline_Decompose` | [`zerlegen`] |
//! | `ftoutln.c` `FT_Outline_Get_CBox` | [`Umriss::kontrollkasten`] |
//! | `ftobjs.c` `ft_glyphslot_preset_bitmap` (Modus NORMAL) | [`rendern`] |
//! | `ftsmooth.c` `ft_smooth_render`, `ft_smooth_raster_overlap`, `ft_smooth_overlap_spans` | [`rendern`] |
//! | `ftgrays.c` `gray_set_cell`, `gray_render_line` (Fassung mit `FT_INT64`), `gray_render_conic` (DDA), `gray_split_cubic`, `gray_render_cubic`, `gray_sweep`, `gray_sweep_direct` | [`Arbeiter`] |
//!
//! **Welche Fassung gilt, steht in der C-Quelle unter `#if`**, und die
//! Wahl ist nicht gleichgueltig: unter MSVC auf x64 ist `FT_INT64`
//! definiert (also die zweite `gray_render_line` mit `FT_UDIVPREP`) und
//! `BEZIER_USE_DDA` an (Bezier-Boegen ueber Differenzen statt
//! Halbierungen). Die SSE2-Fassung der DDA-Schleife rechnet dieselben
//! 64-Bit-Summen wie die skalare.
//!
//! **Baender** (`gray_convert_glyph` teilt grosse Glyphen in Streifen,
//! wenn der Zellenvorrat ueberlaeuft) aendern am Ergebnis nichts: jede
//! Zeile bekommt dieselben Zellen, gleich in welchem Band sie liegt. Hier
//! gibt es darum ein einziges Band mit unbegrenztem Vorrat.
//!
//! **Typen.** `TPos` ist in C ein `long` - unter Windows 32 Bit. Hier ist
//! es `i64`: fuer jede Glyphe, die in eine Bitmap mit Seiten unter 32768
//! Pixeln passt (`ft_glyphslot_preset_bitmap` lehnt groessere ab), sind
//! beide gleich, und `i64` laeuft im Pruefbau nicht ueber. `TCoord` und
//! `TArea` sind `int`, hier `i32` mit den Ueberlaufregeln von C
//! (`ADD_INT` rechnet vorzeichenlos).

/// 8 Bit Unterpixel - `PIXEL_BITS` in `ftgrays.c`.
const PIXEL_BITS: i64 = 8;
/// `ONE_PIXEL`.
const ONE_PIXEL: i64 = 1 << PIXEL_BITS;

/// `TRUNC`: von 24.8 auf das ganze Pixel.
#[inline]
fn trunc(x: i64) -> i32 {
    (x >> PIXEL_BITS) as i32
}

/// `FRACT`: der Anteil im Pixel.
#[inline]
fn fract(x: i64) -> i32 {
    (x & (ONE_PIXEL - 1)) as i32
}

/// `UPSCALE`: von 26.6 auf 24.8.
#[inline]
fn upscale(x: i64) -> i64 {
    x * (ONE_PIXEL >> 6)
}

/// Punktart wie `FT_CURVE_TAG`: auf der Kurve, quadratischer oder
/// kubischer Kontrollpunkt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Art {
    /// `FT_CURVE_TAG_ON`.
    Auf,
    /// `FT_CURVE_TAG_CONIC`.
    Quadratisch,
    /// `FT_CURVE_TAG_CUBIC`.
    Kubisch,
}

/// Ein Umriss wie `FT_Outline`: Punkte in 26.6, je Punkt seine Art, das
/// Ende jeder Kontur.
#[derive(Debug, Clone, Default)]
pub(crate) struct Umriss {
    pub(crate) punkte: Vec<(i64, i64)>,
    pub(crate) arten: Vec<Art>,
    /// Index des **letzten** Punktes jeder Kontur (`contours[]`).
    pub(crate) enden: Vec<usize>,
    /// `FT_OUTLINE_OVERLAP` - die Kontur ueberlappt sich selbst und wird
    /// darum vierfach ueberabgetastet (`ft_smooth_raster_overlap`).
    pub(crate) ueberlappt: bool,
}

impl Umriss {
    /// `FT_Outline_Get_CBox` - ueber **alle** Punkte, auch die
    /// Kontrollpunkte.
    pub(crate) fn kontrollkasten(&self) -> (i64, i64, i64, i64) {
        let Some(&(x0, y0)) = self.punkte.first() else {
            return (0, 0, 0, 0);
        };
        let (mut xmin, mut ymin, mut xmax, mut ymax) = (x0, y0, x0, y0);
        for &(x, y) in &self.punkte[1..] {
            xmin = xmin.min(x);
            xmax = xmax.max(x);
            ymin = ymin.min(y);
            ymax = ymax.max(y);
        }
        (xmin, ymin, xmax, ymax)
    }
}

/// Was `FT_Outline_Decompose` aufruft - `FT_Outline_Funcs`.
pub(crate) trait Ziel {
    fn move_to(&mut self, to: (i64, i64));
    fn line_to(&mut self, to: (i64, i64));
    fn conic_to(&mut self, c: (i64, i64), to: (i64, i64));
    fn cubic_to(&mut self, c1: (i64, i64), c2: (i64, i64), to: (i64, i64));
}

/// `FT_Outline_Decompose` - Zeile fuer Zeile, mit `shift = delta = 0`.
///
/// Gibt `false` bei einem ungueltigen Umriss (eine Kontur, die mit einem
/// kubischen Kontrollpunkt beginnt, oder ein einzelner kubischer Punkt).
///
/// Die Mittelpunkte zweier quadratischer Kontrollpunkte sind
/// **Ganzzahldivisionen** (`( a + b ) / 2`, gegen null abgeschnitten) -
/// das unterscheidet sich von `>> 1` bei negativen Werten, und genau so
/// steht es in C.
pub(crate) fn zerlegen(u: &Umriss, ziel: &mut impl Ziel) -> bool {
    let mut first = 0usize;
    for &last in &u.enden {
        if last < first || last >= u.punkte.len() {
            return false;
        }
        let mut limit = last;
        let mut v_start = u.punkte[first];
        let v_last = u.punkte[last];
        let mut v_control;
        // `point` zeigt auf den zuletzt verbrauchten Punkt; beginnt die
        // Kontur mit einem Kontrollpunkt, steht er "vor" dem ersten.
        let mut point: isize = first as isize;
        let tag = u.arten[first];
        if tag == Art::Kubisch {
            return false;
        }
        if tag == Art::Quadratisch {
            if u.arten[last] == Art::Auf {
                // mit dem letzten Punkt beginnen, wenn er auf der Kurve liegt
                v_start = v_last;
                limit -= 1;
            } else {
                // sonst in der Mitte zwischen erstem und letztem
                v_start = ((v_start.0 + v_last.0) / 2, (v_start.1 + v_last.1) / 2);
            }
            point -= 1;
        }
        ziel.move_to(v_start);

        let mut geschlossen = false;
        while point < limit as isize {
            point += 1;
            let p = point as usize;
            match u.arten[p] {
                Art::Auf => {
                    ziel.line_to(u.punkte[p]);
                }
                Art::Quadratisch => {
                    v_control = u.punkte[p];
                    loop {
                        // `Do_Conic`
                        if point < limit as isize {
                            point += 1;
                            let q = point as usize;
                            let vec = u.punkte[q];
                            match u.arten[q] {
                                Art::Auf => {
                                    ziel.conic_to(v_control, vec);
                                    break;
                                }
                                Art::Quadratisch => {
                                    let v_middle =
                                        ((v_control.0 + vec.0) / 2, (v_control.1 + vec.1) / 2);
                                    ziel.conic_to(v_control, v_middle);
                                    v_control = vec;
                                    continue;
                                }
                                Art::Kubisch => return false,
                            }
                        }
                        ziel.conic_to(v_control, v_start);
                        geschlossen = true;
                        break;
                    }
                    if geschlossen {
                        break;
                    }
                }
                Art::Kubisch => {
                    if point + 1 > limit as isize || u.arten[p + 1] != Art::Kubisch {
                        return false;
                    }
                    point += 2;
                    let q = point as usize;
                    let vec1 = u.punkte[q - 2];
                    let vec2 = u.punkte[q - 1];
                    if point <= limit as isize {
                        ziel.cubic_to(vec1, vec2, u.punkte[q]);
                        continue;
                    }
                    ziel.cubic_to(vec1, vec2, v_start);
                    geschlossen = true;
                    break;
                }
            }
        }
        // Die Kontur mit einer Strecke schliessen - auch wenn sie dabei
        // keine Laenge hat; `goto Close` springt nur nach einem Bogen
        // zurueck zum Anfang daran vorbei.
        if !geschlossen {
            ziel.line_to(v_start);
        }
        first = last + 1;
    }
    true
}

// ---------------------------------------------------------------------------
//  Der Rasterer
// ---------------------------------------------------------------------------

/// `TCell`.
#[derive(Debug, Clone, Copy)]
struct Zelle {
    x: i32,
    cover: i32,
    area: i32,
    next: usize,
}

/// Index der Nullzelle (`cell_null`): der Muelleimer fuer alles ausserhalb
/// des Clip-Bereichs, und zugleich das Listenende mit `x = INT_MAX`.
const NULL: usize = 0;

/// `gray_TWorker` - ohne Sprungpuffer und Banddaten.
struct Arbeiter {
    min_ex: i32,
    max_ex: i32,
    min_ey: i32,
    max_ey: i32,
    count_ey: i32,
    zellen: Vec<Zelle>,
    ycells: Vec<usize>,
    cell: usize,
    x: i64,
    y: i64,
}

impl Arbeiter {
    fn neu(min_ex: i32, min_ey: i32, max_ex: i32, max_ey: i32) -> Arbeiter {
        let count_ey = max_ey - min_ey;
        Arbeiter {
            min_ex,
            max_ex,
            min_ey,
            max_ey,
            count_ey,
            zellen: vec![Zelle {
                x: i32::MAX,
                cover: 0,
                area: 0,
                next: NULL,
            }],
            ycells: vec![NULL; count_ey.max(0) as usize],
            cell: NULL,
            x: 0,
            y: 0,
        }
    }

    /// `FT_INTEGRATE` - auch auf die Nullzelle, wie in C (dort landet es im
    /// Muelleimer und wird nie gelesen).
    #[inline]
    fn integrate(&mut self, a: i32, b: i32) {
        let c = &mut self.zellen[self.cell];
        c.cover = c.cover.wrapping_add(a);
        c.area = c.area.wrapping_add(a.wrapping_mul(b));
    }

    /// `gray_set_cell`.
    fn set_cell(&mut self, ex: i32, ey: i32) {
        let ey_index = ey - self.min_ey;
        if ey_index < 0 || ey_index >= self.count_ey || ex >= self.max_ex {
            self.cell = NULL;
            return;
        }
        let ex = ex.max(self.min_ex - 1);
        let mut vorher: Option<usize> = None;
        let mut c = self.ycells[ey_index as usize];
        loop {
            let z = self.zellen[c];
            if z.x > ex {
                break;
            }
            if z.x == ex {
                self.cell = c;
                return;
            }
            vorher = Some(c);
            c = z.next;
        }
        let neu = self.zellen.len();
        self.zellen.push(Zelle {
            x: ex,
            cover: 0,
            area: 0,
            next: c,
        });
        match vorher {
            None => self.ycells[ey_index as usize] = neu,
            Some(v) => self.zellen[v].next = neu,
        }
        self.cell = neu;
    }

    /// `gray_render_line` - die Fassung **mit** `FT_INT64`.
    fn render_line(&mut self, to_x: i64, to_y: i64) {
        let mut ey1 = trunc(self.y);
        let ey2 = trunc(to_y);

        // senkrecht ausserhalb des Bandes
        if (ey1 >= self.max_ey && ey2 >= self.max_ey) || (ey1 < self.min_ey && ey2 < self.min_ey)
        {
            self.x = to_x;
            self.y = to_y;
            return;
        }

        let mut ex1 = trunc(self.x);
        let ex2 = trunc(to_x);
        let mut fx1 = fract(self.x);
        let mut fy1 = fract(self.y);
        let dx = to_x - self.x;
        let dy = to_y - self.y;

        if ex1 == ex2 && ey1 == ey2 {
            // innerhalb einer Zelle
        } else if dy == 0 {
            // jede waagerechte Strecke
            self.set_cell(ex2, ey2);
            self.x = to_x;
            self.y = to_y;
            return;
        } else if dx == 0 {
            if dy > 0 {
                loop {
                    let fy2 = ONE_PIXEL as i32;
                    self.integrate(fy2 - fy1, fx1 * 2);
                    fy1 = 0;
                    ey1 += 1;
                    self.set_cell(ex1, ey1);
                    if ey1 == ey2 {
                        break;
                    }
                }
            } else {
                loop {
                    let fy2 = 0;
                    self.integrate(fy2 - fy1, fx1 * 2);
                    fy1 = ONE_PIXEL as i32;
                    ey1 -= 1;
                    self.set_cell(ex1, ey1);
                    if ey1 == ey2 {
                        break;
                    }
                }
            }
        } else {
            let mut prod: i64 = dx * fy1 as i64 - dy * fx1 as i64;
            // `FT_UDIVPREP`: der Kehrwert nur, wenn er gebraucht wird.
            let dx_r: i64 = if ex1 != ex2 { 0xFFFF_FFFFi64 / dx } else { 0 };
            let dy_r: i64 = if ey1 != ey2 { 0xFFFF_FFFFi64 / dy } else { 0 };
            // `FT_UDIV( a, b )` mit `b_r` - vorzeichenlos, wie in C.
            let udiv = |a: i64, b_r: i64| ((a as u64).wrapping_mul(b_r as u64) >> 32) as i32;
            let op = ONE_PIXEL;
            loop {
                let fx2;
                let fy2;
                if prod - dx * op > 0 && prod <= 0 {
                    // links
                    fx2 = 0;
                    fy2 = udiv(-prod, -dx_r);
                    prod -= dy * op;
                    self.integrate(fy2 - fy1, fx1 + fx2);
                    fx1 = op as i32;
                    fy1 = fy2;
                    ex1 -= 1;
                } else if prod - dx * op + dy * op > 0 && prod - dx * op <= 0 {
                    // hoch
                    prod -= dx * op;
                    fx2 = udiv(-prod, dy_r);
                    fy2 = op as i32;
                    self.integrate(fy2 - fy1, fx1 + fx2);
                    fx1 = fx2;
                    fy1 = 0;
                    ey1 += 1;
                } else if prod + dy * op >= 0 && prod - dx * op + dy * op <= 0 {
                    // rechts
                    prod += dy * op;
                    fx2 = op as i32;
                    fy2 = udiv(prod, dx_r);
                    self.integrate(fy2 - fy1, fx1 + fx2);
                    fx1 = 0;
                    fy1 = fy2;
                    ex1 += 1;
                } else {
                    // runter
                    fx2 = udiv(prod, -dy_r);
                    fy2 = 0;
                    prod += dx * op;
                    self.integrate(fy2 - fy1, fx1 + fx2);
                    fx1 = fx2;
                    fy1 = op as i32;
                    ey1 -= 1;
                }
                self.set_cell(ex1, ey1);
                if ex1 == ex2 && ey1 == ey2 {
                    break;
                }
            }
        }

        let fx2 = fract(to_x);
        let fy2 = fract(to_y);
        self.integrate(fy2 - fy1, fx1 + fx2);
        self.x = to_x;
        self.y = to_y;
    }

    /// `gray_render_conic` - die DDA-Fassung (`BEZIER_USE_DDA`).
    fn render_conic(&mut self, control: (i64, i64), to: (i64, i64)) {
        let p0 = (self.x, self.y);
        let p1 = (upscale(control.0), upscale(control.1));
        let p2 = (upscale(to.0), upscale(to.1));

        // den Bogen abkuerzen, der das Band nicht beruehrt
        if (trunc(p0.1) >= self.max_ey && trunc(p1.1) >= self.max_ey && trunc(p2.1) >= self.max_ey)
            || (trunc(p0.1) < self.min_ey && trunc(p1.1) < self.min_ey && trunc(p2.1) < self.min_ey)
        {
            self.x = p2.0;
            self.y = p2.1;
            return;
        }

        let bx = p1.0 - p0.0;
        let by = p1.1 - p0.1;
        let ax = p2.0 - p1.0 - bx;
        let ay = p2.1 - p1.1 - by;

        let mut dx = ax.abs();
        let dy = ay.abs();
        if dx < dy {
            dx = dy;
        }
        if dx <= ONE_PIXEL / 4 {
            self.render_line(p2.0, p2.1);
            return;
        }
        let mut shift = 0u32;
        loop {
            dx >>= 2;
            shift += 1;
            if dx <= ONE_PIXEL / 4 {
                break;
            }
        }

        // `LEFT_SHIFT( a, b )` = `(FT_Int64)( (FT_UInt64)(a) << (b) )`
        let ls = |a: i64, b: u32| ((a as u64) << b) as i64;
        let rx = ls(ax, 33 - 2 * shift);
        let ry = ls(ay, 33 - 2 * shift);
        let mut qx = ls(bx, 33 - shift).wrapping_add(ls(ax, 32 - 2 * shift));
        let mut qy = ls(by, 33 - shift).wrapping_add(ls(ay, 32 - 2 * shift));
        let mut px = ls(p0.0, 32);
        let mut py = ls(p0.1, 32);

        let mut count = 1u32 << shift;
        while count > 0 {
            px = px.wrapping_add(qx);
            py = py.wrapping_add(qy);
            qx = qx.wrapping_add(rx);
            qy = qy.wrapping_add(ry);
            // `(FT_Pos)( px >> 32 )` - ein `long`, unter Windows 32 Bit
            self.render_line((px >> 32) as i32 as i64, (py >> 32) as i32 as i64);
            count -= 1;
        }
    }

    /// `gray_render_cubic` mit `gray_split_cubic`.
    fn render_cubic(&mut self, c1: (i64, i64), c2: (i64, i64), to: (i64, i64)) {
        // In C `16 * 3 + 1` Plaetze - genug, solange keine Glyphe mehr als
        // vierzehn Halbierungen braucht (die Abweichung faellt je Halbierung
        // auf ein Achtel). `split_cubic` schreibt sieben Plaetze ab `arc`;
        // die Reserve macht aus dem undenkbaren Fall keinen Abbruch.
        let mut stapel = [(0i64, 0i64); 16 * 3 + 7];
        let mut arc = 0usize;
        stapel[0] = (upscale(to.0), upscale(to.1));
        stapel[1] = (upscale(c2.0), upscale(c2.1));
        stapel[2] = (upscale(c1.0), upscale(c1.1));
        stapel[3] = (self.x, self.y);

        let ausserhalb = |s: &[(i64, i64)], max_ey: i32, min_ey: i32| {
            (0..4).all(|k| trunc(s[k].1) >= max_ey) || (0..4).all(|k| trunc(s[k].1) < min_ey)
        };
        if ausserhalb(&stapel[0..4], self.max_ey, self.min_ey) {
            self.x = stapel[0].0;
            self.y = stapel[0].1;
            return;
        }

        loop {
            let a = &stapel[arc..arc + 4];
            let flach = !((2 * a[0].0 - 3 * a[1].0 + a[3].0).abs() > ONE_PIXEL / 2
                || (2 * a[0].1 - 3 * a[1].1 + a[3].1).abs() > ONE_PIXEL / 2
                || (a[0].0 - 3 * a[2].0 + 2 * a[3].0).abs() > ONE_PIXEL / 2
                || (a[0].1 - 3 * a[2].1 + 2 * a[3].1).abs() > ONE_PIXEL / 2);
            if flach {
                let (x, y) = stapel[arc];
                self.render_line(x, y);
                if arc == 0 {
                    return;
                }
                arc -= 3;
                continue;
            }
            split_cubic(&mut stapel[arc..arc + 7]);
            arc += 3;
        }
    }

    /// `gray_sweep`: die Zellen in eine Graustufen-Bitmap, Zeile 0 **unten**
    /// (FreeTypes Ursprung), `breite` Bytes je Zeile.
    fn sweep(&self, bild: &mut [u8], breite: usize) {
        for y in self.min_ey..self.max_ey {
            let mut c = self.ycells[(y - self.min_ey) as usize];
            let mut x = self.min_ex;
            let mut cover: i32 = 0;
            let zeile = &mut bild[y as usize * breite..(y as usize + 1) * breite];
            while c != NULL {
                let z = self.zellen[c];
                if cover != 0 && z.x > x {
                    let w = fuellregel(cover);
                    zeile[x as usize..z.x as usize].fill(w);
                }
                cover = cover.wrapping_add(z.cover.wrapping_mul((ONE_PIXEL * 2) as i32));
                let area = cover.wrapping_sub(z.area);
                if area != 0 && z.x >= self.min_ex {
                    zeile[z.x as usize] = fuellregel(area);
                }
                x = z.x + 1;
                c = z.next;
            }
            if cover != 0 {
                let w = fuellregel(cover);
                zeile[x as usize..self.max_ex as usize].fill(w);
            }
        }
    }

    /// `gray_sweep_direct` mit `ft_smooth_overlap_spans` als Empfaenger:
    /// die vierfach ueberabgetasteten Spannen werden in `bild` (die
    /// **normale** Aufloesung, Zeile 0 unten) gemittelt.
    fn sweep_ueberabgetastet(&self, bild: &mut [u8], breite: usize) {
        const SCALE: i32 = 4;
        let mut spanne = |y: i32, x0: i32, len: i32, coverage: u8| {
            let zeile = (y / SCALE) as usize * breite;
            let cover = (coverage as u32 + (SCALE * SCALE / 2) as u32) / (SCALE * SCALE) as u32;
            for x in 0..len {
                let i = zeile + ((x0 + x) / SCALE) as usize;
                let sum = bild[i] as u32 + cover;
                bild[i] = (sum - (sum >> 8)) as u8;
            }
        };
        for y in self.min_ey..self.max_ey {
            let mut c = self.ycells[(y - self.min_ey) as usize];
            let mut x = self.min_ex;
            let mut cover: i32 = 0;
            while c != NULL {
                let z = self.zellen[c];
                if cover != 0 && z.x > x {
                    spanne(y, x, z.x - x, fuellregel(cover));
                }
                cover = cover.wrapping_add(z.cover.wrapping_mul((ONE_PIXEL * 2) as i32));
                let area = cover.wrapping_sub(z.area);
                if area != 0 && z.x >= self.min_ex {
                    spanne(y, z.x, 1, fuellregel(area));
                }
                x = z.x + 1;
                c = z.next;
            }
            if cover != 0 {
                spanne(y, x, self.max_ex - x, fuellregel(cover));
            }
        }
    }
}

/// `FT_FILL_RULE` mit der Nicht-Null-Regel (`fill = INT_MIN`) - TrueType
/// kennt keine andere.
#[inline]
fn fuellregel(area: i32) -> u8 {
    let mut coverage = area >> (PIXEL_BITS * 2 + 1 - 8);
    if coverage & i32::MIN != 0 {
        coverage = !coverage;
    }
    if coverage > 255 {
        coverage = 255;
    }
    coverage as u8
}

/// `gray_split_cubic` - halbiert den Bogen `base[0..4]` nach `base[0..7]`.
fn split_cubic(base: &mut [(i64, i64)]) {
    for k in 0..2 {
        let g = |p: &(i64, i64)| if k == 0 { p.0 } else { p.1 };
        let mut v = [0i64; 7];
        v[6] = g(&base[3]);
        let mut a = g(&base[0]) + g(&base[1]);
        let b = g(&base[1]) + g(&base[2]);
        let mut c = g(&base[2]) + g(&base[3]);
        v[5] = c >> 1;
        c += b;
        v[4] = c >> 2;
        v[1] = a >> 1;
        a += b;
        v[2] = a >> 2;
        v[3] = (a + c) >> 3;
        v[0] = g(&base[0]);
        for (i, w) in v.iter().enumerate() {
            if k == 0 {
                base[i].0 = *w;
            } else {
                base[i].1 = *w;
            }
        }
    }
}

impl Ziel for Arbeiter {
    /// `gray_move_to`.
    fn move_to(&mut self, to: (i64, i64)) {
        let x = upscale(to.0);
        let y = upscale(to.1);
        self.set_cell(trunc(x), trunc(y));
        self.x = x;
        self.y = y;
    }

    /// `gray_line_to`.
    fn line_to(&mut self, to: (i64, i64)) {
        self.render_line(upscale(to.0), upscale(to.1));
    }

    fn conic_to(&mut self, c: (i64, i64), to: (i64, i64)) {
        self.render_conic(c, to);
    }

    fn cubic_to(&mut self, c1: (i64, i64), c2: (i64, i64), to: (i64, i64)) {
        self.render_cubic(c1, c2, to);
    }
}

/// Eine gerasterte Glyphe, wie `FT_GlyphSlot` sie nach `FT_Render_Glyph`
/// traegt.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Graubild {
    /// `bitmap_left`.
    pub(crate) left: i32,
    /// `bitmap_top`.
    pub(crate) top: i32,
    pub(crate) breite: usize,
    pub(crate) hoehe: usize,
    /// Deckung, Zeile 0 **oben** (wie FreeTypes Bitmap mit positivem
    /// `pitch`).
    pub(crate) deckung: Vec<u8>,
}

/// Der Kasten, den `ft_glyphslot_preset_bitmap` im Modus NORMAL schon
/// **beim Laden** setzt - `(left, top, breite, hoehe)`.
///
/// SDL_ttf liest daraus `sz_left/top/width/rows`, und zwar auch fuer
/// einfarbigen Text: `FT_Load_Glyph` rechnet den Kasten mit dem Ziel der
/// Ladeflaggen (`FT_LOAD_TARGET_NORMAL`), das Bild kommt spaeter und hat
/// im Modus MONO einen eigenen Kasten.
pub(crate) fn kasten_normal(u: &Umriss) -> (i32, i32, usize, usize) {
    let (cx0, cy0, cx1, cy1) = u.kontrollkasten();
    let xmin = (cx0 >> 6) + ((cx0 & 63) >> 6);
    let ymin = (cy0 >> 6) + ((cy0 & 63) >> 6);
    let xmax = (cx1 >> 6) + (((cx1 & 63) + 63) >> 6);
    let ymax = (cy1 >> 6) + (((cy1 & 63) + 63) >> 6);
    (xmin as i32, ymax as i32, (xmax - xmin).max(0) as usize, (ymax - ymin).max(0) as usize)
}

/// `FT_Render_Glyph` mit `FT_RENDER_MODE_NORMAL` und ohne Ursprung.
///
/// `ft_glyphslot_preset_bitmap` bestimmt den Kasten aus dem
/// Kontrollkasten (links und unten abgerundet, rechts und oben
/// aufgerundet), `ft_smooth_render` verschiebt den Umriss so, dass die
/// linke untere Ecke des Kastens auf `(0, 0)` liegt, und rastert. Gibt
/// `None`, wo FreeType mit `Raster_Overflow` abbricht.
pub(crate) fn rendern(u: &Umriss) -> Option<Graubild> {
    let (cx0, cy0, cx1, cy1) = u.kontrollkasten();
    // Modus NORMAL: `pbox.xMin += cbox.xMin >> 6` usw. - mit der "groben"
    // Schaetzung zusammen ist das Abrunden links/unten, Aufrunden rechts/oben.
    let xmin = (cx0 >> 6) + ((cx0 & 63) >> 6);
    let ymin = (cy0 >> 6) + ((cy0 & 63) >> 6);
    let xmax = (cx1 >> 6) + (((cx1 & 63) + 63) >> 6);
    let ymax = (cy1 >> 6) + (((cy1 & 63) + 63) >> 6);
    if xmin < -0x8000 || xmax > 0x7FFF || ymin < -0x8000 || ymax > 0x7FFF {
        return None;
    }
    let breite = (xmax - xmin) as usize;
    let hoehe = (ymax - ymin) as usize;
    let mut bild = Graubild {
        left: xmin as i32,
        top: ymax as i32,
        breite,
        hoehe,
        deckung: vec![0u8; breite * hoehe],
    };
    if breite == 0 || hoehe == 0 || u.punkte.is_empty() || u.enden.is_empty() {
        return Some(bild);
    }

    // `x_shift = 64 * -bitmap_left`, `y_shift = 64 * -bitmap_top + 64 * rows`
    let sx = 64 * -xmin;
    let sy = 64 * -ymax + 64 * hoehe as i64;
    let mut verschoben = u.clone();
    for p in &mut verschoben.punkte {
        p.0 += sx;
        p.1 += sy;
    }

    // FreeType rastert mit Zeile 0 unten; umgedreht wird am Ende.
    let mut unten_zuerst = vec![0u8; breite * hoehe];
    if u.ueberlappt {
        // `ft_smooth_raster_overlap`: Umriss mal vier, direkt in Spannen,
        // die zurueck auf die normale Aufloesung gemittelt werden.
        const SCALE: i64 = 4;
        if breite as i64 * SCALE > 0x7FFF {
            return None;
        }
        for p in &mut verschoben.punkte {
            p.0 *= SCALE;
            p.1 *= SCALE;
        }
        let mut w = Arbeiter::neu(0, 0, (breite as i64 * SCALE) as i32, (hoehe as i64 * SCALE) as i32);
        if !zerlegen(&verschoben, &mut w) {
            return None;
        }
        w.sweep_ueberabgetastet(&mut unten_zuerst, breite);
    } else {
        let mut w = Arbeiter::neu(0, 0, breite as i32, hoehe as i32);
        if !zerlegen(&verschoben, &mut w) {
            return None;
        }
        w.sweep(&mut unten_zuerst, breite);
    }
    for y in 0..hoehe {
        let von = (hoehe - 1 - y) * breite;
        bild.deckung[y * breite..(y + 1) * breite].copy_from_slice(&unten_zuerst[von..von + breite]);
    }
    Some(bild)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quadrat(x0: i64, y0: i64, x1: i64, y1: i64) -> Umriss {
        Umriss {
            punkte: vec![(x0, y0), (x0, y1), (x1, y1), (x1, y0)],
            arten: vec![Art::Auf; 4],
            enden: vec![3],
            ueberlappt: false,
        }
    }

    #[test]
    fn ganzes_pixel_ist_voll() {
        // Ein Quadrat genau auf dem Pixelgitter: 2x2 Pixel, alle 255.
        let b = rendern(&quadrat(0, 0, 128, 128)).unwrap();
        assert_eq!((b.left, b.top, b.breite, b.hoehe), (0, 2, 2, 2));
        assert_eq!(b.deckung, vec![255; 4]);
    }

    #[test]
    fn halbes_pixel_ist_halb() {
        // Ein halbes Pixel breit: Deckung 128 (256 * 0,5 abgeschnitten
        // ueber `area >> 9`).
        let b = rendern(&quadrat(0, 0, 32, 64)).unwrap();
        assert_eq!((b.breite, b.hoehe), (1, 1));
        assert_eq!(b.deckung, vec![128]);
    }

    #[test]
    fn kasten_rundet_nach_aussen() {
        // Kontrollkasten (10, 20)-(70, 100) in 26.6: links/unten ab-,
        // rechts/oben aufgerundet -> Pixel 0..2 und 0..2.
        let b = rendern(&quadrat(10, 20, 70, 100)).unwrap();
        assert_eq!((b.left, b.top, b.breite, b.hoehe), (0, 2, 2, 2));
    }

    #[test]
    fn mittelpunkt_ist_ganzzahldivision() {
        // Zwei quadratische Kontrollpunkte mit ungerader negativer Summe:
        // C rundet gegen null (-3 / 2 = -1), `>> 1` gaebe -2.
        struct Merker(Vec<(i64, i64)>);
        impl Ziel for Merker {
            fn move_to(&mut self, to: (i64, i64)) {
                self.0.push(to);
            }
            fn line_to(&mut self, to: (i64, i64)) {
                self.0.push(to);
            }
            fn conic_to(&mut self, _c: (i64, i64), to: (i64, i64)) {
                self.0.push(to);
            }
            fn cubic_to(&mut self, _a: (i64, i64), _b: (i64, i64), to: (i64, i64)) {
                self.0.push(to);
            }
        }
        let u = Umriss {
            punkte: vec![(0, 0), (-1, 10), (-2, 10), (5, 0)],
            arten: vec![Art::Auf, Art::Quadratisch, Art::Quadratisch, Art::Auf],
            enden: vec![3],
            ueberlappt: false,
        };
        let mut m = Merker(Vec::new());
        assert!(zerlegen(&u, &mut m));
        assert_eq!(m.0, vec![(0, 0), (-1, 10), (5, 0), (0, 0)]);
    }
}
