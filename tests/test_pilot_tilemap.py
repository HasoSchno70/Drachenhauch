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
          " " + STR$(kb) + " " + STR$(kh) + " " + STR$(zell())
'''
_FELDER = "codeX codeY kb kh zell".split()

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
