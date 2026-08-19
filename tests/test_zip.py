"""WP J -- ZIP lesen und schreiben.

Der wichtigste Test hier ist `test_zip_slip_...`: ein Archiv darf Eintraege
wie `../../autoexec.bat` enthalten, und wer beim Entpacken den Namen aus dem
Archiv einfach an den Zielordner haengt, schreibt damit ausserhalb davon. Der
Angreifer waehlt die Datei, das Opfer entpackt sie.

Die Namenspruefung selbst pruefen die Rust-`#[test]`s in `src/zipdatei.rs`.
Hier wird ein echtes boesartiges Archiv gebaut und dhrt darauf losgelassen --
weil eine Pruefung, die zwar existiert, aber im Entpackpfad nicht aufgerufen
wird, genau so aussieht wie eine, die wirkt.
"""
import zipfile

import pytest


def _p(pfad) -> str:
    return str(pfad).replace("\\", "/")


def test_schreiben_lesen_auflisten(run_gb, tmp_path):
    z = _p(tmp_path / "a.zip")
    out = run_gb(
        "DIM namen AS ARRAY OF STRING\n"
        "DIM inhalte AS ARRAY OF STRING\n"
        "DIM drin AS ARRAY OF STRING\n"
        'namen = SPLIT$("brief.txt|unter/notiz.txt", "|")\n'
        'inhalte = SPLIT$("Hallo Welt|zweite", "|")\n'
        'PRINT ZIP_WRITE("' + z + '", namen, inhalte)\n'
        'drin = ZIP_LIST("' + z + '")\n'
        'PRINT JOIN$(drin, ",")\n'
        'PRINT ZIP_READ$("' + z + '", "brief.txt")\n')
    assert out.split("\n")[:3] == ["2", "brief.txt,unter/notiz.txt", "Hallo Welt"]


def test_entpacken_legt_unterordner_an(run_gb, tmp_path):
    z = _p(tmp_path / "b.zip")
    ziel = _p(tmp_path / "raus")
    out = run_gb(
        "DIM namen AS ARRAY OF STRING\n"
        "DIM inhalte AS ARRAY OF STRING\n"
        'namen = SPLIT$("tief/im/ordner.txt", "|")\n'
        'inhalte = SPLIT$("da", "|")\n'
        'ZIP_WRITE("' + z + '", namen, inhalte)\n'
        'PRINT ZIP_EXTRACT("' + z + '", "' + ziel + '")\n'
        'PRINT STR$(FILEEXISTS("' + ziel + '/tief/im/ordner.txt"))\n')
    assert out.split("\n")[:2] == ["1", "TRUE"]
    assert (tmp_path / "raus" / "tief" / "im" / "ordner.txt").read_text() == "da"


def test_zip_slip_schreibt_nicht_ausserhalb(run_gb, tmp_path):
    """Ein Archiv mit `../` im Eintragsnamen. Der Eintrag wird uebersprungen,
    nicht geschrieben -- und `ZIP_EXTRACT` zaehlt nur, was wirklich entstand,
    damit der Unterschied auffaellt."""
    z = tmp_path / "boese.zip"
    with zipfile.ZipFile(z, "w") as f:
        f.writestr("harmlos.txt", "ok")
        f.writestr("../entkommen.txt", "sollte nie entstehen")
        f.writestr("../../weiter_oben.txt", "auch nicht")
    ziel = tmp_path / "ziel"
    out = run_gb('PRINT ZIP_EXTRACT("' + _p(z) + '", "' + _p(ziel) + '")\n')

    assert out.strip() == "1", "nur der harmlose Eintrag darf entstehen"
    assert (ziel / "harmlos.txt").exists()
    assert not (tmp_path / "entkommen.txt").exists()
    assert not (tmp_path.parent / "weiter_oben.txt").exists()


def test_zip_slip_mit_absolutem_pfad(run_gb, tmp_path):
    z = tmp_path / "abs.zip"
    with zipfile.ZipFile(z, "w") as f:
        f.writestr("gut.txt", "ok")
        f.writestr("/tmp/absolut.txt", "nein")
        f.writestr("C:/Windows/x.txt", "nein")
    ziel = tmp_path / "ziel2"
    out = run_gb('PRINT ZIP_EXTRACT("' + _p(z) + '", "' + _p(ziel) + '")\n')
    assert out.strip() == "1", out


def test_create_speichert_nur_den_dateinamen(run_gb, tmp_path):
    """Sonst traegt ein Archiv die Verzeichnisstruktur des Rechners nach
    aussen, auf dem es entstanden ist."""
    q = tmp_path / "unterordner"
    q.mkdir()
    (q / "beleg.txt").write_text("Inhalt")
    z = _p(tmp_path / "c.zip")
    out = run_gb(
        "DIM dateien AS ARRAY OF STRING\n"
        "DIM drin AS ARRAY OF STRING\n"
        'dateien = SPLIT$("' + _p(q / "beleg.txt") + '", "|")\n'
        'PRINT ZIP_CREATE("' + z + '", dateien)\n'
        'drin = ZIP_LIST("' + z + '")\n'
        'PRINT drin[0]\n')
    assert out.split("\n")[:2] == ["1", "beleg.txt"]


def test_read_als_buffer(run_gb, tmp_path):
    z = _p(tmp_path / "d.zip")
    out = run_gb(
        "DIM namen AS ARRAY OF STRING\n"
        "DIM inhalte AS ARRAY OF STRING\n"
        "DIM b AS BUFFER\n"
        'namen = SPLIT$("x.bin", "|")\n'
        'inhalte = SPLIT$("ABC", "|")\n'
        'ZIP_WRITE("' + z + '", namen, inhalte)\n'
        'b = ZIP_READ("' + z + '", "x.bin")\n'
        "PRINT BUFFER_LEN(b)\n"
        "PRINT BUFFER_GET(b, 0)\n")
    assert out.split("\n")[:2] == ["3", "65"]


def test_unbekannter_eintrag_meldet_den_namen(run_gb, tmp_path):
    from drachenhauch.errors import DHRuntimeError
    z = _p(tmp_path / "e.zip")
    with pytest.raises(DHRuntimeError, match="gibtsnicht"):
        run_gb(
            "DIM namen AS ARRAY OF STRING\n"
            "DIM inhalte AS ARRAY OF STRING\n"
            'namen = SPLIT$("a.txt", "|")\n'
            'inhalte = SPLIT$("x", "|")\n'
            'ZIP_WRITE("' + z + '", namen, inhalte)\n'
            'PRINT ZIP_READ$("' + z + '", "gibtsnicht.txt")\n')


def test_kaputte_datei_meldet_klar(run_gb, tmp_path):
    from drachenhauch.errors import DHRuntimeError
    kaputt = tmp_path / "kaputt.zip"
    kaputt.write_text("das ist kein ZIP")
    with pytest.raises(DHRuntimeError, match="kein lesbares ZIP"):
        run_gb("DIM d AS ARRAY OF STRING\n"
               'd = ZIP_LIST("' + _p(kaputt) + '")\n')


def test_ungleich_lange_arrays_melden(run_gb, tmp_path):
    from drachenhauch.errors import DHRuntimeError
    z = _p(tmp_path / "f.zip")
    with pytest.raises(DHRuntimeError, match="gleich lang"):
        run_gb(
            "DIM namen AS ARRAY OF STRING\n"
            "DIM inhalte AS ARRAY OF STRING\n"
            'namen = SPLIT$("a|b", "|")\n'
            'inhalte = SPLIT$("nur eins", "|")\n'
            'ZIP_WRITE("' + z + '", namen, inhalte)\n')
