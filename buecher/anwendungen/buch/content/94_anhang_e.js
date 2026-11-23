// Anhang E: die Meldungen, die im Buch abgedruckt sind -- nicht
// abgeschrieben, sondern aus den Kapiteln GESAMMELT, wie in "Kleine
// Wunder" (dort Anhang D). Genommen wird jeder Ausgabe-Block, dessen
// erste Zeile wie eine Meldung von dhrt anfaengt (`datei.dh:N:`,
// `Laufzeitfehler in datei.dh:N:`, im Debugger `Laufzeitfehler in
// Klasse.methode, Zeile N:`), und jeder Block, den ein Kapitel als
// Meldung des Studios kennzeichnet ({ out: true, studio: "Probleme" }).
// Kommt eine dazu oder aendert sich ihr Wortlaut, steht es hier von
// selbst. Die Seite ist die, auf der das Kapitel beginnt (aus
// toc_pages.de.json, die der erste Durchgang des Buchbaus misst).
const fs = require("fs");
const path = require("path");

const MELDUNG = /^(?:Laufzeitfehler in )?[\w./-]+(?:\.dh:\d+|, Zeile \d+): /;

function sammeln() {
  const dir = __dirname;
  const eigene = path.basename(__filename);
  const dateien = fs.readdirSync(dir).filter((f) => f.endsWith(".js") && f < eigene).sort();
  const funde = [];
  let kapitel = "";
  const H = new Proxy({
    chapter: (titel, nr) => { kapitel = nr !== undefined ? `${nr} · ${titel}` : titel; return null; },
    code: (zeilen, opts = {}) => {
      if (!opts.out) return null;
      const z = Array.isArray(zeilen) ? zeilen : [zeilen];
      // Den Zusatz "-- Aufruf: ..." haengt dhrt nur ausserhalb des
      // Debuggers an; ohne ihn ist es dieselbe Meldung.
      const text = z.map((x) => x.trim()).join(" ").replace(/ -- Aufruf: .*$/, "");
      if (opts.studio) funde.push({ art: "Im Studio", text, kapitel, wo: opts.studio });
      else if (MELDUNG.test(z[0])) funde.push({ text, kapitel });
      return null;
    },
  }, { get: (t, k) => (k in t ? t[k] : () => null) });
  for (const f of dateien) require(path.join(dir, f))(H);
  return funde;
}

// "Laufzeitfehler in x.dh:7: GUI_...: ..." -> ("Laufzeitfehler", "GUI_...: ...")
// "x.dh:2: Compile-Fehler: Unbekannter ..." -> ("Compile-Fehler", "Unbekannter ...")
function zerlegen(f) {
  if (f.art) return [f.art, f.text, f.wo];
  const rest = f.text.replace(MELDUNG, "");
  if (f.text.startsWith("Laufzeitfehler")) return ["Laufzeitfehler", rest, ""];
  const m = rest.match(/^([\w-]+): (.*)$/);
  if (m && /^(Warnung|[\w-]*[Ff]ehler)$/.test(m[1])) return [m[1], m[2], ""];
  return ["Andere", rest, ""];
}

const ART_TEXT = {
  "Laufzeitfehler": "Das Programm lief und hielt an dieser Stelle an. Vorn in der Meldung steht, wo: Datei und Zeile, unter dem Debugger die Methode und ihre Zeile.",
  "Compile-Fehler": "Das Programm startet gar nicht erst: Beim Übersetzen fiel etwas auf, das so nicht gehen kann.",
  "Im Studio": "Meldungen des Studios selbst: unter Probleme, was die Prüfung beim Tippen findet, noch bevor das Programm läuft, und in der Statuszeile, was eine Bedienung nicht tun konnte.",
};

module.exports = (H) => {
  const seitenDatei = path.join(__dirname, "..", "toc_pages.de.json");
  const seiten = fs.existsSync(seitenDatei)
    ? JSON.parse(fs.readFileSync(seitenDatei, "utf-8")) : {};
  // Dieselbe Meldung in mehreren Kapiteln: eine Zeile, alle Kapitel.
  const zeile = new Map();
  const nachArt = new Map();
  for (const f of sammeln()) {
    const [art, meldung, wo] = zerlegen(f);
    const nr = f.kapitel.split(" · ")[0];
    const seite = String(seiten[f.kapitel] || "");
    if (zeile.has(meldung)) {
      const z = zeile.get(meldung);
      if (!z.kap.includes(nr)) { z.kap.push(nr); if (seite) z.seiten.push(seite); }
      continue;
    }
    const z = { kap: [nr], seiten: seite ? [seite] : [], wo };
    zeile.set(meldung, z);
    if (!nachArt.has(art)) nachArt.set(art, []);
    nachArt.get(art).push({ meldung, z });
  }
  const anzahl = zeile.size;
  const out = [
    H.chapter("E · Die Meldungen im Buch"),
    H.p(`Jede Meldung, die in diesem Buch abgedruckt ist — ${anzahl} verschiedene —, mit dem Kapitel, in dem sie erklärt wird, und der Seite, auf der dieses Kapitel beginnt. Wer eine davon auf dem eigenen Bildschirm sieht, findet hier, wo sie besprochen wurde. Die Meldungen von dhrt sind die, die es beim Schreiben dieses Buchs wirklich ausgegeben hat; eine Prüfung lässt jedes Beispiel laufen und vergleicht.`),
  ];
  const reihe = ["Laufzeitfehler", "Compile-Fehler", "Im Studio"];
  const arten = [...nachArt.keys()].sort((a, b) =>
    (reihe.includes(a) ? reihe.indexOf(a) : 9) - (reihe.includes(b) ? reihe.indexOf(b) : 9) || a.localeCompare(b));
  for (const art of arten) {
    out.push(H.h2(art));
    if (ART_TEXT[art]) out.push(H.p(ART_TEXT[art]));
    const mitWo = art === "Im Studio";
    const zeilen = nachArt.get(art).map(({ meldung, z }) => {
      const r = [{ text: meldung, mono: true, size: 17 }];
      if (mitWo) r.push(z.wo);
      r.push(z.kap.join(", "), z.seiten.join(", "));
      return r;
    });
    out.push(H.table(zeilen, mitWo
      ? { headers: ["Meldung", "wo", "Kapitel", "Seite"], widths: [5826, 1400, 900, 900] }
      : { headers: ["Meldung", "Kapitel", "Seite"], widths: [7226, 900, 900] }));
  }
  return out;
};
