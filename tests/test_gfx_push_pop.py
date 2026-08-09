"""GFX_PUSH / GFX_POP -- Zeichenzustand sichern und zurueckholen.

Licht, Nebel, Himmel, Schatten, Kamera, Schrift und der Post-Effekt sind
GLOBAL. Wer sie in einer Szene umstellt, muss sie beim Verlassen von Hand
zurueckstellen -- und eine vergessene Zeile faellt erst zwei Szenen spaeter auf
(der HDR-Himmel hinter dem naechsten Bild, der Nebel in der falschen Szene).
Mit PUSH/POP wird daraus eine Eigenschaft der Sprache statt einer
Disziplinfrage.

Der wichtigste Test hier ist der am BILD: zweimal dasselbe zeichnen, dazwischen
den Zustand kraeftig verstellen und wieder zurueckholen -- beide Bilder muessen
Pixel fuer Pixel gleich sein. Zahlen zurueckzugeben ist leicht; dass die Optik
wirklich wieder stimmt, zeigt nur ein Vergleich.
"""
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


def _run(tmp_path, quelle: str):
    (tmp_path / "a.gb").write_text(quelle, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(tmp_path / "a.gb")], capture_output=True,
                       text=True, encoding="utf-8", timeout=120, cwd=str(tmp_path))
    return r


def test_kamera_schrift_und_stapeltiefe(tmp_path):
    r = _run(tmp_path, """
IMPORT "camera"
SCREEN(120, 80, "p", 1)
PRINT GFX_DEPTH()
GFX_PUSH()
CAMERA_SET(120.0, 80.0, 2.0)
PRINT STR$(INT(CAMERA_X())) + " " + FORMAT$(CAMERA_ZOOM(), "%.1f") + " " + STR$(GFX_DEPTH())
GFX_POP()
PRINT STR$(INT(CAMERA_X())) + " " + FORMAT$(CAMERA_ZOOM(), "%.1f") + " " + STR$(GFX_DEPTH())
""")
    assert r.returncode == 0, r.stderr
    zeilen = [z for z in r.stdout.splitlines() if z and not z.startswith(("INFO", "WARNING", "TRACE"))]
    assert zeilen == ["0", "120 2.0 1", "0 1.0 0"]


def test_pop_ohne_push_meldet_sich(tmp_path):
    r = _run(tmp_path, 'SCREEN(64, 64, "p", 1)\nGFX_POP()\n')
    assert r.returncode != 0
    assert "GFX_POP" in r.stderr and "leer" in r.stderr


def test_verschachteln_geht(tmp_path):
    r = _run(tmp_path, """
IMPORT "camera"
SCREEN(64, 64, "p", 1)
GFX_PUSH()
CAMERA_SET(10.0, 0.0, 1.0)
GFX_PUSH()
CAMERA_SET(99.0, 0.0, 1.0)
GFX_POP()
PRINT INT(CAMERA_X())
GFX_POP()
PRINT INT(CAMERA_X())
""")
    assert r.returncode == 0, r.stderr
    zeilen = [z for z in r.stdout.splitlines() if z.strip().isdigit()]
    assert zeilen == ["10", "0"]


@pytest.mark.parametrize("stoerung", [
    'LIGHT_AMBIENT(&HFF0000, 1.0)',
    'LIGHT_FOG(&HFF0000, 0.5)',
    'LIGHT_DIRECTIONAL(1.0, 0.0, 0.0, &HFF0000)',
    'LIGHT_ENV(&HFF0000, &HFF0000, 2.0)',
])
def test_bild_ist_nach_pop_wieder_identisch(tmp_path, stoerung):
    """Der eigentliche Beweis: zweimal dasselbe zeichnen, dazwischen den
    Zustand verstellen und zurueckholen -- die Bilder muessen gleich sein."""
    pytest.importorskip("PIL", reason="Pillow noetig zum Pixel-Vergleich")
    from PIL import Image, ImageChops

    r = _run(tmp_path, f"""
IMPORT "g3d"
SCREEN(160, 120, "p", 1)
LIGHT_ENABLE()
LIGHT_AMBIENT(&H203040, 0.5)
LIGHT_DIRECTIONAL(-0.5, -0.8, -0.4, &HFFE0B0)
DIM kugel AS INTEGER
kugel = MESH_SPHERE(1.0, 24, 24)
MODEL_LIT(kugel)

SUB zeichne()
    DIM f AS INTEGER
    FOR f = 0 TO 3
        CLS(&H101018)
        CAMERA3D(0, 1, 4, 0, 0, 0, 45)
        MODEL(kugel, 0, 0, 0, 1.0, &HD0D0D0)
        FLIP()
    NEXT
END SUB

zeichne()
SAVESCREENSHOT("vorher.png")

GFX_PUSH()
{stoerung}
zeichne()
GFX_POP()

zeichne()
SAVESCREENSHOT("nachher.png")
""")
    assert r.returncode == 0, r.stderr
    with Image.open(tmp_path / "vorher.png") as a, Image.open(tmp_path / "nachher.png") as b:
        diff = ImageChops.difference(a.convert("RGB"), b.convert("RGB"))
    assert diff.getbbox() is None, \
        f"Bild nach GFX_POP unterscheidet sich -- '{stoerung}' wurde nicht zurueckgenommen"


def test_posteffekt_wird_beim_pop_abgeloest(tmp_path):
    """Der Post-Effekt liegt ueber ALLEM -- bleibt er nach dem POP haengen,
    faerbt der Shader einer verlassenen Szene den Rest der Demo ein.

    Der Test hat bewusst eine Kontrolle: das Bild WAEHREND des PUSH muss sich
    unterscheiden. Sonst wuerde er auch dann gruen sein, wenn POSTFX gar nichts
    getan haette."""
    pytest.importorskip("PIL", reason="Pillow noetig zum Pixel-Vergleich")
    from PIL import Image, ImageChops

    (tmp_path / "rot.fs").write_text(
        "#version 330\n"
        "in vec2 fragTexCoord;\nuniform sampler2D texture0;\nout vec4 finalColor;\n"
        "void main() { finalColor = vec4(1.0, 0.0, 0.0, 1.0); }\n", encoding="utf-8")

    r = _run(tmp_path, """
SCREEN(160, 120, "p", 1)
DIM sh AS INTEGER
sh = SHADER_LOAD("rot.fs")

SUB zeichne()
    DIM f AS INTEGER
    FOR f = 0 TO 3
        CLS(&H101018)
        BOX(30, 30, 130, 90, &H40C0FF)
        FLIP()
    NEXT
END SUB

zeichne()
SAVESCREENSHOT("vorher.png")

GFX_PUSH()
POSTFX(sh)
zeichne()
SAVESCREENSHOT("waehrend.png")
GFX_POP()

zeichne()
SAVESCREENSHOT("nachher.png")
""")
    assert r.returncode == 0, r.stderr
    with Image.open(tmp_path / "vorher.png") as a, \
         Image.open(tmp_path / "waehrend.png") as m, \
         Image.open(tmp_path / "nachher.png") as b:
        a, m, b = a.convert("RGB"), m.convert("RGB"), b.convert("RGB")
        wirkte = ImageChops.difference(a, m).getbbox() is not None
        gleich = ImageChops.difference(a, b).getbbox() is None
    assert wirkte, "POSTFX hat gar nicht gewirkt -- der Test wuerde nichts beweisen"
    assert gleich, "Post-Effekt liegt nach GFX_POP immer noch ueber dem Bild"


def test_zusaetzliches_licht_wird_beim_pop_abgeschaltet(tmp_path):
    # Eine Szene darf ein Licht HINZUFUEGEN. POP muss es wieder ausschalten,
    # sonst leuchtet es in der naechsten Szene weiter.
    pytest.importorskip("PIL", reason="Pillow noetig zum Pixel-Vergleich")
    from PIL import Image, ImageChops

    r = _run(tmp_path, """
IMPORT "g3d"
SCREEN(160, 120, "p", 1)
LIGHT_ENABLE()
LIGHT_AMBIENT(&H203040, 0.4)
DIM kugel AS INTEGER
kugel = MESH_SPHERE(1.0, 24, 24)
MODEL_LIT(kugel)

SUB zeichne()
    DIM f AS INTEGER
    FOR f = 0 TO 3
        CLS(&H101018)
        CAMERA3D(0, 1, 4, 0, 0, 0, 45)
        MODEL(kugel, 0, 0, 0, 1.0, &HD0D0D0)
        FLIP()
    NEXT
END SUB

zeichne()
SAVESCREENSHOT("vorher.png")

GFX_PUSH()
LIGHT_POINT(2.0, 2.0, 2.0, &HFF0000)     ' NEUES Licht in der Szene
zeichne()
GFX_POP()

zeichne()
SAVESCREENSHOT("nachher.png")
""")
    assert r.returncode == 0, r.stderr
    with Image.open(tmp_path / "vorher.png") as a, Image.open(tmp_path / "nachher.png") as b:
        diff = ImageChops.difference(a.convert("RGB"), b.convert("RGB"))
    assert diff.getbbox() is None, "das in der Szene hinzugefuegte Licht leuchtet nach GFX_POP weiter"


# ---------------------------------------------------------- AUDIO_PUSH/POP
def test_audio_lautstaerke_kommt_zurueck(tmp_path):
    r = _run(tmp_path, """
IMPORT "audio"
SCREEN(64, 64, "p", 1)
AUDIO_BUS_VOLUME("music", 0.8)
PRINT FORMAT$(AUDIO_BUS_GET_VOLUME("music"), "%.2f") + " " + STR$(AUDIO_DEPTH())
AUDIO_PUSH()
AUDIO_BUS_VOLUME("music", 0.1)
PRINT FORMAT$(AUDIO_BUS_GET_VOLUME("music"), "%.2f") + " " + STR$(AUDIO_DEPTH())
AUDIO_POP()
PRINT FORMAT$(AUDIO_BUS_GET_VOLUME("music"), "%.2f") + " " + STR$(AUDIO_DEPTH())
""")
    assert r.returncode == 0, r.stderr
    zeilen = [z for z in r.stdout.splitlines() if z and z[0].isdigit()]
    assert zeilen == ["0.80 0", "0.10 1", "0.80 0"]


def test_audio_pop_ohne_push_meldet_sich(tmp_path):
    r = _run(tmp_path, 'IMPORT "audio"\nSCREEN(64, 64, "p", 1)\nAUDIO_POP()\n')
    assert r.returncode != 0
    assert "AUDIO_POP" in r.stderr and "leer" in r.stderr


def test_filter_wirkt_nach_pop_nicht_mehr(tmp_path):
    """Am KLANG gemessen, nicht an einer Zahl: ein Tiefpass in der Szene
    schneidet die Hoehen weg -- nach AUDIO_POP muessen sie zurueck sein."""
    r = _run(tmp_path, """
IMPORT "audio"
SCREEN(64, 64, "p", 1)
DIM rauschen AS SOUND
rauschen = AUDIO_NOISE(4000, 0.7)
DIM spek[32] AS FLOAT

' MITTELWERT, nicht Spitze: eine Spitze ueber das Fenster faengt die
' ungefilterten Augenblicke direkt nach dem Umschalten mit ein und
' verwaessert die Messung (gemessen: Faktor 1,3 statt 8).
FUNCTION hoehen() AS FLOAT
    DIM t0 AS INTEGER : t0 = MILLIS()
    DIM summe AS FLOAT : summe = 0.0
    DIM n AS INTEGER : n = 0
    WHILE MILLIS() - t0 < 900
        FLIP()
        AUDIO_FFT(spek)
        DIM s AS FLOAT : s = 0.0
        DIM k AS INTEGER
        FOR k = 20 TO 31
            s = s + spek[k]
        NEXT
        summe = summe + s
        n = n + 1
    WEND
    RETURN summe / n
END FUNCTION

AUDIO_PLAY(rauschen, -1, 0.9)
PRINT "offen:    " + FORMAT$(hoehen(), "%.2f")
AUDIO_PUSH()
AUDIO_FILTER("sfx", 300, 0.0)
PRINT "gefiltert:" + FORMAT$(hoehen(), "%.2f")
AUDIO_POP()
PRINT "nach POP: " + FORMAT$(hoehen(), "%.2f")
""")
    assert r.returncode == 0, r.stderr
    werte = [float(z.split(":")[1]) for z in r.stdout.splitlines() if ":" in z and z[0] in "ogn"]
    assert len(werte) == 3, r.stdout
    offen, gefiltert, nach_pop = werte
    assert gefiltert < offen * 0.3, f"Filter wirkte nicht (offen {offen}, gefiltert {gefiltert})"
    assert nach_pop > offen * 0.7, f"Filter blieb nach AUDIO_POP haengen ({nach_pop} statt ~{offen})"


def test_laufende_modulation_wird_beim_pop_abgeloest(tmp_path):
    """Belegt eine Aussage aus der Doku: `AUDIO_MODULATE` schreibt denselben
    Kira-Parameter wie die normalen Setter -- das Zurueckschreiben beim POP
    loest eine laufende Modulation also ab.

    Gemessen wird die SCHWANKUNG der Hoehen ueber gut eine Sekunde: mit einem
    LFO auf dem Filter schwankt sie stark, mit festem Wert kaum. Vor der ersten
    Messung eine Sekunde Vorlauf -- sonst faellt das Einschwingen des Klangs in
    die Messung und blaeht sie auf.
    """
    r = _run(tmp_path, """
IMPORT "audio"
SCREEN(64, 64, "p", 1)
DIM rauschen AS SOUND
rauschen = AUDIO_NOISE(8000, 0.7)
DIM spek[32] AS FLOAT

FUNCTION streuung() AS FLOAT
    DIM t0 AS INTEGER : t0 = MILLIS()
    DIM mini AS FLOAT : mini = 999.0
    DIM maxi AS FLOAT : maxi = 0.0
    WHILE MILLIS() - t0 < 1400
        FLIP()
        AUDIO_FFT(spek)
        DIM s AS FLOAT : s = 0.0
        DIM k AS INTEGER
        FOR k = 20 TO 31
            s = s + spek[k]
        NEXT
        IF s < mini THEN mini = s
        IF s > maxi THEN maxi = s
    WEND
    RETURN maxi - mini
END FUNCTION

AUDIO_PLAY(rauschen, -1, 0.9)
DIM t AS INTEGER : t = MILLIS()
WHILE MILLIS() - t < 1000
    FLIP()
WEND

AUDIO_PUSH()
DIM lfo AS AUDIO_MOD
lfo = AUDIO_LFO_NEW("sine", 1.5)
AUDIO_MODULATE("sfx", "filter", lfo, 200.0, 12000.0)
PRINT "lfo " + FORMAT$(streuung(), "%.2f")
AUDIO_POP()
PRINT "pop " + FORMAT$(streuung(), "%.2f")
""")
    assert r.returncode == 0, r.stderr
    werte = {z.split()[0]: float(z.split()[1]) for z in r.stdout.splitlines()
             if z.startswith(("lfo ", "pop "))}
    assert len(werte) == 2, r.stdout
    assert werte["pop"] < werte["lfo"] * 0.5, \
        f"Modulation laeuft nach AUDIO_POP weiter (LFO {werte['lfo']}, nach POP {werte['pop']})"
