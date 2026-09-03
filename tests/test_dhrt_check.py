"""Stufe B, Phase 4-Vorbereitung: `dhrt --check` Diagnostik.

Verifiziert, dass dhrt gueltigen Code sauber meldet ([]) und die strukturellen
Compile-/Syntax-Fehler MIT Zeilennummer liefert (Voraussetzung, um die Editor/
LSP-Diagnostik vom Python-Compiler auf dhrt umzustellen). Skippt ohne dhrt.
"""
import json
import os
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent
_EXAMPLES = _ROOT / "examples"


def _find_dhrt():
    base = _ROOT / "rust" / "drachenhauch_runtime" / "target"
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    for variant in ("release", "debug"):
        p = base / variant / exe
        if p.exists():
            return p
    return None


_DHRT = _find_dhrt()
pytestmark = pytest.mark.skipif(_DHRT is None, reason="dhrt nicht gebaut")


def _check(tmp_path, src):
    f = tmp_path / "c.dh"
    f.write_text(src, encoding="utf-8")
    r = subprocess.run([str(_DHRT), "--check", str(f)],
                       capture_output=True, text=True, timeout=30)
    assert r.returncode == 0, r.stderr      # Exit 0 auch bei Diagnosen
    return json.loads(r.stdout)


def test_clean_program_no_diagnostics(tmp_path):
    assert _check(tmp_path, 'DIM x AS INTEGER\nx = 5\nPRINT x\n') == []


def test_parse_error_has_line(tmp_path):
    d = _check(tmp_path, 'PRINT "a"\nFOR i = 1 TO\n')
    assert len(d) == 1 and d[0]["phase"] == "parse" and d[0]["line"] == 2


def test_compile_error_break_outside_loop_has_line(tmp_path):
    d = _check(tmp_path, 'PRINT "a"\nBREAK\n')
    assert len(d) == 1
    assert d[0]["phase"] == "compile" and d[0]["line"] == 2
    assert "BREAK" in d[0]["message"]


def test_compile_error_return_outside_function_has_line(tmp_path):
    d = _check(tmp_path, 'PRINT 1\nRETURN 5\n')
    assert d and d[0]["phase"] == "compile" and d[0]["line"] == 2


def _check_viele(dateien):
    """`dhrt --check` EINMAL fuer alle Dateien -- statt einmal je Datei.

    Gemessen: die drei Sweeps unten riefen `--check` fuer jede der 185
    Beispieldateien einzeln auf, macht 555 Prozessstarts. Auf CI waren das
    32 s von 232 s Testzeit, und weil `--dist loadfile` die Tests einer Datei
    beim selben Arbeiter haelt, lagen sie alle auf demselben kritischen Pfad.

    Bei mehreren Dateien antwortet dhrt mit einer JSON-Zeile je Datei. Liefert
    {Pfad: [Diagnosen]}.

    Der Schluessel ist der VOLLE Pfad, nicht der Dateiname: ueber das ganze
    Repo gesehen gibt es Basisnamen doppelt (`fenster.dh`, `politur.dh`), und
    die zweite Datei haette die erste stillschweigend ueberschrieben -- also
    genau die eine, deren Diagnose keiner mehr sieht.
    """
    if not dateien:
        return {}
    # In Stuecken: Windows begrenzt die Kommandozeile auf rund 32 000 Zeichen.
    # Hier passen 189 Pfade zwar hinein, aber das haengt an der Laenge des
    # Projektpfads -- unter einem tieferen Ordner faellt es sonst um.
    zeilen = []
    stueck, laenge = [], 0
    for f in dateien:
        t = str(f)
        if stueck and laenge + len(t) + 3 > 24000:
            r = subprocess.run([str(_DHRT), "--check"] + stueck,
                               capture_output=True, text=True, timeout=300)
            zeilen.extend((r.stdout or "").splitlines())
            stueck, laenge = [], 0
        stueck.append(t)
        laenge += len(t) + 3
    if stueck:
        r = subprocess.run([str(_DHRT), "--check"] + stueck,
                           capture_output=True, text=True, timeout=300)
        zeilen.extend((r.stdout or "").splitlines())

    raus = {}
    for zeile in zeilen:
        zeile = zeile.strip()
        if not zeile:
            continue
        eintrag = json.loads(zeile)
        raus[eintrag["datei"]] = eintrag["probleme"]
    return raus


def test_all_examples_check_clean():
    """Kein Fehlalarm: jedes gueltige Beispiel meldet keine *Errors*
    (Null-False-Positive). Warnungen (z.B. fehlendes Hardware-Modul im
    Default-Build, siehe Hardware-Beispiele 35-38) sind erlaubt -- sie sind
    keine Fehler, sondern ein bewusster Hinweis."""
    dateien = [f for f in sorted(_EXAMPLES.glob("*.dh"))
               if "_smoketest" not in f.name]
    bad = [(Path(pfad).name, [d for d in diags if d.get("severity") != "warning"])
           for pfad, diags in _check_viele(dateien).items()
           if any(d.get("severity") != "warning" for d in diags)]
    assert not bad, f"Fehlalarme bei gueltigem Code: {bad}"


def test_examples_use_no_unknown_builtin():
    """Drift-Schutz fuer builtin_index.json: KEIN Beispiel ruft ein Builtin auf,
    das dhrt nicht kennt. Schlaegt fehl, sobald ein neues dhrt-Builtin benutzt,
    aber nicht im Index ergaenzt wurde (-> der Index bleibt vollstaendig, sonst
    wuerde gueltiger Code faelschlich die 'Unbekanntes Builtin'-Warnung kriegen).
    Greift Hand in Hand mit compiler::is_known_builtin (G1, systemisch)."""
    dateien = [f for f in sorted(_EXAMPLES.rglob("*.dh"))
               if "_smoketest" not in f.name]
    drift = [(Path(pfad).name, d.get("line"), d.get("message"))
             for pfad, diags in _check_viele(dateien).items()
             for d in diags if "Unbekanntes Builtin" in d.get("message", "")]
    assert not drift, (
        "Beispiele nutzen Builtins, die dhrt nicht (im builtin_index.json) "
        f"kennt -> Index ergaenzen: {drift}")


def test_kein_beispiel_meldet_einen_unbekannten_namen():
    """Null-Falschmeldung fuer die "nirgends deklariert"-Warnung.

    Sie ist die einzige Warnung, die aus einer SCHAETZUNG kommt: der Compiler
    sammelt vorab alle Namen ein, die zur Laufzeit im globalen Verzeichnis
    stehen werden (`sammle_bekannte_namen` in compiler.rs), und meldet alles
    andere. Uebersieht dieser Vorlauf eine Deklarationsform, warnt er vor
    gueltigem Code -- und eine Warnung, die man wegsehen muss, ist schlimmer
    als gar keine.

    Deshalb geht dieser Test ueber ALLE .dh-Dateien des Repos, nicht nur ueber
    examples/: die Piloten, die Demos und die Buch-Beispiele decken zusammen
    deutlich mehr Sprachformen ab als ein Test, den man von Hand schreibt.
    Beim Einbau waren es 384 Dateien mit null Treffern.

    Die Gegenprobe -- dass die Warnung ueberhaupt anschlaegt -- steht in
    tests/test_compiler_warnungen.py; ohne sie waere ein stummgeschaltetes
    Feature hier ebenfalls gruen.
    """
    from drachenhauch.editor_qt import tempdateien
    dateien = [f for f in sorted(_ROOT.rglob("*.dh"))
               if "target" not in f.parts
               and not f.name.startswith(tempdateien.PRAEFIX)]
    assert len(dateien) > 300, f"nur {len(dateien)} .dh-Dateien gefunden -- Sweep leer?"
    funde = [(Path(pfad).name, d.get("line"), d.get("message"))
             for pfad, diags in _check_viele(dateien).items()
             for d in diags if "nirgends deklariert" in d.get("message", "")]
    assert not funde, f"Falschmeldung bei gueltigem Code: {funde}"


def test_hardware_import_warns_at_import(tmp_path):
    """E1: `IMPORT "wifi"` (serial/usb/bt analog) wird im Default-Build (ohne
    --hardware) schon beim IMPORT als Warnung gemeldet -- nicht erst beim ersten
    Funktionsaufruf zur Laufzeit. Ein Hardware-Build meldet stattdessen nichts;
    beide Faelle sind gueltig."""
    d = _check(tmp_path, 'IMPORT "wifi"\nPRINT 1\n')
    if d:  # Default-Build: genau eine Warnung auf der IMPORT-Zeile
        assert len(d) == 1, d
        w = d[0]
        assert w["severity"] == "warning"
        assert w["line"] == 1
        assert "wifi" in w["message"].lower()
        assert "--hardware" in w["message"]


def test_unknown_builtin_warns(tmp_path):
    """G1 (systemisch): Aufruf eines Builtins, das dhrt nicht kennt (Tippfehler
    oder nur-Tree-Walker wie frueher FLT), wird schon von --check als Warnung
    gemeldet -- nicht erst zur Laufzeit."""
    d = _check(tmp_path, 'DIM x AS INTEGER\nx = NOTAREALBUILTIN(5)\n')
    assert len(d) == 1, d
    w = d[0]
    assert w["severity"] == "warning"
    assert w["phase"] == "compile"
    assert w["line"] == 2
    assert "NOTAREALBUILTIN" in w["message"]


def test_known_builtin_no_warning(tmp_path):
    """Echte Builtins (inkl. FLT) loesen KEINE Warnung aus."""
    assert _check(tmp_path, 'DIM x AS FLOAT\nx = FLT(3)\nPRINT INT(x)\n') == []


# ------------------------------------ doppeltes DIM mit unterschiedlichem Typ

def _dim_warnungen(tmp_path, src):
    return [d for d in _check(tmp_path, src)
            if d.get("severity") == "warning" and "schon als" in d.get("message", "")]


def test_zweites_dim_mit_anderem_typ_warnt(tmp_path):
    """Ein Name, zwei Typen -- der Fehler faellt sonst erst weit entfernt auf
    ("Array-Index muss INTEGER sein, erhalten FLOAT") und ist dort nicht mehr
    seiner Ursache zuzuordnen."""
    w = _dim_warnungen(tmp_path, "DIM t AS INTEGER\nDIM t AS FLOAT\nPRINT t\n")
    assert len(w) == 1, w
    m = w[0]["message"]
    assert "INTEGER" in m and "FLOAT" in m
    # Seit WP I.4 nennt die Meldung die Stelle als `datei:zeile` statt als
    # blosse "Zeile N" -- bei IMPORT zeigte die nackte Zahl in die GEMERGTE
    # Quelle und damit oft auf eine unbeteiligte Zeile.
    assert ":1" in m, m                 # zeigt auf die ERSTE Deklaration ...
    assert w[0]["line"] == 2            # ... und steht bei der zweiten


def test_zweites_dim_mit_gleichem_typ_bleibt_still(tmp_path):
    """DIM im Schleifenkoerper ist gaengig und voellig in Ordnung -- eine
    Warnung dafuer waere Laerm, der die echten uebertoent."""
    assert _dim_warnungen(tmp_path, "DIM i AS INTEGER\nDIM i AS INTEGER\nPRINT i\n") == []
    assert _dim_warnungen(tmp_path,
        "DIM n AS INTEGER\nFOR n = 1 TO 3\n    DIM k AS INTEGER\n    k = n\nNEXT\nPRINT k\n") == []


def test_dim_warnung_ignoriert_die_schreibweise(tmp_path):
    """Drachenhauch unterscheidet keine Gross-/Kleinschreibung -- `Wert` und
    `wert` sind DIESELBE Variable, die Warnung muss das auch so sehen."""
    w = _dim_warnungen(tmp_path, 'DIM Wert AS INTEGER\nDIM wert AS STRING\nPRINT Wert\n')
    assert len(w) == 1, w


def test_dim_warnung_auch_innerhalb_einer_funktion(tmp_path):
    """Dort ist die Wirkung eine andere (die ERSTE Deklaration gewinnt, der
    zweite Typ wird verworfen), aber genauso still und genauso falsch."""
    w = _dim_warnungen(tmp_path,
        "SUB f()\n    DIM a AS INTEGER\n    DIM a AS FLOAT\n    PRINT a\nEND SUB\nf()\n")
    assert len(w) == 1, w
    assert "Funktion" in w[0]["message"], w[0]["message"]


def test_lokales_dim_darf_ein_global_verdecken(tmp_path):
    """Verdecken ueber Geltungsbereichs-Grenzen hinweg ist erlaubt und ein
    gaengiges Muster -- kein Fehlalarm."""
    assert _dim_warnungen(tmp_path,
        "DIM a AS INTEGER\nSUB f()\n    DIM a AS FLOAT\n    PRINT a\nEND SUB\nf()\nPRINT a\n") == []


def test_array_gegen_skalar_warnt_lesbar(tmp_path):
    """Der Typ wird so geschrieben, wie er im Quelltext steht -- die interne
    Form (`array:integer`) sagt einem Nutzer nichts."""
    w = _dim_warnungen(tmp_path, "DIM z[3] AS INTEGER\nDIM z AS INTEGER\nPRINT z\n")
    assert len(w) == 1, w
    assert "ARRAY OF INTEGER" in w[0]["message"], w[0]["message"]


def test_kein_beispiel_loest_die_dim_warnung_aus():
    """Eine neue Warnung, die auf dem eigenen Bestand losgeht, ist Laerm."""
    laut = [f"{name}:{d.get('line')}"
            for name, diags in _check_viele(sorted(_EXAMPLES.rglob("*.dh"))).items()
            for d in diags
            if d.get("severity") == "warning" and "schon als" in d.get("message", "")]
    assert laut == [], laut


def test_check_mit_einer_datei_bleibt_ein_array(tmp_path):
    """Rueckwaertskompatibel: der Editor (`error_check.py`) erwartet bei EINER
    Datei genau das Diagnose-Array, kein Objekt. Das bleibt so."""
    f = tmp_path / "a.dh"
    f.write_text("PRINT 1\n", encoding="utf-8")
    r = subprocess.run([str(_DHRT), "--check", str(f)],
                       capture_output=True, text=True, timeout=60)
    assert json.loads(r.stdout.strip()) == []


def test_check_mit_mehreren_dateien_nennt_die_datei(tmp_path):
    """Bei mehreren: eine JSON-Zeile je Datei, mit Namen -- sonst waere nicht
    zuzuordnen, welcher Fehler woher kommt."""
    gut = tmp_path / "gut.dh"
    gut.write_text("PRINT 1\n", encoding="utf-8")
    schlecht = tmp_path / "schlecht.dh"
    schlecht.write_text("DIM x AS\n", encoding="utf-8")
    r = subprocess.run([str(_DHRT), "--check", str(gut), str(schlecht)],
                       capture_output=True, text=True, timeout=60)
    zeilen = [json.loads(z) for z in r.stdout.splitlines() if z.strip()]
    nach_name = {Path(e["datei"]).name: e["probleme"] for e in zeilen}
    assert nach_name["gut.dh"] == []
    assert nach_name["schlecht.dh"], "der Parse-Fehler muss gemeldet werden"


def test_check_bricht_bei_einer_unlesbaren_datei_nicht_ab(tmp_path):
    """Eine fehlende Datei darf die anderen nicht um ihre Diagnose bringen."""
    gut = tmp_path / "gut.dh"
    gut.write_text("PRINT 1\n", encoding="utf-8")
    r = subprocess.run([str(_DHRT), "--check", str(tmp_path / "weg.dh"), str(gut)],
                       capture_output=True, text=True, timeout=60)
    namen = {Path(json.loads(z)["datei"]).name
             for z in r.stdout.splitlines() if z.strip()}
    assert namen == {"weg.dh", "gut.dh"}, namen
