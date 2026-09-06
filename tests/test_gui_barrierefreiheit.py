"""Barrierefreiheit (docs/entwurf-barrierefreiheit.md, Wege A und C).

Der Kern ist der Pruefstein aus dem Entwurf: ein FREMDER Leser -- UI
Automation aus PowerShell heraus (`System.Windows.Automation`, ohne neue
Abhaengigkeit) -- zaehlt die Bedienelemente eines laufenden dhrt-Fensters,
liest ihre Namen, klickt den Knopf, setzt Text und Fokus. Vor dem Bau fand
derselbe Leser NULL Nachkommen. Dazu Weg A: Menue per F10/Alt, setzbare
Tab-Reihenfolge, Kontrast des hellen Themas.

Tasten werden echt eingespeist (Automation-Wiedergabe), und der UIA-Leser
braucht das Fenster -- darum in `_SERIELL` und `_BRAUCHT_GRAFIK`.
"""
import os
import subprocess
import sys
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
pytestmark = pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")

KEY_UP, KEY_DOWN = 1, 2
RL_ESC, RL_ENTER, RL_TAB = 256, 257, 258
RL_RIGHT, RL_LEFT, RL_DOWN, RL_UP = 262, 263, 264, 265
RL_F10, RL_LALT = 299, 342

_KOPF = ('IMPORT "gui"\n'
         'SCREEN(400, 300, "T", 1)\n'
         'SET_WINDOW_POS(-3000, -3000)\n'
         'DIM w AS GUI_WINDOW\n'
         'w = GUI_WINDOW("T", 10, 10, 360, 260)\n')


def _taste(frame, code):
    """Druck im Bild `frame`, Loslassen im naechsten -- eine eingespeiste
    Taste bleibt sonst bis zum Ende gedrueckt."""
    return [(frame, KEY_DOWN, code), (frame + 1, KEY_UP, code)]


def _lauf(tmp_path, src, frames=12, events=None, env=None):
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
                       encoding="utf-8", errors="replace", timeout=120,
                       env=dict(os.environ, DHRT_FRAMES=str(frames), **(env or {})), cwd=str(tmp_path))
    assert r.returncode == 0, (r.stdout, r.stderr)
    return [ln.strip() for ln in (r.stdout or "").splitlines()
            if ln.strip() and not ln.startswith(("WARNING:", "INFO:"))]


_SCHLEIFE = ('WHILE NOT QUITREQUESTED()\n'
             '    GUI_UPDATE()\n'
             '{body}'
             '    CLS(0) : GUI_DRAW() : FLIP()\n'
             'WEND\n')


# ------------------------------------------------------ Weg A: Menue per Tastatur
def test_menue_per_f10_und_alt(tmp_path):
    src = (_KOPF +
           'DIM m AS INTEGER : m = GUI_MENU(w, "Datei")\n'
           'DIM iNeu AS INTEGER : iNeu = GUI_MENU_ITEM(m, "Neu")\n'
           'DIM iAuf AS INTEGER : iAuf = GUI_MENU_ITEM(m, "Oeffnen")\n'
           'DIM tf AS GUI_WIDGET : tf = GUI_TEXTINPUT(w, 10, 60, 200, 26)\n'
           'GUI_FOCUS(tf)\n' +
           _SCHLEIFE.format(body='    IF GUI_CLICKED(iNeu) THEN PRINT "klick Neu"\n'
                                 '    IF GUI_CLICKED(iAuf) THEN PRINT "klick Oeffnen"\n'))
    ev = (_taste(3, RL_F10) + _taste(6, RL_DOWN) + _taste(9, RL_ENTER)      # F10, ab, Enter -> Oeffnen
          + _taste(13, RL_LALT) + _taste(16, RL_ENTER)                     # Alt allein, Enter -> Neu
          + _taste(20, RL_F10) + _taste(23, RL_ESC) + _taste(26, RL_ENTER))  # auf, zu, Enter -> nichts
    out = _lauf(tmp_path, src, frames=32, events=ev)
    assert out == ["klick Oeffnen", "klick Neu"], out


def test_alt_mit_taste_oeffnet_nicht(tmp_path):
    # Alt+X ist ein Kuerzel, kein "Alt allein" -- das Menue darf danach
    # nicht aufstehen. Gegenprobe: Enter danach loest keinen Eintrag aus.
    src = (_KOPF +
           'DIM m AS INTEGER : m = GUI_MENU(w, "Datei")\n'
           'DIM iNeu AS INTEGER : iNeu = GUI_MENU_ITEM(m, "Neu")\n' +
           _SCHLEIFE.format(body='    IF GUI_CLICKED(iNeu) THEN PRINT "klick Neu"\n'))
    ev = ([(3, KEY_DOWN, RL_LALT), (5, KEY_DOWN, RL_RIGHT), (6, KEY_UP, RL_RIGHT), (8, KEY_UP, RL_LALT)]
          + _taste(11, RL_ENTER))
    out = _lauf(tmp_path, src, frames=16, events=ev)
    assert out == [], out


# ------------------------------------------------------ Weg A: Tab-Reihenfolge
def test_tab_reihenfolge_ist_setzbar(tmp_path):
    src = (_KOPF +
           'DIM b1 AS GUI_WIDGET : b1 = GUI_BUTTON(w, "Eins", 10, 10, 80, 28)\n'
           'DIM b2 AS GUI_WIDGET : b2 = GUI_BUTTON(w, "Zwei", 10, 50, 80, 28)\n'
           'DIM b3 AS GUI_WIDGET : b3 = GUI_BUTTON(w, "Drei", 10, 90, 80, 28)\n'
           'GUI_SET_TAB_INDEX(b3, 1)\n'
           'GUI_SET_TAB_INDEX(b1, 2)\n'
           'DIM letzt AS GUI_WIDGET : letzt = -1\n' +
           _SCHLEIFE.format(body='    IF GUI_FOCUSED() <> letzt THEN\n'
                                 '        letzt = GUI_FOCUSED()\n'
                                 '        PRINT GUI_TEXT(letzt)\n'
                                 '    END IF\n'))
    ev = _taste(3, RL_TAB) + _taste(6, RL_TAB) + _taste(9, RL_TAB) + _taste(12, RL_TAB)
    out = _lauf(tmp_path, src, frames=16, events=ev)
    assert out == ["Drei", "Eins", "Zwei", "Drei"], out


def test_tab_index_headless(tmp_path):
    out = _lauf(tmp_path,
                'IMPORT "gui"\n'
                'DIM w AS GUI_WINDOW : w = GUI_WINDOW("T", 0, 0, 200, 100)\n'
                'DIM b AS GUI_WIDGET : b = GUI_BUTTON(w, "Ok", 10, 10, 80, 28)\n'
                'GUI_SET_TAB_INDEX(b, 2)\n'
                'PRINT INSTR(GUI_TO_JSON(w), CHR$(34) + "tab_index" + CHR$(34) + ": 2") > 0\n'
                'TRY\n    GUI_SET_TAB_INDEX(b, -1)\nCATCH e\n    PRINT e\nEND TRY\n'
                'PRINT GUI_SCREENREADER()\n'
                'GUI_ANNOUNCE("ohne Zuhoerer passiert nichts")\n'
                'GUI_ANNOUNCE("dringend", TRUE)\n'
                'PRINT "ok"\n')
    assert out[0] == "TRUE"
    assert "GUI_SET_TAB_INDEX" in out[1]
    assert out[2] == "FALSE", "ohne Hilfsprogramm hoert niemand zu"
    assert out[3] == "ok"


# ------------------------------------------------------ Weg A: Kontrast
def _kontrast(a, b):
    def lum(c):
        r, g, bl = [((c >> s) & 255) / 255 for s in (16, 8, 0)]
        f = lambda x: x / 12.92 if x <= 0.03928 else ((x + 0.055) / 1.055) ** 2.4
        return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(bl)
    la, lb = sorted((lum(a), lum(b)), reverse=True)
    return (la + 0.05) / (lb + 0.05)


@pytest.mark.parametrize("thema", ["dark", "light", "contrast"])
def test_texte_der_themen_erreichen_wcag_aa(tmp_path, thema):
    out = _lauf(tmp_path,
                'IMPORT "gui"\n'
                f'GUI_THEME_PRESET("{thema}")\n'
                'PRINT GUI_THEME_GET("text_fg") ; " " ; GUI_THEME_GET("muted_fg") ; " " ; '
                'GUI_THEME_GET("win_bg") ; " " ; GUI_THEME_GET("widget_bg") ; " " ; GUI_THEME_GET("accent")\n')
    text, muted, win_bg, widget_bg, accent = [int(x) & 0xFFFFFF for x in out[0].split()]
    assert _kontrast(text, win_bg) >= 4.5 and _kontrast(text, widget_bg) >= 4.5
    # Gedaempfter Text lag im hellen Thema bei 2,76:1 -- unter AA.
    assert _kontrast(muted, win_bg) >= 4.5, f"{thema}: gedaempft {_kontrast(muted, win_bg):.2f}:1"
    # Der Akzent ist Fokusring und Haekchen: mindestens 3:1 (Bedienelemente).
    assert _kontrast(accent, win_bg) >= 3.0, f"{thema}: Akzent {_kontrast(accent, win_bg):.2f}:1"


# ------------------------------------------------------ Weg C: der fremde Leser
_PS_LESER = r'''
param([int]$ZielPid)
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$A = [System.Windows.Automation.AutomationElement]
# Ueber das Fensterhandle, nicht ueber die Kinder der Wurzel: ein Fenster
# ausserhalb des Schirms (SET_WINDOW_POS -3000) zaehlt dort nicht mit.
$win = $null
$kids = $null
for ($i = 0; $i -lt 100; $i++) {
    $h = (Get-Process -Id $ZielPid -ErrorAction SilentlyContinue).MainWindowHandle
    if ($h -and $h -ne 0) {
        $win = $A::FromHandle($h)
        $kids = $win.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)
        if ($kids.Count -ge 5) { break }
    }
    Start-Sleep -Milliseconds 100
}
if (-not $win) { "kein Fenster"; exit 2 }
foreach ($k in $kids) { "N: " + $k.Current.ControlType.ProgrammaticName + " [" + $k.Current.Name + "]" }
function Finde($typ, $name) {
    foreach ($k in $win.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)) {
        if ($k.Current.ControlType.ProgrammaticName -eq $typ -and ($name -eq "" -or $k.Current.Name -eq $name)) { return $k }
    }
    return $null
}
$cb = Finde "ControlType.CheckBox" "Bezahlt"
$edit = Finde "ControlType.Edit" ""
$btn = Finde "ControlType.Button" "Speichern"
if (-not $cb -or -not $edit -or -not $btn) { "Element fehlt"; exit 3 }
$cb.SetFocus(); Start-Sleep -Milliseconds 300
$cb.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern).Toggle(); Start-Sleep -Milliseconds 300
$edit.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).SetValue("Anna"); Start-Sleep -Milliseconds 300
"WERT: " + $edit.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).Current.Value
$btn.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
"fertig"
'''


@pytest.mark.skipif(sys.platform != "win32", reason="UI Automation gibt es nur unter Windows")
def test_ein_fremder_leser_sieht_und_bedient_das_fenster(tmp_path):
    src = ('IMPORT "gui"\n'
           'SCREEN(400, 300, "Zugang", 1)\n'
           'SET_WINDOW_POS(-3000, -3000)\n'
           'DIM w AS GUI_WINDOW : w = GUI_WINDOW("Formular", 10, 10, 360, 260)\n'
           'DIM lb AS GUI_WIDGET : lb = GUI_LABEL(w, "Name", 10, 12)\n'
           'DIM tf AS GUI_WIDGET : tf = GUI_TEXTINPUT(w, 70, 8, 200, 26)\n'
           'DIM cb AS GUI_WIDGET : cb = GUI_CHECKBOX(w, "Bezahlt", 10, 50, FALSE)\n'
           'DIM b AS GUI_WIDGET : b = GUI_BUTTON(w, "Speichern", 10, 90, 120, 32)\n'
           'DIM leser AS BOOLEAN : leser = FALSE\n'
           'DIM fok AS BOOLEAN : fok = FALSE\n'
           'DIM haken AS BOOLEAN : haken = FALSE\n'
           'DIM fertig AS BOOLEAN : fertig = FALSE\n'
           'WHILE NOT QUITREQUESTED() AND NOT fertig\n'
           '    GUI_UPDATE()\n'
           '    IF GUI_SCREENREADER() AND NOT leser THEN PRINT "leser da" : leser = TRUE\n'
           '    IF GUI_FOCUSED() = cb AND NOT fok THEN PRINT "fokus kaestchen" : fok = TRUE\n'
           '    IF GUI_CHECKED(cb) AND NOT haken THEN PRINT "haken" : haken = TRUE\n'
           '    IF GUI_CLICKED(b) THEN PRINT "geklickt " + GUI_TEXT(tf) : fertig = TRUE\n'
           '    CLS(0) : GUI_DRAW() : FLIP()\n'
           'WEND\n')
    (tmp_path / "t.dh").write_text(src, encoding="utf-8")
    (tmp_path / "leser.ps1").write_text(_PS_LESER, encoding="utf-8")
    p = subprocess.Popen([str(_DHRT), "run", str(tmp_path / "t.dh")], stdout=subprocess.PIPE,
                         stderr=subprocess.PIPE, text=True, encoding="utf-8", errors="replace",
                         env=dict(os.environ, DHRT_FRAMES="1500"), cwd=str(tmp_path))
    try:
        ps = subprocess.run(["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass",
                             "-File", str(tmp_path / "leser.ps1"), str(p.pid)],
                            capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=60)
        out, err = p.communicate(timeout=40)
    finally:
        if p.poll() is None:
            p.kill()
    assert ps.returncode == 0, (ps.stdout, ps.stderr, out, err)
    knoten = [z[3:] for z in ps.stdout.splitlines() if z.startswith("N: ")]
    # Vor dem Bau: null Nachkommen. Jetzt: Rolle und Name je Bedienelement.
    assert "ControlType.Button [Speichern]" in knoten, knoten
    assert "ControlType.CheckBox [Bezahlt]" in knoten, knoten
    assert "ControlType.Edit [Name]" in knoten, "das Feld heisst wie die Beschriftung links daneben: " + str(knoten)
    assert "ControlType.Group [Formular]" in knoten, knoten
    assert "WERT: Anna" in ps.stdout, ps.stdout
    zeilen = [z.strip() for z in out.splitlines() if z.strip() and not z.startswith(("WARNING:", "INFO:"))]
    assert zeilen == ["leser da", "fokus kaestchen", "haken", "geklickt Anna"], (zeilen, err)
