"""JSON schreiben (Punkt 2 aus docs/allzweck-audit-2.md).

Bis 2026-08 liess sich JSON nur LESEN. Wer eines bauen wollte, klebte
Zeichenketten zusammen -- und brach am ersten Anfuehrungszeichen in einem
Namen. Diese Datei haelt beides fest: dass das Bauen geht, und dass die
Regeln dabei die sind, die in der Doku stehen.

Golden-Tests gegen `dhrt`.
"""
import pytest

from drachenhauch.errors import DHRuntimeError

KOPF = 'IMPORT "json"\nDIM h AS JSON_HANDLE\n'


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


# ------------------------------------------------------------- Grundlagen
def test_leeres_objekt_und_array(run_gb):
    out = run_gb(KOPF + "h = JSON_NEW_OBJECT()\n"
                 "PRINT JSON_STRINGIFY(h)\n"
                 "h = JSON_NEW_ARRAY()\n"
                 "PRINT JSON_STRINGIFY(h)\n")
    assert _lines(out) == ["{}", "[]"]


def test_alle_werttypen(run_gb):
    out = run_gb(KOPF + "h = JSON_NEW_OBJECT()\n"
                 'JSON_SET_STRING(h, "s", "text")\n'
                 'JSON_SET_INT(h, "i", 42)\n'
                 'JSON_SET_FLOAT(h, "f", 1.5)\n'
                 'JSON_SET_BOOL(h, "b", TRUE)\n'
                 'JSON_SET_NULL(h, "n")\n'
                 "PRINT JSON_STRINGIFY(h)\n")
    assert _lines(out) == ['{"s":"text","i":42,"f":1.5,"b":true,"n":null}']


def test_reihenfolge_bleibt_die_einfuege_reihenfolge(run_gb):
    """`preserve_order` in Cargo.toml -- und das gilt auch nach REMOVE."""
    out = run_gb(KOPF + "h = JSON_NEW_OBJECT()\n"
                 'JSON_SET_INT(h, "a", 1)\n'
                 'JSON_SET_INT(h, "b", 2)\n'
                 'JSON_SET_INT(h, "c", 3)\n'
                 'PRINT JSON_REMOVE(h, "b")\n'
                 "PRINT JSON_STRINGIFY(h)\n")
    assert _lines(out) == ["TRUE", '{"a":1,"c":3}']


def test_setzen_ersetzt(run_gb):
    out = run_gb(KOPF + "h = JSON_NEW_OBJECT()\n"
                 'JSON_SET_INT(h, "a", 1)\n'
                 'JSON_SET_INT(h, "a", 2)\n'
                 "PRINT JSON_STRINGIFY(h)\n")
    assert _lines(out) == ['{"a":2}']


# ------------------------------------------------- der eigentliche Anlass
def test_sonderzeichen_werden_richtig_maskiert(run_gb):
    """Der Grund, warum es diese Builtins gibt: von Hand geklebtes JSON
    bricht genau hier."""
    out = run_gb(KOPF + "h = JSON_NEW_OBJECT()\n"
                 'JSON_SET_STRING(h, "wert", "Anna "" mit ""Zitat"" und \\ und ")\n'
                 "PRINT JSON_STRINGIFY(h)\n")
    # Das Ergebnis muss wieder einlesbar sein -- das ist die Zusage.
    zurueck = run_gb(KOPF + "h = JSON_NEW_OBJECT()\n"
                     'JSON_SET_STRING(h, "wert", "Anna "" mit ""Zitat"" und \\ und ")\n'
                     "DIM zwei AS JSON_HANDLE\n"
                     "zwei = JSON_PARSE(JSON_STRINGIFY(h))\n"
                     'PRINT JSON_GET_STRING(zwei, "wert")\n')
    assert _lines(zurueck) == ['Anna " mit "Zitat" und \\ und']
    assert "\\\"" in out


def test_zeilenumbruch_ueberlebt_den_umweg(run_gb):
    out = run_gb(KOPF + "h = JSON_NEW_OBJECT()\n"
                 'JSON_SET_STRING(h, "t", "a" + CHR$(10) + "b")\n'
                 "DIM zwei AS JSON_HANDLE\n"
                 "zwei = JSON_PARSE(JSON_STRINGIFY(h))\n"
                 'PRINT JSON_LEN(zwei, "t")\n')
    assert _lines(out) == ["3"]


# ------------------------------------------------------- verschachtelt
def test_zwischenstufen_entstehen_von_selbst(run_gb):
    out = run_gb(KOPF + "h = JSON_NEW_OBJECT()\n"
                 'JSON_SET_STRING(h, "kunde.adresse.ort", "Koeln")\n'
                 "PRINT JSON_STRINGIFY(h)\n"
                 'PRINT JSON_GET_STRING(h, "kunde.adresse.ort")\n')
    assert _lines(out) == ['{"kunde":{"adresse":{"ort":"Koeln"}}}', "Koeln"]


def test_json_einhaengen_ist_eine_kopie(run_gb):
    """Ein JSON-Baum kann sich keinen Teilbaum mit einem anderen teilen --
    spaetere Aenderungen an der Quelle schlagen deshalb NICHT durch."""
    out = run_gb(KOPF + "DIM teil AS JSON_HANDLE\n"
                 "h = JSON_NEW_OBJECT()\n"
                 "teil = JSON_NEW_OBJECT()\n"
                 'JSON_SET_INT(teil, "x", 1)\n'
                 'JSON_SET_JSON(h, "unten", teil)\n'
                 'JSON_SET_INT(teil, "x", 999)\n'
                 'PRINT JSON_GET_INT(h, "unten.x")\n')
    assert _lines(out) == ["1"]


def test_sich_selbst_einhaengen_geht(run_gb):
    """Quelle und Ziel duerfen dasselbe Handle sein (RefCell-Falle)."""
    out = run_gb(KOPF + "h = JSON_NEW_OBJECT()\n"
                 'JSON_SET_INT(h, "a", 1)\n'
                 'JSON_SET_JSON(h, "kopie", h)\n'
                 "PRINT JSON_STRINGIFY(h)\n")
    assert _lines(out) == ['{"a":1,"kopie":{"a":1}}']


# ------------------------------------------------------------- Arrays
def test_anhaengen_an_die_wurzel(run_gb):
    out = run_gb(KOPF + "h = JSON_NEW_ARRAY()\n"
                 'JSON_APPEND_INT(h, "", 1)\n'
                 'JSON_APPEND_STRING(h, "", "zwei")\n'
                 'JSON_APPEND_BOOL(h, "", FALSE)\n'
                 'JSON_APPEND_FLOAT(h, "", 0.5)\n'
                 "PRINT JSON_STRINGIFY(h)\n")
    assert _lines(out) == ['[1,"zwei",false,0.5]']


def test_anhaengen_an_ein_feld(run_gb):
    out = run_gb(KOPF + "DIM liste AS JSON_HANDLE\n"
                 "h = JSON_NEW_OBJECT()\n"
                 "liste = JSON_NEW_ARRAY()\n"
                 'JSON_SET_JSON(h, "posten", liste)\n'
                 'JSON_APPEND_INT(h, "posten", 7)\n'
                 'JSON_APPEND_INT(h, "posten", 8)\n'
                 'PRINT JSON_LEN(h, "posten")\n'
                 "PRINT JSON_STRINGIFY(h)\n")
    assert _lines(out) == ["2", '{"posten":[7,8]}']


def test_objekte_an_ein_array_haengen(run_gb):
    """Das Muster fuer eine Liste von Datensaetzen."""
    out = run_gb(KOPF + "DIM zeile AS JSON_HANDLE\n"
                 "DIM i AS INTEGER\n"
                 "h = JSON_NEW_ARRAY()\n"
                 "FOR i = 1 TO 2\n"
                 "    zeile = JSON_NEW_OBJECT()\n"
                 '    JSON_SET_INT(zeile, "nr", i)\n'
                 '    JSON_APPEND_JSON(h, "", zeile)\n'
                 "NEXT\n"
                 "PRINT JSON_STRINGIFY(h)\n")
    assert _lines(out) == ['[{"nr":1},{"nr":2}]']


def test_array_element_ersetzen(run_gb):
    out = run_gb(KOPF + "h = JSON_NEW_ARRAY()\n"
                 'JSON_APPEND_INT(h, "", 1)\n'
                 'JSON_APPEND_INT(h, "", 2)\n'
                 'JSON_SET_INT(h, "1", 99)\n'
                 "PRINT JSON_STRINGIFY(h)\n")
    assert _lines(out) == ["[1,99]"]


def test_array_element_entfernen_rueckt_auf(run_gb):
    out = run_gb(KOPF + "h = JSON_NEW_ARRAY()\n"
                 'JSON_APPEND_INT(h, "", 1)\n'
                 'JSON_APPEND_INT(h, "", 2)\n'
                 'JSON_APPEND_INT(h, "", 3)\n'
                 'PRINT JSON_REMOVE(h, "1")\n'
                 "PRINT JSON_STRINGIFY(h)\n")
    assert _lines(out) == ["TRUE", "[1,3]"]


# ------------------------------------------------------------- JSON_KEYS
def test_keys_liefert_die_schluessel(run_gb):
    out = run_gb(KOPF + "DIM k AS ARRAY OF STRING\n"
                 "h = JSON_NEW_OBJECT()\n"
                 'JSON_SET_INT(h, "eins", 1)\n'
                 'JSON_SET_INT(h, "zwei", 2)\n'
                 'k = JSON_KEYS(h, "")\n'
                 'PRINT JOIN$(k, ",")\n')
    assert _lines(out) == ["eins,zwei"]


def test_keys_auf_einem_pfad(run_gb):
    out = run_gb(KOPF + "DIM k AS ARRAY OF STRING\n"
                 'h = JSON_PARSE("{""a"": {""x"": 1, ""y"": 2}}")\n'
                 'k = JSON_KEYS(h, "a")\n'
                 'PRINT JOIN$(k, ",")\n')
    assert _lines(out) == ["x,y"]


def test_keys_auf_einem_array_ist_ein_fehler(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb(KOPF + "h = JSON_NEW_ARRAY()\n"
               'DIM k AS ARRAY OF STRING\n'
               'k = JSON_KEYS(h, "")\n')
    assert "kein Objekt" in str(e.value)


# ------------------------------------------------------- Referenz-Semantik
def test_handles_sind_referenzen(run_gb):
    """Wie MAP und ARRAY: `b = a` legt keine Kopie an."""
    out = run_gb('IMPORT "json"\n'
                 "DIM a AS JSON_HANDLE\n"
                 "DIM b AS JSON_HANDLE\n"
                 "a = JSON_NEW_OBJECT()\n"
                 "b = a\n"
                 'JSON_SET_INT(b, "x", 5)\n'
                 "PRINT JSON_STRINGIFY(a)\n")
    assert _lines(out) == ['{"x":5}']


def test_geparstes_dokument_ist_veraenderbar(run_gb):
    """Lesen und Schreiben auf demselben Handle -- der uebliche Fall:
    Antwort einlesen, ein Feld ergaenzen, zurueckschicken."""
    out = run_gb(KOPF + 'h = JSON_PARSE("{""a"": 1}")\n'
                 'JSON_SET_INT(h, "b", 2)\n'
                 "PRINT JSON_STRINGIFY(h)\n")
    assert _lines(out) == ['{"a":1,"b":2}']


# ----------------------------------------------------------- Fehlerfaelle
def test_leerer_pfad_beim_setzen_ist_ein_fehler(run_gb):
    """Beim LESEN meint der leere Pfad die Wurzel. Beim Schreiben hiesse er
    'alles wegwerfen' -- eine versehentlich leere Variable darf das nicht."""
    with pytest.raises(DHRuntimeError) as e:
        run_gb(KOPF + "h = JSON_NEW_OBJECT()\n" 'JSON_SET_INT(h, "", 1)\n')
    assert "leerer Pfad" in str(e.value)


def test_zahl_segment_ohne_array_erklaert_sich(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb(KOPF + "h = JSON_NEW_OBJECT()\n" 'JSON_SET_STRING(h, "posten.0", "x")\n')
    msg = str(e.value)
    assert "ist eine Zahl" in msg
    # Der Hinweis muss den ELTERN-Pfad nennen, nicht den vollen.
    assert 'JSON_SET_JSON(h, "posten", JSON_NEW_ARRAY())' in msg


def test_zahl_als_schluessel_im_bestehenden_objekt_ist_erlaubt(run_gb):
    """Nur eine FRISCH angelegte Zwischenstufe ist zweideutig."""
    out = run_gb(KOPF + "h = JSON_NEW_OBJECT()\n"
                 'JSON_SET_STRING(h, "t.name", "a")\n'
                 'JSON_SET_STRING(h, "t.0", "erlaubt")\n'
                 "PRINT JSON_STRINGIFY(h)\n")
    assert _lines(out) == ['{"t":{"name":"a","0":"erlaubt"}}']


def test_in_einen_skalar_schreiben_ist_ein_fehler(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb(KOPF + "h = JSON_NEW_OBJECT()\n"
               'JSON_SET_STRING(h, "a", "text")\n'
               'JSON_SET_INT(h, "a.b", 1)\n')
    assert "ist ein Text" in str(e.value)


def test_index_ausserhalb_verweist_auf_append(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb(KOPF + "h = JSON_NEW_ARRAY()\n" 'JSON_SET_INT(h, "0", 1)\n')
    assert "JSON_APPEND" in str(e.value)


def test_anhaengen_an_ein_nicht_array_ist_ein_fehler(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb(KOPF + "h = JSON_NEW_OBJECT()\n"
               'JSON_SET_INT(h, "a", 1)\n'
               'JSON_APPEND_INT(h, "a", 2)\n')
    assert "kein Array" in str(e.value)


def test_remove_meldet_ob_etwas_da_war(run_gb):
    out = run_gb(KOPF + "h = JSON_NEW_OBJECT()\n"
                 'JSON_SET_INT(h, "a", 1)\n'
                 'PRINT JSON_REMOVE(h, "a")\n'
                 'PRINT JSON_REMOVE(h, "a")\n')
    assert _lines(out) == ["TRUE", "FALSE"]


def test_null_laesst_sich_anhaengen(run_gb):
    """Eine Liste mit LEEREN Plaetzen -- so notiert der Tracker eine Zeile
    ohne Note. Bis 2026-09-04 gab es dafuer keine Form: JSON_SET_NULL
    braucht einen Platz, der schon da ist, und den legt bei einem Array nur
    ein APPEND an."""
    out = run_gb(KOPF + "h = JSON_NEW_ARRAY()\n"
                 'JSON_APPEND_INT(h, "", 60)\n'
                 'JSON_APPEND_NULL(h, "")\n'
                 'JSON_APPEND_INT(h, "", 62)\n'
                 "PRINT JSON_STRINGIFY(h)\n"
                 'PRINT JSON_TYPE(h, "1")\n')
    assert _lines(out) == ["[60,null,62]", "null"]
