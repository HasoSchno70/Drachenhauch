# Inkonsistenzen in der Runtime `dhrt`

Stand 2026-11-25, Zweig `thema/wunder-englisch-docx` (Commit `4d4a65b2`).
Untersucht wurde `rust/drachenhauch_runtime/src/` (145 Dateien, rund 138 000
Zeilen): Stellen, an denen man der Runtime anmerkt, dass sie über Monate von
vielen Sitzungen in Stücken gebaut wurde (GameBasic in Python/pygame, dann
raylib, dann Kira, dann der Umzug nach Rust).

**Wie geprüft.** Acht Durchgänge je Bereich (VM/Compiler, reine Builtins,
Grafik/Eingabe/Spiel, Ton, gui außen, gui innen, Daten/Netz, Geräte/System)
haben den Code gelesen. Dazu kam ein Querschnitt über `daten/builtin_index.json`,
`namen.rs` und die Meldungen. Jede Behauptung über Laufzeitverhalten ist, wo
„Lauf“ dabeisteht, mit einem kleinen Programm und
`rust/drachenhauch_runtime/target/release/dhrt.exe run` (Fassung 2026.52)
bzw. den schlanken Laufzeiten aus `target/release/laufzeiten/` nachgestellt.
Seit diesem Bau haben sich nur `main.rs` und `jit.rs` geändert (Meldungen der
Werkzeuge); die Befehle verhalten sich in HEAD gleich. „Code“ heißt: am
Quelltext gelesen, nicht gelaufen. Am Code wurde nichts geändert.

**Fundstellen** sind relativ zu `rust/drachenhauch_runtime/src/`, sofern nicht
anders angegeben. Zeilennummern gelten für den genannten Commit.

**Schwere:**
- **bricht etwas**: plausibler Code rechnet still falsch, stürzt ab oder scheitert.
- **verwirrt**: Nutzer stolpern, es gibt aber einen Ausweg.
- **innen**: nur innen unschön, Gefahr des Auseinanderlaufens.

**Vorschläge** halten `docs/stabilitaet.md` ein. Ein Name bleibt, ein neuer
kommt über `namen.rs` dazu. Parameter bleiben, neue kommen hinten optional
dazu. Wo ein Fehler bisher still falsch rechnete, darf er künftig melden; das
gehört dann unter „Was sich für bestehende Programme ändert“.

## Inhalt

- Das Bild im Ganzen
- A. Gleiches anders gelöst (A1–A20)
- B. Namen (B1–B11)
- C. Reste alter Zeiten (C1–C10)
- D. Struktur (D1–D6)
- E. Doku gegen Code (E1–E6)
- Kleine Einzelfehler (je eine Stelle)
- Rangliste: die zehn Punkte, die sich am meisten lohnen

---

## Das Bild im Ganzen

Die Runtime ist nicht an einzelnen Stellen kaputt. Sie ist **in sechs Mustern
uneinheitlich**, die sich durch fast jeden Bereich ziehen:

1. **Jede `try_*`-Familie hat ihre eigenen Argumenthelfer** (`need_*` in
   `builtins.rs`, `bi_*` in `vm.rs`, `gi`/`gs`/`gb`/`need_f` in `vm_grafik.rs`,
   zwei `gi` in `vm_gui.rs`, `z_int` in `vm_system.rs`). Deshalb heißt dieselbe
   Frage („ist 2.0 ein INTEGER?“, „ist 1 wahr?“) je nach Modul anders, und die
   Meldungen haben fünf Formen (A1, A2).
2. **Jedes Modul hat Handles neu erfunden.** Sie unterscheiden sich in Startwert,
   Freigabe-Wort, Verhalten bei ungültigem oder doppelt freigegebenem Handle und
   darin, ob es überhaupt eine Freigabe gibt (A4).
3. **Einheiten wurden je Sitzung gewählt**: Sekunden, Millisekunden, Bilder,
   Sechzigstelsekunden; Grad und Bogenmaß; Pan als 0..1 oder -1..1; logische
   Punkte oder Bildschirmpixel (A5, A6, A12).
4. **Die Namensreform ist zur Hälfte durch.** `namen.rs` hat 143 neue Namen,
   aber Meldungen nennen weiter den alten. Ein zweiter Alias-Weg läuft direkt
   über Match-Arme, und viele Setter tragen noch kein `SET` (B1–B5).
5. **„Gibt es in diesem Bau nicht“ hat acht Wortlaute.** Der schlanke Export
   erkennt nur vier davon, und `MODULE_AVAILABLE` kennt nur die Hardware-Module (A3).
6. **Reste der Portierung stehen in Meldungen, die Nutzer sehen**: „(Schritt 4)“,
   „im Rust-Kern noch nicht verfuegbar“, „python rust/build_runtime.py“,
   `DH_OHNE_AUDIO`, raylib-Annahmen im Ton, SDL-Tastencodes (C1–C4).

Wer einen dieser sechs Punkte an **einer** Stelle löst, behebt jeweils Dutzende
Einzelbefunde. Darum stehen sie oben in der Rangliste.

---

## A. Gleiches anders gelöst

### A1 · INTEGER-, Zahl- und Wahrheitswert-Argumente: je Familie andere Regeln — *bricht etwas*

- **Wo:** `vm.rs:332` `bi_int` (329 Aufrufe: db, net, xlsx, pdf, httpd, mqtt,
  Geräte) lehnt **jede** FLOAT ab, auch 2.0. `builtins.rs:414` `need_int` und
  `vm_gui.rs:104` (zweite Kopie `vm_gui.rs:2118`) nehmen eine glatte FLOAT
  (`glatt_int`). `vm_grafik.rs:27` `gi` **schneidet still ab** (`*f as i64`,
  Kommentar beruft sich auf `interpreter.py:3541`). `builtins.rs:458`
  `need_int_gerundet` rundet (RGB, RGBA, Partikel). `vm_system.rs:19` `z_int`
  schneidet ab (alle `ZEIT_*`).
  Wahrheitswerte: `vm_grafik.rs:63` `gb` zählt nur `TRUE`, `1` ist FALSE. Daneben
  `vm_grafik.rs:67` `gflag` (TRUE/FALSE und 1/0), `builtins.rs:1187` `need_bool`
  (nur BOOLEAN), `builtins.rs:1195` `need_flag` (BOOLEAN oder Zahl),
  `builtins.rs:4885`/`:3274`/`:5136` `matches!(…, Bool(true))`, `truthy()`
  bei DIRLIST, `vm_geraete.rs:1230` `an_aus` (GPIO).
- **Beleg (Lauf):**
  ```text
  HTTP_TIMEOUT(10 / 2)        -> HTTP_TIMEOUT: erwartet INTEGER, erhalten FLOAT
  DB_GET_INT(r, 4/2 - 2.0)    -> DB_GET_INT: erwartet INTEGER, erhalten FLOAT
  MID$("abcdef", 4 / 2, 2)    -> cd
  BOX(10.9, 10.9, 50.9, 50.9) -> läuft (abgeschnitten)
  GUI_WINDOW("x", 10.5, …)    -> GUI_WINDOW: erwartet INTEGER (Arg 2), erhalten 10.5 …
  ZEIT_PLUS(t, 1.7) - t       -> 1
  TEXT_BOLD(1)                -> wirkt nicht (TEXT_WIDTH 156 -> 156); TEXT_BOLD(TRUE) -> 157, "fett"
  MOUSE_LOCK(1), IMAGE_FLIP(b, 1, 0)      -> still wirkungslos
  FIRMATA_DIGITAL_WRITE(7, 13, 1)         -> erwartet BOOLEAN, erhalten INTEGER
  SORT(a, 1) / RMDIR(p, 1)                -> Fehler;  DIRLIST(p, "", 1) -> geht
  ```
  `dhrt --check` warnt bei `MOUSE_VISIBLE(1)` nicht. Betroffen von `gb` sind
  rund 20 Befehle (`vm_grafik.rs:371, 468, 484, 961-970, 1021-1031, 1601,
  1613, 2331, 2546, 2804`), dazu `SPRITE_SET_FLIP`, `ASTAR_SET_DIAGONAL`,
  `PARTICLE_SET_FADE`.
- **Vorschlag:** **Ein** Satz Helfer in `vm_helfer.rs` (`pub(super)`), den alle
  Familien nehmen:
  - `arg_int`: glatte FLOAT ja, sonst Fehler mit `KEIN_GANZ_HINWEIS`.
  - `arg_koord`: rundet ausdrücklich, für Pixel.
  - `arg_num`, `arg_text`.
  - `arg_schalter`: BOOLEAN oder Zahl, alles andere ist ein Fehler.

  Die lokalen Kopien entfallen. Bisher abgewiesene Werte werden angenommen, und
  `TEXT_BOLD(1)` wirkt endlich. Beides bricht nichts; still Falsches wird richtig.

### A2 · Fehlermeldungen: fünf Formen, falsche Namen, kein Aufruf-Hinweis — *verwirrt*

- **Wo:**
  - **Form:** Die zentralen Helfer schreiben `NAME erwartet …` ohne Doppelpunkt
    (`builtins.rs:326` `need_str`, `:417-422` `need_int`, `:463`, `:471`, `:1190`,
    `:1200`, alle Handle-Prüfer ab `:5835`). Die Zählung in `builtins.rs` ergibt
    66 Stellen ohne Doppelpunkt und 52 mit. `vm_helfer.rs:17` `mit_signatur`
    hängt „-- Aufruf: …“ nur an, wenn vor dem ersten `:` ein Befehlsname steht.
    Deshalb fehlt die Hilfe gerade bei den häufigsten Typfehlern.
  - **Weitere Formen:** `vm_grafik.rs:75` „Argument 2 muss Zahl sein“,
    `vm_grafik.rs:57` „erwartet STRING“ ohne „erhalten“, `vm_gui.rs` „(Arg 2)“,
    `vm_grafik.rs:27` „(Argument 2)“, `vm_helfer.rs:950` großes „Erwartet“,
    „bekommen“ statt „erhalten“ (`builtins.rs:1943`, `:725`). Sätze ganz ohne
    Befehl: `builtins.rs:1572` „SQR von negativer Zahl“, `:1598`, `:1612`,
    `:637`, `:7110` „Map-Wert: Erwartet …“.
  - **Argumentzahl:** acht Formen: „1..2“, „1 oder 2“, „1 bis 2“, „mind. 2“,
    „Argument(e)“ oder „Argumente“, mit oder ohne „-- Aufruf:“. Der Compiler prüft
    dieselbe Zahl schon aus dem Index, **gibt aber nur eine Warnung aus**, mit
    eigenem Wortlaut.
  - **Falsche Befehlsnamen:** Platzhalter `"S"` (`builtins.rs:4848-4851`,
    `:5115-5129`, `:5187`), 23-mal `"T"` in tiled (`builtins.rs:5407-5563`),
    „PAD“ statt `PADL$` (`:2085`), „WRITE“ statt `WRITELINE` (`:3588`),
    „BASE64_DECODE“ bei `BUFFER_FROM_BASE64`/`DECOMPRESS$` (`:5934`), „TWEEN“
    (`:4473`), „PHYS3D_BODY“ (`:3017`). Die Liste nennt die Klappliste
    (`gui_widgets.rs:774-799`), „MIDI:“ statt Befehl (`vm_geraete.rs:214`).
    `handle_get` setzt das Modul statt des Befehls davor (`vm.rs:4587`, „DB: …“,
    „SERIAL: …“, „FIRMATA: …“).
  - **„Falsches Widget“:** rund 40 Wortlaute, „ungültiges Fenster“ zwei
    (`gui*.rs`, `grep 'Widget ist kein'`).
  - **Englisch oder roh:** `net.rs:141/201/293/326` „port out of range“,
    `audio.rs:1768, 2156` `format!("LOADSOUND: {:?}", e)` gibt Symphonias
    Debug-Text aus, serde-Texte bei `JSON_LOAD`.
- **Beleg (Lauf):**
  ```text
  UPPER$(5)            -> UPPER$ erwartet STRING, erhalten INTEGER        (kein "-- Aufruf")
  CHOICE(5)            -> CHOICE erwartet ARRAY
  ROUND(1.5, 1, 2)     -> Warnung "… erwartet werden 1 bis 2", dann
                          "ROUND: erwartet 1 oder 2 Argument(e), erhalten 3 -- Aufruf: ROUND(x[, digits])"
  RANDF(1)             -> "… -- Aufruf: RANDF(min, max) -- Aufruf: RANDF(lo: Zahl, hi: Zahl)"
  PARTICLE_SET_POS(…)  -> PARTICLE_SET_POS: S erwartet Zahl, erhalten STRING
  TILED_OBJECT_X(m,5,0)-> T erwartet STRING, erhalten INTEGER
  GUI_LISTBOX_SELECTED(b) -> GUI_DROPDOWN_SELECTED: Widget ist kein dropdown/listbox
  NET_TCP_LISTEN(70000)-> NET_TCP_LISTEN: port out of range: 70000
  LOADSOUND (qoa)      -> LOADSOUND: "SymphoniaError(Unsupported(\"core (probe): …\"))"
  ```
- **Vorschlag:**
  - Helfer aus A1 mit **einer** Form schreiben lassen:
    `NAME: erwartet X, erhalten Y (Argument n)`.
  - Die Argumentzahl einmal prüfen. Ist die Signatur sicher, ist der Compiler-Fund
    ein **Fehler**, keine Warnung. Zur Laufzeit erzeugt ein Makro den Satz.
  - Einen Helfer `art_pruefen(w, &[Kind], fn_)` für die gui bauen, einen
    `port_pruefen` für das Netz.
  - Platzhalter durch echte Namen ersetzen.
  - `daten/meldungen.en.txt` mitziehen (`dhrt pruef meldungen`). Wortlaute dürfen
    sich laut `docs/stabilitaet.md` ändern.

### A3 · „Gibt es in diesem Bau nicht“: acht Wortlaute, Export und `MODULE_AVAILABLE` raten falsch — *bricht etwas*

- **Wo:**
  - `vm_helfer.rs:78-83` `fehlt_im_bau` erkennt vier Teilsätze.
  - httpd meldet „dieser dhrt-Build hat das Netz-Feature nicht“
    (`vm_dienste.rs:835`), das trifft keinen davon.
  - smtp und pdf melden „ist ohne das Feature … gebaut“ (`vm_dienste.rs:286`, `:575`).
  - db, net, http, mqtt, ws und cloud fallen mit `Ok(None)` durch zu „Builtin 'X'
    im Rust-Kern noch nicht verfuegbar“.
  - Grafik meldet „… (Schritt 4)“ (`vm_helfer.rs:115`).
  - Hardware meldet „gehoert zum Hardware-Modul … Neu bauen mit: python
    rust\build_runtime.py --hardware“ (`vm_helfer.rs:94-103`). Dort fehlt `midi_`.
  - Weitere Formen: „gibt es nur unter Linux“ (`pins.rs:753`), „Modul WIFI: …“
    (`wifi.rs:550`), „dieser Bau hat keine Grafik“ (`vm_dienste.rs:757`), „in
    diesem Build nicht verfuegbar“ (`vm_grafik.rs:1080`).
  - `preprocess.rs:233-247` `hardware_module_compiled` endet mit `_ => true`.
- **Beleg (Lauf):** `dhrt-mini --fehlende httpd_start httpd_accept` meldet nichts.
  Der Export wählt daraufhin `Schlank: dhrt-mini (16,1 MB statt 43,7 MB)`, und
  das Programm bricht beim ersten `HTTPD_START` ab. Seit 2026.52 ist schlank die
  Vorgabe. In `dhrt-mini` liefert `MODULE_AVAILABLE` TRUE für webcam, video,
  websocket, db, pdf, physics2d, mqtt und net; der erste Befehl scheitert dann.
  `MIDI_IN_COUNT` ohne `--hardware` sagt „im Rust-Kern noch nicht verfuegbar“,
  während die IMPORT-Warnung desselben Laufs richtig „--hardware“ sagt.
- **Vorschlag:** **Eine** Tabelle Modul → Präfixe → Feature (in `preprocess.rs`).
  Aus ihr speisen sich `MODULE_AVAILABLE`, die IMPORT-Warnung, die Meldung
  `NAME: gibt es in dieser Laufzeit nicht (Feature x)` und `fehlt_im_bau`. Alle
  Hüllen rufen dieselbe Funktion. Den Satz „Neu bauen mit python …“ nur im
  Entwicklerbau zeigen. `pruefsammlung.rs:1006` liest den alten Satz eine Fassung
  lang mit.

### A4 · Handles: Start, Freigabe, ungültig, doppelt, Laden — je Modul anders — *bricht etwas*

- **Wo und wie:**

  | Frage | Varianten (Beispiele) |
  |---|---|
  | Was ist ein Handle? | Index ab **0** in einem `Vec` (db, net, pdf, xlsx, smtp, serial, Bilder, Modelle …, `vm.rs:1016-1121`), **echte Werte** (JSON, XML, SPRITE, GELD, VEC2, `value.rs:936`) |
  | Freigabe heißt | `_FREE` (IMAGE, SOUND, VIDEO, MIDI_FILE, SPELL), `_CLOSE` (22), `_REMOVE` (AUDIO_CLOCK/EMITTER/LISTENER/MOD, PHYS2D/3D), `_DESTROY` (ECS, GUI), `_DISCONNECT` (BT, MQTT, WIFI), `_STOP`, `GPIO_RELEASE` |
  | Keine Freigabe trotz Int-Handle | `MODEL_LOAD` (`graphics.rs:3554` `models.push`), `FONT_LOAD` (`graphics.rs:5381`), `SHADER_LOAD` (`graphics.rs:8211`), `RENDERTARGET_NEW`, `ATLAS_LOAD`, `SAMPLE_LOAD` (`audio.rs:2160`, wächst nur) |
  | Ungültig bei Gebrauch | Fehler mit Modul („DB: …“), Fehler mit Befehl („WS_SEND: …“), still (`SHADER_SET(99, …)`, `POSTFX(99)`, `PHYS3D_BODY_X(w, 7)` → 0, `MIDI_FILE_FREE(99)`), -1 (`GETPIXEL(99, 0, 0)`), FALSE (`MQTT_IS_CONNECTED(99)`, `TASK_READY(7)`) |
  | Doppelt freigeben | meist still; **Fehler** bei `IMAGE_FREE`, `VIDEO_FREE`, `WEBCAM_CLOSE`, `WS_CLOSE`, `BT_DISCONNECT`, `WINDOW_CLOSE`, `DB_CLOSE_RESULT` |
  | Laden scheitert | Fehler (LOADIMAGE, LOADFONT, VIDEO_LOAD), **0 ohne Fehler** (`LOADMODEL` fehlende Datei, `graphics.rs:3549`: raylib liefert Ersatz, der Fehlerzweig ist tot), **-1** (`SHADER_LOAD`, `graphics.rs:8210`), FALSE (cloud) |
  | gui | zerstörte Widgets/Fenster werden still angenommen (`gui.rs:4267-4279` `wdg`/`win_mut`/`add_widget` prüfen `alive` nicht); `dock_ref` prüft, `dock_mut` nicht |

- **Beleg (Lauf):**
  ```text
  DIM m AS INTEGER = LOADMODEL("gibtsnicht.obj")  -> 0
  SHADER_LOAD("kaputt {")                          -> -1
  DIM p AS PDF = PDF_NEW(); DIM x AS XLSX = XLSX_NEW("B")
  XLSX_SHEET_COUNT(p)                              -> 1   (PDF-Handle 0 trifft Mappe 0)
  db IS DB_CONN -> FALSE ; j IS JSON_HANDLE -> TRUE ; TYPEOF(db) -> INTEGER
  TASK_READY(7) -> FALSE für immer; TASK_RESULT$(7) -> "noch nicht fertig -- erst TASK_READY(7) fragen"
  GUI_DESTROY(b): GUI_SET_TEXT(b, "tot") ok, GUI_GET_TEXT$(b) = tot
  ```
  Dazu werden Modultypen bei der Zuweisung nicht geprüft: `DIM v AS VEC2 = "x"`,
  `DIM j AS JSON_HANDLE = 5`, `DIM d AS DB_CONN = TRUE` gehen durch
  (`vm_helfer.rs:999`). Streng sind nur GELD, TUPLE, FUNCREF und COROUTINE.
- **Vorschlag:** Eine Handle-Regel schriftlich festlegen und **einen** Helfer
  `handle_get(fn_, art, n)` bauen. Die Regel:
  - Ein ungültiges Handle ist ein Fehler, der den Befehl nennt.
  - Freigeben ist immer still, auch doppelt.
  - Ein Lade-Fehlschlag ist ein Fehler.

  Dazu:
  - Die fehlenden Freigaben nachrüsten: `MODEL_FREE`, `FONT_FREE`, `SHADER_FREE`,
    `RENDERTARGET_FREE`, `ATLAS_FREE`, `SAMPLE_FREE`.
  - Für `_REMOVE`/`_DESTROY`/`_DISCONNECT` neue Namen auf `_FREE` über
    `namen.rs`.
  - Für `TASK_READY`/`SHELL_READY` heißt „unbekannte Nummer“ künftig Fehler.
  - Langfristig einen getaggten Handle-Wert einführen, damit `TYPEOF`, `IS` und
    Verwechslungen stimmen. Das ist eine Änderung für „Was sich ändert“. Vorher
    mindestens eine `--check`-Warnung, wenn ein Handle in den Parameter eines
    anderen Typs geht (die Typen stehen im Index).

### A5 · Zeit: Sekunden, Millisekunden, Bilder, Sechzigstel — und vier Uhren — *bricht etwas*

- **Wo:**
  - **Timeouts:** in Sekunden: `HTTP_TIMEOUT` (`vm_dienste.rs:1176`, INTEGER
    1..600, eigens gegen ms-Vertipper begrenzt!), `SERIAL_TIMEOUT`
    (`vm_geraete.rs:517`, FLOAT, still auf 0..3600 geklemmt, NaN → 0), `BT_SCAN`
    (wirft). In ms: `NET_SET_TIMEOUT` (`vm_dienste.rs:177`), `SMTP_TIMEOUT`
    (`:423`, 0 = Fehler), `MODBUS_TIMEOUT` (`vm_geraete.rs:633`), `USB_READ`,
    `MQTT_FLUSH`, mDNS. Fest eingebaut: WS, MQTT und NET-Verbinden 5 s, db und
    cloud 10 s.
  - **Dauern:** ms bei Fades, `TWEEN_NEW`, `CAMERA_SHAKE`, `TIMER_AFTER`,
    `SPRITE_UPDATE`/`PARTICLE_UPDATE` (über `dt_ms`, `builtins.rs:447`, mit Schutz
    gegen Sekunden). Sekunden bei `PHYS2D_STEP`/`PHYS3D_STEP` (still auf
    0.0001..0.05 geklemmt, `physics2d.rs:104`), `CHART_UPDATE`, `VIDEO_SEEK`,
    `AUDIO_AUTOPAN` (`vm_grafik.rs:1936`) neben `AUDIO_PAN_SLIDE` (ms).
    `SAMPLE_LEN` liefert Sekunden, `SAMPLE_SET_LOOP` will Frames. MIDI-Dateien
    schreibt man in Vierteln und liest sie in Sekunden. Der Charakter-Controller
    rechnet je **Bild** (`controller.rs:48`), die Coyote-Zeit in Bildern. Die
    gui-Metrik `caret_period` zählt in 1/60 s (`gui_geometrie.rs:314`).
  - **Uhren:** `DELTA` ist im Testlauf fest 1/60 (`graphics.rs:7719`), Video folgt
    DELTA. TWEEN, CAMERA_SHAKE und das timer-Modul laufen nach der Wanduhr
    (`builtins.rs:6607`, `graphics.rs:3412`, `timer.rs:42`). `GET_TIME` zählt ab
    dem Fenster, `TIMER`/`MILLIS` ab dem Prozess. In der gui gibt es `takt`,
    `uhr` (Summe von `delta`), `nebel_uhr` und `g.get_time()` direkt an 15 Stellen.
  - **gui im Leerlauf:** Countdowns zählen über `g.delta()` herunter, und darin
    steckt die Wartezeit von `WINDOW_WAIT_EVENTS` nicht (`gui_ueber.rs:552, 566,
    582`, `gui_rechtschreibung.rs:112`, `gui_aussehen.rs:341`, `gui_stil.rs:916`).
- **Beleg (Lauf):**
  ```text
  Toast 1000 ms mit WINDOW_WAIT_EVENTS(5): nach 12 s noch sichtbar (delta je Bild 16 ms)
  SETFPS(0), DHRT_FRAMES: 30 Bilder -> Summe DELTA 0,5 s, MILLIS 82 ms, TWEEN_PROGRESS(500 ms) 0,164
  SLEEP(700) vor SCREEN: TIMER() = 0,99, GET_TIME() = 0,27
  laufendes Band (DHRT_FRAMES=10) in fünf gleichen Läufen bei x = 21, 16, 15, 25, 23
  SLEEP(1.5) -> SLEEP: erwartet INTEGER, erhalten FLOAT   (Index: SLEEP(ms: Zahl))
  ```
- **Vorschlag:**
  - Eine Leitlinie in `docs/builtins-namen.md`: Zeitspannen und Timeouts in **ms**
    (Parameter `ms`/`…_ms`), Schritt-dt in **Sekunden** wie `DELTA`.
  - Für die Ausreißer kommen neue Befehle statt neuer Namen, weil die Einheit
    wechselt: `HTTP_SET_TIMEOUT` in ms neben dem alten `HTTP_TIMEOUT`.
  - Gleiche Einheit, nur Namensregel: `SERIAL_SET_TIMEOUT`, `SMTP_SET_TIMEOUT`,
    `MODBUS_SET_TIMEOUT` über `namen.rs`.
  - `PHYS*_STEP` bekommt den spiegelbildlichen Schutz zu `dt_ms`.
  - Die gui speichert Fristen als Zeitpunkt auf `takt` (`bis = takt + dauer`).
  - Eine Spieluhr aus `Graphics::uhr_jetzt` für TWEEN, SHAKE und timer, solange
    ein Fenster da ist.
  - `g.get_time()` im gui-Code nicht mehr verwenden. `SLEEP` nimmt FLOAT.
  - `CHAR_UPDATE` bekommt ein optionales dt.

### A6 · Koordinaten, Winkel, Rechtecke, Strahlen — *bricht etwas*

- **Wo:**
  - **gui logisch oder Pixel:**
    - Logisch relativ zum Fenster: `GUI_GET_X`, `GUI_TOOLBAR_ITEM_X`.
    - Logisch absolut: `GUI_TEXTAREA_CARET_XY`, `GUI_DOCK_RECT`.
    - **Bildschirmpixel:** `GUI_LISTBOX_ITEM_RECT` (`gui_widgets.rs:1076`),
      `GUI_TREE_ITEM_RECT`, `GUI_TABLE_ROW_RECT`, `GUI_TABCONTROL_ITEM_RECT`,
      `GUI_CANVAS_X/Y/W/H` (`gui_widgets.rs:677`).
    - Ohne Umrechnung: `GUI_RICHTEXT_HEIGHT`/`_SCROLL_GET` (`gui_richtext.rs:122`).
    - Der Splitter liefert und nimmt in `GUI_GET_VALUE`/`GUI_SET_VALUE` Pixel
      (`gui_menues.rs:265, 351-353`).
    - Feste Pixel ohne `sk`, obwohl `gui.rs:4199` „jede feste Pixelzahl läuft
      hier durch“ zusagt: `TABBAR_H` (`gui_geometrie.rs:48`), Panel-Kopf 18
      (`gui_zeichnen.rs:277, 854`), GroupBox (`:1982`), Fenster-Rollbalken 10/24
      (`gui_geometrie.rs:220`), Textbereich-Rand 5 an rund 15 Stellen, Mausrad 40
      (`gui_eingabe.rs:218, 411`).
  - **Winkel:** `ARC` in Bogenmaß gegen den Uhrzeigersinn (`graphics.rs:5144`),
    `PIE` in Grad im Uhrzeigersinn (`graphics.rs:4818`). `PHYS2D_BODY_ANGLE` und
    `VEC2_ANGLE` liefern Bogenmaß, `DRAWIMAGEROT` und `CAMERA_SET_ROTATION`
    erwarten Grad. Die Parameternamen heißen `angle`, `angle_deg`, `degrees`,
    `start_rad`.
  - **Rechtecke:** Ecke–Ecke (BOX, `IMAGE_FILL_RECT`), Ecke+Größe
    (`IMAGE_DRAW_RECT`, SCISSOR), Mitte+halbe Größe (`PHYS2D_ADD_BOX`,
    `physics2d.rs:78`), Mitte+ganze Größe (`RAY_HIT_BOX`, `graphics.rs:3948`).
  - **„RAY“:** In 2D ist es eine Strecke, das Ergebnis der Anteil t∈[0,1]
    (`builtins.rs:6980`). In 3D ist es ein Strahl, das Ergebnis die Entfernung.
    Kein Treffer heißt -1, bei `PHYSICS_LINES_X` dagegen NaN.
  - `SPRITE_COLLIDES`/`SPRITE_HIT_*` ignorieren `SPRITE_SET_SCALE`
    (`builtins.rs:3312`), `SPRITE_DRAW` skaliert (`vm_grafik.rs:2516`).
- **Beleg (Lauf):**
  ```text
  GUI_SCALE(2), Splitter bei x=100: GUI_GET_X 100, GUI_GET_VALUE 200; nach SET_VALUE(120): GUI_GET_X 60
  GUI_SCALE(2): GUI_GET_X(b)=10, GUI_LISTBOX_ITEM_RECT=(220,244,200,44), GUI_CANVAS_X=441
  GUI_SCALE(2): Panel-Titel und Fenster-Reiter laufen sichtbar über ihren Streifen hinaus
  IMAGE_FILL_RECT(b,2,2,3,3): 4 Punkte ; IMAGE_DRAW_RECT(b,2,2,3,3): 9 Punkte
  PHYSICS_RAY_BOX Richtung (20,0): 0.5 ; RAY_HIT_BOX Richtung (2,0,0): 10.0
  Sprite 10x10 mit Skala 3, Nachbar bei x=15: SPRITE_COLLIDES -> FALSE
  ```
- **Vorschlag:**
  - gui: In `value()`/`set_value()` des Splitters `unsk`/`sk` verwenden. Für
    Pixel-Abfragen neue Namen mit `_SCREEN_` einführen und daneben logische
    anbieten. Alle festen Pixelzahlen bekommen benannte Konstanten und laufen
    durch `sk`.
  - Leitlinie für Winkel festlegen: Zeichnen in Grad im Uhrzeigersinn, Mathe in
    Bogenmaß. Für die Ausreißer kommen neue Befehle wie
    `PHYS2D_BODY_ANGLE_DEG`.
  - Die Rechteck-Konvention je Modul in der Doku in einem Satz nennen.
  - Die Sprite-Kollision rechnet mit Skala; das war bisher still falsch.

### A7 · Farbe und Deckkraft — *verwirrt*

- **Wo:**
  - `SPRITE_TINT` und `PARTICLE_SET_COLOR` lehnen ein Alpha-Byte ab
    (`builtins.rs:3283`, `:5130`). Überall sonst gilt `col()`
    (`graphics.rs:2194`): oberstes Byte = Deckkraft, 0 = deckend.
  - Die Regel steht viermal im Code: `col`, `vm_grafik.rs:86` `lw_farbe`,
    `chart.rs:370` `alpha_of`, `builtins.rs:2043` ALPHA.
  - `CLS` im transparenten Fenster nimmt das Byte wörtlich (`graphics.rs:4695`).
  - In der gui wirft `mischen` die Deckkraft weg (`gui.rs:328`, setzt nur drei
    Kanäle zusammen), `shade` behält sie seit 2026.50.
  - Die Stildatei hat zwei Farbleser (`gui_stil.rs:594` gegen
    `gui_aussehen.rs:131`), die `.dhform` liest `#FFF` als 0x000FFF
    (`gui_layout.rs:869`).
- **Beleg (Lauf):** `SPRITE_TINT(sp, RGBA(255,0,0,128))` →
  „SPRITE_TINT: Farbe muss 0..0xFFFFFF sein“. Ein Glasknopf
  (`bg: #20FF0000`, Übergang zum Fokus) ist in der Mitte des Übergangs deckend
  (Pixel 255,9,9 statt 53,30,40).
- **Vorschlag:** Eine Funktion `farbe::rgba(i64) -> [u8; 4]` (in `farbraum.rs`),
  die alle nehmen. Die Bereichsprüfung bei Sprite und Partikel öffnen (nimmt nur
  mehr an). `mischen` interpoliert auch Alpha. `@ablauf` und `.dhform` lesen über
  `farbe_lesen`.

### A8 · Ton: Lautstärke, Pan, Tonhöhe, „Kanal“ — *verwirrt*

- **Wo:**
  - **Pan:** `AUDIO_PAN_POS`/`_SLIDE`/`AUTOPAN` 0..1 (`audio.rs:634`),
    `AUDIO_BUS_PAN`, `AUDIO_MODULATE`, `AUDIO_SOUND_MIX` -1..1 (`audio.rs:1380,
    1365, 1950`). `AUDIO_PAN(ch, l, r)` setzt nebenbei die Lautstärke
    (`audio.rs:2356`).
  - **Lautstärke:**
    - Über 1 still geklemmt (`audio.rs:1108, 1578, 2338, 2548`).
    - `AUDIO_SOUND_MIX` 0..4, darüber ein Fehler (`vm_grafik.rs:2026`).
    - `GUI_SOUND` in Prozent 0..200, über 100 wirkungslos (`vm_gui.rs:2054`).
    - Erzeugte Klänge backen `vol` ein **und** merken es als Abspiel-Lautstärke
      (`audio.rs:1798-1800`).
  - **Tonhöhe:** `AUDIO_PITCH` ohne Obergrenze, `AUDIO_MUSIC_PITCH` still auf 64
    geklemmt (`audio.rs:2553`), `SPEAK_RATE` außerhalb 0.5..2 ein Fehler. `sus`
    heißt bei `AUDIO_SFX` eine Zeit, bei `AUDIO_NOTE` ein Pegel (`audio.rs:1830`).
  - **„Kanal“:** Ein Kanal ist der Klang selbst (`audio.rs:713`), ein zweites
    `AUDIO_PLAY` schneidet das erste ab (`audio.rs:1104-1127`). `AUDIO_PLAY_AT`
    und `SPEAK` schneiden nicht ab (`audio.rs:1134`, `:1053`).
  - **Fehlende Argumente:** Bei `AUDIO_DELAY` setzt ein fehlendes `feedback` auf
    die Vorgabe zurück, ein fehlendes `time_ms` bleibt unverändert
    (`audio.rs:1701`). `PLAYMUSIC` ohne `volume` setzt die Musiklautstärke auf 1
    zurück.
- **Beleg (Lauf):** `AUDIO_BUS_VOLUME("sfx", 2.0)` → 1.0;
  `AUDIO_SOUND_MIX(…, vol 5)` → Fehler; `AUDIO_MUSIC_PITCH(100)` → 64;
  `SPEAK_RATE(3)` → Fehler. Zweimal `AUDIO_PLAY` desselben Klangs: BUSY 1,
  und `AUDIO_STOP(a)` stoppt auch b.
- **Vorschlag:**
  - Eine Skala für Pan (-1..1, wie Kira) über neue Befehle, etwa
    `AUDIO_SET_PAN`. Kein neuer Name, weil die Skala wechselt.
  - Eine Regel für Lautstärke: 0..1, Überschreitung ist ein Fehler, oder überall
    Verstärkung bis 4.
  - Erzeugte Klänge legen den Slot mit Lautstärke 1 an.
  - Eine mehrstimmige Abspielform (etwa `AUDIO_PLAY_VOICE` oder ein optionaler
    hinterer Parameter).
  - Regel „fehlend = Vorgabe“ und alle Vorgaben in den Index schreiben.

### A9 · Zahlen: dreimal `coerce`, Überlauf mal gemeldet, mal gesättigt — *bricht etwas*

- **Wo:**
  - `vm_helfer.rs:933` `coerce` (Variablen, Parameter, Index) prüft FLOAT→INTEGER
    auf den Wertebereich und ist bei GELD streng. `builtins.rs:568` `coerce_elem`
    (ARRAY_PUSH/FILL/INSERT) und `builtins.rs:7104` `coerce_map_value` (MAP)
    machen `f as i64` und reichen jeden Wert zu GELD durch. Das sind drei
    Wortlaute.
  - `INT` meldet Überlauf (`builtins.rs:1273`). `FLOOR`, `CEIL` und `ROUND`
    sättigen still (`:1660-1664`, `:990`).
  - `VAL` (`builtins.rs:7036`) kennt `&H`/`0x` und nimmt den Anfang einer Zahl.
    `ISNUMERIC`/`TRYVAL` (`parse_number`, `:555`) kennen kein Hex, nehmen aber
    „inf“ und „NaN“.
- **Beleg (Lauf):**
  ```text
  ARRAY_PUSH(a, 1e19) -> 9223372036854775807 ; m["x"] = 1e19 -> 9223372036854775807
  DIM x AS INTEGER = 1e19 -> Ganzzahl-Ueberlauf
  ARRAY_PUSH(gs, 5) bei ARRAY OF GELD -> geht ; DIM g AS GELD = 5 -> Fehler
  FLOOR(1e19) -> 9223372036854775807 ; INT(1e19) -> Überlauf-Fehler
  VAL("&HFF")=255, ISNUMERIC("&HFF")=FALSE ; VAL("inf")=0, ISNUMERIC("inf")=TRUE
  ```
- **Vorschlag:** `coerce_elem` und `coerce_map_value` durch `vm_helfer::coerce`
  ersetzen. Ein gemeinsamer Helfer für f64 → i64, der Überlauf meldet (INT,
  FLOOR, CEIL, ROUND, ROUND_HALF_UP). `parse_number` lehnt inf und NaN ab und
  kennt `&H`/`&B` wie VAL.

### A10 · Datum und Zeit: zwei Kalender, zwei Wege zur Ortszeit — *bricht etwas*

- **Wo:**
  - `FILETIME` rechnet mit dem Zeitzonenversatz von **heute**
    (`builtins.rs:3662` → `ortszeit_versatz`, `:6140`). `TIME_FROM_UNIX` nimmt
    den vom Tag selbst (`lokaler_versatz`, `builtins.rs:247`).
  - `ZEIT_AUS_TEILEN` prüft nichts (`vm_system.rs:92` → `zeit.rs:53`),
    `ZEIT_PARSE` prüft alles (`zeit.rs:156`).
  - Schaltjahr, Monatslänge, Wochentag und Datumsleser stehen doppelt in
    `kalender.rs:14-56` und `zeit.rs:84-174`, mit Montag = 0 gegen Montag = 1 und
    verschieden strengem Leser.
- **Beleg (Lauf):** Datei vom 2026-07-01 12:00, gelesen im Winter:
  `FILETIME` = 11:00, `TIME_FROM_UNIX` = 12:00. `TIME_FROM_PARTS(2026, 13, 1)`
  ergibt 2027-01-01, `ZEIT_AUS_TEILEN(2026, 2, 31)` den 2026-03-03.
- **Vorschlag:** `FILETIME` über `lokaler_versatz`. `TIME_FROM_PARTS` prüft wie
  `iso_lesen`. `kalender.rs` ruft für Schaltjahr, Monatslänge und Wochentag
  `zeit.rs` auf.

### A11 · Dateien lesen: zwei Wege für Pfade, zwei für Kodierung, vier Fehlerformen — *bricht etwas*

- **Wo:**
  - `resolve_asset_path` (`builtins.rs:30`) nutzen FILEEXISTS, OPENFILE,
    FILESIZE, READALL$, JSON_LOAD und save_load. **Nicht** nutzen ihn COPYFILE
    (`:3419`), DELETEFILE, RENAME, DIREXISTS (`:3601`), DIRLIST,
    SAVE_EXISTS (`:4815`), SAVE_LOAD_OR_NEW (`:4819`) und `ZIP_*`.
  - `text_lesen` (`:6276`, BOM weg, `encoding$`) nutzen READALL$, READLINES,
    CSV/XML/INI_LOAD. **Nicht** nutzen ihn JSON_LOAD (`:4644`) und save_load
    (`:6049`, `read_to_string`). ZIP_READ$ ersetzt still durch U+FFFD (`:3856`),
    BUFFER_TO_STRING$ lehnt ab.
  - Dateifehler mit Pfad, ohne Pfad, „(pfad)“ hinten oder mit Suchort
    (`datei_da`, `:52`).
- **Beleg (Lauf):**
  ```text
  FILEEXISTS("../../z1.txt") = TRUE ; COPYFILE("../../z1.txt", …) -> Datei nicht gefunden
  SAVE_EXISTS = FALSE, SAVE_LOAD findet Version 3, SAVE_LOAD_OR_NEW legt Version 1 an
  JSON_LOAD (Datei mit BOM) -> JSON_LOAD: bom.json: expected value at line 1 column 1
  ZIP_READ$ (latin-1)       -> 'caf�'
  COPYFILE: Das System kann die angegebene Datei nicht finden. (os error 2)   <- welcher Pfad?
  ```
- **Vorschlag:** Jeder lesende oder prüfende Zugriff geht durch denselben
  Pfad-Helfer. JSON_LOAD und save_load lesen über `text_lesen`; JSON_LOAD bekommt
  hinten ein optionales `encoding$`. Ein Helfer `datei_fehler(fn_, pfad, e)`.
  `ZIP_READ$` meldet ungültige Bytes und nimmt ein optionales `encoding$`.

### A12 · Zählen ab 0 oder 1, „nichts“ als -1, 0 oder NIL — *verwirrt*

- **Wo:**
  - `LBOUND`/`UBOUND` zählen Dimensionen ab 1 (`builtins.rs:1502`),
    `DIMSIZE`/`ARRAY_DIM_SIZE` ab 0 (`:1540`).
  - `PDF_PREVIEW` zählt Seiten ab 1 (`vm_dienste.rs:788`), sonst wird überall ab
    0 gezählt.
  - `GUI_TEXTAREA_VIEW` zählt Zeilen ab 0 (`gui_eingabe.rs:1923`), der Rest des
    Textbereichs ab 1.
  - Das Kreuz der Infobar ist in `GUI_INFOBAR_ANSWER` -1, in
    `GUI_INFOBAR_BUTTON_RECT` 0.
  - Annehmen ohne Gast liefert NIL (`NET_TCP_ACCEPT`), -1 (`WS_ACCEPT`) oder
    FALSE (`HTTPD_ACCEPT`).
  - Eine Auswahl außerhalb des Bereichs wird still zu -1 (Liste, Baum, Tabelle),
    still ignoriert (`GUI_SET_ACTIVE_TAB`), ist ein Fehler (Reiterwerk,
    Assistent) oder liefert still `""` (`GUI_TABCONTROL_TITLE$`).
  - `INSTR` klemmt einen negativen Start auf 0, `INSTRREV` liefert -1.
- **Beleg (Lauf):** `DIMSIZE(g, 1) = 4`, `LBOUND(g, 0)` → „Dimension 0 gibt es
  nicht (gezaehlt ab 1)“. Nach `GUI_TEXTAREA_SCROLL(ta, 3)` liefert VIEW 2.
  `GUI_LISTBOX_SET_SELECTED(lb, 99)` → still -1.
- **Vorschlag:** Nicht umdrehen, das würde bestehende Programme brechen. Die Regel
  „Elemente ab 0, -1 = keins; Dialogantworten ab 1, 0 = keine“ schriftlich
  festhalten. Die Ausreißer in der Doku und in Meldungen nennen. Neue
  Abfragen bekommen die Regel, etwa eine 1-basierte Textbereichs-Sicht.

### A13 · Bytes und Kodierung bei Geräten und Netz — *bricht etwas*

- **Wo:** `SERIAL_*` kodiert UTF-8 verlustbehaftet (`serial.rs:41-64`,
  `text_stream.rs:29`). `USB_*`/`BT_*` nehmen latin-1 (`usb.rs:26`, `bt.rs:44`,
  wörtlich gleich, ein dritter Weg in `kodierung.rs:155`). Pins und Firmata nehmen
  ein Feld oder BUFFER. `MQTT_PUBLISH` und `HTTPD_SEND`/`HTTPD_BODY$` nur Text
  (`vm_dienste.rs:1031, 880-907`). `NET_SEND` und `WS_SEND` nehmen BUFFER.
- **Beleg (Code):** Ein empfangenes Byte 0xFF wird bei SERIAL zu U+FFFD
  (`ASC` = 65533), `CHR$(200)` geht als zwei Bytes hinaus. Der Rat in
  `docs/module-serial.md:45`, Binärdaten über MID$/ASC zu lesen, funktioniert
  daher nicht.
- **Vorschlag:** Byte-Befehle überall gleich: `SERIAL_WRITE_BYTES`,
  `SERIAL_READ_BYTES` (BUFFER), dasselbe Paar für USB/BT, BUFFER bei MQTT und
  HTTPD. Ein `bytes_arg`, latin-1 nur aus `kodierung.rs`.

### A14 · Optionswörter: deutsch, englisch oder beides — je Befehl — *verwirrt*

- **Wo:**
  - Die Wellenform nimmt bei `AUDIO_LFO_NEW` auch „sinus“, „dreieck“ und „puls“
    (`audio.rs:1184-1187`). `AUDIO_TONE/NOTE/SFX` nehmen nur Englisch, mit
    „noise“, aber ohne „pulse“ (`audio.rs:2688, 2704, 2716`): drei Parser.
  - Die chart-Schlüssel sind nur deutsch (`chart.rs:2632-2758` „titel“,
    „deckkraft“, „zifferblatt“ …), die Diagrammart zweisprachig (`chart.rs:38`).
  - Die Stildatei ist zweisprachig (`gui_stil.rs:754-792`).
  - Die gui nimmt zweisprachig an und gibt deutsch aus (`GUI_SET_ALIGN(l,
    "right")` → `GUI_GET_ALIGN$` = „rechts“). Knopfart, Form und Sinnbilder
    nehmen nur Deutsch, `GUI_KIND` liefert Englisch. Die Schlüssel-Setter kennen
    teils keinen englischen Namen (`GUI_TEXTAREA_SET` „zeilennummern“,
    `gui.rs:5153`).
  - `TIME_PART(t, "year")` scheitert, nur „jahr“ geht (`zeit.rs:331`).
    `TIME_FORMAT$ %a` = „Fr“.
  - `KEY_NAME$` ist auch mit `DHRT_LANG=en` deutsch („LEER LINKS UMSCHALT“,
    `graphics.rs:10964`).
  - `MIDI_NOTE_NAME$(71)` = „H4“. Die Loop-Art heißt vorgegeben „vorwaerts“.
  - `BUFFER_DEFLATE` nimmt „roh“|„raw“, `FILE_ENCODING$` liefert „binaer“, das
    keine Kodierung annimmt.
  - Die eingebauten Knöpfe sind fest deutsch („Groesse“ sichtbar ohne Umlaut,
    `gui_bausteine.rs:730`).
- **Beleg (Lauf):** `AUDIO_LFO_NEW("sinus")` geht, `AUDIO_TONE("sinus")` →
  „unbekannte Waveform“. `TIME_PART(t, "year")` → „ZEIT_TEIL: 'year' ist kein
  Feld -- moeglich sind jahr, monat …“, auch mit `DHRT_LANG=en`.
- **Vorschlag:** Regel „Eingang immer zweisprachig, englisch zuerst; Ausgang wie
  bisher“ (der Ausgang bleibt wegen Stabilität). Je eine Tabelle für Wellenform,
  Bus, Ausrichtung und Zeitteil. chart-Schlüssel bekommen englische Zweitnamen.
  Knopf-Texte und `KEY_NAME$` richten sich nach `meldung::sprache`.

### A15 · Netz: drei HTTP-Wege, sechs Arten zu verbinden, zwei URL-Dekodierer — *bricht etwas*

- **Wo:**
  - NET, WS, MQTT und SMTP nehmen nur die **erste** Adresse eines Namens
    (`net.rs:183`, `websocket.rs:231`, `mqtt.rs:350`, `smtp.rs:484`). MODBUS, DB
    und HTTP probieren alle (`db.rs:859` beschreibt genau diese Falle).
  - `HTTP_DOWNLOAD` baut eine eigene Anfrage (`html.rs:206`) ohne
    `HTTP_SET_HEADER`/`HTTP_TIMEOUT` und lässt `HTTP_BYTES` auf dem alten Rumpf.
  - `cloud` ist ein dritter HTTP-Weg (`cloud.rs:12-98`: eigener UA, 10 s, keine
    Größengrenze, IDs ungeprüft im Pfad).
  - Zwei URL-Dekodierer: `httpd.rs:187` `prozent_auf` gegen `html.rs:248`
    `url_decode`, mit verschiedener `+`-Regel. Der erste **stürzt ab**, siehe
    Einzelfehler.
  - TLS-Client viermal (`websocket.rs:211`, `mqtt.rs:295`, `smtp.rs:385`,
    `db.rs:970`), `enum Strom` dreimal.
  - Hintergrund-Aufträge zweimal (`html.rs:521` `Abrufe` gegen
    `hintergrund.rs:31`, dessen Kopf sagt, er ersetze genau das).
  - `_PENDING` hat vier Bedeutungen (WS eingehend, MQTT unbestätigt ausgehend,
    HTTP und DB offene Aufträge).
- **Beleg (Lauf):** `NET_TCP_LISTEN(0)` + `NET_TCP_CONNECT("localhost", port)` →
  „Es konnte keine Verbindung hergestellt werden“ (Windows: `::1` zuerst); ebenso
  WS. `HTTP_GET_START("http://localhost:…")` erreicht den eigenen
  `HTTPD_START(0)`. Nach `HTTP_SET_HEADER("X-Probe", "ja")` sieht der Server sie
  bei `HTTP_GET`, nicht bei `HTTP_DOWNLOAD`. `CLOUD_LOAD("a/b?c=1")` fragt
  `/save/a/b` an. `HTTPD_QUERY$` liefert „a b c“, `URL_DECODE` dagegen „a+b c“.
- **Vorschlag:**
  - Ein `verbinden(host, port, frist, fn_)` nach dem Muster aus `modbus.rs`.
  - `HTTP_DOWNLOAD` und cloud über `html::http_request`, cloud mit `url_encode`
    für die Pfadteile.
  - Ein URL-Dekodierer auf Bytes.
  - `tls.rs` mit einem `Strom`; `html::Abrufe` → `hintergrund::Auftraege`.
  - Neue Namen für `MQTT_PENDING` (etwa `MQTT_UNACKED_COUNT`).

### A16 · Gerätelisten und Kindprozesse — *verwirrt*

- **Wo:**
  - **Gerätelisten in fünf Formen:** STRING mit „, “ (`SERIAL_PORTS`,
    `serial.rs:26`), Zeilen mit `|` (`USB_LIST`, `BT_SCAN`, `WIFI_SCAN`),
    Anzahl+Index (MIDI, mDNS) oder ARRAY (`WEBCAM_DEVICES`, `MIC_DEVICES`,
    `GPIO_CHIPS`, `HOTPLUG_LIST`, `PRINTERS`).
  - Geöffnet wird per Name, Index, Teiltext, VID/PID oder Scan-Adresse.
  - `USB_OPEN_PATH` hat keine Quelle: kein Befehl liefert einen HID-Pfad
    (`usb.rs:42-80`).
  - **Fünf Wege zum Kindprozess:** SHELL, SHELL_OUT$ und SHELL_START vererben
    `DHRT_FRAMES`/`DHRT_SCREENSHOT`. Nur PROCESS_* und WINDOW_OPEN entfernen sie,
    und die Liste dafür steht doppelt (`prozess.rs:73`, `fenster.rs:160`). Felder
    als Argumente und „dhrt“ = eigene Laufzeit gibt es nur bei PROCESS_*.
- **Beleg (Lauf):** Mit `DHRT_FRAMES=7` sieht ein Kind von `SHELL_OUT$` den Wert
  7, eines von `PROCESS_START` nicht. `SHELL_OUT$("cmd", args)` →
  „SHELL_OUT$: erwartet STRING (Arg 2)“.
- **Vorschlag:** Neue Listenbefehle als ARRAY OF STRING (`SERIAL_DEVICES`,
  `USB_DEVICES` mit Pfad, `MIDI_IN_DEVICES` …); die alten bleiben. SHELL* und
  TASK_START bauen über `prozess::befehl()`, mit einer Liste der
  Testvariablen.

### A17 · Eingabe: zwei Tastencode-Welten, zwei Knopf-Nummerierungen — *bricht etwas*

- **Wo:**
  - `WAITKEY`/`KEY_WAIT` liefern **raylib**-Codes (`graphics.rs:7058-7066`, im
    Kommentar zugegeben). `KEY_ANY_HIT` und alle `KEY_*`-Konstanten sind
    SDL/pygame-Codes (`KEY_LEFT` = 1073741904).
  - `JOY_BUTTON_*` (-100 …, `vm.rs:4660`) gelten in KEYPRESSED/INPUT_BIND
    (`graphics.rs:6920`). In KEYHIT/KEYRELEASED/KEYREPEAT gelten sie nie
    (`graphics.rs:7227`), in JOYSTICK_BUTTON auch nicht (rohe raylib-Nummern
    0..17, `graphics.rs:7104`), obwohl `docs/builtins-grafik.md:1031` die
    Konstanten gerade dort anbietet.
  - `INPUT_BIND` nimmt keinen Buchstaben (`vm_grafik.rs:2397`), KEYHIT schon
    (`vm_grafik.rs:45` `taste`).
- **Beleg (Code; Lauf für INPUT_BIND):** `IF WAITKEY() = KEY_LEFT` ist nie wahr.
  `INPUT_BIND("jump", "z")` → „erwartet Zahl, erhalten STRING“.
- **Vorschlag:** WAITKEY gibt Sondertasten über `dh_key_code` aus, Buchstaben wie
  bisher. `key_test` und `joy_btn_enum` nehmen die negativen Codes an.
  INPUT_BIND liest über `taste()`. Alles betrifft nur Fälle, die heute still
  FALSE liefern oder scheitern.

### A18 · gui-Bausteine mehrfach gebaut — *verwirrt*

- **Wo:**
  - **Rollbalken:** sieben Bauarten (Breite 10 roh, `sk(8)`, 5 roh, `sk(12)`,
    `sk(4)`). Gespiegelt werden nur manche; Richtext und Akkordeon sind eine
    wörtliche Kopie (`gui_zeichnen_teile.rs:191`, `gui_zeichnen.rs:2029`).
  - **Mausrad:** sieben Schrittweiten (`gui_eingabe.rs:218, 411, 417`,
    `gui_textbereich.rs:1214, 856, 1201` …).
  - **Kürzen mit „...“:** fünf Wege (`gui_bausteine.rs:76`, `gui_menues.rs:963`,
    `gui_ueber.rs:752`, `gui_zeichnen.rs:2255`, `gui_palette.rs:302` mit „…“).
    Nur einer schützt vor Riesentexten.
  - **Popup-Listen:** sechs mit eigenen Farben und Schatten.
  - **„Tippen springt“** viermal kopiert (`gui_bedienung.rs:465, 982`,
    `gui_richtext.rs:1256`, `gui_widgets.rs:1428`).
  - **Doppelklick:** zwei Erkenner (`DBL_TIME` mit Pixelabstand gegen dreimal das
    Literal 0.4 mit ±2 Zeichen).
  - Pfeiltasten mal mit `key_repeat`, mal mit `key_pressed`.
  - **Schlüssel-Setter `GUI_<W>_SET`** je Widget mit anderen Regeln:
    Mehrfachauswahl heißt „multi“ oder „multi_select“, Akkordeon und Assistent
    unterscheiden Groß/klein, Zahl oder BOOLEAN wechselt. Nur drei der 21 haben
    eine Abfrage, mit drei Rückgabetypen.
  - **Platzhalter** in vier Formen. Dem einfachen Baum fehlen COUNT, REMOVE,
    SET_LABEL und EXPANDED.
  - **Farbrollen:** `GUI_SET_COLOR` kennt „hover“ und „pressed“,
    `GUI_GET_COLOR` nicht; Groß/klein je Richtung anders.
- **Beleg (Lauf):** `GUI_TREE_SET(tr, "multi_select", 1)` → Fehler,
  `GUI_LISTBOX_SET(lb, "multi_select", 1)` geht. `GUI_ACCORDION_SET(acc,
  "Mehrere", 1)` → Fehler. `GUI_SET_COLOR(b, "hover", …)` geht,
  `GUI_GET_COLOR(b, "hover")` → Fehler.
- **Vorschlag:** Je eine Funktion `rollbalken`, `rad_schritt`, `kuerzen`,
  `popup_liste_zeichnen`, `tipp_suche` und ein Klickzähler. Ein gemeinsamer
  Helfer für Schlüssel-Setter: Schlüssel klein, englischer Zweitname, Wert als
  Zahl oder BOOLEAN, dazu je ein `GUI_<W>_GET`.

### A19 · Ton-Wege: raylib-Annahmen auf Kira — *verwirrt*

- **Wo:**
  - `PLAYSOUND`/`SOUND_PLAY` und `PLAYMUSIC`/`MUSIC_PLAY` übergehen `loops`
    (`vm_grafik.rs:1817-1848`, „raylib-Sounds loopen nicht“).
  - Endliche Loops, AUTOPAN und `AUDIO_MUSIC_QUEUE` laufen nur in `update()`,
    das nur FLIP und SPEAK_WAIT rufen (`vm_grafik.rs:797`, `audio.rs:2632`).
  - `AUDIO_MUSIC_QUEUE` prüft den Pfad nicht und verschluckt Fehler
    (`audio.rs:2620, 2673`).
  - `SPEAK_SOUND` gibt einen **geteilten** Slot aus dem Vorrat. Gleicher Text
    liefert dasselbe Handle, und nach 64 weiteren wird er still freigegeben
    (`sprache.rs:88-154`).
  - Zwei WAV-Schreiber (`audio.rs:2100`, `mikrofon.rs:170`), Wellenform-Mathe
    dreimal (`audio.rs:2686-2722`). `AUDIO_PEAK` und `MIC_LEVEL` messen dieselbe
    Größe.
- **Beleg (Lauf):** `PLAYSOUND(k, -1, 0.0)` → nach 250 ms spielt nichts mehr,
  `AUDIO_PLAY(k, -1, 0.0)` spielt. `AUDIO_PLAY(k, 3)` ohne FLIP spielt einmal.
  Nach 70 `SPEAK_SOUND` meldet ein früheres Handle „wurde bereits mit UNLOADSOUND
  freigegeben“.
- **Vorschlag:** `loops` durchreichen (Kira kann es). Endliche Loops über Kira
  statt `update()`, oder `update()` auch aus SLEEP und jedem Ton-Befehl.
  `SPEAK_SOUND` liefert eine Kopie. QUEUE prüft wie LOAD. Ein WAV-Schreiber.

### A20 · Werkzeug-Rechnungen doppelt — *bricht etwas* (Debugger), sonst *innen*

- **Wo:**
  - Der Debugger rechnet selbst: `dbg_binop`/`dbg_arith`/`dbg_truthy`
    (`vm.rs:796-856`), statt `vm::konstant_rechnen` zu nehmen. Das verstößt gegen
    die Regel „nie eine zweite Rechnung“.
  - Vier Wege für „erbt A von B“ (`value.rs:714, 736`, `vm.rs:1633, 1676`).
  - Zwei Typnamen-Funktionen (`compiler.rs:1555` `typ_klartext` nicht rekursiv,
    `vm_helfer.rs:1016` `typ_lesbar`).
  - `builtin_index.json` wird viermal geparst (`compiler.rs:547, 568, 601, 838`).
  - Zwei Signaturquellen für „-- Aufruf:“ (`builtins.rs:1219`
    `builtin_signature` neben dem Index).
- **Beleg (Lauf, `dhrt debug`):** `"apfel" < "birne"` ist dort FALSE (VM: TRUE),
  `2^53 + 1 + 0` verliert die letzte Stelle, ein Überlauf sättigt. Ein bedingter
  Haltepunkt `name < "m"` hält deshalb nie. `--check` meldet
  „ARRAY OF ARRAY:INTEGER“ (interne Schreibweise), die Laufzeit
  „ARRAY OF ARRAY OF INTEGER“.
- **Vorschlag:** `dbg_*` auf `wert_rechnen`/`Value::truthy` umstellen. Für die
  übrigen Fälle je eine Quelle; `builtin_signature` entfällt.

---

## B. Namen

### B1 · Neue Namen melden sich mit dem alten — *verwirrt*

- **Wo:** `compiler.rs:879-900` (`builtin_kanonisch`), `namen.rs:10-13` („die
  Laufzeit muss von den neuen Namen nichts wissen“). Fest verdrahtet sind
  außerdem `graphics.rs:6815` „IMAGEWIDTH“, `ffi.rs:1468` „TEXT_AUS_ZEIGER$“ und
  die Hinweise „mit GELD_NEU(…)“ (`vm_helfer.rs:992`), „OPENFILE“, „UNLOADSOUND“.
- **Beleg (Lauf):**
  ```text
  FILE_EXISTS(5)  -> FILEEXISTS erwartet STRING, erhalten INTEGER
  FILE_COPY(…)    -> COPYFILE: Das System kann …
  FONT_SET(99)    -> SETFONT: ungueltiges FONT-Handle 99
  SOUND_PLAY(999) -> PLAYSOUND: ungueltiges Handle 999
  MUSIC_PLAY(x)   -> AUDIO_MUSIC_LOAD: Datei …
  GUI_GET_VALUE(b)-> GUI_VALUE: …
  TIME_PART(…)    -> ZEIT_TEIL: …
  ```
- **Vorschlag:** Die Aufrufstelle merkt sich den geschriebenen Namen (wie
  `Instr::familie`). Beim Melden ersetzt **eine** Stelle das führende `ALT:`
  durch `NEU:`, an der Grenze, an der heute schon `mit_signatur` greift. Die
  Befehle selbst ändern sich nicht. Feste Hinweise schreibt man auf die neuen
  Namen um.

### B2 · Zwei Wege zu einem zweiten Namen — *innen* (Parameter: *verwirrt*)

- **Wo:** Neben `namen.rs` stehen Doppelnamen direkt im Match-Arm:
  `vm_grafik.rs:776` `"imagewidth" | "image_width"`, `:2378`
  `"atlas_draw" | "batch_draw"`, `builtins.rs:3312`
  `"sprite_collides" | "sprite_collide"`, `vm_grafik.rs:1910/2119`
  `audio_set_volume`/`audio_music_set_volume`, `builtins.rs:3578`
  `"writeline" | "write"`, `builtins.rs:3496`/`:3564` EOF und ENDOFFILE (zwei
  Arme, gleiche Arbeit). `docs/builtins-core.md:1682` führt
  `AUDIO_SET_VOLUME` als „Alias“ des alten, „kanonischen“ Namens, also umgekehrt
  zur Leitlinie.
- **Beleg (Index):** `SPRITE_COLLIDE(a, b)` neben `SPRITE_COLLIDES(sp1, sp2)`,
  `AUDIO_SET_VOLUME(ch, v)` neben `AUDIO_VOLUME(ch, volume)`: derselbe Befehl
  mit anderen Parameternamen. Der Hover kennt den Bezug nicht, weil nur
  `namen.rs` `lsp::builtin_doku` speist.
- **Vorschlag:** Die Arm-Aliase nach `namen.rs` ziehen und die Parameternamen der
  Paare angleichen. Die Richtung in `builtins-core.md` umdrehen.

### B3 · Alte Namen ohne neuen Namen — *verwirrt*

- **Wo:**
  - Deutsch ohne englischen Namen: `ZAHL$` (`builtins.rs:1692`), `EURO$`
    (`:1708`), `CENT` (`:1678`), `BUFFER_ZEIGER` (`ffi.rs:1466`).
  - Ohne Bereich: `STARTSWITH`, `ENDSWITH`, `CONTAINS`, `COUNT` (`:2540`),
    `NOSPACES$`, `TEMPDIR$`, `TEMPFILE$`, `CWD$`, `CHDIR`, `MKDIR`, `RMDIR`,
    `RENAME`, `EXEPATH$`, `LOADFONT_SDF`, `GRADIENTH`/`GRADIENTV`, `TEXTROT`,
    `BOXROT`, `LINEDASHED`.
  - `IS_NIL` steht neben `ISNUM`/`ISINT`/`ISSTR`/`ISBOOL` und `ISNUMERIC`.
  - Datei mit Handle: Für `EOF`, `WRITE`, `READ_BYTES`, `WRITE_BYTES`, `SEEK`
    und `TELL` gibt es keinen neuen Namen, und die naheliegenden sind schon
    belegt (`FILE_WRITE` = WRITEALL, `FILE_READ_BYTES` = READALL_BYTES).
  - `READALL` (ohne `$`) fehlt in `namen.rs`, `READALL$` steht drin.
- **Vorschlag:** Neue Namen über `namen.rs`: `IS_NUM` …, `TEXT_STARTS_WITH` …,
  `DIR_TEMP$`, `DIR_CURRENT$`, `DIR_CREATE`, `FILE_RENAME`, `FONT_LOAD_SDF`,
  `BUFFER_POINTER`, `NUMBER_TEXT$` (ZAHL$). Für Dateien mit Handle **jetzt** eine
  freie Gliederung festlegen (etwa `FILE_SEEK`, `FILE_TELL`, `FILE_WRITE_TEXT`),
  bevor weitere Namen kollidieren.

### B4 · Setzen und Lesen gegen Regel 2 — *verwirrt*

- **Wo:**
  - **Setter ohne `SET`:** `HTTP_TIMEOUT`, `SMTP_TIMEOUT`, `MODBUS_TIMEOUT`,
    `SERIAL_TIMEOUT`, `MODBUS_UNIT`, `SPI_SPEED`/`_MODE`, `HOTPLUG_INTERVAL`,
    `VIDEO_LOOP`, `TRAY_TOOLTIP`/`_MENU`, `FIRMATA_STEPPER_SPEED`/`_ACCEL`,
    `AUDIO_VOLUME`, `AUDIO_PITCH`, `AUDIO_PAN_POS`, `AUDIO_BUS_VOLUME` (neben
    `AUDIO_BUS_GET_VOLUME`!), `AUDIO_BUS_PAN`, `SPEAK_RATE`, `SPRITE_TINT`,
    `MODEL_TEXTURE`, `TEXT_SIZE`, `MOUSE_LOCK`, `BLEND_MODE`, `GUI_WINDOW_TITLE`,
    `_MOVABLE`, `_VISIBLE` …
  - **`GET` an drei Stellen:** `GUI_GET_X`, `GUI_BUTTON_GET_VARIANT$`,
    `GUI_SCALE_GET`/`GUI_THEME_GET`/`GUI_PANEL_SCROLL_GET`.
  - **Leser ohne GET:** `CHAR_X`, `CAMERA_ROTATION`, `AUDIO_NUM_CHANNELS`,
    `AUDIO_MUSIC_POSITION`.
  - **Ein Wort, zwei Richtungen:** `GUI_WINDOW_VISIBLE(win, flag)` setzt,
    `GUI_VISIBLE(widget)` liest. `WINDOW_WAIT_EVENTS()` liest,
    `WINDOW_WAIT_EVENTS(s)` setzt (`vm_grafik.rs:933`). `GUI_PALETTE([on])`.
  - **„Schalten“ dreifach:** `GUI_SET_ENABLED` / `GUI_MENU_ENABLE` /
    `GUI_LISTBOX_ENABLE`; `GUI_SET_CHECKED` neben `GUI_MENU_CHECK`.
  - `GUI_STATUSBAR_SET`/`GUI_BREADCRUMB_SET` meinen etwas anderes als die 21
    übrigen `GUI_<W>_SET(w, key$, value)`.
  - `GUI_SET_MOTION`/`GUI_GET_MOTION` sind kein Paar (Modus gegen Zustand).
  - `GUI_WINDOW_CLOSED` ist ein Zustand, alle anderen `*_CLOSED` sind Ereignisse
    eines Bildes.
- **Vorschlag:** Neue Namen über `namen.rs`, nach Bereichen gebündelt.
  Zeitangaben aus A5 beachten: `HTTP_SET_TIMEOUT` bekommt einen eigenen Befehl.
  Für die Doppelformen kommen reine Abfragen dazu, etwa
  `GUI_PALETTE_IS_OPEN` und `GUI_WINDOW_IS_CLOSED`.

### B5 · Freigabe-, Zähl- und Zustandswörter — *verwirrt*

- **Wo:**
  - **Freigabe:** siehe A4.
  - **Zählen:** `_COUNT` (57 Befehle) gegen `JSON_LEN` (Anzahl Elemente),
    `ASTAR_PATH_LEN`, `SAMPLE_LEN` (Sekunden!). `MAPSIZE` hat schon `MAP_COUNT`.
    Der **neue** Name `ARRAY_DIM_SIZE` nimmt `_SIZE` für eine Länge, gegen
    Regel 4. Dazu `XML_COUNT` neben `XML_CHILD_COUNT`, `TILED_COUNT_GID`
    (Wortstellung), `AUDIO_NUM_CHANNELS`.
  - **„Fertig“:** `SPRITE_IS_FINISHED`, `TWEEN_DONE`, `VIDEO_DONE`.
  - **Empfangen:** `WS_NEXT`/`WS_MESSAGE$`, `OSC_NEXT`,
    `MQTT_NEXT_MESSAGE`/`MQTT_MESSAGE_PAYLOAD`.
  - **Schlüssel:** `SAVE_KEYS` liefert einen Text „a, b“, `MAPKEYS`/`JSON_KEYS`
    ein ARRAY. `SAVE_DELETE` liefert NIL, die `_REMOVE`-Befehle BOOLEAN.
    `LEN(BUFFER)` geht nicht.
- **Vorschlag:** Regel 4 ergänzen: `_COUNT` = Anzahl, `_LEN` = Länge in Zeichen,
  Bytes oder Elementen eines Feldes, `_DONE` = fertig, `_NEXT` = nächstes holen.
  Neue Namen über `namen.rs`, wo nur der Name wechselt. Neue Befehle, wo das
  Ergebnis anders wird (`SAVE_GET_KEYS` als ARRAY, `LEN(BUFFER)`).

### B6 · Parameternamen gegen Regel 8 — *bricht etwas* (benannte Argumente)

- **Wo (Index):**
  - Pfad: `FILE_EXISTS(p)` gegen `FILE_SIZE(path)`.
  - Nachkommastellen: `digits` (ROUND, ZAHL$) gegen `decimals` (ROUND_HALF_UP,
    NUMFMT$).
  - Derselbe Match-Arm, zwei Namen: `SQR(v)`/`SQRT(number)`, `SIGN(v)`/`SGN(number)`.
  - Lautstärke: `volume`/`vol`/`v`. Wellenform: `waveform$`/`wf$`/`wf`.
  - Handle: `handle`/`h`/`m`/`conn`/`c`/`r`/`result`. PDF: `x_mm` gegen `x`.
  - `bind_addr` gegen `bind$`, `on` gegen `flag`, `handler` gegen `fn`.
  - Deutsch: `RENAME(alt, new)` (am Test `compiler.rs:7049` vorbei),
    `GUI_TEXTAREA_SELECT(z1, s1, z2, s2)`, `b` statt `w`.
  - **Kurzhilfen in Meldungen** nennen an rund 50 Stellen in `vm_gui.rs` deutsche
    Namen („(tbl, ab, zellen)“ statt `from, cells`); dazu `builtins.rs:2597,
    4186, 4217, 5795`.
  - **Vorgaben fehlen**, obwohl der Code sie kennt: 56 von 62 optionalen
    gui-Parametern, rund 20 bei Geräten (FIRMATA_OPEN baud 57600, MODBUS unit 1,
    MIC_RECORD_START 600 …), viele Farbvorgaben (BOX, CIRCLE … `[, color]` gegen
    ARC `color = &HFFFFFF`). Ohne Vorgabe kann ein benanntes Argument den
    Parameter nicht überspringen.
- **Beleg (Lauf):**
  ```text
  FILE_EXISTS(path: "x.txt")        -> Compile-Fehler: FILE_EXISTS: kein Parameter 'path'. Die Parameter heissen p
  SQR(number: 4)                    -> Compile-Fehler … heissen v ;  SQRT(number: 4) = 2.0
  PDF_TEXT(p: d, x: 10, y: 20, …)   -> kein Parameter 'x' … heissen p, x_mm, y_mm, …
  GUI_SPINNER(w, 10, 240, 80, 0, 10, step: 0.5) -> 'default' hat keine Vorgabe
  ```
- **Vorschlag:** Parameternamen sind fest (`docs/stabilitaet.md`). Deshalb soll
  der Compiler **Zweitnamen für Parameter** lernen (`p`≡`path`,
  `digits`≡`decimals`, `vol`≡`volume`, `n`≡`count`). Die Kurzhilfen werden aus
  dem Index erzeugt. Vorgaben nachtragen; das ist eine reine Index-Änderung. Die
  Prüfung `dhrt pruef parameter` bekommt dazu: gleicher Arm, gleiche Namen.

### B7 · Typ- und Modulnamen — *verwirrt*

- **Wo:**
  - Handle-Typen heißen `JSON_HANDLE`, `XML_HANDLE`, `MQTT_HANDLE`,
    `FIRMATA_HANDLE` gegen `PDF`, `SMTP`, `WEBSOCKET`, `HTTPD`, `DB_CONN`,
    `NET_SOCKET`.
  - `PHYS_WORLD` (`value.rs:947`) steht neben `PHYS2D_WORLD`.
  - Der HTTP-Client liegt im Modul `html`. `IMPORT "http"`, `"csv"`, `"zip"`,
    `"krypto"` scheitern (`preprocess.rs:24` `MODULES`), obwohl CLAUDE.md `csv`
    und `zip` als Module nennt.
  - Im Index stehen `core` (493) und `dhrt` (239) für dasselbe.
  - `LEADERBOARD_*` gehört zu cloud, trägt den Bereich aber nicht vorn.
- **Beleg (Lauf):** `IMPORT "csv"` → „IMPORT: Datei nicht gefunden: csv“.
  `HTTP_TIMEOUT` ohne Import warnt „gehoert zum Modul 'html'“.
- **Vorschlag:** `IMPORT "http"` als zweiten Modulnamen einführen (wie
  `time`/`money`). `csv` und `zip` in der CLAUDE.md als Kern führen. Im Index
  `dhrt` → `core`. Zweite Typnamen nur dort, wo kein Klassenname verdeckt wird
  (Regel 6 der Leitlinie). `CLOUD_LEADERBOARD_*` über `namen.rs`.

### B8 · Umgebungsvariablen: drei Vorsilben, zwei Sprachen — *verwirrt*

- **Wo:** `DHRT_*` (Mehrheit) neben `DH_LOG` (`vm_system.rs:825`), `DH_PATH`
  (`preprocess.rs:283`) und `DH_PYTHON_DLL` (`ffi.rs:485`). Deutsch und Englisch
  für dasselbe: `DHRT_FONT` (`graphics.rs:2527`) gegen `DHRT_SCHRIFT_ORDNER`
  (`gui_bausteine.rs:99`), `DHRT_SCALE` gegen `DHRT_NATIV_MASSSTAB`,
  `DHRT_FRAMES` gegen `DHRT_BILD_ENDE`. Die Verneinung lautet `KEIN_` oder
  `OHNE_`.
- **Vorschlag:** Eine Leitlinie (Vorsilbe `DHRT_`, englisch). Eine kleine Tabelle
  `alt → neu` an einer Stelle, alte Namen weiter lesen.

### B9 · Tastenkonstanten — *verwirrt*

- **Wo:** `vm.rs:4626-4662`: `KEY_LSHIFT`, `KEY_KP0`, `KEY_PAGEUP` ohne
  Unterstrich, `KEY_KP_ENTER` mit. `KEY_RETURN` = `KEY_ENTER` doppelt.
  `CAMERA3D_UPDATE` nimmt den Modus als Zahl, andere Modi als Text.
- **Vorschlag:** Zweite Namen `KEY_LEFT_SHIFT`, `KEY_KP_0`, `KEY_PAGE_UP` in
  `DEFAULT_KEYS`.

### B10 · Regel 5 („das `$` ist frei“) gilt nur für 175 von 2672 Befehlen — *verwirrt*

- **Wo:** `compiler.rs:867` `builtin_liefert_text` liest nur Signaturen mit
  `AS STRING`/`-> STRING`. Eine zweite Hand-Liste steht in `typen.rs:131`.
- **Beleg (Lauf):** `JSON_STRINGIFY$(j)`, `DB_GET_STRING$(r, 0)` → „Unbekannter
  Befehl … Meintest du JSON_STRINGIFY?“.
- **Vorschlag:** Rückgabetypen im Index nachtragen (siehe E1) und einen Test in
  `namen.rs`, der für jedes `NAME$` die Text-Rückgabe verlangt.
  `typen::builtin_typ` aus dem Index speisen.

### B11 · Begriffe in Meldungen — *innen*

- **Wo:** „Unbekanntes Builtin“ (Compiler-Warnung, `compiler.rs:4093`) neben
  „Unbekannter Befehl“ (Laufzeit, `vm_helfer.rs:110`). „Key“ neben „Schluessel“
  (`builtins.rs:4844`). „Integer-Division durch 0“ neben „MOD durch 0“ (ohne
  englische Vorlage).
- **Vorschlag:** Ein Begriff („Befehl“), Vorlagen in `meldungen.en.txt`
  vervollständigen.

---

## C. Reste alter Zeiten

### C1 · `DH_OHNE_AUDIO` liest die Laufzeit nicht — *bricht etwas*

- **Wo:** `CLAUDE.md` („`DH_OHNE_AUDIO=1` ohne Soundkarte“),
  `.github/workflows/ci.yml:95-119`, `release.yml:88`,
  `.github/pull_request_template.md:15`. Kein `.rs` liest die Variable; sie
  stammt aus `conftest.py` der Python-Zeit. `audio_mut` (`vm_gui.rs:2104`)
  versucht ohne Gerät bei **jedem** Ton-Befehl neu zu starten; nur die gui merkt
  sich den Fehlschlag.
- **Beleg (Lauf):** Mit `DH_OHNE_AUDIO=1` läuft der Ton an
  (`AUDIO_BUS_VOLUME("sfx", 0.6)` → 0.6). Zwei Beispiele prüfen die Variable
  selbst (`examples/nachtflug/musik.dh:1246`, `examples/lichterglanz/musik.dh:1460`).
- **Vorschlag:** In `Audio::new` bei gesetzter Variable **und** ohne Gerät Kiras
  `MockBackend` nehmen (die Web-Fassung hat schon einen eigenen Backend-Typ,
  `web_audio.rs`). Den Fehlschlag merken. `DHRT_OHNE_AUDIO` als neuen Namen
  einführen, den alten weiter lesen.

### C2 · Port-Sprache in Meldungen, die Nutzer sehen — *verwirrt*

- **Wo:** „Grafik-Builtin 'X' im Rust-Kern noch nicht verfuegbar (Schritt 4)“
  (`vm_helfer.rs:115`; laut `:105-108` erkennen die Prüfsammlungen daran den Bau
  ohne Fenster). Dazu „Stufe 3e: … noch nicht unterstuetzt“ (`compiler.rs:1499,
  3597`), „Opcode {} im Rust-VM noch nicht implementiert“ (`vm.rs:4464`) und
  „Neu bauen mit: python rust\build_runtime.py …“ in Meldungen an Endnutzer
  (`vm_helfer.rs:100`, `vm_dienste.rs:835`, `main.rs:353` `--version`; in
  `dhrt-mini` bringt `--hardware` Grafik/PDF nicht zurück).
  `builtins.rs:7135` `is_graphics_builtin` führt `keydown`, `color` und `fill`,
  die es nicht gibt.
- **Beleg (Lauf):** `dhrt-mini` mit `IMAGE_LOAD` → „Grafik-Builtin 'LOADIMAGE' im
  Rust-Kern noch nicht verfuegbar (Schritt 4)“.
- **Vorschlag:** Wie A3 („NAME: gibt es in dieser Laufzeit nicht (ohne
  Grafik gebaut)“). Prüfsammlungen und `fehlt_im_bau` lesen den alten Satz eine
  Fassung lang mit. Interne Fehler heißen „interner Fehler … (bitte melden)“.

### C3 · raylib-Zeit im Ton — *verwirrt*

- **Wo:**
  - `loops` übergangen, `update()` nur bei FLIP: siehe A19.
  - `.qoa` steht in Doku und Prosa (`docs/module-audio.md:157, 369`,
    `daten/builtin_prosa.json`, `umstieg.rs:123`), Kira/symphonia lesen es aber
    nicht.
  - `AUDIO_INIT` übergeht alle Argumente (`vm_grafik.rs:1873`, der Index nennt
    trotzdem Vorgaben). `AUDIO_SET_NUM_CHANNELS` wird nur gemerkt
    (`audio.rs:2409`), das Doku-Beispiel `docs/module-audio.md:26-27` verspricht
    Latenz und Stimmenzahl.
  - Kommentare: „verbatim aus audio.rs“ in `audio.rs` selbst (`:537, 1553,
    2683`), „Musik-Stream nachfuettern“ (`vm_grafik.rs:796`).
- **Beleg (Lauf):** Eine gültige QOA-Datei → „LOADSOUND: SymphoniaError(Unsupported
  …)“. Nach `AUDIO_SET_NUM_CHANNELS(1)` und drei Klängen ist BUSY = 3.
- **Vorschlag:** QOA aus Doku und Prosa streichen (oder einen kleinen Dekoder wie
  `flac_strom.rs` bauen). Die Wirkungslosigkeit von `AUDIO_INIT` und
  `AUDIO_SET_NUM_CHANNELS` dokumentieren, das Beispiel streichen und die
  Index-Vorgaben entfernen.

### C4 · SDL/pygame in der Eingabe — *verwirrt*

- **Wo:**
  - Die Tastencodes sind pygame-Werte (`vm.rs:4626`, `graphics.rs:10816`
    `map_key`, `:10917` `dh_key_code`). Die Doku sagt noch „SDL-Code“
    (`docs/builtins-namen.md:80`, `daten/builtin_prosa.json`).
  - Eine eigene `key_from_i32` (`graphics.rs:10894`) steht neben der aus raylib,
    die zwei Stellen weiter benutzt wird (`:7248`).
  - Ein verwaister Kommentar über `neun_rand` (`graphics.rs:10476`).
  - In `key_label` sind NUM, ROLLEN, DRUCK und PAUSE unerreichbar.
  - `SCROLL` tut still nichts und verweist auf den „Tree-Walker“
    (`vm_grafik.rs:803`).
  - Die SDL-Codes stehen an vier Stellen, darunter `gui.rs:81` und `gui.rs:477`
    roh.
  - `IMAGE`, `SOUND` und `FILE` sind Schlüsselwörter aus der pygame-Zeit
    (`lexer.rs:113-115`). Ein Parameter `file` meldet nur „Erwartet
    Parametername“, und `b IS IMAGE` ist nie wahr.
- **Vorschlag:** `SCROLL` meldet einen klaren Fehler. Die eigene `key_from_i32`
  entfällt. gui und graphics lesen die Codes aus `DEFAULT_KEYS`.
  `image`/`sound`/`file` werden kontextuell wie `step`; das bricht nichts, weil
  niemand sie bisher als Namen nutzen konnte.

### C5 · Python als Quelle der Wahrheit in Kommentaren und Werkzeugen — *innen*

- **Wo:**
  - `vm.rs:6` („Semantik 1:1 aus `drachenhauch/vm.py`, damit stdout bit-identisch
    bleibt“), `builtins.rs:1-5` („Schritt 3“, „1:1 aus interpreter.py“,
    „RND/MILLIS sind NICHT hier“, obwohl sie es sind), `vm_grafik.rs:21-26`
    (`interpreter.py:3541` begründet das Abschneiden aus A1), `html.rs:1-4`,
    `net.rs:1-5`, `controller.rs:1`, `tiled.rs:1`, `astar.rs:4`.
  - `kodierung.rs:16-18` behauptet eine Prüfung „Byte für Byte gegen Pythons
    Codecs“ in `test_kodierung.py`; die Datei gibt es nicht mehr.
  - Der Compiler läuft an drei Stellen über die JSON-Form des AST
    („Parität zu Python“, `ast.rs:4-8`, `compiler.rs:5832, 6195, 6212`).
  - Die CLI-Einstiege `--tokens`, `--ast`, `--preprocess` und `--runsrc`
    („Output gegen Python-Tree-Walker“, `main.rs:2284`) ruft niemand mehr.
- **Vorschlag:** Kommentare auf „Regel: …“ umschreiben, Parität-Durchläufe auf
  `Node` stellen. `--runsrc` behalten oder streichen; `docs/stabilitaet.md`
  sichert nur run, --export, --check und test.

### C6 · GameBasic-Kürzel und Platzhalter — *innen*

- **Wo:** `main.rs:242/249` `embedded_gbc`, `main.rs:2621` `gbc_bytes`,
  `compiler.rs:6750` `compile_to_gbc`, `wifi.rs:179` Temp-Datei
  `_gb_wifi_<ssid>.xml`.
  Der User-Agent lautet `Drachenhauch/0.1 (+https://github.com/anthropic-ai)`
  (`html.rs:10`, sichtbar für jeden angefragten Server), `cloud.rs:12`
  `Drachenhauch-cloud/0.1`.
- **Vorschlag:** `dh` statt `gb`; ein UA aus Fassung und Repo-Adresse für html,
  cloud und websocket.

### C7 · `umstieg.rs` empfiehlt alte Namen und widerspricht Befehlen — *verwirrt*

- **Wo:** `umstieg.rs:22` („sqrt heisst SQR“, `SQRT` ist ein Befehl), `:20`
  („count heisst LEN“, `COUNT` ist ein Befehl), `:43` (keydown → KEYPRESSED/KEYHIT,
  danach folgt „Meintest du KEY_DOWN?“), `:52-55` und `:46-47` (FILEEXISTS,
  ZEIT_TEIL). Tote Einträge für Namen, die selbst gültige Befehle sind
  (`:13-32`).
- **Vorschlag:** Hinweise auf die neuen Namen umstellen. Ein Rust-Test verlangt:
  kein Schlüssel von `befehl()` ist ein bekannter Builtin.

### C8 · Toter Code und verdeckende `allow(dead_code)` — *innen*

- **Wo:**
  - Unbenutzt: `Vm::frame_count` (`vm.rs:4566`) und `Gui::frame_count`
    (`gui.rs:4043`, nur hochgezählt), `TaskErgebnis::ausgabe`
    (`hintergrund.rs:126`).
  - `#[allow(dead_code)]` an `bi_str`/`bi_num`/`bi_bool` (`vm.rs:339, 355, 364`),
    die 329-mal aufgerufen werden.
  - Pauschal für ganze Dateien: `nativfenster.rs:22`, `systemaussehen.rs:16`,
    Module in `main.rs:56-73`.
  - 34 Arme `| "gui_x"` neben `"gui_x$"` (`vm_gui.rs:396 …`) sowie
    `"midi_in_name$" | "midi_in_name"`. `builtin_kanonisch` bildet die Form ohne
    `$` schon im Compiler ab, die Arme sind vermutlich tot.
  - `ganze` steht viermal in `vm_gui.rs` (`:138, 400, 1112, 1135`).
- **Vorschlag:** Streichen. `allow(dead_code)` gezielt als
  `cfg_attr(not(windows), …)` setzen.

### C9 · Verrutschte Doku-Kommentare und leere Abschnittsköpfe — *innen*

- **Wo (Auswahl):**
  - `vm.rs:160-167`: Zwei Doku-Blöcke und das `#[inline]` von `call_parts`
    hängen an `gfloats`.
  - `vm.rs:593-597`: Die Doku von `safe_call_builtin`/`BUILTIN_FAMILIEN` sitzt
    an `braucht_zeile`.
  - `vm_dienste.rs:9` „Modul scene“ steht über `try_db`, `:218` „Modul mqtt“ über
    `try_geld`.
  - `vm_geraete.rs:9-12`, `:523-536`: leere oder falsche Abschnittsköpfe.
  - `vm_system.rs:224-232`: Die Doku von `try_hintergrund` sitzt an `prozess()`.
  - `builtins.rs:6115-6139`: Ein eingeschobenes `benutzer_datenordner` zerreißt
    die Doku von `ortszeit_versatz`.
  - `gui_zeichnen.rs:2217-2234`, `gui_baeume.rs:1031-1043` (nennt `tc_titel`, das
    es nicht gibt), `gui.rs:4212` (`caret_period` = Bilder) gegen
    `gui_geometrie.rs:314` (Sechzigstel).
- **Vorschlag:** Beim nächsten Anfassen zurückrücken. Am `#[inline]` lohnt es
  sich sofort (heißer Pfad, nicht gemessen).

### C10 · wifi: veraltete „nur Windows“-Angaben — *verwirrt*

- **Wo:** `vm_geraete.rs:1534`, `rust/build_runtime.py:173-176` („WIFI_* werfen
  auf diesem System“), `docs/module-wifi.md:21`, Kommentar zum Feature in
  `Cargo.toml`. Laut `wifi.rs:1-12` gilt der Befehl für Windows, Linux und macOS.
  `WIFI_CONNECT` scheitert teils mit Fehler, teils mit FALSE (`wifi.rs:173-188`).
  Als Datenwert kommt „(versteckt)“.
- **Vorschlag:** Angaben angleichen und eine Fehlschlag-Form wählen.

---

## D. Struktur

### D1 · Regel 6 verletzt: `__comp_iter` hängt an der Reihenfolge — *bricht etwas*

- **Wo:** `vm.rs:3015` (`try_coro` antwortet nur, wenn das Argument eine
  Koroutine ist, sonst `Ok(None)`), `vm.rs:2369` (gemerkt wird jede Familie außer
  bei `sort`), `builtins.rs:1443` (die reine Familie kennt den Namen, lehnt eine
  Koroutine aber ab).
- **Beleg (Lauf, auch mit `DHRT_JIT=aus`):**
  ```text
  DIM t AS TUPLE = (feld, zahlen())   ' zahlen() ist eine Koroutine
  FOR EACH q IN t : PRINT [x * 10 FOR x IN q] : NEXT
  -> (50, 60)
  -> Laufzeitfehler: Comprehension: nicht iterierbar (COROUTINE)
  ```
  In umgekehrter Reihenfolge laufen beide.
- **Vorschlag:** Den Namen in **einer** Familie beantworten (der Arm in
  `try_coro` nimmt alle Typen) oder ihn vom Merken ausnehmen wie `sort`. Ein
  Prüffall „Feld vor Koroutine an derselben Stelle“.

### D2 · `try_*`-Familien: Panik ungeschützt, falsche Dateien, drei Arten Feature-Hülle — *bricht etwas*

- **Wo:**
  - Nur die reine Familie läuft unter `catch_unwind` (`vm.rs:614`
    `safe_call_builtin`). In `try_geld` greifen sechs Arme ungeprüft auf `a[0]`
    zu (`vm_dienste.rs:245-279`).
  - Ton steht in `vm_grafik.rs:1623-2275`. WebSocket, OSC und mDNS stehen in
    `vm_geraete.rs:537` `try_netzdienste`, net, httpd und mqtt in `vm_dienste.rs`.
    `try_timer`/`try_scene` (Spiel) stehen in `vm_system.rs`. `DB_QUERY_*` liegt
    in Familie 8 (`vm_system.rs:241`), `DB_*` in Familie 9.
  - Die `dbg_*`-Hilfen und `ui_preset_metrics` stehen in `vm.rs`.
  - Feature-Hüllen gibt es in drei Formen: `Ok(None)`, eine eigene Meldung oder
    einen Sonderfall in `unknown_builtin_msg` (siehe A3).
- **Beleg (Lauf):** `PRINT GELD_CENT()` → `thread 'main' panicked at
  src\vm_dienste.rs:245:58`, Exit 101; `TRY` fängt es nicht. Vorher kommt nur
  eine Compiler-**Warnung**. Ein Durchlauf aller 276 Befehle der VM-Familien mit
  0 Argumenten fand genau diese sechs.
- **Vorschlag:** `familie_rufen` als Ganzes unter `catch_unwind` stellen (eine
  Stelle statt 33). `try_geld` prüft die Anzahl. Die Argumentzahl-Prüfung des
  Compilers wird ein Fehler (A2). Die Verschiebungen sind reine Umzüge, die
  Familiennummern bleiben.

### D3 · Doppelte Helfer (Ursache vieler A-Befunde) — *innen*

- **Wo:**
  - Argumente: siehe A1, dazu `bi_bool` wortgleich in `vm.rs:365` und
    `vm_dienste.rs:1278`.
  - `new_str_array`/`new_string_array` (`builtins.rs:740/787`) und
    `new_int_array`/`int_array` (`:796/5849`) sind identisch. Dazu sechs lokale
    String-Feld-Bauer (`vm_geraete.rs:732, 1013, 1107, 1260`, `vm_system.rs:361,
    475`).
  - latin-1 dreimal, `bytes`/`bytes_arg` und `int_opt` je zweimal
    (`vm_geraete.rs:1197/1357`, `:563/1376`).
  - Hex-Ausgabe zweimal, HSV zweimal (`builtins.rs:2060` gegen
    `farbraum.rs:14`, verschieden gerundet).
  - Entities zweimal (`xml.rs:257`, `html.rs:279`), Zufall zweimal
    (`websocket.rs:161`, `smtp.rs:584`).
  - Doppelt in der Sache: Kalender (A10), coerce (A9), URL-Dekodierer und TLS
    (A15), WAV und Wellenform (A19), Hintergrund-Aufträge (A15), Liste der
    Testvariablen (A16), Tastentabellen (C4), Index geparst (A20).
- **Vorschlag:** Je eine Fassung in `vm_helfer.rs` bzw. dem Fachmodul behalten.
  Das bringt keine Verhaltensänderung, bis auf die angeglichenen Meldungen.

### D4 · gui: die „eine Quelle“ steht, wo man sie nicht sucht — *innen*

- **Wo:**
  - Die Tabellen-Bedienung samt `table_geom` steht in `gui_richtext.rs:776-1779`.
  - Der halbe Textbereich samt `ta_rows` steht in `gui_eingabe.rs:1390+`.
  - Farben, Schrift und Flächen-Zeichnen stehen in `gui_geometrie.rs:273-616`.
  - Die `.dhform`-Serialisierung steht in `gui_layout.rs:409-1148`.
  - Alle Köpfe sagen „herausgeloest ohne Aenderung am Inhalt“: Die Aufteilung
    war mechanisch.
  - Zweite, nachgeführte Lagen: `liste_textplatz` (`gui_zeichnen.rs:2235`), die
    Oberkante der Fenster-Reiterleiste fünfmal, die sichtbaren Zeilen des
    Textbereichs sechsmal, `ta_innen` = `ta_breite` (`gui_eingabe.rs:1408/1429`).
  - Zwei Merker für „Taste verbraucht“: `liste_taste` und `kuerzel_gefeuert`
    (`gui.rs:3984`, `gui_menues.rs:185`). Daraus folgt ein Einzelfehler unten.
  - „Bedienbar“ ist zweimal definiert (`gui_pruefen.rs:38` gegen `gui.rs:446`
    `fokussierbar`).
- **Vorschlag:** Die Tabelle nach `gui_tabelle.rs`, den Textbereich nach
  `gui_textbereich.rs`, die Serialisierung nach eigener Datei. `tabbar_y`,
  `ta_sicht` und `taste_verbraucht` werden je ein Weg.

### D5 · Externe Typen: zwei Muster nebeneinander — *verwirrt*

- **Wo:** Echte Werte mit eigener `Value`-Art und `model::neutrales_element`
  (JSON, XML, SPRITE, GELD, VEC2 …) stehen neben INTEGER-Handles mit nur einem
  Typnamen für den Übersetzer (`preprocess.rs:57-90` `MODULE_TYPES`; db, net,
  pdf, xlsx, smtp, httpd, mqtt, Bilder, Klänge …). Folgen für `IS`, `TYPEOF`,
  Zuweisung und Verwechslung siehe A4.
- **Vorschlag:** Eine Tabelle „Typname → Art (Wert oder Handle)“, aus der
  `coerce`, `IS` und `--check` lesen. Langfristig getaggte Handles.

### D6 · Index als „eingefrorene“ Datei aus der Migration — *innen*

- **Wo:** `daten/builtin_index.json` `_comment`: „Stammt aus der
  Stufe-B-Migration (Registry-Union); seitdem von Hand gepflegt“. Das Feld `kind`
  hat fünf Werte (core 2058, graphics 464, module 113, function 25, builtin 12),
  die mit `module` nicht zusammenpassen (`DELTA`, `TIMER`, `GET_TIME`: module
  core, kind graphics).
- **Vorschlag:** `kind` auf eine Bedeutung zurückführen oder streichen, falls
  nichts mehr liest. Den Kommentar auf den Stand bringen.

---

## E. Doku gegen Code

### E1 · Signaturen im Index: vier Stile — *verwirrt*

- **Beleg (Zählung über 2672 Einträge):** 1844 ohne Rückgabetyp, 736 mit `AS`,
  92 mit `->`. 136 Signaturen mit Typ `?`, rund 120 mit deutschem `Zahl`, 8 mit
  „Code oder Buchstabe“. `GUI_RULE(2..5 Argumente)` (gegen CLAUDE.md, nimmt so
  keine benannten Argumente). Der Eintrag `READALL` trägt die Signatur von
  `READALL$`, `TIME` die von `TIME$`. Bei serial, usb, bt und wifi steht noch
  `handle: ?` mit `Zahl` und ohne Rückgabe.
- **Vorschlag:** `dhrt pruef parameter` um „eine Signaturform, kein `?`, kein
  `Zahl`“ erweitern und dann nachziehen. Das wirkt auf Hover, `--check` und
  Regel 5 (B10).

### E2 · Index verspricht, was der Code nicht tut — *bricht etwas*

- **Wo:**
  - `SLEEP(ms: Zahl)` gegen `bi_int` (`vm_system.rs:130`): `SLEEP(1.5)` wirft.
  - `PLAYMUSIC`/`MUSIC_PLAY(…, loops = 0 …)` gegen Doku „-1“ und Code „immer
    endlos“.
  - `AUDIO_INIT([frequency = 44100 …])` übergeht alle Argumente.
  - `SAMPLE_PLAY(sample, semitones, vol[, dur_ms])`: Im Code ist `vol` optional.
  - `CLIPBOARD_SET_IMAGE … AS BOOLEAN` liefert nie FALSE.
  - `WEBCAM_OPEN([device$ = "" [, width = 0, height]])` verschweigt, dass width
    ohne height ein Fehler ist.
- **Vorschlag:** Den Code an die Signatur anpassen, wo das nur erweitert
  (SLEEP), sonst die Signatur an den Code.

### E3 · Handbuch gegen Code — *verwirrt*

- **Wo:**
  - `docs/module-imgfx.md:301` „Immutable: das Original wird nie verändert“,
    obwohl `IMAGE_DRAW_*`/`IMAGE_FILL_*` ins Bild zeichnen und NIL liefern.
  - `docs/module-gui.md:2073` „Breite der Köpfe wird geschätzt“, obwohl
    `leisten_pass` misst (`gui_baeume.rs:1084`).
  - `docs/module-serial.md:45` gibt einen Binär-Rat (A13).
  - `docs/module-hotplug.md:20` sagt „dieselben Namen wie USB_LIST“, das stimmt
    für hid nicht.
  - `docs/module-audio.md:21-34`: Kanäle, Latenz und „Fades über FLIP“ stimmen
    nicht; Fades sind Kira-Tweens, endliche Loops brauchen FLIP.
  - `docs/module-pdf.md:48` sagt nicht, dass Seiten ab 1 zählen.
  - `docs/module-html.md:21`: Kopfzeilen „für alle folgenden Aufrufe“, nicht für
    `HTTP_DOWNLOAD`.
  - `docs/module-net.md:22`: `NET_PEER_ADDR` liefert beim Verbinder den
    getippten Namen, nicht die IP.
  - `docs/builtins-namen.md:68`: `TIMER()` sei „dieselbe Uhr“, `GET_TIME` steht
    daneben ohne Unterschied (A5).
  - `docs/builtins-grafik.md:951` WAITKEY und `:1031` JOY-Konstanten (A17).
  - CLAUDE.md: IMPORT … AS heißt dort „Namensraum (`namensraum.rs`)“, aber das
    eingebaute Modul folgt `preprocess.rs:107` (`j_parse`, nicht `j.parse`).
    Dazu `csv`/`zip` als Module (B7) und `DH_OHNE_AUDIO` (C1).
- **Vorschlag:** Die Absätze in beiden Sprachen berichtigen (`dhrt pruef
  uebersetzung`).

### E4 · `dhrt pruef parameter` sieht nur den ersten Befehl einer Tabellenzeile — *verwirrt*

- **Wo:** `pruef.rs:214` (Regex nur am Zeilenanfang). Deshalb stehen in
  Sammelzeilen von `docs/module-gui.md` (`:106-111, 258-301, 446-450`) und
  `docs/en/` noch `von, nach, farbe, an, seite, schritt`, rund 25 Zeilen je
  Sprache.
- **Vorschlag:** Jeden Befehl in Backticks der ersten Spalte prüfen, dann die
  Funde nachziehen.

### E5 · Kommandozeile: halb übersetzt, Hilfe unvollständig — *verwirrt*

- **Wo:** `main.rs:421, 438, 449, 480, 746, 1801 …`: 13-mal
  `eprintln!("Kann '{}' nicht lesen")` ohne `meldung::t`, der Export mit.
  `dhrt run` ohne Datei sagt „Kann 'run' nicht lesen“ (`main.rs:554`). Die
  Hilfe (`main.rs:360`) nennt `aufnehmen`, `--schlank` und `--mit-daten` nicht
  und `pruef` nur mit fünf von neun Unterbefehlen.
- **Beleg (Lauf, `DHRT_LANG=en`):** `dhrt run nichtda.dh` → deutsch,
  `dhrt --export nichtda.dh` → englisch.
- **Vorschlag:** Eine Funktion `quelle_lesen(pfad)` für die zwölf gleichen Blöcke.
  Unterbefehle ohne Datei vor dem Datei-Rückfall abfangen.

### E6 · Kaputte `.dhc`: Panik statt Lade-Fehler — *innen*

- **Wo:** `model.rs:162-183, 436-461, 488, 520, 623` (`panic!`/`expect`), obwohl
  `load_program` ein `Result` liefert.
- **Beleg (Lauf):** `{"format":"dhc","main":"kaputt"}` → Panik bei
  `model.rs:488`, Exit 101.
- **Vorschlag:** `decode_*` auf `Result` umstellen.

---

## Kleine Einzelfehler (je eine Stelle, nebenbei gefunden)

Diese sind keine Muster, aber sie brechen etwas und sind schnell behoben.

**Stand 2026-11-25:** Auf dem Zweig `thema/argumente-einheitlich` sind 20 der
21 behoben, je mit eigenem Commit, Prüffall und Gegenprobe (Spalte „Stand“).
Offen ist nur der Popover; er ist größer als hier zuerst beschrieben (siehe
seine Zeile).

| Was | Wo | Beleg | Stand |
|---|---|---|---|
| **Webserver stürzt an einer Anfrage ab**: `&s[i+1..i+3]` schneidet mitten in einem Zeichen; HTTPD lauscht per Vorgabe auf `0.0.0.0` | `httpd.rs:194` | Lauf: `GET /%a` + `Ä` → Panik, Exit 101, kein TRY fängt es | behoben: ein Dekodierer auf Bytes für httpd und `URL_DECODE` (`kodierung::prozent_auf`) |
| `GELD_*`/`MONEY_*` ohne Argument: Panik | `vm_dienste.rs:245-279` | Lauf (D2) | behoben |
| Comprehension über Feld, dann Koroutine | `vm.rs:3015` | Lauf (D1) | behoben: vom Merken ausgenommen wie `SORT` |
| `HTML_FIND_ALL` verschluckt Treffer bei `<` im Text (Fix von 2026.27 nur in `HTML_TEXT`) | `html.rs:429` | Lauf: 0 statt 2 | behoben |
| Menükürzel mit Enter/Esc löst zusätzlich Standard-/Abbrechen-Knopf aus | `gui_eingabe.rs:848, 852` | Lauf: „menue bild 6 / ok bild 6“ | behoben |
| Bildschirmleser liest „&Speichern“ (`w.text` statt `anzeige_text`) | `gui_a11y.rs:216-257` | Lauf über UI Automation | behoben, die Befehlspalette gleich mit |
| Popover weicht an `schirm_b/h` aus statt an `grenze()` | `gui_ueber.rs:195, 211` | Lauf: ragt im nativen Fenster 96 px heraus | **offen**: Der eigentliche Fehler ist größer. Ein Popover wird immer auf der Hauptfläche gezeichnet (`gui_ueber.rs:768` `ueber_zeichnen`: „Ein Popover ist nie nativ“), auch wenn sein Widget in einem nativen Fenster steht, und `popup_rechtecke` kennt ihn nicht. Erst wenn er auf der Fläche seines Widgets liegt, ist `grenze()` die richtige Grenze; allein getauscht ändert es nichts Prüfbares. |
| GroupBox schätzt Titelbreite aus Zeichenzahl | `gui_zeichnen.rs:1984` | Lauf: Rahmenlinie läuft durch den Titel | behoben |
| `FILETIME` eine Stunde daneben (Sommer/Winter) | `builtins.rs:3662` | Lauf (A10) | behoben |
| `TIME_FROM_PARTS(2026, 13, 1)` = 2027-01-01 | `vm_system.rs:92` | Lauf | behoben: dieselben Grenzen wie `ZEIT_PARSE` |
| `FLOOR`/`CEIL`/`ROUND(1e19)` sättigen still | `builtins.rs:1660-1664` | Lauf | behoben, im Maschinencode mit |
| `JSON_LOAD` scheitert an BOM | `builtins.rs:4644` | Lauf | behoben, dazu optional `encoding$` |
| `SAVE_LOAD_OR_NEW` verwirft mitgelieferten Spielstand | `builtins.rs:4819` | Lauf (A11) | behoben, `SAVE_EXISTS` mit |
| `DIRLIST_REC(p, "")` liefert nichts, `DIRLIST` alles | `builtins.rs:3608/3621` | Lauf | behoben |
| `SAVE_GET_INT_OR(s, "zz", "text")` liefert Text aus INT-Getter | `builtins.rs:4848-4851` | Lauf | behoben |
| `SPEAK_SOUND`-Handle wird still freigegeben | `sprache.rs:144` | Lauf (A19) | behoben: jeder Aufruf liefert eine eigene Kopie |
| `PHYS3D_*` mit ungültiger Körpernummer still (0,0,0) | `physics3d.rs:123` | Lauf | behoben: ein Fehler wie in 2D |
| `TASK_READY`/`SHELL_READY` mit falscher Nummer ewig FALSE | `hintergrund.rs:62` | Lauf (A4) | behoben, für `DB_QUERY_*` mit |
| `HTTP_DOWNLOAD` lässt `HTTP_BYTES` auf altem Rumpf | `vm_dienste.rs:1203` | Code | behoben, dazu Kopfzeilen und Zeitgrenze wie bei `HTTP_GET` |
| `AUDIO_PUSH` verschluckt ab 64 Ebenen still | `audio.rs:1630` | Lauf | behoben, `GFX_PUSH` hatte denselben Fehler und ist mit behoben |
| `MOUSE_SCALE`/`_OFFSET` wirken nicht auf `MOUSE_DELTA`/`MOUSE_SET_POS` | `graphics.rs:7269` | Vermutung (raylib) | behoben für `MOUSE_SET_POS` (Lauf: nach `MOUSE_OFFSET(-40, 0)`, `MOUSE_SCALE(0.5, 0.5)`, `MOUSE_SET_POS(80, 50)` stand `MOUSEX` bei 20). `MOUSE_DELTA` war richtig: raylibs `GetMouseDelta` rechnet den Maßstab selbst ein, dieser Teil der Vermutung stimmte nicht. |

---

## Rangliste: die zehn Punkte, die sich am meisten lohnen

Gewichtet nach „wie viele Befunde verschwinden“ × „wie schwer sie wiegen“ ÷
Aufwand. Jeder Punkt hält `docs/stabilitaet.md` ein.

1. **Ein Satz Argumenthelfer für alle Familien** (A1, A2, D3). *Umgesetzt auf
   `thema/argumente-einheitlich` (`argument.rs`). Dabei blieb die Grafik bei
   „Nachkommastellen fallen weg“ (`koord`), damit kein Bild um einen Punkt
   verrutscht, und `RMDIR` blieb bewusst streng.*
   - `arg_int` (glatte FLOAT ja), `arg_koord`, `arg_num`, `arg_text`,
     `arg_schalter` (BOOLEAN oder 0/1) in `vm_helfer.rs`, mit **einer**
     Meldungsform `NAME: erwartet X, erhalten Y (Argument n)`.
   - Behebt `TEXT_BOLD(1)` und rund 25 Geschwister, `HTTP_TIMEOUT(10 / 2)`, das
     stille Abschneiden in der Grafik, den fehlenden Aufruf-Hinweis und fünf
     Meldungsformen.
   - Bricht nichts: Es wird nur mehr angenommen, still Falsches wird richtig.
2. **Meldungen mit dem geschriebenen Namen** (B1, B2, A2 Teil). *Umgesetzt auf
   `thema/argumente-einheitlich` (`Func::anzeigen`, `fehler_benennen`).* Eine Stelle an der
   Fehlergrenze ersetzt `ALT:` durch den Namen aus dem Quelltext. Damit ist die
   Namensreform auch in Fehlern angekommen, für alle 143 Paare und die
   Arm-Aliase. Dazu die Arm-Aliase nach `namen.rs`.
3. **Eine Feature-Tabelle** (A3, C2). *Umgesetzt auf `thema/abstuerze-features-handles`
   (`bau.rs`, Modul → Features, gemessen mit `--fehlende` an einem Bau mit nur
   `jit ffi`).* Modul → Präfixe → Feature. Daraus speisen
   sich `MODULE_AVAILABLE`, die IMPORT-Warnung, **eine** Meldung „gibt es in
   dieser Laufzeit nicht“ und `fehlt_im_bau`. Behebt den kaputten schlanken
   Export für httpd, falsche TRUE bei `MODULE_AVAILABLE`, die MIDI-Meldung und
   „(Schritt 4)“.
4. **Eine Handle-Regel und ein `handle_get`** (A4, B5, D5). *Umgesetzt auf
   `thema/abstuerze-features-handles`: Regel in CLAUDE.md, `MODEL_FREE`, `FONT_FREE`,
   `SHADER_FREE`, `RENDERTARGET_FREE`, `ATLAS_FREE`, `SAMPLE_FREE`, Getter nennen den
   Befehl, doppelt freigeben still. Auf `thema/lesen-verbinden` nachgezogen:
   neue Namen `AUDIO_CLOCK/EMITTER/LISTENER/MOD_FREE`, `GUI_WIDGET_FREE`,
   `GUI_WINDOW_FREE`, `PHYS2D/PHYS3D_BODY_FREE`, `BT_CLOSE`, `MQTT_CLOSE`
   (`namen.rs`, die alten bleiben); die DB_RESULT-Meldungen nennen den Befehl
   („DB_NEXT: das Ergebnis wurde mit DB_CLOSE_RESULT geschlossen“). Bewusst
   nicht umbenannt: `ECS_DESTROY` (zerstört ein Wesen im Spiel, gibt kein
   Handle frei) und `WIFI_DISCONNECT` (trennt den Adapter, kein Handle).
   Offen: getaggte Handles.*
   - Ungültig ist ein Fehler mit Befehlsnamen, Freigeben ist idempotent und
     still, Laden scheitert laut (LOADMODEL, SHADER_LOAD).
   - Fehlende `_FREE` nachrüsten (Modell, Schrift, Shader, Render-Ziel, Atlas,
     Sample).
   - `TASK_READY` mit unbekannter Nummer wird ein Fehler.
   - Danach können getaggte Handles `IS`, `TYPEOF` und Verwechslungen lösen.
5. **Abstürze aus der Laufzeit nehmen.** *Umgesetzt auf
   `thema/abstuerze-features-handles`: `builtin_rufen` unter `panik_fangen`, `.dhc`
   mit `Result`, Zufallsproben für jeden Netz-Decoder, dazu die Funde einer
   Zufallssuche (riesige Texte/Felder, imgfx im C-Code von raylib). Offen: die
   Argumentzahl-Warnung des Compilers als Fehler (bräche Programme).*
   `familie_rufen` unter `catch_unwind`
   stellen, `try_geld` prüft die Argumentzahl, die Argumentzahl-Warnung des
   Compilers wird ein Fehler. Der URL-Dekodierer von httpd arbeitet auf Bytes
   (ein Dekodierer statt zwei). `.dhc` lädt mit `Result`. Ein Netzwerkpaket darf
   kein Programm beenden.
6. **Zeit: eine Leitlinie und eine Uhr** (A5, B4 Teil). *Umgesetzt auf `thema/lesen-verbinden`: Regel 9 in
   `docs/builtins-namen.md`, `HTTP_SET_TIMEOUT`/`SERIAL_SET_TIMEOUT` in ms, `SMTP_`/`MODBUS_SET_TIMEOUT`,
   gui-Fristen nach `takt`, `spieluhr.rs` für TWEEN/SHAKE/timer, SLEEP mit Komma,
   PHYS*_STEP in Sekunden. Offen: `CHAR_UPDATE` mit dt, die Coyote-Zeit in Bildern.*
   - Zeitspannen und Timeouts in ms, Schritt-dt in Sekunden.
   - Neue `…_SET_TIMEOUT`-Namen; `HTTP_SET_TIMEOUT` als eigener Befehl in ms.
   - gui-Fristen auf `takt` statt `delta`: der Toast im Leerlauf.
   - Eine Spieluhr für TWEEN, SHAKE und timer, kein `get_time()` in der gui.
   - `SLEEP` nimmt FLOAT, `PHYS*_STEP` bekommt einen Schutz.
7. **gui: nach außen logisch, innen eine Geometrie** (A6 Teil, A18, D4). *Teilweise umgesetzt auf
   `thema/lesen-verbinden`: Splitter logisch, Reiterleiste, Panel-Kopf (`panel_kopf`),
   GroupBox, Fenster-Rollbalken und Mausrad durch `sk`. Offen: Pixel-Abfragen mit
   `_SCREEN_`-Namen (bräche Programme, eine Entscheidung), der Textbereich-Rand 5, die
   Bausteine aus A18 und die Ordnung aus D4.*
   - Splitter-Wert mit `unsk`/`sk`.
   - Pixel-Abfragen bekommen `_SCREEN_`-Namen und logische Geschwister.
   - Alle festen Pixel laufen durch `sk`.
   - Ein Rollbalken, ein Radschritt, ein Kürzen, eine Popup-Liste.
   - Ein Schlüssel-Setter-Helfer mit englischen Zweitnamen und `GUI_<W>_GET`.
8. **Ein Weg zum Lesen und Verbinden** (A11, A15). *Umgesetzt auf `thema/lesen-verbinden`:
   `verbinden.rs` (alle Adressen, NET/WS/MQTT/SMTP/Modbus, UDP nach Art des Sockets),
   cloud kodiert die Pfadteile, COPYFILE/DIREXISTS/DIRLIST/ZIP_* über den Pfad-Helfer,
   `datei_fehler`, ZIP_READ$/SAVE_LOAD/ANIM_FSM_LOAD wie READALL$. Offen (innen): ein
   `tls.rs`, `html::Abrufe` → `hintergrund`, cloud über `http_request`, `MQTT_PENDING`.*
   - Alle lesenden Dateibefehle laufen über `resolve_asset_path` und
     `text_lesen` (JSON_LOAD mit BOM, ZIP_READ$, COPYFILE, SAVE_EXISTS).
   - Ein `datei_fehler` mit Pfad.
   - Ein `verbinden()`, das alle Adressen probiert (`localhost` bei
     NET/WS/MQTT/SMTP).
   - `HTTP_DOWNLOAD` und cloud über `http_request`, ein TLS-Aufbau.
9. **Der Index als eine saubere Quelle** (B6, B10, E1, E2, E4). *Teilweise umgesetzt auf
   `thema/lesen-verbinden`: `dhrt pruef parameter` prüft jeden Befehl einer Zeile (38 Funde
   je Sprache nachgezogen), Zweitnamen für Parameter im Compiler, `RENAME(old, new)`, die
   Signaturen aus E2. Offen: Rückgabetypen und eine Signaturform für alle Einträge (E1,
   B10), Vorgaben nachtragen, Kurzhilfen aus dem Index erzeugen.*
   - Eine Signaturform, Rückgabetypen und Vorgaben nachtragen. Das macht
     benannte Argumente nutzbar und Regel 5 wahr.
   - `builtin_signature` streichen, Kurzhilfen aus dem Index erzeugen.
   - Der Compiler lernt **Zweitnamen für Parameter**: die einzige Möglichkeit,
     die uneinheitlichen Namen zu heilen, ohne die zugesagten zu brechen.
   - `dhrt pruef parameter` prüft jeden Befehl einer Zeile.
10. **Ton aus einem Guss** (C1, C3, A8, A19). *Teilweise umgesetzt auf `thema/lesen-verbinden`:
    `DHRT_OHNE_AUDIO` rechnet stumm (`audio_backend.rs`, `AUDIO_IS_SILENT`), loops bei
    PLAYSOUND/PLAYMUSIC, Loops/AUTOPAN/Warteschlange ohne FLIP, QUEUE prüft, QOA aus der
    Doku. Offen: eine Pan-Skala und eine Lautstärke-Regel (neue Befehle, ein Entwurf),
    `AUDIO_INIT`/`AUDIO_SET_NUM_CHANNELS` ohne Wirkung, ein WAV-Schreiber.*
    - `DH_OHNE_AUDIO` und „kein Gerät“ führen zu Kiras `MockBackend`.
    - `loops` durchreichen, endliche Loops ohne FLIP.
    - Eine Pan-Skala (-1..1) und eine Lautstärke-Regel über neue Befehle.
    - Eine Wellenform-Tabelle mit deutschen Zweitnamen, `SPEAK_SOUND` als Kopie.
    - QOA und die wirkungslosen `AUDIO_INIT`-Argumente aus der Doku.

**Knapp dahinter:**
- Optionswörter zweisprachig über Tabellen (A14).
- Debugger-`eval` über `wert_rechnen` (A20).
- WAITKEY und Gamepad-Codes (A17).
- `coerce` einmal (A9).
- Gerätelisten als ARRAY und Byte-Befehle (A13, A16).
- Umgebungsvariablen-Leitlinie (B8).
- Die kleinen Einzelfehler oben: jeder ist eine Stunde Arbeit samt Prüffall mit
  Gegenprobe.
