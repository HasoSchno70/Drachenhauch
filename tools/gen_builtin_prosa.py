#!/usr/bin/env python
# -*- coding: utf-8 -*-
r"""Kurzbeschreibungen der Builtins aus `docs/` einsammeln.

    <venv>\python.exe tools\gen_builtin_prosa.py [--pruefen]

Schreibt `drachenhauch/editor_qt/builtin_prosa.json`. `--pruefen` schreibt
nichts, sondern meldet nur, ob die Datei noch zum Stand von `docs/` passt
(Rueckgabe 1 bei Abweichung) -- so haelt `tests/test_builtin_prosa.py` sie fest.

WARUM GENERIERT UND NICHT ZUR LAUFZEIT GELESEN
----------------------------------------------
Der Editor braucht die Texte fuer Hover und Signaturhilfe, und der Installer
packt `docs/*.md` NICHT mit (nur examples/, esp32/ und die Buecher, siehe
`installer/Drachenhauch.iss`). Im installierten Editor gaebe es also nichts zu
lesen. Die erzeugte JSON liegt dagegen im Python-Paket und wird von
`collect_data_files("drachenhauch")` automatisch eingepackt -- wie
`builtin_index.json`, dem sie nachgebaut ist.

WARUM UEBERHAUPT
----------------
`builtin_docs.BUILTIN_DOCS` ist von Hand gepflegt und deckte 328 von 1558
Builtins ab -- 21 %. Ganze Module standen bei null: gui (161 Befehle), g3d,
m3d, chart, json, sprite, tiled. Der Hover fiel dort auf die blosse Signatur
zurueck. Die Beschreibungen EXISTIEREN aber laengst, sie stehen in den
Modul-Dokumenten; sie ein zweites Mal von Hand zu tippen waere genau die Sorte
Kopie, die danach auseinanderlaeuft.

WAS NICHT AUSGEWERTET WIRD
--------------------------
Planungsdokumente (`entwurf-*`, `*roadmap*`, `*-design`, `allzweck-*`,
PERFORMANCE.md). Sie beschreiben, was sein SOLL -- teils ausdruecklich nicht
Umgesetztes. Ein Hover, der einen Entwurf zitiert, waere schlimmer als keiner.
Das ist keine Theorie: `befehlssatz-roadmap.md` lieferte fuer SCROLL einen
abgeschnittenen Satz ueber den Command-Buffer.
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

WURZEL = Path(__file__).resolve().parents[1]
DOCS = WURZEL / "docs"
ZIEL = WURZEL / "drachenhauch" / "editor_qt" / "builtin_prosa.json"

# Planungsdokumente -- siehe Kopfkommentar.
AUS = re.compile(r"^(entwurf-|allzweck-)|roadmap|-design\.md$|^PERFORMANCE\.md$")

NAME = r"([A-Z][A-Z0-9_]*\$?)"
# | `NAME(...)` | Beschreibung |            (auch dreispaltig: | sig | Typ | Text |)
TABELLE = re.compile(r"^\|\s*`" + NAME + r"[^`|]*`\s*\|(.+)$")
# Listenzeile. Der Kopf darf MEHRERE Namen tragen und die Signatur getrennt
# fuehren -- `rust-runtime.md` schreibt den ganzen 3D-Zweig so:
#     - `CUBE` / `CUBE_WIRES` `(x,y,z, w,h,d, farbe)` — gefuellter Quader.
# Beide Namen bekommen dann dieselbe Beschreibung. Als Trenner gilt nur der
# Gedankenstrich oder ein Doppelpunkt: ein schlichtes "-" kommt in Fliesstext
# zu haeufig vor, um damit Kopf und Beschreibung zu trennen.
LISTE = re.compile(r"^\s*[-*]\s+((?:`[^`]+`[\s/,]*)+)\s*[—–:]\s*(.+)$")
KOPF_NAME = re.compile(r"`\s*" + NAME + r"[^`]*`")

MAX_LAENGE = 400


def _quellen() -> list[Path]:
    """Referenzdokumente, beste zuerst: die Modul- und Befehlsreferenzen vor
    allem anderen -- ein Builtin wird dort beschrieben, wo es hingehoert."""
    alle = [f for f in sorted(DOCS.glob("*.md")) if not AUS.search(f.name)]
    vorn = [f for f in alle if f.name.startswith(("module-", "builtins-"))]
    return vorn + [f for f in alle if f not in vorn]


def _saeubern(roh: str) -> str:
    s = roh.strip().strip("|").strip()
    spalten = [t.strip() for t in s.split("|") if t.strip()]
    # Eine Spalte aus lauter Strichen ist ein LEERES Feld (die gui-Doku schreibt
    # "| sig | — | Text |", wenn es keinen Rueckgabewert gibt).
    spalten = [t for t in spalten if t.strip("—–-— ")]
    if len(spalten) > 1 and len(spalten[0]) <= 12:
        # Kurze erste Spalte ist der Rueckgabetyp ("BOOLEAN") -- als Praefix behalten.
        s = spalten[0] + " — " + " ".join(spalten[1:])
    else:
        s = " ".join(spalten)
    s = re.sub(r"\[([^\]]+)\]\([^)]*\)", r"\1", s)       # Links auf ihren Text
    s = re.sub(r"`([^`]*)`", r"\1", s)                    # Inline-Code
    s = re.sub(r"\*\*([^*]*)\*\*", r"\1", s)              # Fettdruck
    s = re.sub(r"\s+", " ", s).strip()
    # Ein Doppelpunkt am Ende kuendigt in der Doku das Codebeispiel an, das
    # gleich darunter steht -- im Tooltip steht es nicht, der Satz zeigte also
    # ins Leere.
    s = s.rstrip(":").rstrip()
    if len(s) > MAX_LAENGE:
        schnitt = s.rfind(" ", 0, MAX_LAENGE)
        s = s[:schnitt if schnitt > 0 else MAX_LAENGE].rstrip(" ,;-") + " …"
    return s


def _zeilen(datei: Path) -> list[str]:
    """Zeilen der Datei, Listeneintraege zu EINER Zeile zusammengezogen.

    Markdown bricht lange Listeneintraege um, und die Fortsetzung traegt die
    Haelfte des Satzes:

        - `RAY_HIT_BOX(ox,oy,oz, ...)` — AABB (Mittelpunkt c,
          Vollgroesse s) via `GetRayCollisionBox`.

    Ohne dieses Zusammenziehen stuende im Hover "AABB (Mittelpunkt c," -- ein
    abgeschnittener Satz ist schlimmer als keiner.
    """
    roh = datei.read_text(encoding="utf-8").splitlines()
    raus: list[str] = []
    for zeile in roh:
        rumpf = zeile.lstrip()
        # "**fett**" faengt auch mit * an -- ohne diese Unterscheidung galt eine
        # Fortsetzung, die mit einer Hervorhebung beginnt, als neuer
        # Listenpunkt, und der Satz brach davor ab.
        neuer_punkt = (rumpf.startswith(("- ", "* "))
                       or rumpf.startswith(("|", "#", "```")))
        fortsetzung = (raus and raus[-1].lstrip().startswith(("- ", "* "))
                       and zeile[:1] in (" ", "	") and rumpf and not neuer_punkt)
        if fortsetzung:
            raus[-1] = raus[-1].rstrip() + " " + zeile.strip()
        else:
            raus.append(zeile)
    return raus


def sammeln() -> dict[str, str]:
    namen = {b["name"].upper() for b in
             json.loads((WURZEL / "drachenhauch" / "editor_qt" / "builtin_index.json")
                        .read_text(encoding="utf-8"))["builtins"]}
    ohne_dollar = {n.rstrip("$") for n in namen}
    raus: dict[str, str] = {}
    for datei in _quellen():
        for zeile in _zeilen(datei):
            m = TABELLE.match(zeile)
            treffer = [m.group(1).upper()] if m else []
            if not m:
                m = LISTE.match(zeile)
                if m:
                    treffer = [x.upper() for x in KOPF_NAME.findall(m.group(1))]
            if not m:
                continue
            text = _saeubern(m.group(2))
            # Zu kurz ist keine Beschreibung, sondern ein Verweis oder eine
            # blosse Typangabe ("INTEGER").
            if len(text) < 8:
                continue
            for name in treffer:
                if name not in namen and name.rstrip("$") not in ohne_dollar:
                    continue
                raus.setdefault(name, text)
    return dict(sorted(raus.items()))


def _json_text(daten: dict[str, str]) -> str:
    kopf = ("Erzeugt aus docs/ von tools/gen_builtin_prosa.py -- NICHT von Hand "
            "aendern. Ausfuehrlichere Texte gehoeren in builtin_docs.py (die "
            "gewinnen), Korrekturen an einer Beschreibung in das jeweilige "
            "docs/module-*.md.")
    return json.dumps({"_comment": kopf, "count": len(daten), "docs": daten},
                      indent=1, ensure_ascii=False) + "\n"


def main() -> int:
    daten = sammeln()
    text = _json_text(daten)
    if "--pruefen" in sys.argv:
        alt = ZIEL.read_text(encoding="utf-8") if ZIEL.exists() else ""
        if alt == text:
            print(f"builtin_prosa.json ist aktuell ({len(daten)} Beschreibungen).")
            return 0
        print("builtin_prosa.json weicht von docs/ ab -- "
              "`python tools/gen_builtin_prosa.py` laufen lassen.")
        return 1
    ZIEL.write_text(text, encoding="utf-8")
    print(f"{len(daten)} Beschreibungen -> {ZIEL.relative_to(WURZEL).as_posix()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
