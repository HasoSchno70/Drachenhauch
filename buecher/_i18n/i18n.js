// Uebersetzungsschicht fuer die Buecher (zuerst das Lehrbuch, seit 2026-11
// auch "Kleine Wunder"; je Buch rufen kleine Dateien in buch/ sie auf).
//
// Der Kniff ist derselbe, mit dem schon .docx und .epub aus EINER Quelle
// entstehen: content/NN_*.js ist als `(H) => [bloecke]` geschrieben und
// importiert nichts selbst. Hier wird ein H hereingereicht, das jede
// Zeichenkette vorher durch einen Katalog schickt.
//
// Damit braucht KEINES der 75 Kapitel angefasst zu werden -- ein zweiter,
// englischer Satz Kapiteldateien waere binnen eines Monats vom deutschen
// abgedriftet, genau wie es ein dritter Renderer getan haette.
//
//   wrap(H, katalog, ausgaben) -> H', das uebersetzt
//   sammle(H)          -> [H', strings] zum Einsammeln aller Texte
//
// Fehlt ein Eintrag, bleibt der deutsche Text stehen. Das Buch baut also
// auch bei halb gefuelltem Katalog, nur eben teilweise deutsch -- besser
// als ein Abbruch mitten im Satz.
"use strict";

const { vorlagen, fuellen } = require("./vorlage");

// Welche Argumente eines H-Aufrufs sind Text? Der Rest (Bilddateinamen,
// Zahlen, Optionen) darf NICHT durch den Katalog.
//   true  = uebersetzen
//   false = unveraendert lassen
const FELDER = {
  p:           [true],
  h1:          [true],
  h2:          [true],
  h3:          [true],
  chapter:     [true],
  part:        [true],
  bullet:      [true],
  bulletRich:  [true, true],
  smallLabel:  [true],
  sig:         [true],
  // figure(datei, bildunterschrift) -- der Dateiname bleibt
  figure:      [false, true],
  // tip(titel, text), note(text, titel), warn(text, titel)
  tip:         [true, true],
  note:        [true, true],
  warn:        [true, true],
  // code(zeilen, opts) -- Quelltext bleibt, siehe unten
  code:        [false, false],
  // cmd(name, syntax, beschreibung, codezeilen, opts)
  //   Name und Syntax sind Sprache, keine Prosa.
  cmd:         [false, false, true, false, false],
  // table(zeilen, opts) -- eigene Behandlung, siehe uebersetzeTabelle
  table:       [false, false],
  pmix:        [false],          // Mischform, eigene Behandlung
};

// Was ueberhaupt Prosa ist. Tabellen enthalten neben Saetzen auch Daten:
// Tastencodes (`1073741904`), Befehlsnamen (`PRINT`), Signaturen
// (`BOX(x1, y1, x2, y2)`), Farbwerte (`&HFF8800`). Die durch einen
// Uebersetzungskatalog zu schicken ist im besten Fall sinnlos und im
// schlechtesten zerstoerend -- ein uebersetztes `PRINT` waere ein Fehler
// im Buch, den kein Leser sich erklaeren koennte.
const NICHT_PROSA = [
  /^[\s\d.,;:+\-*/%()[\]{}<>=!|&^~"'`_#$?]*$/,   // nur Zeichen ohne Buchstaben
  /^&[Hh][0-9A-Fa-f]+$/,                          // Hex-Literal
  /^[A-Z][A-Z0-9_$]*$/,                           // BEFEHL / KONSTANTE
  // BEFEHL(...) und BEFEHL(...) AS TYP bzw. -> TYP.
  //
  // Das ist der Befehls-Index (Anhang A): 202 von 229 Eintraegen sind reine
  // Signaturen. Sie zu uebersetzen waere nicht nur Arbeit ohne Nutzen,
  // sondern FALSCH -- in den Kapiteln bleibt die Signatur unangetastet
  // (siehe FELDER: cmd laesst Name und Syntax durch), ein uebersetzter
  // Index widerspraeche also dem Kapitel, auf das er verweist.
  //
  // Die Klammer darf KEINE weitere `)` enthalten. Mit `.*` griff der Ausdruck
  // ueber ganze Absaetze hinweg: "KEYPRESSED(code) und die input-Befehle …
  // (Mehr dazu in Kapitel 45.)" sah wie eine Signatur aus, weil `.*` bis zur
  // LETZTEN Klammer lief -- der Absatz blieb dann stumm deutsch.
  /^[A-Z][A-Z0-9_$]*\s*\([^)]*\)(\s*(AS|->)\s*\S.*)?$/,
  /^— [A-Z] —$/,                                  // Index-Buchstabentrenner
  /^[a-z_][a-z0-9_]*$/,                           // bezeichner_ohne_leerzeichen
  /^\.[a-z]+$/,                                   // .endung
];
function istProsa(s) {
  const t = s.trim();
  if (t.length < 2) return false;
  return !NICHT_PROSA.some((r) => r.test(t));
}

const istText = (x) => typeof x === "string" && istProsa(x);

function uebersetzeWert(x, tr) {
  if (istText(x)) return tr(x);
  if (Array.isArray(x)) return x.map((e) => uebersetzeWert(e, tr));
  return x;
}

// Tabellen: Zellen sind Strings ODER {text, mono, bold, color, swatch}.
// Uebersetzt wird nur `text` bzw. der nackte String -- eine Farbe wie
// "FF8800" durch den Katalog zu schicken waere bestenfalls sinnlos.
function uebersetzeTabelle(zeilen, tr) {
  if (!Array.isArray(zeilen)) return zeilen;
  return zeilen.map((zeile) => Array.isArray(zeile)
    ? zeile.map((zelle) => {
        if (istText(zelle)) return tr(zelle);
        // `roh`: steht schon in der Zielsprache (Anhang D holt die
        // Meldungen von dhrt selbst) -- nicht noch einmal durch den Katalog.
        if (zelle && typeof zelle === "object" && zelle.roh) return zelle;
        if (zelle && typeof zelle === "object" && istText(zelle.text)) {
          return { ...zelle, text: tr(zelle.text) };
        }
        return zelle;
      })
    : zeile);
}

// pmix(["text", ["code"], "text"]) -- Array-Eintraege sind Inline-Code
// und bleiben; die nackten Strings sind Prosa. ["wort", false] ist ein
// betontes Wort im Fliesstext ("Kleine Wunder") und wird uebersetzt.
// In pmix steht Code IMMER als ["code", true]; ein nackter Text und eine
// Betonung sind darum immer Prosa -- auch " und " zwischen zwei Codestuecken
// oder ein einzelnes Wort, das ausserhalb von pmix als Bezeichner gaelte.
const hatBuchstaben = (x) => typeof x === "string" && /\p{L}/u.test(x);
function uebersetzePmix(teile, tr) {
  if (!Array.isArray(teile)) return teile;
  return teile.map((x) => {
    if (Array.isArray(x) && x[1] === false && hatBuchstaben(x[0])) return [tr(x[0]), false];
    return hatBuchstaben(x) ? tr(x) : x;
  });
}

// Kommentare in Beispielprogrammen. Der Code selbst MUSS unangetastet
// bleiben -- er wird vom Compiler gelesen -, aber `' so geht das` ist
// Prosa und im Buch genauso wichtig wie der Fliesstext.
const KOMMENTAR = /^(\s*)'(.*)$/;
const KOMMENTAR_HINTEN = /^(.*?\S)\s{2,}'(.*)$/;

function uebersetzeCode(zeilen, tr) {
  const arr = Array.isArray(zeilen) ? zeilen : [zeilen];
  return arr.map((z) => {
    if (typeof z !== "string") return z;
    let m = z.match(KOMMENTAR);
    if (m && m[2].trim()) return `${m[1]}'${tr2(m[2], tr)}`;
    m = z.match(KOMMENTAR_HINTEN);
    if (m && m[2].trim()) return `${m[1]}  '${tr2(m[2], tr)}`;
    return z;
  });
}
// Kommentartexte werden ohne fuehrendes Leerzeichen katalogisiert, damit
// derselbe Satz nicht zweimal drinsteht, nur wegen der Einrueckung.
function tr2(roh, tr) {
  const vorn = roh.match(/^\s*/)[0];
  const rest = roh.slice(vorn.length);
  return rest ? vorn + tr(rest) : roh;
}

// Ausgabe-Bloecke (`{ out: true }`). Was ein Programm ausgibt, folgt dem
// Code und bleibt. Was DHRT sagt -- Meldungen, Werkzeug-Ausgaben --, steht
// in `ausgaben` (i18n/ausgaben.<sprache>.json, von ausgaben.js bei dhrt
// selbst geholt): eine umbrochene Meldung unter ihrem ganzen Text, sonst
// Zeile fuer Zeile. Die Meldung wird in der Breite der deutschen Zeilen neu
// umbrochen. `{ out: true, uebersetzen: true }` ist ein Block, den das Buch
// selbst schreibt (eine Dateiliste): hinter dem Namen und mindestens zwei
// Leerzeichen steht ein Text, der durch den Katalog geht.
function umbrechen(text, breite) {
  const zeilen = [];
  let z = "";
  for (const w of text.split(" ")) {
    if (z && (z + " " + w).length > breite) { zeilen.push(z); z = w; }
    else z = z ? z + " " + w : w;
  }
  if (z) zeilen.push(z);
  return zeilen;
}
function uebersetzeAusgabe(zeilen, opts, ausgaben, tr) {
  const arr = (Array.isArray(zeilen) ? zeilen : [zeilen]).map(String);
  const ganz = arr.map((z) => z.trim()).filter((z) => z !== "").join(" ");
  if (arr.length > 1 && ausgaben[ganz] !== undefined) {
    const breite = Math.max(...arr.map((z) => z.trimEnd().length));
    return umbrechen(ausgaben[ganz], breite);
  }
  return arr.map((z) => {
    const vorn = z.match(/^\s*/)[0];
    const t = z.trim();
    if (ausgaben[t] !== undefined) return vorn + ausgaben[t];
    const m = opts.uebersetzen && z.match(/^(\s*\S+\s{2,})(.*\S)\s*$/);
    if (m && istProsa(m[2])) return m[1] + tr(m[2]);
    return z;
  });
}

function baue(H, tr, ausgaben = {}) {
  const neu = {};
  for (const [name, fn] of Object.entries(H)) {
    if (typeof fn !== "function") { neu[name] = fn; continue; }
    const spec = FELDER[name];
    neu[name] = (...args) => {
      if (name === "table") {
        // Die Kopfzeile steckt in den Optionen (`{headers: [...]}`), nicht in
        // den Zeilen -- ohne diesen Zweig blieb ueber jeder uebersetzten
        // Tabelle "Konstante | Code | Bedeutung" stehen.
        let opt = args[1];
        if (opt && typeof opt === "object" && Array.isArray(opt.headers)) {
          opt = { ...opt, headers: uebersetzeTabelle([opt.headers], tr)[0] };
        }
        return fn(uebersetzeTabelle(args[0], tr), opt, ...args.slice(2));
      }
      if (name === "pmix") {
        return fn(uebersetzePmix(args[0], tr), ...args.slice(1));
      }
      if (name === "code") {
        // Ein Ausgabe-Block ist, was das Programm (oder dhrt) ausgibt --
        // er bleibt woertlich. Eine Zeile wie "'CIRLCE' -- dhrt kennt ihn
        // nicht" sah sonst wie ein Kommentar aus und ging in den Katalog.
        // `{ out: [...] }` (Code MIT Ausgabe darunter, im Lehrbuch) bleibt
        // Code; nur `{ out: true }` ist ein reiner Ausgabe-Block.
        if (args[1] && args[1].out === true) {
          return fn(uebersetzeAusgabe(args[0], args[1], ausgaben, tr), ...args.slice(1));
        }
        return fn(uebersetzeCode(args[0], tr), ...args.slice(1));
      }
      if (name === "cmd") {
        // (name, syntax, beschreibung, codezeilen, opts)
        const a = args.slice();
        if (a[2] !== undefined) a[2] = uebersetzeWert(a[2], tr);
        if (a[3] !== undefined) a[3] = uebersetzeCode(a[3], tr);
        if (a[4] && typeof a[4] === "object" && istText(a[4].caption)) {
          a[4] = { ...a[4], caption: tr(a[4].caption) };
        }
        return fn(...a);
      }
      if (!spec) return fn(...args);
      return fn(...args.map((a, i) => (spec[i] ? uebersetzeWert(a, tr) : a)));
    };
  }
  return neu;
}

/** H, das uebersetzt. `katalog` = { "deutscher text": "english text" };
 *  `ausgaben` = was dhrt in der Zielsprache sagt (siehe uebersetzeAusgabe). */
function wrap(H, katalog, ausgaben = {}) {
  const fehlend = new Set();
  const tr = (s) => {
    // Ein Satz mit eingesetzten Zahlen (vorlage.js): uebersetzt wird die
    // Vorlage, die Zahlen kommen danach hinein.
    const v = vorlagen.get(s);
    const schluessel = v ? v.vorlage : s;
    const t = katalog[schluessel];
    if (t === undefined || t === "") { fehlend.add(schluessel); return s; }
    return v ? fuellen(t, v.werte) : t;
  };
  const H2 = baue(H, tr, ausgaben);
  H2.__fehlend = fehlend;
  return H2;
}

/** H, das nur einsammelt (fuer den Katalog-Auszug). */
function sammle(H) {
  const texte = [];
  const gesehen = new Set();
  const tr = (s) => {
    const v = vorlagen.get(s);
    const schluessel = v ? v.vorlage : s;
    if (!gesehen.has(schluessel)) { gesehen.add(schluessel); texte.push(schluessel); }
    return s;
  };
  return [baue(H, tr), texte];
}

module.exports = { wrap, sammle };
