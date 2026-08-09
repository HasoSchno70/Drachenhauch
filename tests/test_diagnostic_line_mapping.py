"""Diagnose-Zeilen muessen auf Puffer-Zeilen zeigen, auch mit IMPORT.

Aus dem Clean-Code-Review des Python-Frontends. dhrt preprocesst intern
selbst und meldet Diagnosen deshalb in Zeilen der GEMERGTEN Quelle --
`main.rs::check_main` sagt das ausdruecklich ("der Editor mappt via
origins zurueck"). Genau diese Haelfte fehlte in `_check_via_dhrt`: die
Zeilen wurden woertlich uebernommen, wodurch in JEDER Datei mit
`IMPORT "x.gb"` saemtliche Marker um die Laenge des inlinierten Codes
verrutschten -- im Reproduktionsfall bis hinter das Dateiende.

`_map_back` selbst existierte und war korrekt, wurde aber nur vom
Fallback `_check_syntax_only` benutzt und hatte NULL Testabdeckung --
deshalb ist der Fehler nie aufgefallen. Diese Datei schliesst beides.
"""
from pathlib import Path

import pytest

from drachenhauch.editor_qt.error_check import _check_source, _map_back
from drachenhauch.preprocess import process
from drachenhauch.errors import LexerError


HELPER = """\
' helper 1
' helper 2
FUNCTION helper_add(a AS INTEGER, b AS INTEGER) AS INTEGER
    RETURN a + b
END FUNCTION
' helper 6
' helper 7
' helper 8
"""


@pytest.fixture
def proj(tmp_path):
    (tmp_path / "helper.gb").write_text(HELPER, encoding="utf-8")
    return tmp_path


def test_error_after_import_maps_to_buffer_line(proj):
    """Der Kern-Regressionsfall: 8 Zeilen werden inliniert, der Fehler steht
    auf Puffer-Zeile 4 -- gemeldet wurde vorher Zeile 14 (hinter EOF)."""
    buf = ('IMPORT "helper.gb"\n'
           'DIM x AS INTEGER\n'
           'x = helper_add(1, 2)\n'
           'x = = 5\n')
    probs = [p for p in _check_source(buf, proj) if p.severity == "error"]
    assert probs, "Fehler haette gemeldet werden muessen"
    assert probs[0].line == 4, f"erwartet Puffer-Zeile 4, bekam {probs[0].line}"


def test_error_line_never_exceeds_buffer_length(proj):
    """Egal wie viel inliniert wird -- eine Fehlerzeile hinter dem Dateiende
    ist immer falsch und im Editor nicht anzeigbar."""
    buf = 'IMPORT "helper.gb"\nx = = 1\n'
    n_lines = len(buf.splitlines())
    for p in _check_source(buf, proj):
        assert 1 <= p.line <= n_lines, (
            f"Zeile {p.line} liegt ausserhalb des {n_lines}-Zeilen-Puffers: {p.message}")


def test_error_inside_imported_file_pins_to_line_one_and_names_file(tmp_path):
    """Ein Fehler IN der importierten Datei hat keine Entsprechung im Puffer
    des Nutzers -- er wird auf Zeile 1 gepinnt und die Datei in den Text
    gezogen, statt eine unbeteiligte Puffer-Zeile zu markieren."""
    (tmp_path / "broken.gb").write_text("' a\n' b\nx = = 9\n", encoding="utf-8")
    probs = [p for p in _check_source('IMPORT "broken.gb"\nPRINT 1\n', tmp_path)
             if p.severity == "error"]
    assert probs
    assert probs[0].line == 1
    assert "broken.gb" in probs[0].message


def test_hardware_warning_line_is_not_shifted(proj):
    """dhrt liefert in EINEM Array drei Koordinatensysteme: lex/parse/compile
    beziehen sich auf die gemergte Quelle, die Hardware-Import-Warnungen
    werden dagegen aus der ROHEN Quelle berechnet und sind schon
    Puffer-Zeilen. Alles blind zu mappen wuerde genau diese korrekten
    Warnungen kaputtmachen."""
    buf = 'IMPORT "helper.gb"\nIMPORT "serial"\nPRINT 1\n'
    warns = [p for p in _check_source(buf, proj) if p.severity == "warning"]
    hw = [p for p in warns if "serial" in p.message]
    if not hw:
        pytest.skip("dhrt-Build enthaelt das serial-Modul -- keine Warnung erwartet")
    assert hw[0].line == 2, (
        f"Hardware-Warnung gehoert auf die IMPORT-Zeile 2, bekam {hw[0].line}")


def test_no_import_lines_are_unchanged(tmp_path):
    """Regression: ohne IMPORT darf das Mapping nichts verschieben."""
    probs = [p for p in _check_source('PRINT 1\nx = = 2\n', tmp_path)
             if p.severity == "error"]
    assert probs
    assert probs[0].line == 2


# --- origins / _map_back direkt -------------------------------------

def test_origins_roundtrip_maps_every_buffer_line(proj):
    """Jede Zeile des Nutzer-Puffers muss sich aus der gemergten Quelle
    exakt zurueckgewinnen lassen."""
    buf = 'PRINT 1\nIMPORT "helper.gb"\nPRINT 2\nPRINT 3\n'
    merged, origins = process(buf, proj, file_label="<editor>")
    seen_buffer_lines = set()
    for merged_line in range(1, len(merged.split("\n")) + 1):
        origin = origins[merged_line]
        if not origin or origin[0] != "<editor>":
            continue                      # stammt aus der importierten Datei
        mapped, _msg = _map_back(origins, merged_line, "x")
        assert mapped == origin[1], (
            f"merged {merged_line} -> {mapped}, erwartet {origin[1]}")
        seen_buffer_lines.add(mapped)
    # Jede Zeile des Puffers (inkl. der leeren Schlusszeile durch das
    # abschliessende \n) muss vertreten sein -- keine darf verlorengehen.
    assert seen_buffer_lines == set(range(1, len(buf.split("\n")) + 1))


def test_nested_import_error_reports_the_users_own_import_line(tmp_path):
    """Ein IMPORT-Fehler tiefer in der Kette trug bisher die Zeile der
    INNEREN Datei nach oben -- der Editor markierte damit eine voellig
    unbeteiligte Zeile. Die einzige Koordinate, die der Nutzer anfassen
    kann, ist seine eigene IMPORT-Zeile."""
    (tmp_path / "outer.gb").write_text('\' a\n\' b\nIMPORT "nichtda.gb"\n',
                                       encoding="utf-8")
    buf = "PRINT 1\nPRINT 2\nPRINT 3\nPRINT 4\nPRINT 5\nIMPORT \"outer.gb\"\n"
    with pytest.raises(LexerError) as ei:
        process(buf, tmp_path, file_label="<editor>")
    assert ei.value.line == 6, f"erwartet Zeile 6, bekam {ei.value.line}"
    assert "outer.gb" in str(ei.value)


def test_origins_distinguish_same_named_files_in_different_dirs(tmp_path):
    """`origins` trug nur den Dateinamen -- zwei `util.gb` aus verschiedenen
    Verzeichnissen waren nicht unterscheidbar."""
    (tmp_path / "a").mkdir()
    (tmp_path / "b").mkdir()
    (tmp_path / "a" / "util.gb").write_text("' aus a\n", encoding="utf-8")
    (tmp_path / "b" / "util.gb").write_text("' aus b\n", encoding="utf-8")
    buf = 'IMPORT "a/util.gb"\nIMPORT "b/util.gb"\n'
    _merged, origins = process(buf, tmp_path, file_label="<editor>")
    labels = {o[0] for o in origins[1:] if o and o[0] != "<editor>"}
    assert len(labels) == 2, f"Labels nicht unterscheidbar: {labels}"
