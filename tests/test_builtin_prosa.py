"""Kurzbeschreibungen der Builtins, erzeugt aus `docs/`.

`builtin_docs.json` (die handgepflegte Tabelle) deckte 328 von 1558 Builtins
ab -- 21 %. Ganze Module standen bei null: gui (161 Befehle), g3d, m3d,
chart, json, sprite, tiled. Hover und Signaturhilfe fielen dort auf die
blosse Signatur zurueck, obwohl die Beschreibungen laengst in den
Modul-Dokumenten stehen.

`dhrt doku prosa` sammelt sie ein und schreibt
`drachenhauch/editor_qt/builtin_prosa.json` (bis 2026-09-06 tat das
`tools/gen_builtin_prosa.py` in Python -- Weg A aus
docs/entwurf-python-abbau.md). Erzeugt statt zur Laufzeit gelesen, weil der
Installer `docs/*.md` nicht mitpackt und dhrt die Datei fuer `dhrt lsp`
einbettet.

Diese Tests halten dreierlei fest: dass die Datei zum Stand von `docs/` passt,
dass die Texte brauchbar sind (kein Markdown-Schrott, keine abgeschnittenen
Saetze), und dass die handgepflegte Tabelle weiterhin gewinnt.
"""
from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path

import pytest

from drachenhauch.editor_qt.builtin_docs import BUILTIN_DOCS, get_doc
from drachenhauch.editor_qt.dhrt_meta import builtin_index

WURZEL = Path(__file__).resolve().parents[1]
PROSA = WURZEL / "drachenhauch" / "editor_qt" / "builtin_prosa.json"


def _dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    for v in ("release", "debug"):
        p = WURZEL / "rust" / "drachenhauch_runtime" / "target" / v / exe
        if p.exists():
            return p
    return None


def _prosa() -> dict[str, str]:
    return json.loads(PROSA.read_text(encoding="utf-8"))["docs"]


def _node_da() -> bool:
    try:
        return subprocess.run(["node", "--version"], capture_output=True,
                              timeout=30).returncode == 0
    except (OSError, subprocess.SubprocessError):
        return False


@pytest.mark.skipif(not _node_da(), reason="ohne Node ist buch-referenz nicht lesbar")
def test_datei_passt_zum_stand_von_docs():
    """Der Generator im Pruefmodus. Wer eine Beschreibung in `docs/` oder im
    Referenzbuch aendert, aendert damit auch den Hover -- die erzeugte Datei
    muss also mitkommen."""
    dhrt = _dhrt()
    if dhrt is None:
        pytest.skip("native Runtime 'dhrt' nicht gebaut")
    r = subprocess.run([str(dhrt), "doku", "prosa", "--pruefen"], capture_output=True, text=True,
                       encoding="utf-8", cwd=str(WURZEL), timeout=120)
    assert r.returncode == 0, r.stdout + r.stderr


def test_deckt_die_grossen_module_ab():
    """Je ein Vertreter der Module, die vorher komplett ohne Doku waren."""
    prosa = _prosa()
    for name in ("GUI_TABLE_SET_CELL", "CUBE", "CHART_HOVER_VALUE",
                 "VEC2_LERP", "SPRITE_PLAY", "TILED_LOAD"):
        assert name in prosa, name


def test_nur_echte_builtins():
    namen = {b["name"].upper() for b in builtin_index()}
    namen |= {n.rstrip("$") for n in namen}
    fremd = [n for n in _prosa() if n not in namen]
    assert not fremd, fremd[:10]


def test_texte_sind_sauber():
    """Kein Markdown-Rest im Tooltip -- Backticks, Pipes und Link-Klammern
    haben in einer Kurzbeschreibung nichts zu suchen."""
    schrott = {n: t for n, t in _prosa().items()
               if any(z in t for z in ("`", "|", "](", "**"))}
    assert not schrott, list(schrott.items())[:5]


def test_keine_abgeschnittenen_saetze():
    """Ein Satz, der mitten im Wort endet, ist schlimmer als keiner. Gekuerzte
    Texte enden auf ein Auslassungszeichen und sind damit als gekuerzt
    erkennbar -- alles andere darf nicht auf einem Bindewort enden."""
    # " ein" steht bewusst NICHT dabei: im Deutschen ist es genauso oft die
    # Partikel eines trennbaren Verbs wie ein Artikel -- "sammelt aber die
    # Ausgabe ein" (SHELL_OUT$) ist ein vollstaendiger Satz. Der Test meldete
    # ihn prompt als abgeschnitten.
    offen = {n: t for n, t in _prosa().items()
             if not t.endswith("…")
             and t.rstrip().endswith((",", ":", " und", " oder", " der", " die",
                                      " das", " eine", " via", " mit"))}
    assert not offen, list(offen.items())[:5]


def test_handgepflegte_tabelle_gewinnt():
    """Die Texte aus `docs/` sind Tabellenzellen und oft knapp; die Tabelle in
    `builtin_docs.json` ist auf den Hover zugeschnitten und muss vorgehen."""
    gemeinsam = [n for n in _prosa() if n.lower() in BUILTIN_DOCS]
    assert gemeinsam, "kein ueberlappender Eintrag -- Test waere wirkungslos"
    name = gemeinsam[0]
    assert get_doc(name)[1] == BUILTIN_DOCS[name.lower()][1]


def test_abdeckung_bleibt_deutlich_ueber_dem_alten_stand():
    """Bremse gegen stilles Verschlechtern: vor dem Einsammeln lag die
    Abdeckung bei 21 %, jetzt bei rund 54 %. Faellt sie unter 45 %, hat
    entweder der Generator etwas verloren oder docs/ wurde umgebaut."""
    idx = builtin_index()
    mit = sum(1 for b in idx if get_doc(b["name"]) is not None)
    assert mit / len(idx) > 0.45, f"nur {mit}/{len(idx)}"
