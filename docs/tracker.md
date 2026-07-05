# Tracker (Musik-Editor)

Mehrspuriger Tracker zum Komponieren von Melodien/Musik — **einstellbare Kanalzahl** (4–32 über die `Kanäle:`-Spinbox, je eigene Wellenform **oder Sample-Instrument**; der LETZTE Kanal ist immer der Noise-/Drum-Kanal), mit **mehreren Patterns einstellbarer Länge** und **Song-Arrangement**. Über die reinen Chiptune-Wellenformen hinaus lassen sich **gesampelte Instrumente** (WAV/OGG) laden und über die Klaviatur spielen (Resampling). Komplementär zum [SFX-Generator](sfx-generator.md) (der einzelne Effekte macht).

Der Tracker ist ein **Tab im [Audio Studio](#audio-studio)** (`gbsound`), das ihn mit dem SFX-Generator unter einem fullscreen Fenster vereint.

## Starten

Am bequemsten als Tab im **Audio Studio**: `gbsound` (oder `gbrun.py --audio`) — fullscreen, Tracker + SFX-Generator zusammen.

Auch einzeln: aus dem **Code-Editor** Toolbar-Button (Noten-Symbol) oder `Datei → Tracker (Musik) öffnen ...` (`Strg+Shift+L`); standalone `gbtracker` oder `gbrun.py --tracker` (öffnen jetzt ebenfalls das Audio Studio auf dem Tracker-Tab). Braucht `PySide6` + `numpy`.

## Bedienung

Das Fenster ist im **Renoise-Stil** aufgeteilt: links ein **Instrument-Panel** (Spur-Sounds + Bibliothek), rechts die Pattern-Steuerung über dem **Gitter**, unten Song-Arrangement + Klaviatur.

- **Kanäle** — die `Kanäle:`-Spinbox neben dem BPM-Feld ändert die Kanalzahl des ganzen Songs (4–32). Bestehende Noten/Zuweisungen bleiben beim Vergrößern erhalten, neue Kanäle kommen leer dazu; der **letzte** Kanal bleibt immer Drum/Noise, unabhängig von der Gesamtzahl.
- **Pattern-Gitter** — Reihen `00`…`N` (Zeit, von oben nach unten) × `Ch1`…`Ch(n-1)` (Töne) + `Drum` (Noise, letzte Spalte). **Jeder Kanal hat seine eigene Akzentfarbe** (Spalten-Header, Notentext, Lautstärke-Regler, VU-Meter — zyklisch aus der Theme-Palette), damit sich viele Kanäle auf einen Blick unterscheiden lassen. Drum-Hits bleiben immer magenta, Note-Off gedämpft-grau; Lautstärke-Suffix `v…` mint, Slide `s…` amber, Effekt (`Arp…` etc.) magenta. Jede 4. Reihe ist leicht, jede 16. stärker hinterlegt (Beat-Raster), die laufende Wiedergabe-Reihe wird betont.
- **Note setzen:** Zelle anklicken (auswählen), dann auf der **Klaviatur** unten eine Taste klicken → die Note (z. B. `C4`) landet in der Zelle, der Cursor springt eine Reihe weiter. `Entf`/`Rücktaste` löscht die Zelle (bei markiertem Block: alle markierten Zellen). Die Klaviatur spielt auch einzelne Töne zum Vorhören; **Oktave** wählt den Bereich.
- **Block-Auswahl** (Shift-Klick/Ziehen über mehrere Zellen) für Mehrfach-Operationen:
  - **`Strg+C`/`Strg+X`/`Strg+V`** — Kopieren/Ausschneiden/Einfügen eines rechteckigen Zellen-Blocks (Note+Lautstärke+Slide+Effekt), Einfügen an der aktuellen Auswahlposition (am Pattern-Rand geclippt).
  - **`Strg+Pfeil hoch/runter`** (+ `Shift` = Oktave) — transponiert alle Noten im markierten Block um ±1 (bzw. ±12) Halbtöne; der Drum-Kanal bleibt dabei unangetastet.
  - **`Strg+I`** — interpoliert je Kanal linear zwischen der ersten und letzten gesetzten Note im markierten Bereich (klassisches Tracker-Werkzeug für Glissandi/Tonhöhen-Übergänge, z. B. für ein Portamento über mehrere Reihen).
- **Mute / Solo** — jede Kanalzeile im linken Panel hat einen **`M`**- (stumm) und **`S`**-Knopf (solo). Solo lässt nur die Solo-Spuren klingen, `M` schlägt Solo. Wirkt beim **Vorhören/Playback** (der WAV-Render mischt weiterhin alle Spuren) — praktisch, um beim Komponieren einzelne Spuren isoliert zu hören.
- **Kanal-Lautstärke (Mixer-Fader)** — unter jeder Kanalzeile ein echter **Schieberegler** (0–100 %, in der Kanalfarbe), wie der Lautstärke-Fader in Renoise/FastTracker/OpenMPT. Separat von der Noten-Lautstärke (`Vol`-Spalte) und vom Instrument-Volume — wirkt als Mixer-Multiplikator im Vorhören, im **WAV-Render** UND im **GB-Code-Export**. Bleibt er auf 100 %, wird nichts in die Projektdatei geschrieben (abwärtskompatibel).
- **VU-Meter** — unter jeder Kanalzeile leuchtet während der Wiedergabe ein dünner Pegel-Balken auf (Peak der gespielten Note, klingt sanft ab) — du siehst auf einen Blick, welche Spur gerade spielt.
- **Spur-Sounds (ein Keyboard-Klang pro Spur)** — der Abschnitt **`Spur-Sounds`** im linken Panel hat für jeden Kanal (`Ch1`…`Ch(n-1)`/`Drum`, scrollbar bei vielen Kanälen) ein **Dropdown mit fertigen Instrumenten**: Flügel, E-Piano, Orgel, Streicher, Synth-Pad, Bass, Lead, Glocke, Marimba, Chip-Sounds … und Drum-Sounds (Kick/Snare/HiHat/Tom). Einfach pro Spur einen Sound wählen — kein WAV-Suchen nötig. Out of the box ist schon je Kanal ein sinnvoller Klang gesetzt (Flügel / Streicher / Bass / Kick, weitere Kanäle: Standard-Wellenform).
- **Notenlänge:** Eine Note klingt so lange, **bis die nächste Note derselben Spur kommt** (bzw. bis zum Pattern-Ende). Haltende Instrumente (Orgel/Streicher) klingen durch, perkussive (Piano/Glocke) klingen natürlich aus — du steuerst die Länge über den Abstand der Noten im Gitter.
- **Note-Off** (Taste `0` oder Button „◼ Note Aus") — schneidet eine klingende Note **gezielt vor** der nächsten Note ab, statt sie automatisch bis dahin durchklingen zu lassen (klassisches Tracker-Konzept, „Key Off" bei XM/IT). Zelle in der Zeile zeigt `OFF`. Wirkt im **WAV-Render** als echte Sustain-Grenze; im Live-GB-Export (der ohnehin kein Sustain über Reihen hinweg kennt) wie eine leere Zelle.
- **Eigene Instrumente** (Abschnitt **`Bibliothek`** im linken Panel) — über die Presets hinaus kannst du **eigene Samples** laden (`+ Sample (WAV)...` — WAV/OGG wird über die Klaviatur resampelt, MOD/XM/IT-Stil) oder ein **Keymap** bauen (`+ Keymap...`). Geladene Instrumente erscheinen in den Spur-Sound-Dropdowns und werden im Projekt (`.json`) eingebettet (self-contained).
- **SoundFont laden** (`+ SoundFont (.sf2)...`) — **echte Instrumente aus SoundFont-Dateien** (General MIDI oder Hersteller-Sounds): wähle eine `.sf2`-Datei, dann im Such-Dialog ein Preset (z. B. „Acoustic Grand Piano", „Strings", „Bass"). Der SoundFont-Reader baut daraus ein **Keymap-Instrument** (Multisample mit Tasten-Zonen, Grundton und Loop genau wie im SoundFont) — sofort über die ganze Tastatur spielbar und im WAV-Render dabei. So bekommst du Hunderte realistische Klänge ohne eigenes Sampling. (Pure-Python-Reader, keine externe Abhängigkeit; Velocity-Layer/Modulatoren werden vereinfacht, Stereo-Samples als Mono.)
- **Keymap-Instrument** (`+ Keymap...`) — **verschiedene Samples über die Klaviatur verteilen** (Multisampling / Drumkit). Im Dialog fügst du per `Sample hinzufügen...` mehrere WAVs hinzu; jedes bildet eine **Zone** mit einem Tastenbereich (`Lo`–`Hi`) und einem **Root** (die Taste, bei der es unverschoben klingt). Spielst du eine Note, wählt das Instrument die passende Zone und resampelt ihr Sample relativ zum Root.
  - **Multisample:** ein Instrument an mehreren Tönen aufgenommen (z. B. Klavier-C2/C3/C4), jede Aufnahme deckt einen Bereich ab → weniger Resampling-Artefakte, realistischer.
  - **Drumkit:** `Auto-Drumkit (ab C2)` legt jedem Sample **genau eine Taste** zu (Root == diese Taste → kein Pitch-Shift) — Kick, Snare, HiHat … je eine Note. So spielst du ein Schlagzeug aus dem Pattern-Gitter.
  - Keymap-Instrumente erscheinen mit `▦` in der Liste, werden wie andere Instrumente einem Kanal zugewiesen und sind im Vorhören, Pattern-Playback und **WAV-Render** voll dabei. `Bearbeiten...` (siehe unten) setzt ADSR auch für Keymaps.
- **Instrument bearbeiten** (`Bearbeiten...`) — Dialog für das gewählte Sample-Instrument:
  - **Grundton** (MIDI) — die Note, bei der das Sample 1:1 (unverschoben) klingt. Stimmt das Sample richtig ein.
  - **Loop** (`none`/`forward`/`pingpong` + Start/Ende in Samples) — lässt ein kurzes Sample **endlos sustainen**: beim Erreichen des Loop-Endes springt die Wiedergabe zum Loop-Start zurück (`forward`) bzw. läuft im Zickzack (`pingpong`). Ohne Loop verstummt das Sample nach einmaligem Durchlauf. Eine **Wellenform-Ansicht** zeigt das Sample mit zwei ziehbaren Markern (grün = Start, rot = Ende) — per Maus direkt auf der Wellenform setzen, bidirektional mit den Start-/Ende-Spinboxen synchron.
  - **ADSR-Hüllkurve** (Attack/Decay/Sustain/Release) — formt die Lautstärke über die Notendauer (weiches Ein-/Ausblenden, Sustain-Pegel). Ein kurzer Anti-Click-Fade am Ende ist immer aktiv.
  - **Pan** (Schieberegler L … Mitte … R) — Stereo-Position des Instruments für den WAV-Render; Label zeigt „L 40 %"/„Mitte"/„R 70 %" live an.
- **Audio rendern** (`Audio (WAV)...`) — **der Weg, Sample-Songs ins Spiel zu bringen:** der ganze Song wird offline zu einer WAV gemischt (alle Kanäle gleichzeitig, mit Resampling, Loop, ADSR, Noten-Lautstärke, **Pitch-Slide und Effekt-Spalte**). Eine Note klingt bis zur nächsten Note desselben Kanals (Sustain über leere Reihen). Vor dem Render fragt ein Dialog **Stereo** (wertet den Instrument-Pan aus) und **Amiga-Hard-Panning** (Kanal 1+4 links, 2+3 rechts — der klassische Paula-Stereoeindruck; bei mehr als 4 Kanälen wiederholt sich das Muster) ab. Im Spiel dann einfach `PLAYMUSIC("song.wav")` — völlig unabhängig von den Engine-Audio-Grenzen (das Mischen passiert im Editor in numpy). Ideal für fertige Spielmusik mit echten Samples.
  - *Hinweis:* Der `GB-Code`-Export erzeugt weiterhin den **Live-Synth-Player** (Chiptune, zur Laufzeit, inkl. Lautstärke + Slide via `AUDIO_SFX`) und kann Sample-Kanäle und die Effekt-Spalte (Arp/Vib/Ret/Off) nicht direkt; für Sample-Songs und Effekte nimmt man den **Audio-Export**.
- **Effekt-Spalten pro Note:** Zelle mit Note auswählen, dann:
  - **`Vol`** (1–15, `–` = Standard) — Lautstärke; Suffix `v9` in der Zelle, wirkt auf Amplitude (Vorhören + Player).
  - **`Slide`** (−12…+12 Halbtöne, 0 = kein Slide; nur Ton-Kanäle) — **Pitch-Slide/Portamento**: die Note gleitet über die Reihen-Dauer um die angegebenen Halbtöne nach oben/unten. Suffix `s+2`/`s-3` in der Zelle. Im WAV-Render gilt der Slide für **alle** Instrumente (Synth + Sample/Keymap); im GB-Code-Export werden Slide-Noten als `AUDIO_SFX` (vorberechneter Hz/s-Bend) gerendert, ohne Slide bleibt `AUDIO_TONE`.
  - **`FX`** + **Parameter** — klassische Tracker-Effekte (wirken im **WAV-Render**, instrument-unabhängig):
    - **`Arp`** (Arpeggio) — Parameter als zwei Hex-Nibbles `xy`: die Note springt im Tick-Takt zwischen Grundton, +`x` und +`y` Halbtönen (z. B. `71` = `0x47` → Dur-Akkord +4/+7). Der typische C64-Akkord aus einem Kanal.
    - **`Vib`** (Vibrato) — `xy`: Speed `x` (Hz), Tiefe `y` (·0,125 Halbtöne); die Tonhöhe pendelt sinusförmig.
    - **`Ret`** (Retrigger) — schlägt den Notenkopf alle *Parameter* Ticks neu an (Stotter-/Roll-Effekt).
    - **`Off`** (Sample-Offset) — startet das Sample `Parameter`·512 Frames später.
    - Anzeige in der Zelle als Suffix, z. B. `Arp47`.
  - Eine Note zu löschen entfernt auch ihre Effekte.
- **BPM** stellt das Tempo (16tel-Schritte).
- **↶/↷** (oder `Strg+Z` / `Strg+Y`) machen Änderungen rückgängig bzw. wieder her — Noten, Pattern-/Order-Operationen, BPM, Wellenform. `Neu`/`Öffnen` verwerfen die Historie.

### Patterns

- **Pattern**-Auswahl (Combo) wechselt das angezeigte Pattern. **Reihen** stellt die Länge des aktuellen Patterns ein (1–64; bestehende Noten oben bleiben erhalten).
- **+ Pattern** legt ein neues an, **Duplizieren** kopiert das aktuelle, **Löschen** entfernt es (mind. eines bleibt). **Leeren** setzt nur das aktuelle Pattern zurück.

### Song-Arrangement (Order)

Die **Song**-Leiste unten ist die Abspiel-Reihenfolge der Patterns — ein Pattern darf mehrfach vorkommen (z. B. `Intro → Vers → Vers → Refrain`).

- **+ akt.** hängt das aktuelle Pattern hinten an, **entf.** entfernt den ausgewählten Eintrag, **◀**/**▶** verschieben ihn. **Doppelklick** auf einen Eintrag öffnet dessen Pattern im Gitter.

### Abspielen

- **▶ Pattern** spielt das aktuelle Pattern in Schleife.
- **▶ Song** spielt die ganze Order ab; das Gitter folgt automatisch dem laufenden Pattern, die aktuelle Reihe ist markiert.

### Speichern / Laden

**Neu** / **Öffnen** / **Speichern** verwalten das Projekt als `.json` (Tempo, Wellenformen, alle Patterns, Order). Eigenes Editor-Format — nicht zu verwechseln mit dem GB-Code-Export.

## Export (GB-Code)

`GB-Code` erzeugt einen **frame-basierten Player**. Die Order wird zu einer flachen Timeline expandiert (wiederholte Patterns werden dupliziert), die Noten landen als `INTEGER`-Arrays pro Kanal, plus zwei SUBs (`TRACKER_PLAY_ROW`, `TRACKER_UPDATE`). Hat ein Kanal Noten mit gesetzter **Lautstärke**, kommt eine `trkV<n>`-Spur + ein `TRACKER_AMP`-Helfer dazu (Amplitude in Prozent, 0 = Standard 0.5); mit **Slide** kommt eine `trkSl<n>`-Spur dazu (Hz/s, der Player nutzt dann `AUDIO_SFX` statt `AUDIO_TONE`). Ohne Effekte bleibt der Player unverändert schlank. Im Game-Loop rufst du:

```basic
TRACKER_UPDATE(DELTA() * 1000.0)
```

Das spielt den Song non-blocking ab (advanced über die Zeit, nutzt `AUDIO_TONE`/`AUDIO_NOISE` + `PLAYSOUND`). Läuft über die native Runtime `gbrt`.

Das Datenmodell + I/O + Export liegen Qt-frei in `gamebasic/tracker/song.py`, die **Sample-Instrumente** (Laden/Resampling/Serialisierung) in `gamebasic/tracker/instrument.py`, der **Mixer/Render** in `gamebasic/tracker/mixer.py` (headless getestet: `tests/test_tracker_song.py`, `tests/test_tracker_instrument.py`, `tests/test_tracker_mixer.py`).

## Audio Studio

Tracker und [SFX-Generator](sfx-generator.md) leben zusammen im **Audio Studio** — einem fullscreen Fenster mit zwei Reitern (`🎹 Tracker / Song` und `💥 SFX-Generator`). Start: `gbsound` / `gbrun.py --audio`, oder im Code-Editor die jeweiligen Menüpunkte (sie öffnen dasselbe Studio auf dem passenden Tab). `F11` schaltet echtes Vollbild, `Strg+1`/`Strg+2` wechseln die Tabs. Jeder Tab behält seinen eigenen Undo-Verlauf (`Strg+Z`/`Strg+Y` wirken auf den fokussierten Tab).

> **Sampler-Ausbau (laufend):** Der Tracker wird schrittweise vom Chiptune-Synth zum vollwertigen Sampler ausgebaut. **Stufe 1 (fertig):** Sample-Instrumente laden + über die Klaviatur resampeln + vorhören. **Stufe 2 (fertig):** Grundton, Loop-Punkte (forward/pingpong), ADSR-Hüllkurve. **Stufe 4+5 (fertig):** numpy-Software-Mixer (`tracker/mixer.py`) + **Render-to-File** (`Audio (WAV)...` → Song als WAV für `PLAYMUSIC`), inkl. **Stereo + Amiga-Hard-Panning** und **Pitch-Slide für alle Instrumente**. **Stufe „Keymap" (fertig):** Multisample/Drumkit — Samples über Tasten-Zonen (`Keymap...`). **Effekt-Spalte (fertig):** Arpeggio/Vibrato/Retrigger/Sample-Offset im Render. **Kanäle/Komfort (fertig):** einstellbare Kanalzahl (4–32), Block-Copy/Transpose/Interpolate, grafischer Instrument-Editor (Wellenform+Loop-Marker), Note-Off, Kanal-Mixer-Fader + Kanalfarben, Pan als Schieberegler. **Geplant:** Live-Sampler-Export (`SAMPLE_PLAY`-basierter GB-Code), Architektur-Wechsel auf Instrument-pro-Note statt pro-Kanal.

> **Effekt-Spalten:** **Lautstärke** (`Vol`), **Pitch-Slide/Portamento** (`Slide`) und die **Effekt-Spalte** (`FX`: Arp/Vib/Ret/Off) pro Note. Vol + Slide gehen in den GB-Code-Live-Player (Slide via `AUDIO_SFX`-Hz/s-Bend); die `FX`-Effekte wirken im **WAV-Render** (als instrument-unabhängiges Post-Processing der gerenderten Note in `mixer.apply_effect`).
