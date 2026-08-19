"""Mengen ueber MAP (WP J) -- `SET_ADD` und Verwandte.

Kein eigener Typ: eine Menge IST eine `MAP OF INTEGER`, deren Werte niemanden
interessieren. Gemessen kostete der `STR$()`-Umweg, den man sonst von Hand
geht, 0,35 us je Aufnahme -- es ging hier nie um Tempo, sondern darum, dass
`MAPPUT(m, STR$(x), 1)` die Absicht verdeckt.

Der heikle Punkt ist die ELEMENTART: Schluessel sind immer Zeichenketten,
also fielen `5` und `"5"` sonst auf denselben Eintrag und die Menge haette
danach ein Element statt zwei. Die erste Aufnahme legt die Art fest.
Entwurf: docs/entwurf-set-builtins.md.
"""
import pytest

MENGE = "DIM m AS MAP OF INTEGER\n"


def test_doppelte_aufnahme_zaehlt_einmal(run_gb):
    out = run_gb(MENGE +
                 "SET_ADD(m, 7)\n"
                 "SET_ADD(m, 3)\n"
                 "SET_ADD(m, 7)\n"
                 "PRINT SET_SIZE(m)\n")
    assert out.strip() == "2"


def test_zugehoerigkeit(run_gb):
    out = run_gb(MENGE +
                 "SET_ADD(m, 7)\n"
                 'PRINT STR$(SET_HAS(m, 7)) + "," + STR$(SET_HAS(m, 9))\n')
    assert out.strip() == "TRUE,FALSE"


def test_entfernen_meldet_ob_es_drin_war(run_gb):
    out = run_gb(MENGE +
                 "SET_ADD(m, 7)\n"
                 "PRINT SET_REMOVE(m, 7)\n"
                 "PRINT SET_REMOVE(m, 7)\n"
                 "PRINT SET_SIZE(m)\n")
    assert out.split("\n")[:3] == ["TRUE", "FALSE", "0"]


def test_items_liefern_zahlen_bei_einer_zahlenmenge(run_gb):
    """Die Menge merkt sich die Art -- sonst kaemen hier Zeichenketten
    zurueck, und der Nutzer muesste jedes Element selbst umwandeln."""
    out = run_gb(MENGE +
                 "DIM z AS ARRAY OF INTEGER\n"
                 "SET_ADD(m, 20)\n"
                 "SET_ADD(m, 4)\n"
                 "z = SET_ITEMS(m)\n"
                 "PRINT z[0] + z[1]\n")
    assert out.strip() == "24"


def test_reihenfolge_ist_zugesagt(run_gb):
    """Aufnahme-Reihenfolge, nicht Zufall -- das steht so in der Doku und
    macht Ausgaben reproduzierbar."""
    out = run_gb(MENGE +
                 "DIM z AS ARRAY OF INTEGER\n"
                 "SET_ADD(m, 30)\n"
                 "SET_ADD(m, 10)\n"
                 "SET_ADD(m, 20)\n"
                 "z = SET_ITEMS(m)\n"
                 'PRINT STR$(z[0]) + "," + STR$(z[1]) + "," + STR$(z[2])\n')
    assert out.strip() == "30,10,20"


def test_textmenge(run_gb):
    out = run_gb(MENGE +
                 'SET_ADD(m, "anna")\n'
                 'SET_ADD(m, "bert")\n'
                 'SET_ADD(m, "anna")\n'
                 'PRINT JOIN$(SET_ITEMS(m), ",")\n')
    assert out.strip() == "anna,bert"


def test_gemischte_arten_melden(run_gb):
    """Der Fall, der sonst still danebengeht: `5` und `"5"` fallen auf
    denselben Schluessel."""
    from drachenhauch.errors import DHRuntimeError
    with pytest.raises(DHRuntimeError, match="Elementart"):
        run_gb(MENGE + "SET_ADD(m, 5)\n" + 'SET_ADD(m, "5")\n')


def test_auch_das_nachfragen_prueft_die_art(run_gb):
    """`SET_HAS(zahlen, "5")` ist ein Tippfehler, keine Frage -- stumm FALSE
    zu liefern waere die unfreundlichste Antwort darauf."""
    from drachenhauch.errors import DHRuntimeError
    with pytest.raises(DHRuntimeError, match="Elementart"):
        run_gb(MENGE + "SET_ADD(m, 5)\n" + 'PRINT SET_HAS(m, "5")\n')


def test_nach_clear_darf_die_art_wechseln(run_gb):
    """Mit dem letzten Element geht auch die Elementart -- sonst waere eine
    geleerte Menge fuer immer auf ihre erste Art festgelegt."""
    out = run_gb(MENGE +
                 "SET_ADD(m, 5)\n"
                 "SET_CLEAR(m)\n"
                 'SET_ADD(m, "jetzt text")\n'
                 'PRINT JOIN$(SET_ITEMS(m), ",")\n')
    assert out.strip() == "jetzt text"


def test_leere_menge(run_gb):
    out = run_gb(MENGE +
                 "PRINT SET_SIZE(m)\n"
                 "PRINT SET_HAS(m, 1)\n")
    assert out.split("\n")[:2] == ["0", "FALSE"]


def test_falscher_elementtyp_meldet(run_gb):
    from drachenhauch.errors import DHRuntimeError
    with pytest.raises(DHRuntimeError, match="INTEGER oder STRING"):
        run_gb(MENGE + "SET_ADD(m, 1.5)\n")


def test_die_menge_bleibt_eine_map(run_gb):
    """Kein neuer Typ: MAPSIZE und SET_SIZE sehen dieselbe Sache."""
    out = run_gb(MENGE +
                 "SET_ADD(m, 1)\n"
                 "SET_ADD(m, 2)\n"
                 'PRINT STR$(MAPSIZE(m)) + "," + STR$(SET_SIZE(m))\n')
    assert out.strip() == "2,2"
