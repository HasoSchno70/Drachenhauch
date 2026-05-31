"""Tests fuer das net-Modul (TCP/UDP-Sockets).

Wir nutzen Loopback (127.0.0.1) und Port 0 (= OS-zugewiesener freier Port),
damit die Tests nicht miteinander konkurrieren. Jeder Test schliesst seine
Sockets explizit -- ohne das wuerden flapping Tests passieren wenn der OS
sie nicht schnell genug freigibt.
"""
import time

import pytest

from gamebasic.modules import load_module
from gamebasic.errors import GBRuntimeError, TypeMismatchError


@pytest.fixture(scope="module", autouse=True)
def _load_net():
    assert load_module("net")


# --- TCP -----------------------------------------------------------

def test_tcp_listen_returns_listener(call_builtin):
    lst = call_builtin("net_tcp_listen", [0])
    from gamebasic.modules.net import _TCPListener
    assert isinstance(lst, _TCPListener)
    assert lst.port > 0
    call_builtin("net_close_listener", [lst])


def test_tcp_listen_invalid_port_raises(call_builtin):
    with pytest.raises(GBRuntimeError, match="out of range"):
        call_builtin("net_tcp_listen", [99999])


def test_tcp_accept_non_blocking_returns_nil(call_builtin):
    """Ohne wartenden Client liefert ACCEPT NIL (None)."""
    lst = call_builtin("net_tcp_listen", [0])
    res = call_builtin("net_tcp_accept", [lst])
    assert res is None
    call_builtin("net_close_listener", [lst])


def test_tcp_full_roundtrip(call_builtin):
    """Server + Client: Connect, Send, Recv, Close."""
    lst = call_builtin("net_tcp_listen", [0])
    port = lst.port
    client = call_builtin("net_tcp_connect", ["127.0.0.1", port])

    # Kurze Schleife: ACCEPT bis es klappt (auch unter Last sollte das
    # innerhalb 1s passen, das Loopback ist instant).
    server_sock = None
    for _ in range(50):
        server_sock = call_builtin("net_tcp_accept", [lst])
        if server_sock is not None:
            break
        time.sleep(0.01)
    assert server_sock is not None

    # Client schickt, Server liest
    sent = call_builtin("net_send", [client, "hallo"])
    assert sent == 5  # 5 Bytes

    # ACCEPT in non-blocking -- recv koennte initial leer sein
    received = ""
    for _ in range(50):
        received = call_builtin("net_recv", [server_sock, 1024])
        if received:
            break
        time.sleep(0.01)
    assert received == "hallo"

    # Cleanup
    call_builtin("net_close", [client])
    call_builtin("net_close", [server_sock])
    call_builtin("net_close_listener", [lst])


def test_tcp_peer_info(call_builtin):
    lst = call_builtin("net_tcp_listen", [0])
    client = call_builtin("net_tcp_connect", ["127.0.0.1", lst.port])
    assert call_builtin("net_peer_addr", [client]) == "127.0.0.1"
    assert call_builtin("net_peer_port", [client]) == lst.port
    call_builtin("net_close", [client])
    call_builtin("net_close_listener", [lst])


def test_tcp_send_validates_socket_type(call_builtin):
    with pytest.raises(TypeMismatchError, match="NET_SOCKET"):
        call_builtin("net_send", ["not a socket", "hi"])


def test_tcp_set_timeout_no_crash(call_builtin):
    lst = call_builtin("net_tcp_listen", [0])
    client = call_builtin("net_tcp_connect", ["127.0.0.1", lst.port])
    call_builtin("net_set_timeout", [client, 0])    # non-blocking
    call_builtin("net_set_timeout", [client, 100])  # 100ms
    call_builtin("net_set_timeout", [client, -1])   # blocking
    call_builtin("net_close", [client])
    call_builtin("net_close_listener", [lst])


# --- UDP -----------------------------------------------------------

def test_udp_bind_returns_socket(call_builtin):
    s = call_builtin("net_udp_bind", [0])
    from gamebasic.modules.net import _UDPSocket
    assert isinstance(s, _UDPSocket)
    assert s.bound_port > 0
    call_builtin("net_udp_close", [s])


def test_udp_open_unbound(call_builtin):
    s = call_builtin("net_udp_open", [])
    from gamebasic.modules.net import _UDPSocket
    assert isinstance(s, _UDPSocket)
    call_builtin("net_udp_close", [s])


def test_udp_send_recv(call_builtin):
    """Client sendet via OPEN, Server empfaengt via BIND."""
    server = call_builtin("net_udp_bind", [0])
    server_port = server.bound_port
    client = call_builtin("net_udp_open", [])
    sent = call_builtin("net_udp_send",
                        [client, "127.0.0.1", server_port, "ping"])
    assert sent == 4

    received = ""
    for _ in range(50):
        received = call_builtin("net_udp_recv", [server, 1024])
        if received:
            break
        time.sleep(0.01)
    assert received == "ping"

    last = call_builtin("net_udp_last_from", [server])
    assert last.startswith("127.0.0.1:")
    call_builtin("net_udp_close", [client])
    call_builtin("net_udp_close", [server])


def test_udp_recv_empty_when_nothing_pending(call_builtin):
    s = call_builtin("net_udp_bind", [0])
    received = call_builtin("net_udp_recv", [s, 1024])
    assert received == ""
    call_builtin("net_udp_close", [s])


def test_udp_invalid_port_raises(call_builtin):
    with pytest.raises(GBRuntimeError, match="out of range"):
        call_builtin("net_udp_bind", [70000])


def test_udp_send_validates_socket(call_builtin):
    with pytest.raises(TypeMismatchError, match="NET_UDP"):
        call_builtin("net_udp_send", ["not a socket", "127.0.0.1", 1234, "hi"])


def test_udp_last_from_empty_initially(call_builtin):
    s = call_builtin("net_udp_bind", [0])
    assert call_builtin("net_udp_last_from", [s]) == ""
    call_builtin("net_udp_close", [s])
