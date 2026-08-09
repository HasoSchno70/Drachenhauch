"""Stueckzahl der Bulk-Zeichenbefehle (PLOTS/LINES/BOXES/CIRCLES/MODEL_INSTANCED).

Ohne Stueckzahl zeichnen sie das GANZE Array. Wer einen Puffer fest
dimensioniert und pro Bild nur teilweise fuellt, bekam die alten Werte der
restlichen Plaetze mitgezeichnet -- und ein trotzdem mitgegebenes viertes
Argument wurde still ignoriert, die Absicht also lautlos verschluckt.

Geprueft wird am Bild: das Programm schreibt einen Screenshot, der Test liest
die Pixel. Anders ist "wurde das gezeichnet oder nicht" nicht zu beantworten --
GETPIXEL arbeitet auf einem IMAGE, nicht auf dem Bildschirm.

Gezeichnet wird dafuer ein paar Bilder lang: SAVESCREENSHOT liest den
Framebuffer, und der ist direkt nach dem allerersten Swap noch leer.
"""
import pytest

from drachenhauch.errors import DHRuntimeError

pytest.importorskip("PIL", reason="Pillow noetig zum Pixel-Pruefen")

ROT = (255, 0, 0)
SCHWARZ = (0, 0, 0)


def _pixel(tmp_path, punkte):
    """Screenshot lesen und die Farben an `punkte` liefern."""
    from PIL import Image

    with Image.open(tmp_path / "out.png") as img:
        rgb = img.convert("RGB")
        return [rgb.getpixel(p) for p in punkte]


def test_plots_zeichnet_nur_die_angegebene_anzahl(run_gb, tmp_path):
    # Drei Punkte im Puffer, gezeichnet werden sollen zwei.
    run_gb("""
SCREEN(64, 64, "b", 1)
DIM xs[3] AS INTEGER
DIM ys[3] AS INTEGER
xs[0] = 10 : ys[0] = 10
xs[1] = 20 : ys[1] = 10
xs[2] = 30 : ys[2] = 10
DIM f AS INTEGER
FOR f = 0 TO 3
CLS(0)
PLOTS(xs, ys, &HFF0000, 2)
FLIP()
NEXT
SAVESCREENSHOT("out.png")
""", base=tmp_path)
    assert _pixel(tmp_path, [(10, 10), (20, 10), (30, 10)]) == [ROT, ROT, SCHWARZ]


def test_ohne_anzahl_wird_weiterhin_alles_gezeichnet(run_gb, tmp_path):
    # Rueckwaertskompatibilitaet: bestehende Programme ohne Stueckzahl
    # duerfen sich nicht anders verhalten als vorher.
    run_gb("""
SCREEN(64, 64, "b", 1)
DIM xs[3] AS INTEGER
DIM ys[3] AS INTEGER
xs[0] = 10 : ys[0] = 10
xs[1] = 20 : ys[1] = 10
xs[2] = 30 : ys[2] = 10
DIM f AS INTEGER
FOR f = 0 TO 3
CLS(0)
PLOTS(xs, ys, &HFF0000)
FLIP()
NEXT
SAVESCREENSHOT("out.png")
""", base=tmp_path)
    assert _pixel(tmp_path, [(30, 10)]) == [ROT]


def test_anzahl_groesser_als_das_array_wird_gekappt(run_gb, tmp_path):
    # Kappen statt Fehler: eine zu grosse Stueckzahl ist harmlos, ein Absturz
    # mitten in der Bildschleife waere es nicht.
    run_gb("""
SCREEN(64, 64, "b", 1)
DIM xs[2] AS INTEGER
DIM ys[2] AS INTEGER
xs[0] = 5 : ys[0] = 5
xs[1] = 6 : ys[1] = 5
DIM f AS INTEGER
FOR f = 0 TO 3
CLS(0)
PLOTS(xs, ys, &HFF0000, 99)
FLIP()
NEXT
SAVESCREENSHOT("out.png")
""", base=tmp_path)
    assert _pixel(tmp_path, [(6, 5)]) == [ROT]


def test_lines_boxes_circles_nehmen_die_anzahl_auch(run_gb, tmp_path):
    run_gb("""
SCREEN(64, 64, "b", 1)
DIM a[2] AS INTEGER
DIM b[2] AS INTEGER
DIM c[2] AS INTEGER
DIM d[2] AS INTEGER
DIM r[2] AS INTEGER
a[0] = 2  : b[0] = 2  : c[0] = 6  : d[0] = 2  : r[0] = 1
a[1] = 40 : b[1] = 40 : c[1] = 44 : d[1] = 40 : r[1] = 1
DIM f AS INTEGER
FOR f = 0 TO 3
CLS(0)
LINES(a, b, c, d, &HFF0000, 1)
BOXES(a, b, c, d, &HFF0000, 1)
CIRCLES(a, b, r, &HFF0000, 1)
FLIP()
NEXT
SAVESCREENSHOT("out.png")
""", base=tmp_path)
    # Nur der jeweils erste Eintrag darf gezeichnet sein.
    assert _pixel(tmp_path, [(4, 2), (42, 40)]) == [ROT, SCHWARZ]


def test_farb_array_darf_laenger_sein_als_die_stueckzahl(run_gb, tmp_path):
    # Wer einen Puffer fest dimensioniert, dimensioniert auch die Farben fest.
    run_gb("""
SCREEN(64, 64, "b", 1)
DIM xs[3] AS INTEGER
DIM ys[3] AS INTEGER
DIM cs[3] AS INTEGER
xs[0] = 10 : ys[0] = 10 : cs[0] = &HFF0000
xs[1] = 20 : ys[1] = 10 : cs[1] = &HFF0000
xs[2] = 30 : ys[2] = 10 : cs[2] = &HFF0000
DIM f AS INTEGER
FOR f = 0 TO 3
CLS(0)
PLOTS(xs, ys, cs, 1)
FLIP()
NEXT
SAVESCREENSHOT("out.png")
""", base=tmp_path)
    assert _pixel(tmp_path, [(10, 10), (20, 10)]) == [ROT, SCHWARZ]


@pytest.mark.parametrize("aufruf,erwartet", [
    ("PLOTS(xs, ys, &HFF0000, 1, 2)", "zu viele Argumente"),
    ("PLOTS(xs, ys, &HFF0000, -1)", "nicht negativ"),
    ("PLOTS(xs, ys, &HFF0000, 1.5)", "ganze Zahl"),
])
def test_falsche_anzahl_wird_gemeldet(run_gb, tmp_path, aufruf, erwartet):
    src = f"""
SCREEN(64, 64, "b", 1)
DIM xs[3] AS INTEGER
DIM ys[3] AS INTEGER
{aufruf}
"""
    with pytest.raises(DHRuntimeError, match=erwartet):
        run_gb(src, base=tmp_path)
