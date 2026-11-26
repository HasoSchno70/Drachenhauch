# Für die Notizen der nächsten Fassung

Sammelstelle, bis die Fassung ihre Notizen bekommt
(`docs/releases/release-<fassung>.md` und `.en.md`). Was hier steht, gehört
in den ersten Abschnitt „Was sich für bestehende Programme ändert“. Herkunft:
die Reste aus `docs/intern/inkonsistenzen-runtime.md`, Zweig
`thema/lesen-verbinden`.

## Was sich für bestehende Programme ändert

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

**Zahlen**

* Werte in Feldern und MAPs prüft dieselbe Regel wie Variablen:
  `ARRAY_PUSH(a, 1e19)` in ein `ARRAY OF INTEGER` ist ein Fehler (vorher still
  9223372036854775807), eine Zahl in ein `ARRAY OF GELD` ebenso.

## Neu

* `GUI_TREE_COUNT`, `GUI_TREE_SET_LABEL`, `GUI_TREE_REMOVE`,
  `GUI_TREE_EXPANDED`; `GUI_GET_COLOR` kennt „hover“ und „pressed“.
* `CHAR_UPDATE(c, map, layer, dt)`, `CHAR_SET_COYOTE_MS`,
  `CHAR_SET_JUMP_BUFFER_MS`.
* `_FREE`/`_CLOSE`-Namen: `AUDIO_CLOCK/EMITTER/LISTENER/MOD_FREE`,
  `GUI_WIDGET_FREE`, `GUI_WINDOW_FREE`, `PHYS2D/3D_BODY_FREE`, `BT_CLOSE`,
  `MQTT_CLOSE`.
