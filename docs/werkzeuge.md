# Die Werkzeuge um die Sprache

`dhrt` führt nicht nur aus. Diese Seite sammelt, was daneben steht: prüfen,
testen, formatieren, nachsehen was drin ist.

Ein Überblick über alle Unterbefehle steht in der Runtime selbst:

```bash
dhrt --help
```

## `dhrt --version` — was steckt in diesem Binary?

```text
$ dhrt --version
dhrt 2026.8
dabei: grafik, dialoge, datenbank, netz, http, seriell, usb, bluetooth, wlan
```

Die zweite Zeile ist kein Beiwerk. Ein Bau **ohne** `--hardware` lässt
`serial`, `usb`, `bt` und `wifi` weg, ohne dass man es dem Programm ansieht —
die Meldung kam bisher erst beim ersten Aufruf, tief im Programm:

```text
fehlt: seriell, usb, bluetooth, wlan (neu bauen mit: python rust/build_runtime.py --hardware)
```

Die Fassung ist dieselbe wie in `pyproject.toml` und
`drachenhauch/__init__.py`; ein Test hält die drei Angaben zusammen.

## `dhrt test` — Prüfprogramme laufen lassen

Die Bausteine gibt es seit WP E (`ASSERT`, `ASSERT_COLLECT`, `ASSERT_REPORT`
und der Rückgabewert über `EXIT`). Was fehlte, war das Dach:

```bash
dhrt test                    # unterhalb des aktuellen Verzeichnisses
dhrt test code/              # nur dort
dhrt test code/eins_pruefung.dh
```

```text
  ok      code\abruf_pruefung.dh  (0.82s)
  FEHLER  code\zeit_pruefung.dh  (Rueckgabewert 1, 0.03s)
          FEHLER: 1 von 12 Pruefungen
          FEHL  Zeile 44: Sommerzeit: erhalten 3600, erwartet 7200

2 Datei(en), 1 ok, 1 mit Fehlern  (0.85s)
```

**Gefunden wird `*_pruefung.dh`** — die Regel ist am Bestand abgelesen, nicht
erfunden: `buch-tippspiel/code/` nennt seine vier Prüfprogramme seit jeher so.
Eine ausdrücklich genannte Datei läuft auch ohne diese Endung; wer sie
hinschreibt, meint sie.

**Der Rückgabewert ist 0 nur, wenn alles durchlief** — damit taugt der Aufruf
für eine Kette oder eine CI. „Nichts gefunden" ist dabei *kein* Fehlschlag,
sonst fällt die Kette über ein noch leeres Projekt.

Zwei Dinge, die man wissen sollte:

* **Jede Datei läuft als eigener Prozess.** Derselbe Grund wie bei
  `TASK_START`: die Prozessgrenze ist die Zusage. Ein Prüfprogramm, das
  abstürzt, ein Fenster öffnet oder Globals hinterlässt, kann dem nächsten
  nichts antun.
* **Die Standardeingabe des Kindes ist leer.** Ein vergessenes `INPUT` würde
  sonst auf eine Eingabe warten, die nie kommt, und der ganze Lauf hängt.

Ein Prüfprogramm sieht so aus — abbrechend (die Vorgabe) oder sammelnd:

```basic
' Bricht beim ersten Fehlschlag ab:
ASSERT(punkte(3, 1, 3, 1) = 4, "exakter Tipp")
ASSERT_EQ(tendenz(2, 1), 1, "Heimsieg")

' ... oder alle Prüfungen laufen lassen und am Ende Bilanz ziehen:
ASSERT_COLLECT(TRUE)
ASSERT_EQ(a, b, "was geprüft wird")
ASSERT_REPORT()
IF ASSERT_FAILED() > 0 THEN EXIT(1)
```

### Prüfsammlungen: viele Fälle in einer Datei (`*.dhtest`)

Ein Prüfprogramm prüft sich selbst mit `ASSERT`. Die meisten Tests der
Sprache sind aber Ausgabevergleiche: ein kurzes Programm, eine erwartete
Ausgabe. Dafür braucht es kein Programm je Datei, sondern eine **Sammlung**
(seit 2026-09-07, Weg D des [Python-Abbaus](entwurf-python-abbau.md)):

```text
' Kopfkommentar bis zum ersten Fall
=== Zähler liefert drei Werte
FUNCTION z() AS INTEGER
    YIELD 1
    YIELD 2
END FUNCTION
DIM c AS COROUTINE : c = z()
PRINT CORO_RESUME(c)
PRINT CORO_RESUME(c)
--- erwartet
1
2
=== Division durch Null bricht ab
PRINT 1 \ 0
--- fehler
Division durch Null
```

`=== Name` beginnt einen Fall, dann folgt sein Quelltext, dann Abschnitte:

| Abschnitt | Bedeutung |
|---|---|
| `--- erwartet` | die Ausgabe, Zeile für Zeile; Leerzeilen am Blockende zählen nicht, ein Zeilenumbruch am Ende auch nicht |
| `--- enthaelt` | jede Zeile des Blocks muss in der Ausgabe vorkommen |
| `--- fehler` | das Programm muss abbrechen (Rückgabewert ungleich 0), und jede Zeile des Blocks steht in der Meldung |
| `--- datei name` | eine Beilage, die vor dem Lauf neben dem Programm liegt (JSON, Karte, Text) |
| `--- umgebung` | `NAME=WERT` je Zeile, etwa `DHRT_FRAMES=1` |

Ohne Erwartung gilt ein Fall als bestanden, wenn er mit 0 endet. **Jeder
Fall läuft als eigener Prozess in einem eigenen Verzeichnis**, die Fälle
einer Datei parallel; `dhrt test datei.dhtest --filter Text` lässt nur die
Fälle laufen, deren Name den Text enthält. Ein Fall, der an einer Maschine
ohne Bildschirm oder Soundkarte scheitert, gilt als übersprungen, nicht als
falsch; mit `DHRT_OHNE_GRAFIK=1` auch einer, dem im Bau ohne raylib ein
Grafik-Befehl fehlt. Die Bilanz nennt Dateien und Fälle:

```text
  ok      tests\pruef\coroutines.dhtest  (13 Faelle, 0.61s)
  FEHLER  tests\pruef\array_literal.dhtest  (1 von 23 Faellen, 0.90s)
          FEHL  Zeile 61: leeres Feld: Ausgabezeile 1: erwartet '0', erhalten '1'

2 Datei(en), 1 ok, 1 mit Fehlern; 36 Faelle, 35 ok, 1 fehl, 0 uebersprungen  (1.52s)
```

Die Sammlungen des Projekts liegen unter `tests/pruef/`; bis pytest ganz
fällt, lässt `tests/test_dhrt_test.py` sie in der CI mitlaufen.

## `dhrt fmt` — einheitlich schreiben

```bash
dhrt fmt datei.dh ...            # Schlüsselwörter groß, Leerraum am Ende weg
dhrt fmt --einruecken datei.dh   # zusätzlich neu einrücken
dhrt fmt --pruefen datei.dh      # schreibt nicht, meldet nur (Rückgabewert 1)
```

**Die Vorgabe ist verlustfrei.** Sie schreibt Schlüsselwörter groß und
entfernt Leerraum am Zeilenende — mehr nicht. Kein Umbruch, keine Einrückung,
nichts innerhalb einer Zeile. Drachenhauch ignoriert Groß-/Kleinschreibung,
also schreibt jeder anders (`If x Then`, `if x then`, `IF x THEN`); genau das
vereinheitlicht dieser Lauf, und zwar an den **Token-Positionen** des Lexers.
Ein `end` in einer Zeichenkette oder einem Kommentar bleibt darum unberührt.

Groß geschrieben wird das Wort, das dasteht (`elif` → `ELIF`), nicht sein
kanonischer Name (`ELSEIF`): vereinheitlicht wird die Schreibweise, nicht der
Wortschatz. Namen, Builtins und Klassen bleiben, wie sie sind.

**`--einruecken` ist bewusst nicht die Vorgabe.** Der Formatierer kennt nur
die Blöcke der *Sprache*. Eine von Hand gesetzte Gruppe, die die Sprache nicht
kennt, zieht er flach:

```basic
RENDERTARGET_BEGIN(badge)
    CLS(&H1A2438)             ' eingerückt, damit man die Gruppe sieht --
    CIRCLE(60, 52, 28, ...)   ' für die Sprache sind das gewöhnliche Aufrufe
RENDERTARGET_END()
```

Beides ist in `examples/` echt vorgekommen (dort auch ein unter seinem
Vorgänger ausgerichteter Kommentar). Ein Werkzeug, das die
Gliederungsabsicht seines Nutzers überschreibt, darf das nicht nebenbei tun —
also nur, wenn man es ausdrücklich verlangt.

Was `--einruecken` kann: eine Einheit von vier Leerzeichen je Ebene,
`CASE` innerhalb von `SELECT` (der Hausstil), `ELSE`/`ELSEIF`/`CATCH`/
`FINALLY` eine Ebene links von ihrem Rumpf, einzeilige `IF … THEN …` ohne
Einrückung, `ENUM` nur im Blockform-Fall. **Fortsetzungszeilen (`_`) bleiben
unangetastet** — wer seine Parameter untereinander ausrichtet, hat sich etwas
dabei gedacht.

**Eine Datei mit Syntaxfehler wird nicht angefasst.** An kaputtem Code
herumzurücken hilft niemandem; der Aufruf meldet es und lässt die Datei in
Ruhe.

## `dhrun.py --doku` — Referenz aus dem Quelltext

Wer eine eigene Bibliothek schreibt (`zeitraum.dh`, `tabellen.dh`), soll sie
nicht ein zweites Mal von Hand beschreiben müssen — eine handgepflegte
Referenz driftet ab dem ersten Tag.

```bash
dhrun.py --doku mathe.dh                     # nach stdout
dhrun.py --doku lib/*.dh -o docs/referenz.md # in eine Datei
```

Genommen wird die **Signatur** und der **Kommentarblock direkt darüber** —
dieselbe Quelle, aus der auch der Hover im Editor kommt
(`editor_qt/symbols.py`); es gibt keine zweite Vorstellung davon, was eine
Signatur ist. Ein Kommentarblock ganz am Dateianfang beschreibt die Datei
selbst.

```basic
' Abstand zweier Punkte in der Ebene.
' Liefert immer einen positiven Wert.
FUNCTION Distanz(x1 AS FLOAT, y1 AS FLOAT, x2 AS FLOAT, y2 AS FLOAT) AS FLOAT
    RETURN SQR((x2 - x1) ^ 2 + (y2 - y1) ^ 2)
END FUNCTION
```

wird zu einem Abschnitt mit Signatur und beiden Sätzen.

**`PRIVATE` bleibt draußen.** Es gehört dem Modul; eine Referenz, die es
auflistet, verspricht etwas, das beim nächsten Umbau verschwindet.

**Warum `dhrun.py` und nicht `dhrt`:** der Rust-Lexer wirft Kommentare weg
(er braucht sie nicht), der Python-Lexer behält die Zeilen. Eine Doku ohne
Kommentare wäre eine Liste von Signaturen — und die kann man auch selbst
lesen.

## `dhrt --check` — übersetzen ohne auszuführen

```bash
dhrt --check datei.dh [weitere.dh ...]
```

Gibt gefundene Probleme als JSON aus (leer = sauber) — das ist, was der
Editor live beim Tippen anzeigt. Der Rückgabewert ist auch bei Funden 0; so
unterscheidet der Editor „Probleme gefunden" von „Werkzeug kaputt".

Was dabei gefunden wird, steht in
[Sprachreferenz → Was der Übersetzer prüft](sprache.md#was-der-übersetzer-prüft).
