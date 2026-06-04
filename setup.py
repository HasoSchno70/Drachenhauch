"""Build-Skript fuer die nativen Cython-Beschleuniger.

Verwendung:
    .venv\\Scripts\\python.exe setup.py build_ext --inplace

Baut die cdef-Klassen, die den Tree-Walker beschleunigen: `_GBArray`
(typed memoryviews) und die native ECS-World. Beide haben Pure-Python-
Fallbacks -- fehlt die `.pyd`, laeuft der Tree-Walker trotzdem (nur langsamer).

(Die fruehere Cython-VM `vm_native.pyx` wurde entfernt: die native Rust-Runtime
`gbrt` hat sie als schnellen/Produktions-Pfad abgeloest.)
"""
from setuptools import setup
from Cython.Build import cythonize

setup(
    name="gamebasic-native",
    ext_modules=cythonize(
        [
            "gamebasic/array_native.pyx",
            "gamebasic/modules/ecs_native.pyx",
        ],
        language_level=3,
        compiler_directives={
            "boundscheck": False,
            "wraparound": False,
            "initializedcheck": False,
            "cdivision": True,
        },
    ),
    zip_safe=False,
)
