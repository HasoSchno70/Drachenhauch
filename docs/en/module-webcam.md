# Module `webcam`

Pictures from a camera into an `IMAGE` — to show them, process them
(`GETPIXEL`, `IMAGE_*`), save them or use them as a texture. That makes a
mirror, a motion detector, a photo booth, colour detection for a robot or a
time-lapse camera possible.

```basic
IMPORT "webcam"
```

One path per system, with no extra library on the user's computer: under
**Linux** Video4Linux2 (`/dev/videoN`), under **Windows** Media Foundation,
under **macOS** AVFoundation. Every camera runs in its own thread; the
program fetches the most recent picture when it asks — a slow program misses
pictures, it does not pile them up.

## First steps

```basic
IMPORT "webcam"
DIM cam AS WEBCAM
cam = WEBCAM_OPEN()
SCREEN(WEBCAM_WIDTH(cam), WEBCAM_HEIGHT(cam), "Spiegel")
WHILE NOT QUITREQUESTED()
    WEBCAM_UPDATE(cam)
    CLS(&H000000)
    DRAWIMAGE(WEBCAM_IMAGE(cam), 0, 0)
    FLIP()
WEND
WEBCAM_CLOSE(cam)
```

## Commands

| Command | What it does |
|---|---|
| `WEBCAM_DEVICES()` → ARRAY OF STRING | the names of all cameras; empty if the computer has none |
| `WEBCAM_OPEN([geraet$ [, breite, hoehe]])` → WEBCAM | open a camera — without a name the first one. A name matches exactly or as a part, ignoring case; an unknown one is an error that lists the available ones. `breite`/`hoehe` (width/height) are a wish the camera gets as close to as it can (0 = what it suggests) |
| `WEBCAM_CLOSE(kamera)` | close the camera and free its picture |
| `WEBCAM_UPDATE(kamera)` → BOOLEAN | has a new picture arrived since the last question? Then it is now in the camera's image |
| `WEBCAM_IMAGE(kamera)` → IMAGE | the camera's image with the most recent picture (fetches it itself). It is always the same `IMAGE` — unless the camera changes its size, then there is a new one |
| `WEBCAM_WIDTH(kamera)` / `WEBCAM_HEIGHT(kamera)` → INTEGER | the size the camera really delivers |
| `WEBCAM_FRAMES(kamera)` → INTEGER | how many pictures arrived since opening — shows whether anything is coming at all, and the frame rate |
| `WEBCAM_NAME$(kamera)` → STRING | the name of the open camera |
| `WEBCAM_ERROR$(kamera)` → STRING | what the camera reported after opening (unplugged, busy) — otherwise `""` |

The image needs graphics: without `SCREEN`, `WEBCAM_OPEN` creates a hidden
window, like `LOADIMAGE`.

## Detecting motion

Compare two pictures point by point — here every eighth point, that is
enough and fast:

```basic
IMPORT "webcam"
IMPORT "imgfx"
DIM cam AS WEBCAM
cam = WEBCAM_OPEN("", 320, 240)
DIM vorher AS IMAGE
vorher = IMAGE_COPY(WEBCAM_IMAGE(cam))
WHILE TRUE
    IF WEBCAM_UPDATE(cam) THEN
        DIM jetzt AS IMAGE : jetzt = WEBCAM_IMAGE(cam)
        DIM anders AS INTEGER : anders = 0
        DIM x AS INTEGER
        DIM y AS INTEGER
        FOR y = 0 TO WEBCAM_HEIGHT(cam) - 1 STEP 8
            FOR x = 0 TO WEBCAM_WIDTH(cam) - 1 STEP 8
                IF ABS((GETPIXEL(jetzt, x, y) BAND 255) - (GETPIXEL(vorher, x, y) BAND 255)) > 40 THEN anders = anders + 1
            NEXT
        NEXT
        IF anders > 50 THEN PRINT "Bewegung! "; anders
        IMAGE_FREE(vorher)
        vorher = IMAGE_COPY(jetzt)
    END IF
    SLEEP(30)
WEND
```

Saving a photo: `IMAGE_SAVE(WEBCAM_IMAGE(cam), "foto.png")`.

## Formats

The camera delivers what it can; dhrt reads **YUYV**, **MJPEG**, **RGB24/
BGR24** and **BGRX** (Linux) or lets the system convert to RGB (Windows,
macOS). For large pictures (above 640 × 480) MJPEG is requested first under
Linux — many cameras manage YUYV at 1280 × 720 over USB 2 only at a few
frames per second. MJPEG pictures without a Huffman table (that is how most
cameras send them) get the standard table.

## Permissions

- **Windows**: *Settings → Privacy & security → Camera* — access for desktop
  apps must be on, otherwise the camera cannot be
  opened.
- **macOS** asks on the first `WEBCAM_OPEN`. The Drachenhauch app brings the
  reason text and the entitlement; in the terminal the terminal's permission
  applies. If access is denied, the message says where to switch it on.
- **Linux**: the user must be in the group `video` (on Raspberry Pi OS and the
  usual systems from the start). A camera another program is already using
  reports "busy".

## Testing without a camera

`DHRT_WEBCAM=test` replaces all cameras by a **test picture** camera: eight
colour bars (white, yellow, cyan, green, magenta, red, blue, black) with a
black square that moves 8 points to the right per picture, 30 pictures per
second, 320 × 240 or the requested size. `tests/pruef/webcam.dhtest` tests
everything except the system's path with it; under Linux it also reads
`vivid`, the kernel's test driver — through the same path as a real camera.

## Limits

- Untested with a real camera under Windows and macOS — the path is
  compiled, the test machines have no camera.
- A Raspberry Pi camera module (CSI) goes through `libcamera` under
  Raspberry Pi OS, not V4L2 — a USB camera works. The
  helper `libcamerify` from libcamera can show the module as a V4L2 camera
  (untested).
- No sound from the camera (for that see [Microphone](module-mikrofon.md)), no
  settings such as exposure or focus.
- There is no camera in the browser yet.
