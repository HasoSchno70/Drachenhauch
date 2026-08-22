module.exports = (H) => [
  H.chapter("Der Zufall"),

  H.p("Bis hierher wusstest du vor dem Start, wie das Bild aussehen würde. Dieselben Zeilen, dasselbe Ergebnis, jedes Mal."),

  H.p("Das ändert sich jetzt. Mit einem einzigen neuen Befehl wird jeder Start ein anderes Bild — und aus einer Handvoll Zeilen wird ein Sternenhimmel, den es vorher noch nie gab."),

  H.h2("Dreihundert Sterne"),

  H.code([
    'SCREEN(640, 400, "Sternenhimmel")',
    "CLS(RGB(5, 5, 20))",
    "",
    "DIM i AS INTEGER",
    "DIM hell AS INTEGER",
    "DIM x AS INTEGER",
    "DIM y AS INTEGER",
    "",
    "FOR i = 1 TO 300",
    "    x = RANDINT(0, 639)",
    "    y = RANDINT(0, 399)",
    "    hell = RANDINT(60, 255)",
    "    CIRCLE(x, y, RANDINT(1, 2), RGB(hell, hell, 255))",
    "NEXT",
    "",
    "FLIP()",
    "SLEEP(4000)",
  ]),

  H.figure("kap04_1_sternenhimmel.png", "Starte das Programm zweimal. Du bekommst nie denselben Himmel.", 440, 280),

  H.h2("Zeile für Zeile"),

  H.pmix([["RANDINT(0, 639)", true], " liefert eine zufällige ganze Zahl zwischen 0 und 639. Der Name kommt von „random integer“, also „zufällige ganze Zahl“. Beide Grenzen sind mit dabei: Die 0 kann herauskommen und die 639 auch."]),

  H.p("Vier Zufallszahlen bestimmen jeden einzelnen Stern:"),

  H.bulletRich("x ", "— wie weit rechts er steht, irgendwo über die volle Fensterbreite."),
  H.bulletRich("y ", "— wie weit unten, über die volle Fensterhöhe."),
  H.bulletRich("hell ", "— wie hell er leuchtet, von blass bis strahlend."),
  H.bulletRich("der Radius ", "— 1 oder 2. Der steht direkt in der Malzeile, weil er nur einmal gebraucht wird. Größere Sterne wären Murmeln."),

  H.pmix(["Warum hell dagegen erst in einen eigenen Karton wandert: Der Wert wird ZWEIMAL gebraucht, im Rot- und im Grünanteil. Stünde ", ["RANDINT(60, 255)", true], " zweimal in der Zeile, wären es zwei verschiedene Würfe — der Stern bekäme unterschiedlich viel Rot und Grün und damit einen Farbstich. So sind beide gleich, und weil der Blauanteil fest auf 255 steht, leuchten alle Sterne weißlich-blau."]),

  H.p("Auch x und y stehen in eigenen Zeilen, obwohl sie nur einmal gebraucht werden. Das ist reine Lesbarkeit: Eine Malzeile mit vier Zufallsaufrufen darin ist ein Wollknäuel, und im Buch passt sie nicht einmal in eine Zeile. Was zu lang wird, zerlegt man."),

  H.note("Das ist eine allgemeine Regel, keine Besonderheit des Zufalls: Ein Ausdruck, der zweimal in derselben Zeile steht, wird auch zweimal ausgerechnet. Bei 3 + 4 ist das gleichgültig. Bei etwas, das jedes Mal anders antwortet, ist es der Unterschied zwischen einem Stern und einem Farbfehler."),

  H.h2("Ein alter BASIC-Merksatz, der hier nicht gilt"),

  H.pmix(["Wer schon einmal in ein älteres BASIC-Buch geschaut hat, kennt die Ermahnung: „Rufe immer zuerst ", ["RANDOMIZE", true], " auf, sonst bekommst du bei jedem Start dieselben Zahlen.“ In Drachenhauch ist das nicht nötig — der Zufall startet von sich aus verschieden."]),

  H.p("Nachgemessen: Dasselbe Programm dreimal gestartet, jedes Mal zwei Zufallszahlen zwischen 1 und 1000:"),

  H.code(["345  455", "333  393", "869  154"], { out: true }),

  H.p("Drei Läufe, sechs verschiedene Zahlen. Du kannst also gleich loswürfeln."),

  H.h2("Konfetti"),

  H.p("Dasselbe Prinzip mit Rechtecken statt Punkten — und diesmal ist auch die Größe gewürfelt:"),

  H.code([
    'SCREEN(640, 400, "Konfetti")',
    "CLS(RGB(250, 250, 250))",
    "",
    "DIM i AS INTEGER",
    "DIM x AS INTEGER",
    "DIM y AS INTEGER",
    "DIM gross AS INTEGER",
    "DIM rot AS INTEGER",
    "DIM gruen AS INTEGER",
    "DIM blau AS INTEGER",
    "",
    "FOR i = 1 TO 200",
    "    x = RANDINT(0, 620)",
    "    y = RANDINT(0, 380)",
    "    gross = RANDINT(6, 18)",
    "    rot = RANDINT(60, 255)",
    "    gruen = RANDINT(60, 255)",
    "    blau = RANDINT(60, 255)",
    "    BOX(x, y, x + gross, y + gross, RGB(rot, gruen, blau))",
    "NEXT",
    "",
    "FLIP()",
    "SLEEP(4000)",
  ]),

  H.figure("kap04_2_konfetti.png", "200 Schnipsel, jeder mit eigener Stelle, Größe und Farbe.", 440, 280),

  H.pmix(["Hier zahlt sich aus, dass ", ["BOX", true], " zwei Ecken will und nicht Breite und Höhe. Die linke obere Ecke ist ", ["x, y", true], "; die rechte untere ist ", ["x + gross, y + gross", true], " — also beide Male dieselbe Strecke weiter. Dadurch entstehen Quadrate, egal wie groß gross gerade ausfällt."]),

  H.pmix(["Bei den Grenzen steckt eine kleine Überlegung: ", ["RANDINT(0, 620)", true], " statt 639. Ein Schnipsel darf bis zu 18 Punkte breit sein, und wenn seine linke Kante schon bei 639 läge, hinge er zur Hälfte aus dem Fenster. Solche Randüberlegungen wirst du in diesem Buch noch oft anstellen."]),

  H.pmix(["Die drei Farbanteile sind unabhängig gewürfelt — hier ist das genau richtig, denn bunt heißt bunt. Und die Untergrenze 60 statt 0 sorgt dafür, dass kein Schnipsel so dunkel wird, dass er auf dem hellen Grund wie ein Loch aussieht."]),

  H.h2("Eine Landschaft aus Zufall und Sinus"),

  H.p("Reiner Zufall sieht schnell nach Rauschen aus. Interessant wird es, wenn man ihn auf etwas Geordnetes draufsetzt — hier auf die Sinuswelle aus dem letzten Kapitel:"),

  H.code([
    'SCREEN(640, 400, "Gebirge")',
    "CLS(RGB(20, 30, 60))",
    "",
    "DIM x AS INTEGER",
    "DIM hoehe AS FLOAT",
    "",
    "FOR x = 0 TO 639",
    "    hoehe = 230 + SIN(RAD(x)) * 45 + RANDINT(-7, 7)",
    "    BOX(x, hoehe, x + 1, 399, RGB(45, 115, 70))",
    "NEXT",
    "",
    "FLIP()",
    "SLEEP(4000)",
  ]),

  H.figure("kap04_3_gebirge.png", "Die Welle gibt die Form, der Zufall gibt die Rauheit.", 440, 280),

  H.p("Die entscheidende Zeile besteht aus drei Teilen, die addiert werden:"),

  H.bulletRich("230 ", "ist die Grundhöhe — ohne alles läge der Boden bei dieser Zeile."),
  H.bulletRich("SIN(RAD(x)) * 45 ", "lässt den Boden weich um bis zu 45 Punkte auf und ab schwingen. Das ist die Hügelform."),
  H.bulletRich("RANDINT(-7, 7) ", "rüttelt jede einzelne Säule ein bisschen — mal sieben Punkte höher, mal sieben tiefer."),

  H.p("Nimm den mittleren Teil weg, und du bekommst einen geraden, rauen Rand. Nimm den Zufall weg, und du bekommst eine glatte, künstlich wirkende Welle. Erst beides zusammen sieht nach Landschaft aus. Diese Mischung aus Regel und Störung steckt hinter erstaunlich vielen Dingen, die auf einem Bildschirm natürlich aussehen."),

  H.pmix(["Die Säulen sind zwei Punkte breit — ", ["x + 1", true], " statt ", ["x", true], ". Mit nur einem Punkt Breite bleiben beim Vergrößern feine Lücken zwischen ihnen, und die Fläche wirkt streifig statt gefüllt. Auch das ist gemessen und nicht geraten."]),

  H.h2("Der Zufall, der keiner ist"),

  H.p("Jetzt kommt der eigentlich interessante Teil dieses Kapitels. Setz eine einzige Zeile vor dein Sternenprogramm:"),

  H.code([
    "RANDOMIZE(7)",
    "",
    'SCREEN(640, 400, "Immer derselbe Himmel")',
    "CLS(RGB(5, 5, 20))",
    "",
    "DIM i AS INTEGER",
    "DIM hell AS INTEGER",
    "DIM x AS INTEGER",
    "DIM y AS INTEGER",
    "",
    "FOR i = 1 TO 300",
    "    x = RANDINT(0, 639)",
    "    y = RANDINT(0, 399)",
    "    hell = RANDINT(60, 255)",
    "    CIRCLE(x, y, RANDINT(1, 2), RGB(hell, hell, 255))",
    "NEXT",
    "",
    "FLIP()",
    "SLEEP(4000)",
  ]),

  H.figure("kap04_4_gleicher_himmel.png", "Diesen Himmel bekommst du. Immer. Bei dir sieht er genauso aus wie hier im Buch.", 440, 280),

  H.p("Starte es zwei-, dreimal. Der Himmel ist jedes Mal exakt derselbe — und zwar genau der, der oben abgebildet ist. Du kannst Stern für Stern vergleichen."),

  H.p("Das liegt daran, dass der Rechner gar nicht wirklich würfeln kann. Er rechnet die „Zufallszahlen“ aus, nach einem festen Verfahren, das aus einer Startzahl die nächste macht, daraus die übernächste und so fort. Die Folge sieht wirr aus, ist aber vollständig festgelegt — sie hängt einzig an der Startzahl."),

  H.pmix([["RANDOMIZE(7)", true], " setzt diese Startzahl auf 7. Ab da liegt die ganze Folge fest. Schreibst du 8 hin, bekommst du einen anderen Himmel — aber auch den immer wieder."]),

  H.p("Ohne diese Zeile sucht sich Drachenhauch beim Start selbst eine Startzahl, und die ist bei jedem Lauf eine andere. Daher der wechselnde Himmel."),

  H.tip("Wofür man das braucht", "Das klingt nach einer Kuriosität, ist aber ausgesprochen nützlich. Wenn dein Programm einen Fehler hat, der nur bei bestimmten Zufallszahlen auftritt, kannst du mit einer festen Startzahl genau diese Situation immer wieder herstellen, statt auf ihr Wiederauftreten zu warten. Und wenn du jemandem deine Welt zeigen willst, bekommt er dieselbe zu sehen — genau deshalb steht die Startzahl 7 in diesem Buch."),

  H.h2("Die Würfel im Überblick"),

  H.table([
    [{ text: "RANDINT(1, 6)", mono: true }, "ganze Zahl von 1 bis 6, beide Enden dabei", "wie ein Würfel"],
    [{ text: "RND(10)", mono: true }, "ganze Zahl von 0 bis 9 — die Obergrenze ist NICHT dabei", "gemessen an 20 000 Würfen"],
    [{ text: "RND()", mono: true }, "Kommazahl zwischen 0 und 1", "z. B. 0.35036…"],
    [{ text: "RANDF(0.5, 2.0)", mono: true }, "Kommazahl in einem Bereich", "für weiche Werte"],
    [{ text: "RANDOMIZE(7)", mono: true }, "legt die Startzahl fest", "macht den Zufall wiederholbar"],
  ], { headers: ["Aufruf", "Was herauskommt", "Anmerkung"], widths: [2400, 4200, 2426] }),

  H.warn("Die beiden häufigsten Befehle zählen unterschiedlich. RANDINT(0, 9) und RND(10) liefern dasselbe — nämlich 0 bis 9 —, aber RANDINT(0, 10) liefert auch die 10, RND(10) nie. Wer hier durcheinanderkommt, bekommt Sterne, die am rechten Rand fehlen, oder ein Programm, das gelegentlich einen Punkt außerhalb des Fensters malt. Nimm im Zweifel RANDINT: Es steht genau da, was du meinst.", "Zwei Arten zu zählen"),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    ["Alle Sterne sitzen an derselben Stelle", "Der Zufallsaufruf steht vor der Schleife statt darin. Er wird dann einmal gewürfelt und danach immer derselbe Wert benutzt."],
    ["Das Bild ist bei jedem Start gleich", "Irgendwo steht noch ein RANDOMIZE mit fester Zahl. Zeile löschen."],
    [{ text: "RGB erwartet INTEGER, erhalten FLOAT", mono: true }, "RND() oder RANDF liefern Kommazahlen. Für Farben RANDINT nehmen."],
    ["Etwas wird am Rand abgeschnitten", "Die Obergrenze ist zu hoch angesetzt: Sie muss die Größe des gemalten Dings berücksichtigen."],
    ["Die Sterne haben einen Farbstich", "Derselbe Zufallsaufruf steht zweimal in einer Zeile und wird zweimal gewürfelt. In einen Karton legen und den benutzen."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3600, 5426] }),

  H.h2("Aufgaben"),

  H.bullet("Gib dem Sternenhimmel einen Mond: einen großen hellen Kreis an fester Stelle, gemalt VOR den Sternen. Male ihn danach nach den Sternen und erkläre den Unterschied."),
  H.bullet("Lass die Sterne oben im Bild kleiner und dichter sein als unten. Tipp: Die Höhe darf mitentscheiden, wie oft und wie groß gemalt wird."),
  H.bullet("Mach aus dem Konfetti Luftballons — Kreise statt Quadrate, und alle in der unteren Hälfte."),
  H.bullet("Setz auf das Gebirge zwanzig Bäume an zufälligen Stellen. Ein Baum ist ein brauner Strich mit einem grünen Kreis obendrauf."),
  H.bullet("Probier RANDOMIZE mit den Zahlen 1 bis 10 durch und such dir den schönsten Himmel aus. Schreib dir die Zahl auf — du hast damit ein Bild, das du jederzeit wiederbekommst."),
  H.bullet("Male einen Farbverlauf wie in Kapitel 3, aber gib jeder Linie einen kleinen Zufallszuschlag auf die Helligkeit. Das ergibt eine Fläche, die aussieht wie altes Filmkorn."),

  H.p("Alle bisherigen Programme haben ein Bild gemalt und sich dann verabschiedet. Im nächsten Kapitel bleibt das Fenster offen — und was darin steht, fängt an sich zu bewegen."),
];
