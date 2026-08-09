# Umbenennung: GameBasic → Drachenhauch

**Anlass:** „GameBasic" ist im Netz mehrfach belegt. „Drachenhauch" wurde am
2026-08-09 geprüft und war auf **jeder** Achse frei: DPMA (national, EU,
international — mit Kontrollsuche verifiziert), PyPI, crates.io, npm,
GitHub-Konto, null Repos, alle Domains (`.de .com .dev .io .org .net`), keine
Firma, kein Softwareprojekt, keine Programmiersprache.

**Der Zeitpunkt ist der beste, den es je geben wird.** Das erste Release
(`v2026.1`) ist einen Tag alt und hat null Downloads — es existiert draußen
keine einzige fremde `.gb`-Datei. Jeder Monat, den man wartet, macht die
Sache teurer.

---

## Was dranhängt

| | heute | Vorschlag |
|---|---|---|
| Produktname | GameBasic | **Drachenhauch** |
| Kurzform im Alltag | — | **Drache** (wie „Postgres" für PostgreSQL) |
| Python-Paket | `gamebasic` | `drachenhauch` |
| Rust-Crate | `gb_runtime` | `drachenhauch_runtime` |
| Runtime-Binary | `gbrt` | `dhrt` |
| Startbefehle | `gb*.cmd` (13) | `dh*.cmd` |
| Quelldateien | `.gb` (232) | `.dh` |
| Bytecode | `.gbc` | `.dhc` |
| Editor-Formate | `.gbsprite` (15), `.gbanim` (3), `.gbform` (1), `.gbproj` | `.dhsprite`, `.dhanim`, `.dhform`, `.dhproj` |
| VSCode-Extension | `vscode-gamebasic` | `vscode-drachenhauch` |

### Warum die Endungen mit müssen

`.gb` ist die Endung für **Game-Boy-ROMs**, `.gbc` die für **Game-Boy-Color-ROMs**.
Für eine Spiele-Programmiersprache ist das die denkbar unglücklichste Kollision:
Emulatoren streiten sich um die Dateiverknüpfung, und eine Suche nach „gb file
game" landet bei Nintendo. Das ist unabhängig vom Namen ein Fehler und gehört
in einem Zug mit repariert.

---

## Reihenfolge

Die Phasen sind so geschnitten, dass nach **jeder** die Testsuite grün sein
muss. Wer mittendrin abbricht, hinterlässt kein kaputtes Projekt.

### Phase 1 — Runtime und Endungen (das Fundament)

- [x] `rust/gb_runtime/` → `rust/drachenhauch_runtime/`, Crate- und Binary-Namen
- [~] Endungen **nach Phase 3 verschoben** — siehe Anmerkung unten
- [x] `PAYLOAD_MAGIC` `GBRTPAY1` → `DHRTPAY1` mitgezogen.
      Alte exportierte Spiele laufen dann nicht mehr mit der neuen Runtime;
      das ist bei null Downloads folgenlos und später nie wieder so billig.
- [x] `cargo test` + volle Suite grün

> **Plankorrektur (2026-08-09).** Die Endungsänderung stand ursprünglich in
> Phase 1. Sie gehört aber in Phase 3, zusammen mit dem Umbenennen der 232
> Dateien: Brächte man der Runtime die neuen Endungen bei, während die Dateien
> noch `.gb` heißen, wären alle Beispiele und das ganze Buch sofort kaputt —
> genau das, was die Regel „nach jeder Phase muss die Suite grün sein"
> verhindern soll. Endung und Dateinamen sind EIN Schritt.

### Phase 2 — Python-Schicht

- [x] `gamebasic/` → `drachenhauch/`, `pyproject.toml` (`name`, Skripte)
- [x] `gbrun.py` → `dhrun.py`, die 13 `gb*.cmd` (+ `gb.sh`) → `dh*`
- [x] Bezeichner: `GameBasicError` → `DrachenhauchError`, `GBRuntimeError` →
      `DHRuntimeError`, `GBHighlighter`, `gb_code`, `gb_symbols`, `gb_path`
- [x] `editor_qt/`: `builtin_index.json`-Pfad überlebt den Paketumzug
      (1298 Builtins geladen — das war das echte Risiko daran)
- [~] Fenstertitel und Dateidialoge **nach Phase 3/4 verschoben**: die Titel
      tragen den Produktnamen, die Dialoge die Dateiendung. Beides jetzt zu
      ändern hieße, die Anwendung „Drachenhauch" nennen zu lassen, während
      README, Buch und Doku noch durchgehend „GameBasic" sagen.
- [x] `pytest tests/` grün (3104), mypy sauber, `dhrun.py` führt aus

### Phase 3 — Endungen und Inhalte

- [x] **3a** Runtime versteht `.dh`/`.dhc` — *zuerst*, damit die Suite während
      des Umbaus lauffähig bleibt. `.gb`/`.gbc` bleiben lesbar (je eine Zeile).
- [x] **3b** 231 `.gb` → `.dh` per `git mv`, 1227 Verweise nachgezogen
- [x] **3c** Editor-Formate `.gbsprite`/`.gbanim`/`.gbform`/`.gbproj` → `.dh*`
      (19 Dateien, 346 Verweise). Öffnen-Dialoge zeigen die alte Endung weiter.
- [x] **3d** Buch: alle 784 Codeblöcke durch `dhrt --check`, 0 Befunde.
      Der Prüfer liegt jetzt als `buch-referenz/buch/pruef_codebloecke.js` im
      Repo statt als Einzeiler pro Sitzung.
- [x] `.docx` und `.epub` neu gebaut (der Buchtext hatte sich geändert)
- [x] 3106 Tests grün, mypy sauber, 65 Rust-Tests, Runtime gebaut

> **Die Massenersetzung frisst ihre eigenen Ausnahmen.** `.gb` → `.dh` lief
> auch über die Duldung, die eine Stunde vorher für genau diese alte Endung
> eingebaut worden war — aus `ends_with(".gb")` wurde `.dh`, die Prüfung
> lautete `.dh || .dh`. Aufgefallen ist es erst beim Bauen, weil danach nur
> pytest lief. Zwei Lehren: die alte Endung steht jetzt als Konstante statt
> als Literal, und **nach jeder Phase wird gebaut, nicht nur getestet**.

### Phase 4 — Außenwirkung

- [x] `README.md` + `README.en.md`, `docs/`, `CLAUDE.md` — 655 Stellen in 256
      Dateien (`GameBasic` 601×, `GAMEBASIC` 54×)
- [x] Schriftzug (`drachenhauch/assets/schriftzug.png`) als README-Kopf; er lag
      bisher ungenutzt herum. Vorher geprüft, dass er auf **beiden**
      GitHub-Themen trägt: dunkle Kontur auf Weiß, helle Glanzlichter auf Dunkel
- [x] Slogan **„Feuer frei für deine Ideen."** (englisch: *Let your ideas
      breathe fire.* — „fire at will" trüge einen Kasernenton mit, den das
      deutsche „Feuer frei" hier nicht hat)
- [x] Buchausgaben umbenannt und neu gebaut (`Drachenhauch-Lehrbuch.docx/.epub`,
      `Drachenhauch-Buch.docx`)
- [x] GitHub-Repo `Game-Basic` → `Drachenhauch`, Beschreibung neu (der Tippfehler
      „Entwicklungs umgebung" ist mit weg). Alte URLs leiten dauerhaft um
- [x] `v2026.1` unangetastet: der letzte Stand unter dem alten Namen

> **Der Produktname steckte in Testdaten.** `LEFT$("GameBasic", 4)` war „Game" —
> die Eingabe wanderte mit, die erwarteten Ausgaben nicht (3 Tests). Jetzt
> „TestString" → Test + String: gleiche Lesbarkeit, aber der nächste
> Namenswechsel bricht sie nicht wieder.
>
> **`DRACHENHAUCH` ist drei Zeichen länger als `GAMEBASIC`.** In
> `examples/49_pong_scene` lief der Titel damit rechts aus dem Bild. Sichtbar
> war das **nur im Screenshot** — kein Test und kein `--check` meldet so etwas.
> Beim Zentrieren kam heraus, dass die Zeile darunter schon vorher zu lang war.

### Phase 5 — Auslieferung

- [ ] `installer/GameBasic.iss` → `Drachenhauch.iss`, Anzeigenamen, Startmenü
- [ ] **`AppId` NICHT ändern.** Sie bleibt die Identität der Installation; mit
      derselben Id ersetzt der neue Installer die alte Fassung sauber, statt
      zwei Einträge in „Programme entfernen" zu hinterlassen.
- [ ] **Alten Beispielordner aktiv entfernen.** Wir haben am 2026-08-09 gelernt:
      `uninsneveruninstall` lässt einen umbenannten Ordner für immer liegen. Der
      Juni-Installer hinterließ so `%PUBLIC%\Documents\GameBasic\Beispiele` —
      225 verwaiste Dateien. Also im `[InstallDelete]`-Abschnitt aufräumen.
- [ ] Neuer Bau, Installation geprüft, Beispiel gestartet
- [ ] Release `v2026.2` unter dem neuen Namen

---

## Fallstricke

**Der Suchen-und-Ersetzen-Reflex ist gefährlich.** „gb" steckt in Bezeichnern,
die nichts mit uns zu tun haben. Gezählt im eigenen Quelltext:

| Vorkommen | davon unsere? |
|---|---|
| `rgba` (92), `rgb` (61) | **nein** — Farbwerte |
| `qdialogbuttonbox` (56) | **nein** — eine Qt-Klasse |
| `gbool` (17) | **nein** — ein Typname |
| `gbrt` (252), `gbrun` (46), `gbc` (43), `gbanim` (38), `gbform` (35), `gbsprite` (20) | ja |

> **Diese Datei von jeder Massenersetzung ausnehmen.** Sie beschreibt den
> Zustand VOR der Umbenennung — ein Lauf über sie macht aus „`gbrt` → `dhrt`"
> die Aussage „`dhrt` → `dhrt`" und löscht damit genau die Angabe, die man
> später zum Nachvollziehen braucht. In Phase 1 und 2 ist das je einmal
> passiert und musste von Hand zurückgenommen werden.

`QDialogButtonBox` ist der lehrreichste Fall: Ein naives Ersetzen von „gb"
zerlegt einen Qt-Aufruf, und der Fehler zeigt sich erst, wenn jemand den
Dialog öffnet — nicht beim Übersetzen. **Nur ganze Bezeichner ersetzen, nie
Teilzeichenketten**, und `rgb`/`rgba`/`QDialogButtonBox`/`gbool` vorher
ausklammern.

**Die Dateiendung steckt an 46 Stellen im Code.** Nicht nur im Preprocessor:
auch im Editor (Dateidialoge, Syntaxhervorhebung), im LSP, in der
VSCode-Grammatik und in der Windows-Registrierung des Installers.

**Der Kontaktbogen der Doku.** `docs/` und das Buch enthalten Hunderte
Verweise auf `examples/NN_name.gb`. Nach dem Umbenennen zeigen die alle ins
Leere — das Prüfskript für README-Verweise
(`pruef_readme.py`-Muster) darauf ansetzen.

**Der Installer erinnert sich an den Ordner.** Bei gleicher `AppId` installiert
Inno in das zuvor benutzte Verzeichnis, also weiterhin `C:\Program
Files\GameBasic`. Wer den Ordner umbenannt haben will, muss den alten explizit
entfernen und die Verknüpfungen neu setzen.

---

## Aufwand

435 Dateien enthalten „GameBasic", 1563 Treffer. Davon sind rund 90 % rein
mechanisch. Der echte Aufwand steckt in:

1. den 46 Code-Stellen mit fest verdrahteter Endung,
2. dem Buch (75 Kapitel, jeder Codeblock will nachgeprüft werden),
3. dem Installer samt Aufräumen der alten Installation.

Realistisch: ein Arbeitstag mit sorgfältiger Prüfung nach jeder Phase.
