# Partikel-Editor

Standalone-Tool zum visuellen Tunen des [`particles`](README.md#module)-Moduls — Parameter live per Slider einstellen, Echtzeit-Vorschau, und das Ergebnis als GB-Code exportieren.

## Starten

```
gbparticles
```

oder

```
.venv\Scripts\python.exe gbrun.py --particles
```

(Braucht `PySide6` und `numpy`.)

## Bedienung

**Links** die Parameter (live in die Vorschau übernommen):

- **Bewegung** — `vx/vy min/max` (Start-Geschwindigkeit, px/s), `Gravity x/y` (px/s²).
- **Aussehen** — **Modus** (`circle` / `pixel` / `square` / `streak` / `glow`), Größe min/max, **Farbe**, optionaler **Farbverlauf** zu einer End-Farbe (über die Lebenszeit interpoliert, z. B. Feuer gelb→rot), **Fade** (am Lebensende abdunkeln).
- **Lebenszeit & Emission** — Lebensdauer min/max (ms), Emission pro Frame.

**Rechts** die Echtzeit-Vorschau. Sie treibt eine echte `_ParticleSystem`-Instanz — **dasselbe Simulationsmodell wie die Engine**, also entspricht die Vorschau exakt dem späteren Verhalten im Spiel (inkl. additivem `glow`).

**Unten:** `Pause` friert die Simulation ein, `Leeren` entfernt alle Partikel, **`GB-Code exportieren`** öffnet ein Fenster mit dem fertigen `PARTICLE_*`-Setup-Snippet und kopiert es auf Wunsch in die Zwischenablage.

## Beispiel-Export

```basic
IMPORT "particles"

DIM ps AS PARTICLE_SYSTEM
ps = PARTICLE_SYSTEM_NEW(160, 120)
PARTICLE_SET_VELOCITY(ps, -80, 80, -160, -60)
PARTICLE_SET_LIFETIME(ps, 700, 1400)
PARTICLE_SET_GRAVITY(ps, 0, 120)
PARTICLE_SET_SIZE(ps, 2, 4)
PARTICLE_SET_COLOR(ps, &HFFDD33)
PARTICLE_SET_COLOR_END(ps, &HFF2000)
PARTICLE_SET_FADE(ps, TRUE)
PARTICLE_SET_MODE(ps, "glow")

' --- im Game-Loop ---
' PARTICLE_EMIT(ps, 8)
' PARTICLE_UPDATE(ps, 16)
' PARTICLE_DRAW(ps)
```

Den `PARTICLE_*`-Setup-Block ins Programm kopieren, die drei Loop-Zeilen in den Game-Loop übernehmen — fertig.
