"""Gemeinsame Pytest-Helpers fuer Drachenhauch-Tests."""
import io
import os
import sys
import contextlib
from pathlib import Path

import pytest


# Sicherstellen, dass das Projekt-Root im sys.path ist (fuer 'drachenhauch' Import).
_ROOT = Path(__file__).resolve().parent.parent
if str(_ROOT) not in sys.path:
    sys.path.insert(0, str(_ROOT))


# --------------------------------------------------------------------------
# Marker `qt`: alles, was PySide6 anfasst
# --------------------------------------------------------------------------
# Damit laesst sich der Qt-freie Kern allein fahren: `pytest tests/ -m "not qt"`
# ist in ~2 Minuten durch (2243 Tests) statt in ~7 -- praktisch, solange man an
# Sprache/Runtime/Werkzeugen arbeitet und die Editoren nicht anfasst.
#
# Ausserdem der Notausgang, falls der Qt-Teardown wieder einen ganzen Lauf
# zerlegt: genau dafuer lief CI kurzzeitig auf Python 3.11 mit `-m "not qt"`
# (dort starb der gemeinsame Qt-Lauf reproduzierbar mit "Windows fatal
# exception: code 0xc0000374"). CI faehrt inzwischen nur noch 3.12, wo das
# Problem nicht auftritt.
#
# Der Marker geht ueber den QUELLTEXT der Testdatei, nicht ueber den Dateinamen:
# 13 der 42 Qt-Dateien heissen gar nicht `*qt*` (test_fader.py,
# test_sfxeditor.py, test_tracker_editor_*.py, ...) -- eine Namensregel wuerde
# sie durchrutschen lassen und den Qt-freien Lauf wieder verunreinigen.
_QT_SOURCE_CACHE: dict[str, bool] = {}


def _module_uses_qt(path: str) -> bool:
    hit = _QT_SOURCE_CACHE.get(path)
    if hit is None:
        try:
            text = Path(path).read_text(encoding="utf-8", errors="replace")
            # "PySide6" allein reicht nicht: drei Dateien importieren Qt nur
            # MITTELBAR (`from drachenhauch.editor_qt.highlighter import ...`)
            # und nennen es selbst nie. Auf einem Runner ohne X11 sterben sie
            # trotzdem beim Import an libEGL -- gefunden vom Linux-Job, der
            # nach der ersten Runde noch drei von zwoelf Fehlern uebrig hatte.
            hit = ("PySide6" in text or "editor_qt" in text
                   or "_qt import" in text or "formdesigner" in text
                   or "spriteeditor" in text)
        except OSError:
            hit = False
        _QT_SOURCE_CACHE[path] = hit
    return hit


# --------------------------------------------------------------------------
# Marker `seriell`: vertraegt keine Nachbarn
# --------------------------------------------------------------------------
# Die Suite laeuft parallel (pytest-xdist) -- 10:40 seriell gegen 48 s auf 16
# Arbeitern, weil fast jeder Test einen dhrt-Prozess startet und auf ihn
# wartet, statt zu rechnen.
#
# Vier Dateien vertragen das nicht, und zwar nicht aus Zufall: sie haengen an
# etwas, das es auf der Maschine nur EINMAL gibt.
#
#   test_automation.py        raylibs AUTOMATION_* schreibt die ECHTE Eingabe
#                             mit und speist sie wieder ein. Jede fremde
#                             Mausbewegung landet in der Aufnahme -- auch die
#                             eines Menschen, der waehrenddessen am Rechner
#                             sitzt (seriell nachgestellt: fuenf Fehlschlaege
#                             hintereinander waehrend ich die Maus bewegte,
#                             danach im Leerlauf wieder gruen).
#   test_gui_form_runner.py   speist einen Klick ein -- dasselbe Nadeloehr.
#   test_audio_modulators.py  misst Pegel an der einen Soundkarte.
#   test_profiler.py          misst LAUFZEITEN; ringen 16 Prozesse um die
#                             Kerne, kippt die Rangfolge der heissen Zeile.
#
# Sie laufen deshalb in einem zweiten, seriellen Durchgang (`-m seriell`,
# 18 s). Empirisch ermittelt, nicht geraten: ueber acht parallele Laeufe
# fielen genau diese vier Dateien um, verteilt und jedes Mal andere Tests --
# das Streumuster ist das Erkennungszeichen fuer geteilte Betriebsmittel.
_SERIELL = {
    "test_automation.py",
    "test_gui_form_runner.py",
    "test_audio_modulators.py",
    "test_profiler.py",
}


# Kein Ton waehrend der Testsuite
# --------------------------------------------------------------------------
# Gemessen am 2026-08-22: laeuft eine sounddevice-Wiedergabe noch, waehrend ein
# Testprozess endet, stirbt er mit STATUS_HEAP_CORRUPTION (0xC0000374) --
# reproduzierbar in test_editor_qt_preset_bar.py (der SfxGenerator spielt beim
# Oeffnen einmal an, zwei Fenster plus ein weiteres Widget genuegen). Mit
# stillgelegtem `sd.play` 3 von 3 Laeufen sauber, mit Ton 3 von 3 kaputt; der
# Rueckgabewert des Prozesses faellt sonst NIEMANDEM auf, weil xdist ihn nicht
# prueft. Details in `drachenhauch/sfxeditor_qt.play`.
#
# Es geht dabei nichts an Zusicherung verloren: kein Test hoert hin. Was die
# Klangerzeugung selbst prueft (`synthesize`, `save_wav`, der Mixer), rechnet
# auf Arrays und braucht kein Geraet.
os.environ.setdefault("DH_OHNE_AUDIO", "1")


# Qt-Testdateien laufen NICHT im gemeinsamen Lauf mit
# --------------------------------------------------------------------------
# Sie lassen ihre Fenster stehen -- sie schliessen sie nie, und zerstoeren
# laesst sich das nicht (Begruendung weiter unten bei
# `_disarm_leftover_qt_widgets`). In EINEM pytest-Prozess sammeln sich diese
# Altlasten ueber ALLE Dateien an, die derselbe xdist-Arbeiter abbekommt.
#
# Gemessen am 2026-08-22, je Datei in einem frischen Prozess gezaehlt:
#
#     2284 uebrig gebliebene Top-Level-Fenster ueber alle 81 Qt-Dateien
#     davon 2078 (91 %) in 16 Dateien; Spitze: test_formdesigner_qt 644,
#     test_scoreeditor_qt 335, test_tracker_editor_instruments 232
#
# Jede Operation, die ueber ALLE Fenster des Prozesses laeuft, fasst diese
# Altlasten FREMDER Dateien an -- `app.processEvents()` genauso wie
# `QApplication.setStyleSheet()` (globales Repolish, das der Editor beim
# Theme-Wechsel in seinem Konstruktor ausloest). Genau dort starb der
# CI-Arbeiter sporadisch mit "Windows fatal exception: access violation", im
# Lauf 32588891599 sogar im ERSTEN processEvents() einer Datei -- da war noch
# nichts Eigenes im Prozess.
#
# Deshalb faehrt der gemeinsame Lauf `-m "not qt"`, und die Qt-Dateien laufen
# je in einem EIGENEN Prozess (`python tools/qt_tests_einzeln.py`, gemessen
# 21,7 s fuer alle 82 Dateien mit vier gleichzeitig). Einzeln ist jede von
# ihnen gruen -- was fehlte, war die Trennung.
#
# Gemessen, warum nicht anders:
#   * Fenster am Datei-Ende zerstoeren -> Absturz in `sendPostedEvents`
#     (DeferredDelete). Die Editor-Widgets haben echte Zerstoerungs-
#     Reihenfolge-Fehler; diese Datei entschaerft sie deshalb nur.
#   * Nur die 16 schwersten Dateien herausnehmen -> von 3 Abstuerzen in 10
#     Laeufen auf 1 in 10. Weniger Altlast hilft, aber ein Rest genuegt.
#   * `pytest --forked` -> gibt es auf Windows nicht.


# Marker `grafik`: braucht einen dhrt MIT raylib
# --------------------------------------------------------------------------
# `dhrt` laesst sich ohne Grafik bauen (`default = []`), und nur so kann CI auf
# Linux und macOS ueberhaupt TESTEN statt bloss `cargo check` zu fahren -- ein
# Grafik-Build braucht dort X11/GL und einen virtuellen Bildschirm.
#
# Gemessen mit dem grafikfreien Build: 2985 Tests laufen durch (2978 parallel
# + 7 seriell), 401 werden uebersprungen. Davon melden 188 einen fehlenden
# Grafik-Builtin -- die faengt `_ueberspringen_ohne_grafik` einzeln ab, damit
# gemischte Dateien nicht ganz herausfallen. Diese 22 Dateien scheitern
# anders: sie pruefen Pixel, Fenster, Audio-Pegel oder Eingabe und haben ohne
# Grafik keinen Gegenstand. Empirisch ermittelt, nicht geraten -- die drei
# seriellen darunter fehlten im ersten Anlauf, weil ich nur den parallelen
# Durchgang gemessen hatte.
#
# Uebersprungen wird NUR, wenn `DHRT_OHNE_GRAFIK=1` gesetzt ist. Ohne die
# Variable aendert sich nichts: der Windows-Lauf und jeder lokale Lauf sehen
# diese Tests wie bisher.
# Zwei Dateien pruefen Windows-Eigenheiten und haben anderswo keinen
# Gegenstand -- gefunden beim ersten Linux-Lauf:
#   test_formdesigner_document  normalisiert `forms\\a.dhform` zu `forms/a...`.
#                               Auf Linux ist `\\` kein Trenner, sondern ein
#                               gueltiges Zeichen im Dateinamen.
#   test_export_signierbar      buendelt eine .exe und fasst dabei Dateirechte
#                               an, die es unter POSIX so nicht gibt.
_NUR_WINDOWS = {
    "test_formdesigner_document.py",
    "test_export_signierbar.py",
}

_BRAUCHT_GRAFIK = {
    "test_arc_width.py",
    "test_audio_modulators.py",
    "test_automation.py",
    "test_beispiel_sqlite_tabelle.py",
    "test_buch_tippspiel.py",
    "test_circuitrunner.py",
    "test_examples.py",
    "test_gebundene_methoden_gui.py",
    "test_gfx_push_pop.py",
    "test_gui_form_runner.py",
    "test_gui_table_frozen_edge.py",
    "test_image_scale_nn.py",
    "test_image_text_extras.py",
    "test_input_edges.py",
    "test_input_polish.py",
    "test_kontaktbogen.py",
    "test_m3d.py",
    "test_modules_audio.py",
    "test_picking_geometry.py",
    "test_runtime_font_delta.py",
    "test_scissor.py",
    "test_shader_uniforms_geometry.py",
    "test_triangle_winding.py",
    "test_window_and_compress.py",
}


def _qt_laeuft() -> bool:
    """Laesst sich PySide6 hier ueberhaupt LADEN?

    Auf einem Linux-Runner ohne X11-Pakete nicht: `import PySide6.QtWidgets`
    stirbt an `libEGL.so.1`. Und zwar beim EINSAMMELN der Tests, also bevor
    ein Marker greifen koennte -- `-m "not qt"` filtert erst, nachdem das
    Modul importiert wurde. Der erste Linux-CI-Lauf endete deshalb mit 12
    Sammel-Fehlern, obwohl die Qt-Tests laengst abgewaehlt waren.

    Einmal probieren, Ergebnis merken.
    """
    global _QT_OK
    if _QT_OK is None:
        # Auf Windows ist Qt da -- der Editor IST eine Windows-Anwendung, und
        # alle Qt-Tests laufen dort. Den Import zu versuchen kostet in JEDEM
        # xdist-Arbeiter Zeit und bringt nichts; die Frage stellt sich nur
        # dort, wo Qt fehlen KANN.
        if os.name == "nt":
            _QT_OK = True
            return True
        try:
            import PySide6.QtWidgets  # noqa: F401
            _QT_OK = True
        except Exception:
            _QT_OK = False
    return _QT_OK


_QT_OK = None


def pytest_ignore_collect(collection_path, config):
    """Qt-Testdateien gar nicht erst einsammeln, wenn Qt nicht laedt.

    Greift VOR dem Import und damit frueh genug. Wo Qt laeuft (Windows, jeder
    Entwicklerrechner) aendert sich nichts.
    """
    if _module_uses_qt(str(collection_path)) and not _qt_laeuft():
        return True
    return None


def pytest_configure(config):
    config.addinivalue_line("markers", "qt: braucht PySide6 (automatisch gesetzt)")
    config.addinivalue_line(
        "markers", "seriell: braucht ein Betriebsmittel exklusiv (automatisch gesetzt)")
    config.addinivalue_line(
        "markers", "grafik: braucht einen dhrt mit raylib (automatisch gesetzt)")


def pytest_collection_modifyitems(items):
    for item in items:
        if _module_uses_qt(str(item.path)):
            item.add_marker(pytest.mark.qt)
        if item.path.name in _SERIELL:
            item.add_marker(pytest.mark.seriell)
        if item.path.name in _NUR_WINDOWS and os.name != "nt":
            item.add_marker(pytest.mark.skip(
                reason="prueft Windows-Eigenheiten (Pfad-Trenner, .exe-Buendel)"))
        if item.path.name in _BRAUCHT_GRAFIK:
            item.add_marker(pytest.mark.grafik)
            if os.environ.get("DHRT_OHNE_GRAFIK"):
                item.add_marker(pytest.mark.skip(
                    reason="braucht einen dhrt mit raylib (DHRT_OHNE_GRAFIK gesetzt)"))


def _find_dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    for variant in ("release", "debug"):
        p = _ROOT / "rust" / "drachenhauch_runtime" / "target" / variant / exe
        if p.exists():
            return p
    return None


_DHRT = _find_dhrt()


# --------------------------------------------------------------------------
# Maschinen ohne Bildschirm: Fenster-Tests ueberspringen statt scheitern
#
# Seit die CI `dhrt` selbst baut, laufen auch die Grafik-Tests dort -- und
# fielen mit 152 Fehlschlaegen um: der GitHub-Runner hat keine Grafikkarte,
# raylib bricht mit "Attempting to create window failed" ab (Exit 101).
#
# Eine Liste betroffener Dateien zu pflegen waere falsch: als Merkmal bot sich
# `SCREEN(` im Quelltext der Testdatei an, aber fuenf der 25 Dateien enthalten
# es gar nicht -- sie starten BEISPIELE von der Platte, die ihrerseits ein
# Fenster oeffnen (test_examples, test_circuitrunner, ...). Eine Heuristik auf
# dem Testtext kann das prinzipiell nicht sehen.
#
# Deshalb an der echten Signatur ansetzen, in zwei Stufen:
#   1. EINMAL pro Lauf pruefen, ob dhrt ueberhaupt ein Fenster oeffnen kann.
#   2. NUR wenn nicht: Fehlschlaege, die genau an der Fenstererzeugung
#      hingen, zu Skips machen.
# Auf einer Maschine mit Bildschirm ist Stufe 1 wahr und der Haken feuert nie
# -- ein echter Grafikfehler bleibt dort also ein Fehlschlag.
# Eine Meldung reicht nicht: dhrt scheitert auf einer Maschine ohne
# Bildschirm/Soundkarte je nach Stelle unterschiedlich -- mal als raylib-Panik,
# mal als GLFW-Warnung, beim Ton als Kira-Fehler. Der erste Anlauf kannte nur
# die Panik und liess deshalb 10 von 152 Faellen stehen.
_KEIN_FENSTER = (
    "Attempting to create window failed",          # raylib-Panik
    "does not appear to support OpenGL",           # GLFW/WGL auf dem CI-Runner
    "Failed to initialize Window",
    "Failed to initialize platform",
    "NoDefaultOutputDevice",                       # keine Soundkarte
)
_fenster_probe: "bool | None" = None


def _fenster_moeglich() -> bool:
    global _fenster_probe
    if _fenster_probe is None:
        _fenster_probe = False
        if _DHRT is not None:
            import subprocess
            import tempfile
            d = Path(tempfile.mkdtemp(prefix="dhrt_fensterprobe_"))
            p = d / "probe.dh"
            p.write_text('SCREEN(64, 48, "Probe", 1)\nFLIP()\n', encoding="utf-8")
            env = {**os.environ, "DHRT_FRAMES": "1"}
            try:
                r = subprocess.run([str(_DHRT), "run", str(p)], capture_output=True,
                                   text=True, timeout=60, env=env)
                text = (r.stderr or "") + (r.stdout or "")
                _fenster_probe = (r.returncode == 0
                                  and not any(s in text for s in _KEIN_FENSTER))
            except (OSError, subprocess.SubprocessError):
                _fenster_probe = False
    return _fenster_probe


@pytest.hookimpl(hookwrapper=True)
def pytest_runtest_makereport(item, call):
    bericht = (yield).get_result()
    if bericht.failed and any(s in str(bericht.longrepr or "") for s in _KEIN_FENSTER):
        if not _fenster_moeglich():
            bericht.outcome = "skipped"
            bericht.longrepr = (str(item.path), item.location[1],
                                "Kein Fenster moeglich (Maschine ohne Bildschirm) "
                                "-- raylib kann keins oeffnen")


def _dhrt_err_message(stderr: str) -> str:
    """Bare Fehlermeldung aus dhrt-stderr extrahieren (ohne 'Laufzeitfehler in
    label:line:'-Praefix), damit `pytest.raises(match=...)` die Meldung trifft."""
    s = (stderr or "").strip()
    import re
    m = re.search(r"(?:Laufzeitfehler in|Fehler in)\s+[^:]+:\d+:\s*(.*)", s, re.S)
    if m:
        return m.group(1).strip()
    # Compile/Parse: "label:line: <phase>-Fehler: MSG"
    m = re.search(r":\d+:\s*(?:[A-Za-z]+-Fehler[^:]*:\s*)?(.*)", s, re.S)
    return m.group(1).strip() if m else s


def _ueberspringen_ohne_grafik(stderr: str) -> None:
    """In einem Build OHNE Grafik einen Grafik-Test ueberspringen statt ihn
    fehlschlagen zu lassen.

    Wozu: `dhrt` laesst sich ohne raylib bauen (`default = []`), und nur so
    kann CI auf Linux und macOS UEBERHAUPT testen -- ein Grafik-Build braucht
    dort X11/GL und einen virtuellen Bildschirm. Gemessen laufen mit dem
    grafikfreien Build 2985 der 3386 Tests durch; die uebrigen haengen an
    SCREEN, GUI, Sprites, Audio oder Eingabe.

    NUR AUF ANFORDERUNG, per Umgebungsvariable `DHRT_OHNE_GRAFIK=1`. Ohne sie
    aendert sich nichts -- der Windows-Lauf und jeder lokale Lauf sollen einen
    fehlenden Grafik-Builtin weiterhin als Fehler sehen, sonst versteckt diese
    Bequemlichkeit eines Tages eine echte Luecke.
    """
    if not os.environ.get("DHRT_OHNE_GRAFIK"):
        return
    if "im Rust-Kern noch nicht verfuegbar" in stderr:
        pytest.skip("Build ohne Grafik: " + stderr.strip().splitlines()[-1][:120])


@pytest.fixture
def run_gb():
    """Fuehrt einen GB-Quelltext ueber die native Runtime (`dhrt run`) aus und
    gibt stdout zurueck (LF). Bei einem Fehler (Exit != 0) wird ein
    DHRuntimeError mit der dhrt-Meldung geworfen -- so funktioniert
    `pytest.raises(DHRuntimeError, match=...)` weiter (Wortlaut = dhrt).

    Beispiel:
        def test_print(run_gb):
            assert run_gb('PRINT "hi"') == "hi\\n"
    """
    import subprocess
    import tempfile
    from drachenhauch.errors import DHRuntimeError, ParseError, LexerError

    def _run(source: str, base: Path | None = None) -> str:
        if _DHRT is None:
            pytest.skip("native Runtime 'dhrt' nicht gebaut")
        # Temp-Datei im System-Temp-Verzeichnis (NICHT in examples/_ROOT, sonst
        # fangen die rust-Parity-Sweeps `examples.glob("*.dh")` die Datei).
        # dhrt-Module (IMPORT "vec2") sind verzeichnis-unabhaengig; relative
        # .dh-Datei-Imports sind in run_gb-Tests nicht in Gebrauch.
        # `base`: wenn gesetzt, die .dh DORT ablegen -- dhrt chdirt ins Datei-
        # Verzeichnis, also finden relative Pfade (TILED_LOAD, LOADIMAGE, ...)
        # Fixture-Dateien, die der Test in `base` abgelegt hat.
        if base is not None:
            fd, tmp = tempfile.mkstemp(suffix=".dh", prefix="_gbtest_", dir=str(base))
        else:
            fd, tmp = tempfile.mkstemp(suffix=".dh", prefix="_gbtest_")
        os.close(fd)
        try:
            Path(tmp).write_text(source, encoding="utf-8")
            # dhrt gibt UTF-8 aus -> explizit so dekodieren (sonst mis-decodet
            # Windows mit dem Locale-Codec cp1252 bei Nicht-ASCII-Ausgabe).
            r = subprocess.run([str(_DHRT), "run", tmp], capture_output=True,
                               text=True, encoding="utf-8", timeout=60)
        finally:
            try:
                os.unlink(tmp)
            except OSError:
                pass
        if r.returncode != 0:
            stderr = r.stderr or ""
            msg = _dhrt_err_message(stderr)
            _ueberspringen_ohne_grafik(stderr)
            # Phasen-passenden Fehlertyp werfen, damit pytest.raises(ParseError/
            # LexerError/...) wie bisher trifft. dhrt unterscheidet keine
            # TYP-Fehler von anderen Laufzeitfehlern -> die kommen als
            # DHRuntimeError (Tests dafuer pruefen die Basis DrachenhauchError).
            if "Parse-Fehler" in stderr:
                raise ParseError(msg)
            if "Lexer-Fehler" in stderr:
                raise LexerError(msg)
            raise DHRuntimeError(msg)
        return (r.stdout or "").replace("\r\n", "\n")

    return _run


@pytest.fixture
def dhrt_pfad():
    """Pfad zur gebauten nativen Runtime -- fuer Tests, die dhrt selbst
    aufrufen muessen (eigene Kommandozeile, Umleitung von stdout/stderr in
    dieselbe Datei, dhrt als Kindprozess von SHELL)."""
    if _DHRT is None:
        pytest.skip("native Runtime 'dhrt' nicht gebaut")
    return str(_DHRT)


@pytest.fixture
def run_gb_roh():
    """Wie `run_gb`, aber OHNE Fehler-Umsetzung: liefert
    `(returncode, stdout, stderr)` und akzeptiert Programm-Argumente.

    Fuer WP A (Betriebssystem-Anbindung) noetig, weil dort genau die Dinge
    geprueft werden, die `run_gb` wegabstrahiert: der Rueckgabewert (`EXIT`),
    die stderr-Ausgabe (`EPRINT`) und die Argumente hinter `--` (`ARGC`/`ARG$`).
    `run_gb` wuerde bei `EXIT(3)` eine DHRuntimeError werfen.

        def test_exit(run_gb_roh):
            code, out, err = run_gb_roh('EXIT(3)')
            assert code == 3
    """
    import subprocess
    import tempfile

    def _run(source: str, args: list[str] | None = None,
             eingabe: str | bytes | None = None):
        """`eingabe` geht als Standardeingabe hinein (fuer `STDIN()`).
        `bytes` schaltet auf den Binaermodus -- eine cp1252- oder rohe
        Byte-Eingabe laesst sich sonst gar nicht stellen."""
        if _DHRT is None:
            pytest.skip("native Runtime 'dhrt' nicht gebaut")
        fd, tmp = tempfile.mkstemp(suffix=".dh", prefix="_gbtest_")
        os.close(fd)
        binaer = isinstance(eingabe, bytes)
        try:
            Path(tmp).write_text(source, encoding="utf-8")
            cmd = [str(_DHRT), "run", tmp]
            if args:
                cmd.append("--")
                cmd.extend(args)
            if binaer:
                r = subprocess.run(cmd, capture_output=True, input=eingabe,
                                   timeout=60)
            else:
                r = subprocess.run(cmd, capture_output=True, text=True,
                                   encoding="utf-8", input=eingabe, timeout=60)
        finally:
            try:
                os.unlink(tmp)
            except OSError:
                pass

        def _txt(x):
            if x is None:
                return ""
            if isinstance(x, bytes):
                x = x.decode("utf-8", "replace")
            return x.replace("\r\n", "\n")

        return r.returncode, _txt(r.stdout), _txt(r.stderr)

    return _run


# Hinweis: `run_vm`, `run_native` und `run_all` sind **Aliase auf `run_gb`**,
# also auf `dhrt run`. Die Namen stammen aus der Zeit mit mehreren
# Ausfuehrungspfaden (Tree-Walker, Python-VM, Cython-VM); die sind alle
# entfernt, `dhrt` ist die einzige Runtime. Die Aliase bleiben, damit die ~550
# bestehenden Tests unveraendert weiterlaufen -- ein neuer Test nimmt besser
# gleich `run_gb`.

@pytest.fixture
def run_vm(run_gb):
    """Alias auf `run_gb` (dhrt). Der Name meinte frueher die Python-VM."""
    return run_gb


@pytest.fixture
def run_native(run_gb):
    """Alias auf `run_gb` (dhrt). Der Name meinte frueher die Cython-VM."""
    return run_gb


@pytest.fixture
def run_all(run_gb):
    """Alias auf `run_gb` (dhrt). Der Name meinte frueher die Bit-Identitaet
    ueber alle drei Pfade -- es gibt nur noch einen.

        def test_x(run_all):
            assert run_all('PRINT 1 + 2') == "3\n"
    """
    return run_gb



# Die fruehere `call_builtin`-Fixture (rief Python-Builtin-Impls direkt via
# `interpreter.BUILTINS`) ist entfernt -- alle Modul-Tests laufen jetzt als
# run_gb-Golden gegen dhrt (Stufe B, Phase 6/7-Teil2). Damit haengt kein Test
# mehr an interpreter.py/modules (Phase-8-Entblocker erledigt).


# --------------------------------------------------------------------------
# Qt-Altlasten nach JEDEM Test stilllegen
# --------------------------------------------------------------------------
# Die Qt-Testdateien bauen ihre Fenster als lokale Variable bzw. in einer
# function-scoped Fixture und schliessen sie nie -- das C++-Objekt lebt danach
# weiter (Qt haelt Top-Level-Widgets selbst am Leben, der Python-Refcount
# entscheidet nicht allein). In EINEM gemeinsamen pytest-Prozess summierte
# sich das auf ~16.000 lebende QObjects mit 244 faelligen QTimern, darunter
# 9 WIEDERHOLENDE 16-ms-Vorschau-Timer (Partikel-Editor, 60 FPS) und
# 8 wiederholende 30-s-Autosave-Timer aus geleakten `DrachenhauchEditor`n.
#
# Solange kein Test die Event-Loop pumpt, faellt das nicht auf. Der EINE Test,
# der es tut (`test_spriteeditor_qt_canvas.py`, "coalesced until event loop
# tick"), liess dann alle 244 ueberfaelligen Timer auf einmal feuern -- und
# weil die 16-ms-Timer sich schneller neu scharf machen, als die Queue
# abgearbeitet wird, kehrte `processEvents()` NIE zurueck (Haenger mit 100 %
# CPU, per py-spy exakt auf dieser Zeile stehend). Wo ein Slot dabei ein
# bereits geloeschtes C++-Objekt traf, gab es stattdessen die
# "Windows fatal exception: access violation".
#
# WARUM ENTSCHAERFEN STATT ZERSTOEREN: der naheliegende Weg waere
# `deleteLater()` auf jedes uebrige Top-Level-Widget. Genau das wurde
# ausprobiert -- und stuerzt beim tatsaechlichen Zerstoeren ab (Abbruch in
# `CodeEditor.event()`, waehrend `sendPostedEvents(DeferredDelete)` das
# Objekt unter dem laufenden Python-Frame zerlegt). Der Grund liegt NICHT im
# Test, sondern in echten Zerstoerungs-Reihenfolge-Fehlern der Editor-Widgets
# (dieselbe bekannte PySide6-Use-after-free-Serie).
#
# NACHGEMESSEN, damit das nicht wieder jemand "aufraeumt": einzeln laesst sich
# JEDES der sieben Editor-Fenster sauber abbauen (erzeugen, schliessen,
# deleteLater, DeferredDelete zustellen -- kein Absturz, kein Rest). Erst im
# echten Testlauf, mit den Altlasten vieler vorheriger Tests, kippt es: die
# Abbau-Variante dieser Fixture stuerzte in 1 von 5 Laeufen ab (immer derselbe
# Stack). Wer es erneut versuchen will, misst also bitte im vollen Lauf und
# mehrfach -- ein einzelner gruener Durchgang beweist hier gar nichts.
#
# Die Fixture nimmt den Leichen daher nur die Zuendschnur:
#
#   * jeder aktive QTimer wird gestoppt  -> keine faelligen/wiederholenden
#     Timer mehr, die Queue laeuft leer, `processEvents()` kehrt zurueck
#   * jeder QFileSystemWatcher verliert seine Pfade -> kein Watcher-Thread,
#     der im Hintergrund weiter Aenderungs-Events nachschiebt
#   * die Fenster werden versteckt -> keine Repaints
#   * die schon GEPOSTETEN Repaint- und Queued-Signal-Ereignisse werden
#     verworfen (siehe unten) -> ein `processEvents()` im naechsten Test
#     stellt nichts mehr an die Halbtoten zu
#
# Zum letzten Punkt, damit die Begruendung ehrlich bleibt: er war als Fix fuer
# den CI-Absturz auf Python 3.11 gedacht ("Windows fatal exception: code
# 0xc0000374", HEAP CORRUPTION, gemeldet beim `topLevelWidgets()`-Aufruf hier)
# -- die Vermutung war, dass ein `app.processEvents()` in einem spaeteren Test
# gepostete Ereignisse an halb abgeraeumte Objekte zustellt. **Das war falsch:**
# mit dieser Aenderung stuerzte CI an exakt derselben Stelle weiter ab. Der
# Verdacht liegt seitdem eher auf dieser Fixture selbst (sie laeuft nach JEDEM
# Test ueber alle uebrigen Fenster). CI faehrt auf 3.11 deshalb nur noch den
# Qt-freien Kern (`-m "not qt"`), siehe .github/workflows/ci.yml.
#
# Behalten wird der Schritt trotzdem, aber aus dem gemessenen Grund: der volle
# lokale Lauf wurde damit 22 % schneller (449 s statt 573 s) -- der Repaint-
# Nachschub der Altlasten war auch Rechenzeit.
#
# Die Objekte selbst bleiben am Leben (Speicher waechst weiter). Das ist
# unschoen, aber harmlos -- toedlich war ausschliesslich das Feuern.

def _disarm_leftover_qt_widgets() -> None:
    """Timer/Watcher aller uebrig gebliebenen Top-Level-Widgets stilllegen.

    STRUKTURELLE GRENZE, die man kennen muss: gesucht wird ab den Top-Level-
    FENSTERN nach unten (`findChildren`). Ein Zeitgeber, dessen Besitzer in
    keinem Fensterbaum haengt (elternloses QObject), ist auf diesem Weg
    unsichtbar und bleibt scharf -- er feuert dann im naechsten `processEvents()`
    einer FREMDEN Testdatei. Genau daran hing der sporadische CI-Absturz
    (2026-08-22, `SnapshotUndo` war elternlos, 2111 Sichtungen ueber acht
    Editor-Testdateien).

    Die Gegenseite ist deshalb Besitzverhaeltnis statt Suchtiefe: wer ein
    QObject mit Zeitgeber anlegt, gibt ihm sein Fenster als Elternteil mit.
    Abgesichert in `tests/test_qt_altlasten_zeitgeber.py`.
    """
    qtwidgets = sys.modules.get("PySide6.QtWidgets")
    if qtwidgets is None:
        return                      # Nicht-Qt-Test: nichts zu tun, nichts zu zahlen
    app = qtwidgets.QApplication.instance()
    if app is None:
        return

    import shiboken6
    from PySide6.QtCore import QFileSystemWatcher, QTimer

    for w in list(app.topLevelWidgets()):
        # Ein zerstoertes Elternteil nimmt seine Kinder mit -- die stehen
        # dann noch in der Liste, sind aber schon ungueltig.
        if not shiboken6.isValid(w):
            continue
        try:
            for t in w.findChildren(QTimer):
                if shiboken6.isValid(t) and t.isActive():
                    t.stop()
            for fsw in w.findChildren(QFileSystemWatcher):
                if not shiboken6.isValid(fsw):
                    continue
                paths = fsw.directories() + fsw.files()
                if paths:
                    fsw.removePaths(paths)
            if not w.isHidden():
                w.hide()
        except RuntimeError:
            # Waehrend des Durchlaufs weggeraeumtes C++-Objekt -- egal,
            # tot ist auch entschaerft.
            continue


def _drop_pending_events() -> None:
    """Gepostete Repaint- und Queued-Signal-Ereignisse der Altlasten wegwerfen.

    Nur diese beiden Arten, NICHT alles: ein pauschales
    `removePostedEvents(None)` wuerde auch `DeferredDelete` mitnehmen und damit
    Objekte, die ein Test ordentlich per `deleteLater()` abgemeldet hat, fuer
    immer am Leben lassen (davor warnt die Qt-Doku ausdruecklich).

    * `UpdateRequest` -- der Repaint-Nachschub versteckter Fenster
    * `MetaCall`      -- Slot-Aufrufe aus queued Signal-Verbindungen; genau die
                         treffen sonst Objekte, deren C++-Seite schon weg ist
    """
    qtcore = sys.modules.get("PySide6.QtCore")
    if qtcore is None:
        return
    ev = qtcore.QEvent.Type
    for kind in (ev.UpdateRequest, ev.MetaCall):
        qtcore.QCoreApplication.removePostedEvents(None, kind)


@pytest.fixture(autouse=True)
def _qt_widget_cleanup():
    """Legt nach jedem Test die zurueckgelassenen Qt-Fenster still.

    Autouse + function-scoped: laeuft damit VOR den modul-eigenen Fixtures
    im Setup und folglich NACH deren Teardown -- eine `win`-Fixture darf ihr
    Fenster also noch ganz normal selbst abbauen.
    """
    yield
    _disarm_leftover_qt_widgets()
    _drop_pending_events()


def quiesce_qt() -> int:
    """Den GANZEN Prozess ruhigstellen: jeden aktiven QTimer stoppen und alle
    schon zugestellten Events verwerfen. Gibt die Zahl gestoppter Timer zurueck.

    Warum zusaetzlich zu `_disarm_leftover_qt_widgets()`: das laeuft ueber
    `topLevelWidget.findChildren()` und erwischt damit nur Timer, die im
    Objektbaum eines Fensters haengen. `SnapshotUndo` (Undo-Debounce der
    Editoren) ist aber ein parentloses QObject -- nach einem gemeinsamen Lauf
    blieben so noch ~39 scharfe Timer uebrig, deren `changed`-Signal ueber
    `_mark_dirty()` wieder ein `mark()` ausloest. Diese Rueckkopplung reicht,
    damit die Event-Queue nie leer wird.

    TEUER (einmaliger Heap-Scan) -- nur direkt vor einer Stelle aufrufen, die
    wirklich die Event-Loop pumpt, nicht pro Test.
    """
    qtcore = sys.modules.get("PySide6.QtCore")
    if qtcore is None:
        return 0

    import gc
    import shiboken6

    stopped = 0
    for obj in gc.get_objects():
        if not isinstance(obj, qtcore.QTimer):
            continue
        try:
            if shiboken6.isValid(obj) and obj.isActive():
                obj.stop()
                stopped += 1
        except RuntimeError:
            pass                    # waehrenddessen weggeraeumt -- auch gut
    # Bereits gepostete Events (Repaints, queued Signals) der Altlasten
    # wegwerfen, damit der folgende Pump nur noch das sieht, was der Test
    # selbst erzeugt.
    qtcore.QCoreApplication.removePostedEvents(None)
    return stopped


# Hartes Prozessende fuer die Einzel-Laeufe
# --------------------------------------------------------------------------
# `tools/qt_tests_einzeln.py` startet je Datei einen eigenen Prozess und prueft
# dessen Rueckgabewert -- zum ersten Mal ueberhaupt, denn xdist sieht ihn nie
# an. Dabei kam heraus: manche Qt-Dateien beenden sich mit
# STATUS_HEAP_CORRUPTION (0xC0000374), NACHDEM alle Tests gruen durch sind.
# Gemessen am 2026-08-22 an test_sfxeditor.py: 4 von 4 Laeufen, auch mit
# stillgelegtem sounddevice -- es ist Qts Abbau der 14 stehen gelassenen
# Editor-Fenster beim Prozessende, dieselbe Zerstoerungs-Reihenfolge-Schwaeche,
# die weiter unten bei `_disarm_leftover_qt_widgets` beschrieben ist. EIN
# Fenster allein wird sauber abgebaut (verifiziert), erst der Stapel kippt.
#
# `DH_TEST_HARTES_ENDE=1` beendet den Prozess deshalb per `os._exit()`, sobald
# das Ergebnis feststeht: nach dem letzten Test, nach der Zusammenfassung. Was
# uebersprungen wird, ist ausschliesslich der Abbau eines Prozesses, der
# ohnehin endet -- kein Test, keine Zusicherung, keine Ausgabe.
#
# EHRLICH DAZU: das behebt den Abbau-Fehler nicht, es haelt ihn aus dem Weg.
# Der Fehler trifft die Anwendung nicht in derselben Form -- dort schliesst ein
# Mensch ein Fenster nach dem anderen, und das ist nachweislich sauber.
_ENDE_STATUS = 0


def pytest_sessionfinish(session, exitstatus):
    global _ENDE_STATUS
    _ENDE_STATUS = int(exitstatus)


def pytest_unconfigure(config):
    # NICHT in `pytest_sessionfinish`: der Terminal-Reporter schreibt seine
    # Zusammenfassung ("5 passed in 1.16s") dort als Hook-Wrapper, also NACH
    # allen gewoehnlichen Implementierungen -- ein `os._exit` von dort
    # verschluckt sie (ausprobiert). `pytest_unconfigure` laeuft ganz zum
    # Schluss, wenn alles geschrieben ist.
    if not os.environ.get("DH_TEST_HARTES_ENDE"):
        return
    if os.environ.get("PYTEST_XDIST_WORKER"):
        return                      # ein Arbeiter muss sein Ergebnis noch melden
    sys.stdout.flush()
    sys.stderr.flush()
    os._exit(_ENDE_STATUS)


@pytest.fixture
def qt_altlasten_entschaerfen():
    """Gibt die Entschaerf-Routine der autouse-Fixture zum direkten Aufruf.

    Fuer `test_qt_altlasten_zeitgeber.py`, das genau diese Routine prueft:
    sieht sie den Entprell-Zeitgeber eines Editors, oder laeuft er danach
    weiter und feuert in den naechsten Test hinein?
    """
    return _disarm_leftover_qt_widgets


@pytest.fixture
def quiet_qt_process():
    """Fixture-Form von `quiesce_qt()` -- fuer Tests, die selbst die
    Event-Loop pumpen muessen und deshalb einen ruhigen Prozess brauchen."""
    quiesce_qt()
    yield
