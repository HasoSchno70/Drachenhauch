const { zeilen, ohneKopf, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.part("Teil I · Zeichnen mit Zahlen"),

  H.p("Die vier Kapitel dieses Teils malen mit nichts als Zahlen: Ein Stern ist eine Stelle im Raum, eine Farbe ist ein Winkel, ein Muster ist eine Formel, eine Uhr ist eine Drehung. Dabei kommen die Werkzeuge zusammen, die jedes größere Programm braucht — Felder für viele gleichartige Werte, Schleifen, die sie durchgehen, Funktionen, die eine Rechnung benennen, und Konstanten für alles, was feststeht."),

  H.chapter("Sternenflug", 1),

  H.h2("Was wir bauen"),

  H.figure("kap01_3_sternenflug.png", "Der Sternenflug: vierhundert Sterne kommen aus der Tiefe auf uns zu. Je näher einer ist, desto länger, dicker und heller ist sein Strich.", 480, 300),

  H.p("Sterne fliegen aus der Mitte des Bildes auf uns zu, erst als winzige Punkte, dann immer schneller nach außen, bis sie als helle Striche am Rand verschwinden. Es ist einer der ältesten Effekte der Computergrafik, und er braucht kaum mehr als eine Division."),

  H.h2("Der Plan"),

  H.p("Ein Stern ist eine Stelle im Raum mit drei Zahlen: x (links–rechts), y (oben–unten) und z, seine Entfernung von uns. Fliegen wir auf die Sterne zu, ändert sich nur z: Es wird kleiner. Auf dem Bildschirm aber wandert der Stern nach außen und wird größer — und genau das muss die Rechnung herausbekommen. Im Einzelnen:"),

  H.bulletRich("Am Anfang ", "bekommt jeder Stern eine zufällige Stelle in einem großen Quader vor uns."),
  H.bulletRich("In jedem Bild ", "wird z jedes Sterns ein Stück kleiner. Ist ein Stern an uns vorbei, fängt er hinten wieder an."),
  H.bulletRich("Gezeichnet ", "wird er dort, wo der Strahl von unserem Auge zu ihm den Bildschirm trifft."),

  H.h2("Schritt 1: zweihundert Punkte"),

  H.p("Zweihundert Sterne heißen sechshundert Zahlen. Sie liegen in drei Feldern fester Größe, eines je Koordinate:"),

  H.code(stueck("kap01/1_sterne.dh", "CONST ANZAHL", "DIM sz[ANZAHL] AS FLOAT")),

  H.pmix([["DIM sx[ANZAHL] AS FLOAT", true], " legt zweihundert Fächer an, ", ["sx[0]", true], " bis ", ["sx[199]", true], ", alle mit 0.0 gefüllt. Stern Nummer 17 ist ", ["sx[17]", true], ", ", ["sy[17]", true], " und ", ["sz[17]", true], " — dieselbe Nummer in allen drei Feldern. Die Zahlen oben sind Konstanten: Sie stehen einmal am Anfang, haben einen Namen und ändern sich nie. Mehr dazu in Kapitel 4; hier genügt, dass ", ["ANZAHL", true], " an jeder Stelle lesbarer ist als eine nackte 200."]),

  H.code(stueck("kap01/1_sterne.dh", "FOR i = 0 TO ANZAHL - 1", "NEXT")),

  H.pmix([["RANDINT(a, b)", true], " würfelt eine ganze Zahl von a bis b, beide eingeschlossen. Die Sterne liegen danach in einem Quader, 800 breit, 500 hoch und 1000 tief."]),

  H.p("In der Schleife fliegt jeder Stern ein Stück näher und wird dann gezeichnet:"),

  H.code(stueck("kap01/1_sterne.dh", "sz[i] = sz[i] - TEMPO * dt", "CIRCLE(px, py, 1")),

  H.p("Die erste Zeile ist die Bewegung: 250 Einheiten je Sekunde auf uns zu. Die zweite fängt die Sterne auf, die an uns vorbei sind, und schickt sie nach hinten. Die beiden Zeilen danach sind der ganze Trick des Kapitels — die Perspektive."),

  H.h2("Die Perspektive ist eine Division"),

  H.figure("kap01_perspektive.png", "Von der Seite gesehen: Zwei Sterne in derselben Höhe, aber verschieden weit weg. Der Strahl zum ferneren trifft den Bildschirm näher an der Mitte.", 470, 295),

  H.p("Stell dir vor, der Bildschirm ist eine Glasscheibe ein Stück vor deinem Auge, im Abstand f. Was du durch die Scheibe siehst, malst du genau dort auf das Glas, wo der Strahl zum Gegenstand die Scheibe durchstößt. Das Bild zeigt es: Zwei Sterne liegen gleich hoch, 150 über der Blickachse, aber der eine ist 300 entfernt, der andere 680. Der Strahl zum nahen trifft das Glas bei 80, der zum fernen bei 35."),

  H.pmix(["Die Rechnung dahinter ist der Strahlensatz aus der Schule: Die kleine Höhe auf dem Glas verhält sich zum Abstand des Glases wie die große Höhe zum Abstand des Sterns. Also y' / f = y / z, und umgestellt ", ["y' = y * f / z", true], ". Für x gilt dasselbe. Dazu kommt noch die Mitte des Bildschirms, (400, 250), denn dort liegt die Blickachse:"]),

  H.code(stueck("kap01/1_sterne.dh", "DIM px AS FLOAT", "DIM py AS FLOAT")),

  H.p("Mehr ist es nicht. Weil z im Nenner steht, wird ein ferner Stern klein gerechnet und ein naher groß; und weil z beim Fliegen kleiner wird, wächst der Bruch immer schneller — die Sterne beschleunigen nach außen, obwohl sie sich mit gleichem Tempo nähern. Genau so sieht es aus dem Fenster eines Zugs aus: Die Bäume am Gleis rasen vorbei, die Berge am Horizont stehen fast still."),

  H.figure("kap01_1_sterne.png", "Der erste Stand: zweihundert weiße Punkte. Schon jetzt sieht man den Flug — wenn auch noch flach.", 420, 262),

  H.h2("Schritt 2: näher heißt größer und heller"),

  H.p("Noch sind alle Sterne gleich groß. Ein Maß dafür, wie nah ein Stern ist, liefert z selbst: Ganz hinten ist z gleich TIEFE, ganz vorn fast 0. Daraus wird eine Zahl von 0 (hinten) bis 1 (vorn):"),

  H.code(stueck("kap01/2_naeher.dh", "DIM naehe AS FLOAT", "CIRCLE(px, py, 0.5 + naehe * 3")),

  H.p("Mit dieser einen Zahl lässt sich alles steuern, was mit der Entfernung zu tun hat: Die Helligkeit läuft von 60 (fast schwarz) bis 255 (weiß), der Radius von einem halben bis zu dreieinhalb Punkten. Solche Zahlen von 0 bis 1 begegnen uns im ganzen Buch; sie lassen sich in jede Größe umrechnen, die man gerade braucht."),

  H.p("Außerdem bekommt ein Stern, der hinten neu anfängt, eine neue Stelle. Sonst käme alle vier Sekunden genau dasselbe Muster auf uns zu. Das erledigt eine SUB, die auch den Anfang füllt:"),

  H.code(stueck("kap01/2_naeher.dh", "SUB neuerStern", "END SUB")),

  H.figure("kap01_2_naeher.png", "Der zweite Stand: nahe Sterne sind größer und heller, die fernen bilden einen schwachen Nebel in der Mitte.", 420, 262),

  H.h2("Schritt 3: Striche"),

  H.p("Den Eindruck von Geschwindigkeit machen Striche: Jeder Stern zieht eine Linie von der Stelle, an der er vor kurzem war, zu der, an der er jetzt ist. Wo er vor kurzem war, ist leicht auszurechnen — dieselbe Division mit einem etwas größeren z:"),

  H.code(stueck("kap01/3_sternenflug.dh", "' Der Schweif reicht", "LINEW(hx, hy, px, py")),

  H.p("TEMPO * 0.15 ist die Strecke, die ein Stern in 0,15 Sekunden fliegt. Ferne Sterne haben fast keinen Strich, weil sich ihr Bild in dieser Zeit kaum bewegt; nahe ziehen lange Striche, weil sich bei kleinem z schon ein kleines Stück stark auswirkt. Die Striche zeigen also von selbst, was die Perspektive tut. Das ganze Programm:"),

  H.code(zeilen("kap01/3_sternenflug.dh")),

  H.h2("Warum so?"),

  H.h3("Drei Felder, noch keine Klasse"),

  H.p("Ein Stern ist hier nur drei Zahlen. Er tut nichts selbst, er wird nicht gelöscht und nicht angehängt, und alle Sterne sind gleich. Für so etwas sind drei Felder mit derselben Nummer die einfachste Form, und die schnellste dazu. In Kapitel 5 wird es anders: Dort hat ein Funke sechs Eigenschaften, Funken entstehen und verschwinden, und dann lohnt es sich, einen Funken als eine Sache zu beschreiben. Man nimmt das schwerere Werkzeug, wenn man es braucht, nicht vorher."),

  H.h3("Feste Größe, und z + TIEFE statt TIEFE"),

  H.pmix(["Es gibt immer genau ANZAHL Sterne; ein Stern, der vorbei ist, wird nicht gelöscht, sondern hinten wiederverwendet. Darum genügt ein Feld fester Größe. Beim Zurückschicken setzt das Programm z nicht einfach auf TIEFE, sondern auf ", ["sz[i] + TIEFE", true], ". Der Unterschied ist klein: Ein Stern, der in diesem Bild auf -3 gerutscht ist, fängt bei 997 an statt bei 1000. Aber so bleibt der Abstand zwischen den Sternen erhalten, und sie kommen nie in Wellen."]),

  H.h3("/ und \\"),

  H.p("In Drachenhauch liefert / immer eine Kommazahl, auch wenn beide Seiten ganze Zahlen sind. Für die Perspektive ist das genau richtig: Ein Stern bei px = 401,6 soll nicht auf 401 springen, das sähe man als Zittern. Wer ganzzahlig teilen will, nimmt den Rückstrich:"),

  H.code(ohneKopf("kap01/4_ganzzahlig.dh")),
  H.code(["3.5", "3", "-4", "4.0"], { out: true }),

  H.pmix([["\\", true], " rundet immer nach unten, auch bei negativen Zahlen: -7 \\ 2 ist -4, nicht -3. Das klingt pedantisch, ist aber genau, was man für Kacheln, Gitter und Raster braucht — die Kachel links neben 0 hat die Nummer -1, nicht 0. Und 8 / 2 ist 4.0, nicht 4: Das Ergebnis von / ist immer eine Kommazahl, auch wenn die Division aufgeht."]),

  H.h2("Was schiefgehen kann"),

  H.h3("Division durch null"),

  H.p("Die Zeile IF sz[i] < 1 hält z vom Nullpunkt fern. Was passiert, wenn z doch einmal genau 0 wird, zeigt ein kleines Programm, das in Fünferschritten herunterzählt:"),

  H.code(zeilen("kap01/fehler_null.dh")),
  H.code(["15.0", "20.0", "30.0", "60.0"], { out: true }),
  H.code(["Laufzeitfehler in fehler_null.dh:3: Division durch 0"], { out: true }),

  H.p("Die Zahlen wachsen, je kleiner z wird — das ist die Perspektive —, und bei z = 0 hält das Programm an. Mit Kommazahlen wie im Sternenflug trifft man die 0 selten genau; dafür wird z negativ, und die Division setzt den Stern gespiegelt auf die andere Seite des Bildes. Im Sternenflug kommt es nicht einmal dazu — was stattdessen passiert, zeigt die Übung Drehen 3."),

  H.h3("Ein Index aus /"),

  H.p("Weil / immer eine Kommazahl liefert, taugt sein Ergebnis nicht als Nummer eines Fachs:"),

  H.code(zeilen("kap01/fehler_index.dh")),
  H.code(["fehler_index.dh:3: Warnung: Der Index ist eine Kommazahl -- das",
          "bricht beim Laufen ab ('Array-Index muss INTEGER sein'). `/`",
          "liefert immer FLOAT; ganzzahlig teilt `\\`."], { out: true }),
  H.code(["Laufzeitfehler in fehler_index.dh:3: Array-Index muss INTEGER sein,",
          "erhalten FLOAT"], { out: true }),

  H.p("Die erste Meldung kommt vor dem Start, die zweite beim Laufen. Selbst wenn die Division aufgeht — i / 2 mit i = 8 —, bleibt das Ergebnis 4.0 und damit eine Kommazahl. Die Abhilfe steht in der Warnung: i \\ 2."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Setze BRENNWEITE auf 100, dann auf 900. Was ändert sich, und woran erinnert es dich bei einem Fotoapparat?"),
  H.bulletRich("Drehen 2. ", "Setze ANZAHL auf 4000. Sieht es besser aus? Läuft es noch flüssig?"),
  H.bulletRich("Drehen 3. ", "Nimm die Zeile IF sz[i] < 1 … heraus und sage vorher, was passiert. Lag deine Vorhersage richtig?"),
  H.bulletRich("Erweitern 1. ", "Ferne Sterne sollen bläulich sein, nahe weiß. Hinweis: HSV mit einer Sättigung, die von naehe abhängt."),
  H.bulletRich("Erweitern 2. ", "Eine Warp-Taste: Solange die Leertaste gedrückt ist, fliegen wir sechsmal so schnell — aber mit Anlauf und Auslauf, nicht schlagartig."),
  H.bulletRich("Erfinden. ", "Mach etwas Eigenes aus dem Sternenfeld: einen Flug durch einen Wirbel, Sterne, die bei einem Mausklick explodieren, einen Flug nach rechts statt nach vorn, Schneeflocken statt Sterne. Anhang A zeigt den Wirbel."),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Sprache → Arrays: ", "Felder fester Größe, eigene Untergrenzen (DIM f[1 TO 31]) und Felder, die wachsen."),
  H.bulletRich("Handbuch, Stolpersteine: ", "/ und \\, MID$ ab 0 und die anderen Stellen, an denen Drachenhauch anders zählt als andere BASICs."),
];
