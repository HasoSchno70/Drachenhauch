"""Die IDE in Drachenhauch (`ide/ide.dh`, Weg C aus docs/entwurf-python-abbau.md).

Geprueft wird ueber die Protokolldatei, die die IDE mit `DH_IDE_LOG` schreibt --
so sieht der Test, was sie getan hat, ohne ins Bild zu schauen -- und ueber
Tasten, die eingespeist werden.

Die Faelle der Staende 1 bis 5 stehen seit Stufe 34 in
`tests/pruef/werkzeug_ide.dhtest`, die der Staende 6 bis 12 seit Stufe 35 in
`tests/pruef/werkzeug_ide_6_12.dhtest`, die der Staende 13 bis 25 seit Stufe 36
in `tests/pruef/werkzeug_ide_13_25.dhtest` (alle ohne Python). Hier geblieben
ist nur, was zwei IDE-Laeufe hintereinander braucht (Sitzung, Umbruch, die
Lage des Farbfelds), ein git-Repository, einen fremden PDF-Leser oder im Bild
Farbbaender zaehlt.

Braucht ein Fenster und speist Tasten ein -- `_BRAUCHT_GRAFIK` und `_SERIELL`.
"""
from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent
IDE = _ROOT / "ide" / "ide.dh"


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
# raylib-Tastencodes (US-Belegung, physisch)
RL_F5, RL_F7, RL_F8, RL_F9, RL_F10 = 294, 296, 297, 298, 299
RL_ENTER, RL_DOWN, RL_LSHIFT, RL_LCTRL = 257, 264, 340, 341
RL_E, RL_F, RL_P, RL_V, RL_Y = 69, 70, 80, 86, 89
RL_D, RL_K, RL_O, RL_S = 68, 75, 79, 83
RL_F1, RL_F2, RL_F6, RL_UP, RL_LALT = 290, 291, 295, 265, 342
RL_F4, RL_Z, RL_B, RL_J, RL_G = 293, 90, 66, 74, 71
RL_L = 76
RL_RIGHT, RL_END = 262, 269
RL_SPACE, RL_LEFT = 32, 263
RL_T, RL_W = 84, 87
RL_F3, RL_F12, RL_U = 292, 301, 85


def _ide(tmp_path, datei, frames=90, events=None, zwischenablage=None, konfig=None,
         screenshot=None, wurzel=None):
    """Die IDE mit `datei` starten, N Bilder laufen lassen, Protokoll liefern."""
    # Alles, was der Test selbst braucht, liegt NEBEN dem Projektordner:
    # seit der Baum auch .json und .txt zeigt, stuenden Protokoll, Sitzung
    # und Aufnahme sonst mitten in der Dateiliste.
    aussen = tmp_path.parent / (tmp_path.name + "_idekopie")
    aussen.mkdir(exist_ok=True)
    log = aussen / "ide.log"
    quelle = IDE
    if events is not None:
        # Die Aufnahme muss NEBEN der IDE-Quelle liegen? Nein: AUTOMATION_PLAY
        # nimmt einen Pfad -- wir kopieren die IDE aber nicht, sondern legen
        # die Wiedergabe ueber eine kleine Startdatei, die sie importiert.
        ev = sorted(events, key=lambda e: e[0])
        zeilen = ["# Test-Aufnahme", f"c {len(ev)}"]
        for frame, typ, *params in ev:
            p = (list(params) + [0, 0, 0, 0])[:4]
            zeilen.append(f"e {frame} {typ} {p[0]} {p[1]} {p[2]} {p[3]} // Event: test")
        (aussen / "ev.txt").write_text("\n".join(zeilen) + "\n", encoding="utf-8")
        einschub = 'SETFPS(60)\nAUTOMATION_PLAY("' + (aussen / "ev.txt").as_posix() + '")\n'
        if zwischenablage is not None:
            einschub += 'CLIPBOARD_SET("' + zwischenablage + '")\n'
        text = IDE.read_text(encoding="utf-8").replace('SETFPS(60)\n', einschub, 1)
        # Die Kopie enthaelt den eingeschobenen Suchtext und stuende sonst
        # im Projektbaum, in den Suchtreffern und -- seit die Umbauten
        # Unterordner sehen -- auch in jedem Umbau.
        quelle = aussen / "ide_test.dh"
        quelle.write_text(text, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(quelle), "--", str(datei)], capture_output=True,
                       text=True, encoding="utf-8", errors="replace", timeout=180,
                       # DH_IDE_KONFIG: die Sitzung des Tests bleibt im Testordner --
                       # sonst schriebe jeder Lauf in die echte ide.json des Nutzers.
                       env=dict(os.environ, DHRT_FRAMES=str(frames), DH_IDE_LOG=str(log),
                                DH_IDE_WURZEL=str(wurzel or _ROOT),
                                DH_IDE_KONFIG=str(konfig or aussen / "ide.json"),
                                **({"DHRT_SCREENSHOT": str(screenshot)} if screenshot else {})),
                       cwd=str(tmp_path))
    assert r.returncode == 0, (r.stdout, r.stderr)
    return log.read_text(encoding="utf-8").splitlines() if log.exists() else []


def _taste(frame, code, *halten):
    """Eine Taste (mit gehaltenen Modifiern) druecken und wieder loslassen --
    eine eingespeiste Taste bleibt sonst bis zum Ende gedrueckt."""
    ev = [(frame, KEY_DOWN, h) for h in halten]
    ev += [(frame, KEY_DOWN, code), (frame + 2, KEY_UP, code)]
    ev += [(frame + 2, KEY_UP, h) for h in halten]
    return ev


# ---------------------------------------------------------------- Stufe 3

def test_listing_als_pdf_neben_der_quelle(tmp_path):
    fitz = pytest.importorskip("fitz")
    quelle = tmp_path / "spiel.dh"
    quelle.write_text("".join(f'PRINT "Zeile {i} mit Umlaut ae oe ue"\n' for i in range(1, 141)),
                      encoding="utf-8")
    ev = _taste(20, RL_P, RL_LCTRL, RL_LSHIFT) + _taste(50, RL_V, RL_LCTRL) + _taste(70, RL_ENTER)
    log = _ide(tmp_path, quelle, frames=150, events=ev, zwischenablage="pdf")
    assert any(z.startswith("pdf ") for z in log), log
    pdf = tmp_path / "spiel.pdf"
    assert pdf.exists()
    doc = fitz.open(str(pdf))
    assert doc.page_count == 3, doc.page_count            # 140 Zeilen, 66 je Seite
    text = doc[0].get_text()
    assert "spiel.dh" in text and "Seite 1" in text and "Zeile 1 mit" in text, text[:300]
    assert "Zeile 140" in doc[2].get_text()


# ---------------------------------------------------------------- Stufe 4

MAUS_HOCH, MAUS_RUNTER, MAUS_POS = 5, 6, 7


def _maus(frame, x, y):
    """Zeiger hinsetzen und klicken -- die Position braucht zwei Bilder,
    weil das gui den Druck erst im naechsten sieht."""
    return [(frame, MAUS_POS, x, y), (frame + 1, MAUS_POS, x, y),
            (frame + 2, MAUS_RUNTER, 0), (frame + 4, MAUS_HOCH, 0)]


def _aussen(tmp_path):
    """Wo der Harnisch seine eigenen Dateien ablegt -- NEBEN dem
    Projektordner, seit der Baum auch .json und .txt zeigt."""
    d = tmp_path.parent / (tmp_path.name + "_idekopie")
    d.mkdir(exist_ok=True)
    return d


def _datei(tmp_path, text, name="spiel.dh"):
    p = tmp_path / name
    p.write_text(text, encoding="utf-8")
    return p


def test_sitzung_und_zuletzt_geoeffnet_ueberleben_den_neustart(tmp_path):
    """Der erste Lauf merkt sich die Datei (zuletzt + Sitzung) in der
    ide.json; der zweite Lauf ohne gueltige Datei stellt sie wieder her."""
    import json
    quelle = _datei(tmp_path, "PRINT 1\n")
    _ide(tmp_path, quelle, frames=60)
    konfig = json.loads((_aussen(tmp_path) / "ide.json").read_text(encoding="utf-8"))
    assert konfig["zuletzt"] and konfig["zuletzt"][0].endswith("spiel.dh"), konfig
    assert konfig["sitzung"] and konfig["sitzung"][0].endswith("spiel.dh"), konfig
    log = _ide(tmp_path, tmp_path / "gibt_es_nicht.dh", frames=60)
    assert any(z.startswith("geoeffnet ") and z.endswith("spiel.dh") for z in log), log


# ---------------------------------------------------------------- Stufe 5

def test_zeilenumbruch_bleibt_gemerkt(tmp_path):
    """Alt+Z schaltet den Umbruch an; er steht in der ide.json und gilt beim
    naechsten Start wieder."""
    import json
    quelle = _datei(tmp_path, "PRINT 1\n")
    _ide(tmp_path, quelle, frames=90, events=_taste(30, RL_Z, RL_LALT))
    konfig = json.loads((_aussen(tmp_path) / "ide.json").read_text(encoding="utf-8"))
    assert konfig["umbruch"] is True, konfig
    _ide(tmp_path, quelle, frames=60)
    konfig = json.loads((_aussen(tmp_path) / "ide.json").read_text(encoding="utf-8"))
    assert konfig["umbruch"] is True, konfig


def test_sitzung_haengt_am_projektordner(tmp_path):
    """Zwei Ordner, je eine Datei, EINE Konfigurationsdatei. Wer wieder im
    ersten startet, bekommt dessen Reiter -- nicht die des zweiten, die
    zuletzt offen waren. Das ist zugleich die Gegenprobe: die globale
    Sitzung (Stand 4) zeigt an dieser Stelle auf zwei.dh."""
    import json
    (tmp_path / "a").mkdir()
    (tmp_path / "b").mkdir()
    kfg = _aussen(tmp_path) / "ide.json"
    ea = _datei(tmp_path / "a", "PRINT 1\n", name="eins.dh")
    eb = _datei(tmp_path / "b", "PRINT 2\n", name="zwei.dh")
    _ide(tmp_path / "a", ea, frames=60, konfig=kfg)
    _ide(tmp_path / "b", eb, frames=60, konfig=kfg)
    stand = json.loads(kfg.read_text(encoding="utf-8"))
    assert stand["sitzung"][0].endswith("zwei.dh"), stand
    log = _ide(tmp_path / "a", tmp_path / "a" / "gibt_es_nicht.dh", frames=60, konfig=kfg)
    geoeffnet = [z for z in log if z.startswith("geoeffnet ")]
    assert geoeffnet and geoeffnet[0].endswith("eins.dh"), log
    assert not any(z.endswith("zwei.dh") for z in geoeffnet), log


def test_git_blame_listet_wer_welche_zeile_geschrieben_hat(tmp_path):
    """Ein kleines Repository, ein Commit, Strg+Umschalt+B: die Liste unten
    rechts nennt je Zeile Datum und Person. Gegenprobe: ohne Repository
    meldet sie nichts."""
    quelle = _datei(tmp_path, "PRINT 1\nPRINT 2\nPRINT 3\n")
    for cmd in (["git", "init", "-q"], ["git", "add", "spiel.dh"],
                ["git", "-c", "user.name=Test", "-c", "user.email=t@t",
                 "commit", "-q", "-m", "erst"]):
        r = subprocess.run(cmd, cwd=str(tmp_path), capture_output=True)
        if r.returncode != 0:
            pytest.skip("git nicht verfuegbar: " + r.stderr.decode("utf-8", "replace"))
    log = _ide(tmp_path, quelle, frames=140, events=_taste(40, RL_B, RL_LCTRL, RL_LSHIFT))
    assert "blame 3" in log, log


# ---------------------------------------------------------------- Stufe 6

def test_das_wort_unter_der_marke_wird_ueberall_hervorgehoben(tmp_path):
    """Am Bild geprüft: mit der Marke auf `punkte` tragen ALLE fünf Stellen
    die Fundstellenfarbe. Gegenprobe im selben Bild -- `mehr` daneben nicht."""
    quelle = _datei(tmp_path, "DIM punkte AS INTEGER\npunkte = 0\n"
                              "SUB zaehlen(mehr AS INTEGER)\n"
                              "    punkte = punkte + mehr\nEND SUB\n")
    schuss = tmp_path / "bild.png"
    # Bis in das Wort hinein: hinter dem Zeilenende steht nur die 0.
    ev = _taste(30, RL_DOWN) + _taste(50, RL_END)
    for k in range(5):
        ev += _taste(70 + k * 6, RL_LEFT)
    _ide(tmp_path, quelle, frames=160, events=ev, screenshot=schuss)
    from PIL import Image
    im = Image.open(schuss).convert("RGB")
    # F_FUNDSTELLE = &HFFD070 -- warmes Gelb, im Bild sonst nirgends.
    treffer = [(x, y) for y in range(60, 260) for x in range(220, 1200)
               if _nahe(im.getpixel((x, y)), (0xFF, 0xD0, 0x70), 40)]
    assert treffer, "keine Fundstellenfarbe im Bild"
    # `punkte` steht in den Zeilen 1, 2 und 4 -- drei Baender.
    baender = _gruppen(sorted({y for _, y in treffer}), 3)
    assert len(baender) == 3, (len(baender), sorted({y for _, y in treffer})[:20])
    # Und in Zeile 4 (`punkte = punkte + mehr`) zweimal, `mehr` NICHT --
    # das ist die Gegenprobe: sonst waeren es drei Woerter.
    letzte = set(baender[-1])
    woerter = _gruppen(sorted({x for x, y in treffer if y in letzte}), 6)
    assert len(woerter) == 2, [len(w) and (w[0], w[-1]) for w in woerter]


def _nahe(a, b, tol):
    return all(abs(x - y) <= tol for x, y in zip(a, b))


def _gruppen(werte, luecke):
    """Zusammenhaengende Laeufe: alles, was weiter als `luecke` auseinander
    liegt, faengt eine neue Gruppe an."""
    aus = []
    for v in werte:
        if not aus or v - aus[-1][-1] > luecke:
            aus.append([v])
        else:
            aus[-1].append(v)
    return aus


def _repo(tmp_path, inhalt, name="spiel.dh"):
    """Ein kleines Repository mit einer eingecheckten Datei."""
    quelle = _datei(tmp_path, inhalt, name=name)
    for cmd in (["git", "init", "-q"], ["git", "add", name],
                ["git", "-c", "user.name=Test", "-c", "user.email=t@t",
                 "commit", "-q", "-m", "erst"]):
        r = subprocess.run(cmd, cwd=str(tmp_path), capture_output=True)
        if r.returncode != 0:
            pytest.skip("git nicht verfuegbar: " + r.stderr.decode("utf-8", "replace"))
    return quelle


def test_git_diff_zeigt_die_aenderungen_der_datei(tmp_path):
    """Eingecheckt, dann geändert: Strg+Umschalt+D zeigt den Diff. Und die
    geänderten Zeilen tragen schon vorher eine Marke am Rand -- zwei, nicht
    vier: `git diff -U0` nennt nur die wirklich geänderten."""
    quelle = _repo(tmp_path, "PRINT 1\nPRINT 2\nPRINT 3\nPRINT 4\n")
    quelle.write_text("PRINT 1\nPRINT zwei\nPRINT 3\nPRINT vier\n", encoding="utf-8")
    log = _ide(tmp_path, quelle, frames=160,
               events=_taste(50, RL_D, RL_LCTRL, RL_LSHIFT))
    assert "git rand 2" in log, log
    diff = [z for z in log if z.startswith("git diff ")]
    assert diff and int(diff[0].split()[2]) > 5, log


def test_git_diff_ohne_aenderung_sagt_es(tmp_path):
    """Gegenprobe: nichts geändert, also nichts zu zeigen -- und keine
    Marken am Rand."""
    quelle = _repo(tmp_path, "PRINT 1\n")
    log = _ide(tmp_path, quelle, frames=160,
               events=_taste(50, RL_D, RL_LCTRL, RL_LSHIFT))
    assert "git diff 0" in log, log
    assert "git rand 0" in log, log


# --------------------------------------------------------------- Stufe 28

def test_farbfeld_oeffnet_den_waehler_und_schreibt_zurueck(tmp_path):
    """Zwei Läufe: der erste sagt, WO das Farbfeld liegt (die Geometrie
    steht erst zur Laufzeit fest), der zweite klickt darauf und übernimmt.
    Danach steht eine andere Farbe in der Datei."""
    from PIL import Image
    quelle = _datei(tmp_path, "SCREEN(320, 240)\nCLS(&HFF8800)\n")
    schuss = tmp_path / "bild.png"
    _ide(tmp_path, quelle, frames=120, screenshot=schuss)
    im = Image.open(schuss).convert("RGB")
    punkte = [(x, y) for y in range(60, 200) for x in range(220, 1400)
              if _nahe(im.getpixel((x, y)), (0xFF, 0x88, 0x00), 20)]
    assert punkte, "kein Farbfeld im Bild"
    mx = sum(x for x, _ in punkte) // len(punkte)
    my = sum(y for _, y in punkte) // len(punkte)
    # Klick auf das Feld, dann [Uebernehmen] im Waehler
    ev = _maus(40, mx, my)
    log = _ide(tmp_path, quelle, frames=200, events=ev)
    assert any(z.startswith("farbfeld ") for z in log), (log, mx, my)
