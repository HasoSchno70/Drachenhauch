// Baut die Einleitung des GameBasic-Galaga-Buchs als farbiges, druckbares .docx.
// Aufruf:  node build_book.js   ->  GameBasic-Buch.docx
const fs = require("fs");
const path = require("path");
const {
  Document, Packer, Paragraph, TextRun, ImageRun, AlignmentType, HeadingLevel,
  BorderStyle, Table, TableRow, TableCell, WidthType, ShadingType, PageBreak,
  Header, Footer, PageNumber, LevelFormat,
  Tab, TabStopType, LeaderType,
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

// "Warum so?"-Kasten (Design-Begruendung) – amber statt blau, klar vom tip() unterscheidbar.
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

// Inhaltsverzeichnis: jede h1()/chapter()-Ueberschrift wird hier registriert
// (Titel + eindeutiges Bookmark fuer den klickbaren Sprung). Die Seitenzahlen
// liefert der Zwei-Pass-Build (make_book.py) ueber toc_pages.json.
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
// Kapitel-Ueberschrift: beginnt immer auf einer neuen Seite (pageBreakBefore
// vermeidet Leerseiten, falls das vorige Kapitel genau am Seitenende endete).
function chapter(text) { return _heading(text, true); }

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

// --- Inhaltsverzeichnis ---
// Hier kommt das fertige Verzeichnis hin. Es wird erst NACH dem Aufbau aller
// Kapitel zusammengesetzt (dann sind alle tocEntries bekannt) und an dieser
// Stelle eingefuegt.
const TOC_INSERT_AT = children.length;

// --- Vorwort ---
children.push(h1("Vorwort"));
children.push(p("Es gibt zwei Sorten von Menschen, die Computerspiele lieben: die einen wollen sie spielen, die anderen wollen wissen, wie zum Teufel das eigentlich funktioniert. Wenn du dieses Buch in der Hand hältst, gehörst du vermutlich zur zweiten Sorte – oder du bist gerade dabei, hinüberzuwechseln. Herzlich willkommen, hier ist genau dein Platz."));
children.push(p("Ich heiße Hans Schnorrenberger, und ich habe GameBasic gebaut, weil ich es leid war, Anfängern erklären zu müssen, warum man für ein simples bewegtes Pixel auf dem Bildschirm erst einmal drei Bibliotheken installieren, ein halbes Dutzend Fehlermeldungen entschlüsseln und an einer Stelle „aus Gründen“ ein Semikolon setzen muss. Programmieren sollte sich nicht wie eine Mutprobe anfühlen. Es sollte sich anfühlen wie das, was es eigentlich ist: das Bauen kleiner Welten, in denen du die Regeln machst."));
children.push(p("Deshalb ist GameBasic so geworden, wie es ist. Die Befehle heißen so, wie sie gemeint sind. Wenn etwas schiefgeht, sagt dir die Sprache in ganzen, lesbaren Sätzen, was los ist – und nicht in kryptischem Fachchinesisch, das man erst googeln muss, um zu verstehen, dass man es googeln muss. Und wenn dein Programm läuft, dann läuft es flott, weil im Hintergrund eine richtige Runtime werkelt und nicht bloß gute Laune."));
children.push(p("Dieses Buch baut mit dir gemeinsam ein vollständiges Arcade-Spiel: einen Klon des unsterblichen Klassikers Galaga. Wir fangen ganz vorne an – mit einem leeren schwarzen Fenster, das tapfer darauf wartet, dass etwas passiert – und hören erst auf, wenn bunte Gegner in geschwungenen Bahnen einfliegen, herabstürzen, Bomben werfen und du sie mit einem selbstgezeichneten Raumschiff zu Pixelstaub schießt. Unterwegs lernst du, fast ohne es zu merken, alles Wichtige über das Programmieren."));
children.push(p("Ein einziger Rat noch, bevor es losgeht: Tippe die Beispiele wirklich selbst ab. Ich weiß, Kopieren wäre schneller. Aber Programmieren lernt man in den Fingern, nicht in den Augen – ungefähr so, wie man Fahrradfahren nicht aus einem Buch über Fahrräder lernt. Mach Fehler, viele sogar. Jeder Fehler, den du selbst gefunden und behoben hast, ist eine Lektion, die sitzt."));
children.push(p("Genug der Vorrede. Schnall dich an, der Hangar ist offen."));
children.push(new Paragraph({
  spacing: { before: 240 },
  children: [new TextRun({ text: "— Hans Schnorrenberger", italics: true, size: 22, color: C_CAP })],
}));

// --- Einleitung ---
children.push(h1("Willkommen!"));
children.push(p("Dieses Buch nimmt dich an die Hand und baut mit dir gemeinsam ein komplettes Arcade-Spiel – einen Klon des Klassikers Galaga. Und mit „komplett“ meine ich komplett: kein abgespecktes Spielzeug-Beispiel, an dessen Ende du dich fragst, wo denn nun das eigentliche Spiel geblieben ist, sondern etwas, das man tatsächlich spielen, verlieren und genervt-aber-glücklich noch einmal von vorne starten will."));
children.push(p("Du brauchst dafür keinerlei Vorkenntnisse. Wirklich keine. Wenn du weißt, wie man einen Computer einschaltet und Text eintippt, bist du qualifiziert. Alles andere bauen wir gemeinsam auf – Stein für Stein, oder besser gesagt: Pixel für Pixel."));
children.push(p("Der Trick dabei ist, dass wir nie auf den großen Knall am Ende warten. Jedes Kapitel fügt ein kleines, sofort sichtbares Stück hinzu: erst ein Fenster, dann ein scrollender Sternenhimmel, dann ein Raumschiff, das auf deine Tasten hört, dann Schüsse, Gegner, geschwungene Einflug-Manöver, Sturzangriffe, Bomben. Am Ende hast du ein richtiges Spiel mit Highscore-Liste, Sound und Effekten – aber schon nach Kapitel 1 läuft etwas, das sich gut anfühlt."));
children.push(p("Programmieren lernt man nämlich am besten, indem man etwas baut, das Spaß macht. Lehrbücher, die einem zur Begrüßung beibringen, wie man die Zahlen von 1 bis 10 untereinander ausdruckt, haben schon so manchen begeisterten Anfänger in einen gelangweilten Aussteiger verwandelt. Das passiert uns hier nicht."));
children.push(p("Und das Beste kommt noch: Die Grafik – Raumschiff, Gegner, Schüsse, Bomben – zeichnest du selbst, mit einem Pixel-Editor, der gleich mitgeliefert wird. Dein Spiel wird also nicht irgendein Spiel sein, sondern ganz und gar deins. Wenn dein Raumschiff am Ende aussieht wie eine fliegende Bratwurst, dann ist das eine bewusste künstlerische Entscheidung, und niemand darf dir widersprechen."));
children.push(tip("So liest du dieses Buch", "Tippe die Beispiele selbst ab und starte sie sofort – nicht erst am Ende des Kapitels, sondern zwischendurch immer wieder. Jeder Codeschnipsel läuft für sich. Und keine Sorge vor Fehlern: Die gehören dazu wie das Salz in die Suppe. GameBasic sagt dir in klaren, ganzen Sätzen, was es nicht verstanden hat – meistens hast du nur einen Buchstaben vertippt."));

// --- Was ist GameBasic ---
children.push(h1("Was ist GameBasic?"));
children.push(p("GameBasic ist eine Programmiersprache aus der ehrwürdigen BASIC-Familie. BASIC steht für „Beginner's All-purpose Symbolic Instruction Code“ – ein etwas sperriger Name für eine erstaunlich freundliche Idee: eine Sprache, die bewusst so leicht lesbar ist, dass sie sich fast wie englische Sätze liest. Generationen von Programmierern haben in den achtziger Jahren auf Heimcomputern mit BASIC angefangen, und viele von ihnen erinnern sich bis heute mit feuchten Augen daran."));
children.push(p("GameBasic nimmt diese alte, gute Idee und schleppt sie ins 21. Jahrhundert. Es ist von Grund auf für Spiele gemacht: Grafik, Sound, Eingabe und Spielablauf sind direkt eingebaut. Du musst dir nicht erst aus dem halben Internet Bibliotheken zusammensuchen, von denen die Hälfte nicht zusammenpasst und die andere Hälfte seit drei Jahren nicht mehr gepflegt wird. Du schreibst SCREEN, und ein Fenster geht auf. So soll es sein."));
children.push(h2("Was sie besonders macht"));
children.push(bulletRich("Einfach zu lesen: ", "Befehle wie SCREEN, PLOT, DRAWIMAGE oder PLAYSOUND sagen, was sie tun."));
children.push(bulletRich("Sicher durch Typen: ", "Jede Variable hat einen klaren Typ (INTEGER, FLOAT, STRING …). Das verhindert viele Anfängerfehler."));
children.push(bulletRich("Modern: ", "Klassen und Objekte (OOP), Funktionen, Module – alles dabei, wenn du es brauchst, aber nie im Weg."));
children.push(bulletRich("Eine Laufzeit: ", "Dein Programm läuft direkt über die schnelle Runtime „dhrt“ – flüssig und auf Wunsch als fertige .exe exportierbar."));
children.push(p("Genug der Theorie – schau dir an, wie wenig nötig ist, um etwas auf den Bildschirm zu bringen. Das hier ist ein vollständiges, lauffähiges GameBasic-Programm:"));
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
children.push(p("Drei Zeilen. Die erste öffnet ein Fenster, die zweite schreibt in leuchtendem Gelb „Hallo Welt!“ hinein, die dritte zeigt das Ergebnis an. Man muss kein Hellseher sein, um zu erraten, was hier passiert – und genau das ist der Punkt. In manch anderer Programmiersprache wäre dasselbe „Hallo Welt“ eine kleine Expedition mit Zwischenlager und Sherpa gewesen."));

// --- Was kann GameBasic ---
children.push(h1("Was kann GameBasic alles?"));
children.push(p("Erstaunlich viel – ehrlich gesagt weit mehr, als wir für unser bescheidenes Galaga-Spiel jemals brauchen werden. Das ist ein bisschen so, als würde man den Führerschein in einem Sportwagen machen: Wir nutzen längst nicht jede Pferdestärke, aber es ist beruhigend zu wissen, dass sie da ist, falls dich später der Ehrgeiz packt."));
children.push(p("Ein kleiner Vorgeschmack, was alles eingebaut ist:"));
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
children.push(p("Im Jahr 1981 stellte die Firma Namco einen Spielautomaten in die Hallen, der die Welt im Sturm eroberte: Galaga. Wer alt genug ist, hat dafür echte Münzen geopfert; wer es nicht ist, kennt es spätestens als das Spiel, das in Filmen immer dann im Hintergrund flimmert, wenn jemand cool sein soll. Es ist einer der berühmtesten Arcade-Shooter überhaupt – und das aus gutem Grund."));
children.push(p("Das Prinzip ist herrlich einfach: Unten steuerst du ein einzelnes Raumschiff, das nur nach links und rechts darf. Oben schwebt eine Formation bunter Gegner. Sie bleiben aber nicht brav oben sitzen – sie fliegen in eleganten, geschwungenen Bahnen ein, lösen sich einzeln aus der Reihe, stürzen auf dich herab und werfen Bomben. Du wiederum schießt nach oben und versuchst, am Leben zu bleiben. Klingt nach wenig, ist aber genau die richtige Mischung aus „eigentlich schaffe ich das“ und „verflixt, noch ein Versuch“."));
children.push(p("Genau dieses Gefühl bauen wir nach – mit allem Drum und Dran. Und keine Sorge: Wir verletzen dabei keine Urheberrechte. Wir kopieren nicht das Original, wir lernen aus seinem Aufbau und bauen unsere eigene Variante, mit unseren eigenen Grafiken und unseren eigenen Tönen."));
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
children.push(p("Wir bauen das Spiel in kleinen, lauffähigen Schritten. Das ist Absicht und kein Zufall: Nichts ist demotivierender, als hundert Zeilen Code abzutippen, am Ende auf „Start“ zu drücken und dann nur eine Fehlermeldung zu ernten, ohne die geringste Ahnung, welche der hundert Zeilen die Schuldige ist."));
children.push(p("Deshalb hat jedes Kapitel ein klares, sichtbares Ziel. Nach jedem Kapitel hast du etwas, das du sofort starten und ausprobieren kannst – und das dir Lust auf das nächste Kapitel macht. Der grobe Fahrplan sieht so aus:"));
children.push(bulletRich("Erst das Fenster: ", "Spielschleife, Sternenhimmel."));
children.push(bulletRich("Dann das Schiff: ", "laden, zeichnen, mit der Tastatur bewegen."));
children.push(bulletRich("Sprites selbst zeichnen: ", "im Pixel-Editor dhsprites."));
children.push(bulletRich("Schiessen, Gegner, Formation: ", "Arrays, Klassen, Bewegung."));
children.push(bulletRich("Einflug, Stürze, Bomben, Kollisionen: ", "das eigentliche Spielgefühl."));
children.push(bulletRich("Politur: ", "Sound, Effekte, Highscores, Export als .exe."));
children.push(bulletRich("Automaten-Schliff: ", "Fenstersymbol, Rüttel-Feedback im Gamepad, Pause beim Wegklicken – und ein Attract-Modus, in dem sich das Spiel selbst vorführt."));
children.push(tip("Alles in Farbe und zum Selbermachen", "Dieses Buch ist als Word-Dokument angelegt. Du kannst es nach Belieben ergänzen, umstellen, eigene Screenshots einfügen – und natürlich ausdrucken."));

// ===================== Kapitel 1 =====================
children.push(chapter("Kapitel 1: Das erste Fenster"));
children.push(tip("In diesem Kapitel",
  "Du öffnest dein allererstes GameBasic-Fenster, lernst die „Spielschleife“ kennen und zauberst einen scrollenden Sternenhimmel – die Bühne, auf der später unser Raumschiff kämpft."));

children.push(h2("Ein Fenster öffnen"));
children.push(p("Jedes Spiel braucht eine Bühne, und auf dem Computer ist diese Bühne ein Fenster. Bevor irgendetwas gezeichnet, bewegt oder abgeschossen werden kann, muss dieses Fenster da sein. In manchen Sprachen ist das der Moment, in dem Anfänger zum ersten Mal aufgeben. In GameBasic ist es eine einzige Zeile:"));
children.push(codeBlock(['SCREEN(480, 640, "Mein Galaga")']));
children.push(pmix([
  ["Der Befehl ", false], ["SCREEN", true],
  [" öffnet ein Fenster: zuerst die Breite (480), dann die Höhe (640), dann der Titel in der Fensterleiste. Wir wählen ein ", false],
  ["Hochformat", false], [" – schmal und hoch, wie ein Arcade-Automat.", false],
]));

children.push(h2("Die Spielschleife"));
children.push(p("Jetzt kommt die wichtigste Idee in diesem ganzen Buch – wenn du nur eine Sache mitnimmst, dann diese. Ein Spiel steht nämlich nie still. Selbst wenn auf dem Bildschirm scheinbar nichts passiert, arbeitet das Spiel im Hintergrund pausenlos weiter: Es fragt die Tastatur ab, bewegt die Sterne, prüft auf Zusammenstöße und zeichnet alles neu. Und zwar viele Male pro Sekunde – flüssige 60 Mal sind das Ziel."));
children.push(p("Ein Film besteht aus vielen Einzelbildern, die so schnell hintereinander gezeigt werden, dass unser Auge eine Bewegung sieht. Bei einem Spiel ist es genau dasselbe – nur dass jedes Einzelbild nicht vorher aufgenommen, sondern in dem Augenblick frisch berechnet wird. Ein solches Einzelbild nennt man auf Englisch einen Frame, und dieses Wort wird uns im ganzen Buch begleiten."));
children.push(p("Das Herzstück, das diese Bilder am Fließband produziert, ist die sogenannte Spielschleife. „Schleife“ heißt: etwas wird immer und immer wieder gemacht. Unsere Schleife läuft so lange, bis du das Fenster schließt:"));
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
children.push(p("Bevor wir ein neues Bild malen, müssen wir das alte loswerden. Täten wir das nicht, würde sich jeder Frame über den vorigen legen, und nach einer Sekunde wäre der Bildschirm ein einziges Schmierbild aus übereinander gepinselten Sternen. Stell es dir vor wie eine Tafel: Bevor der Lehrer etwas Neues anschreibt, wischt er erst einmal das Alte weg."));
children.push(pmix([
  ["Genau das macht ", false],
  ["CLS", true], [" – die Abkürzung steht für das englische „clear screen“. Wir übergeben gleich noch eine Farbe, mit der der ganze Bildschirm gefüllt wird:", false],
]));
children.push(codeBlock(['CLS(&H05060F)        \' dunkles Weltraum-Blau']));
children.push(pmix([
  ["Farben schreibt man als ", false], ["&Hrrggbb", true],
  [" – je zwei Stellen für Rot, Grün, Blau. ", false],
  ["&H05060F", true], [" ist ein sehr dunkles Blau, perfekt fürs All.", false],
]));

children.push(h2("Ein Sternenhimmel"));
children.push(p("Ein leeres dunkelblaues Fenster ist ein guter Anfang, aber niemand würde dafür eine Münze in den Automaten werfen. Geben wir dem All also ein paar Sterne. Das Schöne daran: Sterne sind das einfachste Spielelement überhaupt – einfach viele leuchtende Punkte. Wenn du Sterne hinbekommst, bekommst du später auch Schüsse, Bomben und Explosionen hin, denn im Grunde sind das alles nur Punkte, die sich bewegen und gelegentlich gegen etwas stoßen."));
children.push(p("Wir wollen viele Sterne, sagen wir sechzig. Sechzig einzelne Variablen anzulegen wäre allerdings eine Strafarbeit. Stattdessen nehmen wir eine Liste – im Fachjargon ein Array. Ein Array ist wie ein Setzkasten mit nummerierten Fächern: Fach 0, Fach 1, Fach 2 und so weiter. Wir merken uns die Positionen der Sterne in zwei solchen Listen – eine für die x-Koordinate, eine für die y-Koordinate – und verteilen sie zu Beginn zufällig über den Himmel:"));
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
children.push(chapter("Kapitel 2: Das Schiff"));
children.push(tip("In diesem Kapitel",
  "Du holst dein Raumschiff auf den Bildschirm: ein fertiges Bild laden, an die richtige Stelle zeichnen und mit den Pfeiltasten nach links und rechts steuern."));

children.push(h2("Ein Bild laden"));
children.push(p("Ein Weltraum ohne Held ist nur Tapete. Höchste Zeit für unser Raumschiff. Das Raumschiff ist ein kleines Bild – im Spielejargon ein „Sprite“. Das Wort stammt übrigens von den geisterhaften Lichtgestalten alter englischer Sagen; bei uns ist es schlicht eine kleine Grafik, die sich frei über den Bildschirm bewegt."));
children.push(p("Im nächsten Kapitel zeichnest du dein Schiff höchstpersönlich. Fürs Erste nehmen wir aber das mitgelieferte – damit du dich ganz auf das Programmieren konzentrieren kannst und nicht gleichzeitig auch noch Künstler sein musst. Ein Bild laden wir mit dem Befehl LOADIMAGE und merken es uns in einer Variablen vom Typ IMAGE:"));
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
children.push(p("Ein geladenes Bild allein sieht man noch nicht – es liegt sozusagen im Hangar und wartet. Wir müssen GameBasic noch sagen, wo auf dem Bildschirm es erscheinen soll. Dafür braucht das Schiff eine Position, und eine Position sind schlicht zwei Zahlen: wie weit rechts (x) und wie weit unten (y)."));
children.push(p("Ein kleiner, aber wichtiger Hinweis, der viele Anfänger anfangs stolpern lässt: Am Computer wächst die y-Achse nach unten. Oben ist klein, unten ist groß. Das fühlt sich verkehrt herum an, ist aber überall so. Unser Schiff gehört nach unten in die Mitte, also nehmen wir ein kleines x und ein großes y. Gezeichnet wird dann mit DRAWIMAGE:"));
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
children.push(p("Probiere ruhig einmal aus, das Schiff ganz nach rechts zu fahren. Es verschwindet seelenruhig aus dem Fenster und denkt gar nicht daran, zurückzukommen – Computer nehmen Anweisungen eben wörtlich. Wir haben gesagt „bei jedem Tastendruck vier Pixel weiter“, und genau das tut es, bis in alle Ewigkeit."));
children.push(p("Wir müssen dem Schiff also Grenzen setzen, so wie man einem übermütigen Hund einen Zaun spendiert. Zwei kurze Prüfungen genügen, und das Schiff bleibt brav im Fenster:"));
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

// ===================== Kapitel 3 =====================
children.push(chapter("Kapitel 3: Sprites selbst zeichnen"));
children.push(tip("In diesem Kapitel",
  "Du lernst den mitgelieferten Pixel-Editor dhsprites kennen und zeichnest deine eigene Grafik: das Raumschiff und einen Gegner mit Flügelschlag-Animation. Wer lieber sofort weiterprogrammiert, nimmt einfach die fertigen Sprites aus dem assets-Ordner – beides funktioniert."));

children.push(h2("Was ist ein Sprite?"));
children.push(p("Dieses Kapitel ist eine kleine Verschnaufpause vom Programmieren – wir tauschen die Tastatur kurz gegen den Pinsel. Wer es eilig hat und sofort weiterprogrammieren will, darf das ganze Kapitel überspringen und einfach die fertigen Sprites aus dem Projekt benutzen. Aber unter uns: Selbst zu zeichnen macht riesigen Spaß, und es ist der Moment, in dem aus „irgendeinem Spiel“ dein Spiel wird."));
children.push(p("Ein Sprite ist ein kleines Bild aus einzelnen farbigen Pixeln. Stell dir kariertes Papier vor, bei dem jedes Kästchen genau eine Farbe bekommt – mehr ist es nicht. Das klingt nach wenig, aber die gesamte goldene Ära der Videospiele wurde aus genau solchen Kästchen gebaut. Unser Schiff ist gerade einmal 16 × 16 Pixel groß; das sind 256 Kästchen, mit denen man erstaunlich viel anstellen kann. Stark vergrößert sieht man die einzelnen Pixel ganz deutlich:"));
figure("kap03_pixelraster.png", "Das Raumschiff als Pixel-Raster – jedes Kästchen ist ein Pixel.", 280, 280).forEach(e => children.push(e));

children.push(h2("Der Sprite-Editor dhsprites"));
children.push(pmix([
  ["GameBasic bringt einen eigenen Pixel-Editor mit. Du startest ihn mit dem Befehl ", false],
  ["dhsprites", true],
  [" – entweder leer für ein neues Bild oder mit einer Datei: ", false],
  ["dhsprites player.png", true], [".", false],
]));
children.push(p("Die wichtigsten Werkzeuge:"));
children.push(bulletRich("Stift ", "setzt einzelne Pixel in der gewählten Farbe."));
children.push(bulletRich("Radierer ", "löscht Pixel (macht sie durchsichtig)."));
children.push(bulletRich("Füller (Eimer) ", "füllt eine zusammenhängende Fläche."));
children.push(bulletRich("Pipette ", "übernimmt eine Farbe aus dem Bild."));
children.push(bulletRich("Palette ", "deine Farbauswahl – Klick wählt die aktive Farbe."));
children.push(bulletRich("Symmetrie X ", "spiegelt deine Striche links/rechts – ideal für ein symmetrisches Raumschiff: du zeichnest nur die Hälfte."));

children.push(h2("Das Schiff zeichnen"));
children.push(p("So gehst du vor:"));
children.push(new Paragraph({ numbering: { reference: "num3", level: 0 }, spacing: { after: 60 }, children: [new TextRun({ text: "Neues Bild mit 16 × 16 Pixeln anlegen.", size: 22 })] }));
children.push(new Paragraph({ numbering: { reference: "num3", level: 0 }, spacing: { after: 60 }, children: [new TextRun({ text: "Symmetrie X einschalten – dann genügt es, die linke Hälfte zu malen.", size: 22 })] }));
children.push(new Paragraph({ numbering: { reference: "num3", level: 0 }, spacing: { after: 60 }, children: [new TextRun({ text: "Mit dem Stift die Umrisse (hell) setzen, dann den Rumpf (blau) füllen.", size: 22 })] }));
children.push(new Paragraph({ numbering: { reference: "num3", level: 0 }, spacing: { after: 60 }, children: [new TextRun({ text: "Ein gelbes Cockpit in die Mitte, rote Triebwerke nach unten – fertig.", size: 22 })] }));
children.push(tip("Tipp", "Weniger ist mehr: Bei 16 × 16 Pixeln zählt jedes Kästchen. Wenige klare Farben wirken besser als viele Details."));

children.push(h2("Animation mit Frames"));
children.push(p("Ein Gegner, der nur starr im Raum hängt, wirkt wie ein ausgestopftes Tier im Museum – korrekt, aber leblos. Lebendig wird er erst, wenn er mit den Flügeln schlägt. Und das Geheimnis hinter jeder Animation, vom Daumenkino bis zum Kinofilm, ist immer dasselbe: mehrere leicht unterschiedliche Einzelbilder, schnell hintereinander gezeigt."));
children.push(p("Solche Einzelbilder heißen – du ahnst es bereits aus Kapitel 1 – Frames. Das Beruhigende: Unser Gegner braucht keine aufwändige Daumenkino-Sequenz, sondern nur zwei Frames. Flügel oben, Flügel unten. Im Wechsel gezeigt, ergibt das einen überzeugenden Flügelschlag, und dein Gehirn erledigt den Rest:"));
figure("kap03_frames.png", "Zwei Frames ergeben den Flügelschlag – schnell abgewechselt wirkt es wie Bewegung.", 360, 200).forEach(e => children.push(e));
children.push(pmix([
  ["Im Editor legst du über die Frame-Liste ein zweites Bild an. Die ", false],
  ["Zwiebelschalen-Ansicht", false],
  [" (Onion-Skin) zeigt das vorige Frame blass im Hintergrund – so triffst du die Bewegung leichter.", false],
]));

children.push(h2("Speichern und exportieren"));
children.push(pmix([
  ["Speichere dein Werk als ", false], [".dhsprite", true],
  [" – so kannst du es später weiter bearbeiten (mit allen Frames). Fürs Spiel exportierst du zusätzlich ein ", false],
  ["PNG", true], [": ein einzelnes Bild beim Schiff, oder bei mehreren Frames ein ", false],
  ["Sheet", false], [" (alle Frames nebeneinander in einem Bild).", false],
]));
children.push(p("Genau diese PNGs liegen schon fertig im Projekt – das sind die Sprites, die wir verwenden:"));
figure("kap03_alle_sprites.png", "Unsere Sprites: Schiff, drei Gegner (je 2 Frames), Schuss und Bombe.", 460, 200).forEach(e => children.push(e));

children.push(h2("Im Spiel verwenden"));
children.push(pmix([
  ["Im Spiel lädst du dein PNG genau wie in Kapitel 2 mit ", false],
  ["LOADIMAGE", true], [" und zeichnest es mit ", false], ["DRAWIMAGE", true],
  [". Zeichnest du das Schiff um, ändert sich sofort dein Spiel – ohne eine Zeile Code anzufassen.", false],
]));

children.push(h2("Was du gelernt hast"));
children.push(bulletRich("Sprite ", "= kleines Bild aus einzelnen Pixeln (hier 16 × 16)."));
children.push(bulletRich("dhsprites ", "= der Pixel-Editor mit Stift, Füller, Pipette und Symmetrie."));
children.push(bulletRich("Frames ", "= mehrere Einzelbilder ergeben eine Animation (Onion-Skin hilft)."));
children.push(bulletRich("Export ", "= .dhsprite zum Weiterbearbeiten, PNG/Sheet fürs Spiel."));

children.push(h2("Übung"));
children.push(bullet("Öffne ein vorhandenes Sprite (z. B. player.png) in dhsprites und färbe es um."));
children.push(bullet("Zeichne deinen eigenen Gegner mit zwei Frames."));
children.push(bullet("Exportiere ihn als PNG und tausche ihn testweise im Spiel ein."));

// ===================== Kapitel 4 =====================
children.push(chapter("Kapitel 4: Schiessen"));
children.push(tip("In diesem Kapitel",
  "Dein Schiff bekommt Feuerkraft. Du lernst einen „Pool“ kennen – einen festen Vorrat an Schüssen, den wir clever wiederverwenden – und feuerst genau einen Schuss pro Tastendruck."));

children.push(h2("Viele Schüsse verwalten: der Pool"));
children.push(p("Ein Raumschiff, das nicht schießen kann, ist in einem Shooter ungefähr so nützlich wie ein Regenschirm aus Papier. Höchste Zeit, das zu ändern. Aber halt – bevor wir wild Schüsse erzeugen, lohnt sich ein kurzer Gedanke darüber, wie wir sie verwalten."));
children.push(p("Ein Schuss hat eine Position, und er ist in genau einem von zwei Zuständen: Entweder er fliegt gerade nach oben, oder er fliegt nicht. Eine naheliegende Idee wäre, bei jedem Tastendruck einen neuen Schuss zu erzeugen und ihn wegzuwerfen, sobald er oben aus dem Bild fliegt. Das funktioniert, aber das ständige Erzeugen und Wegwerfen kostet auf Dauer Kraft – ein bisschen so, als würde man für jedes Glas Wasser einen neuen Becher kaufen und den alten wegschmeißen."));
children.push(p("Profis machen das anders, und du machst es ab jetzt auch: Wir legen einen festen Vorrat an wiederverwendbaren Schüssen an, einen sogenannten Pool. Stell dir einen Köcher mit fünf Pfeilen vor – ist ein Pfeil verschossen und gelandet, wandert er zurück in den Köcher und kann erneut benutzt werden. Drei Listen beschreiben alle fünf Schüsse zugleich:"));
children.push(codeBlock([
  "DIM bulletImg AS IMAGE",
  'bulletImg = LOADIMAGE("../../assets/sprites/bullet.png")',
  "CONST NBULLET AS INTEGER = 5",
  "DIM bx[NBULLET] AS INTEGER          ' x-Position",
  "DIM by[NBULLET] AS INTEGER          ' y-Position",
  "DIM bAlive[NBULLET] AS BOOLEAN      ' fliegt dieser Schuss?",
  "FOR i = 0 TO NBULLET - 1 : bAlive[i] = FALSE : NEXT i",
]));
children.push(pmix([
  ["", false],
  ["bAlive", true], [" ist vom Typ ", false], ["BOOLEAN", true],
  [" – das ist ein Ja/Nein-Wert (", false], ["TRUE", true], [" oder ", false], ["FALSE", true],
  ["). So wissen wir für jeden der 5 Plätze, ob dort gerade ein Schuss unterwegs ist. Am Anfang fliegt keiner.", false],
]));
children.push(why("Warum legen wir den Vorrat einmal fest, statt bei jedem Schuss einen neuen zu erschaffen und ihn danach wegzuwerfen? Zwei Gründe. Erstens kostet ständiges Erzeugen und Wegwerfen Rechenzeit und kann ausgerechnet im hitzigsten Gefecht zu winzigen Rucklern führen – dann also, wenn du sie am wenigsten gebrauchen kannst. Zweitens ist ein fester Vorrat vorhersehbar: Mehr als fünf Schüsse gleichzeitig kann es nie geben, nichts gerät außer Kontrolle. Profis nennen dieses Muster „Object Pooling“, und du findest es in praktisch jeder Spiel-Engine wieder."));

children.push(h2("Feuern auf Tastendruck"));
children.push(p("Beim Feuern suchen wir einen freien Platz im Köcher und starten dort einen Schuss. Klingt simpel, hat aber einen Haken, über den fast jeder einmal stolpert: Die Spielschleife läuft sechzig Mal pro Sekunde. Würden wir einfach „wenn Leertaste gedrückt, dann schieße“ schreiben, dann feuerte das Schiff bei gehaltener Taste sechzig Schüsse pro Sekunde – ein Wasserfall aus Geschossen, und der Köcher wäre im selben Wimpernschlag leer."));
children.push(p("Was wir wollen, ist ein Schuss pro Tastendruck. Dafür müssen wir den Moment erwischen, in dem die Taste gerade eben heruntergeht – nicht die ganze Zeit, in der sie unten bleibt. Diesen Moment nennt man die FLANKE eines Tastendrucks, und GameBasic hat dafür einen eigenen Befehl:"));
children.push(codeBlock([
  "IF KEYHIT(KEY_SPACE) THEN",
  "    DIM s AS INTEGER",
  "    FOR s = 0 TO NBULLET - 1",
  "        IF NOT bAlive[s] THEN",
  "            bAlive[s] = TRUE : bx[s] = shipX + 5 : by[s] = shipY - 4",
  "            BREAK",
  "        END IF",
  "    NEXT s",
  "END IF",
]));
children.push(pmix([
  ["", false],
  ["KEYHIT", true],
  [" ist nur in dem einen Bild wahr, in dem die Taste heruntergeht – gehalten feuert sie also nicht weiter. Merke dir das Paar: ", false],
  ["KEYPRESSED", true], [" fragt „wird gehalten?“ (richtig fürs Bewegen), ", false],
  ["KEYHIT", true], [" fragt „gerade gedrückt?“ (richtig für Aktionen). Für die Maus heißen sie ", false],
  ["MOUSEBUTTON", true], [" und ", false], ["MOUSE_HIT", true],
  [", fürs Gamepad ", false], ["JOYSTICK_BUTTON", true], [" und ", false], ["JOYSTICK_HIT", true], [".", false],
]));
children.push(pmix([
  ["Die ", false], ["FOR", true], ["-Schleife sucht den ersten freien Platz im Köcher; ", false],
  ["BREAK", true], [" verlässt sie sofort, sobald einer gefunden ist. Der Schuss startet knapp über dem Schiff.", false],
]));
children.push(why("Wie käme man ohne KEYHIT aus? Man müsste sich von einem Frame zum nächsten selbst merken, ob die Taste schon vorher unten war, und nur dann feuern, wenn sie JETZT unten ist und vorher nicht: eine zusätzliche Variable, zwei zusätzliche Zeilen – und das für jede Taste, die eine Aktion auslöst. Genau so stand es in der ersten Fassung dieses Buchs. Es lohnt sich trotzdem, den Gedanken zu kennen: Die Unterscheidung zwischen „gehalten“ und „gerade gedrückt“ musst du in jedem Spiel treffen, egal in welcher Sprache. GameBasic nimmt dir nur die Buchführung ab."));

children.push(h2("Schüsse bewegen und zeichnen"));
children.push(p("Ein gestarteter Schuss soll natürlich nicht in der Luft hängen bleiben wie ein vergessener Luftballon. Jeden Frame lassen wir deshalb alle gerade fliegenden Schüsse ein Stück nach oben wandern. Und sobald einer oben aus dem Bild verschwindet, geben wir seinen Platz im Köcher wieder frei – damit er für den nächsten Schuss bereitsteht. Genau das macht den Pool so sparsam:"));
children.push(codeBlock([
  "FOR i = 0 TO NBULLET - 1",
  "    IF bAlive[i] THEN",
  "        by[i] = by[i] - 8",
  "        IF by[i] < -8 THEN bAlive[i] = FALSE",
  "        DRAWIMAGE(bulletImg, bx[i], by[i])",
  "    END IF",
  "NEXT i",
]));
children.push(pmix([
  ["Kleiner ", false], ["by[i] - 8", true],
  [" bedeutet weiter oben (in der Grafik wächst y nach unten). Sobald ", false],
  ["by[i] < -8", true], [" ist der Schuss oben raus und sein Platz im Pool steht dem nächsten Schuss zur Verfügung.", false],
]));

children.push(h2("Das ganze Programm"));
children.push(codeBlock([
  'SCREEN(480, 640, "Mein Galaga - Kapitel 4")',
  "CONST NSTARS AS INTEGER = 60",
  "DIM starX[NSTARS] AS INTEGER : DIM starY[NSTARS] AS INTEGER",
  "DIM i AS INTEGER",
  "FOR i = 0 TO NSTARS - 1",
  "    starX[i] = RANDINT(0, 479) : starY[i] = RANDINT(0, 639)",
  "NEXT i",
  "",
  "DIM shipImg AS IMAGE",
  'shipImg = LOADIMAGE("../../assets/sprites/player.png")',
  "DIM shipX AS INTEGER : DIM shipY AS INTEGER",
  "shipX = 232 : shipY = 560",
  "",
  "DIM bulletImg AS IMAGE",
  'bulletImg = LOADIMAGE("../../assets/sprites/bullet.png")',
  "CONST NBULLET AS INTEGER = 5",
  "DIM bx[NBULLET] AS INTEGER : DIM by[NBULLET] AS INTEGER",
  "DIM bAlive[NBULLET] AS BOOLEAN",
  "FOR i = 0 TO NBULLET - 1 : bAlive[i] = FALSE : NEXT i",
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
  "    IF KEYHIT(KEY_SPACE) THEN",
  "        DIM s AS INTEGER",
  "        FOR s = 0 TO NBULLET - 1",
  "            IF NOT bAlive[s] THEN",
  "                bAlive[s] = TRUE : bx[s] = shipX + 5 : by[s] = shipY - 4 : BREAK",
  "            END IF",
  "        NEXT s",
  "    END IF",
  "    FOR i = 0 TO NBULLET - 1",
  "        IF bAlive[i] THEN",
  "            by[i] = by[i] - 8",
  "            IF by[i] < -8 THEN bAlive[i] = FALSE",
  "            DRAWIMAGE(bulletImg, bx[i], by[i])",
  "        END IF",
  "    NEXT i",
  "    DRAWIMAGE(shipImg, shipX, shipY)",
  "    FLIP()",
  "WEND",
]));
figure("kap04_schiessen.png", "Feuer frei: Die Schüsse steigen in einer Reihe nach oben.", 300, 400).forEach(e => children.push(e));

children.push(h2("Was du gelernt hast"));
children.push(bulletRich("Pool ", "= fester Vorrat (hier 5 Schüsse), der wiederverwendet wird – mit bAlive merken wir, welche Plätze belegt sind."));
children.push(bulletRich("BOOLEAN ", "speichert Ja/Nein (TRUE/FALSE)."));
children.push(bulletRich("KEYHIT ", "= die Flanke eines Tastendrucks: ein Schuss pro Druck, statt sechzig pro Sekunde. KEYPRESSED bleibt fürs Bewegen."));
children.push(bulletRich("BREAK ", "verlässt eine Schleife vorzeitig."));

children.push(h2("Übung"));
children.push(bullet("Erlaube mehr Schüsse gleichzeitig (NBULLET erhöhen)."));
children.push(bullet("Mach die Schüsse schneller oder langsamer (die 8)."));
children.push(bullet("Tausche KEYHIT gegen KEYPRESSED (feuere bei gehaltener Taste) – wie fühlt sich das an?"));

// ===================== Kapitel 5 =====================
children.push(chapter("Kapitel 5: Sternenhimmel mit Parallax"));
children.push(tip("In diesem Kapitel",
  "Aus dem schlichten Sternenhimmel von Kapitel 1 wird ein echter Weltraum: Sterne in mehreren Tiefen-Ebenen, die unterschiedlich schnell scrollen und sanft funkeln."));

children.push(h2("Tiefe durch Tempo (Parallax)"));
children.push(p("Unser Sternenhimmel aus Kapitel 1 tut brav seinen Dienst, aber ehrlich gesagt sieht er ein bisschen aus wie weiße Konfetti, die jemand gleichmäßig nach unten rieseln lässt. Echtes Weltall hat Tiefe – manche Sterne stehen gefühlt zum Greifen nah, andere unendlich weit weg. Diese Tiefe bekommen wir mit einem alten, billigen und wunderbar wirkungsvollen Trick."));
children.push(p("Schau einmal aus dem Fenster eines fahrenden Autos: Der Leitpfosten direkt am Straßenrand saust vorbei, dass dir schwindlig wird, während die Berge am Horizont sich kaum von der Stelle rühren. Genau dieser Effekt heißt Parallax – und er ist der ganze Zaubertrick. Dinge, die schneller vorbeiziehen, nimmt unser Gehirn als näher wahr. Wir teilen unsere Sterne deshalb in drei Ebenen: fern (dunkel, langsam), mittel und nah (hell, schnell). Flache Punkte, plötzlich räumlich."));
children.push(pmix([
  ["Für sanfte, unterschiedliche Geschwindigkeiten brauchen wir Kommazahlen – den Typ ", false],
  ["FLOAT", true], [". Jeder Stern bekommt Tempo, Helligkeit und eine zufällige „Funkel-Phase“:", false],
]));
children.push(codeBlock([
  "CONST NSTARS AS INTEGER = 80",
  "DIM starX[NSTARS]   AS FLOAT",
  "DIM starY[NSTARS]   AS FLOAT",
  "DIM starSpd[NSTARS] AS FLOAT        ' Tempo = Tiefe",
  "DIM starBri[NSTARS] AS INTEGER      ' Grund-Helligkeit",
  "DIM starPh[NSTARS]  AS FLOAT        ' Phase fuers Funkeln",
  "DIM i AS INTEGER",
  "FOR i = 0 TO NSTARS - 1",
  "    starX[i] = RANDF(0.0, 479.0)",
  "    starY[i] = RANDF(0.0, 639.0)",
  "    starPh[i] = RANDF(0.0, 6.28)",
  "    DIM layer AS INTEGER : layer = i MOD 3",
  "    IF layer = 0 THEN starSpd[i] = 0.5 : starBri[i] = 80",
  "    IF layer = 1 THEN starSpd[i] = 1.1 : starBri[i] = 150",
  "    IF layer = 2 THEN starSpd[i] = 2.0 : starBri[i] = 240",
  "NEXT i",
]));
children.push(pmix([
  ["", false],
  ["RANDF(min, max)", true], [" liefert eine zufällige Kommazahl (anders als ", false],
  ["RANDINT", true], [", das ganze Zahlen gibt). ", false],
  ["i MOD 3", true], [" ist der Rest beim Teilen durch 3 – ergibt reihum 0, 1, 2 und verteilt die Sterne auf die drei Ebenen.", false],
]));

children.push(h2("Funkeln mit SIN"));
children.push(p("Funkeln heisst: die Helligkeit eines Sterns schwankt sanft auf und ab. Eine weiche Wellenbewegung liefert der Sinus, SIN. Über die Zeit (unseren Frame-Zähler) plus die eigene Phase jedes Sterns funkelt jeder ein bisschen anders:"));
children.push(codeBlock([
  "FOR i = 0 TO NSTARS - 1",
  "    starY[i] = starY[i] + starSpd[i]",
  "    IF starY[i] > 639.0 THEN starY[i] = 0.0 : starX[i] = RANDF(0.0, 479.0)",
  "    DIM tw AS FLOAT : tw = 0.55 + 0.45 * SIN(frame * 0.08 + starPh[i])",
  "    DIM v AS INTEGER : v = INT(starBri[i] * tw)",
  "    PLOT(INT(starX[i]), INT(starY[i]), RGB(v, v, v))",
  "NEXT i",
]));
children.push(pmix([
  ["", false],
  ["SIN(...)", true], [" pendelt weich zwischen -1 und 1. Mit ", false],
  ["0.55 + 0.45 * SIN(...)", true], [" wird daraus ein Faktor zwischen 0,1 und 1,0 für die Helligkeit. ", false],
  ["INT(...)", true], [" macht aus der Kommazahl wieder eine ganze Zahl (Farben und Pixel brauchen ganze Werte), und ", false],
  ["RGB(v, v, v)", true], [" mischt aus gleichen Rot/Grün/Blau-Anteilen einen Grauton – mal heller, mal dunkler.", false],
]));
children.push(tip("Frame-Zähler nicht vergessen",
  "Damit sich das Funkeln über die Zeit bewegt, brauchst du eine Variable frame, die in der Schleife jedes Bild um 1 hochzählt (DIM frame AS INTEGER vor der Schleife, frame = frame + 1 als erste Zeile darin)."));

children.push(h2("Einbauen"));
children.push(pmix([
  ["Ersetze den einfachen Sternen-Teil aus Kapitel 1 durch diese beiden Blöcke. Schiff und Schüsse aus Kapitel 4 bleiben unverändert. Das vollständige Programm liegt fertig in ", false],
  ["code/kap05/sternenhimmel.dh", true], [".", false],
]));
figure("kap05_sternenhimmel.png", "Ein tieferer Weltraum: Sterne in mehreren Helligkeiten, die funkeln und scrollen.", 300, 400).forEach(e => children.push(e));

children.push(h2("Was du gelernt hast"));
children.push(bulletRich("FLOAT ", "speichert Kommazahlen – nötig für feine Geschwindigkeiten."));
children.push(bulletRich("Parallax ", "= verschiedene Ebenen scrollen verschieden schnell → Raumgefühl."));
children.push(bulletRich("SIN ", "liefert eine weiche Welle – ideal fürs Funkeln (mit eigener Phase pro Stern)."));
children.push(bulletRich("INT und RGB ", "wandeln Kommazahlen in ganze Werte und mischen Farben."));

children.push(h2("Übung"));
children.push(bullet("Füge eine vierte, sehr helle und schnelle Ebene hinzu."));
children.push(bullet("Gib den Sternen einen leichten Blaustich (etwas weniger Rot/Grün als Blau)."));
children.push(bullet("Ändere die Funkel-Geschwindigkeit (die 0.08)."));

// ===================== Kapitel 6 =====================
children.push(chapter("Kapitel 6: Gegner & Formation"));
children.push(tip("In diesem Kapitel",
  "Jetzt kommt Leben ins Spiel: eine ganze Formation bunter Gegner. Dafür lernst du Klassen kennen – einen Bauplan für gleichartige Dinge – und ordnest 24 Gegner in einem Gitter an, das sanft schwebt."));

children.push(p("Bisher war unser Spiel eine recht einsame Angelegenheit: ein Schiff, ein paar Schüsse, viel leerer Raum. Das ändert sich jetzt schlagartig. Wir holen die Gegner ins Spiel – und nicht nur einen, sondern gleich vierundzwanzig auf einmal. Damit das nicht im Chaos endet, lernst du in diesem Kapitel ein Werkzeug kennen, das in der Programmierung allgegenwärtig ist: die Klasse."));
children.push(h2("Eine Klasse für die Gegner"));
children.push(p("Schauen wir uns einen einzelnen Gegner an. Was muss er über sich wissen? Wo er steht (seine Position), in welcher Reihe er sitzt (das bestimmt seine Farbe) und ob er überhaupt noch lebt. Bei einem Gegner wären das drei, vier Variablen – kein Problem. Aber bei vierundzwanzig Gegnern bräuchten wir fast hundert einzeln benannte Variablen. Spätestens bei shipPositionVonGegnerNummerSiebzehn würde uns die Lust vergehen."));
children.push(p("Viel klüger ist es, einmal den Bauplan eines Gegners zu beschreiben und GameBasic dann zu sagen: „Mach mir bitte vierundzwanzig davon.“ Genau das ist eine Klasse – ein Bauplan für gleichartige Dinge:"));
children.push(codeBlock([
  "CLASS Bug",
  "    DIM x AS INTEGER",
  "    DIM y AS INTEGER",
  "    DIM row AS INTEGER          ' Reihe 0/1/2 -> Farbe",
  "    DIM alive AS BOOLEAN",
  "",
  "    SUB Init(px AS INTEGER, py AS INTEGER, prow AS INTEGER)",
  "        Self.x = px : Self.y = py : Self.row = prow : Self.alive = TRUE",
  "    END SUB",
  "END CLASS",
]));
children.push(pmix([
  ["Eine ", false], ["CLASS", true],
  [" ist ein Bauplan; ein konkreter Gegner daraus heisst Objekt. Die Felder (", false],
  ["x", true], [", ", false], ["y", true], [", …) gehören jedem Objekt einzeln. ", false],
  ["Init", true], [" ist eine besondere Methode: Sie läuft automatisch, wenn wir mit ", false],
  ["NEW", true], [" einen Gegner erzeugen, und füllt die Felder. ", false],
  ["Self", true], [" meint dabei „dieses eine Objekt“.", false],
]));

children.push(h2("Die Formation aufbauen"));
children.push(p("Vierundzwanzig Gegner ordnen wir nicht wild durcheinander an, sondern ordentlich im Raster: 3 Reihen mal 8 Spalten. Das sieht nicht nur militärisch-bedrohlich aus, es macht uns auch das Leben leichter. Wir legen ein Array von Bug-Objekten an und erzeugen die Gegner in einer einzigen Schleife."));
children.push(p("Jetzt kommt ein kleiner Rechentrick, der dir noch oft begegnen wird. Die Schleife zählt schlicht von 0 bis 23 hoch – eine einzige Laufnummer. Daraus müssen wir für jeden Gegner Spalte und Reihe herausfischen. Zwei alte Bekannte aus dem Mathematikunterricht helfen dabei: der Rest beim Teilen und die Ganzzahl-Division."));
children.push(codeBlock([
  "CONST COLS AS INTEGER = 8",
  "CONST ROWS AS INTEGER = 3",
  "CONST NBUGS AS INTEGER = 24",
  "DIM bugs[NBUGS] AS Bug",
  "DIM gapX AS INTEGER : gapX = 48",
  "DIM startX AS INTEGER : startX = (480 - (COLS - 1) * gapX) \\ 2 - 8",
  "FOR i = 0 TO NBUGS - 1",
  "    DIM col AS INTEGER : col = i MOD COLS",
  "    DIM rw AS INTEGER  : rw = i \\ COLS",
  "    bugs[i] = NEW Bug(startX + col * gapX, 60 + rw * 40, rw)",
  "NEXT i",
]));
children.push(pmix([
  ["", false],
  ["i MOD COLS", true], [" ist der Rest beim Teilen – die Spalte (0…7). ", false],
  ["i \\ COLS", true], [" ist die Ganzzahl-Division – die Reihe (0…2). ", false],
  ["NEW Bug(...)", true], [" erzeugt einen Gegner und ruft dessen ", false],
  ["Init", true], [" auf. So füllt sich das Gitter Zeile für Zeile.", false],
]));
children.push(why("Bei den Sternen und Schüssen kamen wir noch mit nebeneinander laufenden Listen aus – starX, starY und so weiter. Warum jetzt der Aufwand mit einer Klasse? Weil ein Gegner bald viel mehr können wird, als nur herumzuliegen: Er bekommt eine eigene Flugbahn, einen Zustand und eine Update-Logik. Sobald ein Ding nicht nur aus Daten besteht, sondern auch aus Verhalten, lohnt sich eine Klasse – sie hält die Daten und die Funktionen, die mit ihnen arbeiten, an einem Ort zusammen. Mit einem Dutzend paralleler Listen würdest du dagegen früher oder später den Überblick verlieren – und garantiert genau eine davon beim Umbauen vergessen."));

children.push(h2("Gegner zeichnen – mit Schweben und Flügelschlag"));
children.push(pmix([
  ["Jedes Gegner-Bild ist ein kleines Sheet mit zwei Frames (Flügel oben/unten). Mit ", false],
  ["DRAWIMAGEPART", true],
  [" schneiden wir genau ein 16×16-Stück heraus. Die ganze Formation schwebt per ", false],
  ["SIN", true], [" hin und her, und der Flügelschlag wechselt im Takt:", false],
]));
children.push(codeBlock([
  "DIM sway AS INTEGER : sway = INT(SIN(frame * 0.04) * 12.0)",
  "DIM srcX AS INTEGER : srcX = IIF((frame \\ 12) MOD 2 = 0, 0, 16)",
  "FOR i = 0 TO NBUGS - 1",
  "    IF bugs[i].alive THEN",
  "        DRAWIMAGEPART(rowImg[bugs[i].row], srcX, 0, 16, 16, bugs[i].x + sway, bugs[i].y)",
  "    END IF",
  "NEXT i",
]));
children.push(pmix([
  ["", false],
  ["DRAWIMAGEPART(bild, sx, sy, sw, sh, x, y)", true],
  [" zeichnet nur den Ausschnitt ab (sx, sy) mit Grösse sw×sh. Mit ", false],
  ["srcX", true], [" = 0 oder 16 wählen wir Frame 1 oder 2. ", false],
  ["bugs[i].alive", true], [" liest das Feld eines Objekts – nur lebende Gegner werden gezeichnet. ", false],
  ["rowImg", true], [" ist ein Array mit dem passenden Bild je Reihe (bug0/1/2.png).", false],
]));
children.push(pmix([
  ["Das vollständige Programm (mit Sternen, Schiff und Schüssen) liegt in ", false],
  ["code/kap06/gegner.dh", true], [".", false],
]));
figure("kap06_gegner.png", "Die Formation: 3 Reihen, 3 Farben, 24 Gegner – sie schweben und schlagen mit den Flügeln.", 300, 400).forEach(e => children.push(e));

children.push(h2("Was du gelernt hast"));
children.push(bulletRich("CLASS ", "= Bauplan; ein Objekt daraus hat eigene Felder. NEW erzeugt es, Init füllt es, Self meint das Objekt selbst."));
children.push(bulletRich("Array von Objekten ", "(DIM bugs[NBUGS] AS Bug) verwaltet viele Gegner auf einmal."));
children.push(bulletRich("MOD und \\ ", "rechnen aus einer Laufnummer Spalte und Reihe – praktisch für Gitter."));
children.push(bulletRich("DRAWIMAGEPART ", "zeichnet einen Bild-Ausschnitt – so nutzen wir einzelne Animations-Frames."));

children.push(h2("Übung"));
children.push(bullet("Ändere die Formationsgrösse (COLS/ROWS) und die Abstände (gapX)."));
children.push(bullet("Lass die Formation stärker oder schneller schweben (die 12 bzw. 0.04)."));
children.push(bullet("Verlangsame den Flügelschlag (die 12 in der Frame-Wahl)."));

// ===================== Kapitel 7 =====================
children.push(chapter("Kapitel 7: Einflug-Manöver"));
children.push(tip("In diesem Kapitel",
  "Die Gegner stehen nicht mehr einfach da – sie fliegen in eleganten, geschwungenen Bögen ein und sammeln sich dann zur Formation. Das ist der Moment, in dem es sich richtig nach Galaga anfühlt."));

children.push(h2("Geschwungene Bahnen mit Bézier-Kurven"));
children.push(p("Jetzt kommt der Moment, auf den dieses Buch insgeheim die ganze Zeit hingearbeitet hat. Eine Formation, die einfach von oben hereinschwebt, ist nett – aber das unverwechselbare Galaga-Gefühl entsteht erst durch die eleganten, geschwungenen Bahnen, in denen die Gegner hereingleiten, als hätten sie an einer Ballettschule für Außerirdische studiert."));
children.push(p("Eine gerade Linie kriegt jeder hin – aber sie ist eben auch sterbenslangweilig. Schöne, weiche Kurven sind kniffliger. Zum Glück hat sich darüber schon jemand den Kopf zerbrochen: der französische Ingenieur Pierre Bézier, der in den sechziger Jahren bei Renault Karosserien entwarf und eine Methode brauchte, um geschwungene Bleche mathematisch zu beschreiben. Heraus kam die Bézier-Kurve, und sie steckt heute in fast jedem Schriftzeichen, jedem Logo und – ab jetzt – in unseren Gegnern."));
children.push(p("Das Prinzip ist überraschend anschaulich: Aus vier Punkten – einem Start, einem Ziel und zwei „Zieh-Punkten“ dazwischen, die die Kurve in ihre Form locken – entsteht eine weiche Bahn. Man muss die Mathematik dahinter nicht verstehen, denn GameBasic liefert sie fix und fertig im curves-Modul:"));
children.push(codeBlock([
  'IMPORT "curves"',
]));
children.push(pmix([
  ["", false],
  ["IMPORT", true],
  [" holt ein Zusatz-Modul dazu. ", false],
  ["CURVE_BEZIER2(t, x0,y0, x1,y1, x2,y2, x3,y3)", true],
  [" berechnet einen Punkt auf der Kurve. ", false],
  ["t", true], [" läuft dabei von 0 (Start) bis 1 (Ziel) – bei 0,5 ist man in der Mitte der Bahn.", false],
]));

children.push(h2("Tupel: zwei Werte auf einmal"));
children.push(p("Ein Punkt hat zwei Zahlen: x und y. CURVE_BEZIER2 gibt deshalb gleich beide zurück – als Tupel. Mit einer Klammer-Zuweisung holen wir sie in zwei Variablen:"));
children.push(codeBlock([
  "DIM bx AS FLOAT : DIM by AS FLOAT",
  "(bx, by) = CURVE_BEZIER2(t, cx0, cy0, cx1, cy1, cx2, cy2, cx3, cy3)",
]));
children.push(pmix([
  ["Die linke Seite ", false], ["(bx, by)", true],
  [" verteilt die beiden Rückgabewerte auf zwei Variablen – praktisch, wenn etwas mehr als einen Wert liefert.", false],
]));

children.push(h2("Der Gegner bekommt eine Flugbahn"));
children.push(p("Wir erweitern die Bug-Klasse um die vier Kontrollpunkte, einen Fortschritt t mit Tempo, einen Startverzögerung delay (damit nicht alle gleichzeitig losfliegen) und ein Flag inForm. In Init legen wir die Bahn fest – von ausserhalb des Bildes in einem Bogen zur Heimatposition:"));
children.push(codeBlock([
  "SUB Init(phx AS FLOAT, phy AS FLOAT, prow AS INTEGER, pdelay AS INTEGER)",
  "    Self.hx = phx : Self.hy = phy : Self.row = prow",
  "    Self.alive = TRUE : Self.inForm = FALSE",
  "    Self.t = 0.0 : Self.ts = 0.012 : Self.delay = pdelay",
  "    DIM side AS INTEGER : side = IIF(prow MOD 2 = 0, -1, 1)",
  "    Self.cx0 = phx - side * 180.0 : Self.cy0 = -30.0",
  "    Self.cx1 = 240.0 + side * 140.0 : Self.cy1 = 120.0",
  "    Self.cx2 = phx + side * 80.0 : Self.cy2 = 40.0",
  "    Self.cx3 = phx : Self.cy3 = phy",
  "    Self.x = Self.cx0 : Self.y = Self.cy0",
  "END SUB",
]));
children.push(pmix([
  ["", false],
  ["side", true], [" ist je nach Reihe -1 oder 1 – so kommen die Gegner abwechselnd von links und rechts. Start (cx0/cy0) liegt oben ausserhalb des Bildes, Ziel (cx3/cy3) ist die Formationsposition.", false],
]));
children.push(why("Warum speichert jeder Gegner seine vier Kontrollpunkte fest ab, statt die Bahn bei Bedarf neu auszurechnen? Weil sich die Bahn während des Einflugs gar nicht ändert – einmal festgelegt, wird sie nur noch abgefahren. Das Einzige, was sich bewegt, ist der Fortschritt t. Und warum die gestaffelte Startverzögerung? Würden alle vierundzwanzig Gegner im selben Augenblick losfliegen, sähe das aus wie ein Vogelschwarm, der geschlossen gegen eine Fensterscheibe klatscht. Erst die kleinen Verzögerungen erzeugen den eleganten, tröpfelnden Strom, den man aus dem Original kennt."));

children.push(h2("Bewegen entlang der Kurve"));
children.push(p("Die Update-Methode treibt den Gegner über die Bahn. Ist er angekommen (t ≥ 1), schwebt er ab da nur noch in der Formation:"));
children.push(codeBlock([
  "SUB Update(sway AS FLOAT)",
  "    IF Self.inForm THEN",
  "        Self.x = Self.hx + sway : Self.y = Self.hy : RETURN",
  "    END IF",
  "    IF Self.delay > 0 THEN Self.delay = Self.delay - 1 : RETURN",
  "    Self.t = Self.t + Self.ts",
  "    DIM bx AS FLOAT : DIM by AS FLOAT",
  "    (bx, by) = CURVE_BEZIER2(MIN(Self.t, 1.0), Self.cx0, Self.cy0, _",
  "        Self.cx1, Self.cy1, Self.cx2, Self.cy2, Self.cx3, Self.cy3)",
  "    Self.x = bx : Self.y = by",
  "    IF Self.t >= 1.0 THEN Self.inForm = TRUE",
  "END SUB",
]));
children.push(pmix([
  ["", false],
  ["RETURN", true], [" in einer SUB bricht die Methode sofort ab. Der ", false],
  ["delay", true], [" zählt zuerst herunter (gestaffelter Start). ", false],
  ["MIN(Self.t, 1.0)", true], [" sorgt dafür, dass t nicht über 1 hinausläuft. Das Unterstrich-Zeichen ", false],
  ["_", true], [" am Zeilenende setzt eine lange Zeile unten fort.", false],
]));
children.push(pmix([
  ["In der Spielschleife rufst du pro Gegner ", false], ["bugs[i].Update(sway)", true],
  [" auf und zeichnest ihn an ", false], ["INT(bugs[i].x), INT(bugs[i].y)", true],
  [". Das vollständige Programm liegt in ", false], ["code/kap07/einflug.dh", true], [".", false],
]));
figure("kap07_einflug.png", "Einflug: Die obere Reihe steht schon, die nächste strömt im Bogen herein.", 300, 400).forEach(e => children.push(e));

children.push(h2("Was du gelernt hast"));
children.push(bulletRich("IMPORT ", "holt Zusatz-Module dazu (hier curves)."));
children.push(bulletRich("Bézier-Kurve ", "= weiche Bahn aus 4 Punkten; CURVE_BEZIER2(t, …) liefert den Punkt bei t (0…1)."));
children.push(bulletRich("Tupel ", "= mehrere Werte auf einmal: (bx, by) = ..."));
children.push(bulletRich("Methode mit Zustand ", "(inForm, delay, t): jeder Gegner steuert seinen eigenen Einflug."));

children.push(h2("Übung"));
children.push(bullet("Mach den Einflug schneller oder langsamer (ts in Init)."));
children.push(bullet("Verändere die Kontrollpunkte – wie ändert sich die Bahnform?"));
children.push(bullet("Lass die Gegner enger gestaffelt starten (kleinerer delay-Faktor als i * 4)."));

// ===================== Kapitel 8 =====================
children.push(chapter("Kapitel 8: Sturzangriffe"));
children.push(tip("In diesem Kapitel",
  "Die Gegner werden gefährlich: Einzelne lösen sich aus der Formation und stürzen im Bogen auf dich herab. Dafür bekommt jeder Gegner eine kleine Zustands-Maschine – gesteuert über ein ENUM."));

children.push(h2("Zustände mit ENUM"));
children.push(p("Bisher waren unsere Gegner brave Zeitgenossen: einfliegen, sich einreihen, schweben. Charmant, aber als Gegner taugen sie damit ungefähr so viel wie ein Wachhund, der jeden Einbrecher freudig begrüßt. Das ändern wir jetzt. Einzelne Gegner sollen sich aus der Formation lösen und im Sturzflug auf dich herabstoßen – und plötzlich wird aus dem hübschen Bildschirmschoner ein Spiel, bei dem man besser aufpasst."));
children.push(p("Damit ein Gegner zwischen „einfliegen“, „schweben“ und „stürzen“ unterscheiden kann, muss er sich merken, was er gerade tut. Er ist immer in genau einem von drei Zuständen: Er fliegt ein (ENTER), schwebt in der Formation (FORM) oder stürzt gerade (DIVE). Man könnte das mit Zahlen lösen – 0 für einfliegen, 1 für schweben, 2 für stürzen – aber dann sitzt man drei Wochen später vor seinem eigenen Code und rätselt, was zum Henker „state = 2“ wohl bedeuten sollte."));
children.push(p("Viel freundlicher ist es, den Zuständen richtige Namen zu geben. Genau dafür gibt es das ENUM – eine kurze Liste benannter Werte:"));
children.push(codeBlock([
  "ENUM St = ENTER, FORM, DIVE",
]));
children.push(pmix([
  ["Ein ", false], ["ENUM", true],
  [" ist eine Liste benannter Werte. Statt einer nichtssagenden 1 schreiben wir ", false],
  ["St.FORM", true],
  [" – das liest sich wie ein Satz. In der Bug-Klasse ersetzt ein Feld ", false],
  ["state AS INTEGER", true], [" das frühere Ja/Nein-Flag.", false],
]));
children.push(tip("Namen klug wählen",
  "Weil GameBasic Gross-/Kleinschreibung ignoriert, darf der ENUM-Name nicht gleich heissen wie eine deiner Variablen. Wir nennen ihn kurz St – das kollidiert mit nichts."));

children.push(h2("Eine Bahn für den Sturz"));
children.push(pmix([
  ["Den Einflug- und den Sturz-Bogen setzen wir mit derselben Hilfsmethode ", false],
  ["SetPath", true],
  [" (Start, zwei Kontrollpunkte, Ziel + Tempo). Die Dive-Methode legt eine Bahn von der Formation hinunter an der Spielerspalte vorbei und unten aus dem Bild:", false],
]));
children.push(codeBlock([
  "SUB Dive(px AS FLOAT, speed AS FLOAT)",
  "    DIM side AS INTEGER : side = IIF(Self.hx < 240.0, 1, -1)",
  "    Self.SetPath(speed, Self.hx, Self.hy, Self.hx + side * 100.0, Self.hy + 160.0, _",
  "        px, 460.0, px - side * 100.0, 700.0)",
  "    Self.state = St.DIVE",
  "END SUB",
]));

children.push(h2("Die Zustands-Maschine"));
children.push(p("Die Update-Methode reagiert auf den Zustand. Kommt ein Gegner am Ende seiner Bahn an (t ≥ 1), entscheidet sein Zustand, wie es weitergeht: Nach einem Sturz fliegt er von oben wieder in die Formation, sonst hat er die Formation erreicht:"));
children.push(codeBlock([
  "IF Self.t >= 1.0 THEN",
  "    IF Self.state = St.DIVE THEN",
  "        ' nach dem Sturz von oben zurueck in die Formation",
  "        Self.SetPath(0.02, Self.hx, -30.0, Self.hx, 60.0, Self.hx, 30.0, Self.hx, Self.hy)",
  "        Self.state = St.ENTER",
  "    ELSE",
  "        Self.state = St.FORM",
  "    END IF",
  "END IF",
]));
children.push(pmix([
  ["So entsteht ein Kreislauf: ", false],
  ["ENTER", true], [" → ", false], ["FORM", true], [" → (Sturz) ", false],
  ["DIVE", true], [" → ", false], ["ENTER", true], [" → … Genau das ist eine Zustands-Maschine.", false],
]));
children.push(why("Wir hätten die Zustände auch mit drei Ja/Nein-Flags lösen können: einfliegend, in Formation, stürzend. Klingt harmlos, ist aber eine Falle. Nichts würde verhindern, dass aus Versehen zwei davon gleichzeitig TRUE sind – und dann wäre ein Gegner laut Code im selben Moment am Einfliegen UND am Stürzen, was keinen Sinn ergibt und zu Fehlern führt, die man stundenlang sucht. Ein einzelnes state-Feld kann dagegen immer nur genau einen Wert haben. Es macht unmögliche Zustände schlicht unmöglich – und das ist eine der besten Eigenschaften, die Code überhaupt haben kann."));

children.push(h2("Wann stürzt wer?"));
children.push(p("Ein Zähler löst in Abständen einen Sturz aus. Wir suchen einen zufälligen Gegner, der gerade in Formation schwebt, und schicken ihn Richtung Schiff:"));
children.push(codeBlock([
  "diveTimer = diveTimer - 1",
  "IF diveTimer <= 0 THEN",
  "    diveTimer = 80 + RANDINT(0, 60)",
  "    DIM tries AS INTEGER",
  "    FOR tries = 0 TO 10",
  "        DIM pick AS INTEGER : pick = RANDINT(0, NBUGS - 1)",
  "        IF bugs[pick].alive AND bugs[pick].state = St.FORM THEN",
  "            bugs[pick].Dive(shipX, 0.013)",
  "            BREAK",
  "        END IF",
  "    NEXT tries",
  "END IF",
]));
children.push(pmix([
  ["Wir probieren bis zu 11-mal einen zufälligen Gegner; der erste passende (lebt und ist in ", false],
  ["FORM", true], [") stürzt los. ", false],
  ["shipX", true], [" übergibt die Spielerposition als Ziel. Das vollständige Programm liegt in ", false],
  ["code/kap08/sturz.dh", true], [".", false],
]));
figure("kap08_sturz.png", "Ein Gegner hat sich gelöst und stürzt im Bogen auf das Schiff herab.", 300, 400).forEach(e => children.push(e));

children.push(h2("Was du gelernt hast"));
children.push(bulletRich("ENUM ", "gibt Zahlen verständliche Namen (St.ENTER/FORM/DIVE)."));
children.push(bulletRich("Zustands-Maschine ", "= ein Objekt wechselt zwischen klaren Zuständen; die Update-Methode reagiert je nach Zustand."));
children.push(bulletRich("Wiederverwenden ", "= dieselbe SetPath-Methode für Einflug und Sturz."));
children.push(bulletRich("Zufalls-Auslöser ", "= ein Timer + RANDINT bestimmen, wann und wer stürzt."));

children.push(h2("Übung"));
children.push(bullet("Lass Gegner häufiger oder seltener stürzen (diveTimer-Werte)."));
children.push(bullet("Mach den Sturz schneller (die 0.013)."));
children.push(bullet("Zusatz: Lass beim Sturz-Tiefpunkt etwas passieren (Vorschau auf die Bomben im nächsten Kapitel)."));

// ===================== Kapitel 9 =====================
children.push(chapter("Kapitel 9: Bomben & Ausweichen"));
children.push(tip("In diesem Kapitel",
  "Jetzt wird zurückgeschossen: Stürzende Gegner werfen Bomben ab, denen du ausweichen musst. Du nutzt einen zweiten Pool – diesmal für fallende Bomben – und lernst, wie eine Methode mit FUNCTION ein Ergebnis zurückgibt."));

children.push(p("Bisher konnten die Gegner zwar bedrohlich auf dich herabstürzen, aber mehr auch nicht – im schlimmsten Fall rempeln sie dich an. In diesem Kapitel lernen sie eine fiese neue Fähigkeit: Sie werfen im Sturzflug Bomben ab, denen du ausweichen musst. Damit wird das Spiel zum ersten Mal richtig fordernd."));
children.push(h2("Update gibt Auskunft: FUNCTION"));
children.push(p("Bevor die Bomben fallen können, brauchen wir eine Antwort auf eine simple Frage: Wann genau soll ein Gegner abwerfen? Und diese Antwort muss von ihm zur Spielschleife wandern, denn der Gegner weiß, wann der richtige Moment ist – aber die Bomben werden außerhalb verwaltet."));
children.push(pmix([
  ["Hier kommt ein neues Werkzeug ins Spiel. Bisher war ", false], ["Update", true],
  [" eine SUB – sie tut etwas, gibt aber nichts zurück, wie ein fleißiger Helfer, der seine Arbeit erledigt und wortlos wieder geht. Eine ", false],
  ["FUNCTION", true],
  [" ist im Grunde dasselbe, nur dass sie am Ende eine Antwort mitbringt – mit dem Schlüsselwort ", false], ["RETURN", true],
  [". Wir machen aus Update also eine FUNCTION, die ", false], ["TRUE", true],
  [" zurückmeldet, wenn der Gegner gerade jetzt eine Bombe abwerfen soll, und sonst ", false], ["FALSE", true], [":", false],
]));
children.push(codeBlock([
  "FUNCTION Update(sway AS FLOAT) AS BOOLEAN",
  "    ' ... bewegen wie bisher ...",
  "    DIM drop AS BOOLEAN : drop = FALSE",
  "    IF Self.state = St.DIVE AND NOT Self.dropped AND Self.t >= Self.dropAt THEN",
  "        Self.dropped = TRUE : drop = TRUE",
  "    END IF",
  "    ' ... Zustandswechsel bei t >= 1 ...",
  "    RETURN drop",
  "END FUNCTION",
]));
children.push(pmix([
  ["Das Feld ", false], ["dropped", true],
  [" sorgt dafür, dass pro Sturz nur eine Bombe fällt: Einmal abgeworfen, bleibt es ", false],
  ["TRUE", true], [", bis der nächste Sturz beginnt.", false],
]));
children.push(why("Warum wirft der Gegner die Bombe nicht einfach selbst ab? Weil er gar nicht wissen muss – und soll –, wo der Bomben-Vorrat überhaupt liegt. Jeder kümmert sich um das, was er am besten weiß: Der Gegner kennt den richtigen Moment, die Spielschleife kennt die Bomben. Update meldet nur „jetzt!“ und überlässt das Wie dem Aufrufer. Diese saubere Aufgabenteilung – jeder Teil weiß möglichst wenig über die anderen – ist der Grund, warum großer Code überschaubar und änderbar bleibt, statt zu einem unentwirrbaren Knäuel zu werden."));

children.push(h2("Wann fällt eine Bombe?"));
children.push(pmix([
  ["Beim Start des Sturzes legen wir in ", false], ["Dive", true],
  [" einen zufälligen Zeitpunkt fest – etwa in der Bahnmitte:", false],
]));
children.push(codeBlock([
  "Self.dropAt = 0.45 + RANDF(0.0, 0.2)   ' irgendwo um die Mitte (t)",
  "Self.dropped = FALSE",
]));
children.push(pmix([
  ["Sobald ", false], ["Self.t", true], [" diesen ", false], ["dropAt", true],
  [" überschreitet, gibt Update einmalig ", false], ["TRUE", true], [" zurück.", false],
]));

children.push(h2("Der Bomben-Pool"));
children.push(p("Genau wie die Schüsse verwalten wir Bomben in einem Pool – nur dass sie nach unten fallen. Erst der Vorrat:"));
children.push(codeBlock([
  "DIM bombImg AS IMAGE",
  'bombImg = LOADIMAGE("../../assets/sprites/bomb.png")',
  "CONST NBOMB AS INTEGER = 8",
  "DIM bmx[NBOMB] AS FLOAT : DIM bmy[NBOMB] AS FLOAT",
  "DIM boAlive[NBOMB] AS BOOLEAN",
  "FOR i = 0 TO NBOMB - 1 : boAlive[i] = FALSE : NEXT i",
  "DIM bombSpeed AS FLOAT : bombSpeed = 3.0",
]));
children.push(p("Wenn Update TRUE meldet, suchen wir einen freien Bomben-Platz an der Gegnerposition:"));
children.push(codeBlock([
  "IF bugs[i].Update(sway) THEN",
  "    DIM b AS INTEGER",
  "    FOR b = 0 TO NBOMB - 1",
  "        IF NOT boAlive[b] THEN",
  "            boAlive[b] = TRUE : bmx[b] = bugs[i].x + 5.0 : bmy[b] = bugs[i].y + 8.0 : BREAK",
  "        END IF",
  "    NEXT b",
  "END IF",
]));
children.push(p("Und jeden Frame fallen die Bomben nach unten – wer unten herausfällt, gibt seinen Platz frei:"));
children.push(codeBlock([
  "FOR i = 0 TO NBOMB - 1",
  "    IF boAlive[i] THEN",
  "        bmy[i] = bmy[i] + bombSpeed",
  "        IF bmy[i] > 640.0 THEN boAlive[i] = FALSE",
  "        DRAWIMAGEPART(bombImg, srcXb, 0, 6, 6, INT(bmx[i]), INT(bmy[i]))",
  "    END IF",
  "NEXT i",
]));
children.push(pmix([
  ["", false],
  ["srcXb", true], [" wechselt wie beim Flügelschlag zwischen den zwei Bomben-Frames (Pulsieren). Das vollständige Programm liegt in ", false],
  ["code/kap09/bomben.dh", true], [".", false],
]));
figure("kap09_bomben.png", "Ein Gegner stürzt und hat eine Bombe abgeworfen – sie fällt nun nach unten.", 300, 400).forEach(e => children.push(e));

children.push(h2("Was du gelernt hast"));
children.push(bulletRich("FUNCTION ", "= wie SUB, gibt aber mit RETURN ein Ergebnis zurück (hier: „jetzt Bombe?“)."));
children.push(bulletRich("dropped-Flag ", "verhindert, dass pro Sturz mehrere Bomben fallen."));
children.push(bulletRich("Zweiter Pool ", "= Bomben funktionieren wie Schüsse, fallen aber nach unten."));
children.push(bulletRich("Timing über t ", "= der Abwurf hängt am Fortschritt der Sturzbahn (dropAt)."));

children.push(h2("Übung"));
children.push(bullet("Mach die Bomben schneller oder langsamer (bombSpeed)."));
children.push(bullet("Lass Gegner früher oder später abwerfen (dropAt)."));
children.push(bullet("Erhöhe NBOMB, damit mehr Bomben gleichzeitig fallen können."));

// ===================== Kapitel 10 =====================
children.push(chapter("Kapitel 10: Treffer & Punkte"));
children.push(tip("In diesem Kapitel",
  "Jetzt wird es ein echtes Spiel: Schüsse zerstören Gegner und geben Punkte, Bomben und Stürze kosten dich Leben – und bei null Leben heisst es Game Over (mit Neustart). Dafür schreibst du deine erste eigene Funktion."));

children.push(p("Wir haben jetzt fliegende Schüsse, einfliegende und stürzende Gegner, fallende Bomben – und bisher gleiten sie alle höflich aneinander vorbei, als wären sie auf einer sehr förmlichen Cocktailparty. Damit ist jetzt Schluss. In diesem Kapitel lernen die Dinge endlich, sich gegenseitig zu treffen. Das ist der Schritt, der aus einer hübschen Bewegung ein echtes Spiel macht."));
children.push(h2("Berühren sich zwei Rechtecke? Eine Funktion"));
children.push(p("Die Frage „berühren sich diese beiden Dinge?“ stellen wir im Spiel ununterbrochen: Trifft der Schuss den Gegner? Trifft die Bombe das Schiff? Rammt ein stürzender Gegner uns? Es wäre eine Qual, diese Prüfung jedes Mal aufs Neue auszuschreiben – und eine wunderbare Gelegenheit, Tippfehler einzubauen."));
children.push(p("Deshalb tun wir, was Programmierer immer tun, wenn sie etwas öfter als einmal brauchen: Wir geben dem wiederkehrenden Code einen Namen und packen ihn in eine eigene Funktion. Einmal sauber geschrieben, einmal getestet, danach für immer verlässlich. Unsere Funktion prüft, ob sich zwei Rechtecke überlappen. Fachleute nennen das einen AABB-Test (für „achsenausgerichtete Begrenzungsrechtecke“ – ein furchteinflößender Name für eine harmlose Idee):"));
children.push(codeBlock([
  "FUNCTION Overlap(ax AS FLOAT, ay AS FLOAT, aw AS FLOAT, ah AS FLOAT, _",
  "                 cx AS FLOAT, cy AS FLOAT, cw AS FLOAT, ch AS FLOAT) AS BOOLEAN",
  "    RETURN ax < cx + cw AND ax + aw > cx AND ay < cy + ch AND ay + ah > cy",
  "END FUNCTION",
]));
children.push(pmix([
  ["Zwei Rechtecke überlappen nur, wenn sie sich in beiden Richtungen überschneiden – das prüfen die vier mit ", false],
  ["AND", true],
  [" verknüpften Bedingungen. Jedes Rechteck wird durch Ecke (x, y) und Grösse (Breite, Höhe) beschrieben. Die Funktion gibt ", false],
  ["TRUE", true], [" oder ", false], ["FALSE", true], [" zurück.", false],
]));
children.push(why("Zwei Entscheidungen in diesem Kapitel lohnen einen zweiten Blick. Erstens: Warum die Überlappungs-Prüfung in eine eigene Funktion auslagern, statt sie an jeder der vielen Stellen frisch hinzuschreiben? Weil „einmal schreiben, überall benutzen“ heißt: ein möglicher Tippfehler statt fünf, eine Stelle zum Korrigieren statt fünf. Zweitens (gleich beim Spieler-Treffer zu sehen): Warum die kurze Unverwundbarkeit nach einem Treffer? Ohne sie könnte ein und dieselbe Bombe in mehreren aufeinanderfolgenden Frames immer wieder zählen und dir in einem Wimpernschlag alle Leben rauben – maximal unfair. invuln schenkt dir einen kurzen Moment zum Durchatmen."));

children.push(h2("Schuss trifft Gegner"));
children.push(p("Für jeden fliegenden Schuss prüfen wir jeden lebenden Gegner. Bei einer Überlappung verschwinden beide, und es gibt Punkte (obere Reihen zählen mehr):"));
children.push(codeBlock([
  "FOR i = 0 TO NBULLET - 1",
  "    IF bAlive[i] THEN",
  "        FOR j = 0 TO NBUGS - 1",
  "            IF bugs[j].alive AND Overlap(bx[i], by[i], 6, 8, bugs[j].x, bugs[j].y, 16, 16) THEN",
  "                bugs[j].alive = FALSE : bAlive[i] = FALSE",
  "                score = score + (ROWS - bugs[j].row) * 10",
  "                BREAK",
  "            END IF",
  "        NEXT j",
  "    END IF",
  "NEXT i",
]));

children.push(h2("Spieler getroffen"));
children.push(pmix([
  ["Genauso prüfen wir Bomben und stürzende Gegner gegen das Schiff. Ein Treffer kostet ein Leben und macht dich kurz unverwundbar (", false],
  ["invuln", true], ["), damit du nicht sofort mehrfach getroffen wirst:", false],
]));
children.push(codeBlock([
  "IF invuln = 0 THEN",
  "    DIM hit AS BOOLEAN : hit = FALSE",
  "    FOR i = 0 TO NBOMB - 1",
  "        IF boAlive[i] AND Overlap(bmx[i], bmy[i], 6, 6, shipX+3.0, shipY+2.0, 10, 12) THEN",
  "            boAlive[i] = FALSE : hit = TRUE",
  "        END IF",
  "    NEXT i",
  "    ' ... ebenso stuerzende Gegner pruefen ...",
  "    IF hit THEN lives = lives - 1 : invuln = 90",
  "END IF",
]));
children.push(pmix([
  ["Solange ", false], ["invuln", true],
  [" grösser 0 ist, lassen wir das Schiff blinken (zeichnen es nur in jedem zweiten Moment) – ein klares Zeichen, dass du gerade geschützt bist.", false],
]));

children.push(h2("Punkte & Leben anzeigen (HUD)"));
children.push(pmix([
  ["Text schreibst du mit ", false], ["TEXT(x, y, text)", true],
  [". Zahlen musst du dafür in Text verwandeln – das macht ", false],
  ["STR$", true], [":", false],
]));
children.push(codeBlock([
  'TEXT(8, 6, "SCORE: " + STR$(score))',
  'TEXT(360, 6, "LEBEN: " + STR$(lives))',
]));

children.push(h2("Neustart: eine eigene SUB"));
children.push(p("Den Aufbau eines frischen Spiels (Formation bauen, Punkte und Leben zurücksetzen) brauchen wir am Anfang und bei jedem Neustart. Deshalb steckt er in einer eigenen SUB, die wir einfach aufrufen:"));
children.push(codeBlock([
  "SUB NewGame()",
  "    DIM k AS INTEGER",
  "    ' ... Formation 3x8 bauen (wie in Kapitel 6) ...",
  "    FOR k = 0 TO NBULLET - 1 : bAlive[k] = FALSE : NEXT k",
  "    FOR k = 0 TO NBOMB - 1 : boAlive[k] = FALSE : NEXT k",
  "    score = 0 : lives = 3 : invuln = 0 : diveTimer = 120 : shipX = 232",
  "END SUB",
]));
children.push(pmix([
  ["Wir rufen ", false], ["NewGame()", true],
  [" einmal vor der Schleife auf – und bei Game Over erneut, sobald Leertaste gedrückt wird. Das vollständige Programm liegt in ", false],
  ["code/kap10/treffer.dh", true], [".", false],
]));
figure("kap10_treffer.png", "Ein echtes Spiel: Punkte oben links, Leben oben rechts, ausgedünnte Formation.", 300, 400).forEach(e => children.push(e));

children.push(h2("Was du gelernt hast"));
children.push(bulletRich("Eigene FUNCTION ", "(Overlap) bündelt wiederkehrenden Code und gibt ein Ergebnis zurück."));
children.push(bulletRich("AABB-Kollision ", "= Rechteck-Überlappung mit vier AND-Bedingungen."));
children.push(bulletRich("Score, Leben, Unverwundbarkeit ", "machen aus Bewegung ein Spiel."));
children.push(bulletRich("SUB NewGame ", "+ TEXT/STR$ für Neustart und HUD."));

children.push(h2("Übung"));
children.push(bullet("Vergib mehr Punkte oder andere Werte pro Reihe."));
children.push(bullet("Gib dem Spieler 5 statt 3 Leben."));
children.push(bullet("Verlängere die Unverwundbarkeit (invuln = 90 → grösser)."));
children.push(bullet("Zusatz: Zeige „LEVEL geschafft“, wenn alle Gegner abgeschossen sind."));

// ===================== Kapitel 11 =====================
children.push(chapter("Kapitel 11: Vollbild & Kamera"));
children.push(tip("In diesem Kapitel",
  "Dein Spiel wird Vollbild – und zwar ohne dass du eine einzige Spielkoordinate ändern musst. Der Trick: die Kamera skaliert dein festes Spielfeld auf jeden Bildschirm und zentriert es."));

children.push(h2("Das Problem"));
children.push(p("Unser Spiel ist mittlerweile richtig gut – aber es hockt noch immer in einem kleinen Fensterchen mitten auf dem Bildschirm, wie ein Konzertpianist, der darauf besteht, auf einem Spielzeugklavier zu spielen. Auf einem großen, breiten Monitor schreit das geradezu nach echtem Vollbild."));
children.push(p("Hier lauert allerdings eine Falle, in die schon viele getappt sind. Unser gesamter Code rechnet mit einem festen Spielfeld von 480 × 640 Pixeln. Das Schiff, die Sterne, die Formation, jede Flugbahn, jede Kollision – alles verlässt sich auf diese Maße. Würden wir jetzt einfach das Fenster vergrößern, müssten wir streng genommen jede einzelne Zahl im ganzen Programm umrechnen. Das wäre fehleranfällig, mühsam und etwa so erfreulich wie eine Steuererklärung. Es muss eleganter gehen – und das tut es."));

children.push(h2("Die Kamera skaliert für dich"));
children.push(p("Die Lösung ist ein Gedanke, der in der Spieleentwicklung Gold wert ist: Wir trennen, wie groß das Spielfeld in unserem Kopf ist, davon, wie groß es auf dem Bildschirm erscheint. Das Spielfeld bleibt für unseren Code in alle Ewigkeit logisch 480 × 640 – daran ändert sich keine einzige Zeile. Wir öffnen aber ein großes Fenster (oder gleich Vollbild) und stellen davor eine Kamera, die unser kleines Spielfeld passend vergrößert und hübsch in die Mitte rückt."));
children.push(p("Stell dir einen Diaprojektor vor: Das Dia ist winzig und unveränderlich, aber je nachdem, wie weit die Wand entfernt ist, erscheint das Bild handtellergroß oder riesig. Genau diese Rolle übernimmt die Kamera:"));
children.push(codeBlock([
  "CONST PW AS INTEGER = 480",
  "CONST PH AS INTEGER = 640",
  'SCREEN(960, 540, "Mein Galaga", 1)',
  "SET_FULLSCREEN(TRUE)",
  "' ... pro Frame, vor dem Zeichnen: ...",
  "DIM sw AS INTEGER : sw = SCREENWIDTH()",
  "DIM sh AS INTEGER : sh = SCREENHEIGHT()",
  "DIM zoom AS FLOAT : zoom = sh / PH                 ' auf Bildhoehe skalieren",
  "DIM offx AS FLOAT : offx = (sw - PW * zoom) / 2.0  ' zentrieren",
  "CAMERA_SET(-offx / zoom, 0.0, zoom)",
]));
children.push(pmix([
  ["Nach ", false], ["CAMERA_SET", true],
  [" sind alle Zeichen-Befehle in deinen vertrauten Welt-Koordinaten (480×640) – die Kamera vergrössert und verschiebt automatisch. ", false],
  ["SCREENWIDTH()", true], ["/", false], ["SCREENHEIGHT()", true],
  [" liefern die echte (Vollbild-)Grösse, also passt sich alles jedem Monitor an.", false],
]));
children.push(why("Warum trennen wir so streng, wie groß das Spielfeld in unserem Kopf ist, von dem, was am Ende auf dem Bildschirm erscheint? Weil sich genau dieser zweite Teil ständig ändert: heute 480×640, morgen ein breiterer Monitor, übermorgen ein winziges Vorschaufenster. Solange dein Spiel intern immer mit denselben Zahlen rechnet, läuft es überall gleich zuverlässig – nur die Kamera passt die Darstellung an. Diese Trennung von „wie die Dinge sind“ und „wie sie gezeigt werden“ ist einer der wichtigsten Grundgedanken guter Software, weit über Spiele hinaus."));

children.push(h2("Letterbox & HUD"));
children.push(p("Weil unser Spielfeld hochkant ist, ein breiter Bildschirm aber quer, bleiben links und rechts Ränder – die füllen wir schwarz (Letterbox). Den HUD-Text zeichnen wir nach CAMERA_RESET wieder in Bildschirm-Koordinaten, damit er gestochen scharf bleibt:"));
children.push(codeBlock([
  "CLS(&H000000)                          ' Raender schwarz",
  "CAMERA_SET(-offx / zoom, 0.0, zoom)",
  "BOX(0, 0, PW - 1, PH - 1, &H05060F)    ' Spielfeld-Hintergrund",
  "' ... Sterne, Gegner, Schiff usw. zeichnen (Welt-Koordinaten) ...",
  "CAMERA_RESET()                         ' zurueck zu Bildschirm-Koordinaten",
  'TEXT(INT(offx) + 8, 6, "SCORE: " + STR$(score))',
]));
children.push(p("Etwas rechtsbündig an den Spielfeldrand zu setzen ist kniffliger, als es aussieht: Du musst wissen, wie breit dein Text überhaupt wird. Genau das verrät TEXT_WIDTH – und zwar in der Schrift und Größe, die gerade aktiv ist:"));
children.push(codeBlock([
  'DIM hinweis AS STRING : hinweis = "F1:CRT"',
  "TEXT(INT(sw - offx) - TEXT_WIDTH(hinweis) - 4, 6, hinweis, &H5C7088)",
]));
children.push(tip("Nicht schätzen, messen", "Die erste Fassung dieses Spiels zog hier einfach 56 Pixel ab – eine geschätzte Textbreite. Auf einem großen Bildschirm ragte der Hinweis dadurch sichtbar über den Spielfeldrand in die bunten Ränder hinein. Mit TEXT_WIDTH sitzt er auf jedem Bildschirm und in jeder Schriftgröße genau dort, wo er hingehört."));
children.push(pmix([
  ["", false],
  ["CAMERA_RESET", true],
  [" hebt den Zoom wieder auf. Den Score setzen wir um ", false],
  ["offx", true],
  [" versetzt, damit er am linken Spielfeldrand sitzt – nicht im schwarzen Rand. Das vollständige Programm liegt in ", false],
  ["code/kap11/vollbild.dh", true], [".", false],
]));
figure("kap11_vollbild.png", "Vollbild: Das Spielfeld ist hochskaliert und zentriert, mit schwarzen Rändern.", 420, 240).forEach(e => children.push(e));

children.push(h2("Was du gelernt hast"));
children.push(bulletRich("Logische Koordinaten ", "(480×640) bleiben; die Kamera kümmert sich um die Anzeige."));
children.push(bulletRich("CAMERA_SET / CAMERA_RESET ", "schalten den Zoom an und wieder aus."));
children.push(bulletRich("zoom & offx ", "skalieren auf die Bildhöhe und zentrieren (Letterbox)."));
children.push(bulletRich("HUD nach CAMERA_RESET ", "= scharfer Text in Bildschirm-Koordinaten."));

children.push(h2("Übung"));
children.push(bullet("Schalte FULL auf FALSE und spiele im Fenster – die Kamera funktioniert trotzdem."));
children.push(bullet("Färbe die Letterbox-Ränder ein (statt Schwarz)."));
children.push(bullet("Zentriere die GAME-OVER-Schrift genauer auf deinem Bildschirm."));

// ===================== Kapitel 12 =====================
children.push(chapter("Kapitel 12: Politur & Ausblick"));
children.push(tip("In diesem Kapitel",
  "Der letzte Schliff: Sound, mehrere Level mit steigender Schwierigkeit – und wie du dein Spiel als fertiges Programm weitergibst. Zum Schluss ein Ausblick, wie aus diesem Grundgerüst das volle Galaga wird."));

children.push(p("Unser Spiel ist vollständig spielbar – aber spiel es einmal mit geschlossenen Augen, und dir fällt sofort auf, was fehlt: Es ist mucksmäuschenstill. Ein Arcade-Spiel ohne Krach ist wie ein Gewitter ohne Donner. In diesem letzten Kapitel geben wir dem Spiel eine Stimme, mehrere Schwierigkeitsstufen und zeigen, wie du es an Freunde weitergibst."));
children.push(h2("Sound"));
children.push(p("Du könntest jetzt anfangen, im Internet nach Sounddateien zu jagen, dich durch Lizenzbedingungen zu wühlen und dir am Ende doch unsicher zu sein, ob du das „pew“ verwenden darfst. Musst du aber nicht. GameBasic hat einen kleinen Synthesizer eingebaut, der dir deine Arcade-Geräusche direkt aus Zahlen zaubert – ganz so, wie es die alten Spielautomaten auch gemacht haben."));
children.push(pmix([
  ["Erzeugt werden diese Töne mit ", false], ["AUDIO_SFX", true],
  [" – das ist der kleine Synthesizer – und abgespielt mit ", false], ["PLAYSOUND", true], [":", false],
]));
children.push(codeBlock([
  'IMPORT "audio"',
  'DIM sfxShoot AS INTEGER : sfxShoot = AUDIO_SFX("square", 900, -600, 0, 8, 90, 0, 0, 0.30)',
  'DIM sfxBoom  AS INTEGER : sfxBoom  = AUDIO_SFX("noise", 220, -120, 0, 60, 200, 0, 0, 0.50)',
  "' ... beim Schiessen:  PLAYSOUND(sfxShoot)",
  "' ... beim Treffer:    PLAYSOUND(sfxBoom)",
]));
children.push(pmix([
  ["Die Zahlen steuern Wellenform, Tonhöhe, Tonhöhen-Verlauf, Hüllkurve und Lautstärke – einfach ausprobieren, bis es „pew“ und „bumm“ macht. Ein eigener Musik-Generator ", false],
  ["dhsfx", true], [" hilft beim Tüfteln.", false],
]));

children.push(h2("Wellen & Level"));
children.push(p("Ein Spiel, das nach der ersten geräumten Formation einfach aufhört, lässt den Spieler etwas ratlos zurück – so als würde ein Achterbahnzug nach der ersten Kurve anhalten. Wir wollen, dass es weitergeht, und zwar fordernder. Ist die Formation also leergeräumt, beginnt sofort ein neues Level, mit etwas flotteren Gegnern."));
children.push(p("Damit wir merken, wann eine Welle besiegt ist, führen wir einen schlichten Zähler mit: Bei jedem Abschuss eins runter, und sobald er null erreicht, bauen wir eine frische Welle auf. Diese kleine Idee – mitzählen und beim Nulldurchgang reagieren – ist das Rückgrat fast jedes Fortschrittssystems in Spielen:"));
children.push(codeBlock([
  "' beim Abschuss:  bugsAlive = bugsAlive - 1",
  "' nach den Kollisionen:",
  "IF bugsAlive <= 0 THEN level = level + 1 : BuildWave()",
]));
children.push(pmix([
  ["In ", false], ["BuildWave", true],
  [" setzen wir die Formation neu auf und machen die Stürze pro Level ein wenig schneller – so steigt die Spannung. Punkte und Leben bleiben erhalten. Das vollständige Programm liegt in ", false],
  ["code/kap12/politur.dh", true], [".", false],
]));
figure("kap12_politur.png", "Das fertige Grundspiel: Vollbild, Sound, Level, HUD.", 420, 240).forEach(e => children.push(e));

children.push(h2("Als Programm weitergeben (Export)"));
children.push(p("Ein Spiel, das nur auf deinem eigenen Rechner und nur mit installiertem GameBasic läuft, ist ein bisschen wie ein selbstgebackener Kuchen, den man nicht aus der Küche tragen darf. Damit auch Freunde, Geschwister oder ahnungslose Verwandte dein Werk spielen können – ohne irgendetwas installieren zu müssen – exportierst du es als eigenständiges Programm:"));
children.push(codeBlock([
  "dhrt --export  dein-spiel.dh",
]));
children.push(p("Das erzeugt eine startfertige Datei (mit den Bildern und Tönen darin). Doppelklick genügt – kein GameBasic nötig."));

children.push(h2("Wie geht es weiter? (Ausblick)"));
children.push(p("Du hast jetzt ein komplettes, spielbares Galaga-Grundgerüst gebaut. Im Beispielspiel code/galaga.dh findest du dieselben Bausteine – plus die grossen Extras, die du als Nächstes angehen kannst:"));
children.push(bulletRich("Fangstrahl & Doppeljäger: ", "Ein Boss fängt dein Schiff – befreie es und fliege mit zwei Schiffen."));
children.push(bulletRich("Bonus-Wellen: ", "Alle paar Level eine Runde nur für Punkte."));
children.push(bulletRich("Highscore-Liste: ", "Namen eintragen, dauerhaft gespeichert (save-Modul)."));
children.push(bulletRich("Effekte: ", "Explosionen (Partikel), Bildschirm-Wackeln, CRT-Retro-Look, Regenbogen-Titel, Laufschrift."));
children.push(bulletRich("Musik & Gamepad: ", "Hintergrundmusik und Steuerung per Controller."));

// ===================== Kapitel 13 =====================
children.push(chapter("Kapitel 13: Der Automaten-Schliff"));
children.push(tip("In diesem Kapitel",
  "Vier Kleinigkeiten, die aus einem Programm ein Gerät machen: ein eigenes Fenstersymbol, ein Rütteln im Gamepad, eine Pause beim Wegklicken – und ein Attract-Modus, in dem sich das Spiel selbst spielt."));

children.push(p("Dein Spiel ist fertig. Es läuft, es macht Spaß, es hat Sound und Highscores. Was jetzt kommt, braucht kein Mensch – und trotzdem ist es genau das, was den Unterschied zwischen „ein Programm“ und „ein Gerät“ ausmacht. Stell dir einen echten Spielautomaten in einer Spielhalle vor: Er zeigt sein eigenes Logo, er brummt in den Händen, wenn es kracht, und wenn eine Weile niemand spielt, fängt er von selbst an, sich vorzuführen. Genau diese vier Dinge bauen wir jetzt ein."));

children.push(h2("Ein eigenes Fenstersymbol"));
children.push(p("Bisher trägt dein Fenster – und die exportierte .exe – das Standardsymbol der Laufzeit. Dabei liegt ein perfektes Symbol längst herum: dein Raumschiff."));
children.push(codeBlock([
  'WINDOW_ICON(LOADIMAGE("../assets/sprites/player.png"))',
]));
children.push(p("Eine Zeile, direkt nach SCREEN. Das ist der billigste Politur-Punkt im ganzen Buch."));

children.push(h2("Das Gamepad rütteln lassen"));
children.push(p("Wenn dein Schiff getroffen wird, wackelt der Bildschirm und es kracht. Mit einem Gamepad in der Hand kommt eine dritte Sinnesebene dazu – man SPÜRT den Treffer. Ein kurzer, kräftiger Impuls reicht; alles Längere nervt."));
children.push(codeBlock([
  "' Kurzer Ruettler -- nur wenn ueberhaupt ein Pad angeschlossen ist.",
  "' staerke 0..1 je Motor, dauer in SEKUNDEN.",
  "SUB Rumble(staerke AS FLOAT, dauer_s AS FLOAT)",
  "    IF JOYSTICK_COUNT() > 0 THEN JOYSTICK_RUMBLE(0, staerke, staerke, dauer_s)",
  "END SUB",
  "",
  "' ... beim Treffer, neben Wackeln und Krach:",
  "Rumble(0.7, 0.26)",
]));
children.push(p("Achte auf die Einheit: Die Dauer zählt in SEKUNDEN, nicht in Millisekunden. 0.26 ist ein knapper Stoß; wer aus Gewohnheit 260 hinschreibt, bekommt eine Minute Dauervibration (länger geht nicht, dort klemmt die Laufzeit ab)."));
children.push(tip("Erst fragen, dann rütteln", "Ohne angeschlossenes Gamepad melden die JOYSTICK-Befehle einen Fehler – sie prüfen den Index. Deshalb steckt die Abfrage JOYSTICK_COUNT() > 0 in der kleinen Hülle Rumble, und der Spielcode bleibt frei davon. Dasselbe gilt für JOYSTICK_HIT: Baust du es in eine Bedingung ein, schreib eine eigene kleine Funktion drumherum."));

children.push(h2("Pause, wenn niemand hinsieht"));
children.push(p("Klickt der Spieler mitten im Gefecht auf ein anderes Fenster, läuft dein Spiel munter weiter – und er kommt zu einem verlorenen Leben zurück. Ein einziger Befehl verrät dir, ob dein Fenster gerade im Vordergrund ist:"));
children.push(codeBlock([
  "IF NOT WINDOW_FOCUSED() THEN",
  "    CLS(BG)",
  '    CenterText(INT(SCREENHEIGHT() / 2) - 8, "PAUSE", &HFFE070)',
  "    FLIP()",
  "    CONTINUE",
  "END IF",
]));
children.push(pmix([
  ["Das ", false], ["CONTINUE", true],
  [" springt zum Anfang der Spielschleife zurück – die ganze Spiellogik wird also übersprungen, nur das Standbild mit dem Wort PAUSE erscheint. Sobald das Fenster wieder vorn ist, läuft alles weiter, als wäre nichts gewesen.", false],
]));

children.push(h2("Der Attract-Modus: das Spiel spielt sich selbst"));
children.push(p("Und jetzt das Beste. Ein Spielautomat, bei dem eine Weile niemand einwirft, fängt an, sich selbst vorzuführen – damit Vorbeigehende sehen, worum es geht. Das nennt man Attract-Modus. Bisher hätte man dafür eine kleine künstliche Intelligenz schreiben müssen, die das Schiff steuert. Es geht viel einfacher: GameBasic kann deine EINGABE aufzeichnen und später wieder einspeisen. Das Spiel merkt davon nichts – KEYHIT und KEYPRESSED liefern brav die aufgezeichneten Tastendrücke."));
children.push(codeBlock([
  'CONST DEMO_FILE  AS STRING  = "demo.txt"',
  "CONST ATTRACT_MS AS INTEGER = 12000       ' Ruhe vor dem Demo-Start",
  "DIM demoExists AS BOOLEAN : demoExists = FILEEXISTS(DEMO_FILE)",
  "DIM attract AS BOOLEAN                    ' laeuft gerade eine Demo?",
  "DIM idleSince AS INTEGER                  ' Frame der letzten echten Eingabe",
]));
children.push(p("Aufgenommen wird auf Tastendruck – F2 startet und beendet die Aufnahme:"));
children.push(codeBlock([
  "IF KEYHIT(KEY_F2) THEN",
  "    IF AUTOMATION_RECORDING() THEN",
  "        AUTOMATION_STOP()",
  "        demoExists = TRUE",
  "    ELSE",
  "        RANDOMIZE(4711)           ' derselbe Startwert wie bei der Wiedergabe",
  "        AUTOMATION_RECORD(DEMO_FILE)",
  "    END IF",
  "END IF",
]));
children.push(p("Und auf dem Titelbild wartet das Spiel darauf, dass eine Weile nichts passiert:"));
children.push(codeBlock([
  "ELSEIF demoExists AND (frame - idleSince) * 1000 / 60 > ATTRACT_MS THEN",
  "    RANDOMIZE(4711)",
  "    AUTOMATION_PLAY(DEMO_FILE)",
  "    attract = TRUE",
  "END IF",
]));
children.push(pmix([
  ["Beachte, was hier NICHT steht: kein Aufruf von ", false], ["StartGame", true],
  [". Die Aufnahme enthält ja auch den Druck auf die Leertaste, mit dem du das Spiel damals gestartet hast – die Demo startet sich also selbst. Und jede echte Eingabe bricht sie wieder ab:", false],
]));
children.push(codeBlock([
  "IF attract AND (KEY_ANY_HIT() <> -1 OR JOYSTICK_ANY_BUTTON() <> -1) THEN",
  "    AUTOMATION_STOP()             ' Spieler uebernimmt",
  "    attract = FALSE",
  "    mode = GS.TITLE",
  "END IF",
]));
children.push(why("Warum steht bei Aufnahme UND Wiedergabe dasselbe RANDOMIZE? Weil aufgezeichnet wird, was du GEDRÜCKT hast – nicht, was daraus wurde. Die Gegner fliegen ihre Bahnen weiterhin nach Zufall. Bekommt die Wiedergabe andere Zufallszahlen als die Aufnahme, drückt das Schiff zwar dieselben Tasten, trifft aber ins Leere und stirbt an Stellen, an denen du damals ausgewichen bist. Mit demselben Startwert würfelt der Zufall dieselbe Reihenfolge – und die Demo läuft wie am Schnürchen."));
children.push(tip("Selbst ausprobieren", "Starte das Spiel, drücke F2, spiel eine gute Runde, drücke wieder F2. Ab jetzt liegt neben deinem Programm eine Datei demo.txt. Geh zurück zum Titelbild und lass die Finger von der Tastatur – nach zwölf Sekunden übernimmt dein eigener Geist das Steuer. Übrigens: demo.txt ist reiner Text. Öffne sie ruhig mal in einem Editor; jede Zeile ist ein Ereignis mit Bildnummer, Art und Werten."));

children.push(h2("Was du gelernt hast"));
children.push(bulletRich("WINDOW_ICON ", "gibt Fenster und exportierter .exe ein eigenes Symbol."));
children.push(bulletRich("JOYSTICK_RUMBLE ", "lässt das Gamepad vibrieren – vorher mit JOYSTICK_COUNT prüfen, ob eines da ist."));
children.push(bulletRich("WINDOW_FOCUSED ", "verrät, ob dein Fenster vorne ist – der ehrlichste Weg zur automatischen Pause."));
children.push(bulletRich("AUTOMATION_RECORD / _PLAY ", "zeichnen Eingabe auf und spielen sie wieder ein: Attract-Modus, nachspielbare Fehlerberichte, automatische Tests."));

children.push(h2("Übung"));
children.push(bullet("Nimm zwei verschiedene Demos auf und wähle beim Attract-Modus zufällig eine aus."));
children.push(bullet("Blende während der Demo groß „DEMO“ ein, damit niemand denkt, das Spiel hänge."));
children.push(bullet("Lass das Gamepad auch beim Levelaufstieg kurz rütteln – aber schwächer als beim Treffer."));

children.push(h2("Glückwunsch!"));
children.push(p("Halt einen Moment inne und sieh dir an, was du geschafft hast. Du hast bei einem komplett leeren schwarzen Fenster angefangen – und am Ende steht ein echtes Arcade-Spiel mit einfliegenden, stürzenden, bombenwerfenden Gegnern, mit Punkten, Leben, mehreren Leveln, Sound und Effekten. Das ist keine Kleinigkeit. Das ist genau das, was professionelle Spieleentwickler tun, nur eine Nummer kleiner."));
children.push(p("Und das eigentlich Verblüffende: Ganz nebenbei, fast unbemerkt, hast du dabei das halbe Handwerk des Programmierens gelernt. Variablen und Schleifen, Arrays und Klassen, Funktionen und Module, Zustands-Maschinen und Kollisionsprüfungen – das sind keine Galaga-Spezialitäten, das sind die Bausteine, aus denen jede Software der Welt gebaut ist, vom Taschenrechner bis zur Raumfahrt. Du hast sie nicht auswendig gelernt, du hast sie benutzt. Das ist ein himmelweiter Unterschied."));
children.push(p("Jetzt bist du dran. Verändere die Werte, bis sich das Spiel genau richtig anfühlt. Zeichne eigene Gegner. Erfinde eine neue Waffe, einen Endboss, ein zweites Spielfeld. Brich Dinge absichtlich, nur um zu sehen, was passiert – und repariere sie wieder. Genau so wird aus dem Spiel aus diesem Buch dein Spiel."));
children.push(p("Es hat mir große Freude gemacht, dich auf diesem Weg zu begleiten. Wenn du dein fertiges Werk jemandem zeigst und derjenige fragt „Das hast du selbst gemacht?“, dann sag ruhig laut und deutlich: Ja. Habe ich. Viel Spaß beim Weiterbauen – und auf den nächsten Highscore."));
children.push(new Paragraph({
  spacing: { before: 240 },
  children: [new TextRun({ text: "— Hans Schnorrenberger", italics: true, size: 22, color: C_CAP })],
}));

// ===================== Inhaltsverzeichnis =====================
// Erst jetzt sind alle Ueberschriften (tocEntries) bekannt. Wir bauen das
// Verzeichnis und fuegen es an der gemerkten Stelle (Seite 2) ein.
let tocPages = {};
try { tocPages = JSON.parse(fs.readFileSync(path.join(__dirname, "toc_pages.json"), "utf8")); } catch (e) {}
// Titel in Lese-Reihenfolge fuer den Mess-Schritt (make_book.py) ablegen:
fs.writeFileSync(path.join(__dirname, "toc_titles.json"),
  JSON.stringify(tocEntries.map(e => e.title), null, 2));

const tocBlock = [
  new Paragraph({
    spacing: { after: 220 },
    border: { bottom: { style: BorderStyle.SINGLE, size: 12, color: C_ACCENT, space: 4 } },
    children: [new TextRun({ text: "Inhalt", bold: true, color: C_TITLE, size: 36 })],
  }),
];
for (const e of tocEntries) {
  const pg = tocPages[e.title];
  const isChap = e.title.startsWith("Kapitel");
  tocBlock.push(new Paragraph({
    spacing: { after: 90 },
    tabStops: [{ type: TabStopType.RIGHT, position: 9360, leader: LeaderType.DOT }],
    children: [
      new TextRun({ text: e.title, size: 22, bold: isChap, color: isChap ? C_TITLE : "000000" }),
      new TextRun({ children: [new Tab()] }),
      new TextRun({ text: pg ? String(pg) : "", size: 22, bold: isChap }),
    ],
  }));
}
tocBlock.push(new Paragraph({ children: [new PageBreak()] }));
children.splice(TOC_INSERT_AT, 0, ...tocBlock);

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
    { reference: "num3", levels: [{ level: 0, format: LevelFormat.DECIMAL, text: "%1.", alignment: AlignmentType.LEFT,
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
