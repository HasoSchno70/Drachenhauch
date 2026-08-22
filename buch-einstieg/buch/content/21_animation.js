module.exports = (H) => [
  H.chapter("Animation"),

  H.p("Bewegung kennst du seit Kapitel 5: Das Bild wird sechzigmal in der Sekunde neu gemalt, und was darauf steht, hat sich ein Stück verschoben."),

  H.p("Animation ist derselbe Gedanke, nur eine Ebene tiefer. Diesmal verschiebt sich nichts — es wird ein ANDERES Bild gezeigt. Ein Gegner, der abwechselnd die Beine anders stellt, läuft; eine Münze, die vier verschiedene Breiten durchläuft, dreht sich."),

  H.h2("Zwei Bilder, abwechselnd"),

  H.code([
    "feld = (bild \\ 15) MOD 2",
    "DRAWIMAGEPART(gegner, feld * 32, 0, 32, 32, 304, 184)",
  ]),

  H.figure("kap21_1_zwei_frames.png", "Alle fünfzehn Bilder wechselt die Haltung — viermal in der Sekunde.", 440, 280),

  H.p("Zwei Zeilen, und der Gegner lebt. Sie lohnen eine genaue Betrachtung, denn dieselbe Rechnung steckt in jeder Animation, die du je schreiben wirst."),

  H.pmix([["bild \\ 15", true], " teilt den Bildzähler ganzzahlig durch 15. Der Wert ändert sich also nur alle fünfzehn Bilder — bei sechzig Bildern je Sekunde also viermal je Sekunde. Das ist die Geschwindigkeit der Animation."]),

  H.pmix([["MOD 2", true], " lässt das Ergebnis zwischen 0 und 1 kreisen, weil es zwei Felder gibt. Bei vier Feldern stünde dort ", ["MOD 4", true], "."]),

  H.pmix([["feld * 32", true], " macht daraus die Stelle im Streifen: Feld 0 beginnt bei 0, Feld 1 bei 32. Das ist die einzige Stelle, an der die Breite eines Feldes vorkommt."]),

  H.note("Diese Formel ist das Grundmuster jeder Bilderfolge: (zähler \\ tempo) MOD anzahl. Wer sie einmal verstanden hat, kann jede Animation bauen — vom laufenden Männchen bis zum flackernden Feuer."),

  H.h2("Vier Bilder, eine Drehung"),

  H.p("Mit vier Feldern wird aus dem Wechsel eine Bewegung. Die Münze wird von Bild zu Bild schmaler, bis sie nur noch eine Kante ist — und weil es danach wieder von vorn losgeht, sieht es aus, als drehe sie sich."),

  H.code([
    "feld = (bild \\ 8) MOD 4",
    "DRAWIMAGEPART(muenze, feld * 32, 0, 32, 32, 304, 120)",
  ]),

  H.figure("kap21_2_muenze.png", "Oben die laufende Münze, unten der ganze Streifen. Der Strich zeigt, welches Feld gerade dran ist.", 440, 280),

  H.p("Das Programm zeigt beides: oben die Animation, unten alle vier Felder nebeneinander, mit einem Strich unter dem gerade gezeigten. So sieht man beim Laufen zu, wie der Zeiger durch den Streifen wandert."),

  H.pmix(["Hier steht ", ["\\ 8", true], " statt ", ["\\ 15", true], ": Die Münze wechselt siebeneinhalbmal je Sekunde. Dreh an dieser Zahl und sieh dir an, wo die Grenze liegt — unter etwa vier Bildern wird es ein Flimmern, über dreißig ein Ruckeln."]),

  H.tip("Die eine Zahl, die zählt", "Die Anzahl der Felder und die Tempo-Zahl gehören zusammen. Vier Felder alle acht Bilder ergeben eine volle Drehung in gut einer halben Sekunde. Willst du eine langsamere Münze, änderst du die 8 — nicht die Anzahl der Bilder. Mehr Bilder machen die Bewegung feiner, nicht langsamer."),

  H.h2("Die Flotte lebt"),

  H.p("Jetzt beides zusammen: die Flotte aus Kapitel 19 bewegt sich UND animiert."),

  H.code([
    "schwung = SIN(RAD(bild * 2)) * 30",
    "feld = (bild \\ 18) MOD 2",
    "",
    "FOR sy = 0 TO 3",
    "    FOR sx = 0 TO 5",
    "        DRAWIMAGEPART(gegner, feld * 32, 0, 32, 32, _",
    "                      60 + sx * 90 + schwung, 40 + sy * 70)",
    "    NEXT",
    "NEXT",
  ]),

  H.figure("kap21_3_flotte_lebt.png", "Vierundzwanzig Gegner, alle in derselben Haltung — und alle wechseln sie gleichzeitig.", 440, 280),

  H.p("Bemerkenswert ist, dass die Feldnummer AUSSERHALB der Schleifen ausgerechnet wird. Alle vierundzwanzig Gegner zeigen dadurch dasselbe Feld und wechseln im Gleichschritt. Genau das macht den bedrohlichen Eindruck aus, den das Original von 1978 hatte — eine Armee, die gemeinsam atmet."),

  H.pmix(["Willst du das Gegenteil, rechnest du die Feldnummer für jeden einzeln aus und mischst seine Nummer hinein: ", ["(bild \\ 18 + sx + sy) MOD 2", true], ". Dann läuft eine Welle durch die Formation. Ein Zeichen Unterschied, ein völlig anderer Eindruck."]),

  H.h2("Zustände: welche Bilderfolge gerade gilt"),

  H.p("Ein richtiges Spiel hat für eine Figur meist mehrere Folgen: stehen, laufen, springen, sterben. Das löst man, indem man festhält, in welchem Zustand die Figur gerade ist, und je nachdem in einem anderen Bereich des Streifens sucht."),

  H.code([
    "IF laeuft THEN",
    "    feld = 2 + (bild \\ 10) MOD 4",
    "ELSE",
    "    feld = (bild \\ 30) MOD 2",
    "END IF",
  ]),

  H.p("Hier liegen im Streifen zuerst zwei Felder fürs Stehen und danach vier fürs Laufen. Dieselbe Datei, zwei Folgen — und ein einziges IF entscheidet, welche gilt. Mehr steckt hinter dem, was in Spielen „Animationszustände“ heißt, im Kern nicht."),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    ["Die Figur flimmert", "Die Tempo-Zahl ist zu klein. Bei \\ 2 wechselt das Bild dreißigmal je Sekunde."],
    ["Es bewegt sich gar nichts", "Der Bildzähler wird nicht erhöht, oder die Feldnummer wird aus einer festen Zahl gerechnet."],
    ["Es erscheint nur ein halber Gegner", "Die Feldbreite im Aufruf stimmt nicht mit dem Streifen überein — bei 32er-Feldern muss dort 32 stehen, nicht 16."],
    ["Ein Stück vom Nachbarfeld ist zu sehen", "Die Stelle wird falsch gerechnet: Es muss feld * breite sein, nicht feld + breite."],
    ["Die Animation springt zurück", "MOD mit der falschen Anzahl. Bei vier Feldern gehört MOD 4 dorthin."],
    ["Alle Figuren zappeln durcheinander", "Die Feldnummer wird in der Schleife für jede Figur neu und unterschiedlich gerechnet."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3400, 5626] }),

  H.h2("Aufgaben"),

  H.bullet("Lass die Münze rückwärts drehen. Ein Zeichen genügt, wenn du die Feldnummer von der Anzahl abziehst."),
  H.bullet("Bau in die Flotte eine Welle ein, wie oben beschrieben, und vergleiche beide Eindrücke."),
  H.bullet("Zeig neben der Münze an, welches Feld gerade läuft — und dazu den Bildzähler. Beobachte, wie beide zusammenhängen."),
  H.bullet("Gib dem Schiff aus Kapitel 19 eine flackernde Flamme: zwei Fassungen des Sprites, alle fünf Bilder gewechselt."),
  H.bullet("Male eine Figur mit vier Laufbildern und lass sie über den Bildschirm gehen — Bewegung und Animation gleichzeitig."),
  H.bullet("Bau die Zustände aus dem letzten Abschnitt wirklich: Solange eine Pfeiltaste gehalten wird, läuft die Figur; sonst steht sie."),

  H.p("Deine Figuren bewegen sich und leben. Was noch fehlt, damit daraus ein Spiel wird: Sie müssen einander bemerken."),
];
