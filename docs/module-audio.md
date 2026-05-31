# Modul `audio`

Erweiterte Audio-API ueber `pygame.mixer`. Liefert die typischen Game-Engine-Bausteine: Channels (pro-Sound-Kontrolle), Pause/Resume, Fade-in/out, Stereo-Pan, Music-Position, plus Tone-Generation fuer prozedurale Sounds.

Ergaenzt die Core-Builtins `LOADSOUND` / `PLAYSOUND` aus [Grafik-Built-ins](builtins-grafik.md) — die einfachen Calls reichen fuer "Sound abspielen", `audio` bringt das volle Audio-Mixing-Toolkit.

```basic
IMPORT "audio"
```

## Mixer-Lifecycle

Pygame.mixer initialisiert sich automatisch beim ersten Sound-Call mit Defaults (44100 Hz, 16-bit, Stereo, 512-buffer). Wer das Audio-Format anpassen will:

| Funktion | Wirkung |
|---|---|
| `AUDIO_INIT([freq[, channels[, buffer]]])` | Mixer neu starten (z.B. mit 48000 Hz oder Mono) |
| `AUDIO_SET_NUM_CHANNELS(n)` | Anzahl Mixer-Channels (Default: 8, hoch fuer Bullet-Hell) |
| `AUDIO_NUM_CHANNELS()` → INTEGER | aktuelle Channel-Anzahl |
| `AUDIO_BUSY_CHANNELS()` → INTEGER | wie viele Channels gerade spielen |

```basic
AUDIO_INIT(48000, 2, 256)         ' niedrige Latenz fuer Action
AUDIO_SET_NUM_CHANNELS(32)        ' 32 Sounds gleichzeitig moeglich
```

## Sound-Playback

`AUDIO_PLAY(sound, [loops, volume, fade_in_ms])` → `AUDIO_CHANNEL`

Spielt einen SOUND ab und liefert einen Channel-Handle zurueck. Mit dem kann man den Sound pausieren, stoppen, in der Lautstaerke aendern usw.

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

## Musik

Pygame hat einen separaten Music-Channel fuer lange Tracks (laedt streaming statt vollstaendig in RAM). Eigenes API:

| Funktion | Wirkung |
|---|---|
| `AUDIO_MUSIC_LOAD(path$)` | Track laden (laeuft noch nicht) |
| `AUDIO_MUSIC_PLAY([loops[, fade_in_ms]])` | start (loops = -1 = endlos) |
| `AUDIO_MUSIC_STOP([fade_out_ms])` | stoppen, optional fade |
| `AUDIO_MUSIC_PAUSE()` | pausieren |
| `AUDIO_MUSIC_RESUME()` | weiter |
| `AUDIO_MUSIC_VOLUME(vol)` | 0..1 |
| `AUDIO_MUSIC_GET_VOLUME()` → FLOAT | aktuell |
| `AUDIO_MUSIC_POSITION()` → FLOAT | Sekunden seit Start (von Position 0) |
| `AUDIO_MUSIC_BUSY()` → BOOLEAN | Spielt gerade? |
| `AUDIO_MUSIC_QUEUE(path$)` | naechster Track nach Ende (auto-fade) |

**Crossfade zwischen Tracks:**

```basic
AUDIO_MUSIC_LOAD("music/level1.ogg")
AUDIO_MUSIC_PLAY(-1, 1000)              ' fade-in 1s, loop

' Beim Boss-Eingang:
AUDIO_MUSIC_STOP(800)                   ' fade-out 800ms
AUDIO_MUSIC_LOAD("music/boss.ogg")
AUDIO_MUSIC_PLAY(-1, 800)
```

## Tone-Generation

Fuer prozedurale Sounds ohne Audio-Files. Liefert ein `SOUND`-Objekt, das du wie ein normales gemixtes SOUND verwenden kannst (auch mit `PLAYSOUND` aus Core):

| Funktion | Wirkung |
|---|---|
| `AUDIO_TONE(freq_hz, dauer_ms[, waveform$[, vol]])` | Sine/Square/Saw/Triangle-Ton |
| `AUDIO_NOISE(dauer_ms[, vol])` | Weisses Rauschen |

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

## Externer Typ

| Typ | Wirkung |
|---|---|
| `AUDIO_CHANNEL` | Handle auf einen Mixer-Channel (Returnwert von `AUDIO_PLAY`) |

`SOUND` kommt aus Core — `LOADSOUND` und `AUDIO_TONE`/`AUDIO_NOISE` liefern beide ein SOUND. Untereinander austauschbar.

## Typische Game-Patterns

**SFX-Manager mit Lautstaerke-Master:**

```basic
DIM sfx_volume AS FLOAT
sfx_volume = 0.7

SUB PlaySfx(s AS SOUND)
    AUDIO_PLAY(s, 0, sfx_volume)
END SUB
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

## Beispiel

[examples/68_audio.gb](../examples/68_audio.gb) demonstriert das volle Modul-API inklusive Tone-Generation, Pan, Music-Queue.
