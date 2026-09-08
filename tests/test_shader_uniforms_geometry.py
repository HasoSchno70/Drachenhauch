"""Shader-Uniforms (Array / zweiter Sampler / Matrix) und Linien-Geometrie
(Etappe 4 des Ausbaus).

Die Shader-Tests rendern wirklich und messen einzelne Pixel -- nur so ist
belegt, dass das Uniform in der GPU ankommt. Sie sind bewusst so gebaut, dass
der erwartete Wert NICHT dem Ergebnis "Uniform gar nicht gesetzt" entspricht
(sonst wuerde der Test auch ohne die Funktion bestehen).

Die uebertragbaren Tests liegen seit 2026-09-08 als Pruefsammlung in
`tests/pruef/shader_uniforms_geometry.dhtest` (dhrt test); hier bleiben nur die, die ein Bild,
eine geschriebene Datei oder den Quelltext mit einem fremden Leser pruefen.
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
Image = pytest.importorskip("PIL.Image", reason="Pillow fuer die Pixel-Pruefung noetig")


def _run(files: dict, main: str, tmp_path, frames=3, shot=None):
    for name, body in files.items():
        (tmp_path / name).write_text(body, encoding="utf-8")
    env = dict(os.environ, DHRT_FRAMES=str(frames))
    if shot:
        env["DHRT_SCREENSHOT"] = str(tmp_path / shot)
    r = subprocess.run([str(_DHRT), "run", str(tmp_path / main)], capture_output=True,
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
    r = _run({"a.fs": fs, "a.dh": gb}, "a.dh", tmp_path, shot="a.png")
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
    r = _run({"m.fs": fs, "m.dh": gb}, "m.dh", tmp_path, shot="m.png")
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
    r = _run({"t.fs": fs, "t.dh": gb}, "t.dh", tmp_path, shot="t.png")
    assert r.returncode == 0, r.stderr
    assert 120 <= _pixel(tmp_path, "t.png", (60, 45))[0] <= 135


# --------------------------------------------------------------- Geometrie
def _geo(expr_lines, tmp_path):
    gb = 'IMPORT "physics"\n' + "".join(f"PRINT {e}\n" for e in expr_lines)
    return _run({"g.dh": gb}, "g.dh", tmp_path, frames=1)


_POLY = ('DIM xs[4] AS FLOAT\nDIM ys[4] AS FLOAT\n'
         'xs[0]=0.0 : ys[0]=0.0\nxs[1]=10.0 : ys[1]=0.0\n'
         'xs[2]=10.0 : ys[2]=10.0\nxs[3]=0.0 : ys[3]=10.0\n')
