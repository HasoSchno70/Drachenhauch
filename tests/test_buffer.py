"""WP B -- Bytes und Binaerdateien: der BUFFER-Typ, Umwandlungen, Zahlen
packen, READ_BYTES/WRITE_BYTES/SEEK/TELL.

Golden-Tests gegen die native Runtime.
"""
import pytest

from drachenhauch.errors import DHRuntimeError


# ------------------------------------------------------------- Grundlagen

def test_buffer_ist_ein_kerntyp_ohne_import(run_gb):
    # BUFFER braucht kein IMPORT und ist trotzdem kein Lexer-Keyword --
    # `DIM buffer AS INTEGER` muss weiter gehen (siehe is_value_type).
    assert run_gb('DIM b AS BUFFER\nb = BUFFER_NEW(3)\nPRINT BUFFER_LEN(b)') == "3\n"


def test_buffer_als_variablenname_bleibt_erlaubt(run_gb):
    assert run_gb('DIM buffer AS INTEGER\nbuffer = 7\nPRINT buffer') == "7\n"


def test_neuer_buffer_ist_genullt(run_gb):
    assert run_gb('PRINT BUFFER_TO_HEX$(BUFFER_NEW(4))') == "00000000\n"


def test_default_ist_nil(run_gb):
    assert run_gb('DIM b AS BUFFER\nPRINT IS_NIL(b)') == "TRUE\n"


def test_typeof_und_print(run_gb):
    out = run_gb('DIM b AS BUFFER\nb = BUFFER_NEW(5)\nPRINT TYPEOF(b)\nPRINT b')
    # PRINT zeigt bewusst nur die Laenge -- sonst kippte ein grosser Puffer
    # megabyteweise Bytes in die Konsole.
    assert out == "BUFFER\n<BUFFER 5 Bytes>\n"


def test_get_und_set(run_gb):
    assert run_gb('DIM b AS BUFFER\nb = BUFFER_NEW(2)\n'
                  'BUFFER_SET(b, 0, 222)\nBUFFER_SET(b, 1, 173)\n'
                  'PRINT BUFFER_GET(b, 0)\nPRINT BUFFER_TO_HEX$(b)') == "222\ndead\n"


def test_fill_und_resize(run_gb):
    out = run_gb('DIM b AS BUFFER\nb = BUFFER_NEW(3)\n'
                 'BUFFER_FILL(b, 255)\nPRINT BUFFER_TO_HEX$(b)\n'
                 'BUFFER_RESIZE(b, 5)\nPRINT BUFFER_TO_HEX$(b)\n'   # waechst mit Nullen
                 'BUFFER_RESIZE(b, 2)\nPRINT BUFFER_TO_HEX$(b)')     # schrumpft
    assert out == "ffffff\nffffff0000\nffff\n"


def test_buffer_wird_per_referenz_uebergeben(run_gb):
    # Wie ARRAY: die SUB veraendert das Original des Aufrufers.
    assert run_gb('SUB fuellen(p AS BUFFER)\n'
                  '    BUFFER_SET(p, 0, 99)\n'
                  'END SUB\n'
                  'DIM b AS BUFFER\nb = BUFFER_NEW(2)\n'
                  'fuellen(b)\nPRINT BUFFER_TO_HEX$(b)') == "6300\n"


# ------------------------------------------------------- Grenzen & Fehler

def test_index_ausserhalb_wirft(run_gb):
    with pytest.raises(DHRuntimeError, match="ausserhalb des Puffers"):
        run_gb('PRINT BUFFER_GET(BUFFER_NEW(2), 5)')


def test_negative_position_wirft(run_gb):
    with pytest.raises(DHRuntimeError, match="ausserhalb des Puffers"):
        run_gb('PRINT BUFFER_GET(BUFFER_NEW(2), -1)')


@pytest.mark.parametrize("wert", ["256", "-1"])
def test_byte_ausserhalb_0_255_wirft(run_gb, wert):
    # Wird NICHT stillschweigend beschnitten -- das faellt sonst erst in der
    # Ausgabedatei auf.
    with pytest.raises(DHRuntimeError, match="ausserhalb 0..255"):
        run_gb(f'BUFFER_SET(BUFFER_NEW(2), 0, {wert})')


def test_negative_groesse_wirft(run_gb):
    with pytest.raises(DHRuntimeError, match="ist negativ"):
        run_gb('PRINT BUFFER_NEW(-1)')


def test_absurde_groesse_wirft_statt_den_rechner_lahmzulegen(run_gb):
    with pytest.raises(DHRuntimeError, match="Obergrenze"):
        run_gb('PRINT BUFFER_NEW(999999999999)')


# -------------------------------------------------- Slice / Concat / Suche

def test_slice_klemmt_wie_beim_array(run_gb):
    # docs/sprache.md: "Index-Zugriff ist streng, Slicing klemmt."
    out = run_gb('DIM b AS BUFFER\nb = BUFFER_FROM_HEX("00112233")\n'
                 'PRINT BUFFER_TO_HEX$(BUFFER_SLICE(b, 1, 3))\n'
                 'PRINT BUFFER_TO_HEX$(BUFFER_SLICE(b, 0, 99))\n'
                 'PRINT BUFFER_LEN(BUFFER_SLICE(b, 3, 1))')
    assert out == "1122\n00112233\n0\n"


def test_slice_liefert_eine_kopie_nicht_dieselben_bytes(run_gb):
    out = run_gb('DIM b AS BUFFER\nDIM s AS BUFFER\n'
                 'b = BUFFER_FROM_HEX("aabb")\n'
                 's = BUFFER_SLICE(b, 0, 2)\n'
                 'BUFFER_SET(s, 0, 0)\n'
                 'PRINT BUFFER_TO_HEX$(b)')
    assert out == "aabb\n"


def test_concat(run_gb):
    assert run_gb('PRINT BUFFER_TO_HEX$(BUFFER_CONCAT(BUFFER_FROM_HEX("aa"), '
                  'BUFFER_FROM_HEX("bbcc")))') == "aabbcc\n"


def test_concat_mit_sich_selbst_paniert_nicht(run_gb):
    # Zweimal borrow() auf dieselbe RefCell waere sonst eine Panik.
    assert run_gb('DIM b AS BUFFER\nb = BUFFER_FROM_HEX("aabb")\n'
                  'PRINT BUFFER_TO_HEX$(BUFFER_CONCAT(b, b))') == "aabbaabb\n"


def test_indexof(run_gb):
    out = run_gb('DIM h AS BUFFER\nh = BUFFER_FROM_HEX("0011223311")\n'
                 'PRINT BUFFER_INDEXOF(h, BUFFER_FROM_HEX("2233"))\n'
                 'PRINT BUFFER_INDEXOF(h, BUFFER_FROM_HEX("11"))\n'
                 'PRINT BUFFER_INDEXOF(h, BUFFER_FROM_HEX("11"), 2)\n'
                 'PRINT BUFFER_INDEXOF(h, BUFFER_FROM_HEX("ff"))')
    assert out == "2\n1\n4\n-1\n"


# ------------------------------------------------- Text / Hex / Base64

def test_string_hin_und_zurueck(run_gb):
    assert run_gb('PRINT BUFFER_TO_STRING$(BUFFER_FROM_STRING("Grueezi"))') == "Grueezi\n"


def test_bytes_sind_nicht_zeichen(run_gb):
    """Der eigentliche Grund fuer den Typ: STRING ist UTF-8, LEN zaehlt
    Zeichen -- fuer Bytes braucht es etwas anderes."""
    out = run_gb('PRINT LEN("Gruesse")\nPRINT BUFFER_LEN(BUFFER_FROM_STRING("Gruesse"))\n'
                 'PRINT LEN("Grusz")\nPRINT BUFFER_LEN(BUFFER_FROM_STRING("Gr' + 'ü' + 'sz"))')
    zeilen = out.splitlines()
    assert zeilen[0] == zeilen[1] == "7"      # reines ASCII: gleich
    assert zeilen[2] == "5" and zeilen[3] == "6"   # ein Umlaut = 2 Bytes


def test_to_string_ist_streng_bei_kaputtem_utf8(run_gb):
    # Streng statt ersetzend: ein stilles U+FFFD faelschte die Daten.
    with pytest.raises(DHRuntimeError, match="kein gueltiges UTF-8"):
        run_gb('PRINT BUFFER_TO_STRING$(BUFFER_FROM_HEX("fffe"))')


def test_hex_hin_und_zurueck(run_gb):
    assert run_gb('PRINT BUFFER_TO_HEX$(BUFFER_FROM_HEX("DEADBEEF"))') == "deadbeef\n"


def test_hex_darf_leerzeichen_enthalten(run_gb):
    assert run_gb('PRINT BUFFER_TO_HEX$(BUFFER_FROM_HEX("de ad be ef"))') == "deadbeef\n"


def test_hex_ungerade_wirft(run_gb):
    with pytest.raises(DHRuntimeError, match="ungerade"):
        run_gb('PRINT BUFFER_FROM_HEX("abc")')


def test_hex_mit_unsinn_wirft(run_gb):
    with pytest.raises(DHRuntimeError, match="keine Hex-Ziffer"):
        run_gb('PRINT BUFFER_FROM_HEX("zz")')


def test_base64_hin_und_zurueck_mit_rohen_bytes(run_gb):
    # Anders als BASE64_DECODE (das UTF-8 verlangt) darf hier alles durch.
    assert run_gb('DIM b AS BUFFER\nb = BUFFER_FROM_HEX("fffe00")\n'
                  'PRINT BUFFER_TO_HEX$(BUFFER_FROM_BASE64(BUFFER_TO_BASE64$(b)))') == "fffe00\n"


# ------------------------------------------------------- Zahlen packen

@pytest.mark.parametrize("art,wert", [
    ("I16", "-1000"), ("U16", "65535"), ("I32", "-70000"),
    ("U32", "4000000000"), ("I64", "-9007199254740993"),
])
def test_ganzzahlen_hin_und_zurueck(run_gb, art, wert):
    out = run_gb(f'DIM b AS BUFFER\nb = BUFFER_NEW(8)\n'
                 f'BUFFER_SET_{art}(b, 0, {wert})\n'
                 f'PRINT BUFFER_GET_{art}(b, 0)')
    assert out == f"{wert}\n"


def test_gleitkomma_hin_und_zurueck(run_gb):
    out = run_gb('DIM b AS BUFFER\nb = BUFFER_NEW(8)\n'
                 'BUFFER_SET_F64(b, 0, 3.5)\nPRINT BUFFER_GET_F64(b, 0)\n'
                 'BUFFER_SET_F32(b, 0, 0.5)\nPRINT BUFFER_GET_F32(b, 0)')
    assert out == "3.5\n0.5\n"


def test_byte_reihenfolge_ist_wirklich_verschieden(run_gb):
    out = run_gb('DIM b AS BUFFER\nb = BUFFER_NEW(8)\n'
                 'BUFFER_SET_I32(b, 0, 1000)\n'
                 'BUFFER_SET_I32(b, 4, 1000, "be")\n'
                 'PRINT BUFFER_TO_HEX$(b)')
    # 1000 = 0x000003E8: little-endian e8030000, big-endian 000003e8
    assert out == "e8030000000003e8\n"


def test_vorgabe_ist_little_endian(run_gb):
    assert run_gb('DIM b AS BUFFER\nb = BUFFER_NEW(4)\n'
                  'BUFFER_SET_I32(b, 0, 1000, "le")\n'
                  'PRINT BUFFER_GET_I32(b, 0)') == "1000\n"


def test_falsche_byte_reihenfolge_wirft(run_gb):
    with pytest.raises(DHRuntimeError, match="Byte-Reihenfolge 'mittel' unbekannt"):
        run_gb('PRINT BUFFER_GET_I32(BUFFER_NEW(4), 0, "mittel")')


def test_zahl_ausserhalb_des_wertebereichs_wirft(run_gb):
    # Still abschneiden hiesse: eine voellig andere Zahl kommt wieder heraus.
    with pytest.raises(DHRuntimeError, match="ausserhalb 0..65535"):
        run_gb('BUFFER_SET_U16(BUFFER_NEW(4), 0, 70000)')


def test_zahl_ueber_das_pufferende_hinaus_wirft(run_gb):
    with pytest.raises(DHRuntimeError, match="4 Byte\\(s\\) ab Position 1"):
        run_gb('PRINT BUFFER_GET_I32(BUFFER_NEW(4), 1)')


# ---------------------------------------------------------- Binaerdateien

def test_writeall_und_readall_bytes(run_gb, tmp_path):
    out = run_gb('WRITEALL_BYTES("roh.dat", BUFFER_FROM_HEX("00ff0a0d1a"))\n'
                 'PRINT FILESIZE("roh.dat")\n'
                 'PRINT BUFFER_TO_HEX$(READALL_BYTES("roh.dat"))', base=tmp_path)
    # 0a/0d/1a bleiben unveraendert -- keine CRLF-Uebersetzung, kein
    # Ctrl-Z-als-Dateiende wie in alten BASICs.
    assert out == "5\n00ff0a0d1a\n"


def test_read_bytes_und_tell(run_gb, tmp_path):
    out = run_gb('DIM f AS FILE\n'
                 'WRITEALL_BYTES("b.dat", BUFFER_FROM_HEX("00010203040506070809"))\n'
                 'f = OPENFILE("b.dat", "r")\n'
                 'PRINT BUFFER_TO_HEX$(READ_BYTES(f, 3))\n'
                 'PRINT TELL(f)\n'
                 'CLOSEFILE(f)', base=tmp_path)
    assert out == "000102\n3\n"


def test_read_bytes_liefert_am_ende_weniger(run_gb, tmp_path):
    # Die uebliche Abbruchbedingung -- kein Fehler.
    out = run_gb('DIM f AS FILE\n'
                 'WRITEALL_BYTES("b.dat", BUFFER_FROM_HEX("0001"))\n'
                 'f = OPENFILE("b.dat", "r")\n'
                 'PRINT BUFFER_LEN(READ_BYTES(f, 100))\n'
                 'PRINT BUFFER_LEN(READ_BYTES(f, 100))\n'
                 'CLOSEFILE(f)', base=tmp_path)
    assert out == "2\n0\n"


def test_write_bytes(run_gb, tmp_path):
    out = run_gb('DIM f AS FILE\n'
                 'f = OPENFILE("w.dat", "w")\n'
                 'WRITE_BYTES(f, BUFFER_FROM_HEX("aabb"))\n'
                 'WRITE_BYTES(f, BUFFER_FROM_HEX("cc"))\n'
                 'PRINT TELL(f)\n'
                 'CLOSEFILE(f)\n'
                 'PRINT BUFFER_TO_HEX$(READALL_BYTES("w.dat"))', base=tmp_path)
    assert out == "3\naabbcc\n"


def test_seek_verwirft_den_lesepuffer(run_gb, tmp_path):
    """BufReader::seek muss den gepufferten Rest wegwerfen -- sonst laese ein
    READLINE nach dem SEEK noch die alten Bytes."""
    out = run_gb('DIM f AS FILE\n'
                 'WRITEALL("t.txt", "eins" + CHR$(10) + "zwei" + CHR$(10) + "drei" + CHR$(10))\n'
                 'f = OPENFILE("t.txt", "r")\n'
                 'PRINT READLINE(f)\n'
                 'SEEK(f, 10)\n'
                 'PRINT READLINE(f)\n'
                 'CLOSEFILE(f)', base=tmp_path)
    assert out == "eins\ndrei\n"


def test_seek_negativ_wirft(run_gb, tmp_path):
    with pytest.raises(DHRuntimeError, match="Position -1 ist negativ"):
        run_gb('DIM f AS FILE\nWRITEALL("t.txt", "x")\n'
               'f = OPENFILE("t.txt", "r")\nSEEK(f, -1)', base=tmp_path)


def test_read_bytes_auf_schreib_handle_wirft(run_gb, tmp_path):
    with pytest.raises(DHRuntimeError, match="nicht im Lese-Modus"):
        run_gb('DIM f AS FILE\nf = OPENFILE("x.dat", "w")\n'
               'PRINT BUFFER_LEN(READ_BYTES(f, 1))', base=tmp_path)


def test_write_bytes_auf_lese_handle_wirft(run_gb, tmp_path):
    with pytest.raises(DHRuntimeError, match="nicht im Schreib-Modus"):
        run_gb('DIM f AS FILE\nWRITEALL("x.dat", "y")\n'
               'f = OPENFILE("x.dat", "r")\n'
               'WRITE_BYTES(f, BUFFER_NEW(1))', base=tmp_path)


def test_readall_bytes_auf_fehlende_datei_wirft(run_gb):
    with pytest.raises(DHRuntimeError, match="READALL_BYTES:"):
        run_gb('PRINT BUFFER_LEN(READALL_BYTES("gibt_es_nicht_xyz.dat"))')


def test_jedes_byte_ueberlebt_den_dateiweg(run_gb, tmp_path):
    """Die Kernzusage von WP B: alle 256 Bytewerte gehen unveraendert durch
    eine Datei -- genau das kann der STRING-Weg nicht."""
    out = run_gb('DIM b AS BUFFER\nDIM i AS INTEGER\nDIM zurueck AS BUFFER\n'
                 'b = BUFFER_NEW(256)\n'
                 'FOR i = 0 TO 255\n'
                 '    BUFFER_SET(b, i, i)\n'
                 'NEXT\n'
                 'WRITEALL_BYTES("alle.dat", b)\n'
                 'zurueck = READALL_BYTES("alle.dat")\n'
                 'PRINT BUFFER_LEN(zurueck)\n'
                 'FOR i = 0 TO 255\n'
                 '    IF BUFFER_GET(zurueck, i) <> i THEN\n'
                 '        PRINT "FALSCH bei " + STR$(i)\n'
                 '    END IF\n'
                 'NEXT\n'
                 'PRINT "alle gleich"', base=tmp_path)
    assert out == "256\nalle gleich\n"
