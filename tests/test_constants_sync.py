"""Drift-Schutz fuer die doppelt gepflegten Konstanten COLORS/KEYS.

Die Farb- und Tasten-Konstanten existieren zweimal: in
`gamebasic/graphics.py` (COLORS/KEYS, Editor-/LSP-Tooling) und als
vorregistrierte Globals der Runtime (`DEFAULT_COLORS`/`DEFAULT_KEYS` in
`rust/drachenhauch_runtime/src/vm.rs`) -- von Hand synchron gehalten, ein Erbe der
pygame-Zeit (die Werte sind SDL-Keycodes und damit Sprach-API).

Dieser Golden-Test laesst dhrt JEDE Python-seitige Konstante PRINTen und
vergleicht die Werte. Driftet eine Seite (Konstante fehlt in dhrt oder hat
einen anderen Wert), schlaegt er mit dem Namen fehl. Die Gegenrichtung
(zusaetzliche Konstanten nur in dhrt) deckt er bewusst nicht ab -- Python
ist die dokumentierte Quelle.
"""
from gamebasic.graphics import COLORS, KEYS


def test_colors_match_runtime(run_gb):
    names = list(COLORS)
    src = "\n".join(f"PRINT {n.upper()}" for n in names)
    out = [l.strip() for l in run_gb(src).split("\n") if l.strip()]
    assert len(out) == len(names)
    for name, got in zip(names, out):
        assert got == str(COLORS[name]), (
            f"Farbe {name.upper()}: graphics.py={COLORS[name]}, dhrt={got} "
            f"-- vm.rs DEFAULT_COLORS und graphics.py COLORS synchron halten!")


def test_keys_match_runtime(run_gb):
    names = list(KEYS)
    src = "\n".join(f"PRINT {n.upper()}" for n in names)
    out = [l.strip() for l in run_gb(src).split("\n") if l.strip()]
    assert len(out) == len(names)
    for name, got in zip(names, out):
        assert got == str(KEYS[name]), (
            f"Taste {name.upper()}: graphics.py={KEYS[name]}, dhrt={got} "
            f"-- vm.rs DEFAULT_KEYS und graphics.py KEYS synchron halten!")
