# Module `wifi`

Wi-Fi management: scan for networks, query the current connection, connect with SSID + password, manage saved profiles.

```basic
IMPORT "wifi"
```

## Platform & dependency

**Windows, Linux, macOS.** No external library needed — the module calls the operating system's built-in command-line management in each case: Windows `netsh wlan`, Linux `nmcli` (NetworkManager), macOS `networksetup`/`airport`.

**Cross-platform status:** The Windows branch has been verified against real hardware. **Linux (`nmcli`) and macOS (`networksetup`/`airport`) are new and NOT tested on real hardware** (development has so far happened on Windows only) — they were written from the public documentation (Linux) or to the best of our knowledge (macOS). The macOS branch is the least certain: `airport` (for `WIFI_SCAN`/`WIFI_SIGNAL`) lives in a private, undocumented Apple framework whose behaviour has changed several times between macOS versions and which newer macOS versions partly hide behind a Location Services permission — if it fails, you get a clear error message instead of a silent failure. Feedback from real Linux/macOS users is explicitly welcome (e.g. as a GitHub issue).

On other platforms (BSD, ...) every call throws `DHRuntimeError` with a clear hint.

## Overview

| Function | Returns | Meaning |
|---|---|---|
| `WIFI_AVAILABLE()` | BOOLEAN (true if Windows + Wi-Fi adapter) | can the module be used here at all? |
| `WIFI_CURRENT()` | STRING (current SSID, "" if not connected) | which network is the computer connected to? |
| `WIFI_SIGNAL()` | INTEGER (0..100, -1 if not connected) | signal strength of the current connection |
| `WIFI_SCAN()` | STRING (multi-line `ssid\|signal_pct`, sorted by signal) | search for reachable networks |
| `WIFI_CONNECT(ssid$, pass$)` | BOOLEAN (create profile + issue connect command) | connect to a network -- creates a Windows profile in the process |
| `WIFI_DISCONNECT()` | BOOLEAN | disconnect |
| `WIFI_PROFILES()` | STRING (multi-line list of saved profiles) | which networks are saved? |
| `WIFI_DELETE_PROFILE(name$)` | BOOLEAN | forget a saved network |

## Localisation (Windows)

`netsh wlan` prints its output in the system language (German / English / French …). The parser is tolerant — connected/disconnected states are recognised in DE/EN/FR/IT/ES, profile names are found via the keyword „profil" / „Profile". If your language is not recognised: issue / patch welcome.

`WIFI_AVAILABLE()` does NOT check for fixed words such as „WLAN"/„Wireless" (any more) — that was narrower than the five-language detection of `WIFI_CURRENT()` and wrongly returned `FALSE` on every other Windows system language. Instead, all that counts is whether `netsh wlan show interfaces` managed to list an interface (language-independent).

The console output of `netsh` on Windows is not reliably in one fixed encoding (sometimes UTF-8, sometimes the OEM code page, depending on Windows version/configuration) — the module tries UTF-8 first and falls back to the OEM code page on invalid bytes. Umlauts/special characters in SSID/profile names (e.g. `WIFI_PROFILES()`) are therefore shown correctly instead of as mojibake.

`nmcli` (Linux) and `networksetup`/`airport` (macOS) deliver machine-readable or language-neutral output (`nmcli -t` = "terse", `networksetup` messages are stable strings) — there is no localisation problem there.

## Platform differences

- **`WIFI_SIGNAL()`/`WIFI_SCAN()` values**: Windows and Linux deliver real percentages (0..100) straight from the driver. macOS only delivers RSSI in dBm (`airport`) — the module converts that to 0..100 with a rough formula common in the networking world (`-50dBm≈100%`, `-100dBm≈0%`); this is an approximation, not an original driver value.
- **`WIFI_DISCONNECT()`**: Windows/Linux cleanly disconnect the active connection. macOS' `networksetup` has no command of its own for that — the workaround briefly switches the Wi-Fi radio off and on again (disassociates from every network; Wi-Fi stays on afterwards).
- **`WIFI_CONNECT`/profiles**: On all three platforms connecting saves a profile/connection (Windows Wi-Fi profile, NetworkManager connection, macOS „Preferred Network"), which `WIFI_PROFILES()`/`WIFI_DELETE_PROFILE()` manage.

## Example — current status + scan

```basic
IMPORT "wifi"

IF NOT WIFI_AVAILABLE() THEN
    PRINT "Kein WLAN-Adapter gefunden."
ELSE
    DIM ssid AS STRING
    ssid = WIFI_CURRENT()

    IF LEN(ssid) > 0 THEN
        PRINT "Verbunden: ", ssid, "  (", WIFI_SIGNAL(), "%)"
    ELSE
        PRINT "Aktuell nicht verbunden."
    END IF

    PRINT "Scan ..."
    DIM zeilen AS ARRAY OF STRING
    zeilen = SPLIT$(WIFI_SCAN(), CHR$(10))
    DIM i AS INTEGER
    FOR i = 0 TO LEN(zeilen) - 1
        PRINT "  ", zeilen[i]
    NEXT
END IF
```

## `WIFI_CONNECT` — what it does

```basic
WIFI_CONNECT("MeinNetz", "geheim123")
```

Two steps:

1. It generates a Wi-Fi profile XML (WPA2-PSK / AES for password-protected networks, open for an empty password) and creates it via `netsh wlan add profile`. For an SSID that already has a profile, it replaces the profile.
2. It calls `netsh wlan connect name=<ssid>`.

`WIFI_CONNECT` returns immediately — **it does not wait for the connection to actually be established**. Whether the connection is really up, you check with `WIFI_CURRENT()`:

```basic
IF WIFI_CONNECT("MeinNetz", "geheim123") THEN
    ' Wait for the connection
    DIM versuch AS INTEGER
    FOR versuch = 1 TO 10
        SLEEP(1000)
        IF WIFI_CURRENT() = "MeinNetz" THEN
            PRINT "Verbunden!"
            BREAK
        END IF
    NEXT
END IF
```

### What is not supported

- **WPA Enterprise** (RADIUS/802.1X): needs extra fields in the profile XML.
- **WPA3-SAE**: the profile XML schema would have to look different.
- **Hidden networks** (SSID broadcast off): the scan list does not see them; connecting via an explicit SSID can sometimes work anyway.
- **Captive portals** (café Wi-Fi with a login page): the module only connects layer 2 — the browser login has to be left to the user.

### Security note

The profile created by `WIFI_CONNECT` stores the password as **plaintext** in the Windows profile store (`<protected>false</protected>`). That is the default behaviour of `netsh wlan add profile`. Fine for scripts, not suitable for production / multi-user machines.

## Managing profiles

```basic
PRINT WIFI_PROFILES()                   ' list of all saved profiles
WIFI_DELETE_PROFILE("AlteSchule_WLAN")  ' remove a profile
```

## Permissions

`netsh wlan` normally runs without admin rights. Only `WIFI_CONNECT` with a new SSID (= creating a new profile) **may** require admin rights depending on group policy — never an issue in private setups.

## Complete example

See [examples/36_wifi.dh](../../examples/36_wifi.dh).

## In the native runtime (dhrt)

`wifi` runs natively with the Cargo feature `wifi` (Windows via `netsh wlan`, Linux via `nmcli`, macOS via `networksetup`/`airport` -- all three via `std::process`, no additional crate). Every subprocess call has a 10-second timeout -- a hanging/overloaded network stack therefore no longer freezes the game loop. Build: `python rust/build_runtime.py --hardware`. The `rust-check` CI job (`.github/workflows/ci.yml`) compiles the `wifi` feature on all three platforms (ubuntu/macos/windows-latest) -- the only automatic cross-platform signal so far, since CI runners lack real Wi-Fi hardware.
