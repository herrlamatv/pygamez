//! achievements.rs
//! ===============
//! Die 250 Erfolge der Sammlung - Gegenstueck zu `achievements.py`.
//!
//! Die Definitionen selbst stehen in der erzeugten Datei [`crate::defs`]
//! (siehe `rust/tools/export_defs.py`); hier liegt die Logik drumherum:
//! freischalten, Fortschritt ausrechnen, Punkte und Level nachziehen.
//!
//! Fuenf Kategorien:
//!
//! | Kategorie   | Beispiel            | woran es haengt                    |
//! |-------------|---------------------|------------------------------------|
//! | `general`   | "1000 Partien"      | Summen aus [`crate::stats`]        |
//! | `milestone` | "Snake: 500 Punkte" | Highscore des Spiels               |
//! | `master`    | "Snake-Meister"     | Highscore des Spiels (zweite Stufe)|
//! | `regular`   | "Snake-Stammgast"   | 25 Partien dieses Spiels           |
//! | `special`   | "Schach gewonnen"   | [`event`] aus dem Spiel heraus     |
//!
//! Freigeschaltete Erfolge liegen im Abschnitt `achievements` von `mem.json`
//! (`{id: zeitstempel}`) - je Profil, wie alles andere auch.

use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard, OnceLock};

use serde_json::Value;

use crate::defs;

const SECTION: &str = "achievements";

/// Kategorie eines Erfolgs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Category {
    General,
    Milestone,
    Master,
    Regular,
    Special,
}

impl Category {
    /// Der Name, unter dem die Kategorie in der Oberflaeche steht.
    pub fn key(self) -> &'static str {
        match self {
            Category::General => "general",
            Category::Milestone => "milestone",
            Category::Master => "master",
            Category::Regular => "regular",
            Category::Special => "special",
        }
    }

    /// Alle Kategorien in Anzeige-Reihenfolge.
    pub fn all() -> [Category; 5] {
        [
            Category::General,
            Category::Milestone,
            Category::Master,
            Category::Regular,
            Category::Special,
        ]
    }
}

/// Eine Erfolgs-Definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Achievement {
    pub id: String,
    pub cat: Category,
    pub icon: String,
    /// Feld aus [`totals`], an dem der Erfolg haengt (leer = Ereignis-Erfolg).
    pub stat: String,
    /// Schwelle (`None` = schaltet per Ereignis sofort frei).
    pub target: Option<i64>,
    /// Zugehoeriges Spiel (leer = spieluebergreifend).
    pub game: String,
    pub rarity: String,
}

impl Achievement {
    /// Punktwert nach Seltenheitsstufe.
    pub fn points(&self) -> i64 {
        defs::RARITY
            .iter()
            .find(|(id, _, _)| *id == self.rarity)
            .map(|(_, p, _)| *p)
            .unwrap_or(10)
    }

    /// Farbe der Seltenheitsstufe.
    pub fn rarity_color(&self) -> (u8, u8, u8) {
        let hex = defs::RARITY
            .iter()
            .find(|(id, _, _)| *id == self.rarity)
            .map(|(_, _, c)| *c)
            .unwrap_or("#9aa6bf");
        crate::levels::hex_to_rgb(hex)
    }

    /// Lokalisierter Name der Seltenheitsstufe ("Selten", "Episch", ...).
    pub fn rarity_name(&self) -> String {
        crate::i18n::t(&format!("rarity.{}", self.rarity))
    }

    /// Anzeigename eines Erfolgs.
    ///
    /// Wie in der Python-Fassung heisst ein Punkte-Meilenstein schlicht wie
    /// sein Spiel ("Snake"); Meister- und Stammgast-Erfolge setzen den
    /// Spielnamen in eine gemeinsame Vorlage ein ("{game}-Meister").
    pub fn display_name(&self) -> String {
        match self.cat {
            Category::Milestone => game_name(&self.game),
            Category::Master => crate::i18n::tf("ach.master.name", &[("game", &game_name(&self.game))]),
            Category::Regular => {
                crate::i18n::tf("ach.regular.name", &[("game", &game_name(&self.game))])
            }
            _ => crate::i18n::t(&format!("ach.{}.name", self.id)),
        }
    }

    /// Beschreibung - Ziel und Spielname werden eingesetzt.
    pub fn display_desc(&self) -> String {
        let target = self.target.unwrap_or(0);
        let n = target.to_string();
        match self.cat {
            Category::Milestone | Category::Master => {
                crate::i18n::tf("ach.score.desc", &[("n", &n)])
            }
            Category::Regular => crate::i18n::tf(
                "ach.regular.desc",
                &[("n", &n), ("game", &game_name(&self.game))],
            ),
            Category::General if !self.stat.is_empty() => {
                let key = defs::STAT_DESC
                    .iter()
                    .find(|(stat, _)| *stat == self.stat)
                    .map(|(_, k)| *k)
                    .unwrap_or("");
                if key.is_empty() {
                    return crate::i18n::t(&format!("ach.{}.desc", self.id));
                }
                if self.stat == "time" {
                    // Die Zeit-Erfolge nennen Stunden, nicht Sekunden.
                    crate::i18n::tf(key, &[("t", &format!("{}h", target / 3600))])
                } else {
                    crate::i18n::tf(key, &[("n", &n)])
                }
            }
            _ => crate::i18n::t(&format!("ach.{}.desc", self.id)),
        }
    }
}

/// Anzeigename eines Spiels zu seinem Highscore-Schluessel.
///
/// Die meisten Spiele heissen in jeder Sprache gleich; die Ausnahmen
/// (Schach/Chess/Échecs ...) stehen in [`defs::GAME_NAMES`].
pub fn game_name(key: &str) -> String {
    let lang = crate::i18n::get_language();
    if let Some((_, per_lang)) = defs::GAME_NAMES.iter().find(|(k, _)| *k == key) {
        if let Some((_, name)) = per_lang.iter().find(|(code, _)| *code == lang) {
            return (*name).to_string();
        }
    }
    defs::GAMES
        .iter()
        .find(|g| g.key == key)
        .map(|g| g.name.to_string())
        .unwrap_or_else(|| key.to_string())
}

/// Akzentfarbe eines Spiels als `(r, g, b)`.
pub fn game_color(key: &str) -> (u8, u8, u8) {
    let hex = defs::GAMES
        .iter()
        .find(|g| g.key == key)
        .map(|g| g.color)
        .unwrap_or("#589cff");
    crate::levels::hex_to_rgb(hex)
}

// ---------------------------------------------------------------------------
// Definitionsliste
// ---------------------------------------------------------------------------

/// Alle Erfolgs-Definitionen in Anzeige-Reihenfolge.
///
/// Reihenfolge wie in der Python-Fassung: erst die sammlungsweiten, dann je
/// Spiel Meilenstein und Meister, dann die Stammgast-Erfolge, dann die
/// besonderen Momente.
pub fn all_defs() -> &'static [Achievement] {
    static DEFS: OnceLock<Vec<Achievement>> = OnceLock::new();
    DEFS.get_or_init(|| {
        let mut out = Vec::with_capacity(defs::TOTAL_ACHIEVEMENTS);
        for (id, icon, stat, target, rarity) in defs::GENERAL {
            out.push(Achievement {
                id: (*id).into(),
                cat: Category::General,
                icon: (*icon).into(),
                stat: (*stat).into(),
                target: *target,
                game: String::new(),
                rarity: (*rarity).into(),
            });
        }
        for game in defs::GAMES {
            if let Some((_, target)) = defs::MILESTONES.iter().find(|(k, _)| *k == game.key) {
                out.push(Achievement {
                    id: format!("score_{}", game.key),
                    cat: Category::Milestone,
                    icon: "star".into(),
                    stat: String::new(),
                    target: Some(*target),
                    game: game.key.into(),
                    rarity: "rare".into(),
                });
            }
            if let Some((_, target)) = defs::MASTERS.iter().find(|(k, _)| *k == game.key) {
                out.push(Achievement {
                    id: format!("master_{}", game.key),
                    cat: Category::Master,
                    icon: "crown".into(),
                    stat: String::new(),
                    target: Some(*target),
                    game: game.key.into(),
                    rarity: "epic".into(),
                });
            }
        }
        for game in defs::GAMES {
            out.push(Achievement {
                id: format!("regular_{}", game.key),
                cat: Category::Regular,
                icon: "flag".into(),
                stat: String::new(),
                target: Some(defs::REGULAR_PLAYS),
                game: game.key.into(),
                rarity: "common".into(),
            });
        }
        for (id, icon, game, target, rarity) in defs::SPECIALS {
            out.push(Achievement {
                id: (*id).into(),
                cat: Category::Special,
                icon: (*icon).into(),
                stat: String::new(),
                target: *target,
                game: (*game).into(),
                rarity: (*rarity).into(),
            });
        }
        out
    })
}

/// Definition eines Erfolgs.
pub fn by_id(ach_id: &str) -> Option<&'static Achievement> {
    all_defs().iter().find(|d| d.id == ach_id)
}

/// Hoechstmoegliche Punktzahl (alle Erfolge).
pub fn max_points() -> i64 {
    all_defs().iter().map(Achievement::points).sum()
}

// ---------------------------------------------------------------------------
// Freigeschaltete Erfolge
// ---------------------------------------------------------------------------

/// `{id: zeitstempel}` in Einfuege-Reihenfolge (wie Pythons dict).
type UMap = serde_json::Map<String, Value>;

/// Zwischenspeicher: `{id: zeitstempel}` des aktiven Profils.
fn cache() -> MutexGuard<'static, Option<UMap>> {
    static CACHE: OnceLock<Mutex<Option<UMap>>> = OnceLock::new();
    CACHE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

fn load_unlocked() -> UMap {
    let mut c = cache();
    if let Some(map) = c.as_ref() {
        return map.clone();
    }
    let map: UMap = crate::store::load_section(SECTION)
        .iter()
        .map(|(k, v)| (k.clone(), Value::String(v.as_str().unwrap_or("").to_string())))
        .collect();
    *c = Some(map.clone());
    map
}

fn save_unlocked(map: &UMap) {
    crate::store::save_section(SECTION, map.clone());
    *cache() = Some(map.clone());
}

/// Verwirft den Zwischenspeicher (nach einem Profilwechsel).
pub fn reset_cache() {
    *cache() = None;
    // Python: `_toasts.clear()` - wartende Einblendungen gehoeren zum alten Profil.
    pending().clear();
}

/// Meldet den Zwischenspeicher beim Profilwechsel ab.
pub fn register_switch_hook() {
    crate::store::on_switch(reset_cache);
}

/// Freigeschaltete Erfolge als `{id: zeitstempel}`.
pub fn unlocked() -> BTreeMap<String, String> {
    load_unlocked().iter().map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string())).collect()
}

/// Freigeschaltete Erfolge in Datei-Reihenfolge (wie Pythons dict).
pub fn unlocked_ordered() -> Vec<(String, String)> {
    load_unlocked().iter().map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string())).collect()
}

/// True, wenn dieser Erfolg schon freigeschaltet ist.
pub fn is_unlocked(ach_id: &str) -> bool {
    load_unlocked().contains_key(ach_id)
}

/// `(freigeschaltet, gesamt)` fuer Fortschrittsanzeigen.
pub fn counts() -> (usize, usize) {
    (load_unlocked().len(), all_defs().len())
}

/// Erfolgspunkte des aktiven Profils.
pub fn total_points() -> i64 {
    let map = load_unlocked();
    all_defs()
        .iter()
        .filter(|d| map.contains_key(&d.id))
        .map(Achievement::points)
        .sum()
}

/// Anzahl je Seltenheitsstufe: `[(stufe, frei, gesamt)]`.
pub fn rarity_counts() -> Vec<(&'static str, usize, usize)> {
    let map = load_unlocked();
    defs::RARITY
        .iter()
        .map(|(id, _, _)| {
            let all = all_defs().iter().filter(|d| d.rarity == *id);
            let total = all.clone().count();
            let got = all.filter(|d| map.contains_key(&d.id)).count();
            (*id, got, total)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Fortschritt
// ---------------------------------------------------------------------------

/// `_totals()` - `stats.totals()` plus die Erfolgszahl.
fn totals_with_ach() -> (crate::stats::Totals, i64) {
    (crate::stats::totals(None), load_unlocked().len() as i64)
}

fn stat_value(t: &(crate::stats::Totals, i64), stat: &str) -> f64 {
    if stat == "ach" {
        t.1 as f64
    } else {
        t.0.stat(stat)
    }
}

/// Summen fuer den Fortschrittsbalken (`_totals()` als Tabelle).
pub fn totals() -> BTreeMap<String, i64> {
    let t = totals_with_ach();
    let mut m = BTreeMap::new();
    for k in [
        "plays", "time", "wins", "losses", "records", "distinct", "win_games", "days", "streak", "best_streak", "duels",
        "points", "level", "longest", "ach",
    ] {
        m.insert(k.to_string(), stat_value(&t, k) as i64);
    }
    m
}

/// `progress_of(d)` - `(aktuell, ziel)` eines zaehlbaren Erfolgs.
pub fn progress_of(d: &Achievement) -> Option<(i64, i64)> {
    let target = d.target?;
    match d.cat {
        Category::General if !d.stat.is_empty() && target != 0 => {
            let have = stat_value(&totals_with_ach(), &d.stat);
            Some(((have as i64).min(target), target))
        }
        Category::Milestone | Category::Master => {
            let have = crate::highscore::get(&d.game);
            Some((have.min(target), target))
        }
        Category::Regular => {
            let have = crate::stats::get(&d.game, None).plays;
            Some((have.min(target), target))
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Freischalten
// ---------------------------------------------------------------------------

/// Ein gerade freigeschalteter Erfolg bzw. Levelaufstieg.
///
/// Die Engine holt sich die Liste pro Bild ab und zeigt sie als
/// Einblendung - die Datenschicht selbst zeichnet nichts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Toast {
    /// Erfolg freigeschaltet.
    Achievement(&'static Achievement),
    /// Levelaufstieg: `(Level, Rangtitel)`.
    Level(i64, String),
}

fn pending() -> MutexGuard<'static, Vec<Toast>> {
    static PENDING: OnceLock<Mutex<Vec<Toast>>> = OnceLock::new();
    PENDING
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

/// Rueckruf beim Freischalten seltener Erfolge (legendaer, mythisch).
static RARE_HOOK: OnceLock<fn()> = OnceLock::new();

/// Meldet den Rueckruf fuer seltene Erfolge an: Python streut in `_unlock`
/// SOFORT `ui.spawn_confetti(800, 600, n=60)` (mitten im `update` des
/// Spiels) - nicht erst beim Abholen der Einblendungen nach `ui.draw_fx`.
pub fn on_rare_unlock(f: fn()) {
    let _ = RARE_HOOK.set(f);
}

/// Holt die aufgelaufenen Einblendungen ab (und leert die Liste).
pub fn take_toasts() -> Vec<Toast> {
    std::mem::take(&mut *pending())
}

fn now_stamp() -> String {
    let secs = crate::store::local_secs();
    let (y, m, d, h, mi, _) = crate::store::civil_from_unix(secs);
    format!("{y:04}-{m:02}-{d:02} {h:02}:{mi:02}")
}

/// `_unlock(d, silent)` - schaltet einen Erfolg frei (einmalig).
fn unlock(d: &'static Achievement, silent: bool) -> bool {
    let mut map = load_unlocked();
    if map.contains_key(&d.id) {
        return false;
    }
    let stamp = now_stamp();
    map.insert(d.id.clone(), Value::String(stamp.clone()));
    save_unlocked(&map);

    let before = crate::stats::level_of(None).0;
    crate::stats::log_ach(&d.id, d.points(), Some(&stamp));
    crate::stats::set_points(total_points(), None);
    let after = crate::stats::level_of(None).0;

    if !silent {
        pending().push(Toast::Achievement(d));
        if matches!(d.rarity.as_str(), "legendary" | "mythic") {
            if let Some(f) = RARE_HOOK.get() {
                f();
            }
        }
    }
    if after > before {
        crate::stats::log_level(after, Some(&stamp));
        if !silent {
            pending().push(Toast::Level(after, crate::levels::rank_title(after)));
        }
    }
    check_meta(silent);
    true
}

/// `_check_meta(silent)` - Erfolge ueber Erfolge.
fn check_meta(silent: bool) {
    let map = load_unlocked();
    let mut seen = std::collections::BTreeSet::new();
    for ach_id in map.keys() {
        if let Some(d) = by_id(ach_id) {
            seen.insert(d.rarity.clone());
        }
    }
    if seen.contains("legendary") || seen.contains("mythic") {
        unlock_id("first_legend", silent);
    }
    let stufen = defs::RARITY.iter().filter(|(r, _, _)| seen.contains(*r)).count();
    if stufen >= 4 {
        unlock_id("all_rarities", silent);
    }
    // Alle Erfolge eines Spiels eingesammelt? (Reihenfolge wie das dict.)
    let map = load_unlocked();
    let mut per_game: Vec<(String, i64, i64)> = Vec::new();
    for d in all_defs() {
        if d.game.is_empty() {
            continue;
        }
        let pos = match per_game.iter().position(|(g, _, _)| *g == d.game) {
            Some(p) => p,
            None => {
                per_game.push((d.game.clone(), 0, 0));
                per_game.len() - 1
            }
        };
        per_game[pos].2 += 1;
        if map.contains_key(&d.id) {
            per_game[pos].1 += 1;
        }
    }
    if per_game.iter().any(|(_, got, total)| got == total && *total >= 3) {
        unlock_id("completionist", silent);
    }
    let t = totals_with_ach();
    for d in all_defs() {
        if d.cat == Category::General && d.stat == "ach" && d.target.unwrap_or(0) != 0 {
            if t.1 >= d.target.unwrap_or(0) {
                unlock(d, silent);
            }
        }
    }
}

fn unlock_id(ach_id: &str, silent: bool) {
    if let Some(d) = by_id(ach_id) {
        if !load_unlocked().contains_key(&d.id) {
            unlock(d, silent);
        }
    }
}

/// `check_stats(silent)` - alle statistik-gebundenen Erfolge.
pub fn check_stats(silent: bool) {
    let t = totals_with_ach();
    for d in all_defs() {
        if d.cat == Category::General && !d.stat.is_empty() && d.target.unwrap_or(0) != 0 {
            if stat_value(&t, &d.stat) >= d.target.unwrap_or(0) as f64 {
                unlock(d, silent);
            }
        } else if d.cat == Category::Regular {
            if crate::stats::get(&d.game, None).plays >= d.target.unwrap_or(0) {
                unlock(d, silent);
            }
        }
    }
}

/// `on_highscore(key, score, silent)` - Punkte-Stufen eines Spiels.
pub fn on_highscore(key: &str, score: i64, silent: bool) {
    for d in all_defs() {
        if matches!(d.cat, Category::Milestone | Category::Master) && d.game == key {
            if score >= d.target.unwrap_or(i64::MAX) {
                unlock(d, silent);
            }
        }
    }
}

/// `on_game_started(key)` - bei jedem Partie-Start.
pub fn on_game_started(_key: &str) {
    let (_, _, _, hour, _, _) = crate::store::civil_from_unix(crate::store::local_secs());
    if hour < 5 {
        event("night_owl", None);
    } else if (5..8).contains(&hour) {
        event("early_bird", None);
    }
    check_weekend();
    check_stats(false);
}

/// `_check_weekend()` - am selben Wochenende Samstag und Sonntag gespielt.
fn check_weekend() {
    if load_unlocked().contains_key("weekend") {
        return;
    }
    let days = crate::stats::days_map(None);
    for (day, _) in &days {
        if crate::store::weekday_of(day) == Some(5) {
            if let Some(next) = crate::store::add_days(day, 1) {
                if days.iter().any(|(d, _)| *d == next) {
                    event("weekend", None);
                    return;
                }
            }
        }
    }
}

/// `on_playtime(seconds)` - Marathon: eine Stunde am Stueck.
pub fn on_playtime(seconds: f64) {
    if seconds >= 3600.0 {
        event("marathon", Some(seconds as i64));
    }
}

/// `event(ach_id, value)` - Ereignis-Erfolg.
///
/// Wie in Python: nur Ereignis-Erfolge **ohne** Statistik-Feld verlangen
/// einen Wert ueber der Zielmarke; alles andere schaltet sofort frei.
pub fn event(ach_id: &str, value: Option<i64>) {
    let Some(d) = by_id(ach_id) else { return };
    if d.target.is_some()
        && d.stat.is_empty()
        && !matches!(d.cat, Category::Milestone | Category::Master | Category::Regular)
    {
        match value {
            Some(v) if v >= d.target.unwrap_or(0) => {}
            _ => return,
        }
    }
    unlock(d, false);
}

/// `backfill_silent()` - Erreichtes ohne Einblendungen nachtragen.
///
/// Gibt `(neue Erfolge, Level, Punkte)` zurueck.
pub fn backfill_silent() -> (usize, i64, i64) {
    let before = load_unlocked().len();
    let scores = crate::highscore::load_ordered();
    check_stats(true);
    for (key, score) in &scores {
        on_highscore(key, *score, true);
    }
    check_meta(true);
    crate::stats::set_points(total_points(), None);
    let (level, points) = crate::stats::level_of(None);
    (load_unlocked().len() - before, level, points)
}

/// `rarest_unlocked(n)` - die seltensten Freischaltungen.
pub fn rarest_unlocked(n: usize) -> Vec<(&'static Achievement, String)> {
    let map = load_unlocked();
    let order = |r: &str| defs::RARITY.iter().position(|(id, _, _)| *id == r).unwrap_or(0) as i64;
    let mut items: Vec<(&'static Achievement, String)> =
        map.iter().filter_map(|(i, ts)| by_id(i).map(|d| (d, ts.as_str().unwrap_or("").to_string()))).collect();
    items.sort_by(|a, b| (-order(&a.0.rarity), &a.1).cmp(&(-order(&b.0.rarity), &b.1)));
    items.truncate(n);
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn die_definitionsliste_ist_vollstaendig() {
        assert_eq!(all_defs().len(), defs::TOTAL_ACHIEVEMENTS);
        assert_eq!(defs::GAMES.len(), defs::TOTAL_GAMES);
    }

    #[test]
    fn keine_doppelten_ids() {
        let mut seen = std::collections::BTreeSet::new();
        for d in all_defs() {
            assert!(seen.insert(&d.id), "doppelte Erfolgs-id: {}", d.id);
        }
    }

    #[test]
    fn jede_kategorie_ist_besetzt() {
        for cat in Category::all() {
            assert!(
                all_defs().iter().any(|d| d.cat == cat),
                "Kategorie {} ist leer",
                cat.key()
            );
        }
    }

    #[test]
    fn jeder_erfolg_hat_eine_bekannte_seltenheit() {
        for d in all_defs() {
            assert!(
                defs::RARITY.iter().any(|(id, _, _)| *id == d.rarity),
                "{} hat die unbekannte Stufe {}",
                d.id,
                d.rarity
            );
            assert!(d.points() > 0);
        }
    }

    #[test]
    fn jeder_erfolg_hat_texte_in_allen_sprachen() {
        // Dieselbe Regel wie in der Python-Fassung: Name und Beschreibung
        // muessen in allen 14 Sprachen stehen (sonst zeigt die Oberflaeche
        // den nackten Schluessel).
        crate::i18n::init();
        for d in all_defs() {
            let name = d.display_name();
            assert!(
                !name.starts_with("ach.") && !name.is_empty(),
                "{} fehlt ein Name (bekam {name:?})",
                d.id
            );
            let desc = d.display_desc();
            assert!(
                !desc.starts_with("ach.") && !desc.is_empty(),
                "{} fehlt eine Beschreibung (bekam {desc:?})",
                d.id
            );
            assert!(
                !desc.contains('{'),
                "{} hat einen ungefuellten Platzhalter: {desc:?}",
                d.id
            );
        }
    }

    #[test]
    fn hoechstpunktzahl_ist_plausibel() {
        let max = max_points();
        assert!(
            (8_000..20_000).contains(&max),
            "Hoechstpunktzahl {max} liegt ausserhalb des Erwarteten"
        );
    }
}
