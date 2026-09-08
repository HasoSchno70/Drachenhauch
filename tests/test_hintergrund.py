"""WP H -- Nebenlaeufigkeit: Auftraege im Hintergrund.

DB_QUERY_START/READY/RESULT und SHELL_START/READY/RESULT$ -- dasselbe Muster
wie HTTP_GET_START. Was hier laeuft, ist reine Rust-Arbeit ohne VM; warum kein
GB-Code im Hintergrund laufen kann, steht in `rust/.../hintergrund.rs` und in
der Roadmap.

Die uebertragbaren Tests liegen seit 2026-09-08 als Pruefsammlung in
`tests/pruef/hintergrund.dhtest` (dhrt test); hier bleiben nur die, die ein Bild,
eine geschriebene Datei oder den Quelltext mit einem fremden Leser pruefen.
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


def test_abholen_ohne_ergebnis_sagt_was_fehlt(run_gb, tmp_path):
    """Zweimal abholen -- beim zweiten Mal ist nichts mehr da.

    Frueher stand hier "abholen, OHNE vorher READY zu fragen". Das war ein
    Wettlauf: die Pruefung ging nur auf, solange die Abfrage noch LIEF. Unter
    Linux war ein SELECT ueber zehn Zeilen schneller fertig als der naechste
    Bytecode-Befehl, und der Test fiel um (CI-Lauf 32510397485); Windows
    gewann das Rennen bloss zufaellig.

    Beide Wege landen an derselben Stelle: `abholen` gibt None zurueck, und
    die VM macht daraus diese Meldung. Ueber das zweite Abholen ist sie ohne
    jede Zeitannahme zu erreichen. Den Zweig "laeuft noch" deckt test_task.py
    ab -- ein Task laeuft in einem Kindprozess, da genuegt schon dessen Start,
    damit er sicher noch nicht fertig ist.
    """
    _db_bauen(tmp_path / "d.db", 10)
    with pytest.raises(DHRuntimeError, match="erst DB_QUERY_READY"):
        run_gb('IMPORT "db"\n'
               'DIM a AS INTEGER\n'
               'a = DB_QUERY_START("d.db", "SELECT a FROM zahlen")\n'
               'WHILE NOT DB_QUERY_READY(a)\n    SLEEP(1)\nWEND\n'
               'PRINT DB_QUERY_RESULT(a)\n'
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
