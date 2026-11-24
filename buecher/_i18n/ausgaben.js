// Die Ausgaben von dhrt, die ein Buch abdruckt, in einer anderen Sprache --
// nicht uebersetzt, sondern von dhrt selbst geholt.
//
//   node ausgaben.js [sprache] [--pruefen] [--dhrt pfad]
//
// Ein Ausgabe-Block (`{ out: true }`) zeigt entweder, was ein PROGRAMM
// ausgibt -- das folgt dem Code und bleibt, wie der Code ist --, oder was
// DHRT sagt: eine Meldung ("Laufzeitfehler in x.dh:3: ...", "x.dh:2:
// Warnung: ...") oder die Ausgabe eines Werkzeugs (der Block davor ist eine
// Befehlszeile `dhrt --jit x.dh`). Fuer diese beiden laeuft das Programm
// hier zweimal, mit DHRT_LANG=de und mit der Zielsprache, und die Zeile, die
// im Buch steht, wird gegen die Zeile an derselben Stelle der anderen
// Ausgabe getauscht. Was dhrt in der Zielsprache sagt, steht so im Buch --
// keine Uebersetzung von Hand, die neben dem Katalog von dhrt herliefe.
//
// Ergebnis: i18n/ausgaben.<sprache>.json, { "deutsche Zeile": "Zeile" }; bei
// einer umbrochenen Meldung ist der Schluessel der ganze Text (die Zeilen mit
// Leerzeichen verbunden). Die Uebersetzungsschicht (i18n.js, wrap) setzt
// sie beim Bau ein und bricht neu um.
//
// --pruefen schreibt nichts, sondern meldet, wo die Datei nicht mehr zu dhrt
// passt (Rueckgabe 1). Ein Programm, das ohne Bildschirm nicht startet,
// wird dabei uebergangen; sein Eintrag bleibt, wie er ist.
"use strict";
const fs = require("fs");
const os = require("os");
const path = require("path");
const { spawnSync } = require("child_process");

const HERE = process.env.DH_BUCH || process.cwd();
const CODE = path.join(HERE, "..", "code");
const ARGS = process.argv.slice(2);
const PRUEFEN = ARGS.includes("--pruefen");
const di = ARGS.indexOf("--dhrt");
const DHRT = di >= 0 ? ARGS[di + 1] : path.join(HERE, "..", "..", "..", "rust",
  "drachenhauch_runtime", "target", "release", process.platform === "win32" ? "dhrt.exe" : "dhrt");
const SPRACHE = ARGS.find((a, i) => !a.startsWith("--") && ARGS[i - 1] !== "--dhrt") || "en";
const ZIEL = path.join(HERE, "i18n", `ausgaben.${SPRACHE}.json`);

const MELDUNG = /^(?:Laufzeitfehler in )?([\w.-]+\.dh):\d+: /;
// Pfade schreibt das Buch, wie Windows sie zeigt (demo_dist\demo.exe); auf
// macOS/Linux heisst dieselbe Zeile demo_dist/demo. Verglichen wird ohne
// diesen Unterschied.
const norm = (z) => z.replace(/\\/g, "/").replace(/\.exe\b/g, "");
const KEIN_FENSTER = /Kein Fenster moeglich/;

// Die Bloecke in Buchreihenfolge, je mit dem Block davor.
const bloecke = [];
let datei = "";
const H = new Proxy({
  code: (zeilen, opts = {}) => {
    bloecke.push({ datei, zeilen: (Array.isArray(zeilen) ? zeilen : [zeilen]).map(String), out: opts.out === true });
    return "";
  },
}, { get: (t, k) => (k in t ? t[k] : () => "") });
const flach = (a) => { for (const x of a) if (Array.isArray(x)) flach(x); };
const inhalt = path.join(HERE, "content");
for (const f of fs.readdirSync(inhalt).filter((n) => n.endsWith(".js")).sort()) {
  datei = f;
  flach(require(path.join(inhalt, f))(H));
}

function suchen(ordner, name) {
  for (const e of fs.readdirSync(ordner, { withFileTypes: true })) {
    const p = path.join(ordner, e.name);
    if (e.isDirectory()) { const t = suchen(p, name); if (t) return t; }
    else if (e.name === name) return p;
  }
  return null;
}

// Laeuft in einer Kopie des Ordners -- ein Werkzeug wie --export legt
// Dateien an, und die gehoeren nicht neben die Programme des Buchs.
function laufen(quelle, args, sprache) {
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "dhausgabe-"));
  try {
    fs.cpSync(path.dirname(quelle), tmp, { recursive: true });
    const env = { ...process.env, DHRT_LANG: sprache };
    const r = spawnSync(DHRT, args, { cwd: tmp, encoding: "utf8", env, timeout: 120000 });
    return ((r.stdout || "") + "\n" + (r.stderr || "")).replace(/\r/g, "").split("\n").map((z) => z.trim());
  } finally {
    fs.rmSync(tmp, { recursive: true, force: true });
  }
}

const neu = {};
const befunde = [];
let ohneFenster = 0, gelaufen = 0;
for (let i = 0; i < bloecke.length; i++) {
  const b = bloecke[i];
  if (!b.out) continue;
  const vor = i > 0 && !bloecke[i - 1].out && bloecke[i - 1].datei === b.datei ? bloecke[i - 1] : null;
  const m = MELDUNG.exec(b.zeilen[0].trim());
  let name, args, ganz;
  if (m) {
    name = m[1]; args = ["run", name]; ganz = true;
  } else if (vor && vor.zeilen.length === 1 && /^dhrt\s/.test(vor.zeilen[0].trim())) {
    args = vor.zeilen[0].trim().split(/\s+/).slice(1);
    name = args.find((a) => a.endsWith(".dh"));
    ganz = false;
    if (!name) continue;
  } else {
    continue;   // die Ausgabe eines Programms: sie folgt dem Code
  }
  // Zuerst im Ordner des Kapitels (13_mandelbrot.js -> code/kap13): ein
  // Name wie 3_messen.dh kommt in mehreren Kapiteln vor.
  const nr = (b.datei.match(/^(\d+)_/) || [])[1];
  const eigene = nr ? path.join(CODE, `kap${nr}`, name) : "";
  const quelle = eigene && fs.existsSync(eigene) ? eigene : suchen(CODE, name);
  if (!quelle) { befunde.push(`${b.datei}: ${name} liegt nicht unter code/`); continue; }
  const de = laufen(quelle, args, "de");
  if (de.some((z) => KEIN_FENSTER.test(z))) { ohneFenster++; continue; }
  const aus = laufen(quelle, args, SPRACHE);
  gelaufen++;
  if (de.length !== aus.length) {
    befunde.push(`${b.datei}: ${name} gibt deutsch ${de.length}, ${SPRACHE} ${aus.length} Zeilen aus`);
    continue;
  }
  const zeilen = ganz ? [b.zeilen.map((z) => z.trim()).join(" ")]
                      : b.zeilen.map((z) => z.trim()).filter((z) => z !== "");
  for (const z of zeilen) {
    const k = de.findIndex((x) => norm(x) === norm(z));
    if (k < 0) { befunde.push(`${b.datei}: "${z}" steht nicht in der Ausgabe von ${name}`); continue; }
    neu[z] = aus[k];
  }
}

const alt = fs.existsSync(ZIEL) ? JSON.parse(fs.readFileSync(ZIEL, "utf8")) : {};
// Was hier nicht laufen konnte (ohne Bildschirm), behaelt seinen Eintrag.
for (const [k, v] of Object.entries(alt)) if (!(k in neu) && ohneFenster) neu[k] = v;

if (PRUEFEN) {
  for (const [k, v] of Object.entries(neu)) {
    if (!(k in alt)) befunde.push(`fehlt in ${path.basename(ZIEL)}: ${k}`);
    else if (norm(alt[k]) !== norm(v)) befunde.push(`veraltet: ${k}\n    steht: ${alt[k]}\n    dhrt:  ${v}`);
  }
  for (const k of Object.keys(alt)) if (!(k in neu)) befunde.push(`ueberzaehlig: ${k}`);
} else if (!befunde.length) {
  fs.writeFileSync(ZIEL, JSON.stringify(neu, null, 1) + "\n", "utf8");
}
console.log(`${gelaufen} Programme gelaufen, ${Object.keys(neu).length} Zeilen`
  + (ohneFenster ? `, ${ohneFenster} ohne Fenster uebergangen` : "")
  + `, ${befunde.length} Befund(e)` + (PRUEFEN || befunde.length ? "." : ` -> ${path.relative(HERE, ZIEL)}`));
for (const b of befunde) console.log("  " + b);
process.exit(befunde.length ? 1 : 0);
