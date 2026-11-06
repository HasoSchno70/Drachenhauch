# Modul `osc`

**Open Sound Control** — kurze Nachrichten über UDP, die Musik- und
Medienprogramme untereinander austauschen: TouchOSC und Open Stage Control
auf dem Telefon, Pure Data, Max/MSP, SuperCollider, Sonic Pi, VCV Rack,
Resolume, QLab, Reaper, viele Lichtpulte und Bastelprojekte mit ESP32
(`CNMAT/OSC`). Damit steuert man ein Drachenhauch-Programm vom Telefon aus
oder schickt einem Synthesizer Noten.

```basic
IMPORT "osc"
```

Eine Nachricht ist eine **Adresse** (`/synth/1/freq`) und eine Liste
getypter **Werte**. Drachenhauch übersetzt die Werte selbst: `INTEGER` → `i`
(außerhalb von 32 Bit `h`), `FLOAT` → `f`, `STRING` → `s`, `BOOLEAN` →
`T`/`F`, `BUFFER` → `b` (Blob), `NIL` → `N`. Gelesen werden dazu `d` (double),
`t` (Zeitmarke), `c` (Zeichen), `r` (Farbe), `m` (MIDI) und `I`; Bündel
(`#bundle`) kommen als ihre einzelnen Nachrichten an.

## Empfangen

TouchOSC schickt zum Beispiel `/1/fader1` mit einer Kommazahl 0..1:

```basic
IMPORT "osc"
DIM o AS OSC
o = OSC_OPEN(8000)
WHILE TRUE
    WHILE OSC_NEXT(o)
        IF OSC_MATCH("/1/fader*", OSC_ADDRESS$(o)) THEN
            PRINT OSC_ADDRESS$(o); " = "; OSC_ARG(o, 0)
        END IF
    WEND
    SLEEP(10)
WEND
```

## Senden

```basic
IMPORT "osc"
DIM o AS OSC
o = OSC_OPEN()
OSC_SEND(o, "127.0.0.1", 57120, "/note", 60, 0.8, "piano")
```

An die Broadcast-Adresse (`"255.255.255.255"` oder die des Netzes, etwa
`"192.168.1.255"`) erreicht eine Nachricht alle im Netz.

## Befehle

| Befehl | Was es tut |
|---|---|
| `OSC_OPEN([port [, bind$]])` → OSC | einen Anschluss öffnen, der auf `port` hört (Vorgabe 0 = ein freier — genug zum Senden); `bind$` Vorgabe `"0.0.0.0"` |
| `OSC_CLOSE(osc)` | den Anschluss schließen |
| `OSC_PORT(osc)` → INTEGER | der Port, auf dem er hört |
| `OSC_SEND(osc, host$, port, adresse$, ...)` | eine Nachricht mit beliebig vielen Werten schicken; die Adresse beginnt mit `/` |
| `OSC_NEXT(osc)` → BOOLEAN | die nächste angekommene Nachricht holen; FALSE, wenn keine da ist. Wartet nie |
| `OSC_ADDRESS$(osc)` → STRING | Adresse der geholten Nachricht |
| `OSC_TYPES$(osc)` → STRING | ihre Typkennungen, etwa `"fis"` |
| `OSC_COUNT(osc)` → INTEGER | wie viele Werte sie hat |
| `OSC_ARG(osc, index)` | Wert Nummer `index` (ab 0) — als INTEGER, FLOAT, STRING, BOOLEAN, BUFFER oder NIL |
| `OSC_SENDER$(osc)` → STRING | von welcher Adresse sie kam |
| `OSC_SENDER_PORT(osc)` → INTEGER | von welchem Port — dorthin geht eine Antwort |
| `OSC_MATCH(muster$, adresse$)` → BOOLEAN | passt die Adresse auf ein OSC-Muster? Je Teil: `?` ein Zeichen, `*` beliebig viele, `[1-4]`/`[!a-z]` eines aus einer Menge, `{rot,gruen}` eine der Möglichkeiten |

Ein Paket, das kein OSC ist, wird still übergangen; wartende Nachrichten
werden höchstens 4096 gemerkt (die ältesten fallen weg).

## Grenzen

- Nur UDP (OSC über TCP/SLIP nicht).
- Gesendet werden einzelne Nachrichten, keine Bündel mit Zeitmarke.
