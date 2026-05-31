# Vorwort

## Für wen dieses Buch ist

Du willst Spiele schreiben. Vielleicht hast du schon mal Python angefangen und bist im Game-Loop hängen geblieben. Vielleicht hast du als Kind QBasic-Bücher durchgeblättert und vermisst den Stil: kleine, lesbare Programme, die *sofort etwas tun*. Vielleicht bist du komplett neu im Programmieren und möchtest mit etwas anfangen, wo am Ende ein echtes Spiel steht.

Dieses Buch ist für alle drei.

GameBasic ist ein moderner BASIC-Dialekt, der bewusst klassisch aussieht (`DIM`, `SUB`, `PRINT`) und trotzdem alles mitbringt, was man heute zum Spielebauen erwartet: Klassen mit Vererbung, Pascal-strikte Typen, Module für Sprites, Particles, Pathfinding. Du musst keine fünf Frameworks zusammenbauen — du schreibst eine `.gb`-Datei und drückst Play.

## Was du am Ende kannst

Wir bauen zusammen **Star Pilot**, einen Space-Shooter im Stil von Galaga: dein Schiff fliegt am unteren Rand, Wellen von Aliens kommen in Formation runter, Boss am Ende. Mit Highscore-Tabelle, Pause-Menü, Particles, allem.

Aber das eigentliche Ergebnis bist du selbst:

- Du verstehst, wie ein **Game-Loop** funktioniert — die ewig-wiederholte Schleife, die jedes Spiel antreibt.
- Du kannst **Klassen** so bauen, dass sich der Code in mehreren Dateien organisieren lässt, ohne unübersichtlich zu werden.
- Du erkennst Stellen, wo **Vererbung** wirklich hilft (und wo sie nur Komplexität dazuschüttet).
- Du weißt, wie Spiele **Spielstände speichern**, wie **Animationen** mit Tweening funktionieren, wie **Kollisions-Erkennung** intern aussieht.
- Du hast Routine im Umgang mit dem **Editor und den Built-in-Modulen** — und kannst dein eigenes Spiel anfangen.

## Wie du das Buch liest

**Tipp**: lies nicht nur, **tippe ab**. Das ist altmodisch, aber unschlagbar fürs Erlernen einer Sprache. Jedes Code-Beispiel ist klein genug, dass du es in zwei Minuten abtippen kannst — und dann *läuft etwas*.

**Sequenziell statt selektiv**: jedes Kapitel baut auf dem vorigen auf, wir schreiben fortlaufend an demselben Spiel. Wenn du Kapitel 9 anschaust, ohne 5 bis 8 gelesen zu haben, fehlt dir Kontext.

**Übungen** stehen am Ende jedes Kapitels. Sie sind nicht optional. Wer sie überspringt, hat den Stoff nicht verstanden — egal wie überzeugend er klingt.

**Code-Stände**: zu jedem Kapitel gehört ein Ordner unter [`code/kap-NN/`](code/) mit dem vollständigen, lauffähigen Stand des Spiels. Wenn etwas nicht klappt, vergleiche mit dem Original.

## Was dieses Buch *nicht* ist

- Kein Referenzbuch — die [vollständige Sprachreferenz](../docs/sprache.md) liegt im Haupt-Projekt. Wir picken hier raus, was wir gerade brauchen.
- Kein Dogma. Pythonistas und C++-Veteranen werden Stellen finden, wo wir Dinge anders machen als in ihrer Welt. Das ist meist Absicht (BASIC hat eine eigene Tradition); manchmal ist es einfach so wie es ist.
- Kein Sprint zur Game-Industrie. Star Pilot ist klein genug, dass wir es zu Ende bauen können — kein 3D, keine Netzwerk-Spiele, kein Asset-Pipeline-Tooling. Was du danach damit machst, ist deine Entscheidung.

## Bevor wir loslegen

Stell sicher, dass GameBasic auf deinem Rechner läuft. Im [Haupt-README](../README.md) steht der Schnellstart. Wenn du

```
.venv\Scripts\python.exe gbrun.py
```

eintippst und der Editor aufgeht — dann sind wir startklar.

Los geht's.
