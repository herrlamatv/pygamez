//! py.rs
//! =====
//! Python-Semantik, wo Rust anders rechnet.
//!
//! Die Spiele sind Zeile fuer Zeile aus Python uebertragen. An vier Stellen
//! rechnet Rust **anders** als Python, und jede davon aendert ein Spiel
//! still, wenn man sie uebersieht:
//!
//! | Python | Rust | hier |
//! |---|---|---|
//! | `a // b` rundet **ab** | `a / b` schneidet zur Null hin ab | [`fdiv`], [`ffdiv`] |
//! | `a % b` hat das Vorzeichen von `b` | `a % b` das von `a` | [`pmod`], [`fpmod`] |
//! | `round(x)` rundet Halbe zur **geraden** Zahl | `x.round()` von der Null weg | [`round`], [`round_to`] |
//! | `random` ist MT19937 | - | [`Random`] |
//!
//! ## Zufall bitgleich zu CPython
//!
//! [`Random`] ist CPythons `_randommodule.c` samt `random.py` (3.14):
//! gleicher Startwert, gleiche Folge - `random()`, `randint`, `choice`,
//! `shuffle`, `uniform`, `sample`, `choices`, `gauss`, `randbytes`. Damit
//! laufen die Rust- und die Python-Fassung eines Spiels mit demselben
//! Startwert **gleich**, und ein Bildvergleich wird moeglich.

use std::cell::RefCell;

// ---------------------------------------------------------------------------
//  Ganzzahl- und Kommaregeln
// ---------------------------------------------------------------------------

/// Pythons `a // b` fuer ganze Zahlen (rundet zur negativen Unendlichkeit).
pub fn fdiv(a: i64, b: i64) -> i64 {
    let q = a / b;
    if (a % b != 0) && ((a < 0) != (b < 0)) {
        q - 1
    } else {
        q
    }
}

/// [`fdiv`] fuer `i32`.
pub fn fdiv32(a: i32, b: i32) -> i32 {
    fdiv(a as i64, b as i64) as i32
}

/// Pythons `a % b` fuer ganze Zahlen (Vorzeichen des Teilers).
pub fn pmod(a: i64, b: i64) -> i64 {
    let r = a % b;
    if r != 0 && ((r < 0) != (b < 0)) {
        r + b
    } else {
        r
    }
}

/// [`pmod`] fuer `i32`.
pub fn pmod32(a: i32, b: i32) -> i32 {
    pmod(a as i64, b as i64) as i32
}

/// Pythons `a // b` fuer Kommazahlen.
pub fn ffdiv(a: f64, b: f64) -> f64 {
    (a / b).floor()
}

/// Pythons `a % b` fuer Kommazahlen - wie `math.fmod`, aber mit dem
/// Vorzeichen des Teilers (CPythons `float_rem`).
pub fn fpmod(a: f64, b: f64) -> f64 {
    let m = a % b;
    if m != 0.0 {
        if (b < 0.0) != (m < 0.0) {
            m + b
        } else {
            m
        }
    } else {
        // Python gibt die Null mit dem Vorzeichen des Teilers.
        0.0_f64.copysign(b)
    }
}

/// Pythons `round(x)` - Halbe zur geraden Zahl ("Bankrundung").
///
/// `round(2.5)` ist in Python `2`, `round(3.5)` ist `4`; Rusts
/// `f64::round` gaebe `3` und `4`.
pub fn round(x: f64) -> i64 {
    x.round_ties_even() as i64
}

/// [`round`] als `i32`.
pub fn roundi(x: f64) -> i32 {
    x.round_ties_even() as i32
}

/// Pythons `round(x, n)` fuer Kommazahlen.
///
/// CPython rundet hier ueber die **exakte** Dezimaldarstellung
/// (`_Py_dg_dtoa`, Modus 3). Der Weg ueber die Formatierung trifft dieselbe
/// Entscheidung: Rusts `{:.n}` rundet ebenfalls die exakte Binaerzahl
/// korrekt, Halbe zur geraden Ziffer.
pub fn round_to(x: f64, n: usize) -> f64 {
    format!("{:.*}", n, x).parse().unwrap_or(x)
}

/// Pythons `int(x)` fuer Kommazahlen - zur Null hin abgeschnitten.
pub fn int(x: f64) -> i64 {
    x as i64
}

/// [`int`] als `i32`.
pub fn inti(x: f64) -> i32 {
    x as i32
}

/// `max(lo, min(hi, v))` - kommt in den Spielen dutzendfach vor.
pub fn clamp<T: PartialOrd>(v: T, lo: T, hi: T) -> T {
    if v < lo {
        lo
    } else if v > hi {
        hi
    } else {
        v
    }
}

/// Pythons `str(x)` fuer eine Kommazahl (`1.0` statt Rusts `1`).
pub fn fstr(x: f64) -> String {
    if x.is_finite() && x == x.trunc() && x.abs() < 1e16 {
        format!("{:.1}", x)
    } else if x.is_nan() {
        "nan".into()
    } else if x.is_infinite() {
        if x > 0.0 { "inf".into() } else { "-inf".into() }
    } else {
        format!("{}", x)
    }
}

/// Pythons `abs()` fuer ganze Zahlen, ohne Ueberlauf bei `MIN`.
pub fn iabs(v: i64) -> i64 {
    v.saturating_abs()
}

/// Pythons `sign`-Muster `(v > 0) - (v < 0)`.
pub fn sign(v: f64) -> f64 {
    if v > 0.0 {
        1.0
    } else if v < 0.0 {
        -1.0
    } else {
        0.0
    }
}

// CPython rechnet `math.hypot` / `math.dist` NICHT mit der C-Bibliothek,
// sondern mit `vector_norm` (Modules/mathmodule.c): verlustfreies Quadrieren
// per fma, kompensierte Summe, Differenzialkorrektur. Das Ergebnis weicht
// bei etwa jedem achten Wertepaar im letzten Bit von `f64::hypot` ab - und
// solche Bits wandern ueber Bewegung und Abstaende ins Bild.

fn dl_fast_sum(a: f64, b: f64) -> (f64, f64) {
    let x = a + b;
    let y = (a - x) + b;
    (x, y)
}

fn dl_mul(x: f64, y: f64) -> (f64, f64) {
    let z = x * y;
    let zz = x.mul_add(y, -z);
    (z, zz)
}

/// Exponent wie `frexp` (Mantisse in [0.5, 1)), fuer endliche x > 0.
fn frexp_exp(x: f64) -> i32 {
    let e = ((x.to_bits() >> 52) & 0x7ff) as i32;
    if e == 0 {
        return frexp_exp(x * f64::from_bits(((64 + 1023) as u64) << 52)) - 64;
    }
    e - 1022
}

/// `ldexp(1.0, k)` exakt, auch im subnormalen Bereich.
fn pow2(k: i32) -> f64 {
    if k >= -1022 {
        f64::from_bits(((k + 1023) as u64) << 52)
    } else {
        f64::from_bits(1u64 << (k + 1074))
    }
}

fn vector_norm(vec: &mut [f64], max: f64, found_nan: bool) -> f64 {
    if max.is_infinite() {
        return max;
    }
    if found_nan {
        return f64::NAN;
    }
    if max == 0.0 || vec.len() <= 1 {
        return max;
    }
    let max_e = frexp_exp(max);
    if max_e < -1023 {
        for v in vec.iter_mut() {
            *v /= f64::MIN_POSITIVE;
        }
        return f64::MIN_POSITIVE * vector_norm(vec, max / f64::MIN_POSITIVE, found_nan);
    }
    let scale = pow2(-max_e);
    let (mut csum, mut frac1, mut frac2) = (1.0f64, 0.0f64, 0.0f64);
    for &v in vec.iter() {
        let x = v * scale;
        let (hi, lo) = dl_mul(x, x);
        let (shi, slo) = dl_fast_sum(csum, hi);
        csum = shi;
        frac1 += lo;
        frac2 += slo;
    }
    let mut h = (csum - 1.0 + (frac1 + frac2)).sqrt();
    let (hi, lo) = dl_mul(-h, h);
    let (shi, slo) = dl_fast_sum(csum, hi);
    csum = shi;
    frac1 += lo;
    frac2 += slo;
    let x = csum - 1.0 + (frac1 + frac2);
    h += x / (2.0 * h);
    h / scale
}

/// `math.hypot(*coords)` - beliebig viele Koordinaten.
pub fn hypot_n(coords: &[f64]) -> f64 {
    let mut v: Vec<f64> = coords.iter().map(|c| c.abs()).collect();
    let mut max = 0.0f64;
    let mut nan = false;
    for &x in &v {
        nan |= x.is_nan();
        if x > max {
            max = x;
        }
    }
    vector_norm(&mut v, max, nan)
}

/// `math.hypot(dx, dy)` - bitgleich mit CPython.
pub fn hypot(dx: f64, dy: f64) -> f64 {
    let mut v = [dx.abs(), dy.abs()];
    let mut max = 0.0f64;
    let mut nan = false;
    for &x in &v {
        nan |= x.is_nan();
        if x > max {
            max = x;
        }
    }
    vector_norm(&mut v, max, nan)
}

/// `math.dist(a, b)` - bitgleich mit CPython.
pub fn dist(a: (f64, f64), b: (f64, f64)) -> f64 {
    hypot(a.0 - b.0, a.1 - b.1)
}

/// Pythons `x ** y` fuer Kommazahlen (`float_pow`) - bitgleich mit CPython.
///
/// CPython ruft `pow()` der C-Bibliothek; MSVCs `pow(x, 2.0)` ist dabei NICHT
/// immer gleich `x * x` (etwa 1 von 1300 Werten). Rusts `powf` mit einem
/// festen Exponenten 2.0 oder 0.5 faltet LLVM aber zu `x * x` bzw. `sqrt` -
/// darum geht der Exponent durch `black_box`, und die Sonderfaelle von
/// `float_pow` (0, 1, negative Basis) stehen davor wie in CPython.
pub fn pow(x: f64, y: f64) -> f64 {
    if y == 0.0 {
        return 1.0;
    }
    if x.is_nan() {
        return x;
    }
    if y.is_nan() {
        return if x == 1.0 { 1.0 } else { y };
    }
    if y.is_infinite() || x.is_infinite() || x == 0.0 {
        return x.powf(std::hint::black_box(y));
    }
    let (mut b, mut neg) = (x, false);
    if b < 0.0 {
        if y != y.floor() {
            // CPython wuerde hier eine komplexe Zahl liefern
            return f64::NAN;
        }
        b = -b;
        neg = (y % 2.0).abs() == 1.0;
    }
    if b == 1.0 {
        return if neg { -1.0 } else { 1.0 };
    }
    let r = std::hint::black_box(b).powf(std::hint::black_box(y));
    if neg {
        -r
    } else {
        r
    }
}

/// Pythons eingebautes `sum()` ueber Kommazahlen.
///
/// Seit 3.12 summiert CPython Floats kompensiert (Neumaier); ein schlichtes
/// `iter().sum()` weicht davon im letzten Bit ab.
pub fn sum(werte: &[f64]) -> f64 {
    let Some((&erste, rest)) = werte.split_first() else {
        return 0.0;
    };
    // 0 + erste: die Summe startet als ganze Zahl 0
    let mut f = 0.0 + erste;
    let mut c = 0.0f64;
    for &x in rest {
        let t = f + x;
        if f.abs() >= x.abs() {
            c += (f - t) + x;
        } else {
            c += (x - t) + f;
        }
        f = t;
    }
    if c != 0.0 && c.is_finite() {
        f += c;
    }
    f
}

/// `math.radians`.
pub fn radians(g: f64) -> f64 {
    // CPython: x * degToRad mit degToRad = pi / 180 (vorab geteilt)
    g * (std::f64::consts::PI / 180.0)
}

/// `math.degrees`.
pub fn degrees(r: f64) -> f64 {
    // CPython: x * radToDeg mit radToDeg = 180 / pi
    r * (180.0 / std::f64::consts::PI)
}

/// `math.tau`.
pub const TAU: f64 = std::f64::consts::TAU;
/// `math.pi`.
pub const PI: f64 = std::f64::consts::PI;

// ---------------------------------------------------------------------------
//  MT19937 - CPythons _randommodule.c
// ---------------------------------------------------------------------------

const N: usize = 624;
const M: usize = 397;
const MATRIX_A: u32 = 0x9908_b0df;
const UPPER_MASK: u32 = 0x8000_0000;
const LOWER_MASK: u32 = 0x7fff_ffff;

/// Pythons `random.Random`.
#[derive(Clone)]
pub struct Random {
    mt: Box<[u32; N]>,
    mti: usize,
    /// `gauss()` erzeugt zwei Werte auf einmal und merkt sich den zweiten.
    gauss_next: Option<f64>,
}

impl std::fmt::Debug for Random {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Random(mti={})", self.mti)
    }
}

impl Random {
    /// `random.Random(seed)` mit einer ganzen Zahl.
    pub fn new(seed: u64) -> Random {
        let mut r = Random { mt: Box::new([0; N]), mti: N + 1, gauss_next: None };
        r.seed(seed);
        r
    }

    /// Mit Zufall aus der Uhr (wie Pythons `os.urandom`-Startwert: jedes
    /// Mal anders).
    pub fn from_entropy() -> Random {
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x5eed);
        // Adresse eines Stapelwerts dazu, damit zwei Instanzen in derselben
        // Nanosekunde trotzdem verschieden starten.
        let marke = 0u8;
        let adr = &marke as *const u8 as u64;
        Random::new(t ^ adr.rotate_left(17) ^ 0x9e37_79b9_7f4a_7c15)
    }

    /// `seed(n)` - `init_by_array` mit den 32-Bit-Woertern von `abs(n)`.
    pub fn seed(&mut self, n: u64) {
        let mut key: Vec<u32> = Vec::new();
        let mut v = n;
        if v == 0 {
            key.push(0);
        }
        while v > 0 {
            key.push((v & 0xffff_ffff) as u32);
            v >>= 32;
        }
        self.init_by_array(&key);
        self.gauss_next = None;
    }

    fn init_genrand(&mut self, s: u32) {
        self.mt[0] = s;
        for i in 1..N {
            let prev = self.mt[i - 1];
            self.mt[i] = 1_812_433_253u32
                .wrapping_mul(prev ^ (prev >> 30))
                .wrapping_add(i as u32);
        }
        self.mti = N;
    }

    fn init_by_array(&mut self, key: &[u32]) {
        self.init_genrand(19_650_218);
        let mut i = 1usize;
        let mut j = 0usize;
        let len = key.len();
        let mut k = N.max(len);
        while k > 0 {
            let prev = self.mt[i - 1];
            self.mt[i] = (self.mt[i] ^ (prev ^ (prev >> 30)).wrapping_mul(1_664_525))
                .wrapping_add(key[j])
                .wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= N {
                self.mt[0] = self.mt[N - 1];
                i = 1;
            }
            if j >= len {
                j = 0;
            }
            k -= 1;
        }
        k = N - 1;
        while k > 0 {
            let prev = self.mt[i - 1];
            self.mt[i] = (self.mt[i] ^ (prev ^ (prev >> 30)).wrapping_mul(1_566_083_941))
                .wrapping_sub(i as u32);
            i += 1;
            if i >= N {
                self.mt[0] = self.mt[N - 1];
                i = 1;
            }
            k -= 1;
        }
        self.mt[0] = 0x8000_0000;
        self.mti = N;
    }

    /// `genrand_uint32` - ein Wort aus dem Generator.
    pub fn next_u32(&mut self) -> u32 {
        if self.mti >= N {
            let mag01 = [0u32, MATRIX_A];
            let mut kk = 0;
            while kk < N - M {
                let y = (self.mt[kk] & UPPER_MASK) | (self.mt[kk + 1] & LOWER_MASK);
                self.mt[kk] = self.mt[kk + M] ^ (y >> 1) ^ mag01[(y & 1) as usize];
                kk += 1;
            }
            while kk < N - 1 {
                let y = (self.mt[kk] & UPPER_MASK) | (self.mt[kk + 1] & LOWER_MASK);
                self.mt[kk] = self.mt[kk + M - N] ^ (y >> 1) ^ mag01[(y & 1) as usize];
                kk += 1;
            }
            let y = (self.mt[N - 1] & UPPER_MASK) | (self.mt[0] & LOWER_MASK);
            self.mt[N - 1] = self.mt[M - 1] ^ (y >> 1) ^ mag01[(y & 1) as usize];
            self.mti = 0;
        }
        let mut y = self.mt[self.mti];
        self.mti += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^= y >> 18;
        y
    }

    /// `random()` - eine Zahl in `[0, 1)` mit 53 Bit.
    pub fn random(&mut self) -> f64 {
        let a = (self.next_u32() >> 5) as f64;
        let b = (self.next_u32() >> 6) as f64;
        (a * 67_108_864.0 + b) * (1.0 / 9_007_199_254_740_992.0)
    }

    /// `getrandbits(k)` fuer `k <= 64`.
    pub fn getrandbits(&mut self, k: u32) -> u64 {
        if k == 0 {
            return 0;
        }
        if k <= 32 {
            return (self.next_u32() >> (32 - k)) as u64;
        }
        // Mehrere Woerter, das niedrigste zuerst (CPythons wordarray).
        let lo = self.next_u32() as u64;
        let rest = k - 32;
        let hi = (self.next_u32() >> (32 - rest)) as u64;
        lo | (hi << 32)
    }

    /// `_randbelow(n)` - `getrandbits` mit Verwerfen.
    pub fn randbelow(&mut self, n: u64) -> u64 {
        if n == 0 {
            return 0;
        }
        let k = 64 - n.leading_zeros();
        let mut r = self.getrandbits(k);
        while r >= n {
            r = self.getrandbits(k);
        }
        r
    }

    /// `randrange(start, stop)` (Schritt 1).
    pub fn randrange(&mut self, start: i64, stop: i64) -> i64 {
        let width = stop - start;
        if width <= 0 {
            // Python wirft hier ValueError; ein Spiel darf nicht abstuerzen.
            return start;
        }
        start + self.randbelow(width as u64) as i64
    }

    /// `randrange(start, stop, step)`.
    pub fn randrange_step(&mut self, start: i64, stop: i64, step: i64) -> i64 {
        if step == 1 {
            return self.randrange(start, stop);
        }
        let width = stop - start;
        let n = if step > 0 {
            (width + step - 1).div_euclid(step)
        } else {
            (width + step + 1).div_euclid(step)
        };
        if n <= 0 {
            return start;
        }
        start + step * self.randbelow(n as u64) as i64
    }

    /// `randrange(stop)`.
    pub fn randrange0(&mut self, stop: i64) -> i64 {
        self.randrange(0, stop)
    }

    /// `randint(a, b)` - beide Grenzen eingeschlossen.
    pub fn randint(&mut self, a: i64, b: i64) -> i64 {
        self.randrange(a, b + 1)
    }

    /// `uniform(a, b)`.
    pub fn uniform(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.random()
    }

    /// `triangular(low, high, mode)`.
    pub fn triangular(&mut self, low: f64, high: f64, mode: Option<f64>) -> f64 {
        let mut u = self.random();
        let mut c = match mode {
            None => 0.5,
            Some(m) => {
                if high == low {
                    return low;
                }
                (m - low) / (high - low)
            }
        };
        let (mut lo, mut hi) = (low, high);
        if u > c {
            u = 1.0 - u;
            c = 1.0 - c;
            std::mem::swap(&mut lo, &mut hi);
        }
        lo + (hi - lo) * (u * c).sqrt()
    }

    /// `choice(seq)` - der Index.
    pub fn choice_index(&mut self, len: usize) -> usize {
        self.randbelow(len as u64) as usize
    }

    /// `choice(seq)`.
    pub fn choice<T: Clone>(&mut self, seq: &[T]) -> T {
        let i = self.choice_index(seq.len());
        seq[i].clone()
    }

    /// `choice(seq)` als Verweis.
    pub fn choice_ref<'a, T>(&mut self, seq: &'a [T]) -> &'a T {
        let i = self.choice_index(seq.len());
        &seq[i]
    }

    /// `shuffle(x)` - Fisher-Yates von hinten.
    pub fn shuffle<T>(&mut self, x: &mut [T]) {
        if x.len() < 2 {
            return;
        }
        for i in (1..x.len()).rev() {
            let j = self.randbelow(i as u64 + 1) as usize;
            x.swap(i, j);
        }
    }

    /// `sample(population, k)` - beide Zweige von `random.py`.
    pub fn sample<T: Clone>(&mut self, population: &[T], k: usize) -> Vec<T> {
        let n = population.len();
        let k = k.min(n);
        let mut setsize = 21usize;
        if k > 5 {
            // 4 ** ceil(log(k * 3, 4))
            let e = ((k as f64 * 3.0).ln() / 4f64.ln()).ceil() as u32;
            setsize += 4usize.pow(e);
        }
        let mut out = Vec::with_capacity(k);
        if n <= setsize {
            let mut pool: Vec<T> = population.to_vec();
            for i in 0..k {
                let j = self.randbelow((n - i) as u64) as usize;
                out.push(pool[j].clone());
                pool[j] = pool[n - i - 1].clone();
            }
        } else {
            let mut selected = std::collections::HashSet::new();
            for _ in 0..k {
                let mut j = self.randbelow(n as u64) as usize;
                while selected.contains(&j) {
                    j = self.randbelow(n as u64) as usize;
                }
                selected.insert(j);
                out.push(population[j].clone());
            }
        }
        out
    }

    /// `choices(population, k=k)` ohne Gewichte.
    pub fn choices<T: Clone>(&mut self, population: &[T], k: usize) -> Vec<T> {
        let n = population.len() as f64;
        (0..k)
            .map(|_| population[(self.random() * n).floor() as usize].clone())
            .collect()
    }

    /// `choices(population, weights, k=1)[0]` - der gezogene Index.
    pub fn weighted_index(&mut self, weights: &[f64]) -> usize {
        let mut cum = Vec::with_capacity(weights.len());
        let mut s = 0.0;
        for w in weights {
            s += w;
            cum.push(s);
        }
        let total = s;
        let x = self.random() * total;
        // bisect_right(cum, x, 0, hi) mit hi = n - 1
        let hi = weights.len().saturating_sub(1);
        let (mut lo, mut hi) = (0usize, hi);
        while lo < hi {
            let mid = (lo + hi) / 2;
            if x < cum[mid] {
                hi = mid;
            } else {
                lo = mid + 1;
            }
        }
        lo
    }

    /// `gauss(mu, sigma)`.
    pub fn gauss(&mut self, mu: f64, sigma: f64) -> f64 {
        let z = match self.gauss_next.take() {
            Some(z) => z,
            None => {
                let x2pi = self.random() * TAU;
                let g2rad = (-2.0 * (1.0 - self.random()).ln()).sqrt();
                self.gauss_next = Some(x2pi.sin() * g2rad);
                x2pi.cos() * g2rad
            }
        };
        mu + z * sigma
    }

    /// `randbytes(n)` - `getrandbits(n*8).to_bytes(n, 'little')`.
    pub fn randbytes(&mut self, n: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(n);
        let mut rest = n;
        while rest >= 4 {
            out.extend_from_slice(&self.next_u32().to_le_bytes());
            rest -= 4;
        }
        if rest > 0 {
            let bits = (rest * 8) as u32;
            let w = self.next_u32() >> (32 - bits);
            out.extend_from_slice(&w.to_le_bytes()[..rest]);
        }
        out
    }
}

// ---------------------------------------------------------------------------
//  Das Modul `random` selbst - eine Instanz je Strang
// ---------------------------------------------------------------------------

thread_local! {
    static GLOBAL: RefCell<Random> = RefCell::new(Random::from_entropy());
    static FEST: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Pruefstand: auch `random.Random()` ohne Startwert wird wiederholbar.
///
/// Python saet eine solche Instanz aus `os.urandom`; zum Vergleichen leitet
/// `rust/tools/vergleich.py` sie stattdessen aus dem globalen Generator ab
/// (`getrandbits(64)`), und mit diesem Schalter tut Rust dasselbe.
pub fn set_deterministic(an: bool) {
    FEST.with(|f| f.set(an));
}

/// Laeuft der Pruefstand (feste Uhr, wiederholbarer Zufall)?
pub fn deterministic() -> bool {
    FEST.with(|f| f.get())
}

// ---------------------------------------------------------------------------
//  Das Modul `time` - Wanduhr der Spiele
// ---------------------------------------------------------------------------
//
// Einige Spiele begrenzen ihre KI mit einem Zeitbudget (`time.time()`,
// `time.perf_counter()`). Im Pruefstand laeuft diese Uhr - wie in
// `rust/tools/vergleich.py` - mit der festen Bild-Uhr (`ticks / 1000`) mit,
// sonst haengen die Ergebnisse vom Rechnertempo ab.

/// `time.time()` - Sekunden seit 1970 (Pruefstand: feste Uhr).
pub fn time_time() -> f64 {
    if deterministic() {
        return crate::pg::ticks() as f64 / 1000.0;
    }
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// `time.perf_counter()` / `time.monotonic()` (Pruefstand: feste Uhr).
pub fn perf_counter() -> f64 {
    if deterministic() {
        return crate::pg::ticks() as f64 / 1000.0;
    }
    thread_local! {
        static START: std::time::Instant = std::time::Instant::now();
    }
    START.with(|s| s.elapsed().as_secs_f64())
}

impl Random {
    /// `random.Random()` ohne Startwert.
    pub fn unseeded() -> Random {
        if FEST.with(|f| f.get()) {
            let s = with(|g| g.getrandbits(64));
            Random::new(s)
        } else {
            Random::from_entropy()
        }
    }
}

/// Mit dem globalen Generator arbeiten (`random.xyz(...)` in Python).
pub fn with<R>(f: impl FnOnce(&mut Random) -> R) -> R {
    GLOBAL.with(|g| f(&mut g.borrow_mut()))
}

/// `random.seed(n)` - fuer den Pruefstand.
pub fn seed(n: u64) {
    with(|r| r.seed(n));
}

/// `random.random()`.
pub fn random() -> f64 {
    with(|r| r.random())
}

/// `random.randint(a, b)`.
pub fn randint(a: i64, b: i64) -> i64 {
    with(|r| r.randint(a, b))
}

/// `random.randint(a, b)` als `i32`.
pub fn randint32(a: i32, b: i32) -> i32 {
    with(|r| r.randint(a as i64, b as i64)) as i32
}

/// `random.randrange(a, b)`.
pub fn randrange(a: i64, b: i64) -> i64 {
    with(|r| r.randrange(a, b))
}

/// `random.randrange(n)`.
pub fn randrange0(n: i64) -> i64 {
    with(|r| r.randrange0(n))
}

/// `random.uniform(a, b)`.
pub fn uniform(a: f64, b: f64) -> f64 {
    with(|r| r.uniform(a, b))
}

/// `random.choice(seq)`.
pub fn choice<T: Clone>(seq: &[T]) -> T {
    with(|r| r.choice(seq))
}

/// `random.choice(seq)` - nur der Index.
pub fn choice_index(len: usize) -> usize {
    with(|r| r.choice_index(len))
}

/// `random.shuffle(x)`.
pub fn shuffle<T>(x: &mut [T]) {
    with(|r| r.shuffle(x))
}

/// `random.sample(pop, k)`.
pub fn sample<T: Clone>(pop: &[T], k: usize) -> Vec<T> {
    with(|r| r.sample(pop, k))
}

/// `random.gauss(mu, sigma)`.
pub fn gauss(mu: f64, sigma: f64) -> f64 {
    with(|r| r.gauss(mu, sigma))
}

/// `random.choices(range, weights)[0]` - der Index.
pub fn weighted_index(weights: &[f64]) -> usize {
    with(|r| r.weighted_index(weights))
}

/// `random.triangular(lo, hi, mode)`.
pub fn triangular(lo: f64, hi: f64, mode: Option<f64>) -> f64 {
    with(|r| r.triangular(lo, hi, mode))
}

// ---------------------------------------------------------------- set(int)

/// `hash(i)` einer Python-Ganzzahl (fuer |i| < 2**61 - 1).
fn int_hash(i: i64) -> i64 {
    const MOD: i64 = (1 << 61) - 1;
    let h = if i >= 0 { i % MOD } else { -((-i) % MOD) };
    if h == -1 {
        -2
    } else {
        h
    }
}

/// `hash((a, b, ...))` fuer ein Tupel aus Ganzzahlen - CPythons `tuplehash`
/// (xxHash-Variante, `Objects/tupleobject.c`).
pub fn tuple_hash(items: &[i64]) -> i64 {
    const P1: u64 = 11_400_714_785_074_694_791;
    const P2: u64 = 14_029_467_366_897_019_727;
    const P5: u64 = 2_870_177_450_012_600_261;
    let mut acc: u64 = P5;
    for &it in items {
        let lane = int_hash(it) as u64;
        acc = acc.wrapping_add(lane.wrapping_mul(P2));
        acc = acc.rotate_left(31);
        acc = acc.wrapping_mul(P1);
    }
    acc = acc.wrapping_add((items.len() as u64) ^ (P5 ^ 3_527_539));
    if acc == u64::MAX {
        return 1_546_275_796;
    }
    acc as i64
}

/// Hash einer Rasterzelle `(x, y)` wie `hash((x, y))` in Python.
pub fn cell_hash(c: &(i32, i32)) -> i64 {
    tuple_hash(&[c.0 as i64, c.1 as i64])
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum PSlot<K: Copy + PartialEq> {
    Leer,
    Geloescht,
    Wert(K, i64),
}

/// Python-`set` mit beliebigem Schluessel und **derselben Iterationsfolge**
/// wie CPython - wie [`IntSet`], aber mit einer Hash-Funktion (z.B.
/// [`cell_hash`] fuer Zellen-Tupel).
#[derive(Clone, Debug)]
pub struct PySet<K: Copy + PartialEq> {
    table: Vec<PSlot<K>>,
    fill: usize,
    used: usize,
    hash: fn(&K) -> i64,
}

/// Menge von Rasterzellen `(x, y)` mit Pythons Reihenfolge.
pub type CellSet = PySet<(i32, i32)>;

impl CellSet {
    /// `set()` fuer Zellen.
    pub fn cells() -> CellSet {
        PySet::new(cell_hash)
    }
}

impl<K: Copy + PartialEq> PySet<K> {
    /// `set()` mit der Hash-Funktion `hash`.
    pub fn new(hash: fn(&K) -> i64) -> PySet<K> {
        PySet { table: vec![PSlot::Leer; 8], fill: 0, used: 0, hash }
    }

    fn mask(&self) -> usize {
        self.table.len() - 1
    }

    /// `len(s)`
    pub fn len(&self) -> usize {
        self.used
    }

    /// `not s`
    pub fn is_empty(&self) -> bool {
        self.used == 0
    }

    fn finde(&self, key: &K) -> Option<usize> {
        let hash = (self.hash)(key);
        let mask = self.mask();
        let mut i = (hash as usize) & mask;
        let mut perturb = hash as usize;
        loop {
            let probes = if i + LINEAR_PROBES <= mask { LINEAR_PROBES } else { 0 };
            for j in 0..=probes {
                match self.table[i + j] {
                    PSlot::Leer => return None,
                    PSlot::Wert(v, h) if h == hash && v == *key => return Some(i + j),
                    _ => {}
                }
            }
            perturb >>= PERTURB_SHIFT;
            i = (i.wrapping_mul(5).wrapping_add(1).wrapping_add(perturb)) & mask;
        }
    }

    /// `x in s`
    pub fn contains(&self, key: &K) -> bool {
        self.finde(key).is_some()
    }

    /// `s.add(x)`
    pub fn add(&mut self, key: K) {
        let hash = (self.hash)(&key);
        let mask = self.mask();
        let mut i = (hash as usize) & mask;
        let mut perturb = hash as usize;
        let mut freeslot: Option<usize> = None;
        let ziel;
        'suche: loop {
            let probes = if i + LINEAR_PROBES <= mask { LINEAR_PROBES } else { 0 };
            for j in 0..=probes {
                match self.table[i + j] {
                    PSlot::Leer => {
                        ziel = i + j;
                        break 'suche;
                    }
                    PSlot::Wert(v, h) if h == hash && v == key => return,
                    PSlot::Geloescht => freeslot = Some(i + j),
                    _ => {}
                }
            }
            perturb >>= PERTURB_SHIFT;
            i = (i.wrapping_mul(5).wrapping_add(1).wrapping_add(perturb)) & mask;
        }
        if let Some(f) = freeslot {
            self.used += 1;
            self.table[f] = PSlot::Wert(key, hash);
            return;
        }
        self.fill += 1;
        self.used += 1;
        self.table[ziel] = PSlot::Wert(key, hash);
        if self.fill * 5 < mask * 3 {
            return;
        }
        let minused = if self.used > 50000 { self.used * 2 } else { self.used * 4 };
        let mut newsize = 8usize;
        while newsize <= minused {
            newsize <<= 1;
        }
        let alt = std::mem::replace(&mut self.table, vec![PSlot::Leer; newsize]);
        self.fill = self.used;
        let mask = newsize - 1;
        for slot in alt {
            if let PSlot::Wert(v, h) = slot {
                let mut i = (h as usize) & mask;
                let mut perturb = h as usize;
                'frei: loop {
                    if self.table[i] == PSlot::Leer {
                        self.table[i] = PSlot::Wert(v, h);
                        break 'frei;
                    }
                    if i + LINEAR_PROBES <= mask {
                        for j in 1..=LINEAR_PROBES {
                            if self.table[i + j] == PSlot::Leer {
                                self.table[i + j] = PSlot::Wert(v, h);
                                break 'frei;
                            }
                        }
                    }
                    perturb >>= PERTURB_SHIFT;
                    i = (i.wrapping_mul(5).wrapping_add(1).wrapping_add(perturb)) & mask;
                }
            }
        }
    }

    /// `s.discard(x)` - hinterlaesst wie CPython einen Platzhalter.
    pub fn discard(&mut self, key: &K) {
        if let Some(i) = self.finde(key) {
            self.table[i] = PSlot::Geloescht;
            self.used -= 1;
        }
    }

    /// `for x in s` (Tabellen-Reihenfolge).
    pub fn iter(&self) -> impl Iterator<Item = K> + '_ {
        self.table.iter().filter_map(|s| if let PSlot::Wert(v, _) = s { Some(*v) } else { None })
    }

    /// `list(s)`
    pub fn to_vec(&self) -> Vec<K> {
        self.iter().collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Slot {
    Leer,
    Geloescht,
    Wert(i64),
}

/// Python-`set` aus Ganzzahlen mit **derselben Iterationsreihenfolge** wie
/// CPython (`Objects/setobject.c`: lineares Sondieren ueber 9 Nachbarn,
/// dann Stoerung `i*5 + 1 + perturb`, Vergroesserung bei 3/5 Fuellung).
///
/// Spiele, die `list(meine_menge)` oder `for x in meine_menge` nutzen und
/// beim Gleichstand den ersten Treffer nehmen, brauchen genau diese Folge.
#[derive(Clone, Debug)]
pub struct IntSet {
    table: Vec<Slot>,
    fill: usize,
    used: usize,
}

impl Default for IntSet {
    fn default() -> Self {
        IntSet::new()
    }
}

const LINEAR_PROBES: usize = 9;
const PERTURB_SHIFT: u32 = 5;

impl IntSet {
    /// `set()`
    pub fn new() -> IntSet {
        IntSet { table: vec![Slot::Leer; 8], fill: 0, used: 0 }
    }

    /// `set(iterable)` - fuegt in der Reihenfolge des Iterables ein.
    pub fn from_iter_py(werte: impl IntoIterator<Item = i64>) -> IntSet {
        let mut s = IntSet::new();
        for w in werte {
            s.add(w);
        }
        s
    }

    fn mask(&self) -> usize {
        self.table.len() - 1
    }

    /// `len(s)`
    pub fn len(&self) -> usize {
        self.used
    }

    /// `not s`
    pub fn is_empty(&self) -> bool {
        self.used == 0
    }

    fn finde(&self, key: i64) -> Option<usize> {
        let hash = int_hash(key);
        let mask = self.mask();
        let mut i = (hash as usize) & mask;
        let mut perturb = hash as usize;
        loop {
            let probes = if i + LINEAR_PROBES <= mask { LINEAR_PROBES } else { 0 };
            for j in 0..=probes {
                match self.table[i + j] {
                    Slot::Leer => return None,
                    Slot::Wert(v) if v == key => return Some(i + j),
                    _ => {}
                }
            }
            perturb >>= PERTURB_SHIFT;
            i = (i.wrapping_mul(5).wrapping_add(1).wrapping_add(perturb)) & mask;
        }
    }

    /// `x in s`
    pub fn contains(&self, key: i64) -> bool {
        self.finde(key).is_some()
    }

    /// `s.add(x)`
    pub fn add(&mut self, key: i64) {
        let hash = int_hash(key);
        let mask = self.mask();
        let mut i = (hash as usize) & mask;
        let mut perturb = hash as usize;
        let mut freeslot: Option<usize> = None;
        let ziel;
        'suche: loop {
            let probes = if i + LINEAR_PROBES <= mask { LINEAR_PROBES } else { 0 };
            for j in 0..=probes {
                match self.table[i + j] {
                    Slot::Leer => {
                        ziel = i + j;
                        break 'suche;
                    }
                    Slot::Wert(v) if v == key => return,
                    // CPython merkt sich den *letzten* Platzhalter.
                    Slot::Geloescht => freeslot = Some(i + j),
                    _ => {}
                }
            }
            perturb >>= PERTURB_SHIFT;
            i = (i.wrapping_mul(5).wrapping_add(1).wrapping_add(perturb)) & mask;
        }
        if let Some(f) = freeslot {
            self.used += 1;
            self.table[f] = Slot::Wert(key);
            return;
        }
        self.fill += 1;
        self.used += 1;
        self.table[ziel] = Slot::Wert(key);
        if self.fill * 5 < mask * 3 {
            return;
        }
        let minused = if self.used > 50000 { self.used * 2 } else { self.used * 4 };
        self.resize(minused);
    }

    fn resize(&mut self, minused: usize) {
        let mut newsize = 8usize;
        while newsize <= minused {
            newsize <<= 1;
        }
        let alt = std::mem::replace(&mut self.table, vec![Slot::Leer; newsize]);
        self.fill = self.used;
        let mask = newsize - 1;
        for slot in alt {
            if let Slot::Wert(v) = slot {
                let hash = int_hash(v);
                let mut i = (hash as usize) & mask;
                let mut perturb = hash as usize;
                'frei: loop {
                    if self.table[i] == Slot::Leer {
                        self.table[i] = Slot::Wert(v);
                        break 'frei;
                    }
                    if i + LINEAR_PROBES <= mask {
                        for j in 1..=LINEAR_PROBES {
                            if self.table[i + j] == Slot::Leer {
                                self.table[i + j] = Slot::Wert(v);
                                break 'frei;
                            }
                        }
                    }
                    perturb >>= PERTURB_SHIFT;
                    i = (i.wrapping_mul(5).wrapping_add(1).wrapping_add(perturb)) & mask;
                }
            }
        }
    }

    /// `s.discard(x)` - hinterlaesst wie CPython einen Platzhalter.
    pub fn discard(&mut self, key: i64) {
        if let Some(i) = self.finde(key) {
            self.table[i] = Slot::Geloescht;
            self.used -= 1;
        }
    }

    /// `list(s)` - in CPythons Tabellen-Reihenfolge.
    pub fn to_vec(&self) -> Vec<i64> {
        self.iter().collect()
    }

    /// `for x in s`
    pub fn iter(&self) -> impl Iterator<Item = i64> + '_ {
        self.table.iter().filter_map(|s| if let Slot::Wert(v) = s { Some(*v) } else { None })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Die Werte stammen aus CPython 3.14:
    /// `r = random.Random(42); [r.random() for _ in range(3)]`.
    #[test]
    fn random_42_wie_cpython() {
        let mut r = Random::new(42);
        let a = r.random();
        let b = r.random();
        let c = r.random();
        assert_eq!(a, 0.6394267984578837);
        assert_eq!(b, 0.025010755222666936);
        assert_eq!(c, 0.27502931836911926);
    }

    /// `random.Random(1); [r.randint(1, 6) for _ in range(5)]`
    #[test]
    fn randint_1_wie_cpython() {
        let mut r = Random::new(1);
        let v: Vec<i64> = (0..5).map(|_| r.randint(1, 6)).collect();
        assert_eq!(v, vec![2, 5, 1, 3, 1]);
    }

    /// Referenzwerte aus CPython 3.14.4 (venv des Projekts):
    /// `r = random.Random(7); r.randbytes(7).hex(), r.getrandbits(40)` und
    /// die Folge fuer Startwert 3.
    #[test]
    fn weitere_funktionen_wie_cpython() {
        let mut r = Random::new(7);
        let hex: String = r.randbytes(7).iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(hex, "38b4e6524da7f2");
        assert_eq!(r.getrandbits(40), 434439589175);

        let mut r = Random::new(3);
        let mut l: Vec<i32> = (0..10).collect();
        r.shuffle(&mut l);
        assert_eq!(l, vec![1, 5, 6, 0, 9, 4, 7, 2, 8, 3]);
        assert_eq!(r.choice(&['a', 'b', 'c', 'd', 'e', 'f']), 'd');
        assert_eq!(r.uniform(2.0, 5.0), 2.778062042984023);
        let pop: Vec<i32> = (0..100).collect();
        assert_eq!(r.sample(&pop, 7), vec![29, 24, 91, 60, 69, 70, 50]);
        assert_eq!(r.gauss(0.0, 1.0), -0.3667904699805292);
        assert_eq!(r.gauss(0.0, 1.0), -0.4381250344027819);

        // Startwert ueber 32 Bit: zwei Schluesselwoerter.
        let mut r = Random::new((1u64 << 40) + 5);
        assert_eq!(r.random(), 0.5043802970418443);

        // choices(range(5), weights=[1,2,3,4,5], k=3)
        let mut r = Random::new(9);
        let w = [1.0, 2.0, 3.0, 4.0, 5.0];
        let v: Vec<usize> = (0..3).map(|_| r.weighted_index(&w)).collect();
        assert_eq!(v, vec![3, 2, 1]);
    }

    #[test]
    fn rechenregeln_wie_python() {
        assert_eq!(fdiv(-7, 2), -4);
        assert_eq!(pmod(-7, 2), 1);
        assert_eq!(pmod(7, -2), -1);
        assert_eq!(round(2.5), 2);
        assert_eq!(round(3.5), 4);
        assert_eq!(round(-2.5), -2);
        assert_eq!(fpmod(-1.0, 3.0), 2.0);
        assert_eq!(fstr(1.0), "1.0");
        assert_eq!(fstr(0.1), "0.1");
    }

    #[test]
    fn menge_wie_cpython() {
        // Erwartet aus CPython 3.14: list(s) nach den Einfuegungen.
        let s = IntSet::from_iter_py([12, 4]);
        assert_eq!(s.to_vec(), vec![12, 4]);
        let s = IntSet::from_iter_py([40, 8, 72, 1, 33, 65, 2]);
        assert_eq!(s.to_vec(), vec![1, 33, 65, 2, 40, 8, 72]);
        let s = IntSet::from_iter_py((0..60).rev().map(|i| (i * 37) % 81));
        assert_eq!(s.to_vec(), vec![0, 1, 2, 3, 4, 6, 8, 9, 10, 11, 13, 15, 16, 17, 18, 20, 22, 23, 24, 25, 27, 29, 30, 31, 32, 34, 36, 37, 38, 39, 40, 41, 43, 45, 46, 47, 48, 50, 52, 53, 54, 55, 57, 59, 60, 61, 62, 64, 66, 67, 68, 69, 71, 73, 74, 75, 76, 77, 78, 80]);
        let mut s = IntSet::from_iter_py([5, 13, 21, 29]);
        s.discard(13);
        s.add(37);
        s.add(13);
        assert_eq!(s.to_vec(), vec![37, 5, 13, 21, 29]);
    }
}
