"""Lasso und Zauberstab im Sprite-Piloten (`examples/189_sprite_editor.dh`).

Der Pilot ist ein Drachenhauch-Programm, also laesst er sich nicht wie ein
Modul aufrufen -- geprueft wird er so, wie ein Mensch ihn bedient: mit
aufgezeichneter Eingabe (`AUTOMATION_PLAY`). Der Test schreibt die
Aufnahmedatei selbst; raylibs Format ist Text.

An der Logik wird dabei NICHTS geaendert. Die Kopie bekommt zwei Zusaetze:

* `SET_FULLSCREEN(TRUE)` -> `SET_WINDOW_POS(-3000, -3000)`. Die Vollbild-Groesse
  haengt am Monitor des Rechners; derselbe aufgezeichnete Mausweg traefe sonst
  auf jeder Maschine woanders hin. Und aus dem Bild geschoben, damit der
  ECHTE Zeiger nicht mitredet (raylib meldet seine Bewegung auch waehrend
  einer Wiedergabe).
* eine PRINT-Zeile je Bild, die nur BESTEHENDE Werte ausliest.

Weil die Fenstergeometrie erst zur Laufzeit feststeht, laeuft jeder Test
ZWEIMAL: einmal ohne Eingabe, um Ursprung und Zoom der Zeichenflaeche zu
erfahren, und einmal mit dem daraus gerechneten Mausweg.
"""
import os
import re
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent
_PILOT = _ROOT / "examples" / "189_sprite_editor.dh"


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

# raylib-Tastencodes (Grossbuchstaben)
TASTE_P, TASTE_Q, TASTE_Z = ord("P"), ord("Q"), ord("Z")
TASTE_S = ord("S")
TASTE_C, TASTE_D, TASTE_V, TASTE_X = ord("C"), ord("D"), ord("V"), ord("X")
TASTE_STRG = 341   # raylib KEY_LEFT_CONTROL
TASTE_ENTF = 261

# Zaehlt, was auf der aktiven Ebene gemalt ist -- und davon das, was
# AUSSERHALB der Auswahl liegt. Die zweite Zahl ist der eigentliche Punkt:
# sie muss 0 bleiben, solange eine Auswahl steht.
_PROBE = '''    DIM prG AS INTEGER : prG = 0
    DIM prD AS INTEGER : prD = 0
    DIM prX AS INTEGER
    DIM prY AS INTEGER
    FOR prY = 0 TO gh - 1
        FOR prX = 0 TO gw - 1
            IF GETALPHA(ebene[aktBild, aktEb], prX, prY) > 0 THEN
                prG = prG + 1
                IF NOT gewaehlt(prX, prY) THEN prD = prD + 1
            END IF
        NEXT
    NEXT
    ' Loecher: nicht gewaehlte Punkte, die in ihrer Zeile LINKS und RECHTS
    ' gewaehlte Nachbarn haben. Bei einer konvexen Form muss das 0 sein.
    DIM prL AS INTEGER : prL = 0
    IF selAn THEN
        FOR prY = selY TO selY + selH - 1
            DIM prLinks AS BOOLEAN : prLinks = FALSE
            DIM prLuecke AS INTEGER : prLuecke = 0
            FOR prX = selX TO selX + selB - 1
                IF gewaehlt(prX, prY) THEN
                    prL = prL + prLuecke
                    prLuecke = 0
                    prLinks = TRUE
                ELSE
                    IF prLinks THEN prLuecke = prLuecke + 1
                END IF
            NEXT
        NEXT
    END IF
    PRINT "P " + STR$(ox) + " " + STR$(oy) + " " + STR$(zoom) + " " + STR$(werkzeug) + _
          " " + STR$(selN) + " " + STR$(selB) + " " + STR$(selH) + _
          " " + STR$(prG) + " " + STR$(prD) + " " + STR$(prL)
'''


def _kopie(tmp_path):
    src = _PILOT.read_text(encoding="utf-8")
    assert src.count("SET_FULLSCREEN(TRUE)") == 1
    src = src.replace("SET_FULLSCREEN(TRUE)", "SET_WINDOW_POS(-3000, -3000)")
    assert src.count("    FLIP()\nWEND") == 1
    src = src.replace("    FLIP()\nWEND", _PROBE + "    FLIP()\nWEND")
    ziel = tmp_path / "pilot.dh"
    ziel.write_text(src, encoding="utf-8")
    return ziel


def _events(tmp_path, events):
    lines = ["# Test-Aufnahme", "c %d" % len(events)]
    for frame, typ, *params in events:
        p = (list(params) + [0, 0, 0, 0])[:4]
        lines.append("e %d %d %d %d %d %d // Event: test"
                     % (frame, typ, p[0], p[1], p[2], p[3]))
    (tmp_path / "ev.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")


def _lauf(tmp_path, frames, events=None):
    quelle = _kopie(tmp_path)
    if events is not None:
        _events(tmp_path, events)
        text = quelle.read_text(encoding="utf-8")
        text = text.replace("SETFPS(60)", 'SETFPS(60)\nAUTOMATION_PLAY("ev.txt")', 1)
        quelle.write_text(text, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(quelle)], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", timeout=180,
                       env=dict(os.environ, DHRT_FRAMES=str(frames)), cwd=str(tmp_path))
    assert r.returncode == 0, r.stderr
    zeilen = [ln for ln in (r.stdout or "").splitlines() if ln.startswith("P ")]
    assert zeilen, "keine Probe-Zeile\n%s\n%s" % (r.stdout, r.stderr)
    return [[int(v) for v in re.split(r"\s+", ln)[1:]] for ln in zeilen]


def _geometrie(tmp_path):
    """Ursprung und Zoom der Zeichenflaeche -- ohne sie trifft kein Mausweg."""
    letzte = _lauf(tmp_path, 6)[-1]
    ox, oy, zoom = letzte[0], letzte[1], letzte[2]
    assert zoom > 1, "Zoom nicht eingepasst"
    return ox, oy, zoom


def _mitte(ox, oy, zoom, x, y):
    """Bildpunkt -> Bildschirmpunkt (Mitte des Punktes)."""
    return ox + x * zoom + zoom // 2, oy + y * zoom + zoom // 2


def _taste(frame, code):
    return [(frame, KEY_DOWN, code), (frame + 1, KEY_UP, code)]


def _strg(frame, code):
    """Strg + Buchstabe. Strg muss in JEDEM Bild gemeldet werden, und der
    Buchstabe muss seine Flanke haben, WAEHREND Strg schon steht."""
    ev = [(frame + i, KEY_DOWN, TASTE_STRG) for i in range(5)]
    ev += [(frame + 2, KEY_DOWN, code), (frame + 3, KEY_UP, code),
           (frame + 5, KEY_UP, TASTE_STRG)]
    return ev


def _zug(frame, punkte):
    """Ein Mausweg. Gedrueckt halten heisst, den Knopf in JEDEM Bild zu
    melden -- so schreibt raylib eine gehaltene Taste mit."""
    ev = []
    for i, (sx, sy) in enumerate(punkte):
        ev.append((frame + i, MOUSE_POSITION, sx, sy))
        ev.append((frame + i, MOUSE_BUTTON_DOWN, 0))
    ev.append((frame + len(punkte), MOUSE_BUTTON_UP, 0))
    return ev


def _dreieck(ox, oy, zoom):
    """Ein Weg um ein rechtwinkliges Dreieck (2,2)-(20,2)-(2,20)."""
    ecken = [(2, 2), (20, 2), (2, 20)]
    weg = []
    for i in range(3):
        ax, ay = ecken[i]
        bx, by = ecken[(i + 1) % 3]
        for t in range(7):
            weg.append(_mitte(ox, oy, zoom,
                              round(ax + (bx - ax) * t / 7.0),
                              round(ay + (by - ay) * t / 7.0)))
    return weg


# --------------------------------------------------------------- Zauberstab
def test_zauberstab_waehlt_die_ganze_leere_ebene(tmp_path):
    """Eine frische Ebene ist ueberall gleich (durchsichtig) -- der Stab muss
    also alles nehmen. Das prueft den Fuellauf im Ganzen, ohne dass vorher
    etwas gemalt sein muesste."""
    ox, oy, zoom = _geometrie(tmp_path)
    mx, my = _mitte(ox, oy, zoom, 16, 16)
    ev = [(0, MOUSE_POSITION, mx, my)] + _taste(1, TASTE_Z) + [
        (4, MOUSE_POSITION, mx, my),
        (4, MOUSE_BUTTON_DOWN, 0),
        (5, MOUSE_BUTTON_UP, 0),
    ]
    letzte = _lauf(tmp_path, 12, ev)[-1]
    werkzeug, selN, selB, selH = letzte[3], letzte[4], letzte[5], letzte[6]
    assert werkzeug == 10, "Z waehlt den Zauberstab"
    assert (selN, selB, selH) == (32 * 32, 32, 32)


def test_zauberstab_bleibt_in_der_gemalten_flaeche(tmp_path):
    """Nach einem Strich zerfaellt die Ebene in zwei Flaechen. Ein Klick auf
    das Gemalte darf nur dieses nehmen -- nicht den Rest."""
    ox, oy, zoom = _geometrie(tmp_path)
    # Ein waagerechter Strich mit dem Stift (Vorgabewerkzeug), dann der
    # Zauberstab auf einen Punkt DARIN.
    strich = [_mitte(ox, oy, zoom, x, 16) for x in range(4, 28)]
    ev = [(0, MOUSE_POSITION) + _mitte(ox, oy, zoom, 4, 16)] + _zug(1, strich)
    start = 1 + len(strich) + 2
    ev += _taste(start, TASTE_Z)
    kx, ky = _mitte(ox, oy, zoom, 16, 16)
    ev += [(start + 3, MOUSE_POSITION, kx, ky),
           (start + 3, MOUSE_BUTTON_DOWN, 0),
           (start + 4, MOUSE_BUTTON_UP, 0)]
    letzte = _lauf(tmp_path, start + 12, ev)[-1]
    selN, selH, gemalt = letzte[4], letzte[6], letzte[7]
    assert gemalt >= 20, "der Strich muss angekommen sein"
    assert selN == gemalt, "der Stab nimmt genau das Gemalte"
    assert selH == 1, "ein Strich ist eine Zeile hoch"


# ----------------------------------------------------------------- Rechteck
def test_rechteck_entsteht_erst_beim_loslassen_und_klick_hebt_auf(tmp_path):
    """Das Rechteck baut seine Maske jetzt beim LOSLASSEN, nicht bei jedem
    Bild -- und ein Klick ohne Zug hebt die Auswahl auf statt einen
    Ein-Punkt-Rahmen stehen zu lassen. Das zweite ist kein Beiwerk: weil die
    Auswahl das Zeichnen begrenzt, traefe danach kein Strich mehr, und man
    saehe nicht, warum."""
    ox, oy, zoom = _geometrie(tmp_path)
    weg = [_mitte(ox, oy, zoom, x, x) for x in range(4, 14)]
    ev = [(0, MOUSE_POSITION) + weg[0]] + _taste(1, TASTE_S) + _zug(4, weg)
    ende = 4 + len(weg) + 4
    zeilen = _lauf(tmp_path, ende + 8, ev)
    selN, selB, selH = zeilen[-1][4], zeilen[-1][5], zeilen[-1][6]
    assert (selB, selH) == (10, 10), "von (4,4) bis (13,13)"
    assert selN == 100, "ein Rechteck ist ganz gewaehlt"

    # ... und jetzt ein Klick ohne Zug.
    klick = _mitte(ox, oy, zoom, 20, 20)
    ev2 = ev + [(ende, MOUSE_POSITION, klick[0], klick[1]),
                (ende, MOUSE_BUTTON_DOWN, 0),
                (ende + 1, MOUSE_BUTTON_UP, 0)]
    assert _lauf(tmp_path, ende + 10, ev2)[-1][4] == 0, "Klick ohne Zug hebt auf"


# -------------------------------------------------------------------- Lasso
def test_lasso_waehlt_eine_freiform(tmp_path):
    """Um ein Dreieck gezogen. Der Rahmen ist 19x19, gewaehlt ist gut die
    Haelfte davon -- genau das unterscheidet eine Freiform vom Rechteck."""
    ox, oy, zoom = _geometrie(tmp_path)
    weg = _dreieck(ox, oy, zoom)
    ev = [(0, MOUSE_POSITION) + weg[0]] + _taste(1, TASTE_Q) + _zug(4, weg)
    letzte = _lauf(tmp_path, 4 + len(weg) + 10, ev)[-1]
    werkzeug, selN, selB, selH = letzte[3], letzte[4], letzte[5], letzte[6]
    assert werkzeug == 9, "Q waehlt das Lasso"
    assert (selB, selH) == (19, 19), "der Rahmen umspannt die drei Ecken"
    # Die halbe Rahmenflaeche plus der gezogene Rand -- eine feste Zahl waere
    # eine Zusicherung ueber die Rundung, nicht ueber die Auswahl.
    assert 150 <= selN <= 260, selN
    assert selN < selB * selH, "eine Freiform ist nicht ihr Rahmen"


def test_lasso_laesst_keine_loecher_in_der_flaeche(tmp_path):
    """Der Fund, den nur das gerenderte Bild zeigte.

    Das Vieleck lag auf den ECKEN der Punkte, geprueft wurden ihre MITTEN.
    Ein 45-Grad-Rand faellt damit genau auf eine Kante des Vielecks, und dort
    entscheidet die Rundung -- eine ganze Reihe einzelner Punkte blieb
    ungewaehlt, mitten in der Flaeche. In den Zahlen (Rahmen, Anzahl) sah man
    davon nichts; die Anzahl war nur unauffaellig zu klein.
    """
    ox, oy, zoom = _geometrie(tmp_path)
    weg = _dreieck(ox, oy, zoom)
    ev = [(0, MOUSE_POSITION) + weg[0]] + _taste(1, TASTE_Q) + _zug(4, weg)
    letzte = _lauf(tmp_path, 4 + len(weg) + 10, ev)[-1]
    assert letzte[9] == 0, "kein Punkt darf zwischen gewaehlten Nachbarn liegen"


def test_auswahl_begrenzt_den_stift(tmp_path):
    """Der eigentliche Zweck. Ein Strich quer ueber das ganze Bild darf nur
    dort ankommen, wo die Auswahl liegt -- vorher lief er ungehindert durch,
    weil die Auswahl nur fuers Kopieren galt."""
    ox, oy, zoom = _geometrie(tmp_path)
    weg = _dreieck(ox, oy, zoom)
    ev = [(0, MOUSE_POSITION) + weg[0]] + _taste(1, TASTE_Q) + _zug(4, weg)
    start = 4 + len(weg) + 2
    ev += _taste(start, TASTE_P)
    strich = [_mitte(ox, oy, zoom, x, 8) for x in range(0, 32)]
    ev += _zug(start + 3, strich)
    letzte = _lauf(tmp_path, start + 3 + len(strich) + 10, ev)[-1]
    selN, gemalt, draussen = letzte[4], letzte[7], letzte[8]
    assert selN > 100, "die Auswahl muss stehen"
    assert gemalt > 0, "innerhalb der Auswahl muss der Strich ankommen"
    assert draussen == 0, "ausserhalb darf kein Punkt gesetzt sein"
    assert gemalt < 32, "sonst waere gar nichts begrenzt worden"


def test_entf_loescht_genau_die_auswahl(tmp_path):
    """Ein Strich quer durchs Bild, ein Lasso ueber ein Stueck davon, Entf.
    Danach darf INNERHALB der Auswahl nichts mehr stehen und ausserhalb
    alles. Vorher raeumte Entf den ganzen RAHMEN der Auswahl -- bei einer
    Freiform sind das die Ecken mit."""
    ox, oy, zoom = _geometrie(tmp_path)
    strich = [_mitte(ox, oy, zoom, x, 8) for x in range(0, 32)]
    ev = [(0, MOUSE_POSITION) + strich[0]] + _zug(1, strich)
    start = 1 + len(strich) + 2
    weg = _dreieck(ox, oy, zoom)
    ev += _taste(start, TASTE_Q) + _zug(start + 3, weg)
    entf = start + 3 + len(weg) + 2
    ev += _taste(entf, TASTE_ENTF)
    letzte = _lauf(tmp_path, entf + 10, ev)[-1]
    selN, gemalt, draussen = letzte[4], letzte[7], letzte[8]
    assert selN > 100, "die Auswahl muss stehen"
    assert draussen == gemalt, "innerhalb der Auswahl darf nichts uebrig sein"
    assert 15 <= gemalt < 32, "ausserhalb muss der Strich stehen bleiben"


def test_ausschneiden_und_einfuegen_nimmt_nur_die_freiform_mit(tmp_path):
    """Ein Strich, ein Lasso ueber ein Stueck davon, Strg+X, Auswahl weg,
    Strg+V an (0,0). Zurueck kommen darf nur, was gewaehlt war.

    Die Ablage ist ein Rechteck -- sie MUSS der Rahmen der Maske sein. Was
    darin liegt, aber nicht gewaehlt war, bekommt beim Kopieren Deckkraft 0;
    ohne das braechte ein Lasso beim Einfuegen die Ecken seines Rahmens mit
    (in diesem Aufbau: sechs Punkte des Strichs, die neben dem Dreieck
    liegen).
    """
    ox, oy, zoom = _geometrie(tmp_path)
    strich = [_mitte(ox, oy, zoom, x, 8) for x in range(0, 32)]
    ev = [(0, MOUSE_POSITION) + strich[0]] + _zug(1, strich)
    t = 1 + len(strich) + 2
    weg = _dreieck(ox, oy, zoom)
    ev += _taste(t, TASTE_Q) + _zug(t + 3, weg)
    t = t + 3 + len(weg) + 2
    ev += _strg(t, TASTE_X)
    zeilen_x = _lauf(tmp_path, t + 10, ev)
    nach_schnitt = zeilen_x[-1][7]
    assert 15 <= nach_schnitt < 32, "Strg+X raeumt nur die Auswahl"

    t += 8
    ev += _strg(t, TASTE_D)          # Auswahl weg -> Einfuegen an (0,0)
    t += 8
    ev += _strg(t, TASTE_V)
    letzte = _lauf(tmp_path, t + 12, ev)[-1]
    assert letzte[4] == 0, "die Auswahl ist aufgehoben"
    assert letzte[7] == 32, ("eingefuegt gehoert genau das Ausgeschnittene -- "
                             "nicht der ganze Rahmen")
