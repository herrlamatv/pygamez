"""`pygame.docs` - sagt, ob die Doku mitgeliefert wurde.

Das Modul hat genau zwei Namen, und `docs_test.py` prueft genau das:

```python
def test_doc_import_works(self):
    from pygame.docs import PKG_DIR, has_local_docs
```

Die beiden anderen Tests der Datei ueberspringen sich selbst, solange
`CI` nicht in der Umgebung steht - sie wollen `python -m pygame.docs`
starten und eine echte Doku finden.

## Die ehrliche Antwort ist `False`

rustgame liefert keine erzeugte Doku mit, also gibt `has_local_docs()`
`False`. Das ist **kein Mangel dieses Moduls**, sondern seine Aufgabe:
pygame hat es gebaut, um genau diese Frage beantworten zu koennen, und
bei einem Rad ohne Doku ist `False` die richtige Antwort. Ein `True` zu
melden und dann ins Leere zu zeigen waere der schlechtere Weg - dieselbe
Lage wie bei `_sdl2.video.get_grabbed_window`, wo ein Zeiger auf eine
leere Stelle schlimmer war als kein Zeiger.

`PKG_DIR` zeigt dorthin, wo die Doku liegen **wuerde**: neben diesem
Modul, im Ordner `generated`. So macht es pygame auch, und so stimmt die
Form ueberein.

## Gemessen

```text
pygame.docs.PKG_DIR          .../site-packages/pygame/docs/generated
pygame.docs.has_local_docs() True   (pygame-ce bringt sie mit)
```
"""

from pathlib import Path

__all__ = ["PKG_DIR", "has_local_docs"]

# Dieselbe Rechnung wie bei pygame: der Ordner `generated` neben diesem
# Modul. Dass er hier nicht da ist, sagt `has_local_docs()`.
PKG_DIR = Path(__file__).absolute().parent / "generated"


def has_local_docs():
    """Liegt die erzeugte Doku neben dem Rad? Hier **nein**."""
    return PKG_DIR.exists()
