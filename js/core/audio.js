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
  };
})();
