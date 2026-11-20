const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Ein Garten wächst", 12),

  H.h2("Was wir bauen"),

  H.figure("kap12_garten.png", "Der Garten nach sechs Sekunden: ein Farn, ein Busch und ein Bäumchen, jedes aus einem Wort von Zeichen gezeichnet, das zwei oder drei Regeln Generation für Generation umgeschrieben haben.", 470, 294),

  H.p("Pflanzen sehen kompliziert aus, aber sie wachsen nach einfachen Regeln: Ein Zweig treibt Seitenzweige, die wieder Seitenzweige treiben. Dieses Kapitel baut zuerst einen Baum aus einer einzigen solchen Regel, dann einen Garten, in dem Farn, Busch und Bäumchen jede Sekunde ein Stück wachsen. Hinter jeder Pflanze steht ein Bauplan aus Buchstaben, so kurz, dass er in eine Zeile passt."),

  H.h2("Der Plan"),

  H.bulletRich("Ein Ast ", "trägt an seiner Spitze zwei kürzere Äste. Die tragen wieder zwei, und so weiter, bis die Äste klein genug sind."),
  H.bulletRich("Ein Bauplan ", "ist ein Wort wie „F+[[X]-X]-F“. Regeln ersetzen in jeder Generation bestimmte Zeichen durch längere Stücke — das Wort wächst."),
  H.bulletRich("Eine Schildkröte ", "liest das fertige Wort Zeichen für Zeichen: Bei F geht sie einen Schritt und zieht eine Linie, bei + und - dreht sie sich, bei [ und ] merkt sie sich einen Ort und kehrt dorthin zurück."),

  H.h2("Schritt 1: ein Baum aus einer Regel"),

  H.p("Der ganze Baum ist eine SUB, die sich selbst aufruft:"),

  H.code(stueck("kap12/1_baum.dh", "' Ein Ast von (x, y)", "END SUB")),

  H.p("Eine Funktion, die sich selbst aufruft, heißt rekursiv. ast zeichnet eine Linie und ruft dann ast zweimal: einmal 25 Grad nach links gedreht, einmal nach rechts, jeweils mit 72 Prozent der Länge und einer Tiefe weniger. Jeder dieser Aufrufe tut dasselbe mit seinem Ast. Das Hauptprogramm ruft ast nur ein einziges Mal, für den Stamm:"),

  H.code(stueck("kap12/1_baum.dh", "WHILE NOT QUITREQUESTED()", "WEND")),

  H.pmix(["Was hält das Ganze an? Die Tiefe. Der Stamm hat Tiefe 9, seine beiden Äste 8, deren Äste 7 … und bei Tiefe 0 zeichnet ast seine Linie und ruft niemanden mehr. ", ["IF tiefe > 0", true], " ist der wichtigste Satz der SUB — ohne ihn hörte sie nie auf. Bei Tiefe 9 sind es 1 + 2 + 4 + … + 512 = 1023 Linien, und die dünnsten sind nur noch 110 · 0,72⁹ ≈ 6 Punkte lang."]),

  H.figure("kap12_1_baum.png", "Der Baum aus einer Regel: Jede Gabel trägt zwei Äste, um 25 Grad gedreht und 72 Prozent so lang. Die Farbe wandert mit der Tiefe von Braun nach Grün.", 420, 262),

  H.h2("Schritt 2: ein Garten aus Bauplänen"),

  H.p("Ein Baum, der aus einer festen SUB wächst, ist immer derselbe Baum. Für einen ganzen Garten mit verschiedenen Pflanzen trennen wir das Wachsen vom Zeichnen. Eine Pflanze wird ein Wort aus Zeichen, und Regeln schreiben dieses Wort um. Für den Farn sind es zwei, als MAP geschrieben — zu jedem Zeichen, wodurch es ersetzt wird:"),

  H.code(['regeln = {"X": "F+[[X]-X]-F[-FX]+X", "F": "FF"}']),

  H.p("Aus dem Anfangswort „X“ wird in der ersten Generation „F+[[X]-X]-F[-FX]+X“. In der zweiten wird jedes X darin wieder durch dieselbe Zeile ersetzt und jedes F durch FF — das Wort ist schon 89 Zeichen lang. Diese Art Bauplan hat der Biologe Aristid Lindenmayer 1968 erfunden, um das Wachsen von Algen zu beschreiben; man nennt sie L-Systeme."),

  H.p("Das Umschreiben ist wieder rekursiv — eine Generation, und dann dasselbe mit einer Tiefe weniger:"),

  H.code(stueck("kap12/2_garten.dh", "' Ein Wort `tiefe` Generationen", "END FUNCTION")),

  H.pmix([["IIF(MAP_HAS(regeln, z), regeln[z], z)", true], " ist die ganze Regel: Gibt es zu z einen Eintrag, nimm ihn, sonst bleibt z, wie es ist. Dann liest die Schildkröte das fertige Wort:"]),

  H.code(stueck("kap12/2_garten.dh", "SELECT CASE MID$(wort, i, 1)", "END SELECT")),

  H.p("Die Klammern sind der Trick, der aus einer Linie eine Pflanze macht. Bei [ legt die Schildkröte Ort und Richtung auf einen Stapel, zeichnet dann einen Seitenzweig, und bei ] holt sie sich den obersten Ort vom Stapel zurück und macht dort weiter, wo der Zweig abging. Ein Stapel, weil Zweige in Zweigen stecken: Das zuletzt Gemerkte kommt zuerst zurück. ARRAY_PUSH legt oben drauf, ARRAY_POP nimmt oben weg."),

  H.p("Jede Pflanze ist ein STRUCT mit ihrem Anfangswort, ihren Regeln, ihrem Winkel und ihrem Platz. Drei davon bilden den Garten — der Bauplan einer Pflanze ist eine einzige Zeile:"),

  H.code(stueck("kap12/2_garten.dh", "pflanzen(\"X\", {", "RGB(120, 90, 50))")),

  H.p("Gezeichnet wird nicht in jedem Bild, sondern nur, wenn der Garten eine Generation weiter ist — in ein Bild, das sein Aussehen behält (RENDERTARGET_NEW mit TRUE, aus Kapitel 2):"),

  H.code(stueck("kap12/2_garten.dh", "SUB gartenZeichnen", "END SUB")),

  H.p("Das ganze Programm:"),

  H.code(zeilen("kap12/2_garten.dh")),

  H.h2("Warum so?"),

  H.h3("Rekursion statt Schleife"),

  H.p("Ein Baum ist rekursiv gebaut: Ein Ast mit seinen Zweigen sieht aus wie ein kleiner Baum. Eine rekursive SUB schreibt genau das hin — „zeichne einen Ast, dann zwei kleinere Bäume an seiner Spitze“. Mit einer Schleife müsste das Programm selbst Buch führen, welche Astspitzen noch offen sind, und genau das tut die Schildkröte mit ihrem Stapel. Die Rekursion führt diesen Stapel unsichtbar: Jeder Aufruf merkt sich seine x, y und winkel, bis seine beiden Kinder fertig sind. Wo die Sache selbst aus kleineren Ausgaben ihrer selbst besteht — Bäume, Ordner mit Unterordnern, verschachtelte Klammern —, ist Rekursion die natürliche Form."),

  H.h3("Wo die Rekursion endet"),

  H.p("Jede rekursive Funktion braucht einen Fall, in dem sie sich nicht mehr selbst ruft, und jeder Aufruf muss diesem Fall näher kommen. In ast ist das die Tiefe, die mit jedem Aufruf um eins kleiner wird, bis sie 0 ist; in ausrollen ebenso. Fehlt eines von beiden — die Prüfung oder das Kleinerwerden —, ruft die Funktion sich ohne Ende. Was dann passiert, zeigt „Was schiefgehen kann“."),

  H.h3("Ein Bauplan als Zeichenkette"),

  H.p("Der Bauplan ist Text, und das hat Vorteile: Man kann ihn ausdrucken, um zu sehen, was die Schildkröte gleich tut; man kann ihn speichern, vergleichen, von Hand ändern. Und eine neue Pflanze braucht kein neues Programm, nur neue Regeln — der Garten hat drei Pflanzen und eine einzige Zeichenfunktion. Die Regeln stehen in einer MAP, weil es um „zu diesem Zeichen gehört dieser Text“ geht: Nachschlagen ist genau, wofür eine MAP da ist."),

  H.h2("Was schiefgehen kann"),

  H.h3("Eine Rekursion ohne Ende"),

  H.code(zeilen("kap12/fehler_rekursion.dh")),
  H.code(["Laufzeitfehler in fehler_rekursion.dh:3: Maximale Aufruftiefe (1000)",
          "ueberschritten -- unendliche Rekursion?"], { out: true }),

  H.p("Die Tiefe wird kleiner, aber niemand fragt, ob sie schon 0 ist. Drachenhauch zählt, wie tief die Aufrufe ineinanderstecken, und hält bei 1000 an — das ist weit mehr, als ein sinnvoller Baum braucht, und viel weniger, als den Rechner in Schwierigkeiten brächte. Die Meldung kommt nach Bruchteilen einer Sekunde."),

  H.h3("Die Tiefe, die den Speicher sprengt"),

  H.p("Die zweite Gefahr ist leiser. Der Bauplan des Buschs ersetzt jedes F durch ein Stück mit acht F darin — das Wort wird in jeder Generation etwa achtmal so lang:"),

  H.code(zeilen("kap12/3_wachstum.dh")),
  H.code([
    "Generation 1: 20 Zeichen",
    "Generation 2: 172 Zeichen",
    "Generation 3: 1388 Zeichen",
    "Generation 4: 11116 Zeichen",
    "Generation 5: 88940 Zeichen",
    "Generation 6: 711532 Zeichen",
    "Generation 7: 5692268 Zeichen",
  ], { out: true }),

  H.p("Bis Generation 7 geht das schnell. Generation 8 hat 45 Millionen Zeichen, Generation 9 schon 364 Millionen — gut 360 Megabyte Text, nach mehreren Sekunden Rechnen. Generation 10 bräuchte fast drei Gigabyte; das sprengt den Speicher vieler Rechner, und es gibt nichts zu sehen, was Generation 4 nicht auch zeigt: Die Striche wären viel kürzer als ein Bildpunkt. Darum hat jede Pflanze im Garten eine eigene größte Tiefe (bisTiefe), und der Busch hört bei 4 auf."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Ändere im Baum die 25 Grad in 10 und dann in 60. Ändere danach 0.72 in 0.5 und in 0.85."),
  H.bulletRich("Drehen 2. ", "Setze die Tiefe des Baums von 9 auf 14. Wie viele Linien sind es jetzt, und sieht man den Unterschied?"),
  H.bulletRich("Drehen 3. ", "Gib dem Farn im Garten den Winkel 15 statt 25."),
  H.bulletRich("Erweitern 1. ", "Wind im Baum: Die Äste neigen sich hin und her, die äußeren mehr als der Stamm."),
  H.bulletRich("Erweitern 2. ", "Jahreszeiten: An den Astspitzen wachsen Blätter, werden dunkelgrün, gelb, rot und fallen ab. Ein Jahr dauert zwölf Sekunden."),
  H.bulletRich("Erfinden. ", "Ein eigenes Regelwerk: Probiere „F wird zu F+F--F+F“ mit 60 Grad (eine Schneeflockenkante), oder die Drachenkurve mit X wird zu X+YF+ und Y wird zu -FX-Y bei 90 Grad. Anhang A zeigt den Drachen."),

  H.figure("kap12_herbst.png", "Eine Lösung zu Erweitern 2: Der Baum im Herbst — die Blätter an den Spitzen sind orange geworden.", 420, 262),

  H.figure("kap12_drache.png", "Eine Möglichkeit für die Erfinden-Übung: die Drachenkurve in Generation 12, eine einzige Linie, die sich immer wieder faltet. Die Farbe wandert von ihrem Anfang zu ihrem Ende.", 420, 262),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Sprache → Maps: ", "MAP_HAS, MAP_GET_OR, Schlüssel durchgehen mit FOR EACH."),
  H.bulletRich("Handbuch, Sprache → Funktionen: SUB und FUNCTION: ", "Rekursion, die Aufruftiefe und wie man eine sehr tiefe Rekursion in eine Schleife umbaut."),
];
