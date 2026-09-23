# Modul `video`

Videos abspielen: MP4 mit H.264, das Format, das praktisch jedes Programm und
jedes Telefon schreibt. Jedes Bild landet in einem gewöhnlichen `IMAGE` --
gezeichnet wird es wie jedes andere Bild, skaliert, auf einem Layer, in einem
Render-Target oder als Textur. Die IDE zeigt damit ihren Vorspann.

```basic
IMPORT "video"
```

Den Container liest die Laufzeit selbst, die Bilder dekodiert **openh264**
(Ciscos freier H.264-Decoder, beim Bau der Laufzeit aus dem Quelltext
mitübersetzt). Beim Nutzer muss nichts installiert sein.

## Befehle

| Funktion | Wirkung |
|---|---|
| `VIDEO_LOAD(pfad$)` | MP4-Datei öffnen, liefert ein `VIDEO`; das erste Bild steht gleich im Bild |
| `VIDEO_PLAY(v)` | abspielen; ein zu Ende gelaufenes Video beginnt von vorn |
| `VIDEO_PAUSE(v)` | anhalten, das Bild bleibt stehen |
| `VIDEO_STOP(v)` | anhalten und an den Anfang |
| `VIDEO_SEEK(v, sekunden)` | an eine Stelle springen (auf die Dauer begrenzt) |
| `VIDEO_LOOP(v, an)` | am Ende von vorn beginnen statt stehenzubleiben |
| `VIDEO_UPDATE(v)` | die Uhr des Videos um `DELTA()` weiterstellen, ohne zu zeichnen |
| `VIDEO_DRAW(v, x, y [, breite, hoehe])` | weiterstellen und das aktuelle Bild zeichnen |
| `VIDEO_PLAYING(v)` | spielt es gerade? |
| `VIDEO_DONE(v)` | ist es (ohne Schleife) am Ende angekommen? |
| `VIDEO_POSITION(v)` | Stelle in Sekunden |
| `VIDEO_DURATION(v)` | Dauer in Sekunden |
| `VIDEO_WIDTH(v)` | Breite in Punkten |
| `VIDEO_HEIGHT(v)` | Höhe in Punkten |
| `VIDEO_FRAMES(v)` | Zahl der Bilder |
| `VIDEO_FRAME(v)` | Nummer des Bildes, das gerade im `IMAGE` steht (ab 0) |
| `VIDEO_FPS(v)` | Bilder je Sekunde |
| `VIDEO_IMAGE(v)` | das `IMAGE`, in das jedes neue Bild geschrieben wird |
| `VIDEO_FREE(v)` | Video und sein Bild freigeben |

## Beispiel

```basic
IMPORT "video"
SCREEN(1280, 720, "Vorspann", 1)
DIM v AS VIDEO : v = VIDEO_LOAD("intro.mp4")
VIDEO_PLAY(v)
WHILE NOT QUITREQUESTED() AND NOT VIDEO_DONE(v)
    CLS(0)
    VIDEO_DRAW(v, 0, 0, SCREENWIDTH(), SCREENHEIGHT())
    FLIP()
WEND
VIDEO_FREE(v)
```

## Wie die Zeit läuft

Ein Video hat seine eigene Uhr, und die geht mit `DELTA()` -- also mit der
echten Zeit, nicht mit der Bildrate des Programms. Läuft das Programm mit 60
Bildern je Sekunde und das Video mit 24, zeigt es jedes Videobild zwei- oder
dreimal; ruckelt das Programm, überspringt das Video Bilder statt langsamer zu
werden. Die Uhr läuft ab dem Bild **nach** `VIDEO_PLAY`.

`VIDEO_DRAW` stellt die Uhr selbst weiter; wer das Bild anders benutzt (über
`VIDEO_IMAGE` als Textur, in einem Shader), ruft stattdessen je Bild
`VIDEO_UPDATE`. Beides im selben Bild zählt nur einmal.

Ohne Fenster (Tests mit `DHRT_FRAMES`) ist `DELTA()` fest 1/60 s -- ein Video
steht dort nach n Bildern genau bei n/60 Sekunden, jeder Lauf gleich.

Am Ende bleibt das **letzte Bild** stehen und `VIDEO_DONE` wird wahr; ein
Vorspann, der danach schwarz würde, blitzte beim Übergang.

## Grenzen

- **Nur H.264 in MP4** (`.mp4`, `.m4v`, `.mov` mit H.264). Andere Formate
  (H.265/HEVC, VP9, AV1) lehnt `VIDEO_LOAD` mit einer Meldung ab, die das
  Format nennt.
- **Kein Ton.** Eine Tonspur wird überlesen; wer Musik dazu will, spielt sie
  mit `AUDIO_MUSIC_PLAY` und startet beide im selben Bild.
- **Zurückspulen dekodiert vom Anfang.** H.264 kann nur an Schlüsselbildern
  neu ansetzen; `VIDEO_SEEK` nach hinten und die Schleife kosten darum bei
  langen Videos einen Moment. Vorwärts geht es Bild für Bild weiter.
- Nicht im Web-Bau (`dhrt` als WebAssembly) und nicht in einem Bau ohne das
  Feature `video` -- dort sind die Befehle unbekannt.
- H.264 ist patentbelastet. Cisco übernimmt die Lizenzgebühren nur für die
  fertigen openh264-Bibliotheken, die Cisco selbst verteilt; diese Laufzeit
  übersetzt openh264 aus dem Quelltext. Wer ein Programm mit Videos verkauft
  oder weit verteilt, sollte das vorher klären.
