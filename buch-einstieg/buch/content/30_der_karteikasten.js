module.exports = (H) => [
  H.chapter("Der Karteikasten"),

  H.p("Der naheliegende Weg, eine Vokabel auszusuchen, ist der schlechteste:"),

  H.code([
    "dran = INT(RND() * LEN(vokabeln))",
  ]),

  H.p("Eine Zeile, funktioniert sofort, und man merkt jahrelang nicht, wie viel Zeit sie kostet. Denn dem Würfel ist es gleich, ob du ein Wort seit einem Jahr kennst oder es dreimal hintereinander falsch hattest. Er bringt beides gleich oft."),

  H.h2("Nachgemessen: was Würfeln anrichtet"),

  H.p("Sechsundfünfzig Vokabeln, dreihundert Abfragen, rein zufällig ausgewürfelt. Danach gezählt, wie oft jede einzelne drankam:"),

  H.table([
    ["seltenste Vokabel", "0 mal", "in 300 Abfragen kein einziges Mal gezeigt"],
    ["häufigste Vokabel", "13 mal", "ob man sie konnte, spielte keine Rolle"],
    ["Vokabeln ohne einen Treffer", "1", "eine blieb ganz auf der Strecke"],
  ], { headers: ["Gemessen", "Wert", "Was das heißt"], widths: [2800, 1400, 4826] }),

  H.p("Bei tausend Abfragen wird es nicht besser, nur anders: Da lag die seltenste bei 5 und die häufigste bei 32. Der Zufall verteilt gleichmäßig auf lange Sicht — aber er verteilt eben nach nichts. Sechsmal so oft abgefragt zu werden ist kein Zeichen dafür, dass man ein Wort nicht kann. Es ist einfach Pech."),

  H.h2("Der Karteikasten"),

  H.p("Die Lösung ist über hundert Jahre alt und stand früher aus Pappe auf dem Schreibtisch: fünf Fächer. Eine neue Karte kommt in Fach 1. Wusstest du sie, rückt sie ein Fach weiter. Wusstest du sie nicht, fällt sie zurück auf Fach 1."),

  H.p("Und der eigentliche Kniff: Je weiter hinten ein Fach ist, desto seltener sieht man es an."),

  H.table([
    ["1", "nach 2 Runden", "das kann ich noch nicht"],
    ["2", "nach 6 Runden", "einmal gewusst"],
    ["3", "nach 15 Runden", "sitzt allmählich"],
    ["4", "nach 40 Runden", "sitzt"],
    ["5", "nach 100 Runden", "nur noch auffrischen"],
  ], { headers: ["Fach", "Wieder dran", "Was es bedeutet"], widths: [900, 2200, 5926] }),

  H.pmix(["Als „Runde“ zählt hier eine beantwortete Vokabel, nicht ein Tag. Das Programm führt einen Zähler mit, und jede Vokabel merkt sich in ihrer Spalte ", ["faellig", true], ", ab welchem Stand des Zählers sie wieder drankommen darf."]),

  H.h2("In Code sind das sieben Zeilen"),

  H.code([
    "CONST WARTE1 = 2",
    "CONST WARTE2 = 6",
    "CONST WARTE3 = 15",
    "CONST WARTE4 = 40",
    "CONST WARTE5 = 100",
    "",
    "FUNCTION wartezeit(f AS INTEGER) AS INTEGER",
    "    IF f <= 1 THEN RETURN WARTE1",
    "    IF f = 2 THEN RETURN WARTE2",
    "    IF f = 3 THEN RETURN WARTE3",
    "    IF f = 4 THEN RETURN WARTE4",
    "    RETURN WARTE5",
    "END FUNCTION",
  ]),

  H.pmix([["CONST", true], " ist neu und schnell erklärt: ein Name für einen Wert, der sich nie ändert. Man könnte die Zahlen auch direkt hinschreiben — aber dann stünde die 15 mitten im Code, und in einem halben Jahr weiß niemand mehr, warum ausgerechnet 15. Mit ", ["WARTE3", true], " steht der Grund im Namen."]),

  H.note("Diese fünf Zahlen sind eine Entscheidung, keine Naturkonstante. Sie sagen, wie streng der Kasten ist. Wer sie verdoppelt, bekommt einen entspannteren Trainer; wer sie halbiert, einen, der bohrt. Es lohnt sich, damit zu spielen — es ist die einzige Stelle im ganzen Programm, an der man das Lernverhalten direkt einstellt."),

  H.h2("Wer als Nächstes drankommt"),

  H.p("Jetzt könnte man denken: immer das niedrigste Fach zuerst, das ist ja das schwerste. Das wurde ausprobiert — und es ist zu gierig. Wörter aus Fach 1 drängen sich so lange vor, dass ein Wort, das einmal Glück hatte und in Fach 2 gerutscht ist, zehn Runden warten muss, obwohl es längst fällig wäre."),

  H.p("Die richtige Regel ist eine andere: Wer am längsten überfällig ist, kommt zuerst. Und weil die Wartezeiten schon nach Fach gestaffelt sind, kommen die schweren Wörter dabei ganz von selbst öfter dran."),

  H.code([
    'erg = DB_QUERY(con, "SELECT id, de, fremd, fach FROM " + _',
    '                    "vokabeln WHERE faellig <= ? ORDER BY " + _',
    '                    "faellig, fach, RANDOM() LIMIT 1", runde)',
  ]),

  H.p("In diesem einen SQL-Satz steckt die ganze Auswahl. Er liest sich fast wie ein deutscher Satz:"),

  H.bulletRich("WHERE faellig <= ? ", "— nur, was auch dran ist."),
  H.bulletRich("ORDER BY faellig ", "— das am längsten Überfällige zuerst."),
  H.bulletRich(", fach ", "— bei gleichem Stand das schwerere Wort zuerst."),
  H.bulletRich(", RANDOM() ", "— und wenn auch das gleich ist, entscheidet der Zufall."),
  H.bulletRich("LIMIT 1 ", "— es wird nur eines gebraucht."),

  H.pmix(["Das ", ["RANDOM()", true], " am Ende ist wichtiger, als es aussieht. Ohne es käme bei jedem Start dieselbe Reihenfolge heraus, und man würde die Vokabeln bald in ihrer Reihenfolge auswendig können statt einzeln. Der Zufall steht hier NICHT an erster Stelle, sondern an letzter: Er entscheidet nur da, wo alles andere gleich ist."]),

  H.warn("Das ist der Unterschied zwischen „zufällig“ und „stumpfsinnig zufällig“, und er ist der Kern dieses Kapitels. Der Zufall ist ein guter Schiedsrichter und ein schlechter Lehrer. Man setzt ihn dorthin, wo eine Entscheidung wirklich egal ist — nicht dorthin, wo eine zu treffen wäre.", "Wo der Zufall hingehört"),

  H.h2("Das Bild, das den Unterschied zeigt"),

  H.p("Weil man das schwer glaubt, wenn man es nur liest, rechnet das erste Programm dieses Kapitels beides durch und malt es hin. Zwanzig Vokabeln, vierzig Runden, fünf Abfragen je Runde. Jede Reihe ist eine Vokabel, jede Spalte eine Runde, jedes Kästchen eine Abfrage."),

  H.figure("kap30_1_wann_dran.png", "Oben Würfeln, unten Karteikasten. Dieselben Wörter, dieselbe Anzahl Abfragen.", 440, 320),

  H.p("Oben Rauschen. Unten eine Struktur: Erst werden alle zwanzig Vokabeln der Reihe nach eingeführt — das ist die Diagonale links. Danach bleiben die oberen Reihen dicht besetzt und die unteren werden immer luftiger. Die oberen sind die schweren Wörter."),

  H.p("Ausgezählt über 200 Abfragen:"),

  H.table([
    ["die fünf schwersten Vokabeln", "57 mal", "76 mal"],
    ["die fünf leichtesten Vokabeln", "52 mal", "32 mal"],
  ], { headers: ["", "gewürfelt", "Karteikasten"], widths: [4200, 2400, 2426] }),

  H.p("Der Würfel behandelt schwer und leicht gleich — 57 zu 52 ist kein Unterschied. Der Kasten wendet doppelt so viel Zeit auf das Schwere. Bei tausend Abfragen in der größeren Liste war es 382 zu 170."),

  H.tip("Nachgemessen", "Dieselben 56 Vokabeln, 1000 Abfragen: Beim Würfeln kam die seltenste 5 mal dran und die häufigste 32 mal — und welche das war, hing nur vom Zufall ab. Beim Karteikasten lagen alle zwischen 9 und 25 — und welche oben lag, hing davon ab, wie oft man sie falsch hatte. Dieselbe Spannweite, aber sie bedeutet etwas."),

  H.h2("Die Farbe der Kästchen"),

  H.p("Im unteren Bild sind die Kästchen verschieden gefärbt — die Farbe zeigt, in welchem Fach die Vokabel gerade steckte:"),

  H.code([
    "BOX(20 + r * 15, 230 + i * 7, 32 + r * 15, 235 + i * 7, _",
    "    RGB(40 + stufe[n] * 40, 230 - stufe[n] * 20, 170))",
  ]),

  H.pmix(["Ein Rechnen mitten in ", ["RGB", true], " — je höher das Fach, desto mehr Rot und desto weniger Grün. Solche Farbverläufe sind der billigste Weg, einer Zeichnung eine zweite Bedeutung mitzugeben. Man sieht nicht nur WANN etwas drankam, sondern auch, wie sicher es damals saß."]),

  H.warn("Beim Rechnen in RGB ist es leicht, über 255 hinauszuschießen. Der erste Entwurf hatte RGB(60 + stufe * 40, ...), und bei Fach 5 wären das 260 gewesen. Es fiel nicht auf, weil das Programm trotzdem lief — aber eine Zahl über 255 in einem Buch ist Schlamperei. Rechne die Endwerte einmal von Hand aus, bevor du eine Formel stehen lässt.", "Über 255 hinaus"),

  H.h2("Der Kasten zum Anfassen"),

  H.p("Das zweite Programm ist der Karteikasten als kleines Lernprogramm: Leertaste deckt auf, J und N sagen, ob du es wusstest, und unten füllen sich die fünf Fächer."),

  H.figure("kap30_2_karteikasten.png", "Beim ersten Start stecken alle sechsundfünfzig Vokabeln in Fach 1.", 440, 280),

  H.code([
    "IF aufgedeckt AND (KEYPRESSED(KEY_J) OR KEYPRESSED(KEY_N)) THEN",
    "    IF KEYPRESSED(KEY_J) THEN",
    "        IF fach < 5 THEN fach = fach + 1",
    "    ELSE",
    "        fach = 1",
    "    END IF",
    "    runde = runde + 1",
    '    DB_EXEC(con, "UPDATE vokabeln SET fach = ?, " + _',
    '                 "faellig = ? WHERE id = ?", fach, _',
    "                 runde + wartezeit(fach), id)",
    '    DB_EXEC(con, "UPDATE stand SET wert = ? " + _',
    "                 \"WHERE schluessel = 'runde'\", runde)",
    "    neues_wort = TRUE",
    "END IF",
  ]),

  H.pmix(["Der Kern sind die ersten fünf Zeilen: eins weiter oder zurück auf eins, und die neue Fälligkeit ergibt sich aus dem neuen Fach. ", ["IF fach < 5", true], " deckelt oben — es gibt kein Fach 6. Der Rest ist Buchführung: der Rundenzähler wandert in die Tabelle ", ["stand", true], ", damit er den Neustart überlebt."]),

  H.p("Die Fächer füllt eine einzige Abfrage:"),

  H.code([
    'erg = DB_QUERY(con, "SELECT fach, COUNT(*) FROM vokabeln " + _',
    '                    "GROUP BY fach")',
    "WHILE DB_NEXT(erg)",
    "    inhalt[DB_GET_INT(erg, 0)] = DB_GET_INT(erg, 1)",
    "WEND",
  ]),

  H.pmix([["GROUP BY", true], " ist der letzte SQL-Satz, den dieses Buch braucht, und einer der nützlichsten: Er fasst Zeilen zu Gruppen zusammen und zählt sie. „Wie viele Vokabeln je Fach“ ist eine Zeile SQL statt einer Schleife über alles."]),

  H.h2("Ein Nebenbei, das wichtig ist"),

  H.p("Im ersten Programm steht diese Zeile:"),

  H.code([
    "IF best < 0 OR faellig[i] < faellig[best] THEN best = i",
  ]),

  H.pmix(["Solange noch nichts gefunden wurde, ist ", ["best", true], " gleich ", ["-1", true], " — und ", ["faellig[-1]", true], " wäre ein Fehler. Trotzdem läuft die Zeile. Der Grund: ", ["OR", true], " ist kurzgeschlossen. Steht links schon TRUE, wird rechts gar nicht mehr hingesehen."]),

  H.tip("Nachgemessen", "Dieselbe Zeile mit einem echten faellig[-1] daneben: Die IF-Zeile läuft ohne Murren durch, das direkte faellig[-1] bricht ab mit „Index -1 ausserhalb [0..2] in Dimension 0“. Bei AND gilt dasselbe umgekehrt: Steht links FALSE, wird rechts nicht ausgewertet. Man kann sich also darauf verlassen — und das erspart in Prüfungen wie „IF i < LEN(a) AND a[i] = x“ eine ganze Verschachtelung."),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    ["Es kommt immer dieselbe Vokabel", "Die UPDATE-Zeile fehlt oder setzt faellig nicht — die Vokabel bleibt fällig."],
    ["Es kommt sofort „nichts fällig“", "faellig steht in der Zukunft, weil der Rundenzähler beim Start nicht gelesen wurde."],
    ["Die Reihenfolge ist bei jedem Start gleich", "RANDOM() im ORDER BY fehlt."],
    ["Die Reihenfolge ist völlig beliebig", "RANDOM() steht VOR faellig. Es gehört ans Ende."],
    ["Alle Vokabeln bleiben in Fach 1", "Der Zweig für die richtige Antwort erhöht das Fach nicht."],
    [{ text: "Index 6 ausserhalb [0..5]", mono: true }, "Das Fach wird über 5 hinaus erhöht. Deckel einbauen."],
    ["Nach dem Neustart ist alles vergessen", "Der Rundenzähler wird nicht in der Tabelle stand gespeichert."],
    ["Zwei Vokabeln kommen in derselben Runde doppelt", "faellig wird auf die aktuelle Runde gesetzt statt auf eine spätere."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3600, 5426] }),

  H.h2("Aufgaben"),

  H.bullet("Verdopple alle fünf Wartezeiten und sieh dir an, wie das Bild aus dem ersten Programm sich ändert."),
  H.bullet("Bau ein sechstes Fach ein, aus dem eine Vokabel gar nicht mehr kommt."),
  H.bullet("Lass eine falsche Antwort nicht ganz auf Fach 1 zurückfallen, sondern nur ein Fach zurück. Vergleich im Bild, was besser aussieht."),
  H.bullet("Zeig unter jedem Fach zusätzlich an, wie viele Vokabeln gerade fällig sind."),
  H.bullet("Ergänze eine Taste, die den ganzen Kasten zurücksetzt — mit GUI_CONFIRM als Rückfrage."),
  H.bullet("Schreib das erste Programm so um, dass es drei Regeln nebeneinander malt: würfeln, niedrigstes Fach zuerst, längste Überfälligkeit zuerst."),

  H.p("Der Kasten weiß jetzt, WAS drankommt. Was er noch nicht weiß, ist, WIE gefragt werden soll — und das ist keine Kleinigkeit."),
];
