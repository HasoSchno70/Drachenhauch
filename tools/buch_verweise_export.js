// Liefert die Kapitel-Titel des Referenzbuchs und alle Verweise darauf:
//   { titel: [...], verweise: [{datei, ziel}, ...] }
//
// Das Buch nummeriert seine Kapitel NICHT -- die Ueberschrift ist nur der
// Titel. Ein Verweis muss den Titel nennen; eine Nummer zeigt auf etwas, das
// der Leser nirgends sieht (am 2026-08-30 taten das 20 Verweise, es waren die
// Dateinummern der Quelle). Und ein Titel-Verweis kann sich vertippen oder
// veralten, wenn ein Kapitel umbenannt wird.
"use strict";
const fs = require("fs");
const path = require("path");

const CONTENT = path.join(__dirname, "..", "buch-referenz", "buch", "content");
const nix = () => "";
const titel = [];
const verweise = [];
let datei = "";

// Sowohl die typografischen als auch die geraden Anfuehrungszeichen -- im
// Buch stehen beide Formen.
const MUSTER = /Kapitel [„"]([^“”"]+)[“”"]/g;
const sammle = (s) => {
  if (typeof s === "string") {
    for (const m of s.matchAll(MUSTER)) verweise.push({ datei, ziel: m[1] });
  }
  return "";
};

const H = {
  figure: nix, code: nix, table: nix, smallLabel: nix, sig: nix,
  h1: nix, part: nix, PageBreak: null,
  p: sammle, pmix: sammle, bullet: sammle, bulletRich: sammle, h2: sammle,
  tip: (a, b) => { sammle(a); sammle(b); return ""; },
  note: (a, b) => { sammle(a); sammle(b); return ""; },
  warn: (a, b) => { sammle(a); sammle(b); return ""; },
  cmd: (n, s, d) => { sammle(d); return ""; },
  chapter: (t) => { titel.push(t); return ""; },
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
process.stdout.write(JSON.stringify({ titel, verweise }));
