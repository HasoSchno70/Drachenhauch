// Anhang E: wie es weitergeht. Die Zahlen im ersten Absatz werden
// GERECHNET: aus den Programmen unter code/ und aus der Befehlsliste der
// Laufzeit (daten/builtin_index.json) -- hingeschrieben veralteten sie
// mit dem naechsten Kapitel oder der naechsten Fassung.
const fs = require("fs");
const path = require("path");
const { mitZahlen } = require("../../../_i18n/vorlage");

function zahlen() {
  try {
    const idx = JSON.parse(fs.readFileSync(
      path.join(__dirname, "..", "..", "..", "..", "daten", "builtin_index.json"), "utf-8"));
    const namen = new Set((idx.builtins || [])
      .filter((e) => e && e.name && !e.name.startsWith("_")).map((e) => e.name.toLowerCase()));
    const alle = (d) => fs.readdirSync(d, { withFileTypes: true }).flatMap((e) =>
      e.isDirectory() ? alle(path.join(d, e.name))
        : (e.name.endsWith(".dh") ? [path.join(d, e.name)] : []));
    const dateien = alle(path.join(__dirname, "..", "..", "code"));
    const benutzt = new Set();
    for (const f of dateien) {
      // Ohne Kommentare und Texte: nur, was als Befehl dasteht.
      const t = fs.readFileSync(f, "utf-8").replace(/'.*$/gm, "").replace(/"[^"\n]*"/g, "");
      for (const m of t.matchAll(/[A-Za-z_][A-Za-z0-9_]*\$?/g)) {
        if (namen.has(m[0].toLowerCase())) benutzt.add(m[0].toLowerCase());
      }
    }
    const programme = dateien.filter((f) => !f.includes(path.sep + "loesungen" + path.sep)
      && !path.basename(f).startsWith("fehler_")).length;
    return { gesamt: namen.size, benutzt: benutzt.size, programme };
  } catch (e) {
    return null;
  }
}

module.exports = (H) => {
  const z = zahlen();
  const satz = z
    ? mitZahlen("Die {0} Programme der Kapitel und ihre Lösungen benutzen {1} verschiedene Befehle. Drachenhauch hat {2}. Was jetzt kommt, hängt davon ab, wohin du willst.", [z.programme, z.benutzt, z.gesamt])
    : "Die Programme dieses Buchs benutzen nur einen kleinen Teil der Befehle, die Drachenhauch hat. Was jetzt kommt, hängt davon ab, wohin du willst.";
  return [
    H.chapter("E · Wie es weitergeht"),

    H.p(satz),

    H.h2("Wenn du nachschlagen willst"),

    H.pmix(["Das ", ["Handbuch", false], " liegt bei jeder Installation im Ordner docs, deutsch und englisch, und in der IDE öffnet F1 die Seite zu dem Wort unter der Schreibmarke. Es hat ein Kapitel zur Sprache, eines je Bereich der eingebauten Befehle und eines je Modul — g3d, audio, chart, gui und all die anderen. Wer dort liest, findet auch, was dieses Buch weggelassen hat: Datenbanken, Netz, Fenster mit Knöpfen und Tabellen, Drucken, Geräte am USB-Kabel."]),

    H.p("Daneben gibt es zu jedem Bereich Beispiele im Ordner examples, kurz und zum Ausprobieren, und die Stolpersteine — eine Seite mit allem, worüber man beim Umstieg aus einem anderen BASIC fällt."),

    H.h2("Wenn du weiterlesen willst"),

    H.table([
      ["Das Lehrbuch", "zum Nachschlagen", "die ganze Sprache und jeder Befehl mit einem kleinen Beispielprogramm"],
      ["Der Einstieg", "von vorn", "für alle, die noch nie programmiert haben — langsamer als dieses Buch"],
      ["Galaga", "ein Spiel", "ein Arcade-Shooter in Kapiteln, mit selbst gezeichneten Sprites"],
      ["Tippspiel", "eine Anwendung", "Datenbank, Netz und Rangliste für eine Tipprunde"],
      ["Fenster, Formulare, Daten", "Anwendungen", "das Gegenstück zu diesem Buch: Programme mit Knöpfen, Tabellen und Datenbank"],
    ], { headers: ["Buch", "Wofür", "Was drinsteht"], widths: [2600, 1800, 4626] }),

    H.h2("Wenn du etwas zeigen willst"),

    H.pmix(["Die ", ["Galerie", false], " im Handbuch sammelt Programme, die mit Drachenhauch gebaut sind: Spiele, Demos wie Nachtflug und Lichterglanz — die laufen auch im Browser —, und Anwendungen. Sie liegen in eigenen Repositorys, zum Ausprobieren, Nachlesen und Abschauen. Wer selbst etwas gebaut hat, zeigt es in den Diskussionen des Projekts auf GitHub unter „Show and tell“; Fragen gehören unter „Q&A“, und keine ist zu klein."]),

    H.pmix(["Ein einzelnes Programm zeigt man am schnellsten in der ", ["Spielwiese", false], " auf der Webseite: einfügen, laufen lassen, und der Knopf für den Link packt den Quelltext in die Adresse. Für mehr als eine Datei ist die exportierte Datei aus Kapitel 26 der Weg."]),

    H.h2("Wenn du nicht alles selbst bauen willst"),

    H.pmix(["Bibliotheken holt ", ["dhrt paket", true], " in den Ordner pakete des eigenen Projekts; danach bindet man sie mit IMPORT ein wie eine eigene Datei. Drei liegen dem Projekt schon bei — eine Bestenliste, ein Menü und ein Textkasten, der wie in einem Rollenspiel Buchstabe für Buchstabe schreibt:"]),

    H.code(["dhrt paket hole github:HasoSchno70/Drachenhauch@main#bibliotheken/menu"]),

    H.p("Wer selbst eine Bibliothek baut, die anderen nützt, kann sie genauso anbieten: als eigenes Repository oder als Ordner im Projekt. Wie, steht im Handbuch unter „Pakete“."),

    H.h2("Werkzeuge, die schon da sind"),

    H.table([
      [{ text: "Drachenhauch IDE", mono: true }, "Farben im Quelltext, Vervollständigung, Hilfe zum Wort, Debugger und Profiler"],
      [{ text: "Drachenhauch Studio", mono: true }, "Fenster zusammenklicken und den Code der Ereignisse dazuschreiben — für Anwendungen"],
      [{ text: "dhrt profile datei.dh", mono: true }, "wie viel Zeit jede Zeile kostet"],
      [{ text: "dhrt test", mono: true }, "eigene Prüfsammlungen, wie die, mit denen dieses Buch geprüft wird"],
    ], { headers: ["Werkzeug", "Wofür"], widths: [3200, 5826], mono: [0] }),

    H.h2("Der wichtigste Schritt"),

    H.p("Der steht in keinem Buch. Er heißt: etwas bauen, das du selbst sehen willst. Jedes Kapitel hier hatte eine Erfinden-Übung, und die beste Lösung zu keiner davon steht in Anhang A — sie ist die, auf die du selbst kommst."),

    H.p("Fang klein an, sieh dir nach jeder Änderung an, was passiert, und glaub keiner Zeile, die du nicht hast laufen sehen. So ist dieses Buch entstanden: Jede Aussage darin wurde ausprobiert, und wo Drachenhauch nicht tat, was es sollte, wurde Drachenhauch besser. Viel Vergnügen beim nächsten Wunder."),
  ];
};
