"""WP I.1 -- `IMPORT "x.dh" AS x` eroeffnet einen Namensraum.

Bis hierher teilten sich alle per `IMPORT` zusammengefuegten Dateien einen
flachen Namensraum mit ueber 1400 Builtins: zwei Bibliotheken mit einer
Funktion `Init` liessen sich nicht gemeinsam benutzen. Der Alias wurde bei
QUELLDATEIEN sogar stillschweigend verworfen -- der Regex im Preprocessor
kannte `AS` laengst, ausgewertet haben ihn nur die eingebauten Module.

Umgesetzt ist das als Umbenennung zur Uebersetzungszeit (`namensraum.rs`):
aus `Quadrat` in `mathe.dh` wird intern `mathe@quadrat`. VM, Bytecode und
Debugger bleiben unberuehrt.

Die uebertragbaren Tests liegen seit 2026-09-08 als Pruefsammlung in
`tests/pruef/namensraeume.dhtest` (dhrt test); hier bleiben nur die, die ein Bild,
eine geschriebene Datei oder den Quelltext mit einem fremden Leser pruefen.
"""
import subprocess

NL = chr(10)
Q = chr(34)


def _lauf(dhrt_pfad, tmp_path, dateien: dict, haupt="main.dh"):
    """Schreibt die Dateien und laesst `haupt` laufen. Liefert (stdout, stderr)."""
    for name, inhalt in dateien.items():
        (tmp_path / name).write_text(inhalt, encoding="utf-8")
    r = subprocess.run([dhrt_pfad, "run", str(tmp_path / haupt)],
                       capture_output=True, text=True, encoding="utf-8", timeout=60)
    return ((r.stdout or "").replace("\r\n", "\n"),
            (r.stderr or "").replace("\r\n", "\n"))


MATHE = (
    "CONST FAKTOR AS INTEGER = 10\n"
    "\n"
    "FUNCTION Quadrat(x AS INTEGER) AS INTEGER\n"
    "    RETURN x * x\n"
    "END FUNCTION\n"
    "\n"
    "FUNCTION Skaliert(x AS INTEGER) AS INTEGER\n"
    "    DIM h AS INTEGER\n"
    "    h = Quadrat(x)\n"
    "    RETURN h * FAKTOR\n"
    "END FUNCTION\n"
)


# --- PRIVATE -------------------------------------------------------------

PRIV = (
    "PRIVATE CONST GEHEIM AS INTEGER = 42\n"
    "\n"
    "PRIVATE FUNCTION Intern(x AS INTEGER) AS INTEGER\n"
    "    RETURN x + GEHEIM\n"
    "END FUNCTION\n"
    "\n"
    "FUNCTION Offen(x AS INTEGER) AS INTEGER\n"
    "    RETURN Intern(x)\n"
    "END FUNCTION\n"
)


def test_private_vor_etwas_anderem_meldet(dhrt_pfad, tmp_path):
    _, err = _lauf(dhrt_pfad, tmp_path, {
        "main.dh": "PRIVATE PRINT 1\n",
    })
    assert "PRIVATE steht vor" in err, err


# --- Abschottung gegen die Globals des Hauptprogramms --------------------

GLOBAL_LESER = ("FUNCTION LiestGlobal() AS INTEGER\n"
                "    RETURN punkte\n"
                "END FUNCTION\n")


# --- WP I.2: Typen aus dem Namensraum ------------------------------------

KLASSE = ("CLASS Punkt\n"
          "    DIM x AS INTEGER\n"
          "    DIM y AS INTEGER\n"
          "END CLASS\n"
          "FUNCTION Neu(a AS INTEGER) AS Punkt\n"
          "    DIM p AS Punkt\n"
          "    p = NEW Punkt()\n"
          "    p.x = a\n"
          "    RETURN p\n"
          "END FUNCTION\n")


# --- WP I.3: ENUMs aus dem Namensraum ------------------------------------

ENUM_DATEI = ("ENUM Farbe" + NL +
              "    ROT" + NL +
              "    GRUEN" + NL +
              "    BLAU" + NL +
              "END ENUM" + NL +
              "FUNCTION Name$(f AS Farbe) AS STRING" + NL +
              "    IF f = Farbe.ROT THEN RETURN " + Q + "rot" + Q + NL +
              "    IF f = Farbe.GRUEN THEN RETURN " + Q + "gruen" + Q + NL +
              "    RETURN " + Q + "blau" + Q + NL +
              "END FUNCTION" + NL)
