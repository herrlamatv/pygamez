"""Der Gruss beim Laden - das Ende von pygames `__init__.py`.

pygame schreibt beim Import eine Zeile, solange `PYGAME_HIDE_SUPPORT_PROMPT`
nicht in `os.environ` steht:

    pygame-ce 2.5.7 (SDL 2.32.10, Python 3.14.4)

Jeder Messlauf dieses Projekts setzt die Variable, und darum fehlte die
Zeile zwanzig Runden lang, ohne dass ein Werkzeug sie vermisste. Gefunden
beim ersten Programm, das ueber `tools/starten.py` mit der Umgebung des
Nutzers lief (Runde 21).

`ver` und `get_sdl_version` reicht `lib.rs` herein. Der Zweig fuer ein
Python ohne GIL ist mit uebertragen.
"""
import os
import platform
import sys

if "PYGAME_HIDE_SUPPORT_PROMPT" not in os.environ:
    import sysconfig

    python_version = platform.python_version()

    if (
        sys.platform not in ("wasi", "emscripten")
        and (sys.version_info >= (3, 13, 0))
        and sysconfig.get_config_var("Py_GIL_DISABLED")
    ):
        python_version += f"t, {'' if sys._is_gil_enabled() else 'No '}GIL"

    print(
        f"pygame-ce {ver} (SDL {'.'.join(map(str, get_sdl_version()))}, "
        f"Python {python_version})"
    )
