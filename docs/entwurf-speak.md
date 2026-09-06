# Entwurf: SPEAK — Spiele ohne gui sprechen lassen

> **Stand 06.09.2026: C ist gebaut.** `SPEAK`, `SPEAK_STOP`, `SPEAKING`,
> `SPEAK_WAIT`, `SPEAK_VOICE`, `SPEAK_VOICES`, `SPEAK_RATE`, `SPEAK_SOUND`
> (`rust/drachenhauch_runtime/src/sprache.rs`): Windows über WinRT zu PCM,
> als `SOUND` durch Kira auf dem Bus `speech`; die Warteschlange läuft über
> Kiras `StartTime::Delayed` auf dem Audio-Thread, ohne Polling. Läuft ein
> Bildschirmleser, geht der Satz als Ansage in den Baum — auch ohne gui.
> macOS (`say`) und Linux (`espeak-ng`) wie unten beschrieben, ungeprüft.
> Prüfstein: `tests/test_speak.py` (WAV von Pythons `wave` gelesen, Dauern
> gemessen) und der fremde UIA-Leser in `tests/test_gui_barrierefreiheit.py`.
> Doku: [module-audio.md](module-audio.md), Abschnitt „Sprechen".

*Untersuchung, seither umgesetzt.* Der Rest aus der
[Barrierefreiheit](entwurf-barrierefreiheit.md): dort war Weg B („selbst
sprechen") zugunsten des Baums zurückgestellt worden, mit dem Satz, der eine
Befehl `SPEAK` gehöre trotzdem dazu — für das Text-Adventure, das keinen
Knopf hat. Seit dem Bau der Barrierefreiheit gilt: eine **gui**-Anwendung
kann einem Bildschirmleser alles sagen (`GUI_ANNOUNCE`), ein **Spiel auf der
Zeichenfläche** kann es nicht — für den Leser ist die Zeichenfläche ein Bild,
und ohne laufenden Bildschirmleser spricht ohnehin niemand. Dieses Papier
misst, was diese Maschine an Sprachausgabe hergibt, prüft die Bausteine,
entwirft vier Wege und empfiehlt einen. Die Entscheidung fällt jemand anders.

Alle Angaben sind geprüft, nicht angenommen — Stand 06.09.2026, diese
Maschine (Windows 11, deutsches Sprachpaket) und die Quelltexte. Was nur
unter macOS oder Linux zu prüfen wäre, steht als solches da.

## 1. Was heute geht — gemessen

| Weg | Stand | Grenze |
|---|---|---|
| `GUI_ANNOUNCE(text$)` | spricht über den **Bildschirmleser des Nutzers** (Live-Knoten im Barrierefreiheits-Baum) | nur, wenn einer läuft; nur mit gui — ein Programm ohne `GUI_UPDATE` schickt einen Baum ohne Ansage-Knoten |
| `SHELL_START` mit PowerShell und `System.Speech` | spricht | gemessen **194 ms** je Aufruf, kein Anhalten, kein „spricht noch?", ein Prozess je Satz |
| Ton aus dem Programm | `AUDIO_SOUND_NEW/MIX`, `AUDIO_NOTE`, `AUDIO_SAVE_WAV`, Busse, räumliche Sender | alles da — nur keine Stimme |

Ein Audiogame, ein Text-Adventure mit Vorleser, ein Lernspiel für Kinder,
die noch nicht lesen, ein Sprecher im Strategiespiel („Einheit bereit") —
alles heute nur mit vorher aufgenommenen Dateien. Wer **jeden** Text
sprechen will (Namen, Zahlen, Eingaben des Spielers), kann es nicht.

## 2. Was „sprechen" hier heißt

Zwei Fragen, die man auseinanderhalten muss:

1. **Woher kommt die Stimme?** Vom System (installierte Stimmen, kostenlos,
   in der Sprache des Nutzers), vom Bildschirmleser des Nutzers (seine
   Stimme, sein Tempo, seine Braillezeile), oder aus einer mitgelieferten
   Maschine (überall gleich, aber groß).
2. **Wo kommt der Ton an?** Direkt am Lautsprecher, am Mischer vorbei — oder
   **als Klang in Kira**, wie jeder andere Ton des Spiels: mit Lautstärke des
   Busses, Pause, Ausblenden, räumlich an einer Figur, speicherbar als WAV.

Die zweite Frage entscheidet, ob `SPEAK` ein Fremdkörper ist oder ein Ton
unter Tönen.

## 3. Bausteine, geprüft

### Windows: zwei Sprachausgaben, nicht eine

| | SAPI 5 (`ISpVoice`, `Win32_Media_Speech`) | WinRT (`Windows.Media.SpeechSynthesis`) |
|---|---|---|
| Stimmen hier | Hedda (de), Zira (en) — die alten Desktop-Stimmen | **Stefan, Katja, Hedda** (de) — die Stimmen der Systemsprache; weitere kommen mit Sprachpaketen |
| Ausgabe | direkt an den Lautsprecher (eigener Kanal, am Mischer vorbei) | **ein WAV-Strom**: PCM, mono, 16 kHz, 16 bit |
| Gemessen | Aufruf 3 ms, asynchron | Synthese eines Satzes mit 14 Wörtern: **35 ms** beim ersten Mal, danach **12–13 ms**; „Treffer!" 3 ms; 208 KB bzw. 41 KB |
| Steuerung | Tempo, Lautstärke, Stimme, Anhalten, Ereignisse (Wortgrenzen) | Tempo (`SpeakingRate`, gemessen setzbar), Tonhöhe, Lautstärke, Stimme, SSML, Wortgrenzen als Metadaten |
| Aus Rust | `windows`-Crate, Feature `Win32_Media_Speech` | `windows`-Crate, Features `Media_SpeechSynthesis` + `Storage_Streams` (in 0.62 vorhanden), Async über `.get()` |

Der Unterschied ist grundsätzlich: SAPI **spricht**, WinRT **liefert einen
Klang**. Der Klang lässt sich in Kira genau so laden wie das Ergebnis von
`AUDIO_NOTE` (`make_data_mono` aus Abtastwerten) — dann ist eine gesprochene
Zeile ein `SOUND`, und alles, was das audio-Modul kann, gilt für sie.

### Der Bildschirmleser

Läuft einer, hört er den Barrierefreiheits-Baum. `GUI_SCREENREADER()` sagt
es dem Programm. Der Ansage-Knoten hängt heute am gui-Baum; im Baum, den
`FLIP` für Programme ohne gui schickt (nur das Fenster), fehlt er — das ist
eine Zeile Arbeit. Damit kann auch ein Spiel ohne gui über den Leser des
Nutzers sprechen, wenn er da ist. NVDA-Controller, Tolk, SRAL (direkter
Draht zum Leser, Windows) bleiben wie im Barrierefreiheits-Papier: möglich,
hier nicht installiert, für die Frage nicht nötig.

### macOS und Linux

| System | Baustein | Geprüft |
|---|---|---|
| macOS | `say -o datei --data-format=LEI16@22050 "text"` schreibt PCM in eine Datei; `AVSpeechSynthesizer` bräuchte Objective-C | nicht gemessen |
| Linux | `espeak-ng --stdout -v de "text"` liefert ein WAV auf stdout; `spd-say` spricht nur; `pico2wave` schreibt eine Datei | nicht gemessen; keines ist hier installiert (0 von 4 Werkzeugen) |

Beide gehen über einen Prozess je Satz (geschätzt 100–300 ms), beide
liefern einen Klang, den Kira spielt — derselbe Weg wie unter Windows, nur
langsamer. Ohne Werkzeug: ein Fehler mit klarem Wortlaut.

### Mitgelieferte Maschine

piper (ONNX-Modelle, 20–60 MB je Stimme) spräche überall gleich, offline,
und machte aus dem 17-MB-`dhrt` ein 80-MB-`dhrt` mit einer Stimme in einer
Sprache. Für einen Dialekt, der auch Kinder erreichen will, ist die
Systemstimme in der Sprache des Nutzers der bessere Handel. Verworfen.

## 4. Vier Wege

Der Befehlssatz ist bei allen Wegen derselbe; er steht hier einmal:

```text
SPEAK(text$ [, unterbrechen])     ' spricht; unterbrechen = laufende Ansage abbrechen, sonst anhaengen
SPEAK_STOP()                      ' alles verwerfen
SPEAKING() -> BOOLEAN
SPEAK_VOICE(name$)                ' "Katja", "Stefan" -- oder leer fuer die Systemstimme
SPEAK_VOICES() -> ARRAY OF STRING
SPEAK_RATE(faktor)                ' 0.5 .. 2.0
SPEAK_SOUND(text$) -> SOUND       ' nur synthetisieren: AUDIO_PLAY, AUDIO_PLAY_ON(sender), AUDIO_SAVE_WAV
```

### A. SAPI direkt

`ISpVoice::Speak` mit `SPF_ASYNC`, `GetStatus` für *SPEAKING*, `Skip` zum
Anhalten. Ein Tag, Windows. Spricht am Mischer vorbei: `AUDIO_BUS_VOLUME`
gilt nicht, `AUDIO_PAUSE` nicht, kein Ausblenden, keine räumliche Stimme,
kein `SPEAK_SOUND`. Und es sind die alten Stimmen (Hedda), nicht Stefan und
Katja.

### B. Synthese zu PCM, Ton durch Kira

Windows über WinRT (12–35 ms je Satz, gemessen), macOS über `say`, Linux über
`espeak-ng` (beide ungeprüft). Das Ergebnis ist ein `SOUND`: `SPEAK` legt es
auf einen eigenen Bus `speech` (`AUDIO_BUS_VOLUME("speech", …)`), hält eine
Warteschlange (anhängen oder unterbrechen), *SPEAKING* fragt den Kanal.
`SPEAK_SOUND` gibt den Klang heraus — für einen sprechenden Gegner an seinem
räumlichen Sender, für eine WAV, die man ins Spiel einbaut, für das Mischen
in einen Song. Ein Cache je (Text, Stimme, Tempo) spart die Synthese beim
zweiten Mal; 200 KB je Satz sind bei hundert Sätzen 20 MB — begrenzt.
Zwei bis drei Tage. **Prüfbar hier:** `SPEAK_SOUND` liefert einen Klang mit
Hüllkurve (`AUDIO_SOUND_WAVE` ≠ 0), `AUDIO_SAVE_WAV` schreibt 16 kHz, Pythons
`wave` liest es zurück; *SPEAKING* geht an und wieder aus. Auf dem
Windows-Läufer der CI sind die englischen Stimmen da.

### C. B, und der Bildschirmleser zuerst

Wie B, aber `SPEAK` fragt zuerst `GUI_SCREENREADER()`: läuft ein Leser,
geht der Satz als Ansage in den Baum (der Ansage-Knoten kommt dafür auch in
den Baum ohne gui) — mit der Stimme, dem Tempo und der Braillezeile des
Nutzers, und ohne zwei Stimmen gleichzeitig. Sonst die Systemstimme.
`SPEAK_SOUND` bleibt immer Synthese (ein Klang ist ein Klang). Ein Tag mehr.
Das ist der Weg D aus dem Barrierefreiheits-Papier, jetzt mit gemessenen
Zahlen.

### D. Mitgelieferte Maschine

Siehe oben. Wochen, 60 MB, eine Sprache. Verworfen.

## 5. Nebeneinander

| | A SAPI | B PCM → Kira | C B + Leser | D piper |
|---|---|---|---|---|
| Stimmen (hier) | Hedda, Zira | **Stefan, Katja, Hedda** | wie B, oder die des Lesers | eine, mitgeliefert |
| Latenz je Satz | ~100 ms bis Ton | 12–35 ms Synthese + Kira | wie B | ~200 ms + Modell laden |
| Bus, Pause, Ausblenden, räumlich | nein | **ja** | ja | ja |
| `SPEAK_SOUND`, WAV | nein | **ja** | ja | ja |
| Bildschirmleser-Nutzer | zweite Stimme | zweite Stimme | **seine Stimme** | zweite Stimme |
| Systeme | Windows | Windows gemessen, macOS/Linux per Prozess ungeprüft | wie B | alle |
| Hier prüfbar | Windows | Windows | Windows | ja |
| Aufwand | 1 Tag | 2–3 Tage | +1 Tag | Wochen |

## 6. Empfehlung

**C — also B, und der Bildschirmleser zuerst.**

B, weil die Messung die Frage entschieden hat: WinRT liefert die besseren
Stimmen (Stefan und Katja statt Hedda) **und** einen Klang statt eines
Lautsprecherausgangs, in 12 bis 35 Millisekunden. Ein Klang ist in
Drachenhauch ein `SOUND`, und damit ist eine gesprochene Zeile kein
Fremdkörper, sondern ein Ton unter Tönen: sie hat einen Bus, sie lässt sich
pausieren, an eine Figur hängen und als WAV ins Spiel einbacken. SAPI könnte
nichts davon.

C obendrauf, weil es einen Tag kostet und den einzigen Konflikt löst, den B
hat: wer einen Bildschirmleser benutzt, will nicht zwei Stimmen hören. Und
es schließt die letzte Lücke der Barrierefreiheit — ein Spiel auf der
Zeichenfläche kann dem Leser dann sagen, was passiert.

macOS und Linux bekommen denselben Weg über `say` und `espeak-ng`, ungeprüft
wie alles dort, mit klarem Fehler, wenn das Werkzeug fehlt.

**Reihenfolge:** B Windows mit Prüfstein (2 Tage) → C (1 Tag) →
macOS/Linux per Prozess (1 Tag, ungeprüft).

## 7. Was ohne Entscheidung schon geht

* Eine gui-Anwendung spricht über `GUI_ANNOUNCE` mit dem Bildschirmleser des
  Nutzers — seit 2026.13.
* Wer heute eine Stimme im Spiel braucht, nimmt Dateien: `LOADSOUND` mit
  vorher aufgenommenen Sätzen, gemischt wie jeder andere Ton.
* Der teure Notweg bleibt: `SHELL_START` mit PowerShell, 194 ms je Satz,
  ohne Anhalten.
