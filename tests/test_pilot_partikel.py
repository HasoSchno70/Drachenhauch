"""Der Partikel-Pilot, bedient wie von Hand (`examples/185_partikel_editor.dh`).

Abgedeckt ist das Rueckgaengig/Wiederholen -- und dabei genau die Frage, an
der ein Regler-Editor haengt: **wann ist ein Schritt fertig?** Ein gezogener
Regler aendert sich in JEDEM Bild; wer das aufzeichnet, hat nach einer
Sekunde 60 Schritte und nimmt mit Strg+Z ein Sechzigstel der Bewegung
zurueck. Der Test faehrt deshalb einen echten Zug (Taste halten, ziehen,
loslassen) und prueft, dass EIN Rueckgaengig ihn ganz zuruecknimmt.

Wie bei den anderen Piloten wird nichts an der Logik geaendert. Die Kopie
bekommt eine PRINT-Zeile je Bild, die nur BESTEHENDE Werte ausliest, und ein
`SET_WINDOW_POS` aus dem Bild heraus, damit der ECHTE Mauszeiger nicht
mitredet (raylib meldet seine Bewegung auch waehrend einer Wiedergabe).
"""
import os
import re
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent
_PILOT = _ROOT / "examples" / "185_partikel_editor.dh"


def _find_dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    return next((_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
                 for v in ("release", "debug")
                 if (_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe).exists()),
                None)


_DHRT = _find_dhrt()
pytestmark = [pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut"),
              pytest.mark.seriell]

KEY_UP, KEY_DOWN = 1, 2
MOUSE_BUTTON_UP, MOUSE_BUTTON_DOWN, MOUSE_POSITION = 5, 6, 7
TASTE_STRG = 341
TASTE_Z, TASTE_Y = 90, 89

# Die Lage der Knoepfe und des ersten Reglers, dazu der Stand des Verlaufs.
_PROBE = '''    PRINT "P " + STR$(GUI_GET_X(knopfZurueck)) + " " + STR$(GUI_GET_Y(knopfZurueck)) + _
          " " + STR$(GUI_GET_X(knopfVor)) + " " + STR$(GUI_GET_Y(knopfVor)) + _
          " " + STR$(GUI_GET_X(regler[P_GY])) + " " + STR$(GUI_GET_Y(regler[P_GY])) + _
          " " + STR$(GUI_GET_W(regler[P_GY])) + _
          " " + STR$(GUI_GET_X(liste)) + " " + STR$(GUI_GET_Y(liste)) + _
          " " + STR$(uPos) + " " + STR$(uAnz) + _
          " " + STR$(INT(p(P_GY))) + " " + STR$(INT(p(P_R1))) + _
          " " + STR$(GUI_LISTBOX_SELECTED(liste)) + _
          " " + STR$(GUI_CANVAS_X(flaeche) - GUI_GET_X(flaeche)) + _
          " " + STR$(GUI_CANVAS_Y(flaeche) - GUI_GET_Y(flaeche)) + _
          " " + STR$(GUI_GET_H(regler[P_GY])) + _
          " " + STR$(IIF(GUI_HIT_TEST(GUI_CANVAS_X(flaeche) - GUI_GET_X(flaeche) + _
                                      GUI_GET_X(liste) + 40, _
                                      GUI_CANVAS_Y(flaeche) - GUI_GET_Y(flaeche) + _
                                      GUI_GET_Y(liste) + 10) = liste, 1, 0))
'''
# Widget-Koordinaten sind fenster-relativ, die Maus spricht Bildschirm --
# der Versatz kommt aus der Zeichenflaeche, die beides liefert. Ihn zu
# RATEN (Titelhoehe!) waere der sichere Weg, jeden Klick danebenzusetzen.
_FELDER = ("zurX zurY vorX vorY gyX gyY gyW lstX lstY "
           "uPos uAnz gy r1 preset offX offY gyH ersteZeileFrei").split()


def _kopie(tmp_path):
    src = _PILOT.read_text(encoding="utf-8")
    # Das Fenster aus dem Bild schieben -- der echte Zeiger redet sonst mit.
    assert src.count('SETFPS(60)') == 1
    src = src.replace('SETFPS(60)', 'SETFPS(60)\nSET_WINDOW_POS(-3000, -3000)', 1)
    assert src.count("    FLIP()\nWEND") == 1
    src = src.replace("    FLIP()\nWEND", _PROBE + "    FLIP()\nWEND")
    ziel = tmp_path / "pilot.dh"
    ziel.write_text(src, encoding="utf-8")
    return ziel


def _events(tmp_path, events):
    events = sorted(events, key=lambda e: e[0])
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
    assert zeilen, (r.stdout, r.stderr)
    return [dict(zip(_FELDER, [int(v) for v in re.split(r"\s+", ln)[1:]])) for ln in zeilen]


def _klick(frame, x, y):
    return [(frame, MOUSE_POSITION, x, y),
            (frame + 1, MOUSE_POSITION, x, y),
            (frame + 1, MOUSE_BUTTON_DOWN, 0),
            (frame + 2, MOUSE_BUTTON_UP, 0)]


def _zug(frame, x1, y1, x2, y2, schritte=6):
    """Ein echter Zug: druecken, ueber mehrere Bilder ziehen, loslassen.

    Die Taste muss in JEDEM Bild gemeldet werden -- eine eingespeiste
    Eingabe gilt nur fuer ihr eigenes Bild, danach ueberschreibt die echte
    Abfrage sie wieder.
    """
    ev = [(frame, MOUSE_POSITION, x1, y1)]
    for i in range(schritte + 1):
        x = x1 + (x2 - x1) * i // schritte
        y = y1 + (y2 - y1) * i // schritte
        ev.append((frame + 1 + i, MOUSE_POSITION, x, y))
        ev.append((frame + 1 + i, MOUSE_BUTTON_DOWN, 0))
    ev.append((frame + 2 + schritte, MOUSE_POSITION, x2, y2))
    ev.append((frame + 2 + schritte, MOUSE_BUTTON_UP, 0))
    return ev


def _strg(frame, taste, dauer=4):
    """Strg gedrueckt halten und die Taste EINMAL antippen."""
    ev = [(frame + i, KEY_DOWN, TASTE_STRG) for i in range(dauer)]
    ev.append((frame + 1, KEY_DOWN, taste))
    return ev


def _schirm(geo, x, y):
    """Fenster-Koordinaten -> Bildschirm."""
    return geo["offX"] + x, geo["offY"] + y


def _reglermitte(geo):
    return _schirm(geo, geo["gyX"] + geo["gyW"] // 2, geo["gyY"] + geo["gyH"] // 2)


def test_am_anfang_gibt_es_nichts_zurueckzunehmen(tmp_path):
    """Der Anfangsstand ist Schritt 0 -- aber eben nur einer. Wenn hier schon
    zwei stuenden, haette die Werkseinstellung des Starts sich selbst als
    Aenderung aufgezeichnet."""
    letzte = _lauf(tmp_path, 12)[-1]
    assert letzte["uPos"] == 0
    assert letzte["uAnz"] == 1, "der Start ist EIN Stand, nicht zwei gleiche"


def test_ein_zug_am_regler_ist_ein_schritt(tmp_path):
    """Der Kern: ein gezogener Regler aendert sich in jedem Bild. Wuerde
    jedes davon aufgezeichnet, staenden hier ein Dutzend Schritte -- und ein
    Strg+Z naehme nur das letzte Stueckchen zurueck."""
    geo = _lauf(tmp_path, 12)[-1]
    mx, my = _reglermitte(geo)
    proben = _lauf(tmp_path, 40, _zug(6, mx, my, mx - 60, my))
    letzte = proben[-1]
    assert letzte["gy"] != geo["gy"], "der Regler hat sich ueberhaupt bewegt"
    assert letzte["uAnz"] == 2, "genau EIN Schritt fuer den ganzen Zug"
    assert letzte["uPos"] == 1


def test_zurueck_nimmt_den_ganzen_zug_zurueck(tmp_path):
    geo = _lauf(tmp_path, 12)[-1]
    mx, my = _reglermitte(geo)
    ev = _zug(6, mx, my, mx - 60, my)
    ev += _klick(24, *_schirm(geo, geo["zurX"] + 40, geo["zurY"] + 14))
    letzte = _lauf(tmp_path, 44, ev)[-1]
    assert letzte["gy"] == geo["gy"], "der Wert steht wieder wie vorher"
    assert letzte["uPos"] == 0


def test_und_vor_holt_ihn_wieder(tmp_path):
    geo = _lauf(tmp_path, 12)[-1]
    mx, my = _reglermitte(geo)
    ev = _zug(6, mx, my, mx - 60, my)
    nachZug = _lauf(tmp_path, 26, ev)[-1]
    ev += _klick(28, *_schirm(geo, geo["zurX"] + 40, geo["zurY"] + 14))
    ev += _klick(36, *_schirm(geo, geo["vorX"] + 30, geo["vorY"] + 14))
    letzte = _lauf(tmp_path, 56, ev)[-1]
    assert letzte["gy"] == nachZug["gy"], "der gezogene Wert ist zurueck"
    assert letzte["uPos"] == 1


def test_strg_z_und_strg_y_tun_dasselbe(tmp_path):
    """Ohne Tastenkuerzel muesste man fuer jedes Zuruecknehmen zur Maus --
    und ein totes Kuerzel sieht aus wie ein vergessener Aufruf."""
    geo = _lauf(tmp_path, 12)[-1]
    mx, my = _reglermitte(geo)
    ev = _zug(6, mx, my, mx - 60, my)
    ev += _strg(26, TASTE_Z)
    nachZurueck = _lauf(tmp_path, 34, ev)[-1]
    assert nachZurueck["gy"] == geo["gy"] and nachZurueck["uPos"] == 0
    ev += _strg(36, TASTE_Y)
    letzte = _lauf(tmp_path, 46, ev)[-1]
    assert letzte["gy"] != geo["gy"] and letzte["uPos"] == 1


def test_eine_werkseinstellung_ist_genau_ein_schritt(tmp_path):
    """Sie setzt 20 Werte auf einmal -- als 20 Schritte waere sie mit einem
    Strg+Z nicht mehr zurueckzunehmen."""
    geo = _lauf(tmp_path, 12)[-1]
    # Der dritte Eintrag der Liste ("Feuer"), Zeilenhoehe rund 22.
    ev = _klick(6, *_schirm(geo, geo["lstX"] + 60, geo["lstY"] + 55))
    nachPreset = _lauf(tmp_path, 30, ev)[-1]
    assert nachPreset["preset"] == 2, "die Werkseinstellung wurde ueberhaupt gewaehlt"
    assert nachPreset["uAnz"] == 2 and nachPreset["uPos"] == 1
    ev += _klick(32, *_schirm(geo, geo["zurX"] + 40, geo["zurY"] + 14))
    letzte = _lauf(tmp_path, 52, ev)[-1]
    assert letzte["gy"] == geo["gy"], "ein Zurueck reicht fuer die ganze Einstellung"
    assert letzte["r1"] == geo["r1"]


def test_ein_neuer_zug_schneidet_den_vor_weg_ab(tmp_path):
    """Ab hier geht die Geschichte anders weiter -- ein stehengebliebener
    Vor-Weg wuerde beim naechsten Klick einen fremden Stand herstellen."""
    geo = _lauf(tmp_path, 12)[-1]
    mx, my = _reglermitte(geo)
    ev = _zug(6, mx, my, mx - 60, my)
    ev += _klick(24, *_schirm(geo, geo["zurX"] + 40, geo["zurY"] + 14))
    ev += _zug(32, mx, my, mx + 40, my)
    letzte = _lauf(tmp_path, 60, ev)[-1]
    assert letzte["uPos"] == 1 and letzte["uAnz"] == 2, "der alte Vor-Weg ist weg"


def test_der_erste_eintrag_der_liste_ist_anklickbar(tmp_path):
    """Bis 2026-09-02 lagen [Sichern] und [Laden] genau auf den ersten
    beiden Eintraegen der Werkseinstellungs-Liste: "Funken" war verdeckt und
    fing seine Klicks nicht mehr selbst -- der Knopf darueber tat es.

    Gefragt wird `GUI_HIT_TEST`, nicht ein echter Klick: der wuerde beim
    alten Stand einen Datei-Dialog oeffnen und den Test haengen lassen.
    """
    letzte = _lauf(tmp_path, 12)[-1]
    assert letzte["ersteZeileFrei"] == 1, "ueber der ersten Zeile liegt ein Widget"
