//! profiles.rs
//! ===========
//! Spielerprofile - mehrere Spieler an EINEM Geraet. Gegenstueck zu
//! `profiles.py`.
//!
//! Jedes Profil hat eigene Highscores, Erfolge, Statistiken und Spielstaende.
//! Technisch liegt das alles in `mem.json` unter `players/<id>` (siehe
//! [`crate::store`]) bzw. in `stats.pgzlamastats`; gemeinsam bleiben nur die
//! Geraete-Einstellungen (`settings.json`) und die Sprache der Oberflaeche.

use crate::store::{self, ProfileMeta};

/// Profil-Platzhalter fuer "kein Profil" (Spieler 2 spielt als Gast).
pub const GUEST: &str = "guest";

/// Auswaehlbare Symbole.
pub const ICONS: &[&str] = &[
    "llama", "star", "crown", "gem", "heart", "rocket", "moon", "flame", "dice", "target", "medal",
    "check",
];

/// Auswaehlbare Profilfarben (Index = `color` in den Kopfdaten).
pub const COLORS: &[&str] = &[
    "#6ecd8c", "#589cff", "#f0a05a", "#b07fe8", "#5ad4d4", "#f5cd64", "#e05a7d", "#4fd17a",
    "#c86ad8", "#e2725b", "#35e2ff", "#b9f23a",
];

/// Laengster erlaubter Profilname (Anzeige in Sidebar/HUD bleibt lesbar).
pub const MAX_NAME: usize = 14;

/// Warum ein Profilname abgelehnt wurde. Die Oberflaeche zeigt
/// `t("profile.err_<code>")`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameError {
    /// Nichts eingetippt.
    Empty,
    /// Laenger als [`MAX_NAME`].
    Long,
    /// Vom Wortfilter beanstandet.
    Dirty,
    /// Ein anderes Profil heisst schon so.
    Duplicate,
    /// Es passen keine weiteren Profile mehr hinein.
    Full,
}

impl NameError {
    /// Der Code, der in den i18n-Schluessel wandert (`profile.err_<code>`).
    pub fn code(self) -> &'static str {
        match self {
            NameError::Empty => "empty",
            NameError::Long => "long",
            NameError::Dirty => "dirty",
            NameError::Duplicate => "dup",
            NameError::Full => "full",
        }
    }

    /// Fertige, uebersetzte Fehlermeldung.
    pub fn message(self) -> String {
        crate::i18n::t(&format!("profile.err_{}", self.code()))
    }
}

// ----- Abfragen -----------------------------------------------------------

/// id des aktiven Profils.
pub fn active_id() -> String {
    store::active_id()
}

/// `[(id, Kopfdaten), ...]` in Anzeige-Reihenfolge.
pub fn all() -> Vec<(String, ProfileMeta)> {
    store::all_profiles()
}

/// Anzahl angelegter Profile.
pub fn count() -> usize {
    store::profile_ids().len()
}

/// Kopfdaten eines Profils (`None` = aktives).
pub fn meta(pid: Option<&str>) -> ProfileMeta {
    store::profile_meta(pid)
}

/// Anzeigename eines Profils (leerer Name -> "Spieler N").
pub fn name_of(pid: Option<&str>) -> String {
    if pid == Some(GUEST) {
        return crate::i18n::t("profile.guest");
    }
    let meta = store::profile_meta(pid);
    let name = meta.name.trim();
    if !name.is_empty() {
        return name.to_string();
    }
    let ids = store::profile_ids();
    let id = pid.map(str::to_string).unwrap_or_else(store::active_id);
    let n = ids.iter().position(|x| *x == id).map(|i| i + 1).unwrap_or(1);
    crate::i18n::tf("profile.player_n", &[("n", &n.to_string())])
}

/// Symbol eines Profils.
pub fn icon_of(pid: Option<&str>) -> String {
    if pid == Some(GUEST) {
        return "person".into();
    }
    let icon = store::profile_meta(pid).icon;
    if ICONS.contains(&icon.as_str()) || icon == "person" {
        icon
    } else {
        "llama".into()
    }
}

/// Profilfarbe als Hex-Zeichenkette.
pub fn color_hex(pid: Option<&str>) -> String {
    if pid == Some(GUEST) {
        return "#9aa6bf".into();
    }
    let idx = store::profile_meta(pid).color.max(0) as usize;
    COLORS[idx % COLORS.len()].to_string()
}

/// Profilfarbe als `(r, g, b)`.
pub fn color_rgb(pid: Option<&str>) -> (u8, u8, u8) {
    crate::levels::hex_to_rgb(&color_hex(pid))
}

/// `(Level, Punkte)` des Profils aus der Statistik-Datei.
pub fn level_of(pid: Option<&str>) -> (i64, i64) {
    crate::stats::level_of(pid)
}

// ----- Namen pruefen ------------------------------------------------------

/// Prueft einen Profilnamen. `pid` ist das Profil, das umbenannt wird (sein
/// eigener Name gilt nicht als Dublette).
pub fn check_name(name: &str, pid: Option<&str>) -> Result<String, NameError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(NameError::Empty);
    }
    if name.chars().count() > MAX_NAME {
        return Err(NameError::Long);
    }
    if !crate::swear::is_clean(name) {
        return Err(NameError::Dirty);
    }
    let lower = name.to_lowercase();
    for (other, meta) in store::all_profiles() {
        if Some(other.as_str()) == pid {
            continue;
        }
        if meta.name.trim().to_lowercase() == lower {
            return Err(NameError::Duplicate);
        }
    }
    Ok(name.to_string())
}

// ----- Aendern ------------------------------------------------------------

/// Legt ein Profil an und gibt dessen id zurueck.
pub fn create(name: &str, icon: &str, color: i64) -> Result<String, NameError> {
    let name = check_name(name, None)?;
    if count() >= store::MAX_PROFILES {
        return Err(NameError::Full);
    }
    let Some(pid) = store::create_profile(&name, icon, color) else {
        return Err(NameError::Full);
    };
    // Erfolge "Gesellschaftsspieler" / "Familienbande".
    let n = count() as i64;
    crate::achievements::event("profile_second", Some(n));
    crate::achievements::event("profile_family", Some(n));
    Ok(pid)
}

/// Benennt ein Profil um.
pub fn rename(pid: &str, name: &str) -> Result<(), NameError> {
    let name = check_name(name, Some(pid))?;
    store::set_meta(pid, |m| m.name = name);
    Ok(())
}

/// Aendert Symbol und/oder Farbe eines Profils.
pub fn set_look(pid: &str, icon: Option<&str>, color: Option<i64>) {
    store::set_meta(pid, |m| {
        if let Some(icon) = icon {
            m.icon = icon.to_string();
        }
        if let Some(color) = color {
            m.color = color;
        }
    });
}

/// Loescht ein Profil samt Daten (das letzte Profil bleibt bestehen).
pub fn delete(pid: &str) -> bool {
    if count() <= 1 {
        return false;
    }
    crate::stats::flush();
    crate::stats::drop_profile(pid);
    crate::matches::drop_profile(pid);
    store::delete_profile(pid)
}

/// Wechselt das aktive Profil (schreibt vorher alles Offene weg).
pub fn switch(pid: &str) -> bool {
    if pid == store::active_id() {
        return false;
    }
    crate::stats::flush();
    let ok = store::set_active(pid);
    if ok {
        store::touch(Some(pid));
    }
    ok
}

/// Merkt sich den letzten Besuch eines Profils.
pub fn touch(pid: Option<&str>) {
    store::touch(pid);
}
