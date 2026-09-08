"""WP E -- Pruefen und Melden: ASSERT/ASSERT_EQ, Sammel-Modus, Bilanz,
LOG_DEBUG/INFO/WARN/ERROR.

Braucht `run_gb_roh` (aus WP A): geprueft werden hier gerade die Dinge, die
`run_gb` wegabstrahiert -- der Rueckgabewert und die stderr-Ausgabe.

Die uebertragbaren Tests liegen seit 2026-09-08 als Pruefsammlung in
`tests/pruef/pruefen.dhtest` (dhrt test); hier bleiben nur die, die ein Bild,
eine geschriebene Datei oder den Quelltext mit einem fremden Leser pruefen.

Die uebertragbaren Tests liegen seit 2026-09-08 als Pruefsammlung in
`tests/pruef/pruefen.dhtest` (dhrt test); hier bleiben nur die, die ein Bild,
eine geschriebene Datei oder den Quelltext mit einem fremden Leser pruefen.
"""
import pytest

from drachenhauch.errors import DHRuntimeError


# ------------------------------------------- Vorgabe: eine Pruefung bricht ab


def test_abbruch_nennt_datei_und_zeile(run_gb_roh):
    """Im Abbruch-Modus kommt die Fundstelle aus dem gewohnten
    Laufzeitfehler-Pfad -- ohne Zusatzaufwand."""
    _, _, err = run_gb_roh('PRINT "a"\nPRINT "b"\nASSERT(FALSE, "hier")')
    assert ":3:" in err and "hier" in err


def test_ohne_meldung_gibt_es_trotzdem_eine(run_gb_roh):
    _, _, err = run_gb_roh('ASSERT(FALSE)')
    assert "Bedingung nicht erfuellt" in err


# ------------------------------------------------------------ Sammel-Modus


def test_sammeln_laesst_sich_wieder_ausschalten(run_gb_roh):
    code, _, _ = run_gb_roh('ASSERT_COLLECT(TRUE)\n'
                            'ASSERT_EQ(1, 2)\n'
                            'ASSERT_COLLECT(FALSE)\n'
                            'ASSERT_EQ(3, 4)\n'
                            'PRINT "nie"')
    assert code != 0


# --------------------------------------------------- ASSERT_EQ und Typen


# ------------------------------------------------------------------- LOG_*

def test_log_geht_nach_stderr(run_gb_roh):
    code, out, err = run_gb_roh('PRINT "nutzdaten"\nLOG_INFO("meldung")')
    assert out == "nutzdaten\n"
    assert "meldung" in err and "INFO" in err


def test_log_debug_schweigt_per_vorgabe(run_gb_roh):
    _, _, err = run_gb_roh('LOG_DEBUG("leise")\nLOG_INFO("laut")')
    assert "leise" not in err
    assert "laut" in err


def test_log_pegel_ueber_die_umgebung(run_gb_roh, monkeypatch):
    monkeypatch.setenv("DH_LOG", "debug")
    _, _, err = run_gb_roh('LOG_DEBUG("jetzt sichtbar")')
    assert "jetzt sichtbar" in err


def test_log_pegel_warn_unterdrueckt_info(run_gb_roh, monkeypatch):
    monkeypatch.setenv("DH_LOG", "warn")
    _, _, err = run_gb_roh('LOG_INFO("weg")\nLOG_WARN("da")\nLOG_ERROR("auch da")')
    assert "weg" not in err
    assert "da" in err and "auch da" in err


def test_unbekannter_pegel_faellt_auf_info_zurueck(run_gb_roh, monkeypatch):
    monkeypatch.setenv("DH_LOG", "quatsch")
    _, _, err = run_gb_roh('LOG_DEBUG("leise")\nLOG_INFO("laut")')
    assert "leise" not in err and "laut" in err


def test_log_hat_zeitstempel_und_stufe(run_gb_roh):
    import re
    _, _, err = run_gb_roh('LOG_WARN("achtung")')
    assert re.search(r"\d{2}:\d{2}:\d{2} WARN\s+achtung", err), err


def test_log_stringifiziert_wie_print(run_gb_roh):
    _, _, err = run_gb_roh('LOG_INFO(42)\nLOG_INFO(TRUE)')
    assert "42" in err and "TRUE" in err


def test_reihenfolge_von_print_und_log_bleibt_erhalten(dhrt_pfad, tmp_path):
    """Wie bei EPRINT (WP A): PRINT wird gepuffert, LOG_* muss den Puffer
    vorher leeren -- sonst staenden alle Meldungen vor allen Nutzdaten."""
    import subprocess
    quelle = tmp_path / "reihenfolge.dh"
    quelle.write_text('PRINT "eins"\nLOG_INFO("zwei")\nPRINT "drei"\n', encoding="utf-8")
    zusammen = tmp_path / "beides.txt"
    with open(zusammen, "w", encoding="utf-8") as f:
        subprocess.run([dhrt_pfad, "run", str(quelle)], stdout=f, stderr=f, timeout=60)
    zeilen = zusammen.read_text(encoding="utf-8").replace("\r\n", "\n").splitlines()
    assert zeilen[0] == "eins"
    assert "zwei" in zeilen[1]
    assert zeilen[2] == "drei"
