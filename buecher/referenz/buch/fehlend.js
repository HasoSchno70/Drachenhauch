// Meldet je Kapiteldatei, wie viele Texte noch nicht uebersetzt sind.
//
//   node fehlend.js [sprache] [teil-praefix]
//
// Anders als die Gesamtzahl am Ende eines Buchbaus zeigt das, WO die Luecken
// sitzen -- ohne das laesst sich "Teil II ist fertig" nicht belegen.
"use strict";
const fs = require("fs");
const path = require("path");
const { wrap } = require("./i18n");

const HERE = __dirname;
const LANG = process.argv[2] || "en";
const FILTER = process.argv[3] || null;
const katalog = JSON.parse(fs.readFileSync(path.join(HERE, "i18n", `${LANG}.json`), "utf8"));

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
let sumOffen = 0, sumDateien = 0;
const beispiele = [];
for (const m of mods) {
  let teilHier = null;
  const spion = { ...basis, part: (t) => { teilHier = t; return ""; } };
  const H = wrap(spion, katalog);
  flatten(require(path.join(contentDir, m))(H), []);
  if (teilHier) teil = teilHier;
  const passt = !FILTER || teil === FILTER
    || teil.startsWith(FILTER + " ") || teil.startsWith(FILTER + "—");
  if (!passt) continue;
  sumDateien++;
  const offen = H.__fehlend.size;
  sumOffen += offen;
  if (offen) {
    beispiele.push(...[...H.__fehlend].slice(0, 2));
    console.log(`  ${m.padEnd(28)} ${String(offen).padStart(4)} offen`);
  }
}
console.log(sumOffen === 0
  ? `  alle ${sumDateien} Dateien vollstaendig uebersetzt`
  : `  ${sumOffen} Texte offen in ${sumDateien} Dateien`);
for (const b of beispiele.slice(0, 3)) console.log(`    z.B. ${b.slice(0, 70)}`);
