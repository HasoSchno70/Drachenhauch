"""Drift-Schutz fuer die von Hand gepflegten Namenslisten der Editor-Schicht.

Vorbild ist `test_constants_sync.py` (COLORS/KEYS gegen die Runtime). Das
Muster gab es hier laengst, es war nur nie auf die Editor-Listen angewandt --
und genau dort war es passiert:

  * `completer.KEYWORDS` fehlten `IS`, `FINALLY`, `PRIVATE`. `IS` ist der
    Laufzeit-Typtest (`x IS Hund`, `x IS NOT NIL`), also ausgerechnet das
    juengste Sprachfeature.
  * `highlighter._CONTROL_KW/_DECL_KW/_TYPE_KW` fehlten `FINALLY` und
    `PRIVATE` -- beide blieben ungefaerbt.
  * `highlighter.BUILTIN_NAMES` kannte 72 von 1558 Builtins.

Die ersten beiden Listen sind inzwischen abgeleitet bzw. ergaenzt, die
dritte faellt weg (sie zieht aus `dhrt_meta`). Dieser Test haelt den
Zustand fest -- inklusive der Richtung, die am leichtesten uebersehen wird:
ein neues Schluesselwort in dhrt, von dem der Editor nie erfaehrt.
"""
from __future__ import annotations

import re
from pathlib import Path

from drachenhauch.tokens import KEYWORDS, TokenType
from drachenhauch.editor_qt import highlighter as H
from drachenhauch.editor_qt.completer import KEYWORDS as COMPLETION
from drachenhauch.editor_qt.dhrt_meta import builtin_names_upper

WURZEL = Path(__file__).resolve().parents[1]

# TRUE/FALSE/NIL sind Schluesselwoerter, bekommen im Highlighter aber die
# eigene Klasse "bool" statt ctrl/decl/type.
_BOOL_KW = {TokenType.TRUE, TokenType.FALSE, TokenType.NIL}


def test_completion_schlaegt_jedes_schluesselwort_vor():
    fehlend = {k.upper() for k in KEYWORDS} - set(COMPLETION)
    assert not fehlend, f"nicht vorgeschlagen: {sorted(fehlend)}"


def test_completion_schlaegt_jedes_builtin_vor():
    from drachenhauch.editor_qt.completer import collect_builtins
    fehlend = set(builtin_names_upper()) - set(collect_builtins())
    assert not fehlend, f"nicht vorgeschlagen: {sorted(fehlend)[:10]}"


def test_highlighter_faerbt_jedes_schluesselwort():
    """Jedes Keyword-Token muss in genau einer Highlight-Gruppe stehen --
    sonst faellt es durch `classify_token` und bleibt farblos."""
    gruppen = H._CONTROL_KW | H._DECL_KW | H._TYPE_KW | _BOOL_KW
    fehlend = {tt for tt in KEYWORDS.values()} - gruppen
    assert not fehlend, f"ohne Highlight-Klasse: {sorted(str(t) for t in fehlend)}"


def test_highlighter_gruppen_ueberschneiden_sich_nicht():
    """Ein Keyword in zwei Gruppen bekaeme je nach Pruefreihenfolge in
    `classify_token` mal die eine, mal die andere Farbe."""
    for a, b, namen in [(H._CONTROL_KW, H._DECL_KW, "ctrl/decl"),
                        (H._CONTROL_KW, H._TYPE_KW, "ctrl/type"),
                        (H._DECL_KW, H._TYPE_KW, "decl/type")]:
        assert not (a & b), f"{namen} doppelt: {sorted(str(t) for t in a & b)}"


def test_keyword_tabelle_deckt_sich_mit_dhrt():
    """`tokens.KEYWORDS` (Editor-Lexer) gegen `keyword()` in `lexer.rs`
    (dhrt). Beide Tabellen werden von Hand gepflegt; laeuft eine davon
    weg, lext der Editor anders als die Laufzeit -- und ein neues
    Sprach-Schluesselwort waere im Editor ein gewoehnlicher Bezeichner.

    Die Parity-Tests daneben vergleichen Token-STROEME zu Beispiel-
    quelltexten; ein Keyword, das in keinem Beispiel vorkommt, faellt
    ihnen nicht auf. Dieser Test vergleicht die Tabellen selbst.
    """
    quelle = (WURZEL / "rust" / "drachenhauch_runtime" / "src" / "lexer.rs").read_text(encoding="utf-8")
    block = quelle.split("pub(crate) fn keyword")[1].split("_ => return None")[0]
    dhrt = set(re.findall(r'"([a-z_0-9]+)"\s*=>', block))
    py = {k.lower() for k in KEYWORDS}
    assert py - dhrt == set(), f"nur im Editor: {sorted(py - dhrt)}"
    assert dhrt - py == set(), f"nur in dhrt: {sorted(dhrt - py)}"
