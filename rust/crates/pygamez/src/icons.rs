//! icons.rs
//! ========
//! Die 50 Mini-Piktogramme der Spieleliste - `main._draw_icon`.
//!
//! In Python zeichnet sie ein Tk-Canvas (`create_oval`, `create_line`,
//! `create_polygon` ...). Hier zeichnet rustgame dieselben Formen mit
//! denselben Koordinaten. Tk und pygame rastern Kanten verschieden - das
//! Bild ist darum dasselbe Motiv, nicht pixelgleich (die Seitenleiste ist
//! in Python kein pygame-Bild, es gibt dort nichts Pixelgleiches zu treffen).

use pgz_engine::pg::{self, draw, Color, RectExt, SurfExt, Surface};

/// `create_rectangle(x1, y1, x2, y2, fill=..)`
fn rect(s: &mut Surface, c: Color, x1: i32, y1: i32, x2: i32, y2: i32) {
    draw::rect(s, c, (x1, y1, x2 - x1, y2 - y1), 0);
}

/// `create_rectangle(..., outline=c)` (ohne Fuellung).
fn rect_o(s: &mut Surface, c: Color, x1: i32, y1: i32, x2: i32, y2: i32, w: i32) {
    draw::rect(s, c, (x1, y1, x2 - x1 + 1, y2 - y1 + 1), w);
}

/// `create_oval(x1, y1, x2, y2, fill=..)`
fn oval(s: &mut Surface, c: Color, x1: i32, y1: i32, x2: i32, y2: i32) {
    draw::ellipse(s, c, (x1, y1, x2 - x1, y2 - y1), 0);
}

/// `create_oval(..., outline=c, width=w)`
fn oval_o(s: &mut Surface, c: Color, x1: i32, y1: i32, x2: i32, y2: i32, w: i32) {
    draw::ellipse(s, c, (x1, y1, x2 - x1 + 1, y2 - y1 + 1), w);
}

/// `create_line(pts, width=w)` - mit runden Enden bei `round`.
fn line(s: &mut Surface, c: Color, pts: &[(i32, i32)], w: i32, rund: bool) {
    draw::lines(s, c, false, pts, w);
    if rund && w >= 3 {
        for p in pts {
            draw::circle(s, c, *p, w / 2, 0);
        }
    }
}

/// `create_polygon(pts, fill=c)`
fn poly(s: &mut Surface, c: Color, pts: &[(i32, i32)]) {
    draw::polygon(s, c, pts, 0);
}

/// `create_polygon(pts, fill="", outline=c, width=w)`
fn poly_o(s: &mut Surface, c: Color, pts: &[(i32, i32)], w: i32) {
    draw::polygon(s, c, pts, w);
}

/// `_round_rect(cv, x1, y1, x2, y2, r, fill="", outline=c)`
fn rrect_o(s: &mut Surface, c: Color, x1: i32, y1: i32, x2: i32, y2: i32, r: i32) {
    draw::rect_r(s, c, (x1, y1, x2 - x1 + 1, y2 - y1 + 1), 1, r);
}

/// `_round_rect(..., fill=c)`
fn rrect(s: &mut Surface, c: Color, x1: i32, y1: i32, x2: i32, y2: i32, r: i32) {
    draw::rect_r(s, c, (x1, y1, x2 - x1, y2 - y1), 0, r);
}

/// `create_arc(bbox, start, extent, style=pieslice, fill=c)` - Tk zaehlt
/// Grad gegen den Uhrzeigersinn ab 3 Uhr.
fn pie(s: &mut Surface, c: Color, x1: i32, y1: i32, x2: i32, y2: i32, start: f64, extent: f64) {
    let (cx, cy) = ((x1 + x2) as f64 / 2.0, (y1 + y2) as f64 / 2.0);
    let (rx, ry) = ((x2 - x1) as f64 / 2.0, (y2 - y1) as f64 / 2.0);
    let mut pts = vec![(cx, cy)];
    let n = ((extent.abs() / 6.0) as i32).max(4);
    for i in 0..=n {
        let a = (start + extent * i as f64 / n as f64).to_radians();
        pts.push((cx + rx * a.cos(), cy - ry * a.sin()));
    }
    draw::polygon(s, c, &pts, 0);
}

/// `create_text(x, y, text=..)` - zentriert.
fn text(s: &mut Surface, c: Color, x: i32, y: i32, t: &str, groesse: i32) {
    let f = pg::Font::sys("segoeui", groesse, true, false);
    let img = f.render(t, true, c);
    let r = img.get_rect().center_at((x, y));
    s.blr(&img, r);
}

/// Zeichnet das Piktogramm eines Spiels (Python-Klassenname) bei (cx, cy).
pub fn draw_icon(s: &mut Surface, cx: i32, cy: i32, name: &str, color: Color, bg: Color) {
    let c = color;
    match name {
        "SnakeGame" => {
            line(s, c, &[(cx - 8, cy + 6), (cx - 8, cy - 1), (cx + 1, cy - 1), (cx + 1, cy + 6), (cx + 7, cy + 6)], 3, true);
            oval(s, c, cx + 5, cy - 8, cx + 9, cy - 4);
        }
        "PongGame" => {
            rect(s, c, cx - 9, cy - 7, cx - 6, cy + 3);
            rect(s, c, cx + 6, cy - 3, cx + 9, cy + 7);
            oval(s, c, cx - 2, cy - 2, cx + 2, cy + 2);
        }
        "AirHockeyGame" => {
            oval_o(s, c, cx - 9, cy - 9, cx + 1, cy + 1, 2);
            oval(s, c, cx - 6, cy - 6, cx - 2, cy - 2);
            oval(s, c, cx + 3, cy + 3, cx + 9, cy + 9);
        }
        "TicTacToeGame" => {
            line(s, c, &[(cx - 3, cy - 9), (cx - 3, cy + 9)], 2, false);
            line(s, c, &[(cx + 3, cy - 9), (cx + 3, cy + 9)], 2, false);
            line(s, c, &[(cx - 9, cy - 3), (cx + 9, cy - 3)], 2, false);
            line(s, c, &[(cx - 9, cy + 3), (cx + 9, cy + 3)], 2, false);
        }
        "BreakoutGame" => {
            for i in 0..3 {
                rect(s, c, cx - 10 + i * 7, cy - 9, cx - 5 + i * 7, cy - 6);
            }
            oval(s, c, cx - 2, cy - 2, cx + 2, cy + 2);
            rect(s, c, cx - 6, cy + 6, cx + 6, cy + 9);
        }
        "TetrisGame" => {
            let u = 6;
            for (gx, gy) in [(-1, 0), (0, 0), (1, 0), (0, 1)] {
                let x0 = cx + gx * u - 2;
                let y0 = cy - 5 + gy * u;
                rect(s, c, x0 - 2, y0, x0 + 3, y0 + 5);
            }
        }
        "InvadersGame" => {
            rect(s, c, cx - 7, cy - 4, cx + 7, cy + 3);
            line(s, c, &[(cx - 7, cy - 4), (cx - 10, cy - 8)], 2, false);
            line(s, c, &[(cx + 7, cy - 4), (cx + 10, cy - 8)], 2, false);
            rect(s, bg, cx - 4, cy - 2, cx - 2, cy);
            rect(s, bg, cx + 2, cy - 2, cx + 4, cy);
            line(s, c, &[(cx - 5, cy + 3), (cx - 5, cy + 7)], 2, false);
            line(s, c, &[(cx + 5, cy + 3), (cx + 5, cy + 7)], 2, false);
        }
        "AsteroidsGame" => {
            poly_o(s, c, &[(cx - 2, cy - 8), (cx - 8, cy + 7), (cx - 2, cy + 3), (cx + 4, cy + 7)], 2);
            oval_o(s, c, cx + 4, cy - 8, cx + 10, cy - 2, 2);
        }
        "PacmanGame" => {
            pie(s, c, cx - 8, cy - 8, cx + 8, cy + 8, 35.0, 290.0);
            oval(s, c, cx + 6, cy - 2, cx + 9, cy + 1);
        }
        "FlappyGame" => {
            oval(s, c, cx - 8, cy - 6, cx + 4, cy + 6);
            poly(s, c, &[(cx + 3, cy - 2), (cx + 9, cy), (cx + 3, cy + 2)]);
            oval(s, bg, cx - 6, cy - 2, cx - 1, cy + 3);
        }
        "DoodleGame" => {
            line(s, c, &[(cx - 9, cy + 8), (cx - 1, cy + 8)], 3, true);
            line(s, c, &[(cx + 1, cy), (cx + 9, cy)], 3, true);
            line(s, c, &[(cx - 9, cy - 8), (cx - 1, cy - 8)], 3, true);
            oval(s, c, cx + 3, cy - 7, cx + 8, cy - 2);
        }
        "Game2048" => {
            rrect_o(s, c, cx - 8, cy - 8, cx + 8, cy + 8, 4);
            text(s, c, cx, cy, "2", 12);
        }
        "MinesweeperGame" => {
            line(s, c, &[(cx + 3, cy - 8), (cx + 3, cy + 7)], 2, false);
            poly(s, c, &[(cx + 3, cy - 8), (cx - 6, cy - 4), (cx + 3, cy)]);
            line(s, c, &[(cx - 2, cy + 7), (cx + 8, cy + 7)], 2, false);
        }
        "SudokuGame" => {
            rrect_o(s, c, cx - 9, cy - 9, cx + 9, cy + 9, 3);
            line(s, c, &[(cx - 3, cy - 9), (cx - 3, cy + 9)], 1, false);
            line(s, c, &[(cx + 3, cy - 9), (cx + 3, cy + 9)], 1, false);
            line(s, c, &[(cx - 9, cy - 3), (cx + 9, cy - 3)], 1, false);
            line(s, c, &[(cx - 9, cy + 3), (cx + 9, cy + 3)], 1, false);
            text(s, c, cx - 6, cy - 6, "5", 8);
            text(s, c, cx + 6, cy + 6, "3", 8);
        }
        "FroggerGame" => {
            oval(s, c, cx - 7, cy - 5, cx + 7, cy + 8);
            for sx in [-4, 4] {
                oval(s, c, cx + sx - 3, cy - 9, cx + sx + 3, cy - 3);
                oval(s, bg, cx + sx - 1, cy - 7, cx + sx + 1, cy - 5);
            }
            line(s, c, &[(cx - 7, cy + 3), (cx - 10, cy + 9)], 2, false);
            line(s, c, &[(cx + 7, cy + 3), (cx + 10, cy + 9)], 2, false);
        }
        "MemoryGame" => {
            rrect_o(s, c, cx - 10, cy - 8, cx - 1, cy + 8, 3);
            text(s, c, cx - 5, cy, "?", 11);
            rrect_o(s, c, cx + 1, cy - 8, cx + 10, cy + 8, 3);
            poly(
                s,
                c,
                &[
                    (cx + 5, cy - 4),
                    (cx + 7, cy - 1),
                    (cx + 9, cy - 1),
                    (cx + 7, cy + 2),
                    (cx + 8, cy + 5),
                    (cx + 5, cy + 3),
                    (cx + 2, cy + 5),
                    (cx + 3, cy + 2),
                    (cx + 1, cy - 1),
                    (cx + 4, cy - 1),
                ],
            );
        }
        "SolitaireGame" => {
            rrect_o(s, c, cx - 9, cy - 8, cx + 2, cy + 8, 3);
            rrect_o(s, c, cx - 2, cy - 6, cx + 9, cy + 10, 3);
            poly(s, c, &[(cx + 3, cy - 2), (cx + 6, cy + 2), (cx + 3, cy + 6), (cx, cy + 2)]);
        }
        "AimTrainerGame" => {
            oval_o(s, c, cx - 8, cy - 8, cx + 8, cy + 8, 2);
            line(s, c, &[(cx, cy - 11), (cx, cy - 5)], 2, false);
            line(s, c, &[(cx, cy + 5), (cx, cy + 11)], 2, false);
            line(s, c, &[(cx - 11, cy), (cx - 5, cy)], 2, false);
            line(s, c, &[(cx + 5, cy), (cx + 11, cy)], 2, false);
            oval(s, c, cx - 2, cy - 2, cx + 2, cy + 2);
        }
        "ConnectFourGame" => {
            rrect_o(s, c, cx - 10, cy - 8, cx + 10, cy + 9, 3);
            for (gy, row) in [(3, [false, true, true]), (-3, [true, false, true])] {
                for (i, filled) in row.iter().enumerate() {
                    let x = cx - 6 + i as i32 * 6;
                    if *filled {
                        oval(s, c, x - 2, cy + gy - 2, x + 2, cy + gy + 2);
                    } else {
                        oval_o(s, c, x - 2, cy + gy - 2, x + 2, cy + gy + 2, 1);
                    }
                }
            }
        }
        "TankDuelGame" => {
            rrect(s, c, cx - 9, cy, cx + 5, cy + 7, 2);
            oval(s, c, cx - 5, cy - 5, cx + 1, cy + 1);
            line(s, c, &[(cx - 1, cy - 3), (cx + 8, cy - 6)], 2, false);
            oval(s, c, cx + 9, cy - 8, cx + 12, cy - 5);
        }
        "BlackjackGame" => {
            rrect_o(s, c, cx - 10, cy - 7, cx + 1, cy + 9, 3);
            rrect_o(s, c, cx - 1, cy - 9, cx + 10, cy + 7, 3);
            text(s, c, cx + 4, cy - 4, "A", 9);
            poly(s, c, &[(cx - 5, cy - 3), (cx - 2, cy + 2), (cx - 8, cy + 2)]);
            line(s, c, &[(cx - 5, cy + 2), (cx - 5, cy + 5)], 2, false);
        }
        "TunnelRacerGame" => {
            rect_o(s, c, cx - 10, cy - 10, cx + 10, cy + 10, 1);
            rect_o(s, c, cx - 6, cy - 6, cx + 6, cy + 6, 1);
            rect_o(s, c, cx - 3, cy - 3, cx + 3, cy + 3, 1);
            poly(s, c, &[(cx, cy + 1), (cx - 3, cy + 8), (cx, cy + 6), (cx + 3, cy + 8)]);
        }
        "LabyrinthGame" => {
            rrect_o(s, c, cx - 9, cy - 9, cx + 9, cy + 9, 3);
            line(s, c, &[(cx - 5, cy - 9), (cx - 5, cy + 3)], 2, false);
            line(s, c, &[(cx - 5, cy + 3), (cx + 3, cy + 3)], 2, false);
            line(s, c, &[(cx + 3, cy - 4), (cx + 3, cy + 3)], 2, false);
            oval(s, c, cx + 4, cy + 4, cx + 8, cy + 8);
        }
        "TRexRunnerGame" => {
            rect(s, c, cx - 5, cy - 6, cx + 2, cy + 4);
            rect(s, c, cx + 1, cy - 9, cx + 8, cy - 3);
            poly(s, c, &[(cx - 5, cy - 3), (cx - 10, cy - 6), (cx - 5, cy + 1)]);
            oval(s, bg, cx + 5, cy - 8, cx + 7, cy - 6);
            line(s, c, &[(cx - 3, cy + 4), (cx - 3, cy + 9)], 2, false);
            line(s, c, &[(cx + 1, cy + 4), (cx + 1, cy + 9)], 2, false);
        }
        "DameGame" => {
            oval(s, c, cx - 9, cy + 1, cx + 1, cy + 9);
            oval(s, c, cx - 1, cy - 8, cx + 9, cy + 2);
            poly(
                s,
                bg,
                &[(cx + 1, cy - 3), (cx + 1, cy - 7), (cx + 3, cy - 5), (cx + 4, cy - 8), (cx + 5, cy - 5), (cx + 7, cy - 7), (cx + 7, cy - 3)],
            );
        }
        "PokerGame" => {
            rrect_o(s, c, cx - 9, cy - 9, cx + 2, cy + 7, 3);
            text(s, c, cx - 4, cy - 4, "A", 9);
            poly(s, c, &[(cx - 4, cy + 1), (cx - 1, cy + 5), (cx - 7, cy + 5)]);
            oval_o(s, c, cx, cy - 2, cx + 10, cy + 8, 2);
            oval_o(s, c, cx + 3, cy + 1, cx + 7, cy + 5, 1);
        }
        "ChessGame" => {
            rect(s, c, cx - 6, cy + 4, cx + 6, cy + 8);
            poly(s, c, &[(cx - 5, cy + 4), (cx - 3, cy - 3), (cx + 3, cy - 3), (cx + 5, cy + 4)]);
            rect(s, c, cx - 1, cy - 9, cx + 1, cy - 2);
            rect(s, c, cx - 3, cy - 7, cx + 3, cy - 5);
        }
        "MuehleGame" => {
            rect_o(s, c, cx - 9, cy - 9, cx + 9, cy + 9, 1);
            rect_o(s, c, cx - 4, cy - 4, cx + 4, cy + 4, 1);
            line(s, c, &[(cx, cy - 9), (cx, cy - 4)], 1, false);
            line(s, c, &[(cx, cy + 9), (cx, cy + 4)], 1, false);
            line(s, c, &[(cx - 9, cy), (cx - 4, cy)], 1, false);
            line(s, c, &[(cx + 9, cy), (cx + 4, cy)], 1, false);
            oval(s, c, cx - 10, cy - 10, cx - 6, cy - 6);
        }
        "SimonGame" => {
            pie(s, c, cx - 9, cy - 9, cx + 9, cy + 9, 90.0, 90.0);
            oval_o(s, c, cx - 9, cy - 9, cx + 9, cy + 9, 1);
            line(s, c, &[(cx - 9, cy), (cx + 9, cy)], 1, false);
            line(s, c, &[(cx, cy - 9), (cx, cy + 9)], 1, false);
        }
        "BilliardGame" => {
            line(s, c, &[(cx - 10, cy + 10), (cx + 1, cy - 1)], 2, false);
            oval(s, c, cx - 1, cy - 9, cx + 9, cy + 1);
            oval(s, bg, cx + 2, cy - 6, cx + 5, cy - 3);
        }
        "SlidingPuzzleGame" => {
            for (gx, gy) in [(-1, -1), (1, -1), (-1, 1)] {
                let (ccx, ccy) = (cx + gx * 5, cy + gy * 5);
                rrect_o(s, c, ccx - 4, ccy - 4, ccx + 4, ccy + 4, 2);
            }
            line(s, c, &[(cx + 1, cy + 5), (cx + 6, cy + 5)], 2, false);
            poly(s, c, &[(cx + 9, cy + 5), (cx + 5, cy + 2), (cx + 5, cy + 8)]);
        }
        "MastermindGame" => {
            for (i, gx) in [-8, -3, 2, 7].iter().enumerate() {
                if i % 2 == 0 {
                    oval(s, c, cx + gx - 2, cy - 7, cx + gx + 2, cy - 3);
                } else {
                    oval_o(s, c, cx + gx - 2, cy - 7, cx + gx + 2, cy - 3, 1);
                }
            }
            for gx in [-8, -3, 2, 7] {
                oval(s, c, cx + gx - 1, cy + 4, cx + gx + 1, cy + 6);
            }
        }
        "BubbleShooterGame" => {
            for (gx, gy) in [(-5, -7), (1, -7), (-2, -2), (4, -2)] {
                oval(s, c, cx + gx - 3, cy + gy - 3, cx + gx + 3, cy + gy + 3);
            }
            line(s, c, &[(cx, cy + 9), (cx, cy + 3)], 2, false);
            oval(s, c, cx - 2, cy + 1, cx + 2, cy + 5);
        }
        "HangmanGame" => {
            line(s, c, &[(cx - 8, cy + 9), (cx - 1, cy + 9)], 2, false);
            line(s, c, &[(cx - 5, cy + 9), (cx - 5, cy - 8)], 2, false);
            line(s, c, &[(cx - 5, cy - 8), (cx + 5, cy - 8)], 2, false);
            line(s, c, &[(cx + 5, cy - 8), (cx + 5, cy - 5)], 1, false);
            oval_o(s, c, cx + 2, cy - 5, cx + 8, cy + 1, 2);
        }
        "BlockJumpGame" => {
            poly(s, c, &[(cx, cy - 8), (cx + 9, cy - 3), (cx, cy + 2), (cx - 9, cy - 3)]);
            poly_o(s, bg, &[(cx, cy - 8), (cx + 9, cy - 3), (cx, cy + 2), (cx - 9, cy - 3)], 1);
            poly(s, c, &[(cx - 9, cy - 3), (cx, cy + 2), (cx, cy + 11), (cx - 9, cy + 6)]);
            poly_o(s, bg, &[(cx - 9, cy - 3), (cx, cy + 2), (cx, cy + 11), (cx - 9, cy + 6)], 1);
            poly(s, c, &[(cx + 9, cy - 3), (cx, cy + 2), (cx, cy + 11), (cx + 9, cy + 6)]);
            poly_o(s, bg, &[(cx + 9, cy - 3), (cx, cy + 2), (cx, cy + 11), (cx + 9, cy + 6)], 1);
            rect(s, c, cx - 2, cy - 13, cx + 2, cy - 9);
        }
        "LamaTowerDefenseGame" => {
            rect(s, c, cx - 6, cy - 4, cx + 6, cy + 9);
            for dx in [-6, -1, 4] {
                rect(s, c, cx + dx, cy - 8, cx + dx + 2, cy - 4);
            }
            pie(s, bg, cx - 3, cy + 3, cx + 3, cy + 12, 0.0, 180.0);
            line(s, c, &[(cx + 6, cy - 8), (cx + 6, cy - 13)], 1, false);
            poly(s, c, &[(cx + 6, cy - 13), (cx + 11, cy - 11), (cx + 6, cy - 9)]);
        }
        "MiniGolfGame" => {
            oval(s, c, cx - 4, cy + 4, cx + 6, cy + 9);
            line(s, c, &[(cx + 1, cy + 6), (cx + 1, cy - 9)], 2, false);
            poly(s, c, &[(cx + 1, cy - 9), (cx + 9, cy - 6), (cx + 1, cy - 3)]);
            oval(s, c, cx - 10, cy + 3, cx - 5, cy + 8);
        }
        "PinballGame" => {
            oval_o(s, c, cx - 4, cy - 10, cx + 4, cy - 2, 2);
            oval(s, c, cx - 2, cy, cx + 2, cy + 4);
            line(s, c, &[(cx - 9, cy + 4), (cx - 2, cy + 9)], 3, true);
            line(s, c, &[(cx + 9, cy + 4), (cx + 2, cy + 9)], 3, true);
        }
        "BowlingGame" => {
            poly(
                s,
                c,
                &[(cx - 1, cy - 9), (cx + 2, cy - 9), (cx + 3, cy - 4), (cx + 2, cy - 1), (cx + 4, cy + 6), (cx - 3, cy + 6), (cx - 1, cy - 1), (cx - 2, cy - 4)],
            );
            oval_o(s, c, cx - 10, cy - 1, cx - 2, cy + 7, 2);
            oval(s, c, cx - 8, cy + 1, cx - 7, cy + 2);
            oval(s, c, cx - 6, cy + 1, cx - 5, cy + 2);
            oval(s, c, cx - 7, cy + 3, cx - 6, cy + 4);
        }
        "CrossyRoadGame" => {
            rect(s, c, cx - 6, cy - 6, cx + 5, cy + 5);
            rect(s, Color::rgb(0xe0, 0x50, 0x3c), cx - 2, cy - 9, cx + 2, cy - 6);
            rect(s, Color::rgb(0xf0, 0xa0, 0x30), cx + 5, cy - 3, cx + 9, cy);
            rect(s, bg, cx + 1, cy - 4, cx + 3, cy - 2);
            line(s, c, &[(cx - 10, cy + 9), (cx - 4, cy + 9)], 2, false);
            line(s, c, &[(cx + 1, cy + 9), (cx + 7, cy + 9)], 2, false);
        }
        "GeometryDashGame" => {
            rect_o(s, c, cx - 9, cy - 8, cx + 3, cy + 4, 2);
            rect(s, c, cx - 6, cy - 5, cx - 4, cy - 3);
            rect(s, c, cx - 2, cy - 5, cx, cy - 3);
            line(s, c, &[(cx - 6, cy), (cx, cy)], 2, false);
            poly(s, c, &[(cx + 4, cy + 9), (cx + 7, cy + 2), (cx + 10, cy + 9)]);
            line(s, c, &[(cx - 10, cy + 9), (cx + 10, cy + 9)], 1, false);
        }
        "BattleshipGame" => {
            poly(s, c, &[(cx - 10, cy + 1), (cx + 10, cy + 1), (cx + 7, cy + 6), (cx - 7, cy + 6)]);
            rect(s, c, cx - 4, cy - 4, cx + 3, cy + 1);
            line(s, c, &[(cx, cy - 10), (cx, cy - 4)], 2, false);
            let welle: Vec<(f64, f64)> = (0..=20)
                .map(|i| {
                    let x = -10.0 + i as f64;
                    (cx as f64 + x, cy as f64 + 8.5 + 0.5 * (x * py_pi() / 5.0).cos())
                })
                .collect();
            draw::lines(s, c, false, &welle, 1);
        }
        "CasinoGame" => {
            oval_o(s, c, cx - 9, cy - 9, cx + 9, cy + 9, 2);
            oval_o(s, c, cx - 4, cy - 4, cx + 4, cy + 4, 1);
            for (dx, dy) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
                line(s, c, &[(cx + dx * 4, cy + dy * 4), (cx + dx * 8, cy + dy * 8)], 1, false);
            }
            oval(s, c, cx + 3, cy - 8, cx + 7, cy - 4);
        }
        "LamaCrossingGame" => {
            // Haengebruecke (Seil + Planken) mit Lama-Kopf darueber
            line(s, c, &[(cx - 10, cy + 2), (cx, cy + 5), (cx + 10, cy + 2)], 1, false);
            for dx in [-7, -2, 3, 8] {
                rect(s, c, cx + dx - 2, cy + 5, cx + dx + 1, cy + 8);
            }
            rect(s, c, cx - 3, cy - 7, cx, cy + 1);
            oval(s, c, cx - 4, cy - 10, cx + 4, cy - 5);
            poly(s, c, &[(cx - 3, cy - 9), (cx - 2, cy - 12), (cx, cy - 9)]);
        }
        "LamaMinesGame" => {
            // geschliffener Edelstein (Krone + Spitze) mit Facettenlinien
            poly(s, c, &[(cx - 9, cy - 3), (cx - 5, cy - 8), (cx + 5, cy - 8), (cx + 9, cy - 3), (cx, cy + 9)]);
            line(s, bg, &[(cx - 9, cy - 3), (cx + 9, cy - 3)], 1, false);
            line(s, bg, &[(cx - 3, cy - 3), (cx, cy + 8), (cx + 3, cy - 3)], 1, false);
            line(s, bg, &[(cx - 5, cy - 8), (cx - 3, cy - 3)], 1, false);
            line(s, bg, &[(cx + 5, cy - 8), (cx + 3, cy - 3)], 1, false);
        }
        "LamaPlinkoGame" => {
            // Stift-Dreieck (1-2-3 Reihen) mit fallender Kugel und Faechern;
            // px rechnet Python mit Kommazahlen (row / 2.0), die Werte sind
            // aber immer ganz (Abstand 6).
            for row in 0..3 {
                for i in 0..=row {
                    let px = (cx as f64 + (i as f64 - row as f64 / 2.0) * 6.0) as i32;
                    let py = cy - 5 + row * 5;
                    oval(s, c, px - 1, py - 1, px + 1, py + 1);
                }
            }
            oval(s, c, cx + 1, cy - 11, cx + 5, cy - 7);
            for dx in [-9, -3, 3] {
                rect(s, c, cx + dx, cy + 8, cx + dx + 5, cy + 10);
            }
        }
        "LamaCrashGame" => {
            // Ballon mit Korb auf einer steigenden Kurve (smooth=True)
            let kurve = tk_spline(&[(cx - 10, cy + 9), (cx - 4, cy + 7), (cx + 1, cy + 3), (cx + 4, cy - 1)]);
            draw::lines(s, c, false, &kurve, 2);
            oval(s, c, cx - 1, cy - 12, cx + 9, cy - 2);
            line(s, c, &[(cx + 1, cy - 4), (cx + 3, cy + 1)], 1, false);
            line(s, c, &[(cx + 7, cy - 4), (cx + 5, cy + 1)], 1, false);
            rect(s, c, cx + 2, cy + 1, cx + 6, cy + 4);
        }
        _ => {
            let first: String = name.chars().take(1).collect();
            text(s, c, cx, cy, if first.is_empty() { "?" } else { &first }, 15);
        }
    }
}

fn py_pi() -> f64 {
    std::f64::consts::PI
}

/// `create_line(..., smooth=True)` - Tks Bezier-Kurve (`TkMakeBezierCurve`,
/// 12 Schritte je Stueck): offene Kurve durch Anfang, Mitten der inneren
/// Strecken und Ende.
fn tk_spline(p: &[(i32, i32)]) -> Vec<(f64, f64)> {
    let pf: Vec<(f64, f64)> = p.iter().map(|&(x, y)| (x as f64, y as f64)).collect();
    let n = pf.len();
    if n < 3 {
        return pf;
    }
    let mix = |a: (f64, f64), fa: f64, b: (f64, f64), fb: f64| (a.0 * fa + b.0 * fb, a.1 * fa + b.1 * fb);
    let mut out = vec![pf[0]];
    let mut c0 = pf[0];
    let mut c1 = mix(pf[0], 0.333, pf[1], 0.667);
    for i in 2..n {
        let (a, b) = (pf[i - 1], pf[i]);
        let (c2, c3) = if i == n - 1 { (mix(a, 0.667, b, 0.333), b) } else { (mix(a, 0.833, b, 0.167), mix(a, 0.5, b, 0.5)) };
        for k in 1..=12 {
            let t = k as f64 / 12.0;
            let u = 1.0 - t;
            let (w0, w1, w2, w3) = (u * u * u, 3.0 * t * u * u, 3.0 * t * t * u, t * t * t);
            out.push((
                c0.0 * w0 + c1.0 * w1 + c2.0 * w2 + c3.0 * w3,
                c0.1 * w0 + c1.1 * w1 + c2.1 * w2 + c3.1 * w3,
            ));
        }
        c0 = c3;
        c1 = mix(a, 0.167, b, 0.833);
    }
    out
}
