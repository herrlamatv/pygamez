//! `LayeredDirty` - zeichnet nur, was sich geaendert hat.
//!
//! Vorlage: `pygame/sprite.py`, Zeilen 1059-1394.
//!
//! ## Zwei Betriebsarten, und der Umschalter dazwischen
//!
//! `LayeredDirty` faehrt entweder **Vollbild** (alles neu) oder
//! **Teilbild** (nur die schmutzigen Rechtecke). Welche, entscheidet
//! `_use_update`, und das setzt sich nach jedem `draw` selbst neu: hat
//! das Zeichnen laenger gedauert als `_time_threshold` (Vorgabe `1000/80
//! = 12,5` ms), geht es zurueck auf Vollbild, sonst auf Teilbild.
//!
//! **Eine frische Gruppe faengt im Vollbildmodus an** (`_use_update =
//! False`). Der erste `draw` liefert deshalb immer genau ein Rechteck -
//! den Clip-Bereich -, und erst der zweite die einzelnen schmutzigen
//! Stellen. Gemessen:
//!
//! ```text
//! 1. draw -> [Rect(0, 0, 100, 100)]
//! 2. draw -> [Rect(40, 40, 10, 10), Rect(0, 0, 15, 15)]
//! ```
//!
//! ## `dirty == 1` gegen `dirty == 2`
//!
//! * `1` heisst "einmal neu zeichnen"; danach setzt
//!   `_draw_dirty_internal` das Flag auf `0` zurueck.
//! * `2` heisst "immer neu zeichnen"; das Flag bleibt stehen.
//! * `0` heisst "sauber" - das Sprite wird trotzdem angefasst, aber nur
//!   in den Ausschnitten, die andere Sprites schmutzig gemacht haben.
//!   Genau das ist der Gewinn dieser Gruppe.
//!
//! **Das Ueberraschende:** im **Vollbildmodus** werden die Flaggen
//! **nicht** zurueckgesetzt. Nachgemessen - nach dem ersten `draw`
//! stehen alle drei Sprites weiterhin auf `dirty == 1`. Nur der Weg
//! ueber `_use_update` raeumt auf. Wer das uebersieht, baut eine
//! Nachbildung, bei der der zweite `draw` nichts mehr findet.
//!
//! ## Die Rechteckverschmelzung
//!
//! `_find_dirty_area` sammelt je schmutzigem Sprite **zwei** Rechtecke:
//! wo es jetzt ist und wo es zuletzt gezeichnet wurde. Jedes wird vorher
//! mit allen schon gesammelten verschmolzen, die es beruehrt - in einer
//! Schleife, weil die Vereinigung neue Beruehrungen schaffen kann. Das
//! ist pygames eigener Kommentar: *"can it be done better? because that
//! is an O(n**2) algorithm in worst case"*. Hier steht derselbe
//! Algorithmus, denn die **Reihenfolge und Anzahl** der Rechtecke ist
//! beobachtbar - sie ist der Rueckgabewert.

use rustgame_core::time::get_ticks_f64;
use rustgame_core::{Rect, Surface};

use crate::gruppe::{
    blit_sprite, gruppen_rumpf, wahr, AbstractGroup, Art, GruppenHandle, GruppenKern,
    Hinzufuegbar, Verloren,
};
use crate::layered::EbenenGruppe;
use crate::sprite::Sprite;

/// Gruppe fuer [`DirtySprite`](crate::DirtySprite), die nur die
/// veraenderten Bereiche neu zeichnet - `pygame.sprite.LayeredDirty`.
///
/// Nimmt **nur** DirtySprites auf; alles andere wird still uebersprungen
/// (pygame wirft dort `AttributeError` beziehungsweise `TypeError`).
#[derive(Clone)]
pub struct LayeredDirty(GruppenHandle);
gruppen_rumpf!(LayeredDirty, Art::EbenenDirty);

impl EbenenGruppe for LayeredDirty {}

impl LayeredDirty {
    /// Eine leere Gruppe mit eigenen Einstellungen - die kwargs aus
    /// `LayeredDirty(**kwargs)`.
    pub fn mit_einstellungen(
        default_layer: i32,
        use_update: bool,
        time_threshold: f64,
    ) -> LayeredDirty {
        let g = LayeredDirty::new();
        {
            let mut k = g.kern().borrow_mut();
            k.default_layer = default_layer;
            k.use_update = use_update;
            k.time_threshold = time_threshold;
        }
        g
    }

    /// `_use_update` - `false` heisst Vollbild, `true` Teilbild.
    pub fn use_update(&self) -> bool {
        self.kern().borrow().use_update
    }

    /// Setzt `_use_update`.
    ///
    /// In pygame gibt es dafuer nur den Konstruktor-kwarg; `draw` setzt
    /// den Wert danach selbst nach der gemessenen Dauer. Fuer Tests und
    /// fuer alles, was reproduzierbar sein soll, ist ein Setzer noetig -
    /// die gemessene Dauer ist auf einem schnellen Rechner immer
    /// dieselbe Antwort.
    pub fn set_use_update(&self, wert: bool) {
        self.kern().borrow_mut().use_update = wert;
    }

    /// `_time_threshold` in Millisekunden.
    pub fn time_threshold(&self) -> f64 {
        self.kern().borrow().time_threshold
    }

    /// `set_timing_threshold(ms)`.
    pub fn set_timing_threshold(&self, ms: f64) {
        self.kern().borrow_mut().time_threshold = ms;
    }

    /// `group.get_clip()`.
    pub fn get_clip(&self) -> Option<Rect> {
        self.kern().borrow().clip
    }

    /// `group.set_clip(rect)` - beschraenkt das Zeichnen auf diesen
    /// Bereich und faellt auf Vollbild zurueck.
    ///
    /// **Abweichung:** pygames `set_clip(None)` holt sich
    /// `pygame.display.get_surface().get_rect()` und legt **das** ab.
    /// Hier heisst `None` "kein eigener Clip", und `draw` nimmt dann den
    /// Clip der Zielflaeche. Beim ueblichen Aufruf - Zeichnen auf die
    /// Fensterflaeche, deren Clip ihr volles Rechteck ist - kommt
    /// dasselbe heraus, ohne dass diese Kiste vom Fenstermodul abhaengt.
    /// Sichtbar wird der Unterschied nur an [`LayeredDirty::get_clip`].
    pub fn set_clip(&self, rect: Option<Rect>) {
        let mut k = self.kern().borrow_mut();
        k.clip = rect;
        k.use_update = false;
    }

    /// `group.repaint_rect(rect)` - diesen Bereich beim naechsten `draw`
    /// neu zeichnen.
    ///
    /// Auf den eigenen Clip beschnitten, falls einer gesetzt ist
    /// (gemessen: `repaint_rect(0,0,100,100)` bei Clip `(10,10,30,30)`
    /// landet als `(10,10,30,30)` in `lostsprites`).
    pub fn repaint_rect(&self, rect: Rect) {
        let mut k = self.kern().borrow_mut();
        let r = match k.clip {
            Some(c) if wahr(c) => rect.clip(c),
            _ => rect,
        };
        k.lostsprites.push(Verloren::Fest(r));
    }

    /// `group.clear(surface, bgd)` - legt **nur** den Hintergrund ab.
    ///
    /// Die Flaeche wird nicht angefasst; `LayeredDirty` malt den
    /// Hintergrund erst beim naechsten `draw` an den richtigen Stellen
    /// auf. Das ist pygames Fassung, die die geerbte Bedeutung von
    /// `clear` vollstaendig ersetzt.
    pub fn set_hintergrund(&self, bgd: Option<Surface>) {
        self.kern().borrow_mut().bgd = bgd;
    }

    /// Der abgelegte Hintergrund.
    pub fn hintergrund(&self) -> Option<Surface> {
        self.kern().borrow().bgd.clone()
    }

    /// `group.draw(surface, bgd=None, special_flags=None)` - die volle
    /// Fassung.
    ///
    /// * `bgd`: ein Hintergrund, der zugleich abgelegt wird.
    /// * `special_flags`: `None` heisst "je Sprite `blendmode` nehmen",
    ///   `Some(f)` setzt alle Sprites auf `f`. Gemessen, dass `Some(0)`
    ///   einen gesetzten `blendmode` wirklich ueberstimmt.
    pub fn draw_voll(
        &self,
        surface: &mut Surface,
        bgd: Option<&Surface>,
        special_flags: Option<u32>,
    ) -> Vec<Rect> {
        self.kern().borrow_mut().zeichnen(surface, bgd, special_flags)
    }
}

// ---------------------------------------------------------------------------
//  Der Zeichenweg
// ---------------------------------------------------------------------------

/// `LayeredDirty.draw`.
pub(crate) fn zeichnen_dirty(
    k: &mut GruppenKern,
    surface: &mut Surface,
    bgd: Option<&Surface>,
    special_flags: Option<u32>,
) -> Vec<Rect> {
    let orig_clip = surface.get_clip();
    let latest_clip = k.clip.unwrap_or(orig_clip);

    // Ein uebergebener Hintergrund wird abgelegt - danach wirkt er auch
    // bei allen weiteren Aufrufen, und ein spaeteres `bgd` hat keine
    // Wirkung mehr, solange keines abgelegt ist. (pygames Text sagt es
    // andersherum, der Code tut dies.)
    if let Some(b) = bgd {
        k.bgd = Some(b.clone());
    }
    let local_bgd = k.bgd.clone();

    surface.set_clip(Some(latest_clip));
    let start = get_ticks_f64();

    let ergebnis = if k.use_update {
        // pygames `local_update` **ist** `self.lostsprites`. Hier wird
        // die Liste einmal aufgeloest (die Eintraege koennen auf
        // lebende Sprite-Rechtecke zeigen) und danach oertlich gefuehrt;
        // am Ende wird sie ohnehin geleert, also kommt dasselbe heraus.
        let mut update = k.lostsprites_aufloesen();

        // 1. Wo ist es schmutzig?
        finde_dirty(k, latest_clip, &mut update);

        // 2. Dort den Hintergrund auftragen.
        if let Some(b) = &local_bgd {
            let flags = special_flags.unwrap_or(0);
            for r in &update {
                surface.blit_ex(b, (r.x, r.y), Some(*r), flags);
            }
        }

        // 3. Zeichnen.
        zeichne_dirty_intern(k, surface, special_flags, &update);
        update
    } else {
        if let Some(b) = &local_bgd {
            let flags = special_flags.unwrap_or(0);
            surface.blit_ex(b, (0, 0), None, flags);
        }
        for s in k.reihenfolge.clone() {
            if s.visible().unwrap_or(1) != 0 {
                let flags = special_flags.unwrap_or_else(|| s.blendmode().unwrap_or(0));
                if let Some(r) = blit_sprite(surface, &s, s.source_rect(), flags) {
                    k.setze_rect(&s, Some(r));
                }
            }
        }
        // **Kein Zuruecksetzen der dirty-Flaggen** - gemessen.
        vec![latest_clip]
    };

    let ende = get_ticks_f64();
    k.use_update = ende - start <= k.time_threshold;

    k.lostsprites.clear();
    surface.set_clip(Some(orig_clip));
    ergebnis
}

/// Verschmilzt `u` mit allen Rechtecken aus `update`, die es beruehrt,
/// und nimmt diese dabei heraus.
///
/// Die Schleife ist noetig, weil jede Vereinigung `u` vergroessert und
/// damit neue Beruehrungen schaffen kann.
fn verschmelze(u: &mut Rect, update: &mut Vec<Rect>) {
    while let Some(i) = u.collidelist(update) {
        u.union_ip(update[i]);
        update.remove(i);
    }
}

/// `_find_dirty_area`.
fn finde_dirty(k: &mut GruppenKern, clip: Rect, update: &mut Vec<Rect>) {
    for s in k.reihenfolge.clone() {
        if s.dirty().unwrap_or(0) <= 0 {
            continue;
        }
        let Some(rect) = s.rect() else { continue };

        // `if spr.source_rect:` - Wahrheit, nicht `is not None`. Ein
        // leerer Ausschnitt zaehlt hier also als "keiner".
        let mut u = match s.source_rect() {
            Some(sr) if wahr(sr) => Rect::from_parts(rect.topleft(), (sr.w, sr.h)),
            _ => rect,
        };
        verschmelze(&mut u, update);
        update.push(u.clip(clip));

        // Und dort, wo es zuletzt stand.
        if let Some(alt) = k.rect_von(&s) {
            let mut u = alt;
            verschmelze(&mut u, update);
            update.push(u.clip(clip));
        }
    }
}

/// `_draw_dirty_internal`.
fn zeichne_dirty_intern(
    k: &mut GruppenKern,
    surface: &mut Surface,
    special_flags: Option<u32>,
    update: &[Rect],
) {
    for s in k.reihenfolge.clone() {
        let flags = special_flags.unwrap_or_else(|| s.blendmode().unwrap_or(0));
        let dirty = s.dirty().unwrap_or(0);
        let sichtbar = s.visible().unwrap_or(1) != 0;

        if dirty < 1 && sichtbar {
            // Sauberes Sprite: nur die Stuecke auffrischen, die andere
            // schmutzig gemacht haben.
            let (Some(bild), Some(rect)) = (s.image(), s.rect()) else {
                continue;
            };
            // Hier prueft pygame `is not None`, nicht die Wahrheit -
            // anders als in `_find_dirty_area` zwei Bildschirme weiter
            // oben. Der Unterschied ist echt und bleibt stehen.
            let (spr_rect, off_x, off_y) = match s.source_rect() {
                Some(sr) => {
                    let r = Rect::from_parts(rect.topleft(), (sr.w, sr.h));
                    (r, sr.x - r.x, sr.y - r.y)
                }
                None => (rect, -rect.x, -rect.y),
            };
            for idx in spr_rect.collidelistall(update) {
                let c = spr_rect.clip(update[idx]);
                let quelle = Rect::new(c.x + off_x, c.y + off_y, c.w, c.h);
                surface.blit_ex(&bild, (c.x, c.y), Some(quelle), flags);
            }
        } else {
            if sichtbar {
                if let Some(r) = blit_sprite(surface, &s, s.source_rect(), flags) {
                    k.setze_rect(&s, Some(r));
                }
            }
            // Auch ein unsichtbares Sprite wird sauber gemeldet -
            // gemessen: nach `visible = 0` faellt `dirty` von 1 auf 0,
            // ohne dass irgendetwas gezeichnet wurde.
            if dirty == 1 {
                s.set_dirty(0);
            }
        }
    }
}
