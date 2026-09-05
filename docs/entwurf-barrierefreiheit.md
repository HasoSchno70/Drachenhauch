# Entwurf: Barrierefreiheit

*Untersuchung, keine Umsetzung.* Der dritte Architekturpunkt der Lückenliste
nach dem sechsten Piloten (nach den [Fenstern](entwurf-native-fenster.md) und
dem [Drucken](entwurf-drucken.md)): die Rechnungsverwaltung läuft, sieht gut
aus, lässt sich mit der Tastatur bedienen — und ein blinder Nutzer kann sie
nicht öffnen. Nicht „schlecht bedienen": gar nicht. Dieses Papier misst den
Stand, prüft die fertigen Bausteine, die sich einbinden ließen (so, wie raylib,
Kira und Rapier eingebunden sind), entwirft vier Wege und empfiehlt einen. Die
Entscheidung fällt jemand anders.

Alle Angaben sind geprüft, nicht angenommen — Stand 05.09.2026, diese
Maschine (Windows 11) und docs.rs. Was nur unter macOS oder Linux zu prüfen
wäre, steht als solches da.

## 1. Was „Barrierefreiheit" hier heißt

Das Wort deckt vier Gruppen, und sie brauchen Verschiedenes:

| Wer | Braucht | Wer liefert es |
|---|---|---|
| **Blind** | einen **Bildschirmleser** (Windows: NVDA, JAWS, Sprachausgabe; macOS: VoiceOver; Linux: Orca), der Bedienelemente vorliest und Braille ausgibt | das **Toolkit** — es muss dem System sagen, was auf dem Schirm ist |
| **Sehbehindert** | Vergrößerung, die dem Fokus folgt; Kontrast; Schriftgröße; kein Rot-Grün als einzige Information | Toolkit (Themen, Maßstab, Fokus melden) und Programm (Farbwahl) |
| **Motorisch eingeschränkt** | alles ohne Maus; Schaltersteuerung; **Sprachsteuerung** („Klick Speichern") | Toolkit — die Sprachsteuerung sucht den Knopf über dieselbe Schnittstelle wie der Bildschirmleser |
| **Gehörlos / kognitiv** | Untertitel statt Ton, Bewegung abschaltbar, klare Sprache | das **Programm**; das Toolkit kann höchstens einen Schalter anbieten |

Für ein Toolkit reduziert sich das auf zwei Fragen: **was tut es von selbst**,
und **was gibt es dem Programmierer in die Hand**. Der Schlüssel zu drei der
vier Zeilen ist derselbe: die Schnittstelle, über die ein fremdes Programm
fragt, was in unserem Fenster steht — unter Windows *UI Automation* (UIA),
unter macOS *NSAccessibility*, unter Linux *AT-SPI*. Bildschirmleser,
Vergrößerung, Sprachsteuerung und Schaltersteuerung lesen alle daraus.

## 2. Was heute geht — gemessen

**Der Kern zuerst.** Ein laufendes `dhrt` mit dem gui-Beispiel 45, befragt
über UIA (PowerShell, `System.Windows.Automation`):

```text
Fenster: [GUI-Demo (Retained-Mode)]  Typ ControlType.Window  Klasse GLFW30
Nachkommen im UIA-Baum: 0
```

**Null.** Für einen Bildschirmleser ist jedes Drachenhauch-Fenster ein
Fenster mit Titel und sonst nichts — kein Knopf, kein Textfeld, keine
Beschriftung. Die Windows-Sprachausgabe liest den Titel vor und schweigt.
Eine Sprachsteuerung findet nichts, das sie anklicken könnte. Die
Bildschirmlupe kann dem Fokus nicht folgen, weil sie ihn nicht sieht. Das
ist keine Lücke im Detail, sondern die Abwesenheit der Schnittstelle: raylib
malt Pixel in ein GLFW-Fenster, und GLFW meldet dem System keine
Bedienelemente. Der Web-Bau hat dasselbe Loch (eine Leinwand ohne DOM).

Was das Toolkit sonst schon kann, steht im Vergleich dazu gut da:

| Bereich | Stand | Bemerkung |
|---|---|---|
| Tastatur | **Tab/Umschalt+Tab durch alle bedienbaren Widgets**, Leertaste/Enter löst aus, Pfeile verstellen, ESC schließt Klapplisten und Dialoge, Enter/ESC drücken Standard- und Abbrechen-Knopf | seit 2026-08-30; eine Quelle `Kind::fokussierbar` |
| Fokusring | 2 px in Akzentfarbe außerhalb des Widgets | ohne sichtbaren Fokus wäre die Tastatur wertlos |
| Menüleiste | **nur mit der Maus oder über ein Kürzel** — kein Alt, kein F10 | gemessen: `menu_input` liest keine Taste; wer kein Kürzel kennt, kommt ohne Maus nicht ins Menü |
| Tab-Reihenfolge | = Reihenfolge des Anlegens, nicht setzbar | mit Layout-Behältern stimmt sie meist; bei nachträglich angelegten Widgets nicht |
| Maßstab | `GUI_SCALE(0.5..4.0)` — alle Maße und Schriften | für 4K gebaut, hilft genauso bei schwachen Augen |
| Themen | `GUI_THEME_PRESET("contrast")`: Weiß auf Schwarz, gelbe Rahmen — **21:1** | vorhanden, aber nirgends als Barrierefreiheits-Thema genannt |
| Kontrast der Vorgabe (`dark`) | Text 13:1, gedämpfter Text 4,57:1, Akzent 7,8:1 | alles über WCAG AA (4,5:1) |
| Kontrast `light` | Text 11:1, **gedämpfter Text 2,76:1**, Akzent 3,8:1 | gedämpft liegt unter AA; der Akzent knapp unter 4,5, aber über den 3:1 für Bedienelemente |
| Fehler im Formular | roter Doppelrahmen **und** Meldung im Tooltip **und** `GUI_ERROR_LABEL` | nicht nur Farbe — richtig gemacht |
| Tooltips | beim Überfahren mit der Maus | bei Tastaturfokus erscheint keiner |
| Sprachausgabe aus dem Programm | — | kein Befehl; `SHELL_START` mit PowerShell ginge, kostet gemessen **194 ms** je Aufruf |
| Ansage („3 Zeilen gelöscht") | — | eine Statuszeile ist Pixel |

Was ein Programm heute also anbieten kann: Tastaturbedienung, den
Kontrast-Vorsatz, den Maßstab. Was es nicht kann, auch mit Mühe nicht: einem
Bildschirmleser irgendetwas sagen.

## 3. Fertige Bausteine, geprüft

### AccessKit — die Schnittstelle für alle drei Systeme

[AccessKit](https://accesskit.dev/) ist genau das, was fehlt: eine
Rust-Bibliothek, die einem Toolkit ohne eigene Systemanbindung die
Barrierefreiheits-Schnittstellen der drei Systeme gibt. Das Toolkit **schiebt
einen Baum** (Knoten mit Nummer, Rolle, Beschriftung, Wert, Rechteck,
Zuständen und erlaubten Aktionen), AccessKit übersetzt ihn nach UIA,
NSAccessibility oder AT-SPI, und **Aktionen kommen zurück** (Fokus setzen,
Klick, Wert setzen, Textauswahl). Geprüft auf docs.rs, alle am 29.08.2026
erschienen, MIT oder Apache-2.0:

| Crate | Fassung | Was sie braucht |
|---|---|---|
| `accesskit` | 0.25.0 | nichts — Datenmodell (`Node`, `Role`, `TreeUpdate`, `Action`) |
| `accesskit_windows` | 0.35.0 | ein **HWND**; `SubclassingAdapter` hängt sich per Win32-Subclassing an ein fremdes Fenster und beantwortet `WM_GETOBJECT` — gebaut für Fenster, deren Nachrichtenschleife man nicht besitzt |
| `accesskit_macos` | 0.27.0 | das **NSWindow**; `SubclassingAdapter` per dynamischem Objective-C-Subclassing, dazu `add_focus_forwarder_to_window_class` für Bibliotheken, die den Fokus aufs Fenster statt auf die View legen (SDL, und GLFW ebenso) |
| `accesskit_unix` | 0.23.0 | D-Bus über `zbus` (reines Rust) und einen Async-Läufer (`async-io` oder `tokio`, Feature); Fensterlage und -fokus meldet man selbst (`set_root_window_bounds`, `update_window_focus_state`) |

**Dass ein fremdes Fenster geht, ist nicht meine Vermutung, sondern das
offizielle Beispiel:** die C-Bindung liefert `examples/sdl/hello_world.c`, das
ein SDL-Fenster nimmt — unter Windows das HWND aus `SDL_GetWindowWMInfo`,
unter macOS das Cocoa-Fenster, unter Linux den Unix-Adapter — und darüber
zwei Knöpfe samt Ansage-Knoten anbietet. Aktionen kommen auf einem fremden
Faden an und werden als eigenes Ereignis in die SDL-Schleife gestellt; unser
Gegenstück wäre eine Warteschlange, die `GUI_UPDATE` leert. SDL und GLFW sind
für diese Frage dasselbe: ein Fenster, das jemand anders erzeugt hat.

**Das Fensterhandle haben wir:** raylib-rs 6.0 hat `get_window_handle()`
(`src/core/window.rs:942`, ruft `GetWindowHandle`) — unter Windows das HWND,
unter macOS das NSWindow. Die Fensterklasse heißt `GLFW30` (siehe Messung
oben), unter macOS `GLFWWindow`.

**Kosten:** Der Baum wird nur gebaut, wenn ein Hilfsprogramm danach fragt
(`update_if_active`, Aktivierung über den Adapter) — ein Spiel ohne
Bildschirmleser zahlt nichts. `accesskit_windows` zieht `windows` 0.62;
dhrt hat 0.61 (fürs Drucken) — entweder zwei Fassungen im Bau (geht,
kostet Übersetzungszeit) oder ein Sprung auf 0.62. Mindest-Rust 1.85, hier
ist 1.95. Unter Linux kommt mit `zbus` ein Async-Läufer dazu; das ist die
schwerste der drei Abhängigkeiten.

**Der Knoten-Vorrat deckt das gui-Modul ab.** Rollen für Button, CheckBox,
RadioButton, Slider, TextInput, MultilineTextInput, ComboBox, ListBox samt
Option, Table und Cell, Tree und TreeItem, ProgressIndicator, Tab, MenuBar
und MenuItem, Window, Label, Group, ColorWell, DateTime — unsere 24
Widget-Arten finden alle eine Rolle. Zustände `toggled`, `disabled`, `hidden`,
`selected`, `expanded`; Zahlen mit Minimum, Maximum und Schritt; Textauswahl
für die Schreibmarke. **Und Live-Regionen** (`set_live`, höflich oder
dringend): ein Knoten, dessen Text sich ändert, wird vom Bildschirmleser des
Nutzers vorgelesen — mit seiner Stimme, seinem Tempo, auf seiner
Braillezeile. Damit ist eine Ansage aus dem Programm heraus möglich, **ohne
eigene Sprachausgabe.**

**Was AccessKit nicht kann:** den Web-Bau (der Web-Adapter ist angekündigt,
nicht da), und die Sprachausgabe selbst — es redet mit dem Bildschirmleser,
es ist keiner.

### Sprachausgabe — wenn kein Bildschirmleser läuft

Spiele für Blinde (*Audiogames*) gehen traditionell den anderen Weg: das
Programm spricht **selbst**, über die Sprachausgabe des Systems oder direkt
über den laufenden Bildschirmleser. Das braucht keinen Baum, nur einen
Befehl.

| Baustein | System | Geprüft |
|---|---|---|
| **SAPI** (`ISpVoice`, `windows`-Crate Feature `Win32_Media_Speech`) | Windows | diese Maschine: zwei Stimmen (Hedda de-DE, Zira en-US); ein Aufruf **3 ms**, asynchron |
| `say` / `AVSpeechSynthesizer` | macOS | nicht gemessen, Bordmittel |
| `spd-say` (speech-dispatcher) / `espeak-ng` | Linux | nicht gemessen, nicht überall installiert |
| **NVDA Controller Client** (`nvdaControllerClient64.dll`: `nvdaController_speakText`, `speakSsml`, `isSpeaking`, `getProcessId`) | Windows, nur NVDA | LGPL 2.1, darf mitgeliefert werden, wird zur Laufzeit geladen (fehlt sie, fehlt nur NVDA) |
| [Tolk](https://github.com/dkager/tolk) (NVDA, JAWS, ZoomText, SAPI unter einer Haube) | Windows | LGPLv3, C++-DLL, seit Jahren still — [SRAL](https://github.com/m1maker/SRAL) ist der jüngere Nachfolger |

Der Preis der Selbst-Sprache: sie geht am Bildschirmleser **vorbei**. Wer
NVDA mit seiner Stimme und 400 Wörtern je Minute gewohnt ist, bekommt SAPI
mit Hedda; Braille bekommt er gar nicht; Sprachsteuerung und Vergrößerung
gewinnen nichts. Für ein Spiel, das ohnehin eigene Regeln hat, ist das üblich
und akzeptiert. Für eine Rechnungsverwaltung ist es der falsche Weg.

### Was es sonst gäbe

Einen **UIA-Anbieter von Hand** schreiben (`windows`-Crate,
`Win32_UI_Accessibility`): das ist genau, was `accesskit_windows` intern tut
— nur einmal, nur für Windows, mit allen Fallen (Threading, Pattern-Interfaces,
Ereignisse) selbst. Verworfen.

### Prüfbarkeit

Das Wichtigste an einer Barrierefreiheits-Zusage ist, dass sie **nicht vom
Autor selbst** geprüft wird. Der fremde Leser steht schon bereit: die Messung
oben ist PowerShell mit `System.Windows.Automation`, ohne neue Abhängigkeit —
dasselbe Werkzeug zählt nach einer Umsetzung Knöpfe mit Namen, löst über das
Invoke-Muster einen Klick aus (der bei `GUI_CLICKED` ankommen muss), liest
ein Textfeld über das Value-Muster und verfolgt den Fokus beim Tabben. Der
Windows-Runner der CI kann das. **macOS und Linux nicht:** dem prüfenden
Prozess fehlt auf dem macOS-Runner die Berechtigung, und dem Linux-Runner der
Sitzungs-Bus. Diese beiden Adapter wären also *nach bestem Wissen* gebaut,
wie die Pakete des Installers — und so müsste es dann auch dastehen.

## 4. Vier Wege

### A. Handwerk ohne neue Abhängigkeit

Die Zeilen aus der Tabelle in Abschnitt 2, die rot sind, ohne dass es dafür
eine Bibliothek braucht:

* **Menüleiste per Tastatur** — Alt oder F10 öffnet das erste Menü, Pfeile
  laufen, Enter wählt, ESC schließt. Ohne das ist jedes Programm mit Menü
  für Tastaturnutzer nur über auswendig gelernte Kürzel bedienbar.
* **Tab-Reihenfolge setzbar** — ein Befehl, der die Reihenfolge festlegt,
  statt sie dem Anlegen zu überlassen.
* **Tooltip auch bei Tastaturfokus** — der Hilfetext steht heute nur der
  Maus zur Verfügung.
* **`light`-Thema nachziehen** — gedämpfter Text von 2,76:1 auf mindestens
  4,5:1; das Kontrast-Thema in der Doku als solches benennen.
* **Ein Kapitel für Autoren**: nie nur Farbe, Text groß genug, jede
  Maus-Geste auch als Taste, Ton nie als einzige Rückmeldung, Bewegung
  abschaltbar. Das ist Programm-Sache, aber niemand tut es, wenn es nirgends
  steht.

Aufwand ein bis zwei Tage. Hilft Sehbehinderten und Tastaturnutzern
spürbar. **Für Blinde ändert es nichts** — der Baum bleibt leer.

### B. Selbst sprechen

Ein Befehlssatz für Sprachausgabe:

```text
SPEAK(text$ [, unterbrechen])   ' spricht; unterbrechen = laufende Ansage abbrechen
SPEAK_STOP()
SPEAKING() -> BOOLEAN
GUI_SPEAK(an)                   ' das gui spricht Fokuswechsel selbst:
                                ' "Knopf Speichern", "Kaestchen Bezahlt, angehakt"
```

Unter Windows SAPI im eigenen Prozess (3 ms, nicht 194), dazu die
NVDA-DLL, wenn sie neben der Exe liegt und NVDA läuft; unter macOS `say`,
unter Linux `spd-say`. Aufwand zwei bis drei Tage. **Deckt Spiele ab**, die
gar kein gui benutzen — ein Text-Adventure oder ein Audiogame braucht genau
das und nichts sonst. Prüfbar über *SPEAKING* und, unter Windows, über
den Zustand der Stimme.

Deckt **nicht** ab: Braille, Sprachsteuerung, Vergrößerung, die Gewohnheiten
des Nutzers. Und wenn später doch ein Baum kommt, reden zwei Stimmen
gleichzeitig — `GUI_SPEAK` müsste dann schweigen, sobald ein
Bildschirmleser aktiv ist. Das lässt sich lösen, ist aber ein Zeichen, dass
B allein die falsche Grundlage für Anwendungen ist.

### C. AccessKit einbinden

Das gui-Modul baut in `GUI_UPDATE` einen Baum, sobald ein Hilfsprogramm
danach fragt — sonst nichts. Für jedes Fenster ein Knoten `Window`, für jedes
Widget einer mit Rolle nach `Kind`, Beschriftung aus Text oder Tooltip, Wert,
Rechteck aus `abs_rect`, Zuständen aus `enabled`, `visible`, `checked`,
Fokus aus dem bestehenden Fokus; Menüleiste und Reiter als eigene Knoten;
Listen- und Baumeinträge, Tabellenzellen als Kinder. Aktionen zurück auf die
Mechanik, die es schon gibt: Fokus auf `GUI_FOCUS`, Klick auf dieselbe Stelle,
die ein Mausklick setzt (damit `GUI_CLICKED` und `GUI_ON_CLICK` feuern), Wert
setzen auf die Setter, Auswahl auf die Auswahl. Eine Live-Region für Ansagen:

```text
GUI_ANNOUNCE(text$ [, dringend])   ' der Bildschirmleser des Nutzers spricht es
```

**Die harte Stelle sind die Textfelder.** Ein Bildschirmleser will nicht nur
den Inhalt, sondern die Schreibmarke und die Auswahl, zeichenweise und
zeilenweise — das ist AccessKits Text-Modell mit Läufen und Positionen, und
das ist Arbeit. Eine erste Fassung kann Inhalt und Beschriftung liefern
(Rolle TextInput mit Wert) und die Schreibmarke nachziehen; benutzbar ist
das, nur das zeichenweise Lesen fehlt dann noch.

**Nicht abgedeckt:** die Zeichenfläche (für den Leser ein Bild — das Programm
könnte ihr eine Beschreibung geben), das `ui`-Modul (Immediate Mode, kein
Zustand, den man abbilden könnte — es müsste je Bild einen Baum aus den
Aufrufen bauen; machbar, nicht in der ersten Fassung), der Web-Bau.

**Aufwand:** Windows eine bis zwei Wochen, davon der Baum und die Aktionen
drei bis vier Tage, Textfelder der Rest. macOS und Linux je zwei bis drei
Tage zusätzlich — **ohne Möglichkeit, es hier zu prüfen.** Prüfstein unter
Windows: der UIA-Baum des Beispiels 156 (alle Widget-Arten) hat je Widget
einen Knoten mit Rolle und Namen; Invoke auf den Knopf lässt `GUI_CLICKED`
feuern; das Value-Muster liest, was `GUI_TEXT` liefert; der Fokus im Baum
folgt der Tab-Taste; eine Ansage erscheint als Live-Knoten.

Was man dafür bekommt, ist die ganze Tabelle aus Abschnitt 1 auf einmal:
NVDA, JAWS, Sprachausgabe, VoiceOver, Orca, die Lupe, die Sprachsteuerung,
die Schaltersteuerung — alles, was UIA spricht, ohne dass das Programm etwas
davon wissen muss. Ein Formular aus dem Form-Designer wäre damit ohne eine
Zeile Zusatzcode für einen Bildschirmleser bedienbar.

### D. C und B zusammen

C für Anwendungen, dazu aus B nur `SPEAK` für Spiele ohne gui. Die Ansage
läuft über den Baum, wenn ein Hilfsprogramm zuhört, sonst über die
Sprachausgabe — ein Befehl, zwei Wege. Aufwand C plus zwei Tage.

## 5. Nebeneinander

| | A Handwerk | B Sprechen | C AccessKit | D C + SPEAK |
|---|---|---|---|---|
| Bildschirmleser liest die Oberfläche | nein | nein (spricht selbst) | **ja** | ja |
| Braille, Lupe, Sprachsteuerung | nein | nein | **ja** | ja |
| Spiel ohne gui kann sprechen | nein | ja | nur Ansage | **ja** |
| Tastatur vollständig (Menü) | **ja** | nein | nein | nein |
| Neue Abhängigkeit | keine | keine (SAPI im windows-Crate) | 4 Crates, Linux mit zbus | wie C |
| Hier prüfbar | ja | Windows | Windows | Windows |
| Aufwand | 1–2 Tage | 2–3 Tage | 1–2 Wochen + 2×2–3 Tage | + 2 Tage |
| Hilft | Sehbehinderten, Tastatur | Blinden in Spielen | Blinden in Anwendungen, allen aus Abschnitt 1 | beiden |

## 6. Empfehlung

**A sofort, dann C mit Windows zuerst, B nur als der eine Befehl aus D.**

A ist billig, ohne Risiko und schließt Lücken, die auch Sehende stören —
ein Menü, das man ohne Maus nicht erreicht, ist einfach ein Fehler.

C ist der Punkt, an dem die Aussage „damit schreibt man Anwendungen" steht
oder fällt. Eine Behörde, eine Schule, ein Verein darf eine Anwendung, die
ein Bildschirmleser nicht öffnen kann, oft gar nicht einsetzen; und ein
Toolkit, dessen Fenster für UIA leer sind, wird in dieser Frage nicht
genannt, egal wie gut die Tabelle sortiert. Dass es dafür eine reine
Rust-Bibliothek mit einem Beispiel für genau unsere Lage (fremdes Fenster)
gibt, ist derselbe Glücksfall wie raylib und Kira: einbinden statt bauen.
**Windows zuerst**, weil es das einzige System ist, auf dem der Prüfstein
hier laufen kann — und weil ein Baum, der nur behauptet wird, nichts wert
ist. Die anderen beiden Adapter danach, mit dem Vermerk, dass sie ungeprüft
sind, wie beim Installer.

B allein wäre die Abkürzung, die sich rächt: schnell hörbar, aber am
Nutzer vorbei. Der eine Befehl `SPEAK` aus D gehört trotzdem dazu — für das
Text-Adventure, das keinen Knopf hat.

**Reihenfolge:** A (1–2 Tage) → C Windows mit Prüfstein (1–2 Wochen) →
SPEAK und Ansage (2 Tage) → macOS/Linux-Adapter (je 2–3 Tage, ungeprüft).

## 7. Was ohne Entscheidung schon geht

Ein Programm kann heute schon anbieten, was das Toolkit hergibt — und sollte
es in der Doku finden:

* `GUI_THEME_PRESET("contrast")` und `GUI_SCALE(2.0)` als Einstellung
  anbieten, nicht nur als Möglichkeit haben.
* Jede Maus-Geste auch als Taste: das gui tut es von selbst, die
  Zeichenfläche nicht.
* Fehler nie nur rot: `GUI_ERROR_LABEL` macht es richtig — das Muster gilt
  für alles andere auch.
* Wer heute sprechen muss, kann es teuer: `SHELL_START` mit PowerShell und
  `System.Speech` — 194 ms je Ansage, für ein Menü zu langsam, für eine
  Meldung alle paar Sekunden brauchbar. Unter macOS `say`, unter Linux
  `spd-say`.
