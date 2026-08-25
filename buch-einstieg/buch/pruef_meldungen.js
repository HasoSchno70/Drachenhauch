// Prueft, ob jede im Buch WOERTLICH zitierte Fehlermeldung noch so lautet, wie
// die Laufzeit sie ausgibt.
//
// Warum das noetig ist: pruef_codebloecke.js prueft den abgedruckten CODE,
// pruef_abdruck.js die abgedruckten PROGRAMME -- die zitierten MELDUNGEN
// prueft niemand. Sie stehen im Fliesstext und in den Fehlerkuechen, und wenn
// jemand in der Laufzeit einen Wortlaut aendert, steht im gedruckten Buch
// weiter der alte. Genau das ist am 23.08.2026 passiert: RGB nimmt seither
// Kommazahlen an und rundet sie, und "RGB erwartet INTEGER, erhalten FLOAT"
// stand an fuenf Stellen im Buch.
//
// Wie geprueft wird: Zu jeder Meldungsfamilie steht unten ein winziges
// Programm, das sie AUSLOEST. Was dhrt dabei sagt, ist die Wahrheit -- eine
// von Hand gepflegte Liste erwarteter Wortlaute waere selbst wieder etwas,
// das veralten kann, ohne dass es jemand merkt.
//
// Verglichen wird nach dem Abschleifen der veraenderlichen Teile: Dateiname,
// Namen in Hochkommas, Tabellenname, Zahlen. Zahlen fallen dabei ganz weg,
// nicht nur auf ein Platzhalterzeichen: Das Buch schreibt einmal
// "Index 6 ausserhalb [0..5]" und einmal nur "Index ausserhalb" -- beides
// meint dieselbe Meldung, und die FORM ist es, die stimmen muss.
//
// Aufruf:  node pruef_meldungen.js
// Rueckgabe: 0 = alle Zitate gedeckt, 1 = mindestens eines weicht ab,
//            2 = nicht vollstaendig pruefbar (kein Netz)
const fs = require("fs");
const os = require("os");
const path = require("path");
const { spawnSync } = require("child_process");

const HIER = __dirname;
const DHRT = (() => {
  const eigen = path.join(HIER, "..", "..", "rust", "drachenhauch_runtime",
                          "target", "release",
                          process.platform === "win32" ? "dhrt.exe" : "dhrt");
  return fs.existsSync(eigen) ? eigen : "dhrt";
})();

// Jedes Programm loest genau eine Meldungsfamilie aus. `netz` markiert die
// eine, die ohne Internet nicht geht.
const PROVOKATIONEN = [
  { name: "Variable nicht deklariert", zeilen: ["PRINT mmx"] },
  { name: "Unbekanntes Builtin", zeilen: ["CIRLCE(1, 2, 3)"] },
  { name: "Parse-Fehler", zeilen: ["PRINT (1 + 2"] },
  { name: "Index ausserhalb", zeilen: ["DIM a[2] AS INTEGER", "PRINT STR$(a[2])"] },
  { name: "Kommazahl in ganzzahliges Fach",
    zeilen: ["DIM n AS INTEGER", "n = 7 / 2", "PRINT STR$(n)"] },
  { name: "Unbekannter Typ (gui)", zeilen: ["DIM w AS GUI_WINDOW"] },
  { name: "Unbekannter Typ (db)", zeilen: ["DIM c AS DB_CONN"] },
  { name: "RGB ausserhalb 0..255", zeilen: ["PRINT STR$(RGB(300, 0, 0))"] },
  { name: "falscher Argumenttyp",
    zeilen: ["DIM n AS INTEGER", "ARRAY_PUSH(n, 1)"] },
  { name: "falsche Argumentzahl bei Init",
    zeilen: ["CLASS P", "    DIM x AS INTEGER",
             "    SUB Init(a AS INTEGER, b AS INTEGER)", "        x = a + b",
             "    END SUB", "END CLASS", "DIM p AS P", "p = NEW P()"] },
  { name: "Feld gibt es nicht",
    zeilen: ["CLASS P", "    DIM x AS INTEGER", "END CLASS", "DIM p AS P",
             "p = NEW P()", "PRINT STR$(p.y)"] },
  { name: "no such table",
    zeilen: ['IMPORT "db"', "DIM c AS DB_CONN", "DIM r AS DB_RESULT",
             'c = DB_OPEN("pm_a.db")',
             'r = DB_QUERY(c, "SELECT * FROM gibtsnicht")'] },
  { name: "database is locked",
    zeilen: ['IMPORT "db"', "DIM a AS DB_CONN", "DIM b AS DB_CONN",
             'a = DB_OPEN("pm_b.db")',
             'DB_EXEC(a, "CREATE TABLE IF NOT EXISTS t (x INTEGER)")',
             'b = DB_OPEN("pm_b.db")', "DB_BEGIN(a)",
             'DB_EXEC(a, "INSERT INTO t VALUES (1)")',
             'DB_EXEC(b, "INSERT INTO t VALUES (2)")'] },
  { name: "HTTP-Fehlschlag", netz: true,
    zeilen: ['IMPORT "html"', "DIM s AS STRING", "HTTP_TIMEOUT(10)",
             's = HTTP_GET("https://raw.githubusercontent.com/' +
             'HasoSchno70/Drachenhauch/main/gibtsnicht.txt")'] },
];

// Woran ein Zitat als Meldung zu erkennen ist. Eine Heuristik -- ihre Aufgabe
// ist, die Stellen zu FINDEN. Beurteilt werden sie danach am gemessenen
// Wortlaut, nicht an dieser Liste.
const ANKER = new RegExp([
  "erwartet", "nicht deklariert", "ausserhalb", "Unbekannt", "passt nicht",
  "muessen 0\\.\\.255", "fehlt IMPORT", "Laufzeitfehler", "Parse-Fehler",
  "no such table", "Not Found", "nicht verfuegbar", "database is locked",
  "existiert nicht", "Argument\\(e\\)",
].join("|"));

// Alles abschleifen, was von Fall zu Fall anders lautet.
function tokens(s) {
  const glatt = s
    .replace(/\\"/g, '"')
    .replace(/\S+\.dh/g, "*.dh")                 // Datei des Ausloesers
    .replace(/'[^']*'/g, "'*'")                  // Variablen- und Typnamen
    .replace(/(no such table:\s*)\S+/g, "$1*")   // Tabellenname aus SQLite
    .replace(/-?[0-9]+(\.[0-9]+)?/g, " ")        // Zahlen fallen ganz weg
    .replace(/…/g, " ")                          // Auslassung im Buch
    .replace(/\s+/g, " ")
    .trim();
  return glatt.length ? glatt.split(" ") : [];
}

// Das Zitat muss als zusammenhaengende Wortfolge in der Meldung stehen.
function stecktIn(kurz, lang) {
  if (!kurz.length || kurz.length > lang.length) return false;
  for (let i = 0; i + kurz.length <= lang.length; i++) {
    if (kurz.every((w, k) => w === lang[i + k])) return true;
  }
  return false;
}

// ------------------------------------------------------------------ messen
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "dh-meldungen-"));
const gemessen = [];
let netzFehlt = false;
let kaputt = 0;

for (const p of PROVOKATIONEN) {
  const datei = path.join(tmp, "probe.dh");
  fs.writeFileSync(datei, p.zeilen.join("\n") + "\n", "utf8");
  const r = spawnSync(DHRT, ["run", datei], { encoding: "utf8", cwd: tmp });
  const zeilen = ((r.stdout || "") + (r.stderr || "")).split("\n")
    .map((z) => z.trim()).filter((z) => ANKER.test(z));
  if (!zeilen.length) {
    if (p.netz) {
      console.log(`  uebersprungen  ${p.name} (kein Netz?)`);
      netzFehlt = true;
    } else {
      console.log(`  KAPUTT         ${p.name}: loest keine Meldung mehr aus`);
      kaputt++;
    }
    continue;
  }
  for (const z of zeilen) gemessen.push({ von: p.name, roh: z, tok: tokens(z) });
}
fs.rmSync(tmp, { recursive: true, force: true });

// ---------------------------------------------------------- Zitate sammeln
// Ein Zitat ist, was das Buch AUCH ALS SOLCHES setzt: in Anfuehrung „...“, als
// mono-Zelle einer Fehlerkueche oder in einem Ausgabeblock. Die zweite Spalte
// einer Fehlerkueche ist die Erklaerung in eigenen Worten -- die darf jeden
// Wortlaut haben und wird hier nicht geprueft.
function istAusgabeblock(zeilen, index) {
  for (let i = index; i < Math.min(zeilen.length, index + 12); i++) {
    if (/out:\s*true/.test(zeilen[i])) return true;
    if (/^\s*\]\)/.test(zeilen[i])) return false;
  }
  return false;
}

const zitate = [];
const contentDir = path.join(HIER, "content");
for (const datei of fs.readdirSync(contentDir).filter((f) => f.endsWith(".js")).sort()) {
  const zeilen = fs.readFileSync(path.join(contentDir, datei), "utf8").split("\n");
  zeilen.forEach((zeile, i) => {
    if (!ANKER.test(zeile)) return;
    const roh = [];
    for (const m of zeile.matchAll(/„([^„“]{6,160})“/g)) roh.push(m[1]);
    for (const m of zeile.matchAll(
        /\{\s*text:\s*"((?:[^"\\]|\\.)+)"\s*,\s*mono:\s*true/g)) roh.push(m[1]);
    const nur = /^\s*['"](.*)['"],?\s*$/.exec(zeile);
    if (nur && istAusgabeblock(zeilen, i)) roh.push(nur[1]);
    for (const s of roh) {
      if (!ANKER.test(s)) continue;
      zitate.push({ datei, zeile: i + 1, text: s, tok: tokens(s) });
    }
  });
}

// ---------------------------------------------------------------- urteilen
const offen = zitate.filter((z) => !gemessen.some((g) => stecktIn(z.tok, g.tok)));

function gemeinsameWorte(a, b) {
  const s = new Set(a);
  return b.filter((w) => s.has(w)).length;
}

for (const z of offen) {
  console.log(`\n--- ${z.datei}:${z.zeile}`);
  console.log(`    Buch    : ${z.text}`);
  const nah = gemessen.map((g) => ({ g, n: gemeinsameWorte(g.tok, z.tok) }))
                      .sort((a, b) => b.n - a.n)[0];
  console.log(`    Gemessen: ${nah && nah.n ? nah.g.roh : "(nichts Aehnliches gemessen)"}`);
}

const einmalig = new Set(zitate.map((z) => z.tok.join(" "))).size;
console.log(`\n${zitate.length} Zitate (${einmalig} verschiedene) gegen ` +
            `${gemessen.length} gemessene Meldungen aus ` +
            `${PROVOKATIONEN.length} Programmen: ${offen.length} ungedeckt.`);
if (kaputt) console.log(`${kaputt} Provokation(en) loesen nichts mehr aus.`);
if (netzFehlt) console.log("Ohne Netz nicht vollstaendig pruefbar.");

process.exit(offen.length || kaputt ? 1 : (netzFehlt ? 2 : 0));
