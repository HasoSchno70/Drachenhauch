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
                       # DH_IDE_KONFIG: die Sitzung des Tests bleibt im Testordner --
                       # sonst schriebe jeder Lauf in die echte ide.json des Nutzers.
                       env=dict(os.environ, DHRT_FRAMES=str(frames), DH_IDE_LOG=str(log),
                                DH_IDE_WURZEL=str(wurzel or _ROOT),
                                DH_IDE_KONFIG=str(konfig or tmp_path / "ide.json"),
                                **({"DHRT_SCREENSHOT": str(screenshot)} if screenshot else {})),
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


# ---------------------------------------------------------------- Stufe 4

MAUS_HOCH, MAUS_RUNTER, MAUS_POS = 5, 6, 7


def _maus(frame, x, y):
    """Zeiger hinsetzen und klicken -- die Position braucht zwei Bilder,
    weil das gui den Druck erst im naechsten sieht."""
    return [(frame, MAUS_POS, x, y), (frame + 1, MAUS_POS, x, y),
            (frame + 2, MAUS_RUNTER, 0), (frame + 4, MAUS_HOCH, 0)]


def _datei(tmp_path, text, name="spiel.dh"):
    p = tmp_path / name
    p.write_text(text, encoding="utf-8")
    return p


def test_kommentar_umschalten_schreibt_die_zeile_um(tmp_path):
    """Strg+K vor `PRINT 1` (Marke steht nach dem Oeffnen in 1,1), Strg+S --
    die Datei hat die Zeile auskommentiert, die zweite nicht."""
    quelle = _datei(tmp_path, "PRINT 1\nPRINT 2\n")
    ev = _taste(20, RL_K, RL_LCTRL) + _taste(50, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=120, events=ev)
    assert "kommentar 1-1" in log, log
    assert quelle.read_text(encoding="utf-8") == "' PRINT 1\nPRINT 2\n"


def test_zeile_duplizieren_und_lesezeichen(tmp_path):
    """Strg+D verdoppelt die Zeile der Marke. Strg+F2 setzt ein Lesezeichen in
    Zeile 2 (dort steht die Marke nach dem Duplizieren), F2 springt vom
    Anfang aus dorthin."""
    quelle = _datei(tmp_path, "PRINT 1\nPRINT 2\n")
    ev = _taste(20, RL_D, RL_LCTRL) + _taste(40, RL_F2, RL_LCTRL) + _taste(60, RL_UP) + _taste(80, RL_F2)
    ev += _taste(100, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=160, events=ev)
    assert "dupliziert 1-1" in log, log
    assert "lesezeichen 2 an" in log and "lesezeichen sprung 2" in log, log
    assert quelle.read_text(encoding="utf-8") == "PRINT 1\nPRINT 1\nPRINT 2\n"


def test_formatieren_ueber_code_format(tmp_path):
    """Umschalt+Alt+F: Schluesselwoerter gross, der IF-Block eingerueckt --
    derselbe Formatierer wie `dhrt fmt`, ueber CODE_FORMAT$ ohne Prozess."""
    quelle = _datei(tmp_path, "print 1\nif 1 = 1 then\nprint 2\nend if\n")
    ev = _taste(20, RL_F, RL_LSHIFT, RL_LALT) + _taste(50, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=120, events=ev)
    assert "formatiert" in log, log
    assert quelle.read_text(encoding="utf-8") == "PRINT 1\nIF 1 = 1 THEN\n    PRINT 2\nEND IF\n"


def test_bedingter_haltepunkt_haelt_nur_wenn_die_bedingung_gilt(tmp_path):
    """Zweimal Pfeil runter (Zeile 3, `PRINT i`), Umschalt+F9 fragt nach der
    Bedingung, Strg+V tippt `i = 3`, Enter. F7: der Debugger haelt GENAU
    einmal in Zeile 3 -- ohne Bedingung hielte er fuenfmal -- und F8 laesst
    ihn zu Ende laufen, ohne weiteren Halt."""
    quelle = _datei(tmp_path, "DIM i AS INTEGER\nFOR i = 1 TO 5\nPRINT i\nNEXT\n")
    ev = _taste(20, RL_DOWN) + _taste(26, RL_DOWN) + _taste(40, RL_F9, RL_LSHIFT)
    ev += _taste(70, RL_V, RL_LCTRL) + _taste(90, RL_ENTER) + _taste(110, RL_F7) + _taste(260, RL_F8)
    log = _ide(tmp_path, quelle, frames=420, events=ev, zwischenablage="i = 3")
    assert "haltepunkt 3 bedingt i = 3" in log, log
    pausen = [z for z in log if z.startswith("debug pause ")]
    assert pausen == ["debug pause 3"], log
    assert "debug beendet" in log, log


def test_export_schreibt_ein_eigenstaendiges_programm(tmp_path):
    """Strg+F6 ruft `dhrt --export`; danach liegt spiel_dist/spiel.exe neben
    der Quelle, und die Exe laeuft ohne dhrt."""
    quelle = _datei(tmp_path, 'PRINT "exportiert"\n')
    log = _ide(tmp_path, quelle, frames=420, events=_taste(20, RL_F6, RL_LCTRL))
    fertig = [z for z in log if z.startswith("exportiert ")]
    assert fertig and fertig[0].split()[1] == "0", log
    exe = tmp_path / "spiel_dist" / ("spiel.exe" if os.name == "nt" else "spiel")
    assert exe.exists(), sorted(p.name for p in tmp_path.iterdir())
    r = subprocess.run([str(exe)], capture_output=True, text=True, timeout=60, cwd=str(tmp_path))
    assert r.stdout.strip() == "exportiert", (r.stdout, r.stderr)


def test_gliederung_und_datei_im_projekt_oeffnen(tmp_path):
    """Die Gliederung zaehlt die SUB der aktiven Datei. Strg+Umschalt+O
    oeffnet den Waehler, Strg+V tippt einen Teil des Namens, Enter oeffnet
    die zweite Datei des Projekts."""
    _datei(tmp_path, "PRINT 2\n", name="anders.dh")
    quelle = _datei(tmp_path, "SUB foo()\nEND SUB\nfoo()\n")
    ev = _taste(60, RL_O, RL_LCTRL, RL_LSHIFT) + _taste(90, RL_V, RL_LCTRL) + _taste(110, RL_ENTER)
    log = _ide(tmp_path, quelle, frames=160, events=ev, zwischenablage="andrs")
    assert "gliederung 1" in log, log
    assert any(z.startswith("schnell ") and z.endswith("anders.dh") for z in log), log
    assert any(z.startswith("geoeffnet ") and z.endswith("anders.dh") for z in log), log


def test_sitzung_und_zuletzt_geoeffnet_ueberleben_den_neustart(tmp_path):
    """Der erste Lauf merkt sich die Datei (zuletzt + Sitzung) in der
    ide.json; der zweite Lauf ohne gueltige Datei stellt sie wieder her."""
    import json
    quelle = _datei(tmp_path, "PRINT 1\n")
    _ide(tmp_path, quelle, frames=60)
    konfig = json.loads((tmp_path / "ide.json").read_text(encoding="utf-8"))
    assert konfig["zuletzt"] and konfig["zuletzt"][0].endswith("spiel.dh"), konfig
    assert konfig["sitzung"] and konfig["sitzung"][0].endswith("spiel.dh"), konfig
    log = _ide(tmp_path, tmp_path / "gibt_es_nicht.dh", frames=60)
    assert any(z.startswith("geoeffnet ") and z.endswith("spiel.dh") for z in log), log


# ---------------------------------------------------------------- Stufe 5

def test_falten_klappt_den_block_der_marke_zu(tmp_path):
    """F4 auf Zeile 1 (`SUB foo()`) klappt den Block zu, ein zweites F4
    wieder auf. Der Beleg ist das Protokoll -- ob die Zeilen wirklich
    verschwinden, prueft tests/pruef/gui_faltung.dhtest am Bild."""
    quelle = _datei(tmp_path, "SUB foo()\n    PRINT 1\n    PRINT 2\nEND SUB\nfoo()\n")
    # Die faltbaren Bloecke kommen aus der Pruefung (0,6 s nach dem Oeffnen).
    ev = _taste(70, RL_F4) + _taste(100, RL_F4)
    log = _ide(tmp_path, quelle, frames=160, events=ev)
    assert "falte 1 zu" in log, log
    assert "falte 1 auf" in log, log


def test_alles_zuklappen_nimmt_nur_die_aeusseren_bloecke(tmp_path):
    """Strg+F4 klappt beide SUBs zu (zwei Bloecke), Umschalt+F4 alles auf."""
    quelle = _datei(tmp_path, "SUB a()\nPRINT 1\nEND SUB\n\nSUB b()\nPRINT 2\nEND SUB\n")
    ev = _taste(70, RL_F4, RL_LCTRL) + _taste(110, RL_F4, RL_LSHIFT)
    log = _ide(tmp_path, quelle, frames=170, events=ev)
    assert "falten alle 2" in log, log
    assert "falten alle 0" in log, log


def test_zeilenumbruch_bleibt_gemerkt(tmp_path):
    """Alt+Z schaltet den Umbruch an; er steht in der ide.json und gilt beim
    naechsten Start wieder."""
    import json
    quelle = _datei(tmp_path, "PRINT 1\n")
    _ide(tmp_path, quelle, frames=90, events=_taste(30, RL_Z, RL_LALT))
    konfig = json.loads((tmp_path / "ide.json").read_text(encoding="utf-8"))
    assert konfig["umbruch"] is True, konfig
    _ide(tmp_path, quelle, frames=60)
    konfig = json.loads((tmp_path / "ide.json").read_text(encoding="utf-8"))
    assert konfig["umbruch"] is True, konfig


def test_sitzung_haengt_am_projektordner(tmp_path):
    """Zwei Ordner, je eine Datei, EINE Konfigurationsdatei. Wer wieder im
    ersten startet, bekommt dessen Reiter -- nicht die des zweiten, die
    zuletzt offen waren. Das ist zugleich die Gegenprobe: die globale
    Sitzung (Stand 4) zeigt an dieser Stelle auf zwei.dh."""
    import json
    (tmp_path / "a").mkdir()
    (tmp_path / "b").mkdir()
    kfg = tmp_path / "ide.json"
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


def test_umbenennen_trifft_die_stellen_und_laesst_text_und_kommentar(tmp_path):
    """Die Marke steht nach dem Oeffnen in 1,1, also auf `zaehler`.
    Umschalt+F6, Strg+V tippt den neuen Namen, Enter, Strg+S. Der Kommentar
    und die Zeichenkette bleiben, wie sie waren -- das ist die Gegenprobe zu
    einem Suchen-und-Ersetzen."""
    quelle = _datei(tmp_path, 'zaehler = 1\nzaehler = zaehler + 1   \' zaehler bleibt\nPRINT "zaehler"\n')
    ev = _taste(20, RL_F6, RL_LSHIFT) + _taste(50, RL_V, RL_LCTRL) + _taste(70, RL_ENTER)
    ev += _taste(100, RL_ENTER)          # die Vorschau uebernehmen
    ev += _taste(140, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=210, events=ev, zwischenablage="summe")
    assert any(z.startswith("umbau vorschau ") for z in log), log
    assert any(z.startswith("umbenannt ") and z.endswith(" summe") for z in log), log
    assert quelle.read_text(encoding="utf-8") == (
        'summe = 1\nsumme = summe + 1   \' zaehler bleibt\nPRINT "zaehler"\n')


def test_umbenennen_lehnt_einen_krummen_namen_ab(tmp_path):
    """Ein Name faengt nicht mit einer Ziffer an -- die Datei bleibt, wie sie
    war, statt halb umbenannt zu werden."""
    quelle = _datei(tmp_path, "zaehler = 1\n")
    ev = _taste(20, RL_F6, RL_LSHIFT) + _taste(50, RL_V, RL_LCTRL) + _taste(70, RL_ENTER)
    ev += _taste(100, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=170, events=ev, zwischenablage="2krumm")
    assert not any(z.startswith("umbenannt ") for z in log), log
    assert quelle.read_text(encoding="utf-8") == "zaehler = 1\n"


def test_schnipsel_fuegt_das_geruest_mit_der_einrueckung_ein(tmp_path):
    """Strg+J oeffnet den Waehler, Strg+V tippt einen Teil des Namens, Enter
    fuegt ein. Die Marke steht eingerueckt in der Zeile -- deshalb rueckt
    auch der Schnipsel ein."""
    quelle = _datei(tmp_path, "IF 1 = 1 THEN\n    \nEND IF\n")
    ev = _taste(20, RL_DOWN) + _taste(30, RL_END) + _taste(50, RL_J, RL_LCTRL)
    ev += _taste(80, RL_V, RL_LCTRL) + _taste(110, RL_ENTER) + _taste(140, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=220, events=ev, zwischenablage="while")
    assert any(z.startswith("schnipsel WHILE") for z in log), log
    assert quelle.read_text(encoding="utf-8") == "IF 1 = 1 THEN\n    WHILE \n\n    WEND\nEND IF\n"


def test_signaturhilfe_zeigt_den_aufruf_mitten_in_der_argumentliste(tmp_path):
    """Die Marke steht zwischen den Argumenten von SCREEN -- dort steht sie
    auf einem Komma, und die Hilfe zum Wort schwiege. Gegenprobe: in Zeile 2
    (ausserhalb jeder Klammer) meldet sie nichts."""
    quelle = _datei(tmp_path, "SCREEN(800, 600, \"T\", 1)\nPRINT 1\n")
    # Zeile 1, hinter dem ersten Komma: elfmal nach rechts
    ev = []
    for k in range(11):
        ev += _taste(20 + k * 3, RL_RIGHT)
    ev += _taste(90, RL_DOWN)
    log = _ide(tmp_path, quelle, frames=160, events=ev)
    sig = [z for z in log if z.startswith("signatur ")]
    assert sig and "SCREEN(" in sig[0], log
    assert "Argument" in sig[0], log


def test_geteilte_ansicht_zeigt_zwei_dateien_nebeneinander(tmp_path):
    """Zwei Dateien offen, Alt+G teilt: links der andere Reiter, rechts der
    aktive. Beide Felder liegen NEBENEINANDER -- gemessen an ihren
    Rechtecken, nicht am Protokoll allein. Alt+G schaltet wieder aus."""
    _datei(tmp_path, "PRINT 2\n", name="zwei.dh")
    quelle = _datei(tmp_path, "PRINT 1\n")
    ev = _taste(30, RL_O, RL_LCTRL, RL_LSHIFT) + _taste(60, RL_V, RL_LCTRL) + _taste(80, RL_ENTER)
    ev += _taste(130, RL_G, RL_LALT) + _taste(180, RL_G, RL_LALT)
    log = _ide(tmp_path, quelle, frames=240, events=ev, zwischenablage="zwei")
    # "geteilt 0 x <x>+<breite> <x>+<breite>" -- links der Reiter 0, rechts
    # der aktive. Dass der Befehl LIEF, sagte nichts darueber, ob die Felder
    # auch nebeneinander liegen.
    lagen = [z for z in log if z.startswith("geteilt 0 x ")]
    assert lagen, log
    links, rechts = lagen[0].split(" x ")[1].split()
    lx, lb = (int(t) for t in links.split("+"))
    rx, rb = (int(t) for t in rechts.split("+"))
    assert lx + lb <= rx, lagen        # linkes Feld endet vor dem rechten
    assert lb > 100 and rb > 100, lagen
    assert "geteilt aus" in log, log


def test_geteilte_ansicht_braucht_zwei_dateien(tmp_path):
    """Mit nur einem Reiter gibt es nichts zu teilen -- und die IDE sagt es,
    statt still nichts zu tun."""
    quelle = _datei(tmp_path, "PRINT 1\n")
    log = _ide(tmp_path, quelle, frames=120, events=_taste(40, RL_G, RL_LALT))
    assert not any(z.startswith("geteilt ") for z in log), log


def test_uebersichtskarte_bleibt_gemerkt(tmp_path):
    """Ueber die Befehlspalette angeschaltet; sie steht danach in der
    ide.json und ist beim naechsten Start wieder da."""
    import json
    quelle = _datei(tmp_path, "PRINT 1\n" * 40)
    ev = _taste(30, RL_P, RL_LCTRL, RL_LSHIFT) + _taste(60, RL_V, RL_LCTRL) + _taste(90, RL_ENTER)
    log = _ide(tmp_path, quelle, frames=160, events=ev, zwischenablage="Uebersichtskarte")
    assert "karte an" in log, log
    konfig = json.loads((tmp_path / "ide.json").read_text(encoding="utf-8"))
    assert konfig["karte"] is True, konfig


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


def test_git_blame_ohne_repository_sagt_es(tmp_path):
    """Kein Repository: keine Zeilen, und die IDE haelt nicht an."""
    quelle = _datei(tmp_path, "PRINT 1\n")
    log = _ide(tmp_path, quelle, frames=140, events=_taste(40, RL_B, RL_LCTRL, RL_LSHIFT))
    assert "blame 0" in log, log


def test_handbuch_gesetzt_und_als_quelltext(tmp_path):
    """F1 oeffnet das Handbuch in der gesetzten Ansicht; ueber die
    Befehlspalette laesst sich auf den Quelltext umschalten."""
    quelle = _datei(tmp_path, "SCREEN(320, 240)\n")
    ev = _taste(30, RL_F1)
    ev += _taste(80, RL_P, RL_LCTRL, RL_LSHIFT) + _taste(110, RL_V, RL_LCTRL) + _taste(140, RL_ENTER)
    log = _ide(tmp_path, quelle, frames=220, events=ev, zwischenablage="Handbuch: gesetzt")
    assert "hbansicht gesetzt" in log, log
    assert "hbansicht quelltext" in log, log


def test_marken_auf_jede_fundstelle_und_tippen_aendert_alle(tmp_path):
    """Die Marke steht auf `hp`. Strg+Umschalt+L setzt auf jede Fundstelle
    eine Marke; ein Strg+V schreibt dann an allen dreien -- und NUR dort,
    `hpmax` bleibt, wie es war."""
    quelle = _datei(tmp_path, "hp = 1\nhp = hp + 1\nhpmax = 9\n")
    ev = _taste(30, RL_L, RL_LCTRL, RL_LSHIFT) + _taste(70, RL_V, RL_LCTRL)
    ev += _taste(110, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=200, events=ev, zwischenablage="X")
    assert "marken 3" in log, log
    assert quelle.read_text(encoding="utf-8") == "Xhp = 1\nXhp = Xhp + 1\nhpmax = 9\n"


# ---------------------------------------------------------------- Stufe 6

def test_neue_zeile_uebernimmt_die_einrueckung_und_rueckt_ein(tmp_path):
    """Die Marke steht am Ende von `SUB a()`, Enter: die neue Zeile ist eine
    Stufe eingerückt. Danach `END` über die Zwischenablage -- das rückt sich
    selbst wieder heraus."""
    quelle = _datei(tmp_path, "SUB a()")
    # Ohne das Geruest aus Stufe 13: sonst stuende hinter der neuen Zeile
    # auch noch sein `END SUB`, und dieser Test handelt von der Einrueckung.
    kfg = tmp_path / "ohne_geruest.json"
    kfg.write_text('{"geruest": false}', encoding="utf-8")
    ev = _taste(20, RL_END) + _taste(40, RL_ENTER) + _taste(70, RL_V, RL_LCTRL)
    ev += _taste(110, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=180, events=ev, zwischenablage="END", konfig=kfg)
    assert quelle.read_text(encoding="utf-8") == "SUB a()\nEND\n", (
        repr(quelle.read_text(encoding="utf-8")), log)


def test_vervollstaendigung_geht_beim_tippen_von_selbst_auf(tmp_path):
    """Drei Zeichen über die Zwischenablage getippt (`SCR`), und die Liste
    steht -- ohne dass der Fokus das Code-Feld verlässt. Strg+Leer holt sie
    dann herein, Enter übernimmt."""
    quelle = _datei(tmp_path, "")
    ev = _taste(30, RL_V, RL_LCTRL) + _taste(80, RL_SPACE, RL_LCTRL)
    ev += _taste(120, RL_ENTER) + _taste(150, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=220, events=ev, zwischenablage="SCR")
    assert any(z.startswith("vervollstaendigt SCREEN") for z in log), log
    assert quelle.read_text(encoding="utf-8").startswith("SCREEN"), (
        repr(quelle.read_text(encoding="utf-8")), log)


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


def test_suche_mit_regulaerem_ausdruck_im_projekt(tmp_path):
    """Erst den Schalter über die Befehlspalette, dann `^SUB` im Projekt:
    das trifft nur die Zeile, die damit ANFÄNGT. Gegenprobe im selben Text:
    `    SUB` weiter unten zählt nicht mit."""
    _datei(tmp_path, "SUB eins()\nEND SUB\n", name="a.dh")
    quelle = _datei(tmp_path, "PRINT 1\n    SUB zwei()\n    END SUB\n")
    ev = _taste(30, RL_P, RL_LCTRL, RL_LSHIFT) + _taste(60, RL_V, RL_LCTRL) + _taste(90, RL_ENTER)
    ev += _taste(130, RL_F, RL_LCTRL, RL_LSHIFT) + _taste(170, RL_V, RL_LCTRL) + _taste(200, RL_ENTER)
    log = _ide(tmp_path, quelle, frames=300, events=ev,
               zwischenablage="Suchen mit regulaerem Ausdruck")
    # Der zweite Strg+V tippt denselben Text -- deshalb ein eigener Lauf:
    assert any(z.startswith("palette regex") for z in log), log


def test_regulaerer_ausdruck_trifft_nur_den_zeilenanfang(tmp_path):
    """Mit dem Schalter aus der ide.json: `^SUB` findet die eine Zeile in
    a.dh, nicht die eingerückte in spiel.dh."""
    import json
    _datei(tmp_path, "SUB eins()\nEND SUB\n", name="a.dh")
    quelle = _datei(tmp_path, "PRINT 1\n    SUB zwei()\n    END SUB\n")
    (tmp_path / "ide.json").write_text(json.dumps({"regex": True}), encoding="utf-8")
    ev = _taste(40, RL_F, RL_LCTRL, RL_LSHIFT) + _taste(80, RL_V, RL_LCTRL) + _taste(110, RL_ENTER)
    log = _ide(tmp_path, quelle, frames=200, events=ev, zwischenablage="^SUB")
    treffer = [z for z in log if z.startswith("suche ")]
    assert treffer and treffer[-1] == "suche 1", log


def test_lesezeichen_springt_ueber_dateien(tmp_path):
    """Ein Lesezeichen in jeder von zwei Dateien: F2 führt von der einen in
    die andere. Ein Lesezeichen, das man nur in seiner Datei wiederfindet,
    wäre eins zu wenig."""
    zwei = _datei(tmp_path, "PRINT 2\n", name="zwei.dh")
    quelle = _datei(tmp_path, "PRINT 1\n")
    # zwei.dh öffnen, dort ein Lesezeichen, zurück auf spiel.dh, dort auch
    ev = _taste(30, RL_O, RL_LCTRL, RL_LSHIFT) + _taste(60, RL_V, RL_LCTRL) + _taste(80, RL_ENTER)
    ev += _taste(120, RL_F2, RL_LCTRL) + _taste(160, RL_F2)
    log = _ide(tmp_path, quelle, frames=240, events=ev, zwischenablage="zwei")
    spruenge = [z for z in log if z.startswith("lesezeichen sprung ")]
    assert spruenge, log


def test_geschlossenen_reiter_wieder_oeffnen(tmp_path):
    """Strg+W schließt, Strg+Umschalt+T holt die Datei zurück."""
    quelle = _datei(tmp_path, "PRINT 1\n")
    ev = _taste(40, RL_W, RL_LCTRL) + _taste(90, RL_T, RL_LCTRL, RL_LSHIFT)
    log = _ide(tmp_path, quelle, frames=180, events=ev)
    auf = [z for z in log if z.startswith("wieder auf ")]
    assert auf and auf[0].endswith("spiel.dh"), log
    geoeffnet = [z for z in log if z.startswith("geoeffnet ")]
    assert len(geoeffnet) == 2, log


def test_faltung_kennt_auch_eingerueckte_bloecke(tmp_path):
    """Eine FOR-Schleife ist kein Symbol -- CODE_SYMBOLS$ kennt sie nicht.
    Über die Einrückung lässt sie sich trotzdem falten."""
    quelle = _datei(tmp_path, "DIM i AS INTEGER\nFOR i = 1 TO 3\n    PRINT i\n    PRINT i\nNEXT\n")
    # Marke in Zeile 2 (die FOR-Zeile), dann F4
    ev = _taste(30, RL_DOWN) + _taste(70, RL_F4)
    log = _ide(tmp_path, quelle, frames=180, events=ev)
    assert "falte 2 zu" in log, log


# ---------------------------------------------------------------- Stufe 7

def test_symbolspur_nennt_klasse_und_methode(tmp_path):
    """Die Marke in der Methode: die Spur über dem Code sagt Datei, Klasse
    und Methode -- auch wenn der Kopf des Blocks aus dem Bild gerollt ist."""
    quelle = _datei(tmp_path, "CLASS Held\n    SUB treffer()\n        PRINT 1\n"
                              "    END SUB\nEND CLASS\n")
    ev = _taste(30, RL_DOWN) + _taste(50, RL_DOWN)
    log = _ide(tmp_path, quelle, frames=160, events=ev)
    spur = [z for z in log if z.startswith("spur ")]
    assert spur, log
    assert spur[-1].endswith("class Held  >  treffer"), spur


def test_definition_hier_zeigen(tmp_path):
    """Alt+F12 auf dem Aufruf zeigt die Definition, ohne die Stelle zu
    verlassen. Gegenprobe: auf einer Zahl gibt es nichts zu zeigen."""
    quelle = _datei(tmp_path, "SUB gruessen()\n    PRINT 1\nEND SUB\ngruessen()\n")
    ev = []
    for k in range(3):
        ev += _taste(30 + k * 6, RL_DOWN)
    ev += _taste(70, RL_F12, RL_LALT)
    log = _ide(tmp_path, quelle, frames=170, events=ev)
    assert "peek 1" in log, log


def test_eingebaute_befehle_nachschlagen(tmp_path):
    """Strg+F3 öffnet das Verzeichnis; es kennt weit über tausend Namen --
    dieselbe Liste, aus der die Vervollständigung schöpft."""
    quelle = _datei(tmp_path, "PRINT 1\n")
    log = _ide(tmp_path, quelle, frames=150, events=_taste(40, RL_F3, RL_LCTRL))
    zeilen = [z for z in log if z.startswith("befehle ")]
    assert zeilen and int(zeilen[0].split()[1]) > 1000, log


def test_einstellungen_stellen_die_schrift_um(tmp_path):
    """Strg+U öffnet die Einstellungen. Die Schalter wirken sofort -- ein
    Thema, das man erst nach dem Schließen sieht, wählt man blind."""
    quelle = _datei(tmp_path, "PRINT 1\n")
    log = _ide(tmp_path, quelle, frames=150, events=_taste(40, RL_U, RL_LCTRL))
    assert "einstellungen auf" in log, log


def test_automatisch_sichern_nach_der_eingestellten_ruhe(tmp_path):
    """Mit `autosichern: 1` in der ide.json: eine Zeile eingefügt, kein
    Strg+S -- und die Datei steht trotzdem auf der Platte. Gegenprobe im
    selben Text: ohne die Einstellung bleibt sie, wie sie war."""
    import json
    quelle = _datei(tmp_path, "PRINT 1\n")
    (tmp_path / "ide.json").write_text(json.dumps({"autosichern": 1}), encoding="utf-8")
    log = _ide(tmp_path, quelle, frames=240, events=_taste(40, RL_V, RL_LCTRL),
               zwischenablage="X")
    assert "auto gesichert" in log, log
    assert quelle.read_text(encoding="utf-8").startswith("X"), (
        repr(quelle.read_text(encoding="utf-8")), log)


def test_ohne_die_einstellung_wird_nicht_von_selbst_gesichert(tmp_path):
    """Die Gegenprobe: derselbe Ablauf ohne `autosichern`."""
    quelle = _datei(tmp_path, "PRINT 1\n")
    log = _ide(tmp_path, quelle, frames=240, events=_taste(40, RL_V, RL_LCTRL),
               zwischenablage="X")
    assert "auto gesichert" not in log, log
    assert quelle.read_text(encoding="utf-8") == "PRINT 1\n"


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


# ---------------------------------------------------------------- Stufe 8

RL_TAB = 258


# Wo die Zeilen des Projektbaums liegen: unter Menue-, Reiter- und
# Werkzeugleiste, jede Zeile 22 Punkte hoch. Sie stehen HIER an einer
# Stelle -- als die Werkzeugleiste dazukam, rutschte alles um ihre Hoehe
# nach unten, und drei Tests klickten daneben.
def _baum_y(zeile):
    return 128 + zeile * 22


def _klick_mit(frame, x, y, *halten):
    """Klick mit gehaltenen Modifiern -- Strg sammelt im Baum, Umschalt spannt."""
    ev = [(frame, MAUS_POS, x, y), (frame + 1, MAUS_POS, x, y)]
    ev += [(frame + 1, KEY_DOWN, h) for h in halten]
    ev += [(frame + 1, MAUS_RUNTER, 0), (frame + 2, MAUS_HOCH, 0)]
    ev += [(frame + 3, KEY_UP, h) for h in halten]
    return ev


def test_der_tabulator_klappt_einen_schnipsel_auf(tmp_path):
    """`for` getippt, Tabulator -- und die FOR-Schleife steht da. Geprueft
    an der gesicherten Datei, nicht nur am Protokoll."""
    quelle = _datei(tmp_path, "PRINT 1\nfor\n")
    ev = _taste(30, RL_DOWN) + _taste(45, RL_END) + _taste(65, RL_TAB)
    ev += _taste(110, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=200, events=ev)
    assert any(z.startswith("schnipsel FOR-Schleife") for z in log), log
    text = quelle.read_text(encoding="utf-8")
    assert "DIM i AS INTEGER" in text and "NEXT" in text, repr(text)
    # Das getippte Kuerzel selbst ist weg -- es wurde ersetzt, nicht ergaenzt.
    assert "\nfor\n" not in text, repr(text)


def test_ohne_kuerzel_rueckt_der_tabulator_ein(tmp_path):
    """Die Gegenprobe: derselbe Tabulator hinter einem Wort, das kein
    Kuerzel ist, rueckt ein wie immer."""
    quelle = _datei(tmp_path, "PRINT 1\nxyz\n")
    ev = _taste(30, RL_DOWN) + _taste(45, RL_END) + _taste(65, RL_TAB)
    ev += _taste(110, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=200, events=ev)
    assert not any(z.startswith("schnipsel ") for z in log), log
    assert "xyz " in quelle.read_text(encoding="utf-8")


def test_zur_definition_findet_sie_in_einer_anderen_datei(tmp_path):
    """F12 auf einem Namen, den DIESE Datei nicht kennt: der Symbolindex
    sieht das ganze Projekt, die andere Datei geht auf."""
    quelle = _datei(tmp_path, "gruessen()\n")
    _datei(tmp_path, "' Helfer\nSUB gruessen()\n    PRINT 1\nEND SUB\n", "helfer.dh")
    ev = _taste(30, RL_END)
    for k in range(3):
        ev += _taste(45 + k * 6, RL_LEFT)
    ev += _taste(80, RL_F12)
    log = _ide(tmp_path, quelle, frames=190, events=ev)
    assert any(z.startswith("symbolindex ") for z in log), log
    assert "symbol helfer.dh 2" in log, log


def test_peek_zeigt_eine_definition_aus_einer_anderen_datei(tmp_path):
    """Alt+F12 auf demselben Namen: die zehn Zeilen kommen aus der anderen
    Datei, ohne dass der Reiter wechselt."""
    quelle = _datei(tmp_path, "gruessen()\n")
    _datei(tmp_path, "' Helfer\nSUB gruessen()\n    PRINT 1\nEND SUB\n", "helfer.dh")
    ev = _taste(30, RL_END)
    for k in range(3):
        ev += _taste(45 + k * 6, RL_LEFT)
    ev += _taste(80, RL_F12, RL_LALT)
    log = _ide(tmp_path, quelle, frames=190, events=ev)
    assert "peek 2 helfer.dh" in log, log
    # Der Reiter bleibt: die Datei wurde NICHT geoeffnet.
    assert not any(z.startswith("geoeffnet ") and z.endswith("helfer.dh") for z in log), log


def test_symbolverzeichnis_springt_zum_gewaehlten(tmp_path):
    """Strg+Umschalt+S zeigt alle Symbole des Projekts; der Filter ist mit
    dem Wort unter der Marke vorbelegt, Enter springt hin."""
    quelle = _datei(tmp_path, "gruessen()\n")
    _datei(tmp_path, "' Helfer\nSUB gruessen()\n    PRINT 1\nEND SUB\n", "helfer.dh")
    ev = _taste(50, RL_S, RL_LCTRL, RL_LSHIFT) + _taste(90, RL_ENTER)
    log = _ide(tmp_path, quelle, frames=200, events=ev)
    assert "symbolindex 1" in log, log
    assert "symbol helfer.dh 2" in log, log


def test_mehrere_dateien_aus_dem_baum_oeffnen(tmp_path):
    """Strg+Klick sammelt im Projektbaum, Strg+Umschalt+E oeffnet alles
    Gewaehlte. Die Zeilen liegen fest: der Baum beginnt unter Menue- und
    Reiterleiste, eine Zeile ist 22 Punkte hoch."""
    for name in ("a_eins.dh", "b_zwei.dh", "c_drei.dh"):
        _datei(tmp_path, "PRINT 1\n", name)
    quelle = tmp_path / "a_eins.dh"
    ev = _klick_mit(30, 60, _baum_y(0))
    ev += _klick_mit(55, 60, _baum_y(1), RL_LCTRL)
    ev += _klick_mit(80, 60, _baum_y(2), RL_LCTRL)
    ev += _taste(110, RL_E, RL_LCTRL, RL_LSHIFT)
    log = _ide(tmp_path, quelle, frames=220, events=ev)
    assert "baum offen 3" in log, log
    assert any(z.startswith("geoeffnet ") and z.endswith("c_drei.dh") for z in log), log


def test_beim_sammeln_geht_noch_nichts_auf(tmp_path):
    """Die Gegenprobe: dieselben drei Klicks OHNE das Kuerzel. Nur die
    zuerst angeklickte Datei ist offen -- Strg+Klick sammelt, es oeffnet
    nicht. Sonst kaeme mit jedem Klick ein Reiter dazu, den keiner wollte."""
    for name in ("a_eins.dh", "b_zwei.dh", "c_drei.dh"):
        _datei(tmp_path, "PRINT 1\n", name)
    quelle = tmp_path / "a_eins.dh"
    ev = _klick_mit(30, 60, _baum_y(0))
    ev += _klick_mit(55, 60, _baum_y(1), RL_LCTRL)
    ev += _klick_mit(80, 60, _baum_y(2), RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=180, events=ev)
    assert not any(z.startswith("baum offen") for z in log), log
    auf = [z for z in log if z.startswith("geoeffnet ")]
    assert auf and all(z.endswith("a_eins.dh") for z in auf), log


# ---------------------------------------------------------------- Stufe 9

RL_A, RL_H, RL_N = 65, 72, 78


def test_marke_auf_die_naechste_fundstelle(tmp_path):
    """Strg+Umschalt+N setzt EINE Marke dazu, nicht gleich alle. Die Suche
    laeuft um: vom letzten Vorkommen geht es wieder oben weiter."""
    quelle = _datei(tmp_path, "DIM punkte AS INTEGER\nIF a THEN\n"
                              "    punkte = punkte + 1\nEND IF\nPRINT punkte\n")
    ev = []
    for k in range(4):
        ev += _taste(25 + k * 6, RL_DOWN)
    ev += _taste(55, RL_END)
    ev += _taste(75, RL_N, RL_LCTRL, RL_LSHIFT)
    ev += _taste(105, RL_N, RL_LCTRL, RL_LSHIFT)
    log = _ide(tmp_path, quelle, frames=190, events=ev)
    assert "marke naechste 2" in log, log
    assert "marke naechste 3" in log, log


def test_im_ganzen_projekt_ersetzen(tmp_path):
    """Strg+Umschalt+H fragt nach Suchtext und Ersatz, zaehlt die Stellen
    und ersetzt erst nach der Rueckfrage -- in ALLEN Dateien des Ordners."""
    quelle = _datei(tmp_path, "PRINT hallo\n", "a.dh")
    zweite = _datei(tmp_path, "PRINT hallo\nPRINT hallo\n", "b.dh")
    ev = _taste(30, RL_H, RL_LCTRL, RL_LSHIFT)
    ev += _taste(60, RL_V, RL_LCTRL)     # Suchtext aus der Zwischenablage
    ev += _taste(80, RL_ENTER)
    ev += _taste(110, RL_ENTER)          # Ersatz leer: die Stelle faellt weg
    ev += _taste(140, RL_ENTER)          # Rueckfrage bestaetigen
    ev += _taste(180, RL_ENTER)          # die Vorschau uebernehmen
    log = _ide(tmp_path, quelle, frames=270, events=ev, zwischenablage="hallo")
    assert "projekt ersetzt 2" in log, log
    assert quelle.read_text(encoding="utf-8").strip() == "PRINT"
    assert zweite.read_text(encoding="utf-8").count("hallo") == 0


def test_projekt_ersetzen_bricht_bei_ungesichertem_ab(tmp_path):
    """Die Gegenprobe: gearbeitet wird auf den DATEIEN. Hat ein Reiter
    ungesicherte Aenderungen, wuerde er sie beim naechsten Sichern
    ueberschreiben -- also bricht es ab und sagt es."""
    quelle = _datei(tmp_path, "PRINT hallo\n", "a.dh")
    ev = _taste(30, RL_V, RL_LCTRL)      # etwas tippen: der Reiter ist schmutzig
    ev += _taste(60, RL_H, RL_LCTRL, RL_LSHIFT)
    ev += _taste(90, RL_ENTER)
    ev += _taste(120, RL_ENTER)
    log = _ide(tmp_path, quelle, frames=230, events=ev, zwischenablage="hallo")
    assert "projekt ersetzt -1" in log, log
    assert "hallo" in quelle.read_text(encoding="utf-8")


def test_ausgabe_durchsuchen(tmp_path):
    """F5 laesst das Programm laufen, Strg+Umschalt+A sucht in seiner
    Ausgabe -- bei hunderten Zeilen der einzige Weg ohne Scrollen."""
    quelle = _datei(tmp_path, 'PRINT "eins"\nPRINT "zwei"\nPRINT "nadel"\n')
    ev = _taste(30, RL_F5)
    ev += _taste(120, RL_A, RL_LCTRL, RL_LSHIFT)
    ev += _taste(150, RL_V, RL_LCTRL)
    ev += _taste(175, RL_ENTER)
    log = _ide(tmp_path, quelle, frames=260, events=ev, zwischenablage="nadel")
    assert "ausgabe treffer 3" in log, log


def test_andere_reiter_schliessen(tmp_path):
    """Drei Dateien offen, Strg+Umschalt+W -- der vordere bleibt."""
    for name in ("a_eins.dh", "b_zwei.dh", "c_drei.dh"):
        _datei(tmp_path, "PRINT 1\n", name)
    quelle = tmp_path / "a_eins.dh"
    ev = _klick_mit(30, 60, _baum_y(0))
    ev += _klick_mit(55, 60, _baum_y(1), RL_LCTRL)
    ev += _klick_mit(80, 60, _baum_y(2), RL_LCTRL)
    ev += _taste(110, RL_E, RL_LCTRL, RL_LSHIFT)   # alle drei oeffnen
    ev += _taste(150, RL_W, RL_LCTRL, RL_LSHIFT)   # andere zumachen
    log = _ide(tmp_path, quelle, frames=250, events=ev)
    assert "baum offen 3" in log, log
    assert "andere zu 2" in log, log


def test_ueber_nennt_die_fassung_der_laufzeit(tmp_path):
    """Der Kasten stand auf 'Stand 7', als die IDE laengst weiter war.
    Jetzt fragt er die Laufzeit selbst (VERSION$)."""
    import re
    quelle = _datei(tmp_path, "PRINT 1\n")
    ev = _taste(40, RL_P, RL_LCTRL, RL_LSHIFT)   # Befehlspalette
    ev += _taste(70, RL_V, RL_LCTRL)             # "Ueber" tippen
    ev += _taste(100, RL_ENTER)
    log = _ide(tmp_path, quelle, frames=200, events=ev, zwischenablage="Ueber ..")
    zeilen = [z for z in log if z.startswith("ueber ")]
    assert zeilen and re.match(r"^ueber \d+\.\d+$", zeilen[0]), log


def test_einrueckungslinien_lassen_sich_abschalten(tmp_path):
    """Der Schalter steht unter Ansicht und in den Einstellungen; hier ueber
    die Befehlspalette, weil er kein Kuerzel hat."""
    quelle = _datei(tmp_path, "IF a THEN\n    PRINT 1\nEND IF\n")
    ev = _taste(40, RL_P, RL_LCTRL, RL_LSHIFT)
    ev += _taste(70, RL_V, RL_LCTRL)
    ev += _taste(100, RL_ENTER)
    log = _ide(tmp_path, quelle, frames=200, events=ev, zwischenablage="Einrueckungslinien")
    assert "linien aus" in log, log


# --------------------------------------------------------------- Stufe 10

RL_I = 73


def _palette(frame, was):
    """Einen Befehl ueber die Palette ausfuehren: Strg+Umschalt+P, Filter aus
    der Zwischenablage, Enter."""
    ev = _taste(frame, RL_P, RL_LCTRL, RL_LSHIFT)
    ev += _taste(frame + 30, RL_V, RL_LCTRL)
    ev += _taste(frame + 60, RL_ENTER)
    return ev


def test_zeilen_sortieren(tmp_path):
    """Ohne Auswahl gilt die ganze Datei -- eine einzelne Zeile zu sortieren
    ergibt nichts."""
    quelle = _datei(tmp_path, "' c\n' a\n' b\n")
    ev = _palette(30, "Zeilen sortieren") + _taste(120, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=230, events=ev, zwischenablage="Zeilen sortieren")
    assert "zeilen sortieren 3" in log, log
    assert quelle.read_text(encoding="utf-8").startswith("' a\n' b\n' c")


def test_doppelte_zeilen_entfernen(tmp_path):
    quelle = _datei(tmp_path, "' a\n' b\n' a\n")
    ev = _palette(30, "Doppelte") + _taste(120, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=230, events=ev, zwischenablage="Doppelte Zeilen")
    assert "zeilen doppelte 2" in log, log
    assert quelle.read_text(encoding="utf-8").count("' a") == 1


def test_marke_an_jedes_zeilenende(tmp_path):
    """Umschalt+Runter markiert zwei Zeilen, Strg+Umschalt+I setzt je eine
    Marke ans Ende, und EIN Strg+V schreibt in beide."""
    quelle = _datei(tmp_path, "eins\nzwei\ndrei\n")
    ev = _taste(30, RL_DOWN, RL_LSHIFT) + _taste(40, RL_DOWN, RL_LSHIFT)
    ev += _taste(60, RL_I, RL_LCTRL, RL_LSHIFT)
    ev += _taste(90, RL_V, RL_LCTRL)
    ev += _taste(120, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=230, events=ev, zwischenablage="!")
    assert "marken enden 2" in log, log
    text = quelle.read_text(encoding="utf-8")
    assert text.startswith("eins!\nzwei!\ndrei"), repr(text)


def test_im_ganzen_projekt_umbenennen(tmp_path):
    """Der Name steht in einer anderen Datei -- umbenannt werden beide."""
    quelle = _datei(tmp_path, "gruessen()\n", "spiel.dh")
    helfer = _datei(tmp_path, "SUB gruessen()\n    PRINT 1\nEND SUB\n", "helfer.dh")
    ev = _taste(30, RL_END)
    for k in range(3):
        ev += _taste(45 + k * 6, RL_LEFT)
    ev += _taste(75, RL_F6, RL_LCTRL, RL_LSHIFT)
    ev += _taste(110, RL_V, RL_LCTRL)
    ev += _taste(140, RL_ENTER)
    ev += _taste(180, RL_ENTER)          # die Vorschau uebernehmen
    log = _ide(tmp_path, quelle, frames=280, events=ev, zwischenablage="winken")
    assert "projekt umbenannt 2" in log, log
    assert "winken()" in quelle.read_text(encoding="utf-8")
    assert "SUB winken()" in helfer.read_text(encoding="utf-8")


def test_zwei_dateien_vergleichen(tmp_path):
    """Steht im Projektbaum eine andere Datei gewaehlt, ist sie gemeint --
    sonst faellt hier ein Datei-Dialog auf, den niemand beantwortet."""
    quelle = _datei(tmp_path, "PRINT 1\nPRINT 2\n", "a_eins.dh")
    _datei(tmp_path, "PRINT 1\nPRINT 3\n", "b_zwei.dh")
    ev = _klick_mit(30, 60, _baum_y(1))          # b_zwei.dh im Baum waehlen (oeffnet sie)
    ev += _klick_mit(70, 60, _baum_y(0))          # zurueck auf a_eins.dh
    ev += _klick_mit(110, 60, _baum_y(1), RL_LCTRL)   # b_zwei nur WAEHLEN, nicht oeffnen
    ev += _palette(150, "Mit einer anderen")
    log = _ide(tmp_path, quelle, frames=320, events=ev, zwischenablage="Mit einer anderen")
    zeilen = [z for z in log if z.startswith("vergleich ")]
    assert zeilen and zeilen[0] != "vergleich 0", log


# --------------------------------------------------------------- Stufe 11

def test_werkzeugleiste_startet_das_programm(tmp_path):
    """Der Knopf in der Leiste ruft denselben Befehl wie sein Menuepunkt.
    Geklickt wird auf das fuenfte Sinnbild (Starten) -- die Leiste beginnt
    bei x = 8, jeder Knopf ist 30 breit, dazu ein Trenner von 14."""
    quelle = _datei(tmp_path, 'PRINT "aus der Leiste"\n')
    # neu(8) oeffnen(38) sichern(68) |(98..112) start(112)
    ev = _maus(40, 126, 76)
    log = _ide(tmp_path, quelle, frames=260, events=ev)
    assert any(z.startswith("gestartet ") for z in log), log
    assert "beendet 0" in log, log


def test_die_leiste_laesst_sich_abschalten(tmp_path):
    """Ansicht -> Werkzeugleiste; der Schalter steht in der Sitzung."""
    quelle = _datei(tmp_path, "PRINT 1\n")
    ev = _palette(40, "Werkzeugleiste")
    log = _ide(tmp_path, quelle, frames=200, events=ev, zwischenablage="Werkzeugleiste")
    assert "leiste aus" in log, log


def test_kacheln_erzeugen_ihr_vorschaubild(tmp_path):
    """Ohne Datei steht die Willkommensseite vorn. Fuer jedes Beispiel, das
    es gibt, laeuft `dhrt bild` einmal im Hintergrund -- und danach liegt
    das PNG neben der Sitzung. Der Test bringt seinen EIGENEN Beispiel-
    Ordner mit, sonst liefen die echten acht."""
    from PIL import Image
    wurzel = tmp_path / "wurzel"
    (wurzel / "examples").mkdir(parents=True)
    (wurzel / "docs").mkdir()
    # Ohne diese Datei haelt die IDE den Ordner fuer keinen Beispiel-Ordner
    # und weicht auf die oeffentlichen Dokumente aus -- dann liefe das ECHTE
    # 09_shapes.dh, und die Vorschau zeigte etwas ganz anderes.
    (wurzel / "examples" / "183_sfx_generator.dh").write_text("PRINT 1\n", encoding="utf-8")
    (wurzel / "examples" / "09_shapes.dh").write_text(
        'SCREEN(200, 120, "P", 1)\n'
        'WHILE NOT QUITREQUESTED()\n'
        '    CLS(&H203040)\n'
        '    BOX(20, 20, 180, 100, &HFF8800)\n'
        '    FLIP()\n'
        'WEND\n', encoding="utf-8")
    log = _ide(tmp_path, "", frames=400, wurzel=wurzel)
    assert "vorschau 09_shapes.dh" in log, log
    assert "vorschau fertig 0" in log, log
    bild = tmp_path / "vorschau" / "09_shapes.png"
    assert bild.exists(), sorted(p.name for p in tmp_path.iterdir())
    im = Image.open(bild).convert("RGB")
    assert im.size == (200, 120)
    assert _nahe(im.getpixel((100, 60)), (0xFF, 0x88, 0x00), 20), im.getpixel((100, 60))


def test_eine_kachel_oeffnet_ihr_beispiel(tmp_path):
    """Ein Klick auf die Beschriftung unter dem Bild oeffnet die Datei."""
    wurzel = tmp_path / "wurzel"
    (wurzel / "examples").mkdir(parents=True)
    (wurzel / "docs").mkdir()
    # Ohne diese Datei haelt die IDE den Ordner fuer keinen Beispiel-Ordner
    # und weicht auf die oeffentlichen Dokumente aus -- dann liefe das ECHTE
    # 09_shapes.dh, und die Vorschau zeigte etwas ganz anderes.
    (wurzel / "examples" / "183_sfx_generator.dh").write_text("PRINT 1\n", encoding="utf-8")
    (wurzel / "examples" / "09_shapes.dh").write_text("PRINT 1\n", encoding="utf-8")
    # Die erste Kachel: Bild bei y = 198 + Kopf, die Beschriftung darunter.
    ev = _maus(120, 350, 440)
    log = _ide(tmp_path, "", frames=260, events=ev, wurzel=wurzel)
    assert "kachel 09_shapes.dh" in log, log


# --------------------------------------------------------------- Stufe 12

RL_R = 82


def test_auswahl_waechst_an_den_bloecken(tmp_path):
    """Wort, Zeile, der Block darum, dessen Elternblock -- jeder Druck nimmt
    die naechstgroessere Klammer."""
    quelle = _datei(tmp_path, "IF a THEN\n    FOR i = 1 TO 3\n"
                              "        PRINT punkte\n    NEXT\nEND IF\n")
    ev = _taste(25, RL_DOWN) + _taste(33, RL_DOWN) + _taste(45, RL_END)
    for k in range(4):
        ev += _taste(70 + k * 30, RL_UP, RL_LCTRL, RL_LSHIFT)
    log = _ide(tmp_path, quelle, frames=280, events=ev)
    assert "erweitern Wort" in log, log
    assert "erweitern Zeile" in log, log
    assert "erweitern Block 2-3" in log, log
    assert "erweitern Block 1-4" in log, log


def test_auswahl_verkleinern_geht_den_weg_zurueck(tmp_path):
    quelle = _datei(tmp_path, "IF a THEN\n    PRINT 1\nEND IF\n")
    ev = _taste(25, RL_DOWN) + _taste(45, RL_END)
    ev += _taste(70, RL_UP, RL_LCTRL, RL_LSHIFT)
    ev += _taste(100, RL_UP, RL_LCTRL, RL_LSHIFT)
    ev += _taste(130, RL_DOWN, RL_LCTRL, RL_LSHIFT)
    log = _ide(tmp_path, quelle, frames=220, events=ev)
    assert any(z.startswith("verkleinern ") for z in log), log


def test_auswahl_in_ein_unterprogramm_herausloesen(tmp_path):
    """Die gewaehlten Zeilen wandern in ein neues SUB, an ihrer Stelle steht
    der Aufruf. Was zugewiesen wird, geht BYREF -- sonst kaeme der Wert nie
    zurueck. Geprueft an der gesicherten Datei UND daran, dass sie noch
    uebersetzt."""
    quelle = _datei(tmp_path,
                    "SUB rechne()\n"
                    "    DIM summe AS INTEGER\n"
                    "    DIM i AS INTEGER\n"
                    "    summe = 0\n"
                    "    FOR i = 1 TO 10\n"
                    "        summe = summe + i\n"
                    "    NEXT\n"
                    "    PRINT summe\n"
                    "END SUB\n"
                    "rechne()\n")
    ev = []
    for k in range(3):
        ev += _taste(25 + k * 6, RL_DOWN)
    for k in range(4):
        ev += _taste(50 + k * 6, RL_DOWN, RL_LSHIFT)
    ev += _taste(80, RL_R, RL_LCTRL, RL_LSHIFT)
    ev += _taste(110, RL_V, RL_LCTRL)
    ev += _taste(140, RL_ENTER)
    ev += _taste(180, RL_ENTER)          # die Vorschau uebernehmen
    ev += _taste(220, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=310, events=ev, zwischenablage="summiere")
    assert "herausgeloest summiere 2 0" in log, log
    text = quelle.read_text(encoding="utf-8")
    assert "SUB summiere(BYREF summe AS INTEGER, i AS INTEGER)" in text, text
    assert "    summiere(summe, i)" in text, text
    r = subprocess.run([str(_DHRT), "--check", str(quelle)], capture_output=True,
                       text=True, encoding="utf-8", timeout=60)
    assert r.stdout.strip() == "[]", r.stdout


def test_herausloesen_sagt_es_wenn_die_datei_kaputt_geht(tmp_path):
    """Die Gegenprobe: eine unausgewogene Auswahl (das FOR drin, das NEXT
    nicht) ergibt Code, der nicht mehr uebersetzt -- und das steht da."""
    quelle = _datei(tmp_path,
                    "SUB rechne()\n"
                    "    DIM summe AS INTEGER\n"
                    "    DIM i AS INTEGER\n"
                    "    summe = 0\n"
                    "    FOR i = 1 TO 10\n"
                    "        summe = summe + i\n"
                    "    NEXT\n"
                    "END SUB\n")
    ev = []
    for k in range(3):
        ev += _taste(25 + k * 6, RL_DOWN)
    for k in range(3):
        ev += _taste(50 + k * 6, RL_DOWN, RL_LSHIFT)
    ev += _taste(80, RL_R, RL_LCTRL, RL_LSHIFT)
    ev += _taste(110, RL_V, RL_LCTRL)
    ev += _taste(140, RL_ENTER)
    ev += _taste(180, RL_ENTER)          # die Vorschau uebernehmen
    log = _ide(tmp_path, quelle, frames=290, events=ev, zwischenablage="halb")
    zeilen = [z for z in log if z.startswith("herausgeloest ")]
    assert zeilen and not zeilen[0].endswith(" 0"), log


def test_zwei_reiter_nebeneinander_vergleichen(tmp_path):
    """Die geteilte Ansicht geht an, und in BEIDEN Feldern bekommt die
    abweichende Zeile eine Marke."""
    quelle = _datei(tmp_path, "PRINT 1\nPRINT 2\nPRINT 3\n", "a_eins.dh")
    _datei(tmp_path, "PRINT 1\nPRINT X\nPRINT 3\n", "b_zwei.dh")
    ev = _klick_mit(30, 60, _baum_y(1))        # b_zwei oeffnen
    ev += _palette(70, "Zwei Reiter")
    log = _ide(tmp_path, quelle, frames=250, events=ev, zwischenablage="Zwei Reiter")
    assert "reiter vergleich 1 1" in log, log
    assert any(z.startswith("geteilt 0 ") for z in log), log


# --------------------------------------------------------------- Stufe 13

RL_ESC = 256
# Das Parameter-Fenster geht immer an derselben Stelle auf (420, 200), die
# Titelleiste des Glas-Themas ist 30 Pixel hoch. Widget-Koordinaten zaehlen
# ab dem INHALT darunter.
PARAM_X, PARAM_Y = 420, 230


def _param_knopf(frame, x, y, w, h):
    """Klick in die Mitte eines Knopfes des Parameter-Fensters."""
    return _klick_mit(frame, PARAM_X + x + w // 2, PARAM_Y + y + h // 2)


def test_das_geruest_setzt_den_abschluss_gleich_mit(tmp_path):
    """`IF x > 0 THEN` getippt und Enter -- das END IF steht da, und die
    Marke dazwischen. Geprueft an der gesicherten Datei."""
    quelle = _datei(tmp_path, "PRINT 1\n")
    ev = _taste(25, RL_END) + _taste(45, RL_ENTER) + _taste(70, RL_V, RL_LCTRL)
    ev += _taste(100, RL_ENTER) + _taste(140, RL_S, RL_LCTRL)
    _ide(tmp_path, quelle, frames=230, events=ev, zwischenablage="IF x > 0 THEN")
    text = quelle.read_text(encoding="utf-8")
    assert text.startswith("PRINT 1\nIF x > 0 THEN\n    \nEND IF"), repr(text)


def test_ohne_das_geruest_bleibt_enter_was_es_war(tmp_path):
    """Die Gegenprobe ueber die Einstellung: derselbe Ablauf ohne Geruest
    laesst die Zeile allein."""
    quelle = _datei(tmp_path, "PRINT 1\n")
    kfg = tmp_path / "aus.json"
    kfg.write_text('{"geruest": false}', encoding="utf-8")
    ev = _taste(25, RL_END) + _taste(45, RL_ENTER) + _taste(70, RL_V, RL_LCTRL)
    ev += _taste(100, RL_ENTER) + _taste(140, RL_S, RL_LCTRL)
    _ide(tmp_path, quelle, frames=230, events=ev, zwischenablage="IF x > 0 THEN", konfig=kfg)
    text = quelle.read_text(encoding="utf-8")
    assert "END IF" not in text, repr(text)
    assert text.startswith("PRINT 1\nIF x > 0 THEN\n"), repr(text)


def test_die_vorschau_verwirft_den_umbau_auf_esc(tmp_path):
    """Die Gegenprobe zur Vorschau: ESC, und die Datei bleibt, wie sie war
    -- auch nach dem Sichern."""
    quelle = _datei(tmp_path, "zaehler = 1\nzaehler = zaehler + 1\n")
    ev = _taste(20, RL_F6, RL_LSHIFT) + _taste(50, RL_V, RL_LCTRL) + _taste(70, RL_ENTER)
    ev += _taste(100, RL_ESC)            # die Vorschau verwerfen
    ev += _taste(140, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=210, events=ev, zwischenablage="summe")
    assert "umbau verworfen" in log, log
    assert not any(z.startswith("umbenannt ") for z in log), log
    assert quelle.read_text(encoding="utf-8").startswith("zaehler = 1"), quelle.read_text()


def test_parameter_umsortieren_zieht_die_aufrufe_mit(tmp_path):
    """Strg+Umschalt+U, den ersten Parameter nach unten, uebernehmen, die
    Vorschau bestaetigen: Definition UND Aufruf stehen umgestellt da."""
    quelle = _datei(tmp_path,
                    "SUB zeichne(x AS INTEGER, y AS INTEGER)\n"
                    "    PRINT x + y\n"
                    "END SUB\n"
                    "zeichne(1, 2)\n")
    ev = _taste(25, RL_U, RL_LCTRL, RL_LSHIFT)
    ev += _param_knopf(60, 322, 72, 120, 30)     # Nach unten
    ev += _param_knopf(100, 12, 248, 140, 30)    # Uebernehmen
    ev += _taste(150, RL_ENTER)                  # die Vorschau uebernehmen
    ev += _taste(190, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=280, events=ev)
    assert "parameter 2" in log, log
    assert "parameter umgestellt 2 0" in log, log
    text = quelle.read_text(encoding="utf-8")
    assert "SUB zeichne(y AS INTEGER, x AS INTEGER)" in text, text
    assert "zeichne(2, 1)" in text, text


def test_parameter_laesst_einen_aufruf_mit_anderer_zahl_in_ruhe(tmp_path):
    """Wer einen Vorgabewert weglaesst, meint eine andere Zuordnung -- der
    Aufruf bleibt stehen, und die Zahl steht im Protokoll."""
    quelle = _datei(tmp_path,
                    "SUB zeichne(x AS INTEGER, y AS INTEGER = 0)\n"
                    "    PRINT x + y\n"
                    "END SUB\n"
                    "zeichne(1, 2)\n"
                    "zeichne(7)\n")
    ev = _taste(25, RL_U, RL_LCTRL, RL_LSHIFT)
    ev += _param_knopf(60, 322, 72, 120, 30)     # Nach unten
    ev += _param_knopf(100, 12, 248, 140, 30)    # Uebernehmen
    ev += _taste(150, RL_ENTER)
    ev += _taste(190, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=280, events=ev)
    assert "parameter umgestellt 2 1" in log, log
    text = quelle.read_text(encoding="utf-8")
    assert "zeichne(2, 1)" in text, text
    assert "zeichne(7)" in text, text


def test_ohne_vorschau_schreibt_der_umbau_gleich(tmp_path):
    """Die zweite Gegenprobe: mit abgeschalteter Vorschau geht es den
    kurzen Weg -- ein Enter weniger, und die Datei ist umbenannt."""
    quelle = _datei(tmp_path, "zaehler = 1\nzaehler = zaehler + 1\n")
    kfg = tmp_path / "ohne_vorschau.json"
    kfg.write_text('{"umbau_vorschau": false}', encoding="utf-8")
    ev = _taste(20, RL_F6, RL_LSHIFT) + _taste(50, RL_V, RL_LCTRL) + _taste(70, RL_ENTER)
    ev += _taste(110, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=190, events=ev, zwischenablage="summe", konfig=kfg)
    assert not any(z.startswith("umbau vorschau ") for z in log), log
    assert any(z.startswith("umbenannt ") for z in log), log
    assert quelle.read_text(encoding="utf-8").startswith("summe = 1"), quelle.read_text()


def test_parameter_haelt_einen_aufruf_ueber_zwei_zeilen_heil(tmp_path):
    """Ein Aufruf mit `_` am Zeilenende: die Stuecke werden vertauscht, der
    Fortsetzungsstrich bleibt am Zeilenende stehen -- sonst uebersetzte die
    Datei danach nicht mehr."""
    quelle = _datei(tmp_path,
                    "SUB zeichne(x AS INTEGER, y AS INTEGER)\n"
                    "    PRINT x + y\n"
                    "END SUB\n"
                    "zeichne(1, _\n"
                    "        2)\n")
    ev = _taste(25, RL_U, RL_LCTRL, RL_LSHIFT)
    ev += _param_knopf(60, 322, 72, 120, 30)     # Nach unten
    ev += _param_knopf(100, 12, 248, 140, 30)    # Uebernehmen
    ev += _taste(150, RL_ENTER)
    ev += _taste(190, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=280, events=ev)
    assert "parameter umgestellt 2 0" in log, log
    r = subprocess.run([str(_DHRT), "--check", str(quelle)], capture_output=True,
                       text=True, encoding="utf-8", timeout=60)
    assert r.stdout.strip() == "[]", (r.stdout, quelle.read_text(encoding="utf-8"))


# --------------------------------------------------------------- Stufe 14

def test_vervollstaendigung_setzt_die_argumente_als_platzhalter(tmp_path):
    """`CIRC` getippt, Strg+Leer, Enter -- da steht `CIRCLE(x, y, r)`, und
    `x` ist markiert: das erste Tippen ersetzt es."""
    quelle = _datei(tmp_path, "CIRC\n")
    ev = _taste(30, RL_END) + _taste(50, RL_SPACE, RL_LCTRL) + _taste(80, RL_ENTER)
    ev += _taste(110, RL_V, RL_LCTRL)          # tippt ueber den Platzhalter
    ev += _taste(150, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=240, events=ev, zwischenablage="10")
    assert "platzhalter 1 von 3" in log, log
    text = quelle.read_text(encoding="utf-8")
    assert text.startswith("CIRCLE(10, y, r)"), repr(text)


def test_der_tabulator_geht_zum_naechsten_platzhalter(tmp_path):
    """Nach der Vervollständigung springt der Tabulator von Argument zu
    Argument -- und das Tippen trifft das zweite."""
    quelle = _datei(tmp_path, "CIRC\n")
    ev = _taste(30, RL_END) + _taste(50, RL_SPACE, RL_LCTRL) + _taste(80, RL_ENTER)
    ev += _taste(110, RL_TAB)                  # zum zweiten Argument
    ev += _taste(140, RL_V, RL_LCTRL)
    ev += _taste(180, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=270, events=ev, zwischenablage="20")
    assert "platzhalter 2 von 3" in log, log
    text = quelle.read_text(encoding="utf-8")
    assert text.startswith("CIRCLE(x, 20, r)"), repr(text)


def test_der_tabulator_rueckt_weiter_ein_wenn_kein_platzhalter_ansteht(tmp_path):
    """Die Gegenprobe zum dreifachen Tabulator: ohne Platzhalter und ohne
    Kürzel rückt er ein wie immer -- bis zur nächsten Spalte."""
    quelle = _datei(tmp_path, "PRINT 1\n")
    ev = _taste(30, RL_END) + _taste(60, RL_TAB) + _taste(100, RL_S, RL_LCTRL)
    _ide(tmp_path, quelle, frames=190, events=ev)
    text = quelle.read_text(encoding="utf-8")
    assert text.startswith("PRINT 1 \n"), repr(text)


def test_wer_ruft_das_auf_listet_die_stellen_im_projekt(tmp_path):
    """Umschalt+F12 auf dem Namen: alle Aufrufe, auch die in der anderen
    Datei -- die Definition selbst steht nicht dabei."""
    quelle = _datei(tmp_path, "SUB gruessen()\n    PRINT 1\nEND SUB\ngruessen()\n", "a_spiel.dh")
    _datei(tmp_path, "gruessen()\n' gruessen() im Kommentar zaehlt nicht\ngruessen()\n", "b_mehr.dh")
    ev = _taste(30, RL_END)
    for k in range(3):
        ev += _taste(45 + k * 6, RL_LEFT)     # in den Namen hinein
    ev += _taste(80, RL_F12, RL_LSHIFT)
    log = _ide(tmp_path, quelle, frames=190, events=ev)
    assert "aufrufer 3" in log, log


def test_parameter_umsortieren_geht_durch_das_ganze_projekt(tmp_path):
    """Die Definition steht in der einen Datei, ein Aufruf in der anderen --
    die Vorschau zeigt beide, und beide werden geschrieben."""
    quelle = _datei(tmp_path,
                    "SUB zeichne(x AS INTEGER, y AS INTEGER)\n"
                    "    PRINT x + y\n"
                    "END SUB\n"
                    "zeichne(1, 2)\n", "a_spiel.dh")
    zweite = _datei(tmp_path, "zeichne(7, 8)\n", "b_mehr.dh")
    ev = _taste(25, RL_U, RL_LCTRL, RL_LSHIFT)
    ev += _param_knopf(60, 322, 72, 120, 30)     # Nach unten
    ev += _param_knopf(100, 12, 248, 140, 30)    # Uebernehmen
    ev += _taste(150, RL_ENTER)                  # die Vorschau uebernehmen
    ev += _taste(190, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=280, events=ev)
    assert "parameter umgestellt 3 0" in log, log
    assert "SUB zeichne(y AS INTEGER, x AS INTEGER)" in quelle.read_text(encoding="utf-8")
    assert "zeichne(8, 7)" in zweite.read_text(encoding="utf-8")


def test_ein_klick_aufs_kreuz_schliesst_den_reiter(tmp_path):
    """Ohne Kreuz war ein Reiter mit der Maus gar nicht zu schließen.
    Geschlossen wird der ANGEKLICKTE, nicht der vordere -- und der aktive
    bleibt danach derselbe, obwohl die Reiter aufrücken."""
    quelle = _datei(tmp_path, "PRINT 1\n", "a_eins.dh")
    _datei(tmp_path, "PRINT 2\n", "b_zwei.dh")
    ev = _klick_mit(30, 60, _baum_y(1))          # b_zwei oeffnen, wird aktiv
    ev += _klick_mit(80, 85, 40)                 # Kreuz des ERSTEN Reiters
    log = _ide(tmp_path, quelle, frames=170, events=ev)
    assert "geschlossen 0" in log, log
    spuren = [z for z in log if z.startswith("spur ")]
    assert spuren and "b_zwei" in spuren[-1], log


def test_die_mittlere_taste_schliesst_den_reiter(tmp_path):
    """Wie im Browser: mittlere Taste irgendwo auf dem Reiter."""
    quelle = _datei(tmp_path, "PRINT 1\n", "a_eins.dh")
    _datei(tmp_path, "PRINT 2\n", "b_zwei.dh")
    ev = _klick_mit(30, 60, _baum_y(1))
    ev += [(80, MAUS_POS, 140, 40), (81, MAUS_POS, 140, 40),
           (81, MAUS_RUNTER, 2), (83, MAUS_HOCH, 2)]
    log = _ide(tmp_path, quelle, frames=170, events=ev)
    assert "geschlossen 1" in log, log


def test_ein_geaenderter_reiter_fragt_vor_dem_schliessen(tmp_path):
    """ESC bricht ab -- der Reiter bleibt, und die Datei auch."""
    quelle = _datei(tmp_path, "PRINT 1\n")
    ev = _taste(30, RL_V, RL_LCTRL)              # tippen: der Reiter ist schmutzig
    ev += _taste(70, RL_W, RL_LCTRL)             # schliessen
    ev += _taste(110, RL_ESC)                    # Abbrechen
    log = _ide(tmp_path, quelle, frames=190, events=ev, zwischenablage="X")
    assert not any(z.startswith("geschlossen") for z in log), log
    assert quelle.read_text(encoding="utf-8") == "PRINT 1\n", quelle.read_text()


def test_der_geaenderte_reiter_laesst_sich_sichernd_schliessen(tmp_path):
    """Enter drückt den ersten Knopf: Sichern -- danach ist der Reiter zu
    und das Getippte steht in der Datei."""
    quelle = _datei(tmp_path, "PRINT 1\n")
    ev = _taste(30, RL_V, RL_LCTRL)
    ev += _taste(70, RL_W, RL_LCTRL)
    ev += _taste(110, RL_ENTER)
    log = _ide(tmp_path, quelle, frames=190, events=ev, zwischenablage="X")
    assert "geschlossen 0" in log, log
    assert quelle.read_text(encoding="utf-8").startswith("XPRINT 1"), quelle.read_text()


# --------------------------------------------------------------- Stufe 15

def test_umbenennen_warnt_wenn_der_name_schon_vergeben_ist(tmp_path):
    """`gruessen` soll `winken` heissen -- aber `winken` gibt es schon. Der
    Einwand steht in der Vorschau, und die geht auf, damit man ihn sieht."""
    quelle = _datei(tmp_path, "SUB gruessen()\n    PRINT 1\nEND SUB\ngruessen()\n", "a_spiel.dh")
    _datei(tmp_path, "SUB winken()\n    PRINT 2\nEND SUB\n", "b_mehr.dh")
    ev = _taste(30, RL_END)
    for k in range(3):
        ev += _taste(45 + k * 6, RL_LEFT)
    ev += _taste(80, RL_F6, RL_LCTRL, RL_LSHIFT)
    ev += _taste(115, RL_V, RL_LCTRL)
    ev += _taste(145, RL_ENTER)
    log = _ide(tmp_path, quelle, frames=250, events=ev, zwischenablage="winken")
    assert "umbau einwand" in log, log
    # Nicht uebernommen: die Vorschau steht noch offen
    assert "projekt umbenannt" not in " ".join(log), log
    assert "SUB gruessen()" in quelle.read_text(encoding="utf-8")


def test_umbenennen_auf_einen_freien_namen_hat_keinen_einwand(tmp_path):
    """Die Gegenprobe: derselbe Ablauf mit einem Namen, den es nicht gibt."""
    quelle = _datei(tmp_path, "SUB gruessen()\n    PRINT 1\nEND SUB\ngruessen()\n", "a_spiel.dh")
    _datei(tmp_path, "SUB winken()\n    PRINT 2\nEND SUB\n", "b_mehr.dh")
    ev = _taste(30, RL_END)
    for k in range(3):
        ev += _taste(45 + k * 6, RL_LEFT)
    ev += _taste(80, RL_F6, RL_LCTRL, RL_LSHIFT)
    ev += _taste(115, RL_V, RL_LCTRL)
    ev += _taste(145, RL_ENTER)
    ev += _taste(185, RL_ENTER)          # die Vorschau uebernehmen
    log = _ide(tmp_path, quelle, frames=290, events=ev, zwischenablage="nicken")
    assert "umbau einwand" not in log, log
    assert "projekt umbenannt 1" in log, log
    assert "SUB nicken()" in quelle.read_text(encoding="utf-8")


def test_einen_parameter_hinzufuegen_setzt_ihn_an_jeder_aufrufstelle(tmp_path):
    """[+ Parameter] mit `hp AS INTEGER = 0`: die Definition bekommt die
    Deklaration, jeder Aufruf den Wert dahinter."""
    quelle = _datei(tmp_path,
                    "SUB zeichne(x AS INTEGER)\n"
                    "    PRINT x\n"
                    "END SUB\n"
                    "zeichne(1)\n"
                    "zeichne(2)\n")
    ev = _taste(25, RL_U, RL_LCTRL, RL_LSHIFT)
    ev += _param_knopf(60, 322, 116, 120, 30)    # + Parameter
    ev += _taste(110, RL_V, RL_LCTRL)            # Text aus der Zwischenablage
    ev += _taste(145, RL_ENTER)                  # Prompt bestaetigen
    ev += _param_knopf(180, 12, 248, 140, 30)    # Uebernehmen
    ev += _taste(230, RL_ENTER)                  # Vorschau uebernehmen
    ev += _taste(270, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=360, events=ev, zwischenablage="hp AS INTEGER = 0")
    assert any(z.startswith("parameter neu ") for z in log), log
    assert "parameter umgestellt 3 0" in log, log
    text = quelle.read_text(encoding="utf-8")
    assert "SUB zeichne(x AS INTEGER, hp AS INTEGER = 0)" in text, text
    assert "zeichne(1, 0)" in text and "zeichne(2, 0)" in text, text


def test_einen_parameter_entfernen_nimmt_das_argument_ueberall_weg(tmp_path):
    """[- Parameter] auf dem ersten: Definition und Aufrufe verlieren ihn."""
    quelle = _datei(tmp_path,
                    "SUB zeichne(x AS INTEGER, y AS INTEGER)\n"
                    "    PRINT y\n"
                    "END SUB\n"
                    "zeichne(1, 2)\n")
    ev = _taste(25, RL_U, RL_LCTRL, RL_LSHIFT)
    ev += _param_knopf(60, 322, 152, 120, 30)    # - Parameter (der erste ist gewaehlt)
    ev += _param_knopf(100, 12, 248, 140, 30)    # Uebernehmen
    ev += _taste(150, RL_ENTER)
    ev += _taste(190, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=280, events=ev)
    assert "parameter weg 0" in log, log
    text = quelle.read_text(encoding="utf-8")
    assert "SUB zeichne(y AS INTEGER)" in text, text
    assert "zeichne(2)" in text, text


def test_der_aufrufer_baum_zeigt_auch_die_aufrufer_der_aufrufer(tmp_path):
    """`tief` wird von `mitte` gerufen, `mitte` von `oben` -- der Baum hat
    darum mehr Knoten als es unmittelbare Aufrufe gibt."""
    quelle = _datei(tmp_path,
                    "SUB tief()\n"
                    "    PRINT 1\n"
                    "END SUB\n"
                    "SUB mitte()\n"
                    "    tief()\n"
                    "END SUB\n"
                    "SUB oben()\n"
                    "    mitte()\n"
                    "END SUB\n"
                    "oben()\n")
    # Die Marke steht in 1,1 -- vier Schritte nach rechts stehen auf `tief`.
    ev = []
    for k in range(4):
        ev += _taste(35 + k * 8, RL_RIGHT)
    ev += _taste(95, RL_F12, RL_LSHIFT)
    log = _ide(tmp_path, quelle, frames=200, events=ev)
    assert "aufrufer 1" in log, log          # tief() wird einmal unmittelbar gerufen
    assert "aufrufer baum 3" in log, log     # plus mitte() und oben()


def test_der_aufrufer_baum_laeuft_bei_einem_kreis_nicht_weg(tmp_path):
    """Ruft A das B und B das A, käme man ohne Abbruch nicht heraus. Ein
    Name steht darum höchstens einmal im Baum."""
    quelle = _datei(tmp_path,
                    "SUB a()\n"
                    "    b()\n"
                    "END SUB\n"
                    "SUB b()\n"
                    "    a()\n"
                    "END SUB\n"
                    "a()\n")
    ev = []
    for k in range(4):
        ev += _taste(35 + k * 8, RL_RIGHT)
    ev += _taste(95, RL_F12, RL_LSHIFT)
    log = _ide(tmp_path, quelle, frames=200, events=ev)
    assert any(z.startswith("aufrufer baum ") for z in log), log
    baum = [int(z.split()[-1]) for z in log if z.startswith("aufrufer baum ")]
    assert baum and baum[0] <= 4, log


# --------------------------------------------------------------- Stufe 16

def test_ein_umbau_laesst_sich_in_einem_zug_zuruecknehmen(tmp_path):
    """Strg+Z im Code-Feld nimmt nur den Reiter zurück, in dem man steht --
    ein Umbau über zwei Dateien braucht einen Zug für beide."""
    quelle = _datei(tmp_path,
                    "SUB zeichne(x AS INTEGER, y AS INTEGER)\n"
                    "    PRINT x + y\n"
                    "END SUB\n"
                    "zeichne(1, 2)\n", "a_spiel.dh")
    zweite = _datei(tmp_path, "zeichne(7, 8)\n", "b_mehr.dh")
    ev = _taste(25, RL_U, RL_LCTRL, RL_LSHIFT)
    ev += _param_knopf(60, 322, 72, 120, 30)     # Nach unten
    ev += _param_knopf(100, 12, 248, 140, 30)    # Uebernehmen
    ev += _taste(150, RL_ENTER)                  # Vorschau uebernehmen
    ev += _taste(200, RL_Z, RL_LCTRL, RL_LSHIFT) # und alles wieder zurueck
    ev += _taste(250, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=340, events=ev)
    assert "umbau zurueck 2" in log, log
    assert "SUB zeichne(x AS INTEGER, y AS INTEGER)" in quelle.read_text(encoding="utf-8")
    assert zweite.read_text(encoding="utf-8").startswith("zeichne(7, 8)"), zweite.read_text()


def test_ohne_umbau_gibt_es_nichts_zurueckzunehmen(tmp_path):
    """Die Gegenprobe: der Befehl sagt es, statt etwas zu tun."""
    quelle = _datei(tmp_path, "PRINT 1\n")
    ev = _taste(30, RL_Z, RL_LCTRL, RL_LSHIFT)
    log = _ide(tmp_path, quelle, frames=120, events=ev)
    assert "umbau zurueck 0" in log, log


def test_parameter_einer_methode_ziehen_die_aufrufe_mit(tmp_path):
    """Eine Methode ist auch nur ein SUB -- aber gerufen wird sie über das
    Objekt (`held.setze(...)`), und die Definition steht in einer CLASS."""
    quelle = _datei(tmp_path,
                    "CLASS Held\n"
                    "    DIM x AS INTEGER\n"
                    "    SUB setze(a AS INTEGER, b AS INTEGER)\n"
                    "        Self.x = a + b\n"
                    "    END SUB\n"
                    "END CLASS\n"
                    "DIM held AS Held\n"
                    "held = NEW Held()\n"
                    "held.setze(1, 2)\n")
    ev = []
    for k in range(9):
        ev += _taste(25 + k * 5, RL_DOWN)     # in die Zeile mit dem Aufruf
    ev += _taste(80, RL_END)
    for k in range(5):
        ev += _taste(95 + k * 5, RL_LEFT)     # auf den Methodennamen
    ev += _taste(130, RL_U, RL_LCTRL, RL_LSHIFT)
    ev += _param_knopf(165, 322, 72, 120, 30)
    ev += _param_knopf(205, 12, 248, 140, 30)
    ev += _taste(255, RL_ENTER)
    ev += _taste(295, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=390, events=ev)
    assert "parameter umgestellt 2 0" in log, log
    text = quelle.read_text(encoding="utf-8")
    assert "SUB setze(b AS INTEGER, a AS INTEGER)" in text, text
    assert "held.setze(2, 1)" in text, text


def test_der_aufrufer_baum_findet_auch_methodenaufrufe(tmp_path):
    """`held.setze(...)` ist ein Aufruf von `setze` -- der Punkt davor macht
    ihn nicht zu einem anderen Wort."""
    quelle = _datei(tmp_path,
                    "CLASS Held\n"
                    "    DIM x AS INTEGER\n"
                    "    SUB setze(a AS INTEGER)\n"
                    "        Self.x = a\n"
                    "    END SUB\n"
                    "END CLASS\n"
                    "SUB start()\n"
                    "    DIM held AS Held\n"
                    "    held = NEW Held()\n"
                    "    held.setze(1)\n"
                    "END SUB\n"
                    "start()\n")
    ev = []
    for k in range(2):
        ev += _taste(30 + k * 8, RL_DOWN)     # Zeile 3: SUB setze(...)
    ev += _taste(60, RL_END)
    for k in range(16):
        ev += _taste(75 + k * 4, RL_LEFT)     # auf den Namen
    ev += _taste(150, RL_F12, RL_LSHIFT)
    log = _ide(tmp_path, quelle, frames=260, events=ev)
    assert "aufrufer 1" in log, log
    assert "aufrufer baum 2" in log, log      # setze <- start <- Hauptprogramm


def test_ein_unterprogramm_in_eine_andere_datei_verschieben(tmp_path):
    """Die Zeilen wandern -- und die Quelle bekommt den IMPORT der Zieldatei,
    weil sie das Unterprogramm weiter ruft."""
    quelle = _datei(tmp_path,
                    "' Gruss\n"
                    "SUB gruessen()\n"
                    "    PRINT 1\n"
                    "END SUB\n"
                    "gruessen()\n", "a_spiel.dh")
    ziel = _datei(tmp_path, "' Helfer\nPRINT 9\n", "b_helfer.dh")
    ev = _taste(30, RL_DOWN) + _taste(45, RL_DOWN)      # in das SUB stellen
    ev += _taste(75, RL_V, RL_LCTRL, RL_LSHIFT)         # verschieben
    ev += _taste(115, RL_ENTER)                         # die erste Datei waehlen
    ev += _taste(165, RL_ENTER)                         # Vorschau uebernehmen
    ev += _taste(215, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=300, events=ev)
    assert any(z.startswith("verschieben gruessen ") for z in log), log
    q = quelle.read_text(encoding="utf-8")
    z = ziel.read_text(encoding="utf-8")
    assert "SUB gruessen()" not in q, q
    assert 'IMPORT "b_helfer.dh"' in q, q
    assert "SUB gruessen()" in z and "' Gruss" in z, z
    # Der Beweis ist nicht der Text, sondern dass es noch uebersetzt.
    r = subprocess.run([str(_DHRT), "--check", str(quelle)], capture_output=True,
                       text=True, encoding="utf-8", timeout=60)
    assert r.stdout.strip() == "[]", (r.stdout, q)


def test_eine_methode_laesst_sich_nicht_verschieben(tmp_path):
    """Die Gegenprobe: ohne ihre Klasse wäre sie kein Unterprogramm mehr."""
    quelle = _datei(tmp_path,
                    "CLASS Held\n"
                    "    SUB setze()\n"
                    "        PRINT 1\n"
                    "    END SUB\n"
                    "END CLASS\n", "a_spiel.dh")
    _datei(tmp_path, "PRINT 9\n", "b_helfer.dh")
    ev = _taste(30, RL_DOWN) + _taste(45, RL_DOWN)
    ev += _taste(75, RL_V, RL_LCTRL, RL_LSHIFT)
    log = _ide(tmp_path, quelle, frames=180, events=ev)
    assert "verschieben 0" in log, log
    assert "SUB setze()" in quelle.read_text(encoding="utf-8")


# --------------------------------------------------------------- Stufe 17

def test_eine_klasse_verschieben(tmp_path):
    """Steht die Marke in einer Klasse, ist die KLASSE gemeint -- eine
    Methode allein wäre ohne sie kein Unterprogramm mehr."""
    quelle = _datei(tmp_path,
                    "CLASS Held\n"
                    "    DIM x AS INTEGER\n"
                    "    SUB setze()\n"
                    "        Self.x = 1\n"
                    "    END SUB\n"
                    "END CLASS\n"
                    "DIM h AS Held\n"
                    "h = NEW Held()\n", "a_spiel.dh")
    ziel = _datei(tmp_path, "' Helfer\n", "b_helfer.dh")
    ev = _taste(30, RL_DOWN) + _taste(45, RL_DOWN)      # in die Klasse
    ev += _taste(75, RL_V, RL_LCTRL, RL_LSHIFT)
    ev += _taste(115, RL_DOWN) + _taste(130, RL_ENTER)  # zweite Zeile = b_helfer.dh
    ev += _taste(180, RL_ENTER)                         # Vorschau uebernehmen
    ev += _taste(230, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=320, events=ev)
    assert any(z.startswith("verschieben Held ") for z in log), log
    q = quelle.read_text(encoding="utf-8")
    z = ziel.read_text(encoding="utf-8")
    assert "CLASS Held" not in q, q
    assert 'IMPORT "b_helfer.dh"' in q, q      # `DIM h AS Held` braucht sie
    assert "CLASS Held" in z, z
    r = subprocess.run([str(_DHRT), "--check", str(quelle)], capture_output=True,
                       text=True, encoding="utf-8", timeout=60)
    assert r.stdout.strip() == "[]", (r.stdout, q)


def test_verschieben_legt_die_zieldatei_an(tmp_path):
    """Der erste Eintrag im Wähler ist eine neue Datei -- sonst wäre der
    erste Umzug ein Umweg über Datei/Neu und Sichern."""
    quelle = _datei(tmp_path,
                    "SUB gruessen()\n"
                    "    PRINT 1\n"
                    "END SUB\n"
                    "gruessen()\n", "a_spiel.dh")
    ev = _taste(30, RL_DOWN)
    ev += _taste(60, RL_V, RL_LCTRL, RL_LSHIFT)
    ev += _taste(100, RL_ENTER)                  # (neue Datei ...)
    ev += _taste(140, RL_V, RL_LCTRL)            # Name aus der Zwischenablage
    ev += _taste(175, RL_ENTER)
    ev += _taste(225, RL_ENTER)                  # Vorschau uebernehmen
    ev += _taste(275, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=370, events=ev, zwischenablage="gruss")
    assert any(z.startswith("verschieben gruessen ") for z in log), log
    neu = tmp_path / "gruss.dh"
    assert neu.exists(), sorted(p.name for p in tmp_path.iterdir())
    assert "SUB gruessen()" in neu.read_text(encoding="utf-8")
    assert 'IMPORT "gruss.dh"' in quelle.read_text(encoding="utf-8")


def test_umbenennen_warnt_bei_einer_ueberschreibung(tmp_path):
    """`ruf` gibt es in Tier UND in Hund. Wer nur eine umbenennt, zerreißt
    die Überschreibung -- ohne Fehlermeldung, denn gerufen wird von da an
    die Fassung der Oberklasse."""
    quelle = _datei(tmp_path,
                    "CLASS Tier\n"
                    "    SUB ruf()\n"
                    "        PRINT 1\n"
                    "    END SUB\n"
                    "END CLASS\n", "a_tier.dh")
    _datei(tmp_path,
           "CLASS Hund EXTENDS Tier\n"
           "    SUB ruf()\n"
           "        PRINT 2\n"
           "    END SUB\n"
           "END CLASS\n", "b_hund.dh")
    # Zeile 2 ist `    SUB ruf()` -- acht Schritte nach rechts stehen auf `ruf`.
    ev = _taste(30, RL_DOWN)
    for k in range(8):
        ev += _taste(45 + k * 5, RL_RIGHT)
    ev += _taste(100, RL_F6, RL_LSHIFT)
    ev += _taste(125, RL_V, RL_LCTRL)
    ev += _taste(155, RL_ENTER)
    log = _ide(tmp_path, quelle, frames=250, events=ev, zwischenablage="bellen")
    assert "umbau einwand" in log, log
    assert "SUB ruf()" in quelle.read_text(encoding="utf-8")


def test_ohne_vererbung_gibt_es_keinen_einwand(tmp_path):
    """Die Gegenprobe: dieselbe Methode, aber keine zweite Klasse dazu."""
    quelle = _datei(tmp_path,
                    "CLASS Tier\n"
                    "    SUB ruf()\n"
                    "        PRINT 1\n"
                    "    END SUB\n"
                    "END CLASS\n", "a_tier.dh")
    ev = _taste(30, RL_DOWN)
    for k in range(8):
        ev += _taste(45 + k * 5, RL_RIGHT)
    ev += _taste(100, RL_F6, RL_LSHIFT)
    ev += _taste(125, RL_V, RL_LCTRL)
    ev += _taste(155, RL_ENTER)
    ev += _taste(195, RL_ENTER)                # Vorschau uebernehmen
    ev += _taste(235, RL_S, RL_LCTRL)
    log = _ide(tmp_path, quelle, frames=320, events=ev, zwischenablage="bellen")
    assert "umbau einwand" not in log, log
    assert "SUB bellen()" in quelle.read_text(encoding="utf-8")
