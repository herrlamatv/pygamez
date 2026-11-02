//! Eine kleine Szene - ohne Fenster, ohne SDL.
//!
//! ```text
//! cargo run -p rustgame-sprite --example szene
//! ```
//!
//! Gezeigt wird alles, was die Kiste kann, an einem Stueck:
//!
//! * eigener Zustand und eigenes `update` ueber [`SpriteVerhalten`],
//! * ein Spieler in einer [`GroupSingle`],
//! * Funken in einer [`Group`], die an den Waenden abprallen,
//! * Ebenen in [`LayeredUpdates`]: Hintergrund, Funken, Spieler,
//! * [`spritecollide`] mit [`collide_circle`] und `dokill`,
//! * [`AbstractGroup::draw`] auf eine Flaeche, am Ende als ASCII-Bild.
//!
//! Am Ende steht die Flaeche im Terminal - das ist der ganze "Bildschirm".

use std::any::Any;

use rustgame_core::Surface;
use rustgame_sprite::*;

const BREITE: u32 = 64;
const HOEHE: u32 = 24;
const BILDER: usize = 40;

// ---------------------------------------------------------------------------
//  Eigenes Verhalten
// ---------------------------------------------------------------------------

/// Fliegt geradeaus und prallt an den Raendern ab.
struct Funke {
    dx: i32,
    dy: i32,
    /// Wie oft es schon gegen eine Wand ging - eigener Zustand, den die
    /// Gruppenverwaltung nie zu Gesicht bekommt.
    stoesse: u32,
}

impl SpriteVerhalten for Funke {
    fn update(&mut self, s: &Sprite, _args: &dyn Any) {
        let Some(mut r) = s.rect() else { return };
        r.move_ip(self.dx, self.dy);
        if r.x < 0 || r.right() > BREITE as i32 {
            self.dx = -self.dx;
            self.stoesse += 1;
            r.move_ip(self.dx * 2, 0);
        }
        if r.y < 0 || r.bottom() > HOEHE as i32 {
            self.dy = -self.dy;
            self.stoesse += 1;
            r.move_ip(0, self.dy * 2);
        }
        s.set_rect(Some(r));
    }
}

/// Faehrt eine feste Bahn ab - der "Spieler".
struct Bahn {
    schritt: usize,
}

impl SpriteVerhalten for Bahn {
    fn update(&mut self, s: &Sprite, _args: &dyn Any) {
        // Eine gemaechliche Acht ueber die Flaeche.
        let t = self.schritt as f64 * 0.16;
        let x = (BREITE as f64 / 2.0 + 22.0 * t.sin()) as i32;
        let y = (HOEHE as f64 / 2.0 + 8.0 * (2.0 * t).sin()) as i32;
        s.set_rect(s.rect().map(|r| r.with_center((x, y))));
        self.schritt += 1;
    }
}

// ---------------------------------------------------------------------------

fn kachel(w: u32, h: u32, farbe: (u8, u8, u8, u8)) -> Surface {
    let mut s = Surface::new((w, h));
    s.fill(farbe);
    s
}

fn neuer_funke(x: i32, y: i32, dx: i32, dy: i32) -> Sprite {
    let s = Sprite::mit_bild(kachel(3, 2, (240, 200, 60, 255)));
    s.set_topleft((x, y));
    s.set_verhalten(Funke { dx, dy, stoesse: 0 });
    s
}

fn main() {
    // ----- Die Welt ----------------------------------------------------
    let mut bildschirm = Surface::new((BREITE, HOEHE));
    let mut hintergrund = Surface::new((BREITE, HOEHE));
    hintergrund.fill((12, 16, 28, 255));

    // Der Spieler wohnt in einer GroupSingle - da passt genau einer rein,
    // und der Neue wuerfe den Alten hinaus.
    let spieler = Sprite::mit_bild(kachel(5, 3, (90, 220, 255, 255)));
    spieler.set_verhalten(Bahn { schritt: 0 });
    let held = GroupSingle::mit(&spieler);

    // Die Funken.
    let funken = Group::new();
    for i in 0..8i32 {
        funken.add(neuer_funke(
            4 + i * 7,
            2 + (i % 5) * 4,
            if i % 2 == 0 { 2 } else { -2 },
            if i % 3 == 0 { 1 } else { -1 },
        ));
    }

    // Und ein paar ruhende Bloecke als Kulisse.
    let kulisse = Group::new();
    for i in 0..5i32 {
        let b = Sprite::mit_bild(kachel(4, 2, (40, 60, 90, 255)));
        b.set_topleft((6 + i * 12, 20));
        kulisse.add(&b);
    }

    // Gezeichnet wird ueber Ebenen: Kulisse ganz unten, dann die Funken,
    // der Spieler oben drauf.
    let buehne = LayeredUpdates::new();
    buehne.add_mit_ebene(&kulisse, 0);
    buehne.add_mit_ebene(&funken, 1);
    buehne.add_mit_ebene(&spieler, 2);
    println!("Ebenen: {:?}, Sprites auf der Buehne: {}", buehne.layers(), buehne.len());

    // ----- Der Ablauf ---------------------------------------------------
    let mut eingesammelt = 0usize;
    let mut beruehrte_kulisse = 0usize;

    for bild in 0..BILDER {
        bildschirm.blit(&hintergrund, (0, 0));

        held.update();
        funken.update();

        // Funken, die den Spieler beruehren, verschwinden - `dokill`
        // nimmt sie aus **allen** Gruppen, also auch von der Buehne.
        let getroffen = spritecollide(&spieler, &funken, true, Some(&collide_circle));
        eingesammelt += getroffen.len();
        for f in &getroffen {
            let stoesse = f.mit_zustand(|v: &Funke| v.stoesse).unwrap_or(0);
            println!(
                "Bild {bild:>2}: Funke bei {:?} eingesammelt (Wandstoesse: {stoesse}); \
                 in {} Gruppen",
                f.rect().unwrap().topleft(),
                f.anzahl_gruppen()
            );
        }

        // Nur schauen, nicht toeten.
        if spritecollideany(&spieler, &kulisse, None).is_some() {
            beruehrte_kulisse += 1;
        }

        let veraendert = buehne.draw(&mut bildschirm);
        if bild == 0 {
            println!(
                "Bild  0: draw meldet {} veraenderte Bereiche, der erste ist {:?}",
                veraendert.len(),
                veraendert.first()
            );
        }
    }

    // ----- Was uebrig ist ------------------------------------------------
    println!();
    println!("Nach {BILDER} Bildern:");
    println!("  eingesammelte Funken : {eingesammelt}");
    println!("  noch unterwegs       : {}", funken.len());
    println!("  Kulissenberuehrungen : {beruehrte_kulisse}");
    println!("  auf der Buehne       : {}", buehne.len());
    println!(
        "  Spieler lebt in      : {} Gruppen, auf Ebene {}",
        spieler.anzahl_gruppen(),
        buehne.get_layer_of_sprite(&spieler)
    );

    let summe: u32 = funken
        .sprites()
        .iter()
        .map(|s| s.mit_zustand(|f: &Funke| f.stoesse).unwrap_or(0))
        .sum();
    println!("  Wandstoesse zusammen : {summe}");

    println!("\nDer Bildschirm:");
    zeige(&bildschirm);
}

/// Malt die Flaeche als ASCII-Bild - ein Zeichen je Pixel.
fn zeige(s: &Surface) {
    let rand = "-".repeat(s.get_width() as usize + 2);
    println!("+{rand}+");
    for y in 0..s.get_height() as i32 {
        print!("| ");
        for x in 0..s.get_width() as i32 {
            let c = s.get_at((x, y));
            let z = match (c.r, c.g, c.b) {
                (90, 220, 255) => '#', // Spieler
                (240, 200, 60) => '*', // Funke
                (40, 60, 90) => '=',   // Kulisse
                _ => ' ',
            };
            print!("{z}");
        }
        println!(" |");
    }
    println!("+{rand}+");
}
