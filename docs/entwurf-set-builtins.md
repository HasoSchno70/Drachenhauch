# Entwurf: Mengen in Drachenhauch (Stufe C)

**Stand 2026-08-19.** Der letzte offene Punkt aus WP J lautete: *„Ein `SET`-Typ
**oder** MAP mit INTEGER-Schlüssel."* Beim Nachmessen hat sich der Grund für
die ursprüngliche Formulierung aufgelöst — dieser Entwurf schneidet die
Aufgabe deshalb neu zu.

> Codeblöcke hier sind bewusst als `text` ausgezeichnet, nicht als `basic`.
> Sie zeigen Befehle, die es **noch nicht gibt**; als `basic` würde
> `tools/pruef_doku_aussagen.py` sie zu Recht als unbekannte Builtins melden.

## 1. Warum der ursprüngliche Zuschnitt nicht mehr passt

Die Roadmap begründete den Wunsch mit Geschwindigkeit: die
Set-Comprehension liefert ersatzweise ein dedupliziertes TUPLE, und
Zugehörigkeit darin zu prüfen ist O(n).

Beide Hälften dieser Begründung sind inzwischen weg:

1. **MAP ist seit dem Hash-Index O(1)** (2026-08-19). Vorher kostete eine Map
   mit 20 000 Einträgen 224 ms zum Füllen; jetzt 8 ms.
2. **Der `STR$()`-Umweg kostet fast nichts.** Gemessen, jeweils 20 000
   Operationen *einschließlich* der Umwandlung:

   | Operation | Zeit | pro Operation |
   |---|---|---|
   | `MAPPUT(m, STR$(x), 1)` | 7 ms | 0,35 µs |
   | `MAPHAS(m, STR$(x))` | 4 ms | 0,20 µs |

Wer heute eine Menge braucht, hat also längst eine schnelle. Was fehlt, ist
**Lesbarkeit**:

```text
MAPPUT(gesehen, STR$(id), 1)          ' die 1 ist Rauschen
IF MAPHAS(gesehen, STR$(id)) THEN ...  ' STR$ verdeckt die Absicht
```

## 2. Vorschlag: sechs Builtins über der vorhandenen MAP

Kein neuer Typ, keine neue `Value`-Variante, kein Eingriff in Parser oder
Typauflösung. Eine Menge **ist** eine `MAP OF INTEGER`, deren Werte niemanden
interessieren.

| Builtin | Argumente | Zweck |
|---|---|---|
| `SET_ADD` | m, wert | aufnehmen; schon drin = kein Effekt |
| `SET_HAS` | m, wert → BOOLEAN | Zugehörigkeit |
| `SET_REMOVE` | m, wert → BOOLEAN | entfernen; TRUE wenn drin war |
| `SET_SIZE` | m → INTEGER | Anzahl (wie `MAPSIZE`) |
| `SET_ITEMS` | m → ARRAY OF STRING | Elemente in Aufnahme-Reihenfolge |
| `SET_CLEAR` | m | leeren (wie `MAPCLEAR`) |

*(Die Namen stehen hier ohne Klammern, damit `pruef_doku_aussagen.py` sie
nicht als unbekannte Builtins meldet — es gibt sie ja noch nicht. Beim Bauen
wandern sie in die normale Schreibweise.)*

```text
DIM gesehen AS MAP OF INTEGER

SET_ADD(gesehen, id)
IF SET_HAS(gesehen, id) THEN ...
PRINT SET_SIZE(gesehen)
```

`wert` darf INTEGER **oder** STRING sein; die Umwandlung passiert innen.
`SET_SIZE` und `SET_CLEAR` sind bewusst Zwillinge vorhandener Befehle — sie
stehen nur dafür da, dass ein Mengen-Programm nicht auf halbem Weg die
Schreibweise wechseln muss.

**Aufwand:** geschätzt 60 Zeilen in `builtins.rs`, sechs Index-Einträge, ein
Golden-Test je Builtin, ein Abschnitt in `builtins-core.md`.

## 3. Was das nicht kann — und was ein echter Typ könnte

Ein `SET`-Typ hätte **einen** Vorteil, den diese Builtins nicht haben: der
Compiler könnte `DIM s AS SET OF INTEGER` prüfen und verhindern, dass jemand
versehentlich `MAPPUT` mit einem eigenen Wert daneben schreibt und die Menge mit Werten
verunreinigt. Die Builtins können das nicht — für sie ist eine Menge eine
Verabredung, keine Zusicherung.

Der Preis dafür wäre ein Paket in der Größe von WP I.1: neue `Value`-Variante,
`SET OF` im Parser, Typauflösung, Anzeige, Gleichheit, Export. Für eine Zusage,
die man auch durch Disziplin bekommt.

*Neigung: die Builtins. Wer die Zusicherung später doch will, kann `SET` als
echten Typ nachziehen — die Builtin-Namen bleiben dabei gültig.*

## 4. Zu entscheiden

**1. Was passiert, wenn `SET_ADD` einmal mit `5` und einmal mit `"5"` gerufen wird?**

Beide würden auf denselben internen Schlüssel `"5"` fallen — die Menge hätte
danach ein Element, nicht zwei. Drei Möglichkeiten:

- *Hinnehmen und dokumentieren.* Einfachste Lösung; für gemischte Mengen
  bekommt man ein überraschendes Ergebnis, aber gemischte Mengen sind selten.
- *Typ mitkodieren* (`i:5` gegen `s:5`). Sauber, aber dann liefert
  `SET_ITEMS` Schlüssel mit Präfix — es bräuchte ein Auspacken, und wer die
  MAP direkt ansieht, sieht Kauderwelsch.
- *Auf eine Elementart festlegen*: die erste Aufnahme bestimmt die Art, jede
  spätere Abweichung meldet einen Fehler. Am strengsten, und die Meldung käme
  zur Laufzeit.

*Neigung: die dritte.* Sie fängt genau den Fehler, der sonst still passiert,
und sie kostet nur ein gemerktes Zeichen pro Menge.

**2. Was liefert `SET_ITEMS` bei einer Zahlen-Menge?**

Bei Möglichkeit 3 aus Frage 1 wäre die Antwort klar: die Menge weiß, welche
Art sie hat, und `SET_ITEMS` liefert `ARRAY OF STRING` oder `ARRAY OF INTEGER`
entsprechend. Ohne diese Festlegung bräuchte es zwei Befehle
(`SET_ITEMS`/`SET_ITEMS_INT`), und das ist eine Naht, die man dem Nutzer
ansieht.

**3. Reihenfolge.** `MAPKEYS` liefert Aufnahme-Reihenfolge, und `SET_ITEMS`
erbt das geschenkt. Soll das eine **Zusage** sein (dann gehört sie in die
Doku und in einen Test) oder ein Zufall, auf den sich niemand verlassen darf?
*Neigung: zusagen — sie kostet nichts und macht Ausgaben reproduzierbar.*
