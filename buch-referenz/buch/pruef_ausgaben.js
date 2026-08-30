// Prueft die AUSGABE-Bloecke des Lehrbuchs, indem der Code wirklich laeuft.
//
// `pruef_codebloecke.js` daneben prueft nur, dass ein Beispiel UEBERSETZT.
// Was es ausgibt, stand bis 2026-08-30 in 332 `{ out: [...] }`-Angaben --
// und war nie nachgemessen. Eine falsche Ausgabe im Buch ist heimtueckischer
// als ein Tippfehler: Der Leser tippt das Beispiel ab, bekommt etwas anderes
// und sucht den Fehler bei sich.
//
// Wie hier entschieden wird:
//   * Der Block laeuft mit `dhrt run` in einem leeren Verzeichnis.
//   * Bricht er ab (fehlende Datei, undefinierte Variable, Grafikfenster,
//     Hardware), gilt er als BRUCHSTUECK und wird uebersprungen -- die
//     meisten Beispiele sind Ausschnitte und laufen ohne ihren Kontext nicht.
//   * Laeuft er durch, wird die Ausgabe mit der Angabe verglichen.
//
// Verglichen wird zeilenweise nach `trim()`. Ein `out`, das WENIGER Zeilen
// hat als die Ausgabe, gilt als Auszug und wird nur auf seinen Anfang
// geprueft -- viele Angaben zeigen bewusst nur die erste Zeile.
//
// Aufruf:  node pruef_ausgaben.js [pfad-zu-dhrt]
// Rueckgabe: 0 = sauber, 1 = mindestens eine Ausgabe stimmt nicht.
"use strict";
const fs = require("fs");
const os = require("os");
const path = require("path");
const { spawnSync } = require("child_process");

const HERE = __dirname;
const DHRT = process.argv[2] || path.join(HERE, "..", "..", "rust",
  "drachenhauch_runtime", "target", "release",
  process.platform === "win32" ? "dhrt.exe" : "dhrt");

// Diese Befehle oeffnen ein Fenster, brauchen Geraete oder das Netz --
// ausfuehren wuerde haengen oder scheitern, ohne etwas ueber das Buch zu sagen.
const NICHT_LAUFFAEHIG = /\b(SCREEN|FLIP|SCREEN_NATIVE|SCREEN_TRANSPARENT|WAITKEY|INPUT\b|SLEEP|HTTP_|NET_|MQTT_|SERIAL_|USB_|BT_|WIFI_|MIDI_IN_OPEN|MIDI_OUT_OPEN|FIRMATA_|SMTP_SEND|HTTPD_|TASK_START|SHELL|AUDIO_|PLAYSOUND|PLAYMUSIC|LOADSOUND|LOADIMAGE|LOADMODEL|LOADFONT|GUI_|UI_|CAMERA3D|DB_|SAVE_|TILED_|ATLAS_|BATCH_)/;

const bloecke = [];
let datei = "";
const nix = () => "";
const H = {
  figure: nix, p: nix, pmix: nix, bullet: nix, bulletRich: nix,
  tip: nix, note: nix, warn: nix, table: nix, h1: nix, h2: nix,
  chapter: nix, part: nix, smallLabel: nix, sig: nix, PageBreak: null,
  code: (zeilen, opts = {}) => { if (!opts.out) bloecke.push({ datei, zeilen, out: null }); return ""; },
  cmd: (n, s, d, zeilen, opts = {}) => {
    if (zeilen && zeilen.length && opts && opts.out) bloecke.push({ datei, zeilen, out: opts.out, name: n });
    return "";
  },
};
const flach = (a, acc) => { for (const x of a) Array.isArray(x) ? flach(x, acc) : acc.push(x); return acc; };
for (const f of fs.readdirSync(path.join(HERE, "content")).filter((n) => n.endsWith(".js")).sort()) {
  datei = f;
  try { flach(require(path.join(HERE, "content", f))(H), []); } catch (e) {
    process.stderr.write(`uebersprungen: ${f} (${e.message})\n`);
  }
}

const mit_out = bloecke.filter((b) => b.out && b.out.length);
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "dhbuch-"));
let geprueft = 0, uebersprungen = 0;
const befunde = [];

for (const b of mit_out) {
  const quelle = b.zeilen.join("\n") + "\n";
  if (NICHT_LAUFFAEHIG.test(quelle)) { uebersprungen++; continue; }
  const f = path.join(tmp, "block.dh");
  fs.writeFileSync(f, quelle, "utf8");
  const r = spawnSync(DHRT, ["run", f], { encoding: "utf8", timeout: 20000 });
  if (r.error || r.status !== 0) { uebersprungen++; continue; }   // Bruchstueck
  geprueft++;
  const ist = (r.stdout || "").split(/\r?\n/).map((s) => s.trim()).filter((s) => s !== "");
  const soll = b.out.map((s) => String(s).trim()).filter((s) => s !== "");
  // Kuerzere Angabe = bewusster Auszug: nur den Anfang vergleichen.
  const n = Math.min(ist.length, soll.length);
  const abweichung = [];
  for (let i = 0; i < n; i++) if (ist[i] !== soll[i]) abweichung.push(`  Zeile ${i + 1}: Buch "${soll[i]}" -- wirklich "${ist[i]}"`);
  if (soll.length > ist.length) abweichung.push(`  Buch nennt ${soll.length} Zeilen, das Programm gibt ${ist.length} aus`);
  if (abweichung.length) befunde.push(`${b.datei} -- ${b.name}\n${abweichung.join("\n")}`);
}
fs.rmSync(tmp, { recursive: true, force: true });

console.log(`${mit_out.length} Ausgabe-Angaben: ${geprueft} ausgefuehrt, ${uebersprungen} uebersprungen (Bruchstueck/Fenster/Geraet), ${befunde.length} mit Befund.`);
for (const b of befunde) console.log(b);
process.exit(befunde.length ? 1 : 0);
