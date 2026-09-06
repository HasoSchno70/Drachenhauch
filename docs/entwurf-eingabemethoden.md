# Entwurf: Eingabemethoden (IME) und fremde Schriften

> **Stand 06.09.2026: A und B sind gebaut.** Der Grundvorrat reicht jetzt
> von ASCII bis Kyrillisch samt `€` und Interpunktion; was darüber hinausgeht,
> backt die Laufzeit **auf Zuruf** aus den Systemschriften nach (nur die
> gebrauchten Zeichen, je Datei ein Eintrag; Zeichen ohne Glyphe im gewählten
> Font laufen zeichenweise über die Ausweich-Schriften — auch in einer per
> `LOADFONT` geladenen). `LOADFONT(pfad$, groesse[, zeichen$])` nimmt
> Blocknamen oder die Zeichen selbst; Schriftsammlungen (`.ttc`) werden
> aufgelöst — raylib tauschte sie bisher still gegen seine Bitmapschrift, und
> genau so liegen die CJK-Schriften von Windows vor. Die Tipp-Warteschlange
> steht auf 256 (`CFLAGS` in `build_runtime.py`, nachgesehen im CMake-Cache
> von raylib-sys). Weg B: das gui meldet die Schreibmarke des Feldes mit Fokus
> an `ImmSetCompositionWindow`/`ImmSetCandidateWindow` (Windows, ohne IME
> nicht messbar). **Gemessen:** `12,50 € Ω Я ő ł`, `日本語 한글 😀 שלום` und
> `東京` erscheinen statt `?`; das erste Bild mit vier neuen Schriften kostet
> ~100 ms, ein weiteres neues Kanji ~15 ms, der Start 50 ms mehr für den
> größeren Grundvorrat (357 → 407 ms). Tests `tests/test_schriften_vorrat.py`
> (Bildvergleich echt gegen `?`, Gegenprobe U+E000 bleibt `?`). Siehe
> [builtins-grafik.md](builtins-grafik.md#umlaute-euro-und-fremde-schriften).
> C (Vorschau im Feld) und D bleiben ungebaut; Arabisch/Hebräisch ohne
> Formung und Rechts-nach-links.

*Untersuchung, keine Umsetzung.* Der vierte und letzte Architekturpunkt der
Lückenliste nach dem sechsten Piloten (nach den
[Fenstern](entwurf-native-fenster.md), dem [Drucken](entwurf-drucken.md) und
der [Barrierefreiheit](entwurf-barrierefreiheit.md)): Wer Japanisch,
Chinesisch, Koreanisch, Griechisch, Russisch oder Arabisch schreibt, tippt
nicht Taste für Taste, sondern über eine **Eingabemethode** (IME) — man
schreibt Silben, das System schlägt Zeichen vor, Enter übernimmt. Und er
muss das Ergebnis **sehen**. Dieses Papier misst, was davon heute in einem
Drachenhauch-Textfeld ankommt und was auf dem Schirm erscheint, prüft die
Bausteine und entwirft vier Wege. Die Entscheidung fällt jemand anders.

Alle Angaben sind geprüft, nicht angenommen — Stand 06.09.2026, diese
Maschine (Windows 11, nur deutsche Tastatur, **keine IME installiert**) und
die Quelltexte von raylib 6.0 und GLFW 3.4. Was hier nicht messbar war,
steht als „aus dem Quelltext“ da.

## 1. Was heute geht — gemessen

**Der Speicher ist sauber, die Anzeige nicht.** Über die Zwischenablage kam
`aä€日本😀` in ein `GUI_TEXTINPUT`; `GUI_TEXT` liefert es unverändert
zurück, `LEN` sagt 6 — ein Zeichen je Zeichen, auch das Emoji jenseits der
Basisebene. Gezeichnet wird daraus:

```text
Textfeld:      aä????
Beschriftung:  aä????
```

Das Euro-Zeichen, die beiden Kanji und das Emoji sind Fragezeichen. Der
Grund steht in `graphics.rs`: jede Schrift — die eingebaute, die
Ausweich-Schrift (Segoe UI / Arial, unter Linux DejaVu) und jede per
`LOADFONT` geladene — wird mit **einem festen Zeichenvorrat** gebacken
(`zeichensatz()`): ASCII, Latin-1 (`0xA0..0xFF`) und 22 typografische
Zeichen (`…–—„“”·•°→←×÷≤≥`). **Nicht dabei: das `€`**, Latin Extended
(`ő ł č ş`), Griechisch, Kyrillisch, Arabisch, Hebräisch, CJK, Emoji. Eine
japanische Schrift laden nützt nichts — es werden trotzdem nur die 246
Zeichen daraus gebacken.

**Der Weg von der Tastatur ins Feld** (aus dem Quelltext; die Messung per
eingespeister Tastatur hat die Antivirensoftware als Eingabe-Injektion
blockiert):

| Stufe | Stand |
|---|---|
| Windows sendet `WM_CHAR` | GLFW setzt **Surrogatpaare zusammen** (`win32_window.c:662`) — ein Emoji kommt als EIN Codepunkt an; tote Tasten (`´` + `e` → `é`) setzt Windows selbst zusammen |
| raylib nimmt es in die Tipp-Warteschlange | **16 Zeichen je Bild** (`MAX_CHAR_PRESSED_QUEUE`, `rcore_desktop_glfw.c:2091`); was darüber hinausgeht, fällt **still** weg |
| gui liest die Warteschlange | `pop_text_input()` — Textfeld, Textbereich, Zelle, `ui`-Modul, `INKEY$` |
| IME-Bestätigung | kommt als `WM_CHAR`-Folge an — also wie getippt. Ein bestätigter japanischer Satz von 20 Zeichen verliert die letzten vier |
| IME-Umwandlungsfenster | GLFW 3.4 enthält **keine Zeile IMM-Code** (`Imm*`: 0 Treffer): das System zeigt sein Fenster dort, wo es will (meist links oben im Fenster oder beim zuletzt bekannten Ort), nicht an der Schreibmarke; keine Vorschau im Feld (Preedit) |
| macOS | GLFW implementiert `NSTextInputClient` (`cocoa_window.m:670`, `setMarkedText`), **wirft den markierten Text aber weg** — Bestätigtes kommt an, die Umwandlung ist unsichtbar |
| Linux | X11 über XIM (GLFW), Wayland ohne `text-input` — nicht geprüft |
| Web | GLFW-Nachbau von emscripten, Zeichen-Ereignisse ja, IME nein |

**Konsole.** `INPUT` liest über Rusts Standardbibliothek; an einer echten
Konsole ist das `ReadConsoleW` (UTF-16, IME-tauglich, weil die Konsole die
Umwandlung selbst macht). Über eine **Pipe** muss es UTF-8 sein — gemessen:
`aä€日` durch die Pipe ergibt `LEN` 4 und die richtigen Zeichen;
cp850-Bytes (`ä` als `0x84`) ergeben eine **leere Zeile ohne Meldung**.

**Was man heute also tun kann:** Latin-1-Text tippen und sehen (`ä ö ü ß é
ñ`), alles andere hineinbekommen (Zwischenablage, Datei, Datenbank) und
richtig speichern — aber nicht anzeigen. Ein Programm für Griechenland,
Russland, Japan, China oder Korea ist damit heute nicht zu schreiben, ein
Programm mit Euro-Preisen zeigt `?`.

## 2. Was „Eingabemethode“ konkret heißt

| Stufe | Wer | Beispiel |
|---|---|---|
| **Sehen** | Schrift mit den Glyphen, Zeichenvorrat beim Backen | `€`, `Ω`, `Я`, `日` |
| **Bestätigtes annehmen** | Warteschlange ohne Verlust | 20 Zeichen aus einer IME-Bestätigung |
| **Umwandlung am richtigen Ort** | Kompositionsfenster an der Schreibmarke | `ImmSetCompositionWindow` |
| **Umwandlung im Feld** (Preedit) | unterstrichener Vorschautext im Textfeld, Kandidatenliste daneben | Wie Notepad, Browser, jedes moderne Programm |
| Rechts-nach-links, Zeichenformung | Arabisch/Hebräisch | eigene Textformung (HarfBuzz-Klasse) — außerhalb dieses Papiers |

Die ersten drei Stufen sind Handwerk, die vierte braucht die
Fensternachrichten der IME, die fünfte ein Textformungs-Werk.

## 3. Bausteine, geprüft

**raylib:** `LoadFontEx(pfad, größe, codepoints, anzahl)` nimmt eine
beliebige Liste von Codepunkten — der feste Vorrat ist unsere Wahl, nicht
raylibs Grenze. Die Glyphen werden in EINE Textur gebacken: 7000 CJK-Zeichen
bei 20 px sind etwa 2048×2048 Pixel, 16 MB — geht, aber nicht für jede
Größe nebenbei. Die Schriften dafür liegen unter Windows bereits da
(`msgothic.ttc`, `msyh.ttc`, `malgun.ttf`, `seguiemj.ttf` — Emoji allerdings
nur als einfarbige Umrisse, stb_truetype kennt keine Farbschriften).
`MAX_CHAR_PRESSED_QUEUE` ist ein `#define` in raylibs `config.h`; raylib-sys
baut über cmake, das `CFLAGS` aus der Umgebung übernimmt — der Wert ließe
sich aus `build_runtime.py` anheben.

**Windows IMM** (`windows`-Crate, Feature `Win32_UI_Input_Ime`):
`ImmGetContext`, `ImmSetCompositionWindow` (Lage des Umwandlungsfensters),
`ImmSetCandidateWindow`, `ImmGetCompositionStringW` mit `GCS_COMPSTR`
(Vorschau) und `GCS_RESULTSTR` (Ergebnis), die Nachrichten
`WM_IME_STARTCOMPOSITION` / `WM_IME_COMPOSITION` / `WM_IME_ENDCOMPOSITION`.
Um sie zu sehen, braucht es die Fensterprozedur — und **die haben wir seit
der Barrierefreiheit schon in der Hand**: AccessKit hängt sich per
Subclassing an raylibs GLFW-Fenster, ein zweiter Subclass für die
IME-Nachrichten ist derselbe Griff (`SetWindowLongPtrW` oder
`SetWindowSubclass`).

**SDL als raylib-Backend:** raylib-sys 6.0 hat das Feature `sdl`
(`PLATFORM=SDL`, SDL2 oder SDL3). SDL liefert Text-Ereignisse auf allen drei
Systemen und kann in SDL3 auch die Vorschau (`SDL_TEXTEDITING`) und die Lage
des Eingabebereichs — raylib nutzt davon nur `SDL_TEXTINPUT`
(`rcore_desktop_sdl.c:1627`), keine Vorschau, keinen Ort. Ein Wechsel des
Backends betrifft aber alles: Bau (SDL-Bibliothek dazu), Tastencodes,
Automation-Wiedergabe, Fenster-Flags, den Web-Bau. Nicht für diese Frage.

**GLFW mit IME:** ein Pull Request dafür liegt seit Jahren offen, in 3.4 ist
er nicht. raylib bringt sein eigenes GLFW mit; einen Patch müssten wir
selbst pflegen.

## 4. Vier Wege

### A. Sehen: der Zeichenvorrat

1. `zeichensatz()` erweitern — mindestens **`€`** und Latin Extended-A/B
   (die Sprachen Mitteleuropas, ~300 Zeichen); Griechisch und Kyrillisch (~400)
   kosten bei 20 px kaum Textur.
2. `LOADFONT` bekommt einen dritten Parameter — welche Zeichen gebacken
   werden:

```text
LOADFONT(pfad$, groesse [, zeichen$])
    zeichen$ = "kyrillisch" | "griechisch" | "japanisch" | "chinesisch"
             | "koreanisch" | "emoji" | "alles was in diesem Text steht …"
```

   Ein Block-Name oder eine Zeichenkette mit genau den Zeichen, die das
   Programm braucht (bei einem Spiel mit festen Texten das Billigste).
3. **Glyphen auf Zuruf:** die Ausweich-Schrift merkt sich Codepunkte, für
   die sie keine Glyphe hat, und backt sich am Ende des Bildes einmal neu
   (Vereinigung aller bisher gesehenen Zeichen; Schriftkette je Block:
   Segoe UI → MS Gothic → Microsoft YaHei → Malgun Gothic → Segoe UI Emoji).
   Ein Programm, das nichts tut, zeigt damit jeden Text, den es bekommt —
   der erste Frame mit einem neuen Zeichen kostet ein Nachbacken (einige
   Millisekunden), danach nichts mehr.
4. Die Tipp-Warteschlange von 16 auf 256 heben (`CFLAGS` beim Bau), damit
   eine IME-Bestätigung nichts verliert.

Aufwand zwei bis drei Tage. Prüfbar hier: Zwischenablage hinein,
Bildschirmfoto heraus, Pixel zählen (wie beim Barrierefreiheits-Test).
**Ohne A sind B und C wertlos** — eine Umwandlung, deren Ergebnis als `?`
erscheint, hilft niemandem.

### B. Das Umwandlungsfenster an die Schreibmarke

Hat ein Textfeld den Fokus, meldet das gui je Bild die Schreibmarke
(`ImmSetCompositionWindow`, `CFS_POINT`; dazu `ImmSetCandidateWindow`). Das
ist das Verhalten älterer Windows-Programme: man tippt, ein kleines Fenster
**neben der Marke** zeigt die Umwandlung, Enter übernimmt, das Ergebnis
kommt als Tastendrücke ins Feld. Ein Tag, Windows. Nicht messbar ohne IME —
eine Japanische IME lässt sich als Windows-Sprachpaket nachinstallieren
(Systemeinstellung, Netz), das Prüfen bliebe Handarbeit.

### C. Die Umwandlung im Feld (Preedit)

Ein eigener Subclass der Fensterprozedur nimmt `WM_IME_COMPOSITION`: die
Vorschau (`GCS_COMPSTR`) landet als `preedit` am Widget und wird
unterstrichen an der Schreibmarke gezeichnet, das Ergebnis
(`GCS_RESULTSTR`) wird direkt eingefügt, und die `WM_CHAR`, die Windows
danach ohnehin noch schickt, werden verschluckt, sonst käme alles doppelt.
Das Systemfenster wird versteckt, die Kandidatenliste bleibt vom System.
Eine Woche, **nur Windows**: macOS bräuchte den markierten Text, den GLFWs
`NSTextInputClient` wegwirft (Patch an GLFW), Linux/Wayland hat in GLFW
kein `text-input`. Prüfbar nur von Hand mit installierter IME.

### D. Backend wechseln (SDL)

Löst auf allen drei Systemen die Bestätigung sauber und mit SDL3 auch die
Vorschau — aber erst nach einem Patch an raylibs SDL-Schicht, und um den
Preis, dass jede Eingabe-, Fenster- und Automation-Zusage neu zu prüfen ist.
Wochen. Verworfen als Antwort auf diese Frage.

## 5. Nebeneinander

| | A Zeichenvorrat | B Fenster an der Marke | C Preedit | D SDL |
|---|---|---|---|---|
| `€`, Griechisch, Kyrillisch sichtbar | **ja** | — | — | — |
| CJK, Emoji sichtbar | **ja** (auf Zuruf) | — | — | — |
| IME-Bestätigung vollständig | **ja** (Warteschlange) | ja | ja | ja |
| Umwandlung am richtigen Ort | nein | **ja** (Systemfenster) | **ja** (im Feld) | ja |
| Vorschau im Feld | nein | nein | **ja** | nur mit Patch |
| Systeme | alle | Windows | Windows | alle |
| Hier prüfbar | ja | nein (keine IME) | nein | nein |
| Aufwand | 2–3 Tage | 1 Tag | 1 Woche | Wochen |

## 6. Empfehlung

**A, dann B. C nur auf Nachfrage von jemandem, der eine IME benutzt.**

A ist keine Eingabemethoden-Arbeit, sondern die Voraussetzung für alles —
und sie trifft weit mehr Leute als die IME: **jeder Preis in Euro** und jeder
Name mit `ł`, `ő` oder `č` wird heute als `?` gezeichnet, und ein
griechisches oder russisches Programm ist unmöglich, obwohl der Speicher
längst Unicode kann. Das ist mit zwei bis drei Tagen zu haben, und der
Prüfstein läuft hier.

B ist billig und macht aus „die IME funktioniert irgendwie“ das gewohnte
Verhalten klassischer Windows-Programme. Dass ich es nicht messen kann, ist
der Grund, es klein zu halten.

C ist eine Woche für eine Sache, die ich nicht prüfen kann und für die es
bisher keinen Nutzer gibt. Sie gehört gebaut, wenn jemand mit japanischer
oder chinesischer Tastatur an Drachenhauch sitzt — der kann sie dann auch
abnehmen.

**Reihenfolge:** A (2–3 Tage, Prüfstein: Bildschirmfoto mit `€ Ω Я 日 😀`
statt `?`) → B (1 Tag) → C bei Bedarf.

## 7. Was ohne Entscheidung schon geht

* Text in jeder Schrift **speichern, laden, vergleichen, ausgeben** — die
  Zeichenketten sind UTF-8, `LEN` zählt Zeichen; nur der Bildschirm bleibt
  bei Latin-1.
* Über eine Pipe muss die Eingabe UTF-8 sein; an der Konsole kümmert sich
  Windows um die IME.
* Wer heute Griechisch oder Kyrillisch braucht, kann sich mit einer
  Bitmap-Schrift (`LOADFONT_IMAGE`) behelfen — dort bestimmt das Bild die
  Zeichen, nicht der Zeichenvorrat.
