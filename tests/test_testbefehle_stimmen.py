"""Die Testbefehle in der Doku muessen die der CI sein.

Der Anlass ist gemessen, nicht gedacht: in CLAUDE.md und README stand ein
Zwei-Durchgang-Befehl, der die Qt-Dateien im PARALLELEN Lauf mitnimmt. Die
CI tut das seit 2026-08-23 ausdruecklich nicht -- Qt-Fenster kollidieren
ueber Dateigrenzen hinweg, und der xdist-Arbeiter stirbt daran sporadisch.

Wer dem Doku-Befehl folgte, bekam Fehlschlaege in FREMDEN Dateien, die
einzeln gruen sind. Am 2026-09-01 zweimal hintereinander passiert, mit zwei
verschiedenen Dateien. So ein falscher Roter kostet mehr Zeit als der
langsamere Lauf, den er ersetzen sollte.

Zahlen in der Doku werden hier laengst gegen die Wirklichkeit geprueft
(Zeilenzahlen der Piloten, Signaturen im Buch, Widget-Arten des
Form-Designers). BEFEHLE waren es nicht -- das holt diese Datei nach.
"""
import re
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent
_CI = _ROOT / ".github" / "workflows" / "ci.yml"
# Die drei Durchgaenge des Windows-Jobs. Der POSIX-Job weiter unten faehrt
# andere (ohne Grafik, ohne Qt) und wird hier bewusst nicht abgeglichen --
# die Doku beschreibt den Weg auf der Entwicklermaschine.
_SCHRITTE = ("Run tests (parallel)",
             "Run tests (Qt-Editoren -- je Datei ein Prozess)",
             "Run tests (seriell -- exklusive Betriebsmittel)")
_DOKU = ("CLAUDE.md", "README.md", ".github/pull_request_template.md")


def _ci_befehle():
    """Die `run:`-Zeile je genanntem Schritt, in der Reihenfolge oben."""
    text = _CI.read_text(encoding="utf-8")
    aus = []
    for name in _SCHRITTE:
        m = re.search(r"- name: %s\s*\n\s*run: (.+)" % re.escape(name), text)
        assert m, "Schritt fehlt in ci.yml: " + name
        aus.append(m.group(1).strip())
    return aus


@pytest.fixture(scope="module")
def befehle():
    return _ci_befehle()


def test_jeder_ci_befehl_steht_in_der_doku(befehle):
    """Der Befehl selbst, nicht seine Beschreibung. Verglichen wird der Teil
    ab `pytest`/`tools` -- die Doku setzt `.venv\\Scripts\\python.exe` davor,
    die CI nicht."""
    kerne = []
    for b in befehle:
        i = b.find("pytest ")
        kerne.append(b[i:] if i >= 0 else b[b.find("tools/"):])
    for datei in _DOKU:
        text = (_ROOT / datei).read_text(encoding="utf-8")
        # Windows-Pfadtrenner in der Doku (tools\qt_tests_einzeln.py) zaehlt
        # genauso -- es ist derselbe Befehl.
        flach = text.replace("\\", "/")
        for kern in kerne:
            assert kern in flach, "%s nennt nicht: %s" % (datei, kern)


def test_keine_doku_nennt_den_alten_zwei_durchgang_befehl():
    """`-m "not seriell"` OHNE `and not qt` ist genau die Falle. Der Test
    sucht die geschlossene Zeichenkette, damit
    `-m "not seriell and not qt"` nicht mitgezaehlt wird."""
    # Erst pruefen, dann melden: `assert "..." not in text` wuerde die ganze
    # Datei in die Fehlermeldung schreiben (gemessen 1763 Zeilen), und die
    # eigentliche Aussage ginge darin unter.
    treffer = [d for d in _DOKU
               if '-m "not seriell"' in (_ROOT / d).read_text(encoding="utf-8")]
    assert not treffer, (
        "nennt den Befehl ohne `and not qt`: %s -- der nimmt die Qt-Dateien in "
        "den parallelen Durchgang und faellt dort in FREMDEN Dateien um"
        % ", ".join(treffer))


def test_die_qt_dateien_sind_wirklich_ausgenommen(befehle):
    """Gegenprobe zur Absicht: der parallele Durchgang muss die Qt-Dateien
    ausschliessen UND es muss einen eigenen Durchgang fuer sie geben."""
    assert 'not qt' in befehle[0], befehle[0]
    assert "qt_tests_einzeln.py" in befehle[1], befehle[1]


def test_der_qt_laeufer_existiert():
    assert (_ROOT / "tools" / "qt_tests_einzeln.py").exists()
