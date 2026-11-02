//! Die Kollisionsfunktionen.
//!
//! Vorlage: `pygame/sprite.py`, Zeilen 1468-1788.
//!
//! ## Was an `collide_circle` wirklich passiert
//!
//! Der Text sagt: "wenn das Sprite kein `radius` hat, wird einer aus dem
//! Rechteck gerechnet". Der Code tut mehr - er **schreibt ihn an**:
//!
//! ```python
//! try:
//!     leftradius = left.radius
//! except AttributeError:
//!     leftradius = 0.5 * ((leftrect.width**2 + leftrect.height**2) ** 0.5)
//!     left.radius = leftradius      # <- die Zeile
//! ```
//!
//! Gemessen, was das bedeutet: ein Sprite mit `rect = (0,0,10,10)`
//! bekommt nach dem ersten `collide_circle` den Radius
//! `7.0710678118654755`. Waechst sein Rechteck danach auf `100x100`,
//! bleibt der alte Radius stehen und die naechste Pruefung liefert
//! `False`, wo sie `True` liefern muesste. Das ist kein Fehler dieser
//! Uebersetzung, sondern pygames Zwischenspeicher - und er steht hier
//! genauso drin, weil er beobachtbar ist.
//!
//! `collide_circle_ratio` legt den **ungeskalierten** Radius ab und
//! multipliziert erst danach (auch gemessen).
//!
//! ## Was `dokill` genau wann tut
//!
//! `spritecollide(..., dokill=True)` sammelt **erst vollstaendig** und
//! toetet **danach**. Die getoeteten Sprites stehen im Ergebnis, sind
//! aber in keiner Gruppe mehr (`alive()` ist `false`). Gemessen.
//!
//! `groupcollide(a, b, dokilla, dokillb)` ruft `spritecollide` je
//! a-Sprite. Mit `dokillb = true` sind die b-Sprites also **schon tot**,
//! wenn das naechste a-Sprite an die Reihe kommt - das Woerterbuch
//! enthaelt danach nur noch den ersten Treffer. Gemessen an zwei mal
//! zwei ueberlappenden Sprites:
//!
//! ```text
//! groupcollide(A, B, True, True) -> {A1: [B1, B2]}
//! Gruppen danach: A = [A2], B = []
//! ```
//!
//! Wer `{A1: [...], A2: [...]}` erwartet hat, hat `spritecollide` fuer
//! eine reine Abfrage gehalten.

use rustgame_core::Rect;
use rustgame_math::{mask::from_surface, Mask};

use crate::gruppe::AbstractGroup;
use crate::sprite::Sprite;

/// Eine Funktion, die zwei Sprites auf Beruehrung prueft.
///
/// Das Gegenstueck zu pygames `collided`-Rueckruf. Alle Funktionen
/// dieses Moduls passen darauf - `collide_mask` ueber
/// [`collide_mask_trifft`].
pub type Kollision<'a> = &'a dyn Fn(&Sprite, &Sprite) -> bool;

// ---------------------------------------------------------------------------
//  Rechtecke
// ---------------------------------------------------------------------------

/// `collide_rect(a, b)` - beruehren sich die Rechtecke?
///
/// Ein Sprite ohne Rechteck beruehrt nichts (pygame wirft dort
/// `AttributeError`).
pub fn collide_rect(links: &Sprite, rechts: &Sprite) -> bool {
    match (links.rect(), rechts.rect()) {
        (Some(a), Some(b)) => a.colliderect(b),
        _ => false,
    }
}

/// `collide_rect_ratio(ratio)` - wie [`collide_rect`], aber mit
/// skalierten Rechtecken.
///
/// Skaliert wird ueber `inflate(w * ratio - w, h * ratio - h)`, und die
/// Differenz ist in pygame eine **Fliesskommazahl**, die `Rect.inflate`
/// zur Null hin abschneidet. Bei `w = 7` und `ratio = 1.3` sind das
/// `2.0999999999999996` -> `2`, nicht `3`. Genau das macht der Abgleich
/// unten nach.
pub fn collide_rect_ratio(ratio: f64) -> impl Fn(&Sprite, &Sprite) -> bool {
    move |links: &Sprite, rechts: &Sprite| {
        let (Some(a), Some(b)) = (links.rect(), rechts.rect()) else {
            return false;
        };
        aufblasen(a, ratio).colliderect(aufblasen(b, ratio))
    }
}

/// `rect.inflate(w * ratio - w, h * ratio - h)` mit pygames Rundung.
fn aufblasen(r: Rect, ratio: f64) -> Rect {
    let dw = r.w as f64 * ratio - r.w as f64;
    let dh = r.h as f64 * ratio - r.h as f64;
    // `Rect.inflate` nimmt die Argumente als C-`int` entgegen; Python
    // schneidet dabei zur Null hin ab. `as i32` in Rust tut dasselbe.
    r.inflate(dw as i32, dh as i32)
}

// ---------------------------------------------------------------------------
//  Kreise
// ---------------------------------------------------------------------------

/// Der Radius eines Sprites - aus `sprite.radius`, sonst aus dem
/// Rechteck, und dann **angeschrieben**.
fn radius_von(s: &Sprite) -> f64 {
    if let Some(r) = s.radius() {
        return r;
    }
    let rect = s.rect().unwrap_or(Rect::new(0, 0, 0, 0));
    let w = rect.w as f64;
    let h = rect.h as f64;
    // "half of the diagonal" - pygames eigener Kommentar warnt, dass das
    // bei langen schmalen Rechtecken zu grosszuegig ist.
    let r = 0.5 * (w * w + h * h).sqrt();
    s.set_radius(Some(r));
    r
}

/// Der Mittelpunktsabstand im Quadrat, in `f64`.
fn abstand2(links: &Sprite, rechts: &Sprite) -> f64 {
    let a = links.rect().unwrap_or(Rect::new(0, 0, 0, 0));
    let b = rechts.rect().unwrap_or(Rect::new(0, 0, 0, 0));
    let dx = (a.centerx() - b.centerx()) as f64;
    let dy = (a.centery() - b.centery()) as f64;
    dx * dx + dy * dy
}

/// `collide_circle(a, b)` - ueberlappen die umschliessenden Kreise?
///
/// **Verandert die Sprites**: ein fehlender Radius wird gerechnet und
/// abgelegt. Siehe Modulkopf.
pub fn collide_circle(links: &Sprite, rechts: &Sprite) -> bool {
    let d2 = abstand2(links, rechts);
    let rl = radius_von(links);
    let rr = radius_von(rechts);
    d2 <= (rl + rr) * (rl + rr)
}

/// `collide_circle_ratio(ratio)` - wie [`collide_circle`], aber die
/// Radien werden vorher mit `ratio` gestreckt.
///
/// Abgelegt wird der **ungeskalierte** Radius (gemessen).
pub fn collide_circle_ratio(ratio: f64) -> impl Fn(&Sprite, &Sprite) -> bool {
    move |links: &Sprite, rechts: &Sprite| {
        let d2 = abstand2(links, rechts);
        let rl = radius_von(links) * ratio;
        let rr = radius_von(rechts) * ratio;
        d2 <= (rl + rr) * (rl + rr)
    }
}

// ---------------------------------------------------------------------------
//  Masken
// ---------------------------------------------------------------------------

/// Die Maske eines Sprites - `sprite.mask`, sonst frisch aus dem Bild.
///
/// Sie wird **nicht** abgelegt (gemessen: nach `collide_mask` hat das
/// Sprite kein `mask`-Attribut). Bei jedem Aufruf entsteht also eine
/// neue - so steht es in pygame, und es ist der Grund, warum
/// `collide_mask` in Schleifen teuer ist.
fn maske_von(s: &Sprite) -> Option<Mask> {
    if let Some(m) = s.mask() {
        return Some(m);
    }
    // `pygame.mask.from_surface` mit dem Vorgabeschwellwert 127.
    s.image().map(|b| from_surface(&b, 127))
}

/// `collide_mask(a, b)` - der erste ueberlappende Punkt, oder `None`.
///
/// Der Punkt liegt in den Koordinaten der **linken** Maske, wie in
/// pygame. Fuer die Verwendung als Rueckruf gibt es
/// [`collide_mask_trifft`].
pub fn collide_mask(links: &Sprite, rechts: &Sprite) -> Option<(i32, i32)> {
    let (Some(a), Some(b)) = (links.rect(), rechts.rect()) else {
        return None;
    };
    let versatz = (b.x - a.x, b.y - a.y);
    let (ml, mr) = (maske_von(links)?, maske_von(rechts)?);
    ml.overlap(&mr, versatz)
}

/// [`collide_mask`] als Ja/Nein - so passt es auf [`Kollision`].
pub fn collide_mask_trifft(links: &Sprite, rechts: &Sprite) -> bool {
    collide_mask(links, rechts).is_some()
}

// ---------------------------------------------------------------------------
//  Sprite gegen Gruppe
// ---------------------------------------------------------------------------

/// `spritecollide(sprite, group, dokill, collided=None)`.
///
/// Sammelt alle Sprites der Gruppe, die `sprite` beruehren. Mit
/// `dokill` werden sie **danach** aus allen Gruppen genommen - sie
/// stehen trotzdem im Ergebnis.
pub fn spritecollide<G: AbstractGroup>(
    sprite: &Sprite,
    group: &G,
    dokill: bool,
    collided: Option<Kollision<'_>>,
) -> Vec<Sprite> {
    let getroffen: Vec<Sprite> = match collided {
        Some(f) => group
            .sprites()
            .into_iter()
            .filter(|g| f(sprite, g))
            .collect(),
        None => group
            .sprites()
            .into_iter()
            .filter(|g| collide_rect(sprite, g))
            .collect(),
    };
    if dokill {
        for g in &getroffen {
            g.kill();
        }
    }
    getroffen
}

/// `spritecollideany(sprite, group, collided=None)` - irgendeines, oder
/// `None`.
///
/// "Irgendeines" ist in Wahrheit **das erste in Gruppenreihenfolge** -
/// gemessen an einer Gruppe `[weit, nah1, nah2]`, die `nah1` liefert.
/// Die Gruppenreihenfolge ist die des Einhaengens.
pub fn spritecollideany<G: AbstractGroup>(
    sprite: &Sprite,
    group: &G,
    collided: Option<Kollision<'_>>,
) -> Option<Sprite> {
    for g in group.sprites() {
        let treffer = match collided {
            Some(f) => f(sprite, &g),
            None => collide_rect(sprite, &g),
        };
        if treffer {
            return Some(g);
        }
    }
    None
}

/// `groupcollide(groupa, groupb, dokilla, dokillb, collided=None)`.
///
/// Das Ergebnis ist pygames Woerterbuch als geordnete Liste - Pythons
/// Woerterbuecher merken sich die Einfuegereihenfolge, und die ist die
/// der Gruppe A.
///
/// Sprites aus A ohne Treffer stehen **nicht** darin. Zur Wirkung von
/// `dokillb` siehe den Modulkopf: sie greift schon waehrend des
/// Durchlaufs.
pub fn groupcollide<A: AbstractGroup, B: AbstractGroup>(
    groupa: &A,
    groupb: &B,
    dokilla: bool,
    dokillb: bool,
    collided: Option<Kollision<'_>>,
) -> Vec<(Sprite, Vec<Sprite>)> {
    let mut raus: Vec<(Sprite, Vec<Sprite>)> = Vec::new();
    for a in groupa.sprites() {
        let treffer = spritecollide(&a, groupb, dokillb, collided);
        if !treffer.is_empty() {
            raus.push((a, treffer));
        }
    }
    if dokilla {
        for (a, _) in &raus {
            a.kill();
        }
    }
    raus
}
