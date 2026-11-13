//! profileview.rs
//! ==============
//! Die Oberflaeche zu den Profilen - uebertragen aus `profileview.py`:
//!
//! - [`ProfileScreen`]: Profile anlegen, umbenennen, umgestalten, loeschen,
//!   wechseln.
//! - [`PlayerPickScreen`]: Auswahl fuer Spieler 2 vor einer Mehrspieler-Partie.
//! - [`LevelIntroScreen`]: einmalige Karte "Dein Startlevel".

use pgz_core::achievements::{self, Achievement};
use pgz_core::{levels, profiles, stats};
use pgz_engine::achv;
use pgz_engine::game::{Game, GameBase, GameInfo, InputEvent, Kind};
use pgz_engine::pg::{self, rect, rgb, Color, Font, Rect, RectExt, SurfExt};
use pgz_engine::shell::{self, Cmd};
use pgz_engine::ui::{self, TextInput};
use pgz_core::i18n::{t, tf};

const ROW_H: i32 = 62;
const GAP: i32 = 8;

fn col3(c: (u8, u8, u8)) -> Color {
    rgb(c.0, c.1, c.2)
}

fn font(size: i32) -> Font {
    ui::font(size, false, false)
}

/// Knopf-Aktionen (Python: Rueckrufe in `_buttons`).
#[derive(Clone, Debug)]
enum Btn {
    New,
    Delete(String),
    Edit(String),
    Switch(String),
    Icon(usize),
    Color(usize),
    Cancel,
    Save,
}

struct Editor {
    pid: Option<String>,
    field: TextInput,
    icon: usize,
    color: i64,
    err: String,
}

/// Profilverwaltung: wer spielt, und wer bekommt die Punkte?
pub struct ProfileScreen {
    b: GameBase,
    sel: usize,
    editor: Option<Editor>,
    msg: String,
    buttons: Vec<(Rect, Btn)>,
    left: i32,
    right: i32,
    top: i32,
    bottom: i32,
    list: Vec<String>,
    stats: Vec<stats::Totals>,
}

impl ProfileScreen {
    /// `ProfileScreen(surface, w, h, app, on_close)`
    pub fn new(b: GameBase) -> ProfileScreen {
        let mut s = ProfileScreen {
            b,
            sel: 0,
            editor: None,
            msg: String::new(),
            buttons: Vec::new(),
            left: 0,
            right: 0,
            top: 0,
            bottom: 0,
            list: Vec::new(),
            stats: Vec::new(),
        };
        s.build();
        s
    }

    fn build(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        self.left = 28.max(w / 2 - 300);
        self.right = w - self.left;
        self.top = 108;
        self.bottom = h - 56;
        self.list = profiles::all().into_iter().map(|(pid, _)| pid).collect();
        self.stats = self.list.iter().map(|pid| stats::totals(Some(pid))).collect();
        self.sel = self.sel.min(self.list.len());
    }

    fn close(&self) {
        shell::post(Cmd::BackToMenu);
    }

    fn refresh_profile(&self) {
        // Python ruft refresh_profile sofort (backfill vor _build); die
        // Seitenleiste zieht ueber den Auftrag nach.
        achievements::backfill_silent();
        shell::post(Cmd::RefreshProfile);
    }

    fn activate(&mut self, index: usize) {
        if index >= self.list.len() {
            self.new_profile();
            return;
        }
        let pid = self.list[index].clone();
        self.switch(&pid);
    }

    fn switch(&mut self, pid: &str) {
        if pid == profiles::active_id() {
            self.close();
            return;
        }
        self.b.play_sound("click");
        profiles::switch(pid);
        self.refresh_profile();
        self.build();
        self.msg = tf("profile.switched", &[("name", &profiles::name_of(Some(pid)))]);
    }

    fn new_profile(&mut self) {
        if profiles::count() >= 8 {
            self.msg = t("profile.err_full");
            return;
        }
        self.editor = Some(Editor {
            pid: None,
            field: TextInput::new("", profiles::MAX_NAME, None, &t("profile.name_hint")),
            icon: 0,
            color: (profiles::count() % profiles::COLORS.len()) as i64,
            err: String::new(),
        });
    }

    fn edit_profile(&mut self, pid: &str) {
        let meta = profiles::meta(Some(pid));
        let icon = profiles::icon_of(Some(pid));
        self.editor = Some(Editor {
            pid: Some(pid.to_string()),
            field: TextInput::new(&meta.name, profiles::MAX_NAME, None, &t("profile.name_hint")),
            icon: profiles::ICONS.iter().position(|i| *i == icon).unwrap_or(0),
            color: meta.color,
            err: String::new(),
        });
    }

    fn delete_profile(&mut self, pid: &str) {
        if profiles::count() <= 1 {
            self.msg = t("profile.err_last");
            return;
        }
        if profiles::delete(pid) {
            self.refresh_profile();
            self.build();
            self.msg = t("profile.deleted");
        }
    }

    fn press(&mut self, btn: Btn) {
        match btn {
            Btn::New => self.new_profile(),
            Btn::Delete(p) => self.delete_profile(&p),
            Btn::Edit(p) => self.edit_profile(&p),
            Btn::Switch(p) => self.switch(&p),
            Btn::Icon(k) => {
                if let Some(ed) = self.editor.as_mut() {
                    ed.icon = k;
                }
            }
            Btn::Color(k) => {
                if let Some(ed) = self.editor.as_mut() {
                    ed.color = k as i64;
                }
            }
            Btn::Cancel => self.editor = None,
            Btn::Save => self.save_editor(),
        }
    }

    fn editor_event(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown {
            if ev.key == "Escape" {
                self.editor = None;
                return;
            }
            if ev.key == "Return" || ev.key == "KP_Enter" {
                self.save_editor();
                return;
            }
            let Some(ed) = self.editor.as_mut() else { return };
            if (ev.key == "Left" || ev.key == "Right") && ed.field.text.is_empty() {
                let step = if ev.key == "Left" { -1 } else { 1 };
                ed.color = (ed.color + step).rem_euclid(profiles::COLORS.len() as i64);
                return;
            }
            ed.field.handle(ev);
        } else if ev.kind == Kind::MouseDown {
            if let Some((_, cb)) = self.buttons.iter().find(|(r, _)| r.hit(ev.pos)).cloned() {
                self.b.play_sound("click");
                self.press(cb);
            }
        }
    }

    fn save_editor(&mut self) {
        let Some(ed) = self.editor.as_mut() else { return };
        let name = ed.field.text.trim().to_string();
        let icon = profiles::ICONS[ed.icon % profiles::ICONS.len()];
        match ed.pid.clone() {
            None => match profiles::create(&name, icon, ed.color) {
                Ok(pid) => {
                    profiles::switch(&pid);
                }
                Err(e) => {
                    ed.err = t(&format!("profile.err_{}", e.code()));
                    return;
                }
            },
            Some(pid) => {
                if let Err(e) = profiles::rename(&pid, &name) {
                    ed.err = t(&format!("profile.err_{}", e.code()));
                    return;
                }
                profiles::set_look(&pid, Some(icon), Some(ed.color));
            }
        }
        self.editor = None;
        self.refresh_profile();
        self.build();
    }

    fn draw_row(&mut self, s: &mut pg::Surface, pid: &str, y: i32, index: usize) {
        let p = ui::pal();
        let s_rect = Rect::new(self.left, y, self.right - self.left, ROW_H);
        let active = pid == profiles::active_id();
        let selected = index == self.sel;
        ui::draw_panel_ex(s, s_rect, Some(if selected || active { p.panel_light } else { p.panel }), None, Some(12), false, None);
        let color = col3(profiles::color_rgb(Some(pid)));
        if active {
            pg::draw::rect_r(s, color, s_rect, 2, 12);
        }
        achv::draw_badge(s, (s_rect.x + 34, s_rect.centery()), 20, &profiles::icon_of(Some(pid)), color, false);
        let name = ui::fb(17).render(&profiles::name_of(Some(pid)), true, p.text);
        s.bl(&name, (s_rect.x + 64, s_rect.y + 9));
        let tot = &self.stats[index];
        let level = tot.level;
        let sub_txt = format!(
            "{} {} \u{b7} {} \u{b7} {} \u{b7} {}",
            t("level.short"),
            level,
            levels::rank_title(level),
            tf("progress.points", &[("n", &tot.points.to_string())]),
            stats::format_time(tot.time)
        );
        let sub = font(11).render(&sub_txt, true, p.text_dim);
        s.bl_area(&sub, (s_rect.x + 64, s_rect.y + 12 + name.h()), (0, 0, s_rect.w - 190, sub.h()));
        let mut x = s_rect.right() - 12;
        let fnt = ui::fb(11);
        let play = if !active { t("profile.play") } else { t("profile.active") };
        for (label, cb, danger) in [
            ("\u{2715}".to_string(), Btn::Delete(pid.to_string()), true),
            ("\u{270e}".to_string(), Btn::Edit(pid.to_string()), false),
            (play, Btn::Switch(pid.to_string()), false),
        ] {
            let w = 26.max(fnt.size(&label).0 + 16);
            let r = Rect::new(x - w, s_rect.centery() - 11, w, 22);
            let fill = if !danger { p.panel } else { ui::mix(p.panel, p.red, 0.25) };
            pg::draw::rect_r(s, fill, r, 0, 11);
            pg::draw::rect_r(s, p.border_light, r, 1, 11);
            let img = fnt.render(&label, true, p.text_dim);
            s.blr(&img, img.get_rect().center_at(r.center()));
            self.buttons.push((r, cb));
            x -= w + 6;
        }
    }

    fn draw_editor(&mut self, s: &mut pg::Surface, w: i32, h: i32) {
        let p = ui::pal();
        let mut overlay = pg::surface_alpha(w, h);
        overlay.fill_c(pg::rgba(0, 0, 0, 150));
        s.bl(&overlay, (0, 0));
        let (bw, bh) = (460.min(w - 60), 250);
        let bx = Rect::new(w / 2 - bw / 2, h / 2 - bh / 2, bw, bh);
        ui::draw_panel_ex(s, bx, Some(p.panel_light), None, Some(14), true, Some(p.accent));
        let ed = self.editor.as_ref().expect("editor");
        let title = ui::fb(16).render(&if ed.pid.is_none() { t("profile.new") } else { t("profile.edit") }, true, p.text);
        s.bl(&title, (bx.x + 18, bx.y + 12));
        let field = Rect::new(bx.x + 18, bx.y + 46, bx.w - 36, 30);
        ed.field.draw(s, field, &font(15), true, false);
        let label = font(11).render(&t("profile.icon"), true, p.text_dim);
        s.bl(&label, (bx.x + 18, field.bottom() + 8));
        let mut x = bx.x + 18;
        let mut y = field.bottom() + 24;
        let (ed_icon, ed_color) = (ed.icon, ed.color);
        let mut neu = Vec::new();
        for (i, icon) in profiles::ICONS.iter().enumerate() {
            let r = Rect::new(x, y, 30, 30);
            let chosen = i == ed_icon;
            let c = col3(levels::hex_to_rgb(profiles::COLORS[ed_color.rem_euclid(profiles::COLORS.len() as i64) as usize]));
            achv::draw_badge(s, r.center(), 13, icon, if chosen { c } else { p.border_light }, false);
            if chosen {
                pg::draw::circle(s, p.text, r.center(), 15, 1);
            }
            neu.push((r, Btn::Icon(i)));
            x += 34;
            if x + 30 > bx.right() - 18 {
                x = bx.x + 18;
                y += 34;
            }
        }
        let label = font(11).render(&t("profile.color"), true, p.text_dim);
        s.bl(&label, (bx.x + 18, y + 34));
        let mut x = bx.x + 18;
        let cy = y + 52;
        for (i, hex) in profiles::COLORS.iter().enumerate() {
            let r = Rect::new(x, cy, 22, 22);
            pg::draw::rect_r(s, col3(levels::hex_to_rgb(hex)), r, 0, 11);
            if i as i64 == ed_color {
                pg::draw::rect_r(s, p.text, r.inflate(6, 6), 1, 14);
            }
            neu.push((r, Btn::Color(i)));
            x += 28;
        }
        let ed = self.editor.as_ref().expect("editor");
        if !ed.err.is_empty() {
            let img = ui::fb(11).render(&ed.err, true, p.red);
            s.bl(&img, (bx.x + 18, bx.bottom() - 30));
        }
        let fnt = ui::fb(12);
        let mut x = bx.right() - 18;
        for (key, cb) in [("profile.cancel", Btn::Cancel), ("profile.save", Btn::Save)] {
            let label = t(key);
            let w = fnt.size(&label).0 + 24;
            let r = Rect::new(x - w, bx.bottom() - 38, w, 26);
            pg::draw::rect_r(s, if key == "profile.save" { p.btn_sel } else { p.btn }, r, 0, 13);
            let img = fnt.render(&label, true, p.text);
            s.blr(&img, img.get_rect().center_at(r.center()));
            neu.push((r, cb));
            x -= w + 8;
        }
        self.buttons.extend(neu);
    }
}

impl Game for ProfileScreen {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }
    fn reset(&mut self) {}
    fn update(&mut self, _dt: f64) {}
    fn screen_name(&self) -> String {
        t("profile.name")
    }
    fn on_surface_changed(&mut self) {
        self.build();
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if self.editor.is_some() {
            self.editor_event(ev);
            return;
        }
        let n = self.list.len();
        match ev.kind {
            Kind::KeyDown => {
                let key = ev.key.as_str();
                if key == "Escape" {
                    self.close();
                } else if key == "Up" || key == "w" {
                    self.sel = (self.sel as i64 - 1).rem_euclid(n as i64 + 1) as usize;
                    self.b.play_sound("move");
                } else if key == "Down" || key == "s" {
                    self.sel = (self.sel + 1) % (n + 1);
                    self.b.play_sound("move");
                } else if key == "Return" || key == "space" {
                    self.activate(self.sel);
                } else if key == "n" {
                    self.new_profile();
                } else if key == "F2" && self.sel < n {
                    let pid = self.list[self.sel].clone();
                    self.edit_profile(&pid);
                } else if key == "Delete" && self.sel < n {
                    let pid = self.list[self.sel].clone();
                    self.delete_profile(&pid);
                }
            }
            Kind::MouseMove => {
                for (i, (r, _)) in self.buttons.iter().enumerate() {
                    if r.hit(ev.pos) && i <= n {
                        self.sel = i;
                    }
                }
            }
            Kind::MouseDown => {
                if let Some((_, cb)) = self.buttons.iter().find(|(r, _)| r.hit(ev.pos)).cloned() {
                    self.b.play_sound("click");
                    self.press(cb);
                }
            }
            _ => {}
        }
    }

    fn draw(&mut self) {
        let p = ui::pal();
        let mut s = self.b.sf();
        let (w, h) = (self.b.width, self.b.height);
        self.buttons.clear();
        ui::draw_background_ex(&mut s, w, h, false, None);
        ui::draw_title_ex(&mut s, w, &t("profile.name"), Some(&t("profile.subtitle")), 40, Some(&ui::fb(28)), None, Some(p.accent));
        let mut y = self.top;
        for i in 0..self.list.len() {
            let pid = self.list[i].clone();
            self.draw_row(&mut s, &pid, y, i);
            y += ROW_H + GAP;
        }
        let r = Rect::new(self.left, y, self.right - self.left, 34);
        let selected = self.sel == self.list.len();
        ui::draw_panel_ex(&mut s, r, Some(if selected { p.panel_light } else { p.panel }), None, Some(10), false, None);
        pg::draw::rect_r(&mut s, if selected { p.accent } else { p.border }, r, 1, 10);
        let label = ui::fb(13).render(&format!("+  {}", t("profile.new")), true, if selected { p.text } else { p.text_dim });
        s.blr(&label, label.get_rect().center_at(r.center()));
        self.buttons.push((r, Btn::New));
        if !self.msg.is_empty() {
            let img = font(12).render(&self.msg, true, p.gold);
            s.bl(&img, (self.left, self.bottom + 4));
        }
        if self.editor.is_some() {
            self.draw_editor(&mut s, w, h);
            ui::draw_footer(&mut s, w, h, &t("profile.hint_edit"));
        } else {
            ui::draw_footer(&mut s, w, h, &t("profile.hint"));
        }
    }
}

/// Auswahl des Profils fuer Spieler 2 vor einer Mehrspieler-Partie.
pub struct PlayerPickScreen {
    b: GameBase,
    info: GameInfo,
    mode: String,
    sel: usize,
    rects: Vec<Rect>,
    options: Vec<(String, String)>,
}

impl PlayerPickScreen {
    /// `PlayerPickScreen(surface, w, h, app, on_pick, on_cancel, game_cls)`
    pub fn new(b: GameBase, info: GameInfo, mode: &str) -> PlayerPickScreen {
        let mut s = PlayerPickScreen { b, info, mode: mode.to_string(), sel: 0, rects: Vec::new(), options: Vec::new() };
        s.build();
        s
    }

    fn build(&mut self) {
        let me = profiles::active_id();
        self.options = profiles::all()
            .into_iter()
            .filter(|(pid, _)| *pid != me)
            .map(|(pid, _)| {
                let n = profiles::name_of(Some(&pid));
                (pid, n)
            })
            .collect();
        self.options.push((profiles::GUEST.to_string(), t("profile.guest")));
    }

    fn pick(&mut self, index: usize) {
        self.b.play_sound("click");
        let pid = self.options[index].0.clone();
        let p2 = if pid == profiles::GUEST { None } else { Some(pid) };
        shell::post(Cmd::Launch { key: self.info.key.to_string(), mode: self.mode.clone(), p2 });
    }
}

impl Game for PlayerPickScreen {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }
    fn reset(&mut self) {}
    fn update(&mut self, _dt: f64) {}
    fn screen_name(&self) -> String {
        t("profile.pick_name")
    }
    fn on_surface_changed(&mut self) {
        self.build();
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        let n = self.options.len() as i64;
        match ev.kind {
            Kind::KeyDown => {
                let k = ev.key.as_str();
                if k == "Escape" {
                    shell::post(Cmd::PreGame(self.info.key.to_string()));
                } else if k == "Up" || k == "w" {
                    self.sel = (self.sel as i64 - 1).rem_euclid(n) as usize;
                    self.b.play_sound("move");
                } else if k == "Down" || k == "s" {
                    self.sel = (self.sel as i64 + 1).rem_euclid(n) as usize;
                    self.b.play_sound("move");
                } else if k == "Return" || k == "space" {
                    self.pick(self.sel);
                }
            }
            Kind::MouseMove => {
                for (i, r) in self.rects.iter().enumerate() {
                    if r.hit(ev.pos) {
                        self.sel = i;
                    }
                }
            }
            Kind::MouseDown => {
                if let Some(i) = self.rects.iter().position(|r| r.hit(ev.pos)) {
                    self.pick(i);
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
        let accent = ui::game_color(self.info.class_name);
        ui::draw_title_ex(&mut s, w, &t("profile.pick_name"), Some(&tf("profile.pick_sub", &[("name", &profiles::name_of(None))])), 64, None, None, Some(accent));
        let bw = 320.min(w - 60);
        let bh = 40;
        let gap = 8;
        let total = self.options.len() as i32 * (bh + gap) - gap;
        let y0 = 140.max(h / 2 - pgz_engine::py::fdiv32(total, 2));
        self.rects.clear();
        for (i, (pid, name)) in self.options.iter().enumerate() {
            let r = Rect::new(w / 2 - bw / 2, y0 + i as i32 * (bh + gap), bw, bh);
            self.rects.push(r);
            let selected = i == self.sel;
            ui::draw_panel_ex(&mut s, r, Some(if selected { p.panel_light } else { p.panel }), None, Some(12), false, None);
            let col = if pid != profiles::GUEST { col3(profiles::color_rgb(Some(pid))) } else { p.text_faint };
            if selected {
                pg::draw::rect_r(&mut s, col, r, 2, 12);
            }
            achv::draw_badge(&mut s, (r.x + 24, r.centery()), 13, &profiles::icon_of(Some(pid)), col, false);
            let img = ui::fb(15).render(name, true, if selected { p.text } else { p.text_dim });
            s.blr(&img, img.get_rect().midleft_at((r.x + 46, r.centery())));
            if pid != profiles::GUEST {
                let (level, _pts) = profiles::level_of(Some(pid));
                let lvl = font(11).render(&format!("{} {}", t("level.short"), level), true, p.text_faint);
                s.blr(&lvl, lvl.get_rect().midright_at((r.right() - 14, r.centery())));
            }
        }
        ui::draw_footer(&mut s, w, h, &t("profile.pick_hint"));
    }
}

/// Einmalige Begruessung nach dem Update: "Dein Startlevel".
pub struct LevelIntroScreen {
    b: GameBase,
    rarest: Vec<(&'static Achievement, String)>,
    level: i64,
    points: i64,
    n_unlocked: usize,
    n_total: usize,
}

impl LevelIntroScreen {
    /// `LevelIntroScreen(surface, w, h, app, on_close)`
    pub fn new(b: GameBase) -> LevelIntroScreen {
        let rarest = achievements::rarest_unlocked(3);
        let (level, points) = stats::level_of(None);
        let (n_unlocked, n_total) = achievements::counts();
        LevelIntroScreen { b, rarest, level, points, n_unlocked, n_total }
    }
}

impl Game for LevelIntroScreen {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }
    fn reset(&mut self) {}
    fn update(&mut self, _dt: f64) {}
    fn screen_name(&self) -> String {
        t("level.intro_name")
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if (ev.kind == Kind::KeyDown && ["Escape", "Return", "space", "KP_Enter"].contains(&ev.key.as_str())) || ev.kind == Kind::MouseDown {
            shell::post(Cmd::BackToMenu);
        }
    }

    fn draw(&mut self) {
        let p = ui::pal();
        let mut s = self.b.sf();
        let (w, h) = (self.b.width, self.b.height);
        ui::draw_background(&mut s, w, h);
        let color = col3(levels::rank_color(self.level));
        ui::draw_title_ex(&mut s, w, &t("level.intro_title"), Some(&t("level.intro_sub")), 70, None, None, Some(color));
        let (bw, bh) = (520.min(w - 60), 300.min(h - 200));
        let bx = rect(w / 2 - bw / 2, 150, bw, bh);
        ui::draw_panel_ex(&mut s, bx, Some(p.panel_light), None, Some(16), true, Some(color));
        achv::draw_badge(&mut s, (bx.centerx(), bx.y + 52), 34, "bolt", color, false);
        let big = ui::fb(28).render(&format!("{} {}", t("level.short"), self.level), true, p.text);
        s.blr(&big, big.get_rect().midtop_at((bx.centerx(), bx.y + 92)));
        let rank = ui::fb(15).render(&levels::rank_title(self.level), true, color);
        s.blr(&rank, rank.get_rect().midtop_at((bx.centerx(), bx.y + 92 + big.h())));
        let sub = font(12).render(
            &tf("level.intro_points", &[("n", &self.points.to_string()), ("a", &self.n_unlocked.to_string()), ("m", &self.n_total.to_string())]),
            true,
            p.text_dim,
        );
        s.blr(&sub, sub.get_rect().midtop_at((bx.centerx(), bx.y + 130 + big.h())));
        let mut y = bx.y + 160 + big.h();
        for (d, ts) in &self.rarest {
            achv::draw_badge(&mut s, (bx.x + 40, y + 12), 12, &d.icon, col3(d.rarity_color()), false);
            let img = ui::fb(12).render(&d.display_name(), true, p.text);
            s.bl(&img, (bx.x + 60, y + 2));
            let day: String = ts.chars().take(10).collect();
            let meta = font(10).render(&format!("{} \u{b7} {}", d.rarity_name(), day), true, p.text_faint);
            s.bl(&meta, (bx.x + 60, y + 4 + img.h()));
            y += 30;
        }
        ui::draw_footer(&mut s, w, h, &t("level.intro_hint"));
    }
}
