"""Modul `smtp` -- eine E-Mail bauen und verschicken (Punkt 7 des Audits).

Zwei Arten von Test, und beide braucht es:

* **Die Nachricht** wird mit `SMTP_MESSAGE$` abgeholt und von Pythons
  `email`-Modul gelesen -- einem FREMDEN Leser. "Die Nachricht ist in
  Ordnung" hiesse sonst nur "mein Erzeuger ist mit sich einig".
* **Das Protokoll** laeuft gegen einen winzigen SMTP-Server, der hier im
  Test steht (`_MiniServer`) und die Befehle mitschreibt. Damit ist
  nachweisbar, was im UMSCHLAG steht -- und dass die Blindkopie dort
  auftaucht, obwohl sie in den Kopfzeilen fehlt.

**Nicht abgedeckt: TLS.** Ein echter Handschlag braucht ein Zertifikat, dem
der Client traut; ein selbst ausgestelltes wuerde zu Recht abgelehnt. Der
verschluesselte Weg unterscheidet sich nur in der Huelle des Datenstroms --
alles darueber (EHLO/AUTH/MAIL/RCPT/DATA) ist derselbe Code. Steht so auch
in docs/module-smtp.md.
"""
import email
import socket
import threading

import pytest

from drachenhauch.errors import DHRuntimeError


class _MiniServer:
    """Ein SMTP-Server, gerade gross genug, um eine Nachricht anzunehmen."""

    def __init__(self, faehigkeiten=("AUTH PLAIN LOGIN",), auth_ok=True, rcpt_ok=True):
        self.faehigkeiten = list(faehigkeiten)
        self.auth_ok = auth_ok
        self.rcpt_ok = rcpt_ok
        self.befehle: list[str] = []
        self.daten = ""
        self.sock = socket.socket()
        self.sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self.sock.bind(("127.0.0.1", 0))
        self.sock.listen(1)
        self.port = self.sock.getsockname()[1]
        self.fehler: BaseException | None = None
        self.thread = threading.Thread(target=self._laufen, daemon=True)

    def __enter__(self):
        self.thread.start()
        return self

    def __exit__(self, *_):
        self.thread.join(timeout=10)
        self.sock.close()
        if self.fehler is not None:
            raise self.fehler

    def _laufen(self):
        try:
            self.sock.settimeout(15)
            conn, _ = self.sock.accept()
            conn.settimeout(15)
            f = conn.makefile("rwb")
            f.write(b"220 mini.beispiel.de ESMTP\r\n")
            f.flush()
            erwarte_login = 0
            while True:
                zeile = f.readline()
                if not zeile:
                    break
                text = zeile.decode("utf-8", "replace").rstrip("\r\n")
                self.befehle.append(text)
                oben = text.upper()
                if erwarte_login:
                    erwarte_login -= 1
                    f.write(b"334 \r\n" if erwarte_login else
                            (b"235 ok\r\n" if self.auth_ok else b"535 falsch\r\n"))
                elif oben.startswith("EHLO") or oben.startswith("HELO"):
                    for k in self.faehigkeiten:
                        f.write(f"250-{k}\r\n".encode())
                    f.write(b"250 HELP\r\n")
                elif oben.startswith("AUTH LOGIN"):
                    erwarte_login = 2
                    f.write(b"334 \r\n")
                elif oben.startswith("AUTH PLAIN"):
                    f.write(b"235 ok\r\n" if self.auth_ok else b"535 falsch\r\n")
                elif oben.startswith("MAIL FROM"):
                    f.write(b"250 ok\r\n")
                elif oben.startswith("RCPT TO"):
                    f.write(b"250 ok\r\n" if self.rcpt_ok else b"550 kenne ich nicht\r\n")
                elif oben == "DATA":
                    f.write(b"354 los\r\n")
                    f.flush()
                    roh = []
                    while True:
                        d = f.readline()
                        if not d or d in (b".\r\n", b".\n"):
                            break
                        # Punkt-Verdopplung ruecknehmen -- so wie es ein
                        # echter Server auch tut.
                        if d.startswith(b".."):
                            d = d[1:]
                        roh.append(d)
                    self.daten = b"".join(roh).decode("utf-8", "replace")
                    f.write(b"250 angenommen\r\n")
                elif oben == "QUIT":
                    f.write(b"221 tschuess\r\n")
                    f.flush()
                    break
                else:
                    f.write(b"502 kenne ich nicht\r\n")
                f.flush()
            conn.close()
        except BaseException as e:  # noqa: BLE001 -- im Test weitergereicht
            self.fehler = e


def _quelle(port, extra="", sicherheit='"keine"'):
    return (
        'IMPORT "smtp"\n'
        "DIM m AS SMTP\n"
        "m = SMTP_NEW()\n"
        f'SMTP_SERVER(m, "127.0.0.1", {port}, {sicherheit})\n'
        'SMTP_FROM(m, "ich@beispiel.de", "Abteilung Zahlen")\n'
        'SMTP_TO(m, "du@beispiel.de")\n'
        'SMTP_SUBJECT(m, "Auswertung August")\n'
        'SMTP_TEXT(m, "Anbei die Zahlen.")\n'
        + extra +
        "SMTP_SEND(m)\n"
        'PRINT "raus"\n'
    )


# ------------------------------------------------------------ die Nachricht
def _nachricht(run_gb, tmp_path, extra=""):
    src = ('IMPORT "smtp"\nDIM m AS SMTP\nm = SMTP_NEW()\n'
           'SMTP_FROM(m, "ich@beispiel.de")\nSMTP_TO(m, "du@beispiel.de")\n'
           'SMTP_SUBJECT(m, "Test")\nSMTP_TEXT(m, "Hallo")\n'
           + extra + "PRINT SMTP_MESSAGE$(m)\n")
    return email.message_from_string(run_gb(src, base=tmp_path))


def test_eine_einfache_nachricht(run_gb, tmp_path):
    m = _nachricht(run_gb, tmp_path)
    assert m["From"] == "ich@beispiel.de"
    assert m["To"] == "du@beispiel.de"
    assert m["Subject"] == "Test"
    assert m.get_content_type() == "text/plain"
    assert m.get_payload(decode=True).decode() == "Hallo"


def test_umlaute_im_betreff_kommen_an(run_gb, tmp_path):
    m = _nachricht(run_gb, tmp_path,
                   extra='SMTP_SUBJECT(m, "Gr' + chr(252) + 'sse zum Monatsabschlu' + chr(223) + '")\n')
    # Der Leser dekodiert das RFC-2047-Wort selbst -- genau darum geht es.
    kopf = str(email.header.make_header(email.header.decode_header(m["Subject"])))
    assert kopf == "Grüsse zum Monatsabschluß"


def test_ein_langer_umlaut_betreff_wird_richtig_gefaltet(run_gb, tmp_path):
    lang = "ä" * 60
    m = _nachricht(run_gb, tmp_path, extra=f'SMTP_SUBJECT(m, "{lang}")\n')
    kopf = str(email.header.make_header(email.header.decode_header(m["Subject"])))
    assert kopf == lang


def test_anzeigename_wird_uebernommen(run_gb, tmp_path):
    m = _nachricht(run_gb, tmp_path,
                   extra='SMTP_FROM(m, "ich@beispiel.de", "Abteilung Zahlen")\n')
    assert m["From"] == "Abteilung Zahlen <ich@beispiel.de>"


def test_text_und_html_ergeben_zwei_teile(run_gb, tmp_path):
    m = _nachricht(run_gb, tmp_path, extra='SMTP_HTML(m, "<b>Hallo</b>")\n')
    assert m.get_content_type() == "multipart/alternative"
    typen = [t.get_content_type() for t in m.walk() if not t.is_multipart()]
    assert typen == ["text/plain", "text/html"]


def test_anhang_kommt_unveraendert_an(run_gb, tmp_path):
    # Bytes schreiben, nicht Text: sonst macht Windows aus \n ein \r\n, und
    # der Test prueft am Ende die Zeilenenden von Python statt den Anhang.
    (tmp_path / "zahlen.csv").write_bytes(b"a;b\n1;2\n")
    m = _nachricht(run_gb, tmp_path, extra='SMTP_ATTACH(m, "zahlen.csv")\n')
    assert m.get_content_type() == "multipart/mixed"
    anh = [t for t in m.walk() if t.get_filename()]
    assert len(anh) == 1
    assert anh[0].get_filename() == "zahlen.csv"
    assert anh[0].get_payload(decode=True) == b"a;b\n1;2\n"


def test_anhang_bekommt_seine_art_aus_der_endung(run_gb, tmp_path):
    (tmp_path / "bericht.pdf").write_bytes(b"%PDF-1.4 nicht echt")
    m = _nachricht(run_gb, tmp_path, extra='SMTP_ATTACH(m, "bericht.pdf")\n')
    anh = [t for t in m.walk() if t.get_filename()][0]
    assert anh.get_content_type() == "application/pdf"


def test_binaerer_anhang_ueberlebt(run_gb, tmp_path):
    roh = bytes(range(256)) * 8
    (tmp_path / "roh.bin").write_bytes(roh)
    m = _nachricht(run_gb, tmp_path, extra='SMTP_ATTACH(m, "roh.bin")\n')
    anh = [t for t in m.walk() if t.get_filename()][0]
    assert anh.get_payload(decode=True) == roh


def test_blindkopie_steht_nicht_in_den_kopfzeilen(run_gb, tmp_path):
    m = _nachricht(run_gb, tmp_path, extra='SMTP_BCC(m, "heimlich@beispiel.de")\n')
    assert m["Bcc"] is None
    assert "heimlich" not in m.as_string()


def test_jede_nachricht_hat_datum_und_kennung(run_gb, tmp_path):
    m = _nachricht(run_gb, tmp_path)
    assert email.utils.parsedate_tz(m["Date"]) is not None
    assert m["Message-ID"].endswith("@beispiel.de>")


# ------------------------------------------------------------- das Protokoll
def test_senden_gegen_einen_echten_server(run_gb, tmp_path):
    with _MiniServer() as s:
        assert run_gb(_quelle(s.port), base=tmp_path).strip() == "raus"
    assert "MAIL FROM:<ich@beispiel.de>" in s.befehle
    assert "RCPT TO:<du@beispiel.de>" in s.befehle
    assert "DATA" in s.befehle and "QUIT" in s.befehle
    m = email.message_from_string(s.daten)
    assert m["Subject"] == "Auswertung August"
    assert m.get_payload(decode=True).decode() == "Anbei die Zahlen."


def test_die_blindkopie_steht_im_umschlag(run_gb, tmp_path):
    with _MiniServer() as s:
        run_gb(_quelle(s.port, extra='SMTP_BCC(m, "heimlich@beispiel.de")\n'), base=tmp_path)
    assert "RCPT TO:<heimlich@beispiel.de>" in s.befehle
    assert "heimlich" not in s.daten, "in der Nachricht selbst darf sie nicht stehen"


def test_anmeldung_mit_plain(run_gb, tmp_path):
    with _MiniServer() as s:
        run_gb(_quelle(s.port, extra='SMTP_LOGIN(m, "hans", "geheim")\n'), base=tmp_path)
    plain = [b for b in s.befehle if b.upper().startswith("AUTH PLAIN")]
    assert len(plain) == 1
    import base64
    roh = base64.b64decode(plain[0].split()[2]).decode()
    assert roh == "\0hans\0geheim"


def test_anmeldung_mit_login_wenn_plain_fehlt(run_gb, tmp_path):
    with _MiniServer(faehigkeiten=("AUTH LOGIN",)) as s:
        run_gb(_quelle(s.port, extra='SMTP_LOGIN(m, "hans", "geheim")\n'), base=tmp_path)
    import base64
    assert "AUTH LOGIN" in s.befehle
    i = s.befehle.index("AUTH LOGIN")
    assert base64.b64decode(s.befehle[i + 1]).decode() == "hans"
    assert base64.b64decode(s.befehle[i + 2]).decode() == "geheim"


def test_falsches_kennwort_wird_gemeldet(run_gb, tmp_path):
    with _MiniServer(auth_ok=False) as s:
        with pytest.raises(DHRuntimeError) as e:
            run_gb(_quelle(s.port, extra='SMTP_LOGIN(m, "hans", "falsch")\n'), base=tmp_path)
    # Der Wortlaut des Servers muss durchkommen -- sonst raet der Anwender.
    assert "535" in str(e.value) and "falsch" in str(e.value)


def test_ohne_anmeldung_wird_nicht_angemeldet(run_gb, tmp_path):
    with _MiniServer() as s:
        run_gb(_quelle(s.port), base=tmp_path)
    assert not [b for b in s.befehle if b.upper().startswith("AUTH")]


def test_server_ohne_starttls_sagt_es_deutlich(run_gb, tmp_path):
    with _MiniServer() as s:
        with pytest.raises(DHRuntimeError) as e:
            run_gb(_quelle(s.port, sicherheit='"starttls"'), base=tmp_path)
    assert "kein STARTTLS" in str(e.value)


def test_ein_abgelehnter_empfaenger_ist_ein_fehler(run_gb, tmp_path):
    """Ein Tippfehler in der Adresse darf nicht als "verschickt" durchgehen."""
    with _MiniServer(rcpt_ok=False) as s:
        with pytest.raises(DHRuntimeError) as e:
            run_gb(_quelle(s.port), base=tmp_path)
    assert "550" in str(e.value) and "du@beispiel.de" in str(e.value)


# ------------------------------------------------------------- Fehlerfaelle
def test_umbruch_im_betreff_wird_abgelehnt(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb('IMPORT "smtp"\nDIM m AS SMTP\nm = SMTP_NEW()\n'
               'SMTP_SUBJECT(m, "Rechnung" + CHR$(13) + CHR$(10) + "Bcc: fremd@x.de")\n')
    assert "Zeilenumbruch" in str(e.value)


def test_adresse_ohne_klammeraffe(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb('IMPORT "smtp"\nDIM m AS SMTP\nm = SMTP_NEW()\n'
               'SMTP_TO(m, "du.beispiel.de")\n')
    assert "sieht nicht wie eine Adresse aus" in str(e.value)


def test_anzeigename_gehoert_nicht_in_die_adresse(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb('IMPORT "smtp"\nDIM m AS SMTP\nm = SMTP_NEW()\n'
               'SMTP_TO(m, "Du <du@beispiel.de>")\n')
    assert "EIGENES Argument" in str(e.value)


def test_leere_nachricht_wird_abgelehnt(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb('IMPORT "smtp"\nDIM m AS SMTP\nm = SMTP_NEW()\n'
               'SMTP_FROM(m, "ich@beispiel.de")\nSMTP_TO(m, "du@beispiel.de")\n'
               "PRINT SMTP_MESSAGE$(m)\n")
    assert "leer" in str(e.value)


def test_ohne_empfaenger_geht_nichts(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb('IMPORT "smtp"\nDIM m AS SMTP\nm = SMTP_NEW()\n'
               'SMTP_FROM(m, "ich@beispiel.de")\nSMTP_TEXT(m, "x")\n'
               "PRINT SMTP_MESSAGE$(m)\n")
    assert "Empfaenger" in str(e.value)


def test_kennwort_im_klartext_ins_netz_wird_verweigert(run_gb):
    """Nicht der Server entscheidet das, sondern wir: ohne Verschluesselung
    ginge das Kennwort mitlesbar ueber die Leitung."""
    with pytest.raises(DHRuntimeError) as e:
        run_gb('IMPORT "smtp"\nDIM m AS SMTP\nm = SMTP_NEW()\n'
               'SMTP_SERVER(m, "mail.beispiel.de", 2525, "keine")\n'
               'SMTP_LOGIN(m, "hans", "geheim")\n'
               'SMTP_FROM(m, "ich@beispiel.de")\nSMTP_TO(m, "du@beispiel.de")\n'
               'SMTP_TEXT(m, "x")\nSMTP_SEND(m)\n')
    assert "Klartext" in str(e.value)


def test_unbekannte_sicherheit(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb('IMPORT "smtp"\nDIM m AS SMTP\nm = SMTP_NEW()\n'
               'SMTP_SERVER(m, "x", 587, "vielleicht")\n')
    assert "starttls" in str(e.value)


def test_geschlossenes_handle(run_gb):
    with pytest.raises(DHRuntimeError) as e:
        run_gb('IMPORT "smtp"\nDIM m AS SMTP\nm = SMTP_NEW()\n'
               'SMTP_CLOSE(m)\nSMTP_SUBJECT(m, "x")\n')
    assert "SMTP-Handle" in str(e.value)
