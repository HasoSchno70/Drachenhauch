"""`TASK_*` -- eine eigene GB-Funktion im Hintergrund (WP H, Weg C).

Der Auftrag laeuft als EIGENER dhrt-Prozess, nicht als Thread: `Value` haelt
ueberall `Rc`, `Program` ist damit weder Send noch Sync und laesst sich nicht
ueber eine Thread-Grenze reichen. Ein Prozess teilt keinen Speicher -- damit
verschwindet das Problem, ohne eine Zeile an `Value`.

Die Prozessgrenze ist zugleich die ZUSAGE: ein Auftrag sieht keine Globals des
Hauptprogramms, auch keine CONST. Er bekommt mit, was er braucht. Dieselbe
Entscheidung wie bei einem mit `AS` importierten Modul (WP I.1).

Siehe docs/entwurf-task-start.md.
"""
import subprocess

import pytest

Q = chr(34)      # Anfuehrungszeichen im GB-Quelltext


# Der Auftrag muss nicht LANGE rechnen -- allein der Prozessstart kostet rund
# 12 ms, und darauf warten die Tests. Eine grosse Schleife hier hiesse nur,
# auf einem schwachen Rechner (CI: zwei Kerne) beide Seiten gegeneinander
# rechnen zu lassen, waehrend die Warteschleife einen Kern verbrennt.
LANGSAM = (
    "FUNCTION Langsam(x AS INTEGER) AS INTEGER\n"
    "    DIM i AS INTEGER\n"
    "    DIM s AS INTEGER\n"
    "    s = 0\n"
    "    FOR i = 1 TO 2000\n"
    "        s = s + i MOD 7\n"
    "    NEXT\n"
    "    RETURN x * 2\n"
    "END FUNCTION\n"
)

GRUSS = (
    "FUNCTION Gruss(wer AS STRING) AS STRING\n"
    '    RETURN "hallo " + wer\n'
    "END FUNCTION\n"
)


def _lauf(dhrt_pfad, tmp_path, quelle):
    datei = tmp_path / "t.dh"
    datei.write_text(quelle, encoding="utf-8")
    r = subprocess.run([dhrt_pfad, "run", str(datei)], capture_output=True,
                       text=True, encoding="utf-8", timeout=120)
    return ((r.stdout or "").replace("\r\n", "\n"),
            (r.stderr or "").replace("\r\n", "\n"))


def test_auftrag_rechnet_und_liefert(dhrt_pfad, tmp_path):
    out, err = _lauf(dhrt_pfad, tmp_path, LANGSAM +
                     "DIM a AS INTEGER\n"
                     "a = TASK_START(Langsam, 21)\n"
                     "WHILE NOT TASK_READY(a)\n"
                     "    SLEEP(2)\n"
                     "WEND\n"
                     "PRINT TASK_RESULT$(a)\n")
    assert out.strip() == "42", (out, err)


def test_die_hauptschleife_laeuft_weiter(dhrt_pfad, tmp_path):
    """Der ganze Zweck: waehrend der Auftrag rechnet, blockiert nichts."""
    out, err = _lauf(dhrt_pfad, tmp_path, LANGSAM +
                     "DIM a AS INTEGER\n"
                     "DIM runden AS INTEGER\n"
                     "a = TASK_START(Langsam, 1)\n"
                     "runden = 0\n"
                     "WHILE NOT TASK_READY(a)\n"
                     "    runden = runden + 1\n"
                     "WEND\n"
                     "PRINT STR$(runden > 0)\n")
    assert out.strip() == "TRUE", (out, err)


def test_pending_zaehlt(dhrt_pfad, tmp_path):
    out, err = _lauf(dhrt_pfad, tmp_path, LANGSAM +
                     "DIM a AS INTEGER\n"
                     "a = TASK_START(Langsam, 1)\n"
                     "PRINT TASK_PENDING()\n"
                     "WHILE NOT TASK_READY(a)\n"
                     "    SLEEP(2)\n"
                     "WEND\n"
                     'PRINT TASK_RESULT$(a)\n'
                     "PRINT TASK_PENDING()\n")
    zeilen = out.split("\n")
    assert zeilen[0] == "1", (out, err)
    assert zeilen[2] == "0", (out, err)


def test_text_als_argument_und_ergebnis(dhrt_pfad, tmp_path):
    out, err = _lauf(dhrt_pfad, tmp_path, GRUSS +
                     "DIM a AS INTEGER\n"
                     'a = TASK_START(Gruss, "welt")\n'
                     "WHILE NOT TASK_READY(a)\n"
                     "    SLEEP(2)\n"
                     "WEND\n"
                     "PRINT TASK_RESULT$(a)\n")
    assert out.strip() == "hallo welt", (out, err)


def test_auftrag_sieht_die_globals_NICHT(dhrt_pfad, tmp_path):
    """Die Zusage, und sie meldet sich statt still zu rechnen."""
    _, err = _lauf(dhrt_pfad, tmp_path,
                   "CONST FAKTOR AS INTEGER = 10\n"
                   "FUNCTION Nutzt() AS INTEGER\n"
                   "    RETURN FAKTOR\n"
                   "END FUNCTION\n"
                   "DIM a AS INTEGER\n"
                   "a = TASK_START(Nutzt)\n"
                   "WHILE NOT TASK_READY(a)\n"
                     "    SLEEP(2)\n"
                   "WEND\n"
                   "PRINT TASK_RESULT$(a)\n")
    assert "Hauptprogramm laeuft dabei NICHT" in err, err


def test_unbekannte_funktion_meldet(dhrt_pfad, tmp_path):
    _, err = _lauf(dhrt_pfad, tmp_path, LANGSAM +
                   "DIM a AS INTEGER\n"
                   'a = TASK_START("Gibtsnicht", 1)\n'
                   "WHILE NOT TASK_READY(a)\n"
                     "    SLEEP(2)\n"
                   "WEND\n"
                   "PRINT TASK_RESULT$(a)\n")
    assert "gibt es nicht" in err, err


def test_abholen_vor_fertig_meldet_klar(dhrt_pfad, tmp_path):
    """Statt zu blockieren oder Unsinn zu liefern: sag, was zu tun ist."""
    _, err = _lauf(dhrt_pfad, tmp_path, LANGSAM +
                   "DIM a AS INTEGER\n"
                   "a = TASK_START(Langsam, 1)\n"
                   "PRINT TASK_RESULT$(a)\n")
    assert "noch nicht fertig" in err, err
    assert "TASK_READY" in err, err


def test_ergebnis_ist_nur_einmal_abholbar(dhrt_pfad, tmp_path):
    """Wie bei SHELL_RESULT$ -- `abholen` nimmt es aus der Verwaltung. Genau
    darum gibt es kein zweites TASK_OUTPUT$ daneben: zwei Abholer wuerden sich
    gegenseitig das Ergebnis wegnehmen."""
    _, err = _lauf(dhrt_pfad, tmp_path, LANGSAM +
                   "DIM a AS INTEGER\n"
                   "a = TASK_START(Langsam, 1)\n"
                   "WHILE NOT TASK_READY(a)\n"
                     "    SLEEP(2)\n"
                   "WEND\n"
                   "PRINT TASK_RESULT$(a)\n"
                   "PRINT TASK_RESULT$(a)\n")
    assert err.strip() != "", "der zweite Abruf muss sich melden"


def test_abbrechen(dhrt_pfad, tmp_path):
    out, err = _lauf(dhrt_pfad, tmp_path, LANGSAM +
                     "DIM a AS INTEGER\n"
                     "a = TASK_START(Langsam, 1)\n"
                     "TASK_CANCEL(a)\n"
                     "PRINT TASK_PENDING()\n")
    assert out.strip() == "0", (out, err)


def test_kein_funktionsname_meldet_mit_beispiel(dhrt_pfad, tmp_path):
    _, err = _lauf(dhrt_pfad, tmp_path,
                   "DIM a AS INTEGER\n"
                   "a = TASK_START(42)\n")
    assert "TASK_START(Rechne, 42)" in err, err


def test_untragbares_argument_meldet(dhrt_pfad, tmp_path):
    """Ein Objekt geht nicht ueber eine Prozessgrenze -- und das soll die
    Meldung sagen, statt es zu versuchen."""
    _, err = _lauf(dhrt_pfad, tmp_path, LANGSAM +
                   "DIM feld AS ARRAY OF INTEGER\n"
                   "DIM a AS INTEGER\n"
                   "a = TASK_START(Langsam, feld)\n")
    assert "Prozessgrenze" in err, err


def test_mehrere_auftraege_nebeneinander(dhrt_pfad, tmp_path):
    out, err = _lauf(dhrt_pfad, tmp_path, LANGSAM +
                     "DIM a AS INTEGER\n"
                     "DIM b AS INTEGER\n"
                     "a = TASK_START(Langsam, 1)\n"
                     "b = TASK_START(Langsam, 2)\n"
                     "PRINT TASK_PENDING()\n"
                     "WHILE NOT TASK_READY(a)\n"
                     "    SLEEP(2)\n"
                     "WEND\n"
                     "WHILE NOT TASK_READY(b)\n"
                     "    SLEEP(2)\n"
                     "WEND\n"
                     'PRINT TASK_RESULT$(a) + "," + TASK_RESULT$(b)\n')
    zeilen = out.split("\n")
    assert zeilen[0] == "2", (out, err)
    assert zeilen[1] == "2,4", (out, err)


def test_pending_zaehlt_das_UNABGEHOLTE_nicht_das_laufende(dhrt_pfad, tmp_path):
    """Die Falle, in die ich beim Schreiben dieser Tests selbst getappt bin.

    `TASK_PENDING` zaehlt Auftraege, die noch nicht ABGEHOLT sind -- ein
    fertiger, aber nicht abgeholter Auftrag zaehlt mit. Wer also
    `WHILE TASK_PENDING() > 0` schreibt und erst danach abholen will, wartet
    ewig. Dasselbe gilt fuer SHELL_PENDING und DB_QUERY_PENDING.
    """
    out, err = _lauf(dhrt_pfad, tmp_path, LANGSAM +
                     "DIM a AS INTEGER\n"
                     "a = TASK_START(Langsam, 1)\n"
                     "WHILE NOT TASK_READY(a)\n"
                     "    SLEEP(2)\n"
                     "WEND\n"
                     "PRINT TASK_PENDING()\n"
                     "PRINT TASK_RESULT$(a)\n"
                     "PRINT TASK_PENDING()\n")
    zeilen = out.split("\n")
    assert zeilen[0] == "1", (out, err)     # fertig, aber nicht abgeholt
    assert zeilen[2] == "0", (out, err)     # erst das Abholen gibt frei


# --- Mehrere Argumente ----------------------------------------------------

MEHR = (
    "FUNCTION Summe(a AS INTEGER, b AS INTEGER, c AS INTEGER) AS INTEGER\n"
    "    RETURN a + b + c\n"
    "END FUNCTION\n"
    "FUNCTION Satz(wer AS STRING, wie AS STRING) AS STRING\n"
    "    RETURN wer + " + Q + " ist " + Q + " + wie\n"
    "END FUNCTION\n"
)


def test_drei_zahlen(dhrt_pfad, tmp_path):
    out, err = _lauf(dhrt_pfad, tmp_path, MEHR +
                     "DIM a AS INTEGER\n"
                     "a = TASK_START(Summe, 1, 2, 3)\n"
                     "WHILE NOT TASK_READY(a)\n"
                     "    SLEEP(2)\n"
                     "WEND\n"
                     "PRINT TASK_RESULT$(a)\n")
    assert out.strip() == "6", (out, err)


def test_text_mit_leerzeichen_bleibt_EIN_argument(dhrt_pfad, tmp_path):
    """Die Argumente gehen als eigene Kommandozeilen-Worte an `dhrt call` --
    also weder getrennt noch zusammengeklebt. Ein Text mit Leerzeichen waere
    genau die Stelle, an der eine schludrige Uebergabe auffliegt."""
    out, err = _lauf(dhrt_pfad, tmp_path, MEHR +
                     "DIM a AS INTEGER\n"
                     "a = TASK_START(Satz, " + Q + "der Text mit Leerzeichen" + Q +
                     ", " + Q + "heil" + Q + ")\n"
                     "WHILE NOT TASK_READY(a)\n"
                     "    SLEEP(2)\n"
                     "WEND\n"
                     "PRINT TASK_RESULT$(a)\n")
    assert out.strip() == "der Text mit Leerzeichen ist heil", (out, err)


def test_ohne_argument_geht_weiterhin(dhrt_pfad, tmp_path):
    out, err = _lauf(dhrt_pfad, tmp_path,
                     "FUNCTION Nix() AS INTEGER\n"
                     "    RETURN 7\n"
                     "END FUNCTION\n"
                     "DIM a AS INTEGER\n"
                     "a = TASK_START(Nix)\n"
                     "WHILE NOT TASK_READY(a)\n"
                     "    SLEEP(2)\n"
                     "WEND\n"
                     "PRINT TASK_RESULT$(a)\n")
    assert out.strip() == "7", (out, err)


def test_zu_wenige_argumente_melden_sich(dhrt_pfad, tmp_path):
    """Die Stelligkeit prueft der Auftrag selbst, im Kindprozess -- die
    Meldung muss trotzdem beim Aufrufer ankommen."""
    _, err = _lauf(dhrt_pfad, tmp_path, MEHR +
                   "DIM a AS INTEGER\n"
                   "a = TASK_START(Summe, 1)\n"
                   "WHILE NOT TASK_READY(a)\n"
                   "    SLEEP(2)\n"
                   "WEND\n"
                   "PRINT TASK_RESULT$(a)\n")
    assert err.strip() != "", "zu wenige Argumente muessen sich melden"


def test_untragbares_argument_nennt_die_stelle(dhrt_pfad, tmp_path):
    _, err = _lauf(dhrt_pfad, tmp_path, MEHR +
                   "DIM feld AS ARRAY OF INTEGER\n"
                   "DIM a AS INTEGER\n"
                   "a = TASK_START(Summe, 1, feld, 3)\n")
    assert "Prozessgrenze" in err, err
    assert "Argument 2" in err, err
