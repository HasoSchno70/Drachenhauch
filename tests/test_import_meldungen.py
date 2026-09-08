"""WP I.4 -- Meldungen, die auf die richtige Datei und Zeile zeigen.

`IMPORT` fuegt Text ein; danach zeigte jede Zeilennummer in die GEMERGTE
Quelle. Eine Datei mit zwei Zeilen bekam so ein "datei.dh:6", und eine
Namenskollision zwischen zwei Bibliotheken nannte weder Zeile noch Datei.

Belegt die Punkte (a) und (d) aus `docs/entwurf-namensraeume.md`.

Die uebertragbaren Tests liegen seit 2026-09-08 als Pruefsammlung in
`tests/pruef/import_meldungen.dhtest` (dhrt test); hier bleiben nur die, die ein Bild,
eine geschriebene Datei oder den Quelltext mit einem fremden Leser pruefen.
"""
import pytest


def _lauf(dhrt_pfad, tmp_path, dateien: dict, haupt="main.dh"):
    """Schreibt mehrere .dh-Dateien und laesst `haupt` laufen. Liefert stderr."""
    import subprocess
    for name, inhalt in dateien.items():
        (tmp_path / name).write_text(inhalt, encoding="utf-8")
    r = subprocess.run([dhrt_pfad, "run", str(tmp_path / haupt)],
                       capture_output=True, text=True, encoding="utf-8", timeout=60)
    return (r.stderr or "").replace("\r\n", "\n")


# ------------------------------------------------------- (a) Kollisionen


def test_kollision_ohne_import_nennt_jetzt_auch_eine_zeile(dhrt_pfad, tmp_path):
    """Auch ohne IMPORT eine Verbesserung: vorher stand da gar keine Zeile."""
    err = _lauf(dhrt_pfad, tmp_path, {
        "main.dh": ("FUNCTION F() AS INTEGER\n    RETURN 1\nEND FUNCTION\n"
                    "FUNCTION F() AS INTEGER\n    RETURN 2\nEND FUNCTION\n"),
    })
    assert "main.dh:4" in err, err      # zweite Deklaration
    assert "main.dh:1" in err, err      # erste


# ------------------------------------------- (d) Zeilen zeigen richtig


# --- BOM ------------------------------------------------------------------

def test_bom_in_der_hauptdatei_stoert_nicht(dhrt_pfad, tmp_path):
    r"""Windows-Editoren schreiben UTF-8 gern mit Byte-Reihenfolge-Markierung.

    Der Lexer meldete dafuer "Unbekanntes Zeichen: '\u{feff}'" in Zeile 1 --
    die schlimmste Sorte Fehler: das Zeichen ist unsichtbar, die Meldung sagt
    einem Menschen nichts, und die Datei sieht im Editor voellig richtig aus.
    Gefunden beim Testen des Installers, weil PowerShell beim Schreiben der
    Probedatei von sich aus ein BOM setzte.
    """
    import subprocess
    (tmp_path / "main.dh").write_bytes(b"\xef\xbb\xbfPRINT 42\n")
    r = subprocess.run([dhrt_pfad, "run", str(tmp_path / "main.dh")],
                       capture_output=True, text=True, encoding="utf-8", timeout=60)
    assert (r.stdout or "").strip() == "42", (r.stdout, r.stderr)
