"""Der Sprite-Pilot, bedient wie von Hand (`examples/189_sprite_editor.dh`).

Abgedeckt: Auswahl-Werkzeuge (Lasso, Zauberstab, Verschieben), GIMP-Paletten
(.gpl), Kachel-Ansicht, Statistik, Zuschneiden/Groesse aendern,
Animationsbereiche und die GB-Code-Ausgabe.

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
import json
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
TASTE_S, TASTE_V_WZ = ord("S"), ord("V")
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
          " " + STR$(prG) + " " + STR$(prD) + " " + STR$(prL) + _
          " " + STR$(selX) + " " + STR$(selY) + _
          " " + STR$(GUI_GET_X(bPalLaden)) + " " + STR$(GUI_GET_Y(bPalLaden)) + _
          " " + STR$(GUI_GET_X(bPalSichern)) + " " + STR$(GUI_GET_Y(bPalSichern)) + _
          " " + STR$(pal[0]) + " " + STR$(pal[15]) + _
          " " + STR$(cw) + " " + STR$(ch) + _
          " " + STR$(GUI_GET_X(bStat)) + " " + STR$(GUI_GET_Y(bStat)) + _
          " " + STR$(GUI_GET_X(cbKacheln)) + " " + STR$(GUI_GET_Y(cbKacheln)) + _
          " " + STR$(gw) + " " + STR$(gh) + " " + STR$(anzBild) + _
          " " + STR$(anzAnim) + " " + STR$(aktAnim) + _
          " " + STR$(anVon[0]) + " " + STR$(anBis[0]) + " " + STR$(anFps[0]) + _
          " " + STR$(GUI_GET_X(bZuschnitt)) + " " + STR$(GUI_GET_Y(bZuschnitt)) + _
          " " + STR$(GUI_GET_X(bGroesse)) + " " + STR$(GUI_GET_Y(bGroesse)) + _
          " " + STR$(GUI_GET_X(bAnimNeu)) + " " + STR$(GUI_GET_Y(bAnimNeu)) + _
          " " + STR$(GUI_GET_X(bAnimWeg)) + " " + STR$(GUI_GET_Y(bAnimWeg)) + _
          " " + STR$(GUI_GET_X(bBildKopie)) + " " + STR$(GUI_GET_Y(bBildKopie)) + _
          " " + STR$(GUI_GET_X(bBildWeg)) + " " + STR$(GUI_GET_Y(bBildWeg)) + _
          " " + STR$(GUI_WINDOW_GET_X(winGr) + GUI_GET_X(bGrOk)) + _
          " " + STR$(GUI_WINDOW_GET_Y(winGr) + GUI_GET_Y(bGrOk)) + _
          " " + STR$(GUI_GET_X(lstBild)) + " " + STR$(GUI_GET_Y(lstBild)) + _
          " " + STR$(GUI_GET_X(bEbNeu)) + " " + STR$(GUI_GET_Y(bEbNeu)) + _
          " " + STR$(GUI_GET_X(cbEbSicht)) + " " + STR$(GUI_GET_Y(cbEbSicht)) + _
          " " + STR$(anzEb) + " " + STR$(aktEb) + " " + STR$(ebSicht[aktEb]) + _
          " " + STR$(GUI_GET_X(bGbCode)) + " " + STR$(GUI_GET_Y(bGbCode)) + _
          " " + STR$(GUI_GET_X(bDhanim)) + " " + STR$(GUI_GET_Y(bDhanim)) + _
          " " + STR$(GUI_GET_X(bBlatt)) + " " + STR$(GUI_GET_Y(bBlatt)) + _
          " " + STR$(GUI_GET_X(bBildName)) + " " + STR$(GUI_GET_Y(bBildName)) + _
          " " + STR$(GUI_WINDOW_GET_X(winName) + GUI_GET_X(bNamOk)) + _
          " " + STR$(GUI_WINDOW_GET_Y(winName) + GUI_GET_Y(bNamOk)) + _
          " " + STR$(LEN(bildName[aktBild])) + _
          " " + STR$(GUI_GET_X(bSpiegelX)) + " " + STR$(GUI_GET_Y(bSpiegelX)) + _
          " " + STR$(GUI_GET_X(bSpiegelY)) + " " + STR$(GUI_GET_Y(bSpiegelY)) + _
          " " + STR$(GUI_GET_X(bDrehR)) + " " + STR$(GUI_GET_Y(bDrehR)) + _
          " " + STR$(GUI_GET_X(bDrehL)) + " " + STR$(GUI_GET_Y(bDrehL)) + _
          " " + STR$(GETALPHA(ebene[0, 0], 0, 0)) + _
          " " + STR$(GETALPHA(ebene[0, 0], gw - 1, 0)) + _
          " " + STR$(GETALPHA(ebene[0, 0], gw - 1, gh - 1)) + _
          " " + STR$(GETALPHA(ebene[0, 0], 0, gh - 1)) + _
          " " + STR$(uAnz) + _
          " " + STR$(GUI_GET_X(spMs)) + " " + STR$(GUI_GET_Y(spMs)) + _
          " " + STR$(GUI_GET_W(spMs)) + _
          " " + STR$(bildMs[0]) + " " + STR$(bildMs[1]) + " " + STR$(dauerMs(0)) + _
          " " + STR$(GUI_GET_X(bGif)) + " " + STR$(GUI_GET_Y(bGif)) + _
          " " + STR$(GUI_GET_X(bSichern)) + " " + STR$(GUI_GET_W(bSichern)) + _
          " " + STR$(GUI_GET_X(bOeffnen)) + " " + STR$(GUI_GET_W(bOeffnen)) + _
          " " + STR$(GUI_GET_X(bNeu)) + " " + STR$(GUI_GET_W(bNeu)) + _
          " " + STR$(GUI_GET_Y(bNeu)) + _
          " " + STR$(GUI_WINDOW_GET_X(winNeu) + GUI_GET_X(bNeuOk)) + _
          " " + STR$(GUI_WINDOW_GET_Y(winNeu) + GUI_GET_Y(bNeuOk))
'''

# Die Reihenfolge der Zahlen in der Probe-Zeile. Ein Test liest sie ueber den
# Namen -- bei vierzig Feldern ist `letzte[8]` nicht mehr zu lesen und beim
# Anhaengen eines Feldes leicht zu verrutschen.
#
# Die Knopf-Lagen stehen mit drin, weil ein aufgezeichneter Klick sie
# braucht: `win` sitzt bei (0,0) ohne Rahmen, dort sind Widget-Koordinaten
# zugleich Bildschirm-Koordinaten. Nur `bGrOk` liegt in einem eigenen
# Fenster -- dessen Ursprung kommt dazu.
_FELDER = ("ox oy zoom werkzeug selN selB selH gemalt draussen loecher "
           "selX selY palLX palLY palSX palSY pal0 pal15 "
           "cw ch statX statY kachX kachY "
           "gw gh anzBild anzAnim aktAnim anVon anBis anFps "
           "zusX zusY grX grY animNX animNY animWX animWY "
           "kopieX kopieY bildWX bildWY grOkX grOkY lstBX lstBY "
           "ebNeuX ebNeuY ebSichtX ebSichtY anzEb aktEb sichtAkt gbX gbY faX faY "
           "blattX blattY namX namY namOkX namOkY namLen "
           "spXX spXY spYX spYY drRX drRY drLX drLY "
           "eckLO eckRO eckRU eckLU uAnz "
           "msX msY msW ms0 ms1 dauer0 gifX gifY "
           "sichX sichW oeffX oeffW neuX neuW kopfY neuOkX neuOkY").split()


def _kopie(tmp_path, dialoge=None):
    """`dialoge` ersetzt einzelne Dateidialog-Aufrufe durch feste Pfade.

    Ein FILE_OPEN_DIALOG ist ein natives, blockierendes Fenster des
    Betriebssystems -- keine aufgezeichnete Eingabe erreicht es. Ersetzt wird
    deshalb GENAU der Dialogaufruf; alles dahinter (Lesen, Zerlegen,
    Uebernehmen) ist der echte Code des Piloten.
    """
    src = _PILOT.read_text(encoding="utf-8")
    for alt, neu in (dialoge or {}).items():
        assert src.count(alt) == 1, alt
        src = src.replace(alt, neu)
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


def _lauf(tmp_path, frames, events=None, dialoge=None):
    return _starte(tmp_path, frames, events, dialoge)[0]


def _starte(tmp_path, frames, events=None, dialoge=None):
    """Wie `_lauf`, liefert aber auch die uebrige Ausgabe -- ein Test, der
    einen mehrzeiligen Text pruefen will, laesst ihn sich auf EINER Zeile
    ausgeben und liest sie hier ab."""
    quelle = _kopie(tmp_path, dialoge)
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
    proben = [dict(zip(_FELDER, [int(v) for v in re.split(r"\s+", ln)[1:]]))
              for ln in zeilen]
    return proben, (r.stdout or "").splitlines()


def _geometrie(tmp_path):
    """Ursprung und Zoom der Zeichenflaeche -- ohne sie trifft kein Mausweg."""
    letzte = _lauf(tmp_path, 6)[-1]
    ox, oy, zoom = letzte["ox"], letzte["oy"], letzte["zoom"]
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
    assert letzte["werkzeug"] == 10, "Z waehlt den Zauberstab"
    assert (letzte["selN"], letzte["selB"], letzte["selH"]) == (32 * 32, 32, 32)


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
    selN, selH, gemalt = letzte["selN"], letzte["selH"], letzte["gemalt"]
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
    selN = zeilen[-1]["selN"]
    selB, selH = zeilen[-1]["selB"], zeilen[-1]["selH"]
    assert (selB, selH) == (10, 10), "von (4,4) bis (13,13)"
    assert selN == 100, "ein Rechteck ist ganz gewaehlt"

    # ... und jetzt ein Klick ohne Zug.
    klick = _mitte(ox, oy, zoom, 20, 20)
    ev2 = ev + [(ende, MOUSE_POSITION, klick[0], klick[1]),
                (ende, MOUSE_BUTTON_DOWN, 0),
                (ende + 1, MOUSE_BUTTON_UP, 0)]
    assert _lauf(tmp_path, ende + 10, ev2)[-1]["selN"] == 0, "Klick ohne Zug hebt auf"


# -------------------------------------------------------------------- Lasso
def test_lasso_waehlt_eine_freiform(tmp_path):
    """Um ein Dreieck gezogen. Der Rahmen ist 19x19, gewaehlt ist gut die
    Haelfte davon -- genau das unterscheidet eine Freiform vom Rechteck."""
    ox, oy, zoom = _geometrie(tmp_path)
    weg = _dreieck(ox, oy, zoom)
    ev = [(0, MOUSE_POSITION) + weg[0]] + _taste(1, TASTE_Q) + _zug(4, weg)
    letzte = _lauf(tmp_path, 4 + len(weg) + 10, ev)[-1]
    werkzeug, selN = letzte["werkzeug"], letzte["selN"]
    selB, selH = letzte["selB"], letzte["selH"]
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
    assert letzte["loecher"] == 0, "kein Punkt darf zwischen gewaehlten Nachbarn liegen"


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
    selN, gemalt = letzte["selN"], letzte["gemalt"]
    draussen = letzte["draussen"]
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
    selN, gemalt = letzte["selN"], letzte["gemalt"]
    draussen = letzte["draussen"]
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
    nach_schnitt = zeilen_x[-1]["gemalt"]
    assert 15 <= nach_schnitt < 32, "Strg+X raeumt nur die Auswahl"

    t += 8
    ev += _strg(t, TASTE_D)          # Auswahl weg -> Einfuegen an (0,0)
    t += 8
    ev += _strg(t, TASTE_V)
    letzte = _lauf(tmp_path, t + 12, ev)[-1]
    assert letzte["selN"] == 0, "die Auswahl ist aufgehoben"
    assert letzte["gemalt"] == 32, ("eingefuegt gehoert genau das Ausgeschnittene -- "
                             "nicht der ganze Rahmen")


# --------------------------------------------------------------- Verschieben
def test_verschieben_nimmt_auswahl_und_inhalt_mit(tmp_path):
    """Ein Strich, ein Lasso ueber ein Stueck davon, dann mit V um (5, 4)
    versetzt. Der Inhalt muss vollstaendig ankommen -- und die MASKE muss
    mitwandern: bliebe sie liegen, zeigte sie auf die Stelle, wo das
    Verschobene gerade nicht mehr ist, und der naechste Strich landete dort.
    """
    ox, oy, zoom = _geometrie(tmp_path)
    strich = [_mitte(ox, oy, zoom, x, 8) for x in range(0, 32)]
    ev = [(0, MOUSE_POSITION) + strich[0]] + _zug(1, strich)
    t = 1 + len(strich) + 2
    weg = _dreieck(ox, oy, zoom)
    ev += _taste(t, TASTE_Q) + _zug(t + 3, weg)
    t = t + 3 + len(weg) + 2
    ev += _taste(t, TASTE_V_WZ)
    ev += _zug(t + 3, [_mitte(ox, oy, zoom, 10, 10), _mitte(ox, oy, zoom, 15, 14)])
    vorher = _lauf(tmp_path, t + 2, ev)[-1]
    letzte = _lauf(tmp_path, t + 16, ev)[-1]
    assert letzte["werkzeug"] == 11, "V waehlt das Verschieben"
    assert (vorher["selX"], vorher["selY"]) == (2, 2), "das Lasso begann links oben"
    assert (letzte["selX"], letzte["selY"]) == (7, 6), "die Maske wandert um (5, 4) mit"
    assert (letzte["selB"], letzte["selH"]) == (19, 19), "die Form aendert sich dabei nicht"
    assert letzte["selN"] == vorher["selN"], "und ihre Punktzahl auch nicht"
    assert letzte["gemalt"] == 32, "kein Punkt darf beim Verschieben verloren gehen"


def test_verschieben_ohne_auswahl_nimmt_die_ganze_ebene(tmp_path):
    """Ohne Auswahl gilt die ganze Ebene -- und was ueber den Rand geschoben
    wird, ist weg. Ein Strich ueber die volle Breite, um 10 nach rechts:
    zehn Punkte fallen heraus."""
    ox, oy, zoom = _geometrie(tmp_path)
    strich = [_mitte(ox, oy, zoom, x, 8) for x in range(0, 32)]
    ev = [(0, MOUSE_POSITION) + strich[0]] + _zug(1, strich)
    t = 1 + len(strich) + 2
    ev += _taste(t, TASTE_V_WZ)
    ev += _zug(t + 3, [_mitte(ox, oy, zoom, 4, 8), _mitte(ox, oy, zoom, 14, 8)])
    letzte = _lauf(tmp_path, t + 16, ev)[-1]
    assert letzte["selN"] == 0, "es entsteht dabei keine Auswahl"
    assert letzte["gemalt"] == 22, "32 minus die zehn ueber den rechten Rand"


# ------------------------------------------------------------------ Palette
_DIALOG_LADEN = 'FILE_OPEN_DIALOG("Palette laden", "gpl")'
_DIALOG_SICHERN = 'FILE_SAVE_DIALOG("Palette sichern", "palette.gpl", "gpl")'


def _klick(frame, x, y):
    """Ein Klick genau auf (x, y)."""
    return [(frame, MOUSE_POSITION, x, y),
            (frame + 1, MOUSE_POSITION, x, y),
            (frame + 1, MOUSE_BUTTON_DOWN, 0),
            (frame + 2, MOUSE_BUTTON_UP, 0)]


def _knopf_klick(frame, x, y, breite=128):
    """Mitte eines Knopfes -- die Lagen in der Probe sind seine linke obere
    Ecke, die meisten Knoepfe sind 128x28 gross.

    `breite` ist kein Beiwerk: die vier Wandlungs-Knoepfe sind nur 62 breit,
    und mit dem festen +64 landete der Klick in der LUECKE zwischen zweien --
    also nirgends. Das sieht in der Probe genauso aus wie ein Knopf, der
    nichts tut."""
    return _klick(frame, x + breite // 2, y + 14)


def test_palette_laden_uebergeht_krumme_zeilen(tmp_path):
    """Eine .gpl von Hand geschrieben, mit allem, was in echten Dateien
    vorkommt: Kommentar, Tabulator, mehrere Leerzeichen, eine Wortzeile und
    eine mit Werten ueber 255. Die beiden letzten muessen UEBERGANGEN werden,
    nicht die Palette verschieben -- daran, dass die 16. Farbe stimmt, sieht
    man beides auf einmal.
    """
    farben = [(10 * (i + 1), 0, 0) for i in range(16)]
    zeilen = ["GIMP Palette", "Name: Probe", "Columns: 8", "# ein Kommentar",
              "%3d %3d %3d	Rot" % farben[0],
              "300 300 300	zu gross",
              "Rot Gruen Blau",
              ""]
    for f in farben[1:]:
        zeilen.append("%d   %d %d	Farbe" % f)
    zeilen.append("0 0 255	die siebzehnte")     # passt nicht mehr hinein
    text = chr(10).join(zeilen) + chr(10)
    (tmp_path / "pal.gpl").write_text(text, encoding="utf-8")

    geo = _lauf(tmp_path, 6)[-1]
    ev = _knopf_klick(2, geo["palLX"], geo["palLY"])
    letzte = _lauf(tmp_path, 14, ev, dialoge={_DIALOG_LADEN: '"pal.gpl"'})[-1]
    assert letzte["pal0"] == 10 * 65536, "erste Farbe der Datei"
    assert letzte["pal15"] == 160 * 65536, ("16. GUELTIGE Farbe -- eine "
                                            "uebernommene Fehlzeile schoebe alles")


def test_palette_sichern_liest_der_qt_editor_zurueck(tmp_path):
    """Geschrieben wird gegen einen FREMDEN Leser geprueft: `_parse_gpl` des
    Qt-Sprite-Editors. Ein Format, das nur sein eigener Schreiber wieder
    liest, ist nicht geprueft, sondern nur in sich stimmig."""
    pytest.importorskip("PySide6")
    os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
    from drachenhauch.spriteeditor_qt import SpriteEditorWindow

    geo = _lauf(tmp_path, 6)[-1]
    ev = _knopf_klick(2, geo["palSX"], geo["palSY"])
    _lauf(tmp_path, 14, ev, dialoge={_DIALOG_SICHERN: '"raus.gpl"'})
    ziel = tmp_path / "raus.gpl"
    assert ziel.exists(), "der Knopf muss geschrieben haben"
    farben = SpriteEditorWindow._parse_gpl(ziel)
    assert len(farben) == 16, "sechzehn Plaetze, sechzehn Zeilen"
    assert farben[0] == (0, 0, 0, 255), "die Palette beginnt mit Schwarz"
    assert farben[2] == (232, 75, 75, 255), "und hat an dritter Stelle das Rot"
    kopf = ziel.read_text(encoding="utf-8").splitlines()[0]
    assert kopf == "GIMP Palette", "die Kennzeile, an der GIMP das Format erkennt"


def test_palette_rundweg_ueber_den_qt_schreiber(tmp_path):
    """Die andere Richtung: der Qt-Editor schreibt, der Pilot liest."""
    pytest.importorskip("PySide6")
    os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
    from drachenhauch.spriteeditor_qt import SpriteEditorWindow

    farben = [(i * 4, 128, 255 - i * 4, 255) for i in range(16)]
    SpriteEditorWindow._write_gpl(tmp_path / "pal.gpl", farben, name="Rundweg")
    geo = _lauf(tmp_path, 6)[-1]
    ev = _knopf_klick(2, geo["palLX"], geo["palLY"])
    letzte = _lauf(tmp_path, 14, ev, dialoge={_DIALOG_LADEN: '"pal.gpl"'})[-1]
    assert letzte["pal0"] == (0 << 16) | (128 << 8) | 255
    assert letzte["pal15"] == (60 << 16) | (128 << 8) | 195


# ---------------------------------------------------- Kachel-Ansicht + Statistik
_DIALOG_STAT = 'statDlg = GUI_DIALOG("Statistik", statistik$())'
_STAT_ERSATZ = ('PRINT "STAT " + REPLACE$(statistik$(), CHR$(10), " | ") : '
                'statDlg = GUI_DIALOG("Statistik", "x")')


def _kasten_klick(frame, x, y):
    """Ein Kaestchen wird auf dem Kaestchen getroffen, nicht auf der
    Beschriftung -- deshalb ein anderer Versatz als beim Knopf."""
    return [(frame, MOUSE_POSITION, x + 8, y + 8),
            (frame + 1, MOUSE_POSITION, x + 8, y + 8),
            (frame + 1, MOUSE_BUTTON_DOWN, 0),
            (frame + 2, MOUSE_BUTTON_UP, 0)]


def _einpassung(cw, ch, kanten):
    """Dieselbe Rechnung wie im Piloten -- der Test sagt damit, WELCHER Zoom
    richtig ist, statt nur "kleiner als vorher"."""
    return max(1, min(32, min((cw - 40) // (32 * kanten), (ch - 40) // (32 * kanten))))


def test_kachel_ansicht_passt_den_zoom_neu_ein(tmp_path):
    """Ohne das waere die Kachel-Ansicht bei eingepasstem Zoom gar nicht zu
    sehen: das mittlere Bild fuellt die Flaeche schon allein, die acht
    Nachbarn lagen ausserhalb."""
    geo = _lauf(tmp_path, 6)[-1]
    assert geo["zoom"] == _einpassung(geo["cw"], geo["ch"], 1), "erst einfach eingepasst"
    ev = _kasten_klick(2, geo["kachX"], geo["kachY"])
    letzte = _lauf(tmp_path, 14, ev)[-1]
    assert letzte["zoom"] == _einpassung(geo["cw"], geo["ch"], 3)
    assert letzte["zoom"] * 32 * 3 <= geo["cw"], "das 3x3 muss hineinpassen"
    assert letzte["zoom"] < geo["zoom"], "und dafuer wird herausgezoomt"


def test_statistik_zaehlt_deckende_punkte_und_farben(tmp_path):
    """Sechzehn Punkte in einer Farbe auf ein leeres 32x32-Bild. Die Zahlen
    im Kasten sind damit von Hand nachzurechnen -- nur die Prozentangabe
    nicht, und genau die verrutscht am leichtesten.

    Der Kasten selbst wird nicht geprueft, sondern der TEXT: die Kopie gibt
    ihn auf einer Zeile aus. Ersetzt ist nur die Anzeige; gerechnet hat die
    echte `statistik$()`.
    """
    geo = _lauf(tmp_path, 6)[-1]
    strich = [_mitte(geo["ox"], geo["oy"], geo["zoom"], x, 16) for x in range(4, 20)]
    ev = [(0, MOUSE_POSITION) + strich[0]] + _zug(1, strich)
    t = 1 + len(strich) + 2
    ev += _knopf_klick(t, geo["statX"], geo["statY"])
    proben, roh = _starte(tmp_path, t + 14, ev, dialoge={_DIALOG_STAT: _STAT_ERSATZ})
    assert proben[-1]["gemalt"] == 16, "so viele Punkte hat der Strich"
    zeilen = [z for z in roh if z.startswith("STAT ")]
    assert zeilen, "der Knopf muss die Statistik gerechnet haben"
    z = zeilen[-1]
    assert "32 x 32 Punkte, 1 Bilder, 1 Ebenen" in z
    assert re.search(r"Punkte gesamt:\s+1024", z), z
    assert re.search(r"davon deckend:\s+16\s+\(1\.6 %\)", z), z
    assert re.search(r"davon durchsichtig:\s+1008\s+\(98\.4 %\)", z), z
    assert re.search(r"verschiedene Farben:\s+1\b", z), z
    assert re.search(r"haeufigste Farbe:\s+#E84B4B\s+\(16 Punkte\)", z), z
    assert "Je Bild:" not in z, "bei einem einzigen Bild waere das eine leere Zeile"


def test_statistik_nennt_die_haeufigste_farbe(tmp_path):
    """DREI Farben, und die haeufigste ist weder die zuerst noch die zuletzt
    gemalte.

    Mit nur zwei Farben war der Test wertlos -- nachgemessen: mit
    ausgebauter Maximumsuche blieb er gruen. Mit dreien faellt er, sobald
    statt des groessten Zaehlers der zuletzt gesehene genommen wird.

    Gegen "nimm den ERSTEN Eintrag" laesst er sich nicht absichern: die
    Reihenfolge von MAPKEYS ist keine zugesicherte (weder Einfuege- noch
    Sortierreihenfolge), also ist nicht vorherzusagen, welche Farbe dabei
    herauskaeme. Das steht hier, damit niemand den Test fuer staerker haelt,
    als er ist.
    """
    geo = _lauf(tmp_path, 6)[-1]

    def strich(y, von, bis):
        return [_mitte(geo["ox"], geo["oy"], geo["zoom"], x, y) for x in range(von, bis)]

    ev = [(0, MOUSE_POSITION) + strich(20, 2, 8)[0]]
    t = 1
    for taste, weg in ((ord("1"), strich(20, 2, 8)),        # Schwarz, 6 Punkte
                       (ord("3"), strich(10, 2, 22)),       # Rot, 20 Punkte
                       (ord("2"), strich(25, 2, 12))):      # Weiss, 10 Punkte
        ev += _taste(t, taste)
        ev += _zug(t + 3, weg)
        t = t + 3 + len(weg) + 2
    ev += _knopf_klick(t, geo["statX"], geo["statY"])
    _, roh = _starte(tmp_path, t + 14, ev, dialoge={_DIALOG_STAT: _STAT_ERSATZ})
    z = [x for x in roh if x.startswith("STAT ")][-1]
    assert re.search(r"davon deckend:\s+36\b", z), z
    assert re.search(r"verschiedene Farben:\s+3\b", z), z
    assert re.search(r"haeufigste Farbe:\s+#E84B4B\s+\(20 Punkte\)", z), z


# ---------------------------------------------- Zuschneiden / Groesse aendern
def test_zuschneiden_nimmt_den_rahmen_ueber_alles(tmp_path):
    """Ein Strich mitten auf der Flaeche, dann [Zuschneiden]: uebrig bleibt
    genau sein Rahmen, und kein Punkt geht dabei verloren."""
    geo = _lauf(tmp_path, 6)[-1]
    strich = [_mitte(geo["ox"], geo["oy"], geo["zoom"], x, 10) for x in range(10, 21)]
    ev = [(0, MOUSE_POSITION) + strich[0]] + _zug(1, strich)
    t = 1 + len(strich) + 2
    ev += _knopf_klick(t, geo["zusX"], geo["zusY"])
    letzte = _lauf(tmp_path, t + 14, ev)[-1]
    assert (letzte["gw"], letzte["gh"]) == (11, 1), "elf Punkte breit, eine Zeile hoch"
    assert letzte["gemalt"] == 11, "kein Punkt darf beim Zuschneiden verloren gehen"


def test_zuschneiden_beachtet_auch_ausgeblendete_ebenen(tmp_path):
    """Der Fall, der die bequeme Loesung verbietet: was auf einer
    ausgeblendeten Ebene liegt, darf nicht weggeschnitten werden -- sonst
    ist es beim Wiedereinblenden weg, und man weiss nicht warum.

    Aufbau: auf Ebene 1 ein kurzer Strich links, dann eine zweite Ebene mit
    einem Strich weit rechts, diese ausblenden, zuschneiden. Die Breite muss
    BEIDE umfassen.
    """
    geo = _lauf(tmp_path, 6)[-1]

    def strich(y, von, bis):
        return [_mitte(geo["ox"], geo["oy"], geo["zoom"], x, y) for x in range(von, bis)]

    ev = [(0, MOUSE_POSITION) + strich(10, 4, 8)[0]] + _zug(1, strich(10, 4, 8))
    t = 1 + 4 + 2
    ev += _knopf_klick(t, geo["ebNeuX"], geo["ebNeuY"])      # zweite Ebene
    t += 6
    ev += _zug(t, strich(20, 24, 28))                        # weit rechts unten
    t += 4 + 2
    ev += _klick(t, geo["ebSichtX"] + 8, geo["ebSichtY"] + 8)  # ausblenden
    t += 6
    ev += _knopf_klick(t, geo["zusX"], geo["zusY"])
    letzte = _lauf(tmp_path, t + 14, ev)[-1]
    assert (letzte["gw"], letzte["gh"]) == (24, 11), "von x=4 bis x=27, y=10 bis y=20"
    assert letzte["gemalt"] == 4, "die aktive (ausgeblendete) Ebene hat vier Punkte"


def test_groesse_aendern_haengt_durchsichtigen_rand_an(tmp_path):
    """Vergroessern ist derselbe Aufruf wie Zuschneiden, nur mit positivem
    Rand. Geprueft, weil die Richtung die andere ist: der Inhalt bleibt
    links oben, der Rest wird durchsichtig -- nicht schwarz.

    Die beiden Zahlen kommen aus einer Ersetzung: die Regler mit
    aufgezeichneten Klicks zu verstellen waere ein Dutzend Klicks auf einen
    Pfeil. Ersetzt ist nur das Ablesen, gerechnet hat `groesseAendern`.
    """
    geo = _lauf(tmp_path, 6)[-1]
    strich = [_mitte(geo["ox"], geo["oy"], geo["zoom"], x, 3) for x in range(2, 9)]
    ev = [(0, MOUSE_POSITION) + strich[0]] + _zug(1, strich)
    t = 1 + len(strich) + 2
    # Das Fenster bekommt eine feste Stelle und keinen Rahmen: sonst haengt
    # die Lage seines Knopfes an der Titelhoehe des Themas, und die kann ein
    # Programm nicht erfragen. Nur Aussehen und Ort, keine Logik.
    ersatz = {
        "GUI_WINDOW_VISIBLE(winGr, FALSE)\nDIM grOffen":
            "GUI_WINDOW_CHROME(winGr, FALSE)\n"
            "GUI_WINDOW_SET_BOUNDS(winGr, 100, 100, 300, 170)\n"
            "GUI_WINDOW_VISIBLE(winGr, FALSE)\nDIM grOffen",
        "DIM ngb AS INTEGER : ngb = INT(GUI_VALUE(spGrB))": "DIM ngb AS INTEGER : ngb = 48",
        "DIM ngh AS INTEGER : ngh = INT(GUI_VALUE(spGrH))": "DIM ngh AS INTEGER : ngh = 40",
    }
    grOk = _lauf(tmp_path, 6, dialoge=ersatz)[-1]
    ev += _knopf_klick(t, geo["grX"], geo["grY"])
    t += 6
    ev += _knopf_klick(t, grOk["grOkX"], grOk["grOkY"])
    letzte = _lauf(tmp_path, t + 14, ev, dialoge=ersatz)[-1]
    assert (letzte["gw"], letzte["gh"]) == (48, 40)
    assert letzte["gemalt"] == 7, "der Inhalt bleibt, der neue Rand ist durchsichtig"


# ------------------------------------------------------ Animationsbereiche
def test_bereich_anlegen_und_wieder_entfernen(tmp_path):
    """[Hinzu] uebernimmt, was in den Feldern steht -- die Vorgaben sind
    von=1, bis=1, fps=8, nach innen also 0/0/8."""
    geo = _lauf(tmp_path, 6)[-1]
    ev = _knopf_klick(2, geo["animNX"], geo["animNY"])
    proben = _lauf(tmp_path, 20, ev)
    nach_hinzu = proben[-1]
    assert nach_hinzu["anzAnim"] == 1
    assert nach_hinzu["aktAnim"] == 0
    assert (nach_hinzu["anVon"], nach_hinzu["anBis"], nach_hinzu["anFps"]) == (0, 0, 8)

    ev += _knopf_klick(10, geo["animWX"], geo["animWY"])
    letzte = _lauf(tmp_path, 24, ev)[-1]
    assert letzte["anzAnim"] == 0, "[Weg] nimmt ihn wieder heraus"
    assert letzte["aktAnim"] == -1


def test_bereiche_wandern_beim_loeschen_eines_bildes(tmp_path):
    """Die Stelle, an der eine Bereichsliste still falsch wird. Nach dem
    Loeschen von Bild 1 muss ein Bereich, der Bild 2..3 umfasste, auf 1..2
    zeigen -- sonst spielt die Vorschau danach etwas anderes, ohne dass sich
    sichtbar etwas geaendert haette.
    """
    # Der Bereich soll Bild 2..3 umfassen. Die beiden Regler mit
    # aufgezeichneten Klicks zu verstellen waere ein Dutzend Klicks auf einen
    # Pfeil -- ersetzt ist nur ihr Ablesen, alles danach ist der echte Code.
    ersatz = {
        "DIM v AS INTEGER : v = INT(GUI_VALUE(spVon)) - 1": "DIM v AS INTEGER : v = 1",
        "DIM b AS INTEGER : b = INT(GUI_VALUE(spBis)) - 1": "DIM b AS INTEGER : b = 2",
    }
    geo = _lauf(tmp_path, 6, dialoge=ersatz)[-1]
    # Drei Bilder: zweimal [Kopie]. Danach steht Bild 3 zur Bearbeitung.
    ev = _knopf_klick(2, geo["kopieX"], geo["kopieY"])
    ev += _knopf_klick(8, geo["kopieX"], geo["kopieY"])
    ev += _knopf_klick(14, geo["animNX"], geo["animNY"])
    mitte = _lauf(tmp_path, 22, ev, dialoge=ersatz)[-1]
    assert mitte["anzBild"] == 3 and mitte["anzAnim"] == 1
    assert (mitte["anVon"], mitte["anBis"]) == (1, 2), "Bild 2..3, nach innen 1..2"

    # Bild 1 in der Liste waehlen und loeschen. Der Bereich lag DAHINTER --
    # also muss er um eins nach vorn ruecken, auf 0..1.
    ev += _klick(24, geo["lstBX"] + 40, geo["lstBY"] + 10)   # erste Zeile
    ev += _knopf_klick(32, geo["bildWX"], geo["bildWY"])
    letzte = _lauf(tmp_path, 42, ev, dialoge=ersatz)[-1]
    assert letzte["anzBild"] == 2
    assert letzte["anzAnim"] == 1, "der Bereich bleibt -- er hing nicht am Bild 1"
    assert (letzte["anVon"], letzte["anBis"]) == (0, 1), "um eins nach vorn"


# ------------------------------------------------------------- GB-Code
_DIALOG_GB = 'FILE_SAVE_DIALOG("GB-Code sichern", "sprite.dh", "dh")'


def test_gb_code_laeuft_wirklich(tmp_path):
    """Der Kreis schliesst sich: der Editor schreibt ein Programm, und der
    Test STARTET es. Das ist die einzige Pruefung, die etwas ueber erzeugten
    Code wirklich aussagt -- ob er uebersetzt, sagt noch nichts darueber, ob
    er ein Blatt findet und eine Animation kennt.

    Aufbau: ein Strich, ein zweites Bild, ein Bereich ueber beide -- dann
    [GB-Code]. Erwartet werden zwei Dateien und ein sauberer Lauf.
    """
    geo = _lauf(tmp_path, 6)[-1]
    strich = [_mitte(geo["ox"], geo["oy"], geo["zoom"], x, 10) for x in range(6, 18)]
    ev = [(0, MOUSE_POSITION) + strich[0]] + _zug(1, strich)
    t = 1 + len(strich) + 2
    ev += _knopf_klick(t, geo["kopieX"], geo["kopieY"])       # zweites Bild
    t += 6
    ev += _knopf_klick(t, geo["animNX"], geo["animNY"])       # ein Bereich
    t += 6
    ev += _knopf_klick(t, geo["gbX"], geo["gbY"])
    _lauf(tmp_path, t + 14, ev, dialoge={_DIALOG_GB: '"raus.dh"'})

    code = tmp_path / "raus.dh"
    blatt = tmp_path / "raus.png"
    assert code.exists() and blatt.exists(), "Code UND Blatt, sonst zeigt er ins Leere"
    text = code.read_text(encoding="utf-8")
    assert 'SPRITE_NEW(blatt, 32, 32)' in text
    assert re.search(r'SPRITE_ADD_ANIM\(sp, "\w+", 0, 0, 8\)', text), text

    # Und jetzt laufen lassen -- headless, ein paar Bilder.
    r = subprocess.run([str(_DHRT), "run", str(code)], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", timeout=120, cwd=str(tmp_path),
                       env=dict(os.environ, DHRT_FRAMES="5"))
    assert r.returncode == 0, r.stderr


def test_gb_code_ohne_bereiche_spielt_trotzdem(tmp_path):
    """Ohne eigene Bereiche gehoert einer ueber alles hinein -- sonst
    uebersetzt das Programm zwar, aber SPRITE_PLAY findet nichts."""
    geo = _lauf(tmp_path, 6)[-1]
    ev = _knopf_klick(2, geo["kopieX"], geo["kopieY"])
    ev += _knopf_klick(10, geo["gbX"], geo["gbY"])
    _lauf(tmp_path, 26, ev, dialoge={_DIALOG_GB: '"raus.dh"'})
    text = (tmp_path / "raus.dh").read_text(encoding="utf-8")
    assert 'SPRITE_ADD_ANIM(sp, "idle", 0, 1, 8)' in text, text
    assert 'SPRITE_PLAY(sp, "idle")' in text
    r = subprocess.run([str(_DHRT), "run", str(tmp_path / "raus.dh")],
                       capture_output=True, text=True, encoding="utf-8", errors="replace",
                       timeout=120, cwd=str(tmp_path), env=dict(os.environ, DHRT_FRAMES="5"))
    assert r.returncode == 0, r.stderr


# -------------------------------------------------------------- dhanim
_DIALOG_FSM = 'FILE_SAVE_DIALOG("Zustandsmaschine sichern", "sprite.dhanim", "dhanim")'

# Ein Programm, das die erzeugte Maschine WIRKLICH laedt -- geprueft wird mit
# dem Leser der Laufzeit, nicht mit einem zweiten eigenen.
_FSM_PRUEFER = '''IMPORT "animfsm"
IMPORT "sprite"
SCREEN(320, 240, "Pruefer", 1)
DIM sp AS SPRITE
sp = SPRITE_NEW(LOADIMAGE("raus.png"), 32, 32)
DIM f AS ANIM_FSM
f = ANIM_FSM_LOAD("raus.dhanim")
ANIM_FSM_SETUP(f, sp)
ANIM_FSM_UPDATE(f, sp, 16)
PRINT "ZUSTAND " + ANIM_FSM_STATE(f)
'''


def _fsm_pruefen(tmp_path):
    """Die Maschine laden und einen Schritt fahren. Liefert den Zustandsnamen."""
    (tmp_path / "pruef.dh").write_text(_FSM_PRUEFER, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(tmp_path / "pruef.dh")],
                       capture_output=True, text=True, encoding="utf-8", errors="replace",
                       timeout=120, cwd=str(tmp_path), env=dict(os.environ, DHRT_FRAMES="3"))
    assert r.returncode == 0, r.stderr
    zeilen = [ln for ln in (r.stdout or "").splitlines() if ln.startswith("ZUSTAND ")]
    assert zeilen, r.stdout + r.stderr
    return zeilen[-1].split(None, 1)[1]


def test_dhanim_wird_von_der_laufzeit_geladen(tmp_path):
    """Der Kreis wie beim GB-Code, nur mit dem anderen Leser: `ANIM_FSM_LOAD`
    der Laufzeit macht die Datei auf, `ANIM_FSM_SETUP` traegt die Zustaende
    als Sprite-Animationen ein, und ein Schritt sagt, in welchem Zustand die
    Maschine steht. Dass die JSON gueltig ist, waere die schwaechere Aussage.
    """
    geo = _lauf(tmp_path, 6)[-1]
    ev = _knopf_klick(2, geo["kopieX"], geo["kopieY"])       # zweites Bild
    ev += _knopf_klick(8, geo["animNX"], geo["animNY"])      # ein Bereich
    ev += _knopf_klick(14, geo["gbX"], geo["gbY"])           # Blatt fuers Sprite
    ev += _knopf_klick(20, geo["faX"], geo["faY"])
    _lauf(tmp_path, 34, ev, dialoge={_DIALOG_GB: '"raus.dh"',
                                     _DIALOG_FSM: '"raus.dhanim"'})
    datei = tmp_path / "raus.dhanim"
    assert datei.exists()
    d = json.loads(datei.read_text(encoding="utf-8"))
    assert d["version"] == 1
    assert d["params"] == [] and d["transitions"] == [], "Vorlage, keine Erfindung"
    assert len(d["states"]) == 1
    z = d["states"][0]
    assert (z["first"], z["last"], z["fps"], z["loop"]) == (0, 0, 8, True)
    assert d["default"] == z["name"]
    assert _fsm_pruefen(tmp_path) == z["name"]


def test_dhanim_ohne_bereiche_bekommt_einen_zustand_ueber_alles(tmp_path):
    """Eine Maschine ohne Zustand laedt zwar, hat aber nichts zu spielen --
    also einer ueber alle Bilder, wie beim GB-Code auch."""
    geo = _lauf(tmp_path, 6)[-1]
    ev = _knopf_klick(2, geo["kopieX"], geo["kopieY"])
    ev += _knopf_klick(8, geo["gbX"], geo["gbY"])
    ev += _knopf_klick(14, geo["faX"], geo["faY"])
    _lauf(tmp_path, 28, ev, dialoge={_DIALOG_GB: '"raus.dh"',
                                     _DIALOG_FSM: '"raus.dhanim"'})
    d = json.loads((tmp_path / "raus.dhanim").read_text(encoding="utf-8"))
    assert [z["name"] for z in d["states"]] == ["idle"]
    assert (d["states"][0]["first"], d["states"][0]["last"]) == (0, 1)
    assert _fsm_pruefen(tmp_path) == "idle"


# --------------------------------------------------- Namen der Einzelbilder
_DIALOG_BLATT = 'FILE_SAVE_DIALOG("Streifen sichern", "sprites.png", "png")'
_NAME_LESEN = 'DIM nn AS STRING : nn = alsName$(GUI_TEXT(tiBild))'

# Ein Programm, das den erzeugten Atlas mit dem Leser der LAUFZEIT aufmacht.
# Ein Schluessel, den es nicht gibt, laesst ATLAS_DRAW scheitern -- damit ist
# der Rueckgabewert des Prozesses die ganze Pruefung.
_ATLAS_PRUEFER = '''SCREEN(320, 240, "Pruefer", 1)
DIM a AS SPRITE_ATLAS
a = ATLAS_LOAD("raus.json")
ATLAS_DRAW(a, "%s", 10, 10)
PRINT "GEZEICHNET"
'''


def _atlas_pruefen(tmp_path, schluessel):
    (tmp_path / "atl.dh").write_text(_ATLAS_PRUEFER % schluessel, encoding="utf-8")
    return subprocess.run([str(_DHRT), "run", str(tmp_path / "atl.dh")],
                          capture_output=True, text=True, encoding="utf-8",
                          errors="replace", timeout=120, cwd=str(tmp_path),
                          env=dict(os.environ, DHRT_FRAMES="3"))


# Das Namensfenster bekommt eine feste Stelle und keinen Rahmen -- sonst
# haengt die Lage seines Knopfes an der Titelhoehe des Themas, und die kann
# ein Programm nicht erfragen (dieselbe Naht wie beim Groessen-Fenster).
_FENSTER_FLACH = {
    "GUI_WINDOW_VISIBLE(winName, FALSE)\nDIM namOffen":
        "GUI_WINDOW_CHROME(winName, FALSE)\n"
        "GUI_WINDOW_SET_BOUNDS(winName, 100, 100, 340, 140)\n"
        "GUI_WINDOW_VISIBLE(winName, FALSE)\nDIM namOffen",
}


def _benennen(tmp_path, name):
    """Bild 1 benennen und den Streifen samt Atlas schreiben."""
    ersatz = dict(_FENSTER_FLACH)
    ersatz[_NAME_LESEN] = 'DIM nn AS STRING : nn = alsName$("%s")' % name
    ersatz[_DIALOG_BLATT] = '"raus.png"' 
    geo = _lauf(tmp_path, 6, dialoge=ersatz)[-1]
    ev = _knopf_klick(2, geo["namX"], geo["namY"])
    ev += _knopf_klick(10, geo["namOkX"], geo["namOkY"])
    ev += _knopf_klick(18, geo["blattX"], geo["blattY"])
    return _lauf(tmp_path, 34, ev, dialoge=ersatz)[-1]


def test_benanntes_bild_wird_zum_atlas_schluessel(tmp_path):
    """Der Zweck der Namen: ein Programm schreibt spaeter
    `ATLAS_DRAW(atlas, "kopf", ...)` statt `"bild_0"`. Geprueft wird deshalb
    mit ATLAS_LOAD der Laufzeit -- ein Schluessel, den es nicht gibt, laesst
    das Zeichnen scheitern.
    """
    _benennen(tmp_path, "kopf")
    assert (tmp_path / "raus.json").exists()
    d = json.loads((tmp_path / "raus.json").read_text(encoding="utf-8"))
    assert list(d["sprites"]) == ["kopf"], d
    assert _atlas_pruefen(tmp_path, "kopf").returncode == 0
    # Gegenprobe: der alte Vorgabe-Schluessel darf es NICHT mehr geben,
    # sonst sagt der Test oben nur, dass irgendein Eintrag existiert.
    r = _atlas_pruefen(tmp_path, "bild_0")
    assert r.returncode != 0, r.stdout


def test_name_wird_auf_eine_kennung_gebracht(tmp_path):
    """Ein Punkt im Namen ist die gefaehrlichste Eingabe: das json-Modul
    liest einen Schluessel mit Punkt als PFAD, aus "held.lauf" wuerde ein
    verschachteltes Objekt statt eines Eintrags -- und `streifenLaden` sucht
    spaeter mit derselben Punkt-Notation. Deshalb wird gesaeubert."""
    _benennen(tmp_path, "held.lauf 2!")
    d = json.loads((tmp_path / "raus.json").read_text(encoding="utf-8"))
    assert list(d["sprites"]) == ["held_lauf_2_"], d
    assert _atlas_pruefen(tmp_path, "held_lauf_2_").returncode == 0


def test_namen_kommen_ueber_den_streifen_zurueck(tmp_path):
    """Rundweg: benennen, Streifen schreiben, denselben Streifen wieder
    aufmachen -- der Name muss wieder da sein. Ohne das verliert jeder Weg
    ueber den Streifen die Namen, und man merkt es erst, wenn der naechste
    Export wieder `bild_0` schreibt."""
    ersatz = dict(_FENSTER_FLACH)
    ersatz[_NAME_LESEN] = 'DIM nn AS STRING : nn = alsName$("kopf")'
    ersatz[_DIALOG_BLATT] = '"raus.png"'
    ersatz['po = FILE_OPEN_DIALOG("Oeffnen", "dhsprite,png")'] = 'po = "raus.png"'
    geo = _lauf(tmp_path, 6, dialoge=ersatz)[-1]
    ev = _knopf_klick(2, geo["namX"], geo["namY"])
    ev += _knopf_klick(10, geo["namOkX"], geo["namOkY"])
    ev += _knopf_klick(18, geo["blattX"], geo["blattY"])
    nach_export = _lauf(tmp_path, 30, ev, dialoge=ersatz)[-1]
    assert nach_export["namLen"] == 4, "der Name steht"

    ev += _knopf_klick(32, 66 - 64, 7 - 14)      # [Oeffnen] in der Werkzeugleiste
    letzte = _lauf(tmp_path, 50, ev, dialoge=ersatz)[-1]
    assert letzte["anzBild"] == 1, "der Streifen ist wieder drin"
    assert letzte["namLen"] == 4, "und sein Name mit ihm"


# --------------------------------------------------- Spiegeln und Drehen
# Vier Wandlungen ueber ALLE Bilder und Ebenen. Die Vierteldrehung tauscht
# Breite und Hoehe -- und die gehoeren dem ganzen Sprite, nicht einem Bild.
# Geprueft wird an den vier ECKEN der ersten Ebene (Deckkraft je Ecke steht
# in der Probe): ein gesetzter Punkt muss nach der Wandlung an genau einer
# anderen Ecke liegen, nicht verschwinden.

def _ecken(probe):
    return (probe["eckLO"], probe["eckRO"], probe["eckRU"], probe["eckLU"])


def _punkt_links_oben(ox, oy, zoom, frame=4):
    """Einen Punkt in die linke obere Ecke malen."""
    return _zug(frame, [_mitte(ox, oy, zoom, 0, 0)] * 2)


def _kleines_sprite(tmp_path):
    """Ein NICHT quadratisches Sprite -- sonst faellt ein Groessentausch
    gar nicht auf. Der Dialog wird durch feste Werte ersetzt."""
    return {'INT(GUI_VALUE(spB))': "24", 'INT(GUI_VALUE(spH))': "12"}


def test_drehen_tauscht_breite_und_hoehe(tmp_path):
    geo = _lauf(tmp_path, 6)[-1]
    ev = _knopf_klick(4, geo["drRX"], geo["drRY"], 62)
    letzte = _lauf(tmp_path, 24, ev)[-1]
    assert (letzte["gw"], letzte["gh"]) == (geo["gh"], geo["gw"]), \
        "Breite und Hoehe muessen tauschen"


def test_die_rechtsdrehung_setzt_den_punkt_nach_rechts_oben(tmp_path):
    """Die eigentliche Zusage: der Punkt geht nicht verloren. Mit
    `IMAGE_ROTATE(bild, 90.0)` waere er weg -- genau dafuer gibt es
    `IMAGE_ROTATE_CW` (siehe tests/test_image_rotate_exakt.py)."""
    ox, oy, zoom = _geometrie(tmp_path)
    geo = _lauf(tmp_path, 6)[-1]
    ev = _punkt_links_oben(ox, oy, zoom)
    vorher = _lauf(tmp_path, 16, ev)[-1]
    assert _ecken(vorher)[0] > 0, "der Punkt liegt erst einmal links oben"
    ev += _knopf_klick(18, geo["drRX"], geo["drRY"], 62)
    letzte = _lauf(tmp_path, 34, ev)[-1]
    assert _ecken(letzte) == (0, vorher["eckLO"], 0, 0), \
        "links oben -> rechts oben, und sonst nichts: %s" % (_ecken(letzte),)


def test_die_linksdrehung_geht_in_die_andere_richtung(tmp_path):
    ox, oy, zoom = _geometrie(tmp_path)
    geo = _lauf(tmp_path, 6)[-1]
    ev = _punkt_links_oben(ox, oy, zoom)
    vorher = _lauf(tmp_path, 16, ev)[-1]
    ev += _knopf_klick(18, geo["drLX"], geo["drLY"], 62)
    letzte = _lauf(tmp_path, 34, ev)[-1]
    assert _ecken(letzte) == (0, 0, 0, vorher["eckLO"]), \
        "links oben -> links unten: %s" % (_ecken(letzte),)


def test_waagerecht_spiegeln_laesst_die_masse_stehen(tmp_path):
    ox, oy, zoom = _geometrie(tmp_path)
    geo = _lauf(tmp_path, 6)[-1]
    ev = _punkt_links_oben(ox, oy, zoom)
    vorher = _lauf(tmp_path, 16, ev)[-1]
    ev += _knopf_klick(18, geo["spXX"], geo["spXY"], 62)
    letzte = _lauf(tmp_path, 34, ev)[-1]
    assert (letzte["gw"], letzte["gh"]) == (vorher["gw"], vorher["gh"])
    assert _ecken(letzte) == (0, vorher["eckLO"], 0, 0), \
        "links oben -> rechts oben: %s" % (_ecken(letzte),)


def test_senkrecht_spiegeln_geht_nach_unten(tmp_path):
    ox, oy, zoom = _geometrie(tmp_path)
    geo = _lauf(tmp_path, 6)[-1]
    ev = _punkt_links_oben(ox, oy, zoom)
    vorher = _lauf(tmp_path, 16, ev)[-1]
    ev += _knopf_klick(18, geo["spYX"], geo["spYY"], 62)
    letzte = _lauf(tmp_path, 34, ev)[-1]
    assert _ecken(letzte) == (0, 0, 0, vorher["eckLO"]), \
        "links oben -> links unten: %s" % (_ecken(letzte),)


def test_zweimal_drehen_und_zurueck_gibt_das_bild_wieder(tmp_path):
    """Vierteldrehungen sind verlustfrei -- rechts und wieder links muss
    denselben Punkt an derselben Stelle ergeben."""
    ox, oy, zoom = _geometrie(tmp_path)
    geo = _lauf(tmp_path, 6)[-1]
    ev = _punkt_links_oben(ox, oy, zoom)
    vorher = _lauf(tmp_path, 16, ev)[-1]
    ev += _knopf_klick(18, geo["drRX"], geo["drRY"], 62)
    ev += _knopf_klick(26, geo["drLX"], geo["drLY"], 62)
    letzte = _lauf(tmp_path, 42, ev)[-1]
    assert (letzte["gw"], letzte["gh"]) == (vorher["gw"], vorher["gh"])
    assert _ecken(letzte) == _ecken(vorher), "der Punkt steht wieder links oben"
    assert letzte["gemalt"] == vorher["gemalt"], "und es ist kein Punkt dazugekommen"


def test_eine_wandlung_leert_den_verlauf(tmp_path):
    """Ein aufgezeichneter Schritt haelt das Bild EINER Ebene. Nach einer
    Wandlung ueber alle waere ein Rueckgaengig darauf halbseitig: es
    spiegelte genau diese eine wieder zurueck, den Rest nicht -- und man
    saehe es erst beim Umschalten der Ebene.
    """
    ox, oy, zoom = _geometrie(tmp_path)
    geo = _lauf(tmp_path, 6)[-1]
    ev = _punkt_links_oben(ox, oy, zoom)
    vorher = _lauf(tmp_path, 16, ev)[-1]
    assert vorher["uAnz"] > 0, "der Strich steht im Verlauf"
    ev += _knopf_klick(18, geo["spXX"], geo["spXY"], 62)
    letzte = _lauf(tmp_path, 34, ev)[-1]
    assert letzte["uAnz"] == 0, "danach ist er leer"


# ------------------------------------------------- Dauer je Einzelbild
# GIF kann eine Dauer JE BILD -- eine Pose wird gehalten, der Lauf dazwischen
# nicht. Der Editor haelt sie in `bildMs` (0 = der Tempo-Regler gilt); die
# Vorschau und die GIF-Ausgabe fragen dieselbe Stelle (`dauerMs`), sonst
# liefe das Bild anders als die Datei.
_DIALOG_GIF = 'FILE_SAVE_DIALOG("Bewegtes GIF sichern", "sprite.gif", "gif")'


def _drehfeld_hoch(frame, geo, mal):
    """Auf den oberen Pfeil eines Drehfelds klicken -- er sitzt rechts,
    obere Haelfte. Ein Klick je Schritt, mit Abstand dazwischen."""
    ev = []
    for i in range(mal):
        ev += _klick(frame + i * 3, geo["msX"] + geo["msW"] - 8, geo["msY"] + 6)
    return ev


def test_ohne_eigene_dauer_gilt_der_tempo_regler(tmp_path):
    """Die Ausgangslage -- und die Gegenprobe zu allem darunter: 0 heisst
    NICHT 0 ms, sondern "der Regler gilt"."""
    letzte = _lauf(tmp_path, 8)[-1]
    assert letzte["ms0"] == 0
    # Der Regler steht auf 8 Bildern je Sekunde -> 125 ms.
    assert letzte["dauer0"] == 125


def test_das_drehfeld_setzt_die_dauer_des_gewaehlten_bildes(tmp_path):
    geo = _lauf(tmp_path, 8)[-1]
    letzte = _lauf(tmp_path, 30, _drehfeld_hoch(4, geo, 3))[-1]
    assert letzte["ms0"] > 0, "die Dauer haengt am ersten Bild"
    assert letzte["dauer0"] == letzte["ms0"], "und sie gewinnt gegen den Regler"


def test_die_eigenen_zeiten_stehen_im_gif(tmp_path):
    """Der ganze Weg bis in die Datei, gelesen von Pillow -- einem FREMDEN
    Leser. Dass die eigene Datei die eigenen Zahlen enthaelt, waere die
    schwaechere Aussage.

    Bild 1 bekommt eine eigene Dauer, Bild 2 nicht: im GIF muessen sich die
    beiden Zeiten deshalb UNTERSCHEIDEN. Waeren beide gleich, wuerde der
    Test auch dann bestehen, wenn die Einzelzeit gar nicht ankommt.
    """
    pytest.importorskip("PIL")
    geo = _lauf(tmp_path, 8)[-1]
    ev = _knopf_klick(4, geo["bildWX"] - 92, geo["bildWY"], 88)   # [Kopie] -> 2 Bilder
    ev += _drehfeld_hoch(12, geo, 4)                              # Bild 2 bekommt Zeit
    ev += _knopf_klick(30, geo["gifX"], geo["gifY"], 52)
    _lauf(tmp_path, 52, ev, dialoge={_DIALOG_GIF: '"raus.gif"'})

    from PIL import Image, ImageSequence
    im = Image.open(tmp_path / "raus.gif")
    dauern = [f.info.get("duration") for f in ImageSequence.Iterator(im)]
    assert len(dauern) == 2, "zwei Einzelbilder"
    assert dauern[0] != dauern[1], "eins traegt seine eigene Zeit: %s" % dauern
    # 8 Bilder/s = 125 ms; GIF rechnet in Hundertstelsekunden, 13 davon
    # kommen als 130 ms zurueck.
    assert 130 in dauern, "das andere folgt dem Tempo-Regler (8/s): %s" % dauern


def test_die_dauer_uebersteht_die_eigene_datei(tmp_path):
    """Rundweg durch `.dhsprite`: setzen, sichern, neu anfangen, laden."""
    # Der Kasten "Neues Sprite" bekommt einen festen Platz OHNE Rahmen --
    # sonst laegen seine Widget-Koordinaten um die Titelhoehe neben dem, was
    # die Maus spricht. Dasselbe Mittel wie beim Groesse-aendern-Test.
    ersatz = {
        "GUI_WINDOW_VISIBLE(winNeu, FALSE)\nDIM neuOffen":
            "GUI_WINDOW_CHROME(winNeu, FALSE)\n"
            "GUI_WINDOW_SET_BOUNDS(winNeu, 100, 100, 300, 170)\n"
            "GUI_WINDOW_VISIBLE(winNeu, FALSE)\nDIM neuOffen",
        'FILE_SAVE_DIALOG("Sichern", "sprite.dhsprite", "dhsprite")': '"r.dhsprite"',
        'FILE_OPEN_DIALOG("Oeffnen", "dhsprite,png")': '"r.dhsprite"',
    }
    geo = _lauf(tmp_path, 8, dialoge=ersatz)[-1]
    ev = _drehfeld_hoch(4, geo, 5)
    gesetzt = _lauf(tmp_path, 26, ev, dialoge=ersatz)[-1]["ms0"]
    assert gesetzt > 0
    # Die Knopf-Lagen kommen aus der PROBE, nicht aus einer Rechnung. Der
    # erste Versuch rechnete [Sichern] aus der Lage von [Streifen] aus und
    # traf damit [PNG] -- das oeffnet einen NATIVEN Dateidialog, den keine
    # aufgezeichnete Eingabe erreicht: der Lauf hing 180 Sekunden lang.
    ev += _knopf_klick(24, geo["sichX"], geo["kopfY"], geo["sichW"])
    ev += _knopf_klick(34, geo["neuX"], geo["kopfY"], geo["neuW"])
    # [Neu] oeffnet einen Kasten und wartet auf sein [Anlegen]. Ohne diesen
    # Klick passierte GAR NICHTS -- und der Test war trotzdem gruen, weil der
    # Wert dann eben nie zurueckgesetzt wurde. Deshalb steht unten die
    # Zwischenpruefung: erst 0, dann wieder da.
    ev += _knopf_klick(44, geo["neuOkX"], geo["neuOkY"], 90)
    leer = _lauf(tmp_path, 62, ev, dialoge=ersatz)[-1]
    assert leer["ms0"] == 0, "ein neues Sprite hat keine eigenen Zeiten"
    ev += _knopf_klick(64, geo["oeffX"], geo["kopfY"], geo["oeffW"])
    letzte = _lauf(tmp_path, 90, ev, dialoge=ersatz)[-1]
    assert letzte["ms0"] == gesetzt, "die Dauer kam zurueck"


def test_eine_datei_ohne_namen_verliert_ihre_bereiche_nicht(tmp_path):
    """Ein Fund, der aelter ist als die Einzelbild-Dauern: `JSON_LEN` WIRFT
    bei einem Pfad, den es nicht gibt -- es liefert nicht 0.

    `bildnamen`, `bilddauern` und `bereiche` stehen alle nur dann in der
    `.dhsprite`, wenn es sie gibt. Ohne `JSON_HAS` brach das Laden an der
    ersten fehlenden ab: die Ebenen standen schon, alles dahinter kam nie
    an. Eine Datei MIT Bereich, aber OHNE Namen verlor also beim Laden
    stillschweigend ihren Bereich -- gemeldet wurde nur eine Pfad-Meldung,
    die nach einem Programmierfehler im Editor aussieht.
    """
    ersatz = {
        "GUI_WINDOW_VISIBLE(winNeu, FALSE)\nDIM neuOffen":
            "GUI_WINDOW_CHROME(winNeu, FALSE)\n"
            "GUI_WINDOW_SET_BOUNDS(winNeu, 100, 100, 300, 170)\n"
            "GUI_WINDOW_VISIBLE(winNeu, FALSE)\nDIM neuOffen",
        'FILE_SAVE_DIALOG("Sichern", "sprite.dhsprite", "dhsprite")': '"o.dhsprite"',
        'FILE_OPEN_DIALOG("Oeffnen", "dhsprite,png")': '"o.dhsprite"',
    }
    geo = _lauf(tmp_path, 8, dialoge=ersatz)[-1]
    # Einen Bereich anlegen (die Vorgaben genuegen), aber KEINEN Namen
    # vergeben -- dann fehlt `bildnamen` in der Datei.
    ev = _knopf_klick(4, geo["animNX"], geo["animNY"], 100)
    ev += _knopf_klick(14, geo["sichX"], geo["kopfY"], geo["sichW"])
    ev += _knopf_klick(24, geo["neuX"], geo["kopfY"], geo["neuW"])
    ev += _knopf_klick(34, geo["neuOkX"], geo["neuOkY"], 90)
    leer = _lauf(tmp_path, 52, ev, dialoge=ersatz)[-1]
    assert leer["anzAnim"] == 0, "ein neues Sprite hat keine Bereiche"
    ev += _knopf_klick(54, geo["oeffX"], geo["kopfY"], geo["oeffW"])
    letzte = _lauf(tmp_path, 80, ev, dialoge=ersatz)[-1]
    assert letzte["anzAnim"] == 1, "der Bereich kam zurueck"
