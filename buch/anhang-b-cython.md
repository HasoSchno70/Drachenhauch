# Anhang B — Performance: Cython-VM und Benchmarks

## Lernziele

Nach diesem Anhang kannst du:

- die drei Ausführungspfade unterscheiden: Tree-Walker, Python-VM, Cython-VM
- mit `gbrun.py --bench` alle drei Pfade vergleichen und die Output-Identität verifizieren
- die Cython-Native-VM aus dem Quelltext bauen
- entscheiden, wann es sich lohnt, die `--vm`-Option zu setzen

## Sektionen

1. Die drei Pfade — was tun sie, was sind die Tradeoffs
2. Bench-Equivalence — die "Alle identisch"-Garantie
3. Cython bauen: `setup.py build_ext --inplace`
4. Wann der Tree-Walker schneller ist (sehr kurze Skripte: Compile-Overhead)
5. Wann der Cython-Pfad spürbar hilft (lange Inner Loops, Bullet-/Particle-Massen)
6. Den eigenen Star-Pilot benchen — typische Zahlen für 200 Bullets + 50 Particles
7. Veröffentlichen: was muss der User installiert haben?

## Status

Skelett — wird in der iterativen Phase ausgearbeitet.
