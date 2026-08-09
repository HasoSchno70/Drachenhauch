# Drachenhauch – Distribution bauen (Windows/macOS/Linux)

Erzeugt eine eigenständige Drachenhauch-Distribution, mit der Drachenhauch **ohne
installiertes Python** läuft: die komplette IDE (Code-Editor +
Sprite-/Tilemap-/Form-/Audio-/Anim-Editor) und die native Runtime `dhrt`
werden mitgeliefert. Windows bekommt einen Installer
(`Drachenhauch-Setup-<version>.exe`), macOS ein `.app` in einem `.dmg`, Linux
einen Tarball mit `install.sh`.

> **Cross-Platform-Status:** Der Windows-Pfad ist etabliert und lokal
> verifiziert. **macOS/Linux sind neu (Cross-Platform-Migration Phase 4) und
> NICHT auf echter Hardware getestet** — Entwicklung läuft bisher
> ausschließlich unter Windows. Nur der PyInstaller-Schritt selbst wurde
> lokal (Windows) regressionsgetestet; die macOS-/Linux-spezifischen Schritte
> (`.dmg` via `hdiutil`, `install.sh`) sind nach bestem Wissen geschrieben +
> isoliert simuliert (Shell-Syntax-Check + Testlauf gegen ein Fake-`$HOME`),
> aber nie auf einem echten Mac/Linux-Rechner gelaufen. Rückmeldungen von
> echten macOS-/Linux-Nutzern sind ausdrücklich erwünscht.

## Schnellstart

```
.venv\Scripts\python.exe installer\build_installer.py     # Windows
.venv/bin/python installer/build_installer.py              # macOS/Linux
```

Das macht in einem Rutsch:
1. **dhrt-Runtime** bauen (falls `rust/drachenhauch_runtime/target/release/dhrt[.exe]` fehlt;
   fehlt sie danach immer noch, wird nur gewarnt statt abzubrechen — nützlich, um
   die Paketierung selbst zu testen, ohne die volle Grafik-Toolchain zu brauchen).
2. **App-Icon** aus `drachenhauch/assets/logo.png` erzeugen (`.ico` Windows, `.icns`
   macOS, `.png` Linux).
3. **PyInstaller**: friert `dhrun.py` + das `drachenhauch`-Paket + PySide6 + numpy +
   Pillow ein (onedir, kein Python nötig) — `dist/Drachenhauch/` (Windows/Linux) bzw.
   `dist/Drachenhauch.app` (macOS).
4. Plattformspezifische Paketierung:
   - **Windows**: Inno Setup (ISCC) → `installer/output/Drachenhauch-Setup-<version>.exe`
     (Beispiele + Lehrbuch + Startmenü + optional PATH/.dh-Dateiverknüpfung).
   - **macOS**: `dhrt` neben die App-Binary legen (`Contents/MacOS/`), `.app` mit
     `hdiutil` in `installer/output/Drachenhauch-<version>-macOS.dmg` packen.
   - **Linux**: `dhrt` neben die Binary legen, `install.sh` (XDG-Desktop-
     Integration ohne sudo/root: `~/.local/share/Drachenhauch` + `.desktop`-Eintrag +
     Icon) dazupacken, alles zu
     `installer/output/Drachenhauch-<version>-linux-x86_64.tar.gz`.

### Optionen
- `--no-installer` – nur PyInstaller (Schritt 3), kein Paketier-Schritt.
- `--rebuild-dhrt` – dhrt vorher neu bauen.

## Voraussetzungen
- Das Projekt-`.venv` mit den `editors`- und `package`-Extras: `pip install -e ".[editors,package]"`.
- **Windows** – Inno Setup 6 (für Schritt 4): https://jrsoftware.org/isdl.php
  (`ISCC.exe`; gefunden unter `C:\Program Files (x86)\Inno Setup 6\`, per `ISCC`
  auf dem PATH, oder über die Umgebungsvariable `ISCC`). Fehlt es, bleibt
  `dist/Drachenhauch` stehen und der Installer-Schritt wird übersprungen.
- **macOS** – `hdiutil` (System-Bordmittel, immer vorhanden).
- **Linux** – keine externen Tools nötig (reines Python + Tarball).
- Rust-Toolchain für dhrt (siehe `docs/rust-runtime.md`), falls dhrt neu gebaut wird.

## Cross-Platform-Verifikation (CI)

`.github/workflows/package.yml` baut die Distribution manuell auslösbar
(`workflow_dispatch`, GitHub → Actions-Tab → „Package (manuell)" → „Run
workflow") auf `ubuntu-latest`/`macos-latest`/`windows-latest` und lädt das
Ergebnis als Artefakt hoch — mit `dhrt --no-graphics` (schnell, ohne
System-Bibliotheken), prüft also nur, ob die Paketier-Schritte selbst
durchlaufen, nicht die volle Runtime.

## Was die Distribution einrichtet

**Windows** (Inno Setup):
- Installation nach `C:\Program Files\Drachenhauch`.
- **Beispiele** (142 `.dh` + Assets + Showcase-Thumbnails `screenshots/`) nach
  `%PUBLIC%\Documents\Drachenhauch\examples`. Das ist exakt der `project_root` der
  installierten App (`dhrun._project_root()`), damit der Editor Beispiele **und**
  Showcase-Vorschaubilder findet – und der Ort ist **beschreibbar** (Program Files
  wäre schreibgeschützt). Beim Deinstallieren bleiben die Beispiele erhalten.
- Startmenü-Einträge: **Drachenhauch** (öffnet direkt den **Code-Editor** – ohne
  Auswahlfenster), Sprite-Editor, Tilemap-Editor, Form-Designer, Audio-Studio,
  Beispiele.
- Optional (im Setup abwählbar): Desktop-Verknüpfung, **PATH-Eintrag** (`dhrt`
  und `Drachenhauch` im Terminal nutzbar), **`.dh`-Dateiverknüpfung** (Doppelklick
  öffnet im Editor, Rechtsklick → „Mit Drachenhauch ausführen").

**macOS** (`.dmg`): `.app` per Drag-and-Drop nach `/Applications` (oder woanders
hin) ziehen — kein Installations-Skript-Schritt wie bei Inno Setup. `.dh`-Dateien
im Finder sind über `CFBundleDocumentTypes` mit Drachenhauch verknüpft. Beispiele
liegen im Bundle und werden beim **ersten Start** automatisch nach
`~/Documents/Drachenhauch/examples` kopiert (`dhrun._seed_examples_if_missing`).

**Linux** (Tarball): `tar xzf Drachenhauch-<version>-linux-x86_64.tar.gz && ./Drachenhauch-dist/install.sh`
installiert nach `~/.local/share/Drachenhauch` (XDG, **kein sudo/root nötig**),
legt einen `drachenhauch`-Befehl unter `~/.local/bin` an und trägt einen
`.desktop`-Eintrag samt Icon ein (Anwendungsmenü). Beispiele werden wie bei
macOS beim ersten Start automatisch aus dem Bundle kopiert.

## Aufbau
| Datei | Zweck |
|---|---|
| `build_installer.py` | Orchestriert dhrt → Icon → Notices → PyInstaller → plattformspezifische Paketierung (Inno/DMG/Tarball). |
| `Drachenhauch.spec` | PyInstaller-Konfiguration (onedir, windowed, bündelt das Paket + Daten; macOS bekommt zusätzlich einen `BUNDLE()`-Schritt für ein echtes `.app`). |
| `Drachenhauch.iss` | Inno-Setup-Skript (Dateien, Verknüpfungen, PATH, Dateiverknüpfung, EULA) — nur Windows. |
| `EULA.txt` | Endbenutzer-Lizenzvertrag (**Vorlage** – vor Verkauf juristisch prüfen, `[PLATZHALTER]` ersetzen). Wird im Windows-Setup als Zustimmungsseite gezeigt; auf macOS/Linux als Referenzdatei mit ins Paket kopiert. |
| `gen_notices.py` | Sammelt alle Drittanbieter-Lizenztexte → `THIRD-PARTY-NOTICES.txt` (plattformunabhängig). |
| `licenses/` | Kanonische Volltexte (LGPL-3.0, GPL-3.0, MPL-2.0) für `gen_notices.py`. |
| `Drachenhauch.ico`/`.icns`/`.png` · `THIRD-PARTY-NOTICES.txt` · `output/` | generiert (gitignored). |

## Lizenz-Compliance (für den Verkauf)
- **`THIRD-PARTY-NOTICES.txt`** wird bei jedem Build automatisch erzeugt (`gen_notices.py`):
  sammelt die Lizenz-/Copyright-Texte aller gebündelten Python-Pakete (PySide6/Qt
  unter LGPLv3, NumPy, Pillow) **und** aller ~250 Rust-Crates der dhrt-Runtime
  (MIT/BSD/Apache-2.0/Zlib/MPL-2.0). MIT/BSD/Apache **verlangen** diese Beilage.
  Liegt nach Installation unter `{app}\THIRD-PARTY-NOTICES.txt` + Startmenü.
- **`EULA.txt`** ist eine Vorlage; ersetze die `[PLATZHALTER]` und lass sie vor einem
  kommerziellen Vertrieb prüfen. Sie regelt u.a., dass **vom Nutzer erstellte Spiele
  ihm gehören** und samt dhrt-Runtime **frei (auch kommerziell) weitergegeben** werden
  dürfen – wichtig, damit deine Nutzer ihre Spiele verkaufen können.
- **Nicht `--onefile` bauen** (LGPL/Qt): die Qt-DLLs müssen als austauschbare Dateien
  vorliegen – die `onedir`-Spec erfüllt das.
- Beispiel-Asset-Lizenzen sind geprüft + dokumentiert (`examples/ASSET-CREDITS.md`); der
  frühere „Mario"-Satz wurde zu einem eigenständigen Plattformer-Satz umgebaut.

## Code-Signing (gegen die SmartScreen-„Unbekannter Herausgeber"-Warnung)
Der Build signiert **automatisch** `Drachenhauch.exe`, `dhrt.exe` und den fertigen
Installer – **sobald** ein Zertifikat über Umgebungsvariablen konfiguriert ist.
Ohne Konfiguration ist die Signierung ein No-Op (der Build läuft normal durch).

```
set GB_SIGN_CERT=C:\keys\meincert.pfx     REM .pfx-Datei ODER SHA1-Thumbprint im Zertspeicher
set GB_SIGN_PASS=geheim                    REM nur bei .pfx
set GB_SIGN_TS=http://timestamp.digicert.com   REM optional (Default gesetzt)
.venv\Scripts\python.exe installer\build_installer.py
```

- Braucht **`signtool.exe`** (Windows SDK; wird automatisch unter
  `Windows Kits\10\bin\*\x64\` gesucht, oder via `SIGNTOOL`/PATH).
- Zertifikat: ein **Code-Signing-Zertifikat** von einer CA (OV günstiger, **EV**
  baut sofort SmartScreen-Reputation auf). EV-Tokens liegen oft als Hardware-USB
  vor – dann Thumbprint statt `.pfx` verwenden.
- Den **Uninstaller** signiert der externe Weg nicht; dafür Inno-`SignTool`
  aktivieren (auskommentiert in `Drachenhauch.iss`).

Offen für den Verkauf (kein Code mehr): Zertifikat kaufen, EULA-`[PLATZHALTER]`
ausfüllen, ggf. DE-Verbraucherrecht (Impressum/Widerruf).

## Wie die installierte App `dhrt` findet
Der Installer legt `dhrt.exe` **neben** `Drachenhauch.exe`. `dhrun._find_dhrt()`
sucht im eingefrorenen Zustand zuerst im Verzeichnis der Exe (bzw. im
PyInstaller-Bundle) und erst danach im Dev-Baum – so funktioniert sowohl die
Installation als auch die Entwicklungsumgebung.

## Nur die Runtime verteilen?
Wer nur Drachenhauch-**Programme** ausführen/weitergeben will, braucht die IDE nicht:
- `dhrt run datei.dh` führt ein Programm aus.
- `dhrt --export datei.dh` baut eine **eigenständige Spiel-.exe** (hängt den
  kompilierten Payload an eine Kopie von `dhrt`). Dafür ist kein Installer nötig.
