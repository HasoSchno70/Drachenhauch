# Drachenhauch 2026.20

*Die Notizen zu dieser Fassung. Einen Tag und fünf Pull Requests nach
2026.19: ein Abgleich gegen `raylib.h` -- 600 Funktionen, davon waren 251
in Gebrauch -- und dann in fünf Etappen alles, was sich davon für
Drachenhauch lohnt. 57 neue Befehle, fünf neue Beispiele. Was danach noch
fehlt, ist anders gelöst (Ton über Kira, Dateien und Zeichenketten über
Rust, Kollisionen über das physics-Modul) oder gibt es nur für C.*

## Neu in dieser Fassung

### 2D-Formen und Bilder

* **Neue Formen:** Kuchenstücke (`PIE`, `PIEOUTLINE`), Kreise mit Verlauf
  von innen nach außen (`CIRCLE_GRADIENT`), gedrehte Rechtecke (`BOXROT`),
  ein Rechteck mit vier Eckfarben (`GRADIENT4`), regelmäßige Vielecke
  (`NGON`, `NGONOUTLINE`) und gestrichelte Linien (`LINEDASHED`).
* **Rahmen in neun Teilen:** `DRAWIMAGE9` zeichnet ein Bild auf jede Größe,
  ohne die Ecken zu verzerren -- für Knöpfe, Fenster, Sprechblasen.
* **Bilder ohne Umweg über eine Datei:** `IMAGE_FROM_BUFFER` liest ein Bild
  aus Bytes (PNG, JPG, BMP, GIF, QOI, DDS -- etwa direkt aus `HTTP_BYTES`),
  `IMAGE_TO_BUFFER` liefert es als PNG-Bytes.
* **Kurven:** quadratische und kubische Bézierkurven (`BEZIER`, `BEZIER3`),
  Bézier-Ketten und B-Splines durch eine Punktliste (`SPLINE_BEZIER`,
  `SPLINE_BASIS`), dazu Punkte auf einem B-Spline (`CURVE_BSPLINE`,
  `CURVE_BSPLINE2`).
* **Bewegte GIFs lesen:** `IMAGE_LOAD_GIF` liefert alle Einzelbilder,
  `IMAGE_GIF_DELAYS` ihre Dauer -- das Gegenstück zu `IMAGE_SAVE_GIF`.
* **Ein Farbkanal als Graustufenbild** (`IMAGE_CHANNEL`), etwa die Deckkraft
  als Maske.

### 3D

* **Formen:** Kapseln (`CAPSULE`, `CAPSULE_WIRES`), Zylinder und Kegel
  zwischen zwei Punkten (`CYLINDER_EX`, `CYLINDER_WIRES`), Dreiecke und Kreise
  im Raum (`TRIANGLE3D`, `CIRCLE3D`), Quader als Kanten (`BBOX_WIRES`).
* **Neue Grundkörper:** Kegel (`MESH_CONE`), Halbkugel (`MESH_HEMISPHERE`),
  flaches Vieleck (`MESH_POLY`) und ein **Labyrinth aus einem Bild**
  (`MESH_CUBICMAP`: jeder weiße Punkt wird ein Würfel).
* **Hüllquader** eines Modells (`MODEL_BBOX`) -- um es auf den Boden zu
  stellen oder für Kollisionen.
* **Modelle als OBJ speichern** (`MODEL_SAVE`): ein erzeugtes Labyrinth oder
  Gelände in Blender weiterbearbeiten, mit `LOADMODEL` zurückholen.
* **Himmel aus einem gewöhnlichen Bild** (`SKYBOX_IMAGE`): sechs Felder als
  Streifen oder Kreuz, keine `.hdr`-Datei nötig.
* **Mipmaps** (`IMAGE_MIPMAPS`): ein feines Muster auf einer fernen Fläche
  flimmert nicht mehr -- gemessen am Horizont 3856 flimmernde Punkte ohne,
  keiner mit.
* **Billboards** mit Ausschnitt aus einem Sprite-Blatt (`BILLBOARD_PART`)
  oder mit eigener Breite, Höhe und Drehung (`BILLBOARD_EX`).
* **Die Kamera vom Programm aus bewegen** (`CAMERA3D_MOVE`): vor, seitwärts,
  hoch, drehen, neigen -- mit eigener Tastenbelegung, Gamepad oder als
  Kamerafahrt.

### Text

* **Schriften, die bei jeder Größe scharf bleiben** (`LOADFONT_SDF`). Eine
  gewöhnliche Schrift verschwimmt, sobald sie größer gezeichnet wird, als sie
  gebacken wurde; eine SDF-Schrift mit 24 Punkten gebacken ist auch bei 160
  Punkten scharf.
* **Fragen an eine Schrift:** hat sie ein Zeichen (`FONT_HAS_GLYPH`), wie
  breit ist es (`FONT_GLYPH_WIDTH`).
* **Eine Schrift als Bild** (`FONT_TO_IMAGE`) im Format von
  `LOADFONT_IMAGE` -- aus einer Vektorschrift wird eine Pixelschrift zum
  Nachbearbeiten.
* **Text mit einer Schrift in ein Bild:** `IMAGE_TEXT` erzeugt ein Bild in
  der Größe des Textes, `IMAGE_DRAW_TEXT` nimmt jetzt eine Schrift -- für
  Schilder, Etiketten auf Texturen, beschriftete Billboards.
* **Bildrate anzeigen** in einer Zeile (`DRAWFPS`).
* **Neue Textbefehle:** Schreibweisen für Bezeichner (`CAMEL$`, `SNAKE$`,
  `PASCAL$`; aus `HTTPServer` wird `http_server`), Text zwischen zwei
  Markierungen holen oder ersetzen (`BETWEEN$`, `REPLACE_BETWEEN$`), jeden
  Leerraum entfernen (`NOSPACES$`).

### Fenster

* `WINDOW_WAIT_EVENTS` lässt `FLIP` auf eine Eingabe warten -- ein Werkzeug,
  das nur auf Klicks reagiert, braucht kaum noch Rechenzeit (gemessen 136
  statt 17 ms je Bild ohne Eingabe).
* `WINDOW_RENDER_WIDTH`/`_HEIGHT` nennen die echte Pixelgröße auf einem
  HiDPI-Bildschirm, `MONITOR_PHYSICAL_WIDTH`/`_HEIGHT` die Größe des
  Bildschirms in Millimetern.
* `MOUSE_OFFSET` und `MOUSE_SCALE` rechnen die Mauslage um -- wenn ein
  kleines Bild vergrößert im Fenster steht.

### Fünf neue Beispiele

`201_formen_und_rahmen`, `202_3d_formen_und_kurven`,
`203_labyrinth_und_himmel`, `204_scharfe_schrift` und
`205_rundgang_mit_schildern` (ein Gang durch einen Hof mit eigener
Steuerung, Schildern mit eingebranntem Text und einem Feuer aus einem GIF).
Die Schrift dafür (Roboto Medium, Apache 2.0) liegt unter
`examples/assets/`.

## Behoben

* **`LOADFONT_IMAGE` brachte die Zeichenlisten durcheinander:** jede danach
  geladene TTF-Schrift zeigte auf die Liste ihres Vorgängers, und ihre
  eigenen Umlaute kamen aus der Ausweich-Schrift.
* raylibs eigener OBJ-Export hätte eine Textur nach dem Wiederladen auf den
  Kopf gestellt (er kippt die Texturkoordinaten nicht, raylibs Lader schon)
  -- `MODEL_SAVE` schreibt darum selbst.
* Text in ein Bild wird selbst eingemischt wie bei `IMAGE_DRAW_IMAGE`;
  raylibs `ImageDraw` rechnet an den Zeichenrändern über halbdurchsichtigem
  Grund andere Farben.

## Unter der Haube

**Zahlen.** Die Befehlsreferenz wächst von 2029 auf **2086** Einträge, es
bleiben 48 Module, die Beispiele wachsen von 216 auf **221**.
**4710 Fälle in 361 Prüfsammlungen** (vorher 4673 in 356) und
**484 Rust-Testfunktionen** (vorher 476).

* SDF-Schriften zeichnen mit einem eigenen Shader, der nur um ihre
  Textstücke gelegt wird; Zeichen, die sie nicht hat, kommen wie immer aus
  der Ausweich-Schrift.
* Bilder mit Mipmaps bekommen nach jedem `IMAGE_DRAW_*` neue kleinere
  Stufen, sonst zeigte die Ferne den alten Inhalt.
* Jede Etappe ist gegengeprüft: 45 gezielt verfälschte Fassungen der
  Laufzeit (7, 8, 9, 11 und 10 je Etappe), jede fällt in ihrem Prüffall.

## Was offen bleibt

* Mit Fokus kostet das Code-Feld bei 100 000 Zeilen je Bild noch rund
  17 ms.
* Auf einem echten Mac ist das Paket weiter nicht ausprobiert und nicht
  beglaubigt.
* Der Installer ist nicht signiert; SmartScreen meldet sich beim ersten Start.
