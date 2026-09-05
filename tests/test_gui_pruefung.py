"""Formularpruefung im gui-Modul: GUI_RULE, GUI_VALIDATE, GUI_ERROR_LABEL.

Der sechste Pilot (Rechnungen) hat die Pruefung dreimal von Hand
geschrieben -- Pflichtfeld, PLZ, E-Mail, Zahl, Datum, jedes Mal mit
Fehlertext, Fokus und einer roten Beschriftung. Jetzt haengen Regeln am
Feld, EIN Aufruf prueft das Fenster, das erste falsche Feld bekommt den
Fokus, die Meldung steht im Tooltip und in einer gebundenen Beschriftung.

Alles headless: Regeln, Meldungen und Fokus lassen sich ohne Eingabe
pruefen. Die reine Regel-Logik hat zusaetzlich Rust-Tests in gui.rs.
"""
import os
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

_KOPF = ('IMPORT "gui"\n'
         'SCREEN(500, 400, "T", 1)\n'
         'SET_WINDOW_POS(-3000, -3000)\n'
         'DIM w AS GUI_WINDOW\n'
         'w = GUI_WINDOW("T", 0, 0, 500, 400)\n'
         'GUI_WINDOW_CHROME(w, FALSE)\n'
         'DIM name AS GUI_WIDGET : name = GUI_TEXTINPUT(w, 10, 10, 200, 28, "Name")\n'
         'DIM plz AS GUI_WIDGET : plz = GUI_TEXTINPUT(w, 10, 50, 200, 28, "PLZ")\n'
         'DIM mail AS GUI_WIDGET : mail = GUI_TEXTINPUT(w, 10, 90, 200, 28, "")\n'
         'DIM lbl AS GUI_WIDGET : lbl = GUI_LABEL(w, "", 220, 10)\n'
         'DIM dd AS GUI_WIDGET : dd = GUI_DROPDOWN(w, 10, 130, 200, 28, ["a", "b"])\n'
         'DIM cb AS GUI_WIDGET : cb = GUI_CHECKBOX(w, "AGB", 10, 170)\n'
         'GUI_RULE(name, "pflicht", "Der Name fehlt.")\n'
         'GUI_RULE(name, "laenge", 2, 40)\n'
         'GUI_RULE(plz, "muster", "[0-9]{5}", "PLZ: fuenf Ziffern")\n'
         'GUI_RULE(mail, "email")\n'
         'GUI_RULE(dd, "pflicht", "Bitte waehlen")\n'
         'GUI_RULE(cb, "pflicht", "Bitte zustimmen")\n'
         'GUI_ERROR_LABEL(name, lbl)\n'
         'GUI_UPDATE()\n')


def _lauf(tmp_path, src, frames=1):
    f = tmp_path / "t.dh"
    f.write_text(src, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(f)], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", timeout=90,
                       env=dict(os.environ, DHRT_FRAMES=str(frames)), cwd=str(tmp_path))
    assert r.returncode == 0, r.stderr
    return [ln.strip() for ln in (r.stdout or "").splitlines()
            if ln.strip() and not ln.startswith(("WARNING:", "INFO:"))]


def test_validate_prueft_alle_felder_und_fokussiert_das_erste(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'PRINT GUI_VALIDATE(w) ; "|" ; GUI_ERROR(name) ; "|" ; GUI_TEXT(lbl) ; "|" ; GUI_ERROR(plz) ; "|" ; GUI_ERROR(mail) ; "|" ; GUI_ERROR(dd) ; "|" ; GUI_ERROR(cb)\n'
                'PRINT GUI_FOCUSED() = name\n')
    teile = out[0].split("|")
    assert teile[0] == "2", "leer: Name und Kaestchen fehlen -- PLZ und Mail duerfen leer sein, die Klappliste steht auf dem ersten Eintrag"
    assert teile[1] == "Der Name fehlt." and teile[2] == "Der Name fehlt.", "eigene Meldung, auch in der Beschriftung"
    assert teile[3] == "" and teile[4] == ""
    assert teile[5] == "" and teile[6] == "Bitte zustimmen"
    assert out[1] == "TRUE", "das erste falsche Feld bekommt den Fokus"


def test_regeln_greifen_nacheinander_und_loesen_sich_auf(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'GUI_SET_TEXT(name, "A") : GUI_SET_TEXT(plz, "1234") : GUI_SET_TEXT(mail, "x@y") : GUI_DROPDOWN_SET_SELECTED(dd, 1) : GUI_SET_CHECKED(cb, TRUE)\n'
                'PRINT GUI_VALIDATE(w) ; "|" ; GUI_ERROR(name) ; "|" ; GUI_ERROR(plz) ; "|" ; GUI_ERROR(mail)\n'
                'GUI_SET_TEXT(name, "Anna") : GUI_SET_TEXT(plz, "12345") : GUI_SET_TEXT(mail, "x@y.de")\n'
                'PRINT GUI_VALIDATE(w) ; "|" ; GUI_TEXT(lbl) ; "|" ; GUI_ERROR(plz)\n')
    assert out[0] == "3|Bitte 2 bis 40 Zeichen eingeben|PLZ: fuenf Ziffern|Bitte eine gueltige E-Mail-Adresse eingeben"
    assert out[1] == "0||", "alles gueltig: keine Fehler, die Beschriftung ist leer"


def test_eigene_meldung_von_aussen_und_zuruecknehmen(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'GUI_SET_ERROR(name, "Nummer schon vergeben")\n'
                'PRINT GUI_ERROR(name) ; "|" ; GUI_TEXT(lbl)\n'
                'GUI_CLEAR_ERRORS(w)\n'
                'PRINT "[" ; GUI_ERROR(name) ; "][" ; GUI_TEXT(lbl) ; "]"\n'
                'GUI_SET_TEXT(plz, "12345")\n'
                'PRINT "[" ; GUI_VALIDATE_WIDGET(plz) ; "][" ; GUI_VALIDATE_WIDGET(name) ; "]"\n'
                'GUI_RULES_CLEAR(name)\n'
                'PRINT "[" ; GUI_VALIDATE_WIDGET(name) ; "]"\n')
    assert out[0] == "Nummer schon vergeben|Nummer schon vergeben"
    assert out[1] == "[][]"
    assert out[2] == "[][Der Name fehlt.]", "GUI_VALIDATE_WIDGET prueft nur das eine Feld"
    assert out[3] == "[]", "ohne Regeln kein Fehler"


def test_zahl_bereich_und_datum(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'DIM menge AS GUI_WIDGET : menge = GUI_TEXTINPUT(w, 10, 210, 100, 28, "")\n'
                'GUI_RULE(menge, "ganz") : GUI_RULE(menge, "bereich", 1, 99)\n'
                'DIM datum AS GUI_WIDGET : datum = GUI_TEXTINPUT(w, 10, 250, 100, 28, "")\n'
                'GUI_RULE(datum, "datum")\n'
                'GUI_SET_TEXT(menge, "2,5") : GUI_SET_TEXT(datum, "2026-02-30")\n'
                'PRINT GUI_VALIDATE_WIDGET(menge) ; "|" ; GUI_VALIDATE_WIDGET(datum)\n'
                'GUI_SET_TEXT(menge, "100") : GUI_SET_TEXT(datum, "2026-02-28")\n'
                'PRINT GUI_VALIDATE_WIDGET(menge) ; "|" ; GUI_VALIDATE_WIDGET(datum)\n'
                'GUI_SET_TEXT(menge, "99")\n'
                'PRINT "[" ; GUI_VALIDATE_WIDGET(menge) ; "]"\n')
    assert out[0] == "Bitte eine ganze Zahl eingeben|Bitte ein Datum als JJJJ-MM-TT eingeben"
    assert out[1] == "Bitte einen Wert von 1 bis 99 eingeben|"
    assert out[2] == "[]"


def test_unsichtbare_felder_zaehlen_nicht(tmp_path):
    """Ein Feld auf einem anderen Reiter oder ausgeblendet kann der Nutzer
    nicht sehen -- ein Fehler dort waere ein Fehler ohne Ausweg."""
    out = _lauf(tmp_path, _KOPF +
                'GUI_SET_TEXT(name, "Anna") : GUI_DROPDOWN_SET_SELECTED(dd, 0)\n'
                'GUI_SET_VISIBLE(cb, FALSE)\n'
                'PRINT GUI_VALIDATE(w)\n'
                'GUI_SET_VISIBLE(cb, TRUE)\n'
                'PRINT GUI_VALIDATE(w)\n')
    assert out == ["0", "1"]


def test_regeln_pruefen_ihre_argumente(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'TRY\n    GUI_RULE(cb, "email")\nCATCH e\n    PRINT e\nEND TRY\n'
                'TRY\n    GUI_RULE(name, "bereich", 5)\nCATCH e2\n    PRINT e2\nEND TRY\n'
                'TRY\n    GUI_RULE(name, "muster", "[0-9")\nCATCH e3\n    PRINT e3\nEND TRY\n'
                'TRY\n    GUI_RULE(name, "farbe")\nCATCH e4\n    PRINT e4\nEND TRY\n'
                'TRY\n    GUI_ERROR_LABEL(name, plz)\nCATCH e5\n    PRINT e5\nEND TRY\n')
    assert "nur 'pflicht'" in out[0]
    assert "von und bis" in out[1]
    assert "ungueltiges Muster" in out[2]
    assert "keine Regel" in out[3]
    assert "Beschriftung" in out[4]


def test_regeln_ueberleben_die_datei(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'DIM j AS STRING : j = GUI_TO_JSON(w)\n'
                'DIM w2 AS GUI_WINDOW : w2 = GUI_FROM_JSON(j)\n'
                'PRINT GUI_TO_JSON(w2) = j ; " " ; INSTR(j, "rules") >= 0 ; " " ; INSTR(j, "error_label") >= 0\n'
                'PRINT GUI_VALIDATE(w2)\n')
    assert out[0] == "TRUE TRUE TRUE"
    assert out[1] == "2", "die geladene Form prueft genauso"
