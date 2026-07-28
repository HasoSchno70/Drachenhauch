"""Tests fuer das `mqtt`-Modul (MQTT-3.1.1-Client, QoS 0). Anders als bei
serial/firmata/usb/bt/wifi braucht MQTT kein echtes externes Geraet -- ein
winziger, unabhaengig geschriebener Test-Broker (reines Python, teilt keinen
Code mit dem Rust-Client) reicht fuer einen echten End-to-End-Test ueber
einen echten TCP-Socket.

Die reine Protokoll-Logik (Byte-Layout, Remaining-Length-Encoding,
Nachrichten-Parsing) wird bereits in Rust getestet
(rust/gb_runtime/src/mqtt.rs, `cargo test`) -- hier zusaetzlich der
End-to-End-Pfad durch die echte VM.
"""
import socket
import struct
import threading

import pytest


def test_mqtt_builtins_registered():
    from gamebasic.editor_qt.gbrt_meta import builtin_names_lower
    expected = {
        "mqtt_connect", "mqtt_disconnect", "mqtt_is_connected",
        "mqtt_publish", "mqtt_subscribe", "mqtt_update",
        "mqtt_next_message", "mqtt_message_topic", "mqtt_message_payload",
    }
    assert expected <= builtin_names_lower()


def test_mqtt_is_a_known_module():
    from gamebasic.modules import is_known_module
    assert is_known_module("mqtt")


def test_mqtt_connect_refused_is_catchable(run_gb):
    """Kein Broker am Port -- MQTT_CONNECT muss einen fangbaren Fehler werfen,
    nicht abstuerzen (kurzer Timeout, kein 5s-Warten auf CONNACK noetig, weil
    schon der TCP-Connect selbst scheitert)."""
    from gamebasic.errors import GBRuntimeError
    with pytest.raises(GBRuntimeError, match="MQTT_CONNECT"):
        run_gb('''
IMPORT "mqtt"
DIM h AS MQTT_HANDLE
h = MQTT_CONNECT("127.0.0.1", 18999, "gb-test")
''')


# --- Minimaler, unabhaengiger Test-Broker (QoS 0) ---------------------

def _encode_remaining_length(n):
    out = bytearray()
    while True:
        b = n % 128
        n //= 128
        if n > 0:
            b |= 0x80
        out.append(b)
        if n == 0:
            break
    return bytes(out)


def _decode_remaining_length(sock):
    multiplier = 1
    value = 0
    while True:
        b = sock.recv(1)
        if not b:
            return None
        byte = b[0]
        value += (byte & 0x7F) * multiplier
        multiplier *= 128
        if byte & 0x80 == 0:
            break
    return value


def _recv_exact(sock, n):
    buf = b""
    while len(buf) < n:
        chunk = sock.recv(n - len(buf))
        if not chunk:
            return None
        buf += chunk
    return buf


def _decode_str(buf, off):
    (length,) = struct.unpack_from(">H", buf, off)
    return buf[off + 2:off + 2 + length].decode("utf-8"), off + 2 + length


def _run_mini_broker(port_holder, ready, stop):
    """Ein Client-Zyklus: CONNECT->CONNACK, dann SUBSCRIBE/PUBLISH bedienen
    und jedes PUBLISH an denselben Client zurueckspiegeln (reicht fuer den
    Round-Trip-Test), bis DISCONNECT oder Verbindungsende."""
    srv = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    srv.bind(("127.0.0.1", 0))
    srv.listen(1)
    port_holder.append(srv.getsockname()[1])
    srv.settimeout(10)
    ready.set()
    try:
        conn, _ = srv.accept()
    except socket.timeout:
        return
    conn.settimeout(10)
    try:
        first = _recv_exact(conn, 1)
        if not first:
            return
        rem_len = _decode_remaining_length(conn)
        _recv_exact(conn, rem_len)  # CONNECT-Body ignorieren
        conn.sendall(bytes([0x20, 0x02, 0x00, 0x00]))  # CONNACK, accepted

        while not stop.is_set():
            first = _recv_exact(conn, 1)
            if not first:
                return
            ptype = first[0] >> 4
            rem_len = _decode_remaining_length(conn)
            body = _recv_exact(conn, rem_len) if rem_len else b""
            if ptype == 12:  # PINGREQ
                conn.sendall(bytes([0xD0, 0x00]))
            elif ptype == 8:  # SUBSCRIBE
                packet_id = body[0:2]
                conn.sendall(bytes([0x90]) + _encode_remaining_length(3) + packet_id + bytes([0x00]))
            elif ptype == 3:  # PUBLISH
                topic, off = _decode_str(body, 0)
                payload = body[off:]
                pub_body = struct.pack(">H", len(topic)) + topic.encode("utf-8") + payload
                conn.sendall(bytes([0x30]) + _encode_remaining_length(len(pub_body)) + pub_body)
            elif ptype == 14:  # DISCONNECT
                return
    finally:
        try:
            conn.close()
        except Exception:
            pass
        srv.close()


@pytest.fixture
def mini_broker():
    port_holder = []
    ready = threading.Event()
    stop = threading.Event()
    t = threading.Thread(target=_run_mini_broker, args=(port_holder, ready, stop), daemon=True)
    t.start()
    assert ready.wait(5), "Test-Broker ist nicht rechtzeitig gestartet"
    yield port_holder[0]
    stop.set()
    t.join(timeout=2)


def test_mqtt_publish_subscribe_round_trip(run_gb, mini_broker):
    port = mini_broker
    out = run_gb(f'''
IMPORT "mqtt"

DIM h AS MQTT_HANDLE
h = MQTT_CONNECT("127.0.0.1", {port}, "gb-e2e-test", 30)
PRINT "connected: ", MQTT_IS_CONNECTED(h)

MQTT_SUBSCRIBE(h, "sensors/temp")
MQTT_PUBLISH(h, "sensors/temp", "21.5")

DIM tries AS INTEGER
DIM got AS BOOLEAN
got = FALSE
FOR tries = 1 TO 40
    MQTT_UPDATE(h)
    IF MQTT_NEXT_MESSAGE(h) THEN
        PRINT "topic: ", MQTT_MESSAGE_TOPIC(h)
        PRINT "payload: ", MQTT_MESSAGE_PAYLOAD(h)
        got = TRUE
        BREAK
    END IF
    SLEEP(25)
NEXT
PRINT "got message: ", got

MQTT_DISCONNECT(h)
PRINT "connected after disconnect: ", MQTT_IS_CONNECTED(h)
''')
    lines = out.strip().splitlines()
    assert lines == [
        "connected:  TRUE",
        "topic:  sensors/temp",
        "payload:  21.5",
        "got message:  TRUE",
        "connected after disconnect:  FALSE",
    ]
