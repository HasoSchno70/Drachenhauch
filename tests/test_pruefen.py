"""WP E -- Pruefen und Melden: ASSERT/ASSERT_EQ, Sammel-Modus, Bilanz,
LOG_DEBUG/INFO/WARN/ERROR.

Alle Faelle liegen als Pruefsammlung in `tests/pruef/pruefen.dhtest`
(dhrt test). Hier bleibt nur die Reihenfolge von PRINT und LOG_* in EINEM
gemeinsamen Ausgabestrom -- der Laeufer von `dhrt test` liest stdout und
stderr getrennt und saehe sie nicht.
"""


def test_reihenfolge_von_print_und_log_bleibt_erhalten(dhrt_pfad, tmp_path):
    """Wie bei EPRINT (WP A): PRINT wird gepuffert, LOG_* muss den Puffer
    vorher leeren -- sonst staenden alle Meldungen vor allen Nutzdaten."""
    import subprocess
    quelle = tmp_path / "reihenfolge.dh"
    quelle.write_text('PRINT "eins"\nLOG_INFO("zwei")\nPRINT "drei"\n', encoding="utf-8")
    zusammen = tmp_path / "beides.txt"
    with open(zusammen, "w", encoding="utf-8") as f:
        subprocess.run([dhrt_pfad, "run", str(quelle)], stdout=f, stderr=f, timeout=60)
    zeilen = zusammen.read_text(encoding="utf-8").replace("\r\n", "\n").splitlines()
    assert zeilen[0] == "eins"
    assert "zwei" in zeilen[1]
    assert zeilen[2] == "drei"
