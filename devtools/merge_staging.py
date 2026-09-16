# -*- coding: utf-8 -*-
"""
merge_staging.py
================
Spielt Übersetzungs- und LamaWiki-Bausteine aus ``devtools/staging/`` in die
14 Sprachdateien ein. Gedacht für große Updates, an denen mehrere Leute (bzw.
Agenten) gleichzeitig arbeiten: jeder schreibt nur SEINE Staging-Datei, dieses
Skript führt alles zusammen - mit Dateisperre, sodass sich parallele Läufe
nicht gegenseitig überschreiben. Es ist idempotent (beliebig oft ausführbar).

Übersetzungen: ``devtools/staging/i18n/<name>.json``

    {
      "keys": {
        "cr.subtitle": {"de": "...", "en": "...", "fr": "...", ... alle 14},
        ...
      },
      "remove": ["alter.key", ...]          # optional
    }

LamaWiki-Seiten: ``devtools/staging/wiki/<page_id>.json``

    {
      "after": "bowling",                    # neue Seite hinter dieser id einfügen
      "pages": {"de": {...Seite...}, "en": {...}, ... alle 14}
    }

Eine vorhandene Seite gleicher id wird an ihrer Stelle ersetzt ("after" gilt
nur für neue Seiten). Aufruf:

    python devtools/merge_staging.py            # alles einspielen
    python devtools/merge_staging.py --check    # nur prüfen, nichts schreiben
"""

import json
import os
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
STAGING = os.path.join(ROOT, "devtools", "staging")
LOCK = os.path.join(STAGING, ".merge.lock")

MAIN_LANGS = ("de", "en", "fr", "es", "pt")
EXP_LANGS = ("pl", "tr", "da", "no", "sv", "fi", "cs", "sl", "hr")
LANGS = MAIN_LANGS + EXP_LANGS


def lang_path(folder, code):
    sub = "" if code in MAIN_LANGS else "lang.expansion"
    return os.path.join(ROOT, folder, sub, code + ".json")


def _read(path):
    with open(path, "rb") as f:
        raw = f.read()
    newline = "\r\n" if b"\r\n" in raw else "\n"
    return json.loads(raw.decode("utf-8")), newline


def _write(path, data, newline, sort_keys):
    text = json.dumps(data, indent=2, ensure_ascii=False, sort_keys=sort_keys) + "\n"
    if newline != "\n":
        text = text.replace("\n", newline)
    tmp = path + ".tmp"
    with open(tmp, "w", encoding="utf-8", newline="") as f:
        f.write(text)
    os.replace(tmp, path)


def _staging_files(kind):
    folder = os.path.join(STAGING, kind)
    if not os.path.isdir(folder):
        return []
    return sorted(os.path.join(folder, n) for n in os.listdir(folder)
                  if n.endswith(".json"))


def collect():
    """Liest alle Staging-Dateien. Gibt (keys, removes, wiki, fehler) zurück."""
    errors = []
    keys, removes, wiki = {}, set(), []
    for path in _staging_files("i18n"):
        name = os.path.basename(path)
        try:
            with open(path, "r", encoding="utf-8") as f:
                data = json.load(f)
        except (OSError, ValueError) as exc:
            errors.append("%s: kein gültiges JSON (%s)" % (name, exc))
            continue
        for key, per_lang in (data.get("keys") or {}).items():
            missing = [c for c in LANGS if not isinstance(per_lang.get(c), str)
                       or not per_lang.get(c)]
            if missing:
                errors.append("%s: %s fehlt in %s" % (name, key, ",".join(missing)))
                continue
            if key in keys and keys[key][1] != per_lang:
                errors.append("%s: %s auch in %s (unterschiedlich)"
                              % (name, key, keys[key][0]))
            keys[key] = (name, per_lang)
        removes.update(data.get("remove") or [])
    for path in _staging_files("wiki"):
        name = os.path.basename(path)
        try:
            with open(path, "r", encoding="utf-8") as f:
                data = json.load(f)
        except (OSError, ValueError) as exc:
            errors.append("%s: kein gültiges JSON (%s)" % (name, exc))
            continue
        pages = data.get("pages") or {}
        missing = [c for c in LANGS if not isinstance(pages.get(c), dict)]
        if missing:
            errors.append("wiki %s: Sprache(n) fehlen: %s" % (name, ",".join(missing)))
            continue
        ids = {pages[c].get("id") for c in LANGS}
        if len(ids) != 1 or None in ids:
            errors.append("wiki %s: id muss in allen Sprachen gleich sein" % name)
            continue
        for c in LANGS:
            page = pages[c]
            for field in ("category", "title", "sections"):
                if field not in page:
                    errors.append("wiki %s/%s: Feld '%s' fehlt" % (name, c, field))
        wiki.append((name, data.get("after"), pages))
    return keys, removes, wiki, errors


def _acquire_lock(timeout=120):
    os.makedirs(STAGING, exist_ok=True)
    t0 = time.time()
    while True:
        try:
            fd = os.open(LOCK, os.O_CREAT | os.O_EXCL | os.O_WRONLY)
            os.write(fd, str(os.getpid()).encode())
            os.close(fd)
            return True
        except FileExistsError:
            # Verwaiste Sperre (älter als 10 Minuten) aufräumen.
            try:
                if time.time() - os.path.getmtime(LOCK) > 600:
                    os.remove(LOCK)
                    continue
            except OSError:
                pass
            if time.time() - t0 > timeout:
                return False
            time.sleep(0.2)


def merge(check_only=False):
    keys, removes, wiki, errors = collect()
    for e in errors:
        print("FEHLER:", e)
    if errors:
        print("%d Fehler - nichts geschrieben." % len(errors))
        return 1
    if check_only:
        print("OK: %d Schlüssel, %d Wiki-Seiten in Staging." % (len(keys), len(wiki)))
        return 0
    if not _acquire_lock():
        print("FEHLER: Sperre nicht bekommen (läuft ein anderer Merge?)")
        return 2
    try:
        for code in LANGS:
            path = lang_path("lang", code)
            data, nl = _read(path)
            for key in removes:
                data.pop(key, None)
            for key, (_, per_lang) in keys.items():
                data[key] = per_lang[code]
            _write(path, dict(sorted(data.items())), nl, sort_keys=False)

            path = lang_path("lamawiki", code)
            data, nl = _read(path)
            pages = data["pages"]
            for _, after, per_lang in wiki:
                page = per_lang[code]
                idx = next((i for i, p in enumerate(pages) if p.get("id") == page["id"]), None)
                if idx is not None:
                    pages[idx] = page
                    continue
                pos = next((i for i, p in enumerate(pages) if p.get("id") == after), None)
                if pos is None:
                    pages.append(page)
                else:
                    pages.insert(pos + 1, page)
            _write(path, data, nl, sort_keys=False)
    finally:
        try:
            os.remove(LOCK)
        except OSError:
            pass
    counts = set()
    for code in LANGS:
        with open(lang_path("lang", code), "r", encoding="utf-8") as f:
            counts.add(frozenset(json.load(f).keys()))
    print("OK: %d Schlüssel und %d Wiki-Seiten eingespielt; Key-Sets identisch: %s"
          % (len(keys), len(wiki), len(counts) == 1))
    return 0


if __name__ == "__main__":
    sys.exit(merge(check_only="--check" in sys.argv))
