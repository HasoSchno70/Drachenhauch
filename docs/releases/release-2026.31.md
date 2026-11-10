# Drachenhauch 2026.31

*Die Notizen zu dieser Fassung. Drei Runden, in denen echte Programme
gebaut wurden: eine Vereinsverwaltung an drei Datenbanken, ein
Werkstatt-Dashboard über MQTT, eine Auswertung von Messdaten — und ein Spiel,
das im Browser läuft. **Einige Änderungen betreffen bestehende Programme**
(gleich zuerst).*

## Was sich für bestehende Programme ändert

* **`GUI_FORM_SAVE` auf eine gelöschte Zeile liefert 0.** Bisher kam die
  id zurück, als wäre gespeichert worden, und die Eingabe war still weg.
  `GUI_FORM_CONFLICT$` sagt dann `geloescht`.
* **`MQTT_UPDATE` wirft nicht mehr, wenn der Broker die Verbindung
  abbricht.** Wie beim net-Modul in 2026.30 ist das ein Zustand:
  `MQTT_IS_CONNECTED` wird FALSE, `MQTT_PUBLISH` und `MQTT_SUBSCRIBE`
  liefern jetzt einen Wahrheitswert. Ein Dashboard ohne `TRY` endete
  bisher beim Neustart des Brokers.
* **Befehle nehmen eine glatte Kommazahl als ganze Zahl** — mit derselben
  Regel wie eine Zuweisung an eine INTEGER-Variable. `/` liefert seit
  2026.16 immer eine Kommazahl, und `GUI_WINDOW("…", BREITE / 2 - 170, …)`
  brach mit „erwartet INTEGER“ ab, obwohl `x = BREITE / 2` ging. `2.5`
  bleibt ein Fehler, die Meldung nennt `INT`, `ROUND` und `\`.

  ```basic
  CONST BREITE = 800
  PRINT LEFT$("abcd", 6 / 3)      ' ab -- 2.0 gilt als 2
  ```
* **`CHART_DATA` mit weniger Punkten kürzt das Diagramm.** Bisher blieb die
  alte Punktzahl stehen, und nach „Woche, dann ein Tag“ hingen Tausende
  Nullen hinter dem Tag.
* **`ZEIT_FORMAT$` versteht die Schreibweise von C und Python**
  (`%Y-%m-%d %H:%M:%S`, `%a`, `%A`, `%%`). Bisher kam so ein Muster wörtlich
  zurück; ein `%`-Platzhalter, den es nicht gibt, ist jetzt ein Fehler.
* **`GENTEX_RADIAL` mit einer `dichte` außerhalb von 0 bis 1 ist ein
  Fehler.** Vorher entstand still eine harte Scheibe statt eines weichen
  Lichts.

## Neu in dieser Fassung

### Datenbank und Formulare

* **Schutz beim gleichzeitigen Bearbeiten:** `GUI_FORM_VERSION(win,
  spalte$)` — `GUI_FORM_LOAD` merkt sich den Stand, `GUI_FORM_SAVE`
  schreibt nur, wenn ihn niemand inzwischen geändert hat, und zählt ihn
  hoch. Sonst liefert es 0, und `GUI_FORM_CONFLICT$` sagt `geaendert` oder
  `geloescht`.
* **Geld in Cent im Formular:** `GUI_TEXTINPUT_SET(tf, "zahlen", 3)` — im
  Feld steht `48,50`, gebunden wird die ganze Zahl `4850`.
* **Klapplisten mit Werten:** `GUI_LISTBOX_SET_DATA` und Verwandte gelten
  auch für Klapplisten; eine gebundene Klappliste bindet den Wert (etwa den
  Steuersatz 19) statt der Nummer des Eintrags.
* MySQL zählt geänderte Zeilen jetzt wie SQLite und PostgreSQL, und `0,99`
  geht als Zahl in die Datenbank (die Regel für Postleitzahlen mit
  führender Null gilt nur noch, wenn auf die Null eine Ziffer folgt).
* **`GUI_CHANGED(wdg)`:** Hat die Bedienung das Widget im letzten
  `GUI_UPDATE` geändert? Was das Programm selbst setzt, zählt nicht.

### MQTT

* **`MQTT_WILL(topic$, inhalt$[, behalten])`**, der letzte Wille: reißt die
  Verbindung ab, ohne dass sich das Programm abmeldet, veröffentlicht der
  Broker diese Nachricht. Ein „online“ mit retain bleibt so nicht nach
  einem Stromausfall für immer stehen.
* Ein Broker, der verschwindet, ohne die Verbindung zu beenden, fällt
  jetzt nach der halben Keepalive-Zeit auf.

### Diagramme und Berichte

* **`CHART_X(c, xwerte)`** gibt den Punkten eines Liniendiagramms echte
  x-Werte, etwa Zeitstempel, und `CHART_SET_NUM(c, "luecke_ab", 60)`
  unterbricht die Linie, wo eine Messung fehlt. Bisher stand 13:00 direkt
  neben 16:00.
* **Einzeln beschriftete Punkte** (die Tagesanfänge unter 60 000 Messwerten)
  stehen jetzt alle da; bisher fielen sie zwischen die gezeichneten Stellen.
* **`CHART_SET_FLAG(c, "komma", TRUE)`:** Dezimalkomma in Diagrammen.
* **`RENDERTARGET_IMAGE(rt)`** liefert den Inhalt eines Render-Ziels als
  Bild, ohne auf `FLIP` zu warten, und **`PDF_IMAGE(p, bild, x, y, b[, h])`**
  setzt ein Bild ins PDF — so kommt ein Diagramm in den Bericht. Bisher
  kannte das pdf-Modul keine Bilder.

### Schrift

* **`LOADFONT("sans", 32)`** (auch `serif`, `mono`, je mit `-fett`,
  `-kursiv`, `-fett-kursiv`) nimmt die eingebaute DejaVu-Schrift — eine
  normale Schrift ohne Pfad, auf jedem System dieselbe, auch im Browser.
  Bisher ging das nur über einen festen Pfad wie
  `C:/Windows/Fonts/segoeui.ttf`, den es unter macOS und Linux nicht gibt.
* Das Euro-Zeichen neben der eingebauten Pixelschrift steht jetzt in der
  Höhe der Großbuchstaben.

### Im Browser

* **Was ein Programm schreibt, überlebt das Neuladen der Seite**
  (Arbeitsordner in der IndexedDB) — ein Spiel behält seine Bestenliste.
* **Ein kurzes Antippen und ein schneller Klick kommen an.** Bisher zählte
  im Browser nur ein langsamer Klick.
* Keine 26 Warnzeilen je Bild mehr in der Ausgabe (`GetKeyName`).

### Neue Beispiele

* **`examples/verein`** — Mitglieder, Beiträge, Zahlungen, Beitragsliste als
  PDF; dasselbe Programm an SQLite, PostgreSQL und MySQL.
* **`examples/werkstatt`** — ein Dashboard für Werkstattsensoren über MQTT,
  mit Alarm, Lüfter-Automatik und Aufzeichnung; `sensor.dh` spielt das Board.
* **`examples/messdaten`** — eine Woche Messwerte auswerten: Ausreißer,
  Lücken, Glättung, Tagesansicht, PDF-Bericht mit Diagrammen.
* **`examples/webspiel`** — „Glühwürmchen“, ein Antipp-Spiel, das auch im
  Web-Playground läuft.

### Kleineres

* Die Filterzeile der Tabelle, Häkchen- und Balkenzellen und der
  Zell-Editor waren in allen Themen schwarz.
* Lange Listeneinträge enden mit „…“, der ganze Text steht im Tooltip.
* Tastennamen kommen unter Windows aus der Belegung, ohne den gemeinsamen
  Tottasten-Zustand zu stören (`input_polish` „Á“ statt „A“).

## Unter der Haube

**Zahlen.** **2140** Befehle in 48 Modulen (vorher 2133), 225 Beispiele auf
der obersten Ebene von `examples/` (die vier neuen liegen in eigenen
Ordnern). **5079 Fälle in 400 Prüfsammlungen** (vorher 5035 in 394) und
**558 Rust-Testfunktionen**.

* Die eingebauten Schriften sind ein eigenes Feature `schriften`, das das
  pdf-Modul mitbringt; der Web-Bau schaltet es eigens ein (`dhrt.wasm`
  18,8 MB statt 13,7).
* **Die Spiel-Laufzeit für den schlanken Export bringt die Schriften mit**
  (27 statt 22 MB). Gefunden beim Bau dieser Fassung: ein Spiel mit
  `LOADFONT("sans")` bekam die Spiel-Laufzeit — den Befehl hat sie ja —
  und brach dann beim Start ab.
* Die Vereinsverwaltung läuft im Test gegen alle drei Datenbanken mit
  derselben erwarteten Ausgabe.

## Was offen bleibt

* Auf dem Handy öffnet die Leinwand keine Bildschirmtastatur.
* Zwei echte Rechner hinter Routern und Firewalls sind nicht ausprobiert.
* Ein Struct als Wert hinter `...`, ein `va_list` im Rückruf; C++ und COM.
* Das macOS-Paket ist nicht beglaubigt, der Installer nicht signiert.
* `ZEIT_PARSE` rechnet die Ortszeit nur unter Windows um; unter macOS und
  Linux gilt UTC.
