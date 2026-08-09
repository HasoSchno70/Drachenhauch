// Zieht alle uebersetzbaren Texte des Lehrbuchs in einen Katalog.
//
//   node extract_strings.js [katalog.json]
//
// Vorhandene Uebersetzungen bleiben erhalten -- die Datei wird gemischt,
// nicht ueberschrieben. Neue Texte kommen mit leerem Wert dazu, Texte die
// es nicht mehr gibt wandern nach `_veraltet`, statt geloescht zu werden
// (ein umformulierter Satz laesst sich so wiederverwenden).
"use strict";
const fs = require("fs");
const path = require("path");
const { sammle } = require("./i18n");

const HERE = __dirname;
const ZIEL = process.argv[2] || path.join(HERE, "i18n", "en.json");

// Ein H, das nichts baut -- wir wollen nur die Aufrufe sehen.
const nix = () => "";
const H = {
  figure: nix, p: nix, pmix: nix, bullet: nix, bulletRich: nix,
  tip: nix, note: nix, warn: nix, code: nix, cmd: nix, table: nix,
  h1: nix, h2: nix, chapter: nix, part: nix, smallLabel: nix, sig: nix,
  PageBreak: null,
};

const [H2, texte] = sammle(H);
const contentDir = path.join(HERE, "content");
const mods = fs.readdirSync(contentDir).filter((f) => f.endsWith(".js")).sort();
function flatten(a, acc) { for (const x of a) Array.isArray(x) ? flatten(x, acc) : acc.push(x); return acc; }
for (const m of mods) flatten(require(path.join(contentDir, m))(H2), []);

let alt = {};
if (fs.existsSync(ZIEL)) alt = JSON.parse(fs.readFileSync(ZIEL, "utf8"));
const veraltet = { ...(alt._veraltet || {}) };
delete alt._veraltet;

const neu = {};
let uebernommen = 0, frisch = 0;
for (const t of texte) {
  if (alt[t] !== undefined) { neu[t] = alt[t]; if (alt[t]) uebernommen++; }
  else if (veraltet[t]) { neu[t] = veraltet[t]; delete veraltet[t]; uebernommen++; }
  else { neu[t] = ""; frisch++; }
}
// Was im Katalog stand, aber nicht mehr vorkommt: aufheben statt wegwerfen.
for (const [k, v] of Object.entries(alt)) {
  if (neu[k] === undefined && v) veraltet[k] = v;
}
if (Object.keys(veraltet).length) neu._veraltet = veraltet;

fs.mkdirSync(path.dirname(ZIEL), { recursive: true });
fs.writeFileSync(ZIEL, JSON.stringify(neu, null, 1), "utf8");

const woerter = texte.reduce((n, t) => n + t.split(/\s+/).length, 0);
const zeichen = texte.reduce((n, t) => n + t.length, 0);
console.log(`${texte.length} verschiedene Texte, ${woerter} Woerter, ${Math.round(zeichen/1024)} KB`);
console.log(`  uebernommen: ${uebernommen}   neu: ${frisch}   ` +
            `veraltet aufgehoben: ${Object.keys(veraltet).length}`);
console.log(`-> ${path.relative(HERE, ZIEL)}`);
