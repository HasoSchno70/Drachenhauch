# Score editor

A tool for composing in real music notation (five lines per track, treble or
bass clef, ledger lines, accidentals) instead of the row grid of the
[tracker](tracker.md). Each track has exactly one instrument; the finished
piece can be saved as a file of its own **or** taken straight into the
tracker (one tracker channel per track).

The score editor is a Drachenhauch program:
`examples/199_notenblatt.dh`. It replaces the former Qt editor `dhscore`
(path B in [entwurf-python-abbau.md](../entwurf-python-abbau.md), German)
and needs no Python.

## Starting

- In the [IDE](ide.md): `Werkzeuge → Notenblatt` (Tools → Score editor) or
  the command palette ("Tool: score editor"). The tool runs as a separate
  `dhrt` process.
- From the command line:

```
dhrt run examples/199_notenblatt.dh [-- stueck.json]
```

With a file as argument, it is loaded; if it does not exist yet, an empty
piece begins, which ends up there when saved. The window starts maximised
and can be resized.

## Layout

At the top the tools (duration, dotted, accidental, mode, fingering number,
BPM, play, to tracker, +/- track) below the menu `Datei`/`Bearbeiten`
(File/Edit), on the left per track the name, clef and instrument (the
tracker's 18 factory presets), in the middle the staves — painted with the
normal drawing commands: five lines, bar shading, bar lines, ledger lines,
note heads (filled below the half note), stems, flags, beams for runs of
eighths and sixteenths of the same duration, rests, slurs as `SPLINE`,
fingering, staccato dot, a preview at the mouse and the playhead.

**Clefs and accidentals are drawn by the tool itself** (a curve through fixed
points, a sharp from four strokes, the flat as a shape of its own): the music
symbols of the symbol fonts arrived as question marks, and a question mark at
the start of the line is worse than a plain shape.

## Operation

| Action | How |
|---|---|
| Place a note | click on the staff. The chosen duration (whole to sixteenth, optionally dotted) is also the grid; the accidental (natural/sharp/flat) comes from the dropdown |
| Remove a note | click the same spot again, or the right mouse button (always removes) |
| Move a note | drag (time and pitch); another note at the target is replaced, slur anchors move along, a rest stays a rest |
| Rest | mode "Pause" (rest): a click places a rest instead of a note |
| Slur | mode "Bindebogen" (slur): first note, then the second; ESC cancels the selection, right-click removes slurs at that spot |
| Fingering | mode "Fingersatz" (fingering): a click assigns the chosen number (1–5), the same click with the same number takes it away again |
| Staccato | mode "Staccato": a click toggles it; it affects the sound (half duration in playback and export) |
| Tracks | `+`/`-` or the `Bearbeiten` (Edit) menu, up to 8 tracks; confirm the name with Enter |
| Clef change | dropdown per track. If the notes then lie far from the staff, an offset by whole octaves brings them closer and the status line says so (Ctrl+Z undoes it) |
| Tempo | BPM field, Enter applies (20..400) |
| Play | Space or F5, again = stop |
| Scrolling | mouse wheel = time, Shift+wheel = tracks |
| Undo | Ctrl+Z / Ctrl+Y — every change; the state is remembered as JSON text |
| File | Ctrl+N new, Ctrl+O open, Ctrl+S save, Ctrl+Shift+S save as, Ctrl+Q quit |
| Open in tracker | Ctrl+T or the button (see below) |

**Playback** runs on an audio clock (`AUDIO_CLOCK_NEW` + `AUDIO_PLAY_AT`, one
sixteenth per tick): all notes are scheduled at the start and sound
sample-accurately, not frame-driven. It is stopped by removing the clock — a
sound waiting for a clock that no longer exists never starts.

**New, Open and Quit** (menu, shortcut, window close button, Alt+F4) only ask
if the piece is not saved: Sichern / Verwerfen / Abbrechen (Save / Discard /
Cancel; Enter = Save, ESC = Cancel). When opening, the file dialog only comes
after the answer. Until 2026-09-21 only quitting asked; New and Open threw
unsaved notes away without a word.

## File format

The score editor reads and writes the same JSON as the former Qt version
(`"format": "dhscore-song"`, times in quarter beats):

- at the top `format`, `version`, `bpm`, `time_sig` (always `[4, 4]`) and
  `tracks`,
- per track `name`, `clef` (`treble`/`bass`), `instrument` (an instrument in
  the tracker's format) and `notes`, plus `slurs` (pairs of beat positions),
- per note `start`, `dur` and `pitch` (MIDI number) or `rest: true`,
  optionally `accidental`, `staccato`, `fingering`.

Loading is lenient: missing fields get defaults (120 BPM, treble clef, grand
piano, "Stimme n"). An example is `examples/notenblatt_demo.json`.

## Taking it into the tracker

**In Tracker öffnen** (Open in tracker) saves the piece (if needed),
converts it into a tracker project and writes it as `<name>_tracker.json`
next to the piece file. Then the tracker in Drachenhauch
(`examples/190_tracker.dh`) starts with this file. The rules:

- one tracker channel per track (at least four) plus an empty drum channel,
- four tracker rows per quarter beat (quarter = 4, eighth = 2,
  sixteenth = 1), each onset rounded to the nearest row,
- patterns of 64 rows, chained via the song order,
- chord (several notes at the same onset) → the highest note,
- staccato = half duration, at least one row; `NOTE_OFF` at the end if the
  cell is free,
- a note that would sound across a pattern boundary is cut off there,
- each track's instrument moves into the instrument pool.

**What is lost in the process is shown in a box BEFORE writing**: chord
reduced to its highest note, note rounded to the nearest tracker row, note
fell out of the song, note cut off at the pattern boundary — with track and
beat. Up to eight warnings are shown ("… und k weitere", … and k more).
"Trotzdem öffnen" (Open anyway, Enter) writes and starts the tracker,
"Abbrechen" (Cancel, ESC) writes nothing.

## Tests

`tests/pruef/werkzeug_notenblatt.dhtest` drives the tool with real clicks and
keys via input playback: place a note, remove it, drag it and Ctrl+Z, rest
and slur via the dropdown, a second track with an instrument, playback. The
tracker export is compared cell by cell on the demo piece (chord, staccato, a
note across the 64-row boundary, two tracks) with a fixed state that the
former Python converter produced; two more cases check the warning box
(Cancel writes nothing, "Trotzdem öffnen" writes the grid), three the prompt
on New (ESC keeps the notes, Discard clears the sheet, no question without a
change). Piece and tracker project are read by a Drachenhauch program with
the `json` module.

## Limits

Deliberate simplifications, not swallowed silently:

- **Fixed 4/4 metre** — the interface neither shows nor changes it.
- **One instrument per track**, at most eight tracks.
- **Chords are reduced on export** (a tracker channel is monophonic); a click
  on a note of a chord hits the first one at that beat.
- **Beams only for equal durations** — a run of eighths and sixteenths breaks
  up at the duration boundary, no partial beams.
- **Slurs and fingerings are pure notation** — export and playback ignore
  them.
- **No drum track type** — the export's drum channel stays empty.
- **No visual layout** — no collision avoidance between accidentals,
  fingerings, slurs and ledger lines; in dense passages they can overlap.
- **Clef change without a prompt** — the Qt version asked before the octave
  offset; here it is shifted and said.
- No real full screen via F11 (the Qt version had it).
