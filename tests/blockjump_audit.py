# -*- coding: utf-8 -*-
"""Headless-Audit fuer Block Jump: Level 1-15 x easy/normal/hard.

Prueft je Level:
  (1) kein Leiterschacht versiegelt + Ausstiegs-Block vorhanden,
  (2) Leiter-Coins in der Kletterspalte,
  (3) jede Luecke per Sprungphysik schaffbar,
  (4) simulierter Leiter-Aufstieg (W halten) erreicht das Folge-Pad.

Aufruf aus dem Repo-Root:  python tests/blockjump_audit.py
Exit-Code 0 = alle Pruefungen bestanden.
"""
import json
import math
import os
import sys

os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import pygame
pygame.init()
pygame.display.set_mode((640, 480))

import settings as settings_mod
from games import blockjump as bj

# Testlaeufe duerfen nie die echte settings.json ueberschreiben
settings_mod.save_settings = lambda s: None

GS = json.loads(json.dumps(settings_mod.DEFAULTS))


class Probe(bj.BlockJumpGame):
    """Zeichnet _pad-Aufrufe auf und stummt den Sound."""

    def _build_level(self, level):
        self.pads = []
        super()._build_level(level)

    def _pad(self, cx, cz, y, hw, hd, typ):
        self.pads.append((cx, cz, y, hw, hd))
        super()._pad(cx, cz, y, hw, hd, typ)

    def play_sound(self, name):
        pass


def ladder_columns(world):
    cols = {}
    for (x, y, z), t in world.items():
        if t == bj.LADDER:
            cols.setdefault((x, z), []).append(y)
    return {k: (min(v), max(v)) for k, v in cols.items()}


def max_jump(dy):
    """Maximale horizontale Sprungweite bei Hoehenversatz dy (Bloecke)."""
    disc = bj.JUMP_VEL ** 2 - 2 * bj.GRAVITY * dy
    if disc <= 0:
        return 0.0
    return bj.MOVE_SPEED * (bj.JUMP_VEL + math.sqrt(disc)) / bj.GRAVITY


def audit(g):
    errs, world = [], g.world
    lads = ladder_columns(world)
    for (x, z), (y0, y1) in lads.items():
        for yy in range(y0, y1 + 1):                       # Sprossen intakt?
            if world.get((x, yy, z)) != bj.LADDER:
                errs.append(f"Sprosse ueberschrieben bei {(x, yy, z)}")
        for yy in (y1 + 1, y1 + 2, y1 + 3):                # Schacht offen?
            if world.get((x, yy, z), bj.EMPTY) in bj.SOLID:
                errs.append(f"Schacht versiegelt ueber {(x, z)} bei y={yy}")
        if world.get((x, y1, z + 1), bj.EMPTY) not in bj.SOLID:
            errs.append(f"Kein Ausstiegs-Block hinter Leiter {(x, z)}")
        for yy in (y1 + 1, y1 + 2):                        # Kopffreiheit Ausstieg
            if world.get((x, yy, z + 1), bj.EMPTY) in bj.SOLID:
                errs.append(f"Ausstieg blockiert ueber {(x, z + 1)} y={yy}")
        if not any(abs(c[0] - (x + 0.5)) < 0.4 and abs(c[2] - (z + 0.5)) < 0.4
                   for c in g.coins):                      # Coin in der Spalte?
            errs.append(f"Kein Coin in Leiter-Spalte {(x, z)}")
    for p, q in zip(g.pads, g.pads[1:]):                   # Luecken-Audit
        (x0, z0, y0, w0, d0), (x1, z1, y1_, w1, d1) = p, q
        if any(lx == x0 and z0 < lz <= z1 for (lx, lz) in lads):
            continue                                       # Leiter-Uebergang
        if world.get((x0, y0 + 1, z0)) == bj.SPRING:
            continue                                       # Katapult-Uebergang
        need = (z1 - d1 - 0.3) - (z0 + d0 + 1 + 0.3)
        if need > max_jump(y1_ - y0) * 0.9:
            errs.append(f"Unschaffbare Luecke {p} -> {q}: brauche {need:.2f}, "
                        f"max {max_jump(y1_ - y0):.2f} (dy={y1_ - y0})")
    return errs


def climb_sim(g):
    fails = []
    for (x, z), (y0, y1) in sorted(ladder_columns(g.world).items()):
        g.px, g.py, g.pz = x + 0.5, float(y0), z + 0.35    # im Schacht starten
        g.vx = g.vy = g.vz = 0.0
        g.yaw = g.pitch = 0.0                              # Blick nach +z
        g.on_ground, g.state, g.lives = False, bj.PLAY, 99
        g.held = {"up"}                                    # W halten
        ok = False
        for _ in range(720):                               # max 12 s Simulation
            g._physics(1 / 60.0)
            # Erfolg: steht auf dem Folge-Pad ODER wurde dort von einem
            # Sprungblock katapultiert (der Bounce loescht on_ground im
            # selben Tick wieder, beweist aber, dass der Ausstieg klappt)
            bounced = g.vy >= bj.SPRING_VEL - 1e-6
            if (g.on_ground or bounced) and g.py > y1 + 0.9 and g.pz > z + 0.8:
                ok = True
                break
        if not ok:
            fails.append(f"Leiter {(x, z)}: haengt bei y={g.py:.2f} "
                         f"z={g.pz:.2f} (Ziel: y>{y1 + 0.9:.1f}, z>{z + 0.8:.1f})")
    return fails


def main():
    bad = 0
    for mode in ("easy", "normal", "hard"):
        g = Probe(pygame.Surface((640, 480)), 640, 480, mode=mode,
                  game_settings=GS)
        for level in range(1, 16):
            g._build_level(level)
            errs = audit(g)          # vor climb_sim (das sammelt Coins ein)
            errs += climb_sim(g)
            for e in errs:
                print(f"[FAIL] {mode} L{level}: {e}")
            bad += len(errs)
    print("OK: alle Pruefungen bestanden" if bad == 0 else f"{bad} Fehler")
    sys.exit(0 if bad == 0 else 1)


if __name__ == "__main__":
    main()
