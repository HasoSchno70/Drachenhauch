# Entwurf: `TASK_START` — GB-Code im Hintergrund (Stufe C)

**Stand 2026-08-19.** Der letzte offene Punkt aus WP H. Er steht seit dem
17.08. ausdrücklich als *nicht umgesetzt, und zwar mit Grund* in der Roadmap;
dieser Entwurf trägt die Begründung zusammen und stellt ihr einen dritten Weg
gegenüber, den sie nicht kennt.

> Codeblöcke sind als `text` ausgezeichnet: sie zeigen Befehle, die es noch
> nicht gibt.

## 1. Was schon geht — und was fehlt

Aus WP H stammt eine ganze Familie von Hintergrund-Befehlen, und sie decken
den Alltag ab:

| vorhanden | tut |
|---|---|
| `HTTP_GET_START` … | Netzabruf, während die Schleife läuft |
| `DB_QUERY_START` … | Datenbankabfrage nebenher (eigene Verbindung) |
| `SHELL_START` … | fremdes Programm starten und einsammeln |

Alle drei nach demselben Muster: `_START` gibt eine Auftragsnummer, `_READY`
fragt nach, `_RESULT`/`_RESULT$` holt ab, `_CANCEL` bricht ab, `_PENDING`
zählt. Die Verwaltung liegt in `hintergrund.rs` und ist **generisch**
(`Auftraege<T>`), also für einen weiteren Auftragstyp schon vorbereitet.

Was fehlt, ist der Fall *„rechne **meine eigene** Funktion nebenher"*:

```text
DIM auftrag AS INTEGER
auftrag = TASK_START(BerechneWeltkarte, 4242)
...
IF TASK_READY(auftrag) THEN PRINT TASK_RESULT$(auftrag)
```

## 2. Warum es bisher nicht geht

`Value` hält Zeichenketten, Arrays, Maps und Objekte durchgehend in `Rc`
(28 Stellen in `value.rs`), und `Func` hält `Vec<Value>` als
Parameter-Vorgaben. `Program` ist damit weder `Send` noch `Sync` — es lässt
sich nicht über eine Thread-Grenze reichen.

## 3. Drei Wege

### A. `Value` auf `Arc` umstellen

Der direkte Weg: `Rc` → `Arc` an allen 28 Stellen, dann ist alles `Send`.

**Kosten:** `Arc` zählt atomar. Das verteuert **jede** Zeichenketten- und
Array-Operation in **jedem** einthreadigen Programm — also in praktisch allen
— um einem seltenen Fall zu helfen. Das ist der falsche Handel.

### B. Frische VM im Arbeitsthread

Dem Thread das Programm mitgeben, damit er sich seine eigene VM baut. Kein
gemeinsamer Speicher, also kein `Send`-Problem.

**Kante:** eine frische VM hat **keine initialisierten Globals**, und eine
`CONST` auf oberster Ebene *ist* ein Global. Eine „reine" Funktion, die eine
Konstante benutzt, sähe dort einen Vorgabewert — still und falsch.

### C. Eigener `dhrt`-Prozess *(neu in diesem Entwurf)*

Statt eines Threads ein Kindprozess: Quelldatei, Funktionsname und Argument
hinein, Ergebnis heraus. Ein Prozess teilt keinen Speicher, also verschwindet
das `Send`/`Sync`-Problem vollständig — ohne eine Zeile an `Value`.

Die Maschinerie steht bereits: `SHELL_START` macht genau das für fremde
Programme, und `Auftraege<T>` in `hintergrund.rs` verwaltet die Nummern.

**Und die Kante aus B wird hier zur Zusage.** *Ein Auftrag sieht keine
Globals — gib ihm mit, was er braucht.* Diese Regel hast du bei WP I.1 schon
einmal getroffen: ein mit `AS` importiertes Modul sieht die Globals des
Hauptprogramms nicht, und genau das war dort der Nutzen. Dieselbe Grenze,
derselbe Gewinn: eine Funktion, die im Hintergrund läuft, hängt nicht mehr
davon ab, was das Hauptprogramm zufällig oben stehen hat.

**Kosten, gemessen:** ein `dhrt`-Start kostet **12,3 ms** im Median (11,7 bis
14,8 ms über zwölf Läufe, leeres Programm). Für *„rechne im Hintergrund,
während die Schleife sich dreht"* ist das nicht spürbar — die Schleife dreht
sich in dieser Zeit rund ein Bild weiter. Für tausend kleine Aufträge pro
Sekunde ist es untauglich.

**Zweite Einschränkung:** Argument und Ergebnis müssen übertragbar sein, also
Zahl oder Zeichenkette. Ein Objekt oder ein Array-Handle geht nicht über eine
Prozessgrenze. Wer mehr braucht, reicht JSON durch — die Befehle dafür gibt es.

## 4. Vorschlag

**Weg C**, mit der Prozessgrenze als ausdrücklicher Zusage statt als
Entschuldigung.

| Builtin | Zweck |
|---|---|
| `TASK_START` | fnref, arg → Auftragsnummer |
| `TASK_READY` | nr → BOOLEAN |
| `TASK_RESULT$` | nr → STRING (Ergebnis, einmal abholbar) |
| `TASK_CANCEL` | nr → bricht ab |
| `TASK_PENDING` | → Anzahl laufender Aufträge |

Gleiche Form wie `HTTP_*`, `DB_*` und `SHELL_*`. Wer eines davon kennt, kennt
alle.

**Aufwand:** die Auftragsverwaltung ist generisch und da; zu bauen sind das
Starten des Kindprozesses mit Funktionsname und Argument, ein Einstiegspunkt
in `dhrt` (`dhrt call <datei> <funktion> <arg>`), Index-Einträge, Tests, Doku.
Grob ein halber Tag — deutlich weniger als A oder B, weil nichts an `Value`
oder an der VM angefasst wird.

## 5. Stand: Schritt 1 gebaut (2026-08-19)

`dhrt call <datei> <funktion> [arg]` steht. Er führt **eine** Funktion aus und
lässt das Hauptprogramm stehen; die Antwort ist eine JSON-Zeile, wie bei
`--check`, `profile` und `debug` — der Aufrufer ist eine Maschine.

```text
> dhrt call c.dh Doppelt 21
{"ok":true,"ergebnis":42,"ausgabe":""}
```

`ausgabe` trägt, was die Funktion selbst gedruckt hat — getrennt vom Ergebnis,
damit der Aufrufer nichts auseinanderfieseln muss. Ein Argument, das wie eine
Zahl aussieht, wird eine.

**Ein Befund, der das Risiko entschärft.** Weg B stand unter dem Vorbehalt,
eine frische VM liefere für ein Global still einen *Vorgabewert* — eine
Funktion mit `CONST` hätte dann heimlich falsch gerechnet. Nachgemessen ist es
besser: der Zugriff **meldet sich**.

```text
> dhrt call c.dh SiehtGlobal
{"ok":false,"fehler":"Global-Slot leer"}
```

Damit ist der Fall laut statt leise, und das war das eigentliche Risiko an der
Prozess- wie an der Thread-Variante. *(Die Meldung selbst ist noch
Maschinensprache — „Global-Slot leer" sagt einem GB-Autor nichts. Sie gehört
verbessert, bevor `TASK_START` obendrauf kommt.)*

Neun Golden-Tests in `tests/test_dhrt_call.py`.

**Als Nächstes:** die fünf `TASK_*`-Builtins an `Auftraege<T>` — Kindprozess
starten, Nummer vergeben, Ergebnis einsammeln.

## 6. Zu entscheiden

1. **Ist die Prozessgrenze als Zusage recht?** *„Ein Auftrag sieht keine
   Globals"* ist eine echte Einschränkung. Sie macht Aufträge aber auch
   berechenbar — und sie ist dieselbe Entscheidung wie bei WP I.1.
2. **Genügen Zahl und Zeichenkette** als Argument und Ergebnis, oder soll
   `TASK_START` von vornherein JSON durchreichen?
3. **Braucht es einen neuen `dhrt`-Einstiegspunkt** (`dhrt call …`) oder soll
   das Kind schlicht die Datei laufen lassen, mit dem Funktionsaufruf als
   angehängter Zeile? Letzteres spart einen Unterbefehl, führt aber das
   Hauptprogramm mit aus — und das ist fast nie gewollt.
   *Neigung: eigener Einstiegspunkt.*
