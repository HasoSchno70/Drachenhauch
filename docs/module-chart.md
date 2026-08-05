# Modul `chart` — Diagramme

Kuchen/Donut, Balken, Linie/Fläche und analoge Tacho-Anzeigen. Handle-basiert
wie `gui` und `particles`: einmal aufbauen und einstellen, dann pro Bild
zeichnen.

```basic
IMPORT "chart"

DIM t AS CHART
t = CHART_NEW("tacho", 500, 300, 240, 240)
CHART_SET(t, "titel", "Drehzahl")
CHART_SET_NUM(t, "max", 8000)
CHART_ZONE(t, 6500, 8000, RED)

' pro Bild:
CHART_VALUE(t, drehzahl)
CHART_DRAW(t)
```

`CHART_DRAW` braucht ein Fenster (`SCREEN`). Alles andere — Aufbauen, Daten
setzen, Kennzahlen abfragen — läuft auch in einem Konsolenprogramm.

## Diagrammarten

`CHART_NEW(art$, x, y, breite, hoehe)`. Die Art versteht deutsche und englische
Namen:

| Art | Namen | Was es zeigt |
|---|---|---|
| Kuchen | `kuchen`, `pie`, `donut` | Anteile an einem Ganzen; mit `innenradius` > 0 als Ring |
| Balken | `balken`, `bar` | Werte je Kategorie, senkrecht oder waagerecht, gruppiert oder gestapelt |
| Linie | `linie`, `line`, `flaeche`, `area` | Verläufe über die Zeit, mehrere Reihen, wahlweise mit Fläche darunter |
| Tacho | `tacho`, `gauge` | Ein Wert auf einer Rundskala mit Zeiger und Farbzonen |
| Leiste | `leiste`, `balkenanzeige`, `bar_gauge` | Ein Wert auf einer liegenden oder stehenden Leiste, mit Marker |
| Lampen | `led`, `lampen`, `zellen` | Ein Wert als Kette diskreter Zellen, die bis dahin leuchten |

## Daten

Zwei Wege — der kurze für Kuchen und Tacho, der volle für mehrere Reihen.

**Kurz** (eine Reihe, Farbe je Segment):

```basic
CHART_ADD(c, "Holz",  45.0, BROWN)     ' Name, Wert, Farbe (Farbe optional)
CHART_ADD(c, "Stein", 30.0, GRAY)
```

**Voll** (mehrere Reihen über gemeinsame Kategorien):

```basic
DIM r AS INTEGER
DIM werte[7] AS FLOAT
r = CHART_SERIES(c, "Einnahmen")       ' -> Reihen-Index
CHART_DATA(c, r, werte)                ' ARRAY OF FLOAT
CHART_LABEL(c, 0, "Mo")                ' Kategorie beschriften
```

> **Achtung bei der Farbe:** `0` ist **Schwarz**, nicht „nimm die nächste
> Palettenfarbe". Wer die Palette will, lässt das Argument weg (oder übergibt
> `-1`). `CHART_SERIES(c, "Name", 0)` liefert eine schwarze Reihe — ein
> leicht gemachter und schwer zu sehender Fehler.

| Befehl | Wirkung |
|---|---|
| `CHART_SERIES(c, name$ [, farbe])` | Reihe anlegen → Index |
| `CHART_ADD(c, name$, wert [, farbe])` | Kategorie + Wert in Reihe 0 → Index |
| `CHART_DATA(c, reihe, werte)` | Werte einer Reihe komplett ersetzen (`ARRAY OF FLOAT`) |
| `CHART_SET_POINT(c, reihe, punkt, wert)` | Einzelnen Wert ändern |
| `CHART_PUSH(c, reihe, wert)` | Wert hinten anhängen (Live-Kurven, siehe `fenster`) |
| `CHART_VALUE(c, wert)` | Kurzform für Reihe 0, Punkt 0 — der Tacho-Weg |
| `CHART_LABEL(c, punkt, text$)` | Kategorie beschriften |
| `CHART_GET(c, reihe, punkt)` | Wert auslesen |
| `CHART_COUNT(c)` / `CHART_SERIES_COUNT(c)` | Anzahl Punkte / Reihen |
| `CHART_CLEAR(c)` | Daten verwerfen |
| `CHART_STAT(c, reihe, kennzahl$)` | `anzahl`/`summe`/`mittel`/`min`/`max` |
| `CHART_BOUNDS(c, x, y, b, h)` | Feld verschieben/skalieren |
| `CHART_ZONE(c, von, bis, farbe [, name$])` / `CHART_ZONE_CLEAR(c)` | Farbzonen der Tacho-Skala, wahlweise beschriftet |
| `CHART_UPDATE(c, sekunden)` | Animation weiterlaufen lassen (siehe unten) |
| `CHART_DRAW(c)` | Zeichnen (braucht `SCREEN`) |

## Aussehen einstellen

Es sind rund vierzig Stellschrauben — als je eigenes Builtin wäre das eine
unüberschaubare Liste. Stattdessen gibt es **vier Setter**, die den Namen der
Eigenschaft als Zeichenkette nehmen:

```basic
CHART_SET(c,       "titel", "Rohstoffe")   ' Text
CHART_SET_NUM(c,   "innenradius", 0.55)    ' Zahl
CHART_SET_COLOR(c, "gitter", DARKGRAY)     ' Farbe
CHART_SET_FLAG(c,  "prozent", TRUE)        ' An/Aus
```

Der Preis dafür: ein Tippfehler fällt erst zur Laufzeit auf. Die Meldung nennt
dann aber immer, was gültig gewesen wäre:

```
CHART_SET_NUM: unbekannte Eigenschaft 'innen_radius' (gueltig: min, max, innenradius, …)
```

### Text — `CHART_SET`

| Name | Bedeutung |
|---|---|
| `titel` | Überschrift über dem Diagramm |
| `einheit` | wird an jede Zahl gehängt (`" km/h"`) |
| `achse_x`, `achse_y` | Achsenbeschriftung (y wird gedreht gezeichnet) |
| `legende` | `aus` / `oben` / `unten` / `links` / `rechts` |
| `werte` | Werte am Datenpunkt: `aus` / `innen` / `aussen` |
| `ausrichtung` | Balken: `senkrecht` / `waagerecht` |
| `zeigerform` | Tacho: `nadel` / `balken` / `pfeil` |
| `zifferblatt` | Tacho: `ring` / `segmente` / `striche` / `baender` |
| `wertanzeige` | Tacho: `aus` / `innen` / `pille` / `blase` / `am_zeiger` |

### Zahlen — `CHART_SET_NUM`

| Name | Bedeutung |
|---|---|
| `min`, `max` | Achsengrenzen; ungesetzt = aus den Daten |
| `innenradius` | 0…0.95, Anteil des Außenradius → Donut |
| `abstand` | Kuchen: Segmente herausziehen. Balken: Lücke |
| `ecken` | Eckenradius |
| `rahmen_dicke`, `polster` | Rahmenstärke, Innenabstand |
| `gitter` | Schrittweite der Wertachse; 0 = automatisch (runde Schritte) |
| `nachkomma` | Nachkommastellen aller Zahlen |
| `titel_groesse`, `text_groesse` | Schriftgrößen |
| `schrift` | FONT-Handle aus `LOADFONT`; -1 = Standardschrift |
| `start_winkel`, `end_winkel` | Tacho-Bogen in Grad (Vorgabe 135…405) |
| `striche`, `unterstriche` | Haupt- und Nebenstriche der Tacho-Skala |
| `linien_dicke`, `punkt_radius` | Liniendiagramm |
| `animation` | Sekunden, die der angezeigte Wert nachzieht; 0 = sofort |
| `fenster` | Gleitendes Fenster für `CHART_PUSH`; 0 = unbegrenzt |
| `schatten` | Versatz des Schlagschattens in Pixeln; 0 = keiner |
| `schatten_weich` | Weichzeichnung des Schattens in Pixeln |
| `blatt_teile`, `blatt_luecke`, `blatt_dicke` | Teilung des Zifferblatts |
| `fassung` | metallischer Ring um die Tacho-Scheibe (Pixel) |
| `deckkraft` | 0…1 für **alle** Datenfarben |
| `flaeche_deckkraft` | 0…1 für die Fläche unter einer Linie |

### Farben — `CHART_SET_COLOR`

`hintergrund`, `rahmen`, `gitter`, `text`, `titel`, `achse`, `zeiger`,
`flaeche`, `verlauf`, `schatten`, `verlauf_ende`, sowie der Skalenverlauf
`skala_von`, `skala_mitte`, `skala_bis`.

Dazu `CHART_PALETTE(c, farben)` mit einem `ARRAY OF INTEGER` — die
Reihenfolge, aus der Reihen und Segmente ohne eigene Farbe bedient werden.

### Schalter — `CHART_SET_FLAG`

`rahmen`, `gitter_x`, `gitter_y`, `prozent`, `flaeche`, `punkte`, `glatt`
(Kurve statt Streckenzug), `null_linie`, `stapel`, `verlauf`
(Hintergrundverlauf), `verlauf_daten`, `schatten_daten`, `kurz`
(1.2M statt 1200000, wie `NUMFMT$`).

### Themen

`CHART_THEME(c, "dunkel" | "hell" | "neon" | "pastell")` setzt alle Farbrollen
und die Palette auf einmal. Danach lässt sich jede einzelne Farbe weiter
übersteuern — ein späteres `CHART_THEME` setzt sie allerdings wieder zurück.

## Tacho gestalten

Vier Bauarten des Zifferblatts, über `zifferblatt`:

| Wert | Aussehen |
|---|---|
| `ring` | durchgehender Bogen (Vorgabe) |
| `segmente` | einzelne Balken mit Lücke — der klassische „Ladebalken im Kreis" |
| `striche` | dieselbe Teilung, aber nur als schmale Striche am Außenrand |
| `baender` | volle Sektoren bis zur Mitte |

`blatt_teile` (Anzahl), `blatt_luecke` (Lücke in Grad) und `blatt_dicke`
steuern die Teilung; `fassung` legt einen metallisch wirkenden Ring um die
Scheibe (Breite in Pixeln, 0 = keiner).

**Beschriftete Zonen.** `CHART_ZONE` nimmt als fünftes Argument einen Namen,
der entlang des Bogens mitgedreht wird:

```basic
CHART_ZONE(t, 0,   350,  RED,    "SCHLECHT")
CHART_ZONE(t, 350, 700,  YELLOW, "NORMAL")
CHART_ZONE(t, 700, 1000, GREEN,  "GUT")
```

Text in der unteren Hälfte wird automatisch umgedreht, damit er nicht auf dem
Kopf steht.

**Wertanzeige** über `wertanzeige`:

| Wert | Aussehen |
|---|---|
| `innen` | schlicht unter der Mitte (Vorgabe) |
| `pille` | abgerundete Kapsel — **in der Farbe der getroffenen Zone** |
| `blase` | dunkler Kasten mit Zipfel |
| `am_zeiger` | kleine Kapsel an der Zeigerspitze, wandert mit |
| `aus` | keine |

Der Tacho hängt allein an `wertanzeige`. Der Schalter `werte` bleibt für die
Beschriftung einzelner Datenpunkte bei Kuchen und Balken zuständig — beides zu
koppeln hieße, zwei Schalter für dieselbe Sache zu haben.

Die Skalenstriche schaltet `striche` = 0 ganz ab (nützlich bei `baender`, wo
die Sektoren bis zur Mitte reichen).

## Leisten und Lampen

Die linearen Geschwister des Tachos — ein Wert, dieselben Farbzonen, nur
gerade statt rund.

```basic
DIM l AS CHART
l = CHART_NEW("leiste", 20, 20, 460, 120)
CHART_SET_NUM(l, "max", 1000)
CHART_SET(l, "zeigerform", "balken")   ' nur bis zum Wert füllen
CHART_VALUE(l, 630.0)
```

Beide liegen per Vorgabe **waagerecht**; `ausrichtung` = `senkrecht` stellt sie
hochkant (der Marker wandert dann an die Seite, damit er die Leiste nicht
verdeckt). Bei `led` bestimmt `blatt_teile` die Zellenzahl, `blatt_luecke` den
Abstand in Pixeln und `blatt_dicke` die Höhe. `zeigerform` = `balken` füllt nur
bis zum Wert, sonst ist die ganze Skala zu sehen und nur der Marker wandert.

Die Zellen einer LED-Anzeige sind einzeln anklickbar — `CHART_HOVER` liefert
den Index der Zelle unter der Maus.

**Farbe der Skala.** Gibt es Farbzonen (`CHART_ZONE`), gewinnen sie — dieselbe
Angabe färbt damit Tacho, Leiste und Lampen. Ohne Zonen läuft ein gerichteter
Verlauf `skala_von` → `skala_mitte` → `skala_bis` (Vorgabe rot → gelb → grün,
im Ton des jeweiligen Themas).

> Das ist bewusst **nicht** die Palette aus `CHART_PALETTE`. Die ist
> kategorial — acht gut unterscheidbare Farben für acht Reihen — und ergibt
> interpoliert einen Regenbogen. Einer Skala muss man ansehen, wo „wenig" und
> wo „viel" ist.

## Durchsichtigkeit

Farben in GameBasic sind `0xAARRGGBB`. Das oberste Byte ist die Deckkraft, und
**0 bedeutet dabei deckend** (damit alte 24-Bit-Farben unverändert bleiben).
Halbdurchsichtig geht am bequemsten über `RGBA(r, g, b, a)`:

```basic
CHART_SET_COLOR(c, "flaeche", RGBA(0, 160, 255, 90))   ' zartblaue Fläche
CHART_SET_COLOR(c, "schatten", RGBA(0, 0, 0, 140))     ' weicher Schatten
```

Wer nicht jede Farbe einzeln anfassen will, dreht stattdessen an
`deckkraft` — der Regler multipliziert die Deckkraft **aller** Datenfarben und
eignet sich gut, um zwei übereinanderliegende Reihen lesbar zu machen.

## Schatten

`schatten` ist der Versatz, `schatten_weich` die Anzahl gestaffelter Kopien mit
jeweils einem Bruchteil der Deckkraft — zusammen ergibt das den weichen Rand
(raylib hat keinen Weichzeichner für Formen). `schatten_daten` schaltet zu, dass
auch die Daten selbst Schatten werfen: Balken, Kuchensegmente und der
Tacho-Zeiger.

```basic
CHART_SET_NUM(c, "schatten", 6)
CHART_SET_NUM(c, "schatten_weich", 6)
CHART_SET_FLAG(c, "schatten_daten", TRUE)
```

## Verläufe

Zwei getrennte Schalter:

- `verlauf` färbt den **Hintergrund** des Feldes von `hintergrund` nach
  `verlauf`.
- `verlauf_daten` färbt die **Daten**: Balken und die Fläche unter einer Linie
  bekommen einen senkrechten Verlauf, Kuchensegmente ein abgedunkeltes
  Innenband. Zielfarbe ist `verlauf_ende`; ist sie nicht gesetzt (-1), wird
  automatisch eine abgedunkelte Fassung der jeweiligen Reihenfarbe genommen —
  so passt der Verlauf ohne Zutun zu jeder Palette.

## Maus: Hervorhebung, Sprechblasen, Klick

`CHART_DRAW` wertet die Maus selbst aus — es braucht keinen zusätzlichen
Aufruf. Wer die Maus über ein Segment, einen Balken oder einen Punkt bewegt,
sieht ihn weich aufleuchten; das Kuchenstück rückt zusätzlich heraus, der
Linienpunkt wächst.

```basic
CHART_SET_FLAG(c, "tooltip", TRUE)      ' Sprechblase mit Name und Wert

' pro Bild, nach CHART_DRAW:
IF CHART_CLICKED(c) >= 0 THEN
    PRINT "Angeklickt: " + CHART_HOVER_LABEL$(c)
END IF
```

| Befehl | Liefert |
|---|---|
| `CHART_HOVER(c)` | Punkt unter der Maus, `-1` = keiner |
| `CHART_HOVER_SERIES(c)` | Reihe unter der Maus, `-1` = keine |
| `CHART_HOVER_LABEL$(c)` | Beschriftung dieses Punktes (`""` = keiner) |
| `CHART_HOVER_VALUE(c)` | sein Wert |
| `CHART_CLICKED(c)` | in **diesem** Bild angeklickter Punkt, sonst `-1` |
| `CHART_CLICKED_SERIES(c)` | zugehörige Reihe |

Alle Werte gelten **nach** `CHART_DRAW` — vorher stehen sie auf `-1`.

Einstellbar: `hover` (Effekt ganz aus), `tooltip` (Sprechblase),
`hover_tempo` (Sekunden fürs Ein-/Ausblenden, 0 = sofort), `hover_weite`
(wie weit herausrücken/wachsen), `hover_glanz` (wie stark aufhellen).

Die Hervorhebung mischt die Farbe gegen **Weiß** statt sie hochzurechnen.
Das ist Absicht: bei gesättigten Farben klemmt der größte Kanal schon bei 255,
sodass nur die kleineren mitwachsen — ein hervorgehobenes Orange wurde dabei
sichtbar gelb und sah aus wie ein anderer Eintrag der Palette.

## Animation

Ohne `animation` zeigt das Diagramm immer den gesetzten Wert. Mit
`animation` > 0 zieht die Anzeige weich nach — dann muss pro Bild
`CHART_UPDATE(c, DELTA())` laufen:

```basic
CHART_SET_NUM(t, "animation", 0.4)
' pro Bild:
CHART_VALUE(t, drehzahl)
CHART_UPDATE(t, DELTA())
CHART_DRAW(t)
```

`CHART_GET` liefert immer den **gesetzten** Wert, nie den gerade angezeigten —
ein Programm kann seine eigenen Daten also zuverlässig zurücklesen.

## Live-Kurven

`fenster` macht aus dem Liniendiagramm einen Schreiber: `CHART_PUSH` hängt
hinten an, vorne fällt der älteste Wert heraus.

```basic
CHART_SET_NUM(linie, "fenster", 120)
' pro Bild:
CHART_PUSH(linie, r, FPS())
```

## Kamera

`CHART_DRAW` zeichnet wie alle anderen Zeichenbefehle in Weltkoordinaten. Für
eine Anzeige, die fest im Bild stehen soll, vorher `CAMERA_RESET()` aufrufen —
genauso wie bei `TEXT`.

## Grenzen

- **Kuchen-Verläufe sind eine Näherung.** Ein echter Radialverlauf ist mit dem
  Ring-Primitiv nicht zu haben; `verlauf_daten` dunkelt beim Kuchen stattdessen
  das Innenband ab. Optisch ergibt das dieselbe Tiefenwirkung, mathematisch
  ist es kein Verlauf.
- **Runde Ecken und Verlauf überlagern sich.** Der Verlauf ist rechteckig und
  sitzt um den Eckenradius eingerückt; der schmale Rand bleibt in der
  Ausgangsfarbe.
- **Kein Streudiagramm, keine zweite Y-Achse, keine logarithmische Skala.**
- **Keine Maus-Interaktion** — kein Anklicken von Segmenten, keine
  Werte-Sprechblasen. Wer das braucht, fragt die Mausposition selbst ab und
  rechnet gegen die eigenen Daten.
- **Beim Balken-Tacho** liegen die Farbzonen als schmaler Außenrand auf dem
  Ring, weil der Fortschrittsbogen denselben Platz füllt.

## Dateien

- Umsetzung: [`rust/gb_runtime/src/chart.rs`](../rust/gb_runtime/src/chart.rs)
  (Datenmodell + Zeichnen), Builtins in `builtins.rs`, `CHART_DRAW` in `vm.rs`
- Demo: [`examples/154_chart.gb`](../examples/154_chart.gb)
- Tests: [`tests/test_modules_chart.py`](../tests/test_modules_chart.py) +
  Rust-`#[test]`s in `chart.rs`
