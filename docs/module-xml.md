# Modul `xml`

XML lesen — Rechnungen, Ausfuhrlisten, GPX-Spuren, SVG, die Antwort einer
älteren Web-Schnittstelle.

```basic
IMPORT "xml"
```

## Nur lesend — und warum

Bei JSON war das **Schreiben** die Lücke (Punkt 2 des Audits): ein Programm
baut ständig JSON-Rümpfe für REST-Schnittstellen. Bei XML ist es umgekehrt.
Der Fall ist fast immer „Daten kommen aus einem fremden System"; einen
XML-Baum selbst zu bauen ist die Ausnahme.

Wer trotzdem XML schreiben muss, klebt es zusammen und schützt die Werte mit
`XML_ESCAPE$` — das ersetzt genau die fünf Zeichen, an denen Handarbeit sonst
bricht:

```basic
text = "<name>" + XML_ESCAPE$(kunde) + "</name>"
```

## Übersicht

| Funktion | Zweck |
|---|---|
| `XML_PARSE(text$)` → XML_HANDLE | aus einer Zeichenkette |
| `XML_LOAD(pfad$[, kodierung$])` → XML_HANDLE | aus einer Datei |
| `XML_NAME$(k)` → STRING | der Name des Elements |
| `XML_TEXT$(k[, pfad$])` → STRING | der Text (samt dem der Kinder) |
| `XML_ATTR$(k, name$[, vorgabe$])` → STRING | ein Attribut |
| `XML_HAS(k, pfad$)` → BOOLEAN | gibt es den Pfad? |
| `XML_FIND(k, pfad$)` → XML_HANDLE | dorthin springen |
| `XML_COUNT(k, pfad$)` → INTEGER | wie viele gleichnamige? |
| `XML_AT(k, pfad$, i)` → XML_HANDLE | der i-te davon (0-basiert) |
| `XML_CHILD_COUNT(k)` / `XML_CHILD(k, i)` | einen unbekannten Baum durchlaufen |
| `XML_ATTR_NAMES(k)` → ARRAY OF STRING | alle Attributnamen |
| `XML_ESCAPE$(text$)` → STRING | die fünf Sonderzeichen ersetzen |

## Pfade

Wie bei JSON, nur mit `/` statt `.` — so steht es in jedem XML-Beispiel:

```basic
IMPORT "xml"

DIM d AS XML_HANDLE
DIM p AS XML_HANDLE
DIM i AS INTEGER

d = XML_LOAD("rechnung.xml")
PRINT XML_ATTR$(d, "nr")                  ' Attribut der Wurzel
PRINT XML_TEXT$(d, "kunde")               ' Text eines Kindes

FOR i = 0 TO XML_COUNT(d, "posten/p") - 1
    p = XML_AT(d, "posten/p", i)
    PRINT XML_ATTR$(p, "menge") + "x " + XML_TEXT$(p)
NEXT
```

**Gibt es einen Namen mehrfach, nimmt `XML_FIND`/`XML_TEXT$` den ersten.** Für
alle anderen gibt es `XML_COUNT` und `XML_AT`. Ein Pfad, der raten würde, wäre
der sicherste Weg, beim zweiten Datensatz etwas anderes zu bekommen.

**Ein fehlender Pfad ist bei `XML_FIND` ein Fehler**, bei `XML_HAS` die
Antwort `FALSE` — erst fragen, dann springen. Ein fehlendes **Attribut** ist
dagegen kein Fehler, sondern liefert die Vorgabe: eine fremde Datei lässt weg,
was sie nicht braucht, und das ist der Normalfall.

## Was gelesen wird

* Elemente, Attribute (beide Anführungszeichen), Text
* selbstschließende Elemente `<b/>`
* `<?xml …?>`, `<!-- … -->`, `<!DOCTYPE …>` werden übersprungen
* `<![CDATA[…]]>` wörtlich (dort wird **nicht** aufgelöst)
* die fünf Entities `&lt; &gt; &amp; &quot; &apos;` sowie `&#65;` und `&#x41;`

**Gemischter Inhalt behält seine Reihenfolge.** `<p>Hallo <b>schöne</b>
Welt</p>` liefert bei `XML_TEXT$` genau `Hallo schöne Welt`. Das klingt
selbstverständlich, ist es aber nicht: wer Text und Kind-Elemente getrennt
speichert, bekommt `Hallo  Weltschöne` — der Fehler fällt bei Daten-XML nie
auf und bei Fließtext sofort.

**Namensräume bleiben im Namen.** `<ns:titel>` heißt hier `ns:titel`, das
`xmlns` ist ein gewöhnliches Attribut. Echte Namensraum-Auflösung braucht
einen Geltungsbereich je Element und beantwortet eine Frage, die beim Auslesen
einer bekannten Datei niemand stellt.

**Gelesen wird streng** — anders als beim [`ini`](module-ini.md)-Modul
nebenan, das kaputte Zeilen überspringt. Eine INI-Datei bearbeitet ein Mensch;
eine XML-Datei kommt aus einem anderen Programm. Ein nicht geschlossenes
Element ist dort kein Tippfehler, sondern meist ein Zeichen, dass die
Übertragung abgebrochen ist — und dann ist stilles Weiterlesen die
schlechteste Antwort. Die Meldung nennt die Zeile.

Die **Kodierung** aus [Textkodierung](builtins-core.md#textkodierung) gilt
auch hier (`XML_LOAD(pfad, "cp1252")`); die `encoding=`-Angabe in der
XML-Deklaration wird **nicht** ausgewertet.

Beispiel: [examples/175_xml.dh](../examples/175_xml.dh).
