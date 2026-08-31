"""Liegengebliebene Temp-`.dh`-Dateien beim IDE-Start entfernen.

Fehlerpruefung, Debugger und Profiler schreiben den Puffer **neben die
Quelle** -- anders loest `IMPORT "helfer.dh"` nicht auf. Wird der Lauf
abgebrochen, bleibt die Datei liegen; in `examples/` kippt so ein Streuner
jede Zaehlung und jeden `glob("*.dh")`-Test.

Beim Loeschen geht es weniger darum, das Richtige zu treffen, als darum,
**nichts Falsches** zu treffen: keine fremden Dateien und keine, die eine
zweite laufende IDE gerade braucht.
"""
import os
import time
from pathlib import Path

import pytest

from drachenhauch.editor_qt import tempdateien


def _alt_machen(p: Path, sekunden: float = 3600.0) -> None:
    """Die Datei um `sekunden` in die Vergangenheit datieren."""
    t = time.time() - sekunden
    os.utime(p, (t, t))


def test_neu_traegt_praefix_und_prozessnummer(tmp_path):
    fd, name = tempdateien.neu(str(tmp_path))
    os.close(fd)
    p = Path(name)
    assert p.parent == tmp_path              # neben der Quelle, nicht im Systemordner
    assert p.name.startswith(f"_dhtmp_{os.getpid()}_")
    assert p.suffix == ".dh"
    p.unlink()


def test_rest_eines_toten_prozesses_verschwindet(tmp_path):
    p = tmp_path / "_dhtmp_999999_abcd1234.dh"     # Prozess gibt es nicht
    p.write_text("PRINT 1\n", encoding="utf-8")
    _alt_machen(p)
    assert tempdateien.aufraeumen([tmp_path]) == [p]
    assert not p.exists()


def test_datei_eines_laufenden_prozesses_bleibt(tmp_path):
    """Eine zweite IDE mitten in einer Pruefung -- oder in einer stundenlangen
    Debug-Sitzung -- darf ihre Datei nicht verlieren. Ein Altersvergleich
    koennte das nicht leisten: eine Sitzung darf beliebig lange dauern."""
    p = tmp_path / f"_dhtmp_{os.getpid()}_abcd1234.dh"
    p.write_text("PRINT 1\n", encoding="utf-8")
    _alt_machen(p, 86400.0)                        # einen Tag alt und trotzdem lebendig
    assert tempdateien.aufraeumen([tmp_path]) == []
    assert p.exists()


def test_frische_datei_bleibt_trotz_toter_nummer(tmp_path):
    """Prozessnummern werden wiederverwendet. Das Mindestalter faengt den
    Fall ab, dass eine gerade angelegte Datei die Nummer eines laengst
    beendeten Prozesses traegt."""
    p = tmp_path / "_dhtmp_999999_abcd1234.dh"
    p.write_text("PRINT 1\n", encoding="utf-8")
    assert tempdateien.aufraeumen([tmp_path]) == []
    assert p.exists()


@pytest.mark.parametrize("name", [
    "tmpkele4kdt.dh",          # das alte Muster -- weist sich NICHT als unseres aus
    "_dhtmp_ohne_nummer.dh",   # Praefix, aber keine Prozessnummer
    "spiel.dh",                # ganz normale Datei
    "_dhtmp_999999_abcd.txt",  # keine .dh
])
def test_fremde_dateien_werden_nie_angefasst(tmp_path, name):
    """Was sich nicht eindeutig als unser Werk ausweist, koennte jemandem
    gehoeren -- auch die `tmpXXXXXXXX.dh` aus der Zeit vor dem Praefix."""
    p = tmp_path / name
    p.write_text("PRINT 1\n", encoding="utf-8")
    _alt_machen(p)
    assert tempdateien.aufraeumen([tmp_path]) == []
    assert p.exists()


def test_mehrere_verzeichnisse_und_doppelte(tmp_path):
    a, b = tmp_path / "a", tmp_path / "b"
    a.mkdir(); b.mkdir()
    for d in (a, b):
        p = d / "_dhtmp_999999_abcd1234.dh"
        p.write_text("x", encoding="utf-8")
        _alt_machen(p)
    # dasselbe Verzeichnis doppelt genannt -> jede Datei genau einmal
    weg = tempdateien.aufraeumen([a, b, a, tmp_path / "gibtsnicht"])
    assert len(weg) == 2


def test_aufraeumen_wirft_nie(tmp_path):
    """Ein Aufraeumen darf den Start der IDE nicht verhindern."""
    assert tempdateien.aufraeumen([None, "", 42, tmp_path / "weg"]) == []


def test_laeuft_ist_im_zweifel_wahr():
    """Lieber eine Datei zu viel liegen lassen als eine fremde loeschen."""
    assert tempdateien._laeuft(os.getpid()) is True
    assert tempdateien._laeuft(0) is True          # unlesbare Nummer -> nicht anfassen
    assert tempdateien._laeuft(-5) is True


# --------------------------------------------------------------- in der IDE
def test_ide_raeumt_beim_start_auf(tmp_path, monkeypatch):
    os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
    pytest.importorskip("PySide6")
    from PySide6.QtWidgets import QApplication
    from drachenhauch.editor_qt import main_window as mw
    QApplication.instance() or QApplication([])
    monkeypatch.setattr(mw, "save_settings", lambda *_a, **_kw: None)
    monkeypatch.setattr(mw, "clear_autosaves", lambda *_a, **_kw: None)

    (tmp_path / "examples").mkdir()
    rest = tmp_path / "examples" / "_dhtmp_999999_abcd1234.dh"
    rest.write_text("PRINT 1\n", encoding="utf-8")
    _alt_machen(rest)
    eigen = tmp_path / "examples" / "spiel.dh"
    eigen.write_text("PRINT 1\n", encoding="utf-8")
    _alt_machen(eigen)

    win = mw.DrachenhauchEditor(tmp_path)
    win._tempdateien_aufraeumen()               # sonst erst im Event-Loop
    assert not rest.exists()
    assert eigen.exists()
    win.close()
