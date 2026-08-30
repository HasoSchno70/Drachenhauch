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


def _node_da() -> bool:
    try:
        return subprocess.run(["node", "--version"], capture_output=True, timeout=30).returncode == 0
    except (OSError, subprocess.SubprocessError):
        return False


def arity(sig: str) -> tuple[int, int] | None:
    """(min, max) Argumente aus einer Signatur; None = nicht sicher entscheidbar.

    Nachgebaut nach `parse_arity` in compiler.rs, plus die Buch-Eigenheit,
    mehrere Aufrufformen in eine Zeile zu schreiben (`RND()   RND(n)`) -- die
    werden zur Spanne ueber alle Formen zusammengefasst.
    """
    sig = sig.strip().replace("→", "->").split("->")[0].strip()
    formen = re.findall(r"[A-Za-z_$][A-Za-z0-9_$]*\s*\(([^()]*)\)", sig)
    if not formen:
        return None
    spannen = []
    for inner in formen:
        inner = inner.strip()
        if not inner:
            spannen.append((0, 0))
            continue
        if "*args" in inner or "..." in inner:
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


def test_buch_signaturen_passen_zum_index(buch_eintraege: list[dict]) -> None:
    idx = {b["name"].upper(): b for b in builtin_index()}
    befund = []
    for e in buch_eintraege:
        name = e["name"].strip().strip("`").split("(")[0].strip().upper()
        if name not in idx or not e["sig"].strip():
            continue
        im_buch = arity(e["sig"])
        im_index = arity(idx[name].get("signature", ""))
        if im_buch is None or im_index is None or im_buch == im_index:
            continue
        befund.append(
            f"{name} ({e['datei']}): Buch {im_buch} -- Index {im_index}\n"
            f"    Buch : {e['sig'].strip()}\n"
            f"    Index: {idx[name]['signature']}"
        )
    assert not befund, "Signatur weicht ab:\n" + "\n".join(befund)
