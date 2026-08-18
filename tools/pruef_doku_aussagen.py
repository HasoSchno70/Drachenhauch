#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""Pruefbare Aussagen in `docs/` gegen die Wirklichkeit halten.

    <venv>\\python.exe tools\\pruef_doku_aussagen.py

Rueckgabe 0 = sauber, 1 = mindestens ein Befund.

ABGRENZUNG ZU tools/pruef_docs.py
---------------------------------
Der andere Pruefer schickt jeden ```basic-BLOCK durch `dhrt --check` und
findet damit Syntaxfehler. Er sieht aber nur Codebloecke -- und ausgerechnet
die Stellen, an denen eine Doku am ehesten luegt, stehen woanders: in
Tabellen ("| `AUDIO_INIT(freq)` | Mixer neu starten |") und in Fliesstext
("38 Module").

Zwei Kategorien lassen sich hier mechanisch pruefen:

1. BEFEHLSNAMEN. Jeder Name der Form `NAME(` in einem Inline-Code-Abschnitt
   muss ein Builtin sein, das es wirklich gibt. Ein Tippfehler oder ein
   umbenanntes Builtin faellt damit auf, ohne dass jemand die Tabelle
   nachtippt.

2. ZAEHLUNGEN. "39 Module", "183 Beispiele" -- Zahlen, die beim naechsten
   Commit veralten und die niemand nachzaehlt.

   NICHT dabei: die Zahl der Tests. Sie aendert sich mit JEDEM neuen Test,
   auch mit denen dieses Pruefers -- eine exakte Angabe waere nach dem
   naechsten Commit wieder falsch. Das README sagt darum "ueber 3400", und
   das bleibt lange richtig.

WAS HIER NICHT GEHT
-------------------
Verhaltensaussagen ("Default-Timeout 10 Sekunden", "laeuft auf dem
Tree-Walker") lassen sich nicht mechanisch pruefen -- die muss man messen.
Genau dort sassen die bisher gefundenen Fehler. Dieser Pruefer ersetzt das
Nachmessen nicht, er raeumt nur die Kategorien ab, die sich automatisieren
lassen.
"""
import json
import re
import sys
from pathlib import Path

WURZEL = Path(__file__).resolve().parents[1]

BLOCK = re.compile(r"```.*?```", re.S)
INLINE = re.compile(r"`([^`\n]+)`")
AUFRUF = re.compile(r"^([A-Z][A-Z0-9_]{2,}\$?)\s*\(")

# Sprach-Schluesselwoerter, Typen und Konstanten -- keine Builtins.
SPRACHE = set("""
DIM AS CONST IF THEN ELSE ELSEIF END WHILE WEND FOR TO STEP NEXT SUB FUNCTION
RETURN CLASS NEW EXTENDS TRUE FALSE NIL AND OR NOT MOD BREAK CONTINUE IMAGE
SOUND ARRAY OF STRUCT FILE MAP TRY CATCH THROW FINALLY IMPORT SELECT CASE IS
REPEAT UNTIL DATA READ RESTORE BYREF ENUM BAND BOR BXOR BNOT SHL SHR TUPLE
WITH STATIC FUNCREF IN WHERE PROPERTY OPERATOR YIELD COROUTINE PRINT INPUT
INTEGER FLOAT STRING BOOLEAN BUFFER PI TAU SELF SUPER ABSTRACT PRIVATE GET SET
LET REM ANY FUNKTION
""".split())

# Namen, die absichtlich nicht existieren. Jeder Eintrag braucht einen Grund --
# so bleibt die Liste eine bewusste Entscheidung und kein Abstellgleis.
GEDULDET = {
    "ECS_ADD_TO":   "module-ecs.md nennt sie ausdruecklich 'hypothetisch'",
    "TASK_START":   "allzweck-roadmap.md: WP H, ausdruecklich NICHT umgesetzt",
    "ERROR_FILE$":  "allzweck-roadmap.md: WP F, ausdruecklich gestrichen",
    "ERROR_TRACE$": "allzweck-roadmap.md: WP F, ausdruecklich gestrichen",
}


def bekannte_builtins() -> set:
    p = WURZEL / "drachenhauch" / "editor_qt" / "builtin_index.json"
    namen = {x["name"].upper() for x in json.loads(p.read_text(encoding="utf-8"))["builtins"]}
    # Konvention: `UPPER$` und `UPPER` sind dasselbe.
    return namen | {n.rstrip("$") for n in namen}


def pruefe_namen(ordner: Path) -> list:
    bekannt = bekannte_builtins()
    funde = []
    for datei in sorted(ordner.glob("*.md")):
        text = datei.read_text(encoding="utf-8")
        # Codebloecke deckt pruef_docs.py ab -- hier nur Prosa und Tabellen.
        ohne = BLOCK.sub(lambda m: "\n" * m.group(0).count("\n"), text)
        for m in INLINE.finditer(ohne):
            k = AUFRUF.match(m.group(1).strip())
            if not k:
                continue
            name = k.group(1).upper()
            if name in SPRACHE or name.rstrip("$") in SPRACHE:
                continue
            if name in bekannt or name.rstrip("$") in bekannt:
                continue
            if name in GEDULDET:
                continue
            zeile = ohne[:m.start()].count("\n") + 1
            funde.append((datei.name, zeile, name,
                          "kein Builtin dieses Namens (Tippfehler? umbenannt? entfernt?)"))
    return funde


def zaehlungen() -> list:
    """Zahlen in README.md gegen die Wirklichkeit."""
    readme = WURZEL / "README.md"
    text = readme.read_text(encoding="utf-8")
    funde = []

    ist_beispiele = len(list((WURZEL / "examples").glob("*.dh")))
    quelle = (WURZEL / "rust" / "drachenhauch_runtime" / "src" / "preprocess.rs").read_text(encoding="utf-8")
    m = re.search(r"const MODULES[^=]*=\s*&\[(.*?)\];", quelle, re.S)
    ist_module = len(re.findall(r'"([a-z_0-9]+)"', m.group(1))) if m else 0

    for muster, ist, was in (
        (r"alle (\d+) Beispiele", ist_beispiele, "Beispiele in examples/"),
        (r"\*\*Module\*\* — (\d+) Stück", ist_module, "Module in preprocess.rs MODULES"),
        (r"^(\d+) Module, per `IMPORT", ist_module, "Module in preprocess.rs MODULES"),
    ):
        for t in re.finditer(muster, text, re.M):
            soll = int(t.group(1))
            if soll != ist:
                zeile = text[:t.start()].count("\n") + 1
                funde.append(("README.md", zeile, t.group(0),
                              f"sagt {soll}, tatsaechlich {ist} ({was})"))
    return funde


def main():
    funde = pruefe_namen(WURZEL / "docs") + zaehlungen()
    print(f"Doku-Aussagen geprueft -- {len(funde)} Befund(e)")
    for datei, zeile, was, msg in funde:
        print(f"\n  {datei}:{zeile}")
        print(f"     {was}")
        print(f"     -> {msg}")
    if GEDULDET:
        print(f"\n({len(GEDULDET)} Namen geduldet, siehe GEDULDET im Quelltext)")
    return 1 if funde else 0


if __name__ == "__main__":
    raise SystemExit(main())
