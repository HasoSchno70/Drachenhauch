"""Networking-Modul fuer GameBasic: TCP + UDP via stdlib-Sockets.

Cross-platform (Windows/Linux/Mac), keine externen Abhaengigkeiten.
Default-Modus: non-blocking mit 0ms-Timeout, was fuer Game-Loops das
naheliegende Verhalten ist (kein Frame-Stall durch wartendes recv).
Wer blocking-Verhalten will, ruft `NET_SET_TIMEOUT(sock, -1)`.

TCP-Server:
    NET_TCP_LISTEN(port)              -> NET_LISTENER
    NET_TCP_ACCEPT(listener)          -> NET_SOCKET | NIL
        (NIL wenn gerade keine Verbindung wartet -- non-blocking-Modus)

TCP-Client:
    NET_TCP_CONNECT(host, port)       -> NET_SOCKET

TCP-Common:
    NET_SEND(sock, text)              -> INTEGER (Bytes geschrieben)
    NET_RECV(sock, max_bytes)         -> STRING ("" wenn nichts)
    NET_PEER_ADDR(sock)               -> STRING
    NET_PEER_PORT(sock)               -> INTEGER
    NET_CLOSE(sock)
    NET_CLOSE_LISTENER(listener)
    NET_SET_TIMEOUT(sock, ms)
        ms = 0     -> non-blocking
        ms < 0     -> blocking (kein Timeout)
        ms > 0     -> Timeout in Millisekunden

UDP:
    NET_UDP_BIND(port)                -> NET_UDP (an localhost gebunden)
    NET_UDP_OPEN()                    -> NET_UDP (nur Client, nicht gebunden)
    NET_UDP_SEND(sock, host, port, text) -> INTEGER
    NET_UDP_RECV(sock, max_bytes)     -> STRING ("" wenn nichts)
    NET_UDP_LAST_FROM(sock)           -> STRING ("host:port" des letzten RECV)
    NET_UDP_SET_TIMEOUT(sock, ms)
    NET_UDP_CLOSE(sock)

Encoding: alles UTF-8. Wer Binary-Daten schicken muss, kodiert via
HEX$/CHR$ -- ein Byte-Type haben wir bewusst nicht. Fuer Game-Networking
ist das in 99% der Faelle ausreichend (JSON-Pakete, Text-Commands).
"""
from __future__ import annotations

import socket as _socket

from ..builtins_registry import builtin
from ..errors import GBRuntimeError, TypeMismatchError
from . import register_type


# --- Wrapper-Typen --------------------------------------------------

class _TCPListener:
    __slots__ = ("sock", "port")

    def __init__(self, sock, port):
        self.sock = sock
        self.port = port

    def __repr__(self):
        return f"<NET_LISTENER port={self.port}>"


class _TCPSocket:
    __slots__ = ("sock", "peer_host", "peer_port")

    def __init__(self, sock, peer_host="", peer_port=0):
        self.sock = sock
        self.peer_host = peer_host
        self.peer_port = peer_port

    def __repr__(self):
        return f"<NET_SOCKET {self.peer_host}:{self.peer_port}>"


class _UDPSocket:
    __slots__ = ("sock", "bound_port", "last_from")

    def __init__(self, sock, bound_port=0):
        self.sock = sock
        self.bound_port = bound_port
        self.last_from = ("", 0)

    def __repr__(self):
        return f"<NET_UDP bound={self.bound_port}>"


register_type("net_listener", _TCPListener)
register_type("net_socket", _TCPSocket)
register_type("net_udp", _UDPSocket)


# --- Type-Helpers ---------------------------------------------------

def _check_listener(v, fn: str) -> _TCPListener:
    if not isinstance(v, _TCPListener):
        raise TypeMismatchError(f"{fn} erwartet NET_LISTENER")
    return v


def _check_socket(v, fn: str) -> _TCPSocket:
    if not isinstance(v, _TCPSocket):
        raise TypeMismatchError(f"{fn} erwartet NET_SOCKET")
    return v


def _check_udp(v, fn: str) -> _UDPSocket:
    if not isinstance(v, _UDPSocket):
        raise TypeMismatchError(f"{fn} erwartet NET_UDP")
    return v


def _check_int(v, fn: str, label: str = "INTEGER") -> int:
    if isinstance(v, bool) or not isinstance(v, int):
        raise TypeMismatchError(f"{fn} erwartet {label}")
    return v


def _check_str(v, fn: str, label: str = "STRING") -> str:
    if not isinstance(v, str):
        raise TypeMismatchError(f"{fn} erwartet {label}")
    return v


def _apply_timeout(sock, ms: int) -> None:
    """ms == 0: non-blocking, ms < 0: blocking, ms > 0: Timeout."""
    if ms == 0:
        sock.setblocking(False)
    elif ms < 0:
        sock.setblocking(True)
        sock.settimeout(None)
    else:
        sock.settimeout(ms / 1000.0)


# --- TCP -----------------------------------------------------------

@builtin("NET_TCP_LISTEN", arity=1, types=("int",))
def _b_listen(port):
    if not (0 <= port <= 65535):
        raise GBRuntimeError(f"NET_TCP_LISTEN: port out of range: {port}")
    s = _socket.socket(_socket.AF_INET, _socket.SOCK_STREAM)
    try:
        s.setsockopt(_socket.SOL_SOCKET, _socket.SO_REUSEADDR, 1)
        s.bind(("", port))
        s.listen(8)
        s.setblocking(False)  # default: ACCEPT ist non-blocking
    except OSError as exc:
        s.close()
        raise GBRuntimeError(f"NET_TCP_LISTEN: {exc}")
    actual_port = s.getsockname()[1]
    return _TCPListener(s, actual_port)


@builtin("NET_LISTENER_PORT", arity=1)
def _b_listener_port(lst):
    """Liefert den tatsaechlich gebundenen Port. Nuetzlich nach
    `NET_TCP_LISTEN(0)` -- der OS waehlt einen freien Port und gibt
    ihn hier raus. Fuer NET_LISTENER, NET_UDP-Variante ist
    `NET_UDP_PORT`."""
    lst = _check_listener(lst, "NET_LISTENER_PORT")
    return lst.port


@builtin("NET_UDP_PORT", arity=1)
def _b_udp_port(sock):
    """Gebundener Port eines UDP-Sockets (0 wenn ungebunden via OPEN)."""
    sock = _check_udp(sock, "NET_UDP_PORT")
    return sock.bound_port


@builtin("NET_TCP_ACCEPT", arity=1)
def _b_accept(lst):
    lst = _check_listener(lst, "NET_TCP_ACCEPT")
    try:
        conn, addr = lst.sock.accept()
    except BlockingIOError:
        return None  # NIL: niemand verbindet sich gerade
    except OSError as exc:
        raise GBRuntimeError(f"NET_TCP_ACCEPT: {exc}")
    conn.setblocking(False)
    host, port = addr[0], addr[1]
    return _TCPSocket(conn, host, port)


@builtin("NET_TCP_CONNECT", arity=2, types=("str", "int"))
def _b_connect(host, port):
    if not (0 <= port <= 65535):
        raise GBRuntimeError(f"NET_TCP_CONNECT: port out of range: {port}")
    s = _socket.socket(_socket.AF_INET, _socket.SOCK_STREAM)
    try:
        s.settimeout(5.0)  # 5s connect-Timeout, danach non-blocking
        s.connect((host, port))
        s.setblocking(False)
    except OSError as exc:
        s.close()
        raise GBRuntimeError(f"NET_TCP_CONNECT: {exc}")
    return _TCPSocket(s, host, port)


@builtin("NET_SEND", arity=2)
def _b_send(sock, text):
    sock = _check_socket(sock, "NET_SEND")
    text = _check_str(text, "NET_SEND")
    try:
        n = sock.sock.send(text.encode("utf-8"))
        return int(n)
    except (BlockingIOError, OSError) as exc:
        raise GBRuntimeError(f"NET_SEND: {exc}")


@builtin("NET_RECV", arity=2)
def _b_recv(sock, max_bytes):
    sock = _check_socket(sock, "NET_RECV")
    n = _check_int(max_bytes, "NET_RECV", "max_bytes")
    if n <= 0:
        raise GBRuntimeError("NET_RECV: max_bytes muss > 0 sein")
    try:
        data = sock.sock.recv(n)
    except BlockingIOError:
        return ""
    except OSError as exc:
        raise GBRuntimeError(f"NET_RECV: {exc}")
    return data.decode("utf-8", errors="replace")


@builtin("NET_PEER_ADDR", arity=1)
def _b_peer_addr(sock):
    return _check_socket(sock, "NET_PEER_ADDR").peer_host


@builtin("NET_PEER_PORT", arity=1)
def _b_peer_port(sock):
    return _check_socket(sock, "NET_PEER_PORT").peer_port


@builtin("NET_CLOSE", arity=1)
def _b_close(sock):
    sock = _check_socket(sock, "NET_CLOSE")
    try:
        sock.sock.close()
    except OSError:
        pass
    return None


@builtin("NET_CLOSE_LISTENER", arity=1)
def _b_close_listener(lst):
    lst = _check_listener(lst, "NET_CLOSE_LISTENER")
    try:
        lst.sock.close()
    except OSError:
        pass
    return None


@builtin("NET_SET_TIMEOUT", arity=2, types=("any", "int"))
def _b_set_timeout(sock, ms):
    sock = _check_socket(sock, "NET_SET_TIMEOUT")
    _apply_timeout(sock.sock, ms)
    return None


# --- UDP -----------------------------------------------------------

@builtin("NET_UDP_BIND", arity=1, types=("int",))
def _b_udp_bind(port):
    if not (0 <= port <= 65535):
        raise GBRuntimeError(f"NET_UDP_BIND: port out of range: {port}")
    s = _socket.socket(_socket.AF_INET, _socket.SOCK_DGRAM)
    try:
        s.setsockopt(_socket.SOL_SOCKET, _socket.SO_REUSEADDR, 1)
        s.bind(("", port))
        s.setblocking(False)
    except OSError as exc:
        s.close()
        raise GBRuntimeError(f"NET_UDP_BIND: {exc}")
    actual_port = s.getsockname()[1]
    return _UDPSocket(s, actual_port)


@builtin("NET_UDP_OPEN", arity=0)
def _b_udp_open():
    s = _socket.socket(_socket.AF_INET, _socket.SOCK_DGRAM)
    s.setblocking(False)
    return _UDPSocket(s, 0)


@builtin("NET_UDP_SEND", arity=4)
def _b_udp_send(sock, host, port, text):
    sock = _check_udp(sock, "NET_UDP_SEND")
    host = _check_str(host, "NET_UDP_SEND", "host")
    port = _check_int(port, "NET_UDP_SEND", "port")
    text = _check_str(text, "NET_UDP_SEND", "text")
    try:
        n = sock.sock.sendto(text.encode("utf-8"), (host, port))
        return int(n)
    except OSError as exc:
        raise GBRuntimeError(f"NET_UDP_SEND: {exc}")


@builtin("NET_UDP_RECV", arity=2)
def _b_udp_recv(sock, max_bytes):
    sock = _check_udp(sock, "NET_UDP_RECV")
    n = _check_int(max_bytes, "NET_UDP_RECV", "max_bytes")
    if n <= 0:
        raise GBRuntimeError("NET_UDP_RECV: max_bytes muss > 0 sein")
    try:
        data, addr = sock.sock.recvfrom(n)
    except BlockingIOError:
        return ""
    except OSError as exc:
        raise GBRuntimeError(f"NET_UDP_RECV: {exc}")
    sock.last_from = (addr[0], addr[1])
    return data.decode("utf-8", errors="replace")


@builtin("NET_UDP_LAST_FROM", arity=1)
def _b_udp_last_from(sock):
    sock = _check_udp(sock, "NET_UDP_LAST_FROM")
    if not sock.last_from[0]:
        return ""
    return f"{sock.last_from[0]}:{sock.last_from[1]}"


@builtin("NET_UDP_SET_TIMEOUT", arity=2, types=("any", "int"))
def _b_udp_set_timeout(sock, ms):
    sock = _check_udp(sock, "NET_UDP_SET_TIMEOUT")
    _apply_timeout(sock.sock, ms)
    return None


@builtin("NET_UDP_CLOSE", arity=1)
def _b_udp_close(sock):
    sock = _check_udp(sock, "NET_UDP_CLOSE")
    try:
        sock.sock.close()
    except OSError:
        pass
    return None
