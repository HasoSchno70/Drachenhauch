const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Schwarm", 8),

  H.h2("Was wir bauen"),

  H.figure("kap08_schwarm.png", "Nach zehn Sekunden: Aus 150 Vögeln, die in alle Richtungen losgeflogen sind, sind mehrere Schwärme geworden. Jeder fliegt geschlossen in eine eigene Richtung.", 470, 294),

  H.p("Hundertfünfzig Vögel, jeder an einer zufälligen Stelle, jeder in eine zufällige Richtung. Nach wenigen Sekunden fliegen sie in Schwärmen, wenden gemeinsam, teilen sich und finden wieder zusammen. Es gibt keinen Anführer und keinen Plan für den Schwarm. Jeder Vogel sieht nur seine Nachbarn und folgt drei einfachen Regeln. Der Informatiker Craig Reynolds hat sie 1986 aufgeschrieben und seine Vögel „Boids“ genannt; seitdem fliegen sie in Filmen und Spielen."),

  H.h2("Der Plan"),

  H.p("Jeder Vogel schaut sich um. Was er in der Nähe sieht, sind seine Nachbarn, und nach ihnen lenkt er:"),

  H.bulletRich("Abstand: ", "Weg von denen, die zu nah sind — sonst stoßen sie zusammen."),
  H.bulletRich("Ausrichtung: ", "Fliegen wie die Nachbarn, in dieselbe Richtung, ungefähr gleich schnell."),
  H.bulletRich("Zusammenhalt: ", "Zur Mitte der Nachbarn — wer am Rand fliegt, rückt herein."),

  H.p("Bisher hat ein Programm Dinge bewegt: Sterne, Funken, Sandkörner. Hier schaut jedes Ding selbst, entscheidet selbst und bewegt sich selbst. Dafür gibt es in Drachenhauch die Klasse."),

  H.h2("Schritt 1: ein Vogel als Klasse"),

  H.p("Eine Klasse beschreibt eine Art von Ding: was es weiß und was es tut. Ein Vogel weiß, wo er ist und wohin er fliegt. Das sind seine Felder, wie bei einem STRUCT in Kapitel 5:"),

  H.code([...stueck("kap08/1_vogel.dh", "CLASS Vogel", "END SUB"), "", "    ' ... bewegen und zeichnen folgen gleich", "END CLASS"]),

  H.pmix(["Neu ist das ", ["SUB Init", true], " innerhalb der Klasse. Ein SUB in einer Klasse heißt Methode, und Init ist eine besondere: Sie läuft, wenn ein neuer Vogel entsteht. Sie schreibt einfach ", ["x = …", true], " — gemeint ist das Feld x des Vogels, der gerade entsteht. Eine Methode sieht die Felder ihres Objekts, ohne dass man sagen muss, wessen."]),

  H.p("Was ein Vogel tut, sind zwei weitere Methoden. Er fliegt ein Stück weiter, und wer hinausfliegt, kommt auf der anderen Seite wieder herein:"),

  H.code(stueck("kap08/1_vogel.dh", "SUB bewegen", "END SUB")),

  H.p("Und er zeichnet sich als schmales Dreieck, dessen Spitze in die Flugrichtung zeigt. ATAN2 macht aus vx und vy diesen Winkel; die beiden hinteren Ecken liegen 2,5 im Bogenmaß — ein gutes Stück mehr als ein rechter Winkel — links und rechts davon:"),

  H.code(stueck("kap08/1_vogel.dh", "' Ein schmales Dreieck", "END SUB")),

  H.p("Hundertfünfzig Vögel entstehen mit NEW. Jedes NEW Vogel() legt einen neuen an und ruft dabei Init:"),

  H.code(stueck("kap08/1_vogel.dh", "DIM schwarm AS ARRAY OF Vogel", "NEXT")),

  H.p("In der Hauptschleife sagt das Programm jedem Vogel, was er tun soll. Der Punkt zwischen Vogel und Methode ist derselbe wie zwischen Vogel und Feld:"),

  H.code(stueck("kap08/1_vogel.dh", "FOR EACH v IN schwarm", "NEXT")),

  H.figure("kap08_1_vogel.png", "Der erste Stand: 150 Vögel, jeder fliegt für sich geradeaus. Noch ist es kein Schwarm, nur ein Gewimmel.", 420, 262),

  H.h2("Schritt 2: umschauen und lenken"),

  H.p("Jetzt bekommt der Vogel Augen. Was er beim Umschauen sieht, schreibt er sich in eigene Felder: wie viele Nachbarn er hat, wo sie zusammen sind, wie sie zusammen fliegen und wohin er ausweichen sollte. Dazu kommt die neue Geschwindigkeit, für die er sich entscheidet:"),

  H.code(stueck("kap08/2_schwarm.dh", "' Was der Vogel beim Umschauen", "DIM neuVy AS FLOAT")),

  H.p("Umschauen heißt, alle anderen Vögel durchzugehen und die zusammenzuzählen, die näher sind als SICHT:"),

  H.code(stueck("kap08/2_schwarm.dh", "' Alle anderen ansehen", "END SUB")),

  H.pmix(["Die Methode bekommt das ganze Feld ", ["alle", true], " und findet darin auch sich selbst. ", ["Self", true], " ist der Vogel, dessen Methode gerade läuft, und ", ["v = Self", true], " fragt, ob v derselbe Vogel ist; ", ["CONTINUE", true], " springt dann gleich zum nächsten. HYPOT rechnet den Abstand aus dx und dy, wie der Satz des Pythagoras. Wer näher als NAEHE ist, schiebt den Vogel weg, und zwar umso stärker, je näher er ist: dx / (d * d) ist eine Richtung von der Länge 1 / d."]),

  H.p("Aus dem, was er gesehen hat, macht jede Regel eine kleine Änderung an der neuen Geschwindigkeit. Jede Regel ist eine eigene Methode:"),

  H.code(stueck("kap08/2_schwarm.dh", "' Regel 1, Abstand", "END SUB")),
  H.code(stueck("kap08/2_schwarm.dh", "' Regel 2, Ausrichtung", "END SUB")),
  H.code(stueck("kap08/2_schwarm.dh", "' Regel 3, Zusammenhalt", "END SUB")),

  H.p("Ausrichtung und Zusammenhalt sind fast gleich gebaut: Sie bilden einen Durchschnitt und lenken ein wenig dorthin. Einmal ist es der Durchschnitt der Geschwindigkeiten, einmal der der Orte. Die Konstanten KRAFT_… sagen, wie stark jede Regel lenkt. Eine vierte Methode hält das Tempo zwischen zwei Grenzen, und entscheiden fasst alles zusammen:"),

  H.code(stueck("kap08/2_schwarm.dh", "' Nie zu langsam", "END SUB")),
  H.code(stueck("kap08/2_schwarm.dh", "' Umschauen und nach", "END SUB")),

  H.p("bewegen übernimmt die neue Geschwindigkeit erst, wenn es so weit ist. Und die Hauptschleife lässt erst alle Vögel entscheiden und dann alle fliegen:"),

  H.code(stueck("kap08/2_schwarm.dh", "' Erst entscheiden ALLE", "NEXT", 2)),

  H.p("Das ganze Programm:"),

  H.code(zeilen("kap08/2_schwarm.dh")),

  H.h2("Warum so?"),

  H.h3("Eine Klasse statt eines STRUCT"),

  H.p("Die Funken in Kapitel 5 waren ein STRUCT: ein Bündel Zahlen, und SUBs außerhalb rechneten damit. Das passte, denn ein Funke tut nichts selbst; er fliegt, wohin die Schwerkraft ihn zieht. Ein Vogel dagegen schaut, wägt ab und entscheidet, und dafür braucht er Notizen, die niemanden sonst angehen — summeX, wegX, neuVx. In einer Klasse stehen die Felder und das, was mit ihnen geschieht, an einer Stelle. Die Hauptschleife muss davon nichts wissen; sie sagt nur v.entscheiden(…) und v.bewegen(…)."),

  H.h3("Jede Regel eine Methode"),

  H.p("entscheiden liest sich wie der Plan: umschauen, Abstand halten, ausrichten, zusammenbleiben, Tempo halten. Wer wissen will, was ein Vogel bei Ausrichtung tut, findet es in einer Methode von sieben Zeilen. Und wer eine Regel dazubauen will — zur Maus fliegen, Felsen ausweichen, vor einem Raubvogel fliehen —, schreibt eine Methode und ruft sie in entscheiden. Alle drei Übungen am Ende des Kapitels tun genau das."),

  H.h3("Erst alle entscheiden, dann alle fliegen"),

  H.p("Das ist die Frage aus Kapitel 6 noch einmal: Sieht ein Vogel beim Umschauen die anderen dort, wo sie waren, oder schon dort, wo sie nach diesem Bild sind? Der Plan meint „wo sie waren“ — alle entscheiden gleichzeitig. Darum schreibt entscheiden nur in neuVx und neuVy, und erst bewegen ändert vx, x und y. neuVx spielt die Rolle, die das zweite Feld im Spiel des Lebens hatte."),

  H.p("Ehrlich gesagt: Probiert man es mit einer einzigen Schleife, die entscheidet und gleich bewegt, sieht man beim Schwarm kaum einen Unterschied. Ein Vogel ändert sich in einem Bild nur wenig, und die Regeln sind weich. Aber das Programm rechnet dann nicht mehr, was der Plan sagt: Ob ein Vogel den anderen schon bewegt sieht, hängt davon ab, wer vorn im Feld steht. Bei harten Regeln wie im Spiel des Lebens wird daraus ein sichtbarer Fehler, bei weichen ein unsichtbarer — und unsichtbare Fehler sind die teureren."),

  H.h3("Warum das Tempo gehalten werden muss"),

  H.p("Ohne tempoHalten werden die Vögel langsamer und langsamer. Nach zehn Sekunden fliegen sie im Schnitt nur noch mit einem Zehntel des Tempos, mit dem sie losgeflogen sind; der Schwarm schwebt fast auf der Stelle. Schuld ist die Ausrichtung: Fliegen zwei Nachbarn in verschiedene Richtungen, ist der Durchschnitt ihrer Geschwindigkeiten kürzer als jede einzelne. Jeder Vogel gleicht sich einem Durchschnitt an, der ein wenig zu langsam ist, Bild für Bild. Echte Vögel halten ihr Tempo von selbst; unsere brauchen dafür eine Methode."),

  H.h2("Was schiefgehen kann"),

  H.h3("Ein Vogel ohne NEW"),

  H.p("Mit den Funken aus Kapitel 5 im Kopf liegt diese Schleife nahe:"),

  H.code(zeilen("kap08/fehler_ohne_new.dh")),
  H.code(["Laufzeitfehler in fehler_ohne_new.dh:15: Methodenaufruf '.bewegen'",
          "bei NIL-Referenz -- die Variable ist noch NIL: eine Klasse muss mit",
          "NEW angelegt werden (p = NEW Klasse())"], { out: true }),

  H.pmix(["Bei einem STRUCT legt ", ["DIM f AS Funke", true], " einen Funken an. Bei einer Klasse legt DIM nur eine Variable an, die auf einen Vogel zeigen KANN — bis NEW einen schafft, zeigt sie auf nichts, und dieses Nichts heißt NIL. Der Fehler fällt nicht bei ARRAY_PUSH auf, sondern erst, als ein Vogel etwas tun soll, in Zeile 15. Richtig ist ", ["ARRAY_PUSH(schwarm, NEW Vogel())", true], ". Dass eine Variable auch auf nichts zeigen kann, ist übrigens nützlich: Der Raubvogel in der Erfinden-Übung beginnt seine Suche mit einem Ziel, das noch NIL ist."]),

  H.h3("Der Vogel, der sich selbst sieht"),

  H.p("Fehlt in umschauen die Zeile mit Self, steht das Programm im ersten Bild:"),

  H.code(stueck("kap08/fehler_selbst.dh", "FOR EACH v IN alle", "NEXT")),
  H.code(["Laufzeitfehler in fehler_selbst.dh:60: Division durch 0"], { out: true }),

  H.p("Jeder Vogel findet sich selbst im Feld alle, im Abstand 0. Das ist weniger als NAEHE, und dx / (d * d) teilt durch 0. Ohne die Teilung wäre der Fehler stiller: Der Vogel zählte sich als eigenen Nachbarn mit und zöge den Durchschnitt zu sich hin. Das Teilen durch 0 ist hier ein Glück — es zeigt die Stelle."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Setze KRAFT_ABSTAND auf 0. Wie viele Vögel siehst du nach zwanzig Sekunden?"),
  H.bulletRich("Drehen 2. ", "Setze KRAFT_RICHTUNG auf 0. Was bleibt vom Schwarm?"),
  H.bulletRich("Drehen 3. ", "Setze SICHT auf 20 und dann auf 150. Was ändert sich?"),
  H.bulletRich("Erweitern 1. ", "Solange die linke Maustaste gedrückt ist, fliegt der Schwarm zur Maus."),
  H.bulletRich("Erweitern 2. ", "Hindernisse: ein paar Felsen, um die der Schwarm herumfliegt."),
  H.bulletRich("Erfinden. ", "Ein Raubvogel, der Jagd auf den Schwarm macht, und Vögel, die vor ihm fliehen. Oder: Vögel in zwei Farben, die nur ihresgleichen folgen; ein Wind, der den Schwarm treibt; ein Fischschwarm, der um Futter kreist. Anhang A zeigt den Raubvogel."),

  H.figure("kap08_raubvogel.png", "Eine Möglichkeit für die Erfinden-Übung: Der Raubvogel (rot) stößt in einen Schwarm, und um ihn herum öffnet sich eine Lücke.", 420, 262),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Sprache → Klassen und Strukturen: ", "Methoden, Self, Vererbung mit EXTENDS und SUPER, ABSTRACT, und warum der Typ einer Klasse gilt."),
];
