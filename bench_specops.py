"""Misst Native-VM-Laufzeit mit und ohne spezialisierte Numeric-Ops.

Compiliert dasselbe Programm zweimal: einmal mit aktivierter
Spezialisierungs-Tabelle (_NUMERIC_SPEC_OPS), einmal mit geleerter Tabelle
(Compiler emittiert nur generische OP_ADD/SUB/MUL/DIV/...). Beide Laeufe
durchlaufen dieselbe Native-VM, der einzige Unterschied ist der Bytecode.
"""
import sys
import time
from pathlib import Path

from gamebasic.preprocess import process
from gamebasic.lexer import Lexer
from gamebasic.parser import Parser
import gamebasic.compiler as compiler_mod
from gamebasic.compiler import Compiler
from gamebasic.vm_native import VM as NativeVM


def compile_program(path: Path):
    src = path.read_text(encoding="utf-8")
    processed, _ = process(src, path.parent)
    tokens = Lexer(processed).tokenize()
    ast = Parser(tokens).parse()
    return Compiler().compile(ast)


def time_native_run(module, runs: int) -> float:
    # Warmup
    for _ in range(2):
        NativeVM().run(module)
    t0 = time.perf_counter()
    for _ in range(runs):
        NativeVM().run(module)
    return (time.perf_counter() - t0) / runs * 1000.0  # ms


def bench(path: Path, runs: int = 3):
    # Mit Spezialisierung
    saved = dict(compiler_mod._NUMERIC_SPEC_OPS)
    mod_with = compile_program(path)
    spec_count = sum(
        1 for op, _ in mod_with.main.code
        if 100 <= int(op) <= 110
    )

    # Ohne Spezialisierung
    compiler_mod._NUMERIC_SPEC_OPS.clear()
    try:
        mod_without = compile_program(path)
    finally:
        compiler_mod._NUMERIC_SPEC_OPS.update(saved)

    t_with = time_native_run(mod_with, runs)
    t_without = time_native_run(mod_without, runs)
    speedup = t_without / t_with if t_with > 0 else float("inf")
    print(f"{path.name:30s}  {t_without:8.2f} ms  ->  {t_with:8.2f} ms   "
          f"speedup {speedup:5.2f}x   (spec-ops im main: {spec_count})")


def main():
    benches = [
        "bench_loop.gb",
        "bench_fib.gb",
        "bench_array_rw.gb",
        "bench_string_concat.gb",
        "bench_method_dispatch.gb",
        "bench_builtin_math.gb",
        "bench_ecs_movement.gb",
    ]
    print(f"{'Programm':30s}  {'OHNE spec':>11s}  ->  {'MIT spec':>11s}     Speedup")
    print("-" * 90)
    base = Path("examples")
    for name in benches:
        bench(base / name, runs=3)


if __name__ == "__main__":
    main()
