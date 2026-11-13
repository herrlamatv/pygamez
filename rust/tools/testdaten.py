# -*- coding: utf-8 -*-
"""
testdaten.py
============
Baut einen gefuellten Datenordner fuer den Pruefstand (Fortschritt, Profile,
Replays): drei Profile, freigeschaltete Erfolge, Highscores, Duelle und eine
Statistik-Datei ueber rund 120 Tage bis heute - fester Zufall, also bei jedem
Aufruf gleich (bis auf das heutige Datum, an dem die Tage haengen).

    .venv/Scripts/python.exe rust/tools/testdaten.py <zielordner>

Der Ordner wird mit ``vergleich.py --daten <zielordner>`` in beide
Datenordner kopiert. Nie die echten Spielstaende anfassen: das Ziel muss ein
eigener (Kratz-)Ordner sein.
"""

import datetime
import json
import os
import random
import sys

HIER = os.path.dirname(os.path.abspath(__file__))
WURZEL = os.path.dirname(os.path.dirname(HIER))
sys.path.insert(0, WURZEL)

SPIELE = ["snake", "tetris", "pong", "geodash", "minigolf", "chess", "2048", "casino",
          "wordle", "breakout", "sudoku", "pinball"]
PROFILE = [("p1", "Lama", "llama", 0), ("p2", "Alpaka", "star", 1), ("p3", "Vikunja", "rocket", 2)]


def stempel(tag, r):
    return "%s %02d:%02d" % (tag.isoformat(), r.randint(8, 23), r.randint(0, 59))


def profil_statistik(r, heute, pid, faktor):
    games, days, series, events = {}, {}, {}, []
    hours = [0.0] * 24
    gespielt = r.sample(SPIELE, 6 + int(3 * faktor))
    for key in gespielt:
        games[key] = {"plays": 0, "time": 0.0, "wins": 0, "losses": 0, "records": 0, "best": 0,
                      "best_at": "", "first": "", "last": "", "longest": 0.0}
    tage = sorted(r.sample(range(1, 120), int(25 + 30 * faktor))) + [0]
    for vor in reversed(tage):
        tag = heute - datetime.timedelta(days=vor)
        eintrag = {"t": 0.0, "p": 0, "w": 0, "a": 0, "g": {}}
        for _ in range(r.randint(1, 4)):
            key = r.choice(gespielt)
            g = games[key]
            dauer = round(r.uniform(30, 900), 1)
            st = stempel(tag, r)
            g["plays"] += 1
            g["time"] = round(g["time"] + dauer, 1)
            g["first"] = g["first"] or st
            g["last"] = st
            g["longest"] = max(g["longest"], dauer)
            if r.random() < 0.5:
                if r.random() < 0.55:
                    g["wins"] += 1
                    eintrag["w"] += 1
                else:
                    g["losses"] += 1
            punkte = r.randint(10, 5000)
            neu = punkte > g["best"]
            if neu:
                g["best"] = punkte
                g["best_at"] = st
                g["records"] += 1
                events.append({"t": st, "k": "rec", "g": key, "v": punkte})
            series.setdefault(key, []).append([st, punkte, 1 if neu else 0])
            eintrag["t"] = round(eintrag["t"] + dauer, 1)
            eintrag["p"] += 1
            eintrag["g"][key] = round(eintrag["g"].get(key, 0.0) + dauer, 1)
            hours[int(st[11:13])] = round(hours[int(st[11:13])] + dauer, 1)
        days[tag.isoformat()] = eintrag
    for key in series:
        series[key] = series[key][-40:]
    return games, days, hours, series, events


def main():
    if len(sys.argv) < 2:
        raise SystemExit("Aufruf: testdaten.py <zielordner>")
    ziel = sys.argv[1]
    os.makedirs(ziel, exist_ok=True)
    r = random.Random(4711)
    heute = datetime.date.today()
    from achievements import _build_defs
    defs = [d["id"] for d in _build_defs()]

    mem = {"mem": {"lang": "de"},
           "profiles": {"active": "p1", "order": [p[0] for p in PROFILE], "list": {}},
           "players": {}}
    stat = {"format": "pygamez.stats", "v": 1, "app": "PyGameZ",
            "updated": stempel(heute, r), "profiles": {}}
    for i, (pid, name, icon, farbe) in enumerate(PROFILE):
        faktor = 1.0 - 0.35 * i
        mem["profiles"]["list"][pid] = {"name": name, "icon": icon, "color": farbe,
                                        "created": stempel(heute - datetime.timedelta(days=130), r),
                                        "seen": stempel(heute, r)}
        games, days, hours, series, events = profil_statistik(r, heute, pid, faktor)
        hs = {k: v["best"] for k, v in games.items()}
        ach = {}
        for aid in r.sample(defs, int(30 * faktor) + 5):
            tag = heute - datetime.timedelta(days=r.randint(0, 100))
            ach[aid] = stempel(tag, r)
            events.append({"t": ach[aid], "k": "ach", "id": aid, "p": 10})
        for lvl in range(2, 2 + int(6 * faktor)):
            tag = heute - datetime.timedelta(days=r.randint(0, 100))
            events.append({"t": stempel(tag, r), "k": "lvl", "v": lvl})
        duel = {}
        for other, _n, _ic, _f in PROFILE:
            if other == pid:
                continue
            duel[other] = {}
            for key in r.sample(["pong", "chess", "tetris", "airhockey"], 2):
                w, l = r.randint(0, 6), r.randint(0, 6)
                duel[other][key] = {"w": w, "l": l}
                for _ in range(min(3, w + l)):
                    tag = heute - datetime.timedelta(days=r.randint(0, 60))
                    events.append({"t": stempel(tag, r), "k": "duel", "g": key, "o": other,
                                   "w": r.random() < 0.5})
        events.sort(key=lambda e: e["t"])
        # "Dein Startlevel" gilt als gesehen (die Karte zeigt main.py beim
        # Programmstart - den bildet der Python-Pruefstand nicht nach).
        mem["players"][pid] = {"highscores": hs, "achievements": ach,
                               "progress": {"intro_seen": True}}
        stat["profiles"][pid] = {"games": games, "days": days, "hours": hours, "series": series,
                                 "events": events, "duel": duel,
                                 "streak": {"cur": 3, "best": 9, "first_day": min(days), "last_day": max(days)},
                                 "points": 0, "level": 1, "longest": max(g["longest"] for g in games.values())}
    with open(os.path.join(ziel, "mem.json"), "w", encoding="utf-8") as f:
        json.dump(mem, f, indent=2, ensure_ascii=False)
    with open(os.path.join(ziel, "stats.pgzlamastats"), "w", encoding="utf-8") as f:
        json.dump(stat, f, indent=2, ensure_ascii=False)
    print("Testdaten in", ziel)


if __name__ == "__main__":
    main()
