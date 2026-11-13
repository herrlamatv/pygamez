//! pong.rs
//! =======
//! Pong - Einzelspieler gegen die KI oder zwei Spieler.
//!
//! Uebertragen aus `games/pong.py`, Zeile fuer Zeile: Ballphysik mit
//! Beschleunigung und Treffwinkel, KI, Dauer-/Halten-Steuerung je
//! Belegung (X / N), bis 5 Punkte, Endstand-Karte.

use pgz_engine::prelude::*;

const COL_BALL: Color = Color::rgb(255, 220, 90);
const COL_P1: Color = Color::rgb(140, 230, 160);
const COL_P2: Color = Color::rgb(150, 200, 255);

const PADDLE_W: f64 = 12.0;
const PADDLE_H: f64 = 80.0;
const PADDLE_SPEED: f64 = 380.0;
const AI_SPEED: f64 = 300.0;
const BALL_SIZE: f64 = 14.0;
const BALL_START_SPEED: f64 = 320.0;
const BALL_MAX_SPEED: f64 = 650.0;
const WIN_SCORE: i64 = 5;
const HOLD_KEEPALIVE: f64 = 0.12;

/// Feste Umschalttasten: X = Steuerung 1, N = Steuerung 2.
fn toggle_scheme(key: &str) -> Option<usize> {
    match key {
        "x" | "X" => Some(0),
        "n" | "N" => Some(1),
        _ => None,
    }
}

const SCHEMES: [&str; 2] = ["p1", "p2"];

/// `PongGame`
pub struct PongGame {
    b: GameBase,
    player_y: f64,
    ai_y: f64,
    player_score: i64,
    ai_score: i64,
    dir: [f64; 2],
    keep: [f64; 2],
    hold: [bool; 2],
    ball_x: f64,
    ball_y: f64,
    ball_vx: f64,
    ball_vy: f64,
    rally: i32,
    font: Font,
    big_font: Font,
    small: Font,
}

impl PongGame {
    /// `PongGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let mut g = PongGame {
            font: b.font.clone(),
            big_font: b.big_font.clone(),
            small: b.font.clone(),
            b,
            player_y: 0.0,
            ai_y: 0.0,
            player_score: 0,
            ai_score: 0,
            dir: [0.0; 2],
            keep: [0.0; 2],
            hold: [false; 2],
            ball_x: 0.0,
            ball_y: 0.0,
            ball_vx: 0.0,
            ball_vy: 0.0,
            rally: 0,
        };
        g.reset();
        Box::new(g)
    }

    fn make_fonts(&mut self) {
        let h = self.b.height;
        self.font = ui::f(16.max(h / 26));
        self.big_font = ui::fb(36.max(h / 10));
        self.small = ui::f(13.max(h / 36));
    }

    fn serve(&mut self, toward_player: bool) {
        self.ball_x = self.b.width as f64 / 2.0 - BALL_SIZE / 2.0;
        self.ball_y = self.b.height as f64 / 2.0 - BALL_SIZE / 2.0;
        self.rally = 0;
        let winkel = py::uniform(-0.35, 0.35);
        let richtung = if toward_player { -1.0 } else { 1.0 };
        self.ball_vx = richtung * BALL_START_SPEED;
        self.ball_vy = BALL_START_SPEED * winkel;
    }

    fn move_scheme(&mut self, i: usize, key: &str) {
        let scheme = SCHEMES[i];
        if self.b.is_action(key, "up", Some(scheme)) {
            self.press(i, -1.0);
        } else if self.b.is_action(key, "down", Some(scheme)) {
            self.press(i, 1.0);
        } else if self.b.is_action(key, "action", Some(scheme)) {
            self.dir[i] = 0.0;
            self.keep[i] = 0.0;
        }
    }

    fn press(&mut self, i: usize, richtung: f64) {
        self.dir[i] = richtung;
        if self.hold[i] {
            self.keep[i] = HOLD_KEEPALIVE;
        }
    }

    fn toggle_hold(&mut self, i: usize) {
        self.hold[i] = !self.hold[i];
        self.dir[i] = 0.0;
        self.keep[i] = 0.0;
        let key = format!("hold_{}", SCHEMES[i]);
        self.b.set_opt("pong", &key, self.hold[i].into());
        self.b.play_sound("select");
    }

    fn bounce(&mut self, paddle: Rect, nach_rechts: bool) {
        let mut treff = (self.ball_y + BALL_SIZE / 2.0) - (paddle.y as f64 + PADDLE_H / 2.0);
        treff /= PADDLE_H / 2.0;
        let speed = BALL_MAX_SPEED.min(self.ball_vx.abs() * 1.06 + 20.0);
        self.ball_vx = speed * if nach_rechts { 1.0 } else { -1.0 };
        self.ball_vy = speed * treff;
        if nach_rechts {
            self.ball_x = paddle.right() as f64;
        } else {
            self.ball_x = paddle.left() as f64 - BALL_SIZE;
        }
        self.b.play_sound("bounce");
        self.b.rumble(60);
        self.rally += 1;
        if self.rally >= 20 {
            self.b.ach_event("pong_rally", None);
        }
    }

    fn nach_punkt(&mut self, toward_player: bool) {
        if self.player_score >= WIN_SCORE || self.ai_score >= WIN_SCORE {
            self.b.game_over = true;
            self.b.play_sound(if self.player_score > self.ai_score { "win" } else { "gameover" });
            if !self.b.multiplayer {
                self.b.report_result(self.player_score > self.ai_score);
                if self.player_score > self.ai_score && self.ai_score == 0 {
                    self.b.ach_event("pong_shutout", None);
                }
            } else {
                self.b.report_duel(if self.player_score > self.ai_score { 0 } else { 1 });
            }
            self.b.rumble(200);
        } else {
            self.serve(toward_player);
        }
    }

    fn draw_overlay(&self, titel: &str, farbe: Color, hinweis: &str) {
        let mut s = self.b.sf();
        let th = self.big_font.get_height();
        let hh = self.font.get_height();
        let bw = self.big_font.width(titel).max(self.font.width(hinweis)) + 80;
        let bh = th + hh + 58;
        let rect = Rect::new(0, 0, bw, bh).center_at((self.b.width / 2, self.b.height / 2));
        let mut panel = pg::surface_alpha(rect.w, rect.h);
        let pr = panel.get_rect();
        draw::rect_r(&mut panel, pg::alpha(ui::panel(), 235), pr, 0, 16);
        draw::rect_r(&mut panel, farbe, pr, 2, 16);
        s.blr(&panel, rect);
        let img = self.big_font.render(titel, true, farbe);
        s.blr(&img, img.get_rect().midtop_at((rect.centerx(), rect.y + 20)));
        let hint_col = mix(ui::text_dim(), ui::text(), ui::pulse(2.0, 0.2, 1.0));
        let img = self.font.render(hinweis, true, hint_col);
        s.blr(&img, img.get_rect().midtop_at((rect.centerx(), rect.y + 20 + th + 14)));
    }

    fn draw_mode_hud(&self) {
        let mut s = self.b.sf();
        let y = self.b.height - self.small.get_height() - 8;
        let zeile = |i: usize| -> (String, Color) {
            let modus = if self.hold[i] { t("pong.hold") } else { t("pong.continuous") };
            let col = if self.hold[i] { ui::green() } else { ui::text_dim() };
            (modus, col)
        };
        let (m1, c1) = zeile(0);
        let t1 = self.small.render(&tf("pong.scheme1", &[("mode", &m1)]), true, c1);
        s.bl(&t1, (10, y));
        let (m2, c2) = zeile(1);
        let t2 = self.small.render(&tf("pong.scheme2", &[("mode", &m2)]), true, c2);
        s.bl(&t2, (self.b.width - t2.w() - 10, y));
    }
}

impl Game for PongGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        self.b.score = 0;
        self.b.game_over = false;
        self.b.new_round_result();
        let h = self.b.height as f64;
        self.player_y = h / 2.0 - PADDLE_H / 2.0;
        self.ai_y = h / 2.0 - PADDLE_H / 2.0;
        self.player_score = 0;
        self.ai_score = 0;
        self.dir = [0.0; 2];
        self.keep = [0.0; 2];
        self.hold = [self.b.opt_bool("pong", "hold_p1", false), self.b.opt_bool("pong", "hold_p2", false)];
        self.make_fonts();
        let toward = py::choice(&[true, false]);
        self.serve(toward);
    }

    fn on_surface_changed(&mut self) {
        self.make_fonts();
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        self.player_y = self.player_y.min(h - PADDLE_H).max(0.0);
        self.ai_y = self.ai_y.min(h - PADDLE_H).max(0.0);
        self.ball_x = self.ball_x.min(w - BALL_SIZE).max(0.0);
        self.ball_y = self.ball_y.min(h - BALL_SIZE).max(0.0);
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if ev.kind != Kind::KeyDown {
            return;
        }
        if self.b.game_over {
            if ev.key_in(&["Return", "space"]) {
                self.reset();
            }
            return;
        }
        if let Some(i) = toggle_scheme(&ev.key) {
            self.toggle_hold(i);
            return;
        }
        let k = ev.key.clone();
        self.move_scheme(0, &k);
        self.move_scheme(1, &k);
    }

    fn update(&mut self, dt: f64) {
        if self.b.game_over {
            return;
        }
        for i in 0..2 {
            if self.hold[i] && self.dir[i] != 0.0 {
                self.keep[i] -= dt;
                if self.keep[i] <= 0.0 {
                    self.dir[i] = 0.0;
                }
            }
        }
        let h = self.b.height as f64;
        let w = self.b.width as f64;
        let left_dir = if self.b.multiplayer { self.dir[0] } else { (self.dir[0] + self.dir[1]).clamp(-1.0, 1.0) };
        self.player_y += left_dir * PADDLE_SPEED * dt;
        self.player_y = self.player_y.min(h - PADDLE_H).max(0.0);

        if self.b.multiplayer {
            self.ai_y += self.dir[1] * PADDLE_SPEED * dt;
        } else {
            let ziel = self.ball_y + BALL_SIZE / 2.0 - PADDLE_H / 2.0;
            if self.ai_y < ziel - 8.0 {
                self.ai_y += AI_SPEED * dt;
            } else if self.ai_y > ziel + 8.0 {
                self.ai_y -= AI_SPEED * dt;
            }
        }
        self.ai_y = self.ai_y.min(h - PADDLE_H).max(0.0);

        self.ball_x += self.ball_vx * dt;
        self.ball_y += self.ball_vy * dt;
        if self.ball_y <= 0.0 {
            self.ball_y = 0.0;
            self.ball_vy = self.ball_vy.abs();
            self.b.play_sound("bounce");
        } else if self.ball_y + BALL_SIZE >= h {
            self.ball_y = h - BALL_SIZE;
            self.ball_vy = -self.ball_vy.abs();
            self.b.play_sound("bounce");
        }

        let ball_rect = pg::rect(self.ball_x, self.ball_y, BALL_SIZE, BALL_SIZE);
        let player_rect = pg::rect(20, self.player_y, PADDLE_W, PADDLE_H);
        let ai_rect = pg::rect(w - 20.0 - PADDLE_W, self.ai_y, PADDLE_W, PADDLE_H);
        if ball_rect.colliderect(player_rect) && self.ball_vx < 0.0 {
            self.bounce(player_rect, true);
        } else if ball_rect.colliderect(ai_rect) && self.ball_vx > 0.0 {
            self.bounce(ai_rect, false);
        }

        if self.ball_x + BALL_SIZE < 0.0 {
            self.ai_score += 1;
            self.b.play_sound("point");
            self.nach_punkt(true);
        } else if self.ball_x > w {
            self.player_score += 1;
            self.b.score = self.player_score;
            self.b.play_sound("point");
            self.nach_punkt(false);
        }
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        let (w, h) = (self.b.width, self.b.height);
        ui::draw_background(&mut s, w, h);
        let mut y = 0;
        while y < h {
            draw::rect(&mut s, ui::border(), (w / 2 - 2, y, 4, 16), 0);
            y += 28;
        }
        draw::rect_r(&mut s, COL_P1, (20.0, self.player_y, PADDLE_W, PADDLE_H), 0, 4);
        let rechts_farbe = if self.b.multiplayer { COL_P2 } else { ui::text() };
        draw::rect_r(&mut s, rechts_farbe, (w as f64 - 20.0 - PADDLE_W, self.ai_y, PADDLE_W, PADDLE_H), 0, 4);
        draw::rect_r(&mut s, COL_BALL, (self.ball_x, self.ball_y, BALL_SIZE, BALL_SIZE), 0, 3);

        let ps = self.big_font.render(&self.player_score.to_string(), true, ui::text());
        let ais = self.big_font.render(&self.ai_score.to_string(), true, ui::text());
        s.blr(&ps, ps.get_rect().topright_at((w / 2 - 44, 16)));
        s.bl(&ais, (w / 2 + 44, 16));

        let links = if self.b.multiplayer { self.b.pname(0).chars().take(6).collect() } else { t("pong.you") };
        let rechts = if self.b.multiplayer { self.b.pname(1).chars().take(6).collect() } else { t("pong.ai") };
        s.bl(&self.font.render(&links, true, COL_P1), (34, 12));
        let r_img = self.font.render(&rechts, true, rechts_farbe);
        s.bl(&r_img, (w - r_img.w() - 34, 12));

        if !self.b.game_over {
            self.draw_mode_hud();
        }
        if self.b.game_over {
            let (text, farbe) = if self.b.multiplayer {
                let sieger = if self.player_score > self.ai_score { 1 } else { 2 };
                (
                    tf("common.player_wins_name", &[("name", &self.b.pname(sieger - 1))]),
                    if sieger == 1 { COL_P1 } else { COL_P2 },
                )
            } else {
                let gewonnen = self.player_score > self.ai_score;
                (if gewonnen { t("pong.won") } else { t("pong.lost") }, if gewonnen { ui::green() } else { ui::red() })
            };
            self.draw_overlay(&text, farbe, &t("common.enter_restart"));
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "pong",
    class_name: "PongGame",
    supports_multiplayer: true,
    modes: &[],
    make: PongGame::new,
};
