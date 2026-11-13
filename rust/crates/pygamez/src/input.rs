//! input.rs
//! ========
//! Von minifb zu `InputEvent` mit **Tkinter-Tastennamen**.
//!
//! Die Python-Fassung bekommt ihre Eingaben aus Tkinter, und alles, was
//! Tasten speichert oder vergleicht, benutzt Tks `keysym`: `settings.json`
//! (`"Up"`, `"w"`, `"space"`), die Spiele (`event.key in ("Return",
//! "space")`), das Eingabefeld. Diese Datei bildet genau diese Namen nach.
//!
//! **Woher das Zeichen kommt.** Bei Buchstaben, Ziffern und Satzzeichen
//! nimmt Tk das *getippte* Zeichen: Umschalt+w ist `"W"`, die Taste rechts
//! neben dem L ist auf deutscher Tastatur `"odiaeresis"`. minifb meldet die
//! Zeichen getrennt von den Tasten (ein Rueckruf); sie werden der Reihe nach
//! den zeichenerzeugenden Tasten zugeordnet - wie in rustgames
//! `minifb_backend.rs`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use minifb::{Key, KeyRepeat, Window};

/// Sammelt die Zeichen aus minifbs Rueckruf.
#[derive(Default)]
pub struct Zeichen {
    pub liste: Rc<RefCell<Vec<char>>>,
}

impl minifb::InputCallback for Zeichen {
    fn add_char(&mut self, uni: u32) {
        if let Some(c) = char::from_u32(uni) {
            self.liste.borrow_mut().push(c);
        }
    }
}

/// Eine Tastenmeldung aus einem Bild.
#[derive(Debug, Clone)]
pub struct Taste {
    pub keysym: String,
    pub ch: String,
    pub repeat: bool,
    pub down: bool,
}

/// Merkt sich gehaltene Tasten (fuer `repeat` und das KEYUP mit dem
/// keysym des ersten Drucks - genau wie `main.py._held_keys`).
#[derive(Default)]
pub struct Tastatur {
    gehalten: HashMap<Key, String>,
    /// Dasselbe fuer die Windows-Nachrichten: (virtueller Code, Scancode).
    gehalten_win: HashMap<(u32, u32), String>,
}

/// Tks keysym fuer eine Windows-Taste (tkWinKey.c `KeycodeToKeysym`): ein
/// druckbares Zeichen bestimmt den Namen (Layout, Umschalt, AltGr: `"z"`,
/// `"Z"`, `"odiaeresis"`, `"at"`), sonst der virtuelle Tastencode.
pub fn keysym_win(vk: u32, scan: u32, zeichen: &[char]) -> String {
    if let Some(&c) = zeichen.first() {
        if (c as u32) >= 0x20 && c != '\x7f' {
            return keysym_aus_zeichen(c);
        }
    }
    let erweitert = scan & 0x100 != 0;
    let s = match vk {
        0x03 => "Cancel",
        0x08 => "BackSpace",
        0x09 => "Tab",
        0x0C => "Clear",
        0x0D => "Return",
        0x10 if scan & 0xff == 0x36 => "Shift_R",
        0x10 => "Shift_L",
        0x11 if erweitert => "Control_R",
        0x11 => "Control_L",
        0x12 if erweitert => "Alt_R",
        0x12 => "Alt_L",
        0x13 => "Pause",
        0x14 => "Caps_Lock",
        0x1B => "Escape",
        0x20 => "space",
        0x21 => "Prior",
        0x22 => "Next",
        0x23 => "End",
        0x24 => "Home",
        0x25 => "Left",
        0x26 => "Up",
        0x27 => "Right",
        0x28 => "Down",
        0x29 => "Select",
        0x2A | 0x2C => "Print",
        0x2B => "Execute",
        0x2D => "Insert",
        0x2E => "Delete",
        0x2F => "Help",
        0x5B => "Win_L",
        0x5C => "Win_R",
        0x5D => "App",
        0x60..=0x69 => return format!("KP_{}", vk - 0x60),
        0x6A => "KP_Multiply",
        0x6B => "KP_Add",
        0x6C => "KP_Separator",
        0x6D => "KP_Subtract",
        0x6E => "KP_Decimal",
        0x6F => "KP_Divide",
        0x70..=0x87 => return format!("F{}", vk - 0x6F),
        0x90 => "Num_Lock",
        0x91 => "Scroll_Lock",
        0xA0 => "Shift_L",
        0xA1 => "Shift_R",
        0xA2 => "Control_L",
        0xA3 => "Control_R",
        0xA4 => "Alt_L",
        0xA5 => "Alt_R",
        // Ziffern und Buchstaben ohne druckbares Zeichen (z.B. mit Strg):
        // der virtuelle Code ist schon der Buchstabe des Layouts.
        0x30..=0x39 => return char::from(b'0' + (vk - 0x30) as u8).to_string(),
        0x41..=0x5A => return char::from(b'a' + (vk - 0x41) as u8).to_string(),
        // Satzzeichen-Tasten ohne Zeichen: Grundzeichen des Layouts.
        _ => return crate::fenster::grundzeichen(vk).map(keysym_aus_zeichen).unwrap_or_default(),
    };
    s.to_string()
}

/// Erzeugt die Taste ein Zeichen?
fn erzeugt_zeichen(k: Key) -> bool {
    use Key::*;
    matches!(
        k,
        Key0 | Key1 | Key2 | Key3 | Key4 | Key5 | Key6 | Key7 | Key8 | Key9 | A | B | C | D | E
            | F | G | H | I | J | K | L | M | N | O | P | Q | R | S | T | U | V | W | X | Y | Z
            | Apostrophe | Backquote | Backslash | Comma | Equal | LeftBracket | Minus | Period
            | RightBracket | Semicolon | Slash | Space | NumPad0 | NumPad1 | NumPad2 | NumPad3
            | NumPad4 | NumPad5 | NumPad6 | NumPad7 | NumPad8 | NumPad9 | NumPadDot
            | NumPadSlash | NumPadAsterisk | NumPadMinus | NumPadPlus
    )
}

/// Tks keysym fuer ein getipptes Zeichen.
pub fn keysym_aus_zeichen(c: char) -> String {
    if c.is_ascii_alphanumeric() {
        return c.to_string();
    }
    let name = match c {
        ' ' => "space",
        '!' => "exclam",
        '"' => "quotedbl",
        '#' => "numbersign",
        '$' => "dollar",
        '%' => "percent",
        '&' => "ampersand",
        '\'' => "apostrophe",
        '(' => "parenleft",
        ')' => "parenright",
        '*' => "asterisk",
        '+' => "plus",
        ',' => "comma",
        '-' => "minus",
        '.' => "period",
        '/' => "slash",
        ':' => "colon",
        ';' => "semicolon",
        '<' => "less",
        '=' => "equal",
        '>' => "greater",
        '?' => "question",
        '@' => "at",
        '[' => "bracketleft",
        '\\' => "backslash",
        ']' => "bracketright",
        '^' => "asciicircum",
        '_' => "underscore",
        '`' => "grave",
        '{' => "braceleft",
        '|' => "bar",
        '}' => "braceright",
        '~' => "asciitilde",
        '\u{a7}' => "section",
        '\u{b0}' => "degree",
        '\u{b4}' => "acute",
        '\u{e4}' => "adiaeresis",
        '\u{f6}' => "odiaeresis",
        '\u{fc}' => "udiaeresis",
        '\u{c4}' => "Adiaeresis",
        '\u{d6}' => "Odiaeresis",
        '\u{dc}' => "Udiaeresis",
        '\u{df}' => "ssharp",
        '\u{20ac}' => "EuroSign",
        _ => return c.to_string(),
    };
    name.to_string()
}

/// Tks keysym einer Taste, die (gerade) kein Zeichen geliefert hat.
pub fn keysym_aus_taste(k: Key) -> String {
    use Key::*;
    let s = match k {
        Key0 => "0",
        Key1 => "1",
        Key2 => "2",
        Key3 => "3",
        Key4 => "4",
        Key5 => "5",
        Key6 => "6",
        Key7 => "7",
        Key8 => "8",
        Key9 => "9",
        A => "a",
        B => "b",
        C => "c",
        D => "d",
        E => "e",
        F => "f",
        G => "g",
        H => "h",
        I => "i",
        J => "j",
        K => "k",
        L => "l",
        M => "m",
        N => "n",
        O => "o",
        P => "p",
        Q => "q",
        R => "r",
        S => "s",
        T => "t",
        U => "u",
        V => "v",
        W => "w",
        X => "x",
        Y => "y",
        Z => "z",
        F1 => "F1",
        F2 => "F2",
        F3 => "F3",
        F4 => "F4",
        F5 => "F5",
        F6 => "F6",
        F7 => "F7",
        F8 => "F8",
        F9 => "F9",
        F10 => "F10",
        F11 => "F11",
        F12 => "F12",
        Down => "Down",
        Left => "Left",
        Right => "Right",
        Up => "Up",
        Apostrophe => "apostrophe",
        Backquote => "grave",
        Backslash => "backslash",
        Comma => "comma",
        Equal => "equal",
        LeftBracket => "bracketleft",
        Minus => "minus",
        Period => "period",
        RightBracket => "bracketright",
        Semicolon => "semicolon",
        Slash => "slash",
        Backspace => "BackSpace",
        Delete => "Delete",
        End => "End",
        Enter => "Return",
        Escape => "Escape",
        Home => "Home",
        Insert => "Insert",
        Menu => "App",
        PageDown => "Next",
        PageUp => "Prior",
        Pause => "Pause",
        Space => "space",
        Tab => "Tab",
        NumLock => "Num_Lock",
        CapsLock => "Caps_Lock",
        ScrollLock => "Scroll_Lock",
        LeftShift => "Shift_L",
        RightShift => "Shift_R",
        LeftCtrl => "Control_L",
        RightCtrl => "Control_R",
        NumPad0 => "KP_0",
        NumPad1 => "KP_1",
        NumPad2 => "KP_2",
        NumPad3 => "KP_3",
        NumPad4 => "KP_4",
        NumPad5 => "KP_5",
        NumPad6 => "KP_6",
        NumPad7 => "KP_7",
        NumPad8 => "KP_8",
        NumPad9 => "KP_9",
        NumPadDot => "KP_Decimal",
        NumPadSlash => "KP_Divide",
        NumPadAsterisk => "KP_Multiply",
        NumPadMinus => "KP_Subtract",
        NumPadPlus => "KP_Add",
        NumPadEnter => "Return",
        LeftAlt => "Alt_L",
        RightAlt => "Alt_R",
        LeftSuper => "Win_L",
        RightSuper => "Win_R",
        _ => "",
    };
    s.to_string()
}

impl Tastatur {
    /// Holt die Tastenmeldungen eines Bildes aus dem Fenster.
    pub fn abholen(&mut self, fenster: &Window, zeichen: &Rc<RefCell<Vec<char>>>) -> Vec<Taste> {
        let mut out = Vec::new();
        for k in fenster.get_keys_released() {
            let keysym = self.gehalten.remove(&k).unwrap_or_else(|| keysym_aus_taste(k));
            if !keysym.is_empty() {
                out.push(Taste { keysym, ch: String::new(), repeat: false, down: false });
            }
        }
        // Steuerzeichen (Return, Backspace, Escape, Tab) kommen ueber die
        // Taste selbst - im Zeichenstrom wuerden sie die Zuordnung verschieben.
        let alle: Vec<char> = zeichen.borrow_mut().drain(..).collect();
        let mut zs = alle.into_iter().filter(|c| (*c as u32) >= 32 && *c != '\x7f');
        let ctrl = fenster.is_key_down(Key::LeftCtrl) || fenster.is_key_down(Key::RightCtrl);
        for k in fenster.get_keys_pressed(KeyRepeat::Yes) {
            let wiederholt = self.gehalten.contains_key(&k);
            let (keysym, ch) = if erzeugt_zeichen(k) && !ctrl {
                match zs.next() {
                    Some(c) => (keysym_aus_zeichen(c), c.to_string()),
                    None => (keysym_aus_taste(k), String::new()),
                }
            } else {
                (keysym_aus_taste(k), String::new())
            };
            if keysym.is_empty() {
                continue;
            }
            let keysym = if wiederholt {
                self.gehalten.get(&k).cloned().unwrap_or(keysym)
            } else {
                self.gehalten.insert(k, keysym.clone());
                keysym
            };
            out.push(Taste { keysym, ch, repeat: wiederholt, down: true });
        }
        out
    }

    /// Tastenmeldungen aus den Windows-Nachrichten (siehe
    /// `fenster::tasten_holen`): Taste und Zeichen gehoeren zusammen, die
    /// Reihenfolge ist die getippte, Wiederholungen kommen von Windows -
    /// wie bei Tk.
    pub fn aus_nachrichten(&mut self, roh: Vec<crate::fenster::RohTaste>) -> Vec<Taste> {
        let mut out = Vec::new();
        for r in roh {
            let id = (r.vk, r.scan);
            if r.down {
                let keysym = keysym_win(r.vk, r.scan, &r.zeichen);
                if keysym.is_empty() {
                    continue;
                }
                let ch: String = r.zeichen.iter().collect();
                let (keysym, repeat) = match self.gehalten_win.get(&id) {
                    Some(erstes) => (erstes.clone(), true),
                    None => {
                        self.gehalten_win.insert(id, keysym.clone());
                        (keysym, false)
                    }
                };
                out.push(Taste { keysym, ch, repeat, down: true });
            } else {
                let keysym = self.gehalten_win.remove(&id).unwrap_or_else(|| keysym_win(r.vk, r.scan, &[]));
                if !keysym.is_empty() {
                    out.push(Taste { keysym, ch: String::new(), repeat: false, down: false });
                }
            }
        }
        out
    }

    /// Alle gehaltenen Tasten loslassen (Fokusverlust, Pause).
    pub fn alle_los(&mut self) -> Vec<String> {
        let mut alle: Vec<String> = self.gehalten.drain().map(|(_, v)| v).collect();
        alle.extend(self.gehalten_win.drain().map(|(_, v)| v));
        alle
    }
}
