const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.part("Teil V · Spielereien mit Eingaben"),

  H.p("Bisher hat man den Programmen meist zugesehen. In diesem Teil spielt man selbst: Die Maus malt, die Maus spielt einen Ton, und am Ende hört ein Bild der Musik zu und tanzt. Dafür muss ein Programm sich merken, was im letzten Bild war, und Eingaben auseinanderhalten, die einmal gelten, von solchen, die andauern."),

  H.chapter("Malen mit Symmetrie", 17),

  H.h2("Was wir bauen"),

  H.figure("kap17_2_kaleidoskop.png", "Vier Striche, gemalt in einem einzigen Sechstel des Kreises — das Programm zeichnet jeden zwölfmal: sechsmal gedreht und jedes Mal auch gespiegelt.", 470, 294),

  H.p("Ein Kaleidoskop zum Malen. Man zieht mit der Maus einen Strich, und er erscheint sechsmal rund um die Mitte und jedes Mal auch gespiegelt. Ein Gekritzel in einer Ecke wird zu einem Muster wie aus Glasscherben. Mit den Tasten 2 bis 9 wählt man, in wie viele Teile der Kreis zerfällt, M schaltet das Spiegeln, das Mausrad macht den Strich dicker. Unterwegs geht es um zwei Dinge: Wie sich ein Programm etwas von einem Bild zum nächsten merkt, und warum ein Bild, das stehen bleiben soll, ein eigenes Zeichenblatt braucht."),

  H.h2("Der Plan"),

  H.bulletRich("Ein Blatt, das bleibt: ", "Gemalt wird auf ein Zeichenziel, das nicht bei jedem Bild geleert wird."),
  H.bulletRich("Von Bild zu Bild: ", "Ein Strich verbindet die Stelle, an der die Maus im letzten Bild war, mit der, an der sie jetzt ist."),
  H.bulletRich("Winkel und Abstand: ", "Wer einen Punkt als Winkel und Abstand von der Mitte beschreibt, dreht ihn mit einem Plus und spiegelt ihn mit einem Minus."),
  H.bulletRich("Darüber, was nur zum Ansehen da ist: ", "Hilfslinien und Ringe kommen jedes Bild neu auf das Fenster, nicht auf das Blatt."),

  H.h2("Schritt 1: das kleinste Malprogramm"),

  H.p("Bevor es symmetrisch wird, malen wir einfach nur. Das ganze Programm:"),

  H.code(zeilen("kap17/1_malen.dh")),

  H.pmix(["Das Blatt ist ein Zeichenziel wie in Kapitel 15, und auch hier steht beim Anlegen ein TRUE: ", ["RENDERTARGET_NEW(BREITE, HOEHE, TRUE)", true], ". Es heißt, dass das Ziel nicht zu Beginn jedes Bildes geleert wird; was einmal darauf steht, bleibt. Warum das Fenster selbst das nicht kann, steht unter „Warum so?“."]),

  H.pmix([["MOUSE_DOWN(0)", true], " ist wahr, solange die linke Maustaste gedrückt ist — in jedem Bild, nicht nur im ersten. Zwischen zwei Bildern vergeht eine sechzigstel Sekunde, und in der Zeit kann die Maus ein gutes Stück weit kommen. Ein Punkt je Bild ergäbe deshalb eine Perlenkette statt eines Strichs. Das Programm merkt sich darum in alt_x und alt_y, wo die Maus im letzten Bild war, und zieht eine Linie von dort bis hierher. Das ist der Zustand über mehrere Bilder: zwei Variablen, die am Ende jeder Runde den Wert von jetzt bekommen, damit die nächste Runde weiß, was vorher war."]),

  H.pmix(["Die Leertaste wischt mit ", ["KEY_HIT(KEY_SPACE)", true], ". KEY_HIT ist nur in dem einen Bild wahr, in dem die Taste heruntergeht, KEY_DOWN dagegen in jedem Bild, solange sie unten ist. Für Wischen ist das gleich; für einen Schalter, wie gleich M für das Spiegeln, ist es der ganze Unterschied."]),

  H.h2("Schritt 2: Winkel und Abstand"),

  H.p("Jetzt soll jeder Strich mehrfach erscheinen. In x und y ist das Drehen um eine Mitte mühsam — für jede Kopie eine Drehung mit Sinus und Kosinus, die man sich merken muss. Viel einfacher wird es, wenn man einen Punkt anders beschreibt: nicht als „so weit rechts und so weit unten“, sondern als „in diese Richtung, so weit weg von der Mitte“. Das sind Polarkoordinaten, ein Winkel und ein Abstand."),

  H.code(stueck("kap17/2_kaleidoskop.dh", "SUB strich", "END SUB")),

  H.pmix(["ATAN2 kennen wir aus Kapitel 8: Es macht aus dem Weg von der Mitte zum Punkt einen Winkel. HYPOT ist der Satz des Pythagoras, die Länge dieses Wegs. Danach ist Drehen ein Plus: Ein Sechstel des Kreises sind 2π/6, und w1 + dreh ist derselbe Punkt, um so viel weitergedreht. Spiegeln ist ein Minus: −w1 ist der Punkt, an der waagerechten Linie durch die Mitte gespiegelt, denn was vorher 30 Grad über der Linie lag, liegt jetzt 30 Grad darunter. dreh − w1 dreht dieses Spiegelbild dann weiter. Zurück in x und y rechnet linie:"]),

  H.code(stueck("kap17/2_kaleidoskop.dh", "SUB linie", "END SUB")),

  H.p("Kosinus und Sinus machen aus Winkel und Abstand wieder einen Punkt — dieselbe Rechnung wie beim Ziffernblatt in Kapitel 4. Ein Strich besteht aus vielen kurzen Linien, eine je Bild. Dicke Linien haben eckige Enden, und wo zwei in einem Winkel aneinanderstoßen, bleibt eine Kerbe. Der kleine Kreis am Ende füllt sie, darum sind die Striche rund."),

  H.h2("Schritt 3: was nur zum Ansehen da ist"),

  H.p("Beim Malen hilft es zu sehen, wo die Teile aneinandergrenzen und wo die Spiegelbilder der Maus gerade sind. Diese Hilfe darf aber nicht ins Bild: Wer das Kaleidoskop speichert, will seine Striche, keine Hilfslinien. Darum kommt sie nicht auf das Blatt, sondern jedes Bild neu auf das Fenster, über das Blatt:"),

  H.code(stueck("kap17/2_kaleidoskop.dh", "RENDERTARGET_DRAW(blatt", "IF hilfe")),
  H.code(stueck("kap17/2_kaleidoskop.dh", "SUB hilfe_zeichnen", "END SUB")),

  H.p("Hier zeigt sich, wozu die beiden Flächen gut sind. Das Blatt behält alles; das Fenster vergisst alles. Was nur einen Augenblick gelten soll — die Ringe folgen der Maus, die Zeile oben zeigt die Einstellungen —, gehört auf das Fenster, und im nächsten Bild steht es dort, wo es dann hingehört, ohne dass jemand die alte Stelle wegwischen muss."),

  H.h2("Schritt 4: Tasten und Mausrad"),

  H.code(stueck("kap17/2_kaleidoskop.dh", "DIM n AS INTEGER", "dicke = CLAMP")),

  H.pmix(["KEY_HIT nimmt auch einen Buchstaben oder eine Ziffer als Text. ", ["STR$(n)", true], " macht aus der Zahl 7 den Text \"7\", und die Schleife fragt so alle acht Zifferntasten ab, statt acht IF-Zeilen untereinander. ", ["MOUSE_WHEEL()", true], " liefert, wie weit das Rad seit dem letzten Bild gedreht wurde: 1 für eine Raste nach vorn, −1 nach hinten, meist 0. ", ["CLAMP", true], " hält die Dicke zwischen 1 und 20. Das ganze Programm:"]),

  H.code(zeilen("kap17/2_kaleidoskop.dh")),

  H.figure("kap17_neun.png", "Dieselben Striche mit der Taste 9 und ohne Spiegeln (M): neun Kopien, die sich wie ein Windrad drehen.", 420, 262),

  H.p("Ohne Spiegeln wirkt das Muster, als drehe es sich: Alle Kopien zeigen in dieselbe Richtung. Erst das Spiegelbild macht daraus die ruhige Form eines Kaleidoskops — darum hat ein echtes Kaleidoskop Spiegel und keine Drehachse."),

  H.h2("Warum so?"),

  H.h3("Warum ein Bild, das bleibt, ein eigenes Ziel braucht"),

  H.p("Man könnte meinen, das Fenster sei selbst ein Blatt: Was man zeichnet, steht da, bis man es übermalt. So war es tatsächlich in den Heimcomputern der Achtziger, und so verhalten sich auch heute noch manche einfachen Zeichenprogramme. In Drachenhauch nicht, und der Grund ist das Flimmern."),

  H.p("Würde ein Programm direkt in das Bild malen, das man gerade sieht, sähe man es beim Malen: halb gelöschte Figuren, Teile des alten Bildes über dem neuen. Darum zeichnet heute fast jedes Grafiksystem in ein Bild, das man noch nicht sieht, und zeigt es erst, wenn es fertig ist; FLIP heißt so, weil es die beiden umdreht. Das unsichtbare Bild ist danach nicht mehr das von eben — oft ist es das von vor zwei Bildern, oder sein Inhalt ist gar nicht festgelegt. Drachenhauch geht noch einen Schritt weiter: Es merkt sich zuerst nur, was gezeichnet werden soll, und zeichnet beim FLIP alles auf eine frisch geleerte Fläche. Wer ohne CLS einen Kreis je Bild zeichnet, sieht deshalb immer nur einen."),

  H.p("Ein Bild, das wachsen soll, braucht also einen eigenen Speicher, den niemand leert: das Zeichenziel mit TRUE. Es ist ein Bild in der Grafikkarte wie das Fenster, nur sieht man es nicht, bis RENDERTARGET_DRAW es aufs Fenster stempelt. Das kostet einmal je Bild ein Rechteck — für die Grafikkarte nichts."),

  H.h3("Warum Winkel und Abstand"),

  H.p("Drehen in x und y geht auch: x′ = x · cos w − y · sin w, y′ = x · sin w + y · cos w. Das ist dieselbe Rechnung, nur steckt der Winkel darin, und man sieht ihr nicht an, dass es eine Drehung ist. In Polarkoordinaten steht da w + dreh, und das kann man lesen. Dazu kommt das Spiegeln: In x und y ist ein Spiegeln an einer schrägen Achse eine eigene Formel, in Polarkoordinaten ist es ein Minus vor dem Winkel. Die Regel dahinter gilt weit über dieses Kapitel hinaus: Man rechnet in der Beschreibung, in der die Aufgabe einfach ist, und rechnet erst ganz am Ende zurück."),

  H.h3("Warum KEY_HIT für Schalter"),

  H.p("Ein Tastendruck dauert für einen Menschen kurz, für ein Programm mit 60 Bildern je Sekunde aber fünf bis zehn Bilder. Mit KEY_DOWN würde M das Spiegeln in jedem dieser Bilder umschalten — an, aus, an, aus —, und am Ende hinge es vom Zufall ab, ob es an ist. KEY_HIT gilt nur in dem Bild, in dem die Taste heruntergeht, und schaltet darum genau einmal. Dieselbe Frage stellt sich bei der Maus: MOUSE_DOWN zum Malen, solange sie unten ist, MOUSE_HIT für etwas, das einmal geschehen soll, wenn man klickt."),

  H.h2("Was schiefgehen kann"),

  H.h3("Der Strich springt"),

  H.code(zeilen("kap17/fehler_sprung.dh")),

  H.figure("kap17_fehler_sprung.png", "Zwei Wellen, weit auseinander gemalt: Der erste Strich beginnt oben links in der Ecke, der zweite am Ende des ersten.", 420, 262),

  H.pmix(["Hier wird die alte Stelle nur nachgeführt, solange man malt. Lässt man los und setzt woanders neu an, steht in alt_x und alt_y noch das Ende des letzten Strichs, und die erste Linie verbindet die beiden. Der allererste Strich beginnt sogar in der Ecke (0, 0), denn so stehen alt_x und alt_y am Anfang. Abhilfe: die alte Stelle in jedem Bild nachführen, wie es die Programme dieses Kapitels tun — oder beim Aufsetzen, ", ["IF MOUSE_HIT(0)", true], ", zuerst alt_x und alt_y auf die Maus setzen."]),

  H.h3("ATAN2 verdreht"),

  H.code(zeilen("kap17/fehler_atan2.dh")),
  H.code(["Maus bei 600, 250",
          "gemalt bei 400, 450"], { out: true }),

  H.p("ATAN2 will zuerst das y und dann das x — umgekehrt als man es gewohnt ist. Vertauscht man die beiden, ist der Winkel an der Diagonale gespiegelt: Die Maus steht rechts der Mitte, gemalt wird darunter. Im Kaleidoskop fällt das nicht als Fehler auf, denn es entsteht ja trotzdem ein symmetrisches Muster. Nur unter der Maus erscheint fast nie etwas. Wer so etwas sieht, schaut zuerst auf die Reihenfolge der Argumente."),

  H.h3("Ein Blatt ohne TRUE"),

  H.p("Fehlt beim Anlegen das TRUE, wird das Blatt zu Beginn jedes Bildes geleert, genau wie das Fenster. Beim Malen sieht man dann nur das kurze Stück, das in diesem Bild dazukam — eine Spur, so lang wie die Bewegung einer sechzigstel Sekunde —, und lässt man los, ist alles weg. Es gibt keine Meldung, denn ein Ziel, das sich leert, ist für Schweife und Zwischenbilder genau richtig."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Drücke 2 und male mit und ohne Spiegeln. Wie viele Kopien siehst du jeweils, und wo liegen die Spiegelachsen?"),
  H.bulletRich("Drehen 2. ", "Ersetze HSV(farbton, 0.7, 1) durch HSV(HYPOT(x - MITTE_X, y - MITTE_Y), 0.7, 1). Wovon hängt die Farbe jetzt ab?"),
  H.bulletRich("Drehen 3. ", "Lösche die beiden Zeilen alt_x = x und alt_y = y am Ende der Schleife. Was malt das Programm jetzt?"),
  H.bulletRich("Erweitern 1. ", "Ein zweiter Pinsel: Die Taste P wechselt zwischen Linie und Sprühdose. Die Sprühdose setzt je Bild ein paar Tupfen um die Maus herum."),
  H.bulletRich("Erweitern 2. ", "Rückgängig: Z nimmt den letzten Strich zurück, auch mehrmals hintereinander. Tipp: RENDERTARGET_IMAGE macht ein Foto des Blatts, DRAWIMAGE legt es wieder darauf."),
  H.bulletRich("Erfinden. ", "Eine Tapete: Das Fenster ist in Kacheln geteilt, und jeder Strich erscheint in allen, jede mit ihrer eigenen Mitte; S sichert das Blatt als PNG. Oder: ein Kaleidoskop, das beim Malen ganz langsam die Zahl der Teile wechselt; Striche, die mit der Zeit verblassen wie in Kapitel 15; eine Spiegelung an einer senkrechten Achse für Gesichter und Schmetterlinge. Anhang A zeigt die Tapete."),

  H.figure("kap17_pinsel.png", "Erweitern 1: die Sprühdose. Jeder Tupfen erscheint zwölfmal, wie ein Strich.", 420, 262),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Grafik-Befehle → Render-Ziele: ", "Zeichenziele anlegen, behalten, leeren und als Bild auslesen."),
  H.bulletRich("Handbuch, Grafik-Befehle → Eingabe: Tastatur und Maus: ", "alle Abfragen für Maus und Tastatur, mit dem Unterschied zwischen _DOWN, _HIT und _RELEASED."),
  H.bulletRich("Kapitel 15: ", "dasselbe Blatt mit TRUE, dort für Leuchtspuren, die langsam verblassen."),
];
