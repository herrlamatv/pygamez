//! filepick.rs
//! ===========
//! Datei-Dialoge und Downloads-Ordner - uebertragen aus `filepick.py`.
//!
//! Python nimmt die Tk-Dialoge des eingebetteten Fensters; die Rust-Fassung
//! hat kein Tk und ruft unter Windows die Standard-Dialoge von comdlg32
//! (`GetSaveFileNameW`/`GetOpenFileNameW`). Im Pruefstand (fester Takt)
//! gibt es keine Dialoge (`available() == false`), und der "Downloads"-Ordner
//! liegt im Datenordner - Tests schreiben nie in echte Benutzerordner.

use std::path::{Path, PathBuf};

use crate::py;

/// `available()` - koennen Datei-Dialoge benutzt werden?
pub fn available() -> bool {
    cfg!(windows) && !py::deterministic()
}

fn type_name(ext: &str) -> String {
    match ext {
        ".lamapgzmap" => "PyGameZ Minigolf-Map".into(),
        ".lamapgzlevel" => "PyGameZ Geometry-Dash-Level".into(),
        ".lamapgzreplay" => "PyGameZ Replay".into(),
        ".json" => "JSON".into(),
        e => e.trim_start_matches('.').to_uppercase(),
    }
}

/// Filterliste "Name\0*.ext\0...\0\0" fuer comdlg32.
fn filter(exts: &[&str]) -> Vec<u16> {
    let mut s = String::new();
    for e in exts {
        s += &format!("{}\0*{}\0", type_name(e), e);
    }
    s += "Alle Dateien\0*.*\0\0";
    s.encode_utf16().collect()
}

/// `save_as(initialfile, title, exts)` - Pfad oder `None`.
pub fn save_as(initialfile: &str, title: &str, exts: &[&str]) -> Option<String> {
    if !available() {
        return None;
    }
    #[cfg(windows)]
    {
        win::dialog(true, initialfile, if title.is_empty() { "Speichern unter" } else { title }, exts)
    }
    #[cfg(not(windows))]
    {
        let _ = (initialfile, title, exts);
        None
    }
}

/// `open_file(title, exts)` - Pfad oder `None`.
pub fn open_file(title: &str, exts: &[&str]) -> Option<String> {
    if !available() {
        return None;
    }
    #[cfg(windows)]
    {
        win::dialog(false, "", if title.is_empty() { "\u{00D6}ffnen" } else { title }, exts)
    }
    #[cfg(not(windows))]
    {
        let _ = (title, exts);
        None
    }
}

/// `downloads_dir()` - der Downloads-Ordner (notfalls das Heimatverzeichnis).
pub fn downloads_dir() -> PathBuf {
    if py::deterministic() {
        // Pruefstand: nie in echte Benutzerordner schreiben.
        let base = std::env::var_os("PYGAMEZ_DATA_DIR").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
        let p = base.join("Downloads");
        let _ = std::fs::create_dir_all(&p);
        return p;
    }
    let home = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    let path = home.join("Downloads");
    if path.is_dir() {
        return path;
    }
    match std::fs::create_dir_all(&path) {
        Ok(()) => path,
        Err(_) => home,
    }
}

/// `to_downloads(filename)` - freier Pfad im Downloads-Ordner (" (2)" ...).
pub fn to_downloads(filename: &str) -> String {
    let folder = downloads_dir();
    let p = Path::new(filename);
    let base = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let ext = p.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    let mut path = folder.join(filename);
    let mut n = 2;
    while path.exists() && n < 1000 {
        path = folder.join(format!("{base} ({n}){ext}"));
        n += 1;
    }
    path.to_string_lossy().to_string()
}

/// `short(path, keep)` - Pfad fuer die Anzeige kuerzen.
pub fn short(path: &str, keep: usize) -> String {
    if path.is_empty() {
        return String::new();
    }
    if path.chars().count() <= keep {
        return path.to_string();
    }
    let sep = std::path::MAIN_SEPARATOR;
    let norm = path.replace('/', &sep.to_string());
    let parts: Vec<&str> = norm.split(sep).collect();
    let tail = if parts.len() >= 2 { parts[parts.len() - 2..].join(&sep.to_string()) } else { parts[parts.len() - 1].to_string() };
    if tail.chars().count() < path.chars().count() {
        format!("...{sep}{tail}")
    } else {
        let n = path.chars().count();
        path.chars().skip(n - keep).collect()
    }
}

#[cfg(windows)]
mod win {
    use std::ffi::c_void;

    #[repr(C)]
    struct OpenFileNameW {
        l_struct_size: u32,
        hwnd_owner: *mut c_void,
        h_instance: *mut c_void,
        lpstr_filter: *const u16,
        lpstr_custom_filter: *mut u16,
        n_max_cust_filter: u32,
        n_filter_index: u32,
        lpstr_file: *mut u16,
        n_max_file: u32,
        lpstr_file_title: *mut u16,
        n_max_file_title: u32,
        lpstr_initial_dir: *const u16,
        lpstr_title: *const u16,
        flags: u32,
        n_file_offset: u16,
        n_file_extension: u16,
        lpstr_def_ext: *const u16,
        l_cust_data: isize,
        lpfn_hook: *mut c_void,
        lp_template_name: *const u16,
        pv_reserved: *mut c_void,
        dw_reserved: u32,
        flags_ex: u32,
    }

    #[link(name = "comdlg32")]
    extern "system" {
        fn GetSaveFileNameW(ofn: *mut OpenFileNameW) -> i32;
        fn GetOpenFileNameW(ofn: *mut OpenFileNameW) -> i32;
    }

    #[link(name = "user32")]
    extern "system" {
        fn GetActiveWindow() -> *mut c_void;
    }

    const OFN_OVERWRITEPROMPT: u32 = 0x2;
    const OFN_NOCHANGEDIR: u32 = 0x8;
    const OFN_PATHMUSTEXIST: u32 = 0x800;
    const OFN_FILEMUSTEXIST: u32 = 0x1000;
    const OFN_EXPLORER: u32 = 0x80000;

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub fn dialog(save: bool, initial: &str, title: &str, exts: &[&str]) -> Option<String> {
        let filter = super::filter(exts);
        let title_w = wide(title);
        let def_ext = wide(exts.first().map(|e| e.trim_start_matches('.')).unwrap_or(""));
        let mut buf = vec![0u16; 1024];
        for (i, c) in initial.encode_utf16().take(1000).enumerate() {
            buf[i] = c;
        }
        let mut flags = OFN_EXPLORER | OFN_NOCHANGEDIR | OFN_PATHMUSTEXIST;
        flags |= if save { OFN_OVERWRITEPROMPT } else { OFN_FILEMUSTEXIST };
        // SAFETY: alle Zeiger zeigen auf lokale, lebende Puffer; die Struktur
        // ist nach der Win32-Vorlage (OPENFILENAMEW) aufgebaut.
        unsafe {
            let mut ofn = OpenFileNameW {
                l_struct_size: std::mem::size_of::<OpenFileNameW>() as u32,
                hwnd_owner: GetActiveWindow(),
                h_instance: std::ptr::null_mut(),
                lpstr_filter: filter.as_ptr(),
                lpstr_custom_filter: std::ptr::null_mut(),
                n_max_cust_filter: 0,
                n_filter_index: 1,
                lpstr_file: buf.as_mut_ptr(),
                n_max_file: buf.len() as u32,
                lpstr_file_title: std::ptr::null_mut(),
                n_max_file_title: 0,
                lpstr_initial_dir: std::ptr::null(),
                lpstr_title: title_w.as_ptr(),
                flags,
                n_file_offset: 0,
                n_file_extension: 0,
                lpstr_def_ext: if save { def_ext.as_ptr() } else { std::ptr::null() },
                l_cust_data: 0,
                lpfn_hook: std::ptr::null_mut(),
                lp_template_name: std::ptr::null(),
                pv_reserved: std::ptr::null_mut(),
                dw_reserved: 0,
                flags_ex: 0,
            };
            let ok = if save { GetSaveFileNameW(&mut ofn) } else { GetOpenFileNameW(&mut ofn) };
            if ok == 0 {
                return None;
            }
        }
        let end = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
        let path = String::from_utf16_lossy(&buf[..end]);
        if path.is_empty() {
            None
        } else {
            Some(path)
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn kuerzen_wie_python() {
        assert_eq!(super::short("", 10), "");
        assert_eq!(super::short("C:\\a\\b.txt", 44), "C:\\a\\b.txt");
        if cfg!(windows) {
            assert_eq!(super::short("C:\\Users\\lama\\Downloads\\pygamez-chess-2026.pgn", 34), "...\\Downloads\\pygamez-chess-2026.pgn");
        }
    }
}
