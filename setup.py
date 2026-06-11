"""Cython-Beschleuniger ENTFERNT -- dieser Build-Schritt ist nicht mehr noetig.

Frueher baute dieses Skript zwei cdef-Module (`array_native` = `_GBArray`,
`ecs_native` = Native-ECS), die den Python-Tree-Walker beschleunigten. Mit
Stufe B sind Tree-Walker und Python-Toolchain komplett geloescht (auch die
`.pyx`-Quellen liegen nicht mehr im Repo) -- die EINZIGE Runtime ist `gbrt`
(Rust/raylib), dort liegt die gesamte Performance.

Diese Datei bleibt als Wegweiser stehen, damit ein gewohnheitsmaessiges
`python setup.py build_ext` eine klare Antwort bekommt statt eines Errors.

Zum Bauen der nativen Runtime: `.venv\\Scripts\\python.exe rust\\build_runtime.py`.
"""
import sys

if __name__ == "__main__":
    sys.stderr.write(
        "setup.py: Die Cython-Beschleuniger wurden entfernt -- kein Build noetig.\n"
        "Native Runtime bauen: python rust/build_runtime.py\n"
    )
    sys.exit(0)
