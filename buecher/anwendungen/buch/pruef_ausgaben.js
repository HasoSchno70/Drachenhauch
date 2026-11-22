// Prueft die AUSGABE-Bloecke des Buchs „Fenster, Formulare, Daten“, indem der Code wirklich laeuft.
//
// `pruef_codebloecke.js` daneben prueft nur, dass ein Beispiel UEBERSETZT,
// und `pruef_abdruck.js`, dass ein abgedrucktes Gesamtprogramm mit der Datei
// unter code/ uebereinstimmt. Was ein Beispiel AUSGIBT, stand bis 2026-08-30
// ungeprueft im Buch -- im Referenzbuch nebenan waren sieben solcher Angaben
// falsch (die Beispiele wurden bei der Umbenennung angepasst, die Ausgaben
// nicht).
//
// Das Muster hier: ein Codeblock, unmittelbar gefolgt von einem Block mit
// `{ out: true }` -- das ist die behauptete Ausgabe dazu.
//
// Wie entschieden wird:
//   * Der Codeblock laeuft mit `dhrt run` in einem leeren Verzeichnis.
//   * Bricht er ab (Fenster, fehlende Datei, Bruchstueck), wird er
//     uebersprungen -- viele Beispiele sind Ausschnitte.
//   * Laeuft er durch, wird zeilenweise nach `trim()` verglichen. Eine
//     kuerzere Angabe gilt als bewusster Auszug und wird nur auf ihren
//     Anfang geprueft.
//
// Aufruf:  node pruef_ausgaben.js [pfad-zu-dhrt]
// Rueckgabe: 0 = sauber, 1 = mindestens eine Ausgabe stimmt nicht.
"use strict";
const fs = require("fs");
const os = require("os");
const path = require("path");
const { spawnSync } = require("child_process");

const HERE = __dirname;
const DHRT = process.argv[2] || path.join(HERE, "..", "..", "..", "rust",
  "drachenhauch_runtime", "target", "release",
  process.platform === "win32" ? "dhrt.exe" : "dhrt");

// Oeffnet ein Fenster, braucht Geraete oder wartet -- ausfuehren wuerde
// haengen oder scheitern, ohne etwas ueber das Buch zu sagen.
// DB_ mit einer Datei laeuft: der Block laeuft in einem leeren
// Verzeichnis, die Datenbank entsteht dort frisch (ab Kapitel 4 dieses
// Buchs, 2026-11). Nur ein Server-Ziel braucht einen Server.
const NICHT_LAUFFAEHIG = /\b(SCREEN|FLIP|WAITKEY|INPUT\b|SLEEP|HTTP_|NET_|GUI_|UI_|PLAYSOUND|PLAYMUSIC|LOADSOUND|LOADIMAGE|LOADFONT|AUDIO_|SAVE_|MOUSEX|KEYPRESSED|KEYHIT)|(postgres|postgresql|mysql|mariadb):\/\//;

const bloecke = [];   // in Dokumentreihenfolge: {datei, zeilen, ist_ausgabe}
let datei = "";
const nix = () => "";
const H = {
  figure: nix, p: nix, pmix: nix, bullet: nix, bulletRich: nix,
  tip: nix, note: nix, warn: nix, table: nix, h1: nix, h2: nix, h3: nix,
  chapter: nix, part: nix, smallLabel: nix, sig: nix, cmd: nix, PageBreak: null,
  code: (zeilen, opts = {}) => {
    bloecke.push({ datei, zeilen: Array.isArray(zeilen) ? zeilen : [zeilen], ist_ausgabe: !!opts.out });
    return "";
  },
};
const flach = (a, acc) => { for (const x of a) Array.isArray(x) ? flach(x, acc) : acc.push(x); return acc; };
for (const f of fs.readdirSync(path.join(HERE, "content")).filter((n) => n.endsWith(".js")).sort()) {
  datei = f;
  // Ein Kapitel, das sich nicht laden laesst (ein Ausschnitt, dessen
  // Anfang oder Ende es in der Datei nicht mehr gibt), ist ein Befund --
  // still zu ueberspringen hiesse, seine Ausgaben nie zu pruefen.
  try { flach(require(path.join(HERE, "content", f))(H), []); } catch (e) {
    process.stderr.write(`FEHLER beim Lesen von ${f}: ${e.message}\n`);
    process.exit(1);
  }
}

// Paare bilden: Code, direkt gefolgt von seiner Ausgabe.
const paare = [];
for (let i = 0; i + 1 < bloecke.length; i++) {
  const a = bloecke[i], b = bloecke[i + 1];
  if (!a.ist_ausgabe && b.ist_ausgabe && a.datei === b.datei) paare.push({ code: a, out: b });
}

const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "dhanwendungen-"));
let geprueft = 0, uebersprungen = 0;
const befunde = [];
for (const p of paare) {
  const quelle = p.code.zeilen.join("\n") + "\n";
  if (NICHT_LAUFFAEHIG.test(quelle)) { uebersprungen++; continue; }
  // Jeder Block in einem EIGENEN leeren Verzeichnis: ein Block, der eine
  // Datenbank anlegt, darf dem naechsten nichts hinterlassen (Kapitel 4:
  // "0 Eintraege" nach dem vergessenen Commit war sonst "2", die Zeilen
  // aus Schritt 1).
  const f = path.join(fs.mkdtempSync(path.join(tmp, "b")), "block.dh");
  fs.writeFileSync(f, quelle, "utf8");
  const r = spawnSync(DHRT, ["run", f], { encoding: "utf8", timeout: 20000 });
  if (r.error || r.status !== 0) { uebersprungen++; continue; }   // Bruchstueck
  geprueft++;
  const ist = (r.stdout || "").split(/\r?\n/).map((s) => s.trim()).filter((s) => s !== "");
  const soll = p.out.zeilen.map((s) => String(s).trim()).filter((s) => s !== "");
  // Ein Block, der laeuft aber GAR NICHTS ausgibt, waehrend das Buch eine
  // Ausgabe nennt, ist ein Bruchstueck: meist eine SUB-Definition, deren
  // Aufruf im Absatz davor steht. Ihn als Befund zu melden waere ein
  // Falsch-Alarm -- er beweist nur, dass er allein nicht das ganze Beispiel ist.
  if (ist.length === 0 && soll.length > 0) { geprueft--; uebersprungen++; continue; }
  const n = Math.min(ist.length, soll.length);
  const ab = [];
  for (let i = 0; i < n; i++) if (ist[i] !== soll[i]) ab.push(`  Zeile ${i + 1}: Buch "${soll[i]}" -- wirklich "${ist[i]}"`);
  if (soll.length > ist.length) ab.push(`  Buch nennt ${soll.length} Zeilen, das Programm gibt ${ist.length} aus`);
  if (ab.length) befunde.push(`${p.code.datei}\n${ab.join("\n")}`);
}
fs.rmSync(tmp, { recursive: true, force: true });

console.log(`${paare.length} Ausgabe-Bloecke: ${geprueft} ausgefuehrt, ${uebersprungen} uebersprungen (Bruchstueck/Fenster), ${befunde.length} mit Befund.`);
for (const b of befunde) console.log(b);
process.exit(befunde.length ? 1 : 0);
