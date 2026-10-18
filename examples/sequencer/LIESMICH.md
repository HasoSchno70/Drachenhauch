# Sequencer

Ein Step-Sequencer: sechzehn Schritte, sechs Spuren — Kick, Snare, Hi-Hat,
Clap, Bass und eine Melodie. Ein Klick in ein Feld schaltet die Note an oder
aus; auf Bass und Lead verstellt das **Mausrad** über einem Feld die
Tonhöhe in Halbtönen.

- **Leertaste** spielt und stoppt (gehört sie gerade einem Kästchen oder
  Regler mit Fokus, bedient sie das)
- **Tempo** und **Swing** wirken sofort, auch während des Spielens
- **Hall**, **Echo** und **Wah** liegen auf dem SFX-Bus; das Echo sitzt auf
  einer punktierten Achtel und folgt dem Tempo
- **stumm** neben jeder Spur blendet sie aus
- **Strg+S**/**Strg+O** sichern und laden `muster.json`
- **Strg+E** schreibt zwei Durchgänge sofort als `muster.wav` — trocken,
  ohne Hall und Echo
- **Strg+Umschalt+E** spielt zwei Durchgänge und schneidet den Mixer mit
  (`muster_live.wav`, mit allen Effekten; dauert so lange wie die Musik)

Gespielt wird auf einer Audio-Uhr: `AUDIO_CLOCK_NEW` mit zwölf Ticks je
Schritt (für den Swing), die nächsten drei Schritte werden mit
`AUDIO_PLAY_AT` vorausgeplant und kommen auf den Takt genau, auch wenn die
Bildrate einbricht. Hängt das Programm länger, überspringt es die verpassten
Schritte, statt sie auf einmal nachzuholen. Die Klänge entstehen mit
`AUDIO_SFX` (Schlagzeug) und `AUDIO_NOTE` (Bass, Lead), der Mitschnitt mit
`AUDIO_RECORD_START`/`AUDIO_RECORD_STOP`. Die Statusleiste zeigt rechts den
Pegel aus `AUDIO_PEAK` und meldet Übersteuern.

Entstanden beim Dogfooding am 2026-10-11; dabei kamen heraus:
`AUDIO_PLAY_AT` löschte eine noch wartende Instanz desselben Klangs,
`AUDIO_IS_PLAYING` galt schon vor dem Tick, eine WAV mit Bus-Effekten ließ
sich nicht schreiben, der Pegel war nicht abzufragen, und ein Menükürzel auf
der Leertaste nahm einem Kästchen mit Fokus die Taste weg.
