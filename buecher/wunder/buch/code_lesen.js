// Liest Programme aus ../code/, damit das Buch sie nicht abschreibt.
//
// Ein abgeschriebenes Programm veraltet still: Wer die Datei verbessert,
// vergisst den Abdruck. Darum holen die Kapitel ihren Code hier -- ganz,
// als Ausschnitt oder (in den Loesungen) nur die Stellen zwischen
// `' >>> neu` und `' <<< neu`. Was im Buch steht, ist damit immer das, was
// die Pruefsammlung laufen laesst.
//
// Liegt bewusst NICHT in content/: der Buchbau laedt jede .js dort als
// Kapitel.
"use strict";
const fs = require("fs");
const path = require("path");

const CODE = path.join(__dirname, "..", "code");

function zeilen(rel) {
  return fs.readFileSync(path.join(CODE, rel), "utf8")
    .replace(/\r/g, "").replace(/\n+$/, "").split("\n");
}

// Ohne die Kopfkommentare (die stehen im Text schon als Prosa).
function ohneKopf(rel) {
  const z = zeilen(rel);
  let i = 0;
  while (i < z.length && (z[i].startsWith("'") || z[i].trim() === "")) i++;
  return z.slice(i);
}

// Von der ersten Zeile, die (ohne Einrueckung) mit `von` beginnt, bis
// einschliesslich der `n`-ten Zeile danach, die mit `bis` beginnt (fuer
// verschachtelte Schleifen: das zweite NEXT). Die gemeinsame Einrueckung
// faellt weg, damit ein Stueck aus einer Schleife links im Kasten steht.
function stueck(rel, von, bis, n = 1) {
  const z = zeilen(rel);
  const a = z.findIndex((x) => x.trim().startsWith(von));
  if (a < 0) throw new Error(`${rel}: Anfang nicht gefunden: ${von}`);
  let b = a;
  if (bis !== undefined) {
    let gefunden = 0;
    b = z.findIndex((x, i) => i > a && x.trim().startsWith(bis) && ++gefunden === n);
    if (b < 0) throw new Error(`${rel}: Ende nicht gefunden: ${bis} (${n}.)`);
  }
  return ausruecken(z.slice(a, b + 1));
}

function ausruecken(z) {
  const ein = Math.min(...z.filter((x) => x.trim()).map((x) => x.match(/^ */)[0].length));
  return z.map((x) => x.slice(ein));
}

// Die neuen Stellen einer Loesung, je Stelle ein Feld von Zeilen.
function neu(rel) {
  const z = zeilen(rel);
  const teile = [];
  let akt = null;
  for (const x of z) {
    if (x.trim() === "' >>> neu") { akt = []; continue; }
    if (x.trim() === "' <<< neu") { teile.push(ausruecken(akt)); akt = null; continue; }
    if (akt) akt.push(x);
  }
  if (!teile.length) throw new Error(`${rel}: keine Stelle zwischen >>> und <<<`);
  return teile;
}

module.exports = { zeilen, ohneKopf, stueck, neu };
