# Tracker (Musik-Editor)

Mehrspuriger Chiptune-Tracker zum Komponieren von Melodien/Musik — 3 Ton-Kanäle (je eigene Wellenform) + 1 Noise-Kanal (Drums), 16 Reihen pro Pattern. Komplementär zum [SFX-Generator](sfx-generator.md) (der einzelne Effekte macht).

## Starten

Aus dem **Code-Editor**: Toolbar-Button (Noten-Symbol) oder `Datei → Tracker (Musik) öffnen ...` (`Strg+Shift+L`). Standalone: `gbtracker` oder `gbrun.py --tracker` (braucht `PySide6` + `numpy`).

## Bedienung

- **Pattern-Gitter** — Reihen `00`…`15` (Zeit, von oben nach unten) × 4 Spalten (`Ch1`/`Ch2`/`Ch3` = Töne, `Drum` = Noise).
- **Note setzen:** Zelle anklicken (auswählen), dann auf der **Klaviatur** unten eine Taste klicken → die Note (z. B. `C4`) landet in der Zelle, der Cursor springt eine Reihe weiter. `Entf`/`Rücktaste` löscht die Zelle. Die Klaviatur spielt auch einzelne Töne zum Vorhören; **Oktave** wählt den Bereich.
- **Wellenform pro Ton-Kanal** (`Ch1`–`Ch3`): square / saw / sine / triangle.
- **BPM** stellt das Tempo (16tel-Schritte). **▶ Abspielen** spielt das Pattern in Schleife, die laufende Reihe ist markiert. **Leeren** setzt alles zurück.

## Export (GB-Code)

`GB-Code` erzeugt einen **frame-basierten Player** — die Noten als `INTEGER`-Arrays pro Kanal plus zwei SUBs (`TRACKER_PLAY_ROW`, `TRACKER_UPDATE`). Im Game-Loop rufst du:

```basic
TRACKER_UPDATE(DELTA() * 1000.0)
```

Das spielt den Song non-blocking ab (advanced über die Zeit, nutzt `AUDIO_TONE`/`AUDIO_NOISE` + `PLAYSOUND`). Läuft in **beiden** Pfaden — Tree-Walker und native Runtime `gbrt`.
