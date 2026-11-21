# ENGINE.md — der Portierungs-Vertrag

Wie ein Spiel aus `games/*.py` in die Rust-Fassung kommt. Wer ein Spiel
überträgt, liest **diese Datei** und die Python-Vorlage.

Die Engine steht auf **rustgame**, dem pixelgleichen Nachbau von pygame-ce.
Deshalb wird **übertragen, nicht nachempfunden**: dieselben Zeichenaufrufe in
derselben Reihenfolge, dieselben Konstanten, dieselben Zufallsaufrufe. Dann
ist das Bild dasselbe wie in Python — gemessen für `ui.py` (`tools/ui_probe.py`:
0 Pixel Unterschied in allen zehn Designs).

---

## 1. Aufbau eines Spiels

`crates/pgz-games/src/<spiel>.rs`, Kopfkommentar mit Herkunft und dem, was
fehlt. Gerüst:

```rust
use pgz_engine::prelude::*;

pub struct PongGame {
    b: GameBase,          // was Python von game_base.Game erbt
    player_y: f64,        // eigener Zustand - Felder statt self.xyz
    ...
}

impl PongGame {
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let mut g = PongGame { b, player_y: 0.0, ... };
        g.reset();                       // __init__ ruft reset()
        Box::new(g)
    }
}

impl Game for PongGame {
    fn base(&self) -> &GameBase { &self.b }
    fn base_mut(&mut self) -> &mut GameBase { &mut self.b }
    fn reset(&mut self) { ... }
    fn update(&mut self, dt: f64) { ... }
    fn draw(&mut self) { let mut s = self.b.sf(); ... }
    fn handle_event(&mut self, ev: &InputEvent) { ... }
}

pub const INFO: GameInfo = GameInfo {
    key: "pong",                   // highscore_key
    class_name: "PongGame",        // Python-Klasse (Farbe, Name, Icon)
    supports_multiplayer: true,
    modes: &[],                    // MODES = [(mode, i18n), ...]
    make: PongGame::new,
};
```

Eintragen: `pub mod pong;` und `pong::INFO` in `all_games()` (Reihenfolge wie
`games/__init__.py`).

Optionale Hooks: `on_surface_changed`, `on_exit`, `wants_escape`,
`wants_right_click`, `capture_mouse`, `show_highscore_banner`.

## 2. Python → Rust, Zeile für Zeile

| Python | Rust |
|---|---|
| `self.score`, `self.game_over`, `self.width` | `self.b.score`, `self.b.game_over`, `self.b.width` |
| `self.surface` | `let mut s = self.b.sf();` (teilt den Speicher) |
| `self.play_sound("eat")` | `self.b.play_sound("eat")` |
| `self.is_action(key, "up", "p1")` | `self.b.is_action(&ev.key, "up", Some("p1"))` |
| `self.report_result(won)` / `report_duel(w)` | `self.b.report_result(won)` / `self.b.report_duel(w)` |
| `self.ach_event("id", v)` | `self.b.ach_event("id", Some(v))` |
| `self.pname(0)` / `self.pcolor(1)` | `self.b.pname(0)` / `self.b.pcolor(1)` |
| `event.kind == InputEvent.KEYDOWN` | `ev.kind == Kind::KeyDown` |
| `event.key in ("Return", "space")` | `ev.key_in(&["Return", "space"])` |
| `pygame.draw.rect(s, c, r, w, border_radius=k)` | `draw::rect_r(&mut s, c, r, w, k)` (ohne Radius: `draw::rect`) |
| `pygame.draw.circle(s, c, p, r, w)` | `draw::circle(&mut s, c, p, r, w)` |
| `pygame.draw.line/lines/polygon/ellipse/arc` | gleichnamig in `pg::draw` |
| `pygame.Surface((w, h), pygame.SRCALPHA)` | `pg::surface_alpha(w, h)` |
| `s.blit(img, (x, y))` / `s.blit(img, rect)` | `s.bl(&img, (x, y))` / `s.blr(&img, rect)` |
| `img.get_rect(center=(x, y))` | `img.get_rect().center_at((x, y))` |
| `s.fill(c, rect)` | `s.fill_r(c, rect)` |
| `img.set_alpha(a)` | `img.alpha_set(a)` |
| `ui.TEXT`, `ui.ACCENT` | `ui::text()`, `ui::accent()` (oder `let p = ui::pal();`) |
| `ui.font(22)` / `bold=True` / `mono=True` | `ui::f(22)` / `ui::fb(22)` / `ui::fm(22)` |
| `font.render(t, True, c)` | `font.render(&t, true, c)` |
| `font.size(t)[0]` | `font.width(&t)` |
| `t("key")` / `t("key", n=3)` | `t("key")` / `tf("key", &[("n", "3")])` |
| `(*ui.PANEL[:3], 235)` | `pg::alpha(ui::panel(), 235)` |
| `random.random()` / `randint` / `choice` | `py::random()` / `py::randint(a, b)` / `py::choice(&v)` |
| `a // b`, `a % b` (negativ möglich) | `py::fdiv(a, b)`, `py::pmod(a, b)` |
| `round(x)` | `py::round(x)` (Bankrundung!) |
| `int(x)` | `x as i32` |
| `self.settings.get("pong", {}).get("hold", False)` | `self.b.opt_bool("pong", "hold", false)` |
| `settings_mod.save_settings(...)` nach Änderung | `self.b.set_opt("pong", "hold", v.into())` |
| `store.load_section("x")` / `save_section` | `pgz_core::store::load_section("x")` / `save_section` |
| `lamabank.bet_and_pay(b, w, KEY, "k", v)` | `lamabank::bet_and_pay(b, w, KEY, Some(("k", v)))` (ohne Zusatz `None`) |
| `charts.line(...)` / `charts.bars(..., max_value=, label_size=)` / `charts.tooltip` | `pgz_engine::charts::line` (`line_ex` mit `fill`) / `bars_ex` / `tooltip` |

**Zahlen**: pygame schneidet Kommazahlen zur Null hin ab; die Fassade
(`pg::Num`, `pg::Pt`, `pg::IntoRect`) tut dasselbe, also Koordinaten einfach
als `f64` hineingeben, wie in Python.

**Textbilder** kommen aus einem Zwischenspeicher und teilen ihren Speicher —
wer in ein gerendertes Textbild hineinzeichnet, nimmt vorher `.copy()`.

## 3. Was das Programm erledigt

Highscore, Statistik, Erfolge bei Game Over, Pause (Esc), Banner, Konfetti,
Übergänge, Einblendungen — wie `main.py`. Ein Spiel setzt nur `score` und
`game_over`.

## 4. Prüfstand

```bash
PYGAMEZ_SHOT=x.png PYGAMEZ_GAME=pong PYGAMEZ_KEYS=Return,Up cargo run
PYGAMEZ_SHOT=ordner PYGAMEZ_GAME=all cargo run
```

Ohne Fenster, mit fester Uhr; Datenordner mit `PYGAMEZ_DATA_DIR` umlenken.
Alle Schalter stehen in `crates/pygamez/src/main.rs` (Ereignisse: `Taste`,
`+Taste`/`-Taste`, `*Taste` = Wiederholung, `@x:y[:3]`, `~x:y`, `^dx:dy`,
`<x:y`/`>x:y`, `%x:y:d` = Mausrad). `tools/vergleich.py` (+ regression)
sowie `stats_check.py`, `import_check.py` und `lamabank_check.py` nehmen
die .exe aus `CARGO_TARGET_DIR`, falls gesetzt (sonst `rust.output/`).
