# GameBasic — Demo

Eine Demo im Sinne der Demoszene: **ein durchlaufendes Stück**, das Szene für
Szene zeigt, was die Laufzeit kann — und dabei auf die Musik reagiert, statt
bloß daneben zu laufen.

```
gbrun.py gbdemo\gbdemo.gb          # oder im Editor F5
```

Sie **startet im Vollbild**. Gezeichnet wird in 1280×720, die Laufzeit
skaliert ganzzahlig auf den Bildschirm (auf 1440p also exakt 2×, ohne
Unschärfe).

| Taste | Wirkung |
|---|---|
| 1 – 8 | direkt in eine Szene springen |
| LEER | eine Szene weiter |
| A S D F G H J K | acht Klänge auf Zuruf (siehe unten) |
| M | nächstes Musikstück |
| F11 | Vollbild an/aus |
| ESC | Ende |

### Die Klang-Tastatur

Acht Klänge, die **in jeder Szene** auf Tastendruck laufen — Kick, Snare, Zap,
Laser, Coin, Boom, Blip, SID-Bass. Alle sind **prozedural**: `AUDIO_SFX` baut
sie aus Wellenform, Tonhöhe, Pitch-Slide, ADSR, Vibrato und den
SID-Erweiterungen (Pulsbreite/PWM + resonanter Filter-Sweep). Es liegt keine
einzige Klangdatei dabei — jeder dieser Klänge steht als eine Zeile im
Quelltext.

**M** schaltet zwischen den **acht** gemeinfreien Musikstücken um — von einem
4-Kanal-Amiga-Modul bis zu einem 24-Kanal-Satz; der Unterschied ist deutlich zu
hören. Der eingeblendete Name nennt Kanäle und Länge dazu. Wichtig dabei:
der Ablaufplan hängt an `AUDIO_MUSIC_POSITION`, und die springt beim Laden auf
0 zurück — die Demo schiebt deshalb `versatz` mit, sonst finge sie bei jedem
Musikwechsel wieder bei Szene 1 an.

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
| 5 | 2:52 | Physik | `physics2d` (Rapier) — Neon-Logo aus Klötzen, Funken und Erschütterung beim Einschlag, dazu ein Trümmerregen im Hintergrund (mehrere hundert Körper gleichzeitig); jeder Buchstabe fällt einzeln |
| 6 | 3:34 | Tunnel | Post-Effekt-Tunnel (1/r), Spektrum-Ring per `SPLINE`, echte Schweife aus einem behaltenen `RENDERTARGET` |
| 7 | 4:16 | Klangfarben | die Effektkette bei der Arbeit: `AUDIO_FILTER`, LFO-Wobble, `AUDIO_DELAY`/`REVERB`/`DISTORTION`, Auto-Pan |
| 8 | 5:00 | Abspann | Drahtgitter-Ring, laufender Abspann, kreisender Ping über `AUDIO_EMITTER`/`LISTENER` |

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
ProTracker-Modul, das die Laufzeit in Echtzeit streamt (kein OGG). Der
Ablaufplan ist auf dieses Stück geschnitten. **Sieben weitere** gemeinfreie
Stücke desselben Urhebers liegen dabei und lassen sich im Betrieb mit **M**
durchschalten:

| # | Stück | Kanäle | Länge |
|---|---|---|---|
| 1 | Stardust Jam | 4 | 6:24 |
| 2 | Silicon Dancer | 4 | 4:05 |
| 3 | Neon Techno | 4 | 3:58 |
| 4 | Mecanum Overdrive | 4 | 2:57 |
| 5 | Assembly! | 4 | 1:55 |
| 6 | Keygen Wraith | 6 | 1:33 |
| 7 | Building Energy | 24 | 1:29 |
| 8 | Cyber Spider | 10 | 1:03 |

Die kurzen Stücke loopen; der Ablaufplan läuft davon unbeirrt weiter. Herkunft und Bezugsskript: `assets/CREDITS.txt` bzw.
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
- **`MODEL_INSTANCED` färbt einen ganzen Draw-Call einheitlich** — raylibs
  `DrawMeshInstanced` überträgt nur Matrizen, keine Farb-Attribute. Man darf
  aber ein `ARRAY OF INTEGER` als Farbe übergeben; die Laufzeit gruppiert dann
  selbst und macht *einen Draw-Call je verschiedener Farbe*. Drei Farben für
  1600 Würfel sind drei Draw-Calls — 1600 verschiedene wären 1600, dafür wäre
  ein Verlauf im Shader die bessere Antwort. *(Das Farb-Array kam 2026-08-02;
  vorher musste man die Matrizen selbst auf mehrere Listen verteilen.)*
- **GameBasic ignoriert Groß-/Kleinschreibung** — eine lokale `hoehe` verdeckt
  damit lautlos die Konstante `HOEHE`, und aus `HOEHE - 54` wird `0.6 - 54`.
  Das fällt erst als merkwürdiger Typfehler weit weg von der Ursache auf.
- **Licht, Himmel, Schatten und die Audio-Bus-Effekte sind globaler Zustand.**
  Die Demo klammert jede Szene deshalb in `GFX_PUSH`/`GFX_POP` und
  `AUDIO_PUSH`/`AUDIO_POP` — der Wechsel holt alles zurück, ohne dass irgendwo
  eine Liste gepflegt wird. *(Bis 2026-08-03 gab es das nicht; `szene_start`
  musste jede Einstellung von Hand zurückdrehen, und eine vergessene Zeile fiel
  erst zwei Szenen später auf.)*
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
- **`MESH_PLANE` kachelt seine Textur nicht** — sie wird *einmal* über die
  ganze Ebene gespannt. Eine 400 Einheiten breite Fläche zeigt eine 1024er
  Textur also so grob gestreckt, dass wieder eine Farbfläche daraus wird. In
  Szene 4 löst das eine kleinere Fläche (130) plus `LIGHT_FOG`, das die Kante
  am Horizont verschluckt.
- **`LIGHT_FOG` ist globaler Zustand** wie Licht und Himmel — beim
  Szenenwechsel zurücksetzen.
- **Ein Render-Target hat seine EIGENE Pixelgröße** — der Inhalt darf nicht mit
  dem Fenster-Maßstab hineingezeichnet werden. Im Vollbild (Maßstab 2) landete
  sonst alles doppelt so groß in einem Ziel fester Größe; was rechts herausfiel,
  blieb in einem behaltenen Target als klebender Rand stehen. *(In dhrt behoben,
  2026-08-02, mit Regressionstest.)*
- **Ein Bloom muss ein halbes Texel vom Rand entfernt abtasten.** Auf 0..1 zu
  klemmen reicht nicht: genau auf der Texturkante mischt die bilineare Filterung
  die letzte Zeile mit der ersten, und der helle Inhalt vom unteren Rand blutet
  oben wieder herein (war als Reihe Farbfetzen in der obersten Pixelzeile zu
  sehen).
- **Ein Spektrum-Ring braucht ein gespiegeltes Spektrum.** Legt man die Bänder
  einfach rundherum, stoßen Band 31 (Höhen, klein) und Band 0 (Bass, groß)
  direkt aneinander — ein Sprung von über 150 Pixeln zwischen zwei
  Nachbarpunkten, aus dem die Catmull-Rom einen weit hinausschießenden Zacken
  macht. Der Spline selbst schließt sauber (nachgemessen); die Daten waren
  unstetig, nicht die Kurve.
