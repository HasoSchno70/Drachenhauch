// GameBasic-Lehrbuch -- baut ein farbiges, druckbares .docx-Referenz-/Lehrbuch.
// Inhalt liegt modular in content/NN_*.js (jede Datei exportiert (H)=>[bloecke]).
// Dieser Renderer stellt die Bausteine (H) bereit, laedt die Module sortiert und
// setzt das Dokument zusammen.  Aufruf:  node build_book.js
const fs = require("fs");
const path = require("path");
const {
  Document, Packer, Paragraph, TextRun, ImageRun, AlignmentType, HeadingLevel,
  BorderStyle, Table, TableRow, TableCell, WidthType, ShadingType, PageBreak,
  Footer, PageNumber, LevelFormat, InternalHyperlink,
  Tab, TabStopType, LeaderType, LineRuleType,
} = require("docx");

// Zeilenabstand fuer Fliesstext (~1,3) -- luftiger, besser lesbar.
const LINE = 312;
const SP = (after, opt = {}) => ({ after, line: LINE, lineRule: LineRuleType.AUTO, ...opt });

const IMG = path.join(__dirname, "images");

// --- Farbpalette ---
const C_TITLE = "1B6CA8";   // Blau (H1)
const C_H2 = "C0451B";      // Orange-Rot (H2)
const C_PART = "10507F";    // dunkles Blau (Teil-Trennseiten)
const C_ACCENT = "1B6CA8";
const C_CMD = "9A3412";     // Befehlsname (kraeftiges Braun-Orange)
const C_CAP = "6A6A6A";     // Bildunterschrift grau
const C_CODEBG = "F4F4F4";  // Code-Hintergrund
const C_OUTBG = "EAF4EA";   // Ausgabe-Hintergrund (gruenlich)
const C_OUTBD = "9AC79A";

// ---------------------------------------------------------------- Bilder
function pngSize(file) {
  const b = fs.readFileSync(file);
  return { w: b.readUInt32BE(16), h: b.readUInt32BE(20) };
}
function figure(name, caption, maxW = 480, maxH = 320) {
  const file = path.join(IMG, name);
  if (!fs.existsSync(file)) {                 // fehlendes Bild -> Platzhalter
    return [new Paragraph({ alignment: AlignmentType.CENTER, spacing: { after: 160 },
      children: [new TextRun({ text: `[Bild: ${name}]`, italics: true, color: C_CAP, size: 18 })] })];
  }
  const { w, h } = pngSize(file);
  let W = maxW, H = Math.round((maxW * h) / w);
  if (H > maxH) { H = maxH; W = Math.round((maxH * w) / h); }
  return [
    new Paragraph({ alignment: AlignmentType.CENTER, spacing: { before: 160, after: 40 }, keepNext: true,
      children: [new ImageRun({ type: "png", data: fs.readFileSync(file),
        transformation: { width: W, height: H },
        altText: { title: caption, description: caption, name } })] }),
    new Paragraph({ alignment: AlignmentType.CENTER, spacing: { after: 200 },
      children: [new TextRun({ text: caption, italics: true, color: C_CAP, size: 18 })] }),
  ];
}

// ---------------------------------------------------------------- Text
function p(text, opts = {}) {
  return new Paragraph({ spacing: SP(180), ...opts,
    children: [new TextRun({ text, size: 22 })] });
}
// Absatz mit gemischtem Text + Inline-Code: parts = ["text", ["code", true], ...]
function pmix(parts) {
  return new Paragraph({ spacing: SP(180),
    children: parts.map((x) => Array.isArray(x)
      ? new TextRun({ text: x[0], font: "Consolas", size: 21, color: "335577" })
      : new TextRun({ text: x, size: 22 })) });
}
function bullet(text) {
  return new Paragraph({ numbering: { reference: "bul", level: 0 }, spacing: SP(100),
    children: [new TextRun({ text, size: 22 })] });
}
function bulletRich(boldText, rest) {
  return new Paragraph({ numbering: { reference: "bul", level: 0 }, spacing: SP(100),
    children: [new TextRun({ text: boldText, bold: true, size: 22 }),
               new TextRun({ text: rest, size: 22 })] });
}

// ---------------------------------------------------------------- Kaesten
function _box(title, text, bg, bd, titleColor) {
  const border = { style: BorderStyle.SINGLE, size: 6, color: bd };
  const kids = [new Paragraph({ spacing: { after: text ? 40 : 0 },
    children: [new TextRun({ text: title, bold: true, color: titleColor, size: 22 })] })];
  if (text) (Array.isArray(text) ? text : [text]).forEach((t) =>
    kids.push(new Paragraph({ spacing: SP(0), children: [new TextRun({ text: t, size: 22 })] })));
  const table = new Table({ width: { size: 9360, type: WidthType.DXA }, columnWidths: [9360],
    rows: [new TableRow({ cantSplit: true, children: [new TableCell({
      width: { size: 9360, type: WidthType.DXA },
      borders: { top: border, bottom: border, left: border, right: border },
      shading: { fill: bg, type: ShadingType.CLEAR },
      margins: { top: 140, bottom: 140, left: 180, right: 180 }, children: kids })] })] });
  // Tabellen haben kein Nachher-Abstand -> kleiner Abstandshalter, damit der
  // folgende Text nicht klebt.
  return [new Paragraph({ spacing: { before: 60 }, children: [] }), table,
          new Paragraph({ spacing: { after: 160 }, children: [] })];
}
function tip(title, text) { return _box(title, text, "E7F2FA", "9CC8E6", C_ACCENT); }
function note(text, title = "Merke") { return _box(title, text, "FDF3E0", "E0B96A", "9A6A1E"); }
function warn(text, title = "Achtung") { return _box(title, text, "FBEAEA", "E0A0A0", "B23030"); }

// ---------------------------------------------------------------- Code
function codeBlock(lines, opts = {}) {
  const bg = opts.out ? C_OUTBG : C_CODEBG;
  const bd = opts.out ? C_OUTBD : C_ACCENT;
  const runs = (Array.isArray(lines) ? lines : [lines]).map((ln, i) =>
    new TextRun({ text: ln, font: "Consolas", size: 19, break: i === 0 ? 0 : 1 }));
  return new Paragraph({
    shading: { fill: bg, type: ShadingType.CLEAR },
    border: { left: { style: BorderStyle.SINGLE, size: 18, color: bd, space: 6 } },
    spacing: { before: 100, after: 200, line: 264, lineRule: LineRuleType.AUTO },
    indent: { left: 140 },
    keepLines: true, children: runs });
}
function smallLabel(text) {
  return new Paragraph({ spacing: { before: 120, after: 30 }, keepNext: true,
    children: [new TextRun({ text, italics: true, bold: true, color: C_CAP, size: 18 })] });
}

// Standardisierter Befehls-Eintrag.
// cmd(name, syntax, desc, codeLines, {out, fig, caption})
function cmd(name, syntax, desc, codeLines, opts = {}) {
  const out = [];
  out.push(new Paragraph({ spacing: { before: 360, after: 40 }, keepNext: true, keepLines: true,
    border: { bottom: { style: BorderStyle.SINGLE, size: 4, color: "DDD0C0", space: 2 } },
    children: [new TextRun({ text: name, bold: true, font: "Consolas", size: 24, color: C_CMD })] }));
  if (syntax) out.push(new Paragraph({ spacing: { after: 120 }, keepNext: true,
    children: [new TextRun({ text: "Syntax:  ", bold: true, size: 18, color: C_CAP }),
               new TextRun({ text: syntax, font: "Consolas", size: 19, color: "335577" })] }));
  if (desc) (Array.isArray(desc) ? desc : [desc]).forEach((d) => out.push(p(d)));
  if (codeLines && codeLines.length) { out.push(smallLabel("Beispiel")); out.push(codeBlock(codeLines)); }
  if (opts.out) { out.push(smallLabel("Ausgabe")); out.push(codeBlock(opts.out, { out: true })); }
  if (opts.fig) figure(opts.fig, opts.caption || "").forEach((e) => out.push(e));
  return out;
}

// ---------------------------------------------------------------- Ueberschriften + ToC
const tocEntries = [];
let _bm = 0;
function _heading(text, kind) {
  const bm = "toc_" + (_bm++);
  tocEntries.push({ title: text, bm, kind });
  if (kind === "part") {
    return new Paragraph({ pageBreakBefore: true, alignment: AlignmentType.CENTER,
      spacing: { before: 2600, after: 200 },
      children: [new TextRun({ text, bold: true, size: 52, color: C_PART })] });
  }
  return new Paragraph({ heading: HeadingLevel.HEADING_1, keepNext: true, keepLines: true,
    ...(kind === "chapter" ? { pageBreakBefore: true } : {}),
    spacing: kind === "chapter" ? { before: 0, after: 160 } : { before: 400, after: 140 },
    border: { bottom: { style: BorderStyle.SINGLE, size: 12, color: C_ACCENT, space: 4 } },
    children: [new TextRun({ text })] });
}
function h1(t) { return _heading(t, "h1"); }
function chapter(t) { return _heading(t, "chapter"); }
function part(t) { return _heading(t, "part"); }
function h2(t) {
  return new Paragraph({ heading: HeadingLevel.HEADING_2, keepNext: true, keepLines: true,
    spacing: { before: 280, after: 120 }, children: [new TextRun({ text: t })] });
}

const H = { figure, p, pmix, bullet, bulletRich, tip, note, warn, code: codeBlock,
            cmd, h1, h2, chapter, part, smallLabel, PageBreak };

// ===================== Inhalt zusammenstellen =====================
const children = [];

// Titelseite
children.push(
  new Paragraph({ spacing: { before: 1500 }, children: [] }),
  new Paragraph({ alignment: AlignmentType.CENTER,
    children: [new TextRun({ text: "GAMEBASIC", bold: true, color: C_TITLE, size: 92 })] }),
  new Paragraph({ alignment: AlignmentType.CENTER, spacing: { before: 140, after: 80 },
    children: [new TextRun({ text: "Das Lehrbuch", size: 40, color: C_H2, bold: true })] }),
  new Paragraph({ alignment: AlignmentType.CENTER, spacing: { after: 80 },
    children: [new TextRun({ text: "Programmieren lernen und alle Befehle verstehen", size: 26, italics: true, color: C_CAP })] }),
  new Paragraph({ alignment: AlignmentType.CENTER, spacing: { before: 1800 },
    children: [new TextRun({ text: "von Hans Schnorrenberger", size: 26, bold: true })] }),
  new Paragraph({ children: [new PageBreak()] }),
);

const TOC_INSERT_AT = children.length;   // hier kommt das Inhaltsverzeichnis hin

// Content-Module der Reihe nach laden (Dateiname-Sortierung = Reihenfolge).
const contentDir = path.join(__dirname, "content");
const mods = fs.existsSync(contentDir)
  ? fs.readdirSync(contentDir).filter((f) => f.endsWith(".js")).sort() : [];
function flatten(a, acc) { for (const x of a) Array.isArray(x) ? flatten(x, acc) : acc.push(x); return acc; }
for (const m of mods) {
  const blocks = require(path.join(contentDir, m))(H);
  flatten(blocks, []).forEach((b) => children.push(b));
}

// ---------------------------------------------------------------- Inhaltsverzeichnis
const pages = fs.existsSync(path.join(__dirname, "toc_pages.json"))
  ? JSON.parse(fs.readFileSync(path.join(__dirname, "toc_pages.json"), "utf-8")) : {};
fs.writeFileSync(path.join(__dirname, "toc_titles.json"),
  JSON.stringify(tocEntries.map((e) => e.title), null, 2));

const tocChildren = [new Paragraph({ heading: HeadingLevel.HEADING_1, spacing: { after: 200 },
  border: { bottom: { style: BorderStyle.SINGLE, size: 12, color: C_ACCENT, space: 4 } },
  children: [new TextRun({ text: "Inhalt" })] })];
for (const e of tocEntries) {
  if (e.kind === "h1") continue;   // nur Teile + Kapitel ins Verzeichnis
  const pg = pages[e.title] ? String(pages[e.title]) : "";
  const isPart = e.kind === "part";
  tocChildren.push(new Paragraph({
    tabStops: [{ type: TabStopType.RIGHT, position: 9200, leader: LeaderType.DOT }],
    spacing: { after: isPart ? 40 : 20, before: isPart ? 120 : 0 },
    indent: isPart ? {} : { left: 280 },
    children: [
      new TextRun({ text: e.title, bold: isPart, size: isPart ? 24 : 22,
                    color: isPart ? C_PART : "222222" }),
      new TextRun({ text: "\t" + pg, size: isPart ? 24 : 22, color: "555555" }),
    ] }));
}
tocChildren.push(new Paragraph({ children: [new PageBreak()] }));
children.splice(TOC_INSERT_AT, 0, ...tocChildren);

// ===================== Dokument =====================
const doc = new Document({
  creator: "Hans Schnorrenberger",
  title: "GameBasic – Das Lehrbuch",
  styles: {
    default: { document: { run: { font: "Arial", size: 22 } } },
    paragraphStyles: [
      { id: "Heading1", name: "Heading 1", basedOn: "Normal", next: "Normal", quickFormat: true,
        run: { size: 34, bold: true, font: "Arial", color: C_TITLE },
        paragraph: { spacing: { before: 320, after: 120 }, outlineLevel: 0 } },
      { id: "Heading2", name: "Heading 2", basedOn: "Normal", next: "Normal", quickFormat: true,
        run: { size: 26, bold: true, font: "Arial", color: C_H2 },
        paragraph: { spacing: { before: 200, after: 80 }, outlineLevel: 1 } },
    ],
  },
  numbering: { config: [
    { reference: "bul", levels: [{ level: 0, format: LevelFormat.BULLET, text: "•", alignment: AlignmentType.LEFT,
      style: { paragraph: { indent: { left: 540, hanging: 280 } } } }] },
  ] },
  sections: [{
    properties: { page: { size: { width: 12240, height: 15840 }, margin: { top: 1440, right: 1440, bottom: 1440, left: 1440 } } },
    footers: { default: new Footer({ children: [new Paragraph({ alignment: AlignmentType.CENTER,
      children: [new TextRun({ text: "GameBasic – Das Lehrbuch  ·  ", size: 16, color: C_CAP }),
                 new TextRun({ children: [PageNumber.CURRENT], size: 16, color: C_CAP })] })] }) },
    children,
  }],
});

Packer.toBuffer(doc).then((buf) => {
  fs.writeFileSync(path.join(__dirname, "GameBasic-Lehrbuch.docx"), buf);
  console.log(`OK -> GameBasic-Lehrbuch.docx (${mods.length} Module, ${tocEntries.length} Ueberschriften)`);
});
