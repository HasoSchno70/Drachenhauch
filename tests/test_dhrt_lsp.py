"""`dhrt lsp` -- der Sprachserver in Rust (Weg A aus docs/entwurf-python-abbau.md).

Ersetzt `tests/test_lsp_features.py` und `tests/test_lsp_server.py`, die den
Python-Server pruefte. Hier laeuft der ECHTE Prozess ueber stdio mit
Content-Length-Rahmung -- so, wie VS Code ihn startet. Jede Zusicherung der
alten Tests steht hier wieder: Faehigkeiten, Diagnose bei didOpen und
didChange, Definition, Hover (eigene Funktion, Builtin, `$`-Builtin, nur
Index), Vervollstaendigung (Praefix, eigene Symbole, Keywords), Fundstellen,
Gliederung, unbekanntes Verfahren, kaputte Rahmung, shutdown/exit.
"""
from __future__ import annotations

import json
import os
import subprocess
import time
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


def _dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    for v in ("release", "debug"):
        p = _ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
        if p.exists():
            return p
    return None


_DHRT = _dhrt()
pytestmark = pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")

SRC = (
    "' Spieler-Klasse\n"           # 1
    "CLASS Player\n"               # 2
    "    DIM hp AS INTEGER\n"      # 3
    "    SUB Init()\n"            # 4
    "        Self.hp = 100\n"     # 5
    "    END SUB\n"              # 6
    "END CLASS\n"               # 7
    "FUNCTION add(a AS INTEGER, b AS INTEGER) AS INTEGER\n"  # 8
    "    RETURN a + b\n"          # 9
    "END FUNCTION\n"            # 10
    "DIM result AS INTEGER\n"     # 11
    "result = add(1, 2)\n"       # 12
)
URI = "file:///tmp/test.dh"


class Server:
    """Ein `dhrt lsp`-Prozess mit gerahmtem Senden und Empfangen."""

    def __init__(self):
        self.p = subprocess.Popen([str(_DHRT), "lsp"], stdin=subprocess.PIPE,
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.naechste_id = 1
        self.eingang: list[dict] = []

    def roh(self, daten: bytes, kopf: bytes | None = None):
        if kopf is None:
            kopf = f"Content-Length: {len(daten)}\r\n\r\n".encode("ascii")
        self.p.stdin.write(kopf + daten)
        self.p.stdin.flush()

    def melden(self, methode, params):
        self.roh(json.dumps({"jsonrpc": "2.0", "method": methode, "params": params}).encode("utf-8"))

    def fragen(self, methode, params, timeout=10.0):
        i = self.naechste_id
        self.naechste_id += 1
        self.roh(json.dumps({"jsonrpc": "2.0", "id": i, "method": methode, "params": params}).encode("utf-8"))
        ende = time.monotonic() + timeout
        while time.monotonic() < ende:
            m = self.lesen(timeout)
            if m is None:
                break
            if m.get("id") == i:
                return m
        raise AssertionError(f"keine Antwort auf {methode}")

    def lesen(self, timeout=10.0):
        # Blockierendes Lesen -- der Test setzt Zeitgrenzen ueber die Gesamtdauer.
        kopf = {}
        while True:
            zeile = self.p.stdout.readline()
            if not zeile:
                return None
            zeile = zeile.decode("ascii", "replace").strip()
            if not zeile:
                break
            k, _, v = zeile.partition(":")
            kopf[k.strip().lower()] = v.strip()
        n = int(kopf["content-length"])
        m = json.loads(self.p.stdout.read(n).decode("utf-8"))
        self.eingang.append(m)
        return m

    def diagnose_abwarten(self, timeout=10.0):
        ende = time.monotonic() + timeout
        while time.monotonic() < ende:
            m = self.lesen(timeout)
            if m is None:
                break
            if m.get("method") == "textDocument/publishDiagnostics":
                return m["params"]["diagnostics"]
        raise AssertionError("keine publishDiagnostics-Meldung")

    def oeffnen(self, text=SRC, uri=URI):
        self.melden("textDocument/didOpen",
                    {"textDocument": {"uri": uri, "languageId": "drachenhauch", "version": 1, "text": text}})

    def schliessen(self):
        try:
            self.fragen("shutdown", {})
            self.melden("exit", {})
            self.p.wait(timeout=5)
        except Exception:
            self.p.kill()
        return self.p.returncode


@pytest.fixture
def srv():
    s = Server()
    init = s.fragen("initialize", {})
    assert init["result"]["capabilities"]["hoverProvider"] is True
    yield s
    s.schliessen()


def test_faehigkeiten_und_sauberes_ende():
    s = Server()
    caps = s.fragen("initialize", {})["result"]["capabilities"]
    assert caps["definitionProvider"] is True
    assert caps["documentSymbolProvider"] is True
    assert caps["referencesProvider"] is True
    assert caps["completionProvider"]["triggerCharacters"] == ["."]
    assert s.schliessen() == 0, "shutdown + exit muss den Prozess beenden"


def test_didopen_meldet_fehler_mit_zeile(srv):
    srv.oeffnen("PRINT 1\nDIM x AS\n")
    d = srv.diagnose_abwarten()
    assert len(d) == 1
    assert d[0]["severity"] == 1
    assert d[0]["source"] == "drachenhauch"
    assert d[0]["range"]["start"]["line"] == 1


def test_didopen_sauber_und_didchange_prueft_neu(srv):
    srv.oeffnen()
    assert srv.diagnose_abwarten() == []
    srv.melden("textDocument/didChange",
               {"textDocument": {"uri": URI, "version": 2}, "contentChanges": [{"text": "DIM x AS\n"}]})
    assert len(srv.diagnose_abwarten()) == 1
    srv.melden("textDocument/didClose", {"textDocument": {"uri": URI}})
    assert srv.diagnose_abwarten() == []


def test_import_fehler_landet_in_der_puffer_zeile(srv, tmp_path):
    """dhrt meldet gemergte Zeilen; der Server rechnet auf den Puffer zurueck.
    Ohne das rutschte jeder Marker um die Laenge des Inlinierten."""
    (tmp_path / "helfer.dh").write_text("SUB gruss()\n    PRINT 1\nEND SUB\n", encoding="utf-8")
    uri = (tmp_path / "haupt.dh").as_uri()
    srv.oeffnen('IMPORT "helfer.dh"\nPRINT 1\nDIM x AS\n', uri=uri)
    d = srv.diagnose_abwarten()
    assert len(d) == 1 and d[0]["range"]["start"]["line"] == 2, d


def test_definition_und_fundstellen(srv):
    srv.oeffnen()
    srv.diagnose_abwarten()
    loc = srv.fragen("textDocument/definition",
                     {"textDocument": {"uri": URI}, "position": {"line": 11, "character": 10}})["result"]
    assert loc["uri"] == URI and loc["range"]["start"]["line"] == 7
    assert loc["range"]["start"]["character"] == 9
    keine = srv.fragen("textDocument/definition",
                       {"textDocument": {"uri": URI}, "position": {"line": 0, "character": 2}})["result"]
    assert keine is None
    refs = srv.fragen("textDocument/references",
                      {"textDocument": {"uri": URI}, "position": {"line": 7, "character": 10},
                       "context": {"includeDeclaration": True}})["result"]
    zeilen = sorted(r["range"]["start"]["line"] for r in refs)
    assert zeilen == [7, 11]


def test_hover_eigene_funktion_und_builtins(srv):
    srv.oeffnen()
    srv.diagnose_abwarten()
    h = srv.fragen("textDocument/hover",
                   {"textDocument": {"uri": URI}, "position": {"line": 11, "character": 10}})["result"]
    assert h["contents"]["kind"] == "markdown"
    assert "FUNCTION add(a AS INTEGER, b AS INTEGER) AS INTEGER" in h["contents"]["value"]
    # Kommentar direkt ueber der Klasse ist ihre Doku.
    h = srv.fragen("textDocument/hover",
                   {"textDocument": {"uri": URI}, "position": {"line": 1, "character": 8}})["result"]
    assert "Spieler-Klasse" in h["contents"]["value"]
    leer = srv.fragen("textDocument/hover",
                      {"textDocument": {"uri": URI}, "position": {"line": 11, "character": 7}})["result"]
    assert leer is None


@pytest.mark.parametrize("quelle, spalte, erwartet", [
    ("DIM x AS INTEGER\nx = ABS(-5)\n", 5, "ABS"),
    ("DIM s AS STRING\ns = STR$(5)\n", 6, "STR$"),          # $-Builtin, Wort ohne $
    ("DIM x AS INTEGER\nx = MODEL_TEXTURE(1, 2)\n", 6, "MODEL_TEXTURE"),  # nur im Index
    ("DIM x AS INTEGER\nx = GUI_TABLE_SET_CELL(1, 2, 3, 4)\n", 8, "GUI_TABLE_SET_CELL"),  # Prosa aus docs/
])
def test_hover_builtins(srv, quelle, spalte, erwartet):
    srv.oeffnen(quelle)
    srv.diagnose_abwarten()
    h = srv.fragen("textDocument/hover",
                   {"textDocument": {"uri": URI}, "position": {"line": 1, "character": spalte}})["result"]
    assert h is not None and erwartet in h["contents"]["value"].upper()


def test_vervollstaendigung(srv):
    srv.oeffnen(SRC + "Pl")
    srv.diagnose_abwarten()
    items = srv.fragen("textDocument/completion",
                       {"textDocument": {"uri": URI}, "position": {"line": 12, "character": 2}})["result"]
    labels = [i["label"] for i in items]
    assert "Player" in labels
    assert all(l.lower().startswith("pl") for l in labels), labels
    srv.melden("textDocument/didChange",
               {"textDocument": {"uri": URI, "version": 2}, "contentChanges": [{"text": "PRI"}]})
    srv.diagnose_abwarten()
    items = srv.fragen("textDocument/completion",
                       {"textDocument": {"uri": URI}, "position": {"line": 0, "character": 3}})["result"]
    labels = {i["label"] for i in items}
    assert "PRINT" in labels and "PRIVATE" in labels
    assert all(l.lower().startswith("pri") for l in labels)


def test_vervollstaendigung_kennt_alle_schluesselwoerter(srv):
    """Die Keyword-Liste in lexer.rs gegen die in tokens.py -- zwei Tabellen,
    die auseinanderlaufen koennten; die Vervollstaendigung zeigt es."""
    from drachenhauch.tokens import KEYWORDS
    srv.oeffnen("")
    srv.diagnose_abwarten()
    items = srv.fragen("textDocument/completion",
                       {"textDocument": {"uri": URI}, "position": {"line": 0, "character": 0}})["result"]
    labels = {i["label"] for i in items}
    fehlen = sorted(k.upper() for k in KEYWORDS if k.upper() not in labels)
    assert not fehlen, fehlen
    assert "KEY_SPACE" in labels and "RED" in labels and "PI" in labels


def test_gliederung(srv):
    srv.oeffnen(SRC + "ENUM State\n  A = 0\nEND ENUM\n")
    srv.diagnose_abwarten()
    syms = srv.fragen("textDocument/documentSymbol", {"textDocument": {"uri": URI}})["result"]
    by = {s["name"]: s for s in syms}
    assert set(by) >= {"Player", "add", "State"}
    assert [c["name"] for c in by["Player"]["children"]] == ["Init"]
    assert by["Player"]["kind"] == 5 and by["State"]["kind"] == 10
    assert by["Player"]["range"]["end"]["line"] == 6


def test_unbekanntes_verfahren_und_kaputte_rahmung(srv):
    assert srv.fragen("textDocument/foobar", {})["result"] is None
    # Ein Kopf ohne Content-Length kostet nur diese Nachricht, nicht die
    # Sitzung. (Ohne Laenge kann der Server keinen Rumpf ueberspringen --
    # darum kommt hier auch keiner; ein Client, der das taete, haette den
    # Strom fuer beide Seiten verdorben.)
    srv.roh(b"", kopf=b"X-Custom-Header: 5\r\n\r\n")
    # Ein JSON-Wert, der kein Objekt ist, wird uebergangen.
    srv.roh(b"[]")
    assert srv.fragen("textDocument/foobar", {})["result"] is None
