//! ui_probe
//! ========
//! Rendert eine feste Szene aus `ui` in allen Designs nach PNG - die
//! Gegenseite von `rust/tools/ui_probe.py`, das dieselbe Szene mit echtem
//! pygame und `ui.py` rendert. Der Vergleich der beiden Bilder ist die
//! Messung, ob `ui.rs` wirklich `ui.py` ist.
//!
//! ```text
//! cargo run -p pgz-engine --example ui_probe -- <ordner>
//! ```

use pgz_engine::game::{InputEvent, Kind};
use pgz_engine::pg::{self, draw, SurfExt};
use pgz_engine::py;
use pgz_engine::ui;

fn szene(s: &mut pg::Surface, feld: &ui::TextInput) {
    let (w, h) = (640, 480);
    ui::draw_background(s, w, h);
    ui::draw_title(s, w, "PyGameZ Probe", Some("Untertitel der Szene"));
    ui::draw_panel(s, (40, 120, 560, 250));
    ui::draw_button(s, (70, 150, 230, 46), "Start", &ui::f(22), true);
    ui::draw_button(s, (340, 150, 230, 46), "Optionen", &ui::f(22), false);
    ui::draw_button_ex(
        s,
        (70, 215, 230, 56),
        "Mehrspieler",
        &ui::fb(20),
        false,
        Some("zwei Spieler"),
        Some(&ui::f(14)),
        Some(pg::Color::rgb(240, 160, 90)),
    );
    ui::draw_panel_ex(s, (340, 215, 230, 56), None, None, Some(8), false, Some(ui::gold()));
    feld.draw(s, (90, 300, 260, 36), &ui::f(18), true, false);
    ui::draw_grad_line(s, 470, 318, 180, 3, None, true);
    let img = ui::grad_text(&ui::fb(26), "Verlauf 123", None, None);
    s.bl(&img, (390, 330));
    draw::circle(s, ui::accent(), (600, 420), 12, 0);
    ui::draw_footer(s, w, h, "Enter = Start  -  Esc = Pause");
    ui::draw_halo(s, (320, 60), 180, None);
}

fn main() {
    let ordner = std::env::args().nth(1).unwrap_or_else(|| "ui_probe".into());
    std::fs::create_dir_all(&ordner).ok();
    pg::init_font();
    let mut feld = ui::TextInput::new("", 20, None, "Name");
    for ch in "Lama 42".chars() {
        let mut ev = InputEvent::new(Kind::KeyDown);
        ev.key = if ch == ' ' { "space".into() } else { ch.to_string() };
        ev.char = ch.to_string();
        feld.handle(&ev);
    }
    for name in ui::theme_names() {
        ui::set_theme(name);
        py::seed(7);
        let mut s = pg::surface(640, 480);
        for i in 0..12u64 {
            pg::set_fixed_ticks(Some(5000 + i * 17));
            szene(&mut s, &feld);
        }
        let pfad = format!("{ordner}/{name}.png");
        pg::image::save(&s, &pfad);
        println!("{pfad}");
    }
}
