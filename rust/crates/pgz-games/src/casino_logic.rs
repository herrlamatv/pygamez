//! casino_logic.rs
//! ===============
//! Spielregeln des Casinos ohne Grafik - Roulette-Tisch und Lama-Slot.
//! Uebertragen aus `games/casino_logic.py`: gleiche Kesselreihenfolge,
//! gleiche Wettarten, gleiche Walzenstreifen und Gewinntabelle.
//!
//! Roulette-Wetten sind Schluessel wie `"plein:17"`, `"cheval:17-20"`,
//! `"column:0"`, `"red"`; Einsaetze liegen als geordnete Liste
//! `(schluessel, betrag)` vor (Python: Woerterbuch in Einfuege-Reihenfolge).

use pgz_engine::py;

// ============================================================= ROULETTE

/// Echte Reihenfolge der Faecher auf dem europaeischen Kessel.
pub const WHEEL_ORDER: [i64; 37] = [
    0, 32, 15, 19, 4, 21, 2, 25, 17, 34, 6, 27, 13, 36, 11, 30, 8, 23, 10, 5, 24, 16, 33, 1, 20,
    14, 31, 9, 22, 18, 29, 7, 28, 12, 35, 3, 26,
];
/// Die roten Zahlen.
pub const RED_NUMBERS: [i64; 18] = [1, 3, 5, 7, 9, 12, 14, 16, 18, 19, 21, 23, 25, 27, 30, 32, 34, 36];
/// Jetonwerte.
pub const CHIP_VALUES: [i64; 5] = [1, 5, 25, 100, 500];
/// Laenge des Zahlenverlaufs.
pub const HISTORY_LEN: usize = 12;

/// Tischbreite in Einheiten.
pub const TABLE_W: f64 = 14.0;
/// Hoehe der Dutzend-Reihe.
pub const DOZEN_H: f64 = 0.8;
/// Hoehe der einfachen Chancen.
pub const OUTSIDE_H: f64 = 0.8;
/// Tischhoehe in Einheiten.
pub const TABLE_H: f64 = 3.0 + DOZEN_H + OUTSIDE_H;
/// So nah an einer Linie zaehlt ein Klick als Kante/Ecke.
pub const EDGE: f64 = 0.26;
/// Die sechs einfachen Chancen in Tischreihenfolge.
pub const OUTSIDE_KEYS: [&str; 6] = ["low", "even", "red", "black", "odd", "high"];

/// Auszahlung "zu 1" je Wettart.
pub fn payout_of(kind: &str) -> Option<i64> {
    Some(match kind {
        "plein" => 35,
        "cheval" => 17,
        "trans" => 11,
        "carre" => 8,
        "sixain" => 5,
        "column" | "dozen" => 2,
        "red" | "black" | "even" | "odd" | "low" | "high" => 1,
        _ => return None,
    })
}

/// `color_of(n)` - "green", "red" oder "black".
pub fn color_of(n: i64) -> &'static str {
    if n == 0 {
        "green"
    } else if RED_NUMBERS.contains(&n) {
        "red"
    } else {
        "black"
    }
}

/// Spalte (0..11) einer Zahl 1..36.
pub fn num_col(n: i64) -> i64 {
    py::fdiv(n - 1, 3)
}

/// Reihe (0 = oben .. 2 = unten) einer Zahl 1..36.
pub fn num_row(n: i64) -> i64 {
    2 - py::pmod(n - 1, 3)
}

/// Zahl in Spalte `col` und Reihe `row`.
pub fn cell_number(col: i64, row: i64) -> i64 {
    3 * col + 3 - row
}

/// `make_key(kind, numbers)` - Zahlen werden sortiert.
pub fn make_key(kind: &str, numbers: &[i64]) -> String {
    let mut v = numbers.to_vec();
    v.sort_unstable();
    let parts: Vec<String> = v.iter().map(|n| n.to_string()).collect();
    format!("{kind}:{}", parts.join("-"))
}

/// `make_key("column"/"dozen", n)`
pub fn make_key_n(kind: &str, n: i64) -> String {
    format!("{kind}:{n}")
}

/// `bet_kind(key)`
pub fn bet_kind(key: &str) -> &str {
    key.split(':').next().unwrap_or("")
}

fn arg_of(key: &str) -> &str {
    key.split_once(':').map(|(_, a)| a).unwrap_or("")
}

/// `bet_numbers(key)` - alle abgedeckten Zahlen (aufsteigend, ohne
/// Doppelte); `None`, wenn der Schluessel nicht lesbar ist.
pub fn bet_numbers(key: &str) -> Option<Vec<i64>> {
    let kind = bet_kind(key);
    let arg = arg_of(key);
    let mut out: Vec<i64> = match kind {
        "column" => {
            let r: i64 = arg.trim().parse().ok()?;
            (0..12).map(|c| cell_number(c, r)).collect()
        }
        "dozen" => {
            let d: i64 = arg.trim().parse().ok()?;
            (1 + 12 * d..13 + 12 * d).collect()
        }
        "red" => RED_NUMBERS.to_vec(),
        "black" => (1..37).filter(|n| !RED_NUMBERS.contains(n)).collect(),
        "even" => (2..37).step_by(2).collect(),
        "odd" => (1..37).step_by(2).collect(),
        "low" => (1..19).collect(),
        "high" => (19..37).collect(),
        _ => {
            let mut v = Vec::new();
            for x in arg.split('-') {
                v.push(x.trim().parse::<i64>().ok()?);
            }
            v
        }
    };
    out.sort_unstable();
    out.dedup();
    Some(out)
}

/// `bet_numbers(key)` fuer sichere Schluessel.
pub fn numbers(key: &str) -> Vec<i64> {
    bet_numbers(key).unwrap_or_default()
}

/// `make_key_of(key)`
pub fn make_key_of(key: &str) -> Option<String> {
    let kind = bet_kind(key);
    let arg = arg_of(key);
    if kind == "column" || kind == "dozen" {
        return if ["0", "1", "2"].contains(&arg) { Some(key.to_string()) } else { None };
    }
    if OUTSIDE_KEYS.contains(&kind) {
        return if arg.is_empty() { Some(key.to_string()) } else { None };
    }
    Some(make_key(kind, &bet_numbers(key)?))
}

/// `valid_key(key)` - fuer geladene/wiederholte Einsaetze.
pub fn valid_key(key: &str) -> bool {
    let kind = bet_kind(key);
    let Some(nums) = bet_numbers(key) else {
        return false;
    };
    let Some(pay) = payout_of(kind) else {
        return false;
    };
    if nums.is_empty() || !nums.iter().all(|n| (0..=36).contains(n)) {
        return false;
    }
    36 / nums.len() as i64 - 1 == pay && make_key_of(key).as_deref() == Some(key)
}

/// `payout_multiplier(key)` - Gewinn zu 1.
pub fn payout_multiplier(key: &str) -> i64 {
    payout_of(bet_kind(key)).unwrap_or(0)
}

/// `settle(bets, number)` -> (Auszahlung, Gewinner-Schluessel).
pub fn settle(bets: &[(String, i64)], number: i64) -> (i64, Vec<String>) {
    let mut total = 0;
    let mut winners = Vec::new();
    for (key, amount) in bets {
        if *amount > 0 && numbers(key).contains(&number) {
            total += amount * (payout_multiplier(key) + 1);
            winners.push(key.clone());
        }
    }
    (total, winners)
}

fn street(col: i64) -> String {
    make_key("trans", &[3 * col + 1, 3 * col + 2, 3 * col + 3])
}

/// `hit_test(ux, uy)` - welche Wette liegt an dieser Tischstelle?
pub fn hit_test(ux: f64, uy: f64) -> Option<String> {
    if !((0.0..TABLE_W).contains(&ux) && (0.0..TABLE_H).contains(&uy)) {
        return None;
    }
    if uy < 3.0 {
        if ux >= 13.0 {
            return Some(make_key_n("column", uy as i64));
        }
        if ux < 1.0 - EDGE {
            return Some(make_key("plein", &[0]));
        }
        let mut col = 11.min(0.max((ux - 1.0) as i64));
        let row = 2.min(uy as i64);
        let mut fx = ux - 1.0 - col as f64;
        let fy = uy - row as f64;
        if ux < 1.0 {
            col = 0;
            fx = 0.0;
        }
        let (left, right) = (fx < EDGE, fx > 1.0 - EDGE);
        let (top, bottom) = (fy < EDGE, fy > 1.0 - EDGE);
        let n = cell_number(col, row);
        let vedge = left || right;
        let hedge = top || bottom;
        if vedge && hedge {
            let (c1, c2) = if left { (col - 1, col) } else { (col, col + 1) };
            let rows = if top && row > 0 {
                Some((row - 1, row))
            } else if bottom && row < 2 {
                Some((row, row + 1))
            } else {
                None
            };
            if let Some((r0, r1)) = rows {
                if c1 < 0 {
                    return Some(make_key("trans", &[0, cell_number(0, r0), cell_number(0, r1)]));
                }
                if c2 > 11 {
                    return Some(make_key("cheval", &[cell_number(col, r0), cell_number(col, r1)]));
                }
                let nums: Vec<i64> = [c1, c2].iter().flat_map(|c| [r0, r1].map(|r| cell_number(*c, r))).collect();
                return Some(make_key("carre", &nums));
            }
            if c1 < 0 {
                return Some(make_key("carre", &[0, 1, 2, 3]));
            }
            if c2 > 11 {
                return Some(street(col));
            }
            let nums: Vec<i64> = (3 * c1 + 1..3 * c2 + 4).collect();
            return Some(make_key("sixain", &nums));
        }
        if vedge {
            if left && col == 0 {
                return Some(make_key("cheval", &[0, n]));
            }
            if right && col == 11 {
                return Some(make_key("plein", &[n]));
            }
            let other = cell_number(if left { col - 1 } else { col + 1 }, row);
            return Some(make_key("cheval", &[n, other]));
        }
        if hedge {
            if top && row > 0 {
                return Some(make_key("cheval", &[n, cell_number(col, row - 1)]));
            }
            if bottom && row < 2 {
                return Some(make_key("cheval", &[n, cell_number(col, row + 1)]));
            }
            return Some(street(col));
        }
        return Some(make_key("plein", &[n]));
    }
    if !(1.0..13.0).contains(&ux) {
        return None;
    }
    if uy < 3.0 + DOZEN_H {
        return Some(make_key_n("dozen", 2.min(((ux - 1.0) / 4.0) as i64)));
    }
    Some(OUTSIDE_KEYS[5.min(((ux - 1.0) / 2.0) as i64) as usize].to_string())
}

/// `anchor(key)` - Mittelpunkt des Chip-Stapels in Tisch-Einheiten.
pub fn anchor(key: &str) -> (f64, f64) {
    let kind = bet_kind(key);
    let arg = || arg_of(key).parse::<i64>().unwrap_or(0);
    if kind == "column" {
        return (13.5, arg() as f64 + 0.5);
    }
    if kind == "dozen" {
        return (1.0 + 4.0 * arg() as f64 + 2.0, 3.0 + DOZEN_H / 2.0);
    }
    if let Some(i) = OUTSIDE_KEYS.iter().position(|k| *k == kind) {
        return (1.0 + 2.0 * i as f64 + 1.0, 3.0 + DOZEN_H + OUTSIDE_H / 2.0);
    }
    let nums = numbers(key);
    if nums.is_empty() {
        return (0.5, 1.5);
    }
    if kind == "plein" {
        let n = nums[0];
        if n == 0 {
            return (0.5, 1.5);
        }
        return (num_col(n) as f64 + 1.5, num_row(n) as f64 + 0.5);
    }
    if nums.contains(&0) {
        let rest = &nums[1..];
        if nums.len() == 4 {
            return (1.0, 3.0);
        }
        let rows: Vec<i64> = rest.iter().map(|n| num_row(*n)).collect();
        if rest.len() == 1 {
            return (1.0, rows[0] as f64 + 0.5);
        }
        return (1.0, *rows.iter().max().unwrap_or(&0) as f64);
    }
    let mut cols: Vec<i64> = nums.iter().map(|n| num_col(*n)).collect();
    cols.sort_unstable();
    cols.dedup();
    let mut rows: Vec<i64> = nums.iter().map(|n| num_row(*n)).collect();
    rows.sort_unstable();
    rows.dedup();
    let x = if cols.len() == 1 { cols[0] as f64 + 1.5 } else { cols[1] as f64 + 1.0 };
    if kind == "trans" || kind == "sixain" {
        return (x, 3.0);
    }
    let y = if rows.len() == 1 { rows[0] as f64 + 0.5 } else { rows[1] as f64 };
    (x, y)
}

// ============================================================= LAMA-SLOT

/// Walzen.
pub const REELS: usize = 5;
/// Reihen.
pub const ROWS: usize = 3;
/// Wild-Symbol.
pub const WILD: &str = "lama";
/// Scatter-Symbol.
pub const SCATTER: &str = "coin";
/// Alle Symbole (Gewinntabellen-Reihenfolge).
pub const SYMBOLS: [&str; 9] = ["lama", "coin", "seven", "gem", "bell", "clover", "grape", "lemon", "cherry"];
/// Einsaetze je Linie.
pub const LINE_BETS: [i64; 4] = [1, 2, 5, 10];
/// Auto-Spin-Anzahlen.
pub const AUTO_COUNTS: [i64; 2] = [10, 25];
/// Freispiele je Ausloesung.
pub const FREE_SPINS: i64 = 10;
/// Gewinnfaktor in Freispielen.
pub const FREE_MULT: i64 = 2;
/// Auszahlungsquote fuer die Gewinntabelle (siehe [`exact_rtp`]).
pub const RTP_TEXT: &str = "96.1";

/// Gewinn je Linie (x Linien-Einsatz) fuer 3/4/5 gleiche.
pub fn pays(sym: &str) -> Option<[i64; 3]> {
    Some(match sym {
        "lama" => [20, 100, 750],
        "seven" => [8, 30, 120],
        "gem" => [6, 20, 80],
        "bell" => [4, 12, 50],
        "clover" => [3, 10, 30],
        "grape" => [2, 5, 20],
        "lemon" => [1, 4, 15],
        "cherry" => [1, 4, 12],
        _ => return None,
    })
}

/// Scatter-Gewinn (x Gesamteinsatz) fuer 3/4/5 Muenzen.
pub fn scatter_pays(n: usize) -> i64 {
    match n {
        3 => 2,
        4 => 10,
        5 => 50,
        _ => 0,
    }
}

/// Die 10 Gewinnlinien: Reihe je Walze.
pub const LINES: [[usize; 5]; 10] = [
    [1, 1, 1, 1, 1],
    [0, 0, 0, 0, 0],
    [2, 2, 2, 2, 2],
    [0, 1, 2, 1, 0],
    [2, 1, 0, 1, 2],
    [0, 0, 1, 2, 2],
    [2, 2, 1, 0, 0],
    [1, 0, 0, 0, 1],
    [1, 2, 2, 2, 1],
    [1, 0, 1, 2, 1],
];

/// Walzenstreifen (je 32 Felder, zyklisch).
pub const STRIPS: [[&str; 32]; 5] = [
    [
        "cherry", "lama", "grape", "bell", "lemon", "gem", "cherry", "clover", "seven", "grape", "lemon", "lama",
        "cherry", "bell", "grape", "coin", "lemon", "clover", "gem", "cherry", "lama", "grape", "bell", "lemon",
        "seven", "clover", "cherry", "gem", "grape", "lama", "lemon", "bell",
    ],
    [
        "lemon", "lama", "cherry", "gem", "grape", "bell", "lama", "clover", "cherry", "seven", "lemon", "grape",
        "lama", "bell", "cherry", "clover", "coin", "grape", "gem", "lemon", "lama", "cherry", "bell", "seven",
        "grape", "clover", "lemon", "gem", "lama", "cherry", "grape", "bell",
    ],
    [
        "grape", "cherry", "lama", "bell", "lemon", "gem", "lama", "grape", "clover", "seven", "cherry", "lemon",
        "lama", "bell", "grape", "gem", "coin", "cherry", "clover", "lemon", "lama", "grape", "bell", "seven",
        "cherry", "clover", "lemon", "gem", "lama", "grape", "cherry", "bell",
    ],
    [
        "cherry", "grape", "lama", "gem", "lemon", "bell", "lama", "cherry", "clover", "seven", "grape", "lemon",
        "lama", "bell", "cherry", "gem", "coin", "grape", "clover", "lemon", "lama", "cherry", "bell", "seven",
        "grape", "clover", "lemon", "gem", "lama", "cherry", "grape", "bell",
    ],
    [
        "lemon", "cherry", "lama", "grape", "bell", "gem", "lemon", "clover", "seven", "cherry", "grape", "lama",
        "lemon", "bell", "cherry", "coin", "grape", "clover", "gem", "lemon", "lama", "cherry", "bell", "grape",
        "seven", "clover", "lemon", "gem", "cherry", "lama", "grape", "bell",
    ],
];

/// Sichtbares Walzenbild: `[walze][reihe]`.
pub type Window = [[&'static str; ROWS]; REELS];

/// `window(stops)`
pub fn window(stops: &[i64; REELS]) -> Window {
    let mut out = [[""; ROWS]; REELS];
    for (r, stop) in stops.iter().enumerate() {
        let strip = &STRIPS[r];
        for (row, cell) in out[r].iter_mut().enumerate() {
            *cell = strip[py::pmod(stop + row as i64, strip.len() as i64) as usize];
        }
    }
    out
}

/// `random_stops(rng)` - ein Stopp je Walze (`rng.randrange`).
pub fn random_stops(rng: &mut py::Random) -> [i64; REELS] {
    let mut out = [0; REELS];
    for (r, o) in out.iter_mut().enumerate() {
        *o = rng.randrange(0, STRIPS[r].len() as i64);
    }
    out
}

/// `line_result(symbols)` -> (Symbol, Anzahl, Multiplikator).
pub fn line_result(symbols: &[&'static str]) -> (Option<&'static str>, i64, i64) {
    let mut wild_run = 0;
    while wild_run < REELS && symbols[wild_run] == WILD {
        wild_run += 1;
    }
    let lama = pays(WILD).unwrap_or([0; 3]);
    let mut best = (Some(WILD), wild_run as i64, if wild_run >= 3 { lama[wild_run - 3] } else { 0 });
    let base = symbols.iter().copied().find(|s| *s != WILD);
    let Some(base) = base.filter(|b| *b != SCATTER) else {
        return if best.2 > 0 { best } else { (None, 0, 0) };
    };
    let mut n = 0;
    for s in symbols {
        if *s == base || *s == WILD {
            n += 1;
        } else {
            break;
        }
    }
    let mult = if n >= 3 { pays(base).map(|p| p[n - 3]).unwrap_or(0) } else { 0 };
    if mult > best.2 {
        best = (Some(base), n as i64, mult);
    }
    if best.2 > 0 {
        best
    } else {
        (None, 0, 0)
    }
}

/// Ergebnis einer Slot-Drehung (`evaluate`).
#[derive(Clone, Debug)]
pub struct SlotRes {
    /// (Linie, Symbol, Anzahl, Gewinn)
    pub lines: Vec<(usize, &'static str, i64, i64)>,
    /// Anzahl Muenzen.
    pub scatter: usize,
    /// Lage der Muenzen (Walze, Reihe).
    pub coins: Vec<(usize, usize)>,
    /// Scatter-Gewinn.
    pub scatter_win: i64,
    /// Gesamtgewinn.
    pub total: i64,
    /// 5 Lamas auf einer Linie.
    pub jackpot: bool,
    /// Neu gewonnene Freispiele.
    pub free_spins: i64,
}

/// `evaluate(win, line_bet, free)`
pub fn evaluate(win: &Window, line_bet: i64, free: bool) -> SlotRes {
    let mult = if free { FREE_MULT } else { 1 };
    let mut lines = Vec::new();
    let mut total = 0;
    let mut jackpot = false;
    for (i, line) in LINES.iter().enumerate() {
        let syms: Vec<&'static str> = (0..REELS).map(|r| win[r][line[r]]).collect();
        let (sym, n, m) = line_result(&syms);
        if m > 0 {
            let amount = m * line_bet * mult;
            let sym = sym.unwrap_or(WILD);
            lines.push((i, sym, n, amount));
            total += amount;
            if sym == WILD && n == REELS as i64 {
                jackpot = true;
            }
        }
    }
    let mut coins = Vec::new();
    for (r, col) in win.iter().enumerate() {
        for (row, s) in col.iter().enumerate() {
            if *s == SCATTER {
                coins.push((r, row));
            }
        }
    }
    let scatter_win = scatter_pays(coins.len()) * line_bet * LINES.len() as i64 * mult;
    total += scatter_win;
    SlotRes { lines, scatter: coins.len(), free_spins: if coins.len() >= 3 { FREE_SPINS } else { 0 }, coins, scatter_win, total, jackpot }
}

/// Ergebnis von [`exact_rtp`].
#[derive(Clone, Copy, Debug)]
pub struct Rtp {
    /// Gesamtquote.
    pub rtp: f64,
    /// Anteil der Linien.
    pub line: f64,
    /// Anteil des Scatters.
    pub scatter: f64,
    /// Wahrscheinlichkeit, Freispiele auszuloesen.
    pub trigger: f64,
    /// Erwartete Freispiele je Ausloesung.
    pub free_spins: f64,
    /// Wahrscheinlichkeit von 5 Lamas auf einer Linie.
    pub jackpot_line: f64,
}

/// `exact_rtp()` - Auszahlungsquote exakt aus den Walzenstreifen.
pub fn exact_rtp() -> Rtp {
    let mut probs: Vec<Vec<(&'static str, f64)>> = Vec::new();
    for strip in STRIPS.iter() {
        let mut counts: Vec<(&'static str, usize)> = Vec::new();
        for s in strip {
            match counts.iter_mut().find(|c| c.0 == *s) {
                Some(c) => c.1 += 1,
                None => counts.push((s, 1)),
            }
        }
        counts.sort_by(|a, b| a.0.cmp(b.0));
        probs.push(counts.iter().map(|(s, c)| (*s, *c as f64 / strip.len() as f64)).collect());
    }
    let mut line = 0.0;
    let mut idx = [0usize; REELS];
    loop {
        let mut p = 1.0;
        let mut syms = [""; REELS];
        for r in 0..REELS {
            let (s, q) = probs[r][idx[r]];
            p *= q;
            syms[r] = s;
        }
        let mult = line_result(&syms).2;
        if mult != 0 {
            line += p * mult as f64;
        }
        let mut r = REELS;
        loop {
            if r == 0 {
                break;
            }
            r -= 1;
            idx[r] += 1;
            if idx[r] < probs[r].len() {
                break;
            }
            idx[r] = 0;
            if r == 0 {
                r = usize::MAX;
                break;
            }
        }
        if r == usize::MAX {
            break;
        }
    }
    let mut dist = vec![1.0f64];
    for strip in STRIPS.iter() {
        let n = strip.len();
        let mut per = [0.0f64; ROWS + 1];
        for stop in 0..n {
            let k = (0..ROWS).filter(|row| strip[(stop + row) % n] == SCATTER).count();
            per[k] += 1.0 / n as f64;
        }
        let mut new = vec![0.0f64; dist.len() + ROWS];
        for (a, pa) in dist.iter().enumerate() {
            for (b, pb) in per.iter().enumerate() {
                new[a + b] += pa * pb;
            }
        }
        dist = new;
    }
    let scatter: f64 = py::sum(&dist.iter().enumerate().map(|(k, p)| p * scatter_pays(k) as f64).collect::<Vec<_>>());
    let trigger: f64 = py::sum(&dist.iter().enumerate().filter(|(k, _)| *k >= 3).map(|(_, p)| *p).collect::<Vec<_>>());
    let base = line + scatter;
    let exp_free = FREE_SPINS as f64 / (1.0 - FREE_SPINS as f64 * trigger);
    let rtp = base + trigger * exp_free * FREE_MULT as f64 * base;
    let mut jackpot_line = 1.0;
    for strip in STRIPS.iter() {
        jackpot_line *= strip.iter().filter(|s| **s == WILD).count() as f64 / strip.len() as f64;
    }
    Rtp { rtp, line, scatter, trigger, free_spins: exp_free, jackpot_line }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rtp_passt_zum_text() {
        let r = exact_rtp();
        assert_eq!(format!("{:.1}", r.rtp * 100.0), RTP_TEXT);
    }

    #[test]
    fn treffer_und_anker() {
        assert_eq!(hit_test(0.3, 1.5).as_deref(), Some("plein:0"));
        assert_eq!(hit_test(1.5, 0.5).as_deref(), Some("plein:3"));
        assert_eq!(hit_test(2.0, 0.5).as_deref(), Some("cheval:3-6"));
        assert_eq!(hit_test(2.0, 1.0).as_deref(), Some("carre:2-3-5-6"));
        assert_eq!(hit_test(1.0, 1.0).as_deref(), Some("trans:0-2-3"));
        assert_eq!(hit_test(13.5, 2.5).as_deref(), Some("column:2"));
        assert_eq!(hit_test(5.5, 3.4).as_deref(), Some("dozen:1"));
        assert_eq!(hit_test(5.5, 4.2).as_deref(), Some("red"));
        assert_eq!(anchor("carre:2-3-5-6"), (2.0, 1.0));
        assert!(valid_key("sixain:1-2-3-4-5-6"));
        // valid_key prueft Form und Quote, nicht die Nachbarschaft (wie Python).
        assert!(valid_key("cheval:1-5"));
        assert!(!valid_key("cheval:5-1"));
        assert!(!valid_key("plein:37"));
        let (pay, win) = settle(&[("red".into(), 5), ("plein:3".into(), 1), ("black".into(), 2)], 3);
        assert_eq!((pay, win), (10 + 36, vec!["red".to_string(), "plein:3".to_string()]));
    }

    #[test]
    fn linien() {
        assert_eq!(line_result(&["lama", "lama", "lama", "seven", "gem"]), (Some("seven"), 4, 30));
        assert_eq!(line_result(&["lama", "lama", "lama", "coin", "gem"]), (Some("lama"), 3, 20));
        assert_eq!(line_result(&["lama", "seven", "seven", "seven", "seven"]), (Some("seven"), 5, 120));
        assert_eq!(line_result(&["coin", "coin", "coin", "lama", "lama"]), (None, 0, 0));
    }
}
