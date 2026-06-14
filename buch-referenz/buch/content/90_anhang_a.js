// Anhang A -- Befehls-Index. Wird beim Build aus der lebenden Builtin-Liste
// (gamebasic/editor_qt/builtin_index.json) erzeugt, bleibt so automatisch in
// Sync mit der Engine. Pro Anfangsbuchstabe eine Tabelle (Signatur | Modul).
const fs = require("fs");
const path = require("path");

module.exports = (H) => {
  const out = [
    H.part("Anhang"),
    H.chapter("Anhang A — Befehls-Index"),
    H.p("Dieser Index listet alle eingebauten Befehle und Modul-Funktionen alphabetisch mit ihren Argumenten. Die Spalte „Modul“ sagt, welches IMPORT du brauchst – ein Strich (—) bedeutet: Kern-Befehl, ohne IMPORT verfügbar. Die ausführliche Erklärung jedes Befehls findest du im jeweiligen Kapitel (Teil III für die Kern-Befehle, Teil IV für Grafik/Sound, Teil V für die Module)."),
  ];

  let entries = [];
  try {
    const idxPath = path.join(__dirname, "..", "..", "..", "gamebasic", "editor_qt", "builtin_index.json");
    const data = JSON.parse(fs.readFileSync(idxPath, "utf-8"));
    entries = (data.builtins || []).filter((e) => e && e.name && !e.name.startsWith("_"));
  } catch (err) {
    out.push(H.warn("Index nicht gefunden: " + err.message, "Build-Hinweis"));
    return out;
  }

  // Modul-Label: core/gbrt -> "—" (kein IMPORT), sonst der Modulname.
  const modLabel = (m) => (!m || m === "core" || m === "gbrt") ? "—" : m;

  // alphabetisch sortieren, Duplikate (gleicher Name) zusammenfassen
  const byName = {};
  for (const e of entries) {
    if (byName[e.name]) continue;
    byName[e.name] = e;
  }
  const names = Object.keys(byName).sort((a, b) => a.localeCompare(b, "en"));

  // nach Anfangsbuchstabe gruppieren
  const groups = {};
  for (const n of names) {
    let L = n[0].toUpperCase();
    if (!/[A-Z]/.test(L)) L = "#";          // Ziffern/Sonderzeichen
    (groups[L] = groups[L] || []).push(n);
  }
  const letters = Object.keys(groups).sort();

  out.push(H.p("Insgesamt " + names.length + " Befehle."));

  for (const L of letters) {
    out.push(H.h2("— " + L + " —"));
    const rows = groups[L].map((n) => {
      const e = byName[n];
      const sig = e.signature || (n + "()");
      return [{ text: sig, mono: true }, { text: modLabel(e.module), color: "555555" }];
    });
    out.push(H.table(rows, {
      headers: ["Befehl & Argumente", "Modul"],
      widths: [6900, 2126],
    }));
  }

  return out;
};
