// ERZEUGT von rustgame/tools/fuzz_ops.py - nicht von Hand aendern.
//
// Jeder Fall ist eine gewuerfelte Folge von Operationen, die **echtes
// pygame-ce** bereits ausgefuehrt hat; hinterlegt ist nur der FNV-1a-Hash
// des Ergebnisses. Weicht der Hash ab, stimmt irgendwo ein Pixel nicht -
// welcher, verraet:
//
//     .venv/Scripts/python.exe rustgame/tools/fuzz_ops.py --zeige <nummer>
//
// Neu wuerfeln (mehr Faelle, andere Folgen):
//     .venv/Scripts/python.exe rustgame/tools/fuzz_ops.py --anzahl 2000

#[allow(clippy::type_complexity)]
fn fuzz_faelle() -> Vec<(u64, bool, fn() -> FuzzErgebnis)> {
    // `bool` = bekannte, benannte Abweichung (siehe `bekannt_abweichend`
    // in fuzz_ops.py). Solche Faelle werden gezaehlt, nicht gewertet.
    let mut v: Vec<(u64, bool, fn() -> FuzzErgebnis)> = Vec::new();
    v.push((0, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((18, 30), 0);
            s.fill((20u8, 132u8, 248u8, 255u8));
            rustgame_core::draw::line(&mut s, (183u8, 111u8, 71u8, 244u8), (5, 4), (20, 2), 5);
            rustgame_core::draw::ellipse(&mut s, (158u8, 50u8, 37u8, 75u8), Rect::new(19, 28, 10, 30), 4);
            s.set_clip(Some(Rect::new(13, 10, 5, 7)));
            rustgame_core::draw::polygon(&mut s, (31u8, 7u8, 47u8, 133u8), &[(19, 21), (18, -4), (15, 27), (6, 11), (19, 16), (18, 0)], 1);
            rustgame_core::draw::line(&mut s, (72u8, 229u8, 46u8, 255u8), (-2, 16), (12, 27), 1);
            s = rustgame_core::transform::grayscale(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xB233CBA2AD0F27CC, flaeche: s }
    }));
    v.push((1, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((10, 24), 0);
            s.fill((32u8, 130u8, 60u8, 255u8));
            rustgame_core::draw::circle(&mut s, (107u8, 48u8, 249u8, 194u8), (-4, 20), 13, 4);
            rustgame_core::draw::circle(&mut s, (228u8, 136u8, 117u8, 255u8), (-1, 16), 0, 0);
            s.fill_rect((110u8, 216u8, 14u8, 195u8), Rect::new(11, 2, 12, 14));
            rustgame_core::draw::polygon(&mut s, (112u8, 235u8, 148u8, 255u8), &[(-4, 22), (13, 2), (1, 14), (-1, 17)], 3);
            let mut q = Surface::with_flags((12, 8), SRCALPHA);
            q = muster_auf(q, 12, 8);
            s.blit_ex(&q, (12, 21), None, 0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x6CD439C52DBC5092, flaeche: s }
    }));
    v.push((2, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 8), SRCALPHA);
            s.fill((128u8, 108u8, 18u8, 157u8));
            rustgame_core::draw::circle(&mut s, (190u8, 227u8, 137u8, 201u8), (10, -3), 0, 2);
            let mut q = Surface::with_flags((7, 8), SRCALPHA);
            q = muster_auf(q, 7, 8);
            q.set_colorkey(Some((90u8, 120u8, 118u8)));
            s.blit_ex(&q, (-4, 1), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            let mut q = Surface::with_flags((7, 10), 0);
            q = muster_auf(q, 7, 10);
            q.set_alpha(Some(228));
            s.blit_ex(&q, (7, 7), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            let mut q = Surface::with_flags((9, 4), 0);
            q = muster_auf(q, 9, 4);
            q.set_alpha(Some(236));
            s.blit_ex(&q, (-1, 11), Some(Rect::new(10, 3, 10, 7)), rustgame_core::blend::flags::BLEND_RGBA_SUB);
            let mut q = Surface::with_flags((9, 9), 0);
            q = muster_auf(q, 9, 9);
            s.blit_ex(&q, (9, 1), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            {
                let mut teil = s.subsurface(Rect::new(3, 4, 3, 4));
                teil.fill((106u8, 250u8, 187u8, 255u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x204BBABC17C9B753, flaeche: s }
    }));
    v.push((3, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((13, 24), 0);
            s.fill((189u8, 242u8, 33u8, 255u8));
            s.fill_rect((119u8, 98u8, 240u8, 132u8), Rect::new(10, 23, 15, 12));
            rustgame_core::draw::rect(&mut s, (77u8, 199u8, 7u8, 255u8), Rect::new(14, -8, 5, 24), 4);
            s.scroll(-13, -7);
            let mut q = Surface::with_flags((13, 14), 0);
            q = muster_auf(q, 13, 14);
            s.blit_ex(&q, (14, 24), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            rustgame_core::draw::line(&mut s, (69u8, 253u8, 111u8, 255u8), (4, 23), (16, 15), 4);
            let mut q = Surface::with_flags((8, 20), SRCALPHA);
            q = muster_auf(q, 8, 20);
            s.blit_ex(&q, (6, -3), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            s = rustgame_core::transform::flip(&s, false, false);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x5D0142EB9F6BEB47, flaeche: s }
    }));
    v.push((4, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((13, 15), SRCALPHA);
            s.fill((79u8, 46u8, 34u8, 255u8));
            let mut q = Surface::with_flags((6, 3), SRCALPHA);
            q = muster_auf(q, 6, 3);
            q.set_colorkey(Some((88u8, 54u8, 134u8)));
            s.blit_ex(&q, (2, -4), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            rustgame_core::draw::ellipse(&mut s, (158u8, 148u8, 190u8, 255u8), Rect::new(-5, 11, 10, 12), 4);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x85833DE6F9DEF79F, flaeche: s }
    }));
    v.push((5, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((25, 14), 0);
            s.fill((14u8, 238u8, 127u8, 255u8));
            s.fill_rect((190u8, 240u8, 126u8, 255u8), Rect::new(11, 10, 3, 7));
            s.fill_rect((143u8, 93u8, 199u8, 255u8), Rect::new(-3, -5, 4, 14));
            s.set_clip(Some(Rect::new(0, 13, 1, 1)));
            rustgame_core::draw::circle(&mut s, (85u8, 148u8, 160u8, 84u8), (8, 13), 21, 1);
            {
                let mut teil = s.subsurface(Rect::new(22, 3, 2, 5));
                teil.fill((212u8, 84u8, 74u8, 255u8));
            }
            {
                let mut teil = s.subsurface(Rect::new(10, 4, 14, 10));
                teil.fill((33u8, 158u8, 181u8, 173u8));
            }
            rustgame_core::draw::ellipse(&mut s, (94u8, 246u8, 241u8, 161u8), Rect::new(-2, -6, 8, 0), 2);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xD1EF44BA663E6D20, flaeche: s }
    }));
    v.push((6, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((31, 24), 0);
            s.fill((248u8, 133u8, 18u8, 255u8));
            s.set_clip(Some(Rect::new(18, 15, 13, 6)));
            s.scroll(-30, -7);
            s = rustgame_core::transform::grayscale(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x3795292C48884325, flaeche: s }
    }));
    v.push((7, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((16, 10), SRCALPHA);
            s.fill((48u8, 187u8, 29u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(2, 6, 7, 1));
                teil.fill((217u8, 30u8, 63u8, 255u8));
            }
            rustgame_core::draw::polygon(&mut s, (25u8, 113u8, 23u8, 203u8), &[(13, 0), (5, 9), (0, 13)], 0);
            rustgame_core::draw::rect_radius(&mut s, (52u8, 96u8, 190u8, 92u8), Rect::new(-5, -3, 18, 0), 4, 2);
            let mut q = Surface::with_flags((15, 7), SRCALPHA);
            q = muster_auf(q, 15, 7);
            s.blit_ex(&q, (3, 1), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            s.fill_rect((253u8, 175u8, 229u8, 255u8), Rect::new(1, -3, 3, 8));
            let mut q = Surface::with_flags((12, 4), 0);
            q = muster_auf(q, 12, 4);
            s.blit_ex(&q, (-2, 13), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xAAB928C77387392C, flaeche: s }
    }));
    v.push((8, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((13, 17), 0);
            s.fill((64u8, 98u8, 22u8, 255u8));
            s.set_clip(Some(Rect::new(12, 16, 1, 1)));
            rustgame_core::draw::rect_radius(&mut s, (199u8, 253u8, 98u8, 232u8), Rect::new(5, -7, 15, 7), 0, 3);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xA67F87F6112D593C, flaeche: s }
    }));
    v.push((9, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((20, 25), SRCALPHA);
            s.fill((3u8, 173u8, 237u8, 255u8));
            s.fill_rect((194u8, 86u8, 231u8, 20u8), Rect::new(17, -3, 5, 7));
            s.fill_rect((32u8, 196u8, 52u8, 255u8), Rect::new(8, 0, 21, 7));
            rustgame_core::draw::circle(&mut s, (136u8, 107u8, 203u8, 44u8), (4, 17), 1, 1);
            rustgame_core::draw::circle(&mut s, (210u8, 28u8, 193u8, 255u8), (23, 27), 4, 0);
            {
                let mut teil = s.subsurface(Rect::new(3, 19, 1, 1));
                teil.fill((103u8, 169u8, 4u8, 101u8));
            }
            rustgame_core::draw::line(&mut s, (8u8, 43u8, 254u8, 255u8), (13, 8), (9, 0), 4);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x560BB0DA37E13A84, flaeche: s }
    }));
    v.push((10, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((24, 7), SRCALPHA);
            s.fill((236u8, 251u8, 142u8, 105u8));
            rustgame_core::draw::ellipse(&mut s, (250u8, 167u8, 38u8, 255u8), Rect::new(3, 7, 11, 0), 3);
            rustgame_core::draw::line(&mut s, (215u8, 145u8, 134u8, 195u8), (25, -2), (15, 6), 3);
            s.set_clip(Some(Rect::new(7, 3, 13, 1)));
            rustgame_core::draw::rect_radius(&mut s, (99u8, 155u8, 187u8, 255u8), Rect::new(3, 1, 21, 8), 3, 5);
            let mut q = Surface::with_flags((22, 7), 0);
            q = muster_auf(q, 22, 7);
            q.set_alpha(Some(114));
            s.blit_ex(&q, (11, -4), Some(Rect::new(8, 5, 21, 1)), rustgame_core::blend::flags::BLEND_RGBA_MULT);
            rustgame_core::draw::line(&mut s, (196u8, 191u8, 76u8, 255u8), (3, 8), (2, 3), 2);
            rustgame_core::draw::polygon(&mut s, (212u8, 228u8, 125u8, 221u8), &[(13, -2), (7, 9), (3, 0), (25, 0)], 1);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xCFF8DA3285C5E424, flaeche: s }
    }));
    v.push((11, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((20, 23), 0);
            s.fill((238u8, 231u8, 97u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (48u8, 228u8, 155u8, 95u8), Rect::new(-1, -7, 17, 25), 0);
            rustgame_core::draw::rect(&mut s, (231u8, 80u8, 7u8, 255u8), Rect::new(-6, -9, 1, 6), 1);
            rustgame_core::draw::circle(&mut s, (100u8, 119u8, 150u8, 255u8), (11, -4), 21, 0);
            s = rustgame_core::transform::flip(&s, true, true);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xD19D948ABBF1B024, flaeche: s }
    }));
    v.push((12, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 14), 0);
            s.fill((179u8, 73u8, 195u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (235u8, 116u8, 0u8, 255u8), &[(17, 15), (0, 10), (7, 1), (6, 2), (-3, 14), (22, 2)], 0);
            let mut q = Surface::with_flags((12, 12), SRCALPHA);
            q = muster_auf(q, 12, 12);
            q.set_colorkey(Some((31u8, 114u8, 46u8)));
            s.blit_ex(&q, (9, 10), Some(Rect::new(7, -2, 10, 5)), 0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x22DFBB25CE549FDD, flaeche: s }
    }));
    v.push((13, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((14, 15), 0);
            s.fill((95u8, 118u8, 75u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(11, 2, 1, 2));
                teil.fill((150u8, 15u8, 220u8, 109u8));
            }
            s.set_clip(Some(Rect::new(10, 9, 1, 3)));
            rustgame_core::draw::ellipse(&mut s, (134u8, 230u8, 223u8, 255u8), Rect::new(-3, 0, 11, 7), 3);
            rustgame_core::draw::line(&mut s, (186u8, 220u8, 160u8, 219u8), (16, -1), (7, 15), 3);
            rustgame_core::draw::circle(&mut s, (225u8, 227u8, 90u8, 69u8), (5, 2), 2, 4);
            let mut q = Surface::with_flags((7, 16), SRCALPHA);
            q = muster_auf(q, 7, 16);
            q.set_alpha(Some(66));
            s.blit_ex(&q, (13, 3), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            rustgame_core::draw::circle(&mut s, (220u8, 237u8, 199u8, 255u8), (-4, 9), 14, 0);
            {
                let mut teil = s.subsurface(Rect::new(7, 12, 6, 2));
                teil.fill((211u8, 128u8, 179u8, 81u8));
            }
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x0ED7716492E98A09, flaeche: s }
    }));
    v.push((14, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((9, 25), 0);
            s.fill((126u8, 138u8, 130u8, 255u8));
            rustgame_core::draw::circle(&mut s, (238u8, 203u8, 201u8, 155u8), (-1, 12), 7, 2);
            let mut q = Surface::with_flags((6, 13), 0);
            q = muster_auf(q, 6, 13);
            q.set_colorkey(Some((140u8, 84u8, 5u8)));
            s.blit_ex(&q, (-2, 3), None, 0);
            s.fill_rect((195u8, 206u8, 224u8, 255u8), Rect::new(-2, -6, 9, 18));
            rustgame_core::draw::rect_radius(&mut s, (48u8, 249u8, 99u8, 255u8), Rect::new(3, 15, 9, 6), 3, 3);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x350FDFF9E998115E, flaeche: s }
    }));
    v.push((15, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 6), 0);
            s.fill((18u8, 80u8, 122u8, 255u8));
            s.fill_rect((188u8, 122u8, 59u8, 75u8), Rect::new(4, 4, 11, 4));
            let mut q = Surface::with_flags((7, 3), 0);
            q = muster_auf(q, 7, 3);
            q.set_alpha(Some(181));
            s.blit_ex(&q, (3, 0), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x643EBD2CD831A8DF, flaeche: s }
    }));
    v.push((16, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((17, 21), SRCALPHA);
            s.fill((2u8, 209u8, 132u8, 228u8));
            rustgame_core::draw::rect_radius(&mut s, (154u8, 171u8, 72u8, 255u8), Rect::new(14, 8, 0, 7), 4, 3);
            s.fill_rect((14u8, 237u8, 233u8, 255u8), Rect::new(10, 7, 7, 9));
            let mut q = Surface::with_flags((15, 4), SRCALPHA);
            q = muster_auf(q, 15, 4);
            s.blit_ex(&q, (14, 5), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x61C5E80F451A6B18, flaeche: s }
    }));
    v.push((17, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((22, 19), 0);
            s.fill((187u8, 148u8, 89u8, 255u8));
            rustgame_core::draw::circle(&mut s, (56u8, 13u8, 127u8, 142u8), (8, 22), 13, 2);
            rustgame_core::draw::ellipse(&mut s, (70u8, 31u8, 71u8, 205u8), Rect::new(1, -6, 22, 17), 4);
            rustgame_core::draw::ellipse(&mut s, (63u8, 35u8, 158u8, 169u8), Rect::new(15, -8, 16, 15), 4);
            s.set_clip(Some(Rect::new(13, 16, 6, 1)));
            let mut q = Surface::with_flags((20, 3), 0);
            q = muster_auf(q, 20, 3);
            q.set_colorkey(Some((244u8, 191u8, 2u8)));
            s.blit_ex(&q, (23, 8), Some(Rect::new(3, -1, 16, 2)), 0);
            {
                let mut teil = s.subsurface(Rect::new(8, 6, 12, 10));
                teil.fill((128u8, 157u8, 178u8, 66u8));
            }
            s.scroll(-1, -13);
            rustgame_core::draw::line(&mut s, (171u8, 4u8, 25u8, 139u8), (20, 7), (-4, 10), 3);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xB80C981A2A7282D8, flaeche: s }
    }));
    v.push((18, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((11, 9), 0);
            s.fill((171u8, 122u8, 101u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (151u8, 234u8, 135u8, 255u8), Rect::new(0, 3, 14, 11), 0, 4);
            rustgame_core::draw::rect_radius(&mut s, (120u8, 86u8, 121u8, 89u8), Rect::new(0, 6, 9, 8), 1, 6);
            {
                let mut teil = s.subsurface(Rect::new(4, 4, 1, 3));
                teil.fill((207u8, 177u8, 139u8, 112u8));
            }
            rustgame_core::draw::ellipse(&mut s, (58u8, 144u8, 244u8, 155u8), Rect::new(12, -1, 7, 10), 1);
            {
                let mut teil = s.subsurface(Rect::new(10, 6, 1, 2));
                teil.fill((103u8, 132u8, 173u8, 51u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x8F17021C39D44183, flaeche: s }
    }));
    v.push((19, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((27, 7), 0);
            s.fill((61u8, 102u8, 201u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (55u8, 133u8, 210u8, 133u8), &[(16, 0), (2, 8), (16, 10), (15, -4), (8, 10)], 0);
            rustgame_core::draw::line(&mut s, (233u8, 198u8, 37u8, 255u8), (2, 2), (-3, -3), 5);
            rustgame_core::draw::circle(&mut s, (225u8, 153u8, 81u8, 202u8), (22, -1), 24, 4);
            rustgame_core::draw::rect_radius(&mut s, (106u8, 199u8, 98u8, 255u8), Rect::new(-6, -2, 20, 7), 0, 6);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xD63ED75FDF18B8B0, flaeche: s }
    }));
    v.push((20, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((29, 27), 0);
            s.fill((77u8, 133u8, 51u8, 255u8));
            s.scroll(28, -17);
            s.fill_rect((52u8, 64u8, 163u8, 38u8), Rect::new(15, 23, 28, 13));
            {
                let mut teil = s.subsurface(Rect::new(10, 20, 11, 3));
                teil.fill((207u8, 41u8, 105u8, 251u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (48u8, 42u8, 97u8, 255u8), Rect::new(1, 4, 19, 22), 2, 1);
            rustgame_core::draw::rect_radius(&mut s, (119u8, 112u8, 121u8, 136u8), Rect::new(-12, -14, 19, 8), 1, 4);
            rustgame_core::draw::rect_radius(&mut s, (140u8, 221u8, 80u8, 78u8), Rect::new(-5, 11, 7, 22), 1, 5);
            let mut q = Surface::with_flags((5, 21), SRCALPHA);
            q = muster_auf(q, 5, 21);
            q.set_alpha(Some(88));
            s.blit_ex(&q, (1, 3), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            s.fill_rect((205u8, 234u8, 27u8, 161u8), Rect::new(27, -4, 7, 30));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x005D0737C32C9E4F, flaeche: s }
    }));
    v.push((21, true, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((11, 19), 0);
            s.fill((144u8, 245u8, 110u8, 255u8));
            let mut q = Surface::with_flags((10, 7), 0);
            q = muster_auf(q, 10, 7);
            q.set_alpha(Some(1));
            q.set_colorkey(Some((219u8, 35u8, 74u8)));
            s.blit_ex(&q, (3, 3), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            rustgame_core::draw::polygon(&mut s, (168u8, 254u8, 59u8, 17u8), &[(7, -4), (0, 19), (-2, -1), (-4, 10), (1, 13), (6, 8)], 3);
            s.set_clip(Some(Rect::new(10, 1, 1, 8)));
            s.fill_rect((36u8, 62u8, 243u8, 255u8), Rect::new(11, 16, 6, 6));
            let mut q = Surface::with_flags((4, 19), SRCALPHA);
            q = muster_auf(q, 4, 19);
            q.set_alpha(Some(231));
            s.blit_ex(&q, (6, 11), None, 0);
            rustgame_core::draw::ellipse(&mut s, (253u8, 1u8, 251u8, 255u8), Rect::new(-4, 7, 9, 16), 1);
            rustgame_core::draw::rect_radius(&mut s, (57u8, 92u8, 87u8, 255u8), Rect::new(5, 16, 14, 10), 2, 3);
            {
                let mut teil = s.subsurface(Rect::new(2, 2, 3, 14));
                teil.fill((18u8, 123u8, 208u8, 255u8));
            }
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xA00A4E096D38C8A0, flaeche: s }
    }));
    v.push((22, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((10, 13), SRCALPHA);
            s.fill((61u8, 176u8, 40u8, 94u8));
            {
                let mut teil = s.subsurface(Rect::new(0, 5, 10, 3));
                teil.fill((24u8, 10u8, 135u8, 220u8));
            }
            s.scroll(-4, -8);
            rustgame_core::draw::ellipse(&mut s, (167u8, 171u8, 132u8, 31u8), Rect::new(0, 5, 4, 16), 1);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xDB11CA16187CD68D, flaeche: s }
    }));
    v.push((23, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((30, 15), 0);
            s.fill((42u8, 8u8, 157u8, 255u8));
            let mut q = Surface::with_flags((13, 6), 0);
            q = muster_auf(q, 13, 6);
            q.set_alpha(Some(7));
            q.set_colorkey(Some((232u8, 12u8, 54u8)));
            s.blit_ex(&q, (1, 18), None, 0);
            let mut q = Surface::with_flags((25, 15), SRCALPHA);
            q = muster_auf(q, 25, 15);
            q.set_alpha(Some(18));
            s.blit_ex(&q, (7, 15), Some(Rect::new(10, 10, 24, 11)), rustgame_core::blend::flags::BLEND_RGBA_MIN);
            s.scroll(-10, 3);
            s.fill_rect((229u8, 83u8, 93u8, 247u8), Rect::new(-4, 9, 20, 16));
            {
                let mut teil = s.subsurface(Rect::new(3, 1, 4, 7));
                teil.fill((141u8, 143u8, 169u8, 255u8));
            }
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x007763475A22AB89, flaeche: s }
    }));
    v.push((24, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((28, 18), 0);
            s.fill((93u8, 111u8, 85u8, 255u8));
            s.set_clip(Some(Rect::new(21, 2, 6, 5)));
            rustgame_core::draw::ellipse(&mut s, (6u8, 225u8, 239u8, 255u8), Rect::new(26, -6, 1, 16), 1);
            rustgame_core::draw::polygon(&mut s, (43u8, 130u8, 80u8, 255u8), &[(16, 5), (0, 13), (19, 17), (-2, 2), (16, 6), (0, 5)], 0);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xBF4BCDCFF75CC936, flaeche: s }
    }));
    v.push((25, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((18, 30), 0);
            s.fill((7u8, 109u8, 156u8, 255u8));
            let mut q = Surface::with_flags((10, 3), SRCALPHA);
            q = muster_auf(q, 10, 3);
            s.blit_ex(&q, (-1, 32), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            let mut q = Surface::with_flags((12, 28), 0);
            q = muster_auf(q, 12, 28);
            s.blit_ex(&q, (12, 2), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            rustgame_core::draw::circle(&mut s, (212u8, 180u8, 96u8, 182u8), (17, 6), 19, 3);
            s.fill_rect((65u8, 20u8, 88u8, 35u8), Rect::new(-7, 22, 7, 27));
            rustgame_core::draw::line(&mut s, (247u8, 226u8, 29u8, 252u8), (11, 14), (1, -1), 4);
            rustgame_core::draw::ellipse(&mut s, (234u8, 9u8, 244u8, 255u8), Rect::new(-2, 12, 19, 22), 0);
            rustgame_core::draw::ellipse(&mut s, (119u8, 95u8, 69u8, 255u8), Rect::new(-2, -6, 11, 13), 1);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xA243EA15FD3D87D5, flaeche: s }
    }));
    v.push((26, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((29, 12), 0);
            s.fill((221u8, 29u8, 65u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (87u8, 219u8, 124u8, 255u8), &[(22, 2), (-3, 3), (5, 0), (9, 15), (26, 8), (19, 3)], 0);
            s.fill_rect((22u8, 3u8, 237u8, 63u8), Rect::new(23, 11, 20, 10));
            rustgame_core::draw::ellipse(&mut s, (28u8, 128u8, 101u8, 102u8), Rect::new(24, 6, 27, 4), 0);
            s.fill_rect((22u8, 24u8, 208u8, 255u8), Rect::new(-2, 5, 5, 10));
            s.set_clip(Some(Rect::new(25, 0, 4, 12)));
            let mut q = Surface::with_flags((4, 2), SRCALPHA);
            q = muster_auf(q, 4, 2);
            s.blit_ex(&q, (4, 4), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            rustgame_core::draw::circle(&mut s, (177u8, 240u8, 199u8, 223u8), (-2, -3), 4, 4);
            s.fill_rect((243u8, 17u8, 93u8, 255u8), Rect::new(24, -5, 1, 13));
            s = s.convert_alpha();
            s = rustgame_core::transform::scale(&s, (20, 7));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x5C05BD2EF38C7A6B, flaeche: s }
    }));
    v.push((27, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((26, 21), 0);
            s.fill((146u8, 100u8, 37u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(17, 10, 5, 6));
                teil.fill((93u8, 126u8, 122u8, 206u8));
            }
            let mut q = Surface::with_flags((4, 22), 0);
            q = muster_auf(q, 4, 22);
            q.set_alpha(Some(217));
            s.blit_ex(&q, (-1, 17), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            s = s.convert_alpha();
            s = rustgame_core::transform::flip(&s, false, true);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x69DA2D7AB0EB8491, flaeche: s }
    }));
    v.push((28, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((9, 29), SRCALPHA);
            s.fill((115u8, 67u8, 237u8, 91u8));
            let mut q = Surface::with_flags((5, 6), SRCALPHA);
            q = muster_auf(q, 5, 6);
            q.set_colorkey(Some((96u8, 103u8, 14u8)));
            s.blit_ex(&q, (9, 11), Some(Rect::new(3, 2, 2, 2)), rustgame_core::blend::flags::BLEND_RGBA_MULT);
            let mut q = Surface::with_flags((6, 21), SRCALPHA);
            q = muster_auf(q, 6, 21);
            q.set_colorkey(Some((6u8, 145u8, 133u8)));
            s.blit_ex(&q, (-2, 29), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            rustgame_core::draw::circle(&mut s, (103u8, 188u8, 35u8, 85u8), (-3, 23), 24, 3);
            let mut q = Surface::with_flags((8, 27), SRCALPHA);
            q = muster_auf(q, 8, 27);
            s.blit_ex(&q, (1, 10), Some(Rect::new(8, 23, 3, 23)), rustgame_core::blend::flags::BLEND_RGBA_MULT);
            let mut q = Surface::with_flags((2, 9), SRCALPHA);
            q = muster_auf(q, 2, 9);
            s.blit_ex(&q, (8, 2), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            rustgame_core::draw::circle(&mut s, (218u8, 81u8, 213u8, 229u8), (3, 25), 27, 2);
            let mut q = Surface::with_flags((5, 3), SRCALPHA);
            q = muster_auf(q, 5, 3);
            s.blit_ex(&q, (10, 4), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            rustgame_core::draw::rect_radius(&mut s, (187u8, 184u8, 43u8, 233u8), Rect::new(2, 10, 3, 6), 4, 1);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x5F62D588EAC052B1, flaeche: s }
    }));
    v.push((29, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((23, 8), SRCALPHA);
            s.fill((44u8, 176u8, 201u8, 147u8));
            s.fill_rect((227u8, 115u8, 178u8, 255u8), Rect::new(-7, 3, 10, 9));
            rustgame_core::draw::rect_radius(&mut s, (228u8, 87u8, 112u8, 102u8), Rect::new(15, 8, 13, 10), 4, 2);
            rustgame_core::draw::rect_radius(&mut s, (112u8, 210u8, 241u8, 142u8), Rect::new(21, -2, 24, 4), 2, 6);
            let mut q = Surface::with_flags((21, 3), 0);
            q = muster_auf(q, 21, 3);
            s.blit_ex(&q, (4, 0), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            rustgame_core::draw::rect_radius(&mut s, (218u8, 238u8, 144u8, 255u8), Rect::new(-3, 1, 23, 11), 1, 3);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xB4CD1219AD722C83, flaeche: s }
    }));
    v.push((30, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((23, 31), SRCALPHA);
            s.fill((107u8, 131u8, 24u8, 255u8));
            let mut q = Surface::with_flags((6, 7), SRCALPHA);
            q = muster_auf(q, 6, 7);
            s.blit_ex(&q, (3, -3), Some(Rect::new(-1, 5, 8, 6)), rustgame_core::blend::flags::BLEND_RGBA_MAX);
            let mut q = Surface::with_flags((4, 27), SRCALPHA);
            q = muster_auf(q, 4, 27);
            s.blit_ex(&q, (20, 23), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            {
                let mut teil = s.subsurface(Rect::new(17, 28, 1, 3));
                teil.fill((66u8, 158u8, 159u8, 158u8));
            }
            {
                let mut teil = s.subsurface(Rect::new(2, 4, 8, 23));
                teil.fill((14u8, 209u8, 154u8, 255u8));
            }
            s.set_clip(Some(Rect::new(20, 5, 2, 21)));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x3138D16525F71DC4, flaeche: s }
    }));
    v.push((31, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((6, 21), SRCALPHA);
            s.fill((22u8, 71u8, 57u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(1, 4, 1, 2));
                teil.fill((229u8, 211u8, 104u8, 255u8));
            }
            rustgame_core::draw::polygon(&mut s, (206u8, 174u8, 101u8, 255u8), &[(-1, 6), (2, 7), (5, 3)], 1);
            rustgame_core::draw::circle(&mut s, (26u8, 92u8, 182u8, 30u8), (-2, 2), 4, 3);
            s.fill_rect((111u8, 106u8, 221u8, 224u8), Rect::new(2, 1, 5, 20));
            rustgame_core::draw::ellipse(&mut s, (172u8, 150u8, 171u8, 134u8), Rect::new(6, 14, 7, 20), 2);
            rustgame_core::draw::polygon(&mut s, (90u8, 151u8, 47u8, 156u8), &[(-2, 2), (-4, 21), (5, 7)], 0);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xDF6D292A47AEA4C0, flaeche: s }
    }));
    v.push((32, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 12), SRCALPHA);
            s.fill((254u8, 12u8, 19u8, 121u8));
            s.scroll(2, -11);
            rustgame_core::draw::rect_radius(&mut s, (189u8, 0u8, 64u8, 240u8), Rect::new(4, -6, 7, 6), 2, 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x375CACCDD5414B65, flaeche: s }
    }));
    v.push((33, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((24, 11), 0);
            s.fill((141u8, 245u8, 95u8, 255u8));
            rustgame_core::draw::line(&mut s, (246u8, 227u8, 144u8, 164u8), (0, 5), (22, 5), 5);
            let mut q = Surface::with_flags((3, 13), 0);
            q = muster_auf(q, 3, 13);
            s.blit_ex(&q, (13, 6), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            {
                let mut teil = s.subsurface(Rect::new(1, 5, 10, 2));
                teil.fill((187u8, 63u8, 164u8, 33u8));
            }
            rustgame_core::draw::circle(&mut s, (3u8, 181u8, 155u8, 163u8), (21, 8), 5, 0);
            rustgame_core::draw::line(&mut s, (63u8, 32u8, 76u8, 255u8), (1, -1), (16, -4), 1);
            let mut q = Surface::with_flags((13, 5), 0);
            q = muster_auf(q, 13, 5);
            s.blit_ex(&q, (26, 7), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
            rustgame_core::draw::ellipse(&mut s, (35u8, 4u8, 246u8, 255u8), Rect::new(12, 6, 24, 12), 2);
            s = s.convert();
            s = rustgame_core::transform::flip(&s, true, true);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x2844B3D2C4A0FD62, flaeche: s }
    }));
    v.push((34, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((22, 17), 0);
            s.fill((117u8, 15u8, 199u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (174u8, 48u8, 79u8, 156u8), &[(-1, 4), (7, 15), (-4, 20)], 1);
            rustgame_core::draw::circle(&mut s, (31u8, 189u8, 130u8, 255u8), (19, 13), 1, 3);
            rustgame_core::draw::polygon(&mut s, (160u8, 108u8, 105u8, 255u8), &[(10, 13), (1, 15), (0, 14), (18, 8), (13, 9)], 1);
            {
                let mut teil = s.subsurface(Rect::new(3, 2, 15, 5));
                teil.fill((101u8, 89u8, 207u8, 144u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x8EB50B4791A6D285, flaeche: s }
    }));
    v.push((35, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((23, 16), 0);
            s.fill((174u8, 78u8, 146u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (187u8, 49u8, 186u8, 140u8), &[(-4, 12), (-4, -1), (19, 11)], 2);
            rustgame_core::draw::circle(&mut s, (254u8, 26u8, 11u8, 255u8), (3, 6), 2, 1);
            let mut q = Surface::with_flags((20, 4), SRCALPHA);
            q = muster_auf(q, 20, 4);
            q.set_alpha(Some(150));
            q.set_colorkey(Some((39u8, 66u8, 225u8)));
            s.blit_ex(&q, (22, 11), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            let mut q = Surface::with_flags((10, 16), 0);
            q = muster_auf(q, 10, 16);
            s.blit_ex(&q, (22, -1), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            let mut q = Surface::with_flags((4, 14), SRCALPHA);
            q = muster_auf(q, 4, 14);
            q.set_alpha(Some(198));
            s.blit_ex(&q, (4, 14), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x56484730E23380C8, flaeche: s }
    }));
    v.push((36, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((16, 7), 0);
            s.fill((145u8, 40u8, 0u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (221u8, 186u8, 137u8, 255u8), Rect::new(9, -3, 18, 6), 3, 5);
            let mut q = Surface::with_flags((14, 5), SRCALPHA);
            q = muster_auf(q, 14, 5);
            q.set_alpha(Some(254));
            s.blit_ex(&q, (2, 4), Some(Rect::new(8, 6, 10, 8)), rustgame_core::blend::flags::BLEND_RGB_SUB);
            s.fill_rect((70u8, 111u8, 213u8, 253u8), Rect::new(3, 8, 6, 7));
            rustgame_core::draw::polygon(&mut s, (236u8, 151u8, 231u8, 120u8), &[(19, -3), (3, 0), (2, 5), (1, -2), (-2, 7)], 1);
            rustgame_core::draw::polygon(&mut s, (194u8, 178u8, 250u8, 8u8), &[(1, 4), (14, 7), (4, -3), (-1, 6), (10, -1), (3, 7)], 2);
            rustgame_core::draw::polygon(&mut s, (43u8, 242u8, 141u8, 92u8), &[(4, 0), (-4, 4), (5, 7), (13, 10)], 0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x42E74EDD8C0B152A, flaeche: s }
    }));
    v.push((37, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((27, 25), SRCALPHA);
            s.fill((189u8, 224u8, 55u8, 18u8));
            let mut q = Surface::with_flags((27, 11), 0);
            q = muster_auf(q, 27, 11);
            s.blit_ex(&q, (5, -1), Some(Rect::new(-10, 11, 9, 14)), rustgame_core::blend::flags::BLEND_RGB_MAX);
            rustgame_core::draw::rect_radius(&mut s, (17u8, 109u8, 213u8, 187u8), Rect::new(-10, -13, 0, 20), 4, 4);
            s.fill_rect((165u8, 180u8, 81u8, 12u8), Rect::new(-13, -13, 28, 6));
            rustgame_core::draw::circle(&mut s, (116u8, 154u8, 21u8, 255u8), (14, 13), 22, 1);
            let mut q = Surface::with_flags((8, 10), 0);
            q = muster_auf(q, 8, 10);
            q.set_alpha(Some(229));
            q.set_colorkey(Some((1u8, 136u8, 122u8)));
            s.blit_ex(&q, (1, 24), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            let mut q = Surface::with_flags((5, 6), SRCALPHA);
            q = muster_auf(q, 5, 6);
            q.set_alpha(Some(125));
            s.blit_ex(&q, (14, 26), Some(Rect::new(-1, 5, 4, 2)), rustgame_core::blend::flags::BLEND_RGB_MULT);
            s.set_clip(Some(Rect::new(5, 13, 19, 9)));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xADE8165F028EA2A1, flaeche: s }
    }));
    v.push((38, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((26, 19), SRCALPHA);
            s.fill((187u8, 237u8, 191u8, 33u8));
            rustgame_core::draw::rect_radius(&mut s, (176u8, 167u8, 136u8, 255u8), Rect::new(8, 9, 23, 9), 3, 5);
            {
                let mut teil = s.subsurface(Rect::new(19, 2, 5, 16));
                teil.fill((126u8, 5u8, 194u8, 255u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x621FD97A4B976B3D, flaeche: s }
    }));
    v.push((39, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 14), SRCALPHA);
            s.fill((202u8, 2u8, 130u8, 255u8));
            let mut q = Surface::with_flags((4, 15), SRCALPHA);
            q = muster_auf(q, 4, 15);
            s.blit_ex(&q, (7, -4), Some(Rect::new(3, 5, 1, 12)), rustgame_core::blend::flags::BLEND_RGBA_MULT);
            rustgame_core::draw::rect_radius(&mut s, (34u8, 235u8, 220u8, 255u8), Rect::new(2, 11, 10, 0), 0, 2);
            rustgame_core::draw::line(&mut s, (195u8, 160u8, 144u8, 246u8), (8, 16), (7, 11), 2);
            rustgame_core::draw::rect_radius(&mut s, (152u8, 147u8, 120u8, 255u8), Rect::new(-3, 5, 15, 16), 0, 5);
            s.fill_rect((2u8, 193u8, 64u8, 214u8), Rect::new(11, 12, 7, 3));
            rustgame_core::draw::rect_radius(&mut s, (123u8, 67u8, 45u8, 116u8), Rect::new(13, 8, 10, 16), 2, 2);
            rustgame_core::draw::ellipse(&mut s, (91u8, 87u8, 185u8, 72u8), Rect::new(4, 2, 5, 1), 0);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xC2B9EEAD00A30CEC, flaeche: s }
    }));
    v.push((40, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((20, 24), 0);
            s.fill((16u8, 125u8, 144u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (178u8, 141u8, 225u8, 255u8), &[(22, -3), (12, 4), (-3, 8), (15, 25)], 0);
            s.set_clip(Some(Rect::new(19, 10, 1, 5)));
            rustgame_core::draw::polygon(&mut s, (208u8, 25u8, 161u8, 49u8), &[(19, 17), (3, 5), (5, 6), (12, 15)], 2);
            let mut q = Surface::with_flags((7, 20), SRCALPHA);
            q = muster_auf(q, 7, 20);
            q.set_colorkey(Some((104u8, 4u8, 45u8)));
            s.blit_ex(&q, (10, 15), Some(Rect::new(-1, 19, 7, 3)), rustgame_core::blend::flags::BLEND_RGBA_SUB);
            rustgame_core::draw::polygon(&mut s, (145u8, 126u8, 35u8, 232u8), &[(-3, 5), (15, 20), (1, 24), (-1, 22), (2, -3)], 3);
            let mut q = Surface::with_flags((9, 9), SRCALPHA);
            q = muster_auf(q, 9, 9);
            q.set_colorkey(Some((81u8, 156u8, 129u8)));
            s.blit_ex(&q, (17, 1), Some(Rect::new(5, 0, 12, 3)), 0);
            {
                let mut teil = s.subsurface(Rect::new(1, 16, 8, 4));
                teil.fill((243u8, 159u8, 158u8, 220u8));
            }
            s = rustgame_core::transform::flip(&s, true, false);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x136024FCE6FF3AE1, flaeche: s }
    }));
    v.push((41, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((18, 16), SRCALPHA);
            s.fill((197u8, 145u8, 141u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (127u8, 9u8, 224u8, 4u8), Rect::new(-5, -4, 10, 5), 2);
            rustgame_core::draw::rect_radius(&mut s, (61u8, 16u8, 221u8, 30u8), Rect::new(18, 1, 6, 2), 2, 6);
            s.set_clip(Some(Rect::new(0, 3, 4, 3)));
            rustgame_core::draw::circle(&mut s, (136u8, 162u8, 115u8, 27u8), (15, -4), 5, 3);
            rustgame_core::draw::line(&mut s, (204u8, 74u8, 54u8, 154u8), (17, -2), (9, -4), 4);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xB3B52509A94E3FEB, flaeche: s }
    }));
    v.push((42, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((26, 9), SRCALPHA);
            s.fill((114u8, 71u8, 52u8, 255u8));
            rustgame_core::draw::circle(&mut s, (16u8, 15u8, 47u8, 216u8), (9, 3), 16, 4);
            s.fill_rect((214u8, 112u8, 229u8, 255u8), Rect::new(24, 3, 25, 0));
            rustgame_core::draw::circle(&mut s, (216u8, 174u8, 142u8, 255u8), (5, 2), 24, 2);
            s.set_clip(Some(Rect::new(12, 1, 6, 6)));
            rustgame_core::draw::rect_radius(&mut s, (63u8, 193u8, 40u8, 235u8), Rect::new(22, 4, 26, 10), 4, 4);
            rustgame_core::draw::rect_radius(&mut s, (116u8, 148u8, 40u8, 23u8), Rect::new(1, -2, 12, 4), 3, 4);
            s.set_clip(Some(Rect::new(11, 3, 11, 3)));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x1DC6EB901AFC8565, flaeche: s }
    }));
    v.push((43, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 15), 0);
            s.fill((73u8, 236u8, 189u8, 255u8));
            rustgame_core::draw::circle(&mut s, (255u8, 9u8, 221u8, 255u8), (5, 7), 9, 4);
            rustgame_core::draw::line(&mut s, (90u8, 29u8, 54u8, 201u8), (9, -1), (2, 0), 1);
            s.set_clip(Some(Rect::new(6, 7, 1, 7)));
            rustgame_core::draw::rect_radius(&mut s, (192u8, 32u8, 25u8, 93u8), Rect::new(5, -7, 2, 1), 1, 3);
            {
                let mut teil = s.subsurface(Rect::new(0, 8, 5, 1));
                teil.fill((48u8, 234u8, 169u8, 31u8));
            }
            s.scroll(3, 7);
            let mut q = Surface::with_flags((4, 4), 0);
            q = muster_auf(q, 4, 4);
            q.set_alpha(Some(137));
            s.blit_ex(&q, (-2, 17), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            s = rustgame_core::transform::scale(&s, (7, 15));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x98A8BC0E7A78BF1F, flaeche: s }
    }));
    v.push((44, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((19, 22), 0);
            s.fill((59u8, 90u8, 194u8, 255u8));
            s.scroll(-5, -15);
            rustgame_core::draw::rect_radius(&mut s, (155u8, 193u8, 207u8, 255u8), Rect::new(14, 9, 11, 2), 2, 3);
            rustgame_core::draw::rect(&mut s, (35u8, 48u8, 138u8, 255u8), Rect::new(20, 9, 3, 20), 2);
            s = rustgame_core::transform::scale(&s, (17, 8));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x6458D0C05A9A765D, flaeche: s }
    }));
    v.push((45, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((14, 19), SRCALPHA);
            s.fill((173u8, 11u8, 37u8, 255u8));
            s.fill_rect((60u8, 159u8, 18u8, 255u8), Rect::new(12, -8, 4, 8));
            rustgame_core::draw::ellipse(&mut s, (219u8, 38u8, 208u8, 108u8), Rect::new(-3, 12, 1, 1), 1);
            s.scroll(-3, 1);
            s.fill_rect((168u8, 162u8, 198u8, 255u8), Rect::new(15, 18, 16, 21));
            rustgame_core::draw::rect_radius(&mut s, (30u8, 252u8, 191u8, 130u8), Rect::new(-1, 1, 4, 20), 0, 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xCA28FE603FA1654A, flaeche: s }
    }));
    v.push((46, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 18), SRCALPHA);
            s.fill((77u8, 16u8, 15u8, 117u8));
            rustgame_core::draw::ellipse(&mut s, (15u8, 69u8, 151u8, 27u8), Rect::new(4, 1, 11, 2), 3);
            rustgame_core::draw::polygon(&mut s, (241u8, 238u8, 38u8, 255u8), &[(10, 19), (-1, 9), (3, 6), (10, 20), (4, 0), (-1, 15)], 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x4F989B3F057BA3DF, flaeche: s }
    }));
    v.push((47, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((17, 8), SRCALPHA);
            s.fill((131u8, 197u8, 200u8, 175u8));
            s.fill_rect((52u8, 0u8, 135u8, 12u8), Rect::new(16, 3, 17, 0));
            rustgame_core::draw::line(&mut s, (117u8, 121u8, 183u8, 218u8), (10, 3), (-3, 7), 3);
            rustgame_core::draw::rect_radius(&mut s, (122u8, 18u8, 64u8, 100u8), Rect::new(1, -3, 11, 9), 3, 3);
            s.fill_rect((171u8, 111u8, 241u8, 116u8), Rect::new(14, -3, 13, 9));
            let mut q = Surface::with_flags((11, 5), 0);
            q = muster_auf(q, 11, 5);
            s.blit_ex(&q, (14, -3), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            rustgame_core::draw::circle(&mut s, (113u8, 58u8, 106u8, 255u8), (8, 11), 8, 4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x7791DE4287AA4D84, flaeche: s }
    }));
    v.push((48, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((23, 16), SRCALPHA);
            s.fill((98u8, 223u8, 84u8, 154u8));
            rustgame_core::draw::polygon(&mut s, (111u8, 79u8, 241u8, 112u8), &[(2, -4), (-2, 9), (-2, 0)], 3);
            rustgame_core::draw::circle(&mut s, (149u8, 173u8, 172u8, 28u8), (-1, 3), 8, 2);
            rustgame_core::draw::rect(&mut s, (70u8, 180u8, 223u8, 144u8), Rect::new(-1, -2, 12, 19), 2);
            s = rustgame_core::transform::flip(&s, true, true);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xBE1A6D0C3DE45AA6, flaeche: s }
    }));
    v.push((49, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 17), SRCALPHA);
            s.fill((25u8, 22u8, 141u8, 255u8));
            let mut q = Surface::with_flags((2, 19), 0);
            q = muster_auf(q, 2, 19);
            s.blit_ex(&q, (7, 8), Some(Rect::new(0, -7, 3, 12)), rustgame_core::blend::flags::BLEND_RGB_MAX);
            rustgame_core::draw::circle(&mut s, (204u8, 26u8, 211u8, 255u8), (2, 16), 2, 4);
            {
                let mut teil = s.subsurface(Rect::new(5, 4, 2, 4));
                teil.fill((223u8, 250u8, 255u8, 255u8));
            }
            s = rustgame_core::transform::flip(&s, true, false);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x9F3E4A8A7B0DBDDC, flaeche: s }
    }));
    v.push((50, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 14), SRCALPHA);
            s.fill((168u8, 43u8, 162u8, 242u8));
            rustgame_core::draw::circle(&mut s, (78u8, 177u8, 50u8, 255u8), (7, 6), 7, 1);
            s.fill_rect((45u8, 32u8, 171u8, 219u8), Rect::new(16, -7, 7, 10));
            {
                let mut teil = s.subsurface(Rect::new(13, 11, 2, 1));
                teil.fill((235u8, 64u8, 138u8, 138u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x50ECECD862CBF12B, flaeche: s }
    }));
    v.push((51, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((13, 32), 0);
            s.fill((82u8, 125u8, 118u8, 255u8));
            rustgame_core::draw::rect(&mut s, (237u8, 175u8, 184u8, 203u8), Rect::new(5, 1, 12, 0), 0);
            rustgame_core::draw::rect_radius(&mut s, (90u8, 177u8, 153u8, 255u8), Rect::new(4, 12, 6, 1), 3, 1);
            let mut q = Surface::with_flags((2, 8), 0);
            q = muster_auf(q, 2, 8);
            s.blit_ex(&q, (14, 2), Some(Rect::new(2, -4, 5, 6)), rustgame_core::blend::flags::BLEND_RGBA_ADD);
            rustgame_core::draw::line(&mut s, (57u8, 248u8, 91u8, 189u8), (14, 18), (10, 27), 1);
            let mut q = Surface::with_flags((9, 2), 0);
            q = muster_auf(q, 9, 2);
            s.blit_ex(&q, (11, 28), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            s.fill_rect((235u8, 14u8, 167u8, 157u8), Rect::new(6, 17, 2, 26));
            rustgame_core::draw::rect_radius(&mut s, (240u8, 247u8, 109u8, 115u8), Rect::new(10, -13, 10, 29), 2, 6);
            rustgame_core::draw::polygon(&mut s, (129u8, 246u8, 75u8, 150u8), &[(13, 23), (1, 3), (5, 25), (-4, 6), (9, 0)], 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x9B8534379D03C4F0, flaeche: s }
    }));
    v.push((52, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((14, 7), 0);
            s.fill((247u8, 189u8, 208u8, 255u8));
            rustgame_core::draw::line(&mut s, (211u8, 93u8, 177u8, 255u8), (10, 6), (-4, -4), 5);
            let mut q = Surface::with_flags((2, 5), SRCALPHA);
            q = muster_auf(q, 2, 5);
            q.set_alpha(Some(118));
            s.blit_ex(&q, (11, 8), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x942D0FEE61A92D14, flaeche: s }
    }));
    v.push((53, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((25, 12), SRCALPHA);
            s.fill((232u8, 15u8, 18u8, 184u8));
            rustgame_core::draw::polygon(&mut s, (28u8, 86u8, 24u8, 133u8), &[(19, 3), (24, 2), (10, 0), (4, 7), (28, 6)], 3);
            rustgame_core::draw::polygon(&mut s, (78u8, 46u8, 211u8, 255u8), &[(20, 10), (11, 0), (-1, 10)], 3);
            {
                let mut teil = s.subsurface(Rect::new(11, 9, 14, 1));
                teil.fill((16u8, 48u8, 26u8, 40u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x2E64686BEF29246F, flaeche: s }
    }));
    v.push((54, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((10, 20), 0);
            s.fill((247u8, 249u8, 194u8, 255u8));
            let mut q = Surface::with_flags((11, 14), 0);
            q = muster_auf(q, 11, 14);
            q.set_colorkey(Some((102u8, 28u8, 253u8)));
            s.blit_ex(&q, (13, 10), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            let mut q = Surface::with_flags((6, 15), SRCALPHA);
            q = muster_auf(q, 6, 15);
            q.set_alpha(Some(133));
            s.blit_ex(&q, (10, 14), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            rustgame_core::draw::circle(&mut s, (0u8, 117u8, 10u8, 255u8), (1, 19), 4, 3);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x0E68B71D14D791D2, flaeche: s }
    }));
    v.push((55, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 12), SRCALPHA);
            s.fill((94u8, 154u8, 45u8, 40u8));
            rustgame_core::draw::line(&mut s, (197u8, 198u8, 207u8, 241u8), (6, 9), (-3, 4), 1);
            rustgame_core::draw::polygon(&mut s, (26u8, 193u8, 118u8, 114u8), &[(-2, 12), (9, 2), (3, 2)], 3);
            s.set_clip(Some(Rect::new(5, 0, 1, 7)));
            s.scroll(3, 7);
            s = s.convert_alpha();
            s = rustgame_core::transform::rotate(&s, -186.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x6622BD8F52848221, flaeche: s }
    }));
    v.push((56, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((23, 6), SRCALPHA);
            s.fill((118u8, 48u8, 191u8, 215u8));
            s.fill_rect((11u8, 107u8, 27u8, 106u8), Rect::new(23, -1, 12, 0));
            rustgame_core::draw::line(&mut s, (126u8, 95u8, 48u8, 255u8), (15, -1), (13, -1), 4);
            {
                let mut teil = s.subsurface(Rect::new(12, 1, 11, 1));
                teil.fill((97u8, 89u8, 69u8, 92u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (2u8, 17u8, 188u8, 184u8), Rect::new(-12, 6, 21, 0), 4, 0);
            rustgame_core::draw::rect(&mut s, (184u8, 243u8, 190u8, 255u8), Rect::new(0, 5, 14, 8), 3);
            rustgame_core::draw::circle(&mut s, (175u8, 239u8, 73u8, 255u8), (-4, -1), 9, 1);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x614A03DCF4E44F3E, flaeche: s }
    }));
    v.push((57, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 17), 0);
            s.fill((9u8, 115u8, 164u8, 255u8));
            s.scroll(-1, 7);
            s.fill_rect((193u8, 203u8, 55u8, 190u8), Rect::new(6, 14, 9, 15));
            rustgame_core::draw::rect_radius(&mut s, (72u8, 71u8, 194u8, 71u8), Rect::new(4, -5, 9, 13), 4, 1);
            rustgame_core::draw::rect(&mut s, (112u8, 121u8, 142u8, 255u8), Rect::new(2, 12, 3, 13), 3);
            rustgame_core::draw::rect(&mut s, (25u8, 174u8, 224u8, 198u8), Rect::new(4, 15, 0, 14), 4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xED5C90033360BA5A, flaeche: s }
    }));
    v.push((58, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((24, 12), SRCALPHA);
            s.fill((21u8, 98u8, 230u8, 102u8));
            let mut q = Surface::with_flags((11, 14), 0);
            q = muster_auf(q, 11, 14);
            s.blit_ex(&q, (22, 5), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            s.fill_rect((37u8, 231u8, 192u8, 255u8), Rect::new(11, -5, 3, 4));
            s.scroll(-13, -3);
            s.scroll(15, 7);
            rustgame_core::draw::circle(&mut s, (81u8, 103u8, 229u8, 38u8), (4, 0), 4, 1);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xDD9D6B45D487568D, flaeche: s }
    }));
    v.push((59, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((13, 8), 0);
            s.fill((235u8, 11u8, 69u8, 255u8));
            s.scroll(-13, -8);
            rustgame_core::draw::line(&mut s, (152u8, 80u8, 173u8, 248u8), (1, -1), (-1, 7), 1);
            rustgame_core::draw::ellipse(&mut s, (23u8, 98u8, 42u8, 255u8), Rect::new(11, 5, 2, 4), 2);
            s.fill_rect((173u8, 65u8, 207u8, 54u8), Rect::new(-7, 8, 2, 7));
            s.scroll(3, -1);
            let mut q = Surface::with_flags((12, 8), 0);
            q = muster_auf(q, 12, 8);
            q.set_colorkey(Some((29u8, 197u8, 230u8)));
            s.blit_ex(&q, (12, 7), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            let mut q = Surface::with_flags((10, 4), SRCALPHA);
            q = muster_auf(q, 10, 4);
            q.set_alpha(Some(241));
            q.set_colorkey(Some((223u8, 119u8, 172u8)));
            s.blit_ex(&q, (12, -3), None, 0);
            s.scroll(7, 3);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x6CEC85632A719B0A, flaeche: s }
    }));
    v.push((60, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((15, 15), 0);
            s.fill((135u8, 117u8, 247u8, 255u8));
            rustgame_core::draw::line(&mut s, (47u8, 94u8, 78u8, 21u8), (8, -3), (3, 16), 1);
            let mut q = Surface::with_flags((7, 6), SRCALPHA);
            q = muster_auf(q, 7, 6);
            s.blit_ex(&q, (12, 15), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            s.fill_rect((76u8, 10u8, 112u8, 154u8), Rect::new(1, -1, 17, 8));
            s.fill_rect((131u8, 104u8, 100u8, 255u8), Rect::new(10, 12, 3, 16));
            s.scroll(0, -4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xD27E6BE05849AFCF, flaeche: s }
    }));
    v.push((61, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 11), 0);
            s.fill((164u8, 150u8, 164u8, 255u8));
            rustgame_core::draw::circle(&mut s, (33u8, 207u8, 236u8, 183u8), (6, 1), 10, 0);
            rustgame_core::draw::line(&mut s, (172u8, 13u8, 221u8, 89u8), (21, 0), (0, 8), 3);
            rustgame_core::draw::line(&mut s, (225u8, 163u8, 190u8, 174u8), (20, 1), (21, 4), 4);
            let mut q = Surface::with_flags((6, 4), SRCALPHA);
            q = muster_auf(q, 6, 4);
            q.set_alpha(Some(7));
            s.blit_ex(&q, (-2, 5), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            s.set_clip(Some(Rect::new(14, 5, 6, 1)));
            let mut q = Surface::with_flags((5, 11), SRCALPHA);
            q = muster_auf(q, 5, 11);
            q.set_alpha(Some(249));
            q.set_colorkey(Some((203u8, 99u8, 69u8)));
            s.blit_ex(&q, (-2, 8), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            s.scroll(10, 9);
            rustgame_core::draw::polygon(&mut s, (104u8, 191u8, 80u8, 188u8), &[(22, 12), (10, 10), (-2, -1)], 1);
            s = rustgame_core::transform::rotate(&s, -188.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x07A2E07D3484A6F1, flaeche: s }
    }));
    v.push((62, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((24, 11), SRCALPHA);
            s.fill((158u8, 182u8, 87u8, 237u8));
            {
                let mut teil = s.subsurface(Rect::new(0, 4, 21, 2));
                teil.fill((241u8, 241u8, 9u8, 15u8));
            }
            rustgame_core::draw::circle(&mut s, (82u8, 108u8, 40u8, 255u8), (16, 13), 14, 4);
            s.fill_rect((156u8, 175u8, 185u8, 255u8), Rect::new(17, -6, 9, 13));
            s.scroll(3, -4);
            rustgame_core::draw::circle(&mut s, (88u8, 15u8, 102u8, 255u8), (-4, 0), 7, 1);
            s.set_clip(Some(Rect::new(11, 7, 8, 3)));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x1CDFE0006AAE9C6C, flaeche: s }
    }));
    v.push((63, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((20, 20), SRCALPHA);
            s.fill((44u8, 197u8, 40u8, 247u8));
            rustgame_core::draw::rect(&mut s, (157u8, 30u8, 113u8, 255u8), Rect::new(-2, 11, 7, 8), 4);
            rustgame_core::draw::polygon(&mut s, (219u8, 220u8, 184u8, 255u8), &[(11, 18), (2, -1), (-3, -3), (6, 15)], 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x33F95478DB87434D, flaeche: s }
    }));
    v.push((64, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 9), 0);
            s.fill((206u8, 8u8, 104u8, 255u8));
            rustgame_core::draw::circle(&mut s, (75u8, 40u8, 213u8, 103u8), (10, 2), 0, 0);
            let mut q = Surface::with_flags((2, 3), SRCALPHA);
            q = muster_auf(q, 2, 3);
            s.blit_ex(&q, (4, 3), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            rustgame_core::draw::ellipse(&mut s, (109u8, 151u8, 70u8, 203u8), Rect::new(16, 6, 16, 4), 2);
            rustgame_core::draw::circle(&mut s, (0u8, 118u8, 238u8, 25u8), (1, 2), 7, 0);
            s = rustgame_core::transform::scale(&s, (9, 14));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x8E5C933C407CE123, flaeche: s }
    }));
    v.push((65, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((19, 15), SRCALPHA);
            s.fill((234u8, 226u8, 173u8, 227u8));
            rustgame_core::draw::rect_radius(&mut s, (239u8, 135u8, 100u8, 36u8), Rect::new(10, 7, 19, 1), 2, 0);
            s.set_clip(Some(Rect::new(4, 12, 11, 1)));
            s.set_clip(Some(Rect::new(11, 5, 6, 10)));
            {
                let mut teil = s.subsurface(Rect::new(7, 6, 8, 8));
                teil.fill((78u8, 10u8, 171u8, 255u8));
            }
            {
                let mut teil = s.subsurface(Rect::new(10, 12, 5, 1));
                teil.fill((220u8, 37u8, 133u8, 255u8));
            }
            rustgame_core::draw::polygon(&mut s, (82u8, 221u8, 220u8, 255u8), &[(0, 12), (20, 9), (19, 6)], 1);
            {
                let mut teil = s.subsurface(Rect::new(17, 5, 1, 6));
                teil.fill((155u8, 33u8, 137u8, 2u8));
            }
            s.scroll(-9, 12);
            s = rustgame_core::transform::grayscale(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x5C66977E7ED1D82A, flaeche: s }
    }));
    v.push((66, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 15), SRCALPHA);
            s.fill((228u8, 150u8, 131u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (253u8, 94u8, 239u8, 255u8), Rect::new(4, -4, 11, 2), 2);
            rustgame_core::draw::circle(&mut s, (15u8, 57u8, 196u8, 234u8), (0, 7), 0, 0);
            rustgame_core::draw::circle(&mut s, (163u8, 247u8, 156u8, 105u8), (2, 5), 1, 1);
            rustgame_core::draw::circle(&mut s, (180u8, 207u8, 39u8, 255u8), (5, -2), 6, 2);
            s.fill_rect((242u8, 212u8, 28u8, 255u8), Rect::new(6, 4, 8, 3));
            rustgame_core::draw::circle(&mut s, (244u8, 190u8, 187u8, 161u8), (2, 1), 0, 2);
            rustgame_core::draw::ellipse(&mut s, (242u8, 164u8, 234u8, 144u8), Rect::new(1, -7, 9, 16), 1);
            rustgame_core::draw::circle(&mut s, (53u8, 175u8, 30u8, 255u8), (7, -1), 0, 3);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x1315BF838F17EAFE, flaeche: s }
    }));
    v.push((67, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 9), 0);
            s.fill((239u8, 212u8, 136u8, 255u8));
            let mut q = Surface::with_flags((6, 5), 0);
            q = muster_auf(q, 6, 5);
            q.set_colorkey(Some((80u8, 225u8, 92u8)));
            s.blit_ex(&q, (-1, -1), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            let mut q = Surface::with_flags((2, 8), 0);
            q = muster_auf(q, 2, 8);
            s.blit_ex(&q, (1, 0), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            s.fill_rect((199u8, 20u8, 183u8, 119u8), Rect::new(-2, 8, 8, 12));
            rustgame_core::draw::line(&mut s, (178u8, 120u8, 13u8, 226u8), (0, -4), (1, 3), 4);
            s.fill_rect((255u8, 68u8, 97u8, 132u8), Rect::new(4, -5, 11, 10));
            rustgame_core::draw::polygon(&mut s, (61u8, 196u8, 32u8, 255u8), &[(5, 10), (8, 3), (-4, 0), (10, 11), (3, 6), (-1, 11)], 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xA8E693FE935E1DB3, flaeche: s }
    }));
    v.push((68, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((29, 20), 0);
            s.fill((56u8, 124u8, 221u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(27, 10, 2, 9));
                teil.fill((63u8, 21u8, 246u8, 255u8));
            }
            s.set_clip(Some(Rect::new(10, 1, 3, 6)));
            {
                let mut teil = s.subsurface(Rect::new(6, 0, 17, 14));
                teil.fill((65u8, 12u8, 75u8, 192u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (63u8, 5u8, 244u8, 255u8), Rect::new(9, 8, 5, 6), 3, 5);
            let mut q = Surface::with_flags((5, 10), SRCALPHA);
            q = muster_auf(q, 5, 10);
            q.set_colorkey(Some((210u8, 197u8, 80u8)));
            s.blit_ex(&q, (11, -2), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x6AB4E2799A6721C9, flaeche: s }
    }));
    v.push((69, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((27, 7), SRCALPHA);
            s.fill((176u8, 167u8, 212u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (225u8, 72u8, 211u8, 164u8), Rect::new(11, -3, 13, 4), 1);
            {
                let mut teil = s.subsurface(Rect::new(9, 1, 17, 2));
                teil.fill((198u8, 33u8, 134u8, 255u8));
            }
            rustgame_core::draw::line(&mut s, (20u8, 32u8, 48u8, 190u8), (9, -4), (5, -1), 2);
            {
                let mut teil = s.subsurface(Rect::new(15, 5, 2, 2));
                teil.fill((17u8, 130u8, 43u8, 213u8));
            }
            rustgame_core::draw::circle(&mut s, (139u8, 78u8, 250u8, 255u8), (26, -1), 8, 4);
            s = s.convert_alpha();
            s = rustgame_core::transform::grayscale(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x2347892111A7BB13, flaeche: s }
    }));
    v.push((70, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((9, 15), 0);
            s.fill((230u8, 69u8, 164u8, 255u8));
            s.set_clip(Some(Rect::new(3, 11, 6, 4)));
            rustgame_core::draw::ellipse(&mut s, (37u8, 163u8, 141u8, 255u8), Rect::new(7, 16, 7, 3), 0);
            s = s.convert_alpha();
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x60938D2783290CA5, flaeche: s }
    }));
    v.push((71, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((16, 22), 0);
            s.fill((4u8, 135u8, 79u8, 255u8));
            rustgame_core::draw::circle(&mut s, (44u8, 102u8, 28u8, 255u8), (15, 2), 16, 0);
            rustgame_core::draw::ellipse(&mut s, (66u8, 178u8, 85u8, 234u8), Rect::new(-7, 11, 19, 0), 3);
            rustgame_core::draw::rect_radius(&mut s, (76u8, 164u8, 223u8, 255u8), Rect::new(6, -5, 0, 7), 4, 1);
            s.set_clip(Some(Rect::new(11, 9, 5, 1)));
            {
                let mut teil = s.subsurface(Rect::new(6, 12, 1, 2));
                teil.fill((50u8, 151u8, 71u8, 255u8));
            }
            let mut q = Surface::with_flags((17, 20), 0);
            q = muster_auf(q, 17, 20);
            q.set_alpha(Some(48));
            q.set_colorkey(Some((110u8, 82u8, 225u8)));
            s.blit_ex(&q, (11, 13), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            rustgame_core::draw::circle(&mut s, (12u8, 17u8, 96u8, 132u8), (3, 11), 10, 4);
            rustgame_core::draw::ellipse(&mut s, (138u8, 72u8, 27u8, 124u8), Rect::new(7, 23, 14, 7), 1);
            s = rustgame_core::transform::rotate(&s, -136.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x44E3FD070321D936, flaeche: s }
    }));
    v.push((72, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 31), 0);
            s.fill((95u8, 176u8, 191u8, 255u8));
            rustgame_core::draw::circle(&mut s, (175u8, 119u8, 194u8, 66u8), (0, -3), 22, 1);
            let mut q = Surface::with_flags((2, 30), SRCALPHA);
            q = muster_auf(q, 2, 30);
            q.set_alpha(Some(162));
            s.blit_ex(&q, (9, 12), Some(Rect::new(1, -1, 2, 31)), rustgame_core::blend::flags::BLEND_RGBA_MAX);
            {
                let mut teil = s.subsurface(Rect::new(7, 22, 1, 5));
                teil.fill((78u8, 232u8, 110u8, 171u8));
            }
            s.fill_rect((31u8, 193u8, 37u8, 255u8), Rect::new(9, -7, 4, 3));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x6DEEB2416529E015, flaeche: s }
    }));
    v.push((73, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((14, 9), 0);
            s.fill((93u8, 236u8, 152u8, 255u8));
            s.fill_rect((230u8, 56u8, 11u8, 103u8), Rect::new(1, 8, 0, 6));
            let mut q = Surface::with_flags((8, 9), 0);
            q = muster_auf(q, 8, 9);
            s.blit_ex(&q, (4, 2), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            let mut q = Surface::with_flags((14, 7), 0);
            q = muster_auf(q, 14, 7);
            q.set_colorkey(Some((111u8, 138u8, 140u8)));
            s.blit_ex(&q, (-3, 3), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            let mut q = Surface::with_flags((3, 3), 0);
            q = muster_auf(q, 3, 3);
            q.set_alpha(Some(72));
            s.blit_ex(&q, (11, 3), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            {
                let mut teil = s.subsurface(Rect::new(6, 1, 7, 7));
                teil.fill((185u8, 210u8, 85u8, 45u8));
            }
            let mut q = Surface::with_flags((9, 3), SRCALPHA);
            q = muster_auf(q, 9, 3);
            s.blit_ex(&q, (13, 5), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x09516823B72604AD, flaeche: s }
    }));
    v.push((74, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((25, 22), 0);
            s.fill((163u8, 61u8, 158u8, 255u8));
            rustgame_core::draw::line(&mut s, (61u8, 226u8, 233u8, 179u8), (13, 10), (6, 6), 3);
            let mut q = Surface::with_flags((2, 24), 0);
            q = muster_auf(q, 2, 24);
            q.set_alpha(Some(116));
            s.blit_ex(&q, (22, 3), Some(Rect::new(2, -12, 4, 6)), rustgame_core::blend::flags::BLEND_RGBA_MAX);
            rustgame_core::draw::line(&mut s, (170u8, 21u8, 10u8, 111u8), (-3, 18), (2, 8), 5);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x090CA6EF04DA4B8C, flaeche: s }
    }));
    v.push((75, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((20, 24), SRCALPHA);
            s.fill((242u8, 176u8, 51u8, 255u8));
            s.scroll(-15, 24);
            {
                let mut teil = s.subsurface(Rect::new(15, 23, 3, 1));
                teil.fill((14u8, 193u8, 236u8, 255u8));
            }
            rustgame_core::draw::line(&mut s, (100u8, 73u8, 154u8, 255u8), (11, 6), (22, 25), 1);
            s.scroll(19, 21);
            s.fill_rect((165u8, 99u8, 29u8, 255u8), Rect::new(-5, 14, 3, 25));
            s.scroll(13, 19);
            s.set_clip(Some(Rect::new(14, 9, 3, 10)));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x035EDF0F388FBADD, flaeche: s }
    }));
    v.push((76, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((17, 20), 0);
            s.fill((102u8, 152u8, 27u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (224u8, 250u8, 118u8, 255u8), Rect::new(-7, 11, 9, 7), 2);
            rustgame_core::draw::ellipse(&mut s, (211u8, 196u8, 121u8, 255u8), Rect::new(1, 19, 0, 2), 3);
            s.set_clip(Some(Rect::new(7, 2, 5, 4)));
            rustgame_core::draw::line(&mut s, (10u8, 107u8, 255u8, 7u8), (11, 1), (15, 15), 4);
            let mut q = Surface::with_flags((15, 7), 0);
            q = muster_auf(q, 15, 7);
            s.blit_ex(&q, (2, 20), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xF9A95DE752EA4ED4, flaeche: s }
    }));
    v.push((77, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((31, 14), SRCALPHA);
            s.fill((58u8, 149u8, 243u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (1u8, 143u8, 14u8, 255u8), &[(27, 1), (8, 12), (20, 1), (17, 6)], 0);
            s.fill_rect((104u8, 115u8, 98u8, 240u8), Rect::new(-12, -5, 12, 17));
            rustgame_core::draw::line(&mut s, (51u8, 108u8, 168u8, 39u8), (1, -1), (32, 10), 1);
            rustgame_core::draw::line(&mut s, (139u8, 227u8, 62u8, 54u8), (18, 12), (-1, 8), 5);
            rustgame_core::draw::polygon(&mut s, (241u8, 206u8, 70u8, 255u8), &[(20, -4), (-2, -3), (27, 16), (-2, 7)], 3);
            s.set_clip(Some(Rect::new(16, 9, 12, 4)));
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x0971F827B08124B0, flaeche: s }
    }));
    v.push((78, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((32, 12), SRCALPHA);
            s.fill((218u8, 17u8, 75u8, 131u8));
            let mut q = Surface::with_flags((14, 13), SRCALPHA);
            q = muster_auf(q, 14, 13);
            q.set_colorkey(Some((120u8, 89u8, 177u8)));
            s.blit_ex(&q, (23, 6), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            rustgame_core::draw::circle(&mut s, (184u8, 186u8, 27u8, 255u8), (9, -1), 12, 1);
            let mut q = Surface::with_flags((11, 11), 0);
            q = muster_auf(q, 11, 11);
            s.blit_ex(&q, (6, 12), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            rustgame_core::draw::line(&mut s, (0u8, 56u8, 234u8, 42u8), (19, 3), (23, -2), 4);
            let mut q = Surface::with_flags((10, 2), 0);
            q = muster_auf(q, 10, 2);
            q.set_alpha(Some(155));
            q.set_colorkey(Some((29u8, 105u8, 203u8)));
            s.blit_ex(&q, (6, -4), Some(Rect::new(-2, 2, 11, 3)), rustgame_core::blend::flags::BLEND_RGB_SUB);
            s.fill_rect((238u8, 201u8, 220u8, 255u8), Rect::new(-8, -4, 8, 6));
            rustgame_core::draw::ellipse(&mut s, (56u8, 45u8, 108u8, 247u8), Rect::new(21, -6, 19, 10), 1);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xCDBEEAE89F2FF599, flaeche: s }
    }));
    v.push((79, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((10, 20), SRCALPHA);
            s.fill((125u8, 253u8, 79u8, 255u8));
            let mut q = Surface::with_flags((5, 5), 0);
            q = muster_auf(q, 5, 5);
            s.blit_ex(&q, (9, 11), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            rustgame_core::draw::rect_radius(&mut s, (54u8, 132u8, 248u8, 255u8), Rect::new(-4, 11, 11, 23), 3, 0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x3425F4CE712B1D18, flaeche: s }
    }));
    v.push((80, true, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((14, 18), 0);
            s.fill((215u8, 188u8, 186u8, 255u8));
            let mut q = Surface::with_flags((2, 19), 0);
            q = muster_auf(q, 2, 19);
            s.blit_ex(&q, (3, 15), Some(Rect::new(0, -5, 4, 18)), 0);
            s.set_clip(Some(Rect::new(9, 4, 2, 4)));
            let mut q = Surface::with_flags((11, 7), 0);
            q = muster_auf(q, 11, 7);
            q.set_colorkey(Some((40u8, 163u8, 51u8)));
            s.blit_ex(&q, (17, -4), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            rustgame_core::draw::rect_radius(&mut s, (188u8, 12u8, 161u8, 255u8), Rect::new(8, 14, 3, 7), 3, 4);
            let mut q = Surface::with_flags((2, 16), SRCALPHA);
            q = muster_auf(q, 2, 16);
            q.set_alpha(Some(246));
            s.blit_ex(&q, (7, 18), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            rustgame_core::draw::line(&mut s, (63u8, 55u8, 5u8, 255u8), (1, -3), (15, 4), 3);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xBAE2FED0F2A27F25, flaeche: s }
    }));
    v.push((81, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((22, 32), SRCALPHA);
            s.fill((245u8, 8u8, 81u8, 196u8));
            let mut q = Surface::with_flags((21, 2), 0);
            q = muster_auf(q, 21, 2);
            q.set_alpha(Some(183));
            q.set_colorkey(Some((67u8, 38u8, 108u8)));
            s.blit_ex(&q, (12, 15), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            rustgame_core::draw::polygon(&mut s, (119u8, 229u8, 125u8, 255u8), &[(18, 24), (-4, 31), (25, 23), (7, 14), (7, 13)], 2);
            rustgame_core::draw::circle(&mut s, (31u8, 218u8, 157u8, 255u8), (23, 13), 11, 1);
            s = rustgame_core::transform::scale(&s, (5, 13));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xD672E908C6D28A97, flaeche: s }
    }));
    v.push((82, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((10, 30), SRCALPHA);
            s.fill((85u8, 133u8, 80u8, 91u8));
            rustgame_core::draw::line(&mut s, (101u8, 157u8, 69u8, 50u8), (-1, 26), (-2, 16), 3);
            rustgame_core::draw::circle(&mut s, (92u8, 0u8, 123u8, 255u8), (7, 15), 0, 4);
            let mut q = Surface::with_flags((9, 15), SRCALPHA);
            q = muster_auf(q, 9, 15);
            s.blit_ex(&q, (4, -3), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            rustgame_core::draw::polygon(&mut s, (109u8, 2u8, 182u8, 255u8), &[(-1, 0), (6, -3), (8, 22), (-4, 15)], 2);
            rustgame_core::draw::line(&mut s, (235u8, 218u8, 12u8, 47u8), (-1, 29), (9, 19), 5);
            let mut q = Surface::with_flags((7, 4), 0);
            q = muster_auf(q, 7, 4);
            s.blit_ex(&q, (7, -2), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            rustgame_core::draw::rect_radius(&mut s, (42u8, 23u8, 199u8, 124u8), Rect::new(9, 29, 5, 12), 1, 5);
            rustgame_core::draw::circle(&mut s, (142u8, 82u8, 100u8, 177u8), (4, 25), 22, 1);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xDC24A9115CDE7ECA, flaeche: s }
    }));
    v.push((83, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 20), SRCALPHA);
            s.fill((25u8, 108u8, 96u8, 255u8));
            s.scroll(-18, 6);
            s.set_clip(Some(Rect::new(9, 17, 4, 2)));
            rustgame_core::draw::ellipse(&mut s, (228u8, 85u8, 22u8, 117u8), Rect::new(-10, -9, 24, 14), 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x37D29B12848B1F25, flaeche: s }
    }));
    v.push((84, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((29, 15), 0);
            s.fill((19u8, 250u8, 2u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (255u8, 187u8, 76u8, 235u8), &[(30, 18), (12, 12), (25, 8), (17, 15), (10, 0)], 3);
            s.fill_rect((23u8, 255u8, 149u8, 76u8), Rect::new(19, -5, 0, 7));
            {
                let mut teil = s.subsurface(Rect::new(13, 2, 10, 12));
                teil.fill((61u8, 52u8, 37u8, 255u8));
            }
            let mut q = Surface::with_flags((2, 6), 0);
            q = muster_auf(q, 2, 6);
            s.blit_ex(&q, (26, -4), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            rustgame_core::draw::rect_radius(&mut s, (80u8, 135u8, 94u8, 97u8), Rect::new(5, 3, 19, 4), 3, 5);
            s.set_clip(Some(Rect::new(20, 13, 1, 1)));
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x7490810BDE834D96, flaeche: s }
    }));
    v.push((85, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 28), 0);
            s.fill((54u8, 164u8, 98u8, 255u8));
            let mut q = Surface::with_flags((14, 29), 0);
            q = muster_auf(q, 14, 29);
            s.blit_ex(&q, (13, 30), Some(Rect::new(14, 22, 14, 27)), rustgame_core::blend::flags::BLEND_RGB_ADD);
            s.fill_rect((240u8, 231u8, 246u8, 220u8), Rect::new(1, 27, 7, 8));
            let mut q = Surface::with_flags((3, 13), 0);
            q = muster_auf(q, 3, 13);
            q.set_alpha(Some(73));
            s.blit_ex(&q, (13, 10), Some(Rect::new(4, 7, 5, 10)), rustgame_core::blend::flags::BLEND_RGB_SUB);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xD5217CC84F165AF2, flaeche: s }
    }));
    v.push((86, true, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((31, 30), SRCALPHA);
            s.fill((6u8, 35u8, 155u8, 162u8));
            rustgame_core::draw::circle(&mut s, (49u8, 209u8, 235u8, 255u8), (-4, 33), 24, 1);
            {
                let mut teil = s.subsurface(Rect::new(1, 25, 29, 2));
                teil.fill((43u8, 242u8, 201u8, 255u8));
            }
            s.fill_rect((252u8, 190u8, 219u8, 70u8), Rect::new(11, 9, 12, 13));
            rustgame_core::draw::circle(&mut s, (203u8, 131u8, 162u8, 255u8), (29, 19), 11, 3);
            rustgame_core::draw::rect_radius(&mut s, (84u8, 252u8, 231u8, 130u8), Rect::new(10, -12, 6, 0), 1, 0);
            rustgame_core::draw::ellipse(&mut s, (6u8, 27u8, 18u8, 114u8), Rect::new(5, 14, 15, 28), 0);
            let mut q = Surface::with_flags((13, 12), 0);
            q = muster_auf(q, 13, 12);
            s.blit_ex(&q, (10, 29), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            rustgame_core::draw::circle(&mut s, (175u8, 19u8, 154u8, 93u8), (33, 1), 10, 2);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xF4086D0A11D0F8C9, flaeche: s }
    }));
    v.push((87, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((10, 29), SRCALPHA);
            s.fill((143u8, 171u8, 167u8, 52u8));
            rustgame_core::draw::line(&mut s, (141u8, 197u8, 45u8, 7u8), (11, 28), (-2, 8), 3);
            rustgame_core::draw::polygon(&mut s, (142u8, 214u8, 221u8, 255u8), &[(6, 24), (2, 18), (2, 26)], 2);
            s.fill_rect((146u8, 175u8, 8u8, 255u8), Rect::new(3, 15, 9, 3));
            s.fill_rect((31u8, 157u8, 248u8, 59u8), Rect::new(1, 25, 6, 17));
            s.fill_rect((195u8, 146u8, 154u8, 65u8), Rect::new(7, -1, 9, 14));
            rustgame_core::draw::ellipse(&mut s, (170u8, 98u8, 57u8, 255u8), Rect::new(-4, 1, 4, 23), 1);
            {
                let mut teil = s.subsurface(Rect::new(6, 25, 3, 2));
                teil.fill((168u8, 19u8, 106u8, 127u8));
            }
            s = rustgame_core::transform::scale(&s, (11, 2));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x0A7BBA293DD37C65, flaeche: s }
    }));
    v.push((88, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((18, 12), SRCALPHA);
            s.fill((7u8, 4u8, 174u8, 255u8));
            let mut q = Surface::with_flags((9, 3), 0);
            q = muster_auf(q, 9, 3);
            s.blit_ex(&q, (6, 9), Some(Rect::new(0, -2, 7, 6)), rustgame_core::blend::flags::BLEND_RGB_MAX);
            s.fill_rect((226u8, 101u8, 112u8, 126u8), Rect::new(-4, -6, 7, 14));
            rustgame_core::draw::ellipse(&mut s, (254u8, 193u8, 197u8, 255u8), Rect::new(6, 10, 2, 15), 1);
            let mut q = Surface::with_flags((6, 4), SRCALPHA);
            q = muster_auf(q, 6, 4);
            q.set_alpha(Some(10));
            s.blit_ex(&q, (-1, 13), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            s.scroll(-5, 1);
            let mut q = Surface::with_flags((2, 9), 0);
            q = muster_auf(q, 2, 9);
            s.blit_ex(&q, (15, 8), Some(Rect::new(1, 8, 4, 12)), rustgame_core::blend::flags::BLEND_RGBA_MIN);
            rustgame_core::draw::ellipse(&mut s, (153u8, 222u8, 114u8, 22u8), Rect::new(4, 5, 1, 1), 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x1D580450740F3AC5, flaeche: s }
    }));
    v.push((89, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 30), 0);
            s.fill((132u8, 71u8, 176u8, 255u8));
            s.fill_rect((107u8, 153u8, 171u8, 171u8), Rect::new(-3, 21, 2, 29));
            rustgame_core::draw::rect_radius(&mut s, (236u8, 27u8, 165u8, 255u8), Rect::new(7, -10, 1, 26), 3, 0);
            let mut q = Surface::with_flags((6, 25), SRCALPHA);
            q = muster_auf(q, 6, 25);
            q.set_alpha(Some(10));
            s.blit_ex(&q, (0, 28), Some(Rect::new(-2, -4, 5, 0)), rustgame_core::blend::flags::BLEND_RGBA_MAX);
            let mut q = Surface::with_flags((10, 7), 0);
            q = muster_auf(q, 10, 7);
            s.blit_ex(&q, (11, 21), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            let mut q = Surface::with_flags((10, 31), 0);
            q = muster_auf(q, 10, 31);
            q.set_colorkey(Some((159u8, 122u8, 182u8)));
            s.blit_ex(&q, (9, 12), Some(Rect::new(11, 3, 10, 15)), rustgame_core::blend::flags::BLEND_RGB_MAX);
            rustgame_core::draw::rect_radius(&mut s, (68u8, 158u8, 154u8, 255u8), Rect::new(5, 22, 4, 4), 4, 5);
            rustgame_core::draw::polygon(&mut s, (24u8, 64u8, 121u8, 224u8), &[(5, 25), (-4, 29), (-3, 24), (2, 12), (8, 17), (-3, 12)], 0);
            let mut q = Surface::with_flags((3, 30), 0);
            q = muster_auf(q, 3, 30);
            q.set_colorkey(Some((70u8, 78u8, 140u8)));
            s.blit_ex(&q, (-4, 23), Some(Rect::new(-1, -7, 4, 3)), 0);
            s = rustgame_core::transform::flip(&s, true, false);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x8B6C4474885F4B39, flaeche: s }
    }));
    v.push((90, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 28), SRCALPHA);
            s.fill((181u8, 231u8, 157u8, 158u8));
            let mut q = Surface::with_flags((7, 4), 0);
            q = muster_auf(q, 7, 4);
            q.set_alpha(Some(195));
            s.blit_ex(&q, (-1, 4), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            {
                let mut teil = s.subsurface(Rect::new(10, 2, 1, 4));
                teil.fill((63u8, 52u8, 217u8, 108u8));
            }
            s.fill_rect((124u8, 82u8, 14u8, 255u8), Rect::new(11, 13, 2, 30));
            s.scroll(-2, 8);
            s = rustgame_core::transform::grayscale(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x20485805FD4E13A9, flaeche: s }
    }));
    v.push((91, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 24), SRCALPHA);
            s.fill((205u8, 234u8, 225u8, 82u8));
            let mut q = Surface::with_flags((6, 9), SRCALPHA);
            q = muster_auf(q, 6, 9);
            s.blit_ex(&q, (2, 9), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            rustgame_core::draw::circle(&mut s, (229u8, 154u8, 197u8, 255u8), (7, 20), 23, 4);
            let mut q = Surface::with_flags((7, 7), SRCALPHA);
            q = muster_auf(q, 7, 7);
            q.set_alpha(Some(109));
            s.blit_ex(&q, (-3, -1), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            {
                let mut teil = s.subsurface(Rect::new(3, 2, 5, 14));
                teil.fill((180u8, 78u8, 227u8, 145u8));
            }
            rustgame_core::draw::circle(&mut s, (103u8, 176u8, 227u8, 255u8), (7, -1), 19, 3);
            s = s.convert_alpha();
            s = rustgame_core::transform::scale(&s, (11, 19));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x051E49C6E544D120, flaeche: s }
    }));
    v.push((92, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((19, 22), 0);
            s.fill((136u8, 48u8, 241u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (31u8, 22u8, 13u8, 173u8), Rect::new(10, -1, 18, 5), 4);
            rustgame_core::draw::rect_radius(&mut s, (84u8, 60u8, 79u8, 247u8), Rect::new(15, 18, 2, 13), 2, 5);
            s.fill_rect((112u8, 91u8, 116u8, 91u8), Rect::new(17, -2, 15, 25));
            s = s.convert_alpha();
            s = rustgame_core::transform::grayscale(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x5A2199C9EB9BA8A5, flaeche: s }
    }));
    v.push((93, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 24), 0);
            s.fill((58u8, 47u8, 181u8, 255u8));
            s.set_clip(Some(Rect::new(3, 5, 2, 18)));
            rustgame_core::draw::line(&mut s, (204u8, 63u8, 36u8, 188u8), (20, 9), (20, 10), 4);
            s.set_clip(Some(Rect::new(9, 19, 5, 3)));
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x4AE8C0B305315185, flaeche: s }
    }));
    v.push((94, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((23, 11), SRCALPHA);
            s.fill((220u8, 8u8, 255u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (10u8, 222u8, 6u8, 140u8), &[(19, 9), (7, 10), (7, 2), (17, -3), (9, 10), (22, 1)], 2);
            s.set_clip(Some(Rect::new(21, 4, 2, 6)));
            rustgame_core::draw::polygon(&mut s, (178u8, 20u8, 9u8, 220u8), &[(4, 3), (-3, 10), (6, 3)], 2);
            s.set_clip(Some(Rect::new(5, 10, 16, 1)));
            s = rustgame_core::transform::grayscale(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x5DCCEA69D70848A4, flaeche: s }
    }));
    v.push((95, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((30, 23), 0);
            s.fill((65u8, 252u8, 14u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (99u8, 22u8, 99u8, 141u8), Rect::new(25, -6, 23, 8), 3);
            let mut q = Surface::with_flags((7, 7), SRCALPHA);
            q = muster_auf(q, 7, 7);
            q.set_alpha(Some(91));
            s.blit_ex(&q, (12, 12), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            rustgame_core::draw::circle(&mut s, (156u8, 7u8, 12u8, 240u8), (19, 18), 16, 4);
            s = rustgame_core::transform::flip(&s, true, false);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xA058970E93562B9E, flaeche: s }
    }));
    v.push((96, true, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((17, 16), 0);
            s.fill((202u8, 105u8, 49u8, 255u8));
            let mut q = Surface::with_flags((3, 2), SRCALPHA);
            q = muster_auf(q, 3, 2);
            q.set_colorkey(Some((173u8, 182u8, 141u8)));
            s.blit_ex(&q, (0, 12), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            let mut q = Surface::with_flags((19, 17), SRCALPHA);
            q = muster_auf(q, 19, 17);
            q.set_alpha(Some(177));
            s.blit_ex(&q, (12, 6), Some(Rect::new(-9, -6, 14, 16)), rustgame_core::blend::flags::BLEND_RGB_MULT);
            let mut q = Surface::with_flags((6, 5), 0);
            q = muster_auf(q, 6, 5);
            s.blit_ex(&q, (14, 16), Some(Rect::new(7, 0, 6, 6)), rustgame_core::blend::flags::BLEND_RGBA_MIN);
            s.fill_rect((217u8, 208u8, 47u8, 182u8), Rect::new(14, 11, 18, 6));
            rustgame_core::draw::circle(&mut s, (113u8, 20u8, 157u8, 160u8), (0, 3), 5, 2);
            let mut q = Surface::with_flags((17, 6), SRCALPHA);
            q = muster_auf(q, 17, 6);
            q.set_colorkey(Some((234u8, 143u8, 3u8)));
            s.blit_ex(&q, (20, 1), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            s.fill_rect((36u8, 137u8, 110u8, 255u8), Rect::new(-5, -3, 0, 3));
            {
                let mut teil = s.subsurface(Rect::new(9, 2, 6, 5));
                teil.fill((117u8, 80u8, 227u8, 43u8));
            }
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x6D54D99EF36060CE, flaeche: s }
    }));
    v.push((97, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 19), 0);
            s.fill((190u8, 26u8, 11u8, 255u8));
            rustgame_core::draw::circle(&mut s, (139u8, 188u8, 13u8, 255u8), (-2, 16), 14, 3);
            rustgame_core::draw::circle(&mut s, (156u8, 93u8, 129u8, 124u8), (9, 13), 6, 1);
            rustgame_core::draw::line(&mut s, (65u8, 33u8, 71u8, 255u8), (-1, -2), (13, 13), 3);
            rustgame_core::draw::line(&mut s, (231u8, 39u8, 122u8, 31u8), (1, -4), (14, 0), 5);
            rustgame_core::draw::polygon(&mut s, (124u8, 6u8, 72u8, 255u8), &[(-4, 0), (11, -4), (14, 19), (12, 11), (-4, 19)], 0);
            rustgame_core::draw::rect_radius(&mut s, (49u8, 22u8, 5u8, 220u8), Rect::new(-5, 7, 7, 17), 3, 5);
            s = rustgame_core::transform::rotate(&s, -112.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x5AF6D939A3A777E3, flaeche: s }
    }));
    v.push((98, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((17, 24), 0);
            s.fill((180u8, 214u8, 30u8, 255u8));
            s.scroll(-13, 7);
            rustgame_core::draw::line(&mut s, (184u8, 2u8, 3u8, 255u8), (20, 20), (20, 15), 3);
            s.fill_rect((98u8, 87u8, 206u8, 248u8), Rect::new(-4, 24, 16, 26));
            {
                let mut teil = s.subsurface(Rect::new(16, 4, 1, 18));
                teil.fill((210u8, 52u8, 81u8, 255u8));
            }
            rustgame_core::draw::ellipse(&mut s, (160u8, 252u8, 91u8, 255u8), Rect::new(13, 6, 16, 2), 1);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x75C94282B18314AD, flaeche: s }
    }));
    v.push((99, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((18, 18), SRCALPHA);
            s.fill((127u8, 68u8, 44u8, 255u8));
            rustgame_core::draw::circle(&mut s, (250u8, 102u8, 215u8, 45u8), (19, 15), 6, 2);
            rustgame_core::draw::ellipse(&mut s, (77u8, 236u8, 91u8, 108u8), Rect::new(18, 9, 10, 2), 4);
            rustgame_core::draw::circle(&mut s, (161u8, 206u8, 21u8, 255u8), (4, 13), 0, 3);
            {
                let mut teil = s.subsurface(Rect::new(17, 2, 1, 7));
                teil.fill((183u8, 66u8, 110u8, 26u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xD51DF02F5A53C2C4, flaeche: s }
    }));
    v.push((100, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((10, 20), SRCALPHA);
            s.fill((179u8, 221u8, 56u8, 201u8));
            s.set_clip(Some(Rect::new(7, 8, 1, 11)));
            rustgame_core::draw::polygon(&mut s, (158u8, 104u8, 91u8, 255u8), &[(0, 2), (7, 7), (9, 22), (2, 8)], 3);
            rustgame_core::draw::rect_radius(&mut s, (82u8, 63u8, 92u8, 192u8), Rect::new(-5, 15, 2, 18), 1, 2);
            s.set_clip(Some(Rect::new(0, 7, 8, 13)));
            rustgame_core::draw::rect_radius(&mut s, (26u8, 126u8, 148u8, 64u8), Rect::new(5, 11, 8, 11), 3, 5);
            rustgame_core::draw::circle(&mut s, (222u8, 129u8, 122u8, 153u8), (9, 20), 16, 3);
            s = rustgame_core::transform::rotate(&s, -345.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xD25370660352A7D2, flaeche: s }
    }));
    v.push((101, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((24, 12), 0);
            s.fill((183u8, 239u8, 24u8, 255u8));
            let mut q = Surface::with_flags((21, 5), 0);
            q = muster_auf(q, 21, 5);
            s.blit_ex(&q, (17, 10), Some(Rect::new(1, -1, 2, 7)), rustgame_core::blend::flags::BLEND_RGBA_MULT);
            let mut q = Surface::with_flags((17, 14), SRCALPHA);
            q = muster_auf(q, 17, 14);
            s.blit_ex(&q, (9, -2), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            {
                let mut teil = s.subsurface(Rect::new(20, 9, 3, 3));
                teil.fill((202u8, 194u8, 117u8, 136u8));
            }
            let mut q = Surface::with_flags((14, 13), 0);
            q = muster_auf(q, 14, 13);
            q.set_alpha(Some(237));
            q.set_colorkey(Some((48u8, 176u8, 11u8)));
            s.blit_ex(&q, (19, 7), Some(Rect::new(11, -3, 1, 13)), rustgame_core::blend::flags::BLEND_RGB_SUB);
            {
                let mut teil = s.subsurface(Rect::new(4, 6, 12, 6));
                teil.fill((255u8, 159u8, 92u8, 85u8));
            }
            rustgame_core::draw::circle(&mut s, (43u8, 33u8, 16u8, 3u8), (18, 13), 22, 3);
            rustgame_core::draw::ellipse(&mut s, (235u8, 71u8, 91u8, 255u8), Rect::new(0, 11, 14, 4), 0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x62842D843BF6B497, flaeche: s }
    }));
    v.push((102, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((10, 27), 0);
            s.fill((86u8, 93u8, 196u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (94u8, 193u8, 226u8, 22u8), Rect::new(2, 18, 2, 2), 0, 1);
            rustgame_core::draw::rect_radius(&mut s, (74u8, 251u8, 210u8, 113u8), Rect::new(10, 28, 1, 7), 4, 0);
            rustgame_core::draw::ellipse(&mut s, (99u8, 218u8, 220u8, 255u8), Rect::new(7, -6, 12, 5), 2);
            {
                let mut teil = s.subsurface(Rect::new(9, 7, 1, 14));
                teil.fill((225u8, 136u8, 161u8, 255u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x1E875339D75E69E1, flaeche: s }
    }));
    v.push((103, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((28, 20), 0);
            s.fill((110u8, 39u8, 62u8, 255u8));
            let mut q = Surface::with_flags((30, 12), 0);
            q = muster_auf(q, 30, 12);
            q.set_alpha(Some(42));
            q.set_colorkey(Some((12u8, 22u8, 115u8)));
            s.blit_ex(&q, (2, 18), Some(Rect::new(18, 3, 11, 13)), rustgame_core::blend::flags::BLEND_RGBA_MIN);
            rustgame_core::draw::circle(&mut s, (44u8, 188u8, 103u8, 127u8), (9, 18), 19, 4);
            let mut q = Surface::with_flags((10, 11), 0);
            q = muster_auf(q, 10, 11);
            s.blit_ex(&q, (25, 12), Some(Rect::new(11, 3, 0, 7)), rustgame_core::blend::flags::BLEND_RGB_SUB);
            s.set_clip(Some(Rect::new(16, 8, 2, 8)));
            rustgame_core::draw::line(&mut s, (46u8, 156u8, 248u8, 255u8), (3, 14), (2, 18), 4);
            s.scroll(7, 10);
            rustgame_core::draw::rect(&mut s, (227u8, 134u8, 33u8, 255u8), Rect::new(-3, 18, 2, 10), 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xD0E80ED5AE4509D1, flaeche: s }
    }));
    v.push((104, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((6, 12), 0);
            s.fill((116u8, 200u8, 169u8, 255u8));
            rustgame_core::draw::circle(&mut s, (85u8, 53u8, 75u8, 8u8), (-3, -2), 10, 2);
            let mut q = Surface::with_flags((2, 5), 0);
            q = muster_auf(q, 2, 5);
            q.set_alpha(Some(165));
            s.blit_ex(&q, (-2, 5), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
            s = rustgame_core::transform::flip(&s, true, false);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x8969519960313E69, flaeche: s }
    }));
    v.push((105, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((24, 17), 0);
            s.fill((55u8, 1u8, 159u8, 255u8));
            rustgame_core::draw::circle(&mut s, (83u8, 178u8, 84u8, 255u8), (5, -1), 21, 3);
            let mut q = Surface::with_flags((7, 15), 0);
            q = muster_auf(q, 7, 15);
            s.blit_ex(&q, (10, 18), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            s.scroll(9, -1);
            s.fill_rect((59u8, 183u8, 95u8, 255u8), Rect::new(13, 8, 16, 18));
            rustgame_core::draw::circle(&mut s, (147u8, 63u8, 214u8, 193u8), (18, 5), 22, 2);
            rustgame_core::draw::rect_radius(&mut s, (200u8, 182u8, 7u8, 246u8), Rect::new(23, -3, 6, 1), 4, 1);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x623A959C516F2469, flaeche: s }
    }));
    v.push((106, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((28, 21), SRCALPHA);
            s.fill((29u8, 132u8, 32u8, 243u8));
            rustgame_core::draw::ellipse(&mut s, (230u8, 144u8, 114u8, 223u8), Rect::new(15, 6, 30, 20), 1);
            {
                let mut teil = s.subsurface(Rect::new(11, 15, 12, 1));
                teil.fill((169u8, 91u8, 71u8, 255u8));
            }
            let mut q = Surface::with_flags((3, 4), 0);
            q = muster_auf(q, 3, 4);
            q.set_alpha(Some(93));
            s.blit_ex(&q, (19, 10), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x48927BB6CE683924, flaeche: s }
    }));
    v.push((107, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((13, 25), 0);
            s.fill((217u8, 247u8, 158u8, 255u8));
            let mut q = Surface::with_flags((10, 26), SRCALPHA);
            q = muster_auf(q, 10, 26);
            q.set_alpha(Some(17));
            s.blit_ex(&q, (4, 16), Some(Rect::new(1, -4, 11, 17)), rustgame_core::blend::flags::BLEND_RGB_SUB);
            {
                let mut teil = s.subsurface(Rect::new(1, 2, 3, 3));
                teil.fill((234u8, 138u8, 45u8, 78u8));
            }
            {
                let mut teil = s.subsurface(Rect::new(6, 13, 7, 4));
                teil.fill((158u8, 204u8, 149u8, 255u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xAD0B092106468E0A, flaeche: s }
    }));
    v.push((108, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((10, 28), 0);
            s.fill((197u8, 141u8, 193u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (103u8, 162u8, 97u8, 255u8), &[(-1, 8), (-2, 31), (9, 2), (13, 16), (8, 9)], 0);
            rustgame_core::draw::circle(&mut s, (17u8, 176u8, 169u8, 229u8), (3, 8), 16, 3);
            {
                let mut teil = s.subsurface(Rect::new(0, 2, 8, 25));
                teil.fill((207u8, 95u8, 173u8, 57u8));
            }
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x9DAE6B43186FF28E, flaeche: s }
    }));
    v.push((109, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((14, 13), SRCALPHA);
            s.fill((116u8, 254u8, 152u8, 51u8));
            rustgame_core::draw::line(&mut s, (120u8, 230u8, 140u8, 255u8), (9, 1), (5, 13), 5);
            s.scroll(2, -8);
            rustgame_core::draw::rect_radius(&mut s, (172u8, 103u8, 70u8, 255u8), Rect::new(12, -2, 12, 14), 1, 5);
            let mut q = Surface::with_flags((4, 2), 0);
            q = muster_auf(q, 4, 2);
            q.set_alpha(Some(199));
            s.blit_ex(&q, (-1, 4), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
            rustgame_core::draw::rect_radius(&mut s, (25u8, 136u8, 238u8, 58u8), Rect::new(4, 11, 15, 0), 0, 5);
            s.fill_rect((96u8, 184u8, 203u8, 255u8), Rect::new(0, 12, 8, 15));
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x71D54A422945E487, flaeche: s }
    }));
    v.push((110, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((18, 31), 0);
            s.fill((126u8, 210u8, 246u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(17, 22, 1, 9));
                teil.fill((20u8, 59u8, 41u8, 161u8));
            }
            s.set_clip(Some(Rect::new(0, 16, 14, 10)));
            rustgame_core::draw::ellipse(&mut s, (147u8, 185u8, 186u8, 255u8), Rect::new(7, -10, 13, 29), 2);
            rustgame_core::draw::line(&mut s, (99u8, 132u8, 210u8, 65u8), (11, 10), (1, 24), 2);
            rustgame_core::draw::ellipse(&mut s, (127u8, 84u8, 156u8, 168u8), Rect::new(-5, -4, 8, 11), 0);
            rustgame_core::draw::rect_radius(&mut s, (237u8, 205u8, 103u8, 6u8), Rect::new(-3, 15, 16, 28), 2, 4);
            s.fill_rect((17u8, 165u8, 218u8, 218u8), Rect::new(-7, -11, 2, 31));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x7A5AEBB59DD2117B, flaeche: s }
    }));
    v.push((111, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((32, 12), 0);
            s.fill((161u8, 252u8, 99u8, 255u8));
            let mut q = Surface::with_flags((12, 12), SRCALPHA);
            q = muster_auf(q, 12, 12);
            q.set_colorkey(Some((237u8, 213u8, 196u8)));
            s.blit_ex(&q, (13, 8), Some(Rect::new(5, 9, 1, 10)), rustgame_core::blend::flags::BLEND_RGBA_MAX);
            rustgame_core::draw::circle(&mut s, (231u8, 208u8, 32u8, 255u8), (23, 12), 6, 2);
            s.fill_rect((174u8, 121u8, 8u8, 110u8), Rect::new(28, -6, 26, 9));
            rustgame_core::draw::circle(&mut s, (55u8, 79u8, 6u8, 250u8), (6, 11), 29, 4);
            rustgame_core::draw::rect_radius(&mut s, (104u8, 227u8, 75u8, 187u8), Rect::new(-7, 12, 34, 12), 4, 6);
            s = s.convert();
            s = rustgame_core::transform::scale(&s, (11, 10));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x1E442847660BD553, flaeche: s }
    }));
    v.push((112, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 32), 0);
            s.fill((153u8, 238u8, 215u8, 255u8));
            rustgame_core::draw::circle(&mut s, (232u8, 13u8, 199u8, 255u8), (15, 11), 29, 3);
            {
                let mut teil = s.subsurface(Rect::new(6, 14, 12, 6));
                teil.fill((208u8, 204u8, 130u8, 176u8));
            }
            s.fill_rect((114u8, 157u8, 203u8, 255u8), Rect::new(-6, 21, 0, 4));
            rustgame_core::draw::polygon(&mut s, (2u8, 132u8, 222u8, 244u8), &[(22, 29), (24, 6), (5, 31), (9, 5), (12, 35)], 0);
            {
                let mut teil = s.subsurface(Rect::new(8, 15, 10, 6));
                teil.fill((84u8, 35u8, 161u8, 119u8));
            }
            rustgame_core::draw::circle(&mut s, (161u8, 52u8, 246u8, 48u8), (-1, 27), 10, 2);
            s.fill_rect((170u8, 190u8, 141u8, 0u8), Rect::new(14, 14, 10, 32));
            {
                let mut teil = s.subsurface(Rect::new(9, 5, 2, 26));
                teil.fill((220u8, 36u8, 70u8, 211u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x626ACC163DB682BE, flaeche: s }
    }));
    v.push((113, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((6, 26), 0);
            s.fill((140u8, 9u8, 244u8, 255u8));
            let mut q = Surface::with_flags((5, 2), 0);
            q = muster_auf(q, 5, 2);
            q.set_alpha(Some(152));
            s.blit_ex(&q, (0, 15), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            {
                let mut teil = s.subsurface(Rect::new(3, 9, 3, 2));
                teil.fill((112u8, 137u8, 252u8, 88u8));
            }
            rustgame_core::draw::circle(&mut s, (208u8, 142u8, 98u8, 255u8), (-1, -3), 1, 3);
            s.set_clip(Some(Rect::new(5, 18, 1, 3)));
            s.set_clip(Some(Rect::new(1, 24, 4, 1)));
            let mut q = Surface::with_flags((6, 23), 0);
            q = muster_auf(q, 6, 23);
            q.set_colorkey(Some((157u8, 71u8, 26u8)));
            s.blit_ex(&q, (7, 25), Some(Rect::new(7, 3, 9, 7)), rustgame_core::blend::flags::BLEND_RGBA_SUB);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x949427DEE18FA7F7, flaeche: s }
    }));
    v.push((114, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((13, 24), SRCALPHA);
            s.fill((24u8, 78u8, 150u8, 124u8));
            s.fill_rect((172u8, 54u8, 47u8, 190u8), Rect::new(7, 13, 3, 19));
            rustgame_core::draw::circle(&mut s, (189u8, 222u8, 235u8, 154u8), (3, 26), 20, 1);
            rustgame_core::draw::line(&mut s, (155u8, 50u8, 40u8, 255u8), (0, 12), (7, 6), 5);
            rustgame_core::draw::ellipse(&mut s, (175u8, 42u8, 71u8, 255u8), Rect::new(-4, 7, 10, 1), 1);
            let mut q = Surface::with_flags((6, 12), SRCALPHA);
            q = muster_auf(q, 6, 12);
            q.set_alpha(Some(225));
            q.set_colorkey(Some((242u8, 212u8, 180u8)));
            s.blit_ex(&q, (7, 2), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            s.fill_rect((134u8, 28u8, 249u8, 255u8), Rect::new(-6, -12, 10, 11));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xE4DE62B678783AAE, flaeche: s }
    }));
    v.push((115, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((15, 12), 0);
            s.fill((57u8, 171u8, 149u8, 255u8));
            rustgame_core::draw::line(&mut s, (173u8, 247u8, 31u8, 255u8), (13, 13), (0, 0), 1);
            s.scroll(11, -7);
            rustgame_core::draw::polygon(&mut s, (74u8, 111u8, 72u8, 255u8), &[(16, 9), (-4, 2), (12, 6), (15, 3)], 2);
            rustgame_core::draw::rect_radius(&mut s, (210u8, 189u8, 98u8, 255u8), Rect::new(7, 4, 5, 12), 4, 6);
            rustgame_core::draw::polygon(&mut s, (170u8, 131u8, 88u8, 69u8), &[(-3, -2), (9, 15), (4, -4), (-3, 0)], 0);
            rustgame_core::draw::polygon(&mut s, (140u8, 151u8, 191u8, 192u8), &[(9, 1), (17, -3), (12, 2), (-1, 15), (-2, 6)], 0);
            rustgame_core::draw::ellipse(&mut s, (94u8, 136u8, 223u8, 174u8), Rect::new(9, 12, 17, 15), 1);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x4E4F2D3C23FFA5DF, flaeche: s }
    }));
    v.push((116, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((32, 29), 0);
            s.fill((152u8, 4u8, 61u8, 255u8));
            s.set_clip(Some(Rect::new(30, 7, 2, 14)));
            rustgame_core::draw::line(&mut s, (231u8, 32u8, 161u8, 37u8), (29, 16), (27, -1), 5);
            rustgame_core::draw::ellipse(&mut s, (206u8, 224u8, 87u8, 255u8), Rect::new(27, 19, 23, 9), 3);
            rustgame_core::draw::rect_radius(&mut s, (207u8, 135u8, 191u8, 106u8), Rect::new(9, -13, 27, 0), 3, 2);
            s.scroll(-19, -28);
            rustgame_core::draw::circle(&mut s, (118u8, 183u8, 246u8, 255u8), (22, 7), 19, 0);
            rustgame_core::draw::circle(&mut s, (54u8, 19u8, 149u8, 255u8), (-1, 6), 20, 2);
            s = s.convert();
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x1EAFA1A32316ACD5, flaeche: s }
    }));
    v.push((117, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((13, 11), SRCALPHA);
            s.fill((81u8, 208u8, 60u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (233u8, 16u8, 65u8, 178u8), Rect::new(4, 7, 2, 2), 3, 5);
            rustgame_core::draw::rect_radius(&mut s, (217u8, 93u8, 72u8, 81u8), Rect::new(8, 11, 14, 6), 1, 1);
            let mut q = Surface::with_flags((11, 3), 0);
            q = muster_auf(q, 11, 3);
            q.set_alpha(Some(107));
            s.blit_ex(&q, (7, 9), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            s.scroll(4, -1);
            rustgame_core::draw::line(&mut s, (3u8, 137u8, 146u8, 188u8), (13, -3), (14, 10), 3);
            s.fill_rect((220u8, 94u8, 25u8, 20u8), Rect::new(7, 11, 14, 10));
            rustgame_core::draw::polygon(&mut s, (14u8, 59u8, 150u8, 199u8), &[(2, 14), (8, 5), (-1, 10), (16, 4), (6, -4)], 1);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x627D5F53A04722DD, flaeche: s }
    }));
    v.push((118, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((28, 26), SRCALPHA);
            s.fill((185u8, 150u8, 9u8, 255u8));
            s.scroll(-27, 12);
            let mut q = Surface::with_flags((21, 12), 0);
            q = muster_auf(q, 21, 12);
            s.blit_ex(&q, (28, -4), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            s.set_clip(Some(Rect::new(27, 12, 1, 13)));
            rustgame_core::draw::ellipse(&mut s, (184u8, 169u8, 232u8, 31u8), Rect::new(13, -8, 28, 6), 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x0CE0634A89317B25, flaeche: s }
    }));
    v.push((119, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((15, 11), 0);
            s.fill((221u8, 55u8, 40u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (91u8, 129u8, 142u8, 255u8), Rect::new(-2, 9, 18, 0), 3, 3);
            rustgame_core::draw::line(&mut s, (173u8, 106u8, 66u8, 31u8), (10, -4), (12, 12), 4);
            rustgame_core::draw::polygon(&mut s, (108u8, 247u8, 195u8, 255u8), &[(11, 13), (15, 13), (1, -2)], 0);
            s.set_clip(Some(Rect::new(1, 2, 10, 3)));
            rustgame_core::draw::ellipse(&mut s, (195u8, 189u8, 104u8, 176u8), Rect::new(8, 11, 7, 11), 1);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xC1C8E35D01E02CD6, flaeche: s }
    }));
    v.push((120, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((22, 13), SRCALPHA);
            s.fill((46u8, 161u8, 109u8, 236u8));
            rustgame_core::draw::ellipse(&mut s, (82u8, 55u8, 162u8, 191u8), Rect::new(8, -3, 9, 9), 2);
            {
                let mut teil = s.subsurface(Rect::new(16, 8, 6, 1));
                teil.fill((15u8, 48u8, 2u8, 102u8));
            }
            rustgame_core::draw::line(&mut s, (202u8, 223u8, 196u8, 196u8), (22, 13), (22, 8), 1);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x73D0965A891674C9, flaeche: s }
    }));
    v.push((121, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 12), 0);
            s.fill((203u8, 90u8, 90u8, 255u8));
            s.scroll(5, -11);
            rustgame_core::draw::circle(&mut s, (66u8, 12u8, 240u8, 221u8), (9, 6), 7, 4);
            {
                let mut teil = s.subsurface(Rect::new(5, 2, 2, 2));
                teil.fill((120u8, 218u8, 250u8, 255u8));
            }
            {
                let mut teil = s.subsurface(Rect::new(5, 1, 2, 1));
                teil.fill((171u8, 235u8, 187u8, 255u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x6777660B21224DBD, flaeche: s }
    }));
    v.push((122, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((22, 12), 0);
            s.fill((154u8, 36u8, 187u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(21, 11, 1, 1));
                teil.fill((123u8, 203u8, 193u8, 255u8));
            }
            rustgame_core::draw::circle(&mut s, (143u8, 241u8, 37u8, 168u8), (8, -1), 16, 3);
            rustgame_core::draw::rect_radius(&mut s, (144u8, 189u8, 20u8, 252u8), Rect::new(23, 4, 12, 13), 1, 4);
            let mut q = Surface::with_flags((16, 3), 0);
            q = muster_auf(q, 16, 3);
            q.set_colorkey(Some((243u8, 212u8, 153u8)));
            s.blit_ex(&q, (7, 11), None, 0);
            rustgame_core::draw::circle(&mut s, (52u8, 246u8, 170u8, 236u8), (15, -1), 7, 0);
            let mut q = Surface::with_flags((5, 14), SRCALPHA);
            q = muster_auf(q, 5, 14);
            s.blit_ex(&q, (-2, 5), Some(Rect::new(6, 10, 3, 12)), rustgame_core::blend::flags::BLEND_RGBA_MIN);
            rustgame_core::draw::ellipse(&mut s, (86u8, 94u8, 194u8, 255u8), Rect::new(12, 4, 15, 5), 3);
            rustgame_core::draw::circle(&mut s, (203u8, 55u8, 212u8, 255u8), (16, 1), 1, 3);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xA49DF025C37E9F4D, flaeche: s }
    }));
    v.push((123, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 14), SRCALPHA);
            s.fill((19u8, 194u8, 170u8, 55u8));
            rustgame_core::draw::ellipse(&mut s, (172u8, 170u8, 125u8, 255u8), Rect::new(-2, -7, 6, 2), 4);
            let mut q = Surface::with_flags((2, 7), 0);
            q = muster_auf(q, 2, 7);
            q.set_alpha(Some(22));
            q.set_colorkey(Some((72u8, 64u8, 10u8)));
            s.blit_ex(&q, (0, 9), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            let mut q = Surface::with_flags((2, 14), SRCALPHA);
            q = muster_auf(q, 2, 14);
            s.blit_ex(&q, (9, 15), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            rustgame_core::draw::ellipse(&mut s, (203u8, 222u8, 248u8, 255u8), Rect::new(4, 14, 6, 11), 4);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xEE021E095DA427BF, flaeche: s }
    }));
    v.push((124, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((14, 23), SRCALPHA);
            s.fill((213u8, 143u8, 186u8, 30u8));
            s.scroll(-2, -4);
            rustgame_core::draw::ellipse(&mut s, (242u8, 217u8, 9u8, 255u8), Rect::new(0, 1, 3, 12), 2);
            s = s.convert_alpha();
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xBB3FEF974C1051B5, flaeche: s }
    }));
    v.push((125, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((13, 13), 0);
            s.fill((154u8, 92u8, 179u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (38u8, 79u8, 240u8, 2u8), Rect::new(9, 14, 13, 3), 2, 3);
            s.set_clip(Some(Rect::new(2, 7, 3, 1)));
            {
                let mut teil = s.subsurface(Rect::new(9, 10, 4, 1));
                teil.fill((82u8, 32u8, 28u8, 91u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x03B4DB3D57DD7F2D, flaeche: s }
    }));
    v.push((126, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((24, 7), 0);
            s.fill((233u8, 126u8, 89u8, 255u8));
            rustgame_core::draw::line(&mut s, (68u8, 249u8, 45u8, 255u8), (14, 4), (5, 1), 2);
            rustgame_core::draw::circle(&mut s, (213u8, 208u8, 142u8, 255u8), (26, 3), 16, 0);
            rustgame_core::draw::ellipse(&mut s, (89u8, 124u8, 110u8, 255u8), Rect::new(13, 2, 7, 6), 0);
            {
                let mut teil = s.subsurface(Rect::new(13, 1, 1, 5));
                teil.fill((99u8, 78u8, 211u8, 72u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (188u8, 46u8, 93u8, 255u8), Rect::new(19, 5, 26, 5), 3, 5);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xB335F39E85DDA815, flaeche: s }
    }));
    v.push((127, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 25), 0);
            s.fill((50u8, 12u8, 15u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (158u8, 195u8, 31u8, 255u8), &[(1, 14), (4, 3), (10, 20), (-3, 1), (7, 19), (1, -3)], 3);
            rustgame_core::draw::polygon(&mut s, (59u8, 62u8, 130u8, 144u8), &[(5, 2), (0, 19), (4, -1), (0, 9), (2, 14)], 3);
            {
                let mut teil = s.subsurface(Rect::new(5, 1, 2, 6));
                teil.fill((115u8, 225u8, 100u8, 124u8));
            }
            let mut q = Surface::with_flags((4, 26), SRCALPHA);
            q = muster_auf(q, 4, 26);
            q.set_colorkey(Some((148u8, 106u8, 216u8)));
            s.blit_ex(&q, (5, 23), Some(Rect::new(4, -13, 7, 3)), rustgame_core::blend::flags::BLEND_RGBA_MAX);
            s = s.convert_alpha();
            s = rustgame_core::transform::grayscale(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x40EE1CE208F649E7, flaeche: s }
    }));
    v.push((128, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((13, 18), 0);
            s.fill((68u8, 247u8, 14u8, 255u8));
            s.scroll(4, -2);
            rustgame_core::draw::rect_radius(&mut s, (34u8, 57u8, 15u8, 153u8), Rect::new(2, 15, 11, 3), 3, 1);
            rustgame_core::draw::polygon(&mut s, (252u8, 164u8, 178u8, 166u8), &[(9, 17), (5, 4), (2, 14), (-4, 0), (14, 0)], 0);
            s = s.convert_alpha();
            s = rustgame_core::transform::rotate(&s, 33.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x0909C0E0B9A7B888, flaeche: s }
    }));
    v.push((129, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((24, 31), 0);
            s.fill((23u8, 204u8, 115u8, 255u8));
            rustgame_core::draw::line(&mut s, (78u8, 69u8, 140u8, 255u8), (10, 4), (26, 34), 4);
            s.set_clip(Some(Rect::new(3, 8, 3, 9)));
            rustgame_core::draw::circle(&mut s, (21u8, 123u8, 89u8, 216u8), (2, 11), 15, 0);
            {
                let mut teil = s.subsurface(Rect::new(9, 10, 15, 5));
                teil.fill((51u8, 74u8, 96u8, 255u8));
            }
            s = rustgame_core::transform::rotate(&s, -51.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x214520E72E170F3D, flaeche: s }
    }));
    v.push((130, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((22, 19), SRCALPHA);
            s.fill((234u8, 11u8, 183u8, 225u8));
            rustgame_core::draw::rect_radius(&mut s, (173u8, 54u8, 107u8, 255u8), Rect::new(11, -10, 13, 0), 2, 1);
            s.fill_rect((174u8, 250u8, 192u8, 217u8), Rect::new(14, -8, 21, 1));
            rustgame_core::draw::rect_radius(&mut s, (66u8, 152u8, 59u8, 68u8), Rect::new(-2, 16, 4, 2), 3, 5);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x4BDFF2AED31D8AB1, flaeche: s }
    }));
    v.push((131, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((16, 17), 0);
            s.fill((44u8, 201u8, 146u8, 255u8));
            let mut q = Surface::with_flags((5, 13), 0);
            q = muster_auf(q, 5, 13);
            s.blit_ex(&q, (3, 18), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            rustgame_core::draw::circle(&mut s, (102u8, 114u8, 26u8, 250u8), (19, 1), 14, 4);
            s = rustgame_core::transform::rotate(&s, 108.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xA4A8EA26D35B2EF2, flaeche: s }
    }));
    v.push((132, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((19, 10), SRCALPHA);
            s.fill((157u8, 62u8, 12u8, 21u8));
            rustgame_core::draw::polygon(&mut s, (231u8, 87u8, 205u8, 255u8), &[(3, 11), (22, 13), (21, 13), (21, 7), (8, 6)], 1);
            rustgame_core::draw::circle(&mut s, (20u8, 181u8, 46u8, 255u8), (14, 0), 11, 0);
            rustgame_core::draw::line(&mut s, (67u8, 21u8, 247u8, 255u8), (3, -1), (10, 0), 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xB7EA44886904C171, flaeche: s }
    }));
    v.push((133, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 17), SRCALPHA);
            s.fill((16u8, 41u8, 144u8, 173u8));
            rustgame_core::draw::line(&mut s, (3u8, 192u8, 216u8, 111u8), (1, -2), (5, 0), 1);
            rustgame_core::draw::line(&mut s, (148u8, 35u8, 65u8, 92u8), (15, -1), (18, 6), 1);
            rustgame_core::draw::line(&mut s, (253u8, 35u8, 36u8, 255u8), (12, 8), (22, 11), 2);
            {
                let mut teil = s.subsurface(Rect::new(3, 12, 16, 4));
                teil.fill((18u8, 202u8, 215u8, 205u8));
            }
            let mut q = Surface::with_flags((7, 16), SRCALPHA);
            q = muster_auf(q, 7, 16);
            s.blit_ex(&q, (8, 10), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            rustgame_core::draw::ellipse(&mut s, (126u8, 121u8, 189u8, 15u8), Rect::new(-2, -8, 3, 5), 0);
            s.fill_rect((103u8, 100u8, 169u8, 255u8), Rect::new(4, 6, 5, 20));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xA1665D6A57301556, flaeche: s }
    }));
    v.push((134, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 22), SRCALPHA);
            s.fill((13u8, 248u8, 56u8, 64u8));
            {
                let mut teil = s.subsurface(Rect::new(7, 19, 9, 2));
                teil.fill((147u8, 141u8, 155u8, 52u8));
            }
            rustgame_core::draw::line(&mut s, (195u8, 183u8, 133u8, 134u8), (17, 7), (0, 22), 5);
            let mut q = Surface::with_flags((21, 11), 0);
            q = muster_auf(q, 21, 11);
            s.blit_ex(&q, (-3, -2), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            rustgame_core::draw::circle(&mut s, (49u8, 182u8, 119u8, 90u8), (16, 4), 10, 3);
            rustgame_core::draw::rect_radius(&mut s, (78u8, 5u8, 201u8, 255u8), Rect::new(0, 1, 11, 19), 4, 0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xFA75310A001E8F22, flaeche: s }
    }));
    v.push((135, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((27, 17), 0);
            s.fill((141u8, 249u8, 150u8, 255u8));
            rustgame_core::draw::circle(&mut s, (78u8, 191u8, 135u8, 5u8), (19, 7), 26, 2);
            {
                let mut teil = s.subsurface(Rect::new(26, 8, 1, 5));
                teil.fill((144u8, 1u8, 129u8, 255u8));
            }
            rustgame_core::draw::polygon(&mut s, (181u8, 175u8, 39u8, 255u8), &[(10, 20), (1, 3), (7, 18)], 3);
            rustgame_core::draw::rect_radius(&mut s, (242u8, 217u8, 156u8, 255u8), Rect::new(-4, -8, 7, 9), 1, 2);
            let mut q = Surface::with_flags((18, 6), 0);
            q = muster_auf(q, 18, 6);
            q.set_alpha(Some(41));
            q.set_colorkey(Some((41u8, 96u8, 89u8)));
            s.blit_ex(&q, (21, -3), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            rustgame_core::draw::rect_radius(&mut s, (119u8, 6u8, 228u8, 255u8), Rect::new(14, 2, 21, 4), 4, 2);
            let mut q = Surface::with_flags((14, 13), SRCALPHA);
            q = muster_auf(q, 14, 13);
            s.blit_ex(&q, (29, 0), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
            {
                let mut teil = s.subsurface(Rect::new(17, 0, 10, 16));
                teil.fill((230u8, 45u8, 213u8, 173u8));
            }
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x81CC13A2E6703943, flaeche: s }
    }));
    v.push((136, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((24, 19), 0);
            s.fill((221u8, 253u8, 103u8, 255u8));
            rustgame_core::draw::circle(&mut s, (255u8, 23u8, 83u8, 255u8), (-1, 18), 19, 0);
            {
                let mut teil = s.subsurface(Rect::new(19, 8, 5, 6));
                teil.fill((36u8, 132u8, 15u8, 15u8));
            }
            rustgame_core::draw::circle(&mut s, (167u8, 158u8, 104u8, 255u8), (12, 4), 13, 0);
            rustgame_core::draw::circle(&mut s, (200u8, 32u8, 12u8, 133u8), (10, 3), 17, 2);
            rustgame_core::draw::rect_radius(&mut s, (5u8, 240u8, 239u8, 76u8), Rect::new(-7, -6, 11, 7), 3, 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xB53B78D98BBD0E02, flaeche: s }
    }));
    v.push((137, true, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 11), SRCALPHA);
            s.fill((250u8, 178u8, 5u8, 118u8));
            let mut q = Surface::with_flags((10, 7), 0);
            q = muster_auf(q, 10, 7);
            s.blit_ex(&q, (0, 9), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            s.fill_rect((72u8, 244u8, 184u8, 255u8), Rect::new(-2, 5, 3, 13));
            let mut q = Surface::with_flags((8, 7), SRCALPHA);
            q = muster_auf(q, 8, 7);
            q.set_alpha(Some(204));
            s.blit_ex(&q, (7, 12), Some(Rect::new(-4, 5, 5, 8)), rustgame_core::blend::flags::BLEND_RGBA_MIN);
            rustgame_core::draw::rect_radius(&mut s, (121u8, 50u8, 196u8, 19u8), Rect::new(4, -6, 11, 11), 1, 4);
            {
                let mut teil = s.subsurface(Rect::new(7, 2, 1, 4));
                teil.fill((138u8, 241u8, 76u8, 140u8));
            }
            rustgame_core::draw::circle(&mut s, (199u8, 42u8, 76u8, 204u8), (9, -1), 10, 0);
            s.set_clip(Some(Rect::new(5, 10, 1, 1)));
            rustgame_core::draw::ellipse(&mut s, (237u8, 227u8, 105u8, 255u8), Rect::new(-2, -3, 3, 11), 4);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xEFD0BE1A2987A8BD, flaeche: s }
    }));
    v.push((138, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 18), SRCALPHA);
            s.fill((138u8, 126u8, 94u8, 106u8));
            rustgame_core::draw::ellipse(&mut s, (105u8, 40u8, 250u8, 255u8), Rect::new(11, -7, 4, 21), 1);
            s.set_clip(Some(Rect::new(5, 0, 6, 12)));
            rustgame_core::draw::line(&mut s, (132u8, 215u8, 68u8, 255u8), (5, 8), (12, -2), 4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x3D9EA38C296ED741, flaeche: s }
    }));
    v.push((139, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((6, 28), 0);
            s.fill((155u8, 3u8, 175u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(4, 24, 1, 3));
                teil.fill((251u8, 71u8, 161u8, 152u8));
            }
            let mut q = Surface::with_flags((6, 7), 0);
            q = muster_auf(q, 6, 7);
            q.set_alpha(Some(185));
            q.set_colorkey(Some((55u8, 130u8, 12u8)));
            s.blit_ex(&q, (-4, 31), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            s.fill_rect((41u8, 217u8, 222u8, 33u8), Rect::new(3, 10, 9, 0));
            s.scroll(2, 16);
            rustgame_core::draw::circle(&mut s, (122u8, 46u8, 152u8, 255u8), (2, 14), 20, 3);
            s.set_clip(Some(Rect::new(3, 11, 1, 13)));
            rustgame_core::draw::rect_radius(&mut s, (107u8, 255u8, 184u8, 255u8), Rect::new(-3, -2, 7, 23), 4, 0);
            {
                let mut teil = s.subsurface(Rect::new(2, 13, 2, 6));
                teil.fill((22u8, 22u8, 1u8, 152u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x674BFEE4559EAFF5, flaeche: s }
    }));
    v.push((140, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((30, 9), SRCALPHA);
            s.fill((27u8, 115u8, 7u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(22, 6, 7, 2));
                teil.fill((117u8, 97u8, 210u8, 89u8));
            }
            rustgame_core::draw::circle(&mut s, (216u8, 202u8, 103u8, 28u8), (27, -3), 22, 3);
            rustgame_core::draw::rect_radius(&mut s, (87u8, 207u8, 53u8, 255u8), Rect::new(27, 10, 13, 5), 1, 1);
            rustgame_core::draw::circle(&mut s, (157u8, 255u8, 73u8, 255u8), (10, 2), 12, 4);
            rustgame_core::draw::ellipse(&mut s, (242u8, 243u8, 179u8, 179u8), Rect::new(22, 7, 19, 12), 3);
            rustgame_core::draw::circle(&mut s, (136u8, 216u8, 102u8, 176u8), (19, 10), 11, 1);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x66916E72123E7F1C, flaeche: s }
    }));
    v.push((141, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((22, 30), SRCALPHA);
            s.fill((236u8, 0u8, 137u8, 90u8));
            rustgame_core::draw::rect_radius(&mut s, (210u8, 122u8, 137u8, 175u8), Rect::new(21, 7, 19, 30), 4, 1);
            s.set_clip(Some(Rect::new(11, 11, 10, 8)));
            rustgame_core::draw::rect_radius(&mut s, (240u8, 133u8, 245u8, 242u8), Rect::new(6, 11, 5, 33), 1, 4);
            rustgame_core::draw::circle(&mut s, (224u8, 8u8, 51u8, 255u8), (-4, 14), 0, 2);
            rustgame_core::draw::rect_radius(&mut s, (204u8, 206u8, 184u8, 238u8), Rect::new(0, 21, 14, 24), 4, 2);
            {
                let mut teil = s.subsurface(Rect::new(15, 1, 3, 11));
                teil.fill((137u8, 12u8, 162u8, 255u8));
            }
            {
                let mut teil = s.subsurface(Rect::new(7, 19, 1, 1));
                teil.fill((35u8, 241u8, 59u8, 255u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x30DFF66C9DDA708E, flaeche: s }
    }));
    v.push((142, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((24, 28), 0);
            s.fill((249u8, 84u8, 248u8, 255u8));
            let mut q = Surface::with_flags((10, 3), SRCALPHA);
            q = muster_auf(q, 10, 3);
            q.set_alpha(Some(105));
            s.blit_ex(&q, (-2, 26), Some(Rect::new(-2, 3, 5, 4)), rustgame_core::blend::flags::BLEND_RGB_MAX);
            s.fill_rect((61u8, 137u8, 68u8, 255u8), Rect::new(-12, 14, 12, 15));
            rustgame_core::draw::circle(&mut s, (204u8, 56u8, 150u8, 255u8), (13, 18), 0, 0);
            rustgame_core::draw::circle(&mut s, (58u8, 171u8, 28u8, 164u8), (7, 15), 9, 1);
            s = s.convert();
            s = rustgame_core::transform::flip(&s, false, true);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xBC8907CECF1DAC65, flaeche: s }
    }));
    v.push((143, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((11, 7), SRCALPHA);
            s.fill((42u8, 209u8, 253u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (221u8, 51u8, 143u8, 151u8), Rect::new(9, 2, 10, 9), 0, 3);
            rustgame_core::draw::polygon(&mut s, (80u8, 218u8, 155u8, 255u8), &[(-2, -1), (6, 7), (2, 4), (11, 9), (2, 5)], 1);
            rustgame_core::draw::circle(&mut s, (181u8, 128u8, 135u8, 96u8), (14, 7), 6, 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x5110D853657B643F, flaeche: s }
    }));
    v.push((144, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((20, 7), SRCALPHA);
            s.fill((203u8, 45u8, 209u8, 86u8));
            rustgame_core::draw::rect_radius(&mut s, (113u8, 214u8, 19u8, 68u8), Rect::new(19, -3, 7, 8), 2, 2);
            rustgame_core::draw::circle(&mut s, (204u8, 43u8, 177u8, 255u8), (2, 5), 7, 4);
            let mut q = Surface::with_flags((4, 9), SRCALPHA);
            q = muster_auf(q, 4, 9);
            s.blit_ex(&q, (15, 2), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            {
                let mut teil = s.subsurface(Rect::new(14, 3, 6, 1));
                teil.fill((129u8, 181u8, 120u8, 255u8));
            }
            rustgame_core::draw::circle(&mut s, (1u8, 161u8, 204u8, 254u8), (1, -1), 12, 3);
            s = s.convert_alpha();
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xA3677D42545A9425, flaeche: s }
    }));
    v.push((145, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((30, 31), SRCALPHA);
            s.fill((12u8, 164u8, 240u8, 225u8));
            let mut q = Surface::with_flags((5, 18), 0);
            q = muster_auf(q, 5, 18);
            q.set_alpha(Some(162));
            s.blit_ex(&q, (-2, 34), Some(Rect::new(1, 3, 2, 10)), 0);
            let mut q = Surface::with_flags((3, 13), 0);
            q = muster_auf(q, 3, 13);
            q.set_colorkey(Some((224u8, 221u8, 234u8)));
            s.blit_ex(&q, (30, 15), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            let mut q = Surface::with_flags((23, 2), SRCALPHA);
            q = muster_auf(q, 23, 2);
            q.set_alpha(Some(112));
            s.blit_ex(&q, (33, 12), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            s.fill_rect((117u8, 44u8, 115u8, 255u8), Rect::new(18, 10, 29, 26));
            let mut q = Surface::with_flags((11, 3), 0);
            q = muster_auf(q, 11, 3);
            s.blit_ex(&q, (7, 26), Some(Rect::new(4, 3, 4, 4)), rustgame_core::blend::flags::BLEND_RGB_ADD);
            rustgame_core::draw::polygon(&mut s, (21u8, 5u8, 12u8, 255u8), &[(19, 8), (21, 0), (10, 28), (2, 10)], 2);
            s = rustgame_core::transform::rotate(&s, 7.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x7A0CBBC8EFE79C92, flaeche: s }
    }));
    v.push((146, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((10, 10), 0);
            s.fill((33u8, 81u8, 144u8, 255u8));
            let mut q = Surface::with_flags((2, 5), 0);
            q = muster_auf(q, 2, 5);
            s.blit_ex(&q, (9, 0), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            s.fill_rect((227u8, 16u8, 201u8, 255u8), Rect::new(-1, 9, 12, 9));
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xCF35643323B6124B, flaeche: s }
    }));
    v.push((147, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((25, 26), SRCALPHA);
            s.fill((229u8, 93u8, 159u8, 225u8));
            rustgame_core::draw::ellipse(&mut s, (180u8, 35u8, 214u8, 237u8), Rect::new(9, 9, 10, 23), 4);
            {
                let mut teil = s.subsurface(Rect::new(7, 6, 2, 15));
                teil.fill((153u8, 119u8, 97u8, 39u8));
            }
            rustgame_core::draw::circle(&mut s, (133u8, 207u8, 248u8, 255u8), (9, 28), 2, 1);
            rustgame_core::draw::rect_radius(&mut s, (109u8, 78u8, 154u8, 44u8), Rect::new(25, 5, 2, 18), 3, 4);
            let mut q = Surface::with_flags((3, 9), 0);
            q = muster_auf(q, 3, 9);
            q.set_alpha(Some(177));
            s.blit_ex(&q, (8, 18), Some(Rect::new(3, 7, 3, 7)), 0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x3EC4619340CB26C1, flaeche: s }
    }));
    v.push((148, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((18, 28), 0);
            s.fill((19u8, 207u8, 179u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(6, 15, 11, 2));
                teil.fill((212u8, 163u8, 209u8, 113u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (153u8, 14u8, 132u8, 255u8), Rect::new(17, -13, 10, 11), 4, 0);
            rustgame_core::draw::polygon(&mut s, (237u8, 65u8, 193u8, 219u8), &[(12, 19), (13, 16), (-4, 22), (14, 26)], 1);
            s.fill_rect((73u8, 246u8, 142u8, 78u8), Rect::new(5, 1, 12, 17));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x2F6A8CE701CB46CD, flaeche: s }
    }));
    v.push((149, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 9), SRCALPHA);
            s.fill((28u8, 11u8, 90u8, 255u8));
            s.fill_rect((154u8, 49u8, 96u8, 255u8), Rect::new(3, -1, 1, 6));
            s.fill_rect((175u8, 242u8, 233u8, 255u8), Rect::new(0, 8, 4, 10));
            let mut q = Surface::with_flags((10, 6), SRCALPHA);
            q = muster_auf(q, 10, 6);
            s.blit_ex(&q, (7, 2), Some(Rect::new(11, 1, 9, 0)), rustgame_core::blend::flags::BLEND_RGB_SUB);
            s.set_clip(Some(Rect::new(5, 0, 3, 7)));
            rustgame_core::draw::polygon(&mut s, (54u8, 60u8, 143u8, 255u8), &[(3, -3), (3, 10), (-3, 8), (2, 3)], 0);
            s.fill_rect((133u8, 126u8, 58u8, 255u8), Rect::new(3, 1, 1, 5));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x4A997353230D48B7, flaeche: s }
    }));
    v.push((150, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((32, 16), 0);
            s.fill((96u8, 116u8, 155u8, 255u8));
            let mut q = Surface::with_flags((12, 15), SRCALPHA);
            q = muster_auf(q, 12, 15);
            s.blit_ex(&q, (24, 8), Some(Rect::new(-2, -4, 15, 11)), rustgame_core::blend::flags::BLEND_RGB_ADD);
            rustgame_core::draw::circle(&mut s, (6u8, 28u8, 54u8, 65u8), (5, 14), 13, 2);
            rustgame_core::draw::circle(&mut s, (14u8, 233u8, 87u8, 236u8), (10, -3), 19, 3);
            {
                let mut teil = s.subsurface(Rect::new(18, 3, 1, 13));
                teil.fill((22u8, 150u8, 67u8, 255u8));
            }
            s.fill_rect((86u8, 137u8, 232u8, 137u8), Rect::new(8, -6, 4, 11));
            rustgame_core::draw::polygon(&mut s, (77u8, 14u8, 49u8, 217u8), &[(7, 4), (14, 14), (20, 8), (19, 0), (34, 18)], 1);
            s.scroll(-22, 11);
            s.fill_rect((147u8, 60u8, 194u8, 255u8), Rect::new(-15, 3, 25, 5));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xF5A3C5272B880DD8, flaeche: s }
    }));
    v.push((151, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((28, 19), 0);
            s.fill((104u8, 180u8, 144u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (234u8, 54u8, 206u8, 37u8), Rect::new(4, 11, 11, 18), 2, 0);
            s.set_clip(Some(Rect::new(9, 5, 6, 3)));
            s.scroll(-25, 2);
            let mut q = Surface::with_flags((5, 11), 0);
            q = muster_auf(q, 5, 11);
            q.set_alpha(Some(239));
            q.set_colorkey(Some((53u8, 174u8, 202u8)));
            s.blit_ex(&q, (3, 20), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xBCD9644F0D71660D, flaeche: s }
    }));
    v.push((152, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((30, 18), 0);
            s.fill((195u8, 102u8, 44u8, 255u8));
            s.fill_rect((59u8, 13u8, 176u8, 255u8), Rect::new(16, 10, 11, 14));
            {
                let mut teil = s.subsurface(Rect::new(7, 2, 10, 2));
                teil.fill((235u8, 166u8, 135u8, 165u8));
            }
            rustgame_core::draw::polygon(&mut s, (50u8, 45u8, 56u8, 255u8), &[(3, 20), (0, 1), (21, 9), (-4, 7), (21, 2), (29, -2)], 1);
            rustgame_core::draw::polygon(&mut s, (238u8, 9u8, 61u8, 255u8), &[(16, 6), (33, 6), (33, 16), (10, 7)], 0);
            rustgame_core::draw::polygon(&mut s, (237u8, 211u8, 148u8, 255u8), &[(14, 5), (2, 21), (22, -2), (10, 9), (10, 11)], 1);
            rustgame_core::draw::ellipse(&mut s, (61u8, 79u8, 233u8, 89u8), Rect::new(-4, -2, 28, 2), 1);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x2465CEE035F701BB, flaeche: s }
    }));
    v.push((153, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 8), SRCALPHA);
            s.fill((162u8, 217u8, 125u8, 212u8));
            rustgame_core::draw::polygon(&mut s, (127u8, 114u8, 215u8, 105u8), &[(23, -2), (16, 4), (21, 2), (22, 7), (18, 5)], 0);
            let mut q = Surface::with_flags((15, 9), 0);
            q = muster_auf(q, 15, 9);
            q.set_alpha(Some(86));
            s.blit_ex(&q, (16, 9), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            s.fill_rect((196u8, 89u8, 165u8, 255u8), Rect::new(8, 7, 11, 8));
            s.set_clip(Some(Rect::new(18, 1, 2, 6)));
            {
                let mut teil = s.subsurface(Rect::new(15, 6, 4, 1));
                teil.fill((245u8, 245u8, 124u8, 255u8));
            }
            rustgame_core::draw::line(&mut s, (135u8, 196u8, 68u8, 255u8), (6, -4), (15, -2), 1);
            s.set_clip(Some(Rect::new(1, 6, 19, 1)));
            rustgame_core::draw::ellipse(&mut s, (204u8, 175u8, 62u8, 127u8), Rect::new(20, 3, 17, 3), 4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xF2D8F35702E958E3, flaeche: s }
    }));
    v.push((154, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 32), 0);
            s.fill((192u8, 249u8, 106u8, 255u8));
            let mut q = Surface::with_flags((9, 18), 0);
            q = muster_auf(q, 9, 18);
            s.blit_ex(&q, (14, -1), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
            rustgame_core::draw::rect(&mut s, (209u8, 230u8, 239u8, 198u8), Rect::new(13, -12, 8, 18), 1);
            let mut q = Surface::with_flags((13, 21), SRCALPHA);
            q = muster_auf(q, 13, 21);
            s.blit_ex(&q, (-3, 28), Some(Rect::new(0, -1, 1, 18)), rustgame_core::blend::flags::BLEND_RGB_MAX);
            rustgame_core::draw::line(&mut s, (231u8, 53u8, 103u8, 255u8), (8, 28), (5, -4), 1);
            let mut q = Surface::with_flags((12, 17), SRCALPHA);
            q = muster_auf(q, 12, 17);
            s.blit_ex(&q, (12, 10), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xD85E8C7818EC023F, flaeche: s }
    }));
    v.push((155, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((24, 22), 0);
            s.fill((146u8, 213u8, 212u8, 255u8));
            rustgame_core::draw::line(&mut s, (31u8, 12u8, 45u8, 98u8), (7, 1), (11, 24), 1);
            rustgame_core::draw::circle(&mut s, (173u8, 41u8, 249u8, 118u8), (9, -3), 16, 4);
            rustgame_core::draw::rect_radius(&mut s, (220u8, 179u8, 57u8, 255u8), Rect::new(21, 7, 17, 20), 3, 6);
            rustgame_core::draw::ellipse(&mut s, (249u8, 16u8, 91u8, 255u8), Rect::new(-3, 22, 16, 12), 1);
            rustgame_core::draw::rect_radius(&mut s, (195u8, 184u8, 234u8, 255u8), Rect::new(25, 2, 6, 7), 3, 6);
            rustgame_core::draw::circle(&mut s, (31u8, 203u8, 119u8, 255u8), (-3, 23), 4, 2);
            s.fill_rect((199u8, 6u8, 252u8, 255u8), Rect::new(19, 6, 8, 9));
            s.scroll(-10, -3);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x586B09462667EDA6, flaeche: s }
    }));
    v.push((156, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 19), 0);
            s.fill((126u8, 183u8, 77u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (192u8, 168u8, 77u8, 228u8), &[(-2, 0), (0, -4), (10, -3), (13, 8)], 3);
            rustgame_core::draw::circle(&mut s, (179u8, 54u8, 150u8, 255u8), (21, 7), 1, 4);
            s.set_clip(Some(Rect::new(14, 5, 3, 1)));
            s = s.convert();
            s = rustgame_core::transform::grayscale(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x3DE5AC516A77CBBB, flaeche: s }
    }));
    v.push((157, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((25, 21), SRCALPHA);
            s.fill((178u8, 177u8, 118u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (114u8, 54u8, 9u8, 255u8), Rect::new(-6, 14, 13, 12), 2);
            rustgame_core::draw::circle(&mut s, (243u8, 86u8, 13u8, 220u8), (3, 6), 24, 2);
            let mut q = Surface::with_flags((20, 13), 0);
            q = muster_auf(q, 20, 13);
            q.set_alpha(Some(47));
            s.blit_ex(&q, (18, -3), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            s.fill_rect((167u8, 71u8, 189u8, 71u8), Rect::new(21, -3, 22, 24));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xA5AD25C2541DF044, flaeche: s }
    }));
    v.push((158, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((27, 13), 0);
            s.fill((52u8, 30u8, 214u8, 255u8));
            rustgame_core::draw::circle(&mut s, (236u8, 20u8, 32u8, 255u8), (3, -4), 17, 2);
            s.fill_rect((140u8, 16u8, 105u8, 255u8), Rect::new(26, -6, 15, 7));
            s.fill_rect((59u8, 83u8, 42u8, 255u8), Rect::new(-6, 11, 30, 3));
            rustgame_core::draw::circle(&mut s, (171u8, 158u8, 121u8, 166u8), (12, 4), 22, 2);
            let mut q = Surface::with_flags((24, 12), SRCALPHA);
            q = muster_auf(q, 24, 12);
            s.blit_ex(&q, (-1, 10), Some(Rect::new(9, 10, 17, 6)), rustgame_core::blend::flags::BLEND_RGBA_MIN);
            s.fill_rect((74u8, 32u8, 99u8, 148u8), Rect::new(6, -3, 26, 4));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x60CBE7A52A1D625F, flaeche: s }
    }));
    v.push((159, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((18, 11), 0);
            s.fill((125u8, 100u8, 99u8, 255u8));
            rustgame_core::draw::circle(&mut s, (139u8, 197u8, 112u8, 17u8), (11, 9), 9, 2);
            rustgame_core::draw::circle(&mut s, (13u8, 108u8, 204u8, 255u8), (11, 11), 1, 0);
            s = rustgame_core::transform::flip(&s, false, true);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x2CC568AD55B66F39, flaeche: s }
    }));
    v.push((160, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((9, 22), SRCALPHA);
            s.fill((179u8, 252u8, 108u8, 255u8));
            let mut q = Surface::with_flags((2, 10), 0);
            q = muster_auf(q, 2, 10);
            q.set_alpha(Some(117));
            q.set_colorkey(Some((214u8, 188u8, 232u8)));
            s.blit_ex(&q, (5, 22), Some(Rect::new(1, 1, 0, 11)), rustgame_core::blend::flags::BLEND_RGBA_MIN);
            rustgame_core::draw::circle(&mut s, (233u8, 39u8, 96u8, 255u8), (0, 0), 13, 3);
            s.scroll(-9, 3);
            rustgame_core::draw::rect_radius(&mut s, (12u8, 108u8, 25u8, 255u8), Rect::new(9, 13, 2, 16), 4, 3);
            rustgame_core::draw::rect_radius(&mut s, (60u8, 196u8, 14u8, 255u8), Rect::new(4, 7, 2, 3), 0, 1);
            rustgame_core::draw::polygon(&mut s, (57u8, 14u8, 10u8, 242u8), &[(-1, 25), (10, 17), (-4, 25), (2, 12)], 0);
            s.fill_rect((51u8, 95u8, 80u8, 255u8), Rect::new(4, 21, 6, 12));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x652494CD1708AA13, flaeche: s }
    }));
    v.push((161, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((22, 12), 0);
            s.fill((117u8, 245u8, 17u8, 255u8));
            rustgame_core::draw::line(&mut s, (249u8, 25u8, 235u8, 246u8), (18, -3), (11, 15), 1);
            rustgame_core::draw::rect_radius(&mut s, (6u8, 133u8, 141u8, 2u8), Rect::new(14, 9, 6, 11), 3, 3);
            let mut q = Surface::with_flags((16, 2), 0);
            q = muster_auf(q, 16, 2);
            q.set_alpha(Some(195));
            q.set_colorkey(Some((208u8, 21u8, 9u8)));
            s.blit_ex(&q, (4, 13), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            rustgame_core::draw::rect_radius(&mut s, (136u8, 39u8, 183u8, 17u8), Rect::new(4, 10, 10, 6), 1, 5);
            rustgame_core::draw::rect_radius(&mut s, (10u8, 91u8, 118u8, 247u8), Rect::new(12, 11, 2, 1), 3, 2);
            rustgame_core::draw::ellipse(&mut s, (212u8, 189u8, 124u8, 255u8), Rect::new(19, -2, 7, 0), 0);
            {
                let mut teil = s.subsurface(Rect::new(14, 3, 5, 7));
                teil.fill((191u8, 68u8, 77u8, 255u8));
            }
            s.scroll(-18, -12);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xC141301A2D32BE4F, flaeche: s }
    }));
    v.push((162, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 28), 0);
            s.fill((104u8, 103u8, 211u8, 255u8));
            rustgame_core::draw::circle(&mut s, (230u8, 235u8, 13u8, 150u8), (-2, 3), 13, 2);
            rustgame_core::draw::rect_radius(&mut s, (8u8, 140u8, 36u8, 234u8), Rect::new(-2, 12, 7, 2), 0, 2);
            rustgame_core::draw::polygon(&mut s, (57u8, 229u8, 129u8, 0u8), &[(-1, 31), (0, 9), (2, 11), (9, 12)], 3);
            {
                let mut teil = s.subsurface(Rect::new(6, 21, 1, 7));
                teil.fill((13u8, 134u8, 15u8, 121u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (158u8, 190u8, 248u8, 255u8), Rect::new(2, 17, 3, 15), 3, 3);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xDFE5CFB8EBCF1724, flaeche: s }
    }));
    v.push((163, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((23, 25), SRCALPHA);
            s.fill((145u8, 110u8, 180u8, 252u8));
            let mut q = Surface::with_flags((6, 6), SRCALPHA);
            q = muster_auf(q, 6, 6);
            q.set_alpha(Some(35));
            s.blit_ex(&q, (-3, 0), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            rustgame_core::draw::rect_radius(&mut s, (80u8, 186u8, 73u8, 255u8), Rect::new(-8, 7, 20, 11), 3, 6);
            s.scroll(18, -10);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xC082E20DD10F3B3C, flaeche: s }
    }));
    v.push((164, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((9, 29), SRCALPHA);
            s.fill((250u8, 196u8, 61u8, 237u8));
            let mut q = Surface::with_flags((2, 28), 0);
            q = muster_auf(q, 2, 28);
            q.set_alpha(Some(156));
            s.blit_ex(&q, (-1, 24), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            rustgame_core::draw::circle(&mut s, (45u8, 187u8, 31u8, 58u8), (5, -2), 22, 2);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x8AA578705ECEE845, flaeche: s }
    }));
    v.push((165, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((6, 7), 0);
            s.fill((220u8, 37u8, 252u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (247u8, 29u8, 101u8, 161u8), &[(-3, 6), (2, 8), (6, -2), (5, 9), (2, -4), (6, 2)], 3);
            rustgame_core::draw::rect_radius(&mut s, (248u8, 219u8, 135u8, 255u8), Rect::new(6, -2, 8, 4), 1, 1);
            s.scroll(2, 2);
            s = rustgame_core::transform::grayscale(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x34625F079A479D07, flaeche: s }
    }));
    v.push((166, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((11, 26), 0);
            s.fill((219u8, 6u8, 167u8, 255u8));
            s.fill_rect((148u8, 102u8, 87u8, 188u8), Rect::new(-6, 18, 12, 3));
            {
                let mut teil = s.subsurface(Rect::new(6, 3, 1, 6));
                teil.fill((111u8, 248u8, 81u8, 255u8));
            }
            s.scroll(-3, -13);
            rustgame_core::draw::rect_radius(&mut s, (239u8, 25u8, 135u8, 255u8), Rect::new(8, 4, 12, 28), 1, 3);
            rustgame_core::draw::rect_radius(&mut s, (253u8, 41u8, 125u8, 54u8), Rect::new(12, -13, 6, 28), 2, 4);
            let mut q = Surface::with_flags((13, 23), 0);
            q = muster_auf(q, 13, 23);
            s.blit_ex(&q, (6, 13), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            rustgame_core::draw::rect_radius(&mut s, (180u8, 56u8, 12u8, 226u8), Rect::new(4, 12, 5, 10), 1, 5);
            rustgame_core::draw::rect_radius(&mut s, (19u8, 2u8, 101u8, 255u8), Rect::new(8, 12, 3, 2), 1, 0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x58E5E644E54256CB, flaeche: s }
    }));
    v.push((167, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((13, 32), SRCALPHA);
            s.fill((1u8, 220u8, 252u8, 42u8));
            rustgame_core::draw::line(&mut s, (58u8, 47u8, 76u8, 255u8), (5, 22), (-3, 29), 3);
            s.set_clip(Some(Rect::new(1, 0, 2, 27)));
            s.fill_rect((229u8, 165u8, 74u8, 183u8), Rect::new(3, 26, 11, 1));
            let mut q = Surface::with_flags((15, 2), 0);
            q = muster_auf(q, 15, 2);
            q.set_alpha(Some(130));
            s.blit_ex(&q, (10, 7), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            let mut q = Surface::with_flags((15, 7), SRCALPHA);
            q = muster_auf(q, 15, 7);
            q.set_alpha(Some(49));
            s.blit_ex(&q, (2, 21), Some(Rect::new(-1, 2, 12, 6)), 0);
            rustgame_core::draw::rect_radius(&mut s, (167u8, 5u8, 72u8, 255u8), Rect::new(9, 24, 7, 30), 2, 2);
            rustgame_core::draw::rect_radius(&mut s, (62u8, 180u8, 70u8, 115u8), Rect::new(12, 11, 9, 15), 4, 4);
            let mut q = Surface::with_flags((8, 24), 0);
            q = muster_auf(q, 8, 24);
            q.set_alpha(Some(73));
            s.blit_ex(&q, (1, 11), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x30AC05414130676C, flaeche: s }
    }));
    v.push((168, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((29, 9), 0);
            s.fill((194u8, 94u8, 113u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(27, 8, 2, 1));
                teil.fill((58u8, 59u8, 214u8, 196u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (86u8, 192u8, 130u8, 255u8), Rect::new(14, -5, 0, 3), 2, 2);
            s.set_clip(Some(Rect::new(13, 7, 1, 2)));
            s.scroll(-2, -5);
            rustgame_core::draw::circle(&mut s, (12u8, 126u8, 17u8, 39u8), (-4, 3), 14, 3);
            rustgame_core::draw::ellipse(&mut s, (202u8, 183u8, 59u8, 77u8), Rect::new(-12, -1, 11, 12), 4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x573A537DCA25F705, flaeche: s }
    }));
    v.push((169, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((14, 22), 0);
            s.fill((62u8, 176u8, 137u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (65u8, 243u8, 23u8, 0u8), Rect::new(0, 17, 4, 1), 4, 2);
            s.scroll(3, -20);
            let mut q = Surface::with_flags((4, 20), SRCALPHA);
            q = muster_auf(q, 4, 20);
            q.set_alpha(Some(252));
            s.blit_ex(&q, (4, 9), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            rustgame_core::draw::rect_radius(&mut s, (98u8, 63u8, 74u8, 255u8), Rect::new(-6, -6, 9, 23), 2, 5);
            {
                let mut teil = s.subsurface(Rect::new(9, 13, 1, 8));
                teil.fill((111u8, 210u8, 239u8, 94u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x3AA864B7C34432E5, flaeche: s }
    }));
    v.push((170, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((31, 17), 0);
            s.fill((163u8, 161u8, 47u8, 255u8));
            s.scroll(6, -10);
            {
                let mut teil = s.subsurface(Rect::new(25, 0, 4, 17));
                teil.fill((255u8, 61u8, 179u8, 255u8));
            }
            rustgame_core::draw::rect(&mut s, (235u8, 161u8, 90u8, 255u8), Rect::new(13, 9, 11, 5), 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xCB9E433FDF2F53D7, flaeche: s }
    }));
    v.push((171, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((31, 31), 0);
            s.fill((211u8, 60u8, 236u8, 255u8));
            s.set_clip(Some(Rect::new(9, 10, 6, 6)));
            s.set_clip(Some(Rect::new(24, 29, 7, 1)));
            rustgame_core::draw::polygon(&mut s, (54u8, 92u8, 31u8, 255u8), &[(31, 34), (16, 1), (33, 17), (21, 3)], 3);
            s.set_clip(Some(Rect::new(4, 16, 16, 15)));
            rustgame_core::draw::rect_radius(&mut s, (217u8, 80u8, 4u8, 12u8), Rect::new(-2, 7, 31, 23), 0, 2);
            s.scroll(3, 12);
            let mut q = Surface::with_flags((28, 23), 0);
            q = muster_auf(q, 28, 23);
            q.set_colorkey(Some((86u8, 246u8, 175u8)));
            s.blit_ex(&q, (16, 12), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x498DA4CBD0A4C187, flaeche: s }
    }));
    v.push((172, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((17, 12), SRCALPHA);
            s.fill((200u8, 236u8, 226u8, 25u8));
            rustgame_core::draw::polygon(&mut s, (48u8, 218u8, 0u8, 153u8), &[(12, 0), (3, 12), (10, 4)], 2);
            s.scroll(-2, 1);
            rustgame_core::draw::circle(&mut s, (231u8, 245u8, 165u8, 218u8), (5, -4), 12, 0);
            rustgame_core::draw::line(&mut s, (236u8, 212u8, 7u8, 216u8), (9, 6), (14, 0), 4);
            rustgame_core::draw::rect_radius(&mut s, (79u8, 156u8, 128u8, 91u8), Rect::new(-2, 8, 9, 6), 1, 3);
            rustgame_core::draw::circle(&mut s, (162u8, 115u8, 158u8, 13u8), (4, 5), 3, 3);
            s.fill_rect((103u8, 30u8, 169u8, 255u8), Rect::new(6, -4, 10, 5));
            let mut q = Surface::with_flags((9, 3), 0);
            q = muster_auf(q, 9, 3);
            q.set_colorkey(Some((92u8, 49u8, 213u8)));
            s.blit_ex(&q, (17, 1), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            s = s.convert_alpha();
            s = rustgame_core::transform::grayscale(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xB19599ACC3CFC5C6, flaeche: s }
    }));
    v.push((173, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((16, 21), 0);
            s.fill((187u8, 125u8, 208u8, 255u8));
            rustgame_core::draw::circle(&mut s, (3u8, 76u8, 231u8, 255u8), (0, 10), 20, 1);
            rustgame_core::draw::circle(&mut s, (1u8, 232u8, 74u8, 255u8), (6, 8), 3, 3);
            {
                let mut teil = s.subsurface(Rect::new(3, 8, 7, 7));
                teil.fill((233u8, 240u8, 40u8, 87u8));
            }
            s.set_clip(Some(Rect::new(14, 19, 1, 2)));
            let mut q = Surface::with_flags((18, 13), 0);
            q = muster_auf(q, 18, 13);
            q.set_colorkey(Some((148u8, 39u8, 110u8)));
            s.blit_ex(&q, (18, 10), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
            let mut q = Surface::with_flags((13, 5), 0);
            q = muster_auf(q, 13, 5);
            s.blit_ex(&q, (5, 11), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            rustgame_core::draw::line(&mut s, (223u8, 171u8, 155u8, 255u8), (-2, 22), (5, -2), 3);
            s = rustgame_core::transform::rotate(&s, 114.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xE48BD01EEE0AC68A, flaeche: s }
    }));
    v.push((174, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((31, 6), SRCALPHA);
            s.fill((184u8, 34u8, 115u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(28, 5, 2, 1));
                teil.fill((45u8, 165u8, 168u8, 191u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (8u8, 59u8, 31u8, 233u8), Rect::new(2, 4, 31, 4), 0, 3);
            let mut q = Surface::with_flags((31, 2), SRCALPHA);
            q = muster_auf(q, 31, 2);
            s.blit_ex(&q, (-1, -1), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            let mut q = Surface::with_flags((13, 6), SRCALPHA);
            q = muster_auf(q, 13, 6);
            q.set_alpha(Some(109));
            s.blit_ex(&q, (22, -4), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            s.fill_rect((141u8, 71u8, 188u8, 70u8), Rect::new(25, 5, 26, 2));
            s.fill_rect((38u8, 170u8, 143u8, 74u8), Rect::new(11, -2, 10, 6));
            s.fill_rect((109u8, 97u8, 234u8, 27u8), Rect::new(19, 1, 19, 5));
            s.set_clip(Some(Rect::new(22, 4, 1, 2)));
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x013F34FBAA36D9E7, flaeche: s }
    }));
    v.push((175, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((20, 13), SRCALPHA);
            s.fill((131u8, 38u8, 255u8, 99u8));
            rustgame_core::draw::ellipse(&mut s, (140u8, 92u8, 155u8, 16u8), Rect::new(1, 8, 17, 14), 3);
            rustgame_core::draw::polygon(&mut s, (176u8, 177u8, 241u8, 238u8), &[(22, -1), (13, 14), (-3, -3), (18, 7), (3, -3)], 1);
            let mut q = Surface::with_flags((18, 14), 0);
            q = muster_auf(q, 18, 14);
            q.set_alpha(Some(136));
            s.blit_ex(&q, (10, -2), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            rustgame_core::draw::ellipse(&mut s, (11u8, 75u8, 104u8, 142u8), Rect::new(10, -1, 18, 16), 4);
            rustgame_core::draw::ellipse(&mut s, (21u8, 58u8, 65u8, 255u8), Rect::new(17, -2, 9, 3), 2);
            rustgame_core::draw::circle(&mut s, (25u8, 21u8, 234u8, 16u8), (16, 12), 1, 1);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x0FC30B1DD3008420, flaeche: s }
    }));
    v.push((176, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((6, 7), 0);
            s.fill((114u8, 56u8, 197u8, 255u8));
            s.scroll(4, 2);
            let mut q = Surface::with_flags((2, 2), SRCALPHA);
            q = muster_auf(q, 2, 2);
            s.blit_ex(&q, (0, 0), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            rustgame_core::draw::rect_radius(&mut s, (91u8, 230u8, 119u8, 8u8), Rect::new(7, 7, 6, 0), 0, 3);
            let mut q = Surface::with_flags((7, 7), SRCALPHA);
            q = muster_auf(q, 7, 7);
            q.set_alpha(Some(188));
            q.set_colorkey(Some((129u8, 50u8, 224u8)));
            s.blit_ex(&q, (2, -3), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
            s.set_clip(Some(Rect::new(0, 1, 5, 3)));
            s.scroll(6, -3);
            let mut q = Surface::with_flags((3, 2), 0);
            q = muster_auf(q, 3, 2);
            q.set_alpha(Some(111));
            s.blit_ex(&q, (0, -3), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x5A84CB2F4612E438, flaeche: s }
    }));
    v.push((177, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 21), SRCALPHA);
            s.fill((206u8, 22u8, 219u8, 255u8));
            rustgame_core::draw::line(&mut s, (140u8, 46u8, 62u8, 153u8), (-1, 22), (13, 18), 3);
            rustgame_core::draw::ellipse(&mut s, (206u8, 38u8, 50u8, 234u8), Rect::new(12, 3, 9, 5), 3);
            rustgame_core::draw::ellipse(&mut s, (66u8, 185u8, 68u8, 255u8), Rect::new(7, 11, 11, 11), 2);
            rustgame_core::draw::circle(&mut s, (238u8, 111u8, 169u8, 255u8), (14, 17), 6, 0);
            let mut q = Surface::with_flags((8, 21), 0);
            q = muster_auf(q, 8, 21);
            q.set_alpha(Some(111));
            s.blit_ex(&q, (15, -1), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            rustgame_core::draw::rect_radius(&mut s, (146u8, 169u8, 7u8, 228u8), Rect::new(4, 9, 11, 19), 0, 1);
            rustgame_core::draw::circle(&mut s, (186u8, 18u8, 246u8, 134u8), (4, 15), 8, 0);
            {
                let mut teil = s.subsurface(Rect::new(11, 19, 1, 2));
                teil.fill((242u8, 10u8, 124u8, 240u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xC4A2D4C3EBB25A85, flaeche: s }
    }));
    v.push((178, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((9, 23), 0);
            s.fill((123u8, 172u8, 247u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(7, 7, 1, 9));
                teil.fill((196u8, 239u8, 58u8, 85u8));
            }
            s.fill_rect((60u8, 134u8, 140u8, 255u8), Rect::new(7, -7, 0, 21));
            rustgame_core::draw::rect_radius(&mut s, (185u8, 55u8, 12u8, 130u8), Rect::new(0, 1, 10, 3), 4, 6);
            rustgame_core::draw::circle(&mut s, (68u8, 98u8, 179u8, 190u8), (-3, 5), 5, 3);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x3DEF6EEAC5E5FBA7, flaeche: s }
    }));
    v.push((179, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 9), 0);
            s.fill((29u8, 155u8, 227u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (112u8, 124u8, 153u8, 130u8), Rect::new(4, 10, 4, 5), 1, 3);
            s.set_clip(Some(Rect::new(3, 1, 5, 1)));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xC9EAC2947B77C295, flaeche: s }
    }));
    v.push((180, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((10, 8), 0);
            s.fill((135u8, 62u8, 230u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (2u8, 15u8, 68u8, 255u8), Rect::new(1, -3, 2, 6), 4, 6);
            rustgame_core::draw::rect_radius(&mut s, (138u8, 170u8, 14u8, 61u8), Rect::new(2, -4, 13, 0), 3, 3);
            rustgame_core::draw::line(&mut s, (219u8, 154u8, 41u8, 199u8), (-4, 7), (3, 3), 3);
            s = s.convert_alpha();
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x59AA9DD4CFF129B9, flaeche: s }
    }));
    v.push((181, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((24, 28), SRCALPHA);
            s.fill((173u8, 98u8, 159u8, 255u8));
            s.scroll(4, 1);
            rustgame_core::draw::circle(&mut s, (115u8, 12u8, 29u8, 255u8), (15, 19), 8, 2);
            {
                let mut teil = s.subsurface(Rect::new(19, 12, 3, 13));
                teil.fill((101u8, 18u8, 235u8, 141u8));
            }
            rustgame_core::draw::ellipse(&mut s, (54u8, 223u8, 115u8, 193u8), Rect::new(1, 26, 4, 28), 1);
            rustgame_core::draw::circle(&mut s, (112u8, 61u8, 254u8, 22u8), (21, 1), 25, 2);
            s.scroll(0, -2);
            rustgame_core::draw::polygon(&mut s, (90u8, 87u8, 17u8, 255u8), &[(22, 14), (20, 28), (5, 27), (18, 7), (22, 19)], 0);
            rustgame_core::draw::line(&mut s, (64u8, 148u8, 41u8, 41u8), (5, -2), (-2, -2), 5);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x7A6E8E760D7AEDF3, flaeche: s }
    }));
    v.push((182, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((20, 17), SRCALPHA);
            s.fill((145u8, 39u8, 136u8, 178u8));
            rustgame_core::draw::rect_radius(&mut s, (47u8, 47u8, 109u8, 141u8), Rect::new(13, 15, 0, 3), 1, 1);
            rustgame_core::draw::rect_radius(&mut s, (120u8, 234u8, 66u8, 10u8), Rect::new(10, 11, 21, 16), 1, 4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xAC5EB49F51945659, flaeche: s }
    }));
    v.push((183, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((9, 8), 0);
            s.fill((0u8, 182u8, 154u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(1, 1, 1, 6));
                teil.fill((37u8, 219u8, 117u8, 16u8));
            }
            rustgame_core::draw::line(&mut s, (230u8, 23u8, 22u8, 7u8), (9, 8), (-3, 0), 1);
            rustgame_core::draw::rect_radius(&mut s, (203u8, 206u8, 155u8, 255u8), Rect::new(3, 7, 4, 8), 0, 5);
            rustgame_core::draw::ellipse(&mut s, (122u8, 35u8, 18u8, 212u8), Rect::new(3, -1, 10, 4), 2);
            let mut q = Surface::with_flags((7, 3), 0);
            q = muster_auf(q, 7, 3);
            s.blit_ex(&q, (-3, -4), Some(Rect::new(8, 2, 2, 0)), rustgame_core::blend::flags::BLEND_RGB_MAX);
            s.fill_rect((10u8, 145u8, 222u8, 255u8), Rect::new(2, -2, 12, 11));
            rustgame_core::draw::ellipse(&mut s, (233u8, 245u8, 71u8, 252u8), Rect::new(6, -2, 5, 2), 2);
            rustgame_core::draw::circle(&mut s, (65u8, 119u8, 66u8, 255u8), (3, 9), 1, 4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xBB131DDE1E9C8AE0, flaeche: s }
    }));
    v.push((184, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((14, 6), SRCALPHA);
            s.fill((170u8, 172u8, 106u8, 166u8));
            let mut q = Surface::with_flags((12, 2), SRCALPHA);
            q = muster_auf(q, 12, 2);
            q.set_alpha(Some(88));
            s.blit_ex(&q, (-2, 9), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            rustgame_core::draw::rect(&mut s, (163u8, 6u8, 165u8, 37u8), Rect::new(6, 6, 2, 4), 4);
            let mut q = Surface::with_flags((13, 8), 0);
            q = muster_auf(q, 13, 8);
            s.blit_ex(&q, (9, 4), Some(Rect::new(14, 5, 11, 11)), rustgame_core::blend::flags::BLEND_RGB_MULT);
            rustgame_core::draw::circle(&mut s, (73u8, 41u8, 105u8, 56u8), (4, 6), 6, 2);
            rustgame_core::draw::polygon(&mut s, (206u8, 67u8, 201u8, 255u8), &[(16, 9), (8, 2), (15, 3), (17, 4), (6, 1)], 3);
            s.fill_rect((84u8, 165u8, 2u8, 132u8), Rect::new(13, 1, 7, 0));
            s.set_clip(Some(Rect::new(7, 5, 5, 1)));
            s.set_clip(Some(Rect::new(1, 1, 8, 4)));
            s = rustgame_core::transform::scale(&s, (3, 4));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xA8D3D4E398C4F0E5, flaeche: s }
    }));
    v.push((185, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((31, 9), SRCALPHA);
            s.fill((248u8, 243u8, 205u8, 255u8));
            s.fill_rect((135u8, 223u8, 13u8, 123u8), Rect::new(24, 1, 24, 4));
            rustgame_core::draw::rect(&mut s, (94u8, 195u8, 177u8, 255u8), Rect::new(21, -1, 7, 2), 1);
            rustgame_core::draw::ellipse(&mut s, (25u8, 65u8, 118u8, 180u8), Rect::new(-11, 9, 23, 6), 2);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x74379428D476959E, flaeche: s }
    }));
    v.push((186, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((16, 27), SRCALPHA);
            s.fill((55u8, 242u8, 61u8, 229u8));
            {
                let mut teil = s.subsurface(Rect::new(10, 17, 1, 8));
                teil.fill((231u8, 201u8, 240u8, 255u8));
            }
            rustgame_core::draw::circle(&mut s, (51u8, 182u8, 40u8, 255u8), (5, 0), 23, 0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xD1A44052991D774F, flaeche: s }
    }));
    v.push((187, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((17, 32), SRCALPHA);
            s.fill((125u8, 253u8, 232u8, 12u8));
            rustgame_core::draw::ellipse(&mut s, (112u8, 169u8, 122u8, 177u8), Rect::new(-1, 23, 19, 21), 3);
            rustgame_core::draw::circle(&mut s, (177u8, 200u8, 220u8, 255u8), (15, 7), 10, 3);
            rustgame_core::draw::rect_radius(&mut s, (188u8, 148u8, 134u8, 92u8), Rect::new(-7, -8, 16, 33), 3, 0);
            let mut q = Surface::with_flags((11, 28), SRCALPHA);
            q = muster_auf(q, 11, 28);
            q.set_colorkey(Some((26u8, 83u8, 184u8)));
            s.blit_ex(&q, (7, 12), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            rustgame_core::draw::circle(&mut s, (238u8, 161u8, 217u8, 28u8), (-1, 24), 14, 1);
            let mut q = Surface::with_flags((5, 4), SRCALPHA);
            q = muster_auf(q, 5, 4);
            q.set_alpha(Some(245));
            s.blit_ex(&q, (-3, 21), Some(Rect::new(3, 4, 6, 6)), rustgame_core::blend::flags::BLEND_RGBA_ADD);
            s.fill_rect((163u8, 48u8, 253u8, 115u8), Rect::new(13, -13, 4, 11));
            rustgame_core::draw::polygon(&mut s, (26u8, 249u8, 251u8, 29u8), &[(-1, 15), (5, 16), (3, 28), (2, 14), (4, -2)], 1);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xCB4D25FCD35790A2, flaeche: s }
    }));
    v.push((188, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((31, 28), 0);
            s.fill((113u8, 135u8, 1u8, 255u8));
            s.fill_rect((135u8, 179u8, 153u8, 255u8), Rect::new(1, -7, 6, 1));
            rustgame_core::draw::circle(&mut s, (176u8, 57u8, 126u8, 242u8), (12, -2), 10, 3);
            {
                let mut teil = s.subsurface(Rect::new(15, 21, 3, 2));
                teil.fill((90u8, 97u8, 174u8, 255u8));
            }
            s = rustgame_core::transform::rotate(&s, 80.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x32A6CE29A362EA0B, flaeche: s }
    }));
    v.push((189, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 32), 0);
            s.fill((73u8, 59u8, 206u8, 255u8));
            s.fill_rect((17u8, 73u8, 102u8, 255u8), Rect::new(8, 2, 1, 25));
            rustgame_core::draw::circle(&mut s, (233u8, 105u8, 3u8, 58u8), (8, 6), 14, 1);
            rustgame_core::draw::rect_radius(&mut s, (9u8, 156u8, 173u8, 209u8), Rect::new(-5, -11, 1, 29), 4, 4);
            s.fill_rect((178u8, 90u8, 49u8, 255u8), Rect::new(5, -9, 12, 33));
            s.set_clip(Some(Rect::new(20, 5, 1, 14)));
            {
                let mut teil = s.subsurface(Rect::new(1, 20, 19, 6));
                teil.fill((161u8, 246u8, 174u8, 203u8));
            }
            rustgame_core::draw::polygon(&mut s, (170u8, 49u8, 247u8, 50u8), &[(3, 14), (22, 31), (20, 20), (6, -2), (5, 26)], 1);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x95D41DCA3DFE05B8, flaeche: s }
    }));
    v.push((190, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 11), 0);
            s.fill((189u8, 106u8, 113u8, 255u8));
            let mut q = Surface::with_flags((3, 4), SRCALPHA);
            q = muster_auf(q, 3, 4);
            s.blit_ex(&q, (-3, 14), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            let mut q = Surface::with_flags((7, 13), SRCALPHA);
            q = muster_auf(q, 7, 13);
            s.blit_ex(&q, (6, 2), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            rustgame_core::draw::line(&mut s, (63u8, 68u8, 170u8, 145u8), (2, -4), (8, 14), 2);
            let mut q = Surface::with_flags((9, 8), 0);
            q = muster_auf(q, 9, 8);
            s.blit_ex(&q, (3, 4), Some(Rect::new(0, -1, 5, 1)), rustgame_core::blend::flags::BLEND_RGBA_SUB);
            s.set_clip(Some(Rect::new(3, 3, 5, 6)));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x8FC5677E1368896D, flaeche: s }
    }));
    v.push((191, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((30, 22), 0);
            s.fill((74u8, 206u8, 245u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (247u8, 187u8, 217u8, 207u8), Rect::new(8, -8, 5, 20), 1, 3);
            s.fill_rect((6u8, 151u8, 139u8, 185u8), Rect::new(8, 3, 1, 11));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x79AE0D4A9CD6210E, flaeche: s }
    }));
    v.push((192, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((17, 32), SRCALPHA);
            s.fill((171u8, 218u8, 141u8, 217u8));
            {
                let mut teil = s.subsurface(Rect::new(5, 8, 7, 6));
                teil.fill((232u8, 103u8, 133u8, 136u8));
            }
            let mut q = Surface::with_flags((11, 33), 0);
            q = muster_auf(q, 11, 33);
            s.blit_ex(&q, (3, 12), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            {
                let mut teil = s.subsurface(Rect::new(7, 16, 3, 12));
                teil.fill((161u8, 53u8, 198u8, 207u8));
            }
            s.fill_rect((156u8, 224u8, 188u8, 64u8), Rect::new(8, 21, 20, 2));
            {
                let mut teil = s.subsurface(Rect::new(4, 19, 12, 12));
                teil.fill((134u8, 106u8, 55u8, 200u8));
            }
            rustgame_core::draw::line(&mut s, (223u8, 180u8, 31u8, 255u8), (4, 29), (2, 5), 4);
            s.scroll(-9, 6);
            s = rustgame_core::transform::scale(&s, (8, 34));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x3213AC7CFD7AAADE, flaeche: s }
    }));
    v.push((193, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((23, 20), 0);
            s.fill((88u8, 152u8, 138u8, 255u8));
            rustgame_core::draw::line(&mut s, (79u8, 232u8, 106u8, 255u8), (25, 2), (21, 23), 4);
            let mut q = Surface::with_flags((10, 12), 0);
            q = muster_auf(q, 10, 12);
            s.blit_ex(&q, (26, 19), Some(Rect::new(0, 3, 12, 13)), rustgame_core::blend::flags::BLEND_RGBA_MAX);
            let mut q = Surface::with_flags((4, 20), SRCALPHA);
            q = muster_auf(q, 4, 20);
            s.blit_ex(&q, (26, 6), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            let mut q = Surface::with_flags((25, 8), SRCALPHA);
            q = muster_auf(q, 25, 8);
            q.set_colorkey(Some((42u8, 187u8, 72u8)));
            s.blit_ex(&q, (18, 16), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            rustgame_core::draw::circle(&mut s, (76u8, 11u8, 224u8, 244u8), (19, 17), 0, 4);
            s.fill_rect((81u8, 183u8, 223u8, 255u8), Rect::new(23, -2, 13, 15));
            s = rustgame_core::transform::flip(&s, false, true);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xBDCDDCA1CBF40D4F, flaeche: s }
    }));
    v.push((194, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((32, 25), SRCALPHA);
            s.fill((198u8, 128u8, 223u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (209u8, 191u8, 73u8, 220u8), Rect::new(24, -7, 10, 21), 3);
            s.set_clip(Some(Rect::new(13, 17, 4, 3)));
            rustgame_core::draw::ellipse(&mut s, (108u8, 108u8, 205u8, 45u8), Rect::new(29, 16, 10, 14), 1);
            let mut q = Surface::with_flags((15, 24), 0);
            q = muster_auf(q, 15, 24);
            q.set_alpha(Some(216));
            s.blit_ex(&q, (30, 20), Some(Rect::new(13, 19, 17, 25)), rustgame_core::blend::flags::BLEND_RGB_SUB);
            let mut q = Surface::with_flags((34, 23), SRCALPHA);
            q = muster_auf(q, 34, 23);
            s.blit_ex(&q, (22, 13), Some(Rect::new(3, 14, 30, 8)), rustgame_core::blend::flags::BLEND_RGBA_MULT);
            s.set_clip(Some(Rect::new(31, 5, 1, 11)));
            s.fill_rect((97u8, 175u8, 53u8, 47u8), Rect::new(-12, 12, 6, 18));
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x8D76303C28A60241, flaeche: s }
    }));
    v.push((195, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((25, 18), SRCALPHA);
            s.fill((106u8, 203u8, 40u8, 255u8));
            s.scroll(11, -6);
            rustgame_core::draw::circle(&mut s, (32u8, 220u8, 75u8, 120u8), (2, 3), 21, 0);
            s.fill_rect((27u8, 74u8, 80u8, 29u8), Rect::new(-5, 1, 16, 0));
            let mut q = Surface::with_flags((23, 11), 0);
            q = muster_auf(q, 23, 11);
            q.set_alpha(Some(214));
            q.set_colorkey(Some((255u8, 99u8, 103u8)));
            s.blit_ex(&q, (14, 21), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            rustgame_core::draw::rect(&mut s, (228u8, 127u8, 128u8, 255u8), Rect::new(-13, 15, 14, 19), 0);
            rustgame_core::draw::ellipse(&mut s, (7u8, 41u8, 188u8, 55u8), Rect::new(3, 2, 10, 17), 3);
            let mut q = Surface::with_flags((26, 9), 0);
            q = muster_auf(q, 26, 9);
            s.blit_ex(&q, (27, 21), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xFF8A616A9ADCEDDB, flaeche: s }
    }));
    v.push((196, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((26, 22), 0);
            s.fill((14u8, 107u8, 216u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (79u8, 142u8, 55u8, 152u8), Rect::new(-5, 8, 19, 8), 2, 5);
            s.fill_rect((143u8, 87u8, 29u8, 247u8), Rect::new(26, 15, 12, 22));
            s.set_clip(Some(Rect::new(5, 8, 9, 13)));
            let mut q = Surface::with_flags((28, 24), 0);
            q = muster_auf(q, 28, 24);
            q.set_alpha(Some(232));
            s.blit_ex(&q, (4, 18), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            let mut q = Surface::with_flags((23, 16), 0);
            q = muster_auf(q, 23, 16);
            s.blit_ex(&q, (19, 8), None, 0);
            s.fill_rect((29u8, 95u8, 203u8, 118u8), Rect::new(-12, -2, 7, 24));
            rustgame_core::draw::circle(&mut s, (126u8, 211u8, 213u8, 255u8), (9, 21), 14, 3);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x6E1DFF24A20B2820, flaeche: s }
    }));
    v.push((197, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((23, 6), SRCALPHA);
            s.fill((96u8, 60u8, 20u8, 255u8));
            let mut q = Surface::with_flags((5, 3), 0);
            q = muster_auf(q, 5, 3);
            s.blit_ex(&q, (17, 5), Some(Rect::new(6, 3, 7, 3)), rustgame_core::blend::flags::BLEND_RGBA_SUB);
            rustgame_core::draw::rect_radius(&mut s, (246u8, 152u8, 39u8, 255u8), Rect::new(3, 3, 7, 7), 3, 4);
            {
                let mut teil = s.subsurface(Rect::new(17, 4, 6, 2));
                teil.fill((255u8, 47u8, 184u8, 255u8));
            }
            let mut q = Surface::with_flags((24, 8), SRCALPHA);
            q = muster_auf(q, 24, 8);
            q.set_alpha(Some(216));
            s.blit_ex(&q, (18, 5), Some(Rect::new(-7, -2, 21, 5)), rustgame_core::blend::flags::BLEND_RGBA_MULT);
            rustgame_core::draw::circle(&mut s, (202u8, 126u8, 169u8, 255u8), (8, -2), 13, 4);
            rustgame_core::draw::circle(&mut s, (29u8, 199u8, 74u8, 218u8), (6, -2), 4, 3);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x40F839A60123B463, flaeche: s }
    }));
    v.push((198, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 24), 0);
            s.fill((51u8, 244u8, 233u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (93u8, 196u8, 15u8, 255u8), Rect::new(5, 19, 8, 18), 4, 4);
            {
                let mut teil = s.subsurface(Rect::new(1, 19, 5, 1));
                teil.fill((113u8, 200u8, 99u8, 28u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x3C45C8299F224F61, flaeche: s }
    }));
    v.push((199, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((17, 29), SRCALPHA);
            s.fill((158u8, 5u8, 244u8, 255u8));
            let mut q = Surface::with_flags((10, 9), 0);
            q = muster_auf(q, 10, 9);
            q.set_alpha(Some(193));
            q.set_colorkey(Some((201u8, 252u8, 119u8)));
            s.blit_ex(&q, (11, 5), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            s.scroll(7, 24);
            rustgame_core::draw::circle(&mut s, (195u8, 97u8, 153u8, 9u8), (4, 14), 2, 0);
            rustgame_core::draw::polygon(&mut s, (27u8, 49u8, 21u8, 219u8), &[(13, 26), (14, 0), (20, -4), (20, 16)], 1);
            s.set_clip(Some(Rect::new(2, 24, 14, 4)));
            s.fill_rect((198u8, 131u8, 74u8, 255u8), Rect::new(7, 9, 12, 27));
            let mut q = Surface::with_flags((16, 3), 0);
            q = muster_auf(q, 16, 3);
            s.blit_ex(&q, (5, 15), Some(Rect::new(2, -2, 11, 5)), rustgame_core::blend::flags::BLEND_RGB_MIN);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x391AB8557A1FBDB4, flaeche: s }
    }));
    v.push((200, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 12), 0);
            s.fill((15u8, 73u8, 136u8, 255u8));
            rustgame_core::draw::circle(&mut s, (224u8, 142u8, 224u8, 8u8), (3, 3), 3, 0);
            s.scroll(4, 0);
            s = rustgame_core::transform::scale(&s, (8, 9));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x9EAFF8435515B8CD, flaeche: s }
    }));
    v.push((201, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 28), SRCALPHA);
            s.fill((144u8, 167u8, 240u8, 255u8));
            rustgame_core::draw::circle(&mut s, (71u8, 244u8, 31u8, 242u8), (3, 9), 22, 1);
            s.fill_rect((136u8, 92u8, 246u8, 255u8), Rect::new(6, -13, 9, 4));
            rustgame_core::draw::circle(&mut s, (248u8, 48u8, 159u8, 150u8), (7, 31), 12, 1);
            s.fill_rect((214u8, 138u8, 6u8, 20u8), Rect::new(-1, 12, 2, 15));
            let mut q = Surface::with_flags((6, 23), 0);
            q = muster_auf(q, 6, 23);
            q.set_alpha(Some(137));
            s.blit_ex(&q, (2, 15), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            let mut q = Surface::with_flags((6, 20), 0);
            q = muster_auf(q, 6, 20);
            q.set_colorkey(Some((91u8, 39u8, 104u8)));
            s.blit_ex(&q, (-3, 30), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
            s = rustgame_core::transform::rotate(&s, 127.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x48B20092B43C97C1, flaeche: s }
    }));
    v.push((202, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((30, 18), 0);
            s.fill((209u8, 101u8, 100u8, 255u8));
            let mut q = Surface::with_flags((3, 15), SRCALPHA);
            q = muster_auf(q, 3, 15);
            s.blit_ex(&q, (16, 2), Some(Rect::new(3, 11, 6, 16)), rustgame_core::blend::flags::BLEND_RGB_MULT);
            rustgame_core::draw::line(&mut s, (65u8, 217u8, 238u8, 255u8), (-2, 12), (11, 20), 3);
            let mut q = Surface::with_flags((29, 17), SRCALPHA);
            q = muster_auf(q, 29, 17);
            q.set_alpha(Some(90));
            s.blit_ex(&q, (2, -3), None, 0);
            s.set_clip(Some(Rect::new(29, 6, 1, 6)));
            s.set_clip(Some(Rect::new(7, 6, 14, 4)));
            s.scroll(11, 4);
            {
                let mut teil = s.subsurface(Rect::new(9, 10, 15, 4));
                teil.fill((99u8, 196u8, 200u8, 29u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x080688C397ED945A, flaeche: s }
    }));
    v.push((203, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 7), 0);
            s.fill((108u8, 14u8, 133u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(0, 6, 4, 1));
                teil.fill((60u8, 52u8, 179u8, 67u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (103u8, 252u8, 53u8, 227u8), Rect::new(0, -3, 7, 9), 3, 3);
            rustgame_core::draw::rect_radius(&mut s, (193u8, 71u8, 246u8, 255u8), Rect::new(4, 1, 1, 10), 3, 3);
            rustgame_core::draw::circle(&mut s, (26u8, 117u8, 89u8, 165u8), (-3, 6), 2, 0);
            rustgame_core::draw::circle(&mut s, (207u8, 110u8, 132u8, 255u8), (4, 8), 4, 3);
            rustgame_core::draw::circle(&mut s, (93u8, 214u8, 16u8, 23u8), (11, 4), 0, 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x392A391216684065, flaeche: s }
    }));
    v.push((204, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((18, 8), 0);
            s.fill((107u8, 67u8, 210u8, 255u8));
            rustgame_core::draw::line(&mut s, (80u8, 167u8, 211u8, 255u8), (-2, 10), (-2, 10), 5);
            rustgame_core::draw::line(&mut s, (101u8, 116u8, 150u8, 77u8), (3, 2), (1, 5), 4);
            let mut q = Surface::with_flags((12, 6), SRCALPHA);
            q = muster_auf(q, 12, 6);
            s.blit_ex(&q, (-3, 0), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
            let mut q = Surface::with_flags((13, 10), SRCALPHA);
            q = muster_auf(q, 13, 10);
            s.blit_ex(&q, (10, 5), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
            rustgame_core::draw::circle(&mut s, (228u8, 201u8, 202u8, 75u8), (21, 7), 13, 0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x673205D434CC2F19, flaeche: s }
    }));
    v.push((205, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((19, 21), 0);
            s.fill((254u8, 239u8, 115u8, 255u8));
            let mut q = Surface::with_flags((5, 8), 0);
            q = muster_auf(q, 5, 8);
            q.set_alpha(Some(103));
            s.blit_ex(&q, (19, 6), None, 0);
            let mut q = Surface::with_flags((12, 22), 0);
            q = muster_auf(q, 12, 22);
            q.set_colorkey(Some((41u8, 149u8, 114u8)));
            s.blit_ex(&q, (5, -4), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            rustgame_core::draw::line(&mut s, (153u8, 186u8, 59u8, 255u8), (11, 18), (5, 18), 3);
            rustgame_core::draw::ellipse(&mut s, (105u8, 52u8, 74u8, 37u8), Rect::new(15, 21, 21, 23), 2);
            rustgame_core::draw::polygon(&mut s, (55u8, 173u8, 122u8, 2u8), &[(13, 20), (19, 16), (4, -2), (3, 11), (12, 20)], 2);
            rustgame_core::draw::ellipse(&mut s, (248u8, 239u8, 35u8, 84u8), Rect::new(4, -6, 8, 14), 2);
            let mut q = Surface::with_flags((15, 10), SRCALPHA);
            q = muster_auf(q, 15, 10);
            q.set_alpha(Some(78));
            s.blit_ex(&q, (6, 3), Some(Rect::new(11, -3, 5, 0)), rustgame_core::blend::flags::BLEND_RGBA_MULT);
            rustgame_core::draw::circle(&mut s, (171u8, 135u8, 165u8, 101u8), (12, 7), 12, 0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xA13D26D9C5A3CA27, flaeche: s }
    }));
    v.push((206, true, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((6, 8), SRCALPHA);
            s.fill((112u8, 126u8, 94u8, 65u8));
            {
                let mut teil = s.subsurface(Rect::new(4, 2, 1, 2));
                teil.fill((221u8, 156u8, 248u8, 140u8));
            }
            let mut q = Surface::with_flags((7, 10), 0);
            q = muster_auf(q, 7, 10);
            q.set_alpha(Some(176));
            s.blit_ex(&q, (6, -1), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            {
                let mut teil = s.subsurface(Rect::new(3, 3, 1, 1));
                teil.fill((33u8, 35u8, 23u8, 48u8));
            }
            s.set_clip(Some(Rect::new(2, 6, 2, 1)));
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x69239DC882B0AB1F, flaeche: s }
    }));
    v.push((207, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((22, 27), 0);
            s.fill((113u8, 81u8, 12u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (30u8, 161u8, 187u8, 43u8), &[(0, 7), (-2, 16), (14, 25), (2, 20)], 3);
            rustgame_core::draw::ellipse(&mut s, (173u8, 21u8, 90u8, 53u8), Rect::new(-5, 7, 24, 27), 2);
            let mut q = Surface::with_flags((6, 14), SRCALPHA);
            q = muster_auf(q, 6, 14);
            q.set_alpha(Some(235));
            s.blit_ex(&q, (7, 19), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            rustgame_core::draw::ellipse(&mut s, (106u8, 116u8, 126u8, 255u8), Rect::new(21, 18, 9, 16), 4);
            rustgame_core::draw::rect_radius(&mut s, (255u8, 101u8, 102u8, 255u8), Rect::new(-1, -13, 18, 8), 4, 4);
            s.set_clip(Some(Rect::new(10, 16, 9, 2)));
            let mut q = Surface::with_flags((15, 22), 0);
            q = muster_auf(q, 15, 22);
            s.blit_ex(&q, (16, 0), Some(Rect::new(3, -10, 12, 7)), rustgame_core::blend::flags::BLEND_RGBA_MULT);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x7838234DC57C3DA3, flaeche: s }
    }));
    v.push((208, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((14, 22), 0);
            s.fill((28u8, 130u8, 234u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (176u8, 215u8, 90u8, 235u8), Rect::new(-3, 3, 9, 21), 4);
            rustgame_core::draw::line(&mut s, (193u8, 133u8, 105u8, 155u8), (3, 15), (13, 0), 5);
            let mut q = Surface::with_flags((2, 20), SRCALPHA);
            q = muster_auf(q, 2, 20);
            s.blit_ex(&q, (16, 19), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            rustgame_core::draw::ellipse(&mut s, (15u8, 14u8, 148u8, 255u8), Rect::new(11, -9, 6, 23), 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x2C1D2ED2BCCF80E2, flaeche: s }
    }));
    v.push((209, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((23, 27), SRCALPHA);
            s.fill((46u8, 93u8, 39u8, 146u8));
            s.set_clip(Some(Rect::new(7, 13, 11, 3)));
            {
                let mut teil = s.subsurface(Rect::new(2, 13, 16, 14));
                teil.fill((14u8, 68u8, 116u8, 255u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (144u8, 64u8, 21u8, 51u8), Rect::new(6, -8, 26, 24), 1, 5);
            {
                let mut teil = s.subsurface(Rect::new(11, 18, 3, 7));
                teil.fill((48u8, 140u8, 235u8, 255u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xC81623E5F10E18B3, flaeche: s }
    }));
    v.push((210, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((18, 6), 0);
            s.fill((207u8, 86u8, 10u8, 255u8));
            s.scroll(-16, 4);
            s.set_clip(Some(Rect::new(17, 5, 1, 1)));
            rustgame_core::draw::rect_radius(&mut s, (71u8, 174u8, 32u8, 211u8), Rect::new(-7, 1, 19, 4), 2, 0);
            s.scroll(2, 2);
            s.fill_rect((116u8, 6u8, 9u8, 134u8), Rect::new(-2, 0, 9, 7));
            rustgame_core::draw::polygon(&mut s, (175u8, 106u8, 32u8, 255u8), &[(18, 9), (13, -2), (7, 2), (12, -2), (8, -2), (6, 4)], 1);
            let mut q = Surface::with_flags((3, 6), SRCALPHA);
            q = muster_auf(q, 3, 6);
            q.set_alpha(Some(21));
            s.blit_ex(&q, (0, 0), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x9C5D6663AADB63ED, flaeche: s }
    }));
    v.push((211, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((24, 17), 0);
            s.fill((10u8, 160u8, 175u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (213u8, 217u8, 91u8, 67u8), Rect::new(11, -4, 22, 20), 4);
            rustgame_core::draw::circle(&mut s, (125u8, 228u8, 65u8, 182u8), (11, 1), 22, 3);
            rustgame_core::draw::rect_radius(&mut s, (196u8, 170u8, 109u8, 32u8), Rect::new(-8, -2, 12, 15), 3, 0);
            rustgame_core::draw::rect_radius(&mut s, (1u8, 147u8, 98u8, 255u8), Rect::new(5, 17, 7, 18), 3, 3);
            rustgame_core::draw::ellipse(&mut s, (221u8, 173u8, 244u8, 140u8), Rect::new(3, 0, 8, 5), 2);
            rustgame_core::draw::circle(&mut s, (254u8, 124u8, 114u8, 35u8), (17, -2), 15, 0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x4FFFB3564B729B8A, flaeche: s }
    }));
    v.push((212, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((18, 25), 0);
            s.fill((30u8, 153u8, 66u8, 255u8));
            rustgame_core::draw::line(&mut s, (94u8, 164u8, 69u8, 255u8), (20, 1), (9, 14), 4);
            s.fill_rect((17u8, 238u8, 147u8, 82u8), Rect::new(-8, 4, 6, 10));
            let mut q = Surface::with_flags((18, 16), SRCALPHA);
            q = muster_auf(q, 18, 16);
            s.blit_ex(&q, (1, 12), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xFCD77423E26F5B04, flaeche: s }
    }));
    v.push((213, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((10, 19), SRCALPHA);
            s.fill((213u8, 158u8, 81u8, 255u8));
            let mut q = Surface::with_flags((5, 21), SRCALPHA);
            q = muster_auf(q, 5, 21);
            s.blit_ex(&q, (2, 21), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            rustgame_core::draw::rect_radius(&mut s, (55u8, 136u8, 160u8, 255u8), Rect::new(1, 8, 7, 17), 2, 0);
            let mut q = Surface::with_flags((7, 14), 0);
            q = muster_auf(q, 7, 14);
            s.blit_ex(&q, (4, -1), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            rustgame_core::draw::circle(&mut s, (139u8, 38u8, 73u8, 131u8), (-1, 15), 5, 2);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x4BEC415BD640C82D, flaeche: s }
    }));
    v.push((214, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((22, 29), SRCALPHA);
            s.fill((225u8, 112u8, 209u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(7, 2, 11, 6));
                teil.fill((25u8, 116u8, 29u8, 255u8));
            }
            rustgame_core::draw::circle(&mut s, (188u8, 100u8, 21u8, 175u8), (9, 27), 14, 0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xE683B406C9DE61E5, flaeche: s }
    }));
    v.push((215, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 26), 0);
            s.fill((50u8, 146u8, 121u8, 255u8));
            rustgame_core::draw::circle(&mut s, (15u8, 58u8, 92u8, 151u8), (0, 3), 16, 3);
            rustgame_core::draw::ellipse(&mut s, (226u8, 210u8, 114u8, 78u8), Rect::new(2, -5, 10, 8), 4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x72B7098B5D94C346, flaeche: s }
    }));
    v.push((216, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 26), SRCALPHA);
            s.fill((193u8, 55u8, 12u8, 9u8));
            let mut q = Surface::with_flags((13, 19), SRCALPHA);
            q = muster_auf(q, 13, 19);
            q.set_alpha(Some(216));
            s.blit_ex(&q, (10, 4), Some(Rect::new(8, 20, 1, 19)), 0);
            rustgame_core::draw::circle(&mut s, (137u8, 147u8, 198u8, 255u8), (11, 3), 23, 2);
            s.scroll(-7, -8);
            rustgame_core::draw::polygon(&mut s, (131u8, 111u8, 234u8, 255u8), &[(7, 28), (8, 15), (-4, 6), (15, 22), (13, 18), (-2, 1)], 1);
            s = rustgame_core::transform::flip(&s, true, true);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x21D058275B8C2647, flaeche: s }
    }));
    v.push((217, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((15, 13), SRCALPHA);
            s.fill((174u8, 76u8, 137u8, 62u8));
            s.scroll(9, 0);
            rustgame_core::draw::rect_radius(&mut s, (47u8, 42u8, 147u8, 108u8), Rect::new(-6, -6, 17, 11), 0, 3);
            s.scroll(-2, -8);
            let mut q = Surface::with_flags((14, 14), 0);
            q = muster_auf(q, 14, 14);
            s.blit_ex(&q, (8, 14), Some(Rect::new(6, 14, 0, 10)), rustgame_core::blend::flags::BLEND_RGBA_SUB);
            s.set_clip(Some(Rect::new(0, 9, 6, 4)));
            let mut q = Surface::with_flags((17, 13), SRCALPHA);
            q = muster_auf(q, 17, 13);
            s.blit_ex(&q, (18, 16), Some(Rect::new(-3, 2, 19, 10)), rustgame_core::blend::flags::BLEND_RGBA_MAX);
            s.set_clip(Some(Rect::new(4, 6, 8, 4)));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xF5B2303F1E6A7720, flaeche: s }
    }));
    v.push((218, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((15, 11), SRCALPHA);
            s.fill((168u8, 105u8, 25u8, 178u8));
            rustgame_core::draw::circle(&mut s, (100u8, 181u8, 169u8, 255u8), (14, 3), 3, 4);
            rustgame_core::draw::ellipse(&mut s, (127u8, 245u8, 255u8, 96u8), Rect::new(-4, -2, 4, 2), 3);
            s.fill_rect((142u8, 60u8, 233u8, 131u8), Rect::new(-3, 3, 1, 2));
            {
                let mut teil = s.subsurface(Rect::new(11, 2, 2, 4));
                teil.fill((254u8, 30u8, 201u8, 221u8));
            }
            {
                let mut teil = s.subsurface(Rect::new(0, 1, 3, 2));
                teil.fill((106u8, 20u8, 198u8, 185u8));
            }
            s.set_clip(Some(Rect::new(2, 0, 13, 8)));
            s = s.convert_alpha();
            s = rustgame_core::transform::rotate(&s, -313.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xF3D1699143DD8F6A, flaeche: s }
    }));
    v.push((219, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((28, 26), SRCALPHA);
            s.fill((52u8, 146u8, 223u8, 255u8));
            s.scroll(2, -24);
            s.set_clip(Some(Rect::new(20, 25, 8, 1)));
            rustgame_core::draw::rect_radius(&mut s, (20u8, 137u8, 168u8, 67u8), Rect::new(11, 7, 4, 26), 4, 0);
            let mut q = Surface::with_flags((24, 24), 0);
            q = muster_auf(q, 24, 24);
            s.blit_ex(&q, (-3, -1), Some(Rect::new(7, 22, 13, 20)), 0);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x3B9516BB0E364925, flaeche: s }
    }));
    v.push((220, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((29, 30), SRCALPHA);
            s.fill((141u8, 205u8, 212u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(10, 7, 16, 6));
                teil.fill((197u8, 99u8, 12u8, 255u8));
            }
            rustgame_core::draw::rect(&mut s, (30u8, 157u8, 154u8, 105u8), Rect::new(-15, 5, 4, 5), 1);
            rustgame_core::draw::circle(&mut s, (165u8, 166u8, 42u8, 255u8), (29, 29), 5, 1);
            s.fill_rect((203u8, 191u8, 250u8, 33u8), Rect::new(15, 2, 27, 10));
            let mut q = Surface::with_flags((26, 26), 0);
            q = muster_auf(q, 26, 26);
            q.set_alpha(Some(201));
            s.blit_ex(&q, (0, 24), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x960BA8821DEB8630, flaeche: s }
    }));
    v.push((221, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((13, 19), SRCALPHA);
            s.fill((118u8, 181u8, 231u8, 255u8));
            rustgame_core::draw::circle(&mut s, (139u8, 42u8, 131u8, 255u8), (16, 1), 13, 3);
            rustgame_core::draw::circle(&mut s, (44u8, 178u8, 53u8, 255u8), (4, 14), 1, 4);
            s.set_clip(Some(Rect::new(8, 13, 1, 4)));
            rustgame_core::draw::ellipse(&mut s, (200u8, 46u8, 150u8, 135u8), Rect::new(-7, -2, 4, 6), 1);
            s.fill_rect((195u8, 200u8, 93u8, 32u8), Rect::new(9, 19, 2, 16));
            rustgame_core::draw::polygon(&mut s, (115u8, 94u8, 182u8, 27u8), &[(9, -2), (3, 16), (13, 20), (2, 1), (9, 11), (3, -4)], 2);
            s.set_clip(Some(Rect::new(0, 9, 7, 1)));
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x2338D248D8CA9B32, flaeche: s }
    }));
    v.push((222, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((30, 9), SRCALPHA);
            s.fill((14u8, 246u8, 114u8, 255u8));
            rustgame_core::draw::circle(&mut s, (8u8, 52u8, 214u8, 38u8), (0, 11), 21, 4);
            {
                let mut teil = s.subsurface(Rect::new(22, 3, 6, 4));
                teil.fill((118u8, 32u8, 140u8, 100u8));
            }
            let mut q = Surface::with_flags((8, 10), SRCALPHA);
            q = muster_auf(q, 8, 10);
            s.blit_ex(&q, (14, 6), Some(Rect::new(6, 0, 9, 11)), rustgame_core::blend::flags::BLEND_RGBA_MULT);
            {
                let mut teil = s.subsurface(Rect::new(11, 3, 6, 5));
                teil.fill((188u8, 40u8, 45u8, 102u8));
            }
            {
                let mut teil = s.subsurface(Rect::new(11, 4, 1, 3));
                teil.fill((87u8, 186u8, 206u8, 255u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xB988A94933392659, flaeche: s }
    }));
    v.push((223, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((16, 17), SRCALPHA);
            s.fill((61u8, 1u8, 174u8, 255u8));
            s.fill_rect((181u8, 183u8, 142u8, 44u8), Rect::new(4, 2, 2, 5));
            rustgame_core::draw::polygon(&mut s, (141u8, 134u8, 96u8, 15u8), &[(-2, 16), (-1, 12), (7, 11), (19, -3), (14, 7), (18, 13)], 0);
            rustgame_core::draw::ellipse(&mut s, (198u8, 76u8, 173u8, 255u8), Rect::new(1, 9, 8, 15), 4);
            rustgame_core::draw::rect_radius(&mut s, (96u8, 4u8, 18u8, 234u8), Rect::new(11, 12, 6, 2), 1, 1);
            rustgame_core::draw::line(&mut s, (224u8, 85u8, 4u8, 249u8), (14, 0), (12, 6), 2);
            {
                let mut teil = s.subsurface(Rect::new(13, 5, 3, 12));
                teil.fill((5u8, 73u8, 76u8, 255u8));
            }
            rustgame_core::draw::polygon(&mut s, (4u8, 116u8, 76u8, 199u8), &[(1, 7), (15, 1), (17, 12), (11, 20), (14, 11)], 0);
            s = rustgame_core::transform::scale(&s, (18, 9));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x808A36650BFA50DF, flaeche: s }
    }));
    v.push((224, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 12), SRCALPHA);
            s.fill((51u8, 218u8, 67u8, 116u8));
            rustgame_core::draw::polygon(&mut s, (65u8, 214u8, 180u8, 53u8), &[(2, 9), (4, 8), (8, 5), (-1, 1), (9, 13)], 3);
            rustgame_core::draw::ellipse(&mut s, (46u8, 27u8, 26u8, 149u8), Rect::new(8, -5, 7, 12), 2);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xC824BACA3FF7A9D3, flaeche: s }
    }));
    v.push((225, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((28, 13), SRCALPHA);
            s.fill((97u8, 188u8, 72u8, 69u8));
            rustgame_core::draw::rect(&mut s, (123u8, 109u8, 149u8, 141u8), Rect::new(4, -4, 27, 14), 1);
            s.scroll(-2, 8);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xA8C2819DD66FB3D9, flaeche: s }
    }));
    v.push((226, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 7), 0);
            s.fill((25u8, 88u8, 188u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (177u8, 45u8, 180u8, 233u8), &[(23, 4), (-4, -2), (18, 4), (7, 1), (9, 4), (-2, 2)], 1);
            let mut q = Surface::with_flags((3, 2), 0);
            q = muster_auf(q, 3, 2);
            s.blit_ex(&q, (4, 2), Some(Rect::new(-1, -1, 4, 5)), rustgame_core::blend::flags::BLEND_RGBA_SUB);
            let mut q = Surface::with_flags((5, 5), SRCALPHA);
            q = muster_auf(q, 5, 5);
            q.set_alpha(Some(46));
            s.blit_ex(&q, (-1, 8), Some(Rect::new(4, 1, 2, 2)), rustgame_core::blend::flags::BLEND_RGB_SUB);
            rustgame_core::draw::ellipse(&mut s, (52u8, 14u8, 170u8, 208u8), Rect::new(-1, 6, 16, 3), 4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x7825163EAE0258C8, flaeche: s }
    }));
    v.push((227, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((18, 32), 0);
            s.fill((189u8, 41u8, 228u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(13, 26, 1, 1));
                teil.fill((123u8, 36u8, 120u8, 255u8));
            }
            let mut q = Surface::with_flags((2, 17), 0);
            q = muster_auf(q, 2, 17);
            q.set_alpha(Some(233));
            s.blit_ex(&q, (21, 25), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            s.scroll(10, 30);
            s.set_clip(Some(Rect::new(3, 9, 6, 1)));
            rustgame_core::draw::polygon(&mut s, (48u8, 7u8, 161u8, 184u8), &[(-2, 23), (21, 30), (-1, 28), (-2, 13), (12, -2), (10, 29)], 2);
            let mut q = Surface::with_flags((14, 24), 0);
            q = muster_auf(q, 14, 24);
            q.set_alpha(Some(3));
            q.set_colorkey(Some((183u8, 61u8, 201u8)));
            s.blit_ex(&q, (13, 29), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            {
                let mut teil = s.subsurface(Rect::new(2, 22, 8, 2));
                teil.fill((97u8, 247u8, 101u8, 123u8));
            }
            s.set_clip(Some(Rect::new(5, 0, 12, 16)));
            s = rustgame_core::transform::flip(&s, false, true);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xA9E33F8AE856C9A2, flaeche: s }
    }));
    v.push((228, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((30, 28), 0);
            s.fill((115u8, 112u8, 239u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (39u8, 66u8, 156u8, 75u8), &[(22, 15), (19, 21), (32, 8), (32, 1), (32, 29), (23, 27)], 0);
            rustgame_core::draw::circle(&mut s, (130u8, 210u8, 124u8, 174u8), (21, 17), 14, 2);
            s.scroll(-17, -23);
            s.scroll(-22, 5);
            rustgame_core::draw::ellipse(&mut s, (189u8, 190u8, 82u8, 152u8), Rect::new(-15, -9, 30, 1), 0);
            rustgame_core::draw::rect_radius(&mut s, (42u8, 148u8, 159u8, 127u8), Rect::new(-8, 2, 6, 8), 4, 3);
            s = rustgame_core::transform::scale(&s, (21, 9));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xAECC71754018ACC9, flaeche: s }
    }));
    v.push((229, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((22, 10), SRCALPHA);
            s.fill((107u8, 87u8, 252u8, 135u8));
            s.scroll(10, -10);
            rustgame_core::draw::ellipse(&mut s, (120u8, 39u8, 161u8, 89u8), Rect::new(6, -4, 0, 3), 3);
            rustgame_core::draw::circle(&mut s, (155u8, 116u8, 239u8, 137u8), (24, 7), 20, 3);
            rustgame_core::draw::line(&mut s, (227u8, 153u8, 12u8, 255u8), (2, 12), (17, 7), 4);
            s.set_clip(Some(Rect::new(17, 8, 4, 2)));
            rustgame_core::draw::polygon(&mut s, (116u8, 22u8, 153u8, 129u8), &[(9, -1), (3, 9), (8, -2), (25, 1), (13, 5), (14, 4)], 2);
            let mut q = Surface::with_flags((20, 2), SRCALPHA);
            q = muster_auf(q, 20, 2);
            s.blit_ex(&q, (5, 3), Some(Rect::new(-9, 2, 12, 0)), rustgame_core::blend::flags::BLEND_RGB_SUB);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xE6EDECBFD198D315, flaeche: s }
    }));
    v.push((230, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((28, 11), SRCALPHA);
            s.fill((222u8, 13u8, 249u8, 8u8));
            rustgame_core::draw::line(&mut s, (130u8, 0u8, 179u8, 208u8), (8, -4), (16, 5), 2);
            s.set_clip(Some(Rect::new(15, 9, 7, 2)));
            s.scroll(-26, -11);
            rustgame_core::draw::polygon(&mut s, (154u8, 115u8, 127u8, 249u8), &[(25, 8), (3, 9), (24, 10), (16, -4), (12, 4)], 2);
            s.set_clip(Some(Rect::new(13, 3, 14, 2)));
            s.set_clip(Some(Rect::new(5, 10, 18, 1)));
            s.set_clip(Some(Rect::new(20, 1, 4, 5)));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x013580AB28A7CD11, flaeche: s }
    }));
    v.push((231, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((6, 10), 0);
            s.fill((163u8, 118u8, 105u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (233u8, 167u8, 150u8, 212u8), Rect::new(7, 8, 1, 9), 0);
            s.fill_rect((176u8, 136u8, 140u8, 255u8), Rect::new(2, -5, 0, 10));
            rustgame_core::draw::ellipse(&mut s, (231u8, 20u8, 26u8, 31u8), Rect::new(7, 11, 1, 12), 1);
            rustgame_core::draw::circle(&mut s, (184u8, 26u8, 127u8, 255u8), (2, -1), 5, 4);
            rustgame_core::draw::circle(&mut s, (5u8, 161u8, 165u8, 255u8), (2, -2), 2, 3);
            let mut q = Surface::with_flags((3, 7), 0);
            q = muster_auf(q, 3, 7);
            q.set_colorkey(Some((156u8, 63u8, 252u8)));
            s.blit_ex(&q, (-4, -4), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            rustgame_core::draw::line(&mut s, (209u8, 169u8, 50u8, 255u8), (-2, 10), (-4, 3), 3);
            rustgame_core::draw::circle(&mut s, (99u8, 251u8, 25u8, 162u8), (0, 1), 8, 2);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xC9E40FFC1134779F, flaeche: s }
    }));
    v.push((232, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((14, 9), SRCALPHA);
            s.fill((139u8, 143u8, 82u8, 255u8));
            s.set_clip(Some(Rect::new(1, 5, 5, 3)));
            let mut q = Surface::with_flags((3, 5), SRCALPHA);
            q = muster_auf(q, 3, 5);
            s.blit_ex(&q, (1, 9), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            rustgame_core::draw::ellipse(&mut s, (165u8, 88u8, 3u8, 255u8), Rect::new(7, 6, 12, 2), 4);
            let mut q = Surface::with_flags((2, 5), SRCALPHA);
            q = muster_auf(q, 2, 5);
            q.set_alpha(Some(178));
            q.set_colorkey(Some((9u8, 170u8, 174u8)));
            s.blit_ex(&q, (2, -1), None, 0);
            s.scroll(-11, 8);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x5359E1ECF5A5FDDD, flaeche: s }
    }));
    v.push((233, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((27, 31), 0);
            s.fill((118u8, 247u8, 104u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (7u8, 39u8, 121u8, 255u8), Rect::new(24, 29, 22, 14), 2);
            s.set_clip(Some(Rect::new(10, 6, 3, 10)));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x885325D0A1FD6EEB, flaeche: s }
    }));
    v.push((234, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((16, 14), 0);
            s.fill((210u8, 246u8, 117u8, 255u8));
            let mut q = Surface::with_flags((17, 11), 0);
            q = muster_auf(q, 17, 11);
            q.set_alpha(Some(199));
            s.blit_ex(&q, (-4, 8), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            {
                let mut teil = s.subsurface(Rect::new(9, 11, 1, 3));
                teil.fill((56u8, 58u8, 154u8, 255u8));
            }
            {
                let mut teil = s.subsurface(Rect::new(7, 6, 3, 7));
                teil.fill((145u8, 119u8, 188u8, 2u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (156u8, 1u8, 180u8, 103u8), Rect::new(-6, -4, 4, 4), 0, 3);
            s = s.convert_alpha();
            s = rustgame_core::transform::scale(&s, (7, 3));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x102FC235E8C10CD7, flaeche: s }
    }));
    v.push((235, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((17, 29), 0);
            s.fill((19u8, 127u8, 38u8, 255u8));
            s.fill_rect((60u8, 31u8, 239u8, 28u8), Rect::new(-4, -4, 10, 27));
            s.scroll(-9, 19);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xCCC3D8B12B19215C, flaeche: s }
    }));
    v.push((236, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((27, 17), SRCALPHA);
            s.fill((195u8, 78u8, 154u8, 40u8));
            s.set_clip(Some(Rect::new(22, 8, 3, 6)));
            let mut q = Surface::with_flags((8, 8), 0);
            q = muster_auf(q, 8, 8);
            q.set_colorkey(Some((25u8, 26u8, 49u8)));
            s.blit_ex(&q, (17, -2), Some(Rect::new(-3, 2, 11, 4)), rustgame_core::blend::flags::BLEND_RGB_MAX);
            rustgame_core::draw::polygon(&mut s, (5u8, 85u8, 64u8, 255u8), &[(27, 1), (17, 9), (26, 8), (19, 18), (29, 17), (4, -1)], 0);
            rustgame_core::draw::ellipse(&mut s, (135u8, 144u8, 23u8, 255u8), Rect::new(-6, 14, 15, 13), 4);
            let mut q = Surface::with_flags((11, 16), SRCALPHA);
            q = muster_auf(q, 11, 16);
            q.set_alpha(Some(47));
            s.blit_ex(&q, (26, 19), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            rustgame_core::draw::circle(&mut s, (172u8, 226u8, 193u8, 231u8), (0, 2), 23, 4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x4AF8EB81D033FE02, flaeche: s }
    }));
    v.push((237, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((25, 21), 0);
            s.fill((121u8, 213u8, 31u8, 255u8));
            s.set_clip(Some(Rect::new(23, 20, 2, 1)));
            let mut q = Surface::with_flags((4, 21), 0);
            q = muster_auf(q, 4, 21);
            q.set_alpha(Some(134));
            q.set_colorkey(Some((247u8, 251u8, 219u8)));
            s.blit_ex(&q, (1, -4), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            rustgame_core::draw::circle(&mut s, (74u8, 141u8, 115u8, 255u8), (23, 10), 11, 1);
            rustgame_core::draw::rect_radius(&mut s, (190u8, 199u8, 202u8, 227u8), Rect::new(25, -5, 20, 12), 3, 1);
            s.set_clip(Some(Rect::new(5, 16, 8, 2)));
            s.set_clip(Some(Rect::new(6, 12, 3, 3)));
            rustgame_core::draw::ellipse(&mut s, (173u8, 26u8, 54u8, 255u8), Rect::new(-1, 5, 24, 9), 4);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xBCCC7C222475E0AD, flaeche: s }
    }));
    v.push((238, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((31, 17), SRCALPHA);
            s.fill((45u8, 74u8, 85u8, 33u8));
            rustgame_core::draw::rect_radius(&mut s, (43u8, 86u8, 157u8, 21u8), Rect::new(2, 4, 1, 13), 2, 3);
            rustgame_core::draw::ellipse(&mut s, (214u8, 50u8, 123u8, 124u8), Rect::new(10, -9, 10, 20), 4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x05AE25A6C22FDF32, flaeche: s }
    }));
    v.push((239, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((9, 12), 0);
            s.fill((76u8, 213u8, 52u8, 255u8));
            s.fill_rect((6u8, 145u8, 132u8, 255u8), Rect::new(4, -2, 12, 0));
            rustgame_core::draw::rect_radius(&mut s, (28u8, 122u8, 137u8, 255u8), Rect::new(6, 9, 0, 13), 3, 6);
            rustgame_core::draw::line(&mut s, (203u8, 222u8, 90u8, 173u8), (2, 3), (0, 6), 1);
            s.fill_rect((105u8, 207u8, 155u8, 65u8), Rect::new(10, -3, 7, 15));
            {
                let mut teil = s.subsurface(Rect::new(7, 6, 2, 3));
                teil.fill((173u8, 78u8, 140u8, 243u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x03AE2D7CE2DCDC21, flaeche: s }
    }));
    v.push((240, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((25, 7), SRCALPHA);
            s.fill((164u8, 18u8, 190u8, 74u8));
            {
                let mut teil = s.subsurface(Rect::new(24, 2, 1, 3));
                teil.fill((127u8, 126u8, 0u8, 83u8));
            }
            let mut q = Surface::with_flags((18, 8), 0);
            q = muster_auf(q, 18, 8);
            s.blit_ex(&q, (15, -4), Some(Rect::new(12, 1, 17, 2)), rustgame_core::blend::flags::BLEND_RGB_MAX);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xFF27CC444CCB9E2D, flaeche: s }
    }));
    v.push((241, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((28, 23), 0);
            s.fill((181u8, 32u8, 120u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (132u8, 136u8, 180u8, 84u8), Rect::new(-6, -12, 10, 22), 4, 2);
            let mut q = Surface::with_flags((18, 20), SRCALPHA);
            q = muster_auf(q, 18, 20);
            q.set_alpha(Some(27));
            s.blit_ex(&q, (2, 22), Some(Rect::new(-5, -1, 11, 5)), rustgame_core::blend::flags::BLEND_RGBA_MAX);
            rustgame_core::draw::rect_radius(&mut s, (16u8, 24u8, 196u8, 255u8), Rect::new(-11, -7, 0, 16), 2, 4);
            rustgame_core::draw::ellipse(&mut s, (224u8, 117u8, 246u8, 91u8), Rect::new(-14, 5, 12, 11), 3);
            {
                let mut teil = s.subsurface(Rect::new(14, 3, 1, 11));
                teil.fill((182u8, 130u8, 244u8, 197u8));
            }
            rustgame_core::draw::line(&mut s, (59u8, 105u8, 161u8, 255u8), (-3, 20), (22, 25), 4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xA2FBCE1BC50A238D, flaeche: s }
    }));
    v.push((242, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((16, 6), SRCALPHA);
            s.fill((87u8, 0u8, 103u8, 248u8));
            rustgame_core::draw::rect_radius(&mut s, (194u8, 20u8, 164u8, 255u8), Rect::new(5, 2, 7, 0), 0, 5);
            let mut q = Surface::with_flags((12, 4), SRCALPHA);
            q = muster_auf(q, 12, 4);
            q.set_alpha(Some(116));
            q.set_colorkey(Some((67u8, 76u8, 183u8)));
            s.blit_ex(&q, (13, 5), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            s.scroll(1, 2);
            {
                let mut teil = s.subsurface(Rect::new(5, 0, 2, 2));
                teil.fill((170u8, 164u8, 133u8, 255u8));
            }
            s = rustgame_core::transform::rotate(&s, 10.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xF061441375A48EDD, flaeche: s }
    }));
    v.push((243, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((22, 16), SRCALPHA);
            s.fill((85u8, 134u8, 62u8, 156u8));
            s.set_clip(Some(Rect::new(10, 6, 6, 10)));
            rustgame_core::draw::rect_radius(&mut s, (81u8, 146u8, 25u8, 255u8), Rect::new(0, 15, 7, 3), 3, 1);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x2F424789E00F18A5, flaeche: s }
    }));
    v.push((244, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((30, 31), 0);
            s.fill((209u8, 196u8, 238u8, 255u8));
            let mut q = Surface::with_flags((25, 11), 0);
            q = muster_auf(q, 25, 11);
            q.set_alpha(Some(141));
            s.blit_ex(&q, (4, 9), Some(Rect::new(14, 1, 3, 5)), 0);
            let mut q = Surface::with_flags((17, 8), SRCALPHA);
            q = muster_auf(q, 17, 8);
            s.blit_ex(&q, (21, 31), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            s = rustgame_core::transform::grayscale(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x62805DD1DE614285, flaeche: s }
    }));
    v.push((245, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((10, 15), SRCALPHA);
            s.fill((23u8, 109u8, 130u8, 237u8));
            rustgame_core::draw::ellipse(&mut s, (254u8, 205u8, 102u8, 157u8), Rect::new(-4, 14, 4, 5), 4);
            rustgame_core::draw::circle(&mut s, (243u8, 27u8, 253u8, 255u8), (2, 17), 9, 0);
            rustgame_core::draw::ellipse(&mut s, (28u8, 26u8, 182u8, 255u8), Rect::new(5, -7, 13, 12), 3);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x9B11DC6480ED1571, flaeche: s }
    }));
    v.push((246, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 22), SRCALPHA);
            s.fill((225u8, 90u8, 91u8, 255u8));
            rustgame_core::draw::line(&mut s, (176u8, 110u8, 144u8, 49u8), (3, 0), (-3, -3), 1);
            s.fill_rect((224u8, 83u8, 241u8, 28u8), Rect::new(3, -11, 7, 16));
            rustgame_core::draw::rect(&mut s, (79u8, 63u8, 75u8, 255u8), Rect::new(1, 2, 0, 4), 3);
            s.fill_rect((242u8, 222u8, 239u8, 255u8), Rect::new(-3, 18, 5, 13));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x406CE7CBE41C7B0D, flaeche: s }
    }));
    v.push((247, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((20, 16), SRCALPHA);
            s.fill((183u8, 70u8, 216u8, 26u8));
            s.set_clip(Some(Rect::new(9, 6, 10, 7)));
            rustgame_core::draw::rect_radius(&mut s, (105u8, 45u8, 50u8, 255u8), Rect::new(7, -7, 14, 4), 0, 4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x3561FD325908F025, flaeche: s }
    }));
    v.push((248, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((32, 19), 0);
            s.fill((244u8, 102u8, 236u8, 255u8));
            s.set_clip(Some(Rect::new(16, 18, 15, 1)));
            let mut q = Surface::with_flags((28, 21), SRCALPHA);
            q = muster_auf(q, 28, 21);
            q.set_alpha(Some(255));
            s.blit_ex(&q, (27, 17), Some(Rect::new(2, 6, 28, 18)), rustgame_core::blend::flags::BLEND_RGB_MULT);
            rustgame_core::draw::circle(&mut s, (234u8, 98u8, 234u8, 83u8), (32, 9), 26, 0);
            s.set_clip(Some(Rect::new(30, 6, 1, 9)));
            rustgame_core::draw::line(&mut s, (145u8, 69u8, 133u8, 255u8), (13, 7), (0, 2), 2);
            let mut q = Surface::with_flags((12, 15), SRCALPHA);
            q = muster_auf(q, 12, 15);
            q.set_alpha(Some(45));
            q.set_colorkey(Some((2u8, 178u8, 42u8)));
            s.blit_ex(&q, (9, 0), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            rustgame_core::draw::rect_radius(&mut s, (48u8, 161u8, 7u8, 184u8), Rect::new(20, 16, 12, 0), 3, 5);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xEE4208739FB4C081, flaeche: s }
    }));
    v.push((249, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((14, 24), 0);
            s.fill((142u8, 253u8, 80u8, 255u8));
            rustgame_core::draw::circle(&mut s, (181u8, 58u8, 79u8, 41u8), (10, 11), 4, 3);
            let mut q = Surface::with_flags((10, 13), SRCALPHA);
            q = muster_auf(q, 10, 13);
            s.blit_ex(&q, (17, 0), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x7F09077BCDB32EFD, flaeche: s }
    }));
    v.push((250, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((18, 18), 0);
            s.fill((34u8, 20u8, 123u8, 255u8));
            rustgame_core::draw::line(&mut s, (216u8, 236u8, 158u8, 134u8), (16, 14), (21, 19), 2);
            rustgame_core::draw::circle(&mut s, (97u8, 240u8, 141u8, 255u8), (2, 14), 13, 3);
            rustgame_core::draw::circle(&mut s, (243u8, 173u8, 110u8, 255u8), (2, 6), 14, 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xC041282678B15CE4, flaeche: s }
    }));
    v.push((251, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((20, 27), SRCALPHA);
            s.fill((126u8, 210u8, 250u8, 233u8));
            {
                let mut teil = s.subsurface(Rect::new(14, 12, 2, 15));
                teil.fill((159u8, 57u8, 227u8, 82u8));
            }
            let mut q = Surface::with_flags((19, 2), SRCALPHA);
            q = muster_auf(q, 19, 2);
            s.blit_ex(&q, (15, 9), Some(Rect::new(-4, 2, 5, 0)), rustgame_core::blend::flags::BLEND_RGB_MULT);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x9CC3BB234A136815, flaeche: s }
    }));
    v.push((252, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((30, 25), SRCALPHA);
            s.fill((99u8, 173u8, 143u8, 74u8));
            let mut q = Surface::with_flags((2, 26), 0);
            q = muster_auf(q, 2, 26);
            s.blit_ex(&q, (27, 1), None, 0);
            {
                let mut teil = s.subsurface(Rect::new(22, 8, 1, 5));
                teil.fill((137u8, 41u8, 122u8, 255u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (221u8, 108u8, 22u8, 255u8), Rect::new(-12, 21, 8, 20), 0, 3);
            let mut q = Surface::with_flags((9, 24), 0);
            q = muster_auf(q, 9, 24);
            q.set_alpha(Some(95));
            s.blit_ex(&q, (4, 28), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            rustgame_core::draw::line(&mut s, (74u8, 246u8, 80u8, 148u8), (-3, 3), (33, 26), 1);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x6348D7E7B2C2900E, flaeche: s }
    }));
    v.push((253, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((23, 23), SRCALPHA);
            s.fill((32u8, 228u8, 32u8, 116u8));
            {
                let mut teil = s.subsurface(Rect::new(10, 17, 12, 1));
                teil.fill((107u8, 20u8, 214u8, 255u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (253u8, 41u8, 144u8, 255u8), Rect::new(4, 0, 14, 20), 4, 1);
            {
                let mut teil = s.subsurface(Rect::new(8, 11, 15, 3));
                teil.fill((146u8, 206u8, 7u8, 255u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xAD2D07DEE363D317, flaeche: s }
    }));
    v.push((254, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((17, 22), SRCALPHA);
            s.fill((1u8, 166u8, 247u8, 111u8));
            rustgame_core::draw::rect_radius(&mut s, (10u8, 32u8, 108u8, 255u8), Rect::new(18, 4, 20, 19), 2, 0);
            rustgame_core::draw::polygon(&mut s, (55u8, 203u8, 58u8, 255u8), &[(16, 21), (18, 7), (4, 9)], 1);
            let mut q = Surface::with_flags((16, 7), 0);
            q = muster_auf(q, 16, 7);
            q.set_colorkey(Some((180u8, 244u8, 202u8)));
            s.blit_ex(&q, (-3, 18), Some(Rect::new(-8, 2, 6, 7)), rustgame_core::blend::flags::BLEND_RGBA_MULT);
            rustgame_core::draw::polygon(&mut s, (173u8, 48u8, 161u8, 88u8), &[(10, 0), (19, 21), (19, 13), (15, 15), (0, 2), (1, 12)], 1);
            s.set_clip(Some(Rect::new(11, 0, 5, 21)));
            s.scroll(-17, -3);
            rustgame_core::draw::polygon(&mut s, (59u8, 48u8, 42u8, 255u8), &[(10, 3), (14, -2), (1, 10)], 2);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xCAEDFDDE84759E71, flaeche: s }
    }));
    v.push((255, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 16), 0);
            s.fill((23u8, 190u8, 64u8, 255u8));
            rustgame_core::draw::circle(&mut s, (149u8, 227u8, 94u8, 196u8), (5, 10), 9, 3);
            rustgame_core::draw::ellipse(&mut s, (149u8, 103u8, 230u8, 180u8), Rect::new(4, 17, 5, 14), 3);
            s.set_clip(Some(Rect::new(1, 8, 5, 6)));
            rustgame_core::draw::rect_radius(&mut s, (220u8, 122u8, 32u8, 218u8), Rect::new(4, 4, 10, 17), 0, 3);
            {
                let mut teil = s.subsurface(Rect::new(3, 12, 2, 2));
                teil.fill((57u8, 113u8, 152u8, 147u8));
            }
            {
                let mut teil = s.subsurface(Rect::new(2, 13, 4, 2));
                teil.fill((181u8, 141u8, 24u8, 81u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x8A4FC27B76960D8D, flaeche: s }
    }));
    v.push((256, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 15), SRCALPHA);
            s.fill((20u8, 234u8, 221u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (163u8, 165u8, 168u8, 255u8), &[(15, -2), (22, 13), (5, 13), (1, 18)], 2);
            s.fill_rect((206u8, 110u8, 72u8, 17u8), Rect::new(-10, 12, 8, 0));
            {
                let mut teil = s.subsurface(Rect::new(13, 5, 6, 8));
                teil.fill((244u8, 39u8, 82u8, 255u8));
            }
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xF8A70B350D12C506, flaeche: s }
    }));
    v.push((257, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((16, 19), SRCALPHA);
            s.fill((6u8, 2u8, 236u8, 108u8));
            rustgame_core::draw::rect_radius(&mut s, (191u8, 244u8, 90u8, 120u8), Rect::new(-3, -10, 16, 5), 0, 6);
            rustgame_core::draw::circle(&mut s, (197u8, 84u8, 195u8, 255u8), (19, 11), 7, 0);
            s.set_clip(Some(Rect::new(3, 4, 3, 12)));
            rustgame_core::draw::polygon(&mut s, (6u8, 92u8, 187u8, 128u8), &[(14, -3), (3, 20), (12, 5), (16, -4), (0, 14), (13, 15)], 1);
            {
                let mut teil = s.subsurface(Rect::new(12, 11, 3, 5));
                teil.fill((253u8, 148u8, 127u8, 255u8));
            }
            {
                let mut teil = s.subsurface(Rect::new(4, 11, 11, 4));
                teil.fill((113u8, 14u8, 154u8, 255u8));
            }
            s.fill_rect((92u8, 245u8, 145u8, 255u8), Rect::new(-2, 5, 16, 2));
            let mut q = Surface::with_flags((9, 2), SRCALPHA);
            q = muster_auf(q, 9, 2);
            s.blit_ex(&q, (-4, 19), Some(Rect::new(3, 3, 12, 5)), rustgame_core::blend::flags::BLEND_RGBA_MAX);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x346EFED17AA16809, flaeche: s }
    }));
    v.push((258, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((26, 8), SRCALPHA);
            s.fill((71u8, 117u8, 0u8, 101u8));
            let mut q = Surface::with_flags((20, 6), 0);
            q = muster_auf(q, 20, 6);
            q.set_alpha(Some(96));
            s.blit_ex(&q, (3, -2), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            rustgame_core::draw::rect_radius(&mut s, (51u8, 144u8, 147u8, 124u8), Rect::new(21, 6, 9, 3), 3, 4);
            let mut q = Surface::with_flags((2, 9), SRCALPHA);
            q = muster_auf(q, 2, 9);
            s.blit_ex(&q, (6, 6), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            let mut q = Surface::with_flags((15, 9), 0);
            q = muster_auf(q, 15, 9);
            q.set_alpha(Some(248));
            q.set_colorkey(Some((124u8, 110u8, 32u8)));
            s.blit_ex(&q, (0, -4), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            rustgame_core::draw::polygon(&mut s, (51u8, 62u8, 107u8, 255u8), &[(8, 10), (14, 4), (13, 0)], 0);
            s = rustgame_core::transform::grayscale(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x443D5E514C5E95EA, flaeche: s }
    }));
    v.push((259, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 31), 0);
            s.fill((222u8, 252u8, 120u8, 255u8));
            s.fill_rect((42u8, 101u8, 140u8, 186u8), Rect::new(8, -8, 2, 11));
            s.fill_rect((254u8, 40u8, 155u8, 80u8), Rect::new(-1, 14, 10, 4));
            let mut q = Surface::with_flags((8, 8), SRCALPHA);
            q = muster_auf(q, 8, 8);
            q.set_colorkey(Some((199u8, 31u8, 242u8)));
            s.blit_ex(&q, (4, 2), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            s.scroll(5, 11);
            {
                let mut teil = s.subsurface(Rect::new(3, 28, 1, 3));
                teil.fill((203u8, 80u8, 57u8, 19u8));
            }
            rustgame_core::draw::polygon(&mut s, (202u8, 9u8, 165u8, 236u8), &[(2, -2), (5, 15), (0, 25), (10, 1), (-2, -1), (7, 4)], 3);
            rustgame_core::draw::circle(&mut s, (135u8, 159u8, 198u8, 114u8), (1, 17), 28, 3);
            s.scroll(-1, 28);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x52FB0E389DA01B47, flaeche: s }
    }));
    v.push((260, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 8), 0);
            s.fill((246u8, 108u8, 30u8, 255u8));
            rustgame_core::draw::circle(&mut s, (135u8, 36u8, 118u8, 33u8), (-3, -1), 11, 2);
            rustgame_core::draw::polygon(&mut s, (193u8, 188u8, 197u8, 218u8), &[(7, -4), (2, -3), (3, 1), (0, -3), (4, 10)], 0);
            let mut q = Surface::with_flags((12, 9), 0);
            q = muster_auf(q, 12, 9);
            s.blit_ex(&q, (6, 2), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
            rustgame_core::draw::circle(&mut s, (12u8, 107u8, 182u8, 18u8), (-3, 0), 5, 3);
            rustgame_core::draw::rect_radius(&mut s, (73u8, 73u8, 48u8, 255u8), Rect::new(-4, 9, 9, 6), 2, 5);
            rustgame_core::draw::rect_radius(&mut s, (149u8, 37u8, 243u8, 111u8), Rect::new(1, 9, 11, 2), 1, 1);
            s = rustgame_core::transform::rotate(&s, -255.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x202F437A519C7C13, flaeche: s }
    }));
    v.push((261, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((19, 6), SRCALPHA);
            s.fill((134u8, 95u8, 134u8, 255u8));
            let mut q = Surface::with_flags((9, 3), SRCALPHA);
            q = muster_auf(q, 9, 3);
            q.set_alpha(Some(62));
            s.blit_ex(&q, (5, -1), None, 0);
            let mut q = Surface::with_flags((6, 7), SRCALPHA);
            q = muster_auf(q, 6, 7);
            s.blit_ex(&q, (-3, 5), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            let mut q = Surface::with_flags((17, 7), 0);
            q = muster_auf(q, 17, 7);
            q.set_alpha(Some(77));
            s.blit_ex(&q, (21, 1), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            {
                let mut teil = s.subsurface(Rect::new(12, 1, 1, 1));
                teil.fill((123u8, 135u8, 192u8, 167u8));
            }
            rustgame_core::draw::circle(&mut s, (21u8, 121u8, 244u8, 255u8), (19, 0), 6, 4);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xB1F2BFF32F06C98E, flaeche: s }
    }));
    v.push((262, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((16, 30), 0);
            s.fill((239u8, 175u8, 185u8, 255u8));
            s.scroll(-2, 20);
            s.fill_rect((212u8, 86u8, 78u8, 119u8), Rect::new(2, 19, 6, 20));
            let mut q = Surface::with_flags((2, 18), SRCALPHA);
            q = muster_auf(q, 2, 18);
            q.set_alpha(Some(149));
            s.blit_ex(&q, (10, 23), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            rustgame_core::draw::ellipse(&mut s, (144u8, 150u8, 139u8, 112u8), Rect::new(7, -6, 19, 19), 0);
            rustgame_core::draw::line(&mut s, (198u8, 141u8, 15u8, 245u8), (-3, 26), (-4, 1), 5);
            rustgame_core::draw::circle(&mut s, (198u8, 107u8, 108u8, 255u8), (-3, 26), 14, 3);
            rustgame_core::draw::ellipse(&mut s, (5u8, 195u8, 152u8, 112u8), Rect::new(17, 15, 7, 27), 1);
            let mut q = Surface::with_flags((14, 12), 0);
            q = muster_auf(q, 14, 12);
            q.set_colorkey(Some((188u8, 52u8, 118u8)));
            s.blit_ex(&q, (8, 6), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x34DBA19C90F98AB3, flaeche: s }
    }));
    v.push((263, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((29, 32), 0);
            s.fill((218u8, 173u8, 193u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(14, 28, 15, 1));
                teil.fill((128u8, 164u8, 73u8, 255u8));
            }
            let mut q = Surface::with_flags((22, 23), 0);
            q = muster_auf(q, 22, 23);
            q.set_alpha(Some(157));
            s.blit_ex(&q, (11, 4), Some(Rect::new(0, 0, 5, 23)), rustgame_core::blend::flags::BLEND_RGB_MIN);
            rustgame_core::draw::rect(&mut s, (193u8, 73u8, 36u8, 111u8), Rect::new(0, 10, 29, 24), 0);
            {
                let mut teil = s.subsurface(Rect::new(6, 21, 7, 9));
                teil.fill((111u8, 29u8, 19u8, 195u8));
            }
            rustgame_core::draw::rect(&mut s, (174u8, 172u8, 139u8, 255u8), Rect::new(28, 18, 12, 20), 3);
            rustgame_core::draw::circle(&mut s, (158u8, 187u8, 167u8, 255u8), (10, 6), 25, 3);
            s.set_clip(Some(Rect::new(27, 7, 1, 14)));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x3FE75547701DF4BC, flaeche: s }
    }));
    v.push((264, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((25, 30), 0);
            s.fill((98u8, 85u8, 164u8, 255u8));
            rustgame_core::draw::line(&mut s, (46u8, 187u8, 226u8, 167u8), (-4, 20), (9, 31), 2);
            s.fill_rect((163u8, 68u8, 141u8, 224u8), Rect::new(-9, -9, 6, 32));
            let mut q = Surface::with_flags((15, 7), 0);
            q = muster_auf(q, 15, 7);
            q.set_colorkey(Some((134u8, 17u8, 178u8)));
            s.blit_ex(&q, (-1, 26), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            s = rustgame_core::transform::scale(&s, (6, 15));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x36BDDEE503CAD032, flaeche: s }
    }));
    v.push((265, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((31, 18), SRCALPHA);
            s.fill((160u8, 105u8, 115u8, 255u8));
            s.scroll(11, -4);
            rustgame_core::draw::circle(&mut s, (84u8, 208u8, 226u8, 255u8), (10, 3), 27, 3);
            rustgame_core::draw::rect_radius(&mut s, (104u8, 112u8, 110u8, 255u8), Rect::new(10, 7, 1, 14), 0, 4);
            s = s.convert_alpha();
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x7E99FEC09182153B, flaeche: s }
    }));
    v.push((266, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 23), 0);
            s.fill((51u8, 228u8, 99u8, 255u8));
            let mut q = Surface::with_flags((2, 5), SRCALPHA);
            q = muster_auf(q, 2, 5);
            s.blit_ex(&q, (-4, 23), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            rustgame_core::draw::polygon(&mut s, (18u8, 133u8, 85u8, 255u8), &[(4, 15), (11, 0), (5, 5), (8, 18)], 3);
            rustgame_core::draw::line(&mut s, (193u8, 215u8, 69u8, 255u8), (11, 12), (-3, 3), 3);
            rustgame_core::draw::ellipse(&mut s, (229u8, 223u8, 82u8, 255u8), Rect::new(-4, 9, 10, 2), 0);
            let mut q = Surface::with_flags((9, 2), 0);
            q = muster_auf(q, 9, 2);
            q.set_alpha(Some(54));
            s.blit_ex(&q, (3, 26), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            {
                let mut teil = s.subsurface(Rect::new(0, 13, 8, 8));
                teil.fill((155u8, 145u8, 249u8, 72u8));
            }
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x73A760331B1E7741, flaeche: s }
    }));
    v.push((267, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 6), SRCALPHA);
            s.fill((52u8, 5u8, 240u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (0u8, 19u8, 67u8, 255u8), Rect::new(11, 4, 12, 0), 2);
            rustgame_core::draw::circle(&mut s, (173u8, 57u8, 252u8, 255u8), (10, 5), 3, 4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x246A993EDDF2E7A9, flaeche: s }
    }));
    v.push((268, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((30, 23), 0);
            s.fill((21u8, 215u8, 62u8, 255u8));
            s.set_clip(Some(Rect::new(18, 8, 4, 15)));
            s.fill_rect((79u8, 111u8, 91u8, 176u8), Rect::new(9, 13, 12, 8));
            rustgame_core::draw::polygon(&mut s, (112u8, 7u8, 185u8, 73u8), &[(13, 7), (30, 22), (1, 5)], 3);
            rustgame_core::draw::rect_radius(&mut s, (107u8, 120u8, 61u8, 255u8), Rect::new(-5, 17, 24, 4), 4, 1);
            rustgame_core::draw::rect_radius(&mut s, (190u8, 48u8, 73u8, 255u8), Rect::new(16, 8, 28, 24), 4, 2);
            rustgame_core::draw::rect_radius(&mut s, (113u8, 105u8, 198u8, 255u8), Rect::new(-3, 5, 12, 13), 0, 5);
            rustgame_core::draw::rect_radius(&mut s, (211u8, 32u8, 214u8, 255u8), Rect::new(3, 13, 26, 7), 4, 4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xF74A0873750AFF1D, flaeche: s }
    }));
    v.push((269, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 6), 0);
            s.fill((159u8, 96u8, 172u8, 255u8));
            rustgame_core::draw::circle(&mut s, (51u8, 188u8, 200u8, 90u8), (-4, -4), 5, 0);
            let mut q = Surface::with_flags((7, 7), SRCALPHA);
            q = muster_auf(q, 7, 7);
            q.set_alpha(Some(115));
            s.blit_ex(&q, (0, 8), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            s.fill_rect((72u8, 252u8, 179u8, 255u8), Rect::new(6, -2, 4, 3));
            s.fill_rect((110u8, 174u8, 78u8, 145u8), Rect::new(8, -1, 7, 7));
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x1A3F16FB9587E81D, flaeche: s }
    }));
    v.push((270, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((11, 17), 0);
            s.fill((153u8, 9u8, 71u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (136u8, 130u8, 222u8, 149u8), Rect::new(4, 10, 2, 18), 3);
            rustgame_core::draw::circle(&mut s, (162u8, 1u8, 22u8, 255u8), (14, 1), 12, 2);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x8611483D65C719F9, flaeche: s }
    }));
    v.push((271, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 26), SRCALPHA);
            s.fill((245u8, 51u8, 39u8, 190u8));
            rustgame_core::draw::ellipse(&mut s, (215u8, 57u8, 181u8, 255u8), Rect::new(16, -8, 10, 9), 1);
            rustgame_core::draw::rect_radius(&mut s, (244u8, 142u8, 21u8, 30u8), Rect::new(-2, 12, 2, 2), 1, 1);
            rustgame_core::draw::ellipse(&mut s, (98u8, 136u8, 135u8, 109u8), Rect::new(-5, 27, 0, 27), 4);
            {
                let mut teil = s.subsurface(Rect::new(8, 4, 7, 10));
                teil.fill((56u8, 77u8, 129u8, 38u8));
            }
            let mut q = Surface::with_flags((16, 17), SRCALPHA);
            q = muster_auf(q, 16, 17);
            s.blit_ex(&q, (-2, 29), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            rustgame_core::draw::ellipse(&mut s, (33u8, 186u8, 110u8, 149u8), Rect::new(16, 21, 19, 13), 3);
            rustgame_core::draw::polygon(&mut s, (129u8, 27u8, 91u8, 39u8), &[(22, 29), (24, -1), (13, 5)], 1);
            s = s.convert_alpha();
            s = rustgame_core::transform::rotate(&s, 150.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x3BB4CBFA004938D7, flaeche: s }
    }));
    v.push((272, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 17), SRCALPHA);
            s.fill((188u8, 140u8, 92u8, 174u8));
            rustgame_core::draw::polygon(&mut s, (11u8, 121u8, 100u8, 255u8), &[(7, 0), (24, 3), (12, 15), (3, 16)], 1);
            let mut q = Surface::with_flags((16, 16), 0);
            q = muster_auf(q, 16, 16);
            s.blit_ex(&q, (0, 14), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            rustgame_core::draw::circle(&mut s, (170u8, 233u8, 40u8, 255u8), (20, 10), 14, 3);
            rustgame_core::draw::polygon(&mut s, (10u8, 60u8, 12u8, 112u8), &[(10, 12), (5, 14), (13, 0)], 1);
            s.scroll(-14, -8);
            s.fill_rect((170u8, 133u8, 16u8, 255u8), Rect::new(7, -3, 9, 17));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x7DA0F07EB023AB1D, flaeche: s }
    }));
    v.push((273, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 27), 0);
            s.fill((122u8, 156u8, 148u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(1, 16, 4, 9));
                teil.fill((201u8, 155u8, 77u8, 255u8));
            }
            rustgame_core::draw::circle(&mut s, (80u8, 90u8, 148u8, 120u8), (7, 5), 15, 4);
            rustgame_core::draw::rect_radius(&mut s, (75u8, 95u8, 214u8, 146u8), Rect::new(-4, 16, 9, 30), 3, 1);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x5AC1775A10CC670B, flaeche: s }
    }));
    v.push((274, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((10, 20), SRCALPHA);
            s.fill((107u8, 13u8, 136u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (7u8, 171u8, 204u8, 111u8), Rect::new(10, 11, 0, 12), 3, 4);
            rustgame_core::draw::rect_radius(&mut s, (27u8, 183u8, 115u8, 142u8), Rect::new(-1, 18, 6, 8), 4, 5);
            let mut q = Surface::with_flags((3, 21), SRCALPHA);
            q = muster_auf(q, 3, 21);
            q.set_alpha(Some(62));
            q.set_colorkey(Some((179u8, 78u8, 132u8)));
            s.blit_ex(&q, (9, 7), None, 0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x5E28157FA45766A3, flaeche: s }
    }));
    v.push((275, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((30, 32), 0);
            s.fill((190u8, 151u8, 163u8, 255u8));
            rustgame_core::draw::line(&mut s, (181u8, 182u8, 232u8, 67u8), (24, 32), (31, 23), 4);
            rustgame_core::draw::circle(&mut s, (23u8, 180u8, 99u8, 255u8), (16, 10), 22, 3);
            rustgame_core::draw::circle(&mut s, (231u8, 96u8, 185u8, 246u8), (27, 1), 15, 1);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xD5ACF3D383CCAF6A, flaeche: s }
    }));
    v.push((276, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((17, 8), SRCALPHA);
            s.fill((139u8, 142u8, 2u8, 163u8));
            let mut q = Surface::with_flags((17, 6), SRCALPHA);
            q = muster_auf(q, 17, 6);
            q.set_alpha(Some(12));
            q.set_colorkey(Some((172u8, 99u8, 10u8)));
            s.blit_ex(&q, (9, -3), Some(Rect::new(3, 6, 13, 7)), rustgame_core::blend::flags::BLEND_RGB_MIN);
            s.set_clip(Some(Rect::new(8, 7, 6, 1)));
            s.fill_rect((149u8, 5u8, 206u8, 121u8), Rect::new(0, 9, 20, 1));
            let mut q = Surface::with_flags((6, 10), SRCALPHA);
            q = muster_auf(q, 6, 10);
            q.set_colorkey(Some((206u8, 131u8, 58u8)));
            s.blit_ex(&q, (20, 11), Some(Rect::new(4, 2, 9, 7)), rustgame_core::blend::flags::BLEND_RGBA_MULT);
            s.fill_rect((196u8, 75u8, 98u8, 19u8), Rect::new(-9, 3, 12, 4));
            rustgame_core::draw::circle(&mut s, (212u8, 190u8, 82u8, 255u8), (10, 10), 16, 2);
            rustgame_core::draw::ellipse(&mut s, (88u8, 119u8, 109u8, 255u8), Rect::new(-3, -2, 7, 9), 3);
            s.scroll(-10, 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x52DD672C806D6435, flaeche: s }
    }));
    v.push((277, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 11), SRCALPHA);
            s.fill((242u8, 64u8, 240u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (156u8, 48u8, 36u8, 255u8), &[(0, 10), (11, -1), (2, 10), (0, -3)], 3);
            let mut q = Surface::with_flags((3, 9), SRCALPHA);
            q = muster_auf(q, 3, 9);
            q.set_alpha(Some(203));
            s.blit_ex(&q, (8, -2), Some(Rect::new(-2, -1, 0, 6)), rustgame_core::blend::flags::BLEND_RGBA_MIN);
            rustgame_core::draw::circle(&mut s, (97u8, 8u8, 74u8, 168u8), (8, -3), 5, 0);
            s.fill_rect((1u8, 98u8, 186u8, 255u8), Rect::new(8, 8, 7, 13));
            s = rustgame_core::transform::grayscale(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x0D6DB282894F5196, flaeche: s }
    }));
    v.push((278, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((15, 21), SRCALPHA);
            s.fill((184u8, 114u8, 234u8, 241u8));
            {
                let mut teil = s.subsurface(Rect::new(5, 11, 1, 7));
                teil.fill((208u8, 3u8, 125u8, 43u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (55u8, 88u8, 137u8, 164u8), Rect::new(9, 17, 8, 15), 0, 2);
            rustgame_core::draw::ellipse(&mut s, (228u8, 189u8, 101u8, 33u8), Rect::new(14, 3, 12, 22), 1);
            s.fill_rect((78u8, 39u8, 166u8, 220u8), Rect::new(-5, 10, 7, 19));
            {
                let mut teil = s.subsurface(Rect::new(7, 12, 5, 6));
                teil.fill((166u8, 173u8, 82u8, 255u8));
            }
            s.fill_rect((203u8, 69u8, 213u8, 255u8), Rect::new(-7, 9, 14, 2));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x6B150002226884E0, flaeche: s }
    }));
    v.push((279, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((19, 17), 0);
            s.fill((26u8, 250u8, 95u8, 255u8));
            let mut q = Surface::with_flags((11, 6), SRCALPHA);
            q = muster_auf(q, 11, 6);
            s.blit_ex(&q, (18, 1), None, 0);
            s.set_clip(Some(Rect::new(8, 15, 10, 1)));
            let mut q = Surface::with_flags((19, 13), SRCALPHA);
            q = muster_auf(q, 19, 13);
            q.set_alpha(Some(239));
            s.blit_ex(&q, (22, 9), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x41CDF7BF7937C5DE, flaeche: s }
    }));
    v.push((280, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((15, 20), 0);
            s.fill((42u8, 133u8, 231u8, 255u8));
            s.scroll(9, -19);
            let mut q = Surface::with_flags((3, 21), 0);
            q = muster_auf(q, 3, 21);
            s.blit_ex(&q, (17, 15), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            rustgame_core::draw::rect(&mut s, (191u8, 139u8, 152u8, 178u8), Rect::new(9, 11, 0, 23), 4);
            s.fill_rect((72u8, 212u8, 210u8, 185u8), Rect::new(7, 9, 8, 10));
            rustgame_core::draw::rect_radius(&mut s, (47u8, 187u8, 80u8, 254u8), Rect::new(10, 19, 18, 0), 0, 2);
            rustgame_core::draw::line(&mut s, (178u8, 65u8, 46u8, 255u8), (6, 11), (-3, 22), 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x73BE7F757AA3A464, flaeche: s }
    }));
    v.push((281, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((6, 15), SRCALPHA);
            s.fill((17u8, 143u8, 81u8, 127u8));
            let mut q = Surface::with_flags((6, 6), SRCALPHA);
            q = muster_auf(q, 6, 6);
            s.blit_ex(&q, (-4, 10), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            let mut q = Surface::with_flags((3, 2), SRCALPHA);
            q = muster_auf(q, 3, 2);
            s.blit_ex(&q, (8, 10), Some(Rect::new(1, 0, 6, 5)), rustgame_core::blend::flags::BLEND_RGBA_MULT);
            s = rustgame_core::transform::grayscale(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x7F7875E7495C1222, flaeche: s }
    }));
    v.push((282, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((27, 7), 0);
            s.fill((245u8, 193u8, 160u8, 255u8));
            s.set_clip(Some(Rect::new(6, 2, 19, 5)));
            s.scroll(3, 3);
            rustgame_core::draw::rect_radius(&mut s, (71u8, 149u8, 84u8, 249u8), Rect::new(28, 8, 10, 6), 2, 5);
            s = rustgame_core::transform::flip(&s, false, false);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xC935BCA3213E0C1A, flaeche: s }
    }));
    v.push((283, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((29, 6), SRCALPHA);
            s.fill((233u8, 197u8, 222u8, 134u8));
            rustgame_core::draw::circle(&mut s, (99u8, 6u8, 92u8, 255u8), (27, 2), 13, 3);
            {
                let mut teil = s.subsurface(Rect::new(16, 2, 7, 2));
                teil.fill((33u8, 100u8, 59u8, 255u8));
            }
            rustgame_core::draw::line(&mut s, (168u8, 96u8, 104u8, 62u8), (31, 9), (7, 3), 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x282C12F8ECCB02D9, flaeche: s }
    }));
    v.push((284, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 32), SRCALPHA);
            s.fill((232u8, 152u8, 72u8, 255u8));
            s.set_clip(Some(Rect::new(0, 20, 3, 3)));
            s.fill_rect((116u8, 160u8, 149u8, 111u8), Rect::new(-2, -6, 6, 21));
            rustgame_core::draw::circle(&mut s, (181u8, 113u8, 169u8, 185u8), (4, 34), 6, 0);
            s.set_clip(Some(Rect::new(5, 13, 2, 10)));
            let mut q = Surface::with_flags((5, 34), SRCALPHA);
            q = muster_auf(q, 5, 34);
            s.blit_ex(&q, (9, 16), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xCA87D50156B39025, flaeche: s }
    }));
    v.push((285, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((24, 15), SRCALPHA);
            s.fill((129u8, 177u8, 201u8, 47u8));
            let mut q = Surface::with_flags((13, 17), 0);
            q = muster_auf(q, 13, 17);
            s.blit_ex(&q, (7, 4), None, 0);
            s.set_clip(Some(Rect::new(23, 8, 1, 7)));
            {
                let mut teil = s.subsurface(Rect::new(6, 12, 2, 2));
                teil.fill((254u8, 221u8, 88u8, 255u8));
            }
            rustgame_core::draw::circle(&mut s, (50u8, 225u8, 197u8, 101u8), (9, 16), 23, 3);
            rustgame_core::draw::ellipse(&mut s, (83u8, 250u8, 176u8, 255u8), Rect::new(22, 8, 25, 7), 2);
            s.fill_rect((105u8, 103u8, 114u8, 255u8), Rect::new(-9, 2, 3, 7));
            let mut q = Surface::with_flags((3, 10), SRCALPHA);
            q = muster_auf(q, 3, 10);
            s.blit_ex(&q, (8, 15), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            rustgame_core::draw::polygon(&mut s, (205u8, 217u8, 231u8, 255u8), &[(0, 9), (16, -1), (-4, 13)], 0);
            s = rustgame_core::transform::scale(&s, (8, 18));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xE23708EFCD511A85, flaeche: s }
    }));
    v.push((286, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 8), SRCALPHA);
            s.fill((210u8, 208u8, 222u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (65u8, 167u8, 198u8, 255u8), Rect::new(-4, 8, 4, 3), 3, 0);
            s.fill_rect((0u8, 68u8, 15u8, 255u8), Rect::new(15, -4, 18, 5));
            rustgame_core::draw::rect_radius(&mut s, (208u8, 213u8, 235u8, 254u8), Rect::new(8, 9, 14, 9), 2, 3);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x6464BD16A44D2A49, flaeche: s }
    }));
    v.push((287, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((23, 6), SRCALPHA);
            s.fill((85u8, 213u8, 115u8, 255u8));
            s.set_clip(Some(Rect::new(10, 2, 3, 4)));
            rustgame_core::draw::circle(&mut s, (168u8, 252u8, 98u8, 255u8), (17, -2), 17, 3);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x7EC6D96E78AD6215, flaeche: s }
    }));
    v.push((288, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((15, 21), 0);
            s.fill((245u8, 240u8, 24u8, 255u8));
            let mut q = Surface::with_flags((2, 16), 0);
            q = muster_auf(q, 2, 16);
            q.set_colorkey(Some((244u8, 86u8, 248u8)));
            s.blit_ex(&q, (18, 21), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            rustgame_core::draw::circle(&mut s, (185u8, 161u8, 65u8, 255u8), (8, 15), 11, 1);
            rustgame_core::draw::line(&mut s, (76u8, 103u8, 107u8, 213u8), (11, -1), (13, 16), 3);
            s.fill_rect((123u8, 238u8, 22u8, 255u8), Rect::new(-5, 1, 11, 14));
            rustgame_core::draw::rect_radius(&mut s, (184u8, 251u8, 137u8, 243u8), Rect::new(16, -10, 15, 10), 0, 2);
            rustgame_core::draw::line(&mut s, (212u8, 188u8, 3u8, 108u8), (8, 16), (4, 11), 1);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xFDB0725C0A6621C6, flaeche: s }
    }));
    v.push((289, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((30, 14), 0);
            s.fill((95u8, 42u8, 226u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (199u8, 14u8, 213u8, 255u8), Rect::new(15, -4, 31, 1), 4);
            rustgame_core::draw::ellipse(&mut s, (34u8, 83u8, 16u8, 83u8), Rect::new(-13, -7, 8, 17), 4);
            s = s.convert_alpha();
            s = rustgame_core::transform::grayscale(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xFC583C10CE7792C5, flaeche: s }
    }));
    v.push((290, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((23, 16), 0);
            s.fill((3u8, 67u8, 126u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (62u8, 210u8, 177u8, 255u8), Rect::new(18, -8, 9, 4), 3, 3);
            s.set_clip(Some(Rect::new(16, 13, 2, 3)));
            s = rustgame_core::transform::scale(&s, (18, 21));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xB599D41E9CD41EA5, flaeche: s }
    }));
    v.push((291, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((27, 21), 0);
            s.fill((46u8, 199u8, 246u8, 255u8));
            rustgame_core::draw::circle(&mut s, (104u8, 132u8, 134u8, 255u8), (17, 9), 3, 1);
            rustgame_core::draw::circle(&mut s, (157u8, 35u8, 166u8, 111u8), (1, 10), 2, 1);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xDC7B5936EB374DE1, flaeche: s }
    }));
    v.push((292, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((30, 10), 0);
            s.fill((76u8, 166u8, 98u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (96u8, 215u8, 8u8, 255u8), Rect::new(-8, -4, 30, 2), 1, 6);
            rustgame_core::draw::rect_radius(&mut s, (16u8, 178u8, 74u8, 118u8), Rect::new(-10, 3, 15, 12), 1, 3);
            rustgame_core::draw::circle(&mut s, (194u8, 13u8, 33u8, 128u8), (23, -4), 28, 3);
            rustgame_core::draw::ellipse(&mut s, (69u8, 186u8, 27u8, 152u8), Rect::new(25, -5, 28, 9), 1);
            rustgame_core::draw::ellipse(&mut s, (241u8, 14u8, 219u8, 254u8), Rect::new(3, 3, 19, 2), 3);
            rustgame_core::draw::circle(&mut s, (249u8, 248u8, 117u8, 203u8), (24, 8), 9, 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x6DA66641D0101F91, flaeche: s }
    }));
    v.push((293, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 21), SRCALPHA);
            s.fill((57u8, 226u8, 166u8, 210u8));
            rustgame_core::draw::polygon(&mut s, (112u8, 251u8, 50u8, 255u8), &[(16, -3), (18, 12), (12, 24), (-3, 2)], 3);
            s.set_clip(Some(Rect::new(13, 2, 5, 11)));
            rustgame_core::draw::rect_radius(&mut s, (202u8, 73u8, 74u8, 255u8), Rect::new(-3, 0, 1, 22), 2, 0);
            rustgame_core::draw::rect_radius(&mut s, (90u8, 120u8, 188u8, 139u8), Rect::new(18, -7, 2, 0), 3, 3);
            s.fill_rect((190u8, 35u8, 202u8, 255u8), Rect::new(5, 20, 12, 18));
            rustgame_core::draw::circle(&mut s, (181u8, 71u8, 53u8, 255u8), (16, 14), 4, 1);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x5980767EE88A7DDF, flaeche: s }
    }));
    v.push((294, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((11, 7), SRCALPHA);
            s.fill((46u8, 222u8, 205u8, 255u8));
            rustgame_core::draw::circle(&mut s, (119u8, 181u8, 156u8, 123u8), (10, 1), 8, 4);
            rustgame_core::draw::rect_radius(&mut s, (132u8, 189u8, 233u8, 255u8), Rect::new(8, -2, 2, 3), 4, 2);
            let mut q = Surface::with_flags((4, 7), 0);
            q = muster_auf(q, 4, 7);
            q.set_alpha(Some(232));
            q.set_colorkey(Some((130u8, 236u8, 111u8)));
            s.blit_ex(&q, (6, 8), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            rustgame_core::draw::rect_radius(&mut s, (237u8, 233u8, 182u8, 255u8), Rect::new(-1, -2, 12, 6), 3, 0);
            s.scroll(-9, 0);
            s = rustgame_core::transform::grayscale(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x6C2E43F7FAC551FF, flaeche: s }
    }));
    v.push((295, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((32, 15), 0);
            s.fill((240u8, 162u8, 244u8, 255u8));
            rustgame_core::draw::line(&mut s, (29u8, 107u8, 113u8, 255u8), (15, 4), (20, -2), 2);
            let mut q = Surface::with_flags((13, 14), SRCALPHA);
            q = muster_auf(q, 13, 14);
            q.set_colorkey(Some((139u8, 111u8, 62u8)));
            s.blit_ex(&q, (2, 3), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            s = rustgame_core::transform::rotate(&s, -192.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xBAF22B94DC2419EB, flaeche: s }
    }));
    v.push((296, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 14), SRCALPHA);
            s.fill((84u8, 61u8, 21u8, 218u8));
            rustgame_core::draw::circle(&mut s, (119u8, 2u8, 210u8, 255u8), (7, 0), 12, 4);
            {
                let mut teil = s.subsurface(Rect::new(1, 2, 2, 10));
                teil.fill((175u8, 98u8, 139u8, 214u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (230u8, 134u8, 221u8, 255u8), Rect::new(1, 7, 8, 16), 2, 4);
            {
                let mut teil = s.subsurface(Rect::new(1, 12, 2, 2));
                teil.fill((2u8, 17u8, 56u8, 24u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x1778E773537D71CD, flaeche: s }
    }));
    v.push((297, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((22, 17), 0);
            s.fill((206u8, 20u8, 164u8, 255u8));
            s.set_clip(Some(Rect::new(5, 0, 6, 15)));
            let mut q = Surface::with_flags((16, 14), SRCALPHA);
            q = muster_auf(q, 16, 14);
            q.set_alpha(Some(201));
            s.blit_ex(&q, (20, 18), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            {
                let mut teil = s.subsurface(Rect::new(13, 7, 9, 10));
                teil.fill((212u8, 10u8, 216u8, 161u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (120u8, 236u8, 129u8, 255u8), Rect::new(0, 11, 17, 12), 4, 3);
            rustgame_core::draw::rect_radius(&mut s, (83u8, 0u8, 152u8, 147u8), Rect::new(1, 6, 15, 1), 0, 6);
            let mut q = Surface::with_flags((19, 14), SRCALPHA);
            q = muster_auf(q, 19, 14);
            s.blit_ex(&q, (24, 3), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            s.set_clip(Some(Rect::new(11, 6, 9, 9)));
            let mut q = Surface::with_flags((20, 12), SRCALPHA);
            q = muster_auf(q, 20, 12);
            s.blit_ex(&q, (5, 16), Some(Rect::new(-1, -5, 17, 0)), rustgame_core::blend::flags::BLEND_RGBA_MIN);
            s = rustgame_core::transform::scale(&s, (10, 3));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x86C7A6A8CA0931CE, flaeche: s }
    }));
    v.push((298, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 23), 0);
            s.fill((92u8, 91u8, 162u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (6u8, 140u8, 156u8, 237u8), Rect::new(13, 16, 4, 0), 3);
            rustgame_core::draw::rect_radius(&mut s, (251u8, 149u8, 24u8, 217u8), Rect::new(-5, -10, 5, 19), 1, 4);
            let mut q = Surface::with_flags((11, 21), SRCALPHA);
            q = muster_auf(q, 11, 21);
            q.set_colorkey(Some((190u8, 180u8, 198u8)));
            s.blit_ex(&q, (2, 26), Some(Rect::new(2, 6, 3, 0)), rustgame_core::blend::flags::BLEND_RGBA_ADD);
            s.fill_rect((182u8, 222u8, 78u8, 192u8), Rect::new(9, 2, 0, 6));
            rustgame_core::draw::rect_radius(&mut s, (26u8, 171u8, 226u8, 186u8), Rect::new(11, -12, 0, 0), 3, 5);
            let mut q = Surface::with_flags((2, 16), 0);
            q = muster_auf(q, 2, 16);
            q.set_alpha(Some(209));
            s.blit_ex(&q, (9, 4), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            rustgame_core::draw::circle(&mut s, (0u8, 77u8, 116u8, 255u8), (1, 4), 21, 3);
            s = rustgame_core::transform::scale(&s, (12, 27));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x8953AE4014CD0C30, flaeche: s }
    }));
    v.push((299, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((17, 24), 0);
            s.fill((6u8, 234u8, 158u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (44u8, 240u8, 173u8, 24u8), Rect::new(12, 9, 12, 13), 4, 4);
            rustgame_core::draw::ellipse(&mut s, (207u8, 205u8, 101u8, 127u8), Rect::new(-6, 11, 3, 1), 1);
            rustgame_core::draw::rect_radius(&mut s, (132u8, 253u8, 201u8, 255u8), Rect::new(15, -8, 20, 7), 2, 5);
            s.fill_rect((142u8, 72u8, 175u8, 82u8), Rect::new(8, -12, 0, 19));
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xB800B0E5DAE6D581, flaeche: s }
    }));
    v.push((300, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((25, 17), SRCALPHA);
            s.fill((177u8, 216u8, 84u8, 255u8));
            s.set_clip(Some(Rect::new(10, 11, 10, 2)));
            rustgame_core::draw::rect_radius(&mut s, (33u8, 151u8, 148u8, 24u8), Rect::new(22, -7, 16, 8), 4, 4);
            s.fill_rect((104u8, 192u8, 154u8, 44u8), Rect::new(-11, 9, 26, 5));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xF42192FB22D39435, flaeche: s }
    }));
    v.push((301, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((18, 26), 0);
            s.fill((252u8, 235u8, 120u8, 255u8));
            rustgame_core::draw::rect(&mut s, (124u8, 67u8, 48u8, 255u8), Rect::new(17, 15, 17, 22), 2);
            rustgame_core::draw::rect(&mut s, (147u8, 163u8, 241u8, 89u8), Rect::new(-2, 26, 14, 29), 0);
            rustgame_core::draw::line(&mut s, (229u8, 145u8, 99u8, 255u8), (6, 6), (20, -3), 5);
            s.scroll(-14, -4);
            s.set_clip(Some(Rect::new(2, 1, 15, 24)));
            s.scroll(6, 23);
            s.set_clip(Some(Rect::new(0, 22, 4, 3)));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x32A1B410ADB8DCCD, flaeche: s }
    }));
    v.push((302, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((23, 23), 0);
            s.fill((208u8, 243u8, 96u8, 255u8));
            s.fill_rect((202u8, 173u8, 57u8, 255u8), Rect::new(22, -11, 20, 7));
            rustgame_core::draw::rect_radius(&mut s, (38u8, 186u8, 193u8, 27u8), Rect::new(18, 6, 17, 22), 3, 5);
            rustgame_core::draw::circle(&mut s, (71u8, 45u8, 31u8, 255u8), (20, 5), 6, 3);
            s.scroll(-8, -5);
            rustgame_core::draw::rect_radius(&mut s, (70u8, 12u8, 126u8, 127u8), Rect::new(-11, -12, 15, 12), 0, 6);
            s.set_clip(Some(Rect::new(1, 15, 8, 3)));
            rustgame_core::draw::circle(&mut s, (97u8, 72u8, 247u8, 255u8), (17, 13), 5, 2);
            s.fill_rect((20u8, 184u8, 210u8, 240u8), Rect::new(7, -3, 10, 19));
            s = rustgame_core::transform::rotate(&s, -12.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x524793C62933E0F1, flaeche: s }
    }));
    v.push((303, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 13), 0);
            s.fill((99u8, 47u8, 132u8, 255u8));
            let mut q = Surface::with_flags((8, 4), SRCALPHA);
            q = muster_auf(q, 8, 4);
            s.blit_ex(&q, (0, 12), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            s.fill_rect((70u8, 252u8, 148u8, 200u8), Rect::new(-4, 12, 10, 13));
            s.scroll(-7, -6);
            {
                let mut teil = s.subsurface(Rect::new(1, 2, 2, 10));
                teil.fill((55u8, 127u8, 128u8, 255u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (10u8, 139u8, 140u8, 255u8), Rect::new(8, 13, 2, 11), 0, 6);
            s.set_clip(Some(Rect::new(4, 2, 1, 11)));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x030D722551EC2D5A, flaeche: s }
    }));
    v.push((304, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((10, 12), SRCALPHA);
            s.fill((28u8, 119u8, 156u8, 231u8));
            rustgame_core::draw::rect_radius(&mut s, (25u8, 26u8, 148u8, 255u8), Rect::new(3, -4, 4, 12), 1, 1);
            rustgame_core::draw::polygon(&mut s, (158u8, 234u8, 164u8, 106u8), &[(-1, 1), (4, 15), (6, 12), (8, 10), (-1, 7), (2, 5)], 1);
            {
                let mut teil = s.subsurface(Rect::new(6, 1, 1, 1));
                teil.fill((24u8, 202u8, 186u8, 238u8));
            }
            s = rustgame_core::transform::scale(&s, (4, 12));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x4084F2A1FD477F6F, flaeche: s }
    }));
    v.push((305, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((11, 24), 0);
            s.fill((145u8, 88u8, 86u8, 255u8));
            let mut q = Surface::with_flags((6, 24), SRCALPHA);
            q = muster_auf(q, 6, 24);
            q.set_alpha(Some(30));
            s.blit_ex(&q, (10, 27), Some(Rect::new(0, 5, 0, 4)), rustgame_core::blend::flags::BLEND_RGB_SUB);
            rustgame_core::draw::rect_radius(&mut s, (180u8, 251u8, 143u8, 17u8), Rect::new(1, 17, 7, 4), 4, 0);
            rustgame_core::draw::rect_radius(&mut s, (166u8, 253u8, 148u8, 255u8), Rect::new(-3, 4, 10, 16), 4, 0);
            s = s.convert();
            s = rustgame_core::transform::flip(&s, false, true);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x8404FACFD3522101, flaeche: s }
    }));
    v.push((306, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((19, 22), 0);
            s.fill((142u8, 106u8, 210u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (29u8, 183u8, 37u8, 18u8), Rect::new(16, 22, 21, 8), 2);
            let mut q = Surface::with_flags((15, 3), 0);
            q = muster_auf(q, 15, 3);
            q.set_colorkey(Some((144u8, 169u8, 142u8)));
            s.blit_ex(&q, (8, 4), Some(Rect::new(10, 3, 17, 2)), rustgame_core::blend::flags::BLEND_RGBA_MULT);
            rustgame_core::draw::polygon(&mut s, (100u8, 30u8, 205u8, 148u8), &[(12, 22), (2, 25), (-3, 0), (4, 4), (14, 22), (8, 1)], 3);
            rustgame_core::draw::rect_radius(&mut s, (176u8, 184u8, 56u8, 255u8), Rect::new(-9, 23, 21, 7), 1, 1);
            {
                let mut teil = s.subsurface(Rect::new(7, 0, 2, 21));
                teil.fill((115u8, 26u8, 166u8, 255u8));
            }
            {
                let mut teil = s.subsurface(Rect::new(9, 10, 3, 6));
                teil.fill((92u8, 102u8, 205u8, 255u8));
            }
            s.set_clip(Some(Rect::new(3, 8, 3, 5)));
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x1554DBF67CAAC3E8, flaeche: s }
    }));
    v.push((307, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((26, 22), SRCALPHA);
            s.fill((218u8, 143u8, 10u8, 246u8));
            let mut q = Surface::with_flags((22, 8), 0);
            q = muster_auf(q, 22, 8);
            q.set_alpha(Some(226));
            s.blit_ex(&q, (20, 25), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            rustgame_core::draw::line(&mut s, (172u8, 127u8, 126u8, 255u8), (7, 8), (6, 12), 1);
            rustgame_core::draw::ellipse(&mut s, (181u8, 53u8, 21u8, 255u8), Rect::new(-12, 5, 0, 15), 2);
            rustgame_core::draw::circle(&mut s, (5u8, 72u8, 64u8, 109u8), (2, 21), 4, 3);
            s.scroll(-8, 16);
            rustgame_core::draw::line(&mut s, (58u8, 147u8, 163u8, 212u8), (0, -2), (12, -2), 5);
            rustgame_core::draw::rect_radius(&mut s, (173u8, 84u8, 123u8, 27u8), Rect::new(25, 8, 0, 7), 2, 0);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x47329DBCAC92E523, flaeche: s }
    }));
    v.push((308, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((11, 10), 0);
            s.fill((181u8, 191u8, 137u8, 255u8));
            let mut q = Surface::with_flags((13, 8), 0);
            q = muster_auf(q, 13, 8);
            q.set_colorkey(Some((44u8, 44u8, 60u8)));
            s.blit_ex(&q, (-3, 10), Some(Rect::new(13, 6, 15, 0)), rustgame_core::blend::flags::BLEND_RGBA_MIN);
            let mut q = Surface::with_flags((10, 6), SRCALPHA);
            q = muster_auf(q, 10, 6);
            q.set_colorkey(Some((205u8, 167u8, 200u8)));
            s.blit_ex(&q, (12, 12), Some(Rect::new(1, 0, 7, 3)), rustgame_core::blend::flags::BLEND_RGB_MAX);
            let mut q = Surface::with_flags((4, 2), SRCALPHA);
            q = muster_auf(q, 4, 2);
            q.set_alpha(Some(178));
            s.blit_ex(&q, (14, -3), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            s = rustgame_core::transform::rotate(&s, 136.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xAA3F8B7E2DE503A5, flaeche: s }
    }));
    v.push((309, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 15), SRCALPHA);
            s.fill((122u8, 104u8, 118u8, 255u8));
            rustgame_core::draw::circle(&mut s, (93u8, 135u8, 208u8, 31u8), (-4, 13), 3, 0);
            rustgame_core::draw::circle(&mut s, (44u8, 162u8, 11u8, 12u8), (4, -4), 3, 4);
            s.set_clip(Some(Rect::new(1, 1, 7, 8)));
            let mut q = Surface::with_flags((8, 7), 0);
            q = muster_auf(q, 8, 7);
            s.blit_ex(&q, (9, 1), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            s.scroll(8, -10);
            rustgame_core::draw::rect_radius(&mut s, (252u8, 52u8, 130u8, 179u8), Rect::new(-3, -8, 0, 10), 3, 0);
            rustgame_core::draw::rect_radius(&mut s, (40u8, 155u8, 122u8, 255u8), Rect::new(-2, 9, 10, 11), 2, 0);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xE1698C3EC30A5E25, flaeche: s }
    }));
    v.push((310, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((23, 8), SRCALPHA);
            s.fill((146u8, 182u8, 161u8, 255u8));
            let mut q = Surface::with_flags((25, 5), SRCALPHA);
            q = muster_auf(q, 25, 5);
            q.set_alpha(Some(71));
            s.blit_ex(&q, (1, 1), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            rustgame_core::draw::polygon(&mut s, (189u8, 148u8, 241u8, 157u8), &[(25, -2), (5, -3), (9, -2)], 3);
            rustgame_core::draw::ellipse(&mut s, (170u8, 177u8, 210u8, 244u8), Rect::new(-4, 8, 13, 4), 4);
            rustgame_core::draw::circle(&mut s, (0u8, 238u8, 77u8, 255u8), (16, -2), 19, 3);
            let mut q = Surface::with_flags((6, 4), SRCALPHA);
            q = muster_auf(q, 6, 4);
            q.set_alpha(Some(49));
            s.blit_ex(&q, (23, 10), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x65B50F19C9C0EAE5, flaeche: s }
    }));
    v.push((311, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((32, 14), SRCALPHA);
            s.fill((223u8, 242u8, 79u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (164u8, 157u8, 48u8, 116u8), Rect::new(31, -3, 16, 9), 0, 2);
            rustgame_core::draw::rect_radius(&mut s, (158u8, 228u8, 236u8, 255u8), Rect::new(33, -6, 16, 3), 0, 0);
            let mut q = Surface::with_flags((10, 14), 0);
            q = muster_auf(q, 10, 14);
            q.set_colorkey(Some((11u8, 105u8, 45u8)));
            s.blit_ex(&q, (-1, 14), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            let mut q = Surface::with_flags((5, 6), SRCALPHA);
            q = muster_auf(q, 5, 6);
            s.blit_ex(&q, (-4, 13), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x9BA7ABA0E740C7F0, flaeche: s }
    }));
    v.push((312, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((26, 10), 0);
            s.fill((16u8, 75u8, 167u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(21, 7, 1, 2));
                teil.fill((78u8, 2u8, 163u8, 170u8));
            }
            {
                let mut teil = s.subsurface(Rect::new(25, 2, 1, 5));
                teil.fill((165u8, 140u8, 162u8, 255u8));
            }
            {
                let mut teil = s.subsurface(Rect::new(5, 6, 7, 2));
                teil.fill((235u8, 70u8, 158u8, 255u8));
            }
            s = s.convert_alpha();
            s = rustgame_core::transform::scale(&s, (22, 5));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x330C160AA2159728, flaeche: s }
    }));
    v.push((313, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((24, 11), 0);
            s.fill((218u8, 244u8, 8u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (168u8, 10u8, 74u8, 255u8), Rect::new(16, 6, 15, 9), 3);
            rustgame_core::draw::line(&mut s, (113u8, 43u8, 1u8, 255u8), (8, 12), (22, 3), 3);
            rustgame_core::draw::line(&mut s, (252u8, 195u8, 1u8, 255u8), (1, 5), (27, 3), 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xCE491667108EC290, flaeche: s }
    }));
    v.push((314, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 20), SRCALPHA);
            s.fill((149u8, 119u8, 205u8, 255u8));
            let mut q = Surface::with_flags((9, 3), 0);
            q = muster_auf(q, 9, 3);
            q.set_alpha(Some(55));
            s.blit_ex(&q, (-4, 3), None, 0);
            {
                let mut teil = s.subsurface(Rect::new(5, 10, 7, 6));
                teil.fill((173u8, 201u8, 225u8, 110u8));
            }
            s.set_clip(Some(Rect::new(0, 14, 10, 3)));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x81CED0BB7140145E, flaeche: s }
    }));
    v.push((315, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((19, 28), SRCALPHA);
            s.fill((25u8, 40u8, 79u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(8, 3, 9, 1));
                teil.fill((106u8, 49u8, 149u8, 101u8));
            }
            rustgame_core::draw::ellipse(&mut s, (209u8, 181u8, 18u8, 255u8), Rect::new(5, 15, 0, 20), 1);
            s.fill_rect((62u8, 115u8, 5u8, 119u8), Rect::new(18, 4, 5, 9));
            rustgame_core::draw::rect_radius(&mut s, (10u8, 177u8, 191u8, 255u8), Rect::new(6, 15, 9, 0), 3, 6);
            rustgame_core::draw::rect_radius(&mut s, (238u8, 37u8, 98u8, 239u8), Rect::new(14, 2, 0, 5), 3, 2);
            {
                let mut teil = s.subsurface(Rect::new(0, 25, 6, 2));
                teil.fill((153u8, 54u8, 200u8, 255u8));
            }
            {
                let mut teil = s.subsurface(Rect::new(18, 2, 1, 25));
                teil.fill((163u8, 4u8, 118u8, 255u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xED0E3285EB481E9A, flaeche: s }
    }));
    v.push((316, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((6, 31), 0);
            s.fill((6u8, 163u8, 5u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (5u8, 233u8, 32u8, 255u8), Rect::new(4, 16, 3, 14), 0);
            rustgame_core::draw::ellipse(&mut s, (167u8, 103u8, 253u8, 255u8), Rect::new(5, 26, 7, 23), 3);
            rustgame_core::draw::circle(&mut s, (77u8, 195u8, 0u8, 135u8), (1, 16), 16, 2);
            rustgame_core::draw::line(&mut s, (72u8, 227u8, 156u8, 49u8), (4, 30), (9, 24), 5);
            rustgame_core::draw::line(&mut s, (57u8, 29u8, 66u8, 255u8), (-3, -2), (9, 25), 1);
            s.scroll(-6, -20);
            rustgame_core::draw::line(&mut s, (136u8, 172u8, 76u8, 24u8), (1, 26), (0, 14), 2);
            rustgame_core::draw::ellipse(&mut s, (237u8, 221u8, 138u8, 255u8), Rect::new(4, -16, 3, 1), 4);
            s = rustgame_core::transform::flip(&s, false, true);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x9283FF309AE81D19, flaeche: s }
    }));
    v.push((317, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((20, 6), 0);
            s.fill((197u8, 9u8, 102u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (146u8, 168u8, 135u8, 224u8), Rect::new(6, 3, 15, 5), 0, 0);
            rustgame_core::draw::polygon(&mut s, (46u8, 152u8, 99u8, 188u8), &[(2, 9), (-4, -4), (5, 8), (-4, 8), (20, 8), (23, 3)], 0);
            rustgame_core::draw::circle(&mut s, (234u8, 128u8, 80u8, 248u8), (-1, 6), 11, 1);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xE4AE0C54E2C9A618, flaeche: s }
    }));
    v.push((318, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((28, 26), SRCALPHA);
            s.fill((84u8, 155u8, 214u8, 214u8));
            rustgame_core::draw::circle(&mut s, (99u8, 209u8, 84u8, 255u8), (-4, 1), 21, 3);
            let mut q = Surface::with_flags((16, 17), SRCALPHA);
            q = muster_auf(q, 16, 17);
            s.blit_ex(&q, (0, 27), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            {
                let mut teil = s.subsurface(Rect::new(2, 21, 8, 3));
                teil.fill((19u8, 38u8, 46u8, 255u8));
            }
            s.set_clip(Some(Rect::new(11, 2, 14, 24)));
            s.set_clip(Some(Rect::new(24, 21, 3, 3)));
            let mut q = Surface::with_flags((21, 19), SRCALPHA);
            q = muster_auf(q, 21, 19);
            q.set_alpha(Some(150));
            s.blit_ex(&q, (0, 19), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            rustgame_core::draw::circle(&mut s, (222u8, 228u8, 117u8, 255u8), (-2, 20), 16, 0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xFDDEE07FC8DC0EE9, flaeche: s }
    }));
    v.push((319, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((11, 7), 0);
            s.fill((204u8, 32u8, 110u8, 255u8));
            rustgame_core::draw::circle(&mut s, (83u8, 181u8, 89u8, 255u8), (2, 1), 9, 4);
            s.scroll(3, 0);
            s.fill_rect((190u8, 132u8, 203u8, 95u8), Rect::new(9, -4, 1, 8));
            rustgame_core::draw::polygon(&mut s, (200u8, 100u8, 189u8, 255u8), &[(13, 6), (6, -2), (11, 0), (-4, 4), (7, 10), (8, 10)], 0);
            rustgame_core::draw::rect_radius(&mut s, (191u8, 78u8, 35u8, 164u8), Rect::new(10, 2, 7, 9), 2, 4);
            rustgame_core::draw::line(&mut s, (4u8, 131u8, 114u8, 255u8), (-3, 1), (2, 10), 1);
            rustgame_core::draw::circle(&mut s, (35u8, 200u8, 91u8, 255u8), (9, 2), 6, 3);
            let mut q = Surface::with_flags((7, 7), 0);
            q = muster_auf(q, 7, 7);
            q.set_colorkey(Some((59u8, 40u8, 8u8)));
            s.blit_ex(&q, (-2, 5), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            s = rustgame_core::transform::flip(&s, false, false);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x8DAB0DE7A7470299, flaeche: s }
    }));
    v.push((320, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((20, 22), 0);
            s.fill((224u8, 164u8, 52u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(16, 11, 2, 9));
                teil.fill((254u8, 246u8, 122u8, 87u8));
            }
            let mut q = Surface::with_flags((2, 17), 0);
            q = muster_auf(q, 2, 17);
            s.blit_ex(&q, (4, 13), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            let mut q = Surface::with_flags((3, 7), SRCALPHA);
            q = muster_auf(q, 3, 7);
            s.blit_ex(&q, (10, 16), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            rustgame_core::draw::rect_radius(&mut s, (75u8, 193u8, 7u8, 189u8), Rect::new(15, 0, 14, 10), 4, 0);
            let mut q = Surface::with_flags((6, 15), SRCALPHA);
            q = muster_auf(q, 6, 15);
            s.blit_ex(&q, (18, 10), Some(Rect::new(-3, 16, 3, 8)), rustgame_core::blend::flags::BLEND_RGBA_MIN);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xC547254EE18A62A3, flaeche: s }
    }));
    v.push((321, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((14, 18), SRCALPHA);
            s.fill((174u8, 125u8, 232u8, 226u8));
            rustgame_core::draw::circle(&mut s, (232u8, 114u8, 7u8, 216u8), (12, 4), 13, 0);
            let mut q = Surface::with_flags((15, 17), SRCALPHA);
            q = muster_auf(q, 15, 17);
            q.set_alpha(Some(107));
            s.blit_ex(&q, (9, 16), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            s.fill_rect((96u8, 200u8, 122u8, 185u8), Rect::new(10, -7, 11, 6));
            let mut q = Surface::with_flags((10, 20), SRCALPHA);
            q = muster_auf(q, 10, 20);
            s.blit_ex(&q, (7, 11), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            s.set_clip(Some(Rect::new(2, 15, 9, 2)));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x106896474D3067F1, flaeche: s }
    }));
    v.push((322, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((28, 32), 0);
            s.fill((79u8, 32u8, 121u8, 255u8));
            s.scroll(8, -13);
            s.scroll(12, -4);
            rustgame_core::draw::ellipse(&mut s, (114u8, 241u8, 183u8, 183u8), Rect::new(6, -10, 28, 7), 3);
            s.scroll(20, -18);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x1B8953739E689C25, flaeche: s }
    }));
    v.push((323, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((29, 29), SRCALPHA);
            s.fill((227u8, 161u8, 164u8, 27u8));
            rustgame_core::draw::ellipse(&mut s, (249u8, 16u8, 131u8, 105u8), Rect::new(23, 8, 4, 32), 3);
            rustgame_core::draw::ellipse(&mut s, (230u8, 11u8, 36u8, 255u8), Rect::new(-5, 21, 27, 26), 1);
            s.set_clip(Some(Rect::new(21, 20, 2, 5)));
            rustgame_core::draw::rect_radius(&mut s, (232u8, 96u8, 198u8, 196u8), Rect::new(18, 7, 23, 8), 1, 6);
            rustgame_core::draw::rect_radius(&mut s, (110u8, 130u8, 107u8, 255u8), Rect::new(10, 7, 8, 0), 0, 3);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x90CAD2505DA66F1B, flaeche: s }
    }));
    v.push((324, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((24, 11), 0);
            s.fill((241u8, 46u8, 217u8, 255u8));
            let mut q = Surface::with_flags((2, 6), 0);
            q = muster_auf(q, 2, 6);
            q.set_alpha(Some(181));
            q.set_colorkey(Some((203u8, 150u8, 112u8)));
            s.blit_ex(&q, (8, -1), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            let mut q = Surface::with_flags((3, 13), SRCALPHA);
            q = muster_auf(q, 3, 13);
            q.set_alpha(Some(250));
            q.set_colorkey(Some((6u8, 255u8, 180u8)));
            s.blit_ex(&q, (-1, 5), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            s.fill_rect((17u8, 227u8, 170u8, 169u8), Rect::new(13, -1, 1, 3));
            rustgame_core::draw::ellipse(&mut s, (21u8, 81u8, 133u8, 137u8), Rect::new(-2, 4, 24, 7), 4);
            rustgame_core::draw::rect_radius(&mut s, (151u8, 152u8, 171u8, 255u8), Rect::new(12, 1, 2, 9), 0, 4);
            s.fill_rect((177u8, 145u8, 111u8, 255u8), Rect::new(17, -2, 19, 13));
            rustgame_core::draw::circle(&mut s, (180u8, 215u8, 23u8, 249u8), (6, 0), 7, 4);
            rustgame_core::draw::circle(&mut s, (26u8, 22u8, 242u8, 65u8), (15, 1), 20, 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x895CF8883B8D7D15, flaeche: s }
    }));
    v.push((325, true, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((23, 10), 0);
            s.fill((37u8, 87u8, 182u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(21, 9, 1, 1));
                teil.fill((71u8, 99u8, 228u8, 56u8));
            }
            let mut q = Surface::with_flags((4, 2), 0);
            q = muster_auf(q, 4, 2);
            s.blit_ex(&q, (-3, 8), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            let mut q = Surface::with_flags((4, 7), SRCALPHA);
            q = muster_auf(q, 4, 7);
            s.blit_ex(&q, (16, 6), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x1290AF055D23D20A, flaeche: s }
    }));
    v.push((326, true, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((9, 21), SRCALPHA);
            s.fill((38u8, 49u8, 134u8, 207u8));
            let mut q = Surface::with_flags((4, 5), SRCALPHA);
            q = muster_auf(q, 4, 5);
            q.set_alpha(Some(57));
            s.blit_ex(&q, (-4, 5), Some(Rect::new(-2, 4, 0, 6)), rustgame_core::blend::flags::BLEND_RGBA_SUB);
            rustgame_core::draw::ellipse(&mut s, (12u8, 193u8, 119u8, 228u8), Rect::new(10, 1, 8, 24), 1);
            rustgame_core::draw::rect_radius(&mut s, (225u8, 104u8, 162u8, 255u8), Rect::new(4, 14, 7, 14), 3, 1);
            let mut q = Surface::with_flags((7, 5), 0);
            q = muster_auf(q, 7, 5);
            s.blit_ex(&q, (1, 10), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x114231C567781389, flaeche: s }
    }));
    v.push((327, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 32), 0);
            s.fill((167u8, 65u8, 75u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (86u8, 91u8, 63u8, 59u8), Rect::new(-1, -4, 0, 0), 1);
            s.set_clip(Some(Rect::new(3, 17, 1, 1)));
            let mut q = Surface::with_flags((5, 28), 0);
            q = muster_auf(q, 5, 28);
            s.blit_ex(&q, (7, 10), Some(Rect::new(4, -11, 0, 25)), rustgame_core::blend::flags::BLEND_RGB_MULT);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xECFAF4B800BEA3A5, flaeche: s }
    }));
    v.push((328, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((26, 22), 0);
            s.fill((238u8, 246u8, 40u8, 255u8));
            let mut q = Surface::with_flags((16, 14), SRCALPHA);
            q = muster_auf(q, 16, 14);
            q.set_alpha(Some(162));
            s.blit_ex(&q, (16, 4), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            rustgame_core::draw::ellipse(&mut s, (63u8, 171u8, 93u8, 255u8), Rect::new(-2, 1, 25, 22), 0);
            let mut q = Surface::with_flags((3, 22), SRCALPHA);
            q = muster_auf(q, 3, 22);
            q.set_alpha(Some(178));
            s.blit_ex(&q, (3, 17), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            rustgame_core::draw::ellipse(&mut s, (133u8, 134u8, 219u8, 129u8), Rect::new(14, 19, 4, 5), 4);
            s.set_clip(Some(Rect::new(17, 2, 6, 16)));
            let mut q = Surface::with_flags((4, 5), 0);
            q = muster_auf(q, 4, 5);
            s.blit_ex(&q, (10, 0), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            s.set_clip(Some(Rect::new(20, 7, 1, 12)));
            s = s.convert_alpha();
            s = rustgame_core::transform::flip(&s, true, true);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xAD275091F3DA181C, flaeche: s }
    }));
    v.push((329, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((32, 21), SRCALPHA);
            s.fill((200u8, 140u8, 104u8, 255u8));
            s.fill_rect((243u8, 28u8, 48u8, 255u8), Rect::new(9, 1, 32, 14));
            {
                let mut teil = s.subsurface(Rect::new(6, 14, 4, 2));
                teil.fill((66u8, 126u8, 56u8, 255u8));
            }
            let mut q = Surface::with_flags((8, 6), SRCALPHA);
            q = muster_auf(q, 8, 6);
            q.set_alpha(Some(248));
            s.blit_ex(&q, (29, 4), Some(Rect::new(0, -1, 11, 0)), rustgame_core::blend::flags::BLEND_RGB_SUB);
            let mut q = Surface::with_flags((18, 2), SRCALPHA);
            q = muster_auf(q, 18, 2);
            q.set_alpha(Some(190));
            s.blit_ex(&q, (7, 8), Some(Rect::new(11, 2, 11, 5)), rustgame_core::blend::flags::BLEND_RGB_ADD);
            rustgame_core::draw::polygon(&mut s, (4u8, 253u8, 135u8, 53u8), &[(14, 2), (10, -2), (-4, 10), (5, 1), (17, 23), (25, 24)], 0);
            s.set_clip(Some(Rect::new(0, 2, 14, 3)));
            s.fill_rect((111u8, 63u8, 111u8, 34u8), Rect::new(25, -2, 10, 11));
            let mut q = Surface::with_flags((28, 6), 0);
            q = muster_auf(q, 28, 6);
            s.blit_ex(&q, (4, -4), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x91AF8C5C2C69E5B4, flaeche: s }
    }));
    v.push((330, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((25, 8), 0);
            s.fill((205u8, 8u8, 170u8, 255u8));
            let mut q = Surface::with_flags((27, 5), SRCALPHA);
            q = muster_auf(q, 27, 5);
            q.set_alpha(Some(217));
            q.set_colorkey(Some((13u8, 36u8, 27u8)));
            s.blit_ex(&q, (-1, 10), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            s.set_clip(Some(Rect::new(15, 6, 9, 1)));
            let mut q = Surface::with_flags((16, 6), SRCALPHA);
            q = muster_auf(q, 16, 6);
            s.blit_ex(&q, (-3, 10), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            rustgame_core::draw::line(&mut s, (69u8, 108u8, 151u8, 166u8), (4, 9), (20, 11), 1);
            rustgame_core::draw::ellipse(&mut s, (92u8, 53u8, 247u8, 99u8), Rect::new(21, 4, 22, 1), 2);
            s.fill_rect((92u8, 119u8, 131u8, 1u8), Rect::new(-8, 9, 26, 9));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x43182C8242491035, flaeche: s }
    }));
    v.push((331, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((32, 29), SRCALPHA);
            s.fill((52u8, 243u8, 239u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (221u8, 214u8, 20u8, 213u8), &[(23, 30), (14, 29), (34, 17), (22, 30), (10, 5)], 2);
            rustgame_core::draw::circle(&mut s, (182u8, 141u8, 20u8, 41u8), (-3, 24), 2, 1);
            let mut q = Surface::with_flags((14, 27), SRCALPHA);
            q = muster_auf(q, 14, 27);
            s.blit_ex(&q, (-2, 17), Some(Rect::new(14, -12, 6, 19)), rustgame_core::blend::flags::BLEND_RGBA_MAX);
            rustgame_core::draw::line(&mut s, (140u8, 52u8, 106u8, 176u8), (7, 28), (-1, 5), 3);
            {
                let mut teil = s.subsurface(Rect::new(8, 26, 6, 3));
                teil.fill((11u8, 145u8, 148u8, 255u8));
            }
            s.set_clip(Some(Rect::new(9, 21, 17, 4)));
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x27FCDA7E77C904AC, flaeche: s }
    }));
    v.push((332, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((13, 22), 0);
            s.fill((214u8, 201u8, 90u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(8, 7, 1, 10));
                teil.fill((103u8, 65u8, 195u8, 255u8));
            }
            s.fill_rect((107u8, 189u8, 47u8, 241u8), Rect::new(8, -2, 11, 7));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x21F904675CAC0E4D, flaeche: s }
    }));
    v.push((333, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((23, 17), SRCALPHA);
            s.fill((144u8, 174u8, 214u8, 42u8));
            {
                let mut teil = s.subsurface(Rect::new(10, 7, 7, 2));
                teil.fill((132u8, 197u8, 246u8, 255u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (233u8, 118u8, 246u8, 130u8), Rect::new(22, -1, 1, 9), 1, 5);
            s.fill_rect((10u8, 26u8, 141u8, 66u8), Rect::new(-4, 12, 13, 4));
            let mut q = Surface::with_flags((5, 9), SRCALPHA);
            q = muster_auf(q, 5, 9);
            s.blit_ex(&q, (25, 10), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xF0BF548B69BBC853, flaeche: s }
    }));
    v.push((334, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((13, 23), 0);
            s.fill((195u8, 215u8, 68u8, 255u8));
            rustgame_core::draw::circle(&mut s, (107u8, 5u8, 135u8, 255u8), (9, 23), 1, 4);
            rustgame_core::draw::circle(&mut s, (46u8, 46u8, 52u8, 255u8), (0, 26), 20, 2);
            {
                let mut teil = s.subsurface(Rect::new(9, 18, 2, 4));
                teil.fill((74u8, 191u8, 105u8, 98u8));
            }
            rustgame_core::draw::line(&mut s, (233u8, 108u8, 192u8, 255u8), (2, 23), (8, 24), 5);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xAD90CCD7A5D0A312, flaeche: s }
    }));
    v.push((335, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((10, 6), 0);
            s.fill((208u8, 28u8, 9u8, 255u8));
            let mut q = Surface::with_flags((8, 7), SRCALPHA);
            q = muster_auf(q, 8, 7);
            q.set_alpha(Some(241));
            s.blit_ex(&q, (-4, 5), Some(Rect::new(-2, 3, 7, 2)), rustgame_core::blend::flags::BLEND_RGBA_SUB);
            rustgame_core::draw::rect(&mut s, (54u8, 226u8, 46u8, 255u8), Rect::new(4, 2, 11, 5), 1);
            s.scroll(1, 0);
            s = rustgame_core::transform::rotate(&s, -259.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xC331E31BD7434C7E, flaeche: s }
    }));
    v.push((336, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((6, 30), 0);
            s.fill((99u8, 73u8, 187u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (252u8, 175u8, 32u8, 13u8), &[(8, 23), (-3, 27), (2, 32), (8, 15)], 1);
            rustgame_core::draw::line(&mut s, (175u8, 100u8, 139u8, 183u8), (4, 0), (-1, 9), 4);
            s.fill_rect((187u8, 6u8, 201u8, 111u8), Rect::new(3, 31, 9, 4));
            let mut q = Surface::with_flags((3, 16), 0);
            q = muster_auf(q, 3, 16);
            s.blit_ex(&q, (7, 8), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
            {
                let mut teil = s.subsurface(Rect::new(1, 23, 4, 7));
                teil.fill((234u8, 194u8, 43u8, 255u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (200u8, 248u8, 236u8, 255u8), Rect::new(7, 16, 6, 21), 1, 2);
            s.fill_rect((4u8, 176u8, 74u8, 22u8), Rect::new(6, 16, 7, 8));
            rustgame_core::draw::circle(&mut s, (240u8, 170u8, 142u8, 145u8), (8, 25), 2, 3);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xFDCF8F3498710BDC, flaeche: s }
    }));
    v.push((337, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 21), 0);
            s.fill((184u8, 6u8, 184u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (249u8, 12u8, 122u8, 255u8), Rect::new(3, -5, 0, 4), 1);
            rustgame_core::draw::rect_radius(&mut s, (253u8, 103u8, 182u8, 183u8), Rect::new(8, 1, 14, 5), 4, 6);
            let mut q = Surface::with_flags((10, 9), SRCALPHA);
            q = muster_auf(q, 10, 9);
            q.set_alpha(Some(60));
            s.blit_ex(&q, (15, 10), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            rustgame_core::draw::rect_radius(&mut s, (114u8, 70u8, 145u8, 149u8), Rect::new(13, 5, 1, 10), 2, 2);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xFFB3A4780243728D, flaeche: s }
    }));
    v.push((338, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((9, 19), SRCALPHA);
            s.fill((149u8, 239u8, 200u8, 30u8));
            rustgame_core::draw::circle(&mut s, (53u8, 22u8, 103u8, 255u8), (9, 17), 2, 2);
            s.set_clip(Some(Rect::new(2, 14, 3, 2)));
            rustgame_core::draw::ellipse(&mut s, (244u8, 206u8, 101u8, 157u8), Rect::new(1, 8, 2, 10), 3);
            let mut q = Surface::with_flags((7, 20), 0);
            q = muster_auf(q, 7, 20);
            s.blit_ex(&q, (0, 9), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xE745401853ECC4C3, flaeche: s }
    }));
    v.push((339, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((10, 32), 0);
            s.fill((64u8, 230u8, 17u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (241u8, 26u8, 254u8, 190u8), Rect::new(-3, 23, 6, 1), 1, 0);
            rustgame_core::draw::line(&mut s, (39u8, 252u8, 176u8, 255u8), (13, 29), (2, 7), 4);
            let mut q = Surface::with_flags((12, 20), 0);
            q = muster_auf(q, 12, 20);
            q.set_alpha(Some(227));
            s.blit_ex(&q, (7, 27), Some(Rect::new(-4, 2, 7, 13)), rustgame_core::blend::flags::BLEND_RGBA_MIN);
            rustgame_core::draw::circle(&mut s, (124u8, 98u8, 122u8, 255u8), (0, 3), 23, 3);
            s.fill_rect((40u8, 3u8, 73u8, 228u8), Rect::new(-1, -13, 6, 18));
            rustgame_core::draw::rect(&mut s, (46u8, 212u8, 41u8, 255u8), Rect::new(3, 17, 4, 25), 0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xC20C8EDD810AC7B6, flaeche: s }
    }));
    v.push((340, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((18, 24), SRCALPHA);
            s.fill((141u8, 91u8, 160u8, 60u8));
            rustgame_core::draw::circle(&mut s, (153u8, 79u8, 233u8, 130u8), (21, 4), 8, 2);
            rustgame_core::draw::rect_radius(&mut s, (22u8, 84u8, 250u8, 255u8), Rect::new(1, -4, 6, 8), 1, 6);
            s.set_clip(Some(Rect::new(8, 5, 7, 19)));
            s = s.convert();
            s = rustgame_core::transform::flip(&s, true, false);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xEB7BA1EB6B77993C, flaeche: s }
    }));
    v.push((341, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((24, 15), 0);
            s.fill((76u8, 30u8, 18u8, 255u8));
            rustgame_core::draw::rect(&mut s, (225u8, 172u8, 112u8, 128u8), Rect::new(-7, 15, 10, 12), 0);
            {
                let mut teil = s.subsurface(Rect::new(2, 4, 3, 7));
                teil.fill((146u8, 195u8, 241u8, 7u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (109u8, 130u8, 21u8, 138u8), Rect::new(22, 2, 24, 7), 2, 1);
            rustgame_core::draw::circle(&mut s, (161u8, 184u8, 189u8, 37u8), (3, -4), 12, 1);
            rustgame_core::draw::polygon(&mut s, (115u8, 236u8, 92u8, 143u8), &[(9, 7), (10, 10), (-4, 0), (1, 4), (-1, 7), (27, 10)], 3);
            rustgame_core::draw::circle(&mut s, (121u8, 71u8, 14u8, 255u8), (7, 14), 21, 1);
            rustgame_core::draw::polygon(&mut s, (104u8, 178u8, 249u8, 255u8), &[(19, 0), (24, 4), (2, 2), (14, 15), (18, 3)], 3);
            rustgame_core::draw::circle(&mut s, (27u8, 218u8, 180u8, 61u8), (13, 11), 4, 4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x3A6F9E7D462FDC8A, flaeche: s }
    }));
    v.push((342, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((9, 12), 0);
            s.fill((35u8, 36u8, 247u8, 255u8));
            s.fill_rect((19u8, 162u8, 253u8, 156u8), Rect::new(-4, -2, 10, 3));
            rustgame_core::draw::ellipse(&mut s, (8u8, 218u8, 154u8, 255u8), Rect::new(9, -2, 0, 8), 4);
            s.set_clip(Some(Rect::new(5, 9, 3, 1)));
            rustgame_core::draw::polygon(&mut s, (156u8, 202u8, 71u8, 255u8), &[(4, -1), (5, 6), (8, 8), (7, 13), (10, -1)], 0);
            rustgame_core::draw::circle(&mut s, (243u8, 247u8, 29u8, 255u8), (-2, 0), 0, 0);
            rustgame_core::draw::line(&mut s, (134u8, 186u8, 27u8, 231u8), (12, 6), (8, 0), 1);
            s.set_clip(Some(Rect::new(1, 7, 7, 5)));
            rustgame_core::draw::circle(&mut s, (123u8, 113u8, 0u8, 255u8), (7, 2), 11, 3);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x562A2BA6350EB8D1, flaeche: s }
    }));
    v.push((343, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 21), 0);
            s.fill((81u8, 232u8, 47u8, 255u8));
            s.scroll(-12, -5);
            s.scroll(-7, -13);
            {
                let mut teil = s.subsurface(Rect::new(1, 8, 5, 13));
                teil.fill((208u8, 247u8, 231u8, 244u8));
            }
            rustgame_core::draw::ellipse(&mut s, (245u8, 68u8, 176u8, 23u8), Rect::new(8, -8, 3, 4), 2);
            {
                let mut teil = s.subsurface(Rect::new(1, 2, 11, 13));
                teil.fill((44u8, 12u8, 252u8, 125u8));
            }
            rustgame_core::draw::circle(&mut s, (46u8, 41u8, 95u8, 11u8), (6, 0), 19, 4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x7EF801A73BCE8E41, flaeche: s }
    }));
    v.push((344, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((18, 18), SRCALPHA);
            s.fill((145u8, 207u8, 221u8, 167u8));
            rustgame_core::draw::rect_radius(&mut s, (221u8, 211u8, 117u8, 255u8), Rect::new(-4, 2, 4, 13), 3, 0);
            {
                let mut teil = s.subsurface(Rect::new(11, 10, 5, 2));
                teil.fill((18u8, 130u8, 51u8, 255u8));
            }
            let mut q = Surface::with_flags((16, 4), SRCALPHA);
            q = muster_auf(q, 16, 4);
            s.blit_ex(&q, (19, 6), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            {
                let mut teil = s.subsurface(Rect::new(13, 2, 2, 2));
                teil.fill((130u8, 116u8, 247u8, 255u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (113u8, 57u8, 27u8, 161u8), Rect::new(-4, 3, 20, 15), 4, 4);
            rustgame_core::draw::line(&mut s, (241u8, 13u8, 81u8, 163u8), (16, 13), (3, 20), 3);
            s.scroll(14, -2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x2C12AD4B356C472B, flaeche: s }
    }));
    v.push((345, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((30, 15), SRCALPHA);
            s.fill((96u8, 188u8, 76u8, 52u8));
            let mut q = Surface::with_flags((30, 11), SRCALPHA);
            q = muster_auf(q, 30, 11);
            q.set_alpha(Some(46));
            s.blit_ex(&q, (17, -4), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            let mut q = Surface::with_flags((29, 16), 0);
            q = muster_auf(q, 29, 16);
            s.blit_ex(&q, (13, 10), Some(Rect::new(14, 14, 9, 1)), rustgame_core::blend::flags::BLEND_RGBA_MAX);
            let mut q = Surface::with_flags((25, 4), 0);
            q = muster_auf(q, 25, 4);
            s.blit_ex(&q, (7, 0), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            rustgame_core::draw::rect_radius(&mut s, (190u8, 152u8, 119u8, 138u8), Rect::new(17, -4, 14, 15), 0, 3);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xAAC4C2901D55FB5A, flaeche: s }
    }));
    v.push((346, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((23, 12), 0);
            s.fill((59u8, 160u8, 230u8, 255u8));
            s.set_clip(Some(Rect::new(3, 1, 5, 7)));
            rustgame_core::draw::rect_radius(&mut s, (23u8, 75u8, 182u8, 142u8), Rect::new(13, 6, 18, 8), 3, 3);
            rustgame_core::draw::rect_radius(&mut s, (233u8, 51u8, 186u8, 65u8), Rect::new(-9, 8, 17, 10), 4, 5);
            {
                let mut teil = s.subsurface(Rect::new(21, 8, 1, 3));
                teil.fill((243u8, 68u8, 98u8, 29u8));
            }
            let mut q = Surface::with_flags((6, 11), 0);
            q = muster_auf(q, 6, 11);
            q.set_alpha(Some(3));
            s.blit_ex(&q, (20, 14), None, 0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xB7B820401640579D, flaeche: s }
    }));
    v.push((347, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((17, 25), SRCALPHA);
            s.fill((97u8, 122u8, 168u8, 253u8));
            s.set_clip(Some(Rect::new(1, 1, 1, 14)));
            rustgame_core::draw::ellipse(&mut s, (242u8, 114u8, 68u8, 170u8), Rect::new(-5, 20, 13, 21), 3);
            rustgame_core::draw::circle(&mut s, (159u8, 187u8, 151u8, 144u8), (11, 9), 14, 1);
            let mut q = Surface::with_flags((5, 21), SRCALPHA);
            q = muster_auf(q, 5, 21);
            s.blit_ex(&q, (2, 21), None, 0);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x64518A100C7F8579, flaeche: s }
    }));
    v.push((348, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((16, 30), 0);
            s.fill((108u8, 124u8, 233u8, 255u8));
            let mut q = Surface::with_flags((8, 31), SRCALPHA);
            q = muster_auf(q, 8, 31);
            s.blit_ex(&q, (6, 6), None, 0);
            rustgame_core::draw::rect_radius(&mut s, (112u8, 197u8, 238u8, 236u8), Rect::new(-3, 0, 12, 29), 4, 2);
            rustgame_core::draw::rect_radius(&mut s, (86u8, 185u8, 32u8, 90u8), Rect::new(5, 23, 3, 30), 3, 1);
            s.set_clip(Some(Rect::new(5, 18, 2, 4)));
            rustgame_core::draw::rect_radius(&mut s, (28u8, 115u8, 172u8, 240u8), Rect::new(6, 26, 6, 28), 4, 6);
            s.fill_rect((7u8, 229u8, 18u8, 255u8), Rect::new(-5, 5, 19, 5));
            s.fill_rect((30u8, 192u8, 253u8, 239u8), Rect::new(14, 1, 13, 5));
            let mut q = Surface::with_flags((18, 9), SRCALPHA);
            q = muster_auf(q, 18, 9);
            q.set_alpha(Some(236));
            s.blit_ex(&q, (-2, 29), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x9E54153DA1AEB988, flaeche: s }
    }));
    v.push((349, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((19, 22), SRCALPHA);
            s.fill((146u8, 170u8, 7u8, 136u8));
            s.scroll(-11, 8);
            {
                let mut teil = s.subsurface(Rect::new(9, 7, 5, 14));
                teil.fill((182u8, 176u8, 212u8, 189u8));
            }
            rustgame_core::draw::rect(&mut s, (49u8, 199u8, 37u8, 24u8), Rect::new(17, -8, 22, 16), 3);
            rustgame_core::draw::rect_radius(&mut s, (50u8, 143u8, 222u8, 183u8), Rect::new(-10, 18, 5, 13), 1, 1);
            let mut q = Surface::with_flags((19, 11), 0);
            q = muster_auf(q, 19, 11);
            q.set_alpha(Some(31));
            s.blit_ex(&q, (11, 4), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            s = s.convert_alpha();
            s = rustgame_core::transform::scale(&s, (14, 9));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x87A2F993456EB522, flaeche: s }
    }));
    v.push((350, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((17, 31), SRCALPHA);
            s.fill((181u8, 131u8, 89u8, 160u8));
            let mut q = Surface::with_flags((6, 33), SRCALPHA);
            q = muster_auf(q, 6, 33);
            q.set_colorkey(Some((186u8, 196u8, 16u8)));
            s.blit_ex(&q, (18, 26), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
            let mut q = Surface::with_flags((18, 29), 0);
            q = muster_auf(q, 18, 29);
            s.blit_ex(&q, (15, 7), Some(Rect::new(11, 4, 12, 21)), rustgame_core::blend::flags::BLEND_RGB_MAX);
            s.fill_rect((96u8, 94u8, 74u8, 196u8), Rect::new(-2, -13, 12, 12));
            rustgame_core::draw::polygon(&mut s, (216u8, 24u8, 216u8, 255u8), &[(8, 12), (-3, 14), (12, 34), (15, 26), (18, 1)], 2);
            rustgame_core::draw::rect(&mut s, (210u8, 134u8, 149u8, 255u8), Rect::new(16, 14, 13, 34), 3);
            rustgame_core::draw::circle(&mut s, (64u8, 10u8, 207u8, 34u8), (7, 27), 15, 4);
            s = rustgame_core::transform::scale(&s, (12, 12));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x1D870D8F30ABB52A, flaeche: s }
    }));
    v.push((351, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((25, 24), SRCALPHA);
            s.fill((151u8, 233u8, 199u8, 163u8));
            let mut q = Surface::with_flags((9, 10), 0);
            q = muster_auf(q, 9, 10);
            s.blit_ex(&q, (12, 18), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            let mut q = Surface::with_flags((4, 18), SRCALPHA);
            q = muster_auf(q, 4, 18);
            q.set_colorkey(Some((255u8, 138u8, 72u8)));
            s.blit_ex(&q, (6, 17), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            {
                let mut teil = s.subsurface(Rect::new(19, 13, 3, 11));
                teil.fill((84u8, 7u8, 103u8, 187u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xD0E21EDA6CF53FD4, flaeche: s }
    }));
    v.push((352, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((15, 10), 0);
            s.fill((138u8, 71u8, 100u8, 255u8));
            s.fill_rect((34u8, 58u8, 85u8, 58u8), Rect::new(-3, 4, 13, 8));
            s.fill_rect((28u8, 224u8, 223u8, 68u8), Rect::new(-3, 3, 1, 6));
            rustgame_core::draw::polygon(&mut s, (176u8, 73u8, 170u8, 255u8), &[(14, 2), (17, 7), (-1, 4), (6, -2), (7, 8)], 1);
            rustgame_core::draw::circle(&mut s, (9u8, 251u8, 153u8, 255u8), (-2, 7), 0, 4);
            rustgame_core::draw::rect_radius(&mut s, (213u8, 11u8, 14u8, 102u8), Rect::new(12, 11, 6, 0), 1, 5);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x58DE926660F969CB, flaeche: s }
    }));
    v.push((353, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((9, 8), SRCALPHA);
            s.fill((206u8, 53u8, 244u8, 254u8));
            rustgame_core::draw::rect_radius(&mut s, (106u8, 252u8, 161u8, 255u8), Rect::new(-3, 0, 0, 5), 4, 5);
            s.set_clip(Some(Rect::new(2, 7, 4, 1)));
            {
                let mut teil = s.subsurface(Rect::new(6, 7, 3, 1));
                teil.fill((185u8, 70u8, 238u8, 255u8));
            }
            let mut q = Surface::with_flags((2, 4), 0);
            q = muster_auf(q, 2, 4);
            q.set_colorkey(Some((250u8, 53u8, 75u8)));
            s.blit_ex(&q, (-3, 3), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            rustgame_core::draw::polygon(&mut s, (65u8, 170u8, 56u8, 172u8), &[(-2, 3), (12, 4), (-2, 4), (10, 8), (6, 3)], 3);
            s = rustgame_core::transform::rotate(&s, 161.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x4B988C0DB550FF32, flaeche: s }
    }));
    v.push((354, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 12), SRCALPHA);
            s.fill((77u8, 0u8, 54u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (146u8, 215u8, 245u8, 144u8), &[(1, 5), (14, 3), (7, 15), (11, 9), (3, 15), (11, 4)], 1);
            rustgame_core::draw::ellipse(&mut s, (6u8, 118u8, 53u8, 5u8), Rect::new(13, 11, 11, 14), 4);
            rustgame_core::draw::rect(&mut s, (95u8, 48u8, 47u8, 131u8), Rect::new(5, -5, 11, 7), 4);
            rustgame_core::draw::ellipse(&mut s, (169u8, 44u8, 28u8, 46u8), Rect::new(2, 9, 1, 2), 3);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x19E49E7DFBFF18E5, flaeche: s }
    }));
    v.push((355, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((11, 9), 0);
            s.fill((172u8, 26u8, 75u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (110u8, 199u8, 251u8, 72u8), Rect::new(4, -1, 7, 5), 1, 6);
            s.scroll(-9, 3);
            rustgame_core::draw::rect_radius(&mut s, (144u8, 218u8, 0u8, 255u8), Rect::new(-2, -4, 2, 4), 0, 6);
            {
                let mut teil = s.subsurface(Rect::new(8, 3, 1, 5));
                teil.fill((157u8, 213u8, 172u8, 42u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (74u8, 171u8, 180u8, 209u8), Rect::new(-4, 7, 11, 3), 0, 6);
            let mut q = Surface::with_flags((6, 3), 0);
            q = muster_auf(q, 6, 3);
            q.set_colorkey(Some((160u8, 34u8, 96u8)));
            s.blit_ex(&q, (-2, -2), Some(Rect::new(1, 4, 9, 4)), rustgame_core::blend::flags::BLEND_RGBA_MULT);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x3B304B0E5DDFD932, flaeche: s }
    }));
    v.push((356, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((26, 7), SRCALPHA);
            s.fill((223u8, 228u8, 208u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (207u8, 158u8, 181u8, 217u8), Rect::new(13, 2, 15, 4), 2, 1);
            s.set_clip(Some(Rect::new(3, 3, 3, 1)));
            let mut q = Surface::with_flags((2, 7), SRCALPHA);
            q = muster_auf(q, 2, 7);
            s.blit_ex(&q, (25, 5), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x126244614D31F479, flaeche: s }
    }));
    v.push((357, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((11, 17), 0);
            s.fill((32u8, 80u8, 123u8, 255u8));
            s.set_clip(Some(Rect::new(3, 12, 7, 2)));
            rustgame_core::draw::circle(&mut s, (18u8, 247u8, 59u8, 255u8), (5, 10), 13, 0);
            let mut q = Surface::with_flags((3, 14), 0);
            q = muster_auf(q, 3, 14);
            s.blit_ex(&q, (14, 6), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            s.set_clip(Some(Rect::new(0, 3, 9, 2)));
            s = rustgame_core::transform::scale(&s, (12, 15));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x348E8820001B2BCD, flaeche: s }
    }));
    v.push((358, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((30, 13), SRCALPHA);
            s.fill((8u8, 48u8, 38u8, 96u8));
            rustgame_core::draw::circle(&mut s, (161u8, 227u8, 177u8, 255u8), (6, 9), 16, 4);
            rustgame_core::draw::rect_radius(&mut s, (36u8, 250u8, 220u8, 255u8), Rect::new(24, -2, 12, 11), 0, 1);
            rustgame_core::draw::ellipse(&mut s, (157u8, 119u8, 165u8, 149u8), Rect::new(13, -6, 3, 16), 0);
            s = rustgame_core::transform::flip(&s, true, false);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x4F2779712146A31F, flaeche: s }
    }));
    v.push((359, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 22), SRCALPHA);
            s.fill((213u8, 11u8, 46u8, 151u8));
            rustgame_core::draw::ellipse(&mut s, (31u8, 232u8, 75u8, 17u8), Rect::new(-4, 23, 10, 6), 3);
            {
                let mut teil = s.subsurface(Rect::new(6, 11, 1, 7));
                teil.fill((107u8, 160u8, 15u8, 255u8));
            }
            s.fill_rect((1u8, 50u8, 136u8, 255u8), Rect::new(-2, 17, 0, 5));
            let mut q = Surface::with_flags((6, 22), 0);
            q = muster_auf(q, 6, 22);
            s.blit_ex(&q, (6, -4), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            s.scroll(4, 10);
            s.fill_rect((58u8, 168u8, 174u8, 107u8), Rect::new(1, 18, 8, 17));
            let mut q = Surface::with_flags((10, 14), SRCALPHA);
            q = muster_auf(q, 10, 14);
            q.set_alpha(Some(41));
            s.blit_ex(&q, (-2, 16), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x66D377BF2942B8D7, flaeche: s }
    }));
    v.push((360, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((13, 15), SRCALPHA);
            s.fill((74u8, 95u8, 40u8, 106u8));
            {
                let mut teil = s.subsurface(Rect::new(11, 1, 2, 8));
                teil.fill((196u8, 246u8, 140u8, 255u8));
            }
            rustgame_core::draw::polygon(&mut s, (150u8, 101u8, 163u8, 255u8), &[(10, -2), (16, 18), (2, 9), (9, 5)], 0);
            rustgame_core::draw::rect_radius(&mut s, (1u8, 174u8, 169u8, 255u8), Rect::new(2, -3, 14, 3), 0, 5);
            rustgame_core::draw::rect_radius(&mut s, (191u8, 32u8, 174u8, 255u8), Rect::new(-1, -5, 12, 5), 1, 6);
            s.fill_rect((4u8, 253u8, 141u8, 139u8), Rect::new(7, -1, 6, 2));
            s.set_clip(Some(Rect::new(0, 11, 7, 1)));
            s.scroll(-13, 12);
            rustgame_core::draw::rect(&mut s, (88u8, 49u8, 66u8, 255u8), Rect::new(1, 8, 14, 4), 2);
            s = s.convert();
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xBD8A2A6C3985298B, flaeche: s }
    }));
    v.push((361, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((25, 10), 0);
            s.fill((204u8, 204u8, 224u8, 255u8));
            rustgame_core::draw::line(&mut s, (84u8, 185u8, 194u8, 65u8), (15, 9), (-3, 10), 5);
            let mut q = Surface::with_flags((14, 6), SRCALPHA);
            q = muster_auf(q, 14, 6);
            q.set_alpha(Some(93));
            s.blit_ex(&q, (26, 8), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
            s = rustgame_core::transform::flip(&s, true, false);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x70FD88896F10F86D, flaeche: s }
    }));
    v.push((362, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((30, 22), 0);
            s.fill((186u8, 124u8, 183u8, 255u8));
            let mut q = Surface::with_flags((22, 11), SRCALPHA);
            q = muster_auf(q, 22, 11);
            s.blit_ex(&q, (33, 14), Some(Rect::new(15, 1, 18, 3)), 0);
            rustgame_core::draw::polygon(&mut s, (19u8, 116u8, 195u8, 18u8), &[(12, 3), (4, 22), (-2, 13), (2, 17)], 1);
            let mut q = Surface::with_flags((22, 18), 0);
            q = muster_auf(q, 22, 18);
            s.blit_ex(&q, (10, 21), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            s.fill_rect((62u8, 108u8, 7u8, 255u8), Rect::new(29, 1, 21, 14));
            rustgame_core::draw::circle(&mut s, (98u8, 10u8, 47u8, 255u8), (6, 7), 24, 4);
            rustgame_core::draw::polygon(&mut s, (227u8, 44u8, 27u8, 159u8), &[(2, 23), (-2, 14), (24, 19), (-3, 1), (19, 9), (32, 11)], 2);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x0CDD1FBC403A52C4, flaeche: s }
    }));
    v.push((363, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((30, 11), 0);
            s.fill((80u8, 152u8, 34u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (215u8, 72u8, 127u8, 36u8), Rect::new(8, 0, 3, 13), 0, 2);
            let mut q = Surface::with_flags((6, 12), 0);
            q = muster_auf(q, 6, 12);
            s.blit_ex(&q, (17, 9), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xF9BF2E01B0E7A82F, flaeche: s }
    }));
    v.push((364, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((31, 6), SRCALPHA);
            s.fill((91u8, 171u8, 117u8, 13u8));
            rustgame_core::draw::rect_radius(&mut s, (94u8, 23u8, 137u8, 98u8), Rect::new(-7, -2, 9, 0), 1, 5);
            let mut q = Surface::with_flags((24, 8), 0);
            q = muster_auf(q, 24, 8);
            q.set_colorkey(Some((24u8, 157u8, 219u8)));
            s.blit_ex(&q, (34, 2), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            rustgame_core::draw::rect_radius(&mut s, (116u8, 223u8, 149u8, 255u8), Rect::new(-4, 7, 34, 2), 1, 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x42FDC8F4D30E1B05, flaeche: s }
    }));
    v.push((365, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((27, 15), 0);
            s.fill((76u8, 146u8, 99u8, 255u8));
            let mut q = Surface::with_flags((14, 7), SRCALPHA);
            q = muster_auf(q, 14, 7);
            q.set_colorkey(Some((226u8, 146u8, 64u8)));
            s.blit_ex(&q, (20, 6), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            {
                let mut teil = s.subsurface(Rect::new(23, 10, 3, 2));
                teil.fill((20u8, 6u8, 77u8, 163u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (122u8, 65u8, 75u8, 134u8), Rect::new(-9, -8, 8, 11), 4, 5);
            rustgame_core::draw::line(&mut s, (16u8, 183u8, 230u8, 248u8), (14, 1), (22, 2), 1);
            {
                let mut teil = s.subsurface(Rect::new(10, 6, 2, 3));
                teil.fill((250u8, 120u8, 201u8, 255u8));
            }
            s.set_clip(Some(Rect::new(12, 12, 2, 3)));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xA4624B14D436402B, flaeche: s }
    }));
    v.push((366, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 21), 0);
            s.fill((61u8, 74u8, 196u8, 255u8));
            let mut q = Surface::with_flags((4, 5), SRCALPHA);
            q = muster_auf(q, 4, 5);
            q.set_colorkey(Some((81u8, 130u8, 118u8)));
            s.blit_ex(&q, (0, 18), Some(Rect::new(1, -2, 1, 3)), rustgame_core::blend::flags::BLEND_RGBA_ADD);
            let mut q = Surface::with_flags((2, 9), SRCALPHA);
            q = muster_auf(q, 2, 9);
            q.set_alpha(Some(56));
            q.set_colorkey(Some((232u8, 9u8, 180u8)));
            s.blit_ex(&q, (6, 14), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            {
                let mut teil = s.subsurface(Rect::new(2, 17, 2, 3));
                teil.fill((210u8, 88u8, 7u8, 255u8));
            }
            s.set_clip(Some(Rect::new(1, 4, 7, 14)));
            rustgame_core::draw::circle(&mut s, (147u8, 211u8, 233u8, 255u8), (-4, 5), 18, 4);
            rustgame_core::draw::line(&mut s, (71u8, 76u8, 233u8, 255u8), (9, -2), (-2, 14), 3);
            s.fill_rect((41u8, 109u8, 57u8, 224u8), Rect::new(-3, 20, 10, 20));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x264C49238A222E29, flaeche: s }
    }));
    v.push((367, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 22), 0);
            s.fill((249u8, 11u8, 176u8, 255u8));
            s.fill_rect((205u8, 228u8, 14u8, 255u8), Rect::new(-4, 3, 0, 14));
            {
                let mut teil = s.subsurface(Rect::new(4, 8, 2, 4));
                teil.fill((89u8, 150u8, 194u8, 255u8));
            }
            s.set_clip(Some(Rect::new(5, 5, 1, 3)));
            s = rustgame_core::transform::grayscale(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x7F62FDEF0CF6117D, flaeche: s }
    }));
    v.push((368, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 24), SRCALPHA);
            s.fill((52u8, 243u8, 142u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (51u8, 215u8, 172u8, 58u8), &[(21, 22), (11, -4), (22, 11), (23, 26), (0, 4)], 3);
            {
                let mut teil = s.subsurface(Rect::new(15, 13, 6, 5));
                teil.fill((251u8, 201u8, 26u8, 26u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (153u8, 125u8, 98u8, 44u8), Rect::new(2, -7, 14, 26), 0, 3);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xA1889C34FDF0186B, flaeche: s }
    }));
    v.push((369, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((30, 17), 0);
            s.fill((159u8, 122u8, 16u8, 255u8));
            let mut q = Surface::with_flags((18, 5), 0);
            q = muster_auf(q, 18, 5);
            s.blit_ex(&q, (31, 11), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            let mut q = Surface::with_flags((12, 2), 0);
            q = muster_auf(q, 12, 2);
            q.set_alpha(Some(163));
            s.blit_ex(&q, (27, -2), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xD0CEEE47B8663B05, flaeche: s }
    }));
    v.push((370, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((9, 26), 0);
            s.fill((147u8, 143u8, 43u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (5u8, 246u8, 25u8, 255u8), Rect::new(5, 25, 9, 9), 4, 2);
            s.scroll(-7, 5);
            let mut q = Surface::with_flags((10, 28), 0);
            q = muster_auf(q, 10, 28);
            s.blit_ex(&q, (4, 5), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            s.set_clip(Some(Rect::new(3, 21, 5, 5)));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xADF3C160FF4A03A1, flaeche: s }
    }));
    v.push((371, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((15, 25), SRCALPHA);
            s.fill((123u8, 162u8, 25u8, 49u8));
            rustgame_core::draw::rect_radius(&mut s, (3u8, 60u8, 64u8, 201u8), Rect::new(14, -1, 15, 25), 0, 2);
            {
                let mut teil = s.subsurface(Rect::new(1, 23, 4, 2));
                teil.fill((51u8, 69u8, 184u8, 186u8));
            }
            rustgame_core::draw::polygon(&mut s, (169u8, 164u8, 21u8, 146u8), &[(-2, 11), (6, 25), (7, 4), (3, 8)], 1);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x39BC97032D70A278, flaeche: s }
    }));
    v.push((372, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((9, 28), 0);
            s.fill((188u8, 124u8, 46u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(7, 13, 1, 7));
                teil.fill((152u8, 124u8, 189u8, 255u8));
            }
            s.fill_rect((108u8, 216u8, 77u8, 155u8), Rect::new(6, 4, 10, 13));
            rustgame_core::draw::polygon(&mut s, (72u8, 128u8, 135u8, 255u8), &[(1, 25), (2, -2), (6, 3), (11, 2), (-3, 31)], 3);
            s.fill_rect((149u8, 42u8, 241u8, 255u8), Rect::new(10, -8, 7, 11));
            rustgame_core::draw::circle(&mut s, (121u8, 193u8, 188u8, 42u8), (1, 24), 13, 0);
            rustgame_core::draw::circle(&mut s, (218u8, 205u8, 213u8, 255u8), (-2, 21), 5, 0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x605DD1E987C4F3BE, flaeche: s }
    }));
    v.push((373, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 9), 0);
            s.fill((61u8, 99u8, 124u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (237u8, 153u8, 235u8, 223u8), Rect::new(-3, 6, 10, 0), 2, 0);
            s.fill_rect((85u8, 70u8, 182u8, 255u8), Rect::new(1, -3, 5, 3));
            rustgame_core::draw::polygon(&mut s, (228u8, 135u8, 138u8, 232u8), &[(-1, 7), (10, 1), (2, -1)], 2);
            {
                let mut teil = s.subsurface(Rect::new(5, 8, 1, 1));
                teil.fill((57u8, 82u8, 72u8, 15u8));
            }
            let mut q = Surface::with_flags((5, 10), SRCALPHA);
            q = muster_auf(q, 5, 10);
            q.set_alpha(Some(232));
            s.blit_ex(&q, (0, 0), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            rustgame_core::draw::circle(&mut s, (41u8, 99u8, 41u8, 78u8), (6, -3), 0, 3);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xB76F9723B725CC40, flaeche: s }
    }));
    v.push((374, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((16, 10), 0);
            s.fill((67u8, 72u8, 122u8, 255u8));
            let mut q = Surface::with_flags((10, 8), 0);
            q = muster_auf(q, 10, 8);
            s.blit_ex(&q, (5, 13), Some(Rect::new(10, 4, 2, 5)), rustgame_core::blend::flags::BLEND_RGBA_SUB);
            rustgame_core::draw::circle(&mut s, (117u8, 24u8, 2u8, 142u8), (5, 2), 1, 0);
            s.fill_rect((182u8, 94u8, 40u8, 255u8), Rect::new(12, 2, 14, 2));
            rustgame_core::draw::circle(&mut s, (120u8, 82u8, 190u8, 137u8), (7, 6), 15, 2);
            s.set_clip(Some(Rect::new(13, 4, 3, 6)));
            s.fill_rect((0u8, 210u8, 91u8, 85u8), Rect::new(14, 3, 16, 11));
            {
                let mut teil = s.subsurface(Rect::new(5, 3, 11, 6));
                teil.fill((154u8, 94u8, 189u8, 218u8));
            }
            {
                let mut teil = s.subsurface(Rect::new(10, 1, 6, 2));
                teil.fill((92u8, 217u8, 238u8, 255u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x4B70A3A7136BDF3D, flaeche: s }
    }));
    v.push((375, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((13, 17), SRCALPHA);
            s.fill((91u8, 160u8, 93u8, 135u8));
            rustgame_core::draw::circle(&mut s, (91u8, 205u8, 19u8, 88u8), (-2, -4), 14, 3);
            s.scroll(-10, -12);
            let mut q = Surface::with_flags((5, 4), 0);
            q = muster_auf(q, 5, 4);
            q.set_alpha(Some(69));
            s.blit_ex(&q, (8, 9), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            s.set_clip(Some(Rect::new(2, 13, 7, 1)));
            rustgame_core::draw::circle(&mut s, (144u8, 164u8, 255u8, 90u8), (-1, -2), 6, 0);
            let mut q = Surface::with_flags((2, 4), SRCALPHA);
            q = muster_auf(q, 2, 4);
            s.blit_ex(&q, (15, 6), Some(Rect::new(2, 1, 1, 3)), rustgame_core::blend::flags::BLEND_RGB_MIN);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x53C14B685D15DF29, flaeche: s }
    }));
    v.push((376, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((27, 29), 0);
            s.fill((241u8, 148u8, 83u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (0u8, 229u8, 167u8, 92u8), Rect::new(-4, 4, 20, 3), 1, 6);
            rustgame_core::draw::ellipse(&mut s, (43u8, 93u8, 57u8, 91u8), Rect::new(-14, 7, 21, 32), 3);
            rustgame_core::draw::circle(&mut s, (103u8, 19u8, 221u8, 87u8), (-1, 32), 11, 4);
            rustgame_core::draw::ellipse(&mut s, (170u8, 226u8, 152u8, 101u8), Rect::new(23, 0, 12, 17), 1);
            s.scroll(-2, 16);
            rustgame_core::draw::ellipse(&mut s, (5u8, 136u8, 236u8, 255u8), Rect::new(24, 0, 9, 6), 4);
            rustgame_core::draw::polygon(&mut s, (46u8, 200u8, 121u8, 255u8), &[(29, 14), (14, 26), (19, -1), (-2, 31), (2, 25)], 0);
            s = rustgame_core::transform::rotate(&s, 364.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xEEA87D46A48490E2, flaeche: s }
    }));
    v.push((377, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((13, 22), SRCALPHA);
            s.fill((9u8, 245u8, 125u8, 41u8));
            let mut q = Surface::with_flags((12, 7), SRCALPHA);
            q = muster_auf(q, 12, 7);
            q.set_alpha(Some(146));
            q.set_colorkey(Some((141u8, 77u8, 5u8)));
            s.blit_ex(&q, (10, 16), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            rustgame_core::draw::circle(&mut s, (228u8, 237u8, 17u8, 55u8), (3, -3), 10, 0);
            {
                let mut teil = s.subsurface(Rect::new(12, 20, 1, 2));
                teil.fill((237u8, 84u8, 167u8, 218u8));
            }
            rustgame_core::draw::ellipse(&mut s, (132u8, 12u8, 182u8, 255u8), Rect::new(14, -7, 11, 22), 4);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x74DCF86E594BBB01, flaeche: s }
    }));
    v.push((378, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((6, 15), 0);
            s.fill((232u8, 74u8, 225u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (117u8, 172u8, 48u8, 75u8), Rect::new(6, 13, 7, 0), 0);
            rustgame_core::draw::line(&mut s, (205u8, 188u8, 250u8, 255u8), (4, 11), (6, 1), 2);
            rustgame_core::draw::rect_radius(&mut s, (218u8, 63u8, 56u8, 252u8), Rect::new(1, -4, 8, 4), 4, 6);
            s.scroll(-6, -13);
            s.fill_rect((13u8, 67u8, 220u8, 190u8), Rect::new(3, 14, 7, 2));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x84EC6F9134E1CF4E, flaeche: s }
    }));
    v.push((379, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((6, 12), 0);
            s.fill((168u8, 160u8, 233u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (200u8, 79u8, 56u8, 56u8), &[(5, 5), (0, 13), (0, 7), (9, 15), (8, -1), (2, 7)], 2);
            rustgame_core::draw::circle(&mut s, (177u8, 189u8, 237u8, 255u8), (-2, 7), 2, 2);
            let mut q = Surface::with_flags((8, 7), 0);
            q = muster_auf(q, 8, 7);
            q.set_alpha(Some(148));
            q.set_colorkey(Some((80u8, 190u8, 239u8)));
            s.blit_ex(&q, (7, 6), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            s.scroll(0, -8);
            rustgame_core::draw::rect_radius(&mut s, (74u8, 45u8, 118u8, 255u8), Rect::new(-2, -5, 7, 6), 2, 0);
            rustgame_core::draw::circle(&mut s, (72u8, 111u8, 104u8, 216u8), (-4, -2), 5, 2);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x2F3FFBE84422B849, flaeche: s }
    }));
    v.push((380, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((10, 16), 0);
            s.fill((208u8, 224u8, 115u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (134u8, 232u8, 134u8, 255u8), Rect::new(7, 0, 9, 17), 4, 1);
            rustgame_core::draw::line(&mut s, (124u8, 192u8, 192u8, 80u8), (0, 3), (-2, 10), 1);
            s.fill_rect((23u8, 202u8, 145u8, 255u8), Rect::new(-2, 11, 7, 5));
            s.fill_rect((48u8, 160u8, 121u8, 180u8), Rect::new(1, 13, 13, 5));
            let mut q = Surface::with_flags((5, 5), 0);
            q = muster_auf(q, 5, 5);
            q.set_colorkey(Some((84u8, 247u8, 176u8)));
            s.blit_ex(&q, (6, 10), Some(Rect::new(6, 5, 1, 2)), rustgame_core::blend::flags::BLEND_RGB_SUB);
            let mut q = Surface::with_flags((2, 14), 0);
            q = muster_auf(q, 2, 14);
            q.set_alpha(Some(0));
            q.set_colorkey(Some((213u8, 174u8, 141u8)));
            s.blit_ex(&q, (2, 13), Some(Rect::new(2, -3, 5, 14)), rustgame_core::blend::flags::BLEND_RGBA_MAX);
            rustgame_core::draw::rect_radius(&mut s, (112u8, 108u8, 97u8, 255u8), Rect::new(11, -8, 0, 1), 4, 3);
            s = s.convert();
            s = rustgame_core::transform::rotate(&s, 174.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x8B4EA19AE106196B, flaeche: s }
    }));
    v.push((381, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 21), 0);
            s.fill((14u8, 103u8, 54u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (119u8, 198u8, 112u8, 137u8), Rect::new(18, 2, 0, 0), 4, 5);
            rustgame_core::draw::circle(&mut s, (173u8, 222u8, 65u8, 5u8), (8, 10), 14, 4);
            let mut q = Surface::with_flags((22, 23), 0);
            q = muster_auf(q, 22, 23);
            q.set_colorkey(Some((199u8, 26u8, 72u8)));
            s.blit_ex(&q, (4, 24), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            s.scroll(-17, -4);
            s = rustgame_core::transform::grayscale(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x08461854EE14B63F, flaeche: s }
    }));
    v.push((382, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 14), SRCALPHA);
            s.fill((23u8, 172u8, 250u8, 255u8));
            rustgame_core::draw::line(&mut s, (173u8, 162u8, 16u8, 255u8), (6, 10), (-1, 16), 2);
            let mut q = Surface::with_flags((4, 7), SRCALPHA);
            q = muster_auf(q, 4, 7);
            q.set_alpha(Some(65));
            q.set_colorkey(Some((163u8, 215u8, 140u8)));
            s.blit_ex(&q, (-4, 13), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            let mut q = Surface::with_flags((7, 2), SRCALPHA);
            q = muster_auf(q, 7, 2);
            s.blit_ex(&q, (-4, 0), Some(Rect::new(-1, 2, 1, 4)), rustgame_core::blend::flags::BLEND_RGB_MAX);
            rustgame_core::draw::line(&mut s, (187u8, 74u8, 154u8, 255u8), (-2, 5), (-2, 9), 2);
            rustgame_core::draw::ellipse(&mut s, (161u8, 237u8, 201u8, 255u8), Rect::new(5, 9, 5, 9), 4);
            {
                let mut teil = s.subsurface(Rect::new(3, 13, 2, 1));
                teil.fill((21u8, 53u8, 0u8, 186u8));
            }
            rustgame_core::draw::ellipse(&mut s, (39u8, 63u8, 211u8, 255u8), Rect::new(5, 7, 3, 1), 4);
            rustgame_core::draw::rect_radius(&mut s, (208u8, 39u8, 229u8, 255u8), Rect::new(8, 10, 9, 17), 3, 5);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x9B92ED4FE483E86B, flaeche: s }
    }));
    v.push((383, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((22, 22), SRCALPHA);
            s.fill((55u8, 188u8, 153u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(2, 6, 9, 4));
                teil.fill((106u8, 147u8, 236u8, 154u8));
            }
            let mut q = Surface::with_flags((20, 20), SRCALPHA);
            q = muster_auf(q, 20, 20);
            q.set_alpha(Some(254));
            s.blit_ex(&q, (2, 2), Some(Rect::new(11, 2, 9, 1)), 0);
            s.set_clip(Some(Rect::new(15, 5, 7, 8)));
            s.set_clip(Some(Rect::new(20, 17, 2, 4)));
            let mut q = Surface::with_flags((7, 3), 0);
            q = muster_auf(q, 7, 3);
            q.set_alpha(Some(14));
            q.set_colorkey(Some((45u8, 37u8, 53u8)));
            s.blit_ex(&q, (2, 9), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            let mut q = Surface::with_flags((14, 20), SRCALPHA);
            q = muster_auf(q, 14, 20);
            q.set_alpha(Some(247));
            q.set_colorkey(Some((238u8, 183u8, 6u8)));
            s.blit_ex(&q, (10, 24), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            {
                let mut teil = s.subsurface(Rect::new(5, 4, 5, 2));
                teil.fill((50u8, 114u8, 59u8, 255u8));
            }
            s = rustgame_core::transform::scale(&s, (10, 12));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xF531C0D5ECBDC330, flaeche: s }
    }));
    v.push((384, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((26, 32), SRCALPHA);
            s.fill((191u8, 124u8, 224u8, 12u8));
            s.set_clip(Some(Rect::new(17, 14, 7, 10)));
            s.fill_rect((93u8, 145u8, 220u8, 255u8), Rect::new(12, 15, 15, 25));
            rustgame_core::draw::line(&mut s, (142u8, 129u8, 102u8, 207u8), (14, 7), (27, 7), 4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x5E87203B7FFAFBD9, flaeche: s }
    }));
    v.push((385, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 32), SRCALPHA);
            s.fill((194u8, 104u8, 192u8, 8u8));
            let mut q = Surface::with_flags((4, 17), 0);
            q = muster_auf(q, 4, 17);
            s.blit_ex(&q, (1, -3), Some(Rect::new(0, 4, 4, 0)), rustgame_core::blend::flags::BLEND_RGB_SUB);
            rustgame_core::draw::line(&mut s, (67u8, 2u8, 142u8, 131u8), (-1, 33), (-1, 2), 3);
            let mut q = Surface::with_flags((8, 17), SRCALPHA);
            q = muster_auf(q, 8, 17);
            s.blit_ex(&q, (4, 30), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            s.fill_rect((148u8, 72u8, 217u8, 171u8), Rect::new(5, 23, 4, 2));
            rustgame_core::draw::rect_radius(&mut s, (70u8, 169u8, 186u8, 191u8), Rect::new(2, 18, 1, 2), 3, 5);
            s.set_clip(Some(Rect::new(3, 25, 3, 1)));
            rustgame_core::draw::line(&mut s, (250u8, 158u8, 195u8, 84u8), (2, 35), (6, 15), 4);
            rustgame_core::draw::circle(&mut s, (57u8, 17u8, 109u8, 50u8), (10, 4), 28, 0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x289514C624FCBD16, flaeche: s }
    }));
    v.push((386, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((25, 15), 0);
            s.fill((226u8, 213u8, 25u8, 255u8));
            s.fill_rect((130u8, 129u8, 122u8, 127u8), Rect::new(5, 7, 10, 4));
            rustgame_core::draw::polygon(&mut s, (42u8, 67u8, 102u8, 12u8), &[(4, 3), (24, 4), (3, 10), (10, 12)], 0);
            rustgame_core::draw::ellipse(&mut s, (192u8, 172u8, 82u8, 255u8), Rect::new(-11, 6, 17, 16), 4);
            let mut q = Surface::with_flags((26, 3), 0);
            q = muster_auf(q, 26, 3);
            s.blit_ex(&q, (-1, 13), Some(Rect::new(20, 2, 17, 0)), rustgame_core::blend::flags::BLEND_RGB_MIN);
            rustgame_core::draw::rect_radius(&mut s, (77u8, 243u8, 35u8, 167u8), Rect::new(6, -3, 20, 15), 2, 1);
            rustgame_core::draw::rect_radius(&mut s, (7u8, 22u8, 206u8, 184u8), Rect::new(8, 1, 14, 13), 4, 0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xF148EAEFBD1F3B47, flaeche: s }
    }));
    v.push((387, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((20, 12), SRCALPHA);
            s.fill((98u8, 190u8, 137u8, 109u8));
            rustgame_core::draw::ellipse(&mut s, (116u8, 44u8, 44u8, 255u8), Rect::new(5, 10, 13, 9), 0);
            rustgame_core::draw::line(&mut s, (245u8, 11u8, 247u8, 73u8), (0, -1), (3, 12), 5);
            rustgame_core::draw::polygon(&mut s, (73u8, 104u8, 95u8, 255u8), &[(-3, -1), (3, -2), (14, 15)], 2);
            s.scroll(-4, 7);
            rustgame_core::draw::rect_radius(&mut s, (7u8, 18u8, 15u8, 185u8), Rect::new(-3, 5, 4, 7), 0, 5);
            s.fill_rect((216u8, 220u8, 178u8, 255u8), Rect::new(18, 7, 0, 1));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x7818AB0931B126EF, flaeche: s }
    }));
    v.push((388, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((31, 17), SRCALPHA);
            s.fill((42u8, 226u8, 121u8, 92u8));
            let mut q = Surface::with_flags((29, 8), 0);
            q = muster_auf(q, 29, 8);
            q.set_colorkey(Some((100u8, 75u8, 157u8)));
            s.blit_ex(&q, (28, 19), Some(Rect::new(2, 9, 21, 5)), rustgame_core::blend::flags::BLEND_RGBA_ADD);
            s.fill_rect((5u8, 67u8, 133u8, 255u8), Rect::new(-1, 17, 23, 3));
            s.fill_rect((238u8, 240u8, 15u8, 186u8), Rect::new(-3, 18, 18, 0));
            s = rustgame_core::transform::rotate(&s, 7.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x04611FB44F43B0CD, flaeche: s }
    }));
    v.push((389, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((27, 26), SRCALPHA);
            s.fill((124u8, 241u8, 199u8, 112u8));
            let mut q = Surface::with_flags((25, 9), SRCALPHA);
            q = muster_auf(q, 25, 9);
            q.set_alpha(Some(151));
            s.blit_ex(&q, (18, 25), Some(Rect::new(-13, 4, 4, 6)), rustgame_core::blend::flags::BLEND_RGBA_MIN);
            let mut q = Surface::with_flags((9, 28), 0);
            q = muster_auf(q, 9, 28);
            s.blit_ex(&q, (21, 21), None, 0);
            s.set_clip(Some(Rect::new(15, 22, 9, 3)));
            s = rustgame_core::transform::flip(&s, true, false);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x235496439B6473AD, flaeche: s }
    }));
    v.push((390, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((24, 24), SRCALPHA);
            s.fill((6u8, 111u8, 160u8, 232u8));
            {
                let mut teil = s.subsurface(Rect::new(21, 16, 1, 6));
                teil.fill((44u8, 155u8, 34u8, 255u8));
            }
            rustgame_core::draw::circle(&mut s, (96u8, 87u8, 240u8, 159u8), (0, -3), 20, 0);
            let mut q = Surface::with_flags((8, 7), 0);
            q = muster_auf(q, 8, 7);
            s.blit_ex(&q, (20, 5), Some(Rect::new(5, 7, 10, 10)), rustgame_core::blend::flags::BLEND_RGB_MIN);
            s.fill_rect((124u8, 251u8, 212u8, 64u8), Rect::new(5, 21, 21, 24));
            s.fill_rect((152u8, 19u8, 217u8, 255u8), Rect::new(14, 12, 12, 24));
            rustgame_core::draw::rect_radius(&mut s, (198u8, 79u8, 83u8, 255u8), Rect::new(-5, 16, 14, 13), 3, 4);
            s.set_clip(Some(Rect::new(0, 5, 13, 7)));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x79FE0E48D3CF375D, flaeche: s }
    }));
    v.push((391, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 11), 0);
            s.fill((123u8, 75u8, 83u8, 255u8));
            s.fill_rect((122u8, 116u8, 217u8, 23u8), Rect::new(13, -3, 7, 10));
            rustgame_core::draw::line(&mut s, (189u8, 140u8, 147u8, 208u8), (16, 2), (23, 9), 4);
            let mut q = Surface::with_flags((4, 2), SRCALPHA);
            q = muster_auf(q, 4, 2);
            s.blit_ex(&q, (5, 12), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            let mut q = Surface::with_flags((9, 12), 0);
            q = muster_auf(q, 9, 12);
            q.set_colorkey(Some((239u8, 153u8, 213u8)));
            s.blit_ex(&q, (13, -2), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            let mut q = Surface::with_flags((11, 6), 0);
            q = muster_auf(q, 11, 6);
            s.blit_ex(&q, (6, 6), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xDDF2C216EFD80099, flaeche: s }
    }));
    v.push((392, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((25, 20), SRCALPHA);
            s.fill((250u8, 108u8, 163u8, 104u8));
            rustgame_core::draw::rect_radius(&mut s, (111u8, 154u8, 71u8, 157u8), Rect::new(-11, -6, 4, 14), 0, 4);
            let mut q = Surface::with_flags((20, 5), 0);
            q = muster_auf(q, 20, 5);
            q.set_colorkey(Some((89u8, 207u8, 105u8)));
            s.blit_ex(&q, (8, 13), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            rustgame_core::draw::ellipse(&mut s, (124u8, 136u8, 59u8, 30u8), Rect::new(-6, 8, 26, 11), 0);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x58451F1532493EF9, flaeche: s }
    }));
    v.push((393, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((17, 16), SRCALPHA);
            s.fill((212u8, 234u8, 70u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(2, 0, 1, 8));
                teil.fill((103u8, 234u8, 8u8, 104u8));
            }
            let mut q = Surface::with_flags((9, 17), 0);
            q = muster_auf(q, 9, 17);
            s.blit_ex(&q, (12, 11), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x3F92AE50721D47CF, flaeche: s }
    }));
    v.push((394, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((13, 13), SRCALPHA);
            s.fill((39u8, 168u8, 185u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(11, 6, 1, 6));
                teil.fill((43u8, 242u8, 191u8, 215u8));
            }
            {
                let mut teil = s.subsurface(Rect::new(3, 9, 1, 2));
                teil.fill((159u8, 7u8, 35u8, 210u8));
            }
            rustgame_core::draw::circle(&mut s, (1u8, 200u8, 198u8, 168u8), (16, -2), 5, 3);
            s = rustgame_core::transform::rotate(&s, -306.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x26B8DD511F167BB9, flaeche: s }
    }));
    v.push((395, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((17, 16), SRCALPHA);
            s.fill((60u8, 191u8, 37u8, 255u8));
            let mut q = Surface::with_flags((5, 15), SRCALPHA);
            q = muster_auf(q, 5, 15);
            s.blit_ex(&q, (13, 5), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            let mut q = Surface::with_flags((11, 6), 0);
            q = muster_auf(q, 11, 6);
            s.blit_ex(&q, (5, 18), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            let mut q = Surface::with_flags((8, 16), SRCALPHA);
            q = muster_auf(q, 8, 16);
            q.set_colorkey(Some((203u8, 43u8, 20u8)));
            s.blit_ex(&q, (9, 2), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            {
                let mut teil = s.subsurface(Rect::new(2, 8, 3, 2));
                teil.fill((91u8, 235u8, 81u8, 162u8));
            }
            let mut q = Surface::with_flags((17, 13), 0);
            q = muster_auf(q, 17, 13);
            s.blit_ex(&q, (17, 1), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            s.fill_rect((114u8, 156u8, 1u8, 134u8), Rect::new(-5, 11, 3, 6));
            let mut q = Surface::with_flags((15, 9), SRCALPHA);
            q = muster_auf(q, 15, 9);
            q.set_colorkey(Some((196u8, 148u8, 79u8)));
            s.blit_ex(&q, (5, 6), None, 0);
            rustgame_core::draw::line(&mut s, (162u8, 137u8, 208u8, 216u8), (-2, 13), (-1, 5), 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xD2AFF8CC7F0C85B6, flaeche: s }
    }));
    v.push((396, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 14), 0);
            s.fill((73u8, 176u8, 215u8, 255u8));
            let mut q = Surface::with_flags((6, 11), SRCALPHA);
            q = muster_auf(q, 6, 11);
            q.set_alpha(Some(189));
            s.blit_ex(&q, (-2, 11), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            s.fill_rect((151u8, 130u8, 176u8, 255u8), Rect::new(-2, 1, 4, 6));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xE2A4034CBE97A87F, flaeche: s }
    }));
    v.push((397, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((9, 8), SRCALPHA);
            s.fill((141u8, 73u8, 121u8, 0u8));
            let mut q = Surface::with_flags((3, 7), SRCALPHA);
            q = muster_auf(q, 3, 7);
            s.blit_ex(&q, (12, 7), None, 0);
            rustgame_core::draw::circle(&mut s, (177u8, 15u8, 151u8, 184u8), (10, 10), 0, 2);
            rustgame_core::draw::rect_radius(&mut s, (249u8, 97u8, 10u8, 255u8), Rect::new(6, 4, 7, 10), 3, 4);
            s.set_clip(Some(Rect::new(3, 2, 4, 2)));
            rustgame_core::draw::circle(&mut s, (199u8, 141u8, 161u8, 191u8), (-2, -4), 2, 3);
            rustgame_core::draw::circle(&mut s, (214u8, 251u8, 101u8, 103u8), (4, -3), 2, 4);
            rustgame_core::draw::rect_radius(&mut s, (213u8, 115u8, 8u8, 19u8), Rect::new(-5, 3, 4, 2), 4, 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x5335D437C8018BAB, flaeche: s }
    }));
    v.push((398, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((18, 19), SRCALPHA);
            s.fill((87u8, 33u8, 20u8, 235u8));
            rustgame_core::draw::rect_radius(&mut s, (33u8, 144u8, 208u8, 182u8), Rect::new(3, 7, 10, 2), 0, 1);
            rustgame_core::draw::polygon(&mut s, (207u8, 3u8, 207u8, 255u8), &[(12, 20), (17, 17), (14, 18), (11, 10)], 0);
            let mut q = Surface::with_flags((9, 15), 0);
            q = muster_auf(q, 9, 15);
            s.blit_ex(&q, (17, 8), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            rustgame_core::draw::circle(&mut s, (86u8, 234u8, 55u8, 255u8), (-2, 1), 10, 0);
            rustgame_core::draw::circle(&mut s, (108u8, 88u8, 39u8, 210u8), (18, 9), 17, 0);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xBEB7B90D3A9E4421, flaeche: s }
    }));
    v.push((399, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 11), 0);
            s.fill((8u8, 119u8, 48u8, 255u8));
            rustgame_core::draw::circle(&mut s, (127u8, 195u8, 182u8, 22u8), (4, 11), 0, 3);
            s.set_clip(Some(Rect::new(5, 7, 2, 1)));
            rustgame_core::draw::line(&mut s, (211u8, 170u8, 224u8, 255u8), (8, -1), (-1, 6), 3);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x71F9484B49AAD0FD, flaeche: s }
    }));
    v.push((400, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((15, 23), SRCALPHA);
            s.fill((234u8, 210u8, 216u8, 250u8));
            {
                let mut teil = s.subsurface(Rect::new(13, 9, 2, 4));
                teil.fill((72u8, 22u8, 127u8, 220u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (157u8, 117u8, 209u8, 197u8), Rect::new(-1, -6, 3, 2), 2, 0);
            rustgame_core::draw::rect_radius(&mut s, (13u8, 31u8, 31u8, 215u8), Rect::new(-5, 2, 4, 17), 2, 0);
            rustgame_core::draw::rect_radius(&mut s, (248u8, 15u8, 122u8, 206u8), Rect::new(-5, 15, 9, 22), 3, 3);
            rustgame_core::draw::rect_radius(&mut s, (85u8, 156u8, 173u8, 246u8), Rect::new(14, 6, 7, 8), 1, 3);
            rustgame_core::draw::rect_radius(&mut s, (33u8, 96u8, 143u8, 38u8), Rect::new(9, 13, 16, 23), 3, 0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xF0A7A71FF5910660, flaeche: s }
    }));
    v.push((401, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 9), SRCALPHA);
            s.fill((246u8, 163u8, 95u8, 200u8));
            rustgame_core::draw::ellipse(&mut s, (63u8, 37u8, 133u8, 95u8), Rect::new(-5, -5, 21, 7), 0);
            s.scroll(21, -6);
            s.set_clip(Some(Rect::new(16, 3, 5, 4)));
            s.set_clip(Some(Rect::new(14, 5, 4, 3)));
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x43B43DF4447E0799, flaeche: s }
    }));
    v.push((402, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((32, 21), SRCALPHA);
            s.fill((154u8, 233u8, 209u8, 207u8));
            let mut q = Surface::with_flags((12, 2), 0);
            q = muster_auf(q, 12, 2);
            s.blit_ex(&q, (7, 20), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            let mut q = Surface::with_flags((11, 21), SRCALPHA);
            q = muster_auf(q, 11, 21);
            q.set_alpha(Some(149));
            s.blit_ex(&q, (19, 12), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            rustgame_core::draw::rect_radius(&mut s, (160u8, 144u8, 187u8, 255u8), Rect::new(19, 3, 19, 20), 4, 2);
            rustgame_core::draw::circle(&mut s, (218u8, 118u8, 75u8, 106u8), (25, 3), 13, 2);
            s.fill_rect((197u8, 167u8, 66u8, 255u8), Rect::new(13, 0, 6, 5));
            s.fill_rect((179u8, 71u8, 29u8, 188u8), Rect::new(-8, -2, 27, 17));
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xDD726AFD7BFAA106, flaeche: s }
    }));
    v.push((403, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((13, 25), SRCALPHA);
            s.fill((244u8, 76u8, 193u8, 137u8));
            rustgame_core::draw::circle(&mut s, (76u8, 59u8, 132u8, 255u8), (0, 5), 4, 3);
            s.set_clip(Some(Rect::new(12, 19, 1, 4)));
            s.fill_rect((113u8, 1u8, 80u8, 218u8), Rect::new(4, 5, 1, 11));
            rustgame_core::draw::rect_radius(&mut s, (175u8, 211u8, 4u8, 60u8), Rect::new(10, 21, 13, 15), 1, 6);
            rustgame_core::draw::rect_radius(&mut s, (34u8, 85u8, 204u8, 53u8), Rect::new(-5, 2, 2, 23), 2, 4);
            let mut q = Surface::with_flags((13, 17), SRCALPHA);
            q = muster_auf(q, 13, 17);
            s.blit_ex(&q, (16, 19), Some(Rect::new(-1, 1, 4, 15)), rustgame_core::blend::flags::BLEND_RGB_SUB);
            s.fill_rect((87u8, 78u8, 105u8, 255u8), Rect::new(4, 19, 5, 11));
            s = rustgame_core::transform::rotate(&s, -200.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x04D5C94A6EAC3515, flaeche: s }
    }));
    v.push((404, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 18), 0);
            s.fill((47u8, 48u8, 170u8, 255u8));
            let mut q = Surface::with_flags((3, 2), 0);
            q = muster_auf(q, 3, 2);
            s.blit_ex(&q, (-4, 1), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            {
                let mut teil = s.subsurface(Rect::new(2, 16, 6, 1));
                teil.fill((136u8, 23u8, 141u8, 255u8));
            }
            s.scroll(-2, -14);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x5323153999270925, flaeche: s }
    }));
    v.push((405, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 11), 0);
            s.fill((235u8, 51u8, 50u8, 255u8));
            let mut q = Surface::with_flags((6, 5), 0);
            q = muster_auf(q, 6, 5);
            s.blit_ex(&q, (2, 12), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            s.scroll(-5, 4);
            rustgame_core::draw::ellipse(&mut s, (197u8, 8u8, 245u8, 164u8), Rect::new(-4, 1, 6, 14), 0);
            rustgame_core::draw::circle(&mut s, (115u8, 199u8, 224u8, 248u8), (6, 4), 5, 0);
            s.fill_rect((176u8, 23u8, 253u8, 149u8), Rect::new(-1, 6, 7, 9));
            rustgame_core::draw::rect_radius(&mut s, (9u8, 77u8, 23u8, 219u8), Rect::new(4, -3, 9, 1), 4, 4);
            let mut q = Surface::with_flags((4, 8), SRCALPHA);
            q = muster_auf(q, 4, 8);
            s.blit_ex(&q, (-3, 2), Some(Rect::new(5, 5, 4, 5)), rustgame_core::blend::flags::BLEND_RGBA_MULT);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x753F1F3F29ED9934, flaeche: s }
    }));
    v.push((406, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((19, 19), 0);
            s.fill((154u8, 57u8, 73u8, 255u8));
            s.fill_rect((42u8, 59u8, 219u8, 157u8), Rect::new(-4, 9, 12, 13));
            s.fill_rect((106u8, 18u8, 44u8, 89u8), Rect::new(4, 5, 16, 9));
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xB95D71E84A8DCF51, flaeche: s }
    }));
    v.push((407, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((27, 32), SRCALPHA);
            s.fill((187u8, 234u8, 196u8, 207u8));
            let mut q = Surface::with_flags((15, 4), 0);
            q = muster_auf(q, 15, 4);
            s.blit_ex(&q, (25, 9), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            s.fill_rect((115u8, 134u8, 73u8, 255u8), Rect::new(27, 7, 16, 6));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xEE3B5763E9F80285, flaeche: s }
    }));
    v.push((408, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((31, 28), SRCALPHA);
            s.fill((48u8, 222u8, 129u8, 121u8));
            rustgame_core::draw::rect(&mut s, (134u8, 140u8, 71u8, 54u8), Rect::new(22, -5, 3, 13), 1);
            s.set_clip(Some(Rect::new(1, 20, 8, 5)));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xA982638AD8E6D316, flaeche: s }
    }));
    v.push((409, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((32, 22), 0);
            s.fill((76u8, 214u8, 51u8, 255u8));
            rustgame_core::draw::line(&mut s, (46u8, 193u8, 97u8, 101u8), (10, 4), (5, 13), 5);
            s.set_clip(Some(Rect::new(30, 1, 1, 19)));
            rustgame_core::draw::rect_radius(&mut s, (101u8, 75u8, 180u8, 255u8), Rect::new(-14, 5, 26, 5), 4, 1);
            let mut q = Surface::with_flags((19, 18), 0);
            q = muster_auf(q, 19, 18);
            q.set_alpha(Some(165));
            s.blit_ex(&q, (20, 1), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            rustgame_core::draw::line(&mut s, (222u8, 24u8, 201u8, 135u8), (20, 24), (18, 6), 4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x60F600FC22B79161, flaeche: s }
    }));
    v.push((410, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((10, 27), 0);
            s.fill((9u8, 168u8, 46u8, 255u8));
            rustgame_core::draw::circle(&mut s, (87u8, 82u8, 242u8, 100u8), (-3, 16), 7, 3);
            rustgame_core::draw::circle(&mut s, (79u8, 237u8, 119u8, 255u8), (1, 21), 17, 0);
            rustgame_core::draw::circle(&mut s, (153u8, 216u8, 213u8, 238u8), (0, 2), 9, 2);
            rustgame_core::draw::ellipse(&mut s, (156u8, 73u8, 126u8, 129u8), Rect::new(-3, 20, 8, 8), 2);
            s = rustgame_core::transform::rotate(&s, 141.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x2EC98E3FE1D1964D, flaeche: s }
    }));
    v.push((411, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 6), SRCALPHA);
            s.fill((243u8, 206u8, 107u8, 119u8));
            s.fill_rect((111u8, 39u8, 188u8, 62u8), Rect::new(13, 5, 9, 5));
            rustgame_core::draw::rect_radius(&mut s, (218u8, 99u8, 71u8, 255u8), Rect::new(-6, -2, 3, 8), 1, 3);
            s.set_clip(Some(Rect::new(5, 1, 3, 3)));
            rustgame_core::draw::rect_radius(&mut s, (241u8, 212u8, 175u8, 255u8), Rect::new(-3, 7, 10, 2), 1, 0);
            let mut q = Surface::with_flags((5, 7), 0);
            q = muster_auf(q, 5, 7);
            s.blit_ex(&q, (10, 6), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            rustgame_core::draw::polygon(&mut s, (64u8, 50u8, 45u8, 255u8), &[(-4, -1), (-2, 5), (14, 5), (0, 4), (13, 8), (0, 3)], 1);
            rustgame_core::draw::polygon(&mut s, (184u8, 212u8, 254u8, 152u8), &[(-1, -2), (-1, 3), (-4, 2), (1, 8)], 1);
            rustgame_core::draw::polygon(&mut s, (35u8, 47u8, 129u8, 250u8), &[(12, 5), (-4, 2), (5, 8), (5, 2), (14, 2), (6, -1)], 1);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xFA1E139D1F8F85E9, flaeche: s }
    }));
    v.push((412, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((15, 28), 0);
            s.fill((54u8, 101u8, 58u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(3, 14, 8, 1));
                teil.fill((90u8, 180u8, 130u8, 255u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (204u8, 224u8, 171u8, 255u8), Rect::new(6, 26, 17, 14), 3, 0);
            rustgame_core::draw::circle(&mut s, (21u8, 142u8, 77u8, 201u8), (17, 8), 12, 0);
            let mut q = Surface::with_flags((3, 22), SRCALPHA);
            q = muster_auf(q, 3, 22);
            q.set_colorkey(Some((61u8, 251u8, 13u8)));
            s.blit_ex(&q, (18, 27), Some(Rect::new(4, 23, 4, 15)), rustgame_core::blend::flags::BLEND_RGB_MULT);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x5439E9807F9FCE64, flaeche: s }
    }));
    v.push((413, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((32, 30), 0);
            s.fill((239u8, 93u8, 41u8, 255u8));
            rustgame_core::draw::circle(&mut s, (72u8, 129u8, 227u8, 255u8), (13, 2), 29, 2);
            s.scroll(12, 6);
            s.set_clip(Some(Rect::new(12, 27, 9, 3)));
            s.scroll(-14, 20);
            s.scroll(8, 28);
            s.fill_rect((167u8, 255u8, 145u8, 255u8), Rect::new(-1, 15, 3, 21));
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x1489C05B9BDDAFAD, flaeche: s }
    }));
    v.push((414, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((19, 17), 0);
            s.fill((119u8, 137u8, 150u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (67u8, 129u8, 253u8, 199u8), &[(1, -4), (15, 5), (-4, 5), (22, 10)], 1);
            rustgame_core::draw::line(&mut s, (143u8, 91u8, 85u8, 255u8), (9, 8), (10, 14), 3);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x2E6670CA2959215C, flaeche: s }
    }));
    v.push((415, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((30, 16), SRCALPHA);
            s.fill((111u8, 68u8, 32u8, 255u8));
            s.fill_rect((46u8, 11u8, 132u8, 110u8), Rect::new(-5, -6, 10, 9));
            s.fill_rect((207u8, 108u8, 202u8, 226u8), Rect::new(5, 13, 26, 4));
            rustgame_core::draw::ellipse(&mut s, (236u8, 238u8, 106u8, 213u8), Rect::new(-11, 1, 2, 14), 1);
            rustgame_core::draw::circle(&mut s, (245u8, 254u8, 113u8, 140u8), (8, 17), 8, 1);
            rustgame_core::draw::circle(&mut s, (74u8, 136u8, 11u8, 45u8), (30, 11), 26, 4);
            {
                let mut teil = s.subsurface(Rect::new(3, 12, 25, 1));
                teil.fill((228u8, 73u8, 32u8, 255u8));
            }
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x649C2E487BC66886, flaeche: s }
    }));
    v.push((416, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((14, 32), SRCALPHA);
            s.fill((161u8, 226u8, 19u8, 150u8));
            rustgame_core::draw::polygon(&mut s, (228u8, 114u8, 199u8, 251u8), &[(15, -4), (5, 27), (-2, 32)], 1);
            rustgame_core::draw::ellipse(&mut s, (249u8, 82u8, 250u8, 166u8), Rect::new(12, 22, 7, 6), 1);
            rustgame_core::draw::line(&mut s, (246u8, 96u8, 89u8, 99u8), (16, -1), (14, 2), 1);
            let mut q = Surface::with_flags((15, 3), SRCALPHA);
            q = muster_auf(q, 15, 3);
            s.blit_ex(&q, (6, 27), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            rustgame_core::draw::circle(&mut s, (200u8, 215u8, 70u8, 167u8), (12, 16), 6, 2);
            s.fill_rect((74u8, 247u8, 253u8, 124u8), Rect::new(15, 18, 8, 2));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x3F42CED1F72A91F6, flaeche: s }
    }));
    v.push((417, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((25, 31), 0);
            s.fill((40u8, 20u8, 198u8, 255u8));
            rustgame_core::draw::circle(&mut s, (3u8, 236u8, 0u8, 255u8), (20, 1), 0, 2);
            let mut q = Surface::with_flags((10, 7), SRCALPHA);
            q = muster_auf(q, 10, 7);
            q.set_alpha(Some(214));
            s.blit_ex(&q, (26, 27), Some(Rect::new(8, 7, 1, 8)), rustgame_core::blend::flags::BLEND_RGBA_ADD);
            {
                let mut teil = s.subsurface(Rect::new(11, 4, 3, 14));
                teil.fill((143u8, 182u8, 68u8, 216u8));
            }
            rustgame_core::draw::ellipse(&mut s, (175u8, 98u8, 47u8, 255u8), Rect::new(25, -15, 28, 33), 0);
            s = rustgame_core::transform::grayscale(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x490DB0B69F1ED039, flaeche: s }
    }));
    v.push((418, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((16, 17), SRCALPHA);
            s.fill((119u8, 140u8, 21u8, 255u8));
            let mut q = Surface::with_flags((13, 8), SRCALPHA);
            q = muster_auf(q, 13, 8);
            s.blit_ex(&q, (13, -1), Some(Rect::new(-5, 5, 14, 5)), rustgame_core::blend::flags::BLEND_RGB_ADD);
            let mut q = Surface::with_flags((3, 2), SRCALPHA);
            q = muster_auf(q, 3, 2);
            q.set_alpha(Some(136));
            s.blit_ex(&q, (3, 0), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            let mut q = Surface::with_flags((16, 8), SRCALPHA);
            q = muster_auf(q, 16, 8);
            q.set_alpha(Some(38));
            q.set_colorkey(Some((28u8, 91u8, 142u8)));
            s.blit_ex(&q, (15, 15), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            s = rustgame_core::transform::scale(&s, (21, 11));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x62988289B2AEBABF, flaeche: s }
    }));
    v.push((419, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((25, 27), 0);
            s.fill((66u8, 144u8, 185u8, 255u8));
            let mut q = Surface::with_flags((24, 2), 0);
            q = muster_auf(q, 24, 2);
            s.blit_ex(&q, (6, 0), None, 0);
            rustgame_core::draw::line(&mut s, (33u8, 227u8, 149u8, 164u8), (22, 27), (3, 17), 3);
            {
                let mut teil = s.subsurface(Rect::new(6, 9, 4, 16));
                teil.fill((7u8, 7u8, 150u8, 26u8));
            }
            let mut q = Surface::with_flags((24, 12), 0);
            q = muster_auf(q, 24, 12);
            s.blit_ex(&q, (13, 18), Some(Rect::new(18, -3, 25, 7)), rustgame_core::blend::flags::BLEND_RGB_ADD);
            rustgame_core::draw::polygon(&mut s, (27u8, 74u8, 72u8, 241u8), &[(27, 25), (7, -3), (17, 26)], 0);
            rustgame_core::draw::ellipse(&mut s, (2u8, 12u8, 188u8, 145u8), Rect::new(25, -1, 20, 24), 1);
            let mut q = Surface::with_flags((2, 25), SRCALPHA);
            q = muster_auf(q, 2, 25);
            q.set_alpha(Some(82));
            q.set_colorkey(Some((69u8, 21u8, 80u8)));
            s.blit_ex(&q, (-1, 17), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x441D4CEEEFEE4579, flaeche: s }
    }));
    v.push((420, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((6, 27), 0);
            s.fill((139u8, 206u8, 48u8, 255u8));
            rustgame_core::draw::circle(&mut s, (166u8, 248u8, 9u8, 15u8), (8, 4), 17, 2);
            rustgame_core::draw::polygon(&mut s, (172u8, 139u8, 74u8, 255u8), &[(6, 2), (0, 27), (-2, 26), (6, 2), (-1, 27)], 0);
            let mut q = Surface::with_flags((6, 27), 0);
            q = muster_auf(q, 6, 27);
            q.set_alpha(Some(16));
            s.blit_ex(&q, (7, 28), Some(Rect::new(-1, 5, 2, 16)), 0);
            rustgame_core::draw::ellipse(&mut s, (12u8, 8u8, 99u8, 215u8), Rect::new(5, -2, 6, 21), 3);
            rustgame_core::draw::circle(&mut s, (64u8, 23u8, 231u8, 245u8), (3, 7), 22, 3);
            let mut q = Surface::with_flags((2, 29), 0);
            q = muster_auf(q, 2, 29);
            s.blit_ex(&q, (9, 14), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            rustgame_core::draw::polygon(&mut s, (19u8, 129u8, 110u8, 102u8), &[(6, 17), (-4, 12), (-2, 5), (-4, 18), (-3, 29), (5, 21)], 3);
            rustgame_core::draw::circle(&mut s, (118u8, 36u8, 120u8, 255u8), (6, 5), 9, 1);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x5EA8C48C4DCBCB35, flaeche: s }
    }));
    v.push((421, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((26, 25), 0);
            s.fill((252u8, 250u8, 143u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (126u8, 193u8, 63u8, 208u8), Rect::new(5, -12, 10, 24), 3, 1);
            rustgame_core::draw::ellipse(&mut s, (160u8, 153u8, 22u8, 227u8), Rect::new(-13, 3, 13, 16), 1);
            {
                let mut teil = s.subsurface(Rect::new(16, 4, 2, 18));
                teil.fill((187u8, 208u8, 66u8, 157u8));
            }
            let mut q = Surface::with_flags((8, 16), 0);
            q = muster_auf(q, 8, 16);
            q.set_alpha(Some(94));
            s.blit_ex(&q, (0, 17), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            s = rustgame_core::transform::scale(&s, (29, 5));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xBF1CE62ECC7641CA, flaeche: s }
    }));
    v.push((422, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((18, 15), SRCALPHA);
            s.fill((100u8, 154u8, 18u8, 34u8));
            rustgame_core::draw::line(&mut s, (22u8, 243u8, 190u8, 77u8), (12, 15), (8, 17), 2);
            rustgame_core::draw::ellipse(&mut s, (219u8, 148u8, 192u8, 255u8), Rect::new(-1, 0, 0, 3), 0);
            rustgame_core::draw::rect_radius(&mut s, (198u8, 240u8, 34u8, 170u8), Rect::new(13, 8, 17, 8), 0, 6);
            rustgame_core::draw::circle(&mut s, (2u8, 86u8, 41u8, 251u8), (16, -3), 0, 0);
            rustgame_core::draw::line(&mut s, (108u8, 223u8, 47u8, 255u8), (9, -1), (5, 14), 4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x273560242CADD115, flaeche: s }
    }));
    v.push((423, true, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((16, 11), 0);
            s.fill((220u8, 111u8, 162u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(14, 4, 1, 5));
                teil.fill((52u8, 62u8, 167u8, 189u8));
            }
            rustgame_core::draw::circle(&mut s, (38u8, 236u8, 53u8, 220u8), (-2, -1), 2, 4);
            let mut q = Surface::with_flags((7, 5), 0);
            q = muster_auf(q, 7, 5);
            q.set_alpha(Some(129));
            q.set_colorkey(Some((34u8, 170u8, 50u8)));
            s.blit_ex(&q, (18, 11), Some(Rect::new(7, 2, 9, 2)), rustgame_core::blend::flags::BLEND_RGBA_MULT);
            s.set_clip(Some(Rect::new(4, 4, 1, 4)));
            {
                let mut teil = s.subsurface(Rect::new(1, 9, 8, 1));
                teil.fill((198u8, 80u8, 105u8, 255u8));
            }
            rustgame_core::draw::polygon(&mut s, (40u8, 249u8, 114u8, 160u8), &[(5, 13), (11, 9), (0, 6), (7, 14)], 1);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x2A910215E2A9CEA5, flaeche: s }
    }));
    v.push((424, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((23, 20), 0);
            s.fill((71u8, 30u8, 211u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(5, 0, 6, 13));
                teil.fill((105u8, 98u8, 54u8, 101u8));
            }
            let mut q = Surface::with_flags((22, 15), SRCALPHA);
            q = muster_auf(q, 22, 15);
            s.blit_ex(&q, (26, 8), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xE3728274893FFA5D, flaeche: s }
    }));
    v.push((425, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((19, 8), SRCALPHA);
            s.fill((114u8, 115u8, 186u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (190u8, 243u8, 51u8, 99u8), Rect::new(6, 8, 1, 10), 0);
            rustgame_core::draw::circle(&mut s, (149u8, 166u8, 237u8, 96u8), (1, 8), 11, 3);
            s.fill_rect((103u8, 31u8, 167u8, 211u8), Rect::new(-4, 0, 21, 9));
            s.scroll(5, 4);
            rustgame_core::draw::ellipse(&mut s, (121u8, 33u8, 234u8, 190u8), Rect::new(16, 0, 5, 3), 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xE3BDC81FD7E94097, flaeche: s }
    }));
    v.push((426, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((11, 16), 0);
            s.fill((179u8, 39u8, 103u8, 255u8));
            s.scroll(10, 5);
            rustgame_core::draw::line(&mut s, (147u8, 159u8, 171u8, 200u8), (10, 7), (7, -2), 3);
            s = rustgame_core::transform::flip(&s, true, false);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xEA6FA8AA83ADE7D5, flaeche: s }
    }));
    v.push((427, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((11, 25), 0);
            s.fill((184u8, 23u8, 85u8, 255u8));
            s.fill_rect((247u8, 172u8, 235u8, 0u8), Rect::new(4, 14, 8, 21));
            s.scroll(-11, -5);
            rustgame_core::draw::line(&mut s, (207u8, 81u8, 247u8, 255u8), (1, 14), (6, 24), 2);
            s.fill_rect((95u8, 185u8, 41u8, 54u8), Rect::new(12, -1, 12, 24));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xBE8EC56446915BB0, flaeche: s }
    }));
    v.push((428, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((14, 7), 0);
            s.fill((50u8, 128u8, 133u8, 255u8));
            s.fill_rect((192u8, 87u8, 4u8, 33u8), Rect::new(-2, 0, 13, 4));
            rustgame_core::draw::rect_radius(&mut s, (101u8, 33u8, 147u8, 255u8), Rect::new(12, 8, 6, 1), 3, 6);
            rustgame_core::draw::rect_radius(&mut s, (160u8, 231u8, 53u8, 47u8), Rect::new(0, -3, 5, 1), 3, 0);
            rustgame_core::draw::circle(&mut s, (86u8, 240u8, 105u8, 78u8), (10, -1), 12, 1);
            rustgame_core::draw::rect_radius(&mut s, (192u8, 177u8, 69u8, 13u8), Rect::new(15, -1, 11, 6), 1, 0);
            let mut q = Surface::with_flags((13, 3), 0);
            q = muster_auf(q, 13, 3);
            s.blit_ex(&q, (10, 9), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x7397F3A0F72EE6F1, flaeche: s }
    }));
    v.push((429, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((25, 9), 0);
            s.fill((175u8, 10u8, 115u8, 255u8));
            let mut q = Surface::with_flags((2, 8), SRCALPHA);
            q = muster_auf(q, 2, 8);
            s.blit_ex(&q, (14, -3), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            rustgame_core::draw::rect_radius(&mut s, (28u8, 81u8, 33u8, 255u8), Rect::new(9, -4, 11, 10), 1, 2);
            s.fill_rect((200u8, 167u8, 164u8, 249u8), Rect::new(18, 9, 26, 12));
            s.set_clip(Some(Rect::new(10, 2, 15, 1)));
            rustgame_core::draw::line(&mut s, (247u8, 210u8, 113u8, 127u8), (16, -4), (26, 6), 2);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x214D293E1F171C8A, flaeche: s }
    }));
    v.push((430, true, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((14, 27), SRCALPHA);
            s.fill((109u8, 233u8, 163u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (36u8, 98u8, 207u8, 96u8), Rect::new(6, -5, 6, 2), 2);
            rustgame_core::draw::ellipse(&mut s, (221u8, 146u8, 105u8, 255u8), Rect::new(-2, -4, 9, 29), 4);
            rustgame_core::draw::line(&mut s, (30u8, 237u8, 15u8, 225u8), (17, 16), (5, 4), 5);
            s.scroll(4, 22);
            s.fill_rect((14u8, 139u8, 249u8, 238u8), Rect::new(12, 12, 2, 5));
            let mut q = Surface::with_flags((10, 3), SRCALPHA);
            q = muster_auf(q, 10, 3);
            q.set_colorkey(Some((54u8, 118u8, 84u8)));
            s.blit_ex(&q, (5, 12), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            let mut q = Surface::with_flags((14, 19), 0);
            q = muster_auf(q, 14, 19);
            q.set_alpha(Some(74));
            q.set_colorkey(Some((61u8, 76u8, 246u8)));
            s.blit_ex(&q, (16, 5), Some(Rect::new(10, 19, 5, 3)), rustgame_core::blend::flags::BLEND_RGBA_MULT);
            rustgame_core::draw::rect_radius(&mut s, (185u8, 84u8, 224u8, 255u8), Rect::new(-2, 15, 13, 12), 1, 1);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xEEBCCE5D27930DFF, flaeche: s }
    }));
    v.push((431, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((15, 30), SRCALPHA);
            s.fill((0u8, 219u8, 80u8, 17u8));
            rustgame_core::draw::rect_radius(&mut s, (127u8, 181u8, 46u8, 255u8), Rect::new(1, 10, 2, 4), 1, 5);
            s.fill_rect((238u8, 177u8, 93u8, 89u8), Rect::new(9, 24, 2, 8));
            rustgame_core::draw::polygon(&mut s, (21u8, 221u8, 97u8, 255u8), &[(12, 1), (12, 26), (14, 5), (16, -2), (12, 27)], 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x63DCA14203DABD0D, flaeche: s }
    }));
    v.push((432, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((24, 26), 0);
            s.fill((32u8, 197u8, 242u8, 255u8));
            rustgame_core::draw::line(&mut s, (83u8, 248u8, 0u8, 255u8), (9, 9), (17, 14), 4);
            {
                let mut teil = s.subsurface(Rect::new(21, 24, 2, 1));
                teil.fill((53u8, 160u8, 116u8, 255u8));
            }
            rustgame_core::draw::line(&mut s, (61u8, 58u8, 167u8, 255u8), (3, 17), (15, 29), 4);
            rustgame_core::draw::circle(&mut s, (191u8, 221u8, 134u8, 255u8), (23, -1), 3, 2);
            rustgame_core::draw::ellipse(&mut s, (13u8, 69u8, 196u8, 255u8), Rect::new(7, -1, 0, 25), 2);
            {
                let mut teil = s.subsurface(Rect::new(16, 23, 5, 2));
                teil.fill((126u8, 119u8, 41u8, 232u8));
            }
            let mut q = Surface::with_flags((11, 14), SRCALPHA);
            q = muster_auf(q, 11, 14);
            s.blit_ex(&q, (1, 29), Some(Rect::new(5, 1, 11, 16)), rustgame_core::blend::flags::BLEND_RGBA_MIN);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x021D0D7E5896C58E, flaeche: s }
    }));
    v.push((433, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 27), SRCALPHA);
            s.fill((178u8, 156u8, 60u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(3, 13, 3, 6));
                teil.fill((52u8, 48u8, 69u8, 21u8));
            }
            let mut q = Surface::with_flags((11, 22), 0);
            q = muster_auf(q, 11, 22);
            s.blit_ex(&q, (2, 3), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            s.set_clip(Some(Rect::new(7, 12, 5, 11)));
            s.fill_rect((51u8, 174u8, 127u8, 255u8), Rect::new(10, 27, 7, 25));
            let mut q = Surface::with_flags((8, 11), 0);
            q = muster_auf(q, 8, 11);
            s.blit_ex(&q, (0, 11), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            let mut q = Surface::with_flags((7, 26), SRCALPHA);
            q = muster_auf(q, 7, 26);
            s.blit_ex(&q, (6, 15), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            s = rustgame_core::transform::scale(&s, (7, 15));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x4A1E2679FAC9A63C, flaeche: s }
    }));
    v.push((434, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((17, 8), 0);
            s.fill((199u8, 150u8, 10u8, 255u8));
            let mut q = Surface::with_flags((15, 7), SRCALPHA);
            q = muster_auf(q, 15, 7);
            s.blit_ex(&q, (-3, 6), Some(Rect::new(14, -2, 3, 3)), rustgame_core::blend::flags::BLEND_RGBA_MULT);
            rustgame_core::draw::ellipse(&mut s, (128u8, 148u8, 18u8, 23u8), Rect::new(-1, 6, 19, 1), 3);
            s.scroll(-3, 8);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x4F424B7CC0BE7F28, flaeche: s }
    }));
    v.push((435, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((32, 12), 0);
            s.fill((159u8, 26u8, 247u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (152u8, 118u8, 35u8, 40u8), Rect::new(13, -3, 25, 7), 4, 5);
            let mut q = Surface::with_flags((5, 14), 0);
            q = muster_auf(q, 5, 14);
            q.set_alpha(Some(84));
            s.blit_ex(&q, (32, 7), None, 0);
            s.fill_rect((43u8, 213u8, 242u8, 255u8), Rect::new(20, 0, 29, 1));
            s.scroll(9, 9);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x0BD08545F990150E, flaeche: s }
    }));
    v.push((436, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((18, 21), 0);
            s.fill((117u8, 91u8, 189u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (55u8, 25u8, 228u8, 255u8), Rect::new(10, -9, 1, 16), 0, 6);
            let mut q = Surface::with_flags((18, 20), SRCALPHA);
            q = muster_auf(q, 18, 20);
            s.blit_ex(&q, (21, 9), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            rustgame_core::draw::circle(&mut s, (42u8, 91u8, 139u8, 255u8), (-2, 9), 5, 4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x5FAA004B611C278C, flaeche: s }
    }));
    v.push((437, true, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((22, 15), SRCALPHA);
            s.fill((6u8, 128u8, 228u8, 255u8));
            rustgame_core::draw::circle(&mut s, (139u8, 161u8, 19u8, 191u8), (9, 4), 19, 0);
            rustgame_core::draw::rect_radius(&mut s, (88u8, 51u8, 5u8, 217u8), Rect::new(-11, 0, 19, 16), 0, 4);
            let mut q = Surface::with_flags((19, 16), 0);
            q = muster_auf(q, 19, 16);
            q.set_colorkey(Some((161u8, 251u8, 79u8)));
            s.blit_ex(&q, (0, 1), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
            rustgame_core::draw::line(&mut s, (81u8, 6u8, 103u8, 114u8), (-2, 5), (15, 16), 2);
            rustgame_core::draw::ellipse(&mut s, (130u8, 24u8, 251u8, 37u8), Rect::new(8, -2, 1, 10), 2);
            s.fill_rect((223u8, 249u8, 237u8, 255u8), Rect::new(10, -7, 12, 5));
            s.fill_rect((67u8, 172u8, 80u8, 21u8), Rect::new(-10, 8, 4, 9));
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xC1F9D2EFFCE1CBD3, flaeche: s }
    }));
    v.push((438, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 23), 0);
            s.fill((42u8, 146u8, 223u8, 255u8));
            rustgame_core::draw::circle(&mut s, (204u8, 117u8, 70u8, 255u8), (-3, 4), 4, 1);
            let mut q = Surface::with_flags((3, 12), SRCALPHA);
            q = muster_auf(q, 3, 12);
            q.set_alpha(Some(14));
            q.set_colorkey(Some((227u8, 129u8, 58u8)));
            s.blit_ex(&q, (0, 14), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x6B9E8D069E4FDFB2, flaeche: s }
    }));
    v.push((439, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((18, 16), SRCALPHA);
            s.fill((222u8, 134u8, 212u8, 255u8));
            rustgame_core::draw::circle(&mut s, (228u8, 153u8, 204u8, 119u8), (0, 15), 0, 0);
            rustgame_core::draw::rect_radius(&mut s, (115u8, 242u8, 62u8, 165u8), Rect::new(11, 13, 10, 5), 3, 1);
            rustgame_core::draw::rect_radius(&mut s, (206u8, 61u8, 236u8, 255u8), Rect::new(-4, 3, 8, 0), 0, 3);
            rustgame_core::draw::polygon(&mut s, (114u8, 197u8, 129u8, 255u8), &[(2, 16), (-2, 0), (-2, 7), (7, 9), (0, 14)], 1);
            rustgame_core::draw::ellipse(&mut s, (116u8, 201u8, 32u8, 102u8), Rect::new(-3, 2, 21, 18), 0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x845A5F827D1777B9, flaeche: s }
    }));
    v.push((440, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((9, 30), SRCALPHA);
            s.fill((151u8, 119u8, 106u8, 255u8));
            s.fill_rect((195u8, 70u8, 119u8, 122u8), Rect::new(7, -14, 1, 26));
            {
                let mut teil = s.subsurface(Rect::new(1, 14, 5, 5));
                teil.fill((183u8, 205u8, 225u8, 255u8));
            }
            rustgame_core::draw::circle(&mut s, (198u8, 244u8, 198u8, 87u8), (9, 3), 1, 1);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xDB4DBEA62FD72AE8, flaeche: s }
    }));
    v.push((441, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((11, 31), SRCALPHA);
            s.fill((240u8, 111u8, 230u8, 78u8));
            rustgame_core::draw::line(&mut s, (1u8, 208u8, 141u8, 255u8), (-1, 28), (7, 33), 5);
            s.set_clip(Some(Rect::new(4, 2, 2, 8)));
            let mut q = Surface::with_flags((10, 2), 0);
            q = muster_auf(q, 10, 2);
            s.blit_ex(&q, (4, -2), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            rustgame_core::draw::circle(&mut s, (61u8, 17u8, 218u8, 18u8), (-2, 33), 10, 2);
            rustgame_core::draw::polygon(&mut s, (172u8, 142u8, 82u8, 255u8), &[(1, 19), (12, 33), (14, 27), (-1, 25), (3, 4), (-3, 33)], 0);
            rustgame_core::draw::circle(&mut s, (152u8, 183u8, 185u8, 106u8), (9, 27), 7, 2);
            let mut q = Surface::with_flags((9, 20), 0);
            q = muster_auf(q, 9, 20);
            q.set_alpha(Some(170));
            s.blit_ex(&q, (2, 6), Some(Rect::new(-3, 9, 7, 22)), rustgame_core::blend::flags::BLEND_RGB_MIN);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xAD3EAB7D2E7B5986, flaeche: s }
    }));
    v.push((442, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((17, 30), SRCALPHA);
            s.fill((211u8, 243u8, 196u8, 216u8));
            rustgame_core::draw::ellipse(&mut s, (167u8, 207u8, 43u8, 124u8), Rect::new(-2, 6, 7, 11), 3);
            rustgame_core::draw::ellipse(&mut s, (223u8, 205u8, 72u8, 103u8), Rect::new(-5, 3, 2, 24), 4);
            let mut q = Surface::with_flags((8, 21), 0);
            q = muster_auf(q, 8, 21);
            s.blit_ex(&q, (11, 11), Some(Rect::new(4, 0, 7, 15)), rustgame_core::blend::flags::BLEND_RGB_MULT);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x43AE57B22E11D615, flaeche: s }
    }));
    v.push((443, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((30, 10), SRCALPHA);
            s.fill((171u8, 147u8, 55u8, 136u8));
            {
                let mut teil = s.subsurface(Rect::new(11, 9, 18, 1));
                teil.fill((232u8, 120u8, 46u8, 255u8));
            }
            let mut q = Surface::with_flags((17, 11), SRCALPHA);
            q = muster_auf(q, 17, 11);
            s.blit_ex(&q, (11, 9), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
            s.fill_rect((27u8, 250u8, 230u8, 144u8), Rect::new(-13, 2, 11, 0));
            s.fill_rect((22u8, 132u8, 190u8, 255u8), Rect::new(11, -5, 32, 12));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x4F2CCAE6A46BEC86, flaeche: s }
    }));
    v.push((444, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((15, 15), SRCALPHA);
            s.fill((226u8, 159u8, 253u8, 154u8));
            let mut q = Surface::with_flags((8, 17), 0);
            q = muster_auf(q, 8, 17);
            q.set_colorkey(Some((14u8, 61u8, 9u8)));
            s.blit_ex(&q, (7, 16), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            let mut q = Surface::with_flags((3, 8), 0);
            q = muster_auf(q, 3, 8);
            q.set_alpha(Some(171));
            s.blit_ex(&q, (5, 16), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x5B07BB20A79C68E9, flaeche: s }
    }));
    v.push((445, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((23, 19), SRCALPHA);
            s.fill((131u8, 207u8, 159u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (130u8, 132u8, 187u8, 216u8), Rect::new(19, -6, 18, 1), 2);
            let mut q = Surface::with_flags((21, 20), 0);
            q = muster_auf(q, 21, 20);
            s.blit_ex(&q, (15, 5), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            s.fill_rect((95u8, 183u8, 114u8, 24u8), Rect::new(-2, 13, 0, 7));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x1FA3658EA946FC0E, flaeche: s }
    }));
    v.push((446, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((31, 21), 0);
            s.fill((110u8, 95u8, 71u8, 255u8));
            let mut q = Surface::with_flags((14, 14), SRCALPHA);
            q = muster_auf(q, 14, 14);
            q.set_alpha(Some(48));
            s.blit_ex(&q, (4, 12), Some(Rect::new(5, 1, 1, 6)), rustgame_core::blend::flags::BLEND_RGBA_SUB);
            let mut q = Surface::with_flags((4, 10), SRCALPHA);
            q = muster_auf(q, 4, 10);
            q.set_colorkey(Some((255u8, 105u8, 166u8)));
            s.blit_ex(&q, (17, -4), Some(Rect::new(1, 8, 4, 11)), rustgame_core::blend::flags::BLEND_RGB_MAX);
            rustgame_core::draw::circle(&mut s, (168u8, 179u8, 23u8, 76u8), (6, -3), 6, 4);
            rustgame_core::draw::rect_radius(&mut s, (92u8, 239u8, 210u8, 164u8), Rect::new(27, 20, 31, 17), 0, 6);
            rustgame_core::draw::polygon(&mut s, (44u8, 181u8, 18u8, 215u8), &[(22, 2), (20, -2), (18, 20), (11, 20), (11, -1)], 1);
            let mut q = Surface::with_flags((27, 17), 0);
            q = muster_auf(q, 27, 17);
            q.set_colorkey(Some((233u8, 172u8, 156u8)));
            s.blit_ex(&q, (15, 11), Some(Rect::new(-3, -4, 13, 11)), rustgame_core::blend::flags::BLEND_RGBA_ADD);
            let mut q = Surface::with_flags((10, 18), 0);
            q = muster_auf(q, 10, 18);
            s.blit_ex(&q, (29, 16), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            rustgame_core::draw::rect_radius(&mut s, (146u8, 168u8, 59u8, 255u8), Rect::new(-7, -6, 0, 21), 3, 2);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x68E5DF9ED5F39E18, flaeche: s }
    }));
    v.push((447, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((19, 27), 0);
            s.fill((192u8, 210u8, 64u8, 255u8));
            rustgame_core::draw::line(&mut s, (148u8, 41u8, 255u8, 255u8), (10, 8), (19, -2), 5);
            rustgame_core::draw::rect_radius(&mut s, (120u8, 45u8, 176u8, 129u8), Rect::new(10, -14, 19, 5), 3, 2);
            rustgame_core::draw::polygon(&mut s, (229u8, 120u8, 211u8, 199u8), &[(0, 22), (8, 9), (-4, 24), (22, -2), (18, 2), (16, 2)], 2);
            s.scroll(8, 15);
            s.scroll(-3, 7);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x53DBDF5EC3CD706E, flaeche: s }
    }));
    v.push((448, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((14, 13), SRCALPHA);
            s.fill((0u8, 176u8, 218u8, 255u8));
            s.set_clip(Some(Rect::new(9, 3, 5, 6)));
            s.scroll(-3, -3);
            rustgame_core::draw::ellipse(&mut s, (121u8, 182u8, 209u8, 156u8), Rect::new(4, -4, 17, 1), 0);
            s.scroll(-8, 6);
            rustgame_core::draw::ellipse(&mut s, (108u8, 90u8, 76u8, 4u8), Rect::new(1, 6, 13, 13), 3);
            rustgame_core::draw::circle(&mut s, (59u8, 14u8, 221u8, 184u8), (-4, 7), 8, 1);
            let mut q = Surface::with_flags((2, 15), 0);
            q = muster_auf(q, 2, 15);
            s.blit_ex(&q, (3, -3), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xF2823B640AF57291, flaeche: s }
    }));
    v.push((449, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 12), SRCALPHA);
            s.fill((109u8, 249u8, 51u8, 255u8));
            let mut q = Surface::with_flags((9, 12), 0);
            q = muster_auf(q, 9, 12);
            s.blit_ex(&q, (3, 9), Some(Rect::new(7, 6, 5, 11)), rustgame_core::blend::flags::BLEND_RGB_MIN);
            rustgame_core::draw::line(&mut s, (183u8, 27u8, 66u8, 52u8), (15, 11), (8, 14), 4);
            s.fill_rect((203u8, 26u8, 132u8, 180u8), Rect::new(11, 1, 1, 15));
            let mut q = Surface::with_flags((4, 8), SRCALPHA);
            q = muster_auf(q, 4, 8);
            s.blit_ex(&q, (-4, -1), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            s.set_clip(Some(Rect::new(0, 4, 6, 7)));
            let mut q = Surface::with_flags((2, 11), SRCALPHA);
            q = muster_auf(q, 2, 11);
            q.set_alpha(Some(236));
            q.set_colorkey(Some((222u8, 123u8, 126u8)));
            s.blit_ex(&q, (-2, 13), Some(Rect::new(3, 11, 5, 8)), rustgame_core::blend::flags::BLEND_RGB_MIN);
            let mut q = Surface::with_flags((11, 6), 0);
            q = muster_auf(q, 11, 6);
            q.set_alpha(Some(71));
            q.set_colorkey(Some((106u8, 53u8, 251u8)));
            s.blit_ex(&q, (15, 11), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            s = s.convert_alpha();
            s = rustgame_core::transform::flip(&s, true, false);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xC1F3D4C002196C98, flaeche: s }
    }));
    v.push((450, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((9, 21), 0);
            s.fill((74u8, 201u8, 30u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (124u8, 238u8, 31u8, 15u8), Rect::new(-3, 9, 2, 23), 2, 0);
            rustgame_core::draw::circle(&mut s, (133u8, 78u8, 60u8, 255u8), (2, 15), 15, 4);
            s = s.convert_alpha();
            s = rustgame_core::transform::rotate(&s, -392.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x9454762815C11EC1, flaeche: s }
    }));
    v.push((451, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 24), SRCALPHA);
            s.fill((48u8, 62u8, 175u8, 22u8));
            rustgame_core::draw::ellipse(&mut s, (251u8, 26u8, 220u8, 255u8), Rect::new(-1, -3, 6, 15), 1);
            rustgame_core::draw::line(&mut s, (206u8, 108u8, 221u8, 89u8), (4, 19), (8, 26), 5);
            s.set_clip(Some(Rect::new(4, 6, 3, 13)));
            rustgame_core::draw::rect_radius(&mut s, (61u8, 109u8, 184u8, 255u8), Rect::new(-3, 9, 6, 10), 3, 0);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x561E66B8FDE094B3, flaeche: s }
    }));
    v.push((452, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((16, 17), 0);
            s.fill((156u8, 168u8, 45u8, 255u8));
            s.fill_rect((28u8, 253u8, 64u8, 249u8), Rect::new(0, 10, 8, 12));
            rustgame_core::draw::line(&mut s, (76u8, 135u8, 170u8, 184u8), (18, 13), (2, 20), 5);
            rustgame_core::draw::polygon(&mut s, (190u8, 40u8, 41u8, 159u8), &[(18, 14), (18, -3), (0, 6), (2, 1), (2, -1)], 2);
            let mut q = Surface::with_flags((15, 7), 0);
            q = muster_auf(q, 15, 7);
            s.blit_ex(&q, (9, 8), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xD4C3C715579F571E, flaeche: s }
    }));
    v.push((453, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((13, 18), 0);
            s.fill((197u8, 192u8, 115u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (243u8, 36u8, 223u8, 160u8), Rect::new(-2, -2, 0, 2), 4, 0);
            rustgame_core::draw::line(&mut s, (201u8, 220u8, 163u8, 255u8), (8, -4), (6, 18), 4);
            rustgame_core::draw::circle(&mut s, (86u8, 11u8, 141u8, 255u8), (16, 19), 10, 3);
            rustgame_core::draw::circle(&mut s, (139u8, 244u8, 177u8, 255u8), (13, 13), 11, 4);
            {
                let mut teil = s.subsurface(Rect::new(12, 17, 1, 1));
                teil.fill((62u8, 56u8, 7u8, 50u8));
            }
            let mut q = Surface::with_flags((14, 13), 0);
            q = muster_auf(q, 14, 13);
            s.blit_ex(&q, (8, 2), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xE25002DCAA3566B9, flaeche: s }
    }));
    v.push((454, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((15, 24), 0);
            s.fill((166u8, 134u8, 217u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(5, 8, 3, 11));
                teil.fill((172u8, 42u8, 110u8, 169u8));
            }
            rustgame_core::draw::polygon(&mut s, (248u8, 123u8, 108u8, 108u8), &[(12, 8), (17, 22), (17, 23), (16, -3)], 1);
            rustgame_core::draw::ellipse(&mut s, (67u8, 137u8, 82u8, 152u8), Rect::new(10, 7, 7, 19), 4);
            rustgame_core::draw::rect_radius(&mut s, (81u8, 153u8, 22u8, 172u8), Rect::new(-7, 1, 12, 25), 0, 4);
            let mut q = Surface::with_flags((8, 19), SRCALPHA);
            q = muster_auf(q, 8, 19);
            q.set_alpha(Some(28));
            q.set_colorkey(Some((90u8, 83u8, 145u8)));
            s.blit_ex(&q, (11, 19), Some(Rect::new(-3, 14, 10, 14)), rustgame_core::blend::flags::BLEND_RGBA_MAX);
            s.fill_rect((28u8, 245u8, 57u8, 31u8), Rect::new(16, 5, 11, 4));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x48DCE33F6880EC52, flaeche: s }
    }));
    v.push((455, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((29, 6), SRCALPHA);
            s.fill((92u8, 119u8, 68u8, 118u8));
            rustgame_core::draw::line(&mut s, (194u8, 86u8, 128u8, 255u8), (0, 3), (29, 9), 4);
            s.fill_rect((150u8, 11u8, 181u8, 255u8), Rect::new(14, 7, 20, 8));
            s.fill_rect((81u8, 78u8, 46u8, 255u8), Rect::new(15, -2, 1, 4));
            let mut q = Surface::with_flags((13, 4), 0);
            q = muster_auf(q, 13, 4);
            s.blit_ex(&q, (32, 0), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            rustgame_core::draw::ellipse(&mut s, (41u8, 149u8, 140u8, 29u8), Rect::new(27, 1, 7, 6), 2);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x118929BA36DA2DD1, flaeche: s }
    }));
    v.push((456, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((29, 20), 0);
            s.fill((222u8, 199u8, 20u8, 255u8));
            rustgame_core::draw::circle(&mut s, (167u8, 250u8, 58u8, 62u8), (1, 11), 22, 3);
            s.fill_rect((97u8, 94u8, 199u8, 255u8), Rect::new(18, 20, 21, 2));
            rustgame_core::draw::rect_radius(&mut s, (159u8, 214u8, 102u8, 255u8), Rect::new(7, -2, 14, 14), 1, 4);
            s.scroll(-28, 0);
            rustgame_core::draw::ellipse(&mut s, (85u8, 206u8, 45u8, 255u8), Rect::new(10, 21, 29, 20), 0);
            let mut q = Surface::with_flags((16, 19), SRCALPHA);
            q = muster_auf(q, 16, 19);
            q.set_alpha(Some(247));
            s.blit_ex(&q, (9, 4), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x5E80591866B21C92, flaeche: s }
    }));
    v.push((457, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((30, 11), 0);
            s.fill((137u8, 235u8, 174u8, 255u8));
            s.set_clip(Some(Rect::new(23, 3, 5, 3)));
            {
                let mut teil = s.subsurface(Rect::new(28, 6, 2, 2));
                teil.fill((18u8, 46u8, 164u8, 237u8));
            }
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xE574635BE8F74A45, flaeche: s }
    }));
    v.push((458, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 25), 0);
            s.fill((33u8, 45u8, 146u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (233u8, 152u8, 166u8, 9u8), Rect::new(-3, -12, 11, 11), 1, 3);
            s.set_clip(Some(Rect::new(4, 3, 2, 21)));
            s.scroll(-7, 1);
            s.fill_rect((127u8, 245u8, 146u8, 237u8), Rect::new(0, 14, 8, 0));
            let mut q = Surface::with_flags((10, 10), 0);
            q = muster_auf(q, 10, 10);
            q.set_alpha(Some(93));
            q.set_colorkey(Some((162u8, 51u8, 61u8)));
            s.blit_ex(&q, (3, 10), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            {
                let mut teil = s.subsurface(Rect::new(1, 8, 6, 9));
                teil.fill((116u8, 84u8, 238u8, 243u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xFCBCFAD6AB748C35, flaeche: s }
    }));
    v.push((459, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((26, 30), 0);
            s.fill((76u8, 204u8, 8u8, 255u8));
            rustgame_core::draw::line(&mut s, (157u8, 42u8, 183u8, 171u8), (8, 30), (25, 1), 2);
            s.fill_rect((19u8, 255u8, 79u8, 9u8), Rect::new(11, 15, 29, 12));
            rustgame_core::draw::line(&mut s, (62u8, 120u8, 138u8, 164u8), (22, 25), (8, 29), 1);
            s = rustgame_core::transform::scale(&s, (15, 18));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x03D50B160A36DCB5, flaeche: s }
    }));
    v.push((460, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((15, 29), 0);
            s.fill((8u8, 203u8, 167u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (244u8, 162u8, 19u8, 232u8), &[(1, -1), (-1, 9), (13, 30), (14, 6), (-4, 23), (4, -4)], 2);
            rustgame_core::draw::rect_radius(&mut s, (195u8, 95u8, 145u8, 98u8), Rect::new(16, 4, 11, 0), 1, 5);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xAD1F5A132FD5BE4D, flaeche: s }
    }));
    v.push((461, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((29, 16), 0);
            s.fill((214u8, 88u8, 45u8, 255u8));
            let mut q = Surface::with_flags((17, 9), SRCALPHA);
            q = muster_auf(q, 17, 9);
            s.blit_ex(&q, (5, -1), None, 0);
            s.fill_rect((155u8, 136u8, 2u8, 34u8), Rect::new(18, 0, 14, 0));
            rustgame_core::draw::circle(&mut s, (198u8, 57u8, 74u8, 255u8), (6, 16), 14, 1);
            s.fill_rect((45u8, 67u8, 46u8, 255u8), Rect::new(17, 15, 3, 13));
            let mut q = Surface::with_flags((13, 10), 0);
            q = muster_auf(q, 13, 10);
            q.set_alpha(Some(77));
            s.blit_ex(&q, (31, 9), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
            let mut q = Surface::with_flags((31, 12), 0);
            q = muster_auf(q, 31, 12);
            q.set_alpha(Some(169));
            s.blit_ex(&q, (3, 18), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            let mut q = Surface::with_flags((23, 2), SRCALPHA);
            q = muster_auf(q, 23, 2);
            s.blit_ex(&q, (14, 13), Some(Rect::new(-2, 3, 21, 3)), rustgame_core::blend::flags::BLEND_RGB_MIN);
            s.fill_rect((198u8, 138u8, 140u8, 221u8), Rect::new(23, 14, 26, 14));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xE55727A483A5C4F1, flaeche: s }
    }));
    v.push((462, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((24, 27), SRCALPHA);
            s.fill((93u8, 13u8, 48u8, 197u8));
            rustgame_core::draw::circle(&mut s, (139u8, 94u8, 186u8, 255u8), (-1, 9), 5, 3);
            rustgame_core::draw::polygon(&mut s, (16u8, 43u8, 111u8, 212u8), &[(1, -1), (12, 20), (12, 18), (19, 23), (14, 7)], 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xBC283818B3C4E762, flaeche: s }
    }));
    v.push((463, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 31), SRCALPHA);
            s.fill((109u8, 35u8, 90u8, 42u8));
            let mut q = Surface::with_flags((12, 17), SRCALPHA);
            q = muster_auf(q, 12, 17);
            s.blit_ex(&q, (15, 1), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            rustgame_core::draw::circle(&mut s, (201u8, 3u8, 200u8, 255u8), (14, 11), 14, 2);
            rustgame_core::draw::circle(&mut s, (168u8, 71u8, 105u8, 255u8), (15, 10), 21, 2);
            s.fill_rect((37u8, 251u8, 24u8, 50u8), Rect::new(-4, 31, 12, 5));
            rustgame_core::draw::rect_radius(&mut s, (169u8, 52u8, 203u8, 100u8), Rect::new(9, -1, 15, 6), 1, 5);
            let mut q = Surface::with_flags((13, 7), 0);
            q = muster_auf(q, 13, 7);
            q.set_alpha(Some(240));
            s.blit_ex(&q, (6, 14), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xDD8808A87264DD8C, flaeche: s }
    }));
    v.push((464, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((14, 11), 0);
            s.fill((122u8, 117u8, 142u8, 255u8));
            s.set_clip(Some(Rect::new(5, 8, 2, 2)));
            s.scroll(2, 3);
            rustgame_core::draw::ellipse(&mut s, (91u8, 41u8, 39u8, 193u8), Rect::new(8, -6, 0, 12), 1);
            s.set_clip(Some(Rect::new(3, 3, 8, 7)));
            s.fill_rect((181u8, 127u8, 173u8, 62u8), Rect::new(13, 1, 4, 4));
            rustgame_core::draw::circle(&mut s, (5u8, 2u8, 150u8, 114u8), (12, 5), 13, 0);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x56528E8F06EB4D25, flaeche: s }
    }));
    v.push((465, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 13), SRCALPHA);
            s.fill((116u8, 91u8, 201u8, 92u8));
            s.scroll(3, -6);
            {
                let mut teil = s.subsurface(Rect::new(8, 9, 4, 4));
                teil.fill((143u8, 83u8, 192u8, 25u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x5A297C8426AEE945, flaeche: s }
    }));
    v.push((466, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 9), SRCALPHA);
            s.fill((68u8, 115u8, 244u8, 76u8));
            rustgame_core::draw::line(&mut s, (205u8, 247u8, 224u8, 64u8), (9, -1), (0, 8), 3);
            s.scroll(-6, 0);
            s.set_clip(Some(Rect::new(4, 2, 1, 5)));
            {
                let mut teil = s.subsurface(Rect::new(2, 7, 4, 1));
                teil.fill((142u8, 19u8, 12u8, 245u8));
            }
            s.set_clip(Some(Rect::new(5, 8, 1, 1)));
            let mut q = Surface::with_flags((5, 3), SRCALPHA);
            q = muster_auf(q, 5, 3);
            s.blit_ex(&q, (6, 5), Some(Rect::new(4, 4, 5, 3)), rustgame_core::blend::flags::BLEND_RGBA_MULT);
            rustgame_core::draw::rect_radius(&mut s, (151u8, 203u8, 196u8, 80u8), Rect::new(0, 3, 6, 9), 3, 4);
            let mut q = Surface::with_flags((5, 5), SRCALPHA);
            q = muster_auf(q, 5, 5);
            s.blit_ex(&q, (-3, 4), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            s = rustgame_core::transform::flip(&s, false, true);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x84E5DDF4768F0597, flaeche: s }
    }));
    v.push((467, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((14, 21), SRCALPHA);
            s.fill((48u8, 155u8, 135u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (49u8, 87u8, 155u8, 93u8), Rect::new(-4, 3, 13, 4), 2);
            s.set_clip(Some(Rect::new(11, 1, 2, 16)));
            rustgame_core::draw::rect_radius(&mut s, (223u8, 233u8, 129u8, 255u8), Rect::new(3, 17, 17, 17), 0, 2);
            rustgame_core::draw::polygon(&mut s, (127u8, 107u8, 220u8, 87u8), &[(-4, 19), (5, 6), (5, 13), (1, 0), (1, 7), (-4, -2)], 3);
            rustgame_core::draw::ellipse(&mut s, (185u8, 19u8, 130u8, 75u8), Rect::new(12, -7, 2, 10), 4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xDA02676C369DE8F1, flaeche: s }
    }));
    v.push((468, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 18), SRCALPHA);
            s.fill((195u8, 77u8, 238u8, 255u8));
            rustgame_core::draw::rect(&mut s, (50u8, 147u8, 92u8, 148u8), Rect::new(-3, -6, 14, 6), 3);
            let mut q = Surface::with_flags((4, 9), SRCALPHA);
            q = muster_auf(q, 4, 9);
            s.blit_ex(&q, (3, 18), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            {
                let mut teil = s.subsurface(Rect::new(8, 9, 4, 4));
                teil.fill((77u8, 166u8, 35u8, 47u8));
            }
            rustgame_core::draw::polygon(&mut s, (29u8, 246u8, 81u8, 100u8), &[(3, 7), (-1, 12), (7, 14), (4, 9)], 1);
            rustgame_core::draw::rect_radius(&mut s, (42u8, 10u8, 32u8, 215u8), Rect::new(-2, 15, 8, 5), 2, 4);
            s.set_clip(Some(Rect::new(6, 6, 4, 12)));
            rustgame_core::draw::line(&mut s, (140u8, 161u8, 209u8, 58u8), (2, 7), (9, -4), 1);
            let mut q = Surface::with_flags((3, 13), 0);
            q = muster_auf(q, 3, 13);
            s.blit_ex(&q, (2, 4), Some(Rect::new(2, 14, 4, 4)), rustgame_core::blend::flags::BLEND_RGBA_MULT);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x57D93FB3609F5E82, flaeche: s }
    }));
    v.push((469, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((14, 31), SRCALPHA);
            s.fill((194u8, 29u8, 48u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(3, 23, 9, 4));
                teil.fill((5u8, 98u8, 178u8, 255u8));
            }
            rustgame_core::draw::circle(&mut s, (225u8, 217u8, 161u8, 204u8), (7, 13), 16, 0);
            s.scroll(6, -15);
            s.fill_rect((0u8, 170u8, 246u8, 202u8), Rect::new(12, -16, 12, 11));
            let mut q = Surface::with_flags((13, 23), SRCALPHA);
            q = muster_auf(q, 13, 23);
            q.set_colorkey(Some((181u8, 30u8, 145u8)));
            s.blit_ex(&q, (4, 5), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            s.fill_rect((209u8, 30u8, 250u8, 61u8), Rect::new(9, 8, 14, 34));
            let mut q = Surface::with_flags((4, 5), 0);
            q = muster_auf(q, 4, 5);
            s.blit_ex(&q, (10, 32), Some(Rect::new(2, -2, 2, 0)), 0);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xF677EDE980BBA9AE, flaeche: s }
    }));
    v.push((470, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 29), SRCALPHA);
            s.fill((245u8, 53u8, 192u8, 255u8));
            let mut q = Surface::with_flags((7, 3), 0);
            q = muster_auf(q, 7, 3);
            s.blit_ex(&q, (-1, 8), None, 0);
            rustgame_core::draw::line(&mut s, (9u8, 64u8, 27u8, 251u8), (-4, 29), (5, 7), 1);
            let mut q = Surface::with_flags((9, 4), 0);
            q = muster_auf(q, 9, 4);
            s.blit_ex(&q, (-1, 26), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            rustgame_core::draw::ellipse(&mut s, (159u8, 71u8, 8u8, 43u8), Rect::new(4, 17, 6, 9), 4);
            rustgame_core::draw::circle(&mut s, (14u8, 236u8, 33u8, 36u8), (-4, 16), 10, 2);
            {
                let mut teil = s.subsurface(Rect::new(0, 26, 6, 2));
                teil.fill((146u8, 99u8, 94u8, 190u8));
            }
            rustgame_core::draw::ellipse(&mut s, (30u8, 145u8, 66u8, 72u8), Rect::new(0, 30, 4, 4), 2);
            rustgame_core::draw::rect_radius(&mut s, (36u8, 66u8, 242u8, 69u8), Rect::new(-3, 28, 1, 24), 3, 4);
            s = rustgame_core::transform::rotate(&s, -277.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x5EA5849866C8CC44, flaeche: s }
    }));
    v.push((471, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((27, 8), 0);
            s.fill((237u8, 165u8, 181u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (29u8, 140u8, 222u8, 252u8), Rect::new(11, 2, 15, 3), 2, 2);
            {
                let mut teil = s.subsurface(Rect::new(20, 6, 3, 1));
                teil.fill((85u8, 120u8, 231u8, 71u8));
            }
            s.scroll(-7, 3);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xB39B501754E57228, flaeche: s }
    }));
    v.push((472, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((10, 12), 0);
            s.fill((232u8, 29u8, 89u8, 255u8));
            let mut q = Surface::with_flags((6, 4), SRCALPHA);
            q = muster_auf(q, 6, 4);
            q.set_colorkey(Some((23u8, 61u8, 152u8)));
            s.blit_ex(&q, (3, 6), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            rustgame_core::draw::circle(&mut s, (148u8, 108u8, 116u8, 23u8), (7, -2), 11, 2);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xD48C052C76350575, flaeche: s }
    }));
    v.push((473, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((16, 16), SRCALPHA);
            s.fill((57u8, 152u8, 145u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (89u8, 162u8, 251u8, 3u8), Rect::new(6, 0, 15, 18), 3);
            rustgame_core::draw::circle(&mut s, (78u8, 56u8, 32u8, 255u8), (0, 18), 9, 4);
            rustgame_core::draw::rect_radius(&mut s, (148u8, 120u8, 125u8, 114u8), Rect::new(-1, 11, 13, 7), 4, 5);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xD8A3099857BE621B, flaeche: s }
    }));
    v.push((474, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 7), SRCALPHA);
            s.fill((185u8, 189u8, 106u8, 152u8));
            s.fill_rect((18u8, 93u8, 124u8, 255u8), Rect::new(8, -4, 6, 6));
            let mut q = Surface::with_flags((4, 9), 0);
            q = muster_auf(q, 4, 9);
            s.blit_ex(&q, (9, 5), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            s.fill_rect((22u8, 65u8, 234u8, 16u8), Rect::new(2, -4, 3, 6));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xA31EDC6AF5449555, flaeche: s }
    }));
    v.push((475, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 6), 0);
            s.fill((226u8, 166u8, 55u8, 255u8));
            let mut q = Surface::with_flags((4, 2), SRCALPHA);
            q = muster_auf(q, 4, 2);
            q.set_alpha(Some(141));
            s.blit_ex(&q, (7, -4), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            rustgame_core::draw::rect(&mut s, (57u8, 247u8, 227u8, 255u8), Rect::new(6, -2, 2, 4), 3);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xD86B6475CCB0F56D, flaeche: s }
    }));
    v.push((476, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 27), 0);
            s.fill((183u8, 182u8, 239u8, 255u8));
            let mut q = Surface::with_flags((16, 28), SRCALPHA);
            q = muster_auf(q, 16, 28);
            q.set_alpha(Some(70));
            s.blit_ex(&q, (2, 30), Some(Rect::new(1, -11, 11, 20)), rustgame_core::blend::flags::BLEND_RGBA_MAX);
            rustgame_core::draw::polygon(&mut s, (193u8, 197u8, 178u8, 255u8), &[(-3, 14), (11, 3), (12, -2), (0, 27), (-4, 18), (19, 17)], 2);
            rustgame_core::draw::rect(&mut s, (186u8, 217u8, 45u8, 255u8), Rect::new(9, 2, 12, 0), 4);
            {
                let mut teil = s.subsurface(Rect::new(5, 25, 11, 1));
                teil.fill((97u8, 57u8, 45u8, 255u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (107u8, 194u8, 212u8, 255u8), Rect::new(5, -14, 24, 25), 1, 5);
            rustgame_core::draw::ellipse(&mut s, (97u8, 137u8, 39u8, 9u8), Rect::new(3, 23, 13, 22), 0);
            s.set_clip(Some(Rect::new(19, 1, 1, 5)));
            {
                let mut teil = s.subsurface(Rect::new(4, 14, 7, 10));
                teil.fill((121u8, 184u8, 143u8, 206u8));
            }
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x1D3B14A18C333E4A, flaeche: s }
    }));
    v.push((477, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((25, 19), 0);
            s.fill((155u8, 87u8, 212u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (90u8, 227u8, 199u8, 255u8), &[(16, -1), (14, 19), (10, -2), (-4, -1)], 0);
            let mut q = Surface::with_flags((6, 6), SRCALPHA);
            q = muster_auf(q, 6, 6);
            q.set_colorkey(Some((133u8, 90u8, 124u8)));
            s.blit_ex(&q, (11, 9), Some(Rect::new(5, 7, 1, 7)), rustgame_core::blend::flags::BLEND_RGBA_MULT);
            s.set_clip(Some(Rect::new(22, 11, 2, 8)));
            let mut q = Surface::with_flags((5, 13), 0);
            q = muster_auf(q, 5, 13);
            q.set_alpha(Some(199));
            q.set_colorkey(Some((246u8, 45u8, 8u8)));
            s.blit_ex(&q, (4, 7), Some(Rect::new(3, 11, 1, 6)), rustgame_core::blend::flags::BLEND_RGBA_MULT);
            let mut q = Surface::with_flags((13, 14), 0);
            q = muster_auf(q, 13, 14);
            s.blit_ex(&q, (28, 7), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            {
                let mut teil = s.subsurface(Rect::new(14, 10, 4, 6));
                teil.fill((69u8, 17u8, 48u8, 255u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x881C088BA3D9D16A, flaeche: s }
    }));
    v.push((478, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((16, 32), SRCALPHA);
            s.fill((200u8, 163u8, 34u8, 149u8));
            rustgame_core::draw::rect_radius(&mut s, (0u8, 171u8, 154u8, 171u8), Rect::new(3, 18, 16, 31), 3, 2);
            rustgame_core::draw::rect_radius(&mut s, (62u8, 159u8, 58u8, 175u8), Rect::new(17, -15, 19, 19), 3, 4);
            rustgame_core::draw::circle(&mut s, (25u8, 83u8, 161u8, 255u8), (2, 20), 26, 2);
            rustgame_core::draw::ellipse(&mut s, (29u8, 141u8, 51u8, 0u8), Rect::new(5, 28, 11, 33), 0);
            {
                let mut teil = s.subsurface(Rect::new(10, 15, 4, 17));
                teil.fill((232u8, 98u8, 67u8, 2u8));
            }
            rustgame_core::draw::line(&mut s, (60u8, 92u8, 106u8, 255u8), (17, 32), (10, 8), 1);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xAA420D8D6FE4F9C5, flaeche: s }
    }));
    v.push((479, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((31, 19), 0);
            s.fill((75u8, 80u8, 94u8, 255u8));
            rustgame_core::draw::circle(&mut s, (4u8, 50u8, 82u8, 255u8), (4, 5), 0, 1);
            rustgame_core::draw::ellipse(&mut s, (150u8, 206u8, 32u8, 170u8), Rect::new(0, 15, 2, 22), 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xCBD5E1F012E537AD, flaeche: s }
    }));
    v.push((480, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((18, 24), 0);
            s.fill((148u8, 233u8, 107u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (100u8, 120u8, 58u8, 47u8), Rect::new(-4, 4, 17, 4), 2);
            rustgame_core::draw::polygon(&mut s, (37u8, 68u8, 23u8, 255u8), &[(7, -4), (4, 6), (9, 1), (2, 10), (7, 23)], 3);
            s.scroll(7, 20);
            {
                let mut teil = s.subsurface(Rect::new(0, 11, 2, 6));
                teil.fill((213u8, 55u8, 43u8, 255u8));
            }
            {
                let mut teil = s.subsurface(Rect::new(14, 22, 3, 2));
                teil.fill((22u8, 90u8, 206u8, 211u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (81u8, 149u8, 196u8, 100u8), Rect::new(15, 3, 9, 4), 3, 0);
            let mut q = Surface::with_flags((18, 7), SRCALPHA);
            q = muster_auf(q, 18, 7);
            s.blit_ex(&q, (6, 25), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x946CADDE3DA0E11F, flaeche: s }
    }));
    v.push((481, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((27, 28), SRCALPHA);
            s.fill((236u8, 157u8, 237u8, 71u8));
            {
                let mut teil = s.subsurface(Rect::new(3, 10, 10, 2));
                teil.fill((174u8, 126u8, 4u8, 46u8));
            }
            s.scroll(26, -3);
            {
                let mut teil = s.subsurface(Rect::new(5, 11, 1, 2));
                teil.fill((229u8, 95u8, 42u8, 85u8));
            }
            rustgame_core::draw::circle(&mut s, (44u8, 0u8, 243u8, 255u8), (7, 10), 9, 3);
            s.set_clip(Some(Rect::new(15, 13, 8, 1)));
            rustgame_core::draw::circle(&mut s, (219u8, 136u8, 78u8, 255u8), (21, 11), 7, 2);
            s = rustgame_core::transform::rotate(&s, 253.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x387DF85C59D64D95, flaeche: s }
    }));
    v.push((482, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((22, 25), SRCALPHA);
            s.fill((128u8, 88u8, 116u8, 62u8));
            rustgame_core::draw::circle(&mut s, (247u8, 4u8, 160u8, 255u8), (24, 20), 18, 3);
            let mut q = Surface::with_flags((20, 4), 0);
            q = muster_auf(q, 20, 4);
            s.blit_ex(&q, (4, 1), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            rustgame_core::draw::rect(&mut s, (184u8, 36u8, 8u8, 75u8), Rect::new(12, -8, 16, 0), 0);
            rustgame_core::draw::rect_radius(&mut s, (32u8, 191u8, 89u8, 255u8), Rect::new(15, 25, 9, 7), 4, 6);
            {
                let mut teil = s.subsurface(Rect::new(5, 12, 1, 3));
                teil.fill((34u8, 183u8, 254u8, 255u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x3473AFD680F57003, flaeche: s }
    }));
    v.push((483, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((22, 27), 0);
            s.fill((161u8, 135u8, 42u8, 255u8));
            s.fill_rect((179u8, 113u8, 113u8, 117u8), Rect::new(-6, -7, 9, 16));
            rustgame_core::draw::rect_radius(&mut s, (130u8, 56u8, 235u8, 147u8), Rect::new(16, 12, 0, 3), 0, 0);
            s = s.convert_alpha();
            s = rustgame_core::transform::scale(&s, (5, 31));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x567036A5E021ABDA, flaeche: s }
    }));
    v.push((484, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((24, 11), SRCALPHA);
            s.fill((48u8, 234u8, 125u8, 255u8));
            s.set_clip(Some(Rect::new(21, 4, 3, 1)));
            rustgame_core::draw::circle(&mut s, (171u8, 205u8, 189u8, 255u8), (4, 6), 10, 1);
            rustgame_core::draw::rect_radius(&mut s, (194u8, 99u8, 24u8, 207u8), Rect::new(11, 11, 16, 0), 4, 2);
            {
                let mut teil = s.subsurface(Rect::new(21, 0, 2, 11));
                teil.fill((92u8, 182u8, 235u8, 244u8));
            }
            let mut q = Surface::with_flags((8, 2), 0);
            q = muster_auf(q, 8, 2);
            q.set_alpha(Some(126));
            q.set_colorkey(Some((49u8, 209u8, 104u8)));
            s.blit_ex(&q, (1, 14), Some(Rect::new(-4, 0, 5, 0)), rustgame_core::blend::flags::BLEND_RGB_MULT);
            s.fill_rect((180u8, 139u8, 7u8, 255u8), Rect::new(12, 4, 21, 5));
            s.fill_rect((136u8, 188u8, 10u8, 255u8), Rect::new(19, -6, 7, 8));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x10362400DF2DA918, flaeche: s }
    }));
    v.push((485, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((10, 10), 0);
            s.fill((247u8, 174u8, 181u8, 255u8));
            let mut q = Surface::with_flags((11, 9), 0);
            q = muster_auf(q, 11, 9);
            q.set_alpha(Some(119));
            q.set_colorkey(Some((123u8, 118u8, 130u8)));
            s.blit_ex(&q, (-1, 2), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            rustgame_core::draw::line(&mut s, (219u8, 254u8, 54u8, 42u8), (12, 9), (7, 0), 1);
            s.fill_rect((110u8, 175u8, 239u8, 255u8), Rect::new(5, 5, 1, 13));
            rustgame_core::draw::ellipse(&mut s, (115u8, 45u8, 198u8, 255u8), Rect::new(2, -4, 11, 4), 4);
            s.fill_rect((114u8, 99u8, 120u8, 60u8), Rect::new(6, 4, 12, 12));
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xEA48C7A505B19464, flaeche: s }
    }));
    v.push((486, true, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((11, 11), SRCALPHA);
            s.fill((255u8, 155u8, 255u8, 255u8));
            let mut q = Surface::with_flags((2, 8), SRCALPHA);
            q = muster_auf(q, 2, 8);
            s.blit_ex(&q, (5, 6), Some(Rect::new(0, 1, 5, 10)), rustgame_core::blend::flags::BLEND_RGBA_MIN);
            rustgame_core::draw::circle(&mut s, (184u8, 232u8, 236u8, 255u8), (2, 5), 7, 1);
            {
                let mut teil = s.subsurface(Rect::new(7, 8, 2, 1));
                teil.fill((237u8, 185u8, 120u8, 255u8));
            }
            rustgame_core::draw::circle(&mut s, (180u8, 133u8, 67u8, 255u8), (13, 3), 3, 4);
            rustgame_core::draw::circle(&mut s, (171u8, 229u8, 25u8, 33u8), (-4, 0), 0, 0);
            s = s.convert_alpha();
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x3BC93D1622BADA05, flaeche: s }
    }));
    v.push((487, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((32, 32), 0);
            s.fill((190u8, 161u8, 20u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(20, 14, 8, 11));
                teil.fill((239u8, 159u8, 200u8, 247u8));
            }
            {
                let mut teil = s.subsurface(Rect::new(5, 16, 8, 6));
                teil.fill((145u8, 188u8, 104u8, 106u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (2u8, 122u8, 243u8, 44u8), Rect::new(22, -16, 33, 26), 3, 0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xD4E4D8DDBB5A5B57, flaeche: s }
    }));
    v.push((488, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((9, 22), 0);
            s.fill((161u8, 204u8, 145u8, 255u8));
            let mut q = Surface::with_flags((7, 6), SRCALPHA);
            q = muster_auf(q, 7, 6);
            s.blit_ex(&q, (10, 15), None, 0);
            let mut q = Surface::with_flags((6, 6), SRCALPHA);
            q = muster_auf(q, 6, 6);
            q.set_alpha(Some(156));
            q.set_colorkey(Some((253u8, 136u8, 46u8)));
            s.blit_ex(&q, (5, -2), None, 0);
            rustgame_core::draw::circle(&mut s, (101u8, 202u8, 221u8, 192u8), (11, 6), 21, 2);
            let mut q = Surface::with_flags((2, 10), 0);
            q = muster_auf(q, 2, 10);
            s.blit_ex(&q, (1, 21), Some(Rect::new(2, -3, 5, 0)), rustgame_core::blend::flags::BLEND_RGBA_ADD);
            let mut q = Surface::with_flags((5, 9), 0);
            q = muster_auf(q, 5, 9);
            q.set_alpha(Some(236));
            s.blit_ex(&q, (-4, 1), Some(Rect::new(2, 8, 6, 0)), rustgame_core::blend::flags::BLEND_RGBA_SUB);
            rustgame_core::draw::rect_radius(&mut s, (52u8, 134u8, 161u8, 255u8), Rect::new(10, -4, 2, 24), 1, 3);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x70D2C4AC0B3BE4EF, flaeche: s }
    }));
    v.push((489, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((11, 25), SRCALPHA);
            s.fill((7u8, 26u8, 184u8, 217u8));
            rustgame_core::draw::polygon(&mut s, (113u8, 78u8, 18u8, 108u8), &[(-2, -4), (8, 13), (-1, 8), (3, 4), (-1, -4), (8, 12)], 0);
            s.fill_rect((11u8, 149u8, 123u8, 16u8), Rect::new(1, -12, 3, 27));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x659EBE1C5078CE72, flaeche: s }
    }));
    v.push((490, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((29, 21), 0);
            s.fill((38u8, 147u8, 253u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (237u8, 82u8, 241u8, 212u8), Rect::new(0, -8, 10, 4), 1, 0);
            rustgame_core::draw::polygon(&mut s, (190u8, 192u8, 157u8, 223u8), &[(15, 21), (-2, 2), (18, 24), (12, 3), (30, 9), (9, 18)], 2);
            rustgame_core::draw::ellipse(&mut s, (234u8, 160u8, 116u8, 255u8), Rect::new(-2, -7, 6, 13), 1);
            s = rustgame_core::transform::rotate(&s, 134.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x4E710535D85F105F, flaeche: s }
    }));
    v.push((491, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 31), SRCALPHA);
            s.fill((81u8, 219u8, 124u8, 171u8));
            {
                let mut teil = s.subsurface(Rect::new(15, 2, 6, 29));
                teil.fill((254u8, 186u8, 35u8, 255u8));
            }
            let mut q = Surface::with_flags((20, 15), 0);
            q = muster_auf(q, 20, 15);
            q.set_alpha(Some(129));
            s.blit_ex(&q, (2, 9), None, 0);
            rustgame_core::draw::circle(&mut s, (121u8, 254u8, 172u8, 145u8), (4, 19), 1, 4);
            rustgame_core::draw::line(&mut s, (1u8, 114u8, 207u8, 255u8), (19, 10), (20, -1), 2);
            s.scroll(6, 21);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x7E880C9251430C3C, flaeche: s }
    }));
    v.push((492, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((31, 25), 0);
            s.fill((187u8, 146u8, 15u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (87u8, 146u8, 58u8, 255u8), Rect::new(14, -8, 17, 7), 3, 1);
            rustgame_core::draw::polygon(&mut s, (126u8, 121u8, 31u8, 255u8), &[(3, 9), (33, 16), (-1, 11), (-3, -3)], 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xEFAA2249FE638A52, flaeche: s }
    }));
    v.push((493, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((14, 27), 0);
            s.fill((154u8, 131u8, 181u8, 255u8));
            rustgame_core::draw::line(&mut s, (22u8, 133u8, 108u8, 253u8), (15, 15), (5, 11), 5);
            let mut q = Surface::with_flags((8, 24), 0);
            q = muster_auf(q, 8, 24);
            q.set_alpha(Some(155));
            s.blit_ex(&q, (-1, -1), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            let mut q = Surface::with_flags((7, 23), SRCALPHA);
            q = muster_auf(q, 7, 23);
            q.set_alpha(Some(155));
            s.blit_ex(&q, (15, -1), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            s.set_clip(Some(Rect::new(7, 11, 7, 4)));
            s.set_clip(Some(Rect::new(1, 7, 11, 8)));
            s.set_clip(Some(Rect::new(12, 19, 1, 7)));
            s = rustgame_core::transform::flip(&s, true, true);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xB17407EDF5176C1A, flaeche: s }
    }));
    v.push((494, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((26, 25), 0);
            s.fill((76u8, 9u8, 13u8, 255u8));
            let mut q = Surface::with_flags((15, 10), 0);
            q = muster_auf(q, 15, 10);
            s.blit_ex(&q, (22, 21), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            let mut q = Surface::with_flags((14, 4), SRCALPHA);
            q = muster_auf(q, 14, 4);
            q.set_alpha(Some(190));
            q.set_colorkey(Some((7u8, 178u8, 71u8)));
            s.blit_ex(&q, (5, 27), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            rustgame_core::draw::rect_radius(&mut s, (224u8, 147u8, 137u8, 134u8), Rect::new(-8, 13, 19, 25), 3, 6);
            rustgame_core::draw::polygon(&mut s, (176u8, 44u8, 160u8, 255u8), &[(1, 8), (-1, 27), (26, 1), (6, 26), (18, 16), (13, 18)], 0);
            rustgame_core::draw::rect_radius(&mut s, (18u8, 190u8, 139u8, 255u8), Rect::new(21, 17, 13, 26), 1, 0);
            rustgame_core::draw::circle(&mut s, (72u8, 66u8, 144u8, 11u8), (14, 16), 22, 2);
            s.fill_rect((161u8, 129u8, 164u8, 114u8), Rect::new(-10, 19, 4, 17));
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x5BA8C8C14EA8CD57, flaeche: s }
    }));
    v.push((495, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((27, 11), SRCALPHA);
            s.fill((180u8, 161u8, 185u8, 104u8));
            rustgame_core::draw::circle(&mut s, (95u8, 9u8, 95u8, 255u8), (10, 1), 25, 2);
            s.set_clip(Some(Rect::new(22, 10, 1, 1)));
            rustgame_core::draw::circle(&mut s, (2u8, 213u8, 201u8, 251u8), (20, 2), 5, 4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x4BC2DD98829520CF, flaeche: s }
    }));
    v.push((496, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((29, 20), SRCALPHA);
            s.fill((123u8, 91u8, 112u8, 162u8));
            rustgame_core::draw::ellipse(&mut s, (192u8, 219u8, 22u8, 164u8), Rect::new(-1, -4, 18, 17), 1);
            {
                let mut teil = s.subsurface(Rect::new(28, 5, 1, 12));
                teil.fill((108u8, 113u8, 133u8, 95u8));
            }
            let mut q = Surface::with_flags((14, 22), 0);
            q = muster_auf(q, 14, 22);
            q.set_alpha(Some(112));
            s.blit_ex(&q, (-2, 21), Some(Rect::new(-4, 15, 4, 19)), rustgame_core::blend::flags::BLEND_RGB_MULT);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xFE9BE46A8D7C9026, flaeche: s }
    }));
    v.push((497, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((14, 25), 0);
            s.fill((80u8, 171u8, 101u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(1, 2, 2, 10));
                teil.fill((53u8, 236u8, 146u8, 164u8));
            }
            rustgame_core::draw::line(&mut s, (91u8, 24u8, 75u8, 57u8), (7, 7), (12, 11), 3);
            rustgame_core::draw::rect(&mut s, (170u8, 21u8, 135u8, 255u8), Rect::new(-7, 0, 8, 19), 0);
            {
                let mut teil = s.subsurface(Rect::new(4, 12, 9, 3));
                teil.fill((146u8, 214u8, 24u8, 9u8));
            }
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x1BFD65945132110F, flaeche: s }
    }));
    v.push((498, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 19), 0);
            s.fill((126u8, 127u8, 189u8, 255u8));
            let mut q = Surface::with_flags((21, 17), 0);
            q = muster_auf(q, 21, 17);
            q.set_alpha(Some(65));
            s.blit_ex(&q, (21, 13), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
            let mut q = Surface::with_flags((13, 19), SRCALPHA);
            q = muster_auf(q, 13, 19);
            s.blit_ex(&q, (13, 13), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            rustgame_core::draw::rect_radius(&mut s, (129u8, 104u8, 160u8, 255u8), Rect::new(11, 6, 24, 11), 2, 5);
            s.scroll(5, -12);
            rustgame_core::draw::circle(&mut s, (3u8, 49u8, 122u8, 208u8), (18, -2), 16, 1);
            rustgame_core::draw::circle(&mut s, (31u8, 212u8, 144u8, 255u8), (15, -4), 20, 0);
            let mut q = Surface::with_flags((12, 19), SRCALPHA);
            q = muster_auf(q, 12, 19);
            q.set_alpha(Some(109));
            q.set_colorkey(Some((121u8, 89u8, 129u8)));
            s.blit_ex(&q, (-3, 6), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            rustgame_core::draw::line(&mut s, (242u8, 223u8, 6u8, 116u8), (5, 20), (-1, 22), 5);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xBAB50C2AC5742BCA, flaeche: s }
    }));
    v.push((499, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((9, 11), 0);
            s.fill((255u8, 154u8, 248u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (33u8, 133u8, 218u8, 129u8), Rect::new(4, 6, 11, 2), 2, 6);
            rustgame_core::draw::line(&mut s, (79u8, 45u8, 92u8, 199u8), (1, 8), (8, 14), 2);
            {
                let mut teil = s.subsurface(Rect::new(7, 5, 1, 5));
                teil.fill((190u8, 30u8, 95u8, 240u8));
            }
            rustgame_core::draw::circle(&mut s, (64u8, 99u8, 66u8, 133u8), (2, 7), 9, 3);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x74F736AE5B6E1650, flaeche: s }
    }));
    v.push((500, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((31, 20), 0);
            s.fill((239u8, 130u8, 192u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (46u8, 9u8, 103u8, 255u8), Rect::new(5, 4, 25, 9), 0);
            rustgame_core::draw::line(&mut s, (229u8, 157u8, 191u8, 255u8), (4, 11), (3, 23), 2);
            let mut q = Surface::with_flags((12, 13), SRCALPHA);
            q = muster_auf(q, 12, 13);
            s.blit_ex(&q, (34, -4), Some(Rect::new(8, 8, 5, 15)), rustgame_core::blend::flags::BLEND_RGBA_MAX);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x08CB6339DEE4128A, flaeche: s }
    }));
    v.push((501, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((26, 15), 0);
            s.fill((123u8, 167u8, 172u8, 255u8));
            let mut q = Surface::with_flags((24, 12), SRCALPHA);
            q = muster_auf(q, 24, 12);
            s.blit_ex(&q, (19, 10), Some(Rect::new(11, 0, 4, 14)), rustgame_core::blend::flags::BLEND_RGBA_MAX);
            s.fill_rect((97u8, 201u8, 189u8, 255u8), Rect::new(9, -5, 25, 0));
            rustgame_core::draw::circle(&mut s, (59u8, 78u8, 255u8, 255u8), (13, 0), 18, 0);
            {
                let mut teil = s.subsurface(Rect::new(0, 14, 11, 1));
                teil.fill((213u8, 202u8, 39u8, 87u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (92u8, 41u8, 106u8, 197u8), Rect::new(19, -1, 26, 13), 3, 5);
            rustgame_core::draw::circle(&mut s, (12u8, 136u8, 37u8, 178u8), (19, 7), 14, 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xB403761128DC1B40, flaeche: s }
    }));
    v.push((502, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((22, 17), SRCALPHA);
            s.fill((86u8, 218u8, 20u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (39u8, 126u8, 163u8, 255u8), Rect::new(1, 11, 24, 3), 2, 2);
            rustgame_core::draw::circle(&mut s, (32u8, 19u8, 35u8, 255u8), (-2, 15), 5, 1);
            rustgame_core::draw::rect(&mut s, (227u8, 51u8, 122u8, 25u8), Rect::new(-3, 7, 7, 0), 1);
            rustgame_core::draw::ellipse(&mut s, (139u8, 49u8, 128u8, 77u8), Rect::new(13, 16, 19, 15), 4);
            s.set_clip(Some(Rect::new(6, 0, 7, 10)));
            let mut q = Surface::with_flags((7, 3), SRCALPHA);
            q = muster_auf(q, 7, 3);
            q.set_colorkey(Some((223u8, 251u8, 45u8)));
            s.blit_ex(&q, (-1, 5), Some(Rect::new(-1, -2, 5, 6)), 0);
            let mut q = Surface::with_flags((4, 9), 0);
            q = muster_auf(q, 4, 9);
            q.set_colorkey(Some((13u8, 163u8, 179u8)));
            s.blit_ex(&q, (16, 15), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            s.fill_rect((250u8, 4u8, 255u8, 255u8), Rect::new(15, 6, 12, 13));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x0F6A07E500A5E201, flaeche: s }
    }));
    v.push((503, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((11, 10), SRCALPHA);
            s.fill((159u8, 70u8, 9u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (17u8, 237u8, 215u8, 14u8), Rect::new(-5, -3, 12, 10), 4);
            let mut q = Surface::with_flags((9, 11), SRCALPHA);
            q = muster_auf(q, 9, 11);
            q.set_colorkey(Some((144u8, 250u8, 69u8)));
            s.blit_ex(&q, (4, 6), Some(Rect::new(2, 10, 6, 14)), rustgame_core::blend::flags::BLEND_RGB_SUB);
            s = s.convert();
            s = rustgame_core::transform::grayscale(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x266AA87228379DFD, flaeche: s }
    }));
    v.push((504, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((20, 32), 0);
            s.fill((6u8, 98u8, 25u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(11, 19, 2, 6));
                teil.fill((208u8, 157u8, 74u8, 123u8));
            }
            {
                let mut teil = s.subsurface(Rect::new(3, 26, 4, 1));
                teil.fill((146u8, 42u8, 84u8, 229u8));
            }
            s.fill_rect((36u8, 139u8, 50u8, 29u8), Rect::new(15, 13, 5, 25));
            s = rustgame_core::transform::flip(&s, true, false);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x35A7C69F54FE294B, flaeche: s }
    }));
    v.push((505, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((10, 14), 0);
            s.fill((143u8, 173u8, 240u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (30u8, 208u8, 207u8, 179u8), Rect::new(-2, 3, 8, 14), 1, 0);
            s.fill_rect((109u8, 62u8, 220u8, 255u8), Rect::new(11, 1, 11, 1));
            let mut q = Surface::with_flags((6, 15), 0);
            q = muster_auf(q, 6, 15);
            s.blit_ex(&q, (13, -2), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            rustgame_core::draw::line(&mut s, (142u8, 4u8, 80u8, 190u8), (1, 15), (6, 9), 1);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x0B80E1420909521A, flaeche: s }
    }));
    v.push((506, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((30, 31), SRCALPHA);
            s.fill((212u8, 138u8, 120u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (176u8, 251u8, 72u8, 218u8), Rect::new(-9, -15, 24, 8), 0, 1);
            rustgame_core::draw::circle(&mut s, (15u8, 111u8, 12u8, 85u8), (3, 30), 25, 3);
            rustgame_core::draw::rect_radius(&mut s, (180u8, 60u8, 108u8, 254u8), Rect::new(18, 32, 2, 11), 3, 0);
            rustgame_core::draw::circle(&mut s, (124u8, 182u8, 97u8, 255u8), (12, 22), 15, 4);
            s.scroll(-6, 22);
            rustgame_core::draw::rect_radius(&mut s, (220u8, 157u8, 161u8, 169u8), Rect::new(-8, 25, 1, 4), 0, 1);
            rustgame_core::draw::rect_radius(&mut s, (15u8, 184u8, 179u8, 255u8), Rect::new(-6, 24, 19, 23), 4, 1);
            let mut q = Surface::with_flags((26, 6), 0);
            q = muster_auf(q, 26, 6);
            q.set_colorkey(Some((249u8, 206u8, 89u8)));
            s.blit_ex(&q, (7, -4), Some(Rect::new(17, -3, 19, 5)), rustgame_core::blend::flags::BLEND_RGBA_MIN);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x51EA6DDA37662038, flaeche: s }
    }));
    v.push((507, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 25), 0);
            s.fill((73u8, 248u8, 124u8, 255u8));
            let mut q = Surface::with_flags((22, 5), SRCALPHA);
            q = muster_auf(q, 22, 5);
            s.blit_ex(&q, (-1, 24), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            rustgame_core::draw::polygon(&mut s, (144u8, 195u8, 248u8, 209u8), &[(6, 19), (23, 21), (14, 9), (5, 18)], 1);
            s.fill_rect((3u8, 248u8, 101u8, 225u8), Rect::new(-11, 25, 16, 23));
            {
                let mut teil = s.subsurface(Rect::new(9, 4, 7, 21));
                teil.fill((187u8, 23u8, 255u8, 34u8));
            }
            s.set_clip(Some(Rect::new(0, 5, 9, 15)));
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xA0B90F3636D15E5B, flaeche: s }
    }));
    v.push((508, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((23, 31), 0);
            s.fill((253u8, 35u8, 51u8, 255u8));
            let mut q = Surface::with_flags((4, 25), 0);
            q = muster_auf(q, 4, 25);
            s.blit_ex(&q, (14, 18), Some(Rect::new(4, 10, 2, 10)), rustgame_core::blend::flags::BLEND_RGB_SUB);
            rustgame_core::draw::circle(&mut s, (54u8, 123u8, 9u8, 255u8), (7, 18), 6, 4);
            rustgame_core::draw::rect_radius(&mut s, (136u8, 101u8, 24u8, 154u8), Rect::new(7, 16, 8, 5), 0, 4);
            let mut q = Surface::with_flags((13, 8), SRCALPHA);
            q = muster_auf(q, 13, 8);
            s.blit_ex(&q, (0, 14), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xDBCE0A0B962C8BED, flaeche: s }
    }));
    v.push((509, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((28, 30), 0);
            s.fill((107u8, 84u8, 68u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (198u8, 157u8, 26u8, 158u8), Rect::new(23, 2, 1, 33), 2, 0);
            {
                let mut teil = s.subsurface(Rect::new(13, 6, 5, 3));
                teil.fill((83u8, 255u8, 197u8, 255u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x6289DE3ED93CDB85, flaeche: s }
    }));
    v.push((510, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((27, 31), 0);
            s.fill((214u8, 208u8, 54u8, 255u8));
            s.fill_rect((95u8, 80u8, 190u8, 169u8), Rect::new(3, 29, 13, 15));
            s.scroll(-6, 1);
            let mut q = Surface::with_flags((2, 18), SRCALPHA);
            q = muster_auf(q, 2, 18);
            s.blit_ex(&q, (22, 30), Some(Rect::new(0, 16, 3, 20)), rustgame_core::blend::flags::BLEND_RGB_SUB);
            rustgame_core::draw::rect_radius(&mut s, (163u8, 145u8, 13u8, 230u8), Rect::new(6, -5, 11, 13), 3, 1);
            rustgame_core::draw::circle(&mut s, (38u8, 61u8, 219u8, 181u8), (17, -4), 22, 2);
            {
                let mut teil = s.subsurface(Rect::new(3, 22, 16, 3));
                teil.fill((14u8, 236u8, 2u8, 255u8));
            }
            rustgame_core::draw::circle(&mut s, (188u8, 46u8, 54u8, 255u8), (17, 29), 2, 3);
            rustgame_core::draw::rect(&mut s, (83u8, 248u8, 111u8, 198u8), Rect::new(-6, 30, 26, 24), 1);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xA3A2731D8B913F85, flaeche: s }
    }));
    v.push((511, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((9, 20), SRCALPHA);
            s.fill((108u8, 140u8, 70u8, 255u8));
            let mut q = Surface::with_flags((2, 11), 0);
            q = muster_auf(q, 2, 11);
            q.set_alpha(Some(211));
            s.blit_ex(&q, (7, -3), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            rustgame_core::draw::circle(&mut s, (194u8, 44u8, 36u8, 69u8), (-2, 15), 9, 1);
            let mut q = Surface::with_flags((5, 10), 0);
            q = muster_auf(q, 5, 10);
            s.blit_ex(&q, (-1, 10), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x7B8380A759873C10, flaeche: s }
    }));
    v.push((512, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((6, 32), SRCALPHA);
            s.fill((210u8, 185u8, 207u8, 255u8));
            s.set_clip(Some(Rect::new(1, 15, 1, 4)));
            let mut q = Surface::with_flags((5, 9), 0);
            q = muster_auf(q, 5, 9);
            s.blit_ex(&q, (0, 2), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            rustgame_core::draw::circle(&mut s, (34u8, 106u8, 185u8, 255u8), (7, 7), 28, 3);
            rustgame_core::draw::circle(&mut s, (5u8, 73u8, 229u8, 90u8), (0, 8), 27, 1);
            rustgame_core::draw::polygon(&mut s, (39u8, 38u8, 94u8, 241u8), &[(1, 32), (6, 34), (-2, 20), (9, 3)], 0);
            rustgame_core::draw::rect_radius(&mut s, (49u8, 29u8, 3u8, 255u8), Rect::new(-3, -14, 8, 1), 2, 1);
            {
                let mut teil = s.subsurface(Rect::new(0, 18, 3, 9));
                teil.fill((122u8, 120u8, 16u8, 144u8));
            }
            {
                let mut teil = s.subsurface(Rect::new(5, 6, 1, 12));
                teil.fill((41u8, 181u8, 168u8, 255u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x234F9AEA0DC2B6F0, flaeche: s }
    }));
    v.push((513, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((6, 12), SRCALPHA);
            s.fill((146u8, 164u8, 211u8, 223u8));
            {
                let mut teil = s.subsurface(Rect::new(2, 11, 4, 1));
                teil.fill((186u8, 75u8, 93u8, 55u8));
            }
            s.fill_rect((113u8, 33u8, 97u8, 255u8), Rect::new(0, 9, 9, 1));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xD0C67E0E381F79B5, flaeche: s }
    }));
    v.push((514, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((17, 9), 0);
            s.fill((106u8, 79u8, 211u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (161u8, 165u8, 246u8, 162u8), Rect::new(18, 3, 0, 10), 0, 0);
            let mut q = Surface::with_flags((14, 5), SRCALPHA);
            q = muster_auf(q, 14, 5);
            s.blit_ex(&q, (14, -1), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            rustgame_core::draw::rect_radius(&mut s, (128u8, 46u8, 242u8, 105u8), Rect::new(2, 0, 4, 2), 2, 6);
            s.set_clip(Some(Rect::new(6, 2, 1, 4)));
            rustgame_core::draw::circle(&mut s, (67u8, 38u8, 153u8, 74u8), (-1, -1), 9, 0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x7C091511ADC209F2, flaeche: s }
    }));
    v.push((515, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((11, 14), 0);
            s.fill((68u8, 48u8, 5u8, 255u8));
            rustgame_core::draw::line(&mut s, (23u8, 85u8, 237u8, 12u8), (3, 8), (6, 11), 5);
            rustgame_core::draw::circle(&mut s, (213u8, 213u8, 117u8, 17u8), (0, 2), 7, 1);
            {
                let mut teil = s.subsurface(Rect::new(5, 10, 1, 1));
                teil.fill((6u8, 65u8, 6u8, 122u8));
            }
            rustgame_core::draw::line(&mut s, (15u8, 217u8, 224u8, 58u8), (7, 1), (14, -1), 3);
            rustgame_core::draw::polygon(&mut s, (33u8, 232u8, 210u8, 255u8), &[(13, 10), (3, 2), (-4, 2)], 1);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x30A923B2A4B1EDF5, flaeche: s }
    }));
    v.push((516, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((32, 29), 0);
            s.fill((8u8, 87u8, 183u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(9, 25, 17, 3));
                teil.fill((151u8, 224u8, 71u8, 255u8));
            }
            let mut q = Surface::with_flags((5, 15), 0);
            q = muster_auf(q, 5, 15);
            q.set_alpha(Some(229));
            s.blit_ex(&q, (1, 11), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xA09FFE88CDA237CB, flaeche: s }
    }));
    v.push((517, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((31, 19), 0);
            s.fill((206u8, 101u8, 221u8, 255u8));
            s.set_clip(Some(Rect::new(26, 15, 3, 2)));
            rustgame_core::draw::rect(&mut s, (43u8, 207u8, 230u8, 255u8), Rect::new(4, -10, 32, 20), 2);
            rustgame_core::draw::rect_radius(&mut s, (9u8, 105u8, 216u8, 125u8), Rect::new(3, 3, 16, 20), 0, 6);
            rustgame_core::draw::circle(&mut s, (190u8, 62u8, 216u8, 34u8), (10, 1), 13, 1);
            s.set_clip(Some(Rect::new(5, 11, 23, 3)));
            rustgame_core::draw::circle(&mut s, (72u8, 29u8, 180u8, 87u8), (17, 17), 1, 4);
            rustgame_core::draw::rect_radius(&mut s, (13u8, 51u8, 201u8, 34u8), Rect::new(29, -2, 33, 22), 0, 0);
            rustgame_core::draw::ellipse(&mut s, (156u8, 11u8, 242u8, 255u8), Rect::new(18, 2, 16, 3), 4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x6BC16FBB47933F4C, flaeche: s }
    }));
    v.push((518, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((32, 17), 0);
            s.fill((130u8, 131u8, 255u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(29, 0, 2, 4));
                teil.fill((70u8, 180u8, 96u8, 255u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (212u8, 65u8, 98u8, 54u8), Rect::new(28, 6, 10, 18), 2, 0);
            rustgame_core::draw::ellipse(&mut s, (186u8, 93u8, 172u8, 201u8), Rect::new(-1, -4, 7, 1), 3);
            rustgame_core::draw::rect_radius(&mut s, (18u8, 101u8, 223u8, 132u8), Rect::new(27, -3, 10, 9), 4, 5);
            s.fill_rect((42u8, 43u8, 91u8, 255u8), Rect::new(8, 9, 9, 12));
            rustgame_core::draw::polygon(&mut s, (134u8, 210u8, 216u8, 184u8), &[(-4, 10), (20, 12), (33, 2), (13, 20), (2, 2), (10, 3)], 0);
            rustgame_core::draw::ellipse(&mut s, (67u8, 51u8, 39u8, 255u8), Rect::new(33, -4, 14, 6), 0);
            s = rustgame_core::transform::scale(&s, (22, 12));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x0025E465B35B76C4, flaeche: s }
    }));
    v.push((519, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((25, 20), SRCALPHA);
            s.fill((66u8, 151u8, 114u8, 255u8));
            s.fill_rect((108u8, 73u8, 35u8, 255u8), Rect::new(13, 8, 0, 14));
            let mut q = Surface::with_flags((11, 22), 0);
            q = muster_auf(q, 11, 22);
            s.blit_ex(&q, (2, -2), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            s.scroll(14, 3);
            s.fill_rect((239u8, 154u8, 124u8, 255u8), Rect::new(13, 8, 3, 13));
            {
                let mut teil = s.subsurface(Rect::new(2, 12, 5, 1));
                teil.fill((124u8, 241u8, 76u8, 78u8));
            }
            s.fill_rect((128u8, 63u8, 116u8, 244u8), Rect::new(1, 3, 21, 16));
            s = rustgame_core::transform::grayscale(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x3DE155931A2E4B30, flaeche: s }
    }));
    v.push((520, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((6, 7), 0);
            s.fill((202u8, 206u8, 70u8, 255u8));
            let mut q = Surface::with_flags((8, 8), 0);
            q = muster_auf(q, 8, 8);
            q.set_alpha(Some(213));
            q.set_colorkey(Some((216u8, 59u8, 186u8)));
            s.blit_ex(&q, (1, 4), Some(Rect::new(2, -1, 2, 7)), rustgame_core::blend::flags::BLEND_RGBA_SUB);
            s.set_clip(Some(Rect::new(5, 0, 1, 5)));
            rustgame_core::draw::ellipse(&mut s, (215u8, 198u8, 179u8, 71u8), Rect::new(-2, -3, 5, 6), 3);
            s.set_clip(Some(Rect::new(4, 1, 1, 5)));
            rustgame_core::draw::polygon(&mut s, (88u8, 81u8, 238u8, 148u8), &[(8, 4), (6, 9), (-3, 5), (-1, -3)], 1);
            s = s.convert();
            s = rustgame_core::transform::flip(&s, true, true);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xC0EB490AD436E1E2, flaeche: s }
    }));
    v.push((521, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((9, 6), 0);
            s.fill((128u8, 209u8, 90u8, 255u8));
            let mut q = Surface::with_flags((7, 2), 0);
            q = muster_auf(q, 7, 2);
            q.set_alpha(Some(158));
            s.blit_ex(&q, (-3, 2), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            rustgame_core::draw::circle(&mut s, (199u8, 163u8, 140u8, 255u8), (11, -4), 2, 0);
            let mut q = Surface::with_flags((2, 6), 0);
            q = muster_auf(q, 2, 6);
            q.set_alpha(Some(61));
            s.blit_ex(&q, (11, 9), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            s.fill_rect((218u8, 200u8, 176u8, 255u8), Rect::new(-1, -3, 11, 0));
            let mut q = Surface::with_flags((8, 6), 0);
            q = muster_auf(q, 8, 6);
            s.blit_ex(&q, (6, -3), Some(Rect::new(1, 1, 7, 8)), rustgame_core::blend::flags::BLEND_RGBA_MIN);
            rustgame_core::draw::rect_radius(&mut s, (155u8, 78u8, 33u8, 235u8), Rect::new(4, -3, 8, 5), 1, 6);
            s.scroll(4, 1);
            {
                let mut teil = s.subsurface(Rect::new(5, 4, 4, 2));
                teil.fill((61u8, 194u8, 161u8, 202u8));
            }
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xFBDF9CEBAAC1DED4, flaeche: s }
    }));
    v.push((522, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 13), SRCALPHA);
            s.fill((68u8, 46u8, 96u8, 10u8));
            s.set_clip(Some(Rect::new(0, 10, 8, 3)));
            rustgame_core::draw::rect_radius(&mut s, (125u8, 208u8, 242u8, 158u8), Rect::new(-2, -7, 5, 12), 0, 6);
            rustgame_core::draw::circle(&mut s, (199u8, 224u8, 241u8, 255u8), (7, 12), 0, 3);
            rustgame_core::draw::rect(&mut s, (146u8, 156u8, 105u8, 69u8), Rect::new(-4, 5, 8, 13), 4);
            let mut q = Surface::with_flags((3, 9), 0);
            q = muster_auf(q, 3, 9);
            s.blit_ex(&q, (-2, 10), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            rustgame_core::draw::line(&mut s, (86u8, 10u8, 116u8, 255u8), (-1, 4), (5, 9), 1);
            rustgame_core::draw::rect_radius(&mut s, (131u8, 226u8, 113u8, 15u8), Rect::new(3, 8, 1, 16), 0, 3);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x6DCA3BFFDAF3A051, flaeche: s }
    }));
    v.push((523, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((27, 27), 0);
            s.fill((193u8, 80u8, 240u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (214u8, 201u8, 244u8, 4u8), Rect::new(9, 11, 27, 19), 3, 3);
            rustgame_core::draw::polygon(&mut s, (20u8, 82u8, 205u8, 133u8), &[(15, -3), (9, 8), (11, 21), (-1, 15)], 0);
            {
                let mut teil = s.subsurface(Rect::new(1, 18, 5, 6));
                teil.fill((194u8, 153u8, 145u8, 255u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x3AAC854AD37B6789, flaeche: s }
    }));
    v.push((524, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((13, 6), 0);
            s.fill((251u8, 172u8, 40u8, 255u8));
            s.scroll(-8, -5);
            rustgame_core::draw::rect_radius(&mut s, (123u8, 162u8, 0u8, 69u8), Rect::new(9, 1, 8, 6), 1, 5);
            let mut q = Surface::with_flags((13, 6), 0);
            q = muster_auf(q, 13, 6);
            q.set_alpha(Some(249));
            s.blit_ex(&q, (13, 8), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            rustgame_core::draw::circle(&mut s, (12u8, 139u8, 253u8, 66u8), (-3, -1), 2, 1);
            {
                let mut teil = s.subsurface(Rect::new(0, 3, 6, 2));
                teil.fill((167u8, 216u8, 128u8, 255u8));
            }
            let mut q = Surface::with_flags((4, 8), SRCALPHA);
            q = muster_auf(q, 4, 8);
            q.set_alpha(Some(105));
            s.blit_ex(&q, (-4, 4), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            s.set_clip(Some(Rect::new(12, 0, 1, 3)));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x47DD5863DC598C55, flaeche: s }
    }));
    v.push((525, true, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((32, 23), 0);
            s.fill((242u8, 87u8, 94u8, 255u8));
            s.set_clip(Some(Rect::new(17, 14, 14, 9)));
            rustgame_core::draw::polygon(&mut s, (242u8, 96u8, 227u8, 125u8), &[(4, 13), (13, 1), (32, 25), (-4, 24)], 0);
            rustgame_core::draw::line(&mut s, (193u8, 154u8, 52u8, 46u8), (31, 15), (14, 20), 5);
            s.fill_rect((165u8, 3u8, 162u8, 144u8), Rect::new(-6, 7, 18, 15));
            s.fill_rect((101u8, 34u8, 185u8, 255u8), Rect::new(31, 5, 6, 7));
            let mut q = Surface::with_flags((19, 20), 0);
            q = muster_auf(q, 19, 20);
            s.blit_ex(&q, (11, 26), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x6E1CCDFF0E11EDE1, flaeche: s }
    }));
    v.push((526, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((10, 12), SRCALPHA);
            s.fill((204u8, 246u8, 18u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (134u8, 51u8, 241u8, 96u8), Rect::new(5, -6, 8, 10), 3, 2);
            rustgame_core::draw::circle(&mut s, (245u8, 150u8, 159u8, 87u8), (10, 12), 3, 3);
            let mut q = Surface::with_flags((3, 10), 0);
            q = muster_auf(q, 3, 10);
            q.set_alpha(Some(206));
            q.set_colorkey(Some((111u8, 61u8, 173u8)));
            s.blit_ex(&q, (-4, -2), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            rustgame_core::draw::circle(&mut s, (52u8, 194u8, 228u8, 255u8), (5, -4), 7, 4);
            rustgame_core::draw::polygon(&mut s, (82u8, 86u8, 165u8, 3u8), &[(13, 10), (-2, 0), (12, -3), (12, 9), (-4, 5), (-4, 10)], 0);
            rustgame_core::draw::rect_radius(&mut s, (248u8, 208u8, 168u8, 255u8), Rect::new(5, 12, 1, 8), 3, 0);
            s.fill_rect((211u8, 50u8, 165u8, 137u8), Rect::new(4, 10, 5, 11));
            rustgame_core::draw::line(&mut s, (5u8, 48u8, 51u8, 255u8), (5, 13), (12, 14), 5);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x6BFCB9DB8E8FBC08, flaeche: s }
    }));
    v.push((527, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((19, 31), SRCALPHA);
            s.fill((161u8, 123u8, 159u8, 82u8));
            rustgame_core::draw::circle(&mut s, (85u8, 144u8, 242u8, 255u8), (17, 19), 26, 0);
            let mut q = Surface::with_flags((14, 3), SRCALPHA);
            q = muster_auf(q, 14, 3);
            s.blit_ex(&q, (11, -4), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            s.scroll(-7, -3);
            rustgame_core::draw::rect_radius(&mut s, (27u8, 186u8, 124u8, 255u8), Rect::new(2, -15, 1, 0), 2, 3);
            rustgame_core::draw::polygon(&mut s, (147u8, 4u8, 247u8, 255u8), &[(1, 1), (9, 2), (-1, 11), (20, 11), (7, 27), (6, 33)], 0);
            {
                let mut teil = s.subsurface(Rect::new(6, 18, 5, 12));
                teil.fill((100u8, 131u8, 167u8, 255u8));
            }
            s = rustgame_core::transform::grayscale(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x9FE64BED831F9A85, flaeche: s }
    }));
    v.push((528, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((31, 7), 0);
            s.fill((156u8, 48u8, 240u8, 255u8));
            rustgame_core::draw::circle(&mut s, (91u8, 203u8, 117u8, 126u8), (8, -2), 4, 2);
            rustgame_core::draw::rect_radius(&mut s, (194u8, 40u8, 151u8, 11u8), Rect::new(-3, -4, 27, 6), 1, 3);
            rustgame_core::draw::circle(&mut s, (228u8, 6u8, 95u8, 15u8), (20, 8), 24, 0);
            let mut q = Surface::with_flags((22, 6), SRCALPHA);
            q = muster_auf(q, 22, 6);
            q.set_alpha(Some(225));
            q.set_colorkey(Some((215u8, 35u8, 20u8)));
            s.blit_ex(&q, (32, -1), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            {
                let mut teil = s.subsurface(Rect::new(22, 3, 1, 1));
                teil.fill((170u8, 170u8, 169u8, 154u8));
            }
            s = rustgame_core::transform::rotate(&s, -248.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x03C90EA429EFEE99, flaeche: s }
    }));
    v.push((529, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((26, 10), 0);
            s.fill((121u8, 235u8, 104u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(13, 6, 13, 1));
                teil.fill((212u8, 81u8, 185u8, 255u8));
            }
            let mut q = Surface::with_flags((19, 7), SRCALPHA);
            q = muster_auf(q, 19, 7);
            q.set_colorkey(Some((247u8, 61u8, 65u8)));
            s.blit_ex(&q, (22, 10), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            let mut q = Surface::with_flags((24, 4), 0);
            q = muster_auf(q, 24, 4);
            q.set_colorkey(Some((95u8, 4u8, 217u8)));
            s.blit_ex(&q, (29, 1), Some(Rect::new(17, 3, 10, 3)), 0);
            s.scroll(13, -1);
            rustgame_core::draw::circle(&mut s, (190u8, 195u8, 202u8, 20u8), (0, -4), 24, 1);
            s = rustgame_core::transform::flip(&s, true, false);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x87159BE941611FCD, flaeche: s }
    }));
    v.push((530, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((16, 9), 0);
            s.fill((114u8, 250u8, 46u8, 255u8));
            let mut q = Surface::with_flags((7, 2), SRCALPHA);
            q = muster_auf(q, 7, 2);
            s.blit_ex(&q, (18, 5), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            rustgame_core::draw::ellipse(&mut s, (12u8, 11u8, 21u8, 176u8), Rect::new(2, -1, 5, 6), 0);
            s.fill_rect((136u8, 104u8, 7u8, 255u8), Rect::new(0, 3, 19, 0));
            s.set_clip(Some(Rect::new(14, 0, 1, 8)));
            s.set_clip(Some(Rect::new(6, 6, 6, 1)));
            rustgame_core::draw::rect_radius(&mut s, (153u8, 163u8, 217u8, 239u8), Rect::new(0, -2, 1, 2), 4, 4);
            s = rustgame_core::transform::scale(&s, (18, 12));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x28063759CAD9374D, flaeche: s }
    }));
    v.push((531, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((29, 27), SRCALPHA);
            s.fill((222u8, 45u8, 24u8, 179u8));
            s.set_clip(Some(Rect::new(7, 6, 20, 6)));
            s.fill_rect((118u8, 23u8, 31u8, 75u8), Rect::new(-1, 10, 25, 22));
            let mut q = Surface::with_flags((31, 29), 0);
            q = muster_auf(q, 31, 29);
            s.blit_ex(&q, (12, 4), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            s.set_clip(Some(Rect::new(8, 13, 12, 1)));
            rustgame_core::draw::ellipse(&mut s, (134u8, 121u8, 111u8, 167u8), Rect::new(14, -5, 7, 30), 4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xD3D6BD554DFFB0C9, flaeche: s }
    }));
    v.push((532, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 26), 0);
            s.fill((47u8, 249u8, 173u8, 255u8));
            s.fill_rect((71u8, 240u8, 182u8, 48u8), Rect::new(-2, 3, 11, 5));
            rustgame_core::draw::rect_radius(&mut s, (97u8, 149u8, 135u8, 255u8), Rect::new(13, -10, 3, 0), 0, 3);
            s.set_clip(Some(Rect::new(1, 23, 10, 1)));
            rustgame_core::draw::rect_radius(&mut s, (158u8, 124u8, 108u8, 255u8), Rect::new(7, -12, 0, 10), 2, 5);
            rustgame_core::draw::circle(&mut s, (240u8, 220u8, 15u8, 187u8), (15, 3), 3, 3);
            {
                let mut teil = s.subsurface(Rect::new(4, 19, 3, 6));
                teil.fill((131u8, 105u8, 230u8, 113u8));
            }
            rustgame_core::draw::circle(&mut s, (217u8, 31u8, 50u8, 255u8), (9, 16), 14, 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x5AE215118D3B9031, flaeche: s }
    }));
    v.push((533, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((18, 11), 0);
            s.fill((132u8, 211u8, 25u8, 255u8));
            let mut q = Surface::with_flags((7, 8), SRCALPHA);
            q = muster_auf(q, 7, 8);
            s.blit_ex(&q, (7, -4), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            {
                let mut teil = s.subsurface(Rect::new(16, 10, 1, 1));
                teil.fill((69u8, 0u8, 122u8, 255u8));
            }
            rustgame_core::draw::rect(&mut s, (4u8, 8u8, 243u8, 62u8), Rect::new(10, -6, 3, 3), 2);
            {
                let mut teil = s.subsurface(Rect::new(8, 9, 4, 2));
                teil.fill((70u8, 177u8, 92u8, 255u8));
            }
            let mut q = Surface::with_flags((4, 3), SRCALPHA);
            q = muster_auf(q, 4, 3);
            q.set_alpha(Some(235));
            q.set_colorkey(Some((57u8, 211u8, 38u8)));
            s.blit_ex(&q, (12, -2), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x5A991378784B8A60, flaeche: s }
    }));
    v.push((534, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((19, 23), SRCALPHA);
            s.fill((182u8, 218u8, 7u8, 128u8));
            s.fill_rect((206u8, 74u8, 89u8, 143u8), Rect::new(20, 7, 8, 20));
            s.fill_rect((164u8, 9u8, 2u8, 62u8), Rect::new(14, 11, 5, 11));
            s.fill_rect((78u8, 103u8, 218u8, 230u8), Rect::new(10, -2, 12, 17));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x3E331423C3C6D8C6, flaeche: s }
    }));
    v.push((535, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((10, 19), SRCALPHA);
            s.fill((73u8, 62u8, 119u8, 207u8));
            rustgame_core::draw::circle(&mut s, (197u8, 95u8, 254u8, 255u8), (2, 22), 0, 3);
            s.fill_rect((108u8, 70u8, 154u8, 225u8), Rect::new(6, 12, 3, 13));
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x6B1F7472588CCFCB, flaeche: s }
    }));
    v.push((536, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((29, 12), 0);
            s.fill((0u8, 192u8, 219u8, 255u8));
            let mut q = Surface::with_flags((17, 4), SRCALPHA);
            q = muster_auf(q, 17, 4);
            q.set_colorkey(Some((193u8, 46u8, 82u8)));
            s.blit_ex(&q, (10, 6), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            let mut q = Surface::with_flags((17, 2), SRCALPHA);
            q = muster_auf(q, 17, 2);
            q.set_alpha(Some(53));
            s.blit_ex(&q, (2, 9), Some(Rect::new(12, 1, 20, 1)), 0);
            s.fill_rect((231u8, 89u8, 237u8, 82u8), Rect::new(13, 0, 0, 2));
            s.fill_rect((246u8, 130u8, 228u8, 255u8), Rect::new(-3, 4, 4, 2));
            rustgame_core::draw::rect_radius(&mut s, (120u8, 160u8, 219u8, 255u8), Rect::new(-3, -2, 7, 12), 4, 3);
            rustgame_core::draw::circle(&mut s, (205u8, 118u8, 14u8, 87u8), (29, 9), 3, 4);
            let mut q = Surface::with_flags((17, 5), 0);
            q = muster_auf(q, 17, 5);
            s.blit_ex(&q, (30, 8), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x30E26D6B2ACF98F9, flaeche: s }
    }));
    v.push((537, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((29, 20), 0);
            s.fill((114u8, 21u8, 235u8, 255u8));
            rustgame_core::draw::circle(&mut s, (143u8, 161u8, 29u8, 255u8), (22, 8), 15, 2);
            rustgame_core::draw::rect_radius(&mut s, (100u8, 117u8, 72u8, 255u8), Rect::new(28, 8, 24, 22), 0, 2);
            rustgame_core::draw::ellipse(&mut s, (108u8, 105u8, 172u8, 167u8), Rect::new(-4, 4, 18, 20), 0);
            let mut q = Surface::with_flags((6, 3), 0);
            q = muster_auf(q, 6, 3);
            q.set_alpha(Some(223));
            q.set_colorkey(Some((226u8, 100u8, 76u8)));
            s.blit_ex(&q, (3, 18), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            rustgame_core::draw::circle(&mut s, (84u8, 212u8, 105u8, 89u8), (10, 23), 7, 3);
            s.fill_rect((138u8, 24u8, 124u8, 127u8), Rect::new(6, 5, 10, 16));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xA96B581716B4C7CF, flaeche: s }
    }));
    v.push((538, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((14, 15), SRCALPHA);
            s.fill((84u8, 129u8, 165u8, 255u8));
            rustgame_core::draw::line(&mut s, (197u8, 8u8, 232u8, 71u8), (8, 9), (0, 14), 3);
            let mut q = Surface::with_flags((9, 17), 0);
            q = muster_auf(q, 9, 17);
            s.blit_ex(&q, (1, 5), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            {
                let mut teil = s.subsurface(Rect::new(12, 4, 2, 5));
                teil.fill((115u8, 68u8, 45u8, 255u8));
            }
            s.set_clip(Some(Rect::new(13, 6, 1, 8)));
            rustgame_core::draw::polygon(&mut s, (141u8, 193u8, 83u8, 169u8), &[(15, 16), (-4, 13), (15, 18)], 0);
            {
                let mut teil = s.subsurface(Rect::new(11, 5, 3, 7));
                teil.fill((106u8, 128u8, 166u8, 255u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x8FFFF681B58023AC, flaeche: s }
    }));
    v.push((539, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((15, 12), 0);
            s.fill((22u8, 73u8, 189u8, 255u8));
            s.fill_rect((157u8, 61u8, 211u8, 255u8), Rect::new(10, -6, 0, 13));
            s.set_clip(Some(Rect::new(0, 2, 11, 1)));
            let mut q = Surface::with_flags((15, 10), SRCALPHA);
            q = muster_auf(q, 15, 10);
            q.set_alpha(Some(191));
            q.set_colorkey(Some((162u8, 162u8, 69u8)));
            s.blit_ex(&q, (15, 3), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x79A411F93C2D203D, flaeche: s }
    }));
    v.push((540, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((25, 25), 0);
            s.fill((14u8, 239u8, 162u8, 255u8));
            let mut q = Surface::with_flags((14, 4), SRCALPHA);
            q = muster_auf(q, 14, 4);
            q.set_colorkey(Some((4u8, 116u8, 248u8)));
            s.blit_ex(&q, (11, 5), None, rustgame_core::blend::flags::BLEND_RGBA_ADD);
            s.fill_rect((134u8, 27u8, 58u8, 132u8), Rect::new(7, -9, 14, 22));
            s.set_clip(Some(Rect::new(8, 2, 8, 12)));
            let mut q = Surface::with_flags((26, 9), SRCALPHA);
            q = muster_auf(q, 26, 9);
            q.set_alpha(Some(116));
            s.blit_ex(&q, (19, 22), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            rustgame_core::draw::rect_radius(&mut s, (30u8, 204u8, 131u8, 78u8), Rect::new(1, 26, 25, 27), 2, 6);
            rustgame_core::draw::polygon(&mut s, (150u8, 113u8, 100u8, 174u8), &[(11, 12), (4, 28), (25, 12), (21, 9), (-2, 7)], 2);
            s = rustgame_core::transform::flip(&s, true, false);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xECAE639D53C098C0, flaeche: s }
    }));
    v.push((541, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((17, 6), SRCALPHA);
            s.fill((89u8, 177u8, 162u8, 255u8));
            rustgame_core::draw::circle(&mut s, (135u8, 6u8, 155u8, 92u8), (15, 5), 3, 2);
            rustgame_core::draw::rect_radius(&mut s, (112u8, 237u8, 185u8, 205u8), Rect::new(-5, 1, 3, 7), 1, 6);
            rustgame_core::draw::polygon(&mut s, (138u8, 238u8, 106u8, 188u8), &[(3, 1), (20, -4), (1, 4), (15, 7), (15, 4)], 0);
            rustgame_core::draw::rect_radius(&mut s, (160u8, 67u8, 8u8, 255u8), Rect::new(11, -1, 10, 0), 4, 4);
            {
                let mut teil = s.subsurface(Rect::new(10, 1, 5, 4));
                teil.fill((179u8, 73u8, 44u8, 255u8));
            }
            s = rustgame_core::transform::flip(&s, false, true);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x8C6A316A50ECEEDE, flaeche: s }
    }));
    v.push((542, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 9), SRCALPHA);
            s.fill((230u8, 186u8, 67u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(0, 2, 1, 2));
                teil.fill((14u8, 189u8, 224u8, 255u8));
            }
            let mut q = Surface::with_flags((14, 8), 0);
            q = muster_auf(q, 14, 8);
            s.blit_ex(&q, (18, -2), Some(Rect::new(5, -4, 9, 5)), rustgame_core::blend::flags::BLEND_RGB_MAX);
            {
                let mut teil = s.subsurface(Rect::new(9, 2, 3, 2));
                teil.fill((97u8, 250u8, 235u8, 255u8));
            }
            {
                let mut teil = s.subsurface(Rect::new(7, 5, 6, 4));
                teil.fill((104u8, 63u8, 5u8, 255u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x3A2195156230D64B, flaeche: s }
    }));
    v.push((543, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((32, 26), 0);
            s.fill((64u8, 46u8, 238u8, 255u8));
            rustgame_core::draw::circle(&mut s, (141u8, 43u8, 36u8, 94u8), (29, 25), 24, 2);
            rustgame_core::draw::circle(&mut s, (57u8, 9u8, 141u8, 163u8), (-3, 24), 21, 4);
            s.set_clip(Some(Rect::new(25, 19, 4, 6)));
            s.set_clip(Some(Rect::new(14, 24, 5, 2)));
            s.set_clip(Some(Rect::new(13, 5, 17, 15)));
            rustgame_core::draw::polygon(&mut s, (64u8, 55u8, 177u8, 255u8), &[(6, -4), (16, 12), (1, 10)], 3);
            let mut q = Surface::with_flags((17, 23), SRCALPHA);
            q = muster_auf(q, 17, 23);
            s.blit_ex(&q, (8, 4), Some(Rect::new(1, -4, 18, 13)), 0);
            {
                let mut teil = s.subsurface(Rect::new(24, 6, 5, 8));
                teil.fill((33u8, 58u8, 151u8, 255u8));
            }
            s = rustgame_core::transform::scale(&s, (33, 20));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xDE832157A0BB4D71, flaeche: s }
    }));
    v.push((544, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((27, 30), SRCALPHA);
            s.fill((142u8, 34u8, 10u8, 255u8));
            let mut q = Surface::with_flags((3, 14), SRCALPHA);
            q = muster_auf(q, 3, 14);
            q.set_colorkey(Some((9u8, 93u8, 149u8)));
            s.blit_ex(&q, (20, 0), None, 0);
            let mut q = Surface::with_flags((4, 18), SRCALPHA);
            q = muster_auf(q, 4, 18);
            q.set_alpha(Some(200));
            s.blit_ex(&q, (17, 32), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            s.scroll(-6, -18);
            {
                let mut teil = s.subsurface(Rect::new(18, 5, 7, 25));
                teil.fill((62u8, 78u8, 197u8, 34u8));
            }
            rustgame_core::draw::circle(&mut s, (225u8, 0u8, 184u8, 176u8), (25, 23), 8, 4);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xEF2FDB9C137255ED, flaeche: s }
    }));
    v.push((545, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 26), 0);
            s.fill((28u8, 118u8, 255u8, 255u8));
            let mut q = Surface::with_flags((7, 24), 0);
            q = muster_auf(q, 7, 24);
            q.set_alpha(Some(109));
            s.blit_ex(&q, (-1, 13), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            rustgame_core::draw::polygon(&mut s, (207u8, 209u8, 35u8, 39u8), &[(4, -2), (0, 6), (0, 10), (7, 24)], 0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xDDAC6AE733D5DD17, flaeche: s }
    }));
    v.push((546, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((6, 14), SRCALPHA);
            s.fill((195u8, 76u8, 167u8, 188u8));
            {
                let mut teil = s.subsurface(Rect::new(2, 8, 3, 1));
                teil.fill((187u8, 77u8, 145u8, 125u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (215u8, 113u8, 61u8, 255u8), Rect::new(1, -1, 5, 8), 2, 4);
            s.fill_rect((230u8, 125u8, 111u8, 255u8), Rect::new(-3, 6, 5, 16));
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x0F1BB0DB3C98FB29, flaeche: s }
    }));
    v.push((547, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 8), 0);
            s.fill((160u8, 249u8, 104u8, 255u8));
            let mut q = Surface::with_flags((5, 9), 0);
            q = muster_auf(q, 5, 9);
            q.set_alpha(Some(237));
            s.blit_ex(&q, (7, -3), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            rustgame_core::draw::rect_radius(&mut s, (228u8, 255u8, 100u8, 255u8), Rect::new(-2, 0, 1, 5), 4, 0);
            rustgame_core::draw::rect_radius(&mut s, (55u8, 10u8, 226u8, 224u8), Rect::new(1, 8, 0, 3), 4, 6);
            let mut q = Surface::with_flags((4, 2), SRCALPHA);
            q = muster_auf(q, 4, 2);
            q.set_colorkey(Some((177u8, 245u8, 98u8)));
            s.blit_ex(&q, (10, 4), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            s.fill_rect((217u8, 142u8, 215u8, 65u8), Rect::new(-3, 1, 3, 10));
            s = rustgame_core::transform::scale(&s, (9, 9));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x23BEC578177087E3, flaeche: s }
    }));
    v.push((548, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 31), SRCALPHA);
            s.fill((201u8, 85u8, 251u8, 40u8));
            {
                let mut teil = s.subsurface(Rect::new(6, 2, 1, 29));
                teil.fill((5u8, 4u8, 201u8, 174u8));
            }
            rustgame_core::draw::ellipse(&mut s, (108u8, 236u8, 217u8, 255u8), Rect::new(-4, 28, 1, 24), 2);
            s.scroll(5, 14);
            rustgame_core::draw::ellipse(&mut s, (211u8, 23u8, 118u8, 208u8), Rect::new(3, -5, 0, 5), 3);
            rustgame_core::draw::ellipse(&mut s, (245u8, 16u8, 75u8, 255u8), Rect::new(4, 30, 7, 32), 2);
            rustgame_core::draw::line(&mut s, (126u8, 34u8, 208u8, 128u8), (-2, 19), (-1, 18), 4);
            rustgame_core::draw::ellipse(&mut s, (175u8, 100u8, 192u8, 44u8), Rect::new(-3, 28, 6, 28), 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xEC7F2B9406149271, flaeche: s }
    }));
    v.push((549, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((26, 12), SRCALPHA);
            s.fill((93u8, 147u8, 197u8, 249u8));
            rustgame_core::draw::line(&mut s, (45u8, 38u8, 104u8, 255u8), (8, 1), (28, -3), 1);
            let mut q = Surface::with_flags((10, 5), 0);
            q = muster_auf(q, 10, 5);
            s.blit_ex(&q, (18, 0), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x20F576DD8CAD8126, flaeche: s }
    }));
    v.push((550, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((32, 21), SRCALPHA);
            s.fill((124u8, 52u8, 219u8, 19u8));
            rustgame_core::draw::rect_radius(&mut s, (102u8, 37u8, 1u8, 11u8), Rect::new(14, 7, 5, 2), 3, 4);
            s.set_clip(Some(Rect::new(18, 0, 4, 14)));
            let mut q = Surface::with_flags((8, 11), 0);
            q = muster_auf(q, 8, 11);
            q.set_alpha(Some(252));
            s.blit_ex(&q, (-2, 11), Some(Rect::new(8, -3, 7, 0)), rustgame_core::blend::flags::BLEND_RGBA_MULT);
            rustgame_core::draw::circle(&mut s, (174u8, 152u8, 204u8, 255u8), (35, 12), 11, 0);
            s.scroll(-14, -21);
            rustgame_core::draw::circle(&mut s, (45u8, 87u8, 92u8, 255u8), (5, 20), 17, 2);
            s = rustgame_core::transform::scale(&s, (30, 24));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x6A1CBE5E92317B92, flaeche: s }
    }));
    v.push((551, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((22, 26), 0);
            s.fill((121u8, 7u8, 106u8, 255u8));
            let mut q = Surface::with_flags((15, 14), 0);
            q = muster_auf(q, 15, 14);
            s.blit_ex(&q, (1, 12), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            rustgame_core::draw::line(&mut s, (2u8, 245u8, 88u8, 222u8), (-3, 28), (25, 15), 3);
            let mut q = Surface::with_flags((14, 19), 0);
            q = muster_auf(q, 14, 19);
            q.set_colorkey(Some((220u8, 134u8, 187u8)));
            s.blit_ex(&q, (14, 3), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            s = s.convert_alpha();
            s = rustgame_core::transform::rotate(&s, -342.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xE6A9B817F7D9C110, flaeche: s }
    }));
    v.push((552, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((6, 29), SRCALPHA);
            s.fill((20u8, 119u8, 195u8, 197u8));
            rustgame_core::draw::rect_radius(&mut s, (114u8, 106u8, 184u8, 54u8), Rect::new(2, 29, 5, 8), 0, 0);
            let mut q = Surface::with_flags((8, 20), SRCALPHA);
            q = muster_auf(q, 8, 20);
            s.blit_ex(&q, (1, 1), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            let mut q = Surface::with_flags((3, 30), 0);
            q = muster_auf(q, 3, 30);
            q.set_alpha(Some(100));
            s.blit_ex(&q, (2, -3), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            rustgame_core::draw::line(&mut s, (127u8, 98u8, 163u8, 255u8), (8, 32), (3, 9), 4);
            s.scroll(4, 16);
            s.set_clip(Some(Rect::new(0, 17, 1, 5)));
            rustgame_core::draw::line(&mut s, (81u8, 3u8, 188u8, 72u8), (6, 1), (9, 6), 4);
            let mut q = Surface::with_flags((5, 18), SRCALPHA);
            q = muster_auf(q, 5, 18);
            s.blit_ex(&q, (0, 32), Some(Rect::new(0, 5, 1, 10)), rustgame_core::blend::flags::BLEND_RGB_MIN);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x4DE6744BA9C282F1, flaeche: s }
    }));
    v.push((553, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((21, 6), SRCALPHA);
            s.fill((197u8, 230u8, 89u8, 209u8));
            rustgame_core::draw::ellipse(&mut s, (230u8, 91u8, 218u8, 14u8), Rect::new(-9, 2, 0, 4), 3);
            rustgame_core::draw::circle(&mut s, (37u8, 13u8, 116u8, 95u8), (-4, -1), 13, 4);
            rustgame_core::draw::rect_radius(&mut s, (119u8, 120u8, 246u8, 255u8), Rect::new(20, 1, 9, 8), 3, 4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x8FC3C490EE5948D5, flaeche: s }
    }));
    v.push((554, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((22, 16), SRCALPHA);
            s.fill((106u8, 192u8, 76u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(9, 11, 2, 2));
                teil.fill((31u8, 97u8, 247u8, 255u8));
            }
            s.fill_rect((85u8, 241u8, 66u8, 255u8), Rect::new(21, 10, 8, 8));
            s = rustgame_core::transform::grayscale(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x57B952409D674195, flaeche: s }
    }));
    v.push((555, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 14), SRCALPHA);
            s.fill((78u8, 130u8, 210u8, 54u8));
            rustgame_core::draw::rect_radius(&mut s, (9u8, 64u8, 17u8, 231u8), Rect::new(9, 7, 1, 3), 0, 0);
            s.scroll(7, 11);
            rustgame_core::draw::line(&mut s, (165u8, 75u8, 62u8, 131u8), (13, 16), (-3, 13), 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x55D44CD15369D868, flaeche: s }
    }));
    v.push((556, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((28, 21), SRCALPHA);
            s.fill((147u8, 19u8, 15u8, 237u8));
            rustgame_core::draw::line(&mut s, (89u8, 41u8, 15u8, 255u8), (13, 21), (-2, 19), 1);
            {
                let mut teil = s.subsurface(Rect::new(4, 8, 5, 3));
                teil.fill((237u8, 46u8, 27u8, 193u8));
            }
            rustgame_core::draw::circle(&mut s, (254u8, 247u8, 233u8, 194u8), (27, 19), 18, 0);
            {
                let mut teil = s.subsurface(Rect::new(24, 7, 4, 8));
                teil.fill((141u8, 46u8, 118u8, 255u8));
            }
            let mut q = Surface::with_flags((24, 11), SRCALPHA);
            q = muster_auf(q, 24, 11);
            q.set_alpha(Some(171));
            s.blit_ex(&q, (30, 2), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            s = rustgame_core::transform::grayscale(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xCEA88B536B6D9FF3, flaeche: s }
    }));
    v.push((557, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((6, 22), SRCALPHA);
            s.fill((207u8, 97u8, 90u8, 131u8));
            s.fill_rect((138u8, 64u8, 35u8, 70u8), Rect::new(4, 16, 0, 16));
            let mut q = Surface::with_flags((6, 9), 0);
            q = muster_auf(q, 6, 9);
            s.blit_ex(&q, (-3, 10), Some(Rect::new(6, 4, 6, 8)), rustgame_core::blend::flags::BLEND_RGB_MAX);
            rustgame_core::draw::line(&mut s, (191u8, 8u8, 70u8, 183u8), (-1, 11), (6, -1), 4);
            s.fill_rect((255u8, 94u8, 226u8, 110u8), Rect::new(2, 9, 4, 14));
            rustgame_core::draw::line(&mut s, (1u8, 21u8, 175u8, 36u8), (6, 9), (-2, 20), 1);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x80338A455393C4CE, flaeche: s }
    }));
    v.push((558, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((18, 14), 0);
            s.fill((10u8, 89u8, 231u8, 255u8));
            s.fill_rect((12u8, 90u8, 75u8, 85u8), Rect::new(15, 0, 16, 9));
            let mut q = Surface::with_flags((11, 14), 0);
            q = muster_auf(q, 11, 14);
            s.blit_ex(&q, (18, 3), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            rustgame_core::draw::rect(&mut s, (120u8, 9u8, 50u8, 255u8), Rect::new(6, 1, 9, 8), 1);
            s.set_clip(Some(Rect::new(0, 1, 18, 7)));
            s.fill_rect((65u8, 169u8, 42u8, 137u8), Rect::new(2, 4, 1, 13));
            rustgame_core::draw::rect_radius(&mut s, (119u8, 42u8, 240u8, 255u8), Rect::new(-4, 5, 5, 8), 1, 4);
            rustgame_core::draw::circle(&mut s, (222u8, 171u8, 190u8, 66u8), (8, 2), 17, 3);
            rustgame_core::draw::rect_radius(&mut s, (78u8, 132u8, 229u8, 243u8), Rect::new(18, -6, 12, 13), 1, 6);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xE9A19C05B0FA7616, flaeche: s }
    }));
    v.push((559, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((6, 25), 0);
            s.fill((174u8, 185u8, 71u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (72u8, 87u8, 2u8, 171u8), Rect::new(2, 20, 6, 21), 0, 6);
            rustgame_core::draw::circle(&mut s, (183u8, 183u8, 63u8, 182u8), (0, 4), 9, 1);
            rustgame_core::draw::rect_radius(&mut s, (21u8, 158u8, 165u8, 145u8), Rect::new(7, -1, 5, 28), 0, 3);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x15750DD909C69AF0, flaeche: s }
    }));
    v.push((560, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((30, 26), SRCALPHA);
            s.fill((196u8, 180u8, 123u8, 1u8));
            s.fill_rect((123u8, 78u8, 247u8, 255u8), Rect::new(-7, 18, 19, 2));
            s.fill_rect((144u8, 230u8, 115u8, 90u8), Rect::new(22, -8, 10, 14));
            rustgame_core::draw::polygon(&mut s, (111u8, 218u8, 143u8, 165u8), &[(2, 0), (6, 10), (29, 12), (33, 10)], 0);
            s = rustgame_core::transform::flip(&s, false, true);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x7E60782710B2AE70, flaeche: s }
    }));
    v.push((561, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((24, 22), SRCALPHA);
            s.fill((103u8, 115u8, 178u8, 102u8));
            rustgame_core::draw::rect_radius(&mut s, (106u8, 222u8, 4u8, 226u8), Rect::new(-8, 23, 22, 23), 1, 6);
            rustgame_core::draw::line(&mut s, (228u8, 103u8, 37u8, 255u8), (9, 19), (13, -1), 1);
            s.set_clip(Some(Rect::new(2, 9, 4, 9)));
            s.scroll(-13, -17);
            let mut q = Surface::with_flags((22, 20), 0);
            q = muster_auf(q, 22, 20);
            s.blit_ex(&q, (2, 3), Some(Rect::new(17, 19, 23, 0)), rustgame_core::blend::flags::BLEND_RGB_ADD);
            s.fill_rect((95u8, 209u8, 67u8, 195u8), Rect::new(6, 3, 18, 4));
            rustgame_core::draw::rect_radius(&mut s, (162u8, 91u8, 241u8, 255u8), Rect::new(-4, 20, 2, 0), 0, 5);
            rustgame_core::draw::line(&mut s, (161u8, 177u8, 146u8, 71u8), (-3, 20), (26, -1), 1);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x3534ACE3A78047C9, flaeche: s }
    }));
    v.push((562, true, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((29, 17), 0);
            s.fill((61u8, 29u8, 100u8, 255u8));
            rustgame_core::draw::circle(&mut s, (183u8, 216u8, 231u8, 93u8), (-1, 10), 28, 3);
            let mut q = Surface::with_flags((21, 2), SRCALPHA);
            q = muster_auf(q, 21, 2);
            q.set_alpha(Some(198));
            s.blit_ex(&q, (1, 15), Some(Rect::new(-8, 0, 3, 0)), rustgame_core::blend::flags::BLEND_RGBA_ADD);
            s = s.convert_alpha();
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xD26BF711BDACFEED, flaeche: s }
    }));
    v.push((563, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((22, 29), 0);
            s.fill((155u8, 2u8, 88u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (232u8, 70u8, 167u8, 255u8), &[(4, -4), (23, 8), (-2, 32), (17, 24), (11, 25)], 3);
            rustgame_core::draw::rect_radius(&mut s, (12u8, 7u8, 134u8, 255u8), Rect::new(11, -15, 21, 9), 0, 6);
            rustgame_core::draw::rect_radius(&mut s, (220u8, 221u8, 104u8, 251u8), Rect::new(-11, -2, 12, 0), 1, 5);
            let mut q = Surface::with_flags((18, 13), 0);
            q = muster_auf(q, 18, 13);
            q.set_colorkey(Some((44u8, 153u8, 120u8)));
            s.blit_ex(&q, (21, 13), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            rustgame_core::draw::line(&mut s, (179u8, 22u8, 221u8, 255u8), (-3, 15), (17, 24), 1);
            rustgame_core::draw::circle(&mut s, (84u8, 194u8, 196u8, 255u8), (21, 26), 23, 4);
            rustgame_core::draw::rect_radius(&mut s, (158u8, 117u8, 30u8, 69u8), Rect::new(2, -10, 10, 13), 2, 1);
            rustgame_core::draw::circle(&mut s, (138u8, 35u8, 144u8, 255u8), (-4, 25), 1, 2);
            s = rustgame_core::transform::rotate(&s, 287.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xEA20FB6605645AC9, flaeche: s }
    }));
    v.push((564, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((15, 12), 0);
            s.fill((28u8, 157u8, 221u8, 255u8));
            let mut q = Surface::with_flags((6, 4), 0);
            q = muster_auf(q, 6, 4);
            q.set_alpha(Some(244));
            s.blit_ex(&q, (3, 4), Some(Rect::new(-1, 4, 8, 5)), rustgame_core::blend::flags::BLEND_RGBA_SUB);
            let mut q = Surface::with_flags((10, 10), SRCALPHA);
            q = muster_auf(q, 10, 10);
            q.set_alpha(Some(192));
            s.blit_ex(&q, (18, 3), Some(Rect::new(-4, 8, 1, 8)), 0);
            let mut q = Surface::with_flags((16, 8), 0);
            q = muster_auf(q, 16, 8);
            q.set_alpha(Some(139));
            s.blit_ex(&q, (7, 5), Some(Rect::new(-4, 5, 19, 0)), rustgame_core::blend::flags::BLEND_RGB_MAX);
            rustgame_core::draw::circle(&mut s, (202u8, 227u8, 211u8, 186u8), (1, 1), 12, 0);
            let mut q = Surface::with_flags((11, 4), SRCALPHA);
            q = muster_auf(q, 11, 4);
            s.blit_ex(&q, (16, 5), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            let mut q = Surface::with_flags((16, 11), 0);
            q = muster_auf(q, 16, 11);
            s.blit_ex(&q, (-3, -2), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            rustgame_core::draw::ellipse(&mut s, (58u8, 147u8, 18u8, 197u8), Rect::new(6, 11, 13, 9), 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x35EADF44EDD19381, flaeche: s }
    }));
    v.push((565, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((26, 31), 0);
            s.fill((165u8, 193u8, 219u8, 255u8));
            s.scroll(-19, -11);
            rustgame_core::draw::polygon(&mut s, (83u8, 79u8, 160u8, 114u8), &[(28, 17), (3, 26), (12, 32), (12, 15), (13, -1)], 0);
            s.set_clip(Some(Rect::new(4, 4, 12, 3)));
            let mut q = Surface::with_flags((27, 15), 0);
            q = muster_auf(q, 27, 15);
            q.set_colorkey(Some((128u8, 127u8, 42u8)));
            s.blit_ex(&q, (8, -3), Some(Rect::new(24, 12, 25, 4)), rustgame_core::blend::flags::BLEND_RGBA_SUB);
            rustgame_core::draw::line(&mut s, (31u8, 80u8, 50u8, 151u8), (14, 13), (0, 1), 1);
            s = rustgame_core::transform::flip(&s, false, false);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xFA0C6C03F54122B9, flaeche: s }
    }));
    v.push((566, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((31, 21), 0);
            s.fill((158u8, 65u8, 221u8, 255u8));
            let mut q = Surface::with_flags((25, 12), SRCALPHA);
            q = muster_auf(q, 25, 12);
            q.set_alpha(Some(56));
            q.set_colorkey(Some((234u8, 90u8, 76u8)));
            s.blit_ex(&q, (9, 16), Some(Rect::new(11, 4, 14, 1)), rustgame_core::blend::flags::BLEND_RGB_MULT);
            rustgame_core::draw::rect_radius(&mut s, (44u8, 98u8, 190u8, 148u8), Rect::new(23, 3, 26, 16), 2, 1);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x3708C9DA9693260E, flaeche: s }
    }));
    v.push((567, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((10, 16), SRCALPHA);
            s.fill((238u8, 201u8, 14u8, 144u8));
            s.set_clip(Some(Rect::new(5, 10, 1, 6)));
            rustgame_core::draw::polygon(&mut s, (116u8, 60u8, 218u8, 204u8), &[(13, -4), (7, 19), (10, 10), (-1, 2), (13, 10), (0, -3)], 2);
            rustgame_core::draw::polygon(&mut s, (120u8, 53u8, 190u8, 52u8), &[(8, 6), (8, 7), (8, 1), (-2, -2), (9, 18), (7, 1)], 3);
            rustgame_core::draw::circle(&mut s, (13u8, 13u8, 105u8, 255u8), (13, 6), 1, 4);
            rustgame_core::draw::line(&mut s, (213u8, 7u8, 180u8, 255u8), (10, 16), (3, -1), 3);
            {
                let mut teil = s.subsurface(Rect::new(8, 7, 1, 9));
                teil.fill((138u8, 191u8, 139u8, 131u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (168u8, 122u8, 48u8, 138u8), Rect::new(5, -1, 5, 4), 4, 3);
            s.scroll(-4, -13);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x7F6ED197DAE53A63, flaeche: s }
    }));
    v.push((568, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((9, 7), 0);
            s.fill((248u8, 139u8, 204u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (167u8, 220u8, 165u8, 255u8), Rect::new(5, -3, 6, 0), 0);
            s.scroll(-4, 0);
            s.set_clip(Some(Rect::new(5, 1, 4, 6)));
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x7235CBD5DDE2BC65, flaeche: s }
    }));
    v.push((569, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((25, 23), SRCALPHA);
            s.fill((135u8, 135u8, 38u8, 255u8));
            let mut q = Surface::with_flags((5, 17), 0);
            q = muster_auf(q, 5, 17);
            s.blit_ex(&q, (15, -4), Some(Rect::new(4, 8, 1, 14)), 0);
            rustgame_core::draw::line(&mut s, (175u8, 105u8, 48u8, 255u8), (27, 8), (24, 12), 2);
            rustgame_core::draw::line(&mut s, (56u8, 83u8, 166u8, 255u8), (7, 14), (22, -4), 3);
            rustgame_core::draw::ellipse(&mut s, (221u8, 110u8, 181u8, 47u8), Rect::new(0, 16, 8, 23), 4);
            s.fill_rect((140u8, 162u8, 244u8, 255u8), Rect::new(7, 14, 16, 15));
            let mut q = Surface::with_flags((22, 25), 0);
            q = muster_auf(q, 22, 25);
            q.set_alpha(Some(71));
            s.blit_ex(&q, (17, 21), None, 0);
            s.scroll(-13, -7);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x9F6E2D6633CB68C2, flaeche: s }
    }));
    v.push((570, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 12), SRCALPHA);
            s.fill((133u8, 170u8, 118u8, 255u8));
            let mut q = Surface::with_flags((2, 8), 0);
            q = muster_auf(q, 2, 8);
            q.set_colorkey(Some((4u8, 228u8, 174u8)));
            s.blit_ex(&q, (-4, 9), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
            {
                let mut teil = s.subsurface(Rect::new(0, 9, 7, 3));
                teil.fill((87u8, 162u8, 222u8, 120u8));
            }
            {
                let mut teil = s.subsurface(Rect::new(6, 8, 1, 3));
                teil.fill((248u8, 15u8, 220u8, 194u8));
            }
            s = rustgame_core::transform::scale(&s, (10, 12));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xC047ED8D010AB699, flaeche: s }
    }));
    v.push((571, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((10, 14), 0);
            s.fill((215u8, 26u8, 64u8, 255u8));
            rustgame_core::draw::rect(&mut s, (93u8, 170u8, 11u8, 86u8), Rect::new(3, 4, 10, 2), 3);
            rustgame_core::draw::rect_radius(&mut s, (166u8, 27u8, 20u8, 143u8), Rect::new(7, 5, 12, 1), 0, 3);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x9456F5EEFDA75256, flaeche: s }
    }));
    v.push((572, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 18), 0);
            s.fill((141u8, 132u8, 247u8, 255u8));
            s.fill_rect((72u8, 83u8, 117u8, 255u8), Rect::new(6, 16, 11, 15));
            let mut q = Surface::with_flags((5, 20), 0);
            q = muster_auf(q, 5, 20);
            q.set_colorkey(Some((124u8, 159u8, 112u8)));
            s.blit_ex(&q, (1, 12), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
            rustgame_core::draw::circle(&mut s, (168u8, 215u8, 92u8, 255u8), (5, 4), 15, 1);
            let mut q = Surface::with_flags((12, 15), SRCALPHA);
            q = muster_auf(q, 12, 15);
            q.set_alpha(Some(38));
            q.set_colorkey(Some((67u8, 53u8, 124u8)));
            s.blit_ex(&q, (13, 13), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
            rustgame_core::draw::rect_radius(&mut s, (85u8, 96u8, 234u8, 255u8), Rect::new(-1, 16, 4, 7), 0, 3);
            s.fill_rect((192u8, 180u8, 243u8, 27u8), Rect::new(11, 4, 1, 14));
            s.fill_rect((64u8, 149u8, 249u8, 200u8), Rect::new(2, 11, 1, 15));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xD468402D070414BC, flaeche: s }
    }));
    v.push((573, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((28, 29), 0);
            s.fill((133u8, 115u8, 33u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (61u8, 241u8, 136u8, 92u8), Rect::new(10, 9, 30, 23), 4, 6);
            let mut q = Surface::with_flags((21, 18), SRCALPHA);
            q = muster_auf(q, 21, 18);
            s.blit_ex(&q, (0, -1), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            rustgame_core::draw::circle(&mut s, (233u8, 162u8, 66u8, 174u8), (19, 29), 1, 1);
            rustgame_core::draw::polygon(&mut s, (201u8, 156u8, 237u8, 255u8), &[(14, 6), (5, 6), (23, 28), (16, 9)], 1);
            rustgame_core::draw::line(&mut s, (119u8, 167u8, 94u8, 236u8), (0, 16), (19, 26), 4);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x86F373CD6815DACC, flaeche: s }
    }));
    v.push((574, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((15, 24), 0);
            s.fill((51u8, 158u8, 118u8, 255u8));
            rustgame_core::draw::polygon(&mut s, (112u8, 138u8, 21u8, 19u8), &[(16, 22), (8, 7), (14, 15), (8, 17), (17, 1), (3, 7)], 0);
            s.fill_rect((25u8, 82u8, 209u8, 153u8), Rect::new(8, 10, 11, 17));
            s.set_clip(Some(Rect::new(10, 2, 3, 14)));
            s.fill_rect((134u8, 63u8, 127u8, 199u8), Rect::new(12, -4, 7, 3));
            let mut q = Surface::with_flags((16, 11), 0);
            q = muster_auf(q, 16, 11);
            s.blit_ex(&q, (10, 7), None, rustgame_core::blend::flags::BLEND_RGBA_MULT);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x40889A9EB8AE8D59, flaeche: s }
    }));
    v.push((575, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((15, 29), 0);
            s.fill((21u8, 198u8, 56u8, 255u8));
            let mut q = Surface::with_flags((12, 5), SRCALPHA);
            q = muster_auf(q, 12, 5);
            q.set_alpha(Some(146));
            s.blit_ex(&q, (18, 32), Some(Rect::new(0, -3, 7, 5)), rustgame_core::blend::flags::BLEND_RGBA_ADD);
            let mut q = Surface::with_flags((16, 22), 0);
            q = muster_auf(q, 16, 22);
            q.set_colorkey(Some((176u8, 53u8, 104u8)));
            s.blit_ex(&q, (-2, 3), None, 0);
            rustgame_core::draw::circle(&mut s, (84u8, 52u8, 171u8, 255u8), (0, 16), 13, 0);
            let mut q = Surface::with_flags((6, 25), SRCALPHA);
            q = muster_auf(q, 6, 25);
            s.blit_ex(&q, (15, -2), Some(Rect::new(1, 2, 8, 13)), rustgame_core::blend::flags::BLEND_RGB_MIN);
            rustgame_core::draw::rect_radius(&mut s, (93u8, 56u8, 205u8, 255u8), Rect::new(-5, -2, 17, 25), 4, 0);
            s = rustgame_core::transform::flip(&s, false, false);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x5EBD30FA4625BA7E, flaeche: s }
    }));
    v.push((576, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((30, 27), 0);
            s.fill((250u8, 100u8, 1u8, 255u8));
            rustgame_core::draw::line(&mut s, (44u8, 102u8, 130u8, 255u8), (25, 12), (5, -4), 3);
            rustgame_core::draw::ellipse(&mut s, (106u8, 180u8, 214u8, 255u8), Rect::new(-6, -5, 32, 15), 3);
            let mut q = Surface::with_flags((5, 28), SRCALPHA);
            q = muster_auf(q, 5, 28);
            q.set_alpha(Some(222));
            s.blit_ex(&q, (0, 28), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
            rustgame_core::draw::circle(&mut s, (74u8, 141u8, 209u8, 202u8), (24, 11), 0, 1);
            rustgame_core::draw::rect_radius(&mut s, (167u8, 128u8, 237u8, 255u8), Rect::new(1, 19, 3, 2), 1, 6);
            rustgame_core::draw::rect_radius(&mut s, (148u8, 94u8, 202u8, 255u8), Rect::new(26, 16, 13, 15), 2, 2);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x93E26532ED9F25CA, flaeche: s }
    }));
    v.push((577, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((18, 17), 0);
            s.fill((175u8, 236u8, 212u8, 255u8));
            rustgame_core::draw::circle(&mut s, (124u8, 204u8, 98u8, 149u8), (-4, -3), 17, 1);
            let mut q = Surface::with_flags((20, 9), 0);
            q = muster_auf(q, 20, 9);
            s.blit_ex(&q, (2, 5), Some(Rect::new(12, 4, 21, 8)), rustgame_core::blend::flags::BLEND_RGB_MULT);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x9E13BAE260CD384B, flaeche: s }
    }));
    v.push((578, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((30, 15), SRCALPHA);
            s.fill((182u8, 187u8, 47u8, 203u8));
            s.set_clip(Some(Rect::new(19, 5, 10, 10)));
            let mut q = Surface::with_flags((32, 3), 0);
            q = muster_auf(q, 32, 3);
            q.set_alpha(Some(28));
            s.blit_ex(&q, (-3, 4), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            let mut q = Surface::with_flags((30, 4), 0);
            q = muster_auf(q, 30, 4);
            s.blit_ex(&q, (-3, 0), Some(Rect::new(24, 0, 17, 7)), rustgame_core::blend::flags::BLEND_RGBA_MIN);
            rustgame_core::draw::rect_radius(&mut s, (54u8, 56u8, 122u8, 236u8), Rect::new(-8, -7, 29, 10), 4, 0);
            let mut q = Surface::with_flags((12, 17), 0);
            q = muster_auf(q, 12, 17);
            q.set_alpha(Some(181));
            s.blit_ex(&q, (9, 18), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            let mut q = Surface::with_flags((18, 13), SRCALPHA);
            q = muster_auf(q, 18, 13);
            q.set_colorkey(Some((212u8, 95u8, 58u8)));
            s.blit_ex(&q, (29, 13), None, 0);
            rustgame_core::draw::circle(&mut s, (239u8, 164u8, 54u8, 90u8), (13, 4), 10, 0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xBE97FD038C683A6B, flaeche: s }
    }));
    v.push((579, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((16, 18), 0);
            s.fill((72u8, 170u8, 16u8, 255u8));
            s.fill_rect((40u8, 142u8, 236u8, 221u8), Rect::new(5, 15, 18, 21));
            rustgame_core::draw::ellipse(&mut s, (88u8, 138u8, 51u8, 185u8), Rect::new(9, 1, 10, 11), 3);
            let mut q = Surface::with_flags((12, 16), 0);
            q = muster_auf(q, 12, 16);
            s.blit_ex(&q, (1, 14), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            s.scroll(5, 18);
            s = rustgame_core::transform::flip(&s, false, true);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xB48F94991C77950A, flaeche: s }
    }));
    v.push((580, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((22, 22), SRCALPHA);
            s.fill((74u8, 222u8, 145u8, 150u8));
            rustgame_core::draw::polygon(&mut s, (137u8, 175u8, 212u8, 255u8), &[(18, 20), (13, 5), (1, 22), (12, 9)], 2);
            rustgame_core::draw::line(&mut s, (218u8, 58u8, 46u8, 255u8), (16, 1), (5, 22), 3);
            rustgame_core::draw::polygon(&mut s, (254u8, 162u8, 252u8, 255u8), &[(15, 18), (-1, 21), (4, 17), (-2, 25), (-1, 25), (8, 1)], 3);
            s.set_clip(Some(Rect::new(1, 15, 14, 5)));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x0842C7D3E3BADC33, flaeche: s }
    }));
    v.push((581, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((28, 6), SRCALPHA);
            s.fill((200u8, 2u8, 211u8, 255u8));
            let mut q = Surface::with_flags((19, 2), 0);
            q = muster_auf(q, 19, 2);
            q.set_alpha(Some(31));
            s.blit_ex(&q, (24, 0), None, 0);
            let mut q = Surface::with_flags((27, 3), 0);
            q = muster_auf(q, 27, 3);
            q.set_alpha(Some(13));
            s.blit_ex(&q, (23, 9), Some(Rect::new(28, -2, 28, 5)), rustgame_core::blend::flags::BLEND_RGB_MULT);
            let mut q = Surface::with_flags((9, 3), 0);
            q = muster_auf(q, 9, 3);
            q.set_alpha(Some(151));
            s.blit_ex(&q, (14, 4), Some(Rect::new(3, -1, 10, 3)), rustgame_core::blend::flags::BLEND_RGB_MULT);
            s.fill_rect((197u8, 129u8, 189u8, 255u8), Rect::new(28, 5, 0, 9));
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xB1920A892BC3969C, flaeche: s }
    }));
    v.push((582, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((12, 28), 0);
            s.fill((116u8, 233u8, 90u8, 255u8));
            s.fill_rect((101u8, 91u8, 71u8, 30u8), Rect::new(-6, 24, 7, 0));
            {
                let mut teil = s.subsurface(Rect::new(2, 9, 2, 5));
                teil.fill((79u8, 26u8, 205u8, 75u8));
            }
            rustgame_core::draw::rect_radius(&mut s, (155u8, 102u8, 114u8, 80u8), Rect::new(10, 14, 10, 20), 2, 4);
            rustgame_core::draw::line(&mut s, (96u8, 40u8, 214u8, 37u8), (5, -4), (13, 19), 3);
            rustgame_core::draw::polygon(&mut s, (148u8, 212u8, 228u8, 255u8), &[(4, 20), (11, 4), (6, 18), (10, -3), (-3, 12)], 2);
            s = s.convert_alpha();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xD825D6D548EDA4DA, flaeche: s }
    }));
    v.push((583, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((23, 9), SRCALPHA);
            s.fill((150u8, 99u8, 132u8, 255u8));
            let mut q = Surface::with_flags((18, 6), 0);
            q = muster_auf(q, 18, 6);
            s.blit_ex(&q, (-1, 0), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            rustgame_core::draw::ellipse(&mut s, (79u8, 203u8, 215u8, 255u8), Rect::new(-3, 5, 20, 0), 3);
            rustgame_core::draw::circle(&mut s, (125u8, 216u8, 217u8, 88u8), (7, -3), 17, 3);
            let mut q = Surface::with_flags((12, 11), 0);
            q = muster_auf(q, 12, 11);
            s.blit_ex(&q, (9, 11), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            rustgame_core::draw::ellipse(&mut s, (226u8, 30u8, 21u8, 255u8), Rect::new(14, -1, 13, 5), 1);
            s.fill_rect((74u8, 63u8, 54u8, 91u8), Rect::new(9, 10, 14, 5));
            s.set_clip(Some(Rect::new(7, 3, 12, 4)));
            {
                let mut teil = s.subsurface(Rect::new(1, 8, 11, 1));
                teil.fill((218u8, 193u8, 228u8, 255u8));
            }
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xC53DC8AF354B3432, flaeche: s }
    }));
    v.push((584, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((25, 28), SRCALPHA);
            s.fill((151u8, 60u8, 42u8, 255u8));
            rustgame_core::draw::ellipse(&mut s, (122u8, 136u8, 196u8, 224u8), Rect::new(24, 14, 20, 25), 4);
            rustgame_core::draw::rect_radius(&mut s, (219u8, 118u8, 169u8, 249u8), Rect::new(24, 19, 12, 3), 2, 4);
            s = rustgame_core::transform::flip(&s, true, false);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xA36F05648FF3C82C, flaeche: s }
    }));
    v.push((585, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((7, 8), 0);
            s.fill((96u8, 224u8, 182u8, 255u8));
            rustgame_core::draw::rect_radius(&mut s, (1u8, 134u8, 239u8, 111u8), Rect::new(0, 6, 3, 1), 1, 6);
            rustgame_core::draw::circle(&mut s, (43u8, 27u8, 239u8, 202u8), (4, 8), 6, 3);
            rustgame_core::draw::line(&mut s, (99u8, 236u8, 162u8, 204u8), (9, 10), (-3, 7), 2);
            rustgame_core::draw::ellipse(&mut s, (119u8, 233u8, 62u8, 195u8), Rect::new(4, 4, 5, 0), 1);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x4A02FB5A282E3B91, flaeche: s }
    }));
    v.push((586, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((29, 14), SRCALPHA);
            s.fill((36u8, 245u8, 104u8, 188u8));
            s.fill_rect((99u8, 239u8, 233u8, 58u8), Rect::new(-3, 12, 10, 4));
            let mut q = Surface::with_flags((23, 12), SRCALPHA);
            q = muster_auf(q, 23, 12);
            q.set_alpha(Some(148));
            s.blit_ex(&q, (7, 4), Some(Rect::new(-10, 1, 14, 6)), rustgame_core::blend::flags::BLEND_RGB_SUB);
            rustgame_core::draw::rect_radius(&mut s, (238u8, 85u8, 212u8, 255u8), Rect::new(27, 11, 15, 14), 2, 3);
            s.fill_rect((151u8, 167u8, 57u8, 255u8), Rect::new(-14, 3, 1, 3));
            s.set_clip(Some(Rect::new(5, 13, 17, 1)));
            let mut q = Surface::with_flags((17, 12), 0);
            q = muster_auf(q, 17, 12);
            s.blit_ex(&q, (22, 4), None, rustgame_core::blend::flags::BLEND_RGB_MAX);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x4834FC0F72A87CBE, flaeche: s }
    }));
    v.push((587, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((28, 31), 0);
            s.fill((166u8, 21u8, 178u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(22, 22, 4, 3));
                teil.fill((125u8, 213u8, 236u8, 98u8));
            }
            rustgame_core::draw::ellipse(&mut s, (37u8, 229u8, 228u8, 255u8), Rect::new(6, -14, 30, 14), 3);
            let mut q = Surface::with_flags((11, 32), SRCALPHA);
            q = muster_auf(q, 11, 32);
            q.set_alpha(Some(28));
            s.blit_ex(&q, (7, 6), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            rustgame_core::draw::polygon(&mut s, (152u8, 250u8, 200u8, 255u8), &[(9, 14), (0, 8), (15, 4), (23, 30)], 1);
            rustgame_core::draw::rect_radius(&mut s, (12u8, 173u8, 145u8, 14u8), Rect::new(27, 25, 15, 25), 4, 0);
            s = s.convert();
            s = rustgame_core::transform::grayscale(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x11F5DDB2BABCB8E5, flaeche: s }
    }));
    v.push((588, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((29, 32), SRCALPHA);
            s.fill((163u8, 60u8, 178u8, 255u8));
            rustgame_core::draw::rect(&mut s, (194u8, 196u8, 85u8, 255u8), Rect::new(-9, 18, 25, 1), 0);
            rustgame_core::draw::circle(&mut s, (36u8, 212u8, 97u8, 21u8), (14, 5), 6, 4);
            s.set_clip(Some(Rect::new(19, 16, 6, 8)));
            let mut q = Surface::with_flags((16, 28), 0);
            q = muster_auf(q, 16, 28);
            s.blit_ex(&q, (20, 18), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            let mut q = Surface::with_flags((24, 8), SRCALPHA);
            q = muster_auf(q, 24, 8);
            q.set_alpha(Some(110));
            q.set_colorkey(Some((8u8, 204u8, 243u8)));
            s.blit_ex(&q, (24, 18), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            let mut q = Surface::with_flags((5, 29), 0);
            q = muster_auf(q, 5, 29);
            s.blit_ex(&q, (0, -3), Some(Rect::new(-2, 23, 1, 14)), rustgame_core::blend::flags::BLEND_RGBA_SUB);
            let mut q = Surface::with_flags((28, 4), 0);
            q = muster_auf(q, 28, 4);
            s.blit_ex(&q, (14, 6), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x56E43F6188C6CBF1, flaeche: s }
    }));
    v.push((589, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((9, 16), 0);
            s.fill((148u8, 197u8, 117u8, 255u8));
            let mut q = Surface::with_flags((3, 4), 0);
            q = muster_auf(q, 3, 4);
            q.set_alpha(Some(110));
            s.blit_ex(&q, (11, 1), Some(Rect::new(4, -1, 3, 2)), rustgame_core::blend::flags::BLEND_RGB_MULT);
            let mut q = Surface::with_flags((10, 6), SRCALPHA);
            q = muster_auf(q, 10, 6);
            s.blit_ex(&q, (9, -4), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            rustgame_core::draw::rect_radius(&mut s, (63u8, 141u8, 86u8, 255u8), Rect::new(3, 4, 12, 15), 3, 4);
            s = rustgame_core::transform::flip(&s, false, false);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x204F37285984F325, flaeche: s }
    }));
    v.push((590, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((6, 7), 0);
            s.fill((204u8, 216u8, 177u8, 255u8));
            let mut q = Surface::with_flags((5, 7), 0);
            q = muster_auf(q, 5, 7);
            q.set_colorkey(Some((22u8, 124u8, 102u8)));
            s.blit_ex(&q, (7, -3), Some(Rect::new(-2, 4, 6, 9)), rustgame_core::blend::flags::BLEND_RGB_MIN);
            s.fill_rect((24u8, 0u8, 127u8, 255u8), Rect::new(5, 3, 0, 3));
            rustgame_core::draw::ellipse(&mut s, (151u8, 142u8, 231u8, 59u8), Rect::new(2, 5, 0, 4), 3);
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x29FFB6FDC2939335, flaeche: s }
    }));
    v.push((591, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((23, 18), 0);
            s.fill((226u8, 65u8, 40u8, 255u8));
            s.scroll(-11, -15);
            rustgame_core::draw::rect_radius(&mut s, (59u8, 0u8, 81u8, 174u8), Rect::new(21, -8, 3, 11), 3, 1);
            rustgame_core::draw::circle(&mut s, (134u8, 94u8, 77u8, 26u8), (24, -2), 15, 0);
            s = rustgame_core::transform::grayscale(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xBE346D215EEBA10E, flaeche: s }
    }));
    v.push((592, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((8, 20), 0);
            s.fill((82u8, 33u8, 75u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(5, 7, 1, 6));
                teil.fill((146u8, 102u8, 158u8, 255u8));
            }
            let mut q = Surface::with_flags((3, 13), 0);
            q = muster_auf(q, 3, 13);
            q.set_colorkey(Some((68u8, 230u8, 103u8)));
            s.blit_ex(&q, (9, 19), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            rustgame_core::draw::circle(&mut s, (91u8, 62u8, 199u8, 255u8), (3, 4), 17, 1);
            rustgame_core::draw::circle(&mut s, (17u8, 42u8, 81u8, 0u8), (-3, 21), 5, 1);
            {
                let mut teil = s.subsurface(Rect::new(6, 4, 1, 7));
                teil.fill((34u8, 187u8, 159u8, 28u8));
            }
            s.set_clip(Some(Rect::new(3, 10, 3, 4)));
            rustgame_core::draw::rect_radius(&mut s, (87u8, 9u8, 223u8, 255u8), Rect::new(-2, 6, 9, 8), 3, 1);
            s = rustgame_core::transform::flip(&s, true, true);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xE4F4B0B29C63B746, flaeche: s }
    }));
    v.push((593, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((16, 17), SRCALPHA);
            s.fill((197u8, 213u8, 2u8, 249u8));
            s.set_clip(Some(Rect::new(12, 8, 1, 4)));
            rustgame_core::draw::ellipse(&mut s, (51u8, 35u8, 6u8, 255u8), Rect::new(0, 7, 5, 11), 3);
            s.fill_rect((38u8, 250u8, 35u8, 255u8), Rect::new(-8, 4, 6, 1));
            s = s.convert();
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x8615320B7EA98CC5, flaeche: s }
    }));
    v.push((594, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((10, 21), 0);
            s.fill((98u8, 151u8, 26u8, 255u8));
            rustgame_core::draw::circle(&mut s, (3u8, 10u8, 47u8, 106u8), (12, 1), 13, 0);
            s.fill_rect((245u8, 216u8, 213u8, 11u8), Rect::new(0, 3, 7, 17));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xD3EE7DAF985EA9B7, flaeche: s }
    }));
    v.push((595, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((17, 17), 0);
            s.fill((212u8, 4u8, 114u8, 255u8));
            let mut q = Surface::with_flags((15, 12), SRCALPHA);
            q = muster_auf(q, 15, 12);
            s.blit_ex(&q, (13, 20), None, rustgame_core::blend::flags::BLEND_RGB_SUB);
            s.fill_rect((247u8, 95u8, 198u8, 255u8), Rect::new(-6, -3, 7, 10));
            s.set_clip(Some(Rect::new(6, 8, 11, 8)));
            let mut q = Surface::with_flags((10, 8), 0);
            q = muster_auf(q, 10, 8);
            s.blit_ex(&q, (6, 0), None, rustgame_core::blend::flags::BLEND_RGB_MIN);
            rustgame_core::draw::circle(&mut s, (142u8, 169u8, 208u8, 234u8), (8, -4), 4, 0);
            s.scroll(-17, -5);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x78B9AD57E50F6F14, flaeche: s }
    }));
    v.push((596, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((18, 10), 0);
            s.fill((73u8, 88u8, 110u8, 255u8));
            {
                let mut teil = s.subsurface(Rect::new(7, 0, 1, 9));
                teil.fill((165u8, 224u8, 8u8, 156u8));
            }
            rustgame_core::draw::polygon(&mut s, (43u8, 232u8, 225u8, 255u8), &[(16, -2), (3, -4), (4, 12), (7, 9), (-4, 6)], 3);
            s.fill_rect((176u8, 181u8, 194u8, 66u8), Rect::new(15, -3, 4, 7));
            let mut q = Surface::with_flags((15, 6), SRCALPHA);
            q = muster_auf(q, 15, 6);
            q.set_alpha(Some(82));
            s.blit_ex(&q, (11, 13), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
            rustgame_core::draw::ellipse(&mut s, (50u8, 99u8, 96u8, 181u8), Rect::new(6, 10, 14, 7), 2);
            let mut q = Surface::with_flags((18, 9), SRCALPHA);
            q = muster_auf(q, 18, 9);
            q.set_alpha(Some(211));
            q.set_colorkey(Some((176u8, 25u8, 35u8)));
            s.blit_ex(&q, (0, 7), None, rustgame_core::blend::flags::BLEND_RGB_MULT);
            rustgame_core::draw::rect(&mut s, (214u8, 163u8, 86u8, 121u8), Rect::new(16, 1, 5, 13), 4);
            s = rustgame_core::transform::rotate(&s, -14.0);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xF0236E16FFB736EF, flaeche: s }
    }));
    v.push((597, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((9, 10), SRCALPHA);
            s.fill((201u8, 136u8, 138u8, 51u8));
            let mut q = Surface::with_flags((8, 9), 0);
            q = muster_auf(q, 8, 9);
            q.set_alpha(Some(249));
            s.blit_ex(&q, (6, 4), None, rustgame_core::blend::flags::BLEND_RGBA_MAX);
            rustgame_core::draw::circle(&mut s, (10u8, 84u8, 135u8, 255u8), (10, 13), 6, 0);
            rustgame_core::draw::rect(&mut s, (229u8, 52u8, 62u8, 77u8), Rect::new(7, -5, 5, 10), 2);
            let mut q = Surface::with_flags((2, 9), SRCALPHA);
            q = muster_auf(q, 2, 9);
            s.blit_ex(&q, (8, 5), None, rustgame_core::blend::flags::BLEND_RGBA_SUB);
            rustgame_core::draw::line(&mut s, (16u8, 197u8, 149u8, 185u8), (3, 0), (-1, 5), 3);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x55C6C6C8F03778D3, flaeche: s }
    }));
    v.push((598, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((32, 9), 0);
            s.fill((231u8, 49u8, 238u8, 255u8));
            let mut q = Surface::with_flags((22, 2), 0);
            q = muster_auf(q, 22, 2);
            s.blit_ex(&q, (19, 12), None, rustgame_core::blend::flags::BLEND_RGBA_MIN);
            rustgame_core::draw::ellipse(&mut s, (243u8, 94u8, 240u8, 39u8), Rect::new(31, 5, 8, 10), 0);
            {
                let mut teil = s.subsurface(Rect::new(25, 1, 1, 7));
                teil.fill((131u8, 50u8, 155u8, 130u8));
            }
            rustgame_core::draw::polygon(&mut s, (45u8, 224u8, 96u8, 29u8), &[(28, -3), (18, 1), (35, -2), (26, 0)], 2);
            rustgame_core::draw::circle(&mut s, (60u8, 163u8, 121u8, 103u8), (1, -3), 22, 4);
            rustgame_core::draw::ellipse(&mut s, (212u8, 204u8, 65u8, 74u8), Rect::new(16, 2, 27, 8), 2);
            rustgame_core::draw::rect_radius(&mut s, (92u8, 156u8, 44u8, 116u8), Rect::new(8, 5, 18, 10), 0, 1);
            rustgame_core::draw::line(&mut s, (170u8, 155u8, 131u8, 193u8), (6, 0), (-4, -3), 4);
            s = rustgame_core::transform::scale(&s, (29, 13));
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0x476A2950A1E9214F, flaeche: s }
    }));
    v.push((599, false, || {
        #[allow(unused_mut)]
        let mut s = Surface::with_flags((28, 7), SRCALPHA);
            s.fill((155u8, 66u8, 133u8, 30u8));
            s.fill_rect((155u8, 41u8, 164u8, 255u8), Rect::new(26, -3, 0, 2));
            let mut q = Surface::with_flags((7, 2), 0);
            q = muster_auf(q, 7, 2);
            q.set_alpha(Some(136));
            s.blit_ex(&q, (18, 10), None, rustgame_core::blend::flags::BLEND_RGB_ADD);
            s = rustgame_core::transform::scale2x(&s);
        FuzzErgebnis { hash: fnv1a(&s.to_rgba_bytes()), erwartet: 0xA8618C122DF31DA5, flaeche: s }
    }));
    v
}
