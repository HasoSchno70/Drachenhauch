"""Build-Skript fuer die native VM.

Verwendung:
    .venv\\Scripts\\python.exe setup.py build_ext --inplace

Das Ergebnis (.pyd) wird neben gamebasic/vm_native.pyx abgelegt.
gbrun.py erkennt sie automatisch zur Laufzeit.
"""
from setuptools import setup
from Cython.Build import cythonize

setup(
    name="gamebasic-native",
    ext_modules=cythonize(
        [
            "gamebasic/vm_native.pyx",
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
