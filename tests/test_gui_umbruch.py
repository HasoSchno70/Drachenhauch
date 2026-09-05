"""Zeilenumbruch im Textbereich (GUI_TEXTAREA_SET "umbruch").

Der Textbereich war ein Code-Feld: lange Zeilen rollten waagerecht. Fuer
Notizen und Briefe bricht er jetzt an Wortgrenzen um. Geprueft wird nicht
das Bild, sondern das Verhalten der Schreibmarke: Ende und Pfeil runter
bewegen sich in SICHTBAREN Zeilen, also innerhalb einer umgebrochenen
logischen Zeile. Eine Schreibmarken-Abfrage gibt es nicht -- ihre Lage
verraet ein eingefuegtes Zeichen (Strg+V), das dort landet, wo sie steht.

Seriell, weil Tasten eingespeist werden.
"""
import os
import subprocess
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

KEY_UP, KEY_DOWN = 1, 2
RL_LCTRL, RL_V, RL_END, RL_HOME, RL_DOWN = 341, 86, 269, 268, 264

_TEXT = "Ein langer Absatz der in einem schmalen Feld mehrfach umbrechen muss damit man es sieht"
_KOPF = ('IMPORT "gui"\n'
         'SCREEN(500, 400, "T", 1)\n'
         'SET_WINDOW_POS(-3000, -3000)\n'
         'DIM w AS GUI_WINDOW\n'
         'w = GUI_WINDOW("T", 0, 0, 500, 400)\n'
         'GUI_WINDOW_CHROME(w, FALSE)\n'
         'DIM ta AS GUI_WIDGET : ta = GUI_TEXTAREA(w, 10, 10, 200, 150)\n'
         f'GUI_SET_TEXT(ta, "{_TEXT}")\n'
         'CLIPBOARD_SET("#")\n')


def _lauf(tmp_path, src, frames=1, events=None):
    if events is not None:
        ev = sorted(events, key=lambda e: e[0])
        zeilen = ["# Test-Aufnahme", f"c {len(ev)}"]
        for frame, typ, *params in ev:
            p = (list(params) + [0, 0, 0, 0])[:4]
            zeilen.append(f"e {frame} {typ} {p[0]} {p[1]} {p[2]} {p[3]} // Event: test")
        (tmp_path / "ev.txt").write_text("\n".join(zeilen) + "\n", encoding="utf-8")
        src = src.replace('SET_WINDOW_POS(-3000, -3000)\n',
                          'SET_WINDOW_POS(-3000, -3000)\nAUTOMATION_PLAY("ev.txt")\n', 1)
    f = tmp_path / "t.dh"
    f.write_text(src, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(f)], capture_output=True, text=True,
                       encoding="utf-8", errors="replace", timeout=90,
                       env=dict(os.environ, DHRT_FRAMES=str(frames)), cwd=str(tmp_path))
    assert r.returncode == 0, r.stderr
    return [ln.rstrip() for ln in (r.stdout or "").splitlines()
            if ln.strip() and not ln.startswith(("WARNING:", "INFO:"))]


def _schleife(bilder):
    return (f"GUI_FOCUS(ta)\nDIM f AS INTEGER\nFOR f = 1 TO {bilder}\n    GUI_UPDATE()\n"
            '    PRINT "T " + GUI_TEXT(ta)\n    GUI_DRAW()\n    FLIP()\nNEXT\n')


def _taste(frame, code, *mods):
    ev = [(frame, KEY_DOWN, m) for m in mods]
    ev += [(frame + 1, KEY_DOWN, code), (frame + 2, KEY_UP, code)]
    ev += [(frame + 3, KEY_UP, m) for m in mods]
    return ev


def _marke(out):
    """Stelle des eingefuegten # im letzten Text."""
    return out[-1][2:].index("#")


def test_view_zaehlt_mit_umbruch_weniger_logische_zeilen(tmp_path):
    out = _lauf(tmp_path, _KOPF +
                'GUI_SET_TEXT(ta, "' + _TEXT + '" + CHR$(10) + "zwei" + CHR$(10) + "drei" + CHR$(10) + "vier")\n'
                'GUI_UPDATE()\nPRINT GUI_TEXTAREA_VIEW(ta)\n'
                'GUI_TEXTAREA_SET(ta, "umbruch", 1)\nGUI_UPDATE()\nPRINT GUI_TEXTAREA_VIEW(ta)\n'
                'DIM j AS STRING : j = GUI_TO_JSON(w)\n'
                'PRINT INSTR(j, "wrap_text") >= 0 ; " " ; GUI_TO_JSON(GUI_FROM_JSON(j)) = j\n')
    ohne = out[0].strip("()").split(", ")
    mit = out[1].strip("()").split(", ")
    assert ohne[0] == "0" and int(ohne[1]) >= 4, "ohne Umbruch: die sichtbaren Plaetze, mindestens die vier Zeilen"
    assert mit[0] == "0" and int(mit[1]) < 4, "mit Umbruch fuellt die lange Zeile mehrere sichtbare Plaetze -- weniger logische Zeilen passen"
    assert int(mit[3]) <= int(ohne[3]), "der Ausschnitt in Zeichen ist hoechstens so lang"
    assert out[2] == "TRUE TRUE"


RL_UP = 265


def _nach_oben(frame):
    """Die Schreibmarke steht nach GUI_SET_TEXT am Ende; sechsmal Pfeil hoch
    und Pos1 bringen sie in jedem Fall an den Anfang der ersten Zeile."""
    ev = []
    for k in range(6):
        ev += _taste(frame + k * 3, RL_UP)
    return ev + _taste(frame + 18, RL_HOME)


def test_ende_und_pfeil_runter_bleiben_in_der_sichtbaren_zeile(tmp_path):
    # Ohne Umbruch: Ende springt ans Ende der (einzigen) Zeile.
    ev = _nach_oben(3) + _taste(25, RL_END) + _taste(28, RL_V, RL_LCTRL)
    out = _lauf(tmp_path, _KOPF + _schleife(34), frames=35, events=ev)
    assert _marke(out) == len(_TEXT)
    # Mit Umbruch: Ende bleibt am Ende der ersten SICHTBAREN Zeile.
    src = _KOPF + 'GUI_TEXTAREA_SET(ta, "umbruch", 1)\n' + _schleife(34)
    out = _lauf(tmp_path, src, frames=35, events=ev)
    ende1 = _marke(out)
    assert 0 < ende1 < len(_TEXT), out[-1]
    assert _TEXT[ende1 - 1] == " ", "der Umbruch sitzt hinter einem Leerzeichen, an der Wortgrenze"
    # Pfeil runter von Pos1: eine sichtbare Zeile tiefer, dieselbe Spalte 0.
    ev2 = _nach_oben(3) + _taste(25, RL_DOWN) + _taste(28, RL_V, RL_LCTRL)
    out = _lauf(tmp_path, src, frames=35, events=ev2)
    runter = _marke(out)
    assert runter == ende1, "die zweite sichtbare Zeile beginnt, wo die erste aufhoert"
