//! vollbild.rs
//! ===========
//! `toggle_fullscreen()` aus `main.py` (Tk: `root.attributes("-fullscreen")`).
//!
//! minifb kennt kein Umschalten in den Vollbildmodus, liefert aber das
//! Fenster-Handle. Unter Windows wird daraus ein randloses Vollbild auf dem
//! Monitor, auf dem das Fenster gerade steht: Rahmen-Stil abnehmen, Fenster
//! auf die Monitorflaeche legen - und beim Zurueckschalten Stil und Lage von
//! vorher wiederherstellen. Auf anderen Systemen bleibt es beim Fenster.

#[cfg(windows)]
mod w32 {
    #[repr(C)]
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
    pub struct Rect {
        pub left: i32,
        pub top: i32,
        pub right: i32,
        pub bottom: i32,
    }

    #[repr(C)]
    #[derive(Default)]
    pub struct MonitorInfo {
        pub cb_size: u32,
        pub rc_monitor: Rect,
        pub rc_work: Rect,
        pub flags: u32,
    }

    #[link(name = "user32")]
    extern "system" {
        pub fn GetWindowLongPtrW(hwnd: isize, index: i32) -> isize;
        pub fn SetWindowLongPtrW(hwnd: isize, index: i32, value: isize) -> isize;
        pub fn GetWindowRect(hwnd: isize, rect: *mut Rect) -> i32;
        pub fn GetClientRect(hwnd: isize, rect: *mut Rect) -> i32;
        pub fn MonitorFromWindow(hwnd: isize, flags: u32) -> isize;
        pub fn GetMonitorInfoW(monitor: isize, info: *mut MonitorInfo) -> i32;
        pub fn SetWindowPos(hwnd: isize, after: isize, x: i32, y: i32, cx: i32, cy: i32, flags: u32) -> i32;
    }

    pub const GWL_STYLE: i32 = -16;
    pub const WS_OVERLAPPEDWINDOW: isize = 0x00CF_0000;
    pub const MONITOR_DEFAULTTONEAREST: u32 = 2;
    pub const SWP_NOZORDER: u32 = 0x0004;
    pub const SWP_FRAMECHANGED: u32 = 0x0020;
    pub const SWP_NOOWNERZORDER: u32 = 0x0200;

    /// Flaeche des Monitors, auf dem das Fenster steht.
    pub fn monitor(hwnd: isize) -> Option<Rect> {
        let mut info = MonitorInfo { cb_size: std::mem::size_of::<MonitorInfo>() as u32, ..Default::default() };
        // SAFETY: hwnd ist ein lebendes Fenster, info lebt fuer den Aufruf.
        let ok = unsafe { GetMonitorInfoW(MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST), &mut info) };
        (ok != 0).then_some(info.rc_monitor)
    }
}

/// Schaltet zwischen Fenster und Vollbild um; Rueckgabe: jetzt Vollbild?
#[cfg(windows)]
pub fn umschalten(hwnd: *mut std::ffi::c_void) -> bool {
    use std::sync::Mutex;
    use w32::*;

    // Gemerkt: Stil und Lage des Fensters vor dem Vollbild.
    static VORHER: Mutex<Option<(isize, Rect)>> = Mutex::new(None);

    let hwnd = hwnd as isize;
    if hwnd == 0 {
        return false;
    }
    let mut vorher = VORHER.lock().unwrap_or_else(|e| e.into_inner());
    // SAFETY: hwnd ist das lebende Fenster von minifb; die Strukturen sind
    // wie in der Win32-API ausgelegt und leben fuer die Dauer der Aufrufe.
    unsafe {
        match vorher.take() {
            None => {
                let stil = GetWindowLongPtrW(hwnd, GWL_STYLE);
                let mut lage = Rect::default();
                if GetWindowRect(hwnd, &mut lage) == 0 {
                    return false;
                }
                let Some(m) = monitor(hwnd) else {
                    return false;
                };
                SetWindowLongPtrW(hwnd, GWL_STYLE, stil & !WS_OVERLAPPEDWINDOW);
                SetWindowPos(hwnd, 0, m.left, m.top, m.right - m.left, m.bottom - m.top, SWP_NOOWNERZORDER | SWP_FRAMECHANGED);
                *vorher = Some((stil, lage));
                true
            }
            Some((stil, lage)) => {
                SetWindowLongPtrW(hwnd, GWL_STYLE, stil);
                SetWindowPos(
                    hwnd,
                    0,
                    lage.left,
                    lage.top,
                    lage.right - lage.left,
                    lage.bottom - lage.top,
                    SWP_NOZORDER | SWP_NOOWNERZORDER | SWP_FRAMECHANGED,
                );
                false
            }
        }
    }
}

/// Andere Systeme: kein Vollbild (minifb kann es nicht umschalten).
#[cfg(not(windows))]
pub fn umschalten(_hwnd: *mut std::ffi::c_void) -> bool {
    false
}

/// Fuer den Pruefschalter `PYGAMEZ_VOLLBILD_PROBE`: Stil, Fensterlage,
/// Innenflaeche und Monitorflaeche als eine Zeile.
#[cfg(windows)]
pub fn zustand(hwnd: *mut std::ffi::c_void) -> String {
    use w32::*;
    let hwnd = hwnd as isize;
    let (mut lage, mut innen) = (Rect::default(), Rect::default());
    // SAFETY: hwnd ist das lebende Fenster von minifb.
    let stil = unsafe {
        GetWindowRect(hwnd, &mut lage);
        GetClientRect(hwnd, &mut innen);
        GetWindowLongPtrW(hwnd, GWL_STYLE)
    };
    let m = monitor(hwnd).unwrap_or_default();
    format!(
        "stil={stil:#010x} rahmen={} fenster={},{} {}x{} innen={}x{} monitor={},{} {}x{}",
        stil & WS_OVERLAPPEDWINDOW == WS_OVERLAPPEDWINDOW,
        lage.left,
        lage.top,
        lage.right - lage.left,
        lage.bottom - lage.top,
        innen.right - innen.left,
        innen.bottom - innen.top,
        m.left,
        m.top,
        m.right - m.left,
        m.bottom - m.top,
    )
}

#[cfg(not(windows))]
pub fn zustand(_hwnd: *mut std::ffi::c_void) -> String {
    String::from("kein Windows")
}
