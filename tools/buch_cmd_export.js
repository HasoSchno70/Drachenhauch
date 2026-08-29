// Liefert die `H.cmd(...)`-Eintraege des Referenzbuchs als JSON auf stdout:
// [[name, beschreibung], ...] -- Rohdaten fuer tools/gen_builtin_prosa.py.
//
// Warum ueber Node und nicht per Regex aus Python: die Kapitel sind
// JavaScript-Module, ihre Texte enthalten Anfuehrungszeichen und Escapes
// ("square" klingt nach ...). Sie zu laden ist zuverlaessig, sie zu parsen
// waere Raten. Dasselbe Muster benutzen `fehlend.js` und `extract_strings.js`
// im Buch-Verzeichnis schon.
"use strict";
const fs = require("fs");
const path = require("path");

const CONTENT = path.join(__dirname, "..", "buch-referenz", "buch", "content");
const nix = () => "";
const treffer = [];
const H = {
  figure: nix, p: nix, pmix: nix, bullet: nix, bulletRich: nix,
  tip: nix, note: nix, warn: nix, code: nix, table: nix,
  h1: nix, h2: nix, chapter: nix, part: nix, smallLabel: nix, sig: nix,
  PageBreak: null,
  cmd: (name, signatur, beschreibung) => {
    treffer.push([String(name || ""), String(beschreibung || "")]);
    return "";
  },
};
function flach(a, acc) {
  for (const x of a) Array.isArray(x) ? flach(x, acc) : acc.push(x);
  return acc;
}
for (const datei of fs.readdirSync(CONTENT).filter((f) => f.endsWith(".js")).sort()) {
  try {
    flach(require(path.join(CONTENT, datei))(H), []);
  } catch (e) {
    process.stderr.write(`uebersprungen: ${datei} (${e.message})\n`);
  }
}
process.stdout.write(JSON.stringify(treffer));
