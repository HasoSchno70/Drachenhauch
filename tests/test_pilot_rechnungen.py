"""Der sechste Pilot: die Rechnungsverwaltung, bedient wie von Hand
(`examples/196_rechnungen.dh`).

Kein Editor, sondern eine Geschaeftsanwendung -- Kunden, Artikel, Rechnungen
mit Positionen, SQLite als Wahrheit. Geprueft wird deshalb an dem, was ein
FREMDER Leser sieht: Pythons `sqlite3` liest die Datenbank, PyMuPDF das
PDF, das `csv`-Modul den Export. Was die Oberflaeche zeigt, kommt ueber
eine PRINT-Zeile je Bild, die nur BESTEHENDE Werte ausliest; an der Logik
des Piloten wird nichts geaendert.

Klicks werden echt eingespeist. Die Lage der Knoepfe kommt aus einem ersten
Lauf (Layout-Behaelter verteilen sie erst zur Laufzeit), der Versatz vom
Fenster zum Bildschirm aus einer Zeichenflaeche bei (0, 0). Seriell, weil
Eingabe eingespeist wird.
"""
import csv
import os
import sqlite3
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent
_PILOT = _ROOT / "examples" / "196_rechnungen.dh"


def _find_dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    return next((_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
                 for v in ("release", "debug")
                 if (_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe).exists()),
                None)


_DHRT = _find_dhrt()
pytestmark = [pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut"),
              pytest.mark.seriell]

KEY_UP, KEY_DOWN = 1, 2
MOUSE_BUTTON_UP, MOUSE_BUTTON_DOWN, MOUSE_POSITION = 5, 6, 7
T_STRG, T_V, T_E, T_ENTER, T_ESC = 341, 86, 69, 257, 256
ROW0, ROW_H = 52, 20          # erste Tabellenzeile: Kopf (22) + Filterzeile (22) + halbe Zeile

_ZAHLEN = ("seite modal kundeId kSel kZeilen rechIdAkt posAnz rSel "
           "nullX nullY").split()
_LAGE = "bKNeu bKSpeichern bKLoeschen tfKName tfKPlz tKunden tRech bRPdf bPAdd bRSpeichern bRLoeschen ddPArtikel".split()
_TEXTE = "kName kFehler status brutto rFehler".split()

_PROBE = ('    PRINT "P " + STR$(GUI_ACTIVE_TAB(win)) + " " + STR$(IIF(GUI_MODAL(), 1, 0)) + " " + STR$(kundeId) + _\n'
          '          " " + STR$(GUI_TABLE_SELECTED(tKunden)) + " " + STR$(GUI_TABLE_ROW_COUNT(tKunden)) + _\n'
          '          " " + STR$(rechIdAkt) + " " + STR$(posAnz) + " " + STR$(GUI_TABLE_SELECTED(tRech)) + _\n'
          '          " " + STR$(GUI_CANVAS_X(nullpunkt)) + " " + STR$(GUI_CANVAS_Y(nullpunkt))'
          + "".join(' + _\n          " " + STR$(GUI_GET_X(%s)) + " " + STR$(GUI_GET_Y(%s)) + '
                    '" " + STR$(GUI_GET_W(%s)) + " " + STR$(GUI_GET_H(%s))' % (w, w, w, w) for w in _LAGE)
          + ' + _\n          "|" + GUI_TEXT(tfKName) + "|" + GUI_TEXT(lKFehler) + "|" + GUI_TEXT(lblStatus) + '
            '"|" + GUI_TEXT(vSBrutto) + "|" + GUI_TEXT(lRFehler)\n')


def _kopie(tmp_path, zusatz="", start=""):
    src = _PILOT.read_text(encoding="utf-8")
    assert src.count("SETFPS(60)") == 1
    src = src.replace("SETFPS(60)", "SETFPS(60)\nSET_WINDOW_POS(-3000, -3000)", 1)
    assert src.count("GUI_WINDOW_CHROME(win, FALSE)\n") == 1
    src = src.replace("GUI_WINDOW_CHROME(win, FALSE)\n",
                      "GUI_WINDOW_CHROME(win, FALSE)\n"
                      "DIM nullpunkt AS GUI_WIDGET : nullpunkt = GUI_CANVAS(win, 0, 0, 1, 1)\n"
                      "DIM tBild AS INTEGER : tBild = 0\n", 1)
    assert src.count("GUI_FOCUS(tfKName)\n") == 1
    src = src.replace("GUI_FOCUS(tfKName)\n", "GUI_FOCUS(tfKName)\n" + start, 1)
    assert src.count("    CLS(&H0E1014)\n") == 1
    src = src.replace("    CLS(&H0E1014)\n", "    tBild = tBild + 1\n" + zusatz + _PROBE + "    CLS(&H0E1014)\n", 1)
    ziel = tmp_path / "pilot.dh"
    ziel.write_text(src, encoding="utf-8")
    return ziel


def _events(tmp_path, events):
    events = sorted(events, key=lambda e: e[0])
    lines = ["# Test-Aufnahme", "c %d" % len(events)]
    for frame, typ, *params in events:
        p = (list(params) + [0, 0, 0, 0])[:4]
        lines.append("e %d %d %d %d %d %d // Event: test" % (frame, typ, p[0], p[1], p[2], p[3]))
    (tmp_path / "ev.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")


def _lauf(tmp_path, frames, events=None, zusatz="", start=""):
    quelle = _kopie(tmp_path, zusatz, start)
    if events is not None:
        _events(tmp_path, events)
        text = quelle.read_text(encoding="utf-8")
        text = text.replace("SETFPS(60)", 'SETFPS(60)\nAUTOMATION_PLAY("ev.txt")', 1)
        quelle.write_text(text, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(quelle)], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", timeout=240,
                       env=dict(os.environ, DHRT_FRAMES=str(frames)), cwd=str(tmp_path))
    assert r.returncode == 0, r.stderr
    out = []
    for ln in (r.stdout or "").splitlines():
        if not ln.startswith("P "):
            continue
        zahlen, _, texte = ln[2:].partition("|")
        werte = [int(v) for v in zahlen.split()]
        d = dict(zip(_ZAHLEN, werte[:len(_ZAHLEN)]))
        rest = werte[len(_ZAHLEN):]
        for k, w in enumerate(_LAGE):
            d[w] = tuple(rest[k * 4:k * 4 + 4])
        d.update(zip(_TEXTE, texte.split("|")))
        out.append(d)
    assert out, (r.stdout, r.stderr)
    return out


def _db(tmp_path):
    return sqlite3.connect(str(tmp_path / "rechnungen.db"))


def _mitte(lage, ox, oy):
    x, y, w, h = lage
    return ox + x + w // 2, oy + y + h // 2


def _klick(frame, x, y):
    return [(frame, MOUSE_POSITION, x, y), (frame + 1, MOUSE_BUTTON_DOWN, 0), (frame + 2, MOUSE_BUTTON_UP, 0)]


def _taste(frame, code, *mods):
    ev = [(frame, KEY_DOWN, m) for m in mods]
    ev += [(frame + 1, KEY_DOWN, code), (frame + 2, KEY_UP, code)]
    ev += [(frame + 3, KEY_UP, m) for m in mods]
    return ev


def _lage(tmp_path, start=""):
    """Erster Lauf: wo liegt was (nach dem Layout-Durchlauf)."""
    p = _lauf(tmp_path, 3, start=start)[-1]
    return p, p["nullX"], p["nullY"]


# ---------------------------------------------------------------- Datenbank
def test_erster_start_legt_datenbank_mit_beispieldaten_an(tmp_path):
    p = _lauf(tmp_path, 2)[-1]
    con = _db(tmp_path)
    assert con.execute("SELECT COUNT(*) FROM kunden").fetchone()[0] == 3
    assert con.execute("SELECT COUNT(*) FROM artikel").fetchone()[0] == 4
    assert con.execute("SELECT COUNT(*) FROM rechnungen").fetchone()[0] == 2
    assert con.execute("SELECT wert FROM einstellungen WHERE key = 'waehrung'").fetchone()[0] == "EUR"
    assert p["kZeilen"] == 3 and p["seite"] == 0 and p["kundeId"] == -1
    # Ein zweiter Start legt nichts doppelt an.
    _lauf(tmp_path, 2)
    assert con.execute("SELECT COUNT(*) FROM kunden").fetchone()[0] == 3


def test_geld_wird_in_cent_gerechnet(tmp_path):
    zusatz = ('    PRINT "G " + geld$(1999) + "|" + STR$(centVon("19,99")) + "|" + STR$(centVon("1.234,56")) + '
              '"|" + STR$(centVon("12.5")) + "|" + STR$(centVon("abc")) + "|" + geld$(123456789) + "|" + geld$(-5)\n')
    quelle = _kopie(tmp_path, zusatz)
    r = subprocess.run([str(_DHRT), "run", str(quelle)], capture_output=True, text=True, encoding="utf-8",
                       errors="replace", timeout=120, env=dict(os.environ, DHRT_FRAMES="4"), cwd=str(tmp_path))
    assert r.returncode == 0, (r.stdout, r.stderr)
    zeilen = [ln for ln in r.stdout.splitlines() if ln.startswith("G ")]
    assert zeilen, (r.stdout, r.stderr)
    g = zeilen[-1][2:].split("|")
    assert g == ["19,99 EUR", "1999", "123456", "1250", "-1", "1.234.567,89 EUR", "-0,05 EUR"]


# ---------------------------------------------------------------- Kunden
def test_kunde_anlegen_landet_in_der_datenbank(tmp_path):
    lage, ox, oy = _lage(tmp_path)
    nx, ny = _mitte(lage["tfKName"], ox, oy)
    sx, sy = _mitte(lage["bKSpeichern"], ox, oy)
    ev = _klick(3, nx, ny) + _taste(7, T_V, T_STRG) + _klick(12, sx, sy)
    out = _lauf(tmp_path, 20, ev, start='CLIPBOARD_SET("Testkunde GmbH")\n')
    con = _db(tmp_path)
    assert con.execute("SELECT name FROM kunden WHERE name = 'Testkunde GmbH'").fetchone()
    assert out[-1]["kZeilen"] == 4 and out[-1]["kundeId"] > 0
    assert "gespeichert" in out[-1]["status"]


def test_pflichtfeld_und_plz_werden_geprueft(tmp_path):
    lage, ox, oy = _lage(tmp_path)
    sx, sy = _mitte(lage["bKSpeichern"], ox, oy)
    out = _lauf(tmp_path, 10, _klick(3, sx, sy))
    assert out[-1]["kFehler"] == "Der Name fehlt."
    assert _db(tmp_path).execute("SELECT COUNT(*) FROM kunden").fetchone()[0] == 3
    # Name da, aber eine dreistellige PLZ.
    nx, ny = _mitte(lage["tfKName"], ox, oy)
    px, py = _mitte(lage["tfKPlz"], ox, oy)
    ev = _klick(3, nx, ny) + _taste(7, T_V, T_STRG) + _klick(12, px, py) + _taste(16, T_V, T_STRG) + _klick(21, sx, sy)
    zusatz = '    IF tBild = 10 THEN CLIPBOARD_SET("123")\n'
    out = _lauf(tmp_path, 28, ev, zusatz=zusatz, start='CLIPBOARD_SET("Kurz")\n')
    assert "fuenf Ziffern" in out[-1]["kFehler"], out[-1]
    assert _db(tmp_path).execute("SELECT COUNT(*) FROM kunden").fetchone()[0] == 3


def test_wechsel_mit_aenderung_fragt_nach_und_esc_bricht_ab(tmp_path):
    lage, ox, oy = _lage(tmp_path)
    nx, ny = _mitte(lage["tfKName"], ox, oy)
    tx, ty, tw, th = lage["tKunden"]
    zeile0 = (ox + tx + 60, oy + ty + ROW0)
    ev = _klick(3, nx, ny) + _taste(7, T_V, T_STRG) + _klick(12, *zeile0) + _taste(18, T_ESC)
    out = _lauf(tmp_path, 26, ev, start='CLIPBOARD_SET("Geaendert")\n')
    assert any(p["modal"] == 1 for p in out), "die Rueckfrage stand offen"
    letzte = out[-1]
    assert letzte["modal"] == 0
    assert letzte["kName"] == "Geaendert", "Abbrechen laesst die Eingabe stehen"
    assert letzte["kundeId"] == -1 and letzte["kSel"] == -1, "und bleibt beim neuen Kunden"
    # Gegenprobe: ohne Aenderung wechselt der Klick sofort.
    out2 = _lauf(tmp_path, 12, _klick(3, *zeile0))
    assert out2[-1]["kundeId"] > 0 and out2[-1]["kSel"] == 0 and out2[-1]["modal"] == 0


def test_fenster_groesser_laesst_formular_und_tabelle_mitgehen(tmp_path):
    zusatz = "    IF tBild = 4 THEN GUI_WINDOW_SET_BOUNDS(win, 0, 0, 1400, 900)\n"
    out = _lauf(tmp_path, 8, zusatz=zusatz)
    vorher, nachher = out[1], out[-1]
    assert nachher["tKunden"][2] == vorher["tKunden"][2] + 280, "Tabelle waechst mit (lrtb)"
    assert nachher["tKunden"][3] == vorher["tKunden"][3] + 220
    assert nachher["bKSpeichern"][0] == vorher["bKSpeichern"][0] + 280, "der Behaelter klebt rechts, der Knopf darin folgt"
    assert nachher["bKNeu"][0] == vorher["bKNeu"][0] + 280


# ---------------------------------------------------------------- Rechnungen
_TAB2 = "GUI_SET_ACTIVE_TAB(win, 2)\n"


def test_pdf_zeigt_was_in_der_datenbank_steht(tmp_path):
    fitz = pytest.importorskip("fitz", reason="PyMuPDF nicht installiert")
    lage, ox, oy = _lage(tmp_path, start=_TAB2)
    tx, ty, tw, th = lage["tRech"]
    # Zeile 0 der Liste ist 2026-0002 (absteigend sortiert) -- Anna Berger, 2 x Fachbuch.
    ev = _klick(3, ox + tx + 40, oy + ty + ROW0) + _klick(8, *_mitte(lage["bRPdf"], ox, oy))
    out = _lauf(tmp_path, 16, ev, start=_TAB2)
    assert out[-1]["brutto"] == "85,39 EUR", out[-1]
    pdf = tmp_path / "rechnung_2026-0002.pdf"
    assert pdf.exists(), out[-1]["status"]
    text = "".join(page.get_text() for page in fitz.open(str(pdf)))
    for erwartet in ("Rechnung 2026-0002", "Anna Berger", "Fachbuch Drachenhauch", "79,80 EUR", "5,59 EUR", "85,39 EUR",
                     "Drachenhauch Software", "Zahlbar innerhalb von 14 Tagen"):
        assert erwartet in text, (erwartet, text)


def test_position_hinzufuegen_und_speichern(tmp_path):
    lage, ox, oy = _lage(tmp_path, start=_TAB2)
    tx, ty, tw, th = lage["tRech"]
    ev = (_klick(3, ox + tx + 40, oy + ty + ROW0) + _klick(8, *_mitte(lage["bPAdd"], ox, oy))
          + _klick(13, *_mitte(lage["bRSpeichern"], ox, oy)))
    out = _lauf(tmp_path, 20, ev, start=_TAB2)
    # Die Artikel-Auswahl steht auf dem ersten Eintrag: "Anfahrt (pauschal)", 19,99 zu 19 %.
    assert out[-1]["posAnz"] == 2 and out[-1]["brutto"] == "109,18 EUR", out[-1]
    con = _db(tmp_path)
    rows = con.execute("SELECT bezeichnung, menge, preis_cent, mwst FROM positionen WHERE rechnung_id = 2 ORDER BY id").fetchall()
    assert rows == [("Fachbuch Drachenhauch", 2, 3990, 7), ("Anfahrt (pauschal)", 1, 1999, 19)]
    assert "gespeichert" in out[-1]["status"]


def test_neue_rechnung_ohne_position_wird_abgelehnt(tmp_path):
    lage, ox, oy = _lage(tmp_path, start=_TAB2)
    out = _lauf(tmp_path, 10, _klick(3, *_mitte(lage["bRSpeichern"], ox, oy)), start=_TAB2)
    assert out[-1]["rFehler"] == "Eine Rechnung ohne Position ist keine."
    assert _db(tmp_path).execute("SELECT COUNT(*) FROM rechnungen").fetchone()[0] == 2


def test_rechnung_loeschen_fragt_und_enter_bestaetigt(tmp_path):
    lage, ox, oy = _lage(tmp_path, start=_TAB2)
    tx, ty, tw, th = lage["tRech"]
    ev = (_klick(3, ox + tx + 40, oy + ty + ROW0) + _klick(8, *_mitte(lage["bRLoeschen"], ox, oy))
          + _taste(14, T_ENTER))
    out = _lauf(tmp_path, 22, ev, start=_TAB2)
    assert any(p["modal"] == 1 for p in out)
    con = _db(tmp_path)
    assert con.execute("SELECT nummer FROM rechnungen ORDER BY nummer").fetchall() == [("2026-0001",)]
    assert con.execute("SELECT COUNT(*) FROM positionen WHERE rechnung_id = 2").fetchone()[0] == 0
    assert out[-1]["rechIdAkt"] == -1 and "geloescht" in out[-1]["status"]


def test_csv_export_per_kuerzel(tmp_path):
    out = _lauf(tmp_path, 12, _taste(3, T_E, T_STRG))
    datei = tmp_path / "rechnungen.csv"
    assert datei.exists(), out[-1]["status"]
    with datei.open(encoding="utf-8", newline="") as f:
        zeilen = list(csv.reader(f, delimiter=";"))
    assert zeilen[0] == ["Nummer", "Datum", "Kunde", "Netto", "MwSt", "Brutto", "Status"]
    assert zeilen[1] == ["2026-0001", "2026-08-14", "Muellerbau GmbH", "304,99", "57,95", "362,94", "bezahlt"]
    assert zeilen[2][0] == "2026-0002" and zeilen[2][5] == "85,39" and zeilen[2][6] == "offen"
