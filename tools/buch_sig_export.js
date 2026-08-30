// Liefert die `H.cmd(...)`-Eintraege des Referenzbuchs MIT Signatur als JSON:
// [{datei, name, sig, text}, ...] -- Grundlage fuer tests/test_buch_signaturen.py.
//
// Getrennt von buch_cmd_export.js, weil der nur (Name, Text) fuer die
// Hover-Prosa braucht und sein Format dort verankert ist.
"use strict";
const fs = require("fs");
const path = require("path");
const CONTENT = path.join(__dirname, "..", "buch-referenz", "buch", "content");
const nix = () => "";
const treffer = [];
let datei = "";
const H = {
  figure: nix, p: nix, pmix: nix, bullet: nix, bulletRich: nix,
  tip: nix, note: nix, warn: nix, code: nix, table: nix,
  h1: nix, h2: nix, chapter: nix, part: nix, smallLabel: nix, sig: nix,
  PageBreak: null,
  cmd: (name, signatur, text) => {
    treffer.push({ datei, name: String(name || ""), sig: String(signatur || ""), text: String(text || "") });
    return "";
  },
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
process.stdout.write(JSON.stringify(treffer));
