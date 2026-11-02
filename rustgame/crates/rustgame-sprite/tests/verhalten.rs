//! Festgenagelte Einzelantworten aus echtem pygame-ce 2.5.7.
//!
//! Der Zufallsabgleich in `gegen_pygame.rs` deckt die Masse ab; hier
//! stehen die **benannten** Faelle - jeder mit dem gemessenen Wert im
//! Kommentar. Wer eine dieser Zeilen aendern will, muss erst pygame
//! umschreiben.
//!
//! Gemessen mit:
//!
//! ```text
//! cd C:\Users\herrl\PycharmProjects\pygamez
//! set SDL_VIDEODRIVER=dummy
//! .venv\Scripts\python.exe -c "import pygame; pygame.init(); ..."
//! ```

use std::any::Any;

use rustgame_core::{Rect, Surface};
use rustgame_sprite::*;

// ---------------------------------------------------------------------------
//  Kleinkram
// ---------------------------------------------------------------------------

fn spr(x: i32, y: i32, w: i32, h: i32) -> Sprite {
    let mut bild = Surface::new((w as u32, h as u32));
    bild.fill((255u8, 0u8, 0u8, 255u8));
    let s = Sprite::new();
    s.set_image(Some(bild));
    s.set_rect(Some(Rect::new(x, y, w, h)));
    s
}

fn dspr(x: i32, y: i32, w: i32, h: i32) -> DirtySprite {
    let mut bild = Surface::new((w as u32, h as u32));
    bild.fill((0u8, 255u8, 0u8, 255u8));
    let d = DirtySprite::new();
    d.set_image(Some(bild));
    d.set_rect(Some(Rect::new(x, y, w, h)));
    d
}

/// Die RGB-Anteile eines Pixels - der Alphakanal ist auf einer
/// undurchsichtigen Flaeche ohnehin immer 255.
fn farbe(s: &Surface, p: (i32, i32)) -> (u8, u8, u8) {
    let c = s.get_at(p);
    (c.r, c.g, c.b)
}

fn wer(gruppe: &impl AbstractGroup, alle: &[Sprite]) -> Vec<usize> {
    gruppe
        .sprites()
        .iter()
        .map(|s| alle.iter().position(|a| a.ist(s)).unwrap())
        .collect()
}

// ---------------------------------------------------------------------------
//  Reihenfolge und Mitgliedschaft
// ---------------------------------------------------------------------------

/// `Group.sprites()` ist geordnet - auch wenn pygames eigener Text
/// "in no particular order" behauptet.
///
/// Gemessen: `Group(a,b,c); add(d)` -> `[a,b,c,d]`;
/// danach `remove(b); add(b)` -> `[a,c,d,b]`.
#[test]
fn group_sprites_reihenfolge() {
    let alle: Vec<Sprite> = (0..4).map(|i| spr(i, 0, 4, 4)).collect();
    let g = Group::mit(alle[..3].to_vec());
    assert_eq!(wer(&g, &alle), vec![0, 1, 2]);
    g.add(&alle[3]);
    assert_eq!(wer(&g, &alle), vec![0, 1, 2, 3]);
    g.remove(&alle[1]);
    g.add(&alle[1]);
    assert_eq!(wer(&g, &alle), vec![0, 2, 3, 1], "b wandert ans Ende");
}

/// Ein Sprite zweimal einhaengen tut nichts.
///
/// Gemessen: `len` bleibt 3, und `a.groups()` waechst nicht.
#[test]
fn doppeltes_add_tut_nichts() {
    let a = spr(0, 0, 4, 4);
    let g1 = Group::mit(&a);
    let g2 = Group::mit(&a);
    assert_eq!(a.anzahl_gruppen(), 2);
    g1.add(&a);
    g1.add(&a);
    assert_eq!(g1.len(), 1);
    assert_eq!(a.anzahl_gruppen(), 2, "kein Doppeleintrag im Sprite");
    assert!(g2.has(&a));
}

/// `has()` ohne Argumente ist `false` - pygames erste Zeile.
#[test]
fn has_ohne_argumente() {
    let a = spr(0, 0, 4, 4);
    let b = spr(9, 9, 4, 4);
    let g = Group::mit(&a);
    let leer: Vec<Sprite> = Vec::new();
    assert!(g.has(&a));
    assert!(!g.has(&b));
    assert!(!g.has(&leer), "has() ohne Sprites -> false");
    assert!(!g.has(vec![a.clone(), b.clone()]), "has(a,b) -> false");
    assert!(g.has(vec![a.clone()]));
}

// ---------------------------------------------------------------------------
//  kill, Lebensdauer, Zerfall
// ---------------------------------------------------------------------------

/// `kill()` nimmt aus allen Gruppen - und das Sprite bleibt benutzbar.
///
/// Gemessen: nach `kill` sind beide Gruppen leer, `alive()` ist `False`,
/// und ein erneutes `add` geht anstandslos.
#[test]
fn kill_und_wiederbelebung() {
    let s = spr(0, 0, 4, 4);
    let ga = Group::mit(&s);
    let gb = Group::mit(&s);
    assert_eq!(s.anzahl_gruppen(), 2);
    s.kill();
    assert!(!s.alive());
    assert_eq!((ga.len(), gb.len()), (0, 0));
    ga.add(&s);
    assert!(s.alive());
    assert_eq!(ga.len(), 1);
}

/// `group.empty()` leert die Gruppe und die Sprites wissen davon.
#[test]
fn empty_raeumt_beide_buecher() {
    let a = spr(0, 0, 4, 4);
    let b = spr(5, 5, 4, 4);
    let g = Group::mit(vec![a.clone(), b.clone()]);
    g.empty();
    assert_eq!(g.len(), 0);
    assert!(!a.alive() && !b.alive());
}

/// **Die eine bewusste Abweichung.**
///
/// In pygame halten Sprite und Gruppe einander stark; faellt die letzte
/// aeussere Variable der Gruppe weg, lebt sie weiter und `groups()`
/// zaehlt sie mit - gemessen: `1`.
///
/// Hier haelt das Sprite die Gruppe nur schwach. Die Gruppe wird
/// freigegeben, `groups()` zaehlt `0`. Das ist der Preis dafuer, dass
/// kein Zyklus leckt, und er steht so im Modulkopf.
#[test]
fn gruppe_faellt_weg_sprite_ueberlebt() {
    let s = spr(0, 0, 4, 4);
    {
        let g = Group::mit(&s);
        assert_eq!(s.anzahl_gruppen(), 1);
        drop(g);
    }
    assert_eq!(
        s.anzahl_gruppen(),
        0,
        "pygame meldet hier 1 - siehe Modulkopf"
    );
    assert!(!s.alive());
    // Und das Sprite ist heil: es laesst sich wieder einhaengen.
    let g2 = Group::mit(&s);
    assert_eq!(g2.len(), 1);
}

/// Andersherum: die Gruppe haelt das Sprite am Leben.
#[test]
fn gruppe_haelt_sprite() {
    let g = Group::new();
    let nummer = {
        let s = spr(1, 2, 3, 4);
        g.add(&s);
        s.nummer()
    };
    assert_eq!(g.len(), 1);
    assert_eq!(g.sprites()[0].nummer(), nummer);
    assert_eq!(g.sprites()[0].rect(), Some(Rect::new(1, 2, 3, 4)));
}

// ---------------------------------------------------------------------------
//  GroupSingle
// ---------------------------------------------------------------------------

/// Der Neue wirft den Alten **ganz** hinaus.
///
/// Gemessen: nach `gs.add(x2)` ist `x1.alive()` `False` und
/// `len(x1.groups())` `0` - nicht nur "aus dieser Gruppe raus".
#[test]
fn groupsingle_verdraengt() {
    let x1 = spr(0, 0, 4, 4);
    let x2 = spr(5, 5, 4, 4);
    let gs = GroupSingle::mit(&x1);
    assert!(gs.sprite().unwrap().ist(&x1));
    gs.add(&x2);
    assert_eq!(gs.len(), 1);
    assert!(gs.sprite().unwrap().ist(&x2));
    assert!(!x1.alive());
    assert_eq!(x1.anzahl_gruppen(), 0);

    gs.set_sprite(&x1);
    assert!(gs.sprite().unwrap().ist(&x1));
    assert!(!x2.alive());
    assert_eq!(x1.anzahl_gruppen(), 1);
}

/// `GroupSingle.add_internal` schreibt **nicht** ins `spritedict` -
/// erst `draw` legt dort einen Eintrag an.
///
/// Gemessen: direkt nach `add` ist `spritedict` `{}`, nach `draw`
/// `{y: Rect(0, 0, 10, 10)}`.
#[test]
fn groupsingle_spritedict_erst_nach_draw() {
    let y = spr(0, 0, 10, 10);
    let gs = GroupSingle::mit(&y);
    assert!(gs.spritedict().is_empty());
    let mut flaeche = Surface::new((40, 40));
    gs.draw(&mut flaeche);
    let d = gs.spritedict();
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].1, Some(Rect::new(0, 0, 10, 10)));
}

// ---------------------------------------------------------------------------
//  draw
// ---------------------------------------------------------------------------

/// **`Group.draw()` gibt immer `[]` zurueck.**
///
/// Der Quelltext sieht so aus, als sammelte er etwas:
/// `self.lostsprites = []; dirty = self.lostsprites; return dirty` -
/// die neue Liste ist leer und bleibt es. Gemessen im ersten wie im
/// zweiten Aufruf.
#[test]
fn group_draw_gibt_immer_leer() {
    let mut flaeche = Surface::new((100, 100));
    let g = Group::mit(vec![spr(5, 5, 10, 10), spr(20, 20, 10, 10)]);
    assert_eq!(g.draw(&mut flaeche), Vec::<Rect>::new());
    assert_eq!(g.draw(&mut flaeche), Vec::<Rect>::new());
    // Gemalt wird trotzdem, und das spritedict wird gefuehrt.
    let d = g.spritedict();
    assert_eq!(d[0].1, Some(Rect::new(5, 5, 10, 10)));
    assert_eq!(d[1].1, Some(Rect::new(20, 20, 10, 10)));
}

/// `RenderUpdates.draw` - Vereinigung bei Beruehrung, sonst beide.
///
/// Gemessen an einem 10x10-Sprite bei `(5,5)`:
/// * 1. Aufruf: `[Rect(5,5,10,10), Rect(20,20,10,10)]`
/// * unbewegt: dasselbe
/// * nach `+3` in x: `[Rect(5,5,13,10), ...]`
/// * nach `+80`: `[Rect(88,5,10,10), Rect(8,5,10,10), ...]`
#[test]
fn renderupdates_draw_rechtecke() {
    let mut flaeche = Surface::new((100, 100));
    let a = spr(5, 5, 10, 10);
    let b = spr(20, 20, 10, 10);
    let g = RenderUpdates::mit(vec![a.clone(), b.clone()]);

    assert_eq!(
        g.draw(&mut flaeche),
        vec![Rect::new(5, 5, 10, 10), Rect::new(20, 20, 10, 10)]
    );
    assert_eq!(
        g.draw(&mut flaeche),
        vec![Rect::new(5, 5, 10, 10), Rect::new(20, 20, 10, 10)]
    );
    a.bewege(3, 0);
    assert_eq!(
        g.draw(&mut flaeche),
        vec![Rect::new(5, 5, 13, 10), Rect::new(20, 20, 10, 10)],
        "Ueberlappung -> Vereinigung"
    );
    a.bewege(80, 0);
    assert_eq!(
        g.draw(&mut flaeche),
        vec![
            Rect::new(88, 5, 10, 10),
            Rect::new(8, 5, 10, 10),
            Rect::new(20, 20, 10, 10)
        ],
        "kein Beruehren -> neu und alt einzeln"
    );
}

/// `clear` malt erst die verlorenen, dann die zuletzt gezeichneten
/// Stellen - und die verlorenen zaehlen erst, wenn wirklich gezeichnet
/// wurde.
///
/// Gemessen: `add + remove` ohne `draw` laesst `lostsprites` leer;
/// `add + draw + remove` legt `Rect(2,2,10,10)` hinein.
#[test]
fn clear_reihenfolge_und_lostsprites() {
    let mut flaeche = Surface::new((60, 60));
    flaeche.fill((0u8, 0u8, 0u8, 255u8));
    let mut bgd = Surface::new((60, 60));
    bgd.fill((10u8, 20u8, 30u8, 255u8));

    let t = spr(2, 2, 10, 10);
    let g = Group::mit(&t);
    g.remove(&t);
    assert!(g.lostsprites().is_empty(), "ohne draw kein verlorenes Rechteck");

    g.add(&t);
    g.draw(&mut flaeche);
    assert_eq!(farbe(&flaeche, (3, 3)), (255, 0, 0));
    g.remove(&t);
    assert_eq!(g.lostsprites(), vec![Rect::new(2, 2, 10, 10)]);

    g.clear(&mut flaeche, &bgd);
    assert_eq!(farbe(&flaeche, (3, 3)), (10, 20, 30));

    // Die Reihenfolge: verlorene zuerst.
    let sx = spr(1, 1, 4, 4);
    let sy = spr(20, 20, 4, 4);
    let g2 = Group::mit(vec![sx.clone(), sy.clone()]);
    g2.draw(&mut flaeche);
    g2.remove(&sx);
    let mut gesehen: Vec<Rect> = Vec::new();
    g2.clear_mit(&mut flaeche, |_, r| gesehen.push(r));
    assert_eq!(gesehen, vec![Rect::new(1, 1, 4, 4), Rect::new(20, 20, 4, 4)]);
}

// ---------------------------------------------------------------------------
//  update
// ---------------------------------------------------------------------------

struct Merker {
    gesehen: Vec<i32>,
}

impl SpriteVerhalten for Merker {
    fn update(&mut self, s: &Sprite, args: &dyn Any) {
        // Die Argumente kommen unveraendert durch - pygames
        // `sprite.update(*args, **kwargs)`.
        if let Some(n) = args.downcast_ref::<i32>() {
            self.gesehen.push(*n);
            s.bewege(*n, 0);
        } else {
            self.gesehen.push(-1);
        }
    }
}

/// `Group.update(*args)` reicht die Argumente unveraendert durch.
///
/// Gemessen: `update(1, 'zwei', drei=3)` dann `update()` liefert dem
/// Sprite `[((1, 'zwei'), {'drei': 3}), ((), {})]`.
#[test]
fn update_reicht_argumente_durch() {
    let s = spr(0, 0, 4, 4);
    s.set_verhalten(Merker { gesehen: vec![] });
    let g = Group::mit(&s);

    g.update_mit(&7i32);
    g.update();
    g.update_mit(&5i32);

    assert_eq!(
        s.mit_zustand(|m: &Merker| m.gesehen.clone()),
        Some(vec![7, -1, 5])
    );
    assert_eq!(s.rect().unwrap().x, 12, "7 + 5 bewegt");
}

/// `update` darf das Sprite toeten, obwohl der Kasten aus dem Sprite
/// kommt - der Kasten ist waehrend des Aufrufs herausgenommen.
#[test]
fn update_darf_sich_selbst_toeten() {
    struct Selbstmoerder;
    impl SpriteVerhalten for Selbstmoerder {
        fn update(&mut self, s: &Sprite, _: &dyn Any) {
            s.set_rect(Some(Rect::new(1, 1, 1, 1)));
            s.kill();
        }
    }
    let s = spr(0, 0, 4, 4);
    s.set_verhalten(Selbstmoerder);
    let g = Group::mit(&s);
    g.update();
    assert!(!s.alive());
    assert_eq!(g.len(), 0);
    assert_eq!(s.rect(), Some(Rect::new(1, 1, 1, 1)));
}

// ---------------------------------------------------------------------------
//  LayeredUpdates
// ---------------------------------------------------------------------------

/// Einsortiert wird **hinter** allen mit gleicher oder kleinerer Ebene.
///
/// Gemessen: `l1@5, l2@1, l3@5` -> `[l2, l1, l3]`, `layers() == [1, 5]`.
#[test]
fn layered_reihenfolge_und_ebenen() {
    let alle: Vec<Sprite> = (0..3).map(|i| spr(i, 0, 4, 4)).collect();
    let lu = LayeredUpdates::new();
    lu.add_mit_ebene(&alle[0], 5);
    lu.add_mit_ebene(&alle[1], 1);
    lu.add_mit_ebene(&alle[2], 5);
    assert_eq!(wer(&lu, &alle), vec![1, 0, 2]);
    assert_eq!(lu.layers(), vec![1, 5]);
    assert_eq!(lu.get_layer_of_sprite(&alle[0]), 5);
    assert_eq!(lu.get_layer_of_sprite(&alle[1]), 1);
}

/// **Das ausdrueckliche `layer=` bleibt nicht am Sprite haengen** -
/// jedenfalls nicht an einem einfachen `Sprite`.
///
/// Gemessen: nach `lu.add(s, layer=5)` wirft `s._layer` einen
/// `AttributeError`. Ein `DirtySprite` bringt `_layer = 0` mit, dort
/// wird die 6 angeschrieben.
#[test]
fn layer_kwarg_schreibt_nur_vorhandenes_attribut_an() {
    let einfach = spr(0, 0, 4, 4);
    let lu = LayeredUpdates::new();
    lu.add_mit_ebene(&einfach, 5);
    assert_eq!(lu.get_layer_of_sprite(&einfach), 5);
    assert_eq!(einfach.layer(), None, "kein _layer entstanden");

    let d = dspr(0, 0, 4, 4);
    assert_eq!(d.layer(), Some(0), "DirtySprite bringt _layer = 0 mit");
    let lu2 = LayeredUpdates::new();
    lu2.add_mit_ebene(d.as_sprite(), 6);
    assert_eq!(d.layer(), Some(6));
}

/// Ohne `layer=` bekommt ein Sprite ohne `_layer` die Vorgabeebene
/// **angeschrieben**.
///
/// Gemessen mit `default_layer=7`: Ebene 7, und `s._layer == 7`.
#[test]
fn vorgabeebene_wird_angeschrieben() {
    let s = spr(0, 0, 4, 4);
    let lu = LayeredUpdates::mit_default_layer(7);
    lu.add(&s);
    assert_eq!(lu.get_layer_of_sprite(&s), 7);
    assert_eq!(s.layer(), Some(7));
}

/// Ein vorher gesetztes `sprite.layer` gewinnt - und danach ist es
/// gesperrt.
///
/// Gemessen: `s.layer = 3` vor dem Einhaengen -> Ebene 3; danach wirft
/// `s.layer = 4` einen `AttributeError`.
#[test]
fn sprite_layer_vorher_setzen_danach_gesperrt() {
    let s = spr(0, 0, 4, 4);
    assert!(s.set_layer(3).is_ok());
    let lu = LayeredUpdates::mit(&s);
    assert_eq!(lu.get_layer_of_sprite(&s), 3);
    assert_eq!(s.set_layer(4), Err(LayerFehler));
    assert_eq!(s.layer(), Some(3));
}

/// `get_layer_of_sprite` auf ein fremdes Sprite liefert die
/// Vorgabeebene; `change_layer` dagegen scheitert.
///
/// Gemessen: `0` beziehungsweise `ValueError: list.remove(x): x not in
/// list`.
#[test]
fn fremdes_sprite_in_ebenengruppe() {
    let drin = spr(0, 0, 4, 4);
    let fremd = spr(9, 9, 4, 4);
    let lu = LayeredUpdates::mit(&drin);
    assert_eq!(lu.get_layer_of_sprite(&fremd), 0, "Vorgabeebene, kein Fehler");
    assert_eq!(lu.change_layer(&fremd, 9), Err(NichtInGruppe));
    assert_eq!(lu.change_layer(&drin, 9), Ok(()));
    assert_eq!(lu.get_layer_of_sprite(&drin), 9);
}

/// `move_to_front` / `move_to_back`.
///
/// Gemessen an `[f1@0, f2@1, f3@1]`:
/// `move_to_front(f1)` -> `[f2, f3, f1]`, f1 auf Ebene 1;
/// danach `move_to_back(f1)` -> `[f1, f2, f3]`, f1 auf Ebene 0.
#[test]
fn nach_vorn_und_nach_hinten() {
    let alle: Vec<Sprite> = (0..3).map(|i| spr(i, 0, 4, 4)).collect();
    let lf = LayeredUpdates::new();
    lf.add_mit_ebene(&alle[0], 0);
    lf.add_mit_ebene(&alle[1], 1);
    lf.add_mit_ebene(&alle[2], 1);

    lf.move_to_front(&alle[0]).unwrap();
    assert_eq!(wer(&lf, &alle), vec![1, 2, 0]);
    assert_eq!(lf.get_layer_of_sprite(&alle[0]), 1);

    lf.move_to_back(&alle[0]).unwrap();
    assert_eq!(wer(&lf, &alle), vec![0, 1, 2]);
    assert_eq!(lf.get_layer_of_sprite(&alle[0]), 0);

    assert_eq!(lf.get_top_layer(), Some(1));
    assert_eq!(lf.get_bottom_layer(), Some(0));
    assert!(lf.get_top_sprite().unwrap().ist(&alle[2]));
}

/// `switch_layer(1, 2)` tauscht die Bewohner.
///
/// Gemessen an `q1@1, q2@2, q3@1` -> `[q2, q1, q3]` mit
/// `q1=2, q2=1, q3=2`.
#[test]
fn switch_layer() {
    let alle: Vec<Sprite> = (0..3).map(|i| spr(i, 0, 4, 4)).collect();
    let ls = LayeredUpdates::new();
    ls.add_mit_ebene(&alle[0], 1);
    ls.add_mit_ebene(&alle[1], 2);
    ls.add_mit_ebene(&alle[2], 1);
    ls.switch_layer(1, 2);
    assert_eq!(wer(&ls, &alle), vec![1, 0, 2]);
    assert_eq!(ls.get_layer_of_sprite(&alle[0]), 2);
    assert_eq!(ls.get_layer_of_sprite(&alle[1]), 1);
    assert_eq!(ls.get_layer_of_sprite(&alle[2]), 2);
}

/// `get_sprites_at` - unterstes zuerst.
///
/// Gemessen an `a1(0,0,20,20)`, `a2(10,10,20,20)`, `a3(50,50,5,5)`:
/// `(15,15)` -> `[a1, a2]`, `(5,5)` -> `[a1]`, `(99,99)` -> `[]`.
#[test]
fn get_sprites_at() {
    let alle = vec![spr(0, 0, 20, 20), spr(10, 10, 20, 20), spr(50, 50, 5, 5)];
    let la = LayeredUpdates::mit(&alle);
    let namen = |p| {
        la.get_sprites_at(p)
            .iter()
            .map(|s| alle.iter().position(|a| a.ist(s)).unwrap())
            .collect::<Vec<_>>()
    };
    assert_eq!(namen((15, 15)), vec![0, 1]);
    assert_eq!(namen((5, 5)), vec![0]);
    assert_eq!(namen((99, 99)), Vec::<usize>::new());
}

/// `remove_sprites_of_layer` gibt sie zurueck **und** toetet sie.
///
/// Gemessen: `[r1, r3]` zurueck, `[r2]` bleibt, `r1.alive()` ist
/// `False`.
#[test]
fn remove_sprites_of_layer() {
    let alle: Vec<Sprite> = (0..3).map(|i| spr(i, 0, 4, 4)).collect();
    let lr = LayeredUpdates::new();
    lr.add_mit_ebene(&alle[0], 1);
    lr.add_mit_ebene(&alle[1], 2);
    lr.add_mit_ebene(&alle[2], 1);
    let weg = lr.remove_sprites_of_layer(1);
    assert_eq!(weg.len(), 2);
    assert!(weg[0].ist(&alle[0]) && weg[1].ist(&alle[2]));
    assert_eq!(wer(&lr, &alle), vec![1]);
    assert!(!alle[0].alive());
}

/// **`LayeredUpdates.copy()` verliert die Ebenen.**
///
/// `copy()` ruft `self.__class__(self.sprites())` - ohne `layer`. Ein
/// einfaches Sprite, das mit `layer=` eingehaengt wurde, hat kein
/// `_layer`, landet also auf der Vorgabeebene. Gemessen an zwei Sprites
/// auf den Ebenen 1 und 2: die Kopie meldet `layers() == [0]`.
#[test]
fn layered_copy_verliert_die_ebenen() {
    let alle: Vec<Sprite> = (0..2).map(|i| spr(i, 0, 4, 4)).collect();
    let lu = LayeredUpdates::new();
    lu.add_mit_ebene(&alle[0], 2);
    lu.add_mit_ebene(&alle[1], 1);
    assert_eq!(lu.layers(), vec![1, 2]);

    let k = lu.copy();
    assert_eq!(k.layers(), vec![0], "pygame verliert sie genauso");
    assert_eq!(wer(&k, &alle), vec![1, 0], "Reihenfolge bleibt");
    // Die Sprites stecken jetzt in beiden Gruppen.
    assert_eq!(alle[0].anzahl_gruppen(), 2);
}

/// Nach `remove` traegt `lostsprites` das **lebende** Rechteck des
/// Sprites - bewegt es sich, wandert der Eintrag mit.
///
/// Gemessen an `(34,24,8,5)` und einer Bewegung um `(-3, +2)`:
/// pygame meldet `(31, 26, 8, 5)`.
#[test]
fn lostsprites_zeigt_auf_das_lebende_rechteck() {
    let s = spr(34, 24, 8, 5);
    let lu = LayeredUpdates::mit(&s);
    lu.remove(&s);
    assert_eq!(lu.lostsprites(), vec![Rect::new(34, 24, 8, 5)]);
    s.bewege(-3, 2);
    assert_eq!(
        lu.lostsprites(),
        vec![Rect::new(31, 26, 8, 5)],
        "der Eintrag wandert mit"
    );
}

// ---------------------------------------------------------------------------
//  LayeredDirty
// ---------------------------------------------------------------------------

/// Grundwerte - `_use_update = False`, `_time_threshold = 12.5`,
/// `_clip = None`.
#[test]
fn layereddirty_grundwerte() {
    let ld = LayeredDirty::new();
    assert!(!ld.use_update());
    assert_eq!(ld.time_threshold(), 12.5);
    assert_eq!(ld.get_clip(), None);
    assert!(ld.hintergrund().is_none());
}

/// Ein Sprite mit `dirty == 0` wird beim Einhaengen schmutzig gemacht.
/// Ein Sprite, das gar kein `dirty` hat, kommt nicht hinein.
#[test]
fn layereddirty_nimmt_nur_dirtysprites() {
    let d = dspr(0, 0, 4, 4);
    d.set_dirty(0);
    let ld = LayeredDirty::new();
    ld.add(&d);
    assert_eq!(d.dirty(), 1, "dirty = 0 wird zu 1");
    assert_eq!(ld.len(), 1);

    let einfach = spr(0, 0, 4, 4);
    ld.add(&einfach);
    assert_eq!(ld.len(), 1, "einfaches Sprite bleibt draussen");
    assert!(!einfach.alive());
}

/// **Der Vollbildmodus raeumt die `dirty`-Flaggen nicht ab.**
///
/// Gemessen an drei Sprites:
/// * 1. `draw` (Vollbild): Rueckgabe `[Rect(0,0,100,100)]`, alle drei
///      stehen danach weiter auf `dirty == 1`, `_use_update` ist `True`;
/// * 2. `draw` (Teilbild): Rueckgabe
///      `[Rect(40,40,10,10), Rect(0,0,15,15)]`, danach `dirty == 0`.
#[test]
fn layereddirty_zwei_betriebsarten() {
    let mut flaeche = Surface::new((100, 100));
    let d1 = dspr(5, 5, 10, 10);
    let d2 = dspr(40, 40, 10, 10);
    let d0 = dspr(0, 0, 10, 10);
    let ld = LayeredDirty::mit(vec![
        d1.as_sprite().clone(),
        d2.as_sprite().clone(),
        d0.as_sprite().clone(),
    ]);

    let r = ld.draw(&mut flaeche);
    assert_eq!(r, vec![Rect::new(0, 0, 100, 100)], "Vollbild");
    assert_eq!(
        (d1.dirty(), d2.dirty(), d0.dirty()),
        (1, 1, 1),
        "Vollbild raeumt nicht auf"
    );
    assert!(ld.use_update(), "schaltet danach auf Teilbild");

    let r = ld.draw(&mut flaeche);
    assert_eq!(
        r,
        vec![Rect::new(40, 40, 10, 10), Rect::new(0, 0, 15, 15)],
        "verschmolzene schmutzige Bereiche"
    );
    assert_eq!((d1.dirty(), d2.dirty(), d0.dirty()), (0, 0, 0));

    // Nichts mehr schmutzig -> nichts zu melden.
    assert_eq!(ld.draw(&mut flaeche), Vec::<Rect>::new());

    // `dirty = 2` bleibt stehen.
    d1.set_dirty(2);
    assert_eq!(ld.draw(&mut flaeche), vec![Rect::new(5, 5, 10, 10)]);
    assert_eq!(d1.dirty(), 2, "dirty = 2 ueberlebt den Durchgang");
}

/// Ein Sprung ohne Ueberlappung meldet beide Stellen - neue zuerst.
///
/// Gemessen: `a` springt von `(5,5)` nach `(60,5)` ->
/// `[Rect(60,5,10,10), Rect(5,5,10,10)]`.
#[test]
fn layereddirty_sprung_meldet_beide_stellen() {
    let mut flaeche = Surface::new((100, 100));
    let a = dspr(5, 5, 10, 10);
    let b = dspr(40, 40, 10, 10);
    let ld = LayeredDirty::mit_einstellungen(0, true, 12.5);
    ld.add(vec![a.as_sprite().clone(), b.as_sprite().clone()]);

    assert_eq!(
        ld.draw(&mut flaeche),
        vec![Rect::new(5, 5, 10, 10), Rect::new(40, 40, 10, 10)]
    );
    assert_eq!((a.dirty(), b.dirty()), (0, 0));
    assert_eq!(ld.draw(&mut flaeche), Vec::<Rect>::new());

    a.set_topleft((60, 5));
    a.set_dirty(1);
    assert_eq!(
        ld.draw(&mut flaeche),
        vec![Rect::new(60, 5, 10, 10), Rect::new(5, 5, 10, 10)]
    );
}

/// `visible = 0` macht schmutzig, verhindert aber das Zeichnen - und
/// `dirty` faellt trotzdem auf 0.
///
/// Gemessen: Rueckgabe `[Rect(5,5,8,8)]`, `dirty == 0`, `spritedict`
/// behaelt das alte Rechteck.
#[test]
fn layereddirty_unsichtbar() {
    let mut flaeche = Surface::new((50, 50));
    flaeche.fill((0u8, 0u8, 0u8, 255u8));
    let d = dspr(5, 5, 8, 8);
    let ld = LayeredDirty::mit_einstellungen(0, true, 12.5);
    ld.add(&d);
    ld.draw(&mut flaeche);
    assert_eq!(farbe(&flaeche, (6, 6)), (0, 255, 0));

    d.set_visible(0);
    assert_eq!(d.dirty(), 1, "visible setzen macht schmutzig");
    let r = ld.draw(&mut flaeche);
    assert_eq!(r, vec![Rect::new(5, 5, 8, 8)]);
    assert_eq!(d.dirty(), 0);
    assert_eq!(ld.spritedict()[0].1, Some(Rect::new(5, 5, 8, 8)));

    // Und `dirty = 2` bleibt bei `visible` unberuehrt.
    d.set_dirty(2);
    d.set_visible(1);
    assert_eq!(d.dirty(), 2);
}

/// Der Hintergrund wird an den schmutzigen Stellen aufgetragen.
///
/// Gemessen: nach der Bewegung steht an der alten Stelle die
/// Hintergrundfarbe, an der unberuehrten weiterhin die Flaechenfarbe.
#[test]
fn layereddirty_hintergrund() {
    let mut flaeche = Surface::new((50, 50));
    flaeche.fill((1u8, 2u8, 3u8, 255u8));
    let mut bgd = Surface::new((50, 50));
    bgd.fill((9u8, 9u8, 9u8, 255u8));

    let c = dspr(5, 5, 8, 8);
    let ld = LayeredDirty::mit_einstellungen(0, true, 12.5);
    ld.add(&c);
    ld.clear(&mut flaeche, &bgd);
    assert!(ld.hintergrund().is_some(), "clear legt nur ab");
    assert_eq!(farbe(&flaeche, (6, 6)), (1, 2, 3), "clear malt nicht");

    ld.draw(&mut flaeche);
    assert_eq!(farbe(&flaeche, (6, 6)), (0, 255, 0));
    assert_eq!(farbe(&flaeche, (30, 30)), (1, 2, 3));

    c.set_topleft((30, 5));
    c.set_dirty(1);
    ld.draw(&mut flaeche);
    assert_eq!(farbe(&flaeche, (6, 6)), (9, 9, 9), "alte Stelle geraeumt");
    assert_eq!(farbe(&flaeche, (31, 6)), (0, 255, 0));
}

/// `source_rect` schneidet das Bild zu - die Rueckgabe traegt seine
/// Groesse.
///
/// Gemessen mit `source_rect = (2,3,6,7)` an einem 20x20-Sprite bei
/// `(5,5)`: `[Rect(5, 5, 6, 7)]`.
#[test]
fn layereddirty_source_rect() {
    let mut flaeche = Surface::new((60, 60));
    let e = dspr(5, 5, 20, 20);
    e.set_source_rect(Some(Rect::new(2, 3, 6, 7)));
    let ld = LayeredDirty::mit_einstellungen(0, true, 12.5);
    ld.add(&e);
    assert_eq!(ld.draw(&mut flaeche), vec![Rect::new(5, 5, 6, 7)]);
    assert_eq!(ld.spritedict()[0].1, Some(Rect::new(5, 5, 6, 7)));
}

/// `set_clip` beschraenkt und faellt auf Vollbild zurueck;
/// `repaint_rect` beschneidet.
///
/// Gemessen: nach `set_clip((10,10,30,30))` ist `_use_update` `False`,
/// der naechste `draw` liefert `[Rect(10,10,30,30)]`, und
/// `repaint_rect((0,0,100,100))` landet als `(10,10,30,30)` in
/// `lostsprites`.
#[test]
fn layereddirty_clip() {
    let mut flaeche = Surface::new((100, 100));
    let d = dspr(5, 5, 10, 10);
    let ld = LayeredDirty::mit(&d);
    ld.draw(&mut flaeche);
    assert!(ld.use_update());

    ld.set_clip(Some(Rect::new(10, 10, 30, 30)));
    assert_eq!(ld.get_clip(), Some(Rect::new(10, 10, 30, 30)));
    assert!(!ld.use_update(), "set_clip faellt auf Vollbild zurueck");
    assert_eq!(ld.draw(&mut flaeche), vec![Rect::new(10, 10, 30, 30)]);

    ld.repaint_rect(Rect::new(0, 0, 100, 100));
    assert_eq!(ld.lostsprites(), vec![Rect::new(10, 10, 30, 30)]);
}

/// `LayeredDirty.change_layer` macht nebenbei schmutzig.
#[test]
fn layereddirty_change_layer_macht_schmutzig() {
    let d = dspr(0, 0, 4, 4);
    let ld = LayeredDirty::mit(&d);
    d.set_dirty(0);
    ld.change_layer(&d, 4).unwrap();
    assert_eq!(d.dirty(), 1);
    assert_eq!(ld.get_layer_of_sprite(&d), 4);
    assert_eq!(d.layer(), Some(4));
}

// ---------------------------------------------------------------------------
//  Kollisionen
// ---------------------------------------------------------------------------

/// **`collide_circle` schreibt den Radius an das Sprite.**
///
/// Gemessen: ein 10x10-Sprite bekommt `radius == 7.0710678118654755`.
/// Waechst sein Rechteck danach auf 100x100, bleibt der alte Radius -
/// und die Pruefung liefert `False`, wo man `True` erwartet.
#[test]
fn collide_circle_merkt_sich_den_radius() {
    let a = spr(0, 0, 10, 10);
    let b = spr(12, 0, 10, 10);
    assert!(collide_circle(&a, &b));
    let r = a.radius().unwrap();
    assert!((r - 7.071_067_811_865_475_5).abs() < 1e-12, "radius = {r}");
    assert_eq!(b.radius(), Some(r));

    a.set_rect(Some(Rect::new(0, 0, 100, 100)));
    assert_eq!(a.radius(), Some(r), "der alte Radius bleibt stehen");
    assert!(
        !collide_circle(&a, &b),
        "und liefert darum das falsche Ergebnis - genau wie pygame"
    );
}

/// `collide_circle_ratio` legt den **ungeskalierten** Radius ab.
#[test]
fn collide_circle_ratio_legt_ungeskaliert_ab() {
    let a = spr(0, 0, 10, 10);
    let b = spr(30, 0, 10, 10);
    assert!(!collide_circle_ratio(1.0)(&a, &b));
    let r = a.radius().unwrap();
    assert!((r - 7.071_067_811_865_475_5).abs() < 1e-12);

    let c = spr(0, 0, 10, 10);
    let d = spr(30, 0, 10, 10);
    assert!(collide_circle_ratio(3.0)(&c, &d));
    assert!((c.radius().unwrap() - r).abs() < 1e-12, "nicht mit 3 gestreckt");
}

/// `collide_mask` legt **kein** `mask` am Sprite ab.
#[test]
fn collide_mask_legt_nichts_ab() {
    let mut bild = Surface::with_flags((10, 10), rustgame_core::SRCALPHA);
    bild.fill((255u8, 0u8, 0u8, 255u8));
    let a = Sprite::new();
    a.set_image(Some(bild.clone()));
    a.set_rect(Some(Rect::new(0, 0, 10, 10)));
    let b = Sprite::new();
    b.set_image(Some(bild));
    b.set_rect(Some(Rect::new(5, 0, 10, 10)));

    assert_eq!(collide_mask(&a, &b), Some((5, 0)));
    assert!(a.mask().is_none(), "pygame legt hier nichts ab");
    assert!(collide_mask_trifft(&a, &b));
}

/// `spritecollide(dokill=true)` toetet **nach** dem Sammeln.
///
/// Gemessen: das Ergebnis enthaelt `[k1, k2]`, beide mit
/// `alive() == False`, und die Gruppe behaelt nur `k3`.
#[test]
fn spritecollide_toetet_nach_dem_sammeln() {
    let t = spr(0, 0, 10, 10);
    let k1 = spr(5, 5, 10, 10);
    let k2 = spr(8, 8, 10, 10);
    let k3 = spr(90, 90, 10, 10);
    let g = Group::mit(vec![k1.clone(), k2.clone(), k3.clone()]);

    let ohne = spritecollide(&t, &g, false, None);
    assert_eq!(ohne.len(), 2);
    assert_eq!(g.len(), 3);

    let mit = spritecollide(&t, &g, true, None);
    assert_eq!(mit.len(), 2);
    assert!(mit[0].ist(&k1) && mit[1].ist(&k2));
    assert!(!k1.alive() && !k2.alive(), "getoetet");
    assert_eq!(g.len(), 1);
    assert!(g.sprites()[0].ist(&k3));
}

/// `spritecollideany` liefert das **erste in Gruppenreihenfolge**.
#[test]
fn spritecollideany_nimmt_das_erste() {
    let t = spr(0, 0, 10, 10);
    let n1 = spr(90, 90, 10, 10);
    let n2 = spr(5, 5, 10, 10);
    let n3 = spr(6, 6, 10, 10);
    let g = Group::mit(vec![n1.clone(), n2.clone(), n3.clone()]);
    assert!(spritecollideany(&t, &g, None).unwrap().ist(&n2));
    let weit = spr(500, 500, 10, 10);
    assert!(spritecollideany(&weit, &g, None).is_none());
}

/// **`groupcollide` mit `dokillb` wirkt schon waehrend des Durchlaufs.**
///
/// Gemessen an zwei mal zwei ueberlappenden Sprites:
/// `groupcollide(A, B, True, True)` liefert `{A1: [B1, B2]}` -
/// **nicht** auch einen Eintrag fuer `A2`, denn `B` ist da schon leer.
/// Danach: `A == [A2]`, `B == []`.
#[test]
fn groupcollide_dokillb_wirkt_sofort() {
    let a1 = spr(0, 0, 10, 10);
    let a2 = spr(5, 5, 10, 10);
    let b1 = spr(2, 2, 10, 10);
    let b2 = spr(8, 8, 10, 10);
    let ga = Group::mit(vec![a1.clone(), a2.clone()]);
    let gb = Group::mit(vec![b1.clone(), b2.clone()]);

    let erg = groupcollide(&ga, &gb, true, true, None);
    assert_eq!(erg.len(), 1, "nur A1 kommt zum Zug");
    assert!(erg[0].0.ist(&a1));
    assert_eq!(erg[0].1.len(), 2);
    assert_eq!(ga.len(), 1);
    assert!(ga.sprites()[0].ist(&a2));
    assert_eq!(gb.len(), 0);
}

/// Ohne `dokill` sieht jedes A-Sprite alle Treffer.
#[test]
fn groupcollide_ohne_toeten() {
    let ga = Group::mit(vec![spr(0, 0, 10, 10), spr(5, 5, 10, 10), spr(200, 200, 10, 10)]);
    let gb = Group::mit(vec![spr(2, 2, 10, 10), spr(8, 8, 10, 10), spr(300, 300, 10, 10)]);
    let erg = groupcollide(&ga, &gb, false, false, None);
    assert_eq!(erg.len(), 2);
    assert_eq!(erg[0].1.len(), 2);
    assert_eq!(erg[1].1.len(), 2);
    assert_eq!((ga.len(), gb.len()), (3, 3));
}

/// Ein eigener Rueckruf kommt durch - hier `collide_circle` bei
/// Abstand 30, das nichts findet.
#[test]
fn groupcollide_mit_rueckruf() {
    let ga = Group::mit(spr(0, 0, 10, 10));
    let gb = Group::mit(spr(30, 0, 10, 10));
    assert!(groupcollide(&ga, &gb, false, false, Some(&collide_circle)).is_empty());
    let weit = collide_rect_ratio(4.0);
    assert_eq!(
        groupcollide(&ga, &gb, false, false, Some(&weit)).len(),
        1,
        "mit Faktor 4 treffen sie sich"
    );
}

// ---------------------------------------------------------------------------
//  Die Namensgleichungen
// ---------------------------------------------------------------------------

/// `RenderPlain`, `RenderClear` und `OrderedUpdates` sind in
/// pygame-ce 2.5.7 **keine** Namensgleichungen, sondern veraltete
/// Unterklassen, die nur warnen.
///
/// Nachgemessen:
/// * `RenderPlain.draw is AbstractGroup.draw` -> `True`
/// * `OrderedUpdates.draw is RenderUpdates.draw` -> `True`
/// * beim Bauen kommt je eine `DeprecationWarning`
///
/// Verhaltensgleich sind sie damit - und hier echte Namensgleichungen.
#[test]
fn veraltete_namen_sind_gleichungen() {
    let mut flaeche = Surface::new((60, 60));
    let a = spr(5, 5, 10, 10);
    let rp: RenderPlain = RenderPlain::mit(&a);
    assert_eq!(rp.draw(&mut flaeche), Vec::<Rect>::new(), "wie Group");

    let b = spr(5, 5, 10, 10);
    let ou: OrderedUpdates = OrderedUpdates::mit(&b);
    assert_eq!(
        ou.draw(&mut flaeche),
        vec![Rect::new(5, 5, 10, 10)],
        "wie RenderUpdates"
    );

    let c = spr(0, 0, 4, 4);
    let rc: RenderClear = RenderClear::mit(&c);
    assert_eq!(rc.len(), 1);
}

/// `Sprite::groups` liefert Gruppen, mit denen sich weiterarbeiten
/// laesst - auch wenn ihr urspruenglicher Typ nicht mehr zu sehen ist.
#[test]
fn groups_liefert_brauchbare_gruppen() {
    let s = spr(1, 1, 4, 4);
    let andere = spr(2, 2, 4, 4);
    let lu = LayeredUpdates::mit(&s);
    let g = Group::mit(&s);
    assert_eq!(s.groups().len(), 2);

    for gr in s.groups() {
        gr.add(&andere);
    }
    assert_eq!(lu.len(), 2);
    assert_eq!(g.len(), 2);
    assert_eq!(andere.anzahl_gruppen(), 2);
}
