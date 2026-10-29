# Particle editor (`examples/185_partikel_editor.dh`)

Adjust every parameter of the [`particles`](module-particles.md) module live,
with a real preview, and take the result with you as DH code — written in
Drachenhauch on its own `gui` module. The preview is **not an imitation**: it
drives a real `PARTICLE_SYSTEM` with the same calls that will later be in the
game, and draws it with `PARTICLE_DRAW`. What you see is what you get.

```
dhrt run examples/185_partikel_editor.dh
```

In the [IDE](ide.md) it is in the **Tools** menu and runs there as a separate
program. It does not need Python.

## The window

The window fills the screen and grows with it (at least 1300 × 820). The
interface writes in DejaVu Sans; at the top a menu and a toolbar, at the bottom
the status bar with the number of living particles and the history step.

| Area | What it does |
|---|---|
| Toolbar | Open, Save, DH code · Back/Forward · **Zufall** (random), **Mutieren** (mutate) · **Pause**/**Weiter**, **Leeren** (clear), **Salve** (burst — 200 particles at once, for explosions) |
| Presets (left) | Funken, Rauch, Feuer, Regen, Schnee, Explosion, Zauber, Springbrunnen, Blasen, Sterne, Staub, Glut (sparks, smoke, fire, rain, snow, explosion, magic, fountain, bubbles, stars, dust, embers) — one click sets all values, render mode and switches and clears the preview; below them **Zufall** and **Mutieren** |
| Background (left) | night, dusk (gradient), light, checkerboard — how the effect looks against different grounds; **Quelle folgt der Maus** puts the source under the pointer |
| Preview (centre) | The particles, clipped to the area; it grows with the window |
| Colours (below the preview) | Two colour pickers: **Farbe am Anfang** (start) and **Farbe am Ende** (end, with **Farbverlauf benutzen**) |
| History | Ctrl+Z/Ctrl+Y or the buttons in the bar |

**Zufall** (Ctrl+R) rolls an effect that looks like something: the values
come in pairs (min ≤ max), the end colour matches the start colour in hue.
**Mutieren** (Ctrl+M) moves every slider by at most 6 % of its range and the
colours by a few steps — the effect stays recognisably the same. Each is one
step in the history.

The sliders (right):

| Group | Sliders |
|---|---|
| Movement | speed X min/max, speed Y min/max (±400 px/s), force X/Y (±600 px/s²) |
| Lifetime (ms) and emission | life min/max (50..4000 ms), per frame (0..60 particles) |
| Appearance | render mode (circle, pixel, square, streak, glow — in code `circle`, `pixel`, `square`, `streak`, `glow`), size min/max, **am Ende ausblenden** (fade out at the end) |
| A particle over its life | A strip from birth (left) to the end (right): colour and, with fading, opacity on a checkerboard |

Without a gradient the system also gets the start colour as its end colour —
the module knows no "off". `PARTICLE_DRAW` currently draws `glow` like
`circle`; for real glowing in the game, wrap the call in `BLEND_MODE("add")`.

Everything also works without a mouse: TAB moves between controls, Space
triggers, the arrow keys adjust a slider; F5 pauses, F6 clears, F7 is the
burst.

## Undo

One **drag** on a slider is **one** step: it is only recorded once nothing
has changed for one frame and no mouse button is held any more. A drag
through a colour picker likewise. Preset, random, mutate and a loaded `.ini`
are one step each the same way. The full state is remembered (11 sliders, two
colours, render mode, two switches), up to 32 steps; a new drag after an undo
cuts off the way forward.

## Saving and loading

**Datei → Einstellung sichern** (Ctrl+S) writes all values into an `.ini`,
**Öffnen** (Ctrl+O) brings them back. The key is the value's name
(`Tempo X min`, …, `Rot Anfang`, `Rot Ende`, …), plus `modus` for the render
mode and `ausblenden`/`verlauf` for the two switches — readable and editable
by hand. An unknown key is skipped, a value outside the slider's range is
clamped. Until 2026-10-29 both colours were called `Rot`/`Gruen`/`Blau` (the
end colour overwrote the start colour); such a file still brings its end
colour along.

On closing (close button, Alt+F4 or ESC) the editor asks
(Sichern / Verwerfen / Abbrechen — Save / Discard / Cancel), but only if
something differs from the last save, load or choice of a preset.

## DH code

**DH-Code kopieren** (Copy DH code) puts the calls that create this system in
your own program onto the clipboard. The source appears there as `x, y` —
the game decides the position —, and you add the `IMPORT "particles"`
yourself. For the preset "Funken" (sparks), with a fixed position in front:

```basic
IMPORT "particles"

DIM x AS INTEGER
DIM y AS INTEGER
x = 160
y = 120

DIM sys AS PARTICLE_SYSTEM
sys = PARTICLE_SYSTEM_NEW(x, y)
PARTICLE_SET_VELOCITY(sys, -90, 90, -200, -60)
PARTICLE_SET_GRAVITY(sys, 0, 420)
PARTICLE_SET_LIFETIME(sys, 300, 800)
PARTICLE_SET_SIZE(sys, 2, 4)
PARTICLE_SET_COLOR(sys, RGB(255, 220, 80))
PARTICLE_SET_COLOR_END(sys, RGB(255, 60, 20))
PARTICLE_SET_MODE(sys, "circle")
PARTICLE_SET_FADE(sys, TRUE)

' per frame:
PARTICLE_EMIT(sys, 8)
PARTICLE_UPDATE(sys, 16)
PARTICLE_DRAW(sys)
```

The three lines after `' per frame:` belong in the frame loop.

## Compared with the old Qt version

The former Qt version (`dhparticles`) had a **preset library** with named
custom settings in one shared file and showed the DH code in a window of its
own. Here every custom setting is a separate `.ini`, and the code goes
straight to the clipboard.

## Tests

`tests/pruef/werkzeug_partikel.dhtest` drives real mouse clicks and keys from
a recording: one drag is one step, Back undoes the whole drag and Forward
brings it back, Ctrl+Z/Ctrl+Y do the same as the buttons, a preset is one
step, a new drag cuts off the way forward, and the first entry of the presets
is clickable (the buttons used to lie on top of it). Plus random (other
values, one step, pairs the right way round), mutate (close to the previous
one), a drag in the colour picker with undo, and the `.ini` with both colours
separate as well as an old file. The prompt on closing is
checked by `tests/pruef/werkzeug_kreuz.dhtest` with real window messages.
