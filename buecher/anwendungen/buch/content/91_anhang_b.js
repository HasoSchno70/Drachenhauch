// Anhang B: Spickzettel gui und Studio. Die Ereignisse und die Tasten des
// Studios werden aus seinen Quellen GELESEN (studio/projekt.dh,
// studio/bibliothek.dh, studio/oberflaeche.dh) -- hingeschrieben veralteten
// sie mit der naechsten Fassung. Die Befehle der gui und was ein Programm
// aus dem Studio mitbringt, stehen in ../spickzettel.json; die
// Pruefsammlung haelt jede Zeile dort gegen dhrt und den erzeugten Code.
const fs = require("fs");
const path = require("path");

const STUDIO = path.join(__dirname, "..", "..", "..", "..", "studio");
const lesen = (f) => fs.readFileSync(path.join(STUDIO, f), "utf8").replace(/\r/g, "");
const zettel = JSON.parse(fs.readFileSync(path.join(__dirname, "..", "spickzettel.json"), "utf8"));

// Die Namen der Bausteine je Art, wie die Bibliothek sie zeigt.
function bausteine() {
  const namen = new Map();
  for (const m of lesen("bibliothek.dh").matchAll(/^\s*baustein\("[^"]*", "(\w+)", "([^"]+)"/gm)) {
    if (!namen.has(m[1])) namen.set(m[1], []);
    if (!namen.get(m[1]).includes(m[2])) namen.get(m[1]).push(m[2]);
  }
  namen.set("menue", ["Menüeintrag"]);
  namen.set("fenster", ["Fenster"]);
  namen.set("control", ["Eigenes Control"]);
  namen.set("app", ["App"]);
  return namen;
}

// Je Art die Ereignisse aus ereignisseFuer; nach `aus = [...]` haengt die
// Funktion MausRein, MausRaus und Abgelegt an, nach RETURN nicht.
function ereignisse() {
  const t = lesen("projekt.dh");
  const rumpf = t.slice(t.indexOf("FUNCTION ereignisseFuer"), t.indexOf("END FUNCTION", t.indexOf("FUNCTION ereignisseFuer")));
  const dazu = [...rumpf.matchAll(/ARRAY_PUSH\(aus, "(\w+)"\)/g)].map((m) => m[1]);
  const zeilen = [];
  for (const m of rumpf.matchAll(/CASE ((?:"[\w:]+"(?:, )?)+)\s*\n\s*(aus =|RETURN) \[([^\]]*)\]/g)) {
    const arten = [...m[1].matchAll(/"([\w:]+)"/g)].map((x) => x[1]);
    let evs = [...m[3].matchAll(/"(\w+)"/g)].map((x) => x[1]);
    if (m[2] === "aus =") evs = evs.concat(dazu);
    if (evs.length) zeilen.push({ arten, evs });
  }
  return { zeilen, dazu };
}

function beschreibungen() {
  const t = lesen("projekt.dh");
  const was = new Map();
  for (const m of t.matchAll(/CASE "(\w+)" : RETURN tr\$\("([^"]+)"\)/g)) was.set(m[1], m[2].replace(/ -- /g, " — "));
  const param = new Map();
  const p = t.slice(t.indexOf("FUNCTION ereignisParameter$"));
  for (const m of p.slice(0, p.indexOf("END FUNCTION")).matchAll(/CASE ((?:"\w+"(?:, )?)+) : RETURN "([^"]+)"/g)) {
    for (const x of m[1].matchAll(/"(\w+)"/g)) param.set(x[1], m[2]);
  }
  return { was, param };
}

// Die Kuerzel des Studios: jeder Menueeintrag mit Taste, nach Menues.
function tasten() {
  const zeilen = [];
  let menue = "";
  for (const z of lesen("oberflaeche.dh").split("\n")) {
    const m = z.match(/GUI_MENU\(win, tr\$\("([^"]+)"\)\)/);
    if (m) { menue = m[1]; continue; }
    let e = z.match(/GUI_MENU_ITEM\(m, tr\$\("([^"]+)"\), kz\$\("([^"]+)"\)\)/)
         || z.match(/anordnenPunkt\(m, tr\$\("([^"]+)"\), "\w+", kz\$\("([^"]+)"\)\)/);
    if (!e) {
      const a = z.match(/GUI_MENU_ITEM\(m, tr\$\("(.+?)\s{2,}(\S+)"\)\)/);
      if (a) e = a;
    }
    if (e && menue) zeilen.push([menue, e[1], { text: e[2], mono: true }]);
  }
  return zeilen;
}

const mono = (t) => ({ text: t, mono: true });

module.exports = (H) => {
  const namen = bausteine();
  const { zeilen: evZeilen, dazu } = ereignisse();
  const { was, param } = beschreibungen();
  const evTabelle = evZeilen
    .map(({ arten, evs }) => [arten.flatMap((a) => namen.get(a) || []).join(", "),
                              evs.filter((e) => !dazu.includes(e)).join(", ")])
    .filter((z) => z[0] !== "");
  const alleEvs = [...new Set(evZeilen.flatMap((z) => z.evs))];
  const wasTabelle = alleEvs.filter((e) => was.has(e))
    .map((e) => [mono(param.has(e) ? `${e}(${param.get(e)})` : e), was.get(e)]);

  const out = [
    H.chapter("B · Spickzettel gui und Studio"),

    H.p("Zum Nachschlagen, wenn man weiß, was man will, aber nicht mehr, wie es heißt: die Befehle der gui, mit denen Teil I gebaut ist, die Ereignisse der Bausteine im Studio, was jedes Programm aus dem Studio mitbringt, und die Tasten des Studios. Die Ereignisse und Tasten liest der Buchbau aus den Quellen des Studios, sie sind also die der Fassung, mit der dieses Buch gebaut ist."),

    H.h2("Die gui von Hand"),

    H.p("Jeder Befehl hier kommt in einem Programm von Teil I vor. Die Zahlen sind Lage und Größe in Punkten, von der linken oberen Ecke des Fensters; w steht für einen Baustein (ein GUI_WIDGET)."),
  ];
  for (const [gruppe, zeilen] of zettel.gui) {
    out.push(H.h3(gruppe));
    out.push(H.table(zeilen.map(([b, w]) => [mono(b), w]),
                     { headers: ["Befehl", "was er tut"], widths: [4800, 4226] }));
  }

  out.push(
    H.h2("Ereignisse im Studio"),
    H.pmix(["Ein Ereignis wird im Studio zu einer Methode des Fensters, benannt nach dem Baustein und dem Ereignis: ", ["knopfNeu_Klick", true], ", ", ["feldSuche_Geaendert", true], ", ", ["netzTimer_Aktion", true], ". Das Fenster selbst hat Methoden ohne Vorsilbe: ", ["Geoeffnet", true], ", ", ["Takt", true], ". Jeder Baustein im Fenster hat außerdem ", ["MausRein", true], ", ", ["MausRaus", true], " und ", ["Abgelegt", true], "."]),
    H.table(evTabelle, { headers: ["Baustein", "Ereignisse"], widths: [4600, 4426] }),
    H.p("Wann jedes kommt, und mit welchen Werten die Methode gerufen wird:"),
    H.table(wasTabelle, { headers: ["Ereignis", "kommt …"], widths: [3600, 5426] }),

    H.h2("Was ein Programm aus dem Studio mitbringt"),
    H.p("Diese Methoden und Funktionen schreibt das Studio beim Start in _studio; im eigenen Code ruft man sie mit Self. (die des Fensters) oder einfach beim Namen. Ein Fenster, das an einer Tabelle hängt, ist eins, dessen Inspektor unter Daten eine Tabelle nennt (Kapitel 14)."),
  );
  for (const [gruppe, zeilen] of zettel.studio) {
    out.push(H.h3(gruppe));
    out.push(H.table(zeilen.map(([s, w]) => [mono(s), w]),
                     { headers: ["Methode", "was sie tut"], widths: [5200, 3826] }));
  }

  out.push(
    H.h2("Tasten im Studio"),
    H.p("Jeder Eintrag der Menüs des Studios, der ein Kürzel hat. In der englischen Oberfläche heißen die Tasten Ctrl, Shift und Del."),
    H.table(tasten(), { headers: ["Menü", "Eintrag", "Taste"], widths: [1800, 4426, 2800] }),
  );
  return out;
};
