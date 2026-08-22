// Prueft JEDEN Codeblock des Lehrbuchs mit `dhrt --check`.
//
// Warum das noetig ist: die Kapitel sind Prosa mit eingebettetem Quelltext,
// und den liest kein Compiler -- ein Tippfehler im Buch faellt erst dem Leser
// auf, der ihn abtippt. Beim letzten Durchlauf steckten drei echte Fehler IM
// BUCH, keiner davon in einer Beispieldatei.
//
// Moeglich ist die Pruefung, weil content/NN_*.js die Bausteine nicht selbst
// importiert, sondern als `(H) => [bloecke]` geschrieben ist: hier kommt ein
// H herein, das nichts formatiert, sondern nur die Codezeilen einsammelt --
// dieselbe Quelle wie .docx und .epub, kein vierter Satz Kapiteltexte.
//
// Aufruf:  node pruef_codebloecke.js [pfad-zu-dhrt]
// Rueckgabe: 0 = sauber, 1 = mindestens ein Block hat einen Befund.
const fs = require("fs");
const os = require("os");
const path = require("path");
const { spawnSync } = require("child_process");

const HERE = __dirname;
const DHRT = process.argv[2] || path.join(HERE, "..", "..", "rust",
  "drachenhauch_runtime", "target", "release",
  process.platform === "win32" ? "dhrt.exe" : "dhrt");

// --------------------------------------------------------------- Einsammeln
const bloecke = [];      // {datei, nr, zeilen}
let aktuelleDatei = "";

// Hoechstlaenge einer Codezeile im Abdruck. Gemessen, nicht geschaetzt: Eine
// Zeile mit 81 Zeichen lief im gesetzten PDF aus dem grauen Kasten heraus, und
// das folgende NEXT rutschte dadurch an den Rand -- in einem Buch zum Abtippen
// ist das schlimmer als ein Schoenheitsfehler. 72 laesst Luft.
const MAX_SPALTEN = 72;
const zuLang = [];

function merke(lines, opts = {}) {
  // `out: true` ist eine AUSGABE, kein Quelltext -- die durch den Compiler zu
  // schicken wuerde nur Rauschen erzeugen.
  if (opts.out) return "";
  const arr = Array.isArray(lines) ? lines : [lines];
  arr.forEach((l) => { if (l.length > MAX_SPALTEN) zuLang.push([aktuelleDatei, l]); });
  if (arr.length) bloecke.push({ datei: aktuelleDatei, nr: bloecke.length, zeilen: arr });
  return "";
}

const nix = () => "";
const H = {
  figure: nix, p: nix, pmix: nix, bullet: nix, bulletRich: nix,
  tip: nix, note: nix, warn: nix, table: nix, h1: nix, h2: nix,
  chapter: nix, part: nix, smallLabel: nix, sig: nix, PageBreak: null,
  code: merke,
  // cmd(name, syntax, desc, codeLines, opts) -- das Beispiel steckt an 4.
  cmd: (_n, _s, _d, codeLines) => merke(codeLines || []),
};

const contentDir = path.join(HERE, "content");
const mods = fs.readdirSync(contentDir).filter((f) => f.endsWith(".js")).sort();
function flatten(a, acc) { for (const x of a) Array.isArray(x) ? flatten(x, acc) : acc.push(x); return acc; }

for (const m of mods) {
  aktuelleDatei = m;
  flatten(require(path.join(contentDir, m))(H), []);
}

// ----------------------------------------------------------------- Pruefen
if (!fs.existsSync(DHRT)) {
  console.error(`dhrt nicht gefunden: ${DHRT}\n(erst bauen: python rust/build_runtime.py)`);
  process.exit(2);
}
// Nicht jeder Block ist Drachenhauch: das Buch zeigt auch Kommandozeilen,
// JSON-Manifeste und Shader. Die durch den Compiler zu schicken meldet
// "Erwartet Zeilenende" -- richtig und voellig nutzlos.
const FREMD = [
  /^(python|py|node|npm|cargo|pip|git|dhrt|dh\w*|mosquitto\w*|arduino-cli)\b/,
  /^[$>#]/,                  // Eingabeaufforderung
  /^[{[]/,                   // JSON
  /^\/\//,                   // `// datei.dhanim` -- Drachenhauch kommentiert
                             // mit `'` oder REM, nie mit `//`
  /^(#version|precision|uniform|void\s+main)/,   // GLSL
  /^(listener|allow_anonymous)\b/,               // mosquitto.conf
];
function istFremd(zeilen) {
  const erste = zeilen.find((z) => z.trim() && !z.trim().startsWith("'"));
  return erste !== undefined && FREMD.some((r) => r.test(erste.trim()));
}

const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "buchcheck-"));
let befunde = 0, geprueft = 0, uebersprungen = 0;

for (const b of bloecke) {
  if (istFremd(b.zeilen)) { uebersprungen++; continue; }
  geprueft++;
  const datei = path.join(tmp, `b${b.nr}.dh`);
  fs.writeFileSync(datei, b.zeilen.join("\n") + "\n", "utf8");
  const r = spawnSync(DHRT, ["--check", datei], { encoding: "utf8" });
  const roh = (r.stdout || "").trim();
  if (roh === "[]" || roh === "") continue;
  let diag;
  try { diag = JSON.parse(roh); } catch { continue; }
  // NUR Lexer/Parser. Ein Buchbeispiel ist meist ein Ausschnitt: es ruft
  // eine Funktion aus einem frueheren Block auf und laesst dessen IMPORT
  // weg. Die `compile`-Befunde ("Unbekanntes Builtin", "fehlt IMPORT")
  // treffen damit fast immer das Fragment, nicht den Fehler -- 86 Befunde,
  // von denen keiner echt war. Was ein Leser wirklich abtippt und was
  // deshalb stimmen MUSS, ist die Syntax.
  diag = diag.filter((d) => d.phase === "parse" || d.phase === "lex");
  // ACHTUNG: Das Referenzbuch verwirft hier Befunde HINTER der letzten Zeile
  // -- dort meldet der Parser abgeschnittene Ausschnitte ("IF ... THEN" ohne
  // END IF). Fuer ein Nachschlagewerk voller Fragmente ist das richtig.
  //
  // Dieses Buch druckt VOLLSTAENDIGE Programme zum Abtippen ab, und genau
  // dort meldet der Parser das Fehlen: ein vergessenes WEND kommt als
  // "WEND erwartet, Programmende erreicht" in Zeile n+1. Mit der Regel des
  // Referenzbuchs rutschte im Kapitel 7 ein Programm ohne WEND durch --
  // gefunden beim Nachlesen, nicht vom Werkzeug. Also bleibt der Befund
  // stehen. Ein bewusst gekuerzter Ausschnitt muss dafuer vollstaendig
  // sein: lieber eine Zeile mehr abdrucken als eine Pruefung weniger.
  if (!diag.length) continue;
  befunde++;
  console.log(`\n--- ${b.datei}  Block ${b.nr}`);
  for (const d of diag.slice(0, 3)) {
    console.log(`    Zeile ${d.line}: ${d.message}`);
  }
  b.zeilen.slice(0, 12).forEach((z, i) => console.log(`    ${String(i + 1).padStart(3)} | ${z}`));
}

fs.rmSync(tmp, { recursive: true, force: true });
console.log(`\n${bloecke.length} Codebloecke gefunden: ${geprueft} geprueft, `
  + `${uebersprungen} uebersprungen (kein Drachenhauch), ${befunde} mit Befund.`);
if (zuLang.length) {
  console.log(`
${zuLang.length} Codezeile(n) laenger als ${MAX_SPALTEN} Zeichen --`
    + " sie laufen im gesetzten Buch aus dem Kasten:");
  for (const [d, l] of zuLang) console.log(`  ${String(l.length).padStart(3)}  ${d}  ${l.slice(0, 60)}`);
}
process.exit(befunde ? 1 : 0);
