// Drachenhauch-Lehrbuch -- baut ein EPUB 3 aus DENSELBEN Inhalts-Modulen wie das
// .docx.  Moeglich ist das, weil content/NN_*.js die Bausteine nicht selbst
// importiert, sondern als (H) => [bloecke] geschrieben ist: hier kommt ein
// zweites H herein, das XHTML erzeugt statt docx-Objekte.  Eine Quelle, zwei
// Ausgaben -- ein dritter Satz Kapiteltexte waere binnen einer Woche vom
// anderen abgedriftet.
//
// Warum kein Konverter?  docx->epub (LibreOffice, pandoc) macht aus jeder
// Formatierung Inline-Styles und aus dem Seitenlayout Unsinn; EPUB ist
// fliessend, das .docx ist auf A4 gesetzt.  Direkt gebaut passt sich das Buch
// der Schriftgroesse des Lesers an.
//
// Aufruf:  node build_epub.js [zielpfad.epub] [--lang en]
//          (ohne Argument: Drachenhauch-Lehrbuch.epub neben dieser Datei;
//           mit --lang en: Drachenhauch-Handbook.epub aus i18n/en.json)
const fs = require("fs");
const path = require("path");
const JSZip = require("jszip");

const HERE = __dirname;
const IMG = path.join(HERE, "images");
const AUTOR = "Hans Schnorrenberger";

// ---------------------------------------------------------------- Sprache
const ARGS = process.argv.slice(2);
const _li = ARGS.indexOf("--lang");
const LANG = _li >= 0 ? (ARGS[_li + 1] || "de") : "de";
const FREI = ARGS.filter((_, i) => i !== _li && i !== _li + 1);

// Feste Texte des Renderers -- sie stehen NICHT in den Kapiteln und kommen
// darum auch nicht durch den Katalog. "Merke"/"Achtung" sind Vorgabewerte
// von note()/warn(), die der Kapiteltext gar nicht mitgibt.
const UI = {
  de: { titel: "Drachenhauch – Das Lehrbuch", untertitel: "Das Lehrbuch",
        zeile: "Programmieren lernen und alle Befehle verstehen", von: "von",
        inhalt: "Inhalt", merke: "Merke", achtung: "Achtung",
        beispiel: "Beispiel", ausgabe: "Ausgabe", datei: "Drachenhauch-Lehrbuch.epub" },
  en: { titel: "Drachenhauch – The Handbook", untertitel: "The Handbook",
        zeile: "Learn to program and understand every command", von: "by",
        inhalt: "Contents", merke: "Remember", achtung: "Careful",
        beispiel: "Example", ausgabe: "Output", datei: "Drachenhauch-Handbook.epub" },
}[LANG];
if (!UI) { console.error(`Unbekannte Sprache: ${LANG} (de|en)`); process.exit(2); }

const TITEL = UI.titel;

// Katalog laden. Fehlt ein Eintrag, bleibt der deutsche Satz stehen -- das
// Buch baut also auch halb uebersetzt, statt mitten im Kapitel abzubrechen.
const KATALOG = LANG === "de" ? null
  : JSON.parse(fs.readFileSync(path.join(HERE, "i18n", `${LANG}.json`), "utf8"));
// Feste Kennung: bei jedem Bau dieselbe, damit ein Lesegeraet eine neue
// Fassung als DASSELBE Buch erkennt (Lesezeichen/Fortschritt bleiben).
// Der Name darin ist Geschichte, kein Etikett: die Kennung darf sich beim
// Umbenennen NICHT aendern, sonst gilt das Buch als ein anderes und jeder
// Leser verliert Lesezeichen und Fortschritt.
const UUID = "urn:uuid:6e1f2c40-9b3a-4d17-8a55-gamebasic-lehrbuch";

// ---------------------------------------------------------------- Werkzeug
const esc = (s) => String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;")
  .replace(/>/g, "&gt;");
const escAttr = (s) => esc(s).replace(/"/g, "&quot;");

// ---------------------------------------------------------------- Bausteine
const bilder = new Set();          // welche PNGs wirklich gebraucht werden

function figure(name, caption) {
  const file = path.join(IMG, name);
  if (!fs.existsSync(file)) {
    return `<p class="fehlt">[Bild: ${esc(name)}]</p>`;
  }
  bilder.add(name);
  return `<figure><img src="../images/${escAttr(name)}" alt="${escAttr(caption)}"/>`
       + `<figcaption>${esc(caption)}</figcaption></figure>`;
}

const p = (text) => `<p>${esc(text)}</p>`;
// parts = ["text", ["code", true], ...] -- Array-Eintrag = Inline-Code
const pmix = (parts) => "<p>" + parts.map((x) =>
  Array.isArray(x) ? `<code>${esc(x[0])}</code>` : esc(x)).join("") + "</p>";

// Aufzaehlungen kommen als einzelne Aufrufe herein, nicht als Gruppe. Sie
// werden hier als <li> erzeugt und spaeter von huelleListen() zu <ul>
// zusammengefasst -- sonst waere jeder Punkt eine eigene Liste, und
// Lesegeraete setzen zwischen zwei Listen einen Absatzabstand.
const bullet = (text) => `<li>${esc(text)}</li>`;
const bulletRich = (fett, rest) => `<li><strong>${esc(fett)}</strong>${esc(rest)}</li>`;

function _box(art, titel, text) {
  const zeilen = text ? (Array.isArray(text) ? text : [text]) : [];
  return `<aside class="kasten ${art}"><p class="kasten-titel">${esc(titel)}</p>`
       + zeilen.map((t) => `<p>${esc(t)}</p>`).join("") + `</aside>`;
}
// Reihenfolge wie im .docx: tip(Titel, Text), note/warn(Text, Titel).
const tip = (titel, text) => _box("tip", titel, text);
const note = (text, titel = UI.merke) => _box("note", titel, text);
const warn = (text, titel = UI.achtung) => _box("warn", titel, text);

function codeBlock(lines, opts = {}) {
  const arr = Array.isArray(lines) ? lines : [lines];
  return `<pre class="${opts.out ? "ausgabe" : "code"}">`
       + arr.map(esc).join("\n") + `</pre>`;
}
const smallLabel = (text) => `<p class="label">${esc(text)}</p>`;
const sig = (text) => `<p class="sig">${esc(text)}</p>`;

function table(rows, opts = {}) {
  const mono = opts.mono || [];
  const zelle = (c, i, kopf) => {
    const cfg = (c && typeof c === "object" && !Array.isArray(c)) ? c : { text: String(c) };
    let inhalt = "";
    if (cfg.swatch) inhalt += `<span class="swatch" style="background:#${escAttr(cfg.swatch)}"></span>`;
    const txt = cfg.text !== undefined ? esc(cfg.text) : "";
    const stil = [];
    if (cfg.color) stil.push(`color:#${cfg.color}`);
    const klasse = (mono.includes(i) || cfg.mono) ? ' class="mono"' : "";
    const stilAttr = stil.length ? ` style="${escAttr(stil.join(";"))}"` : "";
    inhalt += (cfg.bold && !kopf) ? `<strong>${txt}</strong>` : txt;
    return `<${kopf ? "th" : "td"}${klasse}${stilAttr}>${inhalt}</${kopf ? "th" : "td"}>`;
  };
  let s = "<table>";
  if (opts.headers) s += "<thead><tr>"
    + opts.headers.map((h, i) => zelle(h, i, true)).join("") + "</tr></thead>";
  s += "<tbody>" + rows.map((r) => "<tr>"
    + r.map((c, i) => zelle(c, i, false)).join("") + "</tr>").join("") + "</tbody></table>";
  return s;
}

function cmd(name, syntax, desc, codeLines, opts = {}) {
  let s = `<h3 class="cmd">${esc(name)}</h3>`;
  if (syntax) {
    s += `<p class="syntax"><span class="syntax-label">Syntax:</span> `
       + `<code>${String(syntax).split("\n").map(esc).join("<br/>")}</code></p>`;
  }
  if (desc) (Array.isArray(desc) ? desc : [desc]).forEach((d) => { s += p(d); });
  if (codeLines && codeLines.length) s += smallLabel(UI.beispiel) + codeBlock(codeLines);
  if (opts.out) s += smallLabel(UI.ausgabe) + codeBlock(opts.out, { out: true });
  if (opts.fig) s += figure(opts.fig, opts.caption || "");
  return s;
}

// ------------------------------------------------- Ueberschriften + Aufteilung
// Jeder `part`/`chapter` beginnt eine neue XHTML-Datei. Ein EPUB als EINE
// riesige Datei laesst schwache Lesegeraete beim Blaettern haengen, und der
// Fortschrittsbalken springt.
const dateien = [];      // {id, name, titel, kind, html[]}
let aktuell = null;

function neueDatei(titel, kind) {
  const id = "kap" + String(dateien.length + 1).padStart(3, "0");
  aktuell = { id, name: id + ".xhtml", titel, kind, html: [] };
  dateien.push(aktuell);
  return aktuell;
}
let teilBegonnen = false;      // schon ein Teil/Kapitel dagewesen?
function _heading(text, kind) {
  if (kind === "part" || kind === "chapter") {
    teilBegonnen = true;
    neueDatei(text, kind);
    return kind === "part" ? `<h1 class="teil">${esc(text)}</h1>`
                           : `<h1>${esc(text)}</h1>`;
  }
  // h1 im VORSPANN (Vorwort, Lesehinweise): eigene Datei UND eigener
  // Verzeichniseintrag. Im gedruckten Buch blaettert man dorthin; im EPUB
  // kommt man nur ueber das Verzeichnis hin -- ohne Eintrag waere das
  // Vorwort praktisch unerreichbar, und es klebte an der Titelseite.
  if (!teilBegonnen) {
    neueDatei(text, "vorspann");
    return `<h1>${esc(text)}</h1>`;
  }
  // h1 innerhalb eines Kapitels bleibt in der Datei und kommt nicht ins
  // Verzeichnis (genau wie im .docx).
  return `<h1 class="zwischen">${esc(text)}</h1>`;
}
const h1 = (t) => _heading(t, "h1");
const chapter = (t) => _heading(t, "chapter");
const part = (t) => _heading(t, "part");
const h2 = (t) => `<h2>${esc(t)}</h2>`;

const H = { figure, p, pmix, bullet, bulletRich, tip, note, warn, code: codeBlock,
            cmd, table, h1, h2, chapter, part, smallLabel, sig, PageBreak: null };

// Uebersetzendes H -- schiebt jede Zeichenkette durch den Katalog, bevor
// sie beim Renderer ankommt. Die 75 Kapiteldateien merken davon nichts.
const HX = KATALOG ? require("./i18n").wrap(H, KATALOG) : H;

// ---------------------------------------------------------------- Inhalt laden
const contentDir = path.join(HERE, "content");
const mods = fs.existsSync(contentDir)
  ? fs.readdirSync(contentDir).filter((f) => f.endsWith(".js")).sort() : [];
function flatten(a, acc) { for (const x of a) Array.isArray(x) ? flatten(x, acc) : acc.push(x); return acc; }

neueDatei("Titel", "titel");
aktuell.html.push(
  `<div class="titelseite">`,
  `<p class="marke">DRACHENHAUCH</p>`,
  `<p class="untertitel">${esc(UI.untertitel)}</p>`,
  `<p class="zeile">${esc(UI.zeile)}</p>`,
  `<p class="autor">${UI.von} ${esc(AUTOR)}</p>`,
  `</div>`);

for (const m of mods) {
  const blocks = flatten(require(path.join(contentDir, m))(HX), []);
  for (const b of blocks) {
    if (b === null || b === undefined) continue;   // PageBreak hat im EPUB keinen Sinn
    aktuell.html.push(b);
  }
}

// Einzelne <li> zu <ul>-Gruppen zusammenfassen (siehe bullet()).
function huelleListen(teile) {
  const out = [];
  let offen = false;
  for (const t of teile) {
    const istLi = t.startsWith("<li>");
    if (istLi && !offen) { out.push("<ul>"); offen = true; }
    if (!istLi && offen) { out.push("</ul>"); offen = false; }
    out.push(t);
  }
  if (offen) out.push("</ul>");
  return out.join("\n");
}

// ---------------------------------------------------------------- Stylesheet
const CSS = `/* Drachenhauch-Lehrbuch -- EPUB.  Bewusst wenige feste Groessen:
   der Leser stellt die Schrift ein, alles rechnet in em. */
/* Beide Modi anmelden, aber Vorder-/Hintergrund NICHT selbst setzen: das
   Lesegeraet gewinnt (Sepia, Nacht, eigene Schrift). Ohne diese Zeile nimmt
   ein Betrachter im Nachtmodus die dunkle Flaeche, laesst die Schrift aber
   auf Schwarz stehen -- dunkel auf dunkel. */
html { color-scheme: light dark; }
body { font-family: sans-serif; line-height: 1.5; margin: 0 5%; hyphens: auto; }
h1 { color: #1B6CA8; font-size: 1.5em; border-bottom: 2px solid #1B6CA8;
     padding-bottom: .2em; margin: 1.2em 0 .6em; page-break-after: avoid; }
h1.teil { color: #10507F; font-size: 2em; border: 0; text-align: center;
          margin-top: 3em; }
h1.zwischen { margin-top: 1.6em; }
h2 { color: #C0451B; font-size: 1.15em; margin: 1.4em 0 .4em;
     page-break-after: avoid; }
h3.cmd { color: #9A3412; font-family: monospace; font-size: 1.05em;
         border-bottom: 1px solid #DDD0C0; padding-bottom: .15em;
         margin: 1.6em 0 .3em; page-break-after: avoid; }
p { margin: 0 0 .7em; text-align: justify; }
code { font-family: monospace; font-size: .92em; color: #335577; }
pre { font-family: monospace; font-size: .82em; line-height: 1.35;
      background: #F4F4F4; color: #1F1F1F; border-left: 4px solid #1B6CA8;
      padding: .6em .8em; margin: .5em 0 1em; white-space: pre-wrap;
      word-wrap: break-word; page-break-inside: avoid; }
pre.ausgabe { background: #EAF4EA; color: #1F2F1F; border-left-color: #9AC79A; }
p.sig { font-style: italic; color: #6A6A6A; margin-top: 1em; }
p.label { font-style: italic; font-weight: bold; color: #6A6A6A;
          font-size: .8em; margin: .8em 0 .2em; page-break-after: avoid; }
p.syntax { margin: 0 0 .6em; }
p.syntax-label, .syntax-label { font-weight: bold; color: #6A6A6A; font-size: .8em; }
ul { margin: .3em 0 .9em 1.2em; padding: 0; }
li { margin-bottom: .3em; text-align: justify; }
figure { margin: 1.2em 0; text-align: center; page-break-inside: avoid; }
figure img { max-width: 100%; }
figcaption { font-style: italic; color: #6A6A6A; font-size: .8em;
             margin-top: .3em; }
p.fehlt { font-style: italic; color: #6A6A6A; text-align: center; }
aside.kasten { border: 1px solid; padding: .6em .8em; margin: 1em 0;
               color: #1F1F1F; page-break-inside: avoid; }
aside.kasten p { margin: 0 0 .35em; }
aside.kasten p:last-child { margin-bottom: 0; }
aside.kasten .kasten-titel { font-weight: bold; }
aside.tip  { color: #1F1F1F; background: #E7F2FA; border-color: #9CC8E6; }
aside.tip  .kasten-titel { color: #1B6CA8; }
aside.note { color: #1F1F1F; background: #FDF3E0; border-color: #E0B96A; }
aside.note .kasten-titel { color: #8A5E15; }
aside.warn { color: #1F1F1F; background: #FBEAEA; border-color: #E0A0A0; }
aside.warn .kasten-titel { color: #B23030; }
table { border-collapse: collapse; width: 100%; font-size: .82em;
        margin: .6em 0 1.2em; }
th { background: #1B6CA8; color: #fff; text-align: left; }
td { border: 1px solid #D0D0D0; padding: .25em .45em; vertical-align: top; }
th { border: 1px solid #D0D0D0; padding: .25em .45em; vertical-align: top; }
td.mono, th.mono { font-family: monospace; }
/* ohne-schrift: nur ein Farbfeld im Anhang */
.swatch { display: inline-block; width: 1.6em; height: .9em;
          margin-right: .4em; border: 1px solid #999; }
.titelseite { text-align: center; margin-top: 25%; }
.titelseite .marke { font-size: 2.6em; font-weight: bold; color: #1B6CA8;
                     margin-bottom: .1em; }
.titelseite .untertitel { font-size: 1.5em; font-weight: bold; color: #C0451B; }
.titelseite .zeile { font-style: italic; color: #6A6A6A; }
.titelseite .autor { margin-top: 3em; font-weight: bold; }
nav ol { list-style: none; padding-left: 0; }
nav ol ol { padding-left: 1.2em; }
nav li { margin: .25em 0; }
nav .teil > a { font-weight: bold; color: #10507F; }

/* Nachtmodus. Ein Kasten mit hellem Hintergrund und dunkler Schrift bleibt
   zwar LESBAR (siehe Farbpaare oben), leuchtet im dunklen Buch aber wie eine
   Taschenlampe. Lesegeraete, die prefers-color-scheme koennen (Apple Books,
   viele Android-Leser), bekommen deshalb dunkle Entsprechungen. */
@media (prefers-color-scheme: dark) {
  h1 { color: #6FB3E0; border-bottom-color: #6FB3E0; }
  h1.teil { color: #6FB3E0; }
  h2 { color: #F0885A; }
  h3.cmd { color: #E0A070; border-bottom-color: #4A4038; }
  code { color: #9CC4E4; }
  pre { background: #1E1E1E; color: #E0E0E0; border-left-color: #3D7EA6; }
  pre.ausgabe { background: #1A241A; color: #D8E8D8; border-left-color: #5A8A5A; }
  aside.kasten { color: #E0E0E0; }
  aside.tip  { color: #1F1F1F; background: #16283A; border-color: #2F5C80; }
  aside.tip  .kasten-titel { color: #6FB3E0; }
  aside.note { color: #1F1F1F; background: #332A18; border-color: #6E5A2E; }
  aside.note .kasten-titel { color: #D9AE5E; }
  aside.warn { color: #1F1F1F; background: #38201F; border-color: #7A3E3E; }
  aside.warn .kasten-titel { color: #E38A8A; }
  th { background: #2A4A63; color: #FFFFFF; }
  td, th { border-color: #444; }
  figcaption, p.label, p.sig, .syntax-label { color: #A0A0A0; }
  .titelseite .marke { color: #6FB3E0; }
  .titelseite .untertitel { color: #F0885A; }
  .titelseite .zeile { color: #A0A0A0; }
  nav .teil > a { color: #6FB3E0; }
}
`;

// ---------------------------------------------------------------- Zusammenbau
const seite = (titel, koerper, navAttr = "") =>
`<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"
      xml:lang="${LANG}" lang="${LANG}"${navAttr}>
<head><meta charset="utf-8"/><title>${esc(titel)}</title>
<link rel="stylesheet" type="text/css" href="../styles/buch.css"/></head>
<body>
${koerper}
</body></html>`;

// Verzeichnis: Teile als Ebene 1, Kapitel darunter (wie im .docx, h1 raus).
function navPunkte() {
  const out = [];
  for (const d of dateien) {
    if (d.kind === "part" || d.kind === "vorspann") out.push({ d, kinder: [] });
    else if (d.kind === "chapter") {
      if (out.length) out[out.length - 1].kinder.push(d);
      else out.push({ d, kinder: [] });
    }
  }
  return out;
}

function navXhtml() {
  const punkte = navPunkte();
  const li = punkte.map(({ d, kinder }) => {
    const unter = kinder.length
      ? "<ol>" + kinder.map((k) =>
          `<li><a href="${k.name}">${esc(k.titel)}</a></li>`).join("") + "</ol>"
      : "";
    const klasse = (d.kind === "part") ? ' class="teil"' : "";
    return `<li${klasse}><a href="${d.name}">${esc(d.titel)}</a>${unter}</li>`;
  }).join("\n");
  return seite(UI.inhalt,
    `<nav epub:type="toc" id="toc"><h1>${UI.inhalt}</h1>\n<ol>\n${li}\n</ol>\n</nav>`);
}

// NCX zusaetzlich: EPUB-3-Lesegeraete nehmen nav.xhtml, aeltere (Kindle-
// Konverter, alte Sony/Kobo) finden ohne NCX gar kein Verzeichnis.
function ncx() {
  let n = 0;
  const punkte = navPunkte();
  const punkt = (d, tiefe) => {
    n += 1;
    return `<navPoint id="n${n}" playOrder="${n}">`
         + `<navLabel><text>${esc(d.titel)}</text></navLabel>`
         + `<content src="text/${d.name}"/>`;
  };
  let s = "";
  for (const { d, kinder } of punkte) {
    s += punkt(d, 1);
    for (const k of kinder) s += punkt(k, 2) + "</navPoint>";
    s += "</navPoint>";
  }
  return `<?xml version="1.0" encoding="utf-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
<head><meta name="dtb:uid" content="${UUID}"/><meta name="dtb:depth" content="2"/>
<meta name="dtb:totalPageCount" content="0"/><meta name="dtb:maxPageNumber" content="0"/></head>
<docTitle><text>${esc(TITEL)}</text></docTitle>
<navMap>${s}</navMap></ncx>`;
}

function opf(datum) {
  const manifest = [
    `<item id="nav" href="text/nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>`,
    `<item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>`,
    `<item id="css" href="styles/buch.css" media-type="text/css"/>`,
    ...dateien.map((d) =>
      `<item id="${d.id}" href="text/${d.name}" media-type="application/xhtml+xml"/>`),
    ...[...bilder].map((b, i) =>
      `<item id="img${i}" href="images/${escAttr(b)}" media-type="image/png"/>`),
  ].join("\n    ");
  const spine = dateien.map((d) => `<itemref idref="${d.id}"/>`).join("\n    ");
  return `<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="pub-id"
         xml:lang="${LANG}">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="pub-id">${UUID}</dc:identifier>
    <dc:title>${esc(TITEL)}</dc:title>
    <dc:creator>${esc(AUTOR)}</dc:creator>
    <dc:language>${LANG}</dc:language>
    <dc:description>Programmieren lernen und alle Befehle von Drachenhauch verstehen.</dc:description>
    <meta property="dcterms:modified">${datum}</meta>
  </metadata>
  <manifest>
    ${manifest}
  </manifest>
  <spine toc="ncx">
    ${spine}
  </spine>
</package>`;
}

// ---------------------------------------------------------------- schreiben
const zip = new JSZip();
// "mimetype" MUSS der erste Eintrag und UNKOMPRIMIERT sein -- daran erkennen
// Lesegeraete die Datei ueberhaupt als EPUB.
zip.file("mimetype", "application/epub+zip", { compression: "STORE" });
zip.file("META-INF/container.xml",
`<?xml version="1.0" encoding="utf-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles><rootfile full-path="OEBPS/content.opf"
    media-type="application/oebps-package+xml"/></rootfiles>
</container>`);

const o = zip.folder("OEBPS");
o.file("styles/buch.css", CSS);
for (const d of dateien) o.file("text/" + d.name, seite(d.titel, huelleListen(d.html)));
// nav.xhtml liegt BEI den Kapiteln, nicht in OEBPS/: seine Verweise sind
// relativ, und von einer Ebene hoeher zeigten alle 84 ins Leere.
o.file("text/nav.xhtml", navXhtml());
o.file("toc.ncx", ncx());
for (const b of bilder) o.file("images/" + b, fs.readFileSync(path.join(IMG, b)));
o.file("content.opf", opf(new Date().toISOString().replace(/\.\d+Z$/, "Z")));

// Ziel ueberschreibbar (erstes Argument). Der Test baut damit in ein
// Wegwerf-Verzeichnis: die eingecheckte .epub enthaelt einen Zeitstempel
// (dcterms:modified ist in EPUB 3 Pflicht), ein Neubau am selben Ort machte
// also bei JEDEM Testlauf 4,8 MB Unterschied im Arbeitsverzeichnis auf.
const ZIEL = FREI[0] ? path.resolve(FREI[0]) : path.join(HERE, UI.datei);
zip.generateAsync({ type: "nodebuffer", compression: "DEFLATE" }).then((buf) => {
  fs.writeFileSync(ZIEL, buf);
  const kap = dateien.filter((d) => d.kind === "chapter").length;
  const teile = dateien.filter((d) => d.kind === "part").length;
  console.log(`OK -> ${path.basename(ZIEL)} `
    + `(${teile} Teile, ${kap} Kapitel, ${bilder.size} Bilder, `
    + `${(buf.length / 1024 / 1024).toFixed(1)} MB)`);
  if (HX.__fehlend && HX.__fehlend.size) {
    console.log(`   ${HX.__fehlend.size} Texte noch nicht uebersetzt `
      + `-- sie stehen deutsch im Buch.`);
  }
});
