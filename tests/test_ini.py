"""Modul `ini` -- Einstellungsdateien (Punkt 7 des Allzweck-Audits).

Es gab JSON und CSV. Fuer eine Datei, die ein Mensch mit dem Editor anfassen
soll, ist beides unhandlich: JSON verzeiht kein Komma zu viel, CSV hat keine
benannten Felder.

**Ohne eigenen Handle-Typ:** eine INI-Datei IST hier eine `MAP OF STRING` mit
Punkt-Schluesseln. Das spart ein Dutzend Getter -- `MAPGETOR`, `MAPPUT`,
`MAPKEYS` und `VAL` koennen das alles schon.
"""
import pytest

from drachenhauch.errors import DHRuntimeError

KOPF = 'IMPORT "ini"\nDIM c AS MAP OF STRING\n'
NL = ' + CHR$(10) + '


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


def _quelle(*zeilen):
    """INI-Text als GB-Ausdruck -- CHR$(10) statt echter Umbrueche."""
    return NL.join(f'"{z}"' for z in zeilen)


# ------------------------------------------------------------------ lesen
def test_abschnitte_werden_zu_punkt_schluesseln(run_gb):
    out = run_gb(KOPF + f"c = INI_PARSE({_quelle('[fenster]', 'breite=1280')})\n"
                 'PRINT MAPGET(c, "fenster.breite")\n')
    assert _lines(out) == ["1280"]


def test_fehlender_wert_nimmt_die_vorgabe(run_gb):
    """Der haeufigste Fall bei einer Einstellungsdatei: der Schluessel ist
    noch nicht da. Das ist kein Fehler, dafuer gibt es MAPGETOR."""
    out = run_gb(KOPF + f"c = INI_PARSE({_quelle('[a]', 'x=1')})\n"
                 'PRINT MAPGETOR(c, "a.gibtsnicht", "Vorgabe")\n')
    assert _lines(out) == ["Vorgabe"]


def test_kommentare_und_leerzeilen(run_gb):
    out = run_gb(KOPF + f"c = INI_PARSE({_quelle('; hallo', '', '# auch', '[a]', 'x=1')})\n"
                 "PRINT MAPSIZE(c)\n")
    assert _lines(out) == ["1"]


def test_ohne_abschnitt_bleibt_der_name_nackt(run_gb):
    out = run_gb(KOPF + f"c = INI_PARSE({_quelle('name=Anna', '[a]', 'x=1')})\n"
                 'PRINT MAPGET(c, "name")\n')
    assert _lines(out) == ["Anna"]


def test_semikolon_im_wert_bleibt(run_gb):
    """Nur am ZEILENANFANG ist es ein Kommentar -- sonst waere ein Pfad wie
    `C:/a;C:/b` nicht speicherbar."""
    out = run_gb(KOPF + f"c = INI_PARSE({_quelle('pfad=C:/a;C:/b')})\n"
                 'PRINT MAPGET(c, "pfad")\n')
    assert _lines(out) == ["C:/a;C:/b"]


def test_gleichheitszeichen_im_wert(run_gb):
    out = run_gb(KOPF + f"c = INI_PARSE({_quelle('formel=a=b+c')})\n"
                 'PRINT MAPGET(c, "formel")\n')
    assert _lines(out) == ["a=b+c"]


def test_kaputte_zeilen_halten_nicht_auf(run_gb):
    """Eine Einstellungsdatei bearbeitet ein Mensch, oft kein Programmierer.
    Eine unklare Zeile darf nicht den ganzen Start verhindern -- anders als
    bei JSON, wo eine kaputte Datei fast immer ein Programmfehler ist."""
    out = run_gb(KOPF + f"c = INI_PARSE({_quelle('[a]', 'ohne Gleichheitszeichen', 'x=1')})\n"
                 "PRINT MAPSIZE(c)\n"
                 'PRINT MAPGET(c, "a.x")\n')
    assert _lines(out) == ["1", "1"]


def test_reihenfolge_bleibt(run_gb):
    out = run_gb(KOPF + f"c = INI_PARSE({_quelle('[a]', 'z=1', 'b=2', 'm=3')})\n"
                 "DIM k AS ARRAY OF STRING\n"
                 "k = MAPKEYS(c)\n"
                 'PRINT JOIN$(k, ",")\n')
    assert _lines(out) == ["a.z,a.b,a.m"]


# --------------------------------------------------------------- schreiben
def test_text_gruppiert_nach_abschnitt(run_gb):
    out = run_gb(KOPF + "c = INI_PARSE(\"\")\n"
                 'MAPPUT(c, "fenster.breite", "1280")\n'
                 'MAPPUT(c, "ton.laut", "0.8")\n'
                 'MAPPUT(c, "fenster.hoehe", "720")\n'
                 "PRINT INI_TEXT$(c)\n")
    text = out.replace("\r\n", "\n")
    assert "[fenster]\nbreite=1280\nhoehe=720\n" in text, text
    assert "[ton]\nlaut=0.8" in text, text


def test_hin_und_zurueck_ueber_eine_datei(tmp_path, run_gb):
    out = run_gb(KOPF + "DIM zwei AS MAP OF STRING\n"
                 f"c = INI_PARSE({_quelle('[fenster]', 'titel=Mein Spiel', '[ton]', 'laut=0.8')})\n"
                 'INI_SAVE("e.ini", c)\n'
                 'zwei = INI_LOAD("e.ini")\n'
                 "PRINT MAPSIZE(zwei)\n"
                 'PRINT MAPGET(zwei, "fenster.titel")\n'
                 'PRINT MAPGET(zwei, "ton.laut")\n', base=tmp_path)
    assert _lines(out) == ["2", "Mein Spiel", "0.8"]


def test_heikle_werte_ueberleben(tmp_path, run_gb):
    """Leerraum am Rand und ein fuehrendes `#` gingen ohne
    Anfuehrungszeichen verloren."""
    out = run_gb(KOPF + "DIM zwei AS MAP OF STRING\n"
                 'c = INI_PARSE("")\n'
                 'MAPPUT(c, "a.rand", "  x  ")\n'
                 'MAPPUT(c, "a.raute", "#eins")\n'
                 'INI_SAVE("e.ini", c)\n'
                 'zwei = INI_LOAD("e.ini")\n'
                 'PRINT "[" + MAPGET(zwei, "a.rand") + "]"\n'
                 'PRINT MAPGET(zwei, "a.raute")\n', base=tmp_path)
    assert _lines(out) == ["[  x  ]", "#eins"]


def test_kodierung_geht_auch_hier(tmp_path, run_gb):
    """Eine alte Einstellungsdatei ist oft cp1252 -- Punkt 3 gilt weiter."""
    (tmp_path / "alt.ini").write_bytes("[a]\nort=Köln\n".encode("cp1252"))
    out = run_gb(KOPF + 'c = INI_LOAD("alt.ini", "cp1252")\n'
                 'PRINT MAPGET(c, "a.ort")\n', base=tmp_path)
    assert _lines(out) == ["Köln"]


def test_zahlen_map_laesst_sich_speichern(tmp_path, run_gb):
    """Eine INI-Datei kennt nur Text -- wer seine Zahlen in einer
    Zahlen-Map haelt, soll sie trotzdem speichern koennen."""
    out = run_gb('IMPORT "ini"\n'
                 "DIM z AS MAP OF INTEGER\n"
                 'MAPPUT(z, "a.x", 42)\n'
                 "PRINT INI_TEXT$(z)\n")
    assert "x=42" in out


def test_falscher_typ_erklaert_sich(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb('IMPORT "ini"\nPRINT INI_TEXT$("kein Map")\n')
    assert "MAP" in str(e.value)
