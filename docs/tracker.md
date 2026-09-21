# Tracker (Musik-Editor, `examples/190_tracker.dh`)

Ein mehrspuriger Tracker zum Komponieren von Musik — geschrieben in
Drachenhauch selbst, auf dem `gui`-Modul. Er ist das Gegenstück zum
früheren Qt-Werkzeug `dhtracker`, das mit dem Python-Teil des Projekts
entfallen ist, und liest und schreibt **dasselbe Song-JSON** wie dieses
(unten). Komplementär dazu macht der [SFX-Generator](sfx-generator.md)
einzelne Geräusche.

Er hat Patterns (1–64 Reihen) über 4–32 Kanäle, der **letzte Kanal ist
immer der Schlagzeugkanal**; eine Reihenfolge (Order) daraus; 18 fertige
Instrumente mit Hüllkurve, Vibrato und Detune, alle änderbar; je Note
Lautstärke, Portamento, Effekt und ein eigenes Instrument; Blockauswahl mit
Kopieren, Transponieren und Interpolieren; Stumm/Solo und Mixer-Regler je
Kanal; Rückgängig über den ganzen Song; die Ausgabe als WAV und als GB-Code.

## Starten

In der IDE (`ide/ide.dh`) unter **Werkzeuge → Tracker** (auch über die
Befehlspalette, „Werkzeug: Tracker"). Das startet ihn als eigenes Programm.

Von der Kommandozeile:

```
dhrt run examples/190_tracker.dh                   # leerer Song
dhrt run examples/190_tracker.dh -- song.json      # gleich mit einer Datei
```

Ein relativer Dateiname meint den Ordner, in dem man beim Aufruf stand
(`DHRT_START_DIR`), nicht den der Quelle. So öffnet auch das
[Notenblatt](score-editor.md) über **[In Tracker öffnen]** den Tracker mit
dem eben umgerechneten Stück.

## Das Fenster

Oben zwei Werkzeugleisten, links die Spalte für den Kanal unter dem Cursor,
das Instrument dieses Kanals und die Reihenfolge, rechts das **Gitter**
(Reihen von oben nach unten, je Kanal eine Spalte in eigener Farbe), unten
die **Klaviatur**, ganz unten die Statuszeile (ein `*` heißt: ungesichert).

| Bereich | Was es gibt |
|---|---|
| Leiste 1 | [Neu], [Oeffnen], [Sichern], [Sichern als], **BPM**, **Kanäle** (4–32), [> Pattern], [> Song], [Stopp], [Zurueck], [Vor], [WAV ...] mit den Kästchen **Stereo** und **Amiga**, [GB-Code] |
| Leiste 2 | Pattern-Auswahl, **Reihen** (1–64), [+] neues Pattern, [Dup] kopieren, [Loesch], [Leeren], **Oktave** der Tastatur, dann die Felder der Zelle unter dem Cursor: **Vol** (1–15, 0 = Standard), **Slide** (−12…+12 Halbtöne), **FX** mit Parameter (0–255), **Instrument** der Note, [Note aus] |
| Kanal | Standard-Instrument des Kanals (oder nackte Wellenform `square`/`saw`/`sine`/`triangle`), **M** stumm, **S** solo, Mixer-Regler 0–100 % |
| Instrument | Wellenform (dazu `noise`), Lautstärke 1–15, Attack/Decay/Sustain/Release, Vibrato (Tiefe %, Hz), Detune in Cent, Pan; [+ Instrument] legt eine Kopie des gezeigten an, [Entfernen] entfernt es — Kanäle und Noten, die auf die Instrumente dahinter zeigen, rücken mit |
| Song (Reihenfolge) | Liste der Patterns in Abspielfolge; ein Klick zeigt das Pattern im Gitter. [+ akt.] hängt das gezeigte an, [entf.] entfernt, [<]/[>] verschieben |

Ein Pattern trägt seinen Namen aus der Datei (neue heißen `P1`, `P2` …);
umbenennen lässt es sich hier nicht.

## Bedienung

Ein Klick ins Gitter setzt den Cursor, Ziehen oder Umschalt+Klick wählt
einen Block; das Mausrad rollt die Reihen, mit Umschalt die Kanäle. Solange
kein Bedienelement den Fokus hat, gehören die Tasten dem Gitter:

| Taste | Wirkung |
|---|---|
| Pfeile, Bild auf/ab, Pos1/Ende | Cursor bewegen (Bild: 16 Reihen); mit Umschalt einen Block wählen |
| `Z S X D C V G B H N J M` | Noten der unteren Oktave (die Lage der US-Tastatur — auf einer deutschen liegt das Z auf dem Y) |
| `Q 2 W 3 E R 5 T 6 Y 7 U` | Noten der Oktave darüber |
| `0` | Note aus (Key Off, `OFF` in der Zelle) |
| Entf, Rücktaste | Zelle bzw. markierten Block löschen — samt ihren Effekten |
| Strg+C / Strg+X / Strg+V | Block kopieren / ausschneiden / an der Cursorstelle einfügen |
| Strg+Pfeil hoch/runter | Block um einen Halbton transponieren, mit Umschalt um eine Oktave; der Schlagzeugkanal bleibt |
| Strg+I | je Kanal zwischen erster und letzter Note des Blocks interpolieren |
| Strg+Z / Strg+Y | zurück / vor |
| Strg+S / Strg+O | sichern / öffnen |
| Leertaste | das Pattern abspielen, noch einmal: anhalten |
| ESC | beenden |

Eine gesetzte Note klingt kurz zum Vorhören an, und der Cursor rückt eine
Reihe weiter — so tippt man eine Melodie ohne Pfeiltaste. Die Klaviatur
unten setzt Noten auch mit der Maus.

Eine Note klingt, **bis auf ihrem Kanal die nächste Note oder ein `OFF`
kommt**; im Song-Modus auch über das Pattern-Ende hinweg ins nächste
Pattern der Reihenfolge. Die Länge steuert man also über den Abstand im
Gitter.

Die Felder der Zelle (Vol, Slide, FX, Instrument) wirken nur, wo eine Note
steht; Slide nicht auf dem Schlagzeugkanal. Die Effekte:

| FX | Parameter | Wirkung |
|---|---|---|
| `Arp` | zwei Hex-Stellen `xy` | Grundton, +x und +y Halbtöne im Tick-Takt (der C64-Akkord) |
| `Vib` | `xy` | Vibrato mit x Hz und y Achtel-Halbtönen Tiefe |
| `Ret` | Ticks | schlägt die Note alle n Ticks neu an |
| `Off` | — | wird gelesen und gesichert, spielt hier aber nichts (Sample-Offset braucht Samples) |

**Rückgängig** merkt sich den ganzen Song als Text, 32 Stände — Noten,
Patterns, Reihenfolge, Tempo und Instrumente, ohne eigene Buchführung je
Änderungsart. Ein Zug an einem Regler ist dabei EIN Schritt, nicht einer je
Bild; der Cursor bleibt beim Zurücknehmen stehen.

**Stumm und Solo** wirken beim Abspielen und Vorhören; die WAV mischt immer
alle Kanäle.

## Wiedergabe

[> Pattern] spielt das gezeigte Pattern in Schleife, [> Song] die
Reihenfolge; im Song-Modus folgt das Gitter dem laufenden Pattern, die
klingende Reihe ist markiert.

Die Noten laufen auf einer **Audio-Uhr** (`AUDIO_CLOCK_NEW` +
`AUDIO_PLAY_AT`), zwei Reihen voraus geplant — samplegenau vom
Audio-Faden gestartet, nicht bildgetrieben: ein Bild sind 16 ms, und das
hörte man bei Sechzehnteln. Eine Reihe hat 6 Ticks (das Raster für
Arpeggio und Retrigger). Gestoppt wird, indem die Uhr **entfernt** wird,
nicht angehalten: ein Klang, der auf eine Uhr wartet, die es nicht mehr
gibt, startet nie — angehalten käme er beim nächsten Start als Geisternote
an anderer Stelle wieder. Ein Tempowechsel während der Wiedergabe stellt
die Uhr sofort um.

Gespielt wird jede Note als `AUDIO_NOTE` (Hüllkurve mit Sustain-Pegel,
Release hängt hinten an, Detune, Portamento); ohne Instrument klingt die
nackte Wellenform des Kanals, auf dem Schlagzeugkanal ein kurzes Rauschen.

## WAV

[WAV ...] mischt den ganzen Song offline zu einer Datei — über **dieselbe
Routine** wie die Wiedergabe, nur ist das Ziel ein Misch-Puffer
(`AUDIO_SOUND_NEW` + `AUDIO_SOUND_MIX`) statt der Uhr; sonst klänge die
Datei anders als das, was man hört. Danach wird normalisiert
(`AUDIO_SOUND_NORMALIZE`, die Statuszeile nennt den Faktor) und mit
`AUDIO_SAVE_WAV` geschrieben.

- **Stereo** wertet den Pan der Instrumente aus; ohne das Kästchen bleibt
  die WAV einkanalig.
- **Amiga** legt dazu die Kanäle wie Paula: 1 und 4 links, 2 und 3 rechts
  (bei mehr Kanälen wiederholt sich das Muster), nicht ganz hart.

Im Spiel dann `PLAYMUSIC("song.wav")`.

## GB-Code

[GB-Code] schreibt einen **bildgetriebenen Live-Player** als `.dh`-Datei,
wie der Export der Qt-Fassung: die Reihenfolge wird zu einer Zeitachse
(wiederholte Patterns werden dupliziert), je Kanal ein Feld `trk<n>` mit den
Frequenzen, dazu `TRACKER_PLAY_ROW` und `TRACKER_UPDATE`. Hat ein Kanal
Noten mit Lautstärke, kommt eine Spur `trkV<n>` und der Helfer
`TRACKER_AMP` dazu; mit Slide eine Spur `trkSl<n>` (Hz/s), die Note spielt
dann über `AUDIO_SFX` statt `AUDIO_TONE`. Der Mixer-Regler eines Kanals geht
mit in die Lautstärke, der Schlagzeugkanal spielt `AUDIO_NOISE`.

Die Datei importiert sich selbst `audio`; im eigenen Programm bindet man
sie mit `IMPORT "song.dh"` ein und ruft im Game-Loop:

```
TRACKER_UPDATE(DELTA() * 1000.0)
```

Der Player kennt je Kanal **eine** Wellenform (die des Kanal-Instruments)
und spielt jede Note genau eine Reihe lang. Hüllkurven, Instrumente je
Note und die FX-Spalte kann er nicht — für fertige Musik ist die WAV der
Weg.

## Dateiformat

Ein Song ist eine JSON-Datei (`format: "dhtracker-song"`), dieselbe wie die
der Qt-Fassung, in beide Richtungen. Der Tracker schreibt sie eingerückt
mit `JSON_PRETTY`.

| Schlüssel | Inhalt |
|---|---|
| `format`, `version` | `"dhtracker-song"`, `1` |
| `bpm` | Tempo (gelesen wird 40–300) |
| `channels` | Kanalzahl, der letzte ist das Schlagzeug |
| `waves` | Wellenform je Tonkanal (ohne den Schlagzeugkanal) |
| `patterns` | Liste; je Pattern `name`, `rows`, `channels` und die Gitter `data`, `vol`, `slide`, `fx`, `fxp`, `inst` — je Kanal eine Liste mit einem Wert je Reihe, `null` = leer |
| `order` | Pattern-Nummern in Abspielfolge |
| `channel_vol` | Mixer-Regler 0.0–1.0 je Kanal; nur, wenn einer nicht auf 1.0 steht |
| `instruments` | Liste; ein Synth-Instrument mit `name`, `kind: "synth"`, `default_vol`, `pan`, `waveform`, `env_attack_ms`/`env_decay_ms`/`env_sustain`/`env_release_ms`, `vib_depth`, `vib_speed`, `detune_cents` (dazu `loop_mode`/`loop_start`/`loop_end` für die Qt-Fassung) |
| `channel_inst` | Standard-Instrument je Kanal, `null` = nackte Wellenform |

Die Werte in `data` sind MIDI-Noten (0–127), `-1` ist Note aus. `vol` ist
1–15, `slide` −12…12, `fx` 1 = Arp, 2 = Vib, 3 = Ret, 4 = Off, `fxp` 0–255,
`inst` die Nummer in `instruments`. Die Gitter `vol`, `slide`, `fx`/`fxp`
und `inst` stehen nur in der Datei, wenn das Pattern dort etwas hat.
Beim Lesen fehlt nichts, was fehlen darf: ein unbekannter Wert wird
übergangen, eine Instrument-Nummer ohne Instrument dahinter zurückgesetzt.

**Sample-, Keymap- und SoundFont-Instrumente** der Qt-Fassung (eine
SoundFont wurde dort zu einem Keymap-Instrument) **spielt** dieser Tracker
— beim Abspielen, Vorhören und in der WAV. Die Listen nennen ihre Art
(`Klavier [sample]`). Gespielt wird nach den Regeln der Qt-Fassung: die
Tonhöhe ist der Abstand zur Grundtaste (`base_note` bzw. `root_note`), ein
Loop gilt mit `forward` oder `pingpong`, wenn 0 ≤ Anfang < Ende ≤ Länge,
sonst läuft das Sample einmal durch und verstummt; eine Keymap nimmt die
erste Zone, die die Taste abdeckt, sonst die nächstgelegene. Die Hüllkurve
ist dieselbe wie beim Synth (das Ausklingen hängt hinten an), ebenso der
Slide; **Vibrato gibt es für Samples nicht** (auch in der Qt-Fassung nicht).

Bearbeiten lassen sich an einem solchen Instrument nur Name, Lautstärke,
Pan und Hüllkurve; sein JSON wird beim Sichern **unverändert
zurückgeschrieben**, Kopieren und Entfernen tragen es mit. Bis 2026-09-17
schrieb der Tracker es als Synth zurück, und die eingebetteten Samples waren
nach einem Sichern weg. Ein Instrument einer Art, die der Tracker nicht
kennt, bleibt erhalten und **stumm** („hier stumm" in der Liste). Die
Samples werden je Inhalt einmal dekodiert und nicht bei jedem Rückgängig
neu. Der **GB-Code** spielt weiterhin nur Wellenformen — ein
Sample-Instrument klingt dort als die Wellenform seines Kanals.

## Beenden

ESC, das Kreuz des Fensters oder Alt+F4 beenden sofort, wenn alles
gesichert ist. Sonst kommt die Frage **Sichern | Verwerfen | Abbrechen**.
[Neu] fragt bei ungesicherten Änderungen, ob sie verworfen werden sollen.

## Was die Qt-Fassung hatte und hier fehlt

- **Sample-Instrumente anlegen und bearbeiten** — Laden von WAV/OGG/SF2,
  der Keymap-Dialog und der Instrument-Editor mit Wellenform und
  Loop-Markern. Instrumente aus einer Datei der Qt-Fassung spielt der
  Tracker (oben), neue Samples bringt man hier nicht hinein.
- **VU-Meter** je Kanal.
- **Pattern umbenennen** — die Namen aus der Datei bleiben, neue lassen
  sich nicht vergeben.
- **Der Sample-Offset-Effekt** (`Off`) — er wird mitgeführt, wirkt aber
  nicht, weil er Samples braucht.

## Was dieser Pilot in der Laufzeit freigelegt hat

Er war der erste Editor mit einer **Zeitachse** (siehe CLAUDE.md, fünfter
Pilot) und brauchte drei Bausteine, die es vorher nicht gab:
`AUDIO_NOTE` (eine gehaltene Note mit Sustain-**Pegel** und Ausklingen —
`AUDIO_SFX` kennt nur drei Zeiten, Orgel und Klavier waren nicht zu
unterscheiden), `AUDIO_SOUND_NEW`/`AUDIO_SOUND_MIX`/`AUDIO_SOUND_NORMALIZE`
(ohne Mischen keine WAV) und `JSON_APPEND_NULL` (eine Liste mit leeren
Plätzen ließ sich nicht schreiben, und genau so notiert das Format eine
Reihe ohne Note). Siehe [Audio](module-audio.md) und [JSON](module-json.md).

Die Sample-Instrumente brauchten zwei weitere (2026-09-21):
`SAMPLE_FROM_BUFFER` (ein Sample aus 16-Bit-PCM — so liegen die Samples
Base64-kodiert in der Datei, mit `BUFFER_FROM_BASE64` davor ohne Umweg über
eine Datei) und `SAMPLE_NOTE` (eine Note aus einem Sample als **SOUND**
statt sofort abgespielt wie `SAMPLE_PLAY` — nur ein Klang lässt sich auf der
Audio-Uhr planen und in die WAV mischen). Dazu kennt `SAMPLE_SET_LOOP` jetzt
die Art `pingpong`.

## Geprüft

`tests/pruef/werkzeug_tracker.dhtest` bedient den Tracker wie von Hand
(Aufnahme über `AUTOMATION_PLAY`) und prüft an drei Ergebnissen, die
Drachenhauch selbst liest: die **Datei** (json-Modul, mit den Schlüsseln,
die die Qt-Fassung las; dazu zwei Dateien, die die Qt-Fassung geschrieben
hat), die **WAV** (Länge, Noten, Stereo- und Amiga-Pan) und den
**GB-Code** (`dhrt --check` und ein Start). Dazu Tastatur, Rückgängig,
Blockbefehle, Transponieren ohne das Schlagzeug, die Uhr per Leertaste,
das Mitlaufen im Song-Modus, die Lage aller Bedienelemente und die
Sample-/Keymap-Instrumente: ein Lied, das der Fall selbst baut (Sinus mit
bekannter Frequenz, eine Keymap mit einer stillen und einer Loop-Zone),
muss in der WAV mit der richtigen Tonhöhe klingen, das Sample ohne Loop
verstummen und der Loop die Note bis zum Pattern-Ende halten.

## Audio Studio

Das Audio Studio (`dhsound`), das Tracker und SFX-Generator der Qt-Fassung
in einem Vollbildfenster mit Reitern vereinte, gibt es nicht mehr. Beide
Werkzeuge sind eigene Drachenhauch-Programme und liegen in der IDE unter
**Werkzeuge**: der Tracker hier, der SFX-Generator in
`examples/183_sfx_generator.dh` ([SFX-Generator](sfx-generator.md)).
