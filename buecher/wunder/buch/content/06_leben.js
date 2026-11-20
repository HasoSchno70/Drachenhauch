const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Das Spiel des Lebens", 6),

  H.h2("Was wir bauen"),

  H.figure("kap06_leben.png", "Das Spiel des Lebens nach einigen Dutzend Generationen: Aus Zufall sind ruhende Blöcke, pulsierende Blinker und wandernde Gleiter geworden.", 470, 295),

  H.p("Ein Feld aus Zellen, jede lebendig oder tot. In jeder Generation zählt jede Zelle ihre acht Nachbarn, und drei Regeln entscheiden, ob sie lebt, stirbt oder geboren wird. Mehr gibt es nicht — keine Spieler, kein Ziel. Trotzdem entstehen aus einem Feld voller Zufall Formen, die stillstehen, pulsieren oder über das Feld wandern. Der Mathematiker John Conway hat es 1970 erfunden, und seitdem schauen ihm Menschen zu."),

  H.h2("Der Plan"),

  H.p("Die drei Regeln, für jede Zelle und ihre acht Nachbarn:"),

  H.bulletRich("Überleben: ", "Eine lebende Zelle mit zwei oder drei lebenden Nachbarn lebt weiter."),
  H.bulletRich("Sterben: ", "Eine lebende Zelle mit weniger als zwei Nachbarn stirbt an Einsamkeit, mit mehr als drei an Überfüllung."),
  H.bulletRich("Geburt: ", "Eine tote Zelle mit genau drei lebenden Nachbarn wird lebendig."),

  H.p("Wichtig ist das Wörtchen „gleichzeitig“: Alle Zellen werden nach dem Stand der vorigen Generation beurteilt, keine sieht schon das Ergebnis einer anderen. Wie man das im Programm hinbekommt, ist der Kern dieses Kapitels."),

  H.h2("Schritt 1: ein Gleiter"),

  H.p("Das Feld ist ein Raster aus 80 mal 50 Zellen. In Drachenhauch ist das ein Feld mit zwei Nummern, und weil man Zellen ab 1 zählt, bekommt es eigene Untergrenzen:"),

  H.code(stueck("kap06/1_gleiter.dh", "DIM jetzt[1 TO B", "DIM danach[1 TO B")),

  H.pmix([["jetzt[1, 1]", true], " ist die Zelle oben links, ", ["jetzt[80, 50]", true], " die unten rechts. Eine lebende Zelle hat den Wert 1, eine tote 0. Das zweite Feld, ", ["danach", true], ", nimmt die nächste Generation auf — gleich mehr dazu."]),

  H.p("Für die Nachbarn braucht es eine Antwort auf eine kleine Frage: Was ist mit Zellen am Rand, die links oder oben gar keinen Nachbarn haben? Eine eigene Funktion beantwortet sie für das ganze Programm — außerhalb des Felds lebt nichts:"),

  H.code(stueck("kap06/1_gleiter.dh", "' Lebt die Zelle", "END FUNCTION")),
  H.code(stueck("kap06/1_gleiter.dh", "' Wie viele der acht", "END FUNCTION")),

  H.p("Die zwei Schleifen gehen das Quadrat von drei mal drei Zellen um (x, y) durch. Die Zelle selbst liegt in der Mitte dieses Quadrats und wird mitgezählt; darum zieht die Funktion sie am Ende wieder ab. Das ist einfacher, als in der Schleife zu prüfen, ob man gerade in der Mitte ist."),

  H.p("Dann eine Generation. Die Regeln stehen fast wörtlich da:"),

  H.code(stueck("kap06/1_gleiter.dh", "SUB schritt", "END SUB")),

  H.pmix(["Gelesen wird aus ", ["jetzt", true], ", geschrieben in ", ["danach", true], ". Am Ende tauschen die beiden die Rollen: Das neue Feld wird das aktuelle, und das alte steht für die übernächste Generation bereit. Weil ein Feld in Drachenhauch ein Verweis ist, tauscht man dabei keine 4000 Zahlen, sondern drei Namen — es kostet nichts."]),

  H.p("Zum Ausprobieren setzt das Programm ein einziges Muster aus fünf Zellen, den Gleiter, und rechnet alle 0,15 Sekunden eine Generation:"),

  H.code(stueck("kap06/1_gleiter.dh", "' Der Gleiter oben links", "jetzt[4, 4] = 1")),

  H.figure("kap06_1_gleiter.png", "Der Gleiter: Nach vier Generationen hat er wieder dieselbe Form, eine Zelle weiter rechts und eine weiter unten. So wandert er schräg über das Feld.", 420, 262),

  H.h2("Schritt 2: Zufall und Bedienung"),

  H.p("Das fertige Programm füllt das Feld mit Zufall — jede vierte Zelle lebt — und lässt es laufen. R würfelt neu, die Leertaste hält an, oben links steht die Generation:"),

  H.code(stueck("kap06/2_leben.dh", "' Etwa jede vierte", "END SUB")),

  H.p("Aus dem Rauschen wird in wenigen Dutzend Generationen etwas mit Gestalt. Die meisten Zellen sterben; was übrig bleibt, sind Muster, die sich halten: Blöcke aus vier Zellen, Bienenstöcke aus sechs, Blinker aus dreien, die zwischen waagrecht und senkrecht hin- und herwechseln, und hier und da ein Gleiter, der davonzieht, bis er auf etwas trifft. Das ganze Programm:"),

  H.code(zeilen("kap06/2_leben.dh")),

  H.h2("Warum so?"),

  H.h3("Zwei Felder"),

  H.p("Der wichtigste Satz des Plans war „gleichzeitig“. Ein Programm geht die Zellen aber nacheinander durch, eine nach der anderen. Schriebe es das Ergebnis gleich in dasselbe Feld, sähe die nächste Zelle schon den neuen Stand ihres linken Nachbarn und den alten ihres rechten — eine Mischung aus zwei Generationen. Mit zwei Feldern sieht jede Zelle nur die vorige Generation, egal in welcher Reihenfolge das Programm rechnet. Was das ausmacht, zeigt der Abschnitt „Was schiefgehen kann“."),

  H.h3("Eigene Untergrenzen"),

  H.pmix(["Mit ", ["DIM jetzt[1 TO B, 1 TO H]", true], " beginnt das Feld bei 1, wie man Zellen zählt: die erste Spalte, die achtzigste Spalte. Nur beim Zeichnen braucht man die Stelle auf dem Bildschirm, und die beginnt bei 0 — darum ", ["(x - 1) * ZELLE", true], ". Ein Feld ab 0 ginge genauso; man rechnet dann an anderer Stelle mit 1. Wichtig ist nur, dass das Programm die Grenzen an einer Stelle hat: Die Funktion lebt prüft gegen 1 und B, und sonst kein Teil des Programms."]),

  H.h3("Eine Funktion für den Rand"),

  H.p("Der Rand ist eine Sonderregel, und Sonderregeln gehören an eine Stelle. Weil nachbarn nie selbst auf das Feld greift, sondern immer lebt fragt, kann nachbarn nicht über den Rand hinauslesen, und lebt kann die Regel für den Rand ändern, ohne dass sonst etwas anders wird. Genau das tut die Übung mit den herumlaufenden Rändern: Sie ändert nur lebt, und das Feld hat keinen Rand mehr — wie die Oberfläche eines Rings."),

  H.h2("Was schiefgehen kann"),

  H.h3("Ein Feld statt zwei"),

  H.p("Schreibt schritt direkt in jetzt, sieht jede Zelle die schon geänderten Zellen links und oben von sich:"),

  H.code(stueck("kap06/fehler_ein_feld.dh", "' Eine Generation -- FALSCH", "END SUB")),

  H.figure("kap06_fehler_ein_feld.png", "Derselbe Gleiter mit nur einem Feld: Nach wenigen Generationen ist er ein starrer Block aus vier Zellen, und dabei bleibt es.", 420, 262),

  H.p("Eine Meldung gibt es nicht. Das Programm rechnet tadellos — nur nicht das Spiel des Lebens. Der Gleiter wird zum Block, weil die oberen Zellen schon tot oder lebendig sind, wenn die unteren ihre Nachbarn zählen. Solche Fehler findet man nur, wenn man weiß, was herauskommen müsste. Darum zeigt Schritt 1 einen Gleiter: Von dem weiß man, wie er sich bewegen muss."),

  H.h3("Zwei Felder, die eines sind"),

  H.pmix(["Eine zweite, stillere Falle: ", ["danach = jetzt", true], " am Anfang von schritt, um „mit einer Kopie“ anzufangen. Weil ein Feld ein Verweis ist, sind danach und jetzt dann zwei Namen für dasselbe Feld, und das Programm schreibt wieder in das, was es liest — mit demselben Ergebnis wie oben. Wer wirklich eine Kopie braucht, schreibt ", ["danach = ARRAY_COPY(jetzt)", true], "."]),

  H.h3("Ein Index außerhalb"),

  H.pmix(["Fragt nachbarn direkt ", ["jetzt[x + dx, y + dy]", true], " statt lebt, geht alles gut, bis die erste Zelle am Rand drankommt: ", ["jetzt[0, 1]", true], " gibt es nicht, und das Programm bricht mit „Index 0 ausserhalb [1..80] in Dimension 1“ ab. Die Meldung nennt die Grenzen, die das Feld wirklich hat — hier 1 bis 80, nicht 0 bis 79."]),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Ändere die Wahrscheinlichkeit in wuerfeln von 0.25 auf 0.05 und auf 0.7. Was bleibt jeweils nach hundert Generationen übrig?"),
  H.bulletRich("Drehen 2. ", "Ändere die Regel für die Geburt von n = 3 auf n = 3 OR n = 6. Wie verändert sich das Spiel?"),
  H.bulletRich("Drehen 3. ", "Was macht ein Blinker — drei Zellen nebeneinander —, wenn man ihn statt des Gleiters setzt? Sage es vorher, mit Papier und den drei Regeln."),
  H.bulletRich("Erweitern 1. ", "Zellen mit der Maus setzen (linke Taste) und löschen (rechte), auch während das Spiel läuft."),
  H.bulletRich("Erweitern 2. ", "Ränder, die herumlaufen: Was rechts hinausgeht, kommt links wieder herein, oben und unten ebenso."),
  H.bulletRich("Erfinden. ", "Gib den Zellen ein Gedächtnis: Farbe nach Alter, eine Spur, wo eben noch Leben war, oder andere Regeln (es gibt Tausende benannte Varianten). Anhang A zeigt die Farbe nach Alter."),

  H.figure("kap06_alter.png", "Eine Möglichkeit für die Erfinden-Übung: Junge Zellen leuchten gelb, alte werden über Grün und Türkis zu Blau. Die stabilen Blöcke verraten sich sofort.", 420, 262),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Sprache → Arrays: ", "mehrdimensionale Felder, eigene Untergrenzen (LBOUND, UBOUND), ARRAY_COPY und warum ein Feld ein Verweis ist."),
];
