"""WP F -- Fehler, die man behandeln kann: FINALLY, ERROR_LINE, ERROR_CODE$
und `THROW code, meldung`.

Der Schwerpunkt liegt auf den Wegen AUS einem TRY heraus. Ein FINALLY, das
beim normalen Durchlauf laeuft, aber bei RETURN uebersprungen wird, waere
schlimmer als keines -- man verliesse sich darauf.
"""
import pytest

from drachenhauch.errors import DHRuntimeError


# ------------------------------------------------------- Die Wege hinaus

def test_normaler_durchlauf(run_gb):
    assert run_gb('TRY\n    PRINT "rumpf"\nFINALLY\n    PRINT "finally"\nEND TRY') \
        == "rumpf\nfinally\n"


def test_nach_gefangenem_fehler(run_gb):
    out = run_gb('TRY\n    THROW "kaputt"\n'
                 'CATCH e\n    PRINT "gefangen: " + e\n'
                 'FINALLY\n    PRINT "finally"\nEND TRY')
    assert out == "gefangen: kaputt\nfinally\n"


def test_ohne_catch_laeuft_finally_und_der_fehler_geht_weiter(run_gb):
    """`TRY ... FINALLY ... END TRY` faengt NICHT -- es raeumt nur auf."""
    out = run_gb('TRY\n'
                 '    TRY\n        THROW "durch"\n'
                 '    FINALLY\n        PRINT "finally"\n    END TRY\n'
                 'CATCH e\n    PRINT "aussen: " + e\nEND TRY')
    assert out == "finally\naussen: durch\n"


def test_fehler_im_catch_laesst_finally_trotzdem_laufen(run_gb):
    out = run_gb('TRY\n'
                 '    TRY\n        THROW "erster"\n'
                 '    CATCH e\n        THROW "zweiter"\n'
                 '    FINALLY\n        PRINT "finally"\n    END TRY\n'
                 'CATCH e\n    PRINT "aussen: " + e\nEND TRY')
    assert out == "finally\naussen: zweiter\n"


def test_return_laeuft_nicht_am_finally_vorbei(run_gb):
    """Der wichtigste Fall: ein RETURN mitten im TRY. Ohne die Buchfuehrung im
    Compiler liefe es am FINALLY vorbei -- und genau dafuer schreibt man es."""
    out = run_gb('FUNCTION f() AS INTEGER\n'
                 '    TRY\n        RETURN 42\n'
                 '    FINALLY\n        PRINT "finally"\n    END TRY\n'
                 'END FUNCTION\n'
                 'PRINT f()')
    assert out == "finally\n42\n"


def test_return_ohne_wert_in_einer_sub(run_gb):
    out = run_gb('SUB s()\n'
                 '    TRY\n        RETURN\n'
                 '    FINALLY\n        PRINT "finally"\n    END TRY\n'
                 'END SUB\n'
                 's()\nPRINT "danach"')
    assert out == "finally\ndanach\n"


def test_der_rueckgabewert_wird_vom_finally_nicht_veraendert(run_gb):
    """Der Wert wird VOR dem FINALLY berechnet -- was danach mit der Variablen
    passiert, geht den Rueckgabewert nichts mehr an."""
    out = run_gb('DIM x AS INTEGER\n'
                 'FUNCTION f() AS INTEGER\n'
                 '    TRY\n        x = 1\n        RETURN x\n'
                 '    FINALLY\n        x = 99\n    END TRY\n'
                 'END FUNCTION\n'
                 'PRINT f()\nPRINT x')
    assert out == "1\n99\n"


def test_break_und_continue(run_gb):
    out = run_gb('DIM i AS INTEGER\n'
                 'FOR i = 1 TO 3\n'
                 '    TRY\n'
                 '        IF i = 2 THEN CONTINUE\n'
                 '        IF i = 3 THEN BREAK\n'
                 '        PRINT "rumpf " + STR$(i)\n'
                 '    FINALLY\n'
                 '        PRINT "  finally " + STR$(i)\n'
                 '    END TRY\n'
                 'NEXT\n'
                 'PRINT "fertig"')
    assert out == ("rumpf 1\n  finally 1\n"
                   "  finally 2\n"
                   "  finally 3\n"
                   "fertig\n")


def test_verschachtelt_von_innen_nach_aussen(run_gb):
    out = run_gb('FUNCTION f() AS STRING\n'
                 '    TRY\n'
                 '        TRY\n            RETURN "raus"\n'
                 '        FINALLY\n            PRINT "innen"\n        END TRY\n'
                 '    FINALLY\n        PRINT "aussen"\n    END TRY\n'
                 'END FUNCTION\n'
                 'PRINT f()')
    # Reihenfolge zaehlt: das innere FINALLY zuerst.
    assert out == "innen\naussen\nraus\n"


def test_fehler_im_finally_selbst_geht_nach_aussen(run_gb):
    out = run_gb('TRY\n'
                 '    TRY\n        PRINT "rumpf"\n'
                 '    FINALLY\n        THROW "im finally"\n    END TRY\n'
                 'CATCH e\n    PRINT "aussen: " + e\nEND TRY')
    assert out == "rumpf\naussen: im finally\n"


def test_finally_laeuft_auch_wenn_niemand_faengt(run_gb_roh):
    code, out, err = run_gb_roh('TRY\n    THROW "ungefangen"\n'
                                'FINALLY\n    PRINT "aufgeraeumt"\nEND TRY')
    assert code != 0
    assert out == "aufgeraeumt\n"
    assert "ungefangen" in err


# ------------------------------------------------- Bestehendes Verhalten

def test_try_catch_ohne_finally_unveraendert(run_gb):
    assert run_gb('TRY\n    THROW "x"\nCATCH e\n    PRINT "ok: " + e\nEND TRY') == "ok: x\n"


def test_catch_ohne_namen_unveraendert(run_gb):
    assert run_gb('TRY\n    THROW "x"\nCATCH\n    PRINT "gefangen"\nEND TRY') == "gefangen\n"


def test_try_ganz_ohne_catch_und_finally_schluckt_weiterhin(run_gb):
    """Altes Verhalten, bewusst unveraendert: `TRY ... END TRY` ohne beides
    verschluckt den Fehler. Das zu aendern braeche bestehenden Code."""
    assert run_gb('TRY\n    THROW "weg damit"\nEND TRY\nPRINT "weiter"') == "weiter\n"


# ------------------------------------------------- ERROR_CODE$ / ERROR_LINE

def test_throw_mit_code(run_gb):
    out = run_gb('TRY\n    THROW "DB_WEG", "Datenbank nicht erreichbar"\n'
                 'CATCH e\n    PRINT e\n    PRINT ERROR_CODE$()\nEND TRY')
    assert out == "Datenbank nicht erreichbar\nDB_WEG\n"


def test_code_erlaubt_entscheiden_ohne_texte_zu_vergleichen(run_gb):
    out = run_gb('DIM i AS INTEGER\n'
                 'FOR i = 1 TO 2\n'
                 '    TRY\n'
                 '        IF i = 1 THEN THROW "NETZ", "keine Verbindung"\n'
                 '        THROW "DATEI", "nicht gefunden"\n'
                 '    CATCH e\n'
                 '        SELECT CASE ERROR_CODE$()\n'
                 '            CASE "NETZ"\n                PRINT "nochmal versuchen"\n'
                 '            CASE "DATEI"\n                PRINT "Vorgabe benutzen"\n'
                 '        END SELECT\n'
                 '    END TRY\n'
                 'NEXT')
    assert out == "nochmal versuchen\nVorgabe benutzen\n"


def test_eingebauter_fehler_hat_keinen_code(run_gb):
    """Sonst klebte der Code eines frueheren THROW an einem fremden Fehler."""
    out = run_gb('TRY\n    THROW "ALT", "erster"\nCATCH e\n    PRINT ERROR_CODE$()\nEND TRY\n'
                 'TRY\n    PRINT 1 / 0\nCATCH e\n    PRINT "[" + ERROR_CODE$() + "]"\nEND TRY')
    assert out == "ALT\n[]\n"


def test_throw_ohne_code_hat_leeren_code(run_gb):
    assert run_gb('TRY\n    THROW "nur text"\nCATCH e\n'
                  '    PRINT "[" + ERROR_CODE$() + "]"\nEND TRY') == "[]\n"


def test_error_line_nennt_die_wurfstelle(run_gb):
    out = run_gb('PRINT "eins"\n'
                 'TRY\n'
                 '    THROW "hier"\n'
                 'CATCH e\n'
                 '    PRINT ERROR_LINE()\n'
                 'END TRY')
    assert out == "eins\n3\n"


def test_error_line_bei_einem_eingebauten_fehler(run_gb):
    out = run_gb('TRY\n'
                 '    PRINT 1 / 0\n'
                 'CATCH e\n'
                 '    PRINT ERROR_LINE()\n'
                 'END TRY')
    assert out == "2\n"


def test_error_line_zeigt_in_die_gerufene_funktion(run_gb):
    out = run_gb('SUB tief()\n'
                 '    THROW "unten"\n'
                 'END SUB\n'
                 'TRY\n'
                 '    tief()\n'
                 'CATCH e\n'
                 '    PRINT ERROR_LINE()\n'
                 'END TRY')
    assert out == "2\n"


def test_ohne_fehler_sind_die_angaben_leer(run_gb):
    assert run_gb('PRINT ERROR_LINE()\nPRINT "[" + ERROR_CODE$() + "]"') == "0\n[]\n"


def test_throw_mit_code_auch_als_einzeiler(run_gb):
    out = run_gb('TRY\n    IF TRUE THEN THROW "C", "m"\n'
                 'CATCH e\n    PRINT ERROR_CODE$() + "/" + e\nEND TRY')
    assert out == "C/m\n"


def test_code_ueberlebt_ein_finally(run_gb):
    out = run_gb('TRY\n'
                 '    TRY\n        THROW "CODE", "text"\n'
                 '    FINALLY\n        PRINT "aufgeraeumt"\n    END TRY\n'
                 'CATCH e\n    PRINT ERROR_CODE$()\nEND TRY')
    assert out == "aufgeraeumt\nCODE\n"


# ------------------------------------------------------ Der eigentliche Zweck

def test_datei_wird_auch_bei_einem_fehler_geschlossen(run_gb, tmp_path):
    """Wofuer FINALLY da ist: aufraeumen, egal wie man herauskommt."""
    out = run_gb('DIM f AS FILE\n'
                 'WRITEALL("d.txt", "inhalt")\n'
                 'TRY\n'
                 '    TRY\n'
                 '        f = OPENFILE("d.txt", "r")\n'
                 '        THROW "etwas geht schief"\n'
                 '    FINALLY\n'
                 '        CLOSEFILE(f)\n'
                 '        PRINT "Datei geschlossen"\n'
                 '    END TRY\n'
                 'CATCH e\n'
                 '    PRINT "gemeldet: " + e\n'
                 'END TRY', base=tmp_path)
    assert out == "Datei geschlossen\ngemeldet: etwas geht schief\n"
