"""Tests fuer das zeit-Modul (Datumsrechnung).

Golden-Tests gegen `dhrt`: das Programm laeuft im Subprozess, geprueft wird
die Ausgabe. Die Rechenkerne selbst haben zusaetzlich Rust-Unit-Tests in
`rust/drachenhauch_runtime/src/zeit.rs` -- hier geht es um den Weg durch
Compiler und VM: Namen, Argumenttypen, Fehlermeldungen.

Zeitpunkte sind Sekunden seit 1970 in ORTSZEIT. Alle Tests hier rechnen
darum nur mit Werten, die aus dem Modul selbst kommen (ZEIT_PARSE), nie mit
festen Epoch-Zahlen -- die haengen von der Zeitzone der Maschine ab.
"""
import pytest

from drachenhauch.errors import DHRuntimeError


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


def _lauf(run_gb, rumpf):
    return _lines(run_gb('IMPORT "zeit"\n' + rumpf))


# --- Text <-> Zeitpunkt ------------------------------------------------

def test_parse_und_text_sind_umkehrbar(run_gb):
    assert _lauf(run_gb, 'PRINT ZEIT_TEXT$(ZEIT_PARSE("2026-08-28 20:30:00"))\n') == \
        ["2026-08-28 20:30:00"]


@pytest.mark.parametrize("eingabe,erwartet", [
    ("2026-08-28 20:30:00", "2026-08-28 20:30:00"),
    ("2026-08-28T20:30:00", "2026-08-28 20:30:00"),   # ISO mit T (OpenLigaDB)
    ("2026-08-28 20:30", "2026-08-28 20:30:00"),      # ohne Sekunden
    ("2026-08-28", "2026-08-28 00:00:00"),            # nur Datum = Mitternacht
    ("2026-08-28T20:30:00Z", "2026-08-28 20:30:00"),  # Zeitzonen-Anhaengsel
])
def test_parse_versteht_die_ueblichen_schreibweisen(run_gb, eingabe, erwartet):
    assert _lauf(run_gb, f'PRINT ZEIT_TEXT$(ZEIT_PARSE("{eingabe}"))\n') == [erwartet]


def test_parse_meldet_unlesbares_im_klartext(run_gb):
    # Still eine -1 zurueckzugeben waere schlimmer: der Fehler faellt dann
    # erst viel spaeter als unsinnige Rechnung auf.
    with pytest.raises(DHRuntimeError, match="ZEIT_PARSE"):
        run_gb('IMPORT "zeit"\nPRINT ZEIT_PARSE("naechsten Dienstag")\n')


def test_parse_lehnt_unmoegliche_daten_ab(run_gb):
    for text in ("2026-02-30", "2026-13-01", "2026-08-28 25:00:00"):
        with pytest.raises(DHRuntimeError):
            run_gb(f'IMPORT "zeit"\nPRINT ZEIT_PARSE("{text}")\n')


def test_lesbar_fragt_nach_statt_abzubrechen(run_gb):
    assert _lauf(run_gb,
                 'PRINT ZEIT_LESBAR("2026-08-28 20:30:00")\n'
                 'PRINT ZEIT_LESBAR("naechsten Dienstag")\n') == ["TRUE", "FALSE"]


# --- Rechnen -----------------------------------------------------------

def test_tippschluss_ist_anstoss_minus_15_minuten(run_gb):
    assert _lauf(run_gb,
                 'DIM a AS INTEGER : a = ZEIT_PARSE("2026-08-28 20:30:00")\n'
                 'PRINT ZEIT_TEXT$(ZEIT_PLUS(a, -15 * 60))\n') == ["2026-08-28 20:15:00"]


@pytest.mark.parametrize("start,plus,erwartet", [
    ("2026-08-28 00:10:00", -20 * 60, "2026-08-27 23:50:00"),   # ueber Mitternacht
    ("2026-08-31 12:00:00", 86400, "2026-09-01 12:00:00"),      # Monatswechsel
    ("2026-12-31 23:00:00", 3600, "2027-01-01 00:00:00"),       # Jahreswechsel
    ("2028-02-28 12:00:00", 86400, "2028-02-29 12:00:00"),      # Schaltjahr
    ("2026-02-28 12:00:00", 86400, "2026-03-01 12:00:00"),      # kein Schaltjahr
    ("2100-02-28 12:00:00", 86400, "2100-03-01 12:00:00"),      # 2100 ist keins
    ("2000-02-28 12:00:00", 86400, "2000-02-29 12:00:00"),      # 2000 schon
])
def test_plus_geht_ueber_alle_grenzen(run_gb, start, plus, erwartet):
    assert _lauf(run_gb,
                 f'PRINT ZEIT_TEXT$(ZEIT_PLUS(ZEIT_PARSE("{start}"), {plus}))\n') == [erwartet]


def test_diff_zaehlt_sekunden(run_gb):
    assert _lauf(run_gb,
                 'DIM a AS INTEGER : a = ZEIT_PARSE("2026-08-28 20:30:00")\n'
                 'DIM b AS INTEGER : b = ZEIT_PARSE("2026-08-28 18:15:00")\n'
                 'PRINT ZEIT_DIFF(a, b)\n'
                 'PRINT ZEIT_DIFF(b, a)\n') == ["8100", "-8100"]


@pytest.mark.parametrize("sekunden,erwartet", [
    (0, "0 s"),
    (45, "45 s"),
    (720, "12 min"),
    (8100, "2:15 h"),
    (259200, "3 Tage"),
    (86400, "1 Tag"),
    (-3600, "vor 1:00 h"),
    (-45, "vor 45 s"),
])
def test_dauer_liest_sich_wie_gesprochen(run_gb, sekunden, erwartet):
    assert _lauf(run_gb, f'PRINT ZEIT_DAUER$({sekunden})\n') == [erwartet]


# --- Teile und Anzeige -------------------------------------------------

def test_teile_lesen(run_gb):
    assert _lauf(run_gb,
                 'DIM a AS INTEGER : a = ZEIT_PARSE("2026-08-28 20:30:15")\n'
                 'PRINT ZEIT_TEIL(a, "jahr")\n'
                 'PRINT ZEIT_TEIL(a, "monat")\n'
                 'PRINT ZEIT_TEIL(a, "tag")\n'
                 'PRINT ZEIT_TEIL(a, "stunde")\n'
                 'PRINT ZEIT_TEIL(a, "minute")\n'
                 'PRINT ZEIT_TEIL(a, "sekunde")\n') == \
        ["2026", "8", "28", "20", "30", "15"]


def test_unbekanntes_feld_nennt_die_moeglichen(run_gb):
    with pytest.raises(DHRuntimeError, match="ZEIT_TEIL"):
        run_gb('IMPORT "zeit"\nPRINT ZEIT_TEIL(ZEIT_PARSE("2026-08-28"), "quartal")\n')


def test_wochentag_montag_ist_eins(run_gb):
    # 24.08.2026 ist ein Montag.
    assert _lauf(run_gb,
                 'DIM i AS INTEGER\n'
                 'FOR i = 0 TO 6\n'
                 '    PRINT ZEIT_WOCHENTAG(ZEIT_PLUS(ZEIT_PARSE("2026-08-24"), i * 86400));\n'
                 'NEXT\n'
                 'PRINT ""\n') == ["1234567"]


@pytest.mark.parametrize("muster,erwartet", [
    ("TT.MM.JJJJ", "28.08.2026"),
    ("hh:mm", "20:30"),
    ("WT TT.MM. hh:mm", "Fr 28.08. 20:30"),
    ("WTAG, TT.MM.JJJJ", "Freitag, 28.08.2026"),
    ("JJJJ-MM-TT hh:mm:ss", "2026-08-28 20:30:00"),
])
def test_format_setzt_die_muster_ein(run_gb, muster, erwartet):
    assert _lauf(run_gb,
                 f'PRINT ZEIT_FORMAT$(ZEIT_PARSE("2026-08-28 20:30:00"), "{muster}")\n') == \
        [erwartet]


def test_format_ohne_muster_ist_die_normalform(run_gb):
    assert _lauf(run_gb,
                 'PRINT ZEIT_FORMAT$(ZEIT_PARSE("2026-08-28 20:30:00"), "")\n') == \
        ["2026-08-28 20:30:00"]


def test_aus_teilen_bauen(run_gb):
    assert _lauf(run_gb,
                 'PRINT ZEIT_TEXT$(ZEIT_AUS_TEILEN(2026, 8, 28, 20, 30, 0))\n'
                 'PRINT ZEIT_TEXT$(ZEIT_AUS_TEILEN(2026, 8, 28))\n') == \
        ["2026-08-28 20:30:00", "2026-08-28 00:00:00"]


# --- Jetzt -------------------------------------------------------------

def test_jetzt_passt_zu_date_und_time(run_gb):
    """ZEIT_JETZT() und DATE$()/TIME$() muessen dieselbe Uhr meinen --
    sonst rechnet ein Programm mit zwei verschiedenen 'jetzt'."""
    out = _lauf(run_gb,
                'PRINT DATE$() + " " + TIME$()\n'
                'PRINT ZEIT_TEXT$(ZEIT_JETZT())\n')
    # Zwischen den beiden Zeilen kann eine Sekunde vergehen: Datum und
    # Stunde:Minute muessen stimmen, die Sekunde nicht.
    assert out[0][:16] == out[1][:16]


def test_jetzt_ist_umkehrbar(run_gb):
    assert _lauf(run_gb,
                 'DIM j AS INTEGER : j = ZEIT_JETZT()\n'
                 'PRINT ZEIT_PARSE(ZEIT_TEXT$(j)) = j\n') == ["TRUE"]


# --- Zusammenspiel mit der Datenbank ----------------------------------

def test_anstosszeiten_sortieren_gleich_als_text_und_als_zahl(run_gb):
    """Die Datenbank sortiert Anstosszeiten als Text. Das darf nicht
    anders herauskommen als ein Vergleich der Zeitpunkte."""
    assert _lauf(run_gb,
                 'DIM a AS STRING : a = "2026-08-28 20:30:00"\n'
                 'DIM b AS STRING : b = "2026-08-29 15:30:00"\n'
                 'PRINT (a < b) = (ZEIT_PARSE(a) < ZEIT_PARSE(b))\n') == ["TRUE"]
