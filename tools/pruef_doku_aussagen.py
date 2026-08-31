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

Vier Kategorien lassen sich hier mechanisch pruefen:

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
Genau dort sassen ALLE bisher gefundenen Fehler. Dieser Pruefer ersetzt das
Nachmessen nicht, er raeumt nur die Kategorien ab, die sich automatisieren
lassen.

WIE MAN VON HAND SUCHT (aus fuenf Durchgaengen)
-----------------------------------------------
1. ZUERST NACH SELBSTWIDERSPRUECHEN. Eine Doku, die eine Migration hinter
   sich hat, wurde meist abschnittsweise nachgezogen -- die richtige Fassung
   steht dann schon woanders in DERSELBEN Datei. Alle drei Migrations-Fehler,
   die dieses Projekt hatte, waren von der Art:
     module-net.md      Zeile 3 "Python-stdlib-Sockets" <-> letzter Abschnitt
                        "reine std::net, keine zusaetzliche Crate"
     module-audio.md    Zeile 156 "von raylib direkt dekodiert" <-> Zeile 485
                        "eigener Kira-Custom-Sound, der xmrs pollt"
     builtins-grafik.md Abschnitt Z-Layer "jeder Layer ist eine off-screen
                        Surface, FLIP blittet" <-> Abschnitt Sprite-Atlas
                        "Aufzeichnungs-Modell, jeder Befehl haengt ein Cmd an"
   Ein Abgleich der Datei mit sich selbst haette alle drei gefunden.

2. TESTS SIND KEIN BELEG. Fuer NET_SET_TIMEOUT gab es einen Test -- er rief
   die Funktion mit 0, 100 und -1 auf und prueft, dass nichts abstuerzt.
   Die Doku beschrieb die Bedeutung von "0" trotzdem genau verkehrt herum.
   Wer eine Aussage prueft, muss sie AUSFUEHREN, nicht nachsehen, ob es
   einen Test gibt.

3. NEGATIV-AUSSAGEN ALTERN AM SCHNELLSTEN. "X ist nicht moeglich" wird
   falsch, sobald jemand X baut -- und niemand sucht dann die Doku ab.
   (grep -i nach "nicht moeglich", "gibt es nicht", "kann nicht")
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
LET REM ANY FUNKTION IIF
""".split())

# Namen, die absichtlich nicht existieren. Jeder Eintrag braucht einen Grund --
# so bleibt die Liste eine bewusste Entscheidung und kein Abstellgleis.
GEDULDET = {
    "ECS_ADD_TO":   "module-ecs.md nennt sie ausdruecklich 'hypothetisch'",
    "TASK_START":   "allzweck-roadmap.md: WP H, ausdruecklich NICHT umgesetzt",
    "ERROR_FILE$":  "allzweck-roadmap.md: WP F, ausdruecklich gestrichen",
    "ERROR_TRACE$": "allzweck-roadmap.md: WP F, ausdruecklich gestrichen",
    "NAME":         "CLAUDE.md: Platzhalter in der Builtin-Anleitung (`NAME(a, b [, c])`)",
    "BITAND":       "CLAUDE.md: als ENTFERNT beschrieben (heute der Operator BAND)",
    "DECLARE_GLOBAL_SLOT":       "CLAUDE.md: Bytecode-Opcode, kein Builtin",
    "DECLARE_GLOBAL_CONST_SLOT": "CLAUDE.md: Bytecode-Opcode, kein Builtin",
}


def _md_dateien(ordner: Path, nur: set[str] | None = None) -> list:
    """Die zu pruefenden .md-Dateien eines Ordners -- mit `nur` nur die
    genannten. Damit laesst sich CLAUDE.md pruefen, ohne jede andere .md im
    Repo-Wurzelverzeichnis (README, VERKAUF-CHECKLISTE, ...) mitzunehmen."""
    if nur is not None:
        return [ordner / n for n in sorted(nur) if (ordner / n).exists()]
    return sorted(ordner.glob("*.md"))


def bekannte_builtins() -> set:
    p = WURZEL / "drachenhauch" / "editor_qt" / "builtin_index.json"
    namen = {x["name"].upper() for x in json.loads(p.read_text(encoding="utf-8"))["builtins"]}
    # Konvention: `UPPER$` und `UPPER` sind dasselbe.
    return namen | {n.rstrip("$") for n in namen}


def pruefe_namen(ordner: Path, nur: set[str] | None = None) -> list:
    bekannt = bekannte_builtins()
    funde = []
    for datei in _md_dateien(ordner, nur):
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


def beispiele() -> list:
    """Die .dh-Dateien in `examples/` -- ohne Temp-Reste der IDE.

    Fehlerpruefung, Debugger und Profiler legen ihre Arbeitsdatei NEBEN die
    Quelle (sonst loest `IMPORT "helfer.dh"` nicht auf). Bricht so ein Lauf
    ab, bleibt sie liegen -- und kippte bisher jede Zaehlung ueber
    `examples/`. Seit sie ein erkennbares Praefix traegt, laesst sie sich
    ueberspringen.
    """
    from drachenhauch.editor_qt.tempdateien import PRAEFIX
    return [p for p in (WURZEL / "examples").glob("*.dh")
            if not p.name.startswith(PRAEFIX)]


def zaehlungen() -> list:
    """Zahlen in README.md gegen die Wirklichkeit."""
    readme = WURZEL / "README.md"
    text = readme.read_text(encoding="utf-8")
    funde = []

    ist_beispiele = len(beispiele())
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


def konstanten(ordner: Path) -> list:
    """3. KONSTANTEN. Jede eingebaute Tasten-Konstante muss in `docs/` stehen.

    Gefunden beim Durchgang durch `builtins-grafik.md`: die Datei fuehrte eine
    Liste "Verfuegbare Konstanten" -- und die war um 21 Eintraege zu kurz
    (`KEY_F1` bis `KEY_F12` und saemtliche Modifier fehlten). Eine Liste, die
    sich als vollstaendig ausgibt und es nicht ist, kostet mehr als gar keine:
    wer `KEY_F5` sucht, nicht findet und daraus schliesst, es gebe die Taste
    nicht, baut sich einen Umweg um ein Loch, das keines ist.

    Bereichsschreibweisen ("`KEY_A` bis `KEY_Z`") gelten als Abdeckung -- die
    Doku soll 26 Buchstaben nicht einzeln aufzaehlen muessen.
    """
    quelle = (WURZEL / "rust" / "drachenhauch_runtime" / "src" / "vm.rs").read_text(encoding="utf-8")
    stelle = re.search(r"const DEFAULT_KEYS[^=]*=\s*&\[(.*?)\n\];", quelle, re.S)
    if not stelle:
        return [("rust/.../vm.rs", 0, "DEFAULT_KEYS",
                 "Tabelle nicht mehr gefunden -- diese Pruefung laeuft ins Leere")]
    keys = {k.upper() for k in re.findall(r'"(key_[a-z0-9_]+)"', stelle.group(1))}
    doku = "\n".join(d.read_text(encoding="utf-8") for d in sorted(ordner.glob("*.md")))
    bereiche = [(r"KEY_[A-Z]$", "`KEY_A` bis `KEY_Z`"),
                (r"KEY_[0-9]$", "`KEY_0` bis `KEY_9`"),
                (r"KEY_F\d+$", "`KEY_F1` bis `KEY_F12`"),
                (r"KEY_KP\d$", "`KEY_KP0` bis `KEY_KP9`")]

    def fehlt(k: str) -> bool:
        if k in doku:
            return False
        return not any(re.match(m, k) and t in doku for m, t in bereiche)

    offen = sorted(k for k in keys if fehlt(k))
    if not offen:
        return []
    return [("docs/*.md", 0, ", ".join(offen),
             f"{len(offen)} Tasten-Konstante(n) gibt es in der Runtime, "
             "stehen aber in keiner Doku-Datei")]


# Pfade, die es nicht (mehr) gibt und die trotzdem stehen bleiben duerfen.
# Jeder Eintrag braucht einen Grund -- sonst ist es kein Abstellgleis, sondern
# eine Doku, die auf Dateien zeigt, die niemand findet.
GEDULDETE_PFADE = {
    "drachenhauch/interpreter.py": "Tree-Walker, mit Stufe B entfernt -- nur in historischen Notizen",
    "drachenhauch/parser.py": "Python-Parser, 2026-08-19 entfernt (Stufe C) -- nur noch im Entwurf genannt",
    "drachenhauch/ast_nodes.py": "AST-Knoten des Python-Parsers, mit ihm entfernt",
    "tests/test_rust_parser_parity.py": "Parity gegen den entfernten Python-Parser",
    "drachenhauch/serialize.py": "Python-Bytecode-Serializer, mit Stufe B entfernt",
    "drachenhauch/vm.py": "Python-Bytecode-VM, mit Stufe B entfernt",
    "drachenhauch/export.py": "Python-Export, von dhrts Bundler abgeloest",
    "drachenhauch/modules/gui.py": "Modul-Implementierung, in Rust reimplementiert",
    "drachenhauch/modules/ui.py": "Modul-Implementierung, in Rust reimplementiert",
    "tests/test_modules_gui.py": "Test der entfernten Python-Module",
    "tests/test_rust_compiler_parity.py": "Paritaets-Test gegen den entfernten Python-Compiler",
    "gb_native/src/broadphase.rs": "PyO3-Helfer-Crate, nach physics.rs portiert und entfernt",
    "drachenhauch/modules/x.py": "Platzhalter in einer Anleitung, kein echter Pfad",
    "examples/NN_gui.dh": "Platzhalter (NN = laufende Nummer), kein echter Pfad",
    "/program.dh": "virtueller Pfad im WASM-Dateisystem des Web-Playgrounds",
    "web/program.dh": "virtueller Pfad im WASM-Dateisystem des Web-Playgrounds",
    "modules/ecs_py.py": "Python-ECS, mit Stufe B entfernt -- nur in historischen Notizen",
    "rust/build.py": "Cython/PyO3-Build, mit Stufe B entfernt -- nur in historischen Notizen",
}


def pfade(ordner: Path, nur: set[str] | None = None) -> list:
    """4. PFADVERWEISE. `drachenhauch/interpreter.py` in einer Anleitung.

    Der Tree-Walker ist seit Stufe B entfernt -- die Umsetzungs-Checkliste in
    `befehlssatz-roadmap.md` schickte trotzdem noch jeden, der ein Builtin
    baut, als ERSTEN Schritt in `drachenhauch/interpreter.py`. Ein Verweis auf
    eine Datei, die es nicht gibt, kostet den Leser die Zeit, bis er es
    merkt -- und laesst ihn zweifeln, ob der Rest stimmt.

    Geprueft wird jeder Inline-Code-Verweis, der wie ein Pfad aussieht
    (enthaelt `/`, endet auf .py/.rs/.dh/.json/.toml). Repo-relativ ODER als
    Suffix irgendwo im Baum -- `src/vm.rs` meint das Crate-Unterverzeichnis
    und ist gueltig. Bekannte Leichen stehen in GEDULDETE_PFADE.
    """
    alle = set()
    for p in WURZEL.rglob("*"):
        if p.is_file() and "target" not in p.parts and "__pycache__" not in p.parts:
            alle.add(p.relative_to(WURZEL).as_posix())
    muster = re.compile(r"`([A-Za-z0-9_./-]+\.(?:py|rs|dh|json|toml))`")
    # Markdown-Links: `[Text](pfad)` bzw. `[Text](pfad:42)`. Die pruefen sich
    # nicht von selbst -- im Editor sind sie klickbar, und ein toter Link faellt
    # erst auf, wenn jemand darauf klickt. In CLAUDE.md waren am 2026-08-29 vier
    # davon tot (interpreter.py, compiler.py, ecs_py.py, eine Testdatei), alle
    # auf Dateien, die Stufe B geloescht hatte.
    link = re.compile(r"\]\(([^)\s]+)\)")
    verzeichnisse = {d.relative_to(WURZEL).as_posix()
                     for d in WURZEL.rglob("*")
                     if d.is_dir() and "target" not in d.parts and "__pycache__" not in d.parts}

    def lebt(r: str) -> bool:
        r = r.lstrip("/")
        return any(a == r or a.endswith("/" + r) for a in alle)

    def ziel_lebt(r: str) -> bool:
        r = r.split("#")[0].rstrip("/")               # Anker, Schraegstrich am Ende
        r = re.sub(r":\d+$", "", r)                   # `datei.py:114` -> Datei
        while r.startswith("../"):                    # aus docs/ heraus
            r = r[3:]
        # GEDULDETE_PFADE gilt hier BEWUSST NICHT: eine historische Nennung
        # schreibt man als Inline-Code (`drachenhauch/interpreter.py`), nicht
        # als Link. Ein Link verspricht "hier kannst du hinspringen" -- und
        # genau so standen der geloeschte Tree-Walker und die geloeschte
        # Parser-Parity in CLAUDE.md, klickbar und ins Leere zeigend.
        return not r or lebt(r) or r in verzeichnisse

    funde = []
    for datei in _md_dateien(ordner, nur):
        # relative_to wirft ausserhalb des Repos (Tests pruefen mit tmp_path)
        wo = (datei.relative_to(WURZEL).as_posix()
              if datei.is_relative_to(WURZEL) else datei.name)
        for nr, zeile in enumerate(datei.read_text(encoding="utf-8").splitlines(), 1):
            for m in muster.finditer(zeile):
                r = m.group(1)
                if "/" not in r or lebt(r) or r in GEDULDETE_PFADE:
                    continue
                funde.append((wo, nr, r,
                              "Pfad existiert nicht -- entfernt, umbenannt oder vertippt"))
            for m in link.finditer(zeile):
                r = m.group(1)
                if r.startswith(("http", "#", "mailto:")) or ziel_lebt(r):
                    continue
                funde.append((wo, nr, f"[...]({r})",
                              "Link zeigt ins Leere -- Ziel entfernt, umbenannt oder vertippt"))
    return funde


def main():
    # CLAUDE.md wird MITGEPRUEFT -- und das aus dem gleichen Grund wie `docs/`,
    # nur mit mehr Gewicht: sie ist die Datei, die als Erstes gelesen wird, wenn
    # jemand (oder etwas) sich in diesem Repo zurechtfinden will. Beim Sweep am
    # 2026-08-29 standen dort vier tote Markdown-Links (auf den geloeschten
    # Tree-Walker, den Python-Compiler und eine geloeschte Testdatei), ein
    # umbenanntes Beispiel, eine Test-Zaehlung von 3224 statt ueber 3800, und ein
    # ganzer Abschnitt, der den Web-Playground als "Geruest, nicht gebaut"
    # fuehrte -- obwohl der Abschnitt DARUEBER in derselben Datei "gebaut +
    # verifiziert" sagte. Genau die Sorte Selbstwiderspruch, vor der der
    # Kopfkommentar dieses Skripts warnt.
    #
    # Geprueft wird sie wie eine Doku-Datei; die Bereiche, die es hier nicht
    # gibt (Zaehlungen im README, Tastenkonstanten), bleiben bei `docs/`.
    funde = (pruefe_namen(WURZEL / "docs") + zaehlungen()
             + konstanten(WURZEL / "docs") + pfade(WURZEL / "docs")
             + pruefe_namen(WURZEL, nur={"CLAUDE.md"})
             + pfade(WURZEL, nur={"CLAUDE.md"}))
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
