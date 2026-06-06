"""Tests fuer den Profiler-Kern (Qt-frei). `run_profile` laeuft seit Stufe B
ueber `gbrt profile` (native Runtime) -- skippt, wenn gbrt nicht gebaut ist."""
import pytest

from gamebasic.editor_qt.profiler import run_profile, _find_gbrt

pytestmark = pytest.mark.skipif(_find_gbrt() is None, reason="gbrt nicht gebaut")


def test_basic_counts_and_output():
    src = (
        "DIM i AS INTEGER\n"
        "DIM s AS INTEGER\n"
        "s = 0\n"
        "FOR i = 1 TO 100\n"
        "    s = s + i\n"
        "NEXT\n"
        "PRINT s\n"
    )
    r = run_profile(src, ".")
    assert r.output.strip() == "5050"
    by_line = {s.line: s for s in r.lines}
    # Zeile 5 (s = s + i) lief 100x.
    assert by_line[5].count == 100
    # Zeile 3 (s = 0) lief 1x.
    assert by_line[3].count == 1
    # Quelltext wird mitgeliefert.
    assert by_line[5].text == "s = s + i"


def test_hot_line_ranks_first():
    src = (
        "DIM i AS INTEGER\n"
        "DIM x AS INTEGER\n"
        "x = 0\n"
        "FOR i = 1 TO 5000\n"
        "    x = x + 1\n"
        "NEXT\n"
    )
    r = run_profile(src, ".")
    # Zeitlich teuerste Zeile ist die Schleifen-Innenzeile.
    assert r.lines[0].line == 5
    # ~5000 Durchlaeufe; bei zeilenbasiertem Profiling kann die letzte Schleife
    # (als letztes Statement) die Body-Zeile einmal extra zaehlen -> >= 5000.
    assert r.lines[0].count >= 5000


def test_function_aggregation_and_calls():
    src = (
        "FUNCTION sq(n AS INTEGER) AS INTEGER\n"
        "    RETURN n * n\n"
        "END FUNCTION\n"
        "DIM i AS INTEGER\n"
        "DIM t AS INTEGER\n"
        "t = 0\n"
        "FOR i = 1 TO 10\n"
        "    t = t + sq(i)\n"
        "NEXT\n"
    )
    r = run_profile(src, ".")
    funcs = {f.name: f for f in r.funcs}
    assert "sq" in funcs
    # sq wird 10x aufgerufen -> Aufrufzahl ~ Treffer der ersten Body-Zeile.
    assert funcs["sq"].count == 10
    assert funcs["sq"].kind == "function"


def test_total_time_positive():
    r = run_profile("PRINT 1\n", ".")
    assert r.total_time >= 0.0
    assert r.stopped is False


def test_stop_callback_aborts():
    # Endlosschleife -- should_stop bricht nach dem ersten Hook ab.
    src = "DIM i AS INTEGER\ni = 0\nWHILE TRUE\n    i = i + 1\nWEND\n"
    calls = {"n": 0}

    def should_stop():
        calls["n"] += 1
        return calls["n"] > 50      # nach 50 Hooks abbrechen

    r = run_profile(src, ".", should_stop=should_stop)
    assert r.stopped is True
    # Sauberer Abbruch (Stop-Signal statt Prozess-Kill): die bis dahin
    # gesammelten Profile-Daten muessen erhalten bleiben -- sonst bliebe die
    # Auswertung leer (Regression Endlos-Loop/Grafik-Demos wie 99_ibl_hdr.gb).
    assert r.lines, "Stop darf die gesammelten Profile-Daten nicht verwerfen"
    assert any(s.count > 0 for s in r.lines)


def test_error_propagates():
    import pytest
    from gamebasic.errors import GameBasicError
    # Laufzeitfehler (Division durch 0) soll als GameBasicError hochkommen.
    with pytest.raises(GameBasicError):
        run_profile("DIM x AS INTEGER\nx = 1 \\ 0\n", ".")


def test_chdir_to_base_path_for_relative_assets(tmp_path):
    """Der Profiler wechselt ins Datei-Verzeichnis (wie gbrun.py), damit
    relative Laufzeit-Pfade funktionieren -- und stellt das cwd wieder her."""
    import os
    (tmp_path / "data.txt").write_text("hallo\n", encoding="utf-8")
    src = (
        'DIM f AS FILE\n'
        'f = OpenFile("data.txt", "r")\n'
        'PRINT ReadLine(f)\n'
        'CloseFile(f)\n'
    )
    cwd_before = os.getcwd()
    r = run_profile(src, tmp_path)            # base_path != cwd
    assert r.output.strip() == "hallo"        # relativer Pfad gefunden
    assert os.getcwd() == cwd_before          # cwd wiederhergestellt
