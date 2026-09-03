"""Bilder anlegen, zusammensetzen, radieren, auslesen und speichern.

Ein IMAGE war bis 2026-08-31 eine Einbahnstrasse: hineinzeichnen ging
(`IMAGE_DRAW_RECT/CIRCLE/LINE/TEXT`), aber es gab kein durchsichtiges Bild,
kein Bild-in-Bild, kein Radieren, kein Auslesen der Deckkraft und **keinen
Weg, ein Bild zu speichern** (`SAVESCREENSHOT` sichert den Bildschirm, nicht
ein Bild). Aufgefallen bei der Frage, was einem Bild-Editor in Drachenhauch
fehlen wuerde.

Die geschriebenen Dateien liest **Pillow** gegen -- ein fremder Leser. Ein
Format, das nur sein eigener Schreiber wieder liest, ist nicht geprueft.
"""
import os
import subprocess
from pathlib import Path

import pytest

_ROOT = Path(__file__).resolve().parent.parent


def _find_dhrt():
    exe = "dhrt.exe" if os.name == "nt" else "dhrt"
    return next((_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe
                 for v in ("release", "debug")
                 if (_ROOT / "rust" / "drachenhauch_runtime" / "target" / v / exe).exists()), None)


_DHRT = _find_dhrt()
pytestmark = pytest.mark.skipif(_DHRT is None, reason="native Runtime 'dhrt' nicht gebaut")
Image = pytest.importorskip("PIL.Image", reason="Pillow fuer die Pixel-Pruefung noetig")


def _run(src, tmp_path, frames=2):
    voll = 'IMPORT "imgfx"\nSCREEN(160, 80, "t", 1)\n' + src
    (tmp_path / "s.dh").write_text(voll, encoding="utf-8")
    env = dict(os.environ, DHRT_FRAMES=str(frames))
    r = subprocess.run([str(_DHRT), "run", str(tmp_path / "s.dh")], capture_output=True,
                       text=True, encoding="utf-8", env=env, timeout=90, cwd=str(tmp_path))
    assert r.returncode == 0, (r.stdout or "") + (r.stderr or "")
    return [ln for ln in (r.stdout or "").splitlines()
            if not ln.startswith(("WARNING:", "INFO:", "TRACE:"))]


def test_neues_bild_ist_ohne_farbe_vollstaendig_durchsichtig(tmp_path):
    """Der eigentliche Grund fuer `IMAGE_NEW`: die Farbkonvention deutet
    Alpha 0 als DECKEND, ein durchsichtiges Bild liess sich ueber eine FARBE
    (und damit ueber `GENTEX_COLOR`) gar nicht ausdruecken."""
    aus = _run('''
DIM a AS IMAGE : a = IMAGE_NEW(8, 8)
DIM b AS IMAGE : b = IMAGE_NEW(8, 8, &HFF0000)
PRINT GETALPHA(a, 1, 1); " "; GETALPHA(b, 1, 1); " "; HEX$(GETPIXEL(b, 1, 1))
''', tmp_path)
    assert aus[0].split() == ["0", "255", "FF0000"]


def test_clear_radiert_wirklich_statt_zu_mischen(tmp_path):
    """Ein durchsichtiges Rechteck EINZUMISCHEN waere ein Nichts-Tun --
    Radieren waere damit unmoeglich. Geprueft wird, dass es schreibt."""
    aus = _run('''
DIM b AS IMAGE : b = IMAGE_NEW(8, 8, &HFF0000)
IMAGE_CLEAR(b, 2, 2, 3, 3)
PRINT GETALPHA(b, 3, 3); " "; GETALPHA(b, 0, 0); " "; GETALPHA(b, 5, 5)
''', tmp_path)
    # innen leer, links oben und rechts unterhalb des Rechtecks unberuehrt
    assert aus[0].split() == ["0", "255", "255"]


def test_clear_ohne_rechteck_leert_das_ganze_bild(tmp_path):
    aus = _run('''
DIM b AS IMAGE : b = IMAGE_NEW(8, 8, &HFF0000)
IMAGE_CLEAR(b)
PRINT GETALPHA(b, 0, 0); " "; GETALPHA(b, 7, 7)
''', tmp_path)
    assert aus[0].split() == ["0", "0"]


def test_bild_in_bild_zeichnen_und_mischen(tmp_path):
    aus = _run('''
DIM z AS IMAGE : z = IMAGE_NEW(16, 16, &H0000FF)
DIM s AS IMAGE : s = IMAGE_NEW(4, 4, &H00FF00)
IMAGE_DRAW_IMAGE(z, s, 5, 5)
PRINT HEX$(GETPIXEL(z, 6, 6)); " "; HEX$(GETPIXEL(z, 1, 1))
DIM h AS IMAGE : h = IMAGE_NEW(4, 4, RGBA(255, 0, 0, 128))
IMAGE_DRAW_IMAGE(z, h, 0, 0)
PRINT GETPIXEL(z, 1, 1) <> &H0000FF
''', tmp_path)
    # getroffen = gruen, daneben = unveraendert blau
    assert aus[0].split() == ["FF00", "FF"]
    # halbdurchsichtig ueber blau ergibt eine Mischfarbe, kein Ersetzen
    assert aus[1].strip() == "TRUE"


def test_nur_ein_ausschnitt_der_quelle(tmp_path):
    aus = _run('''
DIM q AS IMAGE : q = IMAGE_NEW(8, 8, &HFF0000)
DIM z AS IMAGE : z = IMAGE_NEW(8, 8)
IMAGE_DRAW_IMAGE(z, q, 0, 0, 0, 0, 4, 4)
PRINT GETALPHA(z, 1, 1); " "; GETALPHA(z, 6, 6)
''', tmp_path)
    assert aus[0].split() == ["255", "0"]


def test_getalpha_meldet_ausserhalb_minus_eins(tmp_path):
    """Gleiche Zusage wie `GETPIXEL` -- sonst waere -1 mal Fehler, mal Farbe."""
    aus = _run('''
DIM b AS IMAGE : b = IMAGE_NEW(4, 4, &HFF0000)
PRINT GETALPHA(b, -1, 0); " "; GETALPHA(b, 0, 9); " "; GETALPHA(b, 3, 3)
''', tmp_path)
    assert aus[0].split() == ["-1", "-1", "255"]


def test_gespeichertes_png_liest_ein_fremdes_programm(tmp_path):
    _run('''
DIM b AS IMAGE : b = IMAGE_NEW(6, 4, &H0000FF)
IMAGE_DRAW_RECT(b, 1, 1, 2, 2, &H00FF00)
IMAGE_SAVE(b, "raus.png")
''', tmp_path)
    im = Image.open(tmp_path / "raus.png")
    assert im.format == "PNG"
    assert im.size == (6, 4)
    im = im.convert("RGBA")
    assert im.getpixel((2, 2)) == (0, 255, 0, 255)
    assert im.getpixel((5, 3)) == (0, 0, 255, 255)


def test_durchsichtigkeit_uebersteht_das_speichern(tmp_path):
    """Der Rundweg, auf den es bei Sprites ankommt: was radiert wurde, muss
    in der Datei als Alpha 0 stehen -- nicht als beinahe-durchsichtig."""
    _run('''
DIM b AS IMAGE : b = IMAGE_NEW(4, 4)
IMAGE_DRAW_RECT(b, 0, 0, 2, 2, &HFF0000)
IMAGE_SAVE(b, "alpha.png")
''', tmp_path)
    im = Image.open(tmp_path / "alpha.png").convert("RGBA")
    assert im.getpixel((0, 0)) == (255, 0, 0, 255)
    assert im.getpixel((3, 3))[3] == 0, "leere Flaeche muss Alpha 0 haben"


def test_unbekannte_endung_wird_abgelehnt(tmp_path):
    """Die Endung entscheidet ueber das Format. raylib wuerde bei einer
    unbekannten nur eine Warnung ins Protokoll schreiben und nichts tun --
    das Programm haette geglaubt, es sei gespeichert."""
    aus = _run('''
DIM b AS IMAGE : b = IMAGE_NEW(4, 4, &HFF0000)
TRY
    IMAGE_SAVE(b, "x.xyz")
    PRINT "angenommen"
CATCH e
    PRINT "abgelehnt"
END TRY
''', tmp_path)
    assert aus[0].strip() == "abgelehnt"
    assert not (tmp_path / "x.xyz").exists()


def test_speichern_in_ein_fehlendes_verzeichnis_meldet_sich(tmp_path):
    """Die raylib-Bindung wirft das Erfolgs-Flag weg (`export_image` liefert
    `()`), es bleibt nur der Blick auf die Datei. Ohne diese Pruefung waere
    ein misslungenes Speichern lautlos."""
    aus = _run('''
DIM b AS IMAGE : b = IMAGE_NEW(4, 4, &HFF0000)
TRY
    IMAGE_SAVE(b, "gibtsnicht/x.png")
    PRINT "angenommen"
CATCH e
    PRINT "abgelehnt"
END TRY
''', tmp_path)
    assert aus[0].strip() == "abgelehnt"


def test_bmp_geht_auch(tmp_path):
    _run('''
DIM b AS IMAGE : b = IMAGE_NEW(4, 4, &H00FF00)
IMAGE_SAVE(b, "raus.bmp")
''', tmp_path)
    im = Image.open(tmp_path / "raus.bmp")
    assert im.format == "BMP"
    assert im.convert("RGB").getpixel((1, 1)) == (0, 255, 0)


def test_freigeben_macht_das_handle_unbenutzbar(tmp_path):
    """Der Sinn von `IMAGE_FREE` ist nicht das Freigeben allein, sondern dass
    das Handle danach nicht STILL weiterbenutzt werden kann. Der Platz bleibt
    deshalb stehen und wird nicht neu vergeben."""
    aus = _run('''
DIM b AS IMAGE : b = IMAGE_NEW(16, 16, &HFF0000)
IMAGE_FREE(b)
TRY
    PRINT IMAGEWIDTH(b)
CATCH e
    PRINT "breite abgelehnt"
END TRY
TRY
    IMAGE_DRAW_RECT(b, 0, 0, 2, 2, &H00FF00)
CATCH e
    PRINT "zeichnen abgelehnt"
END TRY
TRY
    DIM k AS IMAGE : k = IMAGE_COPY(b)
CATCH e
    PRINT "kopieren abgelehnt"
END TRY
''', tmp_path)
    assert aus == ["breite abgelehnt", "zeichnen abgelehnt", "kopieren abgelehnt"]


def test_die_meldung_unterscheidet_freigegeben_von_nie_dagewesen(tmp_path):
    """Zwei verschiedene Fehler. Wer den einen fuer den anderen haelt, sucht
    an der falschen Stelle."""
    aus = _run('''
DIM b AS IMAGE : b = IMAGE_NEW(8, 8, &HFF0000)
IMAGE_FREE(b)
TRY
    IMAGE_FREE(b)
CATCH e
    PRINT e
END TRY
TRY
    IMAGE_FREE(9999)
CATCH e
    PRINT e
END TRY
''', tmp_path)
    assert "freigegeben" in aus[0]
    assert "ungueltiges IMAGE-Handle" in aus[1] and "freigegeben" not in aus[1]


def test_getpixel_und_getalpha_bleiben_bei_minus_eins(tmp_path):
    """Beide melden Ungueltiges seit jeher mit -1, nicht mit einem Fehler --
    dabei bleibt es auch fuer ein freigegebenes Bild."""
    aus = _run('''
DIM b AS IMAGE : b = IMAGE_NEW(8, 8, &HFF0000)
IMAGE_FREE(b)
PRINT GETPIXEL(b, 1, 1); " "; GETALPHA(b, 1, 1)
''', tmp_path)
    assert aus[0].split() == ["-1", "-1"]


def test_ein_neues_bild_bekommt_nicht_den_freien_platz(tmp_path):
    """Wuerde der Platz neu vergeben, zeigte ein stehengebliebenes Handle
    still auf ein FREMDES Bild -- der klassische Fehler nach dem Freigeben."""
    aus = _run('''
DIM a AS IMAGE : a = IMAGE_NEW(8, 8, &HFF0000)
DIM b AS IMAGE : b = IMAGE_NEW(8, 8, &H00FF00)
IMAGE_FREE(a)
DIM c AS IMAGE : c = IMAGE_NEW(8, 8, &H0000FF)
PRINT (c = a); " "; (c = b)
''', tmp_path)
    assert aus[0].split() == ["FALSE", "FALSE"]


def test_freigeben_wirft_das_bild_aus_dem_pfad_cache(tmp_path):
    """`LOADIMAGE` merkt sich Pfad -> Handle. Bliebe der Eintrag stehen,
    bekaeme der naechste Aufruf fuer denselben Pfad ein freigegebenes Bild --
    und zwar ohne jeden Hinweis, weil er ja etwas zurueckbekommt."""
    aus = _run('''
DIM q AS IMAGE : q = IMAGE_NEW(8, 8, &HFF0000)
IMAGE_SAVE(q, "x.png")
DIM a AS IMAGE : a = LOADIMAGE("x.png")
IMAGE_FREE(a)
DIM b AS IMAGE : b = LOADIMAGE("x.png")
PRINT IMAGEWIDTH(b); " "; (b = a)
''', tmp_path)
    assert aus[0].split() == ["8", "FALSE"]


def test_freigeben_gibt_den_speicher_wirklich_frei(tmp_path):
    """Ohne diese Zusage waere der Befehl nur Buchhaltung. Gemessen wird
    nicht der Speicher (das kann der Test nicht), sondern dass sich sehr
    viele Kopien mit Freigabe ueberhaupt durchhalten lassen -- ohne sie
    belegten 1200 Bilder zu 256x256 rund 300 MB mehr."""
    aus = _run('''
DIM b AS IMAGE : b = IMAGE_NEW(256, 256, &HFF0000)
DIM i AS INTEGER
DIM h AS IMAGE
FOR i = 1 TO 1200
    h = IMAGE_COPY(b)
    IMAGE_FREE(h)
NEXT
PRINT "durch"
''', tmp_path, frames=40)
    assert aus == ["durch"]


# ---------------------------------------------------------------------------
# Animierte GIFs (IMAGE_SAVE_GIF)
#
# raylib kann GIFs nur LESEN. Fuer den Sprite-Editor war das die letzte
# Einbahnstrasse: Einzelbilder erzeugen ja, sie als Bewegung ausgeben nicht.
# Geschrieben wird ueber die `gif`-Crate -- ein LZW-Kodierer von Hand ist die
# Art Code, die auf den ersten Blick stimmt und im Randfall still etwas
# Falsches liefert.

def _gif(tmp_path, name="a.gif"):
    from PIL import Image
    return Image.open(tmp_path / name)


def test_gif_hat_die_bilder_und_das_tempo(tmp_path):
    _run('''
DIM b[3] AS IMAGE
DIM f AS ARRAY OF INTEGER : f = [&HE84B4B, &H4BE87A, &H2BC4E8]
DIM i AS INTEGER
FOR i = 0 TO 2
    b[i] = IMAGE_NEW(8, 8)
    IMAGE_DRAW_RECT(b[i], i * 2, 1, 2, 6, f[i])
NEXT
IMAGE_SAVE_GIF(b, "a.gif", 8)
''', tmp_path)
    im = _gif(tmp_path)
    assert im.format == "GIF" and im.size == (8, 8)
    assert im.n_frames == 3
    # 8 Bilder/s -> 100/8 = 12,5 -> 13 Hundertstel -> 130 ms
    assert im.info.get("duration") == 130
    assert im.info.get("loop") == 0, "0 heisst endlos"


def test_jedes_bild_kann_seine_eigene_dauer_haben(tmp_path):
    """Der dritte Parameter ist ZWEIERLEI: eine Zahl sind Bilder je Sekunde
    fuer alle, ein FELD ist die Dauer JE BILD in Millisekunden. GIF kann das
    von Haus aus, und eine Bildfolge braucht es -- eine Pose wird gehalten,
    eine Bewegung laeuft schnell durch.

    Gelesen wird mit Pillow, einem FREMDEN Leser: dass die eigene Datei die
    eigenen Zahlen enthaelt, waere die schwaechere Aussage.
    """
    _run('''
DIM b[3] AS IMAGE
b[0] = IMAGE_NEW(8, 8, RGB(255, 0, 0))
b[1] = IMAGE_NEW(8, 8, RGB(0, 255, 0))
b[2] = IMAGE_NEW(8, 8, RGB(0, 0, 255))
IMAGE_SAVE_GIF(b, "a.gif", [1000, 80, 500])
''', tmp_path)
    from PIL import ImageSequence
    dauern = [f.info.get("duration") for f in ImageSequence.Iterator(_gif(tmp_path))]
    assert dauern == [1000, 80, 500]


def test_eine_zahl_gilt_weiter_fuer_alle(tmp_path):
    """Die Gegenprobe zum Test darueber: ohne sie waere nicht belegt, dass
    die beiden Formen ueberhaupt etwas Verschiedenes tun."""
    _run('''
DIM b[3] AS IMAGE
DIM i AS INTEGER
FOR i = 0 TO 2
    b[i] = IMAGE_NEW(8, 8, RGB(i * 100, 0, 0))
NEXT
IMAGE_SAVE_GIF(b, "a.gif", 10)
''', tmp_path)
    from PIL import ImageSequence
    dauern = [f.info.get("duration") for f in ImageSequence.Iterator(_gif(tmp_path))]
    assert dauern == [100, 100, 100]


def test_zu_wenige_zeiten_sind_ein_fehler(tmp_path):
    """Stillschweigend die letzte zu wiederholen waere eine Vermutung -- und
    eine falsche Zeit sieht man dem GIF nicht an, man merkt sie nur."""
    aus = _run('''
DIM b[3] AS IMAGE
DIM i AS INTEGER
FOR i = 0 TO 2
    b[i] = IMAGE_NEW(8, 8, RGB(255, 0, 0))
NEXT
TRY
    IMAGE_SAVE_GIF(b, "a.gif", [100, 200])
    PRINT "angenommen"
CATCH e
    PRINT e
END TRY
''', tmp_path)
    assert "3 Bilder" in aus[0] and "2 Zeiten" in aus[0]


def test_eine_zeit_von_null_wird_abgelehnt(tmp_path):
    aus = _run('''
DIM b[2] AS IMAGE
b[0] = IMAGE_NEW(8, 8, RGB(255, 0, 0))
b[1] = IMAGE_NEW(8, 8, RGB(0, 255, 0))
TRY
    IMAGE_SAVE_GIF(b, "a.gif", [100, 0])
    PRINT "angenommen"
CATCH e
    PRINT e
END TRY
''', tmp_path)
    assert "Zeit 2" in aus[0]


def test_sehr_kurze_zeiten_werden_angehoben(tmp_path):
    """Unter 2 Hundertstel legen die meisten Betrachter still ihre eigene
    Dauer fest (meist 10) -- die Ausgabe liefe dann LANGSAMER als verlangt,
    ohne Hinweis. Dieselbe Klemmung wie bei der Bildrate."""
    _run('''
DIM b[2] AS IMAGE
b[0] = IMAGE_NEW(8, 8, RGB(255, 0, 0))
b[1] = IMAGE_NEW(8, 8, RGB(0, 255, 0))
IMAGE_SAVE_GIF(b, "a.gif", [5, 1000])
''', tmp_path)
    from PIL import ImageSequence
    dauern = [f.info.get("duration") for f in ImageSequence.Iterator(_gif(tmp_path))]
    assert dauern == [20, 1000]


def test_wenige_farben_bleiben_exakt(tmp_path):
    """Bei Pixelgrafik wird die Farbtafel EXAKT gebaut. Ein Verfahren, das
    immer zusammenfasst, haette schon ein Vier-Farben-Sprite verfaelscht."""
    from PIL import ImageSequence
    _run('''
DIM b[2] AS IMAGE
b[0] = IMAGE_NEW(4, 4, &HE84B4B)
b[1] = IMAGE_NEW(4, 4, &H2BC4E8)
IMAGE_SAVE_GIF(b, "a.gif", 10)
''', tmp_path)
    bilder = [f.convert("RGBA").getpixel((1, 1))
              for f in ImageSequence.Iterator(_gif(tmp_path))]
    assert bilder == [(232, 75, 75, 255), (43, 196, 232, 255)]


def test_durchsichtigkeit_kommt_mit(tmp_path):
    from PIL import ImageSequence
    _run('''
DIM b[1] AS IMAGE
b[0] = IMAGE_NEW(6, 6)
IMAGE_DRAW_RECT(b[0], 0, 0, 3, 3, &HFF0000)
IMAGE_SAVE_GIF(b, "a.gif", 10)
''', tmp_path)
    erste = next(ImageSequence.Iterator(_gif(tmp_path))).convert("RGBA")
    assert erste.getpixel((1, 1)) == (255, 0, 0, 255)
    assert erste.getpixel((5, 5))[3] == 0, "leere Flaeche muss durchsichtig bleiben"


def test_anzahl_begrenzt_ein_festes_feld(tmp_path):
    """Ein `DIM b[16] AS IMAGE` mit drei belegten Plaetzen ist der Normalfall.
    Ohne `anzahl` waeren die leeren Plaetze Fehler -- mit ihr gelten nur die
    vorderen."""
    _run('''
DIM b[16] AS IMAGE
DIM i AS INTEGER
FOR i = 0 TO 2
    b[i] = IMAGE_NEW(4, 4, &HFF0000)
NEXT
IMAGE_SAVE_GIF(b, "a.gif", 10, TRUE, 3)
''', tmp_path)
    assert _gif(tmp_path).n_frames == 3


def test_leerer_platz_ohne_anzahl_meldet_sich(tmp_path):
    """... und ohne `anzahl` sagt die Meldung, was zu tun ist."""
    aus = _run('''
DIM b[4] AS IMAGE
b[0] = IMAGE_NEW(4, 4, &HFF0000)
TRY
    IMAGE_SAVE_GIF(b, "a.gif", 10)
    PRINT "angenommen"
CATCH e
    PRINT e
END TRY
''', tmp_path)
    assert "anzahl" in aus[0] and "leer" in aus[0]


def test_wiederholen_laesst_sich_abschalten(tmp_path):
    _run('''
DIM b[1] AS IMAGE
b[0] = IMAGE_NEW(4, 4, &HFF0000)
IMAGE_SAVE_GIF(b, "a.gif", 10, FALSE)
''', tmp_path)
    assert _gif(tmp_path).info.get("loop") is None


def test_verschieden_grosse_bilder_werden_abgelehnt(tmp_path):
    """Ein GIF hat EINE Leinwand. Ein abweichendes Bild waere beschnitten
    oder verschoben -- beides waere stiller Verlust."""
    aus = _run('''
DIM b[2] AS IMAGE
b[0] = IMAGE_NEW(4, 4, &HFF0000)
b[1] = IMAGE_NEW(5, 4, &HFF0000)
TRY
    IMAGE_SAVE_GIF(b, "a.gif", 10)
    PRINT "angenommen"
CATCH e
    PRINT "abgelehnt"
END TRY
''', tmp_path)
    assert aus[0].strip() == "abgelehnt"


def test_unsinniges_tempo_wird_abgelehnt(tmp_path):
    aus = _run('''
DIM b[1] AS IMAGE
b[0] = IMAGE_NEW(4, 4, &HFF0000)
TRY
    IMAGE_SAVE_GIF(b, "a.gif", 0)
    PRINT "angenommen"
CATCH e
    PRINT "abgelehnt"
END TRY
''', tmp_path)
    assert aus[0].strip() == "abgelehnt"


def test_viele_farben_werden_zusammengefasst(tmp_path):
    """Ueber 255 Farben kann GIF nicht -- dann wird zusammengefasst. Der
    Zweig darf nicht abstuerzen und muss ein lesbares Bild liefern."""
    _run('''
DIM b[1] AS IMAGE
b[0] = GENTEX_PERLIN(48, 48, 8.0)
IMAGE_SAVE_GIF(b, "a.gif", 5)
''', tmp_path)
    im = _gif(tmp_path)
    assert im.size == (48, 48)
    assert len(im.convert("RGB").getcolors(70000)) <= 256
