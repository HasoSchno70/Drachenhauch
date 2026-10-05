# Entwurf: Pakete -- Bibliotheken holen und teilen

Stand 2026-10-04. Punkt „Kein Paketverzeichnis“ aus
[entwurf-anwendungen-und-tempo.md](entwurf-anwendungen-und-tempo.md).

**Gebaut** am selben Tag (`rust/drachenhauch_runtime/src/paket.rs`, Handbuch
[pakete.md](pakete.md), Tests `tests/pruef/paket.dhtest`). Beim Bauen kam
dazu bzw. wich ab:

- **Die Pruefsumme geht ueber den entpackten Inhalt**, nicht ueber die
  Bytes des ZIPs -- GitHub hat seine Archive 2023 neu gepackt, ohne dass
  sich der Inhalt aenderte, und ueberall schlugen Pruefsummen an.
- **`dhrt paket hole --erneuern`** nimmt neuen Inhalt an (statt entfernen
  und neu holen).
- **Ein lokaler Pfad in der `paket.json` eines Pakets gilt ab dem Ordner,
  aus dem das Paket kam** -- nicht ab der Kopie in `pakete/`. Ein
  heruntergeladenes Paket darf nicht auf lokale Pfade verweisen.
- In der Sperrdatei steht bei lokalen Quellen die Quelle selbst, kein
  absoluter Pfad -- die Datei gehoert ins Repository.

## Was es heute gibt

`IMPORT "datei.dh"` sucht neben der importierenden Datei, dann in jedem
Ordner aus `DH_PATH`, dann in `<Benutzerordner>/.drachenhauch/bibliothek`
(siehe [sprache.md](sprache.md#wo-gesucht-wird)). Wer eine Bibliothek teilen
will, schickt Dateien herum; wer sie benutzt, kopiert sie von Hand. Es gibt
keine Versionen, keinen Weg, eine Bibliothek zu holen, und nichts, was
festhaelt, welches Projekt welche Fassung braucht -- auf einem zweiten
Rechner fehlt dann etwas, und niemand weiss genau was.

## Ziel

```
dhrt paket hole github:hans/spielkiste@1.2
```

holt die Bibliothek, legt sie ins Projekt und merkt sie sich. Danach geht

```basic
IMPORT "spielkiste/vektor.dh"
```

und auf einem anderen Rechner holt `dhrt paket hole` (ohne Argument) alles,
was das Projekt braucht -- in genau derselben Fassung.

## Entscheidungen

### 1. Kein eigener Server

Ein Verzeichnis mit Anmeldung, Namensvergabe und Missbrauchsschutz waere ein
Betrieb, kein Feature. Pakete kommen von dort, wo sie ohnehin liegen:

| Quelle | Bedeutung |
|---|---|
| `github:nutzer/repo@stand` | das ZIP, das GitHub fuer einen Tag/Zweig/Commit ausliefert (`https://github.com/nutzer/repo/archive/<stand>.zip`) |
| `https://.../datei.zip` | ein beliebiges ZIP |
| `https://.../datei.dh` | eine einzelne Datei |
| `pfad/zum/ordner` oder `pfad/datei.zip` | etwas Lokales (eigene Bibliotheken, Tests, ohne Netz) |

`github:` ist nur eine Abkuerzung; GitLab & Co. gehen ueber die volle
Adresse. **Ohne `@stand` ist es ein Fehler** -- „der neueste Stand“ waere
morgen ein anderer, und genau das soll das Ganze verhindern.

### 2. Pakete gehoeren zum Projekt

Sie landen in `pakete/<name>/` neben der Datei `paket.json` des Projekts --
nicht im Benutzerordner. So hat jedes Projekt seine eigenen Fassungen, zwei
Projekte koennen verschiedene brauchen, und wer den Projektordner
weitergibt (oder einen Export baut), hat alles beisammen.

`<name>` ist der letzte Teil der Quelle (`spielkiste`), mit `--als name`
waehlbar.

### 3. Zwei Dateien: Wunsch und Stand

- **`paket.json`** -- was das Projekt braucht, von Hand lesbar und aenderbar:

  ```json
  {"name": "mein-spiel", "pakete": {"spielkiste": "github:hans/spielkiste@1.2"}}
  ```

- **`paket.lock.json`** -- was tatsaechlich geholt wurde: Quelle, die
  aufgeloeste Adresse und eine **SHA-256-Pruefsumme** des Inhalts. Holt ein
  zweiter Rechner das Paket und kommt etwas anderes an (ein Tag wurde
  verschoben, ein Server liefert etwas Fremdes), ist das ein Fehler statt
  eines stillen Unterschieds.

Beide gehoeren ins Versionsverwaltungssystem, `pakete/` kann man
hineinnehmen oder nicht (mit der Sperrdatei laesst es sich genau wiederherstellen).

### 4. IMPORT sucht in `pakete/`, von innen nach aussen

Ein neuer Schritt in der Suche, zwischen „neben der Datei“ und `DH_PATH`:
von der importierenden Datei aus jeder uebergeordnete Ordner, ob es dort
`pakete/<pfad>` gibt. Das ist dieselbe Regel wie bei `node_modules`, und
sie loest das Abhaengigkeitsproblem ohne Versionsrechnung:

- Das Projekt importiert „spielkiste/vektor.dh“ und bekommt die Datei aus
  dem Ordner spielkiste in seinem `pakete/`.
- Braucht `spielkiste` selbst ein Paket, steht es in
  `pakete/spielkiste/pakete/...` und wird von dort zuerst gefunden -- auch
  wenn das Projekt eine andere Fassung desselben Pakets hat.

Eingebaute Module bleiben eingebaut (`IMPORT "json"` wie heute).

### 5. Ein Paket ist nur Quelltext

Beim Holen wird **nichts ausgefuehrt** -- kein Installationsskript, kein
Bau. Entpackt wird mit den Schutzregeln des `zip`-Moduls (kein `..`, keine
absoluten Pfade). Ein Paket kann beim Laufen trotzdem alles tun, was ein
Drachenhauch-Programm darf; das sagt die Doku deutlich: man holt fremden
Code.

Liegt im ZIP alles in einem einzigen Oberordner (so liefert GitHub,
`spielkiste-1.2/`), wird dieser Ordner weggelassen.

### 6. Abhaengigkeiten eines Pakets

Hat ein Paket selbst eine `paket.json`, holt `dhrt paket hole` dessen
Pakete in `pakete/<name>/pakete/` mit. Kreise werden erkannt (eine Quelle,
die schon auf dem Weg liegt, ist ein Fehler).

## Befehle

| Befehl | Wirkung |
|---|---|
| `dhrt paket hole` | alles aus `paket.json` holen (Pruefsummen aus der Sperrdatei, wenn es sie gibt) |
| `dhrt paket hole <quelle> [--als name]` | eines dazunehmen, `paket.json` und Sperrdatei ergaenzen |
| `dhrt paket entferne <name>` | aus `paket.json`, Sperrdatei und `pakete/` nehmen |
| `dhrt paket liste` | was das Projekt hat, mit Quelle und Pruefsumme |
| `dhrt paket neu [name]` | eine `paket.json` anlegen |

Das Projekt ist der naechste Ordner mit `paket.json`, vom Arbeitsordner aus
nach oben gesucht; `hole` ohne `paket.json` legt sie im Arbeitsordner an.

## Was bewusst nicht dabei ist

- **Versionsbereiche** (`^1.2`, `>=1.0`): ohne Verzeichnis gibt es nichts,
  wogegen man rechnen koennte. Ein Stand ist ein Tag oder Commit.
- **Ein zentrales Verzeichnis zum Suchen**: spaeter allenfalls eine
  Liste in einem Repository (eine Datei mit Namen und Quellen), kein Server.
- **Signaturen**: die Pruefsumme schuetzt vor Veraenderung zwischen zwei
  Holvorgaengen, nicht vor einem boesartigen Original.

## Tests

- Holen aus einem lokalen Ordner, einem lokalen ZIP und ueber den
  Gegenserver (`tests/pruef/_hilfen/gegenserver.dh`) per HTTP.
- `IMPORT` findet ein Paket, ein Paket findet sein eigenes Paket zuerst,
  eine Datei neben dem Programm schlaegt ein Paket.
- Falsche Pruefsumme, Quelle ohne `@stand`, ZIP mit `..`, Kreis.
- Gegenprobe je Regel.

## Spaeter

- ~~In der IDE: Menue „Paket holen …“, Pakete im Projektbaum.~~ Gebaut am
  2026-10-05: Fenster „Pakete des Projekts“ (Strg+Alt+P), die Pakete stehen
  im Projektbaum unter `pakete/`, Umbauten ueber das Projekt lassen sie aus.
- `DH_PATH` und die Benutzerbibliothek bleiben wie sie sind -- fuer Dinge,
  die man ueberall will und nicht versioniert.
