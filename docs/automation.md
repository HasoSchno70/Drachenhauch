# Eingabe aufzeichnen und abspielen (`AUTOMATION_*`)

Die Runtime kann den **Eingabe-Zustand jedes Frames mitschreiben** — Tasten,
Maustasten, Mausposition, Rad, Gamepad, Touch — und ihn später wieder in die
Eingabe einspeisen. Das Spiel merkt davon nichts: `KEYPRESSED`, `MOUSEX`,
`MOUSE_HIT` & Co. liefern während der Wiedergabe die aufgezeichneten Werte.

Drei Dinge, für die man das braucht:

* **Demo-/Attract-Modus** — das Spiel spielt sich selbst, solange niemand spielt.
* **Fehlerberichte zum Nachspielen** — „so bin ich durch die Wand gelaufen".
* **Automatische Spieltests** — derselbe Ablauf bei jedem Lauf, ohne Handarbeit.

Braucht ein Fenster (`SCREEN`) und die native Runtime `dhrt`.

## Befehle

| Befehl | Wirkung |
|---|---|
| `AUTOMATION_RECORD(datei$)` | Aufnahme starten (eine laufende Wiedergabe endet dabei) |
| `AUTOMATION_STOP()` → INTEGER | beendet Aufnahme **oder** Wiedergabe; schreibt die Aufnahme in ihre Datei und liefert die Anzahl der Ereignisse |
| `AUTOMATION_PLAY(datei$)` → INTEGER | Aufnahme laden und ab dem nächsten Frame abspielen; liefert die Anzahl geladener Ereignisse |
| `AUTOMATION_RECORDING()` → BOOLEAN | läuft gerade eine Aufnahme? |
| `AUTOMATION_PLAYING()` → BOOLEAN | läuft gerade eine Wiedergabe? (wird nach dem letzten Ereignis von selbst FALSE) |
| `AUTOMATION_FRAME()` → INTEGER | Frame-Nummer innerhalb der Wiedergabe (0 = erster) |
| `AUTOMATION_COUNT()` → INTEGER | Ereignisse in der laufenden Aufnahme bzw. der geladenen Datei |

```basic
SCREEN(640, 400, "Demo", 1)

IF KEYHIT(KEY_F9) THEN
    IF AUTOMATION_RECORDING() THEN
        PRINT "gespeichert: " + STR$(AUTOMATION_STOP()) + " Ereignisse"
    ELSE
        AUTOMATION_RECORD("aufnahme.txt")
    END IF
END IF

IF KEYHIT(KEY_F10) THEN AUTOMATION_PLAY("aufnahme.txt")
```

Vollständige Demo: [examples/153_automation.dh](../examples/153_automation.dh)
(F9 aufnehmen, F10 abspielen; die Spur zeigt, dass die Bahn identisch ist).

## Was man wissen muss

**Aufgezeichnet wird die EINGABE, nicht der Spielablauf.** Die Wiedergabe
drückt nur dieselben Tasten zur selben Zeit — alles andere muss der gleiche
Ausgangspunkt sein:

* **Startzustand zurücksetzen**, bevor die Wiedergabe beginnt (Position,
  Punkte, Level). Sonst wirkt dieselbe Eingabe auf einen anderen Anfang.
* **Zufall festnageln**: `RANDOMIZE 12345` mit festem Startwert, sonst würfelt
  der zweite Lauf andere Gegner.
* **Pro Frame rechnen, nicht pro Sekunde**, wenn die Bahn exakt gleich sein
  soll. Die Wiedergabe zählt in Frames; mit `DELTA()`-basierter Bewegung ergibt
  dieselbe Aufnahme bei anderer Bildrate eine leicht andere Bahn.

**Zeitliche Zuordnung.** Eingespeist wird am Ende jedes `FLIP` — direkt
nachdem die echte Eingabe für den nächsten Frame gelesen wurde, damit die
aufgezeichneten Werte gewinnen. Ein Ereignis aus Aufnahme-Frame `N` wirkt
daher im Programmdurchlauf `N+1`.

**`KEY_ANY_HIT` sieht die Demo nicht.** Ein Attract-Modus bricht typischerweise
ab, sobald der Spieler irgendeine Taste drückt — und genau dafür ist
`KEY_ANY_HIT()` da. raylib legt eingespeiste Tasten allerdings **auch** in seine
„zuletzt gedrückt"-Warteschlange; ungefiltert hätte die Demo sich an ihrem
eigenen ersten Tastendruck beendet. `dhrt` blendet deshalb aus, was die laufende
Wiedergabe selbst eingespeist hat: `KEY_ANY_HIT` meldet nur echte Eingabe,
während `KEYHIT`/`KEYPRESSED` die aufgezeichneten Tasten weiterhin sehen (darum
geht es ja). `JOYSTICK_ANY_BUTTON` braucht das nicht — die Wiedergabe setzt dort
nur den Knopf-Zustand, nicht raylibs „zuletzt gedrückter Knopf".

```basic
IF attract AND (KEY_ANY_HIT() <> -1 OR JOYSTICK_ANY_BUTTON() <> -1) THEN
    AUTOMATION_STOP()          ' Spieler uebernimmt
    attract = FALSE
END IF
```

**Aufnahme und Wiedergabe schließen sich aus.** raylib spielt während einer
laufenden Aufnahme grundsätzlich nichts ab; `AUTOMATION_PLAY` meldet das als
Fehler, statt es still zu schlucken.

**Das Dateiformat ist raylibs Textformat** (eine Zeile je Ereignis:
`e <frame> <typ> <p0> <p1> <p2> <p3>`). Es lässt sich mit einem Texteditor
ansehen und auch von Hand schreiben — praktisch für Testabläufe, die man nicht
erst „einspielen" will. Die Liste fasst maximal 16384 Ereignisse; eine
gehaltene Taste kostet **ein Ereignis pro Frame**, sehr lange Aufnahmen laufen
also irgendwann voll (raylib hört dann still auf mitzuschreiben).

**Grenze der Prüfung:** die Wiedergabe ist automatisiert getestet
([tests/test_automation.py](../tests/test_automation.py) schreibt
Aufnahmedateien selbst und prüft, dass `KEYPRESSED`/`MOUSEX`/`MOUSE_HIT` genau
die aufgezeichneten Werte liefern). Für die Aufnahme ist geprüft, dass echte
Eingabe erfasst und im richtigen Format geschrieben wird — die Runde
„echte Tastendrücke aufnehmen und identisch wiedergeben" lässt sich nur von
Hand nachvollziehen (synthetische Windows-Eingabe erreicht GLFW nicht
zuverlässig), dafür ist die Demo da.
