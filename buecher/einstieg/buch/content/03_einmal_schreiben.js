module.exports = (H) => [
  H.chapter("Einmal schreiben, tausendmal malen"),

  H.p("Bis hierher stand für jeden Kreis eine Zeile im Programm. Zwei Gesichter waren schon sechs Zeilen; für hundert Kreise wären es hundert. So arbeitet niemand."),

  H.p("Dieses Kapitel bringt dir den Befehl, mit dem aus einer Zeile fünfhundert werden. Er ist der wichtigste in diesem ganzen Buch, und du hast ihn schon einmal gesehen — im Mund des Smileys aus Kapitel 1."),

  H.h2("640 Linien aus vier Zeilen"),

  H.code([
    'SCREEN(640, 400, "Farbverlauf")',
    "",
    "DIM x AS INTEGER",
    "FOR x = 0 TO 639",
    "    LINE(x, 0, x, 399, RGB(x * 255 \\ 640, 40, 255 - x * 255 \\ 640))",
    "NEXT",
    "",
    "FLIP()",
    "SLEEP(3000)",
  ]),

  H.figure("kap03_1_farbverlauf.png", "Eine Malzeile. 640 senkrechte Striche, jeder in einer anderen Farbe.", 440, 280),

  H.p("Das Fenster ist 640 Punkte breit, und in ihm stehen 640 hauchdünne senkrechte Striche nebeneinander — jeder einen Hauch anders gefärbt als sein Nachbar. Zusammen ergeben sie einen weichen Übergang von Blau nach Rot. Geschrieben hast du dafür eine einzige Malzeile."),

  H.h2("Zeile für Zeile"),

  H.pmix([["DIM x AS INTEGER", true], " kennst du aus dem letzten Kapitel: ein Karton für eine ganze Zahl. Diesen hier füllt die Schleife selbst, du musst nichts hineinlegen."]),

  H.pmix([["FOR x = 0 TO 639", true], " ist der Anfang der Schleife und liest sich fast wie ein deutscher Satz: „für x von 0 bis 639“. Drachenhauch legt die 0 in den Karton x, führt alles bis zum ", ["NEXT", true], " aus, erhöht x um eins, führt alles noch einmal aus, und so weiter — bis x bei 639 angekommen ist. Danach geht es unterhalb des ", ["NEXT", true], " weiter."]),

  H.pmix([["NEXT", true], " markiert das Ende. Alles zwischen ", ["FOR", true], " und ", ["NEXT", true], " ist das, was wiederholt wird. Man rückt diese Zeilen um vier Leerzeichen ein — Drachenhauch ist das gleichgültig, deinen Augen nicht. Halte dich daran, von Anfang an."]),

  H.p("Die Malzeile in der Mitte lohnt es, in Teile zu zerlegen. Erst die Form — die Farbe lassen wir einen Moment beiseite:"),

  H.code(["LINE(x, 0, x, 399, RGB(255, 40, 40))"]),

  H.pmix([["LINE", true], " zieht eine Linie von einem Punkt zum anderen: erst wohin sie beginnt (x und y), dann wo sie endet. Hier beginnt sie oben bei ", ["y = 0", true], " und endet unten bei ", ["y = 399", true], " — beide Male an derselben Stelle x. Das ergibt einen senkrechten Strich über die volle Höhe."]),

  H.p("Und weil x bei jedem Durchgang eins weiterrückt, wandert der Strich Punkt für Punkt nach rechts, bis das Fenster voll ist."),

  H.code(["RGB(x * 255 \\ 640, 40, 255 - x * 255 \\ 640)"]),

  H.pmix(["Die Farbe rechnet mit derselben Zählvariablen. ", ["x * 255 \\ 640", true], " macht aus einem x zwischen 0 und 639 eine Zahl zwischen 0 und 255: ganz links kommt 0 heraus, ganz rechts 255. Genau das braucht ", ["RGB", true], "."]),

  H.pmix(["Diese Zahl ist der Rotanteil. Der Blauanteil ist ", ["255 -", true], " dieselbe Zahl, läuft also genau andersherum: links 255, rechts 0. Wo viel Rot ist, ist wenig Blau. Der Grünanteil bleibt fest bei 40 und gibt dem Ganzen einen dunklen Grundton."]),

  H.warn("Hier steht der Rückwärts-Schrägstrich, nicht der normale. RGB käme inzwischen auch mit dem normalen zurecht — es rundet eine Kommazahl selbst. Der Rückwärts-Schrägstrich sagt aber, was gemeint ist: aus 43,7 wird 43, abgeschnitten und nicht gerundet. Und er hat einen zweiten Grund: Schreib einmal n = 7 / 2, wenn n als INTEGER angesagt ist. Der Editor streicht die Zeile an („ist als INTEGER angesagt, rechts steht eine Kommazahl“), und wenn du es trotzdem laufen lässt, bricht das Programm ab. Diese Sorte Fehler ist mir beim Schreiben dieses Kapitels mehrfach passiert — damals ohne Vorwarnung. Verlass dich aber nicht blind darauf: die Prüfung erkennt die Fälle, die sie am Text sehen kann. Ob n = f * 2.0 gutgeht, entscheidet erst der Wert von f.", "Ganze Zahlen und Kommazahlen"),

  H.tip("Der wichtigste Versuch dieses Kapitels", "Ändere die 639 in eine 200 und starte neu. Ändere sie in 5. Dann schreib statt FOR x = 0 TO 639 einmal FOR x = 0 TO 639 STEP 20 — und du siehst auf einen Schlag, was die Schleife eigentlich tut, weil plötzlich Lücken zwischen den Strichen sind."),

  H.h2("Rückwärts zählen"),

  H.pmix(["Eine Schleife darf auch abwärts laufen. Dazu sagt man ihr mit ", ["STEP", true], ", um wie viel sie weiterrücken soll — und eine negative Schrittweite zählt eben rückwärts:"]),

  H.code([
    'SCREEN(640, 400, "Tunnel")',
    "CLS(RGB(0, 0, 0))",
    "",
    "DIM r AS INTEGER",
    "FOR r = 200 TO 4 STEP -6",
    "    CIRCLEOUTLINE(320, 200, r, RGB(255, 255 - r, 40))",
    "NEXT",
    "",
    "FLIP()",
    "SLEEP(3000)",
  ]),

  H.figure("kap03_2_tunnel.png", "33 Ringe. Man kann sie zählen — und genau so oft ist die Schleife gelaufen.", 440, 280),

  H.pmix([["FOR r = 200 TO 4 STEP -6", true], " beginnt bei 200 und zieht bei jedem Durchgang 6 ab: 200, 194, 188 und so fort, bis der Wert unter 4 fiele. Das sind 33 Durchgänge, und du kannst sie im Bild nachzählen."]),

  H.pmix([["CIRCLEOUTLINE", true], " malt nur den Rand eines Kreises statt der gefüllten Scheibe. Nimm hier versehentlich ", ["CIRCLE", true], ", und du bekommst keinen Tunnel, sondern einen weichen Farbklecks: Jede gefüllte Scheibe übermalt die vorige fast vollständig, sichtbar bleibt nur ein schmaler Ring. Auch das sieht hübsch aus — aber man sieht dem Bild dann nicht mehr an, dass es aus vielen Kreisen besteht."]),

  H.pmix(["Die Farbe hängt wieder am Zähler. Der Rotanteil steht fest auf 255, der Grünanteil ist ", ["255 - r", true], ": außen, bei großem r, bleibt wenig Grün übrig — das ergibt Rot. Innen, bei kleinem r, kommt viel Grün dazu, und Rot mit viel Grün ergibt Gelb. Der Tunnel glüht also nach innen."]),

  H.note("Warum steht die Reihenfolge groß nach klein und nicht umgekehrt? Weil später Gemaltes über früher Gemaltem liegt. Bei CIRCLEOUTLINE ist das gleichgültig, bei gefüllten Formen entscheidet es darüber, was man am Ende sieht. Merke: Der Bildschirm ist eine Leinwand, kein Stapel Folien."),

  H.h2("Eine Spirale — und ein Ausflug in die Trigonometrie"),

  H.p("Der nächste Schnipsel sieht aus wie Zauberei und ist zehn Zeilen lang. Er ist der erste Punkt in diesem Buch, an dem du etwas benutzt, das du noch nicht ganz durchschaust — und das ist völlig in Ordnung. Tipp ihn ab, sieh ihn dir an, und lies die Erklärung danach in Ruhe."),

  H.code([
    'SCREEN(640, 400, "Spirale")',
    "CLS(RGB(10, 10, 20))",
    "",
    "DIM i AS INTEGER",
    "DIM winkel AS FLOAT",
    "DIM weite AS FLOAT",
    "DIM sx AS FLOAT",
    "DIM sy AS FLOAT",
    "",
    "FOR i = 0 TO 720",
    "    winkel = RAD(i)",
    "    weite = i / 4",
    "    sx = 320 + COS(winkel) * weite",
    "    sy = 200 + SIN(winkel) * weite",
    "    CIRCLE(sx, sy, 3, RGB(255, i \\ 3, 60))",
    "NEXT",
    "",
    "FLIP()",
    "SLEEP(4000)",
  ]),

  H.figure("kap03_3_spirale.png", "Zwei Umdrehungen, 721 kleine Kreise — und keine einzige Koordinate von Hand ausgerechnet.", 440, 280),

  H.p("Stell dir eine Uhr vor. Der Zeiger steht auf zwölf und dreht sich langsam. Zwei Zahlen beschreiben, wo seine Spitze gerade ist: wie weit sie rechts von der Mitte liegt und wie weit über oder unter ihr. Genau diese beiden Zahlen liefern COS und SIN."),

  H.bulletRich("COS(winkel) ", "sagt, wie weit rechts — eine Zahl zwischen -1 und 1."),
  H.bulletRich("SIN(winkel) ", "sagt, wie weit unten — ebenfalls zwischen -1 und 1."),

  H.pmix(["Beide arbeiten mit einem Winkel, den sie in einem eigenen Maß erwarten, nicht in Grad. ", ["RAD(i)", true], " rechnet Grad in dieses Maß um; mehr musst du darüber vorerst nicht wissen. ", ["i", true], " läuft von 0 bis 720, also zweimal rundherum."]),

  H.pmix([["sx = 320 + COS(winkel) * weite", true], " heißt demnach: geh in die Mitte des Fensters und von dort so weit nach rechts, wie der Zeiger gerade zeigt — mal die Länge des Zeigers. Und ", ["weite = i / 4", true], " macht den Zeiger bei jedem Schritt ein winziges Stück länger. Ein Zeiger, der sich dreht und dabei wächst, malt eine Spirale."]),

  H.tip("Drei Zahlen zum Drehen", "Ändere die 720 in 2160 — drei Umdrehungen. Ändere weite = i / 4 in i / 12, und die Spirale wird eng. Schreib statt der 3 im CIRCLE eine 8, und aus der dünnen Linie wird ein dickes Band. Jede dieser Änderungen ist ein Zeichen und ändert das ganze Bild."),

  H.h2("Eine Schleife in einer Schleife"),

  H.p("Eine Schleife malt eine Reihe. Zwei ineinander malen ein Feld:"),

  H.code([
    'SCREEN(640, 400, "Gitter")',
    "CLS(RGB(20, 20, 30))",
    "",
    "DIM x AS INTEGER",
    "DIM y AS INTEGER",
    "",
    "FOR y = 0 TO 7",
    "    FOR x = 0 TO 9",
    "        CIRCLE(40 + x * 62, 30 + y * 48, 22, RGB(x * 25, y * 32, 200))",
    "    NEXT",
    "NEXT",
    "",
    "FLIP()",
    "SLEEP(4000)",
  ]),

  H.figure("kap03_4_gitter.png", "Zwei Schleifen, achtzig Kreise: von links nach rechts wird es röter, von oben nach unten grüner.", 440, 280),

  H.p("Die äußere Schleife zählt die Zeilen, die innere die Spalten. Wichtig ist die Reihenfolge: Für jeden einzelnen Wert von y läuft die innere Schleife einmal komplett durch. Bei y gleich 0 malt sie zehn Kreise nebeneinander, dann rückt y auf 1, und sie malt die nächsten zehn — eine Zeile tiefer. Zusammen sind das achtmal zehn, also achtzig Kreise."),

  H.pmix(["Die Stellen entstehen wieder durch Rechnen: ", ["40 + x * 62", true], " ist ein Rand von 40 Punkten plus 62 Punkte Abstand je Spalte. ", ["30 + y * 48", true], " macht dasselbe senkrecht."]),

  H.p("Und die Farbe verrät dir die Zähler: Der Rotanteil hängt an x, wächst also nach rechts; der Grünanteil hängt an y und wächst nach unten. Du siehst die beiden Schleifen im Bild."),

  H.warn("Jedes FOR braucht sein eigenes NEXT, und sie müssen richtig ineinander liegen — das innere NEXT gehört zum inneren FOR. Vertauschst du sie, beschwert sich Drachenhauch. Rück die Zeilen ordentlich ein, dann siehst du auf einen Blick, was zu wem gehört.", "Innen zuerst schließen"),

  H.h2("Was in einer Schleife gerne schiefgeht"),

  H.table([
    [{ text: "FLOAT … passt nicht verlustfrei in INTEGER", mono: true }, "Eine Rechnung mit / landet in einer Variablen, die als INTEGER angesagt ist. Für ganzzahlige Division \\ nehmen, sonst INT() oder ROUND() darum."],
    [{ text: "Erwartet NEXT", mono: true }, "Ein NEXT fehlt, oder es steht an der falschen Stelle."],
    ["Nichts erscheint, das Programm hängt", "Eine Schleife, die ihr Ziel nie erreicht — etwa FOR r = 200 TO 4 ohne das negative STEP. Sie beginnt bei 200, das ist schon größer als 4, also läuft sie kein einziges Mal."],
    ["Nur ein einziger Kreis ist da", "Alle Durchgänge malen an dieselbe Stelle: In der Malzeile fehlt die Zählvariable."],
    ["Das Bild ist eine gleichmäßige Fläche", "Die Schritte sind zu klein oder die Formen zu groß — sie übermalen einander. Nimm STEP größer oder die Formen kleiner."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3600, 5426] }),

  H.h2("Aufgaben"),

  H.bullet("Dreh den Farbverlauf um: links rot, rechts blau. Du musst dafür genau zwei Zeichen ändern."),
  H.bullet("Mach aus dem Farbverlauf einen waagerechten — von oben nach unten statt von links nach rechts."),
  H.bullet("Male eine Zielscheibe: abwechselnd dicke rote und weiße Ringe. Tipp: Nimm CIRCLE statt CIRCLEOUTLINE und lass die Schleife von außen nach innen laufen."),
  H.bullet("Lass die Spirale nicht in der Mitte, sondern in der linken oberen Ecke beginnen."),
  H.bullet("Bau das Gitter so um, dass die Kreise nach rechts hin immer größer werden. Der Radius muss dafür von x abhängen."),
  H.bullet("Male mit einer Schleife einen Fächer: zwanzig Linien, die alle im selben Punkt beginnen und im Kreis auseinanderlaufen. Du brauchst dafür SIN und COS aus dem Spiralen-Beispiel."),

  H.p("Bisher wusstest du bei jedem Programm vorher, wie das Bild aussehen wird. Im nächsten Kapitel nicht mehr — da kommt der Zufall dazu, und dein Sternenhimmel sieht bei jedem Start anders aus."),
];
