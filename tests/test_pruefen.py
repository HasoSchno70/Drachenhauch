"""WP E -- Pruefen und Melden: ASSERT/ASSERT_EQ, Sammel-Modus, Bilanz,
LOG_DEBUG/INFO/WARN/ERROR.

Braucht `run_gb_roh` (aus WP A): geprueft werden hier gerade die Dinge, die
`run_gb` wegabstrahiert -- der Rueckgabewert und die stderr-Ausgabe.
"""
import pytest

from drachenhauch.errors import DHRuntimeError


# ------------------------------------------- Vorgabe: eine Pruefung bricht ab

def test_erfuellte_pruefung_laeuft_durch(run_gb):
    assert run_gb('ASSERT(1 < 2)\nPRINT "weiter"') == "weiter\n"


def test_fehlgeschlagene_pruefung_bricht_ab(run_gb_roh):
    code, out, err = run_gb_roh('PRINT "vorher"\nASSERT(1 > 2, "geht nicht")\nPRINT "nachher"')
    assert code != 0
    assert out == "vorher\n"
    assert "nachher" not in out
    assert "geht nicht" in err


def test_abbruch_nennt_datei_und_zeile(run_gb_roh):
    """Im Abbruch-Modus kommt die Fundstelle aus dem gewohnten
    Laufzeitfehler-Pfad -- ohne Zusatzaufwand."""
    _, _, err = run_gb_roh('PRINT "a"\nPRINT "b"\nASSERT(FALSE, "hier")')
    assert ":3:" in err and "hier" in err


def test_ohne_meldung_gibt_es_trotzdem_eine(run_gb_roh):
    _, _, err = run_gb_roh('ASSERT(FALSE)')
    assert "Bedingung nicht erfuellt" in err


def test_assert_verlangt_boolean(run_gb):
    """`ASSERT(anzahl)` waere sonst still 'wahr, weil nicht null' -- eine
    Pruefung, die aus Versehen immer durchgeht, ist schlimmer als keine."""
    with pytest.raises(DHRuntimeError, match="erwartet BOOLEAN"):
        run_gb('ASSERT(5)')


def test_assert_eq_bricht_ab_und_zeigt_beide_werte(run_gb_roh):
    _, _, err = run_gb_roh('ASSERT_EQ(2 + 2, 5)')
    assert "erhalten 4" in err and "erwartet 5" in err


# ------------------------------------------------------------ Sammel-Modus

def test_sammeln_laeuft_weiter_und_zaehlt(run_gb_roh):
    code, out, err = run_gb_roh('ASSERT_COLLECT(TRUE)\n'
                                'ASSERT_EQ(1, 1)\n'
                                'ASSERT_EQ(1, 2)\n'
                                'ASSERT_EQ(3, 3)\n'
                                'PRINT ASSERT_COUNT()\n'
                                'PRINT ASSERT_FAILED()')
    assert code == 0
    assert out.split() == ["3", "1"]
    assert "FEHL" in err


def test_sammeln_meldet_die_zeile(run_gb_roh):
    _, _, err = run_gb_roh('ASSERT_COLLECT(TRUE)\n'
                           'ASSERT_EQ(1, 1)\n'
                           'ASSERT_EQ(1, 2, "zweite")')
    assert "Zeile 3" in err and "zweite" in err


def test_fehler_gehen_nach_stderr_nutzdaten_bleiben_sauber(run_gb_roh):
    """Ein Pruefprogramm soll sich umleiten lassen: `... > bericht.txt` darf
    keine Fehlerzeilen in den Nutzdaten haben."""
    code, out, err = run_gb_roh('ASSERT_COLLECT(TRUE)\n'
                                'PRINT "nutzdaten"\n'
                                'ASSERT_EQ(1, 2, "kaputt")')
    assert out == "nutzdaten\n"
    assert "kaputt" in err


def test_bilanz_gruen(run_gb_roh):
    code, out, _ = run_gb_roh('ASSERT_COLLECT(TRUE)\n'
                              'ASSERT_EQ(1, 1)\nASSERT_EQ(2, 2)\n'
                              'PRINT ASSERT_REPORT()')
    assert code == 0
    assert out == "ALLES GRUEN -- 2 Pruefungen\n0\n"


def test_bilanz_rot(run_gb_roh):
    _, out, _ = run_gb_roh('ASSERT_COLLECT(TRUE)\n'
                           'ASSERT_EQ(1, 1)\nASSERT_EQ(1, 2)\n'
                           'PRINT ASSERT_REPORT()')
    assert out == "FEHLER: 1 von 2 Pruefungen\n1\n"


def test_das_ganze_muster_eines_pruefprogramms(run_gb_roh):
    """Genau die Form, die ein Pruefprogramm haben soll -- inklusive
    Rueckgabewert, den ein Skript auswerten kann."""
    quelle = ('ASSERT_COLLECT(TRUE)\n'
              'ASSERT_EQ(2 + 2, 4, "Addition")\n'
              'ASSERT_EQ(2 * 3, 7, "Multiplikation")\n'
              'IF ASSERT_REPORT() > 0 THEN\n'
              '    EXIT(1)\n'
              'END IF\n')
    code, out, err = run_gb_roh(quelle)
    assert code == 1                       # <- das ging vorher gar nicht
    assert "FEHLER: 1 von 2" in out
    assert "Multiplikation" in err


def test_sammeln_laesst_sich_wieder_ausschalten(run_gb_roh):
    code, _, _ = run_gb_roh('ASSERT_COLLECT(TRUE)\n'
                            'ASSERT_EQ(1, 2)\n'
                            'ASSERT_COLLECT(FALSE)\n'
                            'ASSERT_EQ(3, 4)\n'
                            'PRINT "nie"')
    assert code != 0


def test_assert_collect_verlangt_boolean(run_gb):
    with pytest.raises(DHRuntimeError, match="erwartet BOOLEAN"):
        run_gb('ASSERT_COLLECT(1)')


# --------------------------------------------------- ASSERT_EQ und Typen

def test_assert_eq_vergleicht_wie_der_gleichheitsoperator(run_gb_roh):
    """Dieselbe Gleichheit wie `=` -- eine zweite Vorstellung davon, wann zwei
    Werte gleich sind, waere die sicherste Art, Vertrauen zu verspielen."""
    code, out, _ = run_gb_roh('ASSERT_COLLECT(TRUE)\n'
                              'ASSERT_EQ(1, 1.0)\n'          # cross-numerisch, wie `=`
                              'ASSERT_EQ("a", "a")\n'
                              'ASSERT_EQ(TRUE, TRUE)\n'
                              'PRINT ASSERT_FAILED()')
    assert out.strip().endswith("0")


def test_assert_eq_auf_strings_zeigt_beide(run_gb_roh):
    _, _, err = run_gb_roh('ASSERT_COLLECT(TRUE)\nASSERT_EQ("abc", "abd", "Text")')
    assert "Text" in err and "abc" in err and "abd" in err


def test_zaehler_starten_bei_null(run_gb):
    assert run_gb('PRINT ASSERT_COUNT()\nPRINT ASSERT_FAILED()') == "0\n0\n"


def test_bilanz_ohne_pruefungen_ist_gruen(run_gb):
    assert run_gb('PRINT ASSERT_REPORT()') == "ALLES GRUEN -- 0 Pruefungen\n0\n"


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


def test_log_ganz_aus(run_gb_roh, monkeypatch):
    monkeypatch.setenv("DH_LOG", "aus")
    _, _, err = run_gb_roh('LOG_ERROR("nicht mal das")')
    assert "nicht mal das" not in err


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
