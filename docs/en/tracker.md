# Tracker (music editor, `examples/190_tracker.dh`)

A multi-channel tracker for composing music — written in Drachenhauch
itself, on the `gui` module. It is the counterpart of the former Qt tool
`dhtracker`, which disappeared together with the Python part of the project,
and reads and writes **the same song JSON** as that tool (below). As a
complement, the [SFX generator](sfx-generator.md) makes individual sounds.

It has patterns (1–64 rows) across 4–32 channels, the **last channel always
being the drum channel**; an order made from them; 18 ready-made instruments
with envelope, vibrato and detune, all editable; per note volume,
portamento, an effect and an instrument of its own; block selection with
copying, transposing and interpolating; mute/solo and mixer sliders per
channel; undo across the whole song; output as WAV and as DH code.

## Starting

In the IDE (`ide/ide.dh`) under **Tools → Tracker** (also via the command
palette, "Tool: tracker"). That starts it as a separate program.

From the command line:

```
dhrt run examples/190_tracker.dh                   # leerer Song
dhrt run examples/190_tracker.dh -- song.json      # gleich mit einer Datei
```

(The comments say: empty song; straight away with a file.) A relative file
name means the folder you were in when calling (`DHRT_START_DIR`), not the
folder of the source. That is also how the [score editor](score-editor.md)
opens the tracker with the piece it just converted via **[In Tracker
öffnen]** (Open in tracker).

## The window

At the top two toolbars, on the left the column for the channel under the
cursor, that channel's instrument and the order, on the right the **grid**
(rows from top to bottom, one column per channel in its own colour), below
it the **keyboard**, at the very bottom the status line (a `*` means:
unsaved).

| Area | What there is |
|---|---|
| Toolbar 1 | [Neu], [Oeffnen], [Sichern], [Sichern als] (New, Open, Save, Save as), **BPM**, **Kanäle** (channels, 4–32), [> Pattern], [> Song], [Stopp] (Stop), [Zurueck], [Vor] (Undo, Redo), [WAV ...] with the checkboxes **Stereo** and **Amiga**, [DH-Code] (DH code) |
| Toolbar 2 | pattern selection, **Reihen** (rows, 1–64), [+] new pattern, [Dup] copy, [Loesch] delete, [Leeren] clear, **Oktave** (octave) of the keyboard, then the fields of the cell under the cursor: **Vol** (1–15, 0 = default), **Slide** (−12…+12 semitones), **FX** with parameter (0–255), **Instrument** of the note, [Note aus] (note off) |
| Channel | default instrument of the channel (or a bare waveform `square`/`saw`/`sine`/`triangle`), **M** mute, **S** solo, mixer slider 0–100 % |
| Instrument | waveform (plus `noise`), volume 1–15, attack/decay/sustain/release, vibrato (depth %, Hz), detune in cents, pan; [+ Instrument] creates a copy of the one shown, [Entfernen] (Remove) removes it — channels and notes pointing to the instruments after it move along |
| Song (order) | list of patterns in playing order; a click shows the pattern in the grid. [+ akt.] appends the one shown, [entf.] removes, [<]/[>] move |

A pattern carries its name from the file (new ones are called `P1`, `P2` …);
it cannot be renamed here.

## Operation

A click into the grid sets the cursor, dragging or Shift+click selects a
block; the mouse wheel scrolls the rows, with Shift the channels. As long as
no control has the focus, the keys belong to the grid:

| Key | Effect |
|---|---|
| Arrows, Page Up/Down, Home/End | move the cursor (Page: 16 rows); with Shift select a block |
| `Z S X D C V G B H N J M` | notes of the lower octave (the US keyboard layout — on a German one the Z is on the Y) |
| `Q 2 W 3 E R 5 T 6 Y 7 U` | notes of the octave above |
| `0` | note off (key off, `OFF` in the cell) |
| Del, Backspace | delete the cell or the marked block — including its effects |
| Ctrl+C / Ctrl+X / Ctrl+V | copy / cut / paste the block at the cursor position |
| Ctrl+Arrow up/down | transpose the block by a semitone, with Shift by an octave; the drum channel stays |
| Ctrl+I | interpolate per channel between the first and last note of the block |
| Ctrl+Z / Ctrl+Y | undo / redo |
| Ctrl+S / Ctrl+O | save / open |
| Space | play the pattern, again: stop |
| ESC | quit |

A note you enter sounds briefly for preview, and the cursor moves on one row
— that way you type a melody without an arrow key. The keyboard at the
bottom also places notes with the mouse.

A note sounds **until the next note or an `OFF` comes on its channel**; in
song mode also beyond the end of the pattern into the next pattern of the
order. So you control the length via the distance in the grid.

The fields of the cell (Vol, Slide, FX, Instrument) only have an effect where
there is a note; Slide not on the drum channel. The effects:

| FX | Parameter | Effect |
|---|---|---|
| `Arp` | two hex digits `xy` | root, +x and +y semitones in tick rhythm (the C64 chord) |
| `Vib` | `xy` | vibrato at x Hz and y eighths of a semitone deep |
| `Ret` | ticks | retriggers the note every n ticks |
| `Off` | — | is read and saved but plays nothing here (sample offset needs samples) |

**Undo** remembers the whole song as text, 32 states — notes, patterns,
order, tempo and instruments, without separate bookkeeping per kind of
change. A drag on a slider is ONE step, not one per frame; the cursor stays
where it is when undoing.

**Mute and solo** apply to playback and preview; the WAV always mixes all
channels.

## Playback

[> Pattern] plays the pattern shown in a loop, [> Song] the order; in song
mode the grid follows the running pattern, and the sounding row is marked.

The notes run on an **audio clock** (`AUDIO_CLOCK_NEW` + `AUDIO_PLAY_AT`),
scheduled two rows ahead — started sample-accurately by the audio thread, not
frame-driven: a frame is 16 ms, and you would hear that with sixteenths. A
row has 6 ticks (the grid for arpeggio and retrigger). It is stopped by
**removing** the clock, not pausing it: a sound waiting for a clock that no
longer exists never starts — paused, it would come back as a ghost note at a
different place on the next start. A tempo change during playback adjusts the
clock immediately.

Every note is played as `AUDIO_NOTE` (envelope with sustain level, release
appended at the end, detune, portamento); without an instrument the
channel's bare waveform sounds, on the drum channel a short noise.

## WAV

[WAV ...] mixes the whole song offline into a file — via **the same
routine** as playback, only the target is a mix buffer
(`AUDIO_SOUND_NEW` + `AUDIO_SOUND_MIX`) instead of the clock; otherwise the
file would sound different from what you hear. Afterwards it is normalised
(`AUDIO_SOUND_NORMALIZE`, the status line names the factor) and written with
`AUDIO_SAVE_WAV`.

- **Stereo** evaluates the instruments' pan; without the checkbox the WAV
  stays single-channel.
- **Amiga** additionally places the channels like Paula: 1 and 4 left, 2 and
  3 right (with more channels the pattern repeats), not quite hard.

In the game, then `PLAYMUSIC("song.wav")`.

## DH code

[DH-Code] writes a **frame-driven live player** as a `.dh` file, like the
export of the Qt version: the order becomes a timeline (repeated patterns are
duplicated), per channel an array `trk<n>` with the frequencies, plus
`TRACKER_PLAY_ROW` and `TRACKER_UPDATE`. If a channel has notes with volume,
a track `trkV<n>` and the helper `TRACKER_AMP` are added; with slide a track
`trkSl<n>` (Hz/s), and the note then plays via `AUDIO_SFX` instead of
`AUDIO_TONE`. A channel's mixer slider goes into the volume, the drum channel
plays `AUDIO_NOISE`.

The file imports `audio` itself; in your own program you include it with
`IMPORT "song.dh"` and call in the game loop:

```
TRACKER_UPDATE(DELTA() * 1000.0)
```

The player knows **one** waveform per channel (that of the channel
instrument) and plays every note exactly one row long. It cannot do
envelopes, instruments per note or the FX column — for finished music the WAV
is the way to go.

## File format

A song is a JSON file (`format: "dhtracker-song"`), the same as the Qt
version's, in both directions. The tracker writes it indented with
`JSON_PRETTY`.

| Key | Content |
|---|---|
| `format`, `version` | `"dhtracker-song"`, `1` |
| `bpm` | tempo (40–300 is read) |
| `channels` | number of channels, the last one is the drums |
| `waves` | waveform per tone channel (without the drum channel) |
| `patterns` | list; per pattern `name`, `rows`, `channels` and the grids `data`, `vol`, `slide`, `fx`, `fxp`, `inst` — per channel a list with one value per row, `null` = empty |
| `order` | pattern numbers in playing order |
| `channel_vol` | mixer slider 0.0–1.0 per channel; only if one is not at 1.0 |
| `instruments` | list; a synth instrument with `name`, `kind: "synth"`, `default_vol`, `pan`, `waveform`, `env_attack_ms`/`env_decay_ms`/`env_sustain`/`env_release_ms`, `vib_depth`, `vib_speed`, `detune_cents` (plus `loop_mode`/`loop_start`/`loop_end` for the Qt version) |
| `channel_inst` | default instrument per channel, `null` = bare waveform |

The values in `data` are MIDI notes (0–127), `-1` is note off. `vol` is
1–15, `slide` −12…12, `fx` 1 = Arp, 2 = Vib, 3 = Ret, 4 = Off, `fxp` 0–255,
`inst` the number in `instruments`. The grids `vol`, `slide`, `fx`/`fxp` and
`inst` are only in the file if the pattern has something there. When reading,
nothing that may be missing is required: an unknown value is skipped, an
instrument number without an instrument behind it is reset.

**Sample, keymap and SoundFont instruments** of the Qt version (a SoundFont
became a keymap instrument there) **are played** by this tracker — in
playback, preview and the WAV. The lists name their kind
(`Klavier [sample]`). They are played according to the rules of the Qt
version: the pitch is the distance from the root key (`base_note` or
`root_note`), a loop applies with `forward` or `pingpong` if
0 ≤ start < end ≤ length, otherwise the sample plays through once and falls
silent; a keymap takes the first zone that covers the key, otherwise the
nearest one. The envelope is the same as for the synth (the release is
appended at the end), as is the slide; **there is no vibrato for samples**
(not in the Qt version either).

On such an instrument only name, volume, pan and envelope can be edited; its
JSON is **written back unchanged** on saving, and copying and removing carry
it along. Until 2026-09-17 the tracker wrote it back as a synth, and the
embedded samples were gone after one save. An instrument of a kind the
tracker does not know is kept and stays **silent** ("hier stumm" — silent
here — in the list). The samples are decoded once per content and not anew
with every undo. The **DH code** still plays only waveforms — a sample
instrument sounds there as its channel's waveform.

## Quitting

ESC, the window's close button or Alt+F4 quit at once if everything is
saved. Otherwise the question **Sichern | Verwerfen | Abbrechen** (Save |
Discard | Cancel) comes up. [Neu] (New) asks, if there are unsaved changes,
whether they should be discarded.

## What the Qt version had and is missing here

- **Creating and editing sample instruments** — loading WAV/OGG/SF2, the
  keymap dialog and the instrument editor with waveform and loop markers.
  The tracker plays instruments from a Qt-version file (above), but new
  samples cannot be brought in here.
- **VU meters** per channel.
- **Renaming patterns** — the names from the file stay, new ones cannot be
  assigned.
- **The sample offset effect** (`Off`) — it is carried along but has no
  effect, because it needs samples.

## What this pilot uncovered in the runtime

It was the first editor with a **timeline** (see CLAUDE.md, fifth pilot) and
needed three building blocks that did not exist before: `AUDIO_NOTE` (a held
note with a sustain **level** and release — `AUDIO_SFX` only knows three
times, organ and piano could not be told apart),
`AUDIO_SOUND_NEW`/`AUDIO_SOUND_MIX`/`AUDIO_SOUND_NORMALIZE` (no WAV without
mixing) and `JSON_APPEND_NULL` (a list with empty slots could not be written,
and that is exactly how the format notes a row without a note). See
[Audio](module-audio.md) and [JSON](module-json.md).

The sample instruments needed two more (2026-09-21): `SAMPLE_FROM_BUFFER` (a
sample from 16-bit PCM — that is how the samples lie Base64-encoded in the
file, with `BUFFER_FROM_BASE64` in front and no detour through a file) and
`SAMPLE_NOTE` (a note from a sample as a **SOUND** instead of played
immediately like `SAMPLE_PLAY` — only a sound can be scheduled on the audio
clock and mixed into the WAV). In addition `SAMPLE_SET_LOOP` now knows the
kind `pingpong`.

## Tested

`tests/pruef/werkzeug_tracker.dhtest` operates the tracker as if by hand
(recording via `AUTOMATION_PLAY`) and checks three results that Drachenhauch
reads itself: the **file** (json module, with the keys the Qt version read;
plus two files written by the Qt version), the **WAV** (length, notes, stereo
and Amiga pan) and the **DH code** (`dhrt --check` and a start). Plus the
keyboard, undo, block commands, transposing without the drums, the clock via
Space, following along in song mode, the position of all controls and the
sample/keymap instruments: a song that the case builds itself (a sine of
known frequency, a keymap with a silent zone and a loop zone) must sound in
the WAV at the right pitch, the sample without a loop must fall silent and
the loop must hold the note to the end of the pattern.

## Audio Studio

The Audio Studio (`dhsound`), which combined the tracker and the SFX
generator of the Qt version in one full-screen window with tabs, no longer
exists. Both tools are separate Drachenhauch programs and are in the IDE
under **Tools**: the tracker here, the SFX generator in
`examples/183_sfx_generator.dh` ([SFX generator](sfx-generator.md)).
