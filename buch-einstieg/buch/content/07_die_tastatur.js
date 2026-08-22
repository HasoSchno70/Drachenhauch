module.exports = (H) => [
  H.chapter("Die Tastatur"),

  H.p("Bis hierher hast du zugesehen. Der Ball fiel, der Regen fiel, die Monde zogen ihre Bahn — und du saßest davor wie vor einem Fenster. Das ändert sich auf dieser Seite."),

  H.p("Ein Programm, das auf dich reagiert, fühlt sich vollkommen anders an als eines, das nur abläuft. Es ist derselbe kleine Schritt wie vom Kino zum Spiel."),

  H.note("Die Bilder in diesem Kapitel zeigen den Zustand beim Start — sie können ja niemandes Finger abbilden. Bei diesen Programmen musst du ausnahmsweise selbst tippen, um zu sehen, worum es geht. Es lohnt sich."),

  H.h2("Vier Lampen"),

  H.p("Fangen wir mit etwas an, das nichts weiter tut als hinzusehen: vier Kästchen, angeordnet wie die Pfeiltasten, und jedes leuchtet, solange seine Taste gehalten wird."),

  H.code([
    'SCREEN(640, 400, "Tastenanzeige")',
    "",
    "DIM aus AS INTEGER",
    "DIM an AS INTEGER",
    "DIM f AS INTEGER",
    "DIM hinweis AS INTEGER",
    "aus = RGB(62, 70, 98)",
    "an = RGB(255, 200, 40)",
    "hinweis = RGB(150, 160, 190)",
    "",
    "WHILE NOT QUITREQUESTED() AND NOT KEYPRESSED(KEY_ESCAPE)",
    "    CLS(RGB(18, 22, 38))",
    "",
    "    f = aus",
    "    IF KEYPRESSED(KEY_UP) THEN f = an",
    "    BOX(280, 120, 360, 180, f)",
    "",
    "    f = aus",
    "    IF KEYPRESSED(KEY_LEFT) THEN f = an",
    "    BOX(190, 200, 270, 260, f)",
    "",
    "    f = aus",
    "    IF KEYPRESSED(KEY_DOWN) THEN f = an",
    "    BOX(280, 200, 360, 260, f)",
    "",
    "    f = aus",
    "    IF KEYPRESSED(KEY_RIGHT) THEN f = an",
    "    BOX(370, 200, 450, 260, f)",
    "",
    '    TEXT(190, 300, "Halte die Pfeiltasten gedrueckt.", hinweis)',
    "    FLIP()",
    "WEND",
  ]),

  H.figure("kap07_1_tastenanzeige.png", "So sieht es aus, solange du nichts anfasst. Drück eine Pfeiltaste, und das passende Kästchen leuchtet.", 440, 280),

  H.h2("Zeile für Zeile"),

  H.pmix([["KEYPRESSED(KEY_UP)", true], " ist wahr, solange die Taste Pfeil-nach-oben gehalten wird. Nicht einmal beim Drücken, sondern in JEDEM Bild, in dem sie unten ist — sechzigmal je Sekunde, wenn du sie eine Sekunde hältst."]),

  H.pmix([["KEY_UP", true], " ist eine eingebaute Zahl, die für diese Taste steht. Du musst sie nicht kennen und sollst sie auch nicht auswendig lernen; schreib den Namen hin. Eine Liste steht weiter unten."]),

  H.p("Der Dreisatz für jedes Kästchen ist immer derselbe:"),

  H.bulletRich("f = aus ", "— nimm zunächst die dunkle Farbe an."),
  H.bulletRich("IF ... THEN f = an ", "— wenn die Taste gehalten wird, überschreib sie mit der hellen."),
  H.bulletRich("BOX(...) ", "— male das Kästchen in der Farbe, die jetzt in f steht."),

  H.p("Das ist ein Muster, das dir überall wieder begegnen wird: erst den Normalfall setzen, dann die Ausnahme darüberschreiben. Es ist kürzer und weniger fehleranfällig, als jeden Fall einzeln auszuschreiben."),

  H.tip("Warum vier einzelne Kartons wiederverwendet werden", "f wird viermal benutzt und jedes Mal neu gefüllt. Man könnte auch vier Kartons anlegen — farbe_oben, farbe_links und so fort. Beides ist richtig. Sobald du in Kapitel 12 eigene Befehle schreibst, verschwindet die Wiederholung von selbst."),

  H.h2("Jetzt bewegst du etwas"),

  H.code([
    'SCREEN(640, 400, "Steuern")',
    "",
    "DIM x AS FLOAT",
    "DIM y AS FLOAT",
    "DIM hinweis AS INTEGER",
    "x = 320",
    "y = 200",
    "hinweis = RGB(140, 150, 180)",
    "",
    "WHILE NOT QUITREQUESTED() AND NOT KEYPRESSED(KEY_ESCAPE)",
    "    CLS(RGB(15, 20, 40))",
    "    CIRCLE(x, y, 20, RGB(120, 220, 255))",
    '    TEXT(14, 12, "Pfeiltasten bewegen, ESC beendet", hinweis)',
    "    FLIP()",
    "",
    "    IF KEYPRESSED(KEY_LEFT) THEN x = x - 4",
    "    IF KEYPRESSED(KEY_RIGHT) THEN x = x + 4",
    "    IF KEYPRESSED(KEY_UP) THEN y = y - 4",
    "    IF KEYPRESSED(KEY_DOWN) THEN y = y + 4",
    "",
    "    IF x < 20 THEN x = 20",
    "    IF x > 619 THEN x = 619",
    "    IF y < 20 THEN y = 20",
    "    IF y > 379 THEN y = 379",
    "WEND",
  ]),

  H.figure("kap07_2_steuern.png", "Beim Start steht er in der Mitte. Was dann passiert, liegt an deinen Fingern.", 440, 280),

  H.p("Vier Zeilen für die Bewegung, vier für die Wände. Die ersten vier sind der Kern: Wird links gehalten, geh nach links. Mehr ist es nicht — und weil zwei Tasten gleichzeitig gehalten werden dürfen, läuft der Punkt schräg, ohne dass irgendwo „schräg“ stünde."),

  H.pmix(["Die zweiten vier heißen bei Programmierern begrenzen. Sie sorgen dafür, dass der Punkt im Fenster bleibt: ", ["IF x < 20 THEN x = 20", true], " liest sich als „wenn er zu weit links ist, hol ihn auf 20 zurück“. Die 20 ist wieder der Radius, damit er mit dem Rand anstößt und nicht mit dem Mittelpunkt."]),

  H.p("Lass die vier Begrenzungszeilen einmal weg und fahr aus dem Fenster hinaus. Der Punkt ist dann nicht verschwunden — er wird nur außerhalb gemalt, wo du ihn nicht siehst. Fahr zurück, und er kommt wieder. Programme vergessen nichts, sie zeigen es nur nicht immer."),

  H.warn("Bei diesem Aufbau ist die Geschwindigkeit an die Bildrate gekoppelt: vier Punkte je Bild sind bei sechzig Bildern 240 Punkte je Sekunde. Auf einem Rechner, der langsamer zeichnet, liefe der Punkt langsamer. Für dieses Buch ist das in Ordnung und hält die Programme kurz — merk dir nur, dass ausgewachsene Spiele stattdessen mit der vergangenen ZEIT rechnen.", "Punkte je Bild, nicht je Sekunde"),

  H.h2("Gehalten oder gedrückt — der wichtigste Unterschied"),

  H.p("Für das Bewegen war „solange gehalten“ genau richtig. Für einen Schuss wäre es eine Katastrophe: Du drückst die Leertaste eine halbe Sekunde, und dein Schiff feuert dreißig Schüsse ab."),

  H.p("Deshalb gibt es zwei Befehle, die fast dasselbe zu tun scheinen und sich grundlegend unterscheiden. Dieses winzige Programm zeigt es:"),

  H.code([
    'SCREEN(640, 400, "Gehalten oder gedrueckt")',
    "",
    "DIM gehalten AS INTEGER",
    "DIM gedrueckt AS INTEGER",
    "gehalten = 0",
    "gedrueckt = 0",
    "",
    "WHILE NOT QUITREQUESTED() AND NOT KEYPRESSED(KEY_ESCAPE)",
    "    CLS(RGB(20, 24, 40))",
    '    TEXT(60, 70, "Halte die Leertaste gedrueckt.", RGB(200, 210, 230))',
    '    TEXT(60, 170, "KEYPRESSED: " + STR$(gehalten), RGB(255, 180, 60))',
    '    TEXT(60, 220, "KEYHIT:     " + STR$(gedrueckt), RGB(120, 220, 255))',
    "    FLIP()",
    "",
    "    IF KEYPRESSED(KEY_SPACE) THEN gehalten = gehalten + 1",
    "    IF KEYHIT(KEY_SPACE) THEN gedrueckt = gedrueckt + 1",
    "WEND",
  ]),

  H.p("Halt die Leertaste eine Sekunde lang gedrückt. Der obere Zähler springt auf etwa 60 — einmal je Bild. Der untere steht auf 1. Halt sie fünf Sekunden: oben rund 300, unten immer noch 1. Erst wenn du loslässt und neu drückst, wird unten eine 2 daraus."),

  H.table([
    [{ text: "KEYPRESSED(taste)", mono: true }, "wahr, SOLANGE die Taste unten ist", "Bewegen, Lenken, Gasgeben"],
    [{ text: "KEYHIT(taste)", mono: true }, "wahr nur in dem EINEN Bild, in dem sie heruntergeht", "Schießen, Springen, Umschalten"],
    [{ text: "KEYRELEASED(taste)", mono: true }, "wahr in dem Bild, in dem sie losgelassen wird", "Aufladen und beim Loslassen abfeuern"],
    [{ text: "KEYREPEAT(taste)", mono: true }, "wie KEYHIT, wiederholt aber beim Halten — wie die Tastenwiederholung im Textprogramm", "Zahlen hochzählen, Menüs durchblättern"],
  ], { headers: ["Befehl", "Wann er wahr ist", "Wofür"], widths: [2600, 3800, 2626] }),

  H.warn("Die Namen führen in die Irre. Man würde erwarten, dass KEYPRESSED „wurde gedrückt“ bedeutet, also den einen Moment — es bedeutet aber „ist gedrückt“, also den ganzen Zeitraum. Und KEYHIT, was nach Dauer klingt, ist der Moment. In anderen Sprachen und Baukästen heißen die beiden oft genau umgekehrt. Wenn dein Schiff dreißig Schüsse auf einmal abgibt, weißt du jetzt, welche Zeile schuld ist.", "Zwei Namen, die man sich merken muss"),

  H.h2("Ein Schiff, ein Schuss"),

  H.p("Damit lässt sich das Schießen bauen — und nebenbei lernst du eine neue Sorte Karton kennen, die nur zwei Werte kennt: ja oder nein."),

  H.code([
    'SCREEN(640, 400, "Schuss")',
    "",
    "DIM x AS FLOAT",
    "DIM sx AS FLOAT",
    "DIM sy AS FLOAT",
    "DIM fliegt AS BOOLEAN",
    "DIM hinweis AS INTEGER",
    "x = 320",
    "fliegt = FALSE",
    "hinweis = RGB(140, 150, 180)",
    "",
    "WHILE NOT QUITREQUESTED() AND NOT KEYPRESSED(KEY_ESCAPE)",
    "    CLS(RGB(15, 20, 40))",
    "    BOX(x - 22, 358, x + 22, 374, RGB(120, 220, 255))",
    "    BOX(x - 6, 348, x + 6, 358, RGB(120, 220, 255))",
    "    IF fliegt THEN CIRCLE(sx, sy, 5, RGB(255, 220, 80))",
    '    TEXT(14, 12, "Pfeile bewegen, Leertaste schiesst", hinweis)',
    "    FLIP()",
    "",
    "    IF KEYPRESSED(KEY_LEFT) THEN x = x - 5",
    "    IF KEYPRESSED(KEY_RIGHT) THEN x = x + 5",
    "    IF x < 22 THEN x = 22",
    "    IF x > 617 THEN x = 617",
    "",
    "    IF KEYHIT(KEY_SPACE) AND NOT fliegt THEN",
    "        fliegt = TRUE",
    "        sx = x",
    "        sy = 344",
    "    END IF",
    "",
    "    IF fliegt THEN",
    "        sy = sy - 8",
    "        IF sy < 0 THEN fliegt = FALSE",
    "    END IF",
    "WEND",
  ]),

  H.figure("kap07_4_schuss.png", "Das Schiff wartet. Zwei Rechtecke, mehr ist es nicht.", 440, 280),

  H.pmix([["DIM fliegt AS BOOLEAN", true], " legt einen Karton an, in den nur ", ["TRUE", true], " oder ", ["FALSE", true], " passt — ja oder nein, mehr nicht. Solche Kartons heißen Schalter, und man braucht sie ständig: Läuft das Spiel noch? Ist der Ton an? Fliegt gerade ein Schuss?"]),

  H.pmix([["IF fliegt THEN ...", true], " braucht keinen Vergleich. Ein BOOLEAN ist ja schon die Antwort auf eine Frage; ", ["IF fliegt = TRUE THEN", true], " wäre erlaubt, aber umständlich, so wie „wenn es wahr ist, dass es regnet“."]),

  H.pmix([["IF KEYHIT(KEY_SPACE) AND NOT fliegt THEN", true], " ist die Abschussregel und liest sich fast deutsch: „wenn die Leertaste gedrückt wurde UND gerade KEIN Schuss unterwegs ist“. Der zweite Teil begrenzt dich auf einen Schuss zur Zeit. Nimm ihn weg, und jeder neue Druck versetzt den fliegenden Schuss zurück nach unten — es gibt eben nur einen einzigen Satz Kartons für ihn."]),

  H.p("Der Schuss selbst ist ein Karton für x, einer für y und der Schalter. Er steigt um acht Punkte je Bild, und sobald er oben aus dem Fenster ist, wird der Schalter wieder umgelegt. Danach ist er nicht etwa gelöscht — er wird bloß nicht mehr gemalt und nicht mehr bewegt. Für das Programm ist „weg“ dasselbe wie „wird ignoriert“."),

  H.tip("Für mehrere Schüsse fehlt noch etwas", "Zwanzig Schüsse gleichzeitig bräuchten zwanzig Sätze dieser Kartons, und das schreibt niemand hin. Was dafür fehlt, ist ein Behälter, der viele gleichartige Dinge auf einmal hält — das sind Arrays, und die kommen in Kapitel 13. Bis dahin: ein Schuss."),

  H.h2("Die Tasten, die du brauchst"),

  H.table([
    [{ text: "KEY_LEFT, KEY_RIGHT, KEY_UP, KEY_DOWN", mono: true }, "die Pfeiltasten"],
    [{ text: "KEY_SPACE", mono: true }, "Leertaste"],
    [{ text: "KEY_ESCAPE", mono: true }, "ESC"],
    [{ text: "KEY_RETURN", mono: true }, "Eingabetaste (auch KEY_ENTER)"],
    [{ text: "KEY_A bis KEY_Z", mono: true }, "Buchstaben — für WASD-Steuerung und Kürzel"],
    [{ text: "KEY_0 bis KEY_9", mono: true }, "die obere Ziffernreihe"],
    [{ text: "KEY_F1 bis KEY_F12", mono: true }, "Funktionstasten"],
    [{ text: "KEY_LSHIFT, KEY_LCTRL", mono: true }, "Umschalt und Strg, links (rechts: KEY_RSHIFT, KEY_RCTRL)"],
  ], { headers: ["Name", "Taste"], widths: [3800, 5226] }),

  H.p("Für ein Spiel zu zweit an einer Tastatur belegt man üblicherweise die Pfeiltasten und daneben W, A, S, D — die liegen für die linke Hand genau richtig. Im nächsten Kapitel wird genau das gebraucht."),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    ["Es passiert gar nichts beim Tippen", "Das Fenster hat den Fokus nicht — einmal hineinklicken. Tasten gehen immer an das Fenster, das vorne ist."],
    ["Eine Aktion löst dreißigmal aus", "KEYPRESSED statt KEYHIT. Für alles, was einmal je Druck passieren soll, gehört KEYHIT hin."],
    ["Diagonale Bewegung geht nicht", "Vier einzelne IF nehmen, keine ELSEIF-Kette: Bei einer Kette gewinnt die erste Taste, und die zweite kommt nie dran."],
    ["Der Punkt zittert am Rand", "Die Begrenzung setzt zurück, die Bewegung schiebt wieder hinaus. Erst bewegen, dann begrenzen — nicht umgekehrt."],
    ["Die Bewegung ist ruckelig", "Die Schrittweite ist zu groß. Nimm kleinere Schritte statt seltener Bilder."],
    ["ESC beendet nicht", "In der WHILE-Zeile fehlt die Prüfung. Sie steht in jedem Programm dieses Buchs, Wort für Wort gleich."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3400, 5626] }),

  H.h2("Aufgaben"),

  H.bullet("Erweitere die Tastenanzeige um ein fünftes, breites Kästchen für die Leertaste."),
  H.bullet("Bau die Steuerung auf W, A, S, D um. Danach steuerst du mit beiden Belegungen gleichzeitig — dafür genügt ein OR je Richtung."),
  H.bullet("Lass den gesteuerten Punkt schneller werden, solange eine Taste gehalten wird, und wieder langsamer, wenn keine gedrückt ist. Er fühlt sich dann an wie ein Schlitten auf Eis."),
  H.bullet("Gib dem Schiff aus dem letzten Programm eine Munitionsanzeige: Bei jedem Schuss eins weniger, und bei null geht nichts mehr."),
  H.bullet("Mach aus dem Zähler-Programm eine Stoppuhr: Die Leertaste startet und stoppt sie. Du brauchst dafür KEYHIT und einen Schalter."),
  H.bullet("Lass den Schuss nicht senkrecht, sondern schräg fliegen — und zwar in die Richtung, in die sich das Schiff zuletzt bewegt hat."),

  H.p("Du hast jetzt alles beisammen, was ein Spiel braucht: ein Bild, Bewegung, Entscheidungen und Steuerung. Im nächsten Kapitel setzen wir es zum ersten Mal vollständig zusammen — und spielen zu zweit."),
];
