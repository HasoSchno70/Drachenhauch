"""Golden-Tests fuer das m3d-Modul (VEC3/VEC4/QUAT/MAT4).

run_gb spawnt `gbrt run` -> skippt automatisch, wenn gbrt nicht gebaut ist.
f32-intern: bei nicht-exakten Werten via ROUND vergleichen.
"""
import pytest

from gamebasic.errors import GameBasicError


def _gb(body: str) -> str:
    return 'IMPORT "m3d"\n' + body


# --------------------------------------------------------------- VEC3
def test_vec3_basics(run_gb):
    out = run_gb(_gb(
        "DIM a AS VEC3\nDIM b AS VEC3\n"
        "a = VEC3_NEW(1, 2, 3)\nb = VEC3_NEW(4, 5, 6)\n"
        "PRINT a + b\n"
        "PRINT b - a\n"
        "PRINT a * 2.0\n"
        "PRINT 2.0 * a\n"
        "PRINT VEC3_DOT(a, b)\n"
        "PRINT VEC3_CROSS(VEC3_NEW(1,0,0), VEC3_NEW(0,1,0))\n"
        "PRINT VEC3_LENGTH(VEC3_NEW(3, 4, 0))\n"
        "PRINT VEC3_X(a)\nPRINT VEC3_Y(a)\nPRINT VEC3_Z(a)\n"
    ))
    assert out.splitlines() == [
        "Vec3(5.0, 7.0, 9.0)", "Vec3(3.0, 3.0, 3.0)",
        "Vec3(2.0, 4.0, 6.0)", "Vec3(2.0, 4.0, 6.0)",
        "32.0", "Vec3(0.0, 0.0, 1.0)", "5.0",
        "1.0", "2.0", "3.0",
    ]


def test_vec3_normalize_and_distance(run_gb):
    out = run_gb(_gb(
        "PRINT VEC3_NORMALIZE(VEC3_NEW(0, 5, 0))\n"
        "PRINT VEC3_DISTANCE(VEC3_NEW(0,0,0), VEC3_NEW(0,0,2))\n"
        "PRINT VEC3_NEG(VEC3_NEW(1, -2, 3))\n"
    ))
    assert out.splitlines() == [
        "Vec3(0.0, 1.0, 0.0)", "2.0", "Vec3(-1.0, 2.0, -3.0)",
    ]


# --------------------------------------------------------------- VEC4
def test_vec4_basics(run_gb):
    out = run_gb(_gb(
        "DIM v AS VEC4\nv = VEC4_NEW(1, 2, 3, 4)\n"
        "PRINT v\nPRINT VEC4_W(v)\n"
        "PRINT VEC4_FROM_VEC3(VEC3_NEW(7, 8, 9), 1.0)\n"
        "PRINT VEC4_DOT(VEC4_NEW(1,0,0,0), VEC4_NEW(1,9,9,9))\n"
    ))
    assert out.splitlines() == [
        "Vec4(1.0, 2.0, 3.0, 4.0)", "4.0",
        "Vec4(7.0, 8.0, 9.0, 1.0)", "1.0",
    ]


# --------------------------------------------------------------- QUAT
def test_quat_identity_rotation_is_noop(run_gb):
    out = run_gb(_gb(
        "PRINT QUAT_ROTATE_VEC3(QUAT_IDENTITY(), VEC3_NEW(1, 2, 3))\n"
    ))
    assert out.strip() == "Vec3(1.0, 2.0, 3.0)"


def test_quat_axis_angle_rotates_x_to_y(run_gb):
    # 90 Grad um Z dreht (1,0,0) -> (0,1,0).
    out = run_gb(_gb(
        "DIM q AS QUAT\nq = QUAT_FROM_AXIS_ANGLE(0, 0, 1, 1.5707963267948966)\n"
        "DIM r AS VEC3\nr = QUAT_ROTATE_VEC3(q, VEC3_NEW(1, 0, 0))\n"
        "PRINT f\"{ROUND(VEC3_X(r),4)} {ROUND(VEC3_Y(r),4)} {ROUND(VEC3_Z(r),4)}\"\n"
    ))
    assert out.strip() == "0.0 1.0 0.0"


def test_quat_mul_operator_matches_builtin(run_gb):
    out = run_gb(_gb(
        "DIM a AS QUAT\nDIM b AS QUAT\n"
        "a = QUAT_FROM_AXIS_ANGLE(0,0,1, 0.5)\n"
        "b = QUAT_FROM_AXIS_ANGLE(0,0,1, 0.7)\n"
        "DIM viaop AS QUAT\nDIM viafn AS QUAT\n"
        "viaop = a * b\nviafn = QUAT_MUL(a, b)\n"
        "PRINT viaop = viafn\n"
        # Komposition zweier Z-Drehungen = Summe der Winkel (1.2 rad) auf (1,0,0).
        "DIM r AS VEC3\nr = QUAT_ROTATE_VEC3(viaop, VEC3_NEW(1, 0, 0))\n"
        "PRINT f\"{ROUND(VEC3_X(r),4)} {ROUND(VEC3_Y(r),4)}\"\n"
    ))
    lines = out.splitlines()
    assert lines[0] == "TRUE"
    # cos(1.2)=0.3624, sin(1.2)=0.9320
    assert lines[1] == "0.3624 0.932"


def test_quat_slerp_endpoints(run_gb):
    out = run_gb(_gb(
        "DIM a AS QUAT\nDIM b AS QUAT\n"
        "a = QUAT_IDENTITY()\nb = QUAT_FROM_AXIS_ANGLE(0,1,0, 1.0)\n"
        "PRINT QUAT_SLERP(a, b, 0.0) = a\n"
    ))
    assert out.strip() == "TRUE"


# --------------------------------------------------------------- MAT4
def test_mat4_identity_and_translate(run_gb):
    out = run_gb(_gb(
        "PRINT VEC3_TRANSFORM(VEC3_NEW(3, 4, 5), MAT4_IDENTITY())\n"
        "PRINT VEC3_TRANSFORM(VEC3_NEW(0, 0, 0), MAT4_TRANSLATE(10, 20, 30))\n"
        "PRINT VEC3_TRANSFORM_DIR(VEC3_NEW(1, 0, 0), MAT4_TRANSLATE(10, 20, 30))\n"
    ))
    assert out.splitlines() == [
        "Vec3(3.0, 4.0, 5.0)",
        "Vec3(10.0, 20.0, 30.0)",
        "Vec3(1.0, 0.0, 0.0)",   # Richtung ignoriert Translation
    ]


def test_mat4_trs_and_operator_transform(run_gb):
    out = run_gb(_gb(
        "DIM m AS MAT4\n"
        "m = MAT4_TRS(VEC3_NEW(10, 20, 30), QUAT_IDENTITY(), VEC3_NEW(2, 2, 2))\n"
        "PRINT m * VEC3_NEW(1, 1, 1)\n"          # 1*2 + 10 = 12, ...
        "PRINT m * VEC4_NEW(0, 0, 0, 1)\n"
    ))
    assert out.splitlines() == [
        "Vec3(12.0, 22.0, 32.0)",
        "Vec4(10.0, 20.0, 30.0, 1.0)",
    ]


def test_mat4_mul_operator_matches_builtin(run_gb):
    out = run_gb(_gb(
        "DIM x AS MAT4\nDIM y AS MAT4\n"
        "x = MAT4_TRANSLATE(5, 0, 0)\ny = MAT4_SCALE(2, 2, 2)\n"
        "PRINT (x * y) = MAT4_MUL(x, y)\n"
    ))
    assert out.strip() == "TRUE"


def test_mat4_invert_roundtrip(run_gb):
    out = run_gb(_gb(
        "DIM m AS MAT4\nDIM inv AS MAT4\n"
        "m = MAT4_MUL(MAT4_TRANSLATE(5, -3, 2), MAT4_SCALE(2, 4, 8))\n"
        "inv = MAT4_INVERT(m)\n"
        "PRINT VEC3_TRANSFORM(VEC3_TRANSFORM(VEC3_NEW(3, 4, 5), m), inv)\n"
    ))
    assert out.strip() == "Vec3(3.0, 4.0, 5.0)"


def test_mat4_get(run_gb):
    out = run_gb(_gb(
        "DIM m AS MAT4\nm = MAT4_TRANSLATE(7, 8, 9)\n"
        "PRINT f\"{MAT4_GET(m,0,0)} {MAT4_GET(m,0,3)} {MAT4_GET(m,1,3)} {MAT4_GET(m,2,3)}\"\n"
    ))
    # column-major: row0/col0 = 1 (Diagonale), translation in col3
    assert out.strip() == "1.0 7.0 8.0 9.0"


def test_mat4_invert_singular_raises(run_gb):
    with pytest.raises(GameBasicError, match="nicht invertierbar"):
        run_gb(_gb("PRINT MAT4_INVERT(MAT4_SCALE(0, 0, 0))\n"))


# --------------------------------------------------------------- Typ-Fehler
def test_type_mismatch_raises(run_gb):
    with pytest.raises(GameBasicError, match="Erwartet VEC3"):
        run_gb(_gb("PRINT VEC3_X(VEC3_NEW(1,2,3) + VEC3_NEW(1,1,1))\n"
                   "PRINT VEC3_LENGTH(MAT4_IDENTITY())\n"))


def test_mat4_dim_type_accepted(run_gb):
    # DIM x AS MAT4 muss der Compiler akzeptieren (MODULE_TYPES).
    out = run_gb(_gb("DIM m AS MAT4\nm = MAT4_IDENTITY()\nPRINT MAT4_GET(m, 3, 3)\n"))
    assert out.strip() == "1.0"
