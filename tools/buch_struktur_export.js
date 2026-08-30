// Struktur-Kennzahlen der Buch-Bausteine als JSON:
//   { tips_ohne_rumpf: [...], cmds_ohne_text: [...], cmds_ohne_sig: [...],
//     lange_h2: [...], kapitel: n }
//
// Die Hinweiskaesten haben eine Falle: `H.tip` nimmt (TITEL, text), `H.note`
// und `H.warn` dagegen (TEXT, titel). Wer sich vertut, uebergibt einen ganzen
// Absatz als Titel -- gesetzt wird das als 145 Zeichen Fettdruck in einem
// Kasten ohne Rumpf. Es stuerzt nichts ab, es sieht nur falsch aus, und beim
// Gegenlesen am 2026-08-30 steckten fuenf davon im Buch.
"use strict";
const fs = require("fs");
const path = require("path");

const CONTENT = path.join(__dirname, "..", "buch-referenz", "buch", "content");
const nix = () => "";
const tips_ohne_rumpf = [];
const cmds_ohne_text = [];
const cmds_ohne_sig = [];
const lange_h2 = [];
let kapitel = 0;
let datei = "";

const H = {
  figure: nix, code: nix, table: nix, smallLabel: nix, sig: nix,
  h1: nix, part: nix, PageBreak: null,
  p: nix, pmix: nix, bullet: nix, bulletRich: nix, note: nix, warn: nix,
  chapter: () => { kapitel++; return ""; },
  h2: (t) => {
    if (typeof t === "string" && t.length > 70) lange_h2.push({ datei, text: t });
    return "";
  },
  tip: (titel, text) => {
    if (text === undefined || String(text).trim() === "") {
      tips_ohne_rumpf.push({ datei, titel: String(titel).slice(0, 90) });
    }
    return "";
  },
  cmd: (name, sig, text) => {
    if (!text || String(text).trim() === "") cmds_ohne_text.push({ datei, name: String(name) });
    if (!sig || String(sig).trim() === "") cmds_ohne_sig.push({ datei, name: String(name) });
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
process.stdout.write(JSON.stringify({ tips_ohne_rumpf, cmds_ohne_text, cmds_ohne_sig, lange_h2, kapitel }));
