# Modul `zeit`

Rechnen mit Datum und Uhrzeit. `DATE$()` und `TIME$()` liefern Text — gut zum
Anzeigen, unbrauchbar zum Rechnen. Sobald ein Programm „15 Minuten vor
Anpfiff", „noch 2:15 h" oder „welcher Wochentag ist das" wissen will, braucht
es Zahlen. Genau das macht dieses Modul: Text rein, Zahl raus, rechnen, Text
zurück.

```basic
IMPORT "zeit"
```

## Das Modell: ein Zeitpunkt ist eine Zahl

Ein Zeitpunkt ist ein `INTEGER` — **Sekunden seit dem 1. Januar 1970**, in
**Ortszeit**. Mehr steckt nicht dahinter, und das ist der ganze Trick:

- **Später** heißt *größer*. `IF anstoss > jetzt THEN` ist der ganze Vergleich.
- **Rechnen** heißt Addieren. 15 Minuten sind `15 * 60`, ein Tag ist `86400`.
- **Kein Sonderfall.** Monatsenden, Jahreswechsel und Schaltjahre stecken in
  der Umrechnung, nicht in deinem Programm.

Ortszeit heißt: `ZEIT_TEXT$(ZEIT_JETZT())` zeigt dieselbe Uhrzeit wie deine
Armbanduhr, und `DATE$() + " " + TIME$()` liefert denselben Text. Beim Rechnen
über eine Zeitumstellung hinweg zählt das Modul echte Sekunden, nicht
Zifferblatt-Stunden.

## Text und Zeitpunkt

| Funktion | Wirkung |
|---|---|
| `ZEIT_JETZT()` → INTEGER | jetzt, als Zeitpunkt |
| `ZEIT_PARSE(text$)` → INTEGER | Text zu Zeitpunkt; bricht bei Unlesbarem ab |
| `ZEIT_LESBAR(text$)` → BOOLEAN | ließe sich der Text lesen? (fragt, bricht nicht ab) |
| `ZEIT_TEXT$(zeit)` → STRING | Zeitpunkt als `JJJJ-MM-TT hh:mm:ss` |
| `ZEIT_AUS_TEILEN(jahr, monat, tag [, stunde, minute, sekunde])` → INTEGER | Zeitpunkt aus Einzelwerten |

`ZEIT_PARSE` versteht die Schreibweisen, die einem in Datenbanken und Web-APIs
begegnen:

```basic
IMPORT "zeit"

PRINT ZEIT_TEXT$(ZEIT_PARSE("2026-08-28 20:30:00"))   ' Normalform
PRINT ZEIT_TEXT$(ZEIT_PARSE("2026-08-28T20:30:00"))   ' ISO mit T
PRINT ZEIT_TEXT$(ZEIT_PARSE("2026-08-28T20:30:00Z"))  ' mit Zeitzonen-Anhang
PRINT ZEIT_TEXT$(ZEIT_PARSE("2026-08-28 20:30"))      ' ohne Sekunden
PRINT ZEIT_TEXT$(ZEIT_PARSE("2026-08-28"))            ' nur Datum = 00:00:00
```

Unlesbares bricht mit Klartext ab statt still `-1` zu liefern — ein kaputtes
Datum fällt sonst erst viel später als unsinniges Ergebnis auf. Wo Eingaben
unsicher sind, erst fragen:

```basic
IMPORT "zeit"

DIM eingabe AS STRING
eingabe = "naechsten Dienstag"
IF ZEIT_LESBAR(eingabe) THEN
    PRINT ZEIT_TEXT$(ZEIT_PARSE(eingabe))
ELSE
    PRINT "Bitte als JJJJ-MM-TT hh:mm eingeben."
END IF
```

`ZEIT_TEXT$` und `ZEIT_PARSE` sind Umkehrungen voneinander. Die Normalform ist
dieselbe, die SQLite als Textspalte richtig sortiert — Zeitpunkte lassen sich
also so in die Datenbank schreiben und wieder herausrechnen.

## Rechnen

| Funktion | Wirkung |
|---|---|
| `ZEIT_PLUS(zeit, sekunden)` → INTEGER | Sekunden dazu (negativ = zurück) |
| `ZEIT_DIFF(zeit, frueher)` → INTEGER | Abstand in Sekunden (negativ, wenn `zeit` früher liegt) |
| `ZEIT_DAUER$(sekunden)` → STRING | Sekunden als lesbare Spanne |

```basic
IMPORT "zeit"

DIM anstoss AS INTEGER
anstoss = ZEIT_PARSE("2026-08-28 20:30:00")

' Tippschluss: eine Viertelstunde vor Anpfiff
DIM schluss AS INTEGER
schluss = ZEIT_PLUS(anstoss, -15 * 60)
PRINT "Tippschluss: "; ZEIT_TEXT$(schluss)

' Darf noch getippt werden?
IF ZEIT_JETZT() < schluss THEN
    PRINT "Noch "; ZEIT_DAUER$(ZEIT_DIFF(schluss, ZEIT_JETZT())); " Zeit."
ELSE
    PRINT "Tippschluss ist vorbei."
END IF
```

`ZEIT_DAUER$` wählt die Einheit nach der Größe, damit die Anzeige nicht in
Sekundenzahlen ertrinkt:

| Sekunden | Ausgabe |
|---|---|
| `45` | `45 s` |
| `720` | `12 min` |
| `8100` | `2:15 h` |
| `86400` | `1 Tag` |
| `259200` | `3 Tage` |
| `-3600` | `vor 1:00 h` |

Negative Werte bekommen ein `vor` — dieselbe Funktion beantwortet damit „noch
wie lange" und „wie lange her".

## Teile lesen und anzeigen

| Funktion | Wirkung |
|---|---|
| `ZEIT_TEIL(zeit, feld$)` → INTEGER | ein Feld: `jahr`, `monat`, `tag`, `stunde`, `minute`, `sekunde` |
| `ZEIT_WOCHENTAG(zeit)` → INTEGER | 1 = Montag … 7 = Sonntag |
| `ZEIT_FORMAT$(zeit, muster$)` → STRING | Zeitpunkt nach Muster |

Die Muster stehen in derselben Sprache wie die Anzeige selbst:

| Muster | Bedeutung | Beispiel |
|---|---|---|
| `JJJJ` | Jahr, vierstellig | `2026` |
| `MM` | Monat, zweistellig | `08` |
| `TT` | Tag, zweistellig | `28` |
| `hh` | Stunde, zweistellig | `20` |
| `mm` | Minute, zweistellig | `30` |
| `ss` | Sekunde, zweistellig | `00` |
| `WT` | Wochentag kurz | `Fr` |
| `WTAG` | Wochentag ausgeschrieben | `Freitag` |

```basic
IMPORT "zeit"

DIM a AS INTEGER
a = ZEIT_PARSE("2026-08-28 20:30:00")

PRINT ZEIT_FORMAT$(a, "TT.MM.JJJJ")             ' 28.08.2026
PRINT ZEIT_FORMAT$(a, "WT TT.MM. hh:mm")        ' Fr 28.08. 20:30
PRINT ZEIT_FORMAT$(a, "WTAG, TT.MM.JJJJ hh:mm") ' Freitag, 28.08.2026 20:30
PRINT ZEIT_WOCHENTAG(a)                         ' 5
```

Ohne Muster (`""`) kommt die Normalform heraus, genau wie bei `ZEIT_TEXT$`.

`ZEIT_WOCHENTAG` zählt ab Montag, weil ein Spieltag am Wochenende sonst über
zwei Zahlen läuft: Montag 1 … Sonntag 7.

## Beispiel: Countdown im Fenster

Ein Zeitpunkt als Zahl, jede Sekunde neu angezeigt — mehr braucht ein
Countdown nicht.

```basic
IMPORT "zeit"

DIM anstoss AS INTEGER
anstoss = ZEIT_PARSE("2026-08-28 20:30:00")

SCREEN(400, 120, "Anpfiff")
WHILE NOT QUITREQUESTED()
    CLS(RGB(20, 24, 32))
    DIM rest AS INTEGER
    rest = ZEIT_DIFF(anstoss, ZEIT_JETZT())
    IF rest > 0 THEN
        TEXT(20, 30, "Anpfiff in " + ZEIT_DAUER$(rest), RGB(240, 240, 240))
    ELSE
        TEXT(20, 30, "Laeuft seit " + ZEIT_DAUER$(-rest), RGB(120, 220, 140))
    END IF
    TEXT(20, 60, ZEIT_FORMAT$(anstoss, "WTAG, TT.MM.JJJJ hh:mm"), RGB(160, 170, 190))
    FLIP()
WEND
```

## Fallstricke

- **Zeitpunkte sind Ortszeit.** Ein Zeitpunkt, der auf einem Rechner in Berlin
  entsteht, ergibt auf einem Rechner in Tokio dieselbe *Zahl* nur, wenn beide
  denselben Text lesen — und dieselbe Zahl zeigt dort eine andere Uhrzeit.
  Wer über Zeitzonen hinweg vergleicht, speichert den Text, nicht die Zahl.
- **`MILLIS()` ist etwas anderes und lässt sich nicht umrechnen.** Es ist die
  Stoppuhr für Frame-Zeiten und Timer. Wer `MILLIS() / 1000` mit
  `ZEIT_JETZT()` vergleicht, liegt um den Zeitzonen-Versatz daneben —
  gemessen am 16.08.2026 in Mitteleuropa: `MILLIS()/1000 = 1786883970`,
  `ZEIT_JETZT() = 1786891170`, also **zwei Stunden Unterschied**. Für Datum
  und Uhrzeit immer `ZEIT_JETZT()`.
- **Ein Tag ist nicht immer 86400 Sekunden.** An den Umstellungstagen sind es
  23 oder 25 Stunden. `ZEIT_PLUS(t, 86400)` addiert exakt 86400 Sekunden — das
  ist an diesen zwei Tagen im Jahr nicht dieselbe Uhrzeit am Folgetag. Wenn es
  auf die Uhrzeit ankommt, über `ZEIT_TEIL`/`ZEIT_AUS_TEILEN` gehen.
- **Vor 1970** sind Zeitpunkte negativ. Das Rechnen stimmt trotzdem.

## In der nativen Runtime (dhrt)

`zeit` ist vollständig in Rust umgesetzt (`rust/drachenhauch_runtime/src/zeit.rs`),
ohne zusätzliche Abhängigkeit und ohne Cargo-Feature — das Modul ist in jedem
Build dabei, auch in konsolenreinen. Die Umrechnung Datum ↔ Tage rechnet mit
dem bürgerlichen Kalender (Schaltjahr-Regel inklusive der 100/400-Ausnahme).
