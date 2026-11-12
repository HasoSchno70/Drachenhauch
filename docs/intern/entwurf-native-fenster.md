# Entwurf: Mehrere OS-Fenster in Drachenhauch

> **Stand 12.11.2026: Weg E, Stufen 1 und 2 sind gebaut** (`GUI_WINDOW_NATIVE`,
> Windows und Linux; macOS Rueckfall) -- Abschnitt 8. Davor geplant: ein
> gui-Fenster als echtes OS-Fenster im SELBEN Prozess -- gezeichnet wird
> weiter in raylibs Kontext, ein zweites GLFW-Fenster zeigt das Bild nur und
> liefert die Eingabe. Zuerst ein Spike mit Ausstiegskriterium.
>
> **Stand 05.09.2026: Weg B ist gebaut** — `WINDOW_OPEN/SEND/RECV$/ALIVE/CLOSE`
> und `PARENT_SEND/RECV$/ALIVE`, siehe [builtins-core.md](../builtins-core.md#ein-zweites-fenster-window_open).
> Der Kanal ist eine TCP-Verbindung auf 127.0.0.1 statt stdin/stdout (das Kind
> darf weiter `PRINT`). Die drei Prüfsteine unten, gemessen: 1000 Nachrichten
> im Schub hin und zurück ~60 ms; eine **einzelne** Runde kostet ein bis drei
> Bilder des Kindes (17–50 ms), weil es seine Post einmal je Bild liest — der
> Kanal ist nicht der Engpass, die Bildschleife ist es; das Kind stirbt, die
> Eltern leben (`tests/pruef/fenster/fenster_prozess.dhtest`); die Rechnungsverwaltung
> öffnet eine Rechnung im zweiten Fenster in 64 Zeilen (`196_rechnung_fenster.dh`).
> Wege C und D bleiben ungebaut; Fall a (geteilte Bilder und Objekte) bleibt
> ein Nicht-Ziel.

*Untersuchung, keine Umsetzung.* Nach dem sechsten Piloten (der
Rechnungsverwaltung, [PR #76](https://github.com/HasoSchno70/Drachenhauch/pull/76))
blieb eine Lückenliste übrig, deren Punkte das gui-Modul selbst nicht
schließen kann. Der erste davon: **Drachenhauch kennt genau ein OS-Fenster je
Programm.** Dieses Papier misst den heutigen Stand, benennt, woran das hängt,
entwirft vier Wege und empfiehlt einen. Die Entscheidung fällt jemand anders.

Alle Zahlen sind gemessen, nicht behauptet — Stand 05.09.2026, `dhrt` aus
`rust/drachenhauch_runtime/target/release`.

## 1. Was heute geht

| | Befehle | Grenze |
|---|---|---|
| **ein** OS-Fenster je Prozess | `SCREEN`, `SCREEN_NATIVE`, `SCREEN_TRANSPARENT`, `WINDOW_*` (23 Befehle: Größe, Lage, Monitor, Vollbild, Deckkraft, randlos, immer oben, klick-durchlässig, Symbol) | genau eines; ein zweites `SCREEN` ist kein zweites Fenster |
| beliebig viele **gui-Fenster darin** | `GUI_WINDOW`, modal, Dialoge, `GUI_PROMPT`, Reiter, Rollen, Ziehen, Z-Reihenfolge | nur innerhalb des OS-Fensters: nicht über den Rand, nicht auf den zweiten Monitor, kein eigener Eintrag in der Taskleiste |
| Monitore | `MONITOR_COUNT/WIDTH/HEIGHT/X/Y/NAME/REFRESH`, `CURRENT_MONITOR`, `SET_WINDOW_MONITOR` | man kann das **eine** Fenster auf einen Monitor legen, nicht zwei auf zwei |
| OS-Dialoge | `GUI_MESSAGE`, `GUI_CONFIRM`, `FILE_OPEN_DIALOG`, `FILE_SAVE_DIALOG` (rfd) | blockierend, nur im nativen Bau, nicht gestaltbar |
| andere Prozesse | `SHELL`, `SHELL_START`, `TASK_START` (startet `dhrt call datei funktion`), `NET_TCP/UDP`, `MQTT`, Dateien | ein Kind liefert **eine** Antwort am Ende; ein Fenster hat es nicht |

Gemessen, ein zweiter `dhrt`-Prozess mit eigenem `SCREEN` bis zum ersten
Bild (drei Läufe, Windows 11, SSD):

```text
0,394 s   0,373 s   0,403 s
```

Die Exe ist 17 MB groß; ein zweiter Prozess kostet also Startzeit im Bereich
einer halben Sekunde und einige zehn MB Arbeitsspeicher.

## 2. Woran das eine Fenster hängt

Es ist keine Entscheidung von Drachenhauch, sondern von drei Schichten
darunter, und jede einzelne reicht:

1. **raylib** hält seinen gesamten Zustand in einem globalen `CoreData CORE`
   (`rcore.c:399`) und genau **einem** Fenster-Handle (`platform.handle =
   glfwCreateWindow(...)`, `rcore_desktop_glfw.c:1637/1651`). Eingabe,
   Zeitgeber, Bildschirmgröße, `BeginDrawing` — alles gegen dieses eine
   Handle. Mehrere Fenster stehen bei raylib erklärtermaßen nicht auf dem
   Plan; die Bibliothek ist um Einfachheit gebaut, nicht um Fenster.
2. **raylib-rs** verbietet den zweiten Versuch ausdrücklich:
   `panic!("Attempted to initialize raylib-rs more than once!")`
   (`core/mod.rs:628`).
3. **`graphics.rs`** (6000 Zeilen) ist auf ein `RaylibHandle` gebaut: 145
   direkte Aufrufe `self.rl.…`, ein Aufnahme-Modell mit 71 Zeichenbefehlen
   (`Cmd`), das beim `FLIP` **den** Bildschirm rendert, dazu Kamera, Ebenen,
   Render-Ziele, PostFX, Automation, Schatten und die Kira-Bindung — alles
   einfach vorhanden, nicht je Fenster.

Dazu kommt der Web-Bau: im Browser gibt es ohnehin nur eine Zeichenfläche.

## 3. Was „native Fenster" konkret heißen würde

Der Wunsch zerfällt in vier Fälle, und sie sind verschieden teuer:

| Fall | Beispiel | braucht |
|---|---|---|
| **a** Werkzeugpalette oder Einstellungen auf dem zweiten Monitor | Mischpult, Sprite-Editor mit Palette daneben | ein zweites Fenster, das Zustand mit dem ersten **teilt** (dieselben Bilder, dasselbe Dokument) |
| **b** Dokumentfenster je Datei | drei Rechnungen nebeneinander, jede in der Taskleiste | eigenständige Fenster, die sich **wenig** teilen (dieselbe Datenbank) |
| **c** Abreißbare Panels, Andocken | IDE-artige Werkzeuge | wie a, plus Fenster-in-Fenster-Wechsel zur Laufzeit |
| **d** OS-Dialoge | Datei öffnen, Ja/Nein | **gibt es** (rfd) |

Fall d ist erledigt. Fall c ist die Königsklasse und in keinem der Wege unten
billig. Die Entscheidung dreht sich um a und b.

## 4. Vier Wege

### A. Bleiben — gui-Fenster im einen OS-Fenster

Nichts bauen; die Grenze als Nicht-Ziel dokumentieren. Für a hilft schon
heute `SCREEN_NATIVE` über den größten Monitor plus gui-Fenster, oder ein
großes randloses Fenster über zwei Monitore (`WINDOW_UNDECORATED`,
`SET_WINDOW_POS`, `WINDOW_TOPMOST`).

- **Kosten:** null.
- **Was fehlt:** Taskleiste, echte Trennung, Fenster auf verschiedenen
  Monitoren mit verschiedenen Skalierungen.
- **Risiko:** keins. Der Satz „ein Programm, ein Fenster" ist ehrlich und
  entspricht dem, was Spiele und Werkzeuge in dieser Sprache bisher taten.

### B. Fenster als Prozess — `WINDOW_OPEN`

Ein zweites Fenster ist ein zweiter `dhrt`, verbunden über einen Kanal. Das
ist der Weg, den `TASK_START` schon geht (`dhrt call`), nur dass der Kanal
offen bleibt statt einer einzigen Antwort am Ende.

```text
' im Hauptprogramm
DIM k AS KANAL
k = WINDOW_OPEN("palette.dh", "Palette", 320, 600)     ' zweiter dhrt, eigenes SCREEN
WINDOW_SEND(k, "farbe " + STR$(farbe))                ' eine Zeile Text
WHILE WINDOW_ALIVE(k)
    DIM z AS STRING : z = WINDOW_RECV$(k)              ' "" wenn nichts da
    IF z <> "" THEN verarbeite(z)
    ...
WEND

' in palette.dh
PARENT_SEND("gewaehlt 3")
DIM z AS STRING : z = PARENT_RECV$()
```

Der Kanal sind stdin/stdout des Kindes, zeilenweise, bewusst **Text**
(JSON, wenn es strukturiert sein soll — das json-Modul ist da). Mehr braucht
`hintergrund.rs` nicht: die `Auftraege<T>`-Verwaltung und der
Prozessstart existieren, es fehlen zwei Leser-Threads je Kind und die
Fensterbefehle im Kind.

- **Was man bekommt:** echte OS-Fenster mit allem, was das OS dazutut —
  Taskleiste, eigener Monitor, eigene Skalierung, Alt+Tab, und ein Absturz
  des Kindes reißt das Hauptfenster nicht mit. Fall b vollständig, Fall a
  mit einer Einschränkung.
- **Die Einschränkung:** **kein geteilter Zustand.** Ein `IMAGE`-Handle, ein
  Array, ein Objekt gehört einem Prozess. Die Palette lädt ihre Bilder
  selbst, das Dokument liegt in einer Datei oder Datenbank, die beide
  öffnen. Das ist genau die Grenze, die `TASK_START` auch zieht (dort
  wandern nur Text und Zahlen). Ein Programm, das sein Dokument ohnehin in
  SQLite hält — wie der sechste Pilot — merkt davon wenig; ein Sprite-Editor
  mit Ebenen im Speicher merkt es sofort.
- **Kosten:** ~0,4 s je Fenster beim Öffnen, einige zehn MB je Fenster,
  Nachrichten statt Aufrufe. Fokus und Z-Reihenfolge zwischen den Fenstern
  regelt das OS, nicht das Programm.
- **Aufwand:** klein bis mittel — geschätzt zwei bis drei Tage einschließlich
  Tests (die es gibt: Kind starten, Nachricht hin und zurück messen, Kind
  abschießen, Haupt läuft weiter), Doku, Beispiel. Kein Eingriff in
  `graphics.rs`, kein Eingriff in raylib, Web-Bau unberührt (dort ist der
  Befehl schlicht ein Fehler).
- **Risiko:** gering. Was schiefgehen kann, ist bekannt: Puffer, die
  volllaufen, wenn niemand liest (lösbar: begrenzte Warteschlange, älteste
  fällt weg — wie beim MIDI-Modul); Zombies, wenn das Hauptprogramm stirbt
  (lösbar: das Kind beendet sich, sobald stdin schließt).

### C. Zweites Fenster im selben Prozess über GLFW direkt

raylib umgehen: mit GLFW ein zweites Fenster samt geteiltem GL-Kontext
öffnen und dort mit rlgl zeichnen. Klingt nach einem Nachmittag und ist es
nicht: rlgl hält seinen Zustand ebenfalls global (ein Batch, ein
Standard-Shader, eine Matrix), `BeginDrawing`/`EndDrawing` und die gesamte
Eingabe hängen an `CORE`. Man müsste raylib **forken** und die Kernstrukturen
je Fenster instanziieren — ein Umbau, den das raylib-Projekt selbst seit
Jahren ablehnt, und den jede raylib-Aktualisierung von vorn beginnen ließe.

- **Was man bekäme:** Fall a vollständig (geteilte Handles, ein Prozess),
  Fall c denkbar.
- **Aufwand:** sehr groß, offen nach oben; im Web-Bau unmöglich.
- **Risiko:** hoch — ein Fork von raylib ist Wartung auf Dauer, und die
  Fehler wären die schlimmste Sorte (globaler Zustand, der zwischen zwei
  Fenstern springt).

### D. Die Plattformschicht tauschen (winit + wgpu/glow)

Alles, was in `graphics.rs` an raylib hängt, neu schreiben — und mit ihm 3D
(Modelle, GLTF-Animation, PBR, IBL, Schatten), Shader, PostFX, Bild-Ein- und
-Ausgabe, Automation, Gesten, Bitmap-Schriften. Dazu die Rust-Tests, die
Piloten, die Buchbeispiele. Das ist ein neues Projekt, keine Erweiterung —
und es würde für die Fensterfrage die stabilste Schicht der Laufzeit
opfern.

- **Aufwand:** Monate.
- **Risiko:** das Projekt selbst. Nicht verhältnismäßig für Fenster.

## 5. Nebeneinander

| | A bleiben | B Prozess | C GLFW/Fork | D Plattform |
|---|---|---|---|---|
| Fall a (Palette, geteilter Zustand) | teilweise (großes Fenster) | über Nachrichten | ja | ja |
| Fall b (Dokumentfenster) | nein | **ja** | ja | ja |
| Fall c (Andocken) | nein | nein | schwer | schwer |
| Taskleiste, eigener Monitor | nein | ja | ja | ja |
| Aufwand | 0 | 2–3 Tage | Wochen, offen | Monate |
| Wartung danach | keine | gering | raylib-Fork | alles |
| Web-Bau | wie heute | Befehl = Fehler | unmöglich | neu |
| Eingriff in graphics.rs / raylib | nein | nein | ja / ja | ersetzt |
| Testbar mit den vorhandenen Mitteln | — | ja (Prozesse, Zeit, Ausgabe) | kaum | neu |

## 6. Empfehlung

**B, als begrenztes Experiment mit Ausstiegskriterien — und A als
ausgesprochenes Nicht-Ziel für alles, was B nicht kann.**

Begründung: B ist der einzige Weg, dessen Kosten man kennt und dessen
Nutzen man vorher messen kann. Er passt zur Bauart der Laufzeit
(`TASK_START` ist schon ein Prozess, `SHELL_START` auch), er lässt raylib
und `graphics.rs` unangetastet, und er liefert genau das, was die
Rechnungsverwaltung als Nächstes brauchte: ein zweites Dokumentfenster.
Fall a bleibt dabei eine bewusste Grenze — geteilte Bilder und Objekte
zwischen Fenstern gibt es nicht, und das steht dann so in der Doku statt als
Überraschung im Programm.

Das Experiment hätte drei Prüfsteine, jeder messbar:

1. **Roundtrip** einer Nachricht Haupt → Kind → Haupt unter 5 ms, gemessen
   über 1000 Nachrichten (sonst taugt der Kanal nicht für „Farbe gewählt").
2. **Isolation:** das Kind stirbt (`EXIT(3)` oder ein Laufzeitfehler), das
   Hauptprogramm läuft weiter und erfährt es über `WINDOW_ALIVE`.
3. **Der Pilot:** die Rechnungsverwaltung öffnet eine Rechnung in einem
   zweiten Fenster (`rechnung_fenster.dh`), beide arbeiten auf derselben
   Datenbank, das Hauptfenster sieht die Änderung nach `WINDOW_RECV$`.
   Wenn dieser Fall sich nicht in unter 60 Zeilen Programm schreiben lässt,
   ist die Schnittstelle falsch.

Was man B **nicht** geben sollte: eine Fassade, die so tut, als wären es
Aufrufe — ein „rufe im anderen Fenster die Funktion X" mit Rückgabewert.
Die Verlockung ist groß, und
das Ergebnis wäre ein RPC-System mit allen Fragen nach Wartezeiten und
Fehlern — dieselben Fragen, die der Entwurf zu `TASK_START` schon einmal mit
„nur Text und Zahlen" beantwortet hat.

## 7. Was ohne Entscheidung schon geht

Drei Kleinigkeiten, die die Lücke heute schon kleiner machen und keinen der
vier Wege vorwegnehmen:

- Die Grenze steht als Satz in `docs/module-gui.md` (Limitationen): „ein
  Programm, ein OS-Fenster; gui-Fenster leben darin."
- Ein Rezept für Fall a mit Bordmitteln: randloses Fenster über zwei
  Monitore, gui-Fenster je Monitor-Hälfte (`MONITOR_X/Y`, `WINDOW_UNDECORATED`).
- Ein Rezept für eine Vorschau im zweiten Prozess mit dem, was da ist:
  `SHELL_START("dhrt", "run", "vorschau.dh", pfad)` — ohne Rückkanal, aber
  mit eigenem Fenster. Das ist B ohne Kanal und zeigt, ob der Bedarf echt
  ist, bevor jemand den Kanal baut.

## 8. Weg E: zeigen statt zeichnen (Plan, 12.11.2026)

Weg B hat Fall b geloest. Offen blieb alles, was Zustand TEILT: ein Dialog
oder eine Werkzeugpalette ausserhalb des Hauptfensters, eine abgerissene
Dock-Leiste (Fall a und c). Die Durchsicht der gui nach 2026.46 nannte das
die groesste Luecke gegenueber Qt, WinForms und GTK.

### Die Idee

Weg C scheiterte daran, dass raylib mit ZWEI Fenstern zeichnen muesste. Weg E
laesst raylib bei einem Fenster:

1. Die gui zeichnet ein gui-Fenster wie bisher, mit raylib im Kontext des
   Hauptfensters -- aber in ein Render-Ziel statt auf den Schirm
   (`render_ziel`, dasselbe wie fuer `GUI_DRAW_WINDOW`).
2. Ein zweites OS-Fenster, direkt mit GLFW angelegt
   (`glfwCreateWindow(..., share = Hauptfenster)`), teilt den GL-Kontext und
   zeigt die Textur unveraendert: ein Rechteck, eigene VAO (VAOs teilen sich
   zwischen Kontexten nicht), `glfwSwapBuffers`, zurueck in den Hauptkontext.
3. Maus, Tasten und Text des zweiten Fensters kommen ueber GLFW-Rueckrufe
   an genau DIESEM Fenster (raylib setzt seine nur am Hauptfenster) und gehen
   in die gui -- verschoben auf die Koordinaten des gui-Fensters.

Die Laufzeit ruft GLFW schon heute direkt (`flanken.rs`:
`glfwSetMouseButtonCallback`, `glfwSetKeyCallback`; `graphics.rs`:
`glfwCreateCursor`, `glfwGetKeyScancode`) -- der Weg dorthin ist erprobt,
ohne raylib zu forken. Alles bleibt in einem Prozess: Bilder, Objekte, das
Dokument sind geteilt.

### Stufen

**Stufe 0 -- Spike (ein bis zwei Tage, Wegwerf-Code).** Ein Probeprogramm
oeffnet ein zweites GLFW-Fenster mit geteiltem Kontext, das Hauptfenster
rendert ein gui-Fenster in ein Render-Ziel, das zweite zeigt es; Klicks
kommen an. Gemessen: beide Fenster bei 60 Bildern; der Zustand von rlgl
(Batch, Shader, Matrizen) uebersteht den Wechsel der Kontexte; Windows, dann
Linux (X11) und macOS. **Ausstieg:** laesst sich rlgl nach dem Wechsel nicht
sauber weiterbenutzen, ist Weg E tot -- Kosten zwei Tage.

### Stufe 0, Ergebnis unter Windows (12.11.2026): bestanden

Probeprogramm `rust/drachenhauch_runtime/examples/spike_zweitfenster.rs`
(Wegwerf-Code, baut gegen dasselbe raylib/GLFW wie dhrt; Bau:
`cargo build --release --example spike_zweitfenster --features graphics`
mit der Umgebung aus `rust/build_runtime.py`). Es prueft sich selbst und
endet mit `ERGEBNIS gesamt gut`, Exit 0.

| Frage | gemessen (Windows 11, drei Laeufe) |
|---|---|
| zweites GLFW-Fenster mit geteiltem Kontext | angelegt; FBO mit der geteilten Textur vollstaendig |
| zeigt das AKTUELLE Bild aus raylibs Render-Ziel | 10 bzw. 12 von 10/12 Rueckleseproben richtig, 0 falsch (die Grundfarbe wechselt je Sekunde) |
| Klicks im zweiten Fenster (WM_LBUTTONDOWN) | 2 von 2 im eigenen GLFW-Rueckruf |
| Hauptfenster zeichnet nach den Wechseln richtig | ja, vor und nach dem Schliessen des zweiten Fensters |
| Kreuz des zweiten Fensters (WM_CLOSE) | `glfwWindowShouldClose`, Fenster weg, Hauptfenster laeuft weiter |
| Bildrate | 59 im Mittel (Ziel 60), 240 Bilder in 4,0 s |
| Kosten des Zeigens je Bild | Median 0,43-0,49 ms, 95 % unter 0,83 ms |

Wie es geht: Texturen teilen sich zwischen Kontexten, Framebuffer und VAOs
nicht. Der zweite Kontext bekommt darum eine eigene FBO mit der geteilten
Textur als Lesepuffer und kopiert sie mit `glBlitFramebuffer` (gespiegelt,
weil raylibs Render-Ziele auf dem Kopf stehen) in seinen Bildschirm -- ohne
Shader, ohne VAO, ohne rlgl. `glfwSwapInterval(0)` im zweiten Kontext, sonst
wartet jedes Fenster auf seinen eigenen Bildwechsel. Reihenfolge je Bild:
raylib zeichnet ins Render-Ziel und ins Hauptfenster, `EndDrawing` tauscht
(und flusht damit), dann Kontext wechseln, blitten, tauschen, ZURUECK in den
Hauptkontext.

Gegenproben (`SPIKE_FEHLER=`): ohne Blit fallen alle Rueckleseproben des
zweiten Fensters (0 richtig, 10 falsch); ohne den Wechsel zurueck zeichnet
das Hauptfenster falsch -- der Kontextwechsel ist also genau die Stelle, an
der Weg E haengt, und er laesst sich vollstaendig zuruecknehmen.

Funde am Rand:
- `CloseWindow` gibt es in raylib UND in user32: wer im selben Programm
  `CloseWindow` ruft und user32 frueh zieht (etwa ueber das windows-Crate),
  bekommt LNK2005. dhrt ruft es nicht direkt; in Stufe 1 bleibt das so.
- `LoadImageFromScreen` mitten im Bild sieht den noch gesammelten Batch nicht
  -- vorher `rlDrawRenderBatchActive()`. (Die erste Probe am Hauptfenster
  meldete deshalb faelschlich FALSCH.)

### Stufe 0 unter Linux (WSLg, 12.11.2026): bestanden, bis auf das Kreuz

Ubuntu 24.04 in WSL2, WSLg (XWayland), Mesa 25.2 llvmpipe (OpenGL 4.5 core,
also in Software). dhrt baut GLFW nur fuer X11 (`GLFW_BUILD_X11`) und laeuft
auf Wayland-Desktops ueber XWayland -- genau dieser Weg ist hier gemessen.
Gebaut in WSL mit eigenem Zielordner (`CARGO_TARGET_DIR=/root/dht_spike`,
`LIBCLANG_PATH=/usr/lib/llvm-18/lib`); Klicks von aussen ueber `xdotool`
(`SPIKE_AUSSEN=1`).

| Frage | gemessen |
|---|---|
| Render-Ziel, FBO im zweiten Kontext | vollstaendig (beide Kontexte 4.5 core) |
| zeigt das aktuelle Bild | 24 von 24 Rueckleseproben richtig |
| echte X11-Klicks im zweiten Fenster | 2 von 2 |
| Hauptfenster nach den Wechseln | richtig |
| Bildrate, Zeigen je Bild | 60; Median 1,07 ms (Software-Rendern) |
| Kreuz ueber den Fenstermanager | **nicht pruefbar**: WSLgs X-Server fuehrt keine Fensterliste (`wmctrl -c` scheitert); GLFW meldet WM_DELETE_WINDOW unter X11 wie WM_CLOSE unter Windows |

**Der erste Lauf war schwarz -- auch im Hauptfenster, auch OHNE zweites
Fenster.** Ursache ist die bekannte Linux-Falle (`graphics.rs`,
`render_ziel`): raylibs `LoadRenderTexture` laesst unter gcc/Mesa die Farbe
ohne Speicher (`glCompressedTexImage2D` statt `glTexImage2D`). Das
Spike-Programm legt das Ziel jetzt wie dhrt selbst an, mit leerem Puffer;
in Stufe 1 gilt dasselbe: **ein Render-Ziel fuer Weg E nur ueber
`render_ziel`**, nie ueber `LoadRenderTexture`. Abgegrenzt mit
`SPIKE_OHNE_ZWEIT=1` (nur das Hauptfenster): erst so war klar, dass der
Fehler nicht am zweiten Fenster lag.

### Stufe 0 unter macOS: auf der CI nicht pruefbar

Ein Einmal-Lauf auf den CI-Rechnern (12.11.2026, `macos-latest` = Apple M1
virtuell mit macOS 26.7, `macos-15-intel` = i7-8700B mit macOS 15.7) kam
nicht bis Weg E: schon raylibs HAUPTfenster scheitert mit "NSGL: Failed to
find a suitable pixel format" -- dieselbe Grenze, die `package.yml` am
19.09.2026 festhielt. Die macOS-Rechner der CI haben kein OpenGL fuer
Fenster; Weg E unter macOS laesst sich nur auf einem echten Mac pruefen.

Was fuer macOS spricht: geteilte Kontexte sind unter NSGL der gewoehnliche
Fall (GLFW nutzt dort `shareContext`), `glBlitFramebuffer` gehoert zu
OpenGL 3.2+, und Apple liefert 4.1 core. Der zweite Kontext braucht
`GLFW_OPENGL_FORWARD_COMPAT` (im Spike gesetzt). Was dagegen sprechen
koennte: GLFW verlangt unter macOS alle Fensterbefehle im Hauptfaden --
dhrt zeichnet dort ohnehin.

**Stand Stufe 0: bestanden unter Windows und Linux; macOS offen bis zu
einem Lauf auf einem Mac** (`cargo build --release --example
spike_zweitfenster --features graphics`, dann das Programm starten; ohne
automatische Klicks prueft es Bild, Hauptfenster und Bildrate).

**Folge fuer Stufe 1 (12.11.2026, kein Mac zur Hand):** echte Fenster
werden nur unter Windows und Linux eingeschaltet; unter macOS nimmt der
Befehl den Rueckfall (ein gui-Fenster im Hauptfenster wie bisher), bis ein
Lauf des Spikes auf einem Mac bestanden ist. So bekommt kein Mac-Nutzer
einen ungeprueften Weg, und ein Programm laeuft dort unveraendert.

**Stufe 1 -- der Befehl (etwa eine Woche).** Ein Befehl macht ein gui-Fenster
zum OS-Fenster (Arbeitsname: GUI_WINDOW_NATIVE mit Fenster und an/aus, noch nicht gebaut): Titel,
Taskleiste, zweiter Monitor, Kreuz des Systems (`GUI_WINDOW_CLOSE_REQUESTED`).
Alles andere bleibt: Widgets, Ereignisse, Stildatei, Massstab -- das
Programm muss nicht wissen, wo sein Fenster steht. **Wo es keine echten
Fenster gibt** (Web, CI ohne Schirm, ein Fehler beim Anlegen), bleibt es
still ein gui-Fenster wie heute; ein Programm laeuft ueberall gleich.
Pruefung: Umrechnung von Lage und Eingabe als Rust-Test ohne Fenster, echte
Fenster in eigenen Faellen mit `--- seriell`.

**Stufe 2 -- Verhalten (etwa eine Woche).** Modale Dialoge als echte Fenster
ueber dem Hauptfenster (Windows: Besitzer-Fenster, damit kein eigener
Taskleisten-Eintrag), Fokus und Tastatur zwischen den Fenstern, IME und
Tooltips im zweiten Fenster, eigener Massstab bei anderem DPI des Monitors.
Vorzeigen: Einstellungen in IDE und Studio als herausziehbare Dialoge.

**Stufe 3 -- abreissbare Dock-Leisten (ein bis zwei Wochen).** Fall c: eine
Leiste aus dem Hauptfenster ziehen und wieder andocken. Baut auf 1 und 2 auf
-- eine abgerissene Leiste ist ein gui-Fenster, das zum OS-Fenster wird.

**Stufe 4 -- Schliff.** Bildschirmleser (AccessKit je Fenster), Eigenheiten
von macOS und Linux, Handbuch, ein Beispielprogramm.

### Risiken

| Risiko | was es kostet | Gegenmittel |
|---|---|---|
| rlgl-Zustand beim Kontextwechsel | Weg E insgesamt | Stufe 0 klaert es zuerst |
| Wayland erlaubt keine eigene Fensterlage | Fenster stehen, wo der Compositor will | Lage als Wunsch; Rueckfall bleibt |
| zwei `SwapBuffers` mit Vsync halbieren die Bildrate | ruckelnde Fenster | zweites Fenster mit Intervall 0 |
| Popup-Menues ueber den Rand des zweiten Fensters | abgeschnittene Menues | zuerst innerhalb; spaeter Popups als eigene Fenster |
| AccessKit haengt am Hauptfenster (Subclassing) | Bildschirmleser sieht nur eines | Stufe 4, ein Adapter je Fenster |

### Nebeneinander mit den anderen Wegen

| | B Prozess (gebaut) | E zeigen statt zeichnen |
|---|---|---|
| geteilter Zustand | nein, Nachrichten | ja |
| Fall a, Dialog ausserhalb | ueber Nachrichten | ja |
| Fall c, Andocken | nein | Stufe 3 |
| Eingriff in raylib | nein | nein (GLFW direkt) |
| Web-Bau | Befehl = Fehler | Rueckfall ins eine Fenster |
| Aufwand | erledigt | Spike 2 Tage, danach Wochen |

**Reihenfolge:** Stufe 0 erst nach Fassung 2026.47 und den kleinen
gui-Punkten (Abfragen, Rueckgaengig als API, Aussehen des Systems).
