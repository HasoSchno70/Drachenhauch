# GameBasic — Stolpersteine & Inkonsistenzen (beim Schreiben des Lehrbuchs aufgefallen)

Sammlung von Sprach-/Engine-Reibungspunkten, die beim Verfassen von
`buch-referenz/` (alle Kapitel + Module gegen `gbrt` verifiziert) auftauchten.
**Nicht das Buch, sondern GameBasic selbst** betreffend — als Backlog zum
späteren Beheben. Jeder Punkt ist gegen den aktuellen `gbrt`-release-Build
reproduziert.

Reihenfolge ≈ nach Nutzen/Aufwand. Erledigtes am Ende.

---

## A — Echte Lücken / häufige Stolpersteine (lohnt sich)

### A1. Kein Array-Literal `[1, 2, 3]`
```basic
DIM a AS ARRAY OF INTEGER
a = [1, 2, 3]      ' -> Parse-Fehler (7): Erwartet FOR in List-Comprehension
```
`[...]` ist ausschließlich List-Comprehension. Es gibt **keine** Kurzschreibweise,
ein Array mit Werten zu füllen — man muss `DIM a[3]` + Einzel-Zuweisungen nutzen.
Tupel haben `(1, 2, 3)`, Arrays nichts Vergleichbares. Sehr häufige Erwartung von
Einsteigern.
**Vorschlag:** Im Parser disambiguieren — folgt nach `[` kein `expr FOR …`,
als Array-Literal werten (`[1,2,3]` → `ARRAY OF INTEGER`, Typ aus den Elementen).

### A2. `NIL` ist kein Literal  —  ✅ BEHOBEN (commit f4c8b78)
> NIL ist jetzt ein Keyword-Literal (lexer/parser/compiler in gbrt + Python-Front-End
> fuer Editor/Paritaet). `x = NIL`, `x <> NIL`, `IS_NIL(NIL)` funktionieren; das in der
> db-Doku versprochene NIL→NULL-Binding klappt nun wirklich. Tests: `tests/test_nil_literal.py`.
```basic
IF o = NIL THEN ...        ' -> Laufzeitfehler: Variable 'nil' nicht deklariert (DIM fehlt?)
IF o <> NIL THEN ...       ' dito
PRINT IS_NIL(NIL)          ' dito
```
`NIL` lässt sich nirgends ausschreiben, obwohl ältere Doku/Beispiele `<> NIL`
verwenden. Folgen quer durchs Buch:
- `net`: `NET_TCP_ACCEPT`-Leere nur per `IS_NIL(x)` prüfbar, nicht `x <> NIL`.
- `db`: dokumentiertes „NIL → NULL"-Binding geht nicht (man muss die Spalte im
  INSERT weglassen).
**Vorschlag:** `NIL` als echtes Keyword-Literal einführen (mindestens in
`=`/`<>`-Vergleichen und als Argument). Macht `IS_NIL` halb überflüssig und
das db-NULL-Binding möglich. Größter „Vertrag-vs-Realität"-Punkt.

---

## B — Irreführende Fehlermeldungen (klein, hohe Wirkung)

### B1. „Stufe 3e: DIM-Typ 'vec2' noch nicht unterstuetzt" bei fehlendem IMPORT  —  ✅ BEHOBEN (commit f4c8b78)
> Jetzt: `Unbekannter Typ 'vec2' -- fehlt IMPORT "vec2"?` (bei mehreren Modulen werden
> alle Kandidaten genannt). Kein „Stufe 3e"-Leak mehr in der DIM-Typ-Meldung.
> `preprocess::modules_for_type` + `compiler::unknown_dim_type_msg`. Tests: `tests/test_dim_type_error.py`.
```basic
DIM v AS VEC2              ' ohne vorheriges IMPORT "vec2"
' -> Compile-Fehler: Stufe 3e: DIM-Typ 'vec2' noch nicht unterstuetzt
```
Zwei Probleme:
1. **„Stufe 3e"** ist Compiler-interne Phasen-Benennung (Front-End-Port) und
   gehört nicht in eine User-Meldung.
2. **„noch nicht unterstuetzt"** ist sachlich falsch — der Typ existiert, nur das
   `IMPORT "vec2"` fehlt (externe Typen werden zur Compile-Zeit aus den Imports
   aufgelöst).
**Vorschlag:** z. B. `Unbekannter Typ 'vec2' — fehlt IMPORT "vec2"?`. Gilt für
alle externen Modul-Typen (astar_grid, json_handle, …). Generell mal alle
user-facing Meldungen nach „Stufe Nx"-Lecks absuchen.

---

## C — Doku-/Vertrags-Abweichungen

### C1. `NET_UDP_LAST_FROM` liefert STRING, nicht TUPLE  *(Doku diese Session gefixt)*
gbrt gibt `"host:port"` als STRING zurück (so auch der Golden-Test
`tests/test_modules_net.py`), die alte `docs/module-net.md` versprach ein
`TUPLE (host, port)`. **Doku wurde an die Realität angeglichen.**
**Optional offen:** Für Ergonomie könnte die Engine stattdessen ein echtes
Tupel liefern (dann `peer[0]`/`peer[1]` direkt) — würde Test + Doku-Rückbau
verlangen. Aktuell konsistent, also kein Muss.

---

## D — Bewusste Design-Entscheidungen, die trotzdem überraschen

(Vermutlich Absicht; hier nur dokumentiert, falls man sie je überdenken will.)

### D1. `/` liefert mal INTEGER, mal FLOAT
```basic
PRINT 8 / 2     ' 4     (glatt -> INTEGER)
PRINT 9 / 2     ' 4.5   (nicht glatt -> FLOAT)
PRINT 6 / 3     ' 2     (INTEGER)
```
Der **Ergebnistyp von `/` hängt vom Laufzeitwert ab**. Das erschwert statische
Typ-Annahmen und überrascht. Konsistenter (und in vielen Sprachen üblich): `/`
immer FLOAT, `\` für Integer-Division (existiert bereits). Bewusst so gewählt —
aber ein echter Stolperstein.

### D2. Float-Ausgabe zeigt volle f64-Präzision
```basic
PRINT 0.1 + 0.2                 ' 0.30000000000000004
PRINT CURVE_SMOOTHERSTEP(0.0,1.0,0.3)  ' 0.16308000000000003
```
Plus ein **f32→f64-Sonderfall**: `AUDIO_BUS_GET_VOLUME` nach `AUDIO_BUS_VOLUME(_, 0.8)`
liefert `0.800000011920929` (Bus-Volume wird intern als f32 gehalten, als f64
gedruckt).
**Vorschlag:** (a) Allgemein „shortest round-trip"-Float-Formatierung beim PRINT
(ryu/grisu) — `0.1+0.2` bliebe zwar `0.30000000000000004` (kürzeste exakte), aber
viele Werte würden kürzer. (b) f32-Rückgaben (Audio-Volumes) vor der f64-Anzeige
runden, dann `0.8` statt `0.800000011920929`.

### D3. `MID$` und `INSTR` sind 0-basiert
Untypisch für BASIC (klassisch 1-basiert). `INSTR` liefert `-1` bei „nicht
gefunden" (nicht `0`). Konsistent mit der 0-Index-Linie der Sprache, aber für
BASIC-Umsteiger eine Falle. (Ändern würde alles brechen — eher nur klar
dokumentieren.)

---

## E — Kleinere Inkonsistenzen

### E1. Hardware-Module sind importierbar, aber im Standard-Build tot
`IMPORT "wifi"` (serial/usb/bt ebenso) wird vom Preprocessor akzeptiert (stehen
in `KNOWN_MODULES`), aber **jeder** Funktionsaufruf wirft erst zur Laufzeit
„… Hardware-Modul 'wifi', das in diesem gbrt-Build fehlt. Neu bauen mit:
python rust\build_runtime.py --hardware". Der IMPORT gelingt, die Nutzung nicht.
**Vorschlag:** entweder im Standard-Build mitliefern, oder schon beim IMPORT
(statt erst beim ersten Call) warnen.

### E2. `step` ist reserviert
`FOR … STEP` macht `step` zum Keyword → Variablenname `step` verboten
(„i"/„tick"/„iter" stattdessen). Klein, nur erwähnenswert.

---

## Bereits behoben (diese/letzte Sessions — nicht mehr offen)

- `ATLAS_DRAW_FLIPPED`: flip_x/flip_y nehmen jetzt TRUE/FALSE **oder** 1/0; echtes
  flip_y; tint = Arg7 (war vorher inkonsistent/kaputt). *(commit 8aa315f)*
- `PHYS3D_ADD_BOX`/`ADD_SPHERE`: dynamic-Flag akzeptiert TRUE/FALSE **oder** 1/0
  (vorher nur Zahl — konsistent mit physics2d). *(commit 3928fdd)*
- Gefüllte `TRIANGLE`/`POLYGON`: füllen jetzt wicklungsunabhängig (vorher nur bei
  CCW sichtbar — raylib-Back-Face-Culling). *(commit d68efd7)*
- `docs/module-net.md`: `NET_UDP_LAST_FROM`-Rückgabetyp an die Realität
  angeglichen (STRING statt TUPLE). *(siehe C1)*
- **A2** (`NIL`-Literal) + **B1** (irreführender DIM-Typ-Fehler) — gefixt 2026-06-14,
  commit f4c8b78 (Details bei den jeweiligen Abschnitten oben). Buch-Stellen zu „NIL
  ist kein Literal" (Kap 34/69/73 + Anhang D) entsprechend korrigiert.
