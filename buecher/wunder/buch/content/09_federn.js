const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.part("Teil III · Bewegung, die sich echt anfühlt"),

  H.p("Die Dinge in Teil II haben sich bewegt, aber nach einfachen Regeln: Ein Funke fällt, ein Korn rutscht, ein Vogel lenkt. Keines hat das andere gezogen oder getragen. In diesem Teil hängen Dinge aneinander, stoßen sich, federn und kommen sanft zur Ruhe — und das Auge merkt sofort, wenn es sich falsch anfühlt."),

  H.p("Zwei Werkzeuge kommen dazu. Das erste ist Mathematik, die man schreiben kann, wie man sie denkt: Punkte und Richtungen als ein Wert statt zwei Zahlen. Das zweite ist die Frage, wann man so etwas selbst rechnet und wann man eine fertige Physik nimmt. Kapitel 9 rechnet selbst, Kapitel 10 nimmt die fertige — und erklärt, warum das meistens die bessere Wahl ist."),

  H.chapter("Federn und Seile", 9),

  H.h2("Was wir bauen"),

  H.figure("kap09_bruecke.png", "Die Hängebrücke, an deren Mitte die Maus zieht: Die Fahrbahn gibt nach, die Hänger neigen sich, das Tragseil wird zum V.", 470, 294),

  H.p("Eine Hängebrücke zwischen zwei Türmen: ein Tragseil im Bogen, eine Fahrbahn aus Holz und dazwischen senkrechte Hänger. Mit der Maus greift man einen Punkt und zieht — die Brücke gibt nach, schwingt und kommt wieder zur Ruhe, wenn man loslässt. Alles daran besteht aus zwei Zutaten: Punkten, die fallen, und Gliedern, die zwei Punkte auf Abstand halten."),

  H.h2("Der Plan"),

  H.bulletRich("Punkte: ", "Jeder Punkt hat einen Ort und fällt, wie die Funken in Kapitel 5. Manche sind fest, wie die Turmspitzen."),
  H.bulletRich("Glieder: ", "Ein Glied verbindet zwei Punkte und will eine bestimmte Länge haben. Ist es zu lang oder zu kurz, rückt es die beiden zurecht."),
  H.bulletRich("Ein Schritt: ", "Erst fallen alle Punkte ein Stück, dann ziehen alle Glieder ihre Punkte wieder zurecht — nicht einmal, sondern zwanzigmal, denn jedes Glied verschiebt dabei auch die Punkte seiner Nachbarn."),

  H.h2("Schritt 1: ein Seil"),

  H.p("Bisher hatte jedes Ding ein x und ein y, und jede Rechnung stand zweimal da: einmal für x, einmal für y. In diesem Kapitel rechnen wir mit Punkten und Richtungen, und dafür hat Drachenhauch den Typ VEC2 aus dem Modul vec2 — ein Wert aus zwei Zahlen, mit dem man rechnen kann wie mit einer:"),

  H.code([
    "IMPORT \"vec2\"",
    "DIM a AS VEC2 : a = VEC2_NEW(3, 4)",
    "DIM b AS VEC2 : b = VEC2_NEW(1, 1)",
    "PRINT a + b              ' Vec2(4.0, 5.0)",
    "PRINT a * 2              ' Vec2(6.0, 8.0)",
    "PRINT a.x, a.y           ' 3.0  4.0",
    "PRINT VEC2_LENGTH(a)     ' 5.0",
    "PRINT VEC2_DISTANCE(a, b)",
  ]),
  H.code([
    "Vec2(4.0, 5.0)",
    "Vec2(6.0, 8.0)",
    "3.0 4.0",
    "5.0",
    "3.605551275463989",
  ], { out: true }),

  H.pmix([["a + b", true], " addiert beide Teile, ", ["a * 2", true], " verdoppelt beide. VEC2_LENGTH ist die Länge — derselbe Pythagoras wie HYPOT im Schwarm —, VEC2_DISTANCE der Abstand zweier Punkte. Ein Seil aus 20 Punkten sind drei Felder: wo jeder Punkt ist, wo er einen Schritt vorher war, und ob er festgehalten wird:"]),

  H.code(stueck("kap09/1_seil.dh", "DIM ort AS ARRAY OF VEC2", "NEXT")),

  H.p("Wozu „wo er vorher war“? Hier steckt der Trick dieses Kapitels. Ein Punkt hat keine gespeicherte Geschwindigkeit. Wie schnell er ist, sieht man daran, wie weit er seit dem letzten Schritt gekommen ist — und genau so weit fliegt er im nächsten Schritt weiter:"),

  H.code(stueck("kap09/1_seil.dh", "' Verlet:", "END SUB")),

  H.pmix([["ort[k] - alt[k]", true], " ist der letzte Schritt. Ihn noch einmal draufzurechnen heißt: weiterfliegen wie bisher. Mal 0.995 verliert er dabei ein halbes Prozent, wie durch Luft. Dazu kommt die Schwerkraft. Weil hier ein Weg dazukommt und keine Geschwindigkeit, ist es SCHWERE mal TAKT zum Quadrat — eine Beschleunigung über einen Zeitschritt ergibt so viel Weg. Das Verfahren heißt nach dem Physiker Loup Verlet, der es 1967 für Moleküle benutzt hat; in Spielen ist es seit gut zwanzig Jahren ein üblicher Weg für Seile und Stoffe."]),

  H.p("Dann die Glieder. Ist der Abstand zweier Nachbarn größer als GLIED, rücken beide aufeinander zu, je um die Hälfte des Zuviel; ist er kleiner, rücken sie auseinander:"),

  H.code(stueck("kap09/1_seil.dh", "' Zwei Punkte wieder", "END SUB")),

  H.pmix([["zuviel", true], " ist das Stück, um das das Glied zu lang ist, als Richtung von a nach b: d hat die Länge l, mal ", ["(l - GLIED) / l", true], " ergibt die Länge l - GLIED. Ist das Glied zu kurz, ist zuviel negativ und zeigt in die andere Richtung — dieselbe Zeile schiebt dann auseinander. haelt sagt, ob ein Punkt festgehalten wird, von einem Nagel oder von der Maus."]),

  H.code(stueck("kap09/1_seil.dh", "SUB schritt", "END SUB")),

  H.p("Warum zwanzig Runden? Wenn ein Glied seine beiden Punkte zurechtrückt, verschiebt es dabei auch die Glieder daneben, die eben noch stimmten. Eine Runde über alle Glieder macht das Seil nur ungefähr richtig. Jede weitere Runde macht es genauer; zwanzig sind genug, dass das Auge keinen Unterschied mehr sieht."),

  H.p("Und die Maus: Ein Punkt, den sie hält, geht dorthin, wo sie ist. Sein „vorher“ ist der Ort davor — lässt man los, fliegt er mit dem Schwung weiter, den die Maus ihm gegeben hat:"),

  H.code(stueck("kap09/1_seil.dh", "IF gegriffen >= 0 THEN", "END IF")),

  H.figure("kap09_1_seil.png", "Der erste Stand: Das Seil war waagrecht ausgelegt und fällt. Oben ist es schon fast senkrecht, unten liegt es noch waagrecht.", 420, 262),

  H.h2("Schritt 2: die Brücke"),

  H.p("Das Seil hatte eine einfache Gestalt: Punkt 0 mit Punkt 1, Punkt 1 mit Punkt 2 und so weiter. Eine Brücke ist ein Netz — Fahrbahn, Tragseil und Hänger. Damit eine einzige Rechnung für alles reicht, werden die Glieder eine eigene Liste:"),

  H.code(stueck("kap09/2_bruecke.dh", "' Die Glieder:", "DIM gliedFarbe")),

  H.p("Zwei kleine Helfer bauen das Netz. punkt legt einen Punkt an und sagt, welche Nummer er bekommen hat; verbinden legt ein Glied an, genau so lang, wie die beiden Punkte gerade auseinander sind:"),

  H.code(stueck("kap09/2_bruecke.dh", "' Einen Punkt anlegen", "END FUNCTION")),
  H.code(stueck("kap09/2_bruecke.dh", "' Zwei Punkte verbinden", "END SUB")),

  H.p("Damit liest sich der Bau der Brücke fast wie eine Beschreibung: Für jede der 25 Stellen ein Punkt der Fahrbahn auf gerader Linie und einer des Tragseils auf einem Bogen; dann je ein Hänger, ein Stück Tragseil und ein Stück Fahrbahn:"),

  H.code(stueck("kap09/2_bruecke.dh", "' Die Bruecke:", "NEXT", 2)),

  H.p("ziehen bekommt jetzt die Nummer eines Glieds und schaut in der Liste nach, welche Punkte es verbindet und wie lang es sein soll. Sonst ist es dieselbe Rechnung wie beim Seil; neu ist nur, dass ein Glied zwischen zwei festen Punkten gar nichts tun kann:"),

  H.code(stueck("kap09/2_bruecke.dh", "' Glied k wieder", "END SUB")),

  H.p("Und gezeichnet wird einfach jedes Glied als Linie, so breit und in der Farbe, die es sich beim Verbinden gemerkt hat:"),

  H.code(stueck("kap09/2_bruecke.dh", "SUB zeichnen", "END SUB")),

  H.p("Das ganze Programm:"),

  H.code(zeilen("kap09/2_bruecke.dh")),

  H.h2("Warum so?"),

  H.h3("Abstände festhalten statt Federn rechnen"),

  H.p("Der naheliegende Weg wäre ein anderer: Jedes Glied ist eine Feder, die zieht, wenn sie zu lang ist. Jeder Punkt hat eine Geschwindigkeit, die Federkräfte und die Schwerkraft ändern, wie bei den Funken in Kapitel 5. Das Programm code/kap09/3_federn.dh tut genau das, für dasselbe Seil:"),

  H.code(stueck("kap09/3_federn.dh", "' Die Kraft der Feder", "END FUNCTION")),
  H.code(stueck("kap09/3_federn.dh", "' Punkt 0 ist fest.", "NEXT")),

  H.p("Die Frage ist, wie hart die Federn sein sollen. Mit den Tasten 1, 2 und 3 kann man es ausprobieren, und keine Antwort ist gut:"),

  H.figure("kap09_federn_weich.png", "Weiche Federn (400): Das Seil, das 285 lang sein soll, ist nach knapp zwei Sekunden über 1000 lang und wippt wie ein Gummiband.", 420, 262),

  H.figure("kap09_federn_hart.png", "Harte Federn (4000): Nach weniger als einer Sekunde fliegen die Punkte durcheinander, das Seil ist ein Knäuel, und es wird mit jedem Schritt wilder.", 420, 262),

  H.p("Weiche Federn geben nach; das Seil wird zum Gummiband. Bei 2000 ist es immer noch eineinhalbmal zu lang und fängt an zu zacken. Und harte Federn explodieren: Eine harte Feder zieht in einem Schritt so stark, dass der Punkt über den richtigen Abstand hinausschießt; im nächsten Schritt zieht sie noch stärker zurück, und er schießt noch weiter. Je härter die Feder, desto kleiner müsste der Zeitschritt sein — und ein Seil, das nicht dehnbar ist, bräuchte unendlich harte Federn."),

  H.p("Das Verlet-Seil hat dieses Problem nicht, weil es keine Kraft rechnet. Ein Glied rückt seine Punkte direkt dorthin, wo sie hingehören, und weiter als dorthin schiebt es nie. Gemessen ist es nach zehn Sekunden 286 bis 288 lang, wo es 285 sein soll. Die Geschwindigkeit, die dabei verloren geht oder entsteht, ergibt sich von selbst aus ort - alt — darum braucht Verlet sie nicht zu speichern."),

  H.h3("Ein VEC2 statt zweier Zahlen"),

  H.pmix(["Die Zeile ", ["ort[k] = ort[k] + (ort[k] - alt[k]) * 0.995 + g", true], " stünde ohne VEC2 zweimal da, einmal mit x und einmal mit y, und man müsste beim Lesen prüfen, ob beide wirklich gleich sind. Die Rechnung mit Punkten wird kürzer und sieht aus wie die Formel, die man im Kopf hat. Nur wo ein Befehl zwei Zahlen will, wie LINEW, holt man sie mit .x und .y wieder heraus."]),

  H.h3("Eine Liste von Gliedern"),

  H.p("Das Seil konnte seine Glieder aus den Nummern ablesen: k und k + 1. Die Brücke nicht — ein Punkt der Fahrbahn hängt an zwei Nachbarn und einem Hänger. Mit der Liste weiß ziehen nichts mehr von Brücken; es kennt nur Glieder. Darum lassen sich mit genau diesem ziehen auch ein Vorhang und eine Fahne bauen, wie die Übungen zeigen: Nur der Teil, der das Netz anlegt, ist anders."),

  H.h2("Was schiefgehen kann"),

  H.h3("Einen Teil eines VEC2 setzen"),

  H.code(zeilen("kap09/fehler_vec2.dh")),
  H.code(["Laufzeitfehler in fehler_vec2.dh:4: VEC2 ist unveraenderlich: '.y'",
          "laesst sich lesen, aber nicht setzen -- einen neuen Wert bauen, etwa",
          "v = VEC2_NEW(...)"], { out: true }),

  H.pmix(["Ein VEC2 ist ein Wert wie eine Zahl: Man kann ihn durch einen anderen ersetzen, aber nicht ändern. Das ist Absicht — wer ", ["q = p", true], " schreibt, soll nicht über q den Punkt p verschieben können. Richtig ist ", ["p = p + VEC2_NEW(0, 5)", true], " oder ", ["p = VEC2_NEW(p.x, p.y + 5)", true], ". (Bis zur Fassung 2026.52 ließ sich p.x nicht einmal lesen, und die Meldung sagte nur „bei nicht-Objekt“. Gefunden beim Schreiben dieses Kapitels.)"]),

  H.h3("Das IMPORT vergessen"),

  H.code(zeilen("kap09/fehler_import.dh")),
  H.code(["fehler_import.dh:1: Compile-Fehler: Unbekannter Typ 'vec2' -- fehlt",
          "IMPORT \"vec2\"?"], { out: true }),

  H.p("VEC2 gehört zum Modul vec2, und Module muss man ankündigen. Die Meldung kommt vor dem Start und sagt gleich, welches IMPORT fehlt."),

  H.h3("Zu wenige Runden"),

  H.p("Mit RUNDEN = 1 läuft alles, nur fühlt es sich falsch an: Das Seil ist mal 300, mal fast 390 lang und wippt. Es gibt keine Meldung, weil nichts kaputt ist; die Glieder haben nur nicht genug Gelegenheit, sich zu einigen. Drehen 1 probiert es aus."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Setze im Seil RUNDEN auf 1 und dann auf 100. Wie lang ist das Seil jeweils, und wie fühlt es sich an?"),
  H.bulletRich("Drehen 2. ", "Ersetze in fliegen 0.995 durch 1.0 und dann durch 0.95. Wie lange schwingt das Seil?"),
  H.bulletRich("Drehen 3. ", "Lass in der Brücke die Hänger weg (die Zeile mit verbinden(seil[i], bahn[i], …)). Was macht die Fahrbahn?"),
  H.bulletRich("Erweitern 1. ", "Ein Vorhang: ein Netz aus 30 mal 20 Punkten, die oberste Reihe an einer Stange."),
  H.bulletRich("Erweitern 2. ", "Ein Seil, das reißt: Ein Glied, das zu weit gedehnt wird, geht kaputt und wird nicht mehr gezeichnet."),
  H.bulletRich("Erfinden. ", "Wind: eine Fahne am Mast, die der Wind waagrecht hält und durch die Wellen laufen. Oder eine Kette, die von einer Rolle abrollt; ein Spinnennetz; ein Trampolin, auf das ein Ball fällt. Anhang A zeigt die Fahne."),

  H.figure("kap09_wind.png", "Eine Möglichkeit für die Erfinden-Übung: eine Fahne am Mast, durch die vom Mast weg eine Welle läuft.", 420, 262),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Modul vec2: ", "alle Befehle für Vektoren, etwa VEC2_NORMALIZE, VEC2_DOT und VEC2_REFLECT für das Abprallen."),
  H.bulletRich("Handbuch, Modul m3d: ", "dasselbe in drei Dimensionen, VEC3, und Drehungen als QUAT."),
];
