"""`pygame.key.ScancodeWrapper` - siehe `taste.rs`, Funktion `huelle`."""

class ScancodeWrapper(tuple):
    """Was `pygame.key.get_pressed()` zurueckgibt - **ein `tuple`**.

    Gemessen an pygame-ce 2.5.7:

    ```text
    isinstance(k, tuple)   True
    len(k)                 512
    k.count(False)         512      (nichts unten)
    k.index(False)         0
    k + k                  gewoehnliches Tupel mit 1024 Eintraegen
    list(k)                TypeError: Iterating over key states is not supported
    k[512], k[-1]          False  -  **kein** IndexError
    k[0:2]                 TypeError: 'slice' object cannot be interpreted as an integer
    repr(W((7, 8, 9)))     pygame.key.ScancodeWrapper(7, 8, 9)
    ```

    Die Reihenfolge im Tupel ist der **Tastenplatz** (`KSCAN_*`);
    indiziert wird aber mit dem **Tastencode** (`K_*`), und `K_UP` ist
    1073741906. `__getitem__` rechnet darum um - `_platz` aus `taste.rs`
    macht dasselbe wie pygames C-Code im Verborgenen. Siehe die
    Begruendung dort, warum diese Klasse nicht in Rust steht.
    """

    #: **Kein `__dict__`.** Gemessen gibt pygames Wrapper
    #: `AttributeError: 'pygame.key.ScancodeWrapper' object has no
    #: attribute '__dict__'`; eine gewoehnliche Python-Klasse hat eines,
    #: und hier stand darum `[]`. Ein leeres `__slots__` nimmt es weg -
    #: `tuple` bringt seinen Inhalt ohnehin selbst mit.
    __slots__ = ()

    def __getattribute__(self, name):
        """Der Satz nennt den **ganzen Weg** der Klasse.

        CPython baut ihn aus `tp_name`, und das ist bei einer Klasse aus
        Python der blosse Name - bei pygames C-Typ der ganze Weg:

        ```text
        pygame     'pygame.key.ScancodeWrapper' object has no attribute '__dict__'
        hier       'ScancodeWrapper' object has no attribute '__dict__'
        ```

        Jede andere Klasse hier nennt ihn (`'rustgame_py.surface.Surface'
        object has no attribute ...`), diese eine nicht - weil sie als
        einzige in Python steht (`tuple` laesst sich mit PyO3 nicht
        ableiten, siehe `taste.rs`). Also wird der Satz hier selbst
        geschrieben.

        **`__getattribute__`, nicht `__getattr__`.** Hier stand
        `__getattr__`, und das ist ein Name, den pygames Typ nicht hat -
        die Probe `Klassendoppel` zeigte ihn, unter einem Vermerk, der von
        etwas anderem sprach. `__getattribute__` steht ueber `tuple` und
        `object` ohnehin in `dir()`; es zu ueberschreiben fuegt keinen
        Namen hinzu. Der Zugriff ueber `[]` laeuft am Typ vorbei und kostet
        nichts; nur Methodenaufrufe am Objekt gehen hier durch.
        """
        try:
            return tuple.__getattribute__(self, name)
        except AttributeError:
            pass
        raise AttributeError(
            "'rustgame_py.key.ScancodeWrapper' object has no attribute %r"
            % (name,), name=name, obj=self)

    def __getitem__(self, index):
        """Zugriff mit einem **Tastencode**, nicht mit dem Platz."""
        if isinstance(index, slice):
            # Gemessen: pygame lehnt eine Scheibe ab, und zwar mit genau
            # diesem Satz.
            raise TypeError(
                "'slice' object cannot be interpreted as an integer")
        import rustgame_py

        platz = rustgame_py.key._platz(index)
        if platz < tuple.__len__(self):
            return tuple.__getitem__(self, platz)
        # Ausserhalb gibt pygame `False` und keinen `IndexError`. Bei einem
        # selbst gebauten, kuerzeren Wrapper ist das der **erste** Eintrag -
        # gemessen; Platz 0 ist `SDL_SCANCODE_UNKNOWN`.
        if tuple.__len__(self):
            return tuple.__getitem__(self, 0)
        return False

    def __iter__(self):
        """pygame lehnt das ausdruecklich ab - wortgleich."""
        raise TypeError("Iterating over key states is not supported")

    def __next__(self):
        """**Auch `__next__` ist da**, und es lehnt genauso ab.

        Gemessen hat pygames `ScancodeWrapper` einen `__next__`-Schlitz
        (`<slot wrapper '__next__'>`), obwohl `__iter__` schon abweist -
        `next(k)` gibt denselben Satz. Ein Doppelstrich, nach dem keine der
        804 anderen Proben fragte.
        """
        raise TypeError("Iterating over key states is not supported")

    def __repr__(self):
        """`rustgame_py.key.ScancodeWrapper(...)` - mit dem Klassennamen.

        `tuple.__repr__` gaebe `(False, False, ...)`; gemessen schreibt
        pygame den vollen Namen davor. Durchlaufen laesst sich die Klasse
        nicht, also holt der Aufbau die Eintraege ueber
        `tuple.__getitem__`.
        """
        teile = ", ".join(
            repr(tuple.__getitem__(self, i))
            for i in range(tuple.__len__(self)))
        return "%s.%s(%s)" % (type(self).__module__, type(self).__qualname__,
                              teile)
