// Baut die Einleitung des GameBasic-Galaga-Buchs als farbiges, druckbares .docx.
// Aufruf:  node build_book.js   ->  GameBasic-Buch.docx
const fs = require("fs");
const path = require("path");
const {
  Document, Packer, Paragraph, TextRun, ImageRun, AlignmentType, HeadingLevel,
  BorderStyle, Table, TableRow, TableCell, WidthType, ShadingType, PageBreak,
  Header, Footer, PageNumber, LevelFormat,
} = require("docx");

const IMG = path.join(__dirname, "images");

// --- Farbpalette ---
const C_TITLE  = "1B6CA8";   // Blau (Titel/H1)
const C_H2     = "C0451B";   // Warmes Orange-Rot (H2)
const C_ACCENT = "1B6CA8";
const C_CAP    = "6A6A6A";   // Bildunterschrift grau
const C_TIPBG  = "E7F2FA";   // hellblauer Kasten
const C_TIPBD  = "9CC8E6";

// PNG-Groesse aus dem IHDR lesen.
function pngSize(file) {
  const b = fs.readFileSync(file);
  return { w: b.readUInt32BE(16), h: b.readUInt32BE(20) };
}

// Zentriertes Bild + Bildunterschrift, auf maxW/maxH skaliert (Seitenverhaeltnis bleibt).
function figure(name, caption, maxW = 540, maxH = 360) {
  const file = path.join(IMG, name);
  const { w, h } = pngSize(file);
  let W = maxW, H = Math.round((maxW * h) / w);
  if (H > maxH) { H = maxH; W = Math.round((maxH * w) / h); }
  return [
    new Paragraph({
      alignment: AlignmentType.CENTER,
      spacing: { before: 120, after: 40 },
      children: [new ImageRun({
        type: "png", data: fs.readFileSync(file),
        transformation: { width: W, height: H },
        altText: { title: caption, description: caption, name: name },
      })],
    }),
    new Paragraph({
      alignment: AlignmentType.CENTER,
      spacing: { after: 200 },
      children: [new TextRun({ text: caption, italics: true, color: C_CAP, size: 18 })],
    }),
  ];
}

// Absatz-Hilfen.
function p(text, opts = {}) {
  return new Paragraph({
    spacing: { after: 140 }, ...opts,
    children: [new TextRun({ text, size: 22 })],
  });
}
function bullet(text) {
  return new Paragraph({
    numbering: { reference: "bul", level: 0 }, spacing: { after: 60 },
    children: [new TextRun({ text, size: 22 })],
  });
}
function bulletRich(boldText, rest) {
  return new Paragraph({
    numbering: { reference: "bul", level: 0 }, spacing: { after: 60 },
    children: [
      new TextRun({ text: boldText, bold: true, size: 22 }),
      new TextRun({ text: rest, size: 22 }),
    ],
  });
}

// Farbiger Tipp-Kasten (Ein-Zellen-Tabelle mit Schattierung).
function tip(title, text) {
  const border = { style: BorderStyle.SINGLE, size: 6, color: C_TIPBD };
  return new Table({
    width: { size: 9360, type: WidthType.DXA },
    columnWidths: [9360],
    rows: [new TableRow({ children: [new TableCell({
      width: { size: 9360, type: WidthType.DXA },
      borders: { top: border, bottom: border, left: border, right: border },
      shading: { fill: C_TIPBG, type: ShadingType.CLEAR },
      margins: { top: 120, bottom: 120, left: 160, right: 160 },
      children: [
        new Paragraph({ spacing: { after: 40 }, children: [new TextRun({ text: title, bold: true, color: C_ACCENT, size: 22 })] }),
        new Paragraph({ children: [new TextRun({ text, size: 22 })] }),
      ],
    })] })],
  });
}

// Monospace-Code-Block (grauer Kasten mit blauer Leiste links).
function codeBlock(lines) {
  const runs = lines.map((ln, i) => new TextRun({ text: ln, font: "Consolas", size: 19, break: i === 0 ? 0 : 1 }));
  return new Paragraph({
    shading: { fill: "F4F4F4", type: ShadingType.CLEAR },
    border: { left: { style: BorderStyle.SINGLE, size: 18, color: C_ACCENT, space: 6 } },
    spacing: { before: 80, after: 160 }, indent: { left: 120 },
    children: runs,
  });
}

// Absatz mit gemischtem Text + Inline-Code. parts: ["text"] oder ["code", true].
function pmix(parts) {
  return new Paragraph({
    spacing: { after: 140 },
    children: parts.map(([t, code]) => new TextRun(code
      ? { text: t, font: "Consolas", size: 21 }
      : { text: t, size: 22 })),
  });
}

function h1(text) {
  return new Paragraph({
    heading: HeadingLevel.HEADING_1,
    spacing: { before: 320, after: 80 },
    border: { bottom: { style: BorderStyle.SINGLE, size: 12, color: C_ACCENT, space: 4 } },
    children: [new TextRun({ text })],
  });
}
function h2(text) {
  return new Paragraph({ heading: HeadingLevel.HEADING_2, children: [new TextRun({ text })] });
}

// ===================== Inhalt =====================
const children = [];

// --- Titelseite ---
children.push(
  new Paragraph({ spacing: { before: 1400 }, children: [] }),
  new Paragraph({
    alignment: AlignmentType.CENTER,
    children: [new TextRun({ text: "GAMEBASIC", bold: true, color: C_TITLE, size: 88 })],
  }),
  new Paragraph({
    alignment: AlignmentType.CENTER, spacing: { before: 120, after: 80 },
    children: [new TextRun({ text: "Programmieren lernen – Schritt für Schritt", size: 32, color: C_H2, bold: true })],
  }),
  new Paragraph({
    alignment: AlignmentType.CENTER, spacing: { after: 360 },
    children: [new TextRun({ text: "zum eigenen Arcade-Spiel im Stil von Galaga", size: 28, italics: true, color: C_CAP })],
  }),
);
figure("galaga_titel.png", "Das fertige Spiel – unser Ziel in diesem Buch.", 560, 320).forEach(e => children.push(e));
children.push(
  new Paragraph({
    alignment: AlignmentType.CENTER, spacing: { before: 360 },
    children: [new TextRun({ text: "von Hans Schnorrenberger", size: 26, bold: true })],
  }),
  new Paragraph({ children: [new PageBreak()] }),
);

// --- Einleitung ---
children.push(h1("Willkommen!"));
children.push(p("Dieses Buch nimmt dich an die Hand und baut mit dir gemeinsam ein komplettes Arcade-Spiel – einen Klon des Klassikers Galaga. Du brauchst keine Vorkenntnisse. Jedes Kapitel fügt ein kleines, sichtbares Stück hinzu: erst ein Fenster, dann ein Raumschiff, dann Schiessen, Gegner, Einflug-Manöver – bis am Ende ein richtiges Spiel mit Highscore-Liste, Sound und Effekten vor dir läuft."));
children.push(p("Programmieren lernt man am besten, indem man etwas baut, das Spass macht. Genau das ist der Plan. Und das Beste: die Grafik – Raumschiff, Gegner, Schüsse – zeichnest du selbst im mitgelieferten Pixel-Editor."));
children.push(tip("So liest du dieses Buch", "Tippe die Beispiele selbst ab und starte sie sofort. Jeder Codeschnipsel läuft für sich. Fehler sind normal – GameBasic sagt dir in klaren Worten, was los ist."));

// --- Was ist GameBasic ---
children.push(h1("Was ist GameBasic?"));
children.push(p("GameBasic ist eine Programmiersprache aus der BASIC-Familie – also eine Sprache, die bewusst leicht lesbar ist und sich fast wie englische Sätze liest. Sie wurde von Grund auf für Spiele gemacht: Grafik, Sound, Eingabe und Spielablauf sind direkt eingebaut, ohne dass du erst komplizierte Bibliotheken zusammensuchen musst."));
children.push(h2("Was sie besonders macht"));
children.push(bulletRich("Einfach zu lesen: ", "Befehle wie SCREEN, PLOT, DRAWIMAGE oder PLAYSOUND sagen, was sie tun."));
children.push(bulletRich("Sicher durch Typen: ", "Jede Variable hat einen klaren Typ (INTEGER, FLOAT, STRING …). Das verhindert viele Anfängerfehler."));
children.push(bulletRich("Modern: ", "Klassen und Objekte (OOP), Funktionen, Module – alles dabei, wenn du es brauchst, aber nie im Weg."));
children.push(bulletRich("Eine Laufzeit: ", "Dein Programm läuft direkt über die schnelle Runtime „gbrt“ – flüssig und auf Wunsch als fertige .exe exportierbar."));
children.push(p("Ein winziges Programm sieht zum Beispiel so aus:"));
children.push(new Paragraph({
  shading: { fill: "F4F4F4", type: ShadingType.CLEAR },
  border: { left: { style: BorderStyle.SINGLE, size: 18, color: C_ACCENT, space: 6 } },
  spacing: { after: 160 }, indent: { left: 120 },
  children: [
    new TextRun({ text: "SCREEN(640, 480, \"Hallo\")", font: "Consolas", size: 20 }),
    new TextRun({ text: "\n", break: 1 }),
    new TextRun({ text: "TEXT(40, 40, \"Hallo Welt!\", RGB(255, 220, 0))", font: "Consolas", size: 20, break: 1 }),
    new TextRun({ text: "FLIP()", font: "Consolas", size: 20, break: 1 }),
  ],
}));

// --- Was kann GameBasic ---
children.push(h1("Was kann GameBasic alles?"));
children.push(p("Erstaunlich viel – weit mehr, als wir für unser Galaga-Spiel brauchen. Ein kleiner Vorgeschmack:"));
children.push(bulletRich("2D-Grafik: ", "Linien, Rechtecke, Kreise, Farbverläufe, Splines, dicke Linien, Bilder und Sprites."));
children.push(bulletRich("3D-Grafik: ", "Würfel, Kugeln, geladene 3D-Modelle, Licht, Schatten und Kameras."));
children.push(bulletRich("Fertige Fenster-Oberflächen (GUI): ", "Buttons, Schieberegler, Checkboxen, Textfelder – per Klick zusammengesetzt."));
children.push(bulletRich("Sound & Musik: ", "Toneffekte erzeugen, Musik abspielen, eigene Arcade-Sounds synthetisieren."));
children.push(bulletRich("Spiel-Bausteine: ", "Partikel-Effekte, Animationen, Kollisionen, Tilemaps, Pfadsuche und mehr."));

children.push(h2("2D zeichnen"));
figure("demo_2d.png", "Farbverläufe, runde Rechtecke, dicke Linien und eine weiche Spline-Kurve – alles mit eingebauten Befehlen.").forEach(e => children.push(e));

children.push(h2("Dreidimensional"));
figure("demo_3d.png", "GameBasic kann auch 3D: hier ein Gitterboden mit Drahtgitter-Modellen.").forEach(e => children.push(e));

children.push(h2("Fenster-Oberflächen"));
figure("demo_gui.png", "Ein Einstellungs-Fenster mit Schieberegler, Checkbox, Textfeld und Button – fertige GUI-Bausteine.").forEach(e => children.push(e));

// --- Unser Projekt ---
children.push(h1("Unser Projekt: ein Galaga-Clone"));
children.push(p("Galaga ist einer der berühmtesten Arcade-Shooter: unten steuerst du ein Raumschiff, oben schwebt eine Formation bunter Gegner, die in geschwungenen Bahnen einfliegen, herabstürzen und Bomben werfen. Genau dieses Gefühl bauen wir nach – mit allem Drum und Dran."));
figure("galaga_spiel.png", "Mitten im Gefecht: die Formation oben, Schüsse, Explosionen und Punkte-Einblendungen.").forEach(e => children.push(e));
children.push(p("Und es bleibt nicht beim Standard. Unser Spiel bekommt die ikonischen Spezial-Manöver des Originals:"));
children.push(bulletRich("Der Fangstrahl: ", "Ein Boss-Gegner spannt einen Traktorstrahl auf und kann dein Schiff einfangen – befreist du es, fliegst du mit einem Doppeljäger (zwei Schiffe) weiter!"));
children.push(bulletRich("Bonus-Wellen: ", "Alle paar Stufen eine Bonus-Runde ohne Gegenwehr – schiess für Extrapunkte."));
figure("galaga_fangstrahl.png", "Der Fangstrahl des Bosses – das Markenzeichen von Galaga.").forEach(e => children.push(e));
figure("galaga_bonus.png", "Bonus-Welle: die Gegner fliegen nur in Bögen durch.").forEach(e => children.push(e));

children.push(h2("Was am Ende alles drin ist"));
children.push(bullet("Drei mehrfarbige Gegnertypen mit Einflug- und Sturzangriffen"));
children.push(bullet("Fangstrahl & Doppeljäger, Bonus-Wellen, mehrere Schwierigkeitsstufen"));
children.push(bullet("Titelbild mit Laufschrift, Regenbogen-Rand und persistenter Highscore-Liste"));
children.push(bullet("Explosionen, Funkeln, Bildschirm-Wackeln, Retro-Bildschirm-Effekt (CRT)"));
children.push(bullet("Arcade-Sounds und Hintergrundmusik, Steuerung per Tastatur oder Gamepad"));

// --- Aufbau ---
children.push(h1("Wie dieses Buch funktioniert"));
children.push(p("Wir bauen das Spiel in kleinen, lauffähigen Schritten. Nach jedem Kapitel hast du etwas, das du sofort starten und ausprobieren kannst – und das Lust auf das nächste Kapitel macht."));
children.push(bulletRich("Erst das Fenster: ", "Spielschleife, Sternenhimmel."));
children.push(bulletRich("Dann das Schiff: ", "laden, zeichnen, mit der Tastatur bewegen."));
children.push(bulletRich("Sprites selbst zeichnen: ", "im Pixel-Editor gbsprites."));
children.push(bulletRich("Schiessen, Gegner, Formation: ", "Arrays, Klassen, Bewegung."));
children.push(bulletRich("Einflug, Stürze, Bomben, Kollisionen: ", "das eigentliche Spielgefühl."));
children.push(bulletRich("Politur: ", "Sound, Effekte, Highscores, Export als .exe."));
children.push(tip("Alles in Farbe und zum Selbermachen", "Dieses Buch ist als Word-Dokument angelegt. Du kannst es nach Belieben ergänzen, umstellen, eigene Screenshots einfügen – und natürlich ausdrucken."));

// ===================== Kapitel 1 =====================
children.push(new Paragraph({ children: [new PageBreak()] }));
children.push(h1("Kapitel 1: Das erste Fenster"));
children.push(tip("In diesem Kapitel",
  "Du öffnest dein allererstes GameBasic-Fenster, lernst die „Spielschleife“ kennen und zauberst einen scrollenden Sternenhimmel – die Bühne, auf der später unser Raumschiff kämpft."));

children.push(h2("Ein Fenster öffnen"));
children.push(p("Jedes Spiel braucht ein Fenster. In GameBasic genügt dafür eine einzige Zeile:"));
children.push(codeBlock(['SCREEN(480, 640, "Mein Galaga")']));
children.push(pmix([
  ["Der Befehl ", false], ["SCREEN", true],
  [" öffnet ein Fenster: zuerst die Breite (480), dann die Höhe (640), dann der Titel in der Fensterleiste. Wir wählen ein ", false],
  ["Hochformat", false], [" – schmal und hoch, wie ein Arcade-Automat.", false],
]));

children.push(h2("Die Spielschleife"));
children.push(p("Ein Spiel steht nie still: viele Male pro Sekunde wird das Bild neu gezeichnet und angezeigt. Das erledigt eine Schleife, die so lange läuft, bis du das Fenster schliesst:"));
children.push(codeBlock([
  "WHILE NOT QUITREQUESTED()",
  "    ' ... hier wird gezeichnet ...",
  "    FLIP()",
  "WEND",
]));
children.push(pmix([
  ["", false],
  ["WHILE", true], [" … ", false], ["WEND", true],
  [" wiederholt alles dazwischen immer wieder. ", false],
  ["QUITREQUESTED()", true],
  [" wird erst wahr, wenn du auf das Schliessen-Kreuz klickst. Und ", false],
  ["FLIP()", true],
  [" zeigt das fertig gezeichnete Bild an – erst wird im Hintergrund gemalt, dann in einem Rutsch umgeschaltet. So flackert nichts.", false],
]));

children.push(h2("Das Bild löschen"));
children.push(pmix([
  ["Am Anfang jedes Durchlaufs wischen wir das alte Bild weg – mit ", false],
  ["CLS", true], [" (engl. „clear screen“) und einer Farbe:", false],
]));
children.push(codeBlock(['CLS(&H05060F)        \' dunkles Weltraum-Blau']));
children.push(pmix([
  ["Farben schreibt man als ", false], ["&Hrrggbb", true],
  [" – je zwei Stellen für Rot, Grün, Blau. ", false],
  ["&H05060F", true], [" ist ein sehr dunkles Blau, perfekt fürs All.", false],
]));

children.push(h2("Ein Sternenhimmel"));
children.push(p("Sterne sind einfach viele leuchtende Punkte. Wir merken uns ihre Positionen in zwei Listen (sogenannten Arrays) – eine für x, eine für y – und verteilen sie zu Beginn zufällig:"));
children.push(codeBlock([
  "CONST NSTARS AS INTEGER = 60",
  "DIM starX[NSTARS] AS INTEGER",
  "DIM starY[NSTARS] AS INTEGER",
  "",
  "DIM i AS INTEGER",
  "FOR i = 0 TO NSTARS - 1",
  "    starX[i] = RANDINT(0, 479)",
  "    starY[i] = RANDINT(0, 639)",
  "NEXT i",
]));
children.push(pmix([
  ["", false],
  ["CONST", true], [" ist ein fester Wert, der sich nie ändert (hier: 60 Sterne). ", false],
  ["DIM starX[NSTARS]", true], [" legt eine Liste mit 60 Plätzen an. Die ", false],
  ["FOR", true], ["-Schleife läuft von 0 bis 59 und gibt jedem Stern mit ", false],
  ["RANDINT(min, max)", true], [" eine zufällige Position.", false],
]));
children.push(p("In der Spielschleife lassen wir die Sterne nach unten fallen. Wer unten herausfällt, taucht oben neu auf:"));
children.push(codeBlock([
  "FOR i = 0 TO NSTARS - 1",
  "    starY[i] = starY[i] + 2",
  "    IF starY[i] > 639 THEN starY[i] = 0 : starX[i] = RANDINT(0, 479)",
  "    PLOT(starX[i], starY[i], &HFFFFFF)",
  "NEXT i",
]));
children.push(pmix([
  ["", false],
  ["PLOT(x, y, farbe)", true], [" zeichnet einen einzelnen Punkt – hier in Weiss (", false],
  ["&HFFFFFF", true], ["). Die Zeile mit ", false], ["IF", true],
  [" setzt einen Stern zurück nach oben, sobald er unten austritt – und gibt ihm gleich eine neue x-Position.", false],
]));

children.push(h2("Das ganze Programm"));
children.push(p("Setzt man alles zusammen, ergibt sich dein erstes vollständiges GameBasic-Programm. Tippe es ab und starte es:"));
children.push(codeBlock([
  'SCREEN(480, 640, "Mein Galaga - Kapitel 1")',
  "",
  "CONST NSTARS AS INTEGER = 60",
  "DIM starX[NSTARS] AS INTEGER",
  "DIM starY[NSTARS] AS INTEGER",
  "DIM i AS INTEGER",
  "FOR i = 0 TO NSTARS - 1",
  "    starX[i] = RANDINT(0, 479)",
  "    starY[i] = RANDINT(0, 639)",
  "NEXT i",
  "",
  "WHILE NOT QUITREQUESTED()",
  "    CLS(&H05060F)",
  "    FOR i = 0 TO NSTARS - 1",
  "        starY[i] = starY[i] + 2",
  "        IF starY[i] > 639 THEN starY[i] = 0 : starX[i] = RANDINT(0, 479)",
  "        PLOT(starX[i], starY[i], &HFFFFFF)",
  "    NEXT i",
  "    FLIP()",
  "WEND",
]));
figure("kap01_fenster.png", "Dein erstes Fenster: ein scrollender Sternenhimmel.", 300, 400).forEach(e => children.push(e));

children.push(h2("Was du gelernt hast"));
children.push(bulletRich("SCREEN ", "öffnet ein Fenster (Breite, Höhe, Titel)."));
children.push(bulletRich("WHILE … WEND + FLIP ", "bilden die Spielschleife, die das Bild immer wieder neu anzeigt."));
children.push(bulletRich("CLS ", "löscht das Bild, PLOT zeichnet einen Punkt – Farben als &Hrrggbb."));
children.push(bulletRich("CONST, DIM und FOR ", "speichern und verarbeiten viele Werte (hier: 60 Sterne)."));

children.push(h2("Übung"));
children.push(bullet("Ändere die Fenstergrösse und die Sternenfarbe. Was passiert?"));
children.push(bullet("Lass die Sterne schneller fallen (grössere Zahl statt + 2)."));
children.push(bullet("Erhöhe NSTARS auf 200 – ein dichteres Weltall."));
children.push(bullet("Zusatz: Gib manchen Sternen mit einer zweiten Geschwindigkeit mehr Tiefe (Tipp: ein drittes Array für das Tempo)."));

// ===================== Kapitel 2 =====================
children.push(new Paragraph({ children: [new PageBreak()] }));
children.push(h1("Kapitel 2: Das Schiff"));
children.push(tip("In diesem Kapitel",
  "Du holst dein Raumschiff auf den Bildschirm: ein fertiges Bild laden, an die richtige Stelle zeichnen und mit den Pfeiltasten nach links und rechts steuern."));

children.push(h2("Ein Bild laden"));
children.push(p("Das Raumschiff ist ein kleines Bild (ein „Sprite“). Im nächsten Kapitel zeichnest du es selbst – fürs Erste nehmen wir das mitgelieferte. Ein Bild laden wir mit LOADIMAGE und merken es uns in einer Variablen vom Typ IMAGE:"));
children.push(codeBlock([
  "DIM shipImg AS IMAGE",
  'shipImg = LOADIMAGE("../../assets/sprites/player.png")',
]));
children.push(pmix([
  ["Der Pfad ist ", false], ["relativ zu deiner Programmdatei", false],
  [". Liegt dein Code in ", false], ["code/kap02/", true],
  [" und das Bild in ", false], ["assets/sprites/", true],
  [", führen die zwei ", false], ["..", true],
  [" zwei Ordner nach oben. ", false],
  ["DIM", true], [" legt eine neue Variable an – hier vom Typ ", false],
  ["IMAGE", true], [" (ein Bild).", false],
]));

children.push(h2("Das Schiff zeichnen"));
children.push(p("Wo soll das Schiff stehen? Wir merken uns seine Position in zwei Variablen und starten unten in der Mitte. Gezeichnet wird mit DRAWIMAGE:"));
children.push(codeBlock([
  "DIM shipX AS INTEGER",
  "DIM shipY AS INTEGER",
  "shipX = 232",
  "shipY = 560",
  "",
  "' ... in der Spielschleife: ...",
  "DRAWIMAGE(shipImg, shipX, shipY)",
]));
children.push(pmix([
  ["", false],
  ["DRAWIMAGE(bild, x, y)", true],
  [" malt das Bild an die Position (x, y) – gemessen von der oberen linken Ecke. Bei einem 480 Pixel breiten Fenster und einem 16 Pixel breiten Schiff liegt die Mitte bei ", false],
  ["(480 - 16) / 2 = 232", true], [".", false],
]));

children.push(h2("Mit der Tastatur bewegen"));
children.push(pmix([
  ["Tasten fragen wir mit ", false], ["KEYPRESSED", true],
  [" ab. Ist eine Taste gedrückt, verschieben wir das Schiff – nach links kleiner, nach rechts grösser:", false],
]));
children.push(codeBlock([
  "IF KEYPRESSED(KEY_LEFT)  OR KEYPRESSED(KEY_A) THEN shipX = shipX - 4",
  "IF KEYPRESSED(KEY_RIGHT) OR KEYPRESSED(KEY_D) THEN shipX = shipX + 4",
]));
children.push(pmix([
  ["Mit ", false], ["OR", true],
  [" akzeptieren wir zwei Tasten gleichzeitig: die Pfeiltasten ", false],
  ["oder", false], [" A und D. Die ", false], ["4", true],
  [" ist die Geschwindigkeit – grösser = schneller.", false],
]));

children.push(h2("Am Rand anhalten"));
children.push(p("Ohne Grenzen würde das Schiff aus dem Bild fliegen. Zwei kurze Prüfungen halten es im Fenster:"));
children.push(codeBlock([
  "IF shipX < 0 THEN shipX = 0",
  "IF shipX > 464 THEN shipX = 464",
]));
children.push(pmix([
  ["", false],
  ["464", true], [" ist ", false], ["480 - 16", true],
  [" – so bleibt das ganze Schiff sichtbar, auch ganz rechts.", false],
]));

children.push(h2("Das ganze Programm"));
children.push(p("Zusammengesetzt – auf dem Sternenhimmel aus Kapitel 1:"));
children.push(codeBlock([
  'SCREEN(480, 640, "Mein Galaga - Kapitel 2")',
  "",
  "CONST NSTARS AS INTEGER = 60",
  "DIM starX[NSTARS] AS INTEGER",
  "DIM starY[NSTARS] AS INTEGER",
  "DIM i AS INTEGER",
  "FOR i = 0 TO NSTARS - 1",
  "    starX[i] = RANDINT(0, 479)",
  "    starY[i] = RANDINT(0, 639)",
  "NEXT i",
  "",
  "DIM shipImg AS IMAGE",
  'shipImg = LOADIMAGE("../../assets/sprites/player.png")',
  "DIM shipX AS INTEGER : DIM shipY AS INTEGER",
  "shipX = 232 : shipY = 560",
  "",
  "WHILE NOT QUITREQUESTED()",
  "    CLS(&H05060F)",
  "    FOR i = 0 TO NSTARS - 1",
  "        starY[i] = starY[i] + 2",
  "        IF starY[i] > 639 THEN starY[i] = 0 : starX[i] = RANDINT(0, 479)",
  "        PLOT(starX[i], starY[i], &HFFFFFF)",
  "    NEXT i",
  "    IF KEYPRESSED(KEY_LEFT)  OR KEYPRESSED(KEY_A) THEN shipX = shipX - 4",
  "    IF KEYPRESSED(KEY_RIGHT) OR KEYPRESSED(KEY_D) THEN shipX = shipX + 4",
  "    IF shipX < 0 THEN shipX = 0",
  "    IF shipX > 464 THEN shipX = 464",
  "    DRAWIMAGE(shipImg, shipX, shipY)",
  "    FLIP()",
  "WEND",
]));
figure("kap02_schiff.png", "Dein Raumschiff steht startbereit – mit den Pfeiltasten fliegt es nach links und rechts.", 300, 400).forEach(e => children.push(e));

children.push(h2("Was du gelernt hast"));
children.push(bulletRich("LOADIMAGE ", "lädt ein Bild in eine Variable vom Typ IMAGE (Pfad relativ zur Programmdatei)."));
children.push(bulletRich("DRAWIMAGE ", "zeichnet das Bild an einer Position (x, y)."));
children.push(bulletRich("KEYPRESSED ", "fragt Tasten ab; mit OR akzeptierst du mehrere Tasten."));
children.push(bulletRich("Grenzen mit IF ", "halten das Schiff im sichtbaren Bereich."));

children.push(h2("Übung"));
children.push(bullet("Mach das Schiff schneller oder langsamer (ändere die 4)."));
children.push(bullet("Erlaube auch Bewegung nach oben und unten (KEY_UP / KEY_DOWN, mit shipY) – und begrenze sie."));
children.push(bullet("Setze das Schiff an eine andere Startposition."));

// ===================== Dokument =====================
const doc = new Document({
  creator: "Hans Schnorrenberger",
  title: "GameBasic – Galaga-Buch",
  styles: {
    default: { document: { run: { font: "Arial", size: 22 } } },
    paragraphStyles: [
      { id: "Heading1", name: "Heading 1", basedOn: "Normal", next: "Normal", quickFormat: true,
        run: { size: 36, bold: true, font: "Arial", color: C_TITLE },
        paragraph: { spacing: { before: 320, after: 120 }, outlineLevel: 0 } },
      { id: "Heading2", name: "Heading 2", basedOn: "Normal", next: "Normal", quickFormat: true,
        run: { size: 28, bold: true, font: "Arial", color: C_H2 },
        paragraph: { spacing: { before: 200, after: 80 }, outlineLevel: 1 } },
    ],
  },
  numbering: { config: [
    { reference: "bul", levels: [{ level: 0, format: LevelFormat.BULLET, text: "•", alignment: AlignmentType.LEFT,
      style: { paragraph: { indent: { left: 540, hanging: 280 } } } }] },
  ] },
  sections: [{
    properties: { page: { size: { width: 12240, height: 15840 }, margin: { top: 1440, right: 1440, bottom: 1440, left: 1440 } } },
    footers: { default: new Footer({ children: [new Paragraph({
      alignment: AlignmentType.CENTER,
      children: [new TextRun({ text: "GameBasic – Galaga  ·  ", size: 16, color: C_CAP }),
                 new TextRun({ children: [PageNumber.CURRENT], size: 16, color: C_CAP })],
    })] }) },
    children,
  }],
});

Packer.toBuffer(doc).then(buf => {
  fs.writeFileSync(path.join(__dirname, "GameBasic-Buch.docx"), buf);
  console.log("OK -> GameBasic-Buch.docx");
});
