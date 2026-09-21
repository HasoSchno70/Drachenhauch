# Anim-FSM-Editor — Zustandsmaschinen für Sprite-Animationen

Ein **Knoten-Graph-Editor** für Animations-Zustandsmaschinen im Stil von
Unitys Mecanim, geschrieben in Drachenhauch selbst:
`examples/198_anim_fsm_editor.dh`. Knoten sind **Zustände** (an eine
Sprite-Animation gebunden), Pfeile sind **Übergänge** mit UND-verknüpften
Bedingungen. Das Ergebnis ist eine `.dhanim`-Datei (JSON), die das
Laufzeitmodul [`animfsm`](module-animfsm.md) per `ANIM_FSM_LOAD` lädt. Im
Spiel schaltet man damit keine Animationen mehr von Hand um — man setzt nur
Parameter, die Maschine entscheidet den Zustand.

Bis 2026-09 gab es dafür den Qt-Editor `dhanim` (Python). Er ist mit dem
Python-Teil entfernt; 198 liest und schreibt dasselbe Format.

## Start

In der [IDE](ide.md) über das Menü **Werkzeuge → Anim-FSM-Editor** (oder die
Befehlspalette, „Werkzeug: Anim-FSM-Editor"). Von der Kommandozeile:

```
dhrt run examples/198_anim_fsm_editor.dh
dhrt run examples/198_anim_fsm_editor.dh -- hero.dhanim
```

Ein relativer Dateiname gilt vom Ort des Aufrufs aus (`DHRT_START_DIR`).
**Ohne Datei öffnet der Editor die Beispielmaschine**
`examples/anim_demo.dhanim` — ein Plattformer-Held mit idle/run/jump/fall,
den Parametern `speed`/`grounded`/`jump` und allen Übergangsarten —, damit
man sofort sieht, wie ein fertiger Graph aussieht. `Strg+N` leert.

## Oberfläche

- **Mitte — der Graph:** Zustände als Knoten, Übergänge als Pfeile mit
  Spitze und Beschriftung (Hin- und Rückweg zwischen denselben Knoten liegen
  seitlich versetzt). Ein Eingangspfeil markiert den **Startzustand**, die
  Pille **„Any State"** ist die Quelle für Übergänge von überall.
- **Links — Dokument und Parameter:** Sprite-Blatt (Bildpfad,
  Einzelbild-Breite und -Höhe, Vorschau-Maßstab) und die Liste der
  **Parameter** (`bool`/`float`/`int`/`trigger`) mit Vorgabewert.
- **Rechts — Inspektor:** je nach Auswahl der Zustand (Name, Animation,
  Bild von/bis, Bilder je Sekunde, endlos, Startzustand) oder der Übergang
  („erst wenn fertig" und bis zu **sechs Bedingungen** aus Parameter,
  Operator und Wert).

## Bedienung

| Aktion | so |
|---|---|
| Zustand anlegen | Doppelklick auf freie Fläche, oder `Einfg` |
| Zustand verschieben | Knoten ziehen (Raster 8); Pfeiltasten schieben den gewählten |
| **Übergang ziehen** | **mit der rechten Maustaste** von einem Knoten auf einen anderen ziehen (auch von „Any State"); oder `L` für den Verbinden-Modus, dann mit der linken |
| Übergang bearbeiten | auf den Pfeil klicken → Inspektor; `+ Bedingung` fügt eine Zeile an, `x` nimmt sie weg |
| Zustand bearbeiten | Knoten klicken → Inspektor |
| Startzustand | im Inspektor, oder **Bearbeiten → Als Startzustand** |
| Parameter | links `+`/`-`, Name/Typ/Vorgabe; Umbenennen zieht die Bedingungen mit, Löschen nimmt sie heraus |
| Löschen | `Entf` — ein Zustand nimmt seine Übergänge mit |
| Rückgängig / Wiederholen | `Strg+Z` / `Strg+Y` |
| Neu / Öffnen / Sichern / Sichern unter | `Strg+N` / `Strg+O` / `Strg+S` / `Strg+Umschalt+S` |
| Vorschau | `F5` |
| Beenden | `Strg+Q` oder das Kreuz des Fensters |

Beim Beenden mit ungesicherten Änderungen fragt der Editor nach
(Sichern | Verwerfen | Abbrechen); ist alles gesichert, endet er sofort.

### Vorschau (F5)

`F5` schreibt `<name>_vorschau.dh` neben die `.dhanim` und startet es mit
`dhrt`: links je Parameter ein Regler (über das `ui`-Modul — Schieber für
`float`/`int`, Kästchen für `bool`, Knopf für `trigger`), rechts das Sprite,
das den **aktuellen Zustand** spielt. So testet man die Übergänge, ohne
Spielcode zu schreiben. Das Sprite-Blatt wird wie eingetragen, neben der
`.dhanim` oder vom Startordner aus gesucht und **absolut** in das
Vorschauprogramm geschrieben. Ist die Maschine ungesichert, sichert `F5`
sie vorher (eine noch namenlose fragt nach dem Dateinamen) — die Vorschau
liegt neben der Datei. Ein zweites `F5` beendet die laufende Vorschau und
startet die neue.

## Verwendung im Spiel

```basic
IMPORT "animfsm"
IMPORT "sprite"

SCREEN(320, 240, "Held")
DIM hero AS SPRITE
hero = SPRITE_NEW(LOADIMAGE("assets/hero_walk.png"), 16, 16)
DIM fsm AS ANIM_FSM
fsm = ANIM_FSM_LOAD("assets/hero.dhanim")
ANIM_FSM_SETUP(fsm, hero)

DIM vx AS FLOAT
WHILE NOT QUITREQUESTED()
    vx = 0.0
    IF KEYPRESSED(KEY_RIGHT) THEN vx = 2.0
    ANIM_FSM_SET_FLOAT(fsm, "speed", ABS(vx))
    IF KEYHIT(KEY_SPACE) THEN ANIM_FSM_TRIGGER(fsm, "jump")
    ANIM_FSM_UPDATE(fsm, hero, DELTA() * 1000.0)
    CLS(BLACK)
    SPRITE_DRAW(hero)
    FLIP()
WEND
```

Das Format der Datei und alle Befehle beschreibt
[module-animfsm.md](module-animfsm.md).

## Bauweise

**Das Modell ist das `.dhanim`-JSON** (json-Modul), das Fenster nur die
Ansicht. Jede Änderung schreibt ins JSON, Rückgängig ist ein JSON-Text je
Stand. Was der Inspektor nicht kennt, läuft unverändert mit durch. Der Graph
wird mit den gewöhnlichen Zeichenbefehlen (`LINEW`, `TRIANGLE`, `BOXROUND`)
in einem `SCISSOR`-Bereich zwischen zwei `gui`-Fenstern gemalt; Zeichnen
und Treffertest benutzen dieselbe Geometrie (ein Klick trifft einen Pfeil,
wenn er höchstens 7 Punkte neben der Strecke liegt).

Zwei Dinge, die nicht offensichtlich sind:

- `first`, `last`, `x` und `y` müssen **ganze Zahlen** bleiben — das
  `animfsm`-Modul liest sie als Ganzzahl, aus `3.0` würde „nicht gesetzt",
  und der Zustand hätte keine Bilder. Der Editor schreibt sie darum überall
  mit `JSON_SET_INT`.
- Die Pille „Any State" steht nicht in der Datei; ihre Lage merkt sich der
  Editor selbst.

Übergänge zieht man mit der **rechten** Maustaste statt über einen
Modus-Schalter wie im alten Qt-Editor: ein Schalter ist etwas, das man
vergisst umzulegen. Der Verbinden-Modus (`L`) ist nur noch der Weg ohne
rechte Taste.

## Tests

`tests/pruef/werkzeug_animfsm.dhtest` fährt den Editor mit echten Klicks
über eine Aufnahme: Doppelklick legt einen Zustand an, die rechte Maustaste
zieht einen Übergang, Ziehen und zweimal `Strg+Z`, `Entf` samt Übergängen,
eine Bedingung über den Inspektor, ein neuer Parameter, und `F5` schreibt
eine Vorschau, die übersetzt **und** läuft. Geprüft wird die gesicherte
Datei mit der Laufzeit selbst — `ANIM_FSM_LOAD`, `ANIM_FSM_SETUP`, ein
Schritt, Zustandsname vergleichen (bei der Bedingung: mit `speed` 10 wird
`run` erreicht, mit 1 bleibt es `idle`). Dass die JSON gültig ist, wäre die
schwächere Aussage: ein Übergang zu einem gelöschten Zustand lädt nicht,
und genau das prüft der Fall nach `Entf`. Die Rückfrage beim Kreuz prüft
`tests/pruef/werkzeug_kreuz.dhtest` mit echten Fensternachrichten.

## Was der Qt-Editor hatte und 198 fehlt

- **Umbenennen und Startzustand per Rechtsklick auf einen Knoten** — die
  rechte Taste zieht hier Übergänge; beides geht über den Inspektor bzw.
  das Menü Bearbeiten.
- **Rollen des Graphen:** der sichtbare Ausschnitt ist fest, Knoten jenseits
  des Fensters erreicht man nicht.
