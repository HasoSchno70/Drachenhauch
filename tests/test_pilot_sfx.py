"""Der SFX-Pilot, bedient wie von Hand (`examples/183_sfx_generator.dh`).

Abgedeckt ist sein Rueckgaengig/Wiederholen -- dieselbe Frage wie beim
Partikel-Piloten (`tests/test_pilot_partikel.py`): ein gezogener Regler
aendert sich in JEDEM Bild, ein Zug muss trotzdem EIN Schritt sein.

Dazu die Lage der Beschriftungen: bis 2026-09-02 standen "Bereit." und die
Dauer-Anzeige mitten auf [Sichern] und [Laden] -- lesbar war keins von
beidem, und gesehen hat es nur, wer HINSAH.

Wie bei den anderen Piloten wird an der Logik nichts geaendert. Die Kopie
bekommt eine PRINT-Zeile je Bild, die nur BESTEHENDE Werte ausliest, und
wird aus dem Bild geschoben, damit der ECHTE Mauszeiger nicht mitredet
(raylib meldet seine Bewegung auch waehrend einer Wiedergabe).
"""
import os
import re
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent
_PILOT = _ROOT / "examples" / "183_sfx_generator.dh"


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

# Widget-Koordinaten sind fenster-relativ, die Maus spricht Bildschirm -- der
# Versatz kommt aus der Zeichenflaeche, die beides liefert. Ihn zu RATEN
# (Titelhoehe!) waere der sichere Weg, jeden Klick danebenzusetzen.
_PROBE = '''    PRINT "P " + STR$(GUI_GET_X(knopfZurueck)) + " " + STR$(GUI_GET_Y(knopfZurueck)) + _
          " " + STR$(GUI_GET_X(knopfVor)) + " " + STR$(GUI_GET_Y(knopfVor)) + _
          " " + STR$(GUI_GET_X(regler[P_FREQ])) + " " + STR$(GUI_GET_Y(regler[P_FREQ])) + _
          " " + STR$(GUI_GET_W(regler[P_FREQ])) + " " + STR$(GUI_GET_H(regler[P_FREQ])) + _
          " " + STR$(GUI_GET_X(liste)) + " " + STR$(GUI_GET_Y(liste)) + _
          " " + STR$(GUI_GET_X(knopfZufall)) + " " + STR$(GUI_GET_Y(knopfZufall)) + _
          " " + STR$(uPos) + " " + STR$(uAnz) + _
          " " + STR$(INT(GUI_VALUE(regler[P_FREQ]))) + _
          " " + STR$(GUI_DROPDOWN_SELECTED(waehlWave)) + _
          " " + STR$(GUI_LISTBOX_SELECTED(liste)) + _
          " " + STR$(GUI_CANVAS_X(flaeche) - GUI_GET_X(flaeche)) + _
          " " + STR$(GUI_CANVAS_Y(flaeche) - GUI_GET_Y(flaeche)) + _
          " " + STR$(GUI_GET_X(lblStatus)) + " " + STR$(GUI_GET_Y(lblStatus)) + _
          " " + STR$(GUI_GET_W(lblStatus)) + " " + STR$(GUI_GET_H(lblStatus)) + _
          " " + STR$(GUI_GET_X(lblDauer)) + " " + STR$(GUI_GET_Y(lblDauer)) + _
          " " + STR$(GUI_GET_W(lblDauer)) + " " + STR$(GUI_GET_H(lblDauer)) + _
          " " + STR$(GUI_GET_X(knopfSichern)) + " " + STR$(GUI_GET_Y(knopfSichern)) + _
          " " + STR$(GUI_GET_W(knopfSichern)) + " " + STR$(GUI_GET_H(knopfSichern)) + _
          " " + STR$(GUI_GET_X(knopfLaden)) + " " + STR$(GUI_GET_Y(knopfLaden)) + _
          " " + STR$(GUI_GET_W(knopfLaden)) + " " + STR$(GUI_GET_H(knopfLaden))
'''
_FELDER = ("zurX zurY vorX vorY frX frY frW frH lstX lstY wuerfelX wuerfelY "
           "uPos uAnz freq welle preset offX offY "
           "stX stY stW stH dauX dauY dauW dauH "
           "sicX sicY sicW sicH ladX ladY ladW ladH").split()


def _rect(geo, praefix):
    return (geo[praefix + "X"], geo[praefix + "Y"],
            geo[praefix + "W"], geo[praefix + "H"])


def _ueberlappt(a, b):
    ax, ay, aw, ah = a
    bx, by, bw, bh = b
    return ax < bx + bw and bx < ax + aw and ay < by + bh and by < ay + ah


def _kopie(tmp_path):
    src = _PILOT.read_text(encoding="utf-8")
    assert src.count("SETFPS(60)") == 1
    src = src.replace("SETFPS(60)", "SETFPS(60)\nSET_WINDOW_POS(-3000, -3000)", 1)
    assert src.count("    FLIP()\nWEND") == 1
    src = src.replace("    FLIP()\nWEND", _PROBE + "    FLIP()\nWEND")
    ziel = tmp_path / "pilot.dh"
    ziel.write_text(src, encoding="utf-8")
    return ziel


def _events(tmp_path, events):
    # Nach Bild sortiert: die Wiedergabe arbeitet die Liste der Reihe nach ab
    # und haelt beim ersten spaeteren Eintrag an. Ein nachgereichtes frueheres
    # Ereignis kaeme sonst zusammen mit einem spaeteren in EINEM Bild an.
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
    return geo["offX"] + x, geo["offY"] + y


def _reglermitte(geo):
    return _schirm(geo, geo["frX"] + geo["frW"] // 2, geo["frY"] + geo["frH"] // 2)


def test_die_beschriftungen_liegen_nicht_auf_den_knoepfen(tmp_path):
    """Bis 2026-09-02 standen "Bereit." und die Dauer-Anzeige bei y = 482
    und 504 -- mitten auf [Sichern] und [Laden] (478..506). Der Knopftext
    und die Meldung waren uebereinander gedruckt, lesbar war keins von
    beidem.

    Geprueft werden die RECHTECKE, nicht ein Klick: eine Beschriftung nimmt
    ohnehin keine Klicks an, der Schaden ist rein optisch -- und `GUI_HIT_TEST`
    lieferte an dieser Stelle die zuletzt angelegte Beschriftung, also in
    BEIDEN Faellen dasselbe. (Genau daran ist die erste Fassung dieses Tests
    gescheitert: sie war in der alten Lage ebenso gruen.)
    """
    geo = _lauf(tmp_path, 12)[-1]
    for lbl in ("st", "dau"):
        for knopf in ("sic", "lad"):
            assert not _ueberlappt(_rect(geo, lbl), _rect(geo, knopf)),                 "%s liegt auf %s: %s vs %s" % (lbl, knopf, _rect(geo, lbl),
                                               _rect(geo, knopf))


def test_am_anfang_gibt_es_nichts_zurueckzunehmen(tmp_path):
    letzte = _lauf(tmp_path, 12)[-1]
    assert letzte["uPos"] == 0
    assert letzte["uAnz"] == 1, "der Start ist EIN Stand, nicht zwei gleiche"


def test_ein_zug_am_regler_ist_ein_schritt(tmp_path):
    """Wuerde jedes geaenderte Bild aufgezeichnet, staenden hier ein Dutzend
    Schritte -- und ein Strg+Z naehme nur das letzte Stueckchen zurueck."""
    geo = _lauf(tmp_path, 12)[-1]
    mx, my = _reglermitte(geo)
    letzte = _lauf(tmp_path, 40, _zug(6, mx, my, mx - 50, my))[-1]
    assert letzte["freq"] != geo["freq"], "der Regler hat sich ueberhaupt bewegt"
    assert letzte["uAnz"] == 2, "genau EIN Schritt fuer den ganzen Zug"
    assert letzte["uPos"] == 1


def test_zurueck_und_vor_am_knopf(tmp_path):
    geo = _lauf(tmp_path, 12)[-1]
    mx, my = _reglermitte(geo)
    ev = _zug(6, mx, my, mx - 50, my)
    nachZug = _lauf(tmp_path, 26, ev)[-1]
    ev += _klick(28, *_schirm(geo, geo["zurX"] + 40, geo["zurY"] + 14))
    nachZurueck = _lauf(tmp_path, 44, ev)[-1]
    assert nachZurueck["freq"] == geo["freq"], "der Wert steht wieder wie vorher"
    assert nachZurueck["uPos"] == 0
    ev += _klick(46, *_schirm(geo, geo["vorX"] + 30, geo["vorY"] + 14))
    letzte = _lauf(tmp_path, 62, ev)[-1]
    assert letzte["freq"] == nachZug["freq"], "und ist wieder da"
    assert letzte["uPos"] == 1


def test_strg_z_und_strg_y_tun_dasselbe(tmp_path):
    geo = _lauf(tmp_path, 12)[-1]
    mx, my = _reglermitte(geo)
    ev = _zug(6, mx, my, mx - 50, my)
    ev += _strg(26, TASTE_Z)
    nachZurueck = _lauf(tmp_path, 34, ev)[-1]
    assert nachZurueck["freq"] == geo["freq"] and nachZurueck["uPos"] == 0
    ev += _strg(36, TASTE_Y)
    letzte = _lauf(tmp_path, 46, ev)[-1]
    assert letzte["freq"] != geo["freq"] and letzte["uPos"] == 1


def test_ein_wuerfelwurf_ist_genau_ein_schritt(tmp_path):
    """[Zufall] verstellt alle 16 Regler auf einmal -- als 16 Schritte waere
    er mit einem Strg+Z nicht mehr zurueckzunehmen."""
    geo = _lauf(tmp_path, 12)[-1]
    ev = _klick(6, *_schirm(geo, geo["wuerfelX"] + 40, geo["wuerfelY"] + 14))
    nachWurf = _lauf(tmp_path, 30, ev)[-1]
    assert nachWurf["uAnz"] == 2 and nachWurf["uPos"] == 1
    ev += _klick(32, *_schirm(geo, geo["zurX"] + 40, geo["zurY"] + 14))
    letzte = _lauf(tmp_path, 52, ev)[-1]
    assert letzte["freq"] == geo["freq"], "ein Zurueck reicht fuer den ganzen Wurf"


def test_eine_werkseinstellung_ist_genau_ein_schritt(tmp_path):
    """Sie setzt 16 Regler UND die Wellenform -- auch das muss ein Schritt
    sein, sonst laesst sie sich nicht in einem Zug zuruecknehmen."""
    geo = _lauf(tmp_path, 12)[-1]
    # Der dritte Eintrag ("Explosion"), Zeilenhoehe rund 22.
    ev = _klick(6, *_schirm(geo, geo["lstX"] + 60, geo["lstY"] + 55))
    nachPreset = _lauf(tmp_path, 30, ev)[-1]
    assert nachPreset["preset"] == 2, "die Werkseinstellung wurde ueberhaupt gewaehlt"
    assert nachPreset["uAnz"] == 2 and nachPreset["uPos"] == 1
    ev += _klick(32, *_schirm(geo, geo["zurX"] + 40, geo["zurY"] + 14))
    letzte = _lauf(tmp_path, 52, ev)[-1]
    assert letzte["freq"] == geo["freq"], "ein Zurueck reicht fuer die ganze Einstellung"
    assert letzte["welle"] == geo["welle"], "die Wellenform gehoert zum Stand"


def test_ein_neuer_zug_schneidet_den_vor_weg_ab(tmp_path):
    geo = _lauf(tmp_path, 12)[-1]
    mx, my = _reglermitte(geo)
    ev = _zug(6, mx, my, mx - 50, my)
    ev += _klick(24, *_schirm(geo, geo["zurX"] + 40, geo["zurY"] + 14))
    ev += _zug(32, mx, my, mx + 30, my)
    letzte = _lauf(tmp_path, 60, ev)[-1]
    assert letzte["uPos"] == 1 and letzte["uAnz"] == 2, "der alte Vor-Weg ist weg"
