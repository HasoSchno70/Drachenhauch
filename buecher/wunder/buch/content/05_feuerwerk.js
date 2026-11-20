const { zeilen, ohneKopf, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.part("Teil II · Viele Dinge auf einmal"),

  H.p("Im ersten Teil gab es von allem eines: einen Stern, der heranfliegt, eine Fläche, die wabert, einen Zeiger, der kreist. Hundert Sterne waren hundertmal dasselbe Rechenrezept mit einer anderen Zahl. Dieser Teil handelt von Dingen, die kommen und gehen — die entstehen, eine Weile leben und verschwinden, und von denen niemand vorher weiß, wie viele es gerade sind."),

  H.p("Dafür braucht ein Programm drei Fähigkeiten, und alle drei lernt das erste Kapitel dieses Teils: ein Ding mit mehreren Eigenschaften als EINE Sache beschreiben, beliebig viele davon in einem Feld halten und die toten wieder herausnehmen, ohne dabei durcheinanderzukommen."),

  H.chapter("Feuerwerk", 5),

  H.h2("Was wir bauen"),

  H.figure("kap05_feuerwerk.png", "Das fertige Feuerwerk: zwei Raketen sind schon zerplatzt, zwei steigen noch. Jeder Funke zieht einen kurzen Schweif hinter sich her und verblasst in seiner letzten Sekunde.", 480, 300),

  H.p("Raketen steigen vom unteren Rand auf, werden langsamer und zerplatzen an ihrem höchsten Punkt in hundertvierzig Funken. Die Funken fliegen in alle Richtungen, fallen, glühen nach und verlöschen. Alle ein bis zwei Sekunden startet eine neue Rakete, mit der Leertaste startet man zusätzlich welche — so viele man will."),

  H.h2("Der Plan"),

  H.p("Bevor eine Zeile Code dasteht, lohnt es sich, das Programm in Sätzen zu beschreiben. Sätze verraten, welche Dinge es gibt und was mit ihnen passiert:"),

  H.bulletRich("Eine Rakete ", "hat einen Ort, ein Tempo nach oben und eine Farbe. Die Schwere bremst sie. Wenn sie nicht mehr steigt, zerplatzt sie und ist weg."),
  H.bulletRich("Ein Funke ", "hat einen Ort, ein Tempo in beide Richtungen, eine Farbe und eine Lebenszeit. Die Schwere zieht ihn nach unten. Wenn seine Zeit um ist, ist er weg."),
  H.bulletRich("In jedem Bild ", "bewegen sich alle Raketen und alle Funken ein Stück weiter, die abgelaufenen verschwinden, dann wird alles gezeichnet."),

  H.p("Darin stecken die drei Fragen dieses Kapitels. Wie beschreibt man EIN Ding, das fünf oder sechs Eigenschaften hat? Wo bewahrt man eine Menge davon auf, die ständig wächst und schrumpft? Und wie nimmt man Dinge aus dieser Menge heraus, während man sie gerade durchgeht? Die Antworten heißen STRUCT, ARRAY OF mit ARRAY_PUSH und — überraschend wichtig — rückwärts zählen."),

  H.h2("Schritt 1: ein einziger Funke"),

  H.pmix(["Ein Funke hat vier Zahlen, die ihn ausmachen: wo er ist (", ["x", true], ", ", ["y", true], ") und wie schnell er sich in jede Richtung bewegt (", ["vx", true], ", ", ["vy", true], "). Das „v“ kommt von „velocity“, dem englischen Wort für Geschwindigkeit; so heißen diese Größen in fast jedem Spiel und jedem Physikbuch. Ein STRUCT fasst sie zu einem eigenen Typ zusammen:"]),

  H.code(stueck("kap05/1_ein_funke.dh", "STRUCT Funke", "END STRUCT")),

  H.pmix(["Ab jetzt ist ", ["Funke", true], " ein Typ wie ", ["INTEGER", true], " oder ", ["STRING", true], ". ", ["DIM f AS Funke", true], " legt einen an, und mit dem Punkt kommt man an seine Teile: ", ["f.x", true], ", ", ["f.vy", true], ". Ein ", ["NEW", true], " wie bei Klassen braucht es nicht — ein STRUCT entsteht mit seiner Deklaration, alle Zahlen stehen auf 0."]),

  H.p("Bewegen heißt: in jedem Bild ein kleines Stück weiter. Wie groß das Stück ist, hängt davon ab, wie viel Zeit seit dem letzten Bild vergangen ist, und die liefert DELTA() in Sekunden. Drei Zeilen tun alles:"),

  H.code(stueck("kap05/1_ein_funke.dh", "f.vy = f.vy + SCHWERE", "f.y = f.y + f.vy")),

  H.p("Die erste Zeile ist die Schwere: Sie macht das Tempo nach unten in jeder Sekunde um 120 Pixel pro Sekunde größer. Die beiden anderen schieben den Funken um sein Tempo weiter, mal die vergangene Zeit. Mehr Physik steckt in diesem ganzen Buch selten — fast alles, was fliegt, fällt oder hüpft, ist eine Abwandlung dieser drei Zeilen."),

  H.figure("kap05_bahn.png", "Die Bahn eines Funkens, alle 0,1 Sekunden ein Punkt. Die blauen Pfeile zeigen die Geschwindigkeit: Am Anfang steil nach oben, am höchsten Punkt waagrecht, danach immer steiler nach unten. Waagrecht ändert sich nichts — vx bleibt 140.", 460, 290),

  H.p("Das ganze erste Programm wirft den Funken, lässt ihn fliegen und wirft ihn neu, sobald er unten herausfällt:"),

  H.code(zeilen("kap05/1_ein_funke.dh")),

  H.pmix(["Ein Satz zu ", ["werfen(f)", true], ": Die SUB setzt die Felder des Funkens, den man ihr gibt, und der Aufrufer sieht die Änderung. Das liegt daran, dass ein STRUCT als Verweis übergeben wird — die SUB bekommt keine Kopie, sondern denselben Funken. Für Zahlen und Texte gilt das nicht; dort braucht es ", ["BYREF", true], ". Gleich wird diese Eigenschaft wichtig, und einmal in diesem Kapitel stolpert man auch über sie."]),

  H.h2("Schritt 2: hundert Funken"),

  H.pmix(["Ein Funke ist hübsch, ein Feuerwerk sind Hunderte — und wie viele genau, ändert sich in jedem Bild. Ein Feld mit fester Größe wie ", ["DIM funken[500] AS Funke", true], " passt dazu schlecht. Besser ist ein Feld, das mit seinem Inhalt wächst:"]),

  H.code(stueck("kap05/2_viele_funken.dh", "DIM funken AS ARRAY OF Funke")),

  H.pmix(["So angelegt, ist das Feld leer. ", ["ARRAY_PUSH(funken, f)", true], " hängt einen Funken hinten an, ", ["LEN(funken)", true], " sagt, wie viele es gerade sind, und ", ["ARRAY_REMOVE_AT(funken, i)", true], " nimmt den an der Stelle ", ["i", true], " heraus; alle dahinter rücken eins nach vorn. Eine Explosion ist damit eine Schleife, die hundert Funken baut und anhängt:"]),

  H.code(stueck("kap05/2_viele_funken.dh", "SUB explosion", "END SUB")),

  H.pmix(["Jeder Funke bekommt einen zufälligen Winkel und ein zufälliges Tempo. ", ["COS", true], " und ", ["SIN", true], " machen daraus die beiden Teile der Geschwindigkeit: Ein Winkel von 0 zeigt nach rechts, ", ["PI / 2", true], " nach unten (die y-Achse zeigt auf dem Bildschirm nach unten), ", ["PI", true], " nach links. ", ["RND() * 2 * PI", true], " ist also „irgendeine Richtung“. Weil auch das Tempo streut, entsteht eine gefüllte Kugel statt eines dünnen Rings."]),

  H.pmix(["Wichtig ist, wo das ", ["DIM f AS Funke", true], " steht: INNERHALB der Schleife. Ein DIM in einer Schleife fängt jede Runde frisch an — jede Runde entsteht ein neuer Funke, und das Feld bekommt hundert verschiedene. Was passiert, wenn es außerhalb steht, zeigt der Abschnitt „Was schiefgehen kann“."]),

  H.p("Bleibt das Aufräumen. In jedem Bild altern alle Funken, und wessen Zeit um ist, der fliegt aus dem Feld:"),

  H.code(stueck("kap05/2_viele_funken.dh", "FOR i = LEN(funken) - 1 TO 0 STEP -1", "NEXT")),

  H.pmix(["Die Schleife läuft rückwärts, vom letzten Funken zum ersten. Das ist kein Stil, sondern nötig — warum, steht unter „Warum so?“. Und ", ["funken[i].vy = …", true], " schreibt direkt in den Funken im Feld: Ein Index mit Punkt dahinter geht lesend wie schreibend."]),

  H.figure("kap05_2_viele_funken.png", "Der zweite Zwischenstand: hundert Funken, alle zwei Sekunden aus der Mitte, mit der Maus auch woanders. Oben links steht, wie viele gerade leben.", 440, 275),

  H.tip("Selbst ausprobieren", "Klicke schnell hintereinander auf verschiedene Stellen und beobachte die Zahl oben links. Sie steigt mit jedem Klick um hundert und fällt nach anderthalb Sekunden wieder — das Feld wächst und schrumpft, ohne dass das Programm irgendwo eine Größe festlegt."),

  H.h2("Schritt 3: Raketen, Farben, Glut"),

  H.p("Aus der Explosion an einer festen Stelle wird ein Feuerwerk, wenn Raketen die Explosionen bringen. Eine Rakete ist ein zweiter STRUCT; sie fliegt nur senkrecht, braucht also kein vx, dafür einen Farbton:"),

  H.code(stueck("kap05/3_feuerwerk.dh", "STRUCT Rakete", "END STRUCT")),

  H.code(stueck("kap05/3_feuerwerk.dh", "SUB raketeStarten", "END SUB")),

  H.pmix(["Das Tempo ist negativ, weil „nach oben“ auf dem Bildschirm kleinere y bedeutet. Die Schwere bremst die Rakete, bis ", ["vy", true], " nicht mehr negativ ist — dann ist sie am höchsten Punkt, und genau dort zerplatzt sie:"]),

  H.code(stueck("kap05/3_feuerwerk.dh", "SUB raketenBewegen", "END SUB")),

  H.pmix(["Auch hier läuft die Schleife rückwärts, aus demselben Grund wie bei den Funken. ", ["zerplatzen", true], " ist die Explosion aus Schritt 2 mit zwei Neuerungen: Jeder Funke bekommt eine Farbe, und er lebt unterschiedlich lange."]),

  H.code(stueck("kap05/3_feuerwerk.dh", "f.farbe = HSV", "f.leben = 1.2")),

  H.pmix([["HSV(farbton, sättigung, helligkeit)", true], " beschreibt eine Farbe so, wie man über Farben spricht: Der Farbton ist ein Winkel auf dem Farbkreis (0 Rot, 120 Grün, 240 Blau), die Sättigung sagt, wie kräftig, die Helligkeit, wie hell. Jede Rakete würfelt EINEN Farbton, jeder ihrer Funken weicht um bis zu 20 Grad davon ab — die Explosion ist erkennbar rot oder blau, aber nicht eintönig."]),

  H.p("Den größten Unterschied macht das Zeichnen. Ein Funke ist nicht mehr ein Punkt, sondern drei Schichten übereinander:"),

  H.code(stueck("kap05/3_feuerwerk.dh", "DIM f AS Funke : f = funken[i]", "CIRCLE(x, y, 2, RGBA(255, 255, 255, a))")),

  H.bulletRich("Der Schweif ", "ist eine Linie von der Stelle, an der der Funke vor 0,08 Sekunden war, bis zu ihm. Schnelle Funken ziehen lange Striche, langsame kurze — das Auge liest daraus Bewegung, auch auf einem Standbild."),
  H.bulletRich("Der Schein ", "ist ein großer Kreis in der Funkenfarbe, fast durchsichtig. Viele davon übereinander ergeben das Leuchten in der Mitte einer frischen Explosion."),
  H.bulletRich("Der Kern ", "ist ein kleiner weißer Punkt. Echte Funken sind in der Mitte weiß, die Farbe sieht man nur am Rand."),

  H.pmix(["Alle drei Schichten werden mit der Deckkraft ", ["a", true], " gemalt, und die hängt an der Lebenszeit: ", ["CLAMP(f.leben, 0.0, 1.0)", true], " ist 1, solange der Funke mehr als eine Sekunde vor sich hat, und sinkt in der letzten Sekunde auf 0. Mal 255 ergibt das die Deckkraft für ", ["RGBA", true], ". So verlischt ein Funke weich, statt plötzlich zu verschwinden."]),

  H.p("Hier ist das ganze Programm:"),

  H.code(zeilen("kap05/3_feuerwerk.dh")),

  H.h2("Warum so?"),

  H.p("Fast jede Zeile oben ist eine Entscheidung, zu der es eine naheliegende Alternative gibt. Die wichtigsten, jeweils mit dem Grund:"),

  H.h3("STRUCT statt fünf Feldern"),

  H.pmix(["Man kann viele Funken auch ohne STRUCT halten: ein Feld für alle x, eines für alle y, eines für alle vx und so weiter. Fach 37 in jedem Feld ist dann Funke 37. Das funktioniert (der Einstieg baut so sein erstes Feuerwerk), hat aber eine Schwäche, die mit jedem Feld größer wird: Nichts hält die Felder zusammen. Wer einen Funken löscht, muss es in allen fünf Feldern tun, an derselben Stelle. Vergisst er eines, gehört ab da jedes y zum falschen x — und das Programm meldet keinen Fehler, es sieht nur seltsam aus."]),

  H.pmix(["Mit dem STRUCT ist ein Funke EIN Wert. ", ["ARRAY_REMOVE_AT(funken, i)", true], " nimmt ihn als Ganzes heraus, ", ["ARRAY_PUSH", true], " hängt ihn als Ganzes an. Und eine neue Eigenschaft — gleich in den Übungen bekommt der Funke eine — ist eine Zeile im STRUCT, statt ein weiteres Feld, das man überall mitpflegen muss. Die Regel dahinter: Was zusammengehört, soll im Programm auch zusammenstehen."]),

  H.h3("Ein wachsendes Feld statt eines festen Vorrats"),

  H.p("Die andere übliche Bauweise ist ein Vorrat fester Größe: fünfhundert Funken, immer, und ein abgelaufener wird sofort als neuer wiederverwendet. Das ist richtig, wenn die Zahl der Dinge ungefähr gleich bleibt — ein Springbrunnen, ein Schneefall. Beim Feuerwerk schwankt sie zwischen null und weit über tausend, je nachdem, wie oft man die Leertaste drückt. Ein fester Vorrat müsste für den schlimmsten Fall bemessen sein und meistens leer mitgeschleppt werden, oder er wäre im entscheidenden Moment zu klein. Das wachsende Feld hat immer genau so viele Fächer, wie es Funken gibt."),

  H.h3("Rückwärts löschen"),

  H.p("Das ist die wichtigste Entscheidung des Kapitels, und sie sieht am unscheinbarsten aus. Am einfachsten sieht man sie an Zahlen statt an Funken. Aus dem Feld [5, 0, 0, 7] sollen alle Nullen verschwinden:"),

  H.code(ohneKopf("kap05/4_rueckwaerts.dh")),
  H.code(["5, 7"], { out: true }),

  H.pmix(["Rückwärts geht es gut, weil ", ["ARRAY_REMOVE_AT", true], " nur die Elemente HINTER der gelöschten Stelle verschiebt — und die hat die Schleife schon hinter sich. Alles, was noch vor ihr liegt, steht noch da, wo es stand. Vorwärts dagegen rückt das nächste Element genau in die Lücke, über die die Schleife gerade hinweggeht. Was dann passiert, steht im nächsten Abschnitt."]),

  H.h3("Zeit in Sekunden"),

  H.pmix(["Alle Größen im Programm sind in Sekunden gerechnet: ", ["SCHWERE", true], " ist 120 Pixel pro Sekunde pro Sekunde, ein Funke lebt 1,2 bis 2 Sekunden, die nächste Rakete kommt nach 0,6 bis 1,4 Sekunden. Die Alternative wäre, in Bildern zu zählen: „der Funke lebt 90 Bilder“. Das ist auf einem Bildschirm mit 60 Bildern pro Sekunde dasselbe — und auf einem mit 144 Bildern pro Sekunde zweieinhalbmal so schnell vorbei. Mit ", ["DELTA()", true], " läuft das Feuerwerk überall gleich, und die Zahlen im Programm bedeuten etwas, das man sich vorstellen kann."]),

  H.h3("Zerplatzen am höchsten Punkt"),

  H.pmix(["Man könnte jede Rakete nach einer festen Zeit zerplatzen lassen. Am höchsten Punkt (", ["vy >= 0", true], ") ist aber natürlicher — so tun es echte Raketen — und es lässt sich vorausrechnen. Eine Rakete, die mit dem Tempo v startet, steigt um v² / (2 · SCHWERE). Mit 230 bis 300 Pixeln pro Sekunde sind das 230² / 240 ≈ 220 bis 300² / 240 = 375 Pixel. Das Fenster ist 500 hoch: Jede Rakete zerplatzt sicher im Bild, und keine zu nah am Boden. Die Zahlen in ", ["raketeStarten", true], " sind nicht geraten, sie sind so gewählt."]),

  H.h3("Ganzzahlig teilen"),

  H.pmix(["In ", ["RGBA(rot, gruen, blau, a \\ 2)", true], " steht ein Rückstrich, kein Schrägstrich. ", ["/", true], " liefert in Drachenhauch immer eine Kommazahl, auch bei 200 / 2. Ein Farbanteil ist aber eine ganze Zahl von 0 bis 255; ", ["\\", true], " teilt und rundet ab, das Ergebnis ist ein ", ["INTEGER", true], ". An dieser Stelle ginge auch ", ["/", true], ", denn RGBA rundet selbst. Der Rückstrich ist trotzdem die richtige Gewohnheit: Wo das Ergebnis in einer INTEGER-Variablen landet, ist ", ["/", true], " ein Fehler, der nur bei ungeraden Zahlen auffällt — der dritte Fall unter „Was schiefgehen kann“."]),

  H.h2("Was schiefgehen kann"),

  H.h3("Vorwärts löschen"),

  H.p("Dasselbe Aufräumen wie oben, nur in der Richtung, die man zuerst hinschreibt:"),

  H.code(zeilen("kap05/fehler_vorwaerts.dh")),
  H.code(["Laufzeitfehler in fehler_vorwaerts.dh:4: Index 3 ausserhalb [0..2]",
          "in Dimension 1 -- Felder zaehlen ab 0: DIM a[3] hat a[0] bis a[2]"], { out: true }),

  H.pmix(["Hier geschehen zwei Dinge. Bei ", ["i = 1", true], " wird die erste Null gelöscht; die zweite Null rutscht auf Platz 1 — den die Schleife gerade erledigt hat. Im nächsten Durchgang steht ", ["i", true], " auf 2 und sieht die 7. Die zweite Null wird übersprungen. Und weil ", ["LEN(zahlen) - 1", true], " nur einmal ausgerechnet wird, als das Feld noch vier Elemente hatte, läuft die Schleife bis ", ["i = 3", true], ", obwohl es nur noch die Plätze 0 bis 2 gibt. Das meldet dhrt."]),

  H.p("Der Hinweis am Ende der Meldung („Felder zählen ab 0“) ist ein allgemeiner und trifft hier nicht den Kern. Der Kern steht davor: Index 3, aber nur noch [0..2] — das Feld ist während der Schleife kürzer geworden. Im Feuerwerk bricht das Programm darum ab, sobald zum ersten Mal ein Funke verlischt. Das ist noch das Glück im Unglück: Mit einer WHILE-Schleife, die die Länge jede Runde neu fragt, bliebe nur das Überspringen übrig — und das fällt bei Funken nicht auf, der übersprungene lebt einfach ein Bild länger."),

  H.warn("Wer in einer Schleife Elemente aus dem Feld löscht, das er durchläuft, zählt rückwärts: FOR i = LEN(feld) - 1 TO 0 STEP -1."),

  H.h3("Der Funke vor der Schleife"),

  H.pmix(["Was passiert, wenn das ", ["DIM f AS Funke", true], " vor der Schleife steht statt darin?"]),

  H.code(zeilen("kap05/fehler_ein_funke.dh")),
  H.code(["30.0", "30.0", "30.0"], { out: true }),

  H.pmix(["Drei Einträge, dreimal 30 statt 10, 20, 30. Es gibt nur EINEN Funken, und das Feld enthält dreimal einen Verweis auf ihn. Jede Runde überschreibt ihn, und am Ende zeigen alle drei Einträge dasselbe. Im Feuerwerk sähe das so aus: Hundertvierzig Funken fliegen genau aufeinander, man sieht einen einzigen. Keine Meldung, kein Absturz — nur ein sehr langweiliges Feuerwerk. Die Abhilfe ist, das ", ["DIM", true], " in die Schleife zu ziehen."]),

  H.h3("Die Rakete, die aus dem Bild fliegt"),

  H.pmix(["Wer die Raketen schneller machen will und ", ["r.vy = -RANDINT(400, 450)", true], " schreibt, sieht keine Explosionen mehr — nur Funken, die von oben ins Bild regnen. Nach der Rechnung oben steigt eine Rakete mit 400 Pixeln pro Sekunde um 400² / 240 ≈ 667 Pixel, das Fenster ist aber nur 500 hoch. Sie zerplatzt oberhalb des sichtbaren Bereichs. Auch hier gibt es keine Meldung; Zeichnen außerhalb des Fensters ist erlaubt und tut nichts."]),

  H.h3("Durch zwei geteilt"),

  H.p("Eine Halbierung, die mit 200 tadellos läuft:"),

  H.code(zeilen("kap05/fehler_teilen.dh")),
  H.code(["Laufzeitfehler in fehler_teilen.dh:3: Zuweisung an 'halb': FLOAT",
          "127.5 passt nicht verlustfrei in INTEGER -- fuer ganzzahlige",
          "Division \\ statt / nehmen, sonst mit INT()/ROUND() runden"], { out: true }),

  H.pmix(["Mit 255 ergibt ", ["a / 2", true], " 127,5, und das passt in keine ganze Zahl. dhrt rundet nicht heimlich, sondern hält an und nennt beide Auswege: ", ["\\", true], ", wenn ganzzahlig geteilt werden soll, ", ["INT", true], " oder ", ["ROUND", true], ", wenn bewusst gerundet wird. Schon vor dem Start steht in der Problemliste der IDE eine Warnung zu dieser Zeile — wer sie liest, findet den Fehler, bevor er mit 255 zuschlägt."]),

  H.h2("Übungen"),

  H.p("Bei den Drehen-Übungen gilt: erst vorhersagen, aufschreiben, dann starten. Die Vorhersage ist die eigentliche Übung — wer richtig lag, hat das Programm verstanden, und wer danebenlag, hat gerade etwas gelernt. Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Setze SCHWERE auf 30, dann auf 400. Wo zerplatzen die Raketen jeweils, und wie sehen die Explosionen aus? Rechne die Höhen vorher aus."),
  H.bulletRich("Drehen 2. ", "Setze FUNKEN_JE_RAKETE auf 1000. Was ändert sich am Bild, was am Gefühl beim Zusehen? Zähle mit, wie viele Funken bei fünf schnellen Leertasten gleichzeitig leben."),
  H.bulletRich("Drehen 3. ", "Ersetze r.farbton + RANDINT(-20, 20) durch RANDINT(0, 359). Was geht verloren?"),
  H.bulletRich("Erweitern 1. ", "Ein Mausklick soll eine Rakete starten, die genau in der Höhe des Klicks zerplatzt. Hinweis: Die Formel für die Steighöhe lässt sich nach v umstellen."),
  H.bulletRich("Erweitern 2. ", "Eine steigende Rakete soll eine Spur aus kleinen, kurzlebigen Funken hinter sich herziehen. Braucht es dafür einen neuen STRUCT?"),
  H.bulletRich("Erfinden. ", "Baue ein Feuerwerk in zwei Stufen: Einige Funken zerplatzen selbst noch einmal, wenn sie verlöschen. Oder etwas ganz anderes — eine Explosion in Herzform, Raketen, die sich in der Luft drehen, ein Finale, bei dem zwanzig auf einmal starten. Erfinden-Übungen haben keine Musterlösung; Anhang A zeigt eine Möglichkeit."),

  H.figure("kap05_kaskade.png", "Eine Möglichkeit für die Erfinden-Übung: Jeder zehnte Funke ist ein Knaller und zerplatzt beim Verlöschen in zwanzig weiße Funken.", 440, 275),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Sprache → Arrays: ", "ARRAY OF, ARRAY_PUSH, ARRAY_REMOVE_AT und alles, was Felder sonst noch können (Sortieren, Ausschnitte, FOR EACH)."),
  H.bulletRich("Handbuch, Sprache → Klassen und Strukturen: ", "STRUCT im Vergleich zu CLASS. Kapitel 20 dieses Buchs, die Partikelmaschine, greift die Funken noch einmal auf."),
  H.bulletRich("Handbuch, Kern-Builtins → Farb-Helfer: ", "HSV, RGBA und die Zerlegung einer Farbe mit RED, GREEN, BLUE."),
];
