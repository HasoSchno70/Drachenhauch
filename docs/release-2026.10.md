# Drachenhauch 2026.10

*Die Notizen zu dieser Fassung. Warum die einzelnen Stücke so und nicht anders
gebaut wurden, steht in [Punkt 8 des zweiten Allzweck-Audits](allzweck-audit-2.md)
und in den jeweiligen Modul-Seiten.*

## Neu in dieser Fassung

Eine kleine Fassung mit einem Thema: **Stellen, an denen genau eine Hälfte
fehlte.** Sieben Commits, entstanden aus einer Bestandsaufnahme der Runtime —
was kann der Dialekt inzwischen, und wo fehlt noch das Gegenstück zu etwas, das
längst da ist?

**Objekte können sich selbst als Rückruf eintragen** — nachgeliefert für den
Fall, der in 2026.9 noch aufgefallen war. Neun Stellen der Laufzeit verlangen
eine `FUNCREF`, und die konnte nur eine benannte Funktion auf oberster Ebene
sein; objektorientierter Code brauchte für jeden Knopf eine globale Variable
daneben. Jetzt trägt `obj.methode` ihre Instanz mit:

```basic
GUI_ON_CLICK(knopf, spieler.klick)
TIMER_EVERY(500, gegner.zucken)
SORT(zahlen, regel.cmp)
```

Aufgelöst wird beim **Aufruf**, nicht beim Binden — eine als Elternklasse
gehaltene Instanz ruft also die Überschreibung des Kindes. Ein Feld gewinnt vor
einer gleichnamigen Methode, sonst hätte die Erweiterung bestehenden Code
umgedeutet.

**Und man kann fragen, was man in der Hand hält.** Polymorphie funktionierte
längst; ansehen konnte man einer Referenz nichts. `TYPEOF` sagte pauschal
`"OBJECT"`, einen Typtest gab es gar nicht:

```basic
PRINT TYPEOF(t)      ' "HUND" statt "OBJECT"
PRINT t IS Hund      ' TRUE
PRINT t IS Tier      ' TRUE  -- jede Elternklasse trifft
PRINT t IS NOT NIL   ' TRUE
```

Rechts von `IS` steht ein Typname, kein Ausdruck. Ein unbekannter Name ist ein
**Übersetzungsfehler** — ein Tippfehler wäre sonst still für immer `FALSE`, und
ein Test, der nie zuschlägt, fällt niemandem auf. Nebenbei erledigt: `IS NIL`
und `IS NOT NIL` standen in der Dokumentation ausdrücklich als *nicht
vorhanden*.

**Drei Halbheiten in der Sprache geschlossen.** Bei allen dreien gab es das
Gegenstück schon:

```basic
m = {"a": 1, "b": 2}      ' MAP-Literal -- das Gegenstück zu [1, 2, 3]
m = {}                    ' die leere Map

FOR EACH k, v IN m        ' Schlüssel UND Wert, nicht nur die Schlüssel
    PRINT k; "="; v
NEXT

DO WHILE i < 5            ' die geläufigste der drei Schleifenformen
    i++
LOOP
```

`DO … LOOP` gibt es in allen fünf Formen (Bedingung oben, unten oder gar
nicht). Keine der drei brauchte neue Laufzeit-Mechanik: das MAP-Literal nutzt
dieselbe Sammelstelle wie die Dict-Comprehension, `FOR EACH` mit zwei Variablen
einen Vorschalt-Aufruf, und `DO … LOOP` wird im Parser zu `WHILE` bzw. `REPEAT`
— damit erben beide Formen `BREAK` und `CONTINUE` ohne Zutun.

`do` und `loop` sind dabei **kontextuell** und bleiben gewöhnliche Bezeichner.
`examples/127_filedialog.dh` hat ein `DIM dO AS INTEGER`, das ein echtes
Schlüsselwort gebrochen hätte.

**Ein Clip-Rechteck fürs Zeichnen.** Für eigene scrollbare Flächen, Minikarten
oder geteilte Bildschirme gab es keinen begrenzten Zeichenbereich; wer einen
brauchte, legte ein Render-Target an — eine zweite Zeichenfläche samt Speicher,
wo ein Rechteck genügt.

```basic
SCISSOR(20, 40, 200, 120)
' … alles hier wird auf das Rechteck beschnitten
SCISSOR_END()
```

Die Mechanik lag die ganze Zeit da (das `gui`-Modul clippt seine Fenster
damit) — nur herausgeführt war sie nie. Es ist ein **Stapel**: ein inneres
`SCISSOR` wird mit dem äußeren *geschnitten*, es ersetzt es nicht.

**Ein Instrument anschließen.** Drachenhauch bringt Tracker, Sampler,
sfxr-Synth, Notenblatt-Editor und Kira-Busse mit — bis hierher konnte kein
angeschlossenes Keyboard etwas davon ansteuern. Das neue Modul
[`midi`](module-midi.md) schließt diese Brücke, mit 27 Befehlen zum Auflisten,
Empfangen (Cursor-Muster wie `db`/`mqtt`) und Senden:

```basic
IMPORT "midi"
DIM tastatur AS MIDI_IN
tastatur = MIDI_IN_OPEN(0)
WHILE MIDI_NEXT(tastatur)
    IF MIDI_IS_NOTE_ON(tastatur) THEN
        AUDIO_TONE(MIDI_NOTE_FREQ(MIDI_NOTE(tastatur)), 300, "sine")
    END IF
WEND
```

Zwei Eigenheiten des Protokolls nimmt das Modul einem ab: die meisten
Instrumente schicken **kein „Note aus"**, sondern ein *Note an* mit
Anschlagstärke 0 (`MIDI_IS_NOTE_OFF` fängt beide Formen — wer selbst prüft,
bekommt Töne, die nie aufhören), und Kanäle zählen nach außen **1…16** wie auf
jedem Gerätedisplay statt 0…15 wie im Protokoll.

Wie `serial`/`usb`/`bt`/`wifi` steckt es nur in `--hardware`-Bauten; auf Linux
braucht der Unterbau die ALSA-Header. **Zwei Befehle aber nicht:**
`MIDI_NOTE_NAME$` (60 → `"C4"`) und `MIDI_NOTE_FREQ` (69 → 440.0) rechnen nur
um und stehen in *jedem* Bau — wer eine Notenanzeige baut, braucht dafür kein
Gerät.

**Kleinkram mit Wirkung.** `ENUM_NAME(Zustand, 1)` führt vom Wert zurück zum
Namen — für Fehlersuche und Speicherstände; bisher ging ein ENUM nur in eine
Richtung. Und für plus/minus eins gibt es `i++` / `i--`; beides ist **nur eine
Anweisung, kein Ausdruck** (`j = i++` gibt es nicht), weil der Unterschied
zwischen Präfix- und Postfix-Form die häufigste Fehlerquelle an diesem Operator
ist.

**Das Einsteigerbuch liegt jetzt im Installer.** In 2026.9 war *Der Einstieg*
das Aushängeschild der Fassung — und im Paket fehlte er. Jetzt sind beide
Bücher dabei, mit eigenem Startmenü-Eintrag.

## Unter der Haube

**Zwei Testfehler, die dieselbe Ursache hatten.** Beide Male lief lokal alles
grün und die CI nicht — und beide Male, weil ein Test seine *eigene* Umgebung
für die einzig mögliche hielt.

Der erste: zwei GUI-Tests brauchen ein Fenster, liefen hier aber gegen einen
`dhrt` **mit** Grafik. Die posix-CI baut ohne raylib, die Windows-CI hat keinen
Bildschirm — für beides gibt es im Projekt einen Wächter, und der Test hatte
beide umgangen.

Der zweite: der MIDI-Test fragt `dhrt --version`, ob dieser Bau das Feature
hat — und suchte den Namen im *ganzen* Text. Die Ausgabe hat aber zwei Zeilen,
`dabei:` und `fehlt:`; die Antwort war also immer ja. Nachgeprüft wird seither
mit einem eigens gebauten hardware-freien `dhrt`, nicht mit dem vorhandenen.

Die Testsuite ist von 3792 auf **3799** Prüfungen gewachsen, davon 48 neue für
die Spracherweiterungen. Drei davon prüfen am **gerenderten Bild**: dass ein
Clip gesetzt wurde, sagt nichts darüber, ob auch etwas abgeschnitten wird.

Die Befehlsreferenz zählt 1558 Einträge (31 mehr), die Modulliste 47.

## Was offen bleibt

Der **Notenfluss von einem echten Keyboard** ist ungeprüft — an der
Entwicklungsmaschine hängt keines. Der Sendepfad dagegen läuft nachweislich:
Windows bringt einen MIDI-Ausgang mit (*GS Wavetable Synth*), und im Test geht
wirklich ein Dreiklang zum Synthesizer und wieder aus. Rückmeldungen von
jemandem mit Instrument sind willkommen.
