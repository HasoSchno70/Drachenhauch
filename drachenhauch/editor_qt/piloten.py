"""Die Werkzeuge, die in Drachenhauch selbst geschrieben sind.

Vier der Qt-Begleit-Editoren gibt es ein zweites Mal -- als Drachenhauch-
Programm unter `examples/`. Sie sind nicht der Ersatz fuer die Qt-Fassungen,
sondern die Antwort auf die Frage, ob die Sprache dafuer taugt: gemessen
statt geschaetzt. Und weil sie Drachenhauch sind, lassen sie sich lesen und
aendern -- deshalb steht neben dem Starten das Oeffnen des Quelltexts.

**Einzige Quelle** fuer die Menue-Eintraege der IDE und die Tests. Wer einen
weiteren Piloten schreibt, ergaenzt hier eine Zeile.

Die Zeilenzahlen sind gemessen, nicht geschaetzt (`qt` = die Qt-Fassung,
`dh` = die Drachenhauch-Fassung). **Der Faktor ist nicht uebertragbar** --
er haengt vor allem daran, wie viel die Drachenhauch-Fassung gar nicht erst
hat: beim Sprite-Editor sind allein die benennbaren nicht portierten
Bloecke 1595 der 7379 Qt-Zeilen. Siehe CLAUDE.md, Abschnitt "Editoren in
Drachenhauch selbst".
"""
from __future__ import annotations

from pathlib import Path

PILOTEN: list[dict] = [
    {"datei": "183_sfx_generator.dh", "titel": "SFX-Generator",
     "qt": 522, "dh": 484,
     "kurz": "16 Regler, Wellenform-Anzeige, WAV- und GB-Code-Ausgabe, Einstellungen sichern/laden"},
    {"datei": "185_partikel_editor.dh", "titel": "Partikel-Editor",
     "qt": 802, "dh": 468,
     "kurz": "17 Regler, echte Vorschau, GB-Code-Ausgabe, Einstellungen sichern/laden"},
    {"datei": "187_tilemap_editor.dh", "titel": "Tilemap-Editor",
     "qt": 2428, "dh": 1485,
     "kurz": "sechs Werkzeuge, Auswahl mit Zwischenablage, Ebenen, Objekt-Ebenen, mehrere Tilesets, Kachel-Eigenschaften, Tiled-JSON, GB-Code-Ausgabe"},
    {"datei": "189_sprite_editor.dh", "titel": "Sprite-Editor",
     "qt": 7379, "dh": 2604,
     "kurz": "zwoelf Werkzeuge (mit Lasso, Zauberstab und Verschieben), Ebenen, benannte Einzelbilder, Kachel-Ansicht, Statistik, eigenes Format mit Ebenen, Streifen, Atlas, bewegtes GIF, .gpl-Paletten, Zuschneiden, Animationsbereiche, GB-Code und .dhanim"},
]


def pfad(project_root: Path, eintrag: dict) -> Path:
    """Wo der Pilot liegt -- im Repo wie in der installierten Fassung.

    Der Installer legt `examples/` unter `%PUBLIC%\\Documents\\Drachenhauch`
    ab, und genau das ist dort der `project_root` der eingefrorenen App
    (siehe `dhrun._project_root`). Es braucht also keinen zweiten Weg.
    """
    return Path(project_root) / "examples" / eintrag["datei"]


def beschreibung(eintrag: dict) -> str:
    """Ein Satz fuer den Tooltip -- mit der gemessenen Zahl statt einem
    Werbeversprechen."""
    return (f"{eintrag['kurz']} -- in Drachenhauch geschrieben "
            f"({eintrag['dh']} Zeilen gegen {eintrag['qt']} der Qt-Fassung)")
