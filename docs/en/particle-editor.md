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

| Area | What it does |
|---|---|
| Preview | The particles, clipped to the area; the source sits in the centre or, with **folgt der Maus** (follows the mouse), under the pointer. At the bottom the number of living particles |
| Controls | **Pause**/**Weiter** (Pause/Resume), **Leeren** (Clear), **Salve** (Burst — 200 particles at once, for explosions) |
| Presets | Funken, Rauch, Feuer, Regen, Schnee, Explosion, Zauber, Springbrunnen (sparks, smoke, fire, rain, snow, explosion, magic, fountain) — one click sets all sliders, render mode and switches and clears the preview |
| Output | **Sichern ...**/**Laden ...** (Save/Load, `.ini`), **DH-Code kopieren** (Copy DH code) |
| History | **Zurueck**/**Vor** (Back/Forward) or Ctrl+Z/Ctrl+Y |

The sliders:

| Group | Sliders |
|---|---|
| Movement | speed X min/max, speed Y min/max (±400 px/s), force X/Y (±600 px/s²) |
| Lifetime (ms) and emission | life min/max (50..4000 ms), per frame (0..60 particles) |
| Appearance | render mode (`circle`, `pixel`, `square`, `streak`, `glow`), size min/max, **am Ende ausblenden** (fade out at the end) |
| Colour at the start | red, green, blue, below them a colour swatch with the result |
| Colour at the end | **Farbverlauf benutzen** (use gradient), red, green, blue, colour swatch |

Without a gradient the system also gets the start colour as its end colour —
the module knows no "off". `PARTICLE_DRAW` currently draws `glow` like
`circle`; for real glowing in the game, wrap the call in `BLEND_MODE("add")`.

Everything also works without a mouse: TAB moves between controls, Space
triggers, the arrow keys adjust a slider.

## Undo

One **drag** on a slider is **one** step: it is only recorded once nothing
has changed for one frame and no mouse button is held any more. A preset and
a loaded `.ini` are one step each the same way. The full state is remembered
(17 sliders, render mode, two switches), up to 32 steps; a new drag after an
undo cuts off the way forward.

## Saving and loading

**Sichern ...** (Save) writes all sliders into an `.ini`, **Laden ...**
(Load) brings them back. The key is the slider's **label**, plus `modus` for
the render mode — readable and editable by hand. An unknown key is skipped,
a value outside the slider's range is clamped. The two switches (fade out,
gradient) are not in the file.

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
is clickable (the buttons used to lie on top of it). The prompt on closing is
checked by `tests/pruef/werkzeug_kreuz.dhtest` with real window messages.
