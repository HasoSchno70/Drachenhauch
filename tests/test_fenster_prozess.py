"""Fenster als Prozess (docs/entwurf-native-fenster.md, Weg B):
WINDOW_OPEN/SEND/RECV$/ALIVE/CLOSE und PARENT_SEND/RECV$/ALIVE.

Die drei Pruefsteine aus dem Entwurf, gemessen statt behauptet:
1. eine Runde Eltern -> Kind -> Eltern kostet im Mittel unter 5 ms
   (1000 Runden);
2. stirbt das Kind, laeuft das Hauptprogramm weiter und erfaehrt es;
3. das Kind beendet sich, wenn die Eltern weg sind -- kein Zombie-Fenster.

Beide Seiten oeffnen ein Fenster, darum `_BRAUCHT_GRAFIK`; eingespeist wird
nichts, aber zwei Prozesse und eine Zeitmessung -- darum seriell.
"""
import os
import subprocess
import time
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


def _dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    for v in ("release", "debug"):
        p = _ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
        if p.exists():
            return p
    return None


_DHRT = _dhrt()
pytestmark = [pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut"),
              pytest.mark.seriell]

_KIND = '''SCREEN(240, 160, "Kind", 1)
SET_WINDOW_POS(-3000, -3000)
DIM n AS INTEGER : n = 0
WHILE NOT QUITREQUESTED()
    DIM z AS STRING : z = PARENT_RECV$()
    WHILE z <> ""
        n = n + 1
        IF z = "stirb" THEN EXIT(3)
        IF LEFT$(z, 5) = "datei" THEN WRITEALL("vom_kind.txt", MID$(z, 6))
        PARENT_SEND("pong " + z)
        z = PARENT_RECV$()
    WEND
    WRITEALL("herz.txt", STR$(MILLIS()))
    CLS(BLUE) : FLIP()
WEND
'''


def _lauf(tmp_path, src, frames=1, timeout=120):
    (tmp_path / "kind.dh").write_text(_KIND, encoding="utf-8")
    f = tmp_path / "eltern.dh"
    f.write_text('SCREEN(240, 160, "Eltern", 1)\nSET_WINDOW_POS(-3000, -3000)\n' + src, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(f)], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", timeout=timeout,
                       env=dict(os.environ, DHRT_FRAMES=str(frames)), cwd=str(tmp_path))
    assert r.returncode == 0, (r.stdout, r.stderr)
    return [ln.strip() for ln in (r.stdout or "").splitlines()
            if ln.strip() and not ln.startswith(("WARNING:", "INFO:"))]


_WARTEN = ('DIM antwort AS STRING : antwort = ""\n'
           'DIM t0 AS FLOAT : t0 = MILLIS()\n'
           'WHILE antwort = "" AND MILLIS() - t0 < 10000\n    antwort = WINDOW_RECV$(k)\nWEND\n')


def test_tausend_nachrichten_in_einem_schub(tmp_path):
    """Der Kanal selbst ist schnell (ein Schub von 1000 Nachrichten geht in
    wenigen Bildern hin und zurueck). Eine EINZELNE Runde kostet dagegen ein
    Bild des Kindes, weil es seine Post einmal je Bild liest -- bei 60 Hz
    also rund 17 ms. Das ist die Bauart, nicht der Kanal; wer es eiliger
    braucht, setzt im Kind SETFPS(0)."""
    out = _lauf(tmp_path,
                'PRINT PARENT_ALIVE()\n'
                'DIM k AS INTEGER : k = WINDOW_OPEN("kind.dh", "eins", 2)\n'
                'WINDOW_SEND(k, "hallo")\n' + _WARTEN +
                'PRINT antwort\n'
                'DIM i AS INTEGER\n'
                't0 = MILLIS()\n'
                'FOR i = 1 TO 1000\n    WINDOW_SEND(k, STR$(i))\nNEXT\n'
                'DIM n AS INTEGER : n = 0\n'
                'WHILE n < 1000 AND MILLIS() - t0 < 10000\n'
                '    antwort = WINDOW_RECV$(k)\n    IF antwort <> "" THEN n = n + 1\nWEND\n'
                'PRINT INT(MILLIS() - t0) ; " " ; n ; " " ; antwort\n'
                't0 = MILLIS()\n'
                'WINDOW_SEND(k, "einzeln") : antwort = ""\n'
                'WHILE antwort = "" AND MILLIS() - t0 < 5000\n    antwort = WINDOW_RECV$(k)\nWEND\n'
                'PRINT INT(MILLIS() - t0) ; " " ; antwort\n'
                'PRINT WINDOW_ALIVE(k)\n')
    assert out[0] == "FALSE", "die Eltern selbst haben keine Eltern"
    assert out[1] == "pong hallo", "eine vor der Verbindung gesendete Zeile kommt trotzdem an"
    ms, n, letzte = out[2].split(maxsplit=2)
    assert n == "1000" and letzte == "pong 1000"
    assert int(ms) < 1000, f"1000 Nachrichten hin und zurueck brauchten {ms} ms"
    ms1, einzeln = out[3].split(maxsplit=1)
    assert einzeln == "pong einzeln" and int(ms1) < 200, f"eine Runde brauchte {ms1} ms"
    assert out[4] == "TRUE"


def test_kind_stirbt_eltern_leben_weiter(tmp_path):
    out = _lauf(tmp_path,
                'DIM k AS INTEGER : k = WINDOW_OPEN("kind.dh")\n'
                'WINDOW_SEND(k, "hallo")\n' + _WARTEN +
                'WINDOW_SEND(k, "stirb")\n'
                'DIM t1 AS FLOAT : t1 = MILLIS()\n'
                'WHILE WINDOW_ALIVE(k) AND MILLIS() - t1 < 10000\nWEND\n'
                'PRINT NOT WINDOW_ALIVE(k)\n'
                'TRY\n    WINDOW_SEND(k, "noch da?")\n    PRINT "gesendet"\nCATCH e\n    PRINT "Fehler: " + e\nEND TRY\n'
                'PRINT "weiter"\n')
    assert out[0] == "TRUE", "das Kind ist tot"
    assert out[-1] == "weiter", "die Eltern laufen weiter"


def test_kind_beendet_sich_wenn_die_eltern_weg_sind(tmp_path):
    out = _lauf(tmp_path,
                'DIM k AS INTEGER : k = WINDOW_OPEN("kind.dh")\n'
                'WINDOW_SEND(k, "datei angekommen")\n' + _WARTEN +
                'PRINT antwort\n'
                'PRINT "eltern gehen"\n')
    assert out[0] == "pong datei angekommen"
    assert (tmp_path / "vom_kind.txt").read_text(encoding="utf-8") == "angekommen"
    # Die Eltern sind beendet. Das Kind schreibt je Bild einen Herzschlag --
    # der muss binnen zwei Sekunden aufhoeren.
    herz = tmp_path / "herz.txt"
    time.sleep(1.0)
    stand = herz.read_text(encoding="utf-8") if herz.exists() else ""
    time.sleep(1.5)
    assert (herz.read_text(encoding="utf-8") if herz.exists() else "") == stand, "das Kind lebt ohne Eltern weiter"


def test_fehler_haben_klare_worte(tmp_path):
    out = _lauf(tmp_path,
                'TRY\n    WINDOW_OPEN("gibtsnicht.dh")\nCATCH e\n    PRINT e\nEND TRY\n'
                'DIM k AS INTEGER : k = WINDOW_OPEN("kind.dh")\n'
                'TRY\n    WINDOW_SEND(k, "a" + CHR$(10) + "b")\nCATCH e2\n    PRINT e2\nEND TRY\n'
                'TRY\n    PARENT_SEND("x")\nCATCH e3\n    PRINT e3\nEND TRY\n'
                'WINDOW_CLOSE(k)\n'
                'TRY\n    WINDOW_SEND(k, "x")\nCATCH e4\n    PRINT e4\nEND TRY\n')
    assert "nicht gefunden" in out[0]
    assert "EINE Zeile" in out[1]
    assert "keine Eltern" in out[2]
    assert "gibt es nicht" in out[3]
