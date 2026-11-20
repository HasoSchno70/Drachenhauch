const { zeilen, ohneKopf, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Die Werkstatt", 0),

  H.p("Bevor es losgeht, ein kurzer Gang durch die Werkstatt: wo man schreibt, wie man startet, wo die Fehler stehen und wo man nachschlägt. Und weil ein Buch mit diesem Titel nicht mit einer Liste von Menüpunkten anfangen sollte, steht am Ende des Kapitels das erste kleine Wunder."),

  H.h2("Die IDE in fünf Minuten"),

  H.p("Drachenhauch bringt seine eigene Entwicklungsumgebung mit, die IDE. Sie ist selbst ein Drachenhauch-Programm und nach der Installation im Startmenü zu finden. Links steht der Ordner, in dem man arbeitet, als Baum; in der Mitte der Text der offenen Datei; unten die Ausgabe des Programms und die Probleme, die die IDE beim Lesen findet."),

  H.figure("kap00_ide.png", "Die IDE mit dem Programm dieses Kapitels: links der Ordner, in der Mitte der Code, unten Ausgabe und Probleme. Rechts unten erklärt sie das Wort unter der Schreibmarke.", 480, 300),

  H.p("Für dieses Buch genügen eine Handvoll Handgriffe:"),

  H.table([
    ["Strg+N", "eine neue Datei"],
    ["Strg+S", "sichern — beim ersten Mal mit Namen; er endet auf .dh"],
    ["F5", "starten"],
    ["Umschalt+F5", "ein laufendes Programm anhalten"],
    ["F1", "das Wort unter der Schreibmarke im Handbuch nachschlagen"],
    ["Strg+F3", "alle eingebauten Befehle, mit Beschreibung und Suchfeld"],
  ], { headers: ["Taste", "was sie tut"], widths: [2200, 6826], mono: [0] }),

  H.p("Die Programme dieses Buchs öffnen ein eigenes Fenster. Man schließt es mit dem Kreuz oder mit Esc; dafür sorgt in jedem Programm die Zeile, mit der seine Schleife beginnt."),

  H.h2("Fehler sieht man, bevor man startet"),

  H.p("Die IDE liest den Code schon beim Tippen. Was sie nicht versteht, unterstreicht sie, schreibt eine kurze Erklärung ans Zeilenende und listet es unten unter Probleme auf. Ein vertippter Befehl sieht so aus:"),

  H.figure("kap00_problem.png", "CIRLCE statt CIRCLE: Die Zeile ist markiert, die Erklärung steht daneben und in der Liste unten, mit einem Vorschlag, was gemeint war.", 480, 300),

  H.p("Die Meldung endet mit „Meintest du CIRCLE?“. Das ist der Normalfall: Drachenhauch sucht zu jedem Namen, den es nicht kennt, den ähnlichsten, den es kennt. Wer die Meldungen liest, bevor er F5 drückt, spart sich die meisten Fehlstarts."),

  H.h2("Wie man dieses Buch liest"),

  H.bulletRich("Tippen statt kopieren. ", "Jedes Programm liegt als Datei bei, aber wer es abtippt, liest jede Zeile — und die Tippfehler unterwegs führen genau zu den Meldungen, die man kennen sollte."),
  H.bulletRich("Erst vorhersagen, dann starten. ", "Vor jedem Start eine Vermutung: Was wird passieren? Wer richtig lag, hat verstanden. Wer danebenlag, hat gerade etwas gelernt."),
  H.bulletRich("Zahlen drehen. ", "Jedes Programm hat Zahlen, die man ändern kann, ohne etwas kaputtzumachen. Die Übungen fangen damit an, aber niemand muss auf sie warten."),

  H.h2("Das erste Wunder"),

  H.p("Zwölf Kreise in den Farben des Regenbogens, die sich im Kreis drehen und dabei atmen — größer werden und kleiner, einer nach dem anderen, wie eine Welle, die um den Ring läuft."),

  H.figure("kap00_1_farbkreis.png", "Das erste Wunder: zwölf Kreise, die sich drehen. Die Welle der Größen wandert mit.", 440, 275),

  H.code(ohneKopf("kap00/1_farbkreis.dh")),

  H.p("Siebzehn Zeilen. Was sie tun, von oben nach unten:"),

  H.bulletRich("SCREEN ", "öffnet ein Fenster, 800 Punkte breit und 500 hoch, mit dem Titel Farbkreis. Der Punkt (0, 0) ist die linke obere Ecke; x wächst nach rechts, y nach unten."),
  H.bulletRich("drehung ", "ist der Winkel, um den der ganze Ring gedreht ist, in Grad. Er wächst in jeder Sekunde um 40 — gleich mehr dazu."),
  H.bulletRich("Die Schleife ", "läuft, bis jemand das Fenster schließt oder Esc drückt. Jeder Durchgang ist ein Bild: CLS löscht, die Kreise werden gemalt, FLIP zeigt das fertige Bild."),
  H.bulletRich("Die FOR-Schleife ", "malt die zwölf Kreise. Kreis i sitzt i · 30 Grad weiter als der erste, zwölf mal dreißig sind die vollen 360."),
  H.bulletRich("COS und SIN ", "machen aus einem Winkel einen Punkt auf einem Kreis: 400 + COS(w) · 160 ist die x-Stelle, 250 + SIN(w) · 160 die y-Stelle eines Punkts, der 160 Punkte von der Mitte (400, 250) entfernt ist. RAD rechnet die Grad in das Maß um, das SIN und COS erwarten."),
  H.bulletRich("Der Radius ", "pendelt mit SIN zwischen 18 und 42. Weil jeder Kreis einen anderen Winkel in den SIN steckt, schwingen sie nacheinander — die Welle."),
  H.bulletRich("HSV ", "macht die Farbe: Farbton i · 30 Grad auf dem Farbkreis, kräftig (0,8) und hell (1,0). Zwölf Kreise, zwölf Farben, einmal rundherum."),

  H.h2("Warum so?"),

  H.h3("Drehen nach der Uhr, nicht nach den Bildern"),

  H.pmix(["Die wichtigste Zeile ist die unscheinbarste: ", ["drehung = drehung + 40 * DELTA()", true], ". ", ["DELTA()", true], " liefert, wie viele Sekunden seit dem letzten Bild vergangen sind — bei 60 Bildern je Sekunde etwa 0,017. Mal 40 ergibt das 0,67 Grad je Bild, 40 Grad je Sekunde. Die naheliegende Alternative ist, in jedem Bild ein festes Stück weiterzudrehen:"]),

  H.code(stueck("kap00/2_jedes_bild.dh", "drehung = drehung + 0.67")),

  H.p("Auf einem Bildschirm mit 60 Bildern je Sekunde sieht das genauso aus. Auf einem mit 144 dreht sich der Ring mehr als doppelt so schnell, auf einem alten Laptop, der nur 30 schafft, halb so schnell. Und wenn der Rechner kurz beschäftigt ist, ruckt der Ring nicht weiter, sondern bleibt stehen. Mit DELTA() sind die Zahlen im Programm Geschwindigkeiten — Grad je Sekunde, Punkte je Sekunde —, und das Programm läuft überall gleich schnell. Dieses Buch rechnet darum immer so."),

  H.h3("Grad im Programm, Bogenmaß für SIN"),

  H.p("SIN und COS rechnen im Bogenmaß: Ein voller Kreis ist 2 · π ≈ 6,28. Für Menschen sind Grad handlicher — dreißig Grad zwischen zwei Kreisen versteht man, 0,5236 nicht. Darum rechnet das Programm in Grad und wandelt erst im letzten Moment mit RAD um. Wer lieber im Bogenmaß bleibt, schreibt PI / 6 statt RAD(30); das Ergebnis ist dasselbe."),

  H.h2("Was schiefgehen kann"),

  H.h3("Ein vertippter Befehl"),

  H.p("Das Bild oben hat ihn schon gezeigt. Startet man das Programm trotz der Warnung, kommt der Fehler, sobald die Zeile drankommt:"),

  H.code(zeilen("kap00/fehler_tippfehler.dh")),
  H.code(["Laufzeitfehler in fehler_tippfehler.dh:4: Unbekannter Befehl",
          "'CIRLCE' -- dhrt kennt ihn nicht (Tippfehler?) Meintest du CIRCLE?"], { out: true }),

  H.p("Das Programm öffnet sein Fenster und bricht beim ersten Bild ab, weil Zeile 4 im ersten Durchgang der Schleife drankommt. Die Zeilennummer in der Meldung führt hin; in der IDE genügt ein Klick auf die Meldung."),

  H.h3("Das Programm lässt sich nicht schließen"),

  H.pmix(["Wer die Bedingung der Schleife weglässt und ", ["WHILE TRUE", true], " schreibt, bekommt ein Fenster, das auf das Kreuz nicht reagiert. ", ["QUITREQUESTED()", true], " ist der Weg, auf dem das Fenster dem Programm sagt, dass es zugemacht werden soll — fragt niemand danach, läuft es weiter. In der IDE hält Umschalt+F5 jedes Programm an."]),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Ändere die 40 in -40. Was passiert? Und mit 400?"),
  H.bulletRich("Drehen 2. ", "Ändere in HSV die i * 30 in i * 15. Welche Farben fehlen jetzt, und warum?"),
  H.bulletRich("Drehen 3. ", "Mache aus den zwölf Kreisen vierundzwanzig. Was musst du außer der 11 noch ändern, damit der Ring wieder gleichmäßig und einmal rundherum bunt ist? Und was passiert, wenn du eine der Stellen vergisst?"),
  H.bulletRich("Erweitern 1. ", "Ein zweiter, kleinerer Ring innen, der sich andersherum und doppelt so schnell dreht."),
  H.bulletRich("Erweitern 2. ", "Setze vor die Schleife SETFPS(15) — höchstens fünfzehn Bilder je Sekunde. Starte damit beide Fassungen, die mit DELTA() und die aus „Warum so?“. Was beobachtest du?"),
  H.bulletRich("Erfinden. ", "Ein eigenes Muster aus Kreisen: Spirale statt Ring, Kreise, die vom Rand in die Mitte wandern, ein Ring, der seine Farbe wechselt. Erfinden-Übungen haben keine Musterlösung."),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, IDE: ", "alle Menüs, Tastenkürzel und Leisten der Entwicklungsumgebung."),
  H.bulletRich("Handbuch, Grafik-Befehle: ", "SCREEN, CLS, CIRCLE, FLIP und alles andere, was zeichnet."),
  H.bulletRich("Handbuch, Kern-Builtins → Farb-Helfer: ", "RGB, HSV und ihre Verwandten."),
];
