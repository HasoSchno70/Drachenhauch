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
| 5 | 2:52 | Physik | `physics2d` (Rapier) — Logo aus Klötzen, das auf den Schlag zusammenkracht |
| 6 | 3:34 | Tunnel | Post-Effekt-Tunnel (1/r), Spektrum-Ring per `SPLINE`, echte Schweife aus einem behaltenen `RENDERTARGET` |
| 7 | 4:16 | Klangfarben | die Effektkette bei der Arbeit: `AUDIO_FILTER`, LFO-Wobble, `AUDIO_DELAY`/`REVERB`/`DISTORTION`, Auto-Pan |
| 8 | 5:00 | Abspann | Drahtgitter-Knoten, laufender Abspann, kreisender Ping über `AUDIO_EMITTER`/`LISTENER` |

Am Ende springt die Demo wieder an den Anfang — die Musik loopt endlos, der
Ablaufplan geht mit. Ein kompletter Durchlauf wurde headless verifiziert
(6:24, fehlerfrei, inklusive Rücksprung).

## Der Post-Effekt

`assets/shaders/demo.fs` ist **ein** Shader für alle Szenen; die Demo schaltet
ihn über das Uniform `mode` um (0 = nur Glanz, 1 = Plasma, 2 = Tunnel, 3 = CRT)
und füttert ihn mit `bass`, `hoehen` und `fade`.

Wichtig zum Verständnis: `POSTFX` bekommt das **fertige Bild**. Ein Hintergrund
lässt sich dort nur einblenden, *wo das Bild dunkel ist* — die Demo zeichnet
ihre Szenen deshalb auf Schwarz, und der Shader füllt die dunklen Stellen.

### Die einzige Szene, die den Klang anfasst

Szene 7 hängt echte Effekte in den Musik-Bus — Kira fährt sie auf dem
Audio-Thread, das Programm setzt nur Werte und rechnet pro Bild **nichts** nach.
Ein Filter-Sweep läuft deshalb sample-genau weiter, auch wenn die Bildrate
einbricht.

Der Beweis steht im Bild: die Säulen sind das **echte** Spektrum *hinter* der
Kette. Nachgemessen beim Tiefpass-Sweep — Summe der oberen zwölf Bänder:

| Cutoff | Höhen | Bass |
|---|---|---|
| ~7200 Hz | 0.79 | 3.76 |
| ~5400 Hz | 1.30 | 4.84 |
| ~890 Hz | 0.18 | 3.77 |
| ~460 Hz | 0.13 | 5.58 |

Faktor sechs bei den Höhen, der Bass bleibt stehen. Die rechten Säulen
verschwinden also wirklich, statt es nur zu spielen.

**Effekte sind globaler Bus-Zustand** — beim Verlassen der Szene muss alles auf
neutral zurück (`klang_neutral()`), sonst läuft der Abspann durch den Verzerrer.

## Musik

„Stardust Jam" von **Drozerix**, **gemeinfrei** (The Mod Archive) — ein echtes
ProTracker-Modul, das die Laufzeit in Echtzeit streamt (kein OGG). Drei weitere
gemeinfreie Stücke liegen dabei; zum Wechseln in `gbdemo.gb` die Konstante
`MUSIK` umstellen. Herkunft und Bezugsskript: `assets/CREDITS.txt` bzw.
`download_music.py`.

## Stolpersteine, die hier gelernt wurden

- **`PLOTS`, `LINES`, `BOXES`, `CIRCLES` und `MODEL_INSTANCED` nehmen eine
  optionale Stückzahl** als letztes Argument. Ohne sie zeichnen sie das *ganze*
  Array — wer einen Puffer fest dimensioniert und nur teilweise füllt, bekommt
  sonst die alten Werte der restlichen Plätze mitgezeichnet. *(Bis 2026-08-02
  gab es die Stückzahl nicht und ein viertes Argument wurde stillschweigend
  ignoriert; jetzt ist ein Argument zu viel ein Fehler.)*
- Eine **nicht zugewiesene `MAT4` ist NIL** — `MODEL_INSTANCED` lehnt das mit
  klarer Meldung ab. Mit der Stückzahl kommt man aber gar nicht mehr dorthin:
  ungenutzte Plätze werden schlicht nicht angefasst.
- **`MODEL_INSTANCED` färbt einen ganzen Draw-Call einheitlich.** Mehrere Farben
  = mehrere Aufrufe; drei sind immer noch drei statt 1600.
- **GameBasic ignoriert Groß-/Kleinschreibung** — eine lokale `hoehe` verdeckt
  damit lautlos die Konstante `HOEHE`, und aus `HOEHE - 54` wird `0.6 - 54`.
  Das fällt erst als merkwürdiger Typfehler weit weg von der Ursache auf.
- **Licht, Himmel und Schatten sind globaler Zustand.** Wer sie in einer Szene
  umstellt, muss sie beim Wechsel wieder zurückstellen (hier `szene_start`),
  sonst zieht der HDR-Himmel auch hinter der nächsten Szene auf.
- `MODEL_EMISSIVE` mit Stärke über ~1.5 ergibt zusammen mit dem Bloom nur noch
  eine weiße Scheibe — die Farbe geht verloren.
- **Ein Schlagerkenner braucht das ROHE Spektrum.** Die nachfallende Glättung,
  die die Säulen ruhig aussehen lässt, liegt dauerhaft dicht an ihrem eigenen
  gleitenden Mittel — der Vergleich „deutlich über dem Mittel" schlägt darauf
  nie an. Die Demo führt beides getrennt: `glatt[]` fürs Bild, die rohen Werte
  für den Schlag.
- **Ein Logo, das erst stehen und dann fallen soll, baut man statisch auf** und
  schaltet es im richtigen Moment mit `PHYS2D_SET_DYNAMIC` um. Baut man es gleich
  beweglich, fällt der Querbalken des „G" in dem Moment herunter, in dem die
  Szene beginnt — man sieht das Logo nie. *(Bis 2026-08-02 gab es das Umschalten
  nicht; man musste die Körper entfernen und neu anlegen.)*
- **Ein Render-Target wird normalerweise jedes Bild transparent geleert.**
  `RENDERTARGET_NEW(w, h, TRUE)` lässt es stehen — das ist die Voraussetzung für
  Schweife. Verblassen geht nicht durch Selbst-Zeichnen (ein Target kann sich
  nicht selbst zeichnen), sondern durch **Multiplizieren**: `BLEND_MODE("mult")`
  und ein Vollbild-Rechteck in dunklem Grau. *(Bis 2026-08-02 gab es das
  Stehenbleiben nicht; Szene 6 hat den Nachhall vorher nur nachgestellt.)*
- **`MID$` ist 0-basiert.** Ein führendes Zeichen abschneiden heißt
  `MID$(s, 1, LEN(s) - 2)`, wenn auch hinten eines weg soll — mit `- 1` bleibt
  das letzte stehen.
- **Ein Spektrum-Ring braucht ein gespiegeltes Spektrum.** Legt man die Bänder
  einfach rundherum, stoßen Band 31 (Höhen, klein) und Band 0 (Bass, groß)
  direkt aneinander — ein Sprung von über 150 Pixeln zwischen zwei
  Nachbarpunkten, aus dem die Catmull-Rom einen weit hinausschießenden Zacken
  macht. Der Spline selbst schließt sauber (nachgemessen); die Daten waren
  unstetig, nicht die Kurve.
