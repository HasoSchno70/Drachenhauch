"""Shader-Uniforms (Array / zweiter Sampler / Matrix) und Linien-Geometrie
(Etappe 4 des Ausbaus).

Die Shader-Tests rendern wirklich und messen einzelne Pixel -- nur so ist
belegt, dass das Uniform in der GPU ankommt. Sie sind bewusst so gebaut, dass
der erwartete Wert NICHT dem Ergebnis "Uniform gar nicht gesetzt" entspricht
(sonst wuerde der Test auch ohne die Funktion bestehen).
"""
import os
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


def _find_gbrt():
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    return next((_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
                 for v in ("release", "debug")
                 if (_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe).exists()), None)


_GBRT = _find_gbrt()
pytestmark = pytest.mark.skipif(_GBRT is None, reason="native Runtime 'gbrt' nicht gebaut")
Image = pytest.importorskip("PIL.Image", reason="Pillow fuer die Pixel-Pruefung noetig")


def _run(files: dict, main: str, tmp_path, frames=3, shot=None):
    for name, body in files.items():
        (tmp_path / name).write_text(body, encoding="utf-8")
    env = dict(os.environ, GBRT_FRAMES=str(frames))
    if shot:
        env["GBRT_SCREENSHOT"] = str(tmp_path / shot)
    r = subprocess.run([str(_GBRT), "run", str(tmp_path / main)], capture_output=True,
                       text=True, encoding="utf-8", env=env, timeout=90, cwd=str(tmp_path))
    r.out = [w for ln in (r.stdout or "").splitlines()
             if not ln.startswith(("WARNING:", "INFO:", "TRACE:")) for w in ln.split()]
    return r


def _pixel(tmp_path, shot, xy):
    from PIL import Image as PImage
    return PImage.open(tmp_path / shot).convert("RGB").getpixel(xy)


_LOOP = 'WHILE NOT QUITREQUESTED()\n    CLS(&HFFFFFF)\n    FLIP()\nWEND\n'


# ------------------------------------------------------------ Shader-Uniforms
def test_shader_array_uniform_reaches_the_gpu(tmp_path):
    fs = ("#version 330\n"
          "in vec2 fragTexCoord;\nuniform sampler2D texture0;\n"
          "uniform float stufen[4];\nout vec4 finalColor;\n"
          "void main() {\n"
          "  float s = stufen[0] + stufen[1] + stufen[2] + stufen[3];\n"
          "  finalColor = vec4(texture(texture0, fragTexCoord).rgb * s, 1.0);\n}\n")
    gb = ('SCREEN(120, 90, "A", 1)\n'
          'DIM sh AS INTEGER\nsh = SHADER_LOAD("a.fs")\n'
          'DIM st[4] AS FLOAT\n'
          'st[0] = 0.125 : st[1] = 0.125 : st[2] = 0.125 : st[3] = 0.125\n'
          'SHADER_SET_ARRAY(sh, "stufen", st)\nPOSTFX(sh)\n' + _LOOP)
    r = _run({"a.fs": fs, "a.gb": gb}, "a.gb", tmp_path, shot="a.png")
    assert r.returncode == 0, r.stderr
    # Summe 0.5 auf Weiss -> ~127. Ohne gesetztes Array waere die Summe 0
    # (schwarz), der Test ist also trennscharf.
    assert 120 <= _pixel(tmp_path, "a.png", (60, 45))[0] <= 135


def test_second_sampler_reaches_the_gpu(tmp_path):
    # Der eigentliche Knackpunkt: `SetShaderValueTexture` ruft glUniform1i auf
    # dem GERADE AKTIVEN Programm. Ausserhalb von BeginShaderMode landet die
    # Zuweisung am falschen Shader und der Sampler bleibt schwarz.
    fs = ("#version 330\n"
          "in vec2 fragTexCoord;\nuniform sampler2D texture0;\n"
          "uniform sampler2D maske;\nout vec4 finalColor;\n"
          "void main() {\n"
          "  float m = texture(maske, fragTexCoord).r;\n"
          "  finalColor = vec4(texture(texture0, fragTexCoord).rgb * m, 1.0);\n}\n")
    gb = ('SCREEN(120, 90, "M", 1)\n'
          'DIM sh AS INTEGER\nsh = SHADER_LOAD("m.fs")\n'
          'DIM m AS IMAGE\nm = GENTEX_COLOR(64, 64, &H808080)\n'
          'SHADER_SET_TEXTURE(sh, "maske", m)\nPOSTFX(sh)\n' + _LOOP)
    r = _run({"m.fs": fs, "m.gb": gb}, "m.gb", tmp_path, shot="m.png")
    assert r.returncode == 0, r.stderr
    assert 120 <= _pixel(tmp_path, "m.png", (60, 45))[0] <= 135


def test_matrix_uniform_reaches_the_gpu(tmp_path):
    fs = ("#version 330\n"
          "in vec2 fragTexCoord;\nuniform sampler2D texture0;\n"
          "uniform mat4 probe;\nout vec4 finalColor;\n"
          "void main() {\n"
          "  finalColor = vec4(texture(texture0, fragTexCoord).rgb * probe[3][0], 1.0);\n}\n")
    gb = ('IMPORT "m3d"\nSCREEN(120, 90, "M4", 1)\n'
          'DIM sh AS INTEGER\nsh = SHADER_LOAD("t.fs")\n'
          'DIM m AS MAT4\n'
          'm = MAT4_TRS(VEC3_NEW(0.5, 0.0, 0.0), QUAT_IDENTITY(), VEC3_NEW(1.0, 1.0, 1.0))\n'
          'SHADER_SET_MATRIX(sh, "probe", m)\nPOSTFX(sh)\n' + _LOOP)
    r = _run({"t.fs": fs, "t.gb": gb}, "t.gb", tmp_path, shot="t.png")
    assert r.returncode == 0, r.stderr
    assert 120 <= _pixel(tmp_path, "t.png", (60, 45))[0] <= 135


def test_shader_setters_reject_bad_handles(tmp_path):
    for call in ('SHADER_SET_TEXTURE(99, "x", 0)',
                 'SHADER_SET_ARRAY(99, "x", st)'):
        gb = ('SCREEN(64, 64, "B", 1)\nDIM st[2] AS FLOAT\n' + call + "\n")
        r = _run({"b.gb": gb}, "b.gb", tmp_path, frames=1)
        assert r.returncode != 0, call
        assert "SHADER" in r.stderr


def test_unknown_uniform_name_is_not_an_error(tmp_path):
    # Ein vom Compiler wegoptimiertes Uniform ist haeufig und harmlos --
    # es darf das Programm nicht abbrechen (wie bei SHADER_SET).
    fs = ("#version 330\nin vec2 fragTexCoord;\nuniform sampler2D texture0;\n"
          "out vec4 finalColor;\nvoid main() { finalColor = texture(texture0, fragTexCoord); }\n")
    gb = ('SCREEN(64, 64, "U", 1)\n'
          'DIM sh AS INTEGER\nsh = SHADER_LOAD("u.fs")\n'
          'DIM st[2] AS FLOAT\nSHADER_SET_ARRAY(sh, "gibtsnicht", st)\n'
          'DIM i AS IMAGE\ni = GENTEX_COLOR(4, 4, 255)\n'
          'SHADER_SET_TEXTURE(sh, "auchnicht", i)\nPRINT "ok"\n')
    r = _run({"u.fs": fs, "u.gb": gb}, "u.gb", tmp_path, frames=1)
    assert r.returncode == 0, r.stderr
    assert r.out == ["ok"]


def test_empty_array_is_rejected(tmp_path):
    gb = ('SCREEN(64, 64, "E", 1)\nDIM sh AS INTEGER\nsh = 0\n'
          'DIM st[0] AS FLOAT\nSHADER_SET_ARRAY(sh, "x", st)\n')
    r = _run({"e.gb": gb}, "e.gb", tmp_path, frames=1)
    assert r.returncode != 0 and "SHADER_SET_ARRAY" in r.stderr


# --------------------------------------------------------------- Geometrie
def _geo(expr_lines, tmp_path):
    gb = 'IMPORT "physics"\n' + "".join(f"PRINT {e}\n" for e in expr_lines)
    return _run({"g.gb": gb}, "g.gb", tmp_path, frames=1)


def test_line_intersection(tmp_path):
    r = _geo(["PHYSICS_LINES_HIT(0.0,0.0, 10.0,10.0, 0.0,10.0, 10.0,0.0)",
              "PHYSICS_LINES_X(0.0,0.0, 10.0,10.0, 0.0,10.0, 10.0,0.0)",
              "PHYSICS_LINES_Y(0.0,0.0, 10.0,10.0, 0.0,10.0, 10.0,0.0)"], tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.out == ["TRUE", "5.0", "5.0"]


def test_parallel_and_too_short_lines_do_not_intersect(tmp_path):
    r = _geo(["PHYSICS_LINES_HIT(0.0,0.0, 10.0,0.0, 0.0,5.0, 10.0,5.0)",
              "PHYSICS_LINES_HIT(0.0,0.0, 1.0,1.0, 9.0,10.0, 10.0,9.0)"], tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.out == ["FALSE", "FALSE"]


def test_intersection_point_is_nan_without_a_hit(tmp_path):
    # Bewusst NAN statt einer erfundenen Koordinate -- der Aufrufer soll erst
    # PHYSICS_LINES_HIT fragen.
    r = _geo(["ISNAN(PHYSICS_LINES_X(0.0,0.0, 1.0,1.0, 9.0,10.0, 10.0,9.0))"], tmp_path)
    if r.returncode != 0:            # ISNAN gibt es evtl. nicht -> Wert selbst pruefen
        r = _geo(["PHYSICS_LINES_X(0.0,0.0, 1.0,1.0, 9.0,10.0, 10.0,9.0)"], tmp_path)
        assert r.returncode == 0, r.stderr
        assert "nan" in " ".join(r.out).lower()
    else:
        assert r.out == ["TRUE"]


def test_point_on_line_respects_thickness(tmp_path):
    r = _geo(["PHYSICS_POINT_LINE(5.0,5.2, 0.0,0.0, 10.0,10.0, 1.0)",
              "PHYSICS_POINT_LINE(5.0,9.0, 0.0,0.0, 10.0,10.0, 1.0)"], tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.out == ["TRUE", "FALSE"]


def test_circle_against_line_segment(tmp_path):
    r = _geo(["PHYSICS_CIRCLE_LINE(5.0,7.0, 3.0, 0.0,5.0, 10.0,5.0)",
              "PHYSICS_CIRCLE_LINE(5.0,20.0, 3.0, 0.0,5.0, 10.0,5.0)",
              # jenseits des Streckenendes -- Gerade wuerde treffen, Strecke nicht
              "PHYSICS_CIRCLE_LINE(50.0,5.0, 3.0, 0.0,5.0, 10.0,5.0)"], tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.out == ["TRUE", "FALSE", "FALSE"]


_POLY = ('DIM xs[4] AS FLOAT\nDIM ys[4] AS FLOAT\n'
         'xs[0]=0.0 : ys[0]=0.0\nxs[1]=10.0 : ys[1]=0.0\n'
         'xs[2]=10.0 : ys[2]=10.0\nxs[3]=0.0 : ys[3]=10.0\n')


def test_point_in_polygon(tmp_path):
    gb = ('IMPORT "physics"\n' + _POLY +
          'PRINT PHYSICS_POINT_POLY(5.0, 5.0, xs, ys)\n'
          'PRINT PHYSICS_POINT_POLY(15.0, 5.0, xs, ys)\n'
          'PRINT PHYSICS_POINT_POLY(5.0, -1.0, xs, ys)\n')
    r = _run({"p.gb": gb}, "p.gb", tmp_path, frames=1)
    assert r.returncode == 0, r.stderr
    assert r.out == ["TRUE", "FALSE", "FALSE"]


def test_point_in_concave_polygon(tmp_path):
    # L-Form: der Punkt liegt in der Einbuchtung, also AUSSERHALB. Ein naiver
    # Bounding-Box-Test wuerde hier faelschlich TRUE liefern.
    gb = ('IMPORT "physics"\n'
          'DIM xs[6] AS FLOAT\nDIM ys[6] AS FLOAT\n'
          'xs[0]=0.0 : ys[0]=0.0\nxs[1]=10.0 : ys[1]=0.0\n'
          'xs[2]=10.0 : ys[2]=4.0\nxs[3]=4.0 : ys[3]=4.0\n'
          'xs[4]=4.0 : ys[4]=10.0\nxs[5]=0.0 : ys[5]=10.0\n'
          'PRINT PHYSICS_POINT_POLY(8.0, 8.0, xs, ys)\n'
          'PRINT PHYSICS_POINT_POLY(2.0, 8.0, xs, ys)\n')
    r = _run({"c.gb": gb}, "c.gb", tmp_path, frames=1)
    assert r.returncode == 0, r.stderr
    assert r.out == ["FALSE", "TRUE"]


def test_polygon_argument_errors(tmp_path):
    gb = ('IMPORT "physics"\n'
          'DIM xs[2] AS FLOAT\nDIM ys[2] AS FLOAT\n'
          'PRINT PHYSICS_POINT_POLY(1.0, 1.0, xs, ys)\n')
    r = _run({"q.gb": gb}, "q.gb", tmp_path, frames=1)
    assert r.returncode != 0 and "3 Punkte" in r.stderr
