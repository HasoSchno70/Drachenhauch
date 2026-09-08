"""WP F -- Fehler, die man behandeln kann: FINALLY, ERROR_LINE, ERROR_CODE$
und `THROW code, meldung`.

Der Schwerpunkt liegt auf den Wegen AUS einem TRY heraus. Ein FINALLY, das
beim normalen Durchlauf laeuft, aber bei RETURN uebersprungen wird, waere
schlimmer als keines -- man verliesse sich darauf.

Die uebertragbaren Tests liegen seit 2026-09-08 als Pruefsammlung in
`tests/pruef/finally.dhtest` (dhrt test); hier bleiben nur die, die ein Bild,
eine geschriebene Datei oder den Quelltext mit einem fremden Leser pruefen.
"""
import pytest

from drachenhauch.errors import DHRuntimeError


# ------------------------------------------------------- Die Wege hinaus


def test_finally_laeuft_auch_wenn_niemand_faengt(run_gb_roh):
    code, out, err = run_gb_roh('TRY\n    THROW "ungefangen"\n'
                                'FINALLY\n    PRINT "aufgeraeumt"\nEND TRY')
    assert code != 0
    assert out == "aufgeraeumt\n"
    assert "ungefangen" in err


# ------------------------------------------------- Bestehendes Verhalten


# ------------------------------------------------- ERROR_CODE$ / ERROR_LINE


# ------------------------------------------------------ Der eigentliche Zweck
