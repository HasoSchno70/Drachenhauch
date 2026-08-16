// Baut das Drachenhauch-Tippspiel-Buch als farbiges, druckbares .docx.
// Aufruf:  node build_book.js   ->  Drachenhauch-Tippspiel.docx
//
// Aufbau wie beim Galaga-Buch (buch-galaga/buch/build_book.js): dieselben
// Bausteine, dieselbe Farbwelt -- damit beide Baende zusammen im Regal stehen
// koennen, ohne dass eines wie ein Fremdkoerper wirkt.
const fs = require("fs");
const path = require("path");
const {
  Document, Packer, Paragraph, TextRun, ImageRun, AlignmentType, HeadingLevel,
  BorderStyle, Table, TableRow, TableCell, WidthType, ShadingType, PageBreak,
  Header, Footer, PageNumber, LevelFormat,
  Tab, TabStopType, LeaderType,
} = require("docx");

const IMG = path.join(__dirname, "images");

// --- Farbpalette (wie im Galaga-Band) ---
const C_TITLE  = "1B6CA8";   // Blau (Titel/H1)
const C_H2     = "C0451B";   // Warmes Orange-Rot (H2)
const C_ACCENT = "1B6CA8";
const C_CAP    = "6A6A6A";   // Bildunterschrift grau
const C_TIPBG  = "E7F2FA";   // hellblauer Kasten
const C_TIPBD  = "9CC8E6";

// PNG-Groesse aus dem IHDR lesen (ohne Bildbibliothek).
function pngSize(file) {
  const b = fs.readFileSync(file);
  return { w: b.readUInt32BE(16), h: b.readUInt32BE(20) };
}

// Zentriertes Bild + Bildunterschrift, auf maxW/maxH skaliert.
function figure(name, caption, maxW = 540, maxH = 360) {
  const file = path.join(IMG, name);
  if (!fs.existsSync(file)) {
    // Fehlt ein Bild, soll der Build nicht abbrechen -- sondern es sagen.
    console.warn("  Bild fehlt:", name);
    return [];
  }
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

// Farbiger Tipp-Kasten.
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

// "Warum so?"-Kasten (Begruendung einer Entscheidung) -- amber statt blau.
function why(text, title = "Warum so?") {
  const border = { style: BorderStyle.SINGLE, size: 6, color: "E0B96A" };
  return new Table({
    width: { size: 9360, type: WidthType.DXA },
    columnWidths: [9360],
    rows: [new TableRow({ children: [new TableCell({
      width: { size: 9360, type: WidthType.DXA },
      borders: { top: border, bottom: border, left: border, right: border },
      shading: { fill: "FDF3E0", type: ShadingType.CLEAR },
      margins: { top: 120, bottom: 120, left: 160, right: 160 },
      children: [
        new Paragraph({ spacing: { after: 40 }, children: [new TextRun({ text: title, bold: true, color: "9A6A1E", size: 22 })] }),
        new Paragraph({ children: [new TextRun({ text, size: 22 })] }),
      ],
    })] })],
  });
}

// "Vorsicht"-Kasten fuer die Fallen, die man einmal im Leben tritt.
function warn(text, title = "Vorsicht") {
  const border = { style: BorderStyle.SINGLE, size: 6, color: "D89A9A" };
  return new Table({
    width: { size: 9360, type: WidthType.DXA },
    columnWidths: [9360],
    rows: [new TableRow({ children: [new TableCell({
      width: { size: 9360, type: WidthType.DXA },
      borders: { top: border, bottom: border, left: border, right: border },
      shading: { fill: "FBEDED", type: ShadingType.CLEAR },
      margins: { top: 120, bottom: 120, left: 160, right: 160 },
      children: [
        new Paragraph({ spacing: { after: 40 }, children: [new TextRun({ text: title, bold: true, color: "A33F3F", size: 22 })] }),
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

// Konsolen-Ausgabe: dunkler Kasten, damit man sie vom Quelltext unterscheidet.
function konsole(lines) {
  const runs = lines.map((ln, i) => new TextRun({
    text: ln, font: "Consolas", size: 18, color: "DDDDDD", break: i === 0 ? 0 : 1,
  }));
  return new Paragraph({
    shading: { fill: "26303C", type: ShadingType.CLEAR },
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

// Inhaltsverzeichnis-Registrierung (Seitenzahlen via toc_pages.json, s.u.).
const tocEntries = [];
let _bmCounter = 0;
function _heading(text, isChapter) {
  const bm = "toc_" + (_bmCounter++);
  tocEntries.push({ title: text, bm });
  return new Paragraph({
    heading: HeadingLevel.HEADING_1,
    ...(isChapter ? { pageBreakBefore: true } : {}),
    spacing: isChapter ? { before: 0, after: 80 } : { before: 320, after: 80 },
    border: { bottom: { style: BorderStyle.SINGLE, size: 12, color: C_ACCENT, space: 4 } },
    children: [new TextRun({ text })],
  });
}
function h1(text) { return _heading(text, false); }
function h2(text) {
  return new Paragraph({ heading: HeadingLevel.HEADING_2, children: [new TextRun({ text })] });
}
function chapter(text) { return _heading(text, true); }

// ===================== Inhalt =====================
const children = [];

// --- Titelseite ---
children.push(
  new Paragraph({ spacing: { before: 1200 }, children: [] }),
  new Paragraph({
    alignment: AlignmentType.CENTER,
    children: [new TextRun({ text: "DRACHENHAUCH", bold: true, color: C_TITLE, size: 88 })],
  }),
  new Paragraph({
    alignment: AlignmentType.CENTER, spacing: { before: 120, after: 80 },
    children: [new TextRun({ text: "Die erste richtige Anwendung", size: 32, color: C_H2, bold: true })],
  }),
  new Paragraph({
    alignment: AlignmentType.CENTER, spacing: { after: 300 },
    children: [new TextRun({ text: "ein Bundesliga-Tippspiel, Schritt für Schritt gebaut", size: 28, italics: true, color: C_CAP })],
  }),
);
figure("tippspiel_titel.png", "Das fertige Programm – unser Ziel in diesem Buch.", 660, 420).forEach(e => children.push(e));
children.push(
  new Paragraph({
    alignment: AlignmentType.CENTER, spacing: { before: 300 },
    children: [new TextRun({ text: "von Hans Schnorrenberger", size: 26, bold: true })],
  }),
  new Paragraph({ children: [new PageBreak()] }),
);

// Platzhalter fuer das Inhaltsverzeichnis (wird am Ende eingesetzt).
const TOC_INSERT_AT = children.length;

// --- Vorwort ---
children.push(h1("Vorwort"));
children.push(p("Es gibt einen Moment, den jeder kennt, der einmal ein kleines Spiel programmiert hat: Es läuft, es macht Spaß, man zeigt es stolz herum – und dann fragt jemand: „Kann das auch meine Daten behalten?“ Und plötzlich merkt man, dass zwischen einem Spiel und einem Programm, das man Menschen wirklich in die Hand geben kann, noch ein ganzes Stück Weg liegt."));
children.push(p("Ein Spiel darf beim Beenden alles vergessen. Es fängt beim nächsten Start fröhlich von vorne an, und das ist sogar erwünscht. Eine Anwendung darf das nicht. Wenn dein Tippspiel nach dem Schließen die Tipps deiner Freunde verloren hat, brauchst du es gar nicht erst zu öffnen – dann führt ihr die Runde weiter auf einem Zettel, so wie vorher."));
children.push(p("Genau dieser Unterschied ist der Grund für dieses Buch. Im Galaga-Band haben wir ein Arcade-Spiel gebaut und dabei gelernt, wie man Dinge bewegt, zeichnet und zum Klingen bringt. Hier bauen wir etwas, das bleibt: ein Bundesliga-Tippspiel mit Datenbank, mit Regeln, mit einer Rangliste, die auch morgen noch stimmt. Und mit einem Abruf aus dem Internet, der den Spielplan holt, ohne dass das Fenster einfriert."));
children.push(p("Ich verspreche dir dabei nichts Kompliziertes. Die Werkzeuge sind dieselben freundlichen wie im ersten Band, und die Sprache redet weiterhin in ganzen Sätzen mit dir. Was hinzukommt, ist eine andere Art zu denken – eine, die sich mit einer einzigen Frage zusammenfassen lässt: Wo leben die Daten, und wer darf sie ändern?"));
children.push(p("Wenn du diese Frage nach dem letzten Kapitel im Schlaf beantworten kannst, hast du mehr gelernt als jede Befehlsliste hergibt. Und ganz nebenbei hast du ein Programm, mit dem deine Tipprunde die nächste Saison bestreiten kann."));
children.push(new Paragraph({
  spacing: { before: 240 },
  children: [new TextRun({ text: "— Hans Schnorrenberger", italics: true, size: 22, color: C_CAP })],
}));

// --- Willkommen ---
children.push(h1("Willkommen!"));
children.push(p("Dieses Buch baut mit dir gemeinsam ein vollständiges Programm: ein Tippspiel für die Fußball-Bundesliga. Deine Mitspieler tippen die Ergebnisse des Spieltags, du trägst ein, wie es wirklich ausgegangen ist, und das Programm rechnet Punkte und Rangliste aus. Am Ende steht etwas, das man tatsächlich benutzen kann – nicht ein Beispiel, an dessen Ende man sich fragt, wo denn nun das eigentliche Programm geblieben ist."));
children.push(p("Vorkenntnisse brauchst du keine. Wenn du weißt, wie man einen Computer einschaltet und Text eintippt, bist du qualifiziert. Wer den Galaga-Band gelesen hat, kennt das Fenster und die Schleife schon – dann gehen die ersten Kapitel besonders flott."));
children.push(p("Wir gehen dabei einen ungewöhnlichen Weg: Die ersten Kapitel haben noch gar kein Fenster. Zuerst kommen die Daten, dann die Regel, und erst dann die Oberfläche. Das fühlt sich zunächst falsch an – man will doch etwas sehen! – zahlt sich aber ab Kapitel 4 in jeder Minute aus."));
children.push(tip("So liest du dieses Buch", "Tippe die Beispiele wirklich selbst ab und starte sie sofort. Jeder Kapitelstand ist ein vollständiges, lauffähiges Programm; du kannst jederzeit einsteigen. Und keine Sorge vor Fehlern: Drachenhauch sagt dir in ganzen Sätzen, was es nicht verstanden hat."));

// --- Spiel vs Anwendung ---
children.push(h1("Was eine Anwendung von einem Spiel unterscheidet"));
children.push(p("Auf den ersten Blick nicht viel: beides sind Programme, beide haben ein Fenster, beide reagieren auf Klicks. Der Unterschied liegt woanders – nämlich in dem, was übrig bleibt, wenn man sie schließt."));
children.push(bulletRich("Ein Spiel lebt im Augenblick. ", "Position des Raumschiffs, Anzahl der Gegner, aktueller Punktestand – alles davon existiert nur, solange das Spiel läuft. Ein Highscore ist die berühmte Ausnahme, die die Regel bestätigt."));
children.push(bulletRich("Eine Anwendung lebt von ihren Daten. ", "Die Tipps, die Ergebnisse, die Mitspieler – das alles muss den Programmstart überleben, den Rechnerneustart und im Zweifel auch einen Absturz mitten im Speichern."));
children.push(p("Daraus folgt fast alles, was in diesem Buch anders ist als im Galaga-Band. Wir brauchen einen Ort für die Daten, der zuverlässiger ist als eine Variable: eine Datenbank. Wir brauchen Regeln, die an genau einer Stelle stehen, damit sie nicht auseinanderlaufen. Wir brauchen ein Programm, das nicht abstürzt, wenn das Internet mal weg ist. Und wir brauchen eine Sicherung, denn Wahrheit, die es nur einmal gibt, ist eine Wette."));
children.push(why("Weil ein Spiel Fehler verzeiht und eine Anwendung nicht. Wenn ein Gegner in Galaga einmal falsch fliegt, ärgert man sich zehn Sekunden lang. Wenn ein Tippspiel eine falsche Rangliste zeigt, glaubt ihm niemand mehr – auch nicht, wenn es später wieder stimmt. Vertrauen verliert man schneller, als man es zurückgewinnt.", "Warum so viel Aufwand für eine Tipprunde?"));

// --- Unser Projekt ---
children.push(h1("Unser Projekt: das Tippspiel"));
children.push(p("Das Prinzip kennt jeder, der schon einmal in einer Büro-Tipprunde mitgespielt hat. Vor dem Spieltag tippt jeder die Ergebnisse. Danach wird verglichen, und es gibt Punkte:"));
children.push(bulletRich("3 Punkte ", "für das exakte Ergebnis."));
children.push(bulletRich("2 Punkte ", "für die richtige Tordifferenz bei richtigem Sieger."));
children.push(bulletRich("1 Punkt ", "für den richtigen Sieger (oder ein Unentschieden)."));
children.push(bulletRich("0 Punkte ", "für alles andere."));
children.push(p("Das klingt nach wenig – und ist genau die richtige Größe. Klein genug, dass wir es in dreizehn Kapiteln vollständig bauen. Groß genug, dass jede Frage vorkommt, die eine echte Anwendung stellt: Wie speichere ich? Wie zeige ich Daten an, ohne sie zu verfälschen? Was passiert, wenn zwei Dinge gleichzeitig geschehen? Wie hole ich Daten aus dem Netz? Was tue ich, wenn etwas schiefgeht?"));
figure("tippspiel_titel.png", "Der Zielstand: Anstoßzeiten, Tipps, Ergebnisse und Punkte auf einen Blick.", 640, 400).forEach(e => children.push(e));
children.push(p("Und weil man es sich sonst nicht vorstellen kann, hier schon einmal alles, was am Ende drin ist:"));
children.push(bullet("Eine SQLite-Datenbank, die das Programm beim ersten Start selbst anlegt"));
children.push(bullet("Tipp-Eingabe je Mitspieler, mit einem Tipp je Spieler und Spiel"));
children.push(bullet("Ergebnis-Eingabe, die alle Punkte des Spiels neu berechnet"));
children.push(bullet("Rangliste in einem zweiten Reiter, gerechnet von der Datenbank"));
children.push(bullet("Anstoßzeiten mit Tippschluss und laufendem Countdown"));
children.push(bullet("Spielplan aus dem Internet – ohne dass das Fenster stehenbleibt"));
children.push(bullet("Sicherung, Wiederherstellung und ein Umbau der Datenbank im laufenden Betrieb"));
children.push(bullet("Ein Balkendiagramm der Punkte und eine .exe zum Weitergeben"));

// --- Aufbau ---
children.push(h1("Wie dieses Buch funktioniert"));
children.push(p("Wir bauen in kleinen, lauffähigen Schritten. Jedes Kapitel hat ein Ziel, das man sehen kann, und endet mit einem Programm, das läuft. Nichts ist demotivierender, als zweihundert Zeilen abzutippen und dann eine Fehlermeldung zu ernten, ohne zu wissen, welche der zweihundert Zeilen schuld ist."));
children.push(p("Der Fahrplan:"));
children.push(bulletRich("Kapitel 1–3 – das Fundament: ", "ein Fenster, eine Datenbank, eine Regel. Zwei davon ohne Fenster."));
children.push(bulletRich("Kapitel 4–7 – die Anwendung: ", "Spielplan anzeigen, tippen, werten, Rangliste."));
children.push(bulletRich("Kapitel 8–9 – die Außenwelt: ", "Zeit und Daten aus dem Internet."));
children.push(bulletRich("Kapitel 10–11 – die Wirklichkeit: ", "Fehler, Rückfragen, Sicherung, Umbau."));
children.push(bulletRich("Kapitel 12–13 – der Abschluss: ", "Politur und Weitergeben."));
children.push(tip("Die Kapitelstände liegen bei", "Zu jedem Kapitel gibt es den vollständigen Quelltext im Ordner buch-tippspiel/code/kapNN/. Wenn du irgendwo steckenbleibst, vergleiche einfach – oder starte von dort aus weiter."));

// ===================== Kapitel 1 =====================
children.push(chapter("Kapitel 1: Das erste Fenster"));
children.push(tip("In diesem Kapitel",
  "Du öffnest ein Fenster, lernst die Schleife kennen, in der jedes Programm mit Oberfläche lebt – und drückst einen Knopf, der etwas tut."));

children.push(h2("Ein Fenster und ein Rahmen darin"));
children.push(p("Ein Programm mit Oberfläche braucht zweierlei: ein Fenster des Betriebssystems und darin eine Fläche, auf der die Bedienelemente sitzen. In Drachenhauch sind das zwei Zeilen, und der Unterschied zwischen beiden ist wichtig genug, um ihn einmal auszusprechen."));
children.push(codeBlock([
  'SCREEN(700, 400, "Tippspiel - Kapitel 1")',
  'DIM win AS GUI_WINDOW',
  'win = GUI_WINDOW("Bundesliga Tippspiel", 20, 20, 660, 360)',
]));
children.push(pmix([
  ["", false], ["SCREEN", true], [" ist die Leinwand: das Fenster, das in der Taskleiste auftaucht. ", false],
  ["GUI_WINDOW", true], [" ist ein Rahmen darauf, in den Knöpfe, Tabellen und Textfelder gehören. Alles, was du später anlegst, bekommt diesen Rahmen als erstes Argument mit – so weiß jedes Bedienelement, wohin es gehört.", false],
]));

children.push(h2("Die Schleife"));
children.push(p("Jetzt die wichtigste Idee, die dieses Buch von der ersten bis zur letzten Seite trägt. Ein Programm mit Fenster steht nie still. Selbst wenn scheinbar nichts passiert, arbeitet es pausenlos weiter: Es sieht nach, wo die Maus ist, ob jemand geklickt hat, und zeichnet dann alles neu – sechzig Mal in der Sekunde."));
children.push(codeBlock([
  "WHILE NOT QUITREQUESTED()",
  "    IF KEYPRESSED(27) THEN BREAK        ' 27 = ESC",
  "    GUI_UPDATE()",
  "",
  "    IF GUI_CLICKED(knopf) THEN",
  "        gedrueckt = gedrueckt + 1",
  '        GUI_SET_TEXT(stand, "Gedrueckt: " + STR$(gedrueckt) + " mal")',
  "    END IF",
  "",
  "    CLS(&H0F1420)",
  "    GUI_DRAW()",
  "    FLIP()",
  "WEND",
]));
children.push(p("Diese Reihenfolge ist keine Geschmacksfrage. Würde man zuerst zeichnen und dann nachsehen, was passiert ist, hinkte das Bild immer einen Durchlauf hinterher – man klickt, und der Knopf reagiert erst beim nächsten Mal. Das fühlt sich für den Benutzer nicht nach „langsam“ an, sondern nach „kaputt“."));
children.push(pmix([
  ["Und noch etwas steckt in dieser Reihenfolge: ", false], ["CLS", true],
  [" löscht das Bild, bevor neu gezeichnet wird. Ohne diese Zeile stapeln sich die Zeichnungen aufeinander, und nach einer Minute sieht das Fenster aus wie eine Tafel, die nie gewischt wurde. Gezeichnet wird dabei nicht direkt auf den Bildschirm, sondern in ein unsichtbares zweites Bild; erst ", false],
  ["FLIP", true], [" zeigt es. So sieht niemand das halbfertige Bild – es gibt kein Flackern.", false],
]));
children.push(pmix([
  ["Beachte, dass ", false], ["GUI_CLICKED", true],
  [" nicht fragt „ist der Knopf gerade gedrückt?“, sondern „wurde er in diesem Bild angeklickt?“. Deshalb zählt der Zähler pro Klick um eins hoch und nicht um sechzig, solange man die Maustaste hält.", false],
]));

children.push(h2("Eine Schrift, die nach diesem Jahrtausend aussieht"));
children.push(p("Wenn du das Programm jetzt startest, fällt dir vermutlich die Schrift auf: eine Pixelschrift, die aussieht wie von einem Heimcomputer. Für ein Spiel ist das genau richtig. In einer Anwendung, die Vereinsnamen und Uhrzeiten anzeigt, wirkt sie fehl am Platz."));
children.push(codeBlock([
  "DIM schrift AS INTEGER : schrift = -1",
  "DIM pfad AS STRING",
  'FOR EACH pfad IN ("C:/Windows/Fonts/segoeui.ttf", "C:/Windows/Fonts/arial.ttf")',
  "    IF schrift < 0 AND FILEEXISTS(pfad) THEN schrift = LOADFONT(pfad, 32)",
  "NEXT",
  "IF schrift >= 0 THEN SETFONT(schrift)",
  "TEXT_SIZE(16)",
]));
children.push(pmix([
  ["Erst ", false], ["FILEEXISTS", true],
  [" fragen, dann laden: auf einem anderen Rechner heißt die Schriftdatei anders, und daran soll das Programm nicht scheitern. Geladen wird sie groß (32), gezeichnet in Lesegröße (16) – so bleiben die Buchstaben scharf, auch wenn man sie später vergrößert.", false],
]));
children.push(p("Das ist übrigens ein Muster, das dir in diesem Buch noch oft begegnet: Ein Programm, das für andere Leute gedacht ist, darf nie davon ausgehen, dass ihr Rechner aussieht wie deiner. Die Schrift kann fehlen, das Internet kann weg sein, die Datenbank kann von einer älteren Fassung stammen. Jedes Mal ist die Antwort dieselbe – nachsehen, und einen sinnvollen zweiten Weg haben."));
children.push(h2("Was ein Widget ist"));
children.push(pmix([
  ["Jedes Bedienelement, das du anlegst, gibt dir eine Nummer zurück – ein sogenanntes Handle. Wir merken sie uns in einer Variablen vom Typ ", false],
  ["GUI_WIDGET", true], [" und sprechen das Element später über sie an:", false],
]));
children.push(codeBlock([
  "DIM stand AS GUI_WIDGET",
  'stand = GUI_LABEL(win, "Noch tut der Knopf nicht viel.", 20, 80)',
  "' ... spaeter, irgendwo in der Schleife:",
  'GUI_SET_TEXT(stand, "Gedrueckt: 3 mal")',
]));
children.push(p("Man legt ein Element also einmal an und ändert danach nur noch seinen Inhalt. Es wäre ein verbreiteter Anfängerfehler, in der Schleife jedes Mal ein neues Label zu erzeugen – nach zehn Sekunden hätte das Fenster sechshundert davon, alle übereinander."));
children.push(why("Weil die Schleife sechzigmal pro Sekunde läuft und alles, was man darin anlegt, sechzigmal pro Sekunde entsteht. Anlegen gehört vor die Schleife, Ändern hinein. Diese Trennung zieht sich durch jedes Kapitel dieses Buchs: Aufbau oben, Reaktion in der Schleife.", "Warum Elemente vor der Schleife anlegen?"));
figure("kap01_fenster.png", "Kapitel 1: ein Fenster, ein Knopf, ein Zähler.", 600, 375).forEach(e => children.push(e));

// ===================== Kapitel 2 =====================
children.push(chapter("Kapitel 2: Wo die Daten wohnen"));
children.push(tip("In diesem Kapitel",
  "Kein Fenster. Stattdessen die wichtigste Entscheidung des ganzen Buchs: wo die Daten liegen – und wie man sie hineinbekommt und wieder heraus."));

children.push(p("Ich weiß, du willst etwas sehen. Aber genau hier trennt sich eine Anwendung von einem Programm, das nur so aussieht wie eine. Wer zuerst die Oberfläche baut und die Daten später dazufrickelt, baut die Regeln in die Knöpfe – und hat am Ende drei Stellen, an denen entschieden wird, was ein Tipp wert ist."));

children.push(h2("Eine Datenbank ist eine Datei"));
children.push(p("Drachenhauch bringt SQLite mit: eine vollständige Datenbank, die in einer einzigen Datei lebt. Kein Server, keine Installation, kein Passwort. Man öffnet sie wie eine Datei – und wenn es sie noch nicht gibt, entsteht sie."));
children.push(codeBlock([
  'IMPORT "db"',
  'CONST DATEI = "kap02.db"',
  "DIM db AS DB_CONN",
  "db = DB_OPEN(DATEI)",
]));
children.push(p("Ab diesem Augenblick ist diese Datei die Wahrheit. Nicht irgendeine Variable im Programm, nicht das, was gerade auf dem Bildschirm steht – die Datei. Diesen Satz werden wir im ganzen Buch nicht mehr los, und das ist Absicht."));
children.push(why("Man könnte die Tipps auch in eine Textdatei schreiben, eine Zeile je Tipp. Das geht – bis zur ersten Frage, die etwas komplizierter ist: „Wie viele Punkte hat Ben aus den Spielen, die schon gelaufen sind?“ In einer Textdatei heißt das: alles einlesen, selbst durchsuchen, selbst zusammenzählen, selbst sortieren. In einer Datenbank ist es eine Frage in einer Zeile. Und wenn mitten im Schreiben der Strom ausfällt, ist die Textdatei halb geschrieben, die Datenbank nicht.", "Warum eine Datenbank und keine Textdatei?"));

children.push(h2("Tabellen anlegen"));
children.push(codeBlock([
  'DB_EXEC(db, "CREATE TABLE IF NOT EXISTS spieler (" + _',
  '            "id INTEGER PRIMARY KEY, name TEXT NOT NULL)")',
  "",
  'DB_EXEC(db, "CREATE TABLE IF NOT EXISTS spiele (" + _',
  '            "id INTEGER PRIMARY KEY, spieltag INTEGER NOT NULL, " + _',
  '            "heim TEXT NOT NULL, gast TEXT NOT NULL, " + _',
  '            "tore_heim INTEGER, tore_gast INTEGER)")',
]));
children.push(pmix([
  ["Das ", false], ["IF NOT EXISTS", true],
  [" ist kein Schmuck: ohne es bräche das Programm beim zweiten Start ab, weil es die Tabelle schon gibt. Mit ihm darf man es beliebig oft starten – und genau das ist der Grund, warum eine ausgelieferte Anwendung sich beim Empfänger selbst einrichten kann.", false],
]));
children.push(p("Und ein Detail, das später wichtig wird: tore_heim und tore_gast dürfen leer bleiben. Ein Spiel, das noch nicht gespielt ist, hat kein Ergebnis. Das ist etwas anderes als 0:0 – und die Datenbank kennt diesen Unterschied. Sie nennt ihn NULL."));
children.push(warn("Verwechsle NULL nicht mit 0. Ein Spiel mit 0:0 ist ausgegangen wie es ausgegangen ist. Ein Spiel mit NULL ist noch nicht gespielt. Wer beides gleich behandelt, verteilt Punkte für Spiele, die noch gar nicht stattgefunden haben – und das fällt erst auf, wenn sich jemand beschwert."));
children.push(h2("Drei Tabellen, und warum es genau drei sind"));
children.push(p("Unser Datenmodell besteht am Ende aus drei Tabellen. Es lohnt sich, kurz innezuhalten und zu schauen, warum die Aufteilung so ist:"));
children.push(bulletRich("spieler – ", "wer mitmacht. Ein Eintrag je Person, mit einer id, die sich nie ändert. Auch wenn Anna heiratet und anders heißt, bleibt sie dieselbe Spielerin."));
children.push(bulletRich("spiele – ", "was gespielt wird. Paarung, Spieltag, später Anstoßzeit, und das Ergebnis – solange es keines gibt, bleibt es leer."));
children.push(bulletRich("tipps – ", "wer was auf welches Spiel getippt hat. Diese Tabelle verbindet die beiden anderen: sie enthält eine spieler_id und eine spiel_id."));
children.push(p("Diese dritte Tabelle ist das eigentlich Interessante. Man könnte auf die Idee kommen, die Tipps in die Spieler-Tabelle zu schreiben – eine Spalte je Spiel. Das funktioniert genau so lange, bis ein Spiel dazukommt. Dann müsste man die Tabelle umbauen, für jeden neuen Spieltag aufs Neue."));
children.push(why("Weil eine Beziehung zwischen zwei Dingen selbst ein Ding ist. „Anna tippt auf Spiel 3 ein 2:1“ ist eine Tatsache mit drei Bestandteilen – Spieler, Spiel, Tipp – und die gehört in eine eigene Zeile. Kommt ein Spieler dazu, wächst die Tabelle nach unten. Kommt ein Spiel dazu, ebenfalls. Nichts muss umgebaut werden.", "Warum eine eigene Tabelle für Tipps?"));

children.push(h2("Daten hinein"));
children.push(codeBlock([
  "DB_BEGIN(db)",
  "DIM name AS STRING",
  'FOR EACH name IN ("Anna", "Ben", "Clara")',
  '    DB_EXEC(db, "INSERT INTO spieler (name) VALUES (?)", name)',
  "NEXT",
  "DB_COMMIT(db)",
]));
children.push(pmix([
  ["Zwei Dinge stecken hier drin. Erstens ", false], ["DB_BEGIN", true], [" und ", false], ["DB_COMMIT", true],
  [": alles dazwischen gilt als ein einziger Vorgang. Bricht das Programm mittendrin ab, ist nichts halb eingetragen. Zweitens das Fragezeichen – ein Platzhalter, in den die Datenbank den Wert selbst einsetzt.", false],
]));
children.push(why("Weil ein Name wie O’Brien den Befehl sonst zerreißen würde – das Hochkomma beendet die Zeichenkette mitten im Wort. Und weil jemand, der einen Namen eintippen darf, sonst auch Befehle eintippen könnte. Mit dem Fragezeichen ist ein Wert immer ein Wert und nie ein Befehl.", "Warum nicht einfach den Text zusammenkleben?"));

children.push(h2("Und wieder heraus"));
children.push(codeBlock([
  "DIM r AS DB_RESULT",
  'r = DB_QUERY(db, "SELECT id, heim, gast, tore_heim, tore_gast " + _',
  '                 "FROM spiele WHERE spieltag = ? ORDER BY id", 1)',
  "WHILE DB_NEXT(r)",
  '    DIM ergebnis AS STRING : ergebnis = "noch nicht gespielt"',
  "    IF NOT DB_IS_NULL(r, 3) THEN",
  '        ergebnis = STR$(DB_GET_INT(r, 3)) + ":" + STR$(DB_GET_INT(r, 4))',
  "    END IF",
  '    PRINT "  ["; DB_GET_INT(r, 0); "] "; DB_GET_STRING(r, 1); _',
  '          " - "; DB_GET_STRING(r, 2); "   "; ergebnis',
  "WEND",
  "DB_CLOSE_RESULT(r)",
]));
children.push(p("Gestartet sieht das so aus:"));
children.push(konsole([
  "Spiele des 1. Spieltags:",
  "  [1] FC Bayern München - VfB Stuttgart   noch nicht gespielt",
  "  [2] Borussia Dortmund - Hamburger SV   noch nicht gespielt",
  "  [3] RB Leipzig - Bor. Mönchengladbach   noch nicht gespielt",
]));
children.push(pmix([
  ["", false], ["DB_CLOSE_RESULT", true],
  [" nicht vergessen – ein offenes Ergebnis blockiert die Tabelle. Wir werden in Kapitel 6 sehen, was passiert, wenn man das ignoriert und mitten im Lesen schreiben will.", false],
]));
children.push(h2("Der Primärschlüssel"));
children.push(pmix([
  ["Jede unserer Tabellen beginnt mit ", false], ["id INTEGER PRIMARY KEY", true],
  [". Das ist eine Nummer, die die Datenbank selbst vergibt und die eine Zeile für immer eindeutig macht. Man braucht sie nicht einzutragen – wer eine Zeile einfügt, ohne eine id zu nennen, bekommt die nächste freie.", false],
]));
children.push(p("Diese Nummer ist wichtiger, als sie aussieht. Sie ist das, worauf sich alles andere bezieht: Ein Tipp verweist nicht auf „das Spiel Bayern gegen Stuttgart“, sondern auf Spiel Nummer 1. Wenn sich später herausstellt, dass der Verein in der Datenbank falsch geschrieben war, ändert man den Namen – und alle Tipps zeigen weiterhin auf dasselbe Spiel."));
children.push(tip("Zum Ausprobieren", "Starte das Programm zweimal. Beim zweiten Mal wird nichts neu angelegt – und das Ergebnis, das es beim ersten Mal eingetragen hat, steht noch drin. Genau das ist der Unterschied zu einem Spiel. Lösche danach kap02.db und starte erneut: alles fängt von vorne an, und die Datei entsteht neu."));

// ===================== Kapitel 3 =====================
children.push(chapter("Kapitel 3: Die Regel"));
children.push(tip("In diesem Kapitel",
  "Die eine Funktion, um die sich alles dreht – und ein Programm, das seine eigene Regel nachrechnet, damit sie auch in einem Jahr noch stimmt."));

children.push(p("Ein Tippspiel ist im Kern eine einzige Frage: Wie viele Punkte gibt dieser Tipp? Wir beantworten sie jetzt – bevor es ein Fenster gibt, in dem man die Antwort verstecken könnte."));
children.push(codeBlock([
  "FUNCTION tendenz(heim AS INTEGER, gast AS INTEGER) AS INTEGER",
  "    IF heim > gast THEN RETURN 1",
  "    IF heim < gast THEN RETURN -1",
  "    RETURN 0",
  "END FUNCTION",
  "",
  "FUNCTION punkte(tippH AS INTEGER, tippA AS INTEGER, _",
  "                ergH AS INTEGER, ergA AS INTEGER) AS INTEGER",
  "    IF tippH = ergH AND tippA = ergA THEN RETURN 3",
  "    IF tendenz(tippH, tippA) <> tendenz(ergH, ergA) THEN RETURN 0",
  "    IF (tippH - tippA) = (ergH - ergA) THEN RETURN 2",
  "    RETURN 1",
  "END FUNCTION",
]));
children.push(p("Die Reihenfolge ist die Regel: Erst wird das Genaueste geprüft, dann das Gröbste. Wer die mittleren beiden Zeilen vertauscht, hat ein anderes Spiel – und merkt es womöglich monatelang nicht."));
children.push(why("Weil eine Regel, die in drei Knöpfen steckt, irgendwann drei verschiedene Antworten gibt. Sobald jemand die Punkte ändern will – und irgendwann will das jemand – ist eine einzige Stelle der Unterschied zwischen fünf Minuten und einem verdorbenen Abend.", "Warum steht die Regel in einer eigenen Funktion?"));

children.push(h2("Ein Programm, das sich selbst prüft"));
children.push(p("Und jetzt der Teil, der eigentlich wichtiger ist als die Regel selbst. Eine Regel im Kopf durchzugehen ist mühsam und unzuverlässig. Ein Programm, das seine eigene Regel nachrechnet, sagt in einer Sekunde, ob sie noch stimmt – heute, und nach dem nächsten Umbau auch."));
children.push(codeBlock([
  "SUB pruefe(was AS STRING, ist AS INTEGER, soll AS INTEGER)",
  "    IF ist = soll THEN",
  '        PRINT "  ok     "; was; " = "; ist',
  "    ELSE",
  '        PRINT "  FEHLER "; was; ": ist "; ist; ", soll "; soll',
  "        fehler = fehler + 1",
  "    END IF",
  "END SUB",
  "",
  'pruefe("3:1 getippt, 3:1 gespielt", punkte(3, 1, 3, 1), 3)',
  'pruefe("2:0 getippt, 3:1 gespielt", punkte(2, 0, 3, 1), 2)',
  'pruefe("1:0 getippt, 3:1 gespielt", punkte(1, 0, 3, 1), 1)',
  'pruefe("0:1 getippt, 3:1 gespielt", punkte(0, 1, 3, 1), 0)',
]));
children.push(konsole([
  "=== Die Punkteregel ===",
  "  ok     3:1 getippt, 3:1 gespielt = 3",
  "  ok     2:0 getippt, 3:1 gespielt = 2",
  "  ok     1:0 getippt, 3:1 gespielt = 1",
  "  ok     0:1 getippt, 3:1 gespielt = 0",
  "",
  "ALLES GRUEN",
]));

children.push(h2("Was das Nachrechnen zutage fördert"));
children.push(p("Beim Schreiben dieser Prüfungen ist mir selbst etwas aufgefallen, das ich beim Lesen der Regel übersehen hatte: Zwei Unentschieden haben immer dieselbe Tordifferenz, nämlich null. Wer also ein Unentschieden tippt und es kommt eines, bekommt nie einen Punkt, sondern mindestens zwei."));
children.push(codeBlock([
  'pruefe("1:1 getippt, 2:2 gespielt", punkte(1, 1, 2, 2), 2)',
  'pruefe("0:0 getippt, 3:3 gespielt", punkte(0, 0, 3, 3), 2)',
]));
children.push(p("Ist das gewollt? Das darfst du entscheiden. Wichtig ist etwas anderes: Es steht jetzt schwarz auf weiß da, statt sich in einem Nebensatz der Regel zu verstecken. Genau dafür sind solche Prüfungen da – sie zwingen einen, die eigene Regel wirklich zu Ende zu denken."));
children.push(p("Noch schärfer wird es, wenn man eine Eigenschaft prüft statt einzelner Fälle. Die Punkteregel muss symmetrisch sein: Dreht man bei Tipp und Ergebnis jeweils Heim und Gast um, darf sich nichts ändern."));
children.push(codeBlock([
  "FOR a = 0 TO 4",
  "  FOR b = 0 TO 4",
  "    FOR c = 0 TO 4",
  "      FOR d = 0 TO 4",
  "        IF punkte(a, b, c, d) <> punkte(b, a, d, c) THEN",
  "            abweichungen = abweichungen + 1",
  "        END IF",
  "      NEXT",
  "    NEXT",
  "  NEXT",
  "NEXT",
]));
children.push(p("625 Kombinationen, eine einzige Prüfung – und sie findet einen vertauschten Wert in punkte(), den kein Beispiel je erwischt hätte."));
children.push(h2("Was einen guten Prüffall ausmacht"));
children.push(p("Prüfungen zu schreiben ist eine Fertigkeit für sich, und man lernt sie am schnellsten an einer Faustregel: Prüfe nicht, was offensichtlich stimmt, sondern die Ränder und die Fälle, bei denen du selbst kurz nachdenken musstest."));
children.push(bulletRich("Jeder Rückgabewert mindestens einmal: ", "3, 2, 1 und 0 – wenn eine Punktzahl in keiner Prüfung vorkommt, könnte die Zeile, die sie erzeugt, gelöscht werden, ohne dass es jemand merkt."));
children.push(bulletRich("Die Grenzen zwischen ihnen: ", "der Übergang von 2 zu 1 ist die interessanteste Stelle der ganzen Regel, denn dort steht die Tordifferenz."));
children.push(bulletRich("Die Fälle, bei denen du gezögert hast: ", "Unentschieden auf Unentschieden war so einer. Genau die vergisst man beim Umbauen zuerst."));
children.push(bulletRich("Eine Eigenschaft statt eines Beispiels: ", "Symmetrie, Ober- und Untergrenzen, „nie negativ“ – solche Prüfungen decken tausende Fälle auf einmal ab."));
children.push(p("Und eine Regel, die man sich merken sollte: Eine Prüfung, die nie fehlschlägt, hat noch nichts bewiesen. Wer wissen will, ob seine Prüfungen etwas taugen, ändert einmal absichtlich eine Zeile in der Regel und schaut, ob es auffällt. Fällt nichts auf, prüft man die falschen Dinge."));
children.push(tip("Zum Ausprobieren", "Ändere in punkte() das RETURN 2 zu RETURN 1 und starte das Prüfprogramm. Wie viele Prüfungen schlagen an? Mach es rückgängig und vertausche stattdessen die beiden mittleren Zeilen. Und zum Schluss: Was passiert, wenn du in tendenz() das 1 und das -1 tauschst? (Die Symmetrie-Prüfung bleibt still – sie prüft eine andere Eigenschaft. Auch das ist eine Lektion.)"));

// ===================== Kapitel 4 =====================
children.push(chapter("Kapitel 4: Der Spielplan im Fenster"));
children.push(tip("In diesem Kapitel",
  "Datenbank und Fenster treffen sich: eine Tabelle zeigt, was gespeichert ist. Und du lernst den Satz, an dem sich später alles entscheidet."));

children.push(p("Jetzt kommt zusammen, was in den letzten beiden Kapiteln entstanden ist. Und mit dem ersten Blick auf eine echte Tabelle kommt auch der Satz, den ich dir am liebsten in großen Buchstaben an die Wand schreiben würde:"));
children.push(new Paragraph({
  alignment: AlignmentType.CENTER, spacing: { before: 60, after: 160 },
  children: [new TextRun({ text: "Die Datenbank ist die Wahrheit. Die Tabelle ist nur ihre Ansicht.", bold: true, size: 26, color: C_H2 })],
}));
children.push(p("Das klingt nach Philosophie, hat aber sehr handfeste Folgen. Eine Tabelle auf dem Bildschirm kennt nur Zeilennummern: Zeile 0, Zeile 1, Zeile 2. Die Datenbank kennt nur ids. Sobald jemand die Tabelle sortiert oder filtert, passen diese beiden Nummern nicht mehr zusammen – es sei denn, wir merken uns die Zuordnung."));
children.push(codeBlock([
  "CONST MAXSPIELE = 200",
  "DIM spielId[MAXSPIELE] AS INTEGER",
  "DIM spieleAnzahl AS INTEGER",
  "",
  "SUB spieleLaden()",
  "    GUI_TABLE_CLEAR(tabelle)",
  "    spieleAnzahl = 0",
  "    ' ... Zeile für Zeile aus der Datenbank lesen ...",
  "        GUI_TABLE_ADD_ROW(tabelle, zeile)",
  "        spielId[spieleAnzahl] = DB_GET_INT(r, 0)   ' die Brücke",
  "        spieleAnzahl = spieleAnzahl + 1",
  "END SUB",
]));
children.push(pmix([
  ["Diese eine Zeile – ", false], ["spielId[spieleAnzahl] = DB_GET_INT(r, 0)", true],
  [" – ist die Brücke zwischen beiden Welten. Ohne sie könnte das Programm auf einen Klick nicht antworten, denn es wüsste nicht, welches Spiel gemeint ist.", false],
]));
figure("kap04_spielplan.png", "Kapitel 4: der Spielplan aus der Datenbank, mit einem bereits eingetragenen Ergebnis.", 640, 380).forEach(e => children.push(e));
children.push(warn("Der verlockende Fehler lautet: „Zeile 0 ist Spiel 1, Zeile 1 ist Spiel 2 …“. Solange niemand sortiert, geht das gut. Genau deshalb ist dieser Fehler so gemein – er wartet, bis ein Benutzer zum ersten Mal auf einen Spaltenkopf klickt, und trägt dann das Ergebnis beim falschen Spiel ein."));
children.push(h2("Eine Tabelle einrichten"));
children.push(p("Eine Tabelle ist das Arbeitstier jeder Anwendung. Sie wird einmal eingerichtet und danach nur noch gefüllt:"));
children.push(codeBlock([
  "tabelle = GUI_TABLE(win, 20, 50, 740, 280)",
  "DIM kopf AS ARRAY OF STRING",
  'kopf = SPLIT$("Heim|Gast|Ergebnis", "|")',
  "GUI_TABLE_HEADERS(tabelle, kopf)",
  "DIM breiten AS ARRAY OF INTEGER",
  "breiten = [300, 300, 140]",
  "GUI_TABLE_COL_WIDTHS(tabelle, breiten)",
  'GUI_TABLE_SET(tabelle, "zeilenhoehe", 30)',
  'GUI_TABLE_SET(tabelle, "zebra", 1)',
  'GUI_TABLE_COL_ALIGN(tabelle, 2, "mitte")',
]));
children.push(p("Die Spaltenbreiten sind kein Schönheitsthema: Vereinsnamen sind lang, Ergebnisse sind kurz. Wer allen Spalten dieselbe Breite gibt, bekommt abgeschnittene Namen neben viel Luft. Und das Zebra-Muster – jede zweite Zeile etwas dunkler – ist der billigste Lesbarkeitsgewinn, den es gibt: Das Auge verrutscht nicht mehr zwischen den Zeilen."));
children.push(h2("Neu laden statt nachpflegen"));
children.push(pmix([
  ["Beachte, wie ", false], ["spieleLaden", true],
  [" arbeitet: Es leert die Tabelle komplett und baut sie neu auf. Nicht „die geänderte Zeile suchen und dort den neuen Wert eintragen“, sondern alles verwerfen und noch einmal fragen.", false],
]));
children.push(why("Weil eine einzeln nachgepflegte Anzeige irgendwann von der Wahrheit abweicht – und man nicht merkt, wann. Neu laden ist ein paar Millisekunden langsamer und dafür immer richtig. Erst wenn eine Tabelle zehntausend Zeilen hat, lohnt sich das Nachdenken über Feineres. Unsere hat neun.", "Warum die ganze Tabelle neu aufbauen?"));
children.push(tip("Zum Ausprobieren", "Klicke im laufenden Programm auf einen Spaltenkopf. Die Tabelle sortiert – und die Statuszeile nennt trotzdem weiterhin das richtige Spiel. Das ist kein Zufall, das ist die Brücke."));

// ===================== Kapitel 5 =====================
children.push(chapter("Kapitel 5: Tipps eingeben"));
children.push(tip("In diesem Kapitel",
  "Das Programm schreibt zum ersten Mal zurück. Und die Datenbank sorgt selbst dafür, dass es je Spieler und Spiel nur einen Tipp gibt."));

children.push(p("Bis jetzt hat unser Programm nur gezeigt. Jetzt darf es speichern – und damit taucht sofort eine Frage auf, die sich in jeder Anwendung stellt: Was passiert, wenn jemand denselben Tipp zweimal abgibt?"));
children.push(p("Die naheliegende Antwort wäre: erst nachsehen, ob es schon einen Tipp gibt, dann entweder einfügen oder ändern. Das funktioniert – bis zu dem Tag, an dem zwei Dinge gleichzeitig passieren. Besser ist es, die Frage der Datenbank zu überlassen."));
children.push(codeBlock([
  'DB_EXEC(db, "CREATE TABLE IF NOT EXISTS tipps (" + _',
  '            "id INTEGER PRIMARY KEY, spieler_id INTEGER NOT NULL, " + _',
  '            "spiel_id INTEGER NOT NULL, tipp_heim INTEGER NOT NULL, " + _',
  '            "tipp_gast INTEGER NOT NULL, punkte INTEGER, " + _',
  '            "UNIQUE(spieler_id, spiel_id))")',
]));
children.push(pmix([
  ["", false], ["UNIQUE(spieler_id, spiel_id)", true],
  [" heißt: diese Kombination darf es nur einmal geben. Nicht „das Programm passt schon auf“, sondern eine Regel in den Daten selbst. Sie gilt auch dann noch, wenn jemand die Datei mit einem anderen Werkzeug öffnet.", false],
]));
children.push(p("Und beim Speichern dann der passende Befehl dazu – einfügen, und falls es den Tipp schon gibt, ändern:"));
children.push(codeBlock([
  'DB_EXEC(db, "INSERT INTO tipps (spieler_id, spiel_id, tipp_heim, tipp_gast) " + _',
  '            "VALUES (?, ?, ?, ?) " + _',
  '            "ON CONFLICT(spieler_id, spiel_id) DO UPDATE SET " + _',
  '            "  tipp_heim = excluded.tipp_heim, " + _',
  '            "  tipp_gast = excluded.tipp_gast", _',
  "        aktiverSpieler(), spielId[zeile], th, tg)",
]));

children.push(h2("Eingaben, die gar nicht falsch sein können"));
children.push(pmix([
  ["Für die Torzahlen nehmen wir kein Texteingabefeld, sondern ", false], ["GUI_SPINNER", true],
  [" – ein Feld mit Pfeilchen, das nur ganze Zahlen von 0 bis 9 zulässt. Damit kann niemand „drei“ oder „-1“ eintippen. Die Eingabe ist gültig, bevor sie im Programm ankommt.", false],
]));
children.push(codeBlock([
  "tippHeim = GUI_SPINNER(win, 70, 380, 90, 0, 9, 0, 1)",
]));
children.push(why("Weil jede Prüfung, die man sich sparen kann, eine Prüfung ist, die nicht vergessen werden kann. Ein Textfeld hätte drei Fehlerquellen: leer, keine Zahl, unsinnige Zahl. Ein Spinner hat keine.", "Warum ein Spinner statt eines Textfelds?"));

children.push(h2("Alle Spiele, auch die ohne Tipp"));
children.push(codeBlock([
  'r = DB_QUERY(db, "SELECT s.id, s.heim, s.gast, t.tipp_heim, t.tipp_gast " + _',
  '                 "FROM spiele s " + _',
  '                 "LEFT JOIN tipps t ON t.spiel_id = s.id AND t.spieler_id = ? " + _',
  '                 "WHERE s.spieltag = 1 ORDER BY s.id", aktiverSpieler())',
]));
children.push(pmix([
  ["Das ", false], ["LEFT JOIN", true],
  [" ist der Unterschied zwischen „alle Spiele, dazu der Tipp falls vorhanden“ und „nur die Spiele, die schon getippt wurden“. Mit einem gewöhnlichen JOIN verschwänden genau die Zeilen aus der Liste, die man am dringendsten braucht – die noch untippten.", false],
]));
children.push(p("Ein JOIN führt zwei Tabellen zusammen. Man sagt der Datenbank, woran sie erkennt, welche Zeile zu welcher gehört – hier: der Tipp gehört zum Spiel, wenn seine spiel_id die id des Spiels ist. Das Wörtchen LEFT bedeutet: Die linke Tabelle (die Spiele) bleibt vollständig, auch wenn rechts nichts passt. Dann stehen in den Tipp-Spalten eben NULL-Werte – und genau die fragen wir mit DB_IS_NULL ab."));
children.push(p("Und beachte die zweite Bedingung im JOIN: der Tipp muss außerdem vom aktuell gewählten Spieler sein. Ohne sie stünden alle Tipps aller Mitspieler nebeneinander in derselben Zeile."));

children.push(h2("Der Weg eines Klicks"));
children.push(p("Damit sich das Formular gut anfühlt, muss ein Klick auf eine Zeile mehr tun, als sie zu markieren: Steht dort schon ein Tipp, gehört er in die Eingabefelder. Man ändert dann einen Wert, statt ihn neu zu erfinden."));
children.push(codeBlock([
  "DIM zeile AS INTEGER : zeile = GUI_TABLE_SELECTED(tabelle)",
  "IF zeile >= 0 AND zeile <> letzteZeile THEN",
  "    letzteZeile = zeile",
  "    DIM tippText AS STRING : tippText = GUI_TABLE_GET_CELL(tabelle, zeile, 2)",
  '    IF tippText <> "-:-" THEN',
  "        DIM teile AS ARRAY OF STRING",
  '        teile = SPLIT$(tippText, ":")',
  "        GUI_SET_VALUE(tippHeim, VAL(teile[0]))",
  "        GUI_SET_VALUE(tippGast, VAL(teile[1]))",
  "    END IF",
  "END IF",
]));
children.push(pmix([
  ["Der Vergleich ", false], ["zeile <> letzteZeile", true],
  [" ist der Grund, warum das funktioniert. Ohne ihn liefe dieser Block sechzigmal pro Sekunde und würde die Eingabefelder unablässig zurücksetzen – man könnte gar nichts eintippen, weil das Programm einem sofort wieder dazwischenfunkt. So läuft er nur beim Wechsel der Zeile.", false],
]));
figure("kap05_tipps.png", "Kapitel 5: Spielerauswahl, Tabelle und die Eingabe darunter.", 640, 380).forEach(e => children.push(e));

// ===================== Kapitel 6 =====================
children.push(chapter("Kapitel 6: Ergebnisse und Punkte"));
children.push(tip("In diesem Kapitel",
  "Die Regel aus Kapitel 3 kommt ins Programm. Ein eingetragenes Ergebnis bewertet alle Tipps auf dieses Spiel neu – auch die der anderen."));

children.push(p("Jetzt fügt sich zusammen, was wir getrennt gebaut haben. Sobald ein Ergebnis feststeht, sind alle Tipps auf dieses Spiel neu zu bewerten. Nicht nur der eigene – auch die der Mitspieler, die man gerade gar nicht ansieht."));
children.push(codeBlock([
  "SUB punkteNeu(spiel AS INTEGER, ergH AS INTEGER, ergA AS INTEGER)",
  '    r = DB_QUERY(db, "SELECT id, tipp_heim, tipp_gast FROM tipps WHERE spiel_id = ?", spiel)',
  "",
  "    ' Erst alles einsammeln ...",
  "    WHILE DB_NEXT(r)",
  "        ids[n] = DB_GET_INT(r, 0)",
  "        werte[n] = punkte(DB_GET_INT(r, 1), DB_GET_INT(r, 2), ergH, ergA)",
  "        n = n + 1",
  "    WEND",
  "    DB_CLOSE_RESULT(r)",
  "",
  "    ' ... und erst dann schreiben.",
  "    DB_BEGIN(db)",
  "    FOR i = 0 TO n - 1",
  '        DB_EXEC(db, "UPDATE tipps SET punkte = ? WHERE id = ?", werte[i], ids[i])',
  "    NEXT",
  "    DB_COMMIT(db)",
  "END SUB",
]));
children.push(warn("Die Zweiteilung ist kein Schönheitsfehler, sondern der Kern dieser Prozedur: Während ein Ergebnis-Cursor offen ist, schreibt es sich schlecht in dieselbe Tabelle. Man liest gerade Zeilen, die man gleichzeitig verändert – das geht in jeder Datenbank schief, nur auf unterschiedliche Weise."));
children.push(p("Beachte außerdem, wer hier rechnet: nicht die Datenbank, sondern unsere Funktion punkte() aus Kapitel 3, unverändert. Die Regel steht an einer Stelle, und diese Stelle wird gefragt – von der Anzeige wie von der Neuberechnung."));

children.push(h2("Farbe sagt mehr als eine Zahl"));
children.push(codeBlock([
  'IF punkteText <> "" THEN',
  "    DIM p AS INTEGER : p = DB_GET_INT(r, 7)",
  "    IF p >= 3 THEN",
  "        GUI_TABLE_CELL_COLOR(tabelle, spieleAnzahl, 4, &H7CE38B, -1)",
  "    ELIF p >= 1 THEN",
  "        GUI_TABLE_CELL_COLOR(tabelle, spieleAnzahl, 4, &HE8C46A, -1)",
  "    ELSE",
  "        GUI_TABLE_CELL_COLOR(tabelle, spieleAnzahl, 4, &HE07A7A, -1)",
  "    END IF",
  "END IF",
]));
children.push(p("Grün, gelb, rot – man sieht auf einen Blick, wo Punkte liegen, ohne eine einzige Zahl zu lesen. Und eine leere Zelle bleibt leer: Ein Tipp auf ein Spiel ohne Ergebnis bekommt keine Farbe, weil er noch gar nicht bewertet ist."));
children.push(warn("Farbe darf nie die einzige Information sein. Etwa jeder zwölfte Mann unterscheidet Rot und Grün schlecht – für ihn sind unsere drei Farben drei ähnliche Grautöne. Deshalb steht die Punktzahl in der Zelle, und die Farbe kommt hinzu. Eine Anzeige, die ohne Farbe genauso verständlich ist, ist mit Farbe nur schneller."));

children.push(h2("Ein geänderter Tipp ist ein ungewerteter Tipp"));
children.push(pmix([
  ["Beim Speichern eines Tipps setzen wir ", false], ["punkte = NULL", true],
  [" – die alte Punktzahl wird gelöscht. Das klingt nach einem Detail, ist aber der Unterschied zwischen einer richtigen und einer falschen Rangliste:", false],
]));
children.push(codeBlock([
  '"ON CONFLICT(spieler_id, spiel_id) DO UPDATE SET " + _',
  '"  tipp_heim = excluded.tipp_heim, " + _',
  '"  tipp_gast = excluded.tipp_gast, punkte = NULL"',
]));
children.push(p("Ohne dieses NULL stünde nach einer Änderung die alte Punktzahl an einem neuen Tipp. Und weil ein Spiel, dessen Ergebnis schon feststeht, sofort neu gewertet werden soll, fragt das Programm gleich danach nach – steht ein Ergebnis, ruft es punkteNeu() auf. So gibt es nie einen Zustand, in dem Anzeige und Regel auseinanderfallen."));
children.push(why("Weil ein falscher Wert schlimmer ist als gar keiner. Eine leere Punktespalte sieht aus wie „noch nicht gewertet“ und ist damit ehrlich. Eine alte Punktzahl an einem neuen Tipp sieht richtig aus und ist falsch – und niemand kommt auf die Idee, sie anzuzweifeln.", "Warum die Punkte löschen statt sie stehen zu lassen?"));
figure("kap06_punkte.png", "Kapitel 6: Tipp, Ergebnis und Punkte – grün für den vollen Treffer.", 640, 380).forEach(e => children.push(e));

// ===================== Kapitel 7 =====================
children.push(chapter("Kapitel 7: Die Rangliste"));
children.push(tip("In diesem Kapitel",
  "Ein zweiter Reiter, eine Abfrage – und der Satz, der ein Tippspiel fair macht: gewertet wird nur, was gespielt ist."));

children.push(p("Ein Tippspiel ohne Rangliste ist kein Tippspiel. Und weil wir schon ein Fenster voller Bedienelemente haben, bekommt sie ein eigenes Blatt: Reiter."));
children.push(codeBlock([
  "DIM reiter AS ARRAY OF STRING",
  'reiter = SPLIT$("Tipps|Rangliste", "|")',
  "GUI_TABS(win, reiter)",
  "",
  "GUI_SET_TAB(tabelle, 0)      ' gehört auf Blatt 1",
  "GUI_SET_TAB(rang, 1)         ' gehört auf Blatt 2",
]));
children.push(p("Jedes Bedienelement bekommt gesagt, auf welches Blatt es gehört. Ohne diese Zeilen stünde alles auf beiden – ein Anblick, den man einmal gesehen haben sollte, um ihn nie wieder zu wollen."));

children.push(h2("Rechnen lassen, statt zu rechnen"));
children.push(p("Die Versuchung ist groß, die Rangliste im Programm zusammenzuzählen: über alle Spieler laufen, für jeden über alle Tipps, addieren. Das wären zwei ineinandergeschachtelte Schleifen und ungefähr dreißig Zeilen. Die Datenbank macht dasselbe in einer Abfrage – und zwar schneller und ohne Zählfehler."));
children.push(codeBlock([
  'r = DB_QUERY(db, "SELECT p.name, " + _',
  '                 "  COALESCE(SUM(CASE WHEN s.tore_heim IS NOT NULL " + _',
  '                 "                    THEN t.punkte END), 0) AS punkte, " + _',
  '                 "  COUNT(t.id) AS tipps " + _',
  '                 "FROM spieler p " + _',
  '                 "LEFT JOIN tipps t ON t.spieler_id = p.id " + _',
  '                 "LEFT JOIN spiele s ON s.id = t.spiel_id " + _',
  '                 "GROUP BY p.id, p.name " + _',
  '                 "ORDER BY punkte DESC, p.name")',
]));
children.push(p("Diese Abfrage sieht länger aus, als sie ist. Sie sagt: Nimm jeden Spieler, hänge seine Tipps daran, hänge an jeden Tipp das zugehörige Spiel – und zähle dann pro Spieler zusammen."));
children.push(new Paragraph({
  alignment: AlignmentType.CENTER, spacing: { before: 60, after: 160 },
  children: [new TextRun({ text: "Gewertet wird nur, was gespielt ist.", bold: true, size: 26, color: C_H2 })],
}));
children.push(pmix([
  ["Das steckt im ", false], ["CASE WHEN s.tore_heim IS NOT NULL", true],
  [". Ein Tipp auf ein Spiel ohne Ergebnis ist keine Null – er zählt einfach noch nicht. Lässt man diese Bedingung weg, behauptet die Rangliste, jemand hätte danebengetippt, obwohl das Spiel erst am Samstag stattfindet.", false],
]));
figure("kap07_rangliste.png", "Kapitel 7: Anna führt – ihre zwei Tipps, aber nur einer ist gewertet.", 640, 380).forEach(e => children.push(e));
children.push(h2("Wie eine Gruppierung funktioniert"));
children.push(pmix([
  ["", false], ["GROUP BY p.id, p.name", true],
  [" fasst alle Zeilen zusammen, die zum selben Spieler gehören. Aus zwanzig Zeilen – eine je Tipp – wird eine Zeile je Spieler. Und in dieser einen Zeile darf man nur noch Dinge abfragen, die für die ganze Gruppe gelten: die Summe, die Anzahl, das Maximum.", false],
]));
children.push(bulletRich("SUM(...) – ", "addiert die Punkte aller Tipps dieses Spielers."));
children.push(bulletRich("COUNT(t.id) – ", "zählt die Tipps. Beachte, dass hier t.id gezählt wird und nicht ein Stern: Ein Spieler ohne Tipps hätte durch den LEFT JOIN trotzdem eine Zeile, und COUNT(*) zählte diese eine leere Zeile als Tipp."));
children.push(bulletRich("COALESCE(..., 0) – ", "macht aus „gar nichts“ eine Null. Ein Spieler, der nur ungewertete Tipps hat, bekäme sonst kein Ergebnis, sondern NULL – und die Anzeige zeigte ein leeres Feld statt einer 0."));
children.push(p("Diese drei Kleinigkeiten sind typisch für den Umgang mit Datenbanken: Die Abfrage ist schnell geschrieben, aber die Sonderfälle stecken in den Rändern – bei den Spielern ohne Tipp und den Tipps ohne Ergebnis. Und genau diese Ränder sieht man erst, wenn man sie ausprobiert."));
children.push(h2("Was bei Gleichstand passiert"));
children.push(pmix([
  ["", false], ["ORDER BY punkte DESC, p.name", true],
  [" sortiert nach Punkten, absteigend – und bei gleicher Punktzahl nach dem Namen. Dieses zweite Kriterium ist kein Schnörkel: Ohne es entscheidet die Datenbank selbst, wer zuerst kommt, und darf ihre Meinung zwischen zwei Aufrufen ändern. Eine Rangliste, die bei jedem Neuladen die Plätze tauscht, wirkt kaputt – auch wenn beide Spieler wirklich gleichauf liegen.", false],
]));
children.push(tip("Zum Ausprobieren", "Gib für Clara einen Tipp auf ein Spiel ohne Ergebnis ab. Die Spalte „Tipps“ steigt, die Punkte bleiben. Genau so soll es sein. Entferne danach probeweise das CASE WHEN aus der Abfrage und sieh dir an, was die Rangliste dann behauptet."));

// ===================== Kapitel 8 =====================
children.push(chapter("Kapitel 8: Zeit"));
children.push(tip("In diesem Kapitel",
  "Anstoßzeiten, Tippschluss und ein Countdown, der jede Sekunde weiterläuft. Und ein Grundsatz: Zeit ist eine Zahl, kein Text."));

children.push(p("Bis jetzt durfte man jederzeit tippen – auch nach dem Abpfiff. Das ist in einer Tipprunde unter Freunden vielleicht kein Problem, aber es ist auch keine Lösung. Also bekommen die Spiele einen Anstoß."));
children.push(p("Und damit stellt sich die Frage, an der viele Programme scheitern: Wie speichert man einen Zeitpunkt? Als Text ist er gut lesbar und lässt sich sogar richtig sortieren – aber rechnen kann man damit nicht. Als Zahl kann man rechnen, aber niemand kann sie lesen."));
children.push(p("Die Antwort ist: beides, an unterschiedlichen Orten."));
children.push(codeBlock([
  'IMPORT "zeit"',
  "",
  "' In der Datenbank steht Text -- so sortiert SQLite ihn richtig:",
  "'   anstoss = \"2026-08-29 15:30:00\"",
  "",
  "' Zum Rechnen einmal in eine Zahl umwandeln:",
  "anstoss[zeile] = ZEIT_PARSE(DB_GET_STRING(r, 8))",
  "",
  "' Und zum Anzeigen wieder heraus:",
  'anstossText = ZEIT_FORMAT$(anstoss[zeile], "WT TT.MM. hh:mm")   \' "Sa 29.08. 15:30"',
]));
children.push(p("Ein Zeitpunkt ist dabei nichts Geheimnisvolles: eine ganze Zahl, nämlich die Sekunden seit dem 1. Januar 1970. Mehr steckt nicht dahinter, und das ist der ganze Trick. Später heißt größer. Eine Viertelstunde sind 15 mal 60. Monatsenden, Jahreswechsel und Schaltjahre stecken in der Umrechnung, nicht in deinem Programm."));

children.push(h2("Eine Regel, eine Stelle – schon wieder"));
children.push(codeBlock([
  "CONST TIPPSCHLUSS_VOR = 15 * 60",
  "",
  "FUNCTION tippschluss(zeile AS INTEGER) AS INTEGER",
  "    IF anstoss[zeile] = 0 THEN RETURN 0",
  "    RETURN ZEIT_PLUS(anstoss[zeile], -TIPPSCHLUSS_VOR)",
  "END FUNCTION",
  "",
  "FUNCTION offen(zeile AS INTEGER) AS BOOLEAN",
  "    IF anstoss[zeile] = 0 THEN RETURN TRUE",
  "    RETURN ZEIT_JETZT() < tippschluss(zeile)",
  "END FUNCTION",
]));
children.push(p("Beide Fragen, die das Programm stellt – „darf hier noch getippt werden?“ und „wie lange noch?“ – rechnen mit derselben Zahl. Wer den Tippschluss von einer Viertelstunde auf eine halbe ändern will, ändert genau eine Zeile."));
children.push(pmix([
  ["Und beachte die Zeile ", false], ["IF anstoss[zeile] = 0 THEN RETURN TRUE", true],
  [": Ein Spiel ohne bekannten Termin bleibt tippbar. Ein fehlender Anstoß darf niemanden aussperren – das wäre eine Strafe für einen Fehler, den der Benutzer nicht gemacht hat.", false],
]));
figure("kap08_zeit.png", "Kapitel 8: Anstoßzeiten mit Wochentag; das vergangene Spiel steht grau.", 640, 380).forEach(e => children.push(e));
children.push(h2("Anzeigen, wie Menschen es lesen"));
children.push(pmix([
  ["", false], ["ZEIT_FORMAT$", true],
  [" setzt Muster ein, die in derselben Sprache geschrieben sind wie das, was herauskommt:", false],
]));
children.push(codeBlock([
  "ZEIT_FORMAT$(a, \"TT.MM.JJJJ\")              ' 28.08.2026",
  "ZEIT_FORMAT$(a, \"hh:mm\")                   ' 20:30",
  "ZEIT_FORMAT$(a, \"WT TT.MM. hh:mm\")         ' Fr 28.08. 20:30",
  "ZEIT_FORMAT$(a, \"WTAG, TT.MM.JJJJ hh:mm\")  ' Freitag, 28.08.2026 20:30",
]));
children.push(p("In der Tabelle nehmen wir die kurze Form mit Wochentag. Das ist kein Zufall: Bei einem Spieltag interessiert kaum jemanden das Jahr, aber jeden, ob das Spiel am Freitag, Samstag oder Sonntag ist. Eine gute Anzeige zeigt nicht alles, was sie weiß, sondern das, wonach gefragt wird."));
children.push(h2("Der Countdown"));
children.push(p("Die Statuszeile soll die verbleibende Zeit mitzählen, auch wenn niemand etwas anklickt. Sie sechzigmal pro Sekunde neu zu setzen wäre Verschwendung – die Anzeige zeigt schließlich keine Zehntelsekunden:"));
children.push(codeBlock([
  "IF zeile >= 0 AND ZEIT_JETZT() <> letzteSekunde THEN",
  "    letzteSekunde = ZEIT_JETZT()",
  "    melde(... + restText(zeile))",
  "END IF",
]));
children.push(pmix([
  ["Der Vergleich mit ", false], ["letzteSekunde", true],
  [" ist derselbe Kniff wie beim Zeilenwechsel in Kapitel 5: etwas nur dann tun, wenn es sich geändert hat. Man wird diesem Muster in jedem Programm mit Schleife begegnen.", false],
]));
children.push(warn("MILLIS() ist etwas anderes als ZEIT_JETZT(). MILLIS ist die Stoppuhr des Programms – sie beginnt beim Start bei null und ist für Frame-Zeiten gedacht. Für Datum und Uhrzeit ist ZEIT_JETZT() zuständig. Wer beides verwechselt, liegt um Jahrzehnte daneben."));

// ===================== Kapitel 9 =====================
children.push(chapter("Kapitel 9: Daten aus dem Netz"));
children.push(tip("In diesem Kapitel",
  "Der Spielplan kommt von OpenLigaDB – und das Fenster bleibt dabei bedienbar. Warten heißt nachsehen, nicht stehenbleiben."));

children.push(p("Den Spielplan von Hand einzutippen ist Arbeit, die schon jemand gemacht hat. OpenLigaDB gibt ihn her, kostenlos und ohne Anmeldung. Ein Abruf ist eine Zeile:"));
children.push(codeBlock([
  'antwort = HTTP_GET("https://api.openligadb.de/getmatchdata/bl1/2026/1")',
]));
children.push(p("Und genau diese Zeile ist ein Problem. Sie hält das ganze Programm an, bis die Antwort da ist. Gemessen: rund 200 Millisekunden für eine mittlere Antwort – ein Zwölftel Sekunde, in der weder Maus noch Tastatur reagieren und nichts neu gezeichnet wird. Bei schlechter Verbindung wartet das Fenster bis zum Zeitlimit von zehn Sekunden."));
children.push(new Paragraph({
  alignment: AlignmentType.CENTER, spacing: { before: 60, after: 160 },
  children: [new TextRun({ text: "Warten heißt nachsehen, nicht stehenbleiben.", bold: true, size: 26, color: C_H2 })],
}));
children.push(p("Deshalb gibt es denselben Abruf auch zum Nachsehen. Das Programm stößt ihn an, bekommt sofort eine Nummer zurück und fragt in jedem Bild einmal, ob die Antwort da ist – genau so, wie es auch nach der Maus fragt."));
children.push(codeBlock([
  "DIM abruf AS INTEGER : abruf = -1",
  "",
  "SUB spielplanHolen()",
  '    IF abruf >= 0 THEN melde("Wird schon geholt.") : RETURN',
  "    abruf = HTTP_GET_START(SPIELPLAN_URL)      ' kommt sofort zurück",
  '    melde("Spielplan wird geholt ...")',
  "END SUB",
  "",
  "SUB abrufPruefen()                            ' einmal pro Bild",
  "    IF abruf < 0 THEN RETURN",
  "    IF NOT HTTP_READY(abruf) THEN RETURN",
  "    DIM nummer AS INTEGER : nummer = abruf",
  "    abruf = -1                                 ' erst freigeben ...",
  "    spielplanUebernehmen(HTTP_RESULT(nummer))  ' ... dann auswerten",
  "END SUB",
]));
children.push(p("Gemessen an derselben Antwort: mit HTTP_GET stand das Fenster still, mit HTTP_GET_START lief die Schleife in der Wartezeit über hundert Mal durch. Der Benutzer merkt vom Abruf nichts außer der Meldung in der Statuszeile."));

children.push(h2("Aus JSON werden Spiele"));
children.push(codeBlock([
  "DIM wurzel AS JSON_HANDLE",
  "wurzel = JSON_PARSE(text)",
  'DIM n AS INTEGER : n = JSON_LEN(wurzel, "")',
  "FOR i = 0 TO n - 1",
  "    DIM pfad AS STRING : pfad = STR$(i)",
  '    heim = JSON_GET_STRING(wurzel, pfad + ".team1.teamName")',
  '    gast = JSON_GET_STRING(wurzel, pfad + ".team2.teamName")',
  '    roh  = JSON_GET_STRING(wurzel, pfad + ".matchDateTime")',
  "    IF ZEIT_LESBAR(roh) THEN termin = ZEIT_TEXT$(ZEIT_PARSE(roh))",
  "    spielAnlegen(heim, gast, termin)",
  "NEXT",
]));
children.push(pmix([
  ["Die Schnittstelle liefert Zeitpunkte als ", false], ['"2026-08-29T15:30:00"', true],
  [" – mit einem T in der Mitte. ", false], ["ZEIT_PARSE", true],
  [" versteht das, und ", false], ["ZEIT_TEXT$", true],
  [" macht daraus die Schreibweise, die in unserer Datenbank steht. So bleibt im ganzen Programm ein Format, obwohl die Welt draußen mehrere kennt.", false],
]));
children.push(h2("Zweimal holen darf nichts kaputt machen"));
children.push(p("Jemand wird den Knopf zweimal drücken. Nicht aus Bosheit, sondern weil beim ersten Mal scheinbar nichts passiert ist. Also muss ein zweiter Abruf denselben Zustand herstellen wie der erste – und nicht alles verdoppeln."));
children.push(codeBlock([
  'DB_EXEC(db, "CREATE UNIQUE INDEX IF NOT EXISTS idx_spiel " + _',
  '            "ON spiele (spieltag, heim, gast)")',
  "",
  "SUB spielAnlegen(heim AS STRING, gast AS STRING, termin AS STRING)",
  '    DB_EXEC(db, "INSERT INTO spiele (spieltag, heim, gast, anstoss) " + _',
  '                "VALUES (1, ?, ?, ?) " + _',
  '                "ON CONFLICT(spieltag, heim, gast) DO UPDATE SET " + _',
  '                "  anstoss = excluded.anstoss", heim, gast, termin)',
  "END SUB",
]));
children.push(p("Dieselbe Paarung am selben Spieltag gibt es nur einmal – das sagt der Index. Und der UPSERT davor sagt: Wenn es sie schon gibt, frische den Anstoß auf, statt zu scheitern. Zusammen ergibt das eine Aktion, die man beliebig oft ausführen darf. Wichtig ist dabei, dass die Tipps unangetastet bleiben: Sie hängen an der spiel_id, und die ändert sich nicht."));
children.push(why("Weil man in einer Anwendung nie weiß, wie oft etwas passiert. Der Benutzer klickt doppelt, das Netz wiederholt eine Anfrage, das Programm startet neu. Eine Aktion, die man gefahrlos wiederholen kann, erspart einem all diese Überlegungen – und sie ist meistens nur ein paar Zeichen länger als die gefährliche Variante.", "Warum so viel Aufwand für einen Doppelklick?"));
children.push(warn("Nach dem ersten Abruf stehen plötzlich „Bor. Mönchengladbach“ und „Borussia Mönchengladbach“ nebeneinander in der Liste. Für die Datenbank sind das zwei Vereine – unsere Beispieldaten kürzen anders als die Schnittstelle. Wer Daten aus zwei Quellen zusammenführt, braucht darauf eine Antwort: eine eigene Vereins-Tabelle mit ids, statt Namen zu vergleichen."));

// ===================== Kapitel 10 =====================
children.push(chapter("Kapitel 10: Wenn etwas schiefgeht"));
children.push(tip("In diesem Kapitel",
  "Drei Werkzeuge für drei Zeitpunkte: prüfen, bevor etwas passiert; fragen, wenn es weh tut; fangen, wenn es schiefgeht."));

children.push(p("Bis hierher hat alles geklappt. In Wirklichkeit klappt es nicht immer: Das Netz ist weg, die Antwort ist Unsinn, jemand klickt zweimal, jemand trägt ein Ergebnis falsch ein. Ein Programm, das bei jedem Schluckauf abstürzt, verliert Vertrauen – und Tipps."));

children.push(h2("Fangen"));
children.push(codeBlock([
  "TRY",
  "    spielplanUebernehmen(HTTP_RESULT(nummer))",
  "CATCH e",
  '    melde("Spielplan nicht erreichbar: " + e)',
  "END TRY",
]));
children.push(p("Kein Netz, ein 404, eine kaputte Antwort – all das landet hier. Ohne TRY wäre das Programm weg, und mit ihm die ungespeicherte Arbeit. Die Meldung geht in die Statuszeile, denn dorthin schaut der Benutzer ohnehin."));
children.push(why("Weil der Fehler beim Abholen auftaucht, nicht beim Anstoßen. HTTP_GET_START weiß noch nicht, ob die Verbindung stehen wird. Erst HTTP_RESULT bringt entweder die Antwort oder die schlechte Nachricht – und dort steht der Programmteil, der damit umgehen kann.", "Warum steht das TRY beim Abholen?"));

children.push(h2("Fragen"));
children.push(codeBlock([
  "DIM altes AS STRING : altes = GUI_TABLE_GET_CELL(tabelle, zeile, 4)",
  "DIM weiter AS BOOLEAN : weiter = TRUE",
  'IF altes <> "-:-" AND altes <> STR$(eh) + ":" + STR$(eg) THEN',
  '    weiter = GUI_CONFIRM("Ergebnis aendern?", _',
  '        "Fuer dieses Spiel steht schon " + altes + "." + CHR$(10) + _',
  '        "Auf " + STR$(eh) + ":" + STR$(eg) + " aendern und alle Punkte neu rechnen?", _',
  '        "janein")',
  "END IF",
]));
children.push(p("Beachte die Bedingung davor. Die Rückfrage kommt nur, wenn sich wirklich etwas ändert – nicht beim ersten Eintragen und nicht, wenn jemand denselben Wert noch einmal speichert."));
children.push(warn("Eine Rückfrage, die immer kommt, wird weggeklickt, ohne gelesen zu werden. Und dann ist sie schlimmer als keine: Sie erzeugt das Gefühl von Sicherheit, ohne welche zu geben. Frage selten, aber dann deutlich – und schreibe in die Frage hinein, was verloren geht."));

children.push(h2("Prüfen"));
children.push(codeBlock([
  "IF LEN(text) < 10 THEN",
  '    melde("Die Antwort war leer.")',
  "    RETURN",
  "END IF",
]));
children.push(p("Eine Fehlerseite ist auch eine Antwort, nur eben keine, aus der sich Spiele lesen lassen. Erst prüfen, dann arbeiten – das erspart einem, hinterher aus einem halb gefüllten Datenbestand wieder herauszufinden."));

children.push(h2("Wie eine gute Fehlermeldung klingt"));
children.push(p("Fehlermeldungen schreibt man für den Menschen, der gerade nicht weiterkommt – nicht für den Programmierer, der sie eingebaut hat. Drei Fragen sollte sie beantworten: Was ist passiert? Woran lag es? Was kann ich jetzt tun?"));
children.push(bulletRich("Schlecht: ", "„Fehler 500“. Sagt nichts, was der Benutzer verwenden kann."));
children.push(bulletRich("Besser: ", "„Spielplan nicht erreichbar“. Sagt, was nicht ging."));
children.push(bulletRich("Gut: ", "„Spielplan nicht erreichbar: Zeitüberschreitung nach 10 s. Prüfe die Internetverbindung und versuche es noch einmal.“"));
children.push(pmix([
  ["Deshalb hängen wir in unserem CATCH die ursprüngliche Meldung an: ", false],
  ['melde("Spielplan nicht erreichbar: " + e)', true],
  [". Der Text in ", false], ["e", true],
  [" kommt aus der Runtime und nennt den eigentlichen Grund – so steht beides da, das Verständliche und das Genaue.", false],
]));
children.push(h2("Wo ein Fehler nicht hingehört"));
children.push(p("Es gibt eine Versuchung, der man widerstehen sollte: alles in ein großes TRY zu packen, damit „nichts mehr abstürzen kann“. Das Ergebnis ist ein Programm, das im Fehlerfall irgendwie weiterläuft – mit halb geänderten Daten und einer Anzeige, die etwas anderes behauptet als die Datenbank."));
children.push(why("Weil ein Programm, das nach einem Fehler weiterläuft, als sei nichts gewesen, schlimmer ist als eines, das anhält. Fange dort, wo du weißt, was der Fehler bedeutet und wie es weitergehen soll. Für alles andere ist ein klarer Abbruch die ehrlichere Antwort – und in einer Anwendung mit Datenbank steht dann wenigstens noch alles richtig in der Datei.", "Warum nicht einfach alles in ein TRY packen?"));
figure("kap10_robust.png", "Kapitel 10: äußerlich unverändert – der Unterschied zeigt sich erst, wenn etwas schiefgeht.", 640, 380).forEach(e => children.push(e));

// ===================== Kapitel 11 =====================
children.push(chapter("Kapitel 11: Sicherung und Umbau"));
children.push(tip("In diesem Kapitel",
  "Zwei Dinge, die in keinem Spiel vorkommen und in jeder Anwendung: eine Sicherung – und ein Umbau der Datenbank im laufenden Betrieb."));

children.push(h2("Die Sicherung"));
children.push(p("Die Datenbank ist die Wahrheit. Und Wahrheit, die es nur einmal gibt, ist eine Wette. Eine Sicherung ist eine zweite Datei – und der einzige interessante Teil daran ist der Zeitpunkt."));
children.push(codeBlock([
  "SUB sichern()",
  "    DB_CLOSE(db)                    ' erst schließen ...",
  "    COPYFILE(DATEI, SICHERUNG)      ' ... dann kopieren",
  "    db = DB_OPEN(DATEI)",
  '    melde("Gesichert nach " + SICHERUNG + " (" + _',
  '          ZEIT_FORMAT$(ZEIT_JETZT(), "TT.MM. hh:mm:ss") + ")")',
  "END SUB",
]));
children.push(warn("Warum erst schließen? Eine geöffnete Datenbank kann Änderungen im Speicher haben, die noch nicht in der Datei stehen. Kopiert man sie in diesem Zustand, hat man eine Sicherung, die fast stimmt – und das ist die schlechteste Sorte Sicherung, weil man es erst merkt, wenn man sie braucht."));
children.push(p("Und beim Einspielen wird gefragt – hier lohnt sich die Rückfrage aus Kapitel 10, denn dieser Klick löscht Arbeit:"));
children.push(codeBlock([
  'IF NOT GUI_CONFIRM("Sicherung einspielen?", _',
  '        "Alle Tipps und Ergebnisse seit der Sicherung gehen verloren." + CHR$(10) + _',
  '        "Wirklich einspielen?", "janein") THEN',
  '    melde("Einspielen abgebrochen.")',
  "    RETURN",
  "END IF",
]));

children.push(h2("Der Umbau"));
children.push(p("Das Programm von morgen braucht eine Spalte, die es heute nicht gibt. Die Daten von heute sollen trotzdem erhalten bleiben. Also nachrüsten statt neu anfangen – und zwar so, dass es bei jedem Start funktioniert, auch beim zwanzigsten."));
children.push(codeBlock([
  "FUNCTION spalteDa(tabelle AS STRING, spalte AS STRING) AS BOOLEAN",
  '    r = DB_QUERY(db, "PRAGMA table_info(" + tabelle + ")")',
  "    DIM gefunden AS BOOLEAN : gefunden = FALSE",
  "    WHILE DB_NEXT(r)",
  "        IF DB_GET_STRING(r, 1) = spalte THEN gefunden = TRUE",
  "    WEND",
  "    DB_CLOSE_RESULT(r)",
  "    RETURN gefunden",
  "END FUNCTION",
  "",
  'IF NOT spalteDa("spiele", "notiz") THEN',
  '    DB_EXEC(db, "ALTER TABLE spiele ADD COLUMN notiz TEXT")',
  "END IF",
]));
children.push(why("Weil die Datenbank selbst am besten weiß, wie sie aussieht. Die Alternative wäre eine Versionsnummer, die jemand beim Ändern hochzählen muss – und die genau dann falsch ist, wenn man sich darauf verlässt. PRAGMA table_info fragt einfach nach.", "Warum nachsehen statt eine Versionsnummer führen?"));
children.push(p("Wichtig ist dabei die Reihenfolge im Programm: Der Umbau steht ganz am Anfang, direkt nach dem Öffnen der Datenbank und vor allen Abfragen. Er läuft bei jedem Start, tut aber nur beim ersten Mal etwas. So kann sich niemand eine Fassung des Programms herunterladen, in der die Daten und der Code nicht zusammenpassen."));
children.push(h2("Wie oft, und wie viele?"));
children.push(p("Unsere Sicherung ist eine einzige Datei, die jedes Mal überschrieben wird. Für eine Tipprunde reicht das – aber es lohnt sich zu wissen, was daran eng ist: Wer einen Fehler erst nach zwei Sicherungen bemerkt, kommt nicht mehr an den Stand davor."));
children.push(bulletRich("Eine Datei, überschrieben: ", "einfach, und man ist immer nur einen Klick von der letzten Sicherung entfernt. Unsere Wahl."));
children.push(bulletRich("Eine Datei je Tag: ", "der Dateiname bekommt das Datum – ZEIT_FORMAT$ liefert es. Kostet Platz, erlaubt aber, weiter zurückzugehen."));
children.push(bulletRich("Vor jeder gefährlichen Aktion: ", "zum Beispiel automatisch vor dem Einspielen einer Sicherung. Dann kann man auch das rückgängig machen."));
children.push(p("Für die zweite Variante genügt eine Zeile:"));
children.push(codeBlock([
  'DIM datei AS STRING',
  'datei = "sicherung_" + ZEIT_FORMAT$(ZEIT_JETZT(), "JJJJ-MM-TT") + ".db"',
]));
figure("kap11_sicherung.png", "Kapitel 11: zwei neue Knöpfe – und beim Start rüstet sich die Datenbank selbst nach.", 640, 380).forEach(e => children.push(e));

// ===================== Kapitel 12 =====================
children.push(chapter("Kapitel 12: Politur"));
children.push(tip("In diesem Kapitel",
  "Ein Balkendiagramm, Gold-Silber-Bronze und Tastenkürzel. Politur ist kein Beiwerk – ein Programm, das man gern benutzt, wird benutzt."));

children.push(p("Das Programm kann jetzt alles, was es können muss. Was fehlt, ist der Unterschied zwischen „funktioniert“ und „macht Freude“. Und der ist nicht kosmetisch: Nur ein Programm, das benutzt wird, zeigt, wo es noch hakt."));

children.push(h2("Dieselben Zahlen, anders angesehen"));
children.push(codeBlock([
  'IMPORT "chart"',
  "",
  "DIM diagramm AS CHART",
  'diagramm = CHART_NEW("balken", 60, 130, 860, 400)',
  'CHART_SET(diagramm, "titel", "Punkte je Mitspieler")',
  'CHART_SET(diagramm, "werte", "aussen")',
  'CHART_SET_NUM(diagramm, "abstand", 60)',
  "IF schrift >= 0 THEN CHART_SET_NUM(diagramm, \"schrift\", schrift)",
]));
children.push(p("Gefüllt wird das Diagramm in derselben Schleife, die auch die Rangliste füllt – aus einer Abfrage. Zwei Abfragen könnten auseinanderlaufen, und dann zeigt die Tabelle etwas anderes als das Bild daneben."));
children.push(codeBlock([
  "GUI_TABLE_ADD_ROW(rang, zeile)",
  "CHART_ADD(diagramm, DB_GET_STRING(r, 0), DB_GET_INT(r, 1) * 1.0)",
]));
children.push(pmix([
  ["Ein Diagramm ist kein Bedienelement der Oberfläche, sondern etwas, das auf die Leinwand gezeichnet wird. Deshalb bekommt es kein ", false],
  ["GUI_SET_TAB", true], [", sondern wird nur dann gezeichnet, wenn sein Reiter vorn ist:", false],
]));
children.push(codeBlock([
  "IF GUI_ACTIVE_TAB(win) = 2 THEN CHART_DRAW(diagramm)",
]));
figure("kap12_diagramm.png", "Kapitel 12: die Punkte als Balken – dieselben Zahlen wie in der Rangliste.", 640, 390).forEach(e => children.push(e));

children.push(h2("Gehalten ist nicht gedrückt"));
children.push(codeBlock([
  "IF KEYHIT(290) THEN sichern()             ' F1",
  "IF KEYHIT(291) THEN spielplanHolen()      ' F2",
]));
children.push(warn("Hier lauert eine Falle, in die jeder einmal tritt: KEYPRESSED ist wahr, SOLANGE eine Taste gehalten wird. Bei sechzig Bildern je Sekunde wären das sechzig Sicherungen pro Sekunde. KEYHIT ist nur in dem einen Bild wahr, in dem die Taste heruntergedrückt wurde."));
children.push(p("Und schließlich die Farben der Rangliste: Gold, Silber, Bronze. Dieselbe Ordnung, die jeder vom Podest kennt – drei Zeilen Code, und plötzlich sieht man den Führenden, statt ihn zu lesen."));
children.push(codeBlock([
  "IF platz = 1 THEN GUI_TABLE_ROW_COLOR(rang, platz - 1, &HF4E3A6, -1)",
  "IF platz = 2 THEN GUI_TABLE_ROW_COLOR(rang, platz - 1, &HDCE0E6, -1)",
  "IF platz = 3 THEN GUI_TABLE_ROW_COLOR(rang, platz - 1, &HE0BE96, -1)",
]));
children.push(h2("Welches Diagramm passt?"));
children.push(p("Die Wahl der Diagrammart ist keine Geschmacksfrage, sondern eine Aussage darüber, was die Zahlen bedeuten:"));
children.push(bulletRich("Balken – ", "Werte je Kategorie vergleichen. Genau unser Fall: Wer hat mehr Punkte?"));
children.push(bulletRich("Kuchen – ", "Anteile an einem Ganzen. Passt hier nicht: Punkte sind kein Kuchen, den man aufteilt – wenn Anna mehr bekommt, bekommt Ben deshalb nicht weniger."));
children.push(bulletRich("Linie – ", "Verläufe über die Zeit. Wäre die richtige Wahl, sobald wir mehrere Spieltage haben: Punktestand je Spieltag, eine Linie je Mitspieler."));
children.push(p("Ein Kuchendiagramm der Punkte wäre also nicht bloß hässlich, sondern eine falsche Behauptung über die Daten. Man sieht solche Diagramme trotzdem ständig – und genau deshalb lohnt es sich, einmal darüber nachzudenken."));
children.push(tip("Zum Ausprobieren", "Ändere CHART_NEW(\"balken\", ...) zu \"kuchen\" und schau es dir an. Es funktioniert – und behauptet etwas, das nicht stimmt. Danach wieder zurück."));
figure("kap12_rangliste.png", "Kapitel 12: Gold, Silber, Bronze.", 640, 380).forEach(e => children.push(e));

// ===================== Kapitel 13 =====================
children.push(chapter("Kapitel 13: Weitergeben"));
children.push(tip("In diesem Kapitel",
  "Aus dem Programm wird eine .exe, die auf einem fremden Rechner läuft – ohne Drachenhauch, ohne Editor, ohne irgendetwas."));

children.push(p("Das Programm läuft. Jetzt soll es bei jemand anderem laufen. Das ist eine einzige Zeile:"));
children.push(codeBlock([
  "dhrt --export buch-tippspiel/code/tippspiel.dh",
]));
children.push(p("Das legt einen Ordner tippspiel_dist mit einer .exe an. Die Runtime steckt darin, das übersetzte Programm auch, und referenzierte Dateien werden mitkopiert. Der Empfänger braucht weder Drachenhauch noch Python noch einen Editor – er bekommt ein Fenster, das aufgeht."));

children.push(h2("Wo landen die Daten?"));
children.push(p("Eine Frage, die vorher nie eine war, wird jetzt zu einer: Unser Programm öffnet „tippspiel.db“ – aber neben welcher Datei landet die?"));
children.push(p("Die naheliegende Sorge lautet: im aktuellen Verzeichnis. Welches das ist, entscheidet dann nicht das Programm, sondern derjenige, der es startet. Doppelklick, Verknüpfung und Kommandozeile können drei verschiedene Ordner meinen. Im schlimmsten Fall entstünde bei jedem Start eine neue leere Datenbank, und der Benutzer meldet: „Meine Tipps sind weg.“"));
children.push(p("Nachgesehen statt angenommen: Drachenhauch löst relative Pfade relativ zum Programm auf, nicht zum Startverzeichnis – auch bei der exportierten .exe. Ich habe es gemessen, indem ich eine exportierte .exe in einen fremden Ordner kopiert und von woanders gestartet habe. Die Datenbank entstand neben der .exe."));
children.push(why("Weil eine Annahme, die man nicht geprüft hat, in einer ausgelieferten Anwendung teuer wird. Der Unterschied zwischen „ich glaube, das passt“ und „ich habe nachgesehen“ sind hier zehn Minuten – und im Zweifel die Tipps einer ganzen Saison.", "Warum nachmessen statt nachdenken?"));
children.push(h2("Was im Ordner liegt – und was hineingehört"));
children.push(p("Der Export legt eine .exe an und kopiert die Dateien mit, die das Programm nachweislich benutzt. Was er nicht wissen kann, sind Dateien, deren Namen erst zur Laufzeit entstehen. Vor der ersten Weitergabe lohnt sich deshalb ein Blick in den Ordner – und eine kurze Liste:"));
children.push(bulletRich("Die .exe – ", "enthält Runtime und Programm, sonst nichts."));
children.push(bulletRich("Bilder und andere Dateien – ", "was das Programm lädt, gehört daneben."));
children.push(bulletRich("Die Datenbank – ", "besser NICHT mitgeben. Sie entsteht beim ersten Start neu; eine mitgelieferte enthielte deine Testtipps."));
children.push(bulletRich("Eine kurze Anleitung – ", "zwei Sätze reichen: Was das Programm tut, wie man es startet, wo die Daten liegen."));
children.push(h2("Der erste Start bei jemand anderem"));
children.push(p("Es lohnt sich, diesen Moment einmal selbst nachzuspielen – in einem leeren Ordner, auf einem anderen Rechner, oder wenigstens unter einem anderen Benutzerkonto. Alles, was du dabei über dein eigenes Programm lernst, lernst du sonst aus einer Nachricht, die mit „bei mir geht das nicht“ anfängt."));
children.push(tip("Zum Ausprobieren", "Exportiere den Zielstand, kopiere den Ordner an eine andere Stelle und starte die .exe per Doppelklick. Beim ersten Start legt sie ihre Datenbank selbst an – genau dafür steht seit Kapitel 2 in jedem Stand ein CREATE TABLE IF NOT EXISTS."));

// --- Schluss ---
children.push(chapter("Und jetzt?"));
children.push(p("Du hast eine Anwendung gebaut. Nicht ein Beispiel, nicht ein Fragment – ein Programm mit Datenbank, Regeln, Oberfläche, Netzanbindung, Fehlerbehandlung, Sicherung und einer .exe zum Weitergeben. Wenn du dabei mitgetippt hast, kannst du das jetzt wieder."));
children.push(p("Was als Nächstes kommt, hängt von deiner Tipprunde ab. Ein paar Ideen, jede davon ungefähr ein Abend Arbeit – und mit allem, was in diesem Buch steht, ist keine davon mehr fremd:"));
children.push(bulletRich("Mehrere Spieltage: ", "die Abfragen haben die Spalte spieltag längst; es fehlt nur eine Auswahl im Fenster."));
children.push(bulletRich("Eine Vereins-Tabelle: ", "damit „Bor.“ und „Borussia“ derselbe Verein sind – siehe die Warnung in Kapitel 9."));
children.push(bulletRich("Ergebnisse automatisch holen: ", "derselbe Abruf wie beim Spielplan, nur ein anderes Feld aus der Antwort."));
children.push(bulletRich("Mehrere Tipprunden: ", "eine Tabelle runden, eine Spalte mehr in tipps – und plötzlich können Büro und Familie getrennt tippen."));
children.push(bulletRich("Eine Wertung mit Bonus: ", "Punkte für den ganzen Spieltag richtig? Die Regel steht an einer Stelle. Du weißt, an welcher."));
children.push(p("Und wenn du irgendwann ein anderes Programm baust – eine Vereinskasse, ein Vokabelheft, eine Terminliste – wirst du feststellen, dass es dieselben Fragen sind. Wo leben die Daten? Wer darf sie ändern? Was passiert, wenn etwas schiefgeht? Die Antworten kennst du jetzt."));
children.push(new Paragraph({
  spacing: { before: 240 },
  children: [new TextRun({ text: "— Viel Erfolg in der neuen Saison.", italics: true, size: 22, color: C_CAP })],
}));

// ===================== Inhaltsverzeichnis einsetzen =====================
// Seitenzahlen kommen aus toc_pages.json (vom Zwei-Pass-Build make_book.py).
// Fehlt die Datei, wird das Verzeichnis ohne Zahlen gebaut -- das Dokument
// entsteht trotzdem.
let pages = {};
const pagesFile = path.join(__dirname, "toc_pages.json");
if (fs.existsSync(pagesFile)) {
  try { pages = JSON.parse(fs.readFileSync(pagesFile, "utf8")); } catch (e) { pages = {}; }
}
const toc = [
  new Paragraph({
    heading: HeadingLevel.HEADING_1, spacing: { after: 200 },
    border: { bottom: { style: BorderStyle.SINGLE, size: 12, color: C_ACCENT, space: 4 } },
    children: [new TextRun({ text: "Inhalt" })],
  }),
];
for (const e of tocEntries) {
  const nr = pages[e.title];
  toc.push(new Paragraph({
    spacing: { after: 60 },
    tabStops: [{ type: TabStopType.RIGHT, position: 9000, leader: LeaderType.DOT }],
    children: [
      new TextRun({ text: e.title, size: 22 }),
      ...(nr ? [new TextRun({ children: [new Tab()], size: 22 }), new TextRun({ text: String(nr), size: 22 })] : []),
    ],
  }));
}
toc.push(new Paragraph({ children: [new PageBreak()] }));
children.splice(TOC_INSERT_AT, 0, ...toc);

// Titelliste fuer den Zwei-Pass-Build mitschreiben.
fs.writeFileSync(path.join(__dirname, "toc_titles.json"),
  JSON.stringify(tocEntries.map(e => e.title), null, 2), "utf8");

// ===================== Dokument =====================
const doc = new Document({
  creator: "Hans Schnorrenberger",
  title: "Drachenhauch – Die erste richtige Anwendung",
  description: "Ein Bundesliga-Tippspiel, Schritt für Schritt mit Drachenhauch gebaut.",
  numbering: {
    config: [{
      reference: "bul",
      levels: [{
        level: 0, format: LevelFormat.BULLET, text: "•", alignment: AlignmentType.LEFT,
        style: { paragraph: { indent: { left: 400, hanging: 200 } } },
      }],
    }],
  },
  styles: {
    default: {
      document: { run: { font: "Calibri", size: 22 } },
      heading1: { run: { font: "Calibri", size: 34, bold: true, color: C_TITLE },
                  paragraph: { spacing: { before: 320, after: 80 } } },
      heading2: { run: { font: "Calibri", size: 26, bold: true, color: C_H2 },
                  paragraph: { spacing: { before: 240, after: 60 } } },
    },
  },
  sections: [{
    properties: {
      page: { margin: { top: 1000, bottom: 1000, left: 1100, right: 1100 } },
      // Die Titelseite bekommt eine eigene (leere) Kopf- und Fusszeile --
      // ein Titel, ueber dem noch einmal der Titel steht, sieht nach Versehen aus.
      titlePage: true,
    },
    headers: {
      default: new Header({ children: [new Paragraph({
        alignment: AlignmentType.RIGHT,
        children: [new TextRun({ text: "Drachenhauch – Die erste richtige Anwendung", size: 16, color: C_CAP })],
      })] }),
      first: new Header({ children: [new Paragraph({ children: [] })] }),
    },
    footers: {
      default: new Footer({ children: [new Paragraph({
        alignment: AlignmentType.CENTER,
        children: [new TextRun({ children: [PageNumber.CURRENT], size: 18, color: C_CAP })],
      })] }),
      first: new Footer({ children: [new Paragraph({ children: [] })] }),
    },
    children,
  }],
});

const ZIEL = path.join(__dirname, "Drachenhauch-Tippspiel.docx");
Packer.toBuffer(doc).then(buf => {
  fs.writeFileSync(ZIEL, buf);
  const kb = Math.round(buf.length / 1024);
  console.log(`Geschrieben: ${path.basename(ZIEL)} (${kb} KB, ${tocEntries.length} Verzeichnis-Eintraege)`);
});
