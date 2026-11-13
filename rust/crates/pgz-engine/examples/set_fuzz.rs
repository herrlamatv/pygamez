//! Prueft `py::IntSet` gegen Faelle aus CPython (Datei als Argument).
use pgz_engine::py::IntSet;

fn main() {
    let pfad = std::env::args().nth(1).expect("usage: set_fuzz <file>");
    let text = std::fs::read_to_string(pfad).expect("cannot read file");
    let (mut gut, mut schlecht) = (0, 0);
    let mut kurz: Option<String> = None;
    for zeile in text.lines() {
        let (ops, erwartet) = zeile.split_once(" | ").unwrap_or((zeile, ""));
        let mut s = IntSet::new();
        for op in ops.split_whitespace() {
            let v: i64 = op[1..].parse().unwrap();
            if op.starts_with('+') {
                s.add(v)
            } else {
                s.discard(v)
            }
        }
        let ist: Vec<String> = s.iter().map(|v| v.to_string()).collect();
        if ist.join(" ") == erwartet.trim() {
            gut += 1
        } else {
            let b = format!("{ops}\n  py: {erwartet}\n  rs: {}", ist.join(" "));
            if kurz.is_none() {
                kurz = Some(b);
            }
            schlecht += 1
        }
    }
    println!("{gut} gleich, {schlecht} anders");
    if let Some(k) = kurz {
        println!("kuerzester Fall:\n{k}");
    }
}
