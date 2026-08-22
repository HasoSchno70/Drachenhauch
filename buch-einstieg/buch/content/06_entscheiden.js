module.exports = (H) => [
  H.chapter("Entscheiden"),

  H.p("Der Ball aus dem letzten Kapitel fällt aus dem Bild und kommt nicht wieder. Das lässt sich nicht durch schöneres Rechnen beheben — das Programm muss etwas Neues können: hinsehen und je nachdem anders handeln."),

  H.p("Dafür gibt es einen einzigen Befehl. Mit ihm hört dein Programm auf, ein Ablauf zu sein, und wird zu etwas, das reagiert."),

  H.h2("Der Ball, der abprallt"),

  H.code([
    'SCREEN(640, 400, "Abpraller")',
    "",
    "DIM y AS FLOAT",
    "DIM tempo AS FLOAT",
    "y = 40",
    "tempo = 0",
    "",
    "WHILE NOT QUITREQUESTED() AND NOT KEYPRESSED(KEY_ESCAPE)",
    "    CLS(RGB(15, 20, 40))",
    "    BOX(0, 380, 639, 399, RGB(60, 70, 100))",
    "    CIRCLE(320, y, 20, RGB(255, 90, 60))",
    "    FLIP()",
    "",
    "    tempo = tempo + 0.35",
    "    y = y + tempo",
    "",
    "    IF y > 360 THEN",
    "        y = 360",
    "        tempo = -tempo * 0.9",
    "    END IF",
    "WEND",
  ]),

  H.figure("kap06_1_abpraller.png", "Ein Standbild mitten im Sprung — der Ball ist unterwegs nach oben.", 440, 280),

  H.p("Der Ball fällt, trifft den Boden, springt zurück, wird oben langsamer, fällt wieder — und jeder Sprung ist etwas kleiner als der vorige. Nach einer Weile bleibt er liegen. Vier Zeilen machen das."),

  H.h2("Zeile für Zeile"),

  H.pmix([["IF y > 360 THEN", true], " liest sich wörtlich: „falls y größer als 360 ist, dann“. Alles, was danach bis zum ", ["END IF", true], " steht, wird nur ausgeführt, wenn das zutrifft. Trifft es nicht zu, überspringt Drachenhauch diese Zeilen, als stünden sie nicht da."]),

  H.pmix(["Die Prüfung selbst — ", ["y > 360", true], " — heißt eine Bedingung. Sie ist entweder wahr oder falsch, nie etwas dazwischen. Die 360 ist der Boden abzüglich des Radius: Der Ball ist 20 Punkte groß, der Boden beginnt bei 380, also berührt der Ball ihn, wenn sein Mittelpunkt bei 360 steht."]),

  H.pmix([["y = 360", true], " setzt ihn genau auf den Boden. Ohne diese Zeile bliebe er ein paar Punkte zu tief stecken — er ist ja im letzten Schritt ein Stück zu weit gerutscht. Diese Korrektur wirkt kleinlich und ist bei jeder Kollision nötig, sonst sackt ein Ball mit der Zeit durch den Boden."]),

  H.pmix([["tempo = -tempo * 0.9", true], " ist der eigentliche Abprall, und er steckt in einem einzigen Minuszeichen. War das Tempo +8, also acht Punkte nach unten, wird daraus -8, also acht nach oben. Das Minuszeichen dreht die Richtung um. Mehr ist ein Abprall nicht."]),

  H.pmix(["Das ", ["* 0.9", true], " nimmt bei jedem Aufschlag ein Zehntel des Tempos weg. Deshalb werden die Sprünge kleiner, und deshalb kommt der Ball irgendwann zur Ruhe. Setz dort ", ["1.0", true], " ein, und er springt für immer gleich hoch. Setz ", ["1.1", true], " ein, und er schaukelt sich auf, bis er oben aus dem Fenster fliegt."]),

  H.note("Bemerkenswert ist, was hier NICHT steht. Nirgends kommt das Wort „springen“ vor, und nirgends steht, wie hoch. Es gibt nur eine Regel für das Tempo und eine Regel für den Aufprall. Alles, was du siehst — die Flugbahn, die immer kleiner werdenden Sprünge, das Zur-Ruhe-Kommen —, ergibt sich daraus von selbst. Das ist eine der schönsten Erfahrungen beim Programmieren."),

  H.h2("Sechs Sprünge auf einen Blick"),

  H.p("Auf einem Standbild sieht man von alldem wenig. Also derselbe Trick wie beim Fall: Wir malen den ganzen Weg auf einmal."),

  H.code([
    'SCREEN(640, 400, "Der Weg")',
    "",
    "DIM n AS INTEGER",
    "DIM x AS FLOAT",
    "DIM y AS FLOAT",
    "DIM tempo AS FLOAT",
    "",
    "WHILE NOT QUITREQUESTED() AND NOT KEYPRESSED(KEY_ESCAPE)",
    "    CLS(RGB(15, 20, 40))",
    "    BOX(0, 378, 639, 399, RGB(60, 70, 100))",
    "",
    "    x = 20",
    "    y = 30",
    "    tempo = 0",
    "    FOR n = 0 TO 200",
    "        CIRCLE(x, y, 5, RGB(255, 90, 60))",
    "        tempo = tempo + 0.5",
    "        y = y + tempo",
    "        x = x + 3",
    "        IF y > 370 THEN",
    "            y = 370",
    "            tempo = -tempo * 0.75",
    "        END IF",
    "    NEXT",
    "",
    "    FLIP()",
    "WEND",
  ]),

  H.figure("kap06_2_der_weg.png", "Zweihundert Schritte, sechs Sprünge, jeder kleiner als der vorige. Und nirgends steht, wie hoch er springen soll.", 440, 280),

  H.p("Das ist dasselbe Programm wie oben, nur mit dem Fall aus Kapitel 5 kombiniert: Die Entscheidung sitzt jetzt in einer FOR-Schleife, die zweihundert Schritte auf einmal durchrechnet und jeden davon als Punkt hinsetzt."),

  H.p("Das Bild ist eine Art Beweis. Man sieht die Punkte am oberen Umkehrpunkt dicht beieinander liegen — dort ist der Ball langsam — und unten kurz vor dem Aufprall weit auseinander. Man sieht, dass jeder Bogen niedriger wird. Und man sieht, dass die Bögen nach hinten immer enger werden, weil weniger Tempo auch weniger Zeit in der Luft bedeutet."),

  H.h2("Vier Wände"),

  H.p("Zwei Entscheidungen statt einer, und der Ball ist in einem Kasten gefangen:"),

  H.code([
    'SCREEN(640, 400, "Im Kasten")',
    "",
    "DIM x AS FLOAT",
    "DIM y AS FLOAT",
    "DIM dx AS FLOAT",
    "DIM dy AS FLOAT",
    "x = 320",
    "y = 200",
    "dx = 4",
    "dy = 3",
    "",
    "WHILE NOT QUITREQUESTED() AND NOT KEYPRESSED(KEY_ESCAPE)",
    "    CLS(RGB(15, 20, 40))",
    "    CIRCLE(x, y, 18, RGB(120, 220, 255))",
    "    FLIP()",
    "",
    "    x = x + dx",
    "    y = y + dy",
    "",
    "    IF x < 18 OR x > 621 THEN dx = -dx",
    "    IF y < 18 OR y > 381 THEN dy = -dy",
    "WEND",
  ]),

  H.figure("kap06_3_kasten.png", "Er läuft ewig. Und du wirst ihm länger zusehen, als du zugeben möchtest.", 440, 280),

  H.pmix(["Neu sind zwei Dinge. Erstens die Namen ", ["dx", true], " und ", ["dy", true], ": Sie stehen für die Schrittweite in x- und in y-Richtung. Das ", ["d", true], " ist bei Programmierern die übliche Abkürzung für „Differenz“, also „um wie viel es sich ändert“. Der Ball läuft schräg, weil beide Richtungen gleichzeitig weiterrücken."]),

  H.pmix(["Zweitens steht die Entscheidung hier in EINER Zeile: ", ["IF ... THEN dx = -dx", true], ", ohne ", ["END IF", true], ". Das ist erlaubt, wenn nur eine einzige Anweisung folgt, und es hält kurze Regeln kurz. Sobald es zwei Zeilen werden, brauchst du die ausführliche Form mit ", ["END IF", true], "."]),

  H.pmix([["OR", true], " bedeutet „oder“: Die Bedingung ", ["x < 18 OR x > 621", true], " trifft zu, wenn der Ball entweder zu weit links ODER zu weit rechts ist. Beide Fälle brauchen dieselbe Antwort — Richtung umdrehen —, also darf man sie zusammenfassen."]),

  H.p("Die 18 ist wieder der Radius, die 621 ist 639 minus 18. So prallt der Ball mit seinem Rand ab und nicht mit seinem Mittelpunkt."),

  H.warn("Prüf die Richtung, nicht nur die Stelle. Bleibt der Ball einmal in einer Wand stecken — etwa weil er sehr schnell ist —, dreht diese Regel bei jedem Bild erneut um, und er zittert am Rand fest. Sauberer wäre: nur umdrehen, wenn er sich auch WIRKLICH auf die Wand zu bewegt. Für den Anfang reicht es so; merk dir den Fall für den Tag, an dem dein Ball an einer Wand klebt.", "Der zitternde Ball"),

  H.h2("Wenn es mehr als zwei Möglichkeiten gibt"),

  H.p("Bisher hatten die Entscheidungen zwei Ausgänge: trifft zu oder trifft nicht zu. Oft sind es mehr. Eine Ampel hat vier Phasen, und die schreibt man mit einer Kette:"),

  H.code([
    'SCREEN(640, 400, "Ampel")',
    "",
    "DIM bild AS INTEGER",
    "DIM phase AS INTEGER",
    "DIM aus AS INTEGER",
    "bild = 0",
    "aus = RGB(45, 45, 45)",
    "",
    "WHILE NOT QUITREQUESTED() AND NOT KEYPRESSED(KEY_ESCAPE)",
    "    CLS(RGB(25, 25, 25))",
    "    phase = (bild \\ 60) MOD 4",
    "",
    "    IF phase = 0 THEN",
    "        CIRCLE(320, 110, 50, RGB(255, 0, 0))",
    "        CIRCLE(320, 210, 50, aus)",
    "        CIRCLE(320, 310, 50, aus)",
    "    ELSEIF phase = 1 THEN",
    "        CIRCLE(320, 110, 50, RGB(255, 0, 0))",
    "        CIRCLE(320, 210, 50, RGB(255, 200, 0))",
    "        CIRCLE(320, 310, 50, aus)",
    "    ELSEIF phase = 2 THEN",
    "        CIRCLE(320, 110, 50, aus)",
    "        CIRCLE(320, 210, 50, aus)",
    "        CIRCLE(320, 310, 50, RGB(0, 220, 0))",
    "    ELSE",
    "        CIRCLE(320, 110, 50, aus)",
    "        CIRCLE(320, 210, 50, RGB(255, 200, 0))",
    "        CIRCLE(320, 310, 50, aus)",
    "    END IF",
    "",
    "    FLIP()",
    "    bild = bild + 1",
    "WEND",
  ]),

  H.figure("kap06_5_ampel.png", "Rot und Gelb zusammen — gleich wird es Grün. Die Ampel aus Kapitel 1, jetzt in Betrieb.", 440, 280),

  H.pmix([["ELSEIF", true], " heißt „sonst, falls“. Drachenhauch geht die Kette von oben nach unten durch und nimmt den ERSTEN Zweig, der zutrifft; alle weiteren überspringt es dann. ", ["ELSE", true], " ganz am Ende fängt alles auf, was zu keinem der Fälle passte — hier also Phase 3."]),

  H.pmix([["phase = (bild \\ 60) MOD 4", true], " macht aus dem Bildzähler eine Phasennummer. Der Rückwärts-Schrägstrich teilt ganzzahlig durch 60: Weil sechzig Bilder eine Sekunde sind, wechselt das Ergebnis genau einmal pro Sekunde. Das ", ["MOD 4", true], " lässt es zwischen 0, 1, 2 und 3 kreisen. Eine Sekunde je Phase, dann von vorn."]),

  H.pmix(["Der Karton ", ["aus", true], " hält die Farbe der dunklen Lampe. Sie kommt fünfmal vor; stünde ", ["RGB(45, 45, 45)", true], " fünfmal da, müsstest du beim Ändern fünf Stellen finden. So ist es eine."]),

  H.tip("Rot und Gelb zusammen", "In Deutschland leuchten Rot und Gelb gemeinsam, kurz bevor es Grün wird — im Bild oben ist genau diese Phase zu sehen. In vielen anderen Ländern gibt es das nicht. Programme bilden immer irgendwelche Annahmen über die Welt ab, und die stehen selten dabei."),

  H.h2("Eine Entscheidung je Feld"),

  H.p("Zum Schluss das Muster, das es ohne IF nicht gäbe — achtzig Felder, und für jedes wird einzeln entschieden:"),

  H.code([
    'SCREEN(640, 400, "Schachbrett")',
    "CLS(RGB(30, 30, 30))",
    "",
    "DIM x AS INTEGER",
    "DIM y AS INTEGER",
    "DIM lx AS INTEGER",
    "DIM ly AS INTEGER",
    "",
    "FOR y = 0 TO 7",
    "    FOR x = 0 TO 9",
    "        lx = x * 64",
    "        ly = y * 50",
    "        IF (x + y) MOD 2 = 0 THEN",
    "            BOX(lx, ly, lx + 63, ly + 49, RGB(240, 235, 220))",
    "        ELSE",
    "            BOX(lx, ly, lx + 63, ly + 49, RGB(90, 60, 45))",
    "        END IF",
    "    NEXT",
    "NEXT",
    "",
    "FLIP()",
    "SLEEP(4000)",
  ]),

  H.figure("kap06_4_schachbrett.png", "Achtzig Felder, achtzig Entscheidungen — und die ganze Regel steht in einer Klammer.", 440, 280),

  H.pmix(["Die ganze Kunst steckt in ", ["(x + y) MOD 2 = 0", true], ". Zähl Spalte und Zeile zusammen und sieh nach, ob die Summe gerade ist. Beim Feld links oben, (0,0), ist die Summe 0 — gerade, also hell. Rechts daneben, (1,0), ist sie 1 — ungerade, also dunkel. Eine Zeile tiefer, (0,1), ist sie wieder 1, also dunkel. So entsteht das Muster, in dem sich die Farben in beide Richtungen abwechseln."]),

  H.p("Wenn dir diese Zeile zu schlau vorkommt: Sie ist nicht auf Anhieb erfunden worden. Man probiert, sieht sich das Bild an, ändert etwas. Genau das solltest du hier auch tun — setz statt der 2 eine 3 ein und sieh nach, was für ein Muster dabei herauskommt."),

  H.h2("Die Vergleiche im Überblick"),

  H.table([
    [{ text: "a = b", mono: true }, "ist gleich"],
    [{ text: "a <> b", mono: true }, "ist ungleich"],
    [{ text: "a < b", mono: true }, "ist kleiner"],
    [{ text: "a > b", mono: true }, "ist größer"],
    [{ text: "a <= b", mono: true }, "ist kleiner oder gleich"],
    [{ text: "a >= b", mono: true }, "ist größer oder gleich"],
    [{ text: "A AND B", mono: true }, "beide Bedingungen müssen zutreffen"],
    [{ text: "A OR B", mono: true }, "mindestens eine muss zutreffen"],
    [{ text: "NOT A", mono: true }, "kehrt um: aus wahr wird falsch"],
  ], { headers: ["Schreibweise", "Bedeutung"], widths: [2400, 6626] }),

  H.note("Das Gleichheitszeichen tut in Drachenhauch zwei verschiedene Dinge. Steht es allein in einer Zeile (x = 5), legt es etwas in einen Karton. Steht es in einer Bedingung (IF x = 5 THEN), fragt es nach. Manche Sprachen unterscheiden das durch zwei verschiedene Zeichen; hier entscheidet der Zusammenhang, und das ist beim Lesen sogar angenehmer."),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    [{ text: "Erwartet END IF", mono: true }, "Die mehrzeilige Form ist begonnen, aber nicht geschlossen. Ein einzeiliges IF braucht kein END IF, ein mehrzeiliges immer."],
    ["Der Ball bleibt am Rand kleben", "Die Stelle wird nicht zurückgesetzt. Ohne y = 360 rutscht er tiefer, und die Bedingung bleibt für immer wahr."],
    ["Der Ball springt immer höher", "Der Faktor beim Abprall ist größer als 1. Mit 0.9 verliert er Tempo, mit 1.1 gewinnt er welches."],
    ["Der Zweig wird nie ausgeführt", "Die Bedingung trifft nie zu. Lass dir den Wert mit TEXT anzeigen und sieh nach, was wirklich darin steht."],
    ["Immer wird derselbe Zweig genommen", "In einer ELSEIF-Kette gewinnt der erste zutreffende. Steht eine sehr weite Bedingung oben, kommt der Rest nie an die Reihe."],
    ["Der Ball zittert an der Wand", "Er steckt in der Wand, und die Richtung wird bei jedem Bild neu umgedreht. Siehe den Warnkasten oben."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3400, 5626] }),

  H.h2("Aufgaben"),

  H.bullet("Gib dem Abpraller einen Startschwung nach rechts und lass ihn auch von den Seitenwänden abprallen. Du brauchst dafür beide Entscheidungen aus dem Kasten-Programm."),
  H.bullet("Lass den Ball im Kasten die Farbe wechseln, sobald er eine Wand berührt."),
  H.bullet("Ändere die Ampel so, dass Grün doppelt so lange leuchtet wie Rot. Tipp: Die Phasenrechnung muss dafür mehr als vier Phasen kennen."),
  H.bullet("Male ein Schachbrett mit drei Farben statt zweien. Du brauchst dafür MOD 3 und eine ELSEIF-Kette."),
  H.bullet("Bau einen Zufallshimmel, bei dem jeder zwanzigste Stern rot statt weiß ist. RANDINT und eine Entscheidung reichen."),
  H.bullet("Lass zwei Bälle im Kasten laufen und färb beide rot, solange sie sich näher als hundert Punkte sind. Für den Abstand brauchst du den Satz des Pythagoras — oder du vergleichst einfach die Abstände in x und y einzeln."),

  H.p("Dein Programm kann jetzt entscheiden. Was ihm noch fehlt, ist jemand, der ihm sagt, was es tun soll — im nächsten Kapitel bekommst du die Tastatur in die Hand."),
];
