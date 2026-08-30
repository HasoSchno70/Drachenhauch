"""`GUI_TEXTAREA_VIEW` -- welcher Ausschnitt eines Code-Feldes ist zu sehen?

Damit färbt ein Programm nur ein, was man sieht, statt bei jedem Tastendruck
die ganze Datei. Gemessen an einer 30.000-Zeilen-Datei: **272 ms → 2,1 ms**
je Anschlag.

Dass der Ausschnitt nicht nur schneller, sondern auch **richtig** ist, liegt
an der Sprache: Kommentare und Zeichenketten enden in Drachenhauch an der
Zeile. Ein an Zeilengrenzen geschnittener Ausschnitt kann darum nicht mitten
in einem Gebilde anfangen -- bei einer Sprache mit Blockkommentaren wäre
genau das die Falle.

Braucht ein Fenster (die Zeilenhöhe hängt an der Schrift), steht darum in
`conftest._BRAUCHT_GRAFIK`.
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
         'SCREEN(600, 400, "T", 1)\n'
         'DIM w AS GUI_WINDOW\n'
         'w = GUI_WINDOW("T", 5, 5, 580, 380)\n')


def _lauf(tmp_path, src, frames=2):
    """`assert returncode == 0, r.stderr` ist Pflicht: ohne Bildschirm bricht
    raylib beim Fenster ab, und nur wenn seine Meldung IM FEHLERTEXT steht,
    macht conftest daraus einen Skip statt eines Fehlschlags."""
    (tmp_path / "a.dh").write_text(src, encoding="utf-8")
    env = dict(os.environ, DHRT_FRAMES=str(frames))
    r = subprocess.run([str(_DHRT), "run", str(tmp_path / "a.dh")], capture_output=True,
                       text=True, encoding="utf-8", env=env, timeout=60, cwd=str(tmp_path))
    assert r.returncode == 0, r.stderr
    return [ln for ln in (r.stdout or "").splitlines()
            if not ln.startswith(("WARNING:", "INFO:", "TRACE:"))]


def test_liefert_vier_werte(tmp_path):
    zeilen = _lauf(tmp_path, _KOPF + '''
DIM ta AS GUI_WIDGET
ta = GUI_TEXTAREA(w, 10, 10, 500, 200)
GUI_SET_TEXT(ta, "eins" + CHR$(10) + "zwei")
DIM z0 AS INTEGER
DIM anz AS INTEGER
DIM von AS INTEGER
DIM laenge AS INTEGER
(z0, anz, von, laenge) = GUI_TEXTAREA_VIEW(ta)
PRINT z0; " "; von; " "; laenge; " "; IIF(anz > 3, "genug", "zu wenig")
''')
    z0, von, laenge, genug = zeilen[-1].split()
    assert z0 == "0" and von == "0"
    assert laenge == "9", f"'eins\\nzwei' sind 9 Zeichen, gemeldet {laenge}"
    assert genug == "genug"


def test_ausschnitt_endet_am_sichtbaren_bereich(tmp_path):
    """Der Kern: bei mehr Zeilen als Platz darf `laenge` NICHT den ganzen
    Text umfassen -- sonst faerbt das Programm doch wieder alles."""
    zeilen = _lauf(tmp_path, _KOPF + '''
DIM ta AS GUI_WIDGET
ta = GUI_TEXTAREA(w, 10, 10, 500, 80)
DIM q AS STRING
DIM i AS INTEGER
FOR i = 1 TO 200
    q = q + "zeile " + STR$(i) + CHR$(10)
NEXT
GUI_SET_TEXT(ta, q)
DIM z0 AS INTEGER
DIM anz AS INTEGER
DIM von AS INTEGER
DIM laenge AS INTEGER
(z0, anz, von, laenge) = GUI_TEXTAREA_VIEW(ta)
PRINT anz; " "; laenge; " "; LEN(q)
''')
    anz, laenge, gesamt = (int(x) for x in zeilen[-1].split())
    assert 1 <= anz <= 10, f"in 80 px passen keine {anz} Zeilen"
    assert laenge < gesamt / 10, f"Ausschnitt {laenge} von {gesamt} -- zu viel"


def test_ausschnitt_passt_zur_zeilenzahl(tmp_path):
    """`laenge` muss genau die gemeldeten Zeilen abdecken -- eine zu kurze
    Angabe liesse die letzte sichtbare Zeile farblos."""
    zeilen = _lauf(tmp_path, _KOPF + '''
DIM ta AS GUI_WIDGET
ta = GUI_TEXTAREA(w, 10, 10, 500, 120)
DIM q AS STRING
DIM i AS INTEGER
FOR i = 1 TO 100
    q = q + "abcde" + CHR$(10)
NEXT
GUI_SET_TEXT(ta, q)
DIM z0 AS INTEGER
DIM anz AS INTEGER
DIM von AS INTEGER
DIM laenge AS INTEGER
(z0, anz, von, laenge) = GUI_TEXTAREA_VIEW(ta)
' Jede Zeile ist "abcde" + Umbruch = 6 Zeichen, die letzte ohne Umbruch.
PRINT laenge; " "; anz * 6 - 1
''')
    laenge, erwartet = (int(x) for x in zeilen[-1].split())
    assert laenge == erwartet, f"{laenge} Zeichen fuer die sichtbaren Zeilen, erwartet {erwartet}"


def test_leerer_text(tmp_path):
    zeilen = _lauf(tmp_path, _KOPF + '''
DIM ta AS GUI_WIDGET
ta = GUI_TEXTAREA(w, 10, 10, 500, 100)
DIM z0 AS INTEGER
DIM anz AS INTEGER
DIM von AS INTEGER
DIM laenge AS INTEGER
(z0, anz, von, laenge) = GUI_TEXTAREA_VIEW(ta)
PRINT z0; " "; von; " "; laenge
''')
    assert zeilen[-1].split() == ["0", "0", "0"]


def test_nur_auf_einem_textbereich(tmp_path):
    zeilen = _lauf(tmp_path, _KOPF + '''
DIM b AS GUI_WIDGET
b = GUI_BUTTON(w, "ok", 10, 10, 60, 24)
TRY
    DIM z0 AS INTEGER
    DIM anz AS INTEGER
    DIM von AS INTEGER
    DIM laenge AS INTEGER
    (z0, anz, von, laenge) = GUI_TEXTAREA_VIEW(b)
    PRINT "kein Fehler"
CATCH e
    PRINT "abgelehnt"
END TRY
''')
    assert zeilen[-1].strip() == "abgelehnt"


def test_der_ganze_weg_nur_sichtbar_einfaerben(tmp_path):
    """So sieht es im Editor aus: Ausschnitt holen, nur ihn zerlegen, die
    Abschnitte auf den ganzen Text umrechnen, einfaerben."""
    zeilen = _lauf(tmp_path, _KOPF + '''
DIM ta AS GUI_WIDGET
ta = GUI_TEXTAREA(w, 10, 10, 500, 100)
DIM q AS STRING
DIM i AS INTEGER
FOR i = 1 TO 300
    q = q + "DIM x" + STR$(i) + " AS INTEGER" + CHR$(10)
NEXT
GUI_SET_TEXT(ta, q)

DIM z0 AS INTEGER
DIM anz AS INTEGER
DIM von AS INTEGER
DIM laenge AS INTEGER
(z0, anz, von, laenge) = GUI_TEXTAREA_VIEW(ta)
DIM teil AS STRING
teil = MID$(q, von, laenge)
DIM st AS ARRAY OF INTEGER
DIM ln AS ARRAY OF INTEGER
DIM ar AS ARRAY OF STRING
(st, ln, ar) = SYNTAX_SPANS(teil)
DIM n AS INTEGER
n = LEN(st)
DIM fb[n] AS INTEGER
FOR i = 0 TO n - 1
    st[i] = st[i] + von
    fb[i] = IIF(ar[i] = "schluessel", &H2BC4E8, &HFFFFFF)
NEXT
GUI_TEXTAREA_SPANS(ta, st, ln, fb)
' Vier Abschnitte je Zeile (DIM, xN, AS, INTEGER).
PRINT n; " "; anz * 4
''')
    n, erwartet = (int(x) for x in zeilen[-1].split())
    assert n == erwartet, f"{n} Abschnitte fuer {erwartet} erwartete"
