# Kühlschrank-Verwaltung

Vorrat, Lagerorte, Einkauf, Rezepte und Statistik für die Küche -- ein
Projekt für das Drachenhauch Studio. Es ist ein Port einer PySide6-App
(„Kühlschrank“) und liest deren Datenbank `fridge_database.db` mit
demselben Aufbau.

Im Studio: Datei → Öffnen, diesen Ordner wählen, F5. Ohne Studio, nachdem
es einmal erzeugt hat:

```
dhrt run _studio/main.dh
```

Projekt → Bauen legt ein eigenständiges Programm nach `bau/`.

* **Bestand:** Tabelle mit Suche und Filtern (abgelaufen, bald, frisch,
  unter Minimum), ±1 je Zeile, Kontextmenü (bearbeiten, Mindestmenge,
  verbraucht, abgelaufen, löschen), Rückgängig und Wiederholen (Strg+Z,
  Strg+Y), Export als CSV. Eine Zeile auf einen Lagerort links ziehen
  verlegt sie.
* **Lagerorte:** Kacheln mit Anzahl und „läuft bald ab“, ein Klick zeigt,
  was dort liegt.
* **Einkauf:** Liste mit Haken, Vorschläge beim Tippen, Schnellknöpfe für
  das, was am häufigsten verbraucht wird, Auto-Ergänzen aus den
  Mindestmengen, Sortieren nach den Gängen eines Markts, Drucken als Bericht,
  „Einkauf abschließen“ übernimmt die Zeilen in den Bestand.
* **Rezepte:** 29 Rezepte zum Start, „Was kann ich kochen?“, „Ablaufende
  verwerten“, Kochen mit Verbrauch nach dem ältesten Ablaufdatum zuerst,
  eigene Rezepte; mit einem Schlüssel auch Rezepte von Spoonacular.
* **Statistik:** Kacheln, Diagramme nach Kategorie, Lagerort und Ablauf,
  Hinweise.
* **Scannen:** ein Barcode fragt Open Food Facts (im Hintergrund, mit
  Zwischenspeicher); der Stapelscan nimmt viele Produkte auf einmal auf,
  mit gelernten Vorgaben für Lagerort und Haltbarkeit.
* **Einstellungen:** Design, Sprache, Benachrichtigungen (abgelaufen,
  heute, morgen, bald, unter Minimum, tägliche Zusammenfassung), Lagerorte
  mit Sinnbild und Farbe, Supermärkte mit Gangplan, Sicherung und
  Wiederherstellen der Datenbank.

## Sprache

Die Oberfläche gibt es deutsch und englisch: Einstellungen → Allgemein →
Sprache, ab dem nächsten Start. Ohne Einstellung ist sie englisch, wenn
`DHRT_LANG` englisch ist. Die englischen Texte stehen in `sprache/en.txt`
(je Zeile deutsch, Tabulator, englisch). Was in der Datenbank steht --
Lagerorte, Kategorien, Einheiten, die mitgelieferten Rezepte -- bleibt, wie
es dort steht.

## Wo die Daten liegen

Neben dem Programm (beim Start aus dem Studio in `_studio/`):
`fridge_database.db`, `barcode_cache.json` (Open Food Facts),
`uebersetzungen.json` (übersetzte Rezepte) und `backups/` für die
Sicherungen. Eine neue Datenbank bekommt die Startdaten: Lagerorte,
Kategorien, Einheiten, Märkte mit Gangplänen und die 29 Rezepte.

## Netz

* **Open Food Facts** braucht keinen Schlüssel. Die App nennt sich mit
  eigenem Namen und merkt sich jede Antwort 30 Tage.
* **Spoonacular** nur mit eigenem Schlüssel: `SPOONACULAR_API_KEY` oder im
  Suchfenster eintragen (er geht als Kopfzeile hinaus, nie in der Adresse).
* **MyMemory** übersetzt gefundene Rezepte ins Deutsche, wenn das Häkchen
  gesetzt ist.

## Für Prüfungen

`KUEHL_OFF_URL`, `KUEHL_SPOON_URL` und `KUEHL_UE_URL` ersetzen die
Adressen der Dienste, `KUEHL_HEUTE` und `KUEHL_JETZT` die Uhr,
`KUEHL_MELDUNGEN` schreibt Benachrichtigungen in eine Datei,
`KUEHL_EXPORT`, `KUEHL_EXPORT_ORDNER`, `KUEHL_SICHERUNG` und `KUEHL_ZETTEL`
ersetzen die Dateidialoge. Geprüft in `tests/pruef/kuehlschrank/` (je Stufe
eine Sammlung) -- mit eigenen Servern auf 127.0.0.1, nie gegen die echten
Dienste. Wie der Port entstand und worin er von der Python-App abweicht,
steht in `geschichte/kuehlschrank.md`.
