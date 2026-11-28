# -*- coding: utf-8 -*-
"""
testdaten.py
============
Baut einen gefuellten Datenordner fuer den Pruefstand (Fortschritt, Profile,
Replays, Matches): drei Profile, freigeschaltete Erfolge, Highscores, Duelle, eine
Statistik-Datei ueber rund 120 Tage bis heute und eine Match-Historie
(``matches.pgzlamamatches``) - fester Zufall, also bei jedem
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


def match_historie(heute):
    """matches.pgzlamamatches fuer den MatchScreen (eigener Zufall, damit die
    uebrigen Testdaten gleich bleiben): viele Spiele, alle Info-Schluessel,
    Ergebnisse, Rekord/Abbruch/Tages-/Archiv-Flags, Summen ueber der Kappe."""
    r = random.Random(815)
    t0 = datetime.datetime.combine(heute, datetime.time(20, 0))
    heute_i = (heute - datetime.date(2026, 1, 1)).days

    def stempel_m(k):
        return (t0 - datetime.timedelta(minutes=37 * k + r.randint(0, 20))).strftime("%Y-%m-%d %H:%M")

    def spiel(n_list, extra_n, bauen):
        liste = []
        for k in range(n_list):
            liste.append(bauen(k))
        liste.reverse()                     # aelteste zuerst (wie matches.py)
        g = {"n": len(liste) + extra_n, "w": 0, "l": 0, "d": 0, "best": None, "sum": 0,
             "time": 0, "first": "", "list": liste}
        for e in liste:
            g["w"] += e[3] == 1
            g["l"] += e[3] == 0
            g["d"] += e[3] == 2
            g["best"] = e[2] if g["best"] is None else max(g["best"], e[2])
            g["sum"] += e[2]
            g["time"] += e[4]
        g["first"] = liste[0][0]
        g["time"] += 60 * extra_n
        return g

    def snake(k):
        multi = k % 5 == 0
        info = {"vs": "Alpaka"} if multi else None
        res = (1 if k % 2 else 0) if multi else None
        flags = (1 if k % 7 == 3 else 0) | (2 if k % 9 == 4 else 0)
        return [stempel_m(k), "multi" if multi else "single", r.randint(0, 900), res,
                r.randint(4, 400), flags, info]

    def crossy(k):
        daily = k % 2 == 0
        info = {"c": r.randint(0, 9), "ch": r.choice(["chicken", "fox", "robot", "xyz"])}
        flags = 0
        if daily:
            day = heute_i - (k % 4)
            info = {"day": day, **info}
            flags = 4 | (8 if day != heute_i else 0)
        return [stempel_m(k), "daily" if daily else "endless", r.randint(0, 250), None,
                r.randint(10, 300), flags | (1 if k == 5 else 0), info]

    def blackjack(k):
        bet = r.choice([10, 20, 50, 1500])
        win = r.choice([0, bet, 2 * bet, bet * 5 // 2])
        net = win - bet
        return [stempel_m(k), "single", net, 1 if net > 0 else 0 if net < 0 else 2,
                r.randint(5, 60), 0, {"bet": bet, "win": win}]

    def lamacrossing(k):
        bet = r.choice([10, 100, 1500])
        lane = r.randint(1, 9)
        diff = r.choice(["easy", "medium", "hard", "extreme"])
        win = 0 if k % 3 == 0 else bet * (100 + 37 * lane) // 100
        info = {"bet": bet, "win": win, "ln": lane, "lv": diff}
        if win:
            info["x"] = 100 + 37 * lane
        net = win - bet
        return [stempel_m(k), "single", net, 1 if net > 0 else 0 if net < 0 else 2,
                r.randint(3, 40), 0, info]

    def sudoku(k):
        if k % 3 == 0:
            info = {"day": heute_i - k, "tm": r.randint(100, 900), "st": r.randint(1, 3)}
            flags = 4 | (8 if k else 0)
            res = 1
        else:
            info = {"vr": r.choice(["classic", "x", "killer", "mini"]), "lv": r.randint(1, 100)}
            flags = 2 if k % 4 == 1 else 0
            res = None
        return [stempel_m(k), "comfort", r.randint(0, 3000), res, r.randint(60, 1800), flags, info]

    def wordle(k):
        mode = ["endless", "daily", "dordle", "quordle"][k % 4]
        info = {"len": 5}
        if mode == "endless":
            info["wd"] = r.randint(0, 9)
        else:
            info["g"] = r.randint(2, 9)
        if mode == "daily":
            info = {"day": heute_i - k, **info}
        return [stempel_m(k), mode, r.randint(0, 60), None, r.randint(20, 400),
                (4 | (8 if k else 0)) if mode == "daily" else 0, info]

    def chess(k):
        return [stempel_m(k), "multi" if k % 2 else "single", 0, k % 3, r.randint(60, 2400), 0,
                {"vs": "Vikunja"} if k % 2 else None]

    def lamaplinko(k):
        bet = 10
        x = r.choice([20, 50, 110, 300, 1000])
        win = bet * x // 100
        net = win - bet
        return [stempel_m(k), "single", net, 1 if net > 0 else 0 if net < 0 else 2, 1, 0,
                {"bet": bet, "win": win, "x": x, "rw": r.choice([8, 12, 16]),
                 "lv": r.choice(["low", "medium", "high"])}]

    p1 = {"snake": spiel(45, 230, snake), "crossy": spiel(28, 0, crossy),
          "blackjack": spiel(18, 0, blackjack), "lamacrossing": spiel(14, 0, lamacrossing),
          "sudoku": spiel(12, 0, sudoku), "wordle": spiel(16, 0, wordle), "chess": spiel(9, 0, chess),
          "lamaplinko": spiel(6, 0, lamaplinko), "pong": spiel(1, 0, chess)}
    p2 = {"snake": spiel(3, 0, snake)}
    doc = {"format": "pygamez.matches", "v": 1, "app": "PyGameZ",
           "updated": t0.strftime("%Y-%m-%d %H:%M"), "profiles": {"p1": p1, "p2": p2}}
    return json.dumps(doc, ensure_ascii=False, separators=(",", ":"))


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
        # Favoriten (Stern in der Spieleliste) - je Profil verschieden; das
        # aktive Profil zeigt sie im Startbildschirm gross und vorne.
        mem["players"][pid]["favorites"] = {"games": [["tetris", "snake", "lamacrash"],
                                                      ["chess"], []][i % 3]}
        stat["profiles"][pid] = {"games": games, "days": days, "hours": hours, "series": series,
                                 "events": events, "duel": duel,
                                 "streak": {"cur": 3, "best": 9, "first_day": min(days), "last_day": max(days)},
                                 "points": 0, "level": 1, "longest": max(g["longest"] for g in games.values())}
    with open(os.path.join(ziel, "mem.json"), "w", encoding="utf-8") as f:
        json.dump(mem, f, indent=2, ensure_ascii=False)
    with open(os.path.join(ziel, "stats.pgzlamastats"), "w", encoding="utf-8") as f:
        json.dump(stat, f, indent=2, ensure_ascii=False)
    with open(os.path.join(ziel, "matches.pgzlamamatches"), "w", encoding="utf-8") as f:
        f.write(match_historie(heute))
    print("Testdaten in", ziel)


if __name__ == "__main__":
    main()
