//! Prueft `py::CellSet` gegen CPython: liest Befehle von stdin
//! (`a x y` = add, `d x y` = discard, `p` = Reihenfolge ausgeben) und
//! schreibt je `p` eine Zeile `x,y x,y ...`. Gegenstueck:
//! `rust/tools/cellset_fuzz.py`.

use std::io::{self, BufRead, Write};

use pgz_engine::py::CellSet;

fn main() {
    let mut s = CellSet::cells();
    let stdin = io::stdin();
    let mut out = io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line.unwrap_or_default();
        let t: Vec<&str> = line.split_whitespace().collect();
        match t.first() {
            Some(&"a") | Some(&"d") => {
                let x: i32 = t[1].parse().unwrap_or(0);
                let y: i32 = t[2].parse().unwrap_or(0);
                if t[0] == "a" {
                    s.add((x, y));
                } else {
                    s.discard(&(x, y));
                }
            }
            Some(&"n") => s = CellSet::cells(),
            Some(&"p") => {
                let v: Vec<String> = s.iter().map(|(x, y)| format!("{x},{y}")).collect();
                let _ = writeln!(out, "{}", v.join(" "));
            }
            _ => {}
        }
    }
}
