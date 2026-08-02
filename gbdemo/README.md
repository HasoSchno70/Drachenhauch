# GameBasic — Demo

Eine Demo im Sinne der Demoszene: **ein durchlaufendes Stück**, das Szene für
Szene zeigt, was die Laufzeit kann — und dabei auf die Musik reagiert, statt
bloß daneben zu laufen.

```
gbrun.py gbdemo\gbdemo.gb          # oder im Editor F5
```

| Taste | Wirkung |
|---|---|
| 1 – 8 | direkt in eine Szene springen |
| LEER | eine Szene weiter |
| F11 | Vollbild |
| ESC | Ende |

## Was das Ganze trägt

Zwei Bausteine machen den Unterschied zwischen „Bildchen mit Musik dazu" und
einer Demo:

- **`AUDIO_FFT(arr)`** greift das **echte Spektrum vom Master** ab. Alles, was
  hier pulsiert, pulsiert auf dem tatsächlich hörbaren Mix — nicht auf einem
  nachgebauten Zähler. Die Demo fasst die Bänder zu `bass`/`mitten`/`hoehen`
  zusammen und leitet daraus auch die Schlagerkennung ab (Bass deutlich über
  seinem eigenen gleitenden Mittel, mit Sperrzeit via `COOLDOWN`).
- **`AUDIO_MUSIC_POSITION()`** liefert die **Sekunde im Stück**. Daran hängt der
  Ablaufplan, nicht an der Wanduhr: bricht die Bildrate ein, verrutschen die
  Szenen trotzdem nicht gegen die Musik.

## Die Szenen

| # | ab | Szene | zeigt |
|---|---|---|---|
| 1 | 0:00 | Titel | Plasma im Post-Effekt, Sinus-Scroller, Spektrum-Säulen |
| 2 | 0:42 | Sternenflug | `LINES` — 1400 Streifen in einem Aufruf, `particles`, additives Blenden |
| 3 | 1:24 | Würfelfeld | `MODEL_INSTANCED` — 1600 Würfel in drei Draw-Calls, `m3d`-Matrizen |
| 4 | 2:08 | PBR + HDR | `LIGHT_ENV_HDR` + `SKYBOX`, `MODEL_PBR` von spiegelnd bis matt, `SHADOW_*`, `MODEL_EMISSIVE` |
| 5 | 2:52 | Physik | *in Arbeit* |
| 6 | 3:34 | Tunnel | *in Arbeit* |
| 7 | 4:16 | Klangfarben | *in Arbeit* |
| 8 | 5:00 | Abspann | *in Arbeit* |

## Der Post-Effekt

`assets/shaders/demo.fs` ist **ein** Shader für alle Szenen; die Demo schaltet
ihn über das Uniform `mode` um (0 = nur Glanz, 1 = Plasma, 2 = Tunnel, 3 = CRT)
und füttert ihn mit `bass`, `hoehen` und `fade`.

Wichtig zum Verständnis: `POSTFX` bekommt das **fertige Bild**. Ein Hintergrund
lässt sich dort nur einblenden, *wo das Bild dunkel ist* — die Demo zeichnet
ihre Szenen deshalb auf Schwarz, und der Shader füllt die dunklen Stellen.

## Musik

„Stardust Jam" von **Drozerix**, **gemeinfrei** (The Mod Archive) — ein echtes
ProTracker-Modul, das die Laufzeit in Echtzeit streamt (kein OGG). Drei weitere
gemeinfreie Stücke liegen dabei; zum Wechseln in `gbdemo.gb` die Konstante
`MUSIK` umstellen. Herkunft und Bezugsskript: `assets/CREDITS.txt` bzw.
`download_music.py`.

## Stolpersteine, die hier gelernt wurden

- **`PLOTS`, `LINES`, `BOXES`, `CIRCLES` und `MODEL_INSTANCED` nehmen KEINE
  Stückzahl entgegen** — sie zeichnen immer das *ganze* Array. Ein viertes
  Argument wird stillschweigend ignoriert. Unbenutzte Plätze muss man selbst
  aus dem Bild schieben (Koordinate `-10`, bei Matrizen eine Verschiebung weit
  nach unten).
- Eine **nicht zugewiesene `MAT4` ist NIL** — `MODEL_INSTANCED` lehnt das mit
  klarer Meldung ab. Platzhalter einmal beim Aufbau setzen.
- **`MODEL_INSTANCED` färbt einen ganzen Draw-Call einheitlich.** Mehrere Farben
  = mehrere Aufrufe; drei sind immer noch drei statt 1600.
- `PARTICLE_SET_SIZE` und `PARTICLE_SET_POS` wollen **ganze Zahlen**.
- **GameBasic ignoriert Groß-/Kleinschreibung** — eine lokale `hoehe` verdeckt
  damit lautlos die Konstante `HOEHE`, und aus `HOEHE - 54` wird `0.6 - 54`.
  Das fällt erst als merkwürdiger Typfehler weit weg von der Ursache auf.
- **Licht, Himmel und Schatten sind globaler Zustand.** Wer sie in einer Szene
  umstellt, muss sie beim Wechsel wieder zurückstellen (hier `szene_start`),
  sonst zieht der HDR-Himmel auch hinter der nächsten Szene auf.
- `MODEL_EMISSIVE` mit Stärke über ~1.5 ergibt zusammen mit dem Bloom nur noch
  eine weiße Scheibe — die Farbe geht verloren.
