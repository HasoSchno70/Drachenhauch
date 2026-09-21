# Notenblatt-Editor

Ein Werkzeug zum Komponieren in echter Notensatz-Darstellung (fünf Linien je
Spur, Violin- oder Bassschlüssel, Hilfslinien, Vorzeichen) statt des
Zeilenrasters des [Trackers](tracker.md). Jede Spur hat genau ein
Instrument; das fertige Stück lässt sich als eigene Datei sichern **oder**
direkt in den Tracker übernehmen (ein Tracker-Kanal je Spur).

Das Notenblatt ist ein Drachenhauch-Programm:
`examples/199_notenblatt.dh`. Es ersetzt den früheren Qt-Editor `dhscore`
(Weg B aus [entwurf-python-abbau.md](entwurf-python-abbau.md)) und braucht
kein Python.

## Starten

- In der [IDE](ide.md): `Werkzeuge → Notenblatt` oder die Befehlspalette
  („Werkzeug: Notenblatt“). Das Werkzeug läuft als eigener `dhrt`-Prozess.
- Von der Kommandozeile:

```
dhrt run examples/199_notenblatt.dh [-- stueck.json]
```

Mit einer Datei als Argument wird sie geladen; gibt es sie noch nicht,
beginnt ein leeres Stück, das beim Sichern dort landet. Das Fenster startet
maximiert und lässt sich in der Größe ändern.

## Aufbau

Oben die Werkzeuge (Dauer, punktiert, Vorzeichen, Modus, Fingersatz-Zahl,
BPM, Abspielen, In Tracker, +/- Spur) unter dem Menü `Datei`/`Bearbeiten`,
links je Spur Name, Schlüssel und Instrument (die 18 Werks-Presets des
Trackers), in der Mitte die Notensysteme — mit den normalen Zeichenbefehlen
gemalt: fünf Linien, Taktschattierung, Taktstriche, Hilfslinien, Köpfe
(gefüllt unter der Halben), Hälse, Fähnchen, Balken für Achtel- und
Sechzehntelläufe gleicher Dauer, Pausen, Bindebögen als `SPLINE`,
Fingersatz, Staccato-Punkt, eine Vorschau an der Maus und der Spielkopf.

**Schlüssel und Vorzeichen sind selbst gezeichnet** (Kurve durch feste
Punkte, Kreuz aus vier Strichen, B als eigene Form): die Notenzeichen der
Symbolschriften kamen als Fragezeichen an, und ein Fragezeichen am
Zeilenanfang ist schlimmer als eine schlichte Form.

## Bedienung

| Aktion | so |
|---|---|
| Note setzen | Klick aufs System. Die gewählte Dauer (Ganze bis Sechzehntel, optional punktiert) ist zugleich das Raster; das Vorzeichen (natürlich/Kreuz/B) kommt aus der Klappliste |
| Note entfernen | noch einmal auf dieselbe Stelle klicken, oder rechte Maustaste (entfernt immer) |
| Note verschieben | ziehen (Zeit und Tonhöhe); eine andere Note am Ziel wird ersetzt, Bogen-Anker wandern mit, eine Pause bleibt eine Pause |
| Pause | Modus „Pause“: ein Klick setzt eine Pause statt einer Note |
| Bindebogen | Modus „Bindebogen“: erste Note, dann zweite; ESC bricht die Auswahl ab, Rechtsklick entfernt Bögen an der Stelle |
| Fingersatz | Modus „Fingersatz“: Klick weist die gewählte Zahl (1–5) zu, derselbe Klick mit derselben Zahl nimmt sie wieder weg |
| Staccato | Modus „Staccato“: Klick schaltet es an und aus; es wirkt auf den Klang (halbe Dauer beim Abspielen und beim Export) |
| Spuren | `+`/`-` oder `Bearbeiten`-Menü, bis zu 8 Spuren; Name mit Enter übernehmen |
| Schlüsselwechsel | Klappliste je Spur. Liegen die Noten danach weit ab vom System, rückt ein Versatz um ganze Oktaven sie heran und die Statuszeile sagt es (Strg+Z nimmt es zurück) |
| Tempo | BPM-Feld, Enter übernimmt (20..400) |
| Abspielen | Leertaste oder F5, noch einmal = Stopp |
| Rollen | Mausrad = Zeit, Umschalt+Rad = Spuren |
| Rückgängig | Strg+Z / Strg+Y — jede Änderung, der Stand wird als JSON-Text gemerkt |
| Datei | Strg+N neu, Strg+O öffnen, Strg+S sichern, Strg+Umschalt+S sichern unter, Strg+Q beenden |
| In Tracker öffnen | Strg+T oder der Knopf (siehe unten) |

**Abspielen** läuft auf einer Audio-Uhr (`AUDIO_CLOCK_NEW` +
`AUDIO_PLAY_AT`, eine Sechzehntel je Tick): alle Noten werden beim Start
geplant und klingen samplegenau, nicht bildgetrieben. Gestoppt wird, indem
die Uhr entfernt wird — ein Klang, der auf eine Uhr wartet, die es nicht
mehr gibt, startet nie.

**Beenden** (Menü, Fensterkreuz, Alt+F4) fragt nur dann nach, wenn das
Stück nicht gesichert ist: Sichern / Verwerfen / Abbrechen.

## Dateiformat

Das Notenblatt liest und schreibt dasselbe JSON wie die frühere Qt-Fassung
(`"format": "dhscore-song"`, Zeiten in Viertel-Beats):

- oben `format`, `version`, `bpm`, `time_sig` (immer `[4, 4]`) und `tracks`,
- je Spur `name`, `clef` (`treble`/`bass`), `instrument` (ein Instrument im
  Format des Trackers) und `notes`, dazu `slurs` (Paare von Beat-Positionen),
- je Note `start`, `dur` und `pitch` (MIDI-Nummer) oder `rest: true`,
  wahlweise `accidental`, `staccato`, `fingering`.

Das Laden ist nachsichtig: fehlende Felder bekommen Vorgaben (120 BPM,
Violinschlüssel, Flügel, „Stimme n“). Ein Beispiel ist
`examples/notenblatt_demo.json`.

## In den Tracker übernehmen

**In Tracker öffnen** sichert das Stück (falls nötig), rechnet es in ein
Tracker-Projekt um und schreibt es als `<name>_tracker.json` neben die
Stückdatei. Danach startet der Tracker in Drachenhauch
(`examples/190_tracker.dh`) mit dieser Datei. Die Regeln:

- ein Tracker-Kanal je Spur (mindestens vier) plus ein leerer Drum-Kanal,
- vier Tracker-Zeilen je Viertel-Beat (Viertel = 4, Achtel = 2,
  Sechzehntel = 1), jeder Beginn auf die nächste Zeile gerundet,
- Patterns zu 64 Zeilen, per Song-Reihenfolge verkettet,
- Akkord (mehrere Noten am selben Beginn) → die höchste Note,
- Staccato = halbe Dauer, mindestens eine Zeile; `NOTE_OFF` am Ende, wenn
  die Zelle frei ist,
- eine Note, die über eine Pattern-Grenze klingen würde, wird dort gekürzt,
- das Instrument jeder Spur wandert in den Instrumenten-Pool.

**Was dabei verloren geht, sagt ein Kasten VOR dem Schreiben**: Akkord auf
die höchste Note reduziert, Note auf die nächste Tracker-Zeile gerundet,
Note fiel aus dem Song heraus, Note an der Pattern-Grenze gekürzt — mit Spur
und Beat. Gezeigt werden bis zu acht Warnungen („… und k weitere“).
„Trotzdem öffnen“ (Enter) schreibt und startet den Tracker, „Abbrechen“ (ESC)
schreibt nichts.

## Tests

`tests/pruef/werkzeug_notenblatt.dhtest` fährt das Werkzeug mit echten
Klicks und Tasten über die Eingabe-Wiedergabe: Note setzen, entfernen,
ziehen und Strg+Z, Pause und Bindebogen über die Klappliste, zweite Spur mit
Instrument, Abspielen. Der Tracker-Export wird am Demo-Stück (Akkord,
Staccato, Note über die 64-Zeilen-Grenze, zwei Spuren) Zelle für Zelle mit
einem festen Stand verglichen, den der frühere Python-Konverter geliefert
hat; zwei weitere Fälle prüfen den Warnkasten (Abbrechen schreibt nichts,
„Trotzdem öffnen“ schreibt das Gitter). Stück und Tracker-Projekt liest
dabei ein Drachenhauch-Programm mit dem `json`-Modul.

## Grenzen

Bewusste Vereinfachungen, nicht stillschweigend verschluckt:

- **Festes 4/4-Metrum** — die Oberfläche zeigt und ändert es nicht.
- **Ein Instrument je Spur**, höchstens acht Spuren.
- **Akkorde werden beim Export reduziert** (ein Tracker-Kanal ist
  einstimmig); ein Klick auf eine Note eines Akkords trifft die erste am
  Beat.
- **Balken nur bei gleicher Dauer** — ein Lauf aus Achteln und Sechzehnteln
  bricht an der Dauergrenze auf, keine Teilbalken.
- **Bindebögen und Fingersätze sind reine Notation** — der Export und die
  Wiedergabe ignorieren sie.
- **Kein Schlagzeug-Spurtyp** — der Drum-Kanal des Exports bleibt leer.
- **Kein optisches Layout** — keine Kollisionsvermeidung zwischen
  Vorzeichen, Fingersätzen, Bögen und Hilfslinien; bei dichten Passagen
  können sie sich überlappen.
- **Schlüsselwechsel ohne Rückfrage** — die Qt-Fassung fragte vor dem
  Oktavversatz, hier wird verschoben und gesagt.
- **Neu und Öffnen fragen nicht** nach ungesicherten Änderungen (nur das
  Beenden tut es).
- Kein echtes Vollbild per F11 (die Qt-Fassung hatte es).
