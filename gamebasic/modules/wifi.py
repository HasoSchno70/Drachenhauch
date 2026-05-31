"""WiFi-Management-Modul fuer GameBasic (Windows-only).

Greift auf die Windows-eigene `netsh wlan`-CLI zu - keine externe
Python-Lib noetig. Linux/macOS werden *nicht* unterstuetzt (dort wuerde
man `nmcli` bzw. `networksetup` ansprechen, was eigene Parser braucht).

Built-ins:
    WIFI_AVAILABLE()              -> BOOLEAN  (true wenn Windows + WLAN-Adapter)
    WIFI_CURRENT()                -> STRING   (aktuelle SSID, "" wenn nicht verbunden)
    WIFI_SIGNAL()                 -> INTEGER  (0..100, -1 wenn nicht verbunden)
    WIFI_SCAN()                   -> STRING   (multi-line "ssid|signal_pct")
    WIFI_CONNECT(ssid$, pass$)    -> BOOLEAN  (Profil anlegen + connect-Befehl)
    WIFI_DISCONNECT()             -> BOOLEAN
    WIFI_PROFILES()               -> STRING   (multi-line gespeicherter Profile)
    WIFI_DELETE_PROFILE(name$)    -> BOOLEAN

WICHTIG fuer WIFI_CONNECT:
- Funktioniert fuer offene Netze (pass$ leer) und WPA2-PSK (am haeufigsten).
- WPA-Enterprise (RADIUS), WPA3, captive portals werden NICHT unterstuetzt.
- Gibt nur zurueck ob das connect-Kommando angenommen wurde - polle danach
  WIFI_CURRENT() um zu sehen ob die Verbindung wirklich steht.
- Passwoerter werden in Klartext im Windows-Profil-Store abgelegt (Standard
  von netsh). Fuer Skript-Verwendung okay, fuer Production ungeeignet.
"""
from __future__ import annotations

import re
import subprocess
import sys
import tempfile
from html import escape as _xml_escape
from pathlib import Path

from ..builtins_registry import builtin
from ..errors import GBRuntimeError

_IS_WINDOWS = sys.platform == "win32"

# Versteckt das schwarze Konsolenfenster waehrend netsh-Aufrufen
_CREATE_NO_WINDOW = 0x08000000 if _IS_WINDOWS else 0


def _ensure_platform():
    if not _IS_WINDOWS:
        raise GBRuntimeError(
            "Modul WIFI: nur unter Windows unterstuetzt (nutzt netsh wlan).\n"
            "Linux: 'nmcli' direkt aufrufen; macOS: 'networksetup'."
        )


def _decode(raw: bytes) -> str:
    """netsh schreibt je nach Windows-Version in UTF-8 oder im OEM-Codepage.

    Reihenfolge: utf-8 zuerst, weil moderne Windows-11-Systeme netsh-Output
    bereits als UTF-8 liefern. Single-Byte-Codepages wie cp850/cp1252 wuerden
    diese Bytes klaglos akzeptieren und Umlaute kaputt-dekodieren (z.B. "ä"
    aus den UTF-8-Bytes 0xC3 0xA4 wuerde unter cp850 zu "├ñ"). UTF-8
    schlaegt bei wirklichem OEM-Output sauber fehl, dann faellt der Decoder
    auf den OEM-Codepage zurueck.
    """
    for enc in ("utf-8", "oem", "cp1252", "latin-1"):
        try:
            return raw.decode(enc)
        except (UnicodeDecodeError, LookupError):
            continue
    return raw.decode("latin-1", errors="replace")


def _run_netsh(args: list) -> tuple:
    """(returncode, output_text). Faengt FileNotFound/Timeout sauber ab."""
    try:
        proc = subprocess.run(
            ["netsh"] + args,
            capture_output=True,
            timeout=15,
            creationflags=_CREATE_NO_WINDOW,
        )
    except FileNotFoundError:
        raise GBRuntimeError("WIFI: 'netsh' nicht gefunden")
    except subprocess.TimeoutExpired:
        raise GBRuntimeError("WIFI: 'netsh' Timeout")
    return proc.returncode, _decode(proc.stdout) + _decode(proc.stderr)


# Regex fuer Output-Parsing (lokalisierungs-tolerant: stuetzt sich auf
# stabile Stichworte SSID / Signal / State|Status).

# Match "SSID : foo" aber NICHT "BSSID : ..." dank \b
_RE_SSID_LINE = re.compile(r"(?m)^\s*\bSSID\b\s*:\s*(.*?)\s*$")
# Match z.B. "Signal : 80%" - lokalisierungs-unabhaengig
_RE_SIGNAL_LINE = re.compile(r"(?m)^\s*\S[^:\n]*?:\s*(\d+)%\s*$")
# Scan-Block-Header "SSID 1 : foo"
_RE_SCAN_SSID = re.compile(r"(?m)^SSID\s+\d+\s*:\s*(.*?)\s*$")
# Profil-Liste: "Alle Benutzerprofile : MyNet" / "  All User Profile : MyNet"
# WICHTIG: alle Klassen mit \n verbieten, sonst spannt der Greedy-Match
# ueber mehrere Zeilen und faengt einen Doppelpunkt aus einer Folgezeile.
# Kein \b vor "profil" - im Deutschen wird das Wort in Komposita
# zusammengeschrieben ("Benutzerprofile") und haette dort keine Wortgrenze.
_RE_PROFILE_LINE = re.compile(
    r"(?im)^[^:\n]*profil[^:\n]*?:[ \t]*(.+?)[ \t]*$",
)


# --- Profil-XML-Generator ------------------------------------------------

_PROFILE_TEMPLATE_PSK = """<?xml version="1.0"?>
<WLANProfile xmlns="http://www.microsoft.com/networking/WLAN/profile/v1">
    <name>{ssid}</name>
    <SSIDConfig>
        <SSID>
            <name>{ssid}</name>
        </SSID>
    </SSIDConfig>
    <connectionType>ESS</connectionType>
    <connectionMode>auto</connectionMode>
    <MSM>
        <security>
            <authEncryption>
                <authentication>WPA2PSK</authentication>
                <encryption>AES</encryption>
                <useOneX>false</useOneX>
            </authEncryption>
            <sharedKey>
                <keyType>passPhrase</keyType>
                <protected>false</protected>
                <keyMaterial>{password}</keyMaterial>
            </sharedKey>
        </security>
    </MSM>
</WLANProfile>
"""

_PROFILE_TEMPLATE_OPEN = """<?xml version="1.0"?>
<WLANProfile xmlns="http://www.microsoft.com/networking/WLAN/profile/v1">
    <name>{ssid}</name>
    <SSIDConfig>
        <SSID>
            <name>{ssid}</name>
        </SSID>
    </SSIDConfig>
    <connectionType>ESS</connectionType>
    <connectionMode>auto</connectionMode>
    <MSM>
        <security>
            <authEncryption>
                <authentication>open</authentication>
                <encryption>none</encryption>
                <useOneX>false</useOneX>
            </authEncryption>
        </security>
    </MSM>
</WLANProfile>
"""


def _build_profile_xml(ssid: str, password: str) -> str:
    ssid_x = _xml_escape(ssid, quote=True)
    if not password:
        return _PROFILE_TEMPLATE_OPEN.format(ssid=ssid_x)
    pw_x = _xml_escape(password, quote=True)
    return _PROFILE_TEMPLATE_PSK.format(ssid=ssid_x, password=pw_x)


# --- Built-ins ----------------------------------------------------------

@builtin("WIFI_AVAILABLE", arity=0)
def _wifi_available():
    if not _IS_WINDOWS:
        return False
    try:
        rc, out = _run_netsh(["wlan", "show", "interfaces"])
    except GBRuntimeError:
        return False
    # Wenn kein WLAN-Adapter da ist, gibt netsh i.d.R. einen Hinweis aus
    # ohne SSID/Signal-Sektion; Returncode ist trotzdem 0. Wir checken
    # einfach ob das Schluesselwort 'WLAN' vorkommt.
    return rc == 0 and ("WLAN" in out or "Wireless" in out or "Drahtlos" in out)


@builtin("WIFI_CURRENT", arity=0)
def _wifi_current():
    _ensure_platform()
    _, out = _run_netsh(["wlan", "show", "interfaces"])
    # State/Status pruefen - nur 'connected'/'verbunden' liefert SSID
    state_line = ""
    for line in out.splitlines():
        ls = line.strip().lower()
        if (ls.startswith("state") or ls.startswith("status")
                or ls.startswith("zustand")) and ":" in ls:
            state_line = ls.split(":", 1)[1].strip()
            break
    connected = state_line in (
        "connected", "verbunden", "connecté", "connesso", "conectado"
    )
    if not connected:
        return ""
    m = _RE_SSID_LINE.search(out)
    return m.group(1).strip() if m else ""


@builtin("WIFI_SIGNAL", arity=0)
def _wifi_signal():
    _ensure_platform()
    _, out = _run_netsh(["wlan", "show", "interfaces"])
    m = _RE_SIGNAL_LINE.search(out)
    if not m:
        return -1
    try:
        return int(m.group(1))
    except ValueError:
        return -1


@builtin("WIFI_SCAN", arity=0)
def _wifi_scan():
    _ensure_platform()
    _, out = _run_netsh(["wlan", "show", "networks", "mode=bssid"])
    blocks = []
    cur_ssid = None
    cur_signal = None
    for line in out.splitlines():
        m = _RE_SCAN_SSID.match(line)
        if m:
            if cur_ssid is not None:
                blocks.append((cur_ssid, cur_signal))
            cur_ssid = m.group(1).strip() or "(versteckt)"
            cur_signal = None
            continue
        if cur_ssid is not None and cur_signal is None:
            sm = _RE_SIGNAL_LINE.match(line)
            if sm:
                try:
                    cur_signal = int(sm.group(1))
                except ValueError:
                    pass
    if cur_ssid is not None:
        blocks.append((cur_ssid, cur_signal))
    # nach Signal absteigend sortieren (None ans Ende)
    blocks.sort(key=lambda b: (b[1] is None, -(b[1] or 0)))
    return "\n".join(
        f"{s}|{sig if sig is not None else 0}" for s, sig in blocks
    )


@builtin("WIFI_CONNECT", arity=2, types=("str", "str"))
def _wifi_connect(ssid, password):
    _ensure_platform()
    if not ssid:
        raise GBRuntimeError("WIFI_CONNECT: SSID darf nicht leer sein")
    xml = _build_profile_xml(ssid, password)
    # Temp-Datei mit Profil-XML; netsh add profile filename=...
    tmp = Path(tempfile.gettempdir()) / f"_gb_wifi_{ssid}.xml"
    try:
        tmp.write_text(xml, encoding="utf-8")
        rc, out = _run_netsh([
            "wlan", "add", "profile",
            f"filename={tmp}", "user=current",
        ])
        if rc != 0:
            raise GBRuntimeError(
                f"WIFI_CONNECT: Profil konnte nicht angelegt werden\n{out.strip()}"
            )
    finally:
        try:
            tmp.unlink(missing_ok=True)
        except Exception:
            pass
    rc, out = _run_netsh(["wlan", "connect", f"name={ssid}"])
    return rc == 0


@builtin("WIFI_DISCONNECT", arity=0)
def _wifi_disconnect():
    _ensure_platform()
    rc, _ = _run_netsh(["wlan", "disconnect"])
    return rc == 0


@builtin("WIFI_PROFILES", arity=0)
def _wifi_profiles():
    _ensure_platform()
    _, out = _run_netsh(["wlan", "show", "profiles"])
    names = []
    seen = set()
    for m in _RE_PROFILE_LINE.finditer(out):
        n = m.group(1).strip()
        # Filter: Header-Zeilen "Gruppenrichtlinien..." enthalten zwar
        # "profil", haben aber keinen sinnvollen Wert nach dem Doppelpunkt.
        # Doppelte und group-prefix-Sektionen verwerfen.
        if n and n not in seen and not n.lower().startswith("group"):
            seen.add(n)
            names.append(n)
    return "\n".join(names)


@builtin("WIFI_DELETE_PROFILE", arity=1, types=("str",))
def _wifi_delete_profile(name):
    _ensure_platform()
    if not name:
        raise GBRuntimeError("WIFI_DELETE_PROFILE: Name darf nicht leer sein")
    rc, _ = _run_netsh(["wlan", "delete", "profile", f"name={name}"])
    return rc == 0
