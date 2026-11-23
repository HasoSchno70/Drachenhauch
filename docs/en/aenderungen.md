# Change log

What changes for **existing programs** in each version, newest first --
collected from the release notes. What stays fixed and what may change is
described in [Stability](stabilitaet.md).

*Generated with `dhrt doku aenderungen` from `docs/releases/`; changes belong
in the release notes, not here.*

## 2026.53

**Compiling**

* **Assigning to a `CONST` is a compile error**, even in code that never
  runs (before: only at run time, and `--check` stayed silent). If a
  parameter, a local variable or a field hides the constant, the assignment
  stays allowed.
* **An ENUM value that does not exist** (`Stoff.SNAD`) is a compile error,
  with a suggestion.
* **A `COROUTINE` variable only takes coroutines.**
* **New warnings:** `a BAND b + c` without parentheses (it reads as
  `a BAND (b + c)`; the same for `BOR`, `BXOR`, `SHL`, `SHR`), a coroutine
  used as a bare statement (it would never run), `TRY … END TRY` without
  `CATCH` and `FINALLY` (it swallows the error, as before).

**Graphics**

* **raylib 6.2** draws differently in a few places where 6.0 was wrong:
  `NGONOUTLINE` with a thickness is as thick as the manual says (before
  about a quarter), circles in images are one column wider on the right,
  the pose of a GLTF animation differs slightly, and the frame timer waits
  actively for about 0.8 ms per frame. Gamepads get raylib's own mappings.
* **`SHADER_LOAD` with a missing file is an error** (with a suggestion)
  instead of returning -1. An unknown uniform in `SHADER_SET…` warns once.
* **`BLEND_MODE("subtract")`** subtracts the colour from the background and
  leaves the alpha alone (before: the other way round, alpha included).
* **`RENDERTARGET_BEGIN` without `RENDERTARGET_END` before `FLIP`** stops
  with a message instead of carrying on black.
* **Point lights** are brighter at a distance (the same fall-off as with
  instancing): at 4 units about half as bright, at 10 a fifth.
  `MODEL_INSTANCED` fades into fog like `MODEL`.
* **`MESH_TORUS` and `MESH_KNOT` are as big as given** -- rings and knots
  appear larger than before.
* **`LOADIMAGE` reloads a file that was written anew** (new handle, the old
  one stays valid) instead of returning the old image from the cache.

**Sound**

* **MOD and XM music is about 5 dB quieter**: the player now applies the
  volume the module itself asks for, and no longer clips.
* **`AUDIO_MOD_REMOVE`** restores the parameter's fixed value instead of
  leaving the modulator's last value in place.
* **`AUDIO_FFT` and `MIC_FFT` decay by the clock.** If you ask more than
  sixty times a second, the bands fall more slowly than before.

**Physics**

* **A removed `physics2d` body reports itself** on every query instead of
  quietly returning (0, 0); `PHYS2D_BODY_EXISTS` asks first.

**User interface**

* **`GUI_VALIDATE_LIVE`** checks a field on leaving only if it changed
  since it got the focus; an empty required field is reported by
  `GUI_VALIDATE`.
* **An empty number field** binds `NULL` instead of an empty text.
* **Row colour and zebra stripes of a table** reach the edge.
* **`GUI_DIALOG`** is at most 640 wide and wraps long lines.
* **Muted text (`muted_fg`) and the selected table row** have a contrast of
  at least 4.5:1 in every theme; some colours are stronger for it.

[All notes for 2026.53](../releases/release-2026.53.en.md)

## 2026.52

* **Messages in the language of the system.** Without `DHRT_LANG`, dhrt
  writes its messages in German on a German system and in English on any
  other (before: always German). What a program sees in a `CATCH` variable
  stays German. If you compare output, set `DHRT_LANG=de`.
* **Without a setting, the IDE and the Studio follow the system as well.** On
  an English Windows they start in English; in the IDE, View → "Sprache:
  Deutsch" switches at the next start, and the Studio follows the IDE's
  setting.
* **`dhrt --export` takes the smallest runtime that fits on its own**
  (before only with `--schlank`, which still works). `--voll` takes the full
  one -- for instance for a program that checks with `IMPORT … OPTIONAL` and
  `MODULE_AVAILABLE` at run time what the build can do.
* **A `.dhc` from another version is rejected** ("Diese .dhc stammt aus dhrt
  …"), instead of running differently without a word. One without a version
  (compiled before 2026.52) runs as before.
* **`DIRLIST(path, "")`** returns all entries (before: none).
* **The signatures name English parameter names** (`color` instead of
  `farbe`) -- in the hover, in completion and in the manuals. Parameters with
  a default are no longer part of the completion scaffold.

[All notes for 2026.52](../releases/release-2026.52.en.md)

## 2026.51

* **New warning: spelled differently from the declaration.** Whoever
  declares `CONST CW` and later writes `cw` gets a warning when compiling
  (on stderr for `dhrt run`, the yellow line in the editor). The program
  runs as before -- upper/lower case still does not matter. Ctrl+. or
  `dhrt fmt --schreibweise` writes the names as declared.
* **A label measures its width by its text** until `GUI_SET_BOUNDS` or
  `GUI_SET_WRAP` sets one; `GUI_GET_W` returns the real width.
* **The font of a button with a gradient** follows the least favourable
  spot of the gradient, and `GUI_A11Y_CHECK$` measures there.
* **Lists in a mirrored window** (`GUI_WINDOW_RTL`) now mirror their inside
  as well.
* **Circuit Runner:** the robots move half as often as before (key T in the
  menu switches to the original); recorded solutions only fit at the
  original speed.

[All notes for 2026.51](../releases/release-2026.51.en.md)

## 2026.50

* **Coloured buttons have the more readable text.** On a button with its
  own background colour (primary, success, warning) the gui now picks the
  text by the higher contrast: on the cyan of the dark theme, on green and
  orange there is dark text instead of white (before 2.1:1 to 3.4:1). The
  accent of `modern_light` is slightly darker (`#1D6DCE`).
* **Emoji come in colour.** The text colour no longer applies to them, only
  its opacity.
* **Mirrored windows (`GUI_WINDOW_RTL`)** now also mirror text areas, tables
  and trees inside, and their scroll bars sit on the left.
* **`WINDOW_WAIT_EVENTS`** waits per frame exactly as long as nothing is
  pending: after an input it runs on for 0.35 s without waiting, with a
  fixed frame count (`DHRT_FRAMES`) it never waits.
* **Caret and tooltip** count in seconds instead of frames -- at 60 frames
  a second as before.

[All notes for 2026.50](../releases/release-2026.50.en.md)

## 2026.49

* **Ctrl+Shift+P opens the command palette in every program with a menu
  bar or toolbar.** If the program has a menu shortcut of its own on that
  key, its own wins; `GUI_PALETTE_KEY(FALSE)` switches the key off. While
  the palette is open, no text field below it gets the keys.
* **Programs started from the IDE** open the inspector with
  **Ctrl+Shift+I** (the IDE sets `DHRT_INSPEKTOR=1`). A program started on
  its own notices nothing of it.
* **A click into a right-aligned text field** (`GUI_SET_ALIGN`) now puts the
  caret at the click position; before, it ended up at the end. The
  placeholder of such a field no longer runs over the edge.
* **The IDE's completion** offers only types after `AS` and `OF` --
  `DIM w AS GUI_WIN` used to take the first matching command.

[All notes for 2026.49](../releases/release-2026.49.en.md)

## 2026.48

* **During a modal dialog its own menus work.** Until now every menu was
  locked during a modal dialog -- including the context menu and the menu
  bar in the dialog itself. Now the dialog's work, those of the windows
  behind it stay locked.
* **`GUI_DOCK_WHERE$` knows a new place:** `fenster` for a torn-off panel.
  A program that evaluates the answer with `SELECT CASE` sees this value
  only when a panel is torn off.
* **Recordings (`AUTOMATION_RECORD`)** write additional lines `n ...` at the
  end of the file for real windows. raylib's reader skips them; old
  recordings play unchanged.

[All notes for 2026.48](../releases/release-2026.48.en.md)

## 2026.47

* **A click goes to the topmost widget.** When two widgets overlapped, the
  one created first used to get the click, while the one created last was
  drawn on top -- that is why the OK button in the "About Drachenhauch"
  window could only be pressed with Enter. Now clicking, drawing and
  `GUI_HIT_TEST` all use the same order. Whoever (unintentionally) relied on
  a widget underneath getting the click creates it last.
* **A label draws its own background and border:**
  `GUI_SET_COLOR(lbl, "bg", ..)` and `"border"` as well as `bg`/`border` from
  the style sheet did not apply to `GUI_LABEL` so far. A style sheet that
  gives labels a background now shows it.
* **The border of a large focused text area is quieter:** above the metric
  `verlauf_hoehe` the accent mixes with the border colour. An accent set on
  the widget applies unchanged.

[All notes for 2026.47](../releases/release-2026.47.en.md)

## 2026.46

* **Glutfaden, Funkenflug and the fridge app are no longer in this repository
  and no longer in the installer.** They have their own repositories:
  [Glutfaden](https://github.com/HasoSchno70/Glutfaden) (cross-stitch editor),
  [Funkenflug](https://github.com/HasoSchno70/Funkenflug) (schematic editor) and
  [Kuehlschrank-Drachenhauch](https://github.com/HasoSchno70/Kuehlschrank-Drachenhauch)
  (Studio project), each with its full history and under the same licence.
  They run with this version as before.
* **The form designer (`examples/197_form_designer.dh`) is gone**; Drachenhauch
  Studio replaces it. A `.dhform` in the IDE's project tree now opens in the
  Studio, and the installer removes the old shortcut.
* **Games and books have folders of their own:** `spiele/` (Pyramid Pusher,
  Circuit Runner, Voidreaver, Restaurant, Fireplace) and `buecher/`
  (`referenz`, `einstieg`, `galaga`, `tippspiel` instead of `buch-*`). Starting
  them from the source tree takes the new path, for example
  `dhrt run spiele/pyramid_pusher/pyramid_pusher.dh`. The vocabulary lists of
  the beginner's book stay under `buch-einstieg/vokabellisten/`, so the address
  printed in books already handed out keeps working.
* **Physics (`PHYS2D_*`, `PHYS3D_*`) runs on Rapier 0.36 instead of 0.22.**
  Trajectories may differ in the last digits. A stack created asleep still
  rests until the first push, but settles by a little over a millimetre in the
  very first step, where it used to stay exactly in place.

[All notes for 2026.46](../releases/release-2026.46.en.md)

## 2026.45

*Only in German so far:*

* **Enter auf einem Kästchen, Schalter, Optionsfeld oder Zahlenfeld drückt den
  Standard-Knopf** (wie unter Windows, macOS, Qt und im Web). Vorher schaltete
  Enter das Kästchen um, und ein Dialog ließ sich nicht mit Enter bestätigen;
  umgeschaltet wird mit der Leertaste.
* **In einer nicht bearbeitbaren Klappliste wählt `GUI_SET_TEXT` den Eintrag**
  mit diesem Text, und `GUI_TEXT` liefert die Auswahl. Vorher setzte es einen
  Text, den die Liste nicht zeigte.
* **`HTTP_STATUS` ist 0, wenn eine Anfrage ohne Antwort scheiterte** (kein
  Netz, Zeitüberschreitung) -- vorher stand dort der Status der Anfrage davor.
* **Ein Klick in das Monatsgitter (`GUI_DATEPICKER`) trifft den Tag auch mit
  `wochenbeginn` ungleich Montag.** Vorher zählte der Klick ab Montag, mit
  „Woche ab Sonntag“ traf er den Tag daneben.
* **`AUDIO_SFX`: der Tiefpass schaukelt sich nicht mehr auf.** Bei Resonanz
  und hoher Grenzfrequenz kippte er auf NaN, und der ganze Ton blieb danach
  stumm (gefunden mit Lichterglanz im Browser). Darunter klingt alles wie
  bisher.
* **Im Studio erreicht eine Taste mit Strg oder Alt und jede F-Taste das
  Ereignis `Taste` auch aus einem Eingabefeld**, außer das Feld braucht sie
  selbst (kopieren, einfügen, Pfeile …) oder es ist AltGr. Ein Fenster ohne
  Menüleiste hat so Kürzel.
* **Das Studio baut beim Abgleich einer Datenbank Tabellen ohne
  Fremdschlüssel-Aktionen um.** Vorher nahm `DROP TABLE` beim Umbau über
  `ON DELETE CASCADE` alle Zeilen mit, die auf die Tabelle zeigten -- bei einer
  übernommenen Datei mit Fremdschlüsseln ein stiller Datenverlust.

[All notes for 2026.45](../releases/release-2026.45.md)

## 2026.44

*Only in German so far:*

* **Farbverläufe sehen anders aus – ohne Stufen.** Abgerundete Flächen der gui
  und `GRADIENTV`/`GRADIENTH` (auch in den Farbwählern) runden jede Zeile gegen
  eine eigene Schwelle, statt alle drei Kanäle im selben Moment auf die nächste
  Stufe zu springen. Die sichtbaren Bänder (bis zu 32 Zeilen breit) sind weg,
  die Enden sind genau die angegebenen Farben. Wer ein Bild Punkt für Punkt
  vergleicht, sieht dazwischen leicht andere Werte.
* **Auf großen gui-Flächen ist die Fase leiser** (mit `verlauf_hoehe`): eine
  helle Kante über einer Leiste von 200 Punkten wirkte wie ein Rahmen aus
  Windows 95. Ohne die Metrik bleibt alles, wie es war.
* **`--check` warnt bei Parametern und Klassenfeldern, die eine Konstante
  verdecken** – aber nur, wenn der Name im Block auch anders geschrieben
  vorkommt (`n` neben `N`). Bisher gab es die Warnung nur für lokale
  Variablen; `FUNCTION summe(n AS INTEGER) ... RETURN n + N` rechnete still
  `n + n`. Wer die Warnung sieht, hat sehr wahrscheinlich genau diesen Fehler.

[All notes for 2026.44](../releases/release-2026.44.md)

## 2026.43

*Only in German so far:*

* **Unter Linux funktionieren Render-Ziele jetzt** – `RENDERTARGET_*`,
  `POSTFX` und die HDR-Umgebung (`LIGHT_ENV_HDR`) blieben dort bisher schwarz,
  in jedem Linux-Paket. Die Ursache steckte in raylib (mit gcc übersetzt legte
  es leere Texturen als komprimierte an). Wer unter Linux deshalb auf Effekte
  verzichtet hat, bekommt sie jetzt. Windows und macOS sind unberührt.
* **`FONT_TO_IMAGE`** liest die Zeichenbilder statt des Atlas; das Ergebnis
  ist dasselbe, geht jetzt aber auch unter OpenGL ES.
* **`MQTT_UPDATE` liest alles, was angekommen ist**, nicht mehr ein Stück je
  Aufruf. Wer viele Nachrichten bekommt, sieht sie früher.
* **Ein MIDI-Eingang lässt nur noch SysEx weg.** Uhr-Takte, Zeitcode und
  Active Sensing kommen an, gehen aber in die neue MIDI-Uhr statt in die
  Warteschlange – `MIDI_NEXT` sieht also dasselbe wie bisher.

[All notes for 2026.43](../releases/release-2026.43.md)

## 2026.42

*Only in German so far:*

* **Ein Reiterwerk (`GUI_TABCONTROL`) gibt Klicks an die Widgets seiner
  Seiten weiter.** Bisher schluckte es jeden Klick auf ein Kind: Knöpfe auf
  einer Seite meldeten nie `GUI_CLICKED`, Felder bekamen per Klick keinen
  Fokus. Wer sich mit Tastatur oder `GUI_FOCUS` darum herumgeholfen hat,
  bekommt die Klicks jetzt zusätzlich.
* **`PDF_PREVIEW` setzt Text in der Schrift des PDFs** – den eingebauten
  DejaVu-Schnitten oder einer mit `PDF_FONT_LOAD` geladenen Schrift – und
  Zeichen für Zeichen an die Stelle, die auch das PDF rechnet. Vorher stand
  der Text in raylibs Pixelschrift, und ein rechtsbündiger Betrag endete ein
  Stück zu früh. Vorschaubilder sehen deshalb anders aus als bisher, dafür
  wie das Papier.

[All notes for 2026.42](../releases/release-2026.42.md)

## 2026.41

*Only in German so far:*

* **Ein Drehfeld (`GUI_SPINNER`) nimmt Tastatur-Eingaben.** Bisher kannte es
  nur die Pfeiltasten und seine zwei Knöpfe. Mit dem Fokus ist der Wert jetzt
  markiert, und das erste Getippte ersetzt ihn. Wer bisher Buchstaben- oder
  Zifferntasten abfragte, während ein Drehfeld den Fokus hatte, bekommt sie
  dort nicht mehr – sie gehören dem Feld, wie bei einem Textfeld.

[All notes for 2026.41](../releases/release-2026.41.md)

## 2026.40

*Only in German so far:*

* **Ein fehlendes Blockende heißt jetzt so, wie es ist.** Kommt ein
  `END SUB`, während ein `IF` noch offen ist, meldete der Übersetzer
  „Erwartet IF nach END“ – das las sich, als fehle ein `IF`. Jetzt heißt es
  „END IF fehlt -- hier steht END SUB, aber das IF davor ist noch offen“
  (bzw. „Erwartet END IF“), für alle Blöcke von `IF` bis `STRUCT`. Wer eine
  solche Meldung als Text auswertet, passt den Vergleich an; die
  Schnellkorrektur der IDE kennt beide Formen.
* **`dhrt --export` legt keine eingebundenen Quelltexte mehr neben die
  Exe.** Eine per `IMPORT "datei.dh"` eingebundene Datei galt bisher als
  „verwendete Datei“ und wurde mitkopiert – der Quelltext lag dann offen im
  Bau. Er steckt ohnehin im Bytecode; Bilder, Klänge und andere Dateien
  kommen weiter mit.

[All notes for 2026.40](../releases/release-2026.40.md)

## 2026.39

*Only in German so far:*

* **Die gui zeichnet in der eingebauten Sans statt in der Pixelschrift.**
  Ohne `GUI_FONT` und ohne `SETFONT` nehmen Fenster, Knöpfe, Listen, das
  Modul `ui` und die Diagramme DejaVu Sans in der Textgröße des Programms.
  `TEXT()` und die übrigen Zeichenbefehle bleiben pixelig, Spiele sehen aus
  wie bisher. Wer den alten Anblick will: `GUI_FONT(-1)` schaltet gui, `ui`
  und Diagramme gemeinsam zurück. Die Felder passen unverändert, weil die
  Versalhöhe der Sans kleiner ist als die der Pixelschrift.
* **Meldungen zählen die Dimension eines Feldes ab 1** („Index 5 ausserhalb
  [0..4] in Dimension 1“, vorher „Dimension 0“). Wer eine Meldung mit
  `CATCH` auswertet, prüft die Zahl nach.
* **Text mit Hebräisch oder Arabisch wird von rechts nach links
  gezeichnet**, in den Textbefehlen und jedem Widget (nicht im PDF),
  Arabisch mit seinen verbundenen Formen. Wer eine Zeile schon selbst umgedreht hat, bekommt sie
  jetzt doppelt gedreht und lässt das Umdrehen weg.

[All notes for 2026.39](../releases/release-2026.39.md)

## 2026.38

*Only in German so far:*

* **`\` rundet nach unten wie `FLOOR`**: `-7 \ 2` ist jetzt -4 statt -3.
  `MOD` hat das Vorzeichen des Teilers, und mit dem Abschneiden war
  `(a \ b) * b + a MOD b` bei negativen Zahlen nicht `a`. Für positive Zahlen
  ändert sich nichts. Gilt in der VM, beim Konstantenfalten und im
  Maschinencode.
* **`MODEL_INSTANCED` mit einer Farbe je Instanz** zeichnet in EINEM Aufruf
  statt je Farbe einmal (2600 Würfel: 2380 statt 653 Bilder je Sekunde). Die
  Farbe reist dafür in der untersten Zeile der Matrix mit -- eine
  Projektionsmatrix als Instanz-Matrix geht damit nicht mehr.
* **`DRAWIMAGEPARTEX` mit einem elften Argument ist ein Fehler.** Bisher
  wurde es still übergangen; das zehnte ist jetzt eine Farbe (siehe unten).

[All notes for 2026.38](../releases/release-2026.38.md)

## 2026.37

The notes for this version have no separate section for this.

[All notes for 2026.37](../releases/release-2026.37.md)

## 2026.36

*Only in German so far:*

* **`GUI_CHANGED` meldet bei einem Baum die Auswahl eines Knotens.** Bisher
  blieb es dort stumm, obwohl die Doku „Auswahl“ nannte; ein Programm, das
  `GUI_CHANGED` auf einem Baum abfragt, bekommt jetzt TRUE, wenn ein Knoten
  angeklickt wurde.
* **Befunde von Plugins** sind jetzt im Code unterstrichen und stehen am
  Zeilenende, wie die Meldungen der Laufzeit. Vorher standen sie nur in der
  Liste der Probleme.
* **`dhrt test` lehnt Text hinter `--- programm` ab** (und hinter den anderen
  Köpfen ohne Inhalt wie `--- verzeichnis` oder `--- zeitgrenze`). Solcher
  Text wurde bisher ohne Meldung übergangen; der Quelltext eines Falls, der
  versehentlich hinter `--- programm` stand, lief also nie. Leerzeilen und
  Kommentare bleiben erlaubt. Keine der 444 Sammlungen im Repository war
  betroffen.

[All notes for 2026.36](../releases/release-2026.36.md)

## 2026.35

*Only in German so far:*

* **Die Zuweisung einer ganzen MAP wird geprüft.** Bisher reichte die
  Laufzeit jede MAP an jedes MAP-Ziel durch: Eine `MAP OF STRING` ging an
  eine `MAP OF INTEGER`, ein Literal mit einer Katze an eine `MAP OF Hund`.
  Jetzt gelten dieselben Regeln wie für Felder:
  * Gleicher Werttyp oder der einer Unterklasse passt. Die MAP behält ihren
    Werttyp, jeder Schreibzugriff prüft ihn.
  * Bei einer MAP ohne Werttyp wird jeder Wert geprüft
    (`Wert 'mia': Erwartet HUND ...`). Gehört sie sonst niemandem, nimmt sie
    den Werttyp an; so wird eine frische MAP von Ganzzahlen auch eine von
    Kommazahlen.
  * Sonst bricht die Zuweisung ab.

  Ein MAP-Literal hat nur noch einen Werttyp, wenn alle Werte dieselbe Art
  haben. Bisher gab der erste ihn vor, und `{"a": 1, "b": "x"}` galt als
  `MAP OF INTEGER` mit einem Text darin. Im Repository bricht dadurch kein
  Programm ab (geprüft mit `dhrt --check` über alle `.dh`-Dateien und mit
  allen Prüfsammlungen).

[All notes for 2026.35](../releases/release-2026.35.md)

## 2026.34

*Only in German so far:*

* **Der Typ einer Klasse wird geprüft.** Eine Variable, ein Parameter, ein
  Feld, ein Feldelement, ein MAP-Wert oder eine Rückgabe vom Typ `Tier` nimmt
  nur noch `NIL` und Instanzen von `Tier` oder einer Unterklasse. Bisher
  reichte die Laufzeit alles durch: `DIM t AS Tier : t = NEW Auto()` lief, und
  der Fehler kam erst beim nächsten Methodenaufruf, weit weg von der Stelle.
  Jetzt bricht die Zuweisung selbst ab:

  ```
  Laufzeitfehler in spiel.dh:12: Zuweisung an 't': Erwartet TIER (oder eine Unterklasse), erhalten AUTO
  ```

  `dhrt --check` warnt vorher, wo es sicher nie passt. Herunter
  (`DIM h AS Hund : h = einTier`) geht weiter, wenn in `einTier` wirklich ein
  Hund steckt.
* **Felder behalten ihren Elementtyp.** Ein Feld weiß zur Laufzeit, was es
  enthält, und prüft jeden Schreibzugriff selbst. Daraus folgt:
  * Ein Literal mit Objekten wird beim Zuweisen an ein `ARRAY OF X` Element
    für Element geprüft. Bisher ging `h = [NEW Katze()]` an ein
    `ARRAY OF Hund` ungeprüft durch.
  * Ein Feld einer Unterklasse passt an ein Feld der Vorfahrin
    (`ARRAY OF Hund` an `ARRAY OF Tier`). Über den neuen Namen kommt trotzdem
    keine Katze hinein. Bisher hing es daran, wie das Feld entstanden war.

  Im Repository bricht dadurch kein Programm ab (geprüft mit `dhrt --check`
  über alle `.dh`-Dateien und mit allen Prüfsammlungen).

[All notes for 2026.34](../releases/release-2026.34.md)

## 2026.33

*Only in German so far:*

* **Ein zweites `DIM` desselben Namens mit anderem Typ ist ein Fehler.**
  Gross- und Kleinschreibung zählen nicht, `pd` und `pD` sind also dieselbe
  Variable. Bisher gab es dafür nur eine Warnung, und abgebrochen wurde erst
  beim Laufen, an einer ganz anderen Stelle („Feld 'x' existiert nicht in
  b“). Das gilt im Hauptprogramm, in SUB/FUNCTION und in Methoden, ebenso für
  `CATCH e` mit einem späteren `DIM e AS T`. Ein zweites `DIM` mit
  **gleichem** Typ bleibt eine Warnung. Im Repository löst kein Programm den
  Fehler aus (geprüft mit `dhrt --check` über alle `.dh`-Dateien).

[All notes for 2026.33](../releases/release-2026.33.md)

## 2026.32

*Only in German so far:*

* **BYREF bei Methoden schreibt zurück.** `obj.m(a, b)` mit BYREF-Parametern
  lief bisher still wie BYVAL — die Werte kamen nie zurück. Steht die Klasse
  beim Übersetzen fest (auch bei `Self`), kommen sie jetzt zurück. Ist der Typ
  des Objekts nicht bekannt und wird ein BYREF-Parameter übergeben, ist das ein
  Fehler mit dem Hinweis auf `DIM x AS Klasse` (vorher: still BYVAL). Eine
  Überschreibung, die BYREF an anderen Stellen hat, ist ebenfalls ein Fehler.
* **Unter macOS und Linux gilt die Ortszeit.** Bisher war dort die ganze
  Ortszeit UTC: `DATE$`, `TIME$` und `ZEIT_JETZT` gingen in Berlin im Sommer
  zwei Stunden nach, und `ZEIT_PARSE` mit Zone sowie `FILETIME` rechneten
  nicht um. Jetzt kommen Felder und Versatz (samt Sommerzeit am jeweiligen
  Tag) aus der C-Bibliothek, mit den Regeln aus `TZ` bzw. `/etc/localtime`.
  Wer dort Zeiten gespeichert hat, sieht sie um den Zonenversatz verschoben.
  Unter Windows ändert sich nichts.
* **Fehler in Rückrufen fremder Bibliotheken.** Nach einem Fehler laufen
  weitere Rückrufe weiter (bisher lieferte ab dem ersten jeder stumm 0 — in
  einer GTK-Oberfläche reagierte danach kein Knopf mehr). Gemeldet wird der
  Fehler von dem Aufruf, der den Rückruf ausgelöst hat, nicht mehr vom
  nächsten zurückkehrenden irgendwo später.
* **Fehlerorte nennen die echte Datei.** Ein Laufzeitfehler in einer
  importierten Datei sagt `hilfe.dh:3` statt der Zeile in der
  zusammengefügten Quelle unter dem Namen der Hauptdatei; ein Rückruf-Fehler
  sagt `(fall.dh:6)` statt `(Zeile 6)`. `dhrt --check` liefert die Zeile in
  der eigenen Datei und ein neues Feld `file`. Wer Fehlertexte vergleicht oder
  die JSON-Ausgabe auswertet, sieht anderen Text.
* **`TYPEOF` bei Klassen aus einem Namensraum** liefert `KERN.KNOTEN` statt
  des internen `KERN@KNOTEN`; dieselben lesbaren Namen stehen jetzt in
  `--check`, im Sprachserver und in `CODE_CHECK$`.
* **`ZIP_WRITE`** nimmt in `inhalte` Texte und Puffer; alles andere ist ein
  Fehler. Bisher landete ein Puffer still als „<BUFFER n Bytes>“ im Archiv.
* **JSON-Pfade:** `\.` steht für einen Punkt im Schlüssel, `\\` für einen
  Rückstrich (dafür `JSON_KEY$`). Jede andere Folge mit Rückstrich bleibt
  wörtlich — betroffen sind nur Pfade, die `\.` oder `\\` schon enthielten.

[All notes for 2026.32](../releases/release-2026.32.md)

## 2026.31

*Only in German so far:*

* **`GUI_FORM_SAVE` auf eine gelöschte Zeile liefert 0.** Bisher kam die
  id zurück, als wäre gespeichert worden, und die Eingabe war still weg.
  `GUI_FORM_CONFLICT$` sagt dann `geloescht`.
* **`MQTT_UPDATE` wirft nicht mehr, wenn der Broker die Verbindung
  abbricht.** Wie beim net-Modul in 2026.30 ist das ein Zustand:
  `MQTT_IS_CONNECTED` wird FALSE, `MQTT_PUBLISH` und `MQTT_SUBSCRIBE`
  liefern jetzt einen Wahrheitswert. Ein Dashboard ohne `TRY` endete
  bisher beim Neustart des Brokers.
* **Befehle nehmen eine glatte Kommazahl als ganze Zahl** — mit derselben
  Regel wie eine Zuweisung an eine INTEGER-Variable. `/` liefert seit
  2026.16 immer eine Kommazahl, und `GUI_WINDOW("…", BREITE / 2 - 170, …)`
  brach mit „erwartet INTEGER“ ab, obwohl `x = BREITE / 2` ging. `2.5`
  bleibt ein Fehler, die Meldung nennt `INT`, `ROUND` und `\`.

  ```basic
  CONST BREITE = 800
  PRINT LEFT$("abcd", 6 / 3)      ' ab -- 2.0 gilt als 2
  ```
* **`CHART_DATA` mit weniger Punkten kürzt das Diagramm.** Bisher blieb die
  alte Punktzahl stehen, und nach „Woche, dann ein Tag“ hingen Tausende
  Nullen hinter dem Tag.
* **`ZEIT_FORMAT$` versteht die Schreibweise von C und Python**
  (`%Y-%m-%d %H:%M:%S`, `%a`, `%A`, `%%`). Bisher kam so ein Muster wörtlich
  zurück; ein `%`-Platzhalter, den es nicht gibt, ist jetzt ein Fehler.
* **`GENTEX_RADIAL` mit einer `dichte` außerhalb von 0 bis 1 ist ein
  Fehler.** Vorher entstand still eine harte Scheibe statt eines weichen
  Lichts.

[All notes for 2026.31](../releases/release-2026.31.md)

## 2026.30

*Only in German so far:*

* **Bricht die Gegenseite einer Verbindung hart ab, ist das kein
  Laufzeitfehler mehr.** Programm beendet, Absturz, Kabel gezogen:
  `NET_RECV` liefert dann `""`, `NET_RECV_BYTES` einen leeren Puffer,
  `NET_SEND` 0, und `NET_IS_CONNECTED` wird FALSE — so stand es schon in der
  Doku. Bisher warfen `NET_RECV` und `NET_SEND` „Eine vorhandene Verbindung
  wurde vom Remotehost geschlossen“, und unter Windows warf auch
  `NET_UDP_RECV`, sobald ein früheres Paket an einen Port gegangen war, an
  dem niemand mehr lauschte. Ein Spielwirt starb so, sobald sein Gast das
  Fenster schloss. Wer den Fehler mit TRY abgefangen hat, fragt jetzt
  `NET_IS_CONNECTED`; bei UDP sagt nur die Zeit seit dem letzten Paket, ob
  der andere noch da ist.

  ```basic
  IMPORT "net"
  DIM lst AS NET_LISTENER : lst = NET_TCP_LISTEN(0, "127.0.0.1")
  DIM gast AS NET_SOCKET : gast = NET_TCP_CONNECT("127.0.0.1", NET_LISTENER_PORT(lst))
  SLEEP(50)
  DIM s AS NET_SOCKET : s = NET_TCP_ACCEPT(lst)
  NET_CLOSE(gast)
  SLEEP(100)
  DIM t AS STRING : t = NET_RECV(s, 100)
  IF NOT NET_IS_CONNECTED(s) THEN PRINT "Gast ist weg"
  ```

[All notes for 2026.30](../releases/release-2026.30.md)

## 2026.29

*Only in German so far:*

* **`AUDIO_PLAY_AT` stoppt die vorige Instanz desselben Klangs nicht mehr.**
  Ein Sequencer, der dieselbe Hi-Hat mehrere Schritte voraus plant, löschte
  damit bisher die wartende — am Mischer kam von zwei Einsätzen einer an.
  Jetzt legt jedes `AUDIO_PLAY_AT` eine weitere Instanz an; `AUDIO_STOP`,
  `AUDIO_PAUSE`, `AUDIO_RESUME`, `AUDIO_VOLUME`, `AUDIO_PITCH` und die
  Pan-Befehle wirken auf alle. `AUDIO_PLAY` spielt den Klang weiter genau
  einmal.
* **`AUDIO_IS_PLAYING` ist für einen geplanten Klang erst ab seinem Tick
  wahr.** Vorher galt er schon als spielend, während er noch wartete. Neu
  sagt `AUDIO_IS_WAITING`, ob er noch wartet; mit einer entfernten Uhr ist
  beides FALSE.
* **Ein Tastenkürzel ohne Strg oder Alt gehört dem Bedienelement mit Fokus,
  wenn es die Taste selbst benutzt.** Mit `Leertaste` als Kürzel für
  Spielen/Stopp kippte ein Kästchen mit Fokus UND das Kürzel feuerte. Jetzt
  gehen Leertaste und Enter an Knöpfe, Kästchen, Listen und Tabellen,
  Pfeile, Pos1, Ende und Bild an Regler, Listen, Bäume und Reiterwerke,
  Buchstaben an Listen, Bäume, Tabellen und Klapplisten. F1 bis F12 lösen
  weiter immer aus.
* **`FORMAT$` mit einer Maske ohne Platzhalter ist ein Fehler.** Eine
  VB-Maske wie `"#,##0.00"` gab bisher die Maske zurück, der Wert ging still
  verloren. Für deutsche Zahlen gibt es jetzt `ZAHL$` (unten).

[All notes for 2026.29](../releases/release-2026.29.md)

## 2026.28

The notes for this version have no separate section for this.

[All notes for 2026.28](../releases/release-2026.28.md)

## 2026.27

The notes for this version have no separate section for this.

[All notes for 2026.27](../releases/release-2026.27.md)

## 2026.26

The notes for this version have no separate section for this.

[All notes for 2026.26](../releases/release-2026.26.md)

## 2026.25

The notes for this version have no separate section for this.

[All notes for 2026.25](../releases/release-2026.25.md)

## 2026.24

The notes for this version have no separate section for this.

[All notes for 2026.24](../releases/release-2026.24.md)

## 2026.23

The notes for this version have no separate section for this.

[All notes for 2026.23](../releases/release-2026.23.md)

## 2026.22

The notes for this version have no separate section for this.

[All notes for 2026.22](../releases/release-2026.22.md)

## 2026.21

The notes for this version have no separate section for this.

[All notes for 2026.21](../releases/release-2026.21.md)

## 2026.20

The notes for this version have no separate section for this.

[All notes for 2026.20](../releases/release-2026.20.md)

## 2026.19

The notes for this version have no separate section for this.

[All notes for 2026.19](../releases/release-2026.19.md)

## 2026.18

The notes for this version have no separate section for this.

[All notes for 2026.18](../releases/release-2026.18.md)

## 2026.17

The notes for this version have no separate section for this.

[All notes for 2026.17](../releases/release-2026.17.md)

## 2026.16

The notes for this version have no separate section for this.

[All notes for 2026.16](../releases/release-2026.16.md)

## 2026.15

The notes for this version have no separate section for this.

[All notes for 2026.15](../releases/release-2026.15.md)

## 2026.14

The notes for this version have no separate section for this.

[All notes for 2026.14](../releases/release-2026.14.md)

## 2026.13

The notes for this version have no separate section for this.

[All notes for 2026.13](../releases/release-2026.13.md)

## 2026.12

The notes for this version have no separate section for this.

[All notes for 2026.12](../releases/release-2026.12.md)

## 2026.11

The notes for this version have no separate section for this.

[All notes for 2026.11](../releases/release-2026.11.md)

## 2026.10

The notes for this version have no separate section for this.

[All notes for 2026.10](../releases/release-2026.10.md)

## 2026.9

The notes for this version have no separate section for this.

[All notes for 2026.9](../releases/release-2026.9.md)

## 2026.8

The notes for this version have no separate section for this.

[All notes for 2026.8](../releases/release-2026.8.md)
