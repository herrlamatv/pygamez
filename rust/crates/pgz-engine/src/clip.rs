//! clip.rs
//! =======
//! Zwischenablage - `copy_to_clipboard` / `_win_clipboard` aus
//! `games/wordle.py`.
//!
//! Python versucht zuerst die Tk-Zwischenablage des eingebetteten Fensters
//! und faellt sonst auf die Win32-API zurueck. Die Rust-Fassung hat kein Tk:
//! unter Windows geht der Text direkt als `CF_UNICODETEXT` an das System,
//! sonst gibt es `false` (wie Python ohne Tk ausserhalb von Windows).

/// Legt `text` in die Zwischenablage (`false`, wenn das nicht geht).
pub fn copy_to_clipboard(text: &str) -> bool {
    #[cfg(windows)]
    {
        win::set(text)
    }
    #[cfg(not(windows))]
    {
        let _ = text;
        false
    }
}

#[cfg(windows)]
mod win {
    use std::ffi::c_void;

    type Handle = *mut c_void;

    #[link(name = "user32")]
    extern "system" {
        fn OpenClipboard(hwnd: Handle) -> i32;
        fn CloseClipboard() -> i32;
        fn EmptyClipboard() -> i32;
        fn SetClipboardData(format: u32, mem: Handle) -> Handle;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GlobalAlloc(flags: u32, bytes: usize) -> Handle;
        fn GlobalLock(mem: Handle) -> *mut c_void;
        fn GlobalUnlock(mem: Handle) -> i32;
        fn GlobalFree(mem: Handle) -> Handle;
    }

    /// GMEM_MOVEABLE | GMEM_ZEROINIT
    const GHND: u32 = 0x0042;
    /// CF_UNICODETEXT
    const CF_UNICODETEXT: u32 = 13;

    /// `_win_clipboard(text)` - UTF-16 mit abschliessender Null.
    pub fn set(text: &str) -> bool {
        let mut data: Vec<u16> = text.encode_utf16().collect();
        data.push(0);
        let bytes = data.len() * 2;
        // SAFETY: Win32-Aufrufe in der dokumentierten Reihenfolge; der
        // Speicherblock gehoert nach erfolgreichem SetClipboardData dem
        // System, sonst wird er wieder freigegeben.
        unsafe {
            if OpenClipboard(std::ptr::null_mut()) == 0 {
                return false;
            }
            let ok = (|| {
                EmptyClipboard();
                let handle = GlobalAlloc(GHND, bytes);
                if handle.is_null() {
                    return false;
                }
                let ptr = GlobalLock(handle);
                if ptr.is_null() {
                    GlobalFree(handle);
                    return false;
                }
                std::ptr::copy_nonoverlapping(data.as_ptr() as *const u8, ptr as *mut u8, bytes);
                GlobalUnlock(handle);
                if SetClipboardData(CF_UNICODETEXT, handle).is_null() {
                    GlobalFree(handle);
                    return false;
                }
                true
            })();
            CloseClipboard();
            ok
        }
    }
}
