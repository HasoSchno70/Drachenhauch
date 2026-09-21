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

## Ohne Python: die IDE in Drachenhauch

Seit Weg C, Stand 3 (`docs/ide.md`) gibt es einen zweiten Installer, der
**kein PyInstaller und kein Python** braucht: `Drachenhauch-IDE.iss` packt
`dhrt.exe`, die IDE (`ide\ide.dh`), das Handbuch (`docs\*.md`) und die
Beispiele samt Begleit-Editoren. Gemessen 33 MB statt 92. Eigene AppId und
eigener Ordner (`Drachenhauch-IDE`), damit er die Qt-Fassung nicht ersetzt,
solange die noch die Referenz ist.

**Gebaut wird er in Drachenhauch** (seit 2026-09-16), nicht in Python:

```
.venv\Scripts\python.exe rust\build_runtime.py --hardware   # die Laufzeit selbst
dhrt run installer\bauen.dh                                 # Lizenzen + Inno Setup
```

`installer/bauen.dh` nimmt die Fassung aus der Laufzeit selbst (`VERSION$()`) —
gepackt wird genau die `dhrt.exe`, deren Nummer im Installer steht —, ruft
`installer/lizenzen.dh` und danach ISCC (aus `%ISCC%` oder den zwei
Standardpfaden; fehlt es, bleibt es bei der Lizenzdatei und einer Zeile, wie man
von Hand weiterkommt). `--ohne-installer` sammelt nur die Lizenzen.

**Das Bauen der Laufzeit bleibt bei `rust/build_runtime.py`** — unter Windows
lässt sich eine laufende `.exe` nicht überschreiben, und `bauen.dh` läuft ja in
ihr.

`installer/lizenzen.dh` schreibt `THIRD-PARTY-NOTICES-IDE.txt`: die Rust-Crates
aus `cargo metadata` (mit den ausgelieferten Features) samt ihren
LICENSE-/COPYING-/NOTICE-Texten, dazu den MPL-2.0-Volltext. Python-Pakete und Qt
stehen **nicht** darin — in dieser Distribution steckt keins von beidem. Der
Rust-Abschnitt entsteht Byte für Byte wie der von `gen_notices.py` (nachgemessen
an allen 370 Crates); Tests: `tests/pruef/werkzeug_installer.dhtest`.

Die Verknuepfungen starten `dhrt.exe run "{app}\ide\ide.dh"` mit dem
Beispielordner als Arbeitsverzeichnis; `.dh`-Dateien oeffnen sich in der
IDE (`-- "%1"`).

**Seit 2026-09-21 liefert er alles, was vorher nur der Qt-Installer hatte**
(`docs/entwurf-python-abbau.md`, 7.7 Punkt 4):

* **die Buecher** -- Lehrbuch, Handbook und Einstieg als `.docx` und
  `.epub` nach `{app}\buecher`, samt Startmenue-Eintraegen; wie im
  Qt-Installer nur, wenn sie gebaut sind (`tools/buch_bauen.dh`), fehlende
  werden uebergangen;
* **die ESP32-Sketche** neben die Beispiele (`...\Drachenhauch\esp32`, auf
  den `examples/159_esp32_bruecke.dh` verweist);
* **das Aufraeumen einer alten GameBasic-Installation**: Programm- und
  Beispielordner, Startmenue, Desktop-Verknuepfung, die ProgID
  `GameBasic.Source` und `.gb` -- dieses nur, wenn es noch auf uns zeigt
  (`.gb` ist auch die Endung fuer Game-Boy-ROMs); dazu die veralteten
  Vorschaubilder unter `examples\screenshots`;
* **das Signieren**, mit denselben Variablen wie unten (`GB_SIGN_CERT`,
  `GB_SIGN_PASS`, `GB_SIGN_TS`, `SIGNTOOL`). `bauen.dh` packt dafuer eine
  KOPIE der Laufzeit (`<ausgabe>\stufe\dhrt.exe`, an ISCC ueber
  `/DDhrtQuelle`): signiert wird die Kopie vor dem Einpacken, danach der
  fertige Installer. Die gebaute `dhrt.exe` bleibt unberuehrt -- sie laeuft
  womoeglich gerade als `bauen.dh`. **Ein Fehlschlag ist ein Abbruch**, nicht
  eine Warnung wie in `build_installer.py`: ein Installer, der signiert sein
  sollte und es nicht ist, fiele erst beim Nutzer auf. Ohne `GB_SIGN_CERT`
  sagt der Bau am Ende "Nicht signiert".

Geprueft in `tests/pruef/werkzeug_installer.dhtest` mit Attrappen fuer ISCC
und signtool (welche Datei, in welcher Reihenfolge, Fingerabdruck gegen
`.pfx`, Abbruch vor ISCC); ein echter Bau mit Inno Setup packt die sechs
Buecher und den Sketch nachweislich ein. Die AppId bleibt die eigene: die
Qt-Fassung wird NICHT ersetzt, beide lassen sich nebeneinander
installieren.

### macOS und Linux (seit 2026-09-19)

Dasselbe `bauen.dh` packt auf dem jeweiligen System (tar behaelt die
Ausfuehrungsrechte, `hdiutil` gibt es nur auf dem Mac):

```
cargo build --release --features "graphics dialogs db net http smtp"   # in rust/drachenhauch_runtime
rust/drachenhauch_runtime/target/release/dhrt run installer/bauen.dh
```

- **Linux:** `installer/output/Drachenhauch-IDE-<fassung>-linux-<arch>.tar.gz`.
  Installieren ohne root: `tar xzf ...` und `./Drachenhauch-IDE-.../install.sh`
  -- nach `~/.local/share/drachenhauch-ide`, dazu Menueeintrag, `.dh`-Zuordnung
  und `drachenhauch`/`dhrt` in `~/.local/bin`; `install.sh --entfernen`
  raeumt es wieder ab (ein eigenes `dhrt` in `~/.local/bin` bleibt stehen).
  Braucht zur Laufzeit ALSA, GTK 3 und OpenGL.
- **macOS:** `installer/output/Drachenhauch-IDE-<fassung>-macos-<arch>.dmg` mit
  `Drachenhauch.app` (Ad-hoc-signiert, nicht beglaubigt -- beim ersten Start
  ctrl-Klick -> Oeffnen, steht im LIESMICH des Abbilds).
  `.dh` ist als eigener Typ angemeldet; ein Doppelklick im Finder kommt als
  Apple-Event in der Laufzeit an (`rust/drachenhauch_runtime/src/finder.rs`) und
  oeffnet die Datei in der IDE wie eine ins Fenster gezogene.

Der Starter (`installer/posix/drachenhauch`, unter macOS `dhrt` selbst als
Hauptprogramm des Bundles, siehe `rust/drachenhauch_runtime/src/appstart.rs`)
kopiert die Beispiele beim Start nach `Dokumente/Drachenhauch/examples` (nur,
was fehlt) und oeffnet die IDE dort. Die Symbole liegen fertig in `installer/symbole/`; neu erzeugen mit
`dhrt run installer/symbole.dh`, wenn sich das Logo aendert. Gebaut und
ausprobiert werden beide Pakete im Paket-Lauf der CI (`package.yml`, Job
`paket-ohne-python`); Tests `tests/pruef/werkzeug_paket.dhtest`.
**Auf einem echten Mac ist die IDE noch nicht gestartet worden** -- die
macOS-Laeufer der CI haben kein OpenGL.

### macOS: signieren und beglaubigen (Notarisierung)

`bauen.dh` signiert immer mit Hardened Runtime. Mit zwei Dingen von Apple
signiert es zusaetzlich mit **Developer ID** und laesst das `.dmg` von Apple
**beglaubigen** -- dann oeffnet es sich auf jedem Mac ohne Rueckfrage
(sonst: ctrl-Klick -> Oeffnen). Die Zugangsdaten gehoeren niemals ins Repo;
`bauen.dh` liest sie nur aus der Umgebung:

| Variable | Inhalt |
|---|---|
| `DH_MAC_SIGNATUR` | die Identitaet im Schluesselbund, `Developer ID Application: Name (TEAMID)` |
| `DH_NOTAR_KEY` | Pfad zur `.p8` eines App-Store-Connect-API-Schluessels |
| `DH_NOTAR_KEY_ID` | seine Kennung (Key ID) |
| `DH_NOTAR_ISSUER` | die Aussteller-Kennung (Issuer ID) |

Mit allen vier: `codesign --options runtime --timestamp` mit der Identitaet,
das `.dmg` signiert, `xcrun notarytool submit --wait`, bei Ablehnung das
Protokoll des Auftrags, dann `xcrun stapler staple` und `validate` (das
Ticket haengt am Abbild, macOS prueft auch ohne Netz).

**Einrichten, einmalig** (das kann nur, wer das Apple-Konto hat):

1. Dem **Apple Developer Program** beitreten (kostenpflichtig, jaehrlich).
2. Ein Zertifikat **Developer ID Application** anlegen
   (developer.apple.com -> Certificates). Ohne Mac geht die
   Zertifikatsanfrage auch mit OpenSSL:
   `openssl req -new -newkey rsa:2048 -nodes -keyout dev.key -out dev.csr -subj "/CN=Dein Name/C=DE"`,
   die `.cer` danach mit `openssl x509 -inform DER -in developerID_application.cer -out dev.pem`
   und `openssl pkcs12 -export -legacy -inkey dev.key -in dev.pem -out zert.p12`
   zur `.p12` machen (mit einem Passwort). Auf dem Mac: in der
   Schluesselbundverwaltung als `.p12` exportieren.
3. In **App Store Connect** -> Benutzer und Zugriff -> Integrationen ->
   App Store Connect API einen Schluessel anlegen (Rolle "Developer"), die
   `.p8` herunterladen (geht nur einmal), Key ID und Issuer ID notieren.
4. Im GitHub-Repo unter Settings -> Secrets and variables -> Actions fuenf
   Secrets anlegen: `MAC_ZERT_P12` (die `.p12` als Base64),
   `MAC_ZERT_PASSWORT`, `MAC_NOTAR_KEY_P8` (die `.p8` als Base64),
   `MAC_NOTAR_KEY_ID`, `MAC_NOTAR_ISSUER`. Base64 unter Windows:
   `[Convert]::ToBase64String([IO.File]::ReadAllBytes("zert.p12")) | Set-Clipboard`,
   auf dem Mac: `base64 -i zert.p12 | pbcopy`.
5. Den Workflow **Package (manuell)** starten. Der Schritt "Signatur und
   Beglaubigung vorbereiten" legt einen kurzlebigen Schluesselbund an und
   setzt die vier Variablen; ohne die Secrets wird er uebersprungen und das
   Paket entsteht ad hoc signiert wie bisher. Mit ihnen prueft der Lauf
   danach `spctl --assess` (erwartet "Notarized Developer ID") und
   `stapler validate`.

**Ungeprueft, solange es die Secrets nicht gibt:** der Weg mit echter
Identitaet und die Antwort von Apple. Geprueft ist alles davor -- Hardened
Runtime ist gesetzt, und dhrt laeuft damit (Konsolenprogramm, Beispiele
kopieren, Datei vom Finder).

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
     (Beispiele + Lehrbuch in beiden Sprachen + Startmenü + optional
     PATH/.dh-Dateiverknüpfung).
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
| `bauen.dh` | Baut die Distribution **ohne Python**: Lizenzen + Inno Setup (`Drachenhauch-IDE.iss`). |
| `lizenzen.dh` | Sammelt die Lizenztexte der Rust-Crates → `THIRD-PARTY-NOTICES-IDE.txt`. |
| `gen_notices.py` | Dasselbe für die **Qt**-Fassung (zusätzlich Python-Pakete + Qt) → `THIRD-PARTY-NOTICES.txt`. |
| `licenses/` | Kanonische Volltexte (LGPL-3.0, GPL-3.0, MPL-2.0) für beide Sammler. |
| `Drachenhauch.ico`/`.icns`/`.png` · `THIRD-PARTY-NOTICES*.txt` · `output/` | generiert (gitignored). |

## Lizenz-Compliance (für den Verkauf)
- **`THIRD-PARTY-NOTICES.txt`** wird bei jedem Build automatisch erzeugt (`gen_notices.py`):
  sammelt die Lizenz-/Copyright-Texte aller gebündelten Python-Pakete (PySide6/Qt
  unter LGPLv3, NumPy, Pillow) **und** aller ~250 Rust-Crates der dhrt-Runtime
  (MIT/BSD/Apache-2.0/Zlib/MPL-2.0). MIT/BSD/Apache **verlangen** diese Beilage.
  Liegt nach Installation unter `{app}\THIRD-PARTY-NOTICES.txt` + Startmenü.
- **`THIRD-PARTY-NOTICES-IDE.txt`** ist das Gegenstück für die Distribution ohne
  Python (`installer/lizenzen.dh`): nur die Rust-Crates von `dhrt` plus
  MPL-2.0-Volltext — dort gibt es weder Python-Pakete noch Qt. `bauen.dh` erzeugt
  sie vor jedem Verpacken neu, damit sie zu den wirklich gebauten Features passt.
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

Dieselben Variablen gelten fuer den Installer ohne Python
(`dhrt run installer\bauen.dh`, siehe oben) -- dort signiert der Bau die
Kopie der `dhrt.exe` und den Installer, und ein Fehlschlag bricht ab.

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
