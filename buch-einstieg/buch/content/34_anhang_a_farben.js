module.exports = (H) => [
  H.part("Anhang"),
  H.chapter("A · Farben zum Nachschlagen"),

  H.p("Farben in Drachenhauch sind drei Zahlen von 0 bis 255: wie viel Rot, wie viel Grün, wie viel Blau. Aber es ist Licht, nicht Farbe aus dem Tuschkasten — und deshalb ist alles genau andersherum, als man es aus der Schule kennt."),

  H.bulletRich("Alles auf 0 ", "— kein Licht, also schwarz."),
  H.bulletRich("Alles auf 255 ", "— alles Licht, also weiß."),
  H.bulletRich("Rot und Grün zusammen ", "— nicht Braun, sondern Gelb."),

  H.p("Das ist der eine Satz, an dem man sich am Anfang immer wieder stößt. Wer Gelb sucht und aus dem Malkasten kommt, probiert Rot mit Grün nicht — dabei ist genau das die Antwort."),

  H.h2("Achtzehn Farben, die reichen"),

  H.figure("anhang_farben.png", "Die Tafel zeichnet sich aus denselben Zahlen, die daneben stehen.", 440, 300),

  H.table([
    [{ text: "weiss", mono: true }, "255, 255, 255", { text: "gruen", mono: true }, "90, 210, 120"],
    [{ text: "hellgrau", mono: true }, "200, 205, 215", { text: "tuerkis", mono: true }, "80, 220, 200"],
    [{ text: "grau", mono: true }, "140, 150, 180", { text: "hellblau", mono: true }, "120, 220, 255"],
    [{ text: "dunkelgrau", mono: true }, "60, 70, 95", { text: "blau", mono: true }, "90, 170, 240"],
    [{ text: "schwarz", mono: true }, "0, 0, 0", { text: "dunkelblau", mono: true }, "30, 60, 120"],
    [{ text: "nachtblau", mono: true }, "28, 32, 50", { text: "violett", mono: true }, "170, 120, 240"],
    [{ text: "rot", mono: true }, "255, 60, 60", { text: "pink", mono: true }, "255, 110, 190"],
    [{ text: "orange", mono: true }, "255, 150, 40", { text: "braun", mono: true }, "150, 100, 60"],
    [{ text: "gelb", mono: true }, "255, 210, 70", { text: "sand", mono: true }, "230, 200, 150"],
  ], { headers: ["Name", "R, G, B", "Name", "R, G, B"], widths: [2100, 2400, 2100, 2426], mono: [0, 2] }),

  H.warn("Auf dem Bild fehlt ein Kästchen: „nachtblau“ hat keines. Die Farbe ist da — sie ist nur genau der Hintergrund, auf den sie gemalt wird, RGB(28, 32, 50) auf RGB(28, 32, 50). Nichts ist kaputt, man sieht bloß nichts. Wenn in deinem Programm etwas „nicht gezeichnet wird“, ist das der erste Verdacht, noch vor allen anderen.", "Eine Farbe, die man nicht sieht"),

  H.h2("Heller und dunkler"),

  H.p("Die achtzehn oben sind Ausgangspunkte, keine Auswahl. Aus jeder lassen sich beliebig viele machen, und dafür genügen zwei Rechnungen."),

  H.p("Dunkler heißt: weniger Licht. Man nimmt einen Anteil von jedem der drei Werte."),

  H.code([
    "BOX(x, 56, x + 108, 150, RGB(r * p \\ 100, g * p \\ 100, _",
    "                             b * p \\ 100))",
  ]),

  H.p("Heller heißt nicht „mehr“ — die Werte sind bei 255 am Ende. Heller heißt: näher an Weiß. Man verkleinert den Abstand zu 255."),

  H.code([
    "BOX(x, 246, x + 108, 340, RGB(r + (255 - r) * p \\ 100, _",
    "                              g + (255 - g) * p \\ 100, _",
    "                              b + (255 - b) * p \\ 100))",
  ]),

  H.figure("anhang_heller_dunkler.png", "Dieselbe Farbe, zehnmal. Oben mit Anteil, unten mit Abstand zu 255.", 440, 280),

  H.pmix(["Der ", ["\\", true], " ist die ganzzahlige Division aus Kapitel 2. Mit ", ["/", true], " käme eine Kommazahl heraus — ", ["RGB", true], " nimmt die inzwischen an und rundet sie. Der Unterschied ist einer der Absicht: abschneiden oder runden."]),

  H.h2("Was RGB wirklich tut"),

  H.p("Es sieht nach Zauberei aus, ist aber nur eine Rechnung. Die drei Zahlen werden zu einer einzigen zusammengeschoben:"),

  H.code([
    'PRINT STR$(RGB(255, 136, 0))     \' 16746496',
    "PRINT STR$(&HFF8800)             ' 16746496",
  ], { out: false }),

  H.tip("Nachgemessen", "RGB(255, 136, 0) und &HFF8800 liefern dieselbe Zahl: 16746496. Nachgerechnet ist das 255 × 65536 + 136 × 256 + 0. Eine Farbe ist in Drachenhauch also ein ganz gewöhnlicher INTEGER, und man kann sie in einer Variablen ablegen, in einem Array sammeln oder in eine Datei schreiben wie jede andere Zahl."),

  H.pmix(["Die Schreibweise mit ", ["&H", true], " ist die aus dem Netz: Wer irgendwo ", ["#FF8800", true], " findet, schreibt in Drachenhauch ", ["&HFF8800", true], ". Für ein Anfängerbuch ist ", ["RGB", true], " lesbarer, deshalb kommt sie im Buch nicht vor — aber sie funktioniert."]),

  H.h2("Farben, die zusammen funktionieren"),

  H.p("Die schwierigste Frage bei einer Oberfläche ist nicht, wie man eine Farbe hinschreibt, sondern welche. Drei Regeln bringen einen erstaunlich weit:"),

  H.bulletRich("Dunkler Hintergrund, helle Schrift. ", "Auf einem Bildschirm ist das ruhiger als andersherum, und leuchtende Farben stechen davon ab, statt darin zu ertrinken."),
  H.bulletRich("Wenige Farben. ", "Ein Hintergrund, ein Grauton für Nebensächliches, ein Weißton für Wichtiges, eine Signalfarbe. Wer eine fünfte braucht, braucht meistens eher eine Erklärung."),
  H.bulletRich("Bedeutung statt Geschmack. ", "Grün für gelungen, Rot für misslungen, Grau für unwichtig — nicht, weil es schön ist, sondern weil man es nicht lesen muss."),

  H.tip("Nachgemessen", "In den Programmen dieses Buchs stehen 273 feste Farbangaben — mit 123 verschiedenen Werten. Die häufigste ist RGB(28, 32, 50), der dunkle Hintergrund, siebzehnmal; danach RGB(15, 20, 40) vierzehnmal und RGB(150, 165, 190), das Grau für Nebentexte, elfmal. Über die Hälfte aller Nennungen entfällt auf eine Handvoll Töne — mehr braucht es nicht."),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    [{ text: "RGB-Werte muessen 0..255 sein", mono: true }, "Ein Anteil ist zu groß geworden. Rechne die Endwerte einmal von Hand aus — gerundet wird, aber nicht geklemmt."],
    ["Die Farbe stimmt nicht", "Ein Wert über 255. Rechne die Endwerte einmal von Hand aus."],
    ["Es wird gar nichts gezeichnet", "Die Farbe ist der Hintergrund. Oder es fehlt FLIP()."],
    ["Alles sieht grau aus", "Die drei Werte liegen zu nah beieinander. Farbe entsteht aus dem Unterschied."],
    ["Der Text ist auf dem Hintergrund kaum zu lesen", "Zu wenig Unterschied in der Helligkeit. Mach den Text heller, nicht bunter."],
    ["Rot und Grün ergeben Braun statt Gelb", "Beide Werte sind zu niedrig. Gelb ist RGB(255, 210, 70), nicht RGB(120, 120, 0)."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3600, 5426] }),
];
