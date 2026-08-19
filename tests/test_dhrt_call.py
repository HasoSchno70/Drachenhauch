"""`dhrt call <datei> <funktion> [arg]` -- Schritt 1 zu TASK_START.

Ein Auftrag soll als eigener PROZESS laufen, nicht als Thread: `Value` haelt
ueberall `Rc`, `Program` ist damit weder Send noch Sync. Ein Prozess teilt
keinen Speicher, also verschwindet das Problem, ohne dass eine Zeile an
`Value` angefasst werden muss. Siehe docs/entwurf-task-start.md.

Dieser Einstiegspunkt fuehrt EINE Funktion aus und laesst das Hauptprogramm
stehen. Dass dabei die Globals nicht gesetzt sind, ist die Zusage, nicht die
Panne -- dieselbe Grenze wie bei einem mit `AS` importierten Modul.
"""
import json
import subprocess


QUELLE = (
    "CONST FAKTOR AS INTEGER = 10\n"
    "\n"
    "FUNCTION Doppelt(x AS INTEGER) AS INTEGER\n"
    "    RETURN x * 2\n"
    "END FUNCTION\n"
    "\n"
    "FUNCTION MitAusgabe(x AS INTEGER) AS INTEGER\n"
    '    PRINT "rechne " + STR$(x)\n'
    "    RETURN x + 1\n"
    "END FUNCTION\n"
    "\n"
    "FUNCTION Gruss(wer AS STRING) AS STRING\n"
    '    RETURN "hallo " + wer\n'
    "END FUNCTION\n"
    "\n"
    "FUNCTION SiehtGlobal() AS INTEGER\n"
    "    RETURN FAKTOR\n"
    "END FUNCTION\n"
    "\n"
    'PRINT "HAUPTPROGRAMM"\n'
)


def _call(dhrt_pfad, tmp_path, funktion, arg=None):
    datei = tmp_path / "c.dh"
    datei.write_text(QUELLE, encoding="utf-8")
    befehl = [dhrt_pfad, "call", str(datei), funktion]
    if arg is not None:
        befehl.append(str(arg))
    r = subprocess.run(befehl, capture_output=True, text=True,
                       encoding="utf-8", timeout=60)
    return json.loads((r.stdout or "").strip().split("\n")[-1])


def test_ruft_die_funktion_und_liefert_das_ergebnis(dhrt_pfad, tmp_path):
    a = _call(dhrt_pfad, tmp_path, "Doppelt", 21)
    assert a["ok"] is True
    assert a["ergebnis"] == 42


def test_hauptprogramm_laeuft_NICHT_mit(dhrt_pfad, tmp_path):
    """Der Kern der Sache. Liefe das Hauptprogramm mit, staende hier
    "HAUPTPROGRAMM" -- und ein Auftrag wuerde jedes Mal das ganze Spiel
    starten."""
    a = _call(dhrt_pfad, tmp_path, "Doppelt", 1)
    assert "HAUPTPROGRAMM" not in a["ausgabe"]
    assert a["ausgabe"] == ""


def test_ausgabe_der_funktion_kommt_getrennt(dhrt_pfad, tmp_path):
    """PRINT innerhalb der Funktion landet in `ausgabe`, nicht im Ergebnis --
    sonst muesste der Aufrufer beides auseinanderfieseln."""
    a = _call(dhrt_pfad, tmp_path, "MitAusgabe", 5)
    assert a["ergebnis"] == 6
    assert a["ausgabe"].strip() == "rechne 5"


def test_zahl_bleibt_zahl(dhrt_pfad, tmp_path):
    """Ein Argument, das wie eine Zahl aussieht, wird eine -- sonst muesste
    jede Auftragsfunktion ihr Argument selbst umwandeln."""
    a = _call(dhrt_pfad, tmp_path, "Doppelt", 7)
    assert a["ergebnis"] == 14


def test_text_bleibt_text(dhrt_pfad, tmp_path):
    a = _call(dhrt_pfad, tmp_path, "Gruss", "welt")
    assert a["ergebnis"] == "hallo welt"


def test_ohne_argument(dhrt_pfad, tmp_path):
    a = _call(dhrt_pfad, tmp_path, "SiehtGlobal")
    assert a["ok"] is False       # greift auf ein Global zu, siehe unten


def test_globals_sind_nicht_gesetzt_und_das_MELDET_sich(dhrt_pfad, tmp_path):
    """Die Zusage: ein Auftrag sieht keine Globals.

    Der Entwurf befuerchtete, eine frische VM liefere dafuer still einen
    VORGABEWERT -- eine Funktion mit `CONST` haette dann heimlich falsch
    gerechnet. Nachgemessen ist es besser: der Zugriff meldet sich. Damit ist
    der Fall laut, nicht leise, und das war das eigentliche Risiko.
    """
    a = _call(dhrt_pfad, tmp_path, "SiehtGlobal")
    assert a["ok"] is False
    assert a["fehler"], a


def test_unbekannte_funktion_nennt_die_bekannten(dhrt_pfad, tmp_path):
    a = _call(dhrt_pfad, tmp_path, "Gibtsnicht", 1)
    assert a["ok"] is False
    assert "gibt es nicht" in a["fehler"]
    assert "doppelt" in a["fehler"]        # die bekannten stehen daneben


def test_antwort_ist_immer_eine_json_zeile(dhrt_pfad, tmp_path):
    """Auch im Fehlerfall -- der Aufrufer ist eine Maschine und soll nicht
    stderr parsen muessen."""
    for fn, arg in [("Doppelt", 2), ("Gibtsnicht", 2), ("SiehtGlobal", None)]:
        a = _call(dhrt_pfad, tmp_path, fn, arg)
        assert "ok" in a, (fn, a)
