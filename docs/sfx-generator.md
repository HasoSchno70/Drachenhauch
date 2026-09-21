# SFX-Generator (`examples/183_sfx_generator.dh`)

Ein Werkzeug für Retro-Soundeffekte im sfxr-Stil, geschrieben in Drachenhauch
auf dem eigenen `gui`-Modul. Es baut seinen Klang mit
[`AUDIO_SFX`](module-audio.md) — also mit genau dem Aufruf, der später im
Spiel steht —, zeigt die Wellenform, spielt sie ab und gibt sie als WAV oder
als fertigen GB-Code heraus. Gegenstück zum Partikel-Editor
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
| Wellenform | Die Kurve des aktuellen Klangs (`AUDIO_SOUND_WAVE`), je Abschnitt der größte Ausschlag. Erreicht sie den Rand, steht **Clipping** daneben |
| Transport | **Abspielen**; **Zufall** würfelt neue Werte *um die jetzigen herum* (Lautstärke und Pan bleiben), dazu eine zufällige Wellenform; **Auto-Play** (an) spielt von selbst, sobald ein Regler sechs Bilder lang zur Ruhe gekommen ist — nicht bei jedem Zwischenwert |
| Werkseinstellungen | Muenze, Laser, Explosion, Powerup, Treffer, Sprung, Blip, Motor — ein Klick setzt alle Regler und die Wellenform |
| Ausgabe | **WAV sichern ...**, **GB-Code kopieren**, **Sichern ...**/**Laden ...** (`.ini`), Kästchen **8 Bit (Lo-Fi)** für die WAV |
| Verlauf | **Zurueck**/**Vor** oder Strg+Z/Strg+Y |
| Statuszeile | Meldung und die Dauer des Klangs (Attack + Sustain + Decay in ms) samt Wellenform |

Die Regler stehen in vier Gruppen; ihre Reihenfolge ist zugleich die der
Argumente von `AUDIO_SFX`:

| Gruppe | Regler |
|---|---|
| Ton | Wellenform (`square`, `saw`, `sine`, `triangle`, `noise`), Frequenz (50..8000 Hz), Pitch-Slide (±8000 Hz/s, negativ = fallend), Lautstaerke |
| Huellkurve | Attack (0..2000 ms), Sustain und Decay (0..4000 ms) |
| SID / Filter | Pulsbreite (0,05..0,95; wirkt auf `square`), PWM-Tiefe, PWM-Tempo, Filter (Grenzfrequenz, 0 = aus), Filter-Fahrt (Hz/s), Resonanz |
| Vibrato / Stereo | Vib-Tiefe, Vib-Tempo, Stereo (Breite, 0 = mono), Pan (L..R) |

Alles geht auch ohne Maus: TAB wechselt das Bedienelement, Leertaste und Enter
lösen aus, die Pfeile verstellen einen Regler.

## Rückgängig

Ein **Zug** am Regler ist **ein** Schritt, nicht sechzig je Sekunde:
aufgezeichnet wird erst, wenn sich ein Bild lang nichts geändert hat und keine
Maustaste mehr gedrückt ist. Eine Werkseinstellung, ein Würfelwurf und eine
geladene `.ini` setzen alle Werte auf einmal und sind auf demselben Weg je ein
Schritt. Gemerkt wird jeweils der volle Stand (16 Regler und die Wellenform),
bis zu 32 Schritte. Ein neuer Zug nach einem Zurück schneidet den Vor-Weg ab.

## Sichern und Laden

**Sichern ...** schreibt alle Regler in eine `.ini`, **Laden ...** holt sie
zurück. Schlüssel ist die **Beschriftung** des Reglers (`Frequenz`,
`Pitch-Slide`, `Resonanz`, …), dazu `wellenform` — die Datei lässt sich also
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

**GB-Code kopieren** legt ein lauffähiges Stück in die Zwischenablage: den
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
Aufnahme: Beschriftungen liegen nicht auf den Knöpfen, ein Zug ist ein
Schritt, Zurück/Vor über Knopf und Tasten, Würfelwurf und Werkseinstellung
sind je ein Schritt, ein neuer Zug schneidet den Vor-Weg ab — und der
GB-Code nimmt den Pan mit und läuft durch `dhrt --check`. Die Rückfrage beim
Schließen prüft `tests/pruef/werkzeug_kreuz.dhtest` mit echten
Fensternachrichten.
