# Kapitel 1 — Setup und das erste Programm

Bevor wir Spiele bauen können, brauchen wir das Werkzeug. Dieses Kapitel ist kurz — wir installieren nichts Großartiges, schreiben das berühmt-berüchtigte „Hallo Welt"-Programm (auf unsere Weise) und stellen sicher, dass deine Maschine bereit ist.

Wenn du diese Seite zu Ende gelesen hast, läuft auf deinem Rechner ein Programm, das *du* geschrieben hast. Klein, aber deins.

## Lernziele

Nach diesem Kapitel:

- weißt du, wie du den GameBasic-Editor startest
- hast du dein erstes Programm geschrieben und ausgeführt
- kennst du den Unterschied zwischen einer `.gb`-Datei und dem laufenden Programm
- kannst du Kommentare im Code setzen
- weißt du, wie ein Programm aus der Kommandozeile gestartet wird (für später)

## Was ist GameBasic?

GameBasic ist eine Programmiersprache, gebaut für Spiele. Sie sieht aus wie das BASIC der 80er und 90er — `PRINT`, `DIM`, `SUB`, `END IF` — fühlt sich aber an wie eine moderne Sprache mit Klassen, strikten Typen und einer ordentlichen IDE.

Die Idee: du sollst dich auf das Spiel konzentrieren können, nicht auf die Sprache. Drei Dinge, die GameBasic anders macht als viele moderne Sprachen:

1. **Lesbar**, sogar für Anfänger. `IF score > 100 THEN PRINT "Stark!" END IF` — keine Klammern-Pyramiden, keine Semikolons.
2. **Streng**, was Typen angeht. Eine `INTEGER`-Variable kann keinen Text aufnehmen. Das spart Bugs, bevor sie entstehen.
3. **Spielfertig**. Sprites, Particles, Sounds, Pathfinding, Save-Files — Module dafür liegen schon dabei. Du importierst sie und nutzt sie.

> **Warum BASIC und nicht Python/C#/Lua?** Weil BASIC didaktisch unschlagbar ist: kurze, lineare Programme die sofort etwas tun. Kein `def main(): ... if __name__ == "__main__":`-Drumherum, kein Build-System für ein Mini-Spiel. Du tippst, du drückst Play, es läuft. Das ist die Kraft, die BASIC in den 80ern groß gemacht hat — und die wir hier wiederbeleben.

## Die Werkzeuge

Du brauchst zwei Dinge: einen **Editor**, in dem du Code schreibst, und den **Runner**, der dein Programm ausführt. Beide sind im GameBasic-Projekt schon dabei.

### Der Editor

GameBasic hat einen eingebauten Editor — eine kleine IDE mit Syntax-Highlighting, Auto-Vervollständigung, einer Konsole für die Ausgabe und einem Run-Button. Genau das was du brauchst.

Du startest ihn so:

```
.venv\Scripts\python.exe gbrun.py
```

(Auf macOS/Linux: `.venv/bin/python gbrun.py`. Im Rest des Buchs zeige ich nur die Windows-Variante; die Anpassung ist immer offensichtlich.)

Wenn du den Befehl eintippst und das Editor-Fenster aufgeht — herzlichen Glückwunsch, GameBasic ist installiert. Das ist die einzige Setup-Hürde, ab hier wird's einfacher.

> **Wenn der Editor nicht aufgeht:** schau in den [Anhang A — Troubleshooting](anhang-a-troubleshooting.md). Die häufigste Ursache ist eine fehlende Python-Bibliothek (`customtkinter`). Die Lösung dauert eine Minute.

### Der Runner

`gbrun.py` ist beides: Editor (ohne Argument) und Runner (mit `.gb`-Datei als Argument). Sobald du eine fertige Datei hast, kannst du sie auch ohne Editor laufen lassen:

```
.venv\Scripts\python.exe gbrun.py mein_programm.gb
```

Im Editor erspart dir der **Run-Button** (oder F5) den Tippaufwand.

## Schritt 1: Das erste Programm

Im Editor: **Neue Datei** (oder Strg+N). Ein leeres Fenster begrüßt dich. Tippe:

```basic
PRINT "Hallo, Pilot!"
```

Das ist das ganze Programm. Eine Zeile. Speichere es (Strg+S) als `01_hello.gb` — gerne im Buch-Code-Ordner unter `code/kap-01/`, oder irgendwohin wo du's wiederfindest.

Drück jetzt **Run** (oder F5). Unten in der Konsole erscheint:

```
Hallo, Pilot!
```

Das war's. Du hast gerade dein erstes GameBasic-Programm geschrieben und ausgeführt.

> **Was passiert hier eigentlich?** `PRINT` ist eine eingebaute **Anweisung**. Du gibst ihr einen Text in Anführungszeichen, sie schreibt den in die Konsole. Anführungszeichen sind wichtig — sie sagen GameBasic „das hier ist Text, kein Befehl". Schreib mal zum Test `PRINT Hallo, Pilot!` ohne die Anführungszeichen. Was passiert? (Antwort: ein Fehler, weil GameBasic dann denkt, `Hallo` und `Pilot` seien Variablen, die noch nicht existieren.)

## Schritt 2: Mehrere Zeilen, Kommentare

Ein einziger `PRINT` ist überschaubar. Lass uns zwei, drei mehr machen — und gleich lernen, wie man Kommentare schreibt.

```basic
' Mein erstes GameBasic-Programm.
' Gibt drei Zeilen Begruessung an die Konsole aus.

PRINT "Hallo, Pilot!"
PRINT "Bereit zum Start?"
PRINT "Druecke Run und es geht los."
```

Speichere das als `02_kommentare.gb`, drück Run. Ausgabe:

```
Hallo, Pilot!
Bereit zum Start?
Druecke Run und es geht los.
```

Drei Beobachtungen:

1. **Reihenfolge zählt.** Der erste `PRINT` kommt zuerst, dann der zweite, dann der dritte. Programme arbeiten Zeile für Zeile von oben nach unten ab.
2. **Leerzeilen sind erlaubt.** Die Leerzeile zwischen den Kommentaren und den `PRINT`s ist nur fürs Auge da — GameBasic ignoriert sie.
3. **Kommentare beginnen mit `'`.** Alles ab dem Hochkomma bis zum Zeilenende wird komplett ignoriert. Das ist Platz für Notizen an dich selbst (oder andere) — was tut dieses Programm? Warum ist diese Zeile so seltsam? Wozu der magische Wert `42`?

Kommentare sind kein Luxus. Du wirst Code, den du vor sechs Monaten geschrieben hast, ohne Kommentare nicht mehr verstehen — Profi-Tipp aus erster Hand.

> **Alternative Schreibweise**: GameBasic akzeptiert auch `REM` als Kommentar-Markierer (uralt-BASIC-Tradition). `REM Mein Kommentar` ist äquivalent zu `' Mein Kommentar`. In diesem Buch nutzen wir nur `'` — kürzer, weniger Tipparbeit.

## Schritt 3: Aus der Kommandozeile starten

Im Editor zu sein ist bequem, aber manchmal — vor allem später bei größeren Programmen — willst du das Programm direkt aus dem Terminal starten. Der Befehl ist einfach:

```
.venv\Scripts\python.exe gbrun.py 02_kommentare.gb
```

Das `gbrun.py` öffnet die Datei, führt sie aus und spuckt das Ergebnis ins Terminal:

```
Hallo, Pilot!
Bereit zum Start?
Druecke Run und es geht los.
```

Warum ist das nützlich? Wenn du in Kap 13 dein Spiel mit echten Sprites und Particles gefüllt hast, willst du es vielleicht jemandem schicken. Der jemand kann es dann ohne Editor starten — er braucht nur `gbrun.py` und deine `.gb`-Datei.

## Was eine `.gb`-Datei eigentlich ist

Wenn du deine gespeicherte `01_hello.gb` mit einem normalen Texteditor (Notepad, VS Code, …) öffnest, siehst du genau das was du im GameBasic-Editor getippt hast:

```
PRINT "Hallo, Pilot!"
```

Mehr ist da nicht. Eine `.gb`-Datei ist eine schlichte Textdatei mit einer bestimmten Endung. Der Computer macht aus dieser Textdatei erst ein laufendes Programm, wenn `gbrun.py` ihn dazu auffordert. **Quellcode** ist Text; **das laufende Programm** ist das, was passiert, wenn dieser Text ausgeführt wird.

Diese Trennung ist wichtig: deine Datei lebt auf der Festplatte und ändert sich nicht, bis du sie editierst. Das laufende Programm gibt es nur kurz, während es läuft. Wenn dein Spiel abstürzt — kein Drama, du hast die Datei noch.

## Übungen

**1. Begrüße dich selbst.** Ändere `01_hello.gb` so, dass es dich mit deinem Namen begrüßt. Speichere es als `01_hello.gb` — Run.

**2. Fünf Zeilen.** Erweitere `02_kommentare.gb` auf insgesamt fünf `PRINT`-Zeilen. Was ergibt eine sinnvolle kleine Geschichte? Drei Zeilen Erzählung, eine Pointe, ein Schlusswort?

**3. Kaputtes Programm.** Lass die schließenden Anführungszeichen bei einer `PRINT`-Zeile weg: `PRINT "Hallo, Pilot!`. Drück Run. Lies die Fehlermeldung. Was sagt sie dir? Ab welcher Zeile findet GameBasic das Programm verwirrend?

**4. Stretch.** Schreibe ein Programm, das ein kleines ASCII-Schiff aus mehreren `PRINT`-Zeilen baut. Zum Beispiel:

```
   /\
  /  \
 |    |
  \__/
```

Achtung: jeder Backslash `\` muss korrekt erscheinen — probier aus, wie du den im String unterbringst. (Tipp: GameBasic schluckt Backslashes nicht, du kannst sie direkt schreiben.)

## Zusammenfassung

Du hast in einem Kapitel:

- den Editor gestartet,
- dein erstes Programm getippt, gespeichert, ausgeführt,
- gelernt, dass Programme von oben nach unten abgearbeitet werden,
- Kommentare mit `'` gesetzt,
- gesehen, wie Programme aus der Kommandozeile starten.

Das war Tooling. Im **nächsten Kapitel** geht's um Inhalte: wir lernen Variablen kennen, mit denen sich GameBasic dann nicht mehr nur Texte merken kann, sondern auch Zahlen, Wahrheitswerte und Spielzustände — die Bausteine jedes Spiels.

## Code-Stand am Ende des Kapitels

- [`code/kap-01/01_hello.gb`](code/kap-01/01_hello.gb) — der einzeilige Klassiker
- [`code/kap-01/02_kommentare.gb`](code/kap-01/02_kommentare.gb) — mehrzeilig, mit Header-Kommentar
