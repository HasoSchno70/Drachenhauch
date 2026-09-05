"""Datenbindung im gui-Modul: GUI_BIND, GUI_FORM_GET/SET/CLEAR/CLEAN/CHANGED
und GUI_FORM_LOAD/SAVE gegen SQLite.

Der sechste Pilot fuellte drei Formulare von Hand aus der Datenbank und
schrieb sie von Hand zurueck -- je Feld ein GUI_SET_TEXT und ein
Parameter im UPDATE, dazu ein Text-Vergleich fuer "geaendert?". Jetzt
traegt jedes Widget einen Schluessel, und ein Aufruf laedt, speichert
oder vergleicht das ganze Formular.

Headless; was in der Datenbank steht, liest Pythons `sqlite3` -- ein
fremder Leser, nicht das eigene Modul.
"""
import os
import sqlite3
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

_KOPF = ('IMPORT "gui"\nIMPORT "db"\n'
         'SCREEN(500, 400, "T", 1)\n'
         'SET_WINDOW_POS(-3000, -3000)\n'
         'DIM w AS GUI_WINDOW\n'
         'w = GUI_WINDOW("T", 0, 0, 500, 400)\n'
         'GUI_WINDOW_CHROME(w, FALSE)\n'
         'DIM name AS GUI_WIDGET : name = GUI_TEXTINPUT(w, 10, 10, 200, 28, "")\n'
         'DIM plz AS GUI_WIDGET : plz = GUI_TEXTINPUT(w, 10, 50, 200, 28, "")\n'
         'GUI_TEXTINPUT_SET(plz, "zahlen", 1)\n'
         'DIM preis AS GUI_WIDGET : preis = GUI_TEXTINPUT(w, 10, 90, 200, 28, "")\n'
         'GUI_TEXTINPUT_SET(preis, "zahlen", 2)\n'
         'DIM aktiv AS GUI_WIDGET : aktiv = GUI_CHECKBOX(w, "aktiv", 10, 130)\n'
         'DIM art AS GUI_WIDGET : art = GUI_DROPDOWN(w, 10, 160, 200, 28, ["Privat", "Firma"])\n'
         'DIM andere AS GUI_WIDGET : andere = GUI_TEXTINPUT(w, 250, 10, 200, 28, "")\n'
         'GUI_BIND(name, "name", "kunde") : GUI_BIND(plz, "plz", "kunde") : GUI_BIND(preis, "preis", "kunde")\n'
         'GUI_BIND(aktiv, "aktiv", "kunde") : GUI_BIND(art, "art", "kunde")\n'
         'GUI_BIND(andere, "titel", "artikel")\n'
         'DIM db AS DB_CONN : db = DB_OPEN("b.db")\n'
         'DB_EXEC(db, "CREATE TABLE kunden (id INTEGER PRIMARY KEY, name TEXT, plz INTEGER, preis REAL, aktiv INTEGER, art INTEGER)")\n'
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


def test_get_und_changed_kennen_den_sauberen_stand(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'PRINT GUI_FORM_CHANGED(w, "kunde")\n'
                'GUI_SET_TEXT(name, "Anna") : GUI_SET_CHECKED(aktiv, TRUE) : GUI_DROPDOWN_SET_SELECTED(art, 1)\n'
                'PRINT GUI_FORM_CHANGED(w, "kunde") ; " " ; GUI_FORM_CHANGED(w, "artikel")\n'
                'DIM m AS MAP OF STRING : m = GUI_FORM_GET(w, "kunde")\n'
                'PRINT MAPGET(m, "name") ; "|" ; MAPGET(m, "plz") ; "|" ; MAPGET(m, "aktiv") ; "|" ; MAPGET(m, "art") ; "|" ; MAPHAS(m, "titel")\n'
                'GUI_FORM_CLEAN(w, "kunde")\n'
                'PRINT GUI_FORM_CHANGED(w, "kunde")\n'
                'GUI_FORM_CLEAR(w, "kunde")\n'
                'PRINT "[" ; GUI_TEXT(name) ; "] " ; GUI_CHECKED(aktiv) ; " " ; GUI_DROPDOWN_SELECTED(art) ; " " ; GUI_FORM_CHANGED(w, "kunde")\n')
    assert out[0] == "FALSE", "beim Binden ist der Stand sauber"
    assert out[1] == "TRUE FALSE", "das andere Formular bleibt unberuehrt"
    assert out[2] == "Anna||1|1|FALSE", "Texte; Kaestchen 1/0, Klappliste als Index; fremde Schluessel fehlen"
    assert out[3] == "FALSE"
    assert out[4] == "[] FALSE -1 FALSE", "leeren heisst leer UND sauber"


def test_save_schreibt_typisiert_und_load_holt_zurueck(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'GUI_SET_TEXT(name, "Anna") : GUI_SET_TEXT(plz, "80331") : GUI_SET_TEXT(preis, "19,99")\n'
                'GUI_SET_CHECKED(aktiv, TRUE) : GUI_DROPDOWN_SET_SELECTED(art, 1)\n'
                'DIM id AS INTEGER : id = GUI_FORM_SAVE(w, db, "kunden", -1, "kunde")\n'
                'PRINT id ; " " ; GUI_FORM_CHANGED(w, "kunde")\n'
                'GUI_FORM_CLEAR(w, "kunde")\n'
                'PRINT GUI_FORM_LOAD(w, db, "kunden", id, "kunde")\n'
                'PRINT GUI_TEXT(name) ; "|" ; GUI_TEXT(plz) ; "|" ; GUI_TEXT(preis) ; "|" ; GUI_CHECKED(aktiv) ; "|" ; GUI_DROPDOWN_SELECTED(art) ; "|" ; GUI_FORM_CHANGED(w, "kunde")\n'
                'GUI_SET_TEXT(name, "Anna Berger")\n'
                'PRINT GUI_FORM_SAVE(w, db, "kunden", id, "kunde")\n'
                'PRINT GUI_FORM_LOAD(w, db, "kunden", 999, "kunde")\n')
    assert out[0] == "1 FALSE", "INSERT liefert die neue id, danach ist der Stand sauber"
    assert out[1] == "TRUE"
    assert out[2] == "Anna|80331|19,99|TRUE|1|FALSE", "geladen und sauber; die Kommazahl deutsch"
    assert out[3] == "1", "UPDATE liefert dieselbe id"
    assert out[4] == "FALSE", "eine id, die es nicht gibt"
    con = sqlite3.connect(str(tmp_path / "b.db"))
    zeile = con.execute("SELECT name, plz, preis, aktiv, art, typeof(plz), typeof(preis), typeof(aktiv) FROM kunden").fetchall()
    assert zeile == [("Anna Berger", 80331, 19.99, 1, 1, "integer", "real", "integer")], \
        "Zahlenfeld -> Zahl, Kaestchen -> 0/1, Klappliste -> Index"


def test_set_aus_map_ist_tolerant(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'DIM m AS MAP OF STRING : m = {"name": "Kolb", "aktiv": "ja", "art": "firma", "plz": "01067"}\n'
                'GUI_FORM_SET(w, m, "kunde")\n'
                'PRINT GUI_TEXT(name) ; "|" ; GUI_CHECKED(aktiv) ; "|" ; GUI_DROPDOWN_SELECTED(art) ; "|" ; GUI_TEXT(plz) ; "|" ; GUI_FORM_CHANGED(w, "kunde")\n'
                'DIM z AS MAP OF INTEGER : z = {"aktiv": 0, "art": 0}\n'
                'GUI_FORM_SET(w, z, "kunde")\n'
                'PRINT GUI_CHECKED(aktiv) ; "|" ; GUI_DROPDOWN_SELECTED(art) ; "|" ; GUI_TEXT(name)\n')
    assert out[0] == "Kolb|TRUE|1|01067|FALSE", "ja -> Haken, Eintragstext -> Index, Text bleibt Text"
    assert out[1] == "FALSE|0|Kolb", "Zahlen aus einer MAP OF INTEGER; nicht genannte Felder bleiben"


def test_bindung_prueft_ihre_argumente(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'TRY\n    GUI_BIND(name, "na-me")\nCATCH e\n    PRINT e\nEND TRY\n'
                'DIM b AS GUI_WIDGET : b = GUI_BUTTON(w, "x", 0, 0, 40, 20)\n'
                'TRY\n    GUI_BIND(b, "x")\nCATCH e2\n    PRINT e2\nEND TRY\n'
                'TRY\n    GUI_FORM_LOAD(w, db, "kun den", 1, "kunde")\nCATCH e3\n    PRINT e3\nEND TRY\n'
                'TRY\n    GUI_FORM_SAVE(w, db, "kunden", -1, "nix")\nCATCH e4\n    PRINT e4\nEND TRY\n')
    assert "Spaltenname" in out[0]
    assert "keinen Wert" in out[1]
    assert "kein Tabellenname" in out[2]
    assert "kein Widget gebunden" in out[3]


def test_bindung_ueberlebt_die_datei(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'DIM j AS STRING : j = GUI_TO_JSON(w)\n'
                'DIM w2 AS GUI_WINDOW : w2 = GUI_FROM_JSON(j)\n'
                'PRINT GUI_TO_JSON(w2) = j ; " " ; INSTR(j, "bind") >= 0\n'
                'DIM m AS MAP OF STRING : m = GUI_FORM_GET(w2, "kunde")\n'
                'PRINT MAPHAS(m, "name") ; " " ; MAPHAS(m, "titel")\n')
    assert out == ["TRUE TRUE", "TRUE FALSE"]
