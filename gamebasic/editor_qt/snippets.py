"""Snippet-Definitionen + Trigger-Logik fuer den Code-Editor.

Aufruf: User tippt Trigger (z.B. ``ife``), drueckt Tab. Falls der
Trigger zu einem Snippet matcht UND links davon ein Wort steht (kein
Whitespace), wird der Trigger durch den Snippet-Body ersetzt. Cursor
und Selektion landen am ersten Placeholder.

Body-Marker:
- ``|``         — einzelner Cursor-Anker, leere Selection.
- ``${1:text}`` — Placeholder mit Default. Default-Text wird vom Editor
                  selektiert, sodass der User direkt ueberschreiben kann.
                  Die Nummer ist (noch) nur fuer Lesbarkeit; spaetere
                  Tab-Stops sind ein folge-Feature.
- ``${1}``      — Placeholder ohne Default = leere Selection an dieser
                  Stelle (semantisch wie ``|``, aber nummeriert).

Kein Match -> Tab fuegt Indent ein (Standardverhalten).
"""
from __future__ import annotations

import re


# Placeholder-Erkennung: ``${<n>}`` oder ``${<n>:<default>}``. ``<default>``
# darf alles ausser ``{`` und ``}`` enthalten -- verschachtelte Snippets
# unterstuetzen wir bewusst nicht.
_PLACEHOLDER_RE = re.compile(r"\$\{(\d+)(?::([^{}]*))?\}")


# Trigger -> Body. ``|`` markiert die Cursor-Position nach Insertion.
# ``${N:default}`` markiert einen Placeholder mit auswaehlbarem Default.
SNIPPETS: dict[str, str] = {
    "if":  "IF ${1:condition} THEN\n    ${0}\nEND IF",
    "ife": "IF ${1:condition} THEN\n    ${2}\nELSE\n    ${0}\nEND IF",
    "for": "FOR ${1:i} = ${2:1} TO ${3:n}\n    ${0}\nNEXT",
    "wh":  "WHILE ${1:condition}\n    ${0}\nWEND",
    "rp":  "REPEAT\n    ${0}\nUNTIL ${1:condition}",
    "sc":  "SELECT CASE ${1:value}\n    CASE ${2:1}\n        ${0}\n    CASE ELSE\n        \nEND SELECT",
    "sub": "SUB ${1:name}(${2})\n    ${0}\nEND SUB",
    "fn":  "FUNCTION ${1:name}(${2}) AS ${3:INTEGER}\n    ${0}\nEND FUNCTION",
    "cls": "CLASS ${1:Name}\n    ${0}\nEND CLASS",
    "try": "TRY\n    ${0}\nCATCH ${1:e}\n    PRINT ${1:e}\nEND TRY",
    "gl": (
        "WHILE NOT QUITREQUESTED()\n"
        "    IF KEYPRESSED(27) THEN\n"
        "        BREAK\n"
        "    END IF\n"
        "    ${0}\n"
        "    CLS()\n"
        "    \n"
        "    FLIP()\n"
        "    SLEEP(16)\n"
        "WEND"
    ),
    "scr": 'SCREEN(${1:320}, ${2:240}, "${3:Titel}", ${4:2})${0}',
    "imp": 'IMPORT "${1:module}"${0}',
}


def expand_snippet(body: str, indent: str) -> tuple[str, int]:
    """Bereitet einen Snippet-Body fuer Insertion vor.

    Backwards-compat-Form: liefert (body_ohne_marker, cursor_offset).
    Default-Texte aus Placeholdern werden eingesetzt; der Cursor landet
    auf dem ersten ``${1...}`` (oder auf ``|``, oder am Ende).

    Wer auch die Selection-Laenge braucht (um den Default-Text auszuwaehlen),
    nutzt ``expand_snippet_full``.

    - Folge-Zeilen werden mit `indent` praefigiert (matched die Einrueckung
      der aktuellen Zeile).
    - ``${0}`` markiert die Final-Cursor-Position (wo der Cursor "am Ende"
      sitzt, wenn alle Tab-Stops abgearbeitet sind). Aktuell behandeln wir
      das wie einen normalen Placeholder: wenn der Body kein ``${1}``
      hat, gewinnt ``${0}``; sonst gewinnt der niedrigste positive Index.
    - ``|`` ist Backwards-Compat-Alias fuer ``${0}`` (leerer Anker).
    """
    body, offset, _length = expand_snippet_full(body, indent)
    return body, offset


def expand_snippet_full(body: str, indent: str) -> tuple[str, int, int]:
    """Wie `expand_snippet`, aber liefert zusaetzlich die Laenge des
    Default-Texts an der Cursor-Position. Editor kann den Bereich dann
    selektieren (User tippt -> ueberschreibt Default).
    """
    # 1) Mehrzeiler: Folge-Zeilen mit Indent praefixen.
    lines = body.split("\n")
    if len(lines) > 1:
        body = lines[0] + "\n" + "\n".join(indent + ln for ln in lines[1:])

    # 2) Backwards-compat: `|` zu `${0}` umbiegen, damit der Placeholder-
    #    Pfad unten alle Faelle abdeckt. Wir nehmen nur das ERSTE `|` --
    #    konsistent mit dem alten Verhalten.
    if "|" in body:
        idx = body.index("|")
        body = body[:idx] + "${0}" + body[idx + 1:]

    # 3) Placeholder finden + ersetzen, primary auswaehlen.
    placeholders = list(_PLACEHOLDER_RE.finditer(body))
    if not placeholders:
        return body, len(body), 0

    # Wahl des primaeren Placeholders:
    # - bevorzugt der niedrigste positive Index (${1}, ${2}, ...)
    # - sonst ${0}
    primary = None
    primary_idx = None
    for m in placeholders:
        n = int(m.group(1))
        if n > 0 and (primary_idx is None or n < primary_idx):
            primary = m
            primary_idx = n
    if primary is None:
        # Nur ${0}-Marker vorhanden.
        primary = placeholders[0]

    # 4) Alle Placeholder durch ihren Default-Text ersetzen, dabei den
    #    Offset und die Laenge des primaeren tracken.
    out = []
    pos = 0
    primary_offset = -1
    primary_length = 0
    for m in placeholders:
        out.append(body[pos:m.start()])
        default = m.group(2) or ""
        if m is primary:
            primary_offset = sum(len(s) for s in out)
            primary_length = len(default)
        out.append(default)
        pos = m.end()
    out.append(body[pos:])
    final = "".join(out)
    if primary_offset < 0:
        primary_offset = len(final)
    return final, primary_offset, primary_length
