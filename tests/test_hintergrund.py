"""WP H -- Nebenlaeufigkeit: Auftraege im Hintergrund.

DB_QUERY_START/READY/RESULT und SHELL_START/READY/RESULT$ -- dasselbe Muster
wie HTTP_GET_START. Was hier laeuft, ist reine Rust-Arbeit ohne VM; warum kein
GB-Code im Hintergrund laufen kann, steht in `rust/.../hintergrund.rs` und in
der Roadmap.
"""
import sqlite3
import sys

import pytest

from drachenhauch.errors import DHRuntimeError


def _db_bauen(pfad, zeilen=2000):
    con = sqlite3.connect(str(pfad))
    con.execute("CREATE TABLE zahlen (a INTEGER, b TEXT)")
    con.executemany("INSERT INTO zahlen VALUES (?, ?)",
                    [(i, f"zeile {i}") for i in range(zeilen)])
    con.commit()
    con.close()


# ------------------------------------------------------ Abfrage im Hintergrund

def test_abfrage_laeuft_und_liefert_dieselben_zeilen(run_gb, tmp_path):
    _db_bauen(tmp_path / "d.db")
    out = run_gb('IMPORT "db"\n'
                 'DIM auftrag AS INTEGER\nDIM erg AS INTEGER\nDIM n AS INTEGER\n'
                 'auftrag = DB_QUERY_START("d.db", "SELECT a FROM zahlen")\n'
                 'WHILE NOT DB_QUERY_READY(auftrag)\n'
                 '    SLEEP(1)\n'
                 'WEND\n'
                 'erg = DB_QUERY_RESULT(auftrag)\n'
                 'WHILE DB_NEXT(erg)\n'
                 '    n = n + 1\n'
                 'WEND\n'
                 'PRINT n', base=tmp_path)
    assert out.strip() == "2000"


def test_das_programm_kommt_waehrenddessen_zum_zug(run_gb, tmp_path):
    """Der eigentliche Punkt: die Schleife laeuft weiter, waehrend die Abfrage
    arbeitet. Genau das ging vorher nicht -- DB_QUERY hielt alles an."""
    _db_bauen(tmp_path / "d.db", 20000)
    out = run_gb('IMPORT "db"\n'
                 'DIM auftrag AS INTEGER\nDIM runden AS INTEGER\n'
                 'auftrag = DB_QUERY_START("d.db", "SELECT a, b FROM zahlen WHERE a % ? = 0", 3)\n'
                 'WHILE NOT DB_QUERY_READY(auftrag)\n'
                 '    runden = runden + 1\n'
                 'WEND\n'
                 'PRINT runden > 0\n'
                 'PRINT DB_QUERY_PENDING()', base=tmp_path)
    assert out.split() == ["TRUE", "1"]     # noch nicht abgeholt -> noch offen


def test_parameter_werden_gebunden(run_gb, tmp_path):
    _db_bauen(tmp_path / "d.db", 100)
    out = run_gb('IMPORT "db"\n'
                 'DIM auftrag AS INTEGER\nDIM erg AS INTEGER\n'
                 'auftrag = DB_QUERY_START("d.db", "SELECT b FROM zahlen WHERE a = ?", 42)\n'
                 'WHILE NOT DB_QUERY_READY(auftrag)\n    SLEEP(1)\nWEND\n'
                 'erg = DB_QUERY_RESULT(auftrag)\n'
                 'PRINT DB_NEXT(erg)\n'
                 'PRINT DB_GET_STRING(erg, 0)', base=tmp_path)
    assert out.split("\n")[:2] == ["TRUE", "zeile 42"]


def test_mehrere_auftraege_gleichzeitig(run_gb, tmp_path):
    _db_bauen(tmp_path / "d.db", 500)
    out = run_gb('IMPORT "db"\n'
                 'DIM a1 AS INTEGER\nDIM a2 AS INTEGER\nDIM e AS INTEGER\nDIM n AS INTEGER\n'
                 'a1 = DB_QUERY_START("d.db", "SELECT a FROM zahlen WHERE a < 100")\n'
                 'a2 = DB_QUERY_START("d.db", "SELECT a FROM zahlen WHERE a < 10")\n'
                 'PRINT DB_QUERY_PENDING()\n'
                 'WHILE NOT DB_QUERY_READY(a2)\n    SLEEP(1)\nWEND\n'
                 'e = DB_QUERY_RESULT(a2)\n'
                 'WHILE DB_NEXT(e)\n    n = n + 1\nWEND\n'
                 'PRINT n\n'
                 'WHILE NOT DB_QUERY_READY(a1)\n    SLEEP(1)\nWEND\n'
                 'e = DB_QUERY_RESULT(a1)\n'
                 'n = 0\n'
                 'WHILE DB_NEXT(e)\n    n = n + 1\nWEND\n'
                 'PRINT n', base=tmp_path)
    # Die Nummern bleiben gueltig, auch wenn der zweite zuerst abgeholt wird.
    assert out.split() == ["2", "10", "100"]


def test_abholen_ohne_ready_sagt_was_fehlt(run_gb, tmp_path):
    _db_bauen(tmp_path / "d.db", 10)
    with pytest.raises(DHRuntimeError, match="erst DB_QUERY_READY"):
        run_gb('IMPORT "db"\n'
               'DIM a AS INTEGER\n'
               'a = DB_QUERY_START("d.db", "SELECT a FROM zahlen")\n'
               'PRINT DB_QUERY_RESULT(a)', base=tmp_path)


def test_kaputtes_sql_meldet_sich_beim_abholen(run_gb, tmp_path):
    """Der Fehler entsteht auf dem Auftrags-Thread und wird beim Abholen
    geworfen -- dort, wo das Programm damit umgehen kann."""
    _db_bauen(tmp_path / "d.db", 10)
    out = run_gb('IMPORT "db"\n'
                 'DIM a AS INTEGER\n'
                 'a = DB_QUERY_START("d.db", "SELECT quatsch FROM gibtsnicht")\n'
                 'WHILE NOT DB_QUERY_READY(a)\n    SLEEP(1)\nWEND\n'
                 'TRY\n'
                 '    PRINT DB_QUERY_RESULT(a)\n'
                 'CATCH e\n'
                 '    PRINT "gefangen"\n'
                 'END TRY', base=tmp_path)
    assert out.strip() == "gefangen"


def test_fehlende_datei_wird_von_sqlite_angelegt(run_gb, tmp_path):
    """Gemessen statt angenommen: SQLite legt eine fehlende Datei einfach an.
    Ein Fehler kommt also erst von der fehlenden TABELLE -- und der kommt beim
    Abholen, dort wo das Programm damit umgehen kann."""
    out = run_gb('IMPORT "db"\n'
                 'DIM a AS INTEGER\n'
                 'a = DB_QUERY_START("neu.db", "SELECT * FROM gibtsnicht")\n'
                 'WHILE NOT DB_QUERY_READY(a)\n    SLEEP(1)\nWEND\n'
                 'TRY\n'
                 '    PRINT DB_QUERY_RESULT(a)\n'
                 'CATCH e\n'
                 '    PRINT "gefangen"\n'
                 'END TRY\n'
                 'PRINT FILEEXISTS("neu.db")', base=tmp_path)
    assert out.split() == ["gefangen", "TRUE"]


def test_abbrechen_gibt_den_platz_frei(run_gb, tmp_path):
    _db_bauen(tmp_path / "d.db", 100)
    out = run_gb('IMPORT "db"\n'
                 'DIM a AS INTEGER\n'
                 'a = DB_QUERY_START("d.db", "SELECT a FROM zahlen")\n'
                 'DB_QUERY_CANCEL(a)\n'
                 'PRINT DB_QUERY_PENDING()\n'
                 'PRINT DB_QUERY_READY(a)', base=tmp_path)
    assert out.split() == ["0", "FALSE"]


def test_die_eigene_verbindung_sieht_nur_festgeschriebenes(run_gb, tmp_path):
    """Dokumentierte Grenze: der Auftrag oeffnet seine EIGENE Verbindung, sieht
    also die offene Transaktion des Programms nicht."""
    _db_bauen(tmp_path / "d.db", 10)
    out = run_gb('IMPORT "db"\n'
                 'DIM c AS DB_CONN\nDIM a AS INTEGER\nDIM e AS INTEGER\nDIM n AS INTEGER\n'
                 'c = DB_OPEN("d.db")\n'
                 'DB_BEGIN(c)\n'
                 "DB_EXEC(c, \"INSERT INTO zahlen VALUES (999, 'neu')\")\n"
                 'a = DB_QUERY_START("d.db", "SELECT a FROM zahlen WHERE a = 999")\n'
                 'WHILE NOT DB_QUERY_READY(a)\n    SLEEP(1)\nWEND\n'
                 'e = DB_QUERY_RESULT(a)\n'
                 'WHILE DB_NEXT(e)\n    n = n + 1\nWEND\n'
                 'PRINT n\n'
                 'DB_ROLLBACK(c)\n'
                 'DB_CLOSE(c)', base=tmp_path)
    assert out.strip() == "0"


# ----------------------------------------------------- Prozess im Hintergrund

@pytest.mark.skipif(sys.platform != "win32", reason="benutzt cmd.exe")
def test_prozess_im_hintergrund(run_gb):
    out = run_gb('DIM p AS INTEGER\n'
                 'p = SHELL_START("cmd", "/c", "echo hallo")\n'
                 'WHILE NOT SHELL_READY(p)\n    SLEEP(1)\nWEND\n'
                 'PRINT TRIM$(SHELL_RESULT$(p))')
    assert out.strip() == "hallo"


@pytest.mark.skipif(sys.platform != "win32", reason="benutzt cmd.exe")
def test_rueckgabewert_nach_dem_abholen(run_gb):
    out = run_gb('DIM p AS INTEGER\nDIM t AS STRING\n'
                 'p = SHELL_START("cmd", "/c", "exit 7")\n'
                 'WHILE NOT SHELL_READY(p)\n    SLEEP(1)\nWEND\n'
                 't = SHELL_RESULT$(p)\n'
                 'PRINT SHELL_CODE()')
    assert out.strip() == "7"


@pytest.mark.skipif(sys.platform != "win32", reason="benutzt cmd.exe")
def test_stderr_getrennt_von_stdout(run_gb):
    out = run_gb('DIM p AS INTEGER\nDIM t AS STRING\n'
                 'p = SHELL_START("cmd", "/c", "echo fehler 1>&2")\n'
                 'WHILE NOT SHELL_READY(p)\n    SLEEP(1)\nWEND\n'
                 't = SHELL_RESULT$(p)\n'
                 'PRINT "[" + TRIM$(t) + "]"\n'
                 'PRINT INSTR(SHELL_ERR$(), "fehler") >= 0')
    assert out.split("\n")[:2] == ["[]", "TRUE"]


@pytest.mark.skipif(sys.platform != "win32", reason="benutzt cmd.exe")
def test_das_programm_laeuft_waehrend_der_prozess_arbeitet(run_gb):
    out = run_gb('DIM p AS INTEGER\nDIM runden AS INTEGER\n'
                 'p = SHELL_START("cmd", "/c", "ping -n 2 127.0.0.1 > nul")\n'
                 'WHILE NOT SHELL_READY(p)\n'
                 '    runden = runden + 1\n'
                 'WEND\n'
                 'PRINT runden > 0')
    assert out.strip() == "TRUE"


def test_unbekanntes_programm_meldet_sich_beim_abholen(run_gb):
    out = run_gb('DIM p AS INTEGER\n'
                 'p = SHELL_START("gibt_es_ganz_sicher_nicht_xyz123")\n'
                 'WHILE NOT SHELL_READY(p)\n    SLEEP(1)\nWEND\n'
                 'TRY\n'
                 '    PRINT SHELL_RESULT$(p)\n'
                 'CATCH e\n'
                 '    PRINT "gefangen"\n'
                 'END TRY')
    assert out.strip() == "gefangen"


@pytest.mark.skipif(sys.platform != "win32", reason="benutzt cmd.exe")
def test_abholen_ohne_ready_sagt_auch_hier_was_fehlt(run_gb):
    with pytest.raises(DHRuntimeError, match="erst SHELL_READY"):
        run_gb('DIM p AS INTEGER\n'
               'p = SHELL_START("cmd", "/c", "ping -n 3 127.0.0.1 > nul")\n'
               'PRINT SHELL_RESULT$(p)')


@pytest.mark.skipif(sys.platform != "win32", reason="benutzt cmd.exe")
def test_pending_zaehlt_und_cancel_raeumt_auf(run_gb):
    out = run_gb('DIM p AS INTEGER\n'
                 'p = SHELL_START("cmd", "/c", "ping -n 3 127.0.0.1 > nul")\n'
                 'PRINT SHELL_PENDING()\n'
                 'SHELL_CANCEL(p)\n'
                 'PRINT SHELL_PENDING()')
    assert out.split() == ["1", "0"]


@pytest.mark.skipif(sys.platform != "win32", reason="benutzt cmd.exe")
def test_blockierendes_shell_bleibt_daneben_bestehen(run_gb):
    """SHELL aus WP A ist unveraendert -- SHELL_START ist ein Zusatz, kein
    Ersatz."""
    assert run_gb('PRINT SHELL("cmd", "/c", "exit 4")') == "4\n"
