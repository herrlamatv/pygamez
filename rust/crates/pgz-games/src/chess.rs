//! chess.rs
//! ========
//! Schach - gegen die KI (sechs Staerken), zu zweit am selben Rechner oder
//! im Raetsel-Modus mit 200 Aufgaben. Uebertragen aus `games/chess.py`;
//! Regeln/KI in [`crate::chess_engine`], Zeichnen in [`crate::chess_draw`].
//!
//! KI: In Python ist die Suche ein Generator, der pro Bild ~7 ms rechnet.
//! Im Pruefstand (eingefrorene Uhr) gibt er nie ab und rechnet im ersten
//! Bild durch - genau so rechnet die Rust-Fassung dort synchron. Im echten
//! Spiel laeuft die Suche in einem Hintergrund-Thread (kein Ruckeln) und wird
//! nach der Rechenzeit der Stufe ueber ein Abbruch-Flag beendet.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::Ordering;
use std::thread::JoinHandle;
use std::time::Instant;

use pgz_core::store;
use pgz_engine::filepick;
use pgz_engine::prelude::*;
use serde_json::{json, Value};

use crate::chess_engine::{self as ce, Position, Search, Tt};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum St {
    Setup,
    Play,
    Over,
    PuzMenu,
    Puz,
}

pub(crate) const DIFFS: [&str; 6] = ["lvl0", "lvl1", "lvl2", "lvl3", "lvl4", "lvl5"];
pub(crate) const CLOCKS: [&str; 5] = ["none", "1+0", "3+2", "5+0", "10+5"];

fn clock_time(opt: &str) -> Option<(f64, f64)> {
    match opt {
        "1+0" => Some((60.0, 0.0)),
        "3+2" => Some((180.0, 2.0)),
        "5+0" => Some((300.0, 0.0)),
        "10+5" => Some((600.0, 5.0)),
        _ => None,
    }
}

const AI_MIN_WAIT: f64 = 0.45;
pub(crate) const ANIM_TIME: f64 = 0.2;
pub(crate) const STAGES: [&str; 5] = ["mate1", "mate2", "mate3", "tactic1", "tactic2"];
pub(crate) const PUZ_COLS: usize = 8;

fn mate_n(stage: &str) -> i32 {
    match stage {
        "mate1" => 1,
        "mate2" => 2,
        "mate3" => 3,
        _ => 0,
    }
}

/// Ein Raetsel aus `games/levels/chess-puzzles.json`.
#[derive(Clone, Debug)]
pub(crate) struct Puzzle {
    pub(crate) id: String,
    pub(crate) fen: String,
    pub(crate) moves: Vec<String>,
    pub(crate) rating: Option<Value>,
    pub(crate) theme: String,
}

/// `load_puzzles()` - {stufe: [raetsel]} in STAGES-Reihenfolge.
fn load_puzzles() -> Vec<Vec<Puzzle>> {
    let mut out: Vec<Vec<Puzzle>> = STAGES.iter().map(|_| Vec::new()).collect();
    let text = include_str!("../../../../games/levels/chess-puzzles.json");
    let Ok(data) = serde_json::from_str::<Value>(text) else {
        return out;
    };
    for st in data.get("stages").and_then(|s| s.as_array()).cloned().unwrap_or_default() {
        let Some(i) = st.get("id").and_then(|v| v.as_str()).and_then(|id| STAGES.iter().position(|s| *s == id)) else {
            continue;
        };
        out[i] = st
            .get("puzzles")
            .and_then(|p| p.as_array())
            .cloned()
            .unwrap_or_default()
            .iter()
            .filter_map(|p| {
                let moves = p.get("moves")?.as_array()?;
                let fen = p.get("fen")?.as_str()?;
                if moves.len() < 2 || fen.is_empty() {
                    return None;
                }
                Some(Puzzle {
                    id: p.get("id").map(|v| v.as_str().map(|s| s.to_string()).unwrap_or_else(|| v.to_string())).unwrap_or_default(),
                    fen: fen.to_string(),
                    moves: moves.iter().map(|m| m.as_str().unwrap_or("").to_string()).collect(),
                    rating: p.get("rating").cloned(),
                    theme: p.get("theme").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                })
            })
            .collect();
    }
    out
}

/// Was die Maus gerade beruehrt (Python: `self.hover`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Hover {
    None,
    Btn(&'static str),
    Over(&'static str),
    Cell(usize),
}

#[derive(Clone, Copy)]
pub(crate) struct Anim {
    pub(crate) p: i32,
    pub(crate) a: i32,
    pub(crate) b: i32,
    pub(crate) t0: f64,
}

#[derive(Clone, Copy)]
pub(crate) struct Fade {
    pub(crate) p: i32,
    pub(crate) sq: i32,
    pub(crate) t0: f64,
}

#[derive(Clone, Copy)]
pub(crate) struct Drag {
    pub(crate) sq: i32,
    pub(crate) pos: (i32, i32),
    pub(crate) start: (i32, i32),
    pub(crate) moved: bool,
    pub(crate) was: bool,
}

/// Eine laufende oder fertige Suche.
pub(crate) enum Job {
    /// Noch nicht gestartet (synchron: rechnet beim naechsten Schritt).
    Ready(Box<Search>),
    /// Im Hintergrund-Thread.
    Running { handle: JoinHandle<Search>, stop: std::sync::Arc<std::sync::atomic::AtomicBool>, start: Instant },
    /// Fertig.
    Done(Box<Search>),
}

pub(crate) struct AiJob {
    mv: i32,
    wait: f64,
    search: Option<Job>,
    wall_cap: f64,
}

/// `ChessGame`
pub struct ChessGame {
    pub(crate) b: GameBase,
    pub(crate) state: St,
    pub(crate) diff: usize,
    pub(crate) human_color: i32,
    pub(crate) clock_opt: &'static str,
    pub(crate) c960: bool,
    pub(crate) auto_flip: bool,
    human_wins: i64,
    pub(crate) wins: [i64; 2],
    tt: Option<Tt>,
    ai_job: Option<AiJob>,
    pub(crate) hint_job: Option<Job>,
    pub(crate) dialog: Option<(&'static str, i32)>,
    pub(crate) promo: Option<Vec<i32>>,
    promo_animate: bool,
    pub(crate) msg: Option<String>,
    pub(crate) msg_t: f64,
    pub(crate) hover: Hover,
    pub(crate) setup_focus: usize,
    pub(crate) anims: Vec<Anim>,
    pub(crate) fades: Vec<Fade>,
    pub(crate) flip_anim: Option<f64>,
    pub(crate) list_top: i64,
    pub(crate) list_follow: bool,
    // Caches (chess_draw)
    pub(crate) pieces: HashMap<(i32, i32), Surface>,
    pub(crate) board_cache: Option<((i32, i32, bool, Color), Surface)>,
    pub(crate) overlays: HashMap<String, Surface>,
    pub(crate) arrow: Option<((i32, bool, i32), Surface)>,
    // Schriften
    pub(crate) huge: Font,
    pub(crate) big: Font,
    pub(crate) small: Font,
    pub(crate) tiny: Font,
    pub(crate) clockf: Font,
    pub(crate) listf: Font,
    // Layout
    pub(crate) cell: i32,
    pub(crate) bw: i32,
    pub(crate) frame: i32,
    pub(crate) plate: Rect,
    pub(crate) bx: i32,
    pub(crate) by: i32,
    pub(crate) board_rect: Rect,
    pub(crate) side_rect: Rect,
    pub(crate) gap: i32,
    pub(crate) top_panel: Rect,
    pub(crate) bot_panel: Rect,
    pub(crate) btn_rects: Vec<(&'static str, Rect)>,
    pub(crate) status_rect: Rect,
    pub(crate) list_rect: Rect,
    pub(crate) row_h: i32,
    pub(crate) puz_btn_rects: Vec<(&'static str, Rect)>,
    pub(crate) puz_info_rect: Rect,
    pub(crate) puz_list_rect: Rect,
    pub(crate) over_rect: Rect,
    pub(crate) over_btns: Vec<(&'static str, Rect)>,
    pub(crate) setup_rows: Vec<(&'static str, Vec<Rect>)>,
    pub(crate) start_rect: Rect,
    pub(crate) stage_rects: Vec<Rect>,
    pub(crate) grid_rects: Vec<Rect>,
    pub(crate) puz_start_rect: Rect,
    // Raetsel
    pub(crate) puzzles: Vec<Vec<Puzzle>>,
    pub(crate) puz_solved: HashSet<String>,
    pub(crate) puz_seen: HashSet<String>,
    pub(crate) puz_stage: usize,
    pub(crate) puz: Option<Puzzle>,
    pub(crate) puz_idx: usize,
    pub(crate) puz_cursor: usize,
    pub(crate) puz_line: Vec<String>,
    pub(crate) puz_step: usize,
    pub(crate) puz_player: i32,
    pub(crate) puz_mate: i32,
    pub(crate) puz_status: &'static str,
    puz_timer: f64,
    pub(crate) puz_mistakes: i64,
    puz_shown: bool,
    pub(crate) puz_feedback: (&'static str, Option<Color>),
    // Partie
    pub(crate) pos: Position,
    pub(crate) c960_n: Option<i64>,
    pub(crate) start_fen: String,
    pub(crate) start_side: i32,
    pub(crate) start_full: i32,
    pub(crate) moves: Vec<i32>,
    pub(crate) sans: Vec<String>,
    ucis: Vec<String>,
    pub(crate) caps: Vec<i32>,
    pub(crate) last_move: Option<(i32, i32)>,
    pub(crate) result: Option<(Option<i32>, &'static str)>,
    pub(crate) result_loser: Option<i32>,
    pub(crate) assisted: bool,
    pub(crate) sel: Option<i32>,
    pub(crate) targets: Vec<(i32, Vec<i32>)>,
    pub(crate) drag: Option<Drag>,
    pub(crate) hint_move: i32,
    draw_block: usize,
    ai_wait: f64,
    ai_last_score: Option<i32>,
    low_time_warned: [bool; 2],
    pub(crate) clock: Option<[f64; 2]>,
    inc: f64,
    pub(crate) view_black: bool,
    flip_pending: f64,
    pub(crate) cursor: [i32; 2],
    pub(crate) over_t0: f64,
    pub(crate) legal: Vec<i32>,
    by_from: HashMap<i32, Vec<i32>>,
    pub(crate) check: bool,
}

impl ChessGame {
    /// `ChessGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let mut g = ChessGame {
            b,
            state: St::Setup,
            diff: 2,
            human_color: ce::WHITE,
            clock_opt: "none",
            c960: false,
            auto_flip: false,
            human_wins: 0,
            wins: [0, 0],
            tt: Some(Tt::new()),
            ai_job: None,
            hint_job: None,
            dialog: None,
            promo: None,
            promo_animate: true,
            msg: None,
            msg_t: 0.0,
            hover: Hover::None,
            setup_focus: 0,
            anims: Vec::new(),
            fades: Vec::new(),
            flip_anim: None,
            list_top: 0,
            list_follow: true,
            pieces: HashMap::new(),
            board_cache: None,
            overlays: HashMap::new(),
            arrow: None,
            huge: f.clone(),
            big: f.clone(),
            small: f.clone(),
            tiny: f.clone(),
            clockf: f.clone(),
            listf: f,
            cell: 14,
            bw: 112,
            frame: 11,
            plate: z,
            bx: 0,
            by: 0,
            board_rect: z,
            side_rect: z,
            gap: 4,
            top_panel: z,
            bot_panel: z,
            btn_rects: Vec::new(),
            status_rect: z,
            list_rect: z,
            row_h: 10,
            puz_btn_rects: Vec::new(),
            puz_info_rect: z,
            puz_list_rect: z,
            over_rect: z,
            over_btns: Vec::new(),
            setup_rows: Vec::new(),
            start_rect: z,
            stage_rects: Vec::new(),
            grid_rects: Vec::new(),
            puz_start_rect: z,
            puzzles: Vec::new(),
            puz_solved: HashSet::new(),
            puz_seen: HashSet::new(),
            puz_stage: 0,
            puz: None,
            puz_idx: 0,
            puz_cursor: 0,
            puz_line: Vec::new(),
            puz_step: 0,
            puz_player: ce::WHITE,
            puz_mate: 0,
            puz_status: "intro",
            puz_timer: 0.0,
            puz_mistakes: 0,
            puz_shown: false,
            puz_feedback: ("watch", None),
            pos: Position::default(),
            c960_n: None,
            start_fen: String::new(),
            start_side: ce::WHITE,
            start_full: 1,
            moves: Vec::new(),
            sans: Vec::new(),
            ucis: Vec::new(),
            caps: Vec::new(),
            last_move: None,
            result: None,
            result_loser: None,
            assisted: false,
            sel: None,
            targets: Vec::new(),
            drag: None,
            hint_move: 0,
            draw_block: 0,
            ai_wait: 0.0,
            ai_last_score: None,
            low_time_warned: [false, false],
            clock: None,
            inc: 0.0,
            view_black: false,
            flip_pending: 0.0,
            cursor: [6, 4],
            over_t0: 0.0,
            legal: Vec::new(),
            by_from: HashMap::new(),
            check: false,
        };
        g.reset();
        Box::new(g)
    }

    fn make_fonts(&mut self) {
        let h = self.b.height;
        self.huge = ui::font(26.max(h / 12), true, false);
        self.big = ui::font(17.max(h / 24), true, false);
        self.small = ui::font(13.max(h / 34), false, false);
        self.tiny = ui::font(11.max(h / 46), false, false);
        self.clockf = ui::font(14.max(h / 30), true, true);
        self.listf = ui::font(11.max(h / 42), false, true);
    }

    fn abort_jobs(&mut self) {
        for job in [self.ai_job.take().and_then(|j| j.search), self.hint_job.take()].into_iter().flatten() {
            self.drop_job(job);
        }
    }

    /// Eine Suche verwerfen; die Transpositionstabelle bleibt (Python
    /// teilt das Woerterbuch), ein laufender Thread wird gestoppt.
    fn drop_job(&mut self, job: Job) {
        match job {
            Job::Ready(s) | Job::Done(s) => {
                if self.tt.is_none() {
                    self.tt = Some(s.tt);
                }
            }
            Job::Running { stop, .. } => {
                stop.store(true, Ordering::Relaxed);
            }
        }
    }

    fn take_tt(&mut self) -> Tt {
        self.tt.take().unwrap_or_default()
    }

    /// Eine Suche einen Schritt weiterbringen (Python: `next(gen)`).
    fn step_job(&mut self, job: Job) -> Job {
        match job {
            Job::Ready(mut s) => {
                if py::deterministic() {
                    s.run();
                    s.pick_spread();
                    let mut s = s;
                    self.tt = Some(std::mem::take(&mut s.tt));
                    Job::Done(s)
                } else {
                    let stop = s.stop_flag.clone();
                    let handle = std::thread::spawn(move || {
                        let mut s = *s;
                        s.run();
                        s
                    });
                    Job::Running { handle, stop, start: Instant::now() }
                }
            }
            Job::Running { handle, stop, start } => {
                if handle.is_finished() {
                    match handle.join() {
                        Ok(mut s) => {
                            s.pick_spread();
                            if self.tt.is_none() {
                                self.tt = Some(std::mem::take(&mut s.tt));
                            }
                            Job::Done(Box::new(s))
                        }
                        Err(_) => Job::Done(Box::new(Search::new(&self.pos, &ce::LEVELS[0], Tt::new()))),
                    }
                } else {
                    Job::Running { handle, stop, start }
                }
            }
            d => d,
        }
    }

    fn save_setting(&mut self, key: &str, value: Value) {
        self.b.set_opt("chess", key, value);
    }

    // ===================================================== Layout

    fn layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        let m = 6.max(h / 64);
        let frame = 11.max((h as f64 * 0.03) as i32);
        let side_min = 140.max((w as f64 * 0.29) as i32);
        let cell = (((h - 2 * m - 2 * frame) as f64 / 8.0).min((w - side_min - 3 * m - 2 * frame) as f64 / 8.0)) as i32;
        let cell = 14.max(cell);
        self.cell = cell;
        self.bw = 8 * cell;
        let pw = self.bw + 2 * frame;
        self.frame = frame;
        self.plate = Rect::new(m, py::fdiv32(h - pw, 2), pw, pw);
        self.bx = self.plate.x + frame;
        self.by = self.plate.y + frame;
        self.board_rect = Rect::new(self.bx, self.by, self.bw, self.bw);
        let sx = self.plate.right() + m;
        let sw = w - sx - m;
        self.side_rect = Rect::new(sx, m, sw, h - 2 * m);
        let gap = 4.max(h / 90);
        self.gap = gap;
        let ph = self.small.get_height() + self.tiny.get_height() + 16;
        self.top_panel = Rect::new(sx, m, sw, ph);
        self.bot_panel = Rect::new(sx, h - m - ph, sw, ph);
        let n = 5;
        let bs = 22f64.max(46f64.min((sw - (n - 1) * gap) as f64 / n as f64)) as i32;
        let by = self.bot_panel.y - gap - bs;
        let total = n * bs + (n - 1) * gap;
        let bx0 = sx + py::fdiv32(sw - total, 2);
        self.btn_rects = ["undo", "hint", "flip", "draw", "resign"].iter().enumerate().map(|(i, bid)| (*bid, Rect::new(bx0 + i as i32 * (bs + gap), by, bs, bs))).collect();
        let sh = self.small.get_height() + 8;
        self.status_rect = Rect::new(sx, self.top_panel.bottom() + gap, sw, sh);
        let ly = self.status_rect.bottom() + gap;
        self.list_rect = Rect::new(sx, ly, sw, by - gap - ly);
        self.row_h = self.listf.get_height() + 3;
        let pbs = 24f64.max(48f64.min((sw - 3 * gap) as f64 / 4.0)) as i32;
        let pby = h - m - pbs;
        let total = 4 * pbs + 3 * gap;
        let px0 = sx + py::fdiv32(sw - total, 2);
        self.puz_btn_rects = ["list", "retry", "solution", "next"].iter().enumerate().map(|(i, bid)| (*bid, Rect::new(px0 + i as i32 * (pbs + gap), pby, pbs, pbs))).collect();
        let th = self.tiny.get_height();
        let bh_ = self.big.get_height();
        let ih = 16 + bh_ + 2 * (th + 2) + 6 + self.small.get_height() + 4 + 2 * (th + 1) + 6 + 4.max(th / 3) * 2 + 10 + 2 * (bh_ + 1);
        let ih = ih.min(60.max(py::fdiv32((pby - gap - m) * 2, 3)));
        self.puz_info_rect = Rect::new(sx, m, sw, ih);
        self.puz_list_rect = Rect::new(sx, m + ih + gap, sw, pby - 2 * gap - m - ih);
        let ow = (self.bw - 12).min(220.max((self.bw as f64 * 0.9) as i32));
        let bh = 24.max(h / 18);
        let pad = 10;
        let bw3 = py::fdiv32(ow - 2 * pad - 2 * gap, 3);
        let labels = [t("chess.btn.new"), t("chess.btn.setup"), t("chess.btn.pgn")];
        let two_rows = labels.iter().any(|lb| self.tiny.size(lb).0 > bw3 - 8);
        let btn_h = if two_rows { bh * 2 + gap } else { bh };
        let oh = self.huge.get_height() + 2 * self.small.get_height() + btn_h + 30;
        self.over_rect = Rect::new(self.bx + py::fdiv32(self.bw - ow, 2), self.by + py::fdiv32(self.bw - oh, 2), ow, oh);
        let (ox, oy) = (self.over_rect.x + pad, self.over_rect.bottom() - pad - btn_h);
        if two_rows {
            let bw2 = py::fdiv32(ow - 2 * pad - gap, 2);
            self.over_btns = vec![
                ("new", Rect::new(ox, oy, bw2, bh)),
                ("setup", Rect::new(ox + bw2 + gap, oy, bw2, bh)),
                ("pgn", Rect::new(ox, oy + bh + gap, ow - 2 * pad, bh)),
            ];
        } else {
            self.over_btns = ["new", "setup", "pgn"].iter().enumerate().map(|(i, bid)| (*bid, Rect::new(ox + i as i32 * (bw3 + gap), oy, bw3, bh))).collect();
        }
    }

    fn title_bottom(&self, y: i32) -> i32 {
        y + self.huge.get_height() / 2 + 36 + self.small.get_height() / 2 + 4
    }

    fn build_setup_layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let rows: Vec<(&'static str, i32)> = if self.b.mode == "multi" {
            vec![("clock", CLOCKS.len() as i32), ("c960", 2), ("flip", 2)]
        } else {
            vec![("diff", 6), ("color", 2), ("clock", CLOCKS.len() as i32), ("c960", 2)]
        };
        let bw = (w - 40).min(460.max((w as f64 * 0.56) as i32));
        let bh = 24.max(44.min((h as f64 * 0.068) as i32));
        let lab = self.tiny.get_height() + 3;
        let top = ((h as f64 * 0.25) as i32).max(self.title_bottom((h as f64 * 0.09) as i32) + 4);
        let start_h = bh + 4;
        let bottom = h - 34 - start_h - 6.max(h / 50);
        let step = ((bh + lab + 8.max(h / 40)) as f64).min((bottom - top) as f64 / rows.len() as f64);
        let gap = 6.0;
        self.setup_rows.clear();
        let mut y = top as f64;
        for (rid, n) in rows {
            let y_btn = (y + lab as f64) as i32;
            let cw = (bw as f64 - gap * (n - 1) as f64) / n as f64;
            let rects = (0..n).map(|i| Rect::new((cx as f64 - bw as f64 / 2.0 + i as f64 * (cw + gap)) as i32, y_btn, cw as i32, bh)).collect();
            self.setup_rows.push((rid, rects));
            y += step;
        }
        let sw = 220.min(w - 60);
        self.start_rect = Rect::new(cx - sw / 2, (y + 4.max(h / 60) as f64) as i32, sw, start_h);
        self.setup_focus = self.setup_focus.min(self.setup_rows.len());
    }

    fn build_puz_menu_layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        let m = 10.max(h / 40);
        let top = self.title_bottom((h as f64 * 0.075) as i32) + 4.max(h / 60);
        let tw = (w - 2 * m).min(640.max((w as f64 * 0.78) as i32));
        let gap = 4.max(h / 100);
        let th = 30.max((h as f64 * 0.09) as i32);
        let n = STAGES.len() as f64;
        let tcw = (tw as f64 - gap as f64 * (n - 1.0)) / n;
        let x0 = (w - tw) as f64 / 2.0;
        self.stage_rects = (0..STAGES.len()).map(|i| Rect::new((x0 + i as f64 * (tcw + gap as f64)) as i32, top, tcw as i32, th)).collect();
        let gy = top + th + 8.max(h / 40);
        let rows = 5;
        let start_h = 24.max(40.min(h / 14));
        let foot = 38 + start_h + 4.max(h / 90);
        let ch = (((h - gy - foot - gap * (rows - 1)) as f64 / rows as f64).min(60.0)) as i32;
        let cw = (((tw - gap * (PUZ_COLS as i32 - 1)) as f64 / PUZ_COLS as f64).min(ch as f64 * 1.9)) as i32;
        let gw = PUZ_COLS as i32 * cw + (PUZ_COLS as i32 - 1) * gap;
        let gx = py::fdiv32(w - gw, 2);
        self.grid_rects = (0..PUZ_COLS * rows as usize)
            .map(|i| Rect::new(gx + (i % PUZ_COLS) as i32 * (cw + gap), gy + (i / PUZ_COLS) as i32 * (ch + gap), cw, ch))
            .collect();
        let by = gy + rows * (ch + gap) + 2.max(h / 90);
        let bw = 240.min(w - 60);
        self.puz_start_rect = Rect::new(py::fdiv32(w - bw, 2), by, bw, start_h);
    }

    // ===================================================== Partie aufbauen

    fn new_game(&mut self) {
        self.b.new_round_result();
        self.abort_jobs();
        if self.c960 && (self.b.mode == "single" || self.b.mode == "multi") {
            let n = py::randint(0, 959);
            self.c960_n = Some(n);
            self.pos = ce::from_fen(&ce::chess960_fen(n), Some(true)).unwrap_or_default();
        } else {
            self.c960_n = None;
            self.pos = ce::from_fen(ce::START_FEN, Some(false)).unwrap_or_default();
        }
        self.start_fen = self.pos.fen();
        self.start_side = self.pos.side;
        self.start_full = self.pos.full;
        self.moves.clear();
        self.sans.clear();
        self.ucis.clear();
        self.caps.clear();
        self.last_move = None;
        self.result = None;
        self.result_loser = None;
        self.assisted = false;
        self.sel = None;
        self.targets.clear();
        self.drag = None;
        self.hint_move = 0;
        self.dialog = None;
        self.promo = None;
        self.draw_block = 0;
        self.ai_wait = 0.0;
        self.ai_last_score = None;
        self.low_time_warned = [false, false];
        match clock_time(self.clock_opt) {
            Some((base, inc)) => {
                self.clock = Some([base, base]);
                self.inc = inc;
            }
            None => {
                self.clock = None;
                self.inc = 0.0;
            }
        }
        self.view_black = self.b.mode == "single" && self.human_color == ce::BLACK;
        self.flip_pending = 0.0;
        self.cursor = [6, 4];
        self.anims.clear();
        self.fades.clear();
        self.list_top = 0;
        self.list_follow = true;
        self.refresh();
    }

    fn refresh(&mut self) {
        self.legal = self.pos.legal_moves();
        self.by_from.clear();
        for &m in &self.legal {
            self.by_from.entry(m & 255).or_default().push(m);
        }
        self.check = self.pos.in_check();
    }

    // ===================================================== Koordinaten

    pub(crate) fn sq_to_disp(&self, sq: i32) -> (i32, i32) {
        let (r, f) = (sq >> 4, sq & 7);
        if self.view_black {
            (r, 7 - f)
        } else {
            (7 - r, f)
        }
    }

    pub(crate) fn disp_to_sq(&self, dr: i32, dc: i32) -> i32 {
        if self.view_black {
            dr * 16 + (7 - dc)
        } else {
            (7 - dr) * 16 + dc
        }
    }

    pub(crate) fn sq_rect(&self, sq: i32) -> Rect {
        let (dr, dc) = self.sq_to_disp(sq);
        Rect::new(self.bx + dc * self.cell, self.by + dr * self.cell, self.cell, self.cell)
    }

    pub(crate) fn sq_at(&self, pos: (i32, i32)) -> Option<i32> {
        let c = py::fdiv32(pos.0 - self.bx, self.cell);
        let r = py::fdiv32(pos.1 - self.by, self.cell);
        if (0..8).contains(&r) && (0..8).contains(&c) {
            return Some(self.disp_to_sq(r, c));
        }
        None
    }

    pub(crate) fn move_squares(m: i32) -> (i32, i32) {
        let frm = m & 255;
        let mut to = (m >> 8) & 255;
        if m >> 20 == ce::M_CASTLE {
            to = (frm & 0x70) + if to > frm { 6 } else { 2 };
        }
        (frm, to)
    }

    pub(crate) fn targets_get(&self, sq: i32) -> Option<&Vec<i32>> {
        self.targets.iter().find(|e| e.0 == sq).map(|e| &e.1)
    }

    // ===================================================== Eingabe

    fn fixed_key(&self, key: &str, names: &[&str]) -> bool {
        names.contains(&key) && self.b.key_is_free(key)
    }

    pub(crate) fn setup_values(&self, rid: &str) -> (usize, usize) {
        match rid {
            "diff" => (6, self.diff),
            "color" => (2, self.human_color as usize),
            "clock" => (CLOCKS.len(), CLOCKS.iter().position(|c| *c == self.clock_opt).unwrap_or(0)),
            "c960" => (2, self.c960 as usize),
            _ => (2, self.auto_flip as usize),
        }
    }

    fn setup_set(&mut self, rid: &str, i: usize) {
        match rid {
            "diff" => {
                self.diff = i;
                self.save_setting("difficulty", json!(i));
            }
            "color" => {
                self.human_color = i as i32;
                self.save_setting("color", json!(if i as i32 == ce::BLACK { "black" } else { "white" }));
            }
            "clock" => {
                self.clock_opt = CLOCKS[i];
                let v = self.clock_opt;
                self.save_setting("clock", json!(v));
            }
            "c960" => {
                self.c960 = i == 1;
                let v = self.c960;
                self.save_setting("chess960", json!(v));
            }
            "flip" => {
                self.auto_flip = i == 1;
                let v = self.auto_flip;
                self.save_setting("flip", json!(v));
            }
            _ => {}
        }
    }

    fn handle_setup(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown {
            let k = ev.key.as_str();
            let n_rows = self.setup_rows.len();
            let act = |a: &str| self.b.is_action(k, a, None);
            if ["1", "2", "3", "4", "5", "6"].contains(&k) && self.b.mode == "single" {
                self.setup_set("diff", k.parse::<usize>().unwrap_or(1) - 1);
                self.b.play_sound("click");
            } else if (k == "c" || k == "C") && self.b.mode == "single" {
                self.setup_set("color", (1 - self.human_color) as usize);
                self.b.play_sound("select");
            } else if k == "Up" || act("up") {
                self.setup_focus = py::pmod(self.setup_focus as i64 - 1, n_rows as i64 + 1) as usize;
                self.b.play_sound("move");
            } else if k == "Down" || act("down") {
                self.setup_focus = (self.setup_focus + 1) % (n_rows + 1);
                self.b.play_sound("move");
            } else if k == "Left" || k == "Right" || act("left") || act("right") {
                if self.setup_focus < n_rows {
                    let rid = self.setup_rows[self.setup_focus].0;
                    let (n, cur) = self.setup_values(rid);
                    let step = if k == "Left" || act("left") { -1 } else { 1 };
                    self.setup_set(rid, py::pmod(cur as i64 + step, n as i64) as usize);
                    self.b.play_sound("move");
                }
            } else if ["Return", "space", "KP_Enter"].contains(&k) || act("action") {
                self.start_play();
            }
        } else if ev.kind == Kind::MouseDown {
            for ri in 0..self.setup_rows.len() {
                let (rid, rects) = self.setup_rows[ri].clone();
                for (i, rc) in rects.iter().enumerate() {
                    if rc.hit(ev.pos) {
                        self.setup_focus = ri;
                        self.setup_set(rid, i);
                        self.b.play_sound("click");
                        return;
                    }
                }
            }
            if self.start_rect.hit(ev.pos) {
                self.start_play();
            }
        }
    }

    fn start_play(&mut self) {
        self.b.game_over = false;
        self.new_game();
        self.state = St::Play;
        self.b.play_sound("click");
        ui::begin_transition(None);
    }

    fn handle_board_state(&mut self, ev: &InputEvent) {
        if self.promo.is_some() {
            self.handle_promo(ev);
            return;
        }
        if self.dialog.is_some() {
            self.handle_dialog(ev);
            return;
        }
        let k: Option<String> = if ev.kind == Kind::KeyDown { Some(ev.key.clone()) } else { None };
        if ev.kind == Kind::MouseMove {
            self.hover = self.button_at(ev.pos).map(Hover::Btn).unwrap_or(Hover::None);
        }
        if ev.kind == Kind::Wheel {
            let rect = if self.state == St::Play { self.list_rect } else { self.puz_list_rect };
            if rect.hit(ev.pos) {
                self.scroll_list(-ev.delta as i64 * 2);
            }
            return;
        }
        if ev.kind == Kind::MouseDown {
            if let Some(bid) = self.button_at(ev.pos) {
                self.press_button(bid);
                return;
            }
        }
        if let Some(k) = k.as_deref() {
            if self.state == St::Play {
                if self.fixed_key(k, &["u", "U", "BackSpace"]) {
                    self.undo();
                    return;
                }
                if self.fixed_key(k, &["h", "H"]) {
                    self.request_hint();
                    return;
                }
                if self.fixed_key(k, &["f", "F"]) {
                    self.flip_view();
                    return;
                }
                if self.fixed_key(k, &["o", "O"]) {
                    self.offer_draw();
                    return;
                }
                if self.fixed_key(k, &["x", "X"]) {
                    self.ask_resign();
                    return;
                }
                if k == "Prior" || k == "Next" {
                    self.scroll_list(if k == "Prior" { -6 } else { 6 });
                    return;
                }
            } else {
                if self.fixed_key(k, &["h", "H"]) {
                    self.puz_show_solution();
                    return;
                }
                if self.fixed_key(k, &["u", "U", "BackSpace"]) {
                    self.puz_open(self.puz_stage, self.puz_idx);
                    return;
                }
                if self.fixed_key(k, &["m", "M", "Tab"]) {
                    self.puz_back_to_menu();
                    return;
                }
                if self.fixed_key(k, &["n", "N"]) {
                    self.puz_next();
                    return;
                }
                if (self.puz_status == "solved" || self.puz_status == "shown_done") && (["Return", "space", "KP_Enter"].contains(&k) || self.b.is_action(k, "action", None)) {
                    self.puz_next();
                    return;
                }
            }
        }
        if !self.human_can_move() {
            if ev.kind == Kind::MouseUp {
                self.drag = None;
            }
            return;
        }
        match ev.kind {
            Kind::MouseMove => {
                if let Some(sq) = self.sq_at(ev.pos) {
                    let (r, c) = self.sq_to_disp(sq);
                    self.cursor = [r, c];
                }
                if let Some(d) = self.drag.as_mut() {
                    d.pos = ev.pos;
                    if (ev.pos.0 - d.start.0).abs() + (ev.pos.1 - d.start.1).abs() > 4 {
                        d.moved = true;
                    }
                }
            }
            Kind::MouseDown => {
                let Some(sq) = self.sq_at(ev.pos) else {
                    self.sel = None;
                    self.targets.clear();
                    return;
                };
                let (r, c) = self.sq_to_disp(sq);
                self.cursor = [r, c];
                if self.sel.is_some() && self.targets_get(sq).is_some() {
                    self.try_move(sq, true);
                    return;
                }
                let p = self.pos.b[sq as usize];
                if p != 0 && (p >> 3) == self.pos.side {
                    let was = self.sel == Some(sq);
                    self.select(sq);
                    self.drag = Some(Drag { sq, pos: ev.pos, start: ev.pos, moved: false, was });
                    self.b.play_sound("click");
                } else {
                    self.sel = None;
                    self.targets.clear();
                }
            }
            Kind::MouseUp => {
                let Some(drag) = self.drag.take() else {
                    return;
                };
                let sq = self.sq_at(ev.pos);
                if let Some(s) = sq.filter(|s| *s != drag.sq && self.targets_get(*s).is_some()) {
                    self.try_move(s, false);
                } else if sq == Some(drag.sq) && drag.was && !drag.moved {
                    self.sel = None;
                    self.targets.clear();
                }
            }
            Kind::KeyDown => {
                let k = ev.key.as_str();
                let act = |a: &str| self.b.is_action(k, a, None);
                if k == "Up" || act("up") {
                    self.move_cursor(-1, 0);
                } else if k == "Down" || act("down") {
                    self.move_cursor(1, 0);
                } else if k == "Left" || act("left") {
                    self.move_cursor(0, -1);
                } else if k == "Right" || act("right") {
                    self.move_cursor(0, 1);
                } else if ["Return", "space", "KP_Enter"].contains(&k) || act("action") {
                    let sq = self.disp_to_sq(self.cursor[0], self.cursor[1]);
                    if self.sel.is_some() && self.targets_get(sq).is_some() {
                        self.try_move(sq, true);
                    } else {
                        let p = self.pos.b[sq as usize];
                        if p != 0 && (p >> 3) == self.pos.side && self.sel != Some(sq) {
                            self.select(sq);
                            self.b.play_sound("click");
                        } else {
                            self.sel = None;
                            self.targets.clear();
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn move_cursor(&mut self, dr: i32, dc: i32) {
        self.cursor[0] = 0.max(7.min(self.cursor[0] + dr));
        self.cursor[1] = 0.max(7.min(self.cursor[1] + dc));
        self.b.play_sound("move");
    }

    pub(crate) fn button_at(&self, pos: (i32, i32)) -> Option<&'static str> {
        let rects = if self.state == St::Play { &self.btn_rects } else { &self.puz_btn_rects };
        rects.iter().find(|(_, r)| r.hit(pos)).map(|e| e.0)
    }

    fn press_button(&mut self, bid: &str) {
        match bid {
            "undo" => self.undo(),
            "hint" => self.request_hint(),
            "flip" => self.flip_view(),
            "draw" => self.offer_draw(),
            "resign" => self.ask_resign(),
            "list" => self.puz_back_to_menu(),
            "retry" => self.puz_open(self.puz_stage, self.puz_idx),
            "solution" => self.puz_show_solution(),
            "next" => self.puz_next(),
            _ => {}
        }
    }

    pub(crate) fn human_can_move(&self) -> bool {
        if self.state == St::Puz {
            return self.puz_status == "play" && self.pos.side == self.puz_player;
        }
        if self.state != St::Play {
            return false;
        }
        self.b.mode == "multi" || self.pos.side == self.human_color
    }

    fn select(&mut self, sq: i32) {
        self.sel = Some(sq);
        let moves = self.by_from.get(&sq).cloned().unwrap_or_default();
        let normal: HashSet<i32> = moves.iter().filter(|m| *m >> 20 != ce::M_CASTLE).map(|m| (m >> 8) & 255).collect();
        let mut targets: Vec<(i32, Vec<i32>)> = Vec::new();
        let mut add = |to: i32, m: i32| match targets.iter_mut().find(|e| e.0 == to) {
            Some(e) => e.1.push(m),
            None => targets.push((to, vec![m])),
        };
        for &m in &moves {
            let to = (m >> 8) & 255;
            if m >> 20 == ce::M_CASTLE {
                add(to, m);
                let kt = (sq & 0x70) + if to > sq { 6 } else { 2 };
                if kt != sq && kt != to && !normal.contains(&kt) {
                    add(kt, m);
                }
            } else {
                add(to, m);
            }
        }
        self.targets = targets;
    }

    fn try_move(&mut self, sq: i32, animate: bool) {
        let cands = self.targets_get(sq).cloned().unwrap_or_default();
        if cands.is_empty() {
            return;
        }
        if cands.len() > 1 {
            let order = |m: &i32| match (m >> 16) & 7 {
                ce::QUEEN => 0,
                ce::ROOK => 1,
                ce::BISHOP => 2,
                ce::KNIGHT => 3,
                _ => 9,
            };
            let mut c = cands;
            c.sort_by_key(order);
            self.promo = Some(c);
            self.promo_animate = animate;
            self.b.play_sound("select");
            return;
        }
        self.human_move(cands[0], animate);
    }

    fn human_move(&mut self, m: i32, animate: bool) {
        if self.state == St::Puz {
            self.puz_player_move(m, animate);
        } else {
            self.commit(m, animate);
        }
    }

    pub(crate) fn promo_rects(&self) -> Vec<Rect> {
        let n = 4;
        let size = 36.max(((self.cell as f64 * 1.15) as i32).min(py::fdiv32(self.bw - 40, n)));
        let gap = 6.max(size / 8);
        let total = size * n + gap * (n - 1);
        let x0 = self.bx + py::fdiv32(self.bw - total, 2);
        let y = self.by + self.bw / 2 - size / 2 + self.small.get_height() / 2;
        (0..n).map(|i| Rect::new(x0 + i * (size + gap), y, size, size)).collect()
    }

    fn handle_promo(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::MouseDown {
            if let Some(i) = self.promo_rects().iter().position(|r| r.hit(ev.pos)) {
                self.finish_promo(i);
                return;
            }
            if !self.board_rect.hit(ev.pos) {
                return;
            }
            self.promo = None;
        } else if ev.kind == Kind::KeyDown {
            let k = ev.key.to_lowercase();
            let idx = match k.as_str() {
                "q" | "d" | "1" => Some(0),
                "r" | "t" | "2" => Some(1),
                "b" | "l" | "3" => Some(2),
                "n" | "s" | "4" => Some(3),
                _ => None,
            };
            if let Some(i) = idx {
                self.finish_promo(i);
            } else if k == "escape" || k == "backspace" {
                self.promo = None;
            }
        }
    }

    fn finish_promo(&mut self, i: usize) {
        let cands = self.promo.take();
        if let Some(c) = cands {
            if i < c.len() {
                let anim = self.promo_animate;
                self.human_move(c[i], anim);
            }
        }
    }

    pub(crate) fn dialog_rects(&self) -> (Rect, Rect, Rect) {
        let w = (self.bw - 16).min(240.max((self.bw as f64 * 0.8) as i32));
        let h = self.big.get_height() + self.small.get_height() + 30.max(self.b.height / 15) + 40;
        let panel = Rect::new(self.bx + py::fdiv32(self.bw - w, 2), self.by + py::fdiv32(self.bw - h, 2), w, h);
        let bh = 26.max(self.b.height / 16);
        let bw = py::fdiv32(w - 36, 2);
        let yes = Rect::new(panel.x + 12, panel.bottom() - 12 - bh, bw, bh);
        let no = Rect::new(panel.right() - 12 - bw, panel.bottom() - 12 - bh, bw, bh);
        (panel, yes, no)
    }

    fn handle_dialog(&mut self, ev: &InputEvent) {
        let mut answer = None;
        if ev.kind == Kind::MouseDown {
            let (_, yes, no) = self.dialog_rects();
            if yes.hit(ev.pos) {
                answer = Some(true);
            } else if no.hit(ev.pos) {
                answer = Some(false);
            }
        } else if ev.kind == Kind::KeyDown {
            let k = ev.key.as_str();
            if ["Return", "KP_Enter", "j", "J", "y", "Y"].contains(&k) {
                answer = Some(true);
            } else if ["Escape", "n", "N", "BackSpace"].contains(&k) {
                answer = Some(false);
            }
        }
        let Some(answer) = answer else {
            return;
        };
        let Some((kind, side)) = self.dialog.take() else {
            return;
        };
        self.b.play_sound("click");
        if kind == "resign" && answer {
            self.finish(Some(1 - side), "resign", None);
        } else if kind == "draw" {
            if answer {
                self.finish(None, "agreed", None);
            } else {
                self.toast(t("chess.draw_declined"), 2.6);
            }
        }
    }

    fn handle_over(&mut self, ev: &InputEvent) {
        match ev.kind {
            Kind::KeyDown => {
                let k = ev.key.as_str();
                if ["Return", "space", "KP_Enter"].contains(&k) {
                    self.restart();
                } else if k == "s" || k == "S" {
                    self.to_setup();
                } else if self.fixed_key(k, &["p", "P"]) {
                    self.export_pgn();
                } else if self.fixed_key(k, &["f", "F"]) {
                    self.flip_view();
                } else if ["Prior", "Next", "Up", "Down"].contains(&k) {
                    self.scroll_list(if k == "Prior" || k == "Up" { -3 } else { 3 });
                }
            }
            Kind::MouseDown => {
                if let Some(&(bid, _)) = self.over_btns.iter().find(|(_, r)| r.hit(ev.pos)) {
                    match bid {
                        "new" => self.restart(),
                        "setup" => self.to_setup(),
                        _ => self.export_pgn(),
                    }
                }
            }
            Kind::Wheel => {
                if self.list_rect.hit(ev.pos) {
                    self.scroll_list(-ev.delta as i64 * 2);
                }
            }
            Kind::MouseMove => {
                self.hover = Hover::None;
                for &(bid, rc) in &self.over_btns {
                    if rc.hit(ev.pos) {
                        self.hover = Hover::Over(bid);
                    }
                }
            }
            _ => {}
        }
    }

    fn restart(&mut self) {
        self.b.game_over = false;
        if self.b.mode == "single" {
            self.human_color = 1 - self.human_color;
            let c = if self.human_color == ce::BLACK { "black" } else { "white" };
            self.save_setting("color", json!(c));
        }
        self.new_game();
        self.state = St::Play;
        self.b.play_sound("click");
    }

    fn to_setup(&mut self) {
        self.b.game_over = false;
        self.state = St::Setup;
        self.b.play_sound("click");
    }

    pub(crate) fn list_rows(&self) -> i64 {
        (self.sans.len() as i64 + self.start_side as i64 + 1) / 2
    }

    pub(crate) fn list_visible(&self) -> i64 {
        let rect = if self.state == St::Puz { self.puz_list_rect } else { self.list_rect };
        let head = self.tiny.get_height() + 10;
        1.max(py::fdiv((rect.h - head) as i64, self.row_h as i64))
    }

    fn scroll_list(&mut self, rows: i64) {
        let total = self.list_rows();
        let visible = self.list_visible();
        let top = 0.max((total - visible).min(self.list_top + rows));
        self.list_top = top;
        self.list_follow = top >= total - visible;
    }

    // ===================================================== Zuege

    fn commit(&mut self, m: i32, animate: bool) {
        let mover = self.pos.side;
        let to = (m >> 8) & 255;
        let flag = m >> 20;
        let (cap, cap_sq) = if flag == ce::M_EP {
            (ce::PAWN | ((1 - mover) << 3), if mover == ce::WHITE { to - 16 } else { to + 16 })
        } else if flag == ce::M_CASTLE {
            (0, -1)
        } else {
            (self.pos.b[to as usize], to)
        };
        let legal = self.legal.clone();
        let san = self.pos.san(m, Some(&legal));
        if animate {
            self.animate(m);
        }
        if cap != 0 {
            self.fades.push(Fade { p: cap, sq: cap_sq, t0: Self::now() });
        }
        self.sans.push(san);
        let u = self.pos.uci(m);
        self.ucis.push(u);
        self.pos.make(m);
        self.moves.push(m);
        self.caps.push(cap);
        self.last_move = Some(Self::move_squares(m));
        self.sel = None;
        self.targets.clear();
        self.drag = None;
        self.hint_move = 0;
        if self.hint_job.is_some() {
            self.abort_hint();
        }
        if self.moves.len() > 1 {
            if let Some(c) = self.clock.as_mut() {
                c[mover as usize] += self.inc;
            }
        }
        self.refresh();
        if self.list_follow {
            self.scroll_to_end();
        }
        if self.check {
            self.b.play_sound("select");
        } else if flag == ce::M_CASTLE {
            self.b.play_sound("rotate");
        } else if cap != 0 {
            self.b.play_sound("lock");
            self.b.rumble(40);
        } else {
            self.b.play_sound("move");
        }
        self.ai_wait = 0.0;
        if self.state != St::Play {
            return;
        }
        let status = if self.legal.is_empty() {
            Some(if self.check { "checkmate" } else { "stalemate" })
        } else if self.pos.insufficient_material() {
            Some("material")
        } else if self.pos.half >= 100 {
            Some("fifty")
        } else if self.pos.repetitions() >= 3 {
            Some("threefold")
        } else {
            None
        };
        if let Some(st) = status {
            self.finish(if st == "checkmate" { Some(mover) } else { None }, st, None);
            return;
        }
        if self.b.mode == "multi" && self.auto_flip {
            self.flip_pending = ANIM_TIME + 0.25;
        }
    }

    fn scroll_to_end(&mut self) {
        self.list_top = 0.max(self.list_rows() - self.list_visible());
        self.list_follow = true;
    }

    fn animate(&mut self, m: i32) {
        let frm = m & 255;
        let to = (m >> 8) & 255;
        let now = Self::now();
        if m >> 20 == ce::M_CASTLE {
            let base = frm & 0x70;
            let (kt, rt) = if to > frm { (base + 6, base + 5) } else { (base + 2, base + 3) };
            self.anims = vec![Anim { p: self.pos.b[frm as usize], a: frm, b: kt, t0: now }, Anim { p: self.pos.b[to as usize], a: to, b: rt, t0: now }];
        } else {
            let promo = (m >> 16) & 7;
            let p = if promo != 0 { promo | (self.pos.side << 3) } else { self.pos.b[frm as usize] };
            self.anims = vec![Anim { p, a: frm, b: to, t0: now }];
        }
    }

    pub(crate) fn now() -> f64 {
        pg::ticks() as f64 / 1000.0
    }

    fn undo(&mut self) {
        if self.state != St::Play || self.moves.is_empty() {
            return;
        }
        let n;
        if self.b.mode == "single" {
            let human_moves = (0..self.moves.len()).filter(|i| (*i % 2) as i32 == self.human_color).count();
            if human_moves == 0 {
                self.toast(t("chess.undo_none"), 2.6);
                return;
            }
            n = if self.pos.side == self.human_color { 2 } else { 1 };
        } else {
            n = 1;
        }
        self.abort_jobs();
        for _ in 0..n.min(self.moves.len()) {
            self.pos.unmake();
            self.moves.pop();
            self.sans.pop();
            self.ucis.pop();
            self.caps.pop();
        }
        if !self.assisted {
            self.toast(t("chess.assisted"), 2.6);
        } else {
            self.toast(t("chess.undone"), 2.6);
        }
        self.assisted = true;
        self.last_move = self.moves.last().map(|m| Self::move_squares(*m));
        self.sel = None;
        self.targets.clear();
        self.hint_move = 0;
        self.anims.clear();
        self.fades.clear();
        self.ai_wait = 0.0;
        self.draw_block = self.draw_block.min(self.moves.len());
        self.refresh();
        self.scroll_to_end();
        if self.b.mode == "multi" && self.auto_flip {
            self.view_black = self.pos.side == ce::BLACK;
        }
        self.b.play_sound("rotate");
    }

    fn flip_view(&mut self) {
        self.view_black = !self.view_black;
        self.flip_anim = Some(Self::now());
        self.b.play_sound("rotate");
    }

    // ----------------------------------------------------------- Hinweis

    fn request_hint(&mut self) {
        if self.state != St::Play || self.hint_job.is_some() || self.legal.is_empty() {
            return;
        }
        if self.b.mode == "single" && self.pos.side != self.human_color {
            return;
        }
        if !self.assisted {
            self.toast(t("chess.assisted"), 2.6);
        }
        self.assisted = true;
        let tt = self.take_tt();
        self.hint_job = Some(Job::Ready(Box::new(Search::new(&self.pos, &ce::HINT_LEVEL, tt))));
        self.b.play_sound("select");
    }

    fn abort_hint(&mut self) {
        if let Some(job) = self.hint_job.take() {
            self.drop_job(job);
        }
    }

    fn step_hint(&mut self) {
        let Some(job) = self.hint_job.take() else {
            return;
        };
        let job = self.step_job(job);
        let finished = match &job {
            Job::Done(s) => Some(s.best_move),
            Job::Running { stop, start, .. } => {
                if start.elapsed().as_secs_f64() >= ce::HINT_LEVEL.time {
                    stop.store(true, Ordering::Relaxed);
                }
                None
            }
            Job::Ready(_) => None,
        };
        match finished {
            Some(best) => {
                self.drop_job(job);
                if best != 0 {
                    self.hint_move = best;
                    let legal = self.legal.clone();
                    let san = self.pos.san(best, Some(&legal));
                    self.toast(tf("chess.hint_move", &[("move", &san)]), 2.6);
                }
            }
            None => self.hint_job = Some(job),
        }
    }

    // ----------------------------------------------------------- Remis / Aufgeben

    fn offer_draw(&mut self) {
        if self.state != St::Play || self.moves.is_empty() {
            return;
        }
        if self.b.mode == "multi" {
            self.dialog = Some(("draw", self.pos.side));
            self.b.play_sound("select");
            return;
        }
        if self.moves.len() < self.draw_block {
            self.toast(t("chess.draw_wait"), 2.6);
            return;
        }
        let ai = 1 - self.human_color;
        let score = match self.ai_last_score {
            Some(s) => s,
            None => {
                let e = self.pos.evaluate();
                if self.pos.side == ai {
                    e
                } else {
                    -e
                }
            }
        };
        if score <= 20 {
            self.toast(t("chess.draw_accepted"), 2.6);
            self.finish(None, "agreed", None);
        } else {
            self.draw_block = self.moves.len() + 6;
            self.toast(t("chess.draw_declined_ai"), 2.6);
            self.b.play_sound("hit");
        }
    }

    fn ask_resign(&mut self) {
        if self.state != St::Play {
            return;
        }
        let side = if self.b.mode == "single" { self.human_color } else { self.pos.side };
        self.dialog = Some(("resign", side));
        self.b.play_sound("select");
    }

    // ----------------------------------------------------------- Ende

    fn finish(&mut self, winner: Option<i32>, reason: &'static str, loser: Option<i32>) {
        self.abort_jobs();
        self.state = St::Over;
        self.result = Some((winner, reason));
        self.result_loser = loser.or(winner.map(|w| 1 - w));
        self.dialog = None;
        self.promo = None;
        self.sel = None;
        self.targets.clear();
        self.drag = None;
        self.over_t0 = Self::now();
        if reason == "checkmate" {
            let loser = 1 - winner.unwrap_or(0);
            let r = self.sq_rect(self.pos.kings[loser as usize]);
            ui::spawn_burst(r.centerx() as f64, r.centery() as f64, Some(ui::pal().gold), 30, 260.0);
        }
        if self.b.mode == "multi" {
            if let Some(w) = winner {
                self.wins[w as usize] += 1;
                self.b.report_duel(if w == ce::WHITE { 0 } else { 1 });
                self.b.play_sound("win");
            } else {
                self.b.play_sound("select");
            }
        } else if winner == Some(self.human_color) {
            if !self.assisted {
                self.human_wins += 1;
                self.b.score = self.human_wins;
                self.b.report_result(true);
                self.b.ach_event("chess_win", None);
                if self.diff == 5 {
                    self.b.ach_event("chess_master", None);
                }
            }
            self.b.play_sound("win");
        } else if winner.is_none() {
            self.b.play_sound("select");
        } else {
            self.b.report_result(false);
            self.b.play_sound("gameover");
        }
        self.b.game_over = true;
    }

    fn pgn(&self) -> String {
        let (winner, reason) = match self.result {
            Some((w, r)) => (Some(w), Some(r)),
            None => (None, None),
        };
        let res = ce::pgn_result_code(winner);
        let (white, black) = if self.b.mode == "single" {
            let ai = format!("{} ({})", t("common.ai"), t(&format!("chess.diff.{}", DIFFS[self.diff])));
            let you = t("chess.you");
            if self.human_color == ce::WHITE {
                (you, ai)
            } else {
                (ai, you)
            }
        } else {
            (self.b.pname(0), self.b.pname(1))
        };
        let mut tags: Vec<(String, String)> = vec![
            ("Event".into(), "PyGameZ".into()),
            ("Site".into(), "PyGameZ".into()),
            ("Date".into(), chrono::Local::now().format("%Y.%m.%d").to_string()),
            ("Round".into(), "-".into()),
            ("White".into(), white),
            ("Black".into(), black),
            ("Result".into(), res.into()),
        ];
        if self.c960_n.is_some() {
            tags.push(("Variant".into(), "Chess960".into()));
            tags.push(("SetUp".into(), "1".into()));
            tags.push(("FEN".into(), self.start_fen.clone()));
        }
        if let Some((base, inc)) = clock_time(self.clock_opt) {
            tags.push(("TimeControl".into(), format!("{}+{}", base as i64, inc as i64)));
        }
        if reason == Some("timeout") || reason == Some("timeout_draw") {
            tags.push(("Termination".into(), "time forfeit".into()));
        }
        ce::pgn_text(&tags, &self.sans, res, ce::WHITE, 1)
    }

    fn export_pgn(&mut self) {
        if self.sans.is_empty() {
            self.toast(t("chess.pgn_empty"), 2.6);
            return;
        }
        let name = format!("pygamez-chess-{}.pgn", chrono::Local::now().format("%Y-%m-%d-%H%M"));
        let path = if filepick::available() {
            match filepick::save_as(&name, &t("chess.pgn_title"), &[".pgn"]) {
                Some(p) => p,
                None => return,
            }
        } else {
            filepick::to_downloads(&name)
        };
        if std::fs::write(&path, self.pgn()).is_err() {
            self.toast(t("chess.pgn_failed"), 2.6);
            self.b.play_sound("hit");
            return;
        }
        self.toast(tf("chess.pgn_saved", &[("path", &filepick::short(&path, 34))]), 4.0);
        self.b.play_sound("point");
    }

    pub(crate) fn toast(&mut self, text: String, dur: f64) {
        self.msg = Some(text);
        self.msg_t = dur;
    }

    // ===================================================== Update

    fn tick_clock(&mut self, dt: f64) {
        let side = self.pos.side as usize;
        let Some(clock) = self.clock.as_mut() else {
            return;
        };
        clock[side] -= dt;
        let left = clock[side];
        if left <= 10.0 && !self.low_time_warned[side] && left > 0.0 {
            self.low_time_warned[side] = true;
            if self.b.mode == "multi" || side as i32 == self.human_color {
                self.b.play_sound("hit");
            }
        }
        if left <= 0.0 {
            if let Some(c) = self.clock.as_mut() {
                c[side] = 0.0;
            }
            let winner = 1 - side as i32;
            if self.pos.can_mate(winner) {
                self.finish(Some(winner), "timeout", None);
            } else {
                self.finish(None, "timeout_draw", Some(side as i32));
            }
        }
    }

    fn step_ai(&mut self, dt: f64) {
        self.ai_wait += dt;
        if self.ai_job.is_none() {
            if self.ai_wait < 0.12 || self.legal.is_empty() {
                return;
            }
            self.start_ai();
        }
        let Some(mut job) = self.ai_job.take() else {
            return;
        };
        if job.mv != 0 {
            if self.ai_wait >= job.wait {
                self.commit(job.mv, true);
            } else {
                self.ai_job = Some(job);
            }
            return;
        }
        let lvl = ce::LEVELS[self.diff];
        if let Some(s) = job.search.take() {
            job.search = Some(self.step_job(s));
        }
        let done = match &job.search {
            Some(Job::Done(_)) => true,
            Some(Job::Running { stop, start, .. }) => {
                if start.elapsed().as_secs_f64() >= lvl.time || self.ai_wait >= job.wall_cap {
                    stop.store(true, Ordering::Relaxed);
                }
                false
            }
            _ => false,
        };
        if done && self.ai_wait >= AI_MIN_WAIT {
            let (m, depth_done, score) = match &job.search {
                Some(Job::Done(s)) => (s.best_move, s.depth_done, s.best_score),
                _ => (0, 0, 0),
            };
            let m = if m != 0 { m } else { self.legal[0] };
            if depth_done >= 1 {
                self.ai_last_score = Some(score);
            }
            if let Some(s) = job.search.take() {
                self.drop_job(s);
            }
            self.abort_jobs();
            self.commit(m, true);
            return;
        }
        self.ai_job = Some(job);
    }

    fn start_ai(&mut self) {
        let lvl = ce::LEVELS[self.diff];
        let mut job = AiJob { mv: 0, wait: AI_MIN_WAIT, search: None, wall_cap: f64::INFINITY };
        if self.diff == 0 || py::random() < lvl.random {
            job.mv = ce::pick_weak_move(&mut self.pos);
            job.wait = py::uniform(0.5, 0.9);
        } else if lvl.book && self.c960_n.is_none() && self.moves.len() < 16 {
            let cands: Vec<i32> = ce::book_moves(&self.ucis).iter().map(|u| self.pos.parse_uci(u)).filter(|c| *c != 0).collect();
            if !cands.is_empty() {
                job.mv = py::choice(&cands);
                job.wait = py::uniform(0.45, 0.8);
            }
        }
        if job.mv == 0 {
            let tt = self.take_tt();
            job.search = Some(Job::Ready(Box::new(Search::new(&self.pos, &lvl, tt))));
            if let Some(c) = self.clock {
                let left = c[self.pos.side as usize];
                job.wall_cap = 0.2f64.max(10f64.min(left / 30.0 + self.inc * 0.7));
            }
        }
        self.ai_job = Some(job);
    }

    // ===================================================== Raetsel

    fn puz_load_progress(&mut self) {
        let data = store::load_section("chess");
        let strs = |v: Option<&Value>| -> HashSet<String> {
            match v {
                Some(Value::Array(a)) => a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect(),
                _ => HashSet::new(),
            }
        };
        self.puz_solved = strs(data.get("puzzles_solved"));
        self.puz_seen = strs(data.get("puzzles_seen"));
        self.puz_stage = match data.get("puzzle_stage") {
            Some(Value::Number(n)) if n.is_i64() || n.is_u64() => n.as_i64().filter(|s| (0..STAGES.len() as i64).contains(s)).unwrap_or(0) as usize,
            Some(Value::Bool(b)) => *b as usize,
            _ => 0,
        };
    }

    fn puz_save_progress(&self) {
        let mut data = store::load_section("chess");
        let mut solved: Vec<&String> = self.puz_solved.iter().collect();
        solved.sort();
        let mut seen: Vec<&String> = self.puz_seen.iter().collect();
        seen.sort();
        data.insert("puzzles_solved".into(), json!(solved));
        data.insert("puzzles_seen".into(), json!(seen));
        data.insert("puzzle_stage".into(), json!(self.puz_stage));
        store::save_section("chess", data);
    }

    pub(crate) fn stage_list(&self, stage: Option<usize>) -> &Vec<Puzzle> {
        &self.puzzles[stage.unwrap_or(self.puz_stage)]
    }

    pub(crate) fn stage_solved(&self, stage: usize) -> usize {
        self.stage_list(Some(stage)).iter().filter(|p| self.puz_solved.contains(&p.id)).count()
    }

    fn puz_menu_focus_unsolved(&mut self) {
        self.puz_cursor = self.stage_list(None).iter().position(|p| !self.puz_solved.contains(&p.id)).unwrap_or(0);
    }

    fn handle_puz_menu(&mut self, ev: &InputEvent) {
        let n = self.stage_list(None).len();
        match ev.kind {
            Kind::KeyDown => {
                let k = ev.key.as_str();
                let act = |a: &str| self.b.is_action(k, a, None);
                if ["1", "2", "3", "4", "5"].contains(&k) {
                    self.puz_set_stage(k.parse::<usize>().unwrap_or(1) - 1);
                } else if ["Tab", "Next", "Prior"].contains(&k) {
                    let step = if k == "Prior" { -1 } else { 1 };
                    self.puz_set_stage(py::pmod(self.puz_stage as i64 + step, STAGES.len() as i64) as usize);
                } else if n != 0 && (k == "Left" || act("left")) {
                    self.puz_cursor = py::pmod(self.puz_cursor as i64 - 1, n as i64) as usize;
                    self.b.play_sound("move");
                } else if n != 0 && (k == "Right" || act("right")) {
                    self.puz_cursor = (self.puz_cursor + 1) % n;
                    self.b.play_sound("move");
                } else if n != 0 && (k == "Up" || act("up")) {
                    self.puz_cursor = py::pmod(self.puz_cursor as i64 - PUZ_COLS as i64, n as i64) as usize;
                    self.b.play_sound("move");
                } else if n != 0 && (k == "Down" || act("down")) {
                    self.puz_cursor = (self.puz_cursor + PUZ_COLS) % n;
                    self.b.play_sound("move");
                } else if n != 0 && (["Return", "space", "KP_Enter"].contains(&k) || act("action")) {
                    self.puz_open(self.puz_stage, self.puz_cursor);
                }
            }
            Kind::MouseDown => {
                if let Some(i) = self.stage_rects.iter().position(|r| r.hit(ev.pos)) {
                    self.puz_set_stage(i);
                    return;
                }
                if let Some(i) = self.grid_rects.iter().take(n).position(|r| r.hit(ev.pos)) {
                    self.puz_cursor = i;
                    self.puz_open(self.puz_stage, i);
                    return;
                }
                if n != 0 && self.puz_start_rect.hit(ev.pos) {
                    self.puz_open(self.puz_stage, self.puz_cursor);
                }
            }
            Kind::MouseMove => {
                self.hover = Hover::None;
                for (i, rc) in self.grid_rects.iter().take(n).enumerate() {
                    if rc.hit(ev.pos) {
                        self.hover = Hover::Cell(i);
                    }
                }
            }
            _ => {}
        }
    }

    fn puz_set_stage(&mut self, i: usize) {
        if i == self.puz_stage {
            return;
        }
        self.puz_stage = i;
        self.puz_menu_focus_unsolved();
        self.b.play_sound("select");
    }

    fn puz_open(&mut self, stage: usize, idx: usize) {
        let lst = self.stage_list(Some(stage));
        if lst.is_empty() {
            return;
        }
        let idx = idx.min(lst.len() - 1);
        let p = lst[idx].clone();
        let Ok(pos) = ce::from_fen(&p.fen, Some(false)) else {
            return;
        };
        self.abort_jobs();
        self.puz_stage = stage;
        self.puz_idx = idx;
        self.puz_cursor = idx;
        self.pos = pos;
        self.start_fen = self.pos.fen();
        self.start_side = self.pos.side;
        self.start_full = self.pos.full;
        self.list_top = 0;
        self.list_follow = true;
        self.c960_n = None;
        self.clock = None;
        self.moves.clear();
        self.sans.clear();
        self.ucis.clear();
        self.caps.clear();
        self.last_move = None;
        self.sel = None;
        self.targets.clear();
        self.drag = None;
        self.promo = None;
        self.dialog = None;
        self.hint_move = 0;
        self.anims.clear();
        self.fades.clear();
        self.puz_line = p.moves.clone();
        self.puz = Some(p);
        self.puz_step = 0;
        self.puz_player = 1 - self.pos.side;
        self.puz_mate = mate_n(STAGES[stage]);
        self.puz_status = "intro";
        self.puz_timer = 0.7;
        self.puz_mistakes = 0;
        self.puz_shown = false;
        self.puz_feedback = ("watch", None);
        self.view_black = self.puz_player == ce::BLACK;
        self.cursor = [6, 4];
        self.state = St::Puz;
        self.refresh();
        self.b.play_sound("click");
    }

    fn puz_take_back(&mut self) {
        self.pos.unmake();
        self.moves.pop();
        self.sans.pop();
        self.ucis.pop();
        self.caps.pop();
    }

    fn puz_update(&mut self, dt: f64) {
        if self.puz_timer > 0.0 {
            self.puz_timer -= dt;
            if self.puz_timer > 0.0 {
                return;
            }
        }
        match self.puz_status {
            "intro" => {
                let first = self.puz_line[0].clone();
                let m = self.pos.parse_uci(&first);
                if m == 0 {
                    self.puz_status = "shown_done";
                    return;
                }
                self.commit(m, true);
                self.puz_step = 1;
                self.puz_status = "play";
                self.puz_feedback = ("your_move", None);
            }
            "reply" => {
                let u = self.puz_line[self.puz_step].clone();
                let m = self.pos.parse_uci(&u);
                if m != 0 {
                    self.commit(m, true);
                }
                self.puz_step += 1;
                self.puz_status = "play";
                self.puz_feedback = ("good", Some(ui::pal().green));
            }
            "wrong" => {
                if !self.moves.is_empty() {
                    self.puz_take_back();
                    self.last_move = self.moves.last().map(|m| Self::move_squares(*m));
                    self.anims.clear();
                    self.refresh();
                }
                self.puz_status = "play";
                self.puz_feedback = ("retry", Some(ui::pal().red));
            }
            "shown" => {
                if self.puz_step < self.puz_line.len() {
                    let u = self.puz_line[self.puz_step].clone();
                    let m = self.pos.parse_uci(&u);
                    if m != 0 {
                        self.commit(m, true);
                    }
                    self.puz_step += 1;
                    self.puz_timer = 0.85;
                } else {
                    self.puz_status = "shown_done";
                    self.puz_feedback = ("shown", None);
                }
            }
            _ => {}
        }
    }

    fn puz_player_move(&mut self, m: i32, animate: bool) {
        if self.puz_status != "play" || self.puz_step >= self.puz_line.len() {
            return;
        }
        let u = self.puz_line[self.puz_step].clone();
        let expected = self.pos.parse_uci(&u);
        let mate = ce::gives_mate(&mut self.pos, m);
        if m == expected || mate {
            self.commit(m, animate);
            self.puz_step += 1;
            if mate || self.puz_step >= self.puz_line.len() {
                self.puz_solved_now();
            } else {
                self.puz_status = "reply";
                self.puz_timer = 0.5;
                self.puz_feedback = ("good", Some(ui::pal().green));
                self.b.play_sound("point");
            }
        } else {
            self.commit(m, animate);
            self.puz_status = "wrong";
            self.puz_timer = 0.85;
            self.puz_mistakes += 1;
            self.puz_feedback = ("wrong", Some(ui::pal().red));
            self.b.play_sound("hit");
            self.b.rumble(90);
        }
    }

    fn puz_solved_now(&mut self) {
        self.puz_status = "solved";
        self.puz_feedback = ("solved", Some(ui::pal().green));
        let pid = self.puz.as_ref().map(|p| p.id.clone()).unwrap_or_default();
        if !self.puz_shown && !self.puz_solved.contains(&pid) {
            self.puz_solved.insert(pid);
            self.puz_save_progress();
            self.b.ach_event("chess_puzzles", Some(self.puz_solved.len() as i64));
        }
        self.b.play_sound("win");
        ui::spawn_burst(self.board_rect.centerx() as f64, self.board_rect.centery() as f64, Some(ui::pal().gold), 34, 260.0);
    }

    fn puz_show_solution(&mut self) {
        if self.state != St::Puz || ["solved", "shown", "shown_done"].contains(&self.puz_status) {
            return;
        }
        if self.puz_status == "wrong" && !self.moves.is_empty() {
            self.puz_take_back();
            self.refresh();
        }
        if self.puz_step == 0 {
            let u = self.puz_line[0].clone();
            let m = self.pos.parse_uci(&u);
            if m != 0 {
                self.commit(m, false);
            }
            self.puz_step = 1;
        }
        self.puz_shown = true;
        let pid = self.puz.as_ref().map(|p| p.id.clone()).unwrap_or_default();
        if !self.puz_solved.contains(&pid) {
            self.puz_seen.insert(pid);
            self.puz_save_progress();
        }
        self.sel = None;
        self.targets.clear();
        self.puz_status = "shown";
        self.puz_timer = 0.45;
        self.puz_feedback = ("showing", None);
        self.b.play_sound("select");
    }

    fn puz_next(&mut self) {
        if self.state != St::Puz {
            return;
        }
        let n = self.stage_list(None).len();
        if self.puz_idx + 1 < n {
            self.puz_open(self.puz_stage, self.puz_idx + 1);
        } else if self.puz_stage + 1 < STAGES.len() {
            self.puz_open(self.puz_stage + 1, 0);
        } else {
            self.puz_back_to_menu();
        }
    }

    fn puz_back_to_menu(&mut self) {
        self.abort_jobs();
        self.state = St::PuzMenu;
        self.promo = None;
        self.dialog = None;
        self.puz_save_progress();
        self.b.play_sound("click");
    }
}

impl Game for ChessGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn show_highscore_banner(&self) -> bool {
        self.b.mode == "single"
    }

    fn wants_escape(&self) -> bool {
        matches!(self.state, St::Play | St::Puz) && (self.promo.is_some() || self.dialog.is_some())
    }

    fn reset(&mut self) {
        self.b.score = 0;
        self.b.game_over = false;
        if !["single", "multi", "puzzles"].contains(&self.b.mode.as_str()) {
            self.b.mode = "single".into();
        }
        self.b.multiplayer = self.b.mode == "multi";
        self.diff = match self.b.opt_value("chess", "difficulty") {
            None => 2,
            Some(_) => self.b.opt_int_py("chess", "difficulty", 2).map(|v| 0.max(5.min(v)) as usize).unwrap_or(2),
        };
        self.human_color = if self.b.opt_value("chess", "color").and_then(|v| v.as_str().map(|s| s == "black")).unwrap_or(false) { ce::BLACK } else { ce::WHITE };
        let clock = self.b.opt_value("chess", "clock").and_then(|v| v.as_str().map(|s| s.to_string()));
        self.clock_opt = CLOCKS.iter().copied().find(|c| Some(c.to_string()) == clock).unwrap_or("none");
        self.c960 = self.b.opt_value("chess", "chess960") == Some(Value::Bool(true));
        self.auto_flip = self.b.opt_value("chess", "flip") == Some(Value::Bool(true));
        self.human_wins = 0;
        self.wins = [0, 0];
        self.abort_jobs();
        self.tt = Some(Tt::new());
        self.ai_job = None;
        self.hint_job = None;
        self.dialog = None;
        self.promo = None;
        self.msg = None;
        self.msg_t = 0.0;
        self.hover = Hover::None;
        self.setup_focus = 0;
        self.anims.clear();
        self.fades.clear();
        self.flip_anim = None;
        self.list_top = 0;
        self.list_follow = true;
        self.reset_caches();
        self.make_fonts();
        self.layout();
        self.build_setup_layout();
        if self.puzzles.is_empty() {
            self.puzzles = load_puzzles();
        }
        self.puz_load_progress();
        self.puz = None;
        self.puz_cursor = 0;
        self.build_puz_menu_layout();
        self.new_game();
        if self.b.mode == "puzzles" {
            self.state = St::PuzMenu;
            self.puz_menu_focus_unsolved();
        } else {
            self.state = St::Setup;
        }
    }

    fn on_surface_changed(&mut self) {
        self.reset_caches();
        self.make_fonts();
        self.layout();
        self.build_setup_layout();
        self.build_puz_menu_layout();
    }

    fn on_exit(&mut self) {
        self.abort_jobs();
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        match self.state {
            St::Setup => self.handle_setup(ev),
            St::PuzMenu => self.handle_puz_menu(ev),
            St::Over => self.handle_over(ev),
            St::Play | St::Puz => self.handle_board_state(ev),
        }
    }

    fn update(&mut self, dt: f64) {
        if self.msg_t > 0.0 {
            self.msg_t -= dt;
            if self.msg_t <= 0.0 {
                self.msg = None;
            }
        }
        if self.state == St::Play {
            if self.flip_pending > 0.0 {
                self.flip_pending -= dt;
                if self.flip_pending <= 0.0 && self.view_black != (self.pos.side == ce::BLACK) {
                    self.flip_view();
                }
            }
            if self.clock.is_some() && !self.moves.is_empty() && self.dialog.is_none() {
                self.tick_clock(dt);
                if self.state != St::Play {
                    return;
                }
            }
            if self.hint_job.is_some() {
                self.step_hint();
            }
            if self.b.mode == "single" && self.pos.side != self.human_color && self.dialog.is_none() {
                self.step_ai(dt);
            }
        } else if self.state == St::Puz {
            self.puz_update(dt);
        }
    }

    fn draw(&mut self) {
        self.draw_all();
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "chess",
    class_name: "ChessGame",
    supports_multiplayer: true,
    modes: &[("single", "chess.mode.single"), ("multi", "chess.mode.multi"), ("puzzles", "chess.mode.puzzles")],
    make: ChessGame::new,
};
