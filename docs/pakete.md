# Pakete: Bibliotheken holen und teilen

Eine Bibliothek ist ein Ordner mit `.dh`-Dateien. `dhrt paket` holt sie in
dein Projekt, merkt sich, welche Fassung es war, und holt auf einem anderen
Rechner genau dieselbe wieder.

```bash
dhrt paket hole github:hans/spielkiste@1.2
```

```basic
IMPORT "spielkiste/vektor.dh"
```

## Woher Pakete kommen

Es gibt keinen eigenen Server. Ein Paket kommt von dort, wo es ohnehin liegt:

| Quelle | Bedeutung |
|---|---|
| `github:nutzer/repo@stand` | das ZIP, das GitHub für einen Tag, Zweig oder Commit ausliefert |
| `https://beispiel.de/kiste.zip` | ein beliebiges ZIP |
| `https://beispiel.de/hilfe.dh` | eine einzelne Datei |
| `../libs/werkzeug` | ein lokaler Ordner (relativ zum Projekt) |
| `../libs/werkzeug.zip` | ein lokales ZIP |

**Der Stand ist Pflicht.** `github:hans/spielkiste` ohne `@…` ist ein
Fehler: „der neueste Stand“ wäre morgen ein anderer. Am besten nimmt man
einen Tag (`@1.2`); ein Zweig (`@main`) geht, wandert aber weiter.

Liegt im ZIP alles in einem einzigen Oberordner (so liefert GitHub,
`spielkiste-1.2/`), fällt dieser Ordner weg.

## Wo sie landen

Im Projekt, in `pakete/<name>/` neben der Datei `paket.json`. Jedes Projekt
hat damit seine eigenen Fassungen. `<name>` ist der letzte Teil der Quelle;
mit `--als` wählt man einen anderen:

```bash
dhrt paket hole ../libs/werkzeug --als wz
```

`IMPORT` sucht nach der Datei neben dem Programm in `pakete/`, von der
importierenden Datei aus nach oben. Die genaue Reihenfolge steht in der
[Sprachreferenz](sprache.md#wo-gesucht-wird); kurz:

1. neben der importierenden Datei (eine eigene Kopie gewinnt immer),
2. `pakete/` von dort aus nach oben,
3. `DH_PATH`, dann die Bibliothek im Benutzerordner.

## Die zwei Dateien

**`paket.json`** sagt, was das Projekt braucht. Man kann sie von Hand
lesen und ändern:

```json
{
  "name": "mein-spiel",
  "pakete": {
    "spielkiste": "github:hans/spielkiste@1.2",
    "wz": "../libs/werkzeug"
  }
}
```

**`paket.lock.json`** hält fest, was tatsächlich geholt wurde: die Quelle,
die Adresse und eine **SHA-256-Prüfsumme über den Inhalt**. Kommt beim
nächsten Holen etwas anderes an (ein Tag wurde verschoben, ein Server
liefert etwas Fremdes, eine lokale Bibliothek wurde geändert), ist das ein
Fehler, und das alte Paket bleibt stehen. Wer den neuen Inhalt will:

```bash
dhrt paket hole --erneuern
```

Die Prüfsumme geht über die Dateien, nicht über die Bytes des ZIPs. Packt
ein Server dasselbe neu, schlägt sie also nicht an.

Beide Dateien gehören ins Repository. `pakete/` kann man mit
hineinnehmen oder weglassen; mit der Sperrdatei holt `dhrt paket hole`
genau dasselbe wieder.

## Befehle

| Befehl | Wirkung |
|---|---|
| `dhrt paket hole` | alles aus `paket.json` holen, gegen die Sperrdatei geprüft |
| `dhrt paket hole --erneuern` | dasselbe, neuen Inhalt annehmen |
| `dhrt paket hole <quelle> [--als name]` | ein Paket dazunehmen |
| `dhrt paket entferne <name>` | aus beiden Dateien und aus `pakete/` nehmen |
| `dhrt paket liste` | was das Projekt hat, mit Prüfsumme |
| `dhrt paket neu [name]` | eine leere `paket.json` anlegen |

Das Projekt ist der nächste Ordner mit `paket.json`, vom Arbeitsordner aus
nach oben gesucht.

## Ein eigenes Paket anbieten

Ein Ordner mit `.dh`-Dateien genügt, am besten ein Repository mit Tags.
Braucht das Paket selbst Pakete, bekommt es eine eigene `paket.json`.
`dhrt paket hole` holt sie in **sein** `pakete/`, und dort findet das Paket
sie zuerst, auch wenn das Projekt eine andere Fassung desselben Pakets hat.

Ein lokaler Pfad in dieser `paket.json` gilt ab dem Ordner, aus dem das
Paket kam. Ein heruntergeladenes Paket kann nicht auf lokale Pfade
verweisen. Ein Kreis (A braucht B, B braucht A) ist ein Fehler.

## Sicherheit

Beim Holen wird **nichts ausgeführt**: kein Installationsskript, kein Bau.
Entpackt wird so, dass kein Eintrag aus seinem Ordner herauskommt (kein
`..`, keine absoluten Pfade).

Ein Paket ist aber fremder Code: Beim Laufen darf es alles, was ein
Drachenhauch-Programm darf, also Dateien schreiben und ins Netz gehen.
Die Prüfsumme schützt davor, dass sich ein Paket zwischen zwei
Holvorgängen ändert, nicht vor einem bösartigen Original. Hol Pakete von
Quellen, denen du traust.

## Export

`dhrt --export` übersetzt das Programm samt aller Importe. Pakete stecken
damit in der fertigen Datei, und auf dem Zielrechner braucht es weder
`pakete/` noch `dhrt paket`.

## Warum es so gebaut ist

Die Entscheidungen (kein Server, Pakete im Projekt, Prüfsumme über den
Inhalt) und was bewusst fehlt, stehen im [Entwurf](entwurf-pakete.md).
