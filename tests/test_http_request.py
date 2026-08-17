"""WP C -- HTTP fuer echte Dienste: HTTP_REQUEST mit eigenen Kopfzeilen,
PUT/PATCH/DELETE, Rumpf und Antwort als BUFFER, Zeitgrenze, dauerhafte
Kopfzeilen, Hintergrund-Anfragen fuer alle Methoden.

Wie `test_modules_html.py` gegen einen lokalen Mock-Server im pytest-Prozess;
das GB-Programm laeuft im dhrt-Subprozess und macht echte localhost-Anfragen.
"""
import contextlib
import json
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

import pytest

from drachenhauch.errors import DHRuntimeError


class _EchoHandler(BaseHTTPRequestHandler):
    """Spiegelt zurueck, was wirklich ankam -- Methode, Kopfzeilen, Rumpf.

    Nur so laesst sich pruefen, dass eine Kopfzeile den Server auch erreicht;
    am Rueckgabewert allein saehe man es nicht.
    """

    verzoegerung = 0.0

    def log_message(self, *args, **kwargs):
        pass

    def _antworten(self):
        laenge = int(self.headers.get("Content-Length", 0))
        rumpf = self.rfile.read(laenge) if laenge else b""
        if _EchoHandler.verzoegerung:
            time.sleep(_EchoHandler.verzoegerung)
        if self.path == "/rohe-bytes":
            # Bytes, die absichtlich KEIN gueltiges UTF-8 sind.
            self.send_response(200)
            self.send_header("Content-Type", "application/octet-stream")
            self.end_headers()
            self.wfile.write(bytes([0x00, 0xFF, 0xFE, 0x0A]))
            return
        daten = {
            "methode": self.command,
            "pfad": self.path,
            "auth": self.headers.get("Authorization", ""),
            "ctype": self.headers.get("Content-Type", ""),
            "eigen": self.headers.get("X-Eigen", ""),
            "rumpf_hex": rumpf.hex(),
        }
        nutz = json.dumps(daten).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(nutz)

    do_GET = do_POST = do_PUT = do_PATCH = _antworten
    do_DELETE = do_OPTIONS = _antworten


@contextlib.contextmanager
def _echo_server(verzoegerung: float = 0.0):
    _EchoHandler.verzoegerung = verzoegerung
    server = ThreadingHTTPServer(("127.0.0.1", 0), _EchoHandler)
    t = threading.Thread(target=server.serve_forever, daemon=True)
    t.start()
    try:
        yield f"http://127.0.0.1:{server.server_port}"
    finally:
        _EchoHandler.verzoegerung = 0.0
        server.shutdown()
        server.server_close()


def _echo(ausgabe: str) -> dict:
    """Letzte nichtleere Zeile als JSON lesen."""
    zeilen = [z for z in ausgabe.splitlines() if z.strip()]
    return json.loads(zeilen[-1])


# ------------------------------------------------------------- Methoden

@pytest.mark.parametrize("methode", ["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"])
def test_alle_methoden_kommen_an(run_gb, methode):
    with _echo_server() as basis:
        out = run_gb(f'IMPORT "html"\nPRINT HTTP_REQUEST("{methode}", "{basis}/x")')
    assert _echo(out)["methode"] == methode


def test_methode_ist_gross_klein_egal(run_gb):
    with _echo_server() as basis:
        out = run_gb(f'IMPORT "html"\nPRINT HTTP_REQUEST("put", "{basis}/x")')
    assert _echo(out)["methode"] == "PUT"


def test_tippfehler_in_der_methode_wirft_sofort(run_gb):
    # Ohne Pruefung kaeme das als merkwuerdige Server-Antwort zurueck statt
    # als Fehler an der Zeile, in der der Tippfehler steht.
    with _echo_server() as basis:
        with pytest.raises(DHRuntimeError, match="unbekannte Methode 'GTE'"):
            run_gb(f'IMPORT "html"\nPRINT HTTP_REQUEST("GTE", "{basis}/x")')


# ----------------------------------------------------------- Kopfzeilen

def test_eigene_kopfzeile_erreicht_den_server(run_gb):
    """Der Fall, den die Doku bisher empfahl und die API nicht konnte."""
    with _echo_server() as basis:
        out = run_gb('IMPORT "html"\n'
                     'DIM k AS MAP OF STRING\n'
                     'MAPPUT(k, "Authorization", "Bearer geheim123")\n'
                     f'PRINT HTTP_REQUEST("GET", "{basis}/x", "", k)')
    assert _echo(out)["auth"] == "Bearer geheim123"


def test_content_type_wird_nicht_geraten(run_gb):
    # Bewusst KEINE Vorgabe: welchen Typ ein Rumpf hat, weiss nur der Aufrufer.
    with _echo_server() as basis:
        out = run_gb(f'IMPORT "html"\nPRINT HTTP_REQUEST("POST", "{basis}/x", "irgendwas")')
    assert _echo(out)["ctype"] == ""


def test_content_type_selbst_gesetzt(run_gb):
    with _echo_server() as basis:
        out = run_gb('IMPORT "html"\n'
                     'DIM k AS MAP OF STRING\n'
                     'MAPPUT(k, "Content-Type", "application/json")\n'
                     f'PRINT HTTP_REQUEST("POST", "{basis}/x", "{{}}", k)')
    assert _echo(out)["ctype"] == "application/json"


def test_dauerhafte_kopfzeile_gilt_fuer_folgende_aufrufe(run_gb):
    with _echo_server() as basis:
        out = run_gb('IMPORT "html"\n'
                     'HTTP_SET_HEADER("Authorization", "Bearer dauerhaft")\n'
                     f'PRINT HTTP_REQUEST("GET", "{basis}/x")')
    assert _echo(out)["auth"] == "Bearer dauerhaft"


def test_dauerhafte_kopfzeile_gilt_auch_fuer_http_get(run_gb):
    with _echo_server() as basis:
        out = run_gb('IMPORT "html"\n'
                     'HTTP_SET_HEADER("X-Eigen", "ja")\n'
                     f'PRINT HTTP_GET("{basis}/x")')
    assert _echo(out)["eigen"] == "ja"


def test_kopfzeile_des_aufrufs_gewinnt_gegen_die_dauerhafte(run_gb):
    with _echo_server() as basis:
        out = run_gb('IMPORT "html"\n'
                     'DIM k AS MAP OF STRING\n'
                     'HTTP_SET_HEADER("Authorization", "Bearer dauerhaft")\n'
                     'MAPPUT(k, "Authorization", "Bearer nurhier")\n'
                     f'PRINT HTTP_REQUEST("GET", "{basis}/x", "", k)')
    assert _echo(out)["auth"] == "Bearer nurhier"


def test_set_header_zweimal_ersetzt_statt_zu_haeufen(run_gb):
    # Zweimal derselbe Name ginge sonst raus, und welcher gilt, entschiede
    # der Server.
    with _echo_server() as basis:
        out = run_gb('IMPORT "html"\n'
                     'HTTP_SET_HEADER("Authorization", "alt")\n'
                     'HTTP_SET_HEADER("authorization", "neu")\n'
                     f'PRINT HTTP_REQUEST("GET", "{basis}/x")')
    assert _echo(out)["auth"] == "neu"


def test_clear_headers_raeumt_auf(run_gb):
    with _echo_server() as basis:
        out = run_gb('IMPORT "html"\n'
                     'HTTP_SET_HEADER("Authorization", "weg damit")\n'
                     'HTTP_CLEAR_HEADERS()\n'
                     f'PRINT HTTP_REQUEST("GET", "{basis}/x")')
    assert _echo(out)["auth"] == ""


def test_zeilenumbruch_in_der_kopfzeile_wird_abgelehnt(run_gb):
    """Header-Injection: ein CRLF im Wert haengt beliebige weitere Kopfzeilen
    an. Kommt der Wert aus einer Eingabe, ist das eine Luecke."""
    with pytest.raises(DHRuntimeError, match="Zeilenumbruch"):
        run_gb('IMPORT "html"\n'
               'HTTP_SET_HEADER("X-Bad", "a" + CHR$(13) + CHR$(10) + "X-Rein: ja")')


def test_kopfzeilen_name_mit_doppelpunkt_wird_abgelehnt(run_gb):
    with pytest.raises(DHRuntimeError, match="unerlaubte Zeichen"):
        run_gb('IMPORT "html"\nHTTP_SET_HEADER("X:Y", "z")')


def test_kopfzeilen_map_mit_falschem_werttyp_wirft(run_gb):
    with _echo_server() as basis:
        with pytest.raises(DHRuntimeError, match="erwartet STRING"):
            run_gb('IMPORT "html"\n'
                   'DIM k AS MAP OF INTEGER\n'
                   'MAPPUT(k, "X-Zahl", 5)\n'
                   f'PRINT HTTP_REQUEST("GET", "{basis}/x", "", k)')


# ----------------------------------------------------------------- Rumpf

def test_rumpf_als_string(run_gb):
    with _echo_server() as basis:
        out = run_gb(f'IMPORT "html"\nPRINT HTTP_REQUEST("POST", "{basis}/x", "hallo")')
    assert _echo(out)["rumpf_hex"] == b"hallo".hex()


def test_rumpf_als_buffer(run_gb):
    # Genau der Grund fuer WP B: ein Bild oder eine Zip-Datei hochladen.
    with _echo_server() as basis:
        out = run_gb('IMPORT "html"\n'
                     'DIM b AS BUFFER\nb = BUFFER_FROM_HEX("00ff10")\n'
                     f'PRINT HTTP_REQUEST("POST", "{basis}/x", b)')
    assert _echo(out)["rumpf_hex"] == "00ff10"


def test_ohne_rumpf(run_gb):
    with _echo_server() as basis:
        out = run_gb(f'IMPORT "html"\nPRINT HTTP_REQUEST("GET", "{basis}/x")')
    assert _echo(out)["rumpf_hex"] == ""


def test_falscher_rumpf_typ_wirft(run_gb):
    with _echo_server() as basis:
        with pytest.raises(DHRuntimeError, match="Rumpf erwartet STRING oder BUFFER"):
            run_gb(f'IMPORT "html"\nPRINT HTTP_REQUEST("POST", "{basis}/x", 42)')


# ------------------------------------------------------ Antwort als Bytes

def test_http_bytes_liefert_die_rohen_bytes(run_gb):
    """Der Rueckgabewert ist verlustbehaftet nach UTF-8 gewandelt -- bei einem
    Bild bliebe davon nichts. HTTP_BYTES liefert das Original."""
    with _echo_server() as basis:
        out = run_gb('IMPORT "html"\n'
                     f'DIM t AS STRING\nt = HTTP_REQUEST("GET", "{basis}/rohe-bytes")\n'
                     'PRINT BUFFER_TO_HEX$(HTTP_BYTES())')
    assert out.strip() == "00fffe0a"


def test_http_bytes_auch_nach_http_get(run_gb):
    with _echo_server() as basis:
        out = run_gb('IMPORT "html"\n'
                     f'DIM t AS STRING\nt = HTTP_GET("{basis}/rohe-bytes")\n'
                     'PRINT BUFFER_LEN(HTTP_BYTES())')
    assert out.strip() == "4"


def test_http_bytes_ist_nach_einem_fehler_leer(run_gb):
    """Sonst gehoerten die Bytes der VORIGEN Antwort und kaemen nach einem
    Fehlschlag als neue durch."""
    with _echo_server() as basis:
        out = run_gb('IMPORT "html"\n'
                     f'DIM t AS STRING\nt = HTTP_REQUEST("GET", "{basis}/rohe-bytes")\n'
                     'PRINT BUFFER_LEN(HTTP_BYTES())\n'
                     'TRY\n'
                     '    t = HTTP_REQUEST("GET", "http://127.0.0.1:1/weg")\n'
                     'CATCH e\n'
                     '    PRINT "fehler"\n'
                     'END TRY\n'
                     'PRINT BUFFER_LEN(HTTP_BYTES())')
    assert [z for z in out.splitlines() if z.strip()] == ["4", "fehler", "0"]


# -------------------------------------------------------------- Zeitgrenze

def test_timeout_greift(run_gb):
    with _echo_server(verzoegerung=2.0) as basis:
        with pytest.raises(DHRuntimeError):
            run_gb('IMPORT "html"\n'
                   'HTTP_TIMEOUT(1)\n'
                   f'PRINT HTTP_REQUEST("GET", "{basis}/langsam")')


@pytest.mark.parametrize("wert", ["0", "601"])
def test_unsinnige_zeitgrenze_wirft(run_gb, wert):
    with pytest.raises(DHRuntimeError, match="ausserhalb 1..600"):
        run_gb(f'IMPORT "html"\nHTTP_TIMEOUT({wert})')


# ------------------------------------------------------ Hintergrund-Anfrage

def test_hintergrund_kann_jetzt_auch_post(run_gb):
    """Vorher konnte der Hintergrund-Pfad NUR GET -- ausgerechnet ein POST
    gegen eine langsame API liess sich nicht auslagern."""
    with _echo_server() as basis:
        out = run_gb('IMPORT "html"\n'
                     'DIM nr AS INTEGER\nDIM k AS MAP OF STRING\n'
                     'MAPPUT(k, "Authorization", "Bearer hintergrund")\n'
                     f'nr = HTTP_REQUEST_START("POST", "{basis}/x", "nutzlast", k)\n'
                     'WHILE NOT HTTP_READY(nr)\n'
                     '    SLEEP(10)\n'
                     'WEND\n'
                     'PRINT HTTP_RESULT(nr)')
    d = _echo(out)
    assert d["methode"] == "POST"
    assert d["auth"] == "Bearer hintergrund"
    assert d["rumpf_hex"] == b"nutzlast".hex()


def test_hintergrund_setzt_status_und_bytes(run_gb):
    with _echo_server() as basis:
        out = run_gb('IMPORT "html"\n'
                     'DIM nr AS INTEGER\nDIM t AS STRING\n'
                     f'nr = HTTP_REQUEST_START("GET", "{basis}/rohe-bytes")\n'
                     'WHILE NOT HTTP_READY(nr)\n'
                     '    SLEEP(10)\n'
                     'WEND\n'
                     't = HTTP_RESULT(nr)\n'
                     'PRINT HTTP_STATUS()\n'
                     'PRINT BUFFER_TO_HEX$(HTTP_BYTES())')
    assert [z for z in out.splitlines() if z.strip()] == ["200", "00fffe0a"]


def test_http_get_start_bleibt_die_kurzform(run_gb):
    with _echo_server() as basis:
        out = run_gb('IMPORT "html"\n'
                     'DIM nr AS INTEGER\n'
                     f'nr = HTTP_GET_START("{basis}/x")\n'
                     'WHILE NOT HTTP_READY(nr)\n'
                     '    SLEEP(10)\n'
                     'WEND\n'
                     'PRINT HTTP_RESULT(nr)')
    assert _echo(out)["methode"] == "GET"


# --------------------------------------------------------- Rueckwaertshalt

def test_http_post_behaelt_seine_formular_vorgabe(run_gb):
    """Bestehende Programme haengen daran -- HTTP_POST setzt weiterhin
    form-urlencoded, auch wenn HTTP_REQUEST nichts mehr raet."""
    with _echo_server() as basis:
        out = run_gb(f'IMPORT "html"\nPRINT HTTP_POST("{basis}/x", "a=1")')
    d = _echo(out)
    assert d["ctype"].startswith("application/x-www-form-urlencoded")
    assert d["rumpf_hex"] == b"a=1".hex()
