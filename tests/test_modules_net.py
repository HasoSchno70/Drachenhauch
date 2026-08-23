"""Tests fuer das net-Modul (TCP/UDP-Sockets).

Golden-Tests gegen `dhrt` (Stufe B): jeder Test ist ein eigenstaendiges
GB-Programm, das Server+Client im selben Prozess ueber Loopback (127.0.0.1,
Port 0) verbindet. nil-Check via `IS_NIL()`-Builtin (nicht `IS NIL` -- das ist
gar kein Parser-Konstrukt, weder TW noch dhrt; nur die Doku behauptet es).
Frueher via `call_builtin` gegen die Python-Impl (in Phase 8 geloescht).

Warteschleifen mit SLEEP
------------------------
Die Tests fragen `NET_TCP_ACCEPT`/`NET_RECV` in einer Schleife nach, weil die
Sockets nicht blockieren. Diese Schleifen liefen bis 2026-08-20 OHNE Pause --
50 Durchgaenge in Mikrosekunden. Unter Linux und Windows kamen die Daten auf
Loopback trotzdem rechtzeitig an, auf macOS nicht: `test_tcp_full_roundtrip`
fiel dort beim ersten CI-Lauf um, weil `NET_RECV` fuenfzigmal leer lieferte.
Ein `SLEEP(2)` je Durchgang macht aus 50 Mikrosekunden 100 Millisekunden
Geduld. Das ist kein Zugestaendnis an macOS, sondern die Behebung einer
Wettlaufbedingung, die auf den anderen Systemen nur zufaellig gewann.

Nachtrag 2026-08-23: zwei EMPFANGS-Schleifen waren dabei uebersehen worden und
liefen weiter ohne Pause -- `test_tcp_recv_reassembles_multibyte_char_split_across_reads`
fiel deshalb auf macOS um (`['FALSE', 'FALSE']` statt `['TRUE', 'FALSE']`: die
Schleife war durch, bevor alle acht Bytes da waren, das Ergebnis also nur
abgeschnitten -- nicht kaputt kodiert). Beide warten jetzt. Die Umlaut-Schleife
zaehlt zusaetzlich mit, statt stur 100-mal zu lesen: sie bricht ab, sobald die
erwartete Zeichenzahl da ist, und schlaeft nur, wenn wirklich nichts kam. Damit
ist sie im Normalfall SCHNELLER als vorher und wartet trotzdem bis 400 ms.

"""
import pytest

from drachenhauch.errors import DHRuntimeError


def _lines(out):
    return [l.strip() for l in out.split("\n") if l.strip()]


# --- TCP -----------------------------------------------------------

def test_tcp_listen_returns_listener(run_gb):
    out = _lines(run_gb('IMPORT "net"\nDIM l AS NET_LISTENER\nl = NET_TCP_LISTEN(0)\n'
                        "PRINT NET_LISTENER_PORT(l) > 0\nNET_CLOSE_LISTENER(l)\n"))
    assert out == ["TRUE"]


def test_tcp_listen_invalid_port_raises(run_gb):
    with pytest.raises(DHRuntimeError, match="out of range"):
        run_gb('IMPORT "net"\nDIM l AS NET_LISTENER\nl = NET_TCP_LISTEN(99999)\n')


def test_tcp_accept_non_blocking_returns_nil(run_gb):
    out = _lines(run_gb('IMPORT "net"\nDIM l AS NET_LISTENER\nl = NET_TCP_LISTEN(0)\n'
                        "DIM s AS NET_SOCKET\ns = NET_TCP_ACCEPT(l)\n"
                        "PRINT IS_NIL(s)\nNET_CLOSE_LISTENER(l)\n"))
    assert out == ["TRUE"]


def test_tcp_full_roundtrip(run_gb):
    out = _lines(run_gb(
        'IMPORT "net"\nDIM l AS NET_LISTENER\nl = NET_TCP_LISTEN(0)\n'
        "DIM client AS NET_SOCKET\nclient = NET_TCP_CONNECT(\"127.0.0.1\", NET_LISTENER_PORT(l))\n"
        "DIM srv AS NET_SOCKET\nDIM i AS INTEGER\n"
        "FOR i = 1 TO 50\n    srv = NET_TCP_ACCEPT(l)\n    IF NOT IS_NIL(srv) THEN BREAK\n    SLEEP(2)\nNEXT\n"
        "PRINT IS_NIL(srv)\n"
        'PRINT NET_SEND(client, "hallo")\n'
        'DIM got AS STRING\nFOR i = 1 TO 50\n    got = NET_RECV(srv, 1024)\n'
        '    IF got <> "" THEN BREAK\n    SLEEP(2)\nNEXT\nPRINT got\n'
        "NET_CLOSE(client)\nNET_CLOSE(srv)\nNET_CLOSE_LISTENER(l)\n"))
    assert out == ["FALSE", "5", "hallo"]


def test_tcp_peer_info(run_gb):
    out = _lines(run_gb(
        'IMPORT "net"\nDIM l AS NET_LISTENER\nl = NET_TCP_LISTEN(0)\n'
        "DIM client AS NET_SOCKET\nclient = NET_TCP_CONNECT(\"127.0.0.1\", NET_LISTENER_PORT(l))\n"
        "PRINT NET_PEER_ADDR(client)\n"
        "PRINT NET_PEER_PORT(client) = NET_LISTENER_PORT(l)\n"
        "NET_CLOSE(client)\nNET_CLOSE_LISTENER(l)\n"))
    assert out == ["127.0.0.1", "TRUE"]


def test_tcp_send_validates_socket_type(run_gb):
    # dhrt nutzt Integer-Handles -> Typfehler lautet "erwartet INTEGER".
    with pytest.raises(DHRuntimeError, match="INTEGER"):
        run_gb('IMPORT "net"\nPRINT NET_SEND("not a socket", "hi")\n')


def test_tcp_set_timeout_no_crash(run_gb):
    out = _lines(run_gb(
        'IMPORT "net"\nDIM l AS NET_LISTENER\nl = NET_TCP_LISTEN(0)\n'
        "DIM client AS NET_SOCKET\nclient = NET_TCP_CONNECT(\"127.0.0.1\", NET_LISTENER_PORT(l))\n"
        "NET_SET_TIMEOUT(client, 0)\nNET_SET_TIMEOUT(client, 100)\nNET_SET_TIMEOUT(client, -1)\n"
        'PRINT "ok"\nNET_CLOSE(client)\nNET_CLOSE_LISTENER(l)\n'))
    assert out == ["ok"]


def test_tcp_is_connected_true_while_open(run_gb):
    out = _lines(run_gb(
        'IMPORT "net"\nDIM l AS NET_LISTENER\nl = NET_TCP_LISTEN(0)\n'
        "DIM client AS NET_SOCKET\nclient = NET_TCP_CONNECT(\"127.0.0.1\", NET_LISTENER_PORT(l))\n"
        "PRINT NET_IS_CONNECTED(client)\n"
        "NET_CLOSE(client)\nNET_CLOSE_LISTENER(l)\n"))
    assert out == ["TRUE"]


def test_tcp_is_connected_false_after_peer_closes(run_gb):
    # Server schliesst seine Seite -> Client muss das ueber NET_IS_CONNECTED
    # erkennen koennen (frueher wurde ein sauberes Schliessen genauso wie
    # "gerade nichts da" behandelt -- NET_RECV lief endlos leer weiter).
    out = _lines(run_gb(
        'IMPORT "net"\nDIM l AS NET_LISTENER\nl = NET_TCP_LISTEN(0)\n'
        "DIM client AS NET_SOCKET\nclient = NET_TCP_CONNECT(\"127.0.0.1\", NET_LISTENER_PORT(l))\n"
        "DIM srv AS NET_SOCKET\nDIM i AS INTEGER\n"
        "FOR i = 1 TO 50\n    srv = NET_TCP_ACCEPT(l)\n    IF NOT IS_NIL(srv) THEN BREAK\n    SLEEP(2)\nNEXT\n"
        "NET_CLOSE(srv)\n"
        "DIM waited AS INTEGER\nDIM dummy AS STRING\nwaited = 0\n"
        "WHILE NET_IS_CONNECTED(client) AND waited < 2000\n"
        "    dummy = NET_RECV(client, 1024)\n    SLEEP(20)\n    waited = waited + 20\n"
        "WEND\n"
        "PRINT NET_IS_CONNECTED(client)\n"
        "NET_CLOSE(client)\nNET_CLOSE_LISTENER(l)\n"))
    assert out == ["FALSE"]


def test_tcp_listen_optional_bind_addr_backward_compat(run_gb):
    # Zweites Arg ist optional -- alter 1-Arg-Aufruf muss unveraendert
    # funktionieren (IPv4 auf allen Interfaces).
    out = _lines(run_gb('IMPORT "net"\nDIM l AS NET_LISTENER\nl = NET_TCP_LISTEN(0)\n'
                        "PRINT NET_LISTENER_PORT(l) > 0\nNET_CLOSE_LISTENER(l)\n"))
    assert out == ["TRUE"]


def test_tcp_listen_ipv6_bind_addr(run_gb):
    out = _lines(run_gb(
        'IMPORT "net"\nDIM l AS NET_LISTENER\nl = NET_TCP_LISTEN(0, "::")\n'
        "PRINT NET_LISTENER_PORT(l) > 0\nNET_CLOSE_LISTENER(l)\n"))
    assert out == ["TRUE"]


def test_tcp_recv_reassembles_multibyte_char_split_across_reads(run_gb):
    # NET_RECV(sock, 1) erzwingt Ein-Byte-Reads -- damit muss selbst ein
    # Umlaut, dessen UTF-8-Kodierung ueber zwei RECV-Aufrufe verteilt
    # ankommt, korrekt rekonstruiert werden (vorher: '�'-Ersatzzeichen).
    out = _lines(run_gb(
        'IMPORT "net"\nDIM l AS NET_LISTENER\nl = NET_TCP_LISTEN(0)\n'
        "DIM client AS NET_SOCKET\nclient = NET_TCP_CONNECT(\"127.0.0.1\", NET_LISTENER_PORT(l))\n"
        "DIM srv AS NET_SOCKET\nDIM i AS INTEGER\n"
        "FOR i = 1 TO 50\n    srv = NET_TCP_ACCEPT(l)\n    IF NOT IS_NIL(srv) THEN BREAK\n    SLEEP(2)\nNEXT\n"
        'DIM msg AS STRING\nmsg = "a" + CHR$(228) + CHR$(246) + CHR$(252) + "b"\n'
        "NET_SEND(client, msg)\n"
        'DIM result AS STRING\nresult = ""\nDIM chunk AS STRING\n'
        "FOR i = 1 TO 200\n    chunk = NET_RECV(srv, 1)\n    result = result + chunk\n"
        '    IF LEN(result) = LEN(msg) THEN BREAK\n    IF chunk = "" THEN SLEEP(2)\nNEXT\n'
        "PRINT result = msg\n"
        "PRINT (CHR$(65533) IN result)\n"
        "NET_CLOSE(client)\nNET_CLOSE(srv)\nNET_CLOSE_LISTENER(l)\n"))
    assert out == ["TRUE", "FALSE"]


# --- UDP -----------------------------------------------------------

def test_udp_bind_returns_socket(run_gb):
    out = _lines(run_gb('IMPORT "net"\nDIM s AS NET_UDP\ns = NET_UDP_BIND(0)\n'
                        "PRINT NET_UDP_PORT(s) > 0\nNET_UDP_CLOSE(s)\n"))
    assert out == ["TRUE"]


def test_udp_open_unbound(run_gb):
    out = _lines(run_gb('IMPORT "net"\nDIM s AS NET_UDP\ns = NET_UDP_OPEN()\n'
                        'PRINT "ok"\nNET_UDP_CLOSE(s)\n'))
    assert out == ["ok"]


def test_udp_send_recv(run_gb):
    out = _lines(run_gb(
        'IMPORT "net"\nDIM srv AS NET_UDP\nsrv = NET_UDP_BIND(0)\n'
        "DIM cl AS NET_UDP\ncl = NET_UDP_OPEN()\n"
        'PRINT NET_UDP_SEND(cl, "127.0.0.1", NET_UDP_PORT(srv), "ping")\n'
        'DIM got AS STRING\nDIM i AS INTEGER\n'
        'FOR i = 1 TO 50\n    got = NET_UDP_RECV(srv, 1024)\n    IF got <> "" THEN BREAK\n    SLEEP(2)\nNEXT\n'
        "PRINT got\n"
        'DIM last AS STRING\nlast = NET_UDP_LAST_FROM(srv)\n'
        'PRINT LEFT$(last, 10)\n'
        "NET_UDP_CLOSE(cl)\nNET_UDP_CLOSE(srv)\n"))
    assert out == ["4", "ping", "127.0.0.1:"]   # NET_UDP_LAST_FROM = "ip:port"


def test_udp_recv_empty_when_nothing_pending(run_gb):
    out = _lines(run_gb('IMPORT "net"\nDIM s AS NET_UDP\ns = NET_UDP_BIND(0)\n'
                        'PRINT "[" + NET_UDP_RECV(s, 1024) + "]"\nNET_UDP_CLOSE(s)\n'))
    assert out == ["[]"]


def test_udp_invalid_port_raises(run_gb):
    with pytest.raises(DHRuntimeError, match="out of range"):
        run_gb('IMPORT "net"\nDIM s AS NET_UDP\ns = NET_UDP_BIND(70000)\n')


def test_udp_send_validates_socket(run_gb):
    with pytest.raises(DHRuntimeError, match="INTEGER"):
        run_gb('IMPORT "net"\nPRINT NET_UDP_SEND("not a socket", "127.0.0.1", 1234, "hi")\n')


def test_udp_last_from_empty_initially(run_gb):
    out = _lines(run_gb('IMPORT "net"\nDIM s AS NET_UDP\ns = NET_UDP_BIND(0)\n'
                        'PRINT "[" + NET_UDP_LAST_FROM(s) + "]"\nNET_UDP_CLOSE(s)\n'))
    assert out == ["[]"]


def _verbundenes_paar() -> str:
    """GB-Vorspann: Listener + verbundener Client + angenommener Server-Socket."""
    return ('IMPORT "net"\n'
            'DIM l AS NET_LISTENER\nDIM c AS NET_SOCKET\nDIM s AS NET_SOCKET\n'
            'l = NET_TCP_LISTEN(0)\n'
            'c = NET_TCP_CONNECT("127.0.0.1", NET_LISTENER_PORT(l))\n'
            'WHILE IS_NIL(s)\n    s = NET_TCP_ACCEPT(l)\nWEND\n')


def test_set_timeout_null_ist_non_blocking(run_gb):
    """`NET_SET_TIMEOUT(sock, 0)` stellt NON-BLOCKING her, nicht blockierend.

    Die Doku behauptete jahrelang das Gegenteil ("ms = 0 setzt
    vollblockierendes Lesen"), und der vorhandene Test prueft nur, dass der
    Aufruf nicht abstuerzt -- deshalb fiel es nie auf. Hier steht jetzt die
    Bedeutung selbst.
    """
    out = _lines(run_gb(_verbundenes_paar() +
                        'DIM t0 AS INTEGER\nDIM r AS STRING\n'
                        'NET_SET_TIMEOUT(c, 0)\n'
                        't0 = MILLIS()\n'
                        'r = NET_RECV(c, 64)\n'
                        'PRINT MILLIS() - t0 < 100\n'
                        'PRINT r = ""\n'
                        'NET_CLOSE(c)\nNET_CLOSE(s)\nNET_CLOSE_LISTENER(l)\n'))
    assert out == ["TRUE", "TRUE"]


def test_set_timeout_positiv_wartet_und_wirft_nicht(run_gb):
    """`ms > 0` wartet hoechstens so lange -- und ein abgelaufener Timeout ist
    KEIN Fehler, sondern ein Leerstring (die Doku versprach eine Exception)."""
    out = _lines(run_gb(_verbundenes_paar() +
                        'DIM t0 AS INTEGER\nDIM r AS STRING\n'
                        'NET_SET_TIMEOUT(c, 300)\n'
                        't0 = MILLIS()\n'
                        'r = NET_RECV(c, 64)\n'
                        'PRINT MILLIS() - t0 >= 250\n'
                        'PRINT r = ""\n'
                        'NET_CLOSE(c)\nNET_CLOSE(s)\nNET_CLOSE_LISTENER(l)\n'))
    assert out == ["TRUE", "TRUE"]
