"""Tests fuer das Branding-Modul (Logo-Laden + Skalierung)."""
import pytest


def test_smart_square_crop_already_square_returns_original():
    pytest.importorskip("PIL")
    from PIL import Image
    from gamebasic.branding import _smart_square_crop
    img = Image.new("RGBA", (100, 100), (0, 0, 0, 255))
    out = _smart_square_crop(img)
    assert out.size == (100, 100)


def test_smart_square_crop_wide_returns_square():
    pytest.importorskip("PIL")
    from PIL import Image
    from gamebasic.branding import _smart_square_crop
    img = Image.new("RGBA", (1408, 768), (0, 0, 0, 255))
    out = _smart_square_crop(img)
    w, h = out.size
    # Quadratisch (h auf beiden Seiten = 768)
    assert w == h == 768


def test_smart_square_crop_tall_returns_square():
    pytest.importorskip("PIL")
    from PIL import Image
    from gamebasic.branding import _smart_square_crop
    img = Image.new("RGBA", (200, 800), (0, 0, 0, 255))
    out = _smart_square_crop(img)
    w, h = out.size
    assert w == h == 200


def test_is_available_with_logo_present():
    """Wenn Logo im Asset-Ordner liegt UND Pillow da ist -> True."""
    pytest.importorskip("PIL")
    from gamebasic.branding import is_available, _logo_path
    if not _logo_path().exists():
        pytest.skip("Logo-Datei nicht im Repo")
    assert is_available() is True


def test_scaled_logo_image_scales_proportionally():
    """Gemeinsamer Helfer fuer AboutDialog/WelcomePanel (Review-Fund: beide
    duplizierten frueher identisches Laden+Skalieren mit nur abweichendem
    target_w)."""
    pytest.importorskip("PIL")
    from gamebasic.branding import scaled_logo_image, is_available, _logo_path
    if not _logo_path().exists():
        pytest.skip("Logo-Datei nicht im Repo")
    assert is_available()
    img = scaled_logo_image(320)
    assert img is not None
    w, h = img.size
    assert w == 320
    assert h > 0


def test_scaled_logo_image_none_when_unavailable(monkeypatch):
    import gamebasic.branding as branding
    monkeypatch.setattr(branding, "is_available", lambda: False)
    assert branding.scaled_logo_image(480) is None
