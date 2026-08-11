# Audio-Modulatoren: LFO und Tweener

Ein **Modulator** ist ein Wert, der sich von selbst ändert und damit einen
Audio-Parameter fährt. Statt pro Frame nachzurechnen, sagt man einmal, *was*
sich *wie* bewegen soll — den Rest erledigt Kira auf dem Audio-Thread.

Das ist nicht nur bequemer, sondern auch besser: die Bewegung läuft
**sample-genau** weiter, selbst wenn die Bildrate einbricht. Ein Tremolo, das
im BASIC-Loop pro Frame gesetzt wird, stottert bei einem Ruckler hörbar.

Es gibt zwei Sorten, beide mit demselben Handle-Typ `AUDIO_MOD`:

| | schwingt | fährt zu einem Ziel | typisch für |
|---|---|---|---|
| **LFO** | ja, endlos | nein | Tremolo, Vibrato, Auto-Wah, Wobble-Bass |
| **Tweener** | nein | ja, auf Kommando | Ducking, Filter-Sweep beim Levelwechsel |

## LFO

```basic
IMPORT "audio"
DIM wah AS AUDIO_MOD
wah = AUDIO_LFO_NEW("sine", 1.5)          ' 1,5 Schwingungen pro Sekunde
AUDIO_MODULATE("sfx", "filter", wah, 200.0, 6000.0)
```

Damit wandert der Filter-Cutoff des SFX-Busses dauerhaft zwischen 200 und
6000 Hz — ohne eine einzige Zeile in der Hauptschleife.

- `AUDIO_LFO_NEW(wellenform$, hz [, amplitude [, mitte]])` → `AUDIO_MOD`
  Wellenformen: `sine`, `triangle`, `saw`, `pulse` (deutsche Namen `sinus`,
  `dreieck`, `saegezahn`, `rechteck` gehen auch). Der LFO schwingt zwischen
  `mitte - amplitude` und `mitte + amplitude`; Standard ist Amplitude 1 und
  Mitte 0, also **-1 bis +1**.
- `AUDIO_LFO_SET(modulator [, hz [, amplitude [, mitte]]])` — zur Laufzeit ändern.
  Weggelassene Werte bleiben unverändert.
- `AUDIO_LFO_WAVEFORM(modulator, wellenform$)`

## Tweener

```basic
DIM hall AS AUDIO_MOD
hall = AUDIO_TWEENER_NEW(0.0)
AUDIO_MODULATE("sfx", "reverb", hall, 0.0, 0.9)
AUDIO_TWEENER_TO(hall, 1.0, 1200.0, "inout")   ' in 1,2 s weich aufziehen
```

- `AUDIO_TWEENER_NEW([startwert])` → `AUDIO_MOD`
- `AUDIO_TWEENER_TO(modulator, ziel, dauer_ms [, easing$])`
  Easings wie bei den Fades: `linear`, `in`, `out`, `inout`.

## Binden und lösen

```basic
AUDIO_MODULATE(bus$, ziel$, modulator, min, max)
AUDIO_MOD_REMOVE(modulator)
```

- `bus$`: `sfx`, `music` oder `master`
- `ziel$`:
  - `volume` — **Tremolo**. `min`/`max` sind ein Faktor wie bei
    `AUDIO_BUS_VOLUME` (1.0 = unverändert), nicht Dezibel.
  - `pan` — **Auto-Pan**. -1 = ganz links, +1 = ganz rechts.
  - `filter` (Cutoff in Hz), `resonance`, `reverb` (Mix 0..1),
    `distortion` (Stärke in dB)

Feste Balance ohne Modulator: `AUDIO_BUS_PAN(bus$, pos)`. Bis dahin ließ sich
nur ein *einzelner* Kanal pannen (`AUDIO_PAN`), nicht die Musik als Ganzes.

> Panning liegt bei Kira nicht auf dem Track, sondern in einem eigenen Effekt.
> Er hängt seit Etappe 6 in jeder Bus-Kette — ohne ihn gäbe es weder
> `AUDIO_BUS_PAN` noch Auto-Pan.

Der Wertebereich des Modulators wird auf `min..max` abgebildet — bei einem
LFO mit Standard-Amplitude also **-1 → min** und **+1 → max**. Wer einen LFO
mit `amplitude = 0.5` baut, nutzt entsprechend nur die mittlere Hälfte des
Bereichs.

## Zwei Dinge, die man wissen sollte

**LFO und Tweener teilen sich den Handle-Typ.** Das hält `AUDIO_MODULATE`
einfach — es nimmt beide. Ruft man aber eine LFO-Funktion auf einem Tweener
auf (oder umgekehrt), sagt die Meldung im Klartext, was los ist, statt nur
„ungültiges Handle".

**`AUDIO_MOD_REMOVE` gibt den Modulator frei**, aber die Bindung an den
Parameter bleibt auf dem letzten Wert stehen. Wer wieder einen festen Wert
will, setzt ihn direkt (`AUDIO_FILTER("sfx", 2000.0)`).

## Demo und Tests

- `examples/150_audio_modulatoren.dh` — Auto-Wah mit umschaltbarer Wellenform
  und ein Tweener auf dem Hall, mit Spektrum-Anzeige.
- `tests/test_audio_modulators.py` — der Kern-Test **misst** über den
  FFT-Abgriff, dass der Höhenanteil des Ausgangs wirklich schwankt (gemessen:
  ohne Modulation 0,065–0,218, mit Modulation 0,736–0,830; die Schwelle 0,45
  liegt mittig dazwischen). Eine Gegenprobe ohne Modulation stellt sicher, dass
  der Test nicht bloß das Eigenzappeln des Rauschens misst.
