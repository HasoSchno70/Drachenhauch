// Prueft in allen vier Buechern UND in docs/, ob jede woertlich zitierte
// Fehlermeldung noch so lautet, wie die Laufzeit sie ausgibt.
//
// Warum das noetig ist: Die Buecher haben Pruefer fuer abgedruckten CODE
// (`dhrt --check`) und, im Einstiegsbuch, fuer abgedruckte PROGRAMME. Die
// zitierten MELDUNGEN prueft ohne dieses Skript niemand. Sie stehen im
// Fliesstext und in den Fehlerkuechen, und wenn jemand in der Laufzeit einen
// Wortlaut aendert, steht im gedruckten Buch weiter der alte. Genau das ist am
// 23.08.2026 passiert: RGB nimmt seither Kommazahlen an und rundet sie, und
// "RGB erwartet INTEGER, erhalten FLOAT" stand an fuenf Stellen im
// Einstiegsbuch.
//
// Abgrenzung zu tools/pruef_doku_aussagen.py: Der prueft `docs/` und sagt in
// seinem eigenen Kopf, Verhaltensaussagen liessen sich nicht mechanisch
// pruefen, die muesse man messen. Genau das tut dieses Skript -- es MISST,
// statt eine Erwartungsliste zu pflegen.
//
// Wie geprueft wird: Zu jeder Meldungsfamilie steht unten ein winziges
// Programm, das sie AUSLOEST. Was dhrt dabei sagt, ist die Wahrheit. Eine von
// Hand gepflegte Liste erwarteter Wortlaute waere selbst wieder etwas, das
// veralten kann, ohne dass es jemand merkt -- also dasselbe Problem noch mal.
//
// Verglichen wird nach dem Abschleifen der veraenderlichen Teile: Dateiname,
// Namen in Hochkommas, Tabellenname, Zahlen. Zahlen fallen ganz weg statt auf
// einen Platzhalter zu schrumpfen, denn ein Buch schreibt mal "Index 6
// ausserhalb [0..5]" und mal nur "Index ausserhalb" -- die FORM muss stimmen,
// nicht die Beispielzahl.
//
// WAS DIESER PRUEFER NICHT SIEHT -- nachgemessen, nicht vermutet:
// Er findet ein Zitat nur, wenn darin ein Ankerwort steht. Wer eine Meldung so
// umschreibt, dass KEIN Ankerwort uebrig bleibt ("Index 5 jenseits von
// [0..2]"), faellt aus der Pruefung heraus, ohne dass etwas rot wird -- die
// Gegenprobe dazu ist gelaufen und blieb still. Das ist tragbar, weil der
// realistische Fall der umgekehrte ist: Die Laufzeit aendert sich, der Text
// bleibt stehen -- und dann steht das alte Ankerwort noch da. Genau so wurde
// die RGB-Sache gefunden. Wer die Zahl der Zitate ploetzlich sinken sieht,
// sollte trotzdem hinsehen; sie steht deshalb in jeder Zeile der Ausgabe.
//
// Aufruf:  node tools/pruef_meldungen.js [buch-einstieg|docs ...]
// Rueckgabe: 0 = alle Zitate gedeckt, 1 = mindestens eines weicht ab,
//            2 = nicht vollstaendig pruefbar (kein Netz)
const fs = require("fs");
const os = require("os");
const path = require("path");
const { spawnSync } = require("child_process");

const WURZEL = path.join(__dirname, "..");
const DHRT = (() => {
  const eigen = path.join(WURZEL, "rust", "drachenhauch_runtime", "target",
                          "release",
                          process.platform === "win32" ? "dhrt.exe" : "dhrt");
  return fs.existsSync(eigen) ? eigen : "dhrt";
})();

// Jedes Programm loest genau eine Meldungsfamilie aus. `netz` markiert die
// eine, die ohne Internet nicht geht.
const PROVOKATIONEN = [
  { name: "Variable nicht deklariert", zeilen: ["PRINT mmx"] },
  { name: "Unbekanntes Builtin", zeilen: ["CIRLCE(1, 2, 3)"] },
  { name: "Parse-Fehler", zeilen: ["PRINT (1 + 2"] },
  { name: "Lexer-Fehler", zeilen: ['PRINT "offen'] },
  { name: "Index ausserhalb", zeilen: ["DIM a[2] AS INTEGER", "PRINT STR$(a[2])"] },
  { name: "Kommazahl in ganzzahliges Fach",
    zeilen: ["DIM n AS INTEGER", "n = 7 / 2", "PRINT STR$(n)"] },
  { name: "Division durch null",
    zeilen: ["DIM n AS INTEGER", "DIM m AS INTEGER", "m = 0",
             "n = 7 \\ m", "PRINT STR$(n)"] },
  // Der Builtin-NAME am Anfang wird NICHT abgeschliffen (nur Namen in Hoch-
  // kommas gelten als Daten) -- die Provokation muss also denselben Befehl
  // nehmen, den das Buch zitiert. Kapitel 10 des Einstiegsbuchs nimmt AUDIO_TONE.
  { name: "Modul-Builtin ohne IMPORT",
    zeilen: ["DIM piep AS SOUND", "piep = AUDIO_TONE(440, 100)"] },
  { name: "Unbekannter Typ (gui)", zeilen: ["DIM w AS GUI_WINDOW"] },
  { name: "Unbekannter Typ (db)", zeilen: ["DIM c AS DB_CONN"] },
  { name: "RGB ausserhalb 0..255", zeilen: ["PRINT STR$(RGB(300, 0, 0))"] },
  { name: "falscher Argumenttyp",
    zeilen: ["DIM n AS INTEGER", "ARRAY_PUSH(n, 1)"] },
  { name: "falsche Argumentzahl", zeilen: ["CLS(1, 2, 3, 4, 5)"] },
  { name: "falsche Argumentzahl bei Init",
    zeilen: ["CLASS P", "    DIM x AS INTEGER",
             "    SUB Init(a AS INTEGER, b AS INTEGER)", "        x = a + b",
             "    END SUB", "END CLASS", "DIM p AS P", "p = NEW P()"] },
  { name: "Feld gibt es nicht",
    zeilen: ["CLASS P", "    DIM x AS INTEGER", "END CLASS", "DIM p AS P",
             "p = NEW P()", "PRINT STR$(p.y)"] },
  { name: "Datei gibt es nicht",
    zeilen: ["DIM z AS ARRAY OF STRING", 'z = READLINES("gibtsnicht.txt")'] },
  { name: "Bild gibt es nicht",
    zeilen: ['SCREEN(320, 200, "x")', 'LOADIMAGE("gibtsnicht.png")'] },
  { name: "offener Block", zeilen: ["FOR i = 1 TO 3", "    PRINT i"] },
  { name: "Zuweisung an ein Tupel",
    zeilen: ["DIM t AS TUPLE", "t = (1, 2, 3)", "t[0] = 99"] },
  { name: "Methode gibt es nicht",
    zeilen: ["CLASS A", "    DIM x AS INTEGER", "END CLASS", "DIM a AS A",
             "a = NEW A()", "a.gibtsNicht()"] },
  { name: "falscher Handle-Typ", zeilen: ["PRINT STR$(ENDOFFILE(0))"] },
  { name: "JSON-Handle erwartet",
    zeilen: ['IMPORT "json"', "CLASS K", "    DIM x AS INTEGER", "END CLASS",
             "DIM k AS K", "k = NEW K()", "PRINT JSON_STRINGIFY(k)"] },
  { name: "fehlgeschlagene Pruefung",
    zeilen: ["ASSERT_COLLECT(TRUE)",
             'ASSERT_EQ(3600, 7200, "Sommerzeit")', "ASSERT_REPORT()"] },
  { name: "TUPLE statt ARRAY",
    zeilen: ['IMPORT "gui"', 'SCREEN(320, 200, "x")', "DIM f AS GUI_WINDOW",
             "DIM t AS GUI_WIDGET", 'f = GUI_WINDOW("t", 10, 10, 200, 100)',
             "t = GUI_TABLE(f, 10, 10, 100, 60)",
             'GUI_TABLE_HEADERS(t, ("a", "b"))'] },
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
  "muessen 0\\.\\.255", "fehlt IMPORT", "gehoert zum Modul", "Laufzeitfehler", "Parse-Fehler",
  "Lexer-Fehler", "Compile-Fehler", "no such table", "Not Found",
  "nicht verfuegbar", "database is locked", "existiert nicht",
  "Argument\\(e\\)", "Division durch",
].join("|"));

// Alles abschleifen, was von Fall zu Fall anders lautet.
function tokens(s) {
  const glatt = s
    .replace(/\\(["'])/g, "$1")                  // Escapes aus dem JS-Text
    // "Laufzeitfehler in datei.dh:7:" -> "Laufzeitfehler:". Die Fundstelle ist
    // Beiwerk, und die Doku kuerzt sie in Beispielen weg. Das WORT davor bleibt
    // stehen -- ein "Compile-Fehler", wo die Laufzeit "Laufzeitfehler" sagt,
    // faellt weiter auf.
    .replace(/\s+in\s+\S+\.dh:\d+:/g, ":")
    .replace(/\S+\.dh/g, "*.dh")                 // Datei des Ausloesers
    // Namen in Hochkommas und Anfuehrungszeichen sind Daten, kein Wortlaut:
    // aus `fehlt IMPORT "gui"?` und `fehlt IMPORT "vec2"?` wird dasselbe. Der
    // Preis ist, dass ein falscher Modulname hier nicht auffaellt -- das ist
    // aber ein Inhaltsfehler, kein Wortlautfehler, und dafuer gibt es die
    // Codepruefer.
    .replace(/'[^']*'/g, "'*'")
    .replace(/"[^"]*"/g, '"*"')
    .replace(/(no such table:\s*)\S+/g, "$1*")   // Tabellenname aus SQLite
    // "FEHL  Zeile 44: Sommerzeit: erhalten ..." -- die Beschriftung stammt
    // aus dem ASSERT des Benutzers und ist damit Daten wie der Tabellenname.
    .replace(/(FEHL\s+Zeile\s+\d+:\s*)[^:]+:/g, "$1*:")
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
function messen() {
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "dh-meldungen-"));
  const gemessen = [];
  let netzFehlt = false;
  let kaputt = 0;
  for (const p of PROVOKATIONEN) {
    const datei = path.join(tmp, "probe.dh");
    fs.writeFileSync(datei, p.zeilen.join("\n") + "\n", "utf8");
    // DHRT_FRAMES=1, damit die Provokationen mit SCREEN kein Fenster stehen
    // lassen, falls sie wider Erwarten doch bis zur Schleife kommen.
    const r = spawnSync(DHRT, ["run", datei], {
      encoding: "utf8", cwd: tmp,
      env: { ...process.env, DHRT_FRAMES: "1" },
    });
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
  return { gemessen, netzFehlt, kaputt };
}

// ---------------------------------------------------------- Zitate sammeln
// Ein Zitat ist, was ein Buch AUCH ALS SOLCHES setzt. Vier Schreibweisen, alle
// in Gebrauch:
//
//     „...“                          in Anfuehrung im Fliesstext
//     { text: "...", mono: true }     Zelle einer Tabelle
//     ["...", true]                   Schreibmaschinenteil in H.pmix
//     H.code([...], { out: true })    abgedruckte Ausgabe
//
// Die ersten beiden allein reichen nicht: Das Referenzbuch benutzt vor allem
// die dritte und vierte, und ein erster Entwurf dieses Skripts fand dort
// EINE einzige Meldung statt aller. Ein Pruefer, der stillschweigend nichts
// sieht, ist schlimmer als keiner.
//
// Die Erklaerspalte einer Fehlerkueche ist eigene Formulierung und bleibt
// draussen -- sonst meldet das Skript Saetze wie "Eine Zeile hatte nicht so
// viele Teile wie erwartet" als angeblich falschen Wortlaut.
//
// Wo der Text liegt, ist von Buch zu Buch verschieden: Einstieg und Referenz
// haben content/NN_*.js, Galaga und Tippspiel tragen ihren Text in
// build_book.js. i18n/ bleibt draussen -- das ist eine Uebersetzungstabelle,
// deren Schluessel den deutschen Text nur doppeln.
function quellen(buchDir) {
  const b = path.join(buchDir, "buch");
  const c = path.join(b, "content");
  if (fs.existsSync(c) && fs.statSync(c).isDirectory()) {
    return fs.readdirSync(c).filter((f) => f.endsWith(".js")).sort()
      .map((f) => path.join(c, f));
  }
  const einzeln = path.join(b, "build_book.js");
  return fs.existsSync(einzeln) ? [einzeln] : [];
}

// Absichtlich KEINE aktuellen Wortlaute -- hier steht, was ein Text als
// Vorschlag oder als Schema hinschreibt. Beides laesst sich nicht ausloesen,
// weil es die Meldung so (noch) nicht gibt. Eine kurze, benannte Ausnahmeliste
// ist ehrlicher als eine Heuristik, die eine ganze Sorte stillschweigend
// verschluckt.
const KEINE_MELDUNG = [
  // docs/allzweck-roadmap.md: Schema mit Platzhalter, kein echter Wortlaut.
  '"NAME: erwartet …"',
  // docs/stolpersteine.md: ein VORSCHLAG mit Gedankenstrich statt "--".
  // Der geltende Wortlaut steht dreizehn Zeilen darueber und wird geprueft.
  'Unbekannter Typ \'vec2\' — fehlt IMPORT "vec2"?',
];

// Steht diese Zeile in einem H.code([...], { out: true })-Block?
function istAusgabeblock(zeilen, index) {
  for (let i = index; i < Math.min(zeilen.length, index + 20); i++) {
    if (/out:\s*true/.test(zeilen[i])) return true;
    if (/^\s*\]\s*\)/.test(zeilen[i])) return false;
  }
  return false;
}

// In `docs/` steht der Text als Markdown. Zitiert wird dort in Backticks oder
// in Anfuehrung. Zwei Sorten Fehltreffer muessen raus:
//
//   `u32` passt nicht in `I32`   -- zwischen ZWEI Code-Abschnitten steht
//                                  Fliesstext, den ein naiver Ausdruck als
//                                  einen dritten liest. Erkennbar daran, dass
//                                  er mit Leerzeichen anfaengt oder aufhoert.
//   assert run_gb(src) == erwartet -- Quelltext, keine Meldung.
// Dazu kommen abgedruckte AUSGABEN in Codebloecken. Bloecke mit ```basic sind
// Quelltext -- die prueft tools/pruef_docs.py, und dort ist "erwartet" auch mal
// ein Variablenname. Alle anderen Bloecke zeigen, was ein Werkzeug ausgibt.
function zitateAusMarkdown(datei) {
  const zeilen = fs.readFileSync(datei, "utf8").split("\n");
  const raus = [];
  let imBlock = false;
  let sprache = "";
  zeilen.forEach((zeile, i) => {
    const zaun = /^\s*```(.*)$/.exec(zeile);
    if (zaun) {
      if (imBlock) { imBlock = false; sprache = ""; }
      else { imBlock = true; sprache = zaun[1].trim().toLowerCase(); }
      return;
    }
    if (!ANKER.test(zeile)) return;
    const roh = [];
    if (imBlock) {
      if (sprache === "basic") return;
      // Zwei Randnotizen, die der Text den Ausgabezeilen anhaengt und die
      // nicht zur Meldung gehoeren: ein "-> " davor und ein "   <- stderr"
      // dahinter.
      roh.push(zeile.trim()
        .replace(/^(->|→)\s*/, "")
        .replace(/\s{2,}<-\s.*$/, "")
        .trim());
    }
    for (const m of zeile.matchAll(/`([^`\n]{6,160})`/g)) roh.push(m[1]);
    for (const m of zeile.matchAll(/„([^„“\n]{6,160})“/g)) roh.push(m[1]);
    for (const s of roh) {
      if (!ANKER.test(s)) continue;
      if (s !== s.trim()) continue;              // Fliesstext zwischen Spans
      if (/==|\bassert\b/.test(s)) continue;     // Quelltext
      if (KEINE_MELDUNG.includes(s)) continue;
      raus.push({ datei: path.basename(datei), zeile: i + 1, text: s, tok: tokens(s) });
    }
  });
  return raus;
}

function zitateAus(datei) {
  if (datei.endsWith(".md")) return zitateAusMarkdown(datei);
  const zeilen = fs.readFileSync(datei, "utf8").split("\n");
  const raus = [];
  zeilen.forEach((zeile, i) => {
    if (!ANKER.test(zeile)) return;
    const roh = [];
    for (const m of zeile.matchAll(/„([^„“]{6,160})“/g)) roh.push(m[1]);
    for (const m of zeile.matchAll(
        /\{\s*text:\s*"((?:[^"\\]|\\.)+)"\s*,\s*mono:\s*true/g)) roh.push(m[1]);
    for (const m of zeile.matchAll(
        /\[\s*"((?:[^"\\]|\\.)+)"\s*,\s*true\s*\]/g)) roh.push(m[1]);
    for (const m of zeile.matchAll(
        /\[\s*'((?:[^'\\]|\\.)+)'\s*,\s*true\s*\]/g)) roh.push(m[1]);
    const nur = /^\s*(["'])((?:(?!\1)[^\\]|\\.)*)\1\s*,?\s*$/.exec(zeile);
    if (nur && istAusgabeblock(zeilen, i)) roh.push(nur[2]);
    // Einzeiliger Ausgabeblock: H.code(["..."], { out: true })
    if (/out:\s*true/.test(zeile)) {
      for (const m of zeile.matchAll(
          /(["'])((?:(?!\1)[^\\]|\\.)*)\1/g)) roh.push(m[2]);
    }
    for (const s of roh) {
      if (!ANKER.test(s)) continue;
      if (KEINE_MELDUNG.includes(s)) continue;
      raus.push({ datei: path.basename(datei), zeile: i + 1, text: s, tok: tokens(s) });
    }
  });
  return raus;
}

// ---------------------------------------------------------------- Hauptlauf
function gemeinsameWorte(a, b) {
  const s = new Set(a);
  return b.filter((w) => s.has(w)).length;
}

// Geprueft werden die vier Buecher und `docs/`. Die Doku zitiert dieselben
// Meldungen und veraltet genauso -- tools/pruef_doku_aussagen.py sagt in
// seinem eigenen Kopf, dass ihm genau diese Sorte Aussage entgeht.
const gewuenscht = process.argv.slice(2);
const buecher = fs.readdirSync(WURZEL)
  .filter((d) => /^buch-/.test(d) && fs.statSync(path.join(WURZEL, d)).isDirectory())
  .concat(fs.existsSync(path.join(WURZEL, "docs")) ? ["docs"] : [])
  .filter((d) => !gewuenscht.length || gewuenscht.includes(d))
  .sort();

const { gemessen, netzFehlt, kaputt } = messen();
console.log(`${gemessen.length} Meldungen aus ${PROVOKATIONEN.length} Programmen gemessen.\n`);

let offenGesamt = 0;
let zitateGesamt = 0;
for (const buch of buecher) {
  const dateien = buch === "docs"
    ? fs.readdirSync(path.join(WURZEL, "docs")).filter((f) => f.endsWith(".md"))
        .sort().map((f) => path.join(WURZEL, "docs", f))
    : quellen(path.join(WURZEL, buch));
  if (!dateien.length) { console.log(`${buch}: keine Textquelle gefunden`); continue; }
  const zitate = dateien.flatMap(zitateAus);
  const offen = zitate.filter((z) => !gemessen.some((g) => stecktIn(z.tok, g.tok)));
  zitateGesamt += zitate.length;
  offenGesamt += offen.length;
  const verschieden = new Set(zitate.map((z) => z.tok.join(" "))).size;
  console.log(`${buch}: ${zitate.length} Zitate (${verschieden} verschiedene) ` +
              `aus ${dateien.length} Datei(en), ${offen.length} ungedeckt.`);
  for (const z of offen) {
    console.log(`\n  --- ${z.datei}:${z.zeile}`);
    console.log(`      Buch    : ${z.text}`);
    const nah = gemessen.map((g) => ({ g, n: gemeinsameWorte(g.tok, z.tok) }))
                        .sort((a, b) => b.n - a.n)[0];
    console.log(`      Gemessen: ${nah && nah.n ? nah.g.roh : "(nichts Aehnliches gemessen)"}`);
  }
  if (offen.length) console.log("");
}

console.log(`\n${zitateGesamt} Zitate in ${buecher.length} Quellen: ` +
            `${offenGesamt} ungedeckt.`);
if (kaputt) console.log(`${kaputt} Provokation(en) loesen nichts mehr aus.`);
if (netzFehlt) console.log("Ohne Netz nicht vollstaendig pruefbar.");

process.exit(offenGesamt || kaputt ? 1 : (netzFehlt ? 2 : 0));
