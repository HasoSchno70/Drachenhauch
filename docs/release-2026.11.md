# Drachenhauch 2026.11

*Die Notizen zu dieser Fassung. Kein neuer Befehl, keine neue Syntax — vier
Commits, die alle dieselbe Frage beantworten: **woher weiß ich, dass das
stimmt?***

## Neu in dieser Fassung

2026.10 brachte das `midi`-Modul, und mit ihm einen unangenehmen Satz in den
eigenen Notizen: *„Der Notenfluss von einem echten Keyboard ist ungeprüft."*
An der Entwicklungsmaschine hängt keines, beim Autor auch nicht — der
Empfangspfad wäre also auf Dauer ungeprüft ausgeliefert worden.

**Das war ein Struktur-Problem, kein Hardware-Problem.** Die Entschlüsselung
eingehender Nachrichten — Statusbyte, Kanal, Note, Anschlagstärke, „ist das ein
Note-an?" — hing am Gerätetyp, den man ohne Instrument nicht bauen kann. Sie
ist aber reine Byte-Rechnerei. Jetzt arbeitet sie über rohe Bytes, und **neun
Tests prüfen sie mit erfundenen Nachrichten** — in jedem Bau, auch ohne das
Feature, auch in der CI auf allen drei Systemen. Abgedeckt ist genau das, wo es
schiefgehen kann:

* Note-aus in **beiden** Formen — `0x80`, und die übliche Form *Note an mit
  Anschlagstärke 0*. Wer nur auf `0x80` prüft, bekommt Töne, die nie aufhören.
* Kanäle 1 und 16 (im Protokoll 0 und 15), und dass der Kanal das Statusbyte
  nicht verwäscht.
* Die **leere** Nachricht: ohne Sonderfall wäre `0 & 0x0F + 1` gleich 1
  gewesen und hätte einen Kanal vorgetäuscht, den es gar nicht gibt.
* Eine **zu kurze** Nachricht: fehlt das dritte Byte, ist der Anschlag 0 — also
  Note aus. Lieber ein Ton, der endet, als einer, der hängenbleibt.

**Und dann ging auch der ganze Kreis.** Ein *virtueller* Loopback-Port
(unter Windows z.B. [loopMIDI](https://www.tobias-erichsen.de/software/loopmidi.html),
`winget install TobiasErichsen.loopMIDI`) erscheint dem System als Ein- **und**
Ausgang unter demselben Namen. Damit kann Drachenhauch sich selbst etwas
schicken und es durch Treiber, Bibliothek und Rückruf-Faden zurückbekommen —
**ein Keyboard braucht es dafür nicht.**

Zwei Zusagen, die bisher nur in der Dokumentation standen, sind damit belegt:

1. Ein *Note an* mit Anschlagstärke 0 kommt als **Note aus** an — über einen
   echten Transport, nicht nur in der Byte-Rechnerei.
2. Die Warteschlange deckelt bei **1024** und wirft die **älteste** weg. 1200
   gesendet, keine abgeholt → genau 1024 warten, und die erste überlebende ist
   die 176. gesendete. Bei „jüngste fällt weg" wäre es die erste gewesen.

Erkannt wird der Loopback daran, dass *ein* Name auf beiden Seiten auftaucht —
ein echtes Gerät tut das nie. Ohne so einen Port überspringen sich die beiden
Tests; nachgeprüft, indem loopMIDI beendet wurde.

Was weiterhin offen bleibt und auch so dasteht: dass ein **bestimmtes
Instrument** sich ans Protokoll hält. Das kann kein Loopback zeigen.

## Zwei Beispiele, die aus dem Editor nicht liefen

Gemeldet als *„161_werkzeug.dh funktioniert nicht"*. Auf der Kommandozeile lief
jeder Weg richtig — der Fehler lag im **Editor**: `F5` kann einem Programm
keine Argumente mitgeben, also druckte das Werkzeug nur

```
Verwendung: 161_werkzeug [--nur-zeilen] <datei> [datei ...]
Programm beendet mit Fehler-Code 2.
```

und war fertig. Wer die Datei öffnet und `F5` drückt, sieht zu Recht „geht
nicht" — jedes andere Beispiel im Ordner macht dort etwas Sichtbares.

Beide betroffenen Werkzeuge (`172_filter.dh` hatte dasselbe Problem) führen
sich jetzt ohne Argumente **am eigenen Quelltext** vor: 161 zählt sich selbst,
172 sucht darin nach `STDIN`. Dazu sagen sie, wie ein richtiger Aufruf aussieht
und warum er aus dem Editor nicht geht. Als exportierte `.exe` liegt der
Quelltext nicht daneben — dort bleibt es beim Verwendungshinweis mit
Rückgabewert 2, und damit bleibt auch die Lektion erhalten, um die es dem
Beispiel geht.

Nebenbei ein echter Fehler in 161: eine **unbekannte Option wurde als
Dateiname gelesen**. Ein Tippfehler meldete sich also als
`Nicht gefunden: --nur-zeilne` und versteckte damit, was wirklich los war.

## Der Installer räumt jetzt auf

An einer echten Installation nachgemessen: der Beispielordner beim Nutzer hatte
**drei Dateien mehr** als die Quelle. Zwei davon waren Vorschaubilder aus
früheren Fassungen, die es im Repo längst nicht mehr gibt.

Die Ursache ist dieselbe, die den Juni-Installer 225 verwaiste Dateien
hinterlassen ließ: der Beispielordner ist absichtlich vom Deinstallieren
ausgenommen, damit selbst bearbeitete Beispiele ein Upgrade überleben. Die
Kehrseite — niemand räumt ihn je weg — gilt eben auch für alles, was aus dem
Repo verschwindet. Nur langsamer.

Aufgeräumt wird deshalb **genau der Unterordner mit den Vorschaubildern**: der
ist reine Erzeugung, dort legt niemand etwas Eigenes ab, und er wird direkt
danach wieder gefüllt. Den ganzen Beispielordner zu leeren würde zerstören,
wofür die Ausnahme überhaupt da ist.

Die dritte Zusatzdatei bleibt bewusst liegen: sie war nie im Repo, sondern ist
die Ausgabe eines Beispiels. Was der Nutzer selbst erzeugt hat, löscht kein
Installer.

## Unter der Haube

Die Testsuite ist von 3799 auf **3803** Prüfungen gewachsen, dazu neun neue
Rust-Tests. Beide Werkzeug-Beispiele stehen jetzt im Rauchtest, der
Rückgabewert 0 *und* Ausgabe verlangt — genau der Rückfall, der oben gemeldet
wurde, fällt damit auf.

Befehle und Module sind unverändert (1558 / 47): diese Fassung fügt nichts
hinzu, sie belegt.
