"""WP A -- Betriebssystem-Anbindung: ARGC/ARG$, GETENV$/SETENV, CWD$/CHDIR,
EXIT, EPRINT, SHELL/SHELL_OUT$.

Golden-Tests gegen die native Runtime. Was hier geprueft wird, laesst sich mit
`run_gb` NICHT pruefen (es wirft bei Exit != 0 und verwirft stderr) -- darum
`run_gb_roh`, das `(code, stdout, stderr)` liefert und Argumente durchreicht.

Die uebertragbaren Tests liegen als Pruefsammlung in
`tests/pruef/os_builtins.dhtest` (dhrt test; ARGC ohne `--`, GETENV$ und CWD$
seit Stufe 44). Hier bleiben zwei, die dort nicht gehen: die Reihenfolge von
PRINT und EPRINT in EINER Datei (ein Kind ueber PROCESS_START hat getrennte
Kanaele) und SHELL_OUT$ mit der eigenen Exe (SHELL kennt `dhrt` nicht als
Namen der laufenden Laufzeit, anders als PROCESS_START).
"""


# ----------------------------------------------------------------- EPRINT


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
