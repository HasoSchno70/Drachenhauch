# Tilemap-/Level-Editor (`examples/187_tilemap_editor.dh`)

Ein Werkzeug zum Malen von 2D-Leveln aus einem Tileset-Bild auf ein
Gitter — mit mehreren Ebenen, Objekt-Ebenen für Spawns und Zonen,
Eigenschaften je Kachel und mehreren Tilesets je Karte. Es ist selbst ein
Drachenhauch-Programm und speichert **Tiled-JSON**, genau das Format, das
das [`tiled`-Modul](module-tiled.md) mit `TILED_LOAD` liest (und das
[Tiled](https://www.mapeditor.org/) selbst öffnet).

Es löst den früheren Qt-Editor `dhtilemap` ab; Python braucht es nicht.

## Starten

In der [IDE](ide.md): **Werkzeuge → Tilemap-Editor** (oder in der
Befehlspalette „Werkzeug: Tilemap-Editor"). Von der Kommandozeile:

```
dhrt run examples/187_tilemap_editor.dh
```

Der Editor geht im Vollbild auf und beginnt mit einer leeren Karte von
40×30 Kacheln und dem eingebauten Tileset `examples/assets/editor_tileset.png`.

## Aufbau

Oben die Werkzeugleiste (Neu, Öffnen, Speichern, DH-Code, die sechs
Werkzeuge, Zurück/Vor, Schalter „Gitter" und „Nummern"). Links die Spalte
mit Tileset-Wahl, Kachel-Palette, Ebenenliste und darunter — je nach Art der
aktiven Ebene — entweder die Eigenschaften der gewählten Kachel oder die
Objektliste. Rechts die Karte, unten eine Statuszeile.

## Werkzeuge

| Werkzeug | Taste | Wirkung |
|---|---|---|
| Stift | `P` | malt die gewählte Kachel, auch im Zug |
| Radierer | `E` | leert Zellen, auch im Zug |
| Füllen | `F` | füllt die zusammenhängende Fläche gleicher Kacheln |
| Rechteck | `R` | ziehen füllt beim Loslassen das Rechteck |
| Pipette | `I` | nimmt die Kachel unter der Maus auf |
| Auswahl | `S` | zieht ein Rechteck für die Zwischenablage |

Das aktive Werkzeug ist in der Leiste farbig hinterlegt. Die Pipette schaltet
auch das **Tileset** um, zu dem die aufgenommene Kachel gehört, und rollt die
Palette so, dass sie zu sehen ist — sonst malte der nächste Strich eine
andere Kachel.

**Auswahl und Zwischenablage:** `Strg+C` kopiert die gewählte Fläche,
`Strg+X` schneidet sie aus, `Strg+V` setzt sie an der Auswahl ein (ohne
Auswahl an der Kachel unter der Maus), `Entf` leert die Auswahl. Was beim
Einfügen über den Kartenrand ragt, fällt weg — die Karte wächst dabei nicht.
All das gilt nur auf Kachel-Ebenen.

**Ansicht:** Mausrad über der Karte zoomt (1 bis 6, der Punkt unter dem
Zeiger bleibt stehen), die rechte Maustaste oder die Pfeiltasten schieben.
„Nummern" schreibt die Kachelnummer in jede Zelle (ab Zoom 2, darunter wäre
sie nicht lesbar).

**Rückgängig/Wiederholen:** `Strg+Z` / `Strg+Y` oder die Knöpfe, 16
Schritte. Ein Strich, ein Rechteck, ein Füllen ist jeweils **ein** Schritt.

## Tilesets

Die Klappliste über der Palette wählt, welches Tileset die Palette zeigt;
`+` hängt ein weiteres PNG an (höchstens acht). Die `firstgid` jedes
Tilesets vergibt die Laufzeit selbst aus der Kachelzahl — ein von Hand
gesetzter, überlappender Bereich zerstörte stumm die Zuordnung **aller**
Kacheln. Gemalt wird aus dem GID-Bereich des gezeigten Tilesets, gezeichnet
aber jede Kachel mit **ihrem** Tileset; eine Ebene darf Kacheln aus allen
enthalten.

Die Palette ist ein Ausschnitt fester Größe (8×4 Kacheln): ein größeres
Tileset rollt man mit dem Mausrad, mit Umschalt seitwärts.

Ein Tileset wieder **entfernen** gibt es bewusst nicht: die GIDs dahinter
verschöben sich, und jede damit gemalte Kachel zeigte danach stumm auf ein
anderes Bild.

Fehlt beim Öffnen das Bild eines Tilesets, meldet die Statuszeile es, und
dessen Kacheln bleiben leer, statt mit einem fremden Bild gefüllt zu werden.

## Ebenen

Die Liste zeigt die Ebenen in **Zeichenreihenfolge**: die erste liegt hinten,
weiter unten heißt weiter vorne. Darunter:

- **Anlegen**, **Umbenennen** (Name aus dem Feld darüber), **Weg** (die letzte
  Ebene bleibt),
- **Nach hinten** / **Nach vorne** — nach der Wirkung im Bild benannt, nicht
  nach der Liste. Die Kacheln bleiben bei ihrer Ebene, und ein
  aufgezeichneter Rückgängig-Schritt wandert mit, statt danach in eine
  fremde Ebene zu schreiben,
- **sichtbar** — Tiled speichert die Sichtbarkeit mit, also ist es mehr als
  eine Anzeige,
- **Objekt-Ebene** legt eine Ebene ohne Kacheln an.

## Objekt-Ebenen

Die **Art der aktiven Ebene entscheidet, was die Maus tut** — eine zweite
Werkzeugleiste wäre ein Schalter, den man vergisst. Auf einer Objekt-Ebene:

- **Ziehen** legt ein Rechteck an (Zone, Trigger),
- **Klicken** ohne Zug legt einen **Punkt** an (in Tiled ein Objekt mit
  Breite und Höhe 0, etwa ein Spawn),
- ein Klick auf ein vorhandenes Objekt **wählt es aus** statt ein neues
  darüberzulegen.

Name und Typ des nächsten Objekts stehen in den beiden Feldern links;
**Übernehmen** ändert damit das gewählte, **Objekt weg** oder `Entf` entfernt
es. Koordinaten sind Pixel wie in Tiled. Im Spiel liest man sie mit
`TILED_OBJECT_COUNT/NAME/TYPE/X/Y/WIDTH/HEIGHT`.

## Eigenschaften je Kachel

Auf einer Kachel-Ebene zeigt die linke Spalte die Eigenschaften der in der
Palette gewählten Kachel. Sie gehören dem **Tileset**, gelten also für jedes
Vorkommen dieser Kachel.

- **solid** ist ein eigenes Kästchen — genau diese Eigenschaft fragt
  `tile_collide` ab. Beim **Abwählen** wird sie **entfernt**, nicht auf
  FALSE gesetzt: sobald irgendeine Kachel ein `solid` trägt, blockieren nur
  noch die mit `solid = TRUE`; ein zurückgelassenes FALSE hielte diesen
  Schalter umgelegt, ohne dass man es sieht.
- Beliebige **Schlüssel/Werte** über die zwei Felder und **Setzen**. Der Wert
  wird gedeutet: `true`/`wahr` und `false`/`falsch` werden BOOLEAN, eine Zahl
  mit Punkt FLOAT, ohne Punkt INTEGER, alles andere Text (`1-2` bleibt Text).
- **Weg** entfernt die in der Liste gewählte Eigenschaft.

## Speichern, Öffnen, Beenden

**Speichern** (`Strg+S`) schreibt Tiled-JSON; beim ersten Mal fragt ein
Dateidialog nach dem Namen, danach wird dieselbe Datei überschrieben.
**Öffnen** (`Strg+O`) liest Tiled-JSON, auch aus Tiled selbst. **Neu** fragt
nach Breite und Höhe (4 bis 128).

Das Kreuz des Fensters und `ESC` fragen nach (Sichern | Verwerfen |
Abbrechen), wenn die Karte nicht gesichert ist; sonst endet der Editor sofort.

Die Karte im eigenen Spiel laden:

```basic
IMPORT "tiled"
DIM lvl AS TILED_MAP
lvl = TILED_LOAD("level.json")
PRINT TILED_WIDTH(lvl); "x"; TILED_HEIGHT(lvl)
```

## DH-Code

**DH-Code** schreibt drei Dateien unter **einem** Namen: das Programm
(`karte.dh`), die Karte als Tiled-JSON (`karte.json`) und jedes Tileset-Bild
(`karte_1.png`, `karte_2.png`, …). Das Bild kommt mit, obwohl die Karte
seinen Pfad nennt: der Pfad zeigt dorthin, wo das Tileset beim Bearbeiten
lag, und wer den Ordner weitergibt, hätte sonst eine Karte, die auf nichts
zeigt.

Das Programm lädt alles, ordnet jede GID ihrem Tileset über die
`firstgid`-Kette zu und zeichnet jede sichtbare Kachel-Ebene mit
`DRAWIMAGEPART`; Objekt-Ebenen zeichnet es nicht. Es läuft ohne Änderung mit
`dhrt run karte.dh`.

## Grenzen

Was die Qt-Fassung konnte und dieser Editor nicht:

- Die Kachelgröße ist fest **16×16** (`CONST KACHEL`). Eine Karte mit anderer
  Kachelgröße wird falsch zerlegt.
- Eine Karte hat höchstens **128×128** Kacheln; eine größere wird beim Öffnen
  abgelehnt. Eine vorhandene Karte lässt sich nicht vergrößern.
- Kein **Speichern unter**: nach dem ersten Speichern geht es immer in
  dieselbe Datei.
- Objekte haben **keine Eigenschaften** und lassen sich nicht per Maus
  verschieben — nur Name und Typ ändern und entfernen.
- **Rückgängig** deckt nur Kachel-Änderungen ab, nicht Ebenen, Objekte und
  Kachel-Eigenschaften.
- Kein Abblenden der übrigen Ebenen, kein Tileset entfernen (siehe oben).

## Prüfung

`tests/pruef/werkzeug_tilemap.dhtest` fährt den Editor mit echten Klicks und
Tasten: DH-Code samt Bild des erzeugten Renderers (leere und gemalte Karte),
das Deuten der Eigenschaftswerte, `solid` setzen und entfernen, Objekt-Ebenen
(ziehen, klicken, wählen, `Entf`), mehrere Tilesets mit Pipette und
Palette-Rollen, Umsortieren der Ebenen samt Rückgängig. Gelesen wird die
geschriebene Datei mit dem json-Modul nach den Regeln des Tiled-Formats, nicht
mit dem eigenen Schreiber.
