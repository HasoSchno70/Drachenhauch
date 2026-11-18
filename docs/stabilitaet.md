# Stabilität

Drachenhauch erscheint oft in einer neuen Fassung, meist mehrmals im Monat
(`2026.51`, `2026.52` …). Damit ein Programm, das heute läuft, auch mit den
späteren läuft, gelten seit 2026.52 diese Zusagen. Was sich in jeder Fassung
für bestehende Programme geändert hat, steht im
[Änderungsprotokoll](aenderungen.md).

## Was fest ist

| Was | Zusage |
|---|---|
| Sprache | Schlüsselwörter, Typen, Operatoren und ihre Bedeutung bleiben. Ein Programm, das heute übersetzt, übersetzt auch mit der nächsten Fassung. |
| Befehlsnamen | Kein Befehl verschwindet. Bekommt einer nach der [Leitlinie](builtins-namen.md) einen neuen Namen, gilt der alte weiter, ohne Ablaufdatum. |
| Argumente | Was ein Befehl heute nimmt, nimmt er weiter. Neue Parameter kommen hinten dazu und sind optional. |
| Parameternamen | Die Namen für benannte Argumente (`CIRCLE(x: 10, y: 20, r: 5)`) bleiben, wie sie sind. |
| Dateien | Was ein Programm schreibt (Spielstände mit `save`, Formulare `.dhform`, Projekte des Studios), liest auch jede spätere Fassung. |
| Exportierte Programme | Sie tragen ihre Laufzeit in sich und laufen weiter, gleich, was danach erscheint. |

## Was sich ändern darf

| Was | Warum |
|---|---|
| Wortlaut von Meldungen | Meldungen werden genauer. Ein Programm prüft darum nicht auf den genauen Text einer Meldung. |
| Neue Warnungen | `dhrt --check` und der Editor finden mit jeder Fassung mehr. Eine Warnung hält kein Programm an. |
| Aussehen | Themen, Abstände und Farben der gui, der IDE und des Studios. |
| Vorübersetzter Bytecode (`.dhc`) | Er gehört zu seiner Fassung: nach einem Update neu übersetzen. |
| Einstellungen der Werkzeuge | `ide.json`, `studio.json` und ihre Verwandten. |

## Wenn doch etwas bricht

Einen Fehler, der bisher still falsch rechnete, meldet eine neue Fassung --
lieber ein klarer Abbruch als ein falsches Ergebnis. Solche Änderungen nennt
jede Fassung in ihren Notizen ganz oben, unter „Was sich für bestehende
Programme ändert“, gesammelt im [Änderungsprotokoll](aenderungen.md).

## Fassungsnummern

Jahr und laufende Nummer (`2026.52`). Haupt- und Nebenversionen gibt es
nicht: jede Fassung hält die Zusagen oben, also braucht es keine Nummer, die
einen Bruch ankündigt.
