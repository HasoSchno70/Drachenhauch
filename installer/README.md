# GameBasic – Windows-Installer bauen

Erzeugt einen eigenständigen Windows-Installer (`GameBasic-Setup-<version>.exe`),
mit dem GameBasic **ohne installiertes Python** läuft: die komplette IDE
(Code-Editor + Sprite-/Tilemap-/Form-/Audio-/Anim-Editor) und die native Runtime
`gbrt` werden mitgeliefert.

## Schnellstart

```
.venv\Scripts\python.exe installer\build_installer.py
```

Das macht in einem Rutsch:
1. **gbrt-Runtime** bauen (falls `rust/gb_runtime/target/release/gbrt.exe` fehlt).
2. **App-Icon** `installer/GameBasic.ico` aus `gamebasic/assets/logo.png` erzeugen.
3. **PyInstaller**: friert `gbrun.py` + das `gamebasic`-Paket + PySide6 + numpy +
   Pillow zu `dist/GameBasic/` ein (onedir, kein Python nötig).
4. **Inno Setup** (ISCC): packt `dist/GameBasic` + `gbrt.exe` + Beispiele + Lehrbuch
   zu `installer/output/GameBasic-Setup-<version>.exe`.

Ergebnis: **`installer/output/GameBasic-Setup-<version>.exe`** – verteilbar.

### Optionen
- `--no-installer` – nur PyInstaller (Schritt 3), kein Inno-Schritt.
- `--rebuild-gbrt` – gbrt vorher neu bauen.

## Voraussetzungen
- Das Projekt-`.venv` mit `PyInstaller`, `PySide6`, `numpy`, `Pillow`
  (`requirements.txt`).
- **Inno Setup 6** (für Schritt 4): https://jrsoftware.org/isdl.php
  (`ISCC.exe`; gefunden unter `C:\Program Files (x86)\Inno Setup 6\`, per `ISCC`
  auf dem PATH, oder über die Umgebungsvariable `ISCC`). Fehlt es, bleibt
  `dist/GameBasic` stehen und der Installer-Schritt wird übersprungen.
- Rust-Toolchain für gbrt (siehe `docs/rust-runtime.md`), falls gbrt neu gebaut wird.

## Was der Installer einrichtet
- Installation nach `C:\Program Files\GameBasic`.
- **Beispiele** (142 `.gb` + Assets) nach `Öffentliche Dokumente\GameBasic\Beispiele`
  – ein **beschreibbarer** Ort (Program Files wäre schreibgeschützt, dann scheitern
  Demos, die Dateien schreiben). Der Doku-Ordner `screenshots/` wird weggelassen.
- Startmenü-Einträge: **GameBasic** (Editor/Auswahl), Sprite-Editor, Tilemap-Editor,
  Form-Designer, Audio-Studio, Beispiele.
- Optional (im Setup abwählbar):
  - Desktop-Verknüpfung.
  - **PATH-Eintrag** → `gbrt` und `GameBasic` im Terminal nutzbar.
  - **`.gb`-Dateiverknüpfung** (Doppelklick öffnet im Editor, Rechtsklick → „Mit
    GameBasic ausführen").

## Aufbau
| Datei | Zweck |
|---|---|
| `build_installer.py` | Orchestriert gbrt → Icon → PyInstaller → Inno. |
| `GameBasic.spec` | PyInstaller-Konfiguration (onedir, windowed, bündelt das Paket + Daten). |
| `GameBasic.iss` | Inno-Setup-Skript (Dateien, Verknüpfungen, PATH, Dateiverknüpfung). |
| `GameBasic.ico` | generiert (gitignored). |
| `output/` | fertiger Installer (gitignored). |

## Wie die installierte App `gbrt` findet
Der Installer legt `gbrt.exe` **neben** `GameBasic.exe`. `gbrun._find_gbrt()`
sucht im eingefrorenen Zustand zuerst im Verzeichnis der Exe (bzw. im
PyInstaller-Bundle) und erst danach im Dev-Baum – so funktioniert sowohl die
Installation als auch die Entwicklungsumgebung.

## Nur die Runtime verteilen?
Wer nur GameBasic-**Programme** ausführen/weitergeben will, braucht die IDE nicht:
- `gbrt run datei.gb` führt ein Programm aus.
- `gbrt --export datei.gb` baut eine **eigenständige Spiel-.exe** (hängt den
  kompilierten Payload an eine Kopie von `gbrt`). Dafür ist kein Installer nötig.
