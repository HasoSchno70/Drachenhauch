"""LOAD_ASSETS + Asset-Cache.

Decken ab:
- Manifest mit Liste (Pfad-Cache)
- Manifest mit Object (Alias-Cache + Pfad-Cache)
- Cache-Hit: zweiter LOADIMAGE liefert dieselbe Surface
- Aliase: LOADIMAGE("player") trifft den Cache
- Pfad-Normalisierung: relative und absolute Pfade kollidieren im Cache
- Fehler-Cases: Manifest fehlt, Manifest-Pfad falsch
"""
import json
from pathlib import Path

import pytest


pygame = pytest.importorskip("pygame")


@pytest.fixture
def graphics():
    """Ein frisches Graphics-Objekt, mit Cleanup nach jedem Test."""
    from gamebasic.graphics import Graphics
    g = Graphics()
    yield g
    g.shutdown()


@pytest.fixture
def asset_dir(tmp_path):
    """Legt assets/coin.png + assets/hero.png + assets/manifest.json an
    (kopiert echte PNGs aus examples/assets/)."""
    src = Path(__file__).resolve().parent.parent / "examples" / "assets"
    assets = tmp_path / "assets"
    assets.mkdir()
    for name in ("coin.png", "hero.png"):
        (assets / name).write_bytes((src / name).read_bytes())
    return assets


def test_load_image_caches_repeated_calls(graphics, asset_dir):
    """Zweimal LOADIMAGE auf denselben Pfad liefert dieselbe Surface
    (identische Object-Identitaet)."""
    a = graphics.load_image(str(asset_dir / "coin.png"))
    b = graphics.load_image(str(asset_dir / "coin.png"))
    assert a is b


def test_load_assets_list_form(graphics, asset_dir, call_builtin):
    """Manifest mit list-Form pre-loadet die Bilder."""
    manifest = asset_dir / "m.json"
    manifest.write_text(json.dumps({
        "images": ["coin.png", "hero.png"]
    }))

    count = call_builtin_with_graphics(
        graphics, "LOAD_ASSETS", [str(manifest)]
    )
    assert count == 2
    # Cache-Treffer pruefen: nochmal LOADIMAGE auf identischen Pfad
    # liefert dieselbe Surface.
    rel_path = str(asset_dir / "coin.png")
    surf = graphics.load_image(rel_path)
    # Surface wurde beim Pre-Load gecacht -- jetzt referenz-gleich.
    assert graphics._image_cache[rel_path] is surf


def test_load_assets_dict_form_alias_hits(graphics, asset_dir):
    """Manifest mit Object-Form: LOADIMAGE("alias") trifft den Cache."""
    manifest = asset_dir / "m.json"
    manifest.write_text(json.dumps({
        "images": {"player": "hero.png", "money": "coin.png"}
    }))

    count = call_builtin_with_graphics(
        graphics, "LOAD_ASSETS", [str(manifest)]
    )
    assert count == 2
    # Alias-Hit
    player = graphics.load_image("player")
    money = graphics.load_image("money")
    assert player is not None
    assert money is not None
    # Doppelter Alias-Lookup -> selbe Surface
    assert graphics.load_image("player") is player


def test_load_assets_dict_form_path_also_hits(graphics, asset_dir):
    """Bei Alias-Manifest treffen sowohl Alias ALS AUCH der Pfad."""
    manifest = asset_dir / "m.json"
    manifest.write_text(json.dumps({
        "images": {"player": "hero.png"}
    }))
    call_builtin_with_graphics(graphics, "LOAD_ASSETS", [str(manifest)])

    via_alias = graphics.load_image("player")
    via_path = graphics.load_image(str(asset_dir / "hero.png"))
    assert via_alias is via_path


def test_load_assets_path_normalization(graphics, asset_dir):
    """Manifest-Pfad und User-Pfad muessen nicht identisch geschrieben
    sein -- Cache normalisiert via absolutize+normpath."""
    manifest = asset_dir / "m.json"
    manifest.write_text(json.dumps({
        "images": ["hero.png"]
    }))
    call_builtin_with_graphics(graphics, "LOAD_ASSETS", [str(manifest)])

    # Verschiedene Wege zum gleichen File
    surf1 = graphics.load_image(str(asset_dir / "hero.png"))
    surf2 = graphics.load_image(str(asset_dir.parent / "assets" / "hero.png"))
    assert surf1 is surf2


def test_load_assets_missing_manifest(graphics, tmp_path):
    """Manifest-Datei fehlt -> klare Fehlermeldung."""
    from gamebasic.errors import GBRuntimeError
    with pytest.raises(GBRuntimeError, match="nicht gefunden"):
        call_builtin_with_graphics(
            graphics, "LOAD_ASSETS", [str(tmp_path / "missing.json")]
        )


def test_load_assets_invalid_json(graphics, tmp_path):
    """Manifest mit kaputtem JSON -> klare Fehlermeldung."""
    from gamebasic.errors import GBRuntimeError
    bad = tmp_path / "bad.json"
    bad.write_text("{not valid json}")
    with pytest.raises(GBRuntimeError, match="Lesefehler"):
        call_builtin_with_graphics(graphics, "LOAD_ASSETS", [str(bad)])


def test_load_assets_wrong_top_type(graphics, tmp_path):
    """Manifest ist kein Object (z.B. eine Liste auf Top-Level) -> Error."""
    from gamebasic.errors import GBRuntimeError
    bad = tmp_path / "list.json"
    bad.write_text('["just", "a", "list"]')
    with pytest.raises(GBRuntimeError, match="muss ein JSON-Object"):
        call_builtin_with_graphics(graphics, "LOAD_ASSETS", [str(bad)])


def test_load_assets_wrong_images_type(graphics, tmp_path):
    """'images' ist keine Liste oder Object -> Error."""
    from gamebasic.errors import GBRuntimeError
    bad = tmp_path / "x.json"
    bad.write_text('{"images": "string-instead-of-list"}')
    with pytest.raises(GBRuntimeError, match="'images' muss"):
        call_builtin_with_graphics(graphics, "LOAD_ASSETS", [str(bad)])


def test_load_assets_missing_file_in_manifest(graphics, asset_dir):
    """Manifest referenziert ein nicht-existentes Bild -> Error vom
    load_image-Pfad (durchgereicht)."""
    from gamebasic.errors import GBRuntimeError
    manifest = asset_dir / "m.json"
    manifest.write_text(json.dumps({"images": ["nonexistent.png"]}))
    with pytest.raises(GBRuntimeError, match="Konnte Bild nicht laden"):
        call_builtin_with_graphics(graphics, "LOAD_ASSETS", [str(manifest)])


def test_load_assets_returns_count(graphics, asset_dir):
    """Return-Wert ist die Anzahl geladener Assets (images + sounds)."""
    manifest = asset_dir / "m.json"
    manifest.write_text(json.dumps({"images": ["coin.png", "hero.png"]}))
    n = call_builtin_with_graphics(graphics, "LOAD_ASSETS", [str(manifest)])
    assert n == 2


def test_load_assets_empty_manifest(graphics, tmp_path):
    """Leeres Manifest (keine images/sounds) -> 0, kein Error."""
    manifest = tmp_path / "empty.json"
    manifest.write_text("{}")
    n = call_builtin_with_graphics(graphics, "LOAD_ASSETS", [str(manifest)])
    assert n == 0


# --- Helper -------------------------------------------------------------

def call_builtin_with_graphics(graphics, name, args):
    """Ruft einen graphics_builtin direkt mit injizierter Graphics-Instanz auf.
    Die @graphics_builtin-Decorator-Wrapper rufen normalerweise Interpreter.
    _get_graphics() -- in den Tests injizieren wir sie direkt.
    """
    from gamebasic.builtins_registry import GRAPHICS_BUILTINS
    fn = GRAPHICS_BUILTINS.get(name.lower())
    if fn is None:
        raise KeyError(f"Graphics-Builtin '{name}' nicht registriert")
    return fn(graphics, args)
