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

MOUSE_BUTTON_UP, MOUSE_BUTTON_DOWN, MOUSE_POSITION = 5, 6, 7

# Nur was dieser Test braucht: die Lage des Knopfes und die Kartenmasse.
_PROBE = '''    PRINT "P " + STR$(GUI_GET_X(bCode)) + " " + STR$(GUI_GET_Y(bCode)) + _
          " " + STR$(kb) + " " + STR$(kh) + " " + STR$(zell()) + _
          " " + STR$(GUI_GET_X(cbSolid)) + " " + STR$(GUI_GET_Y(cbSolid)) + _
          " " + STR$(GUI_GET_X(bPropSetz)) + " " + STR$(GUI_GET_Y(bPropSetz)) + _
          " " + STR$(GUI_GET_X(bPropWeg)) + " " + STR$(GUI_GET_Y(bPropWeg)) + _
          " " + STR$(GUI_GET_X(lstProp)) + " " + STR$(GUI_GET_Y(lstProp)) + _
          " " + STR$(gewaehlt)
'''
_FELDER = ("codeX codeY kb kh zell solidX solidY setzX setzY "
           "wegX wegY propX propY gewaehlt").split()

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
    return tmp_path / "raus.dh", tmp_path / "raus.json", tmp_path / "raus.png"


def test_gb_code_bringt_karte_und_tileset_mit(tmp_path):
    """Alle drei Dateien, unter demselben Namen. Fehlt eine, zeigt der Code
    ins Leere -- und das faellt erst beim Starten auf."""
    code, karte, tileset = _exportieren(tmp_path)
    assert code.exists() and karte.exists() and tileset.exists()
    text = code.read_text(encoding="utf-8")
    assert 'LOADIMAGE("raus.png")' in text
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
    # Zweite Palettenkachel waehlen (die Palette sitzt bei (12, 78),
    # jede Kachel ist 16*2 Punkte gross).
    ev = _klick(3, 12 + 48, 78 + 16)
    ev += _klick(9, geo["setzX"] + 50, geo["setzY"] + 14)
    ev += _klick(15, geo["codeX"] + 52, geo["codeY"] + 16)
    proben = _lauf(tmp_path, 30, ev, dialoge=_mit_feldern("art", "wasser"))
    assert proben[-1]["gewaehlt"] == 1, "die zweite Kachel ist gewaehlt"
    assert _eigenschaften_von(tmp_path, 1) == {"art": "wasser"}
    assert _eigenschaften_von(tmp_path, 0) == {}, "Kachel 0 bleibt unberuehrt"
