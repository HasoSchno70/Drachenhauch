"""Geteilte Fuzzy-Match-Scoring fuer Quick-Open und Command-Palette.

Liefert einen Score fuer (query, label) -- hoeher = besseres Match. Bei
keinem Match: None. Aufrufer sortiert die Treffer nach Score absteigend.

Heuristik (lehnt sich an VSCode/Sublime an):
- Gross-/Klein-insensitiv.
- Alle Query-Zeichen muessen IN ORDER im Label vorkommen.
- Bonus-Punkte fuer:
    * Wort-Anfaenge (nach `_`, `-`, `/`, `\\`, `.`, ` `)
    * Camel-Cap-Boundary (Kleinbuchstabe gefolgt von Grossbuchstabe)
    * Aufeinanderfolgende Matches
    * Match am Label-Anfang
"""
from __future__ import annotations


_SEPARATORS = set(" _-./\\")


def score(query: str, label: str) -> int | None:
    """Fuzzy-Match-Score; None wenn die Query kein subsequence in label ist."""
    if not query:
        return 0
    q = query.lower()
    l = label.lower()
    if len(q) > len(l):
        return None

    qi = 0
    s = 0
    last_match = -2
    prev_label_char = ""
    for li, ch in enumerate(l):
        if qi >= len(q):
            break
        if ch == q[qi]:
            # Basis-Punkt fuer das Match.
            s += 1
            # Bonus: aufeinanderfolgende Matches.
            if li == last_match + 1:
                s += 5
            # Bonus: Match am Anfang.
            if li == 0:
                s += 8
            # Bonus: nach Separator.
            elif prev_label_char in _SEPARATORS:
                s += 6
            # Bonus: Camel-Cap (lower -> upper im ORIGINAL-Label).
            elif (li > 0 and label[li - 1].islower() and label[li].isupper()):
                s += 4
            last_match = li
            qi += 1
        prev_label_char = ch

    if qi < len(q):
        return None
    # Penalty: laenge des labels (kuerzeres == besser bei gleichem Match-Bonus).
    s -= max(0, len(label) - len(query)) // 8
    return s
