module.exports = (H) => [
  H.chapter("Dinge mit Eigenschaften"),

  H.p("Erinnerst du dich an die fünf Arrays für die Funken?"),

  H.code([
    "DIM px[500] AS FLOAT",
    "DIM py[500] AS FLOAT",
    "DIM vx[500] AS FLOAT",
    "DIM vy[500] AS FLOAT",
    "DIM leben[500] AS INTEGER",
  ]),

  H.p("Das funktioniert, und es ist schnell. Aber es ist auch eine Notlösung. Ein Funke ist EIN Ding mit fünf Eigenschaften — und wir haben daraus fünf Dinge mit je fünfhundert Werten gemacht, die nur dadurch zusammengehalten werden, dass überall dieselbe Nummer benutzt wird."),

  H.p("Geht dabei etwas schief, merkt es niemand. Sortier eines der Arrays, und die Zuordnung ist zerrissen — der Warnkasten in Kapitel 14 hat davor gewarnt. Dieses Kapitel bringt die Lösung."),

  H.h2("Ein Bauplan"),

  H.code([
    "CLASS Funke",
    "    DIM x AS FLOAT",
    "    DIM y AS FLOAT",
    "    DIM vx AS FLOAT",
    "    DIM vy AS FLOAT",
    "    DIM leben AS INTEGER",
    "END CLASS",
  ]),

  H.pmix(["Das ist eine Klasse: ein Bauplan für eine Sorte Ding. Sie ist selbst kein Funke, so wie ein Bauplan kein Haus ist. Sie sagt nur: „Ein Funke hat fünf Eigenschaften, und die heißen so.“"]),

  H.p("Ein einzelnes Ding nach diesem Bauplan entsteht mit NEW:"),

  H.code([
    "DIM f AS Funke",
    "f = NEW Funke()",
    "",
    "f.x = 320",
    "PRINT f.x",
  ]),

  H.pmix(["Der Punkt zwischen ", ["f", true], " und ", ["x", true], " heißt: „die Eigenschaft x von f“. Man liest ihn wie ein Genitiv."]),

  H.h2("Der Bauplan kann auch etwas TUN"),

  H.p("Das eigentlich Neue ist aber nicht, dass die Eigenschaften zusammenliegen. Es ist, dass die Befehle dazukommen dürfen, die mit ihnen umgehen:"),

  H.code([
    "CLASS Funke",
    "    DIM x AS FLOAT",
    "    DIM y AS FLOAT",
    "    DIM vx AS FLOAT",
    "    DIM vy AS FLOAT",
    "    DIM leben AS INTEGER",
    "",
    "    SUB Init()",
    "        Neustart()",
    "        leben = RANDINT(0, 90)",
    "    END SUB",
    "",
    "    SUB Neustart()",
    "        x = 320",
    "        y = 380",
    "        vx = RANDINT(-40, 40) / 10.0",
    "        vy = RANDINT(-95, -55) / 10.0",
    "        leben = RANDINT(60, 110)",
    "    END SUB",
    "",
    "    SUB Schritt()",
    "        leben = leben - 1",
    "        IF leben <= 0 THEN Neustart()",
    "        vy = vy + 0.11",
    "        x = x + vx",
    "        y = y + vy",
    "    END SUB",
    "",
    "    SUB Malen()",
    "        DIM h AS INTEGER",
    "        h = leben * 2",
    "        IF h > 255 THEN h = 255",
    "        CIRCLE(x, y, 2, RGB(255, h, 40))",
    "    END SUB",
    "END CLASS",
  ]),

  H.p("Ein SUB innerhalb einer Klasse heißt Methode. Es ist genau dasselbe wie ein eigener Befehl aus Kapitel 13, mit einem entscheidenden Zusatz: Es kennt die Eigenschaften seines eigenen Dings, ohne dass man sie ihm übergeben muss."),

  H.pmix(["Deshalb steht in ", ["Schritt()", true], " einfach ", ["x = x + vx", true], " — gemeint ist immer das x DIESES Funkens. Und ", ["Neustart()", true], " ruft eine andere Methode desselben Dings auf, ebenfalls ohne Umschweife."]),

  H.pmix([["Init", true], " ist besonders: Diese Methode wird von ", ["NEW", true], " automatisch aufgerufen, gleich nachdem das Ding entstanden ist. Sie ist der Ort für alles, was am Anfang stimmen muss. Braucht sie Angaben, gibt man sie beim ", ["NEW", true], " mit — dazu gleich mehr."]),

  H.h2("Und jetzt fünfhundert davon"),

  H.code([
    "DIM funken[500] AS Funke",
    "DIM i AS INTEGER",
    "",
    "FOR i = 0 TO 499",
    "    funken[i] = NEW Funke()",
    "NEXT",
  ]),

  H.p("Ein Array aus Funken statt fünf Arrays aus Zahlen. Und die Hauptschleife schrumpft auf vier Zeilen:"),

  H.code([
    "FOR i = 0 TO 499",
    "    funken[i].Schritt()",
    "    funken[i].Malen()",
    "NEXT",
  ]),

  H.figure("kap16_1_funken_klasse.png", "Dasselbe Feuerwerk wie in Kapitel 14. Von außen sieht man dem Programm nicht an, wie es gebaut ist.", 440, 280),

  H.p("Vergleich das mit Kapitel 14. Dort standen achtzehn Zeilen in der Schleife, und alle fünf Arrays wurden mit demselben Index angefasst. Hier stehen zwei Zeilen, und was ein Funke tut, steht bei dem, was ein Funke ist."),

  H.tip("Nachgemessen: Was kostet das?", "Objekte sind bequemer — aber sind sie langsamer? Beide Fassungen liefen je dreimal ohne Bildratenbremse, 400 Bilder mit fünfhundert Funken. Arrays: 0,50 Sekunden. Objekte: 0,53. Das sind rund fünf Prozent, und die Messreihen überschneiden sich nicht. Zum Vergleich: Für die üblichen sechzig Bilder je Sekunde bräuchte man 6,7 Sekunden. Beide Fassungen sind also mehr als zwölfmal schneller als nötig, und der Unterschied fällt nicht ins Gewicht."),

  H.h2("Angaben beim Bauen"),

  H.p("Meist unterscheiden sich die Dinge einer Sorte, und dann bekommt Init Parameter — die man beim NEW mitgibt:"),

  H.code([
    "CLASS Planet",
    "    DIM abstand AS FLOAT",
    "    DIM tempo AS FLOAT",
    "    DIM winkel AS FLOAT",
    "    DIM groesse AS INTEGER",
    "    DIM farbe AS INTEGER",
    "",
    "    SUB Init(a AS FLOAT, t AS FLOAT, g AS INTEGER, f AS INTEGER)",
    "        abstand = a",
    "        tempo = t",
    "        groesse = g",
    "        farbe = f",
    "        winkel = 0",
    "    END SUB",
    "END CLASS",
  ]),

  H.code([
    "planeten[0] = NEW Planet(45.0, 0.062, 5, RGB(190, 170, 150))",
    "planeten[1] = NEW Planet(75.0, 0.041, 8, RGB(230, 190, 120))",
    "planeten[2] = NEW Planet(108.0, 0.030, 9, RGB(90, 170, 240))",
  ]),

  H.figure("kap16_2_planeten.png", "Fünf Planeten. Jeder weiß selbst, wie weit außen er läuft, wie schnell, wie groß und in welcher Farbe.", 440, 280),

  H.pmix(["Die Parameter heißen ", ["a", true], ", ", ["t", true], ", ", ["g", true], ", ", ["f", true], " und nicht wie die Eigenschaften — sonst wüsste ", ["abstand = abstand", true], " nicht, was gemeint ist. Ein Buchstabe ist hier ausnahmsweise in Ordnung, weil die Zeile direkt darunter sagt, wohin er geht."]),

  H.p("Und die Hauptschleife des Sonnensystems ist wieder kurz:"),

  H.code([
    "FOR i = 0 TO 4",
    "    planeten[i].Schritt()",
    "    planeten[i].Malen()",
    "NEXT",
  ]),

  H.p("Beachte, dass nirgends steht, welcher Planet wie schnell ist. Das weiß jeder selbst. Willst du einen sechsten dazu, ist das eine Zeile — und keine einzige Änderung an der Schleife."),

  H.h2("Wann lohnt sich eine Klasse?"),

  H.table([
    ["Ein Ding hat mehrere Eigenschaften, die zusammengehören", "Klasse", "Funke, Planet, Gegner, Vokabel"],
    ["Es gibt viele gleichartige davon", "Klasse im Array", "500 Funken, 20 Gegner"],
    ["Die Eigenschaften müssen zusammen bleiben, auch beim Sortieren", "Klasse", "Name und Punktzahl"],
    ["Es ist eine einzelne Zahlenreihe", "Array reicht", "Frequenzen, Punktestände"],
    ["Du suchst über einen Namen", "Map reicht", "Farbregister, Zähler"],
  ], { headers: ["Woran du es erkennst", "Nimm", "Beispiele"], widths: [3600, 1800, 3626] }),

  H.note("Für einen einzelnen Wert lohnt eine Klasse nicht. Der Nutzen beginnt bei drei, vier Eigenschaften — und er wächst mit jedem Ding, das dazukommt. Bei den fünf Arrays der Funken war die Grenze längst überschritten; bei den zwei Zahlen eines Balls in Pong ist sie es nicht."),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    [{ text: "INIT: erwartet 2..2 Argument(e), erhalten 0", mono: true }, "NEW ruft Init automatisch auf — die Angaben gehören in die Klammern hinter NEW, nicht in einen eigenen Aufruf."],
    ["Alle Dinge sehen gleich aus", "Init setzt feste Werte statt der Parameter, oder die Parameter werden nicht zugewiesen."],
    ["Das Array bleibt leer", "DIM funken[500] AS Funke legt nur Platz an, keine Funken. Jedes Ding braucht sein eigenes NEW."],
    ["Eine Eigenschaft ändert sich nicht", "Der Parameter heißt genauso wie die Eigenschaft — dann überschreibt sich die Zuweisung selbst."],
    [{ text: "Unbekannter Name", mono: true }, "Eine Eigenschaft wird von außen ohne Punkt angesprochen. Von außen immer f.x, innen nur x."],
    [{ text: "Erwartet END CLASS", mono: true }, "Ein Block in der Klasse ist offen geblieben."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3800, 5226] }),

  H.h2("Aufgaben"),

  H.bullet("Gib der Funken-Klasse eine Eigenschaft für die Farbe, damit nicht alle gleich glühen."),
  H.bullet("Bau eine Klasse Gegner mit Position, Tempo und Leben — und lass zehn davon über den Bildschirm ziehen."),
  H.bullet("Gib dem Planeten einen eigenen Mond: eine zweite Klasse, die einen Planeten kennt und um ihn kreist."),
  H.bullet("Schreib eine Klasse Vokabel mit den Eigenschaften deutsch, englisch und wie oft sie schon richtig war. Sie wird im Abschlussprojekt gebraucht."),
  H.bullet("Bau Snake so um, dass ein Glied ein Objekt ist. Überleg dabei, ob das Programm dadurch besser wird — die Antwort ist nicht selbstverständlich ja."),
  H.bullet("Gib dem Planeten eine Methode, die seinen Abstand zur Sonne als Text zurückgibt, und zeig ihn an."),

  H.p("Du kannst jetzt Dinge bauen, die etwas sind und etwas können. Im nächsten Kapitel geht es um die Sorte Wert, die bisher zu kurz gekommen ist: Text."),
];
