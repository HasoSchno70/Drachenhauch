"""Modul `httpd` -- ein kleiner Webserver (Punkt 7 des Allzweck-Audits).

Die erste Allzweck-Roadmap hatte ihn gestrichen ("wer wirklich einen Dienst
braucht, stellt einen fertigen Server davor"). Vor dem Bastler-Leitbild sieht
das anders aus: mit `mqtt`, `firmata`, `serial` und `net` an Bord fehlte fuer
"meine Heizungssteuerung hat eine kleine Weboberflaeche" genau dieser eine
Baustein.

**Der Port kommt ueber stderr.** `HTTPD_START(0)` laesst das Betriebssystem
einen freien Port waehlen; das Programm meldet ihn mit `EPRINT`, weil `PRINT`
gepuffert ist und erst am Programmende erscheint -- ein Test, der darauf
wartet, wartet ewig. So braucht kein Test eine feste Portnummer, und alle
duerfen parallel laufen.
"""
import os
import subprocess
import urllib.error
import urllib.request
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


def _dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    for v in ("release", "debug"):
        p = _ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
        if p.exists():
            return p
    return None


_DHRT = _dhrt()
pytestmark = pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")

KOPF = ('IMPORT "httpd"\n'
        "DIM s AS HTTPD\n"
        "DIM n AS INTEGER\n"
        "s = HTTPD_START(0)\n"
        'EPRINT("PORT=" + STR$(HTTPD_PORT(s)))\n')


class Server:
    def __init__(self, proc, port):
        self.proc, self.port = proc, port

    def hole(self, pfad="/", daten=None, methode=None, timeout=10):
        r = urllib.request.Request(f"http://127.0.0.1:{self.port}{pfad}",
                                   data=daten, method=methode)
        with urllib.request.urlopen(r, timeout=timeout) as a:
            return a.status, a.read().decode("utf-8", "replace"), a.headers

    def ende(self, timeout=15):
        aus, err = self.proc.communicate(timeout=timeout)
        return aus or "", err or ""


@pytest.fixture
def starte(tmp_path):
    laufend = []

    def _start(rumpf: str):
        f = tmp_path / "srv.dh"
        f.write_text(KOPF + rumpf, encoding="utf-8")
        p = subprocess.Popen([str(_DHRT), "run", str(f)], stdout=subprocess.PIPE,
                             stderr=subprocess.PIPE, text=True, encoding="utf-8")
        laufend.append(p)
        zeile = p.stderr.readline()
        assert zeile.startswith("PORT="), f"kein Port gemeldet: {zeile!r}"
        return Server(p, int(zeile.strip().split("=")[1]))

    yield _start
    for p in laufend:
        if p.poll() is None:
            p.kill()


EINMAL = ("WHILE NOT HTTPD_ACCEPT(s)\n"
          "    SLEEP(5)\n"
          "WEND\n")


# ------------------------------------------------------------- Grundlagen
def test_antwortet_ueberhaupt(starte):
    s = starte(EINMAL + 'HTTPD_SEND(s, 200, "text/plain", "hallo")\nHTTPD_STOP(s)\n')
    code, text, kopf = s.hole("/")
    assert code == 200
    assert text == "hallo"
    assert kopf["Content-Type"] == "text/plain"


def test_methode_und_pfad(starte):
    s = starte(EINMAL + 'HTTPD_SEND(s, 200, "text/plain", HTTPD_METHOD$(s) + " " + HTTPD_PATH$(s))\n'
               "HTTPD_STOP(s)\n")
    _, text, _ = s.hole("/mess/raum")
    assert text == "GET /mess/raum"


def test_abfrage_wird_aufgeloest(starte):
    """`%20` und `+` gehoeren zurueckuebersetzt -- sonst steht die Prozent-
    Schreibweise im Programm."""
    s = starte(EINMAL + 'HTTPD_SEND(s, 200, "text/plain", "[" + HTTPD_QUERY$(s, "raum") + "]")\n'
               "HTTPD_STOP(s)\n")
    _, text, _ = s.hole("/x?raum=Wohn%20zimmer&grad=21")
    assert text == "[Wohn zimmer]"


def test_fehlende_abfrage_ist_leer(starte):
    """Was in einer Adresse steht, ist Benutzereingabe -- wie bei ARG$ ist
    ein Leerstring die richtige Antwort, kein Fehler."""
    s = starte(EINMAL + 'HTTPD_SEND(s, 200, "text/plain", "[" + HTTPD_QUERY$(s, "gibtsnicht") + "]")\n'
               "HTTPD_STOP(s)\n")
    _, text, _ = s.hole("/x")
    assert text == "[]"


def test_rumpf_und_kopfzeile(starte):
    s = starte(EINMAL + 'HTTPD_SEND(s, 200, "text/plain", HTTPD_BODY$(s) + "|" + HTTPD_HEADER$(s, "X-Test"))\n'
               "HTTPD_STOP(s)\n")
    r = urllib.request.Request(f"http://127.0.0.1:{s.port}/", data=b"a=1&b=2",
                               headers={"X-Test": "wert"}, method="POST")
    with urllib.request.urlopen(r, timeout=10) as a:
        assert a.read().decode() == "a=1&b=2|wert"


def test_kopfzeilen_ohne_ruecksicht_auf_gross_klein(starte):
    """HTTP schreibt sie mal so, mal so."""
    s = starte(EINMAL + 'HTTPD_SEND(s, 200, "text/plain", HTTPD_HEADER$(s, "USER-AGENT"))\n'
               "HTTPD_STOP(s)\n")
    r = urllib.request.Request(f"http://127.0.0.1:{s.port}/",
                               headers={"User-Agent": "Drachenhauch-Test"})
    with urllib.request.urlopen(r, timeout=10) as a:
        assert a.read().decode() == "Drachenhauch-Test"


def test_statuscode_kommt_durch(starte):
    s = starte(EINMAL + 'HTTPD_SEND(s, 404, "text/plain", "weg")\nHTTPD_STOP(s)\n')
    with pytest.raises(urllib.error.HTTPError) as e:
        s.hole("/")
    assert e.value.code == 404


def test_mehrere_anfragen_nacheinander(starte):
    s = starte("WHILE n < 3\n"
               "    IF HTTPD_ACCEPT(s) THEN\n"
               '        HTTPD_SEND(s, 200, "text/plain", "nr" + STR$(n))\n'
               "        n = n + 1\n"
               "    END IF\n"
               "    SLEEP(5)\n"
               "WEND\n"
               "HTTPD_STOP(s)\n")
    assert [s.hole(f"/{i}")[1] for i in range(3)] == ["nr0", "nr1", "nr2"]


# --------------------------------------------------------- Dateien
@pytest.fixture
def webordner(tmp_path):
    w = tmp_path / "web"
    (w / "unter").mkdir(parents=True)
    (w / "index.html").write_text("<h1>Start</h1>", encoding="utf-8")
    (w / "unter" / "a.txt").write_text("tief", encoding="utf-8")
    (tmp_path / "geheim.txt").write_text("GEHEIM", encoding="utf-8")
    return w


def test_send_dir_liefert_die_startseite(starte, webordner):
    s = starte(EINMAL + 'PRINT HTTPD_SEND_DIR(s, "web")\nHTTPD_STOP(s)\n')
    code, text, kopf = s.hole("/")
    assert code == 200
    assert "<h1>Start</h1>" in text
    assert kopf["Content-Type"].startswith("text/html")


def test_send_dir_liefert_aus_unterordnern(starte, webordner):
    s = starte(EINMAL + 'PRINT HTTPD_SEND_DIR(s, "web")\nHTTPD_STOP(s)\n')
    assert s.hole("/unter/a.txt")[1] == "tief"


def test_send_dir_laesst_niemanden_hinaus(starte, webordner):
    """Der eigentliche Grund, warum es SEND_DIR gibt: die naheliegende
    Handarbeit (`HTTPD_SEND_FILE(s, 200, "web" + HTTPD_PATH$(s))`) laesst
    sich mit `/../geheim.txt` aus dem Ordner herausfuehren. Dieselbe Lehre
    wie bei der Zip-Slip-Pruefung in ZIP_EXTRACT."""
    s = starte(EINMAL + 'PRINT HTTPD_SEND_DIR(s, "web")\nHTTPD_STOP(s)\n')
    # urllib normalisiert `/../` selbst weg -- deshalb roh ueber einen Socket.
    import socket
    with socket.create_connection(("127.0.0.1", s.port), timeout=10) as k:
        k.sendall(b"GET /../geheim.txt HTTP/1.1\r\nHost: x\r\n\r\n")
        antwort = b""
        while True:
            teil = k.recv(4096)
            if not teil:
                break
            antwort += teil
    assert b"403" in antwort.split(b"\r\n")[0]
    assert b"GEHEIM" not in antwort


def test_send_dir_meldet_fehlende_dateien_als_404(starte, webordner):
    """Eine fehlende Datei ist der Alltag eines Servers und darf das Programm
    nicht anhalten -- der Rueckgabewert sagt es trotzdem."""
    s = starte(EINMAL + 'PRINT HTTPD_SEND_DIR(s, "web")\nHTTPD_STOP(s)\n')
    with pytest.raises(urllib.error.HTTPError) as e:
        s.hole("/gibtsnicht.html")
    assert e.value.code == 404
    aus, _ = s.ende()
    assert "FALSE" in aus            # der Rueckgabewert von SEND_DIR


def test_send_file_liefert_eine_bestimmte_datei(starte, webordner):
    s = starte(EINMAL + 'HTTPD_SEND_FILE(s, 200, "web/unter/a.txt")\nHTTPD_STOP(s)\n')
    assert s.hole("/egal")[1] == "tief"


# ------------------------------------------------------------ Fehlerfaelle
def test_abfragen_ohne_anfrage_erklaeren_sich(starte):
    """Wer HTTPD_PATH$ vor HTTPD_ACCEPT ruft, soll das erfahren."""
    s = starte('PRINT HTTPD_PATH$(s)\n')
    _, err = s.ende()
    assert "erst HTTPD_ACCEPT" in err


def test_senden_ohne_anfrage_erklaert_sich(starte):
    s = starte('HTTPD_SEND(s, 200, "text/plain", "x")\n')
    _, err = s.ende()
    assert "keine Anfrage" in err


def test_belegter_port_meldet_sich(starte, tmp_path):
    """Zwei Server auf demselben Port -- der zweite muss es sagen."""
    s = starte(EINMAL + 'HTTPD_SEND(s, 200, "text/plain", "x")\nHTTPD_STOP(s)\n')
    zweit = tmp_path / "zweit.dh"
    zweit.write_text('IMPORT "httpd"\nDIM t AS HTTPD\n'
                     f"t = HTTPD_START({s.port})\n", encoding="utf-8")
    r = subprocess.run([str(_DHRT), "run", str(zweit)], capture_output=True,
                       text=True, encoding="utf-8", timeout=30)
    assert r.returncode != 0
    assert "nicht belegbar" in (r.stderr or "")
