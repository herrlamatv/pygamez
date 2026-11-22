/*
 * lamacrash_music.js - Spannungsmusik von Lama Crash (Port von games/lamacrash_music.py)
 * =====================================================================================
 * Je Hoehenwelt eine Stufe, jede schneller und hoeher als die vorige
 * (0 Wiese … 4 Weltall, 5 = Fanfare der Mondlandung). Laeuft ueber die Musik-API
 * von audio.js (PG.audio.noteSamples / renderVoice / music - vier Stimmen,
 * vorab gerendert, in Dauerschleife).
 *
 * Ein Stueck in sechs Stufen: Akkordfolge I - bVI - IV - V (ein Akkord je Takt,
 * der Bass faellt), Motiv aus fuenf Toenen je Takt, dessen Spitzenton von Takt
 * zu Takt steigt; erst die Fanfare (I - bVI - bVII - I) fuehrt die Linie bis zum
 * Grundton. Jede Stufe steht einen Ganzton hoeher, ist schneller und dichter:
 *
 *   Stufe  Welt       bpm  Tonart           Takte  Satz
 *   0      Wiese       96  A-Dur              4    Herzschlag, Zupfbass, Pendel in Vierteln
 *   1      Wolken     108  H-Dur              4    Huepfbass 3-3-2, Arpeggio in Achteln
 *   2      Gewitter   120  cis-Moll           4    Donner, Galopp-Bass, Tremolo, verm. V
 *   3      Nordlicht  134  Dis-Dur (maj7)     8    Glitzer-Arpeggio ueber zwei Oktaven
 *   4      Weltall    150  f-Moll (dorisch)   8    16tel-Pumpbass, 16tel-Arpeggio
 *   5      Mond       112  G-Dur              4    Fanfare, Marschbass, Wirbel
 *
 * Die Tabellen sind 1:1 aus Python (kein Zufall). Schnittstelle wie dort:
 *   STAGES, stageOf(m100), bpm(stage), beatSeconds(stage), compose(stage),
 *   voices(stage), prerenderStep(), ready(stage), squeak(m100), clear().
 *
 * Speicher (Web): alle sechs Stufen als Float32 waeren ~48 MB (48 kHz). Darum
 * wird nur gerendert, was gebraucht wird: prerenderStep(want) rendert die
 * gewuenschte Stufe sofort (sie wird jetzt gebraucht), danach die naechsthoehere
 * (beim Weltall die Fanfare) in Haeppchen von SLICE Samples je Aufruf - so darf
 * das Spiel auch im Flug vorbereiten, ohne dass ein Bild haengt. Hoechstens
 * KEEP Stufen bleiben liegen, die am weitesten entfernte faellt weg.
 * Wie in Python merkt sich jede gerenderte Stufe ihre Abtastrate: eine mit
 * anderer Rate (PG.audio.musicRate()) gerenderte Stufe gilt als fehlend.
 *
 * Nichts hier haengt vom Platzpunkt ab - nur vom aktuellen Multiplikator.
 */
(function () {
  "use strict";
  const L = PG.lamaCrashLogic;

  const STAGES = 6;
  const BPM = [96, 108, 120, 134, 150, 112];
  // hoechstens so viele gerenderte Stufen im Speicher (Web: Float32 je Stimme)
  const KEEP = 3;
  const store = new Map(); // Stufe -> {rate, voices: [[Float32Array, Lautstaerke], ...], used}
  let useN = 0;

  // Grundtoene je Takt in Halbtoenen ueber dem Grundton der Stufe:
  // 0 = Thema I - bVI - IV - V (fallender Bass), 1 = Fanfare I - bVI - bVII - I
  const ROOTS = [[0, -4, -7, -5], [0, -4, -2, 0]];

  // Modus -> [Akkorde fuers Arpeggio: 4 Toene je Takt, Motiv-Zellen je Takt]
  const MODES = {
    // Dur mit geborgter bVI: C - As - F - G
    dur: [[[0, 4, 7, 12], [0, 3, 8, 12], [0, 5, 9, 12], [-1, 2, 7, 11]],
      [[12, 16, 19, 21, 19], [12, 15, 20, 22, 20], [12, 17, 21, 22, 21], [14, 19, 23, 24, 23]]],
    // Moll, die V als verminderter Septakkord (G7b9 ohne Grundton): cm - As - fm - G
    moll: [[[0, 3, 7, 12], [0, 3, 8, 12], [0, 5, 8, 12], [-1, 2, 5, 8]],
      [[12, 15, 19, 20, 19], [12, 15, 20, 22, 20], [12, 17, 20, 22, 20], [14, 17, 23, 24, 23]]],
    // Traum: Cmaj9 - Asmaj7 - Fmaj7 (lydische Quarte) - Gsus2
    traum: [[[4, 7, 11, 14], [3, 7, 8, 12], [5, 9, 12, 16], [2, 7, 9, 14]],
      [[12, 16, 19, 21, 19], [12, 15, 20, 22, 20], [12, 17, 21, 23, 21], [14, 19, 23, 24, 23]]],
    // Dorisch: cm (mit grosser Sexte) - As - F - G
    dorisch: [[[0, 3, 7, 12], [0, 3, 8, 12], [0, 5, 9, 12], [-1, 2, 7, 11]],
      [[12, 15, 19, 21, 19], [12, 15, 20, 22, 20], [12, 17, 21, 22, 21], [14, 19, 23, 24, 23]]],
    // Fanfare: C - As - B - C, die Linie steigt bis zum Grundton (19, 20, 22, 24)
    fanfare: [[[0, 4, 7, 12], [0, 3, 8, 12], [2, 5, 10, 14], [4, 7, 12, 16]],
      [[12, 16, 19, 21, 19], [12, 15, 20, 22, 20], [14, 17, 22, 24, 22], [16, 19, 24]]],
  };

  // Schlagzeug-Zeichen -> [Welle, Frequenz, Lautstaerke, laengste Dauer in s, decay];
  // ein Schlag klingt hoechstens bis zum naechsten (die Stimme ist monophon).
  const KIT = {
    K: ["kick", 50.0, 0.36, 0.22, 0.0],
    k: ["kick", 50.0, 0.24, 0.18, 0.0],
    S: ["snare", 190.0, 0.22, 0.16, 0.0],
    s: ["snare", 190.0, 0.12, 0.12, 0.0],
    H: ["hat", 8000.0, 0.09, 0.05, 0.0],
    h: ["hat", 8000.0, 0.05, 0.04, 0.0],
    X: ["noise", 1000.0, 0.09, 0.50, 5.0], // Becken / Donner
  };

  // Rhythmus der Motiv-Zelle als [Position, Laenge] in 16teln
  const R8 = [[0, 2], [2, 2], [4, 6], [10, 2], [12, 4]]; // gerade Achtel
  const RF = [[0, 3], [3, 1], [4, 6], [10, 2], [12, 4]]; // Fanfare: punktiert
  const RF4 = [[0, 2], [2, 2], [4, 12]]; // Fanfare: Schlusston
  // Tonfolge der Zelle: Thema bzw. Antwort (2. Haelfte der 8-Takt-Stufen)
  const ORDER = [0, 1, 2, 3, 4];
  const ANSWER = [0, 3, 2, 1, 4];

  // 16tel-Pumpbass (Weltall): Grundton, auf jeder zweiten Achtel die Oktave
  const PUMP = [];
  for (let i = 0; i < 16; i++) PUMP.push([i, 1, i % 4 === 2 ? 12 : 0]);

  // Klangfarben, die mehrere Stufen teilen (gleiche Noten -> Ton-Cache)
  const SOFT_LEAD = { wave: "triangle", vol: 0.22, decay: 2.5, short: 0.18, long: 0.40, hold: 0.40 };
  const HARP = { wave: "triangle", vol: 0.16, decay: 4.0, cap: 0.20 };
  const withExtra = (base, extra) => Object.assign({}, base, extra);

  // Je Stufe: Grundton (MIDI), Modus, Akkordfolge, Takte, Schlagzeug (16 Zeichen je
  // Takt; "drums" reihum, "fill" im letzten Takt), Bass/Arpeggio/Melodie.
  const PLAN = [
    // 0 Wiese - ruhig, aber erwartungsvoll: Herzschlag, Pendel in Vierteln
    {
      key: 57, mode: "dur", prog: 0, bars: 4,
      drums: ["k.h...h.k.h...h."], fill: "k.h...h.k.h.h.hH",
      bass: { wave: "triangle", vol: 0.34, decay: 3.0, cap: 0.30, oct: -12, pat: [[[0, 8, 0], [8, 8, 0]]] },
      arp: withExtra(HARP, { step: 4, pat: [[2, 1, 2, 1]] }),
      lead: withExtra(SOFT_LEAD, { rhythm: [R8], order: [ORDER] }),
    },
    // 1 Wolken - leicht, huepfend
    {
      key: 59, mode: "dur", prog: 0, bars: 4,
      drums: ["K.h.s.h.k.h.s.h."], fill: "K.h.s.h.k.h.s.ss",
      bass: { wave: "triangle", vol: 0.30, decay: 4.0, cap: 0.24, oct: -12,
        pat: [[[0, 3, 0], [3, 3, 0], [6, 2, 12], [8, 3, 0], [11, 3, 0], [14, 2, 12]]] },
      arp: withExtra(HARP, { step: 2, pat: [[0, 2, 1, 3, 0, 2, 1, 3]] }),
      lead: withExtra(SOFT_LEAD, { rhythm: [R8], order: [ORDER] }),
    },
    // 2 Gewitter - dunkel: Moll, Galopp, Donner auf der Eins
    {
      key: 61, mode: "moll", prog: 0, bars: 4,
      drums: ["X...S.h.K.hKS.hh", "K.hhS.h.K.hKS.hh", "K.hhS.h.K.hKS.hh", "K.hhS.h.K.hKS.hh"],
      fill: "K.hhS.h.K.hKSSSS",
      bass: { wave: "saw", vol: 0.22, decay: 7.0, cap: 0.20, oct: -12,
        pat: [[[0, 2, 0], [2, 1, 0], [3, 1, 0], [4, 2, 0], [6, 1, 0], [7, 1, 0], [8, 2, 0], [10, 1, 0], [11, 1, 0],
          [12, 2, 0], [14, 1, 12], [15, 1, 12]]] },
      arp: { wave: "square", vol: 0.07, decay: 9.0, cap: 0.11, step: 1, pat: [[0, 2, 1, 2, 0, 2, 1, 2, 0, 2, 1, 2, 0, 2, 1, 2]] },
      lead: { wave: "square", vol: 0.12, decay: 1.5, short: 0.18, long: 0.34, hold: 0.34, rhythm: [R8], order: [ORDER] },
    },
    // 3 Nordlicht - traeumerisch schimmernd
    {
      key: 63, mode: "traum", prog: 0, bars: 8,
      drums: ["X...shHhkhHhshHh", "khHhshHhkhHhshHh", "khHhshHhkhHhshHh", "khHhshHhkhHhshHh"],
      fill: "khHhshHhkhHhsSsS",
      bass: { wave: "triangle", vol: 0.24, decay: 4.0, cap: 0.20, oct: -12,
        pat: [[[0, 2, 0], [2, 2, 0], [4, 2, 12], [6, 2, 0], [8, 2, 0], [10, 2, 0], [12, 2, 12], [14, 2, 0]]] },
      arp: { wave: "sine", vol: 0.13, decay: 6.0, cap: 0.09, step: 1,
        pat: [[0, 1, 2, 3, 4, 5, 6, 7, 6, 5, 4, 3, 2, 1, 2, 3], [7, 6, 5, 4, 3, 2, 1, 0, 1, 2, 3, 4, 5, 6, 5, 4]] },
      lead: withExtra(SOFT_LEAD, { rhythm: [R8], order: [ORDER, ANSWER] }),
    },
    // 4 Weltall - treibend: Pumpbass und Arpeggio in 16teln
    {
      key: 65, mode: "dorisch", prog: 0, bars: 8,
      drums: ["X...S.hKK.HhS.hK", "K.HhS.hKK.HhS.hK", "K.HhS.hKK.HhS.hK", "K.HhS.hKK.HhS.hK"],
      fill: "K.HhS.hKK.SSSSSS",
      bass: { wave: "saw", vol: 0.24, decay: 10.0, cap: 0.09, oct: -24, pat: [PUMP] },
      arp: { wave: "square", vol: 0.10, decay: 10.0, cap: 0.09, step: 1,
        pat: [[0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3], [0, 1, 2, 3, 4, 5, 6, 7, 4, 5, 6, 7, 0, 1, 2, 3]] },
      lead: { wave: "square", vol: 0.13, decay: 2.5, short: 0.18, long: 0.34, hold: 0.34, rhythm: [R8], order: [ORDER, ANSWER] },
    },
    // 5 Mond - Fanfare (Schleife), die Linie landet auf dem Grundton
    {
      key: 55, mode: "fanfare", prog: 1, bars: 4,
      drums: ["X...S.s.K.s.S.ss", "K.s.S.s.K.s.S.ss", "K.s.S.s.K.s.S.ss", "K.s.S.s.ssssSSSS"],
      fill: "",
      bass: { wave: "saw", vol: 0.22, decay: 3.0, cap: 0.26, oct: -12,
        pat: [[[0, 4, 0], [4, 4, 0], [8, 4, 0], [12, 4, 0]], [[0, 4, 0], [4, 4, 0], [8, 4, 0], [12, 4, 0]],
          [[0, 4, 0], [4, 4, 0], [8, 4, 0], [12, 4, 0]], [[0, 4, 0], [4, 4, 0], [8, 4, 0], [12, 4, -5]]] },
      arp: { wave: "square", vol: 0.08, decay: 5.0, cap: 0.14, step: 2, pat: [[0, 2, 1, 2, 0, 2, 1, 2]] },
      lead: { wave: "square", vol: 0.15, decay: 1.2, short: 0.18, long: 0.34, hold: 0.8, rhythm: [RF, RF, RF, RF4], order: [ORDER] },
    },
  ];

  // Lautstaerke der vier Stimmen (Schlagzeug, Bass, Arpeggio, Melodie)
  const VOICE_VOL = [0.8, 0.9, 0.8, 0.9];

  /** Frequenz eines MIDI-Tons (69 = a' = 440 Hz). */
  function midi(m) {
    return 440.0 * Math.pow(2.0, (m - 69) / 12.0);
  }
  function clampStage(stage) {
    return Math.max(0, Math.min(STAGES - 1, Math.trunc(stage)));
  }
  /** Musikstufe zum Multiplikator (0..4). */
  function stageOf(m100) {
    return Math.min(4, L.worldOf(m100));
  }
  function bpm(stage) {
    return BPM[clampStage(stage)];
  }
  function beatSeconds(stage) {
    return 60.0 / bpm(stage);
  }
  const fdiv = (a, b) => Math.floor(a / b);

  /** Noten der Schleife einer Stufe - ohne Audio (testbar). Vier monophone
   *  Stimmen (Schlagzeug, Bass, Arpeggio, Melodie), Noten als
   *  [start_s, freq, dauer, welle, lautstaerke, decay]. */
  function compose(stage) {
    const s = clampStage(stage);
    const p = PLAN[s];
    const six = beatSeconds(s) / 4.0; // 16tel in Sekunden
    const bars = p.bars;
    const key = p.key;
    const roots = ROOTS[p.prog];
    const [chords, cells] = MODES[p.mode];

    // Schlagzeug: erst alle Schlaege sammeln, dann klingt jeder bis zum naechsten
    const hits = [];
    for (let bar = 0; bar < bars; bar++) {
      let pat = p.drums[bar % p.drums.length];
      if (bar === bars - 1 && p.fill) pat = p.fill;
      for (let i = 0; i < 16; i++) if (pat[i] !== ".") hits.push([bar * 16 + i, pat[i]]);
    }
    const drums = [];
    for (let j = 0; j < hits.length; j++) {
      const [pos, c] = hits[j];
      const nxt = j + 1 < hits.length ? hits[j + 1][0] : bars * 16;
      const [wave, freq, vol, cap, decay] = KIT[c];
      drums.push([pos * six, freq, Math.min(cap, (nxt - pos) * six * 0.95), wave, vol, decay]);
    }

    const b = p.bass, a = p.arp, m = p.lead;
    const bass = [], arp = [], lead = [];
    for (let bar = 0; bar < bars; bar++) {
      const base = bar * 16;
      const half = fdiv(bar, 4);
      const root = key + b.oct + roots[bar % 4];
      for (const [pos, ln, iv] of b.pat[bar % b.pat.length]) {
        bass.push([(base + pos) * six, midi(root + iv), Math.min(b.cap, ln * six * 0.9), b.wave, b.vol, b.decay]);
      }
      const chord = chords[bar % 4];
      const pat = a.pat[half % a.pat.length];
      const step = a.step;
      for (let i = 0; i < pat.length; i++) {
        const tone = key + chord[pat[i] % 4] + 12 * fdiv(pat[i], 4);
        arp.push([(base + i * step) * six, midi(tone), Math.min(a.cap, step * six * 0.9), a.wave, a.vol, a.decay]);
      }
      const cell = cells[bar % 4];
      const rhythm = m.rhythm[bar % m.rhythm.length];
      const order = m.order[half % m.order.length];
      for (let j = 0; j < rhythm.length; j++) {
        const [pos, ln] = rhythm[j];
        const cap = ln <= 3 ? m.short : ln <= 6 ? m.long : m.hold;
        lead.push([(base + pos) * six, midi(key + cell[order[j]]), Math.min(cap, ln * six * 0.9), m.wave, m.vol, m.decay]);
      }
    }
    const all = [drums, bass, arp, lead];
    return { length: bars * 16 * six, bars, bpm: bpm(s), voices: all.map((notes, i) => ({ vol: VOICE_VOL[i], notes })) };
  }

  /** Rendern in Haeppchen: ein Auftrag je Stufe; slice() synthetisiert Noten, bis
   *  budget Samples erreicht sind, und setzt fertige Stimmen zusammen (renderVoice).
   *  Liefert die fertigen Stimmen oder null (noch nicht fertig). */
  function newJob(stage, r) {
    return { stage, rate: r, song: compose(stage), vi: 0, ni: 0, notes: [], out: [] };
  }
  function slice(job, budget) {
    const A = PG.audio;
    const song = job.song;
    let spent = 0;
    while (job.vi < song.voices.length) {
      const v = song.voices[job.vi];
      while (job.ni < v.notes.length) {
        if (spent >= budget) return null;
        const n = v.notes[job.ni++];
        job.notes.push([n[0], A.noteSamples(n[1], n[2], n[3], n[4], n[5])]);
        spent += Math.max(1, Math.floor(job.rate * n[2]));
      }
      job.out.push([A.renderVoice(job.notes, song.length), v.vol]);
      spent += Math.floor(job.rate * song.length) >> 3; // Kopieren ist billig, zaehlt aber
      job.vi += 1;
      job.ni = 0;
      job.notes = [];
    }
    return job.out;
  }
  let job = null; // laufender Haeppchen-Auftrag (naechste Stufe)
  // Haeppchen: so viele Samples Noten-Synthese je Aufruf (~2-3 ms)
  const SLICE = 120000;

  function rate() {
    return PG.audio && PG.audio.musicRate ? PG.audio.musicRate() : 44100;
  }
  /** Stufe schon gerendert (mit der aktuellen Abtastrate)? */
  function ready(stage) {
    const e = store.get(stage);
    return !!e && e.rate === rate();
  }
  /** Gerenderte Stimmen der Stufe [[Float32Array, Lautstaerke], ...] oder null. */
  function voices(stage) {
    if (!ready(stage)) return null;
    const e = store.get(stage);
    e.used = ++useN;
    return e.voices;
  }
  /** Stufen, die zu want gebraucht werden: want, dann die naechste Welt (beim
   *  Weltall die Fanfare); want = 5 (Fanfare) braucht nur sich selbst. */
  function wanted(want) {
    const w = clampStage(want);
    if (w >= 5) return [5];
    return [w, w + 1];
  }
  /** Ueberzaehlige Stufen vergessen: zuerst die, die zu want am wenigsten passen
   *  (nicht gebraucht, am weitesten weg, am laengsten nicht benutzt). */
  function prune(keep) {
    while (store.size > KEEP) {
      let worst = null;
      let worstScore = -1;
      for (const [s, e] of store) {
        if (keep.includes(s)) continue;
        const score = Math.abs(s - keep[0]) * 1e9 - e.used;
        if (score > worstScore) {
          worstScore = score;
          worst = s;
        }
      }
      if (worst === null) break;
      store.delete(worst);
    }
  }
  function keep(s, r, v, need) {
    store.set(s, { rate: r, voices: v, used: ++useN });
    prune(need);
  }
  /** Arbeitet an der naechsten fehlenden Stufe fuer want; true, wenn etwas getan
   *  wurde. Fehlt want selbst, wird sie sofort ganz gerendert (sie wird jetzt
   *  gebraucht - je Aufruf hoechstens eine Stufe, wie in Python); die Stufe
   *  darueber entsteht in Haeppchen (SLICE Samples je Aufruf), damit auch ein
   *  Aufruf mitten im Flug kein Bild verschluckt. */
  function prerenderStep(want = 0) {
    const need = wanted(want);
    const r = rate();
    if (job && (job.rate !== r || !need.includes(job.stage) || ready(job.stage))) job = null;
    const first = need[0];
    if (!ready(first)) {
      const j = job && job.stage === first ? job : newJob(first, r);
      job = null;
      keep(first, r, slice(j, Infinity), need);
      return true;
    }
    for (const s of need.slice(1)) {
      if (ready(s)) continue;
      if (!job) job = newJob(s, r);
      const v = slice(job, SLICE);
      if (v) {
        keep(s, r, v, need);
        job = null;
      }
      return true;
    }
    return false;
  }
  /** Gerenderte Stufen (fuer Tests). */
  function rendered() {
    return [...store.keys()].sort((x, y) => x - y);
  }

  // Quietsch-Schwellen (m100): ab der k-ten klingt der Ton k Halbtoene ueber c''
  // (523 Hz); dicht im haeufigen Bereich x2..x10, oben weit. 24 Toene, Abstand
  // 3,0 s (x2) bis 0,5 s (Mond).
  const SQUEAK_AT = [200, 230, 265, 300, 350, 400, 470, 550, 650, 750, 870, 1000,
    1300, 1700, 2200, 3000, 4000, 5500, 7500, 10000, 30000, 100000, 1000000, 10000000];
  const SQUEAK_BASE = 72;

  /** Dehn-Quietschen ab x2: [Frequenz in Hz (2 Stellen), Abstand in s] oder null. */
  function squeak(m100) {
    if (m100 < SQUEAK_AT[0]) return null;
    let k = 0;
    for (let i = 0; i < SQUEAK_AT.length; i++) if (m100 >= SQUEAK_AT[i]) k = i;
    const top = SQUEAK_AT.length - 1;
    const gapMs = 500 + fdiv(2500 * (top - k), top);
    return [Number(midi(SQUEAK_BASE + k).toFixed(2)), gapMs / 1000.0];
  }

  function clear() {
    store.clear();
    job = null;
  }

  PG.lamaCrashMusic = {
    STAGES, KEEP, BPM, PLAN, SQUEAK_AT,
    stageOf, bpm, beatSeconds, compose, voices, prerenderStep, ready, squeak, clear, rendered, wanted,
  };
})();
