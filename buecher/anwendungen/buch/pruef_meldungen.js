// Prueft die FEHLERMELDUNGEN, die das Buch abdruckt, gegen dhrt.
//
// Unter "Was schiefgehen kann" steht ein Programm mit Fehler und darunter
// (als Block mit `{ out: true }`) die Meldung, die dhrt dazu ausgibt. Das
// Buch verspricht, dass sie wortwoertlich stimmt. pruef_ausgaben.js
// ueberspringt solche Bloecke (das Programm bricht ja ab); hier werden sie
// geprueft:
//
//   * Ein Ausgabe-Block, dessen erste Zeile wie eine Meldung von dhrt
//     anfaengt ("Laufzeitfehler in x.dh:4: ..." oder "x.dh:2: Compile-
//     Fehler: ..."), nennt seine Datei. Die liegt irgendwo unter ../code.
//   * Die Datei laeuft mit `dhrt run` in ihrem Ordner.
//   * Die Zeilen des Blocks, mit Leerzeichen verbunden, muessen eine Zeile
//     der Ausgabe sein -- der Abdruck bricht lange Meldungen nur an
//     Leerzeichen um.
//
// Aufruf:  node pruef_meldungen.js [pfad-zu-dhrt]
// Rueckgabe: 0 = sauber, 1 = mindestens eine Meldung stimmt nicht.
"use strict";
const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");

const HERE = __dirname;
const CODE = path.join(HERE, "..", "code");
const DHRT = process.argv[2] || path.join(HERE, "..", "..", "..", "rust",
  "drachenhauch_runtime", "target", "release",
  process.platform === "win32" ? "dhrt.exe" : "dhrt");

const bloecke = [];
let datei = "";
const H = new Proxy({
  code: (zeilen, opts = {}) => {
    if (opts.out) bloecke.push({ datei, zeilen: Array.isArray(zeilen) ? zeilen : [zeilen] });
    return "";
  },
}, { get: (t, k) => (k in t ? t[k] : () => "") });
const flach = (a, acc) => { for (const x of a) Array.isArray(x) ? flach(x, acc) : acc.push(x); return acc; };
for (const f of fs.readdirSync(path.join(HERE, "content")).filter((n) => n.endsWith(".js")).sort()) {
  datei = f;
  flach(require(path.join(HERE, "content", f))(H), []);
}

function suchen(ordner, name) {
  for (const e of fs.readdirSync(ordner, { withFileTypes: true })) {
    const p = path.join(ordner, e.name);
    if (e.isDirectory()) { const t = suchen(p, name); if (t) return t; }
    else if (e.name === name) return p;
  }
  return null;
}

const MELDUNG = /^(?:Laufzeitfehler in )?([\w.-]+\.dh):\d+: /;
let geprueft = 0, befunde = 0, ohneFenster = 0;
for (const b of bloecke) {
  const m = MELDUNG.exec(b.zeilen[0]);
  if (!m) continue;
  geprueft++;
  const soll = b.zeilen.map((z) => z.trim()).join(" ");
  const quelle = suchen(CODE, m[1]);
  if (!quelle) {
    befunde++;
    console.log(`--- ${b.datei}: ${m[1]} liegt nicht unter code/`);
    continue;
  }
  const r = spawnSync(DHRT, ["run", path.basename(quelle)], { cwd: path.dirname(quelle), encoding: "utf8" });
  const ist = ((r.stdout || "") + "\n" + (r.stderr || "")).replace(/\r/g, "").split("\n").map((z) => z.trim());
  // Ohne Bildschirm (CI) bricht ein Fensterprogramm schon bei SCREEN ab --
  // dann ist ueber seine Meldung nichts zu sagen.
  if (ist.some((z) => z.includes("Kein Fenster moeglich"))) { geprueft--; ohneFenster++; continue; }
  if (!ist.includes(soll)) {
    befunde++;
    console.log(`--- ${b.datei}: ${m[1]}\n    im Buch: ${soll}\n    dhrt:    ${ist.filter((z) => z.includes(m[1])).join(" | ") || "(keine Meldung zu der Datei)"}`);
  }
}
console.log(`${geprueft} Meldungen geprueft, ${befunde} mit Befund`
  + (ohneFenster ? `, ${ohneFenster} ohne Fenster uebersprungen.` : "."));
process.exit(befunde ? 1 : 0);
