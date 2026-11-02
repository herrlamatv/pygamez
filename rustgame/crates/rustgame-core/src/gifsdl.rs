//! gifsdl.rs
//! =========
//! GIF genau so, wie **SDL_image 2.8.8** es liest (`IMG_gif.c`, die
//! LZW-Fassung aus giftopnm, "Adapted for use in SDL by Sam Lantinga").
//!
//! ## Warum nicht die Kiste `gif`
//!
//! Bis Runde 19 las rustgame GIF ueber die Kiste `gif` und baute daraus
//! eine Flaeche in **Bildschirmgroesse**. SDL_image tut etwas anderes:
//!
//! * `image.load` gibt den **ersten Rahmen in Rahmengroesse** zurueck - ein
//!   GIF mit 100x100 Bildschirm und einem ersten Rahmen von 50x50 laedt
//!   als 50x50.
//! * Die Flaeche hat SDLs 256er-Tafel, und nur die ersten `cmapSize`
//!   Eintraege sind gesetzt - **der Rest ist Weiss**, nicht Schwarz.
//! * Ein Rahmen, dessen LZW-Daten mittendrin enden, ist ein **Teilbild**
//!   (der Rest bleibt Index 0) - kein Fehler.
//! * Kaputte Dateien haben SDL_images Saetze (`EOF / read error on image
//!   data`, `bad colormap`, ...).
//! * `load_animation` normalisiert jeden Rahmen auf die Groesse des
//!   **ersten** (`NormalizeFrames`), in ARGB8888 oder RGB888, mit SDLs
//!   Wegwerfregeln - auch dem Fall, dass "auf den vorigen Stand" bei einer
//!   Flaeche mit Alphakanal **mischt** statt kopiert.

use crate::image::{Ergebnis, Fehler};
use crate::surface::Surface;

const MAX_LWZ_BITS: i32 = 12;
const INTERLACE: u8 = 0x40;
const LOCALCOLORMAP: u8 = 0x80;
const DISPOSE_NA: i32 = 0;
const DISPOSE_RESTORE_BACKGROUND: i32 = 2;
const DISPOSE_RESTORE_PREVIOUS: i32 = 3;

/// Ein Lesestrom wie `SDL_RWops`.
struct Strom<'a> {
    d: &'a [u8],
    pos: usize,
}

impl Strom<'_> {
    /// `ReadOK(file, buffer, len)` = `SDL_RWread(file, buffer, len, 1)`.
    fn lesen(&mut self, n: usize) -> Option<&[u8]> {
        if self.pos + n > self.d.len() {
            self.pos = self.d.len();
            return None;
        }
        let s = &self.d[self.pos..self.pos + n];
        self.pos += n;
        Some(s)
    }
}

/// `State_t`.
struct Zustand {
    breite: u32,
    hoehe: u32,
    colormap: [[u8; 256]; 3],
    bitpixel: usize,
    transparent: i32,
    delay_time: i32,
    disposal: i32,
    buf: [u8; 280],
    curbit: i32,
    lastbit: i32,
    done: bool,
    last_byte: i32,
    fresh: bool,
    code_size: i32,
    set_code_size: i32,
    max_code: i32,
    max_code_size: i32,
    firstcode: i32,
    oldcode: i32,
    clear_code: i32,
    end_code: i32,
    table: [[i32; 1 << 12]; 2],
    stack: Vec<i32>,
    zero_data_block: bool,
    /// `IMG_SetError` - der zuletzt gesetzte Satz.
    meldung: Option<&'static str>,
}

/// Ein gelesener Rahmen (`Frame_t`): 8 Bit, Tafel, Lage.
struct Rahmen {
    w: u32,
    h: u32,
    pixel: Vec<u8>,
    tafel: Vec<crate::color::Color>,
    schluessel: Option<u8>,
    x: u32,
    y: u32,
    disposal: i32,
    delay: i32,
}

impl Zustand {
    fn neu() -> Box<Zustand> {
        Box::new(Zustand {
            breite: 0,
            hoehe: 0,
            colormap: [[0; 256]; 3],
            bitpixel: 0,
            transparent: -1,
            delay_time: -1,
            disposal: DISPOSE_NA,
            buf: [0; 280],
            curbit: 0,
            lastbit: 0,
            done: false,
            last_byte: 0,
            fresh: false,
            code_size: 0,
            set_code_size: 0,
            max_code: 0,
            max_code_size: 0,
            firstcode: 0,
            oldcode: 0,
            clear_code: 0,
            end_code: 0,
            table: [[0; 1 << 12]; 2],
            stack: Vec::with_capacity(1 << 13),
            zero_data_block: false,
            meldung: None,
        })
    }

    /// `GetDataBlock`: Laenge oder -1.
    fn datenblock(&mut self, s: &mut Strom, ziel: &mut [u8]) -> i32 {
        let Some(n) = s.lesen(1).map(|b| b[0]) else {
            return -1;
        };
        self.zero_data_block = n == 0;
        if n != 0 {
            let Some(d) = s.lesen(n as usize) else {
                return -1;
            };
            ziel[..n as usize].copy_from_slice(d);
        }
        i32::from(n)
    }

    /// `GetCode`.
    fn code(&mut self, s: &mut Strom, code_size: i32, flag: bool) -> i32 {
        if flag {
            self.curbit = 0;
            self.lastbit = 0;
            self.done = false;
            return 0;
        }
        if self.curbit + code_size >= self.lastbit {
            if self.done {
                if self.curbit >= self.lastbit {
                    self.meldung = Some("ran off the end of my bits");
                }
                return -1;
            }
            // `buf[last_byte - 2]` - vor dem ersten Block zeigt das vor den
            // Puffer; die zwei Bytes werden nie gelesen (`curbit` springt
            // auf 16).
            let lb = self.last_byte;
            self.buf[0] = if lb >= 2 { self.buf[(lb - 2) as usize] } else { 0 };
            self.buf[1] = if lb >= 1 { self.buf[(lb - 1) as usize] } else { 0 };
            let mut block = [0u8; 256];
            let ret = self.datenblock(s, &mut block);
            let count = if ret > 0 {
                self.buf[2..2 + ret as usize].copy_from_slice(&block[..ret as usize]);
                ret
            } else {
                self.done = true;
                0
            };
            self.last_byte = 2 + count;
            self.curbit = (self.curbit - self.lastbit) + 16;
            self.lastbit = (2 + count) * 8;
        }
        let mut ret = 0i32;
        for (j, i) in (self.curbit..self.curbit + code_size).enumerate() {
            let byte = self.buf.get((i / 8) as usize).copied().unwrap_or(0);
            if byte & (1 << (i % 8)) != 0 {
                ret |= 1 << j;
            }
        }
        self.curbit += code_size;
        ret
    }

    /// `LWZReadByte`.
    fn lzw(&mut self, s: &mut Strom, flag: bool, input_code_size: i32) -> i32 {
        if input_code_size > MAX_LWZ_BITS {
            return -1;
        }
        if flag {
            self.set_code_size = input_code_size;
            self.code_size = self.set_code_size + 1;
            self.clear_code = 1 << self.set_code_size;
            self.end_code = self.clear_code + 1;
            self.max_code_size = 2 * self.clear_code;
            self.max_code = self.clear_code + 2;
            self.code(s, 0, true);
            self.fresh = true;
            for i in 0..self.clear_code as usize {
                self.table[0][i] = 0;
                self.table[1][i] = i as i32;
            }
            self.table[1][0] = 0;
            for i in self.clear_code.max(0) as usize..(1 << MAX_LWZ_BITS) {
                self.table[0][i] = 0;
            }
            self.stack.clear();
            return 0;
        } else if self.fresh {
            self.fresh = false;
            loop {
                let c = self.code(s, self.code_size, false);
                self.firstcode = c;
                self.oldcode = c;
                if self.firstcode != self.clear_code {
                    break;
                }
            }
            return self.firstcode;
        }
        if let Some(v) = self.stack.pop() {
            return v;
        }
        loop {
            let mut code = self.code(s, self.code_size, false);
            if code < 0 {
                return code;
            }
            if code == self.clear_code {
                for i in 0..self.clear_code as usize {
                    self.table[0][i] = 0;
                    self.table[1][i] = i as i32;
                }
                for i in self.clear_code as usize..(1 << MAX_LWZ_BITS) {
                    self.table[0][i] = 0;
                    self.table[1][i] = 0;
                }
                self.code_size = self.set_code_size + 1;
                self.max_code_size = 2 * self.clear_code;
                self.max_code = self.clear_code + 2;
                self.stack.clear();
                let c = self.code(s, self.code_size, false);
                self.firstcode = c;
                self.oldcode = c;
                return self.firstcode;
            } else if code == self.end_code {
                if self.zero_data_block {
                    return -2;
                }
                let mut b = [0u8; 260];
                while self.datenblock(s, &mut b) > 0 {}
                return -2;
            }
            let incode = code;
            if code >= self.max_code {
                self.stack.push(self.firstcode);
                code = self.oldcode;
            }
            while code >= self.clear_code {
                if !(0..(1 << MAX_LWZ_BITS)).contains(&code) {
                    self.meldung = Some("invalid LWZ data");
                    return -3;
                }
                self.stack.push(self.table[1][code as usize]);
                if code == self.table[0][code as usize] {
                    self.meldung = Some("circular table entry BIG ERROR");
                    return -3;
                }
                code = self.table[0][code as usize];
                // `stack[(1 << MAX_LWZ_BITS) * 2]` - mehr passt nicht.
                if self.stack.len() >= (1 << MAX_LWZ_BITS) * 2 {
                    self.meldung = Some("invalid LWZ data");
                    return -3;
                }
            }
            if !(0..(1 << MAX_LWZ_BITS)).contains(&code) {
                self.meldung = Some("invalid LWZ data");
                return -4;
            }
            self.firstcode = self.table[1][code as usize];
            self.stack.push(self.firstcode);
            code = self.max_code;
            if code < (1 << MAX_LWZ_BITS) {
                self.table[0][code as usize] = self.oldcode;
                self.table[1][code as usize] = self.firstcode;
                self.max_code += 1;
                if self.max_code >= self.max_code_size && self.max_code_size < (1 << MAX_LWZ_BITS) {
                    self.max_code_size *= 2;
                    self.code_size += 1;
                }
            }
            self.oldcode = incode;
            if let Some(v) = self.stack.pop() {
                return v;
            }
        }
    }

    /// `ReadColorMap`: `false` bei Erfolg.
    fn farbtafel(s: &mut Strom, n: usize, ziel: &mut [[u8; 256]; 3]) -> Result<(), ()> {
        for i in 0..n.min(256) {
            let rgb = s.lesen(3).ok_or(())?;
            for (k, kanal) in ziel.iter_mut().enumerate() {
                kanal[i] = rgb[k];
            }
        }
        Ok(())
    }

    /// `DoExtension`.
    fn erweiterung(&mut self, s: &mut Strom, label: u8) {
        let mut buf = [0u8; 256];
        if label == 0xF9 {
            let _ = self.datenblock(s, &mut buf);
            self.disposal = i32::from((buf[0] >> 2) & 0x7);
            self.delay_time = i32::from(buf[1]) | (i32::from(buf[2]) << 8);
            if buf[0] & 0x1 != 0 {
                self.transparent = i32::from(buf[3]);
            }
        }
        while self.datenblock(s, &mut buf) > 0 {}
    }

    /// `ReadImage` - `None`, wo SDL_image `NULL` gibt.
    fn bild(
        &mut self,
        s: &mut Strom,
        len: u32,
        hoehe: u32,
        cmap_size: usize,
        cmap: &[[u8; 256]; 3],
        interlace: bool,
    ) -> Option<Rahmen> {
        let Some(c) = s.lesen(1).map(|b| b[0]) else {
            self.meldung = Some("EOF / read error on image data");
            return None;
        };
        if self.lzw(s, true, i32::from(c)) < 0 {
            self.meldung = Some("error reading image");
            return None;
        }
        // `SDL_CreateRGBSurfaceWithFormat(0, w, h, 0, INDEX8)` - bei einer
        // Seite null gibt es keine Pixel, und `ReadImage` gibt still `NULL`;
        // ueber 2 GiB scheitert schon das Anlegen (`Out of memory`).
        if len == 0 || hoehe == 0 {
            return None;
        }
        if u64::from(len) * u64::from(hoehe) > i32::MAX as u64 {
            self.meldung = Some("Out of memory");
            return None;
        }
        let mut tafel = vec![crate::color::Color::rgba(255, 255, 255, 255); 256];
        for (i, f) in tafel.iter_mut().enumerate().take(cmap_size.min(256)) {
            *f = crate::color::Color::rgb(cmap[0][i], cmap[1][i], cmap[2][i]);
        }
        let mut pixel = vec![0u8; len as usize * hoehe as usize];
        let (mut xpos, mut ypos, mut pass) = (0u32, 0u32, 0);
        loop {
            let v = self.lzw(s, false, i32::from(c));
            if v < 0 {
                break;
            }
            pixel[(xpos + ypos * len) as usize] = v as u8;
            xpos += 1;
            if xpos == len {
                xpos = 0;
                if interlace {
                    match pass {
                        0 | 1 => ypos += 8,
                        2 => ypos += 4,
                        3 => ypos += 2,
                        _ => {}
                    }
                    if ypos >= hoehe {
                        pass += 1;
                        match pass {
                            1 => ypos = 4,
                            2 => ypos = 2,
                            3 => ypos = 1,
                            _ => break,
                        }
                    }
                } else {
                    ypos += 1;
                }
            }
            if ypos >= hoehe {
                break;
            }
        }
        Some(Rahmen {
            w: len,
            h: hoehe,
            pixel,
            tafel,
            schluessel: None,
            x: 0,
            y: 0,
            disposal: 0,
            delay: 0,
        })
    }
}

/// `IMG_LoadGIF_RW_Internal`.
fn lesen(d: &[u8], alle: bool) -> Ergebnis<Vec<Rahmen>> {
    let mut s = Strom { d, pos: 0 };
    let fehler = |m: &str| Fehler::Format(m.into());
    let kopf = s.lesen(6).ok_or_else(|| fehler("error reading magic number"))?;
    if &kopf[0..3] != b"GIF" {
        return Err(fehler("not a GIF file"));
    }
    if &kopf[3..6] != b"87a" && &kopf[3..6] != b"89a" {
        return Err(fehler("bad version number, not '87a' or '89a'"));
    }
    let mut st = Zustand::neu();
    let b = s.lesen(7).ok_or_else(|| fehler("failed to read screen descriptor"))?.to_vec();
    st.breite = u32::from(b[0]) | (u32::from(b[1]) << 8);
    st.hoehe = u32::from(b[2]) | (u32::from(b[3]) << 8);
    st.bitpixel = 2 << (b[4] & 0x07);
    if b[4] & LOCALCOLORMAP != 0 {
        let mut cm = [[0u8; 256]; 3];
        if Zustand::farbtafel(&mut s, st.bitpixel, &mut cm).is_err() {
            return Err(fehler("error reading global colormap"));
        }
        st.colormap = cm;
    }
    let mut rahmen: Vec<Rahmen> = Vec::new();
    loop {
        let Some(c) = s.lesen(1).map(|b| b[0]) else {
            st.meldung = Some("EOF / read error on image data");
            break;
        };
        if c == b';' {
            break;
        }
        if c == b'!' {
            let Some(label) = s.lesen(1).map(|b| b[0]) else {
                st.meldung = Some("EOF / read error on extension function code");
                break;
            };
            st.erweiterung(&mut s, label);
            continue;
        }
        if c != b',' {
            continue;
        }
        let Some(b) = s.lesen(9).map(|b| b.to_vec()) else {
            st.meldung = Some("couldn't read left/top/width/height");
            break;
        };
        let global = b[8] & LOCALCOLORMAP == 0;
        let bitpixel = 1usize << ((b[8] & 0x07) + 1);
        let w = u32::from(b[4]) | (u32::from(b[5]) << 8);
        let h = u32::from(b[6]) | (u32::from(b[7]) << 8);
        let interlace = b[8] & INTERLACE != 0;
        let bild = if !global {
            let mut lokal = [[0u8; 256]; 3];
            if Zustand::farbtafel(&mut s, bitpixel, &mut lokal).is_err() {
                // `ReadColorMap` setzt "bad colormap", der Aufrufer sofort
                // danach "error reading local colormap".
                st.meldung = Some("error reading local colormap");
                break;
            }
            st.bild(&mut s, w, h, bitpixel, &lokal, interlace)
        } else {
            let cm = st.colormap;
            st.bild(&mut s, w, h, st.bitpixel, &cm, interlace)
        };
        if let Some(mut r) = bild {
            if st.transparent >= 0 {
                r.schluessel = Some(st.transparent as u8);
            }
            r.x = u32::from(b[0]) | (u32::from(b[1]) << 8);
            r.y = u32::from(b[2]) | (u32::from(b[3]) << 8);
            r.disposal = st.disposal;
            r.delay = if st.delay_time < 2 { 100 } else { st.delay_time * 10 };
            rahmen.push(r);
            if !alle {
                break;
            }
        }
    }
    if rahmen.is_empty() {
        // Ohne eigenen Satz bleibt bei SDL der vorige stehen.
        return Err(Fehler::Format(st.meldung.unwrap_or("").into()));
    }
    Ok(rahmen)
}

/// Ein Rahmen als 8-Bit-Flaeche mit SDLs 256er-Tafel.
fn flaeche(r: &Rahmen) -> Surface {
    let mut s = Surface::new((r.w, r.h));
    s.set_format(crate::surface::Format::standard(8, false).expect("8 Bit ist gueltig"));
    s.set_palette(Some(r.tafel.clone()));
    for y in 0..r.h {
        for x in 0..r.w {
            s.setz_gepackt((x as i32, y as i32), u32::from(r.pixel[(y * r.w + x) as usize]));
        }
    }
    if let Some(k) = r.schluessel {
        let c = r.tafel[k as usize];
        s.set_colorkey(Some(crate::color::Color::rgba(c.r, c.g, c.b, 255)));
    }
    s
}

/// `image.load` fuer GIF - `IMG_LoadGIF_RW`: der erste Rahmen.
pub fn laden(d: &[u8]) -> Ergebnis<Surface> {
    let rahmen = lesen(d, false)?;
    Ok(flaeche(&rahmen[0]))
}

/// `image.load_animation` fuer GIF - `IMG_LoadGIFAnimation_RW`.
///
/// Gibt `(Flaeche, Dauer in Millisekunden)`; eine Dauer unter 2
/// Hundertstel wird 100 ms ("matching browsers and Qt").
pub fn bildfolge(d: &[u8]) -> Ergebnis<Vec<(Surface, f64)>> {
    let rahmen = lesen(d, true)?;
    if rahmen.len() == 1 {
        let r = &rahmen[0];
        return Ok(vec![(flaeche(r), f64::from(r.delay))]);
    }
    // `NormalizeFrames`: ARGB8888 mit Schluessel, sonst RGB888, in der
    // Groesse des ersten Rahmens.
    let mit_alpha = rahmen[0].schluessel.is_some();
    let (w, h) = (rahmen[0].w, rahmen[0].h);
    let mut bild = vec![0u32; (w * h) as usize];
    let mut stand: Vec<Vec<u32>> = Vec::with_capacity(rahmen.len());
    let mut last_dispose = DISPOSE_RESTORE_BACKGROUND;
    let mut i_restore = 0usize;
    let (mut rx, mut ry, mut rw, mut rh) = (0u32, 0u32, w, h);
    let mut aus = Vec::with_capacity(rahmen.len());
    for (i, r) in rahmen.iter().enumerate() {
        // Nur der Teil des Rechtecks, der in der Flaeche liegt.
        let zellen = |rx: u32, ry: u32, rw: u32, rh: u32| {
            let x1 = (rx.saturating_add(rw)).min(w);
            let y1 = (ry.saturating_add(rh)).min(h);
            (rx.min(w)..x1).flat_map(move |x| (ry.min(h)..y1).map(move |y| (x, y)))
        };
        match last_dispose {
            DISPOSE_RESTORE_BACKGROUND => {
                for (x, y) in zellen(rx, ry, rw, rh) {
                    bild[(y * w + x) as usize] = 0;
                }
            }
            DISPOSE_RESTORE_PREVIOUS => {
                // Eine ARGB8888-Flaeche mischt beim Blitten
                // (`SDL_BLENDMODE_BLEND`): nur deckende Pixel kommen an.
                let alt = &stand[i_restore];
                for (x, y) in zellen(rx, ry, rw, rh) {
                    let p = alt[(y * w + x) as usize];
                    if !mit_alpha || p >> 24 == 0xFF {
                        bild[(y * w + x) as usize] = p;
                    }
                }
            }
            _ => {}
        }
        if r.disposal != DISPOSE_RESTORE_PREVIOUS {
            i_restore = i;
        }
        (rx, ry, rw, rh) = (r.x, r.y, r.w, r.h);
        for zy in 0..r.h {
            for zx in 0..r.w {
                let (x, y) = (r.x + zx, r.y + zy);
                if x >= w || y >= h {
                    continue;
                }
                let idx = r.pixel[(zy * r.w + zx) as usize];
                if r.schluessel == Some(idx) {
                    continue;
                }
                let c = r.tafel[idx as usize];
                let a: u32 = if mit_alpha { 0xFF } else { 0 };
                bild[(y * w + x) as usize] =
                    (a << 24) | (u32::from(c.r) << 16) | (u32::from(c.g) << 8) | u32::from(c.b);
            }
        }
        stand.push(bild.clone());
        let mut s = if mit_alpha {
            Surface::with_flags((w, h), crate::SRCALPHA)
        } else {
            Surface::new((w, h))
        };
        let quelle = &bild;
        s.mit_pixeln(|ziel, breite, (ox, oy)| {
            for y in 0..h as usize {
                for x in 0..w as usize {
                    let p = quelle[y * w as usize + x];
                    // RGB888: das vierte Byte bleibt 0 (`SDL_MapRGBA` ohne
                    // Alphamaske) - unsichtbar, aber echter Zustand.
                    ziel[(oy + y) * breite + ox + x] = p;
                }
            }
        });
        aus.push((s, f64::from(r.delay)));
        last_dispose = r.disposal;
    }
    Ok(aus)
}
