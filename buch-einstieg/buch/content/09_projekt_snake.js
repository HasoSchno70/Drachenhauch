module.exports = (H) => [
  H.chapter("Projekt: Snake"),

  H.p("Snake ist noch älter als Pong — die Grundidee stammt aus einem Spielhallenautomaten von 1976, und berühmt wurde sie zwanzig Jahre später auf einem Nokia-Handy. Eine Schlange kriecht über ein Feld, frisst, wird länger, und irgendwann ist sie sich selbst im Weg."),

  H.p("Und genau daran scheitert alles, was du bisher kannst. Denn diese Schlange hat nicht eine Position, sondern zwanzig. Oder fünfzig. Man weiß vorher nicht, wie viele."),

  H.h2("Das Problem mit den vielen Kartons"),

  H.pmix(["Ein Glied der Schlange bräuchte zwei Kartons — Spalte und Zeile. Bei sechs Gliedern wären das zwölf: ", ["sx1", true], ", ", ["sy1", true], ", ", ["sx2", true], ", ", ["sy2", true], " und so weiter. Und wenn die Schlange frisst und länger wird, bräuchtest du plötzlich zwei weitere. Die kann man aber nicht während des Laufens dazuschreiben."]),

  H.p("Dieses Problem ist so alt wie das Programmieren, und die Lösung ist eine der ersten Erfindungen, die jede Sprache mitbringt: ein Behälter, der viele gleichartige Werte hält, durchnummeriert."),

  H.h2("Der Behälter"),

  H.code([
    "DIM punkte[5] AS INTEGER",
    "",
    "punkte[0] = 42",
    "punkte[1] = 17",
    "punkte[4] = 99",
    "",
    "PRINT punkte[0]",
    "PRINT punkte[4]",
  ]),

  H.code(["42", "99"], { out: true }),

  H.pmix([["DIM punkte[5] AS INTEGER", true], " legt fünf Kartons auf einmal an — nicht einen, sondern eine ganze Reihe. So etwas heißt ein Array, auf Deutsch manchmal Feld. Ein deutsches Wort hat sich nie durchgesetzt; sag ruhig Array."]),

  H.pmix(["Die eckigen Klammern sind neu und wichtig: Runde Klammern gehören zu Befehlen, eckige zu Arrays. ", ["punkte[1]", true], " heißt „das Fach Nummer 1“."]),

  H.warn("Gezählt wird ab NULL. Bei DIM punkte[5] heißen die Fächer 0, 1, 2, 3 und 4 — ein Fach Nummer 5 gibt es nicht. Das ist die berühmteste Stolperstelle der ganzen Programmiererei, und sie erwischt auch Leute, die das seit Jahren machen. Merksatz: Die Zahl in der DIM-Zeile ist die ANZAHL, die Zahl in den Klammern ist der ABSTAND vom Anfang.", "Das erste Fach heißt null"),

  H.p("Der eigentliche Gewinn zeigt sich erst mit einer Schleife. Denn die Nummer in den Klammern darf eine Variable sein — und damit kann eine einzige Zeile alle Fächer der Reihe nach anfassen:"),

  H.code([
    "DIM zahlen[10] AS INTEGER",
    "DIM i AS INTEGER",
    "",
    "FOR i = 0 TO 9",
    "    zahlen[i] = i * i",
    "NEXT",
    "",
    "PRINT zahlen[7]",
  ]),

  H.code(["49"], { out: true }),

  H.pmix([["LEN(zahlen)", true], " sagt dir jederzeit, wie viele Fächer ein Array hat — hier 10. Praktisch, wenn die Länge nicht direkt daneben steht."]),

  H.h2("Wie man daraus eine Schlange macht"),

  H.p("Für Snake brauchen wir zwei Arrays: eines für die Spalten aller Glieder, eines für die Zeilen. Fach 0 ist immer der Kopf, danach folgt der Körper."),

  H.code([
    "DIM sx[400] AS INTEGER          ' Spalte jedes Gliedes",
    "DIM sy[400] AS INTEGER          ' Zeile jedes Gliedes",
    "DIM laenge AS INTEGER",
  ]),

  H.p("Vierhundert Fächer sind großzügig — das Feld hat 32 mal 19, also 608 Zellen, und so lang wird keine Schlange. Man legt lieber zu viele an als zu wenige; leere Fächer kosten nichts."),

  H.pmix(["Wie lang die Schlange gerade WIRKLICH ist, steht in ", ["laenge", true], ". Die Fächer dahinter enthalten noch alte Werte, aber sie werden weder gemalt noch geprüft. Ein Array plus ein Längenzähler — dieses Paar wirst du überall wiedersehen."]),

  H.h2("Die Bewegung: nachrücken"),

  H.p("Jetzt kommt die Idee, um die sich das ganze Spiel dreht, und sie ist bestechend einfach. Eine Schlange bewegt sich nicht wirklich. Jedes Glied rückt auf den Platz seines Vordermanns, und nur der Kopf geht wirklich einen Schritt weiter:"),

  H.code([
    "FOR i = laenge - 1 TO 1 STEP -1",
    "    sx[i] = sx[i - 1]",
    "    sy[i] = sy[i - 1]",
    "NEXT",
    "sx[0] = sx[0] + dx",
    "sy[0] = sy[0] + dy",
  ]),

  H.p("Das ist alles. Fünf Zeilen für eine kriechende Schlange beliebiger Länge."),

  H.warn("Die Schleife läuft RÜCKWÄRTS, und das ist zwingend. Sie kopiert jedes Fach aus dem davor. Liefe sie vorwärts, würde Fach 1 den Kopf übernehmen — und Fach 2 dann den bereits überschriebenen Wert aus Fach 1, also wieder den Kopf. Nach einem Schritt stünden alle Glieder aufeinander. Dreh die Schleife testweise um und sieh es dir an; es ist eine der lehrreichsten Minuten dieses Buchs.", "Von hinten nach vorn"),

  H.h2("Das Feld"),

  H.p("Snake spielt auf einem Gitter, nicht auf freier Fläche. Die Schlange steht nie zwischen zwei Feldern. Deshalb rechnen wir in Spalten und Zeilen statt in Punkten — und rechnen erst beim Malen um:"),

  H.code([
    "px = sx[i] * 20",
    "py = sy[i] * 20",
    "BOX(px + 2, py + 2, px + 17, py + 17, RGB(80, 190, 110))",
  ]),

  H.pmix(["Jede Zelle ist zwanzig Punkte groß. Spalte 3 beginnt also bei 60, Spalte 4 bei 80. Die Umrechnung wandert erst in zwei Kartons — das spart nicht nur Tipparbeit, es macht die Malzeile überhaupt erst lesbar. Das ", ["+ 2", true], " und ", ["+ 17", true], " lassen ringsum zwei Punkte Luft, dadurch sieht man die einzelnen Glieder als Kästchen und nicht als durchgehenden Balken."]),

  H.p("Bei 32 Spalten und 19 Zeilen füllt das Gitter 640 mal 380 Punkte, und unten bleiben zwanzig für die Anzeige der Länge."),

  H.h2("Fressen, sterben, neu anfangen"),

  H.p("Drei kurze Regeln machen aus dem Kriechen ein Spiel."),

  H.p("Erstens: Wenn der Kopf auf dem Futter steht, wächst die Schlange um ein Glied, und das Futter erscheint woanders."),

  H.code([
    "IF sx[0] = fx AND sy[0] = fy THEN",
    "    laenge = laenge + 1",
    "    fx = RANDINT(0, 31)",
    "    fy = RANDINT(0, 18)",
    "END IF",
  ]),

  H.p("Bemerkenswert daran: Es wird kein neues Glied irgendwo angesetzt. Die Länge wird schlicht um eins größer, und beim nächsten Nachrücken zieht das letzte Glied einfach nicht mehr nach. Die Schlange wächst hinten, ohne dass es dafür eine Zeile gäbe."),

  H.p("Zweitens: Verlässt der Kopf das Feld, ist Schluss."),

  H.code([
    "IF sx[0] < 0 OR sx[0] > 31 THEN tot = TRUE",
    "IF sy[0] < 0 OR sy[0] > 18 THEN tot = TRUE",
  ]),

  H.p("Drittens — und das ist der eigentliche Reiz des Spiels: Beißt sich die Schlange selbst, ist ebenfalls Schluss. Dafür wird der Kopf mit jedem einzelnen Körperglied verglichen:"),

  H.code([
    "FOR i = 1 TO laenge - 1",
    "    IF sx[i] = sx[0] AND sy[i] = sy[0] THEN tot = TRUE",
    "NEXT",
  ]),

  H.pmix(["Die Schleife beginnt bei ", ["1", true], ", nicht bei 0. Fach 0 ist der Kopf selbst, und der steht naturgemäß immer auf seiner eigenen Stelle — begänne die Prüfung bei null, wäre die Schlange sofort tot."]),

  H.p("Auch das ist ein Muster, das immer wiederkommt: eine Sache mit allen anderen vergleichen. Was hier fünf Zeilen sind, heißt in großen Programmen Kollisionsprüfung und funktioniert im Kern genauso."),

  H.h2("Ein Takt für die Schlange"),

  H.p("Ein Problem bleibt. Die Schleife läuft sechzigmal je Sekunde — eine Schlange, die sechzigmal je Sekunde ein Feld weiterrückt, ist nicht spielbar. Sie braucht einen eigenen, langsameren Takt:"),

  H.code([
    "takt = takt + 1",
    "IF takt >= 8 THEN",
    "    takt = 0",
    "END IF",
  ]),

  H.p("Ein Zähler läuft mit, und nur bei jedem achten Bild passiert etwas. Das ergibt siebeneinhalb Schritte je Sekunde — schnell genug, dass es lebt, langsam genug, dass man reagieren kann. Wer das Spiel schwerer machen will, verringert diese Zahl, während die Schlange wächst."),

  H.note("Gemalt wird trotzdem in jedem Bild. Nur das Nachrücken hängt am Takt. Diese Trennung — zeichnen immer, rechnen nach eigenem Rhythmus — ist in Spielen die Regel und nicht die Ausnahme."),

  H.h2("Nicht rückwärts abbiegen"),

  H.p("Eine Kleinigkeit noch, ohne die das Spiel kaputt wäre. Läuft die Schlange nach rechts und du drückst links, würde der Kopf in ihr zweites Glied fahren — sofortiger Tod, ohne dass jemand einen Fehler gemacht hätte. Also:"),

  H.code([
    "IF KEYPRESSED(KEY_LEFT) AND dx = 0 THEN",
    "    dx = -1",
    "    dy = 0",
    "END IF",
  ]),

  H.pmix(["Der Zusatz ", ["AND dx = 0", true], " bedeutet: nach links abbiegen darf nur, wer sich gerade NICHT waagerecht bewegt. Wer nach rechts läuft, hat ein ", ["dx", true], " von 1 und wird abgewiesen. Vier Zeilen dieser Art, für jede Richtung eine."]),

  H.h2("Das fertige Spiel"),

  H.figure("kap09_snake.png", "Sechs Glieder, ein hellerer Kopf, das Futter in Rot. Das angedeutete Gitter zeigt, dass das Feld aus Zellen besteht.", 440, 280),

  H.code([
    "' Snake. Pfeiltasten steuern, R startet neu, ESC beendet.",
    "",
    'SCREEN(640, 400, "Snake")',
    "",
    "DIM sx[400] AS INTEGER          ' Spalte jedes Gliedes",
    "DIM sy[400] AS INTEGER          ' Zeile jedes Gliedes",
    "DIM laenge AS INTEGER",
    "DIM dx AS INTEGER",
    "DIM dy AS INTEGER",
    "DIM fx AS INTEGER",
    "DIM fy AS INTEGER",
    "DIM takt AS INTEGER",
    "DIM tot AS BOOLEAN",
    "DIM i AS INTEGER",
    "DIM neu AS BOOLEAN",
    "DIM g AS INTEGER",
    "DIM px AS INTEGER",
    "DIM py AS INTEGER",
    "DIM warnfarbe AS INTEGER",
    "",
    "neu = TRUE",
    "warnfarbe = RGB(255, 220, 120)",
    "",
    "WHILE NOT QUITREQUESTED() AND NOT KEYPRESSED(KEY_ESCAPE)",
    "",
    "    IF neu THEN",
    "        neu = FALSE",
    "        tot = FALSE",
    "        laenge = 6",
    "        dx = 1",
    "        dy = 0",
    "        takt = 0",
    "        FOR i = 0 TO laenge - 1",
    "            sx[i] = 10 - i",
    "            sy[i] = 10",
    "        NEXT",
    "        fx = RANDINT(0, 31)",
    "        fy = RANDINT(0, 18)",
    "    END IF",
    "",
    "    CLS(RGB(14, 20, 16))",
    "",
    "    FOR g = 0 TO 31",
    "        LINE(g * 20, 0, g * 20, 379, RGB(22, 32, 26))",
    "    NEXT",
    "    FOR g = 0 TO 18",
    "        LINE(0, g * 20, 639, g * 20, RGB(22, 32, 26))",
    "    NEXT",
    "    BOX(0, 380, 639, 399, RGB(20, 28, 23))",
    "",
    "    px = fx * 20",
    "    py = fy * 20",
    "    BOX(px + 3, py + 3, px + 16, py + 16, RGB(240, 90, 70))",
    "",
    "    FOR i = 0 TO laenge - 1",
    "        px = sx[i] * 20",
    "        py = sy[i] * 20",
    "        IF i = 0 THEN",
    "            BOX(px + 1, py + 1, px + 18, py + 18, RGB(180, 255, 140))",
    "        ELSE",
    "            BOX(px + 2, py + 2, px + 17, py + 17, RGB(80, 190, 110))",
    "        END IF",
    "    NEXT",
    "",
    '    TEXT(10, 383, "Laenge: " + STR$(laenge), RGB(140, 170, 150))',
    '    IF tot THEN TEXT(230, 180, "Vorbei -- R fuer neu", warnfarbe)',
    "    FLIP()",
    "",
    "    IF KEYPRESSED(KEY_LEFT) AND dx = 0 THEN",
    "        dx = -1",
    "        dy = 0",
    "    END IF",
    "    IF KEYPRESSED(KEY_RIGHT) AND dx = 0 THEN",
    "        dx = 1",
    "        dy = 0",
    "    END IF",
    "    IF KEYPRESSED(KEY_UP) AND dy = 0 THEN",
    "        dx = 0",
    "        dy = -1",
    "    END IF",
    "    IF KEYPRESSED(KEY_DOWN) AND dy = 0 THEN",
    "        dx = 0",
    "        dy = 1",
    "    END IF",
    "    IF KEYHIT(KEY_R) THEN neu = TRUE",
    "",
    "    IF NOT tot THEN",
    "        takt = takt + 1",
    "        IF takt >= 8 THEN",
    "            takt = 0",
    "",
    "            FOR i = laenge - 1 TO 1 STEP -1",
    "                sx[i] = sx[i - 1]",
    "                sy[i] = sy[i - 1]",
    "            NEXT",
    "            sx[0] = sx[0] + dx",
    "            sy[0] = sy[0] + dy",
    "",
    "            IF sx[0] < 0 OR sx[0] > 31 THEN tot = TRUE",
    "            IF sy[0] < 0 OR sy[0] > 18 THEN tot = TRUE",
    "",
    "            FOR i = 1 TO laenge - 1",
    "                IF sx[i] = sx[0] AND sy[i] = sy[0] THEN tot = TRUE",
    "            NEXT",
    "",
    "            IF sx[0] = fx AND sy[0] = fy THEN",
    "                laenge = laenge + 1",
    "                fx = RANDINT(0, 31)",
    "                fy = RANDINT(0, 18)",
    "            END IF",
    "        END IF",
    "    END IF",
    "WEND",
  ]),

  H.h2("Der Neustart in einem Schalter"),

  H.pmix(["Ein Kniff steckt noch darin, der leicht zu übersehen ist. Ganz oben in der Schleife steht ", ["IF neu THEN", true], " — ein Block, der die Schlange auf den Anfangszustand setzt. Er läuft beim allerersten Durchgang, weil ", ["neu", true], " vorher auf ", ["TRUE", true], " gesetzt wurde, und danach nur noch, wenn jemand R drückt."]),

  H.p("Damit steht die gesamte Anfangsausstattung an genau einer Stelle im Programm, statt einmal oben und ein zweites Mal beim Neustart. Es ist derselbe Gedanke wie beim Nachrücken: eine Sache, ein Ort."),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    ["Alle Glieder liegen aufeinander", "Die Nachrück-Schleife läuft vorwärts statt rückwärts. Siehe den Warnkasten."],
    ["Die Schlange ist sofort tot", "Die Selbstprüfung beginnt bei 0 statt bei 1 und findet den Kopf auf seiner eigenen Stelle."],
    [{ text: "Index ausserhalb", mono: true }, "Ein Fach jenseits der Anzahl. Bei DIM sx[400] geht es bis 399."],
    ["Die Schlange rast", "Der Takt fehlt: Ohne ihn rückt sie sechzigmal je Sekunde vor."],
    ["Das Futter erscheint unter der Schlange", "Kann passieren — es wird ja blind gewürfelt. Als Aufgabe unten steht, wie man das behebt."],
    ["Rückwärts abbiegen tötet sofort", "Die Prüfung AND dx = 0 fehlt in den Steuerzeilen."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3400, 5626] }),

  H.h2("Aufgaben"),

  H.bullet("Lass die Schlange schneller werden, je länger sie ist: Der Takt soll mit wachsender Länge kleiner werden, aber nie unter drei fallen."),
  H.bullet("Sorg dafür, dass das Futter nie auf der Schlange landet. Du musst dafür nach dem Würfeln alle Glieder durchsehen und notfalls neu würfeln."),
  H.bullet("Zeig die beste bisher erreichte Länge an. Sie muss den Neustart überleben, darf also nicht im neu-Block zurückgesetzt werden."),
  H.bullet("Bau Wände ein: ein paar feste Kästchen mitten im Feld, die ebenfalls tödlich sind. Ein drittes Arraypaar hält ihre Stellen."),
  H.bullet("Mach die Ränder durchlässig — wer rechts hinausläuft, kommt links wieder herein. Ein MOD aus Kapitel 5 genügt."),
  H.bullet("Gib jedem Glied eine eigene Farbe, die sich von Kopf zu Schwanz verläuft. Die Nummer des Gliedes ist alles, was du dafür brauchst."),

  H.p("Damit endet Teil I. Du hast zwei vollständige Spiele gebaut und kennst alles, womit man Bilder macht, sie bewegt und auf Menschen reagiert. Im nächsten Teil bekommt das Ganze Ton."),
];
