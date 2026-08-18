# Entwurf: Namensräume (WP I)

> **Status: Entwurf, nichts davon ist gebaut.** Dieses Papier ist zum Lesen und
> Entscheiden da. WP I ist der einzige Punkt der [Allzweck-Roadmap](allzweck-roadmap.md),
> der auf **bestehenden** Code zurückwirkt — deshalb erst der Entwurf, dann
> Code.
>
> Die Beispiele stehen bewusst als ```` ```text ```` und nicht als
> ```` ```basic ````: sie zeigen **vorgeschlagene** Syntax, die es noch
> nicht gibt. `tools/pruef_docs.py` prüft alle `basic`-Blöcke in `docs/`
> gegen den echten Compiler — und soll hier zu Recht anschlagen, wenn
> jemand sie für gültig hält.

## 1. Der Befund

`IMPORT "datei.dh"` fügt Text ein. Alles, was in der Datei steht, landet danach
im selben flachen Namensraum wie das Hauptprogramm — und wie die 1401
eingebauten Namen.

Vier Messungen an der heutigen Runtime, damit die Diskussion an Tatsachen
hängt und nicht an Eindrücken:

**(a) Zwei Bibliotheken mit gleichem Namen gehen nicht zusammen.**

```text
' a.dh und b.dh haben beide FUNCTION Hilf()
IMPORT "a.dh"
IMPORT "b.dh"
```
```
m.dh: Compile-Fehler: 'hilf' bereits deklariert
```

Der Fehler ist da — gut. Aber er nennt **keine Zeile** und **keine der beiden
Dateien**. Wer zwei fremde Bibliotheken benutzt, weiß nicht, welche.

> ✅ **Behoben in I.4.** Die Meldung lautet jetzt
> `b.dh:1: Compile-Fehler: 'hilf' ist schon in a.dh:1 deklariert -- eines von
> beiden umbenennen`.

**(b) Ein Modul sieht die Variablen des Hauptprogramms.**

```text
' c.dh:  FUNCTION Liest() AS INTEGER : RETURN geheim : END FUNCTION
DIM geheim AS INTEGER
geheim = 42
IMPORT "c.dh"
PRINT Liest()          ' -> 42
```

Das ist keine Kapselung, das ist ein gemeinsamer Topf. Eine Bibliothek kann
sich unbemerkt auf eine Variable des Hauptprogramms stützen — und bricht, wenn
die umbenannt wird.

**(c) Es gibt kein Privat.** Jeder Helfer einer Bibliothek ist Teil ihrer
Schnittstelle, ob gewollt oder nicht. Wer ihn umbenennt, bricht fremden Code,
von dem er nichts weiß.

**(d) Zeilennummern zeigen in die gemergte Quelle.** Ein Zweizeiler:

```text
IMPORT "d.dh"
DIM zaehler AS STRING
```
```
m3.dh:6: Warnung: 'zaehler' wurde in Zeile 2 schon als INTEGER angelegt …
```

Zeile 6 in einer Datei mit zwei Zeilen. Der Editor rechnet das über die
`origins`-Tabelle zurück (`editor_qt/error_check.py`), `dhrt` auf der
Kommandozeile nicht.

> ✅ **Behoben in I.4.** `preprocess` liefert die Herkunftstabelle jetzt selbst
> mit; alle Meldungen der Übersetzungs-Phasen (Lexer, Parser, Compiler,
> Warnungen) zeigen auf Datei und Zeile, die der Nutzer vor sich hat:
> `m3.dh:2: Warnung: 'zaehler' wurde in d.dh:1 schon als INTEGER angelegt …`.
> Damit ist zugleich **Schritt 1 des Bauwegs** aus Abschnitt 4 erledigt.

**Was daraus folgt:** unterhalb von ein paar tausend Zeilen fällt nichts davon
auf. Darüber ist es die Hauptbremse — und eine Bibliothek von jemand anderem
kann man heute nicht gefahrlos einbinden.

## 2. Was nicht verhandelbar ist

Diese vier Punkte bestimmen den Entwurf mehr als alles andere:

1. **Bestehender Code muss weiterlaufen.** `examples/21_modules/` und jedes
   Programm da draußen. Ein Umstieg, der alte Programme bricht, ist kein
   Umstieg.
2. **Die Sprache ignoriert Groß-/Kleinschreibung.** `Mathe.Distanz` und
   `mathe.distanz` sind dasselbe. Alles wird kleingeschrieben verglichen.
3. **Der Punkt ist schon vergeben.** `obj.feld`, `Enum.MITGLIED`,
   `Klasse.KONSTANTE` — ein Modulpräfix `mathe.Distanz` sieht syntaktisch
   genauso aus. Das muss der Compiler auseinanderhalten.
4. **Die 1401 Builtins bleiben flach.** Sie in Namensräume zu sortieren wäre
   eine zweite, größere Entscheidung mit noch mehr Rückwirkung. Nicht hier.

## 3. Der Vorschlag

### 3.1 Syntax

```text
IMPORT "mathe.dh"              ' wie bisher: alles wird eingemischt
IMPORT "mathe.dh" AS mathe     ' NEU: erreichbar als mathe.<Name>
```

**Ohne `AS` ändert sich nichts.** Das ist der ganze Trick der Verträglichkeit:
alter Code hat kein `AS`, also verhält er sich exakt wie heute. Wer
Namensräume will, schreibt sie hin.

In der Datei selbst regelt `PRIVATE`, was nach außen sichtbar ist:

```text
' mathe.dh
CONST DEG2RAD AS FLOAT = 0.017453292519943295

FUNCTION Distanz(x1 AS FLOAT, y1 AS FLOAT, x2 AS FLOAT, y2 AS FLOAT) AS FLOAT
    RETURN SQR(quadrat(x2 - x1) + quadrat(y2 - y1))
END FUNCTION

PRIVATE FUNCTION quadrat(x AS FLOAT) AS FLOAT     ' nur innerhalb von mathe.dh
    RETURN x * x
END FUNCTION
```

```text
' Hauptprogramm
IMPORT "mathe.dh" AS mathe

PRINT mathe.Distanz(0.0, 0.0, 3.0, 4.0)   ' 5.0
PRINT mathe.DEG2RAD                        ' geht auch für Konstanten
PRINT mathe.quadrat(3.0)                   ' Fehler: 'quadrat' ist PRIVATE in mathe.dh
```

`PRIVATE` wird — wie `ABSTRACT` und `SUPER` in WP G — **kein reserviertes
Wort**, sondern über eine Vorausschau erkannt (`private` gefolgt von
`SUB`/`FUNCTION`/`DIM`/`CONST`/`CLASS`). Ein neues Schlüsselwort würde
`DIM private AS …` in bestehendem Code zum Fehler machen, und die Liste der
reservierten Wörter ist in [sprache.md](sprache.md) schon als Stolperstein
dokumentiert.

### 3.2 Semantik

| Frage | Antwort |
|---|---|
| Was sieht ein Modul mit `AS`? | Builtins, seine eigenen Namen, die Namen **seiner eigenen** Imports. **Nicht** die Globals des Hauptprogramms. |
| Was sieht das Hauptprogramm? | Alles Eigene, plus `alias.Name` für jedes nicht-private Top-Level-Ding. |
| Wann läuft der Top-Level-Code des Moduls? | An der Stelle des `IMPORT`, wie heute. |
| Zweimal dasselbe Modul mit verschiedenen Aliasen? | Erlaubt, es wird **einmal** eingebunden; beide Aliase zeigen darauf. |
| Zwei Module mit gleichem Alias? | Fehler beim Übersetzen. |
| Kollidieren Namen aus zwei Modulen mit `AS`? | Nein — das ist der Punkt. |
| Was ist mit `IMPORT "json" AS j`? | Unverändert (Präfix-Kopieren für eingebaute Module, siehe CLAUDE.md). Der neue Weg gilt nur für `.dh`-Dateien. |

Der Punkt, an dem sich Aufwand und Nutzen entscheiden, ist Zeile 1 der
Tabelle: **ein Modul mit `AS` sieht die Globals des Hauptprogramms nicht
mehr.** Genau das macht eine Bibliothek weitergabefähig — und genau das kann
bestehenden Code brechen, der sich (wie in Messung (b)) darauf verlässt.
Deshalb nur mit `AS`, nie automatisch.

## 4. Wie es gebaut würde

Der naheliegende Weg wäre, den Textmerge abzuschaffen und jede Datei einzeln zu
parsen. Das wäre ein Umbau von `preprocess` → `parser` → `compiler` und würde
die `origins`-Rückrechnung des Editors mitreißen.

**Der billigere Weg: Umbenennen zur Übersetzungszeit.** Alles bleibt, wie es
ist — nur die Namen bekommen intern ein Präfix.

```
mathe.dh: FUNCTION Distanz(...)        ->  intern: mathe§distanz
Aufrufstelle: mathe.Distanz(...)       ->  intern: mathe§distanz
mathe.dh intern: quadrat(x)            ->  intern: mathe§quadrat
```

Vier Schritte:

1. **`preprocess` liefert zusätzlich eine Zeilen-Herkunftstabelle.** Die
   Schleife in `preprocess.rs::process_inner` weiß bei jeder ausgegebenen
   Zeile, aus welcher Datei sie stammt — heute wirft sie das weg. Sie gibt
   künftig ein `Vec<Option<ModulId>>` parallel zur gemergten Quelle mit aus.
   (Der Editor baut sich dieselbe Tabelle in `error_check.py` schon selbst;
   hier entsteht sie an der richtigen Stelle.)
2. **Ein Umbenennungs-Durchgang über das geparste AST.** Jedes Statement trägt
   seine Zeile (`Node::Stmt { line, body }`) — daraus ergibt sich sein Modul.
   Der Durchgang benennt Deklarationen und Referenzen innerhalb eines Moduls
   um und trägt die exportierten Namen in eine Tabelle `alias -> { Name }` ein.
3. **`mathe.Distanz` an der Aufrufstelle auflösen.** Der Compiler unterscheidet
   heute schon, ob ein `x.y` eine Variable, ein Enum oder ein
   Klassen-Namensraum ist (`Value::Namespace`, `_ClassStaticNamespace`).
   Modul-Aliase kommen als vierte Art dazu — und zwar **rein statisch**, es
   entsteht keine Laufzeitsuche.
4. **`PRIVATE` ist nur ein Häkchen** an der Deklaration: umbenannt wird sie
   trotzdem, sie steht bloß nicht in der Export-Tabelle.

**Was dieser Weg NICHT anfasst:** die VM, das Bytecode-Format, den
`.dhc`-Export, die Zeilennummern, die `origins`-Rückrechnung, den Debugger,
den Profiler. Kein neuer Opcode. Das ist der Grund, ihn zu wählen.

**Was er kostet:** Namen im Bytecode werden länger (`mathe§distanz` statt
`distanz`) — für Fehlermeldungen muss beim Ausgeben zurückübersetzt werden,
sonst liest der Nutzer das Präfix. Das ist eine Stelle, aber sie muss man
finden: `unknown_builtin_msg`, „bereits deklariert", die Namens-Kollisions-
Meldungen.

## 5. Der Stufenplan

| Stufe | Inhalt | Risiko |
|---|---|---|
| **I.1** | `IMPORT … AS` für `SUB`/`FUNCTION`/`CONST`/`DIM` auf Top-Level, plus `PRIVATE`. Klassen/Enums/Structs aus einem Namensraum: klarer Fehler „noch nicht erreichbar". | gering — nichts Bestehendes ändert sich |
| **I.2** | Klassen und Structs: `DIM p AS mathe.Punkt`, `NEW mathe.Punkt(...)`. Berührt die Typauflösung (`is_value_type`, `unknown_dim_type_msg`). | mittel |
| **I.3** | Enums: `mathe.Farbe.ROT` — zwei Punkte hintereinander. Braucht eine Entscheidung, ob das lesbar genug ist. | mittel |
| **I.4** | ✅ **erledigt** — Kollisionen nennen **beide** Dateien und Zeilen, und alle Meldungen der Übersetzungs-Phasen zeigen auf die Datei des Nutzers (behebt (a) und (d)). Brachte nebenbei Schritt 1 des Bauwegs mit: die Herkunftstabelle. | gering |

**I.4 ist gebaut** (2026-08-17) — klein, hilft sofort auch ohne Namensräume,
und es ist genau die Meldung, die man beim Bauen von I.1 dauernd sehen wird.
Schritt 1 des Bauwegs (die Herkunftstabelle aus `preprocess`) fiel dabei als
Nebenprodukt an und steht für I.1 bereit.

**Nächster Schritt wäre I.1** — dafür sind vorher die Fragen in Abschnitt 6 zu
entscheiden.

## 6. Was noch zu entscheiden ist

Diese Fragen sind nicht technisch, sondern Geschmack — und deine:

1. **Alias erzwingen oder ableiten?** `IMPORT "mathe.dh" AS mathe` ist doppelt
   gemoppelt. Alternative: `IMPORT NAMESPACE "mathe.dh"` leitet den Alias aus
   dem Dateinamen ab. Kürzer, aber ein zweites Schlüsselwort — und der
   Dateiname ist nicht immer ein guter Bezeichner (`mein-modul.dh`).
   *Neigung: beim ausdrücklichen `AS` bleiben.*
2. **`PRIVATE` oder umgekehrt `EXPORT`?** Privat als Vorgabe (also alles
   verstecken, was nicht `EXPORT` trägt) wäre strenger und sicherer — aber es
   dreht die Bedeutung bestehender Dateien um, sobald jemand ein `AS`
   dazuschreibt. *Neigung: `PRIVATE`, also öffentlich als Vorgabe.*
3. **Soll ein Modul mit `AS` die Globals des Hauptprogramms wirklich nicht
   sehen?** Das ist der eigentliche Nutzen — und die einzige Stelle, an der
   sich beim Anfügen eines `AS` das Verhalten ändert. *Neigung: ja, und in der
   Doku fett.*
4. **Wie tief soll I gehen?** Bis I.1 ist es überschaubar. Ab I.2 wird es ein
   Umbau der Typauflösung. Es ist völlig vertretbar, bei I.1 + I.4 stehen zu
   bleiben und Klassen weiter flach zu halten.

## 7. Verworfene Alternativen

- **Ein eigener Trenner (`mathe::Distanz`).** Vermeidet die Mehrdeutigkeit mit
  dem Punkt, kostet ein neues Token und sieht nicht nach BASIC aus. Die
  Mehrdeutigkeit ist ohnehin lösbar, weil der Compiler die Aliase kennt.
- **Getrennte Übersetzung (`.dhc` je Modul, Linker).** Wäre der „richtige"
  Weg und die Voraussetzung für eine Paketverwaltung — aber ein Vielfaches an
  Arbeit, und ohne Bibliotheken von Dritten löst er kein heutiges Problem.
  Später möglich; der Umbenennungs-Weg verbaut ihn nicht.
- **Builtins mit in Namensräume nehmen.** 1401 Namen umzusortieren bricht
  jedes bestehende Programm. Kommt nicht in Frage.
- **Automatisch namensräumen, ohne `AS`.** Bricht Messung (b) — also Code, der
  heute läuft.
