# Drachenhauch 2026.23

*Die Notizen zu dieser Fassung. Ein Tag und fünf Pull Requests nach
2026.22: Structs mit fester Lage, GTK als ganze Oberfläche, Funktionen mit
beliebig vielen Argumenten -- und eine Prüfung, die sich nicht mehr stören
lässt, während man daneben weiterarbeitet.*

## Neu in dieser Fassung

### Structs mit fester Lage: `STRUCT … LAYOUT C`

Bisher war ein Struct für eine C-Bibliothek ein `BUFFER`, in den man mit
`BUFFER_SET_I32(b, 8, …)` an selbst gerechneten Stellen schrieb. Jetzt
beschreibt man ihn einmal, und die Laufzeit rechnet die Lage nach den Regeln
von C:

```basic
STRUCT SYSTEMTIME LAYOUT C
    jahr AS USHORT
    monat AS USHORT
    wochentag AS USHORT
    tag AS USHORT
    stunde AS USHORT
    minute AS USHORT
    sekunde AS USHORT
    millisekunden AS USHORT
END STRUCT

DECLARE SUB GetLocalTime LIB "kernel32" (st AS SYSTEMTIME)

DIM jetzt AS SYSTEMTIME
GetLocalTime(jetzt)
PRINT jetzt.tag; "."; jetzt.monat; "."; jetzt.jahr
```

* Eine Variable dieses Typs ist ein `BUFFER` in passender Größe; `jetzt.tag`
  liest und schreibt an der festen Stelle, auch verschachtelt und mit
  Feldern (`st.punkte[i].x`).
* `PACK n` für gepackte Structs, `SIZEOF(typ)` und `OFFSETOF(typ, feld)`
  stehen schon beim Übersetzen fest.

### GTK und Bibliotheken unter mehreren Namen

* `LIB "libgtk-3-0.dll|libgtk-3.so.0|libgtk-3.0.dylib"` -- mehrere Namen,
  der erste, der sich laden lässt, gilt. So läuft dieselbe Zeile auf allen
  drei Systemen.
* `LIB GTK` nimmt den Namen aus einer `CONST` -- bei zwanzig Funktionen aus
  derselben Bibliothek steht er nur einmal da.
* Unter Windows findet eine Bibliothek, die mit vollem Pfad geladen wird,
  ihre Nachbarn im selben Ordner (GTK bringt rund vierzig DLLs mit).
* Neues Beispiel `207_gtk.dh`: ein GTK-Fenster mit Knopf, Tooltip und
  Rückruf, ganz in Drachenhauch.

### Beliebig viele Argumente: `...`

```basic
DECLARE FUNCTION sprintf LIB "msvcrt|c" (ziel AS BUFFER, format AS TEXT, ...) AS LONG

DIM b AS BUFFER
b = BUFFER_NEW(128)
sprintf(b, "%d Drachen, %.1f Meter, %s", 3, 4.5, "feuerrot")
PRINT TEXT_AUS_ZEIGER$(BUFFER_ZEIGER(b))      ' 3 Drachen, 4.5 Meter, feuerrot
```

`printf` und seine Geschwister, `g_object_set` in GTK: der C-Typ jedes
weiteren Werts ergibt sich aus dem Wert selbst (Zahl, Kommazahl, Text,
`BUFFER`, `NIL`). Jedes System ruft solche Funktionen etwas anders auf --
das übernimmt dhrt.

### Prüfen, während man weiterarbeitet

Die Fenster, die `dhrt test` öffnet, gehen jetzt ohne Fokus auf und lassen
die Maus durch. Wer während eines Laufs tippte, schrieb bisher in das
Fenster eines Prüffalls -- im Code eines Falls stand dann „BSCREEN“ statt
„SCREEN“, und der Fall fiel. Aufnahmen und nachgeschickte Fensternachrichten,
über die die Prüfungen die Fenster bedienen, kommen weiter an.

## Behoben

* Die Schnellkorrektur der IDE (Strg+.) brach ab, wenn sich der Text
  änderte, während die Auswahl offen war. Jetzt gilt ein Angebot nur für
  seinen Text.
* `' --- seriell` als Kommentar im Kopf einer Prüfsammlung galt nie; zwei
  Sammlungen liefen so parallel. Das ist jetzt ein Fehler.

## Unter der Haube

**Zahlen.** Die Befehlsreferenz wächst von 2105 auf **2107** Einträge
(`SIZEOF`, `OFFSETOF`), es bleiben 48 Module; **223** Beispiele (vorher
222). **4832 Fälle in 370 Prüfsammlungen** (vorher 4807 in 368) und
**509 Rust-Testfunktionen** (vorher 503).

* Das Lehrbuch wächst um die neuen Abschnitte auf 535 Seiten (englisch
  528).
* Cranelift kennt keine Aufrufe mit variabler Argumentzahl. dhrt baut für
  jeden solchen Aufruf eine gewöhnliche Signatur nach den Regeln des
  Systems: unter Windows reisen Kommazahlen als Bitmuster in
  Ganzzahl-Plätzen, unter Linux und macOS auf x86-64 setzt ein kleines
  Sprungbrett das Register `al`, auf Apple-ARM landen die weiteren Werte auf
  dem Stapel.
* Unter `dhrt test` liefert `WINDOW_FOCUSED()` TRUE: was am Fokus hängt,
  soll nicht davon abhängen, wohin man gerade geklickt hat.

## Was offen bleibt

* Structs als Wert (nur über einen Zeiger), ein Struct als Feld einer
  Klasse und Felder von Structs; Funktionen mit `va_list`, C++ und COM.
* Das Handbuch (`docs/`) gibt es nur auf Deutsch.
* Ein Tray-Symbol unter macOS und Linux.
* Auf einem echten Mac ist das Paket weiter nicht ausprobiert und nicht
  beglaubigt; der Installer ist nicht signiert.
