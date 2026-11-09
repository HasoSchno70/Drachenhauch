# Drachenhauch

BASIC-Dialekt mit Pascal-strikter Typisierung und OOP -- für Spiele UND für
Anwendungen mit GUI (Formulare, Tabellen, Datenbank, Drucken). Früher hieß er
„GameBasic“; das Kürzel ist überall `dh` (`.dh`, `dhrt`, `DH_*`, „DH-Code“),
nicht mehr `gb` -- Ausnahmen sind nur Geschichtsdokumente und die Aufräum-
Regeln für alte GameBasic-Installationen.
**Eine Runtime: `dhrt`** (Rust/raylib) — sie ist Lexer → Parser → Compiler → VM
in einem und übernimmt Ausführung, Konsole, Grafik/Audio, Standalone-Export,
Sprachserver (`dhrt lsp`), Prüfsammlungen (`dhrt test`) und Doku-Werkzeuge.
Die IDE (`ide/ide.dh`), das Studio (`studio/`) und alle Werkzeuge
(`examples/183..199_*.dh`) sind Drachenhauch-Programme. **Python gibt es nur
noch in zwei Bauskripten** (`rust/build_runtime.py`, `rust/build_wasm.py`, nur
Standardbibliothek -- `tests/pruef/werkzeugkette/bauskripte.dhtest` hält die
Liste der erlaubten Importe fest).

Diese Datei sagt, **was heute gilt**. Wie es dazu kam -- jede Ausbaustufe,
jeder Fund, jede Falle mit Datum -- steht unverändert in `geschichte/`
(Übersicht ganz unten). Wer an einem Bereich arbeitet, liest dort den
passenden Abschnitt; wer etwas Neues baut, schreibt das Wichtigste als Regel
hierher und die Geschichte dorthin.

## Verzeichnisstruktur

```
rust/drachenhauch_runtime/   # die Runtime dhrt (Rust/raylib/Kira)
  src/lexer.rs parser.rs compiler.rs vm.rs builtins.rs + <modul>.rs
  src/jit.rs                 # Maschinencode (Cranelift)
  src/lsp.rs symbole.rs doku.rs pruef.rs pruefsammlung.rs   # Werkzeuge in dhrt
rust/build_runtime.py        # Bau der Runtime (Python 3, nur Standardbibliothek)
rust/build_wasm.py           # Bau fuer den Browser (dito)
daten/                       # builtin_index/docs/prosa.json, meldungen.en.txt (eingebettet), bilder/, schriften/
ide/ide.dh                   # die IDE, in Drachenhauch: Zustand, Fenster, Hauptschleife
  teile/*.dh                 # ihre Unterprogramme nach Themen (per IMPORT an Ort und Stelle)
  sprache/en.txt             # englische Oberflaeche
studio/                      # Drachenhauch Studio (Xojo-artige IDE)
examples/*.dh                # Demos; 183..199 die Werkzeuge; Unterordner = groessere Programme
installer/                   # bauen.dh, paket.dh, Drachenhauch-IDE.iss, lizenzen.dh, posix/, symbole/
tests/pruef/<thema>/*.dhtest # die Pruefsammlungen (Ordner in tests/pruef/LIESMICH.md), _hilfen/ gemeinsam
tools/*.dh, tools/*.js       # Website, Buch-Bau, Showcase-Bilder, Buch-Exporte (Node)
docs/ und docs/en/           # Handbuch deutsch + englisch (die IDE liest es fuer F1, die Website auch)
web/                         # Web-Playground (dhrt als WASM), auf der Website als spielwiese/
vscode-drachenhauch/         # VS-Code-Erweiterung (startet dhrt lsp)
geschichte/                  # Entwicklungsgeschichte (frueher in dieser Datei)
```

## Architektur-Pipeline

```
Quelle.dh -> preprocess (IMPORT) -> Lexer -> Parser -> AST -> Compiler -> Bytecode -> VM (+ Maschinencode)
             alles in dhrt: preprocess.rs, lexer.rs, parser.rs, compiler.rs, vm.rs, jit.rs
```

`dhrt run datei.dh` wechselt vor dem Lauf ins Verzeichnis der Quelle (relative
Asset- und IMPORT-Pfade) und hinterlegt den Ort des Aufrufers als
`DHRT_START_DIR` (im Programm `STARTDIR$()`, Argumente als Pfad `ARGPATH$(n)`).
`dhrt --export datei.dh [ziel]` hängt den Bytecode an eine Kopie der eigenen
Exe und kopiert `assets/`. Weitere Einstiege: `dhrt --check`, `dhrt lsp`,
`dhrt debug`, `dhrt profile`, `dhrt test`, `dhrt bild`, `dhrt fmt`,
`dhrt paket`, `dhrt doku`, `dhrt pruef`, `dhrt call`.

## Built-ins erweitern

Builtins leben in `rust/drachenhauch_runtime/src/builtins.rs` (pure) bzw. in den
`try_*`-Familien von `vm.rs` (brauchen VM-/Fenster-Zustand). Neuer Builtin:

1. Einen Match-Arm ergänzen: Arity + Typen selbst prüfen (Validierung gehört in
   den Wrapper, nicht ins Backend), Fehlermeldung im gewohnten Wortlaut
   (`"NAME: erwartet …"`, deutsch -- siehe Meldungen).
2. `daten/builtin_index.json` ergänzen (Name/kind/Signatur/Modul) --
   Vervollständigung, Hover und `--check` ziehen daraus. **Die Signatur muss
   stimmen**, der Compiler leitet die erlaubte Argumentzahl daraus ab. Formen:
   `NAME(a, b [, c])`, `NAME(a, b = "")`, `NAME(a, b, ...)`,
   `NAME(6..8 Argumente)`; `NAME(*args)` schaltet die Prüfung ab (nur wenn es
   wirklich offen ist). Parameter mit Namen, nicht "N..M Argumente".
3. Die Kurzbeschreibung kommt als Tabellenzeile ``| `NAME(args)` | was es tut |``
   ins passende `docs/module-*.md` UND `docs/en/module-*.md`; `dhrt doku prosa`
   sammelt sie nach `daten/builtin_prosa.json` (und `.en.json`).
   `builtin_docs.json` ist für ausführlichere Texte und gewinnt, wo es einen
   Eintrag hat. Danach dhrt neu bauen (eingebettet).
4. Ein Fall in einer Prüfsammlung (`tests/pruef/<thema>/*.dhtest`), mit
   Gegenprobe (siehe Prüfsammlungen).
5. Neues Keyword: `lexer::KEYWORDS` ergänzen, `dhrt doku grammatik`.
6. **Familien:** jede Aufrufstelle merkt sich die Familie, die geantwortet hat
   (`Instr::familie`, `Vm::builtin_rufen`/`familie_rufen`). Eine Familie darf
   nur am NAMEN absagen; sagt sie je nach Argument ab, obwohl eine spätere den
   Namen kennt, muss der Name vom Merken ausgenommen werden (wie `SORT`). Neue
   Familie = neue Nummer in `familie_rufen` UND `BUILTIN_FAMILIEN` anpassen.
7. **Ruft der Befehl Drachenhauch-Code** (Rückrufe, FUNCREF), gehört er in
   `jit::befehl_im_bereich` (verboten) und in die Prüfung `gerufenes_harmlos`
   -- sonst sieht der Rückruf Locals/Globale, die ein Maschinencode-Bereich
   gerade hält.
8. Liefert er IMMER denselben Typ, darf er in `typen::builtin_typ` -- nur
   nachgemessen, jeder Weg ("endet auf `$` = Text" ist falsch: `SPLIT$`).

## Neues Modul, IMPORT-Auflösung

Module sind in `rust/drachenhauch_runtime/src/<modul>.rs` implementiert, Dispatch
in `vm.rs` (`try_<modul>`), externe Typen + ihr Default in `vm.rs`/`value.rs`.
Neues Modul: `.rs` schreiben, im Dispatch einhängen, den Namen in
`preprocess.rs` MODULES ergänzen (sonst kennt der Preprocessor `IMPORT "modul"`
nicht), Fall in einer Prüfsammlung, `builtin_index.json`, `docs/module-*.md`
deutsch und englisch. Rechenkerne als eigene Module mit Rust-`#[test]`s, ohne
Fenster pruefbar (Beispiele: `farbraum.rs`, `kalender.rs`, `leinwand.rs`,
`schnitt.rs`, `midifile.rs`).

**IMPORT** (`preprocess.rs`) löst den Pfad **wörtlich** auf, ohne `.dh`
anzuhängen: (1) gibt es den Pfad als Datei -> textuell einfügen
(`IMPORT "helfer.dh"`), (2) sonst ein eingebautes Modul -> Zeile wird
Kommentar, (3) sonst `pakete/` von der importierenden Datei aufwärts
(`dhrt paket`), (4) sonst Fehler. `IMPORT "json"` nimmt also IMMER das
eingebaute Modul. `IMPORT "x" AS name` = Namensraum (`namensraum.rs`),
`IMPORT "serial" OPTIONAL` + `MODULE_AVAILABLE(...)` für Module, die nicht in
jedem Bau stecken. Fehlerzeilen nennen `datei:zeile` der eingebundenen Datei.

## Module im Überblick

Je Modul ein Handbuch `docs/module-<name>.md` (englisch in `docs/en/`). Die
alte, sehr ausführliche Tabelle samt aller Einzelheiten steht in
`geschichte/module.md`.

| Bereich | Module |
|---|---|
| Daten | `json`, `xml`, `ini`, `csv`, `xlsx`, `zip`, `db` (SQLite, PostgreSQL, MySQL), `save`, `zeit`, `geld`, `regex`, `krypto`, `pdf` (krilla, Drucken) |
| Spiel | `sprite`, `animfsm`, `particles`, `tiled`, `tile_collide`, `controller`, `camera`, `scene`, `astar`, `ecs`, `timer`, `tween`, `curves`, `input` |
| Grafik | Kern-Befehle (2D, Layer, Atlas, Render-Targets, Shader/POSTFX, Bilder), `g3d` + `m3d` (3D, PBR, Schatten, Instancing), `imgfx`, `chart`, `video` |
| Physik | `physics` (Kollisions-Mathe), `physics2d`/`physics3d` (Rapier) |
| Ton | `audio` (Kira: Busse, Effekte, Uhr, Sampler, MOD/XM, räumlich, Modulatoren, `AUDIO_NOTE`, Mischen), `SPEAK`, Mikrofon (`MIC_*`), `midi` (+ `.mid`-Dateien) |
| Oberfläche | `gui` (Retained-Mode: Fenster, alle Widgets, Dock, Stildatei, Dialoge, Tabellen, Bindung, Barrierefreiheit), `ui` (Immediate-Mode) |
| Netz | `net`, `html` (HTTP), `httpd`, `websocket`, `mqtt`, `smtp`, `osc`, `mdns`, `cloud` |
| Geräte | `serial`, `firmata`, `modbus`, `usb`, `bt`, `wifi`, `gpio`/`i2c`/`spi` (Linux), `webcam`, `hotplug` |
| System | Prozesse (`PROCESS_*`, `TASK_START`), Fenster als Prozess (`WINDOW_OPEN`), Tray, Zwischenablage, Dateisystem, FFI (`DECLARE … LIB`) |

Externe Typen registrieren sich lowercase, DH-Code schreibt sie in jeder
Schreibweise (`DIM j AS JSON_HANDLE`). Mathe-Typen starten neutral (`MAT4` =
Einheitsmatrix, `VEC2` = Nullvektor; `model::neutrales_element`).

## Sprache: was man wissen muss

- **Namen ohne Groß/klein.** Eine lokale `n` verdeckt `CONST N`, ebenso ein
  Parameter oder ein Feld `voll` die Konstante `VOLL`. `--check` warnt (bei
  Parametern und Feldern nur, wenn der Name im Block auch anders geschrieben
  vorkommt, `lsp::verdeckung_filtern`); Strg+. benennt die Variable um. Eine Methode
  `zeigen` ersetzt `Zeigen` der Elternklasse (`--check` warnt bei anderer
  Schreibweise). Vorbelegte Namen (`red`, `pi`,
  `KEY_*`) dürfen als Variable verschattet werden; eine Variable darf heißen wie
  ein Builtin, `LEN(s)` meint dann trotzdem den Builtin.
- **`MID$`/`INSTR`/`INSTRREV` zählen ab 0**, nicht gefunden = -1.
- **`/` liefert immer FLOAT**, `\` rundet nach unten wie FLOOR (auch negativ),
  `MOD` passt dazu. Ein Index aus `/` ist ein Fehler.
- **Escape-Folgen nur in `!"..."`** (`\n \t \r \\ \" \0 \e \uXXXX`); in einer
  normalen Zeichenkette bleibt `\` wörtlich (Regex, Windows-Pfade).
- **`x IS Typ`** prüft den Typ (samt Vererbung), "dasselbe Objekt" ist `=`.
- **Ein `DIM` in einer Schleife fängt jede Runde frisch an.**
- `DIM a AS ARRAY OF T` ist ein LEERES Feld; Felder wachsen mit `ARRAY_PUSH`.
  `DIM f[1 TO 31]` = eigene Untergrenze (`LBOUND`/`UBOUND`); **wer in einem
  Feldbefehl einen Index nimmt oder liefert, rechnet über `unten_von(k)`**.
- `BAND`/`BOR` binden schwächer als `+`/`-`; `step` ist Schlüsselwort;
  `swap`/`let`/`exit`/`do`/`each`/`iif` sind kontextuell.
- Lambdas `FUNCTION(x) x * 2` kopieren Locals beim Anlegen, Globale sehen sie
  lebendig. `obj.methode` ohne Klammern ist eine gebundene Methode (FUNCREF).
- BYREF ist Copy-In/Copy-Out; Methoden mit BYREF brauchen eine bekannte Klasse.
- Für Umsteiger kennt `umstieg.rs` die Formen anderer BASICs und sagt, wie es
  hier heißt (`docs/umstieg.md`, `docs/stolpersteine.md`).
- Fehlertexte: Laufzeitfehler nennen `datei:zeile`; ein unbekannter Name
  schlägt den ähnlichsten vor (`aehnlich.rs`).

## Regeln je Bereich

Das sind die Stellen, an denen schon einmal etwas still kaputt war. Die
Einzelheiten stehen in der Geschichte des Bereichs.

**VM und Compiler**
- Seltene Befehle gehören in `Vm::dispatch_selten`; wer `dispatch` vergrößert,
  misst den Stapelrahmen nach (unter 4096 Byte halten, sonst `__chkstk` bei
  jedem Aufruf). `builtin_rufen` nicht `inline(never)` setzen.
- `model::verschmelzen` (Superinstruktionen): der schnelle Weg darf nichts
  verändern, bevor feststeht, dass er gelingt, und nie selbst einen Fehler
  erzeugen.
- Konstanten falten und die Befehle rechnen mit denselben Funktionen
  (`vm::konstant_rechnen`) -- nie eine zweite Rechnung.
- Beide Wege zum Programm (direkt und über .dhc) gehen durch
  `model::func_bauen`/`programm_bauen`: Nachgetragenes nur dort.
- Objektfelder über `feld_platz`; eine PROPERTY darf NIE in einen Merkplatz.
- Wer `typ_von` ändert, lässt `dhrt test tests/pruef` mit
  `DHRT_TYPEN_PRUEFEN=1` laufen.

**Maschinencode (`jit.rs`, an per Vorgabe, `DHRT_JIT=aus` = nur VM)**
- Wer eine Regel der VM ändert (`div`, `modulo`, `int_div`, `cmp`, `coerce`,
  `value_eq`, einen der `zahl_befehl`-Befehle), zieht `rechnen`/`vergleichen`/
  `wandeln`/`zahl_rechnen` mit -- `tests/pruef/laufzeit/jit.dhtest` fängt es.
- Bei jedem Fehlerfall steigt der Maschinencode aus und die VM rechnet nach;
  `bau.punkt` muss vor jeder Prüfung eines neuen Befehls gültig sein.
- Ein Fall, der die VM allein meint, setzt `DHRT_JIT=aus` in `--- umgebung`;
  die CI fährt beide Wege.
- Gemessen wird mit `dhrt run bench_dhrt.dh` mit UND ohne `DHRT_JIT=aus`,
  nicht an gezählten Runden (`DHRT_JIT_BILANZ=1` zeigt nur, wo die VM rechnet).

**Grafik und Eingabe**
- Jede Tastenabfrage in `graphics.rs` geht über `t_unten/t_neu/t_los` bzw.
  `mouse_button/mouse_hit/mouse_released` (verschluckte Klicks, `flanken.rs`),
  nie direkt an `self.rl.is_*`.
- `RaylibHandle` ist das LETZTE Feld von `Graphics` (sonst stürzt Mesa beim
  Beenden ab). Render-Ziele legt `render_ziel` selbst an (Linux/GLES).
- Farbe: Deckkraft 0 im obersten Byte heißt DECKEND; `RGBA(..., 255)` ist
  deckend.
- Kamera: nach `CAMERA_SET` sind alle Zeichenkoordinaten Welt-Koordinaten;
  `TEXT` wird nur verschoben, für scharfen HUD-Text vorher `CAMERA_RESET()`.

**gui**
- Geometrie je Widget aus EINER Quelle, die Zeichnen, Treffertest, Tastatur
  und Bildschirmleser gemeinsam fragen (`table_geom`, `ta_rows`, `tb_geom`,
  `list_bar_geom` …). Eine zweite, nachgeführte Lage läuft auseinander.
- Nach außen ist alles LOGISCH (Maßstab über `unsk`), nur `GUI_HIT_TEST`
  spricht Bildschirmpixel.
- Behälter (Layout, rollendes Panel, Reiterwerk, Akkordeon, Assistent) sind
  Luft für Klicks ihrer Kinder (`luft_fuer_kinder`).
- Eine Taste, die ein Widget, ein Menü oder ein Kürzel genommen hat, ist
  verbraucht (`liste_taste`, `kuerzel_gefeuert`, `widget_nimmt_taste`) -- sonst
  drückt ESC zusätzlich den Abbrechen-Knopf.
- Ein Fenster über einer Zeichenfläche braucht `GUI_DRAW_WINDOW` (Tooltips und
  Kontextmenüs: `GUI_DRAW_TOP`).
- Was die gui in die `.dhform` schreibt, liest sie auch zurück (Test als
  Rundweg); Handles (Texturen) stehen nicht darin.
- Ereignisse (`GUI_CLICKED`, `GUI_CHANGED`, Antworten) gelten genau ein Bild;
  vom Programm gesetzte Werte sind keine Bedienung.
- Breiten von Beschriftungen (Reiterköpfe, Werkzeugleiste) misst
  `leisten_pass` je Bild; eine Schätzung an der Zeichenzahl gilt nur bis
  dahin. Wer eine Lage von außen braucht, fragt die Laufzeit
  (`GUI_TABCONTROL_ITEM_RECT` & Co.), statt sie nachzurechnen.

**Meldungen, Doku, Website**
- Meldungen deutsch schreiben. Wer eine Meldung ändert, die in
  `daten/meldungen.en.txt` steht, zieht die Vorlage mit (`dhrt pruef meldungen`);
  setzt sie sich aus mehreren Literalen zusammen, markiert `[[wort]]`/`[[]]`
  die Naht. Platzhalter im Englischen nur `{}` und `{Zahl}`.
- Wer `docs/<datei>.md` ändert, zieht `docs/en/<datei>.md` mit
  (`dhrt pruef uebersetzung`: Überschriften, Tabellenzeilen, Codeblöcke).
- `dhrt pruef` prüft Codeblöcke, Namen, Pfade und Sprungmarken in `docs/` und
  dieser Datei -- ein Pfad, den es nicht mehr gibt, ist ein Befund.
- Die Website baut `tools/website.dh` (`.github/workflows/website.yml`), samt
  Spielwiese aus `rust/build_wasm.py`; der Web-Bau läuft dort auch für jeden
  PR, der `web/`, die Website oder `Cargo.toml` anfasst.

**Prüfsammlungen (`dhrt test`)**
- Format in `docs/werkzeuge.md`: `=== Fall`, Quelltext, `--- erwartet` /
  `enthaelt` / `fehler` / `bild` / `ton` / `inhalt` / `datei` / `umgebung` /
  `argumente` / `eingabe` / `programm … nach MARKE` / `einschub` / `ersetzen` /
  `vorher` / `zwischen` / `nochmal` / `nachher` / `seriell` / `langsam` /
  `zeitgrenze` / `system`. Platzhalter `{sammlung}`, `{fall}`.
- **Jede neue Prüfung bekommt eine Gegenprobe**: die Stelle verfälschen, bauen,
  zusehen, dass genau dieser Fall fällt, zurücksetzen. Ein grüner Fall, der
  auch ohne die Änderung grün wäre, ist wertlos -- das ist hier oft passiert.
- `--- seriell` nur für echte Einzelstücke (fester Port, Soundkarte, Drucker,
  echte Zwischenablage, Fensternachrichten, MIDI-Port). Jedes Kind bekommt
  `DHRT_ZWISCHENABLAGE=privat` und `DHRT_OHNE_EINGABE=1`.
- Aufnahmen (`AUTOMATION_PLAY`): die Wiedergabe beginnt beim ERSTEN Ereignis
  (ein Ereignis in Bild 0 voranstellen), eine Taste bleibt gedrückt bis KEY_UP,
  eine Beilage ohne Zeilenende verliert ihre letzte Zeile, und `KEY_ANY_HIT`
  sieht eingespielte Tasten nicht -- dafür echte Fensternachrichten über
  `tests/pruef/_hilfen/fenstersender.ps1`.
- `PROCESS_READ$` bis zu fünf leere Abfragen lang weiterlesen, bevor man
  aufgibt (das Kind kann schon beendet sein, bevor die letzte Zeile da ist).
- Fehlertexte des Betriebssystems sind lokalisiert -- in `--- fehler` nur den
  eigenen Teil der Meldung.

**IDE, Studio, Werkzeuge**
- Im Layout eines Studio-Fensters zählt `h` die Titelleiste mit (nur das
  Startfenster läuft ohne), eine Menüleiste kommt im Entwurf wie im
  Programm obendrauf (`menueLeisteH`).
  Ein Datenbank-Element aus einer fremden Datei hält deren Arten wörtlich
  (`sql`, `sqlvorgabe`) -- der Abgleich darf ihre Tabellen nicht umbauen.
- Anzeigetexte gehen durch `tr$` (Tafeln `ide/sprache/en.txt`,
  `studio/sprache/en.txt`, `examples/glutfaden/daten/sprache/en.txt`,
  `examples/studio/kuehlschrank/sprache/en.txt`);
  **Schlüssel bleiben deutsch, übersetzt wird beim Anzeigen** (Ereignisnamen,
  Kategorien, Konfig-Schlüssel, Protokollzeilen).
- Tests der IDE setzen `DH_IDE_KONFIG` (sonst schreibt jeder Lauf dem Nutzer
  eine Sitzung) und lesen `DH_IDE_LOG`; das Studio entsprechend
  `DH_STUDIO_KONFIG`/`DH_STUDIO_LOG`.
- Ein Kürzel steht nur im Menü; dieselbe Taste zusätzlich mit KEYHIT abzufragen
  ließe den Befehl zweimal laufen.
- Studio: wer im Layout auf Widgets per Nummer zeigt (`layout.kinder`,
  `tabctl.kinder`), verschiebt die Nummer beim Ausklappen eines eigenen
  Controls mit (`instanzAusklappen`, `seitenDerTeile` in `studio/modell.dh`).
- Kreuz und ESC: `WINDOW_CLOSE_REQUESTED()` fragt bei Ungesichertem nach;
  `QUITREQUESTED()` ohne Kreuz ist das Bildlimit eines Testlaufs.
- Ein Studio-Projekt übersetzt sich über die Laufzeit des Studios: Tafel
  `sprache/<code>.txt` im Projektordner, im Code `tr$`/`trf$` mit einem
  LITERAL (Prüfungen sammeln die Literale), Fenster, Menüs und Berichte
  übersetzt `studioTexte$` selbst. Daten aus der Datenbank bleiben, wie sie
  sind; was ein Widget zurückliefert, ist die Übersetzung -- verglichen wird
  über den Index oder mit `tr$`.
- Die Kopien der Werkzeuge in Prüfungen laufen in `_programm/` -- Ordner mit
  `_` übergeht die IDE beim Durchsuchen. `--- programm` kopiert per
  `IMPORT "x.dh"` eingebundene Dateien mit, aber `--- einschub nach`,
  `--- ersetzen` und `--- streichen` wirken nur auf die Hauptdatei: Marken,
  an denen Tests der IDE ansetzen, gehören nach `ide/ide.dh`, nicht in
  `ide/teile/`.

**Plattformen und Pakete**
- Unter macOS ist dhrt selbst das Hauptprogramm des Bundles (`appstart.rs`),
  signiert mit Hardened Runtime und `installer/posix/dhrt.entitlements`
  (Maschinencode, Mikrofon, Kamera).
- Pakete und Installer nehmen nur, was git kennt (`gitFremdSammeln`).
- Der Web-Bau schaltet `graphics schriften` ein; was dort nicht übersetzt,
  fällt in der Website-CI auf.

## Arbeitsablauf: Branch + Pull Request

Nicht direkt auf `main` pushen. Für jede Änderung ein Zweig, dann ein PR:

```
git switch -c thema/kurzer-name
# arbeiten, pro abgeschlossenem Punkt ein Commit
git push -u origin thema/kurzer-name
gh pr create --fill        # nutzt .github/pull_request_template.md
gh pr checks --watch       # die sechs Pflicht-Pruefungen abwarten
gh pr merge --squash
```

`main` verlangt sechs Status-Prüfungen (`test (3.12)`, `posix-test` und
`rust-check` je auf ubuntu/macos/windows). Die CI baut ohne Grafik, ohne
Hardware-Features und auf drei Systemen; ein grüner Lauf auf dieser Maschine
sagt darum wenig (der MSVC-Linker schluckt, was ld64 ablehnt). Vor dem Push
auch `cargo check` ohne das Feature `jit` (ein verrutschtes `cfg` fällt sonst
erst in der CI auf).

## Build und Test

Gebraucht werden Rust (`cargo`) und ein beliebiges Python 3 für das Bauskript:
```
python rust\build_runtime.py            # dhrt bauen (--hardware: serial/usb/bt/wifi/midi)
rust\drachenhauch_runtime\target\release\dhrt test tests\pruef             # alle Sammlungen (~15 min)
rust\drachenhauch_runtime\target\release\dhrt test tests\pruef --schnell   # ohne `--- langsam` (~6 min)
rust\drachenhauch_runtime\target\release\dhrt test tests\pruef\module\json.dhtest --filter Text
rust\drachenhauch_runtime\target\release\dhrt pruef      # Doku, Namen, Pfade, Meldungen, Uebersetzung
```
- **Nur `rust/build_runtime.py` baut verlässlich** (CFLAGS für die
  Tipp-Warteschlange, cmake/libclang-Suche); `cargo build --bin dhrt` allein
  kann veraltete Ergebnisse liefern. In einem Worktree mit langem Pfad
  `CARGO_TARGET_DIR=C:/dht` setzen.
- `--schnell` lässt die Sammlungen mit `--- langsam` aus (Werkzeuge, IDE,
  echte Fenster); wer daran ändert, läuft deren Sammlung einzeln, die CI läuft
  alles. `dhrt test` läuft bis zu 8 Dateien nebeneinander, `--- seriell` danach
  allein; jeder Fall hat eine Zeitgrenze (Vorgabe 300 s).
- Die CI fährt dasselbe auf Windows (mit Grafik), Linux und macOS (ohne
  Grafik, `DHRT_OHNE_GRAFIK=1`), dazu `cargo check`/`cargo test` je System.
  `DH_OHNE_AUDIO=1` ohne Soundkarte.
- **Headless prüfen:** `DHRT_FRAMES=n DHRT_SCREENSHOT=p.png dhrt run x.dh`
  liefert ein Bild; für alles, was sich über die Zeit falsch verhält, der
  **Kontaktbogen** `DHRT_FRAMES=480 DHRT_CONTACT=bogen.png` (`DHRT_CONTACT_MAX`,
  `_COLS`, `_EVERY`). Aus einem Programm heraus: `dhrt bild quelle.dh ziel.png
  [bilder]` -- es wechselt in den Ordner der Quelle, ein relatives Ziel landet
  dort.
- Web: `python rust/build_wasm.py examples/webspiel/main.dh web`, dann
  `py -m http.server -d web 8000`.

## Häufige Fallstricke

- **Grafik/Audio brauchen ein Fenster bzw. eine Soundkarte;** ohne Bildschirm
  ist `SCREEN` ein abfangbarer Fehler ("Kein Fenster moeglich"), Zeichnen ohne
  SCREEN ebenfalls.
- `--check` meldet Tippfehler in Variablennamen mit Gültigkeitsbereich; der
  Lauf über alle `.dh` des Repos ist ein Test
  (`tests/pruef/werkzeugkette/dhrt_check.dhtest`) -- neue Fehlalarme fallen
  dort auf. Unter-Projekte (`examples/glutfaden/ansicht/`,
  `examples/bastelzentrale/bereiche/`, Studio-Projekte) sind ausgenommen.
- `JSON_LEN` wirft bei einem Pfad, den es nicht gibt -- vorher `JSON_HAS`; ein
  Punkt im Schlüssel ist ein Pfad, maskiert mit `JSON_KEY$`.
- `PROCESS_START` nimmt Kindern `DHRT_FRAMES`/`DHRT_SCREENSHOT` ab und setzt
  `DHRT_LIVE=1`; "dhrt" als Programm meint die eigene Laufzeit.
- GTK über FFI: ein eben sichtbar gemachtes Widget hat im selben Schritt noch
  keine Größe (`alloc_needed`); `gtk_main_quit` in einem verschachtelten
  `gtk_main` beendet die äußere Schleife.
- Patches mit Rückstrichen nie über einen Bash-Heredoc schreiben (er frisst
  sie) -- Skripte als Datei, Verfälschungen vor dem Bau per grep nachsehen.

## Geschichte

Was wann und warum gebaut wurde, steht unverändert in `geschichte/` (bis
2026-11-07 in dieser Datei). Dateinamen und Zustände dort sind die von damals.

| Datei | Inhalt |
|---|---|
| `geschichte/python-abbau.md` | Stufen B und C, Wege A und D, Rust-Frontend, alte Python-VM |
| `geschichte/tempo.md` | Zeichenketten, Merkplätze, Objektlage, Superinstruktionen, Rahmenstapel, Maschinencode M3–M5 |
| `geschichte/sprache.md` | Sprachfunktionen: Feld-Untergrenzen, `/`, Koroutinen, FUNCREF, Klassen, Comprehensions, Tupel … |
| `geschichte/ffi.md` | `DECLARE … LIB`: Rückrufe, Structs, Bitfelder, va_list, GTK, Python einbetten |
| `geschichte/gui.md` | jede Ausbaustufe des gui-Moduls |
| `geschichte/module.md` | die ausführliche Modultabelle, input, vec2, Layer, Atlas, ECS |
| `geschichte/geraete.md` | Tray, serieller Monitor, Firmata, MQTT, Raspberry Pi, Mikrofon, Webcam, WebSocket/OSC/mDNS, Modbus |
| `geschichte/fenster-und-tests.md` | Klicks zwischen zwei Bildern, Tastennamen, Testfenster, Zeitgrenze |
| `geschichte/werkzeugkette.md` | englische Meldungen, Pakete, englisches Handbuch, Website, LSP, Web-Playground |
| `geschichte/ide.md` | die IDE: alle Stände, Debugger, Dock, Plugins, Hover |
| `geschichte/werkzeuge.md` | die Editor-Piloten und ihre Faktoren, Auffrischen 2026-10, Klassendiagramm |
| `geschichte/studio.md` | Drachenhauch Studio: Stufen, Datenbanken, Berichte, eigene Controls |
| `geschichte/kuehlschrank.md` | Kühlschrank-App (PySide6-Port) im Studio: Stufen und Funde |
| `geschichte/glutfaden.md` | Glutfaden (PySticky-Port), Stufen 1–26 |
| `geschichte/dogfooding.md` | die Dogfooding-Runden, Bastelzentrale, Nachtflug, Lichterglanz |
