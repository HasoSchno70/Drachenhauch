# Allzweck-Audit, zweite Runde (2026-08-23)

Die [Allzweck-Roadmap](allzweck-roadmap.md) ist abgearbeitet — WP A bis J, plus
Namensräume und Plattform. Dieses Dokument stellt dieselbe Frage noch einmal,
jetzt aber vor dem größeren Anspruch: **würde jemand Drachenhauch wählen, um
damit Software zu bauen** — Werkzeuge, Auswertungen, kaufmännische Anwendungen,
Dienste?

Der BASIC-Dialekt bleibt dabei unangetastet. Keiner der Punkte hier verlangt
eine neue Syntax-Philosophie; die meisten sind Prüfungen, Bibliothek oder
Werkzeug drumherum.

**Alles unten ist gegen die gebaute Runtime nachgemessen, nicht aus der Doku
abgeschrieben.**

## Der Befund in drei Sätzen

1. **Der Übersetzer prüft weniger, als die Sprache verspricht.** Drachenhauch
   ist zur *Laufzeit* Pascal-streng und beim *Übersetzen* fast typenblind. Das
   ist die größte Lücke, und es ist keine Bibliotheksarbeit.
2. **Daten kommen schwerer rein und raus, als sie sollten.** JSON lässt sich
   lesen, aber nicht schreiben; eine von Excel geschriebene CSV-Datei lässt
   sich gar nicht lesen; die Standardeingabe gibt es nicht.
3. **Um die Sprache herum fehlt die Werkzeugkette**, an der man abliest, dass
   sie zum Arbeiten gedacht ist: kein Test-Läufer, kein Formatierer, kein Weg,
   eine Bibliothek zu teilen — und kein `--version`.

---

## 1 — Statische Typprüfung (größte Lücke)

Die Typen sind da: `DIM x AS INTEGER`, Signaturen mit Typen, Klassen mit
Feldern. Zur Laufzeit werden sie hart durchgesetzt. Beim Übersetzen fällt
davon fast nichts auf.

Was `dhrt --check` heute **findet**: fehlende und zu viele Argumente,
unbekannte Builtins, eine Kommazahl an eine ganzzahlige Variable (seit
`3e9211d`).

Was es **nicht** findet — jedes davon nachgemessen, jedes bricht erst zur
Laufzeit ab, und nur dann, wenn die Zeile auch wirklich ausgeführt wird:

| Quelltext | `--check` | zur Laufzeit |
|---|---|---|
| `DIM s AS STRING : s = 5` | *nichts* | `Zuweisung an global: Erwartet STRING, erhalten INTEGER` |
| `DIM i AS INTEGER : i = "text"` | *nichts* | `Erwartet INTEGER, erhalten STRING` |
| `f(s)` mit `s AS STRING`, `f(a AS INTEGER)` | *nichts* | `Parameter: Erwartet INTEGER, erhalten STRING` |
| `a.gibtsNicht()` bei `DIM a AS A` | *nichts* | `Methode 'gibtsnicht' existiert nicht` |
| `a.feldGibtsNicht` bei `DIM a AS A` | *nichts* | Laufzeitfehler |
| `FUNCTION f() AS INTEGER` ohne `RETURN` | *nichts* | liefert still **NIL** |

Die letzte Zeile ist die unangenehmste: eine Funktion, die `AS INTEGER`
verspricht, darf keinen NIL zurückgeben. Das ist ein Loch im Typsystem selbst,
kein fehlender Hinweis.

**Warum das der wichtigste Punkt ist.** Bei einem Spiel läuft jeder Pfad
innerhalb von Minuten einmal. Bei Software läuft der Zweig „Rechnung stornieren
mit Teilzahlung" vielleicht das erste Mal beim Kunden. Ein Tippfehler in einem
Feldnamen geht heute durch Übersetzung, Start und Test hindurch und schlägt
dort zu. Genau davor soll eine streng getypte Sprache schützen — das ist ihr
ganzer Handel: Strenge beim Schreiben gegen Ruhe im Betrieb. Die Hälfte davon
wird gerade nicht eingelöst.

**Die Maschinerie steht schon.** Der Compiler kennt die Typen der Globals, der
Locals, der Parameter und der Klassenfelder — er benutzt sie bereits für
spezialisierte Opcodes und für die Kommazahl-Warnung. Es geht um ein
Ausdruckstyp-Urteil an den vier Stellen Zuweisung, Argumentbindung,
Feld-/Methodenzugriff und Rückgabe. Konservativ bleiben, wo der Typ nicht
sicher bestimmbar ist (unbekannt heißt: kein Fund), sonst reißt es bestehenden
Code auf.

Ein Zwischenschritt, der ohne Risiko geht: **erst als `warning` melden**, so
wie es die Kommazahl-Warnung schon macht. Der Editor zeigt es an, kein
bestehendes Programm hört auf zu laufen.

## 2 — JSON kann nur gelesen, nicht geschrieben werden

`JSON_PARSE`/`LOAD`/`GET_*`/`HAS`/`LEN`/`TYPE`/`STRINGIFY`/`PRETTY` — das ist
der ganze Satz. Es gibt **kein `JSON_NEW`, kein `JSON_SET`, kein `JSON_ADD`**,
und `JSON_STRINGIFY` nimmt ausschließlich ein Handle aus `JSON_PARSE`:

```
PRINT JSON_STRINGIFY(k)   ' k ist eine Klasseninstanz
-> Laufzeitfehler: JSON_STRINGIFY erwartet JSON-Handle (aus JSON_PARSE)
```

Wer heute JSON schreiben will, klebt Zeichenketten zusammen — und bricht am
ersten Anführungszeichen, Backslash oder Zeilenumbruch in einem Nutzernamen.
Einen Escape-Helfer gibt es auch nicht.

Das trifft ausgerechnet die Fähigkeit, die WP C gerade gebaut hat: Seit
`HTTP_REQUEST` mit PUT/PATCH kann man jede angemeldete REST-Schnittstelle
*anrufen*, aber ihren Rumpf nicht verlässlich *bauen*. Ebenso: Konfigurations-
dateien schreiben, Zustand sichern, Daten an ein anderes Programm übergeben.

Vorschlag, klein gehalten: `JSON_NEW_OBJECT`/`JSON_NEW_ARRAY`,
`JSON_SET_STRING/INT/FLOAT/BOOL/NULL` (Pfad-Notation wie beim Lesen),
`JSON_APPEND_*`, `JSON_SET_JSON` zum Verschachteln. Dazu die naheliegende
Bequemlichkeit: `JSON_FROM_MAP`.

## 3 — Fremde Textkodierung

Eine Datei, die nicht UTF-8 ist, ist heute unlesbar:

```
READLINES("latin.txt")   -> stream did not contain valid UTF-8
CSV_LOAD("k.csv", ";")   -> CSV_LOAD: k.csv: stream did not contain valid UTF-8
```

Genau diese Datei schreibt Excel auf einem deutschen Windows, wenn man „CSV
(Trennzeichen-getrennt)" wählt: Windows-1252. Die häufigste Herkunft von Daten,
die jemand auswerten will, ist also die eine, die nicht durch die Tür passt —
und die Meldung nennt weder den Grund noch einen Ausweg.

Der Umweg über `READALL_BYTES` + Byte-für-Byte-Umsetzung existiert (seit WP B),
aber niemand sollte ihn gehen müssen, um eine Kundenliste zu zählen.

Vorschlag: optionaler Kodierungs-Parameter bei `READLINES`, `READALL`,
`WRITEALL`, `CSV_LOAD`, `CSV_SAVE` — `"utf8"` (Vorgabe), `"cp1252"`,
`"latin1"`. Und die Fehlermeldung soll den Parameter nennen.

## 4 — Standardeingabe: Filterprogramme lassen sich nicht schreiben

WP A hat „Werkzeug in einer Kette" ausdrücklich zum Ziel erklärt. Der halbe
Weg fehlt noch: ein Programm kann lesen, was ihm als *Argument* gegeben wird,
aber nicht, was ihm *gereicht* wird.

- Es gibt kein stdin-Handle: `ENDOFFILE(0)` → `erwartet FILE`.
- `INPUT` liest zwar aus einer Pipe, aber
  - es druckt sein `? ` **nach stdout** — mitten in die Nutzdaten,
  - und liefert nach dem Dateiende endlos Leerstrings, ohne dass man das Ende
    erkennen kann. Eine Schleife läuft ewig oder rät.

Nachgestellt mit `printf 'a\nb\n' | dhrt run p5.dh`: Ausgabe
`? [a]` `? [b]` `? []` `? []`.

Damit ist `dir | meinwerkzeug | sort` nicht schreibbar — die klassische
Bauform eines Werkzeugs.

Vorschlag: `STDIN_LINE$`, `STDIN_EOF`, `STDIN_ALL$`, `STDIN_BYTES` — hier
bewusst ohne Klammern geschrieben, weil es sie noch nicht gibt und die
Doku-Prüfung jeden Namen mit angehängter Klammer gegen die echten Builtins
hält.
Alternativ `OPENFILE("-", "r")`, damit die vorhandenen Datei-Builtins gelten —
das wäre die kleinere Fläche.

## 5 — Bibliotheken teilen (Ökosystem)

`IMPORT "x.dh"` löst **ausschließlich relativ zur importierenden Datei** auf
(`preprocess.rs`: `base.join(rel)`). Es gibt keinen Suchpfad, keinen
Bibliotheksordner, keine Version, keine Paketverwaltung.

Folge: eine geteilte Bibliothek wird in jedes Projekt kopiert. Damit gibt es
keine Aktualisierung, kein „ich benutze dieselbe Datumsbibliothek wie du" und
für Drachenhauch keinen Ort, an dem eine Gemeinschaft entstehen könnte.

Die Roadmap hatte die Paketverwaltung mit einer Begründung gestrichen, die
inzwischen entfallen ist: *„Ohne WP I hätte sie nichts, was sie verwalten
könnte."* — WP I ist gebaut. Namensräume, `PRIVATE` und Meldungen mit
Dateiangabe stehen.

Der kleinste sinnvolle Schritt ist nicht die Paketverwaltung, sondern der
Suchpfad davor: `DH_PATH` bzw. ein `bibliothek/`-Ordner neben der Runtime, in
dem `IMPORT "zeitraum.dh"` gefunden wird. Alles Weitere (Herunterladen,
Versionen, Abhängigkeiten) kann darauf aufbauen — oder auch nie kommen, ohne
dass es weh tut.

## 6 — Die Werkzeugkette um die Sprache

`dhrt` kann: `run`, `call`, `profile`, `debug`, `--check`, `--export`, dazu die
Entwickler-Einstiege `--tokens`/`--ast`/`--dumpbc`/`--preprocess`/`--runsrc`.

Was fehlt:

- **`dhrt --version`** — gibt es schlicht nicht (die Antwort ist *„Kann
  '--version' nicht lesen"*, weil der Name für eine Datei gehalten wird). Eine
  Kleinigkeit von zehn Minuten und zugleich das Erste, was jemand tippt, der
  prüfen will, ob er die richtige Fassung hat.
- **Ein Test-Läufer.** Die Bausteine stehen seit WP E (`ASSERT`,
  `ASSERT_COLLECT`, `ASSERT_REPORT`, Rückgabewert). Was fehlt, ist das Dach:
  `dhrt test verzeichnis/` — Dateien einsammeln, jede laufen lassen, eine
  Bilanz und einen Rückgabewert liefern. Ohne das schreibt jedes Projekt sich
  seinen eigenen Läufer, so wie `buch-tippspiel` sich vor WP E seinen eigenen
  `ASSERT` geschrieben hat.
- **Ein Formatierer** (`dhrt fmt`). Bei einer Sprache mit Einrückungs-Freiheit
  und Groß-/Kleinschreibungs-Unempfindlichkeit ist das mehr wert als anderswo:
  er beendet jede Diskussion über `IF x THEN` gegen `If X Then` und macht
  fremden Code auf einen Schlag lesbar.
- **Ein Doku-Erzeuger.** Der Sprachserver liest bereits die Kommentare über
  einer Funktion (`extract_user_doc`) — daraus eine Referenz für eine eigene
  Bibliothek zu erzeugen, ist ein kurzer Weg mit sichtbarem Ergebnis.

## 7 — Bibliothekslücken (Fleißarbeit, jede für sich klein)

Nachgemessen am eingefrorenen Builtin-Index (1401 Einträge) — keiner dieser
Bereiche hat heute einen Vertreter:

- **Dateisystem unvollständig:** `MKDIR` gibt es, ein Gegenstück zum Löschen
  eines Verzeichnisses nicht. Ebenso fehlen Zeitstempel (wann zuletzt geändert
  — die Grundlage jeder Sicherung und jedes „was ist neu"), rekursives
  Auflisten, Namensmuster (`*.csv`), ein Temp-Verzeichnis.
- **Nur SQLite.** Für eine Anwendung in einem Betrieb liegen die Daten in
  PostgreSQL, MySQL oder MS SQL. Heute geht das nur über ein fremdes Werkzeug
  per `SHELL`.
- **Keine Konfigurations-/Austauschformate außer JSON und CSV:** kein INI,
  kein XML, kein YAML, kein TOML.
- **Kein PDF, kein Druck.** Rechnung, Lieferschein, Bericht, Etikett — für
  kaufmännische Software fast immer die erste Forderung nach „speichern".
  Heute endet der Weg beim Bildschirm oder bei einer Textdatei.
- **Kein XLSX**, obwohl CSV steht — für Auswertungen, die weitergegeben
  werden, ist das der erwartete Behälter.
- **Kein E-Mail-Versand** (SMTP). Ein Bericht, der sich selbst verschickt, ist
  bei Werkzeugen die Regel, nicht die Ausnahme.
- **Kein HTTP-Server.** Die Roadmap hat ihn gestrichen (*„wer wirklich einen
  Dienst braucht, stellt einen fertigen Server davor"*). Das würde ich vor dem
  Bastler-Leitbild neu bewerten: mit `mqtt`, `firmata`, `serial` und `net` an
  Bord fehlt für „meine Heizungssteuerung hat eine kleine Weboberfläche" genau
  ein Baustein, und es ist der kleinste von allen (`NET_TCP_LISTEN` steht schon
  darunter).
- **Kein Festkomma für Geld.** `0.1 + 0.2` ergibt `0.30000000000000004`. Mit
  `FORMAT$` sieht man das nicht mehr, aber summiert wird trotzdem falsch. Für
  eine Sprache, in der jemand eine Kasse schreiben soll, ist das eine
  Entscheidung wert (eigener Typ, oder die dokumentierte Regel „in Cent
  rechnen").

## 8 — Sprachkomfort, bewusst zu entscheiden

Kein Punkt hier ist ein Mangel; alle drei sind Abwägungen, die man einmal
bewusst treffen und aufschreiben sollte.

- **Keine anonymen Funktionen, keine Closures, keine verschachtelten
  Funktionen.** `FUNCREF` verlangt immer eine benannte Funktion auf oberster
  Ebene, und ihr Rumpf sieht nur Parameter und Globals. Für Rückrufe (GUI,
  `timer`, Vergleichsfunktion beim Sortieren) reicht das; es kostet pro Rückruf
  eine benannte Funktion an anderer Stelle der Datei. Das passt zu BASIC — ich
  würde es lassen und die Entscheidung dokumentieren.
- **`ANY` gibt es nicht.** Die Roadmap begründet den Verzicht auf Generics
  ausdrücklich mit *„`ARRAY OF ANY` plus Laufzeit-Typprüfung (`TYPEOF`) deckt
  die realen Fälle"* — aber `DIM x AS ANY` ist ein Übersetzungsfehler
  (`is_value_type` in `compiler.rs` kennt den Namen nicht). Der Ersatz für
  Generics existiert also nicht. Entweder bauen (klein: ein Name mehr in
  `is_value_type`, Umwandlung als Durchreiche — die interne Typ-Bezeichnung
  `any` gibt es schon) oder die Begründung berichtigen. Heute steht die
  Verzichts-Entscheidung auf einer Stütze, die es nicht gibt.
- **Nur Einfachvererbung, Schnittstellen über `ABSTRACT`.** Für
  Steckmodul-Architekturen („alle Ausgabeformate haben `Schreibe()`") ist eine
  abstrakte Basisklasse eine tragfähige Antwort. Kein Handlungsbedarf.

---

## Vorgeschlagene Reihenfolge

**1 → 2 → 3 → 4 → 6 → 5 → 7**, nach demselben Maßstab wie in der ersten
Roadmap („wie viele neue Programme macht das möglich, pro Aufwand"):

- **1 (Typprüfung)** zuerst, weil es als einziges *jedes* bestehende und
  künftige Programm besser macht, ohne dass jemand etwas dazulernen muss. Und
  weil es das Versprechen einlöst, mit dem die Sprache antritt.
- **2, 3, 4** sind zusammen „Daten rein und raus". Sie öffnen die Klasse
  Programme, die Daten von woanders holt, umformt und weitergibt — das ist der
  Großteil dessen, was Leute „Software" nennen.
- **6** ist billig und wirkt nach außen: `--version`, ein Test-Läufer, ein
  Formatierer sind die Zeichen, an denen jemand erkennt, ob eine Sprache zum
  Arbeiten taugt.
- **5** ist die Voraussetzung dafür, dass andere etwas beitragen können —
  wichtig, aber wirkungslos, solange 1 bis 4 offen sind.
- **7** ist Fleißarbeit und lässt sich nach Bedarf abrufen. Der erste Griff
  daraus wäre PDF (kaufmännisch) oder der HTTP-Server (Bastler), je nachdem,
  wen man zuerst gewinnen will.

## Was ausdrücklich NICHT fehlt

Damit die Liste einordbar bleibt — das hier ist alles nachgeprüft und trägt:

Polymorphie über Arrays (`DIM t[2] AS Tier` mit `Hund` darin ruft die
überschriebene Methode), verschachtelte Behälter (`MAP OF ARRAY OF INTEGER`
funktioniert auch zur Laufzeit), Sortieren mit eigener Vergleichsfunktion über
*beliebige* 1D-Arrays (also auch über Datensätze), Unicode-Zeichenketten
(`LEN("Grüße 😀")` = 7, Index liefert das Emoji), Ganzzahl-Überlauf als
Fehler statt als stiller Umbruch, Zeichenketten-Anhängen in Schleifen
(20 000 Anhänge in 13 ms), Übersetzungszeit (1922 Zeilen in 40 ms — getrennte
Übersetzung braucht niemand).
