# Tracker (Musik-Editor)

Mehrspuriger Chiptune-Tracker zum Komponieren von Melodien/Musik — 3 Ton-Kanäle (je eigene Wellenform) + 1 Noise-Kanal (Drums), mit **mehreren Patterns einstellbarer Länge** und **Song-Arrangement**. Komplementär zum [SFX-Generator](sfx-generator.md) (der einzelne Effekte macht).

## Starten

Aus dem **Code-Editor**: Toolbar-Button (Noten-Symbol) oder `Datei → Tracker (Musik) öffnen ...` (`Strg+Shift+L`). Standalone: `gbtracker` oder `gbrun.py --tracker` (braucht `PySide6` + `numpy`).

## Bedienung

- **Pattern-Gitter** — Reihen `00`…`N` (Zeit, von oben nach unten) × 4 Spalten (`Ch1`/`Ch2`/`Ch3` = Töne, `Drum` = Noise).
- **Note setzen:** Zelle anklicken (auswählen), dann auf der **Klaviatur** unten eine Taste klicken → die Note (z. B. `C4`) landet in der Zelle, der Cursor springt eine Reihe weiter. `Entf`/`Rücktaste` löscht die Zelle. Die Klaviatur spielt auch einzelne Töne zum Vorhören; **Oktave** wählt den Bereich.
- **Wellenform pro Ton-Kanal** (`Ch1`–`Ch3`): square / saw / sine / triangle.
- **Effekt-Spalten pro Note:** Zelle mit Note auswählen, dann:
  - **`Vol`** (1–15, `–` = Standard) — Lautstärke; Suffix `v9` in der Zelle, wirkt auf Amplitude (Vorhören + Player).
  - **`Slide`** (−12…+12 Halbtöne, 0 = kein Slide; nur Ton-Kanäle) — **Pitch-Slide/Portamento**: die Note gleitet über die Reihen-Dauer um die angegebenen Halbtöne nach oben/unten. Suffix `s+2`/`s-3` in der Zelle. Im Export werden Slide-Noten als `AUDIO_SFX` (mit vorberechnetem Hz/s-Bend) gerendert, Noten ohne Slide bleiben `AUDIO_TONE`.
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

Das spielt den Song non-blocking ab (advanced über die Zeit, nutzt `AUDIO_TONE`/`AUDIO_NOISE` + `PLAYSOUND`). Läuft in **beiden** Pfaden — Tree-Walker und native Runtime `gbrt`.

Das Datenmodell + I/O + Export liegen Qt-frei in `gamebasic/tracker/song.py` (headless getestet: `tests/test_tracker_song.py`).

> **Effekt-Spalten:** **Lautstärke** (`Vol`) und **Pitch-Slide/Portamento** (`Slide`) pro Note. Slide-Noten werden im Export über `AUDIO_SFX` mit einem pro Reihe vorberechneten Hz/s-Bend gerendert (kein kontinuierlicher Synth-Umbau nötig).
