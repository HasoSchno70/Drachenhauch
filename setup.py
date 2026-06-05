"""Cython-Beschleuniger ENTFERNT -- dieser Build-Schritt ist nicht mehr noetig.

Frueher baute dieses Skript zwei cdef-Module (`array_native` = `_GBArray`,
`ecs_native` = Native-ECS), die den Tree-Walker beschleunigten. Beide wurden
entfernt: der Tree-Walker ist nur noch Editor-/Referenzpfad, die Performance
liegt in der nativen Runtime `gbrt` (Rust). `_GBArray` (inline in
`interpreter.py`) und die ECS-`_World`/`_Component` (`modules/ecs_py.py`) laufen
jetzt als reine Python-Klassen.

(Die noch fruehere Cython-VM `vm_native.pyx` wurde bereits davor entfernt --
`gbrt` hat sie als Produktionspfad abgeloest. pygame wurde ENTFERNT: Grafik/
Audio laufen in der nativen Runtime `gbrt`; der Tree-Walker ist konsolen-only.)

Zum Bauen der nativen Runtime: `.venv\\Scripts\\python.exe rust\\build_runtime.py`.
"""
import sys

if __name__ == "__main__":
    sys.stderr.write(
        "setup.py: Die Cython-Beschleuniger wurden entfernt -- kein Build noetig.\n"
        "Native Runtime bauen: python rust/build_runtime.py\n"
    )
    sys.exit(0)
