/*
 * audio.js - synthetisierte Soundeffekte (Nachbau von audio.py)
 * ==============================================================
 * Keine Audiodateien: die Effekte werden beim ersten Nutzer-Klick/Tastendruck
 * per WebAudio aus denselben Spezifikationen wie in audio.py erzeugt.
 *
 *   PG.audio.play("eat");                  // respektiert Sound an/aus + Lautstärke
 *   PG.audio.tone(440, 0.18, "sine", 0.35); // freier Ton (z.B. Simon)
 *   PG.audio.rumble(120);                  // Gamepad-Vibration (falls vorhanden)
 */
(function () {
  "use strict";

  const PG = window.PG;

  const SPECS = {
    click: { f0: 880, dur: 0.05, wave: "square", vol: 0.25 },
    select: { f0: 700, dur: 0.05, wave: "sine", vol: 0.25 },
    eat: { f0: 660, f1: 1320, dur: 0.09, wave: "square", vol: 0.3 },
    bounce: { f0: 440, dur: 0.05, wave: "square", vol: 0.3 },
    point: { f0: 520, f1: 1040, dur: 0.18, wave: "square", vol: 0.35 },
    shoot: { f0: 900, f1: 300, dur: 0.12, wave: "saw", vol: 0.28 },
    explode: { f0: 200, dur: 0.22, wave: "noise", vol: 0.4 },
    hit: { f0: 160, dur: 0.18, wave: "square", vol: 0.4 },
    rotate: { f0: 600, dur: 0.04, wave: "sine", vol: 0.22 },
    lock: { f0: 200, dur: 0.07, wave: "square", vol: 0.3 },
    line: { f0: 400, f1: 1200, dur: 0.25, wave: "square", vol: 0.4 },
    merge: { f0: 500, f1: 760, dur: 0.1, wave: "sine", vol: 0.3 },
    move: { f0: 330, dur: 0.03, wave: "square", vol: 0.18 },
    gameover: { f0: 440, f1: 120, dur: 0.5, wave: "saw", vol: 0.4 },
    win: { f0: 520, f1: 1300, dur: 0.45, wave: "square", vol: 0.4 },
    powerup: { f0: 600, f1: 1200, dur: 0.18, wave: "square", vol: 0.32 },
    level: { f0: 440, f1: 1100, dur: 0.32, wave: "square", vol: 0.38 },
  };

  let ctx = null;
  let master = null;
  const cache = new Map();
  const toneCache = new Map();
  const noteCache = new Map();
  let music = null;

  function ensure() {
    if (ctx) return ctx;
    const AC = window.AudioContext || window.webkitAudioContext;
    if (!AC) return null;
    try {
      ctx = new AC();
      master = ctx.createGain();
      master.gain.value = 0.9;
      master.connect(ctx.destination);
    } catch (e) {
      ctx = null;
    }
    return ctx;
  }

  function build(spec) {
    const rate = ctx.sampleRate;
    const n = Math.max(1, Math.floor(rate * spec.dur));
    const buf = ctx.createBuffer(1, n, rate);
    const data = buf.getChannelData(0);
    const f0 = spec.f0, f1 = spec.f1 != null ? spec.f1 : spec.f0;
    const att = Math.max(1, Math.floor(n * 0.05));
    const rel = Math.max(1, Math.floor(n * 0.2));
    let phase = 0;
    for (let i = 0; i < n; i++) {
      const f = f0 + (f1 - f0) * (i / n);
      phase += f / rate;
      const p = phase % 1;
      let v;
      if (spec.wave === "sine") v = Math.sin(PG.TAU * p);
      else if (spec.wave === "square") v = p < 0.5 ? 1 : -1;
      else if (spec.wave === "saw") v = 2 * p - 1;
      else v = Math.random() * 2 - 1;
      const env = Math.min(1, i / att, (n - i) / rel);
      data[i] = v * spec.vol * env;
    }
    return buf;
  }

  function enabled() {
    return !!PG.settings.data.sound;
  }

  function playBuffer(buf, vol) {
    try {
      const src = ctx.createBufferSource();
      src.buffer = buf;
      const g = ctx.createGain();
      g.gain.value = Math.max(0, Math.min(1, vol));
      src.connect(g);
      g.connect(master);
      src.start();
    } catch (e) {}
  }

  PG.audio = {
    /** Beim ersten Nutzer-Ereignis aufrufen (Browser-Autoplay-Regeln). */
    unlock() {
      if (!ensure()) return;
      if (ctx.state === "suspended") ctx.resume().catch(() => {});
    },
    play(name) {
      if (!enabled() || !ensure() || ctx.state !== "running") return;
      const spec = SPECS[name];
      if (!spec) return;
      let buf = cache.get(name);
      if (!buf) {
        buf = build(spec);
        cache.set(name, buf);
      }
      playBuffer(buf, Number(PG.settings.data.volume));
    },
    tone(freq, dur = 0.18, wave = "sine", vol = 0.35) {
      if (!enabled() || !ensure() || ctx.state !== "running") return;
      const key = Math.round(freq * 10) + "|" + Math.round(dur * 1000) + "|" + wave;
      let buf = toneCache.get(key);
      if (!buf) {
        buf = build({ f0: freq, dur, wave, vol: 1 });
        if (toneCache.size > 200) toneCache.clear();
        toneCache.set(key, buf);
      }
      playBuffer(buf, Number(PG.settings.data.volume) * vol);
    },
    rumble(ms = 120, strong = 0.6, weak = 0.4) {
      if (!PG.settings.data.haptik || !navigator.getGamepads) return;
      for (const gp of navigator.getGamepads()) {
        const act = gp && gp.vibrationActuator;
        if (act && act.playEffect) {
          act.playEffect("dual-rumble", { duration: ms, strongMagnitude: strong, weakMagnitude: weak }).catch(() => {});
        }
      }
    },
    SPECS,

    // ----- Musik-Schleifen (Gegenstück zu music_play/music_tick in audio.py) ---
    // Eine Stimme ist eine Float32Array (mono, Abtastrate = musicRate()), aus
    // Noten zusammengesetzt (noteSamples + renderVoice). Alle Stimmen laufen als
    // AudioBufferSource mit loop=true synchron. Die Musik gehört einem Spiel:
    // app.js ruft jedes Frame musicTick(current) auf.
    musicRate() {
      return ensure() ? ctx.sampleRate : 44100;
    },
    /** Samples einer Note (gecacht), Parameter wie note_bytes() in audio.py. */
    noteSamples(freq, dur, wave = "square", vol = 0.3, decay = 0) {
      const rate = this.musicRate();
      const n = Math.max(1, Math.floor(rate * dur));
      const key = [Math.round(freq * 100), n, wave, Math.round(vol * 1000), Math.round(decay * 100), rate].join("|");
      let data = noteCache.get(key);
      if (data) return data;
      data = new Float32Array(n);
      const att = Math.max(1, Math.floor(rate * 0.004));
      const rel = Math.max(1, Math.min(Math.floor(n / 3), Math.floor(rate * 0.03)));
      let phase = 0;
      let seed = (Math.floor(freq * 100) + n) >>> 0;
      const noise = () => {
        seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0;
        return (seed / 4294967296) * 2 - 1;
      };
      for (let i = 0; i < n; i++) {
        const tt = i / rate;
        let v, env;
        if (wave === "kick") {
          phase += (150 * Math.exp(-tt * 28) + 45) / rate;
          v = Math.sin(PG.TAU * phase);
          env = Math.exp(-tt * 9);
        } else if (wave === "snare") {
          phase += 190 / rate;
          v = 0.65 * noise() + 0.35 * Math.sin(PG.TAU * phase);
          env = Math.exp(-tt * 18);
        } else if (wave === "hat") {
          v = noise();
          env = Math.exp(-tt * 45);
        } else {
          phase += freq / rate;
          const p = phase % 1;
          if (wave === "sine") v = Math.sin(PG.TAU * p);
          else if (wave === "saw") v = 2 * p - 1;
          else if (wave === "triangle") v = p < 0.5 ? 4 * p - 1 : 3 - 4 * p;
          else if (wave === "noise") v = noise();
          else v = p < 0.5 ? 1 : -1;
          env = decay > 0 ? Math.exp(-tt * decay) : 1;
        }
        env *= Math.min(1, i / att, (n - i) / rel);
        data[i] = Math.max(-1, Math.min(1, v)) * vol * env;
      }
      if (noteCache.size > 400) noteCache.clear();
      noteCache.set(key, data);
      return data;
    },
    /** Stimme aus [[startSekunde, Float32Array], ...] mit Gesamtlänge in Sekunden. */
    renderVoice(notes, length) {
      const rate = this.musicRate();
      const total = Math.floor(rate * length);
      const out = new Float32Array(total);
      for (const [start, data] of notes) {
        const pos = Math.floor(rate * start);
        if (pos < 0 || pos >= total) continue;
        out.set(total - pos < data.length ? data.subarray(0, total - pos) : data, pos);
      }
      return out;
    },
    /** Startet Musik: voices = [[Float32Array, relVol], ...], owner = Spiel-Objekt. */
    music(voices, owner, volume = 1) {
      this.stopMusic();
      if (!ensure()) return false;
      const rate = ctx.sampleRate;
      const gain = ctx.createGain();
      gain.connect(master);
      const nodes = [];
      const start = ctx.currentTime + 0.05;
      for (const [data, vol] of voices) {
        try {
          const buf = ctx.createBuffer(1, data.length, rate);
          buf.getChannelData(0).set(data);
          const src = ctx.createBufferSource();
          src.buffer = buf;
          src.loop = true;
          const g = ctx.createGain();
          g.gain.value = vol;
          src.connect(g);
          g.connect(gain);
          src.start(start);
          nodes.push(src);
        } catch (e) {}
      }
      music = { owner, gain, nodes, volume, paused: false };
      this.applyMusicVolume();
      return nodes.length > 0;
    },
    stopMusic(owner) {
      if (!music || (owner && music.owner !== owner)) return;
      for (const n of music.nodes) {
        try {
          n.stop();
        } catch (e) {}
      }
      try {
        music.gain.disconnect();
      } catch (e) {}
      music = null;
    },
    setMusicVolume(v) {
      if (!music) return;
      music.volume = Math.max(0, Math.min(1, v));
      this.applyMusicVolume();
    },
    musicPlaying(owner) {
      return !!music && (!owner || music.owner === owner);
    },
    applyMusicVolume() {
      if (!music) return;
      const base = enabled() ? Number(PG.settings.data.volume) : 0;
      const target = music.paused ? 0 : base * music.volume;
      try {
        music.gain.gain.setTargetAtTime(target, ctx.currentTime, 0.02);
      } catch (e) {
        music.gain.gain.value = target;
      }
    },
    /** Jedes Frame aus app.js: Musik endet mit ihrem Spiel und pausiert mit ihm. */
    musicTick(current) {
      if (!music) return;
      if (current !== music.owner) {
        this.stopMusic();
        return;
      }
      music.paused = !!current.paused;
      this.applyMusicVolume();
    },
  };
})();
