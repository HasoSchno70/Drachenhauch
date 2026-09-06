"""Die IDE in Drachenhauch (`ide/ide.dh`, Weg C aus docs/entwurf-python-abbau.md).

Stand 1: Reiter mit Code-Feldern, Projektbaum, Fehlerliste, Hilfe zum Wort,
Vervollstaendigung, Suchen/Ersetzen, Starten mit laufender Ausgabe. Geprueft
wird ueber die Protokolldatei, die die IDE mit `DH_IDE_LOG` schreibt -- so
sieht der Test, was sie getan hat, ohne ins Bild zu schauen -- und ueber
Tasten, die eingespeist werden (F5 startet, F7 prueft).

Braucht ein Fenster und speist Tasten ein -- `_BRAUCHT_GRAFIK` und `_SERIELL`.
"""
from __future__ import annotations

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
RL_F1 = 290


def _ide(tmp_path, datei, frames=90, events=None, zwischenablage=None):
    """Die IDE mit `datei` starten, N Bilder laufen lassen, Protokoll liefern."""
    log = tmp_path / "ide.log"
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
        (tmp_path / "ev.txt").write_text("\n".join(zeilen) + "\n", encoding="utf-8")
        einschub = 'SETFPS(60)\nAUTOMATION_PLAY("' + (tmp_path / "ev.txt").as_posix() + '")\n'
        if zwischenablage is not None:
            einschub += 'CLIPBOARD_SET("' + zwischenablage + '")\n'
        text = IDE.read_text(encoding="utf-8").replace('SETFPS(60)\n', einschub, 1)
        # In einen Unterordner, nicht neben die Testdateien: die Kopie enthaelt
        # den eingeschobenen Suchtext und staende sonst im Projektbaum und in
        # den Suchtreffern.
        (tmp_path / "_ide").mkdir(exist_ok=True)
        quelle = tmp_path / "_ide" / "ide_test.dh"
        quelle.write_text(text, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(quelle), "--", str(datei)], capture_output=True,
                       text=True, encoding="utf-8", errors="replace", timeout=180,
                       env=dict(os.environ, DHRT_FRAMES=str(frames), DH_IDE_LOG=str(log),
                                DH_IDE_WURZEL=str(_ROOT)),
                       cwd=str(tmp_path))
    assert r.returncode == 0, (r.stdout, r.stderr)
    return log.read_text(encoding="utf-8").splitlines() if log.exists() else []


def test_ide_uebersetzt_ohne_befund():
    r = subprocess.run([str(_DHRT), "--check", str(IDE)], capture_output=True, text=True,
                       encoding="utf-8", timeout=120)
    assert r.returncode == 0 and r.stdout.strip() == "[]", r.stdout


def test_ide_oeffnet_datei_und_prueft(tmp_path):
    (tmp_path / "spiel.dh").write_text('PRINT "hallo"\nDIM x AS\n', encoding="utf-8")
    log = _ide(tmp_path, tmp_path / "spiel.dh")
    assert log[0] == "bereit" or "bereit" in log, log
    assert any(z.startswith("geoeffnet ") and z.endswith("spiel.dh") for z in log), log
    # Die Pruefung laeuft 0,6 s nach dem Oeffnen von selbst -- und findet den Fehler.
    assert "geprueft 1" in log, log
    assert log[-1] == "ende"


def test_relativer_name_meint_den_ort_des_nutzers(tmp_path):
    """`dhrt run` wechselt ins Verzeichnis der IDE; `spiel.dh` meint trotzdem
    die Datei dort, wo der Nutzer steht (DHRT_START_DIR), und der Projektbaum
    zeigt diesen Ordner -- nicht ide/."""
    (tmp_path / "spiel.dh").write_text('PRINT 1\n', encoding="utf-8")
    log = _ide(tmp_path, "spiel.dh")
    assert any(z.startswith("geoeffnet ") and z.endswith("spiel.dh") for z in log), log
    projekt = [z for z in log if z.startswith("projekt ")]
    assert projekt and Path(projekt[0][8:]).resolve() == tmp_path.resolve(), log


def test_f5_startet_das_programm_und_zeigt_das_ende(tmp_path):
    (tmp_path / "spiel.dh").write_text('PRINT "hallo aus dem Spiel"\n', encoding="utf-8")
    log = _ide(tmp_path, tmp_path / "spiel.dh", frames=240,
               events=[(20, KEY_DOWN, RL_F5), (22, KEY_UP, RL_F5)])
    assert any(z.startswith("gestartet ") for z in log), log
    assert "beendet 0" in log, log


def test_f7_prueft_auf_tastendruck(tmp_path):
    (tmp_path / "gut.dh").write_text('PRINT 1\n', encoding="utf-8")
    log = _ide(tmp_path, tmp_path / "gut.dh", frames=120,
               events=_taste(60, RL_F7, RL_LSHIFT))
    assert log.count("geprueft 0") >= 2, log


def _taste(frame, code, *halten):
    """Eine Taste (mit gehaltenen Modifiern) druecken und wieder loslassen --
    eine eingespeiste Taste bleibt sonst bis zum Ende gedrueckt."""
    ev = [(frame, KEY_DOWN, h) for h in halten]
    ev += [(frame, KEY_DOWN, code), (frame + 2, KEY_UP, code)]
    ev += [(frame + 2, KEY_UP, h) for h in halten]
    return ev


# ---------------------------------------------------------------- Stufe 2

def test_debugger_haelt_am_haltepunkt_und_schreitet(tmp_path):
    """Pfeil runter + F9 setzt den Haltepunkt in Zeile 2, F7 startet den
    Debugger: er haelt NICHT in Zeile 1 (dort steht kein Haltepunkt), sondern
    in 2; F10 schreitet nach 3; F8 laesst ihn zu Ende laufen."""
    (tmp_path / "spiel.dh").write_text('PRINT 1\nPRINT 2\nPRINT 3', encoding="utf-8")
    ev = _taste(20, RL_DOWN) + _taste(30, RL_F9) + _taste(40, RL_F7)
    ev += _taste(160, RL_F10) + _taste(260, RL_F8)
    log = _ide(tmp_path, tmp_path / "spiel.dh", frames=420, events=ev)
    assert "haltepunkt 2 an" in log, log
    assert "debug gestartet" in log, log
    pausen = [z for z in log if z.startswith("debug pause ")]
    assert pausen[:2] == ["debug pause 2", "debug pause 3"], log
    assert "debug beendet" in log, log


def test_debugger_ohne_haltepunkt_steht_in_zeile_eins(tmp_path):
    (tmp_path / "spiel.dh").write_text('PRINT 1\nPRINT 2', encoding="utf-8")
    log = _ide(tmp_path, tmp_path / "spiel.dh", frames=200, events=_taste(20, RL_F7))
    assert "debug pause 1" in log, log
    assert "debug beendet" not in log, log     # steht noch, als die Bilder ausgehen


def test_profil_zeigt_gemessene_zeilen(tmp_path):
    (tmp_path / "spiel.dh").write_text('DIM i AS INTEGER\nFOR i = 1 TO 200\n    PRINT i\nNEXT\n', encoding="utf-8")
    log = _ide(tmp_path, tmp_path / "spiel.dh", frames=300, events=_taste(20, RL_Y, RL_LCTRL, RL_LSHIFT))
    assert "profil gestartet" in log, log
    profil = [z for z in log if z.startswith("profil ") and z != "profil gestartet"]
    assert profil and int(profil[0].split()[1]) >= 3, log


def test_suche_im_projekt_findet_ueber_dateien(tmp_path):
    # Das Suchwort darf NICHT in der IDE selbst vorkommen -- ihre Testkopie
    # liegt im selben Ordner und wuerde mitgezaehlt.
    (tmp_path / "a.dh").write_text('PRINT "Xyzzy"\nPRINT "nix"\n', encoding="utf-8")
    (tmp_path / "b.dh").write_text('DIM xyzzy AS INTEGER\n', encoding="utf-8")
    # Strg+Umschalt+F oeffnet die Frage, Strg+V tippt den Suchtext, Enter sucht.
    ev = _taste(20, RL_F, RL_LCTRL, RL_LSHIFT) + _taste(50, RL_V, RL_LCTRL) + _taste(70, RL_ENTER)
    log = _ide(tmp_path, tmp_path / "a.dh", frames=150, events=ev, zwischenablage="xyzzy")
    assert "suche 2" in log, log


def test_befehlspalette_fuehrt_den_getippten_befehl_aus(tmp_path):
    (tmp_path / "spiel.dh").write_text('PRINT 1\n', encoding="utf-8")
    ev = _taste(20, RL_P, RL_LCTRL, RL_LSHIFT) + _taste(50, RL_V, RL_LCTRL) + _taste(70, RL_ENTER)
    log = _ide(tmp_path, tmp_path / "spiel.dh", frames=150, events=ev, zwischenablage="profil")
    assert "palette profil" in log, log
    assert "profil gestartet" in log, log


# ---------------------------------------------------------------- Stufe 3

def test_f1_schlaegt_das_wort_unter_der_marke_im_handbuch_nach(tmp_path):
    """Die Marke steht nach dem Oeffnen auf 1,1 -- vor `SPRITE_NEW`. F1 sucht
    das Wort in docs/ und oeffnet das Dokument mit den meisten Fundstellen in
    Codeschrift: das Sprite-Modul, nicht irgendeines, das das Wort erwaehnt."""
    (tmp_path / "spiel.dh").write_text('SPRITE_NEW(1, 2, 3)\n', encoding="utf-8")
    log = _ide(tmp_path, tmp_path / "spiel.dh", frames=150, events=_taste(30, RL_F1))
    treffer = [z for z in log if z.startswith("handbuch ")]
    assert treffer, log
    datei, zeile = treffer[0].split()[1], int(treffer[0].split()[2])
    assert datei == "module-sprite.md" and zeile >= 1, log


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


def test_werkzeug_startet_aus_dem_menue(tmp_path):
    (tmp_path / "spiel.dh").write_text('PRINT 1\n', encoding="utf-8")
    ev = _taste(20, RL_P, RL_LCTRL, RL_LSHIFT) + _taste(50, RL_V, RL_LCTRL) + _taste(70, RL_ENTER)
    log = _ide(tmp_path, tmp_path / "spiel.dh", frames=150, events=ev, zwischenablage="Werkzeug: SFX")
    assert "werkzeug 183_sfx_generator.dh" in log, log


def test_ausdruck_im_angehaltenen_debugger(tmp_path):
    """F7 haelt in Zeile 1 (kein Haltepunkt); Strg+E holt die Eingabezeile,
    Strg+V tippt den Ausdruck, Enter schickt ihn als `eval` -- die Antwort des
    Kindes landet im Protokoll."""
    (tmp_path / "spiel.dh").write_text('DIM a AS INTEGER\na = 20\nPRINT a\n', encoding="utf-8")
    ev = _taste(20, RL_F7) + _taste(120, RL_E, RL_LCTRL) + _taste(140, RL_V, RL_LCTRL) + _taste(160, RL_ENTER)
    log = _ide(tmp_path, tmp_path / "spiel.dh", frames=260, events=ev, zwischenablage="2 * 21")
    assert "debug pause 1" in log, log
    assert "eval 42" in log, log
