"""`tiled` kann jetzt Objekt-Ebenen und Kachel-Eigenschaften ANLEGEN.

Bis 2026-09-02 konnte das Modul beides nur LESEN. Wer eine Karte im Programm
baute, konnte also keine Spawn-Punkte setzen und keiner Kachel `solid`
mitgeben -- genau die zwei Dinge, wegen derer man Objekt-Ebenen und
Eigenschaften ueberhaupt hat. Aufgefallen ist es am Tilemap-Piloten: beide
standen dort in der "nicht portiert"-Liste, und zwar nicht aus Zeitmangel,
sondern weil die Laufzeit es nicht hergab.

Der SCHREIBER war schon vollstaendig -- `speichern()` gab Objektebenen als
`objectgroup` und Kachel-Eigenschaften unter `tiles` aus, es gab nur nichts
zu schreiben. Deshalb sind es reine Anlege-Befehle, kein Umbau des Formats.

Geprueft wird gegen einen FREMDEN Leser: `TileMapDoc` aus
`drachenhauch/tilemap/document.py`, das Modell des Qt-Editors. Ein Format,
das nur sein eigener Leser versteht, ist nicht geprueft, sondern nur in sich
stimmig.
"""
import json
import os
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


def _find_dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    return next((_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
                 for v in ("release", "debug")
                 if (_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe).exists()), None)


_DHRT = _find_dhrt()
pytestmark = pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")


def _run(tmp_path, src):
    (tmp_path / "a.dh").write_text(src, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(tmp_path / "a.dh")], capture_output=True,
                       text=True, encoding="utf-8", errors="replace", timeout=90,
                       cwd=str(tmp_path))
    return r


_BAUEN = '''IMPORT "tiled"
DIM m AS TILED_MAP : m = TILED_NEW(4, 3, 16, 16)
DIM t AS INTEGER : t = TILED_ADD_TILESET(m, "kacheln.png", 8)
DIM e AS INTEGER : e = TILED_ADD_LAYER(m, "Boden")
TILED_TILE_SET(m, 0, 1, 1, 3)
TILED_TILE_SET_PROP(m, 3, "solid", TRUE)
TILED_TILE_SET_PROP(m, 3, "damage", 5)
TILED_TILE_SET_PROP(m, 4, "reibung", 0.5)
TILED_TILE_SET_PROP(m, 5, "art", "wasser")
DIM o AS INTEGER : o = TILED_ADD_OBJECT_LAYER(m, "spawns")
DIM i1 AS INTEGER : i1 = TILED_ADD_OBJECT(m, "spawns", "held", "spawn", 32.0, 48.0, 16.0, 16.0)
DIM i2 AS INTEGER : i2 = TILED_ADD_OBJECT(m, "spawns", "gegner", "spawn", 64.0, 16.0, 8.0, 8.0)
TILED_OBJECT_SET_PROP(m, "spawns", i2, "leben", 3)
TILED_OBJECT_SET_PROP(m, "spawns", i2, "boss", TRUE)
TILED_SAVE(m, "karte.json")
'''


@pytest.fixture
def karte(tmp_path):
    r = _run(tmp_path, _BAUEN)
    assert r.returncode == 0, r.stderr
    return tmp_path / "karte.json"


# ------------------------------------------------------- der fremde Leser
def test_der_qt_editor_liest_die_objekt_ebene(karte):
    from drachenhauch.tilemap.document import TileMapDoc, ObjectLayer
    doc = TileMapDoc.load_json(karte)
    ebenen = [l for l in doc.layers if isinstance(l, ObjectLayer)]
    assert len(ebenen) == 1 and ebenen[0].name == "spawns"
    namen = sorted(o.name for o in ebenen[0].objects)
    assert namen == ["gegner", "held"]
    held = next(o for o in ebenen[0].objects if o.name == "held")
    assert (held.x, held.y, held.width, held.height) == (32, 48, 16, 16)
    assert held.type == "spawn"


def test_der_qt_editor_liest_die_objekt_eigenschaften(karte):
    from drachenhauch.tilemap.document import TileMapDoc, ObjectLayer
    doc = TileMapDoc.load_json(karte)
    ebene = next(l for l in doc.layers if isinstance(l, ObjectLayer))
    gegner = next(o for o in ebene.objects if o.name == "gegner")
    assert gegner.properties == {"leben": 3, "boss": True}


def test_der_qt_editor_liest_die_kachel_eigenschaften(karte):
    """Die drei Typen auf einmal: BOOLEAN, INTEGER, FLOAT und STRING gehen
    ohne Typangabe hinein -- der Wert traegt seinen Typ schon mit sich."""
    from drachenhauch.tilemap.document import TileMapDoc
    doc = TileMapDoc.load_json(karte)
    # Lokale Nummern: firstgid ist 1, also gid 3 -> lokal 2.
    assert doc.properties_of(2) == {"solid": True, "damage": 5}
    assert doc.properties_of(3) == {"reibung": 0.5}
    assert doc.properties_of(4) == {"art": "wasser"}


# ------------------------------------------------------------- Rundweg
def test_die_laufzeit_liest_ihr_eigenes_ergebnis_zurueck(tmp_path):
    r = _run(tmp_path, _BAUEN + '''DIM n AS TILED_MAP : n = TILED_LOAD("karte.json")
PRINT TILED_TILE_PROP_BOOL(n, 3, "solid")
PRINT TILED_TILE_PROP_INT(n, 3, "damage")
PRINT TILED_TILE_PROP_STRING(n, 5, "art")
PRINT TILED_OBJECT_COUNT(n, "spawns")
PRINT TILED_OBJECT_PROP_INT(n, "spawns", 1, "leben")
PRINT TILED_OBJECT_NAME(n, "spawns", 0)
''')
    assert r.returncode == 0, r.stderr
    assert r.stdout.split() == ["TRUE", "5", "wasser", "2", "3", "held"]


def test_entfernen_raeumt_die_kachel_ganz_aus_der_datei(tmp_path):
    """Eine Kachel ohne Eigenschaften darf nicht mit leerer Liste in der
    Datei stehen bleiben -- `speichern` fuehrt jede Kachel auf, die einen
    Eintrag hat, und ein leerer waere Rauschen."""
    r = _run(tmp_path, '''IMPORT "tiled"
DIM m AS TILED_MAP : m = TILED_NEW(2, 2, 16, 16)
DIM t AS INTEGER : t = TILED_ADD_TILESET(m, "k.png", 4)
TILED_TILE_SET_PROP(m, 2, "solid", TRUE)
TILED_TILE_SET_PROP(m, 3, "solid", TRUE)
TILED_TILE_REMOVE_PROP(m, 2, "solid")
TILED_SAVE(m, "k.json")
''')
    assert r.returncode == 0, r.stderr
    d = json.loads((tmp_path / "k.json").read_text(encoding="utf-8"))
    ids = sorted(t["id"] for t in d["tilesets"][0]["tiles"])
    assert ids == [2], "nur die Kachel mit Eigenschaft, gid 3 -> lokal 2"


# --------------------------------------------------------------- Meldungen
@pytest.mark.parametrize("zeile,teil", [
    ('DIM o AS INTEGER : o = TILED_ADD_OBJECT(m, "gibtsnicht", "a", "b", 0, 0, 1, 1)',
     "gibt es nicht"),
    ('DIM o AS INTEGER : o = TILED_ADD_OBJECT(m, "Boden", "a", "b", 0, 0, 1, 1)',
     "keine Objekt-Ebene"),
    ('DIM o AS INTEGER : o = TILED_ADD_OBJECT_LAYER(m, "Boden")', "gibt es schon"),
    ('TILED_TILE_SET_PROP(m, 99, "x", 1)', "ausserhalb des Tilesets"),
    ('TILED_TILE_SET_PROP(m, 3, "", 1)', "darf nicht leer sein"),
])
def test_fehler_sagen_was_los_ist(tmp_path, zeile, teil):
    """Eine Meldung, die den Fall benennt, statt eines stillen Nichts --
    beim Anlegen einer Karte im Programm sieht man das Ergebnis sonst erst
    im fertigen Level."""
    r = _run(tmp_path, '''IMPORT "tiled"
DIM m AS TILED_MAP : m = TILED_NEW(4, 3, 16, 16)
DIM t AS INTEGER : t = TILED_ADD_TILESET(m, "k.png", 8)
DIM e AS INTEGER : e = TILED_ADD_LAYER(m, "Boden")
''' + zeile + "\n")
    assert r.returncode != 0
    assert teil in (r.stderr + r.stdout), r.stderr


def test_ein_array_ist_keine_eigenschaft(tmp_path):
    """Tiled kennt genau vier Arten. Ein ARRAY wuerde beim Speichern still
    zu Text zerfallen -- also lieber gleich melden."""
    r = _run(tmp_path, '''IMPORT "tiled"
DIM m AS TILED_MAP : m = TILED_NEW(2, 2, 16, 16)
DIM t AS INTEGER : t = TILED_ADD_TILESET(m, "k.png", 4)
DIM feld AS ARRAY OF INTEGER : feld = [1, 2]
TILED_TILE_SET_PROP(m, 1, "x", feld)
''')
    assert r.returncode != 0
    assert "BOOLEAN, INTEGER, FLOAT oder STRING" in (r.stderr + r.stdout), r.stderr
