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
def test_recorded_keys_and_mouse_reach_the_program(tmp_path):
    # Eine gehaltene Taste steht in JEDEM Frame in der Aufnahme (so schreibt
    # raylib mit) -- losgelassen wird sie mit einem eigenen KEY_UP.
    _events(tmp_path, "ev.txt", [
        (0, MOUSE_POSITION, 40, 25),
        (1, KEY_DOWN, 32),
        (2, KEY_DOWN, 32),
        (3, KEY_UP, 32),
    ])
    gb = (_HEAD + 'AUTOMATION_PLAY("ev.txt")\n'
          'DIM f AS INTEGER\n'
          'FOR f = 0 TO 5\n'
          '    PRINT STR$(f) + " " + STR$(MOUSEX()) + "," + STR$(MOUSEY()) + " " + _\n'
          '          STR$(KEYPRESSED(KEY_SPACE)) + " " + STR$(KEYHIT(KEY_SPACE))\n'
          '    FLIP()\n'
          'NEXT\n')
    r = _run(gb, tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.lines == [
        "0 0,0 FALSE FALSE",        # noch nichts eingespeist
        "1 40,25 FALSE FALSE",      # Mausposition aus Frame 0
        "2 40,25 TRUE TRUE",        # Taste gedrueckt -> auch die Flanke
        "3 40,25 TRUE FALSE",       # gehalten: keine neue Flanke
        "4 40,25 FALSE FALSE",      # losgelassen
        "5 40,25 FALSE FALSE",
    ]


def test_mouse_buttons_are_replayed(tmp_path):
    _events(tmp_path, "click.txt", [
        (0, MOUSE_BUTTON_DOWN, 0),
        (1, MOUSE_BUTTON_UP, 0),
    ])
    gb = (_HEAD + 'AUTOMATION_PLAY("click.txt")\n'
          'DIM f AS INTEGER\n'
          'FOR f = 0 TO 3\n'
          '    PRINT STR$(MOUSEBUTTON(0)) + " " + STR$(MOUSE_HIT(0))\n'
          '    FLIP()\n'
          'NEXT\n')
    r = _run(gb, tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.lines == ["FALSE FALSE", "TRUE TRUE", "FALSE FALSE", "FALSE FALSE"]


def test_playback_reports_state_and_ends_by_itself(tmp_path):
    _events(tmp_path, "ev.txt", [(0, KEY_DOWN, 32), (1, KEY_UP, 32)])
    gb = (_HEAD + 'PRINT AUTOMATION_PLAY("ev.txt")\n'
          'PRINT AUTOMATION_COUNT()\n'
          'PRINT AUTOMATION_PLAYING()\n'
          'DIM f AS INTEGER\n'
          'FOR f = 0 TO 3\n'
          '    FLIP()\n'
          'NEXT\n'
          'PRINT AUTOMATION_PLAYING()\n'
          'PRINT AUTOMATION_FRAME()\n')
    r = _run(gb, tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.lines[:3] == ["2", "2", "TRUE"]
    assert r.lines[3] == "FALSE", "Wiedergabe muss nach dem letzten Ereignis enden"
    assert int(r.lines[4]) >= 2


def test_far_away_events_wait_instead_of_being_skipped(tmp_path):
    # Die Wiedergabe zaehlt Frame fuer Frame hoch: ein Ereignis weit hinten in
    # der Aufnahme bleibt liegen, bis sein Frame dran ist -- es wird weder
    # vorgezogen noch verworfen (die Wiedergabe endet also auch nicht zu frueh).
    _events(tmp_path, "sparse.txt", [(0, MOUSE_POSITION, 10, 10), (900, MOUSE_POSITION, 99, 88)])
    gb = (_HEAD + 'AUTOMATION_PLAY("sparse.txt")\n'
          'DIM f AS INTEGER\n'
          'FOR f = 0 TO 2\n'
          '    FLIP()\n'
          'NEXT\n'
          'PRINT STR$(MOUSEX()) + "," + STR$(MOUSEY())\n'
          'PRINT AUTOMATION_PLAYING()\n')
    r = _run(gb, tmp_path)
    assert r.returncode == 0, r.stderr
    assert r.lines == ["10,10", "TRUE"]
    # Die zweite Zusage ausdruecklich, weil sie den Namen des Tests traegt:
    # das Ereignis bei Frame 900 darf weder vorgezogen noch verworfen werden.
    assert r.lines[0] != "99,88", "Ereignis aus Frame 900 wurde vorgezogen"


def test_injected_keys_do_not_count_as_user_input(tmp_path):
    # Der Attract-Modus ist der Haupt-Anwendungsfall: die Demo laeuft, bis der
    # Spieler eine Taste drueckt. raylib legt eingespeiste Tasten aber AUCH in
    # seine "zuletzt gedrueckt"-Warteschlange -- ohne Filter meldete
    # KEY_ANY_HIT die Demo-Tasten als Nutzereingabe und die Demo brach sofort
    # an sich selbst ab. KEYHIT muss sie weiterhin sehen (darum geht es ja).
    _events(tmp_path, "demo.txt", [(0, KEY_DOWN, 32), (2, KEY_UP, 32)])
    gb = (_HEAD + 'AUTOMATION_PLAY("demo.txt")\n'
          'DIM f AS INTEGER\n'
          'FOR f = 0 TO 3\n'
          '    PRINT STR$(KEY_ANY_HIT()) + " " + STR$(KEYHIT(KEY_SPACE))\n'
          '    FLIP()\n'
          'NEXT\n')
    r = _run(gb, tmp_path)
    assert r.returncode == 0, r.stderr
    assert [ln.split()[0] for ln in r.lines] == ["-1"] * 4, \
        "KEY_ANY_HIT darf keine eingespeiste Taste melden"
    assert "TRUE" in r.lines[1], "KEYHIT muss die eingespeiste Taste sehr wohl sehen"


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
def test_missing_file_is_reported(tmp_path):
    r = _run(_HEAD + 'AUTOMATION_PLAY("gibtsnicht.txt")\n', tmp_path)
    assert r.returncode != 0
    assert "AUTOMATION_PLAY" in r.stderr and "gibtsnicht" in r.stderr


def test_recording_blocks_playback(tmp_path):
    # raylib spielt waehrend einer Aufnahme grundsaetzlich nichts ab -- das
    # still zu schlucken waere die schlechtere Antwort als eine klare Meldung.
    _events(tmp_path, "ev.txt", [(0, KEY_DOWN, 32)])
    r = _run(_HEAD + 'AUTOMATION_RECORD("x.txt")\nAUTOMATION_PLAY("ev.txt")\n', tmp_path)
    assert r.returncode != 0
    assert "AUTOMATION_STOP" in r.stderr


def test_empty_filename_is_rejected(tmp_path):
    r = _run(_HEAD + 'AUTOMATION_RECORD("")\n', tmp_path)
    assert r.returncode != 0 and "AUTOMATION_RECORD" in r.stderr


# --------------------------------------------------- Tastencode-Umsetzung
# Die Wiedergabe speist ROHE raylib-Tastenwerte ein -- damit laesst sich
# `map_key` (GB-Code -> raylib-Taste) end-to-end pruefen, ohne dass jemand
# eine Taste druecken muss.

RL_KEY_S, RL_KEY_MINUS, RL_KEY_COMMA, RL_KEY_PERIOD = 83, 45, 44, 46


def test_buchstaben_treffen_in_beiden_schreibweisen(tmp_path):
    """Eine Taste, zwei Schreibweisen: `ASC("s")` und `ASC("S")`.

    GB-Tastencodes folgen SDL, dort sind Buchstaben KLEIN (97..122). Bis
    2026-08-31 galten deshalb NUR die kleinen, und `KEYHIT(ASC("S"))` (= 83)
    traf still gar nichts -- kein Fehler, keine Warnung, die Taste existierte
    fuer das Programm einfach nicht.

    Dieser Test hielt frueher genau das fest, mit derselben Beschreibung, die
    es einen "Fehler, der beim Schreiben nicht auffaellt" nannte. Genau als
    solcher trat er dann auch auf: im Tilemap-Editor
    (`examples/187_tilemap_editor.dh`) war JEDES Tastenkuerzel wirkungslos,
    und weil ein totes Kuerzel wie ein vergessener Aufruf aussieht, sucht man
    den Fehler im eigenen Programm. Seitdem nimmt die Umsetzungstabelle beide
    Bereiche an -- 65..90 lief vorher ohnehin ins Leere, das Annehmen kann
    also nichts brechen.
    """
    _events(tmp_path, "ev.txt", [(1, KEY_DOWN, RL_KEY_S)])
    gb = (_HEAD + 'AUTOMATION_PLAY("ev.txt")\n'
          'DIM f AS INTEGER\n'
          'FOR f = 0 TO 3\n'
          '    PRINT STR$(KEYPRESSED(ASC("s"))) + " " + STR$(KEYPRESSED(ASC("S")))\n'
          '    FLIP()\n'
          'NEXT\n')
    r = _run(gb, tmp_path)
    assert r.returncode == 0, r.stderr
    # Beide meinen dieselbe Taste -- sie sind IMMER gleich, nie nur eine.
    assert "TRUE TRUE" in r.lines, r.lines
    assert not any(ln in ("TRUE FALSE", "FALSE TRUE") for ln in r.lines), r.lines


def test_satzzeichen_sind_ansprechbar(tmp_path):
    """Regression: Satzzeichen fehlten in der Umsetzungstabelle ganz --
    `KEYHIT(ASC("-"))` lief ins Leere, ohne Fehlermeldung."""
    _events(tmp_path, "ev.txt", [
        (1, KEY_DOWN, RL_KEY_MINUS),
        (1, KEY_DOWN, RL_KEY_COMMA),
        (1, KEY_DOWN, RL_KEY_PERIOD),
    ])
    gb = (_HEAD + 'AUTOMATION_PLAY("ev.txt")\n'
          'DIM f AS INTEGER\n'
          'FOR f = 0 TO 3\n'
          '    PRINT STR$(KEYPRESSED(ASC("-"))) + " " + STR$(KEYPRESSED(ASC(","))) + _\n'
          '          " " + STR$(KEYPRESSED(ASC(".")))\n'
          '    FLIP()\n'
          'NEXT\n')
    r = _run(gb, tmp_path)
    assert r.returncode == 0, r.stderr
    assert "TRUE TRUE TRUE" in r.lines, r.lines
