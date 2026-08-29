"""Rekursionstiefe: eine Meldung statt eines Absturzes -- auf jeder Plattform.

Die VM rekursiert fuer jeden GB-Funktionsaufruf ueber den NATIVEN Stack
(`exec` -> `run_frame` -> `dispatch` -> `exec`). `MAX_CALL_DEPTH` in vm.rs soll
eine ausufernde Rekursion in einen fangbaren Fehler verwandeln -- stand aber
lange auf 3000, einem Wert, den keine Plattform je erreichte:

    Windows (1 MB Stack per Vorgabe):  Absturz bei 147 Ebenen
    "thread 'main' has overflowed its stack"   -- ohne Programmzeile

Gemessen kostet ein Aufrufrahmen ~6,6 KB, gleich fuer freie Funktion, Methode,
BYREF, FUNCREF und ueberladenen Operator. Windows und macOS bekommen ihre
64 MB ueber `rust/drachenhauch_runtime/build.rs`; Linux gibt dem Hauptthread
8 MB aus RLIMIT_STACK, wohin kein Linker-Flag reicht. Die
kleinste Plattform bestimmt darum die Grenze -- gepruefte 8 MB tragen ~1250
Ebenen, MAX_CALL_DEPTH liegt bei 1000.

Ohne diese Tests war der Zustand jahrelang unbemerkt: eine Schutzgrenze war
eingebaut, aber nie geprueft worden, ob sie ueberhaupt greift.
"""
from __future__ import annotations

import re
from pathlib import Path

import pytest

from drachenhauch.errors import DHRuntimeError

WURZEL = Path(__file__).resolve().parents[1]
_VM_RS = WURZEL / "rust" / "drachenhauch_runtime" / "src" / "vm.rs"
_BUILD_RS = WURZEL / "rust" / "drachenhauch_runtime" / "build.rs"

# Messwerte vom 2026-08-29 (Zwei-Punkte-Messung 1 MB / 8 MB, Windows-Release).
_BYTES_PRO_RAHMEN = 6625
_GRUNDVERBRAUCH = 81_388
# Kleinste Plattform: Linux-Hauptthread, RLIMIT_STACK-Vorgabe.
_KLEINSTER_STACK = 8 * 1024 * 1024


def _max_call_depth() -> int:
    m = re.search(r"const MAX_CALL_DEPTH: u32 = (\d+);", _VM_RS.read_text(encoding="utf-8"))
    assert m, "MAX_CALL_DEPTH nicht in vm.rs gefunden"
    return int(m.group(1))


def test_tiefe_rekursion_laeuft_durch(run_gb):
    """Knapp unter der Grenze muss es normal rechnen -- die Grenze soll
    Endlosschleifen fangen, nicht ernsthafte Rekursion verbieten."""
    tiefe = _max_call_depth() - 100
    src = ("FUNCTION f(n AS INTEGER) AS INTEGER\n"
           "IF n <= 0 THEN RETURN 0\n"
           "RETURN 1 + f(n - 1)\n"
           "END FUNCTION\n"
           f"PRINT f({tiefe})\n")
    assert run_gb(src).strip() == str(tiefe)


def test_endlose_rekursion_meldet_statt_abzustuerzen(run_gb):
    """Der Anfaengerfehler schlechthin -- und genau der Fall, fuer den das
    Lehrbuch ("bis das Programm mit einem Fehler abbricht") eine Meldung
    verspricht. Vorher stuerzte der Prozess ohne Programmbezug ab."""
    src = ("FUNCTION fakultaet(n AS INTEGER) AS INTEGER\n"
           "    RETURN n * fakultaet(n - 1)\n"
           "END FUNCTION\n"
           "PRINT fakultaet(5)\n")
    with pytest.raises(DHRuntimeError, match=r"Maximale Aufruftiefe .* unendliche Rekursion"):
        run_gb(src)


def test_der_fehler_ist_fangbar(run_gb):
    """`exec` beschreibt die Grenze als "fangbaren Fehler" -- ein Absturz war
    es nicht. Jetzt stimmt die Zusage: TRY/CATCH faengt, das Programm laeuft
    weiter."""
    src = ("FUNCTION f(n AS INTEGER) AS INTEGER\n"
           "RETURN f(n + 1)\n"
           "END FUNCTION\n"
           "TRY\n"
           "  PRINT f(1)\n"
           "CATCH e\n"
           '  PRINT "gefangen"\n'
           "END TRY\n"
           'PRINT "weiter"\n')
    assert run_gb(src).split() == ["gefangen", "weiter"]


def test_grenze_passt_zur_kleinsten_plattform():
    """Drift-Schutz fuer den Wert selbst. Wer MAX_CALL_DEPTH erhoeht, ohne die
    Stackgroesse mitzudenken, holt den Absturz auf Linux zurueck -- dort reicht
    kein Linker-Flag an den Hauptthread-Stack heran.

    Die Rechnung ist die aus der Messung: passen so viele Rahmen ueberhaupt in
    den kleinsten Stack, mit Luft fuer Schwankungen?
    """
    traegt = (_KLEINSTER_STACK - _GRUNDVERBRAUCH) / _BYTES_PRO_RAHMEN
    assert _max_call_depth() <= traegt * 0.85, (
        f"MAX_CALL_DEPTH={_max_call_depth()} ist zu hoch: ein 8-MB-Stack traegt "
        f"nur ~{traegt:.0f} Rahmen. Erst die Stackgroesse aller Plattformen "
        f"anheben (auf Linux geht das nur ueber RLIMIT_STACK), dann den Wert.")


def test_build_skript_setzt_den_stack():
    """Windows startet sonst mit 1 MB -- das waren 146 Ebenen."""
    assert _BUILD_RS.exists(), "rust/drachenhauch_runtime/build.rs fehlt"
    txt = _BUILD_RS.read_text(encoding="utf-8")
    assert "/STACK:" in txt and "stack_size" in txt


def test_stack_flag_gilt_nur_fuers_binary():
    """`rustc-link-arg-bins`, NICHT `rustflags`.

    Zuerst stand das Flag als `rustflags` in `.cargo/config.toml` -- und die
    gelten fuer ALLE Artefakte, auch fuer Proc-Macros der Abhaengigkeiten. Auf
    macOS ist das ein harter Fehler, weil ld64 die Option nur fuer Programme
    annimmt; die CI brach beim Linken von `libpaste-*.dylib` ab:

        ld: -stack_size option can only be used when linking a main executable

    Auf Windows fiel es nicht auf (der MSVC-Linker nimmt `/STACK:` auch fuer
    eine DLL und ignoriert es) -- ein gruener Bau auf der Entwicklermaschine
    sagte hier also nichts.
    """
    txt = _BUILD_RS.read_text(encoding="utf-8")
    assert "rustc-link-arg-bins" in txt
    assert not (WURZEL / "rust" / "drachenhauch_runtime" / ".cargo" / "config.toml").exists(), (
        "die alte .cargo/config.toml ist zurueck -- ihre rustflags treffen auch "
        "Proc-Macros und brechen den macOS-Bau")
