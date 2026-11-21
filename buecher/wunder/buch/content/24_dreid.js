const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.part("Teil VII · Die dritte Dimension"),

  H.p("Bis hierher war alles flach. In diesem letzten Teil bekommt die Welt eine Tiefe: Körper im Raum, eine Kamera, die man herumträgt, Licht, Schatten und Dunst, eine Landschaft zum Hineinfliegen. Die Programme werden dabei nicht viel länger, denn das Schwere — welcher Punkt der Welt wo auf dem Bildschirm landet — rechnet die Grafikkarte. Zum Schluss kommt alles zusammen, was das Buch gezeigt hat, in einer Demo mit Musik, die man an andere weitergeben kann."),

  H.chapter("Würfel, Licht und Schatten", 24),

  H.h2("Was wir bauen"),

  H.figure("kap24_2_muehle.png", "Eine Windmühle auf einer Wiese: Die Flügel drehen sich und werfen Schatten, in der Ferne verschwimmen Bäume und Häuser im Dunst.", 470, 294),

  H.p("Die dritte Dimension. Eine Windmühle steht auf einer Wiese, ihre Flügel drehen sich langsam, und ihr Schatten wandert über das Gras. Ringsum zwei Häuser und ein paar Tannen, und weiter hinten verschwimmt alles im Dunst. Mit gedrückter Maustaste dreht man die Kamera um die Mühle, mit dem Mausrad kommt man näher. Das Programm ist nicht länger als die zweidimensionalen der letzten Kapitel. Neu sind drei Dinge: eine Kamera, die bestimmt, von wo man schaut; Licht, das Flächen hell und dunkel macht; und Matrizen, mit denen ein Teil an einem anderen hängt — die Flügel an der Nabe."),

  H.h2("Der Plan"),

  H.bulletRich("Der erste Würfel: ", "eine Kamera, ein Raster, Formen im Raum."),
  H.bulletRich("Modelle: ", "Teile, die einmal gebaut und oft gezeichnet werden."),
  H.bulletRich("Licht, Dunst und Schatten: ", "eine Sonne, etwas Grundhelligkeit, Nebel in der Ferne."),
  H.bulletRich("Matrizen: ", "die Flügel hängen an der Nabe, die Nabe an der Mühle."),
  H.bulletRich("Eine Kamera zum Anfassen: ", "Winkel, Höhe und Abstand mit der Maus."),

  H.h2("Schritt 1: der erste Würfel"),

  H.code(zeilen("kap24/1_wuerfel.dh")),

  H.figure("kap24_1_wuerfel.png", "Schritt 1: ein Würfel, eine Kugel und ein Drahtgitter auf einem Raster, von einer kreisenden Kamera aus gesehen.", 420, 262),

  H.pmix([["IMPORT \"g3d\"", true], " holt die 3D-Befehle. ", ["CAMERA3D", true], " stellt die Kamera auf: die ersten drei Zahlen sind, wo sie steht, die nächsten drei, wohin sie blickt, und die letzte, wie weit sie sieht — 45 Grad von unten nach oben, etwa wie ein Fotoapparat. Weil der Standort mit COS und SIN im Kreis läuft, kreist die Kamera um den Ursprung und schaut immer auf ihn. Die Koordinaten sind keine Bildpunkte mehr, sondern Einheiten im Raum, und y zeigt nach oben."]),

  H.pmix([["GRID3D", true], " legt ein Raster auf den Boden — damit schätzt man Entfernungen. ", ["CUBE", true], " zeichnet einen Quader um einen Mittelpunkt, ", ["SPHERE", true], " eine Kugel, ", ["CUBE_WIRES", true], " nur die Kanten. Der Würfel steht bei y = 0.5, damit seine Unterseite auf dem Boden liegt. Und die Zeile Text oben? Die liegt immer obenauf: Drachenhauch zeichnet erst alles Dreidimensionale und dann alles Flache darüber, gleich in welcher Reihenfolge es im Programm steht."]),

  H.h2("Schritt 2: Modelle"),

  H.code(stueck("kap24/2_muehle.dh", "' Die Teile, je einmal gebaut", "DIM krone")),

  H.pmix(["Formen wie CUBE baut Drachenhauch in jedem Bild neu. Für die Mühle nehmen wir ", ["Modelle", true], ": ", ["MESH_CYLINDER", true], ", ", ["MESH_CONE", true], " und ihre Geschwister bauen ein Teil einmal und liefern eine Nummer, mit der man es danach beliebig oft zeichnet — die Tanne vierundzwanzigmal, den Flügel viermal. Gezeichnet wird mit ", ["MODEL(teil, x, y, z, groesse, farbe)", true], ". Eines muss man wissen: Zylinder und Kegel stehen auf dem Boden, von y = 0 bis zu ihrer Höhe; ein Quader aber steht um seine Mitte. Darum steht das Haus mit seinen 1,6 Einheiten Höhe bei y = 0.8 und das Dach darauf bei 1.6. ", ["MODEL_EX", true], " dreht ein Modell beim Zeichnen um eine Achse — das Dach mit vier Seiten um 45 Grad, damit es zum Haus passt."]),

  H.h2("Schritt 3: Licht, Dunst und Schatten"),

  H.code(stueck("kap24/2_muehle.dh", "' Licht: eine Sonne", "NEXT")),

  H.pmix(["Ohne Licht hat jede Fläche überall dieselbe Farbe, und eine Kugel sieht aus wie eine Scheibe. ", ["LIGHT_ENABLE", true], " schaltet die Beleuchtung ein, ", ["LIGHT_DIRECTIONAL", true], " setzt eine Sonne: Die drei Zahlen sind die Richtung, in die ihr Licht fällt — schräg nach unten. Flächen, die zu ihr zeigen, werden hell, abgewandte dunkel. Damit die dunklen nicht schwarz werden, gibt ", ["LIGHT_AMBIENT", true], " jeder Fläche ein wenig bläuliche Grundhelligkeit, wie der Himmel an einem echten Tag. ", ["LIGHT_FOG", true], " lässt alles mit der Entfernung im Dunst verschwinden, in der Farbe des Himmels, so verschwimmt der Horizont."]),

  H.pmix(["Den Schatten wirft das erste Sonnenlicht, sobald ", ["SHADOW_ENABLE", true], " ihn einschaltet; ", ["SHADOW_AREA", true], " bestimmt, wie groß das Stück Welt ist, das Schatten bekommt. Zuletzt bekommt jedes Modell ", ["MODEL_LIT", true], " — erst dann wird es beleuchtet, wirft Schatten und empfängt welche."]),

  H.h2("Schritt 4: die Flügel"),

  H.code(stueck("kap24/2_muehle.dh", "' Die Nabe sitzt vorn", "NEXT")),

  H.pmix(["Die Flügel sind das Schwierigste: Sie drehen sich um die Nabe, die Nabe sitzt vorn an der Mühle, und jeder Flügel ist um ein Viertel weiter gedreht als der vorige. Mit x, y und z allein wäre das viel Rechnerei mit Sinus und Kosinus. Mit ", ["Matrizen", true], " schreibt man nur auf, was nacheinander geschieht. Eine Matrix vom Typ ", ["MAT4", true], " ist eine Bewegung als Wert: ", ["MAT4_TRANSLATE", true], " verschiebt, ", ["MAT4_ROTATE_Z", true], " dreht um die z-Achse, und ", ["MAT4_MUL(a, b)", true], " setzt zwei Bewegungen zusammen — erst b, dann a."]),

  H.pmix(["Die Nabe: erst um drehung gedreht, dann an ihren Platz vorn am Dach geschoben — das ist ", ["anNabe", true], ". Ein Flügel: erst 1,9 Einheiten nach oben geschoben, dann um k Viertel um den Ursprung gedreht, und das Ganze dann noch durch anNabe. So hängt der Flügel an der Nabe, wie im echten Leben: Dreht sich die Nabe, drehen sich alle Flügel mit, und steht die Mühle woanders, wandert alles zusammen. ", ["MODEL_MATRIX", true], " zeichnet ein Modell mit so einer Matrix. Das ganze Programm:"]),

  H.code(zeilen("kap24/2_muehle.dh")),

  H.pmix(["Die Kamera rechnet in der Hauptschleife, wo sie steht: Mit gedrückter Maustaste ändern ", ["MOUSE_DELTA_X", true], " und ", ["MOUSE_DELTA_Y", true], " — wie weit sich die Maus seit dem letzten Bild bewegt hat — den Winkel und die Höhe, ", ["MOUSE_WHEEL", true], " den Abstand. COS und SIN machen daraus wieder einen Platz auf einem Kreis um die Mühle, und die Kamera blickt auf deren Mitte."]),

  H.h2("Warum so?"),

  H.h3("Was eine Kamera ist"),

  H.code(zeilen("kap24/3_kamera.dh")),
  H.code(["Kamera bei (0.0, 0.0, 10.0)",
          "  (0.0, 0.0, 0.0) -> (0.0, 0.0, -10.0)",
          "  (3.0, 0.0, 0.0) -> (3.0, 0.0, -10.0)",
          "  (0.0, 2.0, 0.0) -> (0.0, 2.0, -10.0)",
          "Kamera bei (10.0, 0.0, 0.0)",
          "  (0.0, 0.0, 0.0) -> (0.0, 0.0, -10.0)",
          "  (3.0, 0.0, 0.0) -> (0.0, 0.0, -7.0)",
          "  (0.0, 2.0, 0.0) -> (0.0, 2.0, -10.0)"], { out: true }),

  H.pmix(["Eine Kamera ist nichts, was im Raum herumfliegt. Sie ist eine Matrix — ", ["MAT4_LOOKAT", true], " baut sie aus Standort, Blickziel und der Richtung, die oben ist. Diese Matrix verschiebt und dreht die ganze Welt so, dass die Kamera danach im Ursprung steht und geradeaus nach −z blickt. Danach ist für jeden Punkt klar, wo er im Bild landet: x nach rechts, y nach oben, und wie weit vorn er liegt. Von vorn gesehen liegt der Punkt (3, 0, 0) drei Einheiten rechts der Mitte, zehn weit weg. Von der Seite gesehen liegt er genau in der Mitte, nur noch sieben weit weg — er ist ja auf die Kamera zugerückt."]),

  H.p("Warum so herum? Weil es einfacher ist, eine Welt um eine Kamera zu bewegen, als eine Kamera durch eine Welt. Für die Grafikkarte gibt es nur eine einzige Kamera, immer am selben Platz; was sich ändert, ist die eine Matrix, durch die alle Punkte gehen. Darum kostet es nichts, die Kamera in jedem Bild woanders hinzustellen — CAMERA3D baut nur diese Matrix neu."),

  H.h3("Warum Matrizen als Werte"),

  H.p("Eine Matrix ist in Drachenhauch ein Wert wie eine Zahl: Man legt sie in eine Variable, gibt sie einer Funktion, setzt zwei zusammen. Darum lässt sich eine Kette wie Mühle → Nabe → Flügel so hinschreiben, wie man sie denkt — jedes Teil bekommt die Matrix seines Trägers und hängt seine eigene Bewegung an. Ein Arm an einer Schulter, eine Hand am Arm, ein Mond an einem Planeten: Es ist immer dasselbe Muster, und die Übung Sonnensystem zeigt es noch einmal."),

  H.h2("Was schiefgehen kann"),

  H.h3("Grad und Bogenmaß"),

  H.code(zeilen("kap24/fehler_grad.dh")),

  H.figure("kap24_fehler_grad.png", "Drei Flügel ungleich verteilt, zwei fast übereinander: 90 als Bogenmaß gerechnet.", 380, 238),

  H.pmix(["Die Flügel stehen schief und ungleich, zwei fast übereinander. ", ["MODEL_EX", true], " nimmt Winkel in Grad, die MAT4-Befehle aus m3d aber im Bogenmaß — 90 sind dort keine Vierteldrehung, sondern über vierzehn ganze Umdrehungen und ein Stück. Eine Vierteldrehung ist PI / 2. Wer lieber in Grad denkt, rechnet mit ", ["RAD(90)", true], " um."]),

  H.h3("Die Matrizen in der falschen Reihenfolge"),

  H.code(zeilen("kap24/fehler_reihenfolge.dh")),

  H.figure("kap24_fehler_reihenfolge.png", "Statt eines Flügelkreuzes um die Nabe ein kleines Kreuz über ihr.", 380, 238),

  H.p("Hier sind nur die zwei Matrizen vertauscht, und aus dem Flügelkreuz wird ein kleines Kreuz über der Nabe. MAT4_MUL wendet die rechte Matrix zuerst an: Erst dreht sich der Flügel um seine eigene Mitte, dann wandert er nach oben — alle vier an dieselbe Stelle. Beim Lesen hilft es, eine Kette von rechts nach links zu sprechen: „nach oben schieben, dann drehen“, nicht umgekehrt."),

  H.h3("Licht für ein Modell, bevor es Licht gibt"),

  H.code(zeilen("kap24/fehler_licht.dh")),
  H.code(["Laufzeitfehler in fehler_licht.dh:6: MODEL_LIT: zuerst LIGHT_ENABLE() /",
          "ein Licht hinzufuegen"], { out: true }),

  H.p("MODEL_LIT verbindet ein Modell mit der Beleuchtung — und die gibt es erst nach LIGHT_ENABLE. Die Reihenfolge ist also immer: Licht einschalten, Lichter setzen, dann die Modelle beleuchten."),

  H.h3("Ein Punktlicht, das nicht leuchtet"),

  H.p("Beim Schreiben der Übung Sonnensystem blieben die Planeten fast schwarz, obwohl in der Sonne ein Punktlicht saß. Drachenhauch ließ ein Punktlicht so schnell schwächer werden, dass schon vier Einheiten weg nur ein Sechzehntel ankam — und ein Modell, das mit MODEL_INSTANCED gezeichnet wurde, war unter demselben Licht viermal so hell. Jetzt werden beide gleich schwach: in vier Einheiten etwa halb so hell, in zehn noch ein Fünftel."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Ändere in CAMERA3D die 45 auf 90. Was passiert?"),
  H.bulletRich("Drehen 2. ", "Ändere bei LIGHT_FOG die 0.02 auf 0.08."),
  H.bulletRich("Drehen 3. ", "Ändere SHADOW_AREA(30, 40) auf SHADOW_AREA(8, 40). Welche Schatten fehlen jetzt?"),
  H.bulletRich("Erweitern 1. ", "Ein Sonnensystem: Die Sonne leuchtet, die Erde kreist um sie und dreht sich, der Mond kreist um die Erde, und weiter draußen zieht Mars seine Bahn."),
  H.bulletRich("Erweitern 2. ", "Tag und Nacht: Die Sonne zieht ihre Bahn, der Himmel wird abends rot und nachts dunkel, und vor einem Haus geht eine Laterne an."),
  H.bulletRich("Erfinden. ", "Ein Labyrinth von oben, das das Programm selbst baut, mit einer Kugel, die man zum Ziel rollt. Oder: eine Stadt aus Würfeln; ein Roboterarm mit drei Gelenken; ein Karussell. Anhang A zeigt das Labyrinth."),

  H.figure("kap24_labyrinth.png", "Erfinden: ein selbst gebautes Labyrinth, die blaue Kugel oben links, das Ziel unten rechts.", 420, 262),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Module → g3d: ", "alle Formen und Modelle, Licht, Schatten, Anklicken im Raum."),
  H.bulletRich("Handbuch, Module → m3d: ", "Vektoren, Quaternionen und Matrizen, und wie man mit ihnen eigene Kameras baut."),
  H.bulletRich("examples/91_lighting.dh bis 96_ibl.dh: ", "Licht, Nebel, Schatten, Normal-Maps und spiegelnde Metalle."),
];
