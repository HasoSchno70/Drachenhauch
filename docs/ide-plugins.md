# Plugins der IDE

Ein Plugin erweitert die IDE um eigene Befehle, eine eigene Leiste, Arbeit
bei Ereignissen (Sichern, Öffnen, Starten …) und Code-Hilfen (Prüfungen,
Schnipsel, Färbung weiterer Dateiarten). Es ist ein **eigenes
Drachenhauch-Programm**: die IDE startet es beim Start mit `WINDOW_OPEN` und
spricht über dessen Leitung mit ihm, eine JSON-Zeile je Nachricht. Ein
Plugin, das abstürzt oder hängt, reißt die IDE nicht mit -- sein Zustand
wird „abgestürzt“, seine Leisten gehen zu, seine Befehle werden gesperrt.
Wie bei VS Code laufen die Erweiterungen in einem eigenen Prozess.

## Wo Plugins liegen

Jedes Plugin ist ein Ordner mit einer `plugin.dh` darin, an einer von zwei
Stellen:

* **Eigene Plugins** unter `plugins/` neben der `ide.json` (Windows:
  `%APPDATA%\Drachenhauch\plugins\<name>\`). Sie laufen, bis man sie
  abschaltet (`plugins_aus` in der `ide.json`).
* **Mitgelieferte Plugins** in `examples/plugins/`, sie kommen mit dem
  Installer. Sie sind aus, bis man sie anhakt (`plugins_an`). Ein eigenes
  Plugin gleichen Namens verdeckt das mitgelieferte.

**Plugins → Neues Plugin …** legt ein eigenes aus der Vorlage an (ein
Befehl, ein Eintrag im Kontextmenü, eine Leiste), öffnet seine `plugin.dh`
und startet es. **Plugins → Plugins verwalten …** zeigt alle mit Titel,
Fassung und Zustand, der Tooltip nennt Beschreibung, Autor und Pfad; der
Haken sagt, ob es beim Start geladen wird. **Plugins → Plugins neu laden**
beendet alle und startet sie wieder -- nach einer Änderung an einem Plugin
genügt das, die IDE muss nicht neu starten. **Plugins → Plugin-Ordner
öffnen** öffnet den Ordner der eigenen.

Mitgeliefert sind drei:

* `todo`: TODO und FIXME der Datei vorn als Leiste rechts, ein Befehl mit
  Kürzel, eine Warnung je FIXME, ein Schnipsel und eine Färbung für
  `.todo`-Dateien.
* `klassen`: alle Klassen und Structs des Projekts als Baum (aus
  `CODE_CLASSES$`, ohne zu übersetzen), darunter Felder, Konstanten,
  Eigenschaften und Methoden mit Typ; ein Filterfeld, „Nur die Datei vorn“,
  und ein Klick springt an die Stelle. Es liest nach dem Sichern neu und
  lässt Ordner aus, die mit `.` oder `_` beginnen, dazu `pakete/` und
  `target/`.
* `schreibweise`: typische Schreibfehler in Kommentaren („Standart“,
  „seperat“) sind unterstrichen, Strg+. ersetzt sie, der Tooltip nennt die
  richtige Schreibweise, und das Kontextmenü hat „Schreibweise: Wort
  erlauben“ für Wörter, die so gemeint sind (`erlaubt.txt` im Datenordner).

### Der Steckbrief

Neben der `plugin.dh` darf eine `plugin.json` liegen:

```json
{"name": "Mein Plugin", "fassung": "1.0", "beschreibung": "Was es tut.", "autor": "Ich", "ide": "2026.36"}
```

Die Verwaltung zeigt Name und Fassung statt des Ordnernamens. `ide` ist die
kleinste Fassung der IDE, die das Plugin braucht -- mit einer älteren startet
es nicht, und sein Zustand sagt das. Ohne Steckbrief gilt der Ordnername.

Jedes Plugin hat einen **Datenordner** neben der `ide.json`
(`plugin-daten/<name>/`, `PLUGIN_DATENORDNER$()`). Dorthin gehören seine
Einstellungen und Listen -- ein mitgeliefertes Plugin liegt womöglich in
einem Ordner, in den es nicht schreiben darf.

## Ein Plugin schreiben

Die Bibliothek `dh_plugin.dh` liegt neben der IDE; die IDE legt ihren
Ordner in `DH_PATH`, ein Plugin schreibt also einfach `IMPORT
"dh_plugin.dh"`. Das Programm meldet an, was es kann, und wartet dann auf
Ereignisse:

```basic
IMPORT "dh_plugin.dh"
PLUGIN_BEFEHL("hallo", "Hallo einfügen", "Strg+Alt+Umschalt+H")
PLUGIN_LEISTE("notizen", "Notizen", "leiste.dhform", "rechts")
PLUGIN_HOERE("gesichert")
WHILE PLUGIN_LAEUFT()
    SELECT CASE PLUGIN_EREIGNIS$()
        CASE "befehl"
            IF PLUGIN_ID$() = "hallo" THEN PLUGIN_EINFUEGEN("' Hallo" + CHR$(10))
        CASE "klick"
            PLUGIN_MELDUNG("geklickt: " + PLUGIN_WIDGET$())
        CASE "ereignis"
            PLUGIN_AUSGABE("gesichert: " + PLUGIN_DATEI$())
        CASE "ende"
            EXIT(0)
    END SELECT
WEND
```

`PLUGIN_EREIGNIS$` wartet bis zu 50 ms und liefert "" ohne Ereignis -- die
Schleife braucht also kein eigenes `SLEEP`. Ohne IDE (das Plugin allein
gestartet) ist `PLUGIN_LAEUFT()` FALSE und das Programm endet gleich.

## Was ein Plugin kann

| Befehl | Was es tut |
|---|---|
| `PLUGIN_BEFEHL(id$, titel$ [, kuerzel$])` | einen Befehl anmelden: er steht im Menü **Plugins**, in der Befehlspalette und auf dem Kürzel. Ausgelöst kommt das Ereignis `befehl` mit `PLUGIN_ID$()` |
| `PLUGIN_LEISTE(id$, titel$, formdatei$ [, bereich$])` | eine eigene Leiste im Dock: eine `.dhform` (etwa aus dem Form-Designer) als Fenster links, rechts (Vorgabe), unten oder schwebend. Der Nutzer kann sie verschieben wie jede Leiste der IDE |
| `PLUGIN_HOERE(ereignis$)` | ein Ereignis hören: `gesichert`, `geoeffnet`, `reiter` (ein anderer Reiter vorn), `gestartet`, `beendet` (ein Programm lief), `geprueft` -- jeweils mit `PLUGIN_DATEI$()`. Dazu `hover`: beim Verweilen der Maus auf einem Wort, als eigenes Ereignis (siehe `PLUGIN_HOVER_ANTWORT`) |
| `PLUGIN_SETZE(leiste$, widget$, was$, wert$)` | ein Widget der eigenen Leiste ändern: `text`, `checked`, `value`, `enabled`, `visible` oder `selected` (Zeile einer Liste, Eintrag einer Klappliste, Knoten eines Baums) |
| `PLUGIN_LISTE(leiste$, widget$, eintraege)` | die Einträge einer Liste oder Klappliste der Leiste |
| `PLUGIN_BAUM(leiste$, widget$, ebenen, texte [, auf])` | einen Baum der Leiste füllen: je Eintrag eine Ebene (0 = oben) und ein Text, in der Reihenfolge von oben nach unten; ein Eintrag hängt am letzten davor mit kleinerer Ebene. Die Nummer eines Knotens ist seine Stelle in der Liste -- dieselbe Zahl kommt bei `geaendert` als Wert. `auf` (Vorgabe TRUE) klappt die Äste auf |
| `PLUGIN_TEXT$()` | der Text im vorderen Reiter |
| `PLUGIN_DATEINAME$()` | sein Pfad ("" = ungesichert) |
| `PLUGIN_AUSWAHL$()` | der markierte Text |
| `PLUGIN_MARKE$()` | Zeile und Spalte der Schreibmarke als `zeile,spalte` |
| `PLUGIN_PROJEKT$()` | der Projektordner |
| `PLUGIN_WORT$()` | das Wort an der Schreibmarke |
| `PLUGIN_EINFUEGEN(text$)` | Text an der Schreibmarke einfügen (eine Auswahl ersetzt er) |
| `PLUGIN_ERSETZEN(zeile, spalte, bis_zeile, bis_spalte, text$ [, datei$])` | einen Bereich ersetzen (Zeilen und Spalten ab 1, `bis_spalte` ausschließlich) -- im Reiter vorn oder in `datei$`, die die IDE dafür öffnet. Für Strg+Z ein Schritt |
| `PLUGIN_AUSWAHL_SETZEN(zeile, spalte [, bis_zeile, bis_spalte [, datei$]])` | die Schreibmarke setzen, mit Ende eine Auswahl |
| `PLUGIN_DATENORDNER$()` | der Ordner, in den das Plugin schreiben darf |
| `PLUGIN_MELDUNG(text$)` | eine Meldung in die Statusleiste |
| `PLUGIN_AUSGABE(text$)` | eine Zeile in die Ausgabe |
| `PLUGIN_OEFFNEN(pfad$ [, zeile])` | eine Datei öffnen, auf Wunsch in einer Zeile |
| `PLUGIN_BEFUND(datei$, zeile, meldung$ [, schwere$ [, spalte, laenge]])` | einen Befund sammeln (`fehler`, `warnung` oder `hinweis`). Er ist im Code unterstrichen wie eine Meldung der Laufzeit, mit Spalte (ab 1) und Länge genau diese Stelle, sonst die Zeile ohne Einrückung, und steht als Text am Zeilenende und im Tooltip |
| `PLUGIN_KORREKTUR(titel$, text$ [, zeile, spalte, bis_zeile, bis_spalte])` | eine Schnellkorrektur (Strg+.) zum zuletzt gesammelten Befund: `text$` ersetzt die Stelle des Befunds, mit eigener Angabe eine andere (Spalten ab 1, `bis_spalte` ausschließlich). Mehrere Aufrufe = mehrere Angebote |
| `PLUGIN_BEFUNDE_SENDEN(datei$)` | die gesammelten Befunde der Datei verschicken -- sie stehen in der Liste der Probleme, solange die Datei vorn ist, mit dem Namen des Plugins davor; ein neues Senden ersetzt die alten, leer gesendet nimmt es sie weg |
| `PLUGIN_SCHNIPSEL(name$, kuerzel$, text$)` | ein Schnipsel (Strg+J, oder das Kürzel + Tabulator); `\|` ist die Stelle der Schreibmarke |
| `PLUGIN_KONTEXT(id$, titel$)` | ein Eintrag im Kontextmenü des Code-Felds. Angeklickt kommt `befehl` mit dieser `id`, `PLUGIN_WERT$()` ist das Wort, auf das geklickt wurde (der Rechtsklick setzt die Schreibmarke dorthin) |
| `PLUGIN_HOVER_ANTWORT(text$)` | Antwort auf `hover` (nach `PLUGIN_HOERE("hover")`): der Text hängt sich an den Tooltip, solange die Maus noch an der Stelle steht; leer = nichts beizutragen. Zum Ereignis gehören `PLUGIN_WERT$()` (das Wort), `PLUGIN_ZEILE()`, `PLUGIN_SPALTE()` (ab 1) und `PLUGIN_DATEI$()`. Die IDE wartet nicht -- ein Plugin, das nicht antwortet, hält den Tooltip nicht auf |
| `PLUGIN_FAERBUNG(endung$, muster$, farbe)` | Färbung für eine weitere Dateiart (`".md"`): jede Regel ein regulärer Ausdruck und eine Farbe; solche Dateien färbt die IDE dann damit statt mit der Färbung für Drachenhauch |

Zum Ereignis gehören `PLUGIN_ID$()` (der Befehl bzw. der Name des
Ereignisses), `PLUGIN_LEISTE$()` und `PLUGIN_WIDGET$()` (bei `klick` und
`geaendert` -- der Name des Widgets aus der `.dhform`, Feld `name`),
`PLUGIN_WERT$()` (bei `geaendert`: der Text, der Haken als `TRUE`/`FALSE`,
der Wert eines Reglers, die Nummer der gewählten Zeile oder des gewählten
Baumknotens; bei `befehl` das Wort an der Schreibmarke) und
`PLUGIN_DATEI$()` (bei `ereignis` und `hover`). Die Fragen (`PLUGIN_TEXT$` und die
anderen) warten bis zu zwei Sekunden auf die Antwort; was dazwischen
ankommt, bekommt das nächste `PLUGIN_EREIGNIS$`.

## Die Leitung

Wer kein `dh_plugin.dh` benutzen will (oder ein Plugin in einer anderen
Sprache schreibt, die über `PARENT_SEND`/`PARENT_RECV$` hinaus nichts
braucht), schickt die Zeilen selbst. Jede ist ein JSON-Objekt mit `art`.

Vom Plugin an die IDE:

| `art` | Felder |
|---|---|
| `befehl` | `id`, `titel`, `kuerzel` |
| `kontext` | `id`, `titel` |
| `leiste` | `id`, `titel`, `bereich`, `form` (die `.dhform` als Objekt) |
| `hoere` | `ereignis` |
| `setze` | `leiste`, `widget` (Nummer in der Form), `was`, `wert` |
| `liste` | `leiste`, `widget`, `eintraege` (Feld) |
| `baum` | `leiste`, `widget`, `ebenen` und `texte` (Felder gleicher Länge), `auf` |
| `frage` | `nr`, `was` (`text`, `datei`, `auswahl`, `marke`, `wort`, `projekt`) |
| `tue` | `was` (`einfuegen`, `meldung`, `ausgabe`, `oeffnen`, `ersetzen`, `waehlen`), `text`, `datei`, `zeile`, dazu `spalte`, `bis_zeile`, `bis_spalte` |
| `befunde` | `datei`, `liste` (Objekte mit `zeile`, `meldung`, `schwere`, auf Wunsch `spalte`, `laenge` und `korrekturen` -- je `titel` und `aenderungen` mit `zeile`, `spalte`, `bis_zeile`, `bis_spalte`, `text`, wie bei `CODE_CHECK$`) |
| `hover` | `nr` (aus dem Ereignis), `text` |
| `schnipsel` | `name`, `kuerzel`, `text` |
| `faerbung` | `endung`, `muster`, `farbe` |

Von der IDE an das Plugin: `befehl` (`id`, `wert`), `klick` und `geaendert`
(`leiste`, `widget`, `wert`), `ereignis` (`name`, `datei`), `hover` (`nr`,
`wert`, `zeile`, `spalte`, `datei`), `antwort`
(`nr`, `wert`) und `ende`, kurz bevor die IDE die Leitung schließt.

Je Bild liest die IDE höchstens 50 Nachrichten eines Plugins -- ein
geschwätziges Plugin hält sie so nicht an. Eine unlesbare Zeile steht im
Protokoll, ein Fehler beim Ausführen (etwa ein unbekanntes Kürzel) dazu in
der Ausgabe mit dem Namen des Plugins davor.

## Plugins in Glutfaden

Glutfaden (`examples/glutfaden`, ab Stufe 26) lädt Plugins auf dieselbe
Weise: ein Ordner `plugins/<name>/plugin.dh` im Einstellungsordner von
Glutfaden (Logo-Menü → Plugins → Plugin-Ordner öffnen), gestartet mit
`WINDOW_OPEN`, dieselbe Leitung und dieselbe Bibliothek `dh_plugin.dh`.
Die Befehle stehen im Logo-Menü unter **Plugins**, ihre Kürzel gelten
überall im Fenster. Die Leiste baut Glutfaden aus GTK-Widgets nach (Lage
und Größe wie in der Form): Beschriftung, Knopf, Textfeld, Kästchen,
Regler, Liste und Klappliste; anderes steht als Hinweis da. Sie liegt in
den andockbaren Leisten und lässt sich ziehen wie Übersicht oder Muster.
Ereignisse sind `gesichert`, `geoeffnet` und `geaendert` (nach jeder
Änderung am Muster).

| Befehl | Was es tut |
|---|---|
| `PLUGIN_MUSTER$()` | das Muster als JSON: `name`, `breite`, `hoehe`, `stiche`, `farbe` (die gewählte) und `farben` (je `name`, `nummer`, `hersteller`, `rgb`, `stiche`); "" ohne Muster |
| `PLUGIN_STICHE(punkte, farben [, text$])` | Stiche auf die aktive Ebene: `punkte` = x, y, x, y ..., `farben` = je Punkt die Nummer der Farbe im Muster (-1 leert). Ein Schritt für Rückgängig, beschriftet mit `text$` (sonst dem Namen des Plugins) |

`PLUGIN_DATEINAME$()` ist der Pfad des Musters, `PLUGIN_PROJEKT$()` der
zuletzt benutzte Ordner, `PLUGIN_TEXT$()` dasselbe wie `PLUGIN_MUSTER$()`.
Was zu einem Code-Editor gehört (Befunde, Schnipsel, Färbung, Einfügen),
übergeht Glutfaden. Auf der Leitung ist `PLUGIN_STICHE` ein `tue` mit
`was` = `stiche` und den Feldern `punkte` und `farben`.
