module.exports = (H) => [
  H.chapter("Arrays können mehr"),

  H.p("Kapitel 9 brauchte Arrays für die Schlange, und mehr als zwei Zeilen Erklärung waren dafür nicht nötig: ein Behälter mit nummerierten Fächern. Dabei ist geblieben, obwohl noch einiges darin steckt."),

  H.p("Dieses Kapitel holt es nach — und dabei entstehen ein Feuerwerk aus fünfhundert Funken und ein Programm, das seit 1970 Menschen fasziniert."),

  H.h2("Fünfhundert Funken"),

  H.p("Ein einzelner Funke braucht fünf Angaben: wo er ist (zwei Zahlen), wohin er fliegt (zwei Zahlen) und wie lange er noch lebt. Fünfhundert Funken brauchen also fünf Arrays:"),

  H.code([
    "DIM px[500] AS FLOAT",
    "DIM py[500] AS FLOAT",
    "DIM vx[500] AS FLOAT",
    "DIM vy[500] AS FLOAT",
    "DIM leben[500] AS INTEGER",
  ]),

  H.p("Das ist dasselbe Muster wie beim Instrument im letzten Teil: parallele Arrays, ein gemeinsamer Index. Fach 137 ist immer derselbe Funke, egal in welchem der fünf Arrays."),

  H.figure("kap14_1_funken.png", "Fünfhundert Funken, gelb an der Quelle, rot beim Verglühen. Kein einziger davon hat einen eigenen Namen.", 440, 280),

  H.p("Der ganze Kern des Programms ist eine Schleife über alle fünfhundert:"),

  H.code([
    "FOR i = 0 TO 499",
    "    leben[i] = leben[i] - 1",
    "",
    "    IF leben[i] <= 0 THEN",
    "        px[i] = 320",
    "        py[i] = 380",
    "        vx[i] = RANDINT(-40, 40) / 10.0",
    "        vy[i] = RANDINT(-95, -55) / 10.0",
    "        leben[i] = RANDINT(60, 110)",
    "    END IF",
    "",
    "    vy[i] = vy[i] + 0.11",
    "    px[i] = px[i] + vx[i]",
    "    py[i] = py[i] + vy[i]",
    "",
    "    h = leben[i] * 2",
    "    IF h > 255 THEN h = 255",
    "    CIRCLE(px[i], py[i], 2, RGB(255, h, 40))",
    "NEXT",
  ]),

  H.p("Vier Abschnitte, und jeder ist für sich einfach:"),

  H.bulletRich("Altern. ", "Jeder Funke verliert ein Leben je Bild."),
  H.bulletRich("Wiedergeburt. ", "Wer aufgebraucht ist, startet unten in der Mitte neu — mit zufälligem Schwung nach oben und leicht zur Seite."),
  H.bulletRich("Fliegen. ", "Schwerkraft aufs Tempo, Tempo auf die Stelle. Das ist der fallende Ball aus Kapitel 5, fünfhundertmal."),
  H.bulletRich("Malen. ", "Die Farbe hängt am Restleben: viel Leben ist gelb, wenig ist rot."),

  H.pmix(["Der Kniff mit der Wiedergeburt lohnt einen zweiten Blick. Es wird nie ein Funke gelöscht und nie einer erzeugt — es gibt immer genau fünfhundert. Wer stirbt, wird sofort wiederverwendet. In Spielen heißt das ein Pool, und es ist die übliche Art, mit vielen kurzlebigen Dingen umzugehen: Schüsse, Funken, Regentropfen, Trümmer."]),

  H.pmix(["Ein Detail: ", ["RANDINT(-40, 40) / 10.0", true], " ergibt eine Kommazahl zwischen -4,0 und 4,0. ", ["RANDINT", true], " liefert nur ganze Zahlen; wer feinere Abstufungen braucht, würfelt größer und teilt hinterher. Das ", ["10.0", true], " mit Punkt-Null ist Absicht — durch ", ["10", true], " geteilt käme hier zwar dasselbe heraus, aber die Schreibweise sagt dem Leser: hier sind Kommazahlen gemeint."]),

  H.h2("Ein Feld mit zwei Nummern"),

  H.p("Bisher hatte jedes Array eine Reihe von Fächern. Manche Dinge sind aber von Natur aus zweidimensional — ein Spielfeld, ein Schachbrett, ein Bild. Dafür darf ein Array zwei Nummern haben:"),

  H.code([
    "DIM feld[64, 38] AS INTEGER",
    "",
    "feld[2, 1] = 9",
    "PRINT feld[2, 1]",
  ]),

  H.code(["9"], { out: true }),

  H.p("Das sind 64 mal 38, also 2432 Fächer — Spalte und Zeile. Man liest es wie Koordinaten: erst wie weit rechts, dann wie weit unten, genau wie beim Malen."),

  H.h2("Das Spiel des Lebens"),

  H.p("Damit lässt sich etwas bauen, das der Mathematiker John Conway 1970 erfunden hat und das seither niemanden mehr losgelassen hat. Es hat nur drei Regeln, keinen Spieler und kein Ziel — und trotzdem sieht man ihm stundenlang zu."),

  H.p("Jede Zelle lebt oder ist tot. Für die nächste Runde zählt sie ihre acht Nachbarn:"),

  H.bulletRich("Eine lebende Zelle ", "bleibt am Leben, wenn zwei oder drei Nachbarn leben. Sonst stirbt sie — an Einsamkeit oder an Überfüllung."),
  H.bulletRich("Eine tote Zelle ", "erwacht, wenn genau drei Nachbarn leben."),

  H.p("Das ist alles. Zwei Sätze."),

  H.figure("kap14_2_leben.png", "Runde 29 aus einem zufälligen Anfang. Die kleinen Quadrate sind stabil, andere Gebilde wandern über das Feld.", 440, 280),

  H.p("In Code sieht die Regel so aus:"),

  H.code([
    "neu[sx, sy] = 0",
    "IF feld[sx, sy] = 1 THEN",
    "    IF nachbarn = 2 OR nachbarn = 3 THEN neu[sx, sy] = 1",
    "ELSE",
    "    IF nachbarn = 3 THEN neu[sx, sy] = 1",
    "END IF",
  ]),

  H.p("Und das Zählen der Nachbarn ist eine Schleife über die neun Felder ringsum, wobei das eigene übersprungen wird:"),

  H.code([
    "nachbarn = 0",
    "FOR dy = -1 TO 1",
    "    FOR dx = -1 TO 1",
    "        IF dx <> 0 OR dy <> 0 THEN",
    "            nx = (sx + dx + 64) MOD 64",
    "            ny = (sy + dy + 38) MOD 38",
    "            nachbarn = nachbarn + feld[nx, ny]",
    "        END IF",
    "    NEXT",
    "NEXT",
  ]),

  H.pmix(["Das ", ["MOD", true], " macht aus dem Feld einen Ring: Wer links hinausschaut, sieht rechts wieder herein. Deshalb steht ", ["+ 64", true], " davor — ", ["-1 MOD 64", true], " wäre negativ, ", ["(-1 + 64) MOD 64", true], " ist 63. Ohne diesen Kniff bräuchte jede Kante eine Sonderbehandlung."]),

  H.warn("ZWEI Felder, nicht eines. Es gibt feld und neu, und die neue Runde wird komplett in neu geschrieben, bevor irgendetwas kopiert wird. Würde man direkt in feld schreiben, sähen die noch nicht bearbeiteten Zellen bereits die neuen Werte ihrer Nachbarn — und die Regel wäre eine völlig andere. Es ist derselbe Gedanke wie beim Nachrücken der Schlange, nur umgekehrt gelöst: dort rückwärts laufen, hier ein zweites Feld.", "Alle Zellen wechseln gleichzeitig"),

  H.p("Beim Zusehen wirst du Dinge entdecken, die niemand hineinprogrammiert hat: Quadrate, die einfach stehenbleiben. Balken, die zwischen waagerecht und senkrecht hin- und herklappen. Und kleine Gebilde aus fünf Zellen, die diagonal über das ganze Feld wandern — die heißen Gleiter, und sie sind berühmt."),

  H.tip("Nachgemessen", "2432 Zellen, je acht Nachbarn — das sind knapp 20 000 Prüfungen für jede Generation. Trotzdem läuft das Programm mit vollen sechzig Bildern je Sekunde: 300 Bilder in 5,02 Sekunden. Man darf einem Rechner mehr zutrauen, als man denkt."),

  H.h2("Listen, die wachsen"),

  H.p("Bisher stand die Größe eines Arrays fest. Manchmal weiß man sie vorher nicht — dann legt man ein leeres an und hängt an:"),

  H.code([
    "DIM punkte[0] AS INTEGER",
    "",
    "ARRAY_PUSH(punkte, 30)",
    "ARRAY_PUSH(punkte, 10)",
    "ARRAY_PUSH(punkte, 20)",
    "",
    "PRINT LEN(punkte)",
  ]),

  H.code(["3"], { out: true }),

  H.pmix([["DIM punkte[0]", true], " legt ein Array ohne Fächer an. ", ["ARRAY_PUSH", true], " hängt hinten eines an; ", ["ARRAY_POP", true], " nimmt das letzte wieder weg. ", ["LEN", true], " sagt, wie viele es gerade sind."]),

  H.h2("Was Drachenhauch für dich rechnet"),

  H.p("Für die üblichen Fragen an eine Zahlenreihe gibt es fertige Befehle. Man muss sie nicht selbst schreiben:"),

  H.table([
    [{ text: "SORT(a)", mono: true }, "sortiert das Array — aufsteigend, an Ort und Stelle. Geht auch mit Text."],
    [{ text: "REVERSE(a)", mono: true }, "dreht die Reihenfolge um. Zusammen mit SORT ergibt das absteigend."],
    [{ text: "ARRAY_MAX(a)", mono: true }, "der größte Wert"],
    [{ text: "ARRAY_MIN(a)", mono: true }, "der kleinste"],
    [{ text: "ARRAY_SUM(a)", mono: true }, "die Summe aller"],
    [{ text: "ARRAY_AVG(a)", mono: true }, "der Mittelwert — eine Kommazahl"],
    [{ text: "ARRAY_INDEXOF(a, w)", mono: true }, "an welcher Stelle w steht, oder -1"],
    [{ text: "JOIN$(a, \", \")", mono: true }, "macht aus einem Text-Array eine Zeile"],
  ], { headers: ["Aufruf", "Was er tut"], widths: [2800, 6226], mono: [0] }),

  H.warn("SORT verändert das Array selbst, es gibt keine Kopie zurück. Nach dem Aufruf ist die alte Reihenfolge weg. Bei parallelen Arrays ist das gefährlich: Sortierst du die Punkte, aber nicht die Namen daneben, gehört danach jeder Name zur falschen Punktzahl. Für solche Fälle braucht man Dinge mit Eigenschaften — die kommen in Kapitel 16.", "Sortieren zerstört die Paarung"),

  H.h2("Zwölf Werte, sortiert und ausgewertet"),

  H.figure("kap14_3_bestenliste.png", "Sortiert, größter zuerst. Die orange Linie ist der Mittelwert — hier 68.", 440, 280),

  H.code([
    "REDIM(punkte, 0)",
    "FOR i = 1 TO 12",
    "    ARRAY_PUSH(punkte, RANDINT(10, 100))",
    "NEXT",
    "SORT(punkte)",
    "REVERSE(punkte)",
    "hoch = ARRAY_MAX(punkte)",
    "mittel = ARRAY_AVG(punkte)",
  ]),

  H.p("Acht Zeilen: leeren, zwölfmal würfeln, sortieren, umdrehen, Höchstwert und Mittel holen. Ohne die fertigen Befehle wären das gut vierzig."),

  H.pmix([["REDIM(punkte, 0)", true], " setzt die Länge auf null zurück — nötig, weil die Leertaste eine neue Runde auslöst und sonst immer zwölf weitere angehängt würden."]),

  H.pmix(["Die Balkenhöhe ist ", ["punkte[i] * 260 \\ hoch", true], ": der Wert im Verhältnis zum größten, mal die verfügbaren 260 Punkte. Dadurch reicht der höchste Balken immer genau bis oben, egal wie die Zahlen ausfallen. Solches Skalieren braucht man bei jedem Diagramm."]),

  H.h2("Alles durchgehen, ohne zu zählen"),

  H.p("Wenn du nur alle Werte brauchst und die Nummer gar nicht, gibt es eine kürzere Schreibweise:"),

  H.code([
    "DIM n AS INTEGER",
    "",
    "FOR EACH n IN [10, 20, 30]",
    "    PRINT n",
    "NEXT",
  ]),

  H.code(["10", "20", "30"], { out: true }),

  H.pmix([["FOR EACH", true], " nimmt der Reihe nach jeden Wert. Die eckigen Klammern im Aufruf sind übrigens ein Array, das direkt hingeschrieben wird — praktisch für kurze feste Listen."]),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    [{ text: "Index ausserhalb", mono: true }, "Ein Fach jenseits der Anzahl. Bei DIM a[10] ist 9 das letzte."],
    [{ text: "ARRAY_PUSH erwartet ARRAY", mono: true }, "Das Array wurde nicht mit eckigen Klammern angelegt. Für eine wachsende Liste: DIM a[0] AS INTEGER."],
    ["Alle Funken sitzen aufeinander", "Der Zufall steht außerhalb der Schleife und wird einmal statt fünfhundertmal gewürfelt."],
    ["Das Spiel des Lebens erstarrt sofort", "Es wird direkt in feld geschrieben statt in neu. Die Zellen sehen dann bereits die neuen Werte ihrer Nachbarn."],
    ["Die Namen passen nicht mehr zu den Punkten", "SORT hat nur eines von zwei parallelen Arrays sortiert."],
    ["Die Liste wächst bei jedem Durchgang", "REDIM auf null fehlt vor dem Neubefüllen."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3600, 5426] }),

  H.h2("Aufgaben"),

  H.bullet("Lass die Funken an der Stelle starten, an der ein mit den Pfeiltasten gesteuerter Punkt gerade steht."),
  H.bullet("Gib jedem Funken eine eigene Größe, die mit dem Alter kleiner wird. Ein sechstes Array genügt."),
  H.bullet("Setz im Spiel des Lebens einen Gleiter von Hand: Die fünf Zellen liegen bei (1,0), (2,1), (0,2), (1,2) und (2,2). Sieh zu, wie er wandert."),
  H.bullet("Färb im Spiel des Lebens die Zellen danach ein, wie lange sie schon leben. Du brauchst dafür ein drittes Feld."),
  H.bullet("Lass beim Spiel des Lebens die Ränder tot sein statt umlaufend, und vergleiche, wie sich das Bild ändert."),
  H.bullet("Zeig in der Bestenliste zusätzlich an, wie viele Werte über dem Mittel liegen."),

  H.p("Arrays halten viele gleichartige Dinge. Im nächsten Kapitel kommt ein Behälter dazu, in dem man nicht nach Nummer sucht, sondern nach Namen."),
];
