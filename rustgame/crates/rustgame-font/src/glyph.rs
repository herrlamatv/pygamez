//! glyph.rs
//! ========
//! Ein einzelnes gerastertes Zeichen - und die beiden **nachgemachten**
//! Stile.
//!
//! Fett und kursiv gibt es in SDL_ttf zweimal: als echte Schriftschnitt-Datei
//! (darum kuemmert sich [`crate::sysfont`]) und als Nachahmung, wenn nur der
//! Grundschnitt da ist. Letztere ist hier drin, und zwar so, wie SDL_ttf sie
//! macht:
//!
//! * **kursiv**: die Kontur wird geschert, `xy = 0x366A` in 16.16, also um
//!   den Faktor 0,2109. fontdue kann keine Kontur scheren, deshalb wird das
//!   fertige Bild zeilenweise verschoben - mit Zwischenwerten, sonst
//!   franst die Kante aus.
//! * **fett**: das Bild wird um `glyph_overhang = ptsize / 10` Spalten
//!   breiter und dann genau so oft nach rechts verschmiert:
//!   `p[x] = min(255, p[x] + p[x-1])`. Das ist die Schleife aus
//!   `SDL_ttf.c`, nicht etwas Aehnliches.

use crate::font::Stil;

/// Die vier Zahlen, die fontdue je Glyph liefert - und die eine
/// Bitmapschrift ebenso hergibt.
///
/// Gibt es, damit [`rastern`] beide Quellen gleich behandeln kann; ein
/// `fontdue::Metrics` laesst sich von aussen nicht bauen.
struct Mass {
    width: usize,
    height: usize,
    xmin: i32,
    ymin: i32,
}

impl Mass {
    fn aus(width: usize, height: usize, xmin: i32, ymin: i32) -> Mass {
        Mass {
            width,
            height,
            xmin,
            ymin,
        }
    }
}

/// Holt ein Zeichen aus einer Bitmapschrift und bringt es auf `px`.
///
/// **Naechster Nachbar, nicht geglaettet.** Eine Bitmapschrift wird in
/// ihrer eingebauten Groesse benutzt; verlangt jemand eine andere, macht
/// FreeType daraus grobe Kloetzchen und nichts Weicheres. Bei gleicher
/// Groesse - dem Normalfall - wird gar nicht gerechnet.
fn aus_punkten(b: &crate::bdf::Bitmapschrift, gid: u16, px: f32) -> (Mass, Vec<u8>) {
    let Some(z) = b.bei(gid) else {
        return (Mass::aus(0, 0, 0, 0), Vec::new());
    };
    if z.breite == 0 || z.hoehe == 0 {
        return (Mass::aus(0, 0, 0, 0), Vec::new());
    }
    let eigen = b.ppem.1.max(1) as f32;
    let f = if px > 0.0 { px / eigen } else { 1.0 };
    if (f - 1.0).abs() < 1e-6 {
        return (
            Mass::aus(
                z.breite as usize,
                z.hoehe as usize,
                z.x_versatz,
                z.y_versatz,
            ),
            z.deckung.clone(),
        );
    }
    let nb = ((z.breite as f32 * f).round() as usize).max(1);
    let nh = ((z.hoehe as f32 * f).round() as usize).max(1);
    let mut aus = vec![0u8; nb * nh];
    for y in 0..nh {
        let qy = ((y as f32 + 0.5) / f) as usize;
        let qy = qy.min(z.hoehe as usize - 1);
        for x in 0..nb {
            let qx = ((x as f32 + 0.5) / f) as usize;
            let qx = qx.min(z.breite as usize - 1);
            aus[y * nb + x] = z.deckung[qy * z.breite as usize + qx];
        }
    }
    (
        Mass::aus(
            nb,
            nh,
            (z.x_versatz as f32 * f).round() as i32,
            (z.y_versatz as f32 * f).round() as i32,
        ),
        aus,
    )
}

/// Macht aus einem Zeichen seinen **Rand**.
///
/// `font.outline = n` laesst FreeType die Glyphenkontur mit `FT_Stroker`
/// um `n` Pixel nach aussen ziehen und rastert den entstandenen Ring.
/// fontdue hat keinen Kontur-Zieher, also wird dasselbe auf dem fertigen
/// Bild gemacht: erst um `n` **aufblasen**, dann das Ursprungsbild wieder
/// abziehen. Was bleibt, ist ein Ring der Dicke `n`.
///
/// Das ist nicht Pixel fuer Pixel FreeTypes Ergebnis - der zieht eine
/// echte Kontur mit runden Ecken -, aber es ist dieselbe Sache und nicht
/// etwas Aehnliches: hohl, `n` Pixel dick, und ringsum `n` Pixel groesser.
/// Die Masse stimmen dadurch auf den Wert, und die pruefen pygames Tests:
/// gemessen waechst `size("Hallo")` von `(41, 16)` bei `outline = 0` auf
/// `(43, 18)`, `(45, 20)`, `(47, 22)` - also **2n** in beiden Richtungen,
/// waehrend der Vorschub unveraendert bleibt.
fn kontur(bild: &GlyphBild, n: i32) -> GlyphBild {
    let (b, h) = (bild.width, bild.rows);
    let nn = n as usize;
    let (nb, nh) = (b + 2 * nn, h + 2 * nn);
    let mut voll = vec![0u8; nb * nh];
    // Aufblasen: jeder Zielpunkt nimmt das Hellste aus der Scheibe mit
    // Halbmesser `n` um ihn herum.
    for y in 0..nh {
        for x in 0..nb {
            let mut beste = 0u8;
            for dy in -n..=n {
                for dx in -n..=n {
                    if dx * dx + dy * dy > n * n {
                        continue;
                    }
                    let qx = x as i32 - n + dx;
                    let qy = y as i32 - n + dy;
                    if qx < 0 || qy < 0 || qx >= b as i32 || qy >= h as i32 {
                        continue;
                    }
                    beste = beste.max(bild.deckung[qy as usize * b + qx as usize]);
                }
            }
            voll[y * nb + x] = beste;
        }
    }
    // Und das Innere wieder heraus.
    for y in 0..h {
        for x in 0..b {
            let i = (y + nn) * nb + x + nn;
            voll[i] = voll[i].saturating_sub(bild.deckung[y * b + x]);
        }
    }
    GlyphBild {
        left: bild.left - n,
        top: bild.top + n,
        width: nb,
        rows: nh,
        deckung: voll,
        m_left: bild.m_left - n,
        m_top: bild.m_top + n,
        m_width: bild.m_width + 2 * n,
        m_rows: bild.m_rows + 2 * n,
    }
}

/// Ein Bild auf FreeTypes ganzem Weg - siehe
/// [`crate::face::Datei::gehintet_fertig`].
#[derive(Debug, Clone)]
pub(crate) struct FtBild {
    /// NORMAL-Kasten beim Laden: `(left, top, breite, hoehe)` - die Masse.
    pub(crate) kasten: (i32, i32, usize, usize),
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) breite: usize,
    pub(crate) hoehe: usize,
    pub(crate) deckung: Vec<u8>,
    pub(crate) mono: bool,
}

/// `GLYPH_ITALICS` aus `SDL_ttf.c`: der Scherfaktor in 16.16.
pub(crate) const GLYPH_ITALICS: i64 = 0x0366A;

/// Ein gerastertes Zeichen, Zeile 0 ganz oben.
///
/// **Zwei Kaesten, nicht einer.** SDL_ttf haelt fuer jeden Glyphen zwei
/// Groessen auseinander, und wer das zusammenwirft, liegt bei kursiver
/// Schrift daneben:
///
/// * `sz_*` sind die **Metriken** (`TTF_GlyphMetrics`, `TTF_Size`). Sie
///   kommen aus der Kontur **vor** der Scherung, plus einer pauschalen
///   Zugabe `(GLYPH_ITALICS * font->height) >> 16`.
/// * `TTF_Image` ist das **Bild**, das geblittet wird - aus der gescherten
///   Kontur, also mit der tatsaechlichen Schraege.
///
/// Die beiden weichen voneinander ab; das Bild wird notfalls abgeschnitten.
/// Genau deshalb hat `render(text, ..., italic)` in pygame dieselbe Breite
/// wie `size(text)`, obwohl die Buchstaben schraeg stehen.
#[derive(Debug)]
pub(crate) struct GlyphBild {
    /// Linke Kante des Bildes, von der Schreibmarke aus (FreeType:
    /// `bitmap_left`).
    pub(crate) left: i32,
    /// Obere Kante des Bildes ueber der Grundlinie (FreeType: `bitmap_top`).
    pub(crate) top: i32,
    pub(crate) width: usize,
    pub(crate) rows: usize,
    /// Deckung, ein Byte je Pixel, `width * rows` Eintraege.
    pub(crate) deckung: Vec<u8>,
    /// `sz_left` - fuer Layout und `metrics()`.
    pub(crate) m_left: i32,
    /// `sz_top`.
    pub(crate) m_top: i32,
    /// `sz_width`.
    pub(crate) m_width: i32,
    /// `sz_rows`.
    pub(crate) m_rows: i32,
}

impl GlyphBild {
    fn leer() -> GlyphBild {
        GlyphBild {
            left: 0,
            top: 0,
            width: 0,
            rows: 0,
            deckung: Vec::new(),
            m_left: 0,
            m_top: 0,
            m_width: 0,
            m_rows: 0,
        }
    }

    /// Deckung an (x, y) - ausserhalb 0.
    #[inline]
    fn at(&self, x: usize, y: usize) -> u8 {
        self.deckung[y * self.width + x]
    }
}

/// Rastert ein Zeichen und wendet die nachgemachten Stile an.
///
/// `overhang` ist `font->glyph_overhang` (`ptsize / 10`), `hoehe` ist
/// `font->height` - daraus kommt die Breitenzugabe fuer kursiv.
///
/// ## Drei Quellen, eine Kette
///
/// Das Pixelmuster kann aus dreierlei kommen, und die Reihenfolge ist
/// die von FreeType:
///
/// 1. **ein eingebautes Bild** (`strike`) - wenn die Schrift fuer genau
///    diese Pixelgroesse eines mitbringt und der Stil es erlaubt,
/// 2. **eine Bitmapschrift** (`punkte`) - wenn die Datei gar keine
///    Umrisse hat (BDF),
/// 3. **der Umriss** (`raster`) - der Normalfall.
///
/// Was danach kommt - Kontur, fett, kursiv, Unterstrich -, ist fuer alle
/// drei dasselbe: es arbeitet auf dem fertigen Bild. Genau so macht es
/// SDL_ttf auch, und es ist der Grund, warum `wide` zusammen mit einer
/// Strike etwas anderes ergibt als `wide` mit dem Umriss - der Test
/// `test_freetype_Font_use_bitmap_strikes` prueft beides.
// Neun Werte - die Quelle ist dreigeteilt, und sie zu einem
// Aufzaehlungstyp zu buendeln haette nur den Aufruf verlaengert.
#[allow(clippy::too_many_arguments)]
pub(crate) fn rastern(
    strike: Option<&crate::face::Strikebild>,
    gehintet: Option<&crate::ftgrays::Graubild>,
    gehintet_schraeg: Option<&crate::ftgrays::Graubild>,
    fertig: Option<&FtBild>,
    raster: Option<&fontdue::Font>,
    punkte: Option<&crate::bdf::Bitmapschrift>,
    gid: u16,
    px: f32,
    stil: Stil,
    overhang: i32,
    hoehe: i32,
    mono: bool,
    outline: i32,
) -> GlyphBild {
    // **Einfarbig oder umrandet, auf FreeTypes Weg**: Masse aus dem
    // NORMAL-Kasten beim Laden, Bild aus `ftraster`/`ftgrays` - schon
    // geschert und umrandet. Fett verschmiert SDL_ttf erst danach auf dem
    // fertigen Bild. Die Kontur aendert die Masse **nicht**; das tun
    // `TTF_Size_Internal` und `TTF_GlyphMetrics32` - siehe
    // `Font::vermessen` und `Font::metrics`.
    if let Some(fb) = fertig {
        let (kl, kt, kb, kh) = &fb.kasten;
        let mb = fb;
        let mut m_width = *kb as i32;
        if stil.bold {
            m_width += overhang;
        }
        if stil.italic {
            m_width += ((GLYPH_ITALICS * i64::from(hoehe)) >> 16) as i32;
        }
        let mut bild = if mb.breite == 0 || mb.hoehe == 0 {
            // "FT can make small size glyph of 'width == 0', and 'rows !=
            // 0'. Make sure 'rows' is also 0"
            GlyphBild::leer()
        } else {
            GlyphBild {
                left: mb.left,
                top: mb.top,
                width: mb.breite,
                rows: mb.hoehe,
                deckung: mb.deckung.clone(),
                ..GlyphBild::leer()
            }
        };
        if stil.bold && overhang > 0 && bild.rows > 0 {
            bild = verfetten(&bild, overhang, fb.mono);
        }
        bild.m_left = *kl;
        bild.m_top = *kt;
        bild.m_width = m_width;
        bild.m_rows = *kh as i32;
        return bild;
    }

    // Eine Bitmapschrift bringt ihr Bild mit - es wird nicht gerastert,
    // sondern nur auf die verlangte Groesse gebracht.
    let (m, mut puffer) = match (strike, gehintet, raster, punkte) {
        // `ymin` zaehlt von der Grundlinie nach oben, `top` ist die
        // Oberkante - die Unterkante ist also `top - rows`.
        (Some(k), _, _, _) => (
            Mass::aus(k.width, k.rows, k.left, k.top - k.rows as i32),
            k.deckung.clone(),
        ),
        // FreeTypes eigener Weg - siehe `crate::hinting`. Der Aufrufer
        // reicht ihn nur dort herein, wo er gilt.
        (None, Some(g), _, _) => (
            Mass::aus(g.breite, g.hoehe, g.left, g.top - g.hoehe as i32),
            g.deckung.clone(),
        ),
        (None, None, Some(r), _) => {
            let (m, p) = r.rasterize_indexed(gid, px);
            (Mass::aus(m.width, m.height, m.xmin, m.ymin), p)
        }
        (None, None, None, Some(b)) => aus_punkten(b, gid, px),
        (None, None, None, None) => (Mass::aus(0, 0, 0, 0), Vec::new()),
    };
    let mut bild = if m.width == 0 || m.height == 0 {
        GlyphBild::leer()
    } else {
        if mono {
            // FreeType rastert fuer `antialias=False` einfarbig: ein Pixel
            // zaehlt, wenn sein Mittelpunkt in der Kontur liegt. fontdue
            // liefert nur Flaechendeckung, also wird bei der Haelfte
            // geschnitten.
            //
            // Dabei faellt aber genau das weg, was FreeTypes "dropout
            // control" rettet: ein Stamm, der duenner als ein Pixel ist,
            // deckt nirgends 50 Prozent und verschwaende ganz. Gemessen bei
            // Verdana 8: `render("i", False, ...)` waere leer geblieben,
            // pygame zeichnet dort fuenf volle Pixel. Deshalb wird, wenn
            // nichts uebrig bleibt, bei der Haelfte des **Hoechstwerts**
            // geschnitten.
            let hoechst = puffer.iter().copied().max().unwrap_or(0);
            let grenze = if hoechst >= 128 {
                128
            } else {
                (hoechst / 2).max(1)
            };
            for v in &mut puffer {
                *v = if *v >= grenze { 255 } else { 0 };
            }
        }
        GlyphBild {
            left: m.xmin,
            top: m.ymin + m.height as i32,
            width: m.width,
            rows: m.height,
            deckung: puffer,
            m_left: m.xmin,
            m_top: m.ymin + m.height as i32,
            m_width: m.width as i32,
            m_rows: m.height as i32,
        }
    };

    // Die Kontur wird **vor** allem anderen gezogen: sie ersetzt das
    // Zeichen durch seinen Rand, und fett oder schraeg bezieht sich dann
    // auf diesen Rand. Das ist der **Notweg** fuer Schriften ohne
    // gehinteten Umriss (CFF, Bitmapschriften); der Normalfall kommt
    // fertig umrandet aus `ftstroke` (siehe oben). Die Masse bleiben auch
    // hier unberuehrt - wie bei SDL_ttf.
    if outline > 0 && bild.width > 0 && bild.rows > 0 {
        let (ml, mt, mw, mr) = (bild.m_left, bild.m_top, bild.m_width, bild.m_rows);
        bild = kontur(&bild, outline);
        bild.m_left = ml;
        bild.m_top = mt;
        bild.m_width = mw;
        bild.m_rows = mr;
    }

    // Erst die Metriken festhalten - sie beschreiben die **ungescherte**
    // Kontur, genau wie `cached->sz_*` in `Load_Glyph`.
    let (m_left, m_top, m_rows) = (bild.m_left, bild.m_top, bild.m_rows);
    let mut m_width = bild.m_width;
    if stil.bold {
        m_width += overhang;
    }
    // **Kursiv nur fuer Umrisse.** `Load_Glyph` schert und verbreitert
    // nur bei `slot->format == FT_GLYPH_FORMAT_OUTLINE`; ein eingebautes
    // Bild bleibt gerade. Gemessen an `PyGameMono.otf` bei 8 und 19 (dort
    // liegen seine Bitmaps): pygames kursives Bild ist das normale.
    let schraeg = stil.italic && strike.is_none();
    if schraeg {
        m_width += ((GLYPH_ITALICS * i64::from(hoehe)) >> 16) as i32;
    }

    if schraeg && bild.rows > 0 {
        bild = match gehintet_schraeg {
            // FreeTypes Weg: der gescherte Umriss ist schon gerastert.
            Some(g) => GlyphBild {
                left: g.left,
                top: g.top,
                width: g.breite,
                rows: g.hoehe,
                deckung: g.deckung.clone(),
                ..GlyphBild::leer()
            },
            None => scheren(&bild, mono),
        };
    }
    if stil.bold && overhang > 0 {
        bild = verfetten(&bild, overhang, mono);
    }
    bild.m_left = m_left;
    bild.m_top = m_top;
    bild.m_width = m_width;
    bild.m_rows = m_rows;
    bild
}

/// Kursiv: Zeilen nach rechts schieben, je hoeher desto weiter.
fn scheren(b: &GlyphBild, mono: bool) -> GlyphBild {
    let f = GLYPH_ITALICS as f32 / 65536.0;
    // Mitte der Zeile r liegt bei y = top - r - 0.5 ueber der Grundlinie.
    let dx = |r: usize| f * (b.top as f32 - r as f32 - 0.5);
    let mut min = f32::MAX;
    let mut max = f32::MIN;
    for r in 0..b.rows {
        let d = dx(r);
        min = min.min(d);
        max = max.max(d);
    }
    let basis = min.floor();
    let breite = b.width + (max.ceil() - basis).max(0.0) as usize + 1;
    let mut neu = vec![0u8; breite * b.rows];
    for r in 0..b.rows {
        let off = dx(r) - basis;
        let i = off.floor() as usize;
        let bruch = off - off.floor();
        for x in 0..b.width {
            let v = f32::from(b.at(x, r));
            if v == 0.0 {
                continue;
            }
            let ziel = r * breite + i + x;
            if mono {
                // Einfarbig gibt es keine Zwischenwerte.
                neu[ziel] = 255;
                continue;
            }
            let a = v * (1.0 - bruch);
            let c = v * bruch;
            neu[ziel] = neu[ziel].saturating_add(a.round() as u8);
            neu[ziel + 1] = neu[ziel + 1].saturating_add(c.round() as u8);
        }
    }
    GlyphBild {
        left: b.left + basis as i32,
        top: b.top,
        width: breite,
        rows: b.rows,
        deckung: neu,
        ..GlyphBild::leer()
    }
}

/// Fett: breiter machen und nach rechts verschmieren - die Schleife aus
/// `SDL_ttf.c`, Zeile fuer Zeile.
fn verfetten(b: &GlyphBild, overhang: i32, mono: bool) -> GlyphBild {
    let zu = overhang as usize;
    let breite = b.width + zu;
    let mut neu = vec![0u8; breite * b.rows];
    for r in 0..b.rows {
        for x in 0..b.width {
            neu[r * breite + x] = b.at(x, r);
        }
    }
    for r in 0..b.rows {
        let zeile = &mut neu[r * breite..(r + 1) * breite];
        for _ in 0..zu {
            for col in (1..breite).rev() {
                if mono {
                    zeile[col] |= zeile[col - 1];
                } else {
                    zeile[col] = zeile[col].saturating_add(zeile[col - 1]);
                }
            }
        }
    }
    GlyphBild {
        left: b.left,
        top: b.top,
        width: breite,
        rows: b.rows,
        deckung: neu,
        ..GlyphBild::leer()
    }
}
