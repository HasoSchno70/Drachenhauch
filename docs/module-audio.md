# Modul `audio`

Erweiterte Audio-API (nativ in der Runtime `dhrt` ueber **Kira**/cpal — eigener Audio-Thread, vom Game-Loop entkoppelt). Liefert die typischen Game-Engine-Bausteine: Channels (pro-Sound-Kontrolle), Pause/Resume, Fade-in/out, Stereo-Pan, Music-Position, plus Tone-Generation fuer prozedurale Sounds.

Ergaenzt die Core-Builtins `LOADSOUND` / `PLAYSOUND` aus [Grafik-Built-ins](builtins-grafik.md) — die einfachen Calls reichen fuer "Sound abspielen", `audio` bringt das volle Audio-Mixing-Toolkit.

```basic
IMPORT "audio"
```

## Mixer-Lifecycle

Das Audio-System startet automatisch beim ersten Sound-Aufruf. Die Abtastrate
bestimmt dabei das **Ausgabegeraet** -- seit dem Wechsel auf [Kira](https://github.com/tesselode/kira)
(2026-06-13, davor raylib-Audio) gibt es kein festes Format mehr, das ein
Programm einstellen koennte.

| Funktion | Wirkung |
|---|---|
| `AUDIO_INIT([...])` | stellt sicher, dass das Audio-System laeuft. **Nimmt Argumente entgegen und ignoriert sie** -- sie stammen aus der Mixer-Zeit und sind nur der Vertraeglichkeit halber geduldet |
| `AUDIO_SET_NUM_CHANNELS(n)` | Anzahl gleichzeitiger Kanaele (Vorgabe **16**, hoch fuer Bullet-Hell) |
| `AUDIO_NUM_CHANNELS()` → INTEGER | aktuelle Channel-Anzahl |
| `AUDIO_BUSY_CHANNELS()` → INTEGER | wie viele Channels gerade spielen |

```basic
AUDIO_INIT(48000, 2, 256)         ' niedrige Latenz fuer Action
AUDIO_SET_NUM_CHANNELS(32)        ' 32 Sounds gleichzeitig moeglich
```

## Sound-Playback

`AUDIO_PLAY(sound, [loops, volume, fade_in_ms])` → `AUDIO_CHANNEL`

Spielt einen SOUND ab und liefert einen Channel-Handle zurueck. Mit dem kann man den Sound pausieren, stoppen, in der Lautstaerke aendern usw.

`loops` folgt der pygame-Semantik: `0` spielt einmal (Default), `N` wiederholt N-mal (also N+1 Durchlaeufe), `-1` loopt endlos. `fade_in_ms` blendet von 0 auf `volume` ein; `AUDIO_STOP(ch, fade_out_ms)` blendet entsprechend aus und stoppt am Fade-Ende. Beide Fades laufen nicht-blockierend ueber den Frame-Update (FLIP).

| Funktion | Wirkung |
|---|---|
| `AUDIO_PLAY(s, [loops, vol, fade_in_ms])` → AUDIO_CHANNEL | Sound starten |
| `AUDIO_PAUSE(ch)` | Channel pausieren |
| `AUDIO_RESUME(ch)` | wieder weiter |
| `AUDIO_STOP(ch[, fade_out_ms])` | Channel stoppen, optional ausfaden |
| `AUDIO_IS_PLAYING(ch)` → BOOLEAN | Spielt der Channel gerade? |
| `AUDIO_VOLUME(ch, vol)` | Lautstaerke 0..1 setzen |
| `AUDIO_GET_VOLUME(ch)` → FLOAT | aktuelle Lautstaerke |
| `AUDIO_PAN(ch, left_vol, right_vol)` | Stereo-Pan (beide 0..1) |
| `AUDIO_PITCH(ch, faktor)` | Tonhoehe/Geschwindigkeit: 1.0 = normal, 2.0 = Oktave hoeher, 0.5 = tiefer |

**Pause/Resume global:**

| Funktion | Wirkung |
|---|---|
| `AUDIO_PAUSE_ALL()` | alle Channels |
| `AUDIO_RESUME_ALL()` | alle weiter |
| `AUDIO_STOP_ALL()` | alle stoppen |

## Beispiel: Bullet-Hell-SFX mit Fade-out

```basic
IMPORT "audio"

DIM laser AS SOUND
laser = LOADSOUND("assets/laser.wav")

DIM ch AS AUDIO_CHANNEL
WHILE shooting
    ch = AUDIO_PLAY(laser, 0, 0.7)     ' loop=0, vol=0.7
    SLEEP(120)
WEND

' Beim Spiel-Ende ausfaden:
AUDIO_STOP(ch, 500)                     ' 500 ms fade-out
```

## Stereo-Pan

`AUDIO_PAN(ch, left, right)` setzt die Lautstaerke der zwei Kanaele unabhaengig. So baut man positional-3D-Sound: ein Enemy links auf dem Screen klingt links.

```basic
' Sound links lokalisiert:
ch = AUDIO_PLAY(footstep)
AUDIO_PAN(ch, 1.0, 0.2)                 ' fast nur links

' Berechne Pan aus Position:
DIM pan_left AS FLOAT
DIM pan_right AS FLOAT
pan_left  = 1.0 - (enemy_x / screen_w)
pan_right = enemy_x / screen_w
AUDIO_PAN(ch, pan_left, pan_right)
```

**Pan-Position + automatische Bewegung:** Fuer wandernde Sounds gibt es drei
Komfort-Builtins, die mit einer einzigen Position arbeiten (0 = links,
0.5 = Mitte, 1 = rechts) und nur das Pan anfassen -- das Volume (und damit
laufende Fades) bleibt unberuehrt. Die Bewegung treibt die Runtime selbst
pro Frame (FLIP), nicht-blockierend:

| Funktion | Wirkung |
|---|---|
| `AUDIO_PAN_POS(ch, p)` | Position direkt setzen (beendet eine laufende Animation) |
| `AUDIO_PAN_SLIDE(ch, von, nach, dauer_ms)` | einmalige Wanderung von → nach; bleibt am Ziel stehen |
| `AUDIO_AUTOPAN(ch, periode_s[, tiefe])` | endloses Pendeln links↔rechts (startet links); `tiefe` 0..1 = Auslenkung um die Mitte, `periode_s <= 0` schaltet ab |

```basic
DIM ton AS SOUND
ton = AUDIO_TONE(440, 2000)

DIM ch AS AUDIO_CHANNEL
ch = AUDIO_PLAY(ton, -1, 0.8)           ' endlos loopen

AUDIO_AUTOPAN(ch, 6.0)                  ' pendelt alle 6s links<->rechts
' ... irgendwann:
AUDIO_PAN_SLIDE(ch, 0.0, 1.0, 2000)     ' einmal in 2s nach rechts wandern
AUDIO_AUTOPAN(ch, 0)                    ' Pendeln aus (Position bleibt)
```

Manuelles `AUDIO_PAN`/`AUDIO_PAN_POS` gewinnt: es beendet eine laufende
SLIDE-/AUTOPAN-Animation. Ein erneutes `AUDIO_PLAY`/`AUDIO_STOP` setzt die
Animation ebenfalls zurueck.

## Musik

Die native Runtime hat einen separaten Music-Channel fuer lange Tracks (laedt streaming statt vollstaendig in RAM). Eigenes API:

| Funktion | Wirkung |
|---|---|
| `AUDIO_MUSIC_LOAD(path$)` | Track laden (laeuft noch nicht) |
| `AUDIO_MUSIC_PLAY([loops[, fade_in_ms]])` | start (loops = -1 = endlos = Default; loops = N → N+1 Durchlaeufe) |
| `AUDIO_MUSIC_STOP([fade_out_ms])` | stoppen, optional ausfaden (nicht-blockierend) |
| `AUDIO_MUSIC_PAUSE()` | pausieren |
| `AUDIO_MUSIC_RESUME()` | weiter |
| `AUDIO_MUSIC_VOLUME(vol)` | 0..1 |
| `AUDIO_MUSIC_GET_VOLUME()` → FLOAT | aktuell |
| `AUDIO_MUSIC_PITCH(faktor)` | Musik-Pitch (1.0 = normal; ueberlebt LOAD/QUEUE — Slow-Motion-Effekt) |
| `AUDIO_MUSIC_GET_PITCH()` → FLOAT | aktueller Pitch |
| `AUDIO_MUSIC_POSITION()` → FLOAT | Sekunden seit Start (von Position 0) |
| `AUDIO_MUSIC_SEEK(sekunden)` | an diese Stelle springen (Sekunden ab Anfang) |
| `AUDIO_MUSIC_BUSY()` → BOOLEAN | Spielt gerade? |
| `AUDIO_MUSIC_QUEUE(path$)` | naechster Track, sobald der aktuelle endet |

**Springen (`AUDIO_MUSIC_SEEK`)** — drei Dinge, die man vorher wissen will:

```basic
AUDIO_MUSIC_LOAD("lied.ogg")
AUDIO_MUSIC_PLAY()
AUDIO_MUSIC_SEEK(30.0)             ' ab Sekunde 30 weiter
```

* **Erst spielen, dann springen.** Vor `AUDIO_MUSIC_PLAY()` gibt es noch nichts zu versetzen; der Aufruf meldet dann einen Fehler, statt stillschweigend nichts zu tun (sonst begänne das Stück wieder bei 0, ohne dass es jemand merkt).
* **Es wirkt nicht sofort.** Gemessen dauert es ~0,3–0,4 s, bis `AUDIO_MUSIC_POSITION()` die neue Stelle zeigt — so lange spielt der Stream seinen Vorlauf noch zu Ende. Wer direkt nach dem Sprung die Position abfragt, bekommt die alte.
Demo: [examples/160_musik_seek.dh](../examples/160_musik_seek.dh) — klickbarer Fortschrittsbalken; der rote Strich zeigt die angesprungene Stelle, und man sieht, wie der Balken sie erst einholt.

* **MOD/XM können es nicht** und sagen es auch. Ihre Zeitachse sind Pattern und Zeilen, nicht Sekunden; wie lange bis zu einer Stelle vergeht, hängt an Tempowechseln im Stück selbst. `AUDIO_MUSIC_POSITION()` liefert für Module trotzdem Sekunden — mitzählen geht, anspringen nicht.

**Formate:** `.ogg`, `.mp3`, `.qoa` — **und Tracker-Module `.mod` (ProTracker/Amiga) + `.xm` (FastTracker II)**. Module enthalten ihre eigenen Samples + Pattern-Daten und werden in **Echtzeit gestreamt**: ein eigener Kira-Sound (`ModuleSound` in `audio.rs`) treibt den reinen Rust-Player [`xmrs`/`xmrsplayer`](https://crates.io/crates/xmrs) auf dem Audio-Thread -- 4+ Kanaele, Sample-basiert, mit exaktem Endlos-Loopen. (Bis 2026-06-13 uebernahm das raylib; seit dem Wechsel auf Kira laeuft es ueber den eigenen Player.) Einfach ein `.mod`/`.xm` (z.B. von [modarchive.org](https://modarchive.org)) laden:

```basic
AUDIO_MUSIC_LOAD("song.mod")
AUDIO_MUSIC_PLAY(-1)               ' loopt -- echter Amiga-Sound

' oder per Core-Builtin:
PLAYMUSIC("song.xm", -1, 1.0)
```

`AUDIO_FFT` greift auch bei Modulmusik den laufenden Mix ab — ideal fuer reaktive Visualizer. Demo: [examples/115_modplayer.dh](../examples/115_modplayer.dh) (Modul-Player mit Spektrum + Drag&Drop fuers eigene Modul).

**Crossfade zwischen Tracks:**

```basic
AUDIO_MUSIC_LOAD("music/level1.ogg")
AUDIO_MUSIC_PLAY(-1, 1000)              ' fade-in 1s, loop

' Beim Boss-Eingang: ausfaden lassen, dann wechseln. Der Fade laeuft
' nicht-blockierend im Game-Loop (FLIP treibt ihn) -- ein sofortiges
' AUDIO_MUSIC_LOAD wuerde ihn hart abschneiden.
AUDIO_MUSIC_STOP(800)                   ' fade-out 800ms
WHILE AUDIO_MUSIC_BUSY()
    FLIP()                              ' Frames weiterlaufen lassen
WEND
AUDIO_MUSIC_LOAD("music/boss.ogg")
AUDIO_MUSIC_PLAY(-1, 800)
```

## Tone-Generation

Fuer prozedurale Sounds ohne Audio-Files. Liefert ein `SOUND`-Objekt, das du wie ein normales gemixtes SOUND verwenden kannst (auch mit `PLAYSOUND` aus Core):

| Funktion | Wirkung |
|---|---|
| `AUDIO_TONE(freq_hz, dauer_ms[, waveform$[, vol]])` | Sine/Square/Saw/Triangle-Ton |
| `AUDIO_NOISE(dauer_ms[, vol])` | Weisses Rauschen |
| `AUDIO_NOTE(wf$, freq, dauer_ms, attack_ms, decay_ms, sustain, release_ms, vol[, vib_depth, vib_speed, detune_cents, slide_halbtoene])` | eine gehaltene Note mit echter ADSR-Hüllkurve, Vibrato, Detune-Schicht und Portamento — das Instrument für Tracker und Musik |

**Waveforms** (case-insensitive): `"sine"`, `"square"`, `"saw"`, `"triangle"`, `"noise"`.

```basic
DIM beep AS SOUND
beep = AUDIO_TONE(440, 200)                ' A 440Hz, 200ms, Sine
PLAYSOUND(beep)

DIM laser AS SOUND
laser = AUDIO_TONE(800, 80, "square", 0.5) ' Square-Welle, leise
PLAYSOUND(laser)

DIM explosion AS SOUND
explosion = AUDIO_NOISE(400, 0.8)
PLAYSOUND(explosion)
```

Generierte Sounds haben automatisch ein kurzes Fade-in/out (5 ms) gegen Clicks am Anfang/Ende.

### `AUDIO_SFX` (sfxr-Stil + SID-Charakter)

Der prozedurale Effekt-Synth mit Pitch-Slide, ADSR, Vibrato, Stereo-Breite —
und **SID-Erweiterungen** (Pulsbreite/PWM + resonanter Tiefpass-Sweep). Die
SID-Argumente sind alle optional; weglassen reproduziert exakt den bisherigen
Klang. Am bequemsten baut man `AUDIO_SFX`-Aufrufe im **Audio Studio** (SFX-Tab,
`dhsound`) und kopiert den GB-Code.

```
AUDIO_SFX(waveform$, freq, slide, attack_ms, sustain_ms, decay_ms,
          vib_depth, vib_speed, vol
          [, stereo_width, duty, pwm_depth, pwm_speed,
           flt_cutoff, flt_sweep, flt_res])
```

| Argument | Wirkung |
|---|---|
| `duty` | Pulsbreite der `square`-Welle (0.05..0.95; **0.5 = symmetrisch = wie bisher**). Schmaler = duenner/naeselnder SID-Puls. |
| `pwm_depth` / `pwm_speed` | Pulsbreiten-**Modulation** (PWM): die Pulsbreite pendelt mit `pwm_speed` Hz um `duty` — der typische SID-Schimmer. |
| `flt_cutoff` | Grenzfrequenz des resonanten Tiefpasses in Hz (**0 = aus**). |
| `flt_sweep` | Cutoff-Verlauf in Hz/s ueber die Note (z. B. -8000 = Acid-Sweep nach unten). |
| `flt_res` | Resonanz 0..0.95 — betont die Grenzfrequenz (SID/TB-303-Charakter). |

```basic
' SID-Pulsbass mit PWM + Filter-Sweep:
DIM s AS SOUND
s = AUDIO_SFX("square", 110, 0, 0, 600, 200, 0, 0, 0.8, _
              0.0, 0.25, 0.15, 5.0, 4000, -7000, 0.7)
PLAYSOUND(s)
```

### `AUDIO_NOTE` (eine Note, kein Effekt)

`AUDIO_SFX` kennt drei **Zeiten** (Attack, Sustain, Decay), die zusammen die
Dauer ergeben — für einen Effekt genau richtig, für ein Instrument nicht: dort
wird eine Note **gehalten**, so lange wie der Spieler will, und klingt
danach aus. `AUDIO_NOTE` ist dafür da.

```
AUDIO_NOTE(waveform$, freq, dauer_ms, attack_ms, decay_ms, sustain, release_ms, vol
           [, vib_depth, vib_speed, detune_cents, slide_halbtoene])
```

| Argument | Wirkung |
|---|---|
| `dauer_ms` | die **gehaltene** Zeit (bis zur nächsten Note) |
| `attack_ms` / `decay_ms` | Anstieg auf voll, dann Abfall auf den Sustain-Pegel |
| `sustain` | der Pegel 0..1, der danach **bleibt** — bei `0.0` verstummt die Note nach dem Decay, egal wie lange sie gehalten wird (Klavier); bei `1.0` bleibt sie voll (Orgel) |
| `release_ms` | das Ausklingen — es hängt **hinten an**, der Klang ist also `dauer_ms + release_ms` lang und überlappt die nächste Note wie bei einem echten Synthesizer |
| `vib_depth` / `vib_speed` | Vibrato wie bei `AUDIO_SFX` |
| `detune_cents` | eine zweite, um so viele Cent verstimmte Schicht darunter (Chorus) — aus einer nackten Wellenform wird ein Instrument |
| `slide_halbtoene` | Portamento: die Tonhöhe gleitet über die gehaltene Zeit zum Ziel, in Halbtönen und exponentiell (musikalisch gleichmäßig), nicht in Hz/s wie bei `AUDIO_SFX` |

```basic
' Ein Klavierton: schneller Anstieg, klingt von allein aus
DIM piano AS SOUND
piano = AUDIO_NOTE("triangle", 261.6, 500, 2, 700, 0.0, 200, 0.8, 0, 0, 6)
' Eine Orgel: bleibt, solange gehalten
DIM orgel AS SOUND
orgel = AUDIO_NOTE("square", 261.6, 500, 6, 0, 1.0, 70, 0.6, 0.06, 6, 5)
```

Der Tracker in Drachenhauch (`examples/190_tracker.dh`) baut jede Note so.

## Klänge mischen

Bis hierher war jeder Klang eine Einbahnstraße: bauen, abspielen, sichern.
Zwei Klänge zu **einem** machen ging nicht — und genau das braucht ein
Musikprogramm, das einen Song als WAV abliefern will.

| Funktion | Wirkung |
|---|---|
| `AUDIO_SOUND_NEW(dauer_ms)` → SOUND | Stille der angegebenen Länge, als Leinwand |
| `AUDIO_SOUND_MIX(ziel, quelle, offset_ms[, vol[, pan]])` | die Quelle ab `offset_ms` in das Ziel **addieren** |
| `AUDIO_SOUND_NORMALIZE(sound[, spitze])` → FLOAT | die lauteste Stelle auf `spitze` bringen (Vorgabe 1.0); liefert den Faktor |

Drei Dinge, die man wissen muss:

1. **Es wird nicht geklemmt.** Zwei laute Klänge übereinander ergeben Werte
   über 1.0, und die bleiben stehen — zwischendurch zu klemmen würde jede
   Überlagerung verzerren. Vor dem Abspielen oder Sichern kommt deshalb
   `AUDIO_SOUND_NORMALIZE`.
2. **Was über das Ende hinausragt, fällt weg.** Ein Ausklingen hinter dem
   letzten Takt ist normal, kein Fehler.
3. **`pan` verteilt die Quelle als Mono** (-1 links, 0 Mitte, +1 rechts,
   Equal-Power). Ohne `pan` bleiben ihre eigenen Kanäle, wie sie sind — ein
   Stereo-Klang bleibt stereo, ein Mono-Klang bleibt in der Mitte bei voller
   Lautstärke.

Eine Quelle mit anderer Abtastrate (eine 48-kHz-Datei) wird beim Mischen
umgerechnet. Mischt man in einen Klang, der gerade **spielt**, arbeitet die
Runtime auf einer Kopie — der Audio-Faden darf den Puffer unter sich nicht
wechseln sehen.

```basic
DIM song AS SOUND
song = AUDIO_SOUND_NEW(2000)                       ' 2 s Leinwand
AUDIO_SOUND_MIX(song, AUDIO_TONE(440, 500), 0)     ' A bei 0 ms
AUDIO_SOUND_MIX(song, AUDIO_TONE(554, 500), 500, 0.8, -0.5)   ' Cis bei 500 ms, halblinks
AUDIO_SOUND_NORMALIZE(song)
AUDIO_SAVE_WAV(song, "akkord.wav")
```

## Einen Klang anschauen und sichern

Zwei Funktionen am **SOUND-Handle** — sie gelten damit für jede Klangquelle
(`AUDIO_TONE`, `AUDIO_NOISE`, `AUDIO_SFX`, geladene Dateien), nicht nur für
den Synthesizer.

| Funktion | Wirkung |
|---|---|
| `AUDIO_SOUND_WAVE(sound, anzahl)` → ARRAY OF FLOAT | `anzahl` Punkte der Wellenform, `-1..1` — zum Zeichnen |
| `AUDIO_SAVE_WAV(sound, pfad$[, bits])` | als WAV-Datei sichern (`bits` = 16, Vorgabe, oder 8) |

`AUDIO_SOUND_WAVE` liefert je Abschnitt das Sample mit dem **grössten Betrag**,
mit seinem Vorzeichen — nicht den Mittelwert. Das ist der ganze Kniff: über
44100 Werte auf 600 Bildpunkte gemittelt hebt sich eine Schwingung gegen null
auf, und die Anzeige zeigte einen Strich statt einer Welle.

```basic
DIM s AS SOUND
s = AUDIO_SFX("square", 900, 600, 0, 40, 160, 0, 0, 0.7)

' Wellenform zeichnen
DIM kurve AS ARRAY OF FLOAT
kurve = AUDIO_SOUND_WAVE(s, 300)
DIM i AS INTEGER
FOR i = 0 TO 298
    LINE(i, 100 - INT(kurve[i] * 90), i + 1, 100 - INT(kurve[i + 1] * 90), CYAN)
NEXT

' und als Datei ablegen -- LOADSOUND liest sie direkt wieder
AUDIO_SAVE_WAV(s, "effekt.wav")
```

Geschrieben werden die Frames, **wie sie sind**: die Lautstärke aus
`AUDIO_SFX` steckt bereits darin. Die Abspiel-Lautstärke des letzten
`AUDIO_PLAY` kommt *nicht* obendrauf — sonst landete ein `volume` von `0.7`
als `0.49` in der Datei. Mono bleibt einkanalig; erst wenn sich linker und
rechter Kanal unterscheiden (`stereo_width > 0`), wird die Datei stereo.

> Der **SFX-Generator** in [`examples/183_sfx_generator.dh`](../examples/183_sfx_generator.dh)
> benutzt beide: er zeichnet die Kurve live und sichert das Ergebnis als WAV.
> Er ist selbst in Drachenhauch geschrieben — auf dem `gui`-Modul.

## Sampler (Amiga-Stil): `SAMPLE_*`

Ein geladenes PCM-Sample ueber die **ganze Klaviatur** spielen, indem es
resampled wird -- hoehere Note = schneller abgespielt, genau wie **Paula** auf
dem Amiga (und wie MOD/XM-Tracker es machen). Aus einem einzigen Zupf-/Bass-/
Drum-Sample wird so ein ganzes Instrument.

| Funktion | Wirkung |
|---|---|
| `SAMPLE_LOAD(pfad$)` → SAMPLE | WAV/OGG/QOA laden, auf Mono normalisiert |
| `SAMPLE_PLAY(sample, halbtoene, vol[, dur_ms])` → AUDIO_CHANNEL | bei relativer Tonhoehe abspielen |
| `SAMPLE_SET_LOOP(sample, start, end)` | Loop-Region in Frames (fuer gehaltene Noten) |
| `SAMPLE_LEN(sample)` → FLOAT | Laenge in Sekunden bei Originaltonhoehe |

`halbtoene` ist relativ zur Originaltonhoehe des Samples: `0` = wie aufgenommen,
`12` = eine Oktave hoeher, `-12` = eine Oktave tiefer (auch krumme/Float-Werte).
`dur_ms <= 0` spielt das **ganze Sample einmal** (One-Shot -- ideal fuer Drums,
Hits, Plucks). `dur_ms > 0` baut einen Klang fester Laenge: ist via
`SAMPLE_SET_LOOP` eine Loop-Region gesetzt, wird sie wiederholt (gehaltene
Note), sonst folgt nach dem Sample-Ende Stille.

Resampelte Varianten werden **gecacht** (pro `sample`/`halbtoene`/`dur_ms`),
ein wiederholter Ton ist also guenstig -- gut fuer Tracker-artige Player.
Der Rueckgabewert ist ein `AUDIO_CHANNEL` -- damit gehen `AUDIO_PAN`,
`AUDIO_VOLUME`, `AUDIO_STOP` usw. wie bei `AUDIO_PLAY`.

```basic
IMPORT "audio"
DIM pluck AS SAMPLE
pluck = SAMPLE_LOAD("assets/pluck.wav")     ' Grundton z.B. A3

SAMPLE_PLAY(pluck, 0, 0.8)                  ' Originaltonhoehe
SAMPLE_PLAY(pluck, 12, 0.8)                 ' eine Oktave hoeher
SAMPLE_PLAY(pluck, -5, 0.6)                 ' eine Quarte tiefer

' Gehaltener Ton mit Loop (z.B. Frames 2000..8000 der Quelle):
SAMPLE_SET_LOOP(pluck, 2000, 8000)
SAMPLE_PLAY(pluck, 0, 0.7, 1000)            ' 1 s, Loop-Region gehalten
```

Demo: [examples/116_sampler.dh](../examples/116_sampler.dh) — ein Zupf-Sample
spielt eine Melodie + Bass ueber die ganze Klaviatur (anklickbar).

> **Sample vs. Modul:** `SAMPLE_*` ist die Live-Primitive, um eigene Samples
> tonhoehen-variabel zu triggern (Sequencer, Instrumente, SFX-Varianten). Wer
> ein fertiges Tracker-Stueck will, spielt ein `.mod`/`.xm` ueber `PLAYMUSIC`
> (siehe oben) -- das bringt seine Samples + Patterns selbst mit.

## Paula-Lo-Fi (Amiga-Klang)

`AUDIO_LOFI(an[, bits[, cutoff_hz]])` schaltet einen **Lo-Fi-Modus** fuer alle
danach **synthetisierten** Sounds (`AUDIO_TONE`/`AUDIO_NOISE`/`AUDIO_SFX` +
`SAMPLE_PLAY`) -- der knusprige Amiga/Paula-Charakter:

- **Bit-Crush** auf `bits` Aufloesung (1..16, Default **8** = Amiga).
- **LED-Tiefpass** bei `cutoff_hz` (Default **3300 Hz** -- der beruehmte
  Amiga-500-Filter; `0` = aus).

Die Kette laeuft in der Reihenfolge des echten Amiga: erst 8-bit-Quantisierung
(DAC), dann der analoge Tiefpass. Sie wirkt nur auf **neu gebaute** Sounds --
der Sample-Cache wird beim Umschalten geleert, geladene Dateien (`LOADSOUND`)
und Musik bleiben unberuehrt.

```basic
AUDIO_LOFI(TRUE)                 ' 8-bit + 3.3 kHz -- klassischer Amiga-Klang
AUDIO_LOFI(TRUE, 4)              ' noch crunchiger (4-bit)
AUDIO_LOFI(TRUE, 8, 0.0)        ' 8-bit, Filter aus (roher Bit-Crush)
AUDIO_LOFI(FALSE)                ' wieder Hi-Fi
```

Demo: in [examples/116_sampler.dh](../examples/116_sampler.dh) mit `L`
umschaltbar (A/B-Vergleich Hi-Fi vs. Paula).

## Mixer-Busse (SFX-/Musik-Master)

Dank Kiras Mixer-Tracks laufen alle SFX/Sampler/Synth-Sounds auf einem
**SFX-Bus** und alle Musik auf einem **Musik-Bus** (beide muenden in den
**Master**). So regelst du Effekte und Musik getrennt mit einem Master-Regler,
unabhaengig von den Einzel-Lautstaerken (sie multiplizieren sich im Mixer).

| Funktion | Wirkung |
|---|---|
| `AUDIO_BUS_VOLUME(bus$, vol)` | Master-Lautstaerke eines Busses (0..1) |
| `AUDIO_BUS_GET_VOLUME(bus$)` → FLOAT | aktuelle Bus-Lautstaerke |

`bus$` ist `"sfx"`, `"music"` oder `"master"` (case-insensitive). Typischer
Einsatz: Optionsmenue mit getrennten Reglern.

```basic
AUDIO_BUS_VOLUME("music", 0.4)        ' Musik leiser
AUDIO_BUS_VOLUME("sfx", 0.8)          ' Effekte etwas runter
AUDIO_BUS_VOLUME("master", 0.0)       ' alles stumm (Pause-Menue)
PRINT AUDIO_BUS_GET_VOLUME("music")   ' 0.4
```

Der `AUDIO_FFT`-Tap haengt am Master, erfasst also weiterhin den gesamten Mix.

## Echtzeit-Effekte (Filter / Reverb / Delay)

Jeder Bus (`sfx`, `music`, `master`) hat eine **Echtzeit-Effektkette** auf dem
Audio-Thread — kein Buffer-Bake wie `AUDIO_LOFI`, sondern echte DSP, die auch
laufende/gestreamte Sounds erfasst und live steuerbar ist.

| Funktion | Wirkung |
|---|---|
| `AUDIO_FILTER(bus$, cutoff_hz[, resonance])` | Tiefpass. `cutoff_hz` 20..20000 (≤0 oder ≥20000 = offen/aus), `resonance` 0..1 (Betonung am Cutoff — der „weeoow"-SID/Acid-Charakter) |
| `AUDIO_REVERB(bus$, mix[, feedback[, damping]])` | Hall. `mix` 0..1 (0 = aus), `feedback` 0..1 (Nachhall-Laenge, Default 0.9), `damping` 0..1 (Hoehen-Daempfung, Default 0.1) |
| `AUDIO_DELAY(bus$, mix[, feedback[, time_ms]])` | Echo. `mix` 0..1 (0 = aus), `feedback` 0..0.95 (Abfall pro Wiederholung, Default 0.5), `time_ms` Echo-Zeit 1..4000 (zur Laufzeit aenderbar, Default 300; weglassen = unveraendert) |
| `AUDIO_DISTORTION(bus$, amount[, mix])` | Overdrive/Fuzz. `amount` 0..1 (0 = aus, → 0..36 dB Drive), `mix` 0..1 (Default 1.0) |
| `AUDIO_COMPRESSOR(bus$, threshold_db, ratio[, makeup_db])` | Dynamik-Kompressor (Glue/Pump). `threshold_db` typ. −24..0, `ratio` ≥ 1 (1 = aus), `makeup_db` Pegel-Anhebung danach |
| `AUDIO_EQ(bus$, freq_hz, gain_db[, q])` | parametrischer Glocken-EQ (eine Band). `gain_db` 0 = transparent, >0 anheben / <0 absenken, `q` Bandbreite (Default 1) |

Signalfluss je Bus: EQ → Filter → Distortion → Compressor → Reverb → Delay.
Alle Effekte starten neutral (kein Klang-Einfluss); ein Mix/Cutoff aktiviert sie.
Parameter wirken sofort und sind animierbar (z.B. Filter-Sweeps).

```basic
' Cave-Level: Musik dumpf + Hall:
AUDIO_FILTER("music", 1200, 0.3)       ' Tiefpass, leicht resonant
AUDIO_REVERB("master", 0.5, 0.9, 0.2)  ' Hall ueber alles

' Acid-Sweep auf der Musik (im Game-Loop):
cutoff = 200 + 6000 * (0.5 + 0.5 * SIN(MILLIS() / 400.0))
AUDIO_FILTER("music", cutoff, 0.8)

' Echo nur auf Effekten:
AUDIO_DELAY("sfx", 0.4, 0.55)

' Alles aus:
AUDIO_FILTER("music", 0, 0) : AUDIO_REVERB("master", 0.0) : AUDIO_DELAY("sfx", 0.0)

' Fuzz-Bass + Mastering-Glue + Bass-Boost:
AUDIO_DISTORTION("sfx", 0.5)             ' Overdrive auf Effekten
AUDIO_COMPRESSOR("master", -18, 4, 3)   ' Kompressor auf der Summe
AUDIO_EQ("music", 100, 6, 1.0)          ' +6 dB Glocke bei 100 Hz
```

Demo: [examples/117_audiofx.dh](../examples/117_audiofx.dh) — Filter-Cutoff per
Maus, Reverb/Delay per Taste, mit Live-Spektrum.

**Gesamt-Showcase:** [examples/118_audio_studio.dh](../examples/118_audio_studio.dh)
— „Audio-Studio", das die ganze Pipeline auf einen Schirm bringt: Modul-Streaming
(Musik-Bus) + Sampler-Arpeggio (SFX-Bus), getrennt stummschaltbar, Master-Filter
per Maus, Reverb/Delay/Distortion/Lo-Fi schaltbar, dauerhafter Mastering-
Kompressor + Bass-EQ, Live-Spektrum des fertigen Mix.

## Uhr: Takt für Musik und Rhythmus

Eine Uhr zählt Ticks auf dem **Audio-Thread** — nicht im Spiel-Takt. Damit lassen
sich Klänge auf den Schlag genau planen, auch wenn die Bildrate einbricht. Das
ist der Unterschied zum `timer`-Modul, das pro Bild abgearbeitet wird.

| Funktion | Rückgabe | Bedeutung |
|---|---|---|
| `AUDIO_CLOCK_NEW(ticks_pro_sekunde)` | AUDIO_CLOCK | Uhr anlegen — **sie startet angehalten** |
| `AUDIO_CLOCK_START(clock)` | — | laufen lassen (auch nach `PAUSE` wieder) |
| `AUDIO_CLOCK_PAUSE(clock)` | — | anhalten; der Tick-Zähler bleibt stehen |
| `AUDIO_CLOCK_STOP(clock)` | — | anhalten und den Tick-Zähler auf 0 setzen |
| `AUDIO_CLOCK_TICKING(clock)` | BOOLEAN | läuft sie gerade? |
| `AUDIO_CLOCK_TICKS(clock)` | INTEGER | wie viele Ticks sind vergangen? |
| `AUDIO_CLOCK_SET_SPEED(clock, ticks_pro_sekunde)` | — | Tempo im Lauf ändern |
| `AUDIO_CLOCK_REMOVE(clock)` | — | Uhr abbauen |

Die Umrechnung von BPM macht man selbst — `bpm / 60 * unterteilungen`:

```basic
IMPORT "audio"
DIM takt AS AUDIO_CLOCK
takt = AUDIO_CLOCK_NEW(120.0 / 60.0 * 4.0)   ' 120 BPM in Sechzehnteln
AUDIO_CLOCK_START(takt)
AUDIO_PLAY_AT(schlag, takt, 8)               ' klingt beim 8. Tick
```

`AUDIO_PLAY_AT` plant den Start; ab da kümmert sich der Audio-Thread darum. Es
gibt **kein** `UPDATE`, das man pro Bild rufen müsste.

## Räumliches Hören: Listener und Emitter

Ein Listener ist das Ohr der Szene (meist die Kamera), ein Emitter eine
Klangquelle im Raum. Lautstärke und Panning rechnet die Audio-Schicht selbst.

| Funktion | Rückgabe | Bedeutung |
|---|---|---|
| `AUDIO_LISTENER_NEW(x, y, z)` | AUDIO_LISTENER | das Ohr setzen; ungedreht blickt es entlang **-Z** |
| `AUDIO_LISTENER_SET_POSITION(listener, x, y, z)` | — | Ohr bewegen (etwa der Kamera nach) |
| `AUDIO_LISTENER_SET_ORIENTATION(listener, yaw_grad)` | — | Blickrichtung um die Y-Achse, **in Grad** |
| `AUDIO_LISTENER_REMOVE(listener)` | — | Listener abbauen |
| `AUDIO_EMITTER_NEW(listener, x, y, z [, min_dist [, max_dist]])` | AUDIO_EMITTER | Klangquelle im Raum; bis `min_dist` voll laut, ab `max_dist` lautlos |
| `AUDIO_EMITTER_SET_POSITION(emitter, x, y, z)` | — | Quelle bewegen |
| `AUDIO_EMITTER_REMOVE(emitter)` | — | Quelle abbauen |

Abgespielt wird mit `AUDIO_PLAY_ON(sound, emitter, ...)`. Der zurückgegebene
Kanal lässt sich danach wie jeder andere behandeln (`AUDIO_PAUSE`, `AUDIO_STOP`,
`AUDIO_VOLUME` …).

Die Orientierung ist bewusst auf **Yaw** beschränkt — die Drehung um die
Hochachse. Das deckt Vogelperspektive und Verfolgerkamera ab, ohne dass man sich
mit Quaternionen befassen muss.

## Zustand sichern: AUDIO_PUSH / AUDIO_POP

Alle Bus-Einstellungen (Lautstärke, Balance, Filter, Hall, Echo, Verzerrer,
Kompressor, EQ) hängen global. Eine vergessene Rücknahme fällt erst Szenen
später auf — dafür gibt es einen Stapel, wie `GFX_PUSH`/`GFX_POP` bei der Grafik.

| Funktion | Rückgabe | Bedeutung |
|---|---|---|
| `AUDIO_PUSH()` | — | alle Bus-Einstellungen auf den Stapel legen |
| `AUDIO_POP()` | — | sie zurückholen; **ohne vorheriges `PUSH` ist das ein Fehler** |
| `AUDIO_DEPTH()` | INTEGER | wie tief ist der Stapel gerade? |

Ein `AUDIO_POP` löst dabei eine laufende `AUDIO_MODULATE`-Bindung: es schreibt
denselben Wert zurück, den der Modulator gerade steuert.

## Externer Typ

| Typ | Wirkung |
|---|---|
| `AUDIO_CHANNEL` | Handle auf einen Mixer-Channel (Returnwert von `AUDIO_PLAY`/`SAMPLE_PLAY`) |
| `SAMPLE` | Handle auf ein geladenes PCM-Sample (Returnwert von `SAMPLE_LOAD`) |

`SOUND` kommt aus Core — `LOADSOUND` und `AUDIO_TONE`/`AUDIO_NOISE` liefern beide ein SOUND. Untereinander austauschbar.

## Sound-Lebensdauer (`UNLOADSOUND` / `AUDIO_SOUND_COUNT`)

`AUDIO_TONE`/`AUDIO_NOISE`/`AUDIO_SFX` bauen bei **jedem** Aufruf einen neuen
`SOUND`-Puffer — ein frame-basierter Song-Player, der pro Note einen Ton
synthetisiert, sammelt so über die Zeit Buffer an. Mit **`UNLOADSOUND(s)`**
gibst du einen nicht mehr gebrauchten Sound frei (stoppt die laufende Instanz und
gibt den Frame-Puffer frei). Der Handle bleibt als Tombstone gültig — er wird nie
recycelt, ein alter Handle aliased also nie einen neuen Sound — aber ein erneutes
`PLAYSOUND`/`AUDIO_PLAY` auf einen freigegebenen Handle wirft eine klare Meldung.
**`AUDIO_SOUND_COUNT()`** liefert die Anzahl lebender (nicht freigegebener)
Slots — praktisch, um Sound-Lecks aufzuspüren.

```basic
DIM t AS SOUND
t = AUDIO_TONE(440, 100, "square", 0.5)
PLAYSOUND(t)
' ... wenn der Ton fertig ist und nicht wieder gebraucht wird:
UNLOADSOUND(t)
PRINT AUDIO_SOUND_COUNT()        ' lebende Slots (Diagnose)
```

Geladene Dateien (`LOADSOUND`) kannst du genauso freigeben; meist hält man die
aber gecacht. Faustregel: nur **pro Note frisch synthetisierte** Einweg-Sounds
nach Gebrauch entladen.

## Typische Game-Patterns

**SFX-Manager mit Lautstaerke-Master:**

```basic
DIM sfx_volume AS FLOAT
sfx_volume = 0.7

SUB PlaySfx(s AS SOUND)
    AUDIO_PLAY(s, 0, sfx_volume)
END SUB
```

**Pitch-Variation gegen Sound-Leiern** (derselbe Schuss klingt 100x anders):

```basic
DIM ch AS AUDIO_CHANNEL
ch = AUDIO_PLAY(laser, 0, 0.7)
AUDIO_PITCH(ch, 0.9 + RANDF() * 0.2)    ' +-10% Zufalls-Pitch
```

**Sound-Cooldown gegen Spam:**

```basic
DIM last_hit_ms AS INTEGER
last_hit_ms = 0

IF MILLIS() - last_hit_ms > 100 THEN
    AUDIO_PLAY(hit_sound)
    last_hit_ms = MILLIS()
END IF
```

**Music-Position fuer rhythmische Spiele:**

```basic
' Beat-Sync: jeder 60. Beat-Frame
DIM beat_at AS FLOAT
beat_at = AUDIO_MUSIC_POSITION()
IF (beat_at MOD 0.5) < 0.05 THEN          ' alle 500ms
    BeatHit()
END IF
```

## Beispiele

[examples/68_audio.dh](../examples/68_audio.dh) demonstriert das volle Modul-API inklusive Tone-Generation, Pan, Music-Queue.

[examples/114_chiptune.dh](../examples/114_chiptune.dh) — **4-Kanal-Chiptune-Demo im C64/Amiga-Stil**: ein komplettes Musikstueck ohne Audio-Dateien. Lead (Square + Vibrato via `AUDIO_SFX`, rechts gepannt), Akkord-Arpeggio (links), Square-Bass und Drums (Kick = Sinus-Pitch-Drop, Snare/HiHat = `AUDIO_NOISE`) laufen parallel auf dem Mixer; ein frame-basierter Pattern-Player (wie der dhtracker-Export) spielt alle 125 ms eine Reihe. Dazu VU-Meter pro Kanal, echtes `AUDIO_FFT`-Spektrum und Sinus-Scroller.

[examples/115_modplayer.dh](../examples/115_modplayer.dh) — **Amiga-Modul-Player**: spielt ProTracker-`.mod`/`.xm` direkt (`PLAYMUSIC`/`AUDIO_MUSIC_*`), mit echtem Spektrum (`AUDIO_FFT`) und Drag&Drop fuers eigene Modul. Liefert ein selbst generiertes, gemeinfreies Demo-Modul mit (`examples/assets/demo.mod`, Generator `examples/assets/make_demo_mod.py`).

[examples/190_tracker.dh](../examples/190_tracker.dh) — **der Tracker in Drachenhauch**: Noten aus `AUDIO_NOTE`, Wiedergabe über `AUDIO_CLOCK`/`AUDIO_PLAY_AT` (zwei Reihen voraus geplant, gestoppt wird durch Entfernen der Uhr), WAV-Export über `AUDIO_SOUND_NEW/MIX/NORMALIZE`.

[examples/116_sampler.dh](../examples/116_sampler.dh) — **Amiga-Stil-Sampler**: ein einziges Zupf-Sample (`SAMPLE_LOAD`) wird per `SAMPLE_PLAY` ueber die ganze Klaviatur gespielt (Resampling = Tonhoehe wie Paula). Auto-Melodie + Bass aus demselben Sample, anklickbare Tasten, `L` schaltet den Paula-Lo-Fi-Modus zu.

## In der nativen Runtime (dhrt)

Das `audio`-Modul laeuft nativ ueber **Kira** (cpal) — ein eigener Audio-Thread, vom Game-Loop entkoppelt (loeste 2026-06-13 raylib-Audio ab; mit dem `graphics`-Feature eingebunden, raylib bleibt fuer Fenster/Input). Audio-Ausgabe gehoert **nicht** zur deterministischen bit-identischen Garantie — wie `RND`/`tween`. Hinweise:

- `SOUND` und `AUDIO_CHANNEL` sind ganzzahlige Handles; ein „Channel“ ist die zuletzt gestartete Instanz eines geladenen/gebauten Sounds.
- Fade-in/out, `AUDIO_STOP` mit Fade und `AUDIO_PAN_SLIDE` sind **native Kira-Tweens** (laufen auf dem Audio-Thread); `loops` via `loop_region` (endlos) bzw. Restart-Zaehlung (endlich). `AUDIO_FFT` zapft den Mixer-Haupttrack ueber einen Effect an.
- Volume wird intern in Dezibel gefuehrt (Kira), die Builtins nehmen weiterhin linear 0..1.
- Ton-Generierung (`AUDIO_TONE`/`AUDIO_NOISE`/`AUDIO_SFX`) und der Sampler bauen die Wellenform als Float-Buffer direkt als Kira-`StaticSoundData`.
- **Tracker-Module** (`.mod`/`.xm`) als Musik werden in **Echtzeit gestreamt** (eigener Kira-Custom-Sound, der den reinen Rust-Player `xmrs` auf dem Audio-Thread pollt): sofort geladen (kein Vorab-Render), exaktes Endlos-Loopen, wenig RAM, mit Pitch-Resampler + klickfreien Volume-Fades. Stream-Formate (ogg/mp3/wav/flac) streamen von Platte.
