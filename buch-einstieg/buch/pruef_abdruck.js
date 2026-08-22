// Prueft, ob jedes abgedruckte GESAMTPROGRAMM Zeichen fuer Zeichen mit der
// Datei unter code/kapNN/ uebereinstimmt.
//
// Warum das noetig ist: pruef_codebloecke.js schickt jeden Block durch
// `dhrt --check` -- der Abdruck ist damit garantiert LAUFFAEHIG, aber nicht
// garantiert DASSELBE wie die Datei im Repo. Beim Schreiben von Kapitel 9
// wichen beide an 15 Stellen voneinander ab, darunter die Todespruefung der
// Schlange: in der Datei eine Zeile mit 84 Zeichen, im Buch zwei kuerzere.
// Beides lief, beides war richtig -- aber wer das Buch abtippt und danach in
// die mitgelieferte Datei sieht, findet ein anderes Programm vor und zweifelt
// an sich statt am Buch.
//
// Geprueft wird jeder Block ab MINDESTZEILEN. Er muss zu GENAU EINER Datei
// unter code/ passen. Verglichen wird ohne Leerzeilen und ohne reine
// Kommentarzeilen (der Dateikopf traegt oft eine Zeile mehr), aber MIT den
// angehaengten Kommentaren hinter Code.
//
// Aufruf:  node pruef_abdruck.js
// Rueckgabe: 0 = alle Abdrucke passen, 1 = mindestens einer weicht ab.
const fs = require("fs");
const path = require("path");

const HIER = __dirname;
const MINDESTZEILEN = 25;

// ---------------------------------------------------------- Bloecke einsammeln
const bloecke = [];
let aktuelleDatei = "";
function merke(lines, opts = {}) {
  if (opts.out) return "";
  const arr = Array.isArray(lines) ? lines : [lines];
  if (arr.length >= MINDESTZEILEN) bloecke.push({ datei: aktuelleDatei, zeilen: arr });
  return "";
}
const nix = () => "";
const H = new Proxy({ code: merke, cmd: (_n, _s, _d, c) => merke(c || []) },
  { get: (t, k) => (k in t ? t[k] : nix) });

const contentDir = path.join(HIER, "content");
for (const m of fs.readdirSync(contentDir).filter((f) => f.endsWith(".js")).sort()) {
  aktuelleDatei = m;
  const bloecke_ = require(path.join(contentDir, m))(H);
  void bloecke_;
}

// ------------------------------------------------------------ Dateien sammeln
const codeDir = path.join(HIER, "..", "code");
const dateien = [];
for (const kap of fs.readdirSync(codeDir)) {
  const d = path.join(codeDir, kap);
  if (!fs.statSync(d).isDirectory()) continue;
  for (const f of fs.readdirSync(d).filter((f) => f.endsWith(".dh"))) {
    dateien.push({ pfad: path.join(kap, f), inhalt: fs.readFileSync(path.join(d, f), "utf8") });
  }
}

// ------------------------------------------------------------------ Vergleich
const norm = (text) => (Array.isArray(text) ? text : text.replace(/\r/g, "").split("\n"))
  .map((z) => z.trimEnd())
  .filter((z) => z.trim() !== "" && !z.trim().startsWith("'"));

let abweichungen = 0;
for (const b of bloecke) {
  const A = norm(b.zeilen);
  let treffer = null, bester = null, besteZahl = Infinity;
  for (const d of dateien) {
    const B = norm(d.inhalt);
    let ungleich = 0;
    for (let n = 0; n < Math.max(A.length, B.length); n++) if (A[n] !== B[n]) ungleich++;
    if (ungleich === 0) { treffer = d; break; }
    if (ungleich < besteZahl) { besteZahl = ungleich; bester = { d, B }; }
  }
  if (treffer) {
    console.log(`  ok  ${b.datei}  =  code/${treffer.pfad.replace(/\\/g, "/")}`);
    continue;
  }
  abweichungen++;
  console.log(`\n--- ${b.datei}: kein Abdruck passt genau.`);
  console.log(`    Naechstdran: code/${bester.d.pfad.replace(/\\/g, "/")}  (${besteZahl} Abweichungen)`);
  const B = bester.B;
  let gezeigt = 0;
  for (let n = 0; n < Math.max(A.length, B.length) && gezeigt < 4; n++) {
    if (A[n] !== B[n]) {
      console.log(`    Zeile ${n + 1}\n      Buch : ${A[n] || "<fehlt>"}\n      Datei: ${B[n] || "<fehlt>"}`);
      gezeigt++;
    }
  }
}

console.log(`\n${bloecke.length} Gesamtprogramme geprueft, ${abweichungen} weichen ab.`);
process.exit(abweichungen ? 1 : 0);
