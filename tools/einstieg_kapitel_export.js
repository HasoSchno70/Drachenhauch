// Kapitel und Querverweise des Einstiegsbuchs als JSON:
//   { kapitel: [{datei, nr, titel}], verweise: [{datei, nr}], hoechste: n,
//     tips_ohne_rumpf: [...], cmds_ohne_text: [...] }
//
// Anders als das Referenzbuch ist der Einstieg ein LINEARER Kurs: er verweist
// an 134 Stellen auf "Kapitel 12" und aehnliche Zahlen. Bis 2026-08-30 druckte
// er die Nummern aber nirgends -- weder in den Ueberschriften noch im
// Verzeichnis. Seither nummeriert `build_book.js` die Kapitel beim ZAEHLEN,
// waehrend die Verweise von Hand die DATEINUMMER meinen. Beide muessen
// uebereinstimmen, sonst verschiebt ein eingeschobenes Kapitel alle 134
// Verweise auf einmal -- lautlos.
"use strict";
const fs = require("fs");
const path = require("path");

const BUCH = path.join(__dirname, "..", "buch-einstieg", "buch");
const CONTENT = path.join(BUCH, "content");
const nix = () => "";
const kapitel = [];
const verweise = [];
const tips_ohne_rumpf = [];
const cmds_ohne_text = [];
let datei = "";

const zaehl = (s) => {
  if (typeof s === "string") {
    for (const m of s.matchAll(/Kapitel (\d+)/g)) verweise.push({ datei, nr: Number(m[1]) });
  }
  return "";
};

const H = {
  figure: nix, code: nix, sig: nix, smallLabel: nix, h1: nix, part: nix, PageBreak: null,
  p: zaehl, pmix: zaehl, bullet: zaehl, bulletRich: zaehl, h2: zaehl,
  table: (zeilen) => { JSON.stringify(zeilen || "").split('"').forEach(zaehl); return ""; },
  tip: (titel, text) => {
    zaehl(titel); zaehl(text);
    if (text === undefined || String(text).trim() === "") tips_ohne_rumpf.push({ datei, titel: String(titel).slice(0, 90) });
    return "";
  },
  note: (text, titel) => { zaehl(text); zaehl(titel); return ""; },
  warn: (text, titel) => { zaehl(text); zaehl(titel); return ""; },
  cmd: (name, sig, text) => {
    zaehl(text);
    if (!text || String(text).trim() === "") cmds_ohne_text.push({ datei, name: String(name) });
    return "";
  },
  chapter: (t) => { kapitel.push({ datei, titel: t }); return ""; },
};

function flach(a, acc) {
  for (const x of a) Array.isArray(x) ? flach(x, acc) : acc.push(x);
  return acc;
}
for (const f of fs.readdirSync(CONTENT).filter((n) => n.endsWith(".js")).sort()) {
  datei = f;
  try {
    flach(require(path.join(CONTENT, f))(H), []);
  } catch (e) {
    process.stderr.write(`uebersprungen: ${f} (${e.message})\n`);
  }
}

// Die gedruckte Nummer entsteht in build_book.js durch Zaehlen -- Vorwort und
// die Anhaenge ("A · …") zaehlen nicht mit. Hier dasselbe nachbilden.
let n = 0;
const nummeriert = kapitel.map((k) => {
  const zaehlt = k.titel !== "Vorwort" && !/^[A-Z] · /.test(k.titel);
  if (zaehlt) n++;
  return { datei: k.datei, titel: k.titel, nr: zaehlt ? n : null };
});
process.stdout.write(JSON.stringify({
  kapitel: nummeriert, verweise, hoechste: n, tips_ohne_rumpf, cmds_ohne_text,
}));
