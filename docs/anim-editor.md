# `dhanim` — Animations-FSM-Editor (Unity-Mecanim-Stil)

Ein **Knoten-Graph-Editor** für Animations-State-Machines: Knoten sind **States**
(an eine Sprite-Animation gebunden), Pfeile sind **Transitions** (mit
Bedingungen). Das Ergebnis ist eine `.gbanim`-JSON, die das Runtime-Modul
[`animfsm`](module-animfsm.md) per `ANIM_FSM_LOAD` lädt. Damit schaltet man im
Spiel keine Animationen mehr von Hand — man setzt nur Parameter, die FSM
entscheidet den Zustand.

## Start

```
dhanim                  # leeres Projekt
dhanim hero.gbanim      # vorhandene FSM öffnen
```

Alternativ `dhrun.py --anim [datei.gbanim]` oder über den Start-Dialog (`gb`
ohne Argument → „Animation-Editor (FSM)"). Benötigt PySide6.

**Ohne Datei startet der Editor mit einer fertigen Beispiel-FSM**
(`examples/anim_demo.gbanim` — ein Platformer-Charakter mit idle/run/jump/fall,
Parametern `speed`/`grounded`/`jump` und allen Übergangs-Arten), damit man sofort
sieht, wie ein Graph aussieht. `Strg+N` leert das Projekt für einen Neuanfang.

## Oberfläche

- **Mitte — Graph:** die States als Knoten, Transitions als Pfeile. Ein grüner
  Eingangspfeil markiert den **Default-State**, die violette Pille **„Any State"**
  ist die Quelle für Übergänge von überall.
- **Links — Dokument + Parameter:** Sprite-Sheet (Bildpfad + Frame-Größe +
  Vorschau-Skala) und die Liste der **Parameter** (`bool`/`float`/`int`/`trigger`)
  mit Default-Wert.
- **Rechts — Inspector:** kontextabhängig die Eigenschaften des ausgewählten
  States (Name, Animation, loop/one-shot, Frame-Range, FPS, Default) **oder** der
  ausgewählten Transition (`wait_finished` + Bedingungs-Tabelle).

## Bedienung

| Aktion | so |
|---|---|
| State anlegen | Doppelklick auf leere Fläche (oder Toolbar „+ State") |
| State verschieben | Knoten ziehen (rastet aufs Raster) |
| State umbenennen | Rechtsklick → „Umbenennen" oder im Inspector |
| Default-State setzen | Rechtsklick auf Knoten → „Als Default-State" |
| **Transition ziehen** | **„Link-Modus" (Taste `L`) aktivieren**, dann von einem Knoten auf einen anderen ziehen |
| Transition bearbeiten | auf den Pfeil klicken → Inspector |
| Bedingung hinzufügen | im Transition-Inspector „+ Bedingung" |
| Löschen | Knoten/Pfeil wählen → `Entf` |
| Undo / Redo | `Strg+Z` / `Strg+Y` |
| **Vorschau** | **`F5`** — startet eine Live-Vorschau mit `dhrt` |

### Vorschau (F5)

`dhanim` erzeugt ein temporäres GameBasic-Programm und startet es mit `dhrt`:
links ein **Live-Parameter-Panel** (Slider für `float`/`int`, Checkbox für
`bool`, Button für `trigger` — über das `ui`-Modul), rechts der Sprite, der den
**aktuellen State** spielt. So testet man die Übergänge sofort, ohne Spielcode zu
schreiben — wie Unitys Animator-Preview. Ist ein Sprite-Sheet gesetzt, wird es in
den Temp-Ordner gespiegelt; ohne Sheet zeigt die Vorschau einen Platzhalter mit
Frame-Nummer.

## Verwendung im Spiel

```basic
IMPORT "animfsm"
IMPORT "sprite"

DIM hero AS SPRITE
hero = SPRITE_NEW(LOADIMAGE("assets/hero_walk.png"), 16, 16)
DIM fsm AS ANIM_FSM
fsm = ANIM_FSM_LOAD("assets/hero.gbanim")
ANIM_FSM_SETUP(fsm, hero)

' pro Frame:
ANIM_FSM_SET_FLOAT(fsm, "speed", ABS(vx))
IF INPUT_PRESSED("jump") THEN ANIM_FSM_TRIGGER(fsm, "jump")
ANIM_FSM_UPDATE(fsm, hero, dt_ms)
SPRITE_DRAW(hero)
```

## Architektur

Wie die anderen Begleit-Tools (`dhform`, `dhtilemap`) ist das **Datenmodell
Qt-frei** und headless testbar:

- [`drachenhauch/animeditor/document.py`](../drachenhauch/animeditor/document.py) —
  `AnimDoc`/`State`/`Transition`/`Condition`/`Param` + `History`, JSON-IO,
  `generate_runner()` (Vorschau-Code-Gen).
- [`drachenhauch/animeditor_qt.py`](../drachenhauch/animeditor_qt.py) — die Qt-UI
  (Graph-Canvas, Inspector, Parameter-Panel).

Tests: `tests/test_animeditor_document.py` (Modell/Roundtrip/Closed-Loop/Codegen)
+ `tests/test_animeditor_qt.py` (Konstruktion/Wiring, offscreen). Das `.gbanim`-
Format ist in [docs/module-animfsm.md](module-animfsm.md) beschrieben.
