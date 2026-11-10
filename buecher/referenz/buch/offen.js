// Gibt die noch NICHT uebersetzten Texte bestimmter Kapiteldateien aus.
//
//   node offen.js en 54_particles 55_physics ...
//
// Der Katalog (i18n/<sprache>.json) ist eine flache Liste in Dokument-
// reihenfolge -- er weiss nicht, aus welchem Kapitel ein Satz stammt.
// Ohne dieses Werkzeug liesse sich nicht gezielt "die Spiel-Module"
// uebersetzen, sondern nur stur von vorne nach hinten.
"use strict";
const fs = require("fs");
const path = require("path");
const { wrap } = require("./i18n");

const HERE = __dirname;
const LANG = process.argv[2] || "en";
const DATEIEN = process.argv.slice(3);
if (!DATEIEN.length) { console.error("Kapiteldateien angeben, z.B. 54_particles"); process.exit(2); }

const katalog = JSON.parse(fs.readFileSync(path.join(HERE, "i18n", `${LANG}.json`), "utf8"));
const nix = () => "";
const basis = {
  figure: nix, p: nix, pmix: nix, bullet: nix, bulletRich: nix,
  tip: nix, note: nix, warn: nix, code: nix, cmd: nix, table: nix,
  h1: nix, h2: nix, chapter: nix, part: nix, smallLabel: nix, sig: nix,
  PageBreak: null,
};
function flatten(a, acc) { for (const x of a) Array.isArray(x) ? flatten(x, acc) : acc.push(x); return acc; }

const contentDir = path.join(HERE, "content");
const alle = fs.readdirSync(contentDir).filter((f) => f.endsWith(".js"));
let n = 0;
for (const wunsch of DATEIEN) {
  const datei = alle.find((f) => f === wunsch || f === wunsch + ".js" || f.startsWith(wunsch));
  if (!datei) { console.error(`--- NICHT GEFUNDEN: ${wunsch}`); continue; }
  const H = wrap(basis, katalog);
  flatten(require(path.join(contentDir, datei))(H), []);
  for (const t of H.__fehlend) { console.log("---"); console.log(t); n++; }
}
console.error(`(${n} offen in ${DATEIEN.length} Dateien)`);
