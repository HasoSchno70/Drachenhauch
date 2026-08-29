"""Builtin-Metadaten-Shim (Stufe B, Phase 2): eingefrorener builtin_index.json
+ dhrt_meta-Loader entkoppeln Editor/LSP von der Laufzeit (interpreter.py)."""
from drachenhauch.editor_qt import dhrt_meta


def test_index_loads_and_is_substantial():
    idx = dhrt_meta.builtin_index()
    assert len(idx) > 700                  # vollstaendige Registry-Union (~764)
    e = idx[0]
    assert {"name", "kind", "signature", "module"} <= set(e)


def test_core_and_graphics_and_dhrt_only_names_present():
    upper = set(dhrt_meta.builtin_names_upper())
    # Core, Grafik und dhrt-only (nur in builtin_docs) muessen alle dabei sein.
    for n in ("ABS", "SIN", "STR$", "LEN"):
        assert n in upper
    for n in ("LINE", "BOX", "DRAWIMAGE"):
        assert n in upper
    for n in ("ARRAY_PUSH", "STARTSWITH"):     # dhrt-only Erweiterungen
        assert n in upper


def test_names_lower_is_lowercased_set():
    low = dhrt_meta.builtin_names_lower()
    assert "abs" in low and "line" in low
    assert all(n == n.lower() for n in low)


def test_signature_lookup():
    assert dhrt_meta.signature("ABS")           # nicht leer
    assert dhrt_meta.signature("nicht_da_xyz") == ""


def test_by_module_grouping():
    groups = dhrt_meta.by_module()
    assert "core" in groups                     # Core-Builtins (kein IMPORT)
    # Modul-Builtins sind nach ihrem Modul gruppiert (z.B. g3d, json, vec2).
    assert any(m in groups for m in ("g3d", "json", "vec2"))


# ------------------------------------------------ Qualitaet der Signaturen
# Der Compiler leitet aus der Signatur die erlaubte Argumentzahl ab und warnt
# bei Abweichung (`dhrt --check`); Editor und LSP zeigen sie als Hover- und
# Signaturhilfe. Zwei Schreibweisen entziehen sich dieser Ableitung, und beide
# waren im Index vertreten:
#
#   NAME(*args)          65x -- praktisch der ganze 3D-Zweig, ein Erbe der
#                        Stufe-B-Migration. Argumentpruefung abgeschaltet,
#                        Hover zeigte nur "CUBE(*args)".
#   NAME(2+ Argumente)    5x -- sieht aus wie eine Angabe, ist aber keine, die
#                        `parse_arity` (compiler.rs) lesen kann. Die richtige
#                        Form fuer "beliebig viele" ist `NAME(a, b, ...)`.
#
# Beide sind aufgeloest; dieser Test haelt es fest. `*args` bleibt als Notausgang
# erlaubt, wenn die Argumentzahl WIRKLICH offen ist -- dann aber bitte bewusst.

def test_keine_args_platzhalter_mehr():
    platzhalter = [e["name"] for e in dhrt_meta.builtin_index()
                   if "(*args)" in (e.get("signature") or "")]
    assert not platzhalter, (
        f"{len(platzhalter)} Builtins ohne echte Signatur: {platzhalter[:8]} -- "
        "die Argumentzahl steht in vm.rs bzw. builtins.rs und gehoert hierher, "
        "sonst pruefen weder Compiler noch Editor den Aufruf.")


def test_keine_unlesbare_mindestangabe():
    import re
    schlecht = [e["name"] for e in dhrt_meta.builtin_index()
                if re.search(r"\(\s*\d+\+\s*Argument", e.get("signature") or "")]
    assert not schlecht, (
        f"'N+ Argumente' kann parse_arity nicht lesen: {schlecht} -- "
        "stattdessen die variadische Form `NAME(a, b, ...)` schreiben.")


def test_signatur_beginnt_mit_dem_eigenen_namen():
    """Sonst zeigt der Hover einen anderen Befehl an, als der Nutzer angeklickt
    hat -- ein Copy-Paste-Fehler, den man in 1558 Zeilen nicht sieht."""
    def passt(name: str, sig: str) -> bool:
        # `CHR` und `CHR$` sind zwei Eintraege mit derselben Signatur -- beide
        # Schreibweisen rufen dasselbe Builtin, das $ darf also abweichen.
        kopf = sig.split("(", 1)[0].strip().upper().rstrip("$")
        return kopf == name.upper().rstrip("$")
    falsch = [(e["name"], e["signature"]) for e in dhrt_meta.builtin_index()
              if not passt(e["name"], e.get("signature") or "")]
    assert not falsch, falsch[:10]
