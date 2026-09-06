# Entwurf: Python abbauen — alles über Rust und Drachenhauch

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
