# Module `video`

Playing videos: MP4 with H.264, the format that practically every program and
every phone writes. Every frame ends up in an ordinary `IMAGE` -- it is drawn
like any other image, scaled, on a layer, in a render target or as a texture.
The IDE uses it to show its intro.

```basic
IMPORT "video"
```

The runtime reads the container itself; the frames are decoded by **openh264**
(Cisco's free H.264 decoder, compiled from source along with the runtime).
Nothing has to be installed on the user's machine.

## Commands

| Function | Effect |
|---|---|
| `VIDEO_LOAD(path$)` | open an MP4 file, returns a `VIDEO`; the first frame is in the image right away |
| `VIDEO_PLAY(video)` | play; a video that has run to the end starts over |
| `VIDEO_PAUSE(video)` | pause, the frame stays where it is |
| `VIDEO_STOP(video)` | stop and go back to the start |
| `VIDEO_SEEK(video, seconds)` | jump to a position (clamped to the duration) |
| `VIDEO_LOOP(video, on)` | start over at the end instead of stopping |
| `VIDEO_UPDATE(video)` | advance the video's clock by `DELTA()` without drawing |
| `VIDEO_DRAW(video, x, y [, width, height])` | advance and draw the current frame |
| `VIDEO_PLAYING(video)` | is it playing right now? |
| `VIDEO_DONE(video)` | has it (without looping) reached the end? |
| `VIDEO_POSITION(video)` | position in seconds |
| `VIDEO_DURATION(video)` | duration in seconds |
| `VIDEO_WIDTH(video)` | width in pixels |
| `VIDEO_HEIGHT(video)` | height in pixels |
| `VIDEO_FRAMES(video)` | number of frames |
| `VIDEO_FRAME(video)` | number of the frame currently in the `IMAGE` (from 0) |
| `VIDEO_FPS(video)` | frames per second |
| `VIDEO_IMAGE(video)` | the `IMAGE` that every new frame is written into |
| `VIDEO_FREE(video)` | free the video and its image |

## Example

```basic
IMPORT "video"
SCREEN(1280, 720, "Vorspann", 1)
DIM v AS VIDEO : v = VIDEO_LOAD("intro.mp4")
VIDEO_PLAY(v)
WHILE NOT QUITREQUESTED() AND NOT VIDEO_DONE(v)
    CLS(0)
    VIDEO_DRAW(v, 0, 0, SCREENWIDTH(), SCREENHEIGHT())
    FLIP()
WEND
VIDEO_FREE(v)
```

## How time passes

A video has its own clock, and it runs with `DELTA()` -- that is, with real
time, not with the program's frame rate. If the program runs at 60 frames per
second and the video at 24, each video frame is shown two or three times; if
the program stutters, the video skips frames instead of slowing down. The
clock starts with the frame **after** `VIDEO_PLAY`.

`VIDEO_DRAW` advances the clock itself; if you use the image differently (via
`VIDEO_IMAGE` as a texture, in a shader), call `VIDEO_UPDATE` once per frame
instead. Both in the same frame only count once.

Without a window (tests with `DHRT_FRAMES`) `DELTA()` is fixed at 1/60 s -- a
video is then at exactly n/60 seconds after n frames, the same on every run.

At the end the **last frame** stays in place and `VIDEO_DONE` becomes true; an
intro that went black afterwards would flash at the transition.

## Limits

- **Only H.264 in MP4** (`.mp4`, `.m4v`, `.mov` with H.264). Other formats
  (H.265/HEVC, VP9, AV1) are rejected by `VIDEO_LOAD` with a message that
  names the format.
- **No sound.** An audio track is skipped; if you want music with it, play it
  with `AUDIO_MUSIC_PLAY` and start both in the same frame.
- **Rewinding decodes from the start.** H.264 can only restart at key frames;
  `VIDEO_SEEK` backwards and looping therefore take a moment with long videos.
  Forwards it continues frame by frame.
- Not in the web build (`dhrt` as WebAssembly) and not in a build without the
  feature `video` -- there the commands are unknown.
- H.264 is encumbered by patents. Cisco only covers the licence fees for the
  ready-made openh264 libraries that Cisco distributes itself; this runtime
  compiles openh264 from source. If you sell a program with videos or
  distribute it widely, you should clarify this beforehand.
