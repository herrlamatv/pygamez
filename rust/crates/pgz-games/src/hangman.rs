//! hangman.rs
//! ==========
//! Galgenmaennchen - errate das Wort, bevor der Galgen fertig ist.
//!
//! Uebertragen aus `games/hangman.py` und `games/hangman_words.py`
//! (Wortlisten erzeugt nach `data/hangman_words.rs`). Drei Laengen-Modi,
//! Endlos-Serie, Bildschirmtastatur (QWERTZ fuer Deutsch).

use pgz_engine::prelude::*;

use crate::data::hangman_words::{RANGES, WORDS};

const COL_KEY: Color = Color::rgb(70, 72, 86);
const COL_KEY_TEXT: Color = Color::rgb(232, 233, 240);
const COL_CORRECT: Color = Color::rgb(106, 190, 120);
const COL_ABSENT: Color = Color::rgb(58, 58, 64);
const COL_WORD: Color = Color::rgb(236, 238, 246);
const MAX_WRONG: i64 = 6;
const QWERTY: [&str; 3] = ["QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"];
const QWERTZ: [&str; 3] = ["QWERTZUIOP", "ASDFGHJKL", "YXCVBNM"];

/// `hangman_words.words_for(lang, mode)`
pub fn words_for(lang: &str, mode: &str) -> Vec<String> {
    let (lo, hi) = RANGES
        .iter()
        .find(|(m, _, _)| *m == mode)
        .or_else(|| RANGES.iter().find(|(m, _, _)| *m == "mixed"))
        .map(|(_, a, b)| (*a, *b))
        .unwrap_or((3, 12));
    let roh = |l: &str| WORDS.iter().find(|(x, _)| *x == l).map(|(_, w)| *w);
    let raw = roh(lang).filter(|w| !w.is_empty()).or_else(|| roh("en")).unwrap_or(&[]);
    let mut out: Vec<String> = Vec::new();
    for w in raw {
        let u = w.trim().to_uppercase();
        let n = u.chars().count();
        if lo <= n && n <= hi && u.is_ascii() && !u.is_empty() && u.chars().all(|c| c.is_alphabetic()) && !out.contains(&u) {
            out.push(u);
        }
    }
    if out.is_empty() {
        let en = roh("en").unwrap_or(&[]);
        out = en.iter().filter(|w| lo <= w.len() && w.len() <= hi).map(|w| w.to_string()).collect();
        if out.is_empty() {
            out = en.iter().map(|w| w.to_string()).collect();
        }
    }
    out
}

#[derive(PartialEq, Clone, Copy)]
enum Zustand {
    Play,
    Solved,
    Over,
}

/// `HangmanGame`
pub struct HangmanGame {
    b: GameBase,
    lang: String,
    words: Vec<String>,
    solved_count: i64,
    word: String,
    guessed: Vec<char>,
    wrong: i64,
    last_points: i64,
    state: Zustand,
    key_h: i32,
    key_gap: i32,
    kb_top: i32,
    key_rects: Vec<(char, Rect)>,
    word_y: i32,
    draw_top: i32,
    draw_bottom: i32,
    hud: Font,
    small: Font,
    key_font: Font,
    word_font: Font,
    huge: Font,
}

impl HangmanGame {
    /// `HangmanGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let mut g = HangmanGame {
            b,
            lang: String::new(),
            words: Vec::new(),
            solved_count: 0,
            word: String::new(),
            guessed: Vec::new(),
            wrong: 0,
            last_points: 0,
            state: Zustand::Play,
            key_h: 0,
            key_gap: 0,
            kb_top: 0,
            key_rects: Vec::new(),
            word_y: 0,
            draw_top: 0,
            draw_bottom: 0,
            hud: f.clone(),
            small: f.clone(),
            key_font: f.clone(),
            word_font: f.clone(),
            huge: f,
        };
        g.reset();
        Box::new(g)
    }

    fn make_fonts(&mut self) {
        let h = self.b.height;
        self.hud = ui::fb(20);
        self.small = ui::f(15);
        self.key_font = ui::fb(16);
        self.word_font = ui::fb(30.max(h / 12));
        self.huge = ui::fb(28.max(h / 12));
    }

    fn new_word(&mut self) {
        self.word = py::choice(&self.words);
        self.guessed.clear();
        self.wrong = 0;
        self.last_points = 0;
        self.state = Zustand::Play;
    }

    fn layout(&mut self) {
        self.key_h = 30.max(self.b.height / 12);
        self.key_gap = 3.max(self.b.width / 160);
        let kb_h = 3 * self.key_h + 4 * self.key_gap;
        self.kb_top = self.b.height - kb_h - 6;
        self.build_keyboard();
        self.word_y = self.kb_top - 46;
        self.draw_top = 64;
        self.draw_bottom = self.word_y - 24;
    }

    fn build_keyboard(&mut self) {
        let layout = if self.lang == "de" { QWERTZ } else { QWERTY };
        self.key_rects.clear();
        let mut y = self.kb_top;
        for rowstr in layout {
            let keys: Vec<char> = rowstr.chars().collect();
            let n = keys.len() as i32;
            let kw = ((self.b.width - (n + 1) * self.key_gap) as f64 / n as f64) as i32;
            let kw = 20.max(kw.min(46));
            let row_w = n * kw + (n - 1) * self.key_gap;
            let mut x = py::fdiv32(self.b.width - row_w, 2);
            for ch in keys {
                self.key_rects.push((ch, Rect::new(x, y, kw, self.key_h)));
                x += kw + self.key_gap;
            }
            y += self.key_h + self.key_gap;
        }
    }

    fn guess(&mut self, ch: char) {
        if self.state != Zustand::Play || self.guessed.contains(&ch) {
            return;
        }
        self.guessed.push(ch);
        if self.word.contains(ch) {
            self.b.play_sound("click");
            if self.word.chars().all(|c| self.guessed.contains(&c)) {
                self.solve();
            }
        } else {
            self.wrong += 1;
            self.b.play_sound("move");
            if self.wrong >= MAX_WRONG {
                self.state = Zustand::Over;
                self.b.game_over = true;
                if self.solved_count == 0 {
                    self.b.report_result(false);
                }
                self.b.play_sound("gameover");
            }
        }
    }

    fn solve(&mut self) {
        let pts = 10 + (MAX_WRONG - self.wrong) * 8 + self.word.chars().count() as i64;
        self.last_points = pts;
        self.b.score += pts;
        self.solved_count += 1;
        self.state = Zustand::Solved;
        self.b.report_result(true);
        if self.wrong == 0 {
            self.b.ach_event("hangman_clean", None);
        }
        self.b.play_sound("win");
    }

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        let img = self.hud.render(&t("hang.title"), true, self.b.accent);
        s.blr(&img, img.get_rect().midleft_at((20, 30)));
        let img = self.small.render(&tf("hang.score", &[("n", &self.b.score.to_string())]), true, p.gold);
        s.blr(&img, img.get_rect().center_at((w / 2, 22)));
        let img = self.small.render(&tf("hang.misses", &[("a", &self.wrong.to_string()), ("b", &MAX_WRONG.to_string())]), true, p.text_dim);
        s.blr(&img, img.get_rect().midright_at((w - 20, 22)));
        let img = self.small.render(&tf("hang.solved_n", &[("n", &self.solved_count.to_string())]), true, p.text_dim);
        s.blr(&img, img.get_rect().midright_at((w - 20, 44)));
    }

    fn draw_gallows(&self, s: &mut Surface) {
        let size = 120.max((self.b.width - 80).min(self.draw_bottom - self.draw_top));
        let ox = (self.b.width - size) / 2;
        let oy = self.draw_top + 0.max((self.draw_bottom - self.draw_top - size) / 2);
        let u = size as f64 / 10.0;
        let wood = mix(self.b.accent, Color::rgb(170, 140, 110), 0.5);
        let lw = 3.max((u * 0.28) as i32);
        let pt = |gx: f64, gy: f64| ((ox as f64 + gx * u) as i32, (oy as f64 + gy * u) as i32);
        draw::line(s, wood, pt(1.0, 9.4), pt(6.0, 9.4), lw);
        draw::line(s, wood, pt(2.5, 9.4), pt(2.5, 0.8), lw);
        draw::line(s, wood, pt(2.5, 0.8), pt(6.2, 0.8), lw);
        draw::line(s, wood, pt(6.2, 0.8), pt(6.2, 1.8), lw);
        let red = Color::rgb(232, 110, 100);
        let parts = self.wrong;
        let (cx, cy) = pt(6.2, 2.7);
        let head_r = (u * 0.9) as i32;
        let (cxf, cyf) = (cx as f64, cy as f64);
        if parts >= 1 {
            draw::circle(s, red, (cx, cy), head_r, 2.max(lw - 1));
        }
        if parts >= 2 {
            draw::line(s, red, (cx, cy + head_r), (cx, (cyf + head_r as f64 + u * 2.6) as i32), lw);
        }
        let body_bottom = (cyf + head_r as f64 + u * 2.6) as i32;
        let shoulder = (cyf + head_r as f64 + u * 0.5) as i32;
        if parts >= 3 {
            draw::line(s, red, (cx, shoulder), ((cxf - u * 1.3) as i32, (shoulder as f64 + u * 1.2) as i32), lw);
        }
        if parts >= 4 {
            draw::line(s, red, (cx, shoulder), ((cxf + u * 1.3) as i32, (shoulder as f64 + u * 1.2) as i32), lw);
        }
        if parts >= 5 {
            draw::line(s, red, (cx, body_bottom), ((cxf - u * 1.2) as i32, (body_bottom as f64 + u * 1.6) as i32), lw);
        }
        if parts >= 6 {
            draw::line(s, red, (cx, body_bottom), ((cxf + u * 1.2) as i32, (body_bottom as f64 + u * 1.6) as i32), lw);
        }
    }

    fn draw_word(&self, s: &mut Surface) {
        let letters: Vec<char> = self.word.chars().collect();
        let n = letters.len() as i32;
        let slot = ((self.word_font.get_height() as f64 * 0.9) as i32).min((self.b.width - 40) / n);
        let gap = 6.max(slot / 5);
        let total = n * slot + (n - 1) * gap;
        let mut x = py::fdiv32(self.b.width - total, 2);
        let reveal_all = self.state == Zustand::Over;
        for ch in letters {
            let shown = self.guessed.contains(&ch) || reveal_all;
            draw::line(s, ui::text_dim(), (x, self.word_y + slot), (x + slot, self.word_y + slot), 3);
            if shown {
                let col = if self.guessed.contains(&ch) { COL_WORD } else { Color::rgb(228, 130, 120) };
                let img = self.word_font.render(&ch.to_string(), true, col);
                s.blr(&img, img.get_rect().center_at((x + slot / 2, self.word_y + slot / 2)));
            }
            x += slot + gap;
        }
    }

    fn draw_keyboard(&self, s: &mut Surface) {
        for (ch, rc) in &self.key_rects {
            let geraten = self.guessed.contains(ch);
            let im_wort = self.word.contains(*ch);
            let col = if geraten {
                if im_wort {
                    COL_CORRECT
                } else {
                    COL_ABSENT
                }
            } else {
                COL_KEY
            };
            draw::rect_r(s, col, *rc, 0, 6);
            let txt = if geraten && !im_wort { ui::text_faint() } else { COL_KEY_TEXT };
            let img = self.key_font.render(&ch.to_string(), true, txt);
            s.blr(&img, img.get_rect().center_at(rc.center()));
        }
    }

    fn draw_banner(&self, s: &mut Surface, title: &str, color: Color, sub: &str) {
        let w = (self.b.width - 40).min(500);
        let h = 100;
        let mut rc = Rect::new((self.b.width - w) / 2, self.word_y - 130, w, h);
        rc.y = 60.max(rc.y);
        let mut panel = pg::surface_alpha(w, h);
        panel.fill_c(Color::rgba(16, 18, 24, 238));
        s.bl(&panel, (rc.x, rc.y));
        draw::rect_r(s, color, rc, 2, 14);
        let img = self.huge.render(title, true, color);
        s.blr(&img, img.get_rect().center_at((rc.centerx(), rc.y + 36)));
        let img = self.small.render(sub, true, ui::text_dim());
        s.blr(&img, img.get_rect().center_at((rc.centerx(), rc.y + 74)));
    }
}

impl Game for HangmanGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        self.lang = pgz_core::i18n::get_language();
        let mode = if matches!(self.b.mode.as_str(), "short" | "mixed" | "long") { self.b.mode.clone() } else { "mixed".into() };
        self.words = words_for(&self.lang, &mode);
        self.b.score = 0;
        self.b.game_over = false;
        self.solved_count = 0;
        self.make_fonts();
        self.new_word();
        self.layout();
    }

    fn on_surface_changed(&mut self) {
        self.make_fonts();
        self.layout();
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if matches!(self.state, Zustand::Solved | Zustand::Over) {
            if ev.kind == Kind::MouseDown || ev.is_any(&["Return", "space"]) {
                if self.state == Zustand::Over {
                    self.b.game_over = false;
                    self.reset();
                } else {
                    self.new_word();
                    self.b.play_sound("click");
                }
            }
            return;
        }
        if ev.kind == Kind::KeyDown {
            let mut it = ev.key.chars();
            if let (Some(c), None) = (it.next(), it.next()) {
                if c.is_ascii_alphabetic() {
                    self.guess(c.to_ascii_uppercase());
                }
            }
        } else if ev.kind == Kind::MouseDown && ev.button == 1 {
            if let Some((ch, _)) = self.key_rects.iter().find(|(_, rc)| rc.hit(ev.pos)).cloned() {
                self.guess(ch);
            }
        }
    }

    fn update(&mut self, _dt: f64) {}

    fn draw(&mut self) {
        let mut s = self.b.sf();
        ui::draw_background_ex(&mut s, self.b.width, self.b.height, false, Some(true));
        self.draw_hud(&mut s);
        self.draw_gallows(&mut s);
        self.draw_word(&mut s);
        self.draw_keyboard(&mut s);
        if self.state == Zustand::Solved {
            self.draw_banner(&mut s, &tf("hang.solved", &[("n", &self.last_points.to_string())]), self.b.accent, &t("hang.next"));
        } else if self.state == Zustand::Over {
            let sub = format!("{}   \u{b7}   {}", tf("hang.word_was", &[("w", &self.word)]), t("common.enter_restart"));
            self.draw_banner(&mut s, &t("hang.gameover"), Color::rgb(228, 96, 96), &sub);
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "hangman",
    class_name: "HangmanGame",
    supports_multiplayer: false,
    modes: &[("short", "hang.mode.short"), ("mixed", "hang.mode.mixed"), ("long", "hang.mode.long")],
    make: HangmanGame::new,
};
