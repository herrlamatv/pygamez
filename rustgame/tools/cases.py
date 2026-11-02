# -*- coding: utf-8 -*-
"""
rustgame/tools/cases.py
=======================
Die Falltabelle - **eine** Quelle fuer beide Seiten.

Jeder Fall beschreibt eine kleine Zeichenszene als Liste von Operationen.
``gen_golden.py`` fuehrt sie mit echtem pygame-ce aus und legt das Ergebnis
als PNG ab; derselbe Fall wird nach Rust uebersetzt und dort nachgespielt.
Weicht auch nur ein Pixel ab, faellt der Test.

Eine Operation ist ein Tupel ``(name, *argumente)``. Die Namen entsprechen
den pygame-Aufrufen; wer einen neuen Fall braucht, traegt ihn einfach hier
ein - beide Seiten ziehen automatisch nach.

Toleranz: ``0`` heisst pixelgenau. Das ist der Normalfall, denn alle von
PyGameZ benutzten Zeichenfunktionen rastern **binaer** (Alpha nur 0 oder
255, gemessen).

Hier stand: "Nur ``smoothscale`` braucht Spielraum - pygame ist dort nicht
einmal mit sich selbst pixelgleich". Die vier Faelle trugen ``tol=2``, und
genau diese Toleranz hat verdeckt, dass rustgame nur den ``GENERIC``-Weg
rechnete, pygame aber ``SSE2`` - jedes Referenzbild war um bis zu zwei
Stufen daneben und trotzdem gruen. Seit Runde 18 rechnet rustgame beide
Wege, und die Faelle sind pixelgenau. *Eine Toleranz ist ein Vermerk, und
ein Vermerk ist der bequemste Platz fuer einen Fehler.*
"""

# Farben, die in den Faellen immer wieder vorkommen.
SCHWARZ = (0, 0, 0, 255)
WEISS = (255, 255, 255, 255)
ROT = (220, 60, 60, 255)
GRUEN = (60, 200, 120, 255)
BLAU = (70, 110, 240, 255)
HALB = (200, 100, 50, 128)

FAELLE = []


def fall(name, size, ops, flags=0, tol=0, gruppe="allgemein"):
    """Traegt einen Fall in die Tabelle ein."""
    FAELLE.append({
        "name": name,
        "size": size,
        "flags": flags,           # 0 oder "SRCALPHA"
        "tol": tol,
        "gruppe": gruppe,
        "ops": ops,
    })


# ---------------------------------------------------------------------------
#  Fuellen und Blitten
# ---------------------------------------------------------------------------

fall("fill_einfach", (16, 12), [("fill", ROT)], gruppe="surface")
fall("fill_rect", (16, 12),
     [("fill", SCHWARZ), ("fill_rect", GRUEN, (2, 3, 8, 5))], gruppe="surface")
fall("fill_rect_ragt_raus", (16, 12),
     [("fill", SCHWARZ), ("fill_rect", BLAU, (-4, -4, 10, 10))], gruppe="surface")

for name, flag in [("add", "BLEND_RGB_ADD"), ("sub", "BLEND_RGB_SUB"),
                   ("mult", "BLEND_RGB_MULT"), ("min", "BLEND_RGB_MIN"),
                   ("max", "BLEND_RGB_MAX"), ("rgba_add", "BLEND_RGBA_ADD"),
                   ("rgba_sub", "BLEND_RGBA_SUB"), ("rgba_mult", "BLEND_RGBA_MULT"),
                   ("rgba_min", "BLEND_RGBA_MIN"), ("rgba_max", "BLEND_RGBA_MAX")]:
    fall("blit_%s" % name, (8, 8),
         [("fill", (100, 150, 200, 255)),
          ("blit_gefuellt", (50, 60, 70, 128), (0, 0), flag)],
         flags="SRCALPHA", gruppe="blend")

fall("blit_alpha_standard", (8, 8),
     [("fill", (100, 150, 200, 255)), ("blit_gefuellt", (50, 60, 70, 128), (0, 0), 0)],
     gruppe="blend")
fall("blit_flaechen_alpha", (8, 8),
     [("fill", SCHWARZ), ("blit_gefuellt_alpha", (200, 100, 50, 128), (0, 0), 128)],
     gruppe="blend")
fall("blit_versetzt", (16, 16),
     [("fill", SCHWARZ), ("blit_gefuellt", (255, 0, 0, 255), (5, 4), 0)],
     gruppe="blend")
fall("blit_teilweise_draussen", (16, 16),
     [("fill", SCHWARZ), ("blit_gefuellt", (255, 0, 0, 255), (12, 12), 0)],
     gruppe="blend")
fall("blit_negativ_versetzt", (16, 16),
     [("fill", SCHWARZ), ("blit_gefuellt", (255, 0, 0, 255), (-3, -3), 0)],
     gruppe="blend")
fall("blit_mit_clip", (16, 16),
     [("fill", SCHWARZ), ("set_clip", (4, 4, 6, 6)),
      ("blit_gefuellt", (255, 0, 0, 255), (0, 0), 0)],
     gruppe="blend")

# ---------------------------------------------------------------------------
#  draw.rect - das meistbenutzte Primitiv (900 Aufrufe, 719 davon mit Radius)
# ---------------------------------------------------------------------------

fall("rect_gefuellt", (16, 12), [("fill", SCHWARZ), ("rect", WEISS, (2, 2, 10, 7), 0, -1)],
     gruppe="draw_rect")
fall("rect_umriss_1", (16, 12), [("fill", SCHWARZ), ("rect", WEISS, (2, 2, 10, 7), 1, -1)],
     gruppe="draw_rect")
fall("rect_umriss_3", (20, 16), [("fill", SCHWARZ), ("rect", WEISS, (3, 3, 13, 9), 3, -1)],
     gruppe="draw_rect")
fall("rect_radius_gefuellt", (11, 11), [("fill", SCHWARZ), ("rect", WEISS, (0, 0, 11, 11), 0, 3)],
     gruppe="draw_rect")
fall("rect_radius_umriss", (24, 16), [("fill", SCHWARZ), ("rect", WEISS, (2, 2, 20, 12), 2, 4)],
     gruppe="draw_rect")
fall("rect_radius_gross", (20, 20), [("fill", SCHWARZ), ("rect", WEISS, (0, 0, 20, 20), 0, 10)],
     gruppe="draw_rect")
fall("rect_radius_zu_gross", (20, 20), [("fill", SCHWARZ), ("rect", WEISS, (0, 0, 20, 20), 0, 50)],
     gruppe="draw_rect")
fall("rect_breite_groesser_als_rect", (16, 16),
     [("fill", SCHWARZ), ("rect", WEISS, (2, 2, 8, 8), 20, -1)], gruppe="draw_rect")
fall("rect_leer", (16, 16), [("fill", SCHWARZ), ("rect", WEISS, (4, 4, 0, 5), 0, -1)],
     gruppe="draw_rect")
fall("rect_alpha", (16, 16),
     [("fill", (50, 50, 50, 255)), ("rect", (255, 0, 0, 128), (2, 2, 10, 10), 0, -1)],
     flags="SRCALPHA", gruppe="draw_rect")

# ---------------------------------------------------------------------------
#  draw.circle / ellipse / line / polygon / arc
# ---------------------------------------------------------------------------

fall("circle_gefuellt", (13, 13), [("fill", SCHWARZ), ("circle", WEISS, (6, 6), 5, 0)],
     gruppe="draw_circle")
fall("circle_umriss_1", (13, 13), [("fill", SCHWARZ), ("circle", WEISS, (6, 6), 5, 1)],
     gruppe="draw_circle")
fall("circle_umriss_2", (21, 21), [("fill", SCHWARZ), ("circle", WEISS, (10, 10), 9, 2)],
     gruppe="draw_circle")
fall("circle_radius_0", (9, 9), [("fill", SCHWARZ), ("circle", WEISS, (4, 4), 0, 0)],
     gruppe="draw_circle")
fall("circle_radius_1", (9, 9), [("fill", SCHWARZ), ("circle", WEISS, (4, 4), 1, 0)],
     gruppe="draw_circle")
fall("circle_am_rand", (16, 16), [("fill", SCHWARZ), ("circle", WEISS, (0, 0), 6, 0)],
     gruppe="draw_circle")

fall("ellipse_gefuellt", (20, 14), [("fill", SCHWARZ), ("ellipse", WEISS, (2, 2, 16, 10), 0)],
     gruppe="draw_ellipse")
fall("ellipse_umriss", (20, 14), [("fill", SCHWARZ), ("ellipse", WEISS, (2, 2, 16, 10), 1)],
     gruppe="draw_ellipse")
fall("ellipse_flach", (20, 6), [("fill", SCHWARZ), ("ellipse", WEISS, (1, 1, 18, 4), 0)],
     gruppe="draw_ellipse")

fall("line_waagerecht", (16, 8), [("fill", SCHWARZ), ("line", WEISS, (2, 4), (13, 4), 1)],
     gruppe="draw_line")
fall("line_senkrecht", (8, 16), [("fill", SCHWARZ), ("line", WEISS, (4, 2), (4, 13), 1)],
     gruppe="draw_line")
fall("line_schraeg", (16, 16), [("fill", SCHWARZ), ("line", WEISS, (1, 1), (14, 12), 1)],
     gruppe="draw_line")
fall("line_dick", (16, 12), [("fill", SCHWARZ), ("line", WEISS, (1, 5), (14, 5), 3)],
     gruppe="draw_line")
fall("line_dick_schraeg", (20, 20), [("fill", SCHWARZ), ("line", WEISS, (2, 2), (17, 15), 4)],
     gruppe="draw_line")
fall("line_punkt", (8, 8), [("fill", SCHWARZ), ("line", WEISS, (4, 4), (4, 4), 1)],
     gruppe="draw_line")

fall("polygon_dreieck", (20, 20),
     [("fill", SCHWARZ), ("polygon", WEISS, [(2, 18), (10, 2), (18, 18)], 0)],
     gruppe="draw_polygon")
fall("polygon_umriss", (20, 20),
     [("fill", SCHWARZ), ("polygon", WEISS, [(2, 18), (10, 2), (18, 18)], 1)],
     gruppe="draw_polygon")
fall("polygon_viereck", (20, 20),
     [("fill", SCHWARZ), ("polygon", WEISS, [(3, 3), (16, 5), (14, 17), (5, 15)], 0)],
     gruppe="draw_polygon")
fall("polygon_konkav", (24, 24),
     [("fill", SCHWARZ),
      ("polygon", WEISS, [(2, 2), (21, 2), (21, 21), (12, 10), (2, 21)], 0)],
     gruppe="draw_polygon")

fall("lines_offen", (20, 20),
     [("fill", SCHWARZ), ("lines", WEISS, False, [(2, 2), (10, 15), (17, 4)], 1)],
     gruppe="draw_line")
fall("lines_geschlossen", (20, 20),
     [("fill", SCHWARZ), ("lines", WEISS, True, [(2, 2), (10, 15), (17, 4)], 1)],
     gruppe="draw_line")

fall("arc_halb", (20, 20), [("fill", SCHWARZ), ("arc", WEISS, (2, 2, 16, 16), 0.0, 3.14159, 1)],
     gruppe="draw_arc")
fall("arc_dick", (24, 24), [("fill", SCHWARZ), ("arc", WEISS, (2, 2, 20, 20), 0.5, 2.5, 3)],
     gruppe="draw_arc")

# ---------------------------------------------------------------------------
#  Kantenglaettung und Fuellen
# ---------------------------------------------------------------------------
#
# Die einzigen Zeichenfunktionen, die **mischen**. Ihre Formel ist keine
# der vier aus `blend_formula` - sie rechnet in Fliesskomma und schneidet
# ab: eine schwarze `aaline` auf Weiss ergibt bei halber Deckung 127,
# nicht 128.

fall("aaline_schraeg", (12, 8),
     [("fill", SCHWARZ), ("aaline", WEISS, (1, 1), (10, 6), 1)], gruppe="draw_aa")
fall("aaline_halbe_pixel", (12, 8),
     [("fill", SCHWARZ), ("aaline", WEISS, (1.5, 1.5), (10.5, 6.5), 1)], gruppe="draw_aa")
fall("aaline_steil", (8, 16),
     [("fill", SCHWARZ), ("aaline", WEISS, (1, 1), (6, 14), 1)], gruppe="draw_aa")
fall("aaline_dick_gerade", (14, 10),
     [("fill", SCHWARZ), ("aaline", WEISS, (1, 1), (12, 8), 4)], gruppe="draw_aa")
fall("aaline_dick_ungerade", (14, 10),
     [("fill", SCHWARZ), ("aaline", WEISS, (1, 1), (12, 8), 5)], gruppe="draw_aa")
# Breite 0 zeichnet hier **doch** - genau umgekehrt zu `line`.
fall("aaline_breite_null", (12, 8),
     [("fill", SCHWARZ), ("aaline", WEISS, (1, 1), (10, 6), 0)], gruppe="draw_aa")
fall("aaline_ueber_den_rand", (12, 8),
     [("fill", SCHWARZ), ("aaline", WEISS, (-8, -4), (20, 12), 1)], gruppe="draw_aa")
fall("aaline_clip", (16, 12),
     [("fill", SCHWARZ), ("set_clip", (3, 2, 7, 6)),
      ("aaline", WEISS, (0, 0), (15, 11), 1)], gruppe="draw_aa")
fall("aaline_clip_dick", (16, 12),
     [("fill", SCHWARZ), ("set_clip", (3, 2, 7, 6)),
      ("aaline", WEISS, (0, 0), (15, 11), 4)], gruppe="draw_aa")
fall("aaline_auf_farbe", (12, 8),
     [("fill", ROT), ("aaline", BLAU, (1, 1), (10, 6), 1)], gruppe="draw_aa")
fall("aaline_alpha", (12, 8),
     [("aaline", WEISS, (1, 1), (10, 6), 1)], flags="SRCALPHA", gruppe="draw_aa")

fall("aalines_offen", (16, 12),
     [("fill", SCHWARZ), ("aalines", ROT, False, [(2, 2), (11, 4), (6, 10)])],
     gruppe="draw_aa")
fall("aalines_geschlossen", (14, 12),
     [("fill", SCHWARZ), ("aalines", ROT, True, [(2, 2), (11, 4), (6, 10)])],
     gruppe="draw_aa")
# Mit genau zwei Punkten zeichnet pygame die Strecke **zweimal** - das ist
# sichtbar und deshalb ein Fall wert. Aber nur mit ganzzahligen Enden:
# bei gebrochenen liest pygame dort ueber das Feldende hinaus und ist nicht
# reproduzierbar.
fall("aalines_zwei_punkte", (12, 8),
     [("fill", SCHWARZ), ("aalines", ROT, False, [(1, 1), (10, 6)])], gruppe="draw_aa")
fall("aalines_knick", (20, 16),
     [("fill", SCHWARZ), ("aalines", ROT, False, [(1, 1), (4, 14), (18, 15)])],
     gruppe="draw_aa")

fall("aacircle_duenn", (12, 12),
     [("fill", SCHWARZ), ("aacircle", ROT, (6, 6), 4, 1)], gruppe="draw_aa")
fall("aacircle_gefuellt", (12, 12),
     [("fill", SCHWARZ), ("aacircle", ROT, (6, 6), 4, 0)], gruppe="draw_aa")
fall("aacircle_dick", (16, 16),
     [("fill", SCHWARZ), ("aacircle", ROT, (8, 8), 6, 3)], gruppe="draw_aa")
fall("aacircle_r1", (8, 8),
     [("fill", SCHWARZ), ("aacircle", ROT, (4, 4), 1, 1)], gruppe="draw_aa")
fall("aacircle_gross", (32, 32),
     [("fill", SCHWARZ), ("aacircle", GRUEN, (16, 16), 14, 2)], gruppe="draw_aa")
fall("aacircle_am_rand", (12, 12),
     [("fill", SCHWARZ), ("aacircle", ROT, (1, 1), 5, 1)], gruppe="draw_aa")
fall("aacircle_auf_farbe", (12, 12),
     [("fill", BLAU), ("aacircle", ROT, (6, 6), 4, 1)], gruppe="draw_aa")
fall("aacircle_alpha", (12, 12),
     [("aacircle", ROT, (6, 6), 4, 1)], flags="SRCALPHA", gruppe="draw_aa")

fall("flood_fill_kreis", (16, 12),
     [("fill", SCHWARZ), ("circle", WEISS, (8, 6), 5, 1), ("flood_fill", ROT, (8, 6))],
     gruppe="draw_aa")
fall("flood_fill_wand", (16, 12),
     [("fill", SCHWARZ), ("line", WEISS, (8, 0), (8, 11), 1),
      ("flood_fill", ROT, (2, 2))], gruppe="draw_aa")
# Von-Neumann-Nachbarschaft: ueber Ecken geht es nicht.
fall("flood_fill_diagonal", (8, 8),
     [("fill", SCHWARZ), ("line", WEISS, (0, 0), (7, 7), 1),
      ("flood_fill", ROT, (7, 0))], gruppe="draw_aa")
fall("flood_fill_clip", (16, 12),
     [("fill", SCHWARZ), ("set_clip", (2, 1, 8, 6)), ("flood_fill", ROT, (4, 3))],
     gruppe="draw_aa")
fall("flood_fill_alpha", (12, 8),
     [("circle", WEISS, (6, 4), 3, 1), ("flood_fill", (0, 255, 0, 128), (0, 0))],
     flags="SRCALPHA", gruppe="draw_aa")

# ---------------------------------------------------------------------------
#  transform
# ---------------------------------------------------------------------------

# Achtung, eine Falle, in die diese Faelle schon einmal gelaufen sind:
# `muster` skaliert das Rohmuster auf die Flaechengroesse. Ist die
# Musterkantenlaenge gleich der Flaechengroesse *und* gleich der Zielgroesse,
# dann ist das nachfolgende `scale` ein Aufruf auf dieselbe Groesse - und den
# behandelt pygame als blosses `memcpy`. Der Filter waere dann nie geprueft.
# Deshalb gilt hier: **Flaechengroesse = Mustergroesse != Zielgroesse.**
fall("scale_hoch", (4, 4),
     [("fill", SCHWARZ), ("muster", 4, 4), ("scale", (16, 16))], gruppe="transform")
fall("scale_runter", (16, 16),
     [("fill", SCHWARZ), ("muster", 16, 16), ("scale", (4, 4))], gruppe="transform")
fall("smoothscale_runter", (16, 16),
     [("fill", SCHWARZ), ("muster", 16, 16), ("smoothscale", (8, 8))],
     gruppe="transform")
fall("smoothscale_hoch", (8, 8),
     [("fill", SCHWARZ), ("muster", 8, 8), ("smoothscale", (16, 16))],
     gruppe="transform")
# Krumme Groessen treffen andere Zweige des Filters als glatte Halbierungen:
# hier laufen Verkleinern (waagerecht) und Vergroessern (senkrecht) zugleich.
fall("smoothscale_krumm", (13, 7),
     [("fill", SCHWARZ), ("muster", 13, 7), ("smoothscale", (6, 11))],
     gruppe="transform")
fall("flip_waagerecht", (12, 8),
     [("fill", SCHWARZ), ("muster", 12, 8), ("flip", True, False)], gruppe="transform")
fall("flip_senkrecht", (12, 8),
     [("fill", SCHWARZ), ("muster", 12, 8), ("flip", False, True)], gruppe="transform")
fall("rotate_90", (12, 8),
     [("fill", SCHWARZ), ("muster", 12, 8), ("rotate", 90.0)], gruppe="transform")
fall("rotate_180", (12, 8),
     [("fill", SCHWARZ), ("muster", 12, 8), ("rotate", 180.0)], gruppe="transform")
# Rechte Winkel nimmt pygame ueber einen eigenen, einfachen Weg. Erst ein
# schraeger Winkel prueft die Abbildung mit Festkommaarithmetik und die
# Hintergrundfarbe der entstehenden Zwickel.
fall("rotate_45", (12, 8),
     [("fill", SCHWARZ), ("muster", 12, 8), ("rotate", 45.0)], gruppe="transform")
# Schraege Winkel jenseits der Viertelkreise - und einer ueber 360 Grad,
# weil pygame dort modulo rechnet statt zu meckern.
fall("rotate_schraeg", (14, 9),
     [("fill", SCHWARZ), ("muster", 14, 9), ("rotate", 12.5)], gruppe="transform")
fall("rotate_rueckwaerts", (11, 11),
     [("fill", SCHWARZ), ("muster", 11, 11), ("rotate", -135.0)], gruppe="transform")
fall("rotate_ueber_voll", (10, 6),
     [("fill", SCHWARZ), ("muster", 10, 6), ("rotate", 417.0)], gruppe="transform")

fall("scale_by_halb", (16, 12),
     [("fill", SCHWARZ), ("muster", 16, 12), ("scale_by", 0.5, 0.5)], gruppe="transform")
fall("scale_by_krumm", (9, 7),
     [("fill", SCHWARZ), ("muster", 9, 7), ("scale_by", 1.75, 0.6)], gruppe="transform")
fall("smoothscale_by_klein", (12, 12),
     [("fill", SCHWARZ), ("muster", 12, 12), ("smoothscale_by", 0.4, 1.3)],
     gruppe="transform")

fall("scale2x_muster", (7, 5),
     [("fill", SCHWARZ), ("muster", 7, 5), ("scale2x",)], gruppe="transform")
fall("scale2x_formen", (10, 10),
     [("fill", SCHWARZ), ("rect", ROT, (1, 1, 5, 5), 0, 2),
      ("circle", GRUEN, (7, 7), 3, 1), ("scale2x",)], gruppe="transform")

fall("grayscale_muster", (12, 10),
     [("fill", SCHWARZ), ("muster", 12, 10), ("grayscale",)], gruppe="transform")
# Die dreizehn Nachzuegler aus `transform`. Alle Faelle stammen aus dem
# Pruefstand, der sie gegen echtes pygame durchgerechnet hat - Toleranz 0.
fall("invert_muster", (12, 8),
     [("fill", SCHWARZ), ("muster", 12, 8), ("invert",)], gruppe="transform")
fall("laplacian_kanten", (13, 9),
     [("fill", SCHWARZ), ("rect", WEISS, (3, 2, 7, 5), 0, -1), ("laplacian",)],
     gruppe="transform")
# Zeigt die stehengebliebenen `sample`-Werte am Rand - siehe transform.rs.
fall("laplacian_rand", (5, 5),
     [("fill", SCHWARZ), ("rect", (255, 200, 100, 255), (0, 0, 1, 5), 0, -1),
      ("rect", (250, 210, 90, 255), (4, 0, 1, 5), 0, -1), ("laplacian",)],
     gruppe="transform")
fall("box_blur_r2", (16, 12),
     [("fill", SCHWARZ), ("muster", 16, 12), ("box_blur", 2, True)], gruppe="transform")
fall("box_blur_ohne_rand", (16, 12),
     [("fill", SCHWARZ), ("muster", 16, 12), ("box_blur", 2, False)], gruppe="transform")
fall("gaussian_blur_r2", (16, 12),
     [("fill", SCHWARZ), ("muster", 16, 12), ("gaussian_blur", 2, True)], gruppe="transform")
# Radius 0 ergibt **schwarz**, nicht die Kopie - eine Null durch Null im
# C-Original. Gemessen, nicht ausgedacht.
fall("gaussian_blur_r0_ist_schwarz", (8, 8),
     [("fill", ROT), ("gaussian_blur", 0, True)], gruppe="transform")
fall("hsl_ton", (12, 8),
     [("fill", SCHWARZ), ("muster", 12, 8), ("hsl", 60.0, 0.0, 0.0)], gruppe="transform")
fall("hsl_hell_voll", (12, 8),
     [("fill", SCHWARZ), ("muster", 12, 8), ("hsl", 400.0, 0.17, 1.0)], gruppe="transform")
fall("hsl_saettigung_weg", (12, 8),
     [("fill", SCHWARZ), ("muster", 12, 8), ("hsl", 0.0, -1.0, 0.0)], gruppe="transform")
fall("chop_mitte", (12, 8),
     [("fill", SCHWARZ), ("muster", 12, 8), ("chop", 4, 3, 3, 2)], gruppe="transform")
# Ein negatives Rechteck macht die Flaeche **groesser**.
fall("chop_negativ_waechst", (8, 6),
     [("fill", SCHWARZ), ("muster", 8, 6), ("chop", 1, 1, -2, -2)], gruppe="transform")
fall("pixelate_3", (15, 12),
     [("fill", SCHWARZ), ("muster", 15, 12), ("pixelate", 3)], gruppe="transform")
fall("pixelate_gross", (12, 9),
     [("fill", SCHWARZ), ("muster", 12, 9), ("pixelate", 7)], gruppe="transform")
# Ohne Alphakanal gibt pygame die frisch angelegte, **leere** Flaeche
# zurueck - weder die Quelle noch die Farbe.
fall("solid_overlay_ohne_alpha_ist_leer", (10, 8),
     [("fill", ROT), ("solid_overlay", GRUEN, False)], gruppe="transform")
fall("solid_overlay_alpha", (10, 8),
     [("fill", (0, 0, 0, 0)), ("circle", (200, 100, 50, 200), (5, 4), 3, 0),
      ("solid_overlay", (255, 9, 8, 77), False)],
     flags="SRCALPHA", gruppe="transform")
fall("solid_overlay_keep_alpha", (10, 8),
     [("fill", (0, 0, 0, 0)), ("circle", (200, 100, 50, 200), (5, 4), 3, 0),
      ("solid_overlay", (255, 9, 8, 77), True)],
     flags="SRCALPHA", gruppe="transform")
fall("rotozoom_90", (12, 8),
     [("fill", SCHWARZ), ("muster", 12, 8), ("rotozoom", 90.0, 1.0)], gruppe="transform")
fall("rotozoom_45", (12, 8),
     [("fill", SCHWARZ), ("muster", 12, 8), ("rotozoom", 45.0, 1.0)], gruppe="transform")
fall("rotozoom_schraeg_klein", (14, 9),
     [("fill", SCHWARZ), ("muster", 14, 9), ("rotozoom", 12.5, 0.6)], gruppe="transform")
fall("rotozoom_nur_zoom", (9, 7),
     [("fill", SCHWARZ), ("muster", 9, 7), ("rotozoom", 0.0, 2.0)], gruppe="transform")
fall("rotozoom_zoom_runter", (16, 12),
     [("fill", SCHWARZ), ("muster", 16, 12), ("rotozoom", 0.0, 0.4)], gruppe="transform")
fall("rotozoom_rueckwaerts", (11, 11),
     [("fill", SCHWARZ), ("muster", 11, 11), ("rotozoom", -135.0, 1.3)], gruppe="transform")

fall("grayscale_mit_alpha", (10, 8),
     [("fill", (0, 0, 0, 0)), ("muster", 10, 8),
      ("blit_gefuellt", (255, 0, 0, 120), (2, 2), 0), ("grayscale",)],
     flags="SRCALPHA", gruppe="transform")


def gruppen():
    """Alle Gruppennamen in Reihenfolge des ersten Auftretens."""
    gesehen = []
    for f in FAELLE:
        if f["gruppe"] not in gesehen:
            gesehen.append(f["gruppe"])
    return gesehen
