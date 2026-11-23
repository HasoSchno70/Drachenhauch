# Drachenhauch 2026.53

*Die Notizen zu dieser Fassung. Sie ist die Fassung **der Bücher**: Zwei
neue Bücher sind fertig -- „Kleine Wunder“ lehrt die Sprache an Programmen,
die man gern ansieht, „Fenster, Formulare, Daten“ baut eine Anwendung erst
von Hand und dann im Studio. Beim Schreiben fielen über siebzig Stellen auf,
an denen Laufzeit, Studio oder Meldungen still falsch lagen; sie sind
behoben. Dazu kommt raylib 6.2. Was bestehende Programme bemerken können,
steht gleich zuerst.*

## Was sich für bestehende Programme ändert

**Übersetzen**

* **Eine Zuweisung an eine `CONST` ist ein Fehler beim Übersetzen**, auch in
  Code, der nie läuft (vorher erst zur Laufzeit, `--check` schwieg).
  Verdeckt ein Parameter, eine lokale Variable oder ein Feld die Konstante,
  bleibt die Zuweisung erlaubt.
* **Ein ENUM-Wert, den es nicht gibt** (`Stoff.SNAD`), ist ein Fehler beim
  Übersetzen, mit Vorschlag.
* **Eine `COROUTINE`-Variable nimmt nur noch Koroutinen.**
* **Neue Warnungen:** `a BAND b + c` ohne Klammern (gelesen wird
  `a BAND (b + c)`; ebenso `BOR`, `BXOR`, `SHL`, `SHR`), eine Koroutine als
  bloße Anweisung (sie liefe nie), `TRY … END TRY` ohne `CATCH` und
  `FINALLY` (es verschluckt den Fehler, wie bisher).

**Grafik**

* **raylib 6.2** zeichnet an einigen Stellen anders, wo 6.0 falsch lag:
  `NGONOUTLINE` mit Dicke ist so dick wie im Handbuch (vorher etwa ein
  Viertel), Kreise in Bildern sind rechts eine Spalte breiter, die Pose
  einer GLTF-Animation ist minimal anders, und der Bildtakt wartet je Bild
  rund 0,8 ms aktiv. Gamepads bekommen raylibs eigene Zuordnungen.
* **`SHADER_LOAD` mit einer fehlenden Datei ist ein Fehler** (mit
  Vorschlag), statt -1 zu liefern. Ein unbekanntes Uniform in `SHADER_SET…`
  warnt einmal.
* **`BLEND_MODE("subtract")`** zieht die Farbe vom Hintergrund ab und lässt
  die Deckkraft stehen (vorher umgekehrt, samt Deckkraft).
* **`RENDERTARGET_BEGIN` ohne `RENDERTARGET_END` vor `FLIP`** hält mit
  Meldung an, statt schwarz weiterzulaufen.
* **Punktlichter** sind auf Entfernung heller (derselbe Abfall wie beim
  Instancing): in 4 Einheiten etwa halb so hell, in 10 ein Fünftel.
  `MODEL_INSTANCED` verschwindet im Nebel wie `MODEL`.
* **`MESH_TORUS` und `MESH_KNOT` sind so groß wie angegeben** -- Ringe und
  Knoten erscheinen größer als bisher.
* **`LOADIMAGE` lädt eine neu geschriebene Datei neu** (neues Handle, das
  alte bleibt gültig), statt das alte Bild aus dem Zwischenspeicher zu
  liefern.

**Ton**

* **MOD- und XM-Musik ist gut 5 dB leiser**: Der Spieler rechnet jetzt die
  Lautstärke ein, die das Modul selbst vorgibt, und übersteuert nicht mehr.
* **`AUDIO_MOD_REMOVE`** stellt den festen Wert des Parameters wieder her,
  statt den letzten Wert des Modulators stehen zu lassen.
* **`AUDIO_FFT` und `MIC_FFT` klingen nach der Uhr ab.** Wer öfter als
  sechzigmal je Sekunde fragt, sieht die Bänder langsamer fallen.

**Physik**

* **Ein entfernter `physics2d`-Körper meldet sich** bei jeder Abfrage,
  statt still (0, 0) zu liefern; `PHYS2D_BODY_EXISTS` fragt vorher.

**Oberfläche**

* **`GUI_VALIDATE_LIVE`** prüft beim Verlassen eines Felds nur, was sich
  seit dem Fokus geändert hat; ein leeres Pflichtfeld meldet
  `GUI_VALIDATE`.
* **Ein leeres Zahlenfeld** bindet `NULL` statt eines leeren Texts.
* **Zeilenfarbe und Zebrastreifen einer Tabelle** reichen bis zum Rand.
* **`GUI_DIALOG`** ist höchstens 640 breit und bricht lange Zeilen um.
* **Die leise Schrift (`muted_fg`) und die gewählte Tabellenzeile** haben in
  jedem Thema mindestens 4,5:1 Kontrast; einige Farben sind dafür kräftiger.

## Neu in dieser Fassung

### Zwei Bücher

* **[Kleine Wunder](../../buecher/wunder/README.md)** -- die Sprache an
  Programmen, die man gern ansieht: Sternenflug, Feuerwerk, Fraktale, ein
  Schwarm, ein Theremin, ein Flug über eine Landschaft, eine Demo aus
  Namensräumen. 27 Kapitel in sieben Teilen, jedes mit dem Grund für seine
  Entscheidungen, Übungen und Lösungen.
* **[Fenster, Formulare, Daten](../../buecher/anwendungen/README.md)** --
  eine Mediathek, erst von Hand mit dem gui-Modul, dann im Studio:
  Datenbank, Bindung, Abfragen, ein Abruf aus dem Netz, Berichte,
  Diagramme, Excel, Barrierefreiheit, zwei Sprachen und ein Installer.
* Beide liegen als `.docx` und `.epub` dem Installer und dem Release bei.
  Jedes Programm, jede abgedruckte Ausgabe und jede Meldung darin prüft
  eine Sammlung gegen dhrt; die Bilder entstehen aus den Programmen.

### Laufzeit

* **`DIR_USER_DATA$(app$)`** -- der Ordner für die Daten eines Programms
  beim Benutzer (APPDATA, Application Support oder XDG), angelegt beim
  ersten Aufruf.

```basic
DIM ordner AS STRING : ordner = DIR_USER_DATA$("Mediathek")
PRINT ordner
```

* **gui:** `GUI_TEXT_WIDTH`, `GUI_TABLE_ROW_RECT`,
  `GUI_TABLE_GET_CELL_COLOR`, `GUI_TABLE_GET_ROW_COLOR`,
  `GUI_MENU_ENABLED`; `GUI_STATE$` nennt Tabellen samt gewählter Zeile,
  Kacheln und die letzte Ansage an den Bildschirmleser.
* **physics2d:** Gelenke mit `PHYS2D_ADD_HINGE`, `PHYS2D_ADD_ROPE`,
  `PHYS2D_REMOVE_JOINT`, dazu `PHYS2D_BODY_EXISTS`.
* **tween:** `TWEEN_NEW` nimmt eine eigene Funktion als Kurve.
* **`v.x`, `v.y`** lesen die Teile von `VEC2`, `VEC3`, `VEC4` und `QUAT`.
* Ein FUNCREF in einem Feld lässt sich wie eine Methode rufen; eine
  Comprehension passt in ein typisiertes Feld und in die lesenden
  Feld-Befehle.
* **Meldungen, die mehr sagen:** ein vertippter Argumentname („Meintest du
  hoehe?“), ein Fehler in einer Vorgabe nennt die Kopfzeile, „database is
  locked“ sagt, worauf gewartet wird, ein Apostroph in eingeklebtem SQL wird
  erkannt, ein verletzter Fremdschlüssel nennt Tabelle und Spalte, ein
  HTTP-Fehler sagt, ob das Netz fehlt oder die Zeit abgelaufen ist,
  `LOADIMAGE` und `OPENFILE` nennen die Datei.
* **Der Debugger** (`dhrt debug`, in IDE und Studio) zeigt `Self` mit den
  eigenen Feldern zuerst und hält auf einer Zeile mit Aufruf nicht mehr
  zweimal.

### Studio

* **Projekt → Sprachtafel ergänzen …** sammelt alle Texte des Projekts und
  hängt die fehlenden an die Tafel.
* **„Im Benutzerordner“** am Datenbank-Element: Das gebaute Programm legt
  seine Datenbank in den Datenordner des Benutzers und darf so unter
  „Programme“ liegen.
* **Berichte:** ein Strichcode (EAN-13), Summen über den ganzen Bericht,
  A6 legt auf A6 aus.
* **Aussehen:** Stildateien im Projekt (`StilLaden`) und Klassen im
  Inspektor; `GUI_SCALE` in `App.Geoeffnet` vergrößert das Programmfenster.

### Gemeinschaft

* **`dhrt paket`** holt mit `#ordner` nur einen Ordner einer Quelle -- so
  liegen mehrere Pakete in einem Repository.
* **Drei Bibliotheken** in `bibliotheken/`: `highscores`, `menu` und
  `dialogbox`.
* **[Galerie](../galerie.md)** und **Discussions** mit Formularen für
  Fragen, eigene Programme und Ideen.

## Unter der Haube

**Zahlen.** 2672 Befehle (2662) in 57 Modulen, 237 Beispiele auf der
obersten Ebene von `examples/`, **6285 Fälle in 525 Prüfsammlungen** (6120
in 518) und 741 Rust-Testfunktionen (732).

* **raylib 6.2** über die Bindung `sola-raylib` statt `raylib-rs` (das bei
  6.0 steht). Wo 6.2 anders zeichnet und 6.0 recht hatte (Linien in
  Bildern, abgerundete Umrisse, Kugeln), ist das alte Bild zurückgeholt.
* **Maschinencode:** Eine Schleife, die fast jede Runde an die VM abgibt
  (etwa mit `VEC2`), bleibt in der VM -- vorher war sie mit Maschinencode
  bis zu siebenmal langsamer als ohne.
* **Abhängigkeiten** auf dem Stand vom 21. November, darunter rusqlite
  0.43, cranelift 0.138 und xmrs 0.15.
* Die Pakete für macOS und Linux bringen jetzt auch `dhrt-mini` mit.

## Was offen bleibt

* **Englische Fassungen** der beiden neuen Bücher gibt es noch nicht.
* **`physics3d`** liefert für einen entfernten Körper weiter still (0, 0).
* **ureq 3** und **zbus 6** sind nicht gehoben (neue Schnittstelle bzw.
  zwei zbus im Baum).
* Das macOS-Paket ist nicht beglaubigt, der Windows-Installer nicht signiert
  (SmartScreen warnt beim Herunterladen).
