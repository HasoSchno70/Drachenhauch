"""WP I.1 -- `IMPORT "x.dh" AS x` eroeffnet einen Namensraum.

Bis hierher teilten sich alle per `IMPORT` zusammengefuegten Dateien einen
flachen Namensraum mit ueber 1400 Builtins: zwei Bibliotheken mit einer
Funktion `Init` liessen sich nicht gemeinsam benutzen. Der Alias wurde bei
QUELLDATEIEN sogar stillschweigend verworfen -- der Regex im Preprocessor
kannte `AS` laengst, ausgewertet haben ihn nur die eingebauten Module.

Umgesetzt ist das als Umbenennung zur Uebersetzungszeit (`namensraum.rs`):
aus `Quadrat` in `mathe.dh` wird intern `mathe@quadrat`. VM, Bytecode und
Debugger bleiben unberuehrt.
"""
import subprocess


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


def test_qualifizierter_aufruf(dhrt_pfad, tmp_path):
    out, err = _lauf(dhrt_pfad, tmp_path, {
        "mathe.dh": MATHE,
        "main.dh": 'IMPORT "mathe.dh" AS mathe\nPRINT mathe.Quadrat(5)\n',
    })
    assert out.strip() == "25", (out, err)


def test_konstante_ueber_den_namensraum(dhrt_pfad, tmp_path):
    out, _ = _lauf(dhrt_pfad, tmp_path, {
        "mathe.dh": MATHE,
        "main.dh": 'IMPORT "mathe.dh" AS mathe\nPRINT mathe.FAKTOR\n',
    })
    assert out.strip() == "10"


def test_modul_ruft_sich_selbst_unqualifiziert(dhrt_pfad, tmp_path):
    """Innerhalb der Datei bleibt `Quadrat` schlicht `Quadrat`.

    `Skaliert` ruft `Quadrat` und liest `FAKTOR` -- beides ohne Praefix im
    Quelltext, beides muss auf die umbenannten Namen zeigen.
    """
    out, err = _lauf(dhrt_pfad, tmp_path, {
        "mathe.dh": MATHE,
        "main.dh": 'IMPORT "mathe.dh" AS mathe\nPRINT mathe.Skaliert(3)\n',
    })
    assert out.strip() == "90", (out, err)


def test_gleicher_name_in_beiden_dateien(dhrt_pfad, tmp_path):
    """Der eigentliche Zweck: dieselbe Funktion zweimal, ohne Kollision."""
    out, err = _lauf(dhrt_pfad, tmp_path, {
        "mathe.dh": MATHE,
        "main.dh": ('IMPORT "mathe.dh" AS mathe\n'
                    "FUNCTION Quadrat(x AS INTEGER) AS INTEGER\n"
                    "    RETURN 999\n"
                    "END FUNCTION\n"
                    "PRINT Quadrat(5)\n"
                    "PRINT mathe.Quadrat(5)\n"),
    })
    assert out.split() == ["999", "25"], (out, err)


def test_ohne_alias_kollidiert_es_weiterhin(dhrt_pfad, tmp_path):
    """Gegenprobe: ohne `AS` bleibt es beim Fehler aus WP I.4."""
    _, err = _lauf(dhrt_pfad, tmp_path, {
        "mathe.dh": MATHE,
        "main.dh": ('IMPORT "mathe.dh"\n'
                    "FUNCTION Quadrat(x AS INTEGER) AS INTEGER\n"
                    "    RETURN 999\n"
                    "END FUNCTION\n"
                    "PRINT Quadrat(5)\n"),
    })
    assert "schon in mathe.dh:3 deklariert" in err, err


def test_unbekannter_name_im_namensraum(dhrt_pfad, tmp_path):
    _, err = _lauf(dhrt_pfad, tmp_path, {
        "mathe.dh": MATHE,
        "main.dh": 'IMPORT "mathe.dh" AS mathe\nPRINT mathe.Gibtsnicht(1)\n',
    })
    assert "kennt keinen Namen" in err, err
    assert "main.dh:2" in err, err


def test_klasse_im_namensraum_meldet_klar(dhrt_pfad, tmp_path):
    """I.1 kann noch keine Typen -- die Meldung muss das SAGEN, nicht bloss
    behaupten, den Namen gebe es nicht."""
    _, err = _lauf(dhrt_pfad, tmp_path, {
        "k.dh": "CLASS Punkt\n    DIM x AS INTEGER\nEND CLASS\n",
        "main.dh": 'IMPORT "k.dh" AS k\nDIM p AS INTEGER\np = k.Punkt\n',
    })
    assert "WP I.2" in err, err
    # Die Meldung muss den Weg nennen, der HEUTE funktioniert -- nicht nur
    # den schwereren Ausweg ueber einen zweiten, flachen IMPORT.
    assert "LIEFERT" in err, err


def test_interner_name_leckt_nicht_in_meldungen(dhrt_pfad, tmp_path):
    """Der Nutzer darf `mathe@quadrat` nie zu sehen bekommen -- er hat den
    Namen nie geschrieben und findet ihn in seiner Datei nicht wieder."""
    _, err = _lauf(dhrt_pfad, tmp_path, {
        "mathe.dh": MATHE,
        "main.dh": 'IMPORT "mathe.dh" AS mathe\nPRINT mathe.Quadrat(1, 2, 3)\n',
    })
    assert "@" not in err, err
    assert "mathe.quadrat" in err, err


def test_lokale_variable_verschmilzt_nicht_mit_globaler(dhrt_pfad, tmp_path):
    """Ein lokales `DIM zaehler` in einer Modul-Funktion darf nicht auf das
    gleichnamige Top-Level-`DIM` desselben Moduls umgebogen werden -- sonst
    rechnet das Programm still falsch."""
    out, err = _lauf(dhrt_pfad, tmp_path, {
        "m.dh": ("DIM zaehler AS INTEGER\n"
                 "zaehler = 100\n"
                 "FUNCTION Zaehle() AS INTEGER\n"
                 "    DIM zaehler AS INTEGER\n"
                 "    zaehler = 7\n"
                 "    RETURN zaehler\n"
                 "END FUNCTION\n"
                 "FUNCTION Global() AS INTEGER\n"
                 "    RETURN zaehler\n"
                 "END FUNCTION\n"),
        "main.dh": ('IMPORT "m.dh" AS m\n'
                    "PRINT m.Zaehle()\n"
                    "PRINT m.Global()\n"),
    })
    assert out.split() == ["7", "100"], (out, err)


def test_ohne_as_bleibt_alles_flach(dhrt_pfad, tmp_path):
    """Bestandsschutz: ohne `AS` aendert sich nichts am bisherigen Verhalten."""
    out, err = _lauf(dhrt_pfad, tmp_path, {
        "mathe.dh": MATHE,
        "main.dh": 'IMPORT "mathe.dh"\nPRINT Quadrat(4)\nPRINT FAKTOR\n',
    })
    assert out.split() == ["16", "10"], (out, err)


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


def test_private_ist_innen_nutzbar(dhrt_pfad, tmp_path):
    """Privat heisst unsichtbar von aussen, nicht unbenutzbar: `Offen` ruft
    `Intern` und liest `GEHEIM`."""
    out, err = _lauf(dhrt_pfad, tmp_path, {
        "p.dh": PRIV,
        "main.dh": 'IMPORT "p.dh" AS p\nPRINT p.Offen(1)\n',
    })
    assert out.strip() == "43", (out, err)


def test_private_ist_von_aussen_gesperrt(dhrt_pfad, tmp_path):
    _, err = _lauf(dhrt_pfad, tmp_path, {
        "p.dh": PRIV,
        "main.dh": 'IMPORT "p.dh" AS p\nPRINT p.Intern(1)\n',
    })
    assert "ist PRIVATE" in err, err


def test_private_ohne_namensraum_stoert_nicht(dhrt_pfad, tmp_path):
    """`PRIVATE` in einer Datei ohne `AS` ist ein wirkungsloser Marker --
    es darf kein Fehler und keine Verhaltensaenderung daraus werden."""
    out, err = _lauf(dhrt_pfad, tmp_path, {
        "main.dh": ("PRIVATE FUNCTION F() AS INTEGER\n"
                    "    RETURN 5\n"
                    "END FUNCTION\n"
                    "PRINT F()\n"),
    })
    assert out.strip() == "5", (out, err)


def test_private_vor_etwas_anderem_meldet(dhrt_pfad, tmp_path):
    _, err = _lauf(dhrt_pfad, tmp_path, {
        "main.dh": "PRIVATE PRINT 1\n",
    })
    assert "PRIVATE steht vor" in err, err


# --- Abschottung gegen die Globals des Hauptprogramms --------------------

GLOBAL_LESER = ("FUNCTION LiestGlobal() AS INTEGER\n"
                "    RETURN punkte\n"
                "END FUNCTION\n")


def test_namensraum_sieht_globals_des_hauptprogramms_nicht(dhrt_pfad, tmp_path):
    """Der eigentliche Gewinn von `AS`: die Datei haengt nicht mehr davon ab,
    welche Globals das Hauptprogramm zufaellig hat."""
    _, err = _lauf(dhrt_pfad, tmp_path, {
        "g.dh": GLOBAL_LESER,
        "main.dh": ('IMPORT "g.dh" AS g\n'
                    "DIM punkte AS INTEGER\n"
                    "punkte = 5\n"
                    "PRINT g.LiestGlobal()\n"),
    })
    assert "kommt aus dem Hauptprogramm" in err, err
    assert "g.dh:2" in err, err


def test_ohne_as_bleibt_der_globale_zugriff_erlaubt(dhrt_pfad, tmp_path):
    """Gegenprobe und Bestandsschutz: dieselbe Datei ohne `AS` laeuft wie eh."""
    out, err = _lauf(dhrt_pfad, tmp_path, {
        "g.dh": GLOBAL_LESER,
        "main.dh": ('IMPORT "g.dh"\n'
                    "DIM punkte AS INTEGER\n"
                    "punkte = 5\n"
                    "PRINT LiestGlobal()\n"),
    })
    assert out.strip() == "5", (out, err)


def test_funktion_darf_einen_modul_typ_liefern(dhrt_pfad, tmp_path):
    """Gemessen beim Zuschnitt von I.2: den Typ BENENNEN geht nicht, ihn
    zurueckgeben schon -- samt Feldzugriff auf das Ergebnis.

    Damit kommt man ohne `DIM ... AS k.Punkt` aus, und genau darauf zeigt die
    Meldung jetzt. Vorher verwies sie nur auf den schwereren Ausweg, die Datei
    zusaetzlich flach zu importieren -- was den Namensraum wieder aufhebt.
    """
    out, err = _lauf(dhrt_pfad, tmp_path, {
        "k.dh": ("CLASS Punkt\n"
                 "    DIM x AS INTEGER\n"
                 "END CLASS\n"
                 "FUNCTION Neu(a AS INTEGER) AS Punkt\n"
                 "    DIM p AS Punkt\n"
                 "    p = NEW Punkt()\n"
                 "    p.x = a\n"
                 "    RETURN p\n"
                 "END FUNCTION\n"),
        "main.dh": 'IMPORT "k.dh" AS k\nPRINT k.Neu(3).x\n',
    })
    assert out.strip() == "3", (out, err)
