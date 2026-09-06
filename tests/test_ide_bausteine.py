"""Die Bausteine der IDE in Drachenhauch (Weg C aus docs/entwurf-python-abbau.md).

Drei Luecken nannte der Entwurf vor die IDE: ein Prozess mit LAUFENDER
Ausgabe (`PROCESS_*`), Befehle fuer den Textbereich (Marke, Auswahl, Suchen,
Einfuegen) und Hover/Vervollstaendigung aus dhrt (`CODE_*`, dieselben
Funktionen wie `dhrt lsp`, in-Prozess). Jeder Baustein wird hier fuer sich
geprueft; die IDE selbst prueft `tests/test_ide.py`.

Die Textbereich-Befehle brauchen ein Fenster (GOTO rechnet mit der
Zeilenhoehe) -- darum in `_BRAUCHT_GRAFIK`, mit dem Fenster ausserhalb des
Schirms.
"""
from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path

import pytest

from drachenhauch.errors import DHRuntimeError

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


def _lauf(tmp_path, src, frames=6):
    f = tmp_path / "t.dh"
    f.write_text(src, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(f)], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", timeout=120,
                       env=dict(os.environ, DHRT_FRAMES=str(frames)), cwd=str(tmp_path))
    assert r.returncode == 0, (r.stdout, r.stderr)
    return [ln.rstrip() for ln in (r.stdout or "").splitlines()
            if ln.strip() and not ln.startswith(("WARNING:", "INFO:"))]


# ------------------------------------------------------------ PROCESS_*

def test_prozess_liefert_ausgabe_waehrend_er_laeuft(run_gb, tmp_path):
    """Das Kind druckt drei Zeilen mit Pausen; der Elternprozess liest sie
    STUECKWEISE -- SHELL_START haette alles erst am Ende geliefert."""
    (tmp_path / "kind.dh").write_text(
        'PRINT "eins"\nSLEEP(300)\nPRINT "zwei"\nSLEEP(300)\nPRINT "drei"\n', encoding="utf-8")
    out = run_gb(
        'DIM p AS INTEGER : p = PROCESS_START("dhrt", "run", "kind.dh")\n'
        'DIM runden AS INTEGER : runden = 0\n'
        'DIM teile AS INTEGER : teile = 0\n'
        'DIM alles AS STRING : alles = ""\n'
        'WHILE PROCESS_RUNNING(p) AND runden < 400\n'
        '    DIM t AS STRING : t = PROCESS_READ$(p)\n'
        '    IF t <> "" THEN teile = teile + 1 : alles = alles + t\n'
        '    SLEEP(20) : runden = runden + 1\n'
        'WEND\n'
        'alles = alles + PROCESS_READ$(p)\n'
        'PRINT teile >= 2\n'
        'PRINT TRIM$(REPLACE$(alles, CHR$(13), ""))\n'
        'PRINT PROCESS_CODE(p)\n'
        'PROCESS_CLOSE(p)\n', base=tmp_path)
    assert out == "TRUE\neins\nzwei\ndrei\n0\n", out


def test_prozess_eingabe_und_stderr(run_gb, tmp_path):
    (tmp_path / "frage.dh").write_text(
        'DIM n AS STRING\nINPUT n\nPRINT "hallo " + n\nEXIT(3)\n', encoding="utf-8")
    out = run_gb(
        'DIM p AS INTEGER : p = PROCESS_START("dhrt", "run", "frage.dh")\n'
        'PROCESS_WRITE(p, "Welt" + CHR$(10))\n'
        'DIM runden AS INTEGER : runden = 0\n'
        'WHILE PROCESS_RUNNING(p) AND runden < 400 : SLEEP(20) : runden = runden + 1 : WEND\n'
        'PRINT TRIM$(REPLACE$(PROCESS_READ$(p), CHR$(13), ""))\n'
        'PRINT PROCESS_CODE(p)\n', base=tmp_path)
    assert out == "? hallo Welt\n3\n", out  # "? " ist die INPUT-Aufforderung


def test_prozess_kill_und_fehler(run_gb, tmp_path):
    (tmp_path / "ewig.dh").write_text('WHILE TRUE : SLEEP(50) : WEND\n', encoding="utf-8")
    out = run_gb(
        'DIM p AS INTEGER : p = PROCESS_START("dhrt", "run", "ewig.dh")\n'
        'SLEEP(200)\n'
        'PRINT PROCESS_RUNNING(p)\n'
        'PROCESS_KILL(p)\n'
        'PRINT PROCESS_RUNNING(p)\n'
        'PRINT PROCESS_CODE(p)\n', base=tmp_path)
    assert out == "TRUE\nFALSE\n-1\n", out
    with pytest.raises(DHRuntimeError, match="PROCESS_START: 'gibt_es_nicht_xyz' laesst sich nicht starten"):
        run_gb('PROCESS_START("gibt_es_nicht_xyz")\n')


# ------------------------------------------------------------ CODE_*

SRC = ("' Spieler-Klasse\\n' Zweite Zeile.\\nCLASS Player\\n    SUB Init()\\n    END SUB\\nEND CLASS\\n"
       "FUNCTION add(a AS INTEGER, b AS INTEGER) AS INTEGER\\n    RETURN a + b\\nEND FUNCTION\\n"
       "DIM r AS INTEGER\\nr = add(1, 2)\\nr = add(3, 4)\\n")
KOPF = 'DIM q AS STRING : q = REPLACE$("' + SRC + '", "\\n", CHR$(10))\n'


def test_code_check_meldet_zeilen_des_puffers(run_gb):
    out = run_gb('PRINT CODE_CHECK$("PRINT 1" + CHR$(10) + "DIM x AS" + CHR$(10))\n'
                 'PRINT CODE_CHECK$("PRINT 1")\n')
    erste, zweite = out.splitlines()
    d = json.loads(erste)
    assert len(d) == 1 and d[0]["zeile"] == 2 and d[0]["schwere"] == "fehler", d
    assert json.loads(zweite) == []


def test_code_check_warnung_und_import(run_gb, tmp_path):
    (tmp_path / "helfer.dh").write_text("SUB gruss()\nEND SUB\n", encoding="utf-8")
    out = run_gb(KOPF.replace(SRC, '') +
                 'PRINT CODE_CHECK$("IMPORT " + CHR$(34) + "helfer.dh" + CHR$(34) + CHR$(10) + "gruss()" + CHR$(10) + "DIM x AS", "' + str(tmp_path).replace("\\", "/") + '")\n',
                 base=tmp_path)
    d = json.loads(out.splitlines()[-1])
    assert len(d) == 1 and d[0]["zeile"] == 3, d


def test_code_hover_complete_definition_references(run_gb):
    out = run_gb(KOPF +
                 'PRINT INSTR(CODE_HOVER$(q, 11, 6), "FUNCTION add(a AS INTEGER, b AS INTEGER) AS INTEGER") >= 0\n'
                 'PRINT INSTR(CODE_HOVER$(q, 3, 8), "Spieler-Klasse") >= 0\n'
                 'PRINT INSTR(UPPER$(CODE_HOVER$("x = ABS(1)", 1, 6)), "ABS") >= 0\n'
                 'PRINT CODE_HOVER$("   ", 1, 2) = ""\n'
                 'DIM v AS ARRAY OF STRING : v = CODE_COMPLETE(q + "Pl", 13, 3)\n'
                 'PRINT v[0]\n'
                 'PRINT LEN(CODE_COMPLETE("PRI", 1, 4)) > 0\n'
                 'DIM z AS INTEGER\nDIM s AS INTEGER\n'
                 '(z, s) = CODE_DEFINITION(q, 11, 6)\n'
                 'PRINT z ; " " ; s\n'
                 '(z, s) = CODE_DEFINITION("PRINT 1", 1, 2)\n'
                 'PRINT z\n'
                 'DIM r AS ARRAY OF INTEGER : r = CODE_REFERENCES(q, 7, 11)\n'
                 'PRINT LEN(r); " "; r[0]; " "; r[1]; " "; r[2]\n')
    assert out == "TRUE\nTRUE\nTRUE\nTRUE\nPlayer\nTRUE\n7 10\n-1\n3 7 11 12\n", out


def test_code_symbols_verschachtelt(run_gb):
    out = run_gb(KOPF + 'PRINT CODE_SYMBOLS$(q)\n')
    s = json.loads(out.strip())
    namen = [e["name"] for e in s]
    assert namen == ["Player", "add"]
    assert s[0]["art"] == "class" and s[0]["von"] == 3 and s[0]["bis"] == 6
    assert s[0]["kinder"][0]["name"] == "Init"


# ------------------------------------------------------------ Textbereich

_TA = ('IMPORT "gui"\n'
       'SCREEN(400, 300, "T", 1)\n'
       'SET_WINDOW_POS(-3000, -3000)\n'
       'DIM w AS GUI_WINDOW : w = GUI_WINDOW("T", 0, 0, 400, 300)\n'
       'DIM ta AS GUI_WIDGET : ta = GUI_TEXTAREA(w, 10, 10, 360, 200)\n'
       'GUI_SET_TEXT(ta, "eins zwei" + CHR$(10) + "drei Zwei" + CHR$(10) + "fuenf")\n'
       'GUI_UPDATE()\n')


def test_textbereich_marke_auswahl_suchen(tmp_path):
    out = _lauf(tmp_path, _TA +
                'DIM z AS INTEGER\nDIM s AS INTEGER\n'
                '(z, s) = GUI_TEXTAREA_CURSOR(ta) : PRINT z ; " " ; s\n'     # nach SET_TEXT am Ende
                'GUI_TEXTAREA_GOTO(ta, 2, 3)\n'
                '(z, s) = GUI_TEXTAREA_CURSOR(ta) : PRINT z ; " " ; s\n'
                'GUI_TEXTAREA_GOTO(ta, 99, 99)\n'                              # geklemmt
                '(z, s) = GUI_TEXTAREA_CURSOR(ta) : PRINT z ; " " ; s\n'
                'GUI_TEXTAREA_SELECT(ta, 1, 6, 2, 5)\n'
                'PRINT GUI_TEXTAREA_SELECTION$(ta)\n'
                '(z, s) = GUI_TEXTAREA_FIND(ta, "zwei") : PRINT z ; " " ; s\n'
                '(z, s) = GUI_TEXTAREA_FIND(ta, "zwei", 1, 7) : PRINT z ; " " ; s\n'    # ab hinter dem ersten
                '(z, s) = GUI_TEXTAREA_FIND(ta, "zwei", 1, 7, TRUE) : PRINT z ; " " ; s\n'  # genau: Zwei zaehlt nicht
                '(z, s) = GUI_TEXTAREA_FIND(ta, "nix") : PRINT z\n'
                'WHILE NOT QUITREQUESTED() : GUI_UPDATE() : CLS(0) : GUI_DRAW() : FLIP() : WEND\n')
    assert out == ["3 6", "2 3", "3 6", "zwei", "drei", "1 6", "2 6", "-1 -1", "-1"], out


def test_textbereich_einfuegen_ist_ein_undo_schritt(tmp_path):
    out = _lauf(tmp_path, _TA +
                'GUI_TEXTAREA_SELECT(ta, 1, 1, 1, 5)\n'
                'GUI_TEXTAREA_INSERT(ta, "EINS")\n'
                'PRINT GUI_TEXT(ta)\n'
                'DIM z AS INTEGER\nDIM s AS INTEGER\n'
                '(z, s) = GUI_TEXTAREA_CURSOR(ta) : PRINT z ; " " ; s\n'
                'GUI_TEXTAREA_GOTO(ta, 3, 6)\n'
                'GUI_TEXTAREA_INSERT(ta, " sechs")\n'
                'PRINT GUI_TEXT(ta)\n'
                'WHILE NOT QUITREQUESTED() : GUI_UPDATE() : CLS(0) : GUI_DRAW() : FLIP() : WEND\n')
    assert out == ["EINS zwei", "drei Zwei", "fuenf", "1 5", "EINS zwei", "drei Zwei", "fuenf sechs"], out


def test_textbereich_marken(tmp_path):
    """Marken haengen an Zeilen; leere Felder loeschen; ungleiche Laengen sind
    ein Fehler. Ob sie ZU SEHEN sind, sagt das Bild (docs/ide.md) -- hier
    steht, dass der Aufruf durchlaeuft und nicht zeichnet, was er nicht soll."""
    out = _lauf(tmp_path, _TA +
                'GUI_TEXTAREA_MARKS(ta, [1, 3], [&HE05050, &HFFD040])\n'
                'GUI_UPDATE() : CLS(0) : GUI_DRAW() : FLIP()\n'
                'GUI_TEXTAREA_MARKS(ta, [], [])\n'
                'PRINT "ok"\n'
                'TRY\n    GUI_TEXTAREA_MARKS(ta, [1, 2], [1])\nCATCH e\n    PRINT e\nEND TRY\n'
                'WHILE NOT QUITREQUESTED() : GUI_UPDATE() : CLS(0) : GUI_DRAW() : FLIP() : WEND\n')
    assert out[0] == "ok" and "gleich lang" in out[1], out


def test_textbereich_befehle_nur_fuer_textareas(tmp_path):
    f = tmp_path / "t.dh"
    f.write_text(_TA + 'DIM b AS GUI_WIDGET : b = GUI_BUTTON(w, "x", 0, 250, 40, 20)\n'
                 'GUI_TEXTAREA_GOTO(b, 1, 1)\n', encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(f)], capture_output=True, text=True, encoding="utf-8",
                       errors="replace", timeout=60, env=dict(os.environ, DHRT_FRAMES="3"), cwd=str(tmp_path))
    assert r.returncode != 0 and "GUI_TEXTAREA_GOTO: das Widget ist kein GUI_TEXTAREA" in r.stderr + r.stdout
