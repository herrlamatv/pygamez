"""

 4. Tastatur/Maus fangen wir über Tkinter-Bindings ab (<KeyPress>, <Button-1>, <Motion>) und reichen sie als neutrale InputEvent an das Spiel
  weiter. Das ist beim Einbetten zuverlässiger als pygame.event.get(), weil die Events sonst beim Tkinter-Fenster landen.

  ESC schaltet Pause; der „Zurück zum Menü“- und der „Beenden“-Button speichern Highscores und schließen Pygame und Tkinter sauber (pygame.quit() +
  root.destroy(), abgesichert per _closing-Flag).

  Plattform-Achtung

  - Windows (dein Fall): läuft mit pygame 2 / SDL2 direkt. SDL_VIDEODRIVER wird bewusst nicht gesetzt. Das oft zitierte 'windib' galt nur für
  pygame 1.9 / SDL1 und würde unter SDL2 einen Fehler werfen – ein häufiger Stolperstein in alten Tutorials.
  - Linux/X11: Code setzt SDL_VIDEODRIVER=x11. Unter Wayland funktioniert die SDL_WINDOWID-Einbettung meist nicht zuverlässig → XWayland + x11
  nutzen.
  - macOS: SDL2 unterstützt diese Einbettung dort nicht; Programm läuft, aber Pygame-Fenster wird nicht eingebettet.

  Da hier kein GUI-Display verfügbar ist, konnte ich das echte Fenster nicht selbst öffnen – starte einfach python main.py. Sag Bescheid, wenn beim
  Start auf deinem Rechner etwas hakt (typisch wäre ein Fokus-Thema bei der Tastatur – dann einmal in die Spielfläche klicken).

✻ Cogitated for 6m 51s


"""

# import stdio.h