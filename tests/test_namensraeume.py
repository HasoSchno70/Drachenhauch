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


def test_klassenname_allein_im_ausdruck_meldet(dhrt_pfad, tmp_path):
    """Seit I.2 ist `k.Punkt` als TYP gueltig. In Ausdrucks-Position bleibt er
    sinnlos -- gemeint ist fast immer `NEW k.Punkt()`, und genau darauf soll
    die Meldung zeigen."""
    _, err = _lauf(dhrt_pfad, tmp_path, {
        "k.dh": "CLASS Punkt" + NL + "    DIM x AS INTEGER" + NL + "END CLASS" + NL,
        "main.dh": 'IMPORT "k.dh" AS k' + NL + "DIM n AS INTEGER" + NL + "n = k.Punkt" + NL,
    })
    assert "ist eine Klasse" in err, err
    # Der Lexer schreibt Bezeichner klein -- die Meldung kennt die
    # urspruengliche Schreibweise gar nicht.
    assert "new k.punkt" in err.lower(), err

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


def test_dim_as_qualifizierter_typ(dhrt_pfad, tmp_path):
    out, err = _lauf(dhrt_pfad, tmp_path, {
        "k.dh": KLASSE,
        "main.dh": ('IMPORT "k.dh" AS k\n'
                    "DIM p AS k.Punkt\n"
                    "p = NEW k.Punkt()\n"
                    "p.x = 7\n"
                    "PRINT p.x\n"),
    })
    assert out.strip() == "7", (out, err)


def test_gleiche_klasse_in_beiden_dateien(dhrt_pfad, tmp_path):
    """Der Zweck der Uebung: zwei Klassen `Punkt`, nebeneinander."""
    out, err = _lauf(dhrt_pfad, tmp_path, {
        "k.dh": KLASSE,
        "main.dh": ('IMPORT "k.dh" AS k\n'
                    "CLASS Punkt\n"
                    "    DIM x AS INTEGER\n"
                    "END CLASS\n"
                    "DIM a AS Punkt\n"
                    "DIM b AS k.Punkt\n"
                    "a = NEW Punkt()\n"
                    "b = NEW k.Punkt()\n"
                    "a.x = 1\n"
                    "b.x = 2\n"
                    'PRINT STR$(a.x) + "," + STR$(b.x)\n'),
    })
    assert out.strip() == "1,2", (out, err)


def test_modul_nutzt_die_eigene_klasse_unqualifiziert(dhrt_pfad, tmp_path):
    """Innerhalb der Datei bleibt `Punkt` schlicht `Punkt` -- auch als Typ,
    als Rueckgabetyp und hinter `NEW`."""
    out, err = _lauf(dhrt_pfad, tmp_path, {
        "k.dh": KLASSE,
        "main.dh": 'IMPORT "k.dh" AS k\nPRINT k.Neu(5).x\n',
    })
    assert out.strip() == "5", (out, err)


def test_array_of_qualifiziertem_typ(dhrt_pfad, tmp_path):
    """`array:` und `map:` koennen den Typnamen schachteln -- der Aufloeser
    muss durch das Praefix hindurchgreifen."""
    out, err = _lauf(dhrt_pfad, tmp_path, {
        "k.dh": KLASSE,
        "main.dh": ('IMPORT "k.dh" AS k\n'
                    "DIM feld AS ARRAY OF k.Punkt\n"
                    "DIM p AS k.Punkt\n"
                    "p = NEW k.Punkt()\n"
                    "p.x = 9\n"
                    "PRINT p.x\n"),
    })
    assert out.strip() == "9", (out, err)


def test_unbekannte_klasse_meldet_klar(dhrt_pfad, tmp_path):
    _, err = _lauf(dhrt_pfad, tmp_path, {
        "k.dh": KLASSE,
        "main.dh": 'IMPORT "k.dh" AS k\nDIM p AS k.Gibtsnicht\n',
    })
    assert "kennt keine Klasse" in err, err


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


def test_enum_member_ueber_den_namensraum(dhrt_pfad, tmp_path):
    """`e.Farbe.BLAU` -- zwei Punkte hintereinander."""
    out, err = _lauf(dhrt_pfad, tmp_path, {
        "e.dh": ENUM_DATEI,
        "main.dh": 'IMPORT "e.dh" AS e\nPRINT e.Farbe.BLAU\n',
    })
    assert out.strip() == "2", (out, err)


def test_enum_als_typ(dhrt_pfad, tmp_path):
    """Ein ENUM ist als Typ ein INTEGER -- wie ein flaches ENUM auch."""
    out, err = _lauf(dhrt_pfad, tmp_path, {
        "e.dh": ENUM_DATEI,
        "main.dh": ('IMPORT "e.dh" AS e\n'
                    "DIM f AS e.Farbe\n"
                    "f = e.Farbe.GRUEN\n"
                    "PRINT f\n"),
    })
    assert out.strip() == "1", (out, err)


def test_modul_nutzt_sein_enum_unqualifiziert(dhrt_pfad, tmp_path):
    """Der Fall, der beim Bauen zuerst brach: `Farbe.ROT` INNERHALB der Datei
    zeigte nach dem Umbenennen ins Leere, und die VM suchte eine Variable
    namens `farbe`."""
    out, err = _lauf(dhrt_pfad, tmp_path, {
        "e.dh": ENUM_DATEI,
        "main.dh": 'IMPORT "e.dh" AS e\nPRINT e.Name$(e.Farbe.ROT)\n',
    })
    assert out.strip() == "rot", (out, err)


def test_gleiches_enum_in_beiden_dateien(dhrt_pfad, tmp_path):
    """Der Zweck: zwei ENUMs `Farbe` mit verschiedener Bedeutung."""
    out, err = _lauf(dhrt_pfad, tmp_path, {
        "e.dh": ENUM_DATEI,
        "main.dh": ('IMPORT "e.dh" AS e\n'
                    "ENUM Farbe\n"
                    "    BLAU\n"
                    "    ROT\n"
                    "END ENUM\n"
                    'PRINT STR$(Farbe.ROT) + "," + STR$(e.Farbe.ROT)\n'),
    })
    assert out.strip() == "1,0", (out, err)


def test_unbekanntes_enum_meldet(dhrt_pfad, tmp_path):
    _, err = _lauf(dhrt_pfad, tmp_path, {
        "e.dh": ENUM_DATEI,
        "main.dh": 'IMPORT "e.dh" AS e\nDIM f AS e.Gibtsnicht\n',
    })
    assert "kennt keine Klasse" in err, err
