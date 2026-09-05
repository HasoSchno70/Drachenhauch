"""Schriftmasse der acht proportionalen Standardschriften fuer das pdf-Modul.

Erzeugt `rust/drachenhauch_runtime/src/pdf_masse.rs`: je Schrift die Breite
jedes WinAnsi-Bytes 32..255 in Tausendsteln der Schriftgroesse. Quelle ist
NICHT das Gedaechtnis, sondern PyMuPDF, das die Base-14-Schriften samt
Metriken mitbringt -- genau die Zahlen, mit denen jeder PDF-Leser den Text
setzt. Ein Byte ohne Zeichen (in cp1252 unbelegt oder ohne Glyphe) steht als 0
und ist beim Messen ein Fehler statt einer Schaetzung.

Aufruf: .venv\\Scripts\\python.exe tools\\gen_pdf_masse.py
Gegenprobe: tests/test_pdf.py misst PDF_TEXT_WIDTH gegen PyMuPDF.
"""
from __future__ import annotations

import io
from pathlib import Path

import fitz  # PyMuPDF

WURZEL = Path(__file__).resolve().parent.parent
ZIEL = WURZEL / "rust" / "drachenhauch_runtime" / "src" / "pdf_masse.rs"

# (Name der Konstante, PyMuPDF-Kuerzel) in der Reihenfolge von SCHRIFTEN in pdf.rs
SCHRIFTEN = [
    ("HELVETICA", "helv"), ("HELVETICA_FETT", "hebo"),
    ("HELVETICA_KURSIV", "heit"), ("HELVETICA_FETT_KURSIV", "hebi"),
    ("TIMES", "tiro"), ("TIMES_FETT", "tibo"),
    ("TIMES_KURSIV", "tiit"), ("TIMES_FETT_KURSIV", "tibi"),
]


def breiten(alias: str) -> list[int]:
    font = fitz.Font(alias)
    aus = []
    for code in range(32, 256):
        try:
            zeichen = bytes([code]).decode("cp1252")
        except UnicodeDecodeError:
            aus.append(0)
            continue
        if not font.has_glyph(ord(zeichen)):
            aus.append(0)
            continue
        aus.append(int(round(font.text_length(zeichen, fontsize=1000))))
    return aus


def main() -> None:
    zeilen = [
        "// Von tools/gen_pdf_masse.py aus PyMuPDFs Base-14-Metriken erzeugt: Breite je",
        "// WinAnsi-Byte 32..255 in Tausendsteln der Schriftgroesse, 0 = kein Zeichen.",
        "// Nicht von Hand aendern -- neu erzeugen.",
        "",
    ]
    for name, alias in SCHRIFTEN:
        ws = breiten(alias)
        assert len(ws) == 224
        zeilen.append(f"pub const BREITEN_{name}: [u16; 224] = [")
        for i in range(0, 224, 16):
            zeilen.append("    " + ", ".join(str(x) for x in ws[i:i + 16]) + ",")
        zeilen.append("];")
        zeilen.append("")
    io.open(ZIEL, "w", encoding="utf-8", newline="\n").write("\n".join(zeilen))
    print(f"geschrieben: {ZIEL}")


if __name__ == "__main__":
    main()
