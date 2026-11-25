const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Bälle in der Kiste", 10),

  H.h2("Was wir bauen"),

  H.figure("kap10_tuerme.png", "Ein Schuss mit der Schleuder: Der Ball trifft den mittleren Turm, das Brett kippt, die rechte Säule bricht in Stücke.", 470, 294),

  H.p("Drei Türme aus Holzklötzen und eine Schleuder. Man zieht den Ball mit der Maus nach hinten, lässt los, und er fliegt in hohem Bogen — trifft er, kippen Bretter, Säulen knicken, Klötze purzeln übereinander und bleiben so liegen, wie Klötze eben liegen bleiben. Davor steht eine Kiste, in die man Bälle und Klötze wirft. Die Rechnung dafür schreiben wir diesmal nicht selbst: Drachenhauch hat eine fertige Physik eingebaut."),

  H.h2("Der Plan"),

  H.bulletRich("Eine Welt: ", "Die Physik lebt in einer Welt, die Drachenhauch für uns verwaltet. Das Programm hält nur einen Griff darauf."),
  H.bulletRich("Körper: ", "Kästen und Kreise, fest oder beweglich. Die Welt gibt jedem eine Nummer."),
  H.bulletRich("Ein Schritt: ", "Sechzigmal je Sekunde rechnet die Welt einen Schritt weiter — Schwerkraft, Stöße, Reibung, Drehung."),
  H.bulletRich("Zeichnen: ", "Die Welt weiß, wo jeder Körper ist und wie er gedreht ist, aber nicht, wie er aussieht. Das Aussehen merkt sich das Programm."),

  H.h2("Schritt 1: eine Kiste"),

  H.p("Die Physik ist das Modul physics2d. PHYS2D_NEW legt eine Welt an, und PHYS2D_ADD_BOX legt einen Kasten hinein — hier den Boden und zwei Wände:"),

  H.code(stueck("kap10/1_kiste.dh", "DIM welt AS PHYS2D_WORLD", "PHYS2D_ADD_BOX(welt, BREITE - 10")),

  H.pmix(["Die Zahlen sind Mitte x und y, dann die ", ["halbe Breite", false], " und die halbe Höhe, dann ob der Körper sich bewegt (FALSE: fest wie eine Wand), und zuletzt die Sprungkraft von 0 bis 1. Der Boden ist also 800 breit und 20 hoch, seine Mitte 10 über dem unteren Rand. Halbe Maße sind ungewohnt, aber in der Physik üblich; was passiert, wenn man sie vergisst, zeigt „Was schiefgehen kann“."]),

  H.p("Jeder Körper, den die Welt anlegt, bekommt eine Nummer. Was sie nicht kennt, ist sein Aussehen — ob er rund ist, wie groß, welche Farbe. Das merkt sich das Programm in eigenen Feldern, Platz für Platz neben der Nummer:"),

  H.code(stueck("kap10/1_kiste.dh", "' Was die Physik nicht weiss", "DIM farbe AS ARRAY")),
  H.code(stueck("kap10/1_kiste.dh", "SUB ball", "END SUB")),

  H.p("Beim Zeichnen fragt das Programm die Welt nach Lage und Drehung jedes Körpers und zeichnet dort, was es sich gemerkt hat:"),

  H.code(stueck("kap10/1_kiste.dh", "FOR i = 0 TO LEN(koerper) - 1", "NEXT")),

  H.p("BOXROT zeichnet einen gedrehten Kasten um seine Mitte. Es will die ganze Breite und den Winkel in Grad; die Welt liefert den Winkel im Bogenmaß, DEG rechnet ihn um. Und die Welt rechnet in festen Schritten, wie der Sand in Kapitel 7:"),

  H.code(stueck("kap10/1_kiste.dh", "WHILE uhr >= TAKT", "WEND")),

  H.figure("kap10_kiste.png", "Die Kiste nach zweieinhalb Sekunden: 24 Bälle und Klötze sind gefallen, aufeinander gerollt und liegen kreuz und quer — kein Klotz steckt im anderen.", 420, 262),

  H.h2("Schritt 2: Türme und eine Schleuder"),

  H.p("Ein Turm sind zwei Säulen aus Steinen und ein Brett darüber. Die Steine stehen genau aufeinander, Mitte über Mitte, jeder zwei halbe Höhen über dem vorigen:"),

  H.code(stueck("kap10/2_tuerme.dh", "' Ein Turm:", "END SUB")),

  H.p("aufbauen legt jedes Mal eine neue Welt an — so baut die Taste R alles wieder auf, ohne dass man die alten Körper einzeln entfernen muss:"),

  H.code(stueck("kap10/2_tuerme.dh", "SUB aufbauen", "END SUB")),

  H.p("Für die Schleuder kommt VEC2 aus Kapitel 9 wieder. zug sagt, wie weit der Ball von der Schleuder weggezogen ist, höchstens 100:"),

  H.code(stueck("kap10/2_tuerme.dh", "' Wohin der Ball", "END FUNCTION")),

  H.p("Beim Loslassen entsteht der Ball an der Schleuder, und PHYS2D_SET_VEL gibt ihm sein Tempo — in die Gegenrichtung des Zugs, zwölfmal so viele Punkte je Sekunde, wie der Zug lang war. Von da an fliegt er, wie die Welt es will:"),

  H.code(stueck("kap10/2_tuerme.dh", "SUB abschiessen", "END SUB")),

  H.p("Das ganze Programm:"),

  H.code(zeilen("kap10/2_tuerme.dh")),

  H.h2("Warum so?"),

  H.h3("Fertige Physik oder selbst rechnen?"),

  H.p("Kapitel 9 hat seine Physik selbst gerechnet, und das ging, weil Punkte einfach sind: Sie haben keine Größe, drehen sich nicht und stoßen nirgends an. Ein Klotz dagegen hat Ecken. Wann berührt eine Ecke eine Kante? Wie fest drückt ein Stapel auf den untersten Stein? Wie weit rutscht ein Klotz, bevor die Reibung ihn hält, und wohin dreht er sich, wenn ihn ein Ball an der Ecke trifft? Darauf gibt es gute Antworten, aber sie füllen Bücher. physics2d ist die Bibliothek Rapier, an der Leute seit Jahren arbeiten."),

  H.p("Darum die Regel: Für feste Körper, die fallen, stoßen und rollen, nimmt man fast immer die fertige Physik. Selbst rechnet man, wenn die Dinge keine festen Körper sind — Seile und Stoffe wie in Kapitel 9, Sand wie in Kapitel 7, ein Schwarm —, oder wenn ein Spiel sich bewusst anders anfühlen soll als die Wirklichkeit."),

  H.h3("Ein Griff und Nummern"),

  H.pmix(["Die Welt selbst steckt in Drachenhauch, nicht im Programm; ", ["welt", true], " ist nur der Griff, über den man mit ihr spricht. Ein Körper ist eine Nummer in dieser Welt. Das ist ein Muster, das Drachenhauch oft benutzt — bei Bildern, Klängen, Datenbanken —, und es hat einen Preis: Die Nummer allein sagt nicht, ob es den Körper noch gibt. Seit diesem Kapitel bricht ein Befehl mit einem Körper, den es nicht mehr gibt, mit einer Meldung ab, und PHYS2D_BODY_EXISTS fragt vorher."]),

  H.h3("Feste Schritte"),

  H.p("Wie im Sandkasten rechnet die Welt in Schritten von genau 1/60 Sekunde, nicht in DELTA(). Hier gibt es einen zweiten Grund: Eine Physik mit Stößen ist empfindlich. Das Programm code/kap10/3_gleich.dh lässt einen Ball viermal in denselben Stapel aus sechs Klötzen fliegen, zweimal in festen Schritten und zweimal in Schritten, die schwanken wie DELTA() — mal etwas kürzer, mal etwas länger, zusammen genau zwei Sekunden. Danach fragt es, wo der oberste Klotz liegt:"),

  H.code(stueck("kap10/3_gleich.dh", "WHILE zeit < 2.0", "WEND")),

  H.p("In festen Schritten lag er bei jedem Lauf bei x = 487,7, auf die Zehntelstelle gleich. In schwankenden Schritten lag er bei 480,5 und 493,5, beim nächsten Start bei 462,6 und 460,4, beim übernächsten bei 443,6 und 493,8. Eine winzige Abweichung beim ersten Stoß wächst mit jedem weiteren. Für ein Spiel heißt das: Mit festen Schritten fällt ein Turm bei jedem Spieler gleich, und ein Fehler, den jemand meldet, lässt sich nachstellen. Dazu kommt: PHYS2D_STEP nimmt höchstens 0,05 Sekunden auf einmal; wer bei einem Ruckler DELTA() hineingibt, verliert den Rest."),

  H.h2("Was schiefgehen kann"),

  H.h3("Ganze statt halbe Maße"),

  H.p("Wer für einen Stein von 28 mal 28 auch 28 und 28 an PHYS2D_ADD_BOX gibt, bekommt Steine von 56 mal 56 — und zeichnet sie 28 groß:"),

  H.code(stueck("kap10/fehler_halb.dh", "FOR i = 0 TO 5", "NEXT")),

  H.figure("kap10_fehler_halb.png", "Ganze statt halbe Maße: Für die Physik sind die Steine doppelt so groß, wie sie gezeichnet werden. Sie stehen aufeinander, aber auf dem Bild schweben sie mit Lücken.", 420, 262),

  H.p("Eine Meldung gibt es nicht, denn beides ist erlaubt. Das Bild verrät es: Steine, die schweben, Bälle, die an unsichtbaren Kanten abprallen. Die Abhilfe ist eine feste Gewohnheit — beim Anlegen halbe Maße, beim Zeichnen mal 2, wie in klotz und zeichnen."),

  H.h3("Ein Körper, den es nicht mehr gibt"),

  H.code(zeilen("kap10/fehler_entfernt.dh")),
  H.code(["Laufzeitfehler in fehler_entfernt.dh:8: PHYS2D_BODY_X: einen Koerper 0",
          "gibt es nicht -- nie angelegt oder mit PHYS2D_REMOVE entfernt",
          "(PHYS2D_BODY_EXISTS fragt danach)"], { out: true }),

  H.p("Entfernt man einen Körper aus der Welt, muss er auch aus den eigenen Feldern heraus — sonst fragt zeichnen nach ihm. (Bis zur Fassung 2026.52 kam keine Meldung: Ein entfernter Körper lag still bei 0, 0, und man wunderte sich über einen Ball, der oben links in der Ecke klebte. Gefunden beim Schreiben dieses Kapitels.)"),

  H.h3("Der Winkel in Grad"),

  H.pmix(["PHYS2D_BODY_ANGLE liefert das Bogenmaß, BOXROT will Grad. Ohne DEG dreht sich ein umfallender Klotz auf dem Bild um weniger als zwei Grad, während er in der Welt längst auf der Seite liegt — er steckt dann scheinbar im Boden. Bogenmaß und Grad kennen wir aus der Werkstatt und vom Ziffernblatt; hier ist die Stelle, an der man sie verwechselt."]),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Gib dem Ball den Radius 30 statt 14 (an drei Stellen). Was wird aus den Türmen?"),
  H.bulletRich("Drehen 2. ", "Setze TAKT auf 1.0 / 15. Was passiert mit den Türmen, wenn der Ball trifft, und wie schnell läuft alles?"),
  H.bulletRich("Drehen 3. ", "Die Schwerkraft des Mondes: PHYS2D_SET_GRAVITY(welt, 0, 160) gleich nach PHYS2D_NEW. Wie musst du jetzt zielen?"),
  H.bulletRich("Erweitern 1. ", "Domino: statt der Türme eine Reihe schmaler, hoher Steine, die einander umwerfen."),
  H.bulletRich("Erweitern 2. ", "Eine Abrissbirne: ein schwerer Ball an einem Seil, der in die Türme schwingt. Ein Seil zwischen zwei Körpern legt PHYS2D_ADD_ROPE(welt, a, b, länge) an."),
  H.bulletRich("Erfinden. ", "Gelenke: PHYS2D_ADD_HINGE(welt, a, b, x, y) verbindet zwei Körper an einem Punkt, um den sie sich drehen können. Bau eine Brücke aus Brettern, eine Wippe, eine Tür oder eine Kanone, deren Rohr sich mit der Maus dreht. Anhang A zeigt die Brücke."),

  H.figure("kap10_bruecke.png", "Eine Möglichkeit für die Erfinden-Übung: zwölf Bretter, an Scharnieren aneinandergehängt — die Brücke aus Kapitel 9, diesmal mit der fertigen Physik. Vier Klötze sind darauf gefallen, und sie hängt durch.", 420, 262),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Modul physics2d: ", "alle Befehle, darunter PHYS2D_APPLY_IMPULSE für Stöße, PHYS2D_LOCK_ROTATION für Spielfiguren, die nicht umkippen sollen, und PHYS2D_SLEEP für große Aufbauten."),
  H.bulletRich("Handbuch, Modul physics3d: ", "dasselbe in drei Dimensionen."),
];
