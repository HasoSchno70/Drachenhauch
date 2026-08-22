module.exports = (H) => [
  H.chapter("Zahlen mit Namen"),

  H.p("Im letzten Kapitel standen alle Zahlen fest im Programm. Der Kopf hatte den Radius 130, die Augen saßen bei 275 und 365. Wolltest du den Kopf größer haben, musstest du fünf Zahlen von Hand nachrechnen — und bei der sechsten hast du dich verzählt."),

  H.p("Dieses Kapitel bringt das in Ordnung. Am Ende hast du ein Gesicht, bei dem du eine einzige Zahl änderst und alles andere wächst von selbst mit."),

  H.h2("Das Gesicht aus drei Zahlen"),

  H.p("Tipp das ab:"),

  H.code([
    "DIM mx AS INTEGER",
    "DIM my AS INTEGER",
    "DIM r AS INTEGER",
    "",
    "mx = 320",
    "my = 200",
    "r = 130",
    "",
    'SCREEN(640, 400, "Ein Gesicht aus Zahlen")',
    "CLS(RGB(30, 60, 120))",
    "CIRCLE(mx, my, r, RGB(255, 210, 60))",
    "CIRCLE(mx - r / 3, my - r / 4, r / 7, RGB(40, 40, 40))",
    "CIRCLE(mx + r / 3, my - r / 4, r / 7, RGB(40, 40, 40))",
    "FLIP()",
    "SLEEP(3000)",
  ]),

  H.figure("kap02_1_gesicht.png", "Kein einziger Kreis kennt seine Zahlen selbst — alle rechnen sie aus.", 440, 280),

  H.h2("Zeile für Zeile"),

  H.pmix([["DIM mx AS INTEGER", true], " — das ist eine Ankündigung. Sie sagt: „Ab jetzt gibt es etwas, das ", ["mx", true], " heißt, und darin wohnt eine ganze Zahl.“ ", ["DIM", true], " kommt von „dimension“ und ist BASIC-Tradition; du darfst es als „richte ein“ lesen. ", ["AS INTEGER", true], " ist die Angabe, was für ein Ding hineinpasst — eine ganze Zahl, ohne Komma."]),

  H.p("Man nennt so etwas eine Variable. Das Bild dafür ist ein beschrifteter Karton: DIM stellt den Karton hin und schreibt den Namen drauf. Er ist zunächst leer."),

  H.pmix([["mx = 320", true], " legt etwas hinein. Das Gleichheitszeichen ist hier keine mathematische Aussage, sondern ein Befehl: „nimm die 320 und tu sie in den Karton mx.“ Der Unterschied wird gleich wichtig."]),

  H.pmix([["SCREEN(640, 400, ...)", true], " kennst du. Neu ist, dass die Zeile jetzt weiter unten steht — nach den Ankündigungen. Das ist Geschmackssache, aber eine gute Gewohnheit: Erst sagen, womit man arbeitet, dann arbeiten."]),

  H.pmix([["CIRCLE(mx, my, r, ...)", true], " — hier passiert das Entscheidende. Wo im letzten Kapitel eine Zahl stand, steht nun ein Name. Drachenhauch sieht ", ["mx", true], ", schaut in den Karton, findet 320 und malt an Stelle 320. Ein Name an der Stelle einer Zahl bedeutet immer: schau nach, was drinsteht, und nimm das."]),

  H.pmix([["CIRCLE(mx - r / 3, my - r / 4, r / 7, ...)", true], " ist die Zeile fürs linke Auge, und sie rechnet. ", ["mx - r / 3", true], " heißt: nimm die Mitte und geh um ein Drittel des Radius nach links. Bei mx = 320 und r = 130 sind das 320 minus 43,3 — also 276,7. Das Auge sitzt nicht mehr an einer festen Stelle, sondern an einer, die sich aus Mitte und Größe ergibt."]),

  H.p("Die zweite Augenzeile ist dieselbe mit einem Plus statt einem Minus. Beide Augen liegen ein Viertel des Radius über der Mitte und sind ein Siebtel des Radius groß."),

  H.note("Diese vier Zahlen — ein Drittel, ein Viertel, ein Siebtel — sind nicht heilig. Sie sind das, was beim Herumprobieren gut aussah. Ändere sie und schau, was passiert. Genau so entstehen sie auch in echten Programmen."),

  H.h2("Und jetzt der Trick"),

  H.p("Ändere eine einzige Zeile:"),

  H.code(["r = 60"]),

  H.figure("kap02_1b_gesicht_klein.png", "Dieselben zwölf Zeilen. Eine Zahl anders.", 440, 280),

  H.p("Der Kopf ist kleiner geworden — und die Augen sind mitgewandert, ohne dass du sie angefasst hast. Sie sind näher zusammengerückt, kleiner geworden und sitzen weiterhin auf der richtigen Höhe. Kein einziger Kreis kennt seine Zahlen selbst; alle rechnen sie aus dem Radius aus."),

  H.p("Das ist der ganze Sinn von Variablen, und es ist ein größerer Gedanke, als er auf den ersten Blick wirkt: Du hast aufgehört, ein bestimmtes Gesicht zu malen, und angefangen, ein Rezept für Gesichter zu schreiben."),

  H.tip("Probier das aus, bevor du weiterliest", "Setz r auf 190. Auf 25. Setz mx auf 100 und schieb das ganze Gesicht nach links. Ändere nur my und lass es wandern. Fünf Sekunden pro Versuch, und du verstehst Variablen besser als nach drei Seiten Text."),

  H.h2("Ein Karton, der sich ändern darf"),

  H.p("Ein Karton behält seinen Inhalt nicht für immer. Du kannst jederzeit etwas Neues hineinlegen — das Alte ist dann weg. Das nutzt das nächste Programm aus: Es malt zwei Gesichter mit denselben drei Zeilen, indem es zwischendurch die Kartons neu füllt."),

  H.code([
    "DIM mx AS INTEGER",
    "DIM my AS INTEGER",
    "DIM r AS INTEGER",
    "",
    'SCREEN(640, 400, "Zwei aus einem Rezept")',
    "CLS(RGB(30, 60, 120))",
    "",
    "mx = 180",
    "my = 200",
    "r = 110",
    "CIRCLE(mx, my, r, RGB(255, 210, 60))",
    "CIRCLE(mx - r / 3, my - r / 4, r / 7, RGB(40, 40, 40))",
    "CIRCLE(mx + r / 3, my - r / 4, r / 7, RGB(40, 40, 40))",
    "",
    "mx = 460",
    "r = 60",
    "CIRCLE(mx, my, r, RGB(120, 220, 255))",
    "CIRCLE(mx - r / 3, my - r / 4, r / 7, RGB(40, 40, 40))",
    "CIRCLE(mx + r / 3, my - r / 4, r / 7, RGB(40, 40, 40))",
    "",
    "FLIP()",
    "SLEEP(3000)",
  ]),

  H.figure("kap02_2_zwei_gesichter.png", "Zweimal dasselbe Rezept, zwischendurch andere Zutaten.", 440, 280),

  H.p("Zwischen den beiden Blöcken stehen nur zwei Zeilen. Sie legen eine neue Mitte und einen neuen Radius in die Kartons; my wird gar nicht angefasst und behält seine 200, weshalb beide Gesichter auf gleicher Höhe sitzen. Die drei Malzeilen darunter sind Zeichen für Zeichen dieselben wie oben — sie finden nur andere Zahlen vor."),

  H.warn("Reihenfolge ist alles. Die Malzeilen benutzen, was IM MOMENT ihres Ablaufs in den Kartons liegt. Schreibst du mx = 460 versehentlich UNTER die Malzeilen, ändert das am Bild gar nichts mehr — das Gesicht ist da schon gemalt. Ein Programm ist keine Sammlung von Regeln, sondern eine Reihenfolge von Handlungen.", "Von oben nach unten"),

  H.h2("Warum muss ich das ankündigen?"),

  H.pmix(["Manche Sprachen lassen dich einfach ", ["mx = 320", true], " schreiben, ohne Ankündigung. Drachenhauch besteht auf dem ", ["DIM", true], ", und dafür gibt es einen guten Grund: Tippfehler."]),

  H.p("Stell dir vor, du schreibst weiter unten aus Versehen mmx statt mx. Ohne Ankündigungspflicht würde die Sprache achselzuckend einen neuen Karton namens mmx anlegen, etwas hineintun — und dein Gesicht wäre auf einmal woanders, ohne jede Fehlermeldung. Du würdest eine halbe Stunde suchen."),

  H.p("Mit Ankündigungspflicht bekommst du stattdessen dies zu sehen, sobald die Zeile an die Reihe kommt:"),

  H.code(["Laufzeitfehler in gesicht.dh:9: Variable 'mmx' nicht deklariert (DIM fehlt?)"], { out: true }),

  H.p("Dateiname, Zeilennummer, Name des Übeltäters. Diese scheinbare Umständlichkeit erspart dir über die Jahre mehr Zeit, als sie kostet."),

  H.h2("Rechnen"),

  H.p("Mit Variablen darfst du rechnen wie auf dem Papier:"),

  H.table([
    ["a + b", "plus"],
    ["a - b", "minus"],
    ["a * b", "mal — der Stern, nicht das x"],
    ["a / b", "geteilt, mit Nachkommastellen: 130 / 3 ergibt 43,333…"],
    ["a \\ b", "geteilt, ganzzahlig: 130 \\ 3 ergibt 43, der Rest fällt weg"],
    ["a MOD b", "der Rest: 130 MOD 3 ergibt 1"],
  ], { headers: ["Schreibweise", "Bedeutung"], widths: [2400, 6626], mono: [0] }),

  H.p("Punkt vor Strich gilt, und Klammern setzen sich durch — genau wie in der Schule. Wenn du unsicher bist, setz lieber eine Klammer zu viel; sie kostet nichts und macht die Zeile lesbar."),

  H.note("Der Kreis oben bekommt mit r / 7 einen Radius von 18,57 — eine Kommazahl, obwohl r als INTEGER angekündigt ist. Das ist erlaubt: Der Schrägstrich liefert immer eine Kommazahl, und CIRCLE kann damit umgehen. Willst du sauber ganze Zahlen, nimm den Rückwärts-Schrägstrich.", "Der Schrägstrich rundet nicht"),

  H.h2("Das Fenster kennt seine eigene Mitte"),

  H.p("Wenn Variablen Rezepte ermöglichen, dann liegt der nächste Schritt nahe: auch die Fenstergröße in Kartons zu legen. Dann rechnet sich alles andere daraus aus."),

  H.code([
    "DIM breite AS INTEGER",
    "DIM hoehe AS INTEGER",
    "",
    "breite = 640",
    "hoehe = 400",
    "",
    'SCREEN(breite, hoehe, "Immer in der Mitte")',
    "CLS(RGB(20, 25, 45))",
    "CIRCLE(breite / 2, hoehe / 2, hoehe / 3, RGB(255, 120, 60))",
    "BOX(0, hoehe / 2 - 2, breite - 1, hoehe / 2 + 2, RGB(255, 255, 255))",
    "FLIP()",
    "SLEEP(3000)",
  ]),

  H.figure("kap02_3_mitte.png", "Ändere breite und hoehe — der Kreis bleibt zentriert, der Strich passt sich an.", 440, 280),

  H.pmix([["SCREEN(breite, hoehe, ...)", true], " nimmt die Werte aus den Kartons, statt sie fest im Text stehen zu haben. Der Kreis sitzt bei ", ["breite / 2", true], " und ", ["hoehe / 2", true], " — das ist die Mitte, was auch immer die Zahlen gerade sind. Sein Radius ist ein Drittel der Höhe, also wächst er mit."]),

  H.pmix([["BOX", true], " malt ein gefülltes Rechteck. Es will vier Zahlen, und zwar zwei Ecken: erst links-oben, dann rechts-unten. Hier also von ganz links, zwei Punkte oberhalb der Mitte, bis ganz rechts, zwei Punkte unterhalb. Das ergibt einen vier Punkte hohen Strich, der immer genau auf der Mitte liegt."]),

  H.warn("BOX will ZWEI ECKEN, nicht Breite und Höhe. Wer BOX(0, 198, 640, 4) schreibt und dabei an „640 breit, 4 hoch“ denkt, bekommt ein Rechteck von der Zeile 4 bis zur Zeile 198 — also eine weiße Fläche über der halben Bildschirmhöhe. Genau das ist mir beim Schreiben dieses Kapitels passiert. Dasselbe gilt für RECT, das den bloßen Rahmen malt.", "Zwei Ecken, keine Größe"),

  H.p("Setz breite auf 900 und hoehe auf 300 und starte neu. Alles sitzt weiterhin richtig. Du hast die Zahl 640 aus dem Programm entfernt und durch eine Beziehung ersetzt — und Beziehungen halten, wenn sich etwas ändert."),

  H.h2("Farben sind auch nur Zahlen"),

  H.pmix(["Bisher stand in ", ["RGB", true], " immer eine feste Zahl. Da dürfen genauso gut Kartons stehen:"]),

  H.code([
    "DIM rot AS INTEGER",
    "DIM gruen AS INTEGER",
    "DIM blau AS INTEGER",
    "",
    "rot = 255",
    "gruen = 120",
    "blau = 30",
    "",
    'SCREEN(640, 400, "Farbmischer")',
    "CLS(RGB(25, 25, 25))",
    "BOX(60, 80, 360, 320, RGB(rot, gruen, blau))",
    'TEXT(410, 140, "rot   = " + STR$(rot), RGB(255, 120, 120))',
    'TEXT(410, 180, "gruen = " + STR$(gruen), RGB(120, 255, 120))',
    'TEXT(410, 220, "blau  = " + STR$(blau), RGB(120, 160, 255))',
    "FLIP()",
    "SLEEP(4000)",
  ]),

  H.figure("kap02_4_farbmischer.png", "Drei Kartons, eine Farbe — und daneben steht, was drinsteckt.", 440, 280),

  H.pmix([["TEXT", true], " schreibt Text ins Fenster: erst wohin (x und y), dann was, dann in welcher Farbe. Neu sind zwei Dinge in der Mitte."]),

  H.pmix([["STR$(rot)", true], " macht aus der Zahl 255 den Text „255“. Das klingt nach Haarspalterei, ist aber ein echter Unterschied: Mit der Zahl 255 kann man rechnen, mit dem Text „255“ nicht — dafür kann man ihn hinschreiben. ", ["STR$", true], " ist der Übersetzer von der einen Welt in die andere. Das Dollarzeichen am Namen ist ein altes BASIC-Zeichen und bedeutet „liefert Text“; es wird dir noch oft begegnen."]),

  H.pmix(["Das ", ["+", true], " zwischen den beiden klebt zwei Texte aneinander. Bei Zahlen bedeutet das Pluszeichen „addieren“, bei Texten „hintereinanderhängen“. Aus „rot   = “ und „255“ wird „rot   = 255“."]),

  H.pmix(["Streng nötig wäre ", ["STR$", true], " hier nicht: Steht links vom Plus ein Text, macht Drachenhauch die Zahl rechts von selbst zu Text. ", ['PRINT "x = " + a', true], " gibt anstandslos ", ["x = 5", true], " aus. Trotzdem ist es eine gute Gewohnheit, den Übersetzer hinzuschreiben — aus dem Grund, der im nächsten Kasten steht."]),

  H.warn("Sobald ein Text im Spiel ist, hängt jedes weitere Plus nur noch an. Bei a = 5 und b = 3 ergibt \"summe: \" + a + b nicht „summe: 8“, sondern „summe: 53“ — erst wird die 5 angehängt, dann die 3. Willst du die Summe, musst du klammern: \"summe: \" + STR$(a + b). Diese Falle sieht harmlos aus und kostet regelmäßig eine Viertelstunde.", "Die Fünfunddreißig-Falle"),

  H.p("Dreh jetzt an den drei Zahlen. Setz alle drei auf 255 und du bekommst Weiß. Setz gruen auf 0 und schau, wie es sich Richtung Rot verschiebt. Der Kasten zeigt dir die Farbe, die Zeilen daneben zeigen dir, woher sie kommt."),

  H.h2("Wie Kartons heißen sollten"),

  H.p("Namen sind fast frei wählbar: Buchstaben, Ziffern und der Unterstrich, aber nicht am Anfang eine Ziffer. Umlaute lässt man besser weg — deshalb steht oben hoehe und nicht höhe."),

  H.p("Wichtiger als die Regeln ist die Gewohnheit. Ein guter Name sagt, was drinsteckt:"),

  H.table([
    [{ text: "r, mx, my", mono: true }, "geht in Ordnung — kurz, und im Zusammenhang eindeutig"],
    [{ text: "breite, hoehe, punkte", mono: true }, "gut: man liest die Zeile und weiß Bescheid"],
    [{ text: "x1, x2, x3, temp, wert", mono: true }, "schlecht: in drei Wochen weißt du nicht mehr, welches welches war"],
    [{ text: "anzahl_gegner", mono: true }, "gut: mehrere Wörter mit Unterstrich verbinden"],
  ], { headers: ["Beispiel", "Urteil"], widths: [3000, 6026] }),

  H.p("Das klingt nach Kleinkram. Es ist der Unterschied zwischen einem Programm, das du in einem Monat noch ändern kannst, und einem, das du dann neu schreibst."),

  H.h2("Es gibt nicht nur ganze Zahlen"),

  H.p("INTEGER ist eine von mehreren Sorten. Du brauchst vorerst diese vier:"),

  H.table([
    [{ text: "INTEGER", mono: true }, "ganze Zahl", { text: "0, 42, -17", mono: true }],
    [{ text: "FLOAT", mono: true }, "Zahl mit Komma", { text: "3.5, -0.25", mono: true }],
    [{ text: "STRING", mono: true }, "Text", { text: '"Hallo"', mono: true }],
    [{ text: "BOOLEAN", mono: true }, "ja oder nein", { text: "TRUE, FALSE", mono: true }],
  ], { headers: ["Sorte", "Was hineinpasst", "Beispiele"], widths: [1800, 3400, 3826] }),

  H.p("Beim Komma schreibt man einen Punkt, nicht ein Komma — das ist in der Programmierung fast überall so. Drei Komma fünf ist 3.5."),

  H.p("Warum überhaupt Sorten? Weil der Rechner sonst nicht wüsste, was er tun soll. Bei zwei Zahlen bedeutet Plus addieren, bei zwei Texten aneinanderhängen. Nur weil die Sorte bekannt ist, kann er das eine vom anderen unterscheiden — und dir sagen, wenn in einem Zahlenkarton plötzlich Text landen soll."),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    [{ text: "Variable 'mmx' nicht deklariert (DIM fehlt?)", mono: true }, "Tippfehler, oder die DIM-Zeile fehlt. Die Meldung nennt den Namen — such genau den, Buchstabe für Buchstabe."],
    [{ text: "Zuweisung an global: Erwartet INTEGER, erhalten STRING", mono: true }, "Du legst Text in einen Zahlenkarton. Meist ist ein Anführungszeichen verrutscht."],
    [{ text: "'a' wurde in Zeile 1 schon als INTEGER angelegt", mono: true }, "Zwei DIM-Zeilen, derselbe Name, verschiedene Sorten. Zweiten Namen vergeben."],
    ["Das Bild sieht falsch aus, aber es läuft", "Rechenfehler in einer Zeile. Setz TEXT ein und lass dir den Wert anzeigen — das ist die häufigste Suchmethode überhaupt."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [4000, 5026] }),

  H.note("Zwei DIM-Zeilen mit demselben Namen UND derselben Sorte beschwert Drachenhauch nicht — der Karton steht dann eben schon da. Ärger gibt es erst, wenn die Sorten sich unterscheiden. Und noch etwas: Groß- und Kleinschreibung zählt bei Namen nicht. mx und MX sind derselbe Karton."),

  H.h2("Aufgaben"),

  H.bullet("Gib dem Gesicht einen Mund, der ebenfalls mit r wächst. Nimm den Bogen aus Kapitel 1 und ersetze die festen Zahlen durch Rechnungen mit mx, my und r."),
  H.bullet("Bau ein drittes Gesicht ins Zwei-Gesichter-Programm, oben in der Mitte, in einer neuen Farbe."),
  H.bullet("Leg auch die Kopffarbe in drei Kartons und mach den zweiten Kopf grün, ohne eine einzige RGB-Zeile anzufassen."),
  H.bullet("Schreib ein Programm, das eine Zielscheibe malt: fünf Kreise mit demselben Mittelpunkt, deren Radien sich aus einer Variablen ergeben. Ändere danach nur diese eine Variable."),
  H.bullet("Lass den Farbmischer zusätzlich anzeigen, wie hell die Farbe insgesamt ist — addiere die drei Zahlen und gib die Summe mit TEXT aus."),

  H.p("Bisher hast du jeden Kreis einzeln hingeschrieben. Im nächsten Kapitel schreibst du einen — und bekommst fünfhundert."),
];
