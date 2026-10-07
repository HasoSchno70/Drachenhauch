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
// --en: die Beschreibungen so, wie das englische Buch sie druckt -- ueber den
// Katalog i18n/en.json (Schluessel = deutscher Text). Was dort fehlt, faellt
// weg: ein deutscher Text im englischen Hover waere schlimmer als keiner.
const EN = process.argv.includes("--en");
const KATALOG = EN
  ? JSON.parse(fs.readFileSync(path.join(__dirname, "..", "buch-referenz", "buch", "i18n", "en.json"), "utf8"))
  : null;
const nix = () => "";
const treffer = [];
const H = {
  figure: nix, p: nix, pmix: nix, bullet: nix, bulletRich: nix,
  tip: nix, note: nix, warn: nix, code: nix, table: nix,
  h1: nix, h2: nix, chapter: nix, part: nix, smallLabel: nix, sig: nix,
  PageBreak: null,
  cmd: (name, signatur, beschreibung) => {
    let text = String(beschreibung || "");
    if (KATALOG) {
      if (!Object.prototype.hasOwnProperty.call(KATALOG, text)) return "";
      text = String(KATALOG[text]);
    }
    treffer.push([String(name || ""), text]);
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
