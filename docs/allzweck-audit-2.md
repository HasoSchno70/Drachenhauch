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

> **Stand 2026-08-24: alle acht Punkte sind abgearbeitet.** 1–6 gebaut,
> die Entscheidungen unter 8 getroffen, und aus **7** sind sieben Module
> geworden (`httpd`, `ini`, `xml`, `pdf`, `xlsx`, `smtp` plus der
> Dateisystem-Ausbau). Die zwei Punkte, die dort keine Bibliotheksarbeit
> waren, sind untersucht statt gebaut: [Geld](entwurf-geldtyp.md) und
> [Datenbank-Treiber](entwurf-datenbanktreiber.md) — beide mit Messung,
> Entwurf und Empfehlung, die Entscheidung steht aus.

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

## 1 — Statische Typprüfung (größte Lücke) — ✅ ERLEDIGT 2026-08-23

> **Gebaut.** `--check` meldet jetzt alle sechs Zeilen der Tabelle unten. Vier
> Commits (`4117a83` Zuweisung, `8472f9c` Argument, `ac1a7f8` Mitglied,
> `881431a` Rückgabe), 26 neue Tests, und vor jedem Schritt der Bestand
> gemessen: alle 365 `.dh`-Dateien im Repo durch `--check`, **keine einzige
> neue Meldung** außer drei echten Blindgängern in `49_pong_scene.dh`
> (`y = HEIGHT / 2 - PADDLE_H / 2` — läuft nur, weil 240 und 40 gerade sind).
> Beschrieben in `docs/sprache.md`, Abschnitt „Was der Übersetzer prüft".
>
> **Warnungen, keine Fehler** — und der Grund ist ein anderer als bei der
> Kommazahl-Warnung von `3e9211d`: dort ist die Regel selbst wertabhängig,
> hier ist sie eindeutig, aber die **Herleitung** des Ausdruckstyps ist neu.
> Ein Irrtum darin soll kein laufendes Programm unübersetzbar machen. Wenn
> sich das über ein paar Fassungen als still erweist, ist die Verschärfung zu
> `error` ein Einzeiler.

Die Typen sind da: `DIM x AS INTEGER`, Signaturen mit Typen, Klassen mit
Feldern. Zur Laufzeit werden sie hart durchgesetzt. Beim Übersetzen fiel
davon fast nichts auf.

Was `dhrt --check` **vorher fand**: fehlende und zu viele Argumente,
unbekannte Builtins, eine Kommazahl an eine ganzzahlige Variable (seit
`3e9211d`).

Was es **nicht fand** — jedes davon nachgemessen, jedes bricht erst zur
Laufzeit ab, und nur dann, wenn die Zeile auch wirklich ausgeführt wird:

| Quelltext | `--check` vorher | zur Laufzeit |
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

**Warum das der wichtigste Punkt war.** Bei einem Spiel läuft jeder Pfad
innerhalb von Minuten einmal. Bei Software läuft der Zweig „Rechnung stornieren
mit Teilzahlung" vielleicht das erste Mal beim Kunden. Ein Tippfehler in einem
Feldnamen ging durch Übersetzung, Start und Test hindurch und schlug dort zu.
Genau davor soll eine streng getypte Sprache schützen — das ist ihr ganzer
Handel: Strenge beim Schreiben gegen Ruhe im Betrieb.

**Die Maschinerie stand schon.** Der Compiler kannte die Typen der Globals, der
Locals, der Parameter und der Klassenfelder — er benutzte sie bereits für
spezialisierte Opcodes und für die Kommazahl-Warnung. Gebraucht wurde ein
Ausdruckstyp-Urteil (`statischer_typ`) an den vier Stellen Zuweisung,
Argumentbindung, Feld-/Methodenzugriff und Rückgabe. Konservativ bleiben, wo
der Typ nicht sicher feststeht — `None` heißt „weiß ich nicht", und darauf
stützt sich keine Meldung.

**Was beim Bauen die eigentliche Arbeit war, ist nicht das Finden, sondern das
Schweigen.** Drei Stellen hätten fast richtigen Code angestrichen:

* **Polymorphie.** `DIM t AS Tier : t = NEW Hund() : t.belle()` ist gültig,
  obwohl `Tier` kein `belle` hat. Deshalb zählen bei der Mitglieds-Prüfung
  auch alle Klassen mit, die von der angesagten *abstammen*.
* **Referenz-Typen.** `coerce()` reicht Klassen, MAP und ARRAY durch — eine
  Meldung „bricht ab" wäre dort unwahr, so unsinnig die Zuweisung auch sein
  mag.
* **`RETURN` in Zweigen.** Eine Pfad-Analyse („wird das Ende erreicht?") ist
  bei IF/SELECT/Schleifen schnell falsch beantwortet. Gemeldet wird nur, wo es
  gar kein `RETURN` gibt — dieser Fall ist ohne Zweifel falsch.

Dazu die drei bewussten Lücken: Rückgabetypen von Builtins (der Index ist
handgepflegt, ein veralteter Eintrag wäre hier ein falscher Alarm statt bloß
einer fehlenden Meldung), `PROPERTY` (dort entscheidet der Setter) und die
Element-Typen von Arrays (`coerce_array` baut ein frisches Zahlen-Literal noch
um, das ist statisch nicht sauber zu trennen).

## 2 — JSON kann nur gelesen, nicht geschrieben werden — ✅ ERLEDIGT 2026-08-23

> **Gebaut** (`72fbc26`): 15 Builtins — `JSON_NEW_OBJECT`/`NEW_ARRAY`,
> `JSON_SET_*` (STRING/INT/FLOAT/BOOL/NULL/JSON), `JSON_APPEND_*`,
> `JSON_REMOVE` und als Zugabe `JSON_KEYS` (ein Objekt ließ sich vorher gar
> nicht durchlaufen: `JSON_LEN` zählte die Schlüssel, herankommen konnte man
> an sie nicht). 26 Tests, Beispiel `examples/171_json_bauen.dh`, die sechs
> Regeln stehen in `docs/module-json.md`.
>
> Ein Handle ist dafür ein **Referenz-Typ** geworden (wie MAP/ARRAY/BUFFER) —
> ein geparstes Dokument ist damit genauso veränderbar wie ein gebautes, was
> den häufigsten Fall abdeckt: Antwort einlesen, ein Feld ergänzen,
> zurückschicken.
>
> Die unangenehmste Frage beim Bauen war nicht das Setzen, sondern
> **`"posten.0"` auf einem frischen Dokument**: Array oder Objekt mit dem
> Schlüssel `"0"`? Beides ist gültiges JSON, und die falsche Wahl fällt erst
> dem Empfänger auf. Statt zu raten nennt die Meldung den Weg zum Array.

`JSON_PARSE`/`LOAD`/`GET_*`/`HAS`/`LEN`/`TYPE`/`STRINGIFY`/`PRETTY` — das war
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

## 3 — Fremde Textkodierung — ✅ ERLEDIGT 2026-08-23

> **Gebaut** (`4ad6beb`): `READLINES`, `WRITEALL`, `APPENDFILE`, `CSV_LOAD`,
> `CSV_SAVE` und `OPENFILE` (von dort aus `READLINE`/`READALL$`/`WRITE`/
> `WRITELINE`, also auch der Weg für große Dateien) nehmen als letztes
> Argument `"utf8"` (Vorgabe), `"cp1252"` oder `"latin1"`. 52 Tests, davon 32
> die cp1252-Tabelle Byte für Byte gegen **Pythons** Codec — eine
> handgeschriebene Tabelle, die nur mit sich selbst übereinstimmt, wäre
> wertlos.
>
> Die Fehlermeldung nennt jetzt Datei, Zeile, das störende Byte und den
> Ausweg. **Lesen** kann in cp1252/latin1 nie fehlschlagen (die fünf
> unbelegten Bytes zeigen auf sich selbst, wie in jedem Browser);
> **Schreiben** eines Zeichens, das es dort nicht gibt, ist dagegen ein
> Fehler und kein stilles `?`.
>
> Offen bleibt **UTF-16** (Excels „Unicode Text"): BOM-Erkennung, zwei
> Byte-Reihenfolgen, Ersatzpaare — eine eigene Entscheidung.

Eine Datei, die nicht UTF-8 war, war unlesbar:

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

## 4 — Standardeingabe: Filterprogramme lassen sich nicht schreiben — ✅ ERLEDIGT 2026-08-23

> **Gebaut** (`2e8c28d`): `STDIN([kodierung$])` liefert die Standardeingabe als
> ganz normales **FILE-Handle** — `READLINE`, `READALL$`, `ENDOFFILE` und
> `READ_BYTES` gelten damit unverändert, die Kodierung aus Punkt 3 ebenfalls.
> Das ist besser als beide hier vorgeschlagenen Wege: auffindbarer als die
> magische Zeichenkette `"-"` und ohne die vier `STDIN_*`-Doppelgänger, die
> dieselbe EOF- und Kodierungslogik ein zweites Mal gebraucht hätten.
>
> **`INPUT` blieb unangetastet.** Der Vorwurf oben (es druckt seinen Prompt
> nach stdout) hat sich beim Nachsehen als *notwendig* erwiesen: die
> Editor-Konsole pipet stdin, und ein Test hält seit jeher fest, dass der
> Prompt trotzdem sichtbar sein muss. Ihn an ein Terminal zu koppeln hätte
> also genau die Konsole kaputtgemacht, für die er da ist. Aus dem Mangel
> wurde damit eine Arbeitsteilung: `INPUT` fragt einen Menschen, `STDIN()`
> verarbeitet einen Strom.
>
> 15 Tests, Beispiel `examples/172_filter.dh` (Dateien **oder**
> Standardeingabe, wie `wc` und `grep`).

WP A hat „Werkzeug in einer Kette" ausdrücklich zum Ziel erklärt. Der halbe
Weg fehlte: ein Programm kann lesen, was ihm als *Argument* gegeben wird,
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

## 5 — Bibliotheken teilen (Ökosystem) — ✅ ERLEDIGT 2026-08-23

> **Gebaut** (`84b8cc2`): gesucht wird jetzt (1) neben der importierenden
> Datei, (2) in jedem Ordner aus `DH_PATH`, (3) in
> `<Benutzerordner>/.drachenhauch/bibliothek`. Die eigene Kopie gewinnt
> immer; die Fehlermeldung nennt alle durchsuchten Orte. 13 Tests,
> beschrieben in `docs/sprache.md` („Wo gesucht wird").
>
> **Die Paketverwaltung bleibt bewusst aus** — genau wie unten vorgeschlagen.
> Ordner 3 ist der Platz, an dem sie später ablegen würde.
>
> **Ein Test hat den ersten Entwurf gerettet:** eine Datei namens `json` im
> Suchpfad verdeckte `IMPORT "json"` — in *jedem* Programm auf dem Rechner.
> Ein bekannter Modulname geht jetzt gar nicht erst in die Suche.

`IMPORT "x.dh"` löste **ausschließlich relativ zur importierenden Datei** auf
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

## 6 — Die Werkzeugkette um die Sprache — ✅ ERLEDIGT 2026-08-23

> **Alle vier gebaut** (`0fd814f`, `a0b3a93`), beschrieben in der neuen Seite
> `docs/werkzeuge.md`:
>
> * **`dhrt --version`** (und `--help`) — die Fassung *und* die eingebauten
>   Bestandteile, denn ein Bau ohne `--hardware` lässt Module weg, ohne dass
>   man es dem Binary ansieht.
> * **`dhrt test`** — `*_pruefung.dh` suchen, jede Datei als eigener Prozess,
>   Rückgabewert 0 nur wenn alles durchlief.
> * **`dhrt fmt`** — Schlüsselwörter groß (verlustfrei, die Vorgabe),
>   `--einruecken` auf Verlangen.
> * **`dhrun.py --doku`** — Referenz aus Signatur + Kommentarblock.
>
> **Der Formatierer hat unterwegs seine Voreinstellung getauscht**, und der
> Grund kam aus der Bestandsmessung: als Einrücker war er in 26 der
> Beispieldateien anderer Meinung als der Hausstil. Drei Runden später waren
> es null — aber die letzten Funde waren *keine* Fehler mehr, sondern von
> Hand gesetzte Gliederung, die die Sprache nicht kennt (eine eingerückte
> `RENDERTARGET_BEGIN`-Gruppe, ein ausgerichteter Kommentar). Daraus wurde
> die Regel: verlustfrei per Vorgabe, Einrücken nur auf ausdrückliche
> Anforderung.

`dhrt` konnte: `run`, `call`, `profile`, `debug`, `--check`, `--export`, dazu
die Entwickler-Einstiege `--tokens`/`--ast`/`--dumpbc`/`--preprocess`/`--runsrc`.

Was fehlte:

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

- ✅ **Dateisystem unvollständig:** `MKDIR` gibt es, ein Gegenstück zum Löschen
  eines Verzeichnisses nicht. Ebenso fehlen Zeitstempel (wann zuletzt geändert
  — die Grundlage jeder Sicherung und jedes „was ist neu"), rekursives
  Auflisten, Namensmuster (`*.csv`), ein Temp-Verzeichnis.
- 🔍 **Nur SQLite.** Für eine Anwendung in einem Betrieb liegen die Daten in
  PostgreSQL, MySQL oder MS SQL. Heute geht das nur über ein fremdes Werkzeug
  per `SHELL`.
- ✅ **Keine Konfigurations-/Austauschformate außer JSON und CSV:** kein INI,
  kein XML, kein YAML, kein TOML.
- ✅ **Kein PDF, kein Druck.** Rechnung, Lieferschein, Bericht, Etikett — für
  kaufmännische Software fast immer die erste Forderung nach „speichern".
  Heute endet der Weg beim Bildschirm oder bei einer Textdatei.
- ✅ **Kein XLSX**, obwohl CSV steht — für Auswertungen, die weitergegeben
  werden, ist das der erwartete Behälter.
- ✅ **Kein E-Mail-Versand** (SMTP). Ein Bericht, der sich selbst verschickt, ist
  bei Werkzeugen die Regel, nicht die Ausnahme.
- ✅ **Kein HTTP-Server.** Die Roadmap hat ihn gestrichen (*„wer wirklich einen
  Dienst braucht, stellt einen fertigen Server davor"*). Das würde ich vor dem
  Bastler-Leitbild neu bewerten: mit `mqtt`, `firmata`, `serial` und `net` an
  Bord fehlt für „meine Heizungssteuerung hat eine kleine Weboberfläche" genau
  ein Baustein, und es ist der kleinste von allen (`NET_TCP_LISTEN` steht schon
  darunter).
- 🔍/✅ **Kein Festkomma für Geld.** `0.1 + 0.2` ergibt `0.30000000000000004`. Mit
  `FORMAT$` sieht man das nicht mehr, aber summiert wird trotzdem falsch. Für
  eine Sprache, in der jemand eine Kasse schreiben soll, ist das eine
  Entscheidung wert (eigener Typ, oder die dokumentierte Regel „in Cent
  rechnen").


### Stand 7 (24.08.2026)

Sechs der acht Zeilen sind gebaut, jede mit Doku, Beispiel und Tests:

| | Modul / Ausbau | Doku |
|---|---|---|
| Dateisystem | `RMDIR`, Zeitstempel, rekursiv suchen, Namensmuster, Temp-Ordner | [builtins-core.md](builtins-core.md) |
| Webserver | `httpd` | [module-httpd.md](module-httpd.md) |
| Einstellungen | `ini` | [module-ini.md](module-ini.md) |
| Austauschformat | `xml` | [module-xml.md](module-xml.md) |
| Druck | `pdf` | [module-pdf.md](module-pdf.md) |
| Auswertung | `xlsx` | [module-xlsx.md](module-xlsx.md) |
| Versand | `smtp` | [module-smtp.md](module-smtp.md) |

YAML und TOML sind bewusst **nicht** dazugekommen: INI deckt den Fall
„Einstellungen, die ein Mensch bearbeitet" ab, JSON den Fall „Daten", und
beide zusammen lassen für YAML/TOML keinen Fall übrig, der die dritte
Schreibweise rechtfertigt.

Die zwei mit 🔍 markierten Zeilen sind keine Bibliotheksarbeit, sondern
Entscheidungen. Sie sind **untersucht, gemessen und entworfen**, aber nicht
gebaut:

* **[Entwurf: Geld](entwurf-geldtyp.md)** — mit dem Befund, dass der übliche
  Ratschlag („in Cent rechnen") selbst eine Falle ist: `INT(19.99 * 100)`
  ergibt **1998**. Empfehlung: erst Dokumentation und drei Helfer, ein
  Geldtyp allenfalls als Modul, nicht im Sprachkern. **Weg A ist am
  24.08.2026 gebaut** (`CENT`, `EURO$`, `ROUND_HALF_UP` plus der Abschnitt
  „Mit Geld rechnen"); über einen eigenen Typ ist damit nichts entschieden.
* **[Untersuchung: Datenbank-Treiber](entwurf-datenbanktreiber.md)** — mit
  gemessenen Zahlen (PostgreSQL: 61 Kisten, +1,09 MB, reines Rust, TLS aus
  dem, was schon da ist; MySQL: 90 Kisten, +2,23 MB, TLS nur mit
  C-Übersetzer). Empfehlung: PostgreSQL machbar und billig, aber erst bei
  konkretem Anlass; MySQL nicht.

## 8 — Sprachkomfort, bewusst zu entscheiden

Kein Punkt hier ist ein Mangel; alle drei sind Abwägungen, die man einmal
bewusst treffen und aufschreiben sollte.

- **Kein `INTERFACE`. Entschieden 2026-08-27: bleibt so.** `ABSTRACT`-Methoden
  decken den Zweck ab (eine Klasse kündigt an, die Erben füllen aus), und seit
  `x IS Typname` lässt sich zur Laufzeit auch danach fragen. Was ein Interface
  zusätzlich könnte, ist *mehrere* Verträge an einer Klasse — dafür bräuchte
  Drachenhauch Mehrfachvererbung oder ein zweites Typsystem daneben. Das ist
  keine Bequemlichkeit mehr, sondern eine Grundsatzentscheidung, und für einen
  BASIC-Dialekt die falsche.
- **Keine anonymen Funktionen, keine Closures, keine verschachtelten
  Funktionen.** `FUNCREF` verlangt eine benannte Funktion, und ihr Rumpf sieht
  nur Parameter und Globals. Das passt zu BASIC und bleibt so.
  **Teilweise erledigt (2026-08-26):** was daran wirklich weh tat, war nicht die
  fehlende Lambda, sondern dass objektorientierter Code sich nicht selbst als
  Rückruf eintragen konnte. `obj.methode` ist jetzt eine FUNCREF, die ihre
  Instanz mitträgt (`GUI_ON_CLICK(knopf, spieler.klick)`) — siehe CLAUDE.md,
  Abschnitt „Function References".
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
  weil es das Versprechen einlöst, mit dem die Sprache antritt. **Erledigt am
  23.08.2026** — der Bestand blieb dabei still, was den Verdacht wert war und
  gegengeprüft wurde: in das größte echte Programm (`buch-tippspiel`, 1922
  Zeilen) eingebaute Fehler werden alle gefunden.
- **2, 3, 4** sind zusammen „Daten rein und raus". Sie öffnen die Klasse
  Programme, die Daten von woanders holt, umformt und weitergibt — das ist der
  Großteil dessen, was Leute „Software" nennen. **2, 3 und 4 sind erledigt**
  (23.08.2026) — damit ist der ganze Block „Daten rein und raus" zu.
- **6** ist billig und wirkt nach außen: `--version`, ein Test-Läufer, ein
  Formatierer sind die Zeichen, an denen jemand erkennt, ob eine Sprache zum
  Arbeiten taugt. **Erledigt am 23.08.2026** — und billig war es auch, mit
  einer Ausnahme: der Formatierer kostete drei Runden gegen den Bestand, bis
  klar war, dass er per Vorgabe *weniger* tun muss.
- **5** ist die Voraussetzung dafür, dass andere etwas beitragen können —
  wichtig, aber wirkungslos, solange 1 bis 4 offen sind. **Erledigt am
  23.08.2026**, nachdem 1 bis 4 und 6 standen.
- **7** ist Fleißarbeit und lässt sich nach Bedarf abrufen. Der erste Griff
  daraus wäre PDF (kaufmännisch) oder der HTTP-Server (Bastler), je nachdem,
  wen man zuerst gewinnen will. **Erledigt am 23./24.08.2026** — und zwar
  beides, in dieser Reihenfolge: `httpd`, `ini`, `xml`, `pdf`, `xlsx`,
  `smtp`. Fleißarbeit war es tatsächlich, mit einer Lehre: **jedes dieser
  Formate musste ein FREMDER Leser gegenlesen** (PyMuPDF, openpyxl, Pythons
  `email`) — sonst hieße „die Datei ist in Ordnung" nur „mein Schreiber ist
  mit sich einig", und genau diese Gegenleser haben je einen echten Fehler
  gefunden.

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
