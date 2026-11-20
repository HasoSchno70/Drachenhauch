const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.part("Teil IV · Bilder, Licht und Farbe"),

  H.p("Bisher haben die Programme Dinge gezeichnet: Punkte, Linien, Kästen, ab und zu ein Bild. In diesem Teil rechnet das Programm das Bild selbst, Punkt für Punkt — eine Landschaft aus Zufall, ein Fraktal, ein Leuchten. Dafür muss es schnell sein, und zum ersten Mal wird wichtig, wie Drachenhauch ein Programm eigentlich ausführt."),

  H.p("Das erste Kapitel zeigt, was der Rechner leisten kann, wenn man ihn lässt: Hunderttausende Bildpunkte, jeder mit bis zu tausenden Rechenschritten, in Bruchteilen einer Sekunde. Die anderen zeigen, wie Rauschen Wolken macht, wie die Grafikkarte selbst rechnet und warum eine Farbe eine Zahl ist."),

  H.chapter("Die Mandelbrotmenge", 13),

  H.h2("Was wir bauen"),

  H.figure("kap13_seepferdchen.png", "Das Seepferdchental, 320-fach vergrößert: Spiralen, die sich in Spiralen winden, und darin wieder kleine schwarze Inseln. Alles aus einer einzigen Formel.", 470, 294),

  H.p("Die Mandelbrotmenge ist das bekannteste Bild der Mathematik. Ihre Formel ist so kurz, dass sie in eine Zeile passt, und ihr Rand so verwickelt, dass man beliebig tief hineinzoomen kann und immer Neues findet — Spiralen, Rüssel, Sterne und immer wieder kleine Kopien der ganzen Menge. Wir bauen ein Programm, das sie zeichnet und mit der Maus hineinzoomt, und messen dabei zum ersten Mal genau, wie schnell Drachenhauch rechnet und warum."),

  H.h2("Der Plan"),

  H.bulletRich("Jeder Bildpunkt ", "steht für eine Zahl c."),
  H.bulletRich("Für diese Zahl ", "rechnet das Programm immer wieder z = z · z + c, beginnend bei z = 0."),
  H.bulletRich("Läuft z davon, ", "gehört c nicht zur Menge, und die Farbe sagt, wie schnell. Bleibt z klein, gehört c dazu und wird schwarz."),

  H.h2("Schritt 1: das ganze Bild"),

  H.p("Die Zahlen c und z sind keine gewöhnlichen Zahlen, sondern Paare (x, y) — Punkte in der Ebene, die man addieren und miteinander malnehmen kann. Addieren geht wie bei Vektoren. Malnehmen hat eine eigene Regel: (x, y) · (x, y) ist (x · x − y · y, 2 · x · y). Mehr braucht man nicht zu wissen; Mathematiker nennen solche Paare komplexe Zahlen. Damit ist die ganze Rechnung:"),

  H.code(stueck("kap13/1_mandel.dh", "FUNCTION schritte", "END FUNCTION")),

  H.pmix(["Die Schleife hat zwei Gründe aufzuhören. Entweder sind GRENZE Schritte gerechnet — dann ist z klein geblieben, und c gehört (vermutlich) zur Menge. Oder z ist weiter als 2 vom Nullpunkt weg; von da an läuft es sicher davon, und ", ["EXIT DO", true], " verlässt die Schleife sofort. ", ["DO WHILE … LOOP", true], " ist dieselbe Schleife wie WHILE … WEND, in der Form, die man aus anderen BASICs kennt; EXIT DO ist ihr Ausgang mitten drin (BREAK täte dasselbe und geht in jeder Schleife). xx und yy merken sich die Quadrate, weil sie zweimal gebraucht werden."]),

  H.p("Die Farbe hängt an der Zahl der Schritte: schwarz für die Menge, sonst ein Farbton, der mit den Schritten wandert, und eine Helligkeit, die mit ihnen wächst:"),

  H.code(stueck("kap13/1_mandel.dh", "FUNCTION farbe", "END FUNCTION")),

  H.p("Dann geht das Programm alle 800 mal 500 Bildpunkte durch. Gerechnet wird nur einmal, in ein Bild, das sein Aussehen behält — wie der Garten in Kapitel 12:"),

  H.code(stueck("kap13/1_mandel.dh", "SUB rechnen", "END SUB")),

  H.figure("kap13_1_mandel.png", "Die ganze Mandelbrotmenge: das Apfelmännchen. Die Farbstufen außen zählen die Schritte — jede Stufe ein Schritt mehr, bis z davonläuft.", 420, 262),

  H.h2("Schritt 2: hineinzoomen"),

  H.p("Die Stufen im ersten Bild verraten, dass die Schritte ganze Zahlen sind. Für weiche Übergänge braucht es einen Bruchteil: wie weit z schon jenseits der Grenze war, als es sie überschritt. Je weiter, desto früher im letzten Schritt war es eigentlich schon draußen. Daraus macht eine Formel mit dem Logarithmus eine Kommazahl — sie funktioniert umso besser, je größer die Grenze ist, darum hier 16 statt 2 (xx + yy > 256):"),

  H.code(stueck("kap13/2_zoom.dh", "FUNCTION flucht", "END FUNCTION")),

  H.p("Zoomen heißt: einen kleineren Ausschnitt rechnen. Der Ausschnitt ist eine Mitte und eine Breite. Ein Klick macht den Punkt unter der Maus zur neuen Mitte und halbiert die Breite; je tiefer man kommt, desto mehr Schritte braucht der verwickelte Rand, darum wächst die Grenze mit:"),

  H.code(stueck("kap13/2_zoom.dh", "SUB rechnen", "END SUB")),

  H.p("Die Tasten 1, 2 und 3 führen zu drei Ausflugszielen. Das dritte liegt gut eine Billion Mal vergrößert — und dort, mitten in Spiralen, sitzt eine winzige Kopie der ganzen Menge:"),

  H.code(stueck("kap13/2_zoom.dh", "SUB ausflug", "END SUB")),

  H.figure("kap13_tief.png", "Ausflugsziel 3, 2⁴⁰-fach vergrößert: in der Mitte eine kleine Mandelbrotmenge, umgeben von Spiralen. Gerechnet mit bis zu 2500 Schritten je Punkt in knapp zwei Sekunden.", 420, 262),

  H.p("Das ganze Programm:"),

  H.code(zeilen("kap13/2_zoom.dh")),

  H.h2("Warum so?"),

  H.h3("Messen statt vermuten"),

  H.p("Das erste Bild sind 400 000 Bildpunkte mit zusammen gut elf Millionen Rechenschritten. Wie lange dauert das? Das Programm code/kap13/3_messen.dh rechnet das Bild ohne zu zeichnen und stoppt die Zeit. Auf meinem Rechner:"),

  H.code(["zahlen: 11405483 Schritte in 0.03 s"], { out: true }),

  H.p("Drei hundertstel Sekunden. Startet man dasselbe Programm mit der Umgebungsvariablen DHRT_JIT=aus, braucht es 1,8 Sekunden — sechzigmal so lange. Der Unterschied ist der Maschinencode. Normalerweise führt Drachenhauch ein Programm in seiner eigenen kleinen Maschine aus, der VM: Sie liest Befehl für Befehl und tut, was er sagt. Merkt dhrt, dass eine Schleife oder eine Funktion oft läuft, übersetzt es sie in Befehle für den Prozessor selbst — Maschinencode —, und von da an rechnet der Prozessor ohne den Umweg. Eingeschaltet ist das immer; DHRT_JIT=aus ist nur zum Vergleichen da."),

  H.p("Welche Funktionen übersetzt werden und warum andere nicht, sagt dhrt auf Nachfrage:"),

  H.code(["dhrt --jit 3_messen.dh"]),
  H.code(["mitzahlen: uebersetzt", "mitvec2: VM: Platz 0 hat den Typ 'vec2'",
          "bild: VM: Platz 0 hat den Typ 'string'"], { out: true }),

  H.h3("Warum die Rechnung nur Zahlen kennt"),

  H.p("Der Maschinencode kennt Zahlen — INTEGER, FLOAT, BOOLEAN — und Felder aus Zahlen. Was darüber hinausgeht, Texte, Objekte, Vektoren, gibt er an die VM ab. In 3_messen.dh steht dieselbe Rechnung ein zweites Mal, mit einem VEC2 z wie in Kapitel 9 statt zweier Zahlen x und y:"),

  H.code(stueck("kap13/3_messen.dh", "FUNCTION mitVec2", "END FUNCTION")),

  H.p("Sie ist hübscher zu lesen und liefert genau dieselben elf Millionen Schritte — aber sie braucht ohne Maschinencode 3,9 Sekunden und mit Maschinencode sogar 7,7. Der Maschinencode übersetzt die Schleife, kann die VEC2-Rechnung darin aber nicht selbst ausführen, gibt bei jedem Schritt an die VM ab und steigt danach wieder ein — das Hin und Her kostet mehr, als es spart. (Gefunden beim Schreiben dieses Kapitels und als Aufgabe für dhrt notiert; vielleicht ist der Unterschied kleiner, wenn du das liest.) Die Regel bleibt: In die innerste Schleife eines Programms, das Millionen Schritte rechnet, gehören nur Zahlen."),

  H.p("Ob die Rechnung in einer eigenen Funktion steht oder mitten im Hauptprogramm, ist für den Maschinencode übrigens gleich — beides gemessen 0,03 Sekunden. Die Funktion steht da, weil sie einen Namen hat und an einer Stelle steht: Die Zoom-Fassung ändert die Rechnung nur in flucht."),

  H.h2("Was schiefgehen kann"),

  H.h3("Die Zahlen gehen aus"),

  H.p("Klickt man vom dritten Ausflugsziel aus weiter hinein, zerfällt das Bild nach etwa zehn Klicks in Blöcke:"),

  H.figure("kap13_bloecke.png", "Zehn Klicks tiefer, 2⁵⁰-fach: Das Bild ist in Blöcke zerfallen. Die Rechnung ist nicht falsch — die Zahlen sind zu grob geworden.", 420, 262),

  H.p("Ein FLOAT hat etwa 16 gültige Stellen. Bei 2⁵⁰-facher Vergrößerung liegen benachbarte Bildpunkte so dicht beieinander, dass ihre x-Werte sich erst in der 17. Stelle unterscheiden würden — und dann sind sie gleich. Das kleine Programm fehler_genau.dh zählt, wie viele verschiedene Werte die 800 Punkte einer Zeile noch bekommen:"),

  H.code(zeilen("kap13/fehler_genau.dh")),
  H.code([
    "2 hoch 30: 800 verschiedene Werte",
    "2 hoch 40: 800 verschiedene Werte",
    "2 hoch 46: 410 verschiedene Werte",
    "2 hoch 48: 103 verschiedene Werte",
    "2 hoch 50: 27 verschiedene Werte",
    "2 hoch 52: 7 verschiedene Werte",
  ], { out: true }),

  H.p("Bei 2⁵⁰ teilen sich im Schnitt 30 Bildpunkte denselben Wert — daher die Blöcke. Eine Meldung gibt es nicht; nichts ist falsch gerechnet, nur zu grob. Programme, die tiefer wollen, rechnen mit besonderen, viel genaueren Zahlen und sind dafür viel langsamer."),

  H.h3("Zu wenige Schritte"),

  H.p("Mit GRENZE = 20 sieht die Menge aufgeblasen aus: Viele Punkte am Rand brauchen mehr als zwanzig Schritte, um davonzulaufen, und werden fälschlich schwarz. Je tiefer man zoomt, desto mehr Schritte braucht der Rand — darum wächst die Grenze in 2_zoom.dh mit der Vergrößerung."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Setze in Schritt 1 GRENZE auf 20 und dann auf 1000. Was ändert sich am Bild, was an der Zeit?"),
  H.bulletRich("Drehen 2. ", "Ändere in farbe die 8 in 2 und dann in 30."),
  H.bulletRich("Drehen 3. ", "Setze in flucht die 256 auf 4. Was passiert mit den weichen Übergängen?"),
  H.bulletRich("Erweitern 1. ", "Zoomen mit dem Mausrad, so dass der Punkt unter der Maus stehen bleibt."),
  H.bulletRich("Erweitern 2. ", "Die Taste S sichert das Bild als PNG-Datei; der Name sagt, wie tief es war."),
  H.bulletRich("Erfinden. ", "Julia-Mengen: Dieselbe Rechnung, aber c steht fest, und z beginnt beim Bildpunkt. Zu jedem Punkt der Mandelbrotmenge gibt es eine. Oder: eine automatische Fahrt, die langsam immer tiefer zoomt; eine andere Formel wie z · z · z + c. Anhang A zeigt die Julia-Mengen."),

  H.figure("kap13_julia.png", "Eine Möglichkeit für die Erfinden-Übung: die Julia-Menge zu einem Punkt im Seepferdchental. Ihre Spiralen gleichen denen der Mandelbrotmenge an dieser Stelle.", 420, 262),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Werkzeuge → Maschinencode: ", "dhrt --jit, DHRT_JIT=aus und DHRT_JIT_BILANZ, die zeigen, was übersetzt wird und wo die VM rechnet."),
  H.bulletRich("Handbuch, Sprache → Schleifen: ", "DO … LOOP in allen Formen, BREAK und CONTINUE."),
];
