# Modul `midi`

Noten von einem angeschlossenen Instrument lesen und welche hinausschicken.

```basic
IMPORT "midi"
```

Drachenhauch bringt einen Tracker, einen Sampler, einen sfxr-Synth, einen
Notenblatt-Editor und Kira-Busse mit — bis hierher konnte kein angeschlossenes
Keyboard etwas davon ansteuern. Genau diese Brücke schließt das Modul.

> **Hardware-Modul.** Es steckt nur in Bauten mit `--hardware`
> (`rust/build_runtime.py --hardware`). Ob dein `dhrt` es hat, sagt
> `dhrt --version` in der zweiten Zeile. Unterbau ist `midir`
> (WinMM / ALSA / CoreMIDI); auf Linux braucht der Bau die ALSA-Header.
>
> **Ausnahme:** `MIDI_NOTE_NAME$` und `MIDI_NOTE_FREQ` rechnen nur um und
> stehen in **jedem** Bau. Wer eine Notenanzeige oder einen Ton aus einer
> Notennummer baut, braucht dafür kein Gerät.

## Anschlüsse finden

| Funktion | Wirkung |
|---|---|
| `MIDI_IN_COUNT()` → INTEGER | Wie viele Eingänge es gibt |
| `MIDI_OUT_COUNT()` → INTEGER | Wie viele Ausgänge es gibt |
| `MIDI_IN_NAME$(i)` → STRING | Name des Eingangs `i` |
| `MIDI_OUT_NAME$(i)` → STRING | Name des Ausgangs `i` |

```basic
DIM i AS INTEGER
FOR i = 0 TO MIDI_IN_COUNT() - 1
    PRINT i; ": "; MIDI_IN_NAME$(i)
NEXT
```

Windows bringt einen Ausgang von Haus aus mit (*Microsoft GS Wavetable
Synth*) — damit lässt sich das Senden ausprobieren, ohne dass ein Gerät
angeschlossen ist.

## Öffnen und schließen

| Funktion | Wirkung |
|---|---|
| `MIDI_IN_OPEN(i)` → `MIDI_IN` | Eingang öffnen |
| `MIDI_OUT_OPEN(i)` → `MIDI_OUT` | Ausgang öffnen |
| `MIDI_IN_CLOSE(h)` / `MIDI_OUT_CLOSE(h)` | Wieder schließen |

Beide Handles sind INTEGER-Indizes (wie `SERIAL_HANDLE`). Nach dem Schließen
ist jeder weitere Zugriff ein Fehler im Klartext, kein stilles Nichts.

## Empfangen

Wie bei `db` und `mqtt` über einen **Cursor**: `MIDI_NEXT` holt die nächste
Nachricht in einen Zwischenspeicher, die übrigen Befehle lesen daraus.

| Funktion | Wirkung |
|---|---|
| `MIDI_NEXT(h)` → BOOLEAN | Nächste Nachricht holen; `FALSE` = nichts da |
| `MIDI_PENDING(h)` → INTEGER | Wie viele noch warten |
| `MIDI_IS_NOTE_ON(h)` / `MIDI_IS_NOTE_OFF(h)` / `MIDI_IS_CC(h)` | Art der Nachricht |
| `MIDI_NOTE(h)` → INTEGER | Notennummer (0..127, 60 = C4) |
| `MIDI_VELOCITY(h)` → INTEGER | Anschlagstärke (0..127) |
| `MIDI_CC_NUMBER(h)` / `MIDI_CC_VALUE(h)` | Regler-Nummer und -Wert |
| `MIDI_CHANNEL(h)` → INTEGER | Kanal **1..16** |
| `MIDI_STATUS(h)` / `MIDI_DATA1(h)` / `MIDI_DATA2(h)` | Die rohen Bytes |

```basic
IMPORT "midi"
IMPORT "audio"

DIM tastatur AS MIDI_IN
tastatur = MIDI_IN_OPEN(0)

SCREEN(320, 200)
WHILE NOT QUITREQUESTED()
    WHILE MIDI_NEXT(tastatur)
        IF MIDI_IS_NOTE_ON(tastatur) THEN
            AUDIO_TONE(MIDI_NOTE_FREQ(MIDI_NOTE(tastatur)), 300, "sine")
            PRINT MIDI_NOTE_NAME$(MIDI_NOTE(tastatur))
        END IF
    WEND
    FLIP()
WEND
```

Die innere `WHILE MIDI_NEXT(...)`-Schleife ist wichtig: pro Bild können
mehrere Nachrichten angekommen sein (ein Akkord sind schon drei).

**Zwei Eigenheiten des Protokolls**, die man kennen muss:

1. **Die meisten Instrumente schicken kein „Note aus".** Sie schicken ein
   *Note an* mit Anschlagstärke 0. `MIDI_IS_NOTE_OFF` fängt beide Formen ab —
   wer stattdessen selbst auf `MIDI_STATUS` prüft, bekommt Töne, die nie
   aufhören.
2. **Kanäle zählen hier ab 1**, wie auf jedem Gerätedisplay. Im Protokoll
   stehen 0..15; die Umrechnung macht das Modul.

Uhr-, Active-Sensing- und SysEx-Nachrichten werden **weggelassen**. Ein
Keyboard schickt davon Dutzende je Sekunde, und keine davon ist eine Note.

Der Puffer fasst **1024 unabgeholte Nachrichten**; läuft er über, fällt die
älteste weg. Wer live spielt, will den aktuellen Anschlag sehen, nicht den von
vor zehn Sekunden.

## Senden

| Funktion | Wirkung |
|---|---|
| `MIDI_NOTE_ON(h, kanal, note, anschlag)` | Note an |
| `MIDI_NOTE_OFF(h, kanal, note)` | Note aus |
| `MIDI_CC(h, kanal, nr, wert)` | Regler (z.B. 7 = Lautstärke) |
| `MIDI_SEND(h, status, d1, d2)` | Rohe Nachricht, für alles Übrige |

```basic
DIM synth AS MIDI_OUT
synth = MIDI_OUT_OPEN(0)
MIDI_NOTE_ON(synth, 1, 60, 100)     ' C4 anschlagen
SLEEP(400)
MIDI_NOTE_OFF(synth, 1, 60)
MIDI_OUT_CLOSE(synth)
```

Note, Anschlag und Reglerwerte müssen zwischen **0 und 127** liegen, Kanäle
zwischen **1 und 16**. Ein größerer Wert setzte im Protokoll das Statusbit und
würde als völlig andere Nachricht gelesen — deshalb wird er abgelehnt, statt
still etwas anderes zu tun. Die Prüfung läuft **vor** dem Gerätezugriff.

## Umrechnen

| Funktion | Wirkung |
|---|---|
| `MIDI_NOTE_NAME$(note)` → STRING | `60` → `"C4"`, leer außerhalb 0..127 |
| `MIDI_NOTE_FREQ(note)` → FLOAT | `69` → `440.0` Hz — die Brücke zu `AUDIO_TONE` |

Die Oktavzählung folgt der verbreiteten Konvention (Note 60 = C4). Manche
Hersteller nennen dieselbe Note C3 — das ist eine Zählweise, kein Fehler. Und
Note 71 heißt hier **H**, nicht B.

## Was geprüft ist

An der Entwicklungsmaschine hängt **kein MIDI-Gerät**, und beim Autor auch
nicht. Das hat den Zuschnitt bestimmt:

* Die **Entschlüsselung** eingehender Nachrichten arbeitet über rohe Bytes,
  nicht über den Gerätetyp — und ist damit vollständig mit erfundenen
  Nachrichten prüfbar. Neun Rust-Tests decken Note an, beide Formen von
  Note aus, Kanäle 1 und 16, Regler, die leere Nachricht und eine zu kurze
  ab. Diese Tests laufen in **jedem** Bau, auch ohne das Feature.
* Das **Senden** läuft echt: Windows bringt einen MIDI-Ausgang mit, im Test
  geht also wirklich ein Dreiklang zum Synthesizer und wieder aus.
* **Auch der ganze Kreis** — senden, durch das Betriebssystem, wieder
  empfangen — läuft im Test, sobald ein **virtueller Loopback-Port**
  vorhanden ist (unter Windows z.B. [loopMIDI](https://www.tobias-erichsen.de/software/loopmidi.html),
  über `winget install TobiasErichsen.loopMIDI`). Ein solcher Port erscheint
  unter demselben Namen als Ein- *und* Ausgang; genau daran erkennen die
  Tests ihn, und ohne ihn überspringen sie sich. **Ein echtes Keyboard ist
  dafür nicht nötig.** Damit sind auch die beiden Zusagen oben belegt und
  nicht mehr nur behauptet: dass ein Note-an mit Anschlag 0 als Note-aus
  ankommt, und dass die Warteschlange bei 1024 deckelt und die **älteste**
  wegwirft (1200 gesendet → 1024 warten, die erste überlebende ist die 176.).

Was damit *nicht* geprüft ist: dass ein bestimmtes **Instrument** sich so
verhält wie das Protokoll es vorsieht. Rückmeldungen von jemandem mit
Hardware bleiben willkommen.

## Was es nicht kann

* **Kein SysEx** — weder senden noch empfangen. Gerätespezifische
  Klangdaten bleiben außen vor.
* **Keine MIDI-Uhr, kein MIDI-Timecode.** Wer zu einer fremden Uhr laufen
  will, findet in `audio` mit `AUDIO_CLOCK_*` eine eigene.
* **Keine MIDI-Dateien** (`.mid`) lesen oder schreiben. Für Musik von der
  Platte gibt es `PLAYMUSIC` (auch `.mod`/`.xm`) und den Tracker.
* **Kein virtueller Anschluss**, den andere Programme sehen.
