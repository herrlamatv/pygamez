/*
 * geodash_music.js - Prozeduraler Soundtrack (Port von games/geodash_music.py)
 * ===========================================================================
 * Aus Stil, Tempo und Level-id entstehen vier Stimmen (Drums, Hats, Bass,
 * Lead). Der Zufall kommt aus PG.seedrand - dieselbe Melodie wie am PC.
 */
(function () {
  "use strict";

  const STYLES = {
    drive: { bpm: 128, bass: "saw", lead: "square", leadStep: 2, hat: 2, kick: [0, 4, 8, 12], snare: [4, 12], octave: 0, decay: 9.0 },
    chip: { bpm: 140, bass: "square", lead: "square", leadStep: 2, hat: 1, kick: [0, 6, 8], snare: [4, 12], octave: 12, decay: 4.0 },
    dream: { bpm: 120, bass: "triangle", lead: "triangle", leadStep: 2, hat: 2, kick: [0, 10], snare: [8], octave: 12, decay: 6.0 },
    dark: { bpm: 174, bass: "saw", lead: "saw", leadStep: 4, hat: 1, kick: [0, 10], snare: [4, 12], octave: -12, decay: 12.0 },
  };
  const PROGRESSIONS = [[0, 5, 2, 6], [0, 3, 4, 0], [0, 6, 5, 4], [0, 5, 3, 4]];
  const MINOR = [0, 2, 3, 5, 7, 8, 10];
  const BARS = 8;

  const midiFreq = (m) => 440 * Math.pow(2, (m - 69) / 12);

  function degree(root, deg) {
    const octv = Math.floor(deg / 7);
    return root + 12 * octv + MINOR[deg - 7 * octv];
  }

  /** Noten des Loops: {length, bpm, voices: [[[start, freq, dur, wave, vol, decay], ...], relVol]}. */
  function compose(style, bpm, seedText) {
    const st = STYLES[style];
    if (!st) return null;
    bpm = Math.trunc(bpm || st.bpm);
    const sr = PG.seedrand;
    const rng = new sr.Rand(sr.seedFrom("gdmusic", seedText, style));
    const root = 45 + rng.randint(0, 7);
    const prog = PROGRESSIONS[rng.randint(0, PROGRESSIONS.length - 1)];
    const six = 60 / bpm / 4;
    const length = BARS * 16 * six;
    const drums = [], hats = [], bass = [], lead = [];
    for (let bar = 0; bar < BARS; bar++) {
      const chord = prog[bar % prog.length];
      const base = bar * 16;
      for (let s = 0; s < 16; s++) {
        const t0 = (base + s) * six;
        if (st.snare.includes(s) || (bar === BARS - 1 && s >= 12 && s % 2 === 0)) drums.push([t0, 190, six * 1.8, "snare", 0.55, 0]);
        else if (st.kick.includes(s)) drums.push([t0, 50, six * 2.4, "kick", 0.9, 0]);
      }
      for (let s = 0; s < 16; s += st.hat) {
        if (st.hat === 2 && s % 4 === 0) continue;
        hats.push([(base + s) * six, 8000, six * 0.9, "hat", s % 4 === 2 ? 0.22 : 0.14, 0]);
      }
      const bassRoot = degree(root, chord);
      for (let s = 0; s < 16; s += 2) {
        const m = bassRoot + (s === 6 || s === 14 ? 12 : 0);
        bass.push([(base + s) * six, midiFreq(m), six * 1.7, st.bass, 0.34, 3]);
      }
      const tones = [0, 2, 4, 7].map((d) => degree(root + 12 + st.octave, chord + d));
      const pattern = [0, 1, 2, 3, 2, 1, 2, 3];
      rng.shuffle(pattern);
      let i = 0;
      for (let s = 0; s < 16; s += st.leadStep, i++) {
        if (rng.chance(0.12) && s % 4) continue;
        let m = tones[pattern[i % pattern.length]];
        if (rng.chance(0.15)) m += 12;
        lead.push([(base + s) * six, midiFreq(m), six * st.leadStep * 0.95, st.lead, 0.2, st.decay]);
      }
    }
    return { length, bpm, voices: [[drums, 0.9], [hats, 0.7], [bass, 0.8], [lead, 0.75]] };
  }

  /** Fertige Stimmen für PG.audio.music (oder null bei "none"). */
  function buildVoices(style, bpm, seedText) {
    const song = compose(style, bpm, seedText);
    if (!song) return null;
    const a = PG.audio;
    return song.voices.map(([notes, vol]) => [
      a.renderVoice(notes.map(([t0, f, dur, wave, v, decay]) => [t0, a.noteSamples(f, dur, wave, v, decay)]), song.length),
      vol,
    ]);
  }

  function styleBpm(level) {
    const st = STYLES[level.music];
    if (!st) return 0;
    const bpm = Math.trunc(Number(level.bpm) || 0);
    return bpm >= 60 && bpm <= 220 ? bpm : st.bpm;
  }

  function beatPulse(t, bpm) {
    if (bpm <= 0 || t < 0) return 0;
    const beat = (t * bpm) / 60;
    return Math.max(0, 1 - (beat - Math.floor(beat)) * 3);
  }

  PG.gdMusic = { STYLES, compose, buildVoices, styleBpm, beatPulse };
})();
