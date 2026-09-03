"""`dhrt --check` meldet Namen, die nirgends deklariert sind.

In Drachenhauch ist `DIM` Pflicht, und ein unbekannter Name bricht zur
Laufzeit ab ("Variable 'x' nicht deklariert"). Nur passiert das erst, wenn
die ZEILE laeuft -- in einem Programm von einigen tausend Zeilen bleibt ein
Tippfehler in einem selten genommenen Zweig damit beliebig lange still.
Gefunden wurde die Luecke 2026-09-03 an einer SUB des Sprite-Editors, die
eine Variable aus einem SCHWESTER-Programm ansprach; `--check` schwieg.

Der zweite Teil dieser Datei ist der wichtigere: die Faelle, in denen NICHT
gewarnt werden darf. Eine Pruefung, die bei richtigem Code anschlaegt,
schaltet man ab -- und dann hat man gar keine.
"""
import json
import os
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


def _find_dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    return next((_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
                 for v in ("release", "debug")
                 if (_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe).exists()), None)


_DHRT = _find_dhrt()
pytestmark = pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")

_MARKE = "nirgends im Programm mit DIM oder CONST angelegt"


def _befunde(src, tmp_path):
    (tmp_path / "s.dh").write_text(src, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "--check", str(tmp_path / "s.dh")],
                       capture_output=True, text=True, encoding="utf-8",
                       errors="replace", timeout=90, cwd=str(tmp_path))
    assert r.returncode == 0, r.stderr
    return [b for b in json.loads(r.stdout or "[]") if _MARKE in b.get("message", "")]


# ------------------------------------------------------------------ Treffer

def test_tippfehler_beim_schreiben(tmp_path):
    b = _befunde('DIM zaehler AS INTEGER\nzaehlr = 1\n', tmp_path)
    assert len(b) == 1
    assert b[0]["line"] == 2 and b[0]["severity"] == "warning"
    assert "Meintest du 'zaehler'?" in b[0]["message"]


def test_tippfehler_beim_lesen(tmp_path):
    b = _befunde('DIM punkte AS INTEGER\nPRINT punke\n', tmp_path)
    assert len(b) == 1 and "Meintest du 'punkte'?" in b[0]["message"]


def test_auch_tief_in_einer_sub(tmp_path):
    """Der eigentliche Anlass: eine SUB, die nie aufgerufen wird. Vorher
    meldete sich das NIE -- weder beim Pruefen noch beim Laufen."""
    b = _befunde('SUB tuwas()\n    fehlt = TRUE\nEND SUB\nPRINT "start"\n', tmp_path)
    assert len(b) == 1 and b[0]["line"] == 2


def test_jeder_name_nur_einmal(tmp_path):
    """Ein falsch geschriebener Name in einer Schleife soll nicht zwanzig
    Zeilen Ausgabe erzeugen."""
    src = 'DIM i AS INTEGER\nFOR i = 0 TO 9\n    PRINT summ\nNEXT\n'
    assert len(_befunde(src, tmp_path)) == 1


def test_die_warnung_blockiert_nicht(tmp_path):
    """Sie ist eine WARNUNG. Ein Programm mit einem Tippfehler in einem
    Zweig, der nie genommen wird, laeuft weiter -- das war schon immer so
    und soll sich nicht mit einer Pruefung aendern."""
    (tmp_path / "s.dh").write_text('SUB tuwas()\n    fehlt = 1\nEND SUB\nPRINT "ok"\n',
                                   encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(tmp_path / "s.dh")], capture_output=True,
                       text=True, encoding="utf-8", timeout=90, cwd=str(tmp_path))
    assert r.returncode == 0 and "ok" in r.stdout


# ------------------------------------------------- und wo sie schweigen muss

def test_vorbelegte_konstanten_sind_keine_befunde(tmp_path):
    """Farben, Tasten, PI und TAU bekommen absichtlich keinen Slot und laufen
    ueber genau denselben Weg wie ein unbekannter Name. Ohne die Ausnahme
    waere jedes `KEYHIT(KEY_SPACE)` ein Befund -- und die Pruefung wertlos."""
    src = ('DIM f AS INTEGER\nf = RED\nPRINT PI + TAU\n'
           'IF KEYHIT(KEY_SPACE) OR KEYHIT(KEY_LSHIFT) THEN PRINT WHITE\n')
    assert _befunde(src, tmp_path) == []


def test_dim_mit_groesse_zaehlt_als_deklaration(tmp_path):
    """`DIM x[N] AS T` laeuft im Compiler ueber einen eigenen Zweig. Genau
    den hat die erste Fassung uebersehen -- das ergab 243 Falschmeldungen
    ueber die Beispiele, und zwar in ausgerechnet dem Fall, den ein
    Spielprogramm am haeufigsten benutzt."""
    src = ('CONST N = 4\nDIM feld[N] AS INTEGER\nDIM gitter[N, N] AS STRING\n'
           'feld[0] = 1\ngitter[0, 0] = "x"\nPRINT feld[0]; gitter[0, 0]\n')
    assert _befunde(src, tmp_path) == []


def test_alle_dim_formen_zaehlen(tmp_path):
    src = ('DIM a AS ARRAY OF INTEGER\nDIM m AS MAP OF STRING\n'
           'DIM t AS TUPLE\nDIM s AS STRING\n'
           'a = [1, 2]\nMAPPUT(m, "k", "v")\nt = (1, 2)\ns = "x"\n'
           'PRINT LEN(a); MAPGET(m, "k"); s\n')
    assert _befunde(src, tmp_path) == []


def test_parameter_und_lokale_zaehlen(tmp_path):
    src = ('FUNCTION doppelt(wert AS INTEGER) AS INTEGER\n'
           '    DIM h AS INTEGER : h = wert * 2\n    RETURN h\n'
           'END FUNCTION\nPRINT doppelt(3)\n')
    assert _befunde(src, tmp_path) == []


def test_schleifen_und_catch_variablen_zaehlen(tmp_path):
    src = ('FOR i = 0 TO 2\n    PRINT i\nNEXT\n'
           'FOR EACH w IN (1, 2)\n    PRINT w\nNEXT\n'
           'TRY\n    THROW "x"\nCATCH e\n    PRINT e\nEND TRY\n')
    assert _befunde(src, tmp_path) == []


def test_klassen_felder_und_self_zaehlen(tmp_path):
    src = ('CLASS Held\n    DIM hp AS INTEGER\n'
           '    SUB Init()\n        Self.hp = 10\n    END SUB\n'
           '    FUNCTION lebt() AS BOOLEAN\n        RETURN hp > 0\n    END FUNCTION\n'
           'END CLASS\nDIM h AS Held\nh = NEW Held()\nPRINT h.lebt()\n')
    assert _befunde(src, tmp_path) == []


def test_ein_enum_und_seine_mitglieder_zaehlen(tmp_path):
    src = ('ENUM Zustand = RUHT, LAEUFT\nDIM z AS Zustand\nz = Zustand.LAEUFT\n'
           'PRINT z\n')
    assert _befunde(src, tmp_path) == []


def test_eine_funktion_als_funcref_ist_kein_befund(tmp_path):
    """Ein blosser Funktionsname in Ausdrucks-Position ist eine FUNCREF --
    er sieht fuer den Compiler aus wie eine unbekannte Variable."""
    src = ('FUNCTION quadrat(x AS INTEGER) AS INTEGER\n    RETURN x * x\nEND FUNCTION\n'
           'DIM f AS FUNCREF\nf = quadrat\nPRINT f(4)\n')
    assert _befunde(src, tmp_path) == []


def test_ein_spaeteres_dim_zaehlt_auch(tmp_path):
    """Geprueft wird ERST am Ende. Waehrend des Uebersetzens weiss niemand,
    ob der Name weiter unten noch deklariert wird -- und in einer SUB, die
    oben steht, ist genau das der Normalfall."""
    src = ('SUB zeig()\n    PRINT spaeter\nEND SUB\n'
           'DIM spaeter AS STRING\nspaeter = "da"\nzeig()\n')
    assert _befunde(src, tmp_path) == []


def test_ein_name_aus_einer_anderen_funktion_wird_uebersehen(tmp_path):
    """Die bewusste Luecke, festgehalten damit sie nicht fuer einen Fehler
    gehalten wird: geprueft wird gegen alle Namen im GANZEN Programm, nicht
    gegen den Gueltigkeitsbereich. `hilf` ist in `b` nicht sichtbar und
    braeche dort ab -- gemeldet wird es trotzdem nicht.

    Das ist der Preis dafuer, dass die Pruefung bei richtigem Code schweigt.
    Wer den Gueltigkeitsbereich mitpruefen will, muss zuerst belegen, dass
    dabei keine Falschmeldung entsteht.
    """
    src = ('SUB a()\n    DIM hilf AS INTEGER : hilf = 1\n    PRINT hilf\nEND SUB\n'
           'SUB b()\n    PRINT hilf\nEND SUB\na()\n')
    assert _befunde(src, tmp_path) == []
