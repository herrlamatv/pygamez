//! progress.rs
//! ===========
//! Der Erfolge-&-Statistik-Screen ("Fortschritt") - uebertragen aus
//! `progress.py`. Fuenf Reiter: Erfolge, Statistiken, Kalender, Verlauf,
//! Profile. Alle Reiter scrollen; die Daten werden einmal in [`build`]
//! gelesen, nie pro Bild.

use std::collections::BTreeMap;

use pgz_core::achievements::{self, Achievement, Category};
use pgz_core::i18n::{t, tf};
use pgz_core::{defs, levels, profiles, stats, store};
use pgz_engine::achv;
use pgz_engine::filepick;
use pgz_engine::game::{Game, GameBase, GameInfo, InputEvent, Kind};
use pgz_engine::pg::{draw, rgb, Color, Font, Rect, RectExt, SurfExt, Surface};
use pgz_engine::shell::{self, Cmd};
use pgz_engine::ui::{self, mix};
use serde_json::Value;

use super::charts;

const ROW_ACH: i32 = 46;
const ROW_GAME: i32 = 30;
const ROW_HEAD: i32 = 30;
const ROW_EVENT: i32 = 28;
const GAP: i32 = 6;
const CATS: [Category; 5] = [Category::General, Category::Milestone, Category::Master, Category::Regular, Category::Special];
const FILTERS: [&str; 3] = ["all", "unlocked", "locked"];
const TABS: [&str; 5] = ["achievements", "stats", "calendar", "timeline", "profiles"];

fn font(size: i32) -> Font {
    ui::font(size, false, false)
}

fn c3(c: (u8, u8, u8)) -> Color {
    rgb(c.0, c.1, c.2)
}

fn rarity_color_of(id: &str) -> Color {
    let hex = defs::RARITY.iter().find(|r| r.0 == id).map(|r| r.2).unwrap_or("#9aa6bf");
    c3(levels::hex_to_rgb(hex))
}

fn rarity_points_of(id: &str) -> i64 {
    defs::RARITY.iter().find(|r| r.0 == id).map(|r| r.1).unwrap_or(10)
}

/// `'2026-09-19'` -> `'19.09.2026'` bzw. heute/gestern.
pub fn pretty_date(day: &str) -> String {
    let Some(date) = charts::parse_day(day) else {
        return day.to_string();
    };
    let today = chrono::Local::now().date_naive();
    if date == today {
        return t("cal.today");
    }
    if (today - date).num_days() == 1 {
        return t("cal.yesterday");
    }
    use chrono::Datelike;
    tf("cal.date", &[("d", &date.day().to_string()), ("m", &date.month().to_string()), ("y", &date.year().to_string())])
}

fn spaced(n: i64) -> String {
    let s = n.abs().to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(' ');
        }
        out.push(ch);
    }
    if n < 0 {
        format!("-{out}")
    } else {
        out
    }
}

#[derive(Clone)]
enum Row {
    Filter,
    Head(String),
    Note(String),
    Ach(&'static Achievement, Option<String>, Option<(i64, i64)>),
    Cards,
    ChartTop(Vec<(usize, stats::GameStats)>),
    ChartSplit,
    TableHead,
    Game(usize, stats::GameStats),
    Wrapped,
    Heat,
    Streaks,
    Weekdays,
    Clock,
    SeriesPick,
    ChartSeries,
    DayHead(String),
    Event(Value),
    Profile(String),
    Board(usize, Vec<(String, i64)>),
    Duel(usize, String, String, i64, i64),
}

#[derive(Clone)]
enum Btn {
    Filter(&'static str),
    Series(String),
    Import,
    Export,
}

#[derive(Default)]
struct Hits {
    top: Option<usize>,
    wd: Option<usize>,
    clock: Option<usize>,
    series: Option<usize>,
    heat: Option<String>,
}

/// Erfolge & Statistik.
pub struct ProgressScreen {
    b: GameBase,
    games: Vec<GameInfo>,
    tab: &'static str,
    filter: &'static str,
    tab_hover: Option<&'static str>,
    scroll: i32,
    hover_row: Option<usize>,
    mouse: (i32, i32),
    hits: Hits,
    tip: Option<Vec<String>>,
    series_game: Option<String>,
    buttons: Vec<(Rect, Btn)>,
    flash: String,
    flash_t: Option<std::time::Instant>,
    left: i32,
    right: i32,
    top: i32,
    bottom: i32,
    tab_font: Font,
    tab_rects: Vec<(Rect, &'static str)>,
    unlocked: BTreeMap<String, String>,
    totals: stats::Totals,
    scores: BTreeMap<String, i64>,
    n_unlocked: usize,
    n_total: usize,
    rows: Vec<(i32, i32, Row)>,
    content_h: i32,
    card_cols: i32,
    card_h: i32,
    days: Vec<(String, stats::DayStats)>,
    day_time: Vec<(String, f64)>,
    streak: stats::Streak,
    facts: Vec<String>,
    played_series: Vec<String>,
    pstats: Vec<(String, stats::Totals)>,
}

impl ProgressScreen {
    /// `ProgressScreen(surface, w, h, app, on_close)`
    pub fn new(b: GameBase, games: Vec<GameInfo>) -> ProgressScreen {
        let f = b.font.clone();
        let mut s = ProgressScreen {
            b,
            games,
            tab: "achievements",
            filter: "all",
            tab_hover: None,
            scroll: 0,
            hover_row: None,
            mouse: (0, 0),
            hits: Hits::default(),
            tip: None,
            series_game: None,
            buttons: Vec::new(),
            flash: String::new(),
            flash_t: None,
            left: 0,
            right: 0,
            top: 0,
            bottom: 0,
            tab_font: f,
            tab_rects: Vec::new(),
            unlocked: BTreeMap::new(),
            totals: stats::Totals::default(),
            scores: BTreeMap::new(),
            n_unlocked: 0,
            n_total: 0,
            rows: Vec::new(),
            content_h: 0,
            card_cols: 3,
            card_h: 64,
            days: Vec::new(),
            day_time: Vec::new(),
            streak: stats::Streak::default(),
            facts: Vec::new(),
            played_series: Vec::new(),
            pstats: Vec::new(),
        };
        s.build();
        s
    }

    /// `set_tab(tab)`
    pub fn set_tab(&mut self, tab: &str) {
        if let Some(k) = TABS.iter().find(|k| **k == tab) {
            self.tab = k;
            self.scroll = 0;
            self.build();
        }
    }

    fn game_idx(&self, key: &str) -> Option<usize> {
        self.games.iter().position(|g| g.key == key)
    }

    fn build(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        self.left = 28.max(w / 2 - 330);
        self.right = w - self.left;
        self.top = 132;
        self.bottom = h - 52;
        self.buttons.clear();
        self.tab_font = ui::fb(13);
        self.tab_rects.clear();
        let mut tx = self.left;
        for key in TABS {
            let tw = self.tab_font.size(&t(&format!("progress.tab_{key}"))).0 + 20;
            self.tab_rects.push((Rect::new(tx, 48, tw, 26), key));
            tx += tw + 6;
        }
        self.unlocked = achievements::unlocked();
        self.totals = stats::totals(None);
        self.scores = pgz_core::highscore::load();
        let (a, m) = achievements::counts();
        self.n_unlocked = a;
        self.n_total = m;
        let mut rows = Vec::new();
        let y = match self.tab {
            "achievements" => self.build_achievements(&mut rows, 0),
            "stats" => self.build_stats(&mut rows, 0),
            "calendar" => self.build_calendar(&mut rows, 0),
            "timeline" => self.build_timeline(&mut rows, 0),
            _ => self.build_profiles(&mut rows, 0),
        };
        self.rows = rows;
        self.content_h = y;
        self.clamp_scroll();
    }

    fn build_achievements(&mut self, rows: &mut Vec<(i32, i32, Row)>, mut y: i32) -> i32 {
        rows.push((y, 26, Row::Filter));
        y += 26 + GAP;
        let mut by_cat: Vec<Vec<&'static Achievement>> = vec![Vec::new(); CATS.len()];
        for d in achievements::all_defs() {
            let ts = self.unlocked.get(&d.id);
            if self.filter == "unlocked" && ts.is_none() {
                continue;
            }
            if self.filter == "locked" && ts.is_some() {
                continue;
            }
            if let Some(ci) = CATS.iter().position(|c| *c == d.cat) {
                by_cat[ci].push(d);
            }
        }
        for (ci, cat) in CATS.iter().enumerate() {
            let list = &by_cat[ci];
            if list.is_empty() {
                continue;
            }
            let got = list.iter().filter(|d| self.unlocked.contains_key(&d.id)).count();
            rows.push((y, ROW_HEAD, Row::Head(format!("{}   {}/{}", t(&format!("progress.cat_{}", cat.key())), got, list.len()))));
            y += ROW_HEAD + GAP;
            for d in list {
                let ts = self.unlocked.get(&d.id).cloned();
                let prog = if ts.is_none() { achievements::progress_of(d) } else { None };
                rows.push((y, ROW_ACH, Row::Ach(d, ts, prog)));
                y += ROW_ACH + GAP;
            }
            y += 6;
        }
        if rows.len() <= 1 {
            rows.push((y, ROW_HEAD, Row::Note(t("progress.filter_empty"))));
            y += ROW_HEAD;
        }
        y
    }

    fn cards_height(&mut self) -> i32 {
        let cols = if self.right - self.left >= 560 { 3 } else { 2 };
        let rows = (8 + cols - 1) / cols;
        self.card_cols = cols;
        self.card_h = 64;
        rows * (self.card_h + 10) + 30
    }

    fn build_stats(&mut self, rows: &mut Vec<(i32, i32, Row)>, mut y: i32) -> i32 {
        let ch = self.cards_height();
        rows.push((y, ch, Row::Cards));
        y += ch + 14;
        let mut entries: Vec<(usize, stats::GameStats)> = Vec::new();
        for (i, g) in self.games.iter().enumerate() {
            let st = stats::get(g.key, None);
            if st.plays > 0 || st.time > 0.0 {
                entries.push((i, st));
            }
        }
        entries.sort_by(|a, b| b.1.time.partial_cmp(&a.1.time).unwrap_or(std::cmp::Ordering::Equal));
        if !entries.is_empty() {
            let top: Vec<_> = entries.iter().take(10).cloned().collect();
            rows.push((y, ROW_HEAD, Row::Head(t("progress.top_games"))));
            y += ROW_HEAD;
            let h = 22 * top.len() as i32 + 10;
            rows.push((y, h, Row::ChartTop(top)));
            y += h + 14;
            rows.push((y, ROW_HEAD, Row::Head(t("progress.balance"))));
            y += ROW_HEAD;
            rows.push((y, 150, Row::ChartSplit));
            y += 150 + 14;
            rows.push((y, ROW_HEAD, Row::TableHead));
            y += ROW_HEAD + 2;
            for (i, st) in &entries {
                rows.push((y, ROW_GAME, Row::Game(*i, st.clone())));
                y += ROW_GAME + 2;
            }
            let rest = self.games.len() as i64 - entries.len() as i64;
            if rest > 0 {
                y += 6;
                rows.push((y, ROW_HEAD, Row::Note(tf("progress.never_played", &[("n", &rest.to_string())]))));
                y += ROW_HEAD;
            }
        } else {
            rows.push((y, ROW_HEAD, Row::Note(t("progress.no_stats"))));
            y += ROW_HEAD;
        }
        y
    }

    fn build_calendar(&mut self, rows: &mut Vec<(i32, i32, Row)>, mut y: i32) -> i32 {
        self.days = stats::days_map(None);
        self.day_time = self.days.iter().map(|(d, v)| (d.clone(), v.t)).collect();
        self.streak = stats::streak(None);
        self.facts = self.build_facts();
        let h = 24 + 20 * self.facts.len() as i32;
        rows.push((y, h, Row::Wrapped));
        y += h + 14;
        rows.push((y, ROW_HEAD, Row::Head(t("progress.calendar_head"))));
        y += ROW_HEAD;
        rows.push((y, 132, Row::Heat));
        y += 132 + 10;
        rows.push((y, 40, Row::Streaks));
        y += 40 + 14;
        rows.push((y, ROW_HEAD, Row::Head(t("progress.weekday_head"))));
        y += ROW_HEAD;
        rows.push((y, 110, Row::Weekdays));
        y += 110 + 14;
        rows.push((y, ROW_HEAD, Row::Head(t("progress.clock_head"))));
        y += ROW_HEAD;
        rows.push((y, 190, Row::Clock));
        y += 190 + 10;
        y
    }

    fn build_facts(&self) -> Vec<String> {
        let mut facts = Vec::new();
        let totals = &self.totals;
        if self.days.is_empty() {
            return facts;
        }
        // Python: max(days.items(), key=t) - bei Gleichstand der erste.
        let mut best = &self.days[0];
        for e in &self.days[1..] {
            if e.1.t > best.1.t {
                best = e;
            }
        }
        if best.1.t > 0.0 {
            facts.push(tf("progress.fact_best_day", &[("day", &pretty_date(&best.0)), ("time", &stats::format_time(best.1.t))]));
        }
        let hours = stats::hours_list(None);
        if hours.iter().any(|h| *h != 0.0) {
            let peak = peak_index(&hours);
            facts.push(tf("progress.fact_peak_hour", &[("h", &peak.to_string()), ("h2", &((peak + 1) % 24).to_string())]));
        }
        if totals.longest != 0.0 {
            facts.push(tf("progress.fact_longest", &[("time", &stats::format_time(totals.longest))]));
        }
        if let Some(fav) = totals.favorite.as_ref().filter(|f| !f.is_empty()) {
            if let Some(i) = self.game_idx(fav) {
                facts.push(tf("progress.fact_favorite", &[("game", &self.games[i].name()), ("time", &stats::format_time(stats::get(fav, None).time))]));
            }
        }
        if let Some((d, _ts)) = achievements::rarest_unlocked(1).first() {
            facts.push(tf("progress.fact_rarest", &[("name", &d.display_name()), ("rarity", &d.rarity_name())]));
        }
        if totals.days != 0 {
            facts.push(tf("progress.fact_days", &[("n", &totals.days.to_string()), ("avg", &stats::format_time(totals.time / 1.max(totals.days) as f64))]));
        }
        facts.truncate(6);
        facts
    }

    fn build_timeline(&mut self, rows: &mut Vec<(i32, i32, Row)>, mut y: i32) -> i32 {
        let played = stats::played_games(None);
        if !self.series_game.as_ref().is_some_and(|g| played.contains(g)) {
            self.series_game = played.first().cloned();
        }
        self.played_series = played.iter().take(8).cloned().collect();
        if self.series_game.is_some() {
            rows.push((y, 26, Row::SeriesPick));
            y += 26 + 6;
            rows.push((y, 140, Row::ChartSeries));
            y += 140 + 14;
        }
        rows.push((y, ROW_HEAD, Row::Head(t("progress.timeline_head"))));
        y += ROW_HEAD + 2;
        let events = stats::events(Some(400), None, None);
        if events.is_empty() {
            rows.push((y, ROW_HEAD, Row::Note(t("progress.no_events"))));
            return y + ROW_HEAD;
        }
        let mut last_day: Option<String> = None;
        for ev in events {
            let stamp = ev_str(&ev, "t");
            let day: String = stamp.chars().take(10).collect();
            if last_day.as_deref() != Some(day.as_str()) {
                last_day = Some(day.clone());
                rows.push((y, 24, Row::DayHead(pretty_date(&day))));
                y += 24;
            }
            rows.push((y, ROW_EVENT, Row::Event(ev)));
            y += ROW_EVENT + 2;
        }
        y
    }

    fn build_profiles(&mut self, rows: &mut Vec<(i32, i32, Row)>, mut y: i32) -> i32 {
        let plist: Vec<String> = profiles::all().into_iter().map(|(pid, _)| pid).collect();
        self.pstats = plist.iter().map(|pid| (pid.clone(), stats::totals(Some(pid)))).collect();
        rows.push((y, ROW_HEAD, Row::Head(t("progress.profiles_head"))));
        y += ROW_HEAD + 2;
        for pid in &plist {
            rows.push((y, 54, Row::Profile(pid.clone())));
            y += 54 + GAP;
        }
        y += 8;
        let board = self.build_board(&plist);
        if !board.is_empty() {
            rows.push((y, ROW_HEAD, Row::Head(t("progress.board_head"))));
            y += ROW_HEAD + 2;
            for (i, values) in board {
                rows.push((y, ROW_GAME, Row::Board(i, values)));
                y += ROW_GAME + 2;
            }
            y += 8;
        }
        let duels = self.build_duels();
        if !duels.is_empty() {
            rows.push((y, ROW_HEAD, Row::Head(t("progress.duel_head"))));
            y += ROW_HEAD + 2;
            for (i, me, other, w, l) in duels {
                rows.push((y, ROW_GAME, Row::Duel(i, me, other, w, l)));
                y += ROW_GAME + 2;
            }
        }
        y
    }

    fn build_board(&self, plist: &[String]) -> Vec<(usize, Vec<(String, i64)>)> {
        let per: Vec<store::Section> = plist.iter().map(|pid| store::load_section_of("highscores", Some(pid))).collect();
        let mut out = Vec::new();
        for (gi, g) in self.games.iter().enumerate() {
            let mut values = Vec::new();
            for (pi, pid) in plist.iter().enumerate() {
                let score = match per[pi].get(g.key) {
                    Some(Value::Number(n)) => n.as_i64().unwrap_or_else(|| n.as_f64().unwrap_or(0.0) as i64),
                    Some(Value::String(s)) => s.trim().parse().unwrap_or(0),
                    Some(Value::Bool(b)) => *b as i64,
                    _ => 0,
                };
                if score > 0 {
                    values.push((pid.clone(), score));
                }
            }
            if !values.is_empty() {
                values.sort_by_key(|v| -v.1);
                out.push((gi, values));
            }
        }
        out.into_iter().filter(|e| e.1.len() > 1).collect()
    }

    fn build_duels(&self) -> Vec<(usize, String, String, i64, i64)> {
        let me = profiles::active_id();
        let mut out = Vec::new();
        for (other, games) in stats::duel_table(Some(&me)) {
            for (key, w, l) in games {
                if let Some(i) = self.game_idx(&key) {
                    out.push((i, me.clone(), other.clone(), w, l));
                }
            }
        }
        out.sort_by_key(|e| -(e.3 + e.4));
        out
    }

    fn viewport(&self) -> i32 {
        self.bottom - self.top
    }

    fn clamp_scroll(&mut self) {
        self.scroll = 0.max(self.scroll.min(0.max(self.content_h - self.viewport())));
    }

    // ----- Eingabe ------------------------------------------------------

    fn switch_tab(&mut self, key: &'static str) {
        if key == self.tab {
            return;
        }
        self.tab = key;
        self.scroll = 0;
        self.hover_row = None;
        self.build();
        self.b.play_sound("move");
        if key == "calendar" || key == "timeline" {
            achievements::event("statistician", None);
        }
        if key == "calendar" {
            achievements::event("wrapped", None);
        }
    }

    fn tab_step(&mut self, d: i64) {
        let idx = TABS.iter().position(|k| *k == self.tab).unwrap_or(0) as i64;
        self.switch_tab(TABS[(idx + d).rem_euclid(TABS.len() as i64) as usize]);
    }

    fn row_at(&self, pos: (i32, i32)) -> Option<usize> {
        let (x, y) = pos;
        if !(self.left <= x && x <= self.right && self.top <= y && y <= self.bottom) {
            return None;
        }
        let cy = y - self.top + self.scroll;
        self.rows.iter().position(|(ry, rh, kind)| *ry <= cy && cy <= ry + rh && matches!(kind, Row::Ach(..) | Row::Game(..) | Row::Board(..)))
    }

    fn row_game(&self, row: &Row) -> Option<usize> {
        match row {
            Row::Game(i, _) | Row::Board(i, _) => Some(*i),
            Row::Ach(d, _, _) if !d.game.is_empty() => self.game_idx(&d.game),
            _ => None,
        }
    }

    fn export(&mut self) {
        if !filepick::available() {
            return;
        }
        let name = format!("pygamez-stats{}", stats::EXT);
        let Some(path) = filepick::save_as(&name, &t("progress.export"), &[stats::EXT, ".json"]) else {
            return;
        };
        let ok = stats::export_to(std::path::Path::new(&path));
        self.set_flash(if ok { t("progress.export_ok") } else { t("progress.export_err") });
    }

    fn import(&mut self) {
        if !filepick::available() {
            return;
        }
        let Some(path) = filepick::open_file(&t("progress.import"), &[stats::EXT, ".json"]) else {
            return;
        };
        let (ok, reason) = stats::import_from(std::path::Path::new(&path), None, false);
        self.set_flash(if ok { t("progress.import_ok") } else { tf("progress.import_err", &[("reason", reason)]) });
        if ok {
            self.build();
        }
    }

    fn set_flash(&mut self, text: String) {
        self.flash = text;
        self.flash_t = Some(std::time::Instant::now());
    }

    // ----- Zeichnen -----------------------------------------------------

    fn draw_level_head(&self, s: &mut Surface, w: i32) {
        let p = ui::pal();
        let (_level, points) = stats::level_of(None);
        let (lvl, in_level, need) = levels::progress(points);
        let color = c3(levels::rank_color(lvl));
        let y = 84;
        achv::draw_badge(s, (self.left + 16, y + 12), 15, "bolt", color, false);
        let name = ui::fb(15).render(&format!("{} {}  \u{b7}  {}", t("level.short"), lvl, levels::rank_title(lvl)), true, p.text);
        s.bl(&name, (self.left + 38, y));
        let sub = font(11).render(
            &format!(
                "{}   \u{b7}   {}",
                tf("progress.points", &[("n", &points.to_string())]),
                tf("progress.next_level", &[("n", &0.max(need - in_level).to_string()), ("m", &(lvl + 1).to_string())])
            ),
            true,
            p.text_dim,
        );
        s.bl(&sub, (self.left + 38, y + name.h() + 1));
        let bar_w = 190.min(w / 4);
        let bx = self.right - bar_w;
        draw::rect_r(s, p.panel_light, (bx, y + 4, bar_w, 7), 0, 4);
        let fw = (bar_w as f64 * 1f64.min(in_level as f64 / 1.max(need) as f64)) as i32;
        if fw > 0 {
            draw::rect_r(s, color, (bx, y + 4, fw, 7), 0, 4);
        }
        draw::rect_r(s, p.border, (bx, y + 4, bar_w, 7), 1, 4);
        let head = ui::fb(11).render(&tf("progress.unlocked_of", &[("n", &self.n_unlocked.to_string()), ("m", &self.n_total.to_string())]), true, p.gold);
        s.blr(&head, head.get_rect().topright_at((self.right, y + 15)));
    }

    fn draw_row(&mut self, s: &mut Surface, y: i32, rh: i32, row: &Row, i: usize) {
        let p = ui::pal();
        match row {
            Row::Head(text) => {
                let img = ui::fb(13).render(text, true, p.accent);
                s.bl(&img, (self.left, y + 8));
                draw::line(s, p.border, (self.left + img.w() + 12, y + 16), (self.right, y + 16), 1);
            }
            Row::DayHead(text) => {
                let img = ui::fb(11).render(text, true, p.text_faint);
                s.bl(&img, (self.left, y + 6));
            }
            Row::Note(text) => {
                let img = font(13).render(text, true, p.text_faint);
                s.bl(&img, (self.left, y + 8));
            }
            Row::Filter => self.draw_filter(s, y),
            Row::Ach(d, ts, prog) => self.draw_ach(s, y, rh, d, ts.as_deref(), *prog, i),
            Row::Cards => self.draw_cards(s, y),
            Row::ChartTop(data) => self.draw_chart_top(s, y, rh, data),
            Row::ChartSplit => self.draw_chart_split(s, y, rh),
            Row::TableHead => self.draw_table_head(s, y),
            Row::Game(gi, st) => self.draw_game(s, y, rh, *gi, st, i),
            Row::Wrapped => self.draw_wrapped(s, y, rh),
            Row::Heat => self.draw_heat(s, y, rh),
            Row::Streaks => self.draw_streaks(s, y, rh),
            Row::Weekdays => self.draw_weekdays(s, y, rh),
            Row::Clock => self.draw_clock(s, y, rh),
            Row::SeriesPick => self.draw_series_pick(s, y),
            Row::ChartSeries => self.draw_chart_series(s, y, rh),
            Row::Event(ev) => self.draw_event(s, y, rh, ev),
            Row::Profile(pid) => self.draw_profile(s, y, rh, pid),
            Row::Board(gi, values) => self.draw_board(s, y, rh, *gi, values, i),
            Row::Duel(gi, me, other, w, l) => self.draw_duel(s, y, rh, *gi, me, other, *w, *l),
        }
    }

    fn draw_filter(&mut self, s: &mut Surface, y: i32) {
        let p = ui::pal();
        let mut x = self.left;
        let fnt = ui::fb(11);
        for key in FILTERS {
            let label = t(&format!("progress.filter_{key}"));
            let w = fnt.size(&label).0 + 18;
            let r = Rect::new(x, y, w, 22);
            let active = self.filter == key;
            draw::rect_r(s, if active { p.panel_light } else { p.panel }, r, 0, 11);
            draw::rect_r(s, if active { p.accent } else { p.border }, r, 1, 11);
            let img = fnt.render(&label, true, if active { p.text } else { p.text_dim });
            s.blr(&img, img.get_rect().center_at(r.center()));
            self.buttons.push((r, Btn::Filter(key)));
            x += w + 6;
        }
        let counts = achievements::rarity_counts();
        let mut x = self.right;
        for (rarity, got, total) in counts.iter().rev() {
            if *total == 0 {
                continue;
            }
            let label = format!("{got}/{total}");
            let col = rarity_color_of(rarity);
            let img = ui::fb(10).render(&label, true, col);
            x -= img.w() + 14;
            draw::circle(s, col, (x - 6, y + 11), 4, 0);
            s.bl(&img, (x, y + 11 - img.h() / 2));
            let bx = Rect::new(x - 12, y, img.w() + 14, 22);
            if bx.hit(self.mouse) {
                self.tip = Some(vec![t(&format!("rarity.{rarity}")), tf("progress.rarity_points", &[("n", &rarity_points_of(rarity).to_string())])]);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_ach(&mut self, s: &mut Surface, y: i32, rh: i32, d: &Achievement, ts: Option<&str>, prog: Option<(i64, i64)>, i: usize) {
        let p = ui::pal();
        let unlocked = ts.is_some();
        let row = Rect::new(self.left, y, self.right - self.left, rh);
        let hover = Some(i) == self.hover_row;
        let has_game = !d.game.is_empty() && self.game_idx(&d.game).is_some();
        ui::draw_panel_ex(s, row, Some(if hover && has_game { p.panel_light } else { p.panel }), None, Some(10), false, None);
        let rarity_col = c3(d.rarity_color());
        if unlocked {
            draw::rect_r(s, rarity_col, (row.x, row.y + 6, 3, rh - 12), 0, 2);
        }
        let accent = if unlocked { rarity_col } else { p.border_light };
        achv::draw_badge(s, (row.x + 26, row.centery()), 15, &d.icon, accent, !unlocked);
        let name_col = if unlocked { p.text } else { p.text_dim };
        let desc_col = if unlocked { p.text_dim } else { p.text_faint };
        let name = ui::fb(15).render(&d.display_name(), true, name_col);
        let desc = font(12).render(&d.display_desc(), true, desc_col);
        let tx = row.x + 50;
        let max_w = row.w - 50 - 150;
        s.bl_area(&name, (tx, row.y + 5), (0, 0, max_w, name.h()));
        s.bl_area(&desc, (tx, row.y + 8 + name.h()), (0, 0, max_w, desc.h()));
        let pts = ui::fb(11).render(&format!("+{}", d.points()), true, if unlocked { rarity_col } else { p.text_faint });
        s.blr(&pts, pts.get_rect().topright_at((row.right() - 14, row.y + 6)));
        if let Some(ts) = ts {
            let day: String = ts.chars().take(10).collect();
            let date = font(11).render(&day, true, p.text_faint);
            s.blr(&date, date.get_rect().bottomright_at((row.right() - 14, row.bottom() - 6)));
        } else if let Some((cur, target)) = prog {
            let bw = 86;
            let bx = row.right() - 14 - bw;
            let byy = row.bottom() - 14;
            let txt = if d.stat == "time" {
                format!("{} / {}", stats::format_time(cur as f64), stats::format_time(target as f64))
            } else {
                format!("{cur} / {target}")
            };
            let label = font(11).render(&txt, true, p.text_faint);
            s.blr(&label, label.get_rect().midright_at((row.right() - 14, byy - 8)));
            draw::rect_r(s, p.btn, (bx, byy, bw, 5), 0, 3);
            let fw = (bw as f64 * cur as f64 / 1.max(target) as f64) as i32;
            if fw > 0 {
                draw::rect_r(s, p.accent_soft, (bx, byy, fw, 5), 0, 3);
            }
        }
        if hover && row.hit(self.mouse) {
            self.tip = Some(vec![d.display_name(), format!("{}  \u{b7}  +{}", d.rarity_name(), d.points())]);
        }
    }

    fn draw_cards(&self, s: &mut Surface, y: i32) {
        let p = ui::pal();
        let tot = &self.totals;
        let cards = [
            ("clock", stats::format_time(tot.time), t("progress.total_time")),
            ("flag", tot.plays.to_string(), t("progress.total_plays")),
            ("medal", tot.wins.to_string(), t("progress.total_wins")),
            ("trophy", tot.records.to_string(), t("progress.total_records")),
            ("gem", format!("{} / {}", tot.distinct, self.games.len()), t("progress.distinct")),
            ("star", format!("{} / {}", self.n_unlocked, self.n_total), t("progress.achievements")),
            ("calendar", tot.days.to_string(), t("progress.total_days")),
            ("bolt", tot.points.to_string(), t("progress.total_points")),
        ];
        let cols = self.card_cols;
        let gap = 10;
        let cw = pgz_engine::py::fdiv32(self.right - self.left - (cols - 1) * gap, cols);
        for (i, (icon, value, label)) in cards.iter().enumerate() {
            let i = i as i32;
            let cx = self.left + (i % cols) * (cw + gap);
            let cy = y + (i / cols) * (self.card_h + gap);
            let r = Rect::new(cx, cy, cw, self.card_h);
            ui::draw_panel_ex(s, r, None, None, Some(10), false, None);
            achv::draw_badge(s, (r.x + 24, r.centery()), 13, icon, p.accent, false);
            let val = ui::fb(19).render(value, true, p.text);
            let lab = font(11).render(label, true, p.text_dim);
            s.bl(&val, (r.x + 46, r.y + 10));
            s.bl_area(&lab, (r.x + 46, r.y + 14 + val.h()), (0, 0, r.w - 56, lab.h()));
        }
        let rows = (cards.len() as i32 + cols - 1) / cols;
        let fy = y + rows * (self.card_h + gap) + 4;
        if let Some(fav) = tot.favorite.as_ref().filter(|f| !f.is_empty()) {
            if let Some(gi) = self.game_idx(fav) {
                let g = &self.games[gi];
                let col = ui::game_color(g.class_name);
                let img1 = font(13).render(&format!("{}:", t("progress.favorite")), true, p.text_dim);
                let img2 = ui::fb(13).render(&format!(" {} ({})", g.name(), stats::format_time(stats::get(fav, None).time)), true, col);
                s.bl(&img1, (self.left, fy));
                s.bl(&img2, (self.left + img1.w(), fy));
            }
        }
    }

    fn track(&mut self, hits: &[(Rect, usize)], tip: impl Fn(usize) -> Vec<String>) -> Option<usize> {
        for (r, idx) in hits {
            if r.hit(self.mouse) {
                let lines = tip(*idx);
                if !lines.is_empty() {
                    self.tip = Some(lines);
                }
                return Some(*idx);
            }
        }
        None
    }

    fn draw_chart_top(&mut self, s: &mut Surface, y: i32, rh: i32, data: &[(usize, stats::GameStats)]) {
        let r = Rect::new(self.left, y + 4, self.right - self.left, rh - 8);
        let values: Vec<f64> = data.iter().map(|(_, st)| st.time).collect();
        let labels: Vec<String> = data.iter().map(|(gi, _)| self.games[*gi].name()).collect();
        let colors: Vec<Color> = data.iter().map(|(gi, _)| ui::game_color(self.games[*gi].class_name)).collect();
        let fmt = stats::format_time;
        let hits = charts::bars(s, r, &values, Some(&labels), Some(&colors), None, true, Some(&fmt), self.hits.top);
        let plays: Vec<i64> = data.iter().map(|(_, st)| st.plays).collect();
        self.hits.top = self.track(&hits, |idx| {
            vec![
                labels[idx].clone(),
                format!("{}: {}", t("progress.col_time"), stats::format_time(values[idx])),
                format!("{}: {}", t("progress.col_plays"), plays[idx]),
            ]
        });
    }

    fn draw_chart_split(&self, s: &mut Surface, y: i32, rh: i32) {
        let p = ui::pal();
        let tot = &self.totals;
        let (wins, losses) = (tot.wins, tot.losses);
        let cx = self.left + 80;
        let cy = y + rh / 2;
        let ratio = if wins + losses != 0 { 100.0 * wins as f64 / (wins + losses) as f64 } else { 0.0 };
        charts::donut(
            s,
            (cx, cy),
            60,
            &[(wins as f64, p.green), (losses as f64, mix(p.panel_light, p.red, 0.8))],
            Some(&format!("{}%", pgz_engine::py::round(ratio))),
            Some(&t("progress.win_ratio")),
        );
        let fnt = font(12);
        let tx = cx + 84;
        let rows = [
            (t("progress.total_wins"), wins.to_string()),
            (t("progress.total_losses"), losses.to_string()),
            (t("progress.total_records"), tot.records.to_string()),
            (t("progress.longest_session"), stats::format_time(tot.longest)),
            (t("progress.duels"), tot.duels.to_string()),
        ];
        let mut ry = y + 14;
        for (label, value) in &rows {
            s.bl(&fnt.render(label, true, p.text_dim), (tx, ry));
            let img = ui::fb(12).render(value, true, p.text);
            s.blr(&img, img.get_rect().topright_at((self.right - 8, ry)));
            ry += 24;
        }
    }

    fn table_columns(&self) -> (Vec<i32>, i32) {
        let w = self.right - self.left;
        let col_w = 64.max(92.min(w / 7));
        ((0..4).map(|i| self.right - 10 - i * col_w).collect(), col_w)
    }

    fn draw_table_head(&self, s: &mut Surface, y: i32) {
        let p = ui::pal();
        let (xs, _cw) = self.table_columns();
        let fnt = ui::fb(11);
        let img = fnt.render(&t("progress.col_game"), true, p.text_faint);
        s.bl(&img, (self.left + 8, y + 10));
        for (x, key) in xs.iter().zip(["col_best", "col_wins", "col_time", "col_plays"]) {
            let img = fnt.render(&t(&format!("progress.{key}")), true, p.text_faint);
            s.blr(&img, img.get_rect().topright_at((*x, y + 10)));
        }
        draw::line(s, p.border, (self.left, y + 26), (self.right, y + 26), 1);
    }

    fn draw_game(&self, s: &mut Surface, y: i32, rh: i32, gi: usize, st: &stats::GameStats, i: usize) {
        let p = ui::pal();
        let g = &self.games[gi];
        let row = Rect::new(self.left, y, self.right - self.left, rh);
        if Some(i) == self.hover_row {
            ui::draw_panel_ex(s, row, Some(p.panel_light), None, Some(8), false, None);
        }
        let color = ui::game_color(g.class_name);
        draw::circle(s, color, (row.x + 12, row.centery()), 4, 0);
        let (xs, cw) = self.table_columns();
        let name_w = xs[xs.len() - 1] - cw + 10 - (row.x + 24);
        let name = ui::fb(13).render(&g.name(), true, p.text);
        s.bl_area(&name, (row.x + 24, row.centery() - name.h() / 2), (0, 0, 40.max(name_w), name.h()));
        let fnt = font(12);
        let values = [
            self.scores.get(g.key).copied().unwrap_or(0).to_string(),
            if st.wins != 0 || st.losses != 0 { st.wins.to_string() } else { "-".into() },
            stats::format_time(st.time),
            st.plays.to_string(),
        ];
        for (x, val) in xs.iter().zip(values.iter()) {
            let img = fnt.render(val, true, p.text_dim);
            s.blr(&img, img.get_rect().midright_at((*x, row.centery())));
        }
    }

    fn draw_wrapped(&mut self, s: &mut Surface, y: i32, rh: i32) {
        let p = ui::pal();
        let r = Rect::new(self.left, y, self.right - self.left, rh);
        ui::draw_panel_ex(s, r, Some(p.panel_light), None, Some(12), true, Some(p.gold));
        let head = ui::fb(14).render(&t("progress.wrapped_head"), true, p.gold);
        s.bl(&head, (r.x + 14, r.y + 6));
        let fnt = font(12);
        let mut ty = r.y + 8 + head.h();
        for fact in &self.facts {
            draw::circle(s, p.accent, (r.x + 20, ty + 8), 3, 0);
            let img = fnt.render(fact, true, p.text_dim);
            s.bl_area(&img, (r.x + 30, ty), (0, 0, r.w - 44, img.h()));
            ty += 20;
        }
        if filepick::available() {
            let fnt = ui::fb(11);
            let mut x = r.right() - 10;
            for (label, cb) in [(t("progress.import"), Btn::Import), (t("progress.export"), Btn::Export)] {
                let w = fnt.size(&label).0 + 20;
                let br = Rect::new(x - w, r.y + 8, w, 22);
                draw::rect_r(s, p.panel, br, 0, 11);
                draw::rect_r(s, p.border_light, br, 1, 11);
                let img = fnt.render(&label, true, p.text_dim);
                s.blr(&img, img.get_rect().center_at(br.center()));
                self.buttons.push((br, cb));
                x -= w + 6;
            }
        }
    }

    fn draw_heat(&mut self, s: &mut Surface, y: i32, rh: i32) {
        let p = ui::pal();
        let r = Rect::new(self.left, y, self.right - self.left, rh - 20);
        let hits = charts::calendar_heat(s, r, &self.day_time, Some(p.accent), self.hits.heat.as_deref());
        for (bx, (day, value)) in &hits {
            if bx.hit(self.mouse) {
                self.hits.heat = Some(day.clone());
                let plays = self.days.iter().find(|e| e.0 == *day).map(|e| e.1.p).unwrap_or(0);
                self.tip = Some(vec![
                    pretty_date(day),
                    tf("progress.tip_time", &[("time", &stats::format_time(*value))]),
                    tf("progress.tip_plays", &[("n", &plays.to_string())]),
                ]);
                break;
            }
        }
        charts::heat_legend(s, Rect::new(r.x, r.bottom() + 4, r.w, 14), Some(p.accent), &t("progress.less"), &t("progress.more"));
    }

    fn draw_streaks(&self, s: &mut Surface, y: i32, rh: i32) {
        let p = ui::pal();
        let cards = [
            (self.streak.cur.to_string(), t("progress.streak_cur"), "flame"),
            (self.streak.best.to_string(), t("progress.streak_best"), "flame"),
            (self.totals.days.to_string(), t("progress.total_days"), "calendar"),
        ];
        let gap = 10;
        let n = cards.len() as i32;
        let cw = pgz_engine::py::fdiv32(self.right - self.left - gap * (n - 1), n);
        for (i, (value, label, icon)) in cards.iter().enumerate() {
            let r = Rect::new(self.left + i as i32 * (cw + gap), y, cw, rh);
            ui::draw_panel_ex(s, r, None, None, Some(10), false, None);
            achv::draw_badge(s, (r.x + 20, r.centery()), 11, icon, if i == 1 { p.gold } else { p.accent }, false);
            let img = ui::fb(15).render(value, true, p.text);
            s.bl(&img, (r.x + 38, r.centery() - img.h() + 2));
            let lab = font(10).render(label, true, p.text_dim);
            s.bl_area(&lab, (r.x + 38, r.centery() + 2), (0, 0, r.w - 46, lab.h()));
        }
    }

    fn draw_weekdays(&mut self, s: &mut Surface, y: i32, rh: i32) {
        let p = ui::pal();
        let r = Rect::new(self.left, y, self.right - self.left, rh);
        let values: Vec<f64> = stats::weekday_list(None).to_vec();
        let labels = charts::weekday_names();
        let fmt = stats::format_time;
        let hits = charts::bars(s, r, &values, Some(&labels), None, Some(p.accent), false, Some(&fmt), self.hits.wd);
        self.hits.wd = self.track(&hits, |idx| vec![labels.get(idx).cloned().unwrap_or_default(), stats::format_time(values[idx])]);
    }

    fn draw_clock(&mut self, s: &mut Surface, y: i32, rh: i32) {
        let p = ui::pal();
        let hours = stats::hours_list(None);
        let cx = self.left + (self.right - self.left) / 2;
        let cy = y + rh / 2;
        let hits = charts::clock_ring(s, (cx, cy), 80.min(rh / 2 - 14), &hours, Some(p.accent), self.hits.clock);
        self.hits.clock = self.track(&hits, |idx| {
            vec![tf("progress.tip_hour", &[("h", &idx.to_string()), ("h2", &((idx + 1) % 24).to_string())]), stats::format_time(hours[idx])]
        });
        if hours.iter().any(|h| *h != 0.0) {
            let peak = peak_index(&hours);
            let img = font(11).render(&tf("progress.fact_peak_hour", &[("h", &peak.to_string()), ("h2", &((peak + 1) % 24).to_string())]), true, p.text_dim);
            s.blr(&img, img.get_rect().midtop_at((cx, y + rh - 16)));
        }
    }

    fn draw_series_pick(&mut self, s: &mut Surface, y: i32) {
        let p = ui::pal();
        let fnt = ui::fb(11);
        let mut x = self.left;
        let label = font(11).render(&format!("{}:", t("progress.pick_game")), true, p.text_faint);
        s.bl(&label, (x, y + 5));
        x += label.w() + 8;
        for key in self.played_series.clone() {
            let Some(gi) = self.game_idx(&key) else { continue };
            let g = &self.games[gi];
            let name = g.name();
            let w = fnt.size(&name).0 + 16;
            if x + w > self.right {
                break;
            }
            let r = Rect::new(x, y, w, 22);
            let active = self.series_game.as_deref() == Some(key.as_str());
            let col = ui::game_color(g.class_name);
            draw::rect_r(s, if active { p.panel_light } else { p.panel }, r, 0, 11);
            draw::rect_r(s, if active { col } else { p.border }, r, 1, 11);
            let img = fnt.render(&name, true, if active { p.text } else { p.text_dim });
            s.blr(&img, img.get_rect().center_at(r.center()));
            self.buttons.push((r, Btn::Series(key.clone())));
            x += w + 6;
        }
    }

    fn draw_chart_series(&mut self, s: &mut Surface, y: i32, rh: i32) {
        let p = ui::pal();
        let Some(key) = self.series_game.clone() else { return };
        let series = stats::series_of(&key, None);
        if series.is_empty() {
            return;
        }
        let color = self.game_idx(&key).map(|gi| ui::game_color(self.games[gi].class_name)).unwrap_or(p.accent);
        let r = Rect::new(self.left, y + 4, self.right - self.left, rh - 8);
        let values: Vec<f64> = series.iter().map(|e| e.1 as f64).collect();
        let marks: Vec<usize> = series.iter().enumerate().filter(|(_, e)| e.2 != 0).map(|(i, _)| i).collect();
        let fmt = |v: f64| (v as i64).to_string();
        let hits = charts::line(s, r, &values, Some(color), &marks, Some(&fmt), self.hits.series);
        self.hits.series = self.track(&hits, |idx| {
            let e = &series[idx];
            let day: String = e.0.chars().take(10).collect();
            let hm: String = e.0.chars().skip(11).take(5).collect();
            vec![e.1.to_string(), format!("{} {}", pretty_date(&day), hm), if e.2 != 0 { t("progress.new_record") } else { String::new() }]
        });
    }

    fn draw_event(&self, s: &mut Surface, y: i32, rh: i32, ev: &Value) {
        let p = ui::pal();
        let row = Rect::new(self.left, y, self.right - self.left, rh);
        let kind = ev_str(ev, "k");
        let stamp: String = ev_str(ev, "t").chars().skip(11).take(5).collect();
        let (mut icon, mut color, mut text, mut extra) = ("star".to_string(), p.accent, String::new(), String::new());
        match kind.as_str() {
            "ach" => {
                let id = ev_str(ev, "id");
                if let Some(d) = achievements::by_id(&id) {
                    icon = d.icon.clone();
                    color = c3(d.rarity_color());
                    text = tf("progress.ev_ach", &[("name", &d.display_name())]);
                    extra = format!("+{}", d.points());
                } else {
                    text = tf("progress.ev_ach", &[("name", &id)]);
                }
            }
            "rec" => {
                let g = ev_str(ev, "g");
                icon = "trophy".into();
                color = p.gold;
                let gname = self.game_idx(&g).map(|i| self.games[i].name()).unwrap_or(g);
                text = tf("progress.ev_rec", &[("game", &gname), ("n", &ev_num(ev, "v").to_string())]);
            }
            "lvl" => {
                icon = "bolt".into();
                let v = ev.get("v").map(|x| ev_num_v(x)).unwrap_or(1);
                color = c3(levels::rank_color(v));
                text = tf("progress.ev_lvl", &[("n", &ev_num(ev, "v").to_string()), ("rank", &levels::rank_title(v))]);
            }
            "duel" => {
                let g = ev_str(ev, "g");
                icon = "swords".into();
                color = p.accent2;
                let other = ev_str(ev, "o");
                let gname = self.game_idx(&g).map(|i| self.games[i].name()).unwrap_or(g);
                let won = ev.get("w").is_some_and(|w| match w {
                    Value::Bool(b) => *b,
                    Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
                    Value::Null => false,
                    Value::String(s) => !s.is_empty(),
                    _ => true,
                });
                text = tf(
                    "progress.ev_duel",
                    &[
                        ("game", &gname),
                        ("name", &if !other.is_empty() { profiles::name_of(Some(&other)) } else { "?".into() }),
                        ("result", &if won { t("progress.ev_won") } else { t("progress.ev_lost") }),
                    ],
                );
            }
            _ => {}
        }
        achv::draw_badge(s, (row.x + 14, row.centery()), 10, &icon, color, false);
        let img = font(12).render(&text, true, p.text_dim);
        s.bl_area(&img, (row.x + 32, row.centery() - img.h() / 2), (0, 0, row.w - 120, img.h()));
        if !extra.is_empty() {
            let pts = ui::fb(11).render(&extra, true, color);
            s.blr(&pts, pts.get_rect().midright_at((row.right() - 54, row.centery())));
        }
        let tm = font(11).render(&stamp, true, p.text_faint);
        s.blr(&tm, tm.get_rect().midright_at((row.right() - 8, row.centery())));
    }

    fn draw_profile(&self, s: &mut Surface, y: i32, rh: i32, pid: &str) {
        let p = ui::pal();
        let row = Rect::new(self.left, y, self.right - self.left, rh);
        let active = pid == profiles::active_id();
        ui::draw_panel_ex(s, row, Some(if active { p.panel_light } else { p.panel }), None, Some(10), false, None);
        let color = c3(profiles::color_rgb(Some(pid)));
        achv::draw_badge(s, (row.x + 28, row.centery()), 17, &profiles::icon_of(Some(pid)), color, false);
        let name = ui::fb(15).render(&profiles::name_of(Some(pid)), true, p.text);
        s.bl(&name, (row.x + 54, row.y + 8));
        let tot = self.pstats.iter().find(|e| e.0 == pid).map(|e| e.1.clone()).unwrap_or_default();
        let level = if self.pstats.iter().any(|e| e.0 == pid) { tot.level } else { 1 };
        let sub = font(11).render(
            &format!(
                "{} {} \u{b7} {} \u{b7} {} \u{b7} {}",
                t("level.short"),
                level,
                levels::rank_title(level),
                tf("progress.points", &[("n", &tot.points.to_string())]),
                stats::format_time(tot.time)
            ),
            true,
            p.text_dim,
        );
        s.bl_area(&sub, (row.x + 54, row.y + 10 + name.h()), (0, 0, row.w - 70, sub.h()));
        if active {
            let chip = ui::fb(10).render(&t("profile.active"), true, p.gold);
            s.blr(&chip, chip.get_rect().topright_at((row.right() - 12, row.y + 8)));
        }
    }

    fn draw_board(&self, s: &mut Surface, y: i32, rh: i32, gi: usize, values: &[(String, i64)], i: usize) {
        let p = ui::pal();
        let g = &self.games[gi];
        let row = Rect::new(self.left, y, self.right - self.left, rh);
        if Some(i) == self.hover_row {
            ui::draw_panel_ex(s, row, Some(p.panel_light), None, Some(8), false, None);
        }
        draw::circle(s, ui::game_color(g.class_name), (row.x + 12, row.centery()), 4, 0);
        let name = ui::fb(13).render(&g.name(), true, p.text);
        s.bl_area(&name, (row.x + 24, row.centery() - name.h() / 2), (0, 0, 150, name.h()));
        let mut x = row.x + 190;
        for (rank_i, (pid, score)) in values.iter().take(3).enumerate() {
            let col = c3(profiles::color_rgb(Some(pid)));
            let label = format!("{} {}", profiles::name_of(Some(pid)), spaced(*score));
            let img = ui::font(11, rank_i == 0, false).render(&label, true, if rank_i == 0 { p.gold } else { p.text_dim });
            draw::circle(s, col, (x, row.centery()), 3, 0);
            s.bl(&img, (x + 8, row.centery() - img.h() / 2));
            x += img.w() + 24;
            if x > row.right() - 40 {
                break;
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_duel(&self, s: &mut Surface, y: i32, rh: i32, gi: usize, me: &str, other: &str, wins: i64, losses: i64) {
        let p = ui::pal();
        let g = &self.games[gi];
        let row = Rect::new(self.left, y, self.right - self.left, rh);
        draw::circle(s, ui::game_color(g.class_name), (row.x + 12, row.centery()), 4, 0);
        let name = ui::fb(13).render(&g.name(), true, p.text);
        s.bl_area(&name, (row.x + 24, row.centery() - name.h() / 2), (0, 0, 150, name.h()));
        let label = format!("{}  {} : {}  {}", profiles::name_of(Some(me)), wins, losses, profiles::name_of(Some(other)));
        let col = if wins > losses { p.gold } else { p.text_dim };
        let img = ui::fb(12).render(&label, true, col);
        s.blr(&img, img.get_rect().midright_at((row.right() - 10, row.centery())));
    }
}

fn peak_index(hours: &[f64]) -> usize {
    let mut best = 0;
    for (i, h) in hours.iter().enumerate() {
        if *h > hours[best] {
            best = i;
        }
    }
    best
}

fn ev_str(ev: &Value, key: &str) -> String {
    match ev.get(key) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Null) | None => String::new(),
        Some(v) => v.to_string(),
    }
}

fn ev_num_v(v: &Value) -> i64 {
    match v {
        Value::Number(n) => n.as_i64().unwrap_or_else(|| n.as_f64().unwrap_or(0.0) as i64),
        Value::String(s) => s.trim().parse().unwrap_or(0),
        Value::Bool(b) => *b as i64,
        _ => 0,
    }
}

fn ev_num(ev: &Value, key: &str) -> i64 {
    ev.get(key).map(ev_num_v).unwrap_or(0)
}

impl Game for ProgressScreen {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }
    fn reset(&mut self) {}
    fn update(&mut self, _dt: f64) {}
    fn screen_name(&self) -> String {
        t("progress.name")
    }
    fn on_surface_changed(&mut self) {
        self.build();
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        match ev.kind {
            Kind::KeyDown => {
                let key = ev.key.as_str();
                match key {
                    "Escape" => shell::post(Cmd::BackToMenu),
                    "Tab" | "Right" | "d" => self.tab_step(1),
                    "Left" | "a" => self.tab_step(-1),
                    "Up" | "w" => {
                        self.scroll -= 40;
                        self.clamp_scroll();
                    }
                    "Down" | "s" => {
                        self.scroll += 40;
                        self.clamp_scroll();
                    }
                    "Prior" => {
                        self.scroll -= self.viewport();
                        self.clamp_scroll();
                    }
                    "Next" => {
                        self.scroll += self.viewport();
                        self.clamp_scroll();
                    }
                    "e" => self.export(),
                    "i" => self.import(),
                    _ => {}
                }
            }
            Kind::Wheel => {
                self.scroll -= ev.delta * 48;
                self.clamp_scroll();
            }
            Kind::MouseMove => {
                self.mouse = ev.pos;
                self.tab_hover = None;
                for (r, key) in &self.tab_rects {
                    if r.hit(ev.pos) {
                        self.tab_hover = Some(key);
                    }
                }
                self.hover_row = self.row_at(ev.pos);
            }
            Kind::MouseDown => {
                if let Some(&(_, key)) = self.tab_rects.iter().find(|(r, _)| r.hit(ev.pos)) {
                    self.switch_tab(key);
                    return;
                }
                if let Some((_, cb)) = self.buttons.iter().find(|(r, _)| r.hit(ev.pos)).cloned() {
                    self.b.play_sound("click");
                    match cb {
                        Btn::Filter(k) => {
                            self.filter = k;
                            self.scroll = 0;
                            self.build();
                        }
                        Btn::Series(k) => {
                            self.series_game = Some(k);
                            self.build();
                        }
                        Btn::Import => self.import(),
                        Btn::Export => self.export(),
                    }
                    return;
                }
                if let Some(hit) = self.row_at(ev.pos) {
                    if let Some(gi) = self.row_game(&self.rows[hit].2) {
                        self.b.play_sound("click");
                        shell::post(Cmd::PreGame(self.games[gi].key.to_string()));
                    }
                }
            }
            _ => {}
        }
    }

    fn draw(&mut self) {
        let p = ui::pal();
        let mut s = self.b.sf();
        let (w, h) = (self.b.width, self.b.height);
        self.tip = None;
        self.buttons.clear();
        ui::draw_background_ex(&mut s, w, h, false, None);
        ui::draw_title_ex(&mut s, w, &t("progress.name"), None, 40, Some(&ui::fb(30)), None, Some(p.gold));
        for (r, key) in &self.tab_rects {
            let active = *key == self.tab;
            let hover = Some(*key) == self.tab_hover;
            draw::rect_r(&mut s, if active || hover { p.panel_light } else { p.panel }, *r, 0, 13);
            draw::rect_r(&mut s, if active { p.gold } else { p.border }, *r, 1, 13);
            let img = self.tab_font.render(&t(&format!("progress.tab_{key}")), true, if active { p.text } else { p.text_dim });
            s.blr(&img, img.get_rect().center_at(r.center()));
        }
        self.draw_level_head(&mut s, w);
        s.set_clip(Some(Rect::new(0, self.top, w, self.bottom - self.top)));
        let rows = self.rows.clone();
        for (i, (ry, rh, row)) in rows.iter().enumerate() {
            let y = self.top + ry - self.scroll;
            if y + rh < self.top - 120 || y > self.bottom + 20 {
                continue;
            }
            self.draw_row(&mut s, y, *rh, row, i);
        }
        s.set_clip(None);
        let vp = self.viewport();
        if self.content_h > vp {
            let track = Rect::new(w - 10, self.top, 4, vp);
            draw::rect_r(&mut s, p.panel, track, 0, 2);
            let th = 24.max((vp as f64 * vp as f64 / self.content_h as f64) as i32);
            let ty = self.top + ((vp - th) as f64 * self.scroll as f64 / 1.max(self.content_h - vp) as f64) as i32;
            draw::rect_r(&mut s, p.border_light, (w - 10, ty, 4, th), 0, 2);
        }
        if let Some(tip) = self.tip.clone() {
            charts::tooltip(&mut s, self.mouse, &tip, Some(w));
        }
        let mut hint = t("progress.hint");
        if !self.flash.is_empty() && self.flash_t.is_some_and(|t0| t0.elapsed().as_secs_f64() < 4.0) {
            hint = self.flash.clone();
        }
        ui::draw_footer(&mut s, w, h, &hint);
    }
}

