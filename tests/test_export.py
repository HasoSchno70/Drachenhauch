"""Tests fuer den Standalone-Export (Schritt 7).

Geprueft wird die Python-Seite: Payload-Footer-Layout, Round-Trip-Extraktion
(Spiegel der Rust-`embedded_gbc`-Logik) und dass `export_standalone` Bytecode
anhaengt + den `assets/`-Ordner mitkopiert. Ein echtes `gbrt` ist nicht noetig --
als Runtime-Platzhalter dienen beliebige Bytes.
"""
import json
import struct
from pathlib import Path

import pytest

from gamebasic.export import append_payload, export_standalone, PAYLOAD_MAGIC


def _extract_payload(exe_bytes: bytes) -> bytes:
    """Spiegelt die Rust-Logik `embedded_gbc`: Footer pruefen, gbc-Bytes lesen."""
    assert len(exe_bytes) >= 16
    footer = len(exe_bytes) - 16
    assert exe_bytes[footer + 8:] == PAYLOAD_MAGIC
    length = struct.unpack("<Q", exe_bytes[footer:footer + 8])[0]
    assert 0 < length <= footer
    return exe_bytes[footer - length:footer]


def test_append_payload_footer_layout():
    exe = b"FAKE-RUNTIME-BINARY"
    gbc = b'{"hello":1}'
    out = append_payload(exe, gbc)
    # Reihenfolge: original | gbc | len(u64 LE) | magic
    assert out.startswith(exe)
    assert out[-8:] == PAYLOAD_MAGIC
    assert struct.unpack("<Q", out[-16:-8])[0] == len(gbc)
    assert _extract_payload(out) == gbc


def test_append_payload_roundtrip_unicode():
    gbc = json.dumps({"text": "Grüße äöü", "n": 42}).encode("utf-8")
    out = append_payload(b"RT", gbc)
    assert json.loads(_extract_payload(out).decode("utf-8"))["n"] == 42


@pytest.fixture
def fake_gbrt(tmp_path):
    p = tmp_path / "gbrt_fake.exe"
    p.write_bytes(b"\x00FAKE-GBRT-RUNTIME\x00" * 64)  # plausibler Binary-Blob
    return p


def test_export_standalone_embeds_runnable_bytecode(tmp_path, fake_gbrt):
    src = tmp_path / "game.gb"
    src.write_text('PRINT "hallo"\n', encoding="utf-8")
    out_dir = tmp_path / "dist"

    exe = export_standalone(src, fake_gbrt, out_dir=out_dir)

    assert exe.exists()
    assert exe.name.startswith("game")
    data = exe.read_bytes()
    # Beginnt mit der Runtime, endet mit dem Payload-Footer.
    assert data.startswith(fake_gbrt.read_bytes())
    gbc = _extract_payload(data)
    mod = json.loads(gbc.decode("utf-8"))
    # Der eingebettete Bytecode ist ein gueltiges serialisiertes Modul.
    assert "functions" in mod or "main" in mod or "code" in mod


def test_export_standalone_copies_assets(tmp_path, fake_gbrt):
    src = tmp_path / "game.gb"
    src.write_text('PRINT "x"\n', encoding="utf-8")
    assets = tmp_path / "assets"
    assets.mkdir()
    (assets / "hero.png").write_bytes(b"PNGDATA")
    (assets / "sfx").mkdir()
    (assets / "sfx" / "jump.wav").write_bytes(b"WAV")

    out_dir = tmp_path / "dist"
    export_standalone(src, fake_gbrt, out_dir=out_dir)

    assert (out_dir / "assets" / "hero.png").read_bytes() == b"PNGDATA"
    assert (out_dir / "assets" / "sfx" / "jump.wav").read_bytes() == b"WAV"


def test_export_missing_runtime_raises(tmp_path):
    src = tmp_path / "game.gb"
    src.write_text('PRINT "x"\n', encoding="utf-8")
    with pytest.raises(FileNotFoundError):
        export_standalone(src, tmp_path / "does_not_exist.exe", out_dir=tmp_path / "d")


def test_export_compile_error_propagates(tmp_path, fake_gbrt):
    from gamebasic.errors import GameBasicError
    src = tmp_path / "bad.gb"
    src.write_text('DIM x AS\n', encoding="utf-8")  # Syntaxfehler
    with pytest.raises(GameBasicError):
        export_standalone(src, fake_gbrt, out_dir=tmp_path / "d")
