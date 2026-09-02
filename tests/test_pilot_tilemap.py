"""Der Tilemap-Pilot, bedient wie von Hand (`examples/187_tilemap_editor.dh`).

Wie beim Sprite-Piloten: er ist ein Drachenhauch-Programm, also wird er
bedient statt aufgerufen -- mit aufgezeichneter Eingabe (`AUTOMATION_PLAY`),
und die Kopie bekommt ein festes Fenster (die Vollbild-Groesse haengt am
Monitor) samt einer PRINT-Zeile, die nur bestehende Werte ausliest.

Geprueft wird hier die GB-Code-Ausgabe -- und zwar so, wie es bei erzeugtem
Code allein zaehlt: das Programm wird GESTARTET und sein Bild angesehen. Ob
es uebersetzt, sagt nichts darueber, ob es sein Tileset findet, die Karte
liest und tatsaechlich etwas zeichnet.
"""
import os
import re
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent
_PILOT = _ROOT / "examples" / "187_tilemap_editor.dh"


def _find_dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    return next((_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
                 for v in ("release", "debug")
                 if (_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe).exists()), None)


_DHRT = _find_dhrt()
pytestmark = [pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut"),
              pytest.mark.seriell]

KEY_UP, KEY_DOWN = 1, 2
MOUSE_BUTTON_UP, MOUSE_BUTTON_DOWN, MOUSE_POSITION = 5, 6, 7
MAUSRAD = 8                    # raylibs INPUT_MOUSE_WHEEL_MOTION (x, y)
TASTE_ENTF = 261
TASTE_I = 73                   # raylib-Codes, nicht die von Drachenhauch
TASTE_UMSCHALT = 340

# Nur was dieser Test braucht: die Lage des Knopfes und die Kartenmasse.
_PROBE = '''    PRINT "P " + STR$(GUI_GET_X(bCode)) + " " + STR$(GUI_GET_Y(bCode)) + _
          " " + STR$(kb) + " " + STR$(kh) + " " + STR$(zell()) + _
          " " + STR$(GUI_GET_X(cbSolid)) + " " + STR$(GUI_GET_Y(cbSolid)) + _
          " " + STR$(GUI_GET_X(bPropSetz)) + " " + STR$(GUI_GET_Y(bPropSetz)) + _
          " " + STR$(GUI_GET_X(bPropWeg)) + " " + STR$(GUI_GET_Y(bPropWeg)) + _
          " " + STR$(GUI_GET_X(lstProp)) + " " + STR$(GUI_GET_Y(lstProp)) + _
          " " + STR$(gewaehlt) + _
          " " + STR$(GUI_GET_X(bObjEbene)) + " " + STR$(GUI_GET_Y(bObjEbene)) + _
          " " + STR$(GUI_GET_X(bObjWeg)) + " " + STR$(GUI_GET_Y(bObjWeg)) + _
          " " + STR$(GUI_GET_X(bObjUeb)) + " " + STR$(GUI_GET_Y(bObjUeb)) + _
          " " + STR$(GUI_GET_X(lstObj)) + " " + STR$(GUI_GET_Y(lstObj)) + _
          " " + STR$(IIF(istObjEbene(), 1, 0)) + " " + STR$(objSel) + _
          " " + STR$(IIF(istObjEbene(), TILED_OBJECT_COUNT(karte, ebenenName$()), -1)) + _
          " " + STR$(GUI_CANVAS_X(leinwand)) + " " + STR$(GUI_CANVAS_Y(leinwand)) + _
          " " + STR$(zoom) + " " + STR$(TILED_LAYER_COUNT(karte)) + _
          " " + STR$(GUI_CANVAS_X(palette)) + " " + STR$(GUI_CANVAS_Y(palette)) + _
          " " + STR$(GUI_GET_X(bTsNeu)) + " " + STR$(GUI_GET_Y(bTsNeu)) + _
          " " + STR$(tsAnz) + " " + STR$(tsAkt) + " " + STR$(basisGid())
'''
_FELDER = ("codeX codeY kb kh zell solidX solidY setzX setzY "
           "wegX wegY propX propY gewaehlt "
           "objEbX objEbY objWegX objWegY objUebX objUebY lstObjX lstObjY "
           "istObj objSel objAnz cvX cvY zoom ebenen "
           "palX palY tsNeuX tsNeuY tsAnz tsAkt basisGid").split()

_DIALOG_CODE = 'FILE_SAVE_DIALOG("GB-Code sichern", "karte.dh", "dh")'


def _kopie(tmp_path, dialoge=None):
    src = _PILOT.read_text(encoding="utf-8")
    for alt, neu in (dialoge or {}).items():
        assert src.count(alt) == 1, alt
        src = src.replace(alt, neu)
    assert src.count("SET_FULLSCREEN(TRUE)") == 1
    src = src.replace("SET_FULLSCREEN(TRUE)", "SET_WINDOW_POS(-3000, -3000)")
    assert src.count("    FLIP()\nWEND") == 1
    src = src.replace("    FLIP()\nWEND", _PROBE + "    FLIP()\nWEND")
    # Das Tileset liegt relativ zum Programm -- die Kopie liegt woanders.
    quelle = _ROOT / "examples" / "assets" / "editor_tileset.png"
    (tmp_path / "assets").mkdir(exist_ok=True)
    (tmp_path / "assets" / "editor_tileset.png").write_bytes(quelle.read_bytes())
    ziel = tmp_path / "pilot.dh"
    ziel.write_text(src, encoding="utf-8")
    return ziel


def _events(tmp_path, events):
    # Nach Bild sortiert: die Wiedergabe arbeitet die Liste der Reihe nach ab
    # und haelt beim ersten spaeteren Eintrag an. Ein nachgereichtes frueheres
    # Ereignis kaeme sonst zusammen mit einem spaeteren in EINEM Bild an --
    # zwei Radschritte in einem Bild sind aber nur einer.
    events = sorted(events, key=lambda e: e[0])
    lines = ["# Test-Aufnahme", "c %d" % len(events)]
    for frame, typ, *params in events:
        p = (list(params) + [0, 0, 0, 0])[:4]
        lines.append("e %d %d %d %d %d %d // Event: test"
                     % (frame, typ, p[0], p[1], p[2], p[3]))
    (tmp_path / "ev.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")


def _lauf(tmp_path, frames, events=None, dialoge=None):
    quelle = _kopie(tmp_path, dialoge)
    if events is not None:
        _events(tmp_path, events)
        text = quelle.read_text(encoding="utf-8")
        text = text.replace("SETFPS(60)", 'SETFPS(60)\nAUTOMATION_PLAY("ev.txt")', 1)
        quelle.write_text(text, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(quelle)], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", timeout=180,
                       env=dict(os.environ, DHRT_FRAMES=str(frames)), cwd=str(tmp_path))
    assert r.returncode == 0, r.stderr
    zeilen = [ln for ln in (r.stdout or "").splitlines() if ln.startswith("P ")]
    assert zeilen, (r.stdout, r.stderr)
    return [dict(zip(_FELDER, [int(v) for v in re.split(r"\s+", ln)[1:]])) for ln in zeilen]


def _klick(frame, x, y):
    return [(frame, MOUSE_POSITION, x, y),
            (frame + 1, MOUSE_POSITION, x, y),
            (frame + 1, MOUSE_BUTTON_DOWN, 0),
            (frame + 2, MOUSE_BUTTON_UP, 0)]


def _exportieren(tmp_path):
    """[GB-Code] druecken. Liefert die drei erwarteten Dateien."""
    geo = _lauf(tmp_path, 6)[-1]
    ev = _klick(3, geo["codeX"] + 52, geo["codeY"] + 16)
    _lauf(tmp_path, 20, ev, dialoge={_DIALOG_CODE: '"raus.dh"'})
    return tmp_path / "raus.dh", tmp_path / "raus.json", tmp_path / "raus_1.png"


def test_gb_code_bringt_karte_und_tileset_mit(tmp_path):
    """Alle drei Dateien, unter demselben Namen. Fehlt eine, zeigt der Code
    ins Leere -- und das faellt erst beim Starten auf."""
    code, karte, tileset = _exportieren(tmp_path)
    assert code.exists() and karte.exists() and tileset.exists()
    text = code.read_text(encoding="utf-8")
    assert 'LOADIMAGE("raus_" + STR$(ti + 1) + ".png")' in text
    assert 'TILED_LOAD("raus.json")' in text


def _rendern(tmp_path):
    """Den erzeugten Renderer starten und die Farben seines Bildes holen."""
    bild = tmp_path / "gerendert.png"
    r = subprocess.run([str(_DHRT), "run", str(tmp_path / "raus.dh")],
                       capture_output=True, text=True, encoding="utf-8", errors="replace",
                       timeout=120, cwd=str(tmp_path),
                       env=dict(os.environ, DHRT_FRAMES="4", DHRT_SCREENSHOT=str(bild)))
    assert r.returncode == 0, r.stderr
    from PIL import Image
    im = Image.open(bild).convert("RGB")
    return {im.getpixel((x, y))
            for x in range(0, im.width, 4) for y in range(0, im.height, 4)}


def test_der_erzeugte_renderer_zeichnet_wirklich(tmp_path):
    """Startet den erzeugten Renderer und sieht sein BILD an.

    Uebersetzen wuerde nichts beweisen: ein Renderer, der sein Tileset nicht
    findet oder die gid falsch aufloest, uebersetzt genauso und zeigt einen
    leeren Schirm. Die Gegenprobe steckt deshalb im Test -- erst die leere
    Karte (ein einziger Farbton), dann dieselbe mit einer gemalten Kachel.
    """
    pytest.importorskip("PIL")
    _exportieren(tmp_path)
    leer = _rendern(tmp_path)
    assert len(leer) == 1, "eine leere Karte ist nur Hintergrund: %s" % leer

    geo = _lauf(tmp_path, 6)[-1]
    # Eine Kachel malen (die Zeichenflaeche beginnt rechts der linken Spalte,
    # unter der Werkzeugleiste), dann ausgeben.
    ev = _klick(3, 320 + geo["zell"] * 2, 60 + geo["zell"] * 2)
    ev += _klick(10, geo["codeX"] + 52, geo["codeY"] + 16)
    _lauf(tmp_path, 26, ev, dialoge={_DIALOG_CODE: '"raus.dh"'})
    gemalt = _rendern(tmp_path)
    assert len(gemalt) > 1, "die gemalte Kachel fehlt im erzeugten Bild"


# ------------------------------------------------------- Kachel-Eigenschaften
# Die Werte kommen aus Eingabefeldern; aufgezeichnete Eingabe erreicht sie
# nicht (Zeichen gehen einen anderen Weg als Tasten). Ersetzt wird deshalb
# genau das ABLESEN der beiden Felder -- gedeutet und gesetzt wird vom
# echten Code.
_KEY = 'DIM sk AS STRING : sk = TRIM$(GUI_TEXT(edKey))'
_WERT = 'eigenschaftSetzen(pg, sk, GUI_TEXT(edWert))'


def _mit_feldern(key, wert):
    return {_KEY: 'DIM sk AS STRING : sk = "%s"' % key,
            _WERT: 'eigenschaftSetzen(pg, sk, "%s")' % wert,
            _DIALOG_CODE: '"raus.dh"'}


def _eigenschaften_von(tmp_path, lokal=0):
    """Die Eigenschaften einer Kachel, gelesen vom FREMDEN Leser -- dem
    Datenmodell des Qt-Editors."""
    from drachenhauch.tilemap.document import TileMapDoc
    return TileMapDoc.load_json(tmp_path / "raus.json").properties_of(lokal)


def test_solid_kaestchen_setzt_und_entfernt(tmp_path):
    """Der haeufigste Fall, und der einzige mit eigenem Schalter: genau
    `solid` fragt `tile_collide` ab."""
    geo = _lauf(tmp_path, 6)[-1]
    ev = _klick(3, geo["solidX"] + 8, geo["solidY"] + 8)
    ev += _klick(9, geo["codeX"] + 52, geo["codeY"] + 16)
    _lauf(tmp_path, 24, ev, dialoge={_DIALOG_CODE: '"raus.dh"'})
    assert _eigenschaften_von(tmp_path) == {"solid": True}

    # ... und wieder ab. ENTFERNT werden muss es, nicht auf FALSE gesetzt:
    # sobald irgendeine Kachel ein `solid` hat, schaltet die Kollision auf
    # "nur die mit solid = TRUE" um.
    ev += _klick(28, geo["solidX"] + 8, geo["solidY"] + 8)
    ev += _klick(34, geo["codeX"] + 52, geo["codeY"] + 16)
    _lauf(tmp_path, 50, ev, dialoge={_DIALOG_CODE: '"raus.dh"'})
    assert _eigenschaften_von(tmp_path) == {}


@pytest.mark.parametrize("eingabe,erwartet", [
    ("5", 5),                 # INTEGER
    ("2.5", 2.5),             # FLOAT -- am Punkt erkannt
    ("true", True),           # BOOLEAN
    ("falsch", False),        # auch deutsch
    ("stein", "stein"),       # alles andere bleibt Text
    ("-3", -3),               # Vorzeichen nur ganz vorn
    ("1-2", "1-2"),           # ... also ist das hier KEINE Zahl
])
def test_der_wert_bekommt_die_passende_art(tmp_path, eingabe, erwartet):
    """Tiled unterscheidet vier Arten, ein Eingabefeld kann sie nicht
    erfragen -- also wird gedeutet. Geprueft am fremden Leser, weil erst der
    zeigt, was WIRKLICH in der Datei steht: eine 5 als Text sieht dort fast
    genauso aus wie eine 5 als Zahl."""
    geo = _lauf(tmp_path, 6)[-1]
    ev = _klick(3, geo["setzX"] + 50, geo["setzY"] + 14)
    ev += _klick(9, geo["codeX"] + 52, geo["codeY"] + 16)
    _lauf(tmp_path, 24, ev, dialoge=_mit_feldern("wert", eingabe))
    gelesen = _eigenschaften_von(tmp_path)
    assert gelesen == {"wert": erwartet}
    assert type(gelesen["wert"]) is type(erwartet), gelesen


def test_eigenschaft_aus_der_liste_entfernen(tmp_path):
    """[Weg] nimmt die in der Liste gewaehlte -- ohne Auswahl sagt es das,
    statt stillschweigend die erste zu nehmen."""
    geo = _lauf(tmp_path, 6)[-1]
    setzen = _klick(3, geo["setzX"] + 50, geo["setzY"] + 14)
    # Erste Zeile der Liste waehlen, dann [Weg].
    ev = setzen + _klick(9, geo["propX"] + 40, geo["propY"] + 10)
    ev += _klick(15, geo["wegX"] + 40, geo["wegY"] + 14)
    ev += _klick(21, geo["codeX"] + 52, geo["codeY"] + 16)
    _lauf(tmp_path, 36, ev, dialoge=_mit_feldern("damage", "7"))
    assert _eigenschaften_von(tmp_path) == {}


def test_die_eigenschaften_gehoeren_der_gewaehlten_kachel(tmp_path):
    """Sie haengen am Tileset, nicht an einem Kartenfeld -- eine andere
    Palettenkachel hat also andere. Ohne diesen Test koennte der Kasten
    stumpf immer Kachel 0 beschreiben, und es fiele nicht auf."""
    geo = _lauf(tmp_path, 6)[-1]
    # Zweite Palettenkachel waehlen (jede ist 16*2 Punkte gross).
    ev = _klick(3, geo["palX"] + 48, geo["palY"] + 16)
    ev += _klick(9, geo["setzX"] + 50, geo["setzY"] + 14)
    ev += _klick(15, geo["codeX"] + 52, geo["codeY"] + 16)
    proben = _lauf(tmp_path, 30, ev, dialoge=_mit_feldern("art", "wasser"))
    assert proben[-1]["gewaehlt"] == 1, "die zweite Kachel ist gewaehlt"
    assert _eigenschaften_von(tmp_path, 1) == {"art": "wasser"}
    assert _eigenschaften_von(tmp_path, 0) == {}, "Kachel 0 bleibt unberuehrt"


# ----------------------------------------------------------- Objekt-Ebenen
_OBJ_NAME = 'DIM nm3 AS STRING : nm3 = TRIM$(GUI_TEXT(edObjName))'
_OBJ_NEU = 'DIM on2 AS STRING : on2 = TRIM$(GUI_TEXT(edName))'


def _mit_objnamen(name, typ="spawn", ebene="spawns"):
    """Die drei Eingabefelder durch feste Werte ersetzen -- aufgezeichnete
    Eingabe erreicht sie nicht."""
    return {
        _OBJ_NEU: 'DIM on2 AS STRING : on2 = "%s"' % ebene,
        _OBJ_NAME: 'DIM nm3 AS STRING : nm3 = "%s"' % name,
        'TRIM$(GUI_TEXT(edObjTyp)), nx, ny, nw, nh)': '"%s", nx, ny, nw, nh)' % typ,
        _DIALOG_CODE: '"raus.dh"',
    }


def _karte_punkt(geo, px, py):
    """Karten-Punkt -> Bildschirm. Ohne Verschiebung, also direkt."""
    return geo["cvX"] + px * geo["zoom"], geo["cvY"] + py * geo["zoom"]


def _ziehen(frame, x1, y1, x2, y2):
    return [(frame, MOUSE_POSITION, x1, y1),
            (frame + 1, MOUSE_POSITION, x1, y1),
            (frame + 1, MOUSE_BUTTON_DOWN, 0),
            (frame + 2, MOUSE_POSITION, x2, y2),
            (frame + 2, MOUSE_BUTTON_DOWN, 0),
            (frame + 3, MOUSE_POSITION, x2, y2),
            (frame + 4, MOUSE_BUTTON_UP, 0)]


def _objekte_von(tmp_path, ebene="spawns"):
    """Die Objekte, gelesen vom FREMDEN Leser."""
    from drachenhauch.tilemap.document import TileMapDoc, ObjectLayer
    doc = TileMapDoc.load_json(tmp_path / "raus.json")
    l = next(l for l in doc.layers if isinstance(l, ObjectLayer) and l.name == ebene)
    return l.objects


def test_objekt_ebene_anlegen_schaltet_den_block_um(tmp_path):
    """Beide Bloecke liegen auf demselben Platz -- welcher zu sehen ist,
    entscheidet die Art der Ebene. Ohne das waere die Spalte laenger als der
    Schirm, und auf einer Objektebene staenden Kachel-Eigenschaften herum,
    die dort nichts bewirken."""
    geo = _lauf(tmp_path, 6)[-1]
    assert geo["istObj"] == 0, "die Startebene ist eine Kachelebene"
    ev = _klick(3, geo["objEbX"] + 70, geo["objEbY"] + 14)
    letzte = _lauf(tmp_path, 20, ev, dialoge=_mit_objnamen("held"))[-1]
    assert letzte["istObj"] == 1 and letzte["ebenen"] == 2
    assert letzte["objAnz"] == 0


def test_ziehen_legt_ein_rechteck_an(tmp_path):
    geo = _lauf(tmp_path, 6)[-1]
    ev = _klick(3, geo["objEbX"] + 70, geo["objEbY"] + 14)
    ev += _ziehen(10, *_karte_punkt(geo, 16, 16), *_karte_punkt(geo, 48, 32))
    ev += _klick(20, geo["codeX"] + 52, geo["codeY"] + 16)
    letzte = _lauf(tmp_path, 40, ev, dialoge=_mit_objnamen("held"))[-1]
    assert letzte["objAnz"] == 1
    obj = _objekte_von(tmp_path)
    assert len(obj) == 1
    assert (obj[0].x, obj[0].y, obj[0].width, obj[0].height) == (16, 16, 32, 16)
    assert obj[0].name == "held" and obj[0].type == "spawn"


def test_klicken_ohne_zug_legt_einen_punkt_an(tmp_path):
    """In Tiled ist ein Objekt mit Breite und Hoehe 0 ein PUNKT -- ein Klick
    legt also bewusst einen an, keinen unsichtbaren Kasten. Der fremde Leser
    sagt genau das (`is_point`)."""
    geo = _lauf(tmp_path, 6)[-1]
    ev = _klick(3, geo["objEbX"] + 70, geo["objEbY"] + 14)
    ev += _klick(10, *_karte_punkt(geo, 32, 32))
    ev += _klick(18, geo["codeX"] + 52, geo["codeY"] + 16)
    _lauf(tmp_path, 36, ev, dialoge=_mit_objnamen("marke", typ="trigger"))
    obj = _objekte_von(tmp_path)
    assert len(obj) == 1 and obj[0].is_point()
    assert (obj[0].x, obj[0].y) == (32, 32)


def test_klick_auf_ein_objekt_waehlt_es_statt_ein_neues_anzulegen(tmp_path):
    """Sonst legte jeder Versuch, eins anzufassen, ein weiteres darueber --
    und man kaeme nie wieder an das erste heran."""
    geo = _lauf(tmp_path, 6)[-1]
    ev = _klick(3, geo["objEbX"] + 70, geo["objEbY"] + 14)
    ev += _ziehen(10, *_karte_punkt(geo, 16, 16), *_karte_punkt(geo, 48, 48))
    ev += _klick(20, *_karte_punkt(geo, 32, 32))       # mitten hinein
    letzte = _lauf(tmp_path, 36, ev, dialoge=_mit_objnamen("held"))[-1]
    assert letzte["objAnz"] == 1, "kein zweites Objekt"
    assert letzte["objSel"] == 0, "sondern das vorhandene gewaehlt"


def test_entf_entfernt_das_gewaehlte_objekt(tmp_path):
    geo = _lauf(tmp_path, 6)[-1]
    ev = _klick(3, geo["objEbX"] + 70, geo["objEbY"] + 14)
    ev += _ziehen(10, *_karte_punkt(geo, 16, 16), *_karte_punkt(geo, 48, 48))
    ev += [(20, KEY_DOWN, TASTE_ENTF), (21, KEY_UP, TASTE_ENTF)]
    letzte = _lauf(tmp_path, 36, ev, dialoge=_mit_objnamen("held"))[-1]
    assert letzte["objAnz"] == 0 and letzte["objSel"] == -1


def test_die_objekte_landen_in_der_datei(tmp_path):
    """Der ganze Weg: anlegen, speichern, und der Qt-Editor findet sie --
    mit Name, Typ und Rechteck."""
    geo = _lauf(tmp_path, 6)[-1]
    ev = _klick(3, geo["objEbX"] + 70, geo["objEbY"] + 14)
    ev += _ziehen(10, *_karte_punkt(geo, 0, 0), *_karte_punkt(geo, 16, 16))
    ev += _ziehen(20, *_karte_punkt(geo, 32, 32), *_karte_punkt(geo, 64, 48))
    ev += _klick(30, geo["codeX"] + 52, geo["codeY"] + 16)
    _lauf(tmp_path, 50, ev, dialoge=_mit_objnamen("gegner", typ="feind"))
    obj = _objekte_von(tmp_path)
    assert len(obj) == 2
    assert all(o.name == "gegner" and o.type == "feind" for o in obj)
    assert sorted((o.x, o.y) for o in obj) == [(0, 0), (32, 32)]


# ------------------------------------------------------------ mehrere Tilesets
# Eine Karte darf mehrere Tilesets mitbringen. Der Editor haengt sie an
# ([+]), zeigt eines in der Palette und muss beim ZEICHNEN jede Kachel ihrem
# eigenen zuordnen -- mit dem aktiven gerechnet saehe die halbe Karte falsch
# aus. Der Dialog liefert den Pfad; alles andere macht der echte Code.
_DIALOG_TS = 'FILE_OPEN_DIALOG("Tileset hinzufuegen", "png")'


def _mit_tileset(datei, weiteres=None):
    d = {_DIALOG_TS: '"%s"' % datei, _DIALOG_CODE: '"raus.dh"'}
    d.update(weiteres or {})
    return d


def _tileset_daneben(tmp_path, name="tiles.png"):
    """Ein zweites Tileset in den Arbeitsordner legen (4x2 Kacheln)."""
    quelle = _ROOT / "examples" / "assets" / name
    (tmp_path / name).write_bytes(quelle.read_bytes())
    return name


def _tileset_bauen(tmp_path, name, spalten, zeilen):
    """Ein Tileset beliebiger Groesse -- jede Kachel eine eigene Farbe, damit
    sich im Bild nachsehen laesst, WELCHE gewaehlt wurde."""
    from PIL import Image
    im = Image.new("RGB", (spalten * 16, zeilen * 16))
    for zy in range(zeilen):
        for zx in range(spalten):
            i = zy * spalten + zx
            for py in range(16):
                for px in range(16):
                    im.putpixel((zx * 16 + px, zy * 16 + py),
                                (20 + i * 3, 200 - i * 2, 60 + i))
    im.save(tmp_path / name)
    return name


def test_ein_zweites_tileset_haengt_sich_hinter_das_erste(tmp_path):
    """Die firstgid vergibt die LAUFZEIT -- der Editor rechnet sie nicht
    nach. Ueberlappende Bereiche zerstoeren sonst stumm die Zuordnung aller
    Kacheln. Gegenprobe im Test: vorher steht sie bei 1."""
    datei = _tileset_daneben(tmp_path)
    geo = _lauf(tmp_path, 6)[-1]
    assert geo["tsAnz"] == 1 and geo["basisGid"] == 1, "vorher nur das eingebaute"
    ev = _klick(3, geo["tsNeuX"] + 16, geo["tsNeuY"] + 13)
    letzte = _lauf(tmp_path, 20, ev, dialoge=_mit_tileset(datei))[-1]
    assert letzte["tsAnz"] == 2, "das zweite ist da"
    assert letzte["tsAkt"] == 1, "und die Palette zeigt es"
    # 8x4 Kacheln im eingebauten Tileset -> das zweite beginnt bei 33.
    assert letzte["basisGid"] == 33


def test_kacheln_beider_tilesets_stehen_in_derselben_ebene(tmp_path):
    """Der eigentliche Punkt: eine Ebene darf Kacheln aus allen Tilesets
    enthalten. Gelesen wird mit dem FREMDEN Leser -- er loest die GID genau
    wie Tiled auf, und nur so ist belegt, dass die Datei stimmt."""
    datei = _tileset_daneben(tmp_path)
    geo = _lauf(tmp_path, 6)[-1]
    ev = _klick(3, *_karte_punkt(geo, 16, 16))              # aus dem ersten
    ev += _klick(9, geo["tsNeuX"] + 16, geo["tsNeuY"] + 13)  # zweites anhaengen
    ev += _klick(18, geo["palX"] + 48, geo["palY"] + 16)     # dessen Kachel 1
    ev += _klick(24, *_karte_punkt(geo, 80, 16))
    ev += _klick(30, geo["codeX"] + 52, geo["codeY"] + 16)
    _lauf(tmp_path, 46, ev, dialoge=_mit_tileset(datei))

    from drachenhauch.tilemap.document import TileMapDoc
    doc = TileMapDoc.load_json(tmp_path / "raus.json")
    assert len(doc.tilesets) == 2
    links = doc.layers[0].get(1, 1)
    rechts = doc.layers[0].get(5, 1)
    assert doc.gid_to_tileset(links) == (0, 0)
    assert doc.gid_to_tileset(rechts) == (1, 1)


def test_die_pipette_schaltet_das_tileset_um(tmp_path):
    """Ohne das zeigte die Palette die aufgenommene Kachel gar nicht -- und
    der naechste Strich malte eine andere."""
    datei = _tileset_daneben(tmp_path)
    geo = _lauf(tmp_path, 6)[-1]
    ev = _klick(3, *_karte_punkt(geo, 16, 16))              # Kachel aus ts 0
    ev += _klick(9, geo["tsNeuX"] + 16, geo["tsNeuY"] + 13)  # ts 1 wird aktiv
    ev += [(16, KEY_DOWN, TASTE_I), (17, KEY_UP, TASTE_I)]   # Pipette
    ev += _klick(22, *_karte_punkt(geo, 16, 16))
    letzte = _lauf(tmp_path, 40, ev, dialoge=_mit_tileset(datei))[-1]
    assert letzte["tsAkt"] == 0, "die Pipette holt das Tileset ihrer Kachel"
    assert letzte["basisGid"] == 1


def test_der_erzeugte_renderer_zeichnet_beide_tilesets(tmp_path):
    """Er laedt ein Blatt je Tileset und loest jede GID einzeln auf. Mit nur
    einem Blatt bliebe die zweite Kachel leer -- und der Code uebersetzte
    genauso. Darum wird er GESTARTET und sein Bild angesehen."""
    pytest.importorskip("PIL")
    datei = _tileset_daneben(tmp_path)
    geo = _lauf(tmp_path, 6)[-1]
    ev = _klick(3, *_karte_punkt(geo, 16, 16))
    ev += _klick(9, geo["tsNeuX"] + 16, geo["tsNeuY"] + 13)
    ev += _klick(18, geo["palX"] + 48, geo["palY"] + 16)
    ev += _klick(24, *_karte_punkt(geo, 80, 16))
    ev += _klick(30, geo["codeX"] + 52, geo["codeY"] + 16)
    _lauf(tmp_path, 46, ev, dialoge=_mit_tileset(datei))
    assert (tmp_path / "raus_1.png").exists() and (tmp_path / "raus_2.png").exists()

    from PIL import Image
    _rendern(tmp_path)
    im = Image.open(tmp_path / "gerendert.png").convert("RGB")
    hintergrund = im.getpixel((300, 300))
    links = im.getpixel((24, 24))
    rechts = im.getpixel((88, 24))
    assert links != hintergrund, "die Kachel aus dem ersten Tileset fehlt"
    assert rechts != hintergrund, "die Kachel aus dem zweiten Tileset fehlt"
    assert links != rechts, "beide zeigen dasselbe -- die GID wurde falsch aufgeloest"


def test_ein_grosses_tileset_laesst_sich_rollen(tmp_path):
    """Die Palette ist ein Ausschnitt fester Groesse. Ohne Rollen waere von
    einem 10x6-Tileset die Haelfte unerreichbar -- ohne dass etwas fehlte."""
    pytest.importorskip("PIL")
    datei = _tileset_bauen(tmp_path, "gross.png", 10, 6)
    geo = _lauf(tmp_path, 6)[-1]
    ev = _klick(3, geo["tsNeuX"] + 16, geo["tsNeuY"] + 13)
    # Das Rad wirkt nur UEBER der Palette -- der Zeiger steht nach dem
    # Klick noch auf dem Knopf.
    ev += [(10, MOUSE_POSITION, geo["palX"] + 8, geo["palY"] + 8)]
    # Zwei Zeilen nach unten, dann die erste Kachel des Ausschnitts.
    ev += [(12, MAUSRAD, 0, -1), (14, MAUSRAD, 0, -1)]
    ev += _klick(18, geo["palX"] + 8, geo["palY"] + 8)
    letzte = _lauf(tmp_path, 34, ev, dialoge=_mit_tileset(datei))[-1]
    assert letzte["gewaehlt"] == 20, "Zeile 2, Spalte 0 eines 10 Spalten breiten"


def test_umschalt_rollt_die_palette_zur_seite(tmp_path):
    """Ein breites Tileset braucht die zweite Richtung -- sonst waeren die
    Spalten ab 8 nicht anzuklicken."""
    pytest.importorskip("PIL")
    datei = _tileset_bauen(tmp_path, "breit.png", 10, 6)
    geo = _lauf(tmp_path, 6)[-1]
    ev = _klick(3, geo["tsNeuX"] + 16, geo["tsNeuY"] + 13)
    ev += [(10, MOUSE_POSITION, geo["palX"] + 8, geo["palY"] + 8)]
    ev += [(12, KEY_DOWN, TASTE_UMSCHALT), (14, KEY_DOWN, TASTE_UMSCHALT),
           (16, KEY_DOWN, TASTE_UMSCHALT)]
    ev += [(14, MAUSRAD, 0, -1), (16, MAUSRAD, 0, -1)]
    ev += [(18, KEY_UP, TASTE_UMSCHALT)]
    ev += _klick(22, geo["palX"] + 8, geo["palY"] + 8)
    letzte = _lauf(tmp_path, 38, ev, dialoge=_mit_tileset(datei))[-1]
    assert letzte["gewaehlt"] == 2, "Zeile 0, Spalte 2"
