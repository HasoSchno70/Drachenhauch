"""EPUB-Bau des Lehrbuchs pruefen.

Ein EPUB ist ein ZIP mit strengen Regeln: falsch abgelegte "mimetype"-Datei,
eine nicht wohlgeformte XHTML-Seite oder ein Verweis ins Leere, und das
Lesegeraet zeigt gar nichts an oder bricht mitten im Buch ab. Von aussen sieht
die Datei dabei jedes Mal gleich aus -- deshalb wird hier nicht "laeuft der
Build durch" geprueft, sondern der Inhalt des Archivs.
"""
import posixpath
import re
import shutil
import subprocess
import xml.etree.ElementTree as ET
import zipfile

import pytest

from pathlib import Path

BUCH = Path(__file__).resolve().parent.parent / "buch-referenz" / "buch"

OPF_NS = "{http://www.idpf.org/2007/opf}"
CONTAINER_NS = "{urn:oasis:names:tc:opendocument:xmlns:container}"

pytestmark = pytest.mark.skipif(
    not (BUCH / "build_epub.js").exists() or shutil.which("node") is None,
    reason="Buch-Verzeichnis oder node fehlt",
)


@pytest.fixture(scope="module")
def epub(tmp_path_factory):
    """Das Buch einmal bauen und als geoeffnetes Archiv liefern.

    Gebaut wird in ein Wegwerf-Verzeichnis, NICHT ueber die eingecheckte
    Datei: in der steckt ein Zeitstempel (`dcterms:modified` ist in EPUB 3
    Pflicht), ein Neubau am selben Ort riss also bei JEDEM Testlauf 4,8 MB
    Unterschied im Arbeitsverzeichnis auf."""
    if not (BUCH / "node_modules" / "jszip").exists():
        pytest.skip("node_modules fehlt (npm install im buch-Verzeichnis)")
    ziel = tmp_path_factory.mktemp("epub") / "probe.epub"
    res = subprocess.run(["node", "build_epub.js", str(ziel)], cwd=BUCH,
                         capture_output=True, text=True, timeout=300)
    assert res.returncode == 0, f"Build fehlgeschlagen:\n{res.stderr}"
    assert ziel.exists(), "Build meldete Erfolg, schrieb aber nichts"
    with zipfile.ZipFile(ziel) as z:
        yield z


def _opf(z):
    c = ET.fromstring(z.read("META-INF/container.xml"))
    pfad = c.find(f"{CONTAINER_NS}rootfiles/{CONTAINER_NS}rootfile").get("full-path")
    return pfad, ET.fromstring(z.read(pfad))


def test_mimetype_ist_erster_eintrag_und_unkomprimiert(epub):
    """Die einzige Regel, an der ein EPUB komplett scheitert: manche Leser
    lesen die ersten Bytes des ZIP roh und erkennen die Datei sonst nicht."""
    erster = epub.infolist()[0]
    assert erster.filename == "mimetype"
    assert erster.compress_type == zipfile.ZIP_STORED
    assert epub.read("mimetype") == b"application/epub+zip"


def test_alle_xml_dateien_sind_wohlgeformt(epub):
    """Die Seiten entstehen aus zusammengesetzten Zeichenketten -- ein nicht
    maskiertes & oder < im Kapiteltext bricht die Seite, und der Leser sieht
    an dieser Stelle einfach nichts mehr."""
    namen = [n for n in epub.namelist()
             if n.endswith((".xhtml", ".opf", ".ncx", ".xml"))]
    assert len(namen) > 50, "verdaechtig wenige Seiten"
    for n in namen:
        try:
            ET.fromstring(epub.read(n))
        except ET.ParseError as e:
            pytest.fail(f"{n} ist nicht wohlgeformt: {e}")


def test_manifest_spine_und_archiv_stimmen_ueberein(epub):
    pfad, opf = _opf(epub)
    assert pfad in epub.namelist()
    basis = posixpath.dirname(pfad)

    manifest = {}
    for it in opf.find(f"{OPF_NS}manifest"):
        ziel = posixpath.normpath(posixpath.join(basis, it.get("href")))
        manifest[it.get("id")] = ziel
        assert ziel in epub.namelist(), f"Manifest nennt {ziel}, fehlt im Archiv"

    spine = [r.get("idref") for r in opf.find(f"{OPF_NS}spine")]
    assert spine, "leerer Spine -- das Buch haette keine Lesereihenfolge"
    for ref in spine:
        assert ref in manifest, f"Spine verweist auf unbekannte id {ref}"

    # Umgekehrt: eine Datei, die keiner kennt, waere unsichtbarer Ballast.
    bekannt = set(manifest.values()) | {"mimetype", "META-INF/container.xml", pfad}
    verwaist = [n for n in epub.namelist()
                if n not in bekannt and not n.endswith("/")]
    assert not verwaist, f"nicht im Manifest: {verwaist}"


def test_alle_internen_verweise_loesen_sich_auf(epub):
    """JEDER Verweis, nicht nur die Bilder: Kapitel-Links im Verzeichnis,
    Stylesheets, Bilder, die Ziele im NCX.

    Alle sind relativ zu der Datei, in der sie stehen. Beim ersten Bau lag
    nav.xhtml eine Ebene ueber den Kapiteln und verlinkte trotzdem, als laege
    es daneben -- alle 84 Verzeichnis-Eintraege und sein Stylesheet zeigten ins
    Leere. Die Datei baute, oeffnete und zeigte Text; nur das Verzeichnis war
    tot. Ein Test, der bloss nachsieht, ob die Titel VORKOMMEN, merkt davon
    nichts."""
    namen = set(epub.namelist())
    geprueft = {"bild": 0, "seite": 0, "css": 0}
    kaputt = []
    for n in sorted(x for x in namen if x.endswith((".xhtml", ".ncx"))):
        inhalt = epub.read(n).decode("utf-8")
        for verweis in re.findall(r'(?:href|src)="([^"]+)"', inhalt):
            if verweis.startswith(("http:", "https:", "#")):
                continue
            ziel = posixpath.normpath(
                posixpath.join(posixpath.dirname(n), verweis.split("#")[0]))
            if ziel not in namen:
                kaputt.append(f"{n} -> {verweis}")
            art = ("bild" if ziel.endswith(".png")
                   else "css" if ziel.endswith(".css") else "seite")
            geprueft[art] += 1

    assert not kaputt, f"{len(kaputt)} Verweise ins Leere, u.a.: {kaputt[:5]}"
    # Nicht nur "nichts kaputt" -- es muss auch wirklich etwas geprueft worden
    # sein, sonst besteht der Test ein leeres Buch.
    assert geprueft["bild"] > 20, f"nur {geprueft['bild']} Bildverweise"
    assert geprueft["seite"] > 150, f"nur {geprueft['seite']} Seitenverweise"
    assert geprueft["css"] > 50, f"nur {geprueft['css']} Stylesheet-Verweise"


def test_verzeichnis_enthaelt_teile_und_kapitel(epub):
    """nav.xhtml ist das Verzeichnis fuer EPUB 3, toc.ncx fuer aeltere Geraete.
    Beide muessen dieselben Kapitel kennen."""
    pfad, opf = _opf(epub)
    basis = posixpath.dirname(pfad)
    # Den Pfad NICHT fest verdrahten: wo nav.xhtml liegt, entscheidet der
    # Renderer (es muss bei den Kapiteln liegen, damit seine Verweise ziehen).
    # Der verbindliche Ort steht im Manifest.
    eintraege = {it.get("properties"): it.get("href")
                 for it in opf.find(f"{OPF_NS}manifest")}
    assert "nav" in eintraege, "kein nav im Manifest -- EPUB 3 findet kein Verzeichnis"
    nav = epub.read(posixpath.join(basis, eintraege["nav"])).decode("utf-8")
    ncx = epub.read(posixpath.join(basis, "toc.ncx")).decode("utf-8")

    # Auch der Vorspann muss drinstehen: im gedruckten Buch blaettert man zum
    # Vorwort, im EPUB kommt man nur ueber das Verzeichnis hin.
    for titel in ("Vorwort", "Modul: chart", "Modul: gui"):
        assert titel in nav, f"{titel} fehlt im nav.xhtml"
        assert titel in ncx, f"{titel} fehlt im toc.ncx"

    # nav muss als solches ausgezeichnet sein, sonst findet EPUB 3 es nicht.
    assert 'epub:type="toc"' in nav


def test_stylesheet_paart_jeden_hintergrund_mit_einer_schriftfarbe(epub):
    """Wer `background` setzt, ohne `color` zu setzen, baut eine Falle: im
    Nachtmodus faerbt das Lesegeraet die Schrift hell und laesst den hellen
    Hintergrund stehen -- hell auf hell. Genau so war es beim ersten Bau."""
    css = epub.read("OEBPS/styles/buch.css").decode("utf-8")
    # Regeln ohne Kommentare, je Block betrachtet
    # Regeln, die mit /* ohne-schrift: ... */ gekennzeichnet sind, tragen
    # keinen Text (Farbfelder) -- dort waere eine Schriftfarbe sinnlos.
    css = re.sub(r"/\* ohne-schrift:.*?\*/\s*[^{]+\{[^}]*\}", "", css, flags=re.S)
    ohne_kommentar = re.sub(r"/\*.*?\*/", "", css, flags=re.S)
    for block in re.findall(r"\{([^{}]*)\}", ohne_kommentar):
        setzt_bg = re.search(r"(^|;)\s*background(-color)?\s*:", block)
        if not setzt_bg:
            continue
        if "transparent" in block:
            continue
        assert re.search(r"(^|;)\s*color\s*:", block), \
            f"Hintergrund ohne Schriftfarbe: {{{block.strip()}}}"


def test_nachtmodus_ist_vorgesehen(epub):
    css = epub.read("OEBPS/styles/buch.css").decode("utf-8")
    assert "prefers-color-scheme: dark" in css
    # Ohne color-scheme bleibt die Schrift im Nachtmodus schwarz auf schwarz.
    assert "color-scheme: light dark" in css
