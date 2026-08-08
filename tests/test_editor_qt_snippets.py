"""Tests fuer Snippet-Expansion."""
from gamebasic.editor_qt.snippets import SNIPPETS, expand_snippet


def test_snippets_dict_has_basics():
    """Sanity: ein paar Trigger sollten dabei sein."""
    for trigger in ("if", "ife", "for", "wh", "sub", "fn", "cls"):
        assert trigger in SNIPPETS
        body = SNIPPETS[trigger]
        assert isinstance(body, str) and body


def test_expand_without_marker():
    body = "PRINT 1"
    out, offset = expand_snippet(body, indent="")
    assert out == "PRINT 1"
    assert offset == len(out)


def test_expand_with_marker_returns_position():
    body = "IF | THEN"
    out, offset = expand_snippet(body, indent="")
    assert out == "IF  THEN"
    assert offset == 3   # nach "IF " und vor " THEN"


def test_expand_multiline_indents_followups():
    body = "IF | THEN\nA\nEND IF"
    out, offset = expand_snippet(body, indent="    ")
    lines = out.split("\n")
    assert lines[0] == "IF  THEN"
    assert lines[1] == "    A"
    assert lines[2] == "    END IF"
    assert offset == 3


def test_expand_first_line_not_indented():
    """Die erste Zeile uebernimmt KEIN zusaetzliches Indent (steht
    schon an der Cursor-Stelle); nur Folge-Zeilen kriegen das Praefix.
    """
    body = "FOR i = 1 TO |\n    \nNEXT"
    out, _offset = expand_snippet(body, indent="  ")
    lines = out.split("\n")
    assert lines[0] == "FOR i = 1 TO "
    # Folge-Zeilen kriegen "  " vorgehaengt
    assert lines[1].startswith("  ")
    assert lines[2].startswith("  NEXT")


def test_expand_marker_only_first_consumed():
    """Wenn der Body mehrere `|` enthaelt, wird nur das erste entfernt
    (defensives Verhalten -- so funktioniert die aktuelle Implementierung).
    """
    body = "A|B|C"
    out, offset = expand_snippet(body, indent="")
    # `|` an Position 1 wird entfernt, das zweite bleibt.
    assert out == "AB|C"
    assert offset == 1


# --- Placeholders -----------------------------------------------------

from gamebasic.editor_qt.snippets import expand_snippet_full


def test_placeholder_default_is_inserted():
    body = "IF ${1:condition} THEN"
    out, offset, length = expand_snippet_full(body, indent="")
    assert out == "IF condition THEN"
    # Cursor steht am Anfang des Defaults, Selection-Laenge = len("condition")
    assert out[offset:offset + length] == "condition"


def test_placeholder_lowest_index_wins():
    body = "FOR ${1:i} = ${2:1} TO ${3:n}"
    out, offset, length = expand_snippet_full(body, indent="")
    assert out == "FOR i = 1 TO n"
    # primary ist ${1} -> selektiert "i"
    assert out[offset:offset + length] == "i"


def test_placeholder_zero_only_means_no_selection():
    body = "PRINT 1${0}"
    out, offset, length = expand_snippet_full(body, indent="")
    assert out == "PRINT 1"
    assert offset == len(out)
    assert length == 0


def test_placeholder_no_default_zero_length():
    body = "DIM ${1} AS INTEGER"
    out, offset, length = expand_snippet_full(body, indent="")
    assert out == "DIM  AS INTEGER"
    assert length == 0
    # Cursor zwischen DIM und AS
    assert out[:offset] == "DIM "


def test_pipe_marker_still_works_backwards_compat():
    body = "IF | THEN"
    out, offset, length = expand_snippet_full(body, indent="")
    assert out == "IF  THEN"
    assert offset == 3
    assert length == 0


def test_multiline_placeholder_after_indent():
    body = "IF ${1:cond} THEN\n${0}\nEND IF"
    out, offset, length = expand_snippet_full(body, indent="    ")
    assert out.startswith("IF cond THEN")
    assert out[offset:offset + length] == "cond"
    # Folge-Zeilen muessen indentiert sein
    lines = out.split("\n")
    assert lines[1] == "    "
    assert lines[2] == "    END IF"


def test_default_snippets_have_placeholders():
    """Sanity: nach dem Upgrade haben unsere Snippets Placeholders."""
    body = SNIPPETS["if"]
    out, _offset, length = expand_snippet_full(body, indent="")
    assert length > 0  # mindestens ein selektierbarer Default
    assert "${" not in out  # keine ungeparste Marker mehr im Output


# ------------------------------------------------------------ Tabellen-Snippets

def test_tabellen_snippets_vorhanden():
    for trigger in ("tab", "tabsel"):
        assert trigger in SNIPPETS, trigger


def test_kein_snippet_enthaelt_ein_rohes_pipe_zeichen():
    """`|` ist im Snippet-Format der Cursor-Anker. Ein Trennzeichen im Text
    (z.B. SPLIT$("a|b", "|")) wird dadurch still verschluckt -- der Compiler
    meldet nichts, weil ein kaputter String-Inhalt einwandfrei uebersetzt.
    Genau das ist beim Tabellen-Snippet passiert."""
    from gamebasic.editor_qt.snippets import expand_snippet_full
    for trigger, body in SNIPPETS.items():
        aus, _, _ = expand_snippet_full(body, "")
        assert "|" not in aus, f"Snippet '{trigger}' verliert Zeichen am Cursor-Anker: {aus!r}"


def test_tabellen_snippet_ergibt_uebersetzbaren_code(tmp_path):
    """Ein Snippet, das nicht compiliert, ist schlimmer als keines."""
    import os
    import subprocess
    from pathlib import Path

    import pytest

    from gamebasic.editor_qt.snippets import expand_snippet_full

    root = Path(__file__).resolve().parent.parent
    exe = "gbrt.exe" if os.name == "nt" else "gbrt"
    gbrt = next((root / "rust" / "gb_runtime" / "target" / v / exe
                 for v in ("release", "debug")
                 if (root / "rust" / "gb_runtime" / "target" / v / exe).exists()), None)
    if gbrt is None:
        pytest.skip("native Runtime 'gbrt' nicht gebaut")

    aufbau, _, _ = expand_snippet_full(SNIPPETS["tab"], "")
    abfrage, _, _ = expand_snippet_full(SNIPPETS["tabsel"], "")
    quelle = (
        'IMPORT "gui"\n'
        'SCREEN(640, 480, "T", 1)\n'
        "DIM win AS GUI_WINDOW\n"
        'win = GUI_WINDOW("T", 10, 10, 600, 400)\n'
        + aufbau + "\n" + abfrage + "\n"
    )
    f = tmp_path / "snip.gb"
    f.write_text(quelle, encoding="utf-8")
    r = subprocess.run([str(gbrt), "--check", str(f)], capture_output=True,
                       text=True, encoding="utf-8", timeout=60)
    assert r.returncode == 0, r.stderr
    assert r.stdout.strip() in ("[]", ""), r.stdout
