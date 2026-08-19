"""MAP: Reihenfolge bleibt, auch mit dem Hash-Index daneben (WP J).

`GbMap` war eine lineare Liste -- gemessen kostete das Fuellen einer Map mit
20 000 Eintraegen 224 ms und das Lesen 187 ms, sauber quadratisch. Seit dem
Hash-Index sind es 8 bzw. 7 ms.

Der Index bringt eine Invariante mit, die vorher nicht existieren konnte:
`eintraege` haelt die Reihenfolge, `index` bildet Key -> Position ab, und beide
muessen zusammenpassen. Am gefaehrlichsten ist `MAPREMOVE` -- danach
verschieben sich alle nachfolgenden Positionen. Genau das pruefen diese Tests.
"""


def _map(inhalt: str) -> str:
    return "DIM m AS MAP OF INTEGER\n" + inhalt


def test_reihenfolge_ist_einfuegereihenfolge(run_gb):
    out = run_gb(_map(
        'MAPPUT(m, "zebra", 1)\n'
        'MAPPUT(m, "anton", 2)\n'
        'MAPPUT(m, "mitte", 3)\n'
        'PRINT JOIN$(MAPKEYS(m), ",")\n'))
    assert out.strip() == "zebra,anton,mitte"


def test_ueberschreiben_behaelt_die_position(run_gb):
    """Ein zweites MAPPUT auf denselben Key ist eine Aenderung, kein Anhaengen."""
    out = run_gb(_map(
        'MAPPUT(m, "a", 1)\n'
        'MAPPUT(m, "b", 2)\n'
        'MAPPUT(m, "a", 99)\n'
        'PRINT JOIN$(MAPKEYS(m), ",")\n'
        'PRINT MAPGET(m, "a")\n'
        'PRINT MAPSIZE(m)\n'))
    assert out.split("\n")[:3] == ["a,b", "99", "2"]


def test_entfernen_in_der_mitte(run_gb):
    """Der heikle Fall: nach dem Loeschen verschieben sich alle Positionen
    dahinter. Zeigt der Index dann noch richtig?"""
    out = run_gb(_map(
        'MAPPUT(m, "a", 1)\n'
        'MAPPUT(m, "b", 2)\n'
        'MAPPUT(m, "c", 3)\n'
        'MAPPUT(m, "d", 4)\n'
        'MAPREMOVE(m, "b")\n'
        'PRINT JOIN$(MAPKEYS(m), ",")\n'
        'PRINT STR$(MAPGET(m, "a")) + "," + STR$(MAPGET(m, "c")) + "," + STR$(MAPGET(m, "d"))\n'))
    assert out.split("\n")[:2] == ["a,c,d", "1,3,4"]


def test_entfernen_und_wieder_einfuegen(run_gb):
    """Nach dem Entfernen muss der Key als NEU gelten -- also hinten landen,
    nicht auf seiner alten Position."""
    out = run_gb(_map(
        'MAPPUT(m, "a", 1)\n'
        'MAPPUT(m, "b", 2)\n'
        'MAPREMOVE(m, "a")\n'
        'MAPPUT(m, "a", 7)\n'
        'PRINT JOIN$(MAPKEYS(m), ",")\n'
        'PRINT MAPGET(m, "a")\n'))
    assert out.split("\n")[:2] == ["b,a", "7"]


def test_mehrfaches_entfernen(run_gb):
    out = run_gb(_map(
        'DIM i AS INTEGER\n'
        'FOR i = 1 TO 10\n'
        '    MAPPUT(m, "k" + STR$(i), i)\n'
        'NEXT\n'
        'MAPREMOVE(m, "k1")\n'
        'MAPREMOVE(m, "k5")\n'
        'MAPREMOVE(m, "k10")\n'
        'PRINT JOIN$(MAPKEYS(m), ",")\n'
        'PRINT STR$(MAPGET(m, "k9")) + "|" + STR$(MAPSIZE(m))\n'))
    assert out.split("\n")[:2] == ["k2,k3,k4,k6,k7,k8,k9", "9|7"]


def test_entfernen_eines_unbekannten_keys(run_gb):
    out = run_gb(_map(
        'MAPPUT(m, "a", 1)\n'
        'PRINT STR$(MAPREMOVE(m, "gibtsnicht"))\n'
        'PRINT JOIN$(MAPKEYS(m), ",")\n'))
    assert out.split("\n")[:2] == ["FALSE", "a"]


def test_leeren_und_neu_fuellen(run_gb):
    """MAPCLEAR griff frueher direkt auf die Liste zu -- mit einem Index
    daneben waere genau das die Stelle, an der beide auseinanderlaufen."""
    out = run_gb(_map(
        'MAPPUT(m, "a", 1)\n'
        'MAPPUT(m, "b", 2)\n'
        'MAPCLEAR(m)\n'
        'PRINT MAPSIZE(m)\n'
        'MAPPUT(m, "c", 3)\n'
        'PRINT JOIN$(MAPKEYS(m), ",")\n'
        'PRINT MAPGET(m, "c")\n'
        'PRINT STR$(MAPHAS(m, "a"))\n'))
    assert out.split("\n")[:4] == ["0", "c", "3", "FALSE"]


def test_werte_und_items_folgen_derselben_reihenfolge(run_gb):
    out = run_gb(_map(
        'MAPPUT(m, "z", 26)\n'
        'MAPPUT(m, "a", 1)\n'
        'PRINT JOIN$(MAPKEYS(m), ",")\n'
        'PRINT STR$(MAPVALUES(m)[0]) + "," + STR$(MAPVALUES(m)[1])\n'))
    assert out.split("\n")[:2] == ["z,a", "26,1"]


def test_viele_eintraege_bleiben_korrekt(run_gb):
    """Der Fall, der die Beschleunigung ueberhaupt noetig machte -- und der
    auch beweist, dass der Index bei Groesse nicht durcheinandergeraet."""
    out = run_gb(_map(
        'DIM i AS INTEGER\n'
        'DIM summe AS INTEGER\n'
        'FOR i = 1 TO 3000\n'
        '    MAPPUT(m, "k" + STR$(i), i)\n'
        'NEXT\n'
        'summe = 0\n'
        'FOR i = 1 TO 3000\n'
        '    summe = summe + MAPGET(m, "k" + STR$(i))\n'
        'NEXT\n'
        'PRINT summe\n'
        'PRINT MAPSIZE(m)\n'
        'PRINT MAPKEYS(m)[0] + "," + MAPKEYS(m)[2999]\n'))
    assert out.split("\n")[:3] == ["4501500", "3000", "k1,k3000"]
