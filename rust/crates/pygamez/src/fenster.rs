//! fenster.rs
//! ==========
//! Was `main.py` am Tk-Fenster einstellt und minifb selbst nicht kann:
//!
//! * DPI-Bewusstsein (`_enable_dpi_awareness`): der Prozess rechnet in
//!   echten Bildschirmpixeln, und Tks Punkt-Schriften wachsen mit der
//!   Anzeigeskalierung (`dpi()`, genutzt von `sidebar::pt`).
//! * Fenster- und Taskleisten-Symbol (`_set_window_icon`): das Logo, das
//!   `build.rs` als Ressource ins Programm legt.
//! * Mindestgroesse (`root.minsize(880, 560)`, Innenflaeche).
//! * Die Tastatur wie Tk: minifb meldet eine Taste erst ein Bild nach ihrem
//!   Zeichen (es zaehlt die Tasten, bevor es die Nachrichten abholt) - so
//!   gingen Zeichen verloren und Buchstaben folgten der US-Tastenlage
//!   (Y/Z vertauscht, keine Umlaute). Die eigene Fensterprozedur holt zu
//!   jedem WM_KEYDOWN sofort sein WM_CHAR ab, wie Tk selbst.
//! * Alt+F4 schliesst (minifb schluckt WM_SYSKEYDOWN).
//! * Bildschirmgroesse fuer die Startgroesse aus `_fit_sidebar`.
//!
//! Auf anderen Systemen bleibt alles beim Standard von minifb.

use std::sync::atomic::{AtomicU32, Ordering};

/// Bildpunkte je Zoll der Anzeige (Tk: `tk scaling` = dpi / 72).
static DPI: AtomicU32 = AtomicU32::new(96);

/// Aktuelle DPI (96, solange `dpi_bewusst` nicht lief - etwa im Pruefstand).
pub fn dpi() -> u32 {
    DPI.load(Ordering::Relaxed)
}

#[cfg(windows)]
mod w32 {
    use std::ffi::c_void;

    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    pub struct Rect {
        pub left: i32,
        pub top: i32,
        pub right: i32,
        pub bottom: i32,
    }

    #[repr(C)]
    pub struct Point {
        pub x: i32,
        pub y: i32,
    }

    /// MINMAXINFO
    #[repr(C)]
    pub struct MinMaxInfo {
        pub reserved: Point,
        pub max_size: Point,
        pub max_position: Point,
        pub min_track: Point,
        pub max_track: Point,
    }

    #[link(name = "user32")]
    extern "system" {
        pub fn SetProcessDPIAware() -> i32;
        pub fn GetSystemMetrics(index: i32) -> i32;
        pub fn GetDC(hwnd: isize) -> isize;
        pub fn ReleaseDC(hwnd: isize, hdc: isize) -> i32;
        pub fn LoadImageW(inst: isize, name: *const u16, typ: u32, cx: i32, cy: i32, flags: u32) -> isize;
        pub fn SendMessageW(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize;
        pub fn SetWindowLongPtrW(hwnd: isize, index: i32, value: isize) -> isize;
        pub fn GetWindowLongPtrW(hwnd: isize, index: i32) -> isize;
        pub fn CallWindowProcW(prev: isize, hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize;
        pub fn DefWindowProcW(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize;
        pub fn GetWindowRect(hwnd: isize, rect: *mut Rect) -> i32;
        pub fn GetClientRect(hwnd: isize, rect: *mut Rect) -> i32;
        pub fn SetWindowPos(hwnd: isize, after: isize, x: i32, y: i32, cx: i32, cy: i32, flags: u32) -> i32;
        pub fn PeekMessageW(msg: *mut Msg, hwnd: isize, min: u32, max: u32, remove: u32) -> i32;
        pub fn MapVirtualKeyW(code: u32, map_type: u32) -> u32;
        pub fn GetMessageW(msg: *mut Msg, hwnd: isize, min: u32, max: u32) -> i32;
        pub fn TranslateMessage(msg: *const Msg) -> i32;
        pub fn DispatchMessageW(msg: *const Msg) -> isize;
        pub fn CreateWindowExW(
            ex_style: u32,
            class: *const u16,
            title: *const u16,
            style: u32,
            x: i32,
            y: i32,
            w: i32,
            h: i32,
            parent: isize,
            menu: isize,
            inst: isize,
            param: *const std::ffi::c_void,
        ) -> isize;
        pub fn DestroyWindow(hwnd: isize) -> i32;
    }

    /// MSG
    #[repr(C)]
    #[derive(Default)]
    pub struct Msg {
        pub hwnd: isize,
        pub message: u32,
        pub wparam: usize,
        pub lparam: isize,
        pub time: u32,
        pub pt_x: i32,
        pub pt_y: i32,
        pub private: u32,
    }

    pub const WM_KEYDOWN: u32 = 0x0100;
    pub const WM_KEYUP: u32 = 0x0101;
    pub const WM_CHAR: u32 = 0x0102;
    pub const WM_SYSKEYDOWN: u32 = 0x0104;
    pub const WM_SYSKEYUP: u32 = 0x0105;
    pub const WM_SYSCHAR: u32 = 0x0106;
    pub const WM_SYSDEADCHAR: u32 = 0x0107;
    pub const PM_REMOVE: u32 = 0x0001;
    pub const VK_F4: usize = 0x73;

    #[link(name = "gdi32")]
    extern "system" {
        pub fn GetDeviceCaps(hdc: isize, index: i32) -> i32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        pub fn GetModuleHandleW(name: *const u16) -> isize;
        pub fn LoadLibraryW(name: *const u16) -> isize;
        pub fn GetProcAddress(module: isize, name: *const u8) -> *const c_void;
    }

    pub const SM_CXSCREEN: i32 = 0;
    pub const SM_CYSCREEN: i32 = 1;
    pub const SM_CXICON: i32 = 11;
    pub const SM_CYICON: i32 = 12;
    pub const SM_CXSMICON: i32 = 49;
    pub const SM_CYSMICON: i32 = 50;
    pub const LOGPIXELSY: i32 = 90;
    pub const IMAGE_ICON: u32 = 1;
    pub const WM_SETICON: u32 = 0x0080;
    pub const WM_GETMINMAXINFO: u32 = 0x0024;
    pub const GWLP_WNDPROC: i32 = -4;

    pub fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }
}

/// `_enable_dpi_awareness()` - muss vor dem ersten Fenster laufen.
///
/// Per-Monitor-DPI-aware (Windows 8.1+), sonst der alte Weg; danach wird die
/// DPI der Anzeige gemerkt (Tk liest sie einmal beim Start).
#[cfg(windows)]
pub fn dpi_bewusst() {
    use w32::*;
    // SAFETY: nur Win32-Aufrufe mit gueltigen, lokal lebenden Argumenten;
    // die Funktionsadresse aus shcore.dll hat die dokumentierte Signatur.
    unsafe {
        let shcore = LoadLibraryW(wide("shcore.dll").as_ptr());
        let f = if shcore != 0 { GetProcAddress(shcore, c"SetProcessDpiAwareness".as_ptr().cast()) } else { std::ptr::null() };
        if f.is_null() {
            SetProcessDPIAware();
        } else {
            // 2 = PROCESS_PER_MONITOR_DPI_AWARE
            let set: extern "system" fn(i32) -> i32 = std::mem::transmute(f);
            set(2);
        }
        let hdc = GetDC(0);
        if hdc != 0 {
            let d = GetDeviceCaps(hdc, LOGPIXELSY);
            ReleaseDC(0, hdc);
            if d > 0 {
                DPI.store(d as u32, Ordering::Relaxed);
            }
        }
    }
}

#[cfg(not(windows))]
pub fn dpi_bewusst() {}

/// Groesse des Hauptbildschirms (`winfo_screenwidth/height`).
#[cfg(windows)]
pub fn bildschirm() -> (i32, i32) {
    // SAFETY: GetSystemMetrics hat keine Vorbedingungen.
    unsafe { (w32::GetSystemMetrics(w32::SM_CXSCREEN), w32::GetSystemMetrics(w32::SM_CYSCREEN)) }
}

#[cfg(not(windows))]
pub fn bildschirm() -> (i32, i32) {
    (1920, 1080)
}

/// `_set_window_icon()` - Symbol fuer Titelleiste (klein) und Taskleiste /
/// Alt+Tab (gross). Rueckgabe: konnte es gesetzt werden?
#[cfg(windows)]
pub fn symbol_setzen(hwnd: *mut std::ffi::c_void) -> bool {
    use w32::*;
    let hwnd = hwnd as isize;
    // SAFETY: hwnd ist das lebende Fenster von minifb; Ressource 1 ist die
    // Icon-Gruppe aus build.rs (MAKEINTRESOURCE(1)).
    unsafe {
        let inst = GetModuleHandleW(std::ptr::null());
        // MAKEINTRESOURCE(1): die Zahl 1 als "Zeiger" (keine echte Adresse).
        let id = std::ptr::without_provenance::<u16>(1);
        let klein = LoadImageW(inst, id, IMAGE_ICON, GetSystemMetrics(SM_CXSMICON), GetSystemMetrics(SM_CYSMICON), 0);
        let gross = LoadImageW(inst, id, IMAGE_ICON, GetSystemMetrics(SM_CXICON), GetSystemMetrics(SM_CYICON), 0);
        if klein != 0 {
            SendMessageW(hwnd, WM_SETICON, 0, klein);
        }
        if gross != 0 {
            SendMessageW(hwnd, WM_SETICON, 1, gross);
        }
        klein != 0 && gross != 0
    }
}

#[cfg(not(windows))]
pub fn symbol_setzen(_hwnd: *mut std::ffi::c_void) -> bool {
    false
}

#[cfg(windows)]
mod minimum {
    use std::sync::atomic::{AtomicI32, AtomicIsize, Ordering};

    pub static ALT: AtomicIsize = AtomicIsize::new(0);
    pub static BREITE: AtomicI32 = AtomicI32::new(0);
    pub static HOEHE: AtomicI32 = AtomicI32::new(0);

    /// Tastenmeldungen in Reihenfolge (siehe [`super::RohTaste`]).
    pub static TASTEN: std::sync::Mutex<Vec<super::RohTaste>> = std::sync::Mutex::new(Vec::new());

    /// Holt die Zeichen ab, die TranslateMessage zu diesem WM_KEYDOWN gerade
    /// in die Schlange gelegt hat - wie Tk (`GetTranslatedKey`): so gehoeren
    /// Taste und Zeichen fest zusammen.
    unsafe fn zeichen_dazu(hwnd: isize) -> Vec<char> {
        use super::w32::*;
        let mut einheiten: Vec<u16> = Vec::new();
        let mut m = Msg::default();
        // SAFETY: m lebt fuer den Aufruf; nur WM_CHAR..WM_SYSDEADCHAR.
        while unsafe { PeekMessageW(&mut m, hwnd, WM_CHAR, WM_SYSDEADCHAR, PM_REMOVE) } != 0 {
            if m.message == WM_CHAR || m.message == WM_SYSCHAR {
                einheiten.push(m.wparam as u16);
            }
        }
        char::decode_utf16(einheiten).filter_map(Result::ok).collect()
    }

    /// Fensterprozedur vor der von minifb: Mindestgroesse (WM_GETMINMAXINFO)
    /// und die Tastatur wie Tk; alles andere geht an minifb weiter.
    pub unsafe extern "system" fn proc(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize {
        use super::w32::*;
        if msg == WM_GETMINMAXINFO && lparam != 0 {
            // SAFETY: Windows uebergibt bei WM_GETMINMAXINFO einen gueltigen
            // Zeiger auf MINMAXINFO.
            let mmi = unsafe { &mut *(lparam as *mut MinMaxInfo) };
            mmi.min_track.x = BREITE.load(Ordering::Relaxed);
            mmi.min_track.y = HOEHE.load(Ordering::Relaxed);
            return 0;
        }
        let taste = matches!(msg, WM_KEYDOWN | WM_SYSKEYDOWN | WM_KEYUP | WM_SYSKEYUP);
        if taste {
            let down = matches!(msg, WM_KEYDOWN | WM_SYSKEYDOWN);
            // SAFETY: nur Nachrichten dieses Fensters aus der eigenen Schlange.
            let zeichen = if down { unsafe { zeichen_dazu(hwnd) } } else { Vec::new() };
            let roh = super::RohTaste {
                down,
                vk: wparam as u32,
                scan: ((lparam as u32) >> 16) & 0x1ff,
                zeichen,
            };
            TASTEN.lock().unwrap_or_else(|e| e.into_inner()).push(roh);
        }
        let alt = ALT.load(Ordering::Relaxed);
        // SAFETY: ALT ist die urspruengliche Fensterprozedur dieses Fensters.
        let ergebnis = unsafe {
            if alt == 0 {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            } else {
                CallWindowProcW(alt, hwnd, msg, wparam, lparam)
            }
        };
        // minifb schluckt WM_SYSKEYDOWN - Alt+F4 soll aber schliessen wie bei
        // jedem Fenster (Tk: WM_DELETE_WINDOW -> beenden).
        if msg == WM_SYSKEYDOWN && wparam == VK_F4 {
            // SAFETY: Standardbehandlung fuer dieses Fenster.
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        }
        ergebnis
    }
}

/// Eine Tastenmeldung, wie sie das Fenster bekommen hat: Druck oder
/// Loslassen, virtueller Tastencode, Scancode (mit Bit 0x100 = erweitert)
/// und die dabei getippten Zeichen. (Wiederholungen erkennt `input.rs` wie
/// `main.py` an der schon gehaltenen Taste.)
#[derive(Debug, Clone)]
pub struct RohTaste {
    pub down: bool,
    pub vk: u32,
    pub scan: u32,
    pub zeichen: Vec<char>,
}

/// Holt die seit dem letzten Aufruf eingegangenen Tastenmeldungen
/// (`None`, solange die eigene Fensterprozedur nicht eingehaengt ist).
#[cfg(windows)]
pub fn tasten_holen() -> Option<Vec<RohTaste>> {
    if minimum::ALT.load(Ordering::Relaxed) == 0 {
        return None;
    }
    Some(std::mem::take(&mut *minimum::TASTEN.lock().unwrap_or_else(|e| e.into_inner())))
}

#[cfg(not(windows))]
pub fn tasten_holen() -> Option<Vec<RohTaste>> {
    None
}

/// Grundzeichen einer Taste im aktuellen Layout (ohne Umschalt), etwa fuer
/// Strg+Taste, bei der Windows kein druckbares Zeichen liefert.
#[cfg(windows)]
pub fn grundzeichen(vk: u32) -> Option<char> {
    // SAFETY: MapVirtualKeyW hat keine Vorbedingungen.
    let v = unsafe { w32::MapVirtualKeyW(vk, 2) };
    // Hohes Bit = Tottaste.
    if v == 0 || v & 0x8000_0000 != 0 {
        return None;
    }
    char::from_u32(v & 0xffff).map(|c| c.to_lowercase().next().unwrap_or(c))
}

#[cfg(not(windows))]
pub fn grundzeichen(_vk: u32) -> Option<char> {
    None
}

/// Haengt die eigene Fensterprozedur ein: `root.minsize(w, h)` (kleinste
/// Innenflaeche beim Ziehen am Rand) und die Tastatur wie Tk
/// ([`tasten_holen`]).
#[cfg(windows)]
pub fn einhaengen(hwnd: *mut std::ffi::c_void, w: i32, h: i32) {
    use w32::*;
    let hwnd = hwnd as isize;
    if hwnd == 0 || minimum::ALT.load(Ordering::Relaxed) != 0 {
        return;
    }
    let (mut aussen, mut innen) = (Rect::default(), Rect::default());
    // SAFETY: hwnd ist das lebende Fenster von minifb; die neue Prozedur
    // reicht alles ausser WM_GETMINMAXINFO an die alte weiter.
    unsafe {
        GetWindowRect(hwnd, &mut aussen);
        GetClientRect(hwnd, &mut innen);
        // Mindestgroesse gilt fuer die Innenflaeche - der Rahmen kommt dazu.
        let rand_w = (aussen.right - aussen.left) - (innen.right - innen.left);
        let rand_h = (aussen.bottom - aussen.top) - (innen.bottom - innen.top);
        minimum::BREITE.store(w + rand_w, Ordering::Relaxed);
        minimum::HOEHE.store(h + rand_h, Ordering::Relaxed);
        // Erst die alte Prozedur merken, dann die neue einsetzen.
        minimum::ALT.store(GetWindowLongPtrW(hwnd, GWLP_WNDPROC), Ordering::Relaxed);
        let zeiger: unsafe extern "system" fn(isize, u32, usize, isize) -> isize = minimum::proc;
        let neu = zeiger as usize as isize;
        SetWindowLongPtrW(hwnd, GWLP_WNDPROC, neu);
    }
}

#[cfg(not(windows))]
pub fn einhaengen(_hwnd: *mut std::ffi::c_void, _w: i32, _h: i32) {}

/// Pruefschalter `PYGAMEZ_TASTEN_PROBE=<datei>` (`tools/tasten_check.py`):
/// ein VERSTECKTES Fenster (bekommt nie echte Eingaben, holt keinen Fokus)
/// mit derselben Fensterprozedur und derselben Nachrichtenschleife wie
/// minifb (TranslateMessage vor DispatchMessage). Das Fenster-Handle steht
/// in `<datei>.hwnd`; jede Taste wird mit `input.rs` uebersetzt und als
/// `druck|los, keysym, Zeichen (hex)` nach `<datei>` geschrieben, bis die
/// Nachricht WM_APP+1 kommt.
#[cfg(windows)]
pub fn tasten_probe(ziel: &str, tastatur: &mut crate::input::Tastatur) {
    use w32::*;
    const ENDE: u32 = 0x8001;
    // SAFETY: Win32-Aufrufe mit lokal lebenden Argumenten; das Fenster gehoert
    // diesem Thread und wird am Ende zerstoert.
    unsafe {
        let hwnd = CreateWindowExW(
            0,
            wide("STATIC").as_ptr(),
            wide("pygamez-tasten").as_ptr(),
            0x8000_0000, // WS_POPUP, ohne WS_VISIBLE
            0,
            0,
            8,
            8,
            0,
            0,
            GetModuleHandleW(std::ptr::null()),
            std::ptr::null(),
        );
        if hwnd == 0 {
            return;
        }
        einhaengen(hwnd as *mut std::ffi::c_void, 0, 0);
        let _ = std::fs::write(format!("{ziel}.hwnd"), hwnd.to_string());
        let mut log = Vec::new();
        let mut m = Msg::default();
        while GetMessageW(&mut m, 0, 0, 0) > 0 {
            if m.message == ENDE {
                break;
            }
            TranslateMessage(&m);
            DispatchMessageW(&m);
            for t in tastatur.aus_nachrichten(tasten_holen().unwrap_or_default()) {
                let art = if t.down { "druck" } else { "los" };
                let codes: Vec<String> = t.ch.chars().map(|c| format!("{:x}", c as u32)).collect();
                log.push(format!("{art}\t{}\t{}\t{}", t.keysym, codes.join(","), t.repeat));
            }
        }
        DestroyWindow(hwnd);
        let _ = std::fs::write(ziel, log.join("\n") + "\n");
    }
}

#[cfg(not(windows))]
pub fn tasten_probe(_ziel: &str, _tastatur: &mut crate::input::Tastatur) {}

/// Fuer den Pruefschalter `PYGAMEZ_VOLLBILD_PROBE`: Innenflaeche auf w x h
/// setzen wollen (Windows haelt dabei die Mindestgroesse ein).
#[cfg(windows)]
pub fn innen_setzen(hwnd: *mut std::ffi::c_void, w: i32, h: i32) {
    use w32::*;
    let hwnd = hwnd as isize;
    let (mut aussen, mut innen) = (Rect::default(), Rect::default());
    // SAFETY: hwnd ist das lebende Fenster von minifb.
    unsafe {
        GetWindowRect(hwnd, &mut aussen);
        GetClientRect(hwnd, &mut innen);
        let rand_w = (aussen.right - aussen.left) - (innen.right - innen.left);
        let rand_h = (aussen.bottom - aussen.top) - (innen.bottom - innen.top);
        // SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE
        SetWindowPos(hwnd, 0, 0, 0, w + rand_w, h + rand_h, 0x0002 | 0x0004 | 0x0010);
    }
}

#[cfg(not(windows))]
pub fn innen_setzen(_hwnd: *mut std::ffi::c_void, _w: i32, _h: i32) {}
