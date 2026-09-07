# Notenblatt-Editor

Eigenständiges Tool zum Komponieren in echter Notensatz-Darstellung (5-Linien-
System, Violin-/Bassschlüssel, Hilfslinien, Vorzeichen) statt des Zeilen-
Rasters des [Trackers](tracker.md). Jede Spur hat genau ein Instrument;
das fertige Stück lässt sich entweder als eigenes Projekt speichern **oder**
direkt in den Tracker übernehmen (ein Tracker-Kanal pro Spur, mit Instrument).

## Starten

Aus dem **Code-Editor**: Toolbar-Button (Notenlinien-Symbol) oder
`Datei → Notenblatt-Editor öffnen ...` (`Strg+Shift+N`). Standalone:
`dhscore [datei.json]` oder `dhrun.py --score [datei.json]` (braucht
`PySide6` und `numpy`; für echte Wiedergabe zusätzlich `sounddevice`).

## Bedienung

- **Dauer-Auswahl** (Ganze/Halbe/Viertel/Achtel/Sechzehntel + `Punktiert`) —
  legt fest, was ein Klick auf die Notenzeile einträgt. Die Dauer ist
  gleichzeitig das **Snap-Raster**: ein Klick landet immer auf dem nächsten
  Vielfachen der gewählten Dauer.
- **Vorzeichen** (♮/♯/♭) — verschiebt die durch den Klick bestimmte Stammnote
  um einen Halbton. Angezeigt wird ein Vorzeichen immer als Kreuz der
  Stammnote darunter (siehe Limitationen).
- **Pause** — solange aktiv, trägt ein Klick eine Pause statt einer Note ein.
- **Linksklick** auf eine Notenzeile setzt/entfernt eine Note (nochmal auf
  dieselbe Stelle klicken entfernt sie wieder). **Rechtsklick** entfernt
  immer, egal welcher Eingabe-Modus aktiv ist.
- **Note verschieben** — Klick auf eine bestehende Note halten und ziehen
  verschiebt sie zu Zeit/Tonhöhe unter dem Cursor (statt Löschen + Neu-
  Setzen). Ein Klick ohne Bewegung entfernt die Note weiterhin wie bisher.
  Landet eine verschobene Note auf einer anderen bestehenden Note, wird
  diese ersetzt. Eine Pause bleibt beim Verschieben eine Pause (nur die
  Zeitposition ändert sich).
- **Vorschau-Cursor** — beim Bewegen der Maus über das Notensystem zeigt ein
  halbtransparenter Notenkopf (bzw. eine Pause) genau an, wo/mit welcher
  Tonhöhe ein Klick landen würde, bevor man tatsächlich klickt.
- **Spur-Leiste** (oberhalb jedes Notensystems): Name, Notenschlüssel
  (Violin-/Bassschlüssel) und Instrument (Werks-Presets aus dem Tracker,
  Flügel/Streicher/Bass/Glocke/…) einstellbar. `+ Spur` / `- Spur` fügen
  Spuren hinzu/entfernen die letzte (mindestens eine bleibt erhalten).
- **Notenschlüssel-Wechsel transponiert NICHT automatisch** — wie in echter
  Notationssoftware ändert sich nur die Darstellung, nicht die Tonhöhe. Hat
  die Spur bereits Noten und würden sie beim neuen Schlüssel weit ab vom
  System liegen (viele Hilfslinien), fragt ein Dialog, ob sie um ganze
  Oktaven verschoben werden sollen — Melodie/Intervalle bleiben dabei exakt
  erhalten, nur das Register ändert sich.
- **Rückgängig/Wiederholen** (`↶`/`↷`-Buttons, `Strg+Z`/`Strg+Y`) — jede
  Änderung (Noten, Instrument, Schlüssel, Spuren, Tempo) ist rückgängig
  machbar, auf Basis von Snapshots des ganzen Stücks (wie beim Tracker).
- **Ungespeicherte Änderungen** — der Fenstertitel zeigt ein `*` nach dem
  Dateinamen, solange etwas nicht gespeichert ist. `Neu`/`Öffnen` und das
  Schließen des Fensters fragen bei ungespeicherten Änderungen nach
  (Speichern/Verwerfen/Abbrechen).
- **Tempo (BPM)** — wirkt auf Wiedergabe-Geschwindigkeit und die spätere
  Tracker-Übernahme (`row_ms = 60000/bpm/4`).
- **▶ Abspielen** — spielt alle Spuren gleichzeitig über den geteilten
  additiven Mixer (`drachenhauch.audio_preview.Mixer`, derselbe Mixer wie im
  Tracker) mit einem laufenden Cursor auf jedem Notensystem.
- **Balken-Gruppierung** — zusammenhängende Achtel/Sechzehntel gleicher Dauer
  im selben Beat bekommen automatisch einen gemeinsamen Balken statt
  Einzel-Fähnchen (Sechzehntel: Doppelbalken).
- **Eingabe-Modus** (Note/Pause/Bindebogen/Fingersatz/Staccato, exklusive
  Buttons) — bestimmt, was ein Linksklick auf eine Note bewirkt:
  - **Note/Pause**: wie oben beschrieben (setzen/verschieben/entfernen).
  - **Bindebogen**: erste Note anklicken, dann die zweite — verbindet beide
    mit einem Phrasierungsbogen (rein visuell, keine Wirkung auf Wiedergabe
    oder Tracker-Export). Dieselbe Note zweimal hintereinander bricht die
    Auswahl ab. **Rechtsklick** entfernt in diesem Modus einen Bogen an
    dieser Stelle, ohne die Note zu löschen. Wird eine verbundene Note
    gezogen, wandert ihr Bogen-Anker automatisch mit.
  - **Fingersatz**: Note anklicken weist ihr die rechts gewählte Zahl (1–5)
    zu; erneuter Klick mit derselben Zahl entfernt sie wieder.
  - **Staccato**: Note anklicken schaltet Staccato an/aus (kurzer, "kurz
    abgehackter" Ton). Wirkt tatsächlich auf den Klang: sowohl bei der
    Wiedergabe im Editor als auch nach der Tracker-Übernahme klingt die Note
    nur die halbe notierte Dauer, dann folgt eine Pause bis zum nächsten
    Ereignis (mindestens 1 Tracker-Zeile bleibt immer hörbar). Pausen können
    nicht staccato sein.
- **Info-Leiste** (unten) — zeigt live den aktuellen Eingabe-Modus (Dauer,
  Vorzeichen, Pause an/aus), einen Stück-Überblick (Spuren, Beats/Takte,
  BPM) und Kurzhinweise zur Bedienung.
- **`F11`** schaltet echtes Vollbild um (das Fenster startet bereits
  maximiert).

## Speichern / Laden

`Datei → Speichern...` schreibt ein eigenes `*.json`-Format
(`"format": "dhscore-song"`, Zeiten in Viertel-Beats). `Öffnen...` liest es
zurück; das Laden ist permissiv (fehlende Felder bekommen sinnvolle
Defaults, wie beim Tracker-Format auch).

## In den Tracker übernehmen

Der Button **„In Tracker öffnen"** konvertiert das Stück
(`drachenhauch.score.convert.to_tracker_song`) in ein Tracker-Projekt:

- Ein Tracker-Kanal pro Spur + der Tracker-Pflicht-Drum-Kanal am Ende.
- Jede Spur-Note wird auf die nächste Tracker-Zeile gerundet (4 Zeilen pro
  Viertel-Beat — Viertel=4 Zeilen, Achtel=2, Sechzehntel=1).
- Überschreitet das Stück 64 Zeilen, wird es automatisch auf mehrere
  Tracker-Patterns aufgeteilt (per Song-Order verkettet).
- Jede Spur-Instrument-Zuweisung wandert in den Tracker-Instrumenten-Pool.

Alle Vereinfachungen/Kürzungen (siehe unten) werden als **Warnungen**
angezeigt, bevor die Datei gespeichert wird — nichts geht unbemerkt verloren.
Anschließend wird die Datei gespeichert und `dhtracker` per Subprozess mit
dieser Datei gestartet.

## In Drachenhauch: `examples/199_notenblatt.dh`

Seit 2026-09-07 gibt es das Notenblatt auch **in Drachenhauch selbst** (Weg B
aus [entwurf-python-abbau.md](entwurf-python-abbau.md), der dritte und
letzte der Editoren ohne Piloten). 1 449 Zeilen gegen 1 710 der Qt-Fassung
(1 266 UI + 282 Modell + 162 Konverter), Faktor 0,85 — hoch, weil hier
nichts wegfällt, was die Laufzeit hätte übernehmen können: Notensatz muss
man zeichnen, und der Tracker-Konverter ist Logik, keine Oberfläche.

```
dhrt run examples/199_notenblatt.dh [-- stueck.json]
```

Oben die Werkzeuge (Dauer, punktiert, Vorzeichen, Modus, Fingersatz-Zahl,
BPM, Abspielen, In Tracker, +/- Spur), links je Spur Name, Schlüssel und
Instrument (die 18 Presets des Trackers), in der Mitte die Notensysteme —
mit den normalen Zeichenbefehlen gemalt: fünf Linien, Taktschattierung,
Taktstriche, Hilfslinien, Köpfe (gefüllt unter der Halben), Hälse, Fähnchen,
Balken für Achtel- und Sechzehntelläufe gleicher Dauer, Pausen, Bögen als
`SPLINE`, Fingersatz, Staccato-Punkt, Vorschau an der Maus, Spielkopf.
**Schlüssel und Vorzeichen sind selbst gezeichnet** (Kurve durch feste
Punkte, Kreuz aus vier Strichen): die Notenzeichen der Symbolschriften
kamen als Fragezeichen an, und ein Fragezeichen am Zeilenanfang ist
schlimmer als eine schlichte Form.

| Aktion | so |
|---|---|
| Note setzen | Klick aufs System (Dauer = Raster, Vorzeichen aus der Klappliste) |
| Note entfernen | nochmal klicken, oder rechte Maustaste |
| Note verschieben | ziehen (Zeit und Tonhöhe); eine andere Note am Ziel wird ersetzt, Bogen-Anker wandern mit |
| Pause / Bindebogen / Fingersatz / Staccato | Modus in der Klappliste, dann wie in der Qt-Fassung (Bogen: erste Note, dann zweite; Rechtsklick entfernt Bögen an der Stelle) |
| Schlüsselwechsel | Klappliste je Spur; liegen die Noten danach weit ab, rückt ein Oktavversatz sie heran und die Statuszeile sagt es (kein Dialog — Strg+Z nimmt es zurück) |
| Abspielen | Leertaste oder F5: alle Noten auf einer Audio-Uhr (`AUDIO_CLOCK` + `AUDIO_PLAY_AT`, eine Sechzehntel je Tick), samplegenau statt bildgetrieben; Staccato halbiert wie beim Export |
| Rollen | Mausrad = Zeit, Umschalt+Rad = Spuren (bis zu 8) |
| In Tracker öffnen | Strg+T: schreibt `<name>_tracker.json` neben das Stück und startet den Tracker-Piloten (190, der seither ein Dateiargument nimmt) damit |

**Der Tracker-Export rechnet mit denselben Regeln wie `score/convert.py`**
(4 Zeilen je Beat, Patterns zu 64 Zeilen, Akkord → höchste Note, Staccato
halbiert mit mindestens einer Zeile, `NOTE_OFF` am Ende wenn die Zelle frei
ist, Kürzung an der Pattern-Grenze, mindestens vier Kanäle, Drum-Kanal
leer) — und der Test hält ihn daran fest: `tests/test_pilot_notenblatt.py`
lässt den Piloten das Demo-Stück `examples/notenblatt_demo.json` exportieren
(Akkord, Staccato, Note über die 64-Zeilen-Grenze, zwei Spuren) und
vergleicht das Gitter Zelle für Zelle mit `to_tracker_song`. Die Stück-Datei
liest `ScoreDoc.load_json`, das Tracker-Projekt `Song.load_json` — drei
fremde Leser.

Noch nicht: der Transponier-Dialog beim Schlüsselwechsel (es wird
verschoben und gesagt), mehr als acht Spuren, ein Klick auf eine Note eines
Akkords trifft immer die erste am Beat. Bewusst nicht: die Notenzeichen aus
einer Schrift.

## V1-Limitationen

Bewusste Vereinfachungen, nicht stillschweigend verschluckt:

- **Festes 4/4-Metrum** — die UI zeigt/ändert das Metrum nicht.
- **Ein Instrument pro Spur** — kein Pattern-Zell-Override wie im Tracker.
- **Akkorde werden reduziert:** mehrere Noten mit demselben Start-Beat auf
  einer Spur → beim Tracker-Export bleibt nur die höchste Note (ein
  Tracker-Kanal ist einstimmig).
- **Vorzeichen immer als Kreuz** der Stammnote darunter, nie als B —
  musikalisch enharmonisch gleichwertig, aber nicht immer die übliche
  Schreibweise.
- **Balken-Gruppierung nur bei gleicher Dauer** — ein Lauf aus Achteln
  UND Sechzehnteln im selben Beat bekommt keine Partial-Balken, sondern
  bricht an der Dauer-Grenze in separate Gruppen/Einzel-Fähnchen auf.
- **Noten über eine Tracker-Pattern-Grenze hinaus** (alle 64 Zeilen = 16
  Beats = 4 Takte) werden dort gekappt — Tracker-Patterns können nicht binden.
- **Kein Schlagzeug-Spurtyp** — der Pflicht-Drum-Kanal des Tracker-Exports
  bleibt unbelegt.
- **Bindebögen und Fingersätze sind reine Notationszusätze** — sie werden
  beim Tracker-Export komplett ignoriert (kein Tracker-Konzept dafür).
- **Kein optisches Notenlinien-Layout** — anders als echte Notensatz-Software
  (Sibelius/Finale/Dorico/LilyPond) gibt es keine automatische Kollisions-
  vermeidung zwischen Vorzeichen/Fingersätzen/Bindebögen/Hilfslinien und
  keinen proportionalen (nur zeit-proportionalen) Notenabstand. Bei sehr
  dichten Passagen können sich Beschriftungen überlappen.

## Geplant

Andere Taktarten, Triolen/Tuplets, Partial-Balken bei gemischten
Notendauern, kontextuelle Kreuz/B-Schreibweise, Mehrstimmigkeit pro Spur
(automatische Verteilung auf zusätzliche Tracker-Kanäle beim Export),
weitere Artikulationen (Akzent, Tenuto), echtes optisches Notenlinien-
Layout mit Kollisionsvermeidung.

## Datenmodell

Qt-frei in `drachenhauch/score/document.py` (`ScoreDoc`/`Track`/`NoteEvent`) +
`drachenhauch/score/convert.py` (`to_tracker_song`) — headless getestet:
`tests/test_score_document.py`, `tests/test_score_convert.py`,
`tests/test_scoreeditor_qt.py` (Offscreen-UI), `tests/test_audio_preview_mixer.py`
(geteilter Mixer).
