module.exports = (H) => [
  H.part("Teil I — Bilder aus Zahlen"),
  H.chapter("Dein erstes Fenster"),

  H.p("In den nächsten zehn Minuten öffnest du ein Fenster und malst eine Sonne hinein. Dafür brauchst du fünf Zeilen. Fangen wir an."),

  H.h2("Drachenhauch einrichten"),

  H.p("Wenn du das Installationsprogramm noch nicht ausgeführt hast, hol das zuerst nach. Es legt Drachenhauch auf deinen Rechner und richtet alles ein, was du zum Loslegen brauchst. Danach hast du zwei Dinge zur Verfügung: einen Editor, in dem du Programme schreibst, und die Laufzeit, die sie ausführt."),

  H.pmix(["Den Editor öffnest du, indem du ", ["dh", true], " eingibst. Er sieht aus wie ein einfacher Texteditor — und das ist er auch, mit dem Unterschied, dass er deine Programme farbig darstellt und auf Knopfdruck startet."]),

  H.bulletRich("Neues Programm: ", "eine leere Datei anlegen und unter einem Namen speichern, der auf .dh endet — zum Beispiel sonne.dh."),
  H.bulletRich("Starten: ", "F5 drücken."),
  H.bulletRich("Speichern: ", "Strg+S, und zwar oft."),

  H.p("Du kannst ein Programm auch ohne Editor starten. Öffne dazu eine Eingabeaufforderung im Ordner deiner Datei und tippe:"),
  H.code(["dhrun.py sonne.dh"]),

  H.tip("Wo soll das alles liegen?", "Leg dir einen Ordner an, in dem deine Programme wohnen — etwa Dokumente/Drachenhauch. Für jedes Kapitel dieses Buchs darin einen Unterordner. Das klingt pedantisch, aber ab Kapitel 15 wirst du dankbar sein."),

  H.h2("Das allererste Programm"),

  H.p("Tippe das hier ab, Zeile für Zeile, und starte es mit F5:"),

  H.code([
    'SCREEN(640, 400, "Mein erstes Fenster")',
    "CLS(RGB(15, 20, 40))",
    "CIRCLE(320, 200, 90, RGB(255, 200, 40))",
    "FLIP()",
    "SLEEP(3000)",
  ]),

  H.p("Ein Fenster geht auf, dunkelblau, mit einer gelben Scheibe in der Mitte. Nach drei Sekunden schließt es sich von selbst."),

  H.figure("kap01_1_erstes_fenster.png", "Fünf Zeilen — und dein erstes eigenes Bild.", 440, 280),

  H.p("Falls stattdessen eine Fehlermeldung kommt: nicht erschrecken. Lies sie durch, sie nennt dir die Zeile. Meist fehlt ein Anführungszeichen oder eine Klammer. Vergleiche deine Zeile Zeichen für Zeichen mit der im Buch."),

  H.h2("Was du da geschrieben hast"),

  H.p("Fünf Zeilen, fünf Befehle. Ein Befehl ist eine Anweisung an den Rechner: tu dieses. Was er tun soll, steht in den Klammern dahinter."),

  H.pmix(["Die erste Zeile öffnet das Fenster. ", ["SCREEN", true], " bekommt drei Angaben mit: wie breit das Fenster sein soll (640 Punkte), wie hoch (400 Punkte) und was oben in der Titelleiste stehen soll. Der Text steht in Anführungszeichen, damit klar ist, wo er anfängt und aufhört."]),

  H.pmix([["CLS", true], " heißt „clear screen“ und streicht das ganze Fenster mit einer Farbe zu. Ohne diese Zeile wäre der Hintergrund schwarz."]),

  H.pmix([["CIRCLE", true], " malt einen ausgefüllten Kreis: an die Stelle 320 von links, 200 von oben, mit einem Radius von 90 Punkten, in Gelb."]),

  H.pmix([["FLIP", true], " ist der Befehl, der das Gemalte tatsächlich anzeigt. Das ist gewöhnungsbedürftig: Drachenhauch malt zunächst auf einer unsichtbaren Fläche, und erst ", ["FLIP", true], " klappt sie nach vorn. Der Grund dafür ist gut — er verhindert später das Flackern in bewegten Bildern —, aber vorerst reicht: ", ["FLIP", true], " nicht vergessen, sonst bleibt das Fenster leer."]),

  H.pmix([["SLEEP", true], " wartet. Die Zahl sind Millisekunden, 3000 davon sind drei Sekunden. Ohne diese Zeile würde das Programm sofort nach dem Malen enden und das Fenster wieder schließen — zu schnell, um etwas zu sehen."]),

  H.note("Die leeren Klammern hinter FLIP sind kein Versehen. Ein Befehl bekommt seine Angaben immer in Klammern, und wenn er keine braucht, bleiben sie eben leer. Weglassen darf man sie nicht."),

  H.h2("Farben sind drei Zahlen"),

  H.pmix(["Jede Farbe entsteht aus drei Zahlen: wie viel Rot, wie viel Grün, wie viel Blau. Genau dafür steht ", ["RGB", true], ". Jede der drei Zahlen liegt zwischen 0 (gar nichts) und 255 (voll aufgedreht)."]),

  H.p("Das ist anfangs ungewohnt, weil es nicht so mischt wie Farbe im Tuschkasten. Hier mischt sich Licht: Rot und Grün zusammen ergeben Gelb, alle drei zusammen ergeben Weiß, und alle drei auf 0 ergeben Schwarz."),

  H.table([
    ["RGB(255, 0, 0)", "Rot"],
    ["RGB(0, 200, 0)", "Grün"],
    ["RGB(0, 0, 255)", "Blau"],
    ["RGB(255, 200, 0)", "Gelb-Orange"],
    ["RGB(255, 255, 255)", "Weiß"],
    ["RGB(0, 0, 0)", "Schwarz"],
    ["RGB(128, 128, 128)", "mittleres Grau"],
    ["RGB(15, 20, 40)", "sehr dunkles Blau — unser Hintergrund"],
  ], { headers: ["Angabe", "Ergebnis"], widths: [3200, 5826], mono: [0] }),

  H.p("Probier es aus. Drei Kreise untereinander, drei Farben:"),

  H.code([
    'SCREEN(640, 400, "Ampel")',
    "CLS(RGB(30, 30, 30))",
    "CIRCLE(320, 110, 50, RGB(255, 0, 0))",
    "CIRCLE(320, 210, 50, RGB(255, 200, 0))",
    "CIRCLE(320, 310, 50, RGB(0, 200, 0))",
    "FLIP()",
    "SLEEP(3000)",
  ]),

  H.figure("kap01_2_ampel.png", "Dreimal derselbe Befehl — andere Zahlen, anderes Bild.", 440, 280),

  H.p("Beachte, dass dreimal genau derselbe Befehl dasteht. Nur die Zahlen unterscheiden sich. Das ist keine Kleinigkeit, sondern die halbe Wahrheit über das Programmieren: Man beschreibt, was getan werden soll, und die Zahlen bestimmen, wie es aussieht."),

  H.h2("Wo im Fenster ist wo?"),

  H.p("Jeder Punkt im Fenster hat zwei Zahlen: wie weit rechts er liegt und wie weit unten. Die erste heißt x, die zweite y. Der Punkt (0, 0) ist die linke obere Ecke. Bei einem Fenster von 640 mal 400 ist (639, 399) die rechte untere."),

  H.warn("Die y-Achse zeigt nach UNTEN. Größeres y heißt weiter unten im Fenster, nicht weiter oben. Das ist bei Bildschirmen so üblich und widerspricht allem, was du im Matheunterricht gelernt hast. Es ist die häufigste Quelle für Bilder, die auf dem Kopf stehen — auch bei Leuten, die das seit Jahren machen.", "Die Kopfstand-Falle"),

  H.pmix(["Ein Beispiel dafür, wofür das gut ist: ein Gesicht. Kopf, zwei Augen, ein Mund. Der Mund ist eine Reihe einzelner Punkte auf einer Kurve — hundertzwanzig Stück, gesetzt mit ", ["PLOT", true], ":"]),

  H.code([
    'SCREEN(640, 400, "Hallo!")',
    "CLS(RGB(30, 60, 120))",
    "",
    "CIRCLE(320, 200, 130, RGB(255, 210, 60))   ' Kopf",
    "CIRCLE(275, 165, 18, RGB(40, 40, 40))      ' linkes Auge",
    "CIRCLE(365, 165, 18, RGB(40, 40, 40))      ' rechtes Auge",
    "",
    "' Der Mund: viele kurze Striche auf einem Bogen",
    "DIM i AS INTEGER",
    "FOR i = -60 TO 60",
    "    PLOT(320 + i, 290 - (i * i) / 90, RGB(40, 40, 40))",
    "    PLOT(320 + i, 291 - (i * i) / 90, RGB(40, 40, 40))",
    "NEXT",
    "",
    "FLIP()",
    "SLEEP(4000)",
  ]),

  H.figure("kap01_3_smiley.png", "Zwei Kreise, ein Bogen — und es guckt zurück.", 440, 280),

  H.p("Hier stehen drei Dinge, die du noch nicht kennst, und du darfst sie getrost überblättern — sie sind die Themen der nächsten Kapitel. Der Vollständigkeit halber:"),

  H.bulletRich("Das Hochkomma ", "leitet einen Kommentar ein. Alles dahinter ist Notiz für Menschen; Drachenhauch überliest es."),
  H.bulletRich("DIM i AS INTEGER ", "kündigt eine Zahl namens i an. Darum geht es in Kapitel 3."),
  H.bulletRich("FOR ... NEXT ", "wiederholt die Zeilen dazwischen — hier 121 Mal, mit i von -60 bis 60. Das ist Kapitel 4."),

  H.p("Der Mund lohnt einen zweiten Blick. Der Bogen entsteht durch i mal i: in der Mitte, bei i gleich null, ist das Ergebnis null, und der Punkt liegt bei y gleich 290 — also weit unten. An den Rändern, bei i gleich 60, sind es 3600 geteilt durch 90, also 40; der Punkt rutscht auf y gleich 250 und damit nach oben. Deshalb lächelt es. Dreh das Minuszeichen in ein Plus um, und aus dem Lächeln wird ein Trauerkloß. Genau diesen Fehler habe ich beim Schreiben dieses Kapitels gemacht — und ihn erst gesehen, als das Bild fertig war."),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    ["Das Fenster bleibt schwarz", "FLIP vergessen — oder es steht vor den Malbefehlen statt danach."],
    ["Das Fenster blitzt nur kurz auf", "SLEEP am Ende vergessen. Das Programm ist fertig und schließt sich."],
    [{ text: "Parse-Fehler (1): Erwartet Rparen", mono: true }, "Eine schließende Klammer fehlt. Die Zahl in der Klammer ist die Spalte. Rparen ist Fachjargon für „right paren“, also die runde Klammer zu."],
    [{ text: "Lexer-Fehler (7): Zeilenumbruch im String nicht erlaubt", mono: true }, "Ein Anführungszeichen fehlt. Der Text läuft bis zum Zeilenende weiter, und dort ist Schluss."],
    [{ text: "Unbekanntes Builtin 'CIRLCE'", mono: true }, "Tippfehler im Befehlsnamen. Groß- und Kleinschreibung ist egal, die Buchstaben nicht."],
    ["Es passiert gar nichts", "Datei gespeichert? Der Editor startet, was auf der Platte steht, nicht was auf dem Schirm steht."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3800, 5226] }),

  H.p("Die Meldungen sehen sperrig aus, und die englischen Brocken darin machen es nicht besser. Aber sie sind verlässlich: Sie nennen immer die Zeile, und sie lügen nie. Lies sie von vorne — Dateiname, Doppelpunkt, Zeilennummer, dann die Sache selbst."),

  H.h2("Aufgaben"),

  H.bullet("Ändere die Farbe der Sonne. Mach sie rot, dann weiß, dann gib allen drei Zahlen den Wert 0 — und erkläre dir, warum du dann nichts mehr siehst."),
  H.bullet("Verschiebe die Sonne in die linke obere Ecke des Fensters, ohne die Fenstergröße zu ändern."),
  H.bullet("Male einen zweiten, kleineren Kreis in die Sonne hinein, so dass sie aussieht wie ein Spiegelei."),
  H.bullet("Gib dem Smiley eine Nase."),
  H.bullet("Mach aus der Ampel eine, bei der nur das rote Licht leuchtet und die anderen beiden dunkelgrau sind."),

  H.p("Im nächsten Kapitel bringen wir Bewegung hinein — und dafür brauchen wir etwas, das sich Zahlen merken kann."),
];
