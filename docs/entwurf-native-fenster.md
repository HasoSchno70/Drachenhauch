# Entwurf: Mehrere OS-Fenster in Drachenhauch

> **Stand 05.09.2026: Weg B ist gebaut** — `WINDOW_OPEN/SEND/RECV$/ALIVE/CLOSE`
> und `PARENT_SEND/RECV$/ALIVE`, siehe [builtins-core.md](builtins-core.md#ein-zweites-fenster-window_open).
> Der Kanal ist eine TCP-Verbindung auf 127.0.0.1 statt stdin/stdout (das Kind
> darf weiter `PRINT`). Die drei Prüfsteine unten, gemessen: 1000 Nachrichten
> im Schub hin und zurück ~60 ms; eine **einzelne** Runde kostet ein bis drei
> Bilder des Kindes (17–50 ms), weil es seine Post einmal je Bild liest — der
> Kanal ist nicht der Engpass, die Bildschleife ist es; das Kind stirbt, die
> Eltern leben (`tests/test_fenster_prozess.py`); die Rechnungsverwaltung
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
