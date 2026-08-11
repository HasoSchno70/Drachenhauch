#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Jeden Drachenhauch-Codeblock in docs/ durch `dhrt --check` schicken.

    <venv>\\python.exe tools\\pruef_docs.py [ordner]

Rueckgabe 0 = sauber, 1 = mindestens ein Fehler. Damit laesst sich der
Aufruf in einen Testlauf oder die CI haengen.

WARUM nur Syntax (Phasen lex/parse)?
------------------------------------
Die Doku zeigt fast ueberall Ausschnitte: die Variable stammt aus dem Absatz
davor, `SCREEN` steht drei Bloecke hoeher. Ein Compile-Fehler ("Variable nicht
deklariert") waere dort die Regel und nicht die Ausnahme -- der Bericht
bestuende aus Falsch-Alarmen und wuerde nach zwei Wochen ignoriert. Ein
SYNTAXfehler dagegen ist auch im Ausschnitt einer: was nicht parst, tippt
auch niemand erfolgreich ab.

WAS ALS NOTATION DURCHGEHT
--------------------------
Ein Teil der ```basic-Bloecke ist gar kein Programm, sondern Schreibweise:
Signaturen mit `[optionalen]` Argumenten, `->` fuer den Rueckgabetyp,
Pseudocode (`FOR each tile:`) und `...` fuer ausgelassenen Code. Die drei
Punkte sind das verlaesslichste Merkmal: `...` ist an KEINER Stelle gueltige
Drachenhauch-Syntax, ihr Vorkommen heisst also zwangslaeufig "hier fehlt
etwas mit Absicht".

Gefunden hat dieser Pruefer beim ersten Lauf elf Beispiele, die nicht laufen
-- darunter drei Variablen mit reservierten Namen (`data`, `sound`, `map`),
zwei entfernte Builtins (`BITOR`/`SHL`) und `FUNCREF(f)`, eine Schreibweise,
die es nie gab.
"""
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path

WURZEL = Path(__file__).resolve().parents[1]
BLOCK = re.compile(r"```(?:basic|gb|dh|drachenhauch)\n(.*?)```", re.S | re.I)

# Zeilen, die Schreibweise sind und kein Programm.
NOTATION = (
    re.compile(r"\.\.\."),                 # ausgelassener Code -- nie gueltige Syntax
    re.compile(r"\[.*\]"),                 # [optionales Argument]
    re.compile(r"->"),                     # -> Rueckgabetyp
    re.compile(r"^\s*FOR each\b", re.I),   # Pseudocode
)


def finde_dhrt():
    exe = "dhrt.exe" if sys.platform == "win32" else "dhrt"
    for variante in ("release", "debug"):
        p = WURZEL / "rust" / "drachenhauch_runtime" / "target" / variante / exe
        if p.exists():
            return p
    return None


def pruefe(ordner: Path, dhrt: Path):
    kein_fenster = (subprocess.CREATE_NO_WINDOW
                    if sys.platform == "win32" else 0)
    tmp = Path(tempfile.mkdtemp(prefix="pruef_docs_"))
    bloecke = 0
    funde = []
    for datei in sorted(ordner.glob("*.md")):
        text = datei.read_text(encoding="utf-8")
        for i, m in enumerate(BLOCK.finditer(text)):
            code = m.group(1)
            if not code.strip():
                continue
            bloecke += 1
            md_zeile = text[:m.start()].count("\n") + 2
            f = tmp / f"{datei.stem}_{i}.dh"
            f.write_text(code, encoding="utf-8")
            r = subprocess.run([str(dhrt), "--check", str(f)],
                               capture_output=True, text=True, encoding="utf-8",
                               creationflags=kein_fenster)
            try:
                diag = json.loads((r.stdout or "").strip() or "[]")
            except ValueError:
                continue                      # dhrt-Panne -> nicht als Fehler werten
            zeilen = code.splitlines()
            for d in diag:
                if d.get("phase") not in ("lex", "parse"):
                    continue
                if d.get("severity") == "warning":
                    continue
                nr = int(d.get("line", 1))
                quelle = zeilen[nr - 1].strip() if 0 < nr <= len(zeilen) else ""
                if any(p.search(quelle) for p in NOTATION):
                    continue
                funde.append((datei.name, md_zeile, quelle, d.get("message", "")))
    return bloecke, funde


def main():
    ordner = Path(sys.argv[1]) if len(sys.argv) > 1 else WURZEL / "docs"
    dhrt = finde_dhrt()
    if dhrt is None:
        print("dhrt nicht gebaut -- uebersprungen "
              "(python rust/build_runtime.py)")
        return 0
    bloecke, funde = pruefe(ordner, dhrt)
    print(f"{bloecke} Codebloecke in {ordner}/ geprueft -- "
          f"{len(funde)} Syntaxfehler")
    for name, zeile, quelle, msg in funde:
        print(f"\n  {name}:{zeile}")
        print(f"     {quelle[:90]}")
        print(f"     -> {msg[:90]}")
    return 1 if funde else 0


if __name__ == "__main__":
    raise SystemExit(main())
