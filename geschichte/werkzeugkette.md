# Werkzeugkette: Meldungen, Pakete, Doku, Website, LSP, Web

> Entwicklungsgeschichte aus CLAUDE.md (dort bis 2026-11-07), unveraendert
> uebernommen: was wann warum gebaut wurde, welche Fallen es gab, welche
> Tests es pruefen. Englische Meldungen, dhrt paket, das englische Handbuch, die Website, der Sprachserver, der Web-Playground.
>
> Dateinamen und Zustaende sind die von damals -- vieles ist seither
> umgezogen (Python ist weg, die Pruefsammlungen liegen in
> `tests/pruef/<thema>/`). Gueltig ist, was im Code steht und was
> CLAUDE.md heute sagt.

## Meldungen auf Englisch (2026-10-04)

`DHRT_LANG=en` uebersetzt Meldungen an der AUSGABE (`meldung.rs`): Konsole
(`dhrt run`), `--check`, `dhrt lsp`, `CODE_CHECK$`, `dhrt debug`. Im Code
bleiben sie deutsch -- die Pruefsammlungen und die Schnellkorrekturen
(`lsp::korrekturen` liest "Meintest du ...") haengen daran; im Sprachserver
wird darum erst NACH den Korrekturen uebersetzt. Katalog
`daten/meldungen.en.txt` (eingebettet): Vorlage TAB Englisch, `{}` = Stueck
(englisch auch `{2}`), eingefangene Stuecke werden rekursiv uebersetzt, die
Vorlage mit den meisten festen Zeichen gewinnt, ein fuehrendes "-- " und ein
Schlusspunkt gehoeren zur Fuge. CATCH-Werte bleiben deutsch. `dhrt test`
streift `DHRT_LANG` ab (die IDE setzt es bei englischer Oberflaeche).
**Regel fuer neue Meldungen:** deutsch schreiben wie bisher; wer eine
Meldung AENDERT, die im Katalog steht, zieht die Vorlage mit --
`dhrt pruef meldungen` (in `meldungen_englisch.dhtest` und `dhrt pruef`)
meldet eine Vorlage, deren feste Stuecke in keinem Literal mehr stehen.
Setzt sich eine Meldung aus mehreren Literalen zusammen, markiert `[[wort]]`
(ganzes eigenes Literal) bzw. `[[]]` (Naht) die Stelle -- ohne die
strenge Regel fand die Pruefung im ganzen Quelltext fast alles. Drei Runden:
Sprache und allgemeine Laufzeitfehler, dann Befehle und Module (530 von 562
Meldungen der Fehlerfaelle), dann gui/Grafik/Ton (2026-10-05, 350 Vorlagen).
`dhrt pruef meldungen --offen` listet danach 121 Stuecke -- mit Absicht
deutsch sind die CLI-Werkzeuge (main.rs, paket.rs), die Gruende des
Maschinencodes (jit.rs) und innere Formatfehler (model.rs); der Rest sind
Bruchstuecke zusammengesetzter Meldungen, die als Ganzes uebersetzt sind
(`--offen` uebersetzt jedes Literal fuer sich). **Platzhalter im Englischen
sind nur `{}` und `{Zahl}`** (`meldung::einsetzen`) -- vorher fiel ein
Beispiel wie `f"{value:.2f}"` still weg. Dabei fiel eine kaputte Meldung
auf: `GUI_SCALE` trug mitten im Text 25 Leerzeichen aus einem Zeilenumbruch
im Quelltext. **Gewaehlt
wird nach dem GESAMTgewicht** (feste Zeichen aller beteiligten Vorlagen,
`meldung::tief`), nicht nach der obersten Vorlage -- sonst schnitt `{} -- {}`
"TASK: Zugriff auf eine globale Variable ..." an der falschen Stelle durch.
Doku `docs/rust-runtime.md` (Meldungen auf Englisch).

## Pakete: `dhrt paket` (2026-10-04)

`paket.rs`, Entwurf `docs/entwurf-pakete.md`, Handbuch `docs/pakete.md`.
Kein eigener Server (Entscheidung des Nutzers): Quellen sind
`github:nutzer/repo@stand` (-> `https://github.com/.../archive/<stand>.zip`),
https-ZIP, einzelne https-`.dh`, lokaler Ordner/ZIP; ohne `@stand` ein Fehler.
Pakete gehoeren zum Projekt (Entscheidung des Nutzers): `pakete/<name>/`
neben `paket.json`; `paket.lock.json` mit SHA-256 **ueber den entpackten
Inhalt** (`ordner_summe`, nicht ueber die ZIP-Bytes -- GitHub hat 2023 neu
gepackt). `IMPORT` sucht nach "neben der Datei" in `pakete/` von der
importierenden Datei aus nach oben (`preprocess::paketpfade`, wie
node_modules) -- so findet ein Paket SEINE Abhaengigkeit zuerst. Geholt wird
in `pakete/.neu-<name>`, erst nach der Pruefsumme umbenannt (ein falscher
Download laesst das alte stehen); `--erneuern` nimmt neuen Inhalt an. Lokale
Pfade in der `paket.json` eines Pakets gelten ab seinem HERKUNFTSordner;
Kreise erkennt der aufgeloeste Ort (`schluessel`). Beim Holen wird nichts
ausgefuehrt, entpackt ueber `zipdatei::entpacke` (Zip-Slip-Schutz). Die
Ausgaben von `dhrt paket` sind deutsch wie die anderen CLI-Werkzeuge. Tests
`tests/pruef/werkzeugkette/paket.dhtest` (9; HTTP ueber `gegenserver.dh` Modus `dateien`,
der von `DHRT_START_DIR` aus liefert, weil `dhrt run` in den Ordner der
Quelle wechselt).
**In der IDE (2026-10-05):** Fenster `winPak` (Datei -> Pakete des
Projekts, Strg+Alt+P): Liste aus `dhrt paket liste` (per `SHELL_OUT$
(EXEPATH$(), "paket", "-C", ordner, "liste")`, zerlegt an zwei
Leerzeichen), Holen/Alle/Erneuern/Entfernen als Prozess `pakProz` ueber
**`dhrt paket -C <ordner>`** (neu, wie git; die IDE stellt ihren eigenen
Arbeitsordner nicht um), danach `pakFertig`: Baum, Symbolverzeichnis
(`symSchluessel = ""`), Zaehler und `pruefen()` neu. **`projektDateien`
laesst `pakete/` aus** (Umbauten, Projektsuche, Projekt pruefen schreiben
bzw. meinen nur eigenen Code -- das naechste Holen ueberschriebe Aenderungen
dort); lesende Stellen fragen `projektDateien(muster, TRUE)`
(Symbolverzeichnis, `hatByref`, `klassenLesen`, Datei im Projekt oeffnen).
Eine geoeffnete Paketdatei sagt das in der Statuszeile (`paketVon$`). Tests
`tests/pruef/ide/werkzeug_ide_pakete.dhtest` (3; die Paketquelle liegt unter
`_quelle/`, sonst faende das Symbolverzeichnis die Funktion auch ohne Paket
-- so blieb die erste Gegenprobe gruen), ein `-C`-Fall in `paket.dhtest`;
sechs Verfaelschungen der IDE fallen je in ihrem Fall.

## Handbuch auf Englisch: `docs/en` (2026-10-07)

`docs/en/<datei>.md` ist die Uebersetzung von `docs/<datei>.md` (72 Dateien:
Sprache, Befehle, alle Module, IDE, Werkzeuge, Editoren); deutsch bleiben nur
`entwurf-*`, `release-*`, `allzweck-*`, Roadmaps, PERFORMANCE,
`rust-frontend-port`, `umbenennung-drachenhauch` (verlinkt als `../x.md`).
**Code wird nicht uebersetzt, Kommentare in Drachenhauch-Bloecken schon**
(wie im Buch). `dhrt pruef uebersetzung` (`pruef.rs`, auch im vollen `dhrt
pruef`) verlangt je Datei: deutsches Gegenstueck, dieselben Ueberschriften
(Zahl und Ebene, der Reihe nach), gleich viele Tabellenzeilen, dieselben
Codebloecke (Drachenhauch-Bloecke ohne Kommentare verglichen, andere
byteweise) und Verweise relativ zum Ordner der Datei samt Sprungmarke nach
GitHubs Regel (`marken`). Dieselbe strenge Verweispruefung laeuft seither
auch ueber `docs/` (fand `#sprite-atlas--batch-draw`). **Wer ein deutsches
Dokument aendert, zieht die Uebersetzung mit** -- die Pruefung sagt, welche.
Die IDE liest mit englischer Oberflaeche `docs/en/<datei>`, sonst die
deutsche (`docPfad$`). Der Hover nimmt mit `DHRT_LANG=en`
`daten/builtin_prosa.en.json` (von `dhrt doku prosa` aus `docs/en` und den
Befehlstexten des englischen Buchs, `buch_cmd_export.js --en`), sonst nur die
Signatur -- nicht die deutsche Handdoku. Tests `doku_pruefungen.dhtest`
(Uebersetzung, Index, englischer Hover), `werkzeug_ide_englisch.dhtest`
(Handbuch aus docs/en mit Rueckfall), Rust-Tests `marken_wie_github`,
`uebersetzung_folgt_der_deutschen`.

## Website und englisches README (2026-10-28)

Wunsch des Nutzers: Drachenhauch soll genannt werden, wenn jemand eine
Suchmaschine oder einen KI-Assistenten nach einer Programmiersprache fragt.
`README.md` ist seither die ENGLISCHE Fassung (Entscheidung des Nutzers),
die deutsche heisst `README.de.md`; beide beginnen mit einer Einordnung
(Spiele UND GUI-Anwendungen, Vergleich mit QB64/FreeBASIC/Blitz Basic,
PICO-8/LOeVE, Visual Basic). **`tools/website.dh`** baut die Website
(https://hasoschno70.github.io/Drachenhauch/, Workflow `website.yml`, nur
der Kern von dhrt): Startseite en/de mit schema.org `ComputerLanguage`, je
Datei aus `docs/en` mit deutschem Gegenstueck eine Seite in `en/` und `de/`
(README -> index.html; Entwuerfe nicht), Markdown -> HTML in Drachenhauch
(Marken wie GitHub = `marken` in pruef.rs, Code ueber `SYNTAX_SPANS`
eingefaerbt; Verweise auf andere Dateien gehen zu GitHub, Bilder zu
raw.githubusercontent), `llms.txt`/`llms-full.txt`, `sitemap.xml`,
`robots.txt`. **`SYNTAX_SPANS` gehoert seither zu den CODE_-Befehlen
(`try_hintergrund`) statt zur Grafik** -- ohne Fenster war es "im Rust-Kern
noch nicht verfuegbar". Tests `tests/pruef/werkzeuge/werkzeug_website.dhtest`
(Markdown-Stuecke an Beilagen; am echten Handbuch trifft jeder der ~1450
lokalen Verweise samt Marke -- mit falscher Markenregel 135 tote). Was
ausserhalb des Repositorys zu tun ist: `docs/entwurf-sichtbarkeit.md`.
**Falle beim Patchen:** ein Python-Ersatz mit `"\1"` landete als
Steuerzeichen (Code 1) in der Datei -- Rueckstriche per Edit-Werkzeug setzen.

## Sprachserver `dhrt lsp` + VS-Code-Erweiterung

Externe Editor-Unterstuetzung via **LSP** -- und der Server ist die Runtime
selbst: `dhrt lsp` (stdio, JSON-RPC). **Weg A aus
`docs/entwurf-python-abbau.md` (2026-09-06):** bis dahin rechnete
`drachenhauch/lsp/` in Python nach, was dhrt beim Uebersetzen laengst wusste.
Jetzt: [`src/lsp.rs`](rust/drachenhauch_runtime/src/lsp.rs) (Rahmung,
Dokumentspeicher, Verfahren, Hover-Daten) und
[`src/symbole.rs`](rust/drachenhauch_runtime/src/symbole.rs) (Port von
editor_qt/symbols.py: Definitionen, Fundstellen, Bloecke, Kommentar-Doku --
zeilenweise mit ausgeblendeten Kommentaren/Zeichenketten, bewusst KEIN Lexer,
weil ein Sprachserver halb getippten Text sieht; Spalten in ZEICHEN). Diagnose
= dieselbe Kette wie `--check` (`check_source`), auf Pufferzeilen
zurueckgerechnet (`Herkunft` aus preprocess; Fehler in importierter Datei ->
Zeile 1 mit `in datei:zeile ->`). **Diagnose im eigenen Faden** mit
Generationszaehler und 120 ms Sammelzeit -- sonst warteten Hover und
Vervollstaendigung hinter jedem didChange; stdout hinter einem Mutex. Hover:
`builtin_docs.json` (handgepflegt; die Tabelle aus `builtin_docs.py` ist seither
eine JSON-Datei, die Python laedt und dhrt per `include_str!` einbettet) vor
`builtin_prosa.json` (aus `docs/`) vor der Signatur aus dem Index; eigene
Symbole ueber den Kommentarblock ueber der Definition. Vervollstaendigung:
Index (`compiler::builtin_eintraege`), `lexer::KEYWORDS` (neue Liste, Test
haelt sie an `keyword()` fest -- 75 Eintraege), Farben/Tasten aus vm.rs, PI/TAU,
eigene Definitionen. Tests: Rust-`#[test]`s in beiden Dateien +
`tests/pruef/werkzeugkette/dhrt_lsp.dhtest` (echter Prozess ueber
stdio, Client `_hilfen/lspklient.dh`; prueft nebenbei die KEYWORDS aus
`tokens.py` gegen die Vervollstaendigung; bis 2026-09-16 pytest). Der
Qt-Editor benutzt weiter seine Python-Bausteine (`symbols.py`,
`error_check.py`) -- die gehen mit der IDE (Weg C). VS-Code-Erweiterung in
[`vscode-drachenhauch/`](vscode-drachenhauch/): `extension.js` startet
`dhrt lsp` (Einstellung `drachenhauch.dhrtPath`), die TextMate-Grammatik wird
aus `lexer::KEYWORDS` + Index **generiert** (`dhrt doku grammatik`). Doku
[docs/lsp.md](docs/lsp.md). **Bei neuen Keywords/Built-ins:** Grammatik neu
generieren (`dhrt doku grammatik`).

## Web-Playground (dhrt → WASM)

`dhrt` als WebAssembly (emscripten) im Browser. **Der Web-Build laeuft** (Stand
2026-08-03, in `docs/web-playground.md` verifiziert): Konsolen- UND
Grafik-Programme werden **im Browser kompiliert** und ausgefuehrt. Teile:
[`rust/build_wasm.py`](rust/build_wasm.py) (bettet `.dh` als `web/program.dh`
ein, `.dhc` bleibt als Fallback; dazu `cargo`+emscripten-Build, tolerant wenn
die Toolchain fehlt), cfg-gegateter WASM-Einstieg in
[`rust/drachenhauch_runtime/src/main.rs`](rust/drachenhauch_runtime/src/main.rs)
(`#[cfg(target_os = "emscripten")]` liest `/program.dh` zuerst, `/program.dhc`
als Rueckfall), Web-Harness [`web/`](web/) (`index.html` + `playground.js`),
Ton ueber ein eigenes Kira-Backend auf OpenAL
([`src/web_audio.rs`](rust/drachenhauch_runtime/src/web_audio.rs) -- cpal hat
keinen emscripten-Host mehr).

**Zwei Aussagen, die hier lange falsch standen** (die richtige Fassung stand
schon in `docs/web-playground.md`): der blockierende Render-Loop in `vm.run()`
war die *frueher* genannte Kernhuerde -- mit ASYNCIFY laeuft er durch. Und der
Compiler ist **nicht** Python geblieben: seit dem Rust-Frontend kompiliert der
Playground die Quelle selbst, **Pyodide braucht es nicht**.

Doku/Grenzen: [docs/web-playground.md](docs/web-playground.md).
Tests [`tests/pruef/werkzeugkette/build_wasm.dhtest`](tests/pruef/werkzeugkette/build_wasm.dhtest) (Geruest/Harness,
nicht der emscripten-Build; bis 2026-09-21 pytest).
