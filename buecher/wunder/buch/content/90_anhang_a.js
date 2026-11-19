const { neu } = require("../code_lesen");

// Anhang A: die Loesungen. Der Code kommt aus code/loesungen/ -- nur die
// Stellen zwischen ' >>> neu und ' <<< neu, der Rest ist das Programm des
// Kapitels. Die Dateien selbst laufen in der Pruefsammlung.
module.exports = (H) => [
  H.chapter("A · Lösungen der Übungen"),

  H.p("Zu den Drehen-Übungen steht hier die Antwort mit ihrer Begründung. Zu Erweitern und Erfinden steht eine Lösung als Datei in code/loesungen; abgedruckt sind nur die Stellen, die gegenüber dem Programm des Kapitels neu sind. In den Dateien sind sie mit >>> und <<< markiert. Eine andere Lösung ist nicht falsch, nur weil sie anders aussieht — wenn sie tut, was die Aufgabe verlangt, ist sie richtig."),

  H.h2("Kapitel 5 · Feuerwerk"),

  H.h3("Drehen 1"),

  H.p("Die Steighöhe ist v² / (2 · SCHWERE). Mit SCHWERE = 30 steigen die Raketen 230² / 60 ≈ 880 bis 300² / 60 = 1500 Pixel — weit über den oberen Rand eines 500 hohen Fensters. Man sieht Raketen aufsteigen, aber keine Explosion, nur sehr viel später ein paar Funken, die langsam von oben hereinsinken (die Schwere ist ja auch für sie schwach)."),

  H.p("Mit SCHWERE = 400 sind es 230² / 800 ≈ 66 bis 300² / 800 ≈ 112 Pixel. Die Raketen zerplatzen nach gut einer halben Sekunde knapp über dem Boden, und die Funken fallen schwer und schnell nach unten. Das sieht weniger nach Feuerwerk aus als nach Funkenflug aus einem Schleifstein."),

  H.h3("Drehen 2"),

  H.p("Die Explosionen werden dichter und heller, weil sich mehr durchsichtige Scheine überlagern; die Mitte einer frischen Explosion wird fast weiß. Bei fünf Raketen kurz hintereinander leben etwa fünftausend Funken gleichzeitig, und je nach Rechner beginnt das Bild dann zu ruckeln. Hier zeigt sich, warum Zeit in Sekunden gerechnet wird: Auch wenn weniger Bilder pro Sekunde kommen, fliegen die Funken gleich schnell, nur in größeren Sprüngen."),

  H.h3("Drehen 3"),

  H.p("Jeder Funke bekommt eine völlig zufällige Farbe, jede Explosion wird bunt gesprenkelt. Verloren geht, dass man die Raketen auseinanderhalten kann: Zwei Explosionen, die sich überlagern, verschwimmen zu einer. Die Farbe einer Rakete ist Information, nicht nur Schmuck."),

  H.h3("Erweitern 1: zünden mit der Maus"),

  H.pmix(["Aus h = v² / (2 · SCHWERE) wird v = √(2 · SCHWERE · h). Die Höhe h ist der Abstand vom unteren Rand bis zum Klick, also ", ["HOEHE - y", true], ". Das Tempo ist negativ, weil die Rakete nach oben fliegt. Neu ist eine SUB neben ", ["raketeStarten", true], " (Datei code/loesungen/kap05_klick.dh):"]),

  ...neu("loesungen/kap05_klick.dh").map((t) => H.code(t)),

  H.pmix(["Die zweite Stelle steht in der Hauptschleife, direkt unter der Abfrage der Leertaste. Die Rakete zerplatzt nicht auf das Pixel genau an der Klickstelle — sie fliegt in Schritten von einem Bild, und der letzte Schritt kann ein wenig über den höchsten Punkt hinausgehen. Bei 60 Bildern pro Sekunde sind das höchstens ein, zwei Pixel."]),

  H.h3("Erweitern 2: die Spur"),

  H.pmix(["Ein neuer STRUCT ist nicht nötig: Ein Spurfunke ist ein Funke wie jeder andere, nur langsamer, gelblich und kurzlebig. Darum genügt es, in ", ["raketenBewegen", true], " in jedem Bild einen Funken unter die Rakete zu setzen (code/loesungen/kap05_spur.dh):"]),

  ...neu("loesungen/kap05_spur.dh").map((t) => H.code(t)),

  H.p("Das kleine Tempo nach unten lässt die Spur ein Stück zurückfallen, so wirkt sie wie ein Ausstoß. Bei 60 Bildern pro Sekunde und drei Raketen in der Luft sind das 180 Funken pro Sekunde zusätzlich — für das Feld kein Problem, es wächst und schrumpft einfach mit."),

  H.h3("Erfinden: das Feuerwerk in zwei Stufen"),

  H.pmix(["Eine Möglichkeit (code/loesungen/kap05_kaskade.dh): Der Funke bekommt ein Feld ", ["knaller", true], ", jeder zehnte ist einer. Wenn ein Knaller verlischt, ruft das Aufräumen ", ["nachknallen", true], " auf, bevor es ihn aus dem Feld nimmt:"]),

  ...neu("loesungen/kap05_kaskade.dh").map((t) => H.code(t)),

  H.pmix(["Zwei Dinge sind daran lehrreich. Erstens nehmen die neuen Funken das Tempo des Knallers mit (", ["k.vx + …", true], "), so fliegt die kleine Explosion in seine Richtung weiter, wie bei einer echten. Zweitens hängt ", ["nachknallen", true], " neue Funken an das Feld an, WÄHREND die Schleife es rückwärts durchläuft. Das ist sicher: Die neuen landen hinten, an Stellen, die die Schleife schon hinter sich hat — sie werden erst im nächsten Bild bewegt."]),
];
