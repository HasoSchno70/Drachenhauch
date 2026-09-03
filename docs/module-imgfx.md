# Modul `imgfx`

Bild-Effekte: skalieren, rotieren, spiegeln, einfärben. Die Filter geben ein **neues** IMAGE zurück, das Original bleibt erhalten — Verkettung ist sicher. Daneben gibt es Befehle, die **in** ein Bild malen (`IMAGE_DRAW_*`), und seit 2026-08-31 solche, mit denen ein Programm Bilder **herstellt** statt sie nur anzuzeigen: [anlegen, zusammensetzen und speichern](#bilder-anlegen-zusammensetzen-und-speichern).

```basic
IMPORT "imgfx"
```

## Übersicht

| Funktion | Rückgabe |
|---|---|
| `IMAGE_SCALE(img, w, h)` | IMAGE (neu, in `w × h`) — **bilinear geglättet** |
| `IMAGE_SCALE_NN(img, w, h)` | IMAGE (neu, in `w × h`) — **ohne Interpolation** (Nearest-Neighbour), für Pixelgrafik |
| `IMAGE_ROTATE(img, grad)` | IMAGE (neu, Bounding-Box wächst ggf.) — **tastet neu ab**, siehe unten |
| `IMAGE_ROTATE_CW(img)` | IMAGE (neu, um 90° im Uhrzeigersinn) — **exakt**, `w × h` → `h × w` |
| `IMAGE_ROTATE_CCW(img)` | IMAGE (neu, um 90° gegen den Uhrzeigersinn) — **exakt** |
| `IMAGE_FLIP(img, flipX, flipY)` | IMAGE (neu, gespiegelt) |
| `IMAGE_TINT(img, color)` | IMAGE (neu, RGB-multipliziert) |
| `IMAGE_COPY(img)` | IMAGE (tiefer Klon) |

### Weitere Filter (geben ebenfalls ein neues IMAGE zurück)

| Funktion | Rückgabe / Wirkung |
|---|---|
| `IMAGE_CROP(img, x, y, w, h)` | IMAGE (neu, Ausschnitt `w × h` ab `x,y`) |
| `IMAGE_RESIZE_CANVAS(img, w, h, offx, offy[, fill])` | IMAGE (neu): Bild auf Leinwand `w × h` ab Offset `offx,offy`; freie Fläche mit `fill` (Default schwarz) |
| `IMAGE_BLUR(img, radius)` | IMAGE (neu, Gauß-Weichzeichner, `radius` in px) |
| `IMAGE_BRIGHTNESS(img, n)` | IMAGE (neu): Helligkeit, `n` = `-255..255` |
| `IMAGE_CONTRAST(img, n)` | IMAGE (neu): Kontrast, `n` = `-100..100` |
| `IMAGE_GRAYSCALE(img)` | IMAGE (neu, in Graustufen) |
| `IMAGE_INVERT(img)` | IMAGE (neu, Farben invertiert) |
| `IMAGE_REPLACE_COLOR(img, from, to)` | IMAGE (neu): tauscht **exakt** die Farbe `from` gegen `to` |

### In ein Image zeichnen (MUTIEREND — verändert das übergebene IMAGE)

Anders als die Filter geben diese **kein** neues Handle zurück, sondern malen direkt in das Image. Ideal, um zur Ladezeit eine Grafik zusammenzubauen (z. B. leere Leinwand via `IMAGE_NEW(w, h)`, dann bemalen).

**Jeder dieser Aufrufe lädt die ganze Textur neu hoch** — gemessen **1,16 µs
pro Aufruf**. Was das heißt, hängt davon ab, wie viele es sind:

| | |
|---|---|
| ein Pinselstrich, ein paar Punkte je Bild | belanglos |
| eine Fläche von 64×64 Punkt für Punkt | 4,7 ms — geht gerade noch in ein Bild |
| dasselbe bei 256×256 | rund 75 ms — ein sichtbarer Ruckler |

Für ein Malprogramm ist das der Normalfall und in Ordnung; wer *ganze
Flächen* setzt, nimmt besser `IMAGE_CLEAR`, `IMAGE_DRAW_RECT` mit echter
Größe oder `IMAGE_DRAW_IMAGE` statt einer Schleife über Einzelpunkte.

**Innerhalb EINES Bildes sieht man nur den Endstand.** Wer dasselbe IMAGE
zweimal im selben Frame zeichnet und dazwischen ändert, bekommt **beide Male
den neuen Inhalt** — ein Vorher/Nachher nebeneinander geht so nicht:

```basic
IMPORT "imgfx"
DIM b AS IMAGE : b = IMAGE_NEW(16, 16, &HFF0000)
DRAWIMAGE(b, 10, 10)                      ' zeigt GRUEN, nicht rot
IMAGE_DRAW_RECT(b, 0, 0, 16, 16, &H00FF00)
DRAWIMAGE(b, 40, 10)                      ' zeigt gruen
```

Der Grund ist das Zeichenmodell: ein Zeichenbefehl merkt sich nur, *welche*
Textur er benutzt, und nachgesehen wird erst beim `FLIP`. Wer zwei Stände
nebeneinander braucht, braucht zwei Bilder (`IMAGE_COPY`).



| Funktion | Wirkung |
|---|---|
| `IMAGE_DRAW_LINE(img, x1, y1, x2, y2, color)` | Linie ins Image |
| `IMAGE_DRAW_CIRCLE(img, cx, cy, r, color)` | gefüllter Kreis ins Image |
| `IMAGE_DRAW_RECT(img, x, y, w, h, color)` | gefülltes Rechteck ins Image |
| `IMAGE_DRAW_TEXT(img, x, y, text$, size, color)` | Text (Standard-Font) ins Image |
| `IMAGE_ALPHA_MASK(bild, maske)` | IMAGE — Deckkraft aus einem zweiten Bild übernehmen (weiche Ränder) |
| `IMAGE_ALPHA_CROP(bild, schwelle)` | IMAGE — durchsichtigen Rand wegschneiden; `schwelle` sagt, ab welcher Deckkraft ein Pixel zählt |
| `IMAGE_ALPHA_PREMULTIPLY(bild)` | IMAGE — Farbe mit der Deckkraft vorab verrechnen; verhindert dunkle Säume beim Skalieren |
| `IMAGE_DITHER(bild, r, g, b, a)` | IMAGE — Farbtiefe senken und den Fehler verteilen. **Nur 5,6,5,0 / 5,5,5,1 / 4,4,4,4** — alles andere wird abgelehnt, weil raylib sonst ein unbrauchbares Format liefert |
| `IMAGE_PALETTE(bild, max)` | ARRAY OF INTEGER — die häufigsten Farben des Bildes |

## Bilder anlegen, zusammensetzen und speichern

Diese fuenf machen aus einem IMAGE etwas, das ein Programm nicht nur anzeigen,
sondern auch **herstellen** kann. Vorher war es eine Einbahnstrasse:
hineinzeichnen ging, aber ein durchsichtiges Bild liess sich nicht anlegen,
kein Bild in ein anderes setzen, nichts wegradieren, die Deckkraft nicht lesen
und ueberhaupt nichts speichern.

| Funktion | Wirkung |
|---|---|
| `IMAGE_NEW(breite, hoehe [, farbe])` | IMAGE — ein neues Bild; **ohne Farbe vollstaendig durchsichtig** |
| `IMAGE_CLEAR(bild [, x, y, b, h])` | einen Bereich vollstaendig durchsichtig machen — der Radierer; ohne Rechteck das ganze Bild |
| `IMAGE_DRAW_IMAGE(ziel, quelle, x, y [, qx, qy, qb, qh] [, faerbung])` | ein Bild in ein anderes zeichnen, mit Deckkraft gemischt; wahlweise nur ein Ausschnitt der Quelle |
| `IMAGE_SAVE(bild, pfad$)` | das Bild in eine Datei schreiben (`.png`, `.bmp`, `.jpg`, `.tga`) |
| `GETALPHA(bild, x, y)` | Deckkraft eines Bildpunkts, 0..255; `-1` ausserhalb (core, kein IMPORT noetig) |
| `IMAGE_FREE(bild)` | das Bild und seine Grafikspeicher-Textur freigeben |
| `IMAGE_SAVE_GIF(bilder, pfad$ [, fps_oder_dauern [, wiederholen [, anzahl]]])` | mehrere Bilder als **bewegtes GIF** schreiben |

```basic
IMPORT "imgfx"
' Zwei Ebenen zu einem Bild verrechnen und sichern.
DIM unten AS IMAGE : unten = IMAGE_NEW(64, 64, &H204060)
DIM oben AS IMAGE  : oben  = IMAGE_NEW(64, 64)          ' durchsichtig
IMAGE_DRAW_CIRCLE(oben, 32, 32, 20, &HFFCC00)
IMAGE_CLEAR(oben, 28, 28, 8, 8)                          ' ein Loch radieren

DIM fertig AS IMAGE : fertig = IMAGE_NEW(64, 64)
IMAGE_DRAW_IMAGE(fertig, unten, 0, 0)
IMAGE_DRAW_IMAGE(fertig, oben, 0, 0)
IMAGE_SAVE(fertig, "ebenen.png")
```

**Warum `IMAGE_NEW` und nicht `GENTEX_COLOR`:** ein vollstaendig durchsichtiges
Bild laesst sich ueber eine **Farbe** gar nicht ausdruecken. Die Farbkonvention
deutet Deckkraft 0 als *deckend* — nur so bleiben `&Hrrggbb` und `RGB(r,g,b)`
undurchsichtig. `IMAGE_NEW` ohne Farbe umgeht das.

**Warum `GETALPHA` und nicht ein viertes Byte in `GETPIXEL`:** aus demselben
Grund. `GETPIXEL` liefert eine *Farbe* zurueck — ein durchsichtiger Punkt kaeme
dort als deckendes Schwarz an. Wer wissen will, ob ein Punkt leer ist, fragt
`GETALPHA(bild, x, y) = 0`.

**`IMAGE_CLEAR` mischt nicht, es schreibt.** Ein durchsichtiges Rechteck
einzumischen waere ein Nichts-Tun; Radieren muss den Punkt ersetzen.

**`IMAGE_SAVE` und die Endung.** Sie bestimmt das Format, und eine unbekannte
wird abgelehnt statt stillschweigend nichts zu tun (raylib schriebe dann nur
eine Zeile ins Protokoll, und das Programm glaubte, es haette gespeichert).
Ein misslungenes Schreiben — fehlendes Verzeichnis, kein Schreibrecht — meldet
sich ebenfalls; nachgesehen wird an der Datei selbst, weil die raylib-Bindung
das Erfolgs-Flag verwirft.

**`IMAGE_FREE` und was danach gilt.** Ein Bild belegt Arbeitsspeicher *und*
eine Textur im Grafikspeicher; bis 2026-08-31 blieben beide bis zum
Programmende liegen. Für ein Spiel, das seine Bilder einmal lädt, ist das
egal — für alles, was laufend welche erzeugt (ein Editor mit
Rückgängig-Schritten, eine Vorschau, ein Bildbetrachter), war es ein Leck,
das mit jedem Schritt wuchs. Gemessen an 1200 Kopien zu 256×256:
**393 MB gegen 91 MB.**

Nach dem Freigeben ist das Handle **nicht wiederverwendbar und wird auch
nicht neu vergeben**. Jede weitere Benutzung meldet sich im Klartext
(`… wurde mit IMAGE_FREE freigegeben`) statt still auf ein fremdes Bild zu
zeigen — das ist der Grund, warum der Platz stehen bleibt. `GETPIXEL` und
`GETALPHA` bleiben bei ihrer alten Zusage und liefern `-1`.

`LOADIMAGE` merkt sich Pfad → Handle; beim Freigeben wird der Eintrag
mitgelöscht, ein späteres `LOADIMAGE` desselben Pfades lädt also neu.

**Wovon es nichts weiß:** eine Textur, die per `MODEL_TEXTURE` an ein Modell
gegangen ist, lebt dort als reine Nummer weiter (raylibs `Texture2D` ist ein
Struct ohne Zählung) — dieses Modell zeigt danach ins Leere. Bilder in einem
Widget oder einem Sprite-Atlas melden sich immerhin, weil ihr Zeichenweg das
Handle prüft. **Nicht anlegen ist billiger als anlegen und freigeben:** wer
in einer Schleife Momentaufnahmen braucht, legt die Plätze besser einmal an
und überschreibt sie mit `IMAGE_CLEAR` + `IMAGE_DRAW_IMAGE`.

**Bewegte GIFs.** `IMAGE_SAVE_GIF` nimmt ein `ARRAY OF IMAGE` (oder ein
TUPLE) und schreibt daraus eine Animation. `wiederholen` ist die
Endlosschleife (Vorgabe TRUE), `anzahl` sagt, **wie viele Plätze des Feldes
gelten** — ein `DIM b[16] AS IMAGE` mit drei belegten Plätzen ist der
Normalfall, und die leeren wären sonst ein Fehler.

**Der dritte Parameter ist zweierlei:**

| Form | Bedeutung |
|---|---|
| eine Zahl | **Bilder je Sekunde** für alle (Vorgabe 10) |
| ein ARRAY / TUPLE | **Dauer je Bild in Millisekunden** |

```basic
IMPORT "imgfx"
DIM b[3] AS IMAGE
DIM i AS INTEGER
FOR i = 0 TO 2
    b[i] = IMAGE_NEW(16, 16)
    IMAGE_DRAW_CIRCLE(b[i], 4 + i * 4, 8, 3, &HE84B4B)
NEXT
IMAGE_SAVE_GIF(b, "lauf.gif", 8)                  ' 8 Bilder je Sekunde
IMAGE_SAVE_GIF(b, "pose.gif", [1000, 80, 80])     ' erste Pose wird gehalten
```

Dass die **Einheit wechselt**, ist Absicht: ein einzelnes Bild hat keine
Bildrate, es hat eine Dauer. `[4, 12]` als „250 ms, dann 83 ms" zu lesen wäre
die schlechtere Zumutung. Sind es weniger Zeiten als Bilder, ist das ein
Fehler — die letzte stillschweigend zu wiederholen wäre eine Vermutung, und
eine falsche Zeit sieht man dem GIF nicht an, man merkt sie nur.

Genau dafür gibt es das: eine Bildfolge hält eine Pose und lässt die Bewegung
dazwischen schnell durchlaufen. Der Sprite-Editor
(`examples/189_sprite_editor.dh`) schreibt seine Einzelbild-Dauern so heraus.

**Was GIF nicht kann, und was daraus folgt:**

- **Höchstens 256 Farben je Bild.** Pixelgrafik bleibt fast immer darunter —
  deshalb wird die Farbtafel *exakt* aus den vorhandenen Farben gebaut, solange
  es höchstens 255 sind. Erst darüber wird zusammengefasst, und dann verschieben
  sich Farben. Ein Verfahren, das immer zusammenfasst, hätte schon ein
  Vier-Farben-Sprite verfälscht.
- **Durchsichtigkeit nur ganz oder gar nicht.** Ein Punkt ist durchsichtig oder
  deckend, nichts dazwischen; entschieden wird bei Deckkraft 128. Das Format
  kennt nichts anderes.
- **Eine Leinwand für alle Bilder.** Verschieden große Bilder werden abgelehnt
  statt beschnitten — Beschneiden wäre stiller Verlust.
- Die Dauer je Bild rechnet GIF in Hundertstelsekunden. Unter 2 legen die
  meisten Betrachter still ihre eigene fest (meist 10), deshalb wird dort
  geklemmt: sonst liefe die Ausgabe **langsamer** als verlangt, ohne Hinweis.

**Nicht dabei: Bilder ueber die Zwischenablage.** `CLIPBOARD_GET`/`SET` koennen
nur Text. raylibs `GetClipboardImage` gibt es nur unter Windows, und ein Befehl,
den es auf zwei von drei Betriebssystemen nicht gibt, ist eine Falle.

Komplette Fenster-Demo mit allen neuen Ops: [examples/122_imgfx.dh](../examples/122_imgfx.dh).

Alle Funktionen brauchen die native Grafik-Runtime (`dhrt` mit dem `graphics`-Feature). Die Bild-Pipeline wird durch ein vorangegangenes `LOADIMAGE` oder `SCREEN` initialisiert.

## Beispiel

```basic
IMPORT "imgfx"

DIM hero AS IMAGE
hero = LOADIMAGE("assets/hero.png")          ' z.B. 16x16

DIM klein AS IMAGE
klein = IMAGE_SCALE(hero, 8, 8)              ' halbe Groesse

DIM gross AS IMAGE
gross = IMAGE_SCALE(hero, 32, 32)            ' doppelte Groesse (weich)

DIM pixelig AS IMAGE
pixelig = IMAGE_SCALE_NN(hero, 32, 32)       ' doppelte Groesse, Pixel bleiben Pixel

DIM rot_held AS IMAGE
rot_held = IMAGE_TINT(hero, RGB(255, 80, 80))   ' rot getoent

DIM mirror AS IMAGE
mirror = IMAGE_FLIP(hero, TRUE, FALSE)       ' horizontal gespiegelt

DIM gedreht AS IMAGE
gedreht = IMAGE_ROTATE(hero, 45.0)           ' 45 Grad
```

## Wichtige Eigenschaften

**Immutable:** Das Original wird nie verändert. `IMAGE_TINT(hero, ...)` gibt ein neues Bild zurück, `hero` bleibt unverändert.

**Verkettbar:** Effekte lassen sich kombinieren:

```basic
DIM kombi AS IMAGE
kombi = IMAGE_ROTATE(IMAGE_TINT(IMAGE_SCALE(hero, 32, 32), RGB(255, 200, 0)), 30.0)
```

(Hero zuerst auf 32×32 skaliert, dann gelb getönt, dann um 30° gedreht.)

**Bounding-Box bei ROTATE wächst:** Wenn man ein 16×16-Quadrat um 45° dreht, wird das resultierende Bild größer (die Ecken stoßen heraus). Bei 90° bleibt's quadratisch — das Bild wird in eine "neue" Bounding-Box passend gerendert.

```basic
DIM r0 AS IMAGE
r0 = IMAGE_ROTATE(hero, 0.0)                 ' immer noch 16x16
DIM r45 AS IMAGE
r45 = IMAGE_ROTATE(hero, 45.0)               ' jetzt 22x22
DIM r90 AS IMAGE
r90 = IMAGE_ROTATE(hero, 90.0)               ' wieder 16x16
```

> **`IMAGE_ROTATE` ist für Pixelgrafik nicht zu gebrauchen — auch nicht bei
> 90°.** Sie rechnet trigonometrisch und tastet dabei neu ab. Gemessen an
> einem 16×16-Bild mit vier verschiedenfarbigen Eckpunkten: nach
> `IMAGE_ROTATE(b, 90.0)` sind **alle vier verschwunden**, und selbst die
> einfarbige Fläche kommt verwaschen zurück (`0x141414` → `0x131413`).
>
> Für Vierteldrehungen deshalb `IMAGE_ROTATE_CW` / `IMAGE_ROTATE_CCW`: die
> sortieren die Punkte nur um, verlieren also nichts und erfinden nichts.
> Breite und Höhe tauschen dabei — aus 16×32 wird 32×16. Dieselbe
> Unterscheidung wie `IMAGE_SCALE` gegen `IMAGE_SCALE_NN`; sie fällt hier nur
> stärker auf, weil eine Vierteldrehung eigentlich verlustfrei ist.

```basic
' Exakt -- fuer Sprites die einzige brauchbare Drehung:
DIM rechts AS IMAGE : rechts = IMAGE_ROTATE_CW(hero)     ' 16x32 -> 32x16
DIM links AS IMAGE  : links  = IMAGE_ROTATE_CCW(hero)
' Zweimal CW ist eine halbe Drehung, dreimal ist CCW.
```

**TINT als RGB-Multiplikation:** `IMAGE_TINT(img, color)` multipliziert jeden Pixel mit `color`/255 pro Kanal:

| Tint-Farbe | Effekt |
|---|---|
| `&HFFFFFF` (Weiß) | unverändert |
| `&H000000` (Schwarz) | komplett schwarz |
| `&HFF0000` (Rot) | nur Rot-Kanal überlebt |
| `&HFF8080` | rosa-getönt (Hell-Rot mit etwas Grün/Blau) |
| `&H808080` | halb so hell |

**SCALE:** verlangt `w > 0` und `h > 0`. Negative oder Null wirft.

**FLIP:** `flipX = TRUE` spiegelt horizontal (links-rechts), `flipY = TRUE` vertikal (oben-unten).

## Anwendungs-Beispiele

**Pickup-Aufblitzen** (Coin wird kurz weiß bevor sie verschwindet):

```basic
IMPORT "imgfx"

DIM coin AS IMAGE
coin = LOADIMAGE("assets/coin.png")
DIM coin_flash AS IMAGE
coin_flash = IMAGE_TINT(coin, RGB(255, 255, 255))

' Im Game-Loop: solange Pickup-Animation laeuft, das aufgehellte Bild zeigen
DRAWIMAGE(coin_flash, coin_x, coin_y)
```

**Vorbereiten von Spritesheet-Variants** (statt jedes Frame zur Laufzeit zu transformieren):

```basic
IMPORT "imgfx"

DIM hero_normal AS IMAGE
hero_normal = LOADIMAGE("assets/hero.png")
DIM hero_red AS IMAGE
hero_red = IMAGE_TINT(hero_normal, RGB(255, 80, 80))     ' getroffen
DIM hero_left AS IMAGE
hero_left = IMAGE_FLIP(hero_normal, TRUE, FALSE)         ' nach links laufen
```

So wirst du ohne Performance-Probleme bei jedem Frame nur noch das richtige Bild blitten, keine Re-Transformation.

**Pixel-Art-Skalierung:**

```basic
IMPORT "imgfx"

' 16x16-Sprite auf 64x64 fuer Detail-Anzeige im Inventar
DIM mini AS IMAGE
mini = LOADIMAGE("assets/sword.png")
DIM big AS IMAGE
big = IMAGE_SCALE(mini, 64, 64)
```

## Performance

`IMAGE_SCALE`, `IMAGE_ROTATE`, `IMAGE_FLIP`, `IMAGE_TINT` allokieren jedes Mal ein neues Bild. Mache sie **einmal beim Laden**, nicht in jedem Frame. Wenn du dynamische Effekte willst (z.B. variabler Tint), nutze stattdessen das [Sprite-Modul](module-sprite.md) mit `SPRITE_TINT` — das kümmert sich pro Frame intelligent darum.

## Komplettes Beispiel

Siehe [examples/27_imgfx.dh](../examples/27_imgfx.dh) — Konsolen-Demo mit allen Effekten und Verkettung.
