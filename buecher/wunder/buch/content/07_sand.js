const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Fallender Sand", 7),

  H.h2("Was wir bauen"),

  H.figure("kap07_sandkasten.png", "Der Sandkasten: Sand häuft sich auf einem Brett und rutscht über die Kante, Wasser fließt das andere Brett hinab und breitet sich am Boden aus.", 470, 295),

  H.p("Ein Kasten aus 200 mal 125 Zellen, jede aus einem Stoff: leer, Sand, Wasser oder Stein. Sand fällt und häuft sich zu Kegeln, Wasser fällt und fließt zur Seite, bis es eben ist, Stein bleibt, wo er ist. Mit der Maus malt man Stoffe hinein. Spiele dieser Art heißen Falling-Sand-Spiele, und hinter dem lebendigen Bild stecken nur ein paar Regeln, je Stoff eine."),

  H.h2("Der Plan"),

  H.bulletRich("Die Welt ", "ist ein Feld wie im Spiel des Lebens, aber jede Zelle trägt einen Stoff statt 0 oder 1."),
  H.bulletRich("Jeder Stoff ", "hat eine Regel: Sand will nach unten, sonst schräg nach unten; Wasser ebenso, und sonst zur Seite; Stein tut nichts."),
  H.bulletRich("Bewegen ", "heißt tauschen: Fällt Sand in eine leere Zelle, tauschen die beiden Zellen ihren Inhalt."),
  H.bulletRich("Sechzig Mal je Sekunde ", "geht ein Schritt durch die ganze Welt."),

  H.h2("Schritt 1: Sand"),

  H.p("Die Stoffe könnten Zahlen sein — 0 leer, 1 Sand. Besser sind Namen, und dafür gibt es ENUM:"),

  H.code(stueck("kap07/1_sand.dh", "ENUM Stoff", "DIM welt[1 TO B")),

  H.pmix(["Ein ENUM ist eine Liste von Namen, die Drachenhauch der Reihe nach durchnummeriert: ", ["Stoff.LEER", true], " ist 0, ", ["Stoff.SAND", true], " ist 1. Im Programm schreibt man die Namen. Und ", ["Stoff", true], " ist zugleich ein Typ: Das Feld welt besteht aus Stoffen."]),

  H.p("Zwei kleine Helfer machen die Regeln kurz. frei fragt, ob eine Zelle im Feld und leer ist; tauschen vertauscht zwei Zellen:"),

  H.code(stueck("kap07/1_sand.dh", "' Ist (x, y) im Feld", "END FUNCTION")),
  H.code(stueck("kap07/1_sand.dh", "SUB tauschen", "END SUB")),

  H.p("Damit ist die Regel für Sand fast ein Satz:"),

  H.code(stueck("kap07/1_sand.dh", "' Sand faellt;", "END SUB")),

  H.pmix(["Ist unter dem Korn frei, fällt es. Sonst versucht es schräg nach unten — zuerst nach einer zufälligen Seite, dann nach der anderen. Der Zufall sorgt dafür, dass keine Seite bevorzugt wird; was ohne ihn geschieht, probiert Drehen 2 aus. Ein Schritt geht die ganze Welt durch, ", ["von unten nach oben", false], ":"]),

  H.code(stueck("kap07/1_sand.dh", "' Ein Schritt der Welt", "END SUB")),

  H.p("Gezeichnet wird wie das Plasma in Kapitel 2: jede Zelle ein Punkt in einem kleinen Bild, das viermal so groß auf den Schirm kommt. Und die Schleife rechnet in festen Schritten:"),

  H.code(stueck("kap07/1_sand.dh", "' Feste Schritte", "WEND")),

  H.figure("kap07_1_sand.png", "Der erste Stand: Aus einer Düse rieselt Sand und wächst zu einem Kegel, dessen Flanken genau 45 Grad steil sind.", 420, 262),

  H.p("Der Kegel ist ein schönes Beispiel dafür, wie aus einer einfachen Regel eine Form entsteht. Niemand hat 45 Grad hineingeschrieben; sie kommen daher, dass ein Korn nur einen Schritt schräg rutschen kann."),

  H.h2("Schritt 2: Wasser, Stein und ein Pinsel"),

  H.p("Das ENUM bekommt zwei Stoffe dazu, und die Welt einen zweiten Bewohner. Wasser fällt wie Sand; kann es nicht fallen, fließt es zur Seite, so weit es frei ist, höchstens vier Zellen:"),

  H.code(stueck("kap07/2_sandkasten.dh", "' Wasser faellt wie Sand", "END SUB")),

  H.p("Sand sinkt im Wasser: Für ihn zählt eine Wasserzelle wie eine leere. Das sagt eine eigene Funktion, die sandFaellt statt frei fragt:"),

  H.code(stueck("kap07/2_sandkasten.dh", "' Kann Sand dorthin?", "END FUNCTION")),

  H.p("Welche Regel für eine Zelle gilt, entscheidet SELECT CASE nach ihrem Stoff:"),

  H.code(stueck("kap07/2_sandkasten.dh", "SELECT CASE welt[x, y]", "END SELECT")),

  H.p("Stein und leere Zellen kommen gar nicht vor — für sie gibt es nichts zu tun. Mit der Maus malt das Programm einen kleinen Kreis aus dem gewählten Stoff, die Tasten 1, 2, 3 und 0 wählen ihn. Und die Farbe eines Korns streut ein wenig, je nach seiner Stelle:"),

  H.code(stueck("kap07/2_sandkasten.dh", "FUNCTION farbe", "END FUNCTION")),

  H.p("Das ganze Programm:"),

  H.code(zeilen("kap07/2_sandkasten.dh")),

  H.h2("Warum so?"),

  H.h3("ENUM statt Zahlen"),

  H.p("Mit Zahlen stünde im Programm IF welt[x, y] = 2 THEN, und man müsste wissen, dass 2 Wasser ist. Mit dem ENUM steht da, was gemeint ist. Und Drachenhauch kennt die Namen: Ein Tippfehler wie Stoff.SNAD fällt vor dem Start auf, mit Vorschlag — unten unter „Was schiefgehen kann“. Eine falsche Zahl fiele nie auf; das Programm malte nur ein seltsames Bild."),

  H.h3("Eine Welt genügt — von unten nach oben"),

  H.p("Das Spiel des Lebens brauchte zwei Felder, weil jede Zelle die vorige Generation sehen muss. Beim Sand ist es anders: Ein Korn zieht nur in eine leere Zelle. Entscheidend ist die Reihenfolge. Geht der Schritt von unten nach oben durch die Welt, fällt ein Korn in eine Zeile, die schon erledigt ist, und wird in diesem Schritt nicht noch einmal bewegt. Ginge er von oben nach unten, fände er das Korn in der nächsten Zeile wieder, es fiele noch einmal, und noch einmal — bis zum Boden, in einem einzigen Schritt. Es ist dieselbe Überlegung wie beim Löschen in Kapitel 5: Die Richtung, in der man ein Feld durchgeht, gehört zur Rechnung."),

  H.h3("Abwechselnd links und rechts"),

  H.p("Für die Seite gilt dasselbe in klein: Wasser, das nach rechts fließt, liegt danach in einer Spalte, die die Schleife noch vor sich hat, und kann im selben Schritt weiterfließen. Wasser nach links nicht. Damit es keine Vorzugsrichtung gibt, geht jeder zweite Schritt die Zeilen von rechts nach links durch."),

  H.h3("Feste Schritte statt DELTA"),

  H.p("Bisher hat jedes Programm seine Bewegung mit DELTA() gemessen. Eine Zelle kann aber nicht 0,3 Zellen fallen; sie fällt eine oder keine. Darum zählt die Schleife die Zeit zusammen und macht für jede sechzigstel Sekunde einen Schritt — bei einem langsamen Rechner zwei je Bild, bei einem schnellen manchmal keinen. So rieselt der Sand überall gleich schnell. Das MIN(…, 0.1) ist eine Bremse: Hängt der Rechner einmal eine Sekunde, holt das Programm nicht sechzig Schritte auf einmal nach, sondern höchstens sechs — sonst würde das Nachholen selbst so lange dauern, dass es nie wieder aufholt."),

  H.h3("Regeln als kleine Funktionen"),

  H.p("Jede Regel ist eine eigene SUB, und schritt sagt nur, welche für welchen Stoff gilt. Ein neuer Stoff ist darum ein neuer Name im ENUM, eine neue SUB und eine Zeile im SELECT CASE — die Erfinden-Übung baut so Holz und Feuer ein, ohne die Regeln für Sand und Wasser anzufassen."),

  H.h2("Was schiefgehen kann"),

  H.h3("Ein Name, den das ENUM nicht kennt"),

  H.code(zeilen("kap07/fehler_enum.dh")),
  H.code(["fehler_enum.dh:7: Compile-Fehler: ENUM stoff hat keinen Wert 'snad'.",
          "Meintest du sand? Die Werte heissen leer, sand, wasser"], { out: true }),

  H.p("Die Meldung kommt vor dem Start und nennt alle Werte. (Bis zur Fassung 2026.52 kam sie erst, wenn die Zeile lief — und stand sie in einer SUB, die nie lief, gar nicht. Gefunden beim Schreiben dieses Kapitels.)"),

  H.h3("Eine Zahl, die kein Stoff ist"),

  H.pmix(["Was ein ENUM nicht verhindert: ", ["welt[x, y] = 7", true], ". Eine Variable vom Typ Stoff nimmt jede ganze Zahl an, auch eine, die keinem Namen entspricht. In der Welt wäre das eine Zelle, die keine Regel kennt und die farbe in der Hintergrundfarbe malt — unsichtbar und unbeweglich. Wer mit den Namen schreibt, kann diesen Fehler nicht machen."]),

  H.h3("Die letzte Zeile"),

  H.p("schritt beginnt bei H - 1, nicht bei H: In der untersten Zeile kann nichts mehr fallen. Das ist richtig, solange jeder Stoff nur nach unten will. Kommt ein Stoff dazu, der nach oben steigt — Feuer in der Erfinden-Übung —, dann rechnet die unterste Zeile für ihn nie: Ein Feuer, das man dorthin setzt, brennt ewig und rührt sich nicht. Die Lösung im Anhang setzt die erste Flamme darum eine Zeile höher."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Lass Wasser seitlich nur eine Zelle weit fließen statt vier. Wie sieht der See am Boden jetzt aus?"),
  H.bulletRich("Drehen 2. ", "Ersetze in sandFaellt und in wasserFliesst den Zufall durch s = -1: Jedes Korn und jeder Tropfen will zuerst nach links. Was passiert mit den Haufen, was mit dem Wasser?"),
  H.bulletRich("Drehen 3. ", "Setze TAKT auf 1.0 / 240. Was ändert sich, und warum nicht die Form der Haufen?"),
  H.bulletRich("Erweitern 1. ", "Das Mausrad macht den Pinsel größer und kleiner."),
  H.bulletRich("Erweitern 2. ", "Die Taste G dreht die Schwerkraft um: Sand und Wasser fallen nach oben. Was muss sich außer der Richtung des Fallens noch umdrehen?"),
  H.bulletRich("Erfinden. ", "Neue Stoffe: Holz und Feuer, das Holz frisst; Öl, das auf Wasser schwimmt; Säure, die Stein auflöst; Pflanzen, die wachsen, wo Wasser ist. Anhang A zeigt Holz und Feuer."),

  H.figure("kap07_feuer.png", "Eine Möglichkeit für die Erfinden-Übung: Ein Stapel Holz am linken Rand brennt von unten nach oben ab.", 420, 262),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Sprache → ENUM: ", "die einzeilige Form, eigene Werte und ENUM_NAME, das zu einer Zahl den Namen liefert."),
  H.bulletRich("Handbuch, Sprache → SELECT CASE: ", "Listen, Bereiche und Vergleiche wie CASE IS > 100."),
];
