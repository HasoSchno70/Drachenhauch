# Entwurf: Schnittstellen (INTERFACE / IMPLEMENTS)

> **Stand 22.10.2026: Weg B ist gebaut** (Klassentypen werden geprüft,
> Abschnitt 4), Weg C bis F nicht. Die Laufzeit nimmt in einem Platz vom Typ
> einer Klasse nur noch NIL und Instanzen der Klasse oder einer Unterklasse,
> `--check` warnt, wo es sicher nie passt; Prüfsammlung
> `tests/pruef/klassentypen.dhtest`. Im Bestand hat das nichts gebrochen
> (575 `.dh`-Dateien ohne neue Warnung, alle Sammlungen grün). Ein Fall in
> `compiler_warnungen.dhtest` hielt das alte Durchreichen fest und ist
> umgestellt. Der Rest des Textes ist der Entwurf, wie er zur Entscheidung
> stand.
>
> Der Entwurf nimmt eine
> Entscheidung wieder auf: Am 27.08.2026 hieß es im
> [Allzweck-Audit](allzweck-audit-2.md) (Punkt 8) „Kein `INTERFACE` …
> bleibt so“. Abschnitt 2 misst, ob die Gründe von damals tragen.
>
> Die Beispiele für **vorgeschlagene** Schreibweisen stehen als
> ```` ```text ````. `dhrt pruef` prüft jeden `basic`-Block gegen den echten
> Übersetzer und soll anschlagen, wenn jemand sie für gültig hält.

*Untersuchung, keine Umsetzung.* Dieses Papier misst den heutigen Stand an
der Laufzeit 2026.33, vergleicht fünf Wege und empfiehlt eine Reihenfolge.
Entschieden wird woanders.

## 1. Die Frage

Eine Schnittstelle ist ein Vertrag: „Wer `Zeichenbar` ist, hat
`zeichnen(cr)` und `treffer(x, y)`.“ Eine Klasse sagt den Vertrag zu,
erbt dabei aber nichts. Damit kann eine Klasse mehrere Verträge erfüllen,
ohne mehrere Elternklassen zu haben.

Drachenhauch hat heute einfache Vererbung (`EXTENDS`), angekündigte
Methoden (`ABSTRACT`) und den Typtest `x IS Klasse`. Zu klären ist:

* Fehlt etwas, das Programme brauchen?
* Was würde eine Schnittstelle in **dieser** Laufzeit kosten?
* Braucht es vorher etwas anderes?

## 2. Der Befund

Gemessen an `dhrt 2026.33` unter Windows; die Programme stehen jeweils dabei.

### (a) Klassentypen werden gar nicht geprüft

Eine Variable vom Typ `Tier` nimmt ein `Auto` an, ohne jede Meldung:

```text
CLASS Tier
    FUNCTION laut() AS STRING
        RETURN "?"
    END FUNCTION
END CLASS
CLASS Auto
    FUNCTION hupe() AS STRING
        RETURN "tut"
    END FUNCTION
END CLASS
DIM t AS Tier
t = NEW Auto()
PRINT TYPEOF(t); " "; t IS Tier
PRINT t.hupe()
PRINT t.laut()
```
```
a1.dh:14: Warnung: TIER hat kein Mitglied 'hupe' -- weder TIER selbst noch eine ihrer Eltern- oder Kindklassen. ...
AUTO FALSE
tut
Laufzeitfehler in a1.dh:15: Methode 'laut' existiert nicht in auto
```

`--check` warnt nur über die *Methode*, nicht über die Zuweisung. Ebenso
nimmt ein Parameter `x AS Datei` die Zahl 5 an. Erst der Methodenaufruf im
Rumpf bricht ab („Methodenaufruf '.name' bei nicht-Objekt (INTEGER)“), und
`--check` meldet gar nichts. Im Quelltext:

* Der Übersetzer lässt jede Zuweisung an eine Klasse, ein Feld oder eine MAP
  durch (`passt_nie` in `compiler.rs`).
* Die Laufzeit reicht Klassenwerte in `coerce` unverändert weiter (`vm.rs`).
* Für Elemente von `ARRAY OF Klasse` und `MAP OF Klasse` gilt dasselbe.

Folge für diesen Entwurf: Eine Variable `AS Zeichenbar` wäre heute nur
Dekoration, denn hineinlegen ließe sich alles. **Jede Schnittstelle setzt
voraus, dass Klassentypen geprüft werden.**

### (b) Polymorphie geht, und sie ist nicht langsam

Eine Million Aufrufe von `flaeche()` über 1000 Formen (Kreis und Quadrat im
Wechsel), Bestwert je Lauf:

| Weg | mit Maschinencode | nur VM (`DHRT_JIT=aus`) |
|---|---|---|
| `ARRAY OF Form`, `ABSTRACT FUNCTION flaeche()`, zwei Unterklassen | 73 ms | 118 ms |
| eine Klasse mit Feld `art`, `SELECT CASE art` in einer Funktion | 185 ms | 171 ms |
| nur `Kreis`, keine Vererbung | 10 ms | 106 ms |

Die Polymorphie über `ABSTRACT` ist schneller als das Art-Feld. Am
schnellsten ist der Aufruf aber, wenn nur eine Klasse vorkommt: Der
Maschinencode merkt sich *eine* Lage je Aufrufstelle. Bei gemischten Klassen
steigt er 500 000-mal mitten in der Schleife aus (`DHRT_JIT_BILANZ=1`).
Das betrifft Vererbung schon heute, mit oder ohne Schnittstellen.

### (c) Die großen Programme benutzen keine Vererbung

| | Klassen | davon mit `EXTENDS` |
|---|---|---|
| Funkenflug (`funkenflug/`) | 47 | 0 |
| Glutfaden (`examples/glutfaden/`) | 59 | 0 |
| übrige Beispiele | 39 | 6, alle in drei Lehrbeispielen (`08`, `48`, `167`) |
| IDE und Werkzeuge 183–199 | 0 | – |

`ABSTRACT` kommt außerhalb der Prüfsammlungen genau in einer Datei vor
(`examples/167_vererbung.dh`), `TYPEOF`/`IS` auf Objekten in keinem
Programm. Stattdessen wird über ein Art-Feld verzweigt; in Funkenflug und
Glutfaden zusammen in rund 50 `SELECT CASE`. Die deutlichsten Stellen:

* **Glutfaden, Werkzeuge:** 15 Werkzeuge als Zahlen (`WZ_STIFT` …
  `WZ_FARBVERLAUF`, `ansicht/werkzeuge.dh`). Drücken, Ziehen, Loslassen und
  Vorschau sind je ein `SELECT CASE wzAktiv`, rund 80 Vergleiche in sechs
  Dateien. In PySticky waren es Klassen (`PencilTool`, `GradientTool` …),
  die Kommentare nennen sie noch.
* **Glutfaden, Rückgängig:** `Aenderung` (`kern/verlauf.dh`) hat 9 Arten
  und rund 25 Felder, ein `SELECT CASE` über alle Arten wendet sie an.
* **Funkenflug, Grundformen:** `Primitiv` (`kern/modell.dh`) trägt die
  Vereinigung aller Felder von Linie bis Text. Zeichnen, Treffertest und
  Radius stehen als `SELECT` in drei Dateien.
* **Funkenflug, Auswahl:** vier Felder `gSelB/L/D/Bus` (Bauteil, Label,
  Draht, Bus) und jede Funktion viermal (`istGewaehltB/L/D`,
  `umschaltenB/L/D` …), weil es keinen gemeinsamen Typ gibt
  (`ansicht/auswahl.dh`).
* **Glutfaden, Einlesen:** `OxsImport`, `PatImport` und `XsdImport` haben
  dieselben Felder (`muster`, `warnungen`) ohne gemeinsamen Typ.
  `fremdLaden` kopiert sie dreimal von Hand in eine vierte Klasse
  `FremdImport` (`kern/verwalten.dh`).

Keiner dieser Fälle **brauchte** eine Schnittstelle: Jeder ließe sich mit
einer abstrakten Basisklasse lösen, die es seit 2026.8 gibt. Das Fehlen von
`INTERFACE` erklärt also nicht, warum niemand Vererbung benutzt. Eine
Schnittstelle allein würde an diesen Stellen nichts ändern.

### (d) Bestehender Code verlässt sich nicht auf die fehlende Prüfung

Die Typprobe `DHRT_TYPEN_PRUEFEN=1` bricht ab, wenn ein Wert nicht zu dem
Typ passt, den der Übersetzer annimmt, und erfasst dabei auch Klassen. Damit
liefen ohne einen Befund:

* der Kerntest von Funkenflug (`funkenflug/pruef_kern.dh`);
* die Sammlungen `glutfaden_kern`, `glutfaden_bearbeiten`, `vererbung`,
  `typtest`, `gebundene_methoden`.

Eine Prüfung der Klassentypen würde dort also nichts brechen. Das ist ein
Anhaltspunkt, kein Beweis: Die Probe sieht nur Ausdrücke, deren Typ der
Übersetzer kennt.

*Nebenfund:* In `objektfelder.dhtest` schlägt die Probe an. Ein Klassenfeld
`DIM tab AS MAP OF INTEGER` ist NIL, eine gleich angelegte Variable im
Hauptprogramm eine leere MAP. Der Übersetzer nimmt `MAP OF INTEGER` an, und
die Sammlung hält das NIL als Erwartung fest. Das gehört nicht hierher, ist
aber ein eigener Punkt wert. *(Behoben am selben Tag: ein solches Feld ist
jetzt eine leere MAP, je Objekt eine eigene.)*

### (e) Was die Begründung von damals übersieht

Am 27.08. hieß es, mehrere Verträge an einer Klasse bräuchten
„Mehrfachvererbung oder ein zweites Typsystem daneben“. In dieser Laufzeit
gilt beides nicht:

* **Methoden werden über ihren Namen gefunden.** `CALL_METHOD` sucht
  zur Laufzeit in der Kette der Klasse (`resolve_method`) und merkt sich das
  Ergebnis je Aufrufstelle. Eine Schnittstelle braucht darum keine eigene
  Methodentabelle und keine Umleitung: Ein Aufruf über `Zeichenbar` ist
  derselbe Befehl wie heute.
* **Was eine Schnittstelle braucht,** ist eine Liste je Klasse (welche
  Verträge sie zusagt), eine Prüfung beim Übersetzen (sind alle Methoden mit
  der richtigen Form da?) und eine Prüfung bei der Zuweisung. Die letzte
  fehlt heute schon für gewöhnliche Klassen, siehe (a).

Die Kosten liegen also nicht im Typsystem, sondern in den Werkzeugen rund
herum: Fehlermeldungen, Sprachserver, IDE, Grammatik, Handbuch, Buch.

## 3. Vorbilder

| Sprache | Form | Zuordnung | Bemerkung |
|---|---|---|---|
| VB6 | `Implements Klasse` | Methoden heißen `Schnittstelle_Methode` | jede Klasse kann als Schnittstelle dienen; gedacht für COM |
| VB.NET | `Interface … End Interface`, `Implements I` | je Methode `Implements I.M`, der Name darf abweichen | ausführlich, aber jede Zuordnung steht da |
| Xojo | „Class Interface“ | Methode gleichen Namens | einem BASIC-Nutzer am nächsten |
| FreeBASIC | keine Schnittstellen | `Extends Object`, `Virtual`/`Abstract` | nur Einfachvererbung |
| PureBasic | `Interface … EndInterface` | Reihenfolge der Methoden | beschreibt COM-Tabellen, keine Sprachverträge |
| Java, C# | `interface`, `implements` | Name und Signatur | dazu Vorgabe-Rümpfe (`default`) |
| Go | strukturell, kein `implements` | wer die Methoden hat, passt | Absicht steht nirgends |

## 4. Die Wege

### Weg A — so lassen

Einfache Vererbung und `ABSTRACT` reichen für alle Fälle in (c). Dazu
gehörte nur Doku: ein Muster „Werkzeuge als Klassen“ im Handbuch, vielleicht
ein Lehrbeispiel.

*Dafür:* Es kostet nichts, und nichts wird größer.
*Dagegen:* Lücke (a) bleibt, und ein typisiertes Feld ist weiter nur eine
Behauptung.

### Weg B — Klassentypen prüfen

Eine Zuweisung an eine Variable, einen Parameter, ein Feld, ein
Feldelement, einen MAP-Wert oder eine Rückgabe vom Typ `K` nimmt nur NIL
oder eine Instanz von `K` oder einer ihrer Unterklassen an.

* **Beim Übersetzen:** Ist der Typ des Werts bekannt und passt sicher nicht
  (`DIM t AS Tier : t = NEW Auto()`, `zeige(5)` bei `x AS Datei`), gibt es
  einen Fehler. Der Fall ist eindeutig, wie bei #423 (zweites `DIM` mit
  anderem Typ).
* **Zur Laufzeit:** `coerce` prüft die Kette der Klasse. Damit das nicht
  jede Zuweisung bremst, merkt sich die Stelle die zuletzt erlaubte Lage
  (`Instr::merk`, wie bei `CALL_METHOD`), und ein Treffer kostet einen
  Zeigervergleich.
* **Meldung**, etwa: `t: erwartet Tier (oder eine Unterklasse), bekommen Auto`.

*Dafür:* Das schließt eine echte Lücke, auch ohne Schnittstellen. Nach (d)
bricht im Bestand nichts. Und es ist die Voraussetzung für Weg C.
*Dagegen:* Es ändert das Verhalten. Wer heute absichtlich fremde Objekte in
eine falsch getypte Variable legt (Duck-Typing über `DIM x AS Irgendwas`),
bekommt einen Fehler. Für solchen Code gäbe es keinen sauberen Ersatz, denn
`ANY` fehlt (Audit Punkt 8). Weg C wäre dieser Ersatz.

### Weg C — nominale Schnittstellen (Vorschlag)

```text
INTERFACE Zeichenbar
    SUB zeichnen(cr AS INTEGER)
    FUNCTION treffer(x AS FLOAT, y AS FLOAT) AS BOOLEAN
    PROPERTY GET name() AS STRING
END INTERFACE

INTERFACE Element EXTENDS Zeichenbar, Speicherbar
END INTERFACE

CLASS Widerstand IMPLEMENTS Zeichenbar, Speicherbar
    ...
END CLASS

CLASS Kreis EXTENDS Form IMPLEMENTS Zeichenbar
    ...
END CLASS

DIM auswahl AS ARRAY OF Zeichenbar : auswahl = []
ARRAY_PUSH(auswahl, NEW Widerstand())
FOR EACH z IN auswahl
    z.zeichnen(cr)
NEXT
IF teil IS Speicherbar THEN ...
```

Die Regeln:

1. **Zuordnung über den Namen** wie bei Xojo, nicht über eine Liste je Methode
   wie bei VB.NET. Die Laufzeit findet Methoden ohnehin über den Namen; eine
   abweichende Zuordnung (`zeichnen` erfüllt `Malbar.malen`) bräuchte eine
   Umleitung, die es sonst nirgends gibt.
2. **Die Form muss genau stimmen:** Zahl und Typen der Parameter, `BYREF`
   und Rückgabetyp. Die Parameternamen dürfen abweichen. Geerbte Methoden
   zählen mit.
3. Eine Klasse mit offenem `ABSTRACT` darf einen Teil des Vertrags ihren
   Unterklassen überlassen; `NEW` geht erst auf die fertige Klasse (wie
   heute). Unterklassen erben die Zusagen ihrer Eltern.
4. **In einer Schnittstelle stehen nur Köpfe:** `SUB`, `FUNCTION`,
   `PROPERTY GET/SET`. Keine Felder, keine Rümpfe, kein `Init`, keine
   Operatoren, keine Konstanten. Ohne Vorgabe-Rümpfe entsteht keine
   Mehrfachvererbung von Code durch die Hintertür.
5. **Schnittstellen können erben,** auch von mehreren
   (`INTERFACE Element EXTENDS Zeichenbar, Speicherbar`): Dort gibt es nichts
   zu erben außer Köpfen, also keinen Konflikt. Gleiche Namen mit
   verschiedener Form sind ein Fehler.
6. **Als Typ** überall, wo eine Klasse stehen darf: Variable, Parameter,
   Rückgabe, Feld, `ARRAY OF`, `MAP OF`. Die Zuweisung prüft wie in Weg B,
   ob die Klasse die Schnittstelle zusagt.
7. **`x IS Zeichenbar`** fragt die Zusage ab; `TYPEOF` bleibt der
   Klassenname.
8. **Aufruf über eine Schnittstelle:** Nur ihre Mitglieder sind erlaubt
   (Fehler, nicht Warnung). Weil die Form feststeht, geht hier `BYREF`, das
   bei Methoden heute nur mit bekannter Klasse geht. Benannte Argumente
   gehen bei Methoden heute gar nicht („Named-Args nur bei
   SUB/FUNCTION/NEW“); über eine Schnittstelle wären sie möglich. Ein Feld
   ist über die Schnittstelle nicht erreichbar.
9. `z.zeichnen` ohne Klammern ist wie heute eine gebundene Methode.

**Wörter:** `INTERFACE` und `IMPLEMENTS` werden wie `ABSTRACT` und `SUPER`
**nicht** reserviert, sondern am Ort erkannt: `INTERFACE` am Anfang einer
Anweisung mit einem Namen dahinter, `IMPLEMENTS` im Kopf von `CLASS`. Eine
Variable `interface` bleibt erlaubt. Heute ist `IMPLEMENTS` im Kopf einer
Klasse ein Parse-Fehler mit dem Hinweis auf `EXTENDS`
(„Eine Klasse erbt mit EXTENDS“), der Hinweis müsste dann entfallen.

**Meldungen**, so wie sie in einem fertigen Bau aussehen sollten:

```text
spiel.dh:12: Compile-Fehler: Widerstand sagt Zeichenbar zu, aber treffer fehlt
  -- verlangt: FUNCTION treffer(x AS FLOAT, y AS FLOAT) AS BOOLEAN
spiel.dh:12: Compile-Fehler: Widerstand.zeichnen passt nicht zu Zeichenbar
  -- verlangt: SUB zeichnen(cr AS INTEGER), steht da: SUB zeichnen(cr AS FLOAT)
spiel.dh:30: Compile-Fehler: auswahl ist ARRAY OF Zeichenbar, Draht sagt Zeichenbar nicht zu
spiel.dh:41: Compile-Fehler: Zeichenbar hat kein Mitglied 'farbe'
```

**Was sonst mitzieht:**

* **Laufzeit/.dhc:** Je Klasse eine Liste ihrer Zusagen, mit den
  Eltern zusammengeführt. `IS` und `coerce` fragen sie ab. Ein
  Methodenaufruf bleibt `CALL_METHOD`.
* **Namensräume:** `kern.Zeichenbar` wie `kern.Knoten`. `namensraum.rs`
  braucht dafür eine Menge mehr in `Modul`, einen Zweig in `sammeln` und
  einen im Umschreiben der Typen.
* **Sprachserver und IDE:** Gliederung mit eigener Art, Hover zeigt den
  Vertrag, „Zur Definition“ springt zur Schnittstelle. Neu wäre
  „Implementierungen finden“ (welche Klassen sagen es zu?).
  `symbole.rs` braucht `END INTERFACE` als Blockende; `klassenLesen` in der
  IDE liest dann auch `IMPLEMENTS`.
* **Grammatik:** Die beiden Wörter kommen in die Liste der Deklarationen
  (`doku.rs`). Bisher stehen dort nur Wörter aus `lexer::KEYWORDS`, also
  ein kleiner eigener Weg für zwei Kontextwörter.
* **Maschinencode:** Ein Platz vom Typ einer Schnittstelle wird wie ein Platz
  ohne Typ behandelt (Wertemodus); schneller als heute wird er dadurch nicht.
* **Doku:** `docs/sprache.md` (de/en), ein Lehrbeispiel, das Referenzbuch.

*Dafür:* Mehrere Verträge an einer Klasse; ein Typ für gemischte Felder
(der Ersatz für das fehlende `ANY`, aber mit Zusage); Fehler beim
Übersetzen statt beim Laufen; `BYREF` über die Schnittstelle auch ohne
bekannte Klasse, benannte Argumente bei Methoden zum ersten Mal. Die Laufzeit bleibt dabei, wie sie ist.
*Dagegen:* Ein Begriff mehr für Einsteiger. Und (c) zeigt, dass die
vorhandene Vererbung kaum genutzt wird; dass Schnittstellen es würden, ist
nicht belegt.

### Weg D — strukturelle Schnittstellen (wie Go)

`INTERFACE` wie in Weg C, aber ohne `IMPLEMENTS`: Jede Klasse mit
passenden Methoden passt.

*Dafür:* Es passt zur Laufzeit, die ohnehin nur nach Namen fragt. Auch
Klassen aus fremden Bibliotheken passen, ohne dass sie etwas zusagen.
*Dagegen:* Die Absicht steht nirgends. Ein Fehler zeigt sich an der
Zuweisung, weit weg von der Klasse, und eine Klasse passt auch aus
Versehen. `IS` müsste jedes Mal alle Methoden vergleichen oder sich das
merken. Für eine Sprache, die Einsteiger mit genauen Meldungen führen will,
ist das der schwächere Weg.

### Weg E — Mehrfachvererbung

Nein. Felder, `Init`-Reihenfolge, `SUPER` und die Lage im Speicher
(`Layout`, Vorfahren zuerst) setzen alle eine einzige Elternkette voraus.

### Daneben: Weg F — Maschinencode für mehrere Klassen an einer Stelle

Unabhängig von A bis E: Die Aufrufstelle merkt sich heute eine Lage. Mit
zwei bis vier Lagen je Stelle (ein kleiner Zwischenspeicher wie bei
JavaScript-Maschinen) liefe das Beispiel aus (b) ohne Aussteigen.
Gewinnen würde jedes polymorphe Programm, schon heute mit `ABSTRACT`. Die
Zahl dazu würde erst gemessen; aus (b) ist nur bekannt, dass es zwischen
10 und 73 ms liegen muss.

## 5. Empfehlung

1. **Zuerst Weg B, unabhängig von allem anderen.** Die fehlende Prüfung der
   Klassentypen ist eine Lücke, kein Komfortpunkt: Eine Variable `AS Tier`
   behauptet etwas, das niemand prüft. Nach (d) bricht im Bestand nichts.
   In den Notizen der Fassung steht es als Verhaltensänderung oben.
2. **Weg C danach, mit einer Probe am echten Programm.** Die Auswahl in
   Funkenflug (vier Felder, jede Funktion viermal) wird einmal mit einer
   abstrakten Basisklasse umgebaut. Reicht die Basisklasse, ist das ein
   Argument für Weg A. Braucht die Auswahl zwei Verträge (zeichenbar und
   speicherbar, aber verschieden geerbt), wird C gebaut. So bliebe es bei
   der bisherigen Arbeitsweise: erst ein Pilot, dann die Sprache.
3. **Weg F, wann immer Zeit ist:** Er beschleunigt, was es schon gibt, und
   hängt an keiner Sprachentscheidung.
4. **Weg D und E nicht.**

Unabhängig vom Ausgang gehört die Zeile im Allzweck-Audit berichtigt: Die
Begründung „zweites Typsystem“ trifft auf diese Laufzeit nicht zu, siehe 2 (e).

## 6. Offene Fragen an die Entscheidung

* Ist die Verhaltensänderung aus Weg B in Ordnung, und kommt sie als Fehler
  oder eine Fassung lang erst als Warnung?
* `INTERFACE`/`IMPLEMENTS` als Kontextwörter (Vorschlag) oder reserviert?
* Soll die Form streng gleich sein (Vorschlag), oder darf eine
  Implementierung zusätzliche optionale Parameter haben?
* Vorgabe-Rümpfe in Schnittstellen (Java `default`)? Vorschlag: nein.
