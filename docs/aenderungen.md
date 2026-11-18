# Änderungsprotokoll

Was sich in jeder Fassung für **bestehende Programme** ändert, die neueste
zuerst -- gesammelt aus den Notizen der Fassungen. Was fest bleibt und was
sich ändern darf, sagt [Stabilität](stabilitaet.md).

*Erzeugt mit `dhrt doku aenderungen` aus `docs/releases/`; Änderungen gehören
in die Notizen, nicht hierher.*

## 2026.52

* **Meldungen in der Sprache des Systems.** Ohne `DHRT_LANG` schreibt dhrt
  seine Meldungen auf einem deutschen System deutsch, auf jedem anderen
  englisch (vorher immer deutsch). Was ein Programm in einer
  `CATCH`-Variable sieht, bleibt deutsch. Wer Ausgaben vergleicht, setzt
  `DHRT_LANG=de`.
* **IDE und Studio folgen ohne Einstellung ebenfalls dem System.** Auf einem
  englischen Windows starten sie englisch; in der IDE stellt Ansicht →
  „Sprache: Deutsch“ beim nächsten Start um, das Studio folgt der Einstellung
  der IDE.
* **`dhrt --export` nimmt von selbst die kleinste passende Laufzeit**
  (vorher nur mit `--schlank`, das weiter geht). `--voll` nimmt die ganze --
  etwa für ein Programm, das mit `IMPORT … OPTIONAL` und `MODULE_AVAILABLE`
  zur Laufzeit nachsieht, was der Bau kann.
* **Eine `.dhc` aus einer anderen Fassung wird abgewiesen** („Diese .dhc
  stammt aus dhrt …“), statt still anders zu laufen. Eine ohne Fassung
  (vor 2026.52 übersetzt) läuft wie bisher.
* **`DIRLIST(pfad, "")`** liefert alle Einträge (vorher keinen).
* **Die Signaturen nennen englische Parameternamen** (`color` statt
  `farbe`) -- im Hover, in der Vervollständigung und in den Handbüchern.
  Parameter mit Vorgabe stehen nicht mehr im Gerüst der Vervollständigung.

[Alle Notizen zu 2026.52](releases/release-2026.52.md)

## 2026.51

* **Neue Warnung: anders geschrieben als angelegt.** Wer `CONST CW` anlegt
  und später `cw` schreibt, bekommt beim Übersetzen eine Warnung (bei
  `dhrt run` auf stderr, im Editor die gelbe Linie). Das Programm läuft
  unverändert -- Groß/klein zählt weiter nicht. Strg+. oder
  `dhrt fmt --schreibweise` schreibt die Namen so wie angelegt.
* **Ein Label misst seine Breite am Text**, bis `GUI_SET_BOUNDS` oder
  `GUI_SET_WRAP` eine setzt; `GUI_GET_W` liefert die echte Breite.
* **Die Schrift eines Knopfs mit Farbverlauf** richtet sich nach der
  ungünstigsten Stelle des Verlaufs, `GUI_A11Y_CHECK$` misst dort.
* **Listen im gespiegelten Fenster** (`GUI_WINDOW_RTL`) spiegeln jetzt auch
  ihr Inneres.
* **Circuit Runner:** die Roboter ziehen halb so oft wie bisher (Taste T im
  Menü schaltet aufs Original); aufgezeichnete Lösungen passen nur noch im
  Originaltempo.

[Alle Notizen zu 2026.51](releases/release-2026.51.md)

## 2026.50

* **Farbige Knöpfe haben die lesbarere Schrift.** Auf einem Knopf mit
  eigener Grundfarbe (Primär, Erfolg, Warnung) wählt die gui die Schrift
  jetzt nach dem höheren Kontrast: auf dem Cyan des dunklen Themas, auf
  Grün und Orange steht dunkle Schrift statt weißer (vorher 2,1:1 bis
  3,4:1). Der Akzent von `modern_light` ist etwas dunkler (`#1D6DCE`).
* **Emoji kommen in Farbe.** Die Textfarbe gilt für sie nicht mehr, nur
  ihre Deckkraft.
* **Gespiegelte Fenster (`GUI_WINDOW_RTL`)** spiegeln jetzt auch Textbereiche,
  Tabellen und Bäume innen, und deren Rollbalken stehen links.
* **`WINDOW_WAIT_EVENTS`** wartet je Bild genau so lange, wie nichts
  ansteht: nach einer Eingabe läuft es 0,35 s ohne Warten nach, mit fester
  Bildzahl (`DHRT_FRAMES`) wartet es nie.
* **Schreibmarke und Hilfetext** rechnen in Sekunden statt in Bildern --
  bei 60 Bildern je Sekunde wie bisher.

[Alle Notizen zu 2026.50](releases/release-2026.50.md)

## 2026.49

* **Strg+Umschalt+P öffnet in jedem Programm mit Menü- oder Werkzeugleiste
  die Befehlspalette.** Hat das Programm selbst ein Menükürzel auf dieser
  Taste, geht seines vor; `GUI_PALETTE_KEY(FALSE)` schaltet die Taste ab.
  Solange die Palette offen ist, bekommt kein Textfeld darunter die Tasten.
* **Aus der IDE gestartete Programme** öffnen mit **Strg+Umschalt+I** den
  Inspektor (die IDE setzt `DHRT_INSPEKTOR=1`). Ein Programm, das selbst
  gestartet wird, bemerkt davon nichts.
* **Ein Klick in ein rechtsbündiges Textfeld** (`GUI_SET_ALIGN`) setzt die
  Schreibmarke jetzt an die Klickstelle; vorher landete sie am Ende. Der
  Platzhalter eines solchen Feldes läuft nicht mehr über den Rand.
* **Die Vervollständigung der IDE** schlägt hinter `AS` und `OF` nur noch
  Typen vor -- `DIM w AS GUI_WIN` nahm vorher den ersten passenden Befehl.

[Alle Notizen zu 2026.49](releases/release-2026.49.md)

## 2026.48

* **Während eines modalen Dialogs gelten seine eigenen Menüs.** Bisher war
  während eines modalen Dialogs jedes Menü gesperrt -- auch das Kontextmenü
  und die Menüleiste im Dialog selbst. Jetzt gehen die des Dialogs, die der
  Fenster dahinter bleiben gesperrt.
* **`GUI_DOCK_WHERE$` kennt einen neuen Ort:** `fenster` für eine
  abgerissene Leiste. Ein Programm, das die Antwort mit `SELECT CASE`
  auswertet, sieht diesen Wert nur, wenn eine Leiste abgerissen wird.
* **Aufnahmen (`AUTOMATION_RECORD`)** schreiben für echte Fenster zusätzliche
  Zeilen `n ...` ans Ende der Datei. raylibs Leser übergeht sie; alte
  Aufnahmen spielen unverändert.

[Alle Notizen zu 2026.48](releases/release-2026.48.md)

## 2026.47

* **Ein Klick geht an das oberste Widget.** Liegen zwei Widgets übereinander,
  bekam bisher das zuerst angelegte den Klick, gezeichnet wurde aber das
  zuletzt angelegte oben -- so ließ sich der OK-Knopf im Fenster „Über
  Drachenhauch“ nur mit Enter drücken. Jetzt fragen Klick, Zeichnen und
  `GUI_HIT_TEST` dieselbe Reihenfolge. Wer sich (unbeabsichtigt) darauf
  verlassen hat, dass ein darunterliegendes Widget den Klick bekommt, legt es
  zuletzt an.
* **Eine Beschriftung zeichnet ihren eigenen Grund und Rand:**
  `GUI_SET_COLOR(lbl, "bg", ..)` und `"border"` sowie `bg`/`border` aus der
  Stildatei galten für `GUI_LABEL` bisher nicht. Eine Stildatei, die Labels
  einen Grund gibt, zeigt ihn jetzt.
* **Der Rand eines großen Textbereichs im Fokus ist leiser:** über der
  Metrik `verlauf_hoehe` mischt sich der Akzent mit der Randfarbe. Ein am
  Widget gesetzter Akzent gilt unverändert.

[Alle Notizen zu 2026.47](releases/release-2026.47.md)

## 2026.46

* **Glutfaden, Funkenflug und die Kühlschrank-App liegen nicht mehr in diesem
  Repo und nicht mehr im Installer.** Sie haben eigene Repos:
  [Glutfaden](https://github.com/HasoSchno70/Glutfaden) (Kreuzstich-Editor),
  [Funkenflug](https://github.com/HasoSchno70/Funkenflug) (Schaltplan-Editor) und
  [Kuehlschrank-Drachenhauch](https://github.com/HasoSchno70/Kuehlschrank-Drachenhauch)
  (Studio-Projekt), jeweils mit ihrer ganzen Geschichte und unter derselben
  Lizenz. Sie laufen mit dieser Fassung wie bisher.
* **Der Form-Designer (`examples/197_form_designer.dh`) ist entfernt**; das
  Drachenhauch Studio ersetzt ihn. Eine `.dhform` im Projektbaum der IDE geht
  jetzt im Studio auf, der Installer löscht die alte Verknüpfung.
* **Spiele und Bücher liegen in eigenen Ordnern:** `spiele/` (Pyramid Pusher,
  Circuit Runner, Voidreaver, Restaurant, Kaminfeuer) und `buecher/`
  (`referenz`, `einstieg`, `galaga`, `tippspiel` statt `buch-*`). Wer sie aus
  dem Quelltext startet, nimmt den neuen Pfad, etwa
  `dhrt run spiele/pyramid_pusher/pyramid_pusher.dh`. Die Vokabellisten des
  Einstiegsbuchs bleiben unter `buch-einstieg/vokabellisten/`, damit die
  Adresse in schon verteilten Büchern gültig bleibt.
* **Physik (`PHYS2D_*`, `PHYS3D_*`) rechnet mit Rapier 0.36 statt 0.22.**
  Bahnen können in den letzten Stellen anders ausfallen. Ein schlafend
  angelegter Stapel bleibt weiter ruhig bis zum ersten Anstoß, setzt sich aber
  im allerersten Schritt um gut einen Millimeter, wo er vorher genau
  stehen blieb.

[Alle Notizen zu 2026.46](releases/release-2026.46.md)

## 2026.45

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

[Alle Notizen zu 2026.45](releases/release-2026.45.md)

## 2026.44

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

[Alle Notizen zu 2026.44](releases/release-2026.44.md)

## 2026.43

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

[Alle Notizen zu 2026.43](releases/release-2026.43.md)

## 2026.42

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

[Alle Notizen zu 2026.42](releases/release-2026.42.md)

## 2026.41

* **Ein Drehfeld (`GUI_SPINNER`) nimmt Tastatur-Eingaben.** Bisher kannte es
  nur die Pfeiltasten und seine zwei Knöpfe. Mit dem Fokus ist der Wert jetzt
  markiert, und das erste Getippte ersetzt ihn. Wer bisher Buchstaben- oder
  Zifferntasten abfragte, während ein Drehfeld den Fokus hatte, bekommt sie
  dort nicht mehr – sie gehören dem Feld, wie bei einem Textfeld.

[Alle Notizen zu 2026.41](releases/release-2026.41.md)

## 2026.40

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

[Alle Notizen zu 2026.40](releases/release-2026.40.md)

## 2026.39

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

[Alle Notizen zu 2026.39](releases/release-2026.39.md)

## 2026.38

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

[Alle Notizen zu 2026.38](releases/release-2026.38.md)

## 2026.37

Die Notizen dieser Fassung haben dafür keinen eigenen Abschnitt.

[Alle Notizen zu 2026.37](releases/release-2026.37.md)

## 2026.36

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

[Alle Notizen zu 2026.36](releases/release-2026.36.md)

## 2026.35

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

[Alle Notizen zu 2026.35](releases/release-2026.35.md)

## 2026.34

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

[Alle Notizen zu 2026.34](releases/release-2026.34.md)

## 2026.33

* **Ein zweites `DIM` desselben Namens mit anderem Typ ist ein Fehler.**
  Gross- und Kleinschreibung zählen nicht, `pd` und `pD` sind also dieselbe
  Variable. Bisher gab es dafür nur eine Warnung, und abgebrochen wurde erst
  beim Laufen, an einer ganz anderen Stelle („Feld 'x' existiert nicht in
  b“). Das gilt im Hauptprogramm, in SUB/FUNCTION und in Methoden, ebenso für
  `CATCH e` mit einem späteren `DIM e AS T`. Ein zweites `DIM` mit
  **gleichem** Typ bleibt eine Warnung. Im Repository löst kein Programm den
  Fehler aus (geprüft mit `dhrt --check` über alle `.dh`-Dateien).

[Alle Notizen zu 2026.33](releases/release-2026.33.md)

## 2026.32

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

[Alle Notizen zu 2026.32](releases/release-2026.32.md)

## 2026.31

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

[Alle Notizen zu 2026.31](releases/release-2026.31.md)

## 2026.30

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

[Alle Notizen zu 2026.30](releases/release-2026.30.md)

## 2026.29

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

[Alle Notizen zu 2026.29](releases/release-2026.29.md)

## 2026.28

Die Notizen dieser Fassung haben dafür keinen eigenen Abschnitt.

[Alle Notizen zu 2026.28](releases/release-2026.28.md)

## 2026.27

Die Notizen dieser Fassung haben dafür keinen eigenen Abschnitt.

[Alle Notizen zu 2026.27](releases/release-2026.27.md)

## 2026.26

Die Notizen dieser Fassung haben dafür keinen eigenen Abschnitt.

[Alle Notizen zu 2026.26](releases/release-2026.26.md)

## 2026.25

Die Notizen dieser Fassung haben dafür keinen eigenen Abschnitt.

[Alle Notizen zu 2026.25](releases/release-2026.25.md)

## 2026.24

Die Notizen dieser Fassung haben dafür keinen eigenen Abschnitt.

[Alle Notizen zu 2026.24](releases/release-2026.24.md)

## 2026.23

Die Notizen dieser Fassung haben dafür keinen eigenen Abschnitt.

[Alle Notizen zu 2026.23](releases/release-2026.23.md)

## 2026.22

Die Notizen dieser Fassung haben dafür keinen eigenen Abschnitt.

[Alle Notizen zu 2026.22](releases/release-2026.22.md)

## 2026.21

Die Notizen dieser Fassung haben dafür keinen eigenen Abschnitt.

[Alle Notizen zu 2026.21](releases/release-2026.21.md)

## 2026.20

Die Notizen dieser Fassung haben dafür keinen eigenen Abschnitt.

[Alle Notizen zu 2026.20](releases/release-2026.20.md)

## 2026.19

Die Notizen dieser Fassung haben dafür keinen eigenen Abschnitt.

[Alle Notizen zu 2026.19](releases/release-2026.19.md)

## 2026.18

Die Notizen dieser Fassung haben dafür keinen eigenen Abschnitt.

[Alle Notizen zu 2026.18](releases/release-2026.18.md)

## 2026.17

Die Notizen dieser Fassung haben dafür keinen eigenen Abschnitt.

[Alle Notizen zu 2026.17](releases/release-2026.17.md)

## 2026.16

Die Notizen dieser Fassung haben dafür keinen eigenen Abschnitt.

[Alle Notizen zu 2026.16](releases/release-2026.16.md)

## 2026.15

Die Notizen dieser Fassung haben dafür keinen eigenen Abschnitt.

[Alle Notizen zu 2026.15](releases/release-2026.15.md)

## 2026.14

Die Notizen dieser Fassung haben dafür keinen eigenen Abschnitt.

[Alle Notizen zu 2026.14](releases/release-2026.14.md)

## 2026.13

Die Notizen dieser Fassung haben dafür keinen eigenen Abschnitt.

[Alle Notizen zu 2026.13](releases/release-2026.13.md)

## 2026.12

Die Notizen dieser Fassung haben dafür keinen eigenen Abschnitt.

[Alle Notizen zu 2026.12](releases/release-2026.12.md)

## 2026.11

Die Notizen dieser Fassung haben dafür keinen eigenen Abschnitt.

[Alle Notizen zu 2026.11](releases/release-2026.11.md)

## 2026.10

Die Notizen dieser Fassung haben dafür keinen eigenen Abschnitt.

[Alle Notizen zu 2026.10](releases/release-2026.10.md)

## 2026.9

Die Notizen dieser Fassung haben dafür keinen eigenen Abschnitt.

[Alle Notizen zu 2026.9](releases/release-2026.9.md)

## 2026.8

Die Notizen dieser Fassung haben dafür keinen eigenen Abschnitt.

[Alle Notizen zu 2026.8](releases/release-2026.8.md)
