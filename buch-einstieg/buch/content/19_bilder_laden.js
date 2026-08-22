module.exports = (H) => [
  H.part("Teil IV — Sprites und Bewegung"),
  H.chapter("Bilder laden"),

  H.p("Achtzehn Kapitel lang hat alles aus Kreisen, Rechtecken und Linien bestanden. Das ist mehr, als man denkt — aber ein Raumschiff aus drei Kreisen bleibt ein Raumschiff aus drei Kreisen."),

  H.p("In diesem Teil kommen richtige Bilder dazu. Und im nächsten Kapitel malst du sie selbst."),

  H.h2("Ein Bild anzeigen"),

  H.code([
    'SCREEN(640, 400, "Ein Bild")',
    "",
    "DIM schiff AS IMAGE",
    'schiff = LOADIMAGE("schiff.png")',
    "",
    "WHILE NOT QUITREQUESTED() AND NOT KEYPRESSED(KEY_ESCAPE)",
    "    CLS(RGB(15, 20, 40))",
    "    DRAWIMAGE(schiff, 304, 184)",
    "    FLIP()",
    "WEND",
  ]),

  H.figure("kap19_1_bild_zeigen.png", "Ein Sprite von 32 mal 32 Punkten, geladen aus einer Datei.", 440, 280),

  H.pmix([["DIM schiff AS IMAGE", true], " ist wieder eine neue Sorte Karton — darin wohnt ein geladenes Bild."]),

  H.pmix([["LOADIMAGE", true], " liest die Datei ein. Sie muss neben dem Programm liegen: Ein Dateiname ohne Pfad bezieht sich auf das Verzeichnis des Programms, genau wie beim Lesen und Schreiben in Kapitel 18."]),

  H.pmix([["DRAWIMAGE(schiff, 304, 184)", true], " malt es. Die beiden Zahlen sind die LINKE OBERE ECKE, nicht die Mitte — das ist bei Bildern anders als bei ", ["CIRCLE", true], ", wo der Mittelpunkt gemeint ist."]),

  H.p("Das Schiff ist 32 mal 32 Punkte groß. Damit es mitten im Fenster steht, muss man also die halbe Größe abziehen: 320 minus 16 sind 304, 200 minus 16 sind 184."),

  H.warn("Nachgemessen liegen die sichtbaren Punkte des Schiffs bei x von 306 bis 333 — nicht bei 304. Der Grund ist harmlos und wird dich trotzdem einmal verwirren: Rings um die Figur liegt ein durchsichtiger Rand. Der zählt zur Bildgröße, ist aber nicht zu sehen. Wenn dein Sprite „nicht ganz da sitzt, wo es soll“, ist fast immer der durchsichtige Rand schuld.", "Das Bild ist größer als die Figur"),

  H.h2("Wichtig: laden gehört nach oben"),

  H.p("Die Ladezeile steht vor der Schleife, und das ist kein Zufall. Sie liest eine Datei von der Platte — verglichen mit allem anderen im Programm ist das eine Ewigkeit."),

  H.warn("Schreib LOADIMAGE nie in die Spielschleife. Sechzig Ladevorgänge je Sekunde für dasselbe Bild bringen jedes Programm zum Stocken. Das ist derselbe Gedanke wie bei den Klängen in Kapitel 11: einmal vorbereiten, oft benutzen.", "Einmal laden, oft malen"),

  H.h2("Das Schiff, das du steuerst"),

  H.p("Damit lässt sich das Programm aus Kapitel 7 aufwerten. Es ist Zeile für Zeile dasselbe — nur steht statt des Kreises jetzt ein Bild:"),

  H.code([
    "DIM schiff AS IMAGE",
    "DIM x AS FLOAT",
    "DIM y AS FLOAT",
    "",
    'schiff = LOADIMAGE("schiff.png")',
    "x = 304",
    "y = 320",
  ]),

  H.code([
    "    DRAWIMAGE(schiff, x, y)",
  ]),

  H.figure("kap19_2_schiff_steuern.png", "Dieselben acht Zeilen Steuerung wie in Kapitel 7. Nur sieht es jetzt nach etwas aus.", 440, 280),

  H.p("Bei den Grenzen ändert sich eine Kleinigkeit: Weil x und y die linke obere Ecke sind, ist die rechte Grenze nicht 639, sondern 639 minus der Bildbreite — also 608 bei einem 32 Punkte breiten Schiff."),

  H.code([
    "IF x < 0 THEN x = 0",
    "IF x > 608 THEN x = 608",
    "IF y < 0 THEN y = 0",
    "IF y > 368 THEN y = 368",
  ]),

  H.h2("Ein Bild, viele Figuren"),

  H.p("Ein geladenes Bild darf so oft gemalt werden, wie du willst — es wird ja nur einmal gelesen. Damit ist eine ganze Flotte eine Doppelschleife:"),

  H.code([
    "FOR sy = 0 TO 3",
    "    FOR sx = 0 TO 5",
    "        DRAWIMAGEPART(gegner, 0, 0, 32, 32, _",
    "                      60 + sx * 90 + schwung, 40 + sy * 70)",
    "    NEXT",
    "NEXT",
  ]),

  H.figure("kap19_3_flotte.png", "Vierundzwanzig Gegner aus einer einzigen Bilddatei — und sie schwanken gemeinsam hin und her.", 440, 280),

  H.pmix(["Hier steht ", ["DRAWIMAGEPART", true], " statt ", ["DRAWIMAGE", true], ", und das lohnt eine Erklärung. Die Datei ", ["gegner.png", true], " ist 64 Punkte breit und enthält ZWEI Figuren nebeneinander — zwei Haltungen desselben Gegners. So etwas heißt ein Streifen."]),

  H.pmix([["DRAWIMAGEPART(bild, sx, sy, sw, sh, x, y)", true], " schneidet ein Stück heraus: ab welcher Stelle im Bild (sx, sy), wie groß der Ausschnitt ist (sw, sh), und wohin er soll (x, y). Mit ", ["0, 0, 32, 32", true], " nimmt man das erste Feld, mit ", ["32, 0, 32, 32", true], " das zweite."]),

  H.p("Warum man Figuren in eine Datei packt statt in viele: Ein Programm mit zwanzig Einzeldateien ist mühsam zu verteilen, und jedes Laden kostet Zeit. Ein Streifen ist eine Datei, ein Ladevorgang — und welches Feld gemeint ist, entscheidet eine Zahl. Im nächsten Kapitel wird daraus eine Animation."),

  H.pmix(["Das ", ["schwung", true], " im Aufruf ist der Sinus aus Kapitel 3: Alle vierundzwanzig Gegner bekommen dieselbe Verschiebung und schwanken deshalb gemeinsam. Genau so bewegt sich die Formation im Original von 1978."]),

  H.h2("Die Befehle im Überblick"),

  H.table([
    [{ text: "LOADIMAGE(datei)", mono: true }, "Bild laden — einmal, vor der Schleife"],
    [{ text: "DRAWIMAGE(bild, x, y)", mono: true }, "ganz malen, linke obere Ecke bei x, y"],
    [{ text: "DRAWIMAGEPART(b, sx, sy, sw, sh, x, y)", mono: true }, "einen Ausschnitt malen"],
    [{ text: "DRAWIMAGEFLIPPED(bild, x, y, wx, wy)", mono: true }, "gespiegelt malen — für Figuren, die nach links laufen"],
    [{ text: "DRAWIMAGEROT(bild, x, y, winkel)", mono: true }, "gedreht malen"],
    [{ text: "IMAGEWIDTH(bild), IMAGEHEIGHT(bild)", mono: true }, "wie groß das Bild ist"],
  ], { headers: ["Aufruf", "Was er tut"], widths: [4400, 4626], mono: [0] }),

  H.note("PNG ist das richtige Format für Sprites, weil es Durchsichtigkeit kann. Ein JPG hat immer einen Hintergrund — bei einem Foto ist das gleichgültig, bei einer Spielfigur bekommst du einen hässlichen Kasten um sie herum."),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    ["Es erscheint nichts", "Die Datei liegt nicht neben dem Programm. Ein Dateiname ohne Pfad zeigt auf das Verzeichnis des Programms."],
    ["Ein weißer Kasten um die Figur", "Das Bild ist ein JPG oder wurde ohne Durchsichtigkeit gespeichert. PNG nehmen."],
    ["Das Programm stockt", "LOADIMAGE steht in der Schleife."],
    ["Die Figur sitzt nicht ganz richtig", "x und y sind die linke obere Ecke, nicht die Mitte — und der durchsichtige Rand zählt mit."],
    ["Die Figur läuft am rechten Rand hinaus", "Bei der Begrenzung muss die Bildbreite abgezogen werden: 639 minus 32."],
    ["Nur ein Teil ist zu sehen", "Bei DRAWIMAGEPART stimmen die Ausschnittmaße nicht mit der Feldgröße im Streifen überein."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3400, 5626] }),

  H.h2("Aufgaben"),

  H.bullet("Male das Schiff doppelt so groß. DRAWIMAGEPARTEX kann das — es nimmt zusätzlich die Zielgröße entgegen."),
  H.bullet("Lass die Flotte nicht nur schwanken, sondern langsam nach unten rücken."),
  H.bullet("Zeig beide Felder des Gegner-Streifens nebeneinander an und sieh dir den Unterschied genau an."),
  H.bullet("Bau die Flotte so um, dass die Reihen unterschiedlich schnell schwanken."),
  H.bullet("Setz das Schiff aus Kapitel 7 wieder ein und lass es auf die Flotte schießen — der Schuss darf noch ein Kreis bleiben."),
  H.bullet("Dreh das Schiff mit DRAWIMAGEROT, während es sich bewegt: nach links geneigt bei Linksbewegung, nach rechts bei Rechtsbewegung."),

  H.p("Bisher hast du fertige Bilder benutzt. Im nächsten Kapitel malst du deine eigenen — mit einem Werkzeug, das schon auf deinem Rechner liegt."),
];
