"""Die Signaturen im Referenzbuch gegen `builtin_index.json` halten.

Das Buch fuehrt jeden Befehl mit seiner Aufrufform (`H.cmd(name, signatur,
...)`). Diese Form altert still: kommt ein Argument dazu, steht sie weiter im
Buch, ohne dass irgendetwas anschlaegt -- beim Abgleich am 2026-08-30 fehlte
sechsmal ein spaeter ergaenztes Argument (`CAMERA_SET` die Drehung, vier
Dateibefehle die Kodierung bzw. das Muster), und einmal zeigte der Index eine
Stelle mehr an, als die Laufzeit liest.

Verglichen wird nur die ARGUMENTZAHL, nicht der Wortlaut: die Namen der
Argumente sind im Buch bewusst deutsch und erklaerend. Gelesen wird die
Signatur wie in `compiler.rs::parse_arity`, damit hier dasselbe gilt wie beim
`dhrt --check` des Nutzers.

Braucht Node (die Kapitel sind JavaScript-Module) -- ohne Node uebersprungen.
"""

from __future__ import annotations

import json
import re
import subprocess
from pathlib import Path

import pytest

from drachenhauch.editor_qt.dhrt_meta import builtin_index

WURZEL = Path(__file__).resolve().parents[1]
EXPORT = WURZEL / "tools" / "buch_sig_export.js"

# Ein Aufruf: Name, dann die Klammer samt Inhalt (eine Verschachtelungsebene
# reicht -- tiefer verschachtelt schreibt keine Signatur).
AUFRUF = re.compile(r"([A-Za-z_$][A-Za-z0-9_$]*)\s*\(([^()]*(?:\([^()]*\))?[^()]*)\)")


def _node_da() -> bool:
    try:
        return subprocess.run(["node", "--version"], capture_output=True, timeout=30).returncode == 0
    except (OSError, subprocess.SubprocessError):
        return False


def arity(sig: str) -> tuple[int, int] | None:
    """(min, max) Argumente aus einer Signatur; None = nicht sicher entscheidbar.

    Nachgebaut nach `parse_arity` in compiler.rs, plus die Buch-Eigenheit,
    mehrere Aufrufformen nebeneinander zu schreiben (`RND()   RND(n)`) -- die
    werden zur Spanne ueber alle Formen zusammengefasst.
    """
    sig = sig.strip().replace("→", "->").split("->")[0].strip()
    formen = [m.group(2) for m in AUFRUF.finditer(sig)]
    if not formen:
        return None
    spannen = []
    for inner in formen:
        inner = inner.strip()
        if not inner:
            spannen.append((0, 0))
            continue
        # "<3 Punkte>" ist die Buch-Kurzform fuer eine Gruppe von Argumenten,
        # "..." und "*args" heissen beliebig viele -- in allen drei Faellen
        # laesst sich die Zahl nicht ablesen, und Raten waere ein Falsch-Alarm.
        if "*args" in inner or "..." in inner or "<" in inner:
            return None  # bewusst offen gelassen
        bereich = re.match(r"\s*(\d+)\s*\.\.\s*(\d+)\s*Argument", inner)
        if bereich:
            spannen.append((int(bereich.group(1)), int(bereich.group(2))))
            continue
        genau = re.match(r"\s*(\d+)\s*Argument", inner)
        if genau:
            spannen.append((int(genau.group(1)), int(genau.group(1))))
            continue
        # Ab der ersten offenen eckigen Klammer ist alles optional; ein
        # Vorgabewert (`an: BOOLEAN = FALSE`) macht das Argument ebenfalls optional.
        schnitt = inner.find("[")
        kopf = inner if schnitt < 0 else inner[:schnitt]
        rest = "" if schnitt < 0 else inner[schnitt:]
        stuecke = [s for s in kopf.split(",") if s.strip()]
        pflicht = len([s for s in stuecke if "=" not in s])
        opt = len(stuecke) - pflicht + len(re.findall(r"[\[,]\s*[A-Za-z_]", rest))
        spannen.append((pflicht, pflicht + opt))
    return (min(s[0] for s in spannen), max(s[1] for s in spannen))


def formen_je_befehl(sig: str, bekannt: set[str]) -> dict[str, str]:
    """Aus einer Signaturzeile: welcher Befehl steht dort in welcher Form?

    Das Buch fasst verwandte Befehle in EINEM Eintrag zusammen und schreibt
    ihre Formen nebeneinander:

        H.cmd("TEMPDIR$ / TEMPFILE$",
              "TEMPDIR$()   TEMPFILE$([praefix$[, endung$]])", ...)

    Ohne diese Zerlegung blieben solche Eintraege ungeprueft -- und genau
    darin steckte am 2026-08-30 ein Fehler: TEMPFILE$ war mit der Endung als
    erstem Argument beschrieben, in Wahrheit ist das ein Vorsatz fuer den
    Namen. Gefunden hat ihn erst ein Lauf gegen dhrt.

    Zwei Faelle, die KEINE Abweichung sind:
    * Mehrere Formen desselben Befehls (`RANGE(ende)   RANGE(start, ende)`)
      sind Alternativen -- sie werden zu einer Spanne zusammengefasst.
    * Ein Name direkt hinter einem Schraegstrich ist die Kurzform einer
      Familie (`ECS_FILL_FLOAT/INT(...)`); dort steht nicht der ganze Name.
    """
    zusammen: dict[str, list[str]] = {}
    for m in AUFRUF.finditer(sig):
        if m.start(1) > 0 and sig[m.start(1) - 1] == "/":
            continue
        name = m.group(1).upper()
        if name in bekannt:
            zusammen.setdefault(name, []).append(f"{m.group(1)}({m.group(2)})")
    return {n: "   ".join(fs) for n, fs in zusammen.items()}


@pytest.fixture(scope="module")
def buch_eintraege() -> list[dict]:
    if not _node_da():
        pytest.skip("node nicht verfuegbar")
    roh = subprocess.run(
        ["node", str(EXPORT)],
        capture_output=True, text=True, encoding="utf-8", cwd=WURZEL, timeout=180,
    )
    if roh.returncode != 0:
        pytest.skip(f"Buch-Export fehlgeschlagen: {roh.stderr[:200]}")
    eintraege = json.loads(roh.stdout)
    # Ohne diese Schranke liefe der Vergleich unten bei einem kaputten Export
    # leer durch und meldete Erfolg -- der teuerste Fehler, den ein
    # Drift-Test machen kann.
    assert len(eintraege) > 500, f"Buch-Export lieferte nur {len(eintraege)} Eintraege"
    return eintraege


def test_arity_liest_die_bekannten_formen() -> None:
    """Der Leser selbst -- sonst faellt eine Regression hier stumm aus."""
    assert arity("MID$(s$, start[, n])") == (2, 3)
    assert arity("ARC(6..8 Argumente)") == (6, 8)
    assert arity("GUI_SPLITTER(7 Argumente)") == (7, 7)
    assert arity("GUI_TOGGLE(win, text: STRING, x, y, an: BOOLEAN = FALSE)") == (4, 5)
    assert arity("RND()        RND(n)") == (0, 1)
    assert arity("SCREENWIDTH()") == (0, 0)
    assert arity("PLOTS(xs, ys, farbe, ...)") is None


def test_zerlegung_trennt_sammel_eintraege() -> None:
    """Auch der Zerleger braucht eigene Proben -- er entscheidet, was geprueft wird."""
    bekannt = {"TEMPDIR$", "TEMPFILE$", "RANGE", "ECS_FILL_FLOAT"}
    z = formen_je_befehl("TEMPDIR$()   TEMPFILE$([praefix$[, endung$]])", bekannt)
    assert z == {"TEMPDIR$": "TEMPDIR$()", "TEMPFILE$": "TEMPFILE$([praefix$[, endung$]])"}
    # Alternativen desselben Befehls kommen zusammen und ergeben eine Spanne.
    assert arity(formen_je_befehl("RANGE(ende)   RANGE(start, ende)", bekannt)["RANGE"]) == (1, 2)
    # Die Kurzform hinter dem Schraegstrich zaehlt nicht als eigener Befehl.
    # Eine zusammengezogene Familie wird GAR NICHT geprueft: vor der Klammer
    # steht nur das letzte Stueck des Namens, und das ist kein Befehl.
    assert formen_je_befehl("ECS_FILL_FLOAT/INT(w, ziel$, wert)", bekannt) == {}


def test_buch_signaturen_passen_zum_index(buch_eintraege: list[dict]) -> None:
    idx = {b["name"].upper(): b for b in builtin_index()}
    bekannt = set(idx)
    befund = []
    for e in buch_eintraege:
        if not e["sig"].strip():
            continue
        for name, form in formen_je_befehl(e["sig"], bekannt).items():
            im_buch = arity(form)
            im_index = arity(idx[name].get("signature", ""))
            if im_buch is None or im_index is None or im_buch == im_index:
                continue
            befund.append(
                f"{name} ({e['datei']}): Buch {im_buch} -- Index {im_index}\n"
                f"    Buch : {form.strip()}\n"
                f"    Index: {idx[name]['signature']}"
            )
    assert not befund, "Signatur weicht ab:\n" + "\n".join(befund)
