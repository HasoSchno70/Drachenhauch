# Notenblatt-Editor

Eigenständiges Tool zum Komponieren in echter Notensatz-Darstellung (5-Linien-
System, Violin-/Bassschlüssel, Hilfslinien, Vorzeichen) statt des Zeilen-
Rasters des [Trackers](tracker.md). Jede Spur hat genau ein Instrument;
das fertige Stück lässt sich entweder als eigenes Projekt speichern **oder**
direkt in den Tracker übernehmen (ein Tracker-Kanal pro Spur, mit Instrument).

## Starten

Aus dem **Code-Editor**: Toolbar-Button (Notenlinien-Symbol) oder
`Datei → Notenblatt-Editor öffnen ...` (`Strg+Shift+N`). Standalone:
`gbscore [datei.json]` oder `gbrun.py --score [datei.json]` (braucht
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
  additiven Mixer (`gamebasic.audio_preview.Mixer`, derselbe Mixer wie im
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
(`"format": "gbscore-song"`, Zeiten in Viertel-Beats). `Öffnen...` liest es
zurück; das Laden ist permissiv (fehlende Felder bekommen sinnvolle
Defaults, wie beim Tracker-Format auch).

## In den Tracker übernehmen

Der Button **„In Tracker öffnen"** konvertiert das Stück
(`gamebasic.score.convert.to_tracker_song`) in ein Tracker-Projekt:

- Ein Tracker-Kanal pro Spur + der Tracker-Pflicht-Drum-Kanal am Ende.
- Jede Spur-Note wird auf die nächste Tracker-Zeile gerundet (4 Zeilen pro
  Viertel-Beat — Viertel=4 Zeilen, Achtel=2, Sechzehntel=1).
- Überschreitet das Stück 64 Zeilen, wird es automatisch auf mehrere
  Tracker-Patterns aufgeteilt (per Song-Order verkettet).
- Jede Spur-Instrument-Zuweisung wandert in den Tracker-Instrumenten-Pool.

Alle Vereinfachungen/Kürzungen (siehe unten) werden als **Warnungen**
angezeigt, bevor die Datei gespeichert wird — nichts geht unbemerkt verloren.
Anschließend wird die Datei gespeichert und `gbtracker` per Subprozess mit
dieser Datei gestartet.

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

Qt-frei in `gamebasic/score/document.py` (`ScoreDoc`/`Track`/`NoteEvent`) +
`gamebasic/score/convert.py` (`to_tracker_song`) — headless getestet:
`tests/test_score_document.py`, `tests/test_score_convert.py`,
`tests/test_scoreeditor_qt.py` (Offscreen-UI), `tests/test_audio_preview_mixer.py`
(geteilter Mixer).
