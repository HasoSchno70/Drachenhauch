"""Code-Formatter: Indent-Normalisierung fuer GameBasic.

Strategie: pro Zeile bestimmen, wie tief wir aktuell stehen
(`indent_level`), basierend auf Block-Openern/-Closern. Tab-Zeichen
und vorhandene Leerzeichen am Zeilenanfang werden verworfen und durch
4*level Spaces ersetzt.

Subtilitaeten:
- `ELSE`, `ELSEIF`, `CASE`, `CATCH` rendert auf level - 1 (sind Klauseln
  *innerhalb* eines noch offenen Blocks).
- `END IF`, `WEND`, `NEXT`, `END SUB`, ... ebenfalls auf level - 1
  (schliessen den Block, ihr Inhalt steht auf level).
- Zeilen, die nichts als Whitespace sind, werden auf "" normalisiert.
- Comment-Zeilen (`'`/`REM`) erben das aktuelle Indent.

Single-Line-IFs (`IF x THEN PRINT 1`) erhoehen den Indent NICHT --
nur `IF ... THEN` ohne Statement nach THEN ist ein Block-Opener.

Zusaetzlich zwei Leerzeilen-Regeln (Sektions-Abstand), je -- falls nicht
schon vorhanden -- EINE Leerzeile:
1. Nach jedem Block-Ende (`END SUB`/`END FUNCTION`/`NEXT`/`WEND`/...), bevor
   der naechste Code/Kommentar folgt. Ohne diese Regel klebten
   aufeinanderfolgende SUB/FUNCTION-Definitionen aneinander.
2. Vor jeder eigenstaendigen Kommentar-Zeile (`'`/`REM`), die eine neue
   Sektion einleitet -- z.B. ein Kommentar direkt nach einer `DIM`-Zeile
   ohne Trennung. Ohne diese Regel klebte Code direkt an den Kommentar der
   naechsten Sektion.
Beide Regeln greifen NICHT, wenn die vorherige Zeile selbst ein Block-
Opener (`SUB`/`FUNCTION`/...) oder eine Zwischen-Klausel (`ELSE`/`CASE`/...)
ist (dann ist der Kommentar der einleitende Block-/Klausel-Kommentar und
soll direkt an seinem Code kleben) -- und nicht, wenn die vorherige Zeile
selbst schon eine Kommentar-Zeile ist (ein mehrzeiliger Kommentarblock wird
nicht auseinandergerissen).
"""
from __future__ import annotations

import re

INDENT_SPACES = 4

_OPENERS = re.compile(
    r"^(SUB|FUNCTION|CLASS|STRUCT|ENUM|WHILE|REPEAT|TRY|WITH|PROPERTY|FOR|SELECT\s+CASE)\b",
    re.IGNORECASE,
)
# IF mit THEN am Ende ohne Statement danach == Block-Opener. Single-Line-IF
# (IF x THEN PRINT y) ist KEIN Opener.
_IF_BLOCK = re.compile(r"^IF\b.*\bTHEN\s*$", re.IGNORECASE)
# Review-Fund: die kompakte ENUM-Form (`ENUM State = MENU, PLAYING, PAUSED`,
# siehe CLAUDE.md) hat KEIN passendes `END ENUM` -- `_OPENERS` behandelte
# aber JEDE mit ENUM beginnende Zeile als Block-Opener, was `level` fuer den
# Rest der Datei permanent um eins zu hoch liess (kein Closer poppt ihn
# jemals zurueck). Nur die Block-Form (`ENUM Name` allein, ohne `=`) oeffnet
# tatsaechlich einen Block.
_ENUM_COMPACT = re.compile(r"^ENUM\b[^=]*=", re.IGNORECASE)
# Closer (1:1 zu Opener)
_CLOSER_EXACT = {
    "END IF", "END SUB", "END FUNCTION", "END CLASS", "END STRUCT",
    "END ENUM", "END SELECT", "END TRY", "END PROPERTY", "END WITH",
    "WEND",
}
_CLOSER_PREFIX = ("NEXT", "UNTIL")
# Mid-Block-Klauseln rendern auf level - 1 ohne Stack-Aenderung.
_MID_EXACT = {"ELSE", "CATCH"}
_MID_PREFIX = ("ELSEIF ", "CATCH ", "CASE ")


def _classify(stripped: str) -> str:
    """Liefert 'open', 'close', 'mid' oder 'plain'."""
    upper = stripped.upper()
    if upper in _CLOSER_EXACT or any(upper == p or upper.startswith(p + " ") for p in _CLOSER_PREFIX):
        return "close"
    if upper in _MID_EXACT or any(upper.startswith(p) for p in _MID_PREFIX):
        return "mid"
    if _OPENERS.match(upper):
        if upper.startswith("ENUM") and _ENUM_COMPACT.match(upper):
            return "plain"
        return "open"
    if _IF_BLOCK.match(upper):
        return "open"
    return "plain"


def format_source(source: str) -> str:
    """Liefert den Quelltext mit normalisiertem Indent zurueck.

    SELECT CASE braucht eine Sonderbehandlung: das `CASE ...` rendert
    eingerueckt unter dem `SELECT CASE`, das Statement-Body danach noch
    eine Stufe tiefer. `select_stack` haelt die Render-Levels der CASE-
    Klauseln pro offenem SELECT.
    """
    out: list[str] = []
    level = 0
    select_stack: list[int] = []   # Render-Level fuer CASE-Klauseln pro SELECT-Block

    def render(lvl: int, body: str) -> str:
        return " " * (lvl * INDENT_SPACES) + body

    for raw in source.split("\n"):
        body = raw.rstrip().lstrip(" \t")
        if not body:
            out.append("")
            continue
        upper = body.upper()

        # END SELECT schliesst den aktuellen SELECT-Block. Wir wechseln
        # zurueck auf das Pre-SELECT-Niveau, unabhaengig davon, wo der
        # letzte CASE-Body uns hingebracht hat (level kann case_base+1 sein).
        if upper == "END SELECT":
            if select_stack:
                case_base = select_stack.pop()
                level = max(0, case_base - 1)
            else:
                level = max(0, level - 1)
            out.append(render(level, body))
            continue

        # CASE-Klausel innerhalb eines SELECT?
        if select_stack and (upper == "CASE ELSE" or upper.startswith("CASE ")):
            case_level = select_stack[-1]
            out.append(render(case_level, body))
            level = case_level + 1
            continue

        # SELECT-Opener separat behandeln (push auf select_stack).
        if upper == "SELECT CASE" or upper.startswith("SELECT CASE "):
            out.append(render(level, body))
            select_stack.append(level + 1)
            level += 1
            continue

        kind = _classify(body)
        if kind == "close":
            level = max(0, level - 1)
            out.append(render(level, body))
            continue
        if kind == "mid":
            out.append(render(max(0, level - 1), body))
            continue
        if kind == "open":
            out.append(render(level, body))
            level += 1
            continue
        out.append(render(level, body))
    return "\n".join(_ensure_section_spacing(out))


def _is_comment(stripped: str) -> bool:
    upper = stripped.upper()
    return stripped.startswith("'") or upper == "REM" or upper.startswith("REM ")


def _ensure_section_spacing(lines: list[str]) -> list[str]:
    """Fuegt eine Leerzeile ein (1) nach jeder Block-Ende-Zeile und (2) vor
    jeder neuen Kommentar-Sektion -- siehe Modul-Docstring. Beide Faelle
    werden PRO ZEILENUEBERGANG hoechstens einmal ausgeloest (kein doppeltes
    Einfuegen, wenn ein Block-Ende direkt vor einem Sektions-Kommentar steht)."""
    out: list[str] = []
    for i, line in enumerate(lines):
        out.append(line)
        cur = line.strip()
        if not cur:
            continue
        nxt = lines[i + 1] if i + 1 < len(lines) else None
        if nxt is None:
            continue
        nxt_stripped = nxt.strip()
        if nxt_stripped == "":
            continue   # schon eine Leerzeile vorhanden
        cur_kind = _classify(cur)
        need_blank = False
        if cur_kind == "close":
            if _classify(nxt_stripped) not in ("close", "mid"):
                need_blank = True
        elif _is_comment(nxt_stripped) and not _is_comment(cur):
            if cur_kind not in ("open", "mid"):
                need_blank = True
        if need_blank:
            out.append("")
    return out
