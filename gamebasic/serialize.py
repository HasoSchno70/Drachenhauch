"""Serialisierung von kompilierten GameBasic-`Module`s ins `.gbc`-Format.

Schritt 1 der Rust-Runtime-Migration (siehe docs/rust-runtime.md): das vom
`Compiler` erzeugte `Module` (bytecode.py) wird in ein selbstbeschreibendes,
sprach-unabhaengiges Format geschrieben, das die Rust-VM einlesen kann.

Format: JSON (debuggbar fuer den Spike; spaeter ggf. kompaktes Binaer-Format).
Die Bit-Identitaets-Garantie haengt daran, dass Werte verlustfrei und
*eindeutig getypt* serialisiert werden -- insbesondere muss INT von FLOAT und
BOOL unterscheidbar bleiben (in Python ist `bool` ein `int`-Subtyp, `1` und
`1.0` haben in GB unterschiedliche Semantik).

Wert-Encoding (`_enc`), rekursiv, eindeutig dekodierbar:
    None            -> null
    bool            -> {"b": true|false}     (VOR int pruefen!)
    int             -> <JSON-Zahl, ganzzahlig>
    float           -> {"f": <JSON-Zahl>}
    str             -> <JSON-String>
    tuple/list      -> [<enc>, ...]
    COMP_MARKER     -> {"comp": true}
    _FuncRef        -> {"funcref": "<name>"}

Code-Instruktionen: `[op:int, arg]`, wobei `arg` mit demselben `_enc` kodiert
wird. Die Rust-VM weiss pro Opcode, welche Struktur `arg` hat.
"""
from __future__ import annotations

import json
from pathlib import Path

from .bytecode import (
    COMP_MARKER,
    CompiledFunction,
    Module,
    VMClassInfo,
    VMFieldDecl,
)

GBC_FORMAT = "gbc"
GBC_VERSION = 1


def _enc(v):
    """Kodiert einen GB-Wert eindeutig nach JSON-faehigem Python."""
    if v is None:
        return None
    # bool VOR int -- isinstance(True, int) ist True.
    if isinstance(v, bool):
        return {"b": v}
    if isinstance(v, int):
        return v
    if isinstance(v, float):
        return {"f": v}
    if isinstance(v, str):
        return v
    if isinstance(v, (tuple, list)):
        return [_enc(x) for x in v]
    if v is COMP_MARKER:
        return {"comp": True}
    # _FuncRef ohne harten Import (vermeidet Zyklus): per Attribut erkennen.
    name = getattr(v, "name", None)
    if name is not None and type(v).__name__ == "_FuncRef":
        return {"funcref": name}
    # ENUM- und STATIC-CONST-Namespaces: liegen als Werte im const-Pool.
    # Beide haben `name` + `members` (lower-name -> Wert); MemberAccess loest
    # sie zur Laufzeit auf. Wir serialisieren beide gleich (Rust unterscheidet
    # nicht -- es ist nur ein Member-Lookup).
    if type(v).__name__ in ("_EnumNamespace", "_ClassStaticNamespace"):
        return {"ns": {"name": v.name, "members": {k: _enc(val) for k, val in v.members.items()}}}
    raise TypeError(
        f"serialize: nicht serialisierbarer Wert {v!r} (Typ {type(v).__name__}). "
        f"Vermutlich ein Laufzeit-Handle (IMAGE/SOUND/MAP/ARRAY/Instanz) im "
        f"const-Pool -- diese sind im .gbc-Spike noch nicht unterstuetzt."
    )


def _enc_func(fn: CompiledFunction) -> dict:
    return {
        "name": fn.name,
        "n_params": fn.n_params,
        "n_required": fn.n_required,
        "is_variadic": fn.is_variadic,
        "is_sub": fn.is_sub,
        "is_main": fn.is_main,
        "return_type": fn.return_type,
        "param_defaults": [_enc(x) for x in fn.param_defaults],
        "param_names": list(fn.param_names),
        "local_types": list(fn.local_types),
        "local_defaults": [_enc(x) for x in fn.local_defaults],
        "constants": [_enc(c) for c in fn.constants],
        # Instruktionen: [op, arg]. Op ist bereits ein IntEnum-Wert.
        "code": [[int(op), _enc(arg)] for (op, arg) in fn.code],
    }


def _enc_field(fd: VMFieldDecl) -> dict:
    return {
        "name": fd.name,
        "type_name": fd.type_name,
        "array_dims": list(fd.array_dims),
    }


def _enc_class(ci: VMClassInfo) -> dict:
    return {
        "name": ci.name,
        "parent_name": ci.parent_name,
        "is_struct": ci.is_struct,
        "fields": [_enc_field(f) for f in ci.fields],
        "methods": {name: _enc_func(m) for name, m in ci.methods.items()},
        "properties": sorted(ci.properties),
    }


def serialize_module(module: Module) -> dict:
    """Wandelt ein `Module` in die JSON-faehige `.gbc`-Struktur."""
    return {
        "format": GBC_FORMAT,
        "version": GBC_VERSION,
        "n_globals": module.n_globals,
        "main": _enc_func(module.main),
        "functions": {name: _enc_func(fn) for name, fn in module.functions.items()},
        "classes": {name: _enc_class(ci) for name, ci in module.classes.items()},
        "data": [_enc(d) for d in module.data],
    }


def dump_gbc(module: Module, path: str | Path, *, indent: int | None = None) -> None:
    """Schreibt das Modul als `.gbc`-Datei.

    `indent=2` fuer menschenlesbares Debugging; `None` (Default) fuer kompakt.
    """
    obj = serialize_module(module)
    text = json.dumps(obj, indent=indent, allow_nan=True)
    Path(path).write_text(text, encoding="utf-8")


def compile_file_to_gbc(src_path: str | Path, out_path: str | Path,
                        *, indent: int | None = None) -> Module:
    """End-to-End: `.gb`-Quelldatei -> Compiler -> `.gbc`-Datei.

    Nutzt denselben Preprocess+Lex+Parse+Compile-Pfad wie `gbrun.py --vm`.
    Gibt das kompilierte `Module` zurueck (praktisch fuer Tests/Bench).
    """
    from .lexer import Lexer
    from .parser import Parser
    from .compiler import Compiler
    from .preprocess import process as _preprocess
    # WICHTIG: interpreter.py importieren, damit alle @builtin-Decorators
    # laufen und die BUILTINS-Registry fuellen. Der Compiler entscheidet
    # anhand dieser Registry, ob ein Identifier-Call zu CALL_BUILTIN wird
    # (statt LOAD_NAME + CALL_VALUE). Ohne diesen Import erzeugt er fuer
    # LEN/INT/... falschen Bytecode -- gbrun.py importiert interpreter am
    # Modul-Anfang, deshalb stimmt dort der Pfad. Gleiche Umgebung herstellen.
    from . import interpreter as _interp  # noqa: F401

    src_path = Path(src_path)
    source = src_path.read_text(encoding="utf-8")
    source, _origins = _preprocess(source, src_path.parent, file_label=src_path.name)
    ast = Parser(Lexer(source).tokenize()).parse()
    module = Compiler().compile(ast)
    dump_gbc(module, out_path, indent=indent)
    return module


def _main(argv=None) -> int:
    import sys

    argv = argv if argv is not None else sys.argv
    args = argv[1:]
    indent = None
    if args and args[0] == "--pretty":
        indent = 2
        args.pop(0)
    if len(args) < 1:
        print("Verwendung: py -m gamebasic.serialize [--pretty] <datei.gb> [out.gbc]")
        return 1
    src = Path(args[0])
    out = Path(args[1]) if len(args) > 1 else src.with_suffix(".gbc")
    compile_file_to_gbc(src, out, indent=indent)
    print(f"{src.name} -> {out}  ({out.stat().st_size} bytes)")
    return 0


if __name__ == "__main__":
    import sys
    sys.exit(_main())
