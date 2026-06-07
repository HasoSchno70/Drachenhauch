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
