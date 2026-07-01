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

Zusaetzlich: nach jedem Block-Ende (`END SUB`/`END FUNCTION`/`NEXT`/`WEND`/...)
wird -- falls nicht schon vorhanden -- EINE Leerzeile eingefuegt, bevor der
naechste Code/Kommentar folgt. Ohne diese Regel klebten aufeinanderfolgende
SUB/FUNCTION-Definitionen (oder ein Block-Ende + der Kommentar der naechsten
Sektion) ohne jede optische Trennung aneinander -- lesbarkeitsschaedlich bei
langen Programmen. Kein Einfuegen, wenn direkt ein WEITERES Block-Ende oder
eine Zwischen-Klausel (`ELSE`/`ELSEIF`/`CASE`/`CATCH`) folgt (verschachtelte
END-Ketten wie `END SUB` direkt vor `END CLASS` bleiben zusammen).
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
    if _OPENERS.match(upper) or _IF_BLOCK.match(upper):
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
    return "\n".join(_ensure_blank_after_closers(out))


def _ensure_blank_after_closers(lines: list[str]) -> list[str]:
    """Fuegt nach jeder Block-Ende-Zeile eine Leerzeile ein, falls dort noch
    keine steht -- ausser die naechste Zeile ist selbst ein Block-Ende oder
    eine Zwischen-Klausel (dann bleibt die END-Kette/Klausel zusammen)."""
    out: list[str] = []
    for i, line in enumerate(lines):
        out.append(line)
        stripped = line.strip()
        if not stripped or _classify(stripped) != "close":
            continue
        nxt = lines[i + 1] if i + 1 < len(lines) else None
        if nxt is None:
            continue
        nxt_stripped = nxt.strip()
        if nxt_stripped == "" or _classify(nxt_stripped) in ("close", "mid"):
            continue
        out.append("")
    return out
