"""Die Liste der seriellen Testdateien darf nicht ins Leere zeigen.

`_SERIELL` in conftest.py nennt Dateien beim Namen. Wird eine davon
umbenannt, faellt sie lautlos aus dem seriellen Durchgang heraus und laeuft
wieder zwischen den anderen -- und meldet sich erst Wochen spaeter als
Fehlschlag, der sich beim Nachstellen nicht reproduzieren laesst. Genau die
Sorte Fehler, die man nicht sucht, sondern verhindert.
"""
from pathlib import Path

from .conftest import _SERIELL

HIER = Path(__file__).resolve().parent


def test_jede_serielle_datei_existiert():
    fehlend = sorted(n for n in _SERIELL if not (HIER / n).exists())
    assert not fehlend, (
        f"_SERIELL in conftest.py nennt Dateien, die es nicht gibt: {fehlend} -- "
        "umbenannt? Dann dort nachziehen, sonst laufen sie wieder parallel.")
