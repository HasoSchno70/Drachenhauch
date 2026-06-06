"""Built-in-Module fuer GameBasic.

Built-in-Module sind Python-Dateien in diesem Paket. Sie registrieren ihre
Built-ins ueber den `@builtin`/`@graphics_builtin`-Decorator beim Import.

Auflosung in `IMPORT "<name>"` (siehe preprocess.py):

  IMPORT "json"            -> erst json.gb suchen, dann gamebasic/modules/json.py
  IMPORT "json.gb"         -> nur json.gb (Quellcode)
  IMPORT "subdir/lib.gb"   -> nur Quellcode, kein Built-in
"""
from __future__ import annotations

import importlib
import re
from typing import Set


# Modul-Namen muessen aus alphanumerischen Zeichen + Unterstrich bestehen.
# So vermeiden wir, dass z.B. "lib/foo" oder "../etc/passwd" als
# Modul-Name akzeptiert werden.
_MODULE_NAME_RE = re.compile(r"^[a-zA-Z][a-zA-Z0-9_]*$")

# Statische Liste der Built-in-Modul-Namen (Stufe B). Maßgeblich ist gbrts
# `rust/gb_runtime/src/preprocess.rs` MODULES -- gbrt implementiert die Module
# nativ. preprocess.py braucht NUR die Namen, um `IMPORT "<modul>"` als Built-in
# zu erkennen und zu einem Kommentar zu machen (kein Python-Impl-Laden mehr).
# Bei neuen Modulen hier UND in preprocess.rs ergänzen (synchron halten).
KNOWN_MODULES: frozenset = frozenset({
    "astar", "audio", "bt", "camera", "controller", "curves", "db", "ecs",
    "g3d", "gui", "html", "imgfx", "input", "json", "net", "particles",
    "physics", "regex", "save", "scene", "serial", "sprite", "tile_collide",
    "tiled", "tween", "ui", "usb", "vec2", "wifi",
})


def is_known_module(name: str) -> bool:
    """True, wenn `name` ein Built-in-Modul ist (statische Liste, kein Import)."""
    return name.lower() in KNOWN_MODULES


_loaded: Set[str] = set()

# Externe Typen, die Built-in-Module registrieren. Mappt den GB-Typ-Namen
# (lowercase, wie ihn der Lexer aus IDENTs liefert) auf eine Python-Klasse.
# Wird beim Type-Coerce in interpreter._coerce / vm._coerce konsultiert,
# nachdem alle Standard- und Klassentypen geprueft sind.
EXTERNAL_TYPES: dict = {}


def register_type(gb_name: str, py_class: type) -> None:
    """Registriert einen externen Typ. `gb_name` wird auf lowercase normalisiert."""
    EXTERNAL_TYPES[gb_name.lower()] = py_class


def call_funcref(g, ref, args):
    """Ruft eine FUNCREF (`_FuncRef`) aus einem Built-in heraus auf -- die
    Bruecke fuer Callback-APIs wie `GUI_ON_CLICK`.

    Funktioniert bit-identisch in allen drei Pfaden: die aktive Engine haengt
    als `g._gb_engine` am Graphics-Kontext und bietet eine einheitliche
    `gb_call_function(name, args)`-Methode (Tree-Walker, Python-VM, Native-VM).
    `args` sind vorab-evaluierte Werte; der Handler laeuft wie eine normale
    User-Function (sieht nur Params + Globals, keine Closures).
    """
    eng = getattr(g, "_gb_engine", None)
    if eng is None:
        from ..errors import GBRuntimeError
        raise GBRuntimeError(
            "FUNCREF-Callback: keine aktive Ausfuehrungsmaschine am "
            "Graphics-Kontext (g._gb_engine fehlt)")
    return eng.gb_call_function(ref.name, args)


# --- Operator-Registry ----------------------------------------------
#
# Module koennen fuer ihre eigenen Typen arithmetische Operatoren
# registrieren ("+", "-", "*", "/"). Tree-Walker und beide VMs konsultieren
# vor ihrem Standard-Dispatch diese Registry. Damit muss z.B. ein Matrix-
# oder Komplex-Modul keinen Code in interpreter.py / vm.py / vm_native.pyx
# mehr anpassen -- ein Aufruf in seinem Modul reicht.
#
# Lookup-Strategie: wenn type(a) registriert ist, dispatche zu dessen
# Handler-Tabelle; sonst type(b). Bei symmetrischen Operatoren wie + ist
# das egal, bei asymmetrischen (Skalar * Vec2) muss der Handler beide
# Reihenfolgen selbst akzeptieren -- siehe vec2._op_mul fuer das Pattern.
#
# Der Handler bekommt (a, b) und liefert entweder das Resultat oder wirft
# TypeMismatchError. Er bekommt NIE NO_OP_MATCH zurueck -- die Registry
# entscheidet binaer "passt / passt nicht".


class _NoOpMatch:
    """Sentinel: kein registrierter Operator passt -- Standard-Dispatch
    soll uebernehmen."""
    __slots__ = ()

    def __repr__(self):
        return "<no-op-match>"


NO_OP_MATCH = _NoOpMatch()

# (py_class) -> { op_name: handler(a, b) -> result }
OPERATORS_BY_TYPE: dict = {}


def register_operators(py_type: type, handlers: dict) -> None:
    """Registriert eine Operator-Handler-Tabelle fuer einen Python-Typ.

    Beispiel:
        register_operators(_Vec2, {"+": _op_add, "-": _op_sub, ...})
    """
    OPERATORS_BY_TYPE[py_type] = handlers


def dispatch_binary_op(op: str, a, b):
    """Schaut, ob `a` oder `b` zu einem registrierten Operator-Typ gehoeren.

    Liefert das Resultat, falls ein Handler fuer `op` registriert ist.
    Liefert NO_OP_MATCH, wenn weder type(a) noch type(b) eine Tabelle
    haben oder ihr Tabellen-Eintrag fehlt.
    """
    handlers = OPERATORS_BY_TYPE.get(type(a))
    if handlers is None:
        handlers = OPERATORS_BY_TYPE.get(type(b))
        if handlers is None:
            return NO_OP_MATCH
    h = handlers.get(op)
    if h is None:
        return NO_OP_MATCH
    return h(a, b)


def reset_for_tests(reload: bool = False) -> None:
    """Setzt globalen Module-State zurueck (fuer Test-Isolation).

    Loescht `_loaded`, `EXTERNAL_TYPES` und `OPERATORS_BY_TYPE`. Damit
    sieht der naechste Test eine "frische" Registry -- was z.B. erlaubt
    zu pruefen, dass Vec2 wirklich erst nach `IMPORT "vec2"` registriert
    ist.

    Mit `reload=True` werden zusaetzlich die importierten Modul-Objekte
    aus `sys.modules` entfernt, sodass `load_module(...)` die Decorator-
    Bodies erneut ausfuehrt und `EXTERNAL_TYPES`/`OPERATORS_BY_TYPE`
    konkret wieder gefuellt werden.

    **Vorsicht** mit `reload=True`: andere Test-Files, die Symbole aus
    `gamebasic.modules.<name>` direkt importiert haben (`from
    gamebasic.modules.vec2 import _Vec2`), halten dann Referenzen auf
    die ALTE Klassen-Definition. `isinstance(...)`-Checks gegen den neu
    importierten Typ schlagen dann fehl. Default ist daher `reload=False`.

    Ueblicher Test-Setup:

        @pytest.fixture(autouse=True)
        def _reset_modules():
            from gamebasic.modules import reset_for_tests
            reset_for_tests()
            yield
    """
    _loaded.clear()
    EXTERNAL_TYPES.clear()
    OPERATORS_BY_TYPE.clear()
    if reload:
        import sys
        for name in [n for n in sys.modules
                     if n.startswith("gamebasic.modules.")
                     and n != "gamebasic.modules.__init__"]:
            del sys.modules[name]


def is_valid_module_name(name: str) -> bool:
    return bool(_MODULE_NAME_RE.match(name))


def discover_modules() -> list:
    """Namen aller Built-in-Module (statische Liste KNOWN_MODULES, Stufe B).

    Früher per Dateisystem-Glob über modules/*.py -- die Impl-Dateien werden in
    Phase 8 gelöscht, gbrt implementiert die Module nativ. Der Editor-File-
    Browser nutzt das nur als deklarative Liste."""
    return sorted(KNOWN_MODULES)


def load_all_modules() -> list:
    """Laedt alle Built-in-Module eager. Gibt die Liste der erfolgreich
    geladenen Modul-Namen zurueck. Wird vom Editor genutzt, damit Autocomplete
    und Highlighting alle Modul-Built-ins kennen, ohne dass der User erst
    IMPORT tippen muss."""
    loaded = []
    for name in discover_modules():
        try:
            if load_module(name):
                loaded.append(name)
        except Exception:
            # Bei Modul-Fehler weiter machen - ein kaputtes Modul soll
            # den Editor nicht beerdigen.
            pass
    return loaded


def load_module(name: str, alias: str | None = None) -> bool:
    """Laedt das Built-in-Modul `name` (idempotent).

    Gibt True zurueck, wenn das Modul existiert (und damit geladen wurde
    bzw. bereits geladen war). False, wenn kein Modul mit dem Namen
    existiert. Andere Fehler (Syntaxfehler im Modul) werden durchgereicht.

    `alias`: optionaler Praefix-Alias. GameBasic-Module teilen einen
    flachen Built-in-Namespace; der Alias dupliziert alle Built-ins
    deren Name mit `<modul-name>_` anfaengt unter `<alias>_` -- plus den
    externen Typ falls er mit dem Modul-Namen uebereinstimmt:

        IMPORT "json" AS j     -> J_PARSE, J_LOAD, J_GET_INT, J_HANDLE, ...
        IMPORT "vec2" AS v     -> V_NEW, V_LENGTH, ...; auch DIM x AS V

    Konvention-basiert: trifft fuer die meisten Module (json, db, tween,
    vec2, sprite, particles*, ...). Module mit abweichendem Praefix
    (imgfx registriert IMAGE_*, nicht IMGFX_*) sind nicht aliasable --
    der Praefix-Match liefert dann leer und der Alias ist no-op.
    """
    if not is_valid_module_name(name):
        return False
    if name not in _loaded:
        try:
            importlib.import_module(f"gamebasic.modules.{name.lower()}")
        except ModuleNotFoundError as exc:
            if exc.name == f"gamebasic.modules.{name.lower()}":
                return False
            raise
        _loaded.add(name)
    if alias:
        _apply_alias_by_convention(name, alias)
    return True


def _apply_alias_by_convention(module_name: str, alias: str) -> None:
    """Dupliziert alle Built-ins / externen Typen die mit `module_name_`
    anfangen (oder == `module_name` sind) unter dem Alias-Praefix.

    Idempotent: wenn ein Alias-Key schon existiert, wird er nicht
    ueberschrieben. Wer denselben Alias zweimal nimmt, kriegt das
    Verhalten des ersten Aufrufs.
    """
    from .. import builtins_registry as _br
    n = module_name.lower()
    prefix = n + "_"
    alias_up = alias.upper()

    def remap(name: str) -> str:
        # `JSON_PARSE` (n="json") -> `j_parse`
        # `vec2` (n="vec2") -> `v` (single-word: kompletter Ersatz)
        if name == n:
            return alias_up.lower()
        if name.startswith(prefix):
            return f"{alias_up.lower()}_{name[len(prefix):]}"
        return ""  # nicht aliasable

    def alias_dict(d):
        for k in list(d):
            ak = remap(k)
            if ak and ak not in d:
                d[ak] = d[k]

    alias_dict(_br.BUILTINS)
    alias_dict(_br.GRAPHICS_BUILTINS)
    alias_dict(EXTERNAL_TYPES)
