# Entwurf: Was der gui gegen GTK 3 noch fehlt

Stand 2026-10-31. Die eigene gui (`rust/drachenhauch_runtime/src/gui.rs`,
rund 22 800 Zeilen, dazu `gui_dock.rs`) hat nach den Ausbaurunden von
2026-09 fast jede Widget-Art, die eine Anwendung braucht: Tabellen mit
Sortieren, Filter, Zellmodus und Baum, Listen mit Gruppen und Umbenennen,
Dateibaum, Reiterwerk, Akkordeon, Assistent, Dock, gesetzter Text, Code-Feld
mit Faltung und mehreren Marken, Formularprüfung, Datenbindung,
Barrierefreiheit über AccessKit, Drucken. Dieses Dokument vergleicht sie mit
GTK 3 und sagt, was noch fehlt, in welcher Reihenfolge und wie es sich
bauen ließe. **Paket 1 ist gebaut** (2026-10-31, `gui_ueber.rs`, Handbuch
`docs/module-gui.md` Abschnitte „Popover, Toast und Infoleiste“ und
„Suchfeld, Sinnbilder, Vorschläge“), **Paket 4 in der ersten Stufe** ebenso
(2026-10-31, `gui_stil.rs`, Abschnitt „Stildatei“), **Paket 2** ebenso
(2026-10-31, `gui_bausteine.rs`, Abschnitt „Bausteine“); Paket 3 und die
weiteren Stufen von Paket 4 noch nicht.

## Gemessen statt vermutet: große Tabellen sind keine Lücke

Die erste Vermutung war, eine Tabelle ohne Modell (GTKs `GtkTreeModel`)
werde bei vielen Zeilen langsam. Gemessen mit 100 000 Zeilen in einer
Tabelle und 100 000 in einer Liste, beide im selben Fenster:

| Schritt | Zeit |
|---|---|
| Tabelle füllen (100 000 × 3 Zellen) | 42 ms |
| Liste füllen (100 000 Einträge) | 11 ms |
| ein gewöhnliches Bild | 16 ms (Bildrate 60) |
| das Bild, in dem sortiert wird | 38 ms |
| das Bild, in dem gefiltert wird (11 333 bleiben) | 17 ms |

Gezeichnet werden nur die sichtbaren Zeilen, Sortieren und Filtern bauen nur
die Ansicht (`view`). Ein Modell mit Rückrufen je Zeile wäre damit Aufwand
ohne Ertrag.

## Die Lücken

### Paket 1: Überlagerungen und Rückmeldung (gebaut)

Anders als hier zuerst geplant liegt die Infoleiste über dem Inhalt, statt
ihn zu verschieben (ein verschobener Inhalt liefe unten aus dem Fenster),
und `GUI_INFOBAR_BUTTON_RECT` kam dazu, damit sich die Knöpfe prüfen lassen.

| GTK 3 | in der gui | Vorschlag |
|---|---|---|
| `GtkPopover`: ein Kasten mit Pfeil, an ein Widget gehängt, schließt bei einem Klick daneben | nur Fenster (`GUI_WINDOW`), Dialoge, Kontextmenüs | `GUI_POPOVER` (widget, b, h [, seite$]) → GUI_WINDOW ohne Rahmen, an das Widget gehängt; ein Klick außerhalb oder ESC schließt; der Pfeil gehört zur Chrome. Inhalt wie in jedem Fenster. Gezeichnet in der oberen Schicht (`draw_top`) wie Kontextmenü und Tooltip, sonst läge er unter einer Zeichenfläche. |
| `GtkInfoBar`, Toast (`GtkRevealer` + Label) | Statusleiste, `GUI_DIALOG`, `NOTIFY` (System) | `GUI_TOAST` (win, text$ [, art$ [, ms]]): Meldung unten im Fenster, blendet ein und nach `ms` aus (Übergang wie Akkordeon), Art info/erfolg/warnung/fehler mit den Farben der Knopfarten; mehrere stapeln sich. Dazu `GUI_INFOBAR` (win, text$, knoepfe$) oben im Inhalt, bleibt bis zur Antwort (wie `GUI_DIALOG`, aber nicht modal). |
| `GtkSearchEntry`: Lupe links, ✕ rechts, meldet erst nach einer Pause | ein Textfeld, das Programm baut den Rest | `GUI_TEXTINPUT_SET(tf, "suche", 1)`: Lupe als Sinnbild, ✕ bei Text (Klick leert, ESC leert), `GUI_SEARCH_CHANGED` (tf) erst 150 ms nach dem letzten Anschlag. Dazu allgemein `GUI_TEXTINPUT_ICON` (tf, links$, rechts$) und `GUI_ICON_CLICKED` (tf) (Auge im Passwortfeld). Gezeichnet über `zeichne_textfeld`, das schon Platz für den Pfeil der Klappliste kennt (`rechts`). |
| `GtkEntryCompletion`: Vorschläge unter einem Textfeld | die bearbeitbare Klappliste kommt nahe; ein Textfeld schlägt nichts vor | `GUI_TEXTINPUT_COMPLETION` (tf, eintraege): dieselbe Liste wie das Popup der Klappliste (`dropdown_popup_rect`, `dd_mark`), nur unter einem Textfeld, Teiltext statt Anfang; Pfeile wählen, Enter übernimmt, ESC verwirft. |

**Warum zuerst:** jede Anwendung braucht Rückmeldung ohne Dialog, und die
IDE baut sich Popover und Suchfelder heute aus Fenstern und Textfeldern
selbst. Alles in diesem Paket setzt auf Vorhandenes auf: obere Schicht,
Übergänge, Popup der Klappliste, Knopfarten.

### Paket 2: Bausteine, die eine Anwendung erwartet (gebaut)

Anders als hier geplant heißt der Kreisel `GUI_BUSY` (`GUI_SPINNER` ist
schon das Zahlenfeld) und ist ein Fortschritt, keine neue Widget-Art -- so
braucht der Form-Designer keinen neuen Eintrag. Eine markierbare
Beschriftung wird nicht gekürzt. Dazu kamen `GUI_DATE_UNMARK`,
`GUI_DATE_MARKED`, `GUI_LABEL_SELECTION$` und der Ring für einen Kreisel
mit Wert.

| GTK 3 | in der gui | Vorschlag |
|---|---|---|
| `GtkFlowBox`: Kacheln, die umbrechen, mit Auswahl | Kacheln (`GUI_CARD`) müssen von Hand verteilt werden; die IDE rechnet ihre Kachelreihe selbst | Layout-Art `fluss` in `GUI_LAYOUT` (bricht um, wenn die Zeile voll ist; `layout_anwenden` kennt zeile/spalte/raster) und `GUI_LAYOUT_SET(l, "auswahl", 1)` mit `GUI_LAYOUT_SELECTED`. Rollen über ein rollendes Panel. |
| Mnemonics: `_Datei` → Alt+D, unterstrichener Buchstabe | Menüs per F10/Alt, aber Knöpfe und Beschriftungen ohne Alt-Taste | `&` im Text (`"&Speichern"`): unterstreicht den Buchstaben, Alt+S löst den Knopf aus bzw. setzt den Fokus auf das Feld hinter der Beschriftung (dieselbe Zuordnung wie `a11y_beschriftung`). Geprüft in `kuerzel_pruefen` nach den Menükürzeln. `&&` = ein `&`. Folge: ein bestehender Knopf mit `&` im Text änderte sich -- nur mit `GUI_MNEMONICS` (TRUE) an. |
| `GtkLabel`: `ellipsize`, `selectable` | Umbruch (`GUI_SET_WRAP`); abschneiden mit „…" kennen nur Listen und Statusleiste | `GUI_LABEL_SET` (lbl, "kuerzen", "ende"\|"mitte"\|"anfang") (Mitte für Pfade) über `text_kuerzen`; `"auswaehlbar"` mit derselben Auswahl wie der gesetzte Text (Ziehen, Doppelklick, Strg+C). |
| `GtkScale` mit Marken (`add_mark`) und Wertanzeige | Regler ohne Striche und ohne Zahl | `GUI_SLIDER_SET` (s, "marken", n), `GUI_SLIDER_MARK` (s, wert, text$), `"wert_zeigen"`, `"einrasten"` (an den Marken). Geometrie über `drag_slider`. |
| `GtkSpinner`: Kreis, der sich dreht | Fortschrittsbalken „unbestimmt" | `GUI_SPINNER(win, x, y, d)` + `GUI_SET_VISIBLE`; gezeichnet aus `g.get_time()` wie der unbestimmte Balken. |
| `GtkCalendar`: Tage markieren | Datumswähler ohne Markierungen | `GUI_DATE_MARK` (dp, datum$ [, farbe]) / `GUI_DATE_CLEAR_MARKS` -- ein Punkt unter dem Tag (`dp_geom`). |
| `GtkFontChooser` | keine Schriftwahl | `GUI_FONT_DIALOG` (...) als `GUI_DIALOG` mit Liste der Systemschriften (aus den Ordnern, die `ausweich_datei_fuer` schon kennt), Größe, Schnitt und Vorschau; liefert Pfad und Größe für `LOADFONT`. |
| `GtkSizeGroup`: gleiche Breite über Behälter hinweg | Layouts messen nur ihre eigenen Kinder | `GUI_SIZE_GROUP` (name$, wdg, ...): im `layout_pass` bekommt jedes Mitglied das größte natürliche Maß der Gruppe (Beschriftungen vor Feldern in mehreren Zeilen). |

### Paket 3: Text, wie GTK ihn kann (Pango)

| GTK 3 | in der gui | Vorschlag |
|---|---|---|
| Textformung (Ligaturen, Arabisch, Indische Schriften) | Glyphe für Glyphe aus raylib; `rustybuzz` ist schon im Bau, aber nur fürs PDF | Formen über `rustybuzz` und die Glyphenatlanten von raylib zeichnen; nur für Läufe, die es brauchen (Schriftblock-Erkennung), damit das Messen der gewöhnlichen Fälle nicht langsamer wird. Schreibmarke, Auswahl und Klick müssen mit den geformten Glyphen rechnen -- der größte Teil der Arbeit. |
| Rechts-nach-links, gemischte Richtung | nein | Crate `unicode-bidi` für die Reihenfolge der Läufe, danach Formen wie oben. Betrifft Textfeld, Textbereich, Liste, Tabelle, Beschriftung. |
| Rechtschreibung (gspell) | Wellenlinien gibt es (`GUI_TEXTAREA_SQUIGGLES`), ein Wörterbuch nicht | Hunspell-Wörterbücher (`.dic`/`.aff`) über eine reine Rust-Crate lesen, `SPELL_CHECK` (wort$)/`SPELL_SUGGEST`; das Feld unterstreicht über die vorhandenen Wellenlinien, Vorschläge im Kontextmenü. Das Plugin `schreibweise` der IDE wäre der erste Nutzer. |

**Warum zuletzt:** das ist der teuerste Teil, und solange niemand Arabisch
oder Hebräisch schreibt, merkt es keiner. Rechtschreibung lässt sich davon
getrennt vorziehen.

### Paket 4: Styling wie GTK-CSS (erste Stufe gebaut)

Gebaut ist die erste Stufe: Stildatei mit Art, Klasse, Name und Zuständen,
`bg`/`fg`/`border`/`accent`, `radius`, `groesse`, `stil`. Anders als hier
geplant gibt es kein `padding`, `border-width`, `schatten` und `verlauf` je
Widget (das braucht den Umbau der Zeichenwege Art für Art) und keinen
`uebergang` je Regel; die Farben gehen über die vorhandenen Stellen
(`wcol` und die eigenen Farben), das deckt alle Arten auf einmal. Neu dazu:
`GUI_GET_COLOR`, damit sich die Auflösung ohne Bildlagen prüfen lässt.

Was die gui **hat**:

| Befehl | was er tut |
|---|---|
| `GUI_THEME_PRESET` | acht fertige Themen (dunkel, hell, retro, kontrast, modern, glas …) |
| `GUI_THEME_SET/GET` | einzelne Farben des Themas |
| `GUI_METRIC_SET/GET` | Maße und Plastik: Abstände, `corner_radius`, `gradient`, `gloss`, `bevel`, `uebergang`, `verlauf_hoehe` |
| `GUI_SET_COLOR(wdg, rolle$, farbe)` | je Widget bg/fg/border/accent/hover/pressed |
| `GUI_STYLE_SET` + `GUI_APPLY_STYLE` | benannte Stile (Farben, Schrift, Größe), auf ein Widget **kopiert** |
| `GUI_BUTTON_VARIANT` | Knopfarten primär/erfolg/warnung/gefahr/umriss/flach/link |
| `GUI_SET_FONT/_SIZE/_STYLE`, `GUI_FONT` | Schrift je Widget und für die ganze gui |

Was GTK-CSS darüber hinaus kann:

1. **Eine Stildatei mit Selektoren.** `button.gefahr:hover { … }`,
   `#speichern { … }`, `entry:focus { … }`, `list row:selected { … }`. Die
   gui setzt alles per Befehl je Widget; wer ein Aussehen ändern will, ändert
   Programmcode.
2. **Zustände als Selektor**: `:hover`, `:active`, `:focus`, `:disabled`,
   `:checked`, `:selected`. Die gui kennt nur hover und pressed als
   Farbrollen, und nur beim Knopf.
3. **Klassen, die lebendig bleiben**: `gtk_style_context_add_class` wirkt,
   solange die Klasse dran ist, und eine Änderung der Klasse trifft alle
   Träger. `GUI_APPLY_STYLE` kopiert einmal -- spätere Änderungen am Stil
   erreichen das Widget nicht.
4. **Kastenmodell je Widget**: `padding`, `margin`, `border-width` je Seite,
   `border-radius` je Ecke, `box-shadow`, `background-image` (Verläufe,
   Bilder), `outline`. In der gui sind das Metriken für alle Widgets
   zugleich.
5. **Übergänge je Eigenschaft** (`transition: background-color 200ms`). Die
   gui hat eine Dauer für alle (`uebergang`).
6. **Schrift**: `letter-spacing`, `font-feature-settings` -- weniger wichtig.
7. **Thema als Datei** laden und tauschen, ohne das Programm anzufassen.

**Vorschlag: `GUI_STYLESHEET` (text$) / `GUI_STYLESHEET_LOAD` (pfad$)** mit
einer kleinen, CSS-ähnlichen Sprache -- nicht CSS selbst, sondern die
Teilmenge, die zur gui passt:

```text
/* thema.dhstil */
button            { radius: 6; padding: 6 12; }
button.gefahr     { bg: #C0392B; fg: #FFFFFF; }
button.gefahr:hover { bg: #E74C3C; }
textinput:focus   { border: #4FA3E0; border-width: 2; }
list:selected     { bg: akzent; }
#speichern        { schrift: sans-fett; groesse: 16; }
.karte            { schatten: 0 2 8 #00000040; radius: 10; }
```

- **Selektor** = Art (die Namen aus `Kind`, dieselben wie in der `.dhform`),
  `.klasse`, `#name` (dafür braucht es neu `GUI_SET_NAME` -- heute liest die
  Laufzeit den `name` einer `.dhform` gar nicht, nur der Form-Designer und
  die Plugin-Bibliothek führen ihn),
  Zustand `:hover :active :focus :disabled :checked :selected`. Keine
  Nachfahren-Selektoren in der ersten Stufe -- die brauchen einen Baum, und
  die gui kennt Behälter nur über Layout/Panel/Reiterwerk.
- **Eigenschaften**: die Farbrollen von heute (`bg fg border accent`), dazu
  `radius` (eine Zahl oder vier), `padding`, `border-width`, `schatten`,
  `verlauf` (an/aus und Stärke wie `gradient`), `schrift`, `groesse`,
  `stil` (fett/kursiv …), `uebergang`. Farben als `#RGB`, `#RRGGBB`,
  `#AARRGGBB` oder als Name aus dem Thema (`akzent`, `win_bg` …), damit eine
  Stildatei mit beiden Themen geht.
- **Klassen**: `GUI_ADD_CLASS` (wdg, klasse$), `GUI_REMOVE_CLASS`,
  `GUI_HAS_CLASS`. Die Knopfarten werden zu Klassen (`GUI_BUTTON_VARIANT`
  bleibt und setzt die Klasse).
- **Vorrang** wie in CSS: Art < Klasse < Name, Zustand zählt als Klasse,
  Späteres gewinnt bei Gleichstand; ein `GUI_SET_COLOR` am Widget gewinnt
  vor allem (es ist der „style="-Eintrag).
- **Auflösen**: je Widget einmal, wenn sich Klassen, Name oder die
  Stildatei ändern; das Ergebnis liegt am Widget (`aufgeloest`), damit das
  Zeichnen nicht je Bild Selektoren vergleicht. Nur der Zustand wird beim
  Zeichnen gewählt (vier Plätze: normal/hover/aktiv/fokus, dazu gesperrt).
- **Zeichnen**: heute fragen die Zeichenwege an etwa 120 Stellen das Thema
  (`self.th(...)`) und an etwa 60 die Metriken (`self.m(...)`). Der Umbau
  ersetzt diese Fragen durch eine je Widget (`self.stil(w, "bg")`), die erst
  das Aufgelöste und dann das Thema nimmt. Das ist der eigentliche Aufwand;
  er lässt sich Art für Art machen (Knopf, Textfeld, Liste zuerst).
- **Datei**: `.dhstil`, im Form-Designer ladbar, damit die Vorschau aussieht
  wie das Programm. Ein Fehler in der Datei (unbekannte Eigenschaft,
  krumme Farbe) ist eine Meldung mit Zeile -- kein stilles Übergehen, sonst
  sucht man den Fehler im Programm.
- **Nicht** in der ersten Stufe: Nachfahren-Selektoren, `:nth-child`,
  Animationen über Schlüsselbilder, Bilder als Rahmen (`border-image`).

**Prüfung**: Bildproben je Eigenschaft (ein Knopf mit und ohne Klasse, im
Zustand hover über eine Aufnahme), Vorrang als Paare (Name schlägt Klasse,
`GUI_SET_COLOR` schlägt beides), eine Stildatei mit Fehler, die `.dhform`
mit Klassen im Rundweg, und die bestehenden Bildfälle unverändert grün --
ohne Stildatei darf sich kein Punkt ändern.

## Was bleibt und bewusst nicht kommt

- **Mehrere echte Betriebssystem-Fenster** in einem Prozess: raylib hat
  eines je Prozess; `WINDOW_OPEN` (zweiter Prozess) und das Dock decken es
  ab. Siehe `docs/entwurf-native-fenster.md`.
- **Touch-Gesten an Widgets** (Wischen in Listen, Kneifen in der
  Zeichenfläche): die Rohdaten gibt es (`GESTURE$`), die gui wertet sie
  nicht aus. Erst, wenn jemand die gui auf einem Tablet benutzt.
- **Ziehen aus der Anwendung hinaus** (Datei auf den Desktop) und **Bilder
  über die Zwischenablage**: raylib/GLFW können beides nicht; wäre je System
  eigener Code.

## Reihenfolge

1. Paket 1 (Popover, Toast/InfoBar, Suchfeld, Vorschläge unter dem
   Textfeld) -- klein, sofort nützlich, die IDE ist der erste Nutzer.
2. Paket 4 in der ersten Stufe (Stildatei, Klassen, Zustände, Knopf und
   Textfeld und Liste umgebaut) -- danach die übrigen Arten nacheinander.
3. Paket 2 (Fluss-Layout, Mnemonics, Beschriftung kürzen/auswählen,
   Reglermarken, Kreisel, Kalendermarken, Schriftwahl, Größengruppen).
4. Paket 3: Rechtschreibung vorziehbar; Formen und RTL zuletzt.
