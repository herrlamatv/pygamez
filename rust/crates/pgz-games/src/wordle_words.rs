//! wordle_words.rs
//! ===============
//! Wortlisten fuer Wordle, je Sprache und Wortlaenge (4 bis 7).
//!
//! Uebertragen aus `games/wordle_words.py`. Die Listen liegen als Textdateien
//! im Ordner `woordlistz/` des Projekts (`<sprache>/answers.txt`,
//! `allowed.txt`, `answers4.txt` ...). Gesucht wird neben der .exe und in den
//! Ordnern darueber (Entwicklung: `rust.output/release/` -> Projektordner).
//! Fehlt der Ordner, greift fuer 5 Buchstaben die eingebaute Notfallliste.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::Rc;

/// Wortlaengen, die das Spiel anbietet (5 = klassisches Wordle).
pub const LENGTHS: [usize; 4] = [4, 5, 6, 7];
/// Klassische Wortlaenge.
pub const DEFAULT_LENGTH: usize = 5;

/// Notfall-Loesungswoerter, falls die Dateien fehlen (aus wordle_words.py erzeugt).
const FALLBACK: &[(&str, &[&str])] = &[    ("de", &[
        "HAUSE", "TISCH", "STUHL", "LAMPE", "APFEL", "BIRNE", "PFERD", "KATZE", "MAUER", "WOLKE",
        "REGEN", "SONNE", "STERN", "BLUME", "BLATT", "BAUER", "TASSE", "KANNE", "GABEL", "HONIG",
        "ROSEN", "TULPE", "HECKE", "BUSCH", "FLUSS", "INSEL", "STADT", "BERGE", "WIESE", "BUCHE",
        "EICHE", "TANNE", "LINDE", "AHORN", "ZWEIG", "KRONE", "RINDE", "BEERE", "GURKE", "PILZE",
        "BOHNE", "ERBSE", "LINSE", "NUDEL", "SUPPE", "TORTE", "KEKSE", "SAHNE", "QUARK", "ESSIG",
        "CHILI", "CURRY", "REISE", "NADEL", "FADEN", "WOLLE", "STOFF", "HOSEN", "JACKE", "SOCKE",
        "RINGE", "KETTE", "PERLE", "EISEN", "STAHL", "STEIN", "FARBE", "SEITE", "ZEILE", "WORTE",
        "BRIEF", "KARTE", "STIFT", "TINTE", "TAFEL", "KREIS", "ECKEN", "KANTE", "LINIE", "PUNKT",
        "SUMME", "REGEL", "PROBE", "MONDE", "NEBEL", "STURM", "BLITZ", "FROST", "WINDE", "FEUER",
        "ASCHE", "KOHLE", "RAUCH", "DAMPF", "FUNKE", "LICHT", "MILCH", "HAFER", "KRAUT", "SPECK",
        "WURST", "STEAK", "GRILL", "HERDE", "SALAT", "PIZZA", "KAKAO", "MOKKA", "LATTE",
    ]),
    ("en", &[
        "APPLE", "BREAD", "CHAIR", "TABLE", "HOUSE", "MOUSE", "LIGHT", "NIGHT", "WATER", "EARTH",
        "PLANT", "STONE", "RIVER", "OCEAN", "BEACH", "CLOUD", "STORM", "SUNNY", "HAPPY", "ANGRY",
        "QUIET", "BRAVE", "SMART", "QUICK", "SWEET", "SPICY", "FRESH", "GREEN", "BROWN", "BLACK",
        "WHITE", "GRAPE", "LEMON", "MELON", "PEACH", "BERRY", "HONEY", "SUGAR", "FLOUR", "DOUGH",
        "PASTA", "PIZZA", "SALAD", "JUICE", "DRINK", "GLASS", "PLATE", "SPOON", "KNIFE", "CLOTH",
        "SHIRT", "PANTS", "SHOES", "SOCKS", "DRESS", "SCARF", "GLOVE", "WATCH", "RINGS", "CHAIN",
        "PEARL", "METAL", "STEEL", "BRICK", "PAPER", "PAINT", "BRUSH", "CHALK", "BOARD", "POINT",
        "ANGLE", "ROUND", "HEART", "SMILE", "LAUGH", "DREAM", "SLEEP", "AWAKE", "HORSE", "SHEEP",
        "GOOSE", "TIGER", "ZEBRA", "PANDA", "KOALA", "SNAKE", "EAGLE", "ROBIN", "WHALE", "SHARK",
        "TROUT", "GRASS", "BLOOM", "PETAL", "THORN", "FRUIT", "MAPLE", "BIRCH", "CEDAR", "ROCKS",
        "SANDY", "FIELD", "CANDY", "MONEY", "MUSIC", "PIANO", "DRAMA", "STAGE", "NOVEL", "STORY",
        "WORDS", "LINES",
    ]),
    ("fr", &[
        "TABLE", "LIVRE", "PORTE", "ARBRE", "FLEUR", "PLAGE", "NUAGE", "ORAGE", "PLUIE", "NEIGE",
        "TERRE", "MONDE", "ROUTE", "VILLE", "OCEAN", "GRAIN", "POMME", "POIRE", "MELON", "SUCRE",
        "PIZZA", "VERRE", "NAPPE", "VESTE", "GANTS", "BAGUE", "PERLE", "ACIER", "CRAIE", "LIGNE",
        "POINT", "CARRE", "COEUR", "TIGRE", "ZEBRE", "PANDA", "AIGLE", "HERBE", "EPINE", "FRUIT",
        "VIGNE", "CHIEN", "LOUPS", "CHATS", "BLEUE", "VERTE", "NOIRE", "ROUGE", "JAUNE", "BRUNE",
        "NUITS", "MATIN", "HIVER", "LUNDI", "MARDI", "AMOUR", "AMIES", "PERES", "MERES", "HEURE",
        "ANNEE", "PLACE", "SALLE", "MAINS", "PIEDS", "TETES", "DENTS", "JOUES", "LEVRE", "GORGE",
        "DOIGT", "POUCE", "GENOU", "TALON", "PIANO", "DANSE", "CHANT", "SCENE", "DRAME", "ROMAN",
        "CONTE",
    ]),
    ("es", &[
        "SILLA", "LIBRO", "ARBOL", "PLAYA", "NIEVE", "MUNDO", "GRANO", "LIMON", "MELON", "FRESA",
        "PASTA", "PIZZA", "PLATO", "FALDA", "PERLA", "ACERO", "PAPEL", "LINEA", "PUNTO", "CARRO",
        "SUENO", "OVEJA", "PERRO", "TIGRE", "CEBRA", "PANDA", "TRIGO", "FRUTA", "PARRA", "NOCHE",
        "TARDE", "LUNES", "VERDE", "NEGRO", "AMIGO", "PADRE", "MADRE", "NINOS", "FELIZ", "LENTO",
        "DULCE", "CIELO", "FUEGO", "CALOR", "RITMO", "PIANO", "CANTO", "BAILE", "DRAMA", "TEXTO",
        "GATOS", "PATOS", "OSITO", "LOBOS", "PECES", "HOJAS", "MONTE", "VALLE", "CAMPO", "PRADO",
        "NUBES", "SOLES", "MARES", "ARENA", "ROCAS", "BARCO", "COCHE", "AVION", "CALLE", "PLAZA",
        "TORRE", "MUROS", "TECHO", "SUELO", "MESAS", "CAMAS", "SOFAS", "VELAS", "ROSAS",
    ]),
    ("pt", &[
        "LIVRO", "PORTA", "PRAIA", "NUVEM", "CHUVA", "TERRA", "MUNDO", "LIMAO", "MELAO", "FRUTA",
        "PASTA", "PIZZA", "PRATO", "PAPEL", "LINHA", "PONTO", "CARRO", "SONHO", "TIGRE", "ZEBRA",
        "PANDA", "TRIGO", "VERDE", "PRETO", "NOITE", "TARDE", "MANHA", "VENTO", "CALOR", "RITMO",
        "PIANO", "VIOLA", "CANTO", "DANCA", "DRAMA", "CAMPO", "MONTE", "AMIGO", "PONTE", "FESTA",
        "LEITE", "PEIXE", "CARNE", "ARROZ", "SALSA", "MOLHO", "VINHO", "MASSA", "FORNO", "FOGAO",
        "GATOS", "PATOS", "LOBOS", "FOLHA", "PEDRA", "VALES", "PRADO", "MARES", "AREIA", "ROCHA",
        "BARCO", "AVIAO", "PRACA", "TORRE", "MUROS", "CAMAS", "SOFAS", "VELAS", "ROSAS", "RELVA",
        "LAGOA", "ILHAS", "AGUAS", "NEVOA",
    ]),
];

/// (Loesungswoerter, erlaubte Rateworte) einer Sprache und Laenge.
pub type Lists = Rc<(Vec<String>, HashSet<String>)>;

thread_local! {
    static CACHE: RefCell<HashMap<(String, usize), Lists>> = RefCell::new(HashMap::new());
}

/// Kandidaten fuer den Ordner `woordlistz` (`_BASES`).
fn bases() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().map(|p| p.to_path_buf());
        for _ in 0..4 {
            let Some(d) = dir else {
                break;
            };
            out.push(d.join("woordlistz"));
            dir = d.parent().map(|p| p.to_path_buf());
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        out.push(cwd.join("woordlistz"));
    }
    out
}

/// `_valid(word, length)` - genau `length` Grossbuchstaben A-Z.
fn valid(word: &str, length: usize) -> bool {
    word.chars().count() == length && word.is_ascii() && word.chars().all(|c| c.is_ascii_alphabetic())
}

/// `file_names(length)` - (Loesungswoerter, Rateworte) als Dateinamen.
fn file_names(length: usize) -> (String, String) {
    if length == DEFAULT_LENGTH {
        return ("answers".into(), "allowed".into());
    }
    (format!("answers{length}"), format!("allowed{length}"))
}

/// `_read_list(lang, name, length)`
fn read_list(lang: &str, name: &str, length: usize) -> Vec<String> {
    for base in bases() {
        let path = base.join(lang).join(format!("{name}.txt"));
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for line in text.lines() {
            let word = line.trim().to_uppercase();
            if valid(&word, length) && !seen.contains(&word) {
                seen.insert(word.clone());
                out.push(word);
            }
        }
        if !out.is_empty() {
            return out;
        }
    }
    Vec::new()
}

/// `_load(lang, length)`
fn load(lang: &str, length: usize) -> Lists {
    let key = (lang.to_string(), length);
    if let Some(l) = CACHE.with(|c| c.borrow().get(&key).cloned()) {
        return l;
    }
    let (ans_name, all_name) = file_names(length);
    let mut answers = read_list(lang, &ans_name, length);
    let mut allowed: HashSet<String> = read_list(lang, &all_name, length).into_iter().collect();
    if answers.is_empty() && length == DEFAULT_LENGTH {
        let fb = FALLBACK.iter().find(|(l, _)| *l == lang).or_else(|| FALLBACK.iter().find(|(l, _)| *l == "en"));
        answers = fb.map(|(_, w)| w.iter().filter(|w| valid(w, DEFAULT_LENGTH)).map(|w| w.to_string()).collect()).unwrap_or_default();
    }
    answers.sort();
    allowed.extend(answers.iter().cloned());
    let l = Rc::new((answers, allowed));
    CACHE.with(|c| c.borrow_mut().insert(key, l.clone()));
    l
}

/// `words_for(lang, length)` - Loesungswoerter (alphabetisch).
pub fn words_for(lang: &str, length: usize) -> Lists {
    load(lang, length)
}

/// `has_length(lang, length)`
pub fn has_length(lang: &str, length: usize) -> bool {
    !load(lang, length).0.is_empty()
}

/// `is_allowed(lang, word)`
pub fn is_allowed(lang: &str, word: &str) -> bool {
    let w = word.to_uppercase();
    load(lang, w.chars().count()).1.contains(&w)
}

/// `counts_for(lang, length)` - (Loesungswoerter, Rateworte).
pub fn counts_for(lang: &str, length: usize) -> (usize, usize) {
    let l = load(lang, length);
    (l.0.len(), l.1.len())
}