"""Tests fuer das wifi-Modul (Windows netsh).

Hauptpruefungen:
- Decoder: utf-8 vor cp850/cp1252 - Umlaute wie "ä" werden korrekt
  durchgereicht statt als "├ñ" zerschossen.
- XML-Generator fuer Connect-Profile: open vs WPA2-PSK, korrekte
  XML-Escaping von Sonderzeichen in SSID/Passwort.
- Parser fuer netsh-Output: WIFI_CURRENT/SIGNAL/SCAN/PROFILES gegen
  echten Output von Windows in Deutsch und Englisch.
"""
import sys

import pytest

from gamebasic.modules import load_module
from gamebasic.modules import wifi as wifi_mod
from gamebasic.errors import GBRuntimeError


@pytest.fixture(scope="module", autouse=True)
def _load():
    assert load_module("wifi")


# Helfer: monkeypatcht _run_netsh so, dass eine fixe Antwort zurueckkommt
def _fake_netsh(monkeypatch, output: str, returncode: int = 0):
    def _run(args):
        return returncode, output
    monkeypatch.setattr(wifi_mod, "_run_netsh", _run)


# --- Registrierung ---------------------------------------------------

def test_all_builtins_registered():
    from gamebasic.interpreter import BUILTINS
    expected = {
        "wifi_available", "wifi_current", "wifi_signal", "wifi_scan",
        "wifi_connect", "wifi_disconnect",
        "wifi_profiles", "wifi_delete_profile",
    }
    assert expected <= set(BUILTINS.keys())


# --- Decoder ---------------------------------------------------------

def test_decode_pure_ascii():
    assert wifi_mod._decode(b"hello") == "hello"


def test_decode_utf8_umlaut_works():
    """ä als UTF-8 (0xC3 0xA4) muss als ä, nicht als ├ñ landen."""
    raw = "Schnittstelle ä".encode("utf-8")
    assert wifi_mod._decode(raw) == "Schnittstelle ä"


def test_decode_cp850_umlaut_falls_back():
    """Echter cp850-Output: ä = 0x84. UTF-8-Decode wuerde scheitern,
    Decoder soll auf oem/cp850 zurueckfallen."""
    raw = b"Status: \x84"  # 0x84 in cp850 == ä
    out = wifi_mod._decode(raw)
    if sys.platform == "win32":
        # Auf Windows oem-Codepage verfuegbar -> ä korrekt
        assert "ä" in out
    else:
        # Auf Nicht-Windows kein oem -> faellt auf cp1252 (0x84 -> "„") oder
        # latin-1 (0x84 -> Steuerzeichen). Hauptsache, kein Crash.
        assert isinstance(out, str)


def test_decode_garbage_does_not_crash():
    raw = bytes(range(256))
    out = wifi_mod._decode(raw)
    assert isinstance(out, str)


# --- XML-Profil-Generator -------------------------------------------

def test_profile_open_network_no_sharedkey():
    xml = wifi_mod._build_profile_xml("MyOpenNet", "")
    assert "<authentication>open</authentication>" in xml
    assert "<encryption>none</encryption>" in xml
    assert "sharedKey" not in xml
    assert "<name>MyOpenNet</name>" in xml


def test_profile_psk_with_password():
    xml = wifi_mod._build_profile_xml("HomeNet", "secret123")
    assert "<authentication>WPA2PSK</authentication>" in xml
    assert "<encryption>AES</encryption>" in xml
    assert "<keyMaterial>secret123</keyMaterial>" in xml


def test_profile_xml_escapes_special_chars_in_ssid():
    """SSID mit < > & " - duerfen das XML nicht zerstoeren."""
    xml = wifi_mod._build_profile_xml('A<B&"C', "")
    # Rohe Sonderzeichen DUERFEN nicht im XML stehen - escapt schon.
    # Das oeffnende <name>-Tag bleibt natuerlich literal,
    # aber der Inhalt muss escapt sein.
    name_open = xml.index("<name>")
    name_close = xml.index("</name>", name_open)
    inner = xml[name_open + len("<name>"):name_close]
    assert "<" not in inner and ">" not in inner
    assert "&" in inner  # escape-Entitaet (z.B. &amp;)


def test_profile_xml_escapes_special_chars_in_password():
    xml = wifi_mod._build_profile_xml("net", 'p&w"<x>')
    km_open = xml.index("<keyMaterial>")
    km_close = xml.index("</keyMaterial>", km_open)
    inner = xml[km_open + len("<keyMaterial>"):km_close]
    assert "<" not in inner and ">" not in inner
    assert "&amp;" in inner


# --- WIFI_CURRENT ----------------------------------------------------

_NETSH_INTERFACES_DE_VERBUNDEN = """
Es ist 1 Schnittstelle auf dem System vorhanden:

    Name                   : WLAN
    Status                 : Verbunden
    SSID                   : HomeNet
    BSSID                  : aa:bb:cc:dd:ee:ff
    Signal                 : 80%
"""

_NETSH_INTERFACES_DE_GETRENNT = """
Es ist 1 Schnittstelle auf dem System vorhanden:

    Name                   : WLAN
    Status                 : getrennt
    Funkstatus             : Hardware Aktiviert
"""

_NETSH_INTERFACES_EN_CONNECTED = """
There is 1 interface on the system:

    Name                   : Wi-Fi
    State                  : connected
    SSID                   : OfficeNet
    BSSID                  : 11:22:33:44:55:66
    Signal                 : 65%
"""


def test_current_when_connected_de(monkeypatch, call_builtin):
    _fake_netsh(monkeypatch, _NETSH_INTERFACES_DE_VERBUNDEN)
    assert call_builtin("wifi_current", []) == "HomeNet"


def test_current_when_connected_en(monkeypatch, call_builtin):
    _fake_netsh(monkeypatch, _NETSH_INTERFACES_EN_CONNECTED)
    assert call_builtin("wifi_current", []) == "OfficeNet"


def test_current_when_disconnected_de(monkeypatch, call_builtin):
    _fake_netsh(monkeypatch, _NETSH_INTERFACES_DE_GETRENNT)
    assert call_builtin("wifi_current", []) == ""


def test_signal_when_connected(monkeypatch, call_builtin):
    _fake_netsh(monkeypatch, _NETSH_INTERFACES_DE_VERBUNDEN)
    assert call_builtin("wifi_signal", []) == 80


def test_signal_when_disconnected(monkeypatch, call_builtin):
    _fake_netsh(monkeypatch, _NETSH_INTERFACES_DE_GETRENNT)
    assert call_builtin("wifi_signal", []) == -1


# --- WIFI_SCAN -------------------------------------------------------

_NETSH_SCAN_DE = """Schnittstellenname : WLAN
Es sind aktuell 3 Netzwerke sichtbar.

SSID 1 : NetzA
    Authentifizierung       : WPA2-Personal
    Verschluesselung        : CCMP
    BSSID 1                 : 00:00:00:00:00:01
         Signal             : 90%

SSID 2 : NetzB
    Authentifizierung       : WPA2-Personal
    BSSID 1                 : 00:00:00:00:00:02
         Signal             : 50%

SSID 3 : NetzSchwach
    Authentifizierung       : WPA2-Personal
    BSSID 1                 : 00:00:00:00:00:03
         Signal             : 20%
"""


def test_scan_parses_three_networks(monkeypatch, call_builtin):
    _fake_netsh(monkeypatch, _NETSH_SCAN_DE)
    out = call_builtin("wifi_scan", [])
    lines = out.splitlines()
    assert len(lines) == 3


def test_scan_sorts_by_signal_descending(monkeypatch, call_builtin):
    _fake_netsh(monkeypatch, _NETSH_SCAN_DE)
    out = call_builtin("wifi_scan", [])
    lines = out.splitlines()
    # Reihenfolge: 90, 50, 20
    assert lines[0].split("|") == ["NetzA", "90"]
    assert lines[1].split("|") == ["NetzB", "50"]
    assert lines[2].split("|") == ["NetzSchwach", "20"]


def test_scan_no_networks(monkeypatch, call_builtin):
    _fake_netsh(monkeypatch, "Keine Netzwerke sichtbar.")
    assert call_builtin("wifi_scan", []) == ""


def test_scan_hidden_ssid(monkeypatch, call_builtin):
    """Versteckte Netze haben SSID-Zeile ohne Wert -> '(versteckt)'."""
    fake = (
        "SSID 1 : \n"
        "    Authentifizierung       : WPA2-Personal\n"
        "    BSSID 1                 : 00:00:00:00:00:01\n"
        "         Signal             : 40%\n"
    )
    _fake_netsh(monkeypatch, fake)
    out = call_builtin("wifi_scan", [])
    assert out == "(versteckt)|40"


# --- WIFI_PROFILES ---------------------------------------------------

_NETSH_PROFILES_DE = """
Profile auf der Schnittstelle WLAN:

Gruppenrichtlinienprofile (schreibgeschuetzt)
---------------------------------
    <Keine>

Alle Benutzerprofile     : HomeNet
Alle Benutzerprofile     : OfficeNet
Alle Benutzerprofile     : CafeWLAN
"""

_NETSH_PROFILES_EN = """
Profiles on interface Wi-Fi:

Group policy profiles (read only)
---------------------------------
    <None>

User profiles
-------------
    All User Profile     : HomeNet
    All User Profile     : OfficeNet
"""


def test_profiles_de(monkeypatch, call_builtin):
    _fake_netsh(monkeypatch, _NETSH_PROFILES_DE)
    names = call_builtin("wifi_profiles", []).splitlines()
    assert "HomeNet" in names
    assert "OfficeNet" in names
    assert "CafeWLAN" in names


def test_profiles_en(monkeypatch, call_builtin):
    _fake_netsh(monkeypatch, _NETSH_PROFILES_EN)
    names = call_builtin("wifi_profiles", []).splitlines()
    assert "HomeNet" in names
    assert "OfficeNet" in names


# --- WIFI_DISCONNECT / WIFI_DELETE_PROFILE ---------------------------

def test_disconnect_returns_true_on_zero_rc(monkeypatch, call_builtin):
    _fake_netsh(monkeypatch, "Disconnect requested.", returncode=0)
    assert call_builtin("wifi_disconnect", []) is True


def test_disconnect_returns_false_on_nonzero_rc(monkeypatch, call_builtin):
    _fake_netsh(monkeypatch, "no interface", returncode=1)
    assert call_builtin("wifi_disconnect", []) is False


def test_delete_profile_empty_name_raises(call_builtin):
    if sys.platform != "win32":
        pytest.skip("WIFI_DELETE_PROFILE checkt Plattform vor Name")
    with pytest.raises(GBRuntimeError, match="leer"):
        call_builtin("wifi_delete_profile", [""])


# --- Plattform-Check ------------------------------------------------

@pytest.mark.skipif(sys.platform == "win32",
                    reason="auf Nicht-Windows liefern die Builtins Plattform-Fehler")
def test_current_errors_on_non_windows(call_builtin):
    with pytest.raises(GBRuntimeError, match="nur unter Windows"):
        call_builtin("wifi_current", [])


def test_available_returns_bool_without_crash(call_builtin):
    out = call_builtin("wifi_available", [])
    assert isinstance(out, bool)


# --- WIFI_CONNECT (smoke - mockt netsh-Calls) -----------------------

def test_connect_empty_ssid_raises(call_builtin):
    if sys.platform != "win32":
        pytest.skip("WIFI_CONNECT checkt Plattform vor SSID")
    with pytest.raises(GBRuntimeError, match="SSID"):
        call_builtin("wifi_connect", ["", "x"])


@pytest.mark.skipif(sys.platform != "win32", reason="netsh nur auf Windows")
def test_connect_calls_add_profile_then_connect(monkeypatch, call_builtin):
    """Reiht 'add profile' und 'connect' korrekt nacheinander."""
    calls = []

    def fake_run(args):
        calls.append(list(args))
        return 0, "OK"

    monkeypatch.setattr(wifi_mod, "_run_netsh", fake_run)

    ok = call_builtin("wifi_connect", ["TestNet", "passwd"])
    assert ok is True
    assert len(calls) == 2
    assert calls[0][:3] == ["wlan", "add", "profile"]
    assert calls[1][:2] == ["wlan", "connect"]
    # SSID muss im connect-Call stehen
    assert any("TestNet" in arg for arg in calls[1])
