# Drachenhauch 2026.37

*Die Notizen zu dieser Fassung. Im Mittelpunkt stehen **die Plugins der
IDE**: Sie lassen sich anlegen, holen, aktualisieren und ohne die IDE
prüfen. Die IDE bringt drei Plugins mit, jedes Plugin hat einen Steckbrief
und einen eigenen Datenordner, Plugins tragen zur Vervollständigung bei und
dürfen Text ändern. **Für bestehende Programme ändert sich nichts**; für
Plugins gibt es zwei Kleinigkeiten, sie stehen gleich zuerst.*

## Was sich für bestehende Plugins ändert

* **Die erste Nachricht der IDE an ein Plugin ist jetzt `hallo`** (mit der
  Fassung der IDE und des Protokolls). Wer `dh_plugin.dh` benutzt, merkt
  davon nichts -- die Bibliothek verarbeitet sie selbst. Ein Plugin, das die
  Leitung selbst liest, bekommt eine Zeile mit der Art `hallo` mehr.
* **Das mitgelieferte Plugin `schreibweise` merkt sich erlaubte Wörter im
  Datenordner** (`plugin-daten/schreibweise/erlaubt.txt` neben der
  `ide.json`) statt neben seinem Programm. Eine `erlaubt.txt` aus der
  vorigen Fassung liest es dort nicht mehr; wer sie behalten will,
  verschiebt sie.

## Neu in dieser Fassung

### Plugins finden, holen und aktualisieren

* **Drei Orte:** eigene Plugins unter `plugins/` neben der `ide.json`,
  **mitgelieferte** in `examples/plugins/` (aus, bis man sie in der
  Verwaltung anhakt) und **geholte** unter `plugins/pakete/`.
* **Holen …** in der Verwaltung (**Plugins → Plugins verwalten …**) nimmt
  eine Quelle wie `dhrt paket`: `github:nutzer/repo@stand`, eine
  https-Adresse auf ein ZIP, einen Ordner oder ein ZIP auf der Platte. Die
  Prüfsumme wird festgehalten; hat sich der Stand an der Quelle geändert,
  ist ein zweites Holen ein Fehler, und das alte Plugin bleibt.
  **Aktualisieren** nimmt einen neuen Stand an, **Entfernen** nimmt ein
  geholtes Plugin wieder weg.
* **Steckbrief:** Eine `plugin.json` neben der `plugin.dh` nennt Name,
  Fassung, Beschreibung, Autor und die kleinste IDE-Fassung, die das Plugin
  braucht. Mit einer älteren IDE startet es nicht, und sein Zustand sagt das.
* **Datenordner:** `PLUGIN_DATENORDNER$()` ist der Ordner, in den ein Plugin
  schreiben darf (`plugin-daten/<name>/` neben der `ide.json`).

### Plugins schreiben und prüfen

* **Plugins → Neues Plugin …** legt ein Plugin aus der Vorlage an (Befehl,
  Eintrag im Kontextmenü, Leiste), öffnet es und startet es -- samt einer
  `test.dhtest`, die es ohne die IDE prüft.
* **Die Schein-IDE `ide/dh_plugin_probe.dh`** startet ein Plugin wie die
  IDE, beantwortet seine Fragen aus einem Stand, den der Test setzt, und
  merkt sich alles, was es schickt (`PROBE_START`, `PROBE_SETZE_TEXT`,
  `PROBE_BEFEHL`, `PROBE_WIDGET$`, `PROBE_BEFUNDE$` …). So läuft eine
  Prüfsammlung für ein Plugin ohne Fenster.
* **Text ändern:** `PLUGIN_ERSETZEN` ersetzt einen Bereich im Reiter vorn
  oder in einer anderen Datei, `PLUGIN_AUSWAHL_SETZEN` setzt Marke und
  Auswahl -- beides ein Schritt für Strg+Z.
* **Vervollständigung:** Nach `PLUGIN_HOERE("vervollstaendigen")` trägt ein
  Plugin mit `PLUGIN_VORSCHLAEGE` Wörter zur Liste der Vervollständigung bei.
* **Fassung der IDE:** `PLUGIN_IDE_FASSUNG$()` und `PLUGIN_PROTOKOLL()` sagen
  einem Plugin, mit welcher IDE es redet, damit es auf Neueres verzichten
  kann.
* Handbuch: `docs/ide-plugins.md`, mit dem neuen Abschnitt „Ein Plugin
  prüfen“.

### Weitere Änderungen

* Aktualisieren und Entfernen eines laufenden geholten Plugins scheiterten
  unter Windows: Das Plugin hat seinen Ordner als Arbeitsordner, und ein
  Ordner in Gebrauch lässt sich nicht ersetzen. Die IDE hält geholte Plugins
  dafür kurz an und startet sie danach wieder.

## Unter der Haube

**Zahlen.** 2183 Befehle in 48 Modulen, 226 Beispiele auf der obersten Ebene
von `examples/`, **5484 Fälle in 446 Prüfsammlungen** (vorher 5473 in 444)
und 569 Rust-Testfunktionen.

* `dhrt pruef` kennt die Namen der Schein-IDE, damit das Handbuch sie wie
  Befehle beschreiben kann.
* Eine Sicherungskopie `cstruct.rs.echt` aus einer Gegenprobe lag seit
  2026.25 im Repository; sie ist entfernt.

## Was offen bleibt

* Holen über `github:` ist nicht gegen das echte GitHub ausprobiert, nur
  über HTTP an einem Gegenserver mit einem ZIP im selben Aufbau.
* Ein Plugin-Beitrag zum Tooltip, der erst ankommt, wenn die Maus
  weitergezogen ist, wird verworfen -- geprüft ist das nicht.
* **raylib 6.2** ist erschienen, die Rust-Bindung raylib-rs hat aber noch
  keine Fassung dafür. Drachenhauch bleibt bei raylib 6.0, bis raylib-rs
  nachzieht.
* Funkenflug, Glutfaden und der Fotobetrachter sind nur unter Windows
  ausprobiert (mit dem GTK von Inkscape).
* Auf dem Handy öffnet die Leinwand keine Bildschirmtastatur.
* Zwei echte Rechner hinter Routern und Firewalls sind nicht ausprobiert.
* Ein Struct als Wert hinter `...`, ein `va_list` im Rückruf; C++ und COM.
* Das macOS-Paket ist nicht beglaubigt, der Installer nicht signiert.
