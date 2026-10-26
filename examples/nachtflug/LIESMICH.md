# Nachtflug

Eine Demo in Drachenhauch: knapp drei Minuten Musik und Bild aus einem
Programm, ohne eine einzige Klang- oder Bilddatei.

```
dhrt run main.dh                     # im Vollbild, von vorn
dhrt run main.dh -- --fenster        # im Fenster
dhrt run main.dh -- --ab 48          # ab Takt 48 (der Breakdown)
dhrt run main.dh -- --wav nacht.wav  # nur die Musik als WAV schreiben
dhrt run main.dh -- --messen         # am Ende Bildrate und Spitze im Mischer
```

ESC beendet, F11 schaltet das Vollbild.

## Die Musik

Trance in A-Moll, 128 Schläge je Minute, 92 Takte. Jeder Klang entsteht im
Programm (`musik.dh`):

* **Bassdrum** aus einem fallenden Sinus und einem kurzen Klick,
  **Klatschen** aus drei versetzten Rauschstößen, **Becken** aus
  gefiltertem Rauschen.
* **Offbeat-Bass**: Säge eine Oktave über dem Grundton, Sinus darunter.
* **Flächen**: jeder Akkordton zweimal, links und rechts verschieden
  verstimmt, dazu ein leises Dreieck eine Oktave höher.
* **Arpeggio** in Sechzehnteln, das zwischen links und rechts pendelt, mit
  Ping-Pong-Echo.
* **Lead**: zwei Sägen auseinander verstimmt und verteilt, ein Rechteck
  darunter, Vibrato auf langen Tönen, Ping-Pong-Echo.
* **Aufstieg** und **Wirbel** vor jedem Höhepunkt, ein **Becken** auf der Eins.

Ablauf: Intro (das Filter öffnet sich), Aufbau, Höhepunkt mit Melodie,
Breakdown mit großen Flächen und Aufstieg, zweiter Höhepunkt, Ausklang.

Gespielt wird auf einer **Audio-Uhr**: jedes Bild plant die nächsten
Sechzehntel voraus, die Uhr spielt sie sample-genau -- und die Bilder fragen
dieselbe Uhr, darum sitzt jeder Puls auf der Bassdrum, auch wenn die Bildrate
schwankt. Echo und Stereobreite sind in die Klänge eingerechnet
(`AUDIO_SOUND_MIX` mit Panorama); Hall, Filter und Kompressor liegen auf dem
Bus.

## Die Bilder

| Takte | Szene | Wie |
|---|---|---|
| 0–16 | Ozean bei Nacht, Polarlicht, Mondglitzern | Fragment-Shader `ozean.fs` |
| 16–32 | Tiefflug durch ein Tal, ein Fluss aus Licht, Wellen im Takt | Raymarching über ein Höhenfeld, `flug.fs` |
| 32–48 | Lichttunnel, die Farbe folgt dem Akkord | `tunnel.fs` |
| 48–64 | Sternennebel, zum Aufstieg hin schneller und heller | Volumen-Raymarching, `nebel.fs` |
| 64–80 | Galaxie aus 1.800 Würfeln | echte 3D-Szene, `MODEL_INSTANCED`, Glanz in `glanz.fs` |
| 80–92 | Ozean in der Morgendämmerung, Abspann | wieder `ozean.fs` |

Die Shader-Szenen rechnen das ganze Bild im Fragment-Shader (`POSTFX`); was
das Programm vorher zeichnet (die Schrift), legt der Shader obenauf.
Gezeichnet wird in 1280×720, im Vollbild skaliert die Laufzeit hoch.

Geprüft in `tests/pruef/beispiel_nachtflug.dhtest`: die Musik an der
ausgeschriebenen WAV (Stereo, Länge, Pegel), jede Szene an einem
Bildschirmfoto.
