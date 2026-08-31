"""Die Werkzeuge um die Sprache herum (Punkt 6 aus docs/allzweck-audit-2.md).

`--version`, `--help` und `dhrt test` -- die Zeichen, an denen jemand
abliest, ob eine Sprache zum Arbeiten taugt. Die Bausteine fuer den
Test-Laeufer (ASSERT, ASSERT_COLLECT, ASSERT_REPORT, Rueckgabewert) gibt es
seit WP E; was fehlte, war das Dach.
"""
import os
import re
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


def _dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    for v in ("release", "debug"):
        p = _ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
        if p.exists():
            return p
    return None


_DHRT = _dhrt()
pytestmark = pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")


def _lauf(*args, cwd=None, timeout=120):
    r = subprocess.run([str(_DHRT), *args], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", cwd=cwd, timeout=timeout)
    return r.returncode, (r.stdout or ""), (r.stderr or "")


# ----------------------------------------------------------------- Fassung
def _fassung_aus_pyproject() -> str:
    text = (_ROOT / "pyproject.toml").read_text(encoding="utf-8")
    return re.search(r'^version = "([^"]+)"', text, re.M).group(1)


def test_version_wird_ueberhaupt_beantwortet():
    """`dhrt --version` antwortete frueher mit 'Kann --version nicht lesen' --
    es hielt den Namen fuer eine Datei. Das ist das Erste, was jemand tippt,
    der wissen will, ob er die richtige Fassung hat."""
    code, out, _ = _lauf("--version")
    assert code == 0
    assert out.startswith("dhrt ")


def test_version_stimmt_mit_dem_projekt_ueberein():
    """Drei Stellen nennen die Fassung -- sie muessen dasselbe sagen."""
    erwartet = _fassung_aus_pyproject()
    _, out, _ = _lauf("--version")
    assert out.split("\n")[0].strip() == f"dhrt {erwartet}"

    cargo = (_ROOT / "rust" / "drachenhauch_runtime" / "Cargo.toml").read_text(encoding="utf-8")
    cargo_v = re.search(r'^version = "([^"]+)"', cargo, re.M).group(1)
    # Cargo verlangt drei Stellen; "2026.8" wird dort zu "2026.8.0".
    assert cargo_v in (erwartet, erwartet + ".0"), cargo_v

    init = (_ROOT / "drachenhauch" / "__init__.py").read_text(encoding="utf-8")
    assert re.search(r'__version__ = "([^"]+)"', init).group(1) == erwartet


def test_version_nennt_die_eingebauten_teile():
    """Ein Bau ohne --hardware laesst Module weg, ohne dass man es dem
    Binary ansieht -- die Meldung kam sonst erst beim ersten Aufruf."""
    _, out, _ = _lauf("--version")
    assert "dabei:" in out


def test_hilfe_nennt_die_unterbefehle():
    code, out, _ = _lauf("--help")
    assert code == 0
    for wort in ("run", "test", "--check", "--export", "--version"):
        assert wort in out, out


# ------------------------------------------------------------ dhrt test
GUT = 'ASSERT(1 + 1 = 2, "Addition")\nPRINT "alles gut"\n'
SCHLECHT = ("ASSERT_COLLECT(TRUE)\n"
            'ASSERT_EQ(2 + 2, 5, "Rechnen")\n'
            "ASSERT_REPORT()\n"
            "IF ASSERT_FAILED() > 0 THEN EXIT(1)\n")


def test_findet_und_besteht(tmp_path):
    (tmp_path / "a_pruefung.dh").write_text(GUT, encoding="utf-8")
    (tmp_path / "b_pruefung.dh").write_text(GUT, encoding="utf-8")
    code, out, _ = _lauf("test", str(tmp_path))
    assert code == 0, out
    assert out.count("  ok ") == 2
    assert "2 Datei(en), 2 ok, 0 mit Fehlern" in out


def test_ein_fehlschlag_faerbt_den_ganzen_lauf(tmp_path):
    (tmp_path / "a_pruefung.dh").write_text(GUT, encoding="utf-8")
    (tmp_path / "b_pruefung.dh").write_text(SCHLECHT, encoding="utf-8")
    code, out, _ = _lauf("test", str(tmp_path))
    assert code == 1, out
    assert "1 ok, 1 mit Fehlern" in out
    # Die Einzelheiten des Fehlschlags muessen mit heraus -- sonst weiss
    # niemand, WAS schiefging.
    assert "Rechnen" in out, out


def test_sucht_rekursiv(tmp_path):
    tief = tmp_path / "eins" / "zwei"
    tief.mkdir(parents=True)
    (tief / "tief_pruefung.dh").write_text(GUT, encoding="utf-8")
    code, out, _ = _lauf("test", str(tmp_path))
    assert code == 0, out
    assert "1 Datei(en)" in out


def test_nur_dateien_mit_der_endung(tmp_path):
    (tmp_path / "echt_pruefung.dh").write_text(GUT, encoding="utf-8")
    (tmp_path / "normal.dh").write_text('PRINT "kein Pruefprogramm"\n', encoding="utf-8")
    _, out, _ = _lauf("test", str(tmp_path))
    assert "1 Datei(en)" in out
    assert "normal.dh" not in out


def test_eine_genannte_datei_laeuft_auch_ohne_die_endung(tmp_path):
    """Wer sie hinschreibt, meint sie."""
    f = tmp_path / "anders.dh"
    f.write_text(GUT, encoding="utf-8")
    code, out, _ = _lauf("test", str(f))
    assert code == 0, out
    assert "1 Datei(en), 1 ok" in out


def test_nichts_gefunden_ist_kein_fehlschlag(tmp_path):
    """Sonst faellt eine Kette ueber ein noch leeres Projekt."""
    code, _, err = _lauf("test", str(tmp_path))
    assert code == 0
    assert "keine Pruefprogramme" in err


def test_pfad_gibt_es_nicht(tmp_path):
    code, _, err = _lauf("test", str(tmp_path / "weg"))
    assert code == 2
    assert "gibt es nicht" in err


def test_vergessenes_input_haengt_nicht(tmp_path):
    """Die Standardeingabe des Kindes ist leer -- ein Pruefprogramm mit einem
    vergessenen INPUT wuerde sonst auf eine Eingabe warten, die nie kommt,
    und der ganze Lauf haengt."""
    (tmp_path / "x_pruefung.dh").write_text(
        'DIM s AS STRING\nINPUT "Name: ", s\nASSERT(s = "", "leer bei EOF")\n',
        encoding="utf-8")
    code, out, _ = _lauf("test", str(tmp_path), timeout=30)
    assert code == 0, out


def test_die_pruefprogramme_des_repos_laufen():
    """Der Beleg am echten Bestand: die vier Pruefprogramme des Tippspiels."""
    ziel = _ROOT / "buch-tippspiel" / "code"
    if not ziel.is_dir():
        pytest.skip("buch-tippspiel/code fehlt")
    code, out, _ = _lauf("test", str(ziel))
    assert code == 0, out
    assert "0 mit Fehlern" in out


# ------------------------------------------------------------- dhrt fmt
def test_schluesselwoerter_werden_gross(tmp_path):
    f = tmp_path / "a.dh"
    f.write_text("dim i as integer\nfor i = 1 to 3\n    print i\nnext\n", encoding="utf-8")
    code, out, _ = _lauf("fmt", str(f))
    assert code == 0, out
    assert f.read_text(encoding="utf-8") == (
        "DIM i AS INTEGER\nFOR i = 1 TO 3\n    PRINT i\nNEXT\n")


def test_namen_bleiben_wie_sie_sind(tmp_path):
    """Nur SCHLUESSELWOERTER -- Variablen, Builtins und Klassen nicht."""
    f = tmp_path / "a.dh"
    quelle = 'DIM meinWert AS STRING\nmeinWert = Left$("abc", 2)\nPRINT meinWert\n'
    f.write_text(quelle, encoding="utf-8")
    _lauf("fmt", str(f))
    assert "meinWert" in f.read_text(encoding="utf-8")
    assert "Left$" in f.read_text(encoding="utf-8")


def test_zeichenketten_und_kommentare_bleiben_unberuehrt(tmp_path):
    """Entschieden wird an den Token, nicht am Text."""
    f = tmp_path / "a.dh"
    quelle = 'PRINT "das end vom lied"    \' hier steht auch ein for\n'
    f.write_text(quelle, encoding="utf-8")
    _lauf("fmt", str(f))
    neu = f.read_text(encoding="utf-8")
    assert '"das end vom lied"' in neu
    assert "hier steht auch ein for" in neu


def test_f_string_praefix_bleibt_klein(tmp_path):
    """Der Lexer loest einen f-String schon beim Lesen in eine Token-Folge
    auf; die erben alle die Position des `f`. Ohne Gegenprobe wurde daraus
    `F"..."` -- beim ersten Lauf ueber den Bestand tatsaechlich passiert."""
    f = tmp_path / "a.dh"
    f.write_text('DIM n AS INTEGER\nn = 5\nPRINT f"Wert {n}"\n', encoding="utf-8")
    _lauf("fmt", str(f))
    assert 'f"Wert {n}"' in f.read_text(encoding="utf-8")


def test_die_einrueckung_bleibt_ohne_flag_unangetastet(tmp_path):
    """Die Vorgabe ist verlustfrei: sie ruehrt kein Layout an."""
    f = tmp_path / "a.dh"
    quelle = "if 1 = 1 then\n        PRINT 1\nend if\n"
    f.write_text(quelle, encoding="utf-8")
    _lauf("fmt", str(f))
    assert "        PRINT 1" in f.read_text(encoding="utf-8")


def test_mit_flag_wird_eingerueckt(tmp_path):
    f = tmp_path / "a.dh"
    f.write_text("IF 1 = 1 THEN\nPRINT 1\nEND IF\n", encoding="utf-8")
    code, out, _ = _lauf("fmt", "--einruecken", str(f))
    assert code == 0, out
    assert f.read_text(encoding="utf-8") == "IF 1 = 1 THEN\n    PRINT 1\nEND IF\n"


def test_select_case_folgt_dem_hausstil(tmp_path):
    """CASE steht INNERHALB von SELECT -- so schreiben es alle Beispiele.
    Die erste, stapellose Fassung war hier anderer Meinung als 26 Dateien."""
    f = tmp_path / "a.dh"
    f.write_text("DIM x AS INTEGER\nx = 1\nSELECT CASE x\nCASE 1\nPRINT 1\n"
                 "CASE ELSE\nPRINT 0\nEND SELECT\n", encoding="utf-8")
    _lauf("fmt", "--einruecken", str(f))
    assert f.read_text(encoding="utf-8") == (
        "DIM x AS INTEGER\n"
        "x = 1\n"
        "SELECT CASE x\n"
        "    CASE 1\n"
        "        PRINT 1\n"
        "    CASE ELSE\n"
        "        PRINT 0\n"
        "END SELECT\n")


def test_einzeiliges_if_rueckt_nicht_ein(tmp_path):
    f = tmp_path / "a.dh"
    f.write_text("DIM x AS INTEGER\nIF x = 1 THEN x = 2\nPRINT x\n", encoding="utf-8")
    _lauf("fmt", "--einruecken", str(f))
    assert f.read_text(encoding="utf-8").endswith("IF x = 1 THEN x = 2\nPRINT x\n")


def test_fortsetzungszeilen_bleiben_ausgerichtet(tmp_path):
    """Wer seine Parameter untereinander ausrichtet, hat sich etwas dabei
    gedacht -- und die Zeile darf die Ebenen-Rechnung nicht verwirren."""
    f = tmp_path / "a.dh"
    quelle = ("FOR i = 1 TO 2\n"
              "    IF MAX(1, _\n"
              "           2) = 2 THEN\n"
              "        PRINT 1\n"
              "    END IF\n"
              "NEXT\n"
              "PRINT 9\n")
    f.write_text("DIM i AS INTEGER\n" + quelle, encoding="utf-8")
    _lauf("fmt", "--einruecken", str(f))
    neu = f.read_text(encoding="utf-8")
    assert "           2) = 2 THEN" in neu, neu     # Ausrichtung erhalten
    assert neu.endswith("NEXT\nPRINT 9\n"), neu     # Ebene laeuft nicht weg


def test_pruefen_schreibt_nicht_und_meldet_sich(tmp_path):
    f = tmp_path / "a.dh"
    f.write_text("print 1\n", encoding="utf-8")
    code, out, _ = _lauf("fmt", "--pruefen", str(f))
    assert code == 1                                # fuer eine Kette
    assert "wuerde sich aendern" in out
    assert f.read_text(encoding="utf-8") == "print 1\n"


def test_saubere_datei_bleibt_unberuehrt(tmp_path):
    f = tmp_path / "a.dh"
    f.write_text("PRINT 1\n", encoding="utf-8")
    code, out, _ = _lauf("fmt", "--pruefen", str(f))
    assert code == 0
    assert out.strip() == ""


def test_kaputte_datei_wird_nicht_angefasst(tmp_path):
    """An kaputtem Code herumzuruecken hilft niemandem."""
    f = tmp_path / "a.dh"
    quelle = 'PRINT "unbeendete Zeichenkette\n'
    f.write_text(quelle, encoding="utf-8")
    code, _, err = _lauf("fmt", str(f))
    assert code == 2
    assert f.read_text(encoding="utf-8") == quelle
    assert "unveraendert" in err


def test_der_bestand_ist_sauber():
    """Nach dem einmaligen Durchlauf muss jede .dh-Datei des Repos ruhig
    bleiben -- sonst waere die Vorgabe nicht verlustfrei."""
    # Temp-Reste der IDE ueberspringen: sie liegen absichtlich neben der
    # Quelle und koennen nach einem Abbruch liegenbleiben -- sie sind nicht
    # Teil des Bestands (siehe editor_qt/tempdateien.py).
    from drachenhauch.editor_qt.tempdateien import PRAEFIX
    dateien = [str(p) for p in (_ROOT / "examples").rglob("*.dh")
               if not p.name.startswith(PRAEFIX)]
    code, out, _ = _lauf("fmt", "--pruefen", *dateien)
    assert code == 0, out


# ------------------------------------------------------ Referenz erzeugen
def _doku(quelle: str, tmp_path):
    """`dhrun.py --doku` auf eine Datei anwenden und die Markdown-Seite
    zurueckgeben."""
    import sys
    f = tmp_path / "mathe.dh"
    f.write_text(quelle, encoding="utf-8")
    venv = _ROOT / ".venv" / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
    py = str(venv) if venv.exists() else sys.executable
    r = subprocess.run([py, str(_ROOT / "dhrun.py"), "--doku", str(f)],
                       capture_output=True, text=True, encoding="utf-8",
                       cwd=str(_ROOT), timeout=60)
    assert r.returncode == 0, r.stderr
    return r.stdout


BIB = '''\' Kleine Sammlung fuer Streckenrechnung.

\' Der groesste erlaubte Abstand.
CONST GRENZE AS FLOAT = 1000.0

\' Abstand zweier Punkte in der Ebene.
\' Liefert immer einen positiven Wert.
FUNCTION Distanz(x1 AS FLOAT, y1 AS FLOAT) AS FLOAT
    RETURN x1 + y1
END FUNCTION

\' Nur fuer den internen Gebrauch.
PRIVATE SUB pruefe(wert AS FLOAT)
    PRINT wert
END SUB
'''


def test_doku_nimmt_signatur_und_kommentar(tmp_path):
    md = _doku(BIB, tmp_path)
    assert "FUNCTION Distanz(x1 AS FLOAT, y1 AS FLOAT) AS FLOAT" in md
    assert "Abstand zweier Punkte in der Ebene." in md
    assert "Liefert immer einen positiven Wert." in md


def test_doku_beschreibt_die_datei_aus_ihrem_kopf(tmp_path):
    md = _doku(BIB, tmp_path)
    assert "Kleine Sammlung fuer Streckenrechnung." in md


def test_doku_laesst_privates_weg(tmp_path):
    """`PRIVATE` gehoert dem Modul -- eine Referenz, die es auffuehrt,
    verspricht etwas, das beim naechsten Umbau verschwindet."""
    md = _doku(BIB, tmp_path)
    assert "pruefe" not in md


def test_doku_ohne_datei_meldet_sich(tmp_path):
    import sys
    venv = _ROOT / ".venv" / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
    py = str(venv) if venv.exists() else sys.executable
    r = subprocess.run([py, str(_ROOT / "dhrun.py"), "--doku"],
                       capture_output=True, text=True, encoding="utf-8",
                       cwd=str(_ROOT), timeout=60)
    assert r.returncode == 2
    assert "Verwendung" in r.stdout
