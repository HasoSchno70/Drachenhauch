# Partikel-Editor (`examples/185_partikel_editor.dh`)

Alle Parameter des [`particles`](module-particles.md)-Moduls live einstellen,
mit echter Vorschau, und das Ergebnis als DH-Code mitnehmen — geschrieben in
Drachenhauch auf dem eigenen `gui`-Modul. Die Vorschau ist **keine
Nachbildung**: sie treibt ein echtes `PARTICLE_SYSTEM` mit denselben Aufrufen,
die später im Spiel stehen, und zeichnet es mit `PARTICLE_DRAW`. Was man
sieht, ist, was man bekommt.

```
dhrt run examples/185_partikel_editor.dh
```

In der [IDE](ide.md) steht er im Menü **Werkzeuge** und läuft dort als eigenes
Programm. Python braucht er nicht.

## Das Fenster

| Bereich | Was geht |
|---|---|
| Vorschau | Die Teilchen, beschnitten auf die Fläche; die Quelle sitzt in der Mitte oder, mit **folgt der Maus**, unter dem Zeiger. Unten die Zahl der lebenden Teilchen |
| Bedienung | **Pause**/**Weiter**, **Leeren**, **Salve** (200 Teilchen auf einmal — für Explosionen) |
| Werkseinstellungen | Funken, Rauch, Feuer, Regen, Schnee, Explosion, Zauber, Springbrunnen — ein Klick setzt alle Regler, Darstellung und Schalter und leert die Vorschau |
| Ausgabe | **Sichern ...**/**Laden ...** (`.ini`), **DH-Code kopieren** |
| Verlauf | **Zurueck**/**Vor** oder Strg+Z/Strg+Y |

Die Regler:

| Gruppe | Regler |
|---|---|
| Bewegung | Tempo X min/max, Tempo Y min/max (±400 px/s), Kraft X/Y (±600 px/s²) |
| Lebenszeit (ms) und Ausstoss | Leben min/max (50..4000 ms), je Bild (0..60 Teilchen) |
| Aussehen | Darstellung (`circle`, `pixel`, `square`, `streak`, `glow`), Groesse min/max, **am Ende ausblenden** |
| Farbe am Anfang | Rot, Gruen, Blau, darunter ein Farbfeld mit dem Ergebnis |
| Farbe am Ende | **Farbverlauf benutzen**, Rot, Gruen, Blau, Farbfeld |

Ohne Farbverlauf bekommt das System die Startfarbe auch als Endfarbe — das
Modul kennt kein „aus“. `glow` zeichnet `PARTICLE_DRAW` derzeit wie `circle`;
für echtes Leuchten im Spiel `BLEND_MODE("add")` um den Aufruf legen.

Alles geht auch ohne Maus: TAB wechselt das Bedienelement, die Leertaste löst
aus, die Pfeile verstellen einen Regler.

## Rückgängig

Ein **Zug** am Regler ist **ein** Schritt: aufgezeichnet wird erst, wenn sich
ein Bild lang nichts geändert hat und keine Maustaste mehr gedrückt ist.
Werkseinstellung und geladene `.ini` sind auf demselben Weg je ein Schritt.
Gemerkt wird der volle Stand (17 Regler, Darstellung, zwei Schalter), bis zu
32 Schritte; ein neuer Zug nach einem Zurück schneidet den Vor-Weg ab.

## Sichern und Laden

**Sichern ...** schreibt alle Regler in eine `.ini`, **Laden ...** holt sie
zurück. Schlüssel ist die **Beschriftung** des Reglers, dazu `modus` für die
Darstellung — lesbar und von Hand änderbar. Ein unbekannter Schlüssel wird
übergangen, ein Wert außerhalb des Reglerbereichs geklemmt. Die beiden Schalter
(Ausblenden, Farbverlauf) stehen nicht in der Datei.

Beim Schließen (Kreuz, Alt+F4 oder ESC) fragt der Editor nach
(Sichern / Verwerfen / Abbrechen), aber nur, wenn etwas vom letzten Sichern,
Laden oder Wählen einer Werkseinstellung abweicht.

## DH-Code

**DH-Code kopieren** legt die Aufrufe in die Zwischenablage, die dieses System
im eigenen Programm erzeugen. Die Quelle steht dort als `x, y` — die Lage
bestimmt das Spiel —, und den `IMPORT "particles"` schreibt man selbst dazu.
Für die Werkseinstellung „Funken“, mit einer festen Lage davor:

```basic
IMPORT "particles"

DIM x AS INTEGER
DIM y AS INTEGER
x = 160
y = 120

DIM sys AS PARTICLE_SYSTEM
sys = PARTICLE_SYSTEM_NEW(x, y)
PARTICLE_SET_VELOCITY(sys, -90, 90, -200, -60)
PARTICLE_SET_GRAVITY(sys, 0, 420)
PARTICLE_SET_LIFETIME(sys, 300, 800)
PARTICLE_SET_SIZE(sys, 2, 4)
PARTICLE_SET_COLOR(sys, RGB(255, 220, 80))
PARTICLE_SET_COLOR_END(sys, RGB(255, 60, 20))
PARTICLE_SET_MODE(sys, "circle")
PARTICLE_SET_FADE(sys, TRUE)

' pro Bild:
PARTICLE_EMIT(sys, 8)
PARTICLE_UPDATE(sys, 16)
PARTICLE_DRAW(sys)
```

Die drei Zeilen nach `' pro Bild:` gehören in die Bildschleife.

## Gegenüber der alten Qt-Fassung

Die frühere Qt-Fassung (`dhparticles`) hatte eine **Preset-Bibliothek** mit
benannten eigenen Einstellungen in einer gemeinsamen Datei und zeigte den
DH-Code in einem eigenen Fenster. Hier ist jede eigene Einstellung eine eigene
`.ini`, und der Code geht direkt in die Zwischenablage.

## Tests

`tests/pruef/werkzeug_partikel.dhtest` fährt echte Mausklicks und Tasten über
eine Aufnahme: ein Zug ist ein Schritt, Zurück nimmt den ganzen Zug zurück und
Vor holt ihn wieder, Strg+Z/Strg+Y tun dasselbe wie die Knöpfe, eine
Werkseinstellung ist ein Schritt, ein neuer Zug schneidet den Vor-Weg ab, und
der erste Eintrag der Werkseinstellungen ist anklickbar (früher lagen die
Knöpfe darauf). Die Rückfrage beim Schließen prüft
`tests/pruef/werkzeug_kreuz.dhtest` mit echten Fensternachrichten.
