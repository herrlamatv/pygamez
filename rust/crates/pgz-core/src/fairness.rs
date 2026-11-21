//! fairness.rs
//! ===========
//! Nachpruefbarer Zufall ("Provably Fair") fuer Lama Mines und Lama Plinko -
//! uebertragen aus `fairness.py` (Gegenstueck im Browser:
//! `web/js/games/fairness.js`).
//!
//! * Vor jeder Runde steht ein geheimer **Server-Seed** fest (32 Hex-Zeichen),
//!   angezeigt wird nur sein **Pruefcode** `sha256_hex(server)`.
//! * Dazu kommen der **Client-Seed** des Spielers und die **Nonce**.
//! * Rundenzufall: `seedrand::Rand::new(seed_from(&[spiel, server, client,
//!   nonce]))` - in Python, JS und Rust bitgleich.
//!
//! SHA-256 ist hier selbst geschrieben (keine fremde Kiste, Bau offline).
//! Neue Seeds zieht das Spiel aus seinem `py::Random` - pgz-core kennt die
//! Engine nicht, deshalb nehmen die Funktionen eine Closure, die
//! `getrandbits(k)` liefert:
//!
//! ```ignore
//! let mut state = fairness::blank_state(&mut |k| self.rng.getrandbits(k));
//! let entry = fairness::begin(&state);
//! fairness::advance(&mut state, &mut |k| self.rng.getrandbits(k));
//! ```

use serde_json::{Map, Value};

use crate::seedrand::{seed_from, Rand};

/// Zeichen des Client-Seeds (ohne verwechselbare l/o/0/1), genau 32 Stueck.
pub const CLIENT_ALPHABET: &str = "abcdefghijkmnpqrstuvwxyz23456789";
/// Laenge eines neuen Client-Seeds.
pub const CLIENT_LEN: usize = 8;
/// Hoechste Laenge eines gespeicherten Client-Seeds.
pub const CLIENT_MAX: usize = 16;
/// Eintraege im Fairness-Fenster.
pub const LOG: usize = 12;
/// Groesste Nonce (bleibt in JS-Zahlen exakt).
pub const MAX_NONCE: i64 = 9_007_199_254_740_991;

// ---------------------------------------------------------------- SHA-256

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98,
    0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786,
    0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8,
    0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819,
    0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a,
    0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
    0xc67178f2,
];

/// SHA-256 ueber Bytes (FIPS 180-4).
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] =
        [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
    let mut msg = data.to_vec();
    let bits = (data.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bits.to_be_bytes());
    let mut w = [0u32; 64];
    for chunk in msg.chunks(64) {
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[i * 4], chunk[i * 4 + 1], chunk[i * 4 + 2], chunk[i * 4 + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (x, y) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *x = x.wrapping_add(y);
        }
    }
    let mut out = [0u8; 32];
    for (i, v) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&v.to_be_bytes());
    }
    out
}

/// `sha256_hex(text)` - SHA-256 eines Textes (UTF-8) als 64 kleine Hex-Zeichen.
pub fn sha256_hex(text: &str) -> String {
    sha256(text.as_bytes()).iter().map(|b| format!("{:02x}", b)).collect()
}

/// `commit(server)` - Pruefcode eines Server-Seeds (wird VOR der Runde gezeigt).
pub fn commit(server: &str) -> String {
    sha256_hex(server)
}

// ---------------------------------------------------------------- Seeds

/// `new_server_seed(rng)` - `"%016x%016x" % (getrandbits(64), getrandbits(64))`.
pub fn new_server_seed(getrandbits: &mut impl FnMut(u32) -> u64) -> String {
    let a = getrandbits(64);
    let b = getrandbits(64);
    format!("{:016x}{:016x}", a, b)
}

/// `new_client_seed(rng)` - 8 x `CLIENT_ALPHABET[randrange(32)]`.
///
/// Pythons `randrange(32)` ist `_randbelow(32)`: `getrandbits(6)`, bis der
/// Wert unter 32 liegt - genau so hier, damit dieselben Zahlen verbraucht
/// werden.
pub fn new_client_seed(getrandbits: &mut impl FnMut(u32) -> u64) -> String {
    let abc = CLIENT_ALPHABET.as_bytes();
    let mut s = String::with_capacity(CLIENT_LEN);
    for _ in 0..CLIENT_LEN {
        let mut r = getrandbits(6);
        while r >= 32 {
            r = getrandbits(6);
        }
        s.push(abc[r as usize] as char);
    }
    s
}

fn is_hex(v: &str, n: usize) -> bool {
    v.len() == n && v.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Ganze Zahl wie Pythons `isinstance(v, int)` (ohne bool, ohne float).
pub fn py_int(v: Option<&Value>) -> Option<i64> {
    match v {
        Some(Value::Number(n)) if n.is_i64() || n.is_u64() => n.as_i64(),
        _ => None,
    }
}

/// Server-Seed gueltig? (genau 32 kleine Hex-Zeichen)
pub fn valid_server(v: &str) -> bool {
    is_hex(v, 32)
}

/// Client-Seed gueltig? (1-16 Zeichen aus `CLIENT_ALPHABET`)
pub fn valid_client(v: &str) -> bool {
    let n = v.chars().count();
    (1..=CLIENT_MAX).contains(&n) && v.chars().all(|c| CLIENT_ALPHABET.contains(c))
}

/// Nonce gueltig? (0 .. MAX_NONCE)
pub fn valid_nonce(v: i64) -> bool {
    (0..=MAX_NONCE).contains(&v)
}

/// Pruefcode gueltig? (genau 64 kleine Hex-Zeichen)
pub fn valid_hash(v: &str) -> bool {
    is_hex(v, 64)
}

fn str_of(v: Option<&Value>) -> Option<&str> {
    v.and_then(|x| x.as_str())
}

/// Fairness-Zustand `{"server", "client", "nonce"}`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub server: String,
    pub client: String,
    pub nonce: i64,
}

impl State {
    /// Als JSON (Reihenfolge wie das Python-dict).
    pub fn to_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("server".into(), Value::from(self.server.clone()));
        m.insert("client".into(), Value::from(self.client.clone()));
        m.insert("nonce".into(), Value::from(self.nonce));
        Value::Object(m)
    }
}

/// `blank_state(rng)` - neuer Server- und Client-Seed, Nonce 0.
pub fn blank_state(getrandbits: &mut impl FnMut(u32) -> u64) -> State {
    let server = new_server_seed(getrandbits);
    let client = new_client_seed(getrandbits);
    State { server, client, nonce: 0 }
}

/// `clean_state(raw, rng)` - Feld fuer Feld pruefen (ungueltig -> neu bzw. 0).
pub fn clean_state(raw: Option<&Value>, getrandbits: &mut impl FnMut(u32) -> u64) -> State {
    let obj = raw.and_then(|v| v.as_object());
    let get = |k: &str| obj.and_then(|o| o.get(k));
    let server = match str_of(get("server")) {
        Some(s) if valid_server(s) => s.to_string(),
        _ => new_server_seed(getrandbits),
    };
    let client = match str_of(get("client")) {
        Some(c) if valid_client(c) => c.to_string(),
        _ => new_client_seed(getrandbits),
    };
    let nonce = match py_int(get("nonce")) {
        Some(n) if valid_nonce(n) => n,
        _ => 0,
    };
    State { server, client, nonce }
}

/// `round_rand(game, server, client, nonce)` - Zufall einer Runde.
pub fn round_rand(game: &str, server: &str, client: &str, nonce: i64) -> Rand {
    Rand::new(seed_from(&[game, server, client, &nonce.to_string()]))
}

/// Eintrag einer Runde `{"n", "s", "c", "h"}` (aus `begin`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub n: i64,
    pub s: String,
    pub c: String,
    pub h: String,
}

impl Entry {
    /// Als JSON (Reihenfolge wie das Python-dict).
    pub fn to_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("n".into(), Value::from(self.n));
        m.insert("s".into(), Value::from(self.s.clone()));
        m.insert("c".into(), Value::from(self.c.clone()));
        m.insert("h".into(), Value::from(self.h.clone()));
        Value::Object(m)
    }

    /// `entry_rand(game, entry)` - Zufall dieser Runde.
    pub fn rand(&self, game: &str) -> Rand {
        round_rand(game, &self.s, &self.c, self.n)
    }

    /// `verify(entry)` - stimmt der Pruefcode?
    pub fn verify(&self) -> bool {
        verify(&self.s, &self.h)
    }
}

/// `entry_rand(game, entry)`.
pub fn entry_rand(game: &str, entry: &Entry) -> Rand {
    entry.rand(game)
}

/// `begin(state)` - Runde beginnt: Eintrag mit den Werten dieser Runde.
/// `s` (Server-Seed) erst NACH der Runde zeigen. Direkt danach `advance`.
pub fn begin(state: &State) -> Entry {
    Entry { n: state.nonce, s: state.server.clone(), c: state.client.clone(), h: commit(&state.server) }
}

/// `advance(state, rng)` - Nonce + 1, neuer Server-Seed (Client bleibt).
pub fn advance(state: &mut State, getrandbits: &mut impl FnMut(u32) -> u64) {
    state.nonce = MAX_NONCE.min(state.nonce + 1);
    state.server = new_server_seed(getrandbits);
}

/// `new_client(state, rng)` - neuer Client-Seed.
pub fn new_client(state: &mut State, getrandbits: &mut impl FnMut(u32) -> u64) {
    state.client = new_client_seed(getrandbits);
}

/// `_plain(v)` - nur JSON-Grundtypen (null, bool, ganze Zahlen bis 2^53-1,
/// Text, Listen, Objekte), keine Gleitkommazahlen, Tiefe hoechstens 4.
pub fn plain(v: &Value, depth: u32) -> bool {
    match v {
        Value::Null | Value::Bool(_) | Value::String(_) => true,
        Value::Number(n) => match (n.as_i64(), n.is_f64()) {
            (Some(i), false) => (-MAX_NONCE..=MAX_NONCE).contains(&i),
            _ => false,
        },
        Value::Array(a) => depth < 4 && a.iter().all(|x| plain(x, depth + 1)),
        Value::Object(o) => depth < 4 && o.values().all(|x| plain(x, depth + 1)),
    }
}

/// Log-Eintrag `{"n", "s", "c", "h", "o"}`.
#[derive(Clone, Debug, PartialEq)]
pub struct LogEntry {
    pub n: i64,
    pub s: String,
    pub c: String,
    pub h: String,
    /// Spiel-eigenes Ergebnis (nur JSON-Grundtypen, sonst null).
    pub o: Value,
}

impl LogEntry {
    /// Als JSON (Reihenfolge wie das Python-dict).
    pub fn to_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("n".into(), Value::from(self.n));
        m.insert("s".into(), Value::from(self.s.clone()));
        m.insert("c".into(), Value::from(self.c.clone()));
        m.insert("h".into(), Value::from(self.h.clone()));
        m.insert("o".into(), self.o.clone());
        Value::Object(m)
    }

    /// Der Rundeneintrag ohne Ergebnis.
    pub fn entry(&self) -> Entry {
        Entry { n: self.n, s: self.s.clone(), c: self.c.clone(), h: self.h.clone() }
    }

    /// `verify(entry)`.
    pub fn verify(&self) -> bool {
        verify(&self.s, &self.h)
    }
}

/// `make_entry(entry, outcome)`.
pub fn make_entry(entry: &Entry, outcome: &Value) -> LogEntry {
    LogEntry {
        n: entry.n,
        s: entry.s.clone(),
        c: entry.c.clone(),
        h: entry.h.clone(),
        o: if plain(outcome, 0) { outcome.clone() } else { Value::Null },
    }
}

/// `record(log, entry, outcome)` - neueste zuletzt, hoechstens `LOG`.
pub fn record(log: &mut Vec<LogEntry>, entry: &Entry, outcome: &Value) {
    log.push(make_entry(entry, outcome));
    if log.len() > LOG {
        let cut = log.len() - LOG;
        log.drain(..cut);
    }
}

/// `clean_log(raw)` - hoechstens `LOG` gueltige Eintraege.
pub fn clean_log(raw: Option<&Value>) -> Vec<LogEntry> {
    let Some(Value::Array(list)) = raw else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for e in list {
        let Some(o) = e.as_object() else {
            continue;
        };
        let (Some(n), Some(s), Some(c), Some(h)) =
            (py_int(o.get("n")), str_of(o.get("s")), str_of(o.get("c")), str_of(o.get("h")))
        else {
            continue;
        };
        if !(valid_nonce(n) && valid_server(s) && valid_client(c) && valid_hash(h)) {
            continue;
        }
        let out_o = match o.get("o") {
            Some(v) if plain(v, 0) => v.clone(),
            _ => Value::Null,
        };
        out.push(LogEntry { n, s: s.to_string(), c: c.to_string(), h: h.to_string(), o: out_o });
    }
    let cut = out.len().saturating_sub(LOG);
    out.split_off(cut)
}

/// Log als JSON-Liste.
pub fn log_to_value(log: &[LogEntry]) -> Value {
    Value::Array(log.iter().map(|e| e.to_value()).collect())
}

/// `verify(entry)` - stimmt `h` mit sha256(`s`) ueberein (Formate gueltig)?
pub fn verify(s: &str, h: &str) -> bool {
    valid_server(s) && valid_hash(h) && commit(s) == h
}

/// `verify(entry)` fuer ein beliebiges JSON (wie Python mit dict).
pub fn verify_value(v: &Value) -> bool {
    match (str_of(v.get("s")), str_of(v.get("h"))) {
        (Some(s), Some(h)) => verify(s, h),
        _ => false,
    }
}

/// `short(h, n)` - erste n Zeichen + "…".
pub fn short(h: &str, n: usize) -> String {
    if h.chars().count() <= n {
        h.to_string()
    } else {
        let mut s: String = h.chars().take(n).collect();
        s.push('\u{2026}');
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_pruefvektoren() {
        assert_eq!(sha256_hex(""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert_eq!(sha256_hex("abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        assert_eq!(
            sha256_hex("abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
        assert_eq!(
            sha256_hex(&"a".repeat(1000)),
            "41edece42d63e8d9bf515a9ba6932e1c20cbc9f5a5d134645adb5db1b9737ea3"
        );
    }

    #[test]
    fn fairness_ablauf() {
        // Zaehler statt py::Random (pgz-core kennt die Engine nicht)
        let mut z: u64 = 0;
        let mut bits = |k: u32| {
            z = z.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            z >> (64 - k)
        };
        let mut st = blank_state(&mut bits);
        assert!(valid_server(&st.server) && valid_client(&st.client) && st.nonce == 0);
        let e = begin(&st);
        assert!(e.verify() && e.h == commit(&st.server));
        let alt = st.clone();
        advance(&mut st, &mut bits);
        assert_eq!(st.nonce, 1);
        assert_ne!(st.server, alt.server);
        assert_eq!(st.client, alt.client);
        let mut log = Vec::new();
        for i in 0..20 {
            record(&mut log, &e, &Value::from(i));
        }
        assert_eq!(log.len(), LOG);
        assert_eq!(log[0].o, Value::from(8));
        assert_eq!(make_entry(&e, &serde_json::json!(1.5)).o, Value::Null);
        assert!(!verify(&e.s, &"0".repeat(64)));
        assert!(!valid_client("lama"));
        assert_eq!(short("abcdefabcdefabcdef", 12), "abcdefabcdef\u{2026}");
        let r = round_rand("lamamines", &"ab".repeat(16), "kaugummi", 7);
        assert_eq!(r, Rand::new(seed_from(&["lamamines", &"ab".repeat(16), "kaugummi", "7"])));
    }
}
