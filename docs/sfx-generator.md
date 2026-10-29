# SFX-Generator (`examples/183_sfx_generator.dh`)

Ein Werkzeug für Retro-Soundeffekte im sfxr-Stil, geschrieben in Drachenhauch
auf dem eigenen `gui`-Modul. Es baut seinen Klang mit
[`AUDIO_SFX`](module-audio.md) — also mit genau dem Aufruf, der später im
Spiel steht —, zeigt die Wellenform, spielt sie ab und gibt sie als WAV oder
als fertigen DH-Code heraus. Gegenstück zum Partikel-Editor
([`particle-editor.md`](particle-editor.md)); Musik macht der
[Tracker](tracker.md).

```
dhrt run examples/183_sfx_generator.dh
```

In der [IDE](ide.md) steht er im Menü **Werkzeuge** und läuft dort als eigenes
Programm. Python braucht er nicht.

## Das Fenster

| Bereich | Was geht |
|---|---|
| Wellenform | Groß in der Mitte: die Hülle des Klangs (`AUDIO_SOUND_WAVE`, je Spalte der größte Ausschlag) über einer Zeitachse in ms, darüber die **Hüllkurve** als gelbe Linie mit drei Punkten (A, S, D) — sie lassen sich mit der Maus ziehen, ein Zug ist ein Schritt. Ein Klick daneben spielt; beim Abspielen läuft ein Strich mit, rechts zeigt ein Balken den Pegel am Ausgang (`AUDIO_PEAK`). Erreicht die Welle den Rand, steht **Übersteuert** daneben |
| Werkzeugleiste und Menü | **Öffnen**/**Sichern** (`.ini`, Strg+O/Strg+S), **WAV** (Strg+E), **DH-Code** (Strg+Umschalt+C), **Abspielen** (Leertaste), **Stopp** (Umschalt+Leertaste), **Zurück**/**Vor** (Strg+Z/Strg+Y); darunter **Auto-Play** (spielt von selbst, sobald ein Regler sechs Bilder lang zur Ruhe gekommen ist) und **WAV mit 8 Bit (Lo-Fi)** |
| Vorlagen | Münze, Laser, Explosion, Powerup, Treffer, Sprung, Blip, Motor, Schuss, Klick, Glocke, Alarm — ein Klick setzt alle Regler und die Wellenform. Jede Vorlage ist zugleich eine **Art** |
| Zufall und Mutieren | **Zufall** (R) würfelt einen neuen Klang der gewählten Art — in den Grenzen, die einen Laser zum Laser machen (Lautstärke und Pan bleiben). **Mutieren** (M) verschiebt jeden Wert, der schon etwas tut, um höchstens 6 % seines Bereichs; die Wellenform bleibt |
| Verlauf | Liste links unten: jeder Schritt mit Namen („Vorlage: Laser“, „Zufall: Laser“, „Mutiert“, „Tonrutsch“, „Hüllkurve“), ein Klick holt ihn zurück; der Vor-Weg ist gedämpft |
| Statusleiste | Meldung, Dauer des Klangs (Anschwellen + Halten + Ausklingen in ms) samt Wellenform, Schritt im Verlauf |

Das Fenster lässt sich frei vergrößern; die Wellenform wächst mit, Vorlagen und
Regler bleiben stehen. Die Schrift ist DejaVu Sans (`LOADFONT("sans")`) statt der
Pixelschrift.

Die Regler stehen in vier Gruppen; ihre Reihenfolge ist zugleich die der
Argumente von `AUDIO_SFX`:

| Gruppe | Regler |
|---|---|
| Ton | Wellenform (`square`, `saw`, `sine`, `triangle`, `noise`), Frequenz (50..8000 Hz), Tonrutsch (±8000 Hz/s, negativ = fallend), Lautstärke |
| Hüllkurve | Anschwellen (Attack, 0..2000 ms), Halten (Sustain) und Ausklingen (Decay, je 0..4000 ms) — auch in der Wellenform zu ziehen |
| SID / Filter | Pulsbreite (0,05..0,95; wirkt auf `square`), PWM-Tiefe, PWM-Tempo, Filter (Grenzfrequenz, 0 = aus), Filter-Fahrt (Hz/s), Resonanz |
| Vibrato / Stereo | Vibrato-Tiefe, Vibrato-Tempo, Stereobreite (0 = mono), Links/Rechts (Pan) |

Alles geht auch ohne Maus: TAB wechselt das Bedienelement, Enter löst aus, die
Pfeile verstellen einen Regler; jeder Regler erklärt sich im Tooltip.

## Rückgängig

Ein **Zug** am Regler ist **ein** Schritt, nicht sechzig je Sekunde:
aufgezeichnet wird erst, wenn sich ein Bild lang nichts geändert hat und keine
Maustaste mehr gedrückt ist. Eine Werkseinstellung, ein Würfelwurf und eine
geladene `.ini` setzen alle Werte auf einmal und sind auf demselben Weg je ein
Schritt. Gemerkt wird jeweils der volle Stand (16 Regler und die Wellenform),
bis zu 32 Schritte. Ein neuer Zug nach einem Zurück schneidet den Vor-Weg ab.

## Sichern und Laden

**Sichern ...** schreibt alle Regler in eine `.ini`, **Laden ...** holt sie
zurück. Schlüssel ist der **Name** des Reglers in der früheren Schreibweise
(`Frequenz`, `Pitch-Slide`, `Lautstaerke`, `Attack`, …, auch wo die Anzeige
jetzt anders heißt — alte Dateien laden weiter), dazu `wellenform` — die Datei lässt sich also
lesen und von Hand ändern. Ein unbekannter Schlüssel wird übergangen statt
gemeldet, und ein Wert außerhalb des Reglerbereichs wird auf den Bereich
geklemmt.

Beim Schließen (Kreuz, Alt+F4 oder ESC) fragt der Generator nach
(Sichern / Verwerfen / Abbrechen), aber nur, wenn die Regler nicht so stehen
wie beim letzten Sichern, Laden oder Wählen einer Werkseinstellung. Wer auf
genau diesen Stand zurückstellt, wird nicht gefragt.

## Ausgabe

**WAV sichern ...** schreibt den Klang mit
`AUDIO_SAVE_WAV` (16 Bit, mit dem Kästchen 8 Bit). Die Datei lädt jedes
Programm mit `LOADSOUND`:

```basic
DIM snd AS SOUND
snd = LOADSOUND("effekt.wav")
PLAYSOUND(snd)
```

**DH-Code kopieren** legt ein lauffähiges Stück in die Zwischenablage: den
`IMPORT`, den `AUDIO_SFX`-Aufruf mit allen 16 Werten und das Abspielen. Der
Klang entsteht dann zur Laufzeit, ein WAV braucht das Spiel nicht. Für die
Werkseinstellung „Muenze“:

```basic
IMPORT "audio"

DIM snd AS SOUND
snd = AUDIO_SFX("square", 900, 600, 0, 40, 160, 0.00, 0, 0.70, 0.00, 0.50, 0.00, 0.0, 0, 0, 0.00)
PLAYSOUND(snd)
```

Steht der Pan-Regler nicht in der Mitte, kommt er mit. Er liegt am
**Wiedergabe-Kanal**, nicht im Klang — `AUDIO_SFX` kennt nur die
Stereo-Breite —, und ohne ihn klänge der kopierte Code anders als die Vorschau.
Statt `PLAYSOUND` steht dann (hier Pan 0,5):

```basic
IMPORT "audio"

DIM snd AS SOUND
snd = AUDIO_SFX("square", 900, 600, 0, 40, 160, 0.00, 0, 0.70, 0.00, 0.50, 0.00, 0.0, 0, 0, 0.00)
DIM ch AS AUDIO_CHANNEL
ch = AUDIO_PLAY(snd)
AUDIO_PAN(ch, 0.50, 1.00)
```

Was die einzelnen Argumente tun, steht bei `AUDIO_SFX` in
[`module-audio.md`](module-audio.md).

## Gegenüber der alten Qt-Fassung

Die frühere Qt-Fassung (`dhsfx`) hatte eine **Preset-Bibliothek** mit
benannten eigenen Klängen in einer gemeinsamen Datei. Hier ist jeder eigene
Klang eine eigene `.ini`, die man ablegt, wo man will.

## Tests

`tests/pruef/werkzeug_sfx.dhtest` fährt echte Mausklicks und Tasten über eine
Aufnahme: kein Bedienelement liegt auf einem anderen oder über dem Rand, ein
Zug ist ein Schritt, Zurück/Vor über Werkzeugleiste und Tasten, Würfelwurf und
Vorlage sind je ein Schritt, ein neuer Zug schneidet den Vor-Weg ab, Zufall
bleibt bei der Art, Mutieren ist ein Schritt, ein Klick in den Verlauf holt
einen Stand zurück, der Endpunkt der Hüllkurve lässt sich ziehen — und der
DH-Code nimmt den Pan mit und läuft durch `dhrt --check`. Die Rückfrage beim
Schließen prüft `tests/pruef/werkzeug_kreuz.dhtest` mit echten
Fensternachrichten.
