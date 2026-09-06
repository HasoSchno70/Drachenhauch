"""Hover-Tooltips fuer Drachenhauch-Built-ins.

Format: ``name_lowercase -> (signature, beschreibung)``. Wird von
``CodeEditor`` beim Hover ueber einem Identifier konsultiert. Die Tabelle
liegt in `builtin_docs.json` (daneben) -- dieselbe Datei bettet dhrt fuer
`dhrt lsp` ein; hier wird sie nur geladen.
"""
from __future__ import annotations


import json
from functools import lru_cache
from pathlib import Path


def _handdoku() -> dict[str, tuple[str, str]]:
    """Die handgepflegte Tabelle aus `builtin_docs.json` -- EINE Datei fuer
    den Qt-Editor und fuer dhrt (`lsp.rs` bettet sie ein). Bis 2026-09-06
    stand sie als Python-Woerterbuch hier; der Sprachserver in Rust haette
    sie dann ein zweites Mal gebraucht."""
    try:
        pfad = Path(__file__).resolve().parent / "builtin_docs.json"
        roh = json.loads(pfad.read_text(encoding="utf-8")).get("docs", {})
        return {k: (str(v[0]), str(v[1])) for k, v in roh.items()}
    except Exception:
        return {}


BUILTIN_DOCS: dict[str, tuple[str, str]] = _handdoku()


@lru_cache(maxsize=1)
def _prosa() -> dict[str, str]:
    """Kurzbeschreibungen aus `docs/`, erzeugt nach `builtin_prosa.json`.

    Faellt die Datei aus (altes Paket, kaputtes JSON), bleibt es beim Stand
    von vorher: Hover zeigt dann nur die Signatur. Ein fehlender Zusatz darf
    den Editor nicht lahmlegen.
    """
    try:
        pfad = Path(__file__).resolve().parent / "builtin_prosa.json"
        return json.loads(pfad.read_text(encoding="utf-8")).get("docs", {})
    except Exception:
        return {}


def get_doc(name: str) -> tuple[str, str] | None:
    """Liefert (Signatur, Beschreibung) zu einem Built-in oder None.

    `name` kommt haeufig OHNE trailing `$` an -- `symbole::wort_bei` in dhrt
    strippt ihn z.B. per Konvention ("Wort ohne trailing $"), waehrend
    BUILTIN_DOCS $-Builtins MIT `$` als Key speichert (z.B. "str$"). Review-
    Fund: dieser Lookup versuchte bisher nur den Namen wie uebergeben --
    Hover fuer JEDES $-Builtin (STR$, LEFT$, MID$, CHR$, ...) lief dadurch
    ueber die LSP immer ins Leere. Der Qt-Editor selbst war unbetroffen
    (sein eigenes _word_at_cursor haelt das `$` im Identifier), daher hier:
    erst den Namen wie uebergeben versuchen, dann zusaetzlich mit
    angehaengtem `$`.
    """
    key = name.lower()
    doc = BUILTIN_DOCS.get(key) or BUILTIN_DOCS.get(key + "$")
    if doc is not None:
        return doc
    # Zweite Quelle: die Kurzbeschreibungen aus `docs/`, erzeugt von
    # `dhrt doku prosa`. Die Tabelle oben ist von Hand gepflegt und
    # deckte 328 von 1558 Builtins ab -- ganze Module standen bei null (gui mit
    # 161 Befehlen, g3d, m3d, chart, json, sprite, tiled), und der Hover fiel
    # dort auf die blosse Signatur zurueck. Die Beschreibungen existieren
    # laengst in den Modul-Dokumenten; sie hier ein zweites Mal zu tippen waere
    # eine Kopie, die auseinanderlaeuft.
    #
    # Vorrang hat die Tabelle oben: ihre Texte sind ausfuehrlicher und auf den
    # Hover zugeschnitten, die aus `docs/` sind Tabellenzellen und oft knapp.
    kurz = _prosa().get(name.upper()) or _prosa().get(name.upper() + "$")
    if kurz is not None:
        from .dhrt_meta import signature
        return (signature(name) or name.upper(), kurz)
    return None
