# Module `hotplug`

Report when a device is **plugged in** or **unplugged**: an Arduino on USB, a
MIDI keyboard, a game controller over HID. A program can then connect by
itself instead of being restarted, or show "device disconnected" instead of
failing at the next request.

```basic
IMPORT "hotplug"
```

What is watched is what this build knows (`HOTPLUG_KINDS`):

| Kind | What | needs |
|---|---|---|
| `"serial"` | serial ports (`COM5`, `/dev/ttyACM0`) | feature `serial` |
| `"midi_in"`, `"midi_out"` | MIDI inputs and outputs | feature `midi` |
| `"hid"` | USB HID devices, as `"VID:PID Produkt"` | feature `usb` |

The names are the same ones `SERIAL_PORTS`, `MIDI_IN_NAME$` and `USB_LIST`
return — whoever is told that `COM5` has appeared opens `COM5`.

## Commands

| Function | Effect |
|---|---|
| `HOTPLUG_NEXT()` → BOOLEAN | fetch the next report; FALSE if there is none. Never waits |
| `HOTPLUG_KIND$()` → STRING | kind of the fetched report, such as `"serial"` |
| `HOTPLUG_NAME$()` → STRING | name of the device |
| `HOTPLUG_ADDED()` → BOOLEAN | TRUE = plugged in, FALSE = unplugged |
| `HOTPLUG_LIST(art$)` → ARRAY OF STRING | what of this kind is present right now, sorted |
| `HOTPLUG_KINDS()` → ARRAY OF STRING | which kinds this build watches |
| `HOTPLUG_INTERVAL(ms)` | how often to look (100..10000, default 500) |

```basic
IMPORT "hotplug"
IMPORT "serial"
DIM s AS SERIAL_HANDLE : s = -1
SCREEN(400, 120, "Arduino")
WHILE NOT QUITREQUESTED()
    WHILE HOTPLUG_NEXT()
        IF HOTPLUG_KIND$() = "serial" AND HOTPLUG_ADDED() AND s < 0 THEN
            s = SERIAL_OPEN(HOTPLUG_NAME$(), 115200)
        END IF
    WEND
    CLS(&H101018)
    TEXT(10, 50, IIF(s < 0, "Arduino anstecken ...", "verbunden"), &HE0E0E0)
    FLIP()
WEND
```

What is already present at the first look counts as the starting point and is
**not** reported — that is what `HOTPLUG_LIST` is for. The first call of a
`HOTPLUG_` command records the starting point. At most 256 reports wait; the
oldest are dropped.

## How it works

Every system reports this in its own way (Windows `WM_DEVICECHANGE`, Linux
udev, macOS IOKit), and none of them covers all three kinds. So a thread of
its own looks at the same lists every 500 ms and compares them with the state
before — a report therefore comes up to half a second after plugging in. An
HID device with several interfaces counts once.

For testing without a device: `DHRT_GERAETE=datei` reads the lists from a text
file instead of the system, one line per device as `art:name`. Changing the
file produces the reports (`tests/pruef/geraete/hotplug.dhtest`).

**Not tested:** real plugging and unplugging. On macOS, CoreMIDI may only see
new MIDI devices once the program has a window (the event loop then runs);
untested.
