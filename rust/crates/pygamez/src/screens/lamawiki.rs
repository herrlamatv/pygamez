//! lamawiki.rs
//! ===========
//! LamaWiki - das In-Game-Wiki der Spielesammlung. Uebertragen aus
//! `lamawiki/lamawiki.py`. Links Suchfeld und kategorisierte Seitenliste,
//! rechts der scrollbare Artikel. Inhalte: `lamawiki/<sprache>.json` (fuenf
//! Sprachen) und `lamawiki/lang.expansion/` (neun weitere), eingebunden.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use pgz_core::i18n::{self, t};
use pgz_engine::game::{Game, GameBase, InputEvent, Kind};
use pgz_engine::pg::{draw, Font, Rect, RectExt, SurfExt, Surface};
use pgz_engine::shell::{self, Cmd};
use pgz_engine::ui::{self, mix};
use serde_json::Value;

const LINE_H: i32 = 22;
const PARA_GAP: i32 = 8;
const SECT_GAP: i32 = 16;
const BULLET_INDENT: i32 = 16;

macro_rules! wiki {
    ($($code:literal => $pfad:literal),* $(,)?) => {
        fn wiki_text(code: &str, expansion: bool) -> Option<&'static str> {
            match (code, expansion) {
                $(($code, x) if x == $pfad.contains("lang.expansion") => Some(include_str!($pfad)),)*
                _ => None,
            }
        }
    };
}

wiki! {
    "de" => "../../../../../lamawiki/de.json",
    "en" => "../../../../../lamawiki/en.json",
    "fr" => "../../../../../lamawiki/fr.json",
    "es" => "../../../../../lamawiki/es.json",
    "pt" => "../../../../../lamawiki/pt.json",
    "pl" => "../../../../../lamawiki/lang.expansion/pl.json",
    "tr" => "../../../../../lamawiki/lang.expansion/tr.json",
    "da" => "../../../../../lamawiki/lang.expansion/da.json",
    "no" => "../../../../../lamawiki/lang.expansion/no.json",
    "sv" => "../../../../../lamawiki/lang.expansion/sv.json",
    "fi" => "../../../../../lamawiki/lang.expansion/fi.json",
    "cs" => "../../../../../lamawiki/lang.expansion/cs.json",
    "sl" => "../../../../../lamawiki/lang.expansion/sl.json",
    "hr" => "../../../../../lamawiki/lang.expansion/hr.json",
}

/// Eine Wiki-Seite.
#[derive(Clone, Debug)]
pub struct Page {
    /// Stabile id.
    pub id: String,
    /// Kategorie.
    pub category: String,
    /// Klassenname des Spiels (leer = keins).
    pub game: String,
    /// Titel.
    pub title: String,
    /// Suchwoerter.
    pub keywords: Vec<String>,
    /// (Ueberschrift, Absaetze).
    pub sections: Vec<(String, Vec<String>)>,
}

fn parse_pages(text: &str) -> Vec<Page> {
    let Ok(data) = serde_json::from_str::<Value>(text) else {
        return Vec::new();
    };
    let s = |v: &Value, k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
    data.get("pages")
        .and_then(|p| p.as_array())
        .map(|list| {
            list.iter()
                .filter(|p| p.is_object())
                .filter(|p| !s(p, "id").is_empty() && !s(p, "title").is_empty() && p.get("sections").and_then(|x| x.as_array()).is_some_and(|a| !a.is_empty()))
                .map(|p| Page {
                    id: s(p, "id"),
                    category: p.get("category").and_then(|x| x.as_str()).unwrap_or("general").to_string(),
                    game: s(p, "game"),
                    title: s(p, "title"),
                    keywords: p.get("keywords").and_then(|x| x.as_array()).map(|a| a.iter().filter_map(|k| k.as_str().map(String::from)).collect()).unwrap_or_default(),
                    sections: p["sections"]
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .map(|sec| {
                                    (s(sec, "h"), sec.get("body").and_then(|b| b.as_array()).map(|b| b.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default())
                                })
                                .collect()
                        })
                        .unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// `load_pages(lang)` - Seiten der Sprache (Fallback: Deutsch).
pub fn load_pages(lang: Option<&str>) -> Vec<Page> {
    static CACHE: OnceLock<Mutex<HashMap<String, Vec<Page>>>> = OnceLock::new();
    let lang = lang.map(String::from).unwrap_or_else(i18n::get_language);
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(p) = cache.lock().unwrap_or_else(|e| e.into_inner()).get(&lang) {
        return p.clone();
    }
    let mut pages = Vec::new();
    'aussen: for code in [lang.as_str(), "de"] {
        for exp in [false, true] {
            if let Some(text) = wiki_text(code, exp) {
                let cand = parse_pages(text);
                if !cand.is_empty() {
                    pages = cand;
                    break 'aussen;
                }
            }
        }
    }
    cache.lock().unwrap_or_else(|e| e.into_inner()).insert(lang, pages.clone());
    pages
}

/// `page_id_for_game(cls_name)` - Seiten-id zur Spielklasse.
pub fn page_id_for_game(cls_name: &str) -> Option<String> {
    load_pages(None).into_iter().find(|p| p.game == cls_name).map(|p| p.id)
}

#[derive(Clone, Debug)]
enum Tok {
    Word(String),
    Chip(String, String),
}

fn chip_of(raw: &str) -> Option<(String, String)> {
    let rest = raw.strip_prefix('[')?;
    let end = rest.find(']')?;
    let inner = &rest[..end];
    let trail = &rest[end + 1..];
    let n = inner.chars().count();
    if !(1..=14).contains(&n) || inner.contains('[') {
        return None;
    }
    let tn = trail.chars().count();
    if tn > 1 || (tn == 1 && !".,:;!?".contains(trail)) {
        return None;
    }
    Some((inner.to_string(), trail.to_string()))
}

/// `parse_inline(text)` - (ist Aufzaehlung, Tokens).
fn parse_inline(text: &str) -> (bool, Vec<Tok>) {
    let is_bullet = text.starts_with("- ");
    let text = if is_bullet { &text[2..] } else { text };
    let toks = text
        .split_whitespace()
        .map(|raw| match chip_of(raw) {
            Some((l, tr)) => Tok::Chip(l, tr),
            None => Tok::Word(raw.to_string()),
        })
        .collect();
    (is_bullet, toks)
}

#[derive(Clone, Debug)]
enum Op {
    Title(String),
    Line(i32),
    Head(String),
    Dot,
    Word(String),
    Chip(String, String),
}

struct RowW {
    cat: Option<String>,
    page: Option<usize>,
    y: i32,
    h: i32,
}

fn fonts() -> (Font, Font, Font, Font) {
    (ui::fb(20), ui::fb(16), ui::font(15, false, false), ui::font(13, false, true))
}

/// Das LamaWiki.
pub struct LamaWikiScreen {
    b: GameBase,
    back: Option<String>,
    pages: Vec<Page>,
    query: String,
    hover: Option<usize>,
    list_scroll: i32,
    art_scroll: i32,
    art_cache: HashMap<(String, i32), (Vec<(Op, i32, i32)>, i32)>,
    page: Option<usize>,
    filtered: Vec<usize>,
    search_rect: Rect,
    list_rect: Rect,
    art_rect: Rect,
    rows: Vec<RowW>,
    rows_h: i32,
}

impl LamaWikiScreen {
    /// `LamaWikiScreen(surface, w, h, app, on_close, page_id)`; `back` =
    /// Spiel-Schluessel, zu dessen Vorspiel "Schliessen" fuehrt.
    pub fn new(b: GameBase, page_id: Option<String>, back: Option<String>) -> LamaWikiScreen {
        let z = Rect::new(0, 0, 0, 0);
        let mut s = LamaWikiScreen {
            b,
            back,
            pages: Vec::new(),
            query: String::new(),
            hover: None,
            list_scroll: 0,
            art_scroll: 0,
            art_cache: HashMap::new(),
            page: None,
            filtered: Vec::new(),
            search_rect: z,
            list_rect: z,
            art_rect: z,
            rows: Vec::new(),
            rows_h: 0,
        };
        // Python: reset() laeuft in Game.__init__, vor dem Erfolg.
        let wanted = match page_id {
            Some(p) if p.starts_with("game:") => page_id_for_game(&p["game:".len()..]),
            other => other,
        };
        s.pages = load_pages(None);
        s.apply_filter(false);
        if let Some(w) = wanted.filter(|w| !w.is_empty()) {
            if let Some(i) = s.pages.iter().position(|p| p.id == w) {
                s.page = Some(i);
            }
        }
        s.build_layout();
        s.ensure_selection_visible();
        pgz_core::achievements::event("wiki_reader", None);
        s
    }

    fn close(&self) {
        match &self.back {
            Some(k) => shell::post(Cmd::PreGame(k.clone())),
            None => shell::post(Cmd::BackToMenu),
        }
    }

    fn build_layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        let lw = 150.max(250.min((w as f64 * 0.30) as i32));
        self.search_rect = Rect::new(16, 48, lw, 26);
        self.list_rect = Rect::new(16, 84, lw, h - 34 - 84);
        let ax = 16 + lw + 12;
        self.art_rect = Rect::new(ax, 48, w - ax - 16, h - 34 - 48);
        self.build_rows();
    }

    fn build_rows(&mut self) {
        self.rows.clear();
        let mut y = 0;
        let mut last: Option<String> = None;
        for &pi in &self.filtered {
            let cat = self.pages[pi].category.clone();
            if last.as_deref() != Some(cat.as_str()) {
                last = Some(cat.clone());
                self.rows.push(RowW { cat: Some(t(&format!("lamawiki.cat.{cat}"))), page: None, y, h: 22 });
                y += 22;
            }
            self.rows.push(RowW { cat: None, page: Some(pi), y, h: 26 });
            y += 26;
        }
        self.rows_h = y;
        self.list_scroll = 0.max(self.list_scroll.min(0.max(y - self.list_rect.h)));
    }

    fn apply_filter(&mut self, layout_ready: bool) {
        let q = self.query.trim().to_lowercase();
        self.filtered = if q.is_empty() {
            (0..self.pages.len()).collect()
        } else {
            (0..self.pages.len())
                .filter(|&i| {
                    let p = &self.pages[i];
                    p.title.to_lowercase().contains(&q) || p.keywords.iter().any(|k| k.contains(&q))
                })
                .collect()
        };
        if !self.page.is_some_and(|p| self.filtered.contains(&p)) {
            self.page = self.filtered.first().copied();
            self.art_scroll = 0;
        }
        if layout_ready {
            self.build_rows();
            self.ensure_selection_visible();
        }
    }

    fn select(&mut self, pi: usize) {
        if Some(pi) != self.page {
            self.page = Some(pi);
            self.art_scroll = 0;
        }
    }

    fn ensure_selection_visible(&mut self) {
        let lh = self.list_rect.h;
        if let Some(r) = self.rows.iter().find(|r| r.page.is_some() && r.page == self.page) {
            if r.y < self.list_scroll {
                self.list_scroll = r.y;
            } else if r.y + r.h > self.list_scroll + lh {
                self.list_scroll = r.y + r.h - lh;
            }
        }
    }

    fn max_art_scroll(&mut self) -> i32 {
        let Some(pi) = self.page else { return 0 };
        let (_ops, content_h) = self.article_layout(pi);
        0.max(content_h - (self.art_rect.h - 24))
    }

    fn on_key(&mut self, key: &str) {
        if key == "Escape" {
            if !self.query.is_empty() {
                self.query.clear();
                self.apply_filter(true);
            } else {
                self.close();
            }
            return;
        }
        if key == "Up" || key == "Down" {
            let pages: Vec<usize> = self.rows.iter().filter_map(|r| r.page).collect();
            if pages.is_empty() {
                return;
            }
            let idx = pages.iter().position(|p| Some(*p) == self.page).unwrap_or(0) as i64;
            let idx = 0.max((pages.len() as i64 - 1).min(idx + if key == "Down" { 1 } else { -1 })) as usize;
            self.select(pages[idx]);
            self.ensure_selection_visible();
            self.b.play_sound("move");
        } else if key == "Prior" || key == "Next" {
            let step = ((self.art_rect.h - 24) as f64 * 0.8) as i32;
            let step = if key == "Next" { step } else { -step };
            let m = self.max_art_scroll();
            self.art_scroll = 0.max((self.art_scroll + step).min(m));
        } else if key == "BackSpace" {
            if !self.query.is_empty() {
                self.query.pop();
                self.apply_filter(true);
            }
        } else if key == "space" {
            if !self.query.is_empty() {
                self.query.push(' ');
                self.apply_filter(true);
            }
        } else if key.chars().count() == 1 && key.chars().all(|c| !c.is_control()) {
            self.query.push_str(&key.to_lowercase());
            self.apply_filter(true);
        }
    }

    fn row_at(&self, pos: (i32, i32)) -> Option<usize> {
        if !self.list_rect.hit(pos) {
            return None;
        }
        let y = pos.1 - self.list_rect.y + self.list_scroll;
        self.rows.iter().position(|r| r.y <= y && y < r.y + r.h)
    }

    fn article_layout(&mut self, pi: usize) -> (Vec<(Op, i32, i32)>, i32) {
        let inner_w = self.art_rect.w - 32;
        let key = (self.pages[pi].id.clone(), inner_w);
        if let Some(c) = self.art_cache.get(&key) {
            return c.clone();
        }
        let (f_title, f_head, f_body, f_chip) = fonts();
        let page = &self.pages[pi];
        let mut ops = Vec::new();
        let mut y = 0;
        ops.push((Op::Title(page.title.clone()), 0, y));
        y += f_title.get_height() + 4;
        let tw = (f_title.size(&page.title).0 + 16).min(inner_w);
        ops.push((Op::Line(tw), 0, y));
        y += 12;
        for (h, body) in &page.sections {
            y += SECT_GAP;
            ops.push((Op::Head(h.clone()), 0, y));
            y += f_head.get_height() + 6;
            for para in body {
                let (is_bullet, toks) = parse_inline(para);
                let x0 = if is_bullet { BULLET_INDENT } else { 0 };
                if is_bullet {
                    ops.push((Op::Dot, 5, y + LINE_H / 2));
                }
                let mut x = x0;
                for tok in toks {
                    let wpx = match &tok {
                        Tok::Word(w) => f_body.size(&format!("{w} ")).0,
                        Tok::Chip(l, tr) => f_chip.size(l).0 + 14 + 6 + if tr.is_empty() { 0 } else { f_body.size(tr).0 },
                    };
                    if x > x0 && x + wpx > inner_w {
                        x = x0;
                        y += LINE_H;
                    }
                    let op = match tok {
                        Tok::Word(w) => Op::Word(w),
                        Tok::Chip(l, tr) => Op::Chip(l, tr),
                    };
                    ops.push((op, x, y));
                    x += wpx;
                }
                y += LINE_H + PARA_GAP;
            }
            y -= PARA_GAP;
        }
        let result = (ops, y + 8);
        if self.art_cache.len() > 24 {
            self.art_cache.clear();
        }
        self.art_cache.insert(key, result.clone());
        result
    }

    fn draw_search(&self, s: &mut Surface) {
        let p = ui::pal();
        let r = self.search_rect;
        ui::draw_panel_ex(s, r, None, None, Some(8), false, None);
        let img = if !self.query.is_empty() { ui::font(14, false, true).render(&self.query, true, p.text) } else { ui::font(14, false, false).render(&t("lamawiki.search"), true, p.text_faint) };
        let prev = s.get_clip();
        s.set_clip(Some(r.inflate(-8, 0)));
        s.blr(&img, img.get_rect().midleft_at((r.x + 8, r.centery())));
        if ui::pulse_d(4.0) > 0.5 {
            let cx = (r.right() - 6).min(r.x + 8 + if !self.query.is_empty() { img.w() } else { 0 });
            draw::line(s, p.accent, (cx + 1, r.y + 5), (cx + 1, r.bottom() - 5), 1);
        }
        s.set_clip(Some(prev));
    }

    fn draw_list(&self, s: &mut Surface) {
        let p = ui::pal();
        let lr = self.list_rect;
        let f_cat = ui::fb(12);
        let f_row = ui::font(15, false, false);
        let prev = s.get_clip();
        s.set_clip(Some(lr));
        if self.rows.is_empty() {
            let img = f_row.render(&t("lamawiki.no_results"), true, p.text_faint);
            s.blr(&img, img.get_rect().center_at(lr.center()));
        }
        for (i, r) in self.rows.iter().enumerate() {
            let y = lr.y + r.y - self.list_scroll;
            if y + r.h < lr.y || y > lr.bottom() {
                continue;
            }
            if let Some(label) = &r.cat {
                let img = f_cat.render(label, true, p.accent);
                s.bl(&img, (lr.x + 2, y + 6));
                continue;
            }
            let pi = r.page.expect("page");
            let page = &self.pages[pi];
            let row_rect = Rect::new(lr.x, y, lr.w - 6, r.h - 2);
            let selected = Some(pi) == self.page;
            if selected {
                draw::rect_r(s, p.panel_light, row_rect, 0, 6);
                draw::rect_r(s, p.accent, (row_rect.x, row_rect.y + 3, 3, row_rect.h - 6), 0, 2);
            } else if Some(i) == self.hover {
                draw::rect_r(s, p.btn, row_rect, 0, 6);
            }
            let mut x = lr.x + 10;
            if !page.game.is_empty() {
                draw::circle(s, ui::game_color(&page.game), (x + 3, row_rect.centery()), 4, 0);
                x += 14;
            }
            let img = f_row.render(&page.title, true, if selected { p.text } else { p.text_dim });
            s.blr(&img, img.get_rect().midleft_at((x, row_rect.centery())));
        }
        s.set_clip(Some(prev));
        if self.rows_h > lr.h {
            let f0 = self.list_scroll as f64 / self.rows_h as f64;
            let f1 = (self.list_scroll + lr.h) as f64 / self.rows_h as f64;
            draw::rect_r(s, p.border_light, (lr.right() - 3, lr.y + (f0 * lr.h as f64) as i32, 3, 12.max(((f1 - f0) * lr.h as f64) as i32)), 0, 2);
        }
    }

    fn draw_article(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let ar = self.art_rect;
        let Some(pi) = self.page else {
            ui::draw_panel_ex(s, ar, None, None, Some(10), false, None);
            return;
        };
        let accent = if !self.pages[pi].game.is_empty() { ui::game_color(&self.pages[pi].game) } else { p.accent };
        ui::draw_panel_ex(s, ar, None, None, Some(10), false, Some(mix(p.panel, accent, 0.45)));
        let (ops, content_h) = self.article_layout(pi);
        let inner = ar.inflate(-32, -24);
        let (f_title, f_head, f_body, f_chip) = fonts();
        let prev = s.get_clip();
        s.set_clip(Some(inner));
        let oy = inner.y - self.art_scroll;
        for (op, x, y) in &ops {
            let yy = oy + y;
            if yy > inner.bottom() || yy + LINE_H + 10 < inner.y {
                continue;
            }
            let xx = inner.x + x;
            match op {
                Op::Title(text) => {
                    let img = f_title.render(text, true, p.text);
                    s.bl(&img, (xx, yy));
                }
                Op::Line(w) => {
                    draw::rect_r(s, accent, (xx, yy, *w, 3), 0, 2);
                }
                Op::Head(text) => {
                    let img = f_head.render(text, true, accent);
                    s.bl(&img, (xx, yy));
                }
                Op::Dot => {
                    draw::circle(s, accent, (xx, yy), 3, 0);
                }
                Op::Word(w) => {
                    let img = f_body.render(w, true, p.text_dim);
                    s.bl(&img, (xx, yy + f_body.get_height() - img.h()));
                }
                Op::Chip(label, trail) => {
                    let img = f_chip.render(label, true, p.gold);
                    let chip = Rect::new(xx, yy + 1, img.w() + 14, LINE_H - 4);
                    draw::rect_r(s, p.panel_light, chip, 0, 5);
                    draw::rect_r(s, p.border_light, chip, 1, 5);
                    s.blr(&img, img.get_rect().center_at(chip.center()));
                    if !trail.is_empty() {
                        let timg = f_body.render(trail, true, p.text_dim);
                        s.bl(&timg, (chip.right() + 1, yy + f_body.get_height() - timg.h()));
                    }
                }
            }
        }
        s.set_clip(Some(prev));
        let view_h = inner.h;
        if content_h > view_h {
            let f0 = self.art_scroll as f64 / content_h as f64;
            let f1 = (self.art_scroll + view_h) as f64 / content_h as f64;
            draw::rect_r(s, p.border_light, (ar.right() - 7, ar.y + 6 + (f0 * (ar.h - 12) as f64) as i32, 3, 16.max(((f1 - f0) * (ar.h - 12) as f64) as i32)), 0, 2);
        }
    }
}

impl Game for LamaWikiScreen {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }
    fn reset(&mut self) {}
    fn update(&mut self, _dt: f64) {}
    fn screen_name(&self) -> String {
        t("lamawiki.name")
    }
    fn on_surface_changed(&mut self) {
        self.build_layout();
        self.art_cache.clear();
        let m = self.max_art_scroll();
        self.art_scroll = 0.max(self.art_scroll.min(m));
        self.ensure_selection_visible();
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        match ev.kind {
            Kind::KeyDown => self.on_key(&ev.key.clone()),
            Kind::MouseMove => self.hover = self.row_at(ev.pos),
            Kind::MouseDown => {
                if let Some(i) = self.row_at(ev.pos) {
                    if let Some(pi) = self.rows[i].page {
                        self.select(pi);
                        self.ensure_selection_visible();
                        self.b.play_sound("click");
                    }
                }
            }
            Kind::Wheel => {
                let step = ev.delta * 40;
                if ev.pos.0 < self.list_rect.right() {
                    self.list_scroll = 0.max((self.list_scroll - step).min(0.max(self.rows_h - self.list_rect.h)));
                } else {
                    let m = self.max_art_scroll();
                    self.art_scroll = 0.max((self.art_scroll - step).min(m));
                }
            }
            _ => {}
        }
    }

    fn draw(&mut self) {
        let p = ui::pal();
        let mut s = self.b.sf();
        let (w, h) = (self.b.width, self.b.height);
        ui::draw_background_ex(&mut s, w, h, false, None);
        let img = ui::fb(22).render(&t("lamawiki.title"), true, p.text);
        s.bl(&img, (16, 12));
        draw::rect_r(&mut s, p.accent, (16, 14 + img.h(), img.w(), 3), 0, 2);
        self.draw_search(&mut s);
        self.draw_list(&mut s);
        self.draw_article(&mut s);
        ui::draw_footer(&mut s, w, h, &t("lamawiki.hint"));
    }
}
