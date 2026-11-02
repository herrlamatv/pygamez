# -*- coding: utf-8 -*-
"""
Derselbe Satz Proben, einmal auf echtem pygame-ce und einmal auf der
rustgame-Huelle - und dann ein Vergleich Zeile fuer Zeile.

Gebaut wie rustgame/tools/vergleich_py.py, nur fuer die Module, die dort
nicht vorkommen: image, time, event, key, mouse, display, font, Color,
Rect, Surface-Formatangaben.

Jede Probe gibt einen Wert zurueck, der sich vergleichen laesst. Wirft
sie, wird die Ausnahme selbst zum Wert - dann faellt auch auf, wenn die
eine Seite meckert und die andere nicht.
"""

import os
import re
import sys

HIER = os.path.dirname(os.path.abspath(__file__))
# tools/ -> rustgame-py -> crates -> rustgame -> pygamez
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(HIER))))
HUELLE = os.path.dirname(HIER)

os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
os.environ.setdefault("RUSTGAME_VIDEODRIVER", "dummy")
# **Vor** dem Import setzen. Ohne diese beiden oeffnet der Mischer das
# echte Tongeraet - daran ist bei dieser Kiste schon einmal jemand
# gestolpert, und ein Vergleichslauf, der Geraeusche macht, ist kein
# Vergleichslauf.
os.environ.setdefault("SDL_AUDIODRIVER", "dummy")
os.environ.setdefault("RUSTGAME_AUDIODRIVER", "dummy")
os.environ.setdefault("PYGAME_HIDE_SUPPORT_PROMPT", "1")


# ---------------------------------------------------------------------------
#  Die Proben
# ---------------------------------------------------------------------------

def proben(pg):
    """Liefert [(name, wert), ...] - beide Engines bekommen dasselbe."""
    p = []

    def probe(name, f):
        try:
            p.append((name, f()))
        except Exception as e:                      # noqa: BLE001
            p.append((name, "%s: %s" % (type(e).__name__, e)))

    pg.init()
    scr = pg.display.set_mode((64, 48))

    # ----- Color ----------------------------------------------------------
    farben = [(100, 150, 200, 250), (0, 0, 0, 0), (255, 255, 255, 255),
              (1, 2, 3, 4), (37, 211, 80, 128), (200, 10, 10, 1)]
    for t in farben:
        c = pg.Color(*t)
        probe("Color%s.hsva" % (t,), lambda c=c: tuple(round(v, 9) for v in c.hsva))
        probe("Color%s.hsla" % (t,), lambda c=c: tuple(round(v, 9) for v in c.hsla))
        probe("Color%s.cmy" % (t,), lambda c=c: tuple(round(v, 9) for v in c.cmy))
        probe("Color%s.i1i2i3" % (t,), lambda c=c: tuple(round(v, 9) for v in c.i1i2i3))
        probe("Color%s.normalize" % (t,), lambda c=c: tuple(round(v, 9) for v in c.normalize()))
        probe("Color%s.premul_alpha" % (t,), lambda c=c: tuple(c.premul_alpha()))
        probe("Color%s.grayscale" % (t,), lambda c=c: tuple(c.grayscale()))
        probe("Color%s.hex" % (t,), lambda c=c: c.hex)
        probe("Color%s.int" % (t,), lambda c=c: int(c))
        probe("Color%s.repr" % (t,), lambda c=c: repr(c))
        probe("Color%s.len" % (t,), lambda c=c: len(c))
        probe("Color%s.liste" % (t,), lambda c=c: list(c))
        probe("Color%s.slice" % (t,), lambda c=c: c[1:3])
        for g in (0.5, 1.0, 2.2, 3.0):
            probe("Color%s.correct_gamma(%s)" % (t, g),
                  lambda c=c, g=g: tuple(c.correct_gamma(g)))
        for tt in (0.0, 0.3, 0.5, 0.77, 1.0):
            probe("Color%s.lerp(black,%s)" % (t, tt),
                  lambda c=c, tt=tt: tuple(c.lerp(pg.Color(0, 0, 0, 0), tt)))
        # Rundlauf ueber die Farbraeume - dort schneidet pygame ab.
        probe("Color%s.from_hsva" % (t,),
              lambda c=c: tuple(pg.Color.from_hsva(*c.hsva)))
        probe("Color%s.from_hsla" % (t,),
              lambda c=c: tuple(pg.Color.from_hsla(*c.hsla)))
        probe("Color%s.from_hex" % (t,),
              lambda c=c: tuple(pg.Color.from_hex(c.hex)))
        probe("Color%s.setze_hsva" % (t,), lambda c=c: setz(pg, c.hsva, "hsva"))
        probe("Color%s.setze_hsla" % (t,), lambda c=c: setz(pg, c.hsla, "hsla"))
        probe("Color%s.setze_cmy" % (t,), lambda c=c: setz(pg, c.cmy, "cmy"))
        probe("Color%s.setze_i1i2i3" % (t,), lambda c=c: setz(pg, c.i1i2i3, "i1i2i3"))

    probe("Color('red')", lambda: tuple(pg.Color("red")))
    probe("Color('#1a2b3c')", lambda: tuple(pg.Color("#1a2b3c")))
    probe("Color(0x11223344)", lambda: tuple(pg.Color(0x11223344)))
    probe("Color(1,2)", lambda: tuple(pg.Color(1, 2)))
    probe("Color(300,0,0)", lambda: tuple(pg.Color(300, 0, 0)))
    probe("Color+Color", lambda: tuple(pg.Color(10, 20, 30, 40) + pg.Color(250, 250, 250, 250)))
    probe("Color-Color", lambda: tuple(pg.Color(10, 20, 30, 40) - pg.Color(5, 50, 5, 50)))
    probe("Color*Color", lambda: tuple(pg.Color(10, 20, 30, 40) * pg.Color(2, 2, 2, 2)))
    probe("Color==tuple", lambda: pg.Color(1, 2, 3, 4) == (1, 2, 3, 4))
    probe("Color.update", lambda: upd(pg))
    probe("hash(Color)", lambda: hash(pg.Color(1, 2, 3, 4)))

    # ----- Rect -----------------------------------------------------------
    rechtecke = [(1, 2, 30, 40), (0, 0, 0, 0), (-5, -5, 10, 10), (7, 7, 1, 1),
                 (3, 4, -6, -8)]
    for t in rechtecke:
        r = pg.Rect(*t)
        probe("Rect%s.repr" % (t,), lambda r=r: repr(r))
        probe("Rect%s.anker" % (t,), lambda r=r: (
            r.left, r.top, r.right, r.bottom, r.centerx, r.centery,
            r.topleft, r.topright, r.bottomleft, r.bottomright, r.center,
            r.midtop, r.midbottom, r.midleft, r.midright, r.size))
        probe("Rect%s.bool" % (t,), lambda r=r: bool(r))
        probe("Rect%s.liste" % (t,), lambda r=r: list(r))
        probe("Rect%s.slice" % (t,), lambda r=r: r[1:3])
        probe("Rect%s.scale_by2" % (t,), lambda r=r: tuple(r.scale_by(2.0)))
        probe("Rect%s.scale_by(2,0.5)" % (t,), lambda r=r: tuple(r.scale_by(2.0, 0.5)))
        probe("Rect%s.unionall" % (t,),
              lambda r=r: tuple(r.unionall([pg.Rect(-5, -5, 2, 2), pg.Rect(50, 50, 3, 3)])))
        probe("Rect%s.collidelist" % (t,),
              lambda r=r: r.collidelist([pg.Rect(100, 100, 1, 1), pg.Rect(0, 0, 5, 5)]))
        probe("Rect%s.collidelistall" % (t,),
              lambda r=r: r.collidelistall([pg.Rect(0, 0, 5, 5), pg.Rect(2, 3, 1, 1),
                                            pg.Rect(99, 99, 1, 1)]))
        for strecke in ((0, 0, 100, 100), (100, 100, 200, 200), (-10, 5, 100, 5),
                        (2, 3, 2, 3), (0, 0, 0, 0)):
            probe("Rect%s.clipline%s" % (t, (strecke,)),
                  lambda r=r, s=strecke: r.clipline(s))
        probe("Rect%s.move_ip" % (t,), lambda r=r: ip(r, "move_ip", 3, -4))
        probe("Rect%s.union_ip" % (t,), lambda r=r: ip(r, "union_ip", pg.Rect(0, 0, 1, 1)))
        probe("Rect%s.clamp_ip" % (t,), lambda r=r: ip(r, "clamp_ip", pg.Rect(0, 0, 10, 10)))
        probe("Rect%s.inflate_ip" % (t,), lambda r=r: ip(r, "inflate_ip", 4, 6))
        probe("Rect%s.normalize" % (t,), lambda r=r: ip(r, "normalize"))
        if t[2] > 0 and t[3] > 0:
            probe("Rect%s.fit" % (t,), lambda r=r: tuple(r.fit(pg.Rect(0, 0, 20, 20))))
        probe("Rect%s.clip" % (t,), lambda r=r: tuple(r.clip(pg.Rect(0, 0, 20, 20))))
        probe("Rect%s.contains" % (t,), lambda r=r: r.contains(pg.Rect(2, 3, 1, 1)))
        probe("Rect%s.in" % (t,), lambda r=r: 1 in r)
    probe("hash(Rect)", lambda: hash(pg.Rect(0, 0, 1, 1)))
    probe("Rect kwargs unbekannt", lambda: pg.Surface((4, 4)).get_rect(blah=1))

    # ----- Surface-Formatangaben -----------------------------------------
    s_ohne = pg.Surface((7, 5))
    s_mit = pg.Surface((7, 5), pg.SRCALPHA)
    for name, s in (("ohne", s_ohne), ("mit", s_mit), ("screen", scr)):
        probe("Surface[%s].format" % name, lambda s=s: (
            s.get_bitsize(), s.get_bytesize(), tuple(s.get_masks()),
            tuple(s.get_shifts()), tuple(s.get_losses()), s.get_flags(),
            s.get_alpha(), s.mustlock(), s.get_locked()))
        probe("Surface[%s].repr" % name, lambda s=s: repr(s))
        probe("Surface[%s].map_rgb" % name, lambda s=s: (
            s.map_rgb((10, 20, 30)), s.map_rgb((10, 20, 30, 40)),
            tuple(s.unmap_rgb(s.map_rgb((10, 20, 30, 40))))))
        probe("Surface[%s].get_at_mapped" % name, lambda s=s: s.get_at_mapped((0, 0)))

    probe("Surface.bounding leer", lambda: tuple(pg.Surface((4, 4), pg.SRCALPHA).get_bounding_rect()))
    probe("Surface.bounding voll", lambda: tuple(pg.Surface((4, 4)).get_bounding_rect()))
    probe("Surface.bounding punkt", lambda: bounding(pg))
    probe("Surface.premul_alpha", lambda: premul(pg))
    probe("Surface.subsurface", lambda: subs(pg))
    probe("Surface.blits", lambda: blits(pg))
    probe("Surface.scroll", lambda: scroll(pg))
    probe("Surface.fill blend", lambda: fill_blend(pg))
    probe("Surface.blit blend", lambda: blit_blend(pg))

    # ----- image ----------------------------------------------------------
    quelle = pg.Surface((5, 3), pg.SRCALPHA)
    for y in range(3):
        for x in range(5):
            quelle.set_at((x, y), (x * 40, y * 80, 255 - x * 30, 255 - x * 20))
    for f in ("RGB", "RGBA", "RGBX", "ARGB", "BGRA", "ABGR",
              "RGBA_PREMULT", "ARGB_PREMULT"):
        probe("image.tobytes[%s]" % f, lambda f=f: pg.image.tobytes(quelle, f).hex())
        probe("image.tobytes[%s] flipped" % f,
              lambda f=f: pg.image.tobytes(quelle, f, True).hex())
    for f in ("RGB", "RGBA", "RGBX", "ARGB", "BGRA", "ABGR"):
        probe("image.rundlauf[%s]" % f, lambda f=f: rundlauf(pg, quelle, f))
    probe("image.tobytes pitch", lambda: pg.image.tobytes(quelle, "RGB", False, 20).hex())
    probe("image.frombuffer", lambda: tuple(
        pg.image.frombuffer(pg.image.tobytes(quelle, "RGBA"), (5, 3), "RGBA").get_at((2, 1))))
    probe("image.get_extended", lambda: pg.image.get_extended())

    # ----- transform: die dreizehn spaet durchgereichten ------------------
    probe("transform.neue", lambda: tf_neue(pg))
    probe("transform.dest_surface", lambda: tf_ziel(pg))
    probe("transform.dest_meldungen", lambda: tf_meldungen(pg))
    # **Vor** `tf_backend` - das setzt den Wert um, und danach waere
    # der Unterschied verschwunden.
    probe("transform.get_backend",
          lambda: pg.transform.get_smoothscale_backend())
    probe("transform.backend", lambda: tf_backend(pg))
    probe("transform.smoothscale Wege", lambda: tf_smoothscale_wege(pg))

    # ----- draw: die vier kantengeglaetteten ------------------------------
    probe("draw.aa und flood_fill", lambda: aa_und_fuellen(pg))
    probe("draw.arc Winkelgrenzen", lambda: arc_winkelgrenzen(pg))
    probe("font.render Zeichenbilder", lambda: text_zeichenbilder(pg))
    probe("font.render Kursiv", lambda: text_kursiv(pg))
    probe("font Stilflaggen", lambda: text_stilflaggen(pg))
    probe("font.render einfarbig", lambda: text_einfarbig(pg))
    probe("font.outline", lambda: text_kontur(pg))
    probe("Surface.get_colorkey Form", lambda: schluessel_form(pg))
    probe("Klangdateien kaputt", lambda: klang_kaputt(pg))
    probe("Klangdateien AIFF und VOC", lambda: klang_aiff_voc(pg))
    probe("Trackermodule und Unbekanntes", lambda: klang_module(pg))
    probe("Musik-Dekodierer kaputt", lambda: musik_kaputt(pg))
    probe("Bilddateien kaputt", lambda: bild_kaputt(pg))
    probe("Bilddateien kaputt, Erkennung", lambda: bild_kaputt_grenze(pg))
    probe("LBM ILBM und PBM", lambda: bild_lbm(pg))
    probe("GIF wie SDL_image", lambda: bild_gif(pg))
    probe("ICO und CUR", lambda: bild_ico(pg))
    probe("XV und RGB332", lambda: bild_xv_rgb332(pg))

    # ----- display: die neunzehn spaet durchgereichten ----------------
    probe("display.auskuenfte", lambda: anz_auskuenfte(pg))
    probe("display.mode_ok", lambda: anz_mode_ok(pg))
    probe("display.gl_set_attribute",
          lambda: anz_versuch(lambda: pg.display.gl_set_attribute(
              pg.GL_DEPTH_SIZE, 24)))
    probe("display.gl_get_attribute",
          lambda: anz_versuch(lambda: pg.display.gl_get_attribute(
              pg.GL_DEPTH_SIZE)))
    probe("display.meldungen", lambda: anz_meldungen(pg))
    probe("display.gamma", lambda: anz_gamma(pg))
    probe("display.FULLSCREEN", lambda: anz_vollbild(pg))

    # ----- cursors ---------------------------------------------------------
    probe("cursors.Cursor", lambda: zeigerformen(pg))
    probe("cursors.daten", lambda: zeigerdaten(pg))
    probe("cursors.compile", lambda: zeiger_uebersetzen(pg))

    # ----- version, joystick, scrap -----------------------------------
    probe("version", lambda: fassungen(pg))
    probe("joystick", lambda: hebel(pg))
    probe("scrap", lambda: ablage(pg))

    # ----- mixer ------------------------------------------------------
    probe("mixer.anlauf", lambda: ton_anlauf(pg))
    probe("mixer.Sound", lambda: ton_klang(pg))
    probe("mixer.Channel", lambda: ton_spur(pg))
    probe("mixer.music", lambda: ton_musik(pg))
    probe("mixer.fehler", lambda: ton_fehler(pg))
    probe("mixer.version", lambda: pg.mixer.get_sdl_mixer_version())
    probe("mixer.endevent", lambda: ton_endereignis(pg))

    # ----- sprite -----------------------------------------------------
    probe("sprite.Sprite", lambda: figuren(pg))
    probe("sprite.Gruppen", lambda: gruppen(pg))
    probe("sprite.Zeichnen", lambda: gruppen_zeichnen(pg))
    probe("sprite.Stoesse", lambda: stoesse(pg))
    probe("sprite.Vererbung", lambda: vererbung(pg))

    # ----- PixelArray ---------------------------------------------------
    probe("PixelArray.form", lambda: feld_form(pg))
    probe("PixelArray.zugriff", lambda: feld_zugriff(pg))
    probe("PixelArray.masken", lambda: feld_masken(pg))

    # ----- Rechtecke aus Objekten mit `rect` ------------------------
    probe("Rect aus .rect", lambda: rect_aus_ding(pg))
    probe("Rect mit Komma", lambda: rect_mit_komma(pg))

    # ----- was die fremde Testsuite gefunden hat ----------------------
    # Jede dieser Proben deckt **einen** Fehler ab, den pygames eigene
    # Testdateien gefunden haben und den keine hiesige Pruefung sah.
    # Sie stehen hier, damit er nicht zurueckkommt.
    probe("Rect.clamp ungerade", lambda: rect_klemmen(pg))
    probe("Rect.fit entartet", lambda: rect_einpassen(pg))
    probe("Rect.collidedict", lambda: rect_woerterbuch(pg))
    probe("Rect Scheiben", lambda: rect_scheiben(pg))
    probe("Rect Unterklasse", lambda: rect_unterklasse(pg))
    probe("Color Rechenzeichen", lambda: farbe_rechnen(pg))
    probe("Color Farbnamen", lambda: farbe_namen(pg))
    probe("Color Setzer-Alpha", lambda: farbe_setzer(pg))
    probe("math freie Funktionen", lambda: math_frei(pg))
    probe("Vector Gleichheit mit epsilon", lambda: vektor_gleich(pg))
    probe("Unterklassen behalten ihren Typ", lambda: typen_bleiben(pg))
    probe("__init__ setzt neu", lambda: init_setzt_neu(pg))

    # ----- Pixelformate: der Kern kennt seit kurzem mehr als eines ----
    # Bis vor kurzem gab `get_bitsize()` hart 32 zurueck. Diese Proben
    # sind die Sicherung dagegen, dass das zurueckfaellt - und die
    # einzige Stelle, an der die abgeleiteten Werte (shifts, losses,
    # pitch) ueberhaupt gegen pygame stehen.
    probe("Format je Tiefe", lambda: format_je_tiefe(pg))
    probe("Format mit Alphakanal", lambda: format_mit_alpha(pg))
    probe("Format mit eigenen Masken", lambda: format_eigene_masken(pg))
    probe("Farbtafel mit 8 Bit", lambda: tafel_acht_bit(pg))
    probe("map_rgb bei 16 Bit", lambda: abbilden_sechzehn(pg))
    probe("average_color liest anders als get_at",
          lambda: mittelfarbe_sechzehn(pg))

    # ----- Fehlerarten: der eigentliche Punkt ist die Sorte -----------
    # pygame unterscheidet `TypeError`, `ValueError` und `pygame.error`
    # sehr genau, und eine Huelle, die alles zu `ValueError` macht,
    # sieht in jeder Pixelpruefung richtig aus.
    probe("Fehlerarten bei blits und fblits", lambda: fehlerarten_blit(pg))
    probe("Fehlerarten in transform", lambda: fehlerarten_transform(pg))
    probe("unbekanntes Schluesselwort", lambda: unbekanntes_wort(pg))
    probe("Fehlerarten bei Farben und Punkten", lambda: fehlerarten_farbe(pg))

    # ----- was heute in `mask` und `mouse` gefunden wurde -------------
    probe("mask area wird normalisiert", lambda: maske_ausschnitt(pg))
    probe("mask copy ruft __copy__", lambda: maske_kopie(pg))
    probe("mask to_surface prueft das Format", lambda: maske_format(pg))
    probe("from_surface mit negativer Schwelle",
          lambda: maske_schwelle(pg))
    probe("mouse get_pressed und get_pos", lambda: maus_stellen(pg))
    probe("bufferproxy", lambda: pufferzeiger(pg))
    probe("Pufferprotokoll", lambda: pufferprotokoll(pg))
    probe("Puffer spaet lesen", lambda: puffer_spaet(pg))
    probe("PixelArray sperrt die Flaeche", lambda: pixelfeld_sperre(pg))

    # ----- Namen: `__module__` verraet eine Huelle ---------------------
    probe("Namen der Typen und Module", lambda: namen(pg))
    probe("draw Viertelkreise und Farbzahlen", lambda: zeichnen_ecken(pg))
    probe("versionsnummern", lambda: (pg.image.get_sdl_image_version(),
                                      pg.font.get_sdl_ttf_version()))
    probe("image.png rundlauf", lambda: png_rundlauf(pg, quelle))
    probe("image.tobytes falsches format", lambda: pg.image.tobytes(quelle, "XYZ"))
    probe("image.frombytes zu kurz",
          lambda: pg.image.frombytes(b"\x00" * 3, (5, 3), "RGB"))

    # ----- time -----------------------------------------------------------
    probe("time.Clock tick(60)", lambda: taktung(pg, 60))
    probe("time.Clock tick(30)", lambda: taktung(pg, 30))
    probe("time.Clock tick(17)", lambda: taktung(pg, 17))
    probe("time.Clock frisch", lambda: frische_uhr(pg))
    probe("time.Clock repr", lambda: repr(pg.time.Clock()))
    probe("time.get_ticks typ", lambda: type(pg.time.get_ticks()).__name__)

    # ----- event ----------------------------------------------------------
    probe("event.poll leer", lambda: leer_poll(pg))
    probe("event.peek leer", lambda: (pg.event.clear(), pg.event.peek())[1])
    probe("event.get leer", lambda: (pg.event.clear(), pg.event.get())[1])
    probe("event.post nutzlast", lambda: nutzlast(pg))
    probe("event.post gesperrt", lambda: gesperrt(pg))
    probe("event.event_name", lambda: [pg.event.event_name(t) for t in
                                       (0, 256, 768, 1024, 32865, 32868, 60000)])
    probe("event.custom_type", lambda: pg.event.custom_type())
    probe("event.Event repr", lambda: repr(pg.event.Event(pg.QUIT)))
    probe("event.Event bool", lambda: (bool(pg.event.Event(pg.QUIT)),
                                       bool(pg.event.Event(pg.NOEVENT))))
    probe("event.Event gleich", lambda: pg.event.Event(pg.QUIT) == pg.event.Event(pg.QUIT))
    probe("event.Event fehlendes feld", lambda: pg.event.Event(pg.QUIT).gibtsnicht)
    probe("event.wait timeout", lambda: wait_leer(pg))
    probe("event.keydown rundlauf", lambda: keydown(pg))
    probe("Ereignisschlange unter Last", lambda: schlange_unter_last(pg))
    probe("Ereignistypen als Argument", lambda: schlange_typen(pg))
    probe("Typliste halb angewandt", lambda: schlange_halb(pg))

    # ----- mouse ----------------------------------------------------------
    probe("mouse.set_visible folge", lambda: (pg.mouse.set_visible(False),
                                              pg.mouse.set_visible(True),
                                              pg.mouse.set_visible(True),
                                              pg.mouse.get_visible()))
    probe("mouse.get_pressed", lambda: (pg.mouse.get_pressed(),
                                        pg.mouse.get_pressed(5),
                                        len(pg.mouse.get_pressed(5))))
    probe("mouse.set_pos", lambda: (pg.mouse.set_pos((3, 4)), pg.mouse.get_pos()))
    probe("mouse.get_rel zweimal", lambda: (pg.mouse.get_rel(), pg.mouse.get_rel()))

    # ----- key ------------------------------------------------------------
    probe("key.get_pressed laenge", lambda: len(pg.key.get_pressed()))
    probe("key.get_pressed zugriff", lambda: [
        pg.key.get_pressed()[i] for i in
        (0, 32, 44, 97, 511, 512, -1, pg.K_UP, pg.K_LSHIFT, pg.K_F13)])
    probe("key.get_pressed iter", lambda: list(pg.key.get_pressed()))
    probe("key.name", lambda: [pg.key.name(k) for k in
                               (pg.K_a, pg.K_SPACE, pg.K_LSHIFT, pg.K_UP,
                                pg.K_F1, pg.K_RETURN, pg.K_KP0)])
    probe("key.name nicht compat", lambda: [pg.key.name(k, use_compat=False) for k in
                                            (pg.K_a, pg.K_SPACE, pg.K_LSHIFT, pg.K_UP)])
    probe("key.key_code", lambda: [pg.key.key_code(n) for n in
                                   ("a", "space", "up", "Up", "left shift", "f1")])
    probe("key.key_code unbekannt", lambda: pg.key.key_code("gibtsnicht"))
    probe("key.mods", lambda: (pg.key.get_mods(),
                               pg.key.set_mods(pg.KMOD_SHIFT),
                               pg.key.get_mods(),
                               pg.key.set_mods(0),
                               pg.key.get_mods()))
    probe("key.repeat", lambda: (pg.key.get_repeat(),
                                 pg.key.set_repeat(300, 50),
                                 pg.key.get_repeat()))

    # ----- display --------------------------------------------------------
    probe("display.Info", lambda: info_werte(pg))
    probe("display.Info repr", lambda: repr(pg.display.Info()))
    probe("display.caption", lambda: (pg.display.set_caption("A", "B"),
                                      pg.display.get_caption()))
    probe("display.driver", lambda: pg.display.get_driver())
    probe("display.window_size", lambda: pg.display.get_window_size())
    probe("display.list_modes", lambda: pg.display.list_modes())
    probe("display.gleiches objekt", lambda: (
        pg.display.set_mode((64, 48)) is scr,
        pg.display.get_surface() is scr,
        pg.display.set_mode((80, 60)) is scr))
    probe("display.screen flags", lambda: scr.get_flags())
    probe("display.get_init", lambda: pg.display.get_init())
    probe("display.toggle_fullscreen", lambda: pg.display.toggle_fullscreen())
    probe("display.flip", lambda: pg.display.flip())
    probe("display.update rect", lambda: pg.display.update(pg.Rect(0, 0, 5, 5)))
    probe("display.update liste", lambda: pg.display.update([pg.Rect(0, 0, 5, 5), None]))

    # ----- Konstanten -----------------------------------------------------
    probe("konstanten", lambda: [(n, getattr(pg, n)) for n in (
        "QUIT", "KEYDOWN", "KEYUP", "MOUSEBUTTONDOWN", "MOUSEMOTION",
        "USEREVENT", "NOEVENT", "SRCALPHA", "FULLSCREEN", "RESIZABLE",
        "NOFRAME", "DOUBLEBUF", "HWSURFACE", "SCALED", "PREALLOC",
        "K_a", "K_z", "K_0", "K_SPACE", "K_ESCAPE", "K_RETURN", "K_UP",
        "K_DOWN", "K_LEFT", "K_RIGHT", "K_LSHIFT", "K_RCTRL", "K_F1",
        "K_F12", "K_KP_ENTER", "K_BACKSPACE", "K_TAB", "K_DELETE",
        "KMOD_SHIFT", "KMOD_CTRL", "KMOD_ALT", "KMOD_NONE",
        "KSCAN_A", "KSCAN_SPACE", "KSCAN_UP",
        "BLEND_ADD", "BLEND_SUB", "BLEND_MULT", "BLEND_MIN", "BLEND_MAX",
        "BLEND_RGBA_ADD", "BLEND_RGBA_SUB", "BLEND_RGBA_MULT",
        "BLEND_RGBA_MIN", "BLEND_RGBA_MAX", "BLEND_PREMULTIPLIED",
        "BUTTON_LEFT", "BUTTON_RIGHT", "BUTTON_WHEELUP",
        "WINDOWRESIZED", "VIDEORESIZE", "ACTIVEEVENT", "TEXTINPUT",
    )])
    probe("locals hat dieselben", lambda: lokale(pg))
    probe("error erbt RuntimeError", lambda: issubclass(pg.error, RuntimeError))
    probe("version.ver", lambda: pg.version.ver)

    # ----- font -----------------------------------------------------------
    pg.font.init()
    probe("font.get_init", lambda: pg.font.get_init())
    probe("font.get_default_font", lambda: pg.font.get_default_font())
    for groesse in (12, 16, 20, 24, 32, 48):
        probe("font.masse(%d)" % groesse, lambda g=groesse: schrift_masse(pg, g))
    probe("font.render leer", lambda: leerer_text(pg))
    probe("font.render bg flags", lambda: mit_hintergrund(pg))
    probe("font.render ohne aa colorkey", lambda: ohne_aa(pg))
    # Ansatz und Vorschub: **exakt**, ueber den ganzen druckbaren
    # ASCII-Bereich und fuenf Groessen - 475 Zeichen. Daran haengt jede
    # Textausrichtung, also wird es auch so breit geprueft.
    probe("font.metrics vorschub", lambda: [
        (g, [(m[0], m[4]) for m in pg.font.Font(None, g).metrics(
            "".join(chr(c) for c in range(32, 127)))])
        for g in (12, 16, 24, 32, 48)])
    # Die Glyphenbox: bekannte Grenze, siehe BEKANNTE_GRENZEN. Auch die
    # wird ueber alle 475 Zeichen geprueft - nicht um gruen zu werden,
    # sondern damit die Meldung sagt, **wie viele** abweichen. Waechst
    # die Zahl, ist es keine bekannte Grenze mehr.
    probe("font.metrics glyphenbox", lambda: [
        (g, [(m[1], m[2], m[3]) for m in pg.font.Font(None, g).metrics(
            "".join(chr(c) for c in range(32, 127)))])
        for g in (12, 16, 24, 32, 48)])
    # Bekannte Grenze, siehe BEKANNTE_GRENZEN: der Tafelindex ueberlebt
    # den Weg durch den Speicher nicht.
    def tafelrundreise():
        f = pg.Surface((8, 8), 0, 8)
        ar = pg.PixelArray(f)
        aus = []
        for i in (0, 1, 63, 127, 128, 200, 255):
            ar[0, 0] = i
            aus.append((i, ar[0, 0]))
        ar.close()
        return aus

    probe("pixelarray.tafelindex", tafelrundreise)

    # Bekannte Grenze, siehe BEKANNTE_GRENZEN: `Surface()` ohne Groesse.
    def ohne_groesse():
        try:
            leer = repr(pg.Surface())
        except Exception as e:                      # noqa: BLE001
            leer = "%s: %s" % (type(e).__name__, e)
        tot = pg.Surface.__new__(pg.Surface)
        try:
            groesse = tot.get_size()
        except Exception as e:                      # noqa: BLE001
            groesse = "%s: %s" % (type(e).__name__, e)
        aus = [leer, repr(tot), groesse, tot.get_locked()]
        # **`tp_new` sieht seine Argumente nicht an** - gemessen gibt
        # `Surface.__new__(Surface, (4, 4))` eine **tote** Flaeche.
        mit = pg.Surface.__new__(pg.Surface, (4, 4))
        try:
            aus.append(mit.get_size())
        except Exception as e:                      # noqa: BLE001
            aus.append("%s: %s" % (type(e).__name__, e))
        # Und jede schlechte Groesse gibt denselben Satz.
        for wert in (1, (1,), "ab", (1, 2, 3), None):
            try:
                pg.Surface(wert)
                aus.append((wert, "ok"))
            except Exception as e:                  # noqa: BLE001
                aus.append((wert, type(e).__name__, str(e)))
        # Eine Unterklasse, die `super().__init__` mit eigener Form ruft -
        # das ging vorher nicht, weil `__new__` die Argumente schon ansah.
        class Zwei(pg.Surface):
            def __init__(self, breite, hoehe):
                super().__init__((breite, hoehe))

        try:
            aus.append(("Unterklasse", Zwei(3, 5).get_size()))
        except Exception as e:                      # noqa: BLE001
            aus.append(("Unterklasse", type(e).__name__, str(e)))
        # Und eine, die `super().__init__` **nicht** ruft: tot, wie bei
        # pygame.
        class Leer(pg.Surface):
            def __init__(self):
                pass

        try:
            aus.append(("ohne super", Leer().get_size()))
        except Exception as e:                      # noqa: BLE001
            aus.append(("ohne super", type(e).__name__, str(e)))
        return tuple(aus)

    probe("Surface ohne Groesse", ohne_groesse)
    # **Verlustfrei muss es genau stimmen.** WAV (auch ADPCM) und FLAC
    # gehen durch denselben Umrechner wie bei SDL - seit dem Umbau auf
    # dessen Sinc-Filter Byte fuer Byte. Diese Probe vergleicht den
    # ganzen Klang, nicht nur den Anfang: eine Abweichung irgendwo in der
    # Mitte faellt sonst nicht auf.
    def klang_hash(datei):
        import hashlib as _h
        import os as _os

        def messen():
            try:
                import pygame as _echt
                ordner = _os.path.join(_os.path.dirname(_echt.__file__),
                                       "examples", "data")
            except Exception:                        # noqa: BLE001
                return "kein pygame daneben"
            pfad = _os.path.join(ordner, datei)
            if not _os.path.exists(pfad):
                return "keine Beispieldaten"
            b = pg.mixer.Sound(pfad).get_raw()
            return (len(b), _h.sha256(b).hexdigest())

        return messen

    probe("mixer.wav umgerechnet", klang_hash("house_lo.wav"))
    probe("mixer.flac umgerechnet", klang_hash("house_lo.flac"))
    probe("mixer.adpcm umgerechnet", klang_hash("secosmic_lo.wav"))
    # Bekannte Grenze: die verlustbehafteten Dekodierer runden anders.
    probe("mixer.ogg umgerechnet", klang_hash("house_lo.ogg"))
    probe("mixer.mp3 umgerechnet", klang_hash("house_lo.mp3"))
    probe("font.size", lambda: [pg.font.Font(None, 24).size(t) for t in
                                ("", "i", "Hallo", "Hallo Welt 123")])
    probe("font.schalter", lambda: schalter(pg))

    # ----- elementweises Rechnen -----------------------------------------
    probe("math.elementwise rechnen", lambda: ew_rechnen(pg))
    probe("math.elementwise vorzeichen", lambda: ew_vorzeichen(pg))
    probe("math.elementwise vergleiche", lambda: ew_vergleiche(pg))
    probe("math.elementwise fehler", lambda: ew_fehler(pg))
    probe("math.elementwise drei achsen",
          lambda: tuple(pg.math.Vector3(1, 2, 3).elementwise() * pg.math.Vector3(2, 3, 4)))
    probe("math.elementwise typen", lambda: (
        type(pg.math.Vector2(1, 2).elementwise() * (2, 3)).__name__,
        pg.math.Vector2(1, 2).elementwise().__class__.__name__,
        bool(pg.math.Vector2(0, 0).elementwise()),
        bool(pg.math.Vector2(0, 1).elementwise())))

    # ----- math -----------------------------------------------------------
    V = pg.math.Vector2
    vektoren = [(3.0, 4.0), (0.0, 0.0), (-1.5, 2.25), (1e8, 0.1),
                (1 / 3, 2 / 3), (0.0, -7.0)]
    for t in vektoren:
        v = V(*t)
        probe("Vector2%s.repr" % (t,), lambda v=v: (repr(v), str(v)))
        probe("Vector2%s.masse" % (t,), lambda v=v: (
            v.length(), v.length_squared(), v.magnitude(),
            v.magnitude_squared(), bool(v), len(v), list(v)))
        probe("Vector2%s.normalize" % (t,), lambda v=v: tuple(v.normalize()))
        probe("Vector2%s.is_normalized" % (t,), lambda v=v: v.is_normalized())
        probe("Vector2%s.as_polar" % (t,), lambda v=v: v.as_polar())
        probe("Vector2%s.from_polar" % (t,), lambda v=v: aus_polar(pg, v.as_polar()))
        for grad in (0.0, 30.0, 90.0, 180.0, -45.0, 361.0):
            probe("Vector2%s.rotate(%s)" % (t, grad),
                  lambda v=v, g=grad: tuple(v.rotate(g)))
        probe("Vector2%s.dot" % (t,), lambda v=v: v.dot(V(1.0, 2.0)))
        probe("Vector2%s.cross" % (t,), lambda v=v: v.cross(V(1.0, 2.0)))
        probe("Vector2%s.angle_to" % (t,), lambda v=v: v.angle_to(V(1.0, 0.0)))
        probe("Vector2%s.distance_to" % (t,), lambda v=v: v.distance_to(V(1.0, 2.0)))
        probe("Vector2%s.project" % (t,), lambda v=v: tuple(v.project(V(1.0, 2.0))))
        probe("Vector2%s.reflect" % (t,), lambda v=v: tuple(v.reflect(V(0.0, 1.0))))
        probe("Vector2%s.clamp_magnitude" % (t,),
              lambda v=v: tuple(v.clamp_magnitude(1.0, 2.0)))
        for tt in (0.0, 0.25, 1.0):
            probe("Vector2%s.lerp(%s)" % (t, tt),
                  lambda v=v, tt=tt: tuple(v.lerp(V(1.0, 2.0), tt)))
            probe("Vector2%s.slerp(%s)" % (t, tt),
                  lambda v=v, tt=tt: tuple(v.slerp(V(1.0, 2.0), tt)))
            probe("Vector2%s.smoothstep(%s)" % (t, tt),
                  lambda v=v, tt=tt: tuple(v.smoothstep(V(1.0, 2.0), tt)))
        probe("Vector2%s.lerp ausserhalb" % (t,),
              lambda v=v: tuple(v.lerp(V(1.0, 2.0), 1.5)))
        probe("Vector2%s.move_towards" % (t,),
              lambda v=v: tuple(v.move_towards(V(1.0, 2.0), 1.0)))
        probe("Vector2%s.rechnen" % (t,), lambda v=v: (
            tuple(v + V(1.0, 1.0)), tuple(v - V(1.0, 1.0)), tuple(v * 2.0),
            tuple(2.0 * v), tuple(v / 2.0), tuple(-v),
            v == V(*t), v == t, v * V(1.0, 2.0)))
    probe("Vector2()", lambda: tuple(V()))
    probe("Vector2(5)", lambda: tuple(V(5)))
    probe("Vector2((3,4))", lambda: tuple(V((3, 4))))
    probe("Vector2 durch null", lambda: tuple(V(1, 2) / 0))
    probe("Vector2 update", lambda: aktualisiere(pg))
    probe("Vector2[1]", lambda: (V(3, 4)[0], V(3, 4)[1], V(3, 4)[-1]))
    probe("Vector2[9]", lambda: V(3, 4)[9])

    V3 = pg.math.Vector3
    for t3 in [(1.0, 2.0, 3.0), (0.0, 0.0, 0.0), (-1.0, 0.5, 2.0)]:
        w = V3(*t3)
        probe("Vector3%s.repr" % (t3,), lambda w=w: (repr(w), str(w)))
        probe("Vector3%s.masse" % (t3,), lambda w=w: (w.length(), w.length_squared(),
                                                      bool(w), list(w)))
        probe("Vector3%s.normalize" % (t3,), lambda w=w: tuple(w.normalize()))
        probe("Vector3%s.cross" % (t3,), lambda w=w: tuple(w.cross(V3(0.0, 1.0, 0.0))))
        probe("Vector3%s.dot" % (t3,), lambda w=w: w.dot(V3(1.0, 1.0, 1.0)))
        probe("Vector3%s.as_spherical" % (t3,), lambda w=w: w.as_spherical())
        probe("Vector3%s.angle_to" % (t3,), lambda w=w: w.angle_to(V3(1.0, 0.0, 0.0)))
        probe("Vector3%s.rotate_x" % (t3,), lambda w=w: tuple(w.rotate_x(30.0)))
        probe("Vector3%s.rotate_y" % (t3,), lambda w=w: tuple(w.rotate_y(30.0)))
        probe("Vector3%s.rotate_z" % (t3,), lambda w=w: tuple(w.rotate_z(30.0)))
        probe("Vector3%s.rotate" % (t3,),
              lambda w=w: tuple(w.rotate(30.0, V3(0.0, 0.0, 1.0))))
        probe("Vector3%s.project" % (t3,), lambda w=w: tuple(w.project(V3(1.0, 0.0, 0.0))))
        probe("Vector3%s.lerp" % (t3,), lambda w=w: tuple(w.lerp(V3(1.0, 1.0, 1.0), 0.25)))
        probe("Vector3%s.move_towards" % (t3,),
              lambda w=w: tuple(w.move_towards(V3(1.0, 1.0, 1.0), 0.5)))
        probe("Vector3%s.rechnen" % (t3,), lambda w=w, t3=t3: (
            tuple(w + V3(1.0, 1.0, 1.0)), tuple(w * 2.0), tuple(-w), w == t3))

    for f in ("clamp", "lerp", "invlerp", "remap", "smoothstep"):
        probe("math.%s da" % f, lambda f=f: hasattr(pg.math, f))
    probe("math.clamp", lambda: [pg.math.clamp(*a) for a in
                                 ((5, 1, 3), (0, 1, 3), (2, 1, 3),
                                  (5.5, 1.0, 3.0), (-1.0, 0.0, 1.0))])
    probe("math.clamp typen", lambda: [type(pg.math.clamp(*a)).__name__ for a in
                                       ((5, 1, 3), (5.0, 1, 3))])
    probe("math.lerp", lambda: [pg.math.lerp(0, 10, t) for t in
                                (0.0, 0.25, 0.5, 1.0, 1.5, -0.5)])
    probe("math.invlerp", lambda: [pg.math.invlerp(0, 10, v) for v in (0, 2.5, 10, 15)])
    probe("math.remap", lambda: [pg.math.remap(0, 10, 0, 100, v) for v in (0, 2.5, 10)])
    probe("math.smoothstep", lambda: [pg.math.smoothstep(0, 10, t) for t in
                                      (0.0, 0.25, 0.5, 1.0)])

    # ----- mask -----------------------------------------------------------
    probe("mask.Mask leer", lambda: maske_grund(pg, False))
    probe("mask.Mask voll", lambda: maske_grund(pg, True))
    probe("mask.set/get", lambda: maske_setzen(pg))
    probe("mask.overlap", lambda: maske_ueberdeckung(pg))
    probe("mask.zeichnen", lambda: maske_zeichnen(pg))
    probe("mask.auswertung", lambda: maske_auswertung(pg))
    probe("mask.to_surface", lambda: maske_flaeche(pg))
    probe("mask.from_surface", lambda: maske_aus_flaeche(pg))
    probe("mask.from_threshold", lambda: maske_schwelle(pg))
    probe("mask.scale", lambda: maske_skalieren(pg))

    # ----- Puffer-Protokoll ------------------------------------------------
    #
    # Neu in diesem Zug, also neu zu messen. Der Plan hatte es
    # ausgeschlossen; jetzt steht es da, und ungemessen gilt hier nichts.
    probe("puffer.get_view Beschreibung", lambda: puffer_sichten(pg))
    probe("puffer.Flaggen", lambda: puffer_flaggen(pg))
    probe("puffer.Sperre", lambda: puffer_sperre(pg))
    probe("puffer.aus Woerterbuch", lambda: puffer_woerterbuch(pg))
    probe("puffer.raw und write", lambda: puffer_rohbytes(pg))
    probe("puffer.Color", lambda: puffer_farbe(pg))
    probe("puffer.Mask", lambda: puffer_maske(pg))
    probe("puffer.get_array_interface", lambda: puffer_feldschnittstelle(pg))
    probe("puffer.unter 32 Bit", lambda: puffer_schmal(pg))

    # ----- pixelcopy -------------------------------------------------------
    probe("pixelcopy.surface_to_array", lambda: kopie_hin(pg))
    probe("pixelcopy.array_to_surface", lambda: kopie_her(pg))
    probe("pixelcopy.map_array", lambda: kopie_karte(pg))
    probe("pixelcopy.make_surface", lambda: kopie_neu(pg))
    probe("pixelcopy.Formate", lambda: kopie_formate(pg))

    # ----- surfarray und sndarray -----------------------------------------
    probe("surfarray.Felder", lambda: felder_flaeche(pg))
    probe("sndarray.Felder", lambda: felder_klang(pg))

    # ----- system, midi und Kamera -----------------------------------------
    probe("system.get_total_ram", lambda: ram_groessenordnung(pg))
    probe("midi.time", lambda: midi_uhr(pg))
    probe("midi.Geraeteliste", lambda: midi_geraete(pg))
    probe("midi.Fehler", lambda: midi_fehler(pg))
    probe("midi.Rechnerei", lambda: midi_rechnerei(pg))
    # Vier Proben an der Kamera, und **drei davon brauchen kein Geraet**:
    # ein Rechner ohne Kamera bekommt dieselben Antworten.
    probe("camera.Auskunft", lambda: kamera_auskunft(pg))
    probe("camera.colorspace", lambda: kamera_farbraum(pg))
    probe("camera.Fehler", lambda: kamera_fehler(pg))
    probe("camera.get_backends", lambda: kamera_rueckwaende(pg))
    probe("camera.Bild", lambda: kamera_bild(pg))

    # ----- Text: die drei Funde, die unter "Rasterkante" lagen -------------
    probe("freetype.Groesse", lambda: ft_groesse(pg))
    # War bis Runde 19 eine benannte Grenze - siehe `ft_textbreite`.
    probe("freetype.Textbreite", lambda: ft_textbreite(pg))
    probe("freetype Stile", lambda: ft_stile(pg))
    probe("freetype gedreht, senkrecht, Bruchgroessen", lambda: ft_lage(pg))
    probe("freetype Bitmapschriften und CFF", lambda: ft_bitmap_cff(pg))
    probe("font CFF und Typo-Masse", lambda: font_cff_typo(pg))
    # Bis Runde 19 eine benannte Grenze, seit Runde 20 gleich.
    probe("freetype zwei Achsen", lambda: ft_zwei_achsen(pg))
    probe("freetype get_metrics", lambda: ft_metriken(pg))
    probe("freetype Bitmapgroessen", lambda: ft_bitmapgroessen(pg))
    probe("freetype Kerning kern-Tabelle", lambda: ft_kern_tabelle(pg))
    probe("freetype kleine Groessen", lambda: ft_kleine_groessen(pg))
    probe("font Strikes, Symbol, Ladefehler", lambda: font_strikes_symbol(pg))
    probe("freetype.Leerkasten", lambda: ft_leerkasten(pg))
    probe("freetype.render_to Stufen", lambda: ft_zwei_stufen(pg))
    probe("font.Standardschrift", lambda: schrift_ohne_ordner(pg))

    # ----- Runde 21: was die Frage "ist alles da" fand ----------------------
    probe("Alpha-Blit unter 32 Bit", lambda: blit_unter_32(pg))
    probe("JPEG lesen", lambda: jpeg_lesen(pg))
    probe("JPEG und PNG schreiben", lambda: bild_schreiben(pg))

    # ----- `_sdl2`: die fuenf Untermodule, die nie gefragt wurden ----------
    probe("_sdl2.Untermodule", lambda: sdl2_untermodule(pg))
    probe("_sdl2.audio Geraete", lambda: sdl2_tongeraete(pg))
    probe("_sdl2.audio Zahlen", lambda: sdl2_audiozahlen(pg))
    probe("_sdl2.sdl2", lambda: sdl2_teilsysteme(pg))
    probe("_sdl2.controller_old", lambda: sdl2_altes_geraet(pg))
    probe("_sdl2.AUDIODEVICEADDED", lambda: sdl2_startereignisse(pg))
    # Bekannte Grenze, siehe BEKANNTE_GRENZEN: es gibt keinen rohen
    # Tonstrom und keine zentrale Mischstufe.
    probe("_sdl2.roher Tonstrom", lambda: sdl2_roher_strom(pg))

    # ----- Die neun Untermodule des neunten blinden Flecks -----------------
    probe("Untermodule gesamt", lambda: alle_untermodule(pg))
    probe("Hauptmodul Namen", lambda: hauptmodul_namen(pg))
    probe("Sternimport", lambda: sternimport(pg))
    probe("Klassenbasen", lambda: klassenbasen(pg))
    probe("Klassendoppel", lambda: klassendoppel(pg))
    probe("Instanznamen", lambda: instanznamen(pg))
    probe("Vektor Reste", lambda: vektor_reste(pg))
    probe("freetype Modul", lambda: freetype_modul(pg))
    probe("sysfont", lambda: sysfont_proben(pg))
    probe("Kleinkram", lambda: kleinkram(pg))
    probe("ftfont", lambda: ftfont_proben(pg))
    probe("sprite Gruppen", lambda: sprite_gruppen(pg))
    probe("Kanal", lambda: kanal_proben(pg))
    probe("Objekte", lambda: objekte(pg))
    probe("Erben", lambda: erben(pg))
    probe("Argumentfehler", lambda: argumentfehler(pg))
    probe("Fehlertexte", lambda: fehlertexte(pg))
    probe("Schluesselwoerter", lambda: schluesselwoerter(pg))
    probe("Klassennamen", lambda: klassennamen(pg))
    probe("Funktionsnamen", lambda: funktionsnamen(pg))
    probe("newbuffer.Py_buffer", lambda: neuer_puffer(pg))
    probe("newbuffer.BufferMixin", lambda: neuer_ausgeber(pg))
    probe("pypm.Auskunft", lambda: pypm_auskunft(pg))
    probe("pkgdata", lambda: paketdaten(pg))

    # ----- Die elfte Runde ------------------------------------------------
    probe("Felder zuweisen", lambda: felder_zuweisen(pg))
    probe("Konstantenwerte", lambda: konstantenwerte(pg))
    probe("Methoden falsch rufen", lambda: methoden_falsch(pg))
    probe("Klassen falsch bauen", lambda: klassen_bauen(pg))
    probe("Signaturform", lambda: signaturform(pg))
    probe("Dokumentation", lambda: dokumentation(pg))

    return p


# ---------------------------------------------------------------------------
#  Puffer-Protokoll
# ---------------------------------------------------------------------------

def puffer_sichten(pg):
    """Wie sich die elf Sichten beschreiben.

    Nur bei 32 Bit - darunter gibt rustgame einen `BufferError`, und das
    ist eine eigene Probe (`puffer.unter 32 Bit`).
    """
    s = pg.Surface((10, 6), pg.SRCALPHA, 32)
    aus = []
    for wort in [None, "0", "1", "2", "3", "r", "g", "b", "a"]:
        v = s.get_buffer() if wort is None else s.get_view(wort)
        d = v.__array_interface__
        aus.append((wort, v.length, d["shape"], d["strides"], d["typestr"]))
    return aus


def puffer_flaggen(pg):
    """Welche Anfrage durchgeht und welche nicht.

    Das ist die Probe, die den Witz der zweidimensionalen Sicht traegt:
    `PyBUF_F_CONTIGUOUS` geht, `PyBUF_C_CONTIGUOUS` nicht.
    """
    from pygame.tests.test_utils import buftools as bt
    s = pg.Surface((10, 6), 0, 32)
    v = s.get_view("2")
    aus = []
    for name in ["PyBUF_SIMPLE", "PyBUF_WRITABLE", "PyBUF_ND", "PyBUF_STRIDES",
                 "PyBUF_RECORDS", "PyBUF_RECORDS_RO", "PyBUF_FULL",
                 "PyBUF_C_CONTIGUOUS", "PyBUF_F_CONTIGUOUS",
                 "PyBUF_ANY_CONTIGUOUS"]:
        try:
            i = bt.Importer(v, getattr(bt, name))
            aus.append((name, i.ndim, i.format, i.shape, i.strides,
                        i.len, i.itemsize, i.readonly))
        except Exception as e:                      # noqa: BLE001
            aus.append((name, type(e).__name__))
    return aus


def puffer_sperre(pg):
    """Wann eine Sicht die Flaeche sperrt - und wann nicht."""
    import gc
    aus = []
    s = pg.Surface((2, 4), 0, 32)
    aus.append(("frisch", s.get_locked()))
    v = s.get_buffer()
    aus.append(("get_buffer", s.get_locked(), len(s.get_locks())))
    v = None
    gc.collect()
    aus.append(("fort", s.get_locked()))
    w = s.get_view("2")
    aus.append(("get_view", s.get_locked()))
    _ = w.length
    aus.append(("nach length", s.get_locked(), s.get_locks() == (w,)))
    w = None
    gc.collect()
    aus.append(("fort", s.get_locked()))
    return aus


def puffer_woerterbuch(pg):
    """`BufferProxy({...})` - und die Fehlertexte dazu."""
    from pygame.bufferproxy import BufferProxy
    aus = []
    b = BufferProxy({"typestr": "|u1", "shape": (10,), "data": (9, True)})
    aus.append(("laenge", b.length, b.__array_interface__))
    for d in [{}, {"shape": (2,)}, {"shape": 2, "typestr": "|u1",
                                    "data": (0, True)},
              {"shape": (), "typestr": "|u1", "data": (0, True)},
              {"shape": (2,), "typestr": "|u", "data": (0, True)},
              {"shape": (2,), "typestr": "|u1", "data": 0},
              {"shape": (2,), "typestr": "|u1", "data": (0, True),
               "strides": (1, 2)}]:
        try:
            BufferProxy(d).length
            aus.append((sorted(d), "geht"))
        except Exception as e:                      # noqa: BLE001
            aus.append((sorted(d), type(e).__name__, str(e)))
    # Ein Objekt ohne alles.
    try:
        BufferProxy(1).length
    except Exception as e:                          # noqa: BLE001
        aus.append(("int", type(e).__name__))
    return aus


def puffer_rohbytes(pg):
    """`raw` und `write` - samt der beiden verschiedenen Fehler."""
    s = pg.Surface((6, 2), 0, 32)
    s.fill((10, 20, 30))
    b = s.get_buffer()
    aus = [("laenge", b.length), ("raw", b.raw[:8])]
    for off, n in [(0, 48), (0, 49), (48, 1), (49, 1), (-1, 4), (2, 3)]:
        try:
            b.write(b"x" * n, off)
            aus.append((off, n, "geht"))
        except Exception as e:                      # noqa: BLE001
            aus.append((off, n, type(e).__name__, str(e)))
    return aus


def puffer_farbe(pg):
    """`memoryview(Color)` und `Color.__array_struct__`."""
    import pygame.tests.test_utils.arrinter as ai
    from pygame.tests.test_utils import buftools as bt
    c = pg.Color(5, 7, 13, 23)
    aus = []
    for n in range(1, 5):
        c.set_length(n)
        i = ai.ArrayInterface(c)
        aus.append((n, i.two, i.nd, i.typekind, i.itemsize, i.flags,
                    i.shape[0], i.strides[0]))
    c.set_length(4)
    i = bt.Importer(c, bt.PyBUF_FORMAT)
    aus.append(("puffer", i.ndim, i.format, i.len, i.itemsize, i.readonly))
    try:
        bt.Importer(c, bt.PyBUF_WRITABLE)
        aus.append(("schreibbar", "geht"))
    except Exception as e:                          # noqa: BLE001
        aus.append(("schreibbar", type(e).__name__))
    return aus


def puffer_maske(pg):
    """`memoryview(Mask)` - Form, Schrittweiten und ein gesetztes Bit."""
    m = pg.Mask((1000, 100))
    for punkt in [(0, 1), (100, 10), (173, 90)]:
        m.set_at(punkt, 1)
    v = memoryview(m)
    breite = 8 * v.strides[1]
    aus = [(v.shape, v.strides, v.itemsize, v.ndim, v.readonly)]
    for x, y in [(0, 1), (100, 10), (173, 90), (0, 0), (101, 10), (173, 91)]:
        aus.append(((x, y), (v[x // breite, y] >> (x % breite)) & 1))
    m2 = pg.Mask((5, 3))
    v2 = memoryview(m2)
    aus.append((v2.shape, v2.strides, v2.nbytes))
    return aus


def puffer_feldschnittstelle(pg):
    """`pygame.get_array_interface` - der Kasten als Woerterbuch."""
    from pygame.bufferproxy import BufferProxy
    kwds = {"shape": (5, 4, 3), "typestr": "|u1", "data": (0, True),
            "strides": (4, 20, 1)}
    d = pg.get_array_interface(BufferProxy(kwds))
    return (len(d), d["version"], d["shape"], d["typestr"], d["data"],
            d["strides"])


def puffer_schmal(pg):
    """Was bei 8, 16 und 24 Bit aus einem Puffer wird.

    **Hier weichen die Engines ab, und das ist die benannte Grenze.**
    pygame gibt eine Beschreibung, rustgame einen `BufferError` mit Grund.
    """
    aus = []
    for tiefe in [8, 16, 24, 32]:
        s = pg.Surface((10, 6), 0, tiefe)
        try:
            d = s.get_view("2").__array_interface__
            aus.append((tiefe, d["shape"], d["strides"], d["typestr"]))
        except Exception as e:                      # noqa: BLE001
            aus.append((tiefe, type(e).__name__))
    return aus


# ---------------------------------------------------------------------------
#  pixelcopy
# ---------------------------------------------------------------------------

def _feld(pg, form, format="=I"):
    from pygame.tests.test_utils import buftools as bt
    return bt.Exporter(form, format=format)


def kopie_hin(pg):
    """`surface_to_array` in allen sechs Sorten."""
    s = pg.Surface((4, 3), pg.SRCALPHA, 32)
    s.fill((10, 20, 30, 40))
    s.set_at((1, 1), (200, 100, 50, 25))
    aus = []
    for art in "PRGBAC":
        e = _feld(pg, (4, 3))
        pg.pixelcopy.surface_to_array(e, s, art)
        aus.append((art, bytes(e._buf)[:16]))
    e = _feld(pg, (4, 3, 3), format="B")
    pg.pixelcopy.surface_to_array(e, s)
    aus.append(("3D", bytes(e._buf)[:16]))
    # Und die Fehler.
    for form, fmt in [((5, 3), "=I"), ((4, 3), "B")]:
        try:
            pg.pixelcopy.surface_to_array(_feld(pg, form, fmt), s)
            aus.append((form, fmt, "geht"))
        except Exception as e:                      # noqa: BLE001
            aus.append((form, fmt, type(e).__name__, str(e)))
    try:
        pg.pixelcopy.surface_to_array(_feld(pg, (4, 3)), s, "X")
    except Exception as e:                          # noqa: BLE001
        aus.append(("X", type(e).__name__, str(e)))
    return aus


def kopie_her(pg):
    """`array_to_surface`, samt Ausbreiten von Spalte, Zeile und Pixel."""
    aus = []
    for form in [(4, 3), (1, 3), (4, 1), (1, 1)]:
        s = pg.Surface((4, 3), 0, 32)
        e = _feld(pg, form)
        e._buf[:] = [42] * e.buflen
        pg.pixelcopy.array_to_surface(s, e)
        aus.append((form, [s.get_at((x, y))[:3]
                           for x in range(4) for y in range(3)]))
    s = pg.Surface((4, 3), 0, 32)
    e = _feld(pg, (4, 3, 3), format="B")
    e._buf[:] = [7] * e.buflen
    pg.pixelcopy.array_to_surface(s, e)
    aus.append(("3D", s.get_at((0, 0))))
    return aus


def kopie_karte(pg):
    """`map_array` - eine Farbe, Spaltenstreifen, Zeilenstreifen."""
    s = pg.Surface((1, 1), 0, 32)
    aus = []
    c = pg.Color("salmon")
    c.set_length(3)
    z = _feld(pg, (4, 3))
    pg.pixelcopy.map_array(z, c, s)
    aus.append(("eine Farbe", bytes(z._buf)[:8], s.map_rgb(c)))
    q = _feld(pg, (3, 3), format="B")
    q._buf[:] = list(range(1, 10))
    z = _feld(pg, (4, 3))
    pg.pixelcopy.map_array(z, q, s)
    aus.append(("Spalten", bytes(z._buf)))
    q = _feld(pg, (4, 1, 3), format="B")
    q._buf[:] = list(range(1, 13))
    z = _feld(pg, (4, 3))
    pg.pixelcopy.map_array(z, q, s)
    aus.append(("Zeilen", bytes(z._buf)))
    return aus


def kopie_neu(pg):
    """`make_surface` - und die gemessene Ueberraschung mit acht Bit."""
    aus = []
    for fmt in ["B", "=H", "=I"]:
        s = pg.pixelcopy.make_surface(_feld(pg, (3, 5), fmt))
        aus.append((fmt, s.get_size(), s.get_bitsize(), s.get_masks()))
    s = pg.pixelcopy.make_surface(_feld(pg, (3, 5, 3), "B"))
    aus.append(("3D", s.get_size(), s.get_bitsize(), s.get_masks()))
    for form in [(5,), (3, 5, 4)]:
        try:
            pg.pixelcopy.make_surface(_feld(pg, form, "B"))
            aus.append((form, "geht"))
        except Exception as e:                      # noqa: BLE001
            aus.append((form, type(e).__name__, str(e)))
    return aus


def kopie_formate(pg):
    """Welche Elementformate `array_to_surface` annimmt - und welche nicht."""
    s = pg.Surface((3, 5), 0, 32)
    aus = []
    for fmt in ["=i", "=I", "=q", "=Q", "<i", ">i", "!i", "1i", "=1i", "q",
                "4x", "8x", "f", "d", "?", "x", "1x", "2x", "3x", "5x", "9x"]:
        from pygame.tests.test_utils import buftools as bt
        e = bt.Exporter((3, 5), format=fmt)
        e._buf[:] = [42] * e.buflen
        try:
            pg.pixelcopy.array_to_surface(s, e)
            aus.append((fmt, "geht", s.get_at((0, 0))))
        except Exception as e2:                     # noqa: BLE001
            aus.append((fmt, type(e2).__name__))
    return aus


# ---------------------------------------------------------------------------
#  surfarray und sndarray
# ---------------------------------------------------------------------------

#: Die elf Unsinnswerte der Probe [`felder_zuweisen`].
#:
#: Die ersten sieben waren der erste Satz; die letzten vier sind
#: dazugekommen, weil die ersten sieben drei Funde **nicht** sehen
#: konnten:
#:
#: * `True` - `freetype.Font.style = True` wurde hier abgewiesen, pygame
#:   nimmt es (`bool` ist eine Unterklasse von `int`),
#: * `2**31` - `font.Font.outline = 2**31` wurde hier stillschweigend
#:   **0**, wo pygame einen `OverflowError` wirft,
#: * `2**31-1` - der letzte Wert, der in einen C-`long` passt, und damit
#:   die Gegenprobe dazu.
#:
#: *Ein Wertesatz ist eine Liste, und eine Liste hat blinde Flecken.*
#: Dieselbe Lehre wie bei den handgepflegten Namenslisten, nur eine
#: Ebene tiefer.
_UNSINN = (
    ("str", lambda: "feldmarke"),
    ("None", lambda: None),
    ("liste", lambda: []),
    ("negativ", lambda: -7),
    ("riesig", lambda: 10 ** 30),
    ("tupel5", lambda: (1, 2, 3, 4, 5)),
    ("bruch", lambda: 1.5),
    ("wahr", lambda: True),
    ("falsch", lambda: False),
    ("clong", lambda: 2 ** 31 - 1),
    ("ueber", lambda: 2 ** 31),
)

#: Felder, an denen [`felder_zuweisen`] **nicht** messen kann, mit Grund.
_FELD_AUS = frozenset({
    # Bekannte Grenze `Cursor.type/.data schreibbar`: bei pygame zwei
    # Faecher in einem `__dict__` ohne jede Pruefung, hier aus dem Zeiger
    # abgeleitet und nur lesbar.
    "Cursor.type", "Cursor.data",
})


def _feld_instanzen(pg):
    """`(Name, Fabrik)` fuer alles, dessen Bau **nichts kostet**.

    Dieselbe Abwaegung wie bei [`instanznamen`]: ein Konstruktor, der ein
    Fenster oeffnet oder ein Geraet sucht, hat in einer Probe nichts zu
    suchen. Die Zahl der Instanzen steht im Ergebnis, damit sichtbar
    bleibt, wie viele es sind.
    """
    import importlib

    # **Nicht ueber `getattr(pg, "geometry")`.** Bei pygame ist das erst
    # ein Feld am Hauptmodul, nachdem es jemand selbst eingefuehrt hat;
    # bei rustgame ist es immer eines (ein einziges Modul, siehe die
    # Grenze `pygame.imageext am Hauptmodul`). Wer am Hauptmodul fragt,
    # misst darum die **Huelle** statt der Klasse - beim ersten Lauf
    # fehlten so 147 Zuweisungen auf der pygame-Seite, und die Probe
    # haette sie als Fund gemeldet.
    geo = importlib.import_module(pg.__name__ + ".geometry")
    return (
        ("Rect", lambda: pg.Rect(1, 2, 3, 4)),
        ("FRect", lambda: pg.FRect(1.5, 2.5, 3.5, 4.5)),
        ("Color", lambda: pg.Color(1, 2, 3, 4)),
        ("Vector2", lambda: pg.Vector2(1, 2)),
        ("Vector3", lambda: pg.Vector3(1, 2, 3)),
        ("Surface", lambda: pg.Surface((4, 4))),
        ("Mask", lambda: pg.mask.Mask((4, 4))),
        ("Clock", lambda: pg.time.Clock()),
        ("Event", lambda: pg.event.Event(pg.USEREVENT, {"x": 1})),
        ("PixelArray", lambda: pg.PixelArray(pg.Surface((2, 2)))),
        ("BufferProxy", lambda: pg.Surface((2, 2)).get_buffer()),
        ("Sprite", lambda: pg.sprite.Sprite()),
        ("DirtySprite", lambda: pg.sprite.DirtySprite()),
        ("Group", lambda: pg.sprite.Group()),
        ("GroupSingle", lambda: pg.sprite.GroupSingle()),
        ("LayeredUpdates", lambda: pg.sprite.LayeredUpdates()),
        ("LayeredDirty", lambda: pg.sprite.LayeredDirty()),
        ("collide_rect_ratio", lambda: pg.sprite.collide_rect_ratio(1.0)),
        ("collide_circle_ratio", lambda: pg.sprite.collide_circle_ratio(1.0)),
        # `None` ist die eingebaute Schrift - jede Huelle nimmt ihre
        # eigene, und einen Pfad braucht es dafuer nicht.
        ("Font", lambda: pg.font.Font(None, 20)),
        ("SysFont", lambda: pg.font.SysFont("arial", 20)),
        ("freetype.Font", lambda: pg.freetype.Font(None, 20)),
        ("Circle", lambda: geo.Circle(1, 2, 3)),
        ("Line", lambda: geo.Line(1, 2, 3, 4)),
        ("VectorElementwiseProxy", lambda: pg.Vector2(1, 2).elementwise()),
        ("display.Info", lambda: pg.display.Info()),
        ("version.vernum", lambda: pg.version.vernum),
        ("Cursor", lambda: pg.cursors.Cursor(pg.SYSTEM_CURSOR_ARROW)),
        ("Sound", lambda: pg.mixer.Sound(buffer=b"\0" * 400)),
        ("Channel", lambda: pg.mixer.Channel(0)),
        ("PowerState", lambda: pg.system.PowerState(
            None, None, False, True, False, False, True, False)),
    )


def felder_zuweisen(pg):
    """Was sagt ein **Feld**, wenn man ihm Unsinn zuweist?

    ## Die elfte Frage

    `Argumentfehler` ruft jede Funktion mit siebzehn Argumenten,
    `Fehlertexte` mit einem Argument falscher Sorte. Zusammen haben die
    zwei **98** echte Fehler gefunden - und keine von beiden hat je einem
    **Feld** etwas zugewiesen. Diese Probe tut das: 31 Instanzen, jedes
    nicht rufbare oeffentliche Feld, elf Unsinnswerte, und festgehalten
    wird `(Ausnahmeart, Satzgestalt)` oder `(OK, der neue Wert)`.

    **1980 Zuweisungen, davon gingen 528 auseinander.** Der Fund in
    Gruppen:

    | Was | Zuweisungen |
    |---|---:|
    | `freetype.Font`s elf Schalter nahmen **jeden** Wert an; pygame prueft mit `PyBool_Check` und lehnt sogar `1` ab | 77 |
    | `font.Font`s vier Schalter lehnten alles ausser `True`/`False` ab; pygame liest dort `PyObject_IsTrue` - **genau umgekehrt** | 56 |
    | Sprite- und Gruppenfelder waren getypt; pygames `sprite.py` ist reines Python und nimmt alles | 75 |
    | `Circle` und `Line` gaben Maschinensaetze, wo pygame vier eigene Sorten hat | 85 |
    | `underline_adjustment` hatte **keine** Bereichspruefung ([-2, 2]) | 3 |
    | `rotation` warf bei `10**30`, wo pygame modulo 360 rechnet (**280**) | 2 |
    | `fgcolor`/`bgcolor` gaben `ValueError`, pygame `AttributeError` | 11 |
    | `style` ohne das Hex im Satz (`Invalid style value fffffff9`) | 3 |
    | und 216 weitere derselben Sorten | |

    ## Warum der **Wert** mitgelesen wird

    Eine gelungene Zuweisung ist kein Beweis. `c.area = 2**31-1` geht auf
    beiden Seiten durch - aber `c.area` gibt danach bei pygame
    `2147483646.9999998` und gab hier `2147483646.9999995`, weil
    `PI * (r * r)` nicht dasselbe ist wie `(PI * r) * r`. Drei Einheiten
    im letzten Bit, und sie waeren unsichtbar geblieben.

    ## Warum jede Zuweisung eine **frische** Instanz bekommt

    Weil eine gelungene Zuweisung das Objekt aendert und die naechste
    Messung dann etwas anderes misst. Das kostet 1980 Konstruktoren, und
    es ist der Preis dafuer, dass die Reihenfolge nichts entscheidet -
    siehe REGELN.md, *eine Messung, die von der Reihenfolge abhaengt, ist
    keine.*
    """
    aus = []
    for kl, fabrik in _feld_instanzen(pg):
        try:
            muster = fabrik()
        except Exception as e:                       # noqa: BLE001
            aus.append((kl, "(bau)", "", type(e).__name__,
                        _satz_gestalt(str(e), pg.__name__)))
            continue
        for a in sorted(dir(muster)):
            if a.startswith("_"):
                continue
            try:
                if callable(getattr(muster, a)):
                    continue
            except Exception:                        # noqa: BLE001
                pass
            if "%s.%s" % (kl, a) in _FELD_AUS:
                continue
            for marke, mach in _UNSINN:
                try:
                    o = fabrik()
                except Exception:                    # noqa: BLE001
                    break
                try:
                    setattr(o, a, mach())
                except BaseException as e:           # noqa: BLE001
                    aus.append((kl, a, marke, type(e).__name__,
                                _satz_gestalt(str(e), pg.__name__)))
                    continue
                try:
                    wert = repr(getattr(o, a))
                except Exception as e:               # noqa: BLE001
                    wert = "<lesen: %s>" % type(e).__name__
                # **Erst normalisieren, dann kuerzen** - sonst brechen die
                # zwei Seiten an verschiedenen Stellen ab (REGELN.md).
                wert = wert.replace(pg.__name__, "@").replace("pygame", "@")
                wert = re.sub(r"0x[0-9a-fA-F]+", "0xADR", wert)
                aus.append((kl, a, marke, "OK", wert[:120]))
    if len(aus) < 1500:
        raise RuntimeError("Felder zuweisen: nur %d Zuweisungen" % len(aus))
    return tuple(aus)


#: Werte, die [`konstantenwerte`] **nicht** vergleichen kann, mit Grund.
#:
#: `sysfont.pygame_main_file` ist der Weg zur `__init__.py` des Pakets
#: und damit notwendig verschieden - dieselbe Sorte wie
#: `pkgdata.getResource` bei den Fehlertexten. Alles andere in dieser
#: Liste ist die **Fassung selbst**: `ver`, `vernum`, `rev` und `SDL`
#: stehen an mehreren Stellen und sagen, was das Paket ist, nicht was es
#: tut.
_WERT_AUS = frozenset({
    # **Wege ins Paket** - notwendig verschieden, dieselbe Sorte wie
    # `pkgdata.getResource` bei den Fehlertexten. Die Huelle ist eine
    # einzelne `.pyd`, pygame ein Ordner mit `__init__.py`.
    "sysfont.pygame_main_file", "docs.PKG_DIR",
    # **Die Fassung selbst.** Sie sagt, was das Paket ist, nicht was es
    # tut; `rev` ist hier `'rustgame'` und bei pygame `''`.
    "(haupt).ver", "(haupt).vernum", "(haupt).rev",
    "version.ver", "version.vernum", "version.rev", "version.SDL",
    "version.sdl_versioned",
    "base.ver", "base.vernum", "base.rev",
    # **Die Schriftentabelle.** Ihr Inhalt haengt an den installierten
    # Schriften, und die **Reihenfolge** der Schnitte ist bei pygame die
    # der Registrierung, hier die sortierte - ein `repr` geht darum
    # auseinander, obwohl beide Woerterbuecher gleich sind. Gemessen
    # wird sie an der richtigen Stelle: die Probe `sysfont` vergleicht
    # Laengen, Namen und Eintraege, und zwar als Woerterbuch.
    "sysfont.Sysfonts", "sysfont.Sysalias",
})


def konstantenwerte(pg):
    """Jeder Konstanten**wert**, abgelesen statt genannt.

    ## Der elfte blinde Fleck

    `alle_untermodule` vergleicht in jedem der sechzig Untermodule die
    **Namen**. Die Probe `konstanten` vergleicht **58 Werte** am
    Hauptmodul - und die 58 stehen dort von Hand aufgeschrieben. Nach
    allem, was REGELN.md ueber handgepflegte Listen sagt, war das die
    naechste Frage: *was sagt der Wert?*

    Abgelesen werden alle 1892 Namen, die keine Klasse, kein Modul und
    nichts Rufbares sind - samt **Typ**, denn:

    | Fund | pygame | hier |
    |---|---|---|
    | `freetype.STYLE_DEFAULT` | `255` | `-1` |
    | `_freetype.STYLE_DEFAULT` | `255` | `-1` |
    | `sysfont.OpenType_extensions` | `frozenset({...})` | `(...)` - ein Tupel |

    `STYLE_DEFAULT` war doppelt falsch: derselbe Wert ist in `freitext.rs`
    auch der Platzhalter fuer "nimm den Stil der Schrift", also gab
    `f.style = 255` einen `ValueError` und `f.style = -1` setzte
    stillschweigend nichts. Keine andere Probe konnte das sehen - der
    Wert steht in keiner erzeugten Tabelle, weil er als Rust-Konstante
    von Hand dalag.

    Der **Typ** gehoert mit, weil `frozenset` und Tupel auf `in` gleich
    antworten und auf `type()`, `repr()` und jede Mengenrechnung nicht.
    """
    import importlib
    import pkgutil

    namen = sorted(m.name for m in pkgutil.iter_modules(
        __import__("pygame").__path__) if m.name != "tests")
    # **Erst alle laden, dann lesen.** Sonst entscheidet die Reihenfolge:
    # `pygame.surfarray` ist am Hauptmodul erst ein Feld, nachdem es
    # jemand eingefuehrt hat, und bei rustgame steht dort bis dahin ein
    # Stellvertreter (`_feld_sucher.Spaet`). Wer das Hauptmodul vor den
    # Untermodulen liest, vergleicht also zwei verschiedene Zustaende -
    # und genau das meldete der erste Lauf als zwei Funde. *Eine Messung,
    # die von der Reihenfolge abhaengt, ist keine.*
    geladen = []
    for n in [""] + namen:
        try:
            geladen.append((n, importlib.import_module(
                pg.__name__ + ("." + n if n else ""))))
        except Exception:                            # noqa: BLE001
            continue
    aus = []
    for n, m in geladen:
        for a in sorted(dir(m)):
            if a.startswith("_"):
                continue
            try:
                w = getattr(m, a)
            except Exception:                        # noqa: BLE001
                continue
            if callable(w) or isinstance(w, type(importlib)):
                continue
            weg = "%s.%s" % (n or "(haupt)", a)
            if weg in _WERT_AUS:
                continue
            text = repr(w).replace(pg.__name__, "@").replace("pygame", "@")
            aus.append((weg, type(w).__name__,
                        re.sub(r"0x[0-9a-fA-F]+", "0xADR", text)[:200]))
    if len(aus) < 1500:
        raise RuntimeError("Konstantenwerte: nur %d Werte" % len(aus))
    return tuple(aus)


#: Methoden, die [`methoden_falsch`] **nicht** ruft, mit Grund.
_METHODE_AUS = frozenset({
    # **Schreibt in die Datei, deren Namen man ihm gibt.** Ein Lauf
    # hinterliess eine Datei `methodenmarke` im Arbeitsverzeichnis -
    # dieselbe Falle wie bei `print_debug_info`. *Eine Probe, die die Welt
    # aendert, ist keine Probe.*
    "Surface.save",
    # Haelt den Lauf an, wenn das Argument doch durchgeht.
    "Clock.tick", "Clock.tick_busy_loop",
    "Sound.fadeout", "Channel.fadeout",
})


def methoden_falsch(pg):
    """Jede **Methode** jeder Instanz, einmal mit falschem Argument.

    ## Die zwoelfte Frage

    `Fehlertexte` ruft jede freie Funktion mit einem Argument falscher
    Sorte - und an Instanzen kannte sie **sieben** Typen: `Vector2`,
    `Vector3`, `Rect`, `FRect`, `Color`, `Surface`, `Mask`. Die Methoden
    von `Font`, `freetype.Font`, `Sound`, `Channel`, `Circle`, `Line`,
    den Sprites und den vier Gruppen hat damit **nie jemand** falsch
    gerufen. Dass dort etwas liegt, war schon aus Runde 11 bekannt:
    `font.set_bold('x')` nimmt pygame an (Wahrheitswert), hier flog ein
    `TypeError` - und **keine** Probe sah es, weil `Font` in keiner
    Instanzliste stand.

    409 Rufe, **46 gingen auseinander**:

    | Was | Rufe |
    |---|---:|
    | `Circle` und `Line`: pygame hat fuer fast jede Methode einen eigenen Satz (`move requires a pair of numbers`, `A CircleType object was expected`, `Invalid shape argument, must be a Circle, Rect / Frect or a coordinate`), hier gab ein gemeinsamer Helfer ueberall denselben | 15 |
    | Rekursionswaechter - **kein Fund**, siehe unten | 16 |
    | `LayeredUpdates`/`LayeredDirty`: `get_sprites_from_layer('x')` und `remove_sprites_of_layer('x')` geben bei pygame `[]`, hier flog ein `TypeError`; `get_sprite('x')` gibt Pythons Listensatz; `get_sprites_at` den von `Rect.collidelistall` | 8 |
    | **`LayeredUpdates.draw('x')` tat hier gar nichts.** pygame holt `surface.blit` als erstes und gibt `AttributeError: 'str' object has no attribute 'blit'`; bei leerer Gruppe lief der Durchlauf hier ins Leere, und ein falsches Ziel fiel nie auf | 1 |
    | `BufferProxy.write('abc')` nimmt pygame an (`s#` liest `str` als UTF-8), hier wies PyO3 ab | 1 |
    | `PixelArray.extract('red')`: pygame nimmt hier **nur** `Color` und RGB[A]-Tupel, nicht die Namenstabelle | 1 |
    | `Font.set_linesize('x')`, `Surface.convert_alpha('x')` (Argument **vor** Zustand), `Circle.rotate('x')` | 4 |

    ## Die sechzehn Rekursionen sind keine Abweichung

    `Group.add('methodenmarke')` laeuft bei **beiden** in eine
    Endlosschleife: eine Zeichenkette ist eine Folge von Zeichenketten,
    und `add` steigt in jede Folge hinab. Beide melden `RecursionError`,
    aber mit dem Satz ihres eigenen Waechters:

    ```text
    pygame     maximum recursion depth exceeded
    rustgame   Stack overflow (used 2912 kB) while calling a Python object
    ```

    Das ist dieselbe Lage wie bei `<argumentfehler>`: **der Satz stammt
    von der Laufzeit, nicht von pygame** - und unserer traegt sogar eine
    Zahl, die von Lauf zu Lauf wechselt. Die **Art** ist gleich, und sie
    ist das Einzige, worauf sich ein `except` verlassen darf. Darum die
    Marke `<rekursion>`; sie erkennt eine Regel, keine Liste.
    """
    aus = []
    for kl, fabrik in _feld_instanzen(pg):
        try:
            muster = fabrik()
        except Exception as e:                       # noqa: BLE001
            aus.append((kl, "(bau)", type(e).__name__,
                        _satz_gestalt(str(e), pg.__name__)))
            continue
        for a in sorted(dir(muster)):
            if a.startswith("_") or "%s.%s" % (kl, a) in _METHODE_AUS:
                continue
            try:
                if not callable(getattr(muster, a)):
                    continue
            except Exception:                        # noqa: BLE001
                continue
            # **Frische Instanz je Ruf** - eine Methode, die durchgeht,
            # aendert das Objekt.
            try:
                w = getattr(fabrik(), a)
            except Exception:                        # noqa: BLE001
                continue
            try:
                w("methodenmarke")
                aus.append((kl, a, "KEIN FEHLER", ""))
            except BaseException as e:               # noqa: BLE001
                aus.append((kl, a, type(e).__name__,
                            _satz_gestalt(str(e), pg.__name__)))
    if len(aus) < 300:
        raise RuntimeError("Methoden falsch rufen: nur %d Rufe" % len(aus))
    return tuple(aus)


#: Klassen, die [`klassen_bauen`] **nicht** baut, mit Grund.
_BAU_AUS = frozenset({
    # **Stuerzt pygame ab**: `newbuffer.Py_buffer(1)` gibt "(pygame
    # parachute) Segmentation Fault" - einer der drei gemessenen
    # pygame-Fehler, siehe GRENZEN.md.
    "newbuffer.Py_buffer",
    # Geraete und Fenster. Ein Bau, der die Welt anfasst, ist keine
    # Probe - dieselbe Abwaegung wie bei [`instanznamen`].
    "_sdl2.video.Window", "_sdl2.Window", "window.Window", "(haupt).Window",
    "_sdl2.audio.AudioDevice", "_sdl2.AudioDevice",
    "_camera.Camera", "camera.Camera",
    "_sdl2.controller.Controller", "_sdl2.Controller",
    "joystick.Joystick", "(haupt).Joystick", "midi.Input", "midi.Output",
    "pypm.Input", "pypm.Output",
    # **Die bekannte Grenze `_render.Image`.** pygame hat dort einen
    # leeren Stummel und daneben die echte Klasse `_sdl2.video.Image`;
    # hier ist es **eine** Klasse mit zwei Namen, also sagt sie unter
    # beiden Namen dasselbe. Gemessen: `_render.Image('x')` baut bei
    # pygame anstandslos (der Stummel nimmt alles), `_render.Renderer('x')`
    # gibt dort den Satz seines Argumentlesers. Unter `_sdl2.*` stimmen
    # beide woertlich.
    "_render.Image", "_render.Renderer",
})


def klassen_bauen(pg):
    """Jede **Klasse**, einmal mit einem Argument falscher Sorte gebaut.

    ## Die dreizehnte Frage

    `Fehlertexte` laesst Klassen ausdruecklich aus
    (`if not callable(w) or isinstance(w, type): continue`), und
    `Argumentfehler` ruft sie mit **siebzehn** Argumenten. Mit **einem**
    falschen hat sie nie jemand gebaut - 105 Klassen, **15 Abweichungen**:

    | Was | Klassen |
    |---|---:|
    | **Ein deutscher Satz in einer Ausnahme**, siehe unten | 7 |
    | `Mask('x')`: pygames `size must be two numbers` statt PyO3s Satz | 3 |
    | `_sdl2.Image('x')`, `_sdl2.Renderer('x')`: `AttributeError` aus der naechsten Zeile statt `TypeError` mit dem erwarteten Typ | 2 |
    | die zwei `_render.*` - **kein Fund**, die bekannte Grenze | 2 |

    ## Der Fund: ein deutscher Satz, den der Waechter nicht sehen konnte

    ```text
    pygame     FileNotFoundError: No file 'nixda' found in working
                                  directory '<Arbeitsordner>'.
    rustgame   FileNotFoundError: Das System kann die angegebene Datei
                                  nicht finden. (os error 2)
    ```

    Das ist der Satz **von Windows**, in der Systemsprache - und
    `tools/deutsche_fehlertexte.py` meldete weiter 0, weil er nirgends im
    Quelltext steht. *Ein Sucher, der Gestalten aufzaehlt, findet nur die,
    die jemand aufgezaehlt hat* - diesmal nicht im Werkzeug, sondern in
    der Annahme, deutsche Saetze kaemen nur von uns.

    Betroffen waren alle sieben Wege zu einer Schrift (`font.Font`,
    `font.FontType`, `freetype.Font`, `_freetype.Font`, `ftfont.Font`,
    `ftfont.FontType` und das `Font` am Hauptmodul). Jetzt steht dort
    pygames Satz, samt seiner Unterscheidung zwischen einem relativen und
    einem absoluten Pfad - siehe [`crate::text::datei_fehlt`].
    """
    import importlib
    import pkgutil

    namen = sorted(m.name for m in pkgutil.iter_modules(
        __import__("pygame").__path__) if m.name != "tests")
    aus = []
    for n in [""] + namen:
        try:
            m = importlib.import_module(pg.__name__ + ("." + n if n else ""))
        except Exception:                            # noqa: BLE001
            continue
        for a in sorted(dir(m)):
            if a.startswith("_"):
                continue
            try:
                w = getattr(m, a)
            except Exception:                        # noqa: BLE001
                continue
            if not isinstance(w, type):
                continue
            weg = "%s.%s" % (n or "(haupt)", a)
            if weg in _BAU_AUS:
                continue
            try:
                o = w("klassenmarke")
                aus.append((weg, "KEIN FEHLER", type(o).__name__))
            except BaseException as e:               # noqa: BLE001
                aus.append((weg, type(e).__name__,
                            _satz_gestalt(str(e), pg.__name__)))
    if len(aus) < 80:
        raise RuntimeError("Klassen falsch bauen: nur %d Klassen" % len(aus))
    return tuple(aus)


#: Zwischenspeicher fuer [`_form_wege`].
_FORMWEGE = []


def _form_wege():
    """Die Wege, auf denen **pygame** eine Signatur hat.

    **Die Frage kommt von pygame, nicht von uns** - dieselbe Regel wie
    bei [`_stub_namen`]. PyO3 erzeugt fuer jede Funktion eine Signatur,
    pygames C-Funktionen haben meist keine (gemessen: **662** von 2018);
    wer beide Seiten vergleicht, ohne die Frage von aussen zu nehmen,
    vergleicht 662-mal "hat eine" mit "hat keine" und sieht nichts
    anderes mehr.
    """
    if _FORMWEGE:
        return _FORMWEGE[0]
    import importlib
    import inspect
    import pkgutil

    import pygame

    namen = sorted(m.name for m in pkgutil.iter_modules(pygame.__path__)
                   if m.name != "tests")
    aus = []
    for n in [""] + namen:
        try:
            m = importlib.import_module("pygame" + ("." + n if n else ""))
        except Exception:                            # noqa: BLE001
            continue
        mn = n or "(haupt)"
        for a in sorted(dir(m)):
            if a.startswith("_"):
                continue
            try:
                w = getattr(m, a)
            except Exception:                        # noqa: BLE001
                continue
            if isinstance(w, type):
                for b in sorted(dir(w)):
                    if b.startswith("_"):
                        continue
                    try:
                        v = getattr(w, b)
                    except Exception:                # noqa: BLE001
                        continue
                    if callable(v) and _signaturform(v) != "keine":
                        aus.append((mn, a, b))
            elif callable(w) and _signaturform(w) != "keine":
                aus.append((mn, None, a))
    _FORMWEGE.append(tuple(aus))
    return _FORMWEGE[0]


def _signaturform(o):
    """Die **Form** einer Signatur - Zahlen, keine Namen.

    `(nur Stellung)+(als Schluesselwort moeglich)` und zwei Marken fuer
    `*args`/`**kwargs`. Die **Namen** stehen absichtlich nicht darin:
    pygames C-Funktionen tragen dort CPythons Platzhalter (`object`),
    unsere den Rust-Namen - `(self, object, /)` gegen `(self, liste, /)`.
    Rufen laesst sich beides gleich, und darauf kommt es an.
    """
    import inspect

    try:
        teile = list(inspect.signature(o).parameters.values())
    except Exception:                                # noqa: BLE001
        return "keine"
    nur = kw = 0
    va = vk = False
    for x in teile:
        if x.name in ("self", "cls"):
            continue
        if x.kind == x.POSITIONAL_ONLY:
            nur += 1
        elif x.kind in (x.POSITIONAL_OR_KEYWORD, x.KEYWORD_ONLY):
            kw += 1
        elif x.kind == x.VAR_POSITIONAL:
            va = True
        else:
            vk = True
    return "%d+%d%s%s" % (nur, kw, "*" if va else "", "**" if vk else "")


#: Wege, an denen [`signaturform`] **nicht** vergleichen kann, mit Grund.
_FORM_AUS = frozenset({
    # `display.Info` ist hier eine **Fabrik** und nimmt `*args, **kwargs`,
    # obwohl sie jedes Argument zurueckweist - nur so kommt pygames
    # genauer Satz heraus: `pygame.display.Info() takes no arguments
    # (1 given)`. Der Wortlaut ist das Verhalten, die Form ist es nicht;
    # bei pygame ist `Info` eine Klasse mit `()`.
    "display.Info",
})


def signaturform(pg):
    """Die Form **jeder** Signatur, die pygame hat - ohne die Namen.

    ## Die vierzehnte Frage

    `Schluesselwoerter` fragt seit Runde sechs, ob eine **Modulfunktion**
    einen Namen als Schluesselwort annimmt, den pygame ablehnt - und
    stand dabei mit der Begruendung da, Methoden seien nicht zu messen,
    weil `inspect` an die Parameternamen der C-Typen nicht herankommt.

    **Das stimmt fuer die Namen und nicht fuer die Form:**

    ```text
    inspect.signature(pygame.font.Font.set_bold)    (self, object, /)
    inspect.signature(rustgame.font.Font.set_bold)  (self, /, an)
    ```

    pygames Methode nimmt **kein** Schluesselwort, unsere nahm `an=...` -
    und `an` ist dazu deutsch. Nachgezaehlt waren es **104 Methoden**, in
    denen ein Schluesselwort ging, das pygame zurueckweist; die Namen
    darin: `an`, `wert`, `liste`, `groesse`, `tafel`, `vorlage`,
    `erstes`, `grad`, `o`. Ein `/` am Ende der PyO3-Signatur stellt das
    gleich und macht die Namen nebenbei unnennbar.

    Dazu fand dieselbe Messung vier Formfehler, die kein Aufruf zeigt:

    | Stelle | pygame | vorher hier |
    |---|---|---|
    | `event.set_blocked`, `set_allowed`, `get_blocked` | ein Pflichtargument (`takes exactly one argument (0 given)`) | `=None`, also weglassbar - `set_blocked()` sperrte **nichts** |
    | `Window.flash`, `set_icon`, `set_modal_for` | nur nach Stellung | nahmen ein Schluesselwort |
    | `ftfont.Font.render` | vier Parameter | **fuenf** - `wraplength` hat pygames `ftfont` nicht |
    | `_sdl2.Image.get_rect` | `(self)` | `**kwargs` - die nimmt nur `Texture.get_rect` |
    | `joystick.JoystickType.set_led` | ein Argument, nur Stellung | `*args` |

    ## Warum die Frage von pygame kommen muss

    PyO3 erzeugt fuer **jede** Funktion eine Signatur; pygames
    C-Funktionen haben meist keine (662 von 2018). Verglichen werden
    darum nur die Wege, auf denen pygame eine hat - siehe
    [`_form_wege`]. Sonst meldete die Probe 662-mal dasselbe und
    verdeckte alles andere.
    """
    import importlib

    aus = []
    for mn, kl, name in _form_wege():
        if "%s.%s" % (mn, name) in _FORM_AUS:
            continue
        try:
            m = importlib.import_module(
                pg.__name__ + ("." + mn if mn != "(haupt)" else ""))
        except Exception:                            # noqa: BLE001
            continue
        o = m
        for teil in ([kl] if kl else []) + [name]:
            try:
                o = getattr(o, teil)
            except Exception:                        # noqa: BLE001
                o = None
                break
        if o is None:
            aus.append((mn, kl, name, "fehlt"))
            continue
        aus.append((mn, kl, name, _signaturform(o)))
    if len(aus) < 800:
        raise RuntimeError("Signaturform: nur %d Wege" % len(aus))
    return tuple(aus)


#: Zwischenspeicher fuer [`_doku_wege`].
_DOKUWEGE = []


def _doku_wege():
    """Die Module und Klassen, an denen **pygame** einen Satz hat.

    Wieder von aussen gefragt, wie bei [`_form_wege`] und
    [`_stub_namen`]: **32** Klassen haben hier einen Satz und bei pygame
    keinen (`sprite.RenderPlain`, `_sdl2.Texture`, `Window` und die
    anderen aus den eingebetteten Modulen). Dort ist nichts zu
    vergleichen - ein `None` nachzubauen hiesse, eine Luecke nachzubauen,
    und das ist in diesem Projekt dreimal als falsch gemessen worden
    (`camera.get_raw`, `query_image`, `mouse.set_relative_mode`).

    Die deutsche Prosa an diesen 32 bleibt also stehen und steht in der
    Grenze `__doc__ traegt deutsche Prosa`.
    """
    if _DOKUWEGE:
        return _DOKUWEGE[0]
    import importlib
    import pkgutil

    import pygame

    namen = sorted(m.name for m in pkgutil.iter_modules(pygame.__path__)
                   if m.name != "tests")
    aus = []
    for n in [""] + namen:
        try:
            m = importlib.import_module("pygame" + ("." + n if n else ""))
        except Exception:                            # noqa: BLE001
            continue
        mn = n or "(haupt)"
        d = getattr(m, "__doc__", None)
        if isinstance(d, str) and d.strip():
            aus.append((mn, ""))
        for a in sorted(dir(m)):
            if a.startswith("_"):
                continue
            try:
                w = getattr(m, a)
            except Exception:                        # noqa: BLE001
                continue
            if not isinstance(w, type):
                continue
            d = getattr(w, "__doc__", None)
            if isinstance(d, str) and d.strip():
                aus.append((mn, a))
    _DOKUWEGE.append(tuple(aus))
    return _DOKUWEGE[0]


def dokumentation(pg):
    """Der `__doc__` von **jedem Modul und jeder Klasse** - woertlich.

    ## Was gemessen wurde

    `__doc__` ist Schnittstelle: `help()`, `pydoc` und jede
    Entwicklungsumgebung zeigt ihn. Gemessen an 2819 Namen:

    | | pygame | vorher hier |
    |---|---:|---:|
    | hat einen Docstring | 2501 | 1830 |
    | erste Zeile ist eine Signatur | 1384 | 0 |
    | **deutsche Prosa** | 0 | 1129 |

    Die 36 fehlenden **Modul**-Saetze und die deutschen Saetze an
    Modulen und Klassen sind behoben: `tools/gen_moduldoku.py` liest sie
    aus echtem pygame, und `lib.rs` legt sie beim Aufbau darueber. Diese
    Probe haelt das nach - **woertlich**, denn es ist eine erzeugte
    Tabelle, und erzeugte Tabellen fallen still zurueck.

    ## Was **nicht** zu beheben ist

    An einer Methode ist `__doc__` nicht schreibbar
    (`attribute '__doc__' of 'method_descriptor' objects is not
    writable`), und der Quelltext tragt seine Doku auf Deutsch - so ist
    die Regel dieses Projekts. Darum bleiben 530 Rufbare ohne Satz und
    1063 mit deutscher Prosa; das ist die benannte Grenze
    `__doc__ traegt deutsche Prosa`. Die **Signatur**, die pygame in
    seine erste Doku-Zeile schreibt, zeigt `help()` auf beiden Seiten -
    bei uns aus einem echten `__text_signature__`.
    """
    import importlib

    aus = []
    for mn, kl in _doku_wege():
        try:
            m = importlib.import_module(
                pg.__name__ + ("." + mn if mn != "(haupt)" else ""))
        except Exception:                            # noqa: BLE001
            continue
        if not kl:
            aus.append((mn, "", getattr(m, "__doc__", None)))
            continue
        try:
            w = getattr(m, kl)
        except Exception:                            # noqa: BLE001
            aus.append((mn, kl, "fehlt"))
            continue
        aus.append((mn, kl, getattr(w, "__doc__", None)))
    # **Die Untergrenze muss unter dem echten Wert liegen** - gemessen
    # sind es 142 (die Module und Klassen, an denen pygame einen Satz
    # hat). Hier stand erst 200 und dann 150, und beide Male warf die
    # Probe auf **beiden** Seiten dieselbe Ausnahme: der Lauf meldete
    # "gleich", ohne einen einzigen Satz verglichen zu haben. Dieselbe
    # Falle wie eine Probe ohne Eintraege, nur eine Ebene weiter - **der
    # Waechter selbst wurde zum Wert.** Zweimal passiert, einmal beim
    # Einbau und einmal, nachdem die Probe geschaerft wurde.
    if len(aus) < 100:
        raise RuntimeError("Dokumentation: nur %d Namen" % len(aus))
    return tuple(aus)


def felder_flaeche(pg):
    """Die numpy-Bruecke fuer Flaechen - nur bei 32 Bit."""
    try:
        import numpy                                # noqa: F401
    except ImportError:
        return "ohne numpy"
    import importlib
    sa = importlib.import_module(pg.__name__ + ".surfarray")
    s = pg.Surface((4, 3), pg.SRCALPHA, 32)
    s.fill((10, 20, 30, 40))
    s.set_at((1, 2), (200, 100, 50, 25))
    a2 = sa.array2d(s)
    a3 = sa.array3d(s)
    aa = sa.array_alpha(s)
    aus = [(a2.shape, str(a2.dtype), int(a2[1, 2])),
           (a3.shape, str(a3.dtype), tuple(int(v) for v in a3[1, 2])),
           (aa.shape, str(aa.dtype), int(aa[1, 2]))]
    p3 = sa.pixels3d(s)
    p3[0, 0] = (1, 2, 3)
    aus.append(("pixels3d schreibt", s.get_at((0, 0))))
    s.set_alpha(None)
    aus.append(("ohne Mischen", int(sa.array_alpha(s)[1, 2])))
    z = sa.make_surface(a3)
    aus.append(("make_surface", z.get_size(), z.get_bitsize(),
                z.get_at((1, 2))))
    aus.append(("colorkey", sa.array_colorkey(s).shape))
    return aus


def felder_klang(pg):
    """Die numpy-Bruecke fuer Klaenge - beim Standardformat `-16`."""
    try:
        import numpy
    except ImportError:
        return "ohne numpy"
    import importlib
    pg.mixer.quit()
    pg.mixer.init(22050, -16, 2, allowedchanges=0)
    sd = importlib.import_module(pg.__name__ + ".sndarray")
    werte = numpy.array([[0, -0x7FFF], [-0x7FFF, 0], [0x7FFF, 0],
                         [0, 0x7FFF]], numpy.int16)
    k = sd.make_sound(werte)
    zurueck = sd.array(k)
    aus = [(zurueck.shape, str(zurueck.dtype),
            [int(v) for v in zurueck.reshape(-1)]),
           ("laenge", round(k.get_length(), 6)),
           ("gleich", bool(numpy.all(zurueck == werte)))]
    s = sd.samples(k)
    aus.append(("samples", s.shape, str(s.dtype)))
    pg.mixer.quit()
    # **Und jetzt das schmale Format.** Ohne diese Zeilen sagte die Probe
    # nur, dass `-16` stimmt - und eine benannte Grenze, die keine Probe
    # ausloest, ist eine Behauptung. Gemessen gibt pygame hier `uint8`,
    # rustgame `int16`; siehe `BEKANNTE_GRENZEN`.
    pg.mixer.init(22050, 8, 1, allowedchanges=0)
    if pg.mixer.get_init()[1] == 8:
        k8 = sd.make_sound(numpy.array([0, 0x0F, 0xF0, 0xFF], numpy.uint8))
        a8 = sd.array(k8)
        aus.append(("size 8", str(a8.dtype), [int(v) for v in a8.reshape(-1)]))
    pg.mixer.quit()
    return aus


# ---------------------------------------------------------------------------
#  system und midi
# ---------------------------------------------------------------------------

def ram_groessenordnung(pg):
    """Nicht die Zahl, sondern ihre **Groessenordnung**.

    Der Wert selbst ist Sache der Maschine, und ein Vergleich waere
    sinnlos. Was sich vergleichen laesst: dass es eine ganze Zahl in
    Megabyte ist und dass beide Engines dieselbe nennen - sie fragen
    dasselbe Betriebssystem.
    """
    r = pg.system.get_total_ram()
    return (type(r).__name__, r == pg.system.get_total_ram(),
            64 <= r < 4_194_304)


def _midi(pg):
    """`pygame.midi` laedt nicht mit `import pygame` mit."""
    import importlib
    importlib.import_module(pg.__name__ + ".midi")
    pg.midi.init()
    return pg.midi


def midi_geraete(pg):
    """Die Geraeteliste - **Form und Inhalt.**

    Welche MIDI-Geraete ein Rechner hat, ist Sache des Rechners - aber
    beide Huellen fragen **denselben** Rechner, so wie bei `sysfont`. Also
    wird verglichen, was sie finden: Anzahl, Nummer fuer Nummer dieselbe
    Auskunft, die Vorgabenummern, und wie `get_device_info` mit Unsinn
    umgeht.

    ## Hier stand eine Grenze, die es nicht gab

    "Die Anzahl selbst ist eine benannte Grenze: pygame zaehlt hier 2,
    rustgame 1" - und darum stand die Zahl nicht in der Probe. In
    `BEKANNTE_GRENZEN` stand dazu aber **nichts**; die Grenze war weder
    gemessen noch benannt. Und es war nicht nur die Zahl: **Nummer 0 war
    ein anderes Geraet** (bei pygame der `Microsoft MIDI Mapper`, hier
    der Wavetable-Synthesizer). Seit `rustgame_mixer::midi` den Mapper in
    PortMidis Reihenfolge fuehrt, ist beides gleich.
    """
    m = _midi(pg)
    n = m.get_count()
    roh = [m.get_device_info(i) for i in range(n)]

    def frag(a):
        try:
            return repr(m.get_device_info(a))
        except Exception as e:                       # noqa: BLE001
            return "%s: %s" % (type(e).__name__, e)
    unsinn = [(repr(a), frag(a)) for a in (
        "x", b"1", None, 1.5, True, -1, n, n + 50, 2 ** 31 - 1, 2 ** 31,
        -2 ** 31 - 1, 10 ** 20)]
    aus = []
    for i in range(n):
        g = m.get_device_info(i)
        aus.append((
            len(g), type(g[0]).__name__, type(g[1]).__name__,
            bool(g[0]), bool(g[1]),
            # Ein Geraet ist Eingang oder Ausgang, nicht beides und nicht
            # keines - das ist bei PortMidi so und bei `midir` auch.
            bool(g[2]) != bool(g[3]),
            g[4],
        ))
    ein, aus_n = m.get_default_input_id(), m.get_default_output_id()
    return (
        # **Als Menge, nicht als Folge.** Wie viele Geraete ein Rechner
        # hat, ist nicht vergleichbar (siehe BEKANNTE_GRENZEN zur Anzahl);
        # welche Form ein Eintrag hat, schon.
        sorted(set(aus)),
        n > 0,
        m.get_device_info(n) is None,
        m.get_device_info(n + 50) is None,
        ein == -1 or 0 <= ein < n,
        aus_n == -1 or 0 <= aus_n < n,
        # Zweimal gefragt, zweimal dasselbe.
        [m.get_device_info(i) for i in range(n)] ==
        [m.get_device_info(i) for i in range(n)],
        n, roh, ein, aus_n, unsinn,
    )


def midi_fehler(pg):
    """Was `Input` und `Output` bei Unsinn sagen - **woertlich**.

    ## Hier stand zwei Zeilen zu viel

    Die erste Fassung fragte zusaetzlich `m._kanal_pruefen(16)` ab - einen
    Namen, den **nur rustgame** hat. Sie fragte ihn mit `hasattr` davor,
    also ohne Fehler; pygame lieferte vier Zeilen, rustgame sechs, und das
    war die eine Abweichung des Laufs. Keine davon lag an rustgame.

    Es ist das fuenfte Mal, dass das Messgeraet selbst daneben lag, und
    das erste Mal in einer Probe, die eine Stunde vorher entstanden war.
    Die Regel daraus: **eine Probe darf nur nach Namen fragen, die pygame
    auch hat** - sonst vergleicht sie rustgame mit rustgame.
    """
    m = _midi(pg)
    aus = []
    for bau, wert in (("Input", -1), ("Output", -1),
                      ("Input", 10_000), ("Output", 10_000)):
        try:
            getattr(m, bau)(wert)
            aus.append((bau, wert, "ok"))
        except Exception as e:                       # noqa: BLE001
            aus.append((bau, wert, type(e).__name__, str(e)))
    return tuple(aus)


def midi_rechnerei(pg):
    """Die Haelfte von `pygame.midi`, die **kein Geraet braucht**.

    Notennummern in Frequenzen und zurueck, Notennamen, und der Umbau von
    MIDI-Nachrichten in `pygame`-Ereignisse. Das ist reine Rechnerei und
    muss auf den Wert stimmen - hier ueber den ganzen MIDI-Bereich.
    """
    m = _midi(pg)
    freq = [(n, m.midi_to_frequency(n)) for n in range(0, 128, 7)]
    zurueck = [(f, m.frequency_to_midi(f))
               for f in (8.0, 27.5, 110.0, 440.0, 441.0, 1000.0, 12543.0)]
    namen = [(n, m.midi_to_ansi_note(n)) for n in range(0, 128, 5)]
    # `midis2events` nimmt die Form, die `Input.read` herausgibt.
    roh = [[[0x90, 60, 100, 0], 1234], [[0x80, 60, 0, 0], 2345],
           [[0xB0, 7, 64, 0], 3456]]
    ereignisse = [(e.type, sorted(e.__dict__.items()))
                  for e in m.midis2events(roh, 0)]
    return (freq, zurueck, namen, ereignisse)


def _kamera(pg):
    """`pygame.camera` laedt ebenfalls nicht mit."""
    import importlib
    importlib.import_module(pg.__name__ + ".camera")
    pg.camera.init()
    return pg.camera


def kamera_auskunft(pg):
    """Die Geraeteliste - **Form und Typ, nicht der Inhalt.**

    Gemessen gibt pygame hier `['USB  Live camera']`; welche Kamera an
    einem Rechner haengt, ist aber nicht die Frage. Die Frage ist, dass
    beide Engines dasselbe Geraet finden - und das tun sie: die Liste wird
    hier mitverglichen, denn beide fragen denselben Treiber.
    """
    c = _kamera(pg)
    liste = c.list_cameras()
    return (
        type(liste).__name__,
        sorted(liste),
        all(isinstance(x, str) for x in liste),
        # Zweimal gefragt, zweimal dasselbe.
        c.list_cameras() == liste,
    )


def kamera_farbraum(pg):
    """`colorspace` - die einzige Funktion, die ohne Geraet rechnet.

    Vier Farben durch drei Raeume, Pixel fuer Pixel verglichen.
    """
    aus = []
    c = _kamera(pg)
    for raum in ("RGB", "YUV", "HSV"):
        f = pg.Surface((2, 2), 0, 32)
        f.set_at((0, 0), (255, 0, 0))
        f.set_at((1, 0), (0, 255, 0))
        f.set_at((0, 1), (0, 0, 255))
        f.set_at((1, 1), (17, 200, 99))
        z = c.colorspace(f, raum)
        aus.append((raum, [tuple(z.get_at((x, y)))
                           for y in range(2) for x in range(2)]))
        # Mit Zielflaeche: dieselbe Flaeche kommt zurueck.
        ziel = pg.Surface((2, 2), 0, 32)
        zurueck = c.colorspace(f, raum, ziel)
        aus.append((raum, zurueck is ziel,
                    [tuple(ziel.get_at((x, y)))
                     for y in range(2) for x in range(2)]))
    return tuple(aus)


def kamera_fehler(pg):
    """Was bei Unsinn herauskommt - **woertlich und mit der richtigen Sorte.**

    Ein unbekannter Name ist ein `ValueError`, eine Nummer ein
    `TypeError`, ein falscher Farbraum ein `ValueError`. Drei verschiedene
    Ausnahmen, und keine davon braucht ein Geraet.
    """
    c = _kamera(pg)
    aus = []
    for was, ruf in (
        ("Nummer", lambda: c.Camera(0)),
        ("None", lambda: c.Camera(None)),
        ("unbekannt", lambda: c.Camera("gibtesnicht-xyzzy")),
        ("Farbraum", lambda: c.colorspace(pg.Surface((2, 2)), "quatsch")),
    ):
        try:
            ruf()
            aus.append((was, "ok"))
        except Exception as e:                       # noqa: BLE001
            aus.append((was, type(e).__name__, str(e)))
    return tuple(aus)


def kamera_rueckwaende(pg):
    """Bekannte Grenze, siehe BEKANNTE_GRENZEN: pygame hat ein zweites
    Kamerastueck ueber OpenCV, rustgame nicht."""
    return _kamera(pg).get_backends()


def kamera_bild(pg):
    """Ein echtes Bild vom Geraet - **Form, Tiefe, Masken, nicht schwarz.**

    Die Pixel selbst lassen sich nicht vergleichen: zwischen den beiden
    Aufnahmen liegt eine Sekunde, und die Kamera sieht dann etwas anderes.
    Vergleichbar ist alles andere, und davon war eines schon falsch -
    `frombytes` hatte eine 24-Bit-Flaeche gebaut, wo pygame eine mit 32
    gibt. Die Masken waren dabei gleich; **nur die Tiefe verriet es.**

    Ohne Kamera gibt die Probe einen festen Satz zurueck und vergleicht
    damit immer noch etwas: dass beide Engines sie nicht finden.
    """
    c = _kamera(pg)
    liste = c.list_cameras()
    if not liste:
        return "keine Kamera"
    k = c.Camera(liste[0], (320, 240), "RGB")
    # **Vor dem Start der Wunsch, danach die Wirklichkeit.** Das ist
    # gemessen und hat beim ersten Mal in die Irre gefuehrt.
    vorher = k.get_size()
    k.start()
    nachher = k.get_size()
    b = k.get_image()
    # **Kein Helligkeitsvergleich mehr.** Hier stand ein `hell = any(...)`
    # ueber jedes siebzehnte Pixel - ob das Bild irgendwo nicht schwarz
    # ist. Der Wert haengt an der **Aufwaermzeit der echten Kamera**: das
    # erste Bild nach `start()` ist oft noch schwarz, und welche Engine
    # zuerst laeuft, entscheidet dann. Zwei Laeufe, zwei Antworten - eine
    # Probe, deren Sollwert vom Licht im Zimmer abhaengt, misst nichts.
    #
    # Dass ueberhaupt ein Bild kommt, sagt `b.get_size()` weiter unten;
    # dass es das richtige Format hat, `get_bitsize()` und `get_masks()`.
    ziel = pg.Surface(nachher)
    gleich = k.get_image(ziel) is ziel
    try:
        k.get_image(pg.Surface((7, 7)))
        schief = "ok"
    except Exception as e:                           # noqa: BLE001
        schief = (type(e).__name__, str(e))
    k.stop()
    k.stop()
    try:
        k.query_image()
        nach_stop = "ok"
    except Exception as e:                           # noqa: BLE001
        nach_stop = (type(e).__name__, str(e))
    return (vorher, nachher, b.get_size(), b.get_bitsize(), b.get_masks(),
            gleich, schief, nach_stop)


def _freetype(pg):
    import importlib
    ft = importlib.import_module(pg.__name__ + ".freetype")
    ft.init()
    return ft


def _schriftordner():
    """Wo pygames Testschriften liegen - immer die von **echtem** pygame."""
    import os as _os
    import pygame.tests as _pt
    return _os.path.join(_os.path.dirname(_pt.__file__), "fixtures", "fonts")


def ft_groesse(pg):
    """Die Groesse, in der `pygame.freetype` die Standardschrift laedt.

    ## Was hier schiefging

    `pygame.font` rechnet die Groesse der **mitgelieferten** Schrift mit
    `0.6875` klein - so steht es in pygames `font.c`, und `Font(None, 16)`
    ist deshalb freesansbold in Groesse 11. `pygame.freetype` tut das
    **nicht**, und rustgame tat es doch, weil beide Wege dieselbe
    Ladefunktion riefen.

    Der Beweis, dass pygame es so meint, steht in seinem eigenen
    `ftfont.py`: dort wird `resolution = int(72 * 0.6875)` gesetzt, nur um
    die kleine Groesse nachzubilden. Bei `ftfont` wirkte der Faktor hier
    also **doppelt** - `0.6875 ** 2 = 0.47`.

    Verglichen werden die **Breiten** und die Zeilenmasse; die Hoehen
    haengen an der bekannten Grenze `font.metrics glyphenbox` (runde
    Buchstaben ragen bei fontdue ein Pixel unter die Grundlinie).
    """
    ft = _freetype(pg)
    f = ft.Font(None)
    # **Einzelne Zeichen, nicht Zeichenketten.** Ueber eine Zeile summiert
    # sich die Glyphenbox-Grenze (bis zu ein Pixel je Zeichen); das ist
    # eine eigene Probe, `freetype.Textbreite`.
    breiten = [(c, f.get_rect(c, size=24).x, f.get_rect(c, size=24).width)
               for c in "AMiWg.oxHn"]
    masse = [(g, f.get_sized_ascender(g), f.get_sized_descender(g),
              f.get_sized_height(g), f.get_sized_glyph_height(g))
             for g in (8, 10, 12, 16, 24, 32, 48, 64)]
    ff = __import__("importlib").import_module(pg.__name__ + ".ftfont")
    ff.init()
    ftf = ff.Font(None, 24)
    # Die Auflösung ist der Kern der Sache: 49 statt 72, und **einmal**
    # statt zweimal angewandt.
    return (breiten, masse, ftf.resolution, f.resolution)


def ft_textbreite(pg):
    """Die Breite einer **Zeile** in `pygame.freetype`.

    **Bis Runde 19 eine benannte Grenze**, und ihre Begruendung war zweimal
    falsch: erst "die Glyphenbox-Grenze summiert sich", dann (Runde 18)
    "die Schreibmarke in `freitext.rs`". Wahr war: die Huelle setzte
    `freetype`-Text mit **SDL_ttfs** Zeilenaufbau und schnitt ihn zu.
    pygames `ft_layout.c` setzt selbst - Stift um `horiAdvance`, Bild bei
    `Stift + bitmap_left`, Kasten aus Tinte und abschliessendem Leerzeichen
    -, und so steht es jetzt in `rustgame_font::Font::ft_zeile`.
    `"Hallo Welt"` ist 116 breit, nicht 122.
    """
    ft = _freetype(pg)
    f = ft.Font(None)
    return [(t, f.get_rect(t, size=24).width)
            for t in ("Music", "Hallo Welt", "iiiii", "WWWWW")]


def ft_stile(pg):
    """`pygame.freetype` in allen vier Stilen, einfarbig, unterschnitten
    und gepolstert - Bild, Rechteck und `render_raw`.

    Bis Runde 19 war hier **kein einziges** Bild gleich, sobald ein Stil
    gesetzt war: `STYLE_STRONG` ist bei pygame `FT_Outline_Embolden` am
    Umriss (nicht SDL_ttfs Verschmieren), `STYLE_OBLIQUE` schert mit 0,22
    (nicht 0,207), `STYLE_WIDE` ist `FT_Bitmap_Embolden`, und der
    Unterstrich ein Rechteck mit Bruchkanten (`__fill_glyph_RGB4`).
    """
    import hashlib
    ft = _freetype(pg)
    aus = []
    texte = ["Hallo Welt", "AVATAR To", "iiiii WWWWW", "g, j; Q!", " a ", "x"]
    stile = [ft.STYLE_NORMAL, ft.STYLE_STRONG, ft.STYLE_OBLIQUE, ft.STYLE_WIDE,
             ft.STYLE_UNDERLINE, ft.STYLE_STRONG | ft.STYLE_UNDERLINE,
             ft.STYLE_OBLIQUE | ft.STYLE_WIDE]
    import os as _os
    ordner = _os.path.join(_os.environ.get("WINDIR", r"C:\Windows"), "Fonts")
    schriften = [None] + [_os.path.join(ordner, n) for n in
                          ("arial.ttf", "consola.ttf", "georgia.ttf")]
    for pfad in schriften:
        # Ein Ladefehler waere auf beiden Seiten derselbe - und die Probe
        # meldete "gleich", ohne gemessen zu haben. So ist es beim ersten
        # Lauf passiert (die Pfade waren beim Schreiben zerlegt).
        try:
            f = ft.Font(pfad)
        except Exception as e:  # noqa: BLE001
            raise RuntimeError("freetype Stile: %r laedt nicht (%s)" % (pfad, e))
        for groesse in (11, 16, 24, 37):
            for aa in (True, False):
                f.antialiased = aa
                for kern in (False, True):
                    f.kerning = kern
                    for stil in stile:
                        # **pygame beschaedigt hier seinen Heap**: Arial,
                        # geglaettet, `STYLE_UNDERLINE` allein - nach einigen
                        # Dutzend Aufrufen `0xc0000374` und Absturz, mal
                        # frueher, mal spaeter. Einzeln in frischen Prozessen
                        # gefahren sind alle 48 Faelle Pixel fuer Pixel gleich
                        # (Runde 19); in dieser Probe wuerde der Absturz den
                        # ganzen Lauf mitnehmen.
                        if (pfad or "").endswith("arial.ttf") and aa and stil == ft.STYLE_UNDERLINE:
                            continue
                        for t in texte:
                            s, r = f.render(t, (255, 255, 255), size=groesse, style=stil)
                            roh, groe = f.render_raw(t, size=groesse, style=stil)
                            aus.append((pfad, groesse, aa, kern, stil, t, tuple(r),
                                        s.get_size(), s.get_bitsize(), hashlib.sha256(
                                            pg.image.tobytes(s, "RGBA")).hexdigest()[:12],
                                        groe, hashlib.sha256(roh).hexdigest()[:12],
                                        tuple(f.get_rect(t, size=groesse, style=stil))))
        f.antialiased = True
        f.kerning = False
        f.pad = True
        for t in texte:
            aus.append((pfad, "pad", t, tuple(f.get_rect(t, size=20))))
    if len(aus) < 2500:
        raise RuntimeError("freetype Stile: nur %d Eintraege" % len(aus))
    return aus


def _ft_bild(pg, s):
    import hashlib
    return hashlib.sha256(pg.image.tobytes(s, "RGBA")).hexdigest()[:12]


def _ft_versuch(f):
    try:
        return f()
    except Exception as e:  # noqa: BLE001
        return "%s: %s" % (type(e).__name__, e)


def ft_lage(pg):
    """`pygame.freetype` **gedreht**, **senkrecht** und in **Bruchgroessen**
    - dazu die Saetze, mit denen `_PGFT_BuildRenderMode` ablehnt.

    Bis Runde 19 gingen alle drei den alten Weg ueber SDL_ttfs Zeile:
    gedreht 0 von 24 Bildern gleich, senkrecht 0 von 3, Bruchgroessen 0
    von 8. Jetzt setzt `ftzeile.rs` sie wie `ft_layout.c` (Drehung als
    Matrix vor dem Rastern, senkrechte Lager aus `compute_glyph_metrics`,
    die Nenngroesse in 26.6 nach `FT_REQUEST_WIDTH`). Und die Saetze fuer
    gedrehten Unterstrich, gedrehtes `wide` und senkrechten Unterstrich
    fehlten ganz - dort wurde gezeichnet, wo pygame `ValueError` gibt.
    """
    import os as _os
    ft = _freetype(pg)
    ordner = _os.path.join(_os.environ.get("WINDIR", r"C:\Windows"), "Fonts")
    schriften = [None] + [_os.path.join(ordner, n) for n in ("consola.ttf", "segoeui.ttf")]
    aus = []
    for pfad in schriften:
        for res in (72, 96):
            try:
                f = ft.Font(pfad, resolution=res)
            except Exception as e:  # noqa: BLE001
                raise RuntimeError("freetype Lage: %r laedt nicht (%s)" % (pfad, e))
            f.kerning = True
            for aa in (True, False):
                f.antialiased = aa
                # gedreht
                for grad in (15, 90, 180, 300, -45):
                    for stil in (ft.STYLE_NORMAL, ft.STYLE_STRONG, ft.STYLE_OBLIQUE):
                        for t in ("Hallo Welt", "AV"):
                            s, r = f.render(t, (255, 255, 255), size=20, style=stil, rotation=grad)
                            aus.append((pfad, res, aa, "dreh", grad, stil, t, tuple(r),
                                        s.get_size(), _ft_bild(pg, s)))
                # senkrecht
                f.vertical = True
                for stil in (ft.STYLE_NORMAL, ft.STYLE_STRONG, ft.STYLE_OBLIQUE, ft.STYLE_WIDE):
                    for t in ("Hallo", "AV g", "", " "):
                        s, r = f.render(t, (255, 255, 255), size=17, style=stil)
                        aus.append((pfad, res, aa, "senk", stil, t, tuple(r), s.get_size(),
                                    _ft_bild(pg, s)))
                s, r = f.render("Ab", (255, 255, 255), size=17, rotation=30)
                aus.append((pfad, res, aa, "senk+dreh", tuple(r), _ft_bild(pg, s)))
                f.vertical = False
                # Bruchgroessen
                for g in (9.5, 12.75, 20.5, 31.3):
                    for stil in (ft.STYLE_NORMAL, ft.STYLE_WIDE, ft.STYLE_UNDERLINE):
                        s, r = f.render("AVa Wg!", (255, 255, 255), size=g, style=stil)
                        aus.append((pfad, res, aa, "bruch", g, stil, tuple(r), s.get_size(),
                                    _ft_bild(pg, s)))
            f.antialiased = True
            for g in (9.5, 12.75, 20.5, 31.3, 17):
                aus.append((pfad, res, "sized", g, f.get_sized_ascender(g),
                            f.get_sized_descender(g), f.get_sized_height(g),
                            f.get_sized_glyph_height(g), tuple(f.get_rect("", size=g))))
    # Was `_PGFT_BuildRenderMode` ablehnt - auch ueber die Einstellungen
    # der Schrift und bei `get_metrics`.
    f = ft.Font(None, 20)
    for name, kw in (("dreh+unter", dict(rotation=15, style=ft.STYLE_UNDERLINE)),
                     ("dreh+breit", dict(rotation=15, style=ft.STYLE_WIDE)),
                     ("dreh360+breit", dict(rotation=360, style=ft.STYLE_WIDE))):
        aus.append((name, _ft_versuch(lambda: tuple(f.get_rect("Ab", **kw))),
                    _ft_versuch(lambda: f.render_raw("Ab", **kw)[1])))
    f.pad = True
    aus.append(("dreh+pad", _ft_versuch(lambda: tuple(f.get_rect("Ab", rotation=10)))))
    f.pad = False
    f.vertical = True
    aus.append(("senk+unter", _ft_versuch(lambda: tuple(f.get_rect("Ab", style=ft.STYLE_UNDERLINE)))))
    f.vertical = False
    f.underline = True
    f.rotation = 20
    aus.append(("metrics unter+dreh", _ft_versuch(lambda: f.get_metrics("Ab"))))
    if len(aus) < 700:
        raise RuntimeError("freetype Lage: nur %d Eintraege" % len(aus))
    return aus


def _ft_schriften(*namen):
    """Pfade zu Schriften: `None` (pygames eigene), Namen aus pygames
    Testdaten (`tests/fixtures/fonts`) oder aus dem Windows-Schriftordner.
    Was fehlt, wirft - eine Probe ohne ihre Schrift misst nichts."""
    import os as _os
    fix = _os.path.join(_os.path.dirname(_echt_pygame_datei()), "tests", "fixtures", "fonts")
    win = _os.path.join(_os.environ.get("WINDIR", r"C:\Windows"), "Fonts")
    aus = []
    for n in namen:
        if n is None:
            aus.append(None)
            continue
        for ordner in (fix, win):
            p = _os.path.join(ordner, n)
            if _os.path.exists(p):
                aus.append(p)
                break
        else:
            raise RuntimeError("Schrift %r fehlt" % n)
    return aus


def _ft_bild_und_kasten(pg, f, t, **kw):
    return _ft_versuch(lambda: (lambda s, r: (tuple(r), s.get_size(), _ft_bild(pg, s)))(
        *f.render(t, (255, 255, 255), **kw)))


def ft_zwei_achsen(pg):
    """Groessen mit **zwei Achsen** (`size=(24, 12)`).

    Bis Runde 19 eine benannte Grenze: FreeType hintet sie mit dem
    **gestreckten** TrueType-Interpreter (`x_ppem != y_ppem`: die CVT mit
    der groesseren Skala, jedes Lesen und Schreiben mal `Current_Ratio()`
    aus dem Projektionsvektor, `MPPEM` gestreckt, `MD`/`MDRP`/`IP` mit zwei
    Skalen), skrifa kannte eine. Gemessen 0 von 18 Bildern gleich. Seit
    Runde 20 steckt der gestreckte Interpreter im skrifa-Abzug
    (`HintingInstance::new_gestreckt`, `Current_Ratio` samt FreeTypes
    CORDIC-`FT_Vector_Length`), der Zeilensatz rechnet senkrecht mit
    `y_scale` und waagerecht mit `x_scale`, CFF setzt sich aus dem `x` in
    der Breite und dem `y` in der Hoehe zusammen (`cf2_glyphpath_hintPoint`),
    und eine BDF-Schrift sieht nur die Hoehe (`BDF_Size_Request`).
    Breit gemessen: 8760 Faelle an zwoelf Windows-Schriften und 8176 an
    pygames Testschriften, alle gleich.
    """
    ft = _freetype(pg)
    aus = []
    schriften = _ft_schriften(None, "arial.ttf", "times.ttf", "cour.ttf", "calibri.ttf",
                              "georgia.ttf", "comic.ttf", "PyGameMono.otf", "test_fixed.otf",
                              "test_sans.ttf", "PyGameMono-8.bdf", "PyGameMono-18-75dpi.bdf")
    groessen = ((24, 12), (12, 24), (20, 30), (16, 10), (9, 14), (13.5, 20), (11, 11.4),
                (31, 7), (16, 8), (19, 19))
    for pfad in schriften:
        f = ft.Font(pfad)
        for g in groessen:
            aus.append((pfad, g, "kopf", _ft_versuch(lambda: (
                f.get_sized_ascender(g), f.get_sized_descender(g), f.get_sized_height(g),
                f.get_sized_glyph_height(g)))))
            for t in ("Hallo Welt", "AVg!\u00c4"):
                for aa in (True, False):
                    f.antialiased = aa
                    for st in (0, 3, 4, 8):
                        aus.append((pfad, g, t, aa, st, _ft_bild_und_kasten(pg, f, t, size=g, style=st)))
                f.antialiased = True
                aus.append((pfad, g, t, "rot", _ft_bild_und_kasten(pg, f, t, size=g, rotation=30)))
                f.vertical = True
                aus.append((pfad, g, t, "senk", _ft_bild_und_kasten(pg, f, t, size=g)))
                f.vertical = False
                f.kerning = True
                aus.append((pfad, g, t, "kern", _ft_bild_und_kasten(pg, f, t, size=g)))
                f.kerning = False
                aus.append((pfad, g, t, "rect", _ft_versuch(lambda: tuple(f.get_rect(t, size=g)))))
                aus.append((pfad, g, t, "metr", _ft_versuch(lambda: f.get_metrics(t, size=g))))
    if len(aus) < 3000:
        raise RuntimeError("freetype zwei Achsen: nur %d Eintraege" % len(aus))
    return aus


def ft_metriken(pg):
    """`freetype.Font.get_metrics` mit Stil, Drehung, Glaettung und Groesse.

    pygame laedt jedes Zeichen wie beim Satz (`_PGFT_GetMetrics` ruft
    `_PGFT_LoadGlyph`) und gibt die Kanten des Bildes und den gedrehten
    Vorschub heraus. Bis Runde 20 kamen die Masse hier aus der
    Schriftkiste und stimmten nur im Grundfall - an `Font(None, 24)` und
    `"Ag"` waren `strong`, `wide`, `oblique`, einfarbig, 30 Grad, `20.5`
    und `(24, 12)` jedes Mal anders, und keine Probe fragte danach.
    Gemessen in Runde 20: 4600 Fragen, alle gleich.
    """
    ft = _freetype(pg)
    aus = []
    text = "Ag\u00c4 .|\x01\u4e2d"
    for pfad in _ft_schriften(None, "arial.ttf", "cour.ttf", "calibri.ttf", "PyGameMono.otf",
                              "test_fixed.otf", "PyGameMono-8.bdf", "PyGameMono-18-75dpi.bdf"):
        f = ft.Font(pfad)
        bitmap = pfad is not None and pfad.endswith(".bdf")
        for g in ((8, 8.4, 19, (16, 8)) if bitmap else (7, 12, 20.5, (24, 12))):
            for aa in (True, False):
                f.antialiased = aa
                for stil in ("std", "strong", "oblique", "wide", "underline"):
                    for rot in ((0,) if bitmap else (0, 30, 200)):
                        for senk in (False, True):
                            f.vertical = senk
                            try:
                                f.rotation = rot
                                if stil != "std":
                                    setattr(f, stil, True)
                            except Exception as e:  # noqa: BLE001
                                aus.append((pfad, g, aa, stil, rot, senk, type(e).__name__))
                                continue
                            aus.append((pfad, g, aa, stil, rot, senk,
                                        _ft_versuch(lambda: f.get_metrics(text, size=g))))
                            if stil != "std":
                                setattr(f, stil, False)
                            f.rotation = 0
                f.vertical = False
    if len(aus) < 1500:
        raise RuntimeError("freetype get_metrics: nur %d Eintraege" % len(aus))
    return aus


def ft_bitmapgroessen(pg):
    """Welche Groessen eine **Bitmapschrift** (BDF) annimmt.

    pygame und FreeType entscheiden das zusammen: `_PGFT_GetFontSized`
    sucht fuer eine Zahl den Strike mit demselben `FX6_ROUND` von
    `bsize->size` und fragt dann nach seinen Pixeln, sonst nach der Zahl
    selbst; `BDF_Size_Request` sieht nur die **Hoehe** an. Also gehen bei
    `PyGameMono-8.bdf` auch `8.4`, `7.6` und `(16, 8)`, nicht aber `9`
    oder `(8, 16)` - und der Satz ist `Failed to resize font: invalid
    pixel size`. Bis Runde 20 gingen hier nur die ganzen Groessen aus
    `get_sizes()`, und der Satz war erfunden.
    """
    ft = _freetype(pg)
    aus = []
    groessen = (8, 8.4, 7.6, 9, 42, (8, 8), (16, 8), (8, 16), (1, 8), (8, 8.4), 18, 19, 18.4,
                19.4, (19, 19), (18, 19), (40, 19), (18, 18), 25, (25, 25), 6)
    for pfad in _ft_schriften("PyGameMono-8.bdf", "PyGameMono-18-75dpi.bdf",
                              "PyGameMono-18-100dpi.bdf"):
        for res in (0, 96, 100):
            f = ft.Font(pfad, resolution=res)
            for g in groessen:
                aus.append((pfad, res, g, _ft_versuch(lambda: (
                    tuple(f.get_rect("Ag", size=g)), f.get_sized_height(g),
                    f.get_sized_ascender(g))), _ft_bild_und_kasten(pg, f, "Ag", size=g)))
            for g in ((16, 8), 8.4, (19, 19), 19.4):
                try:
                    f.size = g
                except Exception as e:  # noqa: BLE001
                    aus.append((pfad, res, "size=", g, type(e).__name__))
                    continue
                aus.append((pfad, res, "size=", g, _ft_bild_und_kasten(pg, f, "Ag"),
                            _ft_versuch(lambda: f.get_metrics("A"))))
    # `_PGFT_SetError` merkt sich den letzten Satz - `get_error()` liest ihn.
    aus.append(("get_error", ft.get_error()))
    if len(aus) < 200:
        raise RuntimeError("freetype Bitmapgroessen: nur %d Eintraege" % len(aus))
    return aus


def font_strikes_symbol(pg):
    """`pygame.font` an drei Stellen, die in Runde 20 der Durchlauf ueber
    **alle** Schriften des Systems fand (`tools/alle_schriften.py`):

    * **Strikes**: kommt eine Glyphe aus einem eingebetteten Bild, steht
      ihr Vorschub in dessen sbit-Massen, nicht in `hmtx` - Calibri hat
      Strikes bei 12, 13, 15, 16, 17 und 19 (`metrics("a")` bei 13: 7, hier
      stand 6).
    * **Symbolschriften**: SDL_ttf waehlt die Zeichentabelle (3,0), und
      `metrics("\uf041")` findet dort das `A` von `symbol.ttf`.
    * **Ein scheiterndes CV-Programm** (`monbaiti.ttf`: `CALL` einer nie
      definierten Funktion): FreeType lehnt dann jede gehintete Glyphe ab,
      `size()` sagt `Couldn't find glyph`, `render()` `Text has zero
      width`, `metrics()` gibt `None`.
    """
    aus = []
    for name, groessen in (("calibri.ttf", (11, 12, 13, 15, 16, 17, 19, 20)),
                           ("cour.ttf", (9, 13, 21, 22)), ("symbol.ttf", (13, 20)),
                           ("WINGDING.TTF", (13,)), ("monbaiti.ttf", (9, 13))):
        pfad = _ft_schriften(name)[0]
        for g in groessen:
            f = pg.font.Font(pfad, g)
            if name == "symbol.ttf":
                # Nur die Zeichentabelle: die Breite eines `υ` (U+F075) ist
                # bei pygame 8, hier 7 - ein Rest aus `alle_schriften.py`,
                # benannt in `mycache/GRENZEN.md`.
                aus.append((name, g, _ft_versuch(lambda: f.metrics("A"))))
                continue
            for t in ("Hamburg 0123", "aÄi", "A"):
                aus.append((name, g, t, _ft_versuch(lambda: f.size(t)),
                            _ft_versuch(lambda: f.metrics(t))))
                for aa in (True, False):
                    aus.append((name, g, t, aa, _ft_versuch(lambda: (lambda s: (
                        s.get_size(), _ft_bild(pg, s)))(f.render(t, aa, (255, 255, 255))))))
            aus.append((name, g, "leer", _ft_versuch(lambda: f.size("")),
                        _ft_versuch(lambda: f.render("", True, (255, 255, 255)).get_size())))
    if len(aus) < 130:
        raise RuntimeError("font Strikes und Symbol: nur %d Eintraege" % len(aus))
    return aus


def _r21_quelle(pg):
    """Eine Flaeche mit Alpha, deren Pixel jede Mischstufe treffen."""
    s = pg.Surface((64, 64), pg.SRCALPHA, 32)
    for y in range(64):
        for x in range(64):
            s.set_at((x, y), ((x * 4) % 256, (y * 4) % 256, (x * y) % 256, (x * 4 + y) % 256))
    return s


def blit_unter_32(pg):
    """Alpha-Blit auf ein Ziel mit **weniger als 32 Bit** - Runde 21.

    Jede Probe davor blittete auf 32 Bit (Bildschirm, `convert_alpha`,
    `SRCALPHA`). Gefunden bei der Frage "ist alles da", an einer
    24-Bit-Flaeche - und eine solche liefert jedes `image.load` einer
    JPEG-Datei. Je Zieltiefe vier Arten; die Abweichung steht als Grenze
    mit Abdruck, die gleichen Faelle wachen mit.
    """
    import hashlib

    aus = []
    for tiefe in (24, 16, 15, 8):
        for art in ("pixelalpha", "flaechenalpha", "schluessel", "add"):
            s = _r21_quelle(pg)
            if art == "flaechenalpha":
                q = pg.Surface((64, 64), 0, 32)
                q.blit(s, (0, 0))
                s = q
                s.set_alpha(137)
            elif art == "schluessel":
                q = pg.Surface((64, 64), 0, 32)
                q.blit(s, (0, 0))
                s = q
                s.set_colorkey((0, 0, 0))
                s.set_alpha(100)
            d = pg.Surface((64, 64), 0, tiefe)
            if tiefe == 8:
                d.set_palette([(i, i, i) for i in range(256)])
            d.fill((90, 160, 30))
            if art == "add":
                d.blit(s, (0, 0), special_flags=pg.BLEND_RGBA_ADD)
            else:
                d.blit(s, (0, 0))
            aus.append((tiefe, art, hashlib.sha256(pg.image.tobytes(d, "RGBA")).hexdigest()[:12]))
    return aus


def jpeg_lesen(pg):
    """`image.load` einer JPEG-Datei - Runde 21.

    SDL_image liest mit libjpeg-turbo, rustgame mit `zune-jpeg`. Groesse,
    Tiefe und Flaggen stimmten immer; die **Pixel** hatte keine Probe
    gefragt. Gemessen an 41 Dateien: 39 verschieden, um 2 bis 61 Stufen.
    Die Dateien hier sind pygames eigene.
    """
    import hashlib
    import os

    try:
        import pygame as _echt
        wurzel = os.path.dirname(_echt.__file__)
    except Exception:                                # noqa: BLE001
        return "kein pygame daneben"
    aus = []
    for teil in (("examples", "data", "alien1.jpg"), ("examples", "data", "red.jpg"),
                 ("docs", "generated", "_images", "intro_blade.jpg"),
                 ("docs", "generated", "_images", "camera_rgb.jpg")):
        pfad = os.path.join(wurzel, *teil)
        if not os.path.exists(pfad):
            aus.append((teil[-1], "fehlt"))
            continue
        s = pg.image.load(pfad)
        aus.append((teil[-1], s.get_size(), s.get_bitsize(), s.get_flags(),
                    hashlib.sha256(pg.image.tobytes(s, "RGB")).hexdigest()[:12]))
    return aus


def bild_schreiben(pg):
    """`image.save` als JPEG und PNG - Runde 21.

    Der PNG-Rundlauf war gemessen (`image.png rundlauf`), die **Datei**
    nie. PNG: dieselben Pixel, dieselben Bloecke, andere Kompression.
    JPEG: pygame ruft `IMG_SaveJPG(.., 85)` ueber libjpeg-turbo, rustgame
    kodiert mit `jpeg-encoder` - hier stand bis Runde 21 die Guete 75.
    Je Format: Laenge und Abdruck der Datei, und die Pixel, die dieselbe
    Engine daraus wieder liest.
    """
    import hashlib
    import tempfile

    s = _r21_quelle(pg)
    q = pg.Surface((64, 64), 0, 24)
    q.fill((200, 120, 40))
    pg.draw.circle(q, (20, 60, 220), (30, 34), 21)
    aus = []
    for name, flaeche in (("rgba", s), ("rgb24", q)):
        for endung in ("png", "jpg"):
            pfad = os.path.join(tempfile.gettempdir(), "rustgame_r21.%s" % endung)
            pg.image.save(flaeche, pfad)
            with open(pfad, "rb") as f:
                daten = f.read()
            zurueck = pg.image.load(pfad)
            aus.append((name, endung, len(daten), hashlib.sha256(daten).hexdigest()[:12],
                        hashlib.sha256(pg.image.tobytes(zurueck, "RGBA")).hexdigest()[:12]))
    return aus


def ft_kern_tabelle(pg):
    """`freetype` mit `kerning = True` - FreeType liest nur die alte
    `kern`-Tabelle, und es liest ihre **Teiltabellenlaenge als 16 Bit**.

    Calibri hat 26.706 Paare (160.250 Bytes) in einer Teiltabelle, deren
    Laengenfeld deshalb 29.178 sagt; `handle broken count` in `ttkern.c`
    kuerzt die Liste auf 4860 Paare. ttf-parser las alle, und jede Zeile
    in Calibri war hier mit Kerning schmaler als bei pygame (`"AVAWAT"`
    bei 24: 83 gegen 85). Jetzt sind `tt_face_load_kern` und
    `tt_face_get_kerning` uebertragen.
    """
    ft = _freetype(pg)
    aus = []
    for pfad in _ft_schriften("calibri.ttf", "arial.ttf", "times.ttf", "verdana.ttf",
                              "segoeui.ttf", "georgia.ttf", None):
        f = ft.Font(pfad)
        f.kerning = True
        for g in (12, 24, 13.5, (24, 12)):
            for t in ("AVAWAT", "Typ", "To Yo", "LT", "Wa, ye."):
                aus.append((pfad, g, t, _ft_versuch(lambda: tuple(f.get_rect(t, size=g)))))
            aus.append((pfad, g, _ft_bild_und_kasten(pg, f, "AVAWAT To", size=g)))
    if len(aus) < 150:
        raise RuntimeError("freetype Kerning: nur %d Eintraege" % len(aus))
    return aus


def ft_kleine_groessen(pg):
    """Kleine Groessen, bei denen das CV-Programm (`prep`) das Hinting
    **abschaltet** (`INSTCTRL` Bit 1), und Bruchgroessen an Schriften mit
    Strikes.

    FreeType laedt dann ohne Anweisungen, aber mit den Skalen des Laders
    (`hinted_metrics` - bei `head.flags` Bit 3 die der gerundeten Groesse:
    `times.ttf` sieht bei 5.5 und 6.25 gleich aus), und es setzt die
    Dropout-Flaggen trotzdem aus `GS.scan_control` (`TT_Load_Glyph` fragt
    sein eigenes `load_flags`). Bis Runde 20 waren hier 92 von 200
    Bildern anders. Dazu `get_sized_*`: bei `cour.ttf` (Strikes 4 bis 21)
    gewinnt der Strike die Groessenanfrage, sobald Breite und Hoehe auf
    seine Pixel runden - `11.4` hat dann die Hoehe 12, nicht 13.
    """
    ft = _freetype(pg)
    aus = []
    for pfad in _ft_schriften("times.ttf", "comic.ttf", "cour.ttf", "arial.ttf", "georgia.ttf",
                              "tahoma.ttf", "calibri.ttf"):
        f = ft.Font(pfad)
        for g in (5, 5.5, 6.25, 6.75, 7, 7.4, 8, 8.3, (8, 6), (6, 8)):
            for aa in (True, False):
                f.antialiased = aa
                for t in ("Hallo Welt", "\u00c4g"):
                    aus.append((pfad, g, aa, t, _ft_bild_und_kasten(pg, f, t, size=g)))
        f.antialiased = True
        # Senkrecht auf einem Strike: kleine sbit-Masse haben keinen
        # senkrechten Vorschub, FreeType nimmt dann `linearVertAdvance`
        # mal der Skala des Strikes ("sanity checks" in `TT_Load_Glyph`).
        # Calibri (Strikes 12 bis 19) war bei 19 13 statt 85 Zeilen hoch.
        if pfad.lower().endswith(("cour.ttf", "calibri.ttf")):
            for strikes in (True, False):
                f.use_bitmap_strikes = strikes
                f.vertical = True
                for g in (12, 13, 15, 16, 17, 19, 14, (19, 19)):
                    aus.append((pfad, g, "senk", strikes, _ft_bild_und_kasten(pg, f, "Hallo", size=g)))
                f.vertical = False
            f.use_bitmap_strikes = True
        for g in (5.5, 11.4, 12.6, 20.5, (11, 11.4), (12, 11.4), (11.4, 11)):
            aus.append((pfad, g, "kopf", _ft_versuch(lambda: (
                f.get_sized_ascender(g), f.get_sized_descender(g), f.get_sized_height(g),
                f.get_sized_glyph_height(g)))))
    if len(aus) < 330:
        raise RuntimeError("freetype kleine Groessen: nur %d Eintraege" % len(aus))
    return aus


def ft_bitmap_cff(pg):
    """`pygame.freetype` mit **Bitmapschriften** (BDF), **eingebetteten
    Bitmaps** (`EBDT`) und **CFF** - samt den Saetzen fuer Bitmapschriften.

    Gemessen in Runde 19: von 918 Fragen an die vier Schriften aus pygames
    eigenen Testdaten waren 539 gleich. BDF ging den alten Weg und wurde in
    der **Punktgroesse** angelegt (`PyGameMono-18-100dpi.bdf` bei 18: kein
    Bild gleich), eingebettete Bitmaps hatten den Vorschub aus `hmtx` statt
    aus den sbit-Massen, CFF ging ungehintet ueber fontdue, und ein
    Unterstrich aus lauter Einbit-Bildern rastet bei pygame auf ganze Pixel
    ein (`is_underline_gray`). Fuer Bitmapschriften stand ein erfundener
    Satz da, und `f.strong = False` warf hier nicht.
    """
    import os as _os
    ft = _freetype(pg)
    ordner = _os.path.join(_os.path.dirname(_echt_pygame_datei()), "tests", "fixtures", "fonts")
    aus = []
    for name, groessen in (("PyGameMono-8.bdf", [8]), ("PyGameMono-18-75dpi.bdf", [18, 19]),
                           ("PyGameMono-18-100dpi.bdf", [18, 25]),
                           ("PyGameMono.otf", [8, 12, 19, 24]), ("test_fixed.otf", [11, 20])):
        try:
            f = ft.Font(_os.path.join(ordner, name))
        except Exception as e:  # noqa: BLE001
            raise RuntimeError("freetype Bitmap: %r laedt nicht (%s)" % (name, e))
        for g in groessen:
            for t in ("Ab", "Hello World", "g", "", " ", "|_|"):
                for stil in (ft.STYLE_DEFAULT, ft.STYLE_UNDERLINE, ft.STYLE_WIDE):
                    for aa in (True, False):
                        f.antialiased = aa
                        aus.append((name, g, t, stil, aa, _ft_versuch(
                            lambda: (lambda s, r: (tuple(r), s.get_size(), s.get_bitsize(),
                                                   _ft_bild(pg, s)))(
                                *f.render(t, (200, 100, 50), size=g, style=stil))),
                            _ft_versuch(lambda: tuple(f.get_rect(t, size=g, style=stil)))))
                f.antialiased = True
                aus.append((name, g, t, "metrics", _ft_versuch(lambda: f.get_metrics(t, size=g))))
                f.vertical = True
                aus.append((name, g, t, "senk", _ft_versuch(
                    lambda: (lambda s, r: (tuple(r), s.get_size(), _ft_bild(pg, s)))(
                        *f.render(t, (200, 100, 50), size=g)))))
                f.vertical = False
            for fn in ("get_sized_ascender", "get_sized_descender", "get_sized_height",
                       "get_sized_glyph_height"):
                aus.append((name, g, fn, _ft_versuch(lambda: getattr(f, fn)(g))))
    f = ft.Font(_os.path.join(ordner, "PyGameMono-8.bdf"))
    for kw in (dict(rotation=15), dict(style=ft.STYLE_STRONG), dict(style=ft.STYLE_OBLIQUE),
               dict(rotation=15, style=ft.STYLE_STRONG)):
        aus.append(("bdf-modus", kw, _ft_versuch(lambda: tuple(f.get_rect("Ab", **kw))),
                    _ft_versuch(lambda: f.render_raw("Ab", **kw)[1])))
    for feld, an, aus_ in (("rotation", 15, 0), ("strong", True, False), ("oblique", True, False),
                           ("wide", True, False), ("underline", True, False)):
        def setze(w):
            setattr(f, feld, w)
            return "ok"
        aus.append(("bdf-setzer", feld, _ft_versuch(lambda: setze(an)),
                    _ft_versuch(lambda: setze(aus_))))
    if len(aus) < 550:
        raise RuntimeError("freetype Bitmap: nur %d Eintraege" % len(aus))
    return aus


def font_cff_typo(pg):
    """`pygame.font` mit **CFF**-Schriften und `USE_TYPO_METRICS`.

    Zwei Funde aus Runde 19, beide unter keiner Probe: CFF (`.otf` mit
    PostScript-Umrissen) ging ungehintet ueber fontdue - an
    `PyGameMono.otf` war **keines** von 165 Bildern gleich, an
    `test_fixed.otf` 8. Und FreeType 2.11.1 nimmt die Typo-Werte aus OS/2,
    sobald `fsSelection` Bit 7 steht; hier stand, SDL_ttf sehe sie nie.
    Unter den Windows-Schriften hat keine das Bit **und** abweichende Werte
    - darum wird hier eine gebaut, im Speicher.
    """
    import io
    import os as _os
    import struct
    ordner = _os.path.join(_os.path.dirname(_echt_pygame_datei()), "tests", "fixtures", "fonts")
    pg.font.init()
    aus = []
    for name in ("PyGameMono.otf", "test_fixed.otf"):
        for g in (8, 12, 19, 24, 33):
            try:
                f = pg.font.Font(_os.path.join(ordner, name), g)
            except Exception as e:  # noqa: BLE001
                raise RuntimeError("font CFF: %r laedt nicht (%s)" % (name, e))
            for t in ("Ab", "Hello World", "g!", "Qy"):
                for st in ("", "b", "i", "u"):
                    f.set_bold("b" in st)
                    f.set_italic("i" in st)
                    f.set_underline("u" in st)
                    for aa in (True, False):
                        s = f.render(t, aa, (200, 100, 50))
                        aus.append((name, g, t, st, aa, s.get_size(), _ft_bild(pg, s), f.size(t)))
            aus.append((name, g, "masse", f.get_ascent(), f.get_descent(), f.get_height(),
                        f.get_linesize(), f.metrics("Ag")))
    roh = bytearray(open(_os.path.join(ordner, "test_sans.ttf"), "rb").read())
    n = struct.unpack(">H", roh[4:6])[0]
    tabellen = {}
    for i in range(n):
        tag, _, off, _ = struct.unpack(">4sIII", roh[12 + 16 * i:28 + 16 * i])
        tabellen[tag] = off
    o = tabellen[b"OS/2"]
    ft = _freetype(pg)
    for bit in (True, False):
        d = bytearray(roh)
        fs = struct.unpack(">H", d[o + 62:o + 64])[0]
        d[o + 62:o + 64] = struct.pack(">H", (fs | 0x80) if bit else (fs & ~0x80))
        d[o + 68:o + 74] = struct.pack(">hhh", 1500, -700, 300)
        f = pg.font.Font(io.BytesIO(bytes(d)), 20)
        g = ft.Font(io.BytesIO(bytes(d)), 20)
        aus.append(("typo", bit, f.get_ascent(), f.get_descent(), f.get_height(),
                    f.get_linesize(), f.size("Ag"), g.get_sized_ascender(20),
                    g.get_sized_descender(20), g.get_sized_height(20), g.height,
                    g.ascender, g.descender, tuple(g.get_rect(""))))
    if len(aus) < 300:
        raise RuntimeError("font CFF: nur %d Eintraege" % len(aus))
    return aus


def ft_leerkasten(pg):
    """Der Kasten eines **leeren** Zeichens am Rand.

    Ein Leerzeichen hat keine Tinte, wird aber nicht weggeschnitten - und
    wie weit es zaehlt, haengt von der Glaettung ab. Gemessen an
    `test_sans.ttf`, Groesse 24:

    ```text
                 geglaettet      einfarbig
      "."        (2, 3,  3, 3)   (2, 3,  2, 3)
      " ."       (0, 3, 12, 3)   (0, 3, 11, 3)
      ". "       (2, 3, 12, 3)   (2, 3,  6, 3)
      " . "      (0, 3, 21, 3)   (0, 3, 15, 3)
    ```

    Geglaettet belegt es `[Stift, Stift + Vorschub]`, einfarbig nur
    `[Stift, Stift + 1]`. Rustgame schnitt es ganz weg, und
    `test_freetype_Font_render_to_mono` fiel darueber.
    """
    ft = _freetype(pg)
    aus = []
    for name in ("test_sans.ttf",):
        for aa in (True, False):
            f = ft.Font(os.path.join(_schriftordner(), name))
            f.antialiased = aa
            aus.append((name, aa, [
                (t, tuple(f.get_rect(t, size=24)))
                for t in (" ", "  ", "   ", ".", " .", ". ", " . ", "..",
                          " ..", ".. ", "M", " M", "M ", " M ")
            ]))
    return tuple(aus)


def ft_zwei_stufen(pg):
    """`render_to` mit **halbdurchsichtigem** Grund und Schrift.

    pygames eigener Test schreibt die Regel aus: *"Blitting is done in two
    stages. First the target is alpha filled with the background color, if
    any. Second, the foreground color is alpha blitted to the
    background."*

    Rustgame hatte eine Stufe - `render` backte Grund und Schrift in ein
    Bild, und das Bild wanderte als Ganzes auf das Ziel. Dann wirkt das
    Alpha des Grundes **zweimal**:

    ```text
      Grund (255,120,13,180), Schrift (128,97,213,60), Ziel schwarz
        zwei Stufen   (168, 87, 57)
        ein Bild      (174, 88, 46)
    ```
    """
    ft = _freetype(pg)
    f = ft.Font(os.path.join(_schriftordner(), "test_sans.ttf"))
    # **Einfarbig**, und das ist nicht Bequemlichkeit: bei Glaettung
    # unterscheidet sich die Deckung der **aeussersten** Glyphenspalte um
    # eine Stufe (die Glyphenbox-Grenze), und die ist hier nicht die
    # Frage. Einfarbig ist die Deckung 0 oder 255, und dann haengt jeder
    # Wert nur noch an der Mischung - genau das soll die Probe pruefen.
    # pygames eigener Test heisst aus demselben Grund `render_to_mono`.
    aus = []
    for aa in (False,):
        f.antialiased = aa
        for flaggen, tiefe in ((pg.SRCALPHA, 32), (0, 32), (pg.SRCALPHA, 16),
                               (0, 16), (0, 8)):
            for grund in (None, (255, 120, 13, 180), (255, 120, 13)):
                s = pg.Surface((24, 12), flaggen, tiefe)
                s.fill(pg.Color("black"))
                r = f.render_to(s, (0, 0), " .", (128, 97, 213, 60),
                                grund, size=24)
                # **Nur die unterste Zeile**, und genau das prueft
                # pygames eigener Test auch (`bottomleft`,
                # `bottomright`). Die obere Zeile des Punktes deckt bei
                # fontdue anders als bei FreeType - das ist die
                # Glyphenbox-Grenze und hier nicht die Frage.
                y = r.height - 1
                aus.append((
                    aa, flaggen, tiefe, grund, tuple(r),
                    [tuple(s.get_at((x, y))) for x in range(r.width)],
                ))
    return tuple(aus)


def schrift_ohne_ordner(pg):
    """Die Standardschrift, **nachdem der Arbeitsordner gewechselt hat.**

    ## Warum das eine eigene Probe braucht

    Die Suche nach `freesansbold.ttf` hat vier Stellen, und drei davon
    haengen am Arbeitsverzeichnis. Das darf sich aendern - und in pygames
    eigener Testsuite tut es das. Zwei Tests in `ftfont_test.py` fielen
    darum **mal um und mal nicht**; allein gefahren war die Datei immer
    gruen.

    Dazu die zweite Haelfte desselben Fundes: der mitgelieferte **Name**
    steht fuer die mitgelieferte Schrift. `Font("freesansbold.ttf")` ist
    bei `pygame.font` und `pygame.ftfont` dasselbe wie `Font(None)` -
    bei `pygame.freetype` dagegen ein Fehler, und zwar bei pygame genauso:

    ```text
    pygame.freetype.Font("freesansbold.ttf")
        FileNotFoundError: No file 'freesansbold.ttf' found in
                           working directory '...'
    ```

    Verglichen wird darum **beides**: dass die drei Wege gehen und dass
    der vierte scheitert.

    Der Ordner wird am Ende zurueckgesetzt - eine Probe, die ihn
    stehenlaesst, veraendert jede nach ihr.
    """
    import importlib
    import tempfile

    fo = importlib.import_module(pg.__name__ + ".font")
    ff = importlib.import_module(pg.__name__ + ".ftfont")
    ft = importlib.import_module(pg.__name__ + ".freetype")
    fo.init()
    ff.init()
    ft.init()
    vorher = os.getcwd()
    aus = []
    try:
        os.chdir(tempfile.gettempdir())
        name = fo.get_default_font()
        for was, ruf in (
            ("font None", lambda: fo.Font(None, 20).get_height()),
            ("font Name", lambda: fo.Font(name, 20).get_height()),
            # **Bei `ftfont` nur "gefunden" und nicht die Hoehe.** Die
            # haengt an der Glyphenbox-Grenze (15 gegen 16), und darum
            # geht es hier nicht - die Frage ist, ob die Schrift ueberhaupt
            # gefunden wird. `freetype.Groesse` prueft die Masse.
            ("ftfont None", lambda: ff.Font(None, 20) and "gefunden"),
            ("ftfont Name", lambda: ff.Font(name, 20) and "gefunden"),
            ("freetype None", lambda: ft.Font(None).get_sized_height(20)),
            ("freetype Name", lambda: ft.Font(name).get_sized_height(20)),
        ):
            try:
                aus.append((was, ruf()))
            except Exception as e:                   # noqa: BLE001
                aus.append((was, type(e).__name__))
    finally:
        os.chdir(vorher)
    return tuple(aus)


def _s2(pg, name):
    import importlib
    return importlib.import_module("%s._sdl2.%s" % (pg.__name__, name))


def sdl2_untermodule(pg):
    """Welche Untermodule `_sdl2` hat und was darin steht.

    ## Der achte blinde Fleck

    `pygame._sdl2` hat **acht** Untermodule. `api_coverage.py` fragte nach
    **einem** (`_sdl2.video`), und rustgame hatte drei - `audio`,
    `controller_old`, `mixer`, `sdl2` und `window` fehlten ganz.

    `traceback` und `errorfnc` stehen nicht in der Liste: das erste ist ein
    **importiertes** Modul, das in pygames `dir()` mitlaeuft, das zweite
    SDLs Fehlerhelfer. Beide sind keine Schnittstelle - dieselbe
    Unterscheidung, die `api_coverage.py` seit dem Umbau trifft.
    """
    import importlib
    aus = []
    for name in ("audio", "controller", "controller_old", "mixer", "sdl2",
                 "touch", "video", "window"):
        try:
            m = importlib.import_module("%s._sdl2.%s" % (pg.__name__, name))
        except Exception as e:                       # noqa: BLE001
            aus.append((name, type(e).__name__))
            continue
        namen = sorted(n for n in dir(m)
                       if not n.startswith("_")
                       and n not in ("traceback", "errorfnc"))
        aus.append((name, tuple(namen)))
    return tuple(aus)


def sdl2_tongeraete(pg):
    """Die Namen der Tongeraete - **Wert fuer Wert**.

    Hier wird nicht nur die Form verglichen, sondern die Liste selbst:
    beide Engines fragen dasselbe Betriebssystem, und gemessen sieht `cpal`
    genau dieselben Geraete wie SDL, in derselben Reihenfolge.

    Der einzige Unterschied, den es zu bauen gab, steht in SDLs eigener
    Beschreibung: *"Returns a list of **unique** devicenames"*. Zwei
    Geraete dieser Maschine heissen gleich, und SDL haengt dem zweiten
    ` (2)` an.
    """
    a = _s2(pg, "audio")
    return (a.get_audio_device_names(),
            a.get_audio_device_names(False),
            a.get_audio_device_names(True))


def sdl2_audiozahlen(pg):
    """Die achtzehn `AUDIO_*`-Zahlen - SDLs Formatkennungen."""
    a = _s2(pg, "audio")
    return tuple(
        (n, getattr(a, n)) for n in sorted(
            x for x in dir(a) if x.startswith("AUDIO_"))
    )


def sdl2_teilsysteme(pg):
    """Die neun `INIT_*`-Flaggen und `init_subsystem`.

    **`INIT_EVERYTHING` ist nicht die Summe der anderen**: 62001 ist
    `0xF231`, und darin fehlt `INIT_NOPARACHUTE`. Wer die Zahl herleitet
    statt sie abzulesen, liegt daneben.
    """
    s = _s2(pg, "sdl2")
    flaggen = tuple((n, getattr(s, n)) for n in sorted(
        x for x in dir(s) if x.startswith("INIT_")))
    rueck = []
    for f in (s.INIT_VIDEO, s.INIT_AUDIO, s.INIT_TIMER, 0):
        try:
            rueck.append((f, s.init_subsystem(f)))
        except Exception as e:                       # noqa: BLE001
            rueck.append((f, type(e).__name__, str(e)))
    return (flaggen, tuple(rueck))


def sdl2_altes_geraet(pg):
    """`_sdl2.controller_old` - die alte Fassung, vier Namen mehr.

    Und `Controller` ist dort eine **eigene** Klasse: gemessen ist
    `controller_old.Controller is controller.Controller` **falsch**. Ein
    Zweitname waere bequem und bei `isinstance` falsch.
    """
    c = _s2(pg, "controller_old")
    n = _s2(pg, "controller")
    c.init()
    aus = [("eigene Klasse", c.Controller is not n.Controller),
           ("get_init", c.get_init()),
           ("get_count", c.get_count()),
           ("get_eventstate", c.get_eventstate()),
           ("set_eventstate", c.set_eventstate(True)),
           ("name_forindex", c.name_forindex(0)),
           ("update", c.update()),
           ("is_controller", c.is_controller(0))]
    for was, ruf in (("Controller(0)", lambda: c.Controller(0)),
                     ("Controller(True)", lambda: c.Controller(True)),
                     ("is_controller('x')", lambda: c.is_controller("x"))):
        try:
            aus.append((was, "ok"))
        except Exception as e:                       # noqa: BLE001
            aus.append((was, type(e).__name__))
        else:
            try:
                ruf()
                aus[-1] = (was, "ok")
            except Exception as e:                   # noqa: BLE001
                aus[-1] = (was, type(e).__name__, str(e))
    return tuple(aus)


def sdl2_startereignisse(pg):
    """`AUDIODEVICEADDED` - je Tongeraet eines, beim Hochfahren.

    Das war der Fund, der die ganze Reihe ausgeloest hat: die Endabnahme
    des Plans verlangt **dreihundert** Bilder, gefahren wurden dreissig.
    Bei dreihundert wich die Ereignisschlange ab, und darin fehlten sechs
    Ereignisse - fuenf Ausgabe-, ein Aufnahmegeraet.

    Die Nummern zaehlen **je Art** von null an: das Mikrofon ist
    `which=0, iscapture=1`, nicht `which=5`.
    """
    # **Die Schlange erst leeren.** Sonst haengt das Ergebnis davon ab,
    # was eine frueher gelaufene Probe darin liegen gelassen hat - und
    # genau daran hat diese hier beim ersten Lauf gelitten: pygame gab
    # zwei Ereignisse statt sechs, weil die Schlange schon voll war.
    pg.init()
    pg.event.clear()
    pg.mixer.quit()
    pg.mixer.init()
    aus = [(e.type, e.which, e.iscapture) for e in pg.event.get()
           if e.type in (pg.AUDIODEVICEADDED, pg.AUDIODEVICEREMOVED)]
    # **Die Form, nicht die Anzahl** - und das ist eine gemessene
    # Einschraenkung, keine Bequemlichkeit.
    #
    # In einem frischen Auswerter legen beide Engines **sechs** Ereignisse
    # ab (fuenf Ausgabe-, ein Aufnahmegeraet). Nach mehrfachem
    # `mixer.quit()` / `mixer.init()` gibt pygame nur noch zwei - SDL fuehrt
    # Buch darueber, welche Geraete es schon gemeldet hat, und diese
    # Buchfuehrung ist von aussen nicht zu sehen. Da `vergleich_module.py`
    # beide Engines in **einem** Auswerter fahrt und vorher hunderte
    # Proben laufen, haengt die Anzahl hier an dieser Buchfuehrung.
    #
    # Vergleichbar bleibt alles andere, und es ist das, worauf ein
    # Programm sich stuetzt: jedes Ereignis ist ein `AUDIODEVICEADDED`,
    # `iscapture` ist 0 oder 1, die Nummern zaehlen **je Art** von null an
    # und lueckenlos, und die Ausgabegeraete kommen zuerst.
    arten = [c for _, _, c in aus]
    nummern = {}
    for _, which, c in aus:
        nummern.setdefault(c, []).append(which)
    return (
        all(t == pg.AUDIODEVICEADDED for t, _, _ in aus),
        all(c in (0, 1) for c in arten),
        arten == sorted(arten),
        {c: n == list(range(len(n))) for c, n in sorted(nummern.items())},
        sorted(nummern) in ([], [0], [0, 1]),
        len(aus) > 0,
    )


def sdl2_roher_strom(pg):
    """Bekannte Grenze: `AudioDevice` und `set_post_mix`.

    Beide brauchen einen **rohen Tonstrom mit Rueckruf** - pygame ruft
    dafuer aus einem Tonfaden heraus Python auf. rustgames Mixer hat keine
    zentrale Mischstufe (jeder Kanal geht einzeln an die Tonkiste), und ein
    Rueckruf aus dem Tonfaden muesste die Sperre des Auswerters holen.

    Was geht, geht: die Geraeteliste, `pygame.mixer` selbst,
    `set_post_mix(None)`.
    """
    a = _s2(pg, "audio")
    m = _s2(pg, "mixer")
    aus = []
    try:
        aus.append(("set_post_mix(None)", m.set_post_mix(None)))
    except Exception as e:                           # noqa: BLE001
        aus.append(("set_post_mix(None)", type(e).__name__))
    for was, ruf in (
        ("AudioDevice()", lambda: a.AudioDevice()),
        ("set_post_mix(f)", lambda: m.set_post_mix(lambda *x: None)),
        ("set_post_mix(7)", lambda: m.set_post_mix(7)),
    ):
        try:
            ruf()
            aus.append((was, "ok"))
        except Exception as e:                       # noqa: BLE001
            aus.append((was, type(e).__name__))
    return tuple(aus)


def alle_untermodule(pg):
    """**Jedes** Untermodul, das `pygame` mitbringt - und seine Namen.

    ## Der neunte blinde Fleck

    `api_coverage.py` pflegte seine Modulliste von Hand, und `pygame` hat
    **sechzig** Untermodule. Neun davon waren in rustgame nicht einmal
    importierbar: `__briefcase`, `__pyinstaller`, `_camera`,
    `_camera_opencv`, `_data_classes`, `examples`, `macosx`, `newbuffer`,
    `pypm`.

    Diese Probe zaehlt sie alle auf und vergleicht die Namen. Zwei Sorten
    bleiben draussen:

    * **mitimportierte Module** (`os`, `sys`, `traceback`, `platform`) -
      sie stehen in `dir()`, sind aber keine Schnittstelle,
    * **`errorfnc`**, SDLs Fehlerhelfer aus `_sdl2.video`.

    `_camera_opencv` scheitert bei **beiden** ohne `cv2` - dieselbe
    Antwort ist keine Luecke, und die Probe haelt das fest.
    """
    import importlib
    import pkgutil

    namen = sorted(m.name for m in pkgutil.iter_modules(
        __import__("pygame").__path__) if m.name != "tests")
    # **Keine Ausnahmeliste.** Hier stand eine: `os`, `sys`, `traceback`,
    # `platform`, `errorfnc`, `importlib`, `warnings`, `dataclasses` -
    # mitimportierte Namen, die "niemandem gehoeren". Genau so eine Liste
    # war aber der **neunte** blinde Fleck dieses Projekts, und sie deckte
    # hier den zehnten: `api_coverage.py` fragt Konstanten in Untermodulen
    # gar nicht, und diese Probe liess die Importe aus - also fragte
    # niemand nach `font.UCS4`, `base.HAVE_NEWBUF`, `_freetype.BBOX_*`,
    # `midi.MIDIIN`, `camera.ABC`.
    #
    # Was `dir()` zeigt, ist die Schnittstelle. **Ein Name zuviel ist
    # derselbe Fehler wie einer zuwenig**, und wer entscheidet, welcher
    # Import "wichtig" ist, pflegt wieder eine Liste von Hand.
    weg = frozenset()
    aus = []
    for n in namen:
        try:
            m = importlib.import_module("%s.%s" % (pg.__name__, n))
        except Exception as e:                       # noqa: BLE001
            aus.append((n, type(e).__name__))
            continue
        aus.append((n, tuple(sorted(
            x for x in dir(m) if not x.startswith("_") and x not in weg))))
    return tuple(aus)


def _ohne_modul(roh):
    """`len(x)` - der Satz **ohne** den Modulnamen.

    `object of type 'pygame.newbuffer.BufferMixin' has no len()` gegen
    `... 'rustgame_py.newbuffer.BufferMixin' ...`: der Unterschied ist der
    Modulname und sonst nichts. Der steht als `Namen der Typen und Module`
    schon in den benannten Grenzen - hier wird der Rest geprueft, damit
    eine *andere* Aenderung an dem Satz nicht darin untergeht.
    """
    try:
        len(roh)
    except TypeError as e:
        return str(e).replace("pygame.newbuffer.", "").replace(
            "rustgame_py.newbuffer.", "")
    return "kein Fehler"


def _alle_laden(pg):
    """Importiert **jedes** Untermodul - und gibt die Namen zurueck.

    Noetig fuer die zwei Proben darunter, und zwar wegen einer Eigenheit
    von Paketen: `pygame.midi` ist kein Feld von `pygame`, bevor jemand es
    importiert hat. Gemessen fehlten `dir(pygame)` darum **zwoelf** Namen,
    die rustgame von Anfang an traegt:

    ```text
    camera docs examples freetype ftfont geometry gfxdraw locals
    macosx midi newbuffer pypm
    ```

    Das ist kein Unterschied der Schnittstelle, sondern einer des
    Zeitpunkts - nach `import pygame.midi` hat pygame den Namen auch. Statt
    diese zwoelf in eine Ausnahmeliste zu schreiben (und damit den naechsten
    blinden Fleck anzulegen), laedt diese Funktion **alles** auf beiden
    Seiten. Danach ist der Vergleich ohne Ausnahme zu haben.
    """
    import importlib
    import pkgutil

    namen = sorted(m.name for m in pkgutil.iter_modules(
        __import__("pygame").__path__) if m.name != "tests")
    for n in namen:
        try:
            importlib.import_module("%s.%s" % (pg.__name__, n))
        except Exception:                            # noqa: BLE001
            continue
    return namen


#: **`tests` zaehlt nicht mit.** pygame bringt seine eigene Testsuite als
#: Unterpaket mit, und irgendwann im Lauf importiert sie etwas davon -
#: danach steht `tests` in `dir(pygame)`. rustgame hat kein solches Paket
#: (es fahrt pygames eigene, siehe `pygame_tests.py`). Das ist kein Teil
#: der Schnittstelle.
#:
#: **Und `imageext` ist der zweite - der einzige echte.** pygame hat ihn
#: nie als Feld des Pakets: sein `__init__.py` holt zwei Funktionen von
#: dort und **loescht den Namen** wieder, und eine `import`-Anweisung
#: setzt ihn fuer ein Modul, das schon in `sys.modules` steht, nicht neu.
#: Gemessen ist `hasattr(pygame, "imageext")` darum immer `False`.
#:
#: rustgame **muss** ihn haben, und das ist keine Bequemlichkeit:
#: `rustgame_py` ist **ein einziges Modul** und hat kein `__path__`. Bei
#: pygame findet `from pygame import imageext` das Untermodul auch ohne
#: Feld - hier nicht. Ohne das Feld greift `imageext_test.py` zum echten
#: pygame und bekommt
#: `TypeError: argument 1 must be pygame.surface.Surface` - drei Fehler,
#: nachgemessen.
#:
#: Also ein Name mehr, mit Grund, und **nur** dieser eine: jeder andere
#: Ueberschuss faellt weiterhin auf.
_KEIN_TEIL = frozenset({"tests", "imageext"})


def hauptmodul_namen(pg):
    """`dir(pygame)` gegen `dir(rustgame_py)` - **in beide Richtungen**.

    ## Der dreizehnte blinde Fleck

    `api_coverage.py` fragt, was pygame **hat**, und meldet, was fehlt. Ein
    **Ueberschuss** faellt dort durch - und am Hauptmodul standen zwei
    Namen zuviel: `Circle` und `Line`. Gemessen gibt es die bei pygame nur
    unter `pygame.geometry`; `hasattr(pygame, "Circle")` ist `False`, auch
    nach `import pygame.geometry`.

    Die Probe `Untermodule gesamt` vergleicht `dir()` schon in beide
    Richtungen - aber nur fuer **Untermodule**. Das Hauptmodul selbst war
    nie dabei. **Ein Name zuviel ist derselbe Fehler wie einer zuwenig**,
    und diesmal hat die Luecke in der Messung sie beide gedeckt.
    """
    _alle_laden(pg)
    return tuple(sorted(n for n in dir(pg)
                        if not n.startswith("_") and n not in _KEIN_TEIL))


def sternimport(pg):
    """Was `from pygame.x import *` bringt - fuer jedes Modul.

    ## Was `dir()` nicht beantwortet

    `import *` nimmt `__all__`, wenn es eines gibt, und sonst jeden Namen
    ohne Unterstrich aus dem **Woerterbuch**. Gemessen haben **51 von 60**
    Untermodulen hier ein anderes `__all__` als pygame - PyO3 traegt jeden
    Namen, der mit `PyModule::add` kommt, dort mit ein -, und bei
    **vierzehn** brachte `import *` deshalb etwas anderes:

    ```text
    camera    fehlt ABC, abstractmethod, os, platform, sys, warnings
    cursors   fehlt pygame
    _sprite   fehlt warnings, zuviel OrderedUpdates
    freetype  zuviel SysFont
    ftfont    zuviel FontType
    ```

    Die ersten drei waren Namen, die mit `setattr` kamen und damit nicht in
    `__all__` landeten; die letzten zwei der umgekehrte Fall - dort hat
    **pygame** ein eigenes `__all__`, und es ist schmaler als sein
    Woerterbuch. Genau **elf** Module geben sich bei pygame selbst eines
    (`constants`, `freetype`, `ftfont`, `locals`, `macosx`, `midi`,
    `pkgdata`, `sndarray`, `surfarray`, `typing`, `version`); die
    Ausnahmen stehen in `lib.rs` am Ende der Modulfunktion, mit der
    Messung daneben.
    """
    import importlib

    namen = _alle_laden(pg)
    aus = []
    for n in [""] + namen:
        voll = pg.__name__ + ("." + n if n else "")
        try:
            importlib.import_module(voll)
        except Exception:                            # noqa: BLE001
            aus.append((n or "(haupt)", "nicht ladbar"))
            continue
        g = {}
        try:
            exec("from %s import *" % voll, g)       # noqa: S102
        except Exception as e:                       # noqa: BLE001
            aus.append((n or "(haupt)", type(e).__name__, str(e)))
            continue
        aus.append((n or "(haupt)",
                    tuple(sorted(k for k in g if not k.startswith("_")
                                 and k not in _KEIN_TEIL))))
    return tuple(aus)


#: Doppelstriche, die bei **jedem** Typ anders sein duerfen.
#:
#: `__module__` und `__qualname__` sagen, wo die Klasse wohnt - das ist die
#: benannte Grenze `Namen der Typen und Module`. `__doc__` ist hier deutsch.
#: `__dict__`, `__weakref__`, `__slots__`, `__abstractmethods__`,
#: `__firstlineno__` und `__static_attributes__` haengen daran, **wie** eine
#: Klasse gebaut ist, nicht was sie kann. `__text_signature__` gibt es nur
#: bei C-Funktionen, `__pyx_vtable__` nur bei Cython, `__orig_bases__` nur
#: bei einer getippten Oberklasse.
#:
#: Jeder andere Doppelstrich wird verglichen - und drei davon waren Fehler.
_DOPPEL_EGAL = frozenset({
    "__module__", "__doc__", "__dict__", "__text_signature__", "__weakref__",
    "__qualname__", "__abstractmethods__", "__slots__", "__firstlineno__",
    "__static_attributes__", "__pyx_vtable__", "__orig_bases__",
})


def _klasse(pg, weg):
    """Die Klasse am Ende eines Wegs - oder `None`."""
    import importlib

    teile = weg.split(".")
    for k in range(len(teile)):
        try:
            importlib.import_module("%s.%s" % (pg.__name__, ".".join(teile[: k + 1])))
        except Exception:                            # noqa: BLE001
            break
    obj = pg
    for stueck in teile:
        obj = getattr(obj, stueck, None)
        if obj is None:
            return None
    return obj if isinstance(obj, type) else None


def klassendoppel(pg):
    """Die **Doppelstriche** jeder Klasse - `__hash__`, `__iadd__`, `__len__`.

    ## Die Frage, die alle anderen Proben auslassen

    Jede Namensprobe hier filtert Namen mit Unterstrich - und ein
    Doppelstrich **ist** ein Name mit Unterstrich. Damit fragte niemand
    nach den Methoden, an denen die Sprache selbst haengt. Gemessen gingen
    **22 von 65** Klassen auseinander, und drei davon waren echte Fehler:

    * **`Vector2` und `Vector3` fehlten `__iadd__`, `__isub__`,
      `__imul__`, `__itruediv__`, `__ifloordiv__`.** Ohne die faellt
      Python auf `v = v + w` zurueck: das Ergebnis stimmt, das **Objekt**
      ist ein neues. Wer den Vektor woanders festhaelt, sieht die
      Aenderung nicht.
    * **`Cursor` war keine Folge.** `len(c)`, `c[0]` und `list(c)` gaben
      `TypeError`, wo pygame 1, die Konstante und `[0]` gibt - pygames
      Zeiger reicht an sein Datentupel durch.
    * **`system.PowerState` war kein `@dataclass`.** Damit fehlten
      `__match_args__` (jedes `match`-Muster), `__dataclass_fields__` und
      `__replace__`.

    Dazu fuenf kleinere: `__safe_for_unpickling__` (bei `Rect` ein Feld,
    bei `Vector2` eine **Methode** - gemessen), `ScancodeWrapper.__next__`,
    `pypm.Input.__setstate__`, `sprite.AbstractGroup.__class_getitem__`,
    und vier zuviel: `Sound.__deepcopy__`, `BufferProxy.__len__`
    (`len(s.get_buffer())` gibt bei pygame `TypeError`) und
    `Mask.__array_struct__` (pygames Maske hat **keine**
    Feldschnittstelle).

    Was dabei legitim anders ist, steht in [`_DOPPEL_EGAL`] - mit einem
    Satz je Name, warum.
    """
    aus = []
    for weg in __import__("api_coverage").alle_klassen():
        k = _klasse(pg, weg)
        if k is None:
            aus.append((weg, "fehlt"))
            continue
        aus.append((weg, tuple(sorted(
            n for n in dir(k)
            if n.startswith("__") and n.endswith("__")
            and n not in _DOPPEL_EGAL))))
    return tuple(aus)


def instanznamen(pg):
    """Die Namen an der **Instanz** - und damit die Pruefung einer Grenze.

    ## Was hier gepruueft wird

    Die benannte Grenze `Felder an der Klasse` sagt: PyO3 legt ein Feld
    (`#[pyo3(get, set)]`) als Beschreiber an der **Klasse** ab, pygames
    Python-Klassen setzen dasselbe Feld im `__init__` an der **Instanz** -
    sechzehn Klassen, immer dieselbe Sorte Name (`lostsprites`, `dirty`,
    `ratio`). Und dann der Satz, auf den es ankommt:

    > An der Instanz hat sie **beide** Seiten.

    **Das ist eine Behauptung, und Behauptungen sind zu messen.** Diese
    Probe baut je eine Instanz und vergleicht ihre oeffentlichen Namen.
    Ergebnis beim Einbau: **0 von 28** gingen auseinander, und keine liess
    sich nicht bauen - die Grenze ist damit nachgeprueft und nicht nur
    begruendet.

    ## Warum nicht alle 65 Klassen

    Weil ein Konstruktor etwas **tut**: `_sdl2.Window` oeffnet ein Fenster,
    `AudioDevice` ein Tongeraet, `Camera` sucht die Kamera, `Joystick` und
    `Controller` scheitern ohnehin. Eine Probe, die Geraete anfasst, ist
    keine Probe mehr. Gebaut werden darum die, bei denen das Bauen nichts
    kostet - und die Zahl steht im Ergebnis, damit sichtbar bleibt, wie
    viele es sind.

    Die **Klassenebene** deckt alle 65 ab (`Klassennamen`,
    `Klassendoppel`, `Klassenbasen`); diese Probe ist die Gegenprobe dazu.
    """
    aus = []
    nicht = []

    def bauen(name, ruf):
        try:
            o = ruf()
        except Exception as e:                       # noqa: BLE001
            nicht.append((name, type(e).__name__))
            return
        # **Der Typname gehoert mit.** `display.Info()` gibt bei pygame
        # eine `VidInfo` - eine Klasse, die in **keinem** Modul steht und
        # nur ueber `type(Info())` zu erreichen ist. `alle_klassen()`
        # zaehlt auf, was in einem Modul steht; eine Klasse, die nur ein
        # Aufruf hergibt, hat darum keine Probe je gesehen. Hier hiess sie
        # `Info` statt `VidInfo`, und `Info` war eine Klasse statt einer
        # Fabrik.
        aus.append((name, type(o).__name__, tuple(sorted(
            v for v in dir(o) if not v.startswith("_")))))

    flaeche = pg.Surface((4, 4))
    for name, ruf in (
        # Die sechzehn, um die es bei der Grenze geht.
        ("sprite.AbstractGroup", pg.sprite.AbstractGroup),
        ("sprite.Group", pg.sprite.Group),
        ("sprite.GroupSingle", pg.sprite.GroupSingle),
        ("sprite.LayeredUpdates", pg.sprite.LayeredUpdates),
        ("sprite.LayeredDirty", pg.sprite.LayeredDirty),
        ("sprite.RenderPlain", pg.sprite.RenderPlain),
        ("sprite.RenderClear", pg.sprite.RenderClear),
        ("sprite.RenderUpdates", pg.sprite.RenderUpdates),
        ("sprite.OrderedUpdates", pg.sprite.OrderedUpdates),
        ("sprite.Sprite", pg.sprite.Sprite),
        ("sprite.DirtySprite", pg.sprite.DirtySprite),
        ("collide_rect_ratio", lambda: pg.sprite.collide_rect_ratio(1.0)),
        ("collide_circle_ratio", lambda: pg.sprite.collide_circle_ratio(1.0)),
        # Und die anderen, bei denen das Bauen nichts kostet.
        ("Rect", lambda: pg.Rect(1, 2, 3, 4)),
        ("FRect", lambda: pg.FRect(1.5, 2.5, 3.5, 4.5)),
        ("Color", lambda: pg.Color(1, 2, 3, 4)),
        ("Vector2", lambda: pg.Vector2(1, 2)),
        ("Vector3", lambda: pg.Vector3(1, 2, 3)),
        ("Surface", lambda: flaeche),
        ("Mask", lambda: pg.mask.Mask((4, 4))),
        ("time.Clock", pg.time.Clock),
        ("Cursor", lambda: pg.cursors.Cursor(pg.SYSTEM_CURSOR_ARROW)),
        ("Event", lambda: pg.event.Event(pg.USEREVENT, {"x": 1})),
        ("PixelArray", lambda: pg.PixelArray(pg.Surface((2, 2)))),
        ("BufferProxy", lambda: pg.Surface((2, 2)).get_buffer()),
        ("system.PowerState",
         lambda: pg.system.PowerState(None, None, False, True, False, False,
                                      True, False)),
        ("math.VectorElementwiseProxy", lambda: pg.Vector2(1, 2).elementwise()),
        ("math.VectorIterator", lambda: iter(pg.Vector2(1, 2))),
        # Rueckgabetypen, die in keinem Modul stehen.
        ("display.Info()", lambda: pg.display.Info()),
        ("Surface.get_rect()", lambda: pg.Surface((2, 2)).get_rect()),
        ("Surface.get_frect()", lambda: pg.Surface((2, 2)).get_frect()),
        ("version.vernum", lambda: pg.version.vernum),
        ("version.SDL", lambda: pg.version.SDL),
    ):
        bauen(name, ruf)
    return (tuple(aus), tuple(nicht))


def vektor_reste(pg):
    """Die sieben Vektormethoden, die keine Probe je gerufen hat.

    `tools/abdeckung.py` hat sie genannt: `magnitude_squared`,
    `rotate_ip_rad`, `rotate_x_ip_rad`, `rotate_y_ip_rad`,
    `rotate_z_ip_rad`, `update` am `Vector3` und `rotate_ip_rad` am
    `Vector2`. Sie sind trivial erreichbar - es hatte nur nie jemand
    gefragt.
    """
    aus = []
    v3 = pg.Vector3(3, 4, 12)
    aus.append(("V3.magnitude_squared", v3.magnitude_squared()))
    v2 = pg.Vector2(3, 4)
    aus.append(("V2.magnitude_squared", v2.magnitude_squared()))
    for winkel in (0.0, 0.5, 1.5707963267948966, 3.141592653589793, -0.75):
        a = pg.Vector2(1, 0)
        a.rotate_ip_rad(winkel)
        aus.append(("V2.rotate_ip_rad(%r)" % winkel,
                    (round(a.x, 12), round(a.y, 12))))
        for achse in ("x", "y", "z"):
            b = pg.Vector3(1, 2, 3)
            getattr(b, "rotate_%s_ip_rad" % achse)(winkel)
            aus.append(("V3.rotate_%s_ip_rad(%r)" % (achse, winkel),
                        tuple(round(w, 12) for w in b)))
        c = pg.Vector3(1, 2, 3)
        c.rotate_ip_rad(winkel, (0, 0, 1))
        aus.append(("V3.rotate_ip_rad(%r)" % winkel,
                    tuple(round(w, 12) for w in c)))
    d = pg.Vector3(1, 2, 3)
    d.update(7, 8, 9)
    aus.append(("V3.update(7,8,9)", tuple(d)))
    d.update((4, 5, 6))
    aus.append(("V3.update(Folge)", tuple(d)))
    d.update(pg.Vector3(0, 0, 0))
    aus.append(("V3.update(Vector3)", tuple(d)))
    e = pg.Vector2(1, 2)
    e.update(5, 6)
    aus.append(("V2.update(5,6)", tuple(e)))
    e.update()
    aus.append(("V2.update()", tuple(e)))
    # Die Fehlerwege gehoeren dazu.
    for ruf, name in [
        (lambda: pg.Vector3(1, 2, 3).rotate_ip_rad("x", (0, 0, 1)),
         "V3.rotate_ip_rad(str)"),
        (lambda: pg.Vector3(1, 2, 3).update(1, 2), "V3.update(1,2)"),
        (lambda: pg.Vector2(1, 2).update(1, 2, 3), "V2.update(1,2,3)"),
        (lambda: pg.Vector3(1, 2, 3).rotate_x_ip_rad("x"),
         "V3.rotate_x_ip_rad(str)"),
    ]:
        try:
            ruf()
            aus.append((name, "KEIN FEHLER"))
        except Exception as e:                      # noqa: BLE001
            aus.append((name, "%s: %s" % (type(e).__name__, e)))
    return tuple(aus)


def freetype_modul(pg):
    """Die Modulfunktionen von `pygame.freetype` - **neun ungerufene.**

    `get_cache_size`, `get_default_resolution`, `get_error`, `get_init`,
    `get_version`, `init`, `quit`, `set_default_resolution` und
    `was_init`. Keine hat je eine Probe gesehen, obwohl alle ohne Geraet
    laufen.

    `get_version()` ist dabei die interessante: pygame nennt die Version
    der eingebundenen FreeType-Bibliothek, hier gibt es keine. Was
    herauskommt, entscheidet die Messung - geraten wird nichts.
    """
    import importlib

    ft = importlib.import_module(pg.__name__ + ".freetype")
    aus = []

    def z(name, f):
        try:
            aus.append((name, f()))
        except Exception as e:                      # noqa: BLE001
            aus.append((name, "%s: %s" % (type(e).__name__, e)))

    z("get_init vor init", ft.get_init)
    z("init", lambda: ft.init())
    z("get_init nach init", ft.get_init)
    z("was_init", lambda: getattr(ft, "was_init", ft.get_init)())
    z("get_error", ft.get_error)
    z("get_default_resolution", ft.get_default_resolution)
    z("set_default_resolution(96)", lambda: ft.set_default_resolution(96))
    z("get_default_resolution nach 96", ft.get_default_resolution)
    z("set_default_resolution()", lambda: ft.set_default_resolution())
    z("get_default_resolution zurueck", ft.get_default_resolution)
    z("get_cache_size", lambda: ft.get_cache_size() >= 0)
    z("get_default_font", ft.get_default_font)
    z("quit", lambda: ft.quit())
    z("get_init nach quit", ft.get_init)
    z("init erneut", lambda: ft.init())
    return tuple(aus)


def sysfont_proben(pg):
    """`pygame.sysfont` - **acht ungerufene Namen.**

    Fuenf davon sind Durchleitungen aus `os.path`, die pygames
    `sysfont.py` am Kopf einfuehrt (`basename`, `dirname`, `exists`,
    `join`, `splitext`); sie stehen damit in `dir()` und sind Teil der
    Schnittstelle. Die drei anderen sind die Arbeit: `SysFont`,
    `get_fonts`, `match_font`.

    **Was hier verglichen wird, ist absichtlich nicht die Schriftenliste
    selbst** - die haengt an den installierten Schriften und waere auf
    einer anderen Maschine anders. Verglichen wird, dass beide Huellen
    **dieselbe** Liste finden, und das ist eine echte Aussage: sie lesen
    dieselbe Registrierung mit demselben portierten Quelltext.
    """
    import importlib

    sf = importlib.import_module(pg.__name__ + ".sysfont")
    aus = []

    def z(name, f):
        try:
            aus.append((name, f()))
        except Exception as e:                      # noqa: BLE001
            aus.append((name, "%s: %s" % (type(e).__name__, e)))

    z("basename", lambda: sf.basename("a/b/c.ttf"))
    z("dirname", lambda: sf.dirname("a/b/c.ttf"))
    z("join", lambda: sf.join("a", "b"))
    z("splitext", lambda: sf.splitext("a/b/c.ttf"))
    z("exists(unsinn)", lambda: sf.exists("gibtesnicht_rustgame"))
    z("get_fonts Anzahl", lambda: len(sf.get_fonts()) > 0)
    z("get_fonts sortiert", lambda: sf.get_fonts() == sorted(sf.get_fonts()))
    z("get_fonts kleingeschrieben",
      lambda: all(n == n.lower() for n in sf.get_fonts()))
    # **Keine Pruefsumme ueber die ganze Liste.**
    #
    # Der erste Entwurf verglich den SHA-256 aller Namen, und das war zu
    # streng: die Liste ist eine Tatsache des **Systems**, und zwei
    # Namen von 391 weichen aus einem Grund ab, der nicht in dieser
    # Liste steht (siehe die Grenze `sysfont`). Eine Probe, die bei
    # jedem neu installierten Zeichensatz anders ausfaellt, misst nicht
    # die Huelle.
    #
    # Verglichen wird darum die **Form** der Liste und ihr Kern: dass
    # beide Huellen dieselben bekannten Schriften finden, dieselbe
    # Anzahl nennen und dieselben Namen klein und sortiert liefern.
    z("get_fonts Anzahl gleich", lambda: len(sf.get_fonts()))
    z("get_fonts bekannte Namen", lambda: tuple(
        n in sf.get_fonts() for n in
        ("arial", "couriernew", "verdana", "timesnewroman", "segoeui",
         "consolas", "tahoma", "comicsansms", "freesansbold")))
    z("get_fonts nur ASCII", lambda: sum(
        1 for n in sf.get_fonts() if not n.isascii()))
    z("match_font(arial) endet auf ttf",
      lambda: (sf.match_font("arial") or "").lower().endswith(
          (".ttf", ".ttc", ".otf")))
    z("match_font Pruefsumme", lambda: __import__("hashlib").sha256(
        repr([sf.match_font(n) for n in ("arial", "couriernew", "verdana",
                                         "timesnewroman", "gibtesnicht")]
             ).encode("utf-8")).hexdigest()[:16])
    z("match_font(bold)", lambda: bool(sf.match_font("arial", bold=True)))
    z("match_font(italic)", lambda: bool(sf.match_font("arial", italic=True)))
    z("match_font(Liste)",
      lambda: bool(sf.match_font("gibtesnicht,arial")))
    z("SysFont Hoehe", lambda: sf.SysFont("arial", 24).get_height() > 0)
    z("SysFont bold ist bold",
      lambda: sf.SysFont("arial", 24, bold=True).get_bold())
    z("SysFont italic ist italic",
      lambda: sf.SysFont("arial", 24, italic=True).get_italic())
    z("SysFont unbekannt faellt zurueck",
      lambda: sf.SysFont("gibtesnicht", 24).get_height() > 0)
    z("SysFont Typ", lambda: type(sf.SysFont("arial", 24)).__name__)

    # ----- Die Tabelle, die pygame an dieser Stelle fuehrt ----------------
    #
    # `Sysfonts`, `Sysalias` und `is_init` sind bei pygame **Zustand**:
    # vor dem ersten `match_font` steht dort `False` und zwei leere
    # Woerterbuecher, danach `True`, 391 Schriften und 33 Zweitnamen -
    # gemessen. Hier blieb alles leer, weil die Suche im Kern steht und
    # niemand die Python-Seite anstiess; `Sysfonts` wurde zudem nur mit
    # `{(False, False): pfad}` gefuellt, wo pygame **alle** Schnitte
    # fuehrt.
    #
    # **Die Reihenfolge zaehlt mit.** Hier stand, verglichen werde "nicht
    # die Reihenfolge" - verglichen wurde aber `repr`, und das kennt sie.
    # Die Probe war darum rot, waehrend `gegenueber` (das Woerterbuecher
    # nach Schluesseln sortiert) keine einzige Zeile zeigte. Und die
    # Reihenfolge ist Verhalten: wo pygame "irgendeinen" Schnitt nimmt,
    # nimmt es den ersten eingetragenen. Siehe `Schnitte` in
    # `rustgame-font/src/sysfont.rs`.
    #
    # Die Namensmenge ging um **zwei von 391** auseinander, solange die
    # Registrierung ueber `reg query` gelesen wurde (Konsolen-Codepage,
    # das a-Umlaut in `lucidasanstypewriterschraeg` kam als Ersatzzeichen
    # an). Seit sie ueber `windows-registry` gelesen wird, steht hier die
    # **ganze** Menge.
    z("is_init nach match_font", lambda: sf.is_init)
    z("Sysfonts Anzahl", lambda: len(sf.Sysfonts))
    z("Sysalias Anzahl", lambda: len(sf.Sysalias))
    z("Sysfonts Namen", lambda: tuple(sorted(sf.Sysfonts)))
    z("Sysfonts Namen in Reihenfolge", lambda: tuple(sf.Sysfonts))
    z("Sysalias Namen", lambda: tuple(sorted(sf.Sysalias)))
    z("Sysfonts arial", lambda: sf.Sysfonts.get("arial"))
    z("Sysfonts courier", lambda: sf.Sysfonts.get("couriernew"))
    z("Sysalias monospace", lambda: sf.Sysalias.get("monospace"))
    z("Sysalias serif", lambda: sf.Sysalias.get("serif"))
    z("Sysfonts Schnitte je Name", lambda: tuple(
        sorted(set(len(v) for v in sf.Sysfonts.values()))))
    z("Sysfonts alle Pfade da", lambda: all(
        isinstance(p, str) and p for v in sf.Sysfonts.values()
        for p in v.values()))
    # pygames `create_aliases` setzt den Zweitnamen auf **dasselbe**
    # Woerterbuch wie die gefundene Schrift, keine Abschrift.
    z("Sysalias zeigt auf Sysfonts", lambda: all(
        any(v is w for w in sf.Sysfonts.values())
        for v in sf.Sysalias.values()))
    # Jeder Name in jedem Schnitt - auch die, bei denen weder der
    # gewuenschte noch der Grundschnitt da ist und der erste eingetragene
    # gewinnt.
    #
    # Die mitgelieferte Schrift liegt bei **jeder** Huelle in ihrem eigenen
    # Paketordner - so will es pygames `join(pygame_folder,
    # "freesansbold.ttf")`. Der Ordner wird darum als `<paket>` geschrieben;
    # verglichen wird, **dass** es der eigene ist.
    paket = os.path.normcase(os.path.dirname(os.path.abspath(pg.__file__)))

    def ort(p):
        if p and os.path.normcase(os.path.dirname(p)) == paket:
            return "<paket>/" + os.path.basename(p)
        return p
    z("match_font jeder Name jeder Schnitt", lambda: tuple(
        (n, b, i, ort(sf.match_font(n, b, i)))
        for n in sorted(sf.Sysfonts) + sorted(sf.Sysalias)
        for b in (False, True) for i in (False, True)))
    z("Sysfonts freesansbold", lambda: {
        k: ort(v) for k, v in sf.Sysfonts.get("freesansbold", {}).items()})
    z("initsysfonts gibt Sysfonts",
      lambda: sf.initsysfonts() is sf.Sysfonts)
    z("initsysfonts Rueckgabe", lambda: repr(sf.initsysfonts()))

    # Die drei Plattformsuchen lesen **neu**, geben ein eigenes
    # Woerterbuch und fassen `Sysfonts` nicht an; `freesansbold` haengt
    # erst `initsysfonts()` an.
    def plattform(name):
        import warnings
        with warnings.catch_warnings(record=True) as w:
            warnings.simplefilter("always")
            r = getattr(sf, name)()
        return (type(r).__name__, len(r), r is sf.Sysfonts,
                "freesansbold" in r, tuple(sorted(r)) == tuple(
                    n for n in sorted(sf.Sysfonts) if n != "freesansbold"),
                tuple(str(x.message) for x in w))
    for name in ("initsysfonts_win32", "initsysfonts_darwin",
                 "initsysfonts_unix"):
        z(name, lambda name=name: plattform(name))
    return tuple(aus)


def kleinkram(pg):
    """Einzelne Namen, die nie gerufen wurden - jeder fuer sich zu klein
    fuer eine eigene Probe, zusammen zwanzig.

    Aus der Liste von `tools/abdeckung.py`: `cursors.compile`,
    `cursors.load_xbm`, `imageext.load_extended`, `imageext.save_extended`,
    `encode_file_path`, `encode_string`, `Surface.get_blendmode`,
    `Cursor.copy`, `newbuffer.BufferMixin`, `newbuffer.Py_buffer`,
    `version.PygameVersion`, `version.SDLVersion`,
    `version.SoftwareVersion` samt `count` und `index`.
    """
    import importlib
    import os
    import tempfile

    aus = []

    def z(name, f):
        try:
            aus.append((name, f()))
        except Exception as e:                      # noqa: BLE001
            aus.append((name, "%s: %s" % (type(e).__name__, e)))

    # --- cursors.compile / load_xbm ---
    cur = importlib.import_module(pg.__name__ + ".cursors")
    bild = ("XXXXXXXX", "X......X", "X.XXXX.X", "X.X..X.X",
            "X.X..X.X", "X.XXXX.X", "X......X", "XXXXXXXX")
    z("cursors.compile", lambda: cur.compile(bild))
    z("cursors.compile black=.", lambda: cur.compile(bild, black=".",
                                                     white="X"))
    z("cursors.compile xor",
      lambda: cur.compile(bild, black="X", white=".", xor="o"))
    z("cursors.compile krumm", lambda: cur.compile(("XXX", "XXX")))
    xbm = (b"#define t_width 8\n#define t_height 8\n"
           b"static unsigned char t_bits[] = {\n"
           b"0xff,0x81,0xbd,0xa5,0xa5,0xbd,0x81,0xff};\n")
    z("cursors.load_xbm", lambda: cur.load_xbm(
        __import__("io").BytesIO(xbm), __import__("io").BytesIO(xbm))[0])
    z("Cursor.copy", lambda: repr(cur.Cursor(pg.SYSTEM_CURSOR_ARROW).copy()))
    z("Cursor.copy ist gleich",
      lambda: cur.Cursor(pg.SYSTEM_CURSOR_HAND).copy()
      == cur.Cursor(pg.SYSTEM_CURSOR_HAND))

    # --- imageext ---
    ie = importlib.import_module(pg.__name__ + ".imageext")
    s = pg.Surface((6, 4))
    s.fill((10, 20, 30))
    pfad = os.path.join(tempfile.gettempdir(), "rustgame_probe_ie.png")
    z("imageext.save_extended", lambda: ie.save_extended(s, pfad))
    z("imageext.load_extended Groesse",
      lambda: ie.load_extended(pfad).get_size())
    z("imageext.load_extended Pixel",
      lambda: tuple(ie.load_extended(pfad).get_at((1, 1))))
    z("imageext.load_extended fehlt",
      lambda: ie.load_extended("gibtesnicht_rustgame.png"))

    # --- encode_file_path / encode_string ---
    z("encode_file_path", lambda: pg.encode_file_path("abc.txt"))
    z("encode_file_path bytes", lambda: pg.encode_file_path(b"abc.txt"))
    z("encode_file_path None", lambda: pg.encode_file_path(None))
    z("encode_string", lambda: pg.encode_string("abc"))
    z("encode_string latin-1",
      lambda: pg.encode_string("abc", "latin-1", "strict"))
    z("encode_string Fehler",
      lambda: pg.encode_string("\u20ac", "ascii", "strict", ValueError))

    # --- Surface.get_blendmode ---
    z("Surface.get_blendmode", lambda: s.get_blendmode())
    z("Surface.get_blendmode SRCALPHA",
      lambda: pg.Surface((2, 2), pg.SRCALPHA).get_blendmode())

    # --- newbuffer ---
    nb = importlib.import_module(pg.__name__ + ".newbuffer")
    z("BufferMixin Typ", lambda: type(nb.BufferMixin()).__name__)
    z("BufferMixin memoryview",
      lambda: memoryview(nb.BufferMixin()).nbytes)
    z("Py_buffer Felder", lambda: tuple(
        (n, getattr(nb.Py_buffer(), n)) for n in
        ("buf", "obj", "len", "itemsize", "readonly", "ndim", "format",
         "shape", "strides", "suboffsets", "internal")))

    # --- version ---
    ver = importlib.import_module(pg.__name__ + ".version")
    z("PygameVersion Felder", lambda: tuple(ver.vernum._fields))
    z("PygameVersion count", lambda: ver.vernum.count(ver.vernum[0]))
    z("PygameVersion index", lambda: ver.vernum.index(ver.vernum[1]))
    z("SDLVersion Felder", lambda: tuple(ver.SDL._fields))
    z("SDLVersion count", lambda: ver.SDL.count(ver.SDL[0]) >= 1)
    z("SoftwareVersion Typ",
      lambda: type(ver.PygameVersion(1, 2, 3, "rc1")).__name__)
    z("SoftwareVersion str", lambda: str(ver.PygameVersion(1, 2, 3, "rc1")))
    z("SDLVersion str", lambda: str(ver.SDLVersion(2, 3, 4)))
    return tuple(aus)


def fehlertexte(pg):
    """Was sagt jede Funktion, wenn ihr Argument die **falsche Sorte** hat?

    ## Die siebzehnte Frage

    `Argumentfehler` ruft mit siebzehn Argumenten und prueft damit die
    **Zahl**. Diese Probe ruft mit **einem** Argument der falschen Sorte
    und prueft den Satz - zwei verschiedene Wege durch denselben
    Argumentleser, und der zweite hat Dinge gefunden, die der erste nicht
    sehen konnte:

    | Fund | was es war |
    |---|---|
    | **86 deutsche Fehlertexte** | `Unbekanntes Klangformat - erwartet wird WAV, OGG, FLAC oder MP3` und 85 weitere - in Ausnahmen, die ein Mensch zu sehen bekommt |
    | **51 deutsche Argumentnamen** | `argument 'winkel': must be real number` - PyO3 haengt den Rust-Namen davor |
    | `Surface.set_clip`, `display.update` | `TypeError` statt `ValueError` - ein `except ValueError` sah den Fehler gar nicht |
    | `Vector.dot`, `Vector3.cross`, `angle_to` | `ValueError: Sequence has the wrong length.` statt pygames `TypeError` mit eigenem Satz |
    | `Mask.get_at/set_at/scale`, `display.set_mode` | **ein** Helfer sagte ueberall `offset`, pygame sagt `pos`, `scale`, `size` |
    | `Surface.get_rect`, `get_frect`, `Mask.get_rect` | pygames `only accepts keyword arguments` (bei der Maske: `supports`) |
    | `time.delay`, `time.wait`, `mixer_music.set_pos`, `Surface.unmap_rgb` | pygame hat fuer jede einen eigenen Satz |
    | `mouse.set_visible`, `mixer.find_channel` | pygame liest dort eine **Zahl**, nicht einen Wahrheitswert |
    | `display.set_icon`, `event.post`, `set_window_position` | PyO3 nannte Rust-Typnamen (`'PyTuple'`, `'Event'`) |
    | `image.load` | unser Satz traegt die Liste der gelesenen Formate; pygames nicht |

    ## Was verglichen wird - und was nicht

    Die **Art** der Ausnahme immer: sie entscheidet, ob ein `except` den
    Fehler faengt, und ist damit das Einzige an einer Fehlermeldung, auf
    das sich ein Programm verlassen darf.

    Der **Wortlaut** nur dort, wo pygame ihn selbst geschrieben hat. Die
    beiden Argumentleser - `PyArg_ParseTuple` in C und PyO3s erzeugter
    Vorbau - sagen dasselbe mit anderen Worten:

    ```text
    pygame    function takes at most 4 arguments (17 given)
    rustgame  Surface._init() takes from 1 to 4 positional arguments but 17 were given

    pygame    argument 1 must be pygame.surface.Surface, not str
    rustgame  rect() missing 2 required positional arguments: 'color' and 'rect'
    ```

    Beides ist derselbe Einwand - "mit diesen Argumenten geht das nicht" -
    und beide Saetze stammen nicht von pygame, sondern von seinem
    Werkzeug. [`_satz_gestalt`] erkennt sie an ihrer Form und macht aus
    jedem die Marke `<argumentfehler>`; was danach noch verschieden ist,
    hat jemand hingeschrieben.

    **Die Erkennung ist eine Regel, keine Liste** - genau deshalb faellt
    ein neuer handgeschriebener Satz weiterhin auf.
    """
    import importlib

    marke = "argname"
    aus = []
    for n in [""] + _alle_laden(pg):
        voll = pg.__name__ + ("." + n if n else "")
        try:
            m = importlib.import_module(voll)
        except Exception:                            # noqa: BLE001
            continue
        for a in sorted(dir(m)):
            if a.startswith("_"):
                continue
            w = getattr(m, a, None)
            if not callable(w) or isinstance(w, type):
                continue
            weg = "%s.%s" % (n or "(haupt)", a)
            if weg in _FEHLERTEXT_AUS:
                continue
            aus.append((weg,) + _einmal_falsch(w, marke, pg))
    # Methoden an Instanzen, die ohne Geraet zu bauen sind.
    bauen = [
        ("Vector2", lambda: pg.Vector2(1, 2)),
        ("Vector3", lambda: pg.Vector3(1, 2, 3)),
        ("Rect", lambda: pg.Rect(1, 2, 3, 4)),
        ("FRect", lambda: pg.FRect(1, 2, 3, 4)),
        ("Color", lambda: pg.Color(1, 2, 3, 4)),
        ("Surface", lambda: pg.Surface((4, 4))),
        ("Mask", lambda: pg.mask.Mask((4, 4))),
    ]
    for kl, f in bauen:
        try:
            o = f()
        except Exception:                            # noqa: BLE001
            continue
        for a in sorted(dir(o)):
            if a.startswith("_"):
                continue
            w = getattr(o, a, None)
            if not callable(w):
                continue
            weg = "%s.%s" % (kl, a)
            if weg in _FEHLERTEXT_AUS:
                continue
            aus.append((weg,) + _einmal_falsch(w, marke, pg))
    if len(aus) < 400:
        raise RuntimeError("Fehlertexte: nur %d Eintraege" % len(aus))
    return tuple(aus)


#: Stellen, an denen der Satz **nicht** vergleichbar ist.
#:
#: `pkgdata.getResource` nennt den Ordner des Pakets, und der ist
#: notwendig verschieden. `camera.list_cameras` ist bei pygame vor
#: `camera.init()` ein Platzhalter mit eigenem Namen
#: (`_pre_init_placeholder()`), hier eine echte Funktion - der
#: Unterschied steht bei `camera.py`.
#: `print_debug_info` ist ausgenommen, weil pygame **in die Datei
#: schreibt**, deren Namen man ihm gibt: ein Lauf dieser Probe hinterliess
#: eine Datei `argname` im Arbeitsverzeichnis. *Eine Probe, die die Welt
#: aendert, ist keine Probe.* (Nebenbei die Antwort auf die Abweichung
#: dort: pygame nimmt jeden Namen an, hier wird er geprueft.)
_FEHLERTEXT_AUS = frozenset({
    "pkgdata.getResource", "camera.list_cameras", "_camera.list_cameras",
    "newbuffer.Py_buffer", "print_debug_info", "_debug.print_debug_info",
    "(haupt).print_debug_info",
})

#: Saetze, die **der Argumentleser** schreibt, nicht pygame.
#:
#: Beide Huellen haben einen, und die zwei sagen dasselbe mit anderen
#: Worten. Siehe [`fehlertexte`], warum daraus eine Marke wird.
_MASCHINE = (
    r"takes (?:exactly|at most|at least) \d+ (?:positional )?arguments?",
    r"takes \d+ positional arguments? but",
    r"takes from \d+ to \d+ positional arguments",
    r"takes no arguments",
    r"takes no keyword arguments",
    r"missing \d+ required positional argument",
    r"function missing required argument",
    r"argument \d+ must be",
    r"got an unexpected keyword argument",
    r"got some positional-only arguments",
    r"takes \d+ positional arguments?",
    # **Auch das ist ein Maschinensatz**, und zwar PyO3s: er steht da,
    # wo pygames `PyArg_ParseTuple` `argument 1 must be T, not str`
    # sagt. Beide nennen die erwartete Sorte, beide stammen vom
    # Werkzeug. Ohne diese Zeile meldete die Probe neun Stellen
    # (`transform.grayscale`, `mask.from_surface`, ...), an denen
    # nichts zu beheben war.
    r"object cannot be converted to",
    r"object cannot be interpreted as an integer",
    r"must be real number, not",
    # **Die zwei Rekursionswaechter.** pygame meldet `maximum recursion
    # depth exceeded` (CPythons Zaehler), die Huelle `Stack overflow (used
    # 2912 kB) while calling a Python object` (PyO3s Waechter, mit einer
    # Zahl, die von Lauf zu Lauf wechselt). Dieselbe Sorte Satz wie die
    # Argumentleser darueber: er stammt von der Laufzeit, nicht von
    # pygame. Gemessen an sechzehn Stellen (`Group.add('x')` und
    # Geschwister) - und die **Art** ist bei beiden `RecursionError`.
    r"maximum recursion depth exceeded",
    r"Stack overflow \(used \d+ kB\)",
)


def _satz_gestalt(text, paket):
    """Der Satz, auf das Vergleichbare gebracht - siehe [`fehlertexte`]."""
    import re

    # **Beide Paketnamen.** Manche Saetze nennen pygame **woertlich**,
    # weil pygame sie so schreibt (`pygame.midi not initialised.`); ohne
    # die zweite Zeile wurde aus pygames Satz `@.midi` und aus unserem
    # `pygame.midi` - eine Abweichung, die es nicht gab.
    text = text.replace(paket, "@").replace("pygame", "@")
    # PyO3 haengt den Namen des Parameters davor; pygame nicht. Der Name
    # selbst wird von der Probe `Schluesselwoerter` geprueft.
    m = re.match(r"argument '[^']+': (.*)$", text, re.S)
    if m:
        text = m.group(1)
    for muster in _MASCHINE:
        if re.search(muster, text):
            return "<argumentfehler>"
    return text


def _einmal_falsch(w, marke, pg):
    """`(Ausnahmeart, Satzgestalt)` fuer einen Ruf mit falscher Sorte."""
    try:
        w(marke)
        return ("KEIN FEHLER", "")
    except BaseException as e:                       # noqa: BLE001
        return (type(e).__name__, _satz_gestalt(str(e), pg.__name__))


def ftfont_proben(pg):
    """`pygame.ftfont.Font` - **sechzehn Namen, die nie gelaufen waren.**

    `pygame.ftfont` ist pygames FreeType-Ersatz fuer `pygame.font`: eine
    Python-Klasse, die `pygame.freetype.Font` so verkleidet, dass sie wie
    `pygame.font.Font` aussieht. `font_test.py` fahrt sie nur, wenn
    `PYGAME_USE_FREETYPE` gesetzt ist - hier war es das nie, und
    `tools/abdeckung.py` hat die Luecke genannt.

    ## Was verglichen wird, und warum nicht die Zahlen

    Die Glyphenmasse weichen ab - FreeType rastet die Kontur aufs
    Pixelgitter ein, fontdue nicht. Das ist eine benannte Grenze
    (`font.metrics glyphenbox`), und sie noch einmal zu messen bringt
    nichts. Verglichen werden darum die **Schalter**, die **Formen** und
    die **Beziehungen** zwischen den Massen:

    ```text
    get_linesize() == get_height() - 1
    size(t)[1] == get_height()
    render("").get_height() == get_linesize()
    get_descent() < 0 < get_ascent()
    ```

    Die letzte Beziehung hat beim ersten Lauf **zwei echte Fehler**
    gefunden, und beide waren Formeln, keine Rasterkanten:

    | Fund | pygame | hier |
    |---|---|---|
    | `freetype.get_rect("")` | `(0, 20, 0, 25)` = `asc - desc` | `(0, 20, 0, 26)` = Zeilenhoehe |
    | `freetype.render("")` | `26` = `get_sized_height()` | `27` = `get_linesize()` der Schriftkiste |

    Beim zweiten widersprachen sich **zwei Zahlen in der eigenen Huelle**,
    die dasselbe heissen. Bei leerem Text gibt es nichts zu rastern - ein
    Unterschied dort ist nie eine Rasterkante, sondern immer eine andere
    Rechnung. *Eine Beziehung zu pruefen findet, was ein Zahlenvergleich
    unter einer benannten Grenze begraebt.*
    """
    import importlib

    ft = importlib.import_module(pg.__name__ + ".ftfont")
    aus = []

    def z(name, f):
        try:
            aus.append((name, f()))
        except Exception as e:                      # noqa: BLE001
            aus.append((name, "%s: %s" % (type(e).__name__, e)))

    z("Modulnamen", lambda: tuple(n for n in sorted(dir(ft))
                                  if not n.startswith("_")))
    z("Klassennamen", lambda: tuple(n for n in sorted(dir(ft.Font))
                                    if not n.startswith("_")))
    z("get_default_font", ft.get_default_font)
    z("Font ist FontType", lambda: ft.Font is ft.FontType)
    f = ft.Font(None, 24)
    # --- die vier Schalter, hin und zurueck -----------------------------
    for name in ("bold", "italic", "underline", "strikethrough"):
        z("%s Vorgabe" % name, lambda n=name: getattr(f, "get_" + n)())
        z("%s an" % name, lambda n=name: (getattr(f, "set_" + n)(True),
                                          getattr(f, "get_" + n)())[1])
        z("%s aus" % name, lambda n=name: (getattr(f, "set_" + n)(False),
                                           getattr(f, "get_" + n)())[1])
    # --- die Beziehungen zwischen den Massen ----------------------------
    z("linesize == height - 1",
      lambda: f.get_linesize() == f.get_height() - 1)
    z("descent < 0 < ascent",
      lambda: f.get_descent() < 0 < f.get_ascent())
    z("ascent - descent >= height",
      lambda: f.get_ascent() - f.get_descent() >= f.get_height())
    z("size('') Breite 0", lambda: f.size("")[0] == 0)
    z("size Hoehe == get_height",
      lambda: tuple(f.size(t)[1] == f.get_height()
                    for t in ("", "Hi", "gjpqy", "WWWWW")))
    z("size waechst mit der Laenge",
      lambda: f.size("ii")[0] < f.size("iii")[0] < f.size("iiii")[0])
    z("render('') Hoehe == linesize",
      lambda: f.render("", True, (255, 255, 255)).get_height()
      == f.get_linesize())
    z("render('') Breite 0",
      lambda: f.render("", True, (255, 255, 255)).get_width() == 0)
    z("render Hoehe == get_height",
      lambda: f.render("Hi", True, (255, 255, 255)).get_height()
      == f.get_height())
    z("render Breite == size Breite",
      lambda: f.render("Hi", True, (255, 255, 255)).get_width()
      == f.size("Hi")[0])
    z("render Alphaflagge",
      lambda: bool(f.render("Hi", True, (255, 255, 255)).get_flags()
                   & pg.SRCALPHA))
    z("render mit Grund Alphaflagge",
      lambda: bool(f.render("Hi", True, (255, 255, 255), (0, 0, 0))
                   .get_flags() & pg.SRCALPHA))
    z("render Typ",
      lambda: type(f.render("Hi", True, (255, 255, 255))).__name__)
    # --- metrics -------------------------------------------------------
    z("metrics('') leer", lambda: f.metrics("") == [])
    z("metrics Laenge", lambda: tuple(len(f.metrics(t))
                                      for t in ("A", "AB", "ABC")))
    z("metrics Stueckzahl", lambda: len(f.metrics("A")[0]))
    z("metrics Sorten",
      lambda: tuple(type(x).__name__ for x in f.metrics("A")[0]))
    # --- Fehlerwege ----------------------------------------------------
    z("render(NUL)", lambda: f.render("a\x00b", True, (255, 255, 255)))
    # **Die Breite, nicht die Hoehe.** Die Hoehe ist `get_height()`,
    # und die weicht um die Glyphenbox ab (benannte Grenze); die
    # **Breite** ist die Aussage: pygame zaehlt das NUL-Byte als
    # Zeichen mit, statt die Zeichenkette dort abzuschneiden.
    z("size(NUL) nimmt NUL mit",
      lambda: f.size("a\x00b")[0] > f.size("ab")[0])
    z("size(NUL) Hoehe == get_height",
      lambda: f.size("a\x00b")[1] == f.get_height())
    z("Font(groesse 0)", lambda: ft.Font(None, 0).get_height() > 0)
    z("Unterklasse",
      lambda: type("U", (ft.Font,), {})(None, 12).get_height() > 0)
    return tuple(aus)


def sprite_gruppen(pg):
    """Die drei Gruppenklassen, die nie gelaufen waren.

    `tools/abdeckung.py` nannte `GroupSingle`, `LayeredUpdates` und
    `LayeredDirty` mit je sechs ungerufenen Namen, dazu `Sprite.update`,
    `AbstractGroup()` und die zwei `collide_*_ratio`-Fabriken. Alles
    reines Python und woertlich aus `pygame/sprite.py` uebernommen - also
    muessen die Werte **genau** stimmen, nicht nur ungefaehr.

    Verglichen wird, was ein Spiel von einer Gruppe sieht: wer drin ist,
    in welcher Reihenfolge, auf welcher Ebene, und welche Rechtecke ein
    `draw` zurueckmeldet.
    """
    import importlib

    sp = importlib.import_module(pg.__name__ + ".sprite")
    aus = []

    def z(name, f):
        try:
            aus.append((name, f()))
        except Exception as e:                      # noqa: BLE001
            aus.append((name, "%s: %s" % (type(e).__name__, e)))

    def figur(x, y, w=4, h=4):
        s = sp.Sprite()
        s.image = pg.Surface((w, h))
        s.image.fill((200, 100, 50))
        s.rect = pg.Rect(x, y, w, h)
        return s

    # --- Sprite --------------------------------------------------------
    z("Sprite() Gruppen", lambda: sp.Sprite().groups())
    z("Sprite().alive", lambda: sp.Sprite().alive())
    z("Sprite.update nimmt alles",
      lambda: sp.Sprite().update(1, 2, drei=3))
    z("AbstractGroup leer", lambda: (len(sp.AbstractGroup()),
                                     sp.AbstractGroup().sprites()))
    # --- GroupSingle ---------------------------------------------------
    a, b = figur(0, 0), figur(10, 10)
    g = sp.GroupSingle(a)
    z("GroupSingle Laenge", lambda: len(g))
    z("GroupSingle sprites", lambda: [s is a for s in g.sprites()])
    z("GroupSingle hat a", lambda: g.has_internal(a))
    z("GroupSingle hat b", lambda: g.has_internal(b))
    z("GroupSingle tauscht", lambda: (g.add_internal(b),
                                      len(g), g.sprites()[0] is b)[1:])
    z("GroupSingle a ist draussen", lambda: a.alive())
    z("GroupSingle copy Typ", lambda: type(g.copy()).__name__)
    z("GroupSingle copy Inhalt", lambda: g.copy().sprites()[0] is b)
    z("GroupSingle entfernt", lambda: (g.remove_internal(b), len(g))[1])
    z("GroupSingle sprite leer", lambda: g.sprite)
    # --- LayeredUpdates ------------------------------------------------
    lu = sp.LayeredUpdates()
    c, d, e = figur(0, 0), figur(5, 5), figur(9, 9)
    lu.add(c, layer=2)
    lu.add(d, layer=0)
    lu.add(e)
    z("LayeredUpdates Ebenen", lambda: lu.layers())
    z("LayeredUpdates Reihenfolge",
      lambda: [lu.get_layer_of_sprite(s) for s in lu.sprites()])
    z("LayeredUpdates get_sprite",
      lambda: lu.get_sprite(0) is lu.sprites()[0])
    z("LayeredUpdates get_top", lambda: lu.get_top_sprite() is lu.sprites()[-1])
    z("LayeredUpdates unten/oben",
      lambda: (lu.get_bottom_layer(), lu.get_top_layer()))
    z("LayeredUpdates wechselt Ebene",
      lambda: (lu.change_layer(d, 5), lu.get_layer_of_sprite(d))[1])
    z("LayeredUpdates Ebene raeumen",
      lambda: (lu.remove_sprites_of_layer(5), len(lu))[1])
    z("LayeredUpdates draw Rechtecke",
      lambda: [tuple(r) for r in lu.draw(pg.Surface((20, 20)))])
    z("LayeredUpdates remove_internal",
      lambda: (lu.remove_internal(c), len(lu))[1])
    z("LayeredUpdates sprites_at",
      lambda: [tuple(s.rect) for s in lu.get_sprites_at((9, 9))])
    # --- LayeredDirty --------------------------------------------------
    ld = sp.LayeredDirty()
    h1 = sp.DirtySprite()
    h1.image = pg.Surface((4, 4))
    h1.image.fill((10, 200, 30))
    h1.rect = pg.Rect(2, 2, 4, 4)
    ld.add(h1)
    z("LayeredDirty Laenge", lambda: len(ld))
    z("LayeredDirty get_clip", lambda: ld.get_clip())
    z("LayeredDirty set_clip",
      lambda: (ld.set_clip(pg.Rect(0, 0, 10, 10)),
               tuple(ld.get_clip()))[1])
    z("LayeredDirty Schwelle",
      lambda: (ld.set_timing_threshold(800.0),
               ld.set_timing_treshold(900.0))[0])
    z("LayeredDirty draw Rechtecke",
      lambda: [tuple(r) for r in ld.draw(pg.Surface((20, 20)))])
    z("LayeredDirty clear",
      lambda: (ld.clear(pg.Surface((20, 20)), pg.Surface((20, 20))),
               len(ld))[1])
    z("LayeredDirty repaint",
      lambda: (ld.repaint_rect(pg.Rect(0, 0, 2, 2)), len(ld))[1])
    # --- die zwei Fabriken ---------------------------------------------
    k1, k2 = figur(0, 0, 10, 10), figur(5, 5, 10, 10)
    k1.radius, k2.radius = 5, 5
    z("collide_rect_ratio(1.0)",
      lambda: sp.collide_rect_ratio(1.0)(k1, k2))
    z("collide_rect_ratio(0.1)",
      lambda: sp.collide_rect_ratio(0.1)(k1, k2))
    z("collide_circle_ratio(1.0)",
      lambda: sp.collide_circle_ratio(1.0)(k1, k2))
    z("collide_circle_ratio(0.1)",
      lambda: sp.collide_circle_ratio(0.1)(k1, k2))
    z("collide_rect_ratio Typ",
      lambda: type(sp.collide_rect_ratio(1.0)).__name__)
    # --- die uebrigen Gruppen, einmal angelegt --------------------------
    for name in ("Group", "RenderPlain", "RenderClear", "RenderUpdates",
                 "OrderedUpdates"):
        z("%s()" % name, lambda n=name: (len(getattr(sp, n)(figur(0, 0))),
                                         type(getattr(sp, n)()).__name__))
    return tuple(aus)


def kanal_proben(pg):
    """`mixer.Channel` - **fuenf Namen, die nie gelaufen waren.**

    `fadeout`, `pause`, `unpause`, `queue` und `get_queue`. Alle fuenf
    gehen ohne Tongeraet, weil der Mixer unter dem Dummy-Treiber laeuft -
    es hatte nur nie jemand gefragt. Der letzte erreichbare Posten aus der
    Liste von `tools/abdeckung.py`.

    Gemessen wird der **Zustand**, nicht die Zeit: ob ein Kanal nach
    `pause()` noch beschaeftigt ist, was `get_queue()` vor und nach einem
    `queue()` sagt, und was ein `fadeout` auf einem stillen Kanal tut.
    Lautstaerken und Abspielstellen haengen an der Uhr und gehoeren nicht
    in einen Wertvergleich.
    """
    import importlib

    mx = importlib.import_module(pg.__name__ + ".mixer")
    aus = []

    def z(name, f):
        try:
            aus.append((name, f()))
        except Exception as e:                      # noqa: BLE001
            aus.append((name, "%s: %s" % (type(e).__name__, e)))

    # Ein Klang aus Bytes - kein Dateizugriff, auf beiden Seiten gleich.
    import struct
    rahmen = 800
    daten = b"".join(struct.pack("<hh", 3000 if i % 80 < 40 else -3000,
                                 -3000 if i % 80 < 40 else 3000)
                     for i in range(rahmen))
    kopf = (b"RIFF" + struct.pack("<I", 36 + len(daten)) + b"WAVEfmt "
            + struct.pack("<IHHIIHH", 16, 1, 2, 44100, 44100 * 4, 4, 16)
            + b"data" + struct.pack("<I", len(daten)))
    z("Sound aus Bytes", lambda: type(mx.Sound(buffer=kopf + daten)).__name__)
    s = mx.Sound(buffer=kopf + daten)

    k = mx.Channel(0)
    z("Channel Typ", lambda: type(k).__name__)
    z("Channel(0) ist Channel(0)", lambda: mx.Channel(0) == mx.Channel(0))
    z("get_busy still", k.get_busy)
    z("get_sound still", k.get_sound)
    z("get_queue still", k.get_queue)
    z("pause still", lambda: (k.pause(), k.get_busy())[1])
    z("unpause still", lambda: (k.unpause(), k.get_busy())[1])
    z("fadeout still", lambda: (k.fadeout(10), k.get_busy())[1])
    z("stop still", lambda: (k.stop(), k.get_busy())[1])
    # --- mit Klang ------------------------------------------------------
    z("play gibt Channel", lambda: type(k.play(s)).__name__)
    z("get_sound nach play", lambda: k.get_sound() is s)
    z("queue leer vor queue", k.get_queue)
    z("queue setzt", lambda: (k.queue(s), k.get_queue() is s)[1])
    z("pause laeuft weiter beschaeftigt",
      lambda: (k.pause(), k.get_busy())[1])
    z("unpause", lambda: (k.unpause(), k.get_busy())[1])
    z("fadeout nimmt die Zahl", lambda: k.fadeout(5))
    # **Nicht die Objekte selbst** - ihr `repr` traegt die Adresse, und
    # die ist bei jedem Lauf eine andere. Gefragt ist, **ob** noch etwas
    # haengt.
    z("stop raeumt", lambda: (k.stop(), k.get_sound() is None,
                              k.get_queue() is None)[1:])
    z("set_volume Rundlauf",
      lambda: (k.set_volume(0.5), round(k.get_volume(), 6))[1])
    z("set_volume zwei Werte",
      lambda: (k.set_volume(0.25, 0.75), round(k.get_volume(), 6))[1])
    z("set_endevent Rundlauf",
      lambda: (k.set_endevent(pg.USEREVENT + 3), k.get_endevent())[1])
    z("set_endevent zurueck",
      lambda: (k.set_endevent(), k.get_endevent())[1])
    # --- Fehlerwege -----------------------------------------------------
    z("queue(kein Klang)", lambda: k.queue("x"))
    z("fadeout(Text)", lambda: k.fadeout("x"))
    z("Channel(-1)", lambda: mx.Channel(-1))
    z("Channel(999)", lambda: mx.Channel(999))
    return tuple(aus)


def objekte(pg):
    """Was kann man mit einem Objekt tun, **ausser seine Methoden zu rufen**?

    ## Die achtzehnte Frage

    Acht Runden haben die Funktionen gemessen: ihre Namen, ihre
    Stelligkeit, ihre Schluesselwoerter, ihre Fehlertexte, ob sie je
    gerufen wurden. Keine hat gefragt, was mit den **Objekten** geht:

    ```text
    hash(r)                 und damit {r: 1} und set([r])
    a == b, a < b           Gleichheit und Ordnung
    bool(a), len(a), list(a), 1 in a
    copy.copy, copy.deepcopy, pickle.dumps
    o.feld = x              schreibbar oder nicht, und welcher Fehler
    ```

    Jedes davon trifft ein Programm, das die Bibliothek benutzt, **ohne
    eine einzige Methode falsch zu rufen** - ein `Rect` in einer Menge,
    ein `deepcopy` eines Spielzustands, ein `pickle` beim Speichern.

    ## Was der erste Lauf gefunden hat

    281 Messungen an zwölf Klassen, **fünfzehn Abweichungen**, und zehn
    davon an einer Stelle:

    | `pygame.cursors.Cursor` | pygame | hier (vorher) |
    |---|---|---|
    | `hash(c)` | geht | `TypeError: unhashable` |
    | `{c: 1}`, `set([c])` | geht | dito |
    | `copy.deepcopy(c)` | geht | `cannot pickle` |
    | `pickle.dumps(c)` | geht | dito |
    | `c.eigenes_feld = 1` | geht | `AttributeError` |

    pygames `Cursor` ist eine **gewoehnliche Python-Klasse**, und `type`
    und `data` liegen als Felder in ihrem `__dict__`; daraus folgt all
    das von selbst. Eine PyO3-Klasse hat nichts davon: mit `__eq__` und
    ohne `__hash__` ist sie unhashbar (genau wie eine Python-Klasse es
    waere), und ohne `__reduce__` laesst sie sich nicht einfrieren.
    Behoben mit `dict`, `__hash__`, `__reduce__` und `__deepcopy__`.

    ## Warum die Fehlertexte normalisiert werden

    Fast jeder Satz hier nennt den Typ, und der traegt den Paketnamen
    (`cannot use 'pygame.rect.Rect' as a dict key`). Ohne Normalisierung
    waeren 40 der 53 ersten Meldungen Scheintreffer - und einmal hat das
    Messgeraet noch schlimmer getrogen: es kuerzte die Texte auf 70
    Zeichen **vor** dem Ersetzen, und weil `pygame` sechs Zeichen und
    `@` eines ist, standen die zwei Seiten an verschiedenen Stellen ab.
    *Wer kuerzt, kuerzt zuletzt.*
    """
    import copy
    import importlib
    import pickle

    aus = []

    def z(name, f):
        try:
            aus.append((name, repr(f())))
        except BaseException as e:                   # noqa: BLE001
            # Erst der ganze Satz, dann der Paketname heraus - nicht
            # umgekehrt, und nie gekuerzt.
            t = "%s: %s" % (type(e).__name__, e)
            aus.append((name, t.replace(pg.__name__, "@")
                        .replace("pygame", "@")))

    m = importlib.import_module(pg.__name__ + ".mask")
    ge = importlib.import_module(pg.__name__ + ".geometry")
    s = pg.Surface((4, 4))
    s.fill((10, 20, 30))
    muster = [
        ("Rect", pg.Rect(1, 2, 3, 4), pg.Rect(1, 2, 3, 4),
         pg.Rect(5, 6, 7, 8)),
        ("FRect", pg.FRect(1.5, 2.5, 3, 4), pg.FRect(1.5, 2.5, 3, 4),
         pg.FRect(9, 9, 1, 1)),
        ("Color", pg.Color(1, 2, 3, 4), pg.Color(1, 2, 3, 4),
         pg.Color(9, 8, 7, 6)),
        ("Vector2", pg.Vector2(1, 2), pg.Vector2(1, 2), pg.Vector2(3, 4)),
        ("Vector3", pg.Vector3(1, 2, 3), pg.Vector3(1, 2, 3),
         pg.Vector3(4, 5, 6)),
        ("Event", pg.event.Event(pg.USEREVENT, a=1),
         pg.event.Event(pg.USEREVENT, a=1),
         pg.event.Event(pg.USEREVENT + 1, a=2)),
        ("Surface", s, pg.Surface((4, 4)), pg.Surface((8, 8))),
        ("Mask", m.Mask((4, 4)), m.Mask((4, 4)), m.Mask((8, 8))),
        ("Circle", ge.Circle(1, 2, 3), ge.Circle(1, 2, 3),
         ge.Circle(4, 5, 6)),
        ("Line", ge.Line(1, 2, 3, 4), ge.Line(1, 2, 3, 4),
         ge.Line(5, 6, 7, 8)),
        ("Clock", pg.time.Clock(), pg.time.Clock(), pg.time.Clock()),
        ("Cursor", pg.cursors.Cursor(pg.SYSTEM_CURSOR_ARROW),
         pg.cursors.Cursor(pg.SYSTEM_CURSOR_ARROW),
         pg.cursors.Cursor(pg.SYSTEM_CURSOR_HAND)),
    ]
    for name, a, b, c in muster:
        z("%s a == b" % name, lambda a=a, b=b: a == b)
        z("%s a == c" % name, lambda a=a, c=c: a == c)
        z("%s a != c" % name, lambda a=a, c=c: a != c)
        z("%s a < c" % name, lambda a=a, c=c: a < c)
        z("%s bool" % name, lambda a=a: bool(a))
        z("%s len" % name, lambda a=a: len(a))
        z("%s list" % name, lambda a=a: list(a))
        z("%s in" % name, lambda a=a: 1 in a)
        z("%s hash gleich" % name, lambda a=a, b=b: hash(a) == hash(b))
        z("%s set-Groesse" % name, lambda a=a, b=b, c=c: len({a, b, c}))
        z("%s als Schluessel" % name, lambda a=a: {a: 1}[a])
        z("%s copy" % name, lambda a=a: copy.copy(a) == a)
        z("%s copy Typ" % name, lambda a=a: type(copy.copy(a)).__name__)
        z("%s deepcopy" % name, lambda a=a: copy.deepcopy(a) == a)
        z("%s pickle" % name,
          lambda a=a: pickle.loads(pickle.dumps(a)) == a)
        z("%s pickle Typ" % name,
          lambda a=a: type(pickle.loads(pickle.dumps(a))).__name__)
        # **Jedes Feld mit seinem eigenen Wert zurueckschreiben.** Das
        # aendert nichts und sagt trotzdem, ob es schreibbar ist - und
        # welchen Fehler es sonst gibt.
        for fn in [n for n in sorted(dir(a)) if not n.startswith("_")]:
            try:
                wert = getattr(a, fn)
            except BaseException:                    # noqa: BLE001
                continue
            if callable(wert):
                continue
            z("%s.%s schreibbar" % (name, fn),
              lambda a=a, fn=fn, wert=wert: (setattr(a, fn, wert), "ok")[1])
        z("%s neues Feld" % name,
          lambda a=a: (setattr(a, "gibtesnicht", 1), "ok")[1])
    if len(aus) < 200:
        raise RuntimeError("Objekte: nur %d Messungen" % len(aus))
    return tuple(aus)


#: Wie eine Instanz zu bauen ist - nur, was ohne Geraet geht.
_ERBEN_BAU = {
    "Rect": (1, 2, 3, 4), "FRect": (1.0, 2.0, 3.0, 4.0),
    "Color": (1, 2, 3, 4), "Vector2": (1, 2), "Vector3": (1, 2, 3),
    "Surface": ((4, 4),), "Mask": ((4, 4),), "Sprite": (),
    "Group": (), "GroupSingle": (), "LayeredUpdates": (),
    "LayeredDirty": (), "DirtySprite": (), "RenderPlain": (),
    "RenderClear": (), "RenderUpdates": (), "OrderedUpdates": (),
    "Clock": (), "Font": (None, 20), "Cursor": (),
    "Circle": (1, 2, 3), "Line": (1, 2, 3, 4), "Event": (32768,),
}


def erben(pg):
    """Laesst sich von dieser Klasse **ableiten** - und taugt die Ableitung?

    ## Die neunzehnte Frage

    `class Spieler(pygame.sprite.Sprite)` ist die haeufigste Zeile in
    jedem pygame-Spiel, und `class Bild(pygame.Surface)` steht in pygames
    eigener Doku. In PyO3 ist das ein **ausdruecklicher Schalter**
    (`subclass`); fehlt er, gibt eine Ableitung

        TypeError: type '@.newbuffer.Py_buffer' is not an acceptable
                   base type

    Neunzehn Runden lang hat keine Probe danach gefragt. Gemessen wird
    je Klasse: ableiten, die Unterklasse **bauen**, und ob sie sich
    verhaelt wie das Original (`isinstance`, ein eigenes Feld, ein neu
    angehaengtes).

    ## Was der erste Lauf gefunden hat

    Drei Klassen liessen sich hier **nicht** ableiten, bei pygame schon:
    `newbuffer.Py_buffer`, `sprite.collide_rect_ratio` und
    `collide_circle_ratio`. Die letzten beiden sind bei pygame
    gewoehnliche Python-Klassen (`class collide_rect_ratio:` mit
    `__call__`), hier PyO3-Klassen - und damit fehlte der Schalter.

    ## Und ein Fehler in pygame, der das Messen selbst gefaehrdet

    `setattr` auf einer Unterklasse von `pygame.Vector3` gibt dort

        SystemError: <built-in function setattr> returned a result with
                     an exception set

    und laesst eine `TypeError: a sequence is expected` im Interpreter
    stehen: der **naechste** Aufruf scheitert ebenfalls, auch ein
    schlichtes `print`. `Vector3`s Setzer liest die Zuweisung als Folge
    (fuer `v.xy = (1, 2)`), scheitert still und meldet trotzdem Erfolg.
    `Vector2` hat den Fehler nicht; hier geht beides sauber.

    Diese eine Messung ist darum ausgenommen - *ein Messgeraet, das das
    Messobjekt vergiftet, misst nichts mehr*, und zwar auch nichts
    danach.
    """
    import importlib

    aus = []

    def z(name, f):
        try:
            aus.append((name, repr(f())))
        except BaseException as e:                   # noqa: BLE001
            t = "%s: %s" % (type(e).__name__, e)
            aus.append((name, t.replace(pg.__name__, "@")
                        .replace("pygame", "@")))

    def klasse(weg):
        obj = pg
        for n in str(weg).split("."):
            try:
                obj = getattr(obj, n)
            except AttributeError:
                try:
                    obj = importlib.import_module(
                        "%s.%s" % (getattr(obj, "__name__", ""), n))
                except Exception:                    # noqa: BLE001
                    return None
        return obj if isinstance(obj, type) else None

    for weg in __import__("api_coverage").alle_klassen():
        k = klasse(weg)
        if k is None:
            continue
        kurz = str(weg).rsplit(".", 1)[-1]
        z("%s ableitbar" % weg,
          lambda k=k, kurz=kurz: type("U_" + kurz, (k,), {}).__name__)
        try:
            U = type("U_" + kurz, (k,), {"eigenes": 7})
        except BaseException:                        # noqa: BLE001
            continue
        if kurz not in _ERBEN_BAU:
            continue
        args = _ERBEN_BAU[kurz]
        z("%s Unterklasse bauen" % weg,
          lambda U=U, args=args: type(U(*args)).__name__)
        try:
            u = U(*args)
        except BaseException:                        # noqa: BLE001
            continue
        z("%s isinstance" % weg, lambda u=u, k=k: isinstance(u, k))
        z("%s eigenes Feld" % weg, lambda u=u: u.eigenes)
        # **`Vector3` nicht** - siehe den Kopf dieser Probe.
        if kurz != "Vector3":
            z("%s neues Feld" % weg,
              lambda u=u: (setattr(u, "zusatz", 1), u.zusatz)[1])
        z("%s Typname" % weg, lambda u=u: type(u).__name__)
    if len(aus) < 120:
        raise RuntimeError("Erben: nur %d Messungen" % len(aus))
    return tuple(aus)


def argumentfehler(pg):
    """Was sagt jede Funktion, wenn man sie **falsch** ruft?

    ## Die sechste Frage, die keiner gestellt hatte

    Alle Proben hier fragten, was eine Funktion tut, wenn man sie richtig
    ruft. Keine fragte nach dem haeufigsten Fall im Leben eines
    Programms: dem Fehlgriff. Jede oeffentliche Funktion und jede Klasse
    wird darum mit **siebzehn** Argumenten gerufen - siebzehn nimmt
    keine, also raeumt jede auf, bevor sie etwas tut, und der Aufruf ist
    nebenwirkungsfrei.

    Gefunden hat die Frage beim ersten Lauf:

    | Fund | was es war |
    |---|---|
    | `display.update` | `TypeError` statt `ValueError`, und "sequence of recstyles" statt "an iterable of rectstyles" - ein Tippfehler aus einer aelteren pygame-Fassung |
    | `display.update` ohne `set_mode` | pygame prueft den **Zustand vor den Argumenten** |
    | `math.clamp` & vier weitere | pygame schreibt eigene Saetze (`clamp requires 3 arguments`) |
    | `mouse.set_pos` | ein Satz fuer jede falsche Zahl von Argumenten |
    | `camera.AbstractCamera` | war keine echte abstrakte Klasse - `__init__` fehlte unter den Pflichtmethoden |
    | `typing.SequenceLike` | die beiden Methoden trugen kein `@abstractmethod` |
    | `geometry.Circle` | der Satz war nach einem Drittel abgeschnitten |
    | `_sdl2.error` | ist eine **eigene** Fehlerklasse, nicht `pygame.error` |
    | `_sdl2.RendererDriverInfo` | nahm fuenf Argumente; pygames nimmt keins |
    | `pkgdata.files` | war nachgebaut statt durchgereicht |
    | `BufferProxy()`, `Mask()` | der Wortlaut nannte die Zahl nicht mit |

    ## Was verglichen wird, und was nicht

    Verglichen wird die **Art** der Ausnahme fuer jede Funktion, und
    dazu, was ihr Text ueber die Stelligkeit sagt. Den Text selbst zu
    vergleichen waere falsch: pygames C-Funktionen melden
    `takes at most 4 arguments (17 given)`, PyO3 meldet
    `takes from 1 to 4 positional arguments but 17 were given` - dieselbe
    Aussage in anderen Worten. Aus beiden Formen wird darum die Zahl
    gelesen, und die Zahl wird verglichen.

    **Der Mindestwert nicht.** pygames `PyArg_ParseTuple` prueft die
    Anzahl, *bevor* es die Pflichtargumente prueft, und meldet darum
    `takes at most 9` auch dort, wo vier Pflicht sind. Die Zahl steht in
    keinem der beiden Texte verlaesslich; das Maximum in beiden.

    Ist pygames Text **keine** Zaehlmeldung, dann ist er handgeschrieben,
    und dann wird er wortgleich verglichen - das ist die Regel, mit der
    `clamp requires 3 arguments` gefunden wurde, ohne eine Liste von Hand
    zu pflegen.
    """
    import importlib

    viel = tuple(["\x00argfehler"] * 17)
    aus = []
    for n in [""] + _alle_laden(pg):
        voll = pg.__name__ + ("." + n if n else "")
        try:
            m = importlib.import_module(voll)
        except Exception:                            # noqa: BLE001
            continue
        for a in sorted(dir(m)):
            if a.startswith("_"):
                continue
            w = getattr(m, a, None)
            if not callable(w):
                continue
            weg = "%s.%s" % (n or "(haupt)", a)
            if weg in _ARGFEHLER_AUS:
                continue
            try:
                w(*viel)
                aus.append((weg, "KEIN FEHLER"))
            except RecursionError:
                # Die Tiefe haengt am C-Stapel und damit an der Hoehe des
                # Aufrufs - `sprite.Group(*17 Zeichenketten)` laeuft in
                # beiden Huellen in einen Ring, meldet aber einmal
                # "maximum recursion depth exceeded" und einmal
                # "Stack overflow (used 2912 kB)". Die Zahl ist nicht
                # wiederholbar; die Art ist es.
                aus.append((weg, "RecursionError"))
            except BaseException as e:               # noqa: BLE001
                # **Den Paketnamen herausnehmen.** Drei Saetze nennen ihn
                # mit (`cannot create 'pygame.math.VectorIterator'
                # instances`), und der ist hier notwendig verschieden -
                # das ist keine Abweichung, sondern der Name der Huelle.
                text = str(e).replace(pg.__name__, "@")
                aus.append((weg, type(e).__name__) + _stelligkeit(text))
    # Auch hier ein Waechter: gemessen sind es ueber fuenfhundert.
    if len(aus) < 400:
        raise RuntimeError("Argumentfehler: nur %d Eintraege" % len(aus))
    return tuple(aus)


# Funktionen, die beim Ruf mit siebzehn Argumenten **pygame zum Absturz
# bringen**: `pygame.newbuffer.Py_buffer(1)` faellt mit
# "(pygame parachute) Segmentation Fault" aus, und mit siebzehn
# Zeichenketten steht es auf der Kippe. Ein Messgeraet, das das Messobjekt
# umbringt, misst nichts.
_ARGFEHLER_AUS = frozenset({"newbuffer.Py_buffer"})

# Die Formen, in denen eine Zaehlmeldung auftritt - pygames
# `PyArg_ParseTuple`, Argument Clinic und PyO3.
_ZAEHLFORMEN = (
    (r"takes exactly (\d+) arguments?", lambda m: int(m.group(1))),
    (r"takes exactly one argument", lambda m: 1),
    (r"takes at most (\d+) arguments?", lambda m: int(m.group(1))),
    (r"takes at least (\d+) arguments?", lambda m: None),
    (r"takes from (\d+) to (\d+) positional arguments",
     lambda m: int(m.group(2))),
    (r"takes exactly (\d+) positional arguments?", lambda m: int(m.group(1))),
    (r"takes at most (\d+) positional arguments?", lambda m: int(m.group(1))),
    (r"takes (\d+) positional arguments?", lambda m: int(m.group(1))),
    (r"takes no arguments", lambda m: 0),
    (r"missing required argument", lambda m: None),
    (r"missing \d+ required positional argument", lambda m: None),
    (r"got an unexpected keyword argument", lambda m: None),
    (r"takes no keyword arguments", lambda m: None),
)


def _stelligkeit(text):
    """`("stellig", max)` oder `("wort", text)` - siehe [`argumentfehler`].

    Die Klammer sagt selbst, ob `self` mitgezaehlt wird: siebzehn
    Argumente wurden uebergeben, steht dort 18, dann zaehlt die Meldung
    `self` mit und das Maximum ist um eins zu hoch. PyO3s `__new__`
    zaehlt `cls` **nicht** mit, ein Python-`__init__` schon - geraten
    werden musste das nicht.
    """
    import re

    for muster, f in _ZAEHLFORMEN:
        m = re.search(muster, text)
        if not m:
            continue
        hoch = f(m)
        if hoch is None:
            return ("stellig", None)
        if "18 given" in text or "18 were given" in text or "19 were given" in text:
            hoch = max(0, hoch - 1)
        return ("stellig", hoch)
    return ("wort", text)


def schluesselwoerter(pg):
    """Nimmt jede Funktion den Namen an, den **pygame** fuer sie nennt?

    ## Der fuenfzehnte blinde Fleck

    `clamp() missing 3 required positional arguments: 'wert', 'min', and
    'max'` - drei deutsche Namen in einer oeffentlichen Schnittstelle,
    und `clamp(wert=1, min=0, max=2)` ging sogar. pygames C-Funktionen
    nehmen dort **ueberhaupt keine** Schluesselwoerter.

    Nachgezaehlt nahmen **58** Funktionen Namen an, die pygame ablehnt,
    und einundzwanzig dieser Namen waren deutsch: `wert`, `typ`, `daten`,
    `art`, `an`, `sichtbar`, `konstante`, `millisekunden`, `ziel`,
    `quelle`, `a_von`, `a_bis`, `b_von`, `b_bis`, `woher`, `freq`, `o`,
    `func`, `text`, `path`, `type_`. Dazu kam, dass einige englische
    Namen trotzdem die falschen waren (`file` statt `filename`, `arg`
    statt `o`).

    ## Zwei Messgeraete, die beide nicht taugten

    1. Ein **erfundenes** Schluesselwort (`zzz_gibt_es_nicht=1`) meldete
       91 Funktionen, darunter 33, die schon nur nach Stellung nehmen:
       PyO3 antwortet auf einen unbekannten Namen immer gleich, gleich
       wie die Signatur aussieht.
    2. Der **eigene** Name trennt die Faelle, aber dann beantworten die
       zwei Huellen zwei verschiedene Fragen - pygame wird nach `parent`
       gefragt und rustgame nach `woher`, und beide sagen "nimmt an".

    Die Frage muss also von **ausserhalb** kommen. Sie kommt aus pygames
    `.pyi`-Dateien: dort steht, welchen Namen jeder Parameter traegt.
    Damit bekommen beide Huellen dasselbe Schluesselwort vorgelegt, und
    drei Antworten sind zu unterscheiden:

    | Antwort | heisst |
    |---|---|
    | `nimmt an` | der Name geht als Schluesselwort durch |
    | `lehnt ab` | nur nach Stellung - pygames C-Weg |
    | `kennt den Namen nicht` | die Funktion hat einen **anderen** Namen |

    Damit faengt eine Probe beides: den falschen Namen und das zuviel
    angebotene Schluesselwort.
    """
    marke = "\x00kwfrage"
    aus = []
    for weg, namen in sorted(_stub_namen().items()):
        if not namen or weg in _ARGFEHLER_AUS:
            continue
        w = _am_weg(pg, weg)
        if w is None or not callable(w):
            continue
        name = namen[0]
        try:
            w(**{name: marke})
            aus.append((weg, name, "nimmt an"))
        except TypeError as e:
            t = str(e)
            if "positional-only" in t or "takes no keyword" in t:
                aus.append((weg, name, "lehnt ab"))
            elif "unexpected keyword argument" in t:
                aus.append((weg, name, "kennt den Namen nicht"))
            else:
                aus.append((weg, name, "nimmt an"))
        except BaseException:                        # noqa: BLE001
            aus.append((weg, name, "nimmt an"))
    # **Ein Waechter gegen die eigene Blindheit.** Diese Probe hat bei
    # ihrem ersten Lauf "gleich" gemeldet, und zwar aus dem schlechtesten
    # Grund: sie hatte **null** Eintraege. `io.open` gab einen NameError
    # (dieses Modul fuehrt kein `io`), der im `except` der Stubsuche
    # still verschwand - und zwei leere Tupel sind nun einmal gleich.
    # *Eine Probe, die nichts misst, ist immer gleich.* Darum eine
    # Untergrenze: gemessen sind es 213 Eintraege.
    if len(aus) < 150:
        raise RuntimeError(
            "Schluesselwoerter: nur %d Eintraege - die Stubs fehlen" % len(aus))
    return tuple(aus)


def _am_weg(pg, weg):
    """Das Ding, das in diesem Modul unter diesem Namen steht."""
    import importlib

    mod, _, name = weg.rpartition(".")
    voll = pg.__name__ + ("" if mod == "(haupt)" else "." + mod)
    try:
        m = importlib.import_module(voll)
    except Exception:                                # noqa: BLE001
        return None
    return getattr(m, name, None)


_STUBS = []


def _stub_namen():
    """`{modul.name: (param, ...)}` aus pygames `.pyi`-Dateien.

    **Von der Platte, nicht von Hand** - und aus pygames Paket, nicht aus
    unserem: die Frage, welchen Namen ein Parameter traegt, beantwortet
    pygame, nicht wir. Beide Huellen bekommen damit dieselbe Frage
    vorgelegt, was der ganze Zweck ist (siehe
    [`schluesselwoerter`]).

    Nur Modulfunktionen und Klassen, keine Methoden: der Weg einer
    Methode haengt an ihrer Klasse, und `inspect` kommt an die
    Methodennamen der C-Typen nicht heran. Was hier zaehlt, sind die 330
    Namen, die ein Programm direkt ruft.
    """
    if _STUBS:
        return _STUBS[0]
    import ast
    import os

    import pygame

    ordner = os.path.dirname(os.path.abspath(pygame.__file__))
    aus = {}
    for datei in sorted(os.listdir(ordner)):
        if not datei.endswith(".pyi"):
            continue
        mod = "(haupt)" if datei == "__init__.pyi" else datei[:-4]
        try:
            # `open`, nicht `io.open` - dieses Modul fuehrt kein `io`,
            # und der NameError verschwand still im `except` darunter:
            # die Probe meldete 0 Eintraege und "gleich".
            with open(os.path.join(ordner, datei), encoding="utf-8") as f:
                baum = ast.parse(f.read())
        except Exception:                            # noqa: BLE001
            continue
        for k in baum.body:
            if isinstance(k, ast.FunctionDef):
                a = k.args
                namen = [x.arg for x in a.posonlyargs + a.args + a.kwonlyargs]
                aus.setdefault("%s.%s" % (mod, k.name), tuple(namen))
            elif isinstance(k, ast.ClassDef):
                for kk in k.body:
                    if (isinstance(kk, ast.FunctionDef)
                            and kk.name == "__init__"):
                        a = kk.args
                        namen = [x.arg for x in
                                 a.posonlyargs + a.args + a.kwonlyargs]
                        if namen and namen[0] in ("self", "cls"):
                            namen = namen[1:]
                        aus.setdefault("%s.%s" % (mod, k.name), tuple(namen))
                        break
    _STUBS.append(aus)
    return aus


def klassennamen(pg):
    """Die **rufbaren** Namen jeder Klasse - in beide Richtungen.

    ## Warum nur die rufbaren

    `api_coverage.py` fragt, was pygame **hat**, und meldet Fehlendes. Ein
    **Ueberschuss** faellt dort nie auf, und er war da: `BufferProxy` trug
    zwei **deutsche** Namen - `laenge` und `aus_woerterbuch` -, weil zwei
    Hilfsrechnungen im `#[pymethods]`-Block standen. Ein
    `#[pymethods]`-Block ist die Schnittstelle; wer dort eine Hilfe ablegt,
    veroeffentlicht sie. Dazu hatte `camera.AbstractCamera` zwei abstrakte
    Methoden mehr als pygames und war damit **strenger** als das Vorbild.

    Verglichen werden nur die **rufbaren** Namen, und das ist eine
    Entscheidung mit Grund, nicht eine Bequemlichkeit: PyO3 legt ein Feld
    (`#[pyo3(get, set)]`) als Beschreiber an der **Klasse** ab, pygames
    Python-Klassen setzen dasselbe Feld im `__init__` an der **Instanz**.
    Gemessen betrifft das sechzehn Klassen und immer dieselbe Sorte Name -
    `sprite.Group.lostsprites`, `DirtySprite.dirty`,
    `collide_rect_ratio.ratio`. An der Instanz hat sie **beide** Seiten; an
    der Klasse nur rustgame. Das steht als benannte Grenze
    `Felder an der Klasse`, und es ist der Grund, warum diese Probe nach
    Methoden fragt und nicht nach allen Namen.

    Ein fehlende oder zusaetzliche **Methode** bricht ein Programm; ein
    Beschreiber an der Klasse, den auch die Instanz hat, nicht.
    """
    aus = []
    for weg in __import__("api_coverage").alle_klassen():
        k = _klasse(pg, weg)
        if k is None:
            aus.append((weg, "fehlt"))
            continue
        aus.append((weg, tuple(sorted(
            n for n in dir(k)
            if not n.startswith("_") and callable(getattr(k, n, None))))))
    return tuple(aus)


def funktionsnamen(pg):
    """Wie jedes rufbare Ding **selbst** heisst - `__name__`, `__qualname__`.

    ## Der vierzehnte blinde Fleck: der Name unter dem Namen

    Alle Namensproben hier vergleichen, unter welchem **Attributnamen**
    etwas im Modul steht. Keine fragte, wie das Ding selbst heisst - und
    das sind zwei verschiedene Dinge:

    ```text
    pygame.event.post.__name__       post
    rustgame_py.event.post.__name__  ereignis_post
    ```

    `#[pyfunction]` nimmt den **Rust**-Namen, wenn kein
    `#[pyo3(name = ..)]` dabeisteht, und `als(m, "post", f)` setzt nur das
    Modulfeld. Nachgezaehlt trugen **258 von 390** Funktionen so ihren
    Rust-Namen: `tf_flip`, `ton_get_busy`, `taste_get_mods`, `bild_load`,
    `anzeige_flip`, `maus_get_pos`, `gfx_aacircle`.

    Sichtbar ist das an drei Stellen:

    * `repr(f)` und jeder Rueckverfolgungsauszug nennen den falschen Namen,
    * **jede** Fehlermeldung ueber ein unbekanntes Schluesselwort nennt ihn
      mit - genau der Satz, der als Grenze `unbekanntes Schluesselwort`
      vermerkt war (`tf_scale_by() got an unexpected keyword argument`).
      Dort stand sogar, ein `#[pyo3(name = ..)]` brachte `scale_by()`
      statt `tf_scale_by()` - "naeher, nicht gleich". Das war richtig und
      hat die 258 falschen Namen trotzdem stehen lassen: **naeher ist
      besser, und "nicht gleich" ist kein Grund, es nicht zu tun.**
    * ein Werkzeug, das Aufrufe mitschreibt, ordnet sie dem falschen Namen
      zu - genau daran ist `tools/abdeckung.py` beim ersten Lauf
      gescheitert und hat `event.post` als "nie gerufen" gemeldet, obwohl
      `event_test.py` es siebenundvierzig Mal ruft.

    ## Was verglichen wird

    `__name__` jeder Modulfunktion, jeder Klasse und jeder Methode, dazu
    `__qualname__` der Klassen. Der letzte Punkt hat einen eigenen Fund
    gebracht: `ftfont.Font` entsteht in einer Funktion, und Python schreibt
    das mit - `_font_klasse.<locals>.Font` statt `Font`.
    """
    import importlib

    aus = []
    for n in [""] + _alle_laden(pg):
        voll = pg.__name__ + ("." + n if n else "")
        try:
            m = importlib.import_module(voll)
        except Exception:                            # noqa: BLE001
            continue
        for a in sorted(dir(m)):
            if a.startswith("_"):
                continue
            w = getattr(m, a, None)
            if not callable(w) or isinstance(w, type):
                continue
            aus.append(("%s.%s" % (n or "(haupt)", a),
                        getattr(w, "__name__", None)))
    for weg in __import__("api_coverage").alle_klassen():
        # **`_render.Image` bleibt draussen**, und zwar beim Namen: dort
        # hat pygame eine **leere** Stummelklasse, waehrend die echte
        # `_sdl2.video.Image` heisst - hier ist es je eine Klasse. Das
        # steht als Grenze `_render.Image` und soll diese Probe nicht
        # mitnehmen; jede andere Klasse wird verglichen.
        if weg == "_render.Image":
            continue
        k = _klasse(pg, weg)
        if k is None:
            continue
        aus.append((weg, k.__name__, k.__qualname__))
        for a in sorted(dir(k)):
            if a.startswith("_"):
                continue
            w = getattr(k, a, None)
            if not callable(w):
                continue
            aus.append(("%s.%s" % (weg, a), getattr(w, "__name__", None)))
    return tuple(aus)


def klassenbasen(pg):
    """Die **Basisklassen** jeder erreichbaren Klasse.

    ## Die Frage, die niemand gestellt hat

    `dir()` gegen `dir()` prueft die Namen. Es prueft **nicht**, von was
    eine Klasse erbt - und daran hingen drei Funde:

    * **`key.ScancodeWrapper` ist ein `tuple`.** Hier war es eine
      eigenstaendige Klasse mit `__len__` und `__getitem__`; es fehlten
      `count`, `index` und `isinstance(k, tuple)`.
    * **`Sound` und `PixelArray` hatten eine Ebene zuviel** - eine duenne
      Python-Unterklasse, die nur `__init__` beisteuerte, weil PyO3 keinen
      `tp_init`-Schlitz kennt. Eine PyO3-Klasse ist aber ein **Heap-Typ**,
      und dort setzt `setattr(Klasse, "__init__", f)` den Schlitz mit.
    * **`camera.Camera` und `_camera.Camera` sind zwei Klassen**, nicht
      eine: die erste ist vor `init()` ein Platzhalter, der von
      `AbstractCamera` erbt und bei jeder Frage
      `pygame.camera is not initialized` wirft. Der Platzhalter fehlte
      hier ganz, und damit arbeitete die Kamera vor `init()`.

    Gemeldet werden die Namen der `__mro__` ohne die Klasse selbst - also
    genau die Kette, an der `isinstance` und `issubclass` haengen.

    ## Die Liste ist gelesen

    Woher die Klassen kommen, steht in `api_coverage.alle_klassen()`;
    diese Probe ruft sie auf, damit es **eine** Quelle bleibt. Faellt der
    Import aus, gibt die Probe das als Wert zurueck - eine Probe, die
    stillschweigend nichts prueft, ist schlimmer als keine.
    """
    import importlib
    import os
    import sys

    tools = os.path.join(ROOT, "rustgame", "tools")
    if tools not in sys.path:
        sys.path.insert(0, tools)
    try:
        import api_coverage
    except Exception as e:                           # noqa: BLE001
        return ("api_coverage nicht ladbar", type(e).__name__)

    aus = []
    for weg in api_coverage.alle_klassen():
        teile = weg.split(".")
        # Erst importieren, dann holen - wie `pygame_namen`.
        for k in range(len(teile)):
            try:
                importlib.import_module(
                    "%s.%s" % (pg.__name__, ".".join(teile[: k + 1])))
            except Exception:                        # noqa: BLE001
                break
        obj = pg
        for stueck in teile:
            obj = getattr(obj, stueck, None)
            if obj is None:
                break
        if not isinstance(obj, type):
            aus.append((weg, "kein Typ" if obj is not None else "fehlt"))
            continue
        aus.append((weg, tuple(b.__name__ for b in obj.__mro__[1:])))
    return tuple(aus)


def neuer_puffer(pg):
    """`newbuffer.Py_buffer` - die Huelle um CPythons Struktur.

    ## Die Schnittstelle ist **rohe Adressen**

    Der erste Entwurf dieser Probe fragte nach `view.format = "B"` und
    `view.shape = [4]` - naheliegend und falsch. pygames eigenes
    Testwerkzeug (`tests/test_utils/buftools.py`) schreibt
    `view.format = addressof(self._format)`: alle sechs Zeigerfelder sind
    **Zahlen oder `None`**. Eine Zeichenkette gibt `TypeError`, und bei
    pygame stuerzt das anschliessende `memoryview()` dann ab.

    ## Drei Zustaende

    Geprueft werden alle drei: **leer** (jede Frage ein `AttributeError`),
    **eigen** (nach `get_buffer`, alles schreibgeschuetzt) und - in
    `neuer_ausgeber` - **geliehen**, der einzige, in dem geschrieben werden
    darf.

    ## Und zwei Fehler von pygame

    Der Fehlertext ist `property %400s is undefined for an unallocated
    view`: zwischen `property` und dem Feldnamen stehen 397 Leerzeichen,
    die Meldung ist 446 Zeichen lang. Und **`strides` meldet `shape`** -
    ein uebersehener Kopiervorgang. Beides ist nachgebaut, also wird beides
    verglichen; die Laenge steht in der Probe, damit ein stillschweigend
    anderes Feld auffaellt.
    """
    import importlib
    N = importlib.import_module(pg.__name__ + ".newbuffer")
    FELD = ("buf", "obj", "len", "itemsize", "ndim", "readonly", "format",
            "shape", "strides", "suboffsets", "internal")
    aus = []

    # ----- leer: lesen, schreiben, hergeben -----------------------------
    b = N.Py_buffer()
    aus.append(("bool leer", bool(b)))
    for feld in FELD:
        try:
            getattr(b, feld)
            aus.append((feld, "ok"))
        except Exception as e:                       # noqa: BLE001
            aus.append((feld, type(e).__name__, len(str(e)), str(e).strip()))
    try:
        b.len = 4
        aus.append(("leer setzen", "ok"))
    except Exception as e:                           # noqa: BLE001
        aus.append(("leer setzen", type(e).__name__, len(str(e)),
                    str(e).strip()))
    aus.append(("leer hergeben", b.release_buffer()))

    # ----- eigen: Werte lesen, nichts schreiben -------------------------
    c = N.Py_buffer()
    aus.append(("get ohne flags", c.get_buffer(bytearray(b"abcdef"))))
    aus.append(("bool eigen", bool(c)))
    aus.append(("Zahlen", c.len, c.itemsize, c.ndim, c.readonly,
                type(c.readonly).__name__))
    # Die Zeiger sind Adressen - vergleichbar sind Typ und "nicht null".
    aus.append(("Zeiger sind Zahlen", tuple(
        (f, type(getattr(c, f)).__name__,
         getattr(c, f) is not None and getattr(c, f) > 0)
        for f in ("buf", "format", "shape", "strides"))))
    aus.append(("suboffsets", c.suboffsets, "internal", c.internal))
    aus.append(("obj ist die Quelle", type(c.obj).__name__, bytes(c.obj)))
    for feld in FELD:
        try:
            setattr(c, feld, None)
            aus.append(("eigen setzen " + feld, "ok"))
        except Exception as e:                       # noqa: BLE001
            aus.append(("eigen setzen " + feld, type(e).__name__,
                        len(str(e)), str(e).strip()))
    try:
        c.get_buffer(bytearray(b"xy"))
        aus.append(("zweimal holen", "ok"))
    except Exception as e:                           # noqa: BLE001
        aus.append(("zweimal holen", type(e).__name__, str(e)))
    aus.append(("hergeben", c.release_buffer(), "bool", bool(c)))
    aus.append(("zweimal hergeben", c.release_buffer()))
    try:
        c.len
        aus.append(("nach hergeben lesen", "ok"))
    except Exception as e:                           # noqa: BLE001
        aus.append(("nach hergeben lesen", type(e).__name__))

    # ----- die Flaggen, mit denen geholt wird ---------------------------
    for name in ("PyBUF_SIMPLE", "PyBUF_FULL_RO", "PyBUF_RECORDS_RO",
                 "PyBUF_ND", "PyBUF_STRIDES", "PyBUF_FORMAT"):
        d = N.Py_buffer()
        try:
            d.get_buffer(bytearray(b"abcd"), getattr(N, name))
            aus.append((name, d.len, d.ndim, d.readonly,
                        d.format is None, d.shape is None,
                        d.strides is None))
            d.release_buffer()
        except Exception as e:                       # noqa: BLE001
            aus.append((name, type(e).__name__, str(e)))
    for was, obj, flags in (
        ("WRITABLE auf bytes", b"abcd", N.PyBUF_WRITABLE),
        ("Zahl als Puffer", 7, N.PyBUF_SIMPLE),
    ):
        d = N.Py_buffer()
        try:
            d.get_buffer(obj, flags)
            aus.append((was, "ok"))
            d.release_buffer()
        except Exception as e:                       # noqa: BLE001
            aus.append((was, type(e).__name__, str(e)))
    aus.append(("Zahlen des Moduls", tuple(
        (n, getattr(N, n)) for n in sorted(dir(N)) if n.startswith("PyBUF"))))
    return tuple(aus)


def neuer_ausgeber(pg):
    """`newbuffer.BufferMixin` - eine **eigene** Klasse wird Ausgeber.

    Gespielt wird hier **pygames eigenes Testwerkzeug** nach: ein
    Minimal-`Exporter` wie in `tests/test_utils/buftools.py`, mit
    `ctypes`-Puffern am Objekt und rohen Adressen in der Sicht. Danach die
    Faelle, in denen es scheitern muss - und einer, der ueberrascht:
    **ohne `view.obj` geht es trotzdem**.

    `_release_buffer` sieht die Sicht **noch vollstaendig**; das ist die
    Stelle, an der ein Ausgeber aufraeumen koennte. Die letzte Frage ist
    die nach dem Schmuggel: wer die Huelle aus dem Rueckruf herausholt,
    darf sie danach nicht mehr lesen koennen.
    """
    import ctypes
    import importlib
    N = importlib.import_module(pg.__name__ + ".newbuffer")
    aus = []

    class Geber(N.BufferMixin):
        """Wie `buftools.Exporter`, nur klein."""

        def __init__(self, daten, nur_lesen=0):
            self.d = (ctypes.c_char * len(daten)).from_buffer_copy(daten)
            self.f = ctypes.create_string_buffer(b"B")
            self.sh = (ctypes.c_ssize_t * 1)(len(daten))
            self.st = (ctypes.c_ssize_t * 1)(1)
            self.nur_lesen = nur_lesen
            self.gesehen = []

        def _get_buffer(self, view, flags):
            view.buf = ctypes.addressof(self.d)
            view.readonly = self.nur_lesen
            view.len = len(self.d)
            view.itemsize = 1
            view.ndim = 1
            view.format = ctypes.addressof(self.f)
            view.shape = ctypes.addressof(self.sh)
            view.strides = ctypes.addressof(self.st)
            view.suboffsets = None
            view.internal = None
            view.obj = self
            # Im geliehenen Zustand ist alles zu lesen - und
            # `view.shape is None` fragt `buftools` tatsaechlich ab.
            self.gesehen.append((
                bool(view), view.len, view.ndim, view.readonly,
                view.shape is None, view.suboffsets, view.internal,
                type(view.obj).__name__,
                type(view.buf).__name__, type(view.format).__name__))

        def _release_buffer(self, view):
            self.gesehen.append(("hergeben", type(view.obj).__name__,
                                 view.len, view.ndim))

    g = Geber(b"hallo")
    mv = memoryview(g)
    aus.append(("sicht", mv.nbytes, mv.format, mv.ndim, mv.readonly,
                mv.shape, mv.strides, bytes(mv), mv.c_contiguous))
    mv[0] = ord("H")
    del mv
    aus.append(("geschrieben", bytes(memoryview(g))))
    aus.append(("im Rueckruf gesehen", tuple(g.gesehen)))

    r = Geber(b"abc", nur_lesen=1)
    mr = memoryview(r)
    aus.append(("nur lesen", mr.readonly, bytes(mr)))
    try:
        mr[0] = 65
        aus.append(("schreiben verboten", "ok"))
    except Exception as e:                           # noqa: BLE001
        aus.append(("schreiben verboten", type(e).__name__, str(e)))
    del mr

    class Flach(N.BufferMixin):
        """Eine Sicht mit `ndim` 0 - die `PyBUF_SIMPLE`-Form."""

        def __init__(self):
            self.d = (ctypes.c_char * 4).from_buffer_copy(b"wxyz")

        def _get_buffer(self, view, flags):
            view.buf = ctypes.addressof(self.d)
            view.len = 4
            view.itemsize = 1
            view.ndim = 0
            view.readonly = 1
            view.format = None
            view.shape = None
            view.strides = None
            view.suboffsets = None
            view.internal = None
            view.obj = self

    f = memoryview(Flach())
    aus.append(("ndim 0", f.nbytes, f.format, f.ndim, f.shape, f.strides,
                bytes(f)))
    del f

    # ----- die Faelle, die scheitern muessen ---------------------------
    #
    # **Zwei Faelle stehen hier absichtlich nicht**: ein `_get_buffer`, das
    # gar nichts setzt, und eines, das alles ausser `view.obj` setzt.
    # pygame nullt CPythons Struktur nicht, bevor es den Rueckruf ruft -
    # beide Faelle lesen dort also Stapelinhalt. Dreimal nacheinander
    # gemessen gab derselbe Fall `MemoryError`, dann `(0, b"")`, dann
    # `ValueError: number of dimensions must not exceed 64`; der zweite
    # Fall **stuerzt ab**. Eine Probe, deren Referenzwert vom Stapel
    # abhaengt, misst nichts, und eine, die den Vergleichslauf abschiesst,
    # misst weniger als nichts. rustgame nullt die Struktur; das steht als
    # Verbesserung im Modulkopf von `neuerpuffer.rs`.
    class OhneGeber(N.BufferMixin):
        """Hat gar kein `_get_buffer`."""

    class Wirft(N.BufferMixin):
        """Wirft - und der Grund muss durchkommen."""

        def _get_buffer(self, view, flags):
            raise BufferError("mein Grund")

    for name, K in (("ohne _get_buffer", OhneGeber), ("wirft", Wirft)):
        try:
            v = memoryview(K())
            aus.append((name, v.nbytes, bytes(v)))
            del v
        except Exception as e:                       # noqa: BLE001
            aus.append((name, type(e).__name__, str(e)))

    roh = N.BufferMixin()
    for was, ruf in (
        ("_get_buffer von Hand", lambda: roh._get_buffer(N.Py_buffer(), 0)),
        ("_release_buffer von Hand",
         lambda: roh._release_buffer(N.Py_buffer())),
        ("BufferMixin mit Argument",
         lambda: type(N.BufferMixin(1, x=2)).__name__),
        # Der Modulname steckt in der Meldung - `pygame.newbuffer` gegen
        # `rustgame_py.newbuffer`. Das ist die Grenze
        # `Namen der Typen und Module`, und sie soll hier nicht ein
        # zweites Mal auffallen; verglichen wird der Rest des Satzes.
        ("len() Satz", lambda: _ohne_modul(roh)),
        # `__module__` steht bei PyO3 im Klassenverzeichnis, bei pygames
        # statischem Typ nicht - dieselbe Grenze, derselbe Grund.
        ("Namen der Klasse", lambda: tuple(
            n for n in dir(N.BufferMixin)
            if n not in dir(object) and n != "__module__")),
    ):
        try:
            aus.append((was, ruf()))
        except Exception as e:                       # noqa: BLE001
            aus.append((was, type(e).__name__, str(e)))

    class Schmuggler(Geber):
        """Holt die Huelle aus dem Rueckruf heraus."""

        def _get_buffer(self, view, flags):
            Geber._get_buffer(self, view, flags)
            self.behalten = view

    sc = Schmuggler(b"xy")
    msc = memoryview(sc)
    del msc
    try:
        sc.behalten.len
        aus.append(("geschmuggelte Huelle", "liest noch"))
    except Exception as e:                           # noqa: BLE001
        aus.append(("geschmuggelte Huelle", type(e).__name__))
    return tuple(aus)


def pypm_auskunft(pg):
    """`pypm` - die Schicht unter `pygame.midi`.

    Verglichen wird, was **nicht** am Geraetebestand haengt: die
    Filterbits, die elf Fehlertexte und `Channel`. Die Geraeteanzahl ist
    eine benannte Grenze (`midi.Geraeteliste`), darum steht sie hier nicht.

    **`Channel(0)` ist `-2147483648`**, und das ist die Zahl, auf die es
    ankommt: PortMidis `1 << (channel - 1)` schiebt dort um -1, der
    Prozessor maskiert die Weite auf fuenf Bit und schiebt um 31.
    """
    import importlib
    pm = importlib.import_module(pg.__name__ + ".pypm")
    filter_ = tuple((n, getattr(pm, n)) for n in sorted(
        x for x in dir(pm) if x.startswith("FILT_")))
    fehler = tuple((n, pm.GetErrorText(n))
                   for n in list(range(-10001, -9990)) + [-1, 0, 1, 2])
    kanal = tuple(pm.Channel(n) for n in range(0, 18))
    return (filter_, fehler, kanal, pm.FALSE, pm.TRUE,
            pm.Initialize(), pm.Terminate(), pm.GetDeviceInfo(-1),
            pm.GetDeviceInfo(9999))


def paketdaten(pg):
    """`pkgdata` - vier Namen, nicht einer.

    `getResource` stand hier allein; `dir(pygame.pkgdata)` zeigt vier.
    Verglichen wird, was beide Engines ueber ihr **eigenes** Paket sagen -
    die Pfade unterscheiden sich natuerlich, die Antworten nicht.
    """
    import importlib
    pk = importlib.import_module(pg.__name__ + ".pkgdata")
    paket = pg.__name__
    aus = []
    for was, ruf in (
        ("exists Sinnbild", lambda: pk.resource_exists(paket, "pygame_icon.bmp")),
        ("exists nix", lambda: pk.resource_exists(paket, "gibtesnicht.xyz")),
        ("stream Sinnbild", lambda: type(
            pk.resource_stream(paket, "pygame_icon.bmp")).__name__),
        ("getResource", lambda: type(pk.getResource("pygame_icon.bmp")).__name__),
        ("files ist Pfad", lambda: hasattr(pk.files(paket), "joinpath")),
    ):
        try:
            aus.append((was, ruf()))
        except Exception as e:                       # noqa: BLE001
            aus.append((was, type(e).__name__))
    return tuple(aus)


def midi_uhr(pg):
    """`midi.time()` - eine Zahl in Millisekunden, direkt nach `init` klein.

    **Erst importieren.** `pygame.midi` laedt nicht mit `import pygame`
    mit; ohne die Zeile gab die Probe fuer pygame
    `AttributeError: module 'pygame' has no attribute 'midi'` und fuer
    rustgame eine Zahl - eine Abweichung, die nur die Probe selbst
    verschuldet hatte. Dieselbe Falle wie bei `gfxdraw`, `freetype` und
    `camera`, und sie ist im Messgeraet schon einmal aufgetreten.
    """
    import importlib
    importlib.import_module(pg.__name__ + ".midi")
    pg.midi.init()
    t = pg.midi.time()
    return (type(t).__name__, 0 <= t < 5000)



# ---------------------------------------------------------------------------
#  Hilfsstuecke fuer die Proben
# ---------------------------------------------------------------------------

def lokale(pg):
    """`pygame.locals` gibt es nur als eigenstaendiges Untermodul."""
    import importlib
    m = importlib.import_module(pg.__name__ + ".locals")
    return (m.QUIT, m.K_SPACE, m.SRCALPHA, m.KMOD_CTRL, m.BLEND_RGBA_MAX)


def aus_polar(pg, polar):
    v = pg.math.Vector2()
    rueck = v.from_polar(polar)
    return (rueck, tuple(v))


def aktualisiere(pg):
    v = pg.math.Vector2(1, 2)
    rueck = v.update(7, 8)
    a = (rueck, tuple(v))
    v.x = 9
    v[1] = 10
    return (a, tuple(v), tuple(v.copy()))


def maske_grund(pg, fill):
    m = pg.mask.Mask((6, 4), fill=fill)
    return (m.get_size(), m.count(), tuple(m.get_rect()), repr(m),
            m.get_at((0, 0)), type(m.get_at((0, 0))).__name__)


def maske_setzen(pg):
    m = pg.mask.Mask((6, 4))
    m.set_at((1, 1), 1)
    m.set_at((2, 2), 1)
    m.set_at((4, 0), 1)
    a = (m.count(), m.get_at((1, 1)), m.get_at((0, 0)))
    m.set_at((1, 1), 0)
    b = m.count()
    m.invert()
    c = m.count()
    m.fill()
    d = m.count()
    m.clear()
    return (a, b, c, d, m.count())


def maske_ueberdeckung(pg):
    a = pg.mask.Mask((6, 4))
    a.set_at((1, 1), 1)
    a.set_at((2, 2), 1)
    b = pg.mask.Mask((6, 4), fill=True)
    leer = pg.mask.Mask((6, 4))
    return (a.overlap(b, (0, 0)), a.overlap(leer, (0, 0)),
            a.overlap_area(b, (0, 0)), a.overlap_area(b, (1, 1)),
            a.overlap_mask(b, (0, 0)).count())


def maske_zeichnen(pg):
    a = pg.mask.Mask((6, 4))
    a.set_at((1, 1), 1)
    b = pg.mask.Mask((6, 4))
    b.set_at((3, 2), 1)
    a.draw(b, (0, 0))
    gezeichnet = (a.count(), a.get_at((3, 2)))
    a.erase(b, (0, 0))
    return (gezeichnet, a.count(), a.get_at((3, 2)))


def maske_auswertung(pg):
    m = pg.mask.Mask((8, 6))
    for p in ((1, 1), (2, 2), (2, 1), (6, 5)):
        m.set_at(p, 1)
    return (m.centroid(), round(m.angle(), 9),
            [tuple(r) for r in m.get_bounding_rects()],
            len(m.connected_components()),
            [x.count() for x in m.connected_components()],
            m.connected_component().count(),
            len(m.connected_components(2)))


def maske_flaeche(pg):
    m = pg.mask.Mask((4, 3))
    m.set_at((1, 1), 1)
    s = m.to_surface()
    a = (s.get_size(), s.get_flags(), tuple(s.get_at((1, 1))), tuple(s.get_at((0, 0))))
    t = m.to_surface(setcolor=(1, 2, 3, 4), unsetcolor=None)
    return (a, tuple(t.get_at((1, 1))), tuple(t.get_at((0, 0))))


def maske_aus_flaeche(pg):
    s = pg.Surface((4, 3), pg.SRCALPHA)
    s.fill((10, 20, 30, 200))
    s.set_at((0, 0), (10, 20, 30, 100))
    s.set_at((1, 0), (10, 20, 30, 127))
    return (pg.mask.from_surface(s).count(),
            pg.mask.from_surface(s, 99).count(),
            pg.mask.from_surface(s, 255).count())


def maske_schwelle(pg):
    s = pg.Surface((4, 3))
    s.fill((10, 20, 30))
    s.set_at((0, 0), (200, 20, 30))
    return (pg.mask.from_threshold(s, (10, 20, 30), (2, 2, 2)).count(),
            pg.mask.from_threshold(s, (200, 20, 30), (2, 2, 2)).count())


def maske_skalieren(pg):
    m = pg.mask.Mask((4, 4))
    m.set_at((0, 0), 1)
    m.set_at((3, 3), 1)
    k = m.scale((8, 8))
    return (k.get_size(), k.count(), k.get_at((0, 0)), k.get_at((7, 7)))

def setz(pg, werte, feld):
    c = pg.Color(9, 9, 9, 9)
    setattr(c, feld, werte)
    return tuple(c)


def upd(pg):
    c = pg.Color(10, 20, 30, 40)
    c.update(1, 2, 3)
    a = tuple(c)
    c.update(9, 8, 7, 6)
    return (a, tuple(c))


def ip(r, name, *args):
    k = r.copy()
    rueck = getattr(k, name)(*args)
    return (rueck, tuple(k))


def bounding(pg):
    s = pg.Surface((6, 6), pg.SRCALPHA)
    s.set_at((2, 3), (255, 0, 0, 200))
    s.set_at((4, 1), (0, 255, 0, 5))
    return (tuple(s.get_bounding_rect()), tuple(s.get_bounding_rect(10)),
            tuple(s.get_bounding_rect(255)))


def premul(pg):
    s = pg.Surface((3, 2), pg.SRCALPHA)
    s.fill((200, 100, 50, 128))
    s.set_at((0, 0), (255, 255, 255, 1))
    s.set_at((2, 1), (10, 20, 30, 255))
    p = s.premul_alpha()
    return [tuple(p.get_at((x, y))) for y in range(2) for x in range(3)]


def subs(pg):
    s = pg.Surface((10, 10))
    t = s.subsurface(pg.Rect(2, 3, 4, 5))
    t.fill((1, 2, 3))
    return (t.get_size(), t.get_offset(), t.get_abs_offset(),
            t.get_parent() is s, tuple(s.get_at((2, 3))), tuple(s.get_at((1, 3))))


def blits(pg):
    z = pg.Surface((8, 8))
    a = pg.Surface((2, 2))
    a.fill((10, 20, 30))
    b = pg.Surface((2, 2))
    b.fill((40, 50, 60))
    r = z.blits([(a, (0, 0)), (b, (3, 3))])
    return ([tuple(x) for x in r], tuple(z.get_at((0, 0))), tuple(z.get_at((3, 3))))


def scroll(pg):
    s = pg.Surface((8, 4))
    for x in range(8):
        for y in range(4):
            s.set_at((x, y), (x * 30, y * 60, 0))
    s.set_clip(pg.Rect(2, 1, 4, 2))
    s.scroll(2, 0)
    aus = [tuple(s.get_at((x, y)))[:3] for y in range(4) for x in range(8)]
    s.set_clip(None)
    return aus


def fill_blend(pg):
    aus = []
    for flag in (0, pg.BLEND_ADD, pg.BLEND_SUB, pg.BLEND_MULT, pg.BLEND_MIN,
                 pg.BLEND_MAX, pg.BLEND_RGBA_ADD, pg.BLEND_RGBA_SUB,
                 pg.BLEND_RGBA_MULT, pg.BLEND_RGBA_MIN, pg.BLEND_RGBA_MAX,
                 pg.BLEND_PREMULTIPLIED):
        s = pg.Surface((2, 2), pg.SRCALPHA)
        s.fill((100, 150, 200, 120))
        s.fill((50, 60, 70, 80), None, flag)
        aus.append((flag, tuple(s.get_at((0, 0)))))
    return aus


def blit_blend(pg):
    aus = []
    for flag in (0, pg.BLEND_ADD, pg.BLEND_RGBA_ADD, pg.BLEND_RGBA_MULT,
                 pg.BLEND_RGBA_MIN, pg.BLEND_RGBA_MAX, pg.BLEND_PREMULTIPLIED):
        z = pg.Surface((2, 2), pg.SRCALPHA)
        z.fill((100, 150, 200, 120))
        q = pg.Surface((2, 2), pg.SRCALPHA)
        q.fill((50, 60, 70, 80))
        z.blit(q, (0, 0), None, flag)
        aus.append((flag, tuple(z.get_at((0, 0)))))
    return aus


def rundlauf(pg, quelle, f):
    b = pg.image.tobytes(quelle, f)
    zurueck = pg.image.frombytes(b, (5, 3), f)
    return (zurueck.get_flags(), zurueck.get_size(),
            pg.image.tobytes(zurueck, "RGBA").hex())


def png_rundlauf(pg, quelle):
    # Im Temp-Ordner wie `rustgame_probe_ie.png` oben: hier stand `HIER`,
    # und nach jedem Lauf lag eine `rundlauf.png` im Quellbaum, die ein
    # Commit mitgenommen haette (Runde 21).
    import tempfile

    pfad = os.path.join(tempfile.gettempdir(), "rustgame_rundlauf.png")
    pg.image.save(quelle, pfad)
    zurueck = pg.image.load(pfad)
    return (zurueck.get_size(), zurueck.get_flags(),
            pg.image.tobytes(zurueck, "RGBA").hex())


def taktung(pg, rate):
    """Die **kuerzeste** Taktlaenge, die bei dieser Bildrate vorkommt.

    Geprueft wird damit die abgeschnittene Rechnung
    `(int)((1.0f/rate)*1000)`: bei 60 sind das 16 Millisekunden, wer
    aufrundet, kommt nie unter 17.

    Hier stand einmal `sorted(set(werte))` mit der Begruendung, die
    einzelnen Werte schwankten zwar, der **Satz** der vorkommenden aber
    nicht. Das war falsch und hat als einzige Probe dieser Datei je nach
    Maschinenlast gemeldet: pygame lieferte einmal `[16, 17, 18]`, rustgame
    `[16, 17]`, und beide Engines waren im Recht. Gewartet wird mit
    `SDL_Delay` bzw. dem Rust-Gegenstueck, und was der Planer des
    Betriebssystems obendrauf legt, gehoert keiner der beiden.

    Das Kleinste ist die Groesse, die nur an der Rechnung haengt: laenger
    kann ein Takt durch Last immer werden, kuerzer nie. Zwoelf Proben
    statt acht, damit auch unter Last eine davon ungestoert durchkommt.
    """
    u = pg.time.Clock()
    u.tick(rate)                       # der erste Takt legt nur an
    return min(u.tick(rate) for _ in range(12))


def ew_rechnen(pg):
    """Alle zwoelf Rechenzeichen des Proxys auf einmal."""
    V = pg.math.Vector2
    v, w = V(3, 4), V(2, 8)
    e = v.elementwise()
    return [tuple(x) for x in
            (e * w, e / w, e + w, e - w, e * 2, e ** 2, e % 3, e // 2,
             10 - e, 2 / e, abs(e), -e)]


def ew_vorzeichen(pg):
    """`%` und `//` folgen **Python**, nicht Cs `fmod`.

    Der Unterschied ist das Vorzeichen: `fmod(-7, 3)` ist `-1`, Python
    sagt `2`. Ohne negative Zahlen faellt das nie auf, deshalb stehen hier
    nur negative Faelle.
    """
    V = pg.math.Vector2
    aus = []
    for a, b in [(-7.0, 3.0), (7.0, -3.0), (-7.0, -3.0), (7.5, 2.0), (-7.5, 2.0)]:
        e = V(a, a).elementwise()
        aus.append(((e % b)[0], (e // b)[0]))
    return aus


def ew_vergleiche(pg):
    """Ein Vergleich gibt `bool`, keinen Vektor - und fragt **alle** Achsen."""
    V = pg.math.Vector2
    a, b, c = V(1, 2), V(3, 4), V(1, 4)
    return (a.elementwise() < b, a.elementwise() <= b,
            a.elementwise() > b, a.elementwise() >= b,
            a.elementwise() == b, a.elementwise() != b,
            a.elementwise() == c, a.elementwise() <= c,
            V(2, 2).elementwise() == 2, V(2, 3).elementwise() == 2)


def ew_fehler(pg):
    """Die vier Fehlerfaelle - samt Wortlaut."""
    V = pg.math.Vector2
    aus = []
    for f in (lambda: V(1, 2).elementwise() / V(0, 1),
              lambda: V(1, 2).elementwise() % V(0, 1),
              lambda: V(-2, 4).elementwise() ** 0.5,
              lambda: V(0, 4).elementwise() ** -1):
        try:
            f()
            aus.append("kein Fehler")
        except Exception as e:                      # noqa: BLE001
            aus.append("%s: %s" % (type(e).__name__, e))
    return aus


def tf_quelle(pg):
    """Eine kleine, bunte Flaeche mit Alphaverlauf."""
    s = pg.Surface((6, 5), pg.SRCALPHA)
    for y in range(5):
        for x in range(6):
            s.set_at((x, y), (x * 40 % 256, y * 50 % 256, (x * y * 17) % 256, 255 - x * 10))
    return s


def tf_neue(pg):
    """Jede der spaet durchgereichten Funktionen, als Pixelbytes."""
    s = tf_quelle(pg)
    aus = []
    for name, f in (
        ("invert", lambda: pg.transform.invert(s)),
        ("laplacian", lambda: pg.transform.laplacian(s)),
        ("chop", lambda: pg.transform.chop(s, pg.Rect(1, 1, 2, 2))),
        ("box_blur", lambda: pg.transform.box_blur(s, 1)),
        ("box_blur hart", lambda: pg.transform.box_blur(s, 2, False)),
        ("gaussian_blur", lambda: pg.transform.gaussian_blur(s, 1)),
        # `radius = 0` ergibt in pygame eine **schwarze, durchsichtige**
        # Flaeche - `expf(-0.0/0.0)` ist NaN, und `(Uint8)NaN` ist 0.
        ("gaussian_blur 0", lambda: pg.transform.gaussian_blur(s, 0)),
        ("pixelate", lambda: pg.transform.pixelate(s, 2)),
        ("solid_overlay", lambda: pg.transform.solid_overlay(s, (255, 0, 0))),
        ("hsl", lambda: pg.transform.hsl(s, 30, 0.2, -0.1)),
        ("rotozoom", lambda: pg.transform.rotozoom(s, 30, 1.5)),
        ("average_surfaces",
         lambda: pg.transform.average_surfaces([s, pg.transform.invert(s)])),
    ):
        r = f()
        aus.append((name, r.get_size(), pg.image.tobytes(r, "RGBA").hex()))
    # `threshold` gibt eine Zahl zurueck, keine Flaeche.
    d = pg.Surface((6, 5), pg.SRCALPHA)
    n = pg.transform.threshold(d, s, (40, 50, 0, 255), (60, 60, 60, 255),
                               (0, 255, 0, 255), 1)
    aus.append(("threshold", n, pg.image.tobytes(d, "RGBA").hex()))
    return aus


def tf_ziel(pg):
    """`dest_surface` wird **beschrieben** und **selbst** zurueckgegeben.

    Das war lange nicht so: der Wert stand als `let _ = dest_surface;` in
    der Huelle, damit der Aufruf nicht scheitert. Ein Programm bekam dann
    eine unveraenderte Flaeche und keinerlei Hinweis - schlimmer als ein
    Fehler, weil es lautlos ist.
    """
    s = tf_quelle(pg)
    d = pg.Surface((6, 5), pg.SRCALPHA)
    r = pg.transform.invert(s, d)
    return (r is d, pg.image.tobytes(d, "RGBA").hex())


def tf_meldungen(pg):
    """Vier verschiedene Wortlaute, je nach Funktion - alle gemessen.

    Einen davon fuer alle zu nehmen waere die naheliegende Vereinfachung
    und in drei Vierteln der Faelle die falsche Meldung.
    """
    s = tf_quelle(pg)
    d = pg.Surface((3, 3), pg.SRCALPHA)
    aus = []
    for name, f in (
        ("scale", lambda: pg.transform.scale(s, (6, 5), d)),
        ("smoothscale", lambda: pg.transform.smoothscale(s, (6, 5), d)),
        ("scale2x", lambda: pg.transform.scale2x(s, d)),
        ("grayscale", lambda: pg.transform.grayscale(s, d)),
        ("invert", lambda: pg.transform.invert(s, d)),
        ("laplacian", lambda: pg.transform.laplacian(s, d)),
        ("box_blur", lambda: pg.transform.box_blur(s, 1, True, d)),
        ("gaussian_blur", lambda: pg.transform.gaussian_blur(s, 1, True, d)),
        ("pixelate", lambda: pg.transform.pixelate(s, 2, d)),
        ("solid_overlay", lambda: pg.transform.solid_overlay(s, (1, 2, 3), d)),
        ("hsl", lambda: pg.transform.hsl(s, 1, 0, 0, d)),
        ("average_surfaces", lambda: pg.transform.average_surfaces([s], d)),
    ):
        try:
            f()
            aus.append((name, "kein Fehler"))
        except Exception as e:                      # noqa: BLE001
            aus.append((name, "%s: %s" % (type(e).__name__, e)))
    return aus


def tf_backend(pg):
    """Was `set_smoothscale_backend` annimmt und was es ablehnt.

    Hier standen nur `XYZ`, `NEON` und `GENERIC` - und rustgame sagte zu
    `MMX` und `SSE` "not supported", wo pygame "Unknown backend type"
    sagt (ohne MMX-Uebersetzung vergleicht pygames Ersatzzweig den
    **bisherigen** Weg). Runde 18 hat die Liste um alles erweitert, was
    pygame kennt, und den Weg nach jedem Schritt zurueckgelesen.
    """
    aus = []
    for wert in ("XYZ", "NEON", "MMX", "SSE", "sse2", "", "GENERIC", "SSE2"):
        try:
            pg.transform.set_smoothscale_backend(wert)
            aus.append((wert, "angenommen", pg.transform.get_smoothscale_backend()))
        except Exception as e:                      # noqa: BLE001
            aus.append((wert, "%s: %s" % (type(e).__name__, e)))
    return aus


def tf_smoothscale_wege(pg):
    """`smoothscale` auf **beiden** Rechenwegen, Byte fuer Byte.

    Bis Runde 18 rechnete rustgame nur `GENERIC` und meldete das auch -
    pygame rechnet aber `SSE2`, und jedes Programm, das nichts umstellt,
    bekam bis zu zwei Stufen andere Pixel. PyGameZ baut jeden Hintergrund
    aus `smoothscale`. Gemessen werden sieben Quell- mal sieben
    Zielgroessen, mit und ohne Alphakanal, je Weg - 196 Bilder.
    """
    import hashlib
    import random
    aus = []
    groessen = ((96, 96), (80, 60), (7, 5), (3, 1), (1, 3), (33, 17), (128, 128))
    ziele = ((640, 480), (192, 192), (5, 9), (200, 1), (1, 1), (61, 43), (96, 96))
    for weg in ("SSE2", "GENERIC"):
        pg.transform.set_smoothscale_backend(weg)
        nr = 0
        for (sw, sh) in groessen:
            for (tw, th) in ziele:
                for alpha in (False, True):
                    r = random.Random(nr)
                    s = pg.Surface((sw, sh), pg.SRCALPHA if alpha else 0)
                    for y in range(sh):
                        for x in range(sw):
                            s.set_at((x, y), (r.randrange(256), r.randrange(256),
                                              r.randrange(256), r.randrange(256)))
                    d = pg.transform.smoothscale(s, (tw, th))
                    aus.append((weg, nr, hashlib.sha256(
                        pg.image.tobytes(d, "RGBA")).hexdigest()[:16]))
                    nr += 1
    pg.transform.set_smoothscale_backend("SSE2")
    if len(aus) < 190:
        raise RuntimeError("smoothscale-Wege: nur %d Eintraege" % len(aus))
    return aus


def text_zeichenbilder(pg):
    """`font.render` Zeichen fuer Zeichen in den Schriften von PyGameZ.

    Bis Runde 18 war **keines** von 11.984 Zeichenbildern gleich: rustgame
    rasterte mit fontdue und ungehintet, pygame mit FreeTypes `ftgrays`
    nach dem TrueType-Interpreter (v40). Die Groesse stimmte fast immer,
    die Pixel nie. Gemessen werden hier die vier Schriften, die PyGameZ
    benutzt (`ui.FONT_UI`, `ui.FONT_MONO`, die Schachfiguren), in sechs
    Groessen, normal und fett, jedes druckbare ASCII-Zeichen und einige
    deutsche - als Groesse und SHA-256 der Alphawerte.
    """
    import hashlib
    aus = []
    zeichen = [chr(c) for c in range(33, 127)] + list("äöüÄÖÜß€")
    for schrift in ("bahnschrift", "segoeui", "consolas", "segoeuisymbol"):
        for g in (11, 14, 18, 22, 28, 48):
            for fett in (False, True):
                f = pg.font.SysFont(schrift, g, bold=fett)
                for z in zeichen:
                    s = f.render(z, True, (255, 255, 255))
                    alpha = pg.image.tobytes(s, "RGBA")[3::4]
                    aus.append((schrift, g, fett, z, s.get_size(),
                                hashlib.sha256(alpha).hexdigest()[:12]))
    if len(aus) < 4000:
        raise RuntimeError("Zeichenbilder: nur %d Eintraege" % len(aus))
    return aus


def text_einfarbig(pg):
    """`font.render(text, False, ..)` - FreeTypes Schwarzweiss-Rasterer.

    **Bis Runde 19 war jede einfarbige Flaeche leer.** `acht_bit` stellte
    die Flaeche mit `set_format` auf acht Bit (Pixel gegen die alte, leere
    Tafel umgerechnet) und setzte die Zweifarbentafel danach ohne
    Umfaerben; keine Probe rief `render` mit `antialias=False` und sah die
    Pixel an. Dahinter lag der zweite Fehler: das Bild kam aus fontdue mit
    einer Schwelle bei halber Deckung, waehrend SDL_ttf `FT_RENDER_MODE_MONO`
    rastert (`ftraster.c`, mit Dropout-Steuerung aus `prep`). Gemessen wird
    hier quer durch Schriften, Groessen und die zwei kuenstlichen Stile -
    als Groesse, Bittiefe, Schluesselfarbe und SHA-256 der Pixel.
    """
    import hashlib
    aus = []
    zeichen = [chr(c) for c in range(33, 127)] + list("äöüß€") + [
        "Score: 1234", "The quick brown fox"]
    for schrift in (None, "bahnschrift", "segoeui", "consolas", "arial", "timesnewroman"):
        for g in (9, 12, 17, 24, 36):
            for stil in ("normal", "fett", "kursiv"):
                if schrift is None:
                    f = pg.font.Font(None, g)
                else:
                    f = pg.font.SysFont(schrift, g)
                f.set_bold(stil == "fett")
                f.set_italic(stil == "kursiv")
                for z in zeichen:
                    s = f.render(z, False, (255, 255, 255))
                    aus.append((schrift, g, stil, z, s.get_size(), s.get_bitsize(),
                                s.get_colorkey(), hashlib.sha256(
                                    pg.image.tobytes(s, "RGB")).hexdigest()[:12]))
    if len(aus) < 8000:
        raise RuntimeError("einfarbig: nur %d Eintraege" % len(aus))
    return aus


def text_kontur(pg):
    """`font.outline = n` - `FT_Glyph_Stroke` und die Masse dazu.

    Zwei Fehler, beide bis Runde 19: das **Bild** war das fertige Zeichen,
    um `n` Pixel geweitet (0 von 240 gleich - SDL_ttf zieht mit
    `FT_Stroker` eine neue Kontur mit Kreisboegen um den gehinteten
    Umriss), und die **Masse** lagen auf der Schrift statt auf der
    Vermessung: `get_height()` wuchs um `2n`, wo pygame gleich bleibt;
    `get_ascent()` wuchs um `n`, wo pygame `2n` meldet (`TTF_FontAscent`),
    und `metrics()` legte die Kontur auf beide Seiten statt nur auf
    `maxx`/`maxy`. Nur `size()` stimmte - und genau das war gemessen.
    """
    import hashlib
    aus = []
    zeichen = [chr(c) for c in range(33, 127, 3)] + ["Hallo Welt", "i ", " "]
    for schrift in (None, "bahnschrift", "arial"):
        for g in (12, 20, 32):
            f = pg.font.Font(None, g) if schrift is None else pg.font.SysFont(schrift, g)
            for n in (0, 1, 2, 3):
                f.outline = n
                aus.append((schrift, g, n, "masse", f.get_height(), f.get_ascent(),
                            f.get_descent(), f.get_linesize(), f.size("Hallo"),
                            f.metrics("Hj")))
                if n == 0:
                    continue
                for aa in (True, False):
                    for z in zeichen:
                        s = f.render(z, aa, (255, 255, 255))
                        aus.append((schrift, g, n, aa, z, s.get_size(), hashlib.sha256(
                            pg.image.tobytes(s, "RGBA")).hexdigest()[:12]))
    if len(aus) < 1500:
        raise RuntimeError("Kontur: nur %d Eintraege" % len(aus))
    return aus


def schluessel_form(pg):
    """`get_colorkey()` ist ein **Tupel**, und eine Zahl in `set_colorkey`
    ein Pixelwert.

    Hier kam bis Runde 19 eine `Color` zurueck - und `Color == Tupel` ist
    wahr, darum hatten die Proben, die den Schluessel vorher in ein Tupel
    wandelten, nichts gesehen. Dazu `set_colorkey(5)` auf einer
    Acht-Bit-Flaeche: bei pygame der **Tafeleintrag 5**
    (`pg_MappedColorFromObj`), hier war es Schwarz.
    """
    aus = []
    for tiefe, flags in ((8, 0), (16, 0), (24, 0), (32, 0), (32, pg.SRCALPHA)):
        s = pg.Surface((3, 3), flags, tiefe)
        for k in ((1, 2, 3), (200, 100, 50, 7), 5, 0x123456, "red", None):
            try:
                s.set_colorkey(k)
                r = s.get_colorkey()
                aus.append((tiefe, flags, repr(k), type(r).__name__, r))
            except Exception as e:  # noqa: BLE001
                aus.append((tiefe, flags, repr(k), type(e).__name__, str(e)))
    return aus


def klang_aiff_voc(pg):
    """AIFF, AIFF-C, 8SVX und Creative Voice - auf beiden Wegen.

    **Bis Runde 19 las rustgame keines davon.** `Sound("x.aiff")` und
    `Sound("x.voc")` gaben `Unrecognized audio format`, wo pygame spielt:
    `Mix_LoadWAV_RW` prueft nach `RIFF`/`WAVE` auf `FORM` (`load_aiff.c`)
    und `Crea` (`load_voc.c`), und `mixer.music` liest `FORM` mit einem
    **zweiten**, groesseren Leser (`music_wav.c`: AIFF-C mit `sowt`,
    `raw `, `ulaw`, `alaw`, `fl32`, `fl64`, 24 und 32 Bit, Etiketten aus
    `NAME`/`AUTH`/`(c) `). Die Dateien werden hier gebaut - pygames
    Beispiele enthalten keine.
    """
    import hashlib
    import io
    import math
    import struct

    def sane(r):
        if r == 0:
            return b"\0" * 10
        e = int(math.floor(math.log2(r)))
        return struct.pack(">HQ", 16383 + e, int(r) << (63 - e))

    def block(name, inhalt):
        return name + struct.pack(">I", len(inhalt)) + inhalt + (b"\0" if len(inhalt) % 2 else b"")

    def aiff(bits, spuren, rate, daten, art=b"AIFF", komp=None, extra=b"", ohne=(), rahmen=None):
        breite = max(1, bits // 8)
        n = rahmen if rahmen is not None else len(daten) // spuren // breite
        comm = struct.pack(">hIh", spuren, n, bits) + sane(rate)
        if komp:
            comm += komp + b"\x04none\0"
        koerper = art
        if art == b"AIFC" and "FVER" not in ohne:
            koerper += block(b"FVER", struct.pack(">I", 0xA2805140))
        koerper += extra
        if "COMM" not in ohne:
            koerper += block(b"COMM", comm)
        if "SSND" not in ohne:
            koerper += block(b"SSND", b"\0" * 8 + daten)
        return b"FORM" + struct.pack(">I", len(koerper)) + koerper

    def svx(rate, daten):
        vhdr = struct.pack(">IIIHBBI", len(daten), 0, 0, rate, 1, 0, 0x10000)
        koerper = b"8SVX" + block(b"VHDR", vhdr) + block(b"BODY", daten)
        return b"FORM" + struct.pack(">I", len(koerper)) + koerper

    def vblk(art, inhalt):
        return bytes([art]) + struct.pack("<I", len(inhalt))[:3] + inhalt

    kopf = b"Creative Voice File\x1a" + struct.pack("<HHH", 26, 0x010A, 0x1129)
    w8 = bytes((int(100 * math.sin(i / 7)) & 0xFF) for i in range(3000))
    u8 = bytes((128 + int(100 * math.sin(i / 5))) & 0xFF for i in range(2000))
    w16 = b"".join(struct.pack(">h", int(20000 * math.sin(i / 9))) for i in range(3000))
    w16le = b"".join(struct.pack("<h", int(20000 * math.sin(i / 9))) for i in range(3000))
    w32 = b"".join(struct.pack(">i", int(2e9 * math.sin(i / 9))) for i in range(3000))
    w24 = b"".join(struct.pack(">i", int(8e6 * math.sin(i / 9)))[1:] for i in range(3000))
    wf = b"".join(struct.pack(">f", 0.8 * math.sin(i / 9)) for i in range(3000))
    wd = b"".join(struct.pack(">d", 0.8 * math.sin(i / 9)) for i in range(3000))
    faelle = []
    for rate in (8000, 11025, 22050, 44100, 48000):
        for spuren in (1, 2):
            faelle.append(("s16 %d %d" % (rate, spuren), aiff(16, spuren, rate, w16)))
            faelle.append(("s8 %d %d" % (rate, spuren), aiff(8, spuren, rate, w8)))
    faelle += [
        ("8svx", svx(8000, w8)),
        ("aifc NONE", aiff(16, 1, 22050, w16, b"AIFC", b"NONE")),
        ("aifc sowt", aiff(16, 2, 22050, w16le, b"AIFC", b"sowt")),
        ("aifc raw", aiff(8, 1, 22050, w8, b"AIFC", b"raw ")),
        ("aifc ulaw", aiff(8, 1, 8000, w8, b"AIFC", b"ulaw")),
        ("aifc alaw", aiff(8, 1, 8000, w8, b"AIFC", b"alaw")),
        ("aifc fl32", aiff(32, 1, 44100, wf, b"AIFC", b"fl32")),
        ("aifc fl64", aiff(64, 1, 44100, wd, b"AIFC", b"fl64")),
        ("aiff 24", aiff(24, 1, 44100, w24)),
        ("aiff 32", aiff(32, 1, 44100, w32)),
        ("aifc ohne FVER", aiff(16, 1, 22050, w16, b"AIFC", b"NONE", ohne=("FVER",))),
        ("ohne SSND", aiff(16, 1, 22050, w16, ohne=("SSND",))),
        ("ohne COMM", aiff(16, 1, 22050, w16, ohne=("COMM",))),
        ("rate 0", aiff(16, 1, 0, w16)),
        ("12 bit", aiff(12, 1, 22050, w16)),
        ("zu kurz", aiff(16, 1, 22050, w16, rahmen=99999)),
        ("etiketten", aiff(16, 1, 22050, w16, extra=block(b"NAME", b"Titel x")
                           + block(b"AUTH", b"Wer") + block(b"(c) ", b"2026 jemand"))),
        ("voc 8", kopf + vblk(1, bytes([156, 0]) + u8) + b"\0"),
        ("voc stille", kopf + vblk(1, bytes([156, 0]) + u8[:500])
         + vblk(3, struct.pack("<HB", 300, 200)) + vblk(2, u8[500:900]) + b"\0"),
        ("voc 16 stereo", kopf + vblk(9, struct.pack("<IBB", 44100, 16, 2) + b"\0" * 6
                                       + w16le[:4000]) + b"\0"),
        ("voc erweitert", kopf + vblk(8, struct.pack("<HBB", 65536 - 256000000 // 44100, 0, 1))
         + vblk(1, bytes([0, 0]) + u8) + b"\0"),
        ("voc raten", kopf + vblk(1, bytes([156, 0]) + u8[:100])
         + vblk(1, bytes([157, 0]) + u8[:100]) + b"\0"),
        ("voc kennung", b"Creative Voice Filx\x1a" + kopf[20:] + vblk(1, bytes([156, 0]) + u8)),
        ("voc leer", kopf + b"\0"),
    ]
    pg.mixer.init(44100, -16, 2)
    aus = []
    for name, d in faelle:
        try:
            s = pg.mixer.Sound(file=io.BytesIO(d))
            roh = s.get_raw()
            aus.append((name, "Sound", len(roh), round(s.get_length(), 6),
                        hashlib.sha256(roh).hexdigest()[:12]))
        except Exception as e:  # noqa: BLE001
            aus.append((name, "Sound", type(e).__name__, str(e)))
        try:
            pg.mixer.music.load(io.BytesIO(d))
            aus.append((name, "music", pg.mixer.music.get_metadata()))
        except Exception as e:  # noqa: BLE001
            aus.append((name, "music", type(e).__name__, str(e)))
    if sum(1 for a in aus if a[1] == "Sound" and len(a) == 5) < 20:
        raise RuntimeError("AIFF/VOC: zu wenige geladene Klaenge")
    return aus


def klang_module(pg):
    """MOD/S3M/IT und Daten ohne Kennung - auf beiden Wegen.

    Bis Runde 19 las rustgame nur XM; ein ProTracker-Modul (`M.K.` bei
    Byte 1080) gab `Unrecognized audio format`, wo pygame es ueber libxmp
    spielt. Und `music.load` mit unbekannten Daten sagt bei pygame `XMP:
    Unrecognized file format` (libxmp ist der letzte Leser, den
    `Mix_LoadMUS_RW` fuer alles ohne Kennung probiert), ein erkanntes,
    kaputtes Modul `XMP: Error loading file`. Gemessen wird Laden,
    Liedlaenge (libxmp und xmrs rechnen denselben Ablauf) und Fehlertext.
    """
    import io
    import struct

    def modul(kennung=b"M.K.", muster=2, ohne_probe=False):
        d = b"testmod".ljust(20, b"\0")
        probe_ = bytes((60 if (i // 20) % 2 else -60) & 0xFF for i in range(2000))
        d += b"probe".ljust(22, b"\0") + struct.pack(">HBBHH", 1000, 0, 64, 0, 1)
        d += (b"\0" * 22 + struct.pack(">HBBHH", 0, 0, 0, 0, 1)) * 30
        d += bytes([muster, 127]) + bytes(range(muster)).ljust(128, b"\0") + kennung
        for _ in range(muster):
            for z in range(64):
                d += (struct.pack(">HBB", 428, 0x10, 0) + b"\0" * 12) if z % 16 == 0 else b"\0" * 16
        return d if ohne_probe else d + probe_

    faelle = [("mod 2", modul()), ("mod 4", modul(muster=4)), ("mod 6CHN", modul(b"6CHN")),
              ("mod 8CHN", modul(b"8CHN")), ("mod ohne probe", modul(ohne_probe=True)),
              ("text", b"hello world 12345"), ("nullen", b"\0" * 40),
              ("GIF", b"GIF89a" + b"\0" * 30), ("RIFF AVI", b"RIFF\0\0\0\0AVI " + b"\0" * 20),
              ("IMPM kaputt", b"IMPM" + b"\0" * 40),
              ("SCRM ohne Typ", b"\0" * 44 + b"SCRM" + b"\0" * 20),
              ("XM kaputt", b"Extended Module: " + b"\0" * 60), ("MThd", b"MThd" + b"\0" * 30),
              ("elf Bytes", b"x" * 11)]
    pg.mixer.init(44100, -16, 2)
    aus = []
    for name, d in faelle:
        try:
            s = pg.mixer.Sound(file=io.BytesIO(d))
            aus.append((name, "Sound", round(s.get_length(), 2)))
        except Exception as e:  # noqa: BLE001
            aus.append((name, "Sound", type(e).__name__, str(e)))
        try:
            pg.mixer.music.load(io.BytesIO(d))
            aus.append((name, "music", "geladen"))
        except Exception as e:  # noqa: BLE001
            aus.append((name, "music", type(e).__name__, str(e)))
    if sum(1 for a in aus if a[1] == "Sound" and len(a) == 3) < 4:
        raise RuntimeError("Trackermodule: zu wenige geladen")
    return aus


def text_kursiv(pg):
    """Kursiv **kuenstlich** (normaler Schnitt + `set_italic`) und auf
    einem **echten** Kursivschnitt.

    SDL_ttf schert den **gehinteten** Umriss mit `FT_Outline_Transform`
    (`xy = 0x366A`) und rastert danach. Hier wurde frueher das fertige
    Graubild geschert; nach dem Hinting-Umbau von Runde 18 war das
    sichtbar falsch (`freetype_test.test_freetype_Font_get_rect`). Auf
    einem Schnitt, der schon kursiv ist, streicht `TTF_SetFontStyle` den
    Wunsch - dann darf **nichts** geschert werden.
    """
    import hashlib
    aus = []
    zeichen = [chr(c) for c in range(33, 127)] + ["Kursiv gesetzter Satz"]
    for schrift in ("bahnschrift", "segoeui", "consolas"):
        for g in (12, 18, 24, 36):
            for art in ("kuenstlich", "echt"):
                f = pg.font.SysFont(schrift, g, italic=(art == "echt"))
                f.set_italic(True)
                for z in zeichen:
                    s = f.render(z, True, (255, 255, 255))
                    aus.append((schrift, g, art, z, s.get_size(),
                                hashlib.sha256(pg.image.tobytes(
                                    s, "RGBA")[3::4]).hexdigest()[:12]))
    if len(aus) < 2000:
        raise RuntimeError("Kursiv: nur %d Eintraege" % len(aus))
    return aus


def text_stilflaggen(pg):
    """`get_bold`/`get_italic` sagen auch, was der **Schnitt** mitbringt.

    `TTF_GetFontStyle` meldet den Stil des Schnitts mit (FreeTypes
    `style_flags`: OS/2 `fsSelection`, sonst `head.macStyle`), und
    `TTF_SetFontStyle` streicht Fett bzw. Kursiv, wenn der Schnitt es
    schon ist. Darum ist `Font("consolab.ttf").get_bold()` wahr, ohne dass
    jemand `set_bold` gerufen haette - und `set_bold(False)` macht daraus
    kein mageres Consolas.
    """
    import os
    aus = []
    fonts = os.path.join(os.environ.get("WINDIR", r"C:\Windows"), "Fonts")
    for datei in ("consola.ttf", "consolab.ttf", "consolai.ttf",
                  "consolaz.ttf", "segoeui.ttf", "segoeuii.ttf",
                  "segoeuib.ttf", "arialbd.ttf", "ariali.ttf"):
        pfad = os.path.join(fonts, datei)
        if not os.path.exists(pfad):
            aus.append((datei, "fehlt"))
            continue
        f = pg.font.Font(pfad, 14)
        a = (f.get_bold(), f.get_italic(), f.size("Hallo"))
        f.set_bold(False)
        f.set_italic(False)
        b = (f.get_bold(), f.get_italic(), f.size("Hallo"))
        f.set_bold(True)
        f.set_italic(True)
        c = (f.get_bold(), f.get_italic(), f.size("Hallo"), f.bold, f.italic)
        aus.append((datei, a, b, c))
    for name in ("consolas", "segoeui", "bahnschrift", "arial", "couriernew"):
        for fett in (False, True):
            for schraeg in (False, True):
                f = pg.font.SysFont(name, 14, bold=fett, italic=schraeg)
                aus.append((name, fett, schraeg, f.get_bold(), f.get_italic(),
                            f.size("Hallo Welt"), f.get_linesize()))
    if len(aus) < 25:
        raise RuntimeError("Stilflaggen: nur %d Eintraege" % len(aus))
    return aus


# ---------------------------------------------------------------------------
#  Kaputte Dateien (Runde 18)
# ---------------------------------------------------------------------------
#
# Bis Runde 18 hatte keine Probe eine **kaputte** Klang- oder Bilddatei
# gefragt, und dort standen 38 deutsche Saetze, ein Absturz (TGA mit 8 Bit
# Echtfarbe) und ein WAV-Leser, wo pygame zwei hat. Die Faelle hier sind die
# gemessenen, aus Bytes gebaut - keine Datei auf der Platte, kein Pfad im
# Satz.
#
# **Weggelassen** sind die Faelle, in denen pygame einen *liegengebliebenen*
# SDL-Fehler meldet: ein GIF ohne Bild sagt dort `JPEG loading error`, ein
# kaputtes TIFF `Surface doesn't have a colorkey` - der Satz des letzten
# fehlgeschlagenen Aufrufs davor. Das haengt an der Reihenfolge der Proben
# und ist nichts, was man nachbaut.

def _riff_wav(*bloecke):
    import struct
    inhalt = b"WAVE" + b"".join(bloecke)
    return b"RIFF" + struct.pack("<I", len(inhalt)) + inhalt


def _riff_block(name, daten):
    import struct
    return (name + struct.pack("<I", len(daten)) + daten
            + (b"\x00" if len(daten) & 1 else b""))


def _wav_fmt(art=1, spuren=1, rate=22050, bits=16, extra=b""):
    import struct
    ausr = spuren * bits // 8
    return _riff_block(b"fmt ", struct.pack(
        "<HHIIHH", art, spuren, rate, rate * ausr, ausr, bits) + extra)


def klang_kaputt(pg):
    """Kaputte Klaenge auf **beiden** Wegen: `Sound` und `music.load`.

    pygame hat zwei WAV-Leser (`SDL_wave.c` fuer `Sound`, `music_wav.c`
    fuer `music`) mit verschiedener Reihenfolge und verschiedenen Saetzen;
    `Mix_LoadWAV_RW` liest erst vier Bytes, `Mix_LoadMUS_RW` zwoelf. Dazu
    A-law und mu-law (G.711, alle 256 Werte) und Nicht-WAV-Muell auf dem
    `Sound`-Weg, der dort immer `Unrecognized audio format` heisst. Die
    Saetze der Musik-Dekodierer stehen in `musik_kaputt`.
    """
    import hashlib
    import io
    import struct
    if not pg.mixer.get_init():
        pg.mixer.init(22050, -16, 2)
    daten = _riff_block(b"data", b"\x00" * 64)
    gut = _wav_fmt()
    faelle = [
        ("ohne fmt", _riff_wav(daten)),
        ("ohne data", _riff_wav(gut)),
        ("data vor fmt", _riff_wav(daten, gut)),
        ("fmt zu kurz", _riff_wav(_riff_block(b"fmt ", b"\x01\x00\x01\x00\x22\x56\x00\x00"), daten)),
        ("fmt zu kurz ohne data", _riff_wav(_riff_block(b"fmt ", b"\x01\x00" * 4))),
        ("0 Spuren", _riff_wav(_wav_fmt(spuren=0), daten)),
        ("300 Spuren", _riff_wav(_wav_fmt(spuren=300), daten)),
        ("Rate 0", _riff_wav(_wav_fmt(rate=0), daten)),
        ("0 Spuren ohne data", _riff_wav(_wav_fmt(spuren=0))),
        ("PCM 0 Bit", _riff_wav(_riff_block(b"fmt ", struct.pack("<HHIIHH", 1, 1, 22050, 44100, 2, 0)), daten)),
        ("PCM 13 Bit", _riff_wav(_wav_fmt(bits=13), daten)),
        ("Kennung 0xABCD", _riff_wav(_wav_fmt(art=0xABCD), daten)),
        ("Kennung 99 ohne data", _riff_wav(_wav_fmt(art=99))),
        ("Kennung 99, 0 Spuren", _riff_wav(_wav_fmt(art=99, spuren=0), daten)),
        ("Float 16", _riff_wav(_wav_fmt(art=3, bits=16, extra=b"\x00\x00"), daten)),
        ("Float 64", _riff_wav(_wav_fmt(art=3, bits=64, extra=b"\x00\x00"), daten)),
        ("RIFF ohne WAVE (Sound)", b"RIFF\x04\x00\x00\x00AVI "),
        ("gut", _riff_wav(gut, daten)),
    ]
    kurz = [b"", b"RI", b"RIFF", b"RIFFxx", b"RIFF\x00\x00\x00\x00WAV",
            b"OggS", b"abcdefgh", b"WAVEabcdabcd"]
    muell = [b"wvpk" + b"\x00" * 32, b"Extended Module: " + b"\x00" * 64,
             b"OggS" + b"\x00" * 60, b"fLaC" + b"\x00" * 60,
             b"ID3\x04\x00\x00\x00\x00\x00\x00" + b"\x00" * 32,
             b"abcdefghijklmnop"]

    def frag(f, d):
        try:
            r = f(io.BytesIO(d))
            return "ok" if r is None else "ok %.6f" % r.get_length()
        except Exception as e:                       # noqa: BLE001
            return "%s: %s" % (type(e).__name__, e)
    aus = []
    for name, d in faelle:
        aus.append((name, frag(pg.mixer.Sound, d),
                    frag(pg.mixer.music.load, d)
                    if not name.endswith("(Sound)") else "-"))
    for d in kurz:
        aus.append((repr(d), frag(pg.mixer.Sound, d),
                    frag(pg.mixer.music.load, d) if d != b"WAVEabcdabcd" else "-"))
    for d in muell:
        aus.append((repr(d[:8]), frag(pg.mixer.Sound, d), "-"))
    # A-law (6) und mu-law (7): jeder Bytewert einmal.
    for art in (6, 7):
        d = _riff_wav(_wav_fmt(art=art, rate=8000, bits=8, extra=b"\x00\x00"),
                      _riff_block(b"fact", struct.pack("<I", 256)),
                      _riff_block(b"data", bytes(range(256))))
        try:
            s = pg.mixer.Sound(io.BytesIO(d))
            aus.append(("G.711 %d" % art, s.get_length(),
                        hashlib.sha256(s.get_raw()).hexdigest()[:16]))
        except Exception as e:                       # noqa: BLE001
            aus.append(("G.711 %d" % art, "%s: %s" % (type(e).__name__, e)))
    if len(aus) < 30:
        raise RuntimeError("Klang kaputt: nur %d Eintraege" % len(aus))
    return tuple(aus)


def musik_kaputt(pg):
    """Was die **Musik-Dekodierer** zu kaputten Daten sagen - eine Grenze.

    Auf dem Weg von `music` meldet sich bei pygame der Dekodierer selbst,
    und seine Saetze haengen an den Innereien von stb_vorbis, dr_flac,
    minimp3, libxmp und wavpack (`stb_vorbis_open_rwops:
    VORBIS_unexpected_eof`, `XMP: Error loading file`, auch `Out of
    memory`). Hier stehen andere Dekodierer. Dazu AIFF, das hier fehlt.
    """
    import io
    if not pg.mixer.get_init():
        pg.mixer.init(22050, -16, 2)
    faelle = [b"wvpk" + b"\x00" * 32, b"Extended Module: " + b"\x00" * 64,
              b"OggS" + b"\x00" * 60, b"fLaC" + b"\x00" * 60,
              b"ID3\x04\x00\x00\x00\x00\x00\x00" + b"\x00" * 32,
              b"abcdefghijklmnop", b"RIFF\x04\x00\x00\x00AVI ",
              b"WAVEabcdabcd", b"FORM\x04\x00\x00\x00AIFF"]
    aus = []
    for d in faelle:
        try:
            pg.mixer.music.load(io.BytesIO(d))
            r = "ok"
        except Exception as e:                       # noqa: BLE001
            r = "%s: %s" % (type(e).__name__, e)
        try:
            pg.mixer.Sound(io.BytesIO(d))
            k = "ok"
        except Exception as e:                       # noqa: BLE001
            k = "%s: %s" % (type(e).__name__, e)
        aus.append((repr(d[:8]), r, k if d.startswith(b"FORM") else "-"))
    return tuple(aus)


def _tga(bits, art=2, farbtafel=0, farben=0, tafelbits=0, w=2, h=2,
         daten=b"", deskriptor=0):
    import struct
    return struct.pack("<BBBHHBHHHHBB", 0, farbtafel, art, 0, farben,
                       tafelbits, 0, 0, w, h, bits, deskriptor) + daten


def bild_kaputt(pg):
    """Kaputte Bilder - SDL_images Saetze und seine Nachsicht.

    TGA wird **nur** mit dem Typ `tga` gelesen (SDL_images einziges Format
    ohne Kennung), dann aber ohne jede Lesepruefung: was fehlt, bleibt
    null, und RLE wiederholt sein letztes Paket. PNM kennt keine 16 Bit
    und skaliert mit `Uint8`-Ueberlauf. PCX und LBM sind dagegen streng.
    """
    import io
    import struct
    lbm_kopf = _riff_be(b"BMHD", struct.pack(
        ">HHhhBBBBHBBhh", 2, 2, 0, 0, 1, 0, 0, 0, 0, 1, 1, 2, 2))
    pcx = bytearray(128)
    pcx[0:4] = b"\x0a\x05\x01\x08"
    struct.pack_into("<HHHH", pcx, 4, 0, 0, 3, 3)
    pcx[65] = 1
    struct.pack_into("<H", pcx, 66, 4)
    pcx_neg = bytearray(pcx)
    struct.pack_into("<HHHH", pcx_neg, 4, 5, 5, 3, 3)
    pcx_null = bytearray(pcx)
    struct.pack_into("<HHHH", pcx_null, 4, 4, 0, 3, 3)
    pcx_roh = bytearray(pcx)
    pcx_roh[2] = 0
    gut_tga = _tga(24, daten=bytes(range(1, 13)))
    faelle = [
        # PNM
        ("pnm kurz", b"P5\n2 2\n255\n\x00", None),
        ("pnm breite 0", b"P5\n0 2\n255\n", None),
        ("pnm keine zahl", b"P5\nx 2\n255\n", None),
        ("pnm hoechst 70000", b"P5\n1 1\n70000\n\x00\x00", None),
        ("pnm hoechst 256", b"P5\n1 1\n256\n\x01\x02", None),
        ("pnm hoechst 0", b"P5\n1 1\n0\n\x00", None),
        ("pnm ohne hoechst", b"P5\n1 1\n", None),
        ("pnm hoechst 15", b"P5\n2 1\n15\n\x0f\x20", None),
        ("pnm ascii kurz", b"P3\n2 1\n255\n1 2 3 4", None),
        ("pbm kurz", b"P4\n9 2\n\x00", None),
        # QOI
        ("qoi kurz", b"qoif" + struct.pack(">II", 2, 2) + b"\x04\x00" + b"\x00" * 3, None),
        ("qoi breite 0", b"qoif" + struct.pack(">II", 0, 2) + b"\x04\x00" + b"\x00" * 8, None),
        # XPM
        ("xpm ohne zeile", b"/* XPM */\nstatic char *x[] = {\n};\n", None),
        ("xpm kopf halb", b'/* XPM */\nstatic char *x[] = {\n"1 1",\n};\n', None),
        ("xpm kopf muell", b'/* XPM */\nstatic char *x[] = {\n"a b c d",\n};\n', None),
        ("xpm zeilen fehlen", b'/* XPM */\nstatic char *x[] = {\n"2 2 1 1",\n"a c #ff0000",\n"aa"\n};\n', None),
        # PCX
        ("pcx kurz", bytes(pcx) + b"\x01", None),
        ("pcx roh kurz", bytes(pcx_roh) + b"\x00" * 5, None),
        ("pcx breite negativ", bytes(pcx_neg) + b"\x00" * 32, None),
        ("pcx breite 0", bytes(pcx_null) + b"\x00" * 16, None),
        ("pcx 1 bit 1 ebene", _pcx(1, 1), None),
        ("pcx 1 bit 2 ebenen", _pcx(1, 2), None),
        ("pcx 1 bit 4 ebenen", _pcx(1, 4), None),
        ("pcx 2 bit", _pcx(2, 1), None),
        ("pcx 4 bit", _pcx(4, 1), None),
        ("pcx 8 bit 2 ebenen", _pcx(8, 2), None),
        ("pcx 8 bit mit 12", _pcx(8, 1, tafel=b"\x0c" + bytes(range(256)) * 3), None),
        ("pcx 8 bit ohne 12 am ende", _pcx(8, 1, tafel=bytes(range(200)) * 4), None),
        # LBM
        ("lbm ohne bmhd", b"FORM" + struct.pack(">I", 4) + b"ILBM", None),
        ("lbm body kurz", _lbm(lbm_kopf, _riff_be(b"BODY", b"\x00")), None),
        ("lbm bmhd kurz", _lbm(_riff_be(b"BMHD", b"\x00\x02"), _riff_be(b"BODY", b"\x00" * 8)), None),
        ("lbm ohne body", _lbm(lbm_kopf), None),
        # JPEG
        ("jpeg nur soi", b"\xff\xd8\xff\xd9", None),
        # TGA: nur auf Wunsch
        ("tga ohne typ", gut_tga, None),
        ("tga typ tga", gut_tga, "x.tga"),
        ("tga typ TGA", gut_tga, "foo.TGA"),
        ("tga typ x.png.tga", gut_tga, "x.png.tga"),
        ("tga typ x.tga.png", gut_tga, "x.tga.png"),
        ("png als tga", None, "x.tga"),
        # TGA: was der Kopf nicht hergibt
        ("tga 13 bit", _tga(13), "x.tga"),
        ("tga 8 bit echtfarbe", _tga(8, daten=bytes(4)), "x.tga"),
        ("tga ursprung rechts", _tga(24, daten=bytes(12), deskriptor=0x10), "x.tga"),
        ("tga verschraenkt", _tga(24, daten=bytes(12), deskriptor=0x40), "x.tga"),
        ("tga grau 16", _tga(16, art=3, daten=bytes(8)), "x.tga"),
        ("tga 300 farben", _tga(8, art=1, farbtafel=1, farben=300, tafelbits=24, daten=bytes(904)), "x.tga"),
        ("tga tafel ohne kennung", _tga(8, art=1, daten=bytes(4)), "x.tga"),
        ("tga kopf kurz", _tga(24)[:10], "x.tga"),
        ("tga breite 0", _tga(24, w=0, h=2), "x.tga"),
        # TGA: was SDL_image trotzdem liest
        ("tga daten kurz", _tga(24, w=4, h=4, daten=bytes(range(1, 6))), "x.tga"),
        ("tga oben kurz", _tga(24, daten=bytes(range(1, 8)), deskriptor=0x20), "x.tga"),
        # Nicht hier: eine TGA-Tafel, die **ueber das Dateiende** reicht.
        # SDL_image liest sie in einen `malloc`-Puffer ohne Vorbelegung -
        # im frischen Prozess stehen dort Nullen, im langen Lauf dieses
        # Werkzeugs gemessen 37, 165, 232. Was fehlt, ist hier null.
        ("tga tafel ohne pixel", _tga(8, art=1, farbtafel=1, farben=3, tafelbits=24, daten=bytes(range(1, 10))), "x.tga"),
        ("tga tafel 16 bit", _tga(8, art=1, farbtafel=1, farben=2, tafelbits=16, daten=struct.pack("<HH", 0x7FFF, 0x4210) + b"\x00\x01\x00\x01"), "x.tga"),
        ("tga tafel 32 bit", _tga(8, art=1, farbtafel=1, farben=2, tafelbits=32, daten=b"\x01\x02\x03\x04\x05\x06\x07\x08\x00\x01\x00\x01"), "x.tga"),
        ("tga grau mit tafel", _tga(8, art=3, farbtafel=1, farben=2, tafelbits=24, daten=b"\x10\x20\x30\x40\x50\x60\x05\x06\x07\x08"), "x.tga"),
        ("tga rle wiederholt", _tga(24, art=10, w=4, h=4, daten=b"\x83\x10\x20\x30"), "x.tga"),
        ("tga rle roh kurz", _tga(24, art=10, w=4, h=2, daten=b"\x01\x10\x20\x30\x40\x50\x60"), "x.tga"),
        ("tga rle roh halb", _tga(24, art=10, w=4, h=2, daten=b"\x02\x10\x20\x30\x40\x50"), "x.tga"),
        ("tga rle pixel halb", _tga(24, art=10, w=4, h=2, daten=b"\x81\x10\x20\x30\x83\x70"), "x.tga"),
        ("tga rle leer", _tga(24, art=10, w=2, h=1), "x.tga"),
        ("tga 16 bit pixel", _tga(16, w=2, h=1, daten=struct.pack("<HH", 0xFFFF, 0x0421)), "x.tga"),
        ("tga 32 bit", _tga(32, w=1, h=1, daten=b"\x01\x02\x03\x04"), "x.tga"),
        ("tga indiziert rle", _tga(8, art=9, farbtafel=1, farben=1, tafelbits=24, daten=b"\x01\x02\x03\x83\x00"), "x.tga"),
    ]
    png = io.BytesIO()
    pg.image.save(pg.Surface((2, 2)), png, "x.png")
    aus = []
    for name, d, typ in faelle:
        if d is None:
            d = png.getvalue()
        try:
            if typ is None:
                s = pg.image.load(io.BytesIO(d))
            else:
                s = pg.image.load(io.BytesIO(d), typ)
            tafel = s.get_palette() if s.get_bitsize() == 8 else None
            schluessel = s.get_colorkey()
            r = ("ok", s.get_size(), s.get_bitsize(),
                 bool(s.get_flags() & pg.SRCALPHA),
                 len(tafel) if tafel is not None else None,
                 tuple(tuple(c) for c in tafel[:4]) if tafel else None,
                 tuple(schluessel) if schluessel else None,
                 pg.image.tobytes(s, "RGBA").hex()[:96])
        except Exception as e:                       # noqa: BLE001
            r = "%s: %s" % (type(e).__name__, e)
        aus.append((name, r))
    # Der Hinweis muss Text sein.
    for h in (5, b"x.tga"):
        try:
            pg.image.load(io.BytesIO(gut_tga), h)
            aus.append(("hinweis %r" % (h,), "ok"))
        except Exception as e:                       # noqa: BLE001
            aus.append(("hinweis %r" % (h,), "%s: %s" % (type(e).__name__, e)))
    # Bildfolge mit Typ.
    try:
        aus.append(("load_animation tga", len(pg.image.load_animation(io.BytesIO(gut_tga), "x.tga"))))
    except Exception as e:                           # noqa: BLE001
        aus.append(("load_animation tga", "%s: %s" % (type(e).__name__, e)))
    if len(aus) < 55:
        raise RuntimeError("Bild kaputt: nur %d Eintraege" % len(aus))
    return tuple(aus)


def bild_kaputt_grenze(pg):
    """Kaputte Bilder, an denen schon die **Erkennung** entscheidet.

    **Bis Runde 19 eine benannte Grenze mit fuenf Eintraegen** - alle
    gefallen: `IMG_isJPG` laeuft beim Erkennen durch die Marker (ein
    kaputter Kopf ist `Unsupported image format`, nicht `JPEG loading
    error`); ein GIF, das nach dem Kopf aufhoert, sagt SDL_images `EOF /
    read error on image data` (rustgame liest GIF jetzt mit SDL_images
    eigenem LZW-Leser, `gifsdl.rs`); ein TGA mit einer Tafel der Laenge 0
    laedt (`get_at` schwarz, `tobytes` weiss - SDLs Farbfeld jenseits von
    `ncolors`); LBM vom Typ `PBM ` wird gelesen.
    """
    import io
    import struct
    faelle = [
        ("jpeg marker laenge 0", b"\xff\xd8\xff\xe0" + b"\x00" * 40, None),
        ("jpeg muell", b"\xff\xd8\xff\xdb\x00\x43" + b"\x00" * 70, None),
        ("gif nur kopf", b"GIF89a\x02\x00\x02\x00\x00\x00\x00", None),
        ("tga tafel leer", _tga(8, art=1, farbtafel=1, farben=0, tafelbits=24, daten=b"\x00\x01\x02\x03"), "x.tga"),
        ("lbm pbm", b"FORM" + struct.pack(">I", 4) + b"PBM ", None),
    ]
    aus = []
    for name, d, typ in faelle:
        try:
            s = pg.image.load(io.BytesIO(d), typ) if typ else pg.image.load(io.BytesIO(d))
            r = ("ok", s.get_size())
        except Exception as e:                       # noqa: BLE001
            r = "%s: %s" % (type(e).__name__, e)
        aus.append((name, r))
    return tuple(aus)


def bild_lbm(pg):
    """LBM/IFF - ILBM und PBM - wie `IMG_LoadLBM_RW`.

    Bis Runde 19 ein Nachbau, der 28 von 37 Dateien verfehlte: die Flaeche
    ist bei SDL_image **auf 16 Pixel aufgerundet** breit, ohne `CMAP` gilt
    SDLs Vorgabetafel (256 mal Weiss, kein Graukeil), die Maske ist eine
    zusaetzliche Ebene im Index, `mask & 2` setzt die Schluesselfarbe, EHB,
    HAM6, HAM8 und 24 Ebenen geben 24 Bit, und `PBM ` fehlte.
    """
    import hashlib
    import io
    import struct

    def blk(n, d):
        return n + struct.pack(">I", len(d)) + d + (b"\0" if len(d) % 2 else b"")

    def lbm(w, h, planes, cmap=None, mask=0, comp=0, tcolor=0, camg=None, pbm=False, body=None):
        bpl = ((w + 15) // 16) * 2
        if pbm:
            bpl *= 8
        ebenen = 1 if pbm else planes + (1 if mask & 1 else 0)
        if body is None:
            body = b""
            for y in range(h):
                for e in range(ebenen):
                    z = bytes(((x * 37 + y * 11 + e * 53) & 0xFF) for x in range(bpl))
                    if comp == 1:
                        z = b"".join(bytes([len(z[i:i + 128]) - 1]) + z[i:i + 128]
                                     for i in range(0, len(z), 128))
                    body += z
        kopf = struct.pack(">HHhhBBBBHBBhh", w, h, 0, 0, planes, mask, comp, 0, tcolor, 10, 11, w, h)
        d = (b"PBM " if pbm else b"ILBM") + blk(b"BMHD", kopf)
        if cmap is not None:
            d += blk(b"CMAP", cmap)
        if camg is not None:
            d += blk(b"CAMG", struct.pack(">I", camg))
        d += blk(b"BODY", body)
        return b"FORM" + struct.pack(">I", len(d)) + d

    def tafel(n):
        return bytes((i * 7 + k * 50) & 0xFF for i in range(n) for k in range(3))

    faelle = [("p%d w%d" % (p, w), lbm(w, 3, p, tafel(1 << p)))
              for p in (1, 2, 3, 4, 5, 8) for w in (5, 16, 23)]
    faelle += [("komprimiert", lbm(20, 4, 4, tafel(16), comp=1)),
               ("ohne cmap", lbm(8, 2, 3)), ("kurze cmap", lbm(8, 2, 4, tafel(5))),
               ("stencil", lbm(16, 2, 3, tafel(8), mask=1)),
               ("schluessel", lbm(16, 2, 3, tafel(8), mask=2, tcolor=3)),
               ("ehb", lbm(16, 2, 6, tafel(32), camg=0x80)),
               ("ham6", lbm(16, 2, 6, tafel(16), camg=0x800)),
               ("ham8", lbm(16, 2, 8, tafel(64), camg=0x800)),
               ("24", lbm(16, 2, 24)),
               ("pbm", lbm(16, 3, 8, tafel(256), pbm=True)),
               ("pbm w5", lbm(5, 3, 8, tafel(256), pbm=True)),
               ("pbm komprimiert", lbm(20, 3, 8, tafel(100), pbm=True, comp=1)),
               ("body kurz", lbm(16, 4, 4, tafel(16), body=b"\0\0\0")),
               ("cmap zu gross", lbm(8, 2, 4, bytes(800)))]
    aus = []
    for name, d in faelle:
        try:
            s = pg.image.load(io.BytesIO(d), "x.lbm")
            w, h = s.get_size()
            tf = s.get_palette() if s.get_bitsize() == 8 else None
            aus.append((name, (w, h), s.get_bitsize(), s.get_colorkey(),
                        [tuple(c) for c in tf] if tf else None,
                        hashlib.sha256(pg.image.tobytes(s, "RGBA")).hexdigest()[:12]))
        except Exception as e:  # noqa: BLE001
            aus.append((name, type(e).__name__, str(e)))
    if sum(1 for a in aus if len(a) == 6) < 25:
        raise RuntimeError("LBM: zu wenige geladen")
    return aus


def bild_gif(pg):
    """GIF wie SDL_image: `image.load` und `image.load_animation`.

    Bis Runde 19 las rustgame GIF ueber die Kiste `gif` und baute eine
    Flaeche in **Bildschirmgroesse**; SDL_image gibt den ersten Rahmen in
    **Rahmengroesse**, mit einer 256er-Tafel, deren ungesetzte Eintraege
    weiss sind, ein abgeschnittener LZW-Strom ist ein Teilbild, und
    `load_animation` normalisiert auf ARGB8888/RGB888 in der Groesse des
    ersten Rahmens. Jetzt liest `gifsdl.rs` SDL_images eigenen Leser.
    Ausgelassen ist eine Datei ohne Bild: dort meldet pygame den Satz, der
    vom **vorigen** Aufruf liegen geblieben ist.
    """
    import glob
    import hashlib
    import io
    import os
    import struct

    def lzw(pixel, mincode):
        clear, end = 1 << mincode, (1 << mincode) + 1
        tab = {bytes([i]): i for i in range(clear)}
        naechst, size = end + 1, mincode + 1
        codes = [(clear, size)]
        w = b""
        for p in pixel:
            wc = w + bytes([p])
            if wc in tab:
                w = wc
                continue
            codes.append((tab[w], size))
            if naechst < 4096:
                tab[wc] = naechst
                naechst += 1
                if naechst - 1 == (1 << size) and size < 12:
                    size += 1
            w = bytes([p])
        if w:
            codes.append((tab[w], size))
        codes.append((end, size))
        bits, n, aus = 0, 0, bytearray()
        for c, s in codes:
            bits |= c << n
            n += s
            while n >= 8:
                aus.append(bits & 0xFF)
                bits >>= 8
                n -= 8
        if n:
            aus.append(bits & 0xFF)
        return bytes(aus)

    def gif(schirm, rahmen, globale=None):
        flags = 0
        if globale is not None:
            bits = max(0, (len(globale) // 3 - 1).bit_length() - 1)
            flags = 0x80 | bits
            globale = globale.ljust(3 * (2 << bits), b"\0")
        d = b"GIF89a" + struct.pack("<HHBBB", schirm[0], schirm[1], flags, 0, 0) + (globale or b"")
        for r in rahmen:
            if "gce" in r:
                disp, delay, t = r["gce"]
                d += (b"!\xf9\x04" + bytes([(disp << 2) | (1 if t is not None else 0)])
                      + struct.pack("<H", delay) + bytes([t or 0]) + b"\0")
            pix, w, h = r["pix"], r["w"], r["h"]
            lf = 0x40 if r.get("ilace") else 0
            if r.get("ilace"):
                reihen = (list(range(0, h, 8)) + list(range(4, h, 8)) + list(range(2, h, 4))
                          + list(range(1, h, 2)))
                pix = b"".join(pix[z * w:(z + 1) * w] for z in reihen)
            daten = lzw(pix, 2)
            if r.get("halb"):
                daten = daten[:len(daten) // 2]
            d += b"," + struct.pack("<HHHHB", r["x"], r["y"], w, h, lf) + b"\x02"
            d += b"".join(bytes([len(daten[i:i + 255])]) + daten[i:i + 255]
                          for i in range(0, len(daten), 255)) + b"\0"
        return d + b";"

    t4 = bytes([255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 0])

    def m(w, h, n):
        return bytes(((x * 3 + y * 5) % n) for y in range(h) for x in range(w))

    def R(**k):
        return dict(dict(x=0, y=0, w=8, h=6, pix=m(8, 6, 4)), **k)

    faelle = [
        ("einfach", gif((8, 6), [R()], t4)),
        ("schirm groesser", gif((20, 12), [R(x=3, y=2)], t4)),
        ("ohne tafel", gif((8, 6), [R()])),
        ("transparent", gif((8, 6), [R(gce=(0, 10, 2))], t4)),
        ("zeilensprung", gif((9, 19), [R(w=9, h=19, pix=m(9, 19, 4), ilace=True)], t4)),
        ("abgeschnitten", gif((16, 16), [R(w=16, h=16, pix=m(16, 16, 4), halb=True)], t4)),
        ("nur kopf", b"GIF89a\x02\x00\x02\x00\x00\x00\x00"),
        ("GIF88a", b"GIF88a" + b"\0" * 20),
        ("anim transparent", gif((8, 6), [R(gce=(2, 1, 3)),
                                          R(x=1, y=1, w=5, h=4, pix=m(5, 4, 4), gce=(1, 20, 3)),
                                          R(x=0, y=0, w=3, h=3, pix=m(3, 3, 3), gce=(3, 7, 3)),
                                          R(x=4, y=2, w=4, h=4, pix=m(4, 4, 2))], t4)),
        ("anim zurueck", gif((8, 6), [R(gce=(1, 4, None)),
                                      R(x=1, y=1, w=4, h=4, pix=m(4, 4, 2), gce=(3, 4, None)),
                                      R(x=3, y=0, w=4, h=4, pix=m(4, 4, 3), gce=(1, 4, None))], t4)),
    ]
    ordner = os.path.dirname(_echt_pygame_datei())
    for p in sorted(glob.glob(os.path.join(ordner, "**", "*.gif"), recursive=True)):
        with open(p, "rb") as f:
            faelle.append((os.path.basename(p), f.read()))

    def kennung(s):
        tf = s.get_palette() if s.get_bitsize() == 8 else None
        return (s.get_size(), s.get_bitsize(), s.get_flags() & 0xFFFF, s.get_colorkey(),
                len(tf) if tf else None,
                hashlib.sha256(pg.image.tobytes(s, "RGBA")).hexdigest()[:12])

    aus = []
    for name, d in faelle:
        try:
            aus.append((name, "load", kennung(pg.image.load(io.BytesIO(d)))))
        except Exception as e:  # noqa: BLE001
            aus.append((name, "load", type(e).__name__, str(e)))
        try:
            a = pg.image.load_animation(io.BytesIO(d))
            aus.append((name, "anim", [(kennung(s), t) for s, t in a]))
        except Exception as e:  # noqa: BLE001
            aus.append((name, "anim", type(e).__name__, str(e)))
    if sum(1 for a in aus if a[1] == "load" and len(a) == 3) < 8:
        raise RuntimeError("GIF: zu wenige geladen")
    return aus


def bild_ico(pg):
    """ICO und CUR - `LoadICOCUR_RW` aus `IMG_bmp.c`.

    Bis Runde 19 fehlten beide Formate: jede `.ico` und `.cur` gab
    `Unsupported image format`. Gebaut werden alle fuenf Bittiefen, ein
    Verzeichnis mit mehreren Eintraegen (SDL_image nimmt den mit der
    groessten Farbzahl, nicht das groesste Bild), ein PNG-Eintrag, ein
    komprimierter und ein falscher Kopf; dazu Symbole und Zeiger aus
    Windows, soweit vorhanden.
    """
    import glob
    import hashlib
    import io
    import os
    import struct

    def dib(w, h, bits, tafel=b"", farbe=0, ohne_maske=False, kompression=0, png=False):
        if png:
            return b"\x89PNG\r\n\x1a\n" + b"\0" * 40
        kopf = struct.pack("<IiiHHIIiiII", 40, w, 2 * h, 1, bits, kompression, 0, 0, 0, 0, 0)
        zeile = {1: (w + 7) // 8, 4: (w + 1) // 2, 8: w, 24: 3 * w, 32: 4 * w}[bits]
        zeile_p = (zeile + 3) // 4 * 4 if bits != 32 else zeile
        daten = bytes(((i * 29 + farbe * 7) & 0xFF) for i in range(zeile_p * h))
        maske = b"" if ohne_maske else bytes(((i * 13) & 0xFF) for i in range(((w + 7) // 8 + 3) // 4 * 4 * h))
        return kopf + tafel + daten + maske

    def datei(art, eintraege):
        kopf = struct.pack("<HHH", 0, art, len(eintraege))
        ofs = 6 + 16 * len(eintraege)
        verz, koerper = b"", b""
        for w, h, farben, d in eintraege:
            verz += struct.pack("<BBBBHHII", w % 256, h % 256, farben % 256, 0, 1, 32, len(d), ofs + len(koerper))
            koerper += d
        return kopf + verz + koerper

    tafel = lambda n: bytes(((i * 17 + k * 60) & 0xFF) if k < 3 else 0 for i in range(n) for k in range(4))
    faelle = [
        ("1 bit", datei(1, [(9, 5, 2, dib(9, 5, 1, tafel(2)))])),
        ("4 bit", datei(1, [(7, 6, 16, dib(7, 6, 4, tafel(16)))])),
        ("8 bit", datei(1, [(10, 4, 0, dib(10, 4, 8, tafel(256)))])),
        ("24 bit", datei(1, [(5, 5, 0, dib(5, 5, 24))])),
        ("32 bit", datei(1, [(6, 3, 0, dib(6, 3, 32))])),
        ("cur", datei(2, [(8, 8, 0, dib(8, 8, 32))])),
        ("farbzahl entscheidet", datei(1, [(4, 4, 16, dib(4, 4, 4, tafel(16), 1)),
                                           (8, 8, 2, dib(8, 8, 1, tafel(2), 2)),
                                           (6, 6, 0, dib(6, 6, 32, farbe=3))])),
        ("gleichstand", datei(1, [(4, 4, 0, dib(4, 4, 32, farbe=4)), (6, 6, 0, dib(6, 6, 32, farbe=5))])),
        ("png eintrag", datei(1, [(16, 16, 0, dib(16, 16, 32, png=True))])),
        ("komprimiert", datei(1, [(4, 4, 0, dib(4, 4, 32, kompression=1))])),
        ("16 bit", datei(1, [(4, 4, 0, struct.pack("<IiiHHIIiiII", 40, 4, 8, 1, 16, 0, 0, 0, 0, 0, 0) + b"\0" * 64)])),
        ("maske fehlt", datei(1, [(4, 4, 0, dib(4, 4, 32, ohne_maske=True))])),
        ("anzahl null", struct.pack("<HHH", 0, 1, 0) + b"\0" * 30),
    ]
    for p in sorted(glob.glob(r"C:\Windows\Cursors\*.cur"))[:4] + [r"C:\Windows\System32\OneDrive.ico"]:
        if os.path.exists(p):
            with open(p, "rb") as f:
                faelle.append((os.path.basename(p), f.read()))
    aus = []
    for name, d in faelle:
        try:
            s = pg.image.load(io.BytesIO(d))
            aus.append((name, s.get_size(), s.get_bitsize(), s.get_flags(),
                        hashlib.sha256(pg.image.tobytes(s, "RGBA")).hexdigest()[:12]))
        except Exception as e:  # noqa: BLE001
            aus.append((name, type(e).__name__, str(e)))
    if sum(1 for a in aus if len(a) == 5) < 8:
        raise RuntimeError("ICO: zu wenige geladen")
    return aus


def bild_xv_rgb332(pg):
    """XV-Vorschaubilder und RGB332.

    Zwei Funde in Runde 19: rustgame kannte XV (`P7 332`, `IMG_xv.c`)
    nicht, und eine 8-Bit-Flaeche **mit Masken** behandelte es als
    Palettenflaeche - `Surface((4, 1), 0, 8, (0xE0, 0x1C, 0x03, 0))` suchte
    beim Setzen die naechste Tafelfarbe, statt die Bits abzuschneiden.
    pygames `image.tobytes` stuerzt auf einer RGB332-Flaeche ab; darum
    wird hier nur `get_at`/`get_at_mapped` gelesen.
    """
    import io
    faelle = {
        "voll": b"P7 332\n#XVVERSION:Version 2.28\n#END_OF_COMMENTS\n16 16\n" + bytes(range(256)),
        "crlf": b"P7 332\r\n#END_OF_COMMENTS\r\n3 1\r\n" + b"\x10\x20\x30",
        "builtin": b"P7 332\n#BUILTIN:UNKNOWN\n#END_OF_COMMENTS\n3 1\n" + b"\x10\x20\x30",
        "zu kurz": b"P7 332\n#END_OF_COMMENTS\n4 4\n" + b"\x10\x20\x30",
        "ohne ende": b"P7 332\n# kommentar\n",
        "null": b"P7 332\n#END_OF_COMMENTS\n0 0\n",
        "falsch": b"P7 333\n#END_OF_COMMENTS\n1 1\n\x00",
    }
    aus = []
    for name, d in faelle.items():
        try:
            s = pg.image.load(io.BytesIO(d))
            w, h = s.get_size()
            aus.append((name, (w, h), s.get_bitsize(), s.get_masks(),
                        [tuple(s.get_at((x, y))) for y in range(h) for x in range(w)],
                        [s.get_at_mapped((x, y)) for y in range(h) for x in range(w)]))
        except Exception as e:  # noqa: BLE001
            aus.append((name, type(e).__name__, str(e)))
    f = pg.Surface((5, 1), 0, 8, (0xE0, 0x1C, 0x03, 0))
    for x, c in enumerate([(72, 182, 85), (255, 255, 255), (10, 200, 100), (31, 32, 63), (250, 5, 128)]):
        f.set_at((x, 0), c)
    aus.append(("rgb332", f.get_masks(), [tuple(f.get_at((x, 0))) for x in range(5)],
                [f.get_at_mapped((x, 0)) for x in range(5)], f.map_rgb((100, 100, 100))))
    if len([a for a in aus if len(a) == 6]) < 3:
        raise RuntimeError("XV: zu wenige geladen")
    return aus


def _echt_pygame_datei():
    """Der Pfad von **echtem** pygame - fuer seine Beispieldateien."""
    import pygame as _echt
    return _echt.__file__


def _pcx(bits, ebenen, w=8, h=2, tafel=b""):
    """Ein rohes PCX mit Kopffarben `0, 5, 10, ..` und Mustern als Pixeln."""
    import struct
    k = bytearray(128)
    k[0:4] = bytes([10, 5, 0, bits])
    struct.pack_into("<HHHH", k, 4, 0, 0, w - 1, h - 1)
    k[16:64] = bytes((i * 5) % 256 for i in range(48))
    k[65] = ebenen
    zeile = max(1, (w * bits + 7) // 8)
    struct.pack_into("<H", k, 66, zeile)
    return (bytes(k) + bytes((i * 37) % 256 for i in range(zeile * ebenen * h))
            + tafel)


def _riff_be(name, daten):
    """Ein IFF-Block (grosses Ende) - fuer LBM."""
    import struct
    return (name + struct.pack(">I", len(daten)) + daten
            + (b"\x00" if len(daten) & 1 else b""))


def _lbm(*bloecke):
    import struct
    inhalt = b"ILBM" + b"".join(bloecke)
    return b"FORM" + struct.pack(">I", len(inhalt)) + inhalt


def arc_winkelgrenzen(pg):
    """`draw.arc` mit Winkeln genau auf und neben den Achsen.

    pygame liest die zwei Winkel als `double` (`"O!OOdd|i"`); hier stand
    `f32`, und `math.pi` als `f32` ist **groesser** als pi. Genau an den
    Achsen, wo ein Bogen beginnt oder endet, entscheidet das letzte Bit
    ueber das Randpixel - gemessen lagen 1887 von 4116 Boegen daneben.
    Gefunden hat es PyGameZ: das Laecheln des Minesweeper-Smileys
    (`arc(.., math.pi, math.tau, 2)`).
    """
    import hashlib
    import math
    winkel = (0.0, math.pi / 2, math.pi, 1.5 * math.pi, math.tau, 3.2,
              -math.pi / 2, 2 * math.tau, 0.1, math.pi * 0.999, math.pi * 1.001)
    aus = []
    for (w, h) in ((14, 12), (20, 9), (7, 7)):
        for breite in (1, 2):
            for a in winkel:
                for b in winkel:
                    s = pg.Surface((40, 30))
                    r = pg.draw.arc(s, (255, 255, 255), (3, 4, w, h), a, b, breite)
                    aus.append((w, h, breite, round(a, 4), round(b, 4), tuple(r),
                                hashlib.sha256(pg.image.tobytes(s, "RGB")).hexdigest()[:12]))
    if len(aus) < 700:
        raise RuntimeError("arc-Winkelgrenzen: nur %d Eintraege" % len(aus))
    return aus


def aa_und_fuellen(pg):
    """Die vier Zeichenfunktionen, die es in der Huelle lange nicht gab.

    `aaline`, `aalines` und `aacircle` sind die **einzigen**
    kantengeglaetteten in `pygame.draw`; alles andere ist binaer
    rasterisiert. Gerade deshalb muessen sie hier stehen: ein Fehler in
    der Glaettung faellt auf keinem anderen Weg auf.
    """
    aus = []
    for name, f in (
        ("aaline", lambda s: pg.draw.aaline(s, (255, 0, 0), (1, 1), (18, 13))),
        ("aaline komma",
         lambda s: pg.draw.aaline(s, (255, 0, 0), (1.5, 1.25), (18.75, 13.5))),
        ("aaline breit",
         lambda s: pg.draw.aaline(s, (255, 0, 0), (2, 2), (18, 13), 3)),
        ("aalines",
         lambda s: pg.draw.aalines(s, (0, 255, 0), False, [(1, 1), (15, 3), (4, 17)])),
        ("aalines zu",
         lambda s: pg.draw.aalines(s, (0, 255, 0), True, [(1, 1), (15, 3), (4, 17)])),
        ("aacircle", lambda s: pg.draw.aacircle(s, (0, 0, 255), (11, 10), 8)),
        ("aacircle ring", lambda s: pg.draw.aacircle(s, (0, 0, 255), (11, 10), 8, 2)),
        ("aacircle viertel",
         lambda s: pg.draw.aacircle(s, (0, 0, 255), (11, 10), 8, 0, draw_top_right=True)),
    ):
        s = pg.Surface((24, 20), pg.SRCALPHA)
        r = f(s)
        aus.append((name, tuple(r), pg.image.tobytes(s, "RGBA").hex()))
    s = pg.Surface((24, 20), pg.SRCALPHA)
    s.fill((1, 2, 3, 255))
    pg.draw.rect(s, (9, 9, 9, 255), (5, 5, 6, 6))
    r = pg.draw.flood_fill(s, (200, 100, 50, 255), (0, 0))
    aus.append(("flood_fill", tuple(r), pg.image.tobytes(s, "RGBA").hex()))
    return aus


def zeigerformen(pg):
    """`pygame.cursors.Cursor` in allen drei Bauformen.

    `mouse.get_cursor()` gibt ein **Objekt** dieser Klasse zurueck, kein
    Tupel - ohne die Klasse liessen sich die drei Zeiger-Aufrufe gar nicht
    wahrheitsgemaess durchreichen.
    """
    C = pg.cursors.Cursor
    a, b = C(), C(11)
    d = C((16, 16), (7, 7), tuple(range(32)), tuple(range(32)))
    f = C((2, 2), pg.Surface((8, 8), pg.SRCALPHA))
    pg.mouse.set_cursor(d)
    return (repr(a), a.data, a.type, repr(b), b.data,
            repr(d), d.type, d.data[0], d.data[1], len(d.data[2]),
            repr(f), f.type, f.data[0],
            C(11) == C(11), C(11) == d, repr(C(d)), bool(C(0)),
            repr(pg.mouse.get_cursor()))


def anz_versuch(f):
    """Ruft auf und macht aus einer Ausnahme einen vergleichbaren Wert."""
    try:
        return f()
    except Exception as e:                          # noqa: BLE001
        return "%s: %s" % (type(e).__name__, e)


def anz_auskuenfte(pg):
    """Alles, was `display` ueber Bildschirm und Fenster sagt."""
    return (pg.display.get_allow_screensaver(),
            pg.display.set_allow_screensaver(True),
            pg.display.get_current_refresh_rate(),
            pg.display.get_desktop_refresh_rates(),
            pg.display.get_desktop_sizes(),
            pg.display.get_num_displays(),
            pg.display.get_window_position(),
            pg.display.set_window_position((10, 20)),
            pg.display.get_wm_info(),
            pg.display.iconify(),
            pg.display.is_fullscreen(),
            pg.display.is_vsync())


def anz_mode_ok(pg):
    """`mode_ok` in allen Faellen, die im Kern ausgemessen sind.

    Bemerkenswert: die Antwort ist **24**, obwohl `Info.bitsize` 32
    meldet. Das ist kein Widerspruch - 24 ist die Farbtiefe der
    Arbeitsflaeche (drei Kanaele), 32 die Breite eines Pixels im
    Speicher.
    """
    return (pg.display.mode_ok((640, 480)),
            pg.display.mode_ok((1024, 768)),
            pg.display.mode_ok((1025, 768)),
            pg.display.mode_ok((0, 0)),
            pg.display.mode_ok((-5, -5)),
            pg.display.mode_ok((640, 480), pg.FULLSCREEN),
            pg.display.mode_ok((1024, 768), pg.FULLSCREEN),
            pg.display.mode_ok((640, 480), 0, 8),
            pg.display.mode_ok((640, 480), 0, 32),
            pg.display.mode_ok((640, 480), 0, 4),
            pg.display.mode_ok((640, 480), 0, 33))


def anz_meldungen(pg):
    """`message_box` - und der Wortlaut bei falschem `message_type`."""
    return (anz_versuch(lambda: pg.display.message_box("Titel", "Text")),
            anz_versuch(lambda: pg.display.message_box("T", "M", "xyz")),
            anz_versuch(lambda: pg.display.set_palette([(1, 2, 3)])))


def anz_gamma(pg):
    """Gamma ist seit SDL2 ersatzlos entfallen - beide melden `False`."""
    import warnings
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        rampe = list(range(256))
        return (pg.display.set_gamma(1.0),
                pg.display.set_gamma(1.0, 1.0, 1.0),
                pg.display.set_gamma_ramp(rampe, rampe, rampe),
                anz_versuch(lambda: pg.display.set_gamma_ramp([1], [2], [3])))


def anz_vollbild(pg):
    """`set_mode` mit `FULLSCREEN` - die Flagge ist **negativ**.

    `pygame.FULLSCREEN` ist `0x80000000`, und weil das oberste Bit
    gesetzt ist, fuehrt pygame sie als `-2147483648`. Die Huelle nahm
    Flaggen lange als `u32` entgegen und wies den haeufigsten
    Vollbildaufruf ueberhaupt mit `OverflowError` ab. Aufgefallen ist es
    erst ueber `mode_ok`; `set_mode` selbst hatte niemand mit
    `FULLSCREEN` geprueft.
    """
    s = pg.display.set_mode((64, 48), pg.FULLSCREEN)
    a = (pg.FULLSCREEN, s.get_size(), pg.display.is_fullscreen())
    pg.display.set_mode((64, 48))
    return a


def zeigerdaten(pg):
    """Die sechs fertigen Zeiger und die fuenf Zeichenkettenformen.

    Uebernommen aus pygames eigener `cursors.py` - das sind rund 430
    Zeilen Hexzahlen, die niemand mit der Hand abschreiben sollte.
    `tools/gen_zeiger.py` schneidet sie heraus.
    """
    aus = []
    for name in ("arrow", "diamond", "ball", "broken_x", "tri_left", "tri_right"):
        z = getattr(pg.cursors, name)
        aus.append((name, z.type, z.data))
    for name in ("thickarrow_strings", "sizer_x_strings", "sizer_y_strings",
                 "sizer_xy_strings", "textmarker_strings"):
        aus.append((name, getattr(pg.cursors, name)))
    return aus


def zeiger_uebersetzen(pg):
    """`compile` samt der beiden Fehlerfaelle - Wortlaut gemessen."""
    aus = []
    for name in ("thickarrow_strings", "sizer_x_strings", "textmarker_strings"):
        aus.append(pg.cursors.compile(getattr(pg.cursors, name)))
    for schlecht in (("XXX", "XXX", "XXX"),
                     ("X" * 8, "X" * 7) + ("X" * 8,) * 6):
        try:
            pg.cursors.compile(schlecht)
            aus.append("kein Fehler")
        except Exception as e:                      # noqa: BLE001
            aus.append("%s: %s" % (type(e).__name__, e))
    return aus


def fassungen(pg):
    """`pygame.version` - und warum die Klassen Tupel sind.

    `vernum >= (2, 0, 0)` ist die uebliche Fassungsabfrage. Sie geht nur,
    weil `SoftwareVersion` von `tuple` erbt; ein eigener Typ mit neun
    nachgebauten Vergleichsmethoden waere neun Gelegenheiten, anders zu
    vergleichen. `rev` steht hier nicht: rustgame meldet dort seinen
    eigenen Namen statt der leeren Zeichenkette (siehe `fassung.rs`).
    """
    v = pg.version
    return (v.ver, str(v.vernum), repr(v.vernum), tuple(v.vernum),
            v.vernum == (2, 5, 7), v.vernum >= (2, 0, 0), len(v.vernum),
            (v.vernum.major, v.vernum.minor, v.vernum.patch),
            str(v.SDL), tuple(v.SDL), v.get_sdl_version(),
            v.get_sdl_version(linked=False),
            (pg.vernum == v.vernum, pg.ver == v.ver),
            (v.SoftwareVersion.__name__, v.PygameVersion.__name__,
             v.SDLVersion.__name__))


def hebel(pg):
    """`pygame.joystick` ohne angeschlossenes Geraet.

    Es gibt nie einen - und gerade deshalb laesst sich das Modul
    vollstaendig wahrheitsgetreu anbieten: was pygame in diesem Zustand
    tut, ist gemessen.
    """
    def v(f):
        try:
            return f()
        except Exception as e:                      # noqa: BLE001
            return "%s: %s" % (type(e).__name__, e)

    pg.init()
    nach_init = (pg.joystick.get_init(), v(pg.joystick.get_count))
    pg.joystick.quit()
    nach_quit = (pg.joystick.get_init(), v(pg.joystick.get_count))
    pg.joystick.init()
    wieder = (pg.joystick.get_init(), v(pg.joystick.get_count),
              v(lambda: pg.joystick.Joystick(0)))
    return (nach_init, nach_quit, wieder)


def ablage(pg):
    """`pygame.scrap` - samt der sechs Veraltet-Warnungen.

    Unter `SDL_VIDEODRIVER=dummy` liest pygame **nicht** die
    Zwischenablage des Systems: mit PowerShell abgelegter Text kommt
    nicht an. Ein prozesslokaler Speicher ist hier also kein Ersatz,
    sondern dasselbe Verhalten.
    """
    import warnings
    s = pg.scrap
    aus = []

    def v(name, f):
        with warnings.catch_warnings(record=True) as ws:
            warnings.simplefilter("always")
            try:
                r = f()
            except Exception as e:                  # noqa: BLE001
                r = "%s: %s" % (type(e).__name__, e)
        aus.append((name, r, [str(w.message) for w in ws]))

    v("get_init", s.get_init)
    v("init", s.init)
    v("put_text", lambda: s.put_text("hallo"))
    v("get_text", s.get_text)
    v("has_text", s.has_text)
    v("get_types", s.get_types)
    v("contains", lambda: s.contains("text/plain"))
    v("get", lambda: s.get("text/plain"))
    v("put", lambda: s.put("text/plain", b"x"))
    # `lost` und `set_mode` warnen **nicht** - gemessen, sieht nach einem
    # Versehen in pygame aus und wird trotzdem nachgebaut.
    v("lost", s.lost)
    v("set_mode", lambda: s.set_mode(pg.SCRAP_SELECTION))
    v("leer", lambda: (s.put_text(""), s.get_text(), s.has_text())[1:])
    return aus


def ton_welle(sekunden=0.25, rate=44100):
    """Ein kurzer WAV-Block als Bytes - ohne Datei auf der Platte."""
    import struct
    n = int(rate * sekunden)
    daten = b"".join(struct.pack("<h", 0) for _ in range(n))
    kopf = b"RIFF" + struct.pack("<I", 36 + len(daten)) + b"WAVE"
    kopf += b"fmt " + struct.pack("<IHHIIHH", 16, 1, 1, rate, rate * 2, 2, 16)
    kopf += b"data" + struct.pack("<I", len(daten))
    return kopf + daten


def ton_anlauf(pg):
    """`init`, `get_init`, `quit` und die Kanalzaehlung."""
    pg.mixer.quit()
    vorher = pg.mixer.get_init()
    pg.mixer.init()
    nachher = pg.mixer.get_init() is not None
    kanaele = pg.mixer.get_num_channels()
    pg.mixer.set_num_channels(16)
    return (vorher, nachher, kanaele, pg.mixer.get_num_channels(),
            pg.mixer.get_busy(), pg.mixer.get_driver())


def ton_klang(pg):
    """`Sound` aus einem Puffer - und was daran gemessen ist.

    `Sound(bytes)` **positional** ist ein roher Puffer, `Sound(BytesIO)`
    ein Dateiinhalt. Gemessen ergibt derselbe WAV-Block darum zwei
    verschiedene Laengen.
    """
    import io as _io
    pg.mixer.init()
    welle = ton_welle()
    s = pg.mixer.Sound(_io.BytesIO(welle))
    return (round(s.get_length(), 6), s.get_volume(),
            (s.set_volume(0.5), s.get_volume())[1],
            s.get_num_channels(),
            len(s.get_raw()),
            s == s, s.copy() == s)


def ton_spur(pg):
    """`Channel` - abspielen, halten, weitermachen, anhalten."""
    import io as _io
    pg.mixer.init()
    s = pg.mixer.Sound(_io.BytesIO(ton_welle()))
    c = pg.mixer.Channel(0)
    aus = [c.get_id() if hasattr(c, "get_id") else c.id, c.get_busy(),
           c.get_volume(), c.get_sound() is None]
    c.set_volume(0.25)
    aus.append(c.get_volume())
    c.set_endevent(pg.USEREVENT + 3)
    aus.append(c.get_endevent())
    c.stop()
    aus.append(c.get_busy())
    return tuple(aus)


def ton_musik(pg):
    """`pygame.mixer.music` - und die Verknuepfung selbst."""
    pg.mixer.init()
    return (pg.mixer.music is pg.mixer_music,
            pg.mixer.music.__name__.split(".")[-1],
            pg.mixer.music.get_busy(),
            pg.mixer.music.get_volume(),
            (pg.mixer.music.set_volume(0.5), pg.mixer.music.get_volume())[1],
            pg.mixer.music.get_endevent(),
            # `get_pos` nach `stop()` und erneutem `load()`.
            #
            # Das stand einmal als **benannte Grenze** im README, mit der
            # Begruendung, pygame liefere dort wegen eines
            # `Uint64`-Vergleichs eine sinnlose Zahl. Der Satz stand
            # laenger da als die Grenze - hier wird jetzt nachgemessen
            # statt behauptet. Faellt es auseinander, taucht es als
            # Abweichung auf und bekommt eine Begruendung mit Datum.
            musik_stelle(pg))


def musik_stelle(pg):
    """`music.get_pos()` in den drei Lagen, die es gibt.

    Ohne geladenes Stueck, nach `load()`, und nach `stop()` samt erneutem
    `load()`. Unter dem Treiber `dummy` spielt nichts, es geht also nur um
    die **Buchfuehrung** - und genau die ist die Stelle, an der ein
    Nachbau danebengreift.
    """
    def v(f):
        try:
            return f()
        except Exception as e:                      # noqa: BLE001
            return type(e).__name__

    aus = [v(pg.mixer.music.get_pos)]
    # Ohne Datei laesst sich nichts laden - dann bleibt es bei der ersten
    # Lage, und das ist auch eine Aussage.
    import os
    pfad = os.path.join(os.path.dirname(pg.__file__ or "."), "..", "nichts.ogg")
    aus.append(v(lambda: pg.mixer.music.load(pfad)))
    aus.append(v(pg.mixer.music.get_pos))
    aus.append(v(pg.mixer.music.stop))
    aus.append(v(pg.mixer.music.get_pos))
    return tuple(aus)


def ton_fehler(pg):
    """Was **vor** `init()` passiert - und was nicht.

    Gemessen: `music.get_busy()` und `music.get_volume()` werfen,
    `mixer.get_busy()` und `music.get_endevent()` nicht. Zwei
    gleichnamige Aufrufe, zwei Verhalten.

    **Sieben Aufrufe fehlen hier mit Absicht**: `s.play()`, `c.play(s)`,
    `c.queue(s)`, `c.get_sound()`, `c.get_queue()`, `c.get_endevent()`
    und `c.set_endevent(1)` lassen echtes pygame nach `quit()`
    **abstuerzen** - fehlendes `MIXER_INIT_CHECK()` in `mixer.c`. Ein
    Vergleich, der die Gegenseite umbringt, vergleicht nichts.
    """
    def v(f):
        try:
            return f()
        except Exception as e:                      # noqa: BLE001
            return "%s: %s" % (type(e).__name__, e)

    pg.mixer.quit()
    aus = (v(pg.mixer.get_busy),
           v(pg.mixer.music.get_busy),
           v(pg.mixer.music.get_volume),
           v(pg.mixer.music.get_endevent),
           v(lambda: pg.mixer.Sound(b"kein wav")),
           v(lambda: pg.mixer.Sound()),
           v(lambda: pg.mixer.Sound(None)),
           v(lambda: pg.mixer.Sound(foo=1)))
    pg.mixer.init()
    return aus


def _figur(pg, x, y, b=4, h=4, farbe=(200, 100, 50)):
    """Ein Sprite mit Bild und Rechteck - das Mindeste, was pygame will."""
    s = pg.sprite.Sprite()
    s.image = pg.Surface((b, h))
    s.image.fill(farbe)
    s.rect = s.image.get_rect(topleft=(x, y))
    return s


def figuren(pg):
    """`Sprite` allein: Gruppen betreten, verlassen, sterben.

    Die eine Zeile, an der alles haengt, ist `spr.rect.x += 1`: Python
    macht daraus `tmp = spr.rect; tmp.x = tmp.x + 1` und schreibt nichts
    zurueck. Das wirkt **nur**, wenn `spr.rect` jedes Mal dasselbe
    Objekt ist - deshalb steht sie hier.
    """
    a = _figur(pg, 1, 2)
    g1, g2 = pg.sprite.Group(), pg.sprite.Group()
    aus = [a.alive(), len(a.groups())]
    a.add(g1, g2)
    aus += [a.alive(), len(a.groups()), g1.has(a), g2.has(a)]
    a.remove(g2)
    aus += [len(a.groups()), g2.has(a)]
    a.rect.x += 1
    aus.append(tuple(a.rect))
    a.kill()
    aus += [a.alive(), len(a.groups()), g1.has(a)]
    return tuple(aus)


def gruppen(pg):
    """Die Gruppenarten und was sie unterscheidet."""
    a, b = _figur(pg, 0, 0), _figur(pg, 10, 10)
    g = pg.sprite.Group(a, b)
    aus = [len(g), g.has(a), g.has(a, b), len(g.sprites())]
    g.remove(a)
    aus += [len(g), g.has(a)]
    g.empty()
    aus.append(len(g))
    k = pg.sprite.Group(a, b).copy()
    aus += [type(k).__name__, len(k)]
    einzel = pg.sprite.GroupSingle(a)
    aus += [len(einzel), einzel.sprite is a]
    einzel.add(b)
    aus += [len(einzel), einzel.sprite is b]
    # Gemessen: es sind **echte** Unterklassen, `is` sagt nein.
    aus += [pg.sprite.OrderedUpdates is pg.sprite.RenderUpdates,
            pg.sprite.RenderPlain is pg.sprite.Group,
            issubclass(pg.sprite.OrderedUpdates, pg.sprite.RenderUpdates)]
    return tuple(aus)


def gruppen_zeichnen(pg):
    """`draw` - und die drei Bedeutungen seines Rueckgabewerts.

    Gemessen: `Group.draw()` gibt **dasselbe** Objekt zurueck, das
    danach `g.lostsprites` ist; `RenderUpdates.draw()` die **alte**
    Liste; `LayeredDirty.draw()` eine **Kopie**. Drei Varianten.
    """
    ziel = pg.Surface((30, 20))
    a, b = _figur(pg, 1, 1), _figur(pg, 10, 5, farbe=(10, 20, 30))
    g = pg.sprite.Group(a, b)
    r1 = g.draw(ziel)
    aus = [sorted(tuple(x) for x in r1), r1 is g.lostsprites,
           pg.image.tobytes(ziel, "RGBA").hex()]
    ziel2 = pg.Surface((30, 20))
    ru = pg.sprite.RenderUpdates(_figur(pg, 2, 2))
    r2 = ru.draw(ziel2)
    aus += [sorted(tuple(x) for x in r2),
            pg.image.tobytes(ziel2, "RGBA").hex()]
    g.clear(ziel, pg.Surface((30, 20)))
    aus.append(pg.image.tobytes(ziel, "RGBA").hex())
    return tuple(aus)


def stoesse(pg):
    """Die sechs Kollisionsaufrufe."""
    a = _figur(pg, 0, 0, 10, 10)
    b = _figur(pg, 5, 5, 10, 10)
    c = _figur(pg, 100, 100, 10, 10)
    g = pg.sprite.Group(b, c)
    aus = [pg.sprite.collide_rect(a, b), pg.sprite.collide_rect(a, c),
           len(pg.sprite.spritecollide(a, g, False)),
           pg.sprite.spritecollideany(a, g) is not None,
           len(pg.sprite.spritecollide(a, g, False,
                                       pg.sprite.collide_circle))]
    g2 = pg.sprite.Group(a)
    aus.append(len(pg.sprite.groupcollide(g2, g, False, False)))
    # `collide_rect_ratio` und `collide_circle_ratio` sind **Klassen**.
    aus += [pg.sprite.collide_rect_ratio(1.5)(a, b),
            pg.sprite.collide_circle_ratio(0.5)(a, b)]
    return tuple(aus)


def vererbung(pg):
    """Eine eigene Unterklasse mit eigenem Konstruktor.

    Das ist der Fall, den jedes zweite pygame-Spiel schreibt - und der
    in PyO3 nicht von selbst geht, weil es kein `__init__` kennt.
    """
    class MeineFigur(pg.sprite.Sprite):
        def __init__(self, x):
            super().__init__()
            self.image = pg.Surface((2, 2))
            self.rect = self.image.get_rect(topleft=(x, 0))
            self.zaehler = 0

        def update(self, *args):
            self.zaehler += 1

    class MeineGruppe(pg.sprite.Group):
        def __init__(self, *figuren):
            super().__init__(*figuren)
            self.name = "meine"

    f = MeineFigur(7)
    g = MeineGruppe(f)
    g.update()
    g.update()
    return (isinstance(f, pg.sprite.Sprite), tuple(f.rect), f.zaehler,
            len(g), g.name, type(g.copy()).__name__,
            issubclass(MeineGruppe, pg.sprite.Group))


def _feld(pg):
    s = pg.Surface((4, 3))
    for y in range(3):
        for x in range(4):
            s.set_at((x, y), (x * 20, y * 30, 40))
    return s


def feld_form(pg):
    """Form, Achsen und Schrittweiten.

    **`pa[x]` ist eine Spalte**, nicht eine Zeile: `len(pa)` ist die
    Breite, `pa[0]` hat die Hoehe als Laenge. Wer das verwechselt, baut
    eine Sicht, die bei quadratischen Flaechen genau richtig aussieht.
    """
    pa = pg.PixelArray(_feld(pg))
    return (len(pa), tuple(pa.shape), pa.ndim, pa.itemsize,
            tuple(pa.strides), len(pa[0]), tuple(pa[0].shape),
            pa[0].ndim, tuple(pa[0].strides),
            tuple(pa[0:2].shape), tuple(pa[0][0:2].shape),
            len(list(pa)), type(list(pa)[0]).__name__,
            tuple(pa.transpose().shape))


def feld_zugriff(pg):
    """Lesen, Setzen, Farbtupel, `surface is`, `close`."""
    s = _feld(pg)
    pa = pg.PixelArray(s)
    aus = [pa.surface is s, pa[0][0], pa[0, 0], pa[3][2], pa[-1][-1]]
    pa[0] = s.map_rgb((9, 9, 9))
    aus.append([s.get_at((0, y))[:3] for y in range(3)])
    pa[1:3] = s.map_rgb((1, 2, 3))
    aus.append((s.get_at((1, 0))[:3], s.get_at((2, 2))[:3]))
    pa[3][1] = s.map_rgb((7, 7, 7))
    aus.append(s.get_at((3, 1))[:3])
    # Gemessen: ein Farbtupel geht auch, nicht nur die Zahl.
    pa[0] = (5, 5, 5)
    aus.append(s.get_at((0, 0))[:3])
    aus.append(pa.make_surface().get_size())
    pa.close()
    try:
        pa[0][0]
        aus.append("kein Fehler")
    except Exception as e:                          # noqa: BLE001
        aus.append("%s: %s" % (type(e).__name__, e))
    try:
        pg.PixelArray(42)
        aus.append("kein Fehler")
    except Exception as e:                          # noqa: BLE001
        aus.append("%s: %s" % (type(e).__name__, e))
    return tuple(aus)


def feld_masken(pg):
    """`extract`, `compare`, `replace` - und ihre Farbrichtung.

    `extract` malt die **Treffer** weiss, `compare` die
    **Uebereinstimmungen**. Die zweite Richtung ist die ueberraschende:
    `compare` markiert nicht die Unterschiede.
    """
    s = _feld(pg)
    pa = pg.PixelArray(s)
    t = pa.extract(s.map_rgb((20, 0, 40))).make_surface()
    aus = [[[t.get_at((x, y))[:3] for x in range(4)] for y in range(3)]]
    b = _feld(pg)
    b.set_at((1, 1), (255, 255, 255))
    u = pa.compare(pg.PixelArray(b)).make_surface()
    aus.append([[u.get_at((x, y))[:3] for x in range(4)] for y in range(3)])
    r = pa.replace(s.map_rgb((20, 0, 40)), s.map_rgb((9, 9, 9)))
    aus += [r, s.get_at((1, 0))[:3]]
    return tuple(aus)


def rect_aus_ding(pg):
    """**Alles mit einem `rect` zaehlt als Rechteck** - gemessen.

    Das ist der Grund, warum `pygame.sprite` so bequem ist: ein Sprite
    **ist** fuer jeden Rechteckaufruf sein Rechteck. Und `rect` darf
    eine Methode sein, dann wird sie gerufen.
    """
    class Ding:
        def __init__(self, r):
            self.rect = r

    class Ding2:
        def rect(self):
            return pg.Rect(1, 1, 2, 2)

    d = Ding(pg.Rect(0, 0, 4, 4))
    return (pg.Rect(5, 5, 1, 1).collidelistall([d]),
            pg.Rect(0, 0, 2, 2).collidelistall([d]),
            pg.Rect(0, 0, 2, 2).colliderect(d),
            tuple(pg.Rect(0, 0, 2, 2).union(d)),
            tuple(pg.Rect(d)),
            pg.Rect(0, 0, 9, 9).colliderect(Ding2()))


def rect_mit_komma(pg):
    """`move` und `inflate` nehmen Fliesskomma und schneiden **zur Null**.

    `inflate(2.9, -2.9)` gibt `Rect(-1, 1, 12, 8)`: aus `-2.9` wird
    `-2`, nicht `-3`.
    """
    r = pg.Rect(0, 0, 10, 10)
    q = pg.Rect(0, 0, 10, 10)
    r.move_ip(2.9, -2.9)
    q.inflate_ip(2.9, -2.9)
    return (tuple(pg.Rect(0, 0, 10, 10).inflate(2.9, -2.9)),
            tuple(pg.Rect(0, 0, 10, 10).inflate(-2.9, 2.9)),
            tuple(pg.Rect(0, 0, 10, 10).move(2.9, -2.9)),
            tuple(pg.Rect(0, 0, 10, 10).move(2, -2)),
            tuple(r), tuple(q))


def ton_endereignis(pg):
    """Kommt `Channel.set_endevent` wirklich in `event.get()` an?

    Die Naht dahinter ist der Teil, den ein Nachbau leicht vergisst: in
    pygame laeuft SDLs Tonfaden nebenher und legt das Ereignis von selbst
    ab. `rustgame-mixer` hat keinen eigenen Faden - dort vergeht die Zeit
    nur, wenn jemand die Kiste anfasst. Ohne einen Anstoss aus dem
    Ereignis-Durchlauf blieb die Schlange leer, obwohl `set_endevent`
    und `get_endevent` sauber zusammenpassten: gesetzt, gelesen, nie
    zugestellt.

    Die Probe wartet hoechstens eine Sekunde und meldet, was sie
    gefunden hat - beide Engines bekommen dieselbe Frist.
    """
    import io as _io
    import struct
    import time

    rate, sek = 44100, 0.02
    roh = b"".join(struct.pack("<h", 0) for _ in range(int(rate * sek)))
    wav = (b"RIFF" + struct.pack("<I", 36 + len(roh)) + b"WAVE"
           + b"fmt " + struct.pack("<IHHIIHH", 16, 1, 1, rate, rate * 2, 2, 16)
           + b"data" + struct.pack("<I", len(roh)) + roh)

    pg.display.set_mode((8, 8))
    pg.mixer.init()
    pg.event.clear()
    klang = pg.mixer.Sound(_io.BytesIO(wav))
    spur = pg.mixer.Channel(0)
    typ = pg.USEREVENT + 7
    spur.set_endevent(typ)
    spur.play(klang)
    ende = time.time() + 1.0
    while time.time() < ende:
        for e in pg.event.get():
            if e.type == typ:
                return (e.type, getattr(e, "code", None))
        pg.time.wait(5)
    return "kein Endereignis binnen einer Sekunde"


def rect_klemmen(pg):
    """`clamp` mit ungeraden, negativen und Null-Groessen.

    Der Kern rechnete `other.x + (other.w - self.w) / 2` statt
    `other.x + other.w / 2 - self.w / 2`. Algebraisch dasselbe, in ganzen
    Zahlen nicht - **1134 von 336.000** gewuerfelten Proben lagen um eins
    daneben. Keine der hiesigen Pruefungen sah es.
    """
    aus = []
    for a in ((0, 0, 3, 3), (1, 1, 0, 0), (-5, -5, 7, 9), (2, 2, 100, 100)):
        for b in ((0, 0, 10, 10), (0, 0, 4, 4), (-3, -3, 5, 7), (0, 0, 0, 0)):
            aus.append(tuple(pg.Rect(a).clamp(pg.Rect(b))))
            aus.append(tuple(pg.FRect(a).clamp(pg.FRect(b))))
    return aus


def rect_einpassen(pg):
    """`fit`, auch mit Groesse Null - dort wird pygame entartet.

    Gemessen: `Rect(0,0,0,0).fit(Rect(0,0,10,10))` gibt
    `Rect(-1073741819, -1073741819, -2147483648, -2147483648)`. Das ist
    Cs unbestimmte Gleitkomma-Umwandlung, kein sinnvoller Wert - und
    genau deshalb eine gute Probe: eine Nachbildung, die hier
    "vernuenftig" klemmt, weicht ab.
    """
    aus = []
    for a in ((0, 0, 3, 5), (0, 0, 0, 0), (0, 0, 10, 1)):
        for b in ((0, 0, 10, 10), (2, 2, 7, 3), (0, 0, 0, 0)):
            aus.append(tuple(pg.Rect(a).fit(pg.Rect(b))))
    return aus


def rect_woerterbuch(pg):
    """`collidedict` und `collidedictall`, mit und ohne `values`."""
    r = pg.Rect(1, 1, 10, 10)
    d = {(0, 0, 4, 4): "a", (100, 100, 4, 4): "b", (5, 5, 9, 9): "c"}
    dv = {"a": (0, 0, 4, 4), "b": (100, 100, 4, 4)}
    return (sorted(str(x) for x in (r.collidedict(d) or ())),
            sorted(str(k) for k, _ in r.collidedictall(d)),
            sorted(str(x) for x in (r.collidedict(dv, 1) or ())),
            sorted(str(k) for k, _ in r.collidedictall(dv, 1)))


def rect_scheiben(pg):
    """Ellipse, Scheiben und Loeschen am Rechteck."""
    def v(f):
        try:
            return f()
        except Exception as e:                      # noqa: BLE001
            return "%s: %s" % (type(e).__name__, e)

    r = pg.Rect(1, 2, 3, 4)
    aus = [list(r[1:3]), v(lambda: r.__delitem__(0))]
    q = pg.Rect(1, 2, 3, 4)
    q[...] = pg.Rect(9, 9, 9, 9)
    aus.append(tuple(q))
    t = pg.Rect(1, 2, 3, 4)
    t[1:3] = (7, 8)
    aus.append(tuple(t))
    return tuple(aus)


def rect_unterklasse(pg):
    """Eine Unterklasse - und was `copy`/`move`/`clip` daraus machen.

    Gemessen: pygame baut Ergebnisse mit `Py_TYPE(self)->tp_new` und
    **ueberspringt `tp_init`**. Der Typ bleibt also erhalten, das eigene
    Feld aber nicht.
    """
    class MeinRect(pg.Rect):
        def __init__(self, *a, **k):
            super().__init__(*a, **k)
            self.markierung = True

    r = MeinRect(1, 2, 3, 4)
    ergebnisse = (r.copy(), r.move(1, 1), r.clip(pg.Rect(0, 0, 2, 2)),
                  r.inflate(2, 2), r.union(pg.Rect(0, 0, 1, 1)))
    return (tuple(r), r.markierung,
            [type(x).__name__ for x in ergebnisse],
            [hasattr(x, "markierung") for x in ergebnisse])


def farbe_rechnen(pg):
    """`Color` rechnet **nur** mit `Color` - gemessen.

    `Color(1,2,3,4) + (1,1,1,1)` ist ein TypeError, `Color * 2` auch.
    Und `==` nimmt ein Tupel, aber **keine Liste** - bei Vektoren ist es
    umgekehrt.
    """
    def v(f):
        try:
            return f()
        except Exception as e:                      # noqa: BLE001
            return type(e).__name__

    c = pg.Color(1, 2, 3, 4)
    return (v(lambda: tuple(c + (1, 1, 1, 1))),
            v(lambda: tuple(c * 2)),
            tuple(c + pg.Color(1, 1, 1, 1)),
            c == (1, 2, 3, 4), c == [1, 2, 3, 4],
            pg.math.Vector2(1, 0) == (1, 0),
            pg.math.Vector2(1, 0) == [1, 0],
            v(lambda: c[-1]), tuple(c[-2:]))


def farbe_namen(pg):
    """Die Farbnamen - 665 Stueck, und pygames dreistufige Auflösung."""
    return (len(pg.color.THECOLORS),
            pg.color.THECOLORS is pg.colordict.THECOLORS,
            pg.color.THECOLORS["aquamarine3"],
            tuple(pg.Color("aquamarine3")),
            tuple(pg.Color("Alice Blue")),
            tuple(pg.Color("0x1a2B3c4D")),
            tuple(pg.Color("#1a2b3c")))


def farbe_setzer(pg):
    """Die Setzer und ihr Umgang mit dem vierten Kanal.

    Gemessen und ueberraschend: `c.hsva = (h, s, v)` setzt das Alpha auf
    **0**, nicht auf 100 und nicht auf den alten Wert. Bei `cmy` und
    `i1i2i3` bleibt es stehen, bei `normalized` wird es 255.
    """
    aus = []
    c = pg.Color(1, 2, 3, 4)
    c.hsva = (0, 100, 100)
    aus.append(tuple(c))
    c = pg.Color(1, 2, 3, 4)
    c.cmy = (0.5, 0.5, 0.5)
    aus.append(tuple(c))
    c = pg.Color(1, 2, 3, 4)
    c.normalized = (0.5, 0.5, 0.5, 0.5)
    aus.append(tuple(c))
    # `from_normalized` **rundet** - die einzige Stelle in `color.c`, die
    # das tut. Abgeschnitten waere 127.
    aus.append(tuple(pg.Color.from_normalized(0.5, 0.5, 0.5, 1.0)))
    return tuple(aus)


def math_frei(pg):
    """Die freien Funktionen von `pygame.math`.

    `lerp` begrenzt `t`, `remap` **nicht** - der Kern hatte das Begrenzen
    eine Schicht zu tief, und damit war `remap` falsch.
    """
    def v(f):
        try:
            return f()
        except Exception as e:                      # noqa: BLE001
            return type(e).__name__

    return (pg.math.lerp(0, 10, 1.5),
            pg.math.lerp(0, 10, 1.5, False),
            pg.math.remap(0, 10, 0, 100, -1),
            pg.math.invlerp(0, 10, 15),
            v(lambda: pg.math.invlerp(5, 5, 1)),
            pg.math.clamp(12 ** 300, 1, 2),
            pg.math.smoothstep(0, 10, 0.5))


def vektor_gleich(pg):
    """`==` bei Vektoren vergleicht mit `epsilon`, nicht bitgenau.

    Das allein erklaerte 20 von 29 Fehlschlaegen in `math_test.py`: die
    Rechnungen waren alle richtig, nur der Vergleich war zu streng.
    """
    import math as _m
    v = pg.math.Vector2(1, 0).rotate_rad(_m.pi)
    return (tuple(v), v == (-1.0, 0.0), v == pg.math.Vector2(-1, 0),
            pg.math.Vector2(1, 2).epsilon,
            pg.math.Vector3(1, 0, 0).rotate_x_rad(_m.pi) == (1, 0, 0))


def typen_bleiben(pg):
    """Unterklassen behalten ihren Typ - bei Vektor, Farbe und Maske."""
    class V(pg.math.Vector2):
        pass

    class C(pg.Color):
        pass

    class M(pg.mask.Mask):
        def __init__(self, *a, **k):
            super().__init__(*a, **k)
            self.markierung = True

    v, c, m = V(1, 2), C(1, 2, 3), M((4, 4))
    return (type(v + v).__name__, type(v.rotate(90)).__name__,
            type(v.elementwise() * 2).__name__,
            type(c.correct_gamma(1.0)).__name__,
            type(m.copy()).__name__, hasattr(m.copy(), "markierung"))


def init_setzt_neu(pg):
    """`__init__` **setzt den Wert neu** - nicht nur beim Anlegen.

    PyO3 kennt nur `__new__`; pygames Typen haben zusaetzlich ein echtes
    `tp_init`. Sichtbar wird der Unterschied bei jeder Unterklasse mit
    `super().__init__(...)`.
    """
    r = pg.Rect(1, 2, 3, 4)
    r.__init__(9, 9, 9, 9)
    c = pg.Color(1, 2, 3)
    c.__init__(9, 9, 9)
    v = pg.math.Vector2(1, 2)
    v.__init__(9, 9)
    m = pg.mask.Mask((5, 3))
    m.__init__((9, 9))
    s = pg.Surface((4, 4))
    s.__init__((9, 9))
    return (tuple(r), tuple(c), tuple(v), m.get_size(), s.get_size())


def format_je_tiefe(pg):
    """Alle sechs Tiefen, und jede Auskunft dazu.

    `shifts`, `losses` und `pitch` sind aus den Masken **ableitbar** und
    werden im Kern deshalb nicht abgelegt. Genau darum gehoeren sie hier
    hin: eine falsche Ableitung faellt sonst nirgends auf, weil bei 24
    und 32 Bit alle Verluste null sind und die naheliegende Rechnung
    stimmt.
    """
    aus = []
    for d in (8, 12, 15, 16, 24, 32):
        s = pg.Surface((4, 4), depth=d)
        aus.append((d, s.get_bitsize(), s.get_bytesize(), s.get_masks(),
                    s.get_shifts(), s.get_losses(), s.get_pitch()))
    return tuple(aus)


def format_mit_alpha(pg):
    """`SRCALPHA` gibt es nur bei 16 und 32 Bit.

    Bei 8, 12, 15 und 24 meldet pygame
    `ValueError: no standard masks exist for given bitdepth with alpha`,
    bei einer unbekannten Tiefe `nonstandard bit depth given`. Zwei
    verschiedene Saetze fuer zwei verschiedene Faelle.
    """
    def v(f):
        try:
            return f()
        except Exception as e:                      # noqa: BLE001
            return "%s: %s" % (type(e).__name__, e)

    aus = []
    for d in (8, 12, 15, 16, 24, 32, 7, 64):
        aus.append((d, v(lambda d=d: pg.Surface((4, 4), pg.SRCALPHA, d).get_masks())))
    return tuple(aus)


def format_eigene_masken(pg):
    """Eigene Masken - und was daraus an Verschiebungen wird."""
    def v(f):
        try:
            return f()
        except Exception as e:                      # noqa: BLE001
            return "%s: %s" % (type(e).__name__, e)

    s = pg.Surface((4, 4), 0, 32, (0xFF000000, 0xFF0000, 0xFF00, 0xFF))
    return (s.get_shifts(), s.get_masks(), s.get_losses(),
            v(lambda: pg.Surface((4, 4), 0, 32, (1, 2, 3)).get_masks()))


def tafel_acht_bit(pg):
    """Die Vorgabe-Farbtafel, **alle** 256 Eintraege.

    pygames Tafel hat nur **128** verschiedene Farben; Eintrag 128 faengt
    wieder bei Schwarz an. Der erste Nachbau hier war eine R3G3B2-Tafel
    und in 216 von 256 Eintraegen falsch - eine Stichprobe auf die ersten
    acht haette das nicht gezeigt, denn dort stimmen beide.
    """
    s = pg.Surface((4, 4), depth=8)
    t = s.get_palette()
    return (len(t), len(set(tuple(c) for c in t)),
            tuple(tuple(c) for c in t))


def abbilden_sechzehn(pg):
    """`map_rgb`/`unmap_rgb` bei 16 Bit - dort gehen Bits verloren."""
    s = pg.Surface((4, 4), depth=16)
    aus = []
    for c in ((255, 0, 0), (74, 76, 74), (1, 2, 3), (255, 255, 255)):
        v = s.map_rgb(c)
        aus.append((c, v, tuple(s.unmap_rgb(v))))
    return tuple(aus)


def mittelfarbe_sechzehn(pg):
    """`average_color` sieht **nicht** dasselbe wie `get_at`.

    pygame holt die Kanaele dort von Hand als
    `((pixel & mask) >> shift) << loss` - **ohne** SDLs Auffuellung
    `(v*255)/((1<<(8-loss))-1)`. Bei 16 Bit macht das bis zu sieben Stufen
    aus: dieselbe Flaeche liest `get_at` als `(74,76,74)` und
    `average_color` als `(72,76,72)`. Bei 24 und 32 Bit ist `loss` null
    und beide Wege fallen zusammen - deshalb faellt es sonst nie auf.
    """
    aus = []
    for d in (16, 24, 32):
        s = pg.Surface((4, 4), depth=d)
        s.fill((74, 76, 74))
        aus.append((d, tuple(s.get_at((0, 0))),
                    tuple(pg.transform.average_color(s))))
    return tuple(aus)


def fehlerarten_blit(pg):
    """`blits` und `fblits` - und ihre vier verschiedenen Einwaende."""
    def v(f):
        try:
            return f()
        except Exception as e:                      # noqa: BLE001
            return "%s: %s" % (type(e).__name__, e)

    s = pg.Surface((8, 8))
    q = pg.Surface((2, 2))
    return (v(lambda: s.fblits()),
            v(lambda: s.blits(None)),
            v(lambda: s.blits([(q, (0, 0))], doreturn=0)),
            v(lambda: s.blits([(q, (0, 0))], 0)),
            v(lambda: s.fblits([(q, (0, 0))], special_flags=0)),
            v(lambda: s.blits([(q,)])),
            v(lambda: s.get_locks()))


def fehlerarten_transform(pg):
    """Sieben Stellen, an denen pygame `TypeError` nimmt, nicht `ValueError`.

    Und **ausdrueckliches `None`** ist dabei nicht dasselbe wie
    Weglassen: `invert(s)` gibt eine Flaeche, `invert(s, None)` gibt
    `TypeError`. PyO3 legt die beiden Faelle von sich aus zusammen.
    """
    def v(f):
        try:
            return f()
        except Exception as e:                      # noqa: BLE001
            return "%s: %s" % (type(e).__name__, e)

    s = pg.Surface((4, 4))
    c = pg.Color(1, 2, 3)
    return (v(lambda: pg.transform.invert(s, None) and None),
            v(lambda: pg.transform.threshold(None, s, c)),
            v(lambda: pg.transform.threshold(None, s, c, "x")),
            v(lambda: pg.transform.average_surfaces([])),
            v(lambda: pg.transform.grayscale(s, pg.Surface((4, 4), 0, 24))),
            v(lambda: pg.transform.scale(s, (2, 2), pg.Surface((4, 4), 0, 24))),
            v(lambda: pg.transform.hsl(s, 0, 0, 0, pg.Surface((4, 4), 0, 24))))


def unbekanntes_wort(pg):
    """Der Satz fuer ein Schluesselwort, das die Funktion nicht kennt.

    Steht als **eigene** Probe, weil er hin und wieder huellenweit
    abweicht (siehe `BEKANNTE_GRENZEN`) und in einer Sammelprobe sonst
    sieben richtige Messungen mit sich in die Tiefe zieht.
    """
    def v(f):
        try:
            return f()
        except Exception as e:                      # noqa: BLE001
            return "%s: %s" % (type(e).__name__, e)

    s = pg.Surface((4, 4))
    return (v(lambda: pg.transform.scale_by(s, 2, y=3)),
            v(lambda: pg.transform.flip(s, True, True, z=1)),
            v(lambda: pg.draw.rect(s, (1, 2, 3), pg.Rect(0, 0, 1, 1), foo=1)),
            v(lambda: s.get_rect(mitte=(1, 1))))


def fehlerarten_farbe(pg):
    """Eine kaputte Farbe und ein kaputter Punkt - Sorte **und** Wortlaut.

    Und eine blanke Zahl ist kein RGBA-Wort, sondern der **abgebildete
    Pixelwert der Zielflaeche**: `draw.rect(s, 5, r)` faerbt auf einer
    ARGB-Flaeche `(0, 0, 5, 0)`, nicht `(5, 5, 5)`.
    """
    def v(f):
        try:
            return f()
        except Exception as e:                      # noqa: BLE001
            return "%s: %s" % (type(e).__name__, e)

    s = pg.Surface((8, 8), pg.SRCALPHA)
    r = pg.Rect(0, 0, 4, 4)
    pg.draw.rect(s, 5, r)
    zahl = tuple(s.get_at((0, 0)))
    return (zahl,
            v(lambda: pg.draw.rect(s, (1, 2), r)),
            v(lambda: pg.draw.rect(s, "keinefarbe", r)),
            v(lambda: pg.draw.rect(s, None, r)),
            v(lambda: pg.draw.line(s, (1, 2, 3), (0,), (1, 1))),
            v(lambda: pg.draw.line(s, (1, 2, 3), "xy", (1, 1))),
            v(lambda: pg.draw.circle(s, (1, 2, 3), (0, 0), -1)),
            v(lambda: pg.draw.polygon(s, (1, 2, 3), [(0, 0)])))


def maske_ausschnitt(pg):
    """`to_surface(area=...)` - **normalisiert**, dann geschnitten.

    Die Reihenfolge ist der ganze Punkt. Ein negatives Rechteck bezeichnet
    in pygame denselben Bereich von der anderen Seite; wer die Seite
    stattdessen auf Null klemmt, bekommt eine leere Flaeche. Und was
    ausserhalb der Maske liegt, wird **nicht gezeichnet** - auch nicht mit
    `unsetcolor`.
    """
    aus = []
    m = pg.mask.Mask((7, 7), fill=True)
    for a in ((-3, 0, 7, 7), (3, 0, 7, 7), (0, 0, -2, -2), (10, 10, 3, 3),
              (0, 1, 1, -1), (1, 0, -1, 1), (1, 1, -1, -1)):
        f = m.to_surface(area=pg.Rect(a))
        aus.append((a, f.get_size(),
                    tuple(f.get_at((0, 0))) if f.get_size()[0] else None))
    # Und dass ein Ueberstand das Ziel in Ruhe laesst.
    z = pg.Surface((3, 5))
    z.fill((255, 0, 0))
    k = pg.mask.Mask((3, 5))
    k.set_at((0, 0))
    z2 = k.to_surface(z, area=pg.Rect((0, 1), (1, -1)))
    aus.append((tuple(z2.get_at((0, 0))), tuple(z2.get_at((2, 4)))))
    return tuple(aus)


def maske_kopie(pg):
    """`copy()` ruft das `__copy__` der **Instanz**, nicht umgekehrt."""
    import copy as _c

    spuren = []

    class M(pg.mask.Mask):
        def __copy__(self):
            spuren.append("dunder")
            k = super().__copy__()
            k.marke = True
            return k

    m = M((4, 3), fill=True)
    a, b = m.copy(), _c.copy(m)
    return (type(a).__name__, type(b).__name__,
            getattr(a, "marke", None), getattr(b, "marke", None),
            len(spuren), a.count(), b.count())


def maske_format(pg):
    """Die Vorlagen brauchen **genau** dasselbe Format wie das Ziel.

    Nicht nur dieselbe Tiefe: `Surface(g, 0, 32)` und
    `Surface(g, SRCALPHA, 32)` haben beide 32 Bit und unterscheiden sich
    in der Alphamaske. Eine Pruefung auf `bitsize` allein laesst die
    Haelfte der Faelle durch.
    """
    def v(f):
        try:
            return f()
        except Exception as e:                      # noqa: BLE001
            return "%s: %s" % (type(e).__name__, e)

    g = (5, 4)
    m = pg.mask.Mask(g)
    aus = []
    for zt, st in ((32, 16), (32, 24), (16, 32), (8, 16)):
        aus.append(v(lambda zt=zt, st=st: m.to_surface(
            pg.Surface(g, depth=zt), pg.Surface(g, depth=st)) and None))
    for zf, sf in ((pg.SRCALPHA, 0), (0, pg.SRCALPHA)):
        aus.append(v(lambda zf=zf, sf=sf: m.to_surface(
            pg.Surface(g, zf, 32), pg.Surface(g, sf, 32)) and None))
    return tuple(aus)


def maske_schwelle(pg):
    """Die Schwelle ist **vorzeichenbehaftet und ungeklemmt**.

    `from_surface(s, -1)` setzt jeden Pixel, weil auch `alpha == 0` noch
    groesser ist als `-1`. Auf 0 geklemmt kommt keiner heraus. Die
    Gegenprobe nach oben (`256`) verdeckt den Fehler, weil `255 > 255` und
    `255 > 256` beide falsch sind - nur die Richtung nach unten trennt die
    Faelle.
    """
    aus = []
    for d in (16, 32):
        s = pg.Surface((11, 9), pg.SRCALPHA, d)
        for alpha in (0, 1, 127, 128, 255):
            s.fill((55, 155, 255, alpha))
            echt = s.get_at((0, 0))[3]
            for t in (-1, 0, 255, 256, echt - 1, echt, echt + 1):
                aus.append((d, echt, t, pg.mask.from_surface(s, t).count()))
    return tuple(aus)


def maus_stellen(pg):
    """`get_pressed` nimmt **nur** 3 oder 5, `get_pos` nimmt `desktop`."""
    def v(f):
        try:
            return f()
        except Exception as e:                      # noqa: BLE001
            return "%s: %s" % (type(e).__name__, e)

    return (pg.mouse.get_pos(), pg.mouse.get_pos(True),
            pg.mouse.get_pos(desktop=True),
            len(pg.mouse.get_pressed()), len(pg.mouse.get_pressed(5)),
            len(pg.mouse.get_pressed(num_buttons=5)),
            len(pg.mouse.get_pressed(3, True)),
            v(lambda: pg.mouse.get_pressed(4)),
            v(lambda: pg.mouse.get_pressed(0)),
            v(lambda: pg.mouse.get_pressed(-1)),
            tuple(v(lambda p=p: pg.mouse.set_pos(p))
                  for p in ((1,), [1, 2, 3], 1, "1", (1, "1"), [])))


def schlange_unter_last(pg):
    """Die Schlange mit tausend Ereignissen - Reihenfolge und Verlust.

    Bis Runde 17 hatte jede Probe drei Ereignisse eingestellt und eines
    wieder geholt. Gefuellt gefragt kamen vier Funde heraus, und drei
    davon betreffen die **Reihenfolge**:

    * `get(liste)` gibt **Typ fuer Typ** heraus, in der Reihenfolge der
      Liste - nicht in Schlangenfolge. `get([C, A])` gibt erst alle `C`.
    * `get(exclude=liste)` laesst das Ausgenommene genauso gruppiert
      zurueck.
    * `set_blocked` **raeumt die Schlange aus** (SDLs `SDL_EventState`
      ruft `SDL_FlushEvent` mit), `set_allowed` nicht.

    `custom_type` steht **nicht** hier: sein Vorrat ist je Prozess
    einmal da, und wer ihn aufbraucht, nimmt der Probe `event.custom_type`
    daneben ihre Antwort. Der Satz am Anschlag
    (`pygame.event.custom_type made too many event types.`) ist in
    `rustgame-display/src/event.rs` vermerkt.
    """
    A, B, C, D = pg.USEREVENT, pg.KEYDOWN, pg.QUIT, pg.MOUSEMOTION
    kurz = {A: "A", B: "B", C: "C", D: "D"}

    def frisch():
        pg.event.set_allowed(None)
        pg.event.clear()

    def fuellen(mal=3):
        frisch()
        for i, t in enumerate([A, B, C, D] * mal):
            pg.event.post(pg.event.Event(t, zahl=i))

    def folge(liste):
        return "".join("%s%d" % (kurz.get(e.type, "?"), getattr(e, "zahl", -1))
                       for e in liste)

    aus = []
    # 1. Wie viele passen hinein, und bleibt die Reihenfolge?
    for n in (1, 127, 128, 129, 1000, 5000):
        frisch()
        gut = sum(1 for i in range(n)
                  if pg.event.post(pg.event.Event(A, zahl=i)))
        hol = pg.event.get()
        zahlen = [e.zahl for e in hol]
        aus.append(("fasst", n, gut, len(hol),
                    zahlen == list(range(len(zahlen)))))
    # 2. Reihenfolge bei der Auswahl - die eigentliche Frage.
    for marke, ruf in (
        ("get()", lambda: pg.event.get()),
        ("get(A)", lambda: pg.event.get(A)),
        ("get([A,C])", lambda: pg.event.get([A, C])),
        ("get([C,A])", lambda: pg.event.get([C, A])),
        ("get([D,B,A])", lambda: pg.event.get([D, B, A])),
        ("get([A,A,C])", lambda: pg.event.get([A, A, C])),
        ("get((C,A))", lambda: pg.event.get((C, A))),
        ("get(exclude=A)", lambda: pg.event.get(exclude=A)),
        ("get(exclude=[A,B])", lambda: pg.event.get(exclude=[A, B])),
        ("get(exclude=[B,A])", lambda: pg.event.get(exclude=[B, A])),
        ("get(A,exclude=C)", lambda: pg.event.get(A, exclude=C)),
        ("clear([A,C])", lambda: pg.event.clear([A, C])),
        ("clear(A)", lambda: pg.event.clear(A)),
        ("peek([A,C])", lambda: pg.event.peek([A, C])),
        ("peek([])", lambda: pg.event.peek([])),
        ("peek(D)", lambda: pg.event.peek(D)),
    ):
        fuellen()
        try:
            w = ruf()
            w = folge(w) if isinstance(w, list) else repr(w)
            aus.append((marke, w, folge(pg.event.get())))
        except Exception as e:                      # noqa: BLE001
            aus.append((marke, "%s: %s" % (type(e).__name__,
                                           _satz_gestalt(str(e), pg.__name__))))
    # 3. Sperren und Freigeben - was raeumt aus?
    for marke, ruf in (
        ("set_blocked(B)", lambda: pg.event.set_blocked(B)),
        ("set_blocked([B,D])", lambda: pg.event.set_blocked([B, D])),
        ("set_blocked(None)", lambda: pg.event.set_blocked(None)),
        ("set_blocked([])", lambda: pg.event.set_blocked([])),
        ("set_allowed(B)", lambda: pg.event.set_allowed(B)),
        ("set_allowed(None)", lambda: pg.event.set_allowed(None)),
        ("set_allowed([])", lambda: pg.event.set_allowed([])),
        ("blocked dann allowed",
         lambda: (pg.event.set_blocked(B), pg.event.set_allowed(B))),
    ):
        fuellen()
        ruf()
        aus.append((marke, folge(pg.event.get()),
                    [kurz[t] for t in (A, B, C, D) if pg.event.get_blocked(t)]))
    frisch()
    # 4. Ein gesperrter Typ kommt nicht hinein.
    pg.event.set_blocked(B)
    aus.append(("post gesperrt", pg.event.post(pg.event.Event(B, zahl=1)),
                len(pg.event.get())))
    frisch()
    # 5. poll in der Reihe, und die Nutzlast unter Last.
    for i in range(300):
        pg.event.post(pg.event.Event(A, zahl=i, text="x" * 50,
                                     liste=list(range(20))))
    gezogen = []
    while True:
        e = pg.event.poll()
        if e.type == pg.NOEVENT:
            break
        gezogen.append((e.zahl, len(e.text), len(e.liste)))
    aus.append(("poll 300", len(gezogen),
                gezogen == [(i, 50, 20) for i in range(len(gezogen))]))
    frisch()
    # 32 sind es wirklich (6 + 16 + 8 + 1 + 1); die Grenze liegt darunter.
    if len(aus) < 30:
        raise RuntimeError("Ereignisschlange: nur %d Antworten" % len(aus))
    return tuple(aus)


def schlange_typen(pg):
    """Was gilt als Ereignistyp? Drei Fehlerarten, und sie haengen zusammen.

    Gemessen Wert fuer Wert - die Regel ist nicht herzuleiten, weil sie
    an C-Typen haengt:

    ```text
    0 .. 65534      gut            True        gut (ein bool ist eine Zahl)
    65535           ValueError     2**31       TypeError (passt nicht in int)
    -1              ValueError     1.5         TypeError (eigener Satz)
    [1.9]           gut, wird 1    {0: 1}      TypeError (keine Folge)
    ```

    Dieselbe Frage an `set_blocked`, `set_allowed`, `get_blocked`,
    `get`, `clear` und `peek`, weil jede ihren eigenen Weg hineinnimmt -
    `get_blocked(None)` ist die Ausnahme, die das belegt: ueberall heisst
    `None` "alle Typen", dort ist es ein Fehler.
    """
    werte = [0, 1, 32767, 65534, 65535, 65536, 70000, 2 ** 31 - 1,
             2 ** 31, 2 ** 63, -1, -(2 ** 31), 1.5, "x", b"0", None, True,
             [1.9], [-0.5], [True], ["5"], [None], [[0]], [0, 2 ** 31],
             (True, 1), [65535], "ab", range(3), {0: 1}, {0, 1}, [], ()]
    wege = [
        ("set_blocked", lambda w: pg.event.set_blocked(w)),
        ("set_allowed", lambda w: pg.event.set_allowed(w)),
        ("get_blocked", lambda w: pg.event.get_blocked(w)),
        ("get", lambda w: len(pg.event.get(w))),
        ("clear", lambda w: pg.event.clear(w)),
        ("peek", lambda w: pg.event.peek(w)),
    ]
    aus = []
    for wert in werte:
        for name, ruf in wege:
            pg.event.set_allowed(None)
            pg.event.clear()
            try:
                antwort = repr(ruf(wert))
            except Exception as e:                  # noqa: BLE001
                antwort = "%s: %s" % (type(e).__name__,
                                      _satz_gestalt(str(e), pg.__name__))
            # Was sperrt danach wirklich? Daran faellt auf, wenn ein
            # angenommener Wert den **falschen** Typ getroffen hat.
            gesperrt = [t for t in (0, 1, 2, 256, 768, 1024, 32865)
                        if pg.event.get_blocked(t)]
            aus.append((repr(wert)[:14], name, antwort, gesperrt))
    pg.event.set_allowed(None)
    if len(aus) < 180:
        raise RuntimeError("Ereignistypen: nur %d Antworten" % len(aus))
    return tuple(aus)


def schlange_halb(pg):
    """Was eine **kaputte** Typliste mitnimmt, bevor sie scheitert.

    Steht als eigene Probe, weil **ein** Weg davon eine benannte Grenze
    ist (siehe `BEKANNTE_GRENZEN`) - in der Sammelprobe daneben zoege er
    dreissig richtige Messungen mit sich.

    Gemessen an der Schlange `A B C A B C A B C`:

    ```text
    get([A, kaputt])          TypeError - bei pygame sind die A **weg**
    get(exclude=[A, kaputt])  dasselbe
    clear([A, kaputt])        TypeError - die A sind geleert (nachgebaut)
    set_blocked([B, kaputt])  TypeError - B gesperrt (nachgebaut)
    ```
    """
    A, B, C = pg.USEREVENT, pg.KEYDOWN, pg.QUIT
    kurz = {A: "A", B: "B", C: "C"}
    kaputt = 2 ** 31

    def fuellen():
        pg.event.set_allowed(None)
        pg.event.clear()
        for i, t in enumerate([A, B, C] * 3):
            pg.event.post(pg.event.Event(t, zahl=i))

    aus = []
    for marke, ruf in (
        ("get([A,kaputt])", lambda: pg.event.get([A, kaputt])),
        ("get([kaputt,A])", lambda: pg.event.get([kaputt, A])),
        ("get(exclude=[A,kaputt])", lambda: pg.event.get(exclude=[A, kaputt])),
        ("clear([A,kaputt])", lambda: pg.event.clear([A, kaputt])),
        ("clear([kaputt,A])", lambda: pg.event.clear([kaputt, A])),
        ("set_blocked([B,kaputt])",
         lambda: pg.event.set_blocked([B, kaputt])),
        ("set_allowed([B,kaputt])",
         lambda: pg.event.set_allowed([B, kaputt])),
        ("peek([A,kaputt])", lambda: pg.event.peek([A, kaputt])),
        ("peek([kaputt,A])", lambda: pg.event.peek([kaputt, A])),
        ("get_blocked([B,kaputt])",
         lambda: pg.event.get_blocked([B, kaputt])),
    ):
        fuellen()
        try:
            antwort = repr(ruf())
        except Exception as e:                      # noqa: BLE001
            antwort = "%s: %s" % (type(e).__name__,
                                  _satz_gestalt(str(e), pg.__name__))
        gesperrt = [kurz[t] for t in (A, B, C) if pg.event.get_blocked(t)]
        pg.event.set_allowed(None)
        rest = "".join("%s%d" % (kurz[e.type], e.zahl)
                       for e in pg.event.get())
        aus.append((marke, antwort, gesperrt, rest))
    if len(aus) < 8:
        raise RuntimeError("Typliste halb: nur %d Antworten" % len(aus))
    return tuple(aus)


def pufferzeiger(pg):
    """`get_buffer()` - Laenge, Eltern, Rohbytes.

    Die Laenge ist `pitch * hoehe`, nicht `breite * hoehe * bytesize`;
    bei 24 Bit fallen die beiden Rechnungen auseinander, weil `pitch`
    aufgerundet ist.
    """
    def v(f):
        try:
            return f()
        except Exception as e:                      # noqa: BLE001
            return "%s: %s" % (type(e).__name__, e)

    aus = [pg.get_sdl_byteorder()]
    for d in (8, 16, 24, 32):
        s = pg.Surface((3, 2), depth=d)
        s.fill((255, 0, 0))
        b = v(lambda s=s: s.get_buffer())
        if isinstance(b, str):
            aus.append((d, b))
            continue
        aus.append((d, b.length, b.parent is s, repr(b), bytes(b.raw)))
    return tuple(aus)


_PUFFER_FELDER = ("format", "itemsize", "ndim", "shape", "strides",
                  "suboffsets", "readonly", "nbytes", "c_contiguous",
                  "f_contiguous", "contiguous")


def _puffer_ausgeber(pg):
    """`(Marke, Ruf)` fuer jeden Ausgeber eines Puffers.

    Gefragt wird **jede** Flaechentiefe mal **jede** Sichtart: bei 24 Bit
    fallen `pitch` und `breite * bytesize` auseinander, bei den
    Kanalsichten `'r'`..`'a'` ist `itemsize` 1 und die Schrittweiten
    stehen trotzdem in Pixelbreiten, und `get_view('3')` hat eine
    **negative** Schrittweite, wenn die Maschine BGR ablegt.
    """
    aus = []
    for tiefe, flagge in ((8, 0), (16, 0), (24, 0), (32, 0),
                          (32, pg.SRCALPHA)):
        def mach(tiefe=tiefe, flagge=flagge):
            s = pg.Surface((4, 3), flagge, tiefe)
            if tiefe == 8:
                s.set_palette([(i, i, i) for i in range(256)])
            s.fill((10, 20, 30))
            return s
        marke = "Surface%d%s" % (tiefe, "A" if flagge else "")
        aus.append(("%s.get_buffer" % marke, lambda m=mach: m().get_buffer()))
        for art in (0, 1, 2, 3, "0", "1", "2", "3", "r", "g", "b", "a"):
            aus.append(("%s.get_view(%r)" % (marke, art),
                        lambda m=mach, a=art: m().get_view(a)))
        aus.append(("%s selbst" % marke, lambda m=mach: m()))
    aus.append(("Mask(4,3)", lambda: pg.mask.Mask((4, 3))))
    aus.append(("Mask(4,3,fill)", lambda: pg.mask.Mask((4, 3), fill=True)))
    # Breiter als 32: die Maske braucht zwei Woerter je Zeile.
    aus.append(("Mask(70,3)", lambda: pg.mask.Mask((70, 3))))
    aus.append(("Color", lambda: pg.Color(1, 2, 3, 4)))
    aus.append(("PixelArray",
                lambda: pg.PixelArray(pg.Surface((4, 3), 0, 32))))
    aus.append(("PixelArray[1:]",
                lambda: pg.PixelArray(pg.Surface((4, 3), 0, 32))[1:]))
    aus.append(("Sound", lambda: pg.mixer.Sound(buffer=b"\x01\x02" * 200)))
    return aus


def pufferprotokoll(pg):
    """Jede Kennzahl jeder Sicht - so, wie `memoryview` sie meldet.

    Bis Runde 16 hatte jede Probe **eine** Sicht gefragt und davon
    `length` und die Bytes. Gemessen wurden hier zum ersten Mal alle elf
    Kennzahlen des Pufferprotokolls fuer **jeden** Ausgeber, und das gab
    56 abweichende Zeilen: `.raw` gab Bytes heraus, wo pygame
    `the bytes are not contiguous` sagt (13 Stellen, denn das verlangte
    `A` heisst C **oder** Fortran), die Maske meldete `I` statt `L`, und
    `strides: None` im Woerterbuch galt als weggelassen.

    Die Adressen werden ersetzt - sie sind in jedem Lauf andere.
    """
    def satz(e):
        return "%s: %s" % (type(e).__name__,
                           _satz_gestalt(str(e), pg.__name__))

    def wert(f):
        try:
            return re.sub(r"0x[0-9a-fA-F]+", "0xADR", repr(f()))
        except Exception as e:                       # noqa: BLE001
            return satz(e)

    aus = []
    for marke, ruf in _puffer_ausgeber(pg):
        try:
            o = ruf()
        except Exception as e:                       # noqa: BLE001
            aus.append((marke, "(bau)", satz(e)))
            continue
        try:
            m = memoryview(o)
        except Exception as e:                       # noqa: BLE001
            aus.append((marke, "memoryview", satz(e)))
            continue
        for feld in _PUFFER_FELDER:
            aus.append((marke, feld, wert(lambda m=m, f=feld: getattr(m, f))))
        aus.append((marke, "tobytes",
                    wert(lambda m=m: m.tobytes().hex()[:48])))
        aus.append((marke, "tolist", wert(lambda m=m: repr(m.tolist())[:60])))
        try:
            m.release()
        except Exception:                            # noqa: BLE001
            pass
        if type(o).__name__ == "BufferProxy":
            aus.append((marke, ".length", wert(lambda o=o: o.length)))
            aus.append((marke, ".parent",
                        wert(lambda o=o: type(o.parent).__name__)))
            aus.append((marke, ".raw", wert(lambda o=o: o.raw.hex()[:32])))
            aus.append((marke, ".ai", wert(lambda o=o: {
                k: (v if k != "data" else ("ADR", v[1]))
                for k, v in sorted(o.__array_interface__.items())})))
            aus.append((marke, ".as", wert(
                lambda o=o: type(o.__array_struct__).__name__)))
    if len(aus) < 700:
        raise RuntimeError("Pufferprotokoll: nur %d Kennzahlen" % len(aus))
    return tuple(aus)


def puffer_spaet(pg):
    """`BufferProxy(woerterbuch)` - **wann** wird es gelesen?

    pygames Erzeuger nimmt jedes Woerterbuch an, auch ein leeres; jeder
    der sieben Einwaende kommt beim ersten Gebrauch. Gelesen wird so
    lange neu, bis es einmal gelingt - danach steht das Ergebnis fest,
    auch wenn sich das Woerterbuch aendert.

    Und `before` feuert **danach**: holt sich der Puffer nicht, bleibt
    der Rueckruf aus. Das trennt die beiden Reihenfolgen, die man sonst
    fuer dieselbe haelt.

    Die Adresse ist die eines **eigenen** Speichers von `ctypes`, weil
    `.raw` und `write` wirklich dort lesen und schreiben.

    ## Vorher: `id(bytearray) + bytearray.__basicsize__ - 1`

    Das ist nicht der Inhalt des `bytearray`, sondern das **letzte Byte
    seines Kopfes** - unter CPython 3.14 das obere Byte von `ob_exports`
    (Versatz 48 bis 55, `__basicsize__` 56). `.raw` las von dort vier
    Bytes, drei davon **hinter** dem Objekt, und `write(b"\\x09", 0)`
    schrieb `0x09` in den Zaehler. Beim Freigeben meldete CPython dann

        SystemError: deallocated bytearray object has exported buffers

    - in **beiden** Engines, darum fiel es im Vergleich nie auf. Es stand
    zweimal in jeder Ausgabe dieses Werkzeugs, seit es diese Probe gibt.
    Ein Messgeraet, das fremden Speicher beschreibt, misst Glueck.
    """
    import ctypes

    def satz(e):
        return "%s: %s" % (type(e).__name__,
                           _satz_gestalt(str(e), pg.__name__))

    traeger = ctypes.create_string_buffer(b"\x01\x02\x03\x04" * 4, 16)
    adr = ctypes.addressof(traeger)
    gut = {"shape": (4,), "typestr": "|u1", "data": (adr, False)}
    kaputt = [
        ("leer", {}),
        ("ohne shape", {"typestr": "|u1", "data": (adr, False)}),
        ("ohne typestr", {"shape": (4,), "data": (adr, False)}),
        ("shape als Liste",
         {"shape": [4], "typestr": "|u1", "data": (adr, False)}),
        ("typestr zu kurz",
         {"shape": (4,), "typestr": "u1", "data": (adr, False)}),
        ("data als Zahl", {"shape": (4,), "typestr": "|u1", "data": adr}),
        ("strides None", dict(gut, strides=None)),
        ("strides zu kurz", dict(gut, strides=(1, 1))),
    ]
    aus = []
    # 1. Der Erzeuger nimmt alles - und jeder Zugriff beschwert sich.
    for marke, d in kaputt:
        try:
            b = pg.BufferProxy(dict(d))
        except Exception as e:                       # noqa: BLE001
            aus.append((marke, "(bau)", satz(e)))
            continue
        aus.append((marke, "(bau)", "ok"))
        for feld, ruf in (("length", lambda b=b: b.length),
                          ("parent", lambda b=b: type(b.parent).__name__),
                          ("raw", lambda b=b: b.raw.hex()),
                          ("repr", lambda b=b: repr(b)),
                          ("memoryview", lambda b=b: memoryview(b).nbytes),
                          ("ai", lambda b=b: sorted(b.__array_interface__)),
                          ("astruct", lambda b=b:
                           type(b.__array_struct__).__name__)):
            try:
                aus.append((marke, feld, repr(ruf())))
            except Exception as e:                   # noqa: BLE001
                aus.append((marke, feld, satz(e)))
    # 2. Gelesen wird, bis es gelingt - danach nicht mehr.
    leer = {}
    b = pg.BufferProxy(leer)
    try:
        aus.append(("nachfuellen", "leer", repr(b.length)))
    except Exception as e:                           # noqa: BLE001
        aus.append(("nachfuellen", "leer", satz(e)))
    leer.update(gut)
    aus.append(("nachfuellen", "gefuellt", repr(b.length)))
    leer["shape"] = (8,)
    aus.append(("nachfuellen", "geaendert", repr(b.length)))
    # 3. `before` feuert erst, wenn der Puffer da ist.
    for feld, ruf in (("length", lambda b: b.length),
                      ("parent", lambda b: b.parent),
                      ("raw", lambda b: b.raw),
                      ("repr", lambda b: repr(b)),
                      ("memoryview", lambda b: memoryview(b).nbytes),
                      ("ai", lambda b: b.__array_interface__),
                      ("astruct", lambda b: b.__array_struct__),
                      ("write", lambda b: b.write(b"\x09", 0)),
                      ("bool", lambda b: bool(b))):
        for art in ("gut", "kaputt"):
            gerufen = []
            d = dict(gut, parent=["die Eltern"],
                     before=lambda p: gerufen.append(p))
            if art == "kaputt":
                d["strides"] = None
            b = pg.BufferProxy(d)
            try:
                ruf(b)
                ergebnis = "ok"
            except Exception as e:                   # noqa: BLE001
                ergebnis = satz(e)
            aus.append(("before", "%s %s" % (feld, art),
                        (len(gerufen), ergebnis)))
    if len(aus) < 70:
        raise RuntimeError("Puffer spaet lesen: nur %d Antworten" % len(aus))
    return tuple(aus)


def pixelfeld_sperre(pg):
    """`PixelArray(s)` sperrt `s`, `close()` gibt sie frei.

    In pygame steht in `s.get_locks()` danach **das PixelArray selbst**,
    nicht die Flaeche.
    """
    s = pg.Surface((8, 8))
    pa = pg.PixelArray(s)
    gesperrt = (len(s.get_locks()), s.get_locks()[0] is pa if s.get_locks() else None)
    pa.close()
    return (gesperrt, len(s.get_locks()))


def namen(pg):
    """`__name__` und `__module__` - daran erkennt man eine Huelle.

    pygame schreibt `pygame.surface.Surface`; eine Huelle, die ihre
    Rust-Namen durchlaesst, schreibt `PySurface` - und jede
    Fehlermeldung, die einen Typnamen nennt, weicht dann ab.
    """
    aus = []
    for name in ("Surface", "Rect", "FRect", "Color", "Mask", "Font",
                 "Vector2", "Vector3"):
        t = getattr(pg, name, None) or getattr(pg.math, name, None)
        if t is None:
            aus.append((name, None))
            continue
        aus.append((name, t.__name__, t.__module__))
    for m in ("draw", "transform", "image", "surface", "rect", "color",
              "math", "mask", "mixer", "font", "display", "event"):
        mod = getattr(pg, m, None)
        aus.append((m, getattr(mod, "__name__", None)))
    return tuple(aus)


def zeichnen_ecken(pg):
    """Viertelkreise, Randbreiten und `closed` in allen Wahrheitsformen.

    `draw.rect(..., border_top_left_radius=...)` schaltet **einzelne**
    Viertel; ein Nachbau, der alle vier gleich behandelt, faellt hier auf.
    Und `closed` nimmt jede Wahrheit, nicht nur `True`/`False`.
    """
    def bild(f):
        s = pg.Surface((16, 16), pg.SRCALPHA)
        f(s)
        return bytes(pg.image.tobytes(s, "RGBA"))

    aus = []
    for wort in ("border_top_left_radius", "border_top_right_radius",
                 "border_bottom_left_radius", "border_bottom_right_radius"):
        for w in (0, 1, 3):
            aus.append(bild(lambda s, wort=wort, w=w: pg.draw.rect(
                s, (255, 0, 0), pg.Rect(1, 1, 13, 13), w,
                border_radius=2, **{wort: 5})))
    for c in (True, False, 1, 0, 2, "ja", "", None, [], [0]):
        aus.append(bild(lambda s, c=c: pg.draw.lines(
            s, (0, 255, 0), c, [(1, 1), (12, 3), (6, 13)], 2)))
    # `flood_fill` mit einer Musterflaeche statt einer Farbe.
    if hasattr(pg.draw, "flood_fill"):
        aus.append(bild(lambda s: pg.draw.flood_fill(s, (8, 8), (0, 0, 255))))
    return tuple(aus)


def frische_uhr(pg):
    u = pg.time.Clock()
    return (u.get_fps(), u.get_time(), u.get_rawtime())


def leer_poll(pg):
    pg.event.clear()
    e = pg.event.poll()
    return (e.type, e.type == pg.NOEVENT, repr(e), e.dict)


def nutzlast(pg):
    pg.event.clear()
    t = pg.USEREVENT + 3
    ok = pg.event.post(pg.event.Event(t, {"nutzlast": [1, 2], "code": 7}))
    g = pg.event.get()
    return (ok, len(g), g[0].type, g[0].dict, g[0].nutzlast, g[0].code, repr(g[0]))


def gesperrt(pg):
    pg.event.clear()
    pg.event.set_blocked(pg.QUIT)
    a = pg.event.get_blocked(pg.QUIT)
    ok = pg.event.post(pg.event.Event(pg.QUIT))
    pg.event.set_allowed(None)
    b = pg.event.get_blocked(pg.QUIT)
    nachher = len(pg.event.get(pg.QUIT))
    return (a, ok, b, nachher)


def wait_leer(pg):
    pg.event.clear()
    e = pg.event.wait(20)
    return (e.type, repr(e))


def keydown(pg):
    pg.event.clear()
    pg.event.post(pg.event.Event(pg.KEYDOWN, key=pg.K_a, mod=0,
                                 unicode="a", scancode=4))
    g = pg.event.get()
    return (len(g), g[0].type, g[0].key, g[0].mod, g[0].unicode,
            g[0].scancode, sorted(g[0].dict))


def info_werte(pg):
    i = pg.display.Info()
    return (i.hw, i.wm, i.video_mem, i.bitsize, i.bytesize, tuple(i.masks),
            tuple(i.shifts), tuple(i.losses), i.current_w, i.current_h,
            i.pixel_format, i.blit_hw, i.blit_hw_CC, i.blit_hw_A,
            i.blit_sw, i.blit_sw_CC, i.blit_sw_A)


def schrift_masse(pg, groesse):
    f = pg.font.Font(None, groesse)
    s = f.render("Hallo Welt", True, (255, 255, 255))
    return (f.get_height(), f.get_linesize(), f.get_ascent(), f.get_descent(),
            f.get_point_size(), f.size("Hallo Welt"), s.get_size(), s.get_flags())


def leerer_text(pg):
    f = pg.font.Font(None, 24)
    s = f.render("", True, (255, 255, 255))
    return (s.get_size(), s.get_flags())


def mit_hintergrund(pg):
    f = pg.font.Font(None, 24)
    s = f.render("Hi", True, (255, 255, 255), (0, 0, 0))
    return (s.get_flags(), s.get_size()[1])


def ohne_aa(pg):
    f = pg.font.Font(None, 24)
    s = f.render("Hi", False, (200, 100, 50))
    k = s.get_colorkey()
    return (s.get_flags(), s.get_size()[1], tuple(k) if k else None)


def schalter(pg):
    f = pg.font.Font(None, 24)
    vorher = (f.bold, f.italic, f.underline, f.strikethrough)
    f.bold = True
    f.italic = True
    nachher = (f.bold, f.italic, f.get_bold(), f.get_italic())
    f.set_bold(False)
    f.point_size = 30
    return (vorher, nachher, f.bold, f.get_point_size(), f.point_size)


# ---------------------------------------------------------------------------
#  Der Vergleich
# ---------------------------------------------------------------------------

# ---------------------------------------------------------------------------
#  Bekannte Grenzen
# ---------------------------------------------------------------------------
#
# Proben, die **nicht** uebereinstimmen koennen und deren Grund bekannt und
# nachgemessen ist. Sie werden gesondert gezaehlt statt stillschweigend
# uebergangen: der Lauf wird gruen, die Grenze bleibt sichtbar, und eine
# **neue** Abweichung faellt weiterhin auf. Dasselbe Verfahren wie beim
# Fuzzer mit `scale2x`.
#
# Wer eine Zeile hier eintraegt, schreibt die Begruendung dazu - eine
# Grenze ohne Begruendung ist ein verstecktes Versagen.

# **Drei Eintraege sind in einem Zug weggefallen**, und alle drei aus
# demselben Grund: der Pixelspeicher haelt jetzt Bytes im gemeldeten
# Format, und der Klangspeicher Bytes im Ausgabeformat des Mixers.
#
# * `puffer.unter 32 Bit` - "ein Puffer auf eine Flaeche mit 8, 16 oder 24
#   Bit gibt hier einen `BufferError`". Gibt es nicht mehr; die Bytes
#   stimmen jetzt, also darf der Puffer sie zeigen.
# * `pixelarray.tafelindex` - "auf einer Palettenflaeche ueberlebt der
#   Index den Weg durch den Speicher nicht". Er liegt jetzt selbst dort.
# * `sndarray.Felder` - "die Abtastwerte liegen immer 16-bittig". Tun sie
#   nicht mehr.
#
# Ein Eintrag hier, der nie anschlaegt, ist dieselbe Sorte Schaden wie ein
# veralteter Vermerk im Quelltext: er behauptet eine Grenze, die es nicht
# gibt. Deshalb zaehlt das Werkzeug sie mit und nennt sie einzeln - eine
# Grenze, die niemand mehr sieht, faellt beim naechsten Lauf auf.
BEKANNTE_GRENZEN = {
    "Erben":
        "Vier Eintraege von 179, alle derselben Gestalt: **pygame "
        "verbietet das Ableiten, hier geht es**. `_camera.Camera`, "
        "`_render.Image`, `_render.Renderer` und `_render.Texture` sind "
        "dort Cython-Typen ohne `Py_TPFLAGS_BASETYPE`; hier sind es "
        "gewoehnliche Python-Klassen in den eingebetteten Modulen, und "
        "eine Python-Klasse laesst sich immer ableiten. Das zu "
        "verbieten hiesse, ein `__init_subclass__` zu schreiben, das "
        "nichts tut als werfen - und es bricht kein Programm, das mit "
        "pygame laeuft: wer dort nicht ableiten kann, versucht es nicht. "
        "In die andere Richtung waren es drei, und die sind behoben "
        "(`newbuffer.Py_buffer`, `sprite.collide_rect_ratio`, "
        "`collide_circle_ratio` - ihnen fehlte PyO3s `subclass`).",
    "Objekte":
        "Acht Eintraege von 281, in zwei Gruppen. **`geometry.Line`** hat "
        "bei pygame **kein** `__eq__`: `Line(1,2,3,4) == Line(1,2,3,4)` "
        "ist dort `False`, und weil die Gleichheit die der Identitaet "
        "ist, laesst sich eine Strecke hashen und als Schluessel "
        "benutzen. Hier vergleicht sie ihre Werte und ist darum "
        "unhashbar - genau wie pygames eigener `Circle`, der es so "
        "macht: `Circle(1,2,3) == Circle(1,2,3)` ist dort `True` und "
        "`hash(circle)` ein `TypeError`. pygame ist an dieser Stelle mit "
        "sich selbst uneinig, und von den zwei Antworten ist die des "
        "`Circle` die brauchbare. **`Cursor.type` und `Cursor.data`** "
        "sind bei pygame beschreibbar, weil sie Faecher in einem "
        "`__dict__` sind - ohne jede Pruefung: `c.type = 77` geht dort. "
        "Hier werden sie aus dem Zeiger abgeleitet und sind nur lesbar; "
        "ein Fach ohne Pruefung nachzubauen hiesse, `type` und `data` "
        "auseinanderlaufen zu lassen. Alles andere an `Cursor` stimmt "
        "seit dieser Runde: hashen, `copy`, `deepcopy`, `pickle` und "
        "eigene Felder. Dazu nennt CPython in "
        "`'<' not supported between instances of ..` bei einer "
        "Python-Klasse den **kurzen** Namen (`Cursor`) und bei einer aus "
        "C oder Rust den ganzen (`@.cursors.Cursor`) - dasselbe wie bei "
        "der Grenze `__module__ der Typen`.",
    "Vektor Reste":
        "**Ein Fehler in pygame-ce 2.5.7, und zwar ein stiller.** "
        "`Vector3.rotate_y_ip_rad` und `rotate_z_ip_rad` drehen dort um "
        "die **x**-Achse. Gemessen an `(1, 2, 3)` mit einem halben "
        "Bogenmass:\n"
        "    rotate_x_rad     (1.0,      0.316889, 3.591599)\n"
        "    rotate_y_rad     (2.315859, 2.0,      2.153322)\n"
        "    rotate_z_rad     (-0.081269, 2.234591, 3.0)\n"
        "    rotate_x_ip_rad  (1.0,      0.316889, 3.591599)\n"
        "    rotate_y_ip_rad  (1.0,      0.316889, 3.591599)  <- x!\n"
        "    rotate_z_ip_rad  (1.0,      0.316889, 3.591599)  <- x!\n"
        "Die neun anderen Varianten stimmen; nur diese zwei sind "
        "verdrahtet auf die x-Fassung. Die Warnung dazu verraet den "
        "Grund: `vector3_rotate_y_rad_ip() now has all the functionality "
        "of vector3_rotate_y_ip_rad(), so vector3_rotate_y_ip_rad() will "
        "be deprecated` - beim Zusammenlegen der alten und neuen Namen "
        "ist die Verdrahtung verrutscht. Hier drehen sie um y und z. "
        "**Einen falschen Drehsinn nachzubauen waere schlimmer als jede "
        "Abweichung:** ein Programm bekaeme stillschweigend Unsinn "
        "heraus, und der Fehler stammte nicht einmal von hier. Dazu zwei "
        "Fehlertexte: `Vector2.update(1,2,3)` gibt bei pygame "
        "`Vector2() takes at most 2 arguments (3 given)` - es nennt den "
        "**Erzeuger**, nicht die Methode -, und `rotate_ip_rad(\"x\")` "
        "nennt keinen Argumentnamen; siehe die Grenze `Fehlertexte`.",
    "Fehlertexte":
        "Ein Eintrag von 616. **`mouse.set_relative_"
        "mode`** gibt bei pygame auf dieser Plattform `pygame.error: No "
        "relative mode implementation available` - der Aufruf ist dort "
        "nicht umgesetzt, und zwar unabhaengig vom Argument. Hier ist er "
        "umgesetzt, also kommt der Argumentfehler. Dieselbe Abwaegung wie "
        "bei `camera.query_image` und `get_raw`: eine Luecke "
        "nachzubauen waere die schlechtere Antwort. "
        "Hier stand \"vier Eintraege\", dazu die drei `XMP`-Saetze von "
        "`mixer_music`; die kamen in der Probe nicht mehr vor, dafuer zwei "
        "andere, die der Vermerk verdeckte: `midi.get_device_info(\"x\")` "
        "und `music.get_metadata` ohne Arbeitsverzeichnis im Satz - beide "
        "in Runde 18 behoben, gefunden ueber den Abdruck dieser Grenze. Die "
        "Saetze der Musik-Dekodierer (`XMP: ..`, `stb_vorbis_open_rwops: "
        "..`) stehen als eigene Grenze in `mycache/GRENZEN.md`.",
    "Argumentfehler":
        "Zwei Eintraege von 511, und keiner davon ist zu beheben. "
        "**`_render.Image`** nimmt bei pygame siebzehn Argumente ohne "
        "Einwand an, weil die Klasse dort ein **leerer Stummel** ist: "
        "`dir()` nennt keinen einzigen oeffentlichen Namen, `__init__` ist "
        "`object.__init__`, und `render_test.test_blit` schreibt daneben "
        "`# TODO Assert after Image implementation`. Die echte Klasse "
        "heisst `pygame._sdl2.video.Image`; hier ist es **eine** Klasse "
        "mit zwei Namen, also nimmt sie zwei bis drei Argumente und sagt "
        "das auch. Einen leeren Zwilling daneben zu bauen waere toter "
        "Quelltext fuer eine Zahl - dieselbe Abwaegung steht bei "
        "`Klassennamen`. **`draw.aaline`** meldet bei pygame eine Zahl, "
        "die pygame selbst widerlegt: mit siebzehn Argumenten sagt es "
        "`function takes at most 6 arguments (17 given)`, mit sechs "
        "`function takes at most 5 positional arguments (6 given)` - die "
        "sechste Stelle ist das abgekuendigte `blend`, das nur als "
        "Schluesselwort durchgeht. Hier sind es fuenf, und das ist die "
        "Wahrheit ueber die Funktion; `aaline(.., blend=1)` nimmt sie "
        "ebenso an wie pygame. Eine falsche Zahl nachzuschreiben, um eine "
        "Probe gruen zu bekommen, waere das Gegenteil von messen.",
    "Klassendoppel":
        "Zwei Namen mehr, und beide heissen `__release_buffer__`: "
        "`Color` und `PixelArray`. PyO3 legt den Schlitz an, sobald eine "
        "Klasse `__releasebuffer__` umsetzt - und umsetzen **muss** sie es "
        "hier, weil `puffer::fuellen` die Zahlenfelder (`shape`, `strides`) "
        "auf dem Haufen anlegt und sie in `view.internal` parkt. Ohne den "
        "Haken bleibt bei **jeder** Puffersicht ein Kasten liegen. pygames "
        "C-Typen brauchen ihn nicht: ihre Zahlenfelder stehen im Objekt "
        "selbst, also gibt es nichts freizugeben. Die Wahl ist also zwei "
        "Namen gegen ein Leck, und alles andere an dieser Probe stimmt - "
        "`Vector2.__iadd__`, `Cursor.__len__`, `PowerState.__match_args__` "
        "und die anderen zwanzig Klassen sind Zeichen fuer Zeichen gleich. "
        "Unter diesem Vermerk standen bis Runde 18 **vier** Namen: dazu "
        "`Cursor.__deepcopy__` (ueberfluessig, `__reduce__` deckt "
        "`deepcopy` ab) und `ScancodeWrapper.__getattr__` (jetzt ein "
        "ueberschriebenes `__getattribute__`, das ohnehin in `dir()` "
        "steht). Gefunden ueber den Abdruck dieser Grenze.",
    "Klassennamen":
        "`pygame._render.Image` hat **keinen einzigen** oeffentlichen "
        "Namen - gemessen, und das ist pygames eigene Merkwuerdigkeit: die "
        "echte Klasse mit ihren elf Namen heisst `pygame._sdl2.video.Image`, "
        "und `_render.Image` ist ein leerer Stummel (auch `_render.Renderer` "
        "und `_sdl2.video.Renderer` sind dort **zwei** Objekte mit 23 und 24 "
        "Namen). Hier ist es je **eine** Klasse, also hat `_render.Image` "
        "`draw` und `get_rect` zuviel. Einen leeren Zwilling daneben zu "
        "bauen waere toter Quelltext fuer eine Zahl. Alles andere stimmt: "
        "`BufferProxy` hat seine zwei deutschen Hilfsnamen nicht mehr, und "
        "`camera.AbstractCamera` nicht mehr zwei abstrakte Methoden zuviel.",
    "mixer.ogg umgerechnet":
        "OGG/Vorbis liest hier `lewton`, bei pygame `stb_vorbis` - und "
        "zwei Dekodierer eines **verlustbehafteten** Formats runden "
        "verschieden. Gemessen an `house_lo.ogg`: 51,2 % der 626.648 "
        "Werte sind gleich, der Mittelwert der Abweichung ist **0,52**, "
        "also im Wesentlichen eine Eins im letzten Bit. Groesser als 2 "
        "sind genau 178 Werte (0,03 %), und die liegen **alle an der "
        "Aussteuerungsgrenze**: wo stb_vorbis auf 32767 begrenzt, gibt "
        "lewton 32047. Das ist kein Fehler in der Kette davor - der "
        "Umrechner ist seit dem Umbau auf SDLs Sinc-Filter byteweise "
        "derselbe, was die drei verlustfreien Proben daneben zeigen.",
    "mixer.mp3 umgerechnet":
        "MP3 liest hier `symphonia`, bei pygame SDL_mixers eigener "
        "Dekodierer. Gemessen an `house_lo.mp3`: 49 % der 640.512 Werte "
        "gleich, Mittel **0,53**, groesste Abweichung **2**. Dieselbe "
        "Lage wie bei OGG, nur ohne Ausreisser - siehe dort.",
    "Namen der Typen und Module":
        "`__module__` sagt `rustgame_py.surface`, wo pygame "
        "`pygame.surface` sagt - und das bleibt so, denn es ist die "
        "**Wahrheit**: das Modul heisst `rustgame_py`. Ein `pygame.*` "
        "hineinzuschreiben waere gelogen, sobald jemand die Huelle unter "
        "ihrem eigenen Namen einsetzt statt als Ersatz. Die Typnamen "
        "selbst (`Surface`, `Rect`, `Color`, `Mask`, `Vector2`) stimmen "
        "alle, und - wichtiger - die **Fehlermeldungen** schreiben "
        "`pygame.surface.Surface`, weil pygames eigene Tests darauf "
        "pruefen. Betroffen ist nur, wer `type(x).__module__` vergleicht. "
        "Hier gemessen, damit ein *anderer* Unterschied in derselben "
        "Probe - ein falscher Typname etwa - nicht darin untergeht.",
    "Typliste halb angewandt":
        "Zwei Eintraege von zehn, und beide derselbe Fall: **eine kaputte "
        "Typliste nimmt bei pygame Ereignisse mit.** `get([A, 2**31])` "
        "und `get(exclude=[A, 2**31])` werfen hier wie dort "
        "`TypeError: type sequence must contain valid event types` - aber "
        "bei pygame sind die Ereignisse vom Typ `A` danach **weg**: es "
        "liest Glied fuer Glied und holt sie heraus, bevor es am zweiten "
        "scheitert. Wer den Fehler faengt, findet dort eine halb "
        "geleerte Schlange. Dieselbe Stueckweise ist bei `clear`, "
        "`set_blocked` und `set_allowed` **nachgebaut** (und bei `peek` "
        "und `get_blocked` das Abbrechen beim ersten Treffer, was "
        "aendert, ob es ueberhaupt einen Fehler gibt) - nur bei `get` "
        "nicht: die herausgenommenen Ereignisse gibt pygame niemandem "
        "mehr, der Aufruf wirft ja. Ein `get`, das die Schlange leert "
        "und nichts zurueckgibt, waere der eine Fall, in dem die Treue "
        "Daten vernichtet, die kein Programm mehr sehen kann. Nachgebaut "
        "ist stattdessen der Zeitpunkt: zuerst gelesen, dann gehandelt.",
    "unbekanntes Schluesselwort":
        "Drei Eintraege. Wo pygame `this function got an unexpected keyword "
        "argument 'y'` sagt, sagt die Huelle `scale_by() got an unexpected "
        "keyword argument 'y'`: PyO3s Argumentleser nennt die Funktion, "
        "pygames C-Fassung nennt keine. Beim zweiten Fall (`flip(.., z=1)`) "
        "zaehlt pygame zuerst: `function takes at most 3 arguments (4 "
        "given)` - `PyArg_ParseTupleAndKeywords` prueft die Anzahl vor den "
        "Namen, PyO3 die Namen vor der Anzahl. Genau treffen kann man die "
        "Saetze nur, indem "
        "die Funktion `**kwargs` selbst nimmt und sie selbst wirft. Das "
        "ist an **jeder** Funktion der Huelle zu tun oder an keiner, "
        "darum steht es als eine Grenze und nicht als 600 Einzelfaelle. "
        "Wo ein Test darauf prueft, wird die Stelle einzeln nachgezogen.",
    "display.gl_set_attribute":
        "pygame nimmt den Wunsch stillschweigend an (`None`), rustgame "
        "lehnt ihn mit `pygame.error` ab. Absicht, und die Begruendung "
        "steht im Kern: es gibt keinen GL-Kontext, auf den der Wunsch je "
        "wirken koennte. Ein `None` hiesse \"gemerkt, wird gemacht\" - "
        "ein Programm, das `set_mode(.., OPENGL)` plant, soll es hier "
        "erfahren und nicht erst beim ersten leeren Bild. `gl_get_attribute` "
        "verhaelt sich dagegen gleich: beide scheitern.",
    # **Ein Geraet, das es nicht gibt.** `pygame.midi.get_count()` gibt
    # auf dieser Maschine 2, rustgame 1. Die beiden bei pygame sind
    # `Microsoft MIDI Mapper` und `Microsoft GS Wavetable Synth`; der
    # erste ist seit Windows 8 ein **Altlast-Pseudogeraet**, das auf das
    # Standardgeraet zeigt und sonst nichts tut. PortMidi zaehlt es mit,
    # `midir` nicht.
    #
    # Darum vergleicht `midi.Geraeteliste` die **Form** der Auskunft und
    # nicht die Anzahl: dass jeder Eintrag fuenf Felder hat, dass die
    # ersten beiden `bytes` sind, dass ein Geraet Eingang **oder** Ausgang
    # ist, dass hinter dem Ende `None` steht und dass zwei Abfragen
    # dasselbe ergeben. Das stimmt bei beiden ueberein.
    #
    # **Drei Unterschiede, und bei allen dreien tut rustgame mehr.**
    #
    # Sie stehen hier nicht als Probe, sondern als Vermerk: keine Probe
    # vergleicht sie, weil der Vergleich einen Mangel bei pygame zur
    # Messlatte machen wuerde.
    #
    #     Aufruf             pygame (MSMF)   rustgame
    #     query_image()      immer False     True, solange sie laeuft
    #     get_raw()          immer None      die Bytes des Geraets
    #     get_controls()[2]  immer -1        der Reglerwert, sonst -1
    #
    # pygames MSMF-Stueck setzt diese drei nicht um; sein V4L2-Stueck gibt
    # bei `get_raw()` die Bytes heraus, die Luecke ist also keine Absicht.
    # Bei `query_image` wiegt sie schwer, denn pygames **eigenes**
    # Doku-Muster
    #
    #     while not cam.query_image():
    #         pass
    #     bild = cam.get_image()
    #
    # haengt unter Windows fuer immer. Eine Luecke nachzubauen, waere hier
    # die schlechtere Antwort - und `-1` bei der Helligkeit heisst
    # "weiss nicht", nicht "dunkel": wer den Regler abfragt, darf den
    # Wert auch nennen.
    #
    # Gemessen, vierzig Mal in Folge im Abstand von 20 ms:
    # `pygame.camera.Camera(..).query_image()` war **kein einziges Mal**
    # wahr.
    "_sdl2.roher Tonstrom":
        "`_sdl2.audio.AudioDevice` und `_sdl2.mixer.set_post_mix` brauchen "
        "einen **rohen Tonstrom mit Rueckruf**: pygame ruft dafuer aus "
        "einem Tonfaden heraus Python auf - gemessen siebenundzwanzig Mal "
        "in drei Zehntelsekunden, jeweils mit einem `memoryview` ueber "
        "2048 Bytes. Zwei Dinge stehen dem hier entgegen, und beide sind "
        "Entscheidungen aus dem Plan: rustgames Mixer hat **keine "
        "zentrale Mischstufe** (jeder Kanal geht einzeln an die Tonkiste, "
        "und alles Messbare rechnet der Zustand aus der Uhr aus), und ein "
        "Rueckruf aus dem Tonfaden muesste die Sperre des Auswerters "
        "holen - ein Tonfaden, der darauf wartet, verschluckt Abschnitte. "
        "Also ein klarer Fehler statt eines Hakens, der angenommen und "
        "nie gerufen wird: ein Programm, das damit ein Pegelbild zeichnet, "
        "saehe sonst eine flache Linie und suchte den Fehler bei sich. "
        "Nebenbei gemessen: pygame nimmt auch `set_post_mix(7)` an und "
        "**stuerzt ab**, sobald der Haken gerufen wird (kein `TypeError`, "
        "ein Speicherzugriffsfehler in `recording_cb`) - das ist der "
        "Grund, warum diese Grenze keine schmerzt. Alles ohne Rueckruf "
        "geht: `get_audio_device_names` nennt dieselben Geraete wie SDL, "
        "`pygame.mixer` spielt Klaenge, `set_post_mix(None)` schaltet ab.",
    "Musik-Dekodierer kaputt":
        "Vier Eintraege, eine Sorte: auf dem Weg von `mixer.music` meldet "
        "sich bei pygame der **Dekodierer selbst**, und sein Satz haengt an "
        "dessen Innereien - `stb_vorbis_open_rwops: VORBIS_unexpected_eof` "
        "(je nach Fall auch `VORBIS_invalid_setup` oder `No samples in "
        "ogg/vorbis stream.`), `music_drflac: corrupt flac file (bad "
        "stream).`, `music_minimp3: corrupt mp3 file (bad tags).`, WavPacks "
        "`not compatible with this version of WavPack file!`. Hier stehen "
        "andere Dekodierer (lewton, symphonia, claxon, ein eigener "
        "WavPack-Leser), und ihre Saetze nachzusprechen hiesse, die "
        "Fehlerpfade von vier C-Bibliotheken nachzubauen. **Bis Runde 19 "
        "waren es zehn**: was keiner nimmt, endet bei libxmp mit `XMP: "
        "Unrecognized file format`, ein erkanntes, kaputtes Trackermodul mit "
        "`XMP: Error loading file` - beides ist jetzt nachgebaut, samt der "
        "Erkennung (`IMPM`, `SCRM` mit Dateityp 0x10, `M.K.` bei 1080; nur "
        "`MThd` bleibt bei `Unrecognized audio format`). Und **AIFF** liest "
        "rustgame jetzt auf beiden Wegen, siehe `Klangdateien AIFF und VOC`. "
        "Auf dem Weg von `mixer.Sound` sagt pygame immer `Unrecognized audio "
        "format` - das war schon nachgebaut, siehe `Klangdateien kaputt`. "
        "Eine AIFF-Musik mit Rate 0 laedt bei pygame und spielt danach "
        "**endlos Stille**; hier laedt sie und ist sofort zu Ende.",
    "camera.get_backends":
        "pygame meldet auf dieser Maschine `['_camera (MSMF)', "
        "'OpenCV']`, rustgame `['_camera (MSMF)']`. Der zweite Eintrag "
        "ist kein Treiber, sondern ein **zweites Kamerastueck**: pygame "
        "hat neben seinem C-Modul eine reine Python-Fassung ueber `cv2`, "
        "und die taucht in der Liste auf, weil `opencv-python` in "
        "derselben Umgebung liegt. Ohne `cv2` gibt pygame denselben einen "
        "Eintrag wie rustgame. Hier gibt es den einen Weg - unter Windows "
        "Media Foundation, unter Linux V4L2, unter macOS AVFoundation, "
        "alles ueber `nokhwa`. Ein OpenCV-Weg daneben waere kein "
        "Zugewinn: er fuehrt zum selben Geraet.",
    "Alpha-Blit unter 32 Bit":
        "Runde 21. Alpha auf ein Ziel mit 24, 16, 15 oder 8 Bit mischt "
        "anders als pygame: 24 Bit um eine Stufe (Pixel- und "
        "Flaechenalpha), 16 und 15 Bit um eine Stufe des Kanals (nur "
        "Flaechenalpha), 8 Bit mit Farbtafel bei jeder Art mit Alpha und "
        "weit daneben. Auf 32 Bit ist alles gleich, `BLEND_RGBA_ADD` auf "
        "jeder Tiefe auch. Bei pygame mischt dort SDLs eigener Blitter "
        "(`SDL_BlitA.c`) und nicht `alphablit.c` - zu uebertragen.",
    "JPEG lesen":
        "Runde 21. SDL_image liest JPEG mit libjpeg-turbo (`JDCT_ISLOW`, "
        "weiches Hochrechnen), rustgame mit `zune-jpeg`. Groesse, Tiefe und "
        "Flaggen stimmen, die Pixel nicht: von 41 Dateien (pygames "
        "eigene, PyGameZ' Logos, Windows-Hintergruende) 39 verschieden, um "
        "2 bis 61 Stufen. Abhilfe ist der Dekodierer von libjpeg-turbo, "
        "uebertragen.",
    "JPEG und PNG schreiben":
        "Runde 21. PNG: dieselben Pixel und Bloecke (`IHDR`, `PLTE`, "
        "`IDAT`, `IEND`), aber eine andere Kompression - pygame schreibt "
        "ueber libpng und zlib. JPEG: pygame ruft `IMG_SaveJPG(.., 85)` "
        "ueber libjpeg-turbo, rustgame kodiert mit `jpeg-encoder`; die "
        "Guete ist seit Runde 21 dieselbe (vorher 75), Datei und Pixel "
        "sind es nicht.",
}


#: **Was jede Grenze deckt** - der Abdruck ihrer Abweichung (`abdruck`).
#:
#: Eine Grenze hing bis Runde 18 nur am **Namen** der Probe. Das deckte
#: alles, was in dieser Probe je abweichen wuerde: `sysfont` war als "drei
#: Eintraege" vermerkt und zeigte sechs - `freesansbold` fehlte (der
#: Schriftordner wurde zu spaet gesetzt), und das stand unter einem
#: Vermerk, der von Umlauten sprach. Jetzt zaehlt eine Grenze nur, solange
#: ihre Abweichung **dieselbe** ist; jede andere ist ein Fund.
#:
#: Nachtragen nur bewusst: `--abdruecke` gibt die gemessenen aus, und wer
#: einen uebernimmt, hat vorher mit `--genau` nachgesehen.
GRENZEN_ABDRUCK = {
    'display.gl_set_attribute': 'a215bb522f38',
    'unbekanntes Schluesselwort': 'af159413dbc0',
    'Namen der Typen und Module': '9b7ef1ed9853',
    'Typliste halb angewandt': '28c7006c44d7',
    'mixer.ogg umgerechnet': '8de4b0b20513',
    'mixer.mp3 umgerechnet': '322829684158',
    'camera.get_backends': '74d3ce4fc705',
    '_sdl2.roher Tonstrom': 'a1af1d1d560a',
    'Klassendoppel': '15d352c7399b',
    'Vektor Reste': 'bbf60faf3a49',
    'Objekte': '7b9a70beae89',
    'Erben': '124a20893376',
    'Argumentfehler': '1007b0bf2641',
    'Fehlertexte': 'cef3317a71f6',
    'Klassennamen': 'e01b4aa9b7e3',
    'Musik-Dekodierer kaputt': '4d51f7a7b8b8',
    'Alpha-Blit unter 32 Bit': 'c081d19cbf20',
    'JPEG lesen': 'e4a39d0c85c6',
    'JPEG und PNG schreiben': 'ec926235fb16',
}


def abdruck(a, b):
    """Ein kurzer Fingerabdruck **der Abweichung** zwischen `a` und `b`.

    SHA-256 ueber genau die Zeilen, die `gegenueber` zeigt - also ueber
    das, was ein Mensch beim Nachsehen liest, nicht ueber `repr` der ganzen
    Werte. Siehe `GRENZEN_ABDRUCK`.
    """
    import hashlib
    return hashlib.sha256(
        "\n".join(gegenueber(a, b)).encode("utf-8")).hexdigest()[:12]


def gegenueber(a, b, tiefe=0, weg=""):
    """Zeigt **wo** zwei Werte auseinandergehen - rekursiv.

    Zwei Listen von vierzig Eintraegen, von denen einer abweicht: ohne
    diese Funktion steht in der Ausgabe nur, dass sie verschieden sind.
    Hier steht der Weg dorthin (`[3][1]`) und beide Werte **ganz**.
    """
    ein = "  " * (tiefe + 1)
    if repr(a) == repr(b):
        return []
    if (isinstance(a, (list, tuple)) and isinstance(b, (list, tuple))
            and type(a) is type(b)):
        # **Eine Liste von Namen ist eine Menge.** Steht in beiden nur Text
        # und ist jede Seite fuer sich sortiert, dann ist die Frage nicht
        # "was steht an Stelle 7", sondern "was fehlt und was ist zuviel".
        # Ohne diesen Zweig verschiebt ein einziger fehlender Name die
        # ganze Liste, und die Ausgabe ist zwanzig Zeilen Rauschen.
        if (a and b and all(isinstance(x, str) for x in a)
                and all(isinstance(x, str) for x in b)
                and list(a) == sorted(a) and list(b) == sorted(b)):
            fehlt = [x for x in a if x not in set(b)]
            zuviel = [x for x in b if x not in set(a)]
            # **Eine** Zeile, nicht zwei: zwei Zeilen untereinander sind
            # bei zwanzig Eintraegen nicht mehr einander zuzuordnen.
            return ["%s%s %d/%d  fehlt %s  zuviel %s" % (
                ein, weg, len(a), len(b), fehlt or "-", zuviel or "-")]
        zeilen = []
        if len(a) != len(b):
            zeilen.append("%s%s Laenge %d gegen %d" % (ein, weg, len(a), len(b)))
        for i in range(min(len(a), len(b))):
            # **Wenn der Eintrag ein Paar (Name, Wert) ist und die Namen
            # gleich sind, wird der Name der Weg.** Ohne das steht in der
            # Ausgabe `[6][1]`, und man zaehlt die Liste mit dem Finger ab,
            # um herauszufinden, welches Modul das war.
            teil = "%s[%d]" % (weg, i)
            x, y = a[i], b[i]
            if (isinstance(x, (list, tuple)) and isinstance(y, (list, tuple))
                    and len(x) == len(y) == 2 and isinstance(x[0], str)
                    and x[0] == y[0]):
                zeilen += gegenueber(x[1], y[1], tiefe,
                                     "%s{%s}" % (weg, x[0]))
                continue
            zeilen += gegenueber(x, y, tiefe, teil)
        for i in range(len(b), len(a)):
            zeilen.append("%s%s[%d] nur pygame:   %r" % (ein, weg, i, a[i]))
        for i in range(len(a), len(b)):
            zeilen.append("%s%s[%d] nur rustgame: %r" % (ein, weg, i, b[i]))
        # Ein Netz unter allem: `repr` ist verschieden (sonst waeren wir
        # nicht hier), und keine Zeile sagt warum. Das darf nicht still
        # bleiben - eine rote Probe ohne Befund ist ein Messfehler.
        if not zeilen:
            zeilen += ["%s%s repr verschieden, kein Einzelbefund" % (ein, weg),
                       "%s  pygame   %s" % (ein, repr(a)[:300]),
                       "%s  rustgame %s" % (ein, repr(b)[:300])]
        return zeilen
    if isinstance(a, dict) and isinstance(b, dict):
        zeilen = []
        for k in sorted(set(a) | set(b), key=repr):
            if k not in a:
                zeilen.append("%s%s[%r] nur rustgame: %r" % (ein, weg, k, b[k]))
            elif k not in b:
                zeilen.append("%s%s[%r] nur pygame:   %r" % (ein, weg, k, a[k]))
            else:
                zeilen += gegenueber(a[k], b[k], tiefe, "%s[%r]" % (weg, k))
        # **Dieselben Paare, andere Reihenfolge.** Die Schleife oben
        # sortiert die Schluessel und sieht das darum nicht - verglichen
        # wird aber `repr`, und das kennt die Reihenfolge. So stand die
        # Probe `sysfont` auf ABWEICHUNG und zeigte keine einzige Zeile.
        if not zeilen and list(a) != list(b):
            zeilen.append("%s%s Reihenfolge der Schluessel" % (ein, weg))
            zeilen.append("%s  pygame   %r" % (ein, list(a)))
            zeilen.append("%s  rustgame %r" % (ein, list(b)))
        return zeilen
    if isinstance(a, str) and isinstance(b, str) and max(len(a), len(b)) > 90:
        # Bei langen Texten die **erste** abweichende Stelle nennen - sonst
        # sucht man 446 Zeichen mit dem Auge ab.
        i = next((i for i in range(min(len(a), len(b))) if a[i] != b[i]),
                 min(len(a), len(b)))
        return ["%s%s Text, %d gegen %d Zeichen, erster Unterschied bei %d"
                % (ein, weg, len(a), len(b), i),
                "%s  pygame   ...%r..." % (ein, a[max(0, i - 30):i + 30]),
                "%s  rustgame ...%r..." % (ein, b[max(0, i - 30):i + 30])]
    return ["%s%s pygame   %r" % (ein, weg, a),
            "%s%s rustgame %r" % (ein, weg, b)]


def main():
    # **--genau <Teilname>**: nur die Proben, deren Name den Text enthaelt,
    # und die dafuer ganz. Ohne das schneidet die Ausgabe bei 200 Zeichen
    # ab, und bei einer Probe mit vierzig Eintraegen sieht man dann, dass
    # etwas anders ist, aber nicht was.
    genau = None
    if "--genau" in sys.argv:
        genau = sys.argv[sys.argv.index("--genau") + 1]

    import pygame
    echt = proben(pygame)

    sys.path.insert(0, HUELLE)
    try:
        import rustgame_py as rg
    except ImportError as e:
        print("rustgame_py nicht gefunden (%s)." % e)
        return 2
    nach = proben(rg)

    if len(echt) != len(nach):
        print("Verschieden viele Proben: %d gegen %d" % (len(echt), len(nach)))
        return 1

    # **Ein gefallener Waechter darf nicht als "gleich" durchgehen.**
    #
    # `probe()` macht aus einer Ausnahme eine Zeichenkette, damit eine
    # Probe, die bei **einer** Engine wirft, auffaellt. Wirft sie bei
    # **beiden** - und genau das tut eine Untergrenze, die zu hoch steht
    # -, dann sind die zwei Zeichenketten gleich, und der Lauf meldet
    # "0 Abweichungen", ohne einen einzigen Wert verglichen zu haben.
    #
    # Zweimal passiert, beide Male in Runde 14 an der Probe
    # `Dokumentation` (Grenze 200 und dann 150 bei 142 echten Namen).
    # Jede Untergrenze in dieser Datei wirft `RuntimeError`, und keine
    # Probe hat sonst einen Grund, einen zu melden.
    gefallen = [n for n, v in echt
                if isinstance(v, str) and v.startswith("RuntimeError: ")]
    gefallen += [n for n, v in nach
                 if isinstance(v, str) and v.startswith("RuntimeError: ")
                 and n not in gefallen]
    if gefallen:
        print("**Waechter gefallen** - diese Proben haben nichts gemessen:")
        for n in gefallen:
            wert = dict(echt).get(n) or dict(nach).get(n)
            print("   %-28s %s" % (n, wert))
        print()
        print("Eine Probe, die wirft, vergleicht ihre Ausnahme mit sich")
        print("selbst. Untergrenze pruefen (sie muss **unter** dem")
        print("gemessenen Wert liegen).")
        return 1

    schlecht = []
    bekannt = []
    veraendert = []
    for (n1, v1), (n2, v2) in zip(echt, nach):
        assert n1 == n2, (n1, n2)
        if repr(v1) != repr(v2):
            if n1 not in BEKANNTE_GRENZEN:
                schlecht.append((n1, v1, v2))
                continue
            # **Die Grenze deckt nur, was vermerkt ist.** Siehe
            # `GRENZEN_ABDRUCK`: weicht die Abweichung selbst ab, ist es
            # eine neue - auch wenn die Probe denselben Namen hat.
            ist = abdruck(v1, v2)
            soll = GRENZEN_ABDRUCK.get(n1)
            if soll is not None and soll != ist:
                veraendert.append((n1, soll, ist))
                schlecht.append((n1, v1, v2))
            else:
                bekannt.append((n1, v1, v2))

    if "--abdruecke" in sys.argv:
        # Zum bewussten Nachtragen: die Abdruecke aller Grenzen, die
        # gerade anschlagen, in der Schreibweise von `GRENZEN_ABDRUCK`.
        for (n1, v1), (_, v2) in zip(echt, nach):
            if n1 in BEKANNTE_GRENZEN and repr(v1) != repr(v2):
                print("    %r: %r," % (n1, abdruck(v1, v2)))
        return 0

    if genau is not None:
        print("pygame-ce %s gegen rustgame-Huelle - genau: %r"
              % (pygame.version.ver, genau))
        traf = 0
        for (n1, v1), (_, v2) in zip(echt, nach):
            if genau.lower() not in n1.lower():
                continue
            traf += 1
            gleich = repr(v1) == repr(v2)
            print()
            print("%s  %s%s" % (n1, "gleich" if gleich else "ABWEICHUNG",
                                " [bekannte Grenze]"
                                if n1 in BEKANNTE_GRENZEN else ""))
            if not gleich:
                for zeile in gegenueber(v1, v2):
                    print(zeile)
        print()
        print("%d Probe(n) mit %r im Namen." % (traf, genau))
        return 0

    print("pygame-ce %s gegen rustgame-Huelle" % pygame.version.ver)
    print("%d Proben, %d Abweichungen, %d bekannte Grenzen"
          % (len(echt), len(schlecht), len(bekannt)))
    if bekannt:
        print()
        for n, a, b in bekannt:
            print("  [bekannt] %s" % n)
            print("      pygame   %s" % (repr(a)[:120],))
            print("      rustgame %s" % (repr(b)[:120],))
            print("      %s" % BEKANNTE_GRENZEN[n])
    if veraendert:
        print()
        print("**Grenzen, deren Abweichung sich geaendert hat** - sie zaehlen")
        print("als Abweichung, denn der Vermerk beschreibt etwas anderes:")
        for n, soll, ist in veraendert:
            print("  %-28s Abdruck vermerkt %s, gemessen %s" % (n, soll, ist))
        print("Was genau: --genau <Name>. Ist die neue Lage gewollt, Vermerk")
        print("und Abdruck nachziehen (--abdruecke).")
    if not schlecht:
        print()
        print("Dasselbe Skript, beide Engines, Wert fuer Wert dasselbe.")
        return 0
    print()
    for n, a, b in schlecht:
        sa, sb = repr(a), repr(b)
        if len(sa) > 200:
            sa = sa[:200] + "..."
        if len(sb) > 200:
            sb = sb[:200] + "..."
        print("%-42s pygame   %s" % (n, sa))
        print("%-42s rustgame %s" % ("", sb))
    return 1


if __name__ == "__main__":
    sys.exit(main())
