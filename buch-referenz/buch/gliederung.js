// Zeigt die Gliederung des Buchs: welche Kapiteldatei zu welchem Teil
// gehoert, und wie viel uebersetzbarer Text darin steckt.
//
//   node gliederung.js            alle Teile
//   node gliederung.js "Teil I"   nur die Dateien eines Teils, mit Wortzahl
//
// Gebraucht wird das beim Uebersetzen: der Katalog (i18n/en.json) ist eine
// flache Liste in Dokumentreihenfolge und weiss selbst nicht, wo ein Teil
// anfaengt.
"use strict";
const fs = require("fs");
const path = require("path");
const { sammle } = require("./i18n");

const HERE = __dirname;
const FILTER = process.argv[2] || null;

const nix = () => "";
const basis = {
  figure: nix, p: nix, pmix: nix, bullet: nix, bulletRich: nix,
  tip: nix, note: nix, warn: nix, code: nix, cmd: nix, table: nix,
  h1: nix, h2: nix, chapter: nix, part: nix, smallLabel: nix, sig: nix,
  PageBreak: null,
};

const contentDir = path.join(HERE, "content");
const mods = fs.readdirSync(contentDir).filter((f) => f.endsWith(".js")).sort();
function flatten(a, acc) { for (const x of a) Array.isArray(x) ? flatten(x, acc) : acc.push(x); return acc; }

let teil = "(Vorspann)";
const zeilen = [];
for (const m of mods) {
  // Erst die Struktur mitschneiden ...
  let teilHier = null, kapitel = [];
  const spion = { ...basis,
    part: (t) => { teilHier = t; return ""; },
    chapter: (t) => { kapitel.push(t); return ""; } };
  const [H2, texte] = sammle(spion);
  flatten(require(path.join(contentDir, m))(H2), []);
  if (teilHier) teil = teilHier;
  const woerter = texte.reduce((n, t) => n + t.split(/\s+/).length, 0);
  zeilen.push({ datei: m, teil, kapitel, texte: texte.length, woerter });
}

let sumT = 0, sumW = 0;
for (const z of zeilen) {
  if (FILTER && !z.teil.startsWith(FILTER)) continue;
  sumT += z.texte; sumW += z.woerter;
  const k = z.kapitel.length ? z.kapitel.join(" / ") : "—";
  console.log(`${z.datei.padEnd(26)} ${String(z.texte).padStart(4)} Texte `
    + `${String(z.woerter).padStart(6)} W   ${z.teil.slice(0, 22).padEnd(24)} ${k}`);
}
console.log(`${"".padEnd(26)} ${String(sumT).padStart(4)} Texte ${String(sumW).padStart(6)} W`
  + (FILTER ? `   (nur ${FILTER})` : "   gesamt"));
