// Uebersetzungsschicht fuer das Lehrbuch.
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
//   wrap(H, katalog)   -> H', das uebersetzt
//   sammle(H)          -> [H', strings] zum Einsammeln aller Texte
//
// Fehlt ein Eintrag, bleibt der deutsche Text stehen. Das Buch baut also
// auch bei halb gefuelltem Katalog, nur eben teilweise deutsch -- besser
// als ein Abbruch mitten im Satz.
"use strict";

// Welche Argumente eines H-Aufrufs sind Text? Der Rest (Bilddateinamen,
// Zahlen, Optionen) darf NICHT durch den Katalog.
//   true  = uebersetzen
//   false = unveraendert lassen
const FELDER = {
  p:           [true],
  h1:          [true],
  h2:          [true],
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
  /^[A-Z][A-Z0-9_$]*\s*\(.*\)$/,                  // BEFEHL(...)
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
        if (zelle && typeof zelle === "object" && istText(zelle.text)) {
          return { ...zelle, text: tr(zelle.text) };
        }
        return zelle;
      })
    : zeile);
}

// pmix(["text", ["code"], "text"]) -- Array-Eintraege sind Inline-Code
// und bleiben; die nackten Strings sind Prosa.
function uebersetzePmix(teile, tr) {
  if (!Array.isArray(teile)) return teile;
  return teile.map((x) => (istText(x) ? tr(x) : x));
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

function baue(H, tr) {
  const neu = {};
  for (const [name, fn] of Object.entries(H)) {
    if (typeof fn !== "function") { neu[name] = fn; continue; }
    const spec = FELDER[name];
    neu[name] = (...args) => {
      if (name === "table") {
        return fn(uebersetzeTabelle(args[0], tr), ...args.slice(1));
      }
      if (name === "pmix") {
        return fn(uebersetzePmix(args[0], tr), ...args.slice(1));
      }
      if (name === "code") {
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

/** H, das uebersetzt. `katalog` = { "deutscher text": "english text" }. */
function wrap(H, katalog) {
  const fehlend = new Set();
  const tr = (s) => {
    const t = katalog[s];
    if (t === undefined || t === "") { fehlend.add(s); return s; }
    return t;
  };
  const H2 = baue(H, tr);
  H2.__fehlend = fehlend;
  return H2;
}

/** H, das nur einsammelt (fuer den Katalog-Auszug). */
function sammle(H) {
  const texte = [];
  const gesehen = new Set();
  const tr = (s) => { if (!gesehen.has(s)) { gesehen.add(s); texte.push(s); } return s; };
  return [baue(H, tr), texte];
}

module.exports = { wrap, sammle };
