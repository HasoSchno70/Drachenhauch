# Drachenhauch – Distribution bauen (Windows/macOS/Linux)

Die Distribution enthält die Laufzeit `dhrt`, die IDE in Drachenhauch
(`ide/ide.dh`), das Handbuch (`docs/*.md`), die Beispiele samt der Werkzeuge
(Sprite, Tilemap, Tracker, SFX, Partikel, Form-Designer, Anim-FSM,
Notenblatt), die Bücher und die ESP32-Sketche — **ohne Python und ohne Qt**.
Windows bekommt einen Installer (`Drachenhauch-IDE-Setup-<fassung>.exe`),
macOS ein `.dmg` mit `Drachenhauch.app`, Linux ein `.tar.gz` mit `install.sh`.

Bis 2026-09-21 gab es daneben die Qt-Fassung (PyInstaller,
`build_installer.py`, `Drachenhauch.iss`, `Drachenhauch.spec`,
`gen_notices.py`); sie ist mit dem Python-Teil gelöscht.

## Windows

**Gebaut wird er in Drachenhauch** (seit 2026-09-16):

```
python rust\build_runtime.py --hardware                   # die Laufzeit selbst
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
* **das Signieren**, mit denselben Variablen wie unten (`DH_SIGN_CERT`,
  `DH_SIGN_PASS`, `DH_SIGN_TS`, `SIGNTOOL`). `bauen.dh` packt dafuer eine
  KOPIE der Laufzeit (`<ausgabe>\stufe\dhrt.exe`, an ISCC ueber
  `/DDhrtQuelle`): signiert wird die Kopie vor dem Einpacken, danach der
  fertige Installer. Die gebaute `dhrt.exe` bleibt unberuehrt -- sie laeuft
  womoeglich gerade als `bauen.dh`. **Ein Fehlschlag ist ein Abbruch**, nicht
  eine Warnung wie in `build_installer.py`: ein Installer, der signiert sein
  sollte und es nicht ist, fiele erst beim Nutzer auf. Ohne `DH_SIGN_CERT`
  sagt der Bau am Ende "Nicht signiert".

Geprueft in `tests/pruef/werkzeug_installer.dhtest` mit Attrappen fuer ISCC
und signtool (welche Datei, in welcher Reihenfolge, Fingerabdruck gegen
`.pfx`, Abbruch vor ISCC); ein echter Bau mit Inno Setup packt die sechs
Buecher und den Sketch nachweislich ein. Die AppId ist eine eigene (nicht die der früheren Qt-Fassung): ein Rechner,
auf dem die Qt-Fassung noch installiert ist, behält sie, bis man sie über
„Programme entfernen" löscht.

### macOS und Linux (seit 2026-09-19)

Dasselbe `bauen.dh` packt auf dem jeweiligen System (tar behaelt die
Ausfuehrungsrechte, `hdiutil` gibt es nur auf dem Mac):

```
cargo build --release --features "graphics dialogs video db net http smtp"   # in rust/drachenhauch_runtime
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

## Voraussetzungen

- **Rust** (`cargo`) und ein beliebiges **Python 3** für `rust/build_runtime.py`
  — nur die Standardbibliothek, ohne venv (`tests/pruef/bauskripte.dhtest`
  hält das fest).
- **Windows:** Inno Setup 6 (`ISCC.exe` unter `C:\Program Files (x86)\Inno Setup 6\`
  oder über die Umgebungsvariable `ISCC`). Fehlt es, bleibt es bei der
  Lizenzdatei und einer Zeile, wie man von Hand weiterkommt.
- **macOS:** `hdiutil` und `codesign` (Bordmittel).
- **Linux:** nichts weiter.

## Paket-Lauf in der CI

`.github/workflows/package.yml` (manuell, Actions-Tab → „Run workflow“) baut
die Pakete für macOS und Linux MIT Grafik und probiert sie aus: unter Linux
entpacken, `install.sh`, die IDE unter xvfb starten, die Sammlung
`tests/pruef/werkzeug_paket.dhtest` mit dieser Laufzeit; unter macOS das `.dmg`
einhängen, Signatur und Aufbau prüfen. Den Windows-Installer baut er nicht —
Inno Setup haben die Läufer nicht.

## Aufbau

| Datei | Zweck |
|---|---|
| `bauen.dh` | Baut die Distribution: Lizenzen, dann Inno Setup (Windows) bzw. `paket.dh` (macOS/Linux); signiert unter Windows, wenn `DH_SIGN_CERT` gesetzt ist. |
| `paket.dh` | Das `.tar.gz` für Linux und das `.dmg` für macOS. |
| `Drachenhauch-IDE.iss` | Inno-Setup-Skript: Dateien, Verknüpfungen, PATH, `.dh`-Zuordnung, Bücher, ESP32, Aufräumen von GameBasic. |
| `lizenzen.dh` | Sammelt die Lizenztexte der Rust-Crates → `THIRD-PARTY-NOTICES-IDE.txt`. |
| `licenses/` | Der MPL-2.0-Volltext für `lizenzen.dh`. |
| `posix/` | Vorlagen für Linux und macOS (Starter, `install.sh`, `Info.plist`, LIESMICH). |
| `symbole/`, `symbole.dh` | Die Programmsymbole, fertig erzeugt aus `daten/bilder/logo.png`. |
| `Drachenhauch.ico` | Symbol des Setup-Programms; installiert als `drachenhauch.ico` fuer Verknuepfungen, `.dh`-Dateien und die Deinstallation (dhrt.exe selbst traegt keins -- ein exportiertes Spiel ist eine Kopie davon). |
| `EULA.txt` | Endbenutzer-Lizenzvertrag (**Vorlage** – vor Verkauf juristisch prüfen, `[PLATZHALTER]` ersetzen). |
| `THIRD-PARTY-NOTICES-IDE.txt` · `output/` | erzeugt (gitignored). |

## Lizenz-Compliance (für den Verkauf)
- **`THIRD-PARTY-NOTICES-IDE.txt`** (`installer/lizenzen.dh`): die Lizenztexte
  aller Rust-Crates von `dhrt` (MIT/BSD/Apache-2.0/Zlib/MPL-2.0) plus
  MPL-2.0-Volltext. MIT/BSD/Apache **verlangen** diese Beilage. `bauen.dh`
  erzeugt sie vor jedem Verpacken neu, damit sie zu den wirklich gebauten
  Features passt; nach der Installation liegt sie neben `dhrt.exe` und im
  Startmenü.
- **`EULA.txt`** ist eine Vorlage; ersetze die `[PLATZHALTER]` und lass sie vor einem
  kommerziellen Vertrieb prüfen. Sie regelt u.a., dass **vom Nutzer erstellte Spiele
  ihm gehören** und samt dhrt-Runtime **frei (auch kommerziell) weitergegeben** werden
  dürfen – wichtig, damit deine Nutzer ihre Spiele verkaufen können.
- Beispiel-Asset-Lizenzen sind geprüft + dokumentiert (`examples/ASSET-CREDITS.md`).

## Code-Signing (gegen die SmartScreen-„Unbekannter Herausgeber"-Warnung)
`bauen.dh` signiert **automatisch** eine Kopie der `dhrt.exe` (vor dem
Einpacken) und den fertigen Installer – **sobald** ein Zertifikat über
Umgebungsvariablen konfiguriert ist. Ohne Konfiguration sagt der Bau am Ende
„Nicht signiert“; ein Fehlschlag beim Signieren bricht ab.

```
set DH_SIGN_CERT=C:\keys\meincert.pfx     REM .pfx-Datei ODER SHA1-Thumbprint im Zertspeicher
set DH_SIGN_PASS=geheim                    REM nur bei .pfx
set DH_SIGN_TS=http://timestamp.digicert.com   REM optional (Default gesetzt)
dhrt run installer\bauen.dh
```

- Braucht **`signtool.exe`** (Windows SDK; wird automatisch unter
  `Windows Kits\10\bin\*\x64\` gesucht, oder via `SIGNTOOL`).
- Zertifikat: ein **Code-Signing-Zertifikat** von einer CA (OV günstiger, **EV**
  baut sofort SmartScreen-Reputation auf). EV-Tokens liegen oft als Hardware-USB
  vor – dann Thumbprint statt `.pfx` verwenden.
- Den **Uninstaller** signiert dieser Weg nicht; dafür bräuchte es Inno-`SignTool`
  in `Drachenhauch-IDE.iss`.

Offen für den Verkauf (kein Code mehr): Zertifikat kaufen, EULA-`[PLATZHALTER]`
ausfüllen, ggf. DE-Verbraucherrecht (Impressum/Widerruf).

## Nur die Runtime verteilen?
Wer nur Drachenhauch-**Programme** ausführen/weitergeben will, braucht die IDE nicht:
- `dhrt run datei.dh` führt ein Programm aus.
- `dhrt --export datei.dh` baut eine **eigenständige Spiel-.exe** (hängt den
  kompilierten Payload an eine Kopie von `dhrt`). Dafür ist kein Installer nötig.
