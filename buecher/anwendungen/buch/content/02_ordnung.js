const { zeilen, ohneKopf, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Ordnung im Fenster", 2),

  H.h2("Was wir bauen"),

  H.figure("kap02_3_mindestmass.png", "Das Formular der Mediathek: vier Felder, bündig untereinander, der Knopf unten rechts. Keine Zahl darin ist ausgerechnet, und es wächst mit dem Fenster.", 460, 250),

  H.p("Ein Eintrag der Mediathek bekommt mehr als einen Titel: wer ihn gemacht hat, was für eine Art Medium er ist und wann er erschienen ist. Das Formular dafür hat vier Zeilen aus Beschriftung und Feld, darunter eine Statuszeile und den Knopf. Zieht man das Fenster größer, werden die langen Felder breiter und der Knopf wandert mit in die Ecke; kleiner als sein Inhalt lässt es sich nicht ziehen."),

  H.h2("Der Plan"),

  H.p("In Kapitel 1 stand jedes Ding an einer Stelle, die wir ausgerechnet hatten: das Feld 80 Punkte vom linken Rand, der Knopf bei 370. Bei drei Dingen geht das. Bei einem Formular mit zwanzig Feldern ist es Fleißarbeit, und sie hält nicht: Ein längeres Wort, eine größere Schrift oder ein breiteres Fenster, und die ausgerechneten Zahlen stimmen nicht mehr."),

  H.p("Der Plan ist darum, keine Zahlen mehr auszurechnen, sondern zu sagen, wie die Dinge zueinander stehen:"),

  H.bulletRich("Untereinander: ", "die vier Zeilen und die Knopfreihe stehen in einer Spalte."),
  H.bulletRich("Nebeneinander: ", "in jeder Zeile die Beschriftung und daneben das Feld."),
  H.bulletRich("Gleich breit: ", "alle Beschriftungen so breit wie die längste, damit die Felder bündig stehen."),
  H.bulletRich("Wer den Rest bekommt: ", "die Felder für Titel und Urheber wachsen mit, das Jahr bleibt schmal."),

  H.p("Das Ausrechnen übernehmen Behälter: unsichtbare Kästen, die ihren Inhalt in jedem Bild neu verteilen."),

  H.h2("Schritt 1: jedes Feld an seinem Platz"),

  H.p("Zuerst die Mediathek mit vier Feldern, gebaut wie in Kapitel 1 — jedes an seinem festen Platz:"),

  H.code(stueck("kap02/1_fest.dh", "DIM l1 AS GUI_WIDGET", "DIM knopf AS GUI_WIDGET")),

  H.figure("kap02_1_fest.png", "Feste Plätze: „Erscheinungsjahr“ ist länger als die 80 Punkte, die für Beschriftungen vorgesehen waren, und das Feld liegt darüber.", 460, 250),

  H.p("Die Felder fangen alle bei 100 an, und für „Titel“, „Urheber“ und „Art“ reicht das. „Erscheinungsjahr“ ist länger, und das Feld deckt die zweite Hälfte des Worts zu. Natürlich kann man alle Felder nach rechts schieben, bis es passt — vier Zahlen ändern. Aber auf Englisch hieße die Zeile anders, mit einer anderen Schrift wäre das Wort anders lang, und wer am Fenster zieht, sieht, dass auch die Größe festgeschrieben ist: Das Programmfenster wird größer, das Formular bleibt in der Ecke oben links, wie es war."),

  H.h2("Schritt 2: Behälter"),

  H.p("Ein Behälter ist ein Baustein ohne eigenes Aussehen. Er hat eine Lage und eine Größe, und was man in ihn hineinlegt, ordnet er darin an — als Spalte untereinander oder als Zeile nebeneinander. Der äußerste Behälter ist so groß wie das Fenster:"),

  H.code(stueck("kap02/2_behaelter.dh", "DIM form AS GUI_WIDGET", "GUI_LAYOUT_SET(form, \"abstand\"")),

  H.pmix([["GUI_LAYOUT", true], " legt ihn an: eine Spalte, an der Stelle 0, 0 und so groß wie das Fenster. ", ["GUI_SET_ANCHOR(form, \"lrtb\")", true], " verankert ihn an allen vier Kanten des Fensters — links, rechts, oben, unten —, und damit wird er größer, wenn das Fenster größer wird. Der ", ["rand", true], " ist der Platz zwischen seiner Kante und dem Inhalt, der ", ["abstand", true], " der zwischen zwei Dingen darin."]),

  H.p("Jede Zeile des Formulars ist ein kleiner Behälter für sich, eine Zeile mit Beschriftung und Feld. Weil es vier davon gibt, baut sie eine SUB:"),

  H.code(stueck("kap02/2_behaelter.dh", "SUB zeile", "END SUB")),

  H.pmix(["In der Zeile steht die Beschriftung mit ihrer eigenen Breite und daneben das Feld. Die Zahl hinter dem Feld in ", ["GUI_LAYOUT_ADD(z, feld, gewicht)", true], " ist sein ", ["Gewicht", true], ": 0 heißt, das Feld behält seine eigene Breite; 1 heißt, es bekommt den Platz, der übrig bleibt. Ein schmales Feld (gewicht 0) bekommt dahinter einen Leerraum, ", ["GUI_LAYOUT_SPACER", true], ", der den Rest schluckt. ", ["ausrichtung 1", true], " setzt Beschriftung und Feld auf eine gemeinsame Mitte, ", ["dehnen 0", true], " lässt jedem seine Höhe, statt alle auf die Höhe der Zeile zu ziehen."]),

  H.pmix(["Bleibt die Frage, wie die Felder bündig werden, wenn jede Beschriftung so breit ist wie ihr Wort. Das löst ", ["GUI_SIZE_GROUP(l, \"beschriftung\", \"breite\")", true], ": Alle Bausteine einer Größengruppe bekommen dasselbe Maß, das größte unter ihnen. Wird „Erscheinungsjahr“ zu „Year of publication“, wachsen alle vier Beschriftungen mit, und die Felder rücken gemeinsam nach rechts."]),

  H.p("Die Felder selbst entstehen mit der Größe 0 — das heißt: nach Inhalt. Breite und Lage gibt ihnen ihre Zeile; nur die beiden schmalen Felder bekommen eine feste Breite mit, denn sie haben Gewicht 0 und behalten sie:"),

  H.code(stueck("kap02/2_behaelter.dh", "DIM titel AS GUI_WIDGET", "zeile(\"Erscheinungsjahr\"")),

  H.p("Unten die Knopfreihe, nach demselben Muster: Statuszeile, Leerraum, Knopf. Und ein Leerraum in der äußeren Spalte vor der Knopfreihe schiebt sie an den unteren Rand:"),

  H.code(stueck("kap02/2_behaelter.dh", "DIM knoepfe AS GUI_WIDGET", "GUI_LAYOUT_ADD(form, knoepfe)")),

  H.p("Zum Schluss muss das gui-Fenster erfahren, dass das Programmfenster größer geworden ist. Das sagt WINDOW_RESIZED, wahr in dem Bild nach einer Größenänderung:"),

  H.code(stueck("kap02/2_behaelter.dh", "IF WINDOW_RESIZED()", "END IF")),

  H.p("Damit läuft eine Kette: Das Programmfenster wächst, also setzt das Programm das gui-Fenster auf die neue Größe; die Spalte klebt an dessen Kanten und wächst mit; sie verteilt ihren neuen Platz auf ihre Zeilen, und in jeder Zeile bekommt das Feld mit Gewicht 1 den Zuwachs."),

  H.figure("kap02_2_behaelter.png", "Dasselbe Formular aus Behältern. Die Beschriftungen sind gleich breit — so breit wie „Erscheinungsjahr“ —, und die Felder stehen bündig daneben.", 460, 250),

  H.h2("Schritt 3: nicht kleiner als nötig"),

  H.p("Größer ziehen ist jetzt kein Problem mehr, kleiner schon: Wird das Fenster zu klein, schrumpfen die Felder, bis nichts mehr geht; dann läuft die Auswahl der Art rechts über den Rand, und die Knopfreihe rutscht unten aus dem Fenster. Die Spalte weiß aber, wie viel Platz ihr Inhalt mindestens braucht — Beschriftungen in voller Breite, Felder mit Gewicht so schmal wie möglich, Abstände und Rand dazu:"),

  H.code(stueck("kap02/3_mindestmass.dh", "WINDOW_MIN_SIZE")),

  H.pmix([["WINDOW_MIN_SIZE", true], " ist die kleinste Größe, auf die man das Programmfenster ziehen kann, und ", ["GUI_LAYOUT_MIN_W", true], " und ", ["GUI_LAYOUT_MIN_H", true], " liefern genau die Zahlen dafür. Das ganze Programm:"]),

  H.code(zeilen("kap02/3_mindestmass.dh")),

  H.h2("Warum so?"),

  H.h3("Beschreiben statt ausrechnen"),

  H.p("Mit festen Zahlen steckt das Wissen, wie ein Formular aufgebaut ist, nur im Kopf dessen, der es ausgerechnet hat: Dass das Feld bei 100 anfängt, weil die Beschriftungen davor 80 Punkte brauchen, steht nirgends. Ändert sich eine Beschriftung, muss man es wieder ausrechnen. Ein Behälter schreibt dieses Wissen hin — „Beschriftung und Feld nebeneinander, Beschriftungen gleich breit“ —, und weil es dasteht, kann die gui es bei jeder Änderung neu anwenden."),

  H.h3("Warum keine Prozente"),

  H.p("Man könnte jedem Feld sagen: „du bekommst 70 Prozent der Breite“. Das wächst zwar mit, aber falsch: Bei einem breiten Fenster werden auch die Beschriftungen breiter, obwohl ihr Text derselbe bleibt, und bei einem schmalen wird „Erscheinungsjahr“ abgeschnitten. Die Gewichte verteilen nur, was übrig ist, nachdem jeder Baustein ohne Gewicht seine eigene Größe bekommen hat. So bleibt eine Beschriftung so breit wie ihr Wort, und nur die Felder, die von mehr Platz etwas haben, bekommen ihn."),

  H.h3("Längere Wörter, größere Schrift, anderer Maßstab"),

  H.pmix(["Feste Zahlen zerbrechen an allem, was die Länge eines Texts ändert: an einer Übersetzung (Kapitel 22 macht die Mediathek zweisprachig), an einer anderen Schrift, an einer größeren Schrift für jemanden, der schlecht sieht. Der Maßstab eines hochauflösenden Bildschirms ist dagegen kein Grund — für ihn gibt es ", ["GUI_SCALE", true], ", das vor dem ersten Fenster gesetzt wird und jede Zahl, die in die gui hineingeht, mit demselben Faktor malnimmt, Schrift eingeschlossen. Ein Formular aus festen Zahlen wird damit gleichmäßig größer; was es nicht übersteht, ist ein Wort, das länger wird als sein Platz."]),

  H.h3("Ein Behälter hat keine Farbe"),

  H.p("Ein Behälter zeichnet nichts und nimmt keine Klicks an — ein Klick auf ein Feld darin landet beim Feld. Das ist wichtig, weil fast alles in einem Fenster in einem Behälter steckt: Würde er Klicks schlucken, wäre das Formular taub. Zum Entwickeln kann man ihn sichtbar machen, mit GUI_LAYOUT_SET(form, \"rahmen\", 1) als gestrichelten Rahmen."),

  H.h2("Was schiefgehen kann"),

  H.h3("Das Formular wächst nicht mit"),

  H.p("Fehlt die Abfrage von WINDOW_RESIZED, wird das Programmfenster größer, das gui-Fenster darin aber nicht, und die Spalte, die an seinen Kanten klebt, auch nicht. Rechts und unten bleibt eine leere Fläche. Fehlt der Anker der Spalte, wächst das gui-Fenster mit, die Spalte aber nicht — dasselbe Bild. Wer ein Formular sieht, das nicht mitwächst, prüft diese beiden Stellen."),

  H.h3("Ein Feld in zwei Behältern"),

  H.p("Ein Baustein steckt in höchstens einem Behälter. Wer dasselbe Feld in zwei Zeilen legt, weil es zu beiden passen soll, bekommt keine Meldung — das zweite GUI_LAYOUT_ADD nimmt es aus der ersten Zeile heraus und hängt es in die zweite um:"),

  H.code(stueck("kap02/fehler_zweimal.dh", "' Dasselbe Feld", "GUI_LAYOUT_ADD(unten, feld, 1)")),

  H.figure("kap02_fehler_zweimal.png", "Das Feld ist umgezogen: Die Zeile „Titel“ ist leer, „Originaltitel“ hat es bekommen.", 460, 115),

  H.p("Umhängen ist erlaubt, weil man damit ein Formular umbauen kann, während es läuft — ein Feld wandert in eine andere Spalte, wenn das Fenster schmal wird. Zwei Zeilen, die dasselbe zeigen sollen, brauchen zwei Felder."),

  H.h3("GUI_SCALE zu spät"),

  H.code(zeilen("kap02/fehler_massstab.dh")),
  H.code(["Laufzeitfehler in fehler_massstab.dh:5: GUI_SCALE: erst den",
          "Massstab setzen, dann Fenster und Widgets anlegen (bereits",
          "angelegte werden nicht umgerechnet)"], { out: true }),

  H.p("Der Maßstab wird beim Anlegen in jede Zahl eingerechnet. Was schon da ist, nachträglich umzurechnen, ginge nur ungefähr — und eine halb umgerechnete Oberfläche wäre schlimmer als eine klare Absage. GUI_SCALE gehört darum an den Anfang, vor das erste GUI_WINDOW."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Nimm GUI_SIZE_GROUP aus der SUB zeile heraus. Wie sehen die Felder jetzt aus?"),
  H.bulletRich("Drehen 2. ", "Gib dem Titelfeld das Gewicht 0 statt 1. Wie breit ist es, und was passiert, wenn das Fenster wächst?"),
  H.bulletRich("Drehen 3. ", "Nimm GUI_LAYOUT_SPACER(form) heraus. Wo steht die Knopfreihe jetzt? Sage es vorher."),
  H.bulletRich("Erweitern 1. ", "Dasselbe Formular in zwei Spalten: Titel und Urheber links, Art und Jahr rechts. Jede Spalte richtet ihre Felder für sich aus."),
  H.bulletRich("Erweitern 2. ", "Das Fenster merkt sich seine Größe: Beim Beenden schreibt es Breite und Höhe in eine Datei mediathek.ini, beim nächsten Start öffnet es sich gleich so groß."),
  H.bulletRich("Erfinden. ", "Baue ein Formular, das du kennst — die Anmeldung eines Vereins, eine Bestellung beim Bäcker, ein Steckbrief — nur aus Behältern, ohne eine einzige ausgerechnete Zahl außer der Größe des Fensters."),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Modul gui → Layout: Größe nach Inhalt und Behälter: ", "Zeile, Spalte, Raster und Fluss, Gewichte, Mindestmaße — alle Regeln dieses Kapitels mit ihren Einzelheiten."),
  H.bulletRich("Handbuch, Modul gui → Maßstab: ", "GUI_SCALE, und warum nach außen alle Zahlen logisch bleiben."),
  H.bulletRich("Beispiel examples/194_gui_layout.dh: ", "ein Formular für eine Nachricht, mit einem Schalter, der alle Behälter sichtbar macht."),
];
