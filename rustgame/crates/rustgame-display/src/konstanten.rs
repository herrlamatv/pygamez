//! Konstanten von pygame-ce 2.5.7 (SDL 2.32.10)
//! ==========================================
//!
//! **Diese Datei ist erzeugt - nicht von Hand aendern.**
//!
//! Quelle: `pygame.locals` aus dem echten pygame-ce im `.venv` des
//! Projekts, ausgelesen von `tools/export_display_defs.py` neben
//! dieser Kiste. Neu erzeugen (aus dem Projektordner `pygamez`):
//!
//! ```text
//! .venv/Scripts/python.exe \
//!     rustgame/crates/rustgame-display/tools/export_display_defs.py
//! ```
//!
//! Das Skript braucht kein Fenster - es setzt selbst
//! `SDL_VIDEODRIVER=dummy`.
//!
//! Warum erzeugt und nicht getippt: ein Zahlendreher in einem der
//! knapp 600 Werte faellt nicht beim Uebersetzen auf, sondern erst,
//! wenn Monate spaeter die falsche Taste gemeldet wird. Das Skript
//! prueft beim Erzeugen ausserdem nach, dass **jeder** ganzzahlige
//! und jeder Zeichenketten-Name aus `pygame.locals` hier steht -
//! eine vergessene Gruppe faellt damit sofort auf.

#![allow(non_upper_case_globals)]

/// Ein Tastencode (`K_*`), wie ihn `Event::Key` fuehrt.
pub type Key = u32;
/// Ein Tastenplatz (`KSCAN_*`) - die *Stelle* auf der Tastatur,
/// unabhaengig von der Belegung.
pub type Scancode = u32;
/// Ein Ereignistyp (`QUIT`, `KEYDOWN`, ...).
pub type EventType = u32;
/// Ein Satz Zusatztasten (`KMOD_*`), als Bitmaske.
pub type KeyMod = u32;

// ---------------------------------------------------------------
// Ereignistypen
// ---------------------------------------------------------------

/// `NoEvent` - Ereignistyp 0.
pub const NOEVENT: EventType = 0;
/// `Quit` - Ereignistyp 256.
pub const QUIT: EventType = 256;
/// `ActiveEvent` - Ereignistyp 32768.
pub const ACTIVEEVENT: EventType = 32768;
/// `KeyDown` - Ereignistyp 768.
pub const KEYDOWN: EventType = 768;
/// `KeyUp` - Ereignistyp 769.
pub const KEYUP: EventType = 769;
/// `TextEditing` - Ereignistyp 770.
pub const TEXTEDITING: EventType = 770;
/// `TextInput` - Ereignistyp 771.
pub const TEXTINPUT: EventType = 771;
/// `KeyMapChanged` - Ereignistyp 772.
pub const KEYMAPCHANGED: EventType = 772;
/// `MouseMotion` - Ereignistyp 1024.
pub const MOUSEMOTION: EventType = 1024;
/// `MouseButtonDown` - Ereignistyp 1025.
pub const MOUSEBUTTONDOWN: EventType = 1025;
/// `MouseButtonUp` - Ereignistyp 1026.
pub const MOUSEBUTTONUP: EventType = 1026;
/// `MouseWheel` - Ereignistyp 1027.
pub const MOUSEWHEEL: EventType = 1027;
/// `JoyAxisMotion` - Ereignistyp 1536.
pub const JOYAXISMOTION: EventType = 1536;
/// `JoyBallMotion` - Ereignistyp 1537.
pub const JOYBALLMOTION: EventType = 1537;
/// `JoyHatMotion` - Ereignistyp 1538.
pub const JOYHATMOTION: EventType = 1538;
/// `JoyButtonDown` - Ereignistyp 1539.
pub const JOYBUTTONDOWN: EventType = 1539;
/// `JoyButtonUp` - Ereignistyp 1540.
pub const JOYBUTTONUP: EventType = 1540;
/// `JoyDeviceAdded` - Ereignistyp 1541.
pub const JOYDEVICEADDED: EventType = 1541;
/// `JoyDeviceRemoved` - Ereignistyp 1542.
pub const JOYDEVICEREMOVED: EventType = 1542;
/// `ControllerAxisMotion` - Ereignistyp 1616.
pub const CONTROLLERAXISMOTION: EventType = 1616;
/// `ControllerButtonDown` - Ereignistyp 1617.
pub const CONTROLLERBUTTONDOWN: EventType = 1617;
/// `ControllerButtonUp` - Ereignistyp 1618.
pub const CONTROLLERBUTTONUP: EventType = 1618;
/// `ControllerDeviceAdded` - Ereignistyp 1619.
pub const CONTROLLERDEVICEADDED: EventType = 1619;
/// `ControllerDeviceRemoved` - Ereignistyp 1620.
pub const CONTROLLERDEVICEREMOVED: EventType = 1620;
/// `ControllerDeviceMapped` - Ereignistyp 1621.
pub const CONTROLLERDEVICEREMAPPED: EventType = 1621;
/// `ControllerTouchpadDown` - Ereignistyp 1622.
pub const CONTROLLERTOUCHPADDOWN: EventType = 1622;
/// `ControllerTouchpadMotion` - Ereignistyp 1623.
pub const CONTROLLERTOUCHPADMOTION: EventType = 1623;
/// `ControllerTouchpadUp` - Ereignistyp 1624.
pub const CONTROLLERTOUCHPADUP: EventType = 1624;
/// `ControllerSensorUpdate` - Ereignistyp 1625.
pub const CONTROLLERSENSORUPDATE: EventType = 1625;
/// `FingerDown` - Ereignistyp 1792.
pub const FINGERDOWN: EventType = 1792;
/// `FingerUp` - Ereignistyp 1793.
pub const FINGERUP: EventType = 1793;
/// `FingerMotion` - Ereignistyp 1794.
pub const FINGERMOTION: EventType = 1794;
/// `MultiGesture` - Ereignistyp 2050.
pub const MULTIGESTURE: EventType = 2050;
/// `ClipboardUpdate` - Ereignistyp 2304.
pub const CLIPBOARDUPDATE: EventType = 2304;
/// `DropFile` - Ereignistyp 4096.
pub const DROPFILE: EventType = 4096;
/// `DropText` - Ereignistyp 4097.
pub const DROPTEXT: EventType = 4097;
/// `DropBegin` - Ereignistyp 4098.
pub const DROPBEGIN: EventType = 4098;
/// `DropComplete` - Ereignistyp 4099.
pub const DROPCOMPLETE: EventType = 4099;
/// `AudioDeviceAdded` - Ereignistyp 4352.
pub const AUDIODEVICEADDED: EventType = 4352;
/// `AudioDeviceRemoved` - Ereignistyp 4353.
pub const AUDIODEVICEREMOVED: EventType = 4353;
/// `RenderTargetsReset` - Ereignistyp 8192.
pub const RENDER_TARGETS_RESET: EventType = 8192;
/// `RenderDeviceReset` - Ereignistyp 8193.
pub const RENDER_DEVICE_RESET: EventType = 8193;
/// `LocaleChanged` - Ereignistyp 263.
pub const LOCALECHANGED: EventType = 263;
/// `AppTerminating` - Ereignistyp 257.
pub const APP_TERMINATING: EventType = 257;
/// `AppLowMemory` - Ereignistyp 258.
pub const APP_LOWMEMORY: EventType = 258;
/// `AppWillEnterBackground` - Ereignistyp 259.
pub const APP_WILLENTERBACKGROUND: EventType = 259;
/// `AppDidEnterBackground` - Ereignistyp 260.
pub const APP_DIDENTERBACKGROUND: EventType = 260;
/// `AppWillEnterForeground` - Ereignistyp 261.
pub const APP_WILLENTERFOREGROUND: EventType = 261;
/// `AppDidEnterForeground` - Ereignistyp 262.
pub const APP_DIDENTERFOREGROUND: EventType = 262;
/// `VideoResize` - Ereignistyp 32769.
pub const VIDEORESIZE: EventType = 32769;
/// `VideoExpose` - Ereignistyp 32770.
pub const VIDEOEXPOSE: EventType = 32770;
/// `SysWMEvent` - Ereignistyp 513.
pub const SYSWMEVENT: EventType = 513;
/// `WindowShown` - Ereignistyp 32773.
pub const WINDOWSHOWN: EventType = 32773;
/// `WindowHidden` - Ereignistyp 32774.
pub const WINDOWHIDDEN: EventType = 32774;
/// `WindowExposed` - Ereignistyp 32775.
pub const WINDOWEXPOSED: EventType = 32775;
/// `WindowMoved` - Ereignistyp 32776.
pub const WINDOWMOVED: EventType = 32776;
/// `WindowResized` - Ereignistyp 32777.
pub const WINDOWRESIZED: EventType = 32777;
/// `WindowSizeChanged` - Ereignistyp 32778.
pub const WINDOWSIZECHANGED: EventType = 32778;
/// `WindowMinimized` - Ereignistyp 32779.
pub const WINDOWMINIMIZED: EventType = 32779;
/// `WindowMaximized` - Ereignistyp 32780.
pub const WINDOWMAXIMIZED: EventType = 32780;
/// `WindowRestored` - Ereignistyp 32781.
pub const WINDOWRESTORED: EventType = 32781;
/// `WindowEnter` - Ereignistyp 32782.
pub const WINDOWENTER: EventType = 32782;
/// `WindowLeave` - Ereignistyp 32783.
pub const WINDOWLEAVE: EventType = 32783;
/// `WindowFocusGained` - Ereignistyp 32784.
pub const WINDOWFOCUSGAINED: EventType = 32784;
/// `WindowFocusLost` - Ereignistyp 32785.
pub const WINDOWFOCUSLOST: EventType = 32785;
/// `WindowClose` - Ereignistyp 32786.
pub const WINDOWCLOSE: EventType = 32786;
/// `WindowTakeFocus` - Ereignistyp 32787.
pub const WINDOWTAKEFOCUS: EventType = 32787;
/// `WindowHitTest` - Ereignistyp 32788.
pub const WINDOWHITTEST: EventType = 32788;
/// `WindowICCProfChanged` - Ereignistyp 32789.
pub const WINDOWICCPROFCHANGED: EventType = 32789;
/// `WindowDisplayChanged` - Ereignistyp 32790.
pub const WINDOWDISPLAYCHANGED: EventType = 32790;
/// `MidiIn` - Ereignistyp 32771.
pub const MIDIIN: EventType = 32771;
/// `MidiOut` - Ereignistyp 32772.
pub const MIDIOUT: EventType = 32772;
/// `UserEvent` - Ereignistyp 32865.
pub const USEREVENT: EventType = 32865;
/// `DropFile` - Ereignistyp 4096.
pub const USEREVENT_DROPFILE: EventType = 4096;
/// `Unknown` - Ereignistyp 65535.
pub const NUMEVENTS: EventType = 65535;

// ---------------------------------------------------------------
// Fenster- und Flaechen-Flaggen
// ---------------------------------------------------------------

/// `pygame.SWSURFACE`.
pub const SWSURFACE: u32 = 0;
/// `pygame.HWSURFACE`.
pub const HWSURFACE: u32 = 1;
/// `pygame.ASYNCBLIT`.
pub const ASYNCBLIT: u32 = 4;
/// `pygame.ANYFORMAT`.
pub const ANYFORMAT: u32 = 268435456;
/// `pygame.HWPALETTE`.
pub const HWPALETTE: u32 = 536870912;
/// `pygame.DOUBLEBUF`.
pub const DOUBLEBUF: u32 = 1073741824;
/// `pygame.FULLSCREEN`.
/// In pygame steht hier -2147483648 - CPython macht aus dem obersten
/// SDL-Bit ein Vorzeichen. Derselbe Bitwert als `u32`.
pub const FULLSCREEN: u32 = 0x80000000;
/// `pygame.OPENGL`.
pub const OPENGL: u32 = 2;
/// `pygame.OPENGLBLIT`.
pub const OPENGLBLIT: u32 = 10;
/// `pygame.RESIZABLE`.
pub const RESIZABLE: u32 = 16;
/// `pygame.NOFRAME`.
pub const NOFRAME: u32 = 32;
/// `pygame.SCALED`.
pub const SCALED: u32 = 512;
/// `pygame.SHOWN`.
pub const SHOWN: u32 = 64;
/// `pygame.HIDDEN`.
pub const HIDDEN: u32 = 128;
/// `pygame.SRCALPHA`.
pub const SRCALPHA: u32 = 65536;
/// `pygame.SRCCOLORKEY`.
pub const SRCCOLORKEY: u32 = 4096;
/// `pygame.RLEACCELOK`.
pub const RLEACCELOK: u32 = 8192;
/// `pygame.RLEACCEL`.
pub const RLEACCEL: u32 = 16384;
/// `pygame.PREALLOC`.
pub const PREALLOC: u32 = 16777216;

// ---------------------------------------------------------------
// Tastencodes (K_*)
// ---------------------------------------------------------------

/// Taste `0`.
pub const K_0: Key = 48;
/// Taste `1`.
pub const K_1: Key = 49;
/// Taste `2`.
pub const K_2: Key = 50;
/// Taste `3`.
pub const K_3: Key = 51;
/// Taste `4`.
pub const K_4: Key = 52;
/// Taste `5`.
pub const K_5: Key = 53;
/// Taste `6`.
pub const K_6: Key = 54;
/// Taste `7`.
pub const K_7: Key = 55;
/// Taste `8`.
pub const K_8: Key = 56;
/// Taste `9`.
pub const K_9: Key = 57;
/// Taste `AC Back`.
pub const K_AC_BACK: Key = 1073742094;
/// Taste `&`.
pub const K_AMPERSAND: Key = 38;
/// Taste `*`.
pub const K_ASTERISK: Key = 42;
/// Taste `@`.
pub const K_AT: Key = 64;
/// Taste ```.
pub const K_BACKQUOTE: Key = 96;
/// Taste `\`.
pub const K_BACKSLASH: Key = 92;
/// Taste `backspace` (SDL nennt sie `Backspace`).
pub const K_BACKSPACE: Key = 8;
/// Taste `break` (SDL nennt sie `Pause`).
pub const K_BREAK: Key = 1073741896;
/// Taste `caps lock` (SDL nennt sie `CapsLock`).
pub const K_CAPSLOCK: Key = 1073741881;
/// Taste `^`.
pub const K_CARET: Key = 94;
/// Taste `clear` (SDL nennt sie `Clear`).
pub const K_CLEAR: Key = 1073741980;
/// Taste `:`.
pub const K_COLON: Key = 58;
/// Taste `,`.
pub const K_COMMA: Key = 44;
/// Taste `CurrencySubUnit`.
pub const K_CURRENCYSUBUNIT: Key = 1073742005;
/// Taste `euro` (SDL nennt sie `CurrencyUnit`).
pub const K_CURRENCYUNIT: Key = 1073742004;
/// Taste `delete` (SDL nennt sie `Delete`).
pub const K_DELETE: Key = 127;
/// Taste `$`.
pub const K_DOLLAR: Key = 36;
/// Taste `down` (SDL nennt sie `Down`).
pub const K_DOWN: Key = 1073741905;
/// Taste `end` (SDL nennt sie `End`).
pub const K_END: Key = 1073741901;
/// Taste `=`.
pub const K_EQUALS: Key = 61;
/// Taste `escape` (SDL nennt sie `Escape`).
pub const K_ESCAPE: Key = 27;
/// Taste `euro` (SDL nennt sie `CurrencyUnit`).
pub const K_EURO: Key = 1073742004;
/// Taste `!`.
pub const K_EXCLAIM: Key = 33;
/// Taste `f1` (SDL nennt sie `F1`).
pub const K_F1: Key = 1073741882;
/// Taste `f10` (SDL nennt sie `F10`).
pub const K_F10: Key = 1073741891;
/// Taste `f11` (SDL nennt sie `F11`).
pub const K_F11: Key = 1073741892;
/// Taste `f12` (SDL nennt sie `F12`).
pub const K_F12: Key = 1073741893;
/// Taste `f13` (SDL nennt sie `F13`).
pub const K_F13: Key = 1073741928;
/// Taste `f14` (SDL nennt sie `F14`).
pub const K_F14: Key = 1073741929;
/// Taste `f15` (SDL nennt sie `F15`).
pub const K_F15: Key = 1073741930;
/// Taste `f2` (SDL nennt sie `F2`).
pub const K_F2: Key = 1073741883;
/// Taste `f3` (SDL nennt sie `F3`).
pub const K_F3: Key = 1073741884;
/// Taste `f4` (SDL nennt sie `F4`).
pub const K_F4: Key = 1073741885;
/// Taste `f5` (SDL nennt sie `F5`).
pub const K_F5: Key = 1073741886;
/// Taste `f6` (SDL nennt sie `F6`).
pub const K_F6: Key = 1073741887;
/// Taste `f7` (SDL nennt sie `F7`).
pub const K_F7: Key = 1073741888;
/// Taste `f8` (SDL nennt sie `F8`).
pub const K_F8: Key = 1073741889;
/// Taste `f9` (SDL nennt sie `F9`).
pub const K_F9: Key = 1073741890;
/// Taste `>`.
pub const K_GREATER: Key = 62;
/// Taste `#`.
pub const K_HASH: Key = 35;
/// Taste `help` (SDL nennt sie `Help`).
pub const K_HELP: Key = 1073741941;
/// Taste `home` (SDL nennt sie `Home`).
pub const K_HOME: Key = 1073741898;
/// Taste `insert` (SDL nennt sie `Insert`).
pub const K_INSERT: Key = 1073741897;
/// Taste `[0]` (SDL nennt sie `Keypad 0`).
pub const K_KP0: Key = 1073741922;
/// Taste `[1]` (SDL nennt sie `Keypad 1`).
pub const K_KP1: Key = 1073741913;
/// Taste `[2]` (SDL nennt sie `Keypad 2`).
pub const K_KP2: Key = 1073741914;
/// Taste `[3]` (SDL nennt sie `Keypad 3`).
pub const K_KP3: Key = 1073741915;
/// Taste `[4]` (SDL nennt sie `Keypad 4`).
pub const K_KP4: Key = 1073741916;
/// Taste `[5]` (SDL nennt sie `Keypad 5`).
pub const K_KP5: Key = 1073741917;
/// Taste `[6]` (SDL nennt sie `Keypad 6`).
pub const K_KP6: Key = 1073741918;
/// Taste `[7]` (SDL nennt sie `Keypad 7`).
pub const K_KP7: Key = 1073741919;
/// Taste `[8]` (SDL nennt sie `Keypad 8`).
pub const K_KP8: Key = 1073741920;
/// Taste `[9]` (SDL nennt sie `Keypad 9`).
pub const K_KP9: Key = 1073741921;
/// Taste `[0]` (SDL nennt sie `Keypad 0`).
pub const K_KP_0: Key = 1073741922;
/// Taste `[1]` (SDL nennt sie `Keypad 1`).
pub const K_KP_1: Key = 1073741913;
/// Taste `[2]` (SDL nennt sie `Keypad 2`).
pub const K_KP_2: Key = 1073741914;
/// Taste `[3]` (SDL nennt sie `Keypad 3`).
pub const K_KP_3: Key = 1073741915;
/// Taste `[4]` (SDL nennt sie `Keypad 4`).
pub const K_KP_4: Key = 1073741916;
/// Taste `[5]` (SDL nennt sie `Keypad 5`).
pub const K_KP_5: Key = 1073741917;
/// Taste `[6]` (SDL nennt sie `Keypad 6`).
pub const K_KP_6: Key = 1073741918;
/// Taste `[7]` (SDL nennt sie `Keypad 7`).
pub const K_KP_7: Key = 1073741919;
/// Taste `[8]` (SDL nennt sie `Keypad 8`).
pub const K_KP_8: Key = 1073741920;
/// Taste `[9]` (SDL nennt sie `Keypad 9`).
pub const K_KP_9: Key = 1073741921;
/// Taste `[/]` (SDL nennt sie `Keypad /`).
pub const K_KP_DIVIDE: Key = 1073741908;
/// Taste `enter` (SDL nennt sie `Keypad Enter`).
pub const K_KP_ENTER: Key = 1073741912;
/// Taste `equals` (SDL nennt sie `Keypad =`).
pub const K_KP_EQUALS: Key = 1073741927;
/// Taste `[-]` (SDL nennt sie `Keypad -`).
pub const K_KP_MINUS: Key = 1073741910;
/// Taste `[*]` (SDL nennt sie `Keypad *`).
pub const K_KP_MULTIPLY: Key = 1073741909;
/// Taste `[.]` (SDL nennt sie `Keypad .`).
pub const K_KP_PERIOD: Key = 1073741923;
/// Taste `[+]` (SDL nennt sie `Keypad +`).
pub const K_KP_PLUS: Key = 1073741911;
/// Taste `left alt` (SDL nennt sie `Left Alt`).
pub const K_LALT: Key = 1073742050;
/// Taste `left ctrl` (SDL nennt sie `Left Ctrl`).
pub const K_LCTRL: Key = 1073742048;
/// Taste `left` (SDL nennt sie `Left`).
pub const K_LEFT: Key = 1073741904;
/// Taste `[`.
pub const K_LEFTBRACKET: Key = 91;
/// Taste `(`.
pub const K_LEFTPAREN: Key = 40;
/// Taste `<`.
pub const K_LESS: Key = 60;
/// Taste `left meta` (SDL nennt sie `Left GUI`).
pub const K_LGUI: Key = 1073742051;
/// Taste `left meta` (SDL nennt sie `Left GUI`).
pub const K_LMETA: Key = 1073742051;
/// Taste `left shift` (SDL nennt sie `Left Shift`).
pub const K_LSHIFT: Key = 1073742049;
/// Taste `left meta` (SDL nennt sie `Left GUI`).
pub const K_LSUPER: Key = 1073742051;
/// Taste `menu` (SDL nennt sie `Menu`).
pub const K_MENU: Key = 1073741942;
/// Taste `-`.
pub const K_MINUS: Key = 45;
/// Taste `alt gr` (SDL nennt sie `ModeSwitch`).
pub const K_MODE: Key = 1073742081;
/// Taste `numlock` (SDL nennt sie `Numlock`).
pub const K_NUMLOCK: Key = 1073741907;
/// Taste `numlock` (SDL nennt sie `Numlock`).
pub const K_NUMLOCKCLEAR: Key = 1073741907;
/// Taste `page down` (SDL nennt sie `PageDown`).
pub const K_PAGEDOWN: Key = 1073741902;
/// Taste `page up` (SDL nennt sie `PageUp`).
pub const K_PAGEUP: Key = 1073741899;
/// Taste `break` (SDL nennt sie `Pause`).
pub const K_PAUSE: Key = 1073741896;
/// Taste `%`.
pub const K_PERCENT: Key = 37;
/// Taste `.`.
pub const K_PERIOD: Key = 46;
/// Taste `+`.
pub const K_PLUS: Key = 43;
/// Taste `power` (SDL nennt sie `Power`).
pub const K_POWER: Key = 1073741926;
/// Taste `print screen` (SDL nennt sie `PrintScreen`).
pub const K_PRINT: Key = 1073741894;
/// Taste `print screen` (SDL nennt sie `PrintScreen`).
pub const K_PRINTSCREEN: Key = 1073741894;
/// Taste `?`.
pub const K_QUESTION: Key = 63;
/// Taste `'`.
pub const K_QUOTE: Key = 39;
/// Taste `"`.
pub const K_QUOTEDBL: Key = 34;
/// Taste `right alt` (SDL nennt sie `Right Alt`).
pub const K_RALT: Key = 1073742054;
/// Taste `right ctrl` (SDL nennt sie `Right Ctrl`).
pub const K_RCTRL: Key = 1073742052;
/// Taste `return` (SDL nennt sie `Return`).
pub const K_RETURN: Key = 13;
/// Taste `right meta` (SDL nennt sie `Right GUI`).
pub const K_RGUI: Key = 1073742055;
/// Taste `right` (SDL nennt sie `Right`).
pub const K_RIGHT: Key = 1073741903;
/// Taste `]`.
pub const K_RIGHTBRACKET: Key = 93;
/// Taste `)`.
pub const K_RIGHTPAREN: Key = 41;
/// Taste `right meta` (SDL nennt sie `Right GUI`).
pub const K_RMETA: Key = 1073742055;
/// Taste `right shift` (SDL nennt sie `Right Shift`).
pub const K_RSHIFT: Key = 1073742053;
/// Taste `right meta` (SDL nennt sie `Right GUI`).
pub const K_RSUPER: Key = 1073742055;
/// Taste `scroll lock` (SDL nennt sie `ScrollLock`).
pub const K_SCROLLLOCK: Key = 1073741895;
/// Taste `scroll lock` (SDL nennt sie `ScrollLock`).
pub const K_SCROLLOCK: Key = 1073741895;
/// Taste `;`.
pub const K_SEMICOLON: Key = 59;
/// Taste `/`.
pub const K_SLASH: Key = 47;
/// Taste `space` (SDL nennt sie `Space`).
pub const K_SPACE: Key = 32;
/// Taste `sys req` (SDL nennt sie `SysReq`).
pub const K_SYSREQ: Key = 1073741978;
/// Taste `tab` (SDL nennt sie `Tab`).
pub const K_TAB: Key = 9;
/// Taste `_`.
pub const K_UNDERSCORE: Key = 95;
/// Taste ``.
pub const K_UNKNOWN: Key = 0;
/// Taste `up` (SDL nennt sie `Up`).
pub const K_UP: Key = 1073741906;
/// Taste `a` (SDL nennt sie `A`).
pub const K_a: Key = 97;
/// Taste `b` (SDL nennt sie `B`).
pub const K_b: Key = 98;
/// Taste `c` (SDL nennt sie `C`).
pub const K_c: Key = 99;
/// Taste `d` (SDL nennt sie `D`).
pub const K_d: Key = 100;
/// Taste `e` (SDL nennt sie `E`).
pub const K_e: Key = 101;
/// Taste `f` (SDL nennt sie `F`).
pub const K_f: Key = 102;
/// Taste `g` (SDL nennt sie `G`).
pub const K_g: Key = 103;
/// Taste `h` (SDL nennt sie `H`).
pub const K_h: Key = 104;
/// Taste `i` (SDL nennt sie `I`).
pub const K_i: Key = 105;
/// Taste `j` (SDL nennt sie `J`).
pub const K_j: Key = 106;
/// Taste `k` (SDL nennt sie `K`).
pub const K_k: Key = 107;
/// Taste `l` (SDL nennt sie `L`).
pub const K_l: Key = 108;
/// Taste `m` (SDL nennt sie `M`).
pub const K_m: Key = 109;
/// Taste `n` (SDL nennt sie `N`).
pub const K_n: Key = 110;
/// Taste `o` (SDL nennt sie `O`).
pub const K_o: Key = 111;
/// Taste `p` (SDL nennt sie `P`).
pub const K_p: Key = 112;
/// Taste `q` (SDL nennt sie `Q`).
pub const K_q: Key = 113;
/// Taste `r` (SDL nennt sie `R`).
pub const K_r: Key = 114;
/// Taste `s` (SDL nennt sie `S`).
pub const K_s: Key = 115;
/// Taste `t` (SDL nennt sie `T`).
pub const K_t: Key = 116;
/// Taste `u` (SDL nennt sie `U`).
pub const K_u: Key = 117;
/// Taste `v` (SDL nennt sie `V`).
pub const K_v: Key = 118;
/// Taste `w` (SDL nennt sie `W`).
pub const K_w: Key = 119;
/// Taste `x` (SDL nennt sie `X`).
pub const K_x: Key = 120;
/// Taste `y` (SDL nennt sie `Y`).
pub const K_y: Key = 121;
/// Taste `z` (SDL nennt sie `Z`).
pub const K_z: Key = 122;

// ---------------------------------------------------------------
// Tastenplaetze (KSCAN_*)
// ---------------------------------------------------------------

/// Tastenplatz von [`K_0`] - Platz 39.
pub const KSCAN_0: Scancode = 39;
/// Tastenplatz von [`K_1`] - Platz 30.
pub const KSCAN_1: Scancode = 30;
/// Tastenplatz von [`K_2`] - Platz 31.
pub const KSCAN_2: Scancode = 31;
/// Tastenplatz von [`K_3`] - Platz 32.
pub const KSCAN_3: Scancode = 32;
/// Tastenplatz von [`K_4`] - Platz 33.
pub const KSCAN_4: Scancode = 33;
/// Tastenplatz von [`K_5`] - Platz 34.
pub const KSCAN_5: Scancode = 34;
/// Tastenplatz von [`K_6`] - Platz 35.
pub const KSCAN_6: Scancode = 35;
/// Tastenplatz von [`K_7`] - Platz 36.
pub const KSCAN_7: Scancode = 36;
/// Tastenplatz von [`K_8`] - Platz 37.
pub const KSCAN_8: Scancode = 37;
/// Tastenplatz von [`K_9`] - Platz 38.
pub const KSCAN_9: Scancode = 38;
/// Tastenplatz 4.
pub const KSCAN_A: Scancode = 4;
/// Tastenplatz von [`K_AC_BACK`] - Platz 270.
pub const KSCAN_AC_BACK: Scancode = 270;
/// Tastenplatz 52.
pub const KSCAN_APOSTROPHE: Scancode = 52;
/// Tastenplatz 5.
pub const KSCAN_B: Scancode = 5;
/// Tastenplatz von [`K_BACKSLASH`] - Platz 49.
pub const KSCAN_BACKSLASH: Scancode = 49;
/// Tastenplatz von [`K_BACKSPACE`] - Platz 42.
pub const KSCAN_BACKSPACE: Scancode = 42;
/// Tastenplatz von [`K_BREAK`] - Platz 72.
pub const KSCAN_BREAK: Scancode = 72;
/// Tastenplatz 6.
pub const KSCAN_C: Scancode = 6;
/// Tastenplatz von [`K_CAPSLOCK`] - Platz 57.
pub const KSCAN_CAPSLOCK: Scancode = 57;
/// Tastenplatz von [`K_CLEAR`] - Platz 156.
pub const KSCAN_CLEAR: Scancode = 156;
/// Tastenplatz von [`K_COMMA`] - Platz 54.
pub const KSCAN_COMMA: Scancode = 54;
/// Tastenplatz von [`K_CURRENCYSUBUNIT`] - Platz 181.
pub const KSCAN_CURRENCYSUBUNIT: Scancode = 181;
/// Tastenplatz von [`K_CURRENCYUNIT`] - Platz 180.
pub const KSCAN_CURRENCYUNIT: Scancode = 180;
/// Tastenplatz 7.
pub const KSCAN_D: Scancode = 7;
/// Tastenplatz von [`K_DELETE`] - Platz 76.
pub const KSCAN_DELETE: Scancode = 76;
/// Tastenplatz von [`K_DOWN`] - Platz 81.
pub const KSCAN_DOWN: Scancode = 81;
/// Tastenplatz 8.
pub const KSCAN_E: Scancode = 8;
/// Tastenplatz von [`K_END`] - Platz 77.
pub const KSCAN_END: Scancode = 77;
/// Tastenplatz von [`K_EQUALS`] - Platz 46.
pub const KSCAN_EQUALS: Scancode = 46;
/// Tastenplatz von [`K_ESCAPE`] - Platz 41.
pub const KSCAN_ESCAPE: Scancode = 41;
/// Tastenplatz von [`K_EURO`] - Platz 180.
pub const KSCAN_EURO: Scancode = 180;
/// Tastenplatz 9.
pub const KSCAN_F: Scancode = 9;
/// Tastenplatz von [`K_F1`] - Platz 58.
pub const KSCAN_F1: Scancode = 58;
/// Tastenplatz von [`K_F10`] - Platz 67.
pub const KSCAN_F10: Scancode = 67;
/// Tastenplatz von [`K_F11`] - Platz 68.
pub const KSCAN_F11: Scancode = 68;
/// Tastenplatz von [`K_F12`] - Platz 69.
pub const KSCAN_F12: Scancode = 69;
/// Tastenplatz von [`K_F13`] - Platz 104.
pub const KSCAN_F13: Scancode = 104;
/// Tastenplatz von [`K_F14`] - Platz 105.
pub const KSCAN_F14: Scancode = 105;
/// Tastenplatz von [`K_F15`] - Platz 106.
pub const KSCAN_F15: Scancode = 106;
/// Tastenplatz von [`K_F2`] - Platz 59.
pub const KSCAN_F2: Scancode = 59;
/// Tastenplatz von [`K_F3`] - Platz 60.
pub const KSCAN_F3: Scancode = 60;
/// Tastenplatz von [`K_F4`] - Platz 61.
pub const KSCAN_F4: Scancode = 61;
/// Tastenplatz von [`K_F5`] - Platz 62.
pub const KSCAN_F5: Scancode = 62;
/// Tastenplatz von [`K_F6`] - Platz 63.
pub const KSCAN_F6: Scancode = 63;
/// Tastenplatz von [`K_F7`] - Platz 64.
pub const KSCAN_F7: Scancode = 64;
/// Tastenplatz von [`K_F8`] - Platz 65.
pub const KSCAN_F8: Scancode = 65;
/// Tastenplatz von [`K_F9`] - Platz 66.
pub const KSCAN_F9: Scancode = 66;
/// Tastenplatz 10.
pub const KSCAN_G: Scancode = 10;
/// Tastenplatz 53.
pub const KSCAN_GRAVE: Scancode = 53;
/// Tastenplatz 11.
pub const KSCAN_H: Scancode = 11;
/// Tastenplatz von [`K_HELP`] - Platz 117.
pub const KSCAN_HELP: Scancode = 117;
/// Tastenplatz von [`K_HOME`] - Platz 74.
pub const KSCAN_HOME: Scancode = 74;
/// Tastenplatz 12.
pub const KSCAN_I: Scancode = 12;
/// Tastenplatz von [`K_INSERT`] - Platz 73.
pub const KSCAN_INSERT: Scancode = 73;
/// Tastenplatz 135.
pub const KSCAN_INTERNATIONAL1: Scancode = 135;
/// Tastenplatz 136.
pub const KSCAN_INTERNATIONAL2: Scancode = 136;
/// Tastenplatz 137.
pub const KSCAN_INTERNATIONAL3: Scancode = 137;
/// Tastenplatz 138.
pub const KSCAN_INTERNATIONAL4: Scancode = 138;
/// Tastenplatz 139.
pub const KSCAN_INTERNATIONAL5: Scancode = 139;
/// Tastenplatz 140.
pub const KSCAN_INTERNATIONAL6: Scancode = 140;
/// Tastenplatz 141.
pub const KSCAN_INTERNATIONAL7: Scancode = 141;
/// Tastenplatz 142.
pub const KSCAN_INTERNATIONAL8: Scancode = 142;
/// Tastenplatz 143.
pub const KSCAN_INTERNATIONAL9: Scancode = 143;
/// Tastenplatz 13.
pub const KSCAN_J: Scancode = 13;
/// Tastenplatz 14.
pub const KSCAN_K: Scancode = 14;
/// Tastenplatz von [`K_KP0`] - Platz 98.
pub const KSCAN_KP0: Scancode = 98;
/// Tastenplatz von [`K_KP1`] - Platz 89.
pub const KSCAN_KP1: Scancode = 89;
/// Tastenplatz von [`K_KP2`] - Platz 90.
pub const KSCAN_KP2: Scancode = 90;
/// Tastenplatz von [`K_KP3`] - Platz 91.
pub const KSCAN_KP3: Scancode = 91;
/// Tastenplatz von [`K_KP4`] - Platz 92.
pub const KSCAN_KP4: Scancode = 92;
/// Tastenplatz von [`K_KP5`] - Platz 93.
pub const KSCAN_KP5: Scancode = 93;
/// Tastenplatz von [`K_KP6`] - Platz 94.
pub const KSCAN_KP6: Scancode = 94;
/// Tastenplatz von [`K_KP7`] - Platz 95.
pub const KSCAN_KP7: Scancode = 95;
/// Tastenplatz von [`K_KP8`] - Platz 96.
pub const KSCAN_KP8: Scancode = 96;
/// Tastenplatz von [`K_KP9`] - Platz 97.
pub const KSCAN_KP9: Scancode = 97;
/// Tastenplatz von [`K_KP_0`] - Platz 98.
pub const KSCAN_KP_0: Scancode = 98;
/// Tastenplatz von [`K_KP_1`] - Platz 89.
pub const KSCAN_KP_1: Scancode = 89;
/// Tastenplatz von [`K_KP_2`] - Platz 90.
pub const KSCAN_KP_2: Scancode = 90;
/// Tastenplatz von [`K_KP_3`] - Platz 91.
pub const KSCAN_KP_3: Scancode = 91;
/// Tastenplatz von [`K_KP_4`] - Platz 92.
pub const KSCAN_KP_4: Scancode = 92;
/// Tastenplatz von [`K_KP_5`] - Platz 93.
pub const KSCAN_KP_5: Scancode = 93;
/// Tastenplatz von [`K_KP_6`] - Platz 94.
pub const KSCAN_KP_6: Scancode = 94;
/// Tastenplatz von [`K_KP_7`] - Platz 95.
pub const KSCAN_KP_7: Scancode = 95;
/// Tastenplatz von [`K_KP_8`] - Platz 96.
pub const KSCAN_KP_8: Scancode = 96;
/// Tastenplatz von [`K_KP_9`] - Platz 97.
pub const KSCAN_KP_9: Scancode = 97;
/// Tastenplatz von [`K_KP_DIVIDE`] - Platz 84.
pub const KSCAN_KP_DIVIDE: Scancode = 84;
/// Tastenplatz von [`K_KP_ENTER`] - Platz 88.
pub const KSCAN_KP_ENTER: Scancode = 88;
/// Tastenplatz von [`K_KP_EQUALS`] - Platz 103.
pub const KSCAN_KP_EQUALS: Scancode = 103;
/// Tastenplatz von [`K_KP_MINUS`] - Platz 86.
pub const KSCAN_KP_MINUS: Scancode = 86;
/// Tastenplatz von [`K_KP_MULTIPLY`] - Platz 85.
pub const KSCAN_KP_MULTIPLY: Scancode = 85;
/// Tastenplatz von [`K_KP_PERIOD`] - Platz 99.
pub const KSCAN_KP_PERIOD: Scancode = 99;
/// Tastenplatz von [`K_KP_PLUS`] - Platz 87.
pub const KSCAN_KP_PLUS: Scancode = 87;
/// Tastenplatz 15.
pub const KSCAN_L: Scancode = 15;
/// Tastenplatz von [`K_LALT`] - Platz 226.
pub const KSCAN_LALT: Scancode = 226;
/// Tastenplatz 144.
pub const KSCAN_LANG1: Scancode = 144;
/// Tastenplatz 145.
pub const KSCAN_LANG2: Scancode = 145;
/// Tastenplatz 146.
pub const KSCAN_LANG3: Scancode = 146;
/// Tastenplatz 147.
pub const KSCAN_LANG4: Scancode = 147;
/// Tastenplatz 148.
pub const KSCAN_LANG5: Scancode = 148;
/// Tastenplatz 149.
pub const KSCAN_LANG6: Scancode = 149;
/// Tastenplatz 150.
pub const KSCAN_LANG7: Scancode = 150;
/// Tastenplatz 151.
pub const KSCAN_LANG8: Scancode = 151;
/// Tastenplatz 152.
pub const KSCAN_LANG9: Scancode = 152;
/// Tastenplatz von [`K_LCTRL`] - Platz 224.
pub const KSCAN_LCTRL: Scancode = 224;
/// Tastenplatz von [`K_LEFT`] - Platz 80.
pub const KSCAN_LEFT: Scancode = 80;
/// Tastenplatz von [`K_LEFTBRACKET`] - Platz 47.
pub const KSCAN_LEFTBRACKET: Scancode = 47;
/// Tastenplatz von [`K_LGUI`] - Platz 227.
pub const KSCAN_LGUI: Scancode = 227;
/// Tastenplatz von [`K_LMETA`] - Platz 227.
pub const KSCAN_LMETA: Scancode = 227;
/// Tastenplatz von [`K_LSHIFT`] - Platz 225.
pub const KSCAN_LSHIFT: Scancode = 225;
/// Tastenplatz von [`K_LSUPER`] - Platz 227.
pub const KSCAN_LSUPER: Scancode = 227;
/// Tastenplatz 16.
pub const KSCAN_M: Scancode = 16;
/// Tastenplatz von [`K_MENU`] - Platz 118.
pub const KSCAN_MENU: Scancode = 118;
/// Tastenplatz von [`K_MINUS`] - Platz 45.
pub const KSCAN_MINUS: Scancode = 45;
/// Tastenplatz von [`K_MODE`] - Platz 257.
pub const KSCAN_MODE: Scancode = 257;
/// Tastenplatz 17.
pub const KSCAN_N: Scancode = 17;
/// Tastenplatz 100.
pub const KSCAN_NONUSBACKSLASH: Scancode = 100;
/// Tastenplatz 50.
pub const KSCAN_NONUSHASH: Scancode = 50;
/// Tastenplatz von [`K_NUMLOCK`] - Platz 83.
pub const KSCAN_NUMLOCK: Scancode = 83;
/// Tastenplatz von [`K_NUMLOCKCLEAR`] - Platz 83.
pub const KSCAN_NUMLOCKCLEAR: Scancode = 83;
/// Tastenplatz 18.
pub const KSCAN_O: Scancode = 18;
/// Tastenplatz 19.
pub const KSCAN_P: Scancode = 19;
/// Tastenplatz von [`K_PAGEDOWN`] - Platz 78.
pub const KSCAN_PAGEDOWN: Scancode = 78;
/// Tastenplatz von [`K_PAGEUP`] - Platz 75.
pub const KSCAN_PAGEUP: Scancode = 75;
/// Tastenplatz von [`K_PAUSE`] - Platz 72.
pub const KSCAN_PAUSE: Scancode = 72;
/// Tastenplatz von [`K_PERIOD`] - Platz 55.
pub const KSCAN_PERIOD: Scancode = 55;
/// Tastenplatz von [`K_POWER`] - Platz 102.
pub const KSCAN_POWER: Scancode = 102;
/// Tastenplatz von [`K_PRINT`] - Platz 70.
pub const KSCAN_PRINT: Scancode = 70;
/// Tastenplatz von [`K_PRINTSCREEN`] - Platz 70.
pub const KSCAN_PRINTSCREEN: Scancode = 70;
/// Tastenplatz 20.
pub const KSCAN_Q: Scancode = 20;
/// Tastenplatz 21.
pub const KSCAN_R: Scancode = 21;
/// Tastenplatz von [`K_RALT`] - Platz 230.
pub const KSCAN_RALT: Scancode = 230;
/// Tastenplatz von [`K_RCTRL`] - Platz 228.
pub const KSCAN_RCTRL: Scancode = 228;
/// Tastenplatz von [`K_RETURN`] - Platz 40.
pub const KSCAN_RETURN: Scancode = 40;
/// Tastenplatz von [`K_RGUI`] - Platz 231.
pub const KSCAN_RGUI: Scancode = 231;
/// Tastenplatz von [`K_RIGHT`] - Platz 79.
pub const KSCAN_RIGHT: Scancode = 79;
/// Tastenplatz von [`K_RIGHTBRACKET`] - Platz 48.
pub const KSCAN_RIGHTBRACKET: Scancode = 48;
/// Tastenplatz von [`K_RMETA`] - Platz 231.
pub const KSCAN_RMETA: Scancode = 231;
/// Tastenplatz von [`K_RSHIFT`] - Platz 229.
pub const KSCAN_RSHIFT: Scancode = 229;
/// Tastenplatz von [`K_RSUPER`] - Platz 231.
pub const KSCAN_RSUPER: Scancode = 231;
/// Tastenplatz 22.
pub const KSCAN_S: Scancode = 22;
/// Tastenplatz von [`K_SCROLLLOCK`] - Platz 71.
pub const KSCAN_SCROLLLOCK: Scancode = 71;
/// Tastenplatz von [`K_SCROLLOCK`] - Platz 71.
pub const KSCAN_SCROLLOCK: Scancode = 71;
/// Tastenplatz von [`K_SEMICOLON`] - Platz 51.
pub const KSCAN_SEMICOLON: Scancode = 51;
/// Tastenplatz von [`K_SLASH`] - Platz 56.
pub const KSCAN_SLASH: Scancode = 56;
/// Tastenplatz von [`K_SPACE`] - Platz 44.
pub const KSCAN_SPACE: Scancode = 44;
/// Tastenplatz von [`K_SYSREQ`] - Platz 154.
pub const KSCAN_SYSREQ: Scancode = 154;
/// Tastenplatz 23.
pub const KSCAN_T: Scancode = 23;
/// Tastenplatz von [`K_TAB`] - Platz 43.
pub const KSCAN_TAB: Scancode = 43;
/// Tastenplatz 24.
pub const KSCAN_U: Scancode = 24;
/// Tastenplatz von [`K_UNKNOWN`] - Platz 0.
pub const KSCAN_UNKNOWN: Scancode = 0;
/// Tastenplatz von [`K_UP`] - Platz 82.
pub const KSCAN_UP: Scancode = 82;
/// Tastenplatz 25.
pub const KSCAN_V: Scancode = 25;
/// Tastenplatz 26.
pub const KSCAN_W: Scancode = 26;
/// Tastenplatz 27.
pub const KSCAN_X: Scancode = 27;
/// Tastenplatz 28.
pub const KSCAN_Y: Scancode = 28;
/// Tastenplatz 29.
pub const KSCAN_Z: Scancode = 29;

// ---------------------------------------------------------------
// Zusatztasten (KMOD_*)
// ---------------------------------------------------------------

/// Zusatztaste `KMOD_ALT` - Bitmaske 768.
pub const KMOD_ALT: KeyMod = 768;
/// Zusatztaste `KMOD_CAPS` - Bitmaske 8192.
pub const KMOD_CAPS: KeyMod = 8192;
/// Zusatztaste `KMOD_CTRL` - Bitmaske 192.
pub const KMOD_CTRL: KeyMod = 192;
/// Zusatztaste `KMOD_GUI` - Bitmaske 3072.
pub const KMOD_GUI: KeyMod = 3072;
/// Zusatztaste `KMOD_LALT` - Bitmaske 256.
pub const KMOD_LALT: KeyMod = 256;
/// Zusatztaste `KMOD_LCTRL` - Bitmaske 64.
pub const KMOD_LCTRL: KeyMod = 64;
/// Zusatztaste `KMOD_LGUI` - Bitmaske 1024.
pub const KMOD_LGUI: KeyMod = 1024;
/// Zusatztaste `KMOD_LMETA` - Bitmaske 1024.
pub const KMOD_LMETA: KeyMod = 1024;
/// Zusatztaste `KMOD_LSHIFT` - Bitmaske 1.
pub const KMOD_LSHIFT: KeyMod = 1;
/// Zusatztaste `KMOD_META` - Bitmaske 3072.
pub const KMOD_META: KeyMod = 3072;
/// Zusatztaste `KMOD_MODE` - Bitmaske 16384.
pub const KMOD_MODE: KeyMod = 16384;
/// Zusatztaste `KMOD_NONE` - Bitmaske 0.
pub const KMOD_NONE: KeyMod = 0;
/// Zusatztaste `KMOD_NUM` - Bitmaske 4096.
pub const KMOD_NUM: KeyMod = 4096;
/// Zusatztaste `KMOD_RALT` - Bitmaske 512.
pub const KMOD_RALT: KeyMod = 512;
/// Zusatztaste `KMOD_RCTRL` - Bitmaske 128.
pub const KMOD_RCTRL: KeyMod = 128;
/// Zusatztaste `KMOD_RGUI` - Bitmaske 2048.
pub const KMOD_RGUI: KeyMod = 2048;
/// Zusatztaste `KMOD_RMETA` - Bitmaske 2048.
pub const KMOD_RMETA: KeyMod = 2048;
/// Zusatztaste `KMOD_RSHIFT` - Bitmaske 2.
pub const KMOD_RSHIFT: KeyMod = 2;
/// Zusatztaste `KMOD_SHIFT` - Bitmaske 3.
pub const KMOD_SHIFT: KeyMod = 3;

// ---------------------------------------------------------------
// Maustasten, Fensterposition, Zeigerformen
// ---------------------------------------------------------------

/// `pygame.BUTTON_LEFT`.
pub const BUTTON_LEFT: u32 = 1;
/// `pygame.BUTTON_MIDDLE`.
pub const BUTTON_MIDDLE: u32 = 2;
/// `pygame.BUTTON_RIGHT`.
pub const BUTTON_RIGHT: u32 = 3;
/// `pygame.BUTTON_WHEELUP`.
pub const BUTTON_WHEELUP: u32 = 4;
/// `pygame.BUTTON_WHEELDOWN`.
pub const BUTTON_WHEELDOWN: u32 = 5;
/// `pygame.BUTTON_X1`.
pub const BUTTON_X1: u32 = 6;
/// `pygame.BUTTON_X2`.
pub const BUTTON_X2: u32 = 7;
/// `pygame.WINDOWPOS_CENTERED`.
pub const WINDOWPOS_CENTERED: u32 = 805240832;
/// `pygame.WINDOWPOS_UNDEFINED`.
pub const WINDOWPOS_UNDEFINED: u32 = 536805376;
/// `pygame.APPMOUSEFOCUS`.
pub const APPMOUSEFOCUS: u32 = 1;
/// `pygame.APPINPUTFOCUS`.
pub const APPINPUTFOCUS: u32 = 2;
/// `pygame.APPACTIVE`.
pub const APPACTIVE: u32 = 4;
/// `pygame.SYSTEM_CURSOR_ARROW`.
pub const SYSTEM_CURSOR_ARROW: u32 = 0;
/// `pygame.SYSTEM_CURSOR_CROSSHAIR`.
pub const SYSTEM_CURSOR_CROSSHAIR: u32 = 3;
/// `pygame.SYSTEM_CURSOR_HAND`.
pub const SYSTEM_CURSOR_HAND: u32 = 11;
/// `pygame.SYSTEM_CURSOR_IBEAM`.
pub const SYSTEM_CURSOR_IBEAM: u32 = 1;
/// `pygame.SYSTEM_CURSOR_NO`.
pub const SYSTEM_CURSOR_NO: u32 = 10;
/// `pygame.SYSTEM_CURSOR_SIZEALL`.
pub const SYSTEM_CURSOR_SIZEALL: u32 = 9;
/// `pygame.SYSTEM_CURSOR_SIZENESW`.
pub const SYSTEM_CURSOR_SIZENESW: u32 = 6;
/// `pygame.SYSTEM_CURSOR_SIZENS`.
pub const SYSTEM_CURSOR_SIZENS: u32 = 8;
/// `pygame.SYSTEM_CURSOR_SIZENWSE`.
pub const SYSTEM_CURSOR_SIZENWSE: u32 = 5;
/// `pygame.SYSTEM_CURSOR_SIZEWE`.
pub const SYSTEM_CURSOR_SIZEWE: u32 = 7;
/// `pygame.SYSTEM_CURSOR_WAIT`.
pub const SYSTEM_CURSOR_WAIT: u32 = 2;
/// `pygame.SYSTEM_CURSOR_WAITARROW`.
pub const SYSTEM_CURSOR_WAITARROW: u32 = 4;

// ---------------------------------------------------------------
// OpenGL-Merkmale (GL_*)
// ---------------------------------------------------------------

// **Diese Kiste kann kein OpenGL.** Die Namen stehen hier, weil
// `pygame.locals` sie fuehrt und Programme sie schreiben duerfen -
// `display::gl_set_attribute` nimmt sie entgegen und sagt ehrlich
// nein. Gemessen an echtem pygame unter `SDL_VIDEODRIVER=dummy`:
// `gl_get_attribute(GL_DEPTH_SIZE)` wirft dort
// `No dynamic GL support in current SDL video driver (dummy)`.

/// `pygame.GL_ACCELERATED_VISUAL`.
pub const GL_ACCELERATED_VISUAL: u32 = 15;
/// `pygame.GL_ACCUM_ALPHA_SIZE`.
pub const GL_ACCUM_ALPHA_SIZE: u32 = 11;
/// `pygame.GL_ACCUM_BLUE_SIZE`.
pub const GL_ACCUM_BLUE_SIZE: u32 = 10;
/// `pygame.GL_ACCUM_GREEN_SIZE`.
pub const GL_ACCUM_GREEN_SIZE: u32 = 9;
/// `pygame.GL_ACCUM_RED_SIZE`.
pub const GL_ACCUM_RED_SIZE: u32 = 8;
/// `pygame.GL_ALPHA_SIZE`.
pub const GL_ALPHA_SIZE: u32 = 3;
/// `pygame.GL_BLUE_SIZE`.
pub const GL_BLUE_SIZE: u32 = 2;
/// `pygame.GL_BUFFER_SIZE`.
pub const GL_BUFFER_SIZE: u32 = 4;
/// `pygame.GL_CONTEXT_DEBUG_FLAG`.
pub const GL_CONTEXT_DEBUG_FLAG: u32 = 1;
/// `pygame.GL_CONTEXT_FLAGS`.
pub const GL_CONTEXT_FLAGS: u32 = 20;
/// `pygame.GL_CONTEXT_FORWARD_COMPATIBLE_FLAG`.
pub const GL_CONTEXT_FORWARD_COMPATIBLE_FLAG: u32 = 2;
/// `pygame.GL_CONTEXT_MAJOR_VERSION`.
pub const GL_CONTEXT_MAJOR_VERSION: u32 = 17;
/// `pygame.GL_CONTEXT_MINOR_VERSION`.
pub const GL_CONTEXT_MINOR_VERSION: u32 = 18;
/// `pygame.GL_CONTEXT_PROFILE_COMPATIBILITY`.
pub const GL_CONTEXT_PROFILE_COMPATIBILITY: u32 = 2;
/// `pygame.GL_CONTEXT_PROFILE_CORE`.
pub const GL_CONTEXT_PROFILE_CORE: u32 = 1;
/// `pygame.GL_CONTEXT_PROFILE_ES`.
pub const GL_CONTEXT_PROFILE_ES: u32 = 4;
/// `pygame.GL_CONTEXT_PROFILE_MASK`.
pub const GL_CONTEXT_PROFILE_MASK: u32 = 21;
/// `pygame.GL_CONTEXT_RELEASE_BEHAVIOR`.
pub const GL_CONTEXT_RELEASE_BEHAVIOR: u32 = 24;
/// `pygame.GL_CONTEXT_RELEASE_BEHAVIOR_FLUSH`.
pub const GL_CONTEXT_RELEASE_BEHAVIOR_FLUSH: u32 = 1;
/// `pygame.GL_CONTEXT_RELEASE_BEHAVIOR_NONE`.
pub const GL_CONTEXT_RELEASE_BEHAVIOR_NONE: u32 = 0;
/// `pygame.GL_CONTEXT_RESET_ISOLATION_FLAG`.
pub const GL_CONTEXT_RESET_ISOLATION_FLAG: u32 = 8;
/// `pygame.GL_CONTEXT_ROBUST_ACCESS_FLAG`.
pub const GL_CONTEXT_ROBUST_ACCESS_FLAG: u32 = 4;
/// `pygame.GL_DEPTH_SIZE`.
pub const GL_DEPTH_SIZE: u32 = 6;
/// `pygame.GL_DOUBLEBUFFER`.
pub const GL_DOUBLEBUFFER: u32 = 5;
/// `pygame.GL_FRAMEBUFFER_SRGB_CAPABLE`.
pub const GL_FRAMEBUFFER_SRGB_CAPABLE: u32 = 23;
/// `pygame.GL_GREEN_SIZE`.
pub const GL_GREEN_SIZE: u32 = 1;
/// `pygame.GL_MULTISAMPLEBUFFERS`.
pub const GL_MULTISAMPLEBUFFERS: u32 = 13;
/// `pygame.GL_MULTISAMPLESAMPLES`.
pub const GL_MULTISAMPLESAMPLES: u32 = 14;
/// `pygame.GL_RED_SIZE`.
pub const GL_RED_SIZE: u32 = 0;
/// `pygame.GL_SHARE_WITH_CURRENT_CONTEXT`.
pub const GL_SHARE_WITH_CURRENT_CONTEXT: u32 = 22;
/// `pygame.GL_STENCIL_SIZE`.
pub const GL_STENCIL_SIZE: u32 = 7;
/// `pygame.GL_STEREO`.
pub const GL_STEREO: u32 = 12;
/// `pygame.GL_SWAP_CONTROL`.
pub const GL_SWAP_CONTROL: u32 = 0;

// ---------------------------------------------------------------
// Spielsteuerungen (CONTROLLER_*, HAT_*)
// ---------------------------------------------------------------

/// Als `i32` und nicht als `u32`: `CONTROLLER_AXIS_INVALID` und
/// `CONTROLLER_BUTTON_INVALID` sind in pygame wirklich `-1`, nicht
/// bloss ein gesetztes oberstes Bit wie bei [`FULLSCREEN`]. Auf 32
/// Bit maskiert ergaebe das 4294967295 - einen Achsenindex, den
/// niemand wiedererkennt.
pub type ControllerIndex = i32;

/// `pygame.CONTROLLER_AXIS_INVALID`.
pub const CONTROLLER_AXIS_INVALID: i32 = -1;
/// `pygame.CONTROLLER_AXIS_LEFTX`.
pub const CONTROLLER_AXIS_LEFTX: i32 = 0;
/// `pygame.CONTROLLER_AXIS_LEFTY`.
pub const CONTROLLER_AXIS_LEFTY: i32 = 1;
/// `pygame.CONTROLLER_AXIS_MAX`.
pub const CONTROLLER_AXIS_MAX: i32 = 6;
/// `pygame.CONTROLLER_AXIS_RIGHTX`.
pub const CONTROLLER_AXIS_RIGHTX: i32 = 2;
/// `pygame.CONTROLLER_AXIS_RIGHTY`.
pub const CONTROLLER_AXIS_RIGHTY: i32 = 3;
/// `pygame.CONTROLLER_AXIS_TRIGGERLEFT`.
pub const CONTROLLER_AXIS_TRIGGERLEFT: i32 = 4;
/// `pygame.CONTROLLER_AXIS_TRIGGERRIGHT`.
pub const CONTROLLER_AXIS_TRIGGERRIGHT: i32 = 5;
/// `pygame.CONTROLLER_BUTTON_A`.
pub const CONTROLLER_BUTTON_A: i32 = 0;
/// `pygame.CONTROLLER_BUTTON_B`.
pub const CONTROLLER_BUTTON_B: i32 = 1;
/// `pygame.CONTROLLER_BUTTON_BACK`.
pub const CONTROLLER_BUTTON_BACK: i32 = 4;
/// `pygame.CONTROLLER_BUTTON_DPAD_DOWN`.
pub const CONTROLLER_BUTTON_DPAD_DOWN: i32 = 12;
/// `pygame.CONTROLLER_BUTTON_DPAD_LEFT`.
pub const CONTROLLER_BUTTON_DPAD_LEFT: i32 = 13;
/// `pygame.CONTROLLER_BUTTON_DPAD_RIGHT`.
pub const CONTROLLER_BUTTON_DPAD_RIGHT: i32 = 14;
/// `pygame.CONTROLLER_BUTTON_DPAD_UP`.
pub const CONTROLLER_BUTTON_DPAD_UP: i32 = 11;
/// `pygame.CONTROLLER_BUTTON_GUIDE`.
pub const CONTROLLER_BUTTON_GUIDE: i32 = 5;
/// `pygame.CONTROLLER_BUTTON_INVALID`.
pub const CONTROLLER_BUTTON_INVALID: i32 = -1;
/// `pygame.CONTROLLER_BUTTON_LEFTSHOULDER`.
pub const CONTROLLER_BUTTON_LEFTSHOULDER: i32 = 9;
/// `pygame.CONTROLLER_BUTTON_LEFTSTICK`.
pub const CONTROLLER_BUTTON_LEFTSTICK: i32 = 7;
/// `pygame.CONTROLLER_BUTTON_MAX`.
pub const CONTROLLER_BUTTON_MAX: i32 = 21;
/// `pygame.CONTROLLER_BUTTON_RIGHTSHOULDER`.
pub const CONTROLLER_BUTTON_RIGHTSHOULDER: i32 = 10;
/// `pygame.CONTROLLER_BUTTON_RIGHTSTICK`.
pub const CONTROLLER_BUTTON_RIGHTSTICK: i32 = 8;
/// `pygame.CONTROLLER_BUTTON_START`.
pub const CONTROLLER_BUTTON_START: i32 = 6;
/// `pygame.CONTROLLER_BUTTON_X`.
pub const CONTROLLER_BUTTON_X: i32 = 2;
/// `pygame.CONTROLLER_BUTTON_Y`.
pub const CONTROLLER_BUTTON_Y: i32 = 3;

/// `pygame.HAT_CENTERED`.
pub const HAT_CENTERED: u32 = 0;
/// `pygame.HAT_DOWN`.
pub const HAT_DOWN: u32 = 4;
/// `pygame.HAT_LEFT`.
pub const HAT_LEFT: u32 = 8;
/// `pygame.HAT_LEFTDOWN`.
pub const HAT_LEFTDOWN: u32 = 12;
/// `pygame.HAT_LEFTUP`.
pub const HAT_LEFTUP: u32 = 9;
/// `pygame.HAT_RIGHT`.
pub const HAT_RIGHT: u32 = 2;
/// `pygame.HAT_RIGHTDOWN`.
pub const HAT_RIGHTDOWN: u32 = 6;
/// `pygame.HAT_RIGHTUP`.
pub const HAT_RIGHTUP: u32 = 3;
/// `pygame.HAT_UP`.
pub const HAT_UP: u32 = 1;

// ---------------------------------------------------------------
// Tonformate (AUDIO_*)
// ---------------------------------------------------------------

/// Diese Werte gehoeren zu `pygame.mixer`, nicht hierher - aber
/// `pygame.locals` fuehrt sie, und `rustgame-py` reicht die ganze
/// Datei als `pygame.*` weiter. Wer sie auswertet, ist
/// `rustgame-mixer`.
pub type AudioFormat = u32;

/// `pygame.AUDIO_ALLOW_ANY_CHANGE`.
pub const AUDIO_ALLOW_ANY_CHANGE: AudioFormat = 15;
/// `pygame.AUDIO_ALLOW_CHANNELS_CHANGE`.
pub const AUDIO_ALLOW_CHANNELS_CHANGE: AudioFormat = 4;
/// `pygame.AUDIO_ALLOW_FORMAT_CHANGE`.
pub const AUDIO_ALLOW_FORMAT_CHANGE: AudioFormat = 2;
/// `pygame.AUDIO_ALLOW_FREQUENCY_CHANGE`.
pub const AUDIO_ALLOW_FREQUENCY_CHANGE: AudioFormat = 1;
/// `pygame.AUDIO_S16`.
pub const AUDIO_S16: AudioFormat = 32784;
/// `pygame.AUDIO_S16LSB`.
pub const AUDIO_S16LSB: AudioFormat = 32784;
/// `pygame.AUDIO_S16MSB`.
pub const AUDIO_S16MSB: AudioFormat = 36880;
/// `pygame.AUDIO_S16SYS`.
pub const AUDIO_S16SYS: AudioFormat = 32784;
/// `pygame.AUDIO_S8`.
pub const AUDIO_S8: AudioFormat = 32776;
/// `pygame.AUDIO_U16`.
pub const AUDIO_U16: AudioFormat = 16;
/// `pygame.AUDIO_U16LSB`.
pub const AUDIO_U16LSB: AudioFormat = 16;
/// `pygame.AUDIO_U16MSB`.
pub const AUDIO_U16MSB: AudioFormat = 4112;
/// `pygame.AUDIO_U16SYS`.
pub const AUDIO_U16SYS: AudioFormat = 16;
/// `pygame.AUDIO_U8`.
pub const AUDIO_U8: AudioFormat = 8;

// ---------------------------------------------------------------
// Mischen (BLEND_*, BLENDMODE_*, BLENDFACTOR_*, BLENDOPERATION_*)
// ---------------------------------------------------------------

/// `BLEND_*` sind die Werte, mit denen `Surface::blit` und
/// `Surface::fill` rechnen - dieselben Zahlen stehen in
/// `rustgame_core::blend::flags`, dort aber von Hand gepflegt und
/// von einem Test gegen genau diese Tabelle geprueft. Die
/// `BLENDMODE_*`/`BLENDFACTOR_*`/`BLENDOPERATION_*` gehoeren zu
/// SDLs Renderer, den es hier nicht gibt.
pub type BlendMode = u32;

/// `pygame.BLEND_ADD`.
pub const BLEND_ADD: BlendMode = 1;
/// `pygame.BLEND_ALPHA_SDL2`.
pub const BLEND_ALPHA_SDL2: BlendMode = 18;
/// `pygame.BLEND_MAX`.
pub const BLEND_MAX: BlendMode = 5;
/// `pygame.BLEND_MIN`.
pub const BLEND_MIN: BlendMode = 4;
/// `pygame.BLEND_MULT`.
pub const BLEND_MULT: BlendMode = 3;
/// `pygame.BLEND_PREMULTIPLIED`.
pub const BLEND_PREMULTIPLIED: BlendMode = 17;
/// `pygame.BLEND_RGBA_ADD`.
pub const BLEND_RGBA_ADD: BlendMode = 6;
/// `pygame.BLEND_RGBA_MAX`.
pub const BLEND_RGBA_MAX: BlendMode = 16;
/// `pygame.BLEND_RGBA_MIN`.
pub const BLEND_RGBA_MIN: BlendMode = 9;
/// `pygame.BLEND_RGBA_MULT`.
pub const BLEND_RGBA_MULT: BlendMode = 8;
/// `pygame.BLEND_RGBA_SUB`.
pub const BLEND_RGBA_SUB: BlendMode = 7;
/// `pygame.BLEND_RGB_ADD`.
pub const BLEND_RGB_ADD: BlendMode = 1;
/// `pygame.BLEND_RGB_MAX`.
pub const BLEND_RGB_MAX: BlendMode = 5;
/// `pygame.BLEND_RGB_MIN`.
pub const BLEND_RGB_MIN: BlendMode = 4;
/// `pygame.BLEND_RGB_MULT`.
pub const BLEND_RGB_MULT: BlendMode = 3;
/// `pygame.BLEND_RGB_SUB`.
pub const BLEND_RGB_SUB: BlendMode = 2;
/// `pygame.BLEND_SUB`.
pub const BLEND_SUB: BlendMode = 2;
/// `pygame.BLENDMODE_ADD`.
pub const BLENDMODE_ADD: u32 = 2;
/// `pygame.BLENDMODE_BLEND`.
pub const BLENDMODE_BLEND: u32 = 1;
/// `pygame.BLENDMODE_MOD`.
pub const BLENDMODE_MOD: u32 = 4;
/// `pygame.BLENDMODE_MUL`.
pub const BLENDMODE_MUL: u32 = 8;
/// `pygame.BLENDMODE_NONE`.
pub const BLENDMODE_NONE: u32 = 0;
/// `pygame.BLENDFACTOR_DST_ALPHA`.
pub const BLENDFACTOR_DST_ALPHA: u32 = 9;
/// `pygame.BLENDFACTOR_DST_COLOR`.
pub const BLENDFACTOR_DST_COLOR: u32 = 7;
/// `pygame.BLENDFACTOR_ONE`.
pub const BLENDFACTOR_ONE: u32 = 2;
/// `pygame.BLENDFACTOR_ONE_MINUS_DST_ALPHA`.
pub const BLENDFACTOR_ONE_MINUS_DST_ALPHA: u32 = 10;
/// `pygame.BLENDFACTOR_ONE_MINUS_DST_COLOR`.
pub const BLENDFACTOR_ONE_MINUS_DST_COLOR: u32 = 8;
/// `pygame.BLENDFACTOR_ONE_MINUS_SRC_ALPHA`.
pub const BLENDFACTOR_ONE_MINUS_SRC_ALPHA: u32 = 6;
/// `pygame.BLENDFACTOR_ONE_MINUS_SRC_COLOR`.
pub const BLENDFACTOR_ONE_MINUS_SRC_COLOR: u32 = 4;
/// `pygame.BLENDFACTOR_SRC_ALPHA`.
pub const BLENDFACTOR_SRC_ALPHA: u32 = 5;
/// `pygame.BLENDFACTOR_SRC_COLOR`.
pub const BLENDFACTOR_SRC_COLOR: u32 = 3;
/// `pygame.BLENDFACTOR_ZERO`.
pub const BLENDFACTOR_ZERO: u32 = 1;
/// `pygame.BLENDOPERATION_ADD`.
pub const BLENDOPERATION_ADD: u32 = 1;
/// `pygame.BLENDOPERATION_MAXIMUM`.
pub const BLENDOPERATION_MAXIMUM: u32 = 5;
/// `pygame.BLENDOPERATION_MINIMUM`.
pub const BLENDOPERATION_MINIMUM: u32 = 4;
/// `pygame.BLENDOPERATION_REV_SUBTRACT`.
pub const BLENDOPERATION_REV_SUBTRACT: u32 = 3;
/// `pygame.BLENDOPERATION_SUBTRACT`.
pub const BLENDOPERATION_SUBTRACT: u32 = 2;

// ---------------------------------------------------------------
// Schrift, Bilder, Zwischenablage, Endianness
// ---------------------------------------------------------------

/// `pygame.DIRECTION_BTT`.
pub const DIRECTION_BTT: u32 = 3;
/// `pygame.DIRECTION_LTR`.
pub const DIRECTION_LTR: u32 = 0;
/// `pygame.DIRECTION_RTL`.
pub const DIRECTION_RTL: u32 = 1;
/// `pygame.DIRECTION_TTB`.
pub const DIRECTION_TTB: u32 = 2;
/// `pygame.FLASH_BRIEFLY`.
pub const FLASH_BRIEFLY: u32 = 1;
/// `pygame.FLASH_CANCEL`.
pub const FLASH_CANCEL: u32 = 0;
/// `pygame.FLASH_UNTIL_FOCUSED`.
pub const FLASH_UNTIL_FOCUSED: u32 = 2;
/// `pygame.FLIP_HORIZONTAL`.
pub const FLIP_HORIZONTAL: u32 = 1;
/// `pygame.FLIP_NONE`.
pub const FLIP_NONE: u32 = 0;
/// `pygame.FLIP_VERTICAL`.
pub const FLIP_VERTICAL: u32 = 2;
/// `pygame.FONT_CENTER`.
pub const FONT_CENTER: u32 = 1;
/// `pygame.FONT_LEFT`.
pub const FONT_LEFT: u32 = 0;
/// `pygame.FONT_RIGHT`.
pub const FONT_RIGHT: u32 = 2;
/// `pygame.TEXTUREACCESS_STATIC`.
pub const TEXTUREACCESS_STATIC: u32 = 0;
/// `pygame.TEXTUREACCESS_STREAMING`.
pub const TEXTUREACCESS_STREAMING: u32 = 1;
/// `pygame.TEXTUREACCESS_TARGET`.
pub const TEXTUREACCESS_TARGET: u32 = 2;
/// `pygame.SCRAP_CLIPBOARD`.
pub const SCRAP_CLIPBOARD: u32 = 0;
/// `pygame.SCRAP_SELECTION`.
pub const SCRAP_SELECTION: u32 = 1;
/// `pygame.SCROLL_ERASE`.
pub const SCROLL_ERASE: u32 = 4;
/// `pygame.SCROLL_REPEAT`.
pub const SCROLL_REPEAT: u32 = 1;
/// `pygame.BIG_ENDIAN`.
pub const BIG_ENDIAN: u32 = 4321;
/// `pygame.LIL_ENDIAN`.
pub const LIL_ENDIAN: u32 = 1234;
/// `pygame.HWACCEL`.
pub const HWACCEL: u32 = 256;
/// `pygame.IS_CE`.
pub const IS_CE: u32 = 1;
/// `pygame.TIMER_RESOLUTION`.
pub const TIMER_RESOLUTION: u32 = 0;


// Die vier MIME-Typen der Zwischenablage sind in pygame
// Zeichenketten, keine Zahlen - gemessen: `SCRAP_TEXT` ist
// `"text/plain"`.
/// `pygame.SCRAP_BMP`.
pub const SCRAP_BMP: &str = "image/bmp";
/// `pygame.SCRAP_PBM`.
pub const SCRAP_PBM: &str = "image/pbm";
/// `pygame.SCRAP_PPM`.
pub const SCRAP_PPM: &str = "image/ppm";
/// `pygame.SCRAP_TEXT`.
pub const SCRAP_TEXT: &str = "text/plain";

// Der Treibername, unter dem pygame ohne Fenster laeuft - und
// damit genau der Fall, fuer den es hier
// [`crate::backend::DummyBackend`] gibt.
/// `pygame.NULL_VIDEODRIVER`.
pub const NULL_VIDEODRIVER: &str = "dummy";

// ---------------------------------------------------------------
// Gemessene Groessen
// ---------------------------------------------------------------

/// Laenge der Tabelle aus `key::get_pressed()`.
///
/// Gemessen an echtem pygame: `len(pygame.key.get_pressed())` ist
/// 512, und der Index ist der **Tastenplatz**, nicht der Tastencode.
pub const NUM_SCANCODES: usize = 512;

/// Bit, das einen Tastencode von seinem Tastenplatz trennt.
///
/// SDL bildet Tasten ohne Zeichen (Pfeile, F-Tasten, ...) als
/// `platz | SCANCODE_MASK` ab; gemessen: `K_UP` ist 1073741906,
/// `KSCAN_UP` ist 82, und 1073741824 ist genau `1 << 30`.
pub const SCANCODE_MASK: u32 = 1 << 30;

/// Tastencode -> Tastenplatz, nach Tastencode sortiert.
///
/// `key::get_pressed()` wird mit dem Tastenplatz indiziert, die
/// Spiele fragen aber mit `K_*`. Diese Tabelle ueberbrueckt das;
/// sie stammt aus den `KSCAN_*`-Gegenstuecken in `pygame.locals`.
pub static KEY_TO_SCANCODE: &[(Key, Scancode)] = &[
    (0, 0), // K_UNKNOWN
    (8, 42), // K_BACKSPACE
    (9, 43), // K_TAB
    (13, 40), // K_RETURN
    (27, 41), // K_ESCAPE
    (32, 44), // K_SPACE
    (44, 54), // K_COMMA
    (45, 45), // K_MINUS
    (46, 55), // K_PERIOD
    (47, 56), // K_SLASH
    (48, 39), // K_0
    (49, 30), // K_1
    (50, 31), // K_2
    (51, 32), // K_3
    (52, 33), // K_4
    (53, 34), // K_5
    (54, 35), // K_6
    (55, 36), // K_7
    (56, 37), // K_8
    (57, 38), // K_9
    (59, 51), // K_SEMICOLON
    (61, 46), // K_EQUALS
    (91, 47), // K_LEFTBRACKET
    (92, 49), // K_BACKSLASH
    (93, 48), // K_RIGHTBRACKET
    (97, 4), // K_a
    (98, 5), // K_b
    (99, 6), // K_c
    (100, 7), // K_d
    (101, 8), // K_e
    (102, 9), // K_f
    (103, 10), // K_g
    (104, 11), // K_h
    (105, 12), // K_i
    (106, 13), // K_j
    (107, 14), // K_k
    (108, 15), // K_l
    (109, 16), // K_m
    (110, 17), // K_n
    (111, 18), // K_o
    (112, 19), // K_p
    (113, 20), // K_q
    (114, 21), // K_r
    (115, 22), // K_s
    (116, 23), // K_t
    (117, 24), // K_u
    (118, 25), // K_v
    (119, 26), // K_w
    (120, 27), // K_x
    (121, 28), // K_y
    (122, 29), // K_z
    (127, 76), // K_DELETE
    (1073741881, 57), // K_CAPSLOCK
    (1073741882, 58), // K_F1
    (1073741883, 59), // K_F2
    (1073741884, 60), // K_F3
    (1073741885, 61), // K_F4
    (1073741886, 62), // K_F5
    (1073741887, 63), // K_F6
    (1073741888, 64), // K_F7
    (1073741889, 65), // K_F8
    (1073741890, 66), // K_F9
    (1073741891, 67), // K_F10
    (1073741892, 68), // K_F11
    (1073741893, 69), // K_F12
    (1073741894, 70), // K_PRINT
    (1073741895, 71), // K_SCROLLLOCK
    (1073741896, 72), // K_BREAK
    (1073741897, 73), // K_INSERT
    (1073741898, 74), // K_HOME
    (1073741899, 75), // K_PAGEUP
    (1073741901, 77), // K_END
    (1073741902, 78), // K_PAGEDOWN
    (1073741903, 79), // K_RIGHT
    (1073741904, 80), // K_LEFT
    (1073741905, 81), // K_DOWN
    (1073741906, 82), // K_UP
    (1073741907, 83), // K_NUMLOCK
    (1073741908, 84), // K_KP_DIVIDE
    (1073741909, 85), // K_KP_MULTIPLY
    (1073741910, 86), // K_KP_MINUS
    (1073741911, 87), // K_KP_PLUS
    (1073741912, 88), // K_KP_ENTER
    (1073741913, 89), // K_KP1
    (1073741914, 90), // K_KP2
    (1073741915, 91), // K_KP3
    (1073741916, 92), // K_KP4
    (1073741917, 93), // K_KP5
    (1073741918, 94), // K_KP6
    (1073741919, 95), // K_KP7
    (1073741920, 96), // K_KP8
    (1073741921, 97), // K_KP9
    (1073741922, 98), // K_KP0
    (1073741923, 99), // K_KP_PERIOD
    (1073741926, 102), // K_POWER
    (1073741927, 103), // K_KP_EQUALS
    (1073741928, 104), // K_F13
    (1073741929, 105), // K_F14
    (1073741930, 106), // K_F15
    (1073741941, 117), // K_HELP
    (1073741942, 118), // K_MENU
    (1073741978, 154), // K_SYSREQ
    (1073741980, 156), // K_CLEAR
    (1073742004, 180), // K_CURRENCYUNIT
    (1073742005, 181), // K_CURRENCYSUBUNIT
    (1073742048, 224), // K_LCTRL
    (1073742049, 225), // K_LSHIFT
    (1073742050, 226), // K_LALT
    (1073742051, 227), // K_LGUI
    (1073742052, 228), // K_RCTRL
    (1073742053, 229), // K_RSHIFT
    (1073742054, 230), // K_RALT
    (1073742055, 231), // K_RGUI
    (1073742081, 257), // K_MODE
    (1073742094, 270), // K_AC_BACK
];

/// Tastencode -> Name, wie `pygame.key.name()` ihn liefert.
///
/// Zwei Spalten, weil pygame zwei Schreibweisen kennt: die eigene
/// (`name(k)`, kleingeschrieben - `"a"`, `"left shift"`) und die
/// von SDL (`name(k, use_compat=False)` - `"A"`, `"Left Shift"`).
/// Beide sind hier gemessen, nicht geraten.
pub static KEY_NAMES: &[(Key, &str, &str)] = &[
    (0, "", ""),
    (8, "backspace", "Backspace"),
    (9, "tab", "Tab"),
    (13, "return", "Return"),
    (27, "escape", "Escape"),
    (32, "space", "Space"),
    (33, "!", "!"),
    (34, "\"", "\""),
    (35, "#", "#"),
    (36, "$", "$"),
    (37, "%", "%"),
    (38, "&", "&"),
    (39, "'", "'"),
    (40, "(", "("),
    (41, ")", ")"),
    (42, "*", "*"),
    (43, "+", "+"),
    (44, ",", ","),
    (45, "-", "-"),
    (46, ".", "."),
    (47, "/", "/"),
    (48, "0", "0"),
    (49, "1", "1"),
    (50, "2", "2"),
    (51, "3", "3"),
    (52, "4", "4"),
    (53, "5", "5"),
    (54, "6", "6"),
    (55, "7", "7"),
    (56, "8", "8"),
    (57, "9", "9"),
    (58, ":", ":"),
    (59, ";", ";"),
    (60, "<", "<"),
    (61, "=", "="),
    (62, ">", ">"),
    (63, "?", "?"),
    (64, "@", "@"),
    (91, "[", "["),
    (92, "\\", "\\"),
    (93, "]", "]"),
    (94, "^", "^"),
    (95, "_", "_"),
    (96, "`", "`"),
    (97, "a", "A"),
    (98, "b", "B"),
    (99, "c", "C"),
    (100, "d", "D"),
    (101, "e", "E"),
    (102, "f", "F"),
    (103, "g", "G"),
    (104, "h", "H"),
    (105, "i", "I"),
    (106, "j", "J"),
    (107, "k", "K"),
    (108, "l", "L"),
    (109, "m", "M"),
    (110, "n", "N"),
    (111, "o", "O"),
    (112, "p", "P"),
    (113, "q", "Q"),
    (114, "r", "R"),
    (115, "s", "S"),
    (116, "t", "T"),
    (117, "u", "U"),
    (118, "v", "V"),
    (119, "w", "W"),
    (120, "x", "X"),
    (121, "y", "Y"),
    (122, "z", "Z"),
    (127, "delete", "Delete"),
    (1073741881, "caps lock", "CapsLock"),
    (1073741882, "f1", "F1"),
    (1073741883, "f2", "F2"),
    (1073741884, "f3", "F3"),
    (1073741885, "f4", "F4"),
    (1073741886, "f5", "F5"),
    (1073741887, "f6", "F6"),
    (1073741888, "f7", "F7"),
    (1073741889, "f8", "F8"),
    (1073741890, "f9", "F9"),
    (1073741891, "f10", "F10"),
    (1073741892, "f11", "F11"),
    (1073741893, "f12", "F12"),
    (1073741894, "print screen", "PrintScreen"),
    (1073741895, "scroll lock", "ScrollLock"),
    (1073741896, "break", "Pause"),
    (1073741897, "insert", "Insert"),
    (1073741898, "home", "Home"),
    (1073741899, "page up", "PageUp"),
    (1073741901, "end", "End"),
    (1073741902, "page down", "PageDown"),
    (1073741903, "right", "Right"),
    (1073741904, "left", "Left"),
    (1073741905, "down", "Down"),
    (1073741906, "up", "Up"),
    (1073741907, "numlock", "Numlock"),
    (1073741908, "[/]", "Keypad /"),
    (1073741909, "[*]", "Keypad *"),
    (1073741910, "[-]", "Keypad -"),
    (1073741911, "[+]", "Keypad +"),
    (1073741912, "enter", "Keypad Enter"),
    (1073741913, "[1]", "Keypad 1"),
    (1073741914, "[2]", "Keypad 2"),
    (1073741915, "[3]", "Keypad 3"),
    (1073741916, "[4]", "Keypad 4"),
    (1073741917, "[5]", "Keypad 5"),
    (1073741918, "[6]", "Keypad 6"),
    (1073741919, "[7]", "Keypad 7"),
    (1073741920, "[8]", "Keypad 8"),
    (1073741921, "[9]", "Keypad 9"),
    (1073741922, "[0]", "Keypad 0"),
    (1073741923, "[.]", "Keypad ."),
    (1073741926, "power", "Power"),
    (1073741927, "equals", "Keypad ="),
    (1073741928, "f13", "F13"),
    (1073741929, "f14", "F14"),
    (1073741930, "f15", "F15"),
    (1073741941, "help", "Help"),
    (1073741942, "menu", "Menu"),
    (1073741978, "sys req", "SysReq"),
    (1073741980, "clear", "Clear"),
    (1073742004, "euro", "CurrencyUnit"),
    (1073742005, "CurrencySubUnit", "CurrencySubUnit"),
    (1073742048, "left ctrl", "Left Ctrl"),
    (1073742049, "left shift", "Left Shift"),
    (1073742050, "left alt", "Left Alt"),
    (1073742051, "left meta", "Left GUI"),
    (1073742052, "right ctrl", "Right Ctrl"),
    (1073742053, "right shift", "Right Shift"),
    (1073742054, "right alt", "Right Alt"),
    (1073742055, "right meta", "Right GUI"),
    (1073742081, "alt gr", "ModeSwitch"),
    (1073742094, "AC Back", "AC Back"),
];

/// Name -> Tastencode, die Umkehrung von `KEY_NAMES`.
///
/// Enthaelt jede Schreibweise, die echtes pygame aufloest - das
/// sind mehr als nur die Namen aus `KEY_NAMES`, weil
/// `key_code()` bei Mehrzeichen-Namen die Gross-/Kleinschreibung
/// nicht beachtet (`"Up"` und `"up"` ergeben beide 1073741906),
/// bei Einzelzeichen dagegen schon.
/// Nach Name sortiert, damit binaer gesucht werden kann.
pub static KEY_CODES: &[(&str, Key)] = &[
    ("!", 33),
    ("\"", 34),
    ("#", 35),
    ("$", 36),
    ("%", 37),
    ("&", 38),
    ("'", 39),
    ("(", 40),
    (")", 41),
    ("*", 42),
    ("+", 43),
    (",", 44),
    ("-", 45),
    (".", 46),
    ("/", 47),
    ("0", 48),
    ("1", 49),
    ("2", 50),
    ("3", 51),
    ("4", 52),
    ("5", 53),
    ("6", 54),
    ("7", 55),
    ("8", 56),
    ("9", 57),
    (":", 58),
    (";", 59),
    ("<", 60),
    ("=", 61),
    (">", 62),
    ("?", 63),
    ("@", 64),
    ("A", 97),
    ("AC Back", 1073742094),
    ("B", 98),
    ("Backspace", 8),
    ("C", 99),
    ("CapsLock", 1073741881),
    ("Clear", 1073741980),
    ("CurrencySubUnit", 1073742005),
    ("CurrencyUnit", 1073742004),
    ("D", 100),
    ("Delete", 127),
    ("Down", 1073741905),
    ("E", 101),
    ("End", 1073741901),
    ("Escape", 27),
    ("F", 102),
    ("F1", 1073741882),
    ("F10", 1073741891),
    ("F11", 1073741892),
    ("F12", 1073741893),
    ("F13", 1073741928),
    ("F14", 1073741929),
    ("F15", 1073741930),
    ("F2", 1073741883),
    ("F3", 1073741884),
    ("F4", 1073741885),
    ("F5", 1073741886),
    ("F6", 1073741887),
    ("F7", 1073741888),
    ("F8", 1073741889),
    ("F9", 1073741890),
    ("G", 103),
    ("H", 104),
    ("Help", 1073741941),
    ("Home", 1073741898),
    ("I", 105),
    ("Insert", 1073741897),
    ("J", 106),
    ("K", 107),
    ("Keypad *", 1073741909),
    ("Keypad +", 1073741911),
    ("Keypad -", 1073741910),
    ("Keypad .", 1073741923),
    ("Keypad /", 1073741908),
    ("Keypad 0", 1073741922),
    ("Keypad 1", 1073741913),
    ("Keypad 2", 1073741914),
    ("Keypad 3", 1073741915),
    ("Keypad 4", 1073741916),
    ("Keypad 5", 1073741917),
    ("Keypad 6", 1073741918),
    ("Keypad 7", 1073741919),
    ("Keypad 8", 1073741920),
    ("Keypad 9", 1073741921),
    ("Keypad =", 1073741927),
    ("Keypad Enter", 1073741912),
    ("L", 108),
    ("Left", 1073741904),
    ("Left Alt", 1073742050),
    ("Left Ctrl", 1073742048),
    ("Left GUI", 1073742051),
    ("Left Shift", 1073742049),
    ("M", 109),
    ("Menu", 1073741942),
    ("ModeSwitch", 1073742081),
    ("N", 110),
    ("Numlock", 1073741907),
    ("O", 111),
    ("P", 112),
    ("PageDown", 1073741902),
    ("PageUp", 1073741899),
    ("Pause", 1073741896),
    ("Power", 1073741926),
    ("PrintScreen", 1073741894),
    ("Q", 113),
    ("R", 114),
    ("Return", 13),
    ("Right", 1073741903),
    ("Right Alt", 1073742054),
    ("Right Ctrl", 1073742052),
    ("Right GUI", 1073742055),
    ("Right Shift", 1073742053),
    ("S", 115),
    ("ScrollLock", 1073741895),
    ("Space", 32),
    ("SysReq", 1073741978),
    ("T", 116),
    ("Tab", 9),
    ("U", 117),
    ("Up", 1073741906),
    ("V", 118),
    ("W", 119),
    ("X", 120),
    ("Y", 121),
    ("Z", 122),
    ("[", 91),
    ("[*]", 1073741909),
    ("[+]", 1073741911),
    ("[-]", 1073741910),
    ("[.]", 1073741923),
    ("[/]", 1073741908),
    ("[0]", 1073741922),
    ("[1]", 1073741913),
    ("[2]", 1073741914),
    ("[3]", 1073741915),
    ("[4]", 1073741916),
    ("[5]", 1073741917),
    ("[6]", 1073741918),
    ("[7]", 1073741919),
    ("[8]", 1073741920),
    ("[9]", 1073741921),
    ("\\", 92),
    ("]", 93),
    ("^", 94),
    ("_", 95),
    ("`", 96),
    ("a", 97),
    ("ac back", 1073742094),
    ("alt gr", 1073742081),
    ("b", 98),
    ("backspace", 8),
    ("break", 1073741896),
    ("c", 99),
    ("caps lock", 1073741881),
    ("capslock", 1073741881),
    ("clear", 1073741980),
    ("currencysubunit", 1073742005),
    ("currencyunit", 1073742004),
    ("d", 100),
    ("delete", 127),
    ("down", 1073741905),
    ("e", 101),
    ("end", 1073741901),
    ("enter", 1073741912),
    ("equals", 1073741927),
    ("escape", 27),
    ("euro", 1073742004),
    ("f", 102),
    ("f1", 1073741882),
    ("f10", 1073741891),
    ("f11", 1073741892),
    ("f12", 1073741893),
    ("f13", 1073741928),
    ("f14", 1073741929),
    ("f15", 1073741930),
    ("f2", 1073741883),
    ("f3", 1073741884),
    ("f4", 1073741885),
    ("f5", 1073741886),
    ("f6", 1073741887),
    ("f7", 1073741888),
    ("f8", 1073741889),
    ("f9", 1073741890),
    ("g", 103),
    ("h", 104),
    ("help", 1073741941),
    ("home", 1073741898),
    ("i", 105),
    ("insert", 1073741897),
    ("j", 106),
    ("k", 107),
    ("l", 108),
    ("left", 1073741904),
    ("left alt", 1073742050),
    ("left ctrl", 1073742048),
    ("left meta", 1073742051),
    ("left shift", 1073742049),
    ("m", 109),
    ("menu", 1073741942),
    ("n", 110),
    ("numlock", 1073741907),
    ("o", 111),
    ("p", 112),
    ("page down", 1073741902),
    ("page up", 1073741899),
    ("pagedown", 1073741902),
    ("pageup", 1073741899),
    ("pause", 1073741896),
    ("power", 1073741926),
    ("print screen", 1073741894),
    ("printscreen", 1073741894),
    ("q", 113),
    ("r", 114),
    ("return", 13),
    ("right", 1073741903),
    ("right alt", 1073742054),
    ("right ctrl", 1073742052),
    ("right meta", 1073742055),
    ("right shift", 1073742053),
    ("s", 115),
    ("scroll lock", 1073741895),
    ("scrolllock", 1073741895),
    ("space", 32),
    ("sys req", 1073741978),
    ("sysreq", 1073741978),
    ("t", 116),
    ("tab", 9),
    ("u", 117),
    ("up", 1073741906),
    ("v", 118),
    ("w", 119),
    ("x", 120),
    ("y", 121),
    ("z", 122),
];

/// Tkinter-`keysym` -> Tastencode.
///
/// Das umgebende Projekt (pygamez) speichert Tastenbelegungen in
/// seiner Einstellungsdatei als Tkinter-Namen - `"Up"`, `"space"`,
/// `"Return"`, `"w"` - weil seine Oberflaeche auf Tkinter sitzt.
/// Ohne diese Tabelle waere genau dort die Bruchstelle, wenn
/// rustgame die gespeicherten Belegungen liest.
///
/// Die meisten Namen loest `key_code()` von selbst auf; die
/// Ausnahmen sind im Erzeuger-Skript aufgelistet. Zwei Faelle, die
/// nicht zu raten sind: Tkinter nennt Bild-auf `"Prior"` und
/// Bild-ab `"Next"`.
/// Nach Name sortiert, damit binaer gesucht werden kann.
pub static TK_KEYSYMS: &[(&str, Key)] = &[
    ("0", 48),
    ("1", 49),
    ("2", 50),
    ("3", 51),
    ("4", 52),
    ("5", 53),
    ("6", 54),
    ("7", 55),
    ("8", 56),
    ("9", 57),
    ("A", 97),
    ("Alt_L", 1073742050),
    ("Alt_R", 1073742054),
    ("B", 98),
    ("BackSpace", 8),
    ("Break", 1073741896),
    ("C", 99),
    ("Caps_Lock", 1073741881),
    ("Clear", 1073741980),
    ("Control_L", 1073742048),
    ("Control_R", 1073742052),
    ("D", 100),
    ("Delete", 127),
    ("Down", 1073741905),
    ("E", 101),
    ("End", 1073741901),
    ("Escape", 27),
    ("F", 102),
    ("F1", 1073741882),
    ("F10", 1073741891),
    ("F11", 1073741892),
    ("F12", 1073741893),
    ("F13", 1073741928),
    ("F14", 1073741929),
    ("F15", 1073741930),
    ("F2", 1073741883),
    ("F3", 1073741884),
    ("F4", 1073741885),
    ("F5", 1073741886),
    ("F6", 1073741887),
    ("F7", 1073741888),
    ("F8", 1073741889),
    ("F9", 1073741890),
    ("G", 103),
    ("H", 104),
    ("Help", 1073741941),
    ("Home", 1073741898),
    ("I", 105),
    ("Insert", 1073741897),
    ("J", 106),
    ("K", 107),
    ("KP_0", 1073741922),
    ("KP_1", 1073741913),
    ("KP_2", 1073741914),
    ("KP_3", 1073741915),
    ("KP_4", 1073741916),
    ("KP_5", 1073741917),
    ("KP_6", 1073741918),
    ("KP_7", 1073741919),
    ("KP_8", 1073741920),
    ("KP_9", 1073741921),
    ("KP_Add", 1073741911),
    ("KP_Begin", 1073741917),
    ("KP_Decimal", 1073741923),
    ("KP_Delete", 1073741923),
    ("KP_Divide", 1073741908),
    ("KP_Down", 1073741914),
    ("KP_End", 1073741913),
    ("KP_Enter", 1073741912),
    ("KP_Home", 1073741919),
    ("KP_Insert", 1073741922),
    ("KP_Left", 1073741916),
    ("KP_Multiply", 1073741909),
    ("KP_Next", 1073741915),
    ("KP_Prior", 1073741921),
    ("KP_Right", 1073741918),
    ("KP_Separator", 1073741923),
    ("KP_Subtract", 1073741910),
    ("KP_Up", 1073741920),
    ("L", 108),
    ("Left", 1073741904),
    ("M", 109),
    ("Menu", 1073741942),
    ("Meta_L", 1073742051),
    ("Meta_R", 1073742055),
    ("N", 110),
    ("Next", 1073741902),
    ("Num_Lock", 1073741907),
    ("O", 111),
    ("P", 112),
    ("Pause", 1073741896),
    ("Print", 1073741894),
    ("Prior", 1073741899),
    ("Q", 113),
    ("R", 114),
    ("Return", 13),
    ("Right", 1073741903),
    ("S", 115),
    ("Scroll_Lock", 1073741895),
    ("Shift_L", 1073742049),
    ("Shift_R", 1073742053),
    ("Super_L", 1073742051),
    ("Super_R", 1073742055),
    ("Sys_Req", 1073741978),
    ("T", 116),
    ("Tab", 9),
    ("U", 117),
    ("Up", 1073741906),
    ("V", 118),
    ("W", 119),
    ("X", 120),
    ("Y", 121),
    ("Z", 122),
    ("a", 97),
    ("ampersand", 38),
    ("apostrophe", 39),
    ("asciicircum", 94),
    ("asterisk", 42),
    ("at", 64),
    ("b", 98),
    ("backslash", 92),
    ("bracketleft", 91),
    ("bracketright", 93),
    ("c", 99),
    ("colon", 58),
    ("comma", 44),
    ("d", 100),
    ("dollar", 36),
    ("e", 101),
    ("equal", 61),
    ("exclam", 33),
    ("f", 102),
    ("g", 103),
    ("grave", 96),
    ("greater", 62),
    ("h", 104),
    ("i", 105),
    ("j", 106),
    ("k", 107),
    ("l", 108),
    ("less", 60),
    ("m", 109),
    ("minus", 45),
    ("n", 110),
    ("numbersign", 35),
    ("o", 111),
    ("p", 112),
    ("parenleft", 40),
    ("parenright", 41),
    ("percent", 37),
    ("period", 46),
    ("plus", 43),
    ("q", 113),
    ("question", 63),
    ("quotedbl", 34),
    ("quoteleft", 96),
    ("quoteright", 39),
    ("r", 114),
    ("s", 115),
    ("semicolon", 59),
    ("slash", 47),
    ("space", 32),
    ("t", 116),
    ("u", 117),
    ("underscore", 95),
    ("v", 118),
    ("w", 119),
    ("x", 120),
    ("y", 121),
    ("z", 122),
];

/// minifb-Taste -> pygame-Tastencode.
///
/// Links der Name der `minifb::Key`-Variante (so, wie `Debug` ihn
/// schreibt), rechts der pygame-Tastencode - aus echtem pygame
/// ausgelesen, nicht abgeschrieben. Das minifb-Hintergrundstueck
/// rechnet mit einem `match` um; ein Test prueft dieses `match`
/// gegen diese Tabelle, damit beide nicht auseinanderlaufen.
///
/// Nach Name sortiert, damit binaer gesucht werden kann.
pub static MINIFB_KEYS: &[(&str, Key)] = &[
    ("A", 97), // K_a
    ("Apostrophe", 39), // K_QUOTE
    ("B", 98), // K_b
    ("Backquote", 96), // K_BACKQUOTE
    ("Backslash", 92), // K_BACKSLASH
    ("Backspace", 8), // K_BACKSPACE
    ("C", 99), // K_c
    ("CapsLock", 1073741881), // K_CAPSLOCK
    ("Comma", 44), // K_COMMA
    ("D", 100), // K_d
    ("Delete", 127), // K_DELETE
    ("Down", 1073741905), // K_DOWN
    ("E", 101), // K_e
    ("End", 1073741901), // K_END
    ("Enter", 13), // K_RETURN
    ("Equal", 61), // K_EQUALS
    ("Escape", 27), // K_ESCAPE
    ("F", 102), // K_f
    ("F1", 1073741882), // K_F1
    ("F10", 1073741891), // K_F10
    ("F11", 1073741892), // K_F11
    ("F12", 1073741893), // K_F12
    ("F13", 1073741928), // K_F13
    ("F14", 1073741929), // K_F14
    ("F15", 1073741930), // K_F15
    ("F2", 1073741883), // K_F2
    ("F3", 1073741884), // K_F3
    ("F4", 1073741885), // K_F4
    ("F5", 1073741886), // K_F5
    ("F6", 1073741887), // K_F6
    ("F7", 1073741888), // K_F7
    ("F8", 1073741889), // K_F8
    ("F9", 1073741890), // K_F9
    ("G", 103), // K_g
    ("H", 104), // K_h
    ("Home", 1073741898), // K_HOME
    ("I", 105), // K_i
    ("Insert", 1073741897), // K_INSERT
    ("J", 106), // K_j
    ("K", 107), // K_k
    ("Key0", 48), // K_0
    ("Key1", 49), // K_1
    ("Key2", 50), // K_2
    ("Key3", 51), // K_3
    ("Key4", 52), // K_4
    ("Key5", 53), // K_5
    ("Key6", 54), // K_6
    ("Key7", 55), // K_7
    ("Key8", 56), // K_8
    ("Key9", 57), // K_9
    ("L", 108), // K_l
    ("Left", 1073741904), // K_LEFT
    ("LeftAlt", 1073742050), // K_LALT
    ("LeftBracket", 91), // K_LEFTBRACKET
    ("LeftCtrl", 1073742048), // K_LCTRL
    ("LeftShift", 1073742049), // K_LSHIFT
    ("LeftSuper", 1073742051), // K_LGUI
    ("M", 109), // K_m
    ("Menu", 1073741942), // K_MENU
    ("Minus", 45), // K_MINUS
    ("N", 110), // K_n
    ("NumLock", 1073741907), // K_NUMLOCK
    ("NumPad0", 1073741922), // K_KP0
    ("NumPad1", 1073741913), // K_KP1
    ("NumPad2", 1073741914), // K_KP2
    ("NumPad3", 1073741915), // K_KP3
    ("NumPad4", 1073741916), // K_KP4
    ("NumPad5", 1073741917), // K_KP5
    ("NumPad6", 1073741918), // K_KP6
    ("NumPad7", 1073741919), // K_KP7
    ("NumPad8", 1073741920), // K_KP8
    ("NumPad9", 1073741921), // K_KP9
    ("NumPadAsterisk", 1073741909), // K_KP_MULTIPLY
    ("NumPadDot", 1073741923), // K_KP_PERIOD
    ("NumPadEnter", 1073741912), // K_KP_ENTER
    ("NumPadMinus", 1073741910), // K_KP_MINUS
    ("NumPadPlus", 1073741911), // K_KP_PLUS
    ("NumPadSlash", 1073741908), // K_KP_DIVIDE
    ("O", 111), // K_o
    ("P", 112), // K_p
    ("PageDown", 1073741902), // K_PAGEDOWN
    ("PageUp", 1073741899), // K_PAGEUP
    ("Pause", 1073741896), // K_PAUSE
    ("Period", 46), // K_PERIOD
    ("Q", 113), // K_q
    ("R", 114), // K_r
    ("Right", 1073741903), // K_RIGHT
    ("RightAlt", 1073742054), // K_RALT
    ("RightBracket", 93), // K_RIGHTBRACKET
    ("RightCtrl", 1073742052), // K_RCTRL
    ("RightShift", 1073742053), // K_RSHIFT
    ("RightSuper", 1073742055), // K_RGUI
    ("S", 115), // K_s
    ("ScrollLock", 1073741895), // K_SCROLLLOCK
    ("Semicolon", 59), // K_SEMICOLON
    ("Slash", 47), // K_SLASH
    ("Space", 32), // K_SPACE
    ("T", 116), // K_t
    ("Tab", 9), // K_TAB
    ("U", 117), // K_u
    ("Unknown", 0), // K_UNKNOWN
    ("Up", 1073741906), // K_UP
    ("V", 118), // K_v
    ("W", 119), // K_w
    ("X", 120), // K_x
    ("Y", 121), // K_y
    ("Z", 122), // K_z
];

/// Ereignistyp -> Name, wie `pygame.event.event_name()` ihn liefert.
///
/// Gemessen: unbekannte Typen ergeben `"Unknown"`, und alles ab
/// `USEREVENT` heisst `"UserEvent"`.
pub static EVENT_NAMES: &[(EventType, &str)] = &[
    (0, "NoEvent"),
    (256, "Quit"),
    (257, "AppTerminating"),
    (258, "AppLowMemory"),
    (259, "AppWillEnterBackground"),
    (260, "AppDidEnterBackground"),
    (261, "AppWillEnterForeground"),
    (262, "AppDidEnterForeground"),
    (263, "LocaleChanged"),
    (513, "SysWMEvent"),
    (768, "KeyDown"),
    (769, "KeyUp"),
    (770, "TextEditing"),
    (771, "TextInput"),
    (772, "KeyMapChanged"),
    (1024, "MouseMotion"),
    (1025, "MouseButtonDown"),
    (1026, "MouseButtonUp"),
    (1027, "MouseWheel"),
    (1536, "JoyAxisMotion"),
    (1537, "JoyBallMotion"),
    (1538, "JoyHatMotion"),
    (1539, "JoyButtonDown"),
    (1540, "JoyButtonUp"),
    (1541, "JoyDeviceAdded"),
    (1542, "JoyDeviceRemoved"),
    (1616, "ControllerAxisMotion"),
    (1617, "ControllerButtonDown"),
    (1618, "ControllerButtonUp"),
    (1619, "ControllerDeviceAdded"),
    (1620, "ControllerDeviceRemoved"),
    (1621, "ControllerDeviceMapped"),
    (1622, "ControllerTouchpadDown"),
    (1623, "ControllerTouchpadMotion"),
    (1624, "ControllerTouchpadUp"),
    (1625, "ControllerSensorUpdate"),
    (1792, "FingerDown"),
    (1793, "FingerUp"),
    (1794, "FingerMotion"),
    (2050, "MultiGesture"),
    (2304, "ClipboardUpdate"),
    (4096, "DropFile"),
    (4097, "DropText"),
    (4098, "DropBegin"),
    (4099, "DropComplete"),
    (4352, "AudioDeviceAdded"),
    (4353, "AudioDeviceRemoved"),
    (8192, "RenderTargetsReset"),
    (8193, "RenderDeviceReset"),
    (32768, "ActiveEvent"),
    (32769, "VideoResize"),
    (32770, "VideoExpose"),
    (32771, "MidiIn"),
    (32772, "MidiOut"),
    (32773, "WindowShown"),
    (32774, "WindowHidden"),
    (32775, "WindowExposed"),
    (32776, "WindowMoved"),
    (32777, "WindowResized"),
    (32778, "WindowSizeChanged"),
    (32779, "WindowMinimized"),
    (32780, "WindowMaximized"),
    (32781, "WindowRestored"),
    (32782, "WindowEnter"),
    (32783, "WindowLeave"),
    (32784, "WindowFocusGained"),
    (32785, "WindowFocusLost"),
    (32786, "WindowClose"),
    (32787, "WindowTakeFocus"),
    (32788, "WindowHitTest"),
    (32789, "WindowICCProfChanged"),
    (32790, "WindowDisplayChanged"),
    (32865, "UserEvent"),
];

