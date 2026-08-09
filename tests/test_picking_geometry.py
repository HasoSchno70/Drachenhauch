"""Picking auf echter Geometrie (Dreieck/Viereck) + 3D-Kollisionsmathematik.

Bis hierher konnte GB-Code einen Strahl nur gegen Huellkoerper testen
(RAY_HIT_BOX/SPHERE) oder gegen ein ganzes Modell (RAY_HIT_MODEL). Eine
einzelne Flaeche -- Boden-Kachel, Wandstueck, In-Welt-Panel -- war nicht
adressierbar.

Die Tests pruefen bewusst die Faelle, in denen eine naive Weiterreichung an
raylib FALSCH waere: unnormalisierte Richtung (Distanz waere skaliert),
Treffer hinter dem Ursprung, Ecke des Vierecks (nur das zweite Dreieck
trifft), Rueckseite (raylib cullt absichtlich nicht).
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


def _run(src, tmp_path, frames=1):
    (tmp_path / "p.gb").write_text(src, encoding="utf-8")
    env = dict(os.environ, GBRT_FRAMES=str(frames))
    r = subprocess.run([str(_GBRT), "run", str(tmp_path / "p.gb")], capture_output=True,
                       text=True, encoding="utf-8", env=env, timeout=90, cwd=str(tmp_path))
    r.out = [w for ln in (r.stdout or "").splitlines()
             if not ln.startswith(("WARNING:", "INFO:", "TRACE:")) for w in ln.split()]
    return r


# Dreieck in der Ebene z = 0, Schwerpunkt bei (0, 0).
_TRI = "-1.0,-1.0,0.0, 1.0,-1.0,0.0, 0.0,1.0,0.0"
# Viereck REIHUM (nicht ueberkreuz) -- Einheitsquadrat in der Ebene z = 0.
_QUAD = "-1.0,-1.0,0.0, 1.0,-1.0,0.0, 1.0,1.0,0.0, -1.0,1.0,0.0"
_HEAD = 'SCREEN(160, 120, "Pick", 1)\n'


def _floats(r):
    return [float(w) for w in r.out]


def test_ray_hits_triangle_in_front(tmp_path):
    r = _run(_HEAD + f"PRINT RAY_HIT_TRI(0.0,0.0,5.0, 0.0,0.0,-1.0, {_TRI})\n", tmp_path)
    assert r.returncode == 0, r.stderr
    assert _floats(r)[0] == pytest.approx(5.0, abs=1e-4)


def test_unnormalized_direction_still_yields_world_distance(tmp_path):
    # Der eigentliche Knackpunkt: raylib liefert die Distanz in Vielfachen der
    # RICHTUNGSLAENGE. Ohne Normalisierung stuende hier 2.5 statt 5.0 -- der
    # Test ist damit trennscharf gegen "einfach durchgereicht".
    r = _run(_HEAD + f"PRINT RAY_HIT_TRI(0.0,0.0,5.0, 0.0,0.0,-2.0, {_TRI})\n", tmp_path)
    assert r.returncode == 0, r.stderr
    assert _floats(r)[0] == pytest.approx(5.0, abs=1e-4)


def test_triangle_miss_and_behind_the_origin(tmp_path):
    r = _run(_HEAD +
             # daneben (Strahl geht seitlich am Dreieck vorbei)
             f"PRINT RAY_HIT_TRI(5.0,0.0,5.0, 0.0,0.0,-1.0, {_TRI})\n"
             # Dreieck liegt HINTER dem Ursprung -> kein Treffer, keine
             # negative Distanz (der Fehler, der PICK_SPHERE einmal hatte)
             f"PRINT RAY_HIT_TRI(0.0,0.0,5.0, 0.0,0.0,1.0, {_TRI})\n"
             # Nullrichtung ist kein Strahl
             f"PRINT RAY_HIT_TRI(0.0,0.0,5.0, 0.0,0.0,0.0, {_TRI})\n", tmp_path)
    assert r.returncode == 0, r.stderr
    assert _floats(r) == [-1.0, -1.0, -1.0]


def test_triangle_is_hit_from_the_back_too(tmp_path):
    # raylib cullt bewusst nicht ("Avoid culling!") -- fuer Levelgeometrie ist
    # das richtig: eine Wand blockt auch von hinten.
    r = _run(_HEAD + f"PRINT RAY_HIT_TRI(0.0,0.0,-5.0, 0.0,0.0,1.0, {_TRI})\n", tmp_path)
    assert r.returncode == 0, r.stderr
    assert _floats(r)[0] == pytest.approx(5.0, abs=1e-4)


def test_quad_covers_both_of_its_triangles(tmp_path):
    # (0.9, 0.9) liegt in der Ecke, die NUR das zweite Teildreieck abdeckt --
    # wer nur p1/p2/p3 testet, verfehlt hier.
    r = _run(_HEAD +
             f"PRINT RAY_HIT_QUAD(-0.9,-0.9,5.0, 0.0,0.0,-1.0, {_QUAD})\n"
             f"PRINT RAY_HIT_QUAD(0.9,0.9,5.0, 0.0,0.0,-1.0, {_QUAD})\n"
             f"PRINT RAY_HIT_QUAD(1.5,0.0,5.0, 0.0,0.0,-1.0, {_QUAD})\n", tmp_path)
    assert r.returncode == 0, r.stderr
    d = _floats(r)
    assert d[0] == pytest.approx(5.0, abs=1e-4)
    assert d[1] == pytest.approx(5.0, abs=1e-4)
    assert d[2] == -1.0


def test_pick_matches_the_ray_through_the_mouse_position(tmp_path):
    # PICK_* ist definitionsgemaess RAY_HIT_* mit dem Mausstrahl. Genau das
    # wird hier nachgerechnet -- ein grosses Viereck vor der Kamera, damit der
    # Strahl (Maus bei 0,0 im Testlauf) sicher trifft.
    big = "-100.0,-100.0,0.0, 100.0,-100.0,0.0, 100.0,100.0,0.0, -100.0,100.0,0.0"
    r = _run(_HEAD +
             "CAMERA3D(0.0, 0.0, 10.0, 0.0, 0.0, 0.0, 45.0)\n"
             f"PRINT PICK_QUAD({big})\n"
             "PRINT RAY_HIT_QUAD(CAMERA3D_X(), CAMERA3D_Y(), CAMERA3D_Z(), "
             "SCREEN_TO_WORLD_DIR_X(MOUSEX(), MOUSEY()), "
             "SCREEN_TO_WORLD_DIR_Y(MOUSEX(), MOUSEY()), "
             f"SCREEN_TO_WORLD_DIR_Z(MOUSEX(), MOUSEY()), {big})\n", tmp_path)
    assert r.returncode == 0, r.stderr
    d = _floats(r)
    assert d[0] > 0.0, "Mausstrahl trifft das Viereck nicht"
    assert d[0] == pytest.approx(d[1], abs=1e-3)


def test_pick_tri_reports_a_hit_on_a_face_in_view(tmp_path):
    big = "-100.0,-100.0,0.0, 100.0,-100.0,0.0, 0.0,100.0,0.0"
    r = _run(_HEAD +
             "CAMERA3D(0.0, 0.0, 10.0, 0.0, 0.0, 0.0, 45.0)\n"
             f"PRINT PICK_TRI({big})\n", tmp_path)
    assert r.returncode == 0, r.stderr
    assert _floats(r)[0] > 0.0


def test_wrong_argument_count_is_reported(tmp_path):
    r = _run(_HEAD + "PRINT RAY_HIT_TRI(0.0, 0.0, 5.0)\n", tmp_path)
    assert r.returncode != 0
    assert "RAY_HIT_TRI" in r.stderr and "15" in r.stderr


# --------------------------------------------------- pure 3D-Mathematik
def _pure(lines, tmp_path):
    return _run('IMPORT "physics"\n' + "".join(f"PRINT {e}\n" for e in lines), tmp_path)


def test_spheres_overlap(tmp_path):
    r = _pure(["PHYSICS_SPHERE_SPHERE(0.0,0.0,0.0, 1.0, 1.5,0.0,0.0, 1.0)",
               "PHYSICS_SPHERE_SPHERE(0.0,0.0,0.0, 1.0, 3.0,0.0,0.0, 1.0)",
               # Beruehrung genau auf Abstand r1+r2 zaehlt NICHT als Treffer
               # (gleiche Konvention wie PHYSICS_CIRCLE_CIRCLE)
               "PHYSICS_SPHERE_SPHERE(0.0,0.0,0.0, 1.0, 2.0,0.0,0.0, 1.0)",
               # nur in Z versetzt -- ein 2D-Test wuerde hier faelschlich treffen
               "PHYSICS_SPHERE_SPHERE(0.0,0.0,0.0, 1.0, 0.0,0.0,5.0, 1.0)"], tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.out == ["TRUE", "FALSE", "FALSE", "FALSE"]


def test_distance3(tmp_path):
    r = _pure(["PHYSICS_DISTANCE3(0.0,0.0,0.0, 1.0,2.0,2.0)"], tmp_path)
    assert r.returncode == 0, r.stderr
    assert _floats(r)[0] == pytest.approx(3.0, abs=1e-6)


def test_point_in_triangle(tmp_path):
    tri = "0.0,0.0, 10.0,0.0, 0.0,10.0"
    r = _pure([f"PHYSICS_POINT_TRI(1.0, 1.0, {tri})",
               f"PHYSICS_POINT_TRI(9.0, 9.0, {tri})",
               # gegen den Uhrzeigersinn aufgezaehlt -> gleiches Ergebnis
               "PHYSICS_POINT_TRI(1.0, 1.0, 0.0,10.0, 10.0,0.0, 0.0,0.0)"], tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.out == ["TRUE", "FALSE", "TRUE"]


def test_pure_3d_math_needs_no_window(tmp_path):
    # Konsolenprogramm ohne SCREEN: die reinen physics-Builtins muessen laufen
    # (das ist der Unterschied zu RAY_HIT_*, die an der 3D-Kamera haengen).
    r = _run('IMPORT "physics"\n'
             'PRINT PHYSICS_SPHERE_SPHERE(0.0,0.0,0.0, 2.0, 1.0,1.0,1.0, 0.5)\n', tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.out == ["TRUE"]
