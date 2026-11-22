// Kapitel 7: Rueckblick auf Teil I. Die Zahlen werden aus dem letzten
// Stand der Mediathek GERECHNET (code/kap06/3_schliessen.dh): jede SUB,
// FUNCTION und die Hauptschleife mit ihrer Zeilenzahl, der Rest ist der
// Aufbau. Aendert sich das Programm, stimmt die Tabelle von selbst.
const fs = require("fs");
const path = require("path");

const DATEI = path.join(__dirname, "..", "..", "code", "kap06", "3_schliessen.dh");
const EINTRAG = path.join(__dirname, "..", "..", "code", "kap04", "eintrag.dh");

// Was jedes Stueck tut und wer es im Studio uebernimmt. Ein Stueck, das
// hier fehlt, faellt beim Bauen auf (siehe teile()).
const ROLLE = {
  aufbau: ["Fenster, Felder, Tabelle, Knöpfe, Menü, Regeln und Bindungen anlegen", "der Entwurf: Fenster zeichnen, Eigenschaften setzen (Kapitel 8, 9)"],
  zeile: ["eine Zeile aus Beschriftung und Feld bauen", "der Entwurf (Kapitel 8)"],
  laden: ["alle Einträge aus der Datenbank lesen", "das Datenbank-Element (Kapitel 13, 14)"],
  einfuegen: ["einen Eintrag einfügen (Beispiele)", "das Datenbank-Element (Kapitel 13)"],
  tabelle_fuellen: ["die Tabelle aus dem Feld füllen", "die gebundene Tabelle (Kapitel 14)"],
  zeile_von: ["die Zeile zu einer Kennung finden", "die gebundene Tabelle (Kapitel 14)"],
  formular_leeren: ["das Formular für einen neuen Eintrag leeren", "das gebundene Formular (Kapitel 14)"],
  bearbeiten: ["einen Eintrag ins Formular laden", "das gebundene Formular (Kapitel 14)"],
  formular_speichern: ["prüfen und speichern", "das gebundene Formular (Kapitel 14)"],
  auswahl_entfernen: ["gewählte Einträge entfernen, in einer Transaktion", "bleibt eigener Code"],
  entfernen_fragen: ["vor dem Entfernen nachfragen", "bleibt eigener Code"],
  beenden_fragen: ["beim Beenden nach Ungespeichertem fragen", "bleibt eigener Code"],
  Schleife: ["Ereignisse abfragen und verteilen", "Ereignis-Methoden statt Abfragen (Kapitel 10)"],
};

function teile() {
  const z = fs.readFileSync(DATEI, "utf8").replace(/\r/g, "").replace(/\n+$/, "").split("\n");
  const stuecke = [];
  let aufbau = 0;
  for (let i = 0; i < z.length; i++) {
    const t = z[i].trim();
    let m = t.match(/^(SUB|FUNCTION) (\w+)/);
    if (m) {
      let j = i;
      while (!/^END (SUB|FUNCTION)/.test(z[j].trim())) j++;
      stuecke.push([m[2], j - i + 1]);
      i = j;
      continue;
    }
    if (t.startsWith("WHILE NOT beenden")) {
      let j = i;
      while (z[j].trim() !== "WEND") j++;
      stuecke.push(["Schleife", j - i + 1]);
      i = j;
      continue;
    }
    aufbau++;
  }
  stuecke.unshift(["aufbau", aufbau]);
  for (const [name] of stuecke) {
    if (!ROLLE[name]) throw new Error(`07_rueckblick.js: kein Eintrag in ROLLE fuer "${name}"`);
  }
  const eintrag = fs.readFileSync(EINTRAG, "utf8").replace(/\n+$/, "").split("\n").length;
  return { stuecke, gesamt: z.length, eintrag };
}

module.exports = (H) => {
  const { stuecke, gesamt, eintrag } = teile();
  const zeile = (n) => stuecke.find((s) => s[0] === n)[1];
  const bleibt = stuecke.filter(([n]) => ROLLE[n][1] === "bleibt eigener Code").reduce((a, [, k]) => a + k, 0);
  const tabelle = stuecke.map(([name, n]) => [
    { text: name === "aufbau" ? "Aufbau" : (name === "Schleife" ? "die Schleife" : name), mono: name !== "aufbau" && name !== "Schleife" },
    String(n), ROLLE[name][0], ROLLE[name][1]]);
  return [
    H.chapter("Was wir von Hand gelernt haben", 7),

    H.p(`Die Mediathek am Ende von Kapitel 6 hat ${gesamt} Zeilen, dazu ${eintrag} in eintrag.dh. Sie hat ein Formular mit Regeln, eine Tabelle zum Sortieren und Filtern, eine Datenbank, Menüs mit Kürzeln und Dialoge, die nachfragen. Bevor sie im Studio ein zweites Mal entsteht, lohnt ein Blick darauf, woraus diese Zeilen bestehen — denn genau daran sieht man, was das Studio einem abnimmt und was nicht.`),

    H.h2("Woraus die Mediathek besteht"),

    H.p("Die Tabelle zählt die Zeilen jedes Teils des Programms: den Aufbau ganz oben, wo Fenster, Felder und Menü entstehen, jede SUB und FUNCTION und die Schleife. Kommentare und Leerzeilen zählen mit, denn sie gehören zum Programm. Die letzte Spalte sagt, wer diese Arbeit im Studio übernimmt."),

    H.table(tabelle, { headers: ["Teil", "Zeilen", "Was es tut", "Im Studio"], widths: [2000, 900, 3300, 2826] }),

    H.h2("Handwerk"),

    H.p(`Der größte Teil ist der Aufbau: ${zeile("aufbau")} Zeilen, und die meisten davon haben mit Büchern und Filmen wenig zu tun. Sie öffnen die Datenbank, legen ein Feld an, geben ihm eine Breite, hängen es in eine Zeile, die Zeile in eine Spalte, binden es an eine Spalte der Datenbank und hängen eine Regel daran. Das ist Handwerk: nötig, aber jedes Mal gleich. Im Studio zeichnet man das Fenster, und die Eigenschaften — Breite, Anker, Regel, Bindung — stellt man im Inspektor ein. Das alles steht dann in der .dhform, und das Programm baut sein Fenster beim Start daraus — dieselben Bausteine, ohne dass eine Zeile davon im eigenen Code steht.`),

    H.p(`Die Schleife hat ${zeile("Schleife")} Zeilen und tut fast nur eines: Sie fragt in jedem Bild jeden Knopf, jeden Menüeintrag und jeden Dialog, ob etwas passiert ist, und ruft dann die passende SUB. Im Studio schreibt man stattdessen eine Methode für das Ereignis Klick des Knopfs; die Abfrage, die sie in jedem Bild aufruft, schreibt das Studio selbst (Kapitel 8). Laden, Speichern und Füllen — laden, tabelle_fuellen, bearbeiten, formular_speichern — übernimmt ab Teil III das Datenbank-Element, an das man Formular und Tabelle hängt.`),

    H.h2("Was bleibt"),

    H.p(`Übrig bleiben ${bleibt} Zeilen: auswahl_entfernen mit seiner Transaktion, die Rückfrage vor dem Entfernen und die Frage beim Beenden. Das ist das, was die Mediathek zu IHRER Anwendung macht — welche Fragen sie stellt, was ganz oder gar nicht geschehen muss, wann sie sich weigert, Ungespeichertes zu verwerfen. Kein Werkzeug kann das wissen; es bleibt Code, im Studio genauso wie hier. Nur steht er dort an dem Knopf oder Menüeintrag, zu dem er gehört, statt in einer langen Schleife.`),

    H.h2("Was man von Hand gelernt hat"),

    H.p("Der Umweg hat gezeigt, was unter der Oberfläche passiert, und das bleibt nützlich, wenn das Studio die Befehle schreibt:"),

    H.bulletRich("Ein Ereignis gilt ein Bild. ", "Ein Klick, eine Antwort, ein Doppelklick sind Ereignisse; was länger gelten soll, merkt sich das Programm in einer Variablen (Kapitel 1, 6)."),
    H.bulletRich("Behälter statt Zahlen. ", "Ein Fenster, das wächst, braucht Gewichte und Anker, keine ausgerechneten Lagen (Kapitel 2)."),
    H.bulletRich("Die Anzeige folgt den Daten. ", "Die Tabelle zeigt nur; die Wahrheit steht in der Datenbank, und geändert wird sie zuerst (Kapitel 3, 4)."),
    H.bulletRich("Werte als Parameter. ", "Was von außen kommt, geht mit ? in die Datenbank, nie in den SQL-Text geklebt (Kapitel 4)."),
    H.bulletRich("Regeln am Feld. ", "Was für ein Feld gilt, steht beim Feld; geprüft wird an einer Stelle für jeden Weg zum Speichern (Kapitel 5)."),
    H.bulletRich("Ein Kürzel steht im Menü. ", "Dort ist es sichtbar und gilt genau einmal (Kapitel 6)."),

    H.p("Im Studio wird man jedem dieser Punkte wieder begegnen — als Eigenschaft im Inspektor, als Ereignis eines Fensters, als Einstellung des Datenbank-Elements. Wer weiß, was dahinter steht, versteht, warum die Eigenschaft so heißt, und weiß, wo er nachsehen muss, wenn etwas nicht tut, was es soll."),
  ];
};
