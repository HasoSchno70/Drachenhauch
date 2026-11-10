# Drachenhauch 2026.19

*Die Notizen zu dieser Fassung. Zwei Tage und 27 Pull Requests nach
2026.18, und diesmal kaum Neues, dafür viel Härte: die IDE wurde benutzt,
als wollte man mit ihr arbeiten (ein Haushaltsbuch, ein Schlangen-Spiel),
und dann in zwölf Runden absichtlich überladen -- 100 000 Zeilen, Zeilen
mit einer Million Zeichen, 20 000 Dateien, 900 Aufrufe tief im Debugger.
Was dabei hängen blieb oder abstürzte, ist behoben, jedes mit einem
Prüffall, der am alten Stand fällt.*

## Neu in dieser Fassung

### Aus der Benutzung (Dogfooding)

* **Klicks und Tastendrücke zwischen zwei Bildern gehen nicht mehr
  verloren.** raylib liest Tasten als Zustand; kamen Drücken und Loslassen
  vor dem nächsten Bild an, sah niemand den Druck -- ein Tippen aufs
  Touchpad bei 60 Bildern je Sekunde ging oft verloren. Gilt für jedes
  Programm, auch für `MOUSE_HIT`/`KEYHIT`.
* **Tastenkürzel nach der Tastaturbelegung:** auf QWERTZ war Strg+Z bisher
  Wiederholen (raylib benennt Tasten nach der US-Lage).
* **Umlaute bleiben in der eingebauten Schrift:** „Löschen“ stand bisher in
  einer anderen Schrift als „Buchen“ daneben.
* **F12 legt kein Bildschirmfoto mehr ab** (raylibs eingebaute Aufnahme ist
  aus; `SAVESCREENSHOT` bleibt). In der IDE ist F12 „Zur Definition“.
* Die **Vorschlagsliste** steht beim Tippen sichtbar an der Schreibmarke und
  geht nur beim Tippen auf, nicht beim Setzen der Marke.
* Ein **Klick in die Gliederung** springt, ohne zu markieren; Pfeile laufen
  im Code mit.
* **Schnellkorrektur** legt eine fehlende Variable auf Wunsch auch global an
  und schreibt den Namen so, wie er im Programm steht.
* **Debugger:** das Programmfenster kommt bei Weiter nach vorn, Variablen
  stehen in ihrer Schreibweise, Handles mit ihrem Typ; Haltepunkte in
  fremden Dateien sind kein Befund mehr.
* **Form-Designer mit Zoom** (50 bis 300 %) und einheitlich deutscher
  Palette; die **Umbau-Vorschau** ist so breit wie der Bildschirm.
* Im Startmenü startet jede Verknüpfung wieder das, was draufsteht (eigene
  App-Kennung je Verknüpfung).
* Neues Beispiel **Haushaltsbuch** (`examples/haushaltsbuch/`): Einnahmen
  und Ausgaben in SQLite, in ganzen Cent gerechnet.

### Aus dem Stresstest

| Fall | vorher | jetzt |
|---|---|---|
| Programm gibt 200 000 Zeilen aus | IDE hängt 19–29 s | 1,8 s für alles |
| Zeile mit 270 000 Zeichen öffnen | über 5 Minuten | normal |
| Klick weit rechts in einer Zeile mit 25 000 Zeichen | 59 s | unter 0,1 s |
| 120 000 Zeilen in eine Baumtabelle | 104 s | 57 ms |
| Profil eines Programms mit 20 000 Zeilen anzeigen | 13,7 s | 0,4 s |
| Prüfen einer Datei mit 5000 Warnungen | 7,3 s | 0,2 s |
| 100 000 Zeilen einfügen, dann prüfen | 2,7 s | 0,9 s |
| Debugger-Halt 900 Aufrufe tief | 150 ms | 18 ms |
| Datei-Wähler, 20 000 Dateien, je Tastendruck | 0,3 s | 0,07 s |

Dazu, was vorher die IDE samt ungesicherter Arbeit beendete:

* **Plattenzugriffe:** eine Datei, die sich nicht sichern lässt, eine Datei
  in cp1252, ein schreibgeschützter Ordner -- jetzt eine Meldung statt eines
  Abbruchs. Dateien in cp1252/latin1 werden gelesen.
* **Einstellungsdateien** mit Werten vom falschen Typ (`ide.json`,
  `farben.json`, `tasten.json`) brachen jeden Start ab.
* **Riesige Zeilennummern** in „Gehe zu Zeile“ oder in einer Ausgabezeile.
* **Wege in anderer Schreibweise** (kurzer Name, `..`) öffneten dieselbe
  Datei ein zweites Mal, und ein Umbau verfehlte den offenen Reiter.

Und was falsch war, ohne abzustürzen:

* Ein **Vergleich** (mit der gesicherten Fassung, zweier Reiter, die
  Umbau-Vorschau) zeigte in Dateien über 1500 Zeilen nach einer einzigen
  eingefügten Zeile alles darunter als geändert. Jetzt nur, was sich
  geändert hat.
* Der **Git-Unterschied** zeigte eine 20-MB-Protokolldatei ganz an (und die
  erste Datei steht beim Öffnen gewählt); große neue Dateien stehen jetzt
  mit ihrer Größe da, binäre als binär, lange Unterschiede gekürzt.
* Der **Rückgängig-Verlauf** hielt 100 volle Textstände -- bei einer
  30-MB-Datei drei Gigabyte. Jetzt höchstens 64 MB.
* Mit **zwölf offenen Reitern** meldete ein Sprung in eine schon offene
  Datei „Höchstens 12 Reiter“.
* **Strg+Alt+F5** sicherte nur die Prüfsammlung, nicht den geprüften Code.
* Das **Terminal** zeigte nach einer sehr langen Ausgabe 2999 statt 3000
  Zeilen (gefunden im Prüflauf zu dieser Fassung).

### Neue Befehle

`GUI_WINDOW_FRONT` (ein Fenster nach vorn, ohne ihm den Fokus zu geben),
`GUI_TEXTAREA_CARET_XY` (wo die Schreibmarke auf dem Bildschirm steht),
`GUI_WINDOW_ZOOM`/`GUI_WINDOW_GET_ZOOM` (Entwurfsfenster vergrößern) und
`PROCESS_FRONT` (das Fenster eines gestarteten Programms nach vorn holen).

## Unter der Haube

**Zahlen.** Die Befehlsreferenz wächst von 2024 auf **2029**
Einträge, es bleiben 48 Module und 216 Beispiele.
**4673 Fälle in 356 Prüfsammlungen** (vorher 4571 in
340) und **476 Rust-Testfunktionen** (vorher 458).

* Die gui merkt sich die Zerlegung eines Textbereichs, solange sein Text
  derselbe ist: `GUI_TEXTAREA_CURSOR`, `_VIEW` und `_POS_AT` fragen bei
  100 000 Zeilen 0,05 statt 2,4 ms.
* Tabellen nehmen angehängte Zeilen direkt in die Ansicht, statt sie bei
  jeder Zeile neu aufzubauen; eine Liste zeichnet von langen Einträgen nur,
  was sichtbar sein kann.
* Die Schnellkorrekturen des Sprachservers rechnen den ganzen Text einmal
  je Prüfung statt je Meldung.
* `CODE_TYPES$` übersetzt nicht, wenn kein `FOR EACH` im Text steht.

## Was offen bleibt

* Mit Fokus kostet das Code-Feld bei 100 000 Zeilen je Bild noch rund
  17 ms (die Bearbeitung kopiert und zerlegt in jedem Bild den ganzen Text).
* Auf einem echten Mac ist das Paket weiter nicht ausprobiert und nicht
  beglaubigt.
* Der Installer ist nicht signiert; SmartScreen meldet sich beim ersten Start.
