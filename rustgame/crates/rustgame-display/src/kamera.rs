//! kamera.rs
//! =========
//! Die Geraeteseite von `pygame.camera`.
//!
//! ## Hier stand einmal "es gibt keine Kamera"
//!
//! Das Modul `pygame.camera` war in rustgame lange eine Huelle mit einem
//! ehrlichen Fehler: `list_cameras()` gab `[]`, `Camera(...)` warf
//! `pygame.error`. Die Begruendung daneben lautete, ein Kamerabild
//! brauche den Geraetezugriff des Betriebssystems (DirectShow, V4L2), und
//! der ginge nur ueber FFI - der Arbeitsbereich habe kein `unsafe`.
//!
//! **Der erste Teil ist wahr, der zweite war eine Verwechslung**, und zwar
//! dieselbe wie bei `system.get_total_ram` und bei `pygame.midi`: `unsafe`
//! braucht irgendwer, aber nicht dieses Programm. Die Kiste `nokhwa`
//! spricht unter Windows mit Media Foundation, unter Linux mit V4L2 und
//! unter macOS mit AVFoundation - hinter einer sicheren Schnittstelle.
//!
//! Ausgeloest hat es eine einzige Probe:
//!
//! ```text
//! pygame.camera.list_cameras()            ['USB  Live camera']
//! pygame.camera.Camera(..).get_image()    (320, 240), 32 Bit
//! ```
//!
//! Es gab hier also eine Kamera, und pygame holte ein Bild davon. Wer
//! dieselben Namen anbietet und bei jedem Aufruf einen Fehler wirft, hat
//! die Namen und nicht die Sache.
//!
//! ## Warum diese Kiste und nicht `rustgame-core`
//!
//! Eine Kamera ist ein Geraet, und Geraete liegen hier: Fenster, Tasten,
//! Maus, Hebel, Ablage, Systemauskunft. `rustgame-core` soll **ohne jede
//! Betriebssystem-Abhaengigkeit** bauen und kopflos testen; das ist der
//! Grund fuer den Schnitt zwischen den Kisten und bleibt es.
//!
//! Herausgegeben wird trotzdem eine [`rustgame_core::Surface`] - genauer:
//! die rohen RGB-Bytes, aus denen die Huelle eine macht. Die Richtung
//! stimmt, denn diese Kiste haengt an `rustgame-core` und nicht umgekehrt.
//!
//! ## Was gemessen ist
//!
//! Alles hier Festgelegte stammt aus echtem pygame-ce 2.5.7 mit der Kamera
//! `USB  Live camera` an diesem Rechner:
//!
//! ```text
//! get_backends()                  ['_camera (MSMF)', 'OpenCV']
//! list_cameras()                  ['USB  Live camera']   (str, nicht int)
//! Camera(name, (320,240), "RGB")  geht
//! Camera(77)                      TypeError: bad argument type
//! Camera("gibtesnicht")           ValueError: Couldn't find a camera
//!                                 with that name
//! get_size() vor start()          (320, 240)
//! query_image() vor start()       error: Camera needs to be started
//!                                 to read data
//! get_controls()                  (False, False, -1)
//! set_controls(hflip=True)        (True, False, -1)
//! set_controls(brightness=1.5)    TypeError: 'float' object cannot be
//!                                 interpreted as an integer
//! get_image()                     (320, 240), 32 Bit,
//!                                 Masken (0xff0000, 0xff00, 0xff, 0)
//! get_image(zu_kleine_flaeche)    ValueError: Destination surface not
//!                                 the correct width or height.
//! stop(); stop()                  beides None
//! ```
//!
//! Drei dieser Messungen sind ueberraschend und stehen genau deshalb hier:
//!
//! * **Die Groesse ist ein Wunsch, keine Vorgabe.** `Camera(name, (1,1))`
//!   und `Camera(name, (99999,99999))` geben beide `(320, 240)` - das
//!   Geraet kann nur das, und der Treiber nimmt das Naechstliegende. Wer
//!   hier einen Fehler wirft, weil die Zahl nicht passt, verhaelt sich
//!   anders als pygame.
//! * **`format` wird nicht geprueft.** `"YUV"`, `"HSV"`, `"RGBA"` und
//!   sogar `"quatsch"` kommen alle durch. Die Umrechnung macht
//!   `colorspace`, und die ist eine eigene Funktion.
//! * **`get_raw()` gibt unter Windows `None`.** Siehe [`Kamera::roh`].

use std::collections::HashSet;

use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{
    ApiBackend, CameraFormat, CameraIndex, FrameFormat, KnownCameraControl, RequestedFormat,
    RequestedFormatType, Resolution,
};
use nokhwa::FormatDecoder;

/// Ein gefundenes Kameragerat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Geraet {
    /// Der Name, den `list_cameras()` herausgibt.
    pub name: String,
    /// Was der Treiber sonst noch dazu sagt - bei pygame nicht sichtbar.
    pub beschreibung: String,
    /// Die Nummer, unter der der Treiber das Geraet fuehrt.
    pub nummer: u32,
}

/// Welche Wege es hier zu einer Kamera gibt.
///
/// pygame gibt auf dieser Maschine `['_camera (MSMF)', 'OpenCV']` - der
/// zweite Eintrag steht nur da, weil `cv2` in derselben Umgebung
/// installiert ist; pygame hat dafuer ein zweites, reines Python-Stueck.
/// Hier gibt es den einen Weg, und der heisst wie pygames erster.
#[must_use]
pub fn rueckwaende() -> Vec<String> {
    let name = match std::env::consts::OS {
        "windows" => "_camera (MSMF)",
        "macos" | "ios" => "_camera (AVFoundation)",
        "linux" => "_camera (V4L2)",
        _ => return Vec::new(),
    };
    vec![name.to_string()]
}

/// Welchen Treiber diese Maschine hat - `None` auf allem Uebrigen.
fn rueckwand() -> Option<ApiBackend> {
    nokhwa::native_api_backend()
}

/// Die gefundenen Kameras, in der Reihenfolge des Treibers.
///
/// **Leer ist keine Ausnahme.** Ein Rechner ohne Kamera bekommt hier eine
/// leere Liste und keinen Fehler - genau wie bei pygame, und `camera_test`
/// rechnet damit: seine Tests ueberspringen sich selbst, wenn die Liste
/// leer ist.
#[must_use]
pub fn geraete() -> Vec<Geraet> {
    let Some(api) = rueckwand() else {
        return Vec::new();
    };
    let Ok(liste) = nokhwa::query(api) else {
        return Vec::new();
    };
    liste
        .into_iter()
        .map(|g| Geraet {
            name: g.human_name(),
            beschreibung: g.description().to_string(),
            nummer: g.index().as_index().unwrap_or(0),
        })
        .collect()
}

/// Was beim Kamerazugriff schiefgehen kann.
///
/// Die Texte sind die von pygame, und die Unterscheidung in zwei Sorten
/// ist es auch: ein unbekannter Name ist dort ein `ValueError`, ein
/// Geraet, das sich nicht oeffnen laesst, ein `pygame.error`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fehler {
    /// Es gibt keine Kamera mit diesem Namen.
    ///
    /// pygame sagt woertlich `Couldn't find a camera with that name` und
    /// wirft es als `ValueError`.
    UnbekannterName,
    /// Das Geraet liess sich nicht oeffnen oder nicht lesen.
    Geraet(String),
    /// Es wurde gelesen, ohne vorher zu starten.
    ///
    /// pygame sagt woertlich `Camera needs to be started to read data`.
    NichtGestartet,
}

impl std::fmt::Display for Fehler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Fehler::UnbekannterName => write!(f, "Couldn't find a camera with that name"),
            Fehler::Geraet(s) => write!(f, "{s}"),
            Fehler::NichtGestartet => write!(f, "Camera needs to be started to read data"),
        }
    }
}

impl std::error::Error for Fehler {}

/// Ein Bild, wie es vom Geraet kommt.
#[derive(Debug, Clone)]
pub struct Bild {
    /// Drei Byte je Pixel, zeilenweise von oben nach unten.
    pub rgb: Vec<u8>,
    /// Breite in Pixeln.
    pub w: u32,
    /// Hoehe in Pixeln.
    pub h: u32,
}

/// Eine Kamera.
///
/// Gebaut wird sie ohne Strom: `neu` sucht das Geraet, waehlt das Format
/// und fragt die Groesse ab - lesen laesst sich erst nach [`Kamera::starten`].
/// Genau diese Reihenfolge hat pygame auch, und sie ist messbar:
/// `get_size()` antwortet vor `start()`, `query_image()` nicht.
pub struct Kamera {
    geraet: nokhwa::Camera,
    laeuft: bool,
    /// Die Groesse, die beim Bauen gewuenscht wurde.
    ///
    /// **Wird gebraucht, nicht nur gemerkt.** Siehe [`Kamera::groesse`]:
    /// vor dem Start nennt pygame den Wunsch, nach dem Start die
    /// Wirklichkeit.
    wunsch: (u32, u32),
    /// Waagerecht spiegeln - rechnet diese Kiste selbst.
    pub hflip: bool,
    /// Senkrecht spiegeln - dito.
    pub vflip: bool,
}

/// Die Bildformate, aus denen `RgbFormat` ein RGB-Bild machen kann.
///
/// **Nicht geraten, sondern abgefragt.** `nokhwa`s `Closest` scheitert,
/// wenn das gewuenschte [`FrameFormat`] am Geraet nicht vorkommt - also
/// wird hier selbst gesucht, und zwar nur unter den Formaten, die der
/// Umrechner danach auch lesen kann.
fn lesbare_formate() -> HashSet<FrameFormat> {
    <RgbFormat as FormatDecoder>::FORMATS.iter().copied().collect()
}

impl Kamera {
    /// Sucht das Geraet mit diesem Namen und waehlt ein Format dazu.
    ///
    /// `(w, h)` ist ein **Wunsch**: gewaehlt wird das naechstliegende
    /// Format, das das Geraet wirklich hat, und bei gleichem Abstand das
    /// mit der hoeheren Bildrate. Welche Groesse daraus wurde, sagt
    /// [`Kamera::groesse`] - so macht es pygame auch.
    ///
    /// # Fehler
    ///
    /// [`Fehler::UnbekannterName`], wenn kein Geraet so heisst;
    /// [`Fehler::Geraet`], wenn es sich nicht oeffnen laesst.
    pub fn neu(name: &str, w: u32, h: u32) -> Result<Kamera, Fehler> {
        let nummer = geraete()
            .into_iter()
            .find(|g| g.name == name)
            .map(|g| g.nummer)
            .ok_or(Fehler::UnbekannterName)?;
        // Erst ohne Wunsch oeffnen - sonst gibt es keine Liste, aus der
        // sich der Wunsch erfuellen liesse.
        let wunsch = RequestedFormat::new::<RgbFormat>(RequestedFormatType::None);
        let mut geraet = nokhwa::Camera::new(CameraIndex::Index(nummer), wunsch)
            .map_err(|e| Fehler::Geraet(e.to_string()))?;
        if let Some(fmt) = Kamera::nahestes_format(&mut geraet, w, h) {
            // Ein Fehlschlag ist hier keiner: dann bleibt das Format, mit
            // dem geoeffnet wurde, und `groesse()` sagt die Wahrheit
            // darueber. pygame verhaelt sich genauso - die Groesse ist
            // ein Wunsch.
            // `set_camera_requset` und nicht `set_camera_format`: der
            // zweite ist in `nokhwa 0.10` als veraltet gekennzeichnet (der
            // Schreibfehler im Namen des ersten ist dort so).
            let genau = RequestedFormat::new::<RgbFormat>(RequestedFormatType::Exact(fmt));
            let _ = geraet.set_camera_requset(genau);
        }
        Ok(Kamera {
            geraet,
            laeuft: false,
            wunsch: (w, h),
            hflip: false,
            vflip: false,
        })
    }

    /// Das Format des Geraets, das dem Wunsch am naechsten kommt.
    fn nahestes_format(geraet: &mut nokhwa::Camera, w: u32, h: u32) -> Option<CameraFormat> {
        let lesbar = lesbare_formate();
        let alle = geraet.compatible_camera_formats().ok()?;
        alle.into_iter()
            .filter(|f| lesbar.contains(&f.format()))
            .min_by_key(|f| {
                let dx = i64::from(f.resolution().x()) - i64::from(w);
                let dy = i64::from(f.resolution().y()) - i64::from(h);
                // Abstand im Quadrat, dann die hoehere Bildrate zuerst.
                (dx * dx + dy * dy, -i64::from(f.frame_rate()))
            })
    }

    /// Vor dem Start der **Wunsch**, danach die **Wirklichkeit**.
    ///
    /// Das ist keine Nachlaessigkeit, sondern gemessen - und es hat mich
    /// beim ersten Messen hereingelegt. pygame mit `Camera(name, (1,1))`:
    ///
    /// ```text
    /// get_size() vor  start()   (1, 1)        der Wunsch
    /// get_size() nach start()   (320, 240)    was das Geraet kann
    /// ```
    ///
    /// Die erste Messung hier hatte vorher gestartet und darum nur die
    /// zweite Zeile gesehen; die zweite hatte nicht gestartet und nur die
    /// erste. Beide stimmten, und zusammen ergaben sie einen Widerspruch,
    /// der keiner war. **Zwei Messungen, die sich widersprechen, messen
    /// meist zwei verschiedene Zustaende.**
    #[must_use]
    pub fn groesse(&self) -> (u32, u32) {
        if !self.laeuft {
            return self.wunsch;
        }
        let r: Resolution = self.geraet.resolution();
        (r.x(), r.y())
    }

    /// Faengt an, Bilder zu holen. Zweimal starten ist kein Fehler.
    ///
    /// # Fehler
    ///
    /// [`Fehler::Geraet`], wenn der Strom sich nicht oeffnen laesst - etwa
    /// weil ein anderes Programm die Kamera schon hat.
    pub fn starten(&mut self) -> Result<(), Fehler> {
        if self.laeuft {
            return Ok(());
        }
        self.geraet
            .open_stream()
            .map_err(|e| Fehler::Geraet(e.to_string()))?;
        self.laeuft = true;
        Ok(())
    }

    /// Hoert auf. Zweimal aufhoeren ist kein Fehler - gemessen gibt
    /// pygames `stop()` beide Male `None`.
    pub fn stoppen(&mut self) {
        if self.laeuft {
            let _ = self.geraet.stop_stream();
            self.laeuft = false;
        }
    }

    /// Laeuft sie?
    #[must_use]
    pub fn laeuft(&self) -> bool {
        self.laeuft
    }

    /// Die Helligkeit, wie der Treiber sie meldet - `-1`, wenn er sie
    /// nicht kennt.
    ///
    /// **`-1` ist nicht "dunkel", sondern "weiss nicht".** pygame gibt
    /// auf dieser Maschine `-1` zurueck, weil sein MSMF-Stueck den Regler
    /// ueberhaupt nicht abfragt. Hier wird er abgefragt; hat das Geraet
    /// keinen, steht wieder `-1` da.
    #[must_use]
    pub fn helligkeit(&self) -> i32 {
        match self.geraet.camera_control(KnownCameraControl::Brightness) {
            Ok(c) => i32::try_from(c.value().as_integer().copied().unwrap_or(-1)).unwrap_or(-1),
            Err(_) => -1,
        }
    }

    /// Stellt die Helligkeit - `false`, wenn das Geraet sie nicht hat.
    pub fn helligkeit_setzen(&mut self, wert: i32) -> bool {
        use nokhwa::utils::ControlValueSetter;
        self.geraet
            .set_camera_control(
                KnownCameraControl::Brightness,
                ControlValueSetter::Integer(i64::from(wert)),
            )
            .is_ok()
    }

    /// Holt ein Bild, schon gespiegelt.
    ///
    /// # Fehler
    ///
    /// [`Fehler::NichtGestartet`] vor [`Kamera::starten`],
    /// [`Fehler::Geraet`], wenn das Bild nicht kommt oder sich nicht
    /// umrechnen laesst.
    pub fn bild(&mut self) -> Result<Bild, Fehler> {
        if !self.laeuft {
            return Err(Fehler::NichtGestartet);
        }
        let rahmen = self
            .geraet
            .frame()
            .map_err(|e| Fehler::Geraet(e.to_string()))?;
        let r = rahmen.resolution();
        let (w, h) = (r.x(), r.y());
        let rgb = rahmen
            .decode_image::<RgbFormat>()
            .map_err(|e| Fehler::Geraet(e.to_string()))?
            .into_raw();
        Ok(Bild {
            rgb: spiegeln(rgb, w, h, self.hflip, self.vflip),
            w,
            h,
        })
    }

    /// Die rohen Bytes, wie das Geraet sie liefert - **undekodiert**.
    ///
    /// `None`, solange nicht gestartet ist.
    ///
    /// ## Eine benannte Abweichung, und sie ist ein Plus
    ///
    /// pygames `get_raw()` gibt auf dieser Maschine `None`: sein
    /// MSMF-Stueck hat den Aufruf nicht umgesetzt. Unter Linux gibt
    /// dasselbe pygame die Bytes des V4L2-Puffers heraus. Hier gibt es sie
    /// auf jeder Plattform.
    ///
    /// Das ist der einzige Punkt, an dem diese Kiste absichtlich **mehr**
    /// tut als pygame unter Windows - eine Luecke nachzubauen, waere die
    /// schlechtere Antwort. Es steht in den benannten Grenzen, damit die
    /// Messung es nicht fuer einen Fehler haelt.
    pub fn roh(&mut self) -> Option<Vec<u8>> {
        if !self.laeuft {
            return None;
        }
        self.geraet.frame().ok().map(|r| r.buffer().to_vec())
    }
}

/// Spiegelt ein RGB-Bild an einer oder beiden Achsen.
///
/// Steht hier und nicht in `transform`, weil es auf **drei** Byte je Pixel
/// arbeitet: das Bild ist zu diesem Zeitpunkt noch keine Flaeche.
fn spiegeln(mut rgb: Vec<u8>, w: u32, h: u32, hflip: bool, vflip: bool) -> Vec<u8> {
    let (w, h) = (w as usize, h as usize);
    let zeile = w * 3;
    if rgb.len() < zeile * h {
        return rgb;
    }
    if hflip {
        for y in 0..h {
            let a = y * zeile;
            for x in 0..w / 2 {
                let (l, r) = (a + x * 3, a + (w - 1 - x) * 3);
                for k in 0..3 {
                    rgb.swap(l + k, r + k);
                }
            }
        }
    }
    if vflip {
        for y in 0..h / 2 {
            let (o, u) = (y * zeile, (h - 1 - y) * zeile);
            for k in 0..zeile {
                rgb.swap(o + k, u + k);
            }
        }
    }
    rgb
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Die Geraeteliste darf leer sein, aber nicht widerspruechlich.
    ///
    /// Auf einem Rechner ohne Kamera prueft dieser Test die leere Liste -
    /// und das ist Absicht: er muss **ueberall** laufen.
    #[test]
    fn die_geraeteliste_ist_widerspruchsfrei() {
        for g in geraete() {
            assert!(!g.name.is_empty(), "ein Geraet ohne Namen");
        }
    }

    /// Ein Name, den es nicht gibt, ist ein klarer Fehler mit pygames Text.
    #[test]
    fn ein_unbekannter_name_gibt_einen_fehler() {
        let f = Kamera::neu("gibtesnicht-xyzzy", 320, 240).err();
        assert_eq!(f, Some(Fehler::UnbekannterName));
        assert_eq!(
            Fehler::UnbekannterName.to_string(),
            "Couldn't find a camera with that name"
        );
    }

    /// Die Wege zur Kamera sind entweder einer oder keiner.
    #[test]
    fn es_gibt_hoechstens_einen_rueckweg() {
        let r = rueckwaende();
        assert!(r.len() <= 1);
        assert_eq!(r.is_empty(), rueckwand().is_none());
    }

    /// Spiegeln an beiden Achsen zweimal ist das Original.
    #[test]
    fn zweimal_spiegeln_gibt_das_original() {
        let urbild: Vec<u8> = (0..(4 * 3 * 3u32)).map(|i| i as u8).collect();
        for (hf, vf) in [(true, false), (false, true), (true, true)] {
            let einmal = spiegeln(urbild.clone(), 4, 3, hf, vf);
            assert_ne!(einmal, urbild, "spiegeln hat nichts getan");
            assert_eq!(spiegeln(einmal, 4, 3, hf, vf), urbild);
        }
    }

    /// Ein Pixel bleibt ein Pixel: die drei Byte wandern zusammen.
    #[test]
    fn spiegeln_laesst_die_farben_zusammen() {
        // Zwei Pixel, rot und gruen.
        let bild = vec![255, 0, 0, 0, 255, 0];
        assert_eq!(spiegeln(bild, 2, 1, true, false), vec![0, 255, 0, 255, 0, 0]);
    }

    /// Ein zu kurzer Puffer wird nicht angefasst, statt zu platzen.
    #[test]
    fn ein_zu_kurzer_puffer_bleibt_liegen() {
        let kurz = vec![1, 2, 3];
        assert_eq!(spiegeln(kurz.clone(), 4, 3, true, true), kurz);
    }
}
