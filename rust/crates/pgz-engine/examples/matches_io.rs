//! Rust-Seite von `tools/matches_check.py`: fuehrt eine Folge von Befehlen
//! auf der Match-Historie (`pgz_core::matches`) und dem Tages-Archiv
//! (`pgz_engine::dailyarchive`, `seedrand`) aus - eine je Zeile, Ergebnis je
//! Zeile auf die Standardausgabe - im Datenordner aus `PYGAMEZ_DATA_DIR`.
//! Die Python-Seite tut dasselbe mit `matches.py`/`dailyarchive.py`;
//! verglichen werden Ausgabe und die geschriebene Datei.
//!
//!     matches_io <befehle.txt>
//!
//! Befehle (Felder durch ` | ` getrennt, JSON-Werte als JSON):
//! `record | KEY | MODE | SCORE | RESULT | DUR | FLAGS | INFO | PID | REPLACE`,
//! `entries | KEY | PID`, `recent | KEY | N`, `totals | KEY | PID`, `modes | KEY`,
//! `flush`, `forget`, `drop | PID`, `clear | KEY`, `write | TEXT`, `writebak | TEXT`,
//! `fmt | N`, `iso | TEXT`, `date_of | N`, `dates | N | TODAY`,
//! `clean | WIDTH | JSON`, `upsert | ROWS | ROW`, `avail | PLAYED | TODAY`,
//! `rects | X | Y | W | H`, `seed | GAME | DAY`.

use pgz_core::{matches, seedrand};
use pgz_engine::dailyarchive as da;
use serde_json::{json, Value};

fn arg(w: &[&str], i: usize) -> String {
    w.get(i).map(|s| s.to_string()).unwrap_or_default()
}

fn num(w: &[&str], i: usize) -> i64 {
    w.get(i).and_then(|s| s.trim().parse().ok()).unwrap_or(0)
}

fn opt(w: &[&str], i: usize) -> Option<String> {
    w.get(i).map(|s| s.to_string()).filter(|s| !s.is_empty() && s != "-")
}

fn entry_json(e: &matches::Entry) -> Value {
    json!([e.stamp, e.mode, e.score, e.result.clone().unwrap_or(Value::Null), e.dur, e.flags,
        e.info.clone().map(Value::Object).unwrap_or(Value::Null)])
}

fn rows_of(v: &Value) -> Vec<[i64; 3]> {
    v.as_array()
        .map(|a| a.iter().filter_map(|r| r.as_array().map(|r| [r[0].as_i64().unwrap_or(0), r[1].as_i64().unwrap_or(0), r[2].as_i64().unwrap_or(0)])).collect())
        .unwrap_or_default()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(datei) = args.first() else {
        eprintln!("Aufruf: matches_io <befehle.txt>");
        std::process::exit(2);
    };
    let text = std::fs::read_to_string(datei).unwrap_or_default();
    pgz_core::init();
    pgz_core::i18n::set_language("de", false);
    let pfad = pgz_core::store::path_for(matches::FILENAME);
    for zeile in text.lines() {
        let w: Vec<&str> = zeile.split(" | ").collect();
        let Some(cmd) = w.first().map(|c| c.trim()) else { continue };
        if cmd.is_empty() {
            continue;
        }
        let out: String = match cmd {
            "record" => {
                let result = match arg(&w, 4).as_str() {
                    "null" | "" => None,
                    r => r.parse().ok(),
                };
                let info: Option<serde_json::Map<String, Value>> = serde_json::from_str::<Value>(&arg(&w, 7)).ok().and_then(|v| v.as_object().cloned());
                let e = matches::record(&arg(&w, 1), &arg(&w, 2), num(&w, 3), result, num(&w, 5), num(&w, 6), info.as_ref(), opt(&w, 8).as_deref(), arg(&w, 9) == "1");
                entry_json(&e).to_string()
            }
            "entries" => Value::Array(matches::entries(&arg(&w, 1), opt(&w, 2).as_deref()).iter().map(entry_json).collect()).to_string(),
            "recent" => Value::Array(matches::recent(&arg(&w, 1), num(&w, 2) as usize, None).iter().map(entry_json).collect()).to_string(),
            "totals" => {
                let t = matches::totals(&arg(&w, 1), opt(&w, 2).as_deref());
                json!([t.n, t.w, t.l, t.d, t.best, t.sum, t.time, t.first, t.stored]).to_string()
            }
            "modes" => json!(matches::modes_of(&arg(&w, 1), None)).to_string(),
            "flush" => {
                matches::flush();
                "ok".into()
            }
            "forget" => {
                matches::forget();
                "ok".into()
            }
            "drop" => {
                matches::drop_profile(&arg(&w, 1));
                "ok".into()
            }
            "clear" => {
                matches::clear_game(&arg(&w, 1), None);
                "ok".into()
            }
            "write" => {
                std::fs::write(&pfad, arg(&w, 1)).ok();
                "ok".into()
            }
            "writebak" => {
                let mut bak = pfad.as_os_str().to_os_string();
                bak.push(".bak");
                std::fs::write(std::path::PathBuf::from(bak), arg(&w, 1)).ok();
                "ok".into()
            }
            "fmt" => matches::format_duration(num(&w, 1)),
            "iso" => match seedrand::day_index_checked(&arg(&w, 1)) {
                Some(d) => d.to_string(),
                None => "ValueError".into(),
            },
            "date_of" => seedrand::date_of(num(&w, 1)),
            "dates" => {
                let (d, today) = (num(&w, 1), num(&w, 2));
                format!("{} {} {} {} {}", da::date_str(d), da::chip_date(d), da::short_date(d), da::long_date(d), da::day_label(d, Some(today)))
            }
            "clean" => {
                let raw: Option<Value> = serde_json::from_str(&arg(&w, 2)).ok();
                Value::Array(da::clean_rows(raw.as_ref(), num(&w, 1) as usize).into_iter().map(Value::Array).collect()).to_string()
            }
            "upsert" => {
                let mut rows = rows_of(&serde_json::from_str(&arg(&w, 1)).unwrap_or(Value::Null));
                let row = rows_of(&json!([serde_json::from_str::<Value>(&arg(&w, 2)).unwrap_or(Value::Null)]));
                if let Some(r) = row.first() {
                    da::upsert(&mut rows, *r);
                }
                json!(rows).to_string()
            }
            "avail" => {
                let played: Vec<i64> = serde_json::from_str::<Value>(&arg(&w, 1))
                    .ok()
                    .and_then(|v| v.as_array().map(|a| a.iter().filter_map(|x| x.as_i64()).collect()))
                    .unwrap_or_default();
                json!(da::available(played, Some(num(&w, 2)))).to_string()
            }
            "rects" => {
                let r = da::day_row_rects(num(&w, 1) as i32, num(&w, 2) as i32, num(&w, 3) as i32, num(&w, 4) as i32);
                json!(r.iter().map(|x| [x.x, x.y, x.w, x.h]).collect::<Vec<_>>()).to_string()
            }
            "seed" => da::seed_hex(&arg(&w, 1), num(&w, 2)),
            "file" => std::fs::read_to_string(&pfad).unwrap_or_else(|_| "-".into()),
            other => format!("? {other}"),
        };
        println!("{out}");
    }
}
