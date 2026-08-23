"""Modul `xml` -- XML lesen (Punkt 7 des Allzweck-Audits).

Der Fall dafuer ist fast immer derselbe: Daten kommen aus einem fremden
System. Rechnungen, Ausfuhrlisten, GPX-Spuren, SVG, die Antwort einer
aelteren Web-Schnittstelle.

**Nur lesend** -- anders als bei JSON, wo das Schreiben die eigentliche Luecke
war. Wer XML schreiben muss, klebt es mit `XML_ESCAPE$` zusammen.
"""
import pytest

from drachenhauch.errors import DHRuntimeError

KOPF = 'IMPORT "xml"\nDIM d AS XML_HANDLE\n'
Q = chr(34)


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


def _gb(text: str) -> str:
    """XML als GB-Zeichenkette (Anfuehrungszeichen verdoppeln)."""
    return '"' + text.replace('"', '""') + '"'


RECHNUNG = ('<rechnung nr="4711">'
            '<kunde>Anna</kunde>'
            '<posten><p menge="2">Schraube</p><p menge="5">Mutter</p></posten>'
            '</rechnung>')


# ------------------------------------------------------------------- lesen
def test_wurzel_name_und_attribut(run_gb):
    out = run_gb(KOPF + f"d = XML_PARSE({_gb(RECHNUNG)})\n"
                 'PRINT XML_NAME$(d)\nPRINT XML_ATTR$(d, "nr")\n')
    assert _lines(out) == ["rechnung", "4711"]


def test_pfad_mit_schraegstrich(run_gb):
    """Dieselbe Idee wie bei JSON, nur mit `/` -- so steht es in jedem
    XML-Beispiel der Welt."""
    out = run_gb(KOPF + f"d = XML_PARSE({_gb(RECHNUNG)})\n"
                 'PRINT XML_TEXT$(d, "kunde")\n')
    assert _lines(out) == ["Anna"]


def test_mehrfache_kinder_zaehlen_und_holen(run_gb):
    out = run_gb(KOPF + "DIM p AS XML_HANDLE\nDIM i AS INTEGER\n"
                 f"d = XML_PARSE({_gb(RECHNUNG)})\n"
                 'PRINT XML_COUNT(d, "posten/p")\n'
                 'FOR i = 0 TO XML_COUNT(d, "posten/p") - 1\n'
                 '    p = XML_AT(d, "posten/p", i)\n'
                 '    PRINT XML_ATTR$(p, "menge") + "x " + XML_TEXT$(p)\n'
                 "NEXT\n")
    assert _lines(out) == ["2", "2x Schraube", "5x Mutter"]


def test_fehlendes_attribut_nimmt_die_vorgabe(run_gb):
    """Eine fremde Datei laesst weg, was sie nicht braucht -- das ist der
    Normalfall und kein Fehler."""
    out = run_gb(KOPF + f"d = XML_PARSE({_gb(RECHNUNG)})\n"
                 'PRINT "[" + XML_ATTR$(d, "gibtsnicht") + "]"\n'
                 'PRINT XML_ATTR$(d, "gibtsnicht", "Vorgabe")\n')
    assert _lines(out) == ["[]", "Vorgabe"]


def test_has_vor_find(run_gb):
    out = run_gb(KOPF + f"d = XML_PARSE({_gb(RECHNUNG)})\n"
                 'PRINT XML_HAS(d, "kunde")\n'
                 'PRINT XML_HAS(d, "anschrift")\n')
    assert _lines(out) == ["TRUE", "FALSE"]


def test_find_ohne_treffer_sagt_wo_man_haette_fragen_koennen(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb(KOPF + f"d = XML_PARSE({_gb(RECHNUNG)})\n"
               'PRINT XML_NAME$(XML_FIND(d, "anschrift"))\n')
    assert "XML_HAS" in str(e.value)


def test_entities_und_cdata(run_gb):
    out = run_gb(KOPF + f"d = XML_PARSE({_gb('<a>5 &lt; 6 &amp; 7</a>')})\n"
                 "PRINT XML_TEXT$(d)\n"
                 f"d = XML_PARSE({_gb('<a><![CDATA[roh & <ungeschuetzt>]]></a>')})\n"
                 "PRINT XML_TEXT$(d)\n")
    assert _lines(out) == ["5 < 6 & 7", "roh & <ungeschuetzt>"]


def test_gemischter_inhalt_behaelt_die_reihenfolge(run_gb):
    """Der Modellfehler, den es zu vermeiden galt: Text und Kind-Elemente
    getrennt zu speichern verliert ihre Reihenfolge -- `Hallo <b>x</b> Welt`
    kaeme dann als `Hallo  Weltx` heraus."""
    out = run_gb(KOPF + f"d = XML_PARSE({_gb('<p>Hallo <b>schoene</b> Welt</p>')})\n"
                 "PRINT XML_TEXT$(d)\n")
    assert _lines(out) == ["Hallo schoene Welt"]


def test_deklaration_kommentar_doctype(run_gb):
    quelle = '<?xml version="1.0"?><!-- Hinweis --><!DOCTYPE a><a>x</a>'
    out = run_gb(KOPF + f"d = XML_PARSE({_gb(quelle)})\n"
                 "PRINT XML_NAME$(d)\nPRINT XML_TEXT$(d)\n")
    assert _lines(out) == ["a", "x"]


def test_namensraeume_bleiben_im_namen(run_gb):
    """Echte Namensraum-Aufloesung beantwortet eine Frage, die beim Auslesen
    einer bekannten Datei niemand stellt."""
    quelle = '<ns:a xmlns:ns="http://x"><ns:b>1</ns:b></ns:a>'
    out = run_gb(KOPF + f"d = XML_PARSE({_gb(quelle)})\n"
                 "PRINT XML_NAME$(d)\n"
                 'PRINT XML_TEXT$(d, "ns:b")\n')
    assert _lines(out) == ["ns:a", "1"]


def test_unbekannten_baum_durchlaufen(run_gb):
    """Wer die Struktur nicht kennt, geht ueber XML_CHILD_COUNT/XML_CHILD."""
    out = run_gb(KOPF + "DIM i AS INTEGER\nDIM k AS XML_HANDLE\n"
                 f"d = XML_PARSE({_gb('<a><b/><c>x</c></a>')})\n"
                 "FOR i = 0 TO XML_CHILD_COUNT(d) - 1\n"
                 "    k = XML_CHILD(d, i)\n"
                 '    PRINT XML_NAME$(k)\n'
                 "NEXT\n")
    assert _lines(out) == ["b", "c"]


def test_attributnamen_auflisten(run_gb):
    out = run_gb(KOPF + "DIM n AS ARRAY OF STRING\n"
                 f"d = XML_PARSE({_gb('<a x=' + Q + '1' + Q + ' y=' + Q + '2' + Q + '/>')})\n"
                 "n = XML_ATTR_NAMES(d)\n"
                 'PRINT JOIN$(n, ",")\n')
    assert _lines(out) == ["x,y"]


# ----------------------------------------------------------------- Fehler
def test_nicht_geschlossenes_element(run_gb):
    """Bewusst STRENG (anders als INI): eine XML-Datei kommt aus einem
    anderen Programm -- ein offenes Element heisst dort meist, dass die
    Uebertragung abgebrochen ist."""
    with pytest.raises(DHRuntimeError) as e:
        run_gb(KOPF + f"d = XML_PARSE({_gb('<a><b></a>')})\n")
    assert "schliesst" in str(e.value)


def test_fehler_nennt_die_zeile(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb(KOPF + f"d = XML_PARSE({_gb('<a>')} + CHR$(10) + {_gb('<b>')} + CHR$(10))\n")
    assert "Zeile" in str(e.value)


def test_kein_xml(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb(KOPF + f"d = XML_PARSE({_gb('einfach nur Text')})\n")
    assert "wirklich XML" in str(e.value)


def test_zwei_wurzeln(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb(KOPF + f"d = XML_PARSE({_gb('<a/><b/>')})\n")
    assert "Wurzelelemente" in str(e.value)


# ---------------------------------------------------------------- Datei
def test_laden_mit_kodierung(tmp_path, run_gb):
    (tmp_path / "alt.xml").write_bytes("<a><ort>Köln</ort></a>".encode("cp1252"))
    out = run_gb(KOPF + 'd = XML_LOAD("alt.xml", "cp1252")\n'
                 'PRINT XML_TEXT$(d, "ort")\n', base=tmp_path)
    assert _lines(out) == ["Köln"]


# --------------------------------------------------------------- schreiben
def test_escape_schuetzt_handarbeit(run_gb):
    out = run_gb('IMPORT "xml"\n'
                 f'PRINT XML_ESCAPE$({_gb("5 < 6 & " + Q + "sieben" + Q)})\n')
    assert _lines(out) == ['5 &lt; 6 &amp; &quot;sieben&quot;']


def test_escape_und_wieder_lesen(run_gb):
    """Die Zusage: was durch XML_ESCAPE$ gegangen ist, kommt beim Lesen
    unveraendert zurueck."""
    out = run_gb(KOPF + 'DIM roh AS STRING\n'
                 'roh = "5 < 6 & Ende"\n'
                 'd = XML_PARSE("<a>" + XML_ESCAPE$(roh) + "</a>")\n'
                 "PRINT XML_TEXT$(d)\n")
    assert _lines(out) == ["5 < 6 & Ende"]
