"""imgfx-Erweiterungen: nur Registrierung.

Die Filter (IMAGE_CROP/RESIZE_CANVAS/BLUR/BRIGHTNESS/CONTRAST/GRAYSCALE/
INVERT/REPLACE_COLOR) und die In-Image-Zeichenops (IMAGE_DRAW_LINE/CIRCLE/
RECT/TEXT) sind raylib-Engine-Builtins: sie brauchen den GL-Kontext (Image
<-> Textur) und werden daher nicht via run_gb funktionsgetestet, sondern live
(examples/122_imgfx.gb). Hier wird geprueft, dass sie im eingefrorenen
dhrt-Index stehen -- sonst warnt der Editor live und der Drift-Test schlaegt
an, sobald das Beispiel sie nutzt.
"""
from drachenhauch.editor_qt.dhrt_meta import builtin_names_lower


def test_imgfx_extension_builtins_registered():
    n = builtin_names_lower()
    for name in ("image_crop", "image_resize_canvas", "image_blur",
                 "image_brightness", "image_contrast", "image_grayscale",
                 "image_invert", "image_replace_color",
                 "image_draw_line", "image_draw_circle",
                 "image_draw_rect", "image_draw_text"):
        assert name in n, name
