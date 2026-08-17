"""WP D -- Pruefsummen und Identitaet: SHA256$/SHA1$/MD5$, HMAC_SHA256$,
SECURE_EQUALS, UUID4$, RANDOM_BYTES.

Die Hash-Tests pruefen gegen die Vektoren aus den Normen (FIPS 180-4,
RFC 1321, RFC 4231) bzw. gegen Pythons `hashlib`/`hmac`. Das ist der Punkt:
eine Pruefsumme, die nur mit sich selbst uebereinstimmt, ist wertlos -- sie
muss dasselbe liefern wie jedes andere Werkzeug der Welt.
"""
import hashlib
import hmac as py_hmac
import re

import pytest

from drachenhauch.errors import DHRuntimeError


# ------------------------------------------------------ Vektoren der Normen

@pytest.mark.parametrize("text", ["abc", "", "The quick brown fox jumps over the lazy dog"])
def test_sha256_gegen_hashlib(run_gb, text):
    out = run_gb(f'PRINT SHA256$("{text}")').strip()
    assert out == hashlib.sha256(text.encode()).hexdigest()


@pytest.mark.parametrize("text", ["abc", ""])
def test_sha1_gegen_hashlib(run_gb, text):
    assert run_gb(f'PRINT SHA1$("{text}")').strip() == hashlib.sha1(text.encode()).hexdigest()


@pytest.mark.parametrize("text", ["abc", ""])
def test_md5_gegen_hashlib(run_gb, text):
    assert run_gb(f'PRINT MD5$("{text}")').strip() == hashlib.md5(text.encode()).hexdigest()


def test_sha256_bekannter_vektor(run_gb):
    # FIPS 180-4, der meistzitierte Vektor ueberhaupt -- fest verdrahtet,
    # damit der Test auch dann noch etwas aussagt, wenn hashlib mal luegt.
    assert run_gb('PRINT SHA256$("abc")').strip() == \
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"


def test_hmac_sha256_gegen_python(run_gb):
    out = run_gb('PRINT HMAC_SHA256$("key", "The quick brown fox jumps over the lazy dog")').strip()
    assert out == py_hmac.new(b"key",
                              b"The quick brown fox jumps over the lazy dog",
                              hashlib.sha256).hexdigest()


def test_hmac_mit_leerem_schluessel(run_gb):
    # HMAC erlaubt jede Schluessellaenge, auch 0 -- darf also nicht werfen.
    out = run_gb('PRINT HMAC_SHA256$("", "daten")').strip()
    assert out == py_hmac.new(b"", b"daten", hashlib.sha256).hexdigest()


def test_hmac_mit_langem_schluessel(run_gb):
    # Laenger als die Blockgroesse (64 Byte) -- wird intern gehasht.
    key = "k" * 100
    out = run_gb(f'PRINT HMAC_SHA256$("{key}", "daten")').strip()
    assert out == py_hmac.new(key.encode(), b"daten", hashlib.sha256).hexdigest()


# ------------------------------------------------------- STRING und BUFFER

def test_hash_ueber_buffer_gleicht_dem_ueber_string(run_gb):
    out = run_gb('PRINT SHA256$("abc")\nPRINT SHA256$(BUFFER_FROM_STRING("abc"))')
    a, b = out.split()
    assert a == b == hashlib.sha256(b"abc").hexdigest()


def test_hash_ueber_bytes_die_kein_text_sind(run_gb):
    """Der Grund, warum BUFFER zugelassen ist: eine Signatur bildet man ueber
    die Bytes, die wirklich uebertragen werden."""
    out = run_gb('PRINT SHA256$(BUFFER_FROM_HEX("00fffe"))').strip()
    assert out == hashlib.sha256(bytes([0x00, 0xFF, 0xFE])).hexdigest()


def test_hmac_nimmt_buffer_als_daten(run_gb):
    out = run_gb('PRINT HMAC_SHA256$("k", BUFFER_FROM_HEX("00ff"))').strip()
    assert out == py_hmac.new(b"k", bytes([0x00, 0xFF]), hashlib.sha256).hexdigest()


def test_falscher_typ_wirft(run_gb):
    with pytest.raises(DHRuntimeError, match="erwartet STRING oder BUFFER"):
        run_gb('PRINT SHA256$(42)')


def test_umlaute_werden_als_utf8_gehasht(run_gb):
    text = "Grüße"
    assert run_gb(f'PRINT SHA256$("{text}")').strip() == \
        hashlib.sha256(text.encode("utf-8")).hexdigest()


# ------------------------------------------------------------ Datei-Hashes

def test_datei_hash_gleicht_dem_ueber_den_inhalt(run_gb, tmp_path):
    (tmp_path / "d.bin").write_bytes(bytes(range(256)) * 10)
    out = run_gb('PRINT SHA256_FILE$("d.bin")\n'
                 'PRINT SHA256$(READALL_BYTES("d.bin"))', base=tmp_path)
    a, b = out.split()
    assert a == b == hashlib.sha256(bytes(range(256)) * 10).hexdigest()


def test_datei_hash_ueber_mehrere_bloecke(run_gb, tmp_path):
    # Groesser als der 64-KiB-Lesepuffer -- prueft, dass blockweise gehasht
    # wird und nicht nur der erste Block.
    daten = bytes(range(256)) * 1000        # 256 000 Byte
    (tmp_path / "gross.bin").write_bytes(daten)
    out = run_gb('PRINT SHA256_FILE$("gross.bin")', base=tmp_path).strip()
    assert out == hashlib.sha256(daten).hexdigest()


@pytest.mark.parametrize("fn,py", [("MD5_FILE$", hashlib.md5), ("SHA1_FILE$", hashlib.sha1)])
def test_weitere_datei_hashes(run_gb, tmp_path, fn, py):
    (tmp_path / "x.bin").write_bytes(b"inhalt")
    assert run_gb(f'PRINT {fn}("x.bin")', base=tmp_path).strip() == py(b"inhalt").hexdigest()


def test_datei_hash_auf_fehlende_datei_wirft(run_gb):
    with pytest.raises(DHRuntimeError, match="SHA256_FILE"):
        run_gb('PRINT SHA256_FILE$("gibt_es_nicht_xyz.bin")')


# --------------------------------------------------------- SECURE_EQUALS

def test_secure_equals_vergleicht_richtig(run_gb):
    out = run_gb('PRINT SECURE_EQUALS(SHA256$("a"), SHA256$("a"))\n'
                 'PRINT SECURE_EQUALS(SHA256$("a"), SHA256$("b"))\n'
                 'PRINT SECURE_EQUALS("kurz", "laenger")')
    assert out.split() == ["TRUE", "FALSE", "FALSE"]


def test_secure_equals_auf_buffern(run_gb):
    out = run_gb('PRINT SECURE_EQUALS(BUFFER_FROM_HEX("00ff"), BUFFER_FROM_HEX("00ff"))\n'
                 'PRINT SECURE_EQUALS(BUFFER_FROM_HEX("00ff"), BUFFER_FROM_HEX("00fe"))')
    assert out.split() == ["TRUE", "FALSE"]


def test_secure_equals_leer(run_gb):
    assert run_gb('PRINT SECURE_EQUALS("", "")').strip() == "TRUE"


# ------------------------------------------------------------------- UUID

def test_uuid4_hat_die_richtige_form(run_gb):
    u = run_gb('PRINT UUID4$()').strip()
    # 8-4-4-4-12, Version 4, Variante 8/9/a/b
    assert re.fullmatch(r"[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}", u), u


def test_uuid4_wiederholt_sich_nicht(run_gb):
    out = run_gb('DIM i AS INTEGER\nFOR i = 1 TO 50\n    PRINT UUID4$()\nNEXT')
    ids = out.split()
    assert len(ids) == 50
    assert len(set(ids)) == 50


# ----------------------------------------------------------- RANDOM_BYTES

def test_random_bytes_hat_die_gewuenschte_laenge(run_gb):
    assert run_gb('PRINT BUFFER_LEN(RANDOM_BYTES(32))').strip() == "32"


def test_random_bytes_null_ist_erlaubt(run_gb):
    assert run_gb('PRINT BUFFER_LEN(RANDOM_BYTES(0))').strip() == "0"


def test_random_bytes_wiederholt_sich_nicht(run_gb):
    out = run_gb('DIM i AS INTEGER\nFOR i = 1 TO 20\n'
                 '    PRINT BUFFER_TO_HEX$(RANDOM_BYTES(16))\nNEXT')
    werte = out.split()
    assert len(werte) == 20 and len(set(werte)) == 20


def test_random_bytes_folgt_nicht_dem_saatbaren_generator(run_gb):
    """Der entscheidende Unterschied zu RND: RANDOMIZE legt die Folge von RND
    fest -- fuer ein Wuerfelspiel richtig, fuer ein Passwort ein Fehler.
    RANDOM_BYTES kommt aus der Quelle des Betriebssystems und darf sich davon
    NICHT beeindrucken lassen."""
    erst = run_gb('RANDOMIZE(1)\nPRINT RND(1000)\nPRINT BUFFER_TO_HEX$(RANDOM_BYTES(16))')
    zweit = run_gb('RANDOMIZE(1)\nPRINT RND(1000)\nPRINT BUFFER_TO_HEX$(RANDOM_BYTES(16))')
    rnd1, zufall1 = erst.split()
    rnd2, zufall2 = zweit.split()
    assert rnd1 == rnd2, "RND muss bei gleicher Saat reproduzierbar bleiben"
    assert zufall1 != zufall2, "RANDOM_BYTES darf NICHT von RANDOMIZE abhaengen"


def test_random_bytes_negativ_wirft(run_gb):
    with pytest.raises(DHRuntimeError, match="ist negativ"):
        run_gb('PRINT BUFFER_LEN(RANDOM_BYTES(-1))')


def test_random_bytes_absurd_gross_wirft(run_gb):
    with pytest.raises(DHRuntimeError, match="Obergrenze"):
        run_gb('PRINT BUFFER_LEN(RANDOM_BYTES(999999999999))')


# ------------------------------------------------- Zusammenspiel mit WP C

def test_signatur_pruefen_wie_bei_einem_webhook(run_gb):
    """Der Fall, fuer den WP D da ist: ein Dienst schickt Daten plus eine
    HMAC-Signatur, das Programm rechnet nach."""
    geheim = "streng-geheim"
    nutzlast = '{"ereignis":"zahlung"}'
    echte = py_hmac.new(geheim.encode(), nutzlast.encode(), hashlib.sha256).hexdigest()
    quelle = (f'DIM erwartet AS STRING\n'
              f'erwartet = HMAC_SHA256$("{geheim}", "{nutzlast.replace(chr(34), chr(34) * 2)}")\n'
              f'PRINT SECURE_EQUALS(erwartet, "{echte}")\n'
              f'PRINT SECURE_EQUALS(erwartet, "{"0" * 64}")')
    assert run_gb(quelle).split() == ["TRUE", "FALSE"]
