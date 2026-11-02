// faelle.rs
// =========
// **Erzeugte Datei - nicht von Hand aendern.**
//
// Quelle: `rustgame/tools/gen_golden.py` uebersetzt `tools/cases.py`.
// Dieselbe Falltabelle erzeugt die Referenzbilder mit echtem pygame
// und wird hier nachgespielt - so koennen beide Seiten gar nicht
// auseinanderlaufen.

/// Alle Faelle: `(Name, Toleranz, Bauanweisung)`.
#[allow(clippy::type_complexity)]
#[allow(clippy::approx_constant)]
fn faelle() -> Vec<(&'static str, u8, Box<dyn Fn() -> Ergebnis>)> {
    vec![
        ("fill_einfach", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 12), 0);
            s.fill((220u8, 60u8, 60u8, 255u8));
            Ergebnis::Fertig(s)
        })),
        ("fill_rect", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 12), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s.fill_rect((60u8, 200u8, 120u8, 255u8), Rect::new(2, 3, 8, 5));
            Ergebnis::Fertig(s)
        })),
        ("fill_rect_ragt_raus", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 12), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s.fill_rect((70u8, 110u8, 240u8, 255u8), Rect::new(-4, -4, 10, 10));
            Ergebnis::Fertig(s)
        })),
        ("blit_add", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((8, 8), SRCALPHA);
            s.fill((100u8, 150u8, 200u8, 255u8));
            let q = gefuellt(s.get_size(), (50u8, 60u8, 70u8, 128u8));
            s.blit_ex(&q, (0, 0), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            Ergebnis::Fertig(s)
        })),
        ("blit_sub", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((8, 8), SRCALPHA);
            s.fill((100u8, 150u8, 200u8, 255u8));
            let q = gefuellt(s.get_size(), (50u8, 60u8, 70u8, 128u8));
            s.blit_ex(&q, (0, 0), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            Ergebnis::Fertig(s)
        })),
        ("blit_mult", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((8, 8), SRCALPHA);
            s.fill((100u8, 150u8, 200u8, 255u8));
            let q = gefuellt(s.get_size(), (50u8, 60u8, 70u8, 128u8));
            s.blit_ex(&q, (0, 0), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            Ergebnis::Fertig(s)
        })),
        ("blit_min", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((8, 8), SRCALPHA);
            s.fill((100u8, 150u8, 200u8, 255u8));
            let q = gefuellt(s.get_size(), (50u8, 60u8, 70u8, 128u8));
            s.blit_ex(&q, (0, 0), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            Ergebnis::Fertig(s)
        })),
        ("blit_max", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((8, 8), SRCALPHA);
            s.fill((100u8, 150u8, 200u8, 255u8));
            let q = gefuellt(s.get_size(), (50u8, 60u8, 70u8, 128u8));
            s.blit_ex(&q, (0, 0), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            Ergebnis::Fertig(s)
        })),
        ("blit_rgba_add", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((8, 8), SRCALPHA);
            s.fill((100u8, 150u8, 200u8, 255u8));
            let q = gefuellt(s.get_size(), (50u8, 60u8, 70u8, 128u8));
            s.blit_ex(&q, (0, 0), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            Ergebnis::Fertig(s)
        })),
        ("blit_rgba_sub", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((8, 8), SRCALPHA);
            s.fill((100u8, 150u8, 200u8, 255u8));
            let q = gefuellt(s.get_size(), (50u8, 60u8, 70u8, 128u8));
            s.blit_ex(&q, (0, 0), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
            Ergebnis::Fertig(s)
        })),
        ("blit_rgba_mult", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((8, 8), SRCALPHA);
            s.fill((100u8, 150u8, 200u8, 255u8));
            let q = gefuellt(s.get_size(), (50u8, 60u8, 70u8, 128u8));
            s.blit_ex(&q, (0, 0), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            Ergebnis::Fertig(s)
        })),
        ("blit_rgba_min", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((8, 8), SRCALPHA);
            s.fill((100u8, 150u8, 200u8, 255u8));
            let q = gefuellt(s.get_size(), (50u8, 60u8, 70u8, 128u8));
            s.blit_ex(&q, (0, 0), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            Ergebnis::Fertig(s)
        })),
        ("blit_rgba_max", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((8, 8), SRCALPHA);
            s.fill((100u8, 150u8, 200u8, 255u8));
            let q = gefuellt(s.get_size(), (50u8, 60u8, 70u8, 128u8));
            s.blit_ex(&q, (0, 0), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            Ergebnis::Fertig(s)
        })),
        ("blit_alpha_standard", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((8, 8), 0);
            s.fill((100u8, 150u8, 200u8, 255u8));
            let q = gefuellt(s.get_size(), (50u8, 60u8, 70u8, 128u8));
            s.blit_ex(&q, (0, 0), None, 0);
            Ergebnis::Fertig(s)
        })),
        ("blit_flaechen_alpha", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((8, 8), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            let mut q = gefuellt(s.get_size(), (200u8, 100u8, 50u8, 128u8));
            q.set_alpha(Some(128));
            s.blit(&q, (0, 0));
            Ergebnis::Fertig(s)
        })),
        ("blit_versetzt", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 16), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            let q = gefuellt(s.get_size(), (255u8, 0u8, 0u8, 255u8));
            s.blit_ex(&q, (5, 4), None, 0);
            Ergebnis::Fertig(s)
        })),
        ("blit_teilweise_draussen", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 16), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            let q = gefuellt(s.get_size(), (255u8, 0u8, 0u8, 255u8));
            s.blit_ex(&q, (12, 12), None, 0);
            Ergebnis::Fertig(s)
        })),
        ("blit_negativ_versetzt", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 16), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            let q = gefuellt(s.get_size(), (255u8, 0u8, 0u8, 255u8));
            s.blit_ex(&q, (-3, -3), None, 0);
            Ergebnis::Fertig(s)
        })),
        ("blit_mit_clip", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 16), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s.set_clip(Some(Rect::new(4, 4, 6, 6)));
            let q = gefuellt(s.get_size(), (255u8, 0u8, 0u8, 255u8));
            s.blit_ex(&q, (0, 0), None, 0);
            Ergebnis::Fertig(s)
        })),
        ("rect_gefuellt", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 12), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::rect(&mut s, (255u8, 255u8, 255u8, 255u8), Rect::new(2, 2, 10, 7), 0);
            Ergebnis::Fertig(s)
        })),
        ("rect_umriss_1", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 12), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::rect(&mut s, (255u8, 255u8, 255u8, 255u8), Rect::new(2, 2, 10, 7), 1);
            Ergebnis::Fertig(s)
        })),
        ("rect_umriss_3", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((20, 16), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::rect(&mut s, (255u8, 255u8, 255u8, 255u8), Rect::new(3, 3, 13, 9), 3);
            Ergebnis::Fertig(s)
        })),
        ("rect_radius_gefuellt", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((11, 11), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (255u8, 255u8, 255u8, 255u8), Rect::new(0, 0, 11, 11), 0, 3);
            Ergebnis::Fertig(s)
        })),
        ("rect_radius_umriss", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((24, 16), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (255u8, 255u8, 255u8, 255u8), Rect::new(2, 2, 20, 12), 2, 4);
            Ergebnis::Fertig(s)
        })),
        ("rect_radius_gross", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((20, 20), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (255u8, 255u8, 255u8, 255u8), Rect::new(0, 0, 20, 20), 0, 10);
            Ergebnis::Fertig(s)
        })),
        ("rect_radius_zu_gross", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((20, 20), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (255u8, 255u8, 255u8, 255u8), Rect::new(0, 0, 20, 20), 0, 50);
            Ergebnis::Fertig(s)
        })),
        ("rect_breite_groesser_als_rect", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 16), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::rect(&mut s, (255u8, 255u8, 255u8, 255u8), Rect::new(2, 2, 8, 8), 20);
            Ergebnis::Fertig(s)
        })),
        ("rect_leer", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 16), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::rect(&mut s, (255u8, 255u8, 255u8, 255u8), Rect::new(4, 4, 0, 5), 0);
            Ergebnis::Fertig(s)
        })),
        ("rect_alpha", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 16), SRCALPHA);
            s.fill((50u8, 50u8, 50u8, 255u8));
            rustgame_core::draw::rect(&mut s, (255u8, 0u8, 0u8, 128u8), Rect::new(2, 2, 10, 10), 0);
            Ergebnis::Fertig(s)
        })),
        ("circle_gefuellt", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((13, 13), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::circle(&mut s, (255u8, 255u8, 255u8, 255u8), (6, 6), 5, 0);
            Ergebnis::Fertig(s)
        })),
        ("circle_umriss_1", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((13, 13), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::circle(&mut s, (255u8, 255u8, 255u8, 255u8), (6, 6), 5, 1);
            Ergebnis::Fertig(s)
        })),
        ("circle_umriss_2", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((21, 21), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::circle(&mut s, (255u8, 255u8, 255u8, 255u8), (10, 10), 9, 2);
            Ergebnis::Fertig(s)
        })),
        ("circle_radius_0", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((9, 9), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::circle(&mut s, (255u8, 255u8, 255u8, 255u8), (4, 4), 0, 0);
            Ergebnis::Fertig(s)
        })),
        ("circle_radius_1", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((9, 9), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::circle(&mut s, (255u8, 255u8, 255u8, 255u8), (4, 4), 1, 0);
            Ergebnis::Fertig(s)
        })),
        ("circle_am_rand", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 16), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::circle(&mut s, (255u8, 255u8, 255u8, 255u8), (0, 0), 6, 0);
            Ergebnis::Fertig(s)
        })),
        ("ellipse_gefuellt", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((20, 14), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (255u8, 255u8, 255u8, 255u8), Rect::new(2, 2, 16, 10), 0);
            Ergebnis::Fertig(s)
        })),
        ("ellipse_umriss", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((20, 14), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (255u8, 255u8, 255u8, 255u8), Rect::new(2, 2, 16, 10), 1);
            Ergebnis::Fertig(s)
        })),
        ("ellipse_flach", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((20, 6), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (255u8, 255u8, 255u8, 255u8), Rect::new(1, 1, 18, 4), 0);
            Ergebnis::Fertig(s)
        })),
        ("line_waagerecht", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 8), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::line(&mut s, (255u8, 255u8, 255u8, 255u8), (2, 4), (13, 4), 1);
            Ergebnis::Fertig(s)
        })),
        ("line_senkrecht", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((8, 16), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::line(&mut s, (255u8, 255u8, 255u8, 255u8), (4, 2), (4, 13), 1);
            Ergebnis::Fertig(s)
        })),
        ("line_schraeg", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 16), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::line(&mut s, (255u8, 255u8, 255u8, 255u8), (1, 1), (14, 12), 1);
            Ergebnis::Fertig(s)
        })),
        ("line_dick", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 12), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::line(&mut s, (255u8, 255u8, 255u8, 255u8), (1, 5), (14, 5), 3);
            Ergebnis::Fertig(s)
        })),
        ("line_dick_schraeg", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((20, 20), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::line(&mut s, (255u8, 255u8, 255u8, 255u8), (2, 2), (17, 15), 4);
            Ergebnis::Fertig(s)
        })),
        ("line_punkt", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((8, 8), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::line(&mut s, (255u8, 255u8, 255u8, 255u8), (4, 4), (4, 4), 1);
            Ergebnis::Fertig(s)
        })),
        ("polygon_dreieck", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((20, 20), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (255u8, 255u8, 255u8, 255u8), &[(2, 18), (10, 2), (18, 18)], 0);
            Ergebnis::Fertig(s)
        })),
        ("polygon_umriss", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((20, 20), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (255u8, 255u8, 255u8, 255u8), &[(2, 18), (10, 2), (18, 18)], 1);
            Ergebnis::Fertig(s)
        })),
        ("polygon_viereck", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((20, 20), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (255u8, 255u8, 255u8, 255u8), &[(3, 3), (16, 5), (14, 17), (5, 15)], 0);
            Ergebnis::Fertig(s)
        })),
        ("polygon_konkav", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((24, 24), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (255u8, 255u8, 255u8, 255u8), &[(2, 2), (21, 2), (21, 21), (12, 10), (2, 21)], 0);
            Ergebnis::Fertig(s)
        })),
        ("lines_offen", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((20, 20), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::lines(&mut s, (255u8, 255u8, 255u8, 255u8), false, &[(2, 2), (10, 15), (17, 4)], 1);
            Ergebnis::Fertig(s)
        })),
        ("lines_geschlossen", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((20, 20), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::lines(&mut s, (255u8, 255u8, 255u8, 255u8), true, &[(2, 2), (10, 15), (17, 4)], 1);
            Ergebnis::Fertig(s)
        })),
        ("arc_halb", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((20, 20), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::arc(&mut s, (255u8, 255u8, 255u8, 255u8), Rect::new(2, 2, 16, 16), 0.0, 3.14159, 1);
            Ergebnis::Fertig(s)
        })),
        ("arc_dick", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((24, 24), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::arc(&mut s, (255u8, 255u8, 255u8, 255u8), Rect::new(2, 2, 20, 20), 0.5, 2.5, 3);
            Ergebnis::Fertig(s)
        })),
        ("aaline_schraeg", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 8), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::aaline(&mut s, (255u8, 255u8, 255u8, 255u8), (1.0, 1.0), (10.0, 6.0), 1);
            Ergebnis::Fertig(s)
        })),
        ("aaline_halbe_pixel", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 8), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::aaline(&mut s, (255u8, 255u8, 255u8, 255u8), (1.5, 1.5), (10.5, 6.5), 1);
            Ergebnis::Fertig(s)
        })),
        ("aaline_steil", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((8, 16), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::aaline(&mut s, (255u8, 255u8, 255u8, 255u8), (1.0, 1.0), (6.0, 14.0), 1);
            Ergebnis::Fertig(s)
        })),
        ("aaline_dick_gerade", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((14, 10), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::aaline(&mut s, (255u8, 255u8, 255u8, 255u8), (1.0, 1.0), (12.0, 8.0), 4);
            Ergebnis::Fertig(s)
        })),
        ("aaline_dick_ungerade", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((14, 10), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::aaline(&mut s, (255u8, 255u8, 255u8, 255u8), (1.0, 1.0), (12.0, 8.0), 5);
            Ergebnis::Fertig(s)
        })),
        ("aaline_breite_null", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 8), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::aaline(&mut s, (255u8, 255u8, 255u8, 255u8), (1.0, 1.0), (10.0, 6.0), 0);
            Ergebnis::Fertig(s)
        })),
        ("aaline_ueber_den_rand", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 8), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::aaline(&mut s, (255u8, 255u8, 255u8, 255u8), (-8.0, -4.0), (20.0, 12.0), 1);
            Ergebnis::Fertig(s)
        })),
        ("aaline_clip", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 12), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s.set_clip(Some(Rect::new(3, 2, 7, 6)));
            rustgame_core::draw::aaline(&mut s, (255u8, 255u8, 255u8, 255u8), (0.0, 0.0), (15.0, 11.0), 1);
            Ergebnis::Fertig(s)
        })),
        ("aaline_clip_dick", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 12), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s.set_clip(Some(Rect::new(3, 2, 7, 6)));
            rustgame_core::draw::aaline(&mut s, (255u8, 255u8, 255u8, 255u8), (0.0, 0.0), (15.0, 11.0), 4);
            Ergebnis::Fertig(s)
        })),
        ("aaline_auf_farbe", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 8), 0);
            s.fill((220u8, 60u8, 60u8, 255u8));
            rustgame_core::draw::aaline(&mut s, (70u8, 110u8, 240u8, 255u8), (1.0, 1.0), (10.0, 6.0), 1);
            Ergebnis::Fertig(s)
        })),
        ("aaline_alpha", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 8), SRCALPHA);
            rustgame_core::draw::aaline(&mut s, (255u8, 255u8, 255u8, 255u8), (1.0, 1.0), (10.0, 6.0), 1);
            Ergebnis::Fertig(s)
        })),
        ("aalines_offen", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 12), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::aalines(&mut s, (220u8, 60u8, 60u8, 255u8), false, &[(2.0, 2.0), (11.0, 4.0), (6.0, 10.0)]);
            Ergebnis::Fertig(s)
        })),
        ("aalines_geschlossen", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((14, 12), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::aalines(&mut s, (220u8, 60u8, 60u8, 255u8), true, &[(2.0, 2.0), (11.0, 4.0), (6.0, 10.0)]);
            Ergebnis::Fertig(s)
        })),
        ("aalines_zwei_punkte", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 8), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::aalines(&mut s, (220u8, 60u8, 60u8, 255u8), false, &[(1.0, 1.0), (10.0, 6.0)]);
            Ergebnis::Fertig(s)
        })),
        ("aalines_knick", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((20, 16), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::aalines(&mut s, (220u8, 60u8, 60u8, 255u8), false, &[(1.0, 1.0), (4.0, 14.0), (18.0, 15.0)]);
            Ergebnis::Fertig(s)
        })),
        ("aacircle_duenn", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 12), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::aacircle(&mut s, (220u8, 60u8, 60u8, 255u8), (6, 6), 4, 1);
            Ergebnis::Fertig(s)
        })),
        ("aacircle_gefuellt", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 12), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::aacircle(&mut s, (220u8, 60u8, 60u8, 255u8), (6, 6), 4, 0);
            Ergebnis::Fertig(s)
        })),
        ("aacircle_dick", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 16), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::aacircle(&mut s, (220u8, 60u8, 60u8, 255u8), (8, 8), 6, 3);
            Ergebnis::Fertig(s)
        })),
        ("aacircle_r1", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((8, 8), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::aacircle(&mut s, (220u8, 60u8, 60u8, 255u8), (4, 4), 1, 1);
            Ergebnis::Fertig(s)
        })),
        ("aacircle_gross", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((32, 32), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::aacircle(&mut s, (60u8, 200u8, 120u8, 255u8), (16, 16), 14, 2);
            Ergebnis::Fertig(s)
        })),
        ("aacircle_am_rand", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 12), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::aacircle(&mut s, (220u8, 60u8, 60u8, 255u8), (1, 1), 5, 1);
            Ergebnis::Fertig(s)
        })),
        ("aacircle_auf_farbe", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 12), 0);
            s.fill((70u8, 110u8, 240u8, 255u8));
            rustgame_core::draw::aacircle(&mut s, (220u8, 60u8, 60u8, 255u8), (6, 6), 4, 1);
            Ergebnis::Fertig(s)
        })),
        ("aacircle_alpha", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 12), SRCALPHA);
            rustgame_core::draw::aacircle(&mut s, (220u8, 60u8, 60u8, 255u8), (6, 6), 4, 1);
            Ergebnis::Fertig(s)
        })),
        ("flood_fill_kreis", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 12), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::circle(&mut s, (255u8, 255u8, 255u8, 255u8), (8, 6), 5, 1);
            rustgame_core::draw::flood_fill(&mut s, (220u8, 60u8, 60u8, 255u8), (8, 6));
            Ergebnis::Fertig(s)
        })),
        ("flood_fill_wand", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 12), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::line(&mut s, (255u8, 255u8, 255u8, 255u8), (8, 0), (8, 11), 1);
            rustgame_core::draw::flood_fill(&mut s, (220u8, 60u8, 60u8, 255u8), (2, 2));
            Ergebnis::Fertig(s)
        })),
        ("flood_fill_diagonal", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((8, 8), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::line(&mut s, (255u8, 255u8, 255u8, 255u8), (0, 0), (7, 7), 1);
            rustgame_core::draw::flood_fill(&mut s, (220u8, 60u8, 60u8, 255u8), (7, 0));
            Ergebnis::Fertig(s)
        })),
        ("flood_fill_clip", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 12), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s.set_clip(Some(Rect::new(2, 1, 8, 6)));
            rustgame_core::draw::flood_fill(&mut s, (220u8, 60u8, 60u8, 255u8), (4, 3));
            Ergebnis::Fertig(s)
        })),
        ("flood_fill_alpha", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 8), SRCALPHA);
            rustgame_core::draw::circle(&mut s, (255u8, 255u8, 255u8, 255u8), (6, 4), 3, 1);
            rustgame_core::draw::flood_fill(&mut s, (0u8, 255u8, 0u8, 128u8), (0, 0));
            Ergebnis::Fertig(s)
        })),
        ("scale_hoch", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((4, 4), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 4, 4);
            s = rustgame_core::transform::scale(&s, (16, 16));
            Ergebnis::Fertig(s)
        })),
        ("scale_runter", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 16), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 16, 16);
            s = rustgame_core::transform::scale(&s, (4, 4));
            Ergebnis::Fertig(s)
        })),
        ("smoothscale_runter", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 16), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 16, 16);
            s = rustgame_core::transform::smoothscale(&s, (8, 8));
            Ergebnis::Fertig(s)
        })),
        ("smoothscale_hoch", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((8, 8), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 8, 8);
            s = rustgame_core::transform::smoothscale(&s, (16, 16));
            Ergebnis::Fertig(s)
        })),
        ("smoothscale_krumm", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((13, 7), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 13, 7);
            s = rustgame_core::transform::smoothscale(&s, (6, 11));
            Ergebnis::Fertig(s)
        })),
        ("flip_waagerecht", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 8), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 12, 8);
            s = rustgame_core::transform::flip(&s, true, false);
            Ergebnis::Fertig(s)
        })),
        ("flip_senkrecht", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 8), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 12, 8);
            s = rustgame_core::transform::flip(&s, false, true);
            Ergebnis::Fertig(s)
        })),
        ("rotate_90", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 8), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 12, 8);
            s = rustgame_core::transform::rotate(&s, 90.0);
            Ergebnis::Fertig(s)
        })),
        ("rotate_180", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 8), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 12, 8);
            s = rustgame_core::transform::rotate(&s, 180.0);
            Ergebnis::Fertig(s)
        })),
        ("rotate_45", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 8), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 12, 8);
            s = rustgame_core::transform::rotate(&s, 45.0);
            Ergebnis::Fertig(s)
        })),
        ("rotate_schraeg", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((14, 9), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 14, 9);
            s = rustgame_core::transform::rotate(&s, 12.5);
            Ergebnis::Fertig(s)
        })),
        ("rotate_rueckwaerts", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((11, 11), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 11, 11);
            s = rustgame_core::transform::rotate(&s, -135.0);
            Ergebnis::Fertig(s)
        })),
        ("rotate_ueber_voll", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((10, 6), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 10, 6);
            s = rustgame_core::transform::rotate(&s, 417.0);
            Ergebnis::Fertig(s)
        })),
        ("scale_by_halb", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 12), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 16, 12);
            s = rustgame_core::transform::scale_by(&s, 0.5, 0.5);
            Ergebnis::Fertig(s)
        })),
        ("scale_by_krumm", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((9, 7), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 9, 7);
            s = rustgame_core::transform::scale_by(&s, 1.75, 0.6);
            Ergebnis::Fertig(s)
        })),
        ("smoothscale_by_klein", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 12), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 12, 12);
            s = rustgame_core::transform::smoothscale_by(&s, 0.4, 1.3);
            Ergebnis::Fertig(s)
        })),
        ("scale2x_muster", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((7, 5), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 7, 5);
            s = rustgame_core::transform::scale2x(&s);
            Ergebnis::Fertig(s)
        })),
        ("scale2x_formen", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((10, 10), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (220u8, 60u8, 60u8, 255u8), Rect::new(1, 1, 5, 5), 0, 2);
            rustgame_core::draw::circle(&mut s, (60u8, 200u8, 120u8, 255u8), (7, 7), 3, 1);
            s = rustgame_core::transform::scale2x(&s);
            Ergebnis::Fertig(s)
        })),
        ("grayscale_muster", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 10), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 12, 10);
            s = rustgame_core::transform::grayscale(&s);
            Ergebnis::Fertig(s)
        })),
        ("invert_muster", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 8), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 12, 8);
            s = rustgame_core::transform::invert(&s);
            Ergebnis::Fertig(s)
        })),
        ("laplacian_kanten", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((13, 9), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::rect(&mut s, (255u8, 255u8, 255u8, 255u8), Rect::new(3, 2, 7, 5), 0);
            s = rustgame_core::transform::laplacian(&s);
            Ergebnis::Fertig(s)
        })),
        ("laplacian_rand", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((5, 5), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            rustgame_core::draw::rect(&mut s, (255u8, 200u8, 100u8, 255u8), Rect::new(0, 0, 1, 5), 0);
            rustgame_core::draw::rect(&mut s, (250u8, 210u8, 90u8, 255u8), Rect::new(4, 0, 1, 5), 0);
            s = rustgame_core::transform::laplacian(&s);
            Ergebnis::Fertig(s)
        })),
        ("box_blur_r2", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 12), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 16, 12);
            s = rustgame_core::transform::box_blur(&s, 2, true).unwrap();
            Ergebnis::Fertig(s)
        })),
        ("box_blur_ohne_rand", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 12), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 16, 12);
            s = rustgame_core::transform::box_blur(&s, 2, false).unwrap();
            Ergebnis::Fertig(s)
        })),
        ("gaussian_blur_r2", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 12), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 16, 12);
            s = rustgame_core::transform::gaussian_blur(&s, 2, true).unwrap();
            Ergebnis::Fertig(s)
        })),
        ("gaussian_blur_r0_ist_schwarz", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((8, 8), 0);
            s.fill((220u8, 60u8, 60u8, 255u8));
            s = rustgame_core::transform::gaussian_blur(&s, 0, true).unwrap();
            Ergebnis::Fertig(s)
        })),
        ("hsl_ton", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 8), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 12, 8);
            s = rustgame_core::transform::hsl(&s, 60.0, 0.0, 0.0).unwrap();
            Ergebnis::Fertig(s)
        })),
        ("hsl_hell_voll", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 8), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 12, 8);
            s = rustgame_core::transform::hsl(&s, 400.0, 0.17, 1.0).unwrap();
            Ergebnis::Fertig(s)
        })),
        ("hsl_saettigung_weg", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 8), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 12, 8);
            s = rustgame_core::transform::hsl(&s, 0.0, -1.0, 0.0).unwrap();
            Ergebnis::Fertig(s)
        })),
        ("chop_mitte", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 8), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 12, 8);
            s = rustgame_core::transform::chop(&s, Rect::new(4, 3, 3, 2));
            Ergebnis::Fertig(s)
        })),
        ("chop_negativ_waechst", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((8, 6), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 8, 6);
            s = rustgame_core::transform::chop(&s, Rect::new(1, 1, -2, -2));
            Ergebnis::Fertig(s)
        })),
        ("pixelate_3", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((15, 12), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 15, 12);
            s = rustgame_core::transform::pixelate(&s, 3).unwrap();
            Ergebnis::Fertig(s)
        })),
        ("pixelate_gross", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 9), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 12, 9);
            s = rustgame_core::transform::pixelate(&s, 7).unwrap();
            Ergebnis::Fertig(s)
        })),
        ("solid_overlay_ohne_alpha_ist_leer", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((10, 8), 0);
            s.fill((220u8, 60u8, 60u8, 255u8));
            s = rustgame_core::transform::solid_overlay(&s, (60u8, 200u8, 120u8, 255u8), false);
            Ergebnis::Fertig(s)
        })),
        ("solid_overlay_alpha", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((10, 8), SRCALPHA);
            s.fill((0u8, 0u8, 0u8, 0u8));
            rustgame_core::draw::circle(&mut s, (200u8, 100u8, 50u8, 200u8), (5, 4), 3, 0);
            s = rustgame_core::transform::solid_overlay(&s, (255u8, 9u8, 8u8, 77u8), false);
            Ergebnis::Fertig(s)
        })),
        ("solid_overlay_keep_alpha", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((10, 8), SRCALPHA);
            s.fill((0u8, 0u8, 0u8, 0u8));
            rustgame_core::draw::circle(&mut s, (200u8, 100u8, 50u8, 200u8), (5, 4), 3, 0);
            s = rustgame_core::transform::solid_overlay(&s, (255u8, 9u8, 8u8, 77u8), true);
            Ergebnis::Fertig(s)
        })),
        ("rotozoom_90", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 8), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 12, 8);
            s = rustgame_core::transform::rotozoom(&s, 90.0, 1.0);
            Ergebnis::Fertig(s)
        })),
        ("rotozoom_45", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((12, 8), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 12, 8);
            s = rustgame_core::transform::rotozoom(&s, 45.0, 1.0);
            Ergebnis::Fertig(s)
        })),
        ("rotozoom_schraeg_klein", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((14, 9), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 14, 9);
            s = rustgame_core::transform::rotozoom(&s, 12.5, 0.6);
            Ergebnis::Fertig(s)
        })),
        ("rotozoom_nur_zoom", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((9, 7), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 9, 7);
            s = rustgame_core::transform::rotozoom(&s, 0.0, 2.0);
            Ergebnis::Fertig(s)
        })),
        ("rotozoom_zoom_runter", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((16, 12), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 16, 12);
            s = rustgame_core::transform::rotozoom(&s, 0.0, 0.4);
            Ergebnis::Fertig(s)
        })),
        ("rotozoom_rueckwaerts", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((11, 11), 0);
            s.fill((0u8, 0u8, 0u8, 255u8));
            s = muster_auf(s, 11, 11);
            s = rustgame_core::transform::rotozoom(&s, -135.0, 1.3);
            Ergebnis::Fertig(s)
        })),
        ("grayscale_mit_alpha", 0, Box::new(|| {
            #[allow(unused_mut)]
            let mut s = Surface::with_flags((10, 8), SRCALPHA);
            s.fill((0u8, 0u8, 0u8, 0u8));
            s = muster_auf(s, 10, 8);
            let q = gefuellt(s.get_size(), (255u8, 0u8, 0u8, 120u8));
            s.blit_ex(&q, (2, 2), None, 0);
            s = rustgame_core::transform::grayscale(&s);
            Ergebnis::Fertig(s)
        })),
    ]
}
