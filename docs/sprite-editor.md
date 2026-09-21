# Sprite-Editor (`examples/189_sprite_editor.dh`)

Ein Pixel-Art-Editor, geschrieben in Drachenhauch selbst: mehrere
Einzelbilder, Ebenen, eine echte Auswahl-Maske, Rückgängig, Paletten im
GIMP-Format und Ausgaben, die das Spiel direkt lädt — Streifen-PNG mit
Atlas-JSON für `ATLAS_LOAD`, ein lauffähiges Programm für `SPRITE_NEW`, eine
Zustandsmaschine für `ANIM_FSM_LOAD` und ein bewegtes GIF.

Er ersetzt den früheren Qt-Editor `dhsprites`. Was der zusätzlich konnte,
steht am Ende.

## Starten

In der IDE ([`ide/ide.dh`](ide.md)) über das Menü **Werkzeuge → Sprite-Editor**
— er läuft dort als eigener Prozess. Ohne IDE:

```
dhrt run examples/189_sprite_editor.dh
```

Der Editor startet im Vollbild mit einem leeren Sprite von 32×32 Punkten.
Eine Datei nimmt er nicht auf der Kommandozeile, sondern über **[Oeffnen]**.

## Bedienung

| Eingabe | Wirkung |
|---|---|
| Maus links | zeichnen mit dem gewählten Werkzeug |
| Maus rechts (ziehen) | Ansicht verschieben |
| Mausrad über der Fläche | Zoom (1 bis 32) |
| `P` `E` `F` `L` `R` `O` `Y` `I` `S` `Q` `Z` `V` | Werkzeug wählen (siehe unten) |
| `T` | Kachel-Ansicht 3×3 an/aus |
| `1`..`6` | Palettenfarbe 1 bis 6 |
| `+` / `-` | Pinsel größer / kleiner (1 bis 6 Punkte) |
| `Strg+Z` / `Strg+Y` | zurück / vor (24 Schritte) |
| `Strg+C` / `Strg+X` / `Strg+V` | Auswahl kopieren / ausschneiden / einsetzen |
| `Strg+D` | Auswahl aufheben |
| `Entf` | Auswahl leeren |
| `ESC` oder das Fensterkreuz | beenden |

Ist das Sprite seit dem letzten Sichern als `.dhsprite` verändert worden,
fragen ESC und das Kreuz vorher nach (**Sichern | Verwerfen | Abbrechen**).
Ein Export als Streifen oder PNG zählt dabei nicht als gesichert — dort sind
die Ebenen weg.

## Werkzeuge

Die Werkzeugleiste oben trägt alle zwölf als Knöpfe, daneben
**[Zurueck]**/**[Vor]** und die Schalter **gefuellt**, **Spiegel X**,
**Spiegel Y**, **Zwiebelhaut** und **Raster**.

| Taste | Werkzeug | Wirkung |
|---|---|---|
| `P` | Stift | Punkte setzen, Pinselgröße 1..6 |
| `E` | Radierer | Punkte durchsichtig machen |
| `F` | Füllen | zusammenhängende Fläche gleicher Farbe und Deckkraft füllen |
| `L` | Linie | Linie ziehen |
| `R` | Rechteck | Rechteck — Umriss, mit **gefuellt** gefüllt |
| `O` | Ellipse | Ellipse — Umriss, mit **gefuellt** gefüllt |
| `Y` | Sprühen | zufällige Punkte im Pinselradius |
| `I` | Pipette | Farbe aus dem Bild übernehmen |
| `S` | Auswahl | Rechteck-Auswahl (ein Klick ohne Zug hebt sie auf) |
| `Q` | Lasso | Freiform-Auswahl: den Bereich umfahren, Loslassen schließt ihn |
| `Z` | Zauberstab | alle zusammenhängenden Punkte gleicher Farbe wählen |
| `V` | Verschieben | die Auswahl abheben und woanders absetzen — ohne Auswahl die ganze Ebene |

**Spiegel X/Y** malen beim Zeichnen gespiegelt mit (für Figuren, Symbole,
Logos). **Zwiebelhaut** legt das vorige Einzelbild blass unter das aktuelle.

### Die Auswahl ist eine Maske

Rechteck, Lasso und Zauberstab schreiben alle in dieselbe **Punkt-Maske**,
nicht in einen Rahmen — und alles, was zeichnet, fragt sie. Solange eine
Auswahl steht, trifft kein Strich daneben. Kopieren, Ausschneiden und Entf
wirken nur auf die gewählten Punkte; beim Verschieben wird der Inhalt
abgehoben (an der alten Stelle bleibt ein Loch, keine Kopie) und die Maske
wandert mit. Halbdurchsichtige Punkte behalten dabei ihre Deckkraft.

## Linke Spalte: Farbe, Palette, Leinwand

- **Farbe** — ein Farbwähler, darunter die **Palette** mit 16 Plätzen.
- **[Palette laden]** / **[Palette sichern]** — GIMP-Paletten (`.gpl`), das
  Format, das GIMP, Aseprite, Krita und die Palettensammlungen im Netz
  sprechen. Beim Laden werden krumme Zeilen übergangen statt gemeldet (die
  Dateien werden von Hand bearbeitet); hat die Datei mehr als 16 Farben,
  sagt die Statuszeile, wie viele nicht passten.
- **Pinsel** — Größe 1 bis 6.
- **Kachel-Ansicht 3×3** (`T`) — zeigt das Bild acht Mal ringsum. Eine Naht
  sieht man erst neben ihrer Wiederholung; der Zoom passt sich dafür neu ein.
- **[Statistik]** — Punkte, Farben und häufigste Farbe, gesamt und je Bild.
- **[Zuschneiden]** — auf den Inhalt, gemessen über **alle** Ebenen, auch
  ausgeblendete (sonst verschwände Inhalt, den man gerade nicht sieht).
- **[Groesse aendern]** — neue Breite und Höhe; dazukommender Rand ist
  durchsichtig.
- **[Spiegeln |]**, **[Spiegeln --]**, **[90 >]**, **[< 90]** — spiegeln und
  vierteldrehen. Alle vier gelten für das **ganze** Sprite (alle Bilder,
  alle Ebenen); eine Drehung tauscht Breite und Höhe. Sie stehen **nicht** im
  Verlauf — der zeichnet je Schritt eine Ebene auf, ein Rückgängig danach
  drehte nur eine zurück. Der Verlauf wird deshalb geleert; die
  Gegenrichtung nimmt eine Wandlung zurück.
- **[GB-Code]** und **[dhanim]** — siehe Ausgaben.

## Rechte Spalte: Bilder, Ebenen, Vorschau, Bereiche

### Einzelbilder

**[Neu]** hängt ein leeres Bild an, **[Kopie]** eine Kopie des aktuellen,
**[Weg]** entfernt es. **[Name]** gibt dem Bild einen Namen — er wird sein
Schlüssel im Atlas (`ATLAS_DRAW(atlas, "kopf", ...)` statt `"bild_0"`).
Namen sind Kennungen: erlaubt sind Buchstaben, Ziffern, `_` und `-`, alles
andere wird zum Unterstrich. Ein Punkt ginge nicht, weil das json-Modul einen
Schlüssel mit Punkt als Pfad liest.

**Dauer** stellt je Bild eine eigene Zeit in Millisekunden ein; **0 heißt
„der Tempo-Regler gilt"**. So wird eine Pose gehalten und der Lauf dazwischen
nicht. Die Zeit steht in der Bildliste, wandert beim Kopieren mit (der Name
nicht — er ist eine Kennung) und gilt für Vorschau und GIF gleichermaßen.

### Ebenen

Ebenen gelten für **alle** Einzelbilder, wie in Aseprite: eine Ebene
ausblenden blendet sie überall aus. **[Neu]**, **[Weg]** und **sichtbar**.
Gezeichnet wird auf der gewählten Ebene; Anzeige und alle Ausgaben außer der
`.dhsprite` zeigen die sichtbaren Ebenen zusammengerechnet.

### Vorschau und Bereiche

Die **Vorschau** spielt laufend ab, das Tempo kommt vom Regler (1 bis 24
Bilder je Sekunde), Bilder mit eigener Dauer halten ihre Zeit.

**Bereiche** sind benannte Animationen — Name, von, bis, fps, genau das, was
`SPRITE_ADD_ANIM` braucht. **[Hinzu]**, **[Aendern]**, **[Weg]**; die
Vorschau spielt den gewählten. Wird ein Bild gelöscht, wandern die Bereiche
mit, sonst spielte die Vorschau danach etwas anderes.

## Grenzen

| | |
|---|---|
| Kantenlänge | 4 bis 256 Punkte |
| Einzelbilder | 64 |
| Ebenen | 8 |
| Bereiche | 8 |
| Rückgängig | 24 Schritte |

Ein Bild einer Ebene entsteht erst, wenn es gebraucht wird — ein Sprite mit
einem Bild belegt auch nur eines. Fest angelegt sind nur die Plätze des
Verlaufs in Sprite-Größe; daran hängt die Grenze von 256 Punkten.

## Dateien

### `.dhsprite` — das eigene Format

**[Sichern]** schreibt zwei Dateien: die Beschreibung `name.dhsprite` (JSON)
und daneben das Raster `name.dhsprite.png` — Spalten sind die Einzelbilder,
Zeilen die Ebenen. Das PNG öffnet jeder Bildbetrachter; die JSON trägt
Maße, Ebenennamen und Sichtbarkeit und, falls vorhanden, Bildnamen,
Einzeldauern und Bereiche:

```json
{
  "format": "dhsprite-gitter-1",
  "bild": "held.dhsprite.png",
  "breite": 16, "hoehe": 16, "bilder": 4,
  "ebenen": [ { "name": "Ebene 1", "sichtbar": true } ],
  "bildnamen": [ "stand", "", "", "" ],
  "bilddauern": [ 400, 0, 0, 0 ],
  "bereiche": [ { "name": "lauf", "von": 1, "bis": 3, "fps": 8 } ]
}
```

Die drei letzten Blöcke sind optional. Gesucht wird das Raster beim Laden
**neben** der Beschreibung, nicht unter dem eingetragenen Namen — so bleibt
ein verschobenes Paar zusammen.

Dateien der früheren Qt-Fassung (JSON mit base64-Pixeln) öffnet dieser Editor
nicht.

### Öffnen

**[Oeffnen]** nimmt zweierlei, die Endung entscheidet: eine `.dhsprite` mit
allen Ebenen, oder ein Streifen-PNG, neben dem seine Atlas-JSON liegt. Beim
Streifen kommen die Bilder auf einer Ebene zurück — er **ist** das
zusammengerechnete Bild; die Bildnamen kommen über die Atlas-Schlüssel mit.

## Ausgaben

| Knopf | Ergebnis |
|---|---|
| **[PNG]** | das aktuelle Einzelbild (sichtbare Ebenen) als PNG |
| **[Streifen]** | alle Bilder nebeneinander als PNG, dazu die Atlas-JSON gleichen Namens |
| **[GIF]** | bewegtes GIF mit Durchsichtigkeit; Bilder mit eigener Dauer behalten sie |
| **[GB-Code]** | ein lauffähiges Programm `name.dh` samt Blatt `name.png` |
| **[dhanim]** | die Bereiche als Zustandsmaschine `.dhanim` |

### Streifen + Atlas

Die JSON hat genau die Form, die `ATLAS_LOAD` liest — Schlüssel ist der
Bildname, sonst `bild_<n>`; ein doppelter Name bekommt `_<n>` angehängt:

```json
{
  "image": "held.png",
  "sprites": {
    "stand":  [0,  0, 16, 16],
    "bild_1": [16, 0, 16, 16]
  }
}
```

Im Spiel:

```basic
SCREEN(320, 200, "Atlas", 1)
DIM atlas AS SPRITE_ATLAS
atlas = ATLAS_LOAD("held.json")
WHILE NOT QUITREQUESTED()
    CLS(BLACK)
    ATLAS_DRAW(atlas, "stand", 40, 40)
    ATLAS_DRAW(atlas, "bild_1", 60, 40)
    FLIP()
WEND
```

### GB-Code

**[GB-Code]** schreibt Programm und Blatt in einem Zug und unter demselben
Namen — wer nur den Code hätte, hätte einen Verweis ins Leere. Das Programm
lädt das Blatt mit `SPRITE_NEW`, legt je Bereich eine
`SPRITE_ADD_ANIM`-Zeile an (ohne Bereiche eine `"idle"` über alle Bilder mit
dem Tempo des Reglers) und spielt die erste.

### dhanim

**[dhanim]** schreibt je Bereich einen Zustand, der erste ist der
Startzustand (ohne Bereiche einer `"idle"` über alles). Übergänge und
Parameter bleiben leer — wann welcher Zustand in welchen wechselt, ist eine
Aussage über das Spiel. Die Datei lädt `ANIM_FSM_LOAD` so, wie sie ist; die
Übergänge ergänzt man im [Anim-FSM-Editor](anim-editor.md).

## Typische Wege

### Lauf-Zyklus mit gehaltener Pose

1. **[Neu]** in der Werkzeugleiste, 16×16.
2. Bild 1 zeichnen, rechts dreimal **[Kopie]** und abwandeln; mit
   **Zwiebelhaut** sieht man das vorige Bild darunter.
3. Bild 1 eine **Dauer** von 400 ms geben, die übrigen folgen dem Tempo.
4. Einen Bereich `lauf` von 2 bis 4 anlegen.
5. **[Sichern]** als `.dhsprite` (Arbeitsstand), **[GB-Code]** für ein
   Programm, das sofort läuft.

### Kachelsatz

1. 16×16, je Einzelbild eine Kachel, jede mit **[Name]** benannt.
2. `T` für die Kachel-Ansicht — kachelt es ohne Naht?
3. **[Streifen]** schreibt PNG und Atlas; im Spiel
   `ATLAS_DRAW(atlas, "gras", x, y)`.

## Was die frühere Qt-Fassung zusätzlich konnte

- Export-Skalierung (1×–8×) für alle Bild-Ausgaben.
- Sheet-Import eines beliebigen PNG mit Angabe der Bildgröße, Einsetzen eines
  Bildes aus der System-Zwischenablage als neues Einzelbild.
- Ebenen je Einzelbild mit Deckkraft, Umordnen und Zusammenführen.
- Bild-Operationen auf der Bildliste: umkehren, Ping-Pong anhängen, auf das
  aktuelle Bild reduzieren.
- Farbe ersetzen, Palette aus dem Sprite übernehmen, einstellbare
  Zwiebelhaut (Deckkraft, bis zu drei Bilder je Richtung), Dateibrowser.

## Prüfung

Die Zusagen dieser Seite prüft [`tests/pruef/werkzeug_sprite.dhtest`](../tests/pruef/werkzeug_sprite.dhtest)
mit echten Mauswegen über die Aufnahme-Wiedergabe: Auswahl, Lasso und
Zauberstab, Verschieben, Paletten gegen einen fremden Leser, Statistik,
Zuschneiden, Bereiche, GB-Code und `.dhanim` (gestartet bzw. von der Laufzeit
geladen), Atlas-Schlüssel, Drehen und Spiegeln, Einzeldauern im GIF und die
Grenzen von 64 Bildern und 8 Ebenen.
