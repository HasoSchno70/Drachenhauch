# Drachenhauch 2026.46

*Die Notizen zu dieser Fassung. Sie ist die Fassung **zum Aufräumen und
Weitergeben**: drei große Programme haben eigene Repos bekommen, das Repo hat
eine klare Ordnung, drei Sicherheitslücken in Abhängigkeiten sind behoben, und
ein Release entsteht jetzt aus einem einzigen Tag. Dazu ein Startbildschirm,
der zwischen IDE und Studio wählen lässt. Was bestehende Programme bemerken
können, steht gleich zuerst.*

## Was sich für bestehende Programme ändert

* **Glutfaden, Funkenflug und die Kühlschrank-App liegen nicht mehr in diesem
  Repo und nicht mehr im Installer.** Sie haben eigene Repos:
  [Glutfaden](https://github.com/HasoSchno70/Glutfaden) (Kreuzstich-Editor),
  [Funkenflug](https://github.com/HasoSchno70/Funkenflug) (Schaltplan-Editor) und
  [Kuehlschrank-Drachenhauch](https://github.com/HasoSchno70/Kuehlschrank-Drachenhauch)
  (Studio-Projekt), jeweils mit ihrer ganzen Geschichte und unter derselben
  Lizenz. Sie laufen mit dieser Fassung wie bisher.
* **Der Form-Designer (`examples/197_form_designer.dh`) ist entfernt**; das
  Drachenhauch Studio ersetzt ihn. Eine `.dhform` im Projektbaum der IDE geht
  jetzt im Studio auf, der Installer löscht die alte Verknüpfung.
* **Spiele und Bücher liegen in eigenen Ordnern:** `spiele/` (Pyramid Pusher,
  Circuit Runner, Voidreaver, Restaurant, Kaminfeuer) und `buecher/`
  (`referenz`, `einstieg`, `galaga`, `tippspiel` statt `buch-*`). Wer sie aus
  dem Quelltext startet, nimmt den neuen Pfad, etwa
  `dhrt run spiele/pyramid_pusher/pyramid_pusher.dh`. Die Vokabellisten des
  Einstiegsbuchs bleiben unter `buch-einstieg/vokabellisten/`, damit die
  Adresse in schon verteilten Büchern gültig bleibt.
* **Physik (`PHYS2D_*`, `PHYS3D_*`) rechnet mit Rapier 0.36 statt 0.22.**
  Bahnen können in den letzten Stellen anders ausfallen. Ein schlafend
  angelegter Stapel bleibt weiter ruhig bis zum ersten Anstoß, setzt sich aber
  im allerersten Schritt um gut einen Millimeter, wo er vorher genau
  stehen blieb.

## Neu in dieser Fassung

### IDE und Studio

* **Startbildschirm:** Nach dem Vorspann fragt die IDE, ob es *klassisch* in
  der IDE oder *modern* im Studio weitergeht (Maus, Pfeile, Enter, 1 und 2).
  „Nicht mehr fragen“ merkt die Wahl; zurück über die Einstellungen oder mit
  gehaltener Umschalttaste beim Start.
* **Aufgeklappte Menüs und Klapplisten liegen über allen Fenstern**, auch über
  einem Entwurfsfenster im Studio. Vorher lag das Menü „Ansicht“ darunter, und
  ein Klick auf das Verdeckte wählte trotzdem den Eintrag.
* **`PROCESS_START_DETACHED(programm$, ...)`** startet ein Programm, das
  weiterläuft, wenn der Aufrufer endet (ein Kind aus `PROCESS_START` endet mit
  ihm). Die IDE startet damit das Studio.

### Sicherheit

* **Drei Lücken in Abhängigkeiten behoben**, gefunden mit dem neu
  eingerichteten `cargo audit`:
  * das eingebaute **SQLite** (`libsqlite3-sys`, RUSTSEC-2026-0459) -- über
    `rusqlite` 0.41,
  * die mitgelieferten **Stammzertifikate** (`webpki-roots`, RUSTSEC-2026-0381)
    vertrauten Zwischenzertifikaten, denen die Browser inzwischen misstrauen,
  * `ctutils` (RUSTSEC-2026-0385), Vergleiche nicht in konstanter Zeit.
* **Sicherheitslücken lassen sich privat melden**: über „Report a
  vulnerability“ im Reiter *Security* des Repos (siehe `SECURITY.md`).

### Herunterladen und Mitmachen

* **Jedes Release bringt eine `SHA256SUMS.txt`** mit den Prüfsummen aller
  Dateien und die drei großen Bücher (Lehrbuch, Handbook, Einstieg als
  `.docx` und `.epub`) als eigene Downloads.
* `CONTRIBUTING.md`, `SECURITY.md`, ein Verhaltenskodex und Formulare für
  Fehlerberichte und Wünsche; die README zeigt ein Hallo-Welt ohne
  Zusatzdateien, die Bauanleitung für Windows, macOS und Linux und alle vier
  Bücher. Herkunft und Lizenz von Vorspann, Logo und Schriften stehen in
  `daten/HERKUNFT.md`.

## Unter der Haube

**Zahlen.** 2433 Befehle (vorher 2432) in 57 Modulen, 233 Beispiele auf der
obersten Ebene von `examples/` (234; der Form-Designer ist weg),
**5849 Fälle in 480 Prüfsammlungen** (6068 in 518 -- die 40 Sammlungen von
Glutfaden und der Kühlschrank-App sind mit ihnen umgezogen) und 670
Rust-Testfunktionen (670).

* **Ein Release entsteht aus einem Tag:** `.github/workflows/release.yml` baut
  den Windows-Installer, die Pakete für macOS und Linux und die Prüfsummen und
  legt einen Release-Entwurf an. Bisher entstand der Installer nur auf dem
  Rechner des Autors.
* **Abhängigkeiten aktuell**: Rapier 0.36, die Hash-Crates gemeinsam auf
  `digest` 0.11 (`sha2`, `sha1`, `md-5`, `hmac`), `gif` 0.14, `libloading` 0.9,
  `tokio-postgres-rustls` 0.14, `webpki-roots` 1.0 und 16 kleinere.
  Dependabot schlägt Updates jetzt monatlich gebündelt vor, `cargo audit`
  prüft jeden Montag.
* **Die Website samt Spielwiese baut neu**, wenn sich die Laufzeit ändert --
  vorher nur bei Änderungen an `Cargo.toml`.
* **`docs/` ist nur noch das Handbuch**: Entwürfe und Roadmaps liegen in
  `docs/intern/`, die Release-Notizen in `docs/releases/`.
* Aufgeräumt: eine private Verkaufs-Checkliste, liegengebliebene Spielstände,
  ein doppeltes 4,9-MB-Bild, die letzten `gb`-Namen und Python-Spuren.

## Was offen bleibt

* **Echte Geräte sind weitgehend ungeprüft** (Raspberry Pi, SPI, Kamera,
  Mikrofon, Arduino, Modbus) -- Rückmeldungen mit Hardware sind willkommen.
* **Volle Textformung** (Ligaturen anderer Schriften, indische Schriften)
  kann die gui noch nicht.
* **raylib 6.2** ist erschienen, raylib-rs hat aber noch keine Fassung dafür.
* Das macOS-Paket ist nicht beglaubigt, der Windows-Installer nicht signiert
  (SmartScreen warnt beim Herunterladen).
* Die fertigen Bücher und die Vorspann-Videos liegen noch im Repo (rund
  40 MB); sie sollen künftig nur noch als Downloads am Release hängen.
