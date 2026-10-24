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

Jedes Plugin ist ein Ordner unter `plugins/` neben der `ide.json`
(Windows: `%APPDATA%\Drachenhauch\plugins\<name>\`) mit einer `plugin.dh`
darin. **Plugins → Plugin-Ordner öffnen** legt den Ordner an und öffnet ihn.
**Plugins → Plugins verwalten …** zeigt alle mit ihrem Zustand; der Haken
sagt, ob es beim Start geladen wird (`plugins_aus` in der `ide.json`).
**Plugins → Plugins neu laden** beendet alle und startet sie wieder -- nach
einer Änderung an einem Plugin genügt das, die IDE muss nicht neu starten.

Ein Beispiel liegt in `examples/plugins/todo/`: TODO und FIXME der Datei
vorn als Leiste rechts, ein Befehl mit Kürzel, eine Warnung je FIXME, ein
Schnipsel und eine Färbung für `.todo`-Dateien. Zum Ausprobieren den Ordner
`todo` in den Plugin-Ordner kopieren und neu laden.

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
| `PLUGIN_HOERE(ereignis$)` | ein Ereignis hören: `gesichert`, `geoeffnet`, `reiter` (ein anderer Reiter vorn), `gestartet`, `beendet` (ein Programm lief), `geprueft` -- jeweils mit `PLUGIN_DATEI$()` |
| `PLUGIN_SETZE(leiste$, widget$, was$, wert$)` | ein Widget der eigenen Leiste ändern: `text`, `checked`, `value`, `enabled`, `visible` oder `selected` |
| `PLUGIN_LISTE(leiste$, widget$, eintraege)` | die Einträge einer Liste oder Klappliste der Leiste |
| `PLUGIN_TEXT$()` | der Text im vorderen Reiter |
| `PLUGIN_DATEINAME$()` | sein Pfad ("" = ungesichert) |
| `PLUGIN_AUSWAHL$()` | der markierte Text |
| `PLUGIN_MARKE$()` | Zeile und Spalte der Schreibmarke als `zeile,spalte` |
| `PLUGIN_PROJEKT$()` | der Projektordner |
| `PLUGIN_EINFUEGEN(text$)` | Text an der Schreibmarke einfügen (eine Auswahl ersetzt er) |
| `PLUGIN_MELDUNG(text$)` | eine Meldung in die Statusleiste |
| `PLUGIN_AUSGABE(text$)` | eine Zeile in die Ausgabe |
| `PLUGIN_OEFFNEN(pfad$ [, zeile])` | eine Datei öffnen, auf Wunsch in einer Zeile |
| `PLUGIN_BEFUND(datei$, zeile, meldung$ [, schwere$])` | einen Befund sammeln (`fehler`, `warnung` oder `hinweis`) |
| `PLUGIN_BEFUNDE_SENDEN(datei$)` | die gesammelten Befunde der Datei verschicken -- sie stehen in der Liste der Probleme, solange die Datei vorn ist, mit dem Namen des Plugins davor; ein neues Senden ersetzt die alten, leer gesendet nimmt es sie weg |
| `PLUGIN_SCHNIPSEL(name$, kuerzel$, text$)` | ein Schnipsel (Strg+J, oder das Kürzel + Tabulator); `\|` ist die Stelle der Schreibmarke |
| `PLUGIN_FAERBUNG(endung$, muster$, farbe)` | Färbung für eine weitere Dateiart (`".md"`): jede Regel ein regulärer Ausdruck und eine Farbe; solche Dateien färbt die IDE dann damit statt mit der Färbung für Drachenhauch |

Zum Ereignis gehören `PLUGIN_ID$()` (der Befehl bzw. der Name des
Ereignisses), `PLUGIN_LEISTE$()` und `PLUGIN_WIDGET$()` (bei `klick` und
`geaendert` -- der Name des Widgets aus der `.dhform`, Feld `name`),
`PLUGIN_WERT$()` (bei `geaendert`: der Text, der Haken als `TRUE`/`FALSE`,
der Wert eines Reglers, die Nummer der gewählten Zeile) und
`PLUGIN_DATEI$()` (bei `ereignis`). Die Fragen (`PLUGIN_TEXT$` und die
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
| `leiste` | `id`, `titel`, `bereich`, `form` (die `.dhform` als Objekt) |
| `hoere` | `ereignis` |
| `setze` | `leiste`, `widget` (Nummer in der Form), `was`, `wert` |
| `liste` | `leiste`, `widget`, `eintraege` (Feld) |
| `frage` | `nr`, `was` (`text`, `datei`, `auswahl`, `marke`, `projekt`) |
| `tue` | `was` (`einfuegen`, `meldung`, `ausgabe`, `oeffnen`), `text`, `datei`, `zeile` |
| `befunde` | `datei`, `liste` (Objekte mit `zeile`, `meldung`, `schwere`) |
| `schnipsel` | `name`, `kuerzel`, `text` |
| `faerbung` | `endung`, `muster`, `farbe` |

Von der IDE an das Plugin: `befehl` (`id`), `klick` und `geaendert`
(`leiste`, `widget`, `wert`), `ereignis` (`name`, `datei`), `antwort`
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
