"""Die vier in Drachenhauch geschriebenen Werkzeuge in der IDE.

Sie lagen bisher nur als Beispieldateien unter `examples/` -- erreichbar
also nur, wenn man wusste, dass es sie gibt. Jetzt stehen sie im
Datei-Menue und in der Befehlspalette, mit zwei Wegen: **starten** (sie sind
Drachenhauch-Programme, laufen also ueber dieselbe Konsole wie alles andere)
und **Quelltext oeffnen** (sie zu lesen ist der halbe Zweck).
"""
import os
from pathlib import Path

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

import pytest

pytest.importorskip("PySide6")

from drachenhauch.editor_qt.piloten import (   # noqa: E402
    PILOTEN, beschreibung, pfad,
)

_WURZEL = Path(__file__).resolve().parent.parent


@pytest.fixture(scope="module", autouse=True)
def _qapp():
    from PySide6.QtWidgets import QApplication
    app = QApplication.instance() or QApplication([])
    yield app


@pytest.fixture
def win(tmp_path, monkeypatch):
    from drachenhauch.editor_qt import main_window as mw
    monkeypatch.setattr(mw, "save_settings", lambda *_a, **_kw: None)
    monkeypatch.setattr(mw, "clear_autosaves", lambda *_a, **_kw: None)
    return mw.DrachenhauchEditor(tmp_path)


# ----------------------------------------------------------------- Daten
def test_jeder_pilot_existiert_wirklich():
    """Die Liste zeigt auf echte Dateien -- ein Menue-Eintrag, der ins Leere
    fuehrt, ist schlimmer als keiner."""
    for e in PILOTEN:
        assert pfad(_WURZEL, e).exists(), e["datei"]


def test_die_zeilenzahlen_stimmen_mit_der_datei():
    """Die Zahlen im Tooltip sind gemessen, nicht geschaetzt. Waechst ein
    Pilot, faellt das hier auf, statt dass die IDE eine veraltete Zahl
    behauptet."""
    for e in PILOTEN:
        ist = len(pfad(_WURZEL, e).read_text(encoding="utf-8").splitlines())
        assert ist == e["dh"], f"{e['datei']}: {ist} Zeilen, eingetragen {e['dh']}"


def test_beschreibung_nennt_beide_zahlen():
    for e in PILOTEN:
        t = beschreibung(e)
        assert str(e["dh"]) in t and str(e["qt"]) in t


# ------------------------------------------------------------------- IDE
def test_menue_und_palette_kennen_alle_piloten(win):
    assert len(win.act_piloten) == len(PILOTEN)
    txt = [a.text() for a in win._all_palette_actions()]
    for e in PILOTEN:
        assert any(e["titel"] in t and "Drachenhauch" in t for t in txt), e["titel"]
    assert win.act_piloten_quelltext.text() in txt


def test_quelltexte_oeffnen_legt_je_einen_tab_an(win, monkeypatch):
    from drachenhauch.editor_qt import main_window as mw
    monkeypatch.setattr(mw, "piloten_pfad", lambda _root, e: pfad(_WURZEL, e))
    geoeffnet = []
    monkeypatch.setattr(win, "_open_file", lambda p: geoeffnet.append(Path(p)))
    win._open_pilot_sources()
    assert [p.name for p in geoeffnet] == [e["datei"] for e in PILOTEN]


def test_starten_reicht_den_pfad_an_die_konsole(win, monkeypatch):
    from drachenhauch.editor_qt import main_window as mw
    monkeypatch.setattr(mw, "piloten_pfad", lambda _root, e: pfad(_WURZEL, e))
    gestartet = []
    monkeypatch.setattr(win.console, "is_running", lambda: False)
    monkeypatch.setattr(win.console, "start_run_auto",
                        lambda p: gestartet.append(Path(p)) or "native")
    win._run_pilot(PILOTEN[0])
    assert gestartet == [pfad(_WURZEL, PILOTEN[0])]


def test_kein_zweiter_start_waehrend_etwas_laeuft(win, monkeypatch):
    """Sonst stuenden zwei Programme auf derselben Konsole und der
    Stopp-Knopf traefe nur eines."""
    from drachenhauch.editor_qt import main_window as mw
    monkeypatch.setattr(mw, "piloten_pfad", lambda _root, e: pfad(_WURZEL, e))
    monkeypatch.setattr(win.console, "is_running", lambda: True)
    monkeypatch.setattr(win.console, "start_run_auto",
                        lambda p: pytest.fail("darf nicht starten"))
    win._run_pilot(PILOTEN[0])


def test_fehlende_datei_nennt_den_pfad(win, monkeypatch):
    """Ein blosses 'nicht gefunden' laesst einen raten -- die Meldung muss
    sagen, WO gesucht wurde (im Installer liegt `examples/` woanders als im
    Repo)."""
    from PySide6.QtWidgets import QMessageBox
    from drachenhauch.editor_qt import main_window as mw
    monkeypatch.setattr(mw, "piloten_pfad",
                        lambda _root, e: Path("/gibt/es/nicht") / e["datei"])
    gesehen = []
    monkeypatch.setattr(QMessageBox, "warning", lambda *a: gesehen.append(a) or None)
    assert win._pilot_pfad(PILOTEN[0]) is None
    assert gesehen and "gibt" in gesehen[0][2] and PILOTEN[0]["datei"] in gesehen[0][2]
