# Modul `tiled`

Karten im [Tiled](https://www.mapeditor.org/)-JSON-Format **lesen, aendern, anlegen und schreiben**. Tiled ist Industriestandard fuer 2D-Level-Design — fast jeder Indie-2D-Engine-Workflow geht durch Tiled.

Bis 2026-08-30 war das Modul ein reiner Leser: eine geladene Karte liess sich im Speicher aendern, aber nicht zurueckschreiben, und eine neue gar nicht erst anlegen. Damit war alles, was Karten **baut** statt sie nur zu benutzen — ein Editor, ein Generator, ein Umwandlungswerkzeug — ausgeschlossen. Die Abschnitte [Karten anlegen und speichern](#karten-anlegen-und-speichern) und [Ebenen aendern](#ebenen-aendern) schliessen das.

```basic
IMPORT "tiled"
```

**Workflow:**

1. In Tiled das Level bauen (Tilesets, Layer, Object-Layer mit Custom-Properties)
2. **File → Save As → JSON Map** (Tiled speichert nativ als `.tmx`, kann aber auch direkt JSON schreiben)
3. Im Spiel: `m = TILED_LOAD("level.json")`
4. Layer iterieren, Objekte spawnen, Tile-Properties pruefen

**TMX (XML) wird in v1 nicht unterstuetzt** — saving as JSON ist die einzige Sache, die der Artist anders machen muss als gewohnt.

## Übersicht

### Map-Metadata

| Funktion | Rueckgabe |
|---|---|
| `TILED_LOAD(path$)` | TILED_MAP |
| `TILED_WIDTH(m)` / `TILED_HEIGHT(m)` | INTEGER (in Tiles) |
| `TILED_TILE_WIDTH(m)` / `TILED_TILE_HEIGHT(m)` | INTEGER (Pixel pro Tile) |

### Layer

| Funktion | Rueckgabe | Bedeutung |
|---|---|---|
| `TILED_LAYER_COUNT(m)` | INTEGER | wie viele Ebenen hat die Karte? |
| `TILED_LAYER_NAME(m, idx)` | STRING | Name der Ebene (Index ab 0) |
| `TILED_LAYER_TYPE(m, idx)` | `"tile"`, `"object"`, `"image"` | Art der Ebene |
| `TILED_LAYER_INDEX(m, name$)` | INTEGER (-1 wenn nicht da) | Ebene ueber ihren Namen finden |
| `TILED_LAYER_WIDTH/HEIGHT(m, idx)` | INTEGER (nur Tile-Layer) | Groesse der Ebene in Kacheln |

### Tile-Daten

| Funktion | Rueckgabe | Bedeutung |
|---|---|---|
| `TILED_TILE_AT(m, layer_idx, tx, ty)` | INTEGER (GID, 0 = leer; OOB = 0) | welche Kachel liegt an dieser Stelle? |
| `TILED_TILE_SET(m, layer_idx, tx, ty, gid)` | INTEGER (alte GID). 0 = Tile loeschen. OOB = no-op. | Kachel setzen |
| `TILED_TILE_PROP_BOOL(m, gid, key$)` | BOOLEAN (FALSE wenn nicht gesetzt) | Eigenschaft der Kachel lesen -- in Tiled je Kachel gepflegt (`solid`, `damage`, ...) |
| `TILED_TILE_PROP_INT(m, gid, key$)` | INTEGER (0 wenn nicht gesetzt) | Eigenschaft der Kachel als ganze Zahl |
| `TILED_TILE_PROP_FLOAT(m, gid, key$)` | FLOAT | Eigenschaft der Kachel als Kommazahl |
| `TILED_TILE_PROP_STRING(m, gid, key$)` | STRING | Eigenschaft der Kachel als Text |
| `TILED_TILE_HAS_PROP(m, gid, key$)` | BOOLEAN |  |

### Object-Layer

| Funktion | Rueckgabe | Bedeutung |
|---|---|---|
| `TILED_OBJECT_COUNT(m, layer_name$)` | INTEGER | wie viele Objekte hat diese Objekt-Ebene? |
| `TILED_OBJECT_NAME(m, layer_name$, idx)` | STRING | Name des Objekts (Index ab 0) |
| `TILED_OBJECT_TYPE(m, layer_name$, idx)` | STRING (Tiled "type" oder "class") | Typ des Objekts, wie in Tiled eingetragen |
| `TILED_OBJECT_X/Y(...)` | FLOAT | Position des Objekts in Pixeln |
| `TILED_OBJECT_WIDTH/HEIGHT(...)` | FLOAT | Groesse des Objekts in Pixeln |
| `TILED_OBJECT_PROP_BOOL/INT/FLOAT/STRING(...)` | Custom Property |  |

### Tileset

| Funktion | Rueckgabe | Bedeutung |
|---|---|---|
| `TILED_TILESET_COUNT(m)` | INTEGER | wie viele Tilesets bringt die Karte mit? |
| `TILED_TILESET_IMAGE(m, idx)` | STRING (absoluter Pfad zum Tileset-Bild) | Bilddatei des Tilesets |
| `TILED_TILESET_FIRSTGID(m, idx)` | INTEGER | erste GID dieses Tilesets -- damit ordnet man eine GID ihrem Tileset zu |
| `TILED_FILL_RECT(m, layer_idx, tx, ty, w, h, gid)` | INTEGER | Rechteck mit einer GID füllen; liefert die Zahl geänderter Kacheln |
| `TILED_REPLACE(m, layer_idx, from_gid, to_gid)` | INTEGER | jede GID `from_gid` durch `to_gid` ersetzen — Tileset tauschen ohne Schleife |
| `TILED_COUNT_GID(m, layer_idx, gid)` | INTEGER | wie oft kommt diese GID in der Ebene vor? |
| `TILED_FLOOD_FILL(m, layer_idx, tx, ty, gid)` | INTEGER | Farbeimer: die zusammenhängende Fläche ab `(tx,ty)` umfärben |

## Karten anlegen und speichern

| Funktion | Rueckgabe | Bedeutung |
|---|---|---|
| `TILED_NEW(breite, hoehe, kachel_w, kachel_h)` | TILED_MAP | eine leere Karte anlegen -- ohne Ebene und ohne Tileset |
| `TILED_ADD_LAYER(m, name$)` | INTEGER (Index) | eine leere Kachel-Ebene anhaengen |
| `TILED_ADD_TILESET(m, bild$, kachelzahl)` | INTEGER (Index) | ein Tileset anhaengen; die `firstgid` vergibt die Laufzeit selbst |
| `TILED_TILESET_TILES(m, idx)` | INTEGER | wie viele Kacheln hat dieses Tileset? |
| `TILED_ADD_OBJECT_LAYER(m, name$)` | INTEGER (Index) | eine Objekt-Ebene anhaengen |
| `TILED_ADD_OBJECT(m, ebene$, name$, typ$, x, y, w, h)` | INTEGER (Index) | ein Objekt in eine Objekt-Ebene legen |
| `TILED_TILE_SET_PROP(m, gid, key$, wert)` | — | Eigenschaft einer Kachel setzen |
| `TILED_TILE_REMOVE_PROP(m, gid, key$)` | — | Eigenschaft einer Kachel entfernen |
| `TILED_OBJECT_SET_PROP(m, ebene$, idx, key$, wert)` | — | Eigenschaft eines Objekts setzen |
| `TILED_OBJECT_REMOVE_PROP(m, ebene$, idx, key$)` | — | Eigenschaft eines Objekts entfernen |
| `TILED_REMOVE_OBJECT(m, ebene$, idx)` | — | ein Objekt entfernen |
| `TILED_OBJECT_SET_NAME(m, ebene$, idx, name$)` | — | ein Objekt umbenennen |
| `TILED_OBJECT_SET_TYPE(m, ebene$, idx, typ$)` | — | seinen Typ aendern |
| `TILED_OBJECT_SET_RECT(m, ebene$, idx, x, y, w, h)` | — | Lage und Groesse aendern |
| `TILED_TILE_PROP_KEYS(m, gid)` | ARRAY OF STRING | welche Eigenschaften hat diese Kachel? |
| `TILED_OBJECT_PROP_KEYS(m, ebene$, idx)` | ARRAY OF STRING | dasselbe fuer ein Objekt |
| `TILED_SAVE(m, pfad$)` | — | die Karte als Tiled-JSON schreiben |

```basic
IMPORT "tiled"
DIM m AS TILED_MAP
m = TILED_NEW(40, 30, 16, 16)
DIM ts AS INTEGER : ts = TILED_ADD_TILESET(m, "tiles.png", 32)
DIM boden AS INTEGER : boden = TILED_ADD_LAYER(m, "Boden")
TILED_FILL_RECT(m, boden, 0, 0, 40, 30, 1)
TILED_SAVE(m, "level.json")
```

**Die `firstgid` wird nicht von Hand gesetzt.** Sie ist die erste globale ID des
Tilesets; das erste bekommt 1, jedes weitere schliesst hinter dem vorigen an.
Ueberlappende Bereiche waeren die haeufigste und unangenehmste Fehlerquelle --
sie zerstoeren stillschweigend die Zuordnung **aller** Kacheln, ohne dass
irgendetwas eine Fehlermeldung gibt. Deshalb braucht `TILED_ADD_TILESET` die
Kachelzahl: nur damit laesst sich die naechste GID ausrechnen.

**Geschrieben wird echtes Tiled-JSON** — eingebettete Tilesets, `type: "map"`,
Kacheldaten als Zahlenliste. Nachgeprueft ist das nicht am eigenen Leser, sondern
an einem **fremden**: `drachenhauch/tilemap/document.py`, dem Datenmodell des
Qt-Editors `dhtilemap` (`tests/test_tiled_schreiben.py`). Ein Format, das nur sein
eigener Schreiber wieder liest, ist nicht geprueft, sondern nur in sich stimmig.

### Objekt-Ebenen und Eigenschaften anlegen

```basic
DIM sp AS INTEGER : sp = TILED_ADD_OBJECT_LAYER(m, "spawns")
DIM held AS INTEGER
held = TILED_ADD_OBJECT(m, "spawns", "held", "spawn", 32.0, 48.0, 16.0, 16.0)
TILED_OBJECT_SET_PROP(m, "spawns", held, "leben", 3)

TILED_TILE_SET_PROP(m, 5, "solid", TRUE)      ' GID, nicht lokale Nummer
TILED_TILE_SET_PROP(m, 5, "damage", 10)
```

**Ein Setzer statt vier.** Die Leser sind typisiert
(`TILED_TILE_PROP_BOOL/INT/FLOAT/STRING`), weil dort der Aufrufer sagt, was er
erwartet. Beim Schreiben ist das unnoetig: Drachenhauch unterscheidet BOOLEAN,
INTEGER, FLOAT und STRING schon im Wert, also traegt der Wert seinen Typ mit
sich. Alles andere (ARRAY, MAP, Handles) wird abgelehnt — Tiled kennt genau
diese vier Arten, und ein Feld wuerde beim Speichern still zu Text zerfallen.

Die **GID** ist auch beim Setzen die Adresse, nicht die lokale Nummer — wie beim
Lesen. Gespeichert wird die Eigenschaft beim Tileset unter der lokalen Nummer,
aber das ist Buchhaltung, die ein Programm nicht kennen muss. Flip-Bits werden
dabei genauso maskiert wie beim Lesen; sonst legte eine gespiegelte Kachel ihre
Eigenschaft woanders ab als eine ungespiegelte.

`TILED_OBJECT_SET_RECT` setzt **alle vier** Werte auf einmal, nicht vier
einzelne Setzer: in einem Editor gehoeren Verschieben und Groessenaendern zur
selben Geste, und ein halb nachgezogenes Rechteck ist ein Fehler, den man
nicht sieht.

`TILED_REMOVE_OBJECT` laesst die Indizes **dahinter aufruecken** — wie jede
Liste. Wer sich einen gemerkt hat, holt ihn danach neu; das ist die einzige
Ueberraschung daran.

**Was hat diese Kachel eigentlich?** `TILED_TILE_HAS_PROP` beantwortet nur
eine Frage, die man schon kennt. `TILED_TILE_PROP_KEYS` (und
`TILED_OBJECT_PROP_KEYS`) liefern die Schluessel selbst — **sortiert**, weil
die Ablage eine HashMap ist und eine Liste, die sich bei jedem Auffrischen
neu mischt, nicht zu bedienen waere.

Ein `TILED_TILE_REMOVE_PROP`, das die letzte Eigenschaft einer Kachel nimmt,
entfernt auch ihren Eintrag ganz: `TILED_SAVE` fuehrt jede Kachel auf, die einen
hat, und eine mit leerer Liste waere Rauschen in der Datei.

**Grenzen:** `TILED_NEW` legt nur orthogonale, endliche Karten an (kein
isometrisch, kein `infinite`). Objekte sind immer Rechtecke — Polygone,
Ellipsen und Punkte (Tiled: `"point": true`) lassen sich nicht anlegen, ein
Objekt mit Breite und Hoehe 0 liest der Qt-Editor allerdings als Punkt.
Objekt-Ebenen lassen sich nicht wieder in Kachel-Ebenen verwandeln.
Sehr grosse Karten werden abgelehnt (ueber 4 Millionen Kacheln), damit ein
Tippfehler in der Groesse nicht den Speicher frisst.

## Ebenen aendern

| Funktion | Rueckgabe | Bedeutung |
|---|---|---|
| `TILED_LAYER_RENAME(m, idx, name$)` | — | eine Ebene umbenennen |
| `TILED_LAYER_VISIBLE(m, idx)` | BOOLEAN | ist die Ebene eingeblendet? |
| `TILED_LAYER_SET_VISIBLE(m, idx, an)` | — | Ebene ein- oder ausblenden |
| `TILED_REMOVE_LAYER(m, idx)` | — | eine Ebene entfernen |
| `TILED_MOVE_LAYER(m, von, nach)` | — | eine Ebene an eine andere Stelle setzen |

Die Sichtbarkeit ist **nicht nur Anzeige** -- Tiled speichert sie in der Datei.
Ohne `TILED_LAYER_SET_VISIBLE` liess sich eine ausgeblendete Ebene also gar nicht
so speichern: sie kam beim naechsten Laden sichtbar zurueck.

**Die Reihenfolge ist die ZEICHENreihenfolge**, nicht nur eine Anzeige-Sache:
Ebene 0 liegt hinten, die letzte vorne. `TILED_MOVE_LAYER` nimmt die Ebene
heraus und setzt sie an `nach` wieder ein (0 = ganz hinten) -- es TAUSCHT nicht,
denn bei einem Sprung ueber mehrere Stellen waere das etwas anderes.

**Achtung, Indizes verschieben sich.** `TILED_REMOVE_LAYER` rueckt alle Ebenen
dahinter auf, und `TILED_MOVE_LAYER` stellt sie um. Wer sich eine Ebenennummer
gemerkt hat, muss sie danach neu holen (`TILED_LAYER_INDEX(m, name$)`). Der
Namensindex wird bei Umbenennen, Entfernen und Verschieben mitgefuehrt -- ein
alter Name liefert danach -1.

## Konzept: GIDs vs. lokale Tile-IDs

Tiled vergibt **globale Tile-IDs (GIDs)** ueber alle Tilesets hinweg. Wenn ein Tileset mit `firstGid=1` 8 Tiles enthaelt, sind dessen Tile-IDs `1..8`. Ein zweites Tileset mit `firstGid=9` setzt fort.

GID = 0 bedeutet "leeres Tile" — in jeder Layer-Cell, die nicht gefuellt ist, steht 0.

`TILED_TILE_AT(...)` liefert immer GIDs. Wer das Tile als Sub-Sprite aus einem `SPRITE_ATLAS` rendern will, muss die Mappung GID → Atlas-Name selbst pflegen (siehe Beispiel unten) oder den Atlas so anlegen, dass die Sprite-Indizes mit den GIDs uebereinstimmen.

## Custom Properties

Das groesste Tiled-Killer-Feature: **alles** kann Custom-Properties haben — Tiles, Objects, Layers, sogar die ganze Map. Im Editor: Rechtsklick → "Custom Properties" → Neuer Property mit Name + Type (bool, int, float, string, color).

**Beispiel-Workflow fuer Platformer:**

- Tile-Type `grass` bekommt Property `solid: true` (alle Wand-Tiles)
- Tile-Type `spikes` bekommt `damage: 1` und `solid: false` (Trigger ohne Block)
- Object-Type `enemy` bekommt `hp: 30`, `speed: 1.2`, `pattern: "patrol"`
- Object-Type `door` bekommt `target_level: "level_2"`

Im Code:

```basic
' Bei jedem Spawn-Objekt
DIM hp AS INTEGER
hp = TILED_OBJECT_PROP_INT(m, "enemies", i, "hp")

' Beim Tile-Test (in einer Damage-System-Schleife):
IF TILED_TILE_PROP_INT(m, gid, "damage") > 0 THEN
    HurtPlayer(...)
END IF
```

## Object-Layer Pattern

Tiled erlaubt freie Rechtecke / Polygone / Points / Ellipsen als "Objects" in einem dedizierten Layer. Klassische Use-Cases:

- **Spawn-Punkte** — Player-Start, Enemy-Spawn, Item-Spawn
- **Trigger-Zonen** — Door-Trigger, Speicherpunkt, Cutscene-Auslöser
- **Wegpunkte** — Patrol-Pfad, AI-Navigation

Beim Laden iteriert man durch alle Objects im Layer und spawnt entsprechend:

```basic
DIM n AS INTEGER
n = TILED_OBJECT_COUNT(m, "spawns")
DIM i AS INTEGER
FOR i = 0 TO n - 1
    DIM obj_type AS STRING
    obj_type = TILED_OBJECT_TYPE(m, "spawns", i)
    SELECT CASE obj_type
        CASE "player_start"
            player_x = TILED_OBJECT_X(m, "spawns", i)
            player_y = TILED_OBJECT_Y(m, "spawns", i)
        CASE "enemy"
            SpawnEnemy(
                TILED_OBJECT_X(m, "spawns", i),
                TILED_OBJECT_Y(m, "spawns", i),
                TILED_OBJECT_PROP_INT(m, "spawns", i, "hp"))
        CASE "door"
            SpawnDoor(
                TILED_OBJECT_X(m, "spawns", i),
                TILED_OBJECT_Y(m, "spawns", i),
                TILED_OBJECT_PROP_STRING(m, "spawns", i, "target"))
    END SELECT
NEXT
```

## Layer-Konvention

Typisches Setup:

| Layer-Name | Type | Zweck |
|---|---|---|
| `background` | tile | Parallax-Hintergrund |
| `decor` | tile | nicht-solid Deko-Tiles |
| `ground` | tile | **Collision-Layer** (mit solid-Tiles) |
| `foreground` | tile | vor dem Player (Stein-Vordergrund) |
| `spawns` | object | Spawn-Punkte, Trigger |
| `paths` | object | Patrol-Wegpunkte |

Im Spiel: das `tile_collide`-Modul nimmt EIN Tile-Layer als Collision-Layer (typisch `ground`). Andere Layer werden nur gerendert.

## Tile-Rendering: Pattern mit Sprite-Atlas

Tile-Layer rendern: pro Frame ueber alle Tile-Cells iterieren, gefuellte Tiles aus dem Atlas zeichnen. (`BATCH_DRAW` unten ist nur ein Zweitname fuer `ATLAS_DRAW` und `BATCH_FLUSH` ein No-Op -- siehe Grafik-Doku; sie buendeln nichts.)

```basic
DIM atlas AS SPRITE_ATLAS
atlas = ATLAS_LOAD("assets/tiles_atlas.json")

' Tile-Namen pro Tile-ID (Atlas-Sprite-Namen)
DIM names[8] AS STRING
names[0] = "grass"     ' GID 1
names[1] = "stone"     ' GID 2
' ...

' Pro Frame:
DIM tx AS INTEGER
DIM ty AS INTEGER
FOR ty = 0 TO TILED_HEIGHT(m) - 1
    FOR tx = 0 TO TILED_WIDTH(m) - 1
        DIM gid AS INTEGER
        gid = TILED_TILE_AT(m, 0, tx, ty)    ' 0 = ground-Layer-Index
        IF gid > 0 THEN
            BATCH_DRAW(atlas, names[gid - 1],
                       tx * TILED_TILE_WIDTH(m),
                       ty * TILED_TILE_HEIGHT(m))
        END IF
    NEXT
NEXT
BATCH_FLUSH()
```

Bei grossen Maps (z.B. 100×100) + Camera-Scrolling lohnt es sich, nur den sichtbaren Bereich zu iterieren — Pseudo:

```basic
DIM first_tx AS INTEGER
DIM last_tx AS INTEGER
first_tx = MAX(0, cam_x / TW)
last_tx  = MIN(TILED_WIDTH(m) - 1, (cam_x + screen_w) / TW + 1)
' ... analog fuer ty ...
```

## Externe Tilesets

Tiled erlaubt Tilesets als externe `.json`-Datei (zur Wiederverwendung in mehreren Maps). Die Map referenziert sie via `"source": "tileset.json"`. Das Modul loest das automatisch auf — du musst nichts extra tun.

Was **nicht** unterstuetzt wird:
- TMX (XML-Format) — Map als JSON speichern
- Base64/zlib/gzip-codierte Tile-Daten — In Tiled: `Edit → Preferences → "Store tile layer data as: CSV"` (Default)
- Group-Layer — werden ignoriert, ihre Sub-Layer nicht expandiert
- Hex / Iso-Maps — Modul ist auf orthogonale Maps zugeschnitten

## Externer Typ

| Typ | Wirkung |
|---|---|
| `TILED_MAP` | Geladene Tiled-Map. `DIM m AS TILED_MAP` |

## Ein Editor als Beispiel

[`examples/187_tilemap_editor.dh`](../examples/187_tilemap_editor.dh) ist ein
vollstaendiger Tilemap-Editor in Drachenhauch: Stift/Radierer/Fuellen/Rechteck/
Pipette/Auswahl, Zwischenablage, Rueckgaengig, Ebenen, Laden und Speichern.
Er benutzt genau die Befehle dieses Abschnitts und ist der kuerzeste Weg,
sie im Zusammenspiel zu sehen.

## Beispiel

[examples/77_tiled_platformer.dh](../examples/77_tiled_platformer.dh) zeigt das volle Pattern: Tiled-Level laden, Tile-Layer batchen, Object-Layer fuer Player-Spawn nutzen, Tile-Properties als Collision-Source verwenden.

Vollstaendiger Workflow inklusive Kollision: siehe [tile_collide-Modul](module-tile-collide.md).

## In der nativen Runtime (dhrt)

`tiled` laeuft nativ (immer dabei, JSON-Loader via serde_json) und ist **bit-identisch** zu den Python-Pfaden — inkl. External-Tilesets, Tile-/Object-Properties und Bulk-Ops (`TILED_FILL_RECT`/`REPLACE`/`COUNT_GID`/`FLOOD_FILL`).
