"""Eingabe aufzeichnen und abspielen (AUTOMATION_*).

raylib kann den kompletten Eingabe-Zustand eines Frames mitschreiben und
spaeter wieder in seinen Eingabe-Zustand einspeisen -- Grundlage fuer
Demo-Modus, nachspielbare Fehlerberichte und automatische Spieltests.

Echte Tastendruecke lassen sich im Test nicht erzeugen. Die WIEDERGABE aber
schon: die Aufnahmedatei ist Text, der Test schreibt sie selbst und prueft
dann, ob das GB-Programm genau die aufgezeichneten Maus-/Tastenwerte sieht --
das ist die Haelfte, auf die es ankommt (dass eingespeiste Eingabe wirklich
bei KEYPRESSED/MOUSEX ankommt).

Zeitliche Zuordnung: eingespeist wird am ENDE eines FLIP (direkt nachdem
raylib die echte Eingabe fuer den naechsten Frame gelesen hat). Ein
Ereignis mit Aufnahme-Frame N wirkt daher im Programm-Durchlauf N+1 -- die
Erwartungen unten sind entsprechend um eins verschoben.

Die uebertragbaren Tests liegen seit 2026-09-08 als Pruefsammlung in
`tests/pruef/automation.dhtest` (dhrt test); hier bleiben nur die, die ein Bild,
eine geschriebene Datei oder den Quelltext mit einem fremden Leser pruefen.
"""
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

# raylibs AutomationEventType-Nummern (rcore.c)
KEY_UP, KEY_DOWN = 1, 2
MOUSE_BUTTON_UP, MOUSE_BUTTON_DOWN, MOUSE_POSITION = 5, 6, 7


def _events(tmp_path, name, events):
    """Aufnahmedatei im raylib-Textformat schreiben (wie ExportAutomationEventList)."""
    lines = ["# Test-Aufnahme", f"c {len(events)}"]
    for frame, typ, *params in events:
        p = (list(params) + [0, 0, 0, 0])[:4]
        lines.append(f"e {frame} {typ} {p[0]} {p[1]} {p[2]} {p[3]} // Event: test")
    path = tmp_path / name
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return path


def _run(src, tmp_path, frames=12):
    (tmp_path / "a.dh").write_text(src, encoding="utf-8")
    env = dict(os.environ, DHRT_FRAMES=str(frames))
    r = subprocess.run([str(_DHRT), "run", str(tmp_path / "a.dh")], capture_output=True,
                       text=True, encoding="utf-8", env=env, timeout=90, cwd=str(tmp_path))
    r.lines = [ln for ln in (r.stdout or "").splitlines()
               if not ln.startswith(("WARNING:", "INFO:", "TRACE:"))]
    return r


# Das Fenster wird aus dem Bild geschoben, BEVOR irgendetwas aufgezeichnet
# oder abgespielt wird.
#
# Grund: raylib schreibt jede Aenderung der Mausposition mit und liefert sie
# auch an ein laufendes Programm -- und ein Fenster geht dort auf, wo der
# Zeiger gerade steht. Liegt er darin, enthaelt schon der erste Frame zwei
# Ereignisse (INPUT_MOUSE_POSITION + INPUT_TOUCH_POSITION), und eine
# eingespeiste Position wird von der echten ueberschrieben. Drei Tests dieser
# Datei haben dadurch gelegentlich versagt -- je nachdem, wo die Maus des
# Rechners gerade lag.
#
# Gemessen: Fenster 1600x1000 (deckt den Zeiger sicher ab) ohne Verschieben
# 2 Ereignisse in 3 von 3 Laeufen, mit Verschieben 0 in 3 von 3.
_HEAD = ('SCREEN(160, 120, "Auto", 1)\n'
         'SET_WINDOW_POS(-3000, -3000)\n')


# ------------------------------------------------------------- Wiedergabe


# -------------------------------------------------------------- Aufnahme
def test_recording_writes_a_readable_file(tmp_path):
    gb = (_HEAD + 'AUTOMATION_RECORD("out.txt")\n'
          'PRINT AUTOMATION_RECORDING()\n'
          'DIM f AS INTEGER\n'
          'FOR f = 0 TO 3\n'
          '    FLIP()\n'
          'NEXT\n'
          'PRINT AUTOMATION_STOP()\n'
          'PRINT AUTOMATION_RECORDING()\n')
    r = _run(gb, tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.lines[0] == "TRUE" and r.lines[2] == "FALSE"
    out = (tmp_path / "out.txt").read_text(encoding="utf-8")
    # Die Datei muss das Format haben, das AUTOMATION_PLAY wieder liest -- und
    # ihre Kopfzeile muss zu dem passen, was AUTOMATION_STOP gemeldet hat.
    #
    # NICHT auf "c 0" pruefen: raylib schreibt die Mausposition mit, sobald sie
    # sich aendert, und das Fenster geht dort auf, wo der Zeiger gerade steht.
    # Liegt er im Fensterbereich, stehen zwei Ereignisse drin
    # (INPUT_MOUSE_POSITION + INPUT_TOUCH_POSITION), sonst keins -- der Test
    # hing damit am Mausstand des Rechners, auf dem er lief.
    assert f"c {r.lines[1]}" in out, (r.lines[1], out[:400])
    assert int(r.lines[1]) >= 0


def test_recorded_file_can_be_played_back(tmp_path):
    """Aufnehmen -> stoppen -> abspielen liefert dieselbe Ereigniszahl zurueck.

    Geprueft wird die BEZIEHUNG, nicht eine feste Zahl: wie viele Ereignisse
    in zwei Frames anfallen, haengt davon ab, ob der Mauszeiger gerade ueber
    dem Testfenster steht -- raylib schreibt jede Positionsaenderung mit, und
    das Fenster geht dort auf, wo der Zeiger nun mal ist. Frueher stand hier
    `== ["0", "FALSE"]`; der Test flackerte damit je nach Mausstand des
    Rechners, auf dem er lief.
    """
    gb = (_HEAD + 'AUTOMATION_RECORD("rt.txt")\n'
          'FLIP()\nFLIP()\n'
          'PRINT AUTOMATION_STOP()\n'
          'PRINT AUTOMATION_PLAY("rt.txt")\n'
          'PRINT AUTOMATION_PLAYING()\n')
    r = _run(gb, tmp_path)
    assert r.returncode == 0, r.stderr
    aufgenommen, abgespielt, laeuft = r.lines[0], r.lines[1], r.lines[2]
    assert abgespielt == aufgenommen, \
        (r.lines, "Wiedergabe meldet eine andere Zahl als die Aufnahme")
    # Eine Wiedergabe laeuft genau dann, wenn es ueberhaupt etwas abzuspielen gab.
    assert laeuft == ("TRUE" if int(aufgenommen) > 0 else "FALSE"), r.lines


# ----------------------------------------------------------------- Fehler


# --------------------------------------------------- Tastencode-Umsetzung
# Die Wiedergabe speist ROHE raylib-Tastenwerte ein -- damit laesst sich
# `map_key` (GB-Code -> raylib-Taste) end-to-end pruefen, ohne dass jemand
# eine Taste druecken muss.

RL_KEY_S, RL_KEY_MINUS, RL_KEY_COMMA, RL_KEY_PERIOD = 83, 45, 44, 46
