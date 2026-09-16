/*
 * util.js - Grundlagen der PyGameZ-Web-Engine
 * ============================================
 * Entspricht store.py / i18n.py / settings.py / highscore.py / stats.py der
 * Desktop-Version, nur im Browser: gespeichert wird im localStorage.
 *
 *   PG.store      - JSON-Speicher (localStorage, Präfix "pygamez.")
 *   PG.t(key, p)  - Übersetzung (14 Sprachen, Fallback Deutsch)
 *   PG.rand       - Zufall (uniform/randint/choice/shuffle/...), PG.Random(seed)
 *   PG.Rect       - pygame.Rect-Nachbau
 *   PG.settings   - Einstellungen (global + je Spiel)
 *   PG.highscore  - Highscores
 *   PG.stats      - kleine Statistik (Partien, Siege, Spielzeit)
 */
(function () {
  "use strict";

  const PG = (window.PG = window.PG || {});

  PG.VERSION = "web-1.0";
  // Logische Spielfläche, auf die ALLE Spiele zeichnen (wird skaliert).
  PG.W = 800;
  PG.H = 600;

  // ---------------------------------------------------------------- Speicher
  PG.store = {
    prefix: "pygamez.",
    get(key, def) {
      try {
        const raw = window.localStorage.getItem(this.prefix + key);
        return raw == null ? def : JSON.parse(raw);
      } catch (e) {
        return def;
      }
    },
    set(key, value) {
      try {
        window.localStorage.setItem(this.prefix + key, JSON.stringify(value));
      } catch (e) {
        /* Speicher voll/gesperrt -> ohne Speichern weiterspielen */
      }
    },
    remove(key) {
      try {
        window.localStorage.removeItem(this.prefix + key);
      } catch (e) {}
    },
  };

  // ------------------------------------------------------------ Mathe/Zufall
  PG.TAU = Math.PI * 2;
  PG.clamp = (v, lo, hi) => (v < lo ? lo : v > hi ? hi : v);
  PG.lerp = (a, b, f) => a + (b - a) * f;
  PG.dist = (x1, y1, x2, y2) => Math.hypot(x2 - x1, y2 - y1);
  PG.sign = (v) => (v > 0 ? 1 : v < 0 ? -1 : 0);
  PG.radians = (d) => (d * Math.PI) / 180;
  PG.degrees = (r) => (r * 180) / Math.PI;
  /** Python-Modulo (Ergebnis hat das Vorzeichen des Divisors). */
  PG.mod = (a, b) => ((a % b) + b) % b;

  /** Zufallsgenerator mit festem Seed (mulberry32) - wie random.Random(seed). */
  class Random {
    constructor(seed) {
      if (seed === undefined || seed === null) {
        this._rnd = Math.random;
      } else {
        let s = typeof seed === "number" ? seed >>> 0 : Random.hash(String(seed));
        this._rnd = function () {
          s = (s + 0x6d2b79f5) >>> 0;
          let t = s;
          t = Math.imul(t ^ (t >>> 15), t | 1);
          t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
          return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
        };
      }
    }
    static hash(str) {
      let h = 2166136261 >>> 0;
      for (let i = 0; i < str.length; i++) {
        h ^= str.charCodeAt(i);
        h = Math.imul(h, 16777619);
      }
      return h >>> 0;
    }
    /** 0 <= x < 1 */
    random() {
      return this._rnd();
    }
    uniform(a, b) {
      return a + (b - a) * this._rnd();
    }
    /** Ganzzahl a..b INKLUSIVE (wie Python random.randint). */
    randint(a, b) {
      return a + Math.floor(this._rnd() * (b - a + 1));
    }
    /** wie Python random.randrange(start, stop[, step]) */
    randrange(start, stop, step) {
      if (stop === undefined) {
        stop = start;
        start = 0;
      }
      step = step || 1;
      const n = Math.ceil((stop - start) / step);
      return start + step * Math.floor(this._rnd() * n);
    }
    choice(arr) {
      return arr[Math.floor(this._rnd() * arr.length)];
    }
    /** mischt IN PLACE und gibt das Array zurück */
    shuffle(arr) {
      for (let i = arr.length - 1; i > 0; i--) {
        const j = Math.floor(this._rnd() * (i + 1));
        const t = arr[i];
        arr[i] = arr[j];
        arr[j] = t;
      }
      return arr;
    }
    sample(arr, k) {
      return this.shuffle(arr.slice()).slice(0, k);
    }
    /** Gewichtete Auswahl (wie random.choices(arr, weights)[0]) */
    weighted(arr, weights) {
      let total = 0;
      for (const w of weights) total += w;
      let r = this._rnd() * total;
      for (let i = 0; i < arr.length; i++) {
        r -= weights[i];
        if (r < 0) return arr[i];
      }
      return arr[arr.length - 1];
    }
    gauss(mu = 0, sigma = 1) {
      const u = 1 - this._rnd();
      const v = this._rnd();
      return mu + sigma * Math.sqrt(-2 * Math.log(u)) * Math.cos(PG.TAU * v);
    }
  }
  PG.Random = Random;
  PG.rand = new Random();

  // ------------------------------------------------------------------- Rect
  /** Nachbau von pygame.Rect (ganz bewusst mit denselben Namen). */
  class Rect {
    constructor(x = 0, y = 0, w = 0, h = 0) {
      if (x instanceof Rect) {
        ({ x, y, w, h } = x);
      } else if (Array.isArray(x)) {
        if (x.length === 2 && Array.isArray(x[0])) {
          h = x[1][1];
          w = x[1][0];
          y = x[0][1];
          x = x[0][0];
        } else {
          [x, y, w, h] = x;
        }
      }
      this.x = x;
      this.y = y;
      this.w = w;
      this.h = h;
    }
    get width() { return this.w; }
    set width(v) { this.w = v; }
    get height() { return this.h; }
    set height(v) { this.h = v; }
    get left() { return this.x; }
    set left(v) { this.x = v; }
    get top() { return this.y; }
    set top(v) { this.y = v; }
    get right() { return this.x + this.w; }
    set right(v) { this.x = v - this.w; }
    get bottom() { return this.y + this.h; }
    set bottom(v) { this.y = v - this.h; }
    get centerx() { return this.x + this.w / 2; }
    set centerx(v) { this.x = v - this.w / 2; }
    get centery() { return this.y + this.h / 2; }
    set centery(v) { this.y = v - this.h / 2; }
    get center() { return [this.centerx, this.centery]; }
    set center(p) { this.centerx = p[0]; this.centery = p[1]; }
    get size() { return [this.w, this.h]; }
    set size(p) { this.w = p[0]; this.h = p[1]; }
    get topleft() { return [this.x, this.y]; }
    set topleft(p) { this.x = p[0]; this.y = p[1]; }
    get topright() { return [this.right, this.y]; }
    set topright(p) { this.right = p[0]; this.y = p[1]; }
    get bottomleft() { return [this.x, this.bottom]; }
    set bottomleft(p) { this.x = p[0]; this.bottom = p[1]; }
    get bottomright() { return [this.right, this.bottom]; }
    set bottomright(p) { this.right = p[0]; this.bottom = p[1]; }
    get midtop() { return [this.centerx, this.y]; }
    set midtop(p) { this.centerx = p[0]; this.y = p[1]; }
    get midbottom() { return [this.centerx, this.bottom]; }
    set midbottom(p) { this.centerx = p[0]; this.bottom = p[1]; }
    get midleft() { return [this.x, this.centery]; }
    set midleft(p) { this.x = p[0]; this.centery = p[1]; }
    get midright() { return [this.right, this.centery]; }
    set midright(p) { this.right = p[0]; this.centery = p[1]; }

    copy() { return new Rect(this.x, this.y, this.w, this.h); }
    /** collidepoint(x, y) oder collidepoint([x, y]) - rechte/untere Kante exklusiv */
    collidepoint(px, py) {
      if (Array.isArray(px)) { py = px[1]; px = px[0]; }
      if (px == null || py == null) return false;
      return px >= this.x && px < this.x + this.w && py >= this.y && py < this.y + this.h;
    }
    colliderect(r) {
      // wie pygame 2: Rechtecke ohne Fläche kollidieren mit nichts
      if (!this.w || !this.h || !r.w || !r.h) return false;
      return this.x < r.x + r.w && r.x < this.x + this.w && this.y < r.y + r.h && r.y < this.y + this.h;
    }
    contains(r) {
      return r.x >= this.x && r.y >= this.y && r.right <= this.right && r.bottom <= this.bottom;
    }
    /** neues, um dx/dy vergrößertes Rechteck (Mitte bleibt) */
    inflate(dx, dy) {
      return new Rect(this.x - dx / 2, this.y - dy / 2, this.w + dx, this.h + dy);
    }
    inflateIp(dx, dy) {
      this.x -= dx / 2; this.y -= dy / 2; this.w += dx; this.h += dy;
      return this;
    }
    move(dx, dy) { return new Rect(this.x + dx, this.y + dy, this.w, this.h); }
    moveIp(dx, dy) { this.x += dx; this.y += dy; return this; }
    union(r) {
      const x = Math.min(this.x, r.x), y = Math.min(this.y, r.y);
      return new Rect(x, y, Math.max(this.right, r.right) - x, Math.max(this.bottom, r.bottom) - y);
    }
    /** Rechteck, das innerhalb von r gehalten wird (wie pygame clamp) */
    clamp(r) {
      const out = this.copy();
      if (out.w >= r.w) out.centerx = r.centerx;
      else out.x = PG.clamp(out.x, r.x, r.right - out.w);
      if (out.h >= r.h) out.centery = r.centery;
      else out.y = PG.clamp(out.y, r.y, r.bottom - out.h);
      return out;
    }
    toArray() { return [this.x, this.y, this.w, this.h]; }
  }
  PG.Rect = Rect;
  /** Rect aus beliebiger Form ([x,y,w,h] / Rect / x,y,w,h) */
  PG.rect = (x, y, w, h) => (x instanceof Rect ? x : new Rect(x, y, w, h));

  // ------------------------------------------------------------------- i18n
  PG.LANGS = [
    ["de", "Deutsch"], ["en", "English"], ["fr", "Français"], ["es", "Español"],
    ["pt", "Português"], ["pl", "Polski"], ["tr", "Türkçe"], ["da", "Dansk"],
    ["no", "Norsk"], ["sv", "Svenska"], ["fi", "Suomi"], ["cs", "Čeština"],
    ["sl", "Slovenščina"], ["hr", "Hrvatski"],
  ];
  const LANG_CODES = PG.LANGS.map((l) => l[0]);

  // Texte, die es nur in der Web-Version gibt (Sidebar/Overlay).
  const WEB_STR = {
    de: { "web.pause": "Pause", "web.resume": "Weiter", "web.fullscreen": "Vollbild", "web.settings": "Einstellungen", "web.language": "Sprache", "web.theme": "Design", "web.sound": "Sound", "web.volume": "Lautstärke", "web.search": "Suchen…", "web.no_results": "Keine Treffer", "web.reset_scores": "Highscores löschen", "web.reset_confirm": "Wirklich alle Highscores löschen?", "web.capture": "Klicken, um die Maus zu steuern", "web.error": "Fehler im Spiel - siehe Konsole (F12)", "web.wiki_search": "Wiki durchsuchen…", "web.games_count": "{n} Spiele" },
    en: { "web.pause": "Pause", "web.resume": "Resume", "web.fullscreen": "Fullscreen", "web.settings": "Settings", "web.language": "Language", "web.theme": "Design", "web.sound": "Sound", "web.volume": "Volume", "web.search": "Search…", "web.no_results": "No results", "web.reset_scores": "Reset highscores", "web.reset_confirm": "Really delete all highscores?", "web.capture": "Click to control with the mouse", "web.error": "Game error - see console (F12)", "web.wiki_search": "Search the wiki…", "web.games_count": "{n} games" },
    fr: { "web.pause": "Pause", "web.resume": "Reprendre", "web.fullscreen": "Plein écran", "web.settings": "Paramètres", "web.language": "Langue", "web.theme": "Thème", "web.sound": "Son", "web.volume": "Volume", "web.search": "Rechercher…", "web.no_results": "Aucun résultat", "web.reset_scores": "Effacer les records", "web.reset_confirm": "Vraiment effacer tous les records ?", "web.capture": "Cliquez pour contrôler avec la souris", "web.error": "Erreur du jeu - voir la console (F12)", "web.wiki_search": "Rechercher dans le wiki…", "web.games_count": "{n} jeux" },
    es: { "web.pause": "Pausa", "web.resume": "Continuar", "web.fullscreen": "Pantalla completa", "web.settings": "Ajustes", "web.language": "Idioma", "web.theme": "Diseño", "web.sound": "Sonido", "web.volume": "Volumen", "web.search": "Buscar…", "web.no_results": "Sin resultados", "web.reset_scores": "Borrar récords", "web.reset_confirm": "¿Borrar de verdad todos los récords?", "web.capture": "Haz clic para controlar con el ratón", "web.error": "Error del juego - ver consola (F12)", "web.wiki_search": "Buscar en la wiki…", "web.games_count": "{n} juegos" },
    pt: { "web.pause": "Pausa", "web.resume": "Continuar", "web.fullscreen": "Ecrã inteiro", "web.settings": "Definições", "web.language": "Idioma", "web.theme": "Design", "web.sound": "Som", "web.volume": "Volume", "web.search": "Pesquisar…", "web.no_results": "Sem resultados", "web.reset_scores": "Apagar recordes", "web.reset_confirm": "Apagar mesmo todos os recordes?", "web.capture": "Clica para controlar com o rato", "web.error": "Erro no jogo - ver consola (F12)", "web.wiki_search": "Pesquisar na wiki…", "web.games_count": "{n} jogos" },
    pl: { "web.pause": "Pauza", "web.resume": "Wznów", "web.fullscreen": "Pełny ekran", "web.settings": "Ustawienia", "web.language": "Język", "web.theme": "Wygląd", "web.sound": "Dźwięk", "web.volume": "Głośność", "web.search": "Szukaj…", "web.no_results": "Brak wyników", "web.reset_scores": "Usuń rekordy", "web.reset_confirm": "Na pewno usunąć wszystkie rekordy?", "web.capture": "Kliknij, aby sterować myszą", "web.error": "Błąd gry - zobacz konsolę (F12)", "web.wiki_search": "Szukaj w wiki…", "web.games_count": "{n} gier" },
    tr: { "web.pause": "Duraklat", "web.resume": "Devam", "web.fullscreen": "Tam ekran", "web.settings": "Ayarlar", "web.language": "Dil", "web.theme": "Tasarım", "web.sound": "Ses", "web.volume": "Ses düzeyi", "web.search": "Ara…", "web.no_results": "Sonuç yok", "web.reset_scores": "Rekorları sil", "web.reset_confirm": "Tüm rekorlar gerçekten silinsin mi?", "web.capture": "Fareyle kontrol için tıkla", "web.error": "Oyun hatası - konsola bak (F12)", "web.wiki_search": "Wikide ara…", "web.games_count": "{n} oyun" },
    da: { "web.pause": "Pause", "web.resume": "Fortsæt", "web.fullscreen": "Fuld skærm", "web.settings": "Indstillinger", "web.language": "Sprog", "web.theme": "Design", "web.sound": "Lyd", "web.volume": "Lydstyrke", "web.search": "Søg…", "web.no_results": "Ingen resultater", "web.reset_scores": "Slet highscores", "web.reset_confirm": "Vil du virkelig slette alle highscores?", "web.capture": "Klik for at styre med musen", "web.error": "Spilfejl - se konsollen (F12)", "web.wiki_search": "Søg i wikien…", "web.games_count": "{n} spil" },
    no: { "web.pause": "Pause", "web.resume": "Fortsett", "web.fullscreen": "Fullskjerm", "web.settings": "Innstillinger", "web.language": "Språk", "web.theme": "Design", "web.sound": "Lyd", "web.volume": "Volum", "web.search": "Søk…", "web.no_results": "Ingen treff", "web.reset_scores": "Slett rekorder", "web.reset_confirm": "Vil du virkelig slette alle rekorder?", "web.capture": "Klikk for å styre med musen", "web.error": "Spillfeil - se konsollen (F12)", "web.wiki_search": "Søk i wikien…", "web.games_count": "{n} spill" },
    sv: { "web.pause": "Paus", "web.resume": "Fortsätt", "web.fullscreen": "Helskärm", "web.settings": "Inställningar", "web.language": "Språk", "web.theme": "Design", "web.sound": "Ljud", "web.volume": "Volym", "web.search": "Sök…", "web.no_results": "Inga träffar", "web.reset_scores": "Radera rekord", "web.reset_confirm": "Vill du verkligen radera alla rekord?", "web.capture": "Klicka för att styra med musen", "web.error": "Spelfel - se konsolen (F12)", "web.wiki_search": "Sök i wikin…", "web.games_count": "{n} spel" },
    fi: { "web.pause": "Tauko", "web.resume": "Jatka", "web.fullscreen": "Koko näyttö", "web.settings": "Asetukset", "web.language": "Kieli", "web.theme": "Ulkoasu", "web.sound": "Ääni", "web.volume": "Äänenvoimakkuus", "web.search": "Hae…", "web.no_results": "Ei tuloksia", "web.reset_scores": "Poista ennätykset", "web.reset_confirm": "Poistetaanko todella kaikki ennätykset?", "web.capture": "Napsauta ohjataksesi hiirellä", "web.error": "Pelivirhe - katso konsoli (F12)", "web.wiki_search": "Hae wikistä…", "web.games_count": "{n} peliä" },
    cs: { "web.pause": "Pauza", "web.resume": "Pokračovat", "web.fullscreen": "Celá obrazovka", "web.settings": "Nastavení", "web.language": "Jazyk", "web.theme": "Vzhled", "web.sound": "Zvuk", "web.volume": "Hlasitost", "web.search": "Hledat…", "web.no_results": "Žádné výsledky", "web.reset_scores": "Smazat rekordy", "web.reset_confirm": "Opravdu smazat všechny rekordy?", "web.capture": "Klikni pro ovládání myší", "web.error": "Chyba hry - viz konzole (F12)", "web.wiki_search": "Hledat ve wiki…", "web.games_count": "{n} her" },
    sl: { "web.pause": "Premor", "web.resume": "Nadaljuj", "web.fullscreen": "Celozaslonsko", "web.settings": "Nastavitve", "web.language": "Jezik", "web.theme": "Videz", "web.sound": "Zvok", "web.volume": "Glasnost", "web.search": "Išči…", "web.no_results": "Ni zadetkov", "web.reset_scores": "Izbriši rekorde", "web.reset_confirm": "Res izbrišem vse rekorde?", "web.capture": "Klikni za upravljanje z miško", "web.error": "Napaka igre - glej konzolo (F12)", "web.wiki_search": "Išči po wikiju…", "web.games_count": "{n} iger" },
    hr: { "web.pause": "Pauza", "web.resume": "Nastavi", "web.fullscreen": "Cijeli zaslon", "web.settings": "Postavke", "web.language": "Jezik", "web.theme": "Izgled", "web.sound": "Zvuk", "web.volume": "Glasnoća", "web.search": "Traži…", "web.no_results": "Nema rezultata", "web.reset_scores": "Obriši rekorde", "web.reset_confirm": "Stvarno obrisati sve rekorde?", "web.capture": "Klikni za upravljanje mišem", "web.error": "Greška u igri - vidi konzolu (F12)", "web.wiki_search": "Pretraži wiki…", "web.games_count": "{n} igara" },
  };

  // Web-eigene Spieltexte in den übrigen Sprachen (Deutsch/Englisch liefern die
  // Spiele selbst per PG.addStrings) - ersetzen Python-Texte, die noch von
  // "Spieler 2" bzw. "zu zweit" sprechen.
  const GAME_STR = {
    fr: { "web.connect4.subtitle": "Quatre en ligne - contre l'IA", "web.reversi.subtitle": "Encercler et retourner les pions - contre l'IA", "web.reversi.pass_ai": "L'IA doit passer", "web.reversi.pass_you": "Tu dois passer", "web.snake.boost_hint": "Boost en jeu :  maintenir Espace/Maj ou Entrée", "web.muehle.subtitle": "Jeu du moulin – contre l'IA", "web.tanks.controls_hint": "WASD ou flèches = rouler   -   Espace ou Entrée = tirer" },
    es: { "web.connect4.subtitle": "Cuatro en raya - contra la IA", "web.reversi.subtitle": "Encierra y voltea fichas - contra la IA", "web.reversi.pass_ai": "La IA tiene que pasar", "web.reversi.pass_you": "Tienes que pasar", "web.snake.boost_hint": "Turbo en juego:  mantén Espacio/Shift o Intro", "web.muehle.subtitle": "Molino – contra la IA", "web.tanks.controls_hint": "WASD o flechas = conducir   -   Espacio o Intro = disparar" },
    pt: { "web.connect4.subtitle": "Quatro em linha - contra a IA", "web.reversi.subtitle": "Cercar e virar peças - contra a IA", "web.reversi.pass_ai": "A IA tem de passar", "web.reversi.pass_you": "Tens de passar", "web.snake.boost_hint": "Boost no jogo:  mantém Espaço/Shift ou Enter", "web.muehle.subtitle": "Moinho – contra a IA", "web.tanks.controls_hint": "WASD ou setas = conduzir   -   Espaço ou Enter = disparar" },
    pl: { "web.connect4.subtitle": "Cztery w rzędzie - przeciwko SI", "web.reversi.subtitle": "Otaczaj i odwracaj pionki - przeciwko SI", "web.reversi.pass_ai": "SI musi spasować", "web.reversi.pass_you": "Musisz spasować", "web.snake.boost_hint": "Boost w grze:  przytrzymaj Spację/Shift lub Enter", "web.muehle.subtitle": "Młynek – przeciwko SI", "web.tanks.controls_hint": "WASD lub strzałki = jazda   -   Spacja lub Enter = strzał" },
    tr: { "web.connect4.subtitle": "Dörtlü sıra - yapay zekâya karşı", "web.reversi.subtitle": "Taşları kuşat ve çevir - yapay zekâya karşı", "web.reversi.pass_ai": "Yapay zekâ pas geçmeli", "web.reversi.pass_you": "Pas geçmelisin", "web.snake.boost_hint": "Oyunda hız:  Boşluk/Shift veya Enter'a basılı tut", "web.muehle.subtitle": "Dokuz Taş – yapay zekâya karşı", "web.tanks.controls_hint": "WASD veya oklar = sür   -   Boşluk veya Enter = ateş" },
    da: { "web.connect4.subtitle": "Fire på stribe - mod AI'en", "web.reversi.subtitle": "Indeslut og vend brikker - mod AI'en", "web.reversi.pass_ai": "AI'en må melde pas", "web.reversi.pass_you": "Du må melde pas", "web.snake.boost_hint": "Boost i spillet:  hold Mellemrum/Shift eller Enter", "web.muehle.subtitle": "Mølle – mod AI'en", "web.tanks.controls_hint": "WASD eller pile = kør   -   Mellemrum eller Enter = skyd" },
    no: { "web.connect4.subtitle": "Fire på rad - mot KI-en", "web.reversi.subtitle": "Omring og snu brikker - mot KI-en", "web.reversi.pass_ai": "KI-en må stå over", "web.reversi.pass_you": "Du må stå over", "web.snake.boost_hint": "Boost i spillet:  hold Mellomrom/Shift eller Enter", "web.muehle.subtitle": "Mølle – mot KI-en", "web.tanks.controls_hint": "WASD eller piler = kjør   -   Mellomrom eller Enter = skyt" },
    sv: { "web.connect4.subtitle": "Fyra i rad - mot AI:n", "web.reversi.subtitle": "Ringa in och vänd brickor - mot AI:n", "web.reversi.pass_ai": "AI:n måste passa", "web.reversi.pass_you": "Du måste passa", "web.snake.boost_hint": "Boost i spelet:  håll ned Mellanslag/Shift eller Enter", "web.muehle.subtitle": "Kvarnspel – mot AI:n", "web.tanks.controls_hint": "WASD eller pilar = kör   -   Mellanslag eller Enter = skjut" },
    fi: { "web.connect4.subtitle": "Neljän suora - tekoälyä vastaan", "web.reversi.subtitle": "Saarra ja käännä nappulat - tekoälyä vastaan", "web.reversi.pass_ai": "Tekoälyn on passattava", "web.reversi.pass_you": "Sinun on passattava", "web.snake.boost_hint": "Boost pelissä:  pidä Välilyönti/Shift tai Enter pohjassa", "web.muehle.subtitle": "Mylly – tekoälyä vastaan", "web.tanks.controls_hint": "WASD tai nuolet = aja   -   Välilyönti tai Enter = ammu" },
    cs: { "web.connect4.subtitle": "Čtyři v řadě - proti AI", "web.reversi.subtitle": "Obklič a otoč kameny - proti AI", "web.reversi.pass_ai": "AI musí passovat", "web.reversi.pass_you": "Musíš passovat", "web.snake.boost_hint": "Boost ve hře:  drž Mezerník/Shift nebo Enter", "web.muehle.subtitle": "Mlýn – proti AI", "web.tanks.controls_hint": "WASD nebo šipky = jízda   -   Mezerník nebo Enter = palba" },
    sl: { "web.connect4.subtitle": "Štiri v vrsto - proti UI", "web.reversi.subtitle": "Obkoli in obrni žetone - proti UI", "web.reversi.pass_ai": "UI mora izpustiti potezo", "web.reversi.pass_you": "Moraš izpustiti potezo", "web.snake.boost_hint": "Pospešek v igri:  drži Preslednico/Shift ali Enter", "web.muehle.subtitle": "Mlin – proti UI", "web.tanks.controls_hint": "WASD ali puščice = vožnja   -   Preslednica ali Enter = strel" },
    hr: { "web.connect4.subtitle": "Četiri u nizu - protiv UI", "web.reversi.subtitle": "Okruži i okreni žetone - protiv UI", "web.reversi.pass_ai": "UI mora preskočiti potez", "web.reversi.pass_you": "Moraš preskočiti potez", "web.snake.boost_hint": "Boost u igri:  drži Razmaknicu/Shift ili Enter", "web.muehle.subtitle": "Mlin – protiv UI", "web.tanks.controls_hint": "WASD ili strelice = vožnja   -   Razmaknica ili Enter = pucaj" },
  };
  for (const code in GAME_STR) Object.assign(WEB_STR[code], GAME_STR[code]);

  function guessLang() {
    const nav = (navigator.languages || [navigator.language || "de"]).map((l) => String(l).slice(0, 2).toLowerCase());
    for (const code of nav) {
      if (code === "nb" || code === "nn") return "no";
      if (LANG_CODES.includes(code)) return code;
    }
    return "de";
  }

  PG.lang = PG.store.get("lang", null);
  if (!LANG_CODES.includes(PG.lang)) PG.lang = guessLang();

  PG.setLang = function (code) {
    if (!LANG_CODES.includes(code)) code = "de";
    PG.lang = code;
    PG.store.set("lang", code);
    document.documentElement.lang = code;
  };

  /** Python-str.format-Nachbau für {name}, {name:,}, {name:.1f}, {name:>3} ... */
  PG.format = function (s, params) {
    if (!params || typeof s !== "string") return s;
    return s
      .replace(/\{\{/g, "")
      .replace(/\}\}/g, "")
      .replace(/\{([A-Za-z_0-9]+)(?::([^}]*))?\}/g, (m, key, spec) => {
        if (!(key in params)) return m;
        return spec ? formatSpec(params[key], spec) : String(params[key]);
      })
      .replace(//g, "{")
      .replace(//g, "}");
  };

  // Format-Spezifikation wie Python: [[fill]align][sign][#][0][width][,|_][.precision][type]
  const SPEC_RE = /^(?:(.)?([<>^=]))?([+\- ])?(#)?(0)?(\d+)?([,_])?(?:\.(\d+))?([sdfF%])?$/;

  /**
   * toFixed für Zahlen >= 0 mit Python-Rundung: bei exaktem Gleichstand
   * (z.B. 2.5 -> "2", 0.125 -> "0.12") auf die gerade Ziffer statt aufrunden.
   */
  function fixedAbs(a, prec) {
    const s = a.toFixed(prec);
    if (!isFinite(a) || a >= 1e21 || prec > 70) return s;
    const long = a.toFixed(prec + 25);
    if (!/50{24}$/.test(long)) return s;
    const cut = long.slice(0, -25).replace(/\.$/, "");
    return Number(cut[cut.length - 1]) % 2 === 0 ? cut : s;
  }

  /** Ein Platzhalter mit Format-Spezifikation (Teilmenge von format(v, spec)). */
  function formatSpec(v, spec) {
    const m = SPEC_RE.exec(spec);
    if (!m) return String(v);
    let [, fill, align, sign, , zero, width, group, prec, type] = m;
    const numType = type && type !== "s";
    if (numType && typeof v !== "number" && v !== "" && v != null && !isNaN(Number(v))) v = Number(v);
    const isNum = typeof v === "number";
    let body, signStr = "";
    if (isNum && type !== "s") {
      const a = Math.abs(v);
      if (type === "%") body = fixedAbs(a * 100, prec != null ? +prec : 6);
      else if (type === "f" || type === "F") body = fixedAbs(a, prec != null ? +prec : 6);
      else if (type === "d") body = String(Math.trunc(a));
      else if (prec != null) body = fixedAbs(a, +prec);
      else body = String(a);
      if (group) {
        const parts = body.split(".");
        parts[0] = parts[0].replace(/\B(?=(\d{3})+(?!\d))/g, group);
        body = parts.join(".");
      }
      if (type === "%") body += "%";
      signStr = v < 0 || Object.is(v, -0) ? "-" : sign === "+" ? "+" : sign === " " ? " " : "";
      if (body === "0" && signStr === "-" && type === "d") signStr = "";
    } else {
      body = String(v);
      if (prec != null) body = body.slice(0, +prec);
    }
    if (zero && !align) {
      fill = "0";
      align = isNum ? "=" : "<";
    }
    fill = fill || " ";
    align = align || (isNum ? ">" : "<");
    const pad = (width ? +width : 0) - signStr.length - body.length;
    if (pad <= 0) return signStr + body;
    if (align === "=") return signStr + fill.repeat(pad) + body;
    if (align === ">") return fill.repeat(pad) + signStr + body;
    if (align === "^") return fill.repeat(pad >> 1) + signStr + body + fill.repeat(pad - (pad >> 1));
    return signStr + body + fill.repeat(pad);
  }

  /** Übersetzt 'key' in die aktive Sprache (Fallback Deutsch, sonst der Key). */
  PG.t = function (key, params) {
    const L = window.PG_LANG || {};
    let s = L[PG.lang] && L[PG.lang][key];
    if (s == null) s = WEB_STR[PG.lang] && WEB_STR[PG.lang][key];
    if (s == null) s = L.de && L.de[key];
    if (s == null) s = WEB_STR.en[key];
    if (s == null) return key;
    return params ? PG.format(s, params) : s;
  };
  /** true, wenn es den Schlüssel gibt */
  PG.hasT = function (key) {
    const L = window.PG_LANG || {};
    return !!((L[PG.lang] && key in L[PG.lang]) || (L.de && key in L.de) || key in WEB_STR.en);
  };
  /** Einmalig eigene Texte ergänzen: PG.addStrings({de:{...}, en:{...}}) */
  PG.addStrings = function (table) {
    for (const code in table) {
      WEB_STR[code] = Object.assign(WEB_STR[code] || {}, table[code]);
    }
  };
  /** Wählt aus {de:"..", en:"..", default:".."} den passenden Text. */
  PG.pick = function (obj) {
    if (obj == null || typeof obj === "string") return obj;
    return obj[PG.lang] != null ? obj[PG.lang] : obj.default != null ? obj.default : obj.en != null ? obj.en : obj.de;
  };

  // ------------------------------------------------------------ Einstellungen
  // Standard-Tastenbelegung wie settings.DEFAULT_CONTROLS - im Einzelspieler
  // steuern WASD+Leertaste UND Pfeile+Enter die Figur.
  PG.CONTROLS = {
    p1: { up: "w", down: "s", left: "a", right: "d", action: "space" },
    p2: { up: "Up", down: "Down", left: "Left", right: "Right", action: "Return" },
  };

  PG.THEME_NAMES = [
    ["v42", "UI v4.2"], ["v41", "UI v4.1"], ["v411", "UI v4.1.1"], ["v412", "UI v4.1.2"], ["v413", "UI v4.1.3"],
    ["v414", "UI v4.1.4"], ["modern", "UI v4"], ["classic", "UI v3 (Classic)"], ["v2", "UI v2"], ["v1", "UI v1"],
  ];

  const GLOBAL_DEFAULTS = { theme: "v42", sound: true, volume: 0.6, haptik: false };

  const clone = (o) => JSON.parse(JSON.stringify(o));

  PG.settings = {
    data: Object.assign(clone(GLOBAL_DEFAULTS), PG.store.get("settings", {})),
    /** Abschnitt eines Spiels mit Standardwerten auffüllen und zurückgeben. */
    section(name, defaults) {
      const cur = this.data[name];
      const merged = Object.assign(clone(defaults || {}), cur && typeof cur === "object" ? cur : {});
      this.data[name] = merged;
      return merged;
    },
    save() {
      PG.store.set("settings", this.data);
    },
  };
  if (!PG.THEME_NAMES.some((t) => t[0] === PG.settings.data.theme)) PG.settings.data.theme = "v42";

  // -------------------------------------------------------------- Highscores
  PG.highscore = {
    all() {
      const d = PG.store.get("highscores", {});
      return d && typeof d === "object" ? d : {};
    },
    get(key) {
      return Number(this.all()[key]) || 0;
    },
    /** gibt [highscore, istNeuerRekord] zurück (wie highscore.update_highscore) */
    update(key, score) {
      const d = this.all();
      const old = Number(d[key]) || 0;
      score = Math.floor(Number(score) || 0);
      if (score > old) {
        d[key] = score;
        PG.store.set("highscores", d);
        return [score, true];
      }
      return [old, false];
    },
    /** direkt setzen (z.B. Spiele mit eigenem Highscore-Konzept) */
    set(key, score) {
      const d = this.all();
      d[key] = Math.floor(score);
      PG.store.set("highscores", d);
    },
    clear() {
      PG.store.set("highscores", {});
    },
  };

  // ---------------------------------------------------------------- Statistik
  PG.stats = {
    _data: null,
    _dirty: false,
    _last: 0,
    get data() {
      if (!this._data) {
        const d = PG.store.get("stats", {});
        this._data = d && typeof d === "object" ? d : {};
      }
      return this._data;
    },
    entry(key) {
      const d = this.data;
      if (!d[key]) d[key] = { played: 0, wins: 0, losses: 0, time: 0, records: 0 };
      return d[key];
    },
    gameStarted(key) { this.entry(key).played++; this._dirty = true; },
    recordResult(key, won) { this.entry(key)[won ? "wins" : "losses"]++; this._dirty = true; this.flush(); },
    recordBroken(key) { this.entry(key).records++; this._dirty = true; },
    addPlaytime(key, dt) { this.entry(key).time += dt; this._dirty = true; },
    event(id, value) {
      const ev = this.data._events || (this.data._events = {});
      const old = ev[id];
      ev[id] = value == null ? (old || 0) + 1 : Math.max(Number(old) || 0, Number(value) || 0);
      this._dirty = true;
    },
    maybeFlush() {
      const now = performance.now();
      if (this._dirty && now - this._last > 5000) this.flush();
    },
    flush() {
      if (!this._dirty) return;
      this._last = performance.now();
      this._dirty = false;
      PG.store.set("stats", this.data);
    },
  };

  // ------------------------------------------------------------------ Datei
  /** Bietet Text als Download an (z.B. Export von Minigolf-Bahnen). */
  PG.downloadText = function (filename, text, mime) {
    const blob = new Blob([text], { type: mime || "application/json" });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    setTimeout(() => {
      URL.revokeObjectURL(a.href);
      a.remove();
    }, 1000);
  };

  /** Öffnet einen Dateiauswahl-Dialog; callback(text, filename) */
  PG.pickTextFile = function (accept, callback) {
    const input = document.createElement("input");
    input.type = "file";
    if (accept) input.accept = accept;
    input.onchange = () => {
      const f = input.files && input.files[0];
      if (!f) return;
      const reader = new FileReader();
      reader.onload = () => callback(String(reader.result), f.name);
      reader.readAsText(f);
    };
    input.click();
  };
})();
