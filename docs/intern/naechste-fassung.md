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

* Ein freigegebenes gui-Widget oder -Fenster: Fragen geht weiter (Ereignisse
  FALSE), **Setzen ist ein Fehler** („GUI_SET_TEXT: das Widget wurde mit
  GUI_WIDGET_FREE freigegeben“); vorher ging es still durch.
* `MIDI_FILE_FREE` und `MQTT_IS_CONNECTED` mit einer Nummer, die es nie gab,
  sind Fehler (vorher still bzw. FALSE).

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
