"""Modul `midi` -- Noten von einem Instrument lesen und welche hinausschicken.

Drachenhauch bringt Tracker, Sampler, sfxr-Synth, Notenblatt-Editor und
Kira-Busse mit; bis hierher konnte kein angeschlossenes Keyboard etwas davon
ansteuern.

Was hier wie geprueft wird:

* **Die Umrechner** (`MIDI_NOTE_NAME$`, `MIDI_NOTE_FREQ`) brauchen kein Geraet
  und stehen darum in JEDEM Bau -- sie werden ganz normal per `run_gb`
  geprueft, auch in der posix-CI ohne das `midi`-Feature.
* **Die Geraete-Befehle** haengen am Feature. Der Bau sagt selbst, ob er sie
  hat (`dhrt --version`); je nachdem wird die Auflistung geprueft ODER die
  klare Meldung, dass der Bau sie nicht enthaelt.
* **Der ganze Kreis** -- senden, durch das Betriebssystem, wieder empfangen --
  wird geprueft, sobald ein VIRTUELLER Loopback-Port da ist (unter Windows
  z.B. loopMIDI). Ein solcher Port erscheint unter demselben Namen als Ein-
  UND Ausgang; genau daran erkennen die Tests ihn. Ohne ihn ueberspringen
  sie sich. Ein echtes Keyboard braucht es dafuer nicht.
"""
import os
import subprocess
from pathlib import Path

import pytest

from drachenhauch.errors import DHRuntimeError

_ROOT = Path(__file__).resolve().parent.parent


def _dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    return next((_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
                 for v in ("release", "debug")
                 if (_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe).exists()), None)


_DHRT = _dhrt()


def _hat_midi() -> bool:
    """Ist das `midi`-Feature in DIESEM Bau drin? Der Bau sagt es selbst.

    NUR die `dabei:`-Zeile lesen. `dhrt --version` nennt darunter auch eine
    `fehlt:`-Zeile mit denselben Namen -- ein blosses `"midi" in stdout` ist
    deshalb IMMER wahr und hat genau diesen Test in der Windows-CI
    fehlschlagen lassen (dort wird ohne --hardware gebaut).
    """
    if _DHRT is None:
        return False
    r = subprocess.run([str(_DHRT), "--version"], capture_output=True, text=True,
                       encoding="utf-8", timeout=30)
    for zeile in (r.stdout or "").splitlines():
        if zeile.startswith("dabei:"):
            return "midi" in [t.strip() for t in zeile[len("dabei:"):].split(",")]
    return False


# ------------------------------------------------------- Verdrahtung

def test_midi_ist_ein_bekanntes_modul():
    from drachenhauch.modules import is_known_module
    assert is_known_module("midi")


def test_builtins_sind_registriert():
    from drachenhauch.editor_qt.dhrt_meta import builtin_names_lower
    erwartet = {
        "midi_in_count", "midi_out_count", "midi_in_name$", "midi_out_name$",
        "midi_in_open", "midi_out_open", "midi_in_close", "midi_out_close",
        "midi_next", "midi_pending", "midi_status", "midi_channel",
        "midi_data1", "midi_data2", "midi_is_note_on", "midi_is_note_off",
        "midi_is_cc", "midi_note", "midi_velocity", "midi_cc_number",
        "midi_cc_value", "midi_note_on", "midi_note_off", "midi_cc",
        "midi_send", "midi_note_name$", "midi_note_freq",
    }
    assert erwartet <= builtin_names_lower()


def test_handle_typen_lassen_sich_deklarieren(run_gb):
    out = run_gb("""
IMPORT "midi"
DIM ein AS MIDI_IN
DIM aus AS MIDI_OUT
PRINT "ok"
""")
    assert out == "ok\n"


# ------------------------------------------- Umrechner (ohne Geraet)

def test_notenname(run_gb):
    out = run_gb("""
IMPORT "midi"
PRINT MIDI_NOTE_NAME$(60); " "; MIDI_NOTE_NAME$(69); " "; MIDI_NOTE_NAME$(71)
""")
    # 71 ist im deutschen Sprachraum H, nicht B.
    assert out == "C4 A4 H4\n"


def test_notenname_ausserhalb_des_protokolls_ist_leer(run_gb):
    """MIDI kennt 0..127. Ausserhalb lieber nichts sagen als raten."""
    out = run_gb('IMPORT "midi"\nPRINT "["; MIDI_NOTE_NAME$(200); "]"\n')
    assert out == "[]\n"


def test_notenfrequenz(run_gb):
    out = run_gb("""
IMPORT "midi"
PRINT MIDI_NOTE_FREQ(69)
PRINT MIDI_NOTE_FREQ(81)
""")
    assert out == "440.0\n880.0\n"


# --------------------------------------------------- Geraete-Befehle

@pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")
def test_auflistung_oder_klare_meldung(run_gb):
    """Mit dem Feature muss die Auflistung antworten, ohne es eine klare
    Meldung kommen -- lautlos ins Leere laufen darf keiner der beiden."""
    quelle = 'IMPORT "midi"\nPRINT MIDI_OUT_COUNT() >= 0\n'
    if _hat_midi():
        assert run_gb(quelle) == "TRUE\n"
    else:
        with pytest.raises(DHRuntimeError, match="MIDI_OUT_COUNT"):
            run_gb(quelle)


@pytest.mark.skipif(not _hat_midi(), reason="Bau ohne das midi-Feature")
def test_unbekannter_anschluss_wird_benannt(run_gb):
    with pytest.raises(DHRuntimeError, match="es gibt keinen Eingang 999"):
        run_gb('IMPORT "midi"\nDIM h AS MIDI_IN\nh = MIDI_IN_OPEN(999)\n')


@pytest.mark.skipif(not _hat_midi(), reason="Bau ohne das midi-Feature")
def test_wertebereiche_beim_senden(run_gb):
    """Ein Wert ueber 127 setzte im Protokoll das Statusbit und wuerde als
    voellig andere Nachricht gelesen -- das muss vorher auffallen.

    Geprueft ohne offenen Ausgang: die Bereichspruefung kommt VOR dem
    Zugriff auf das Geraet, sonst haette sie auf einer Maschine ohne
    Anschluss keine Wirkung.
    """
    for quelle, muster in [
        ('MIDI_NOTE_ON(0, 0, 60, 100)', "Kanal muss zwischen 1 und 16"),
        ('MIDI_NOTE_ON(0, 17, 60, 100)', "Kanal muss zwischen 1 und 16"),
        ('MIDI_NOTE_ON(0, 1, 128, 100)', "Note muss zwischen 0 und 127"),
        ('MIDI_NOTE_ON(0, 1, 60, -1)', "Anschlag muss zwischen 0 und 127"),
        ('MIDI_CC(0, 1, 200, 0)', "Reglernummer muss zwischen 0 und 127"),
        ('MIDI_SEND(0, 64, 0, 0)', "Statusbyte muss zwischen 128 und 255"),
    ]:
        with pytest.raises(DHRuntimeError, match=muster):
            run_gb('IMPORT "midi"\n' + quelle + "\n")


@pytest.mark.skipif(not _hat_midi(), reason="Bau ohne das midi-Feature")
def test_geschlossenes_handle_wird_gemeldet(run_gb):
    """Nach MIDI_OUT_CLOSE darf ein Senden nicht mehr durchgehen."""
    if not _hat_ausgang():
        pytest.skip("kein MIDI-Ausgang an dieser Maschine")
    with pytest.raises(DHRuntimeError, match="geschlossenes MIDI_OUT"):
        run_gb("""
IMPORT "midi"
DIM s AS MIDI_OUT
s = MIDI_OUT_OPEN(0)
MIDI_OUT_CLOSE(s)
MIDI_NOTE_ON(s, 1, 60, 100)
""")


def _hat_ausgang() -> bool:
    """Gibt es an dieser Maschine ueberhaupt einen MIDI-Ausgang?

    Unter Windows liefert der eingebaute GS-Wavetable-Synth einen; auf einem
    nackten Rechner kann die Liste leer sein.
    """
    if not _hat_midi() or _DHRT is None:
        return False
    import tempfile
    with tempfile.TemporaryDirectory() as d:
        p = Path(d) / "n.dh"
        p.write_text('IMPORT "midi"\nPRINT MIDI_OUT_COUNT()\n', encoding="utf-8")
        r = subprocess.run([str(_DHRT), "run", str(p)], capture_output=True,
                           text=True, encoding="utf-8", timeout=30)
        erste = (r.stdout or "0").strip().splitlines()
        return bool(erste) and erste[-1].isdigit() and int(erste[-1]) > 0


@pytest.mark.skipif(not _hat_midi(), reason="Bau ohne das midi-Feature")
def test_noten_wirklich_hinausschicken(run_gb):
    """Der einzige Weg, den Sendepfad ohne fremdes Geraet zu belegen:
    Windows bringt einen MIDI-Ausgang mit (GS Wavetable Synth). Hier laeuft
    also wirklich ein Dreiklang zum Synthesizer und wieder aus."""
    if not _hat_ausgang():
        pytest.skip("kein MIDI-Ausgang an dieser Maschine")
    out = run_gb("""
IMPORT "midi"
DIM s AS MIDI_OUT
s = MIDI_OUT_OPEN(0)
DIM i AS INTEGER
FOR i = 0 TO 2
    MIDI_NOTE_ON(s, 1, 60 + i * 4, 100)
NEXT
FOR i = 0 TO 2
    MIDI_NOTE_OFF(s, 1, 60 + i * 4)
NEXT
MIDI_CC(s, 1, 7, 90)
MIDI_OUT_CLOSE(s)
PRINT "durch"
""")
    assert out == "durch\n"


# ------------------------------------------- der ganze Kreis (Loopback)

def _loopback() -> str:
    """Name eines Ports, der Ein- UND Ausgang ist -- also eine Schleife.

    Ein virtueller Loopback (loopMIDI o.ae.) erscheint unter demselben Namen
    auf beiden Seiten; ein echtes Geraet nie. Deshalb ist der Namensvergleich
    hier kein Behelf, sondern das eigentliche Erkennungsmerkmal.
    """
    if not _hat_midi() or _DHRT is None:
        return ""
    import tempfile
    quelle = (
        'IMPORT "midi"\n'
        'DIM i AS INTEGER\n'
        'FOR i = 0 TO MIDI_IN_COUNT() - 1\n'
        '    DIM j AS INTEGER\n'
        '    FOR j = 0 TO MIDI_OUT_COUNT() - 1\n'
        '        IF MIDI_IN_NAME$(i) = MIDI_OUT_NAME$(j) THEN PRINT MIDI_IN_NAME$(i)\n'
        '    NEXT\n'
        'NEXT\n')
    with tempfile.TemporaryDirectory() as d:
        p = Path(d) / "lb.dh"
        p.write_text(quelle, encoding="utf-8")
        r = subprocess.run([str(_DHRT), "run", str(p)], capture_output=True,
                           text=True, encoding="utf-8", timeout=30)
        zeilen = [z for z in (r.stdout or "").splitlines() if z.strip()]
        return zeilen[0].strip() if zeilen else ""


_LOOPBACK = _loopback()
_ohne_loopback = pytest.mark.skipif(
    not _LOOPBACK, reason="kein virtueller MIDI-Loopback-Port vorhanden")


def _oeffnen(port: str) -> str:
    """GB-Vorspann, der denselben Port als Ein- und Ausgang oeffnet."""
    return (
        'IMPORT "midi"\n'
        'DIM ein AS MIDI_IN\n'
        'DIM aus AS MIDI_OUT\n'
        'DIM i AS INTEGER\n'
        'FOR i = 0 TO MIDI_IN_COUNT() - 1\n'
        f'    IF MIDI_IN_NAME$(i) = "{port}" THEN ein = MIDI_IN_OPEN(i)\n'
        'NEXT\n'
        'FOR i = 0 TO MIDI_OUT_COUNT() - 1\n'
        f'    IF MIDI_OUT_NAME$(i) = "{port}" THEN aus = MIDI_OUT_OPEN(i)\n'
        'NEXT\n')


@_ohne_loopback
def test_gesendetes_kommt_wieder_an(run_gb):
    """Der Kreis: senden, durch das Betriebssystem, wieder empfangen.

    Das ist der Teil, den die Rust-Tests NICHT abdecken -- sie pruefen die
    Entschluesselung mit erfundenen Bytes. Hier laeuft eine echte Nachricht
    durch midir, den Treiber und den Rueckruf-Faden zurueck.

    Die dritte Nachricht ist die wichtigste: ein Note-AN mit Anschlag 0. So
    schicken die meisten Instrumente ein Note-aus, und hier ist belegt, dass
    es auch ueber einen echten Transport so ankommt.
    """
    out = run_gb(_oeffnen(_LOOPBACK) + """
MIDI_NOTE_ON(aus, 3, 60, 100)
MIDI_NOTE_OFF(aus, 3, 60)
MIDI_NOTE_ON(aus, 1, 64, 0)
MIDI_CC(aus, 16, 7, 90)
SLEEP(400)
DO WHILE MIDI_NEXT(ein)
    DIM art AS STRING
    art = "?"
    IF MIDI_IS_NOTE_ON(ein) THEN art = "an"
    IF MIDI_IS_NOTE_OFF(ein) THEN art = "aus"
    IF MIDI_IS_CC(ein) THEN art = "cc"
    PRINT art; " "; MIDI_CHANNEL(ein); " "; MIDI_DATA1(ein); " "; MIDI_DATA2(ein)
LOOP
MIDI_IN_CLOSE(ein)
MIDI_OUT_CLOSE(aus)
""")
    assert out.splitlines() == [
        "an 3 60 100",     # Note an, Kanal 3
        "aus 3 60 0",      # sauberes Note aus
        "aus 1 64 0",      # Note AN mit Anschlag 0 -- gilt als Note aus
        "cc 16 7 90",      # Regler, Kanal 16 (hoechster)
    ]


@_ohne_loopback
def test_puffer_deckelt_und_wirft_das_aelteste_weg(run_gb):
    """Die Warteschlange fasst 1024 und wirft beim Ueberlauf die AELTESTE weg.

    Beides stand bisher nur in der Doku. 1200 Nachrichten werden geschickt
    und keine abgeholt; danach muessen genau 1024 warten, und die erste
    ueberlebende muss die 176. gesendete sein (1200 - 1024) -- bei
    "juengste faellt weg" waere es die erste.
    """
    out = run_gb(_oeffnen(_LOOPBACK) + """
FOR i = 0 TO 1199
    MIDI_NOTE_ON(aus, 1, 20 + (i MOD 100), 100)
NEXT
SLEEP(1800)
PRINT MIDI_PENDING(ein)
IF MIDI_NEXT(ein) THEN PRINT MIDI_NOTE(ein)
MIDI_IN_CLOSE(ein)
MIDI_OUT_CLOSE(aus)
""")
    zeilen = out.splitlines()
    assert zeilen[0] == "1024", f"Deckel nicht eingehalten: {zeilen}"
    assert zeilen[1] == str(20 + (176 % 100)), (
        f"nicht die aelteste weggefallen: {zeilen}")
