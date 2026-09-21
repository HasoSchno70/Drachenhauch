# Form-Designer (WYSIWYG, Xojo-Stil)

Oberflächen für das `gui`-Modul zusammenklicken: Controls ablegen, im
Inspektor einstellen, als `.dhform` sichern und im eigenen Programm mit
`GUI_LOAD` laden — oder gleich mit F5 laufen lassen. Der Designer ist selbst
ein Drachenhauch-Programm:
[`examples/197_form_designer.dh`](../examples/197_form_designer.dh)
(1 342 Zeilen). Die frühere Qt-Fassung (PySide6) ist mit dem Python-Teil des
Projekts entfernt.

## Starten

- **Aus der IDE:** Menü *Werkzeuge → Form-Designer* (oder in der
  Befehlspalette „Werkzeug: Form-Designer"). Ein Klick auf eine `.dhform` im
  Projektbaum öffnet den Designer **mit dieser Datei**; wer sie als Text sehen
  will, nimmt in der Befehlspalette „Formular als Text öffnen".
- **Von der Kommandozeile:**

```
dhrt run examples/197_form_designer.dh [-- formular.dhform]
```

Ein relativer Dateiname gilt vom Ordner aus, in dem der Aufruf stand
(`DHRT_START_DIR`). Gibt es die Datei noch nicht, beginnt ein leeres
Formular, das beim ersten Sichern dorthin geschrieben wird.

## Aufbau

- **Links — Menü und Palette.** Die Palette listet **alle 30 Widget-Arten der
  Laufzeit** und dazu das **Gitter** (eine Tabelle im Zellmodus, alle Spalten
  bearbeitbar). Eintrag anklicken („scharf"), dann auf die Form klicken =
  ablegen, am 8-px-Raster. Unter der Palette steht die Statuszeile.
- **Mitte — die Form.** Sie ist ein **echtes `GUI_WINDOW` im Entwurfsmodus**
  (`GUI_WINDOW_DESIGN(win, TRUE)`): die Laufzeit zeichnet die Controls genau
  so, wie das Programm sie später bekommt, nimmt ihnen aber jede Eingabe; die
  Maus verwaltet der Designer selbst (`GUI_HIT_TEST` geht weiter). Eine
  nachgemalte Vorschau, die von der Laufzeit abweichen könnte, gibt es nicht.
  Klick = auswählen, ziehen = verschieben, an den **acht Griffen** ziehen =
  Größe ändern — alles am Raster.
- **Rechts — Inspektor.** Mit ausgewähltem Control: Name, X, Y, Breite, Höhe,
  Text, Tooltip, Anker (`lrtb`), `on_click`, `on_change`, `on_enter`,
  **Schriftstil** (`fett`, `kursiv`, `unterstrichen`, `durchgestrichen`,
  verbunden mit `+`) und *Aktiviert*, dazu die Felder je Art (unten). Ohne Auswahl zeigt er das
  **Formular selbst**: Titel, Breite, Höhe, *Größenveränderbar* und das
  **Thema** (`glas_dunkel`, `glas_hell`, `dark`, `light` oder keines).
  Enter in einem Feld oder [Übernehmen] schreibt die Werte.

## Tasten

| Taste | Wirkung |
|---|---|
| `Strg+N` / `Strg+O` | neues Formular / Formular öffnen |
| `Strg+S` / `Strg+Umschalt+S` | sichern / sichern unter |
| `F5` | Formular ausführen (siehe unten) |
| `Strg+G` | GB-Code schreiben (siehe unten) |
| `Strg+Z` / `Strg+Y` | rückgängig / wiederholen |
| `Strg+D` | Control verdoppeln (um ein Raster versetzt) |
| `Entf` | Control löschen |
| Pfeile | um ein Raster schieben, mit `Umschalt` um einen Punkt |
| `Esc` | Palette entschärfen, Auswahl aufheben |
| `Strg+Q` | beenden |

*Nach vorn* und *Nach hinten* stehen im Menü *Bearbeiten*: die Reihenfolge
der Controls ist ihre Zeichenreihenfolge, das letzte liegt vorn. Entf und die
Pfeile wirken nur, solange kein Eingabefeld des Inspektors den Fokus hat.

**Rückgängig** merkt sich je Schritt das ganze Formular als JSON-Text (bis zu
200 Stände). Ein Zug mit der Maus ist **ein** Schritt, gemerkt beim
Loslassen.

**Beenden** — über das Menü, das Kreuz oder Alt+F4 — fragt nach
(*Sichern|Verwerfen|Abbrechen*), wenn das Formular nicht gesichert ist;
sonst endet der Designer sofort.

## Felder je Art

Sichtbar nur bei der passenden Art; Listen werden mit **Semikolon** getrennt,
weil ein Komma zu oft in einem Eintrag selbst steht.

| Art | Felder |
|---|---|
| Klappliste, Liste | Einträge (`Rot; Grün; Blau`) |
| Tabelle, Gitter | Spalten, Breiten, Bearbeitbar (`0; 2` oder `alle`), Spaltenarten (`text; ganz; zahl; auswahl`), Auswahl (`2 = Rot\|Grün`), Kästchen *Zellmodus* |
| Regler, Fortschritt, Zahlenfeld, Drehknopf | Min, Max, Wert |

Eine Spalte mit Auswahlliste wird dabei **zur Auswahlspalte**, auch wenn es
unter Spaltenarten nicht steht — beim Laden wirkte die Liste sonst nicht.

Die **Datenzeilen einer Tabelle** trägt man bewusst nicht im Designer ein.
Eine Tabelle wird im Normalfall zur Laufzeit gefüllt — aus einer Datei, einer
Datenbank, dem Spielstand. Der Designer legt das Gerüst fest, die Zeilen
kommen aus dem Programm (`GUI_TABLE_ADD_ROW`).

## Nutzen: drei Wege

1. **Im eigenen Code:** `GUI_LOAD("meinform.dhform")` und die Handler-`SUB`s
   schreiben. Das `.dhform` speichert je Control den **Namen** seines
   Handlers (`on_click`, `on_change`, …), und `GUI_UPDATE` ruft ausgelöste
   Handler automatisch per Name auf — kein Verdrahten von Hand
   (Formular-Workflow in [module-gui.md](module-gui.md)).

```basic
' So nutzt du ein gespeichertes Formular im eigenen Programm:
IMPORT "gui"
SCREEN(800, 480, "App", 1)
DIM frm AS GUI_WINDOW
frm = GUI_LOAD("forms/settings.dhform")

SUB on_save()            ' Name = der im Inspektor eingetragene Handler
    PRINT "gespeichert"
END SUB

WHILE NOT QUITREQUESTED()
    GUI_UPDATE() : CLS(0) : GUI_DRAW() : FLIP()
WEND
```

2. **Direkt ausführen (F5):** der Designer sichert (ein neues Formular fragt
   dabei nach dem Namen), schreibt `<name>_lauf.dh` **neben die `.dhform`**
   und startet es mit `dhrt run`. Das Gerüst setzt das Thema, lädt die Form
   per `GUI_LOAD` randlos auf das Programmfenster (Fenstergröße = Formgröße)
   und enthält je Handler eine `SUB` — mit dem Rumpf aus dem Feld `code` der
   `.dhform`, sonst mit einem `' TODO`. Ein erneutes F5 beendet den vorigen
   Lauf; beim Ende des Designers wird er ebenfalls beendet. Wie er ausging,
   meldet die Statuszeile.
3. **GB-Code (Strg+G):** `<name>_code.dh` baut das Formular **Aufruf für
   Aufruf** — Konstruktor je Control, bei der Tabelle Kopf, Breiten,
   Zellmodus, bearbeitbare Spalten, Spaltenarten und Auswahllisten, dazu
   Datum und Uhrzeit, gesperrt, Anker, Tooltip, Schriftstil und die Handler samt Rümpfen
   aus `code`; ohne `GUI_LOAD` und ohne die `.dhform` zur Laufzeit, lesbar
   und von Hand weiterzuschreiben. Die Konstruktoren, die sich selbst messen
   (Beschriftung, Kästchen, Regler …), bekommen ein `GUI_SET_BOUNDS`
   hinterher, sonst ginge die Größe aus dem Designer verloren. Übersprungen
   wird nur das Bild (die `.dhform` kennt keine Bildquelle, und `GUI_IMAGE`
   bräuchte ein `LOADIMAGE`) — mit einem Kommentar an der Stelle. Menüs baut
   der Code nicht nach und sagt es ebenfalls in einem Kommentar.

## Dateiformat

`.dhform` ist exakt das JSON, das `GUI_SAVE`/`GUI_LOAD` und
`GUI_TO_JSON`/`GUI_FROM_JSON` schreiben und lesen (siehe
[module-gui.md](module-gui.md)) — plus zwei Designer-Felder, die die
Laufzeit übergeht: `name` je Control und ein `code` auf oberster Ebene
(`{handler_name: rumpf}`) mit den Handler-Rümpfen.

**Das Modell des Designers IST dieses JSON** (json-Modul), die Form auf dem
Schirm nur die Ansicht: jede Änderung schreibt ins JSON und baut die Ansicht
neu (`GUI_FROM_JSON`). Sichern ist `JSON_PRETTY`, Laden `JSON_LOAD`. Daraus
folgt: **alles, was der Inspektor nicht zeigt, läuft unverändert mit
durch** — Menüs, Reiter, Tabellendaten, Baumknoten, `code`, Regeln,
Bindungen. Eine im Programm gebaute und mit `GUI_SAVE` gesicherte Form lässt
sich also im Designer öffnen und nachjustieren, ohne dass etwas davon
verloren geht; bearbeiten lässt es sich dort aber nicht.

Beispiel-Formular: `examples/forms/settings.dhform` mit
[examples/105_form_runner.dh](../examples/105_form_runner.dh).

## Was die Qt-Fassung hatte und diese nicht

Der Designer in Drachenhauch hat ein Viertel der Zeilen der Qt-Fassung
(1 342 gegen 5 055, Faktor 0,27) — und der Faktor misst vor allem, was
weggelassen ist. Nicht (oder nicht mehr) vorhanden:

- **Mehrfachauswahl** samt Auswahlrahmen, **Ausrichten**, **gleiche Größe**
  und **Verteilen**; ausgewählt ist immer genau ein Control.
- **Ausrichtungs-Hilfslinien** beim Ziehen, **Zoom**, ein **Kontextmenü**.
- **Kopieren/Einfügen** (nur Verdoppeln) und **Ablegen per Drag&Drop** aus
  der Palette (nur anklicken, dann auf die Form klicken).
- Ein **Code-Editor für Handler** (Doppelklick auf ein Control legte dort
  einen Handler an). Hier steht der Rumpf im Feld `code` der `.dhform` und
  wird nur durchgereicht; geschrieben wird er im eigenen Programm oder von
  Hand in der Datei.
- **Mehrformular-Projekte** (`.dhproj`): hier ist immer genau ein Formular
  offen.
- Im Inspektor: **Layout-/Panel-Zuordnung**, **Regeln** und **Bindung**,
  Tab-Reihenfolge, Radio-**Gruppe**, **Platzhalter**, die **Auswahl** einer
  Klappliste, Min/Max-Größe und *beweglich/schließbar/sichtbar* des
  Formulars, die Schalter der Tabelle (Zebra, Filterzeile, Sortieren …) und
  die Werte von Baum, Farb- und Datumswähler. Was davon in einer Datei
  steht, bleibt beim Öffnen und Sichern erhalten (siehe Dateiformat).
- **Menüs im GB-Code** — die Qt-Fassung schrieb sie mit, diese sagt nur in
  einem Kommentar, dass `GUI_LOAD` sie baut. Einen Menü-*Editor* hatte auch
  die Qt-Fassung nicht.
- F5 prüft das Laufprogramm nicht vorher mit `dhrt --check`; ein Fehler zeigt
  sich im gestarteten Programm.
- Die Form auf dem Schirm erscheint im Thema des Designers, nicht im
  eingestellten Thema des Formulars — das bekommt erst das erzeugte Programm.

## Drei Fallen beim Bau

- **Eine Liste meldet kein `GUI_CLICKED`** — die Palette wird über ihre
  AUSWAHL scharf, die Auswahl ist das Ereignis.
- **Ein neu gebautes Fenster nimmt den Fokus**, und Menü-Kürzel galten nur im
  Fenster mit Fokus. `ansichtBauen` merkt sich darum `GUI_FOCUSED()` und gibt
  ihn zurück — ohne das wären Strg+S und F5 nach dem ersten Ablegen tot.
  (Seit 2026-09-07 gelten Kürzel in allen sichtbaren Fenstern; das
  Zurückgeben blieb.)
- **Ganze Zahlen bleiben ganz:** `JSON_TYPE` sagt nur `number`. Beim Kopieren
  eines Teilbaums (Verdoppeln, Umordnen) wird eine ganze Zahl darum als
  ganze geschrieben — aus `48.0` liest `GUI_FROM_JSON` kein x mehr.

## Prüfung

[`tests/pruef/werkzeug_formdesigner.dhtest`](../tests/pruef/werkzeug_formdesigner.dhtest)
(`dhrt test`): ein Fall legt per echtem Klick einen Button ab, sichert mit
Strg+S und liest die Datei mit dem json-Modul; einer zieht ein Control und
nimmt zweimal zurück; einer prüft das F5-Laufprogramm; einer, dass fremde
Felder einer bestehenden Form erhalten bleiben; einer legt **jede** Art an
und lässt den GB-Code durch `dhrt --check` und einen Lauf; zwei prüfen den
Inspektor an Tabelle und Gitter; einer misst die Palette gegen
`Kind::from_str` in `gui.rs` — eine Art, die die Laufzeit kann und der
Designer nicht, fällt sonst niemandem auf; zwei belegen den Entwurfsmodus
(mit Gegenprobe). `DH_FORM_LOG=<datei>` lässt den Designer seine Ereignisse
zeilenweise protokollieren.
