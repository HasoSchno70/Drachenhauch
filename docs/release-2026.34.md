# Drachenhauch 2026.34

*Die Notizen zu dieser Fassung. Im Mittelpunkt stehen **Klassen und Felder**:
Der Typ einer Klasse gilt jetzt überall, Felder behalten ihren Elementtyp,
und Typfehler nennen die Variable oder den Parameter beim Namen. Der
Maschinencode kommt mit mehreren Klassen an einer Stelle zurecht und ist bei
Objekten deutlich schneller. **Zwei Änderungen betreffen bestehende
Programme**, sie stehen gleich zuerst.*

## Was sich für bestehende Programme ändert

* **Der Typ einer Klasse wird geprüft.** Eine Variable, ein Parameter, ein
  Feld, ein Feldelement, ein MAP-Wert oder eine Rückgabe vom Typ `Tier` nimmt
  nur noch `NIL` und Instanzen von `Tier` oder einer Unterklasse. Bisher
  reichte die Laufzeit alles durch: `DIM t AS Tier : t = NEW Auto()` lief, und
  der Fehler kam erst beim nächsten Methodenaufruf, weit weg von der Stelle.
  Jetzt bricht die Zuweisung selbst ab:

  ```
  Laufzeitfehler in spiel.dh:12: Zuweisung an 't': Erwartet TIER (oder eine Unterklasse), erhalten AUTO
  ```

  `dhrt --check` warnt vorher, wo es sicher nie passt. Herunter
  (`DIM h AS Hund : h = einTier`) geht weiter, wenn in `einTier` wirklich ein
  Hund steckt.
* **Felder behalten ihren Elementtyp.** Ein Feld weiß zur Laufzeit, was es
  enthält, und prüft jeden Schreibzugriff selbst. Daraus folgt:
  * Ein Literal mit Objekten wird beim Zuweisen an ein `ARRAY OF X` Element
    für Element geprüft. Bisher ging `h = [NEW Katze()]` an ein
    `ARRAY OF Hund` ungeprüft durch.
  * Ein Feld einer Unterklasse passt an ein Feld der Vorfahrin
    (`ARRAY OF Hund` an `ARRAY OF Tier`). Über den neuen Namen kommt trotzdem
    keine Katze hinein. Bisher hing es daran, wie das Feld entstanden war.

  Im Repository bricht dadurch kein Programm ab (geprüft mit `dhrt --check`
  über alle `.dh`-Dateien und mit allen Prüfsammlungen).

## Neu in dieser Fassung

### `ARRAY_OF_TYPE(feld, Typ)`

Holt die Elemente eines Typs als neues Feld, in ihrer Reihenfolge. Rechts
steht ein Typname wie bei `IS`: eine Klasse samt Unterklassen, ein Werttyp wie
`INTEGER` oder ein Name mit Namensraum.

```basic
CLASS Tier
END CLASS
CLASS Hund EXTENDS Tier
END CLASS
CLASS Katze EXTENDS Tier
END CLASS

DIM alle AS ARRAY OF Tier
alle = [NEW Hund(), NEW Katze(), NEW Hund()]
DIM hunde AS ARRAY OF Hund
hunde = ARRAY_OF_TYPE(alle, Hund)
PRINT LEN(hunde)
```

### Meldungen nennen, was gemeint ist

Typfehler sagten bisher „Zuweisung an global“ oder „Parameter“. Jetzt heißt
es `Zuweisung an 'zahl'`, `Parameter 't' von futter` oder `RETURN von finde`,
auch bei Aufrufen über eine FUNCREF, ein Lambda oder einen Rückruf
(`Parameter 'n' einer Lambda-Funktion`). Der Name wird erst gesucht, wenn
ein Fehler auftritt; im Normalfall kostet das nichts.

### Der Übersetzer warnt bei Feldern, die nie passen

`dhrt --check` warnt bei einem `ARRAY OF Hund` an ein `ARRAY OF Katze`, einem
`ARRAY OF STRING` an ein `ARRAY OF INTEGER`, einer Zahl an ein Feld und einer
Variable `ARRAY OF INTEGER` an ein `ARRAY OF FLOAT` (umgebaut wird nur ein
frisches Feld). Die Warnung zitiert die Meldung der Laufzeit und nennt einen
Ausweg.

### Weitere Änderungen

* Ein Klassenfeld `DIM tab AS MAP OF INTEGER` ist eine leere MAP wie im
  Hauptprogramm, nicht mehr NIL.
* Die IDE leert die Gliederung, wenn der letzte Reiter geschlossen wird.
* Glutfaden zoomt mit dem Mausrad auch wieder heraus.
* Glutfaden: die drei Einleser (OXS, PAT, XSD) teilen sich ein Ergebnis und
  eine Basisklasse für das Lesen der Bytes. Funkenflug: die Auswahl ist eine
  Menge aus `kern.Element` statt vier Feldern.
* `docs/entwurf-schnittstellen.md` wägt `INTERFACE`/`IMPLEMENTS` ab. Zwei
  Proben an Funkenflug und Glutfaden kamen mit einer abstrakten Basisklasse
  aus; `INTERFACE` wird vorerst nicht gebaut.

## Schneller

Der Maschinencode kennt jetzt **mehrere Klassen an einer Stelle**: Eine
Objektart kann eine Familie aus bis zu vier Klassen mit gemeinsamer Vorfahrin
sein, ein Methodenaufruf verzweigt über die Klasse des Objekts, und eine
Variable oder ein Feld vom Typ einer Basisklasse wird gleich als Familie
geführt. Dazu lesen und schreiben die Feld-Helfer Zahlen an ihrem Platz.

| | 2026.33 | 2026.34 |
|---|---|---|
| eine Million Methodenaufrufe, zwei Klassen | 72 ms | 8 ms |
| lokale Variable vom Typ der Basisklasse | 134 ms | 8 ms |
| Feld vom Typ der Basisklasse | 146 ms | 9 ms |
| `tools/tempo/teilchen.dh` | 27,7 ms | 12,8 ms |
| `tools/tempo/felder.dh` | 12,9 ms | 11,4 ms |
| `tools/tempo/objekte.dh` | 11,9 ms | 11,0 ms |

Die übrigen Programme in `tools/tempo` sind gleich schnell oder schneller.

## Unter der Haube

**Zahlen.** 2154 Befehle in 48 Modulen (neu: `ARRAY_OF_TYPE`), 225 Beispiele
auf der obersten Ebene von `examples/`, **5408 Fälle in 436 Prüfsammlungen**
(vorher 5328 in 431) und 560 Rust-Testfunktionen.

* Das Verzeichnis der Klassen kennt jetzt die Vorfahren jeder Klasse; damit
  prüft die Laufzeit Klassen- und Feldtypen ohne das Programm zu fragen.
* Die Namen in Typfehlern sucht die Laufzeit erst, wenn der Fehler die
  Funktion verlässt (`fehler_benennen`). Eine erste Fassung, die den Namen
  vorher zusammensetzte, kostete `teilchen.dh` 9 %.

## Was offen bleibt

* **raylib 6.2** ist erschienen, die Rust-Bindung raylib-rs hat aber noch
  keine Fassung dafür. Drachenhauch bleibt bei raylib 6.0, bis raylib-rs
  nachzieht.
* Funkenflug, Glutfaden und der Fotobetrachter sind nur unter Windows
  ausprobiert (mit dem GTK von Inkscape).
* Auf dem Handy öffnet die Leinwand keine Bildschirmtastatur.
* Zwei echte Rechner hinter Routern und Firewalls sind nicht ausprobiert.
* Ein Struct als Wert hinter `...`, ein `va_list` im Rückruf; C++ und COM.
* Das macOS-Paket ist nicht beglaubigt, der Installer nicht signiert.
