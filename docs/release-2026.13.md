# Drachenhauch 2026.13

*Die Notizen zu dieser Fassung. Ein Tag und sieben Commits nach 2026.12, und
alle sieben gehören zu den letzten beiden Punkten der Lückenliste nach dem
sechsten Piloten: **Barrierefreiheit** und **fremde Schriften**. Beide begannen
als Untersuchung mit einer Messung, und beide Messungen ergaben null.*

## Neu in dieser Fassung

### Ein Bildschirmleser sieht das Fenster

Gemessen vor dem Bau: ein laufendes Drachenhauch-Fenster hatte im
Barrierefreiheits-Baum von Windows **null Nachkommen**. Für einen
Bildschirmleser, die Bildschirmlupe und die Sprachsteuerung war jedes
Programm ein Titel und sonst nichts — raylib malt Pixel, und GLFW meldet dem
System keine Bedienelemente. Die Rechnungsverwaltung aus 2026.12 lief, sah
gut aus und ließ sich mit der Tastatur bedienen, und ein blinder Nutzer
konnte sie nicht öffnen.

Jetzt **meldet das `gui`-Modul seinen Baum von selbst**, über die
Rust-Bibliothek AccessKit: jedes Fenster, jedes Widget mit Rolle,
Beschriftung, Wert, Lage und Zustand, die Menüleiste samt Einträgen und
Kürzeln, die Reiter, Listeneinträge, Tabellenzeilen und Baumknoten. Ein
Eingabefeld heißt wie die Beschriftung links daneben oder darüber. Und die
Gegenrichtung geht auch: klickt ein Hilfsprogramm einen Knopf, feuern
`GUI_CLICKED` und `GUI_ON_CLICK` wie bei einem Mausklick; setzt es Fokus oder
Text, sind es dieselben Wege wie bei der Tastatur. **Ein Programm muss dafür
nichts tun**, und ein Spiel ohne Bildschirmleser zahlt nichts — der Baum
wird nur gebaut, wenn jemand danach fragt.

Derselbe Leser, der vorher null fand, zählt im gui-Beispiel jetzt acht
Knoten, mit Namen:

```text
Group    [Einstellungen]
Text     [Lautstaerke: 50%]
Slider   [Lautstaerke: 50%]     der Name kommt von der Beschriftung daneben
CheckBox [Sound an]
Edit     []
Button   [Start]
```

Im Test ist dieser Leser ein fremder: UI Automation aus PowerShell heraus,
ohne neue Abhängigkeit. Er klickt den Knopf, setzt Text und Fokus, und das
Programm sieht es über `GUI_CLICKED`, `GUI_TEXT` und `GUI_FOCUSED`. Das läuft
auch auf dem Windows-Läufer der CI.

Was das Programm beisteuern kann: `GUI_ANNOUNCE(text$)` gibt dem
Bildschirmleser des Nutzers einen Satz mit — er spricht ihn mit seiner
Stimme, auf seiner Braillezeile, ohne eigene Sprachausgabe. Die
Rechnungsverwaltung sagt so ihre Statuszeile an. `GUI_SCREENREADER()` sagt,
ob jemand zuhört.

**Drei Fallen beim Bau**, weil sie wiederkommen werden: der Windows-Adapter
verlangt ein Fenster, das noch nie sichtbar war — raylib legt es jetzt immer
versteckt an und zeigt es danach. Mit zwei Fassungen des `windows`-Crates
(eine fürs Drucken, eine aus AccessKit) brach der Link an doppelten Symbolen
für `CloseWindow` und `ShowCursor`, weil raylibs eigene Funktionen so heißen
wie Exporte von user32 — eine Fassung, und es linkt. Und die Aktionen des
Hilfsprogramms müssen *nach* dem Bildlauf kommen, weil der zuerst die
Klick-Flags löscht; ein davor gesetzter Klick war im selben Bild schon
wieder weg.

**macOS und Linux** haben ihre Adapter in derselben Bibliothek; sie sind
eingehängt und werden in der CI auf den echten Systemen übersetzt — gelaufen
ist dort noch nichts, weil es hier weder Mac noch Linux gibt. Wer VoiceOver
oder Orca hat, ist die Abnahme; die Doku sagt das so.

### Bedienung ohne Maus, zweiter Teil

Aus derselben Untersuchung, ohne Bibliothek: die **Menüleiste geht jetzt per
Tastatur** — F10 oder Alt allein öffnet sie, Pfeile laufen, Enter wählt, ESC
schließt eine Ebene; ein Alt+X bleibt ein Kürzel. Die **Tab-Reihenfolge**
lässt sich setzen (`GUI_SET_TAB_INDEX`), ein per Tastatur fokussiertes
Widget zeigt nach kurzem Verweilen seinen Tooltip, und das helle Thema hat
seinen gedämpften Text von 2,76:1 auf 6,3:1 gehoben — der lag unter dem
WCAG-Wert, das Kontrast-Thema mit 21:1 gab es längst, nur nannte es niemand.

### Das Euro-Zeichen war ein Fragezeichen

Die zweite Messung: `aä€日本😀` per Zwischenablage in ein Textfeld, `LEN`
sagt 6, `GUI_TEXT` gibt es unverändert zurück — und gezeichnet wird
`aä????`. Der Speicher war Unicode, die Anzeige Latin-1. Jede Schrift, auch
eine per `LOADFONT` geladene japanische, wurde mit einem festen Vorrat von
246 Zeichen gebacken, und **das `€` war nicht dabei**. Jedes Programm mit
Euro-Preisen zeigte Fragezeichen; ein griechisches oder russisches Programm
war unmöglich.

Jetzt gilt:

* Der **Grundvorrat** jeder Schrift reicht von ASCII bis Kyrillisch, mit
  Latin Extended (`ő ł č`), Griechisch, Interpunktion und `€`. Der Start
  kostet dafür 50 ms mehr.
* **Glyphen auf Zuruf:** steht ein Zeichen in keiner geladenen Schrift
  (Kanji, Hangul, Emoji, Arabisch, Hebräisch), backt der nächste `FLIP` es
  aus der passenden Systemschrift nach — nur die gebrauchten Zeichen, nicht
  der ganze Block. Zeichnen und Messen zerlegen den Text in Läufe je Schrift,
  auch in einer selbst geladenen. Gemessen: das erste Bild mit Kanji, Hangul,
  Emoji und Hebräisch zugleich kostet einmalig etwa 100 ms, jedes weitere
  neue Zeichen etwa 15 ms, danach nichts.
* `LOADFONT(pfad$, groesse[, zeichen$])` nimmt Blocknamen (`"kyrillisch,
  griechisch"`, `"japanisch"`, `"emoji"`) oder die Zeichen selbst.
* **Schriftsammlungen (`.ttc`)** gehen. Das war der versteckte Fund: raylib
  kann sie nicht lesen und tauschte die Schrift bisher **still** gegen seine
  Bitmapschrift — `LOADFONT` gab ein Handle, der Text kam in der falschen
  Schrift ohne Meldung. Auf Windows liegen alle CJK-Schriften nur so vor.
  Die erste Schrift der Sammlung wird herausgelöst; eine Ersatzschrift ist
  jetzt ein Fehler.
* Die Tipp-Warteschlange fasst 256 Zeichen je Bild statt 16 — eine
  bestätigte Eingabe aus einer Eingabemethode hätte den Rest still verloren.

### Eingabemethoden

Wer Japanisch, Chinesisch oder Koreanisch schreibt, tippt Silben, und eine
Eingabemethode wandelt sie um. Diese Umwandlung steht jetzt **im Feld**:
unterstrichen an der Schreibmarke des Textfelds oder Textbereichs, die
Kandidatenliste des Systems daneben, das Ergebnis wird durch dieselben
Grenzen wie getippter Text eingefügt. Ohne Textfeld mit Fokus — ein Spiel
mit `INKEY$` — läuft die Eingabemethode wie bisher, damit dort weiter
Zeichen ankommen. Windows; nach der Windows-Dokumentation gebaut und **ohne
installierte Eingabemethode nicht nachgemessen** — auf der
Entwicklungsmaschine gibt es nur die deutsche Tastatur. Wer eine hat, ist
die Abnahme.

## Unter der Haube

**Die Rust-Tests liefen nirgends.** Die CI rief nur `cargo check`, und lokal
scheiterte `cargo test` an raylibs Pfadlänge. Mit einem kurzen Zielordner
laufen jetzt alle 310 Tests mit allen Features durch — dabei fielen zwei
kaputte auf, ein falsch gesetztes Testattribut und zwei veraltete Aufrufe,
beide behoben. Die CI hat seither einen Testschritt ohne Grafik und einen
Prüfschritt, der die Barrierefreiheits-Adapter auf jedem Läufer für sein
System übersetzt.

**Vier Untersuchungen, vier Entscheidungen.** Die Lückenliste nach dem
sechsten Piloten nannte vier Architekturpunkte: mehrere Fenster, Drucken,
Barrierefreiheit, Eingabemethoden. Jeder bekam erst ein Papier mit
Messungen und Wegen (`docs/entwurf-*.md`), dann eine Entscheidung, dann den
Bau. Mit dieser Fassung ist die Liste abgearbeitet.

**Zahlen.** Die Befehlsreferenz von 1712 auf **1715** Einträge
(`GUI_ANNOUNCE`, `GUI_SCREENREADER`, `GUI_SET_TAB_INDEX`), die Modulliste
bleibt bei 47, die Beispiele bei 212. Die Testsuite von 4428 auf **4441**
gesammelte Prüfungen, dazu 310 Rust-Tests, die jetzt auch laufen.

## Was offen bleibt

* Drei Dinge warten auf eine Abnahme, die diese Maschine nicht leisten kann:
  VoiceOver auf dem Mac, Orca unter Linux, eine japanische oder chinesische
  Eingabemethode für die Vorschau im Feld.
* Arabisch und Hebräisch erscheinen Zeichen für Zeichen von links nach
  rechts, ohne Verbindung der Buchstaben — Textformung und Rechts-nach-links
  sind nicht gebaut. Emoji kommen einfarbig.
* Die Schreibmarke in einem Textfeld wird dem Bildschirmleser noch nicht
  zeichenweise gemeldet, der Inhalt schon. Das `ui`-Modul und der Web-Bau
  melden nichts.
