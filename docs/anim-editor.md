# `dhanim` — Animations-FSM-Editor (Unity-Mecanim-Stil)

Ein **Knoten-Graph-Editor** für Animations-State-Machines: Knoten sind **States**
(an eine Sprite-Animation gebunden), Pfeile sind **Transitions** (mit
Bedingungen). Das Ergebnis ist eine `.dhanim`-JSON, die das Runtime-Modul
[`animfsm`](module-animfsm.md) per `ANIM_FSM_LOAD` lädt. Damit schaltet man im
Spiel keine Animationen mehr von Hand — man setzt nur Parameter, die FSM
entscheidet den Zustand.

## Start

```
dhanim                  # leeres Projekt
dhanim hero.dhanim      # vorhandene FSM öffnen
```

Alternativ `dhrun.py --anim [datei.dhanim]` oder über den Start-Dialog (`gb`
ohne Argument → „Animation-Editor (FSM)"). Benötigt PySide6.

**Ohne Datei startet der Editor mit einer fertigen Beispiel-FSM**
(`examples/anim_demo.dhanim` — ein Platformer-Charakter mit idle/run/jump/fall,
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

`dhanim` erzeugt ein temporäres Drachenhauch-Programm und startet es mit `dhrt`:
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
fsm = ANIM_FSM_LOAD("assets/hero.dhanim")
ANIM_FSM_SETUP(fsm, hero)

' pro Frame:
ANIM_FSM_SET_FLOAT(fsm, "speed", ABS(vx))
IF INPUT_PRESSED("jump") THEN ANIM_FSM_TRIGGER(fsm, "jump")
ANIM_FSM_UPDATE(fsm, hero, dt_ms)
SPRITE_DRAW(hero)
```

## In Drachenhauch: `examples/198_anim_fsm_editor.dh`

Seit 2026-09-07 gibt es den Editor auch **in Drachenhauch selbst** (Weg B
aus [entwurf-python-abbau.md](entwurf-python-abbau.md), der zweite der
vier Editoren ohne Piloten). 1 336 Zeilen gegen 1 728 der Qt-Fassung
(1 247 UI + 481 Modell), Faktor 0,77 — mit dem üblichen Vorbehalt: der
Faktor misst vor allem, was weggelassen ist (siehe unten).

```
dhrt run examples/198_anim_fsm_editor.dh [-- maschine.dhanim]
```

Ohne Datei öffnet er wie die Qt-Fassung `anim_demo.dhanim`. Der Graph in
der Mitte wird mit den normalen Zeichenbefehlen gemalt (Knoten, Pfeile mit
Spitze und Beschriftung, die Pille „Any State", der Eingangspfeil zum
Startzustand); links Sprite-Blatt und Parameter, rechts der Inspektor für
den gewählten Zustand oder Übergang, alles `gui`-Fenster.

| Aktion | so |
|---|---|
| Zustand anlegen | Doppelklick auf freie Fläche, oder `Einfg` |
| Zustand verschieben | Knoten ziehen (Raster 8), Pfeile schieben den gewählten |
| **Übergang ziehen** | **mit der rechten Maustaste** von einem Knoten auf einen anderen ziehen (auch von „Any State"); oder `L` für den Verbinden-Modus mit der linken |
| Übergang bearbeiten | auf den Pfeil klicken → Inspektor: „erst wenn fertig", bis zu sechs Bedingungen (Parameter, Operator, Wert), `+ Bedingung`, `x` |
| Zustand bearbeiten | Knoten klicken → Name, Animation, Bild von/bis, Bilder/s, endlos, Startzustand |
| Parameter | links `+`/`-`, Name/Typ/Vorgabe, Umbenennen zieht die Bedingungen mit |
| Löschen | `Entf` — ein Zustand nimmt seine Übergänge mit |
| Rückgängig / Wiederholen | `Strg+Z` / `Strg+Y` |
| Vorschau | `F5` — schreibt `<name>_vorschau.dh` neben die `.dhanim` (Regler links, Sprite rechts, wie die Qt-Vorschau) und startet es mit `dhrt` |

**Das Modell ist das `.dhanim`-JSON** (json-Modul), Rückgängig ein JSON-Text
je Stand; was der Inspektor nicht kennt, läuft unverändert mit durch.
Geprüft wird die Datei von **zwei fremden Lesern**
(`tests/test_pilot_animfsm.py`): dem Modell des Qt-Editors (`AnimDoc.load`)
und der Laufzeit selbst — `ANIM_FSM_LOAD`, `ANIM_FSM_SETUP`, ein Schritt,
Zustandsname vergleichen. Dass die JSON gültig ist, wäre die schwächere
Aussage: ein Übergang zu einem gelöschten Zustand lädt nicht, und genau das
prüft der Test nach `Entf`.

Noch nicht: Umbenennen per Rechtsklick (nur im Inspektor), Rollen des
Graphen (Knoten jenseits des Fensters erreicht man nicht), Rückfrage beim
Schließen nur über das Menü. Zwei Fallen beim Bau: die Pille „Any State"
steht nicht in der Datei, also merkt der Editor sie sich selbst; und ein
Klick auf einen Knopf im Inspektor gibt dessen Fenster den Fokus — und bis
2026-09-07 galt ein Kürzel wie `Strg+S` nur im Fenster mit Fokus, dessen
Menü hier links hängt. Der Editor holte sich den Fokus nach jedem Knopf
zurück, sonst war Sichern nach `+ Bedingung` stumm (der Test sah es). Seit
die Laufzeit Kürzel in allen Fenstern des Programms prüft, braucht er das
nicht mehr.

## Architektur

Wie die anderen Begleit-Tools (`dhform`, `dhtilemap`) ist das **Datenmodell
Qt-frei** und headless testbar:

- [`drachenhauch/animeditor/document.py`](../drachenhauch/animeditor/document.py) —
  `AnimDoc`/`State`/`Transition`/`Condition`/`Param` + `History`, JSON-IO,
  `generate_runner()` (Vorschau-Code-Gen).
- [`drachenhauch/animeditor_qt.py`](../drachenhauch/animeditor_qt.py) — die Qt-UI
  (Graph-Canvas, Inspector, Parameter-Panel).

Tests: `tests/test_animeditor_document.py` (Modell/Roundtrip/Closed-Loop/Codegen)
+ `tests/test_animeditor_qt.py` (Konstruktion/Wiring, offscreen). Das `.dhanim`-
Format ist in [docs/module-animfsm.md](module-animfsm.md) beschrieben.
