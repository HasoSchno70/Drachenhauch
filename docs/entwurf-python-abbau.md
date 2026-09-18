# Entwurf: Python abbauen — alles über Rust und Drachenhauch

> **Stand 17.09.2026: Bestandsaufnahme vor dem Abbau** in Abschnitt 7 --
> was noch Python ist (~62 500 Zeilen), was außerhalb davon am Paket hängt,
> was die Qt-IDE und die Qt-Editoren noch voraushaben, ein Datenverlust im
> Tracker, und eine Reihenfolge fürs Löschen mit vier offenen Entscheidungen.
> **Abgearbeitet ist daraus, was keine Entscheidung braucht:** der
> Tracker-Datenverlust, die toten Starter samt `openpyxl`, der Pan-Regler im
> GB-Code des SFX-Generators und der Umzug der drei `builtin_*.json` nach
> `daten/` (7.2). Von den vier Entscheidungen am Ende von 7.7 sind zwei
> gefallen (18.09.): alte Qt-`.dhsprite` müssen nicht mehr aufgehen (d),
> und die zwei Bauskripte bleiben Python, nur mit der Standardbibliothek
> (c). Und keine Qt-Funktion blockiert das Löschen (a): was fehlt, wird
> gebaut, erst die acht Punkte der Stufe A (7.7). Offen ist nur noch (b).
>
> **Stand 06.09.2026: Weg A ist gebaut.** `dhrt lsp` (Sprachserver in Rust,
> `lsp.rs` + `symbole.rs`), `dhrt doku prosa|grammatik|referenz` und
> `dhrt pruef bloecke|namen|zaehlungen|konstanten|pfade` ersetzen
> `drachenhauch/lsp/`, `tools/gen_builtin_prosa.py`, `tools/pruef_docs.py`,
> `tools/pruef_doku_aussagen.py`, `vscode-drachenhauch/build_grammar.py` und
> `drachenhauch/doku.py`; die handgepflegten Hover-Texte liegen als
> `builtin_docs.json` vor (Python lädt, dhrt bettet ein). Die VS-Code-Erweiterung
> startet `dhrt lsp` und braucht kein Python mehr; `dhrun.py --doku` reicht nur
> noch durch. Offen aus A: `build_runtime.py` bleibt (Bau der Runtime), der
> Installer bleibt bei PyInstaller, solange die IDE Python ist (Weg C).
>
> **Stand 06.09.2026, abends: Weg C hat Stufe 1.** `ide/ide.dh` (rund 760
> Zeilen Drachenhauch) oeffnet Dateien in Reitern, faerbt ein, prueft ueber
> `CODE_CHECK$`, zeigt Hilfe und Vervollstaendigung aus dem Sprachserver-Kern
> (`CODE_HOVER$`, `CODE_COMPLETE`), sucht und ersetzt, startet das Programm
> mit laufender Ausgabe (`PROCESS_*`) und reicht Eingaben durch. Dafuer kamen
> in dhrt die drei Bausteine aus Abschnitt 2: Prozesse mit laufender Ausgabe,
> die Textbereich-Befehle (`GUI_TEXTAREA_CURSOR/GOTO/SELECTION$/SELECT/INSERT/FIND`)
> und die Sprachdienste als Builtins (`CODE_*`). **Stufe 2 (gleicher Tag):**
> Debugger als Client von `dhrt debug` (Haltepunkte, Schritte, Variablen),
> Profil aus `dhrt profile`, Suche im Projekt, Befehlspalette, Marken in der
> Nummernspalte (`GUI_TEXTAREA_MARKS`), echte Schriften, Start maximiert.
> **Stufe 3 (gleicher Tag):** Handbuch im Fenster (F1), Listing drucken/PDF,
> Werkzeuge-Menue mit den fuenf Begleit-Editoren, Ausdruck-Auswertung im
> Debugger, und `installer/Drachenhauch-IDE.iss` -- ein Installer ohne
> Python (33 MB statt 92). Damit ist die Liste aus Abschnitt C bis auf
> Willkommensseite und Themen abgearbeitet; was der Qt-IDE noch voraus ist,
> steht in `docs/ide.md` unter "Was noch fehlt". **Stufe 4 (2026-09-09):**
> Willkommensseite, Sitzung und Einstellungen (`ide.json`), Datei im
> Projekt oeffnen, die Zeilen-Handgriffe samt Formatieren (`CODE_FORMAT$`),
> Lesezeichen, Gliederung, bedingte Haltepunkte, Export als Programm,
> Druckdialog, TODO-Liste, Tastenkuerzel-Uebersicht -- gemessen an den
> Menues der Qt-IDE; der Rest dort steht jetzt in `docs/ide.md`.
>
> **Weg B hat begonnen (gleicher Tag):** der Form-Designer als
> `examples/197_form_designer.dh` (860 Zeilen gegen 5 055, Faktor 0,17) auf
> einem neuen Entwurfsmodus der Laufzeit (`GUI_WINDOW_DESIGN`); Details in
> `docs/form-designer.md`. Der Anim-FSM-Editor folgte am 2026-09-07
> (`examples/198_anim_fsm_editor.dh`, 1 336 gegen 1 728, Faktor 0,77;
> `docs/anim-editor.md`), das Notenblatt am selben Tag
> (`examples/199_notenblatt.dh`, 1 449 gegen 1 710, Faktor 0,85;
> `docs/score-editor.md`). Audio Studio wird nicht portiert -- die
> Qt-Fassung ist ein 123-Zeilen-Reiterrahmen um Tracker und SFX-Generator,
> und das Werkzeuge-Menue der IDE ersetzt ihn. **Damit hat jeder
> Qt-Editor eine Drachenhauch-Fassung.** Offen aus B: je Pilot der Rest
> aus dem Kopfkommentar.
>
> **Weg D hat begonnen (2026-09-07):** `dhrt test` kann Pruefsammlungen
> (`*.dhtest`: viele Faelle mit erwarteter Ausgabe in einer Datei, siehe
> `docs/werkzeuge.md`). Gemessen sind 1 084 der 3 949 pytest-Tests reine
> Ausgabevergleiche in 79 Dateien. **74 Sammlungen mit 1 540 Faellen sind
> umgezogen, 73 pytest-Dateien geloescht** -- nicht durch Umbau des
> Quelltexts, sondern durch Aufzeichnung: ein Wegwerf-Plugin schrieb bei zwei
> pytest-Laeufen je Aufruf Quelltext, Ausgabe und Beilagen mit, und nur was
> beide Male gleich war, wurde ein Fall. Was Zufall, Uhr oder absolute Pfade
> enthielt, wurde von Hand zu selbstpruefenden Faellen; Binaerbeilagen gehen
> als `--- datei name base64`; Grafik-Tests pruefen Punkte am
> Bildschirmfoto mit `--- bild`, WAV-Dateien mit `--- ton`, und
> `--- seriell` laesst die Faelle einer Datei nacheinander laufen. Stand
> 2026-09-08: 151 Sammlungen mit 2403 Faellen, 118 pytest-Dateien weniger;
> Standardeingabe, Argumente, Rueckgabewert und stderr gehen ueber
> `--- eingabe`, `--- argumente`, `--- rueckgabe`, `--- stderr`.
> Bleiben in pytest: das Dateisystem (Dateizeiten, Gross/Klein je System),
> Tests mit fremden Umgebungsvariablen oder Zeitwerten, und Tests, die
> Quelltext, Bauskripte oder fremde Formate mit einem fremden Leser pruefen.

*Untersuchung, keine Umsetzung.* Die Richtung ist ausgesprochen: Python
soll irgendwann ganz wegfallen, sämtlicher Code läuft über Rust — also über
`dhrt` und über Programme, die in Drachenhauch selbst geschrieben sind.
Dieses Papier misst, was Python heute noch tut und für wen, was `dhrt` für
die Ablösung schon kann und was ihm fehlt, entwirft die Wege und empfiehlt
eine Reihenfolge. Die Entscheidung fällt jemand anders.

Alle Zahlen sind gemessen — Stand 06.09.2026, dieses Repository, diese
Maschine.

## 1. Was Python heute tut — gemessen

**Zeilen Python nach Bereich** (ohne `.venv`, `target`, `build`, `dist`):

| Bereich | Zeilen | Dateien | Wer braucht es |
|---|---|---|---|
| `tests/` | 58 600 | 300 | die Entwicklung und die CI |
| die neun Qt-Editoren (`*_qt.py`) | 16 250 | 9 | Nutzer der IDE |
| `editor_qt/` (IDE, Highlighter, Completer, Symbole, Konsole, Debugger-Anbindung) | 16 000 | 53 | Nutzer der IDE |
| Datenmodelle der Editoren (`spriteeditor`, `tracker`, `formdesigner`, `tilemap`, `animeditor`, `score`) | 6 000 | 24 | die Editoren und ihre Tests |
| `tools/` (Doku-Prüfer, Prosa-Generator, Schriftmaße, Qt-Testläufer) | 1 450 | 7 | Entwicklung, CI |
| `lsp/` + VSCode-Grammatik | 600 | 4 | Nutzer von VS Code |
| `installer/`, `rust/build_runtime.py`, `dhrun.py` | 1 700 | 5 | Entwicklung, Bau der Fassung |
| Spiele, Buch-Werkzeuge, `cloudserver` | 3 000 | ~12 | einzelne Projekte |
| **gesamt** | **~110 000** | | |

`dhrt` hat 61 600 Zeilen Rust.

**Wer Python wirklich braucht.** Ein Programm in Drachenhauch braucht heute
schon **kein** Python: `dhrt run`, `dhrt --export`, `dhrt test`, `dhrt
--check`, `dhrt debug`, `dhrt profile`, `dhrt fmt` sind Rust. Die
Python-Abhängigkeit hängt an genau zwei Stellen: an der **IDE mit ihren
Editoren** und an der **Entwicklung des Projekts selbst** (Tests, Prüfer,
Bau). Der Nutzer, der ein Spiel schreibt, sieht Python nur, weil die IDE es
mitbringt.

**Was der Installer davon trägt:** die entpackte Verteilung ist 168 MB.
Davon sind PySide6 und shiboken **95 MB**, numpy und Pillow **19 MB** — 114
der 168 MB, zwei Drittel, sind die Python-Seite. `dhrt` selbst hat 17 MB.

**Die Tests, aufgeteilt:** 114 Dateien (25 100 Zeilen) importieren
PySide6, einen Editor oder ein Python-Datenmodell — sie prüfen Python und
verschwinden mit ihm. 186 Dateien (33 500 Zeilen) tun das nicht; 174 davon
treiben `dhrt` (Golden-Tests über `run_gb`, Bildvergleiche, Prozess- und
Automation-Tests). **Diese 174 sind das, was bleibt und übersetzt werden
muss.**

## 2. Was `dhrt` für die Ablösung schon kann

Die fünf Piloten (SFX, Partikel, Tilemap, Sprite, Tracker) haben gemessen,
dass ein Editor in Drachenhauch geht — Faktor 0,38 bis 1,18 gegen die
Qt-Fassung, je nachdem, wie viel weggelassen wird. Für eine **IDE** in
Drachenhauch liegt außerdem bereit:

| Baustein | Stand |
|---|---|
| Code-Feld | `GUI_TEXTAREA` mit Syntax-Einfärbung (`SYNTAX_SPANS`), Zeilennummern, aktiver Zeile, Tabulator, Umbruch, Strg+Z; 30 000 Zeilen kosten je Anschlag 2 ms |
| Oberfläche | Menüs mit Kürzeln, Reiter, Baum, Tabelle, Trenner, Layout-Behälter, Dialoge, Ziehen und Ablegen, Themen, Maßstab, Bedienung ohne Maus, Barrierefreiheit |
| Dateien | Verzeichnisse lesen, Zeitstempel, native Datei-Dialoge (`rfd`), Zwischenablage |
| Übersetzen und Prüfen | `dhrt --check` liefert Probleme als JSON; Hover-Texte liegen in `builtin_prosa.json` (aus `docs/` erzeugt) |
| Laufen lassen | `SHELL_START` (Prozess im Hintergrund), `WINDOW_OPEN` (zweiter `dhrt` mit Textkanal), `dhrt debug` (JSON-Protokoll über stdin/stdout — der Qt-Debugger ist nur ein Client davon), `dhrt profile` (JSON) |
| Drucken, Ausgabe | pdf-Modul mit Drucken und Vorschau, `OPENDOC`, `OPENURL` |
| Geschwindigkeit | `--check` von 1 200 bis 2 800 Zeilen: **79–95 ms**. Eine IDE von 20 000 Zeilen übersetzte damit in unter einer Sekunde (hochgerechnet, nicht gemessen) |

Was noch nicht da ist, aber gebraucht würde — je Stück klein:

| Lücke | Wozu | Aufwand |
|---|---|---|
| **Prozess mit laufender Ausgabe** | die Konsole der IDE zeigt, was ein Programm druckt, WÄHREND es läuft; `SHELL_START` liefert heute nur das Ergebnis am Ende (`SHELL_RESULT$`) | 1–2 Tage (`SHELL_READ$`-Muster wie `WINDOW_RECV$`, dazu stdin schreiben für `INPUT` und den Debugger) |
| Textbereich-Befehle | Suchen/Ersetzen, zu Zeile springen, Auswahl lesen und setzen, Schreibmarke abfragen (heute gibt es keine Abfrage — der Test tippt ein `#`) | 2 Tage |
| `dhrt lsp` | Symbole, Hover, Vervollständigung für VS Code und die eigene IDE aus EINER Quelle; heute rechnen `symbols.py` (468 Zeilen), `completer.py` (102), `builtin_docs.py` (567) in Python nach, was dhrt beim Übersetzen längst weiß | 1 Woche |
| `dhrt doku` / `dhrt pruef` | die Doku-Werkzeuge aus `tools/` (Prosa aus `docs/`, Codeblöcke prüfen, Aussagen gegen den Index) | 3 Tage |
| Markdown anzeigen | das Handbuch in der IDE (`markdown_viewer.py`, 414 Zeilen) | 2–3 Tage in Drachenhauch (Text mit Überschriften, Listen, Code — kein Browser) |
| Installer ohne PyInstaller | `dhrt.exe`, die IDE und Editoren als `.dh`, Beispiele, Bücher; Inno Setup bleibt | 2 Tage — und der Installer schrumpft um zwei Drittel |

## 3. Vier Wege

### A. Werkzeugkette und LSP nach `dhrt`

`dhrt lsp`, `dhrt doku`, `dhrt pruef`; `dhrun.py` wird ein dünner Starter
oder fällt weg (`dhrt` chdirt schon selbst); `build_runtime.py` bleibt als
einziges Python für den Bau der Runtime (oder wird ein Skript des
Betriebssystems); der Installer kommt ohne PyInstaller aus, sobald die IDE
kein Python mehr ist. **Ein bis zwei Wochen.** Gewinn: VS-Code-Nutzer und
die CI brauchen kein Python mehr; der Index und die Prosa haben einen
einzigen Herrn. Kein Nutzer der IDE merkt etwas.

### B. Die Editoren in Drachenhauch

Fünf gibt es als Piloten. Damit sie die Qt-Fassungen **ablösen**, fehlt je
Pilot der Rest, der im Kopfkommentar steht (beim Sprite-Editor: Dialoge,
Datei-Browser, Statistik-Feinheiten; beim Tracker: Sample-, Keymap- und
SoundFont-Instrumente, VU-Meter). Vier gibt es noch nicht: Form-Designer
(1 558 Zeilen Modell + 3 500 Qt), Anim-FSM-Editor, Notenblatt, Audio
Studio (das Tracker und SFX vereint — in Drachenhauch wäre es ein
Programm, das die beiden Piloten lädt). **Je Editor ein bis drei Wochen,
zusammen zwei bis drei Monate.** Jeder fertige Editor nimmt seine Qt-Fassung
und deren Tests mit.

### C. Die IDE in Drachenhauch

Reiter mit Code-Feldern, Projektbaum, Suche im Projekt, Konsole mit
laufender Ausgabe, Fehlerliste aus `--check`, Hover und Vervollständigung
aus `dhrt lsp`, Debugger-Fenster als Client von `dhrt debug`,
Profiler-Ansicht aus `dhrt profile`, Befehlspalette, Willkommensseite,
Themen, Druck des Listings über das pdf-Modul, das Handbuch im Fenster.
Alles, was die Qt-IDE in 32 000 Zeilen Python tut — nach den Pilotfaktoren
**12 000 bis 25 000 Zeilen Drachenhauch**, das größte Programm, das je in
dem Dialekt geschrieben wurde, und damit zugleich der härteste Test für ihn.
**Ein bis zwei Monate**, nach den Lücken aus Abschnitt 2. Solange sie nicht
gleichzieht, bleibt die Qt-IDE im Installer; die Ablösung ist eine
Checkliste, kein Datum.

### D. Die Tests

114 Dateien gehen mit ihren Editoren. Die 174 `dhrt`-Tests werden
Prüfprogramme für `dhrt test` (ein `.dh` mit Erwartung, für Bilder ein
Vergleich gegen eine PNG, für Prozesse ein zweites `.dh`) oder
Rust-Integrationstests (Golden-Dateien neben dem Quelltext). Mechanisch,
aber viel: **zwei bis drei Wochen**, am besten je Bereich, wenn der Bereich
ohnehin angefasst wird. Bis dahin läuft pytest weiter — es prüft ja `dhrt`,
nicht Python.

### E. Die IDE in Rust (egui, iced)

Schneller zu bauen als in Drachenhauch, aber gegen die Identität: die IDE
wäre dann das eine Programm der Familie, das man **nicht** in Drachenhauch
lesen und ändern kann, und jede Lücke des Dialekts bliebe unentdeckt, statt
am größten Programm aufzufallen. Verworfen.

## 4. Nebeneinander

| | A Werkzeugkette | B Editoren | C IDE | D Tests | E Rust-IDE |
|---|---|---|---|---|---|
| Python weg für | VS Code, CI | je Editor | IDE-Nutzer | Entwicklung | IDE-Nutzer |
| Zeilen Python, die fallen | ~3 700 | ~22 000 + 25 000 Tests | ~16 000 | ~33 500 | ~16 000 |
| Installer | — | — | **−114 MB** | — | −114 MB |
| Braucht von `dhrt` | `lsp`, `doku`, `pruef` | wenig (Piloten laufen) | Prozess-Ausgabe, Textbereich-Befehle, `lsp` | `dhrt test` (da) | nichts |
| Aufwand | 1–2 Wochen | 2–3 Monate | 1–2 Monate | 2–3 Wochen | 1 Monat |
| Risiko | gering | gering, je Editor | mittel: das größte Programm im Dialekt | gering, viel Fleiß | gegen die Identität |

## 5. Empfehlung

**A, dann C, dann B — D je Bereich, wenn der Bereich fällt.**

A zuerst, weil es klein ist, sofort wirkt (VS Code und CI ohne Python) und
das baut, was C braucht: `dhrt lsp` ist der Kern jeder IDE, und er gehört
nach `dhrt`, wo Lexer, Parser und Index schon liegen — heute rechnet Python
nach, was Rust weiß.

C vor B, weil die IDE das ist, was jeder Nutzer sieht, und weil sie den
Installer um zwei Drittel verkleinert. Und weil sie die Frage beantwortet,
die hinter der ganzen Richtung steht: **kann man in Drachenhauch ein
Programm dieser Größe schreiben?** Die Piloten sagen ja bis 2 800 Zeilen.
Die IDE ist die Probe aufs Zehnfache — was ihr im Dialekt fehlt, fehlt
allen großen Programmen, und man findet es nur so.

B danach, Editor für Editor, aus der neuen IDE heraus gestartet wie heute
die Piloten. D nebenher: jeder Bereich, der nach Drachenhauch geht, nimmt
seine Tests als `dhrt test`-Programme mit.

Bis zum Ende bleibt die Qt-IDE im Installer, und pytest prüft weiter. Die
Regel ab sofort ist eine andere: **nichts Neues mehr in Python**, wenn es in
Rust oder Drachenhauch geht.

**Reihenfolge:** A (1–2 Wochen) → Lücken aus Abschnitt 2 (1 Woche) → C
(1–2 Monate) → B (2–3 Monate) → D nebenher.

## 6. Was ohne Entscheidung schon gilt

* Ein Spiel oder eine Anwendung in Drachenhauch braucht **heute kein
  Python** — `dhrt run`, `dhrt --export`, `dhrt test`.
* Die fünf Piloten laufen aus der IDE heraus und tun, was sie tun; wer
  einen Editor in Drachenhauch will, hat fünf Vorlagen.
* `dhrt debug` und `dhrt profile` sind Protokolle, keine Qt-Funktionen —
  jede IDE kann sie nutzen, auch eine in Drachenhauch.

## 7. Bestandsaufnahme vor dem Abbau (Stand 17.09.2026)

*Untersuchung, keine Umsetzung.* Die Wege A bis D sind so weit gegangen,
dass die Frage nicht mehr lautet, **ob** Python wegfallen kann, sondern
**was dabei verloren ginge** und in welcher Reihenfolge man löschen müsste.
Gemessen ist dieses Repository; der Vergleich der Oberflächen stammt aus
einer Durchsicht der Menüs, Aktionen und Dialoge beider Fassungen, die
schwersten Befunde sind im Quelltext nachgesehen (unten mit ✔ markiert).

### 7.1 Was noch Python ist

| Bereich | Dateien | Zeilen | fällt mit |
|---|---|---|---|
| die neun Qt-Editoren (`*_qt.py`) | 9 | 16 340 | den Editoren |
| `editor_qt/` (die Qt-IDE) | 53 | 15 440 | der IDE |
| Datenmodelle (`spriteeditor`, `tracker`, `formdesigner`, `tilemap`, `animeditor`, `score`) | 21 | 7 530 | den Editoren |
| Reste der Sprache (`lexer.py`, `tokens.py`, `preprocess.py`, `graphics.py`, `synth.py`, `particle_sim.py`, `audio_preview.py` …) | 12 | 1 750 | der IDE (nur sie liest sie noch) |
| Tests zu Qt | 67 | 14 110 | den Editoren und der IDE |
| Tests zu Python-Modellen | 20 | 4 150 | den Modellen |
| übrige pytest-Dateien und `conftest.py` | 9 | 1 310 | siehe 7.6 |
| `dhrun.py`, `installer/*.py`, `rust/build_*.py`, `tools/qt_tests_einzeln.py` | 6 | 1 830 | siehe 7.5 |
| **gesamt** | **197** | **~62 500** | |

Am 06.09. waren es rund 110 000 Zeilen. Die Prüfsammlungen stehen bei 235
Dateien mit 3 475 Fällen. **Kein pytest-Test prüft noch etwas an `dhrt`,
das nicht auch eine Sammlung prüft** -- bis auf die Ausnahmen in 7.6.

### 7.2 Was außerhalb von Python am Paket hängt

Diese Stellen brechen, wenn `drachenhauch/` einfach gelöscht wird -- sie
müssen **vorher** umziehen:

* ~~**`builtin_index.json`, `builtin_docs.json`, `builtin_prosa.json`** liegen
  in `drachenhauch/editor_qt/`.~~ **Erledigt am 17.09.2026:** sie liegen jetzt
  in **`daten/`** neben `docs/`. Angefasst wurden die sechs `include_str!`
  (viermal `compiler.rs`, zweimal `lsp.rs`), die Repo-Wurzel-Erkennung in
  `doku.rs` (sie erkannte das Repo **an genau diesem Pfad**) samt ihrer
  Meldung, das Schreibziel der Prosa, der Index-Leser in `pruef.rs`, die
  Anhänge beider Bücher (`90_anhang_a.js`, `36_anhang_c_weiter.js`), die
  Sammlungen `buch_pruefungen` und `doku_pruefungen` -- und auf der
  Python-Seite `dhrt_meta.daten_datei()`, das den Ordner wie
  `dhrt_locate.find_dhrt` aus Kandidaten sucht (Bundle zuerst, dann
  Repo-Wurzel). **Die eine stille Stelle dabei war der Installer:**
  `collect_data_files("drachenhauch")` sammelte die drei Dateien, weil sie
  im Paket lagen -- die PyInstaller-Spec nennt `daten/` jetzt ausdrücklich,
  nachgesehen im gebauten Bundle (`_internal/daten/`, alter Ort leer).
  Ohne diesen Eintrag wäre der Hover der installierten IDE stumm auf seinen
  Minimal-Satz zurückgefallen, denn die Leser fangen eine fehlende Datei ab.
* **`tokens.py`** liest `dhrt_lsp.dhtest`, um die Schlüsselwörter gegen die
  Vervollständigung zu halten. Ersatz: gegen `lexer::KEYWORDS` (die Liste
  hat schon einen Rust-Test an `keyword()`) oder den Fall streichen.
* **`graphics.py`** (`KEYS`/`COLORS`) hält `test_constants_sync.py` gegen
  `vm.rs` -- ein Drift-Schutz zwischen zwei Pflegestellen, der mit der
  zweiten Stelle überflüssig wird.
* **`modules/__init__.py`** muss laut Anleitung synchron zu
  `preprocess.rs` bleiben -- das fällt ersatzlos, sobald der
  Python-Preprocess fällt.

### 7.3 Die IDE: was die Qt-Fassung noch voraus hat

Auf **Menü-Ebene** stimmt die Aussage aus `docs/ide.md`, dass seit Stand 9
nichts mehr offen ist. Die Lücken liegen in Tastatur, Maus, Panels und
Dialog-Optionen -- einzeln klein, zusammen spürbar für jemanden, der die
Qt-IDE gewohnt ist:

| Bereich | fehlt in `ide/ide.dh` |
|---|---|
| Datei | ~~**Absturz-Wiederherstellung**~~ (seit 18.09. da, siehe `docs/ide.md`; dabei gefunden: das **Kreuz des Fensters** beendete die IDE ohne Rückfrage, und ESC in einem Dialog beendete sie ganz -- beides seit 18.09. behoben, `WINDOW_CLOSE_REQUESTED`); **Dateien ins Fenster ziehen** ✔; Druckoptionen (Schriftgröße, Duplex, Ränder, Zeilennummern, Umfang); gefaltete Blöcke je Datei merken |
| Bearbeiten | Suchen mit **Groß/klein** und **ganzes Wort** als Schalter ✔; **einzeln ersetzen** (nur „alle"); Fundstellen **beliebiger Namen** (heute nur „Wer ruft das auf?" für Unterprogramme); **Klammern und Anführungszeichen automatisch schließen**; Formatieren beim Sichern |
| Ausführen | **nur die Auswahl ausführen** ✔; Haltepunkt per **Klick in die Nummernspalte**, Bedingung per Rechtsklick; Profil nach **Funktionen** gruppiert; ~~**Klick auf `datei:zeile` in der Ausgabe** springt dorthin~~ ✔; Ausgabe leeren |
| Ansicht | **Zoom mit Strg+Rad und Strg+Plus/Minus/0**; **eine Datei geteilt** in zwei Ansichten (heute nur zwei verschiedene Reiter); ein- und ausblendbare Seitenleisten und Panels |
| Hilfe | **Tooltip beim Überfahren** (heute steht `CODE_HOVER$` in der Statuszeile, für das Wort an der Marke) ✔; **Strg+Klick** springt zur Definition; Filterfeld und Beispiel-Kategorien im Dateibaum |

Umgekehrt hat `ide.dh` vieles, was die Qt-IDE nie hatte (Umbauten mit
Vorschau, Aufrufer-Baum mit Durchläufen, git-Fenster, Spaltenauswahl,
Sitzung je Projekt) -- und sie startet **alle acht** Editoren in
Drachenhauch; die Qt-IDE erreicht den Form-Designer und den Anim-Editor gar
nicht.

### 7.4 Die Editoren: was fehlt, und ob die Dateien zusammenpassen

| Editor | Dateiformat Qt ↔ Drachenhauch | größte Lücken der Drachenhauch-Fassung |
|---|---|---|
| SFX (183) | **verschieden**: Qt-Presets in `~/.drachenhauch/presets/sfx.json`, Drachenhauch in `.ini`-Dateien | Abtastrate beim WAV-Export; benannte Presets (~~der GB-Code übergeht den Pan-Regler~~ -- am 17.09. behoben, er liefert jetzt lauffähigen Code samt Pan) |
| Partikel (185) | **verschieden** (wie SFX) | Hintergrund der Vorschau; „in Drachenhauch testen", „als .dh speichern"; benannte Presets |
| Tilemap (187) | Tiled-JSON, **aber** `CONST KACHEL = 16` ✔ -- eine Qt-Karte mit anderer Kachelgröße wird falsch zerlegt; höchstens 128×128 ✔ (Qt 1000) | Kachelgröße wählen, Karte vergrößern; Eigenschaften an Objekten; Rückgängig für Ebenen und Objekte; „Speichern unter" |
| Sprite (189) | **unverträglich** ✔: Qt-`.dhsprite` ist JSON mit base64-Pixeln, Drachenhauch liest PNG + JSON (bewusst, siehe Kopfkommentar); einziger Weg ist der Atlas-Export, ohne Ebenen | Farbe ersetzen; Frame-Werkzeuge (umkehren, Ping-Pong, zusammenfügen, ziehen); Deckkraft, Namen und Reihenfolge der Ebenen (höchstens 4); Sheet-Import mit Gitter; höchstens 16 Frames und 128 px |
| Tracker (190) | dasselbe JSON; ein Sample-, Keymap- oder SoundFont-Instrument wird beim Laden stumm -- bis 2026-09-17 schrieb der Pilot es beim **Sichern als `synth`** zurück und die Samples waren weg (**behoben:** es läuft jetzt unverändert durch) | genau diese drei Instrumentarten; VU-Meter; Patterns benennen; der Sample-Offset-Effekt |
| Form-Designer (197) | dieselbe `.dhform`, fremde Felder laufen durch; **Qt-Projekte** (`.dhproj`) nicht | Mehrfachauswahl und Ausrichten; Kopieren/Einfügen; Projekte mit mehreren Formularen; Code-Fenster; Zoom und Lineale; viele Inspektor-Felder (sichtbar, Gruppe, Platzhalter, Passwort, Regeln, Bindung, Layout-Zuordnung, Fokus-Handler) |
| Anim-FSM (198) | voll verträglich | höchstens sechs Bedingungen je Übergang |
| Notenblatt (199) | voll verträglich | Warnungen beim Umrechnen in den Tracker; Speicherort des Tracker-Projekts; Vollbild |
| Audio Studio | -- | bewusst nicht portiert (Reiterrahmen) |

**Der Tracker-Befund war ein Fehler, keine Lücke** -- er vernichtete
Daten, die der Nutzer nicht sah. Behoben am selben Tag: ein unbekanntes
Instrument wird als rohes JSON mitgeführt (`JSON_GET_JSON`, dafür neu) und
unverändert zurückgeschrieben; beim Abspielen ist es jetzt auch wirklich
stumm (vorher klang es als Rechteck).

### 7.5 Starter, Bau, Verteilung, CI

* **`dhrun.py` und die dreizehn `.cmd`-Starter.** Alles, was nicht Qt ist,
  reicht nur an `dhrt` durch (`run`, `--export`, `--tokens`, `--ast`,
  `--doku`). Die Qt-Modi haben `ide.dh` und die `.dh`-Editoren ersetzt, so
  wie es die Verknüpfungen in `Drachenhauch-IDE.iss` schon tun. **Tot und
  am 17.09. gelöscht:** `dh-build.cmd` (rief ein `setup.py`, das es nicht
  mehr gibt) und `dh-package.cmd` (rief `gb-package.py`, ebenso); dazu nannte
  `pycharm-setup.txt` beide samt `gb.cmd`/`gb.sh` und pygame. Der
  PyInstaller-Rest `22_tetris.spec` lag nur in der Arbeitskopie -- `*.spec`
  ist ignoriert, versioniert war er nie.
  **Kaputt:** `dhrun.py --doku` bricht mit einem NameError ab, weil
  `subprocess` dort nicht importiert ist ✔.
* **`requirements.txt` und `pyproject.toml`.** `openpyxl` stand noch darin,
  obwohl der Test dazu längst eine Sammlung ist ✔ -- am 17.09. gestrichen.
  **Damit ist der fremde Leser der `.xlsx`-Ausgabe weg:** die Erwartungen der
  Sammlung stammen aus dem letzten Lauf mit openpyxl (daher auch die
  Standard-Vorlage in `xlsx.rs`), gegengelesen wird seither mit dem eigenen
  Leser. Wer das zurückwill, braucht einen Leser, der nicht von hier stammt.
  Die Extras `serial`/`usb`/`bt`/`hw` meinen Python-Pakete, die `dhrt` nicht
  braucht.
* **Die beiden Installer.** Nur der PyInstaller-Weg (`build_installer.py`,
  `Drachenhauch.iss`) liefert heute: die Qt-IDE; **die Bücher als
  `.docx`/`.epub`**; die **ESP32-Sketche**; die Lizenzhinweise samt
  Qt/LGPL; das **Aufräumen einer alten GameBasic-Installation**; die
  **Code-Signierung**; und **macOS-`.dmg` und Linux-Paket**. Der
  Python-freie Weg (`bauen.dh`, `Drachenhauch-IDE.iss`) kennt nur Windows.
* **Den Bau der Laufzeit** erledigt weiter `rust/build_runtime.py`: unter
  Windows cmake und libclang suchen, `LIBCLANG_PATH` und
  `CFLAGS=-DMAX_CHAR_PRESSED_QUEUE=256` setzen, Feature-Sätze wählen.
  `build_wasm.py` verdrahtet dazu emscripten (Compiler-Pfade je Version,
  bindgen-Includes, leere GL-Archive, `EMCC_CFLAGS` mit absoluten Pfaden,
  ein Stempel für das Neu-Linken). **Ein Ersatz fehlt, und er ist nicht
  trivial:** die Variablen müssen die Bauskripte FREMDER Crates erreichen
  (raylib-sys, cmake, bindgen) -- ein `build.rs` kann das nicht,
  eine Cargo-Konfiguration mit `[env]` nur für feste Werte. Und `bauen.dh`
  kann die `dhrt.exe` nicht bauen, in der es selbst läuft. **Entschieden
  am 18.09. (c): beide bleiben Python** -- nur mit der Standardbibliothek,
  siehe unten.
* **Die CI.** Der Windows-Job braucht `setup-python`, `pip install -e
  ".[dev,editors]"` (47 s), mypy (10 s), `build_runtime.py`, drei
  pytest-Schritte und den Qt-Läufer; der serielle pytest-Schritt (279 s)
  ist fast ganz der Anker, der `dhrt test tests/pruef` aufruft. Die
  POSIX-Jobs bauen schon ohne Python und brauchen es nur für pytest.
  `package.yml` baut über PyInstaller.
* **Texte.** 47 Markdown-Dateien nennen Python; die Anleitungen in
  `README.md`, `README.en.md`, `docs/rust-runtime.md`, `docs/editor.md`,
  `docs/werkzeuge.md` und `docs/web-playground.md` setzen es voraus. **Die
  Bücher zeigen es dem Leser:** sieben Kapitel des Einstiegs- und des
  Referenzbuchs nennen `dhrun.py` oder `python`, dazu die Kopfkommentare in
  den Buch-Beispielen und `web/playground.js`.

### 7.6 Tests, die nach dem Löschen noch fehlen würden

Die 87 Dateien zu Qt und den Python-Modellen fallen mit ihrem Code. Von den
übrigen prüfen nur vier etwas, das bleibt:

| Datei | prüft | Weg |
|---|---|---|
| `test_dhrt_test.py` | Anker: `dhrt test tests/pruef` | die CI ruft es direkt auf |
| `test_drucken.py` | Druck durch „Microsoft Print to PDF", gelesen mit PyMuPDF | ein Leser für fremde PDFs in Drachenhauch -- oder den Fall aufgeben |
| `test_os_builtins.py` (2) | PRINT/EPRINT-Reihenfolge in einem Strom; `SHELL_OUT$` mit der eigenen Exe | kleiner Baustein in `dhrt` (Ströme zusammenführen) oder aufgeben |
| `test_midi_module.py` | echter und Loopback-MIDI-Anschluss | als Sammlung, die sich ohne Anschluss überspringt |

`test_pruefen.py` (LOG-Reihenfolge), `test_build_wasm.py`,
`test_dhrun_chooser.py`, `test_seriell_liste.py` und
`test_testbefehle_stimmen.py` prüfen Python-Werkzeuge oder pytest selbst
und fallen mit ihnen.

### 7.7 Vorschlag: Reihenfolge

1. **Sofort, unabhängig von allem:** ~~den Tracker-Datenverlust beheben~~,
   ~~die toten Starter und `openpyxl` streichen~~, ~~den Pan-Regler in den
   GB-Code des SFX-Generators~~ -- alles drei am 17.09. erledigt.
2. **Umzüge, die Python nicht stören:** ~~die drei `builtin_*.json` aus
   `drachenhauch/` heraus~~ (am 17.09. nach `daten/`, siehe 7.2);
   `dhrt_lsp.dhtest` gegen `lexer::KEYWORDS`; `test_midi_module` als
   Sammlung.
3. **Lücken schließen, die man vermissen würde** -- entschieden in (a):
   zuerst die acht Punkte der Stufe A (Liste unten), der Rest bei Bedarf.
   ~~Für Qt-`.dhsprite`-Dateien ein einmaliger Import~~ -- entfällt,
   siehe (d).
4. **Verteilung:** der Python-freie Installer übernimmt Bücher,
   ESP32-Sketche, Signierung und das Aufräumen der alten Installation;
   macOS und Linux brauchen einen eigenen Weg oder bleiben vorerst ohne
   Paket.
5. **Bau:** ~~`build_runtime.py` und `build_wasm.py` durch etwas ohne
   Python ersetzen~~ -- sie bleiben als letzte zwei Python-Dateien, siehe
   (c).
6. **CI:** `dhrt test tests/pruef` direkt aufrufen, dann `pip install`,
   pytest, mypy und den Qt-Läufer streichen. Ein Python 3 bleibt für den
   Bauschritt -- ohne venv und ohne Pakete, die Läufer bringen es mit.
7. **Löschen:** `drachenhauch/`, `tests/*.py`, `conftest.py`, `dhrun.py`,
   die Starter, `pyproject.toml`, `requirements.txt`, dazu die 14
   Qt-`.dhsprite`-Dateien in `buch-galaga/assets/sprites/` und
   `examples/platformer/` (das PNG daneben bleibt, es ist das, was die
   Spiele laden); danach Anleitungen und Buchkapitel umschreiben.

**Offene Entscheidungen:** ~~(a) welche Qt-Funktionen wegfallen dürfen~~,
(b) ob macOS und Linux ein Paket brauchen, ~~(c) ob die zwei Bauskripte
Python bleiben dürfen~~, ~~(d) ob alte Qt-`.dhsprite`-Dateien noch geöffnet
werden müssen~~.

**(a) entschieden am 18.09.2026: keine Qt-Funktion blockiert das Löschen,
und keine ist dauerhaft ausgeschlossen.** Was die Drachenhauch-Fassung
noch nicht kann, wird gebaut -- zuerst Stufe A, der Rest, sobald er
fehlt. Vorher nachgesehen, ob eine eigene Datei an einer Qt-Funktion
hängt: keine gespeicherten Qt-Presets (`~/.drachenhauch` ist leer), keine
`.dhproj`-Projekte, alle Karten mit 16er-Kacheln, kein Tracker-Song mit
Sample-Instrumenten. Keiner der Punkte rettet also Daten; es geht nur
darum, was beim Arbeiten fehlen würde.

* **Stufe A, vor dem Abschalten der Qt-Fassung:** in der IDE
  ~~Absturz-Wiederherstellung~~ (18.09., `tests/pruef/werkzeug_ide_wiederherstellung.dhtest`),
  ~~Klick auf `datei:zeile` in der Ausgabe~~ (18.09., `tests/pruef/werkzeug_ide_ausgabe.dhtest`), Zoom mit
  Strg+Rad und Strg+Plus/Minus/0, Tooltip beim Überfahren und Strg+Klick
  zur Definition, Haltepunkt per Klick in die Nummernspalte, Suchen mit
  Groß/klein und ganzem Wort samt einzeln ersetzen; im Notenblatt die
  Warnungen beim Umrechnen in den Tracker (Akkorde werden heute still auf
  eine Note gekürzt); im Sprite-Editor die festen Grenzen anheben (16
  Bilder, 128 px, 4 Ebenen).
* **Stufe B, wenn es fehlt:** Fundstellen beliebiger Namen, Dateien ins
  Fenster ziehen, Klammern automatisch schließen; Tilemap: Kachelgröße aus
  der Datei, Karte vergrößern, „Speichern unter“; Form-Designer:
  Mehrfachauswahl und Ausrichten, Kopieren/Einfügen, einfache
  Inspektor-Felder; Tracker: Sample-Instrumente abspielen.
* **Stufe C, nur auf Zuruf:** die übrigen Punkte aus 7.3 und 7.4.

**(d) entschieden am 18.09.2026: nein.** Den Qt-Sprite-Editor hat nur der
Nutzer selbst benutzt -- fremde Arbeit, die unbemerkt kaputtginge, gibt es
nicht. Im Repo liegen
14 Qt-`.dhsprite`-Dateien, alle von Skripten erzeugt (`make_sprites.dh`
schreibt sie seit dem Umzug nicht mehr) und alle mit dem PNG daneben, das
die Spiele laden -- die Laufzeit liest `.dhsprite` nirgends. Verloren geht
damit nur die Möglichkeit, ihre Ebenen im neuen Editor zu öffnen; die
Pixel bleiben als PNG, und das Skript zeichnet sie jederzeit neu. Kein
Umwandler, kein zweites Format in
`189`. Die Dateien fallen in Schritt 7 mit dem Qt-Editor weg.

**(c) entschieden am 18.09.2026: ja, mit einer Grenze.** `rust/build_runtime.py`
und `rust/build_wasm.py` bleiben die bewusst letzten zwei Python-Dateien.
Sie brauchen nur die Standardbibliothek -- ein beliebiges Python 3 ohne
venv, ohne `pip`, ohne `requirements.txt` --, und gebraucht werden sie nur
von dem, der `dhrt` baut; wer den Installer nimmt oder `dhrt test` laufen
lässt, braucht kein Python. Ersetzen ließen sie sich nur schlecht: auf
einem frischen Klon gibt es noch kein `dhrt`, das ein Bauskript in
Drachenhauch laufen lassen könnte (und unter Windows überschreibt sich eine
laufende `dhrt.exe` nicht selbst); Skripte des Betriebssystems hießen alles
zweimal, als PowerShell und bash, samt der 350 Zeilen emscripten-Einrichtung;
eine Cargo-Konfiguration mit `[env]` deckt nur feste Werte ab, nicht die
Suche nach cmake und libclang -- und `rekursionstiefe.dhtest` hält fest,
dass es sie nicht gibt. **Die Grenze prüft `tests/pruef/bauskripte.dhtest`:**
jede `import`-Zeile beider Skripte muss ein Modul aus einer festen Liste
nennen (`__future__`, `os`, `platform`, `subprocess`, `sys`, `pathlib`,
`shutil`); ein Paket von außen, ein Modul aus `drachenhauch/` oder ein
relativer Import fällt auf. Die Liste ist bewusst eng statt „die ganze
Standardbibliothek“ -- wer ein Modul ergänzt, sieht dabei, dass er die
Grenze berührt. Gegenprobe an den echten Skripten: `numpy`, ein Modul aus
`drachenhauch/`, ein relativer Import, eine Komma-Zeile mit `requests` und
ein eingerückter Import in einer Funktion -- jeder lässt den Fall fallen.
Dass `tests/test_build_wasm.py` das Skript lädt, schadet nicht; es fällt
mit pytest (7.6).
