# Sprite-Editor (`gbsprites`)

Ein vollwertiger Pixel-Art-Editor fuer GameBasic. Built mit PySide6, exportiert PNG-Sheets, Animated GIFs, einzelne PNG-Frames und **Sprite-Atlas-Manifeste** -- alle direkt im Spiel via `LOADIMAGE`, `SPRITE_NEW` oder `ATLAS_LOAD` ladbar.

## Starten

```
gbsprites                        ' leerer Editor (32x32 Default)
gbsprites assets\hero.png        ' bestehende Datei oeffnen
```

Auf Windows ueber `gbsprites.cmd`. Aus dem Code-Editor heraus: Werkzeuge → Sprite-Editor.

## Tools

| Taste | Tool | Wirkung |
|---|---|---|
| **B** | Pencil | Pixel setzen (Linke Maustaste = FG, Rechte = BG) |
| **E** | Eraser | Pixel transparent setzen |
| **G** | Bucket | Flood-Fill |
| **L** | Line | Linie ziehen |
| **R** | Rect | Rechteck (gefuellt mit FG) |
| **O** | Ellipse | Ellipse (gefuellt) |
| **I** | Eyedropper | Farbe vom Pixel pickern |
| **M** | Select | Rechteck-Auswahl (Cut/Copy/Paste) |
| **V** | Move | Auswahl oder ganzes Frame verschieben |
| **W** | Magic Wand | Auswahl per Flood (zusammenhaengende Pixel mit gleicher Farbe) |
| **Y** | Spray | Pixel-Spray (zufaellige Verteilung im Brush-Radius) |

Brush-Groesse mit Tasten **1**, **2**, **3**, **4**. **X** tauscht FG/BG-Farbe.

## Datei-Operationen

| Shortcut | Aktion |
|---|---|
| `Ctrl+N` | Neu (Dialog: Groesse + Anzahl Frames) |
| `Ctrl+O` | Oeffnen (.png, .gbsprite) |
| `Ctrl+S` / `Ctrl+Shift+S` | Speichern / Speichern unter |
| `F5` | Von Disk neu laden (z.B. wenn Datei extern geaendert wurde) |
| `Ctrl+W` | Schliessen |

**Datei-Formate:**

- **`.png`** — Standard. Bei Multi-Frame wird ein horizontaler Sheet geschrieben.
- **`.gbsprite`** — natives Format (JSON + base64-RGBA pro Frame). Erhaelt Frame-Dauern und Animation-Daten. Empfohlen fuer Work-in-Progress.

## Export-Optionen

Vier Export-Pfade, je nach Use-Case:

### Sheet-PNG (`Ctrl+E`)

Schreibt alle Frames als horizontaler Sheet in ein einziges PNG. Klassisch fuer `SPRITE_NEW`:

```basic
DIM hero AS IMAGE
hero = LOADIMAGE("hero_sheet.png")
DIM sp AS SPRITE
sp = SPRITE_NEW(hero, 16, 16)          ' Frame-Groesse
SPRITE_ADD_ANIM(sp, "idle", 0, 3, 8)   ' Frames 0..3 mit 8 fps
SPRITE_PLAY(sp, "idle")
```

### Sprite-Atlas (`Ctrl+Shift+E`) — neu

Schreibt **PNG + JSON-Manifest** gemeinsam. Das JSON beschreibt jedes Frame mit Namen + Rect. Sprite-Namen sind standardmaessig `<dateiname>_<index>` (also bei `tiles.png`: `tiles_0`, `tiles_1`, ...).

**Frame-Namen:** Rechtsklick auf ein Frame in der Frame-Liste → **Umbenennen...** vergibt einen eigenen Namen. Benannte Frames nutzen diesen Namen direkt als Sprite-ID im Atlas (statt `<dateiname>_<index>`) — `ATLAS_DRAW(atlas, "idle", x, y)` statt `"hero_0"`. Doppelte Namen werden beim Export eindeutig gemacht (Suffix `_<index>`). Der Name wird in `.gbsprite`-Dateien mitgespeichert (Format-Version 3; aeltere Dateien laden mit leerem Namen).

In GameBasic dann:

```basic
DIM atlas AS SPRITE_ATLAS
atlas = ATLAS_LOAD("tiles.json")
ATLAS_DRAW(atlas, "tiles_0", x, y)

' Oder fuer Batch-Rendering (z.B. Tilemap):
BATCH_DRAW(atlas, "tiles_3", x, y)
BATCH_FLUSH()
```

Vorteil ggue Sheet-PNG: **benannte Sub-Sprites** statt Index-Rechnerei, und die Engine-Batch-API (`BATCH_DRAW`/`BATCH_FLUSH`) profitiert vom Atlas-Format. Workflow-Loop ist geschlossen — der Editor schreibt, was die Engine direkt lesen kann.

### Animation-GIF (`Ctrl+G`)

Schreibt alle Frames als animiertes GIF mit transparentem Hintergrund. Fuer Vorschau, Doku, Itch-Page-Screenshots.

Bei unterschiedlich langen Frames fragt der Dialog, ob die individuellen Dauern verwendet werden sollen oder eine einheitliche FPS gilt.

### Frame als PNG

Nur das aktuelle Frame als einzelnes PNG. Praktisch fuer Icons, einzelne Charakter-Sprites, Logos.

## Multi-Frame und Animation

Der Editor unterstuetzt von Anfang an Multi-Frame-Animationen. Frames-Panel rechts zeigt alle Frames als Thumbnails; Klick wechselt.

| Aktion | Wirkung |
|---|---|
| Neues Frame | leeres Frame nach dem aktuellen |
| Frame duplizieren | Kopie nach dem aktuellen (Basis fuer Tweens) |
| Frame loeschen | aktuelles Frame raus (mind. 1 Frame bleibt) |
| `F2`..`F9` | Frame N direkt anspringen |
| `Ctrl+P` | Animation-Preview-Fenster (Live-Loop) |
| Frames umkehren | Reihenfolge invertieren |
| Ping-Pong (anfuegen) | aus `0,1,2,3` wird `0,1,2,3,2,1` (typisch fuer Walk-Cycles) |
| Auf aktuelles Frame reduzieren | alle anderen Frames verwerfen |
| Frames zusammenfuegen (Composite) | alle Frames uebereinander stempeln (Sandbox fuer Pixel-Art) |

**Frame-Dauer** pro Frame einstellbar (Statusbar links). Fuer die Animation-Preview und den GIF-Export wird sie verwendet.

## Onion-Skinning

Toggle ueber Aktion "Onion-Skin". Im aktiven Zustand sind vorherige Frames blau, naechste rot durchscheinend hinter dem aktuellen — klassisches Pattern fuer Animation-Konsistenz (z.B. Walk-Cycle).

## Symmetrie-Modus

| Shortcut | Wirkung |
|---|---|
| `Ctrl+Shift+X` | X-Symmetrie (alles links wird auch rechts gemalt) |
| `Ctrl+Shift+Y` | Y-Symmetrie (oben/unten gespiegelt) |

Ideal fuer Charakter-Sprites, Symbole, Logos.

## Tile-Preview

**Taste T**: 3×3-Tiling-Preview daneben einblenden. Wichtig fuer Tilemap-Sprites — sieht das Tile gekachelt gut aus oder zeigt sich eine sichtbare Naht?

## Palette

| Aktion | Wirkung |
|---|---|
| Palette aus Sprite extrahieren | sammelt alle verwendeten Farben des Sprites in die Palette |
| Palette laden (`.gpl`) | GIMP-kompatible Palette importieren (NES-/Gameboy-/PICO-8-Sets etc.) |
| Palette speichern (`.gpl`) | aktuelle Palette als GPL exportieren |
| Farbe ersetzen... | alle Pixel einer Farbe gegen eine andere tauschen (dialog) |

## Canvas + Zoom

| Aktion | Shortcut |
|---|---|
| Zoom + | `Ctrl++` |
| Zoom - | `Ctrl+-` |
| Zoom 100% | `Ctrl+0` |
| Canvas-Groesse aendern... | Dialog mit Anker-Position |
| Auf Inhalt zuschneiden | Crop auf Bounding-Box des sichtbaren Pixels |
| Grid umschalten | Pixel-Grid an/aus |

## Editier-Operationen

| Aktion | Shortcut |
|---|---|
| Undo / Redo | `Ctrl+Z` / `Ctrl+Y` (oder `Ctrl+Shift+Z`) |
| Ausschneiden / Kopieren / Einfuegen | `Ctrl+X` / `Ctrl+C` / `Ctrl+V` |
| Auswahl loeschen | `Delete` |
| Auswahl aufheben | `Escape` |
| Horizontal spiegeln | `Ctrl+H` |
| Vertikal spiegeln | `Ctrl+J` |
| 90° rechts | `Ctrl+.` |
| 90° links | `Ctrl+,` |
| Frame leeren | — (Menue) |

## Asset-Browser

Linker Panel-Bereich. Zeigt alle Bilder/Sprites im Projekt-Verzeichnis. Doppelklick lädt eine Datei. Hilfreich fuer Sets von zusammenhaengenden Sprites (Player-Frames, Enemy-Pack).

## Sheet-Import

Datei → Oeffnen mit einem PNG, das ein bestehender Sheet ist: der Editor fragt nach Frame-Groesse (z.B. 16×16) und zerlegt es in Einzel-Frames. Praktisch fuer Sprites aus anderen Editors importieren (Aseprite, Piskel).

## Test-Sprite

Aktion "Test-Sprite" im Datei-Menue: oeffnet ein kleines Pygame-Fenster, das die Animation rendert (mit aktuellen Frame-Dauern). Schneller Sanity-Check, ob das Timing stimmt.

## Code-Export

Aktion "Code kopieren" liefert die `SPRITE_NEW`/`ADD_ANIM`/`PLAY`-Sequenz fuer den aktuellen Sprite in die Zwischenablage. Spart manuelle Tipparbeit beim Wechsel zum Code-Editor.

## Typische Workflows

### Charakter-Animation (4-Frame-Walk-Cycle)

1. `Ctrl+N` neu, 16×16 mit 4 Frames
2. Frame 0 zeichnen (idle), Frame 1 (step left), Frame 2 (idle), Frame 3 (step right)
3. `Ctrl+P` zum Preview ansehen
4. `Ctrl+E` als Sheet-PNG exportieren
5. Im Spiel: `SPRITE_NEW(LOADIMAGE("hero.png"), 16, 16); SPRITE_ADD_ANIM(sp, "walk", 0, 3, 8)`

### Tile-Set fuer Platformer

1. `Ctrl+N` neu, 16×16 mit z.B. 8 Frames (= 8 verschiedene Tiles)
2. Jedes Frame = ein Tile-Typ (Gras, Wasser, Stein, ...)
3. `T` einschalten fuer Tile-Preview — sicherstellen dass die Tiles gut kacheln
4. **`Ctrl+Shift+E` als Sprite-Atlas exportieren** (PNG + JSON)
5. Im Spiel: `atlas = ATLAS_LOAD("tiles.json")` + `BATCH_DRAW(atlas, "tiles_0", x, y)` fuer jedes Tile + `BATCH_FLUSH()` am Ende

### Particle / Explosion-Sprite-Sheet

1. 8 Frames mit explodierendem Effekt
2. Onion-Skin einschalten, um sichtbare Vorlaeufer/Nachfolger zu sehen
3. Frame-Dauer pro Frame anpassen (Beginn schnell, Ende langsam)
4. `Ctrl+G` als GIF fuer Vorschau
5. `Ctrl+E` als Sheet-PNG fuer Spiel

### Schnelle Variationen via Color-Replace

1. Basis-Sprite zeichnen
2. `Ctrl+S` speichern (als `.gbsprite`)
3. `Ctrl+Shift+S` als `enemy_blue.png`
4. "Farbe ersetzen..." — rot durch blau ersetzen
5. Wiederholen fuer gruen, gelb, ...

## Eigene Klassen-Architektur (fuer Tests + Erweiterung)

Falls du den Editor erweitern willst:

| Datei | Inhalt |
|---|---|
| [`gamebasic/spriteeditor_qt.py`](../gamebasic/spriteeditor_qt.py) | UI-Schicht: SpriteEditorWindow, ColorPanel, FramesPanel, Canvas, Dialogs |
| [`gamebasic/spriteeditor/document.py`](../gamebasic/spriteeditor/document.py) | Datenmodell: `SpriteDoc`, `Frame`. Save/Load (PNG, .gbsprite, GIF, Atlas) |
| [`gamebasic/spriteeditor/tools.py`](../gamebasic/spriteeditor/tools.py) | Pixel-Tools (Pencil, Eraser, Bucket, Line, ...) |
| [`gamebasic/spriteeditor/tool_context.py`](../gamebasic/spriteeditor/tool_context.py) | `ToolHost`-Protocol (welche app-Attribute die Tools brauchen) |
| [`gamebasic/spriteeditor/icons.py`](../gamebasic/spriteeditor/icons.py) | Programmatisch gerenderte Toolbar-Icons (kein PNG-Asset noetig) |

Tools-API ist klar dokumentiert (siehe `tools.py`-Header). Neue Tools: subclass von `Tool`, `name` setzen, `begin/move/end` implementieren, in `SpriteEditorWindow._setup_tools()` registrieren.

Tests: [`tests/test_spriteeditor_document.py`](../tests/test_spriteeditor_document.py), [`tests/test_spriteeditor_tools.py`](../tests/test_spriteeditor_tools.py), [`tests/test_spriteeditor_tool_context.py`](../tests/test_spriteeditor_tool_context.py).
