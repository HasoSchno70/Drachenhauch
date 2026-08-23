"""Compiler-Warnungen, die Fehler melden, bevor sie wehtun.

Beide Warnungen hier stammen aus echten Stolpersteinen beim Bauen der Demo:

* **Argumentzahl** -- die Grafik-Builtins greifen ihre Argumente per Index ab
  und ignorieren ueberzaehlige STILL. Ein mitgegebenes Argument tat damit
  einfach nichts, ohne ein Wort. (Kostete in Szene 2 der Demo einen halben
  Nachmittag: die Stueckzahl von PLOTS wurde verschluckt.)
* **Verdeckte Konstante** -- Drachenhauch ignoriert Gross-/Kleinschreibung, eine
  lokale `hoehe` verdeckt also die Konstante `HOEHE`. Der Fehler taucht dann
  weit weg von der Ursache auf.

Beides sind WARNUNGEN, keine Fehler: das Programm laeuft weiter. Geprueft wird
ueber `dhrt --check`, das die Warnungen als JSON ausgibt.
"""
import json
import os
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


def _find_dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    return next((_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
                 for v in ("release", "debug")
                 if (_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe).exists()), None)


_DHRT = _find_dhrt()
pytestmark = pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")


def _warnungen(tmp_path, quelle: str):
    """`dhrt --check` laufen lassen und die Warnungstexte liefern."""
    f = tmp_path / "w.dh"
    f.write_text(quelle, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "--check", str(f)], capture_output=True,
                       text=True, encoding="utf-8", timeout=60)
    return [w.get("message", "") for w in json.loads(r.stdout or "[]")]


# --------------------------------------------------------- Argumentzahl
def test_zu_viele_argumente_werden_gemeldet(tmp_path):
    w = _warnungen(tmp_path, """
SCREEN(64, 64, "a", 1)
DIM xs[3] AS INTEGER
DIM ys[3] AS INTEGER
PLOTS(xs, ys, &HFF0000, 2, 9)
""")
    assert any("PLOTS" in m and "5 Argumente" in m for m in w), w


def test_zu_wenige_argumente_werden_gemeldet(tmp_path):
    w = _warnungen(tmp_path, 'PRINT MID$("abc")\n')
    assert any("MID$" in m and "Argument" in m for m in w), w


def test_richtige_aufrufe_schweigen(tmp_path):
    # Pflicht-, Optional- und Vorgabewert-Parameter in allen Kombinationen.
    w = _warnungen(tmp_path, """
SCREEN(64, 64, "a", 1)
DIM xs[3] AS INTEGER
DIM ys[3] AS INTEGER
PLOTS(xs, ys, &HFF0000)
PLOTS(xs, ys, &HFF0000, 2)
LINE(0, 0, 10, 10)
LINE(0, 0, 10, 10, &HFFFFFF)
PRINT MID$("abc", 1)
PRINT MID$("abc", 1, 2)
""")
    assert not any("Argument" in m for m in w), w


def test_variadische_builtins_werden_nicht_gemeldet(tmp_path):
    # PATHJOIN nimmt beliebig viele Teile ("..."-Signatur) -- da darf die
    # Pruefung keine Obergrenze erfinden.
    w = _warnungen(tmp_path, 'PRINT PATHJOIN("a", "b")\nPRINT PATHJOIN("a", "b", "c", "d")\n')
    assert not any("Argument" in m for m in w), w


# ---------------------------------------------------- verdeckte Konstante
def test_lokale_variable_verdeckt_konstante(tmp_path):
    w = _warnungen(tmp_path, """
CONST HOEHE AS INTEGER = 720

SUB stolpert()
    DIM hoehe AS FLOAT
    hoehe = 0.6
END SUB

stolpert()
""")
    assert any("verdeckt" in m and "hoehe" in m for m in w), w


def test_anderer_name_wird_nicht_gemeldet(tmp_path):
    w = _warnungen(tmp_path, """
CONST HOEHE AS INTEGER = 720

SUB sauber()
    DIM hoch AS FLOAT
    hoch = 0.6
END SUB

sauber()
""")
    assert not any("verdeckt" in m for m in w), w


def test_gleicher_name_im_hauptprogramm_ist_kein_verdecken(tmp_path):
    # Auf oberster Ebene waere es eine Namens-Kollision (eigener Fehler),
    # kein Verdecken -- die Warnung darf hier nicht zusaetzlich feuern.
    w = _warnungen(tmp_path, """
CONST GRENZE AS INTEGER = 10

SUB nutzt()
    PRINT GRENZE
END SUB

nutzt()
""")
    assert not any("verdeckt" in m for m in w), w


# ------------------------------------------- Kommazahl an ganzzahlige Variable
#
# Die Laufzeit-Regel ist WERTbasiert ("passt verlustfrei?"): `n = f * 2.0`
# laeuft bei f = 1.5 durch und bricht bei f = 1.6 ab. Statisch entscheidbar ist
# das nicht -- deshalb WARNT der Compiler nur, statt abzuweisen. Anlass: im
# Einsteigerbuch ist genau dieser Abbruch fuenfmal angefallen, und `--check`
# hat jedes Mal geschwiegen.

def test_division_an_ganzzahl_warnt(tmp_path):
    w = _warnungen(tmp_path, "DIM n AS INTEGER" + chr(10) + "n = 7 / 2" + chr(10))
    assert any("als INTEGER angesagt" in m for m in w), w
    # Der Hinweis nennt bei einer Division den ganzzahligen Operator.
    assert any(chr(92) in m for m in w), w


def test_komma_literal_an_ganzzahl_warnt(tmp_path):
    w = _warnungen(tmp_path, "DIM n AS INTEGER" + chr(10) + "n = 3.5" + chr(10))
    assert any("als INTEGER angesagt" in m and "INT()" in m for m in w), w


def test_float_variable_an_ganzzahl_warnt(tmp_path):
    w = _warnungen(tmp_path, "DIM f AS FLOAT" + chr(10) + "f = 1.5" + chr(10)
                   + "DIM n AS INTEGER" + chr(10) + "n = f" + chr(10))
    assert any("als INTEGER angesagt" in m for m in w), w


def test_ganzzahlige_division_warnt_nicht(tmp_path):
    """Der empfohlene Weg darf nicht selbst angemeckert werden."""
    w = _warnungen(tmp_path, "DIM n AS INTEGER" + chr(10) + "n = 7 " + chr(92) + " 2" + chr(10))
    assert not any("als INTEGER angesagt" in m for m in w), w


def test_ganzzahl_rechnung_warnt_nicht(tmp_path):
    w = _warnungen(tmp_path, "DIM i AS INTEGER" + chr(10) + "i = 2" + chr(10)
                   + "DIM n AS INTEGER" + chr(10) + "n = i * 2 + 3" + chr(10))
    assert not any("als INTEGER angesagt" in m for m in w), w


def test_kommazahl_an_kommazahl_warnt_nicht(tmp_path):
    w = _warnungen(tmp_path, "DIM f AS FLOAT" + chr(10) + "f = 7 / 2" + chr(10))
    assert not any("als INTEGER angesagt" in m for m in w), w


def test_warnung_blockiert_nicht(tmp_path):
    """Es bleibt eine Warnung: das Programm laeuft, solange der Wert passt."""
    f = tmp_path / "lauf.dh"
    f.write_text("DIM g AS FLOAT" + chr(10) + "g = 1.5" + chr(10)
                 + "DIM n AS INTEGER" + chr(10) + "n = g * 2.0" + chr(10)
                 + "PRINT n" + chr(10), encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(f)], capture_output=True,
                       text=True, encoding="utf-8", timeout=60)
    assert r.stdout.strip() == "3", (r.stdout, r.stderr)


# ------------------------------------------- Typ, den das Ziel nie annimmt
#
# Anders als die Kommazahl-Warnung ist das NICHT wertabhaengig: `s = 5` bei
# `DIM s AS STRING` bricht ab, sobald die Zeile laeuft -- immer. Gemeldet wird
# trotzdem als Warnung, weil das Risiko nicht in der Regel liegt, sondern in
# der Herleitung des Typs; die ist neu.

def test_zahl_an_text_warnt(tmp_path):
    w = _warnungen(tmp_path, "DIM s AS STRING\ns = 5\n")
    assert any("als STRING angesagt" in m and "INTEGER" in m for m in w), w
    assert any("STR$()" in m for m in w), w


def test_text_an_zahl_warnt(tmp_path):
    w = _warnungen(tmp_path, 'DIM i AS INTEGER\ni = "text"\n')
    assert any("als INTEGER angesagt" in m and "VAL()" in m for m in w), w


def test_boolean_an_zahl_warnt(tmp_path):
    """BOOLEAN ist in Drachenhauch keine Zahl -- die Laufzeit lehnt es ab."""
    w = _warnungen(tmp_path, "DIM b AS BOOLEAN\nb = TRUE\nDIM i AS INTEGER\ni = b\n")
    assert any("erhalten BOOLEAN" in m for m in w), w


def test_vergleich_an_text_warnt(tmp_path):
    """Ein Vergleich liefert BOOLEAN -- auch ohne Variable dazwischen."""
    w = _warnungen(tmp_path, "DIM i AS INTEGER\nDIM s AS STRING\ns = i > 3\n")
    assert any("als STRING angesagt" in m and "BOOLEAN" in m for m in w), w


def test_richtige_zuweisungen_schweigen(tmp_path):
    """Alles hier ist gueltig -- inklusive der beiden STRING-Operatoren."""
    w = _warnungen(tmp_path, """
DIM s AS STRING
DIM i AS INTEGER
DIM f AS FLOAT
DIM b AS BOOLEAN
i = 3
f = i * 2
s = "a" + "b"
s = "-" * 40
b = i > 3
b = NOT b
""")
    assert not any("bricht beim Laufen ab" in m for m in w), w


def test_parametertyp_wird_erkannt(tmp_path):
    """Der Typ eines Parameters steht in der Signatur -- ohne DIM davor."""
    w = _warnungen(tmp_path, """
FUNCTION f(a AS INTEGER) AS INTEGER
    DIM s AS STRING
    s = a
    RETURN a
END FUNCTION
PRINT f(1)
""")
    assert any("als STRING angesagt" in m for m in w), w


def test_globaler_typ_gilt_auch_in_der_funktion(tmp_path):
    """Funktionen werden VOR dem Hauptprogramm uebersetzt -- die Typen der
    Globals muessen trotzdem schon feststehen."""
    w = _warnungen(tmp_path, """
DIM titel AS STRING

SUB setze()
    titel = 42
END SUB

setze()
""")
    assert any("'titel'" in m and "als STRING angesagt" in m for m in w), w


def test_feld_ueber_objekt_warnt(tmp_path):
    w = _warnungen(tmp_path, """
CLASS P
    DIM name AS STRING
END CLASS
DIM p AS P
p = NEW P()
p.name = 5
""")
    assert any("'p.name'" in m for m in w), w


def test_eigenes_feld_in_der_methode_warnt(tmp_path):
    w = _warnungen(tmp_path, """
CLASS P
    DIM name AS STRING
    SUB Init()
        Self.name = 5
    END SUB
END CLASS
DIM p AS P
p = NEW P()
""")
    assert any("'Self.name'" in m for m in w), w


def test_property_bleibt_still(tmp_path):
    """Ein Setter darf einen anderen Typ nehmen als das Feld dahinter."""
    w = _warnungen(tmp_path, """
CLASS P
    DIM _hp AS STRING
    PROPERTY GET hp() AS INTEGER
        RETURN VAL(Self._hp)
    END PROPERTY
    PROPERTY SET hp(v AS INTEGER)
        Self._hp = STR$(v)
    END PROPERTY
END CLASS
DIM p AS P
p = NEW P()
p.hp = 5
""")
    assert not any("bricht beim Laufen ab" in m for m in w), w


def test_referenztypen_bleiben_still(tmp_path):
    """Klassen/MAP reicht die Laufzeit durch -- eine Meldung 'bricht ab' waere
    schlicht unwahr, egal wie sinnvoll die Zuweisung ist."""
    w = _warnungen(tmp_path, """
CLASS A
    DIM x AS INTEGER
END CLASS
CLASS B
    DIM y AS INTEGER
END CLASS
DIM a AS A
a = NEW B()
""")
    assert not any("bricht beim Laufen ab" in m for m in w), w


def test_unbekannter_typ_bleibt_still(tmp_path):
    """Rueckgabetypen von Builtins werden bewusst nicht hergeleitet -- lieber
    ein Fund weniger als ein falscher Alarm."""
    w = _warnungen(tmp_path, 'DIM i AS INTEGER\ni = LEN("abc")\n')
    assert not any("bricht beim Laufen ab" in m for m in w), w


def test_meldung_blockiert_die_uebersetzung_nicht(tmp_path):
    """Es bleibt eine Warnung: alles vor der schlechten Zeile laeuft."""
    f = tmp_path / "lauf.dh"
    f.write_text('PRINT "davor"\nDIM s AS STRING\ns = 5\n', encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(f)], capture_output=True,
                       text=True, encoding="utf-8", timeout=60)
    assert "davor" in r.stdout, (r.stdout, r.stderr)
    assert "Erwartet STRING" in (r.stdout + r.stderr), (r.stdout, r.stderr)


# ------------------------------------------------- Argumenttyp am Aufruf
#
# Dieselbe Regel wie bei einer Zuweisung: die Laufzeit wandelt jedes Argument
# auf den angesagten Parametertyp um ("Parameter: Erwartet INTEGER, erhalten
# STRING"). Der Unterschied ist der Ort -- bisher fiel es erst auf, wenn der
# Aufruf auch wirklich ausgefuehrt wurde.

def test_falscher_argumenttyp_warnt(tmp_path):
    w = _warnungen(tmp_path, """
FUNCTION f(a AS INTEGER) AS INTEGER
    RETURN a
END FUNCTION
DIM s AS STRING
s = "x"
PRINT f(s)
""")
    assert any("Parameter 'a'" in m and "erhalten STRING" in m for m in w), w


def test_falscher_typ_bei_named_arg_warnt(tmp_path):
    """Named-Args werden ueber den Namen zugeordnet, nicht ueber die Stelle."""
    w = _warnungen(tmp_path, """
SUB zeichne(text AS STRING, x AS INTEGER)
    PRINT text
END SUB
zeichne(x: 1, text: 5)
""")
    assert any("Parameter 'text'" in m for m in w), w


def test_methode_wird_geprueft(tmp_path):
    w = _warnungen(tmp_path, """
CLASS P
    SUB setze(n AS STRING)
        PRINT n
    END SUB
END CLASS
DIM p AS P
p = NEW P()
p.setze(42)
""")
    assert any("P.setze" in m and "Parameter 'n'" in m for m in w), w


def test_implizite_methode_wird_geprueft(tmp_path):
    """`b(42)` ohne `Self.` innerhalb derselben Klasse."""
    w = _warnungen(tmp_path, """
CLASS P
    SUB a()
        b(42)
    END SUB
    SUB b(n AS STRING)
        PRINT n
    END SUB
END CLASS
DIM p AS P
p = NEW P()
""")
    assert any("P.b" in m and "Parameter 'n'" in m for m in w), w


def test_richtige_aufrufe_bleiben_still(tmp_path):
    """INTEGER an einen FLOAT-Parameter ist gueltig (und haeufig)."""
    w = _warnungen(tmp_path, """
FUNCTION f(a AS INTEGER, b AS FLOAT) AS FLOAT
    RETURN a + b
END FUNCTION
DIM i AS INTEGER
i = 2
PRINT f(i, 1.5)
PRINT f(3, 4)
""")
    assert not any("Parameter" in m for m in w), w


def test_variadischer_rest_wird_nicht_geprueft(tmp_path):
    """Der Sammel-Parameter bekommt ein TUPLE, nicht den Typ der Einzelwerte."""
    w = _warnungen(tmp_path, """
SUB log(stufe AS STRING, ...rest)
    PRINT stufe
END SUB
log("INFO", 5, TRUE, "x")
""")
    assert not any("Parameter" in m for m in w), w


# ------------------------------------- Mitglied, das es bei der Klasse nicht gibt
#
# Der Tippfehler im Feldnamen. Die Kunst liegt hier nicht im Finden, sondern im
# SCHWEIGEN: eine als Basisklasse angesagte Variable darf zur Laufzeit eine
# abgeleitete halten, und dann ist der Zugriff auf deren eigenes Mitglied
# richtig. Eine Pruefung, die das anstreicht, wuerde genau den Code melden,
# fuer den es Vererbung gibt.

def test_tippfehler_im_feldnamen_warnt(tmp_path):
    w = _warnungen(tmp_path, """
CLASS P
    DIM anzahl AS INTEGER
END CLASS
DIM p AS P
p = NEW P()
PRINT p.anzhal
""")
    assert any("kein Mitglied 'anzhal'" in m for m in w), w
    assert any("Meintest du 'anzahl'?" in m for m in w), w


def test_unbekannte_methode_warnt(tmp_path):
    w = _warnungen(tmp_path, """
CLASS P
    SUB tu()
    END SUB
END CLASS
DIM p AS P
p = NEW P()
p.gibtsNicht()
""")
    assert any("kein Mitglied 'gibtsnicht'" in m for m in w), w


def test_kindmethode_ueber_basistyp_bleibt_still(tmp_path):
    """Der Fall, um den es geht: `t` ist als Tier angesagt, haelt aber einen
    Hund -- `t.belle()` ist gueltig und muss still bleiben."""
    w = _warnungen(tmp_path, """
CLASS Tier
    SUB laut()
    END SUB
END CLASS
CLASS Hund EXTENDS Tier
    SUB belle()
        PRINT "wau"
    END SUB
END CLASS
DIM t AS Tier
t = NEW Hund()
t.belle()
""")
    assert not any("kein Mitglied" in m for m in w), w


def test_geerbtes_feld_bleibt_still(tmp_path):
    w = _warnungen(tmp_path, """
CLASS Basis
    DIM hp AS INTEGER
END CLASS
CLASS Kind EXTENDS Basis
END CLASS
DIM k AS Kind
k = NEW Kind()
PRINT k.hp
""")
    assert not any("kein Mitglied" in m for m in w), w


def test_property_zaehlt_als_mitglied(tmp_path):
    w = _warnungen(tmp_path, """
CLASS P
    DIM _h AS INTEGER
    PROPERTY GET hp() AS INTEGER
        RETURN Self._h
    END PROPERTY
END CLASS
DIM p AS P
p = NEW P()
PRINT p.hp
""")
    assert not any("kein Mitglied" in m for m in w), w


def test_container_methoden_bleiben_still(tmp_path):
    """STRING/MAP/ARRAY haben ihre eigene Methoden-Tabelle, keine Klasse."""
    w = _warnungen(tmp_path, """
DIM s AS STRING
s = "abc"
PRINT s.upper()
DIM m AS MAP OF INTEGER
PRINT m.size()
""")
    assert not any("kein Mitglied" in m for m in w), w


def test_schreibender_zugriff_wird_auch_geprueft(tmp_path):
    w = _warnungen(tmp_path, """
CLASS P
    DIM anzahl AS INTEGER
END CLASS
DIM p AS P
p = NEW P()
p.anzhal = 5
""")
    assert any("kein Mitglied 'anzhal'" in m for m in w), w
