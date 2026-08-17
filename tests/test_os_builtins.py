"""WP A -- Betriebssystem-Anbindung: ARGC/ARG$, GETENV$/SETENV, CWD$/CHDIR,
EXIT, EPRINT, SHELL/SHELL_OUT$.

Golden-Tests gegen die native Runtime. Was hier geprueft wird, laesst sich mit
`run_gb` NICHT pruefen (es wirft bei Exit != 0 und verwirft stderr) -- darum
`run_gb_roh`, das `(code, stdout, stderr)` liefert und Argumente durchreicht.
"""
import os
import sys

import pytest

from drachenhauch.errors import DHRuntimeError


# --------------------------------------------------------------- Argumente

def test_argc_ohne_argumente_ist_null(run_gb):
    assert run_gb("PRINT ARGC()") == "0\n"


def test_argumente_hinter_doppelstrich(run_gb_roh):
    code, out, _ = run_gb_roh(
        'DIM i AS INTEGER\n'
        'PRINT ARGC()\n'
        'FOR i = 0 TO ARGC() - 1\n'
        '    PRINT ARG$(i)\n'
        'NEXT\n', args=["eins", "zwei drei"])
    assert code == 0
    # "zwei drei" bleibt EIN Argument -- die Anfuehrungszeichen der Shell
    # duerfen nicht in zwei zerfallen.
    assert out == "2\neins\nzwei drei\n"


def test_arg_ausserhalb_liefert_leerstring_statt_fehler(run_gb_roh):
    # Bewusste Entscheidung (builtins.rs): Argumente sind Benutzereingabe,
    # `IF ARG$(0) = "" THEN` soll ohne ARGC()-Geruest funktionieren.
    code, out, _ = run_gb_roh('PRINT "[" + ARG$(5) + "]"\nPRINT "[" + ARG$(-1) + "]"')
    assert code == 0
    assert out == "[]\n[]\n"


def test_ohne_doppelstrich_bekommt_das_programm_keine_argumente(dhrt_pfad, tmp_path):
    """Die `--`-Konvention ist der Kern des Entwurfs: ohne sie koennte dhrt sich
    keine eigenen Schalter mehr zulegen, ohne bestehende Programme zu brechen."""
    import subprocess
    quelle = tmp_path / "argc.dh"
    quelle.write_text("PRINT ARGC()", encoding="utf-8")
    r = subprocess.run([dhrt_pfad, "run", str(quelle), "ohne", "trenner"],
                       capture_output=True, text=True, encoding="utf-8", timeout=60)
    assert r.returncode == 0
    assert (r.stdout or "").replace("\r\n", "\n") == "0\n"


# --------------------------------------------------------------- Umgebung

def test_setenv_dann_getenv(run_gb):
    assert run_gb('SETENV("DH_TESTVAR", "hallo")\nPRINT GETENV$("DH_TESTVAR")') == "hallo\n"


def test_getenv_unbekannt_ist_leer_und_nimmt_vorgabe(run_gb):
    out = run_gb('PRINT "[" + GETENV$("DH_GIBTESGANZSICHERNICHT") + "]"\n'
                 'PRINT GETENV$("DH_GIBTESGANZSICHERNICHT", "vorgabe")')
    assert out == "[]\nvorgabe\n"


def test_getenv_sieht_die_umgebung_des_aufrufers(run_gb, monkeypatch):
    monkeypatch.setenv("DH_VON_AUSSEN", "durchgereicht")
    assert run_gb('PRINT GETENV$("DH_VON_AUSSEN")') == "durchgereicht\n"


@pytest.mark.parametrize("name", ['""', '"MIT=GLEICH"'])
def test_setenv_lehnt_ungueltige_namen_ab(run_gb, name):
    with pytest.raises(DHRuntimeError, match="SETENV: ungueltiger Name"):
        run_gb(f'SETENV({name}, "x")')


# ------------------------------------------------- Arbeitsverzeichnis

def test_cwd_ist_das_verzeichnis_der_quelldatei(run_gb, tmp_path):
    # dhrt chdirt beim Start ins Datei-Verzeichnis (relative Asset-Pfade) --
    # CWD$() muss genau das zeigen, sonst ueberrascht es.
    out = run_gb("PRINT CWD$()", base=tmp_path).strip()
    assert os.path.realpath(out) == os.path.realpath(str(tmp_path))


def test_chdir_wirkt_auf_folgende_dateizugriffe(run_gb, tmp_path):
    unter = tmp_path / "unter"
    unter.mkdir()
    (unter / "beleg.txt").write_text("gefunden", encoding="utf-8")
    out = run_gb('PRINT FILEEXISTS("beleg.txt")\n'
                 'CHDIR("unter")\n'
                 'PRINT FILEEXISTS("beleg.txt")\n'
                 'PRINT READALL$(OPENFILE("beleg.txt", "r"))', base=tmp_path)
    assert out == "FALSE\nTRUE\ngefunden\n"


def test_chdir_auf_nichtexistierendes_wirft(run_gb):
    with pytest.raises(DHRuntimeError, match="CHDIR:"):
        run_gb('CHDIR("gibt_es_nicht_xyz_123")')


# ------------------------------------------------------------------- EXIT

def test_exit_setzt_den_rueckgabewert(run_gb_roh):
    code, out, _ = run_gb_roh('PRINT "vorher"\nEXIT(3)\nPRINT "nachher"')
    assert code == 3
    # Die bis dahin gesammelte Ausgabe darf nicht verloren gehen ...
    assert out == "vorher\n"
    # ... und nach EXIT laeuft nichts mehr.
    assert "nachher" not in out


def test_exit_ohne_argument_ist_null(run_gb_roh):
    code, out, _ = run_gb_roh('PRINT "fertig"\nEXIT()')
    assert code == 0
    assert out == "fertig\n"


def test_exit_meldet_keinen_laufzeitfehler(run_gb_roh):
    """EXIT laeuft ueber denselben Kanal wie ein Fehler -- der Nutzer darf davon
    nichts merken."""
    code, _, err = run_gb_roh("EXIT(2)")
    assert code == 2
    assert "Laufzeitfehler" not in err
    assert "__EXIT__" not in err


def test_try_catch_faengt_exit_nicht(run_gb_roh):
    code, out, _ = run_gb_roh('TRY\n'
                              '    EXIT(4)\n'
                              'CATCH e\n'
                              '    PRINT "gefangen"\n'
                              'END TRY\n'
                              'PRINT "danach"')
    assert code == 4
    assert out == ""


def test_throw_exit_sentinel_bleibt_ein_normaler_fehler(run_gb_roh):
    """Der Signalkanal ist das Flag, nicht der Text -- ein Programm darf sich
    keinen Rueckgabewert erschleichen, indem es den Sentinel wirft."""
    code, out, _ = run_gb_roh('TRY\n'
                              '    THROW "__EXIT__"\n'
                              'CATCH e\n'
                              '    PRINT "gefangen: " + e\n'
                              'END TRY')
    assert code == 0
    assert out == "gefangen: __EXIT__\n"


def test_exit_ausserhalb_von_0_bis_255_wirft(run_gb):
    # 256 wuerde vom Betriebssystem zu 0 gekappt -- aus "Fehler" wuerde
    # stillschweigend "alles gut". Darum Fehler mit Ansage.
    with pytest.raises(DHRuntimeError, match="ausserhalb 0..255"):
        run_gb("EXIT(256)")


def test_exit_in_einer_funktion_beendet_das_ganze_programm(run_gb_roh):
    code, out, _ = run_gb_roh('SUB abbrechen()\n'
                              '    EXIT(5)\n'
                              'END SUB\n'
                              'PRINT "start"\n'
                              'abbrechen()\n'
                              'PRINT "nie"')
    assert code == 5
    assert out == "start\n"


# ----------------------------------------------------------------- EPRINT

def test_eprint_geht_nach_stderr_nicht_nach_stdout(run_gb_roh):
    code, out, err = run_gb_roh('PRINT "nutzdaten"\nEPRINT("meldung")')
    assert code == 0
    assert out == "nutzdaten\n"
    assert "meldung" in err
    assert "meldung" not in out


def test_eprint_stringifiziert_wie_print(run_gb_roh):
    _, _, err = run_gb_roh('EPRINT(42)\nEPRINT(TRUE)\nEPRINT(1.5)')
    assert err.splitlines() == ["42", "TRUE", "1.5"]


def test_reihenfolge_von_print_und_eprint_bleibt_erhalten(dhrt_pfad, tmp_path):
    """PRINT wird gepuffert (self.out, geschrieben erst am Ende). Ohne den
    Flush in try_os erschienen alle PRINTs NACH allen EPRINTs, sobald beide
    im selben Terminal landen."""
    import subprocess
    quelle = tmp_path / "reihenfolge.dh"
    quelle.write_text('PRINT "eins"\nEPRINT("zwei")\nPRINT "drei"\n', encoding="utf-8")
    zusammen = tmp_path / "beides.txt"
    with open(zusammen, "w", encoding="utf-8") as f:
        subprocess.run([dhrt_pfad, "run", str(quelle)], stdout=f, stderr=f, timeout=60)
    assert zusammen.read_text(encoding="utf-8").replace("\r\n", "\n") == "eins\nzwei\ndrei\n"


# ------------------------------------------------------------------ SHELL

@pytest.mark.skipif(sys.platform != "win32", reason="benutzt cmd.exe")
def test_shell_liefert_den_rueckgabewert(run_gb):
    assert run_gb('PRINT SHELL("cmd", "/c", "exit 7")') == "7\n"


@pytest.mark.skipif(sys.platform != "win32", reason="benutzt cmd.exe")
def test_shell_out_sammelt_die_ausgabe_ein(run_gb):
    assert run_gb('PRINT TRIM$(SHELL_OUT$("cmd", "/c", "echo hallo"))') == "hallo\n"


@pytest.mark.skipif(sys.platform != "win32", reason="benutzt cmd.exe")
def test_shell_out_nimmt_stderr_nicht_in_die_nutzdaten(run_gb_roh):
    code, out, err = run_gb_roh(
        'PRINT "[" + TRIM$(SHELL_OUT$("cmd", "/c", "echo fehler 1>&2")) + "]"')
    assert code == 0
    assert out == "[]\n"          # stdout des Kindes war leer
    assert "fehler" in err        # stderr wurde durchgereicht, nicht verschluckt


def test_shell_argumente_bleiben_einzeln(run_gb, dhrt_pfad, tmp_path):
    """Argumente werden EINZELN uebergeben, nicht zu einer Kommandozeile
    zusammengeklebt -- ein Wert mit Leerzeichen darf nicht in zwei zerfallen.

    Als Kindprozess laeuft bewusst `dhrt` selbst und nicht `cmd /c`: cmd bringt
    eigene, kaum vorhersagbare Quoting-Regeln mit, die hier nur verdecken
    wuerden, was geprueft werden soll. Nebenbei prueft der Test damit die
    `--`-Konvention von der anderen Seite: SHELL gibt sie weiter, das Kind
    liest sie als ARGC/ARG$.
    """
    kind = tmp_path / "kind.dh"
    kind.write_text('PRINT ARGC()\nPRINT ARG$(0)\n', encoding="utf-8")
    p = str(kind).replace("\\", "/")
    out = run_gb(f'PRINT TRIM$(SHELL_OUT$("{dhrt_pfad.replace(chr(92), "/")}", '
                 f'"run", "{p}", "--", "zwei woerter"))')
    assert out == "1\nzwei woerter\n"


def test_shell_auf_unbekanntes_programm_wirft(run_gb):
    with pytest.raises(DHRuntimeError, match="laesst sich nicht starten"):
        run_gb('PRINT SHELL("gibt_es_ganz_sicher_nicht_xyz123")')


def test_shell_sieht_was_setenv_gesetzt_hat(run_gb):
    """SETENV wirkt auf diesen Prozess UND seine Kinder -- das ist der Weg,
    einem Kindprogramm etwas mitzugeben."""
    if sys.platform != "win32":
        pytest.skip("benutzt cmd.exe")
    out = run_gb('SETENV("DH_FUER_KIND", "weitergereicht")\n'
                 'PRINT TRIM$(SHELL_OUT$("cmd", "/c", "echo %DH_FUER_KIND%"))')
    assert out == "weitergereicht\n"
