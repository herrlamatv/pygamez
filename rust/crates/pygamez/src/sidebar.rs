//! sidebar.rs
//! ==========
//! Die Seitenleiste - in Python ein Tkinter-Rahmen (`App._build_sidebar`,
//! `NeoButton`, `GameList`, Profil- und Statuskarte).
//!
//! Hier gibt es kein Tk: die Leiste wird mit rustgame in dieselbe Flaeche
//! gezeichnet wie das Spiel. Aufbau, Masse, Farben (die Tk-Paletten aus
//! `ui.py`) und Verhalten folgen der Python-Fassung; Tks Schriftgroessen in
//! Punkt werden zu Pixeln (`pt * dpi / 72`, bei 96 dpi also `pt * 4/3`).

use pgz_core::i18n::t;
use pgz_core::{levels, profiles};
use pgz_engine::pg::{self, draw, Color, Font, Rect, RectExt, SurfExt, Surface};
use pgz_engine::ui::{self, mix};
use pgz_engine::GameInfo;

use crate::icons;

/// Zeilenhoehe der Spieleliste (`GameList.ROW_H`).
const ROW_H: i32 = 44;
/// Abstand der Zeilen (`GameList.GAP`).
const GAP: i32 = 4;
/// Trefferbreite des Favoriten-Sterns am rechten Zeilenrand (`GameList.STAR_HIT`).
const STAR_HIT: i32 = 30;

/// Die Knoepfe unten, von oben nach unten (Python packt sie von unten).
pub const CTRL: &[(&str, &str)] = &[
    ("replays", "\u{1f3ac}"),
    ("lamawiki", "\u{1f4d6}"),
    ("progress", "\u{1f3c6}"),
    ("profiles", "\u{1f464}"),
    ("options", "\u{2699}"),
    ("language", "\u{1f310}"),
    ("fullscreen", "\u{26f6}"),
];

/// Was ein Klick in der Leiste ausloest.
#[derive(Debug, Clone, PartialEq)]
pub enum Aktion {
    /// Spiel `index` (in `games`) starten.
    Spiel(usize),
    /// Stern bei Spiel `index` (in `games`): Favorit umschalten.
    Favorit(usize),
    /// Steuer-Knopf (`"options"`, `"quit"`, `"back_to_menu"` ...).
    Knopf(String),
    /// Profilkarte.
    Profil,
}

/// Tk-Punkte in Pixel: `pt * dpi / 72` (bei 96 dpi also `pt * 4/3`).
fn pt(p: i32) -> i32 {
    (p as f64 * crate::fenster::dpi() as f64 / 72.0).round() as i32
}

/// Ein `NeoButton`.
#[derive(Debug, Clone)]
struct Knopf {
    key: String,
    icon: &'static str,
    rect: Rect,
    v: f64,
    gedrueckt: bool,
    fill: Color,
    hover: Color,
    bold: bool,
    center: bool,
}

/// Die ganze Leiste.
pub struct Sidebar {
    pub breite: i32,
    knoepfe: Vec<Knopf>,
    zeilen_v: Vec<f64>,
    hover_zeile: Option<usize>,
    /// Zeile, deren Stern gerade unter der Maus ist (`GameList._star_hover`).
    stern_hover: Option<usize>,
    /// Angezeigte Reihenfolge als Index in `games` (Favoriten ggf. oben).
    ordnung: Vec<usize>,
    /// highscore_keys der Favoriten (aktives Profil).
    pub favs: Vec<String>,
    scroll: i32,
    liste: Rect,
    profilkarte: Rect,
    maus: (i32, i32),
    pub aktiv: Option<usize>,
    pub status: String,
    pub perf: String,
    pub punkt: Color,
    pub scores: std::collections::BTreeMap<String, i64>,
    logo: Option<Surface>,
    hoehe: i32,
}

fn f_ui(px: i32, bold: bool) -> Font {
    thread_local! {
        static C: std::cell::RefCell<std::collections::HashMap<(String, i32, bool), Font>> =
            std::cell::RefCell::new(std::collections::HashMap::new());
    }
    sys("segoeui", px, bold, &C)
}

fn sys(
    name: &str,
    px: i32,
    bold: bool,
    c: &'static std::thread::LocalKey<std::cell::RefCell<std::collections::HashMap<(String, i32, bool), Font>>>,
) -> Font {
    let key = (name.to_string(), px, bold);
    if let Some(f) = c.with(|m| m.borrow().get(&key).cloned()) {
        return f;
    }
    let f = Font::sys(name, px, bold, false);
    c.with(|m| m.borrow_mut().insert(key, f.clone()));
    f
}

fn f_sym(px: i32) -> Font {
    thread_local! {
        static C: std::cell::RefCell<std::collections::HashMap<(String, i32, bool), Font>> =
            std::cell::RefCell::new(std::collections::HashMap::new());
    }
    sys("segoeuisymbol", px, false, &C)
}

fn f_mono(px: i32) -> Font {
    thread_local! {
        static C: std::cell::RefCell<std::collections::HashMap<(String, i32, bool), Font>> =
            std::cell::RefCell::new(std::collections::HashMap::new());
    }
    sys("consolas", px, false, &C)
}

/// `_round_rect(cv, x1, y1, x2, y2, r, fill, outline)`
fn round_rect(s: &mut Surface, x1: i32, y1: i32, x2: i32, y2: i32, r: i32, fill: Option<Color>, outline: Option<Color>) {
    let rr = Rect::new(x1, y1, x2 - x1, y2 - y1);
    if let Some(f) = fill {
        draw::rect_r(s, f, rr, 0, r);
    }
    if let Some(o) = outline {
        draw::rect_r(s, o, rr, 1, r);
    }
}

impl Sidebar {
    /// Neue Leiste.
    pub fn new() -> Sidebar {
        let logo = pg::image::load_bytes(include_bytes!("../../../../logo/pygamez3-128.png")).map(|img| {
            // Tk: `img.subsample(k, k)` mit k = round(128 / 30) = 4.
            let k = 4.max(((img.w() as f64) / 30.0).round() as i32);
            pg::transform::scale(&img, img.w() / k, img.h() / k)
        });
        Sidebar {
            breite: 232,
            knoepfe: Vec::new(),
            zeilen_v: Vec::new(),
            hover_zeile: None,
            stern_hover: None,
            ordnung: Vec::new(),
            favs: Vec::new(),
            scroll: 0,
            liste: Rect::new(0, 0, 0, 0),
            profilkarte: Rect::new(0, 0, 0, 0),
            maus: (-1, -1),
            aktiv: None,
            status: t("app.no_game"),
            perf: String::new(),
            punkt: ui::tk_colors().text_dim,
            scores: pgz_core::highscore::load(),
            logo,
            hoehe: 0,
        }
    }

    /// `_fit_sidebar` - Breite nach den Beschriftungen.
    pub fn fit(&mut self, games: &[GameInfo]) {
        let f_ctrl = f_ui(pt(9), false);
        let f_game = f_ui(pt(10), true);
        let mut texte: Vec<String> = CTRL.iter().map(|(k, _)| t(&format!("app.{k}"))).collect();
        texte.push(t("app.quit"));
        texte.push(t("app.back_to_menu"));
        let w_ctrl = texte.iter().map(|s| f_ctrl.width(s)).max().unwrap_or(0) + 60;
        // + Piktogramm links, + Favoriten-Stern rechts (STAR_HIT)
        let w_game = games.iter().map(|g| f_game.width(&g.name())).max().unwrap_or(0) + 80 + STAR_HIT + 6;
        self.breite = 340.min(232.max(w_ctrl).max(w_game));
    }

    /// `refresh_scores()`
    pub fn refresh_scores(&mut self) {
        self.scores = pgz_core::highscore::load();
    }

    /// `refresh_favorites()` - Favoriten neu lesen, nach "Favoriten oben"
    /// sortieren. Ziehen Zeilen um, gelten die Hover-Zustaende nicht mehr.
    pub fn refresh_favorites(&mut self, games: &[GameInfo], top: bool) {
        self.favs = pgz_core::favorites::load();
        let keys: Vec<&str> = games.iter().map(|g| g.key).collect();
        let neu = pgz_core::favorites::ordered(&keys, &self.favs, top);
        if neu != self.ordnung {
            self.zeilen_v = vec![0.0; games.len()];
            self.hover_zeile = None;
            self.stern_hover = None;
        }
        self.ordnung = neu;
    }

    /// Spiel-Index (in `games`) der angezeigten Zeile `i`.
    fn spiel_der_zeile(&self, i: usize) -> usize {
        self.ordnung.get(i).copied().unwrap_or(i)
    }

    fn layout(&mut self, h: i32) {
        let tk = ui::tk_colors();
        let w = self.breite;
        // Von unten: Statuskarte, Beenden, Zurueck, sieben Knoepfe, Trennlinie.
        let status_h = self.status_hoehe();
        let mut y = h - 12 - status_h - 4;
        let mut neu = Vec::new();
        let quit = Rect::new(12, y - 8 - 32, w - 24, 32);
        y = quit.y - 4;
        let back = Rect::new(12, y - 12 - 40, w - 24, 40);
        y = back.y - 4;
        let mut ctrl_rects = Vec::new();
        for _ in CTRL.iter().rev() {
            let r = Rect::new(12, y - 2 - 28, w - 24, 28);
            y = r.y - 2;
            ctrl_rects.push(r);
        }
        ctrl_rects.reverse();
        for ((key, icon), r) in CTRL.iter().zip(ctrl_rects) {
            neu.push(Knopf {
                key: key.to_string(),
                icon,
                rect: r,
                v: 0.0,
                gedrueckt: false,
                fill: tk.btn,
                hover: tk.btn_hover,
                bold: false,
                center: false,
            });
        }
        neu.push(Knopf {
            key: "back_to_menu".into(),
            icon: "\u{2302}",
            rect: back,
            v: 0.0,
            gedrueckt: false,
            fill: tk.back,
            hover: tk.back_hover,
            bold: true,
            center: true,
        });
        neu.push(Knopf {
            key: "quit".into(),
            icon: "\u{2715}",
            rect: quit,
            v: 0.0,
            gedrueckt: false,
            fill: tk.danger,
            hover: tk.danger_hover,
            bold: true,
            center: true,
        });
        // Hover-Werte bei gleichem Aufbau behalten.
        for k in &mut neu {
            if let Some(alt) = self.knoepfe.iter().find(|a| a.key == k.key) {
                k.v = alt.v;
                k.gedrueckt = alt.gedrueckt;
            }
        }
        self.knoepfe = neu;
        let trenn_y = y - 6;
        // Oben: Kopf, Linie, Profilkarte, Abschnitt "Spiele".
        let kopf_h = self.kopf_hoehe();
        let karte_y = kopf_h + 2 + 10;
        // Zeile = max(Punkt 22 px, Name + Rang), dazu Abstaende und Balken.
        let namen_h = f_ui(pt(10), true).get_height() + f_ui(pt(8), false).get_height();
        let karte_h = 2 + 6 + 22.max(namen_h) + 2 + 5 + 7;
        self.profilkarte = Rect::new(12, karte_y, w - 24, karte_h);
        let sec_y = karte_y + karte_h + 2 + 10;
        let sec_h = f_ui(pt(9), true).get_height();
        let liste_y = sec_y + sec_h + 4;
        self.liste = Rect::new(12, liste_y, w - 24, (trenn_y - 6 - liste_y).max(0));
        self.hoehe = h;
    }

    /// Hoehe, bei der die ganze Spieleliste ohne Rollen passt
    /// (`need_h` aus `_fit_sidebar`: alles ausser der Liste + alle Zeilen).
    pub fn bedarf_hoehe(&mut self, n_spiele: usize) -> i32 {
        let probe = 2000;
        self.layout(probe);
        let rest = probe - self.liste.h;
        self.layout(self.hoehe.max(1));
        rest + n_spiele as i32 * (ROW_H + GAP)
    }

    fn kopf_hoehe(&self) -> i32 {
        let titel_h = f_ui(pt(16), true).get_height().max(self.logo.as_ref().map(|l| l.h()).unwrap_or(0));
        14 + titel_h + 2 + f_ui(pt(9), false).get_height() + 10
    }

    fn status_zeilen(&self) -> Vec<String> {
        let f = f_mono(pt(9));
        let max_w = 120.max(self.breite - 70);
        let mut zeilen = Vec::new();
        for absatz in self.status.split('\n') {
            let mut cur = String::new();
            for wort in absatz.split(' ') {
                let probe = if cur.is_empty() { wort.to_string() } else { format!("{cur} {wort}") };
                if !cur.is_empty() && f.width(&probe) > max_w {
                    zeilen.push(cur);
                    cur = wort.to_string();
                } else {
                    cur = probe;
                }
            }
            zeilen.push(cur);
        }
        zeilen
    }

    fn status_hoehe(&self) -> i32 {
        let z = self.status_zeilen().len() as i32;
        6 + z * f_mono(pt(9)).get_linesize() + 4 + f_mono(pt(8)).get_height() + 6 + 2
    }

    /// Zeichnet die Leiste in `s` (links, volle Hoehe).
    pub fn draw(&mut self, s: &mut Surface, h: i32, games: &[GameInfo]) {
        self.layout(h);
        let tk = ui::tk_colors();
        let w = self.breite;
        s.fill_r(tk.sidebar, (0, 0, w, h));

        // --- Kopf -------------------------------------------------------------
        let kopf_h = self.kopf_hoehe();
        s.fill_r(tk.header, (0, 0, w, kopf_h));
        let titel = f_ui(pt(16), true).render("PyGameZ", true, tk.text);
        let beta = f_ui(pt(7), true).render("BETA", true, tk.accent2);
        let logo_w = self.logo.as_ref().map(|l| l.w() + 8).unwrap_or(0);
        let reihe_w = logo_w + titel.w() + 6 + beta.w();
        let reihe_h = titel.h().max(self.logo.as_ref().map(|l| l.h()).unwrap_or(0));
        let mut x = w / 2 - reihe_w / 2;
        if let Some(l) = &self.logo {
            s.bl(l, (x, 14 + (reihe_h - l.h()) / 2));
            x += l.w() + 8;
        }
        s.bl(&titel, (x, 14 + (reihe_h - titel.h()) / 2));
        s.bl(&beta, (x + titel.w() + 6, 14 + 3));
        let sub = f_ui(pt(9), false).render(&t("app.menu_title"), true, tk.text_dim);
        s.blr(&sub, sub.get_rect().midtop_at((w / 2, 14 + reihe_h + 2)));

        // --- Akzentlinie unter dem Kopf (_build_header_gradient) -------------
        let fx = ui::fx();
        let mut linie_h = 2;
        if let Some(g3) = fx.grad3 {
            let stops = [ui::c3(g3[0]), ui::c3(g3[1]), ui::c3(g3[2])];
            verlauf(s, 0, kopf_h, w, 2, &stops);
        } else if ui::is_modern() {
            s.fill_r(mix(tk.border, tk.accent, 0.35), (0, kopf_h, w, 1));
            linie_h = 1;
        } else if fx.style == "v1" {
            linie_h = 0;
        } else if fx.style == "v2" {
            s.fill_r(tk.accent, (0, kopf_h, w, 2));
        } else {
            verlauf(s, 0, kopf_h, w, 2, &[tk.accent, tk.accent2]);
        }
        let _ = linie_h;

        // --- Profilkarte -------------------------------------------------------
        self.draw_profil(s);

        // --- Abschnitt "Spiele" -------------------------------------------------
        let sec_f = f_ui(pt(9), true);
        let sec_y = self.profilkarte.bottom() + 2 + 10;
        // Tk packt den Abschnitt nach den Knoepfen unten: bei sehr kleinem
        // Fenster bekommt er nur den Rest ueber der Trennlinie und wird
        // abgeschnitten (nicht ueber die Knoepfe gezeichnet).
        let erster = self.knoepfe.first().map(|k| k.rect.y).unwrap_or(h);
        let alt_clip = s.get_clip();
        s.set_clip(Some(Rect::new(0, sec_y, w, (erster - 2 - 6 - 6 - sec_y).max(0))));
        s.bl(&sec_f.render(&t("app.games"), true, tk.accent), (16, sec_y));
        let anz = f_ui(pt(8), true).render(&games.len().to_string(), true, tk.text_faint);
        s.bl(&anz, (w - 16 - anz.w(), sec_y + 1));
        s.set_clip(Some(alt_clip));

        // --- Spieleliste ---------------------------------------------------------
        self.draw_liste(s, games);

        // --- Trennlinie, Knoepfe ----------------------------------------------
        s.fill_r(tk.border, (12, erster - 2 - 6 - 1, w - 24, 1));
        let knoepfe = self.knoepfe.clone();
        for k in &knoepfe {
            self.draw_knopf(s, k);
        }

        // --- Statuskarte -----------------------------------------------------
        self.draw_status(s, h);
    }

    fn draw_profil(&self, s: &mut Surface) {
        let tk = ui::tk_colors();
        let r = self.profilkarte;
        s.fill_r(tk.card, r);
        draw::rect(s, tk.border, r, 1);
        let pid = profiles::active_id();
        let (level, points) = profiles::level_of(Some(&pid));
        let name = profiles::name_of(Some(&pid));
        let farbe = ui::c3(profiles::color_rgb(Some(&pid)));
        // Punkt mit Anfangsbuchstabe.
        let namen_h = f_ui(pt(10), true).get_height() + f_ui(pt(8), false).get_height();
        let zeile_h = 22.max(namen_h);
        let (dx, dy) = (r.x + 1 + 8, r.y + 1 + 6 + (zeile_h - 22) / 2);
        let ny = r.y + 1 + 6 + (zeile_h - namen_h) / 2;
        draw::ellipse(s, mix(tk.card, farbe, 0.45), (dx + 2, dy + 2, 18, 18), 0);
        draw::ellipse(s, farbe, (dx + 2, dy + 2, 18, 18), 2);
        let initial: String = name.chars().take(1).collect::<String>().to_uppercase();
        let ini = f_ui(pt(9), true).render(if initial.is_empty() { "?" } else { &initial }, true, tk.text);
        s.blr(&ini, ini.get_rect().center_at((dx + 11, dy + 11)));
        let nf = f_ui(pt(10), true);
        let rf = f_ui(pt(8), false);
        let nx = dx + 22 + 8;
        let name_img = nf.render(&name, true, tk.text);
        s.bl(&name_img, (nx, ny));
        let rang = format!("{} {}  \u{b7}  {}", t("level.short"), level, levels::rank_title(level));
        s.bl(&rf.render(&rang, true, tk.text_dim), (nx, ny + name_img.h()));
        // XP-Balken.
        let (_lv, in_level, need) = levels::progress(points);
        let bar = Rect::new(r.x + 1 + 8, r.bottom() - 1 - 7 - 5, r.w - 2 - 16, 5);
        s.fill_r(tk.btn, bar);
        let fill = (bar.w as f64 * (in_level as f64 / need.max(1) as f64).min(1.0)) as i32;
        if fill > 0 {
            s.fill_r(farbe, (bar.x, bar.y, fill, bar.h));
        }
    }

    fn draw_liste(&mut self, s: &mut Surface, games: &[GameInfo]) {
        let tk = ui::tk_colors();
        let r = self.liste;
        if self.zeilen_v.len() != games.len() {
            self.zeilen_v = vec![0.0; games.len()];
        }
        // Hover-Animation (GameList._animate: 30 % je Schritt).
        for (i, v) in self.zeilen_v.iter_mut().enumerate() {
            let target = if Some(i) == self.hover_zeile { 1.0 } else { 0.0 };
            if (*v - target).abs() < 0.05 {
                *v = target;
            } else {
                *v += (target - *v) * 0.3;
            }
        }
        let total = games.len() as i32 * (ROW_H + GAP);
        let max_scroll = (total - r.h).max(0);
        self.scroll = self.scroll.clamp(0, max_scroll);
        let alt_clip = s.get_clip();
        s.set_clip(Some(r));
        let w = r.w;
        for i in 0..games.len() {
            let gi = self.spiel_der_zeile(i);
            let g = &games[gi.min(games.len() - 1)];
            let y0 = r.y + i as i32 * (ROW_H + GAP) - self.scroll;
            let y1 = y0 + ROW_H;
            if y1 < r.y || y0 > r.bottom() {
                continue;
            }
            let cy = (y0 + y1) / 2;
            let accent = ui::game_color(g.class_name);
            let active = self.aktiv == Some(gi);
            let v = self.zeilen_v[i];
            let base = if active { mix(tk.btn, accent, 0.14) } else { tk.btn };
            let hover = mix(tk.btn_hover, accent, 0.10);
            let x0 = r.x;
            round_rect(
                s,
                x0 + 1,
                y0 + 1,
                x0 + w - 7,
                y1 - 1,
                10,
                Some(mix(base, hover, v)),
                if active { Some(accent) } else { None },
            );
            let m = if active { 8 } else { (14.0 - 6.0 * v) as i32 };
            s.fill_r(accent, (x0 + 6, y0 + m, 4, ROW_H - 2 * m));
            let chip_bg = mix(accent, tk.sidebar, 0.84);
            round_rect(s, x0 + 17, y0 + 8, x0 + 45, y1 - 8, 8, Some(chip_bg), None);
            icons::draw_icon(s, x0 + 31, cy, g.class_name, accent, chip_bg);
            let hs = self.scores.get(g.key).copied().unwrap_or(0);
            let nf = f_ui(pt(10), true);
            let name = nf.render(&g.name(), true, tk.text);
            if hs > 0 {
                s.blr(&name, name.get_rect().midleft_at((x0 + 54, cy - 9)));
                // Pokal statt Stern - der Stern gehoert jetzt den Favoriten.
                let txt = format!("\u{1f3c6} {}", tausender(hs));
                let img = f_sym(pt(8)).render(&txt, true, tk.text_dim);
                s.blr(&img, img.get_rect().midleft_at((x0 + 54, cy + 9)));
            } else {
                s.blr(&name, name.get_rect().midleft_at((x0 + 54, cy)));
            }
            // Favoriten-Stern: gold gefuellt (Favorit), sonst blasser Umriss,
            // beim Zeilen-Hover heller, direkt darueber gold.
            let fav = self.favs.iter().any(|f| f == g.key);
            let stern_hover = self.stern_hover == Some(i);
            let farbe = if fav {
                if stern_hover { mix(tk.gold, Color::rgb(255, 255, 255), 0.25) } else { tk.gold }
            } else if stern_hover {
                tk.gold
            } else {
                mix(tk.text_faint, tk.text_dim, v)
            };
            let stern = f_sym(pt(12)).render(if fav { "\u{2605}" } else { "\u{2606}" }, true, farbe);
            s.blr(&stern, stern.get_rect().center_at((x0 + w - 7 - STAR_HIT / 2, cy)));
        }
        // Scroll-Anzeige.
        if total > r.h && r.h > 1 {
            let f0 = self.scroll as f64 / total as f64;
            let f1 = (self.scroll + r.h) as f64 / total as f64;
            let y_a = r.y + (f0 * r.h as f64) as i32;
            let y_b = r.y + (f1 * r.h as f64) as i32;
            s.fill_r(Color::rgb(0x3a, 0x44, 0x60), (r.x + w - 4, y_a, 3, y_b - y_a));
        }
        s.set_clip(Some(alt_clip));
    }

    fn draw_knopf(&self, s: &mut Surface, k: &Knopf) {
        let tk = ui::tk_colors();
        let r = k.rect;
        let mut col = mix(k.fill, k.hover, k.v);
        if k.gedrueckt {
            col = mix(col, Color::rgb(0, 0, 0), 0.18);
        }
        let dy = if k.gedrueckt { 1 } else { 0 };
        round_rect(s, r.x + 1, r.y + 1, r.right() - 2, r.bottom() - 2, 9, Some(col), Some(mix(tk.border, tk.accent, k.v * 0.7)));
        let fg = mix(tk.text, Color::rgb(255, 255, 255), k.v * 0.35);
        let font = f_ui(pt(9), k.bold);
        let text = if k.key == "quit" || k.key == "back_to_menu" { t(&format!("app.{}", k.key)) } else { t(&format!("app.{}", k.key)) };
        let img = font.render(&text, true, fg);
        let cy = r.y + r.h / 2 + dy;
        if k.center {
            // Tk misst mit der normalen Schrift und zeichnet fett (NeoButton).
            let x = r.x + r.w / 2 - f_ui(pt(9), false).width(&text) / 2;
            let ic = f_sym(pt(10)).render(k.icon, true, fg);
            s.blr(&ic, ic.get_rect().midright_at((x - 8, cy)));
            s.blr(&img, img.get_rect().midleft_at((x, cy)));
        } else {
            let ic = f_sym(pt(10)).render(k.icon, true, mix(tk.text_dim, tk.accent, k.v));
            s.blr(&ic, ic.get_rect().midleft_at((r.x + 12, cy)));
            s.blr(&img, img.get_rect().midleft_at((r.x + 12 + 22, cy)));
        }
    }

    fn draw_status(&self, s: &mut Surface, h: i32) {
        let tk = ui::tk_colors();
        let sh = self.status_hoehe();
        let r = Rect::new(12, h - 12 - sh, self.breite - 24, sh);
        s.fill_r(tk.card, r);
        draw::rect(s, tk.border, r, 1);
        s.fill_r(tk.accent, (r.x + 1, r.y + 1, 3, r.h - 2));
        let bx = r.x + 1 + 3 + 8;
        let mut y = r.y + 1 + 6;
        draw::ellipse(s, self.punkt, (bx + 1, y + 3 + 1, 7, 7), 0);
        let f = f_mono(pt(9));
        for z in self.status_zeilen() {
            s.bl(&f.render(&z, true, tk.text_dim), (bx + 9 + 6, y));
            y += f.get_linesize();
        }
        if !self.perf.is_empty() {
            s.bl(&f_mono(pt(8)).render(&self.perf, true, tk.text_faint), (bx, y + 4));
        }
    }

    // ----- Maus ---------------------------------------------------------------

    /// Mausbewegung (Koordinaten der Leiste).
    pub fn bewegung(&mut self, pos: (i32, i32)) {
        self.maus = pos;
        for k in &mut self.knoepfe {
            if !k.rect.hit(pos) {
                k.gedrueckt = false;
            }
        }
        self.hover_zeile = self.zeile_bei(pos);
        self.stern_hover = self.hover_zeile.filter(|_| self.stern_bei(pos));
    }

    /// Maus hat die Leiste verlassen.
    pub fn verlassen(&mut self) {
        self.maus = (-1, -1);
        self.hover_zeile = None;
        self.stern_hover = None;
        for k in &mut self.knoepfe {
            k.gedrueckt = false;
        }
    }

    /// Hover-Animation der Knoepfe (NeoButton._step: 30 % je Schritt).
    pub fn animieren(&mut self) {
        let maus = self.maus;
        for k in &mut self.knoepfe {
            let target = if k.rect.hit(maus) { 1.0 } else { 0.0 };
            let d = target - k.v;
            if d.abs() < 0.05 {
                k.v = target;
            } else {
                k.v += d * 0.3;
            }
        }
    }

    fn zeile_bei(&self, pos: (i32, i32)) -> Option<usize> {
        if !self.liste.hit(pos) {
            return None;
        }
        let y = pos.1 - self.liste.y + self.scroll;
        let i = y / (ROW_H + GAP);
        if y % (ROW_H + GAP) <= ROW_H && i >= 0 {
            Some(i as usize)
        } else {
            None
        }
    }

    /// Maustaste gedrueckt.
    pub fn druecken(&mut self, pos: (i32, i32), anzahl: usize) -> Option<Aktion> {
        for k in &mut self.knoepfe {
            if k.rect.hit(pos) {
                k.gedrueckt = true;
                return None;
            }
        }
        if self.profilkarte.hit(pos) {
            return Some(Aktion::Profil);
        }
        if let Some(i) = self.zeile_bei(pos) {
            if i < anzahl {
                let gi = self.spiel_der_zeile(i);
                if self.stern_bei(pos) {
                    // Stern: nur Favorit umschalten, kein Spielstart.
                    return Some(Aktion::Favorit(gi));
                }
                if let Some(v) = self.zeilen_v.get_mut(i) {
                    *v = 1.0;
                }
                return Some(Aktion::Spiel(gi));
            }
        }
        None
    }

    /// `_star_hit(x)` - liegt x im Stern-Bereich am rechten Rand der Zeilenkarte?
    fn stern_bei(&self, pos: (i32, i32)) -> bool {
        let right = self.liste.x + self.liste.w - 7;
        right - STAR_HIT <= pos.0 && pos.0 <= right
    }

    /// Maustaste losgelassen - NeoButtons loesen erst hier aus.
    pub fn loslassen(&mut self, pos: (i32, i32)) -> Option<Aktion> {
        for k in &mut self.knoepfe {
            let war = k.gedrueckt;
            k.gedrueckt = false;
            if war && k.rect.hit(pos) {
                return Some(Aktion::Knopf(k.key.clone()));
            }
        }
        None
    }

    /// Mausrad ueber der Liste.
    pub fn rad(&mut self, pos: (i32, i32), delta: i32) {
        if self.liste.hit(pos) {
            // Tk: yview_scroll(-delta * 4, "units") mit 8 px je Einheit.
            self.scroll -= delta * 4 * 8;
        }
    }
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

/// Waagerechter Verlauf (der 2-px-Canvas aus `_header_gradient_canvas`).
fn verlauf(s: &mut Surface, x: i32, y: i32, w: i32, h: i32, stops: &[Color]) {
    let seg = (stops.len() - 1).max(1);
    let n = 16 * seg + 16;
    for i in 0..n {
        let f = i as f64 / (n - 1) as f64 * seg as f64;
        let k = (seg - 1).min(f as usize);
        let col = mix(stops[k], stops[k + 1], f - k as f64);
        let x1 = x + (w as f64 * i as f64 / n as f64) as i32;
        let x2 = x + (w as f64 * (i + 1) as f64 / n as f64) as i32 + 1;
        s.fill_r(col, (x1, y, x2 - x1, h));
    }
}

/// `f"{hs:,}".replace(",", " ")`
pub fn tausender(v: i64) -> String {
    let s = v.abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(' ');
        }
        out.push(c);
    }
    if v < 0 {
        format!("-{out}")
    } else {
        out
    }
}
