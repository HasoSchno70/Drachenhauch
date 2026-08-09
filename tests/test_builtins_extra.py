"""Golden-Tests fuer die Befehlssatz-Ergaenzungen: Game-Math, Perlin-Noise,
Laufzeit-Typen, Encoding/Hash und Datei/OS-Helfer. run_gb -> dhrt run."""
import pytest

from drachenhauch.errors import DrachenhauchError


# ----------------------------------------------------------- Farben / Alpha
def test_rgba_packs_alpha_in_high_byte(run_gb):
    # (a<<24)|(r<<16)|(g<<8)|b
    expected = (200 << 24) | (255 << 16) | (128 << 8) | 64
    assert run_gb("PRINT RGBA(255, 128, 64, 200)\n").strip() == str(expected)


def test_alpha_extracts_and_defaults_opaque(run_gb):
    out = run_gb(
        "PRINT ALPHA(RGBA(10, 20, 30, 128))\n"
        "PRINT ALPHA(RGB(10, 20, 30))\n"          # 24-bit -> deckend (255)
    ).splitlines()
    assert out == ["128", "255"]


def test_rgb_channels_ignore_alpha_byte(run_gb):
    out = run_gb(
        "PRINT RED(RGBA(200, 100, 50, 128))\n"
        "PRINT GREEN(RGBA(200, 100, 50, 128))\n"
        "PRINT BLUE(RGBA(200, 100, 50, 128))\n"
    ).splitlines()
    assert out == ["200", "100", "50"]


def test_rgba_alpha_zero_clamped_to_one(run_gb):
    # Alpha 0 ist als Farb-Zahl nicht von "deckend" unterscheidbar -> auf 1.
    assert run_gb("PRINT ALPHA(RGBA(1, 2, 3, 0))\n").strip() == "1"


def test_rgba_out_of_range_errors(run_gb):
    with pytest.raises(DrachenhauchError, match="RGBA"):
        run_gb("PRINT RGBA(0, 0, 0, 300)\n")


def test_rgb_still_24bit(run_gb):
    assert run_gb("PRINT RGB(255, 128, 64)\n").strip() == str((255 << 16) | (128 << 8) | 64)


# ----------------------------------------------------------- Game-Math
def test_wrap(run_gb):
    assert run_gb("PRINT WRAP(370, 0, 360)\nPRINT WRAP(-10, 0, 360)\n").splitlines() == ["10.0", "350.0"]


def test_pingpong(run_gb):
    assert run_gb("PRINT PINGPONG(0.5, 2)\nPRINT PINGPONG(2.5, 2)\nPRINT PINGPONG(3.0, 2)\n").splitlines() \
        == ["0.5", "1.5", "1.0"]


def test_movetoward(run_gb):
    out = run_gb("PRINT MOVETOWARD(0, 10, 3)\nPRINT MOVETOWARD(0, 2, 5)\nPRINT MOVETOWARD(10, 0, 3)\n")
    assert out.splitlines() == ["3.0", "2.0", "7.0"]


def test_smoothstep_clamp01_approx_log10(run_gb):
    out = run_gb(
        "PRINT SMOOTHSTEP(0, 1, 0.5)\nPRINT SMOOTHSTEP(0, 1, -1)\n"
        "PRINT CLAMP01(1.7)\nPRINT CLAMP01(-0.3)\n"
        "PRINT APPROX(0.1 + 0.2, 0.3)\nPRINT APPROX(1, 2)\n"
        "PRINT LOG10(1000)\n")
    assert out.splitlines() == ["0.5", "0.0", "1.0", "0.0", "TRUE", "FALSE", "3.0"]


# ----------------------------------------------------------- Perlin-Noise
def test_noise_deterministic_and_zero_at_lattice(run_gb):
    # Perlin ist 0 an ganzzahligen Gitterpunkten und reproduzierbar.
    out = run_gb(
        "PRINT NOISE(0)\nPRINT NOISE2(3, 5)\n"
        "PRINT NOISE2(1.5, 2.5) = NOISE2(1.5, 2.5)\n")
    assert out.splitlines() == ["0.0", "0.0", "TRUE"]


def test_noise_in_range(run_gb):
    # Werte muessen in ~[-1, 1] liegen (hier grob geprueft).
    out = run_gb(
        "DIM i AS INTEGER\nDIM ok AS INTEGER\nok = 1\n"
        "FOR i = 0 TO 200\n"
        "    DIM n AS FLOAT\n"
        "    n = NOISE2(i * 0.137, i * 0.071)\n"
        "    IF n < -1.001 OR n > 1.001 THEN ok = 0\n"
        "NEXT\n"
        "PRINT ok\n")
    assert out.strip() == "1"


def test_fbm_deterministic(run_gb):
    assert run_gb("PRINT FBM(1.5, 2.5, 4) = FBM(1.5, 2.5, 4)\n").strip() == "TRUE"


# ----------------------------------------------------------- Laufzeit-Typen
def test_typeof(run_gb):
    out = run_gb(
        'DIM s AS STRING\ns = "x"\n'
        "PRINT TYPEOF(5)\nPRINT TYPEOF(3.0)\nPRINT TYPEOF(s)\nPRINT TYPEOF(TRUE)\n")
    assert out.splitlines() == ["INTEGER", "FLOAT", "STRING", "BOOLEAN"]


def test_type_predicates(run_gb):
    out = run_gb(
        "PRINT ISNUM(5)\nPRINT ISNUM(3.0)\nPRINT ISINT(5)\nPRINT ISINT(3.0)\n"
        'PRINT ISSTR("x")\nPRINT ISBOOL(TRUE)\nPRINT ISINT(TRUE)\n')
    assert out.splitlines() == ["TRUE", "TRUE", "TRUE", "FALSE", "TRUE", "TRUE", "FALSE"]


# ----------------------------------------------------------- Encoding / Hash
def test_base64_roundtrip(run_gb):
    out = run_gb(
        'PRINT BASE64_ENCODE("Hello, World!")\n'
        'PRINT BASE64_DECODE("SGVsbG8sIFdvcmxkIQ==")\n'
        'PRINT BASE64_DECODE(BASE64_ENCODE("Umlaute: äöü"))\n')
    assert out.splitlines() == ["SGVsbG8sIFdvcmxkIQ==", "Hello, World!", "Umlaute: äöü"]


def test_crc32_known_value(run_gb):
    # CRC32("hello") = 0x3610A686 = 907060870 (Referenzwert).
    assert run_gb('PRINT CRC32("hello")\n').strip() == "907060870"


def test_hash_deterministic(run_gb):
    out = run_gb('PRINT HASH("hello") = HASH("hello")\nPRINT HASH("a") = HASH("b")\n')
    assert out.splitlines() == ["TRUE", "FALSE"]


def test_base64_decode_invalid_raises(run_gb):
    with pytest.raises(DrachenhauchError, match="BASE64_DECODE"):
        run_gb('PRINT BASE64_DECODE("@@@@")\n')


# ----------------------------------------------------------- Datei / OS
def test_dirlist(run_gb, tmp_path):
    (tmp_path / "b.txt").write_text("x", encoding="utf-8")
    (tmp_path / "a.txt").write_text("y", encoding="utf-8")
    (tmp_path / "sub").mkdir()
    # listet sortiert; der run_gb-Tempfile (_gbtest_*.dh) liegt auch in base.
    out = run_gb(
        'DIM names AS ARRAY OF STRING\nnames = DIRLIST(".")\n'
        "DIM i AS INTEGER\n"
        "FOR i = 0 TO LEN(names) - 1\n"
        '    IF names[i] = "a.txt" OR names[i] = "b.txt" OR names[i] = "sub" THEN PRINT names[i]\n'
        "NEXT\n", base=tmp_path)
    assert out.splitlines() == ["a.txt", "b.txt", "sub"]


def test_path_helpers(run_gb):
    out = run_gb(
        'PRINT BASENAME("a/b/c.txt")\nPRINT DIRNAME("a/b/c.txt")\n')
    assert out.splitlines()[0] == "c.txt"
    # DIRNAME normalisiert Separatoren je nach OS -> nur den letzten Teil prüfen.
    assert out.splitlines()[1].replace("\\", "/") == "a/b"


def test_copy_rename_append(run_gb, tmp_path):
    (tmp_path / "src.txt").write_text("hallo", encoding="utf-8")
    out = run_gb(
        'COPYFILE("src.txt", "dst.txt")\n'
        'APPENDFILE("dst.txt", " welt")\n'
        'RENAME("dst.txt", "final.txt")\n'
        'DIM f AS FILE\nf = OpenFile("final.txt", "r")\nPRINT ReadLine(f)\nCloseFile(f)\n'
        'PRINT FILEEXISTS("dst.txt")\nPRINT FILEEXISTS("final.txt")\n',
        base=tmp_path)
    assert out.splitlines() == ["hallo welt", "FALSE", "TRUE"]


def test_dirlist_missing_dir_raises(run_gb):
    with pytest.raises(DrachenhauchError, match="DIRLIST"):
        run_gb('PRINT LEN(DIRLIST("does_not_exist_xyz_123"))\n')
