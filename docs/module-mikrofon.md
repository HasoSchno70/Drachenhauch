# Mikrofon (`MIC_*`)

Was am Eingang des Rechners ankommt: wie laut es ist, welcher Ton gerade
klingt, das Spektrum als Balken, die Wellenform zum Zeichnen — und eine
Aufnahme als Klang oder WAV-Datei. Damit gehen ein Stimmgerät, ein Pegelmesser,
ein Spiel, das man mit der Stimme steuert, oder ein Klatschschalter.

```basic
IMPORT "audio"
```

Die Befehle gehören zum Modul `audio`. Gelesen wird über dieselbe Schicht, über
die auch der Ton hinausgeht (cpal): unter Windows WASAPI, unter macOS
CoreAudio, unter Linux ALSA (auch PulseAudio und PipeWire über ihre
ALSA-Brücke). Es läuft **ein** Eingang zur Zeit; er wird nach dem
Standard-Format des Geräts geöffnet und zu **einem** Kanal gemischt.

## Erste Schritte

```basic
IMPORT "audio"
SCREEN(400, 120, "Pegel")
MIC_START()
WHILE NOT QUITREQUESTED()
    CLS(&H101018)
    DIM p AS FLOAT : p = MIC_LEVEL()
    BOX(20, 40, 20 + INT(p * 360), 80, IIF(p > 0.9, &HFF4040, &H40D060))
    TEXT(20, 90, MIC_DEVICE$())
    FLIP()
WEND
```

## Geräte

| Befehl | Was es tut |
|---|---|
| `MIC_DEVICES()` → ARRAY OF STRING | die Namen aller Eingänge, der Standard zuerst; leer, wenn der Rechner keinen hat |
| `MIC_START([geraet$])` | einen Eingang öffnen — ohne Namen den Standard. Ein Name passt genau oder als Teiltext ohne Groß/klein (`"USB"` findet „Mikrofon (USB Audio Device)"); ein unbekannter ist ein Fehler, der die vorhandenen nennt. Ein laufender Eingang wird vorher geschlossen |
| `MIC_STOP()` | den Eingang schließen; eine laufende Aufnahme fällt weg |
| `MIC_ACTIVE()` → BOOLEAN | läuft ein Eingang (und hat er keinen Fehler gemeldet)? |
| `MIC_ERROR$()` → STRING | der Fehler, den das Gerät gemeldet hat (abgezogen, verweigert) — sonst `""` |
| `MIC_DEVICE$()` → STRING | Name des offenen Eingangs, sonst `""` |
| `MIC_RATE()` → INTEGER | Abtastrate des offenen Eingangs (meist 44100 oder 48000), sonst 0 |
| `MIC_SAMPLES()` → INTEGER | wie viele Werte seit `MIC_START` ankamen — so sieht man, ob überhaupt etwas kommt |

Alles weiter unten verlangt einen laufenden Eingang; ohne ist es ein Fehler
— eine stille Null wäre von „es ist leise" nicht zu unterscheiden.

## Pegel und Tonhöhe

| Befehl | Was es tut |
|---|---|
| `MIC_LEVEL()` → FLOAT | höchster Ausschlag seit dem letzten Aufruf, 0 bis 1 — einmal je Bild gefragt ist das eine Pegelanzeige, die keinen Knacks verpasst |
| `MIC_RMS()` → FLOAT | Effektivwert der letzten 50 ms — das, was man als Lautstärke hört (ein Sinus mit Ausschlag 1 hat 0,707) |
| `MIC_PITCH()` → FLOAT | Grundton in Hz, 50 bis 2000; 0, wenn es leise ist oder kein Ton heraussticht (Rauschen, Sprache zwischen zwei Silben) |
| `MIC_FFT(baender)` | füllt ein `ARRAY OF FLOAT` mit Bandpegeln 0..1, logarithmisch verteilt, mit Nachleuchten und eigener Verstärkung — dieselbe Rechnung wie `AUDIO_FFT`, aber vom Eingang |
| `MIC_WAVE(werte)` | füllt ein `ARRAY OF FLOAT` mit den jüngsten Werten (-1..1), älteste zuerst; höchstens 8192 |

`MIC_PITCH` rechnet nach dem YIN-Verfahren und trifft auch Töne mit vielen
Obertönen (Gitarre, Stimme, Sägezahn) auf ihren Grundton. Aus der Frequenz
wird ein Notenname mit `MIDI_NOTE_NAME$`:

```basic
IMPORT "audio"
IMPORT "midi"
MIC_START()
WHILE TRUE
    DIM f AS FLOAT : f = MIC_PITCH()
    IF f > 0 THEN
        DIM n AS FLOAT : n = 69 + 12 * LOG(f / 440) / LOG(2)
        PRINT MIDI_NOTE_NAME$(ROUND(n)); "  "; FORMAT$(f, "%.1f"); " Hz"
    END IF
    SLEEP(100)
WEND
```

## Aufnahme

| Befehl | Was es tut |
|---|---|
| `MIC_RECORD_START([max_sekunden])` | ab jetzt mitschreiben, höchstens `max_sekunden` (Vorgabe 600, höchstens 3600); eine laufende Aufnahme beginnt von vorn |
| `MIC_RECORDING()` → BOOLEAN | läuft eine Aufnahme? |
| `MIC_RECORD_STOP()` → SOUND | die Aufnahme beenden und als Klang liefern — abspielen, mischen, mit `AUDIO_SAVE_WAV` sichern. Braucht die Ton-Ausgabe |
| `MIC_RECORD_SAVE(pfad$)` → INTEGER | die Aufnahme beenden und als WAV (16 Bit, ein Kanal) sichern; liefert die Zahl der Werte. Geht auch ohne Ton-Ausgabe |

```basic
IMPORT "audio"
MIC_START()
MIC_RECORD_START(5)
SLEEP(5000)
DIM s AS SOUND
s = MIC_RECORD_STOP()
PLAYSOUND(s)
```

## Berechtigungen

- **Windows**: *Einstellungen → Datenschutz und Sicherheit → Mikrofon* — der
  Zugriff für Desktop-Apps muss an sein. Ist er aus, bleibt der Eingang
  stumm.
- **macOS** fragt beim ersten `MIC_START`. Die Drachenhauch-App bringt die
  Begründung und die Berechtigung dafür mit; wer `dhrt` im Terminal startet,
  bekommt die Frage für das Terminal.
- **Linux**: der Benutzer braucht Zugriff auf `/dev/snd` (Gruppe `audio`,
  auf den üblichen Systemen von Anfang an).

## Prüfen ohne Mikrofon

`DHRT_MIKROFON=datei.wav` macht aus einer WAV-Datei ein Mikrofon: sie ist
dann das einzige Gerät (`Datei: name.wav`) und wird in Echtzeit und in
Schleife eingespeist — über denselben Weg wie die Werte eines echten Geräts.
So prüft `tests/pruef/mikrofon.dhtest` Pegel, Tonhöhe, Spektrum, Wellenform
und Aufnahme auch auf Rechnern ohne Eingang.

## Grenzen

- Ein Eingang zur Zeit, ein Kanal (Stereo wird gemittelt).
- Im Browser gibt es noch kein Mikrofon.
- Die Abtastrate ist die des Geräts; umgerechnet wird nicht.
