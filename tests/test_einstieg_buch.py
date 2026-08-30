"""Das Einstiegsbuch: Kapitelnummern, Querverweise, Bausteine.

Anders als das Referenzbuch ist der Einstieg ein LINEARER Kurs und verweist an
134 Stellen auf "Kapitel 12" und aehnliche Zahlen. Bis zum Gegenlesen am
2026-08-30 druckte er die Nummern nirgends -- weder in den Ueberschriften noch
im Inhaltsverzeichnis stand mehr als der Titel. Alle 134 Verweise zeigten also
auf etwas, das der Leser nicht finden konnte.

Seither nummeriert `build_book.js` die Kapitel. Die Nummer entsteht dort durch
ZAEHLEN, die Verweise meinen von Hand die DATEINUMMER -- beide muessen
uebereinstimmen. Sonst verschiebt ein spaeter eingeschobenes Kapitel alle
Verweise auf einmal, und zwar lautlos: `Kapitel 12` bleibt lesbar, zeigt aber
aufs falsche.

Vier Verweise waren dabei schon falsch: DIM wurde nach Kapitel 3 verwiesen
(gehoert nach 2), FOR ... NEXT nach 4 (gehoert nach 3), und zweimal wurden
eigene Befehle in Kapitel 12 verortet statt in 13.

Braucht Node (die Kapitel sind JavaScript-Module) -- ohne Node uebersprungen.
"""

from __future__ import annotations

import json
import re
import subprocess
from pathlib import Path

import pytest

WURZEL = Path(__file__).resolve().parents[1]
EXPORT = WURZEL / "tools" / "einstieg_kapitel_export.js"
CONTENT = WURZEL / "buch-einstieg" / "buch" / "content"


def verweise() -> list[tuple[str, int]]:
    """Alle `Kapitel N` aus dem ROHTEXT der Kapiteldateien.

    Nicht ueber die H-Bausteine: Verweise stehen auch in Bildunterschriften
    und Tabellenzellen, und ein Sammler, der die uebersieht, prueft weniger
    als er verspricht -- beim ersten Anlauf fand er 93 von 134.
    """
    raus = []
    for p in sorted(CONTENT.glob("*.js")):
        for m in re.finditer(r"Kapitel (\d+)", p.read_text(encoding="utf-8")):
            raus.append((p.name, int(m.group(1))))
    return raus


def _node_da() -> bool:
    try:
        return subprocess.run(["node", "--version"], capture_output=True, timeout=30).returncode == 0
    except (OSError, subprocess.SubprocessError):
        return False


@pytest.fixture(scope="module")
def buch() -> dict:
    if not _node_da():
        pytest.skip("node nicht verfuegbar")
    roh = subprocess.run(
        ["node", str(EXPORT)],
        capture_output=True, text=True, encoding="utf-8", cwd=WURZEL, timeout=180,
    )
    if roh.returncode != 0:
        pytest.skip(f"Export fehlgeschlagen: {roh.stderr[:200]}")
    d = json.loads(roh.stdout)
    # Ohne diese Schranken liefen die Pruefungen bei einem kaputten Export leer
    # durch und meldeten Erfolg.
    assert len(d["kapitel"]) > 30, f"nur {len(d['kapitel'])} Kapitel gefunden"
    return d


def test_gedruckte_nummer_ist_die_dateinummer(buch: dict) -> None:
    """Sonst verschiebt ein eingeschobenes Kapitel alle Verweise auf einmal."""
    schief = []
    for k in buch["kapitel"]:
        if k["nr"] is None:
            continue
        m = re.match(r"(\d+)_", k["datei"])
        assert m, k["datei"]
        if int(m.group(1)) != k["nr"]:
            schief.append(f"  {k['datei']}: gedruckt als Kapitel {k['nr']}")
    assert not schief, (
        "Die gedruckte Kapitelnummer weicht von der Dateinummer ab. Die "
        "Verweise im Text meinen die Dateinummer:\n" + "\n".join(schief)
    )


def test_jeder_verweis_trifft_ein_kapitel(buch: dict) -> None:
    alle = verweise()
    assert len(alle) > 100, f"nur {len(alle)} Verweise gefunden"
    hoechste = buch["hoechste"]
    tot = sorted({(d, n) for d, n in alle if n < 1 or n > hoechste})
    assert not tot, (
        f"Verweis auf ein Kapitel ausserhalb 1..{hoechste}:\n"
        + "\n".join(f"  {d}: Kapitel {n}" for d, n in tot)
    )


def test_bausteine_sind_befuellt(buch: dict) -> None:
    """H.tip(TITEL, text) -- mit nur einem Argument wird der Text zur Ueberschrift."""
    fehlt = buch["tips_ohne_rumpf"] + buch["cmds_ohne_text"]
    assert not fehlt, "Baustein ohne Rumpf bzw. ohne Beschreibung:\n" + "\n".join(
        f"  {e['datei']}: {e.get('titel') or e.get('name')}" for e in fehlt
    )

# --------------------------------------------------------- Code im Buch
# Drei Werkzeuge im Buchverzeichnis erledigen die Arbeit; hier laufen sie mit.
# Bis 2026-08-30 rief sie WEDER die Suite NOCH die CI auf -- sie liefen nur,
# wenn jemand daran dachte.

BUCH = WURZEL / "buch-einstieg" / "buch"
DHRT = WURZEL / "rust" / "drachenhauch_runtime" / "target" / "release" / "dhrt.exe"
DHRT_POSIX = DHRT.with_suffix("")


def _lauf(werkzeug: str) -> subprocess.CompletedProcess:
    if not _node_da():
        pytest.skip("node nicht verfuegbar")
    if not (DHRT.exists() or DHRT_POSIX.exists()):
        pytest.skip("dhrt nicht gebaut")
    return subprocess.run(
        ["node", str(BUCH / werkzeug)],
        capture_output=True, text=True, encoding="utf-8", cwd=BUCH, timeout=1800,
    )


def test_alle_codebloecke_uebersetzen() -> None:
    r = _lauf("pruef_codebloecke.js")
    assert r.returncode == 0, r.stdout + r.stderr
    zahl = int(r.stdout.split("Codebloecke")[0].strip().split()[-1])
    assert zahl > 150, f"nur {zahl} Codebloecke gefunden"


def test_jeder_abdruck_ist_die_datei() -> None:
    """Wer abtippt und danach in code/kapNN/ sieht, soll dasselbe vorfinden."""
    r = _lauf("pruef_abdruck.js")
    assert r.returncode == 0, r.stdout + r.stderr
    # Die Schlusszeile lautet "N Abdrucke geprueft, 0 weichen ab." -- davor
    # steht je eine ok-Zeile, also nicht die erste Zeile nehmen.
    zahl = int(r.stdout.split("Abdrucke")[0].strip().split()[-1])
    assert zahl > 10, f"nur {zahl} Abdrucke geprueft"


def test_die_behaupteten_ausgaben_stimmen() -> None:
    r = _lauf("pruef_ausgaben.js")
    assert r.returncode == 0, r.stdout + r.stderr
    # Die Schranke faengt den LEEREN Lauf. Die meisten Beispiele hier oeffnen
    # ein Fenster und lassen sich nicht ausfuehren; uebrig bleiben die
    # Konsolen-Beispiele.
    zahl = int(r.stdout.split("ausgefuehrt")[0].strip().split()[-1])
    assert zahl >= 5, f"nur {zahl} Ausgabe-Bloecke ausgefuehrt -- laeuft dhrt?"

def test_die_zahlen_ueber_die_anderen_baende_stimmen() -> None:
    """Anhang C nennt die Kapitelzahlen der Nachbarbaende.

    Die stehen dort von Hand -- und veralten, sobald einer der Baende waechst.
    Genau das war am 2026-08-30 passiert: Das Lehrbuch stand mit 75 Kapiteln
    da, obwohl es inzwischen 84 hat (die sieben neuen Modul-Kapitel plus die
    vier fuer die Kern-Befehle). Galaga stand mit 12 statt 13.
    """
    anhang = (WURZEL / "buch-einstieg" / "buch" / "content" / "36_anhang_c_weiter.js").read_text(encoding="utf-8")

    lehrbuch = len(list((WURZEL / "buch-referenz" / "buch" / "content").glob("*.js")))
    # Vorwort und die vier Anhaenge zaehlen im Lehrbuch als Kapitel mit -- die
    # Angabe im Text meint alle Kapiteldateien.
    galaga = len(re.findall(
        r'chapter\("Kapitel \d+:',
        (WURZEL / "buch-galaga" / "buch" / "build_book.js").read_text(encoding="utf-8")))
    tippspiel = len(re.findall(
        r'chapter\("Kapitel \d+:',
        (WURZEL / "buch-tippspiel" / "buch" / "build_book.js").read_text(encoding="utf-8")))

    schief = []
    for name, ist, muster in [
        ("Lehrbuch", lehrbuch, r"die ganze Sprache, (\d+) Kapitel"),
        ("Galaga", galaga, r"Arcade-Shooter in (\d+) Kapiteln"),
        ("Tippspiel", tippspiel, r"Rangliste in (\d+) Kapiteln"),
    ]:
        m = re.search(muster, anhang)
        assert m, f"Angabe zu {name} nicht mehr gefunden -- Muster anpassen"
        if int(m.group(1)) != ist:
            schief.append(f"  {name}: Anhang C nennt {m.group(1)}, gezaehlt sind {ist}")
    assert not schief, "Kapitelzahl eines Nachbarbands veraltet:\n" + "\n".join(schief)
