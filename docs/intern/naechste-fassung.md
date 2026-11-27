# Für die Notizen der nächsten Fassung

Sammelstelle, bis die Fassung ihre Notizen bekommt
(`docs/releases/release-<fassung>.md` und `.en.md`). Was hier steht, gehört
in den ersten Abschnitt „Was sich für bestehende Programme ändert“. Herkunft:
die Reste aus `docs/intern/inkonsistenzen-runtime.md`, Zweig
`thema/lesen-verbinden`.

## Was sich für bestehende Programme ändert

**Übersetzen**

* **Eine falsche Zahl von Argumenten an einen eingebauten Befehl ist ein
  Fehler beim Übersetzen** (bis 2026.53 eine Warnung; zu viele wurden je nach
  Befehl still verworfen oder brachen zur Laufzeit ab, zu wenige brachen
  immer ab, aber erst, wenn die Zeile lief). Die Meldung nennt die
  Aufrufform aus dem Handbuch; `--check` und der Editor zeigen alle solchen
  Stellen einer Datei, nicht nur die erste.
* Die Argumentzahl-Meldungen haben drei Formen: „erwartet 2 Argument(e)“,
  „erwartet 1 bis 3 Argumente“, „erwartet mindestens 2 Argument(e)“, jeweils
  mit „erhalten n“; die Aufrufform steht genau einmal dahinter.

**gui: Einstellungen nach einer Regel**

* `GUI_<W>_SET(w, key$, value)` nimmt den Schlüssel bei jeder Art in jeder
  Schreibweise, deutsch oder englisch (`multi_select` auch bei Baum und
  Tabelle, `MEHRERE` beim Akkordeon), den Wert als Zahl oder BOOLEAN; ein
  unbekannter Schlüssel meldet sich überall gleich („unbekannter Schluessel
  'x' -- moeglich sind mehrfachauswahl/multi_select, …“). Jede Art hat ihre
  Abfrage `GUI_<W>_GET(w, key$)`; sie liefert eine Zahl, ganz als INTEGER --
  `GUI_TABLE_GET` lieferte bis 2026.53 immer FLOAT (`PRINT` zeigt jetzt `24`
  statt `24.0`).
* Die gui-Rollbalken sind überall 8 Punkte breit (die Tabelle hatte 12, das
  Fenster 10) und stehen gespiegelt links; Popup-Listen und Tipps haben
  denselben Schatten. Pfeil- und Bildtasten wiederholen gehalten in jedem
  Widget.

**gui: Lagen sind logisch**

* `GUI_LISTBOX_ITEM_RECT`, `GUI_TREE_ITEM_RECT`, `GUI_TABLE_ROW_RECT`,
  `GUI_TABCONTROL_ITEM_RECT` und `GUI_CANVAS_X/Y/W/H` liefern logische Punkte
  wie `GUI_GET_X`, `GUI_RICHTEXT_SCROLL`/`_SCROLL_GET`/`_HEIGHT` und die y aus
  `GUI_RICHTEXT_FIND*` ebenso. Bis 2026.53 waren es Bildschirmpixel -- bei
  `GUI_SCALE(2)` das Doppelte. Wer Pixel braucht (Zeichenbefehle in einer
  Zeichenfläche, echte Klicks in Prüfungen), nimmt die neuen Namen
  `GUI_LISTBOX_ITEM_SCREEN_RECT`, `GUI_TREE_ITEM_SCREEN_RECT`,
  `GUI_TABLE_ROW_SCREEN_RECT`, `GUI_TABCONTROL_ITEM_SCREEN_RECT`,
  `GUI_CANVAS_SCREEN_X/Y/W/H`. Bei `GUI_SCALE(1)` (Vorgabe) ändert sich nichts.
* Der Innenabstand von Textbereich und Textfeld wächst mit `GUI_SCALE`
  (vorher roh 5 Pixel).
* `GUI_SET_COLOR`, `GUI_STYLE_SET`/`_GET` nehmen die Rolle in jeder
  Schreibweise (vorher nur klein).

**Meldungen**

* Viele Fehlertexte haben einen neuen Wortlaut (Programme, die im `CATCH`
  den Text vergleichen, prüfen nach): „`NAME`: das Widget ist kein `GUI_X`“
  für alle gui-Arten, `SQR: erwartet eine Zahl >= 0, erhalten …`,
  `RGB: die Werte muessen 0..255 sein, erhalten …`, `NET_*: Port … liegt
  ausserhalb 0..65535`, JSON-Lesefehler auf Deutsch mit Zeile und Spalte,
  Ton-Fehler ohne rohe Bibliotheksnamen, `PADL$`/`PADR$`/`TWEEN_NEW*`/
  `ASTAR_*`/`PARTICLE_*`/`TILED_*` nennen sich selbst statt „PAD“, „S“, „T“.

**Typen**

* Ein Modultyp, der ein echter Wert ist (`VEC2`, `JSON_HANDLE`, `SPRITE`,
  `TILED_MAP` …), nimmt bei der Zuweisung nur seinesgleichen (oder NIL);
  `DIM j AS JSON_HANDLE = 5` ist ein Fehler.

**Schalter**

* `DIRLIST` (rekursiv), `CODE_FORMAT$`, `FONT_LIGATURES`, `AUDIO_LOFI`,
  `GUI_TEXTAREA_FOLD`/`_FOLD_ALL` und `UI_CHECKBOX` nehmen wie alle Schalter
  BOOLEAN oder eine Zahl; ein Text ist ein Fehler (vorher war „nein“ wahr).

**Zahlen**

* Werte in Feldern und MAPs prüft dieselbe Regel wie Variablen:
  `ARRAY_PUSH(a, 1e19)` in ein `ARRAY OF INTEGER` ist ein Fehler (vorher still
  9223372036854775807), eine Zahl in ein `ARRAY OF GELD` ebenso.

**Handles**

* **Handles tragen ihre Art** (`Value::Handle`, `handle.rs`): `TYPEOF(p)`
  sagt `PDF` statt `INTEGER`, `p IS PDF` ist TRUE, `p IS INTEGER` FALSE.
  Ein Handle einer anderen Art an einem Befehl ist ein Fehler
  („XLSX_SHEET_COUNT: Argument 1 erwartet ein XLSX-Handle, erhalten ein
  PDF-Handle“) -- bis 2026.53 traf das PDF 0 still die Mappe 0; `--check`
  meldet es vorher. Eine INTEGER-Variable nimmt weiter die Nummer, eine Zahl
  an einer Variable des Handle-Typs wird zum Handle; `DIM d AS DB_CONN =
  TRUE` ist ein Fehler. Rechnen (`h + 1`) mit einem Handle ist ein Fehler,
  vergleichen (`h = 0`, `h < 0`) geht wie mit der Nummer.

* Ein freigegebenes gui-Widget oder -Fenster: Fragen geht weiter (Ereignisse
  FALSE), **Setzen ist ein Fehler** („GUI_SET_TEXT: das Widget wurde mit
  GUI_WIDGET_FREE freigegeben“); vorher ging es still durch.
* `MIDI_FILE_FREE` und `MQTT_IS_CONNECTED` mit einer Nummer, die es nie gab,
  sind Fehler (vorher still bzw. FALSE).
* `HTTP_READY` und `HTTP_RESULT` mit einer Nummer, die es nicht (mehr) gibt
  (abgeholt, abgebrochen, nie gestartet), sind Fehler wie bei `TASK_READY`;
  vorher lieferte `HTTP_READY` FALSE. Die Meldung „Abruf n ist nicht fertig
  (oder schon abgeholt)“ heißt jetzt „ist noch nicht fertig“, ebenso bei
  `SHELL_RESULT$` und `DB_QUERY_RESULT`.
* `CLOUD_*` geht durch denselben HTTP-Weg wie `HTTP_REQUEST`: ohne Netz
  dieselbe Meldung („keine Verbindung …“), Antworten über 64 MB sind ein
  Fehler.
* Neuer Name `MQTT_UNACKED_COUNT` für `MQTT_PENDING` (der alte bleibt).

**Kleinigkeiten**

* `SAVE_DELETE` liefert BOOLEAN (gab es den Schlüssel?) statt NIL; `LEN(b)`
  nimmt einen BUFFER (seine Bytes). `EOF` ist derselbe Befehl wie
  `ENDOFFILE` (Meldung „Datei wurde nicht im Lese-Modus geoeffnet“),
  `SPRITE_COLLIDE` heißt in den Parametern `sp1, sp2` wie `SPRITE_COLLIDES`.
* Hinweise in Meldungen nennen die neuen Namen (`SOUND_FREE`, `MONEY_NEW`,
  `FILE_OPEN`, `IMAGE_WIDTH`).

**Ton**

* Eine Lautstärke über 1 ist ein Fehler (vorher still 1); nur
  `AUDIO_SOUND_MIX` verstärkt beim Mischen bis 4. `GUI_SOUND` und die
  Stildatei nehmen 0..100 % (vorher 0..200, über 100 ohne Wirkung).
* Erzeugte Klänge (`AUDIO_TONE`, `AUDIO_NOISE`, `AUDIO_NOTE`, `AUDIO_SFX`,
  `SAMPLE_NOTE`) spielen mit Lautstärke 1 -- ihre Lautstärke steckt schon im
  Klang. Bis 2026.53 zählte sie doppelt: `AUDIO_TONE(…, 0.5)` klang wie 0.25,
  jetzt wie 0.5.
* `AUDIO_PITCH` hat dieselbe Grenze wie `AUDIO_MUSIC_PITCH` (> 0 und <= 64,
  sonst ein Fehler; dort vorher still 64).
* `AUDIO_DELAY` ohne `time_ms` nimmt die Vorgabe 300 ms (vorher blieb die
  alte Zeit stehen).

**Sprache der Eingaben und Ausgaben**

* Eingaben nehmen englische Zweitnamen: Wellenformen (eine Tafel für
  `AUDIO_TONE`/`_NOTE`/`_SFX`/`AUDIO_LFO_NEW`, auch „pulse“, „rechteck“,
  „saege“), chart-Schlüssel und -Wörter, Knopfarten, Formen, Sinnbilder,
  `GUI_TEXTAREA_SET`-Schlüssel, `TIME_PART`-Teile.
* Eingebaute Texte folgen der Sprache der Laufzeit: `KEY_NAME$` („SPACE“
  statt „LEER“ auf einem englischen System), die Knöpfe von Assistent und
  Ja/Nein-Dialog, der Schriftdialog, die Wochentage im Datumswähler.

**Eingabe**

* `WAITKEY` liefert Sondertasten als DH-Code wie die Konstanten
  (`WAITKEY() = KEY_LEFT` ist jetzt wahr; vorher kam der rohe raylib-Code)
  und wartet nicht mehr für immer, wenn kein FLIP dazwischen liegt.
* `JOY_BUTTON_*` gelten auch in `KEYHIT`, `KEYRELEASED`, `KEYREPEAT` und
  `JOYSTICK_BUTTON` (vorher still FALSE).
* `INPUT_BIND` nimmt einen Buchstaben oder eine Ziffer als Text, nach der
  Belegung wie `KEYHIT("z")`.

**Kindprozesse**

* `SHELL`, `SHELL_OUT$`, `SHELL_START` und `TASK_START` geben `DHRT_FRAMES`,
  `DHRT_SCREENSHOT` & Co. nicht mehr an Kinder weiter (wie `PROCESS_START`);
  „dhrt“ meint bei `SHELL*` die eigene Laufzeit, ein Feld von Texten wird
  ausgebreitet.

**Sprites und Partikel**

* `SPRITE_COLLIDES`/`SPRITE_HIT_*` rechnen mit `SPRITE_SET_SCALE` wie
  `SPRITE_DRAW` (vorher ohne Skala).
* `SPRITE_TINT`, `PARTICLE_SET_COLOR`/`_END` nehmen eine Deckkraft.

**gui-Farben**

* Übergänge mischen die Deckkraft mit (ein Glasknopf blieb mitten im
  Übergang deckend). Die `.dhform` liest `#RGB` wie die Stildatei.

**Namen**

* `IMAGE`, `SOUND` und `FILE` sind keine Schlüsselwörter mehr: ein Parameter
  `file` oder eine Variable `sound` geht; als Typ (`DIM f AS FILE`) gelten
  sie weiter.
* Die Parameter von `AUDIO_SET_VOLUME`/`AUDIO_MUSIC_SET_VOLUME` heißen
  `volume` (wie bei den alten Namen `AUDIO_VOLUME`), der von `FONT_SET`
  heißt `font` (wie bei `SETFONT`). Die bisherigen Namen `v` und `handle`
  gelten in benannten Argumenten weiter.
* Ein Befehl mit `$` und derselbe ohne sind wie bisher derselbe; die Laufzeit
  kennt intern nur noch eine Form (unsichtbar für Programme).

**Umgebungsvariablen**

* Alle heißen `DHRT_…`, englisch (Tafel in `docs/werkzeuge.md`,
  Umgebungsvariablen): `DHRT_PATH` statt `DH_PATH`, `DHRT_LOG` statt
  `DH_LOG`, `DHRT_NO_AUDIO`, `DHRT_NO_GRAPHICS`, `DHRT_CLIPBOARD`,
  `DHRT_PLAYBACK`, `DHRT_RECORD` … Die alten Namen gelten weiter; steht beides
  da, gewinnt der neue. `dhrt aufnehmen` schreibt in neue Fälle
  `DHRT_PLAYBACK=…`.

**Grafik und Eingabe**

* `SCROLL(dx, dy)` ist ein Fehler, der sagt, was stattdessen verschiebt
  (`CAMERA_SET`, ein Render-Ziel); bis 2026.53 tat es still nichts.
* `CLIPBOARD_SET_IMAGE` liefert nichts mehr (vorher immer TRUE) -- wie
  `CLIPBOARD_SET` ist ein Misserfolg ein Fehler. `IF CLIPBOARD_SET_IMAGE(b)`
  wird zu `CLIPBOARD_SET_IMAGE(b)`.
* Die Zahlenfelder der gui (`GUI_TEXTAREA_SPANS`, `_MARKS`, `_HINTS`,
  `_LINE_COLORS`, `_FOLDABLE`, `_SWATCHES`, `GUI_LISTBOX_SPANS`) nehmen nur
  ganze Zahlen; `2.0` geht, `2.5` ist ein Fehler (vorher still 2). Ein BUFFER
  geht auch.

**Geräte**

* `WIFI_CONNECT` liefert auf allen drei Systemen FALSE, wenn das System
  ablehnt; ein Fehler kommt nur bei leerer SSID oder fehlendem Werkzeug
  (bis 2026.53 unter Linux/macOS immer ein Fehler, unter Windows je nach
  Schritt). `WIFI_DISCONNECT` unter macOS sagt, ob es gelang (vorher immer
  TRUE). Ein verstecktes Netz steht mit leerem Namen in `WIFI_SCAN`/
  `WIFI_NETWORKS` (vorher „(versteckt)“).

**Netz**

* dhrt meldet sich bei Servern als `Drachenhauch/<Fassung>
  (+https://github.com/HasoSchno70/Drachenhauch)` (vorher
  „Drachenhauch/0.1“ mit einer fremden Adresse; cloud „Drachenhauch-cloud/0.1“).

**Kommandozeile**

* `dhrt run` (und `debug`, `profile`, `--check` …) ohne Datei endet mit Code
  2 und der passenden Zeile der Hilfe statt „Kann 'run' nicht lesen“ (Code 1).
  „Kann '…' nicht lesen“ spricht bei allen Einstiegen die Sprache der
  Laufzeit. `dhrt --runsrc` gibt es nicht mehr (war ein interner Rest).
* Die Kurzhilfe in Meldungen zeigt die Aufrufform ohne Rückgabetyp
  (`REPEAT$(s, n)`, nicht `REPEAT$(s, n) AS STRING`).
* `daten/builtin_index.json` hat kein Feld `kind` und kein `count` mehr
  (niemand las sie; `kind` passte nicht zu `module`).

## Neu

* `GUI_TREE_COUNT`, `GUI_TREE_SET_LABEL`, `GUI_TREE_REMOVE`,
  `GUI_TREE_EXPANDED`; `GUI_GET_COLOR` kennt „hover“ und „pressed“.
* `CHAR_UPDATE(c, map, layer, dt)`, `CHAR_SET_COYOTE_MS`,
  `CHAR_SET_JUMP_BUFFER_MS`.
* `_FREE`/`_CLOSE`-Namen: `AUDIO_CLOCK/EMITTER/LISTENER/MOD_FREE`,
  `GUI_WIDGET_FREE`, `GUI_WINDOW_FREE`, `PHYS2D/3D_BODY_FREE`, `BT_CLOSE`,
  `MQTT_CLOSE`.
* `ARC_DEG`, `PHYS2D_BODY_ANGLE_DEG`, `GUI_TEXTAREA_GET_VIEW`,
  `AUDIO_SET_PAN`/`_SLIDE` (Skala -1..1), `AUDIO_PLAY_VOICE`,
  `AUDIO_VOICE_COUNT`, `AUDIO_AUTOPAN_MS`, `BT_SCAN_MS`,
  `SAMPLE_SET_LOOP_TIME`; `AUDIO_SFX` mit Parameternamen `attack_ms`,
  `sustain_ms`, `decay_ms`, `vib_depth`, `vib_speed` (die alten gelten).
* Regeln 10 bis 12 in `docs/builtins-namen.md`: Winkel, Rechtecke und
  Strahlen, Zählen ab 0 oder 1.
* Bytes für Geräte und Netz: `SERIAL_WRITE_BYTES`/`_READ_BYTES`,
  `USB_WRITE_BYTES`/`_READ_BYTES`, `BT_WRITE_BYTES`/`_READ_BYTES` (BUFFER oder
  Feld aus Zahlen), `HTTPD_BODY_BYTES`; `MQTT_PUBLISH` und `HTTPD_SEND`
  nehmen einen BUFFER.
* Gerätelisten als ARRAY OF STRING: `SERIAL_DEVICES`, `USB_DEVICES` (mit dem
  HID-Pfad für `USB_OPEN_PATH`), `MIDI_IN_DEVICES`, `MIDI_OUT_DEVICES`,
  `WIFI_NETWORKS`; `BT_SCAN_MS` liefert ein Feld.
* Abfragen der gui-Einstellungen für jede Art: `GUI_LISTBOX_GET`,
  `GUI_TREE_GET`, `GUI_ACCORDION_GET`, `GUI_WIZARD_GET`, `GUI_TABCONTROL_GET`,
  `GUI_SLIDER_GET`, `GUI_DROPDOWN_GET`, `GUI_PROGRESS_GET`, `GUI_LAYOUT_GET`,
  `GUI_LABEL_GET`, `GUI_TEXTAREA_GET`, `GUI_RICHTEXT_GET`,
  `GUI_COLORPICKER_GET`, `GUI_DATEPICKER_GET`, `GUI_TIMEPICKER_GET`,
  `GUI_FILETREE_GET`; Platzhalter in einer Form für alle Arten
  (`GUI_TEXTINPUT_PLACEHOLDER`, `GUI_TEXTAREA_PLACEHOLDER`/`$`,
  `GUI_DROPDOWN_PLACEHOLDER$`, `GUI_TREE_PLACEHOLDER$`,
  `GUI_TABLE_PLACEHOLDER$`); `GUI_GET_MOTION_MODE$`.
* Neue Namen nach `docs/builtins-namen.md` (die alten bleiben): der Bereich
  vorn (`GUI_WINDOW_SET_TITLE`, `GUI_MENU_SET_ENABLED`, `CHAR_GET_X` …),
  Setzen heißt SET (`AUDIO_SET_PITCH`, `TEXT_SET_SIZE`, `MOUSE_SET_LOCK`,
  `SPI_SET_SPEED`, `TRAY_SET_MENU` …), Zählen heißt COUNT (`JSON_COUNT`,
  `XML_NAME_COUNT`, `ASTAR_PATH_COUNT`, `AUDIO_CHANNEL_COUNT`,
  `TILED_GID_COUNT`), dazu `SPRITE_DONE`, `MQTT_NEXT`, `MQTT_MESSAGE$`,
  `MQTT_TOPIC$`, `SAMPLE_DURATION`, `SAVE_GET_KEYS`, `PHYS3D_WORLD` (Typ).
* Tasten: `KEY_NUM_LOCK`, `KEY_SCROLL_LOCK`, `KEY_PRINT_SCREEN`, `KEY_PAUSE`
  (bis 2026.53 ohne Code), Zweitnamen `KEY_LEFT_SHIFT` …, `KEY_KP_0` …,
  `KEY_PAGE_UP`/`_DOWN`, `KEY_CAPS_LOCK`.
* `CAMERA3D_UPDATE` nimmt den Modus als Wort: "free"/"frei",
  "orbital"/"umkreisend", "first_person"/"ego", "third_person"/"verfolger",
  "custom"/"eigen".
