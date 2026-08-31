"""Karten ANLEGEN und SPEICHERN (`TILED_NEW` / `TILED_SAVE`).

Bis 2026-08-30 konnte ein Programm eine Tiled-Karte laden und im Speicher
ändern, aber weder eine neue anlegen noch etwas zurückschreiben. Für alles,
was Karten **baut** statt sie nur zu benutzen, war das die Sackgasse --
aufgefallen beim dritten Editor-Piloten.

**Die geschriebenen Dateien liest ein FREMDER Leser gegen**: `TileMapDoc`
aus `drachenhauch/tilemap/document.py`, das Modell des Qt-Tilemap-Editors.
Ein Format, das nur sein eigener Schreiber wieder lesen kann, ist nicht
geprüft, sondern nur in sich stimmig.
"""
import json
import sys
from pathlib import Path

_ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(_ROOT))

from drachenhauch.tilemap.document import TileMapDoc


_BAUEN = '''
IMPORT "tiled"
DIM m AS TILED_MAP
m = TILED_NEW(8, 5, 16, 16)
DIM ts AS INTEGER
ts = TILED_ADD_TILESET(m, "tiles.png", 64)
DIM boden AS INTEGER
boden = TILED_ADD_LAYER(m, "Boden")
DIM deko AS INTEGER
deko = TILED_ADD_LAYER(m, "Deko")
TILED_FILL_RECT(m, boden, 0, 0, 8, 5, 1)
TILED_TILE_SET(m, boden, 3, 2, 7)
TILED_TILE_SET(m, deko, 1, 1, 12)
TILED_SAVE(m, "karte.json")
'''


def test_ein_fremder_leser_versteht_die_datei(run_gb, tmp_path):
    run_gb(_BAUEN, base=tmp_path)
    d = TileMapDoc.load_json(str(tmp_path / "karte.json"))
    assert (d.width, d.height) == (8, 5)
    assert (d.tile_w, d.tile_h) == (16, 16)
    assert [l.name for l in d.layers] == ["Boden", "Deko"]
    assert d.layers[0].get(3, 2) == 7
    assert d.layers[1].get(1, 1) == 12
    assert d.layers[0].get(0, 0) == 1, "FILL_RECT muss den Rest gefuellt haben"


def test_die_datei_ist_eine_tiled_map(run_gb, tmp_path):
    """`type: "map"` ist das, woran jeder Leser sie erkennt -- der eigene
    lehnt ohne es ausdrücklich ab."""
    run_gb(_BAUEN, base=tmp_path)
    roh = json.loads((tmp_path / "karte.json").read_text(encoding="utf-8"))
    assert roh["type"] == "map"
    assert roh["orientation"] == "orthogonal"
    assert roh["infinite"] is False
    assert [l["type"] for l in roh["layers"]] == ["tilelayer", "tilelayer"]


def test_der_eigene_leser_bekommt_dasselbe_zurueck(run_gb, tmp_path):
    out = run_gb(_BAUEN + '''
DIM m2 AS TILED_MAP
m2 = TILED_LOAD("karte.json")
PRINT TILED_WIDTH(m2); " "; TILED_HEIGHT(m2); " "; TILED_LAYER_COUNT(m2); " "; _
      TILED_TILE_AT(m2, 0, 3, 2); " "; TILED_TILE_AT(m2, 1, 1, 1)
''', base=tmp_path)
    assert out.strip().split() == ["8", "5", "2", "7", "12"]


def test_gids_der_tilesets_ueberlappen_nicht(run_gb, tmp_path):
    """Die `firstgid` wird selbst vergeben. Überlappende Bereiche zerstören
    stillschweigend die Zuordnung ALLER Kacheln -- das ist der Grund, warum
    sie nicht von Hand gesetzt wird."""
    out = run_gb('''
IMPORT "tiled"
DIM m AS TILED_MAP
m = TILED_NEW(4, 4, 16, 16)
DIM a AS INTEGER
a = TILED_ADD_TILESET(m, "a.png", 64)
DIM b AS INTEGER
b = TILED_ADD_TILESET(m, "b.png", 20)
DIM c AS INTEGER
c = TILED_ADD_TILESET(m, "c.png", 5)
PRINT TILED_TILESET_FIRSTGID(m, 0); " "; TILED_TILESET_FIRSTGID(m, 1); " "; _
      TILED_TILESET_FIRSTGID(m, 2)
''', base=tmp_path)
    # 1..64, dann 65..84, dann 85..89
    assert out.strip().split() == ["1", "65", "85"]


def test_doppelter_ebenenname_wird_abgelehnt(run_gb, tmp_path):
    """Ebenen werden auch über den Namen gefunden (`TILED_LAYER_INDEX`) --
    zwei gleiche Namen machten den einen unerreichbar."""
    out = run_gb('''
IMPORT "tiled"
DIM m AS TILED_MAP
m = TILED_NEW(4, 4, 16, 16)
DIM a AS INTEGER
a = TILED_ADD_LAYER(m, "Boden")
TRY
    DIM b AS INTEGER
    b = TILED_ADD_LAYER(m, "Boden")
    PRINT "angenommen"
CATCH e
    PRINT "abgelehnt"
END TRY
''', base=tmp_path)
    assert out.strip() == "abgelehnt"


def test_unsinnige_masse_werden_abgelehnt(run_gb, tmp_path):
    for w, h in (("0", "5"), ("5", "0"), ("-3", "5")):
        out = run_gb(f'''
IMPORT "tiled"
TRY
    DIM m AS TILED_MAP
    m = TILED_NEW({w}, {h}, 16, 16)
    PRINT "angenommen"
CATCH e
    PRINT "abgelehnt"
END TRY
''', base=tmp_path)
        assert out.strip() == "abgelehnt", f"{w}x{h} wurde angenommen"


def test_riesige_karte_wird_abgelehnt(run_gb, tmp_path):
    """Ein Tippfehler in der Kachelzahl darf nicht den Speicher fressen."""
    out = run_gb('''
IMPORT "tiled"
TRY
    DIM m AS TILED_MAP
    m = TILED_NEW(9000, 9000, 16, 16)
    PRINT "angenommen"
CATCH e
    PRINT "abgelehnt"
END TRY
''', base=tmp_path)
    assert out.strip() == "abgelehnt"


def test_leere_karte_bleibt_lesbar(run_gb, tmp_path):
    """Ohne Ebenen und ohne Tileset -- der Grenzfall, den ein Editor beim
    ersten `Neu` erzeugt."""
    run_gb('''
IMPORT "tiled"
DIM m AS TILED_MAP
m = TILED_NEW(3, 3, 8, 8)
TILED_SAVE(m, "leer.json")
''', base=tmp_path)
    d = TileMapDoc.load_json(str(tmp_path / "leer.json"))
    assert (d.width, d.height) == (3, 3)
    assert (d.tile_w, d.tile_h) == (8, 8)
    # `TileMapDoc` legt beim Laden selbst eine Ebene an, wenn keine da ist
    # (document.py: `if not doc.layers`). Das ist sein Verhalten, nicht das
    # der Datei -- geprüft wird hier, dass die Datei überhaupt gelesen wird
    # und die Maße überstehen.


def test_ebene_umbenennen_fuehrt_den_namensindex_mit(run_gb, tmp_path):
    """`TILED_LAYER_INDEX` sucht ueber den Namen. Bliebe der alte Eintrag
    stehen, zeigte er stillschweigend auf die falsche Ebene."""
    out = run_gb('''
IMPORT "tiled"
DIM m AS TILED_MAP
m = TILED_NEW(4, 4, 16, 16)
DIM a AS INTEGER : a = TILED_ADD_LAYER(m, "Boden")
DIM b AS INTEGER : b = TILED_ADD_LAYER(m, "Deko")
TILED_LAYER_RENAME(m, 1, "Vordergrund")
PRINT TILED_LAYER_NAME(m, 1); " "; TILED_LAYER_INDEX(m, "Vordergrund"); " "; _
      TILED_LAYER_INDEX(m, "Deko")
''', base=tmp_path)
    assert out.strip().split() == ["Vordergrund", "1", "-1"]


def test_umbenennen_auf_den_eigenen_namen_ist_kein_fehler(run_gb, tmp_path):
    """Aus einem Eingabefeld heraus ist genau das der Normalfall."""
    out = run_gb('''
IMPORT "tiled"
DIM m AS TILED_MAP
m = TILED_NEW(4, 4, 16, 16)
DIM a AS INTEGER : a = TILED_ADD_LAYER(m, "Boden")
DIM b AS INTEGER : b = TILED_ADD_LAYER(m, "Deko")
TRY
    TILED_LAYER_RENAME(m, 0, "Boden")
    PRINT "ok"
CATCH e
    PRINT "abgelehnt"
END TRY
TRY
    TILED_LAYER_RENAME(m, 0, "Deko")
    PRINT "kollision-durch"
CATCH e
    PRINT "kollision-erkannt"
END TRY
''', base=tmp_path)
    assert out.strip().splitlines() == ["ok", "kollision-erkannt"]


def test_sichtbarkeit_uebersteht_die_datei(run_gb, tmp_path):
    """Tiled speichert `visible`. Ohne den Setter liess sich eine
    ausgeblendete Ebene gar nicht so sichern -- sie kam sichtbar zurueck."""
    out = run_gb('''
IMPORT "tiled"
DIM m AS TILED_MAP
m = TILED_NEW(4, 4, 16, 16)
DIM a AS INTEGER : a = TILED_ADD_LAYER(m, "Boden")
DIM b AS INTEGER : b = TILED_ADD_LAYER(m, "Deko")
TILED_LAYER_SET_VISIBLE(m, 0, FALSE)
TILED_SAVE(m, "v.json")
DIM m2 AS TILED_MAP : m2 = TILED_LOAD("v.json")
PRINT TILED_LAYER_VISIBLE(m2, 0); " "; TILED_LAYER_VISIBLE(m2, 1)
''', base=tmp_path)
    assert out.strip().split() == ["FALSE", "TRUE"]
    d = TileMapDoc.load_json(str(tmp_path / "v.json"))
    assert d.layers[0].visible is False
    assert d.layers[1].visible is True


def test_ebene_entfernen_rueckt_die_dahinter_auf(run_gb, tmp_path):
    out = run_gb('''
IMPORT "tiled"
DIM m AS TILED_MAP
m = TILED_NEW(4, 4, 16, 16)
DIM a AS INTEGER : a = TILED_ADD_LAYER(m, "Eins")
DIM b AS INTEGER : b = TILED_ADD_LAYER(m, "Zwei")
DIM c AS INTEGER : c = TILED_ADD_LAYER(m, "Drei")
TILED_TILE_SET(m, 2, 1, 1, 9)
TILED_REMOVE_LAYER(m, 0)
PRINT TILED_LAYER_COUNT(m); " "; TILED_LAYER_NAME(m, 0); " "; TILED_LAYER_NAME(m, 1)
PRINT TILED_LAYER_INDEX(m, "Drei"); " "; TILED_LAYER_INDEX(m, "Eins"); " "; _
      TILED_TILE_AT(m, 1, 1, 1)
''', base=tmp_path)
    zeilen = out.strip().splitlines()
    assert zeilen[0].split() == ["2", "Zwei", "Drei"]
    # "Drei" ist nach dem Entfernen Ebene 1 -- und traegt weiter ihre Kachel.
    assert zeilen[1].split() == ["1", "-1", "9"]


def test_ebenenbefehle_lehnen_falsche_nummern_ab(run_gb, tmp_path):
    out = run_gb('''
IMPORT "tiled"
DIM m AS TILED_MAP
m = TILED_NEW(4, 4, 16, 16)
DIM a AS INTEGER : a = TILED_ADD_LAYER(m, "Boden")
DIM n AS INTEGER : n = 0
TRY
    TILED_LAYER_RENAME(m, 5, "x")
CATCH e
    n = n + 1
END TRY
TRY
    TILED_LAYER_SET_VISIBLE(m, -1, FALSE)
CATCH e
    n = n + 1
END TRY
TRY
    TILED_REMOVE_LAYER(m, 9)
CATCH e
    n = n + 1
END TRY
TRY
    DIM v AS BOOLEAN : v = TILED_LAYER_VISIBLE(m, 3)
CATCH e
    n = n + 1
END TRY
PRINT n
''', base=tmp_path)
    assert out.strip() == "4"
