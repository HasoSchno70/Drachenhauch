"""Render-Targets, die ueber das Bild hinaus stehen bleiben.

Voreinstellung bleibt: ein Target wird vor jedem Bild transparent geleert --
richtig fuer "Szene zwischenspeichern". `RENDERTARGET_NEW(w, h, TRUE)` laesst
den Inhalt dagegen stehen, und genau das ist die Voraussetzung fuer
Rueckkopplung: Schweife, Nachzieher, der klassische Demo-Feedback-Effekt.

Geprueft wird am Bild (Screenshot + Pixel) -- anders ist "steht da noch etwas"
nicht zu beantworten.
"""
import pytest

pytest.importorskip("PIL", reason="Pillow noetig zum Pixel-Pruefen")


def _pixel(tmp_path, name, punkt):
    from PIL import Image

    with Image.open(tmp_path / name) as img:
        return img.convert("RGB").getpixel(punkt)


def _quelle(behalten: str, extra: str = "") -> str:
    """Zeichnet in Bild 0 einen Punkt links, danach nur noch rechts."""
    return f"""
SCREEN(64, 64, "rt", 1)
DIM rt AS INTEGER
rt = RENDERTARGET_NEW(64, 64{behalten})
DIM f AS INTEGER
FOR f = 0 TO 5
    RENDERTARGET_BEGIN(rt)
    {extra}
    IF f = 0 THEN PLOT(10, 10, &HFF0000)
    PLOT(40, 10, &H00FF00)
    RENDERTARGET_END()
    CLS(0)
    RENDERTARGET_DRAW(rt, 0, 0)
    FLIP()
NEXT
SAVESCREENSHOT("out.png")
"""


def test_ohne_behalten_ist_der_alte_inhalt_weg(run_gb, tmp_path):
    run_gb(_quelle(""), base=tmp_path)
    assert _pixel(tmp_path, "out.png", (10, 10)) == (0, 0, 0)
    assert _pixel(tmp_path, "out.png", (40, 10)) == (0, 255, 0)


def test_mit_behalten_steht_der_alte_inhalt_noch_da(run_gb, tmp_path):
    run_gb(_quelle(", TRUE"), base=tmp_path)
    assert _pixel(tmp_path, "out.png", (10, 10)) == (255, 0, 0), \
        "Der Punkt aus Bild 0 muss stehen geblieben sein"
    assert _pixel(tmp_path, "out.png", (40, 10)) == (0, 255, 0)


def test_rendertarget_clear_raeumt_ein_behaltenes_target(run_gb, tmp_path):
    # CLEAR wirkt auf das ganze Bild -- auch auf das, was vorher schon
    # hineingezeichnet wurde.
    run_gb(_quelle(", TRUE", "IF f = 3 THEN RENDERTARGET_CLEAR(rt)"), base=tmp_path)
    assert _pixel(tmp_path, "out.png", (10, 10)) == (0, 0, 0)
    assert _pixel(tmp_path, "out.png", (40, 10)) == (0, 255, 0)


def test_verblassen_ueber_multiplizieren(run_gb, tmp_path):
    # Das uebliche Rezept fuer Schweife: den Inhalt jedes Bild mit einem
    # dunklen Grau multiplizieren. Ein Target kann sich NICHT selbst
    # zeichnen, dieser Weg ist also der richtige.
    run_gb("""
SCREEN(64, 64, "rt", 1)
DIM rt AS INTEGER
rt = RENDERTARGET_NEW(64, 64, TRUE)
DIM f AS INTEGER
FOR f = 0 TO 9
    RENDERTARGET_BEGIN(rt)
    BLEND_MODE("mult")
    BOX(0, 0, 64, 64, RGB(200, 200, 200))
    BLEND_MODE("alpha")
    IF f = 0 THEN PLOT(10, 10, &HFF0000)
    RENDERTARGET_END()
    CLS(0)
    RENDERTARGET_DRAW(rt, 0, 0)
    FLIP()
NEXT
SAVESCREENSHOT("out.png")
""", base=tmp_path)
    r, g, b = _pixel(tmp_path, "out.png", (10, 10))
    assert 0 < r < 255, f"Der Punkt muss verblasst, aber noch da sein (rot={r})"
    assert g == 0 and b == 0


def test_inhalt_wird_nicht_vom_fenster_massstab_verzerrt(run_gb, tmp_path):
    # Ein Render-Target hat seine EIGENE Pixelgroesse. Wurde der Inhalt mit dem
    # Fenster-Massstab hineingezeichnet (SCREEN(..., 2) -> doppelt so gross),
    # fiel alles rechts/unten davon weg -- und in einem behaltenen Target
    # blieben die abgeschnittenen Raender stehen.
    run_gb("""
SCREEN(64, 64, "rt", 2)
DIM rt AS INTEGER
rt = RENDERTARGET_NEW(64, 64)
DIM f AS INTEGER
FOR f = 0 TO 3
    RENDERTARGET_BEGIN(rt)
    PLOT(50, 50, &HFF0000)
    RENDERTARGET_END()
    CLS(0)
    RENDERTARGET_DRAW(rt, 0, 0)
    FLIP()
NEXT
SAVESCREENSHOT("out.png")
""", base=tmp_path)
    # Fenster ist 128x128 (Massstab 2), der Punkt liegt im Target bei 50,50
    # -> auf dem Schirm bei 100,100. Mit dem alten Fehler waere er bei 100,100
    # gar nicht angekommen (im Target auf 100,100 = ausserhalb geclippt).
    from PIL import Image
    with Image.open(tmp_path / "out.png") as img:
        rgb = img.convert("RGB")
        assert rgb.size == (128, 128)
        assert rgb.getpixel((100, 100)) == (255, 0, 0)
