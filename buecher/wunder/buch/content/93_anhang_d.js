// Anhang D: die Meldungen von dhrt, die im Buch abgedruckt sind -- nicht
// abgeschrieben, sondern aus den Kapiteln GESAMMELT, wie pruef_meldungen.js
// sie findet: ein Ausgabe-Block, dessen erste Zeile mit `datei.dh:N:` (oder
// `Laufzeitfehler in datei.dh:N:`) beginnt. Kommt eine Meldung in einem
// Kapitel dazu oder aendert sich ihr Wortlaut, steht es hier von selbst.
// Die Seite ist die, auf der das Kapitel beginnt (aus toc_pages.de.json,
// die der erste Durchgang des Buchbaus misst).
const fs = require("fs");
const path = require("path");

const MELDUNG = /^(?:Laufzeitfehler in )?([\w.-]+\.dh):\d+: /;

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
      if (MELDUNG.test(z[0])) funde.push({ text: z.map((x) => x.trim()).join(" "), kapitel });
      return null;
    },
  }, { get: (t, k) => (k in t ? t[k] : () => null) });
  for (const f of dateien) require(path.join(dir, f))(H);
  return funde;
}

// "Laufzeitfehler in x.dh:7: ARRAY_AVG: ..." -> ("Laufzeitfehler", "ARRAY_AVG: ...")
// "x.dh:6: Warnung: TRY ohne ..."           -> ("Warnung", "TRY ohne ...")
function zerlegen(text) {
  const rest = text.replace(MELDUNG, "");
  if (text.startsWith("Laufzeitfehler")) return ["Laufzeitfehler", rest];
  const m = rest.match(/^([\w-]+): (.*)$/);
  if (m && /^(Warnung|[\w-]*[Ff]ehler)$/.test(m[1])) return [m[1], m[2]];
  return ["Andere", rest];
}

const ART_TEXT = {
  "Laufzeitfehler": "Das Programm lief und hielt an dieser Stelle an. Die Zeilennummer vorn in der Meldung sagt, wo.",
  "Warnung": "Beim Übersetzen bemerkt, noch bevor das Programm läuft. Es läuft trotzdem — aber meist nicht so, wie gemeint.",
  "Compile-Fehler": "Das Programm startet gar nicht erst: Beim Übersetzen fiel etwas auf, das so nicht gehen kann.",
};

module.exports = (H) => {
  const seitenDatei = path.join(__dirname, "..", "toc_pages.de.json");
  const seiten = fs.existsSync(seitenDatei)
    ? JSON.parse(fs.readFileSync(seitenDatei, "utf-8")) : {};
  // Dieselbe Meldung in mehreren Kapiteln: eine Zeile, alle Kapitel.
  const zeile = new Map();
  const nachArt = new Map();
  for (const f of sammeln()) {
    const [art, meldung] = zerlegen(f.text);
    const nr = f.kapitel.split(" · ")[0];
    const seite = String(seiten[f.kapitel] || "");
    if (zeile.has(meldung)) {
      const z = zeile.get(meldung);
      if (!z.kap.includes(nr)) { z.kap.push(nr); if (seite) z.seiten.push(seite); }
      continue;
    }
    const z = { kap: [nr], seiten: seite ? [seite] : [] };
    zeile.set(meldung, z);
    if (!nachArt.has(art)) nachArt.set(art, []);
    nachArt.get(art).push({ meldung, z });
  }
  for (const [art, liste] of nachArt) {
    nachArt.set(art, liste.map(({ meldung, z }) => [{ text: meldung, mono: true, size: 17 },
                                                    z.kap.join(", "), z.seiten.join(", ")]));
  }
  const anzahl = zeile.size;
  const out = [
    H.chapter("D · Die Meldungen im Buch"),
    H.p(`Jede Meldung von dhrt, die in diesem Buch abgedruckt ist — ${anzahl} verschiedene —, mit dem Kapitel, in dem sie erklärt wird, und der Seite, auf der dieses Kapitel beginnt. Wer eine davon auf dem eigenen Bildschirm sieht, findet hier, wo sie besprochen wurde. Die Meldungen sind die, die dhrt beim Schreiben dieses Buchs wirklich ausgegeben hat; eine Prüfung lässt jedes Beispiel laufen und vergleicht.`),
  ];
  const reihe = ["Laufzeitfehler", "Warnung"];
  const arten = [...nachArt.keys()].sort((a, b) =>
    (reihe.includes(a) ? reihe.indexOf(a) : 9) - (reihe.includes(b) ? reihe.indexOf(b) : 9) || a.localeCompare(b));
  for (const art of arten) {
    out.push(H.h2(art === "Warnung" ? "Warnungen" : art));
    if (ART_TEXT[art]) out.push(H.p(ART_TEXT[art]));
    out.push(H.table(nachArt.get(art), { headers: ["Meldung", "Kapitel", "Seite"],
                                          widths: [7226, 900, 900] }));
  }
  return out;
};
