"""Aus Drachenhauch-Quelltext eine Referenz in Markdown erzeugen.

Der vierte Teil von Punkt 6 aus `docs/allzweck-audit-2.md`. Wer eine
Bibliothek schreibt (`zeitraum.dh`, `tabellen.dh`), hatte keinen Weg, sie zu
beschreiben, ohne alles ein zweites Mal von Hand zu tippen -- und eine von
Hand gepflegte Referenz driftet ab dem ersten Tag.

**Die Bausteine standen schon da.** `editor_qt/symbols.py` liest fuer den
Sprachserver ohnehin Definitionen und den Kommentarblock DIREKT ueber ihnen
(`scan_definitions`, `extract_user_doc`) -- also dieselbe Quelle, aus der
auch der Hover im Editor kommt. Dieses Modul setzt daraus nur Markdown
zusammen; es gibt keine zweite Vorstellung davon, was eine Signatur ist.

**Warum hier und nicht in `dhrt`:** der Rust-Lexer wirft Kommentare weg (er
braucht sie nicht), der Python-Lexer behaelt die Zeilen. Eine Doku ohne die
Kommentare waere eine Liste von Signaturen -- und die kann man auch selbst
lesen.

Aufruf: `dhrun.py --doku datei.dh [...] [-o referenz.md]`
"""
from __future__ import annotations

import re
from pathlib import Path

from .editor_qt.symbols import extract_user_doc, scan_definitions

# Was in eine Referenz gehoert -- und in welcher Reihenfolge die Abschnitte
# stehen. `param` und `dim` fehlen mit Absicht: das sind Innereien.
_ARTEN: list[tuple[str, str]] = [
    ("const", "Konstanten"),
    ("enum", "Aufzaehlungen"),
    ("struct", "Strukturen"),
    ("class", "Klassen"),
    ("function", "Funktionen"),
    ("sub", "Prozeduren"),
]

_PRIVAT = re.compile(r"^\s*PRIVATE\b", re.IGNORECASE)


def _ist_privat(quelle: str, zeile: int) -> bool:
    """`PRIVATE SUB` gehoert dem Modul, nicht seinen Nutzern.

    Seit WP I gibt es das Schluesselwort; eine Referenz, die private Namen
    auffuehrt, verspricht etwas, das beim naechsten Umbau verschwindet.
    """
    zeilen = quelle.split("\n")
    return 1 <= zeile <= len(zeilen) and bool(_PRIVAT.match(zeilen[zeile - 1]))


def datei_doku(pfad: Path) -> str:
    """Die Referenz EINER Datei als Markdown-Abschnitt."""
    quelle = pfad.read_text(encoding="utf-8")
    gesehen: set[tuple[str, str]] = set()
    nach_art: dict[str, list[tuple[str, str, str]]] = {}
    for d in scan_definitions(quelle):
        if d.kind not in {a for a, _ in _ARTEN}:
            continue
        if _ist_privat(quelle, d.line):
            continue
        schluessel = (d.kind, d.name.lower())
        if schluessel in gesehen:      # Ueberladungen gibt es nicht, aber
            continue                   # ein zweites DIM desselben Namens schon
        gesehen.add(schluessel)
        paar = extract_user_doc(quelle, d.name)
        if paar is None:
            continue
        sig, doc = paar
        nach_art.setdefault(d.kind, []).append((d.name, sig, doc))

    teile: list[str] = [f"## {pfad.name}\n"]
    # Ein Kommentarblock ganz oben in der Datei beschreibt die Datei selbst.
    kopf = _kopfkommentar(quelle)
    if kopf:
        teile.append(kopf + "\n")
    leer = True
    for art, ueberschrift in _ARTEN:
        eintraege = nach_art.get(art)
        if not eintraege:
            continue
        leer = False
        teile.append(f"### {ueberschrift}\n")
        for name, sig, doc in eintraege:
            teile.append(f"#### `{name}`\n")
            teile.append(f"```basic\n{sig}\n```\n")
            teile.append((doc.strip() or "*(nicht beschrieben)*") + "\n")
    if leer:
        teile.append("*(nichts Oeffentliches gefunden)*\n")
    return "\n".join(teile)


def _kopfkommentar(quelle: str) -> str:
    """Der Kommentarblock am Dateianfang -- die Beschreibung der Datei.

    Abgebrochen wird bei der ersten Zeile, die kein Kommentar ist; eine
    Leerzeile davor wird uebersprungen, damit ein `' Titel` nach einer
    Leerzeile noch zaehlt.
    """
    raus: list[str] = []
    for zeile in quelle.split("\n"):
        s = zeile.strip()
        if not s:
            if raus:
                break
            continue
        if s.startswith("'"):
            raus.append(s.lstrip("'").strip())
        elif s.upper().startswith("REM "):
            raus.append(s[4:].strip())
        else:
            break
    return "\n".join(raus).strip()


def erzeuge(pfade: list[Path], titel: str = "Referenz") -> str:
    """Die Referenz mehrerer Dateien als eine Markdown-Seite."""
    teile = [f"# {titel}\n",
             "*Erzeugt aus dem Quelltext -- nicht von Hand aendern.*\n"]
    for p in sorted(pfade):
        teile.append(datei_doku(p))
    return "\n".join(teile)
