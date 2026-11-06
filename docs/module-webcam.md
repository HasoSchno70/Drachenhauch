# Modul `webcam`

Bilder einer Kamera in ein `IMAGE` — zum Anzeigen, Verarbeiten
(`GETPIXEL`, `IMAGE_*`), Speichern oder als Textur. Damit gehen ein Spiegel,
ein Bewegungsmelder, eine Fotobox, eine Farberkennung für einen Roboter oder
eine Zeitrafferkamera.

```basic
IMPORT "webcam"
```

Je System ein eigener Weg, ohne zusätzliche Bibliothek beim Nutzer: unter
**Linux** Video4Linux2 (`/dev/videoN`), unter **Windows** Media Foundation,
unter **macOS** AVFoundation. Jede Kamera läuft in einem eigenen Faden; das
Programm holt das jüngste Bild ab, wenn es fragt — ein langsames Programm
verpasst Bilder, es staut sie nicht.

## Erste Schritte

```basic
IMPORT "webcam"
DIM cam AS WEBCAM
cam = WEBCAM_OPEN()
SCREEN(WEBCAM_WIDTH(cam), WEBCAM_HEIGHT(cam), "Spiegel")
WHILE NOT QUITREQUESTED()
    WEBCAM_UPDATE(cam)
    CLS(&H000000)
    DRAWIMAGE(WEBCAM_IMAGE(cam), 0, 0)
    FLIP()
WEND
WEBCAM_CLOSE(cam)
```

## Befehle

| Befehl | Was es tut |
|---|---|
| `WEBCAM_DEVICES()` → ARRAY OF STRING | die Namen aller Kameras; leer, wenn der Rechner keine hat |
| `WEBCAM_OPEN([geraet$ [, breite, hoehe]])` → WEBCAM | eine Kamera öffnen — ohne Namen die erste. Ein Name passt genau oder als Teiltext ohne Groß/klein; ein unbekannter ist ein Fehler, der die vorhandenen nennt. `breite`/`hoehe` sind ein Wunsch, dem die Kamera so nah wie möglich kommt (0 = was sie vorschlägt) |
| `WEBCAM_CLOSE(kamera)` | die Kamera schließen und ihr Bild freigeben |
| `WEBCAM_UPDATE(kamera)` → BOOLEAN | ist seit der letzten Frage ein neues Bild gekommen? Dann steht es jetzt im Bild der Kamera |
| `WEBCAM_IMAGE(kamera)` → IMAGE | das Bild der Kamera mit dem jüngsten Bild (holt es selbst ab). Es ist immer dasselbe `IMAGE` — außer die Kamera wechselt die Größe, dann gibt es ein neues |
| `WEBCAM_WIDTH(kamera)` / `WEBCAM_HEIGHT(kamera)` → INTEGER | die Größe, die die Kamera wirklich liefert |
| `WEBCAM_FRAMES(kamera)` → INTEGER | wie viele Bilder seit dem Öffnen kamen — zeigt, ob überhaupt etwas kommt, und die Bildrate |
| `WEBCAM_NAME$(kamera)` → STRING | der Name der offenen Kamera |
| `WEBCAM_ERROR$(kamera)` → STRING | was die Kamera nach dem Öffnen gemeldet hat (abgezogen, belegt) — sonst `""` |

Das Bild braucht die Grafik: ohne `SCREEN` legt `WEBCAM_OPEN` ein
verstecktes Fenster an, wie `LOADIMAGE`.

## Bewegung erkennen

Zwei Bilder Punkt für Punkt vergleichen — hier jeder achte Punkt, das reicht
und ist schnell:

```basic
IMPORT "webcam"
IMPORT "imgfx"
DIM cam AS WEBCAM
cam = WEBCAM_OPEN("", 320, 240)
DIM vorher AS IMAGE
vorher = IMAGE_COPY(WEBCAM_IMAGE(cam))
WHILE TRUE
    IF WEBCAM_UPDATE(cam) THEN
        DIM jetzt AS IMAGE : jetzt = WEBCAM_IMAGE(cam)
        DIM anders AS INTEGER : anders = 0
        DIM x AS INTEGER
        DIM y AS INTEGER
        FOR y = 0 TO WEBCAM_HEIGHT(cam) - 1 STEP 8
            FOR x = 0 TO WEBCAM_WIDTH(cam) - 1 STEP 8
                IF ABS((GETPIXEL(jetzt, x, y) BAND 255) - (GETPIXEL(vorher, x, y) BAND 255)) > 40 THEN anders = anders + 1
            NEXT
        NEXT
        IF anders > 50 THEN PRINT "Bewegung! "; anders
        IMAGE_FREE(vorher)
        vorher = IMAGE_COPY(jetzt)
    END IF
    SLEEP(30)
WEND
```

Ein Foto sichern: `IMAGE_SAVE(WEBCAM_IMAGE(cam), "foto.png")`.

## Formate

Die Kamera liefert, was sie kann; dhrt liest **YUYV**, **MJPEG**, **RGB24/
BGR24** und **BGRX** (Linux) bzw. lässt das System nach RGB umrechnen
(Windows, macOS). Bei großen Bildern (über 640 × 480) wird unter Linux zuerst
MJPEG verlangt — viele Kameras schaffen YUYV in 1280 × 720 über USB 2 nur
mit wenigen Bildern je Sekunde. MJPEG-Bilder, denen die Huffman-Tafel fehlt
(so schicken es die meisten Kameras), bekommen die Standardtafel.

## Berechtigungen

- **Windows**: *Einstellungen → Datenschutz und Sicherheit → Kamera* — der
  Zugriff für Desktop-Apps muss an sein, sonst lässt sich die Kamera nicht
  öffnen.
- **macOS** fragt beim ersten `WEBCAM_OPEN`. Die Drachenhauch-App bringt die
  Begründung und die Berechtigung mit; im Terminal gilt die Erlaubnis des
  Terminals. Ist der Zugriff verweigert, sagt die Meldung, wo man ihn
  einschaltet.
- **Linux**: der Benutzer muss in der Gruppe `video` sein (auf Raspberry Pi
  OS und den üblichen Systemen von Anfang an). Eine Kamera, die schon ein
  anderes Programm benutzt, meldet „belegt".

## Prüfen ohne Kamera

`DHRT_WEBCAM=test` ersetzt alle Kameras durch eine **Testbild**-Kamera: acht
Farbbalken (weiß, gelb, türkis, grün, violett, rot, blau, schwarz), darin
ein schwarzes Quadrat, das je Bild 8 Punkte nach rechts wandert, 30 Bilder
je Sekunde, 320 × 240 oder die gewünschte Größe. `tests/pruef/webcam.dhtest`
prüft damit alles außer dem Weg des Systems; unter Linux liest es zusätzlich
`vivid`, den Testtreiber des Kernels — über denselben Weg wie eine echte
Kamera.

## Grenzen

- Ungeprüft mit einer echten Kamera unter Windows und macOS — der Weg ist
  übersetzt, auf den Prüfrechnern gibt es keine Kamera.
- Ein Raspberry-Pi-Kameramodul (CSI) geht unter Raspberry Pi OS über
  `libcamera`, nicht über V4L2 — mit einer USB-Kamera geht es. Das
  Hilfsprogramm `libcamerify` von libcamera kann das Modul als V4L2-Kamera
  zeigen (ungeprüft).
- Kein Ton von der Kamera (dafür [Mikrofon](module-mikrofon.md)), keine
  Einstellungen wie Belichtung oder Fokus.
- Im Browser gibt es noch keine Kamera.
