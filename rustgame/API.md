# API-Abdeckung: pygame-ce 2.5.7 gegen rustgame

**Erzeugt** von `tools/api_coverage.py` - nicht von Hand aendern.

Die linke Spalte kommt aus dem **installierten pygame** (per
`dir()`), nicht aus einer gepflegten Liste. Sie kann also weder
veralten noch geschoenfaerbt werden.

Ein Haken heisst: **es gibt den Namen**. Ob sich alles dahinter
bis ins letzte Pixel wie pygame verhaelt, beantworten die
Referenzbilder (`cargo test --test golden`) und der differenzielle
Fuzzer (`cargo test --test fuzz`), nicht diese Tabelle.

## Uebersicht

| Bereich | da | von | Anteil |
|---|---:|---:|---:|
| Das Hauptmodul (`pygame`) | 679 | 679 | 100 % |
| Flaechen (`pygame.Surface`) | 56 | 56 | 100 % |
| Rechtecke (`pygame.Rect`) | 44 | 44 | 100 % |
| Rechtecke mit Komma (`pygame.FRect`) | 44 | 44 | 100 % |
| Farben (`pygame.Color`) | 18 | 18 | 100 % |
| Zeichnen (`pygame.draw`) | 11 | 11 | 100 % |
| Verwandeln (`pygame.transform`) | 22 | 22 | 100 % |
| Bilder (`pygame.image`) | 14 | 14 | 100 % |
| Zeit (`pygame.time`) | 5 | 5 | 100 % |
| Text (`pygame.font`) | 9 | 9 | 100 % |
| Fenster (`pygame.display`) | 35 | 35 | 100 % |
| Ereignisse (`pygame.event`) | 16 | 16 | 100 % |
| Tastatur (`pygame.key`) | 14 | 14 | 100 % |
| Maus (`pygame.mouse`) | 14 | 14 | 100 % |
| Ton (`pygame.mixer`) | 20 | 20 | 100 % |
| Sprites (`pygame.sprite`) | 21 | 21 | 100 % |
| Masken (`pygame.mask`) | 3 | 3 | 100 % |
| Vektoren (`pygame.math`) | 8 | 8 | 100 % |
| Zeigerformen (`pygame.cursors`) | 15 | 15 | 100 % |
| Spielhebel (`pygame.joystick`) | 6 | 6 | 100 % |
| Musik (`pygame.mixer.music`) | 17 | 17 | 100 % |
| Pixelfeld (`pygame.pixelarray`) | 1 | 1 | 100 % |
| Pixel kopieren (`pygame.pixelcopy`) | 4 | 4 | 100 % |
| Flaechen als Felder (`pygame.surfarray`) | 30 | 30 | 100 % |
| Klaenge als Felder (`pygame.sndarray`) | 9 | 9 | 100 % |
| Zwischenablage (`pygame.scrap`) | 11 | 11 | 100 % |
| System (`pygame.system`) | 4 | 4 | 100 % |
| Fenster als Objekt (`pygame.window`) | 2 | 2 | 100 % |
| Ein Fenster (`pygame.window.Window`) | 33 | 33 | 100 % |
| Fassungen (`pygame.version`) | 8 | 8 | 100 % |
| Farbnamen (`pygame.colordict`) | 1 | 1 | 100 % |
| Zeichnen mit Kantenglaettung (`pygame.gfxdraw`) | 22 | 22 | 100 % |
| Text ueber FreeType (`pygame.freetype`) | 20 | 20 | 100 % |
| Eine FreeType-Schrift (`pygame.freetype.Font`) | 39 | 39 | 100 % |
| MIDI (`pygame.midi`) | 20 | 20 | 100 % |
| Kamera (`pygame.camera`) | 14 | 14 | 100 % |
| Beschleunigtes Zeichnen (`pygame._sdl2.video`) | 14 | 14 | 100 % |
| Tongeraete (`pygame._sdl2.audio`) | 22 | 22 | 100 % |
| Spielgeraete, alt (`pygame._sdl2.controller_old`) | 11 | 11 | 100 % |
| Mischhaken (`pygame._sdl2.mixer`) | 3 | 3 | 100 % |
| Teilsysteme (`pygame._sdl2.sdl2`) | 11 | 11 | 100 % |
| Fenster (`pygame._sdl2.window`) | 1 | 1 | 100 % |
| Vektor in der Ebene (`pygame.math.Vector2`) | 38 | 38 | 100 % |
| Vektor im Raum (`pygame.math.Vector3`) | 52 | 52 | 100 % |
| Eine Maske (`pygame.mask.Mask`) | 23 | 23 | 100 % |
| Die Uhr (`pygame.time.Clock`) | 5 | 5 | 100 % |
| Eine Schrift (`pygame.font.Font`) | 29 | 29 | 100 % |
| Ein Klang (`pygame.mixer.Sound`) | 9 | 9 | 100 % |
| Eine Tonspur (`pygame.mixer.Channel`) | 15 | 15 | 100 % |
| Eine Spielfigur (`pygame.sprite.Sprite`) | 11 | 11 | 100 % |
| Eine Gruppe (`pygame.sprite.Group`) | 12 | 12 | 100 % |
| Geometrie (`pygame.geometry`) | 2 | 2 | 100 % |
| Grundfunktionen (`pygame.base`) | 12 | 12 | 100 % |
| Pufferzeiger (`pygame.bufferproxy`) | 1 | 1 | 100 % |
| Dateiobjekte (`pygame.rwobject`) | 2 | 2 | 100 % |
| Systemschriften (`pygame.sysfont`) | 25 | 25 | 100 % |
| Text ueber FreeType (`pygame.ftfont`) | 10 | 10 | 100 % |
| Beschleunigtes Zeichnen (`pygame._render`) | 3 | 3 | 100 % |
| Ein Kreis (`pygame.geometry.Circle`) | 30 | 30 | 100 % |
| Eine Linie (`pygame.geometry.Line`) | 16 | 16 | 100 % |
| Ein Pufferzeiger (`pygame.BufferProxy`) | 4 | 4 | 100 % |
| Eine Zeigerform (`pygame.Cursor`) | 1 | 1 | 100 % |
| Ein Ereignis (`pygame.Event`) | 2 | 2 | 100 % |
| Ein Pixelfeld (`pygame.PixelArray`) | 11 | 11 | 100 % |
| Die Gruppenbasis (`pygame.sprite.AbstractGroup`) | 12 | 12 | 100 % |
| Eine schmutzige Figur (`pygame.sprite.DirtySprite`) | 12 | 12 | 100 % |
| Eine Einzelgruppe (`pygame.sprite.GroupSingle`) | 13 | 13 | 100 % |
| Gruppe mit Ebenen (`pygame.sprite.LayeredUpdates`) | 25 | 25 | 100 % |
| Ebenen und schmutzige Rechtecke (`pygame.sprite.LayeredDirty`) | 30 | 30 | 100 % |
| Gruppe in Reihenfolge (`pygame.sprite.OrderedUpdates`) | 12 | 12 | 100 % |
| Gruppe mit Rechtecken (`pygame.sprite.RenderUpdates`) | 12 | 12 | 100 % |
| Zweitname von Group (`pygame.sprite.RenderPlain`) | 12 | 12 | 100 % |
| Zweitname von Group (`pygame.sprite.RenderClear`) | 12 | 12 | 100 % |
| MIDI-Eingang (`pygame.midi.Input`) | 3 | 3 | 100 % |
| MIDI-Ausgang (`pygame.midi.Output`) | 9 | 9 | 100 % |
| Die Fassung (`pygame.version.PygameVersion`) | 6 | 6 | 100 % |
| SDLs Fassung (`pygame.version.SDLVersion`) | 6 | 6 | 100 % |
| Ein Zeichner (`pygame._sdl2.video.Renderer`) | 23 | 23 | 100 % |
| Eine Textur (`pygame._sdl2.video.Texture`) | 12 | 12 | 100 % |
| Ein Bildausschnitt (`pygame._sdl2.video.Image`) | 11 | 11 | 100 % |
| Untermodul (`pygame.__pyinstaller`) | 2 | 2 | 100 % |
| Untermodul (`pygame._camera`) | 3 | 3 | 100 % |
| Untermodul (`pygame._data_classes`) | 2 | 2 | 100 % |
| Untermodul (`pygame._debug`) | 10 | 10 | 100 % |
| Untermodul (`pygame._freetype`) | 21 | 21 | 100 % |
| Untermodul (`pygame._sdl2`) | 53 | 53 | 100 % |
| Untermodul (`pygame._sprite`) | 19 | 19 | 100 % |
| Untermodul (`pygame.color`) | 2 | 2 | 100 % |
| Untermodul (`pygame.constants`) | 594 | 594 | 100 % |
| Untermodul (`pygame.docs`) | 3 | 3 | 100 % |
| Untermodul (`pygame.locals`) | 597 | 597 | 100 % |
| Untermodul (`pygame.macosx`) | 4 | 4 | 100 % |
| Untermodul (`pygame.newbuffer`) | 20 | 20 | 100 % |
| Untermodul (`pygame.pkgdata`) | 6 | 6 | 100 % |
| Untermodul (`pygame.pypm`) | 36 | 36 | 100 % |
| Untermodul (`pygame.rect`) | 4 | 4 | 100 % |
| Untermodul (`pygame.surface`) | 2 | 2 | 100 % |
| Untermodul (`pygame.typing`) | 7 | 7 | 100 % |
| Klasse (`pygame.BufferError`) | 3 | 3 | 100 % |
| Klasse (`pygame._camera.Camera`) | 8 | 8 | 100 % |
| Klasse (`pygame._render.Renderer`) | 23 | 23 | 100 % |
| Klasse (`pygame._render.Texture`) | 12 | 12 | 100 % |
| Klasse (`pygame._sdl2.AudioDevice`) | 10 | 10 | 100 % |
| Klasse (`pygame._sdl2.controller.Controller`) | 15 | 15 | 100 % |
| Klasse (`pygame._sdl2.controller_old.Controller`) | 14 | 14 | 100 % |
| Klasse (`pygame._sdl2.error`) | 3 | 3 | 100 % |
| Klasse (`pygame.camera.AbstractCamera`) | 6 | 6 | 100 % |
| Klasse (`pygame.camera.Camera`) | 8 | 8 | 100 % |
| Klasse (`pygame.error`) | 3 | 3 | 100 % |
| Klasse (`pygame.ftfont.Font`) | 54 | 54 | 100 % |
| Klasse (`pygame.joystick.JoystickType`) | 19 | 19 | 100 % |
| Klasse (`pygame.key.ScancodeWrapper`) | 2 | 2 | 100 % |
| Klasse (`pygame.midi.MidiException`) | 3 | 3 | 100 % |
| Klasse (`pygame.newbuffer.Py_buffer`) | 13 | 13 | 100 % |
| Klasse (`pygame.pypm.Input`) | 5 | 5 | 100 % |
| Klasse (`pygame.pypm.Output`) | 5 | 5 | 100 % |
| Klasse (`pygame.version.SoftwareVersion`) | 6 | 6 | 100 % |
| **Zusammen** | **3471** | **3471** | **100 %** |

## Das Hauptmodul (`pygame`)

679 von 679 (100 %)

**Da:** `ACTIVEEVENT`, `ANYFORMAT`, `APPACTIVE`, `APPINPUTFOCUS`, `APPMOUSEFOCUS`, `APP_DIDENTERBACKGROUND`, `APP_DIDENTERFOREGROUND`, `APP_LOWMEMORY`, `APP_TERMINATING`, `APP_WILLENTERBACKGROUND`, `APP_WILLENTERFOREGROUND`, `ASYNCBLIT`, `AUDIODEVICEADDED`, `AUDIODEVICEREMOVED`, `AUDIO_ALLOW_ANY_CHANGE`, `AUDIO_ALLOW_CHANNELS_CHANGE`, `AUDIO_ALLOW_FORMAT_CHANGE`, `AUDIO_ALLOW_FREQUENCY_CHANGE`, `AUDIO_S16`, `AUDIO_S16LSB`, `AUDIO_S16MSB`, `AUDIO_S16SYS`, `AUDIO_S8`, `AUDIO_U16`, `AUDIO_U16LSB`, `AUDIO_U16MSB`, `AUDIO_U16SYS`, `AUDIO_U8`, `BIG_ENDIAN`, `BLENDFACTOR_DST_ALPHA`, `BLENDFACTOR_DST_COLOR`, `BLENDFACTOR_ONE`, `BLENDFACTOR_ONE_MINUS_DST_ALPHA`, `BLENDFACTOR_ONE_MINUS_DST_COLOR`, `BLENDFACTOR_ONE_MINUS_SRC_ALPHA`, `BLENDFACTOR_ONE_MINUS_SRC_COLOR`, `BLENDFACTOR_SRC_ALPHA`, `BLENDFACTOR_SRC_COLOR`, `BLENDFACTOR_ZERO`, `BLENDMODE_ADD`, `BLENDMODE_BLEND`, `BLENDMODE_MOD`, `BLENDMODE_MUL`, `BLENDMODE_NONE`, `BLENDOPERATION_ADD`, `BLENDOPERATION_MAXIMUM`, `BLENDOPERATION_MINIMUM`, `BLENDOPERATION_REV_SUBTRACT`, `BLENDOPERATION_SUBTRACT`, `BLEND_ADD`, `BLEND_ALPHA_SDL2`, `BLEND_MAX`, `BLEND_MIN`, `BLEND_MULT`, `BLEND_PREMULTIPLIED`, `BLEND_RGBA_ADD`, `BLEND_RGBA_MAX`, `BLEND_RGBA_MIN`, `BLEND_RGBA_MULT`, `BLEND_RGBA_SUB`, `BLEND_RGB_ADD`, `BLEND_RGB_MAX`, `BLEND_RGB_MIN`, `BLEND_RGB_MULT`, `BLEND_RGB_SUB`, `BLEND_SUB`, `BUTTON_LEFT`, `BUTTON_MIDDLE`, `BUTTON_RIGHT`, `BUTTON_WHEELDOWN`, `BUTTON_WHEELUP`, `BUTTON_X1`, `BUTTON_X2`, `BufferError`, `BufferProxy`, `CLIPBOARDUPDATE`, `CONTROLLERAXISMOTION`, `CONTROLLERBUTTONDOWN`, `CONTROLLERBUTTONUP`, `CONTROLLERDEVICEADDED`, `CONTROLLERDEVICEREMAPPED`, `CONTROLLERDEVICEREMOVED`, `CONTROLLERSENSORUPDATE`, `CONTROLLERTOUCHPADDOWN`, `CONTROLLERTOUCHPADMOTION`, `CONTROLLERTOUCHPADUP`, `CONTROLLER_AXIS_INVALID`, `CONTROLLER_AXIS_LEFTX`, `CONTROLLER_AXIS_LEFTY`, `CONTROLLER_AXIS_MAX`, `CONTROLLER_AXIS_RIGHTX`, `CONTROLLER_AXIS_RIGHTY`, `CONTROLLER_AXIS_TRIGGERLEFT`, `CONTROLLER_AXIS_TRIGGERRIGHT`, `CONTROLLER_BUTTON_A`, `CONTROLLER_BUTTON_B`, `CONTROLLER_BUTTON_BACK`, `CONTROLLER_BUTTON_DPAD_DOWN`, `CONTROLLER_BUTTON_DPAD_LEFT`, `CONTROLLER_BUTTON_DPAD_RIGHT`, `CONTROLLER_BUTTON_DPAD_UP`, `CONTROLLER_BUTTON_GUIDE`, `CONTROLLER_BUTTON_INVALID`, `CONTROLLER_BUTTON_LEFTSHOULDER`, `CONTROLLER_BUTTON_LEFTSTICK`, `CONTROLLER_BUTTON_MAX`, `CONTROLLER_BUTTON_RIGHTSHOULDER`, `CONTROLLER_BUTTON_RIGHTSTICK`, `CONTROLLER_BUTTON_START`, `CONTROLLER_BUTTON_X`, `CONTROLLER_BUTTON_Y`, `Channel`, `Clock`, `Color`, `Cursor`, `DIRECTION_BTT`, `DIRECTION_LTR`, `DIRECTION_RTL`, `DIRECTION_TTB`, `DOUBLEBUF`, `DROPBEGIN`, `DROPCOMPLETE`, `DROPFILE`, `DROPTEXT`, `Event`, `FINGERDOWN`, `FINGERMOTION`, `FINGERUP`, `FLASH_BRIEFLY`, `FLASH_CANCEL`, `FLASH_UNTIL_FOCUSED`, `FLIP_HORIZONTAL`, `FLIP_NONE`, `FLIP_VERTICAL`, `FONT_CENTER`, `FONT_LEFT`, `FONT_RIGHT`, `FRect`, `FULLSCREEN`, `Font`, `GL_ACCELERATED_VISUAL`, `GL_ACCUM_ALPHA_SIZE`, `GL_ACCUM_BLUE_SIZE`, `GL_ACCUM_GREEN_SIZE`, `GL_ACCUM_RED_SIZE`, `GL_ALPHA_SIZE`, `GL_BLUE_SIZE`, `GL_BUFFER_SIZE`, `GL_CONTEXT_DEBUG_FLAG`, `GL_CONTEXT_FLAGS`, `GL_CONTEXT_FORWARD_COMPATIBLE_FLAG`, `GL_CONTEXT_MAJOR_VERSION`, `GL_CONTEXT_MINOR_VERSION`, `GL_CONTEXT_PROFILE_COMPATIBILITY`, `GL_CONTEXT_PROFILE_CORE`, `GL_CONTEXT_PROFILE_ES`, `GL_CONTEXT_PROFILE_MASK`, `GL_CONTEXT_RELEASE_BEHAVIOR`, `GL_CONTEXT_RELEASE_BEHAVIOR_FLUSH`, `GL_CONTEXT_RELEASE_BEHAVIOR_NONE`, `GL_CONTEXT_RESET_ISOLATION_FLAG`, `GL_CONTEXT_ROBUST_ACCESS_FLAG`, `GL_DEPTH_SIZE`, `GL_DOUBLEBUFFER`, `GL_FRAMEBUFFER_SRGB_CAPABLE`, `GL_GREEN_SIZE`, `GL_MULTISAMPLEBUFFERS`, `GL_MULTISAMPLESAMPLES`, `GL_RED_SIZE`, `GL_SHARE_WITH_CURRENT_CONTEXT`, `GL_STENCIL_SIZE`, `GL_STEREO`, `GL_SWAP_CONTROL`, `HAT_CENTERED`, `HAT_DOWN`, `HAT_LEFT`, `HAT_LEFTDOWN`, `HAT_LEFTUP`, `HAT_RIGHT`, `HAT_RIGHTDOWN`, `HAT_RIGHTUP`, `HAT_UP`, `HAVE_NEWBUF`, `HIDDEN`, `HWACCEL`, `HWPALETTE`, `HWSURFACE`, `IS_CE`, `JOYAXISMOTION`, `JOYBALLMOTION`, `JOYBUTTONDOWN`, `JOYBUTTONUP`, `JOYDEVICEADDED`, `JOYDEVICEREMOVED`, `JOYHATMOTION`, `Joystick`, `KEYDOWN`, `KEYMAPCHANGED`, `KEYUP`, `KMOD_ALT`, `KMOD_CAPS`, `KMOD_CTRL`, `KMOD_GUI`, `KMOD_LALT`, `KMOD_LCTRL`, `KMOD_LGUI`, `KMOD_LMETA`, `KMOD_LSHIFT`, `KMOD_META`, `KMOD_MODE`, `KMOD_NONE`, `KMOD_NUM`, `KMOD_RALT`, `KMOD_RCTRL`, `KMOD_RGUI`, `KMOD_RMETA`, `KMOD_RSHIFT`, `KMOD_SHIFT`, `KSCAN_0`, `KSCAN_1`, `KSCAN_2`, `KSCAN_3`, `KSCAN_4`, `KSCAN_5`, `KSCAN_6`, `KSCAN_7`, `KSCAN_8`, `KSCAN_9`, `KSCAN_A`, `KSCAN_AC_BACK`, `KSCAN_APOSTROPHE`, `KSCAN_B`, `KSCAN_BACKSLASH`, `KSCAN_BACKSPACE`, `KSCAN_BREAK`, `KSCAN_C`, `KSCAN_CAPSLOCK`, `KSCAN_CLEAR`, `KSCAN_COMMA`, `KSCAN_CURRENCYSUBUNIT`, `KSCAN_CURRENCYUNIT`, `KSCAN_D`, `KSCAN_DELETE`, `KSCAN_DOWN`, `KSCAN_E`, `KSCAN_END`, `KSCAN_EQUALS`, `KSCAN_ESCAPE`, `KSCAN_EURO`, `KSCAN_F`, `KSCAN_F1`, `KSCAN_F10`, `KSCAN_F11`, `KSCAN_F12`, `KSCAN_F13`, `KSCAN_F14`, `KSCAN_F15`, `KSCAN_F2`, `KSCAN_F3`, `KSCAN_F4`, `KSCAN_F5`, `KSCAN_F6`, `KSCAN_F7`, `KSCAN_F8`, `KSCAN_F9`, `KSCAN_G`, `KSCAN_GRAVE`, `KSCAN_H`, `KSCAN_HELP`, `KSCAN_HOME`, `KSCAN_I`, `KSCAN_INSERT`, `KSCAN_INTERNATIONAL1`, `KSCAN_INTERNATIONAL2`, `KSCAN_INTERNATIONAL3`, `KSCAN_INTERNATIONAL4`, `KSCAN_INTERNATIONAL5`, `KSCAN_INTERNATIONAL6`, `KSCAN_INTERNATIONAL7`, `KSCAN_INTERNATIONAL8`, `KSCAN_INTERNATIONAL9`, `KSCAN_J`, `KSCAN_K`, `KSCAN_KP0`, `KSCAN_KP1`, `KSCAN_KP2`, `KSCAN_KP3`, `KSCAN_KP4`, `KSCAN_KP5`, `KSCAN_KP6`, `KSCAN_KP7`, `KSCAN_KP8`, `KSCAN_KP9`, `KSCAN_KP_0`, `KSCAN_KP_1`, `KSCAN_KP_2`, `KSCAN_KP_3`, `KSCAN_KP_4`, `KSCAN_KP_5`, `KSCAN_KP_6`, `KSCAN_KP_7`, `KSCAN_KP_8`, `KSCAN_KP_9`, `KSCAN_KP_DIVIDE`, `KSCAN_KP_ENTER`, `KSCAN_KP_EQUALS`, `KSCAN_KP_MINUS`, `KSCAN_KP_MULTIPLY`, `KSCAN_KP_PERIOD`, `KSCAN_KP_PLUS`, `KSCAN_L`, `KSCAN_LALT`, `KSCAN_LANG1`, `KSCAN_LANG2`, `KSCAN_LANG3`, `KSCAN_LANG4`, `KSCAN_LANG5`, `KSCAN_LANG6`, `KSCAN_LANG7`, `KSCAN_LANG8`, `KSCAN_LANG9`, `KSCAN_LCTRL`, `KSCAN_LEFT`, `KSCAN_LEFTBRACKET`, `KSCAN_LGUI`, `KSCAN_LMETA`, `KSCAN_LSHIFT`, `KSCAN_LSUPER`, `KSCAN_M`, `KSCAN_MENU`, `KSCAN_MINUS`, `KSCAN_MODE`, `KSCAN_N`, `KSCAN_NONUSBACKSLASH`, `KSCAN_NONUSHASH`, `KSCAN_NUMLOCK`, `KSCAN_NUMLOCKCLEAR`, `KSCAN_O`, `KSCAN_P`, `KSCAN_PAGEDOWN`, `KSCAN_PAGEUP`, `KSCAN_PAUSE`, `KSCAN_PERIOD`, `KSCAN_POWER`, `KSCAN_PRINT`, `KSCAN_PRINTSCREEN`, `KSCAN_Q`, `KSCAN_R`, `KSCAN_RALT`, `KSCAN_RCTRL`, `KSCAN_RETURN`, `KSCAN_RGUI`, `KSCAN_RIGHT`, `KSCAN_RIGHTBRACKET`, `KSCAN_RMETA`, `KSCAN_RSHIFT`, `KSCAN_RSUPER`, `KSCAN_S`, `KSCAN_SCROLLLOCK`, `KSCAN_SCROLLOCK`, `KSCAN_SEMICOLON`, `KSCAN_SLASH`, `KSCAN_SPACE`, `KSCAN_SYSREQ`, `KSCAN_T`, `KSCAN_TAB`, `KSCAN_U`, `KSCAN_UNKNOWN`, `KSCAN_UP`, `KSCAN_V`, `KSCAN_W`, `KSCAN_X`, `KSCAN_Y`, `KSCAN_Z`, `K_0`, `K_1`, `K_2`, `K_3`, `K_4`, `K_5`, `K_6`, `K_7`, `K_8`, `K_9`, `K_AC_BACK`, `K_AMPERSAND`, `K_ASTERISK`, `K_AT`, `K_BACKQUOTE`, `K_BACKSLASH`, `K_BACKSPACE`, `K_BREAK`, `K_CAPSLOCK`, `K_CARET`, `K_CLEAR`, `K_COLON`, `K_COMMA`, `K_CURRENCYSUBUNIT`, `K_CURRENCYUNIT`, `K_DELETE`, `K_DOLLAR`, `K_DOWN`, `K_END`, `K_EQUALS`, `K_ESCAPE`, `K_EURO`, `K_EXCLAIM`, `K_F1`, `K_F10`, `K_F11`, `K_F12`, `K_F13`, `K_F14`, `K_F15`, `K_F2`, `K_F3`, `K_F4`, `K_F5`, `K_F6`, `K_F7`, `K_F8`, `K_F9`, `K_GREATER`, `K_HASH`, `K_HELP`, `K_HOME`, `K_INSERT`, `K_KP0`, `K_KP1`, `K_KP2`, `K_KP3`, `K_KP4`, `K_KP5`, `K_KP6`, `K_KP7`, `K_KP8`, `K_KP9`, `K_KP_0`, `K_KP_1`, `K_KP_2`, `K_KP_3`, `K_KP_4`, `K_KP_5`, `K_KP_6`, `K_KP_7`, `K_KP_8`, `K_KP_9`, `K_KP_DIVIDE`, `K_KP_ENTER`, `K_KP_EQUALS`, `K_KP_MINUS`, `K_KP_MULTIPLY`, `K_KP_PERIOD`, `K_KP_PLUS`, `K_LALT`, `K_LCTRL`, `K_LEFT`, `K_LEFTBRACKET`, `K_LEFTPAREN`, `K_LESS`, `K_LGUI`, `K_LMETA`, `K_LSHIFT`, `K_LSUPER`, `K_MENU`, `K_MINUS`, `K_MODE`, `K_NUMLOCK`, `K_NUMLOCKCLEAR`, `K_PAGEDOWN`, `K_PAGEUP`, `K_PAUSE`, `K_PERCENT`, `K_PERIOD`, `K_PLUS`, `K_POWER`, `K_PRINT`, `K_PRINTSCREEN`, `K_QUESTION`, `K_QUOTE`, `K_QUOTEDBL`, `K_RALT`, `K_RCTRL`, `K_RETURN`, `K_RGUI`, `K_RIGHT`, `K_RIGHTBRACKET`, `K_RIGHTPAREN`, `K_RMETA`, `K_RSHIFT`, `K_RSUPER`, `K_SCROLLLOCK`, `K_SCROLLOCK`, `K_SEMICOLON`, `K_SLASH`, `K_SPACE`, `K_SYSREQ`, `K_TAB`, `K_UNDERSCORE`, `K_UNKNOWN`, `K_UP`, `K_a`, `K_b`, `K_c`, `K_d`, `K_e`, `K_f`, `K_g`, `K_h`, `K_i`, `K_j`, `K_k`, `K_l`, `K_m`, `K_n`, `K_o`, `K_p`, `K_q`, `K_r`, `K_s`, `K_t`, `K_u`, `K_v`, `K_w`, `K_x`, `K_y`, `K_z`, `LIL_ENDIAN`, `LOCALECHANGED`, `MIDIIN`, `MIDIOUT`, `MOUSEBUTTONDOWN`, `MOUSEBUTTONUP`, `MOUSEMOTION`, `MOUSEWHEEL`, `MULTIGESTURE`, `Mask`, `NOEVENT`, `NOFRAME`, `NULL_VIDEODRIVER`, `NUMEVENTS`, `OPENGL`, `OPENGLBLIT`, `PREALLOC`, `PixelArray`, `QUIT`, `RENDER_DEVICE_RESET`, `RENDER_TARGETS_RESET`, `RESIZABLE`, `RLEACCEL`, `RLEACCELOK`, `Rect`, `SCALED`, `SCRAP_BMP`, `SCRAP_CLIPBOARD`, `SCRAP_PBM`, `SCRAP_PPM`, `SCRAP_SELECTION`, `SCRAP_TEXT`, `SCROLL_ERASE`, `SCROLL_REPEAT`, `SDL`, `SHOWN`, `SRCALPHA`, `SRCCOLORKEY`, `SWSURFACE`, `SYSTEM_CURSOR_ARROW`, `SYSTEM_CURSOR_CROSSHAIR`, `SYSTEM_CURSOR_HAND`, `SYSTEM_CURSOR_IBEAM`, `SYSTEM_CURSOR_NO`, `SYSTEM_CURSOR_SIZEALL`, `SYSTEM_CURSOR_SIZENESW`, `SYSTEM_CURSOR_SIZENS`, `SYSTEM_CURSOR_SIZENWSE`, `SYSTEM_CURSOR_SIZEWE`, `SYSTEM_CURSOR_WAIT`, `SYSTEM_CURSOR_WAITARROW`, `SYSWMEVENT`, `Sound`, `Surface`, `SurfaceType`, `TEXTEDITING`, `TEXTINPUT`, `TEXTUREACCESS_STATIC`, `TEXTUREACCESS_STREAMING`, `TEXTUREACCESS_TARGET`, `TIMER_RESOLUTION`, `USEREVENT`, `USEREVENT_DROPFILE`, `VIDEOEXPOSE`, `VIDEORESIZE`, `Vector2`, `Vector3`, `WINDOWCLOSE`, `WINDOWDISPLAYCHANGED`, `WINDOWENTER`, `WINDOWEXPOSED`, `WINDOWFOCUSGAINED`, `WINDOWFOCUSLOST`, `WINDOWHIDDEN`, `WINDOWHITTEST`, `WINDOWICCPROFCHANGED`, `WINDOWLEAVE`, `WINDOWMAXIMIZED`, `WINDOWMINIMIZED`, `WINDOWMOVED`, `WINDOWPOS_CENTERED`, `WINDOWPOS_UNDEFINED`, `WINDOWRESIZED`, `WINDOWRESTORED`, `WINDOWSHOWN`, `WINDOWSIZECHANGED`, `WINDOWTAKEFOCUS`, `Window`, `base`, `bufferproxy`, `camera`, `color`, `colordict`, `constants`, `cursors`, `display`, `docs`, `draw`, `encode_file_path`, `encode_string`, `error`, `event`, `examples`, `font`, `freetype`, `ftfont`, `geometry`, `get_array_interface`, `get_error`, `get_init`, `get_sdl_byteorder`, `get_sdl_version`, `gfxdraw`, `image`, `init`, `joystick`, `key`, `locals`, `macosx`, `mask`, `math`, `midi`, `mixer`, `mixer_music`, `mouse`, `newbuffer`, `pixelarray`, `pixelcopy`, `pkgdata`, `print_debug_info`, `pypm`, `quit`, `rect`, `register_quit`, `rev`, `rwobject`, `scrap`, `set_error`, `sndarray`, `sprite`, `surface`, `surfarray`, `surflock`, `sysfont`, `system`, `time`, `transform`, `typing`, `ver`, `vernum`, `version`, `window`

## Flaechen (`pygame.Surface`)

56 von 56 (100 %)

**Da:** `blit`, `blits`, `convert`, `convert_alpha`, `copy`, `fblits`, `fill`, `get_abs_offset`, `get_abs_parent`, `get_alpha`, `get_at`, `get_at_mapped`, `get_bitsize`, `get_blendmode`, `get_bounding_rect`, `get_buffer`, `get_bytesize`, `get_clip`, `get_colorkey`, `get_flags`, `get_frect`, `get_height`, `get_locked`, `get_locks`, `get_losses`, `get_masks`, `get_offset`, `get_palette`, `get_palette_at`, `get_parent`, `get_pitch`, `get_rect`, `get_shifts`, `get_size`, `get_view`, `get_width`, `height`, `lock`, `map_rgb`, `mustlock`, `premul_alpha`, `premul_alpha_ip`, `scroll`, `set_alpha`, `set_at`, `set_clip`, `set_colorkey`, `set_masks`, `set_palette`, `set_palette_at`, `set_shifts`, `size`, `subsurface`, `unlock`, `unmap_rgb`, `width`

## Rechtecke (`pygame.Rect`)

44 von 44 (100 %)

**Da:** `bottom`, `bottomleft`, `bottomright`, `center`, `centerx`, `centery`, `clamp`, `clamp_ip`, `clip`, `clipline`, `collidelist`, `collidelistall`, `collideobjects`, `collideobjectsall`, `collidepoint`, `colliderect`, `contains`, `copy`, `fit`, `height`, `inflate`, `inflate_ip`, `left`, `midbottom`, `midleft`, `midright`, `midtop`, `move`, `move_ip`, `normalize`, `relcenter`, `right`, `scale_by`, `scale_by_ip`, `size`, `top`, `topleft`, `topright`, `union`, `union_ip`, `unionall`, `unionall_ip`, `update`, `width`

**Anders gebaut** (zaehlt weder als da noch als fehlend):

- `collidedict` -> es gibt kein Python-Woerterbuch - `collideobjects`
- `collidedictall` -> dito - `collideobjectsall`
- `h` -> oeffentliches Feld `r.h`
- `move_to` -> `get_rect().with_center(p)` und die 19 Geschwister
- `w` -> oeffentliches Feld `r.w`
- `x` -> oeffentliches Feld `r.x`
- `y` -> oeffentliches Feld `r.y`

## Rechtecke mit Komma (`pygame.FRect`)

44 von 44 (100 %)

**Da:** `bottom`, `bottomleft`, `bottomright`, `center`, `centerx`, `centery`, `clamp`, `clamp_ip`, `clip`, `clipline`, `collidelist`, `collidelistall`, `collideobjects`, `collideobjectsall`, `collidepoint`, `colliderect`, `contains`, `copy`, `fit`, `height`, `inflate`, `inflate_ip`, `left`, `midbottom`, `midleft`, `midright`, `midtop`, `move`, `move_ip`, `normalize`, `relcenter`, `right`, `scale_by`, `scale_by_ip`, `size`, `top`, `topleft`, `topright`, `union`, `union_ip`, `unionall`, `unionall_ip`, `update`, `width`

**Anders gebaut** (zaehlt weder als da noch als fehlend):

- `collidedict` -> es gibt kein Python-Woerterbuch - `collideobjects`
- `collidedictall` -> dito - `collideobjectsall`
- `h` -> oeffentliches Feld `r.h`
- `move_to` -> `get_frect().with_center(p)` und die 19 Geschwister
- `w` -> oeffentliches Feld `r.w`
- `x` -> oeffentliches Feld `r.x`
- `y` -> oeffentliches Feld `r.y`

## Farben (`pygame.Color`)

18 von 18 (100 %)

**Da:** `cmy`, `correct_gamma`, `from_cmy`, `from_hex`, `from_hsla`, `from_hsva`, `from_i1i2i3`, `from_normalized`, `grayscale`, `hex`, `hsla`, `hsva`, `i1i2i3`, `lerp`, `normalize`, `normalized`, `premul_alpha`, `update`

**Anders gebaut** (zaehlt weder als da noch als fehlend):

- `a` -> oeffentliches Feld `c.a`
- `b` -> oeffentliches Feld `c.b`
- `g` -> oeffentliches Feld `c.g`
- `r` -> oeffentliches Feld `c.r`
- `set_length` -> seit pygame 2.1.3 veraltet - `Color` hat hier immer vier Kanaele

## Zeichnen (`pygame.draw`)

11 von 11 (100 %)

**Da:** `aacircle`, `aaline`, `aalines`, `arc`, `circle`, `ellipse`, `flood_fill`, `line`, `lines`, `polygon`, `rect`

## Verwandeln (`pygame.transform`)

22 von 22 (100 %)

**Da:** `average_color`, `average_surfaces`, `box_blur`, `chop`, `flip`, `gaussian_blur`, `get_smoothscale_backend`, `grayscale`, `hsl`, `invert`, `laplacian`, `pixelate`, `rotate`, `rotozoom`, `scale`, `scale2x`, `scale_by`, `set_smoothscale_backend`, `smoothscale`, `smoothscale_by`, `solid_overlay`, `threshold`

## Bilder (`pygame.image`)

14 von 14 (100 %)

**Da:** `frombuffer`, `frombytes`, `fromstring`, `get_extended`, `get_sdl_image_version`, `load`, `load_animation`, `load_basic`, `load_extended`, `load_sized_svg`, `save`, `save_extended`, `tobytes`, `tostring`

## Zeit (`pygame.time`)

5 von 5 (100 %)

**Da:** `Clock`, `delay`, `get_ticks`, `set_timer`, `wait`

## Text (`pygame.font`)

9 von 9 (100 %)

**Da:** `Font`, `UCS4`, `get_default_font`, `get_fonts`, `get_init`, `get_sdl_ttf_version`, `init`, `match_font`, `quit`

**Anders gebaut** (zaehlt weder als da noch als fehlend):

- `FontType` -> Zweitname von `Font`
- `SysFont` -> `sys_font` - `SysFont` waere in Rust eine Warnung (`non_snake_case`)

## Fenster (`pygame.display`)

35 von 35 (100 %)

**Da:** `Info`, `flip`, `get_active`, `get_allow_screensaver`, `get_caption`, `get_current_refresh_rate`, `get_desktop_refresh_rates`, `get_desktop_sizes`, `get_driver`, `get_init`, `get_num_displays`, `get_surface`, `get_window_position`, `get_window_size`, `get_wm_info`, `gl_get_attribute`, `gl_set_attribute`, `iconify`, `init`, `is_fullscreen`, `is_vsync`, `list_modes`, `message_box`, `mode_ok`, `quit`, `set_allow_screensaver`, `set_caption`, `set_gamma`, `set_gamma_ramp`, `set_icon`, `set_mode`, `set_palette`, `set_window_position`, `toggle_fullscreen`, `update`

## Ereignisse (`pygame.event`)

16 von 16 (100 %)

**Da:** `Event`, `EventType`, `clear`, `custom_type`, `event_name`, `get`, `get_blocked`, `get_grab`, `peek`, `poll`, `post`, `pump`, `set_allowed`, `set_blocked`, `set_grab`, `wait`

## Tastatur (`pygame.key`)

14 von 14 (100 %)

**Da:** `ScancodeWrapper`, `get_focused`, `get_just_pressed`, `get_just_released`, `get_mods`, `get_pressed`, `get_repeat`, `key_code`, `name`, `set_mods`, `set_repeat`, `set_text_input_rect`, `start_text_input`, `stop_text_input`

## Maus (`pygame.mouse`)

14 von 14 (100 %)

**Da:** `get_cursor`, `get_focused`, `get_just_pressed`, `get_just_released`, `get_pos`, `get_pressed`, `get_rel`, `get_relative_mode`, `get_visible`, `set_cursor`, `set_pos`, `set_relative_mode`, `set_system_cursor`, `set_visible`

## Ton (`pygame.mixer`)

20 von 20 (100 %)

**Da:** `Channel`, `Sound`, `fadeout`, `find_channel`, `get_busy`, `get_driver`, `get_init`, `get_num_channels`, `get_sdl_mixer_version`, `get_soundfont`, `init`, `music`, `pause`, `pre_init`, `quit`, `set_num_channels`, `set_reserved`, `set_soundfont`, `stop`, `unpause`

**Anders gebaut** (zaehlt weder als da noch als fehlend):

- `ChannelType` -> Zweitname von `Channel`
- `SoundType` -> Zweitname von `Sound`

## Sprites (`pygame.sprite`)

21 von 21 (100 %)

**Da:** `AbstractGroup`, `DirtySprite`, `Group`, `GroupSingle`, `LayeredDirty`, `LayeredUpdates`, `OrderedUpdates`, `RenderClear`, `RenderPlain`, `RenderUpdates`, `Sprite`, `collide_circle`, `collide_circle_ratio`, `collide_mask`, `collide_rect`, `collide_rect_ratio`, `groupcollide`, `pygame`, `spritecollide`, `spritecollideany`, `types`

**Anders gebaut** (zaehlt weder als da noch als fehlend):

- `Rect` -> `rustgame_core::Rect`, von der Kiste weitergereicht
- `from_surface` -> `rustgame_math::mask::from_surface`
- `get_ticks` -> `rustgame_core::time::get_ticks`
- `warn` -> Pythons eigenes `warnings.warn` - gibt es in Rust nicht

## Masken (`pygame.mask`)

3 von 3 (100 %)

**Da:** `Mask`, `from_surface`, `from_threshold`

**Anders gebaut** (zaehlt weder als da noch als fehlend):

- `MaskType` -> Zweitname von `Mask` (in pygame ist es dasselbe Objekt)

## Vektoren (`pygame.math`)

8 von 8 (100 %)

**Da:** `Vector2`, `Vector3`, `VectorElementwiseProxy`, `clamp`, `invlerp`, `lerp`, `remap`, `smoothstep`

**Anders gebaut** (zaehlt weder als da noch als fehlend):

- `VectorIterator` -> Python-Hilfstyp; in Rust uebernimmt das `Iterator`
- `disable_swizzling` -> dito
- `enable_swizzling` -> in pygame abgeschafft - der Aufruf warnt, dass die Wirkung entfernt wurde

## Zeigerformen (`pygame.cursors`)

15 von 15 (100 %)

**Da:** `Cursor`, `arrow`, `ball`, `broken_x`, `compile`, `diamond`, `load_xbm`, `pygame`, `sizer_x_strings`, `sizer_xy_strings`, `sizer_y_strings`, `textmarker_strings`, `thickarrow_strings`, `tri_left`, `tri_right`

## Spielhebel (`pygame.joystick`)

6 von 6 (100 %)

**Da:** `Joystick`, `JoystickType`, `get_count`, `get_init`, `init`, `quit`

## Musik (`pygame.mixer.music`)

17 von 17 (100 %)

**Da:** `fadeout`, `get_busy`, `get_endevent`, `get_metadata`, `get_pos`, `get_volume`, `load`, `pause`, `play`, `queue`, `rewind`, `set_endevent`, `set_pos`, `set_volume`, `stop`, `unload`, `unpause`

## Pixelfeld (`pygame.pixelarray`)

1 von 1 (100 %)

**Da:** `PixelArray`

## Pixel kopieren (`pygame.pixelcopy`)

4 von 4 (100 %)

**Da:** `array_to_surface`, `make_surface`, `map_array`, `surface_to_array`

## Flaechen als Felder (`pygame.surfarray`)

30 von 30 (100 %)

**Da:** `array2d`, `array3d`, `array_alpha`, `array_blue`, `array_colorkey`, `array_green`, `array_red`, `array_to_surface`, `blit_array`, `get_arraytype`, `get_arraytypes`, `make_surface`, `map_array`, `numpy`, `numpy_array`, `numpy_empty`, `numpy_floats`, `numpy_ndarray`, `numpy_uint32`, `pix_make_surface`, `pix_map_array`, `pixels2d`, `pixels3d`, `pixels_alpha`, `pixels_blue`, `pixels_green`, `pixels_red`, `surface_to_array`, `use_arraytype`, `warnings`

## Klaenge als Felder (`pygame.sndarray`)

9 von 9 (100 %)

**Da:** `array`, `get_arraytype`, `get_arraytypes`, `make_sound`, `mixer`, `numpy`, `samples`, `use_arraytype`, `warnings`

## Zwischenablage (`pygame.scrap`)

11 von 11 (100 %)

**Da:** `contains`, `get`, `get_init`, `get_text`, `get_types`, `has_text`, `init`, `lost`, `put`, `put_text`, `set_mode`

## System (`pygame.system`)

4 von 4 (100 %)

**Da:** `PowerState`, `get_cpu_instruction_sets`, `get_pref_path`, `get_total_ram`

**Anders gebaut** (zaehlt weder als da noch als fehlend):

- `get_power_state` -> antwortet `None` - "nicht zu ermitteln"; den Akku wirklich zu lesen braeuchte `GetSystemPowerStatus` ueber FFI, und pygames eigener Test laesst `None` ausdruecklich zu
- `get_pref_locales` -> liest `LC_ALL`, `LC_MESSAGES`, `LANG`, `LANGUAGE` statt `GetUserPreferredUILanguages` - auf Windows sind die gewoehnlich leer, dann kommt eine leere Liste

## Fenster als Objekt (`pygame.window`)

2 von 2 (100 %)

**Da:** `Window`, `get_grabbed_window`

## Ein Fenster (`pygame.window.Window`)

33 von 33 (100 %)

**Da:** `always_on_top`, `borderless`, `destroy`, `flash`, `flip`, `focus`, `focused`, `from_display_module`, `get_surface`, `grab_keyboard`, `grab_mouse`, `hide`, `id`, `keyboard_grabbed`, `maximize`, `maximum_size`, `minimize`, `minimum_size`, `mouse_grabbed`, `mouse_rect`, `opacity`, `opengl`, `position`, `resizable`, `restore`, `set_fullscreen`, `set_icon`, `set_modal_for`, `set_windowed`, `show`, `size`, `title`, `utility`

## Fassungen (`pygame.version`)

8 von 8 (100 %)

**Da:** `PygameVersion`, `SDL`, `SDLVersion`, `SoftwareVersion`, `get_sdl_version`, `rev`, `ver`, `vernum`

## Farbnamen (`pygame.colordict`)

1 von 1 (100 %)

**Da:** `THECOLORS`

## Zeichnen mit Kantenglaettung (`pygame.gfxdraw`)

22 von 22 (100 %)

**Da:** `aacircle`, `aaellipse`, `aapolygon`, `aatrigon`, `arc`, `bezier`, `box`, `circle`, `ellipse`, `filled_circle`, `filled_ellipse`, `filled_polygon`, `filled_trigon`, `hline`, `line`, `pie`, `pixel`, `polygon`, `rectangle`, `textured_polygon`, `trigon`, `vline`

## Text ueber FreeType (`pygame.freetype`)

20 von 20 (100 %)

**Da:** `Font`, `STYLE_DEFAULT`, `STYLE_NORMAL`, `STYLE_OBLIQUE`, `STYLE_STRONG`, `STYLE_UNDERLINE`, `STYLE_WIDE`, `SysFont`, `get_cache_size`, `get_default_font`, `get_default_resolution`, `get_error`, `get_fonts`, `get_init`, `get_version`, `init`, `match_font`, `quit`, `set_default_resolution`, `was_init`

## Eine FreeType-Schrift (`pygame.freetype.Font`)

39 von 39 (100 %)

**Da:** `antialiased`, `ascender`, `bgcolor`, `descender`, `fgcolor`, `fixed_sizes`, `fixed_width`, `get_metrics`, `get_rect`, `get_sized_ascender`, `get_sized_descender`, `get_sized_glyph_height`, `get_sized_height`, `get_sizes`, `height`, `kerning`, `name`, `oblique`, `origin`, `pad`, `path`, `render`, `render_raw`, `render_raw_to`, `render_to`, `resolution`, `rotation`, `scalable`, `size`, `strength`, `strong`, `style`, `style_name`, `ucs4`, `underline`, `underline_adjustment`, `use_bitmap_strikes`, `vertical`, `wide`

## MIDI (`pygame.midi`)

20 von 20 (100 %)

**Da:** `Input`, `MIDIIN`, `MIDIOUT`, `MidiException`, `Output`, `atexit`, `frequency_to_midi`, `get_count`, `get_default_input_id`, `get_default_output_id`, `get_device_info`, `get_init`, `init`, `math`, `midi_to_ansi_note`, `midi_to_frequency`, `midis2events`, `pygame`, `quit`, `time`

## Kamera (`pygame.camera`)

14 von 14 (100 %)

**Da:** `ABC`, `AbstractCamera`, `Camera`, `abstractmethod`, `colorspace`, `error`, `get_backends`, `init`, `list_cameras`, `os`, `platform`, `quit`, `sys`, `warnings`

## Beschleunigtes Zeichnen (`pygame._sdl2.video`)

14 von 14 (100 %)

**Da:** `Image`, `Renderer`, `RendererDriverInfo`, `SCALEQUALITY_BEST`, `SCALEQUALITY_LINEAR`, `SCALEQUALITY_NEAREST`, `Texture`, `WINDOWPOS_CENTERED`, `WINDOWPOS_UNDEFINED`, `Window`, `error`, `errorfnc`, `get_drivers`, `get_grabbed_window`

## Tongeraete (`pygame._sdl2.audio`)

22 von 22 (100 %)

**Da:** `AUDIO_ALLOW_ANY_CHANGE`, `AUDIO_ALLOW_CHANNELS_CHANGE`, `AUDIO_ALLOW_FORMAT_CHANGE`, `AUDIO_ALLOW_FREQUENCY_CHANGE`, `AUDIO_F32`, `AUDIO_F32LSB`, `AUDIO_F32MSB`, `AUDIO_S16`, `AUDIO_S16LSB`, `AUDIO_S16MSB`, `AUDIO_S32`, `AUDIO_S32LSB`, `AUDIO_S32MSB`, `AUDIO_S8`, `AUDIO_U16`, `AUDIO_U16LSB`, `AUDIO_U16MSB`, `AUDIO_U8`, `AudioDevice`, `error`, `get_audio_device_names`, `traceback`

## Spielgeraete, alt (`pygame._sdl2.controller_old`)

11 von 11 (100 %)

**Da:** `Controller`, `error`, `get_count`, `get_eventstate`, `get_init`, `init`, `is_controller`, `name_forindex`, `quit`, `set_eventstate`, `update`

## Mischhaken (`pygame._sdl2.mixer`)

3 von 3 (100 %)

**Da:** `error`, `set_post_mix`, `traceback`

## Teilsysteme (`pygame._sdl2.sdl2`)

11 von 11 (100 %)

**Da:** `INIT_AUDIO`, `INIT_EVENTS`, `INIT_EVERYTHING`, `INIT_GAMECONTROLLER`, `INIT_HAPTIC`, `INIT_JOYSTICK`, `INIT_NOPARACHUTE`, `INIT_TIMER`, `INIT_VIDEO`, `error`, `init_subsystem`

## Fenster (`pygame._sdl2.window`)

1 von 1 (100 %)

**Da:** `Window`

## Vektor in der Ebene (`pygame.math.Vector2`)

38 von 38 (100 %)

**Da:** `angle`, `angle_rad`, `angle_to`, `as_polar`, `clamp_magnitude`, `clamp_magnitude_ip`, `copy`, `cross`, `distance_squared_to`, `distance_to`, `dot`, `elementwise`, `epsilon`, `from_polar`, `is_normalized`, `length`, `length_squared`, `lerp`, `magnitude`, `magnitude_squared`, `move_towards`, `move_towards_ip`, `normalize`, `normalize_ip`, `project`, `reflect`, `reflect_ip`, `rotate`, `rotate_ip`, `rotate_ip_rad`, `rotate_rad`, `rotate_rad_ip`, `scale_to_length`, `slerp`, `smoothstep`, `update`, `x`, `y`

## Vektor im Raum (`pygame.math.Vector3`)

52 von 52 (100 %)

**Da:** `angle_to`, `as_spherical`, `clamp_magnitude`, `clamp_magnitude_ip`, `copy`, `cross`, `distance_squared_to`, `distance_to`, `dot`, `elementwise`, `epsilon`, `from_spherical`, `is_normalized`, `length`, `length_squared`, `lerp`, `magnitude`, `magnitude_squared`, `move_towards`, `move_towards_ip`, `normalize`, `normalize_ip`, `project`, `reflect`, `reflect_ip`, `rotate`, `rotate_ip`, `rotate_ip_rad`, `rotate_rad`, `rotate_rad_ip`, `rotate_x`, `rotate_x_ip`, `rotate_x_ip_rad`, `rotate_x_rad`, `rotate_x_rad_ip`, `rotate_y`, `rotate_y_ip`, `rotate_y_ip_rad`, `rotate_y_rad`, `rotate_y_rad_ip`, `rotate_z`, `rotate_z_ip`, `rotate_z_ip_rad`, `rotate_z_rad`, `rotate_z_rad_ip`, `scale_to_length`, `slerp`, `smoothstep`, `update`, `x`, `y`, `z`

## Eine Maske (`pygame.mask.Mask`)

23 von 23 (100 %)

**Da:** `angle`, `centroid`, `clear`, `connected_component`, `connected_components`, `convolve`, `copy`, `count`, `draw`, `erase`, `fill`, `get_at`, `get_bounding_rects`, `get_rect`, `get_size`, `invert`, `outline`, `overlap`, `overlap_area`, `overlap_mask`, `scale`, `set_at`, `to_surface`

## Die Uhr (`pygame.time.Clock`)

5 von 5 (100 %)

**Da:** `get_fps`, `get_rawtime`, `get_time`, `tick`, `tick_busy_loop`

## Eine Schrift (`pygame.font.Font`)

29 von 29 (100 %)

**Da:** `align`, `bold`, `get_ascent`, `get_bold`, `get_descent`, `get_height`, `get_italic`, `get_linesize`, `get_point_size`, `get_strikethrough`, `get_underline`, `italic`, `metrics`, `name`, `outline`, `point_size`, `render`, `set_bold`, `set_direction`, `set_italic`, `set_linesize`, `set_point_size`, `set_script`, `set_strikethrough`, `set_underline`, `size`, `strikethrough`, `style_name`, `underline`

## Ein Klang (`pygame.mixer.Sound`)

9 von 9 (100 %)

**Da:** `copy`, `fadeout`, `get_length`, `get_num_channels`, `get_raw`, `get_volume`, `play`, `set_volume`, `stop`

## Eine Tonspur (`pygame.mixer.Channel`)

15 von 15 (100 %)

**Da:** `fadeout`, `get_busy`, `get_endevent`, `get_queue`, `get_sound`, `get_volume`, `id`, `pause`, `play`, `queue`, `set_endevent`, `set_source_location`, `set_volume`, `stop`, `unpause`

## Eine Spielfigur (`pygame.sprite.Sprite`)

11 von 11 (100 %)

**Da:** `add`, `add_internal`, `alive`, `groups`, `image`, `kill`, `layer`, `rect`, `remove`, `remove_internal`, `update`

## Eine Gruppe (`pygame.sprite.Group`)

12 von 12 (100 %)

**Da:** `add`, `add_internal`, `clear`, `copy`, `draw`, `empty`, `has`, `has_internal`, `remove`, `remove_internal`, `sprites`, `update`

## Geometrie (`pygame.geometry`)

2 von 2 (100 %)

**Da:** `Circle`, `Line`

## Grundfunktionen (`pygame.base`)

12 von 12 (100 %)

**Da:** `BufferError`, `HAVE_NEWBUF`, `error`, `get_array_interface`, `get_error`, `get_init`, `get_sdl_byteorder`, `get_sdl_version`, `init`, `quit`, `register_quit`, `set_error`

## Pufferzeiger (`pygame.bufferproxy`)

1 von 1 (100 %)

**Da:** `BufferProxy`

## Dateiobjekte (`pygame.rwobject`)

2 von 2 (100 %)

**Da:** `encode_file_path`, `encode_string`

## Systemschriften (`pygame.sysfont`)

25 von 25 (100 %)

**Da:** `Font`, `OpenType_extensions`, `SysFont`, `Sysalias`, `Sysfonts`, `basename`, `create_aliases`, `difflib`, `dirname`, `exists`, `font_constructor`, `get_fonts`, `initsysfonts`, `initsysfonts_darwin`, `initsysfonts_unix`, `initsysfonts_win32`, `is_init`, `itertools`, `join`, `match_font`, `os`, `pygame_main_file`, `splitext`, `sys`, `warnings`

## Text ueber FreeType (`pygame.ftfont`)

10 von 10 (100 %)

**Da:** `Font`, `FontType`, `SysFont`, `draw`, `get_default_font`, `get_fonts`, `get_init`, `init`, `match_font`, `quit`

## Beschleunigtes Zeichnen (`pygame._render`)

3 von 3 (100 %)

**Da:** `Image`, `Renderer`, `Texture`

## Ein Kreis (`pygame.geometry.Circle`)

30 von 30 (100 %)

**Da:** `area`, `as_frect`, `as_rect`, `bottom`, `center`, `circumference`, `collidecircle`, `collidelist`, `collidelistall`, `collidepoint`, `colliderect`, `collideswith`, `contains`, `copy`, `d`, `diameter`, `intersect`, `left`, `move`, `move_ip`, `r`, `r_sqr`, `radius`, `right`, `rotate`, `rotate_ip`, `top`, `update`, `x`, `y`

## Eine Linie (`pygame.geometry.Line`)

16 von 16 (100 %)

**Da:** `a`, `ax`, `ay`, `b`, `bx`, `by`, `copy`, `flip_ab`, `flip_ab_ip`, `length`, `move`, `move_ip`, `project`, `scale`, `scale_ip`, `update`

## Ein Pufferzeiger (`pygame.BufferProxy`)

4 von 4 (100 %)

**Da:** `length`, `parent`, `raw`, `write`

## Eine Zeigerform (`pygame.Cursor`)

1 von 1 (100 %)

**Da:** `copy`

## Ein Ereignis (`pygame.Event`)

2 von 2 (100 %)

**Da:** `dict`, `type`

## Ein Pixelfeld (`pygame.PixelArray`)

11 von 11 (100 %)

**Da:** `close`, `compare`, `extract`, `itemsize`, `make_surface`, `ndim`, `replace`, `shape`, `strides`, `surface`, `transpose`

## Die Gruppenbasis (`pygame.sprite.AbstractGroup`)

12 von 12 (100 %)

**Da:** `add`, `add_internal`, `clear`, `copy`, `draw`, `empty`, `has`, `has_internal`, `remove`, `remove_internal`, `sprites`, `update`

## Eine schmutzige Figur (`pygame.sprite.DirtySprite`)

12 von 12 (100 %)

**Da:** `add`, `add_internal`, `alive`, `groups`, `image`, `kill`, `layer`, `rect`, `remove`, `remove_internal`, `update`, `visible`

## Eine Einzelgruppe (`pygame.sprite.GroupSingle`)

13 von 13 (100 %)

**Da:** `add`, `add_internal`, `clear`, `copy`, `draw`, `empty`, `has`, `has_internal`, `remove`, `remove_internal`, `sprite`, `sprites`, `update`

## Gruppe mit Ebenen (`pygame.sprite.LayeredUpdates`)

25 von 25 (100 %)

**Da:** `add`, `add_internal`, `change_layer`, `clear`, `copy`, `draw`, `empty`, `get_bottom_layer`, `get_layer_of_sprite`, `get_sprite`, `get_sprites_at`, `get_sprites_from_layer`, `get_top_layer`, `get_top_sprite`, `has`, `has_internal`, `layers`, `move_to_back`, `move_to_front`, `remove`, `remove_internal`, `remove_sprites_of_layer`, `sprites`, `switch_layer`, `update`

## Ebenen und schmutzige Rechtecke (`pygame.sprite.LayeredDirty`)

30 von 30 (100 %)

**Da:** `add`, `add_internal`, `change_layer`, `clear`, `copy`, `draw`, `empty`, `get_bottom_layer`, `get_clip`, `get_layer_of_sprite`, `get_sprite`, `get_sprites_at`, `get_sprites_from_layer`, `get_top_layer`, `get_top_sprite`, `has`, `has_internal`, `layers`, `move_to_back`, `move_to_front`, `remove`, `remove_internal`, `remove_sprites_of_layer`, `repaint_rect`, `set_clip`, `set_timing_threshold`, `set_timing_treshold`, `sprites`, `switch_layer`, `update`

## Gruppe in Reihenfolge (`pygame.sprite.OrderedUpdates`)

12 von 12 (100 %)

**Da:** `add`, `add_internal`, `clear`, `copy`, `draw`, `empty`, `has`, `has_internal`, `remove`, `remove_internal`, `sprites`, `update`

## Gruppe mit Rechtecken (`pygame.sprite.RenderUpdates`)

12 von 12 (100 %)

**Da:** `add`, `add_internal`, `clear`, `copy`, `draw`, `empty`, `has`, `has_internal`, `remove`, `remove_internal`, `sprites`, `update`

## Zweitname von Group (`pygame.sprite.RenderPlain`)

12 von 12 (100 %)

**Da:** `add`, `add_internal`, `clear`, `copy`, `draw`, `empty`, `has`, `has_internal`, `remove`, `remove_internal`, `sprites`, `update`

## Zweitname von Group (`pygame.sprite.RenderClear`)

12 von 12 (100 %)

**Da:** `add`, `add_internal`, `clear`, `copy`, `draw`, `empty`, `has`, `has_internal`, `remove`, `remove_internal`, `sprites`, `update`

## MIDI-Eingang (`pygame.midi.Input`)

3 von 3 (100 %)

**Da:** `close`, `poll`, `read`

## MIDI-Ausgang (`pygame.midi.Output`)

9 von 9 (100 %)

**Da:** `abort`, `close`, `note_off`, `note_on`, `pitch_bend`, `set_instrument`, `write`, `write_short`, `write_sys_ex`

## Die Fassung (`pygame.version.PygameVersion`)

6 von 6 (100 %)

**Da:** `count`, `fields`, `index`, `major`, `minor`, `patch`

## SDLs Fassung (`pygame.version.SDLVersion`)

6 von 6 (100 %)

**Da:** `count`, `fields`, `index`, `major`, `minor`, `patch`

## Ein Zeichner (`pygame._sdl2.video.Renderer`)

23 von 23 (100 %)

**Da:** `blit`, `clear`, `compose_custom_blend_mode`, `coordinates_from_window`, `coordinates_to_window`, `draw_blend_mode`, `draw_color`, `draw_line`, `draw_point`, `draw_quad`, `draw_rect`, `draw_triangle`, `fill_quad`, `fill_rect`, `fill_triangle`, `from_window`, `get_viewport`, `logical_size`, `present`, `scale`, `set_viewport`, `target`, `to_surface`

## Eine Textur (`pygame._sdl2.video.Texture`)

12 von 12 (100 %)

**Da:** `alpha`, `blend_mode`, `color`, `draw`, `draw_quad`, `draw_triangle`, `from_surface`, `get_rect`, `height`, `renderer`, `update`, `width`

## Ein Bildausschnitt (`pygame._sdl2.video.Image`)

11 von 11 (100 %)

**Da:** `alpha`, `angle`, `blend_mode`, `color`, `draw`, `flip_x`, `flip_y`, `get_rect`, `origin`, `srcrect`, `texture`

## Untermodul (`pygame.__pyinstaller`)

2 von 2 (100 %)

**Da:** `get_hook_dirs`, `os`

## Untermodul (`pygame._camera`)

3 von 3 (100 %)

**Da:** `Camera`, `colorspace`, `list_cameras`

## Untermodul (`pygame._data_classes`)

2 von 2 (100 %)

**Da:** `PowerState`, `dataclass`

## Untermodul (`pygame._debug`)

10 von 10 (100 %)

**Da:** `attempt_import`, `environ`, `get_cpu_instruction_sets`, `importlib`, `platform`, `print_debug_info`, `str_from_tuple`, `sys`, `traceback`, `ver`

## Untermodul (`pygame._freetype`)

21 von 21 (100 %)

**Da:** `BBOX_EXACT`, `BBOX_EXACT_GRIDFIT`, `BBOX_PIXEL`, `BBOX_PIXEL_GRIDFIT`, `Font`, `STYLE_DEFAULT`, `STYLE_NORMAL`, `STYLE_OBLIQUE`, `STYLE_STRONG`, `STYLE_UNDERLINE`, `STYLE_WIDE`, `get_cache_size`, `get_default_font`, `get_default_resolution`, `get_error`, `get_init`, `get_version`, `init`, `quit`, `set_default_resolution`, `was_init`

## Untermodul (`pygame._sdl2`)

53 von 53 (100 %)

**Da:** `AUDIO_ALLOW_ANY_CHANGE`, `AUDIO_ALLOW_CHANNELS_CHANGE`, `AUDIO_ALLOW_FORMAT_CHANGE`, `AUDIO_ALLOW_FREQUENCY_CHANGE`, `AUDIO_F32`, `AUDIO_F32LSB`, `AUDIO_F32MSB`, `AUDIO_S16`, `AUDIO_S16LSB`, `AUDIO_S16MSB`, `AUDIO_S32`, `AUDIO_S32LSB`, `AUDIO_S32MSB`, `AUDIO_S8`, `AUDIO_U16`, `AUDIO_U16LSB`, `AUDIO_U16MSB`, `AUDIO_U8`, `AudioDevice`, `INIT_AUDIO`, `INIT_EVENTS`, `INIT_EVERYTHING`, `INIT_GAMECONTROLLER`, `INIT_HAPTIC`, `INIT_JOYSTICK`, `INIT_NOPARACHUTE`, `INIT_TIMER`, `INIT_VIDEO`, `Image`, `Renderer`, `RendererDriverInfo`, `SCALEQUALITY_BEST`, `SCALEQUALITY_LINEAR`, `SCALEQUALITY_NEAREST`, `Texture`, `WINDOWPOS_CENTERED`, `WINDOWPOS_UNDEFINED`, `Window`, `audio`, `controller`, `controller_old`, `error`, `errorfnc`, `get_audio_device_names`, `get_drivers`, `get_grabbed_window`, `init_subsystem`, `mixer`, `sdl2`, `touch`, `traceback`, `video`, `window`

## Untermodul (`pygame._sprite`)

19 von 19 (100 %)

**Da:** `AbstractGroup`, `DirtySprite`, `Group`, `GroupSingle`, `LayeredDirty`, `LayeredUpdates`, `RenderClear`, `RenderPlain`, `RenderUpdates`, `Sprite`, `collide_circle`, `collide_circle_ratio`, `collide_mask`, `collide_rect`, `collide_rect_ratio`, `groupcollide`, `spritecollide`, `spritecollideany`, `warnings`

## Untermodul (`pygame.color`)

2 von 2 (100 %)

**Da:** `Color`, `THECOLORS`

## Untermodul (`pygame.constants`)

594 von 594 (100 %)

**Da:** `ACTIVEEVENT`, `ANYFORMAT`, `APPACTIVE`, `APPINPUTFOCUS`, `APPMOUSEFOCUS`, `APP_DIDENTERBACKGROUND`, `APP_DIDENTERFOREGROUND`, `APP_LOWMEMORY`, `APP_TERMINATING`, `APP_WILLENTERBACKGROUND`, `APP_WILLENTERFOREGROUND`, `ASYNCBLIT`, `AUDIODEVICEADDED`, `AUDIODEVICEREMOVED`, `AUDIO_ALLOW_ANY_CHANGE`, `AUDIO_ALLOW_CHANNELS_CHANGE`, `AUDIO_ALLOW_FORMAT_CHANGE`, `AUDIO_ALLOW_FREQUENCY_CHANGE`, `AUDIO_S16`, `AUDIO_S16LSB`, `AUDIO_S16MSB`, `AUDIO_S16SYS`, `AUDIO_S8`, `AUDIO_U16`, `AUDIO_U16LSB`, `AUDIO_U16MSB`, `AUDIO_U16SYS`, `AUDIO_U8`, `BIG_ENDIAN`, `BLENDFACTOR_DST_ALPHA`, `BLENDFACTOR_DST_COLOR`, `BLENDFACTOR_ONE`, `BLENDFACTOR_ONE_MINUS_DST_ALPHA`, `BLENDFACTOR_ONE_MINUS_DST_COLOR`, `BLENDFACTOR_ONE_MINUS_SRC_ALPHA`, `BLENDFACTOR_ONE_MINUS_SRC_COLOR`, `BLENDFACTOR_SRC_ALPHA`, `BLENDFACTOR_SRC_COLOR`, `BLENDFACTOR_ZERO`, `BLENDMODE_ADD`, `BLENDMODE_BLEND`, `BLENDMODE_MOD`, `BLENDMODE_MUL`, `BLENDMODE_NONE`, `BLENDOPERATION_ADD`, `BLENDOPERATION_MAXIMUM`, `BLENDOPERATION_MINIMUM`, `BLENDOPERATION_REV_SUBTRACT`, `BLENDOPERATION_SUBTRACT`, `BLEND_ADD`, `BLEND_ALPHA_SDL2`, `BLEND_MAX`, `BLEND_MIN`, `BLEND_MULT`, `BLEND_PREMULTIPLIED`, `BLEND_RGBA_ADD`, `BLEND_RGBA_MAX`, `BLEND_RGBA_MIN`, `BLEND_RGBA_MULT`, `BLEND_RGBA_SUB`, `BLEND_RGB_ADD`, `BLEND_RGB_MAX`, `BLEND_RGB_MIN`, `BLEND_RGB_MULT`, `BLEND_RGB_SUB`, `BLEND_SUB`, `BUTTON_LEFT`, `BUTTON_MIDDLE`, `BUTTON_RIGHT`, `BUTTON_WHEELDOWN`, `BUTTON_WHEELUP`, `BUTTON_X1`, `BUTTON_X2`, `CLIPBOARDUPDATE`, `CONTROLLERAXISMOTION`, `CONTROLLERBUTTONDOWN`, `CONTROLLERBUTTONUP`, `CONTROLLERDEVICEADDED`, `CONTROLLERDEVICEREMAPPED`, `CONTROLLERDEVICEREMOVED`, `CONTROLLERSENSORUPDATE`, `CONTROLLERTOUCHPADDOWN`, `CONTROLLERTOUCHPADMOTION`, `CONTROLLERTOUCHPADUP`, `CONTROLLER_AXIS_INVALID`, `CONTROLLER_AXIS_LEFTX`, `CONTROLLER_AXIS_LEFTY`, `CONTROLLER_AXIS_MAX`, `CONTROLLER_AXIS_RIGHTX`, `CONTROLLER_AXIS_RIGHTY`, `CONTROLLER_AXIS_TRIGGERLEFT`, `CONTROLLER_AXIS_TRIGGERRIGHT`, `CONTROLLER_BUTTON_A`, `CONTROLLER_BUTTON_B`, `CONTROLLER_BUTTON_BACK`, `CONTROLLER_BUTTON_DPAD_DOWN`, `CONTROLLER_BUTTON_DPAD_LEFT`, `CONTROLLER_BUTTON_DPAD_RIGHT`, `CONTROLLER_BUTTON_DPAD_UP`, `CONTROLLER_BUTTON_GUIDE`, `CONTROLLER_BUTTON_INVALID`, `CONTROLLER_BUTTON_LEFTSHOULDER`, `CONTROLLER_BUTTON_LEFTSTICK`, `CONTROLLER_BUTTON_MAX`, `CONTROLLER_BUTTON_RIGHTSHOULDER`, `CONTROLLER_BUTTON_RIGHTSTICK`, `CONTROLLER_BUTTON_START`, `CONTROLLER_BUTTON_X`, `CONTROLLER_BUTTON_Y`, `DIRECTION_BTT`, `DIRECTION_LTR`, `DIRECTION_RTL`, `DIRECTION_TTB`, `DOUBLEBUF`, `DROPBEGIN`, `DROPCOMPLETE`, `DROPFILE`, `DROPTEXT`, `FINGERDOWN`, `FINGERMOTION`, `FINGERUP`, `FLASH_BRIEFLY`, `FLASH_CANCEL`, `FLASH_UNTIL_FOCUSED`, `FLIP_HORIZONTAL`, `FLIP_NONE`, `FLIP_VERTICAL`, `FONT_CENTER`, `FONT_LEFT`, `FONT_RIGHT`, `FULLSCREEN`, `GL_ACCELERATED_VISUAL`, `GL_ACCUM_ALPHA_SIZE`, `GL_ACCUM_BLUE_SIZE`, `GL_ACCUM_GREEN_SIZE`, `GL_ACCUM_RED_SIZE`, `GL_ALPHA_SIZE`, `GL_BLUE_SIZE`, `GL_BUFFER_SIZE`, `GL_CONTEXT_DEBUG_FLAG`, `GL_CONTEXT_FLAGS`, `GL_CONTEXT_FORWARD_COMPATIBLE_FLAG`, `GL_CONTEXT_MAJOR_VERSION`, `GL_CONTEXT_MINOR_VERSION`, `GL_CONTEXT_PROFILE_COMPATIBILITY`, `GL_CONTEXT_PROFILE_CORE`, `GL_CONTEXT_PROFILE_ES`, `GL_CONTEXT_PROFILE_MASK`, `GL_CONTEXT_RELEASE_BEHAVIOR`, `GL_CONTEXT_RELEASE_BEHAVIOR_FLUSH`, `GL_CONTEXT_RELEASE_BEHAVIOR_NONE`, `GL_CONTEXT_RESET_ISOLATION_FLAG`, `GL_CONTEXT_ROBUST_ACCESS_FLAG`, `GL_DEPTH_SIZE`, `GL_DOUBLEBUFFER`, `GL_FRAMEBUFFER_SRGB_CAPABLE`, `GL_GREEN_SIZE`, `GL_MULTISAMPLEBUFFERS`, `GL_MULTISAMPLESAMPLES`, `GL_RED_SIZE`, `GL_SHARE_WITH_CURRENT_CONTEXT`, `GL_STENCIL_SIZE`, `GL_STEREO`, `GL_SWAP_CONTROL`, `HAT_CENTERED`, `HAT_DOWN`, `HAT_LEFT`, `HAT_LEFTDOWN`, `HAT_LEFTUP`, `HAT_RIGHT`, `HAT_RIGHTDOWN`, `HAT_RIGHTUP`, `HAT_UP`, `HIDDEN`, `HWACCEL`, `HWPALETTE`, `HWSURFACE`, `IS_CE`, `JOYAXISMOTION`, `JOYBALLMOTION`, `JOYBUTTONDOWN`, `JOYBUTTONUP`, `JOYDEVICEADDED`, `JOYDEVICEREMOVED`, `JOYHATMOTION`, `KEYDOWN`, `KEYMAPCHANGED`, `KEYUP`, `KMOD_ALT`, `KMOD_CAPS`, `KMOD_CTRL`, `KMOD_GUI`, `KMOD_LALT`, `KMOD_LCTRL`, `KMOD_LGUI`, `KMOD_LMETA`, `KMOD_LSHIFT`, `KMOD_META`, `KMOD_MODE`, `KMOD_NONE`, `KMOD_NUM`, `KMOD_RALT`, `KMOD_RCTRL`, `KMOD_RGUI`, `KMOD_RMETA`, `KMOD_RSHIFT`, `KMOD_SHIFT`, `KSCAN_0`, `KSCAN_1`, `KSCAN_2`, `KSCAN_3`, `KSCAN_4`, `KSCAN_5`, `KSCAN_6`, `KSCAN_7`, `KSCAN_8`, `KSCAN_9`, `KSCAN_A`, `KSCAN_AC_BACK`, `KSCAN_APOSTROPHE`, `KSCAN_B`, `KSCAN_BACKSLASH`, `KSCAN_BACKSPACE`, `KSCAN_BREAK`, `KSCAN_C`, `KSCAN_CAPSLOCK`, `KSCAN_CLEAR`, `KSCAN_COMMA`, `KSCAN_CURRENCYSUBUNIT`, `KSCAN_CURRENCYUNIT`, `KSCAN_D`, `KSCAN_DELETE`, `KSCAN_DOWN`, `KSCAN_E`, `KSCAN_END`, `KSCAN_EQUALS`, `KSCAN_ESCAPE`, `KSCAN_EURO`, `KSCAN_F`, `KSCAN_F1`, `KSCAN_F10`, `KSCAN_F11`, `KSCAN_F12`, `KSCAN_F13`, `KSCAN_F14`, `KSCAN_F15`, `KSCAN_F2`, `KSCAN_F3`, `KSCAN_F4`, `KSCAN_F5`, `KSCAN_F6`, `KSCAN_F7`, `KSCAN_F8`, `KSCAN_F9`, `KSCAN_G`, `KSCAN_GRAVE`, `KSCAN_H`, `KSCAN_HELP`, `KSCAN_HOME`, `KSCAN_I`, `KSCAN_INSERT`, `KSCAN_INTERNATIONAL1`, `KSCAN_INTERNATIONAL2`, `KSCAN_INTERNATIONAL3`, `KSCAN_INTERNATIONAL4`, `KSCAN_INTERNATIONAL5`, `KSCAN_INTERNATIONAL6`, `KSCAN_INTERNATIONAL7`, `KSCAN_INTERNATIONAL8`, `KSCAN_INTERNATIONAL9`, `KSCAN_J`, `KSCAN_K`, `KSCAN_KP0`, `KSCAN_KP1`, `KSCAN_KP2`, `KSCAN_KP3`, `KSCAN_KP4`, `KSCAN_KP5`, `KSCAN_KP6`, `KSCAN_KP7`, `KSCAN_KP8`, `KSCAN_KP9`, `KSCAN_KP_0`, `KSCAN_KP_1`, `KSCAN_KP_2`, `KSCAN_KP_3`, `KSCAN_KP_4`, `KSCAN_KP_5`, `KSCAN_KP_6`, `KSCAN_KP_7`, `KSCAN_KP_8`, `KSCAN_KP_9`, `KSCAN_KP_DIVIDE`, `KSCAN_KP_ENTER`, `KSCAN_KP_EQUALS`, `KSCAN_KP_MINUS`, `KSCAN_KP_MULTIPLY`, `KSCAN_KP_PERIOD`, `KSCAN_KP_PLUS`, `KSCAN_L`, `KSCAN_LALT`, `KSCAN_LANG1`, `KSCAN_LANG2`, `KSCAN_LANG3`, `KSCAN_LANG4`, `KSCAN_LANG5`, `KSCAN_LANG6`, `KSCAN_LANG7`, `KSCAN_LANG8`, `KSCAN_LANG9`, `KSCAN_LCTRL`, `KSCAN_LEFT`, `KSCAN_LEFTBRACKET`, `KSCAN_LGUI`, `KSCAN_LMETA`, `KSCAN_LSHIFT`, `KSCAN_LSUPER`, `KSCAN_M`, `KSCAN_MENU`, `KSCAN_MINUS`, `KSCAN_MODE`, `KSCAN_N`, `KSCAN_NONUSBACKSLASH`, `KSCAN_NONUSHASH`, `KSCAN_NUMLOCK`, `KSCAN_NUMLOCKCLEAR`, `KSCAN_O`, `KSCAN_P`, `KSCAN_PAGEDOWN`, `KSCAN_PAGEUP`, `KSCAN_PAUSE`, `KSCAN_PERIOD`, `KSCAN_POWER`, `KSCAN_PRINT`, `KSCAN_PRINTSCREEN`, `KSCAN_Q`, `KSCAN_R`, `KSCAN_RALT`, `KSCAN_RCTRL`, `KSCAN_RETURN`, `KSCAN_RGUI`, `KSCAN_RIGHT`, `KSCAN_RIGHTBRACKET`, `KSCAN_RMETA`, `KSCAN_RSHIFT`, `KSCAN_RSUPER`, `KSCAN_S`, `KSCAN_SCROLLLOCK`, `KSCAN_SCROLLOCK`, `KSCAN_SEMICOLON`, `KSCAN_SLASH`, `KSCAN_SPACE`, `KSCAN_SYSREQ`, `KSCAN_T`, `KSCAN_TAB`, `KSCAN_U`, `KSCAN_UNKNOWN`, `KSCAN_UP`, `KSCAN_V`, `KSCAN_W`, `KSCAN_X`, `KSCAN_Y`, `KSCAN_Z`, `K_0`, `K_1`, `K_2`, `K_3`, `K_4`, `K_5`, `K_6`, `K_7`, `K_8`, `K_9`, `K_AC_BACK`, `K_AMPERSAND`, `K_ASTERISK`, `K_AT`, `K_BACKQUOTE`, `K_BACKSLASH`, `K_BACKSPACE`, `K_BREAK`, `K_CAPSLOCK`, `K_CARET`, `K_CLEAR`, `K_COLON`, `K_COMMA`, `K_CURRENCYSUBUNIT`, `K_CURRENCYUNIT`, `K_DELETE`, `K_DOLLAR`, `K_DOWN`, `K_END`, `K_EQUALS`, `K_ESCAPE`, `K_EURO`, `K_EXCLAIM`, `K_F1`, `K_F10`, `K_F11`, `K_F12`, `K_F13`, `K_F14`, `K_F15`, `K_F2`, `K_F3`, `K_F4`, `K_F5`, `K_F6`, `K_F7`, `K_F8`, `K_F9`, `K_GREATER`, `K_HASH`, `K_HELP`, `K_HOME`, `K_INSERT`, `K_KP0`, `K_KP1`, `K_KP2`, `K_KP3`, `K_KP4`, `K_KP5`, `K_KP6`, `K_KP7`, `K_KP8`, `K_KP9`, `K_KP_0`, `K_KP_1`, `K_KP_2`, `K_KP_3`, `K_KP_4`, `K_KP_5`, `K_KP_6`, `K_KP_7`, `K_KP_8`, `K_KP_9`, `K_KP_DIVIDE`, `K_KP_ENTER`, `K_KP_EQUALS`, `K_KP_MINUS`, `K_KP_MULTIPLY`, `K_KP_PERIOD`, `K_KP_PLUS`, `K_LALT`, `K_LCTRL`, `K_LEFT`, `K_LEFTBRACKET`, `K_LEFTPAREN`, `K_LESS`, `K_LGUI`, `K_LMETA`, `K_LSHIFT`, `K_LSUPER`, `K_MENU`, `K_MINUS`, `K_MODE`, `K_NUMLOCK`, `K_NUMLOCKCLEAR`, `K_PAGEDOWN`, `K_PAGEUP`, `K_PAUSE`, `K_PERCENT`, `K_PERIOD`, `K_PLUS`, `K_POWER`, `K_PRINT`, `K_PRINTSCREEN`, `K_QUESTION`, `K_QUOTE`, `K_QUOTEDBL`, `K_RALT`, `K_RCTRL`, `K_RETURN`, `K_RGUI`, `K_RIGHT`, `K_RIGHTBRACKET`, `K_RIGHTPAREN`, `K_RMETA`, `K_RSHIFT`, `K_RSUPER`, `K_SCROLLLOCK`, `K_SCROLLOCK`, `K_SEMICOLON`, `K_SLASH`, `K_SPACE`, `K_SYSREQ`, `K_TAB`, `K_UNDERSCORE`, `K_UNKNOWN`, `K_UP`, `K_a`, `K_b`, `K_c`, `K_d`, `K_e`, `K_f`, `K_g`, `K_h`, `K_i`, `K_j`, `K_k`, `K_l`, `K_m`, `K_n`, `K_o`, `K_p`, `K_q`, `K_r`, `K_s`, `K_t`, `K_u`, `K_v`, `K_w`, `K_x`, `K_y`, `K_z`, `LIL_ENDIAN`, `LOCALECHANGED`, `MIDIIN`, `MIDIOUT`, `MOUSEBUTTONDOWN`, `MOUSEBUTTONUP`, `MOUSEMOTION`, `MOUSEWHEEL`, `MULTIGESTURE`, `NOEVENT`, `NOFRAME`, `NULL_VIDEODRIVER`, `NUMEVENTS`, `OPENGL`, `OPENGLBLIT`, `PREALLOC`, `QUIT`, `RENDER_DEVICE_RESET`, `RENDER_TARGETS_RESET`, `RESIZABLE`, `RLEACCEL`, `RLEACCELOK`, `SCALED`, `SCRAP_BMP`, `SCRAP_CLIPBOARD`, `SCRAP_PBM`, `SCRAP_PPM`, `SCRAP_SELECTION`, `SCRAP_TEXT`, `SCROLL_ERASE`, `SCROLL_REPEAT`, `SHOWN`, `SRCALPHA`, `SRCCOLORKEY`, `SWSURFACE`, `SYSTEM_CURSOR_ARROW`, `SYSTEM_CURSOR_CROSSHAIR`, `SYSTEM_CURSOR_HAND`, `SYSTEM_CURSOR_IBEAM`, `SYSTEM_CURSOR_NO`, `SYSTEM_CURSOR_SIZEALL`, `SYSTEM_CURSOR_SIZENESW`, `SYSTEM_CURSOR_SIZENS`, `SYSTEM_CURSOR_SIZENWSE`, `SYSTEM_CURSOR_SIZEWE`, `SYSTEM_CURSOR_WAIT`, `SYSTEM_CURSOR_WAITARROW`, `SYSWMEVENT`, `TEXTEDITING`, `TEXTINPUT`, `TEXTUREACCESS_STATIC`, `TEXTUREACCESS_STREAMING`, `TEXTUREACCESS_TARGET`, `TIMER_RESOLUTION`, `USEREVENT`, `USEREVENT_DROPFILE`, `VIDEOEXPOSE`, `VIDEORESIZE`, `WINDOWCLOSE`, `WINDOWDISPLAYCHANGED`, `WINDOWENTER`, `WINDOWEXPOSED`, `WINDOWFOCUSGAINED`, `WINDOWFOCUSLOST`, `WINDOWHIDDEN`, `WINDOWHITTEST`, `WINDOWICCPROFCHANGED`, `WINDOWLEAVE`, `WINDOWMAXIMIZED`, `WINDOWMINIMIZED`, `WINDOWMOVED`, `WINDOWPOS_CENTERED`, `WINDOWPOS_UNDEFINED`, `WINDOWRESIZED`, `WINDOWRESTORED`, `WINDOWSHOWN`, `WINDOWSIZECHANGED`, `WINDOWTAKEFOCUS`

## Untermodul (`pygame.docs`)

3 von 3 (100 %)

**Da:** `PKG_DIR`, `Path`, `has_local_docs`

## Untermodul (`pygame.locals`)

597 von 597 (100 %)

**Da:** `ACTIVEEVENT`, `ANYFORMAT`, `APPACTIVE`, `APPINPUTFOCUS`, `APPMOUSEFOCUS`, `APP_DIDENTERBACKGROUND`, `APP_DIDENTERFOREGROUND`, `APP_LOWMEMORY`, `APP_TERMINATING`, `APP_WILLENTERBACKGROUND`, `APP_WILLENTERFOREGROUND`, `ASYNCBLIT`, `AUDIODEVICEADDED`, `AUDIODEVICEREMOVED`, `AUDIO_ALLOW_ANY_CHANGE`, `AUDIO_ALLOW_CHANNELS_CHANGE`, `AUDIO_ALLOW_FORMAT_CHANGE`, `AUDIO_ALLOW_FREQUENCY_CHANGE`, `AUDIO_S16`, `AUDIO_S16LSB`, `AUDIO_S16MSB`, `AUDIO_S16SYS`, `AUDIO_S8`, `AUDIO_U16`, `AUDIO_U16LSB`, `AUDIO_U16MSB`, `AUDIO_U16SYS`, `AUDIO_U8`, `BIG_ENDIAN`, `BLENDFACTOR_DST_ALPHA`, `BLENDFACTOR_DST_COLOR`, `BLENDFACTOR_ONE`, `BLENDFACTOR_ONE_MINUS_DST_ALPHA`, `BLENDFACTOR_ONE_MINUS_DST_COLOR`, `BLENDFACTOR_ONE_MINUS_SRC_ALPHA`, `BLENDFACTOR_ONE_MINUS_SRC_COLOR`, `BLENDFACTOR_SRC_ALPHA`, `BLENDFACTOR_SRC_COLOR`, `BLENDFACTOR_ZERO`, `BLENDMODE_ADD`, `BLENDMODE_BLEND`, `BLENDMODE_MOD`, `BLENDMODE_MUL`, `BLENDMODE_NONE`, `BLENDOPERATION_ADD`, `BLENDOPERATION_MAXIMUM`, `BLENDOPERATION_MINIMUM`, `BLENDOPERATION_REV_SUBTRACT`, `BLENDOPERATION_SUBTRACT`, `BLEND_ADD`, `BLEND_ALPHA_SDL2`, `BLEND_MAX`, `BLEND_MIN`, `BLEND_MULT`, `BLEND_PREMULTIPLIED`, `BLEND_RGBA_ADD`, `BLEND_RGBA_MAX`, `BLEND_RGBA_MIN`, `BLEND_RGBA_MULT`, `BLEND_RGBA_SUB`, `BLEND_RGB_ADD`, `BLEND_RGB_MAX`, `BLEND_RGB_MIN`, `BLEND_RGB_MULT`, `BLEND_RGB_SUB`, `BLEND_SUB`, `BUTTON_LEFT`, `BUTTON_MIDDLE`, `BUTTON_RIGHT`, `BUTTON_WHEELDOWN`, `BUTTON_WHEELUP`, `BUTTON_X1`, `BUTTON_X2`, `CLIPBOARDUPDATE`, `CONTROLLERAXISMOTION`, `CONTROLLERBUTTONDOWN`, `CONTROLLERBUTTONUP`, `CONTROLLERDEVICEADDED`, `CONTROLLERDEVICEREMAPPED`, `CONTROLLERDEVICEREMOVED`, `CONTROLLERSENSORUPDATE`, `CONTROLLERTOUCHPADDOWN`, `CONTROLLERTOUCHPADMOTION`, `CONTROLLERTOUCHPADUP`, `CONTROLLER_AXIS_INVALID`, `CONTROLLER_AXIS_LEFTX`, `CONTROLLER_AXIS_LEFTY`, `CONTROLLER_AXIS_MAX`, `CONTROLLER_AXIS_RIGHTX`, `CONTROLLER_AXIS_RIGHTY`, `CONTROLLER_AXIS_TRIGGERLEFT`, `CONTROLLER_AXIS_TRIGGERRIGHT`, `CONTROLLER_BUTTON_A`, `CONTROLLER_BUTTON_B`, `CONTROLLER_BUTTON_BACK`, `CONTROLLER_BUTTON_DPAD_DOWN`, `CONTROLLER_BUTTON_DPAD_LEFT`, `CONTROLLER_BUTTON_DPAD_RIGHT`, `CONTROLLER_BUTTON_DPAD_UP`, `CONTROLLER_BUTTON_GUIDE`, `CONTROLLER_BUTTON_INVALID`, `CONTROLLER_BUTTON_LEFTSHOULDER`, `CONTROLLER_BUTTON_LEFTSTICK`, `CONTROLLER_BUTTON_MAX`, `CONTROLLER_BUTTON_RIGHTSHOULDER`, `CONTROLLER_BUTTON_RIGHTSTICK`, `CONTROLLER_BUTTON_START`, `CONTROLLER_BUTTON_X`, `CONTROLLER_BUTTON_Y`, `Color`, `DIRECTION_BTT`, `DIRECTION_LTR`, `DIRECTION_RTL`, `DIRECTION_TTB`, `DOUBLEBUF`, `DROPBEGIN`, `DROPCOMPLETE`, `DROPFILE`, `DROPTEXT`, `FINGERDOWN`, `FINGERMOTION`, `FINGERUP`, `FLASH_BRIEFLY`, `FLASH_CANCEL`, `FLASH_UNTIL_FOCUSED`, `FLIP_HORIZONTAL`, `FLIP_NONE`, `FLIP_VERTICAL`, `FONT_CENTER`, `FONT_LEFT`, `FONT_RIGHT`, `FULLSCREEN`, `GL_ACCELERATED_VISUAL`, `GL_ACCUM_ALPHA_SIZE`, `GL_ACCUM_BLUE_SIZE`, `GL_ACCUM_GREEN_SIZE`, `GL_ACCUM_RED_SIZE`, `GL_ALPHA_SIZE`, `GL_BLUE_SIZE`, `GL_BUFFER_SIZE`, `GL_CONTEXT_DEBUG_FLAG`, `GL_CONTEXT_FLAGS`, `GL_CONTEXT_FORWARD_COMPATIBLE_FLAG`, `GL_CONTEXT_MAJOR_VERSION`, `GL_CONTEXT_MINOR_VERSION`, `GL_CONTEXT_PROFILE_COMPATIBILITY`, `GL_CONTEXT_PROFILE_CORE`, `GL_CONTEXT_PROFILE_ES`, `GL_CONTEXT_PROFILE_MASK`, `GL_CONTEXT_RELEASE_BEHAVIOR`, `GL_CONTEXT_RELEASE_BEHAVIOR_FLUSH`, `GL_CONTEXT_RELEASE_BEHAVIOR_NONE`, `GL_CONTEXT_RESET_ISOLATION_FLAG`, `GL_CONTEXT_ROBUST_ACCESS_FLAG`, `GL_DEPTH_SIZE`, `GL_DOUBLEBUFFER`, `GL_FRAMEBUFFER_SRGB_CAPABLE`, `GL_GREEN_SIZE`, `GL_MULTISAMPLEBUFFERS`, `GL_MULTISAMPLESAMPLES`, `GL_RED_SIZE`, `GL_SHARE_WITH_CURRENT_CONTEXT`, `GL_STENCIL_SIZE`, `GL_STEREO`, `GL_SWAP_CONTROL`, `HAT_CENTERED`, `HAT_DOWN`, `HAT_LEFT`, `HAT_LEFTDOWN`, `HAT_LEFTUP`, `HAT_RIGHT`, `HAT_RIGHTDOWN`, `HAT_RIGHTUP`, `HAT_UP`, `HIDDEN`, `HWACCEL`, `HWPALETTE`, `HWSURFACE`, `IS_CE`, `JOYAXISMOTION`, `JOYBALLMOTION`, `JOYBUTTONDOWN`, `JOYBUTTONUP`, `JOYDEVICEADDED`, `JOYDEVICEREMOVED`, `JOYHATMOTION`, `KEYDOWN`, `KEYMAPCHANGED`, `KEYUP`, `KMOD_ALT`, `KMOD_CAPS`, `KMOD_CTRL`, `KMOD_GUI`, `KMOD_LALT`, `KMOD_LCTRL`, `KMOD_LGUI`, `KMOD_LMETA`, `KMOD_LSHIFT`, `KMOD_META`, `KMOD_MODE`, `KMOD_NONE`, `KMOD_NUM`, `KMOD_RALT`, `KMOD_RCTRL`, `KMOD_RGUI`, `KMOD_RMETA`, `KMOD_RSHIFT`, `KMOD_SHIFT`, `KSCAN_0`, `KSCAN_1`, `KSCAN_2`, `KSCAN_3`, `KSCAN_4`, `KSCAN_5`, `KSCAN_6`, `KSCAN_7`, `KSCAN_8`, `KSCAN_9`, `KSCAN_A`, `KSCAN_AC_BACK`, `KSCAN_APOSTROPHE`, `KSCAN_B`, `KSCAN_BACKSLASH`, `KSCAN_BACKSPACE`, `KSCAN_BREAK`, `KSCAN_C`, `KSCAN_CAPSLOCK`, `KSCAN_CLEAR`, `KSCAN_COMMA`, `KSCAN_CURRENCYSUBUNIT`, `KSCAN_CURRENCYUNIT`, `KSCAN_D`, `KSCAN_DELETE`, `KSCAN_DOWN`, `KSCAN_E`, `KSCAN_END`, `KSCAN_EQUALS`, `KSCAN_ESCAPE`, `KSCAN_EURO`, `KSCAN_F`, `KSCAN_F1`, `KSCAN_F10`, `KSCAN_F11`, `KSCAN_F12`, `KSCAN_F13`, `KSCAN_F14`, `KSCAN_F15`, `KSCAN_F2`, `KSCAN_F3`, `KSCAN_F4`, `KSCAN_F5`, `KSCAN_F6`, `KSCAN_F7`, `KSCAN_F8`, `KSCAN_F9`, `KSCAN_G`, `KSCAN_GRAVE`, `KSCAN_H`, `KSCAN_HELP`, `KSCAN_HOME`, `KSCAN_I`, `KSCAN_INSERT`, `KSCAN_INTERNATIONAL1`, `KSCAN_INTERNATIONAL2`, `KSCAN_INTERNATIONAL3`, `KSCAN_INTERNATIONAL4`, `KSCAN_INTERNATIONAL5`, `KSCAN_INTERNATIONAL6`, `KSCAN_INTERNATIONAL7`, `KSCAN_INTERNATIONAL8`, `KSCAN_INTERNATIONAL9`, `KSCAN_J`, `KSCAN_K`, `KSCAN_KP0`, `KSCAN_KP1`, `KSCAN_KP2`, `KSCAN_KP3`, `KSCAN_KP4`, `KSCAN_KP5`, `KSCAN_KP6`, `KSCAN_KP7`, `KSCAN_KP8`, `KSCAN_KP9`, `KSCAN_KP_0`, `KSCAN_KP_1`, `KSCAN_KP_2`, `KSCAN_KP_3`, `KSCAN_KP_4`, `KSCAN_KP_5`, `KSCAN_KP_6`, `KSCAN_KP_7`, `KSCAN_KP_8`, `KSCAN_KP_9`, `KSCAN_KP_DIVIDE`, `KSCAN_KP_ENTER`, `KSCAN_KP_EQUALS`, `KSCAN_KP_MINUS`, `KSCAN_KP_MULTIPLY`, `KSCAN_KP_PERIOD`, `KSCAN_KP_PLUS`, `KSCAN_L`, `KSCAN_LALT`, `KSCAN_LANG1`, `KSCAN_LANG2`, `KSCAN_LANG3`, `KSCAN_LANG4`, `KSCAN_LANG5`, `KSCAN_LANG6`, `KSCAN_LANG7`, `KSCAN_LANG8`, `KSCAN_LANG9`, `KSCAN_LCTRL`, `KSCAN_LEFT`, `KSCAN_LEFTBRACKET`, `KSCAN_LGUI`, `KSCAN_LMETA`, `KSCAN_LSHIFT`, `KSCAN_LSUPER`, `KSCAN_M`, `KSCAN_MENU`, `KSCAN_MINUS`, `KSCAN_MODE`, `KSCAN_N`, `KSCAN_NONUSBACKSLASH`, `KSCAN_NONUSHASH`, `KSCAN_NUMLOCK`, `KSCAN_NUMLOCKCLEAR`, `KSCAN_O`, `KSCAN_P`, `KSCAN_PAGEDOWN`, `KSCAN_PAGEUP`, `KSCAN_PAUSE`, `KSCAN_PERIOD`, `KSCAN_POWER`, `KSCAN_PRINT`, `KSCAN_PRINTSCREEN`, `KSCAN_Q`, `KSCAN_R`, `KSCAN_RALT`, `KSCAN_RCTRL`, `KSCAN_RETURN`, `KSCAN_RGUI`, `KSCAN_RIGHT`, `KSCAN_RIGHTBRACKET`, `KSCAN_RMETA`, `KSCAN_RSHIFT`, `KSCAN_RSUPER`, `KSCAN_S`, `KSCAN_SCROLLLOCK`, `KSCAN_SCROLLOCK`, `KSCAN_SEMICOLON`, `KSCAN_SLASH`, `KSCAN_SPACE`, `KSCAN_SYSREQ`, `KSCAN_T`, `KSCAN_TAB`, `KSCAN_U`, `KSCAN_UNKNOWN`, `KSCAN_UP`, `KSCAN_V`, `KSCAN_W`, `KSCAN_X`, `KSCAN_Y`, `KSCAN_Z`, `K_0`, `K_1`, `K_2`, `K_3`, `K_4`, `K_5`, `K_6`, `K_7`, `K_8`, `K_9`, `K_AC_BACK`, `K_AMPERSAND`, `K_ASTERISK`, `K_AT`, `K_BACKQUOTE`, `K_BACKSLASH`, `K_BACKSPACE`, `K_BREAK`, `K_CAPSLOCK`, `K_CARET`, `K_CLEAR`, `K_COLON`, `K_COMMA`, `K_CURRENCYSUBUNIT`, `K_CURRENCYUNIT`, `K_DELETE`, `K_DOLLAR`, `K_DOWN`, `K_END`, `K_EQUALS`, `K_ESCAPE`, `K_EURO`, `K_EXCLAIM`, `K_F1`, `K_F10`, `K_F11`, `K_F12`, `K_F13`, `K_F14`, `K_F15`, `K_F2`, `K_F3`, `K_F4`, `K_F5`, `K_F6`, `K_F7`, `K_F8`, `K_F9`, `K_GREATER`, `K_HASH`, `K_HELP`, `K_HOME`, `K_INSERT`, `K_KP0`, `K_KP1`, `K_KP2`, `K_KP3`, `K_KP4`, `K_KP5`, `K_KP6`, `K_KP7`, `K_KP8`, `K_KP9`, `K_KP_0`, `K_KP_1`, `K_KP_2`, `K_KP_3`, `K_KP_4`, `K_KP_5`, `K_KP_6`, `K_KP_7`, `K_KP_8`, `K_KP_9`, `K_KP_DIVIDE`, `K_KP_ENTER`, `K_KP_EQUALS`, `K_KP_MINUS`, `K_KP_MULTIPLY`, `K_KP_PERIOD`, `K_KP_PLUS`, `K_LALT`, `K_LCTRL`, `K_LEFT`, `K_LEFTBRACKET`, `K_LEFTPAREN`, `K_LESS`, `K_LGUI`, `K_LMETA`, `K_LSHIFT`, `K_LSUPER`, `K_MENU`, `K_MINUS`, `K_MODE`, `K_NUMLOCK`, `K_NUMLOCKCLEAR`, `K_PAGEDOWN`, `K_PAGEUP`, `K_PAUSE`, `K_PERCENT`, `K_PERIOD`, `K_PLUS`, `K_POWER`, `K_PRINT`, `K_PRINTSCREEN`, `K_QUESTION`, `K_QUOTE`, `K_QUOTEDBL`, `K_RALT`, `K_RCTRL`, `K_RETURN`, `K_RGUI`, `K_RIGHT`, `K_RIGHTBRACKET`, `K_RIGHTPAREN`, `K_RMETA`, `K_RSHIFT`, `K_RSUPER`, `K_SCROLLLOCK`, `K_SCROLLOCK`, `K_SEMICOLON`, `K_SLASH`, `K_SPACE`, `K_SYSREQ`, `K_TAB`, `K_UNDERSCORE`, `K_UNKNOWN`, `K_UP`, `K_a`, `K_b`, `K_c`, `K_d`, `K_e`, `K_f`, `K_g`, `K_h`, `K_i`, `K_j`, `K_k`, `K_l`, `K_m`, `K_n`, `K_o`, `K_p`, `K_q`, `K_r`, `K_s`, `K_t`, `K_u`, `K_v`, `K_w`, `K_x`, `K_y`, `K_z`, `LIL_ENDIAN`, `LOCALECHANGED`, `MIDIIN`, `MIDIOUT`, `MOUSEBUTTONDOWN`, `MOUSEBUTTONUP`, `MOUSEMOTION`, `MOUSEWHEEL`, `MULTIGESTURE`, `NOEVENT`, `NOFRAME`, `NULL_VIDEODRIVER`, `NUMEVENTS`, `OPENGL`, `OPENGLBLIT`, `PREALLOC`, `QUIT`, `RENDER_DEVICE_RESET`, `RENDER_TARGETS_RESET`, `RESIZABLE`, `RLEACCEL`, `RLEACCELOK`, `Rect`, `SCALED`, `SCRAP_BMP`, `SCRAP_CLIPBOARD`, `SCRAP_PBM`, `SCRAP_PPM`, `SCRAP_SELECTION`, `SCRAP_TEXT`, `SCROLL_ERASE`, `SCROLL_REPEAT`, `SHOWN`, `SRCALPHA`, `SRCCOLORKEY`, `SWSURFACE`, `SYSTEM_CURSOR_ARROW`, `SYSTEM_CURSOR_CROSSHAIR`, `SYSTEM_CURSOR_HAND`, `SYSTEM_CURSOR_IBEAM`, `SYSTEM_CURSOR_NO`, `SYSTEM_CURSOR_SIZEALL`, `SYSTEM_CURSOR_SIZENESW`, `SYSTEM_CURSOR_SIZENS`, `SYSTEM_CURSOR_SIZENWSE`, `SYSTEM_CURSOR_SIZEWE`, `SYSTEM_CURSOR_WAIT`, `SYSTEM_CURSOR_WAITARROW`, `SYSWMEVENT`, `TEXTEDITING`, `TEXTINPUT`, `TEXTUREACCESS_STATIC`, `TEXTUREACCESS_STREAMING`, `TEXTUREACCESS_TARGET`, `TIMER_RESOLUTION`, `USEREVENT`, `USEREVENT_DROPFILE`, `VIDEOEXPOSE`, `VIDEORESIZE`, `WINDOWCLOSE`, `WINDOWDISPLAYCHANGED`, `WINDOWENTER`, `WINDOWEXPOSED`, `WINDOWFOCUSGAINED`, `WINDOWFOCUSLOST`, `WINDOWHIDDEN`, `WINDOWHITTEST`, `WINDOWICCPROFCHANGED`, `WINDOWLEAVE`, `WINDOWMAXIMIZED`, `WINDOWMINIMIZED`, `WINDOWMOVED`, `WINDOWPOS_CENTERED`, `WINDOWPOS_UNDEFINED`, `WINDOWRESIZED`, `WINDOWRESTORED`, `WINDOWSHOWN`, `WINDOWSIZECHANGED`, `WINDOWTAKEFOCUS`, `pygame`

## Untermodul (`pygame.macosx`)

4 von 4 (100 %)

**Da:** `Video_AutoInit`, `os`, `platform`, `sys`

## Untermodul (`pygame.newbuffer`)

20 von 20 (100 %)

**Da:** `BufferMixin`, `PyBUFFER_SIZEOF`, `PyBUF_ANY_CONTIGUOUS`, `PyBUF_CONTIG`, `PyBUF_CONTIG_RO`, `PyBUF_C_CONTIGUOUS`, `PyBUF_FORMAT`, `PyBUF_FULL`, `PyBUF_FULL_RO`, `PyBUF_F_CONTIGUOUS`, `PyBUF_INDIRECT`, `PyBUF_ND`, `PyBUF_RECORDS`, `PyBUF_RECORDS_RO`, `PyBUF_SIMPLE`, `PyBUF_STRIDED`, `PyBUF_STRIDED_RO`, `PyBUF_STRIDES`, `PyBUF_WRITABLE`, `Py_buffer`

## Untermodul (`pygame.pkgdata`)

6 von 6 (100 %)

**Da:** `files`, `getResource`, `os`, `resource_exists`, `resource_stream`, `sys`

## Untermodul (`pygame.pypm`)

36 von 36 (100 %)

**Da:** `Channel`, `CountDevices`, `FALSE`, `FILT_ACTIVE`, `FILT_AFTERTOUCH`, `FILT_CHANNEL_AFTERTOUCH`, `FILT_CLOCK`, `FILT_CONTROL`, `FILT_F9`, `FILT_FD`, `FILT_MTC`, `FILT_NOTE`, `FILT_PITCHBEND`, `FILT_PLAY`, `FILT_POLY_AFTERTOUCH`, `FILT_PROGRAM`, `FILT_REALTIME`, `FILT_RESET`, `FILT_SONG_POSITION`, `FILT_SONG_SELECT`, `FILT_SYSEX`, `FILT_TICK`, `FILT_TUNE`, `FILT_UNDEFINED`, `GetDefaultInputDeviceID`, `GetDefaultOutputDeviceID`, `GetDeviceInfo`, `GetErrorText`, `Initialize`, `Input`, `Output`, `TRUE`, `Terminate`, `Time`, `array`, `sys`

## Untermodul (`pygame.rect`)

4 von 4 (100 %)

**Da:** `FRect`, `FRectType`, `Rect`, `RectType`

## Untermodul (`pygame.surface`)

2 von 2 (100 %)

**Da:** `Surface`, `SurfaceType`

## Untermodul (`pygame.typing`)

7 von 7 (100 %)

**Da:** `ColorLike`, `FileLike`, `IntPoint`, `Point`, `RectLike`, `SequenceLike`, `TypeAlias`

## Klasse (`pygame.BufferError`)

3 von 3 (100 %)

**Da:** `add_note`, `args`, `with_traceback`

## Klasse (`pygame._camera.Camera`)

8 von 8 (100 %)

**Da:** `get_controls`, `get_image`, `get_raw`, `get_size`, `query_image`, `set_controls`, `start`, `stop`

## Klasse (`pygame._render.Renderer`)

23 von 23 (100 %)

**Da:** `blit`, `clear`, `compose_custom_blend_mode`, `coordinates_from_window`, `coordinates_to_window`, `draw_blend_mode`, `draw_color`, `draw_line`, `draw_point`, `draw_quad`, `draw_rect`, `draw_triangle`, `fill_quad`, `fill_rect`, `fill_triangle`, `from_window`, `get_viewport`, `logical_size`, `present`, `scale`, `set_viewport`, `target`, `to_surface`

## Klasse (`pygame._render.Texture`)

12 von 12 (100 %)

**Da:** `alpha`, `blend_mode`, `color`, `draw`, `draw_quad`, `draw_triangle`, `from_surface`, `get_rect`, `height`, `renderer`, `update`, `width`

## Klasse (`pygame._sdl2.AudioDevice`)

10 von 10 (100 %)

**Da:** `audioformat`, `callback`, `chunksize`, `close`, `deviceid`, `devicename`, `frequency`, `iscapture`, `numchannels`, `pause`

## Klasse (`pygame._sdl2.controller.Controller`)

15 von 15 (100 %)

**Da:** `as_joystick`, `attached`, `from_joystick`, `get_axis`, `get_button`, `get_init`, `get_mapping`, `id`, `init`, `name`, `quit`, `rumble`, `set_led`, `set_mapping`, `stop_rumble`

## Klasse (`pygame._sdl2.controller_old.Controller`)

14 von 14 (100 %)

**Da:** `as_joystick`, `attached`, `from_joystick`, `get_axis`, `get_button`, `get_init`, `get_mapping`, `id`, `init`, `name`, `quit`, `rumble`, `set_mapping`, `stop_rumble`

## Klasse (`pygame._sdl2.error`)

3 von 3 (100 %)

**Da:** `add_note`, `args`, `with_traceback`

## Klasse (`pygame.camera.AbstractCamera`)

6 von 6 (100 %)

**Da:** `get_image`, `get_raw`, `get_size`, `query_image`, `start`, `stop`

## Klasse (`pygame.camera.Camera`)

8 von 8 (100 %)

**Da:** `get_controls`, `get_image`, `get_raw`, `get_size`, `query_image`, `set_controls`, `start`, `stop`

## Klasse (`pygame.error`)

3 von 3 (100 %)

**Da:** `add_note`, `args`, `with_traceback`

## Klasse (`pygame.ftfont.Font`)

54 von 54 (100 %)

**Da:** `antialiased`, `ascender`, `bgcolor`, `bold`, `descender`, `fgcolor`, `fixed_sizes`, `fixed_width`, `get_ascent`, `get_bold`, `get_descent`, `get_height`, `get_italic`, `get_linesize`, `get_metrics`, `get_rect`, `get_sized_ascender`, `get_sized_descender`, `get_sized_glyph_height`, `get_sized_height`, `get_sizes`, `get_strikethrough`, `get_underline`, `height`, `italic`, `kerning`, `metrics`, `name`, `oblique`, `origin`, `pad`, `path`, `render`, `render_raw`, `render_raw_to`, `render_to`, `resolution`, `rotation`, `scalable`, `set_bold`, `set_italic`, `set_strikethrough`, `set_underline`, `size`, `strength`, `strong`, `style`, `style_name`, `ucs4`, `underline`, `underline_adjustment`, `use_bitmap_strikes`, `vertical`, `wide`

## Klasse (`pygame.joystick.JoystickType`)

19 von 19 (100 %)

**Da:** `get_axis`, `get_ball`, `get_button`, `get_guid`, `get_hat`, `get_id`, `get_init`, `get_instance_id`, `get_name`, `get_numaxes`, `get_numballs`, `get_numbuttons`, `get_numhats`, `get_power_level`, `init`, `quit`, `rumble`, `set_led`, `stop_rumble`

## Klasse (`pygame.key.ScancodeWrapper`)

2 von 2 (100 %)

**Da:** `count`, `index`

## Klasse (`pygame.midi.MidiException`)

3 von 3 (100 %)

**Da:** `add_note`, `args`, `with_traceback`

## Klasse (`pygame.newbuffer.Py_buffer`)

13 von 13 (100 %)

**Da:** `buf`, `format`, `get_buffer`, `internal`, `itemsize`, `len`, `ndim`, `obj`, `readonly`, `release_buffer`, `shape`, `strides`, `suboffsets`

## Klasse (`pygame.pypm.Input`)

5 von 5 (100 %)

**Da:** `Close`, `Poll`, `Read`, `SetChannelMask`, `SetFilter`

## Klasse (`pygame.pypm.Output`)

5 von 5 (100 %)

**Da:** `Abort`, `Close`, `Write`, `WriteShort`, `WriteSysEx`

## Klasse (`pygame.version.SoftwareVersion`)

6 von 6 (100 %)

**Da:** `count`, `fields`, `index`, `major`, `minor`, `patch`
