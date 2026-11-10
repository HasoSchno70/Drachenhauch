// Anhang B baut seine Tabellen NICHT aus einer gepflegten Liste, sondern aus
// den Programmen unter code/kapNN/. Eine Handliste veraltet beim ersten neuen
// Kapitel, und niemand merkt es -- diese hier kann es nicht.
//
// Aus dem Code kommen die Namen und das frueheste Kapitel, aus BEFEHLE die
// Zuordnung zu einer Gruppe und der erklaerende Halbsatz. Steht ein Befehl im
// Code, aber nicht in BEFEHLE, landet er sichtbar in einer eigenen Tabelle am
// Ende -- dann faellt es beim naechsten Blick ins Buch auf.
const fs = require("fs");
const path = require("path");

const GRUPPEN = [
  "Fenster und Bild", "Zeichnen", "Bilder", "Tastatur und Maus",
  "Rechnen und Zufall", "Klang", "Arrays", "Nachschlagen", "Text",
  "Dateien", "Oberfläche", "Datenbank", "Netz",
];

const BEFEHLE = {
  SCREEN: ["Fenster und Bild", "Fenster öffnen — Breite, Höhe, Titel"],
  CLS: ["Fenster und Bild", "das Bild mit einer Farbe füllen"],
  FLIP: ["Fenster und Bild", "das fertige Bild zeigen"],
  QUITREQUESTED: ["Fenster und Bild", "hat jemand das Fenster geschlossen?"],
  SLEEP: ["Fenster und Bild", "warten, in Millisekunden"],
  LOADFONT: ["Fenster und Bild", "eine Schriftdatei laden"],
  SETFONT: ["Fenster und Bild", "ab jetzt in dieser Schrift zeichnen"],

  RGB: ["Zeichnen", "eine Farbe aus Rot, Grün, Blau (je 0 bis 255)"],
  PLOT: ["Zeichnen", "einen einzelnen Punkt setzen"],
  LINE: ["Zeichnen", "eine Linie von hier nach dort"],
  CIRCLE: ["Zeichnen", "einen gefüllten Kreis"],
  CIRCLEOUTLINE: ["Zeichnen", "nur den Rand eines Kreises"],
  BOX: ["Zeichnen", "ein gefülltes Rechteck — zwei Ecken, keine Größe"],
  RECT: ["Zeichnen", "nur den Rand eines Rechtecks"],
  TEXT: ["Zeichnen", "Text an eine Stelle schreiben"],
  TEXT_SIZE: ["Zeichnen", "Schriftgröße für die folgenden Texte"],
  TEXT_WIDTH: ["Zeichnen", "wie breit ein Text würde — zum Zentrieren"],

  LOADIMAGE: ["Bilder", "eine Bilddatei laden"],
  DRAWIMAGE: ["Bilder", "ein Bild zeichnen"],
  DRAWIMAGEPART: ["Bilder", "einen Ausschnitt daraus — für Animationen"],

  KEYPRESSED: ["Tastatur und Maus", "ist die Taste gerade unten?"],
  KEYHIT: ["Tastatur und Maus", "wurde sie eben gedrückt? (einmal je Druck)"],
  KEY_ANY_HIT: ["Tastatur und Maus", "irgendeine Taste gedrückt? liefert ihren Code"],
  "KEY_NAME$": ["Tastatur und Maus", "der Name zu einem Tastencode"],
  MOUSEX: ["Tastatur und Maus", "waagerechte Mausposition"],
  MOUSEY: ["Tastatur und Maus", "senkrechte Mausposition"],
  MOUSEBUTTON: ["Tastatur und Maus", "ist die Maustaste unten? 0 ist links"],

  INT: ["Rechnen und Zufall", "Kommazahl zu ganzer Zahl, abgeschnitten"],
  MIN: ["Rechnen und Zufall", "die kleinere von zwei Zahlen"],
  SQR: ["Rechnen und Zufall", "Quadratwurzel"],
  SIN: ["Rechnen und Zufall", "Sinus — für Wellen und Kreisbahnen"],
  COS: ["Rechnen und Zufall", "Kosinus, das Gegenstück dazu"],
  RAD: ["Rechnen und Zufall", "Grad in Bogenmaß, das SIN und COS wollen"],
  RND: ["Rechnen und Zufall", "Zufallszahl zwischen 0 und 1"],
  RANDINT: ["Rechnen und Zufall", "ganze Zufallszahl von … bis …"],
  RANDOMIZE: ["Rechnen und Zufall", "den Zufall neu mischen — oder festnageln"],

  AUDIO_TONE: ["Klang", "einen Ton bauen: Höhe, Dauer, Form, Lautstärke"],
  AUDIO_NOISE: ["Klang", "ein Rauschen bauen — Explosionen, Schritte"],
  PLAYSOUND: ["Klang", "einen gebauten Klang abspielen"],

  REDIM: ["Arrays", "die Größe eines Arrays ändern"],
  LEN: ["Arrays", "wie viele Fächer — oder wie viele Zeichen"],
  ARRAY_PUSH: ["Arrays", "hinten anhängen"],
  ARRAY_REMOVE_AT: ["Arrays", "einen Eintrag herausnehmen, der Rest rückt auf"],
  ARRAY_INDEXOF: ["Arrays", "wo steht das? oder -1"],
  ARRAY_MAX: ["Arrays", "der größte Wert darin"],
  ARRAY_AVG: ["Arrays", "der Durchschnitt"],
  SORT: ["Arrays", "der Größe nach ordnen"],
  REVERSE: ["Arrays", "umdrehen"],

  MAPPUT: ["Nachschlagen", "unter einem Namen ablegen"],
  MAPGET: ["Nachschlagen", "unter einem Namen holen"],
  MAPGETOR: ["Nachschlagen", "holen, und wenn nichts da ist, dieses hier"],
  MAPKEYS: ["Nachschlagen", "alle Namen als Array"],
  MAPSIZE: ["Nachschlagen", "wie viele Einträge"],

  "STR$": ["Text", "Zahl zu Text — nötig vor jedem +"],
  VAL: ["Text", "Text zu Zahl"],
  "LEFT$": ["Text", "die ersten n Zeichen"],
  "MID$": ["Text", "ab Stelle n, gezählt ab null"],
  "TRIM$": ["Text", "Leerzeichen vorn und hinten weg"],
  "UPPER$": ["Text", "alles groß"],
  "LOWER$": ["Text", "alles klein — kennt auch Umlaute"],
  "CHR$": ["Text", "das Zeichen zu einer Nummer, 10 ist der Zeilenumbruch"],
  "REPEAT$": ["Text", "einen Text n-mal aneinanderhängen"],
  "SPLIT$": ["Text", "an einem Trennzeichen zerlegen, ergibt ein Array"],

  FILEEXISTS: ["Dateien", "gibt es die Datei?"],
  READLINES: ["Dateien", "die ganze Datei als Zeilen-Array"],
  OPENFILE: ["Dateien", "eine Datei zum Schreiben öffnen"],
  WRITELINE: ["Dateien", "eine Zeile hineinschreiben"],
  CLOSEFILE: ["Dateien", "schließen — sonst fehlt am Ende etwas"],

  GUI_WINDOW: ["Oberfläche", "ein Fenster im Fenster"],
  GUI_LABEL: ["Oberfläche", "eine Beschriftung"],
  GUI_BUTTON: ["Oberfläche", "ein Knopf"],
  GUI_CHECKBOX: ["Oberfläche", "ein Schalter zum An- und Ausklicken"],
  GUI_SLIDER: ["Oberfläche", "ein Schieber"],
  GUI_TEXTINPUT: ["Oberfläche", "ein Feld zum Hineinschreiben"],
  GUI_LISTBOX: ["Oberfläche", "eine Liste zum Auswählen"],
  GUI_DROPDOWN: ["Oberfläche", "ein aufklappendes Auswahlfeld"],
  GUI_PROGRESS: ["Oberfläche", "ein Balken, Wert von 0.0 bis 1.0"],
  GUI_GROUPBOX: ["Oberfläche", "ein Rahmen um Zusammengehöriges"],
  GUI_TABS: ["Oberfläche", "eine Reiterleiste"],
  GUI_UPDATE: ["Oberfläche", "nachsehen, was Maus und Tastatur taten"],
  GUI_DRAW: ["Oberfläche", "die Oberfläche zeichnen"],
  GUI_CLICKED: ["Oberfläche", "wurde geklickt?"],
  GUI_CHECKED: ["Oberfläche", "ist der Schalter an?"],
  GUI_VALUE: ["Oberfläche", "der Wert eines Schiebers — als Kommazahl"],
  GUI_TEXT: ["Oberfläche", "was im Eingabefeld steht"],
  GUI_LISTBOX_SELECTED: ["Oberfläche", "gewählte Zeile, oder -1"],
  GUI_DROPDOWN_SELECTED: ["Oberfläche", "gewählte Nummer im Auswahlfeld"],
  GUI_ACTIVE_TAB: ["Oberfläche", "welcher Reiter ist vorn"],
  GUI_SET_TEXT: ["Oberfläche", "Beschriftung oder Feldinhalt ändern"],
  GUI_SET_VALUE: ["Oberfläche", "Schieber oder Balken setzen"],
  GUI_SET_LISTBOX: ["Oberfläche", "die Liste neu füllen"],
  GUI_SET_DROPDOWN: ["Oberfläche", "das Auswahlfeld neu füllen"],
  GUI_DROPDOWN_SET_SELECTED: ["Oberfläche", "die Auswahl setzen"],
  GUI_SET_TAB: ["Oberfläche", "auf welchen Reiter das Element gehört"],
  GUI_SET_ACTIVE_TAB: ["Oberfläche", "den Reiter umschalten"],
  GUI_SET_ANCHOR: ["Oberfläche", "an welchen Kanten das Element klebt"],
  GUI_WINDOW_RESIZABLE: ["Oberfläche", "das Fenster größenveränderbar machen"],

  DB_OPEN: ["Datenbank", "Datenbankdatei öffnen — und anlegen, wenn nötig"],
  DB_EXEC: ["Datenbank", "einen SQL-Befehl ausführen"],
  DB_QUERY: ["Datenbank", "eine Frage stellen"],
  DB_NEXT: ["Datenbank", "zur nächsten Antwortzeile, oder FALSE"],
  DB_GET_INT: ["Datenbank", "eine Zahlenspalte holen, gezählt ab null"],
  DB_GET_STRING: ["Datenbank", "eine Textspalte holen"],
  DB_CLOSE_RESULT: ["Datenbank", "die Antwort schließen — auch vor RETURN"],
  DB_BEGIN: ["Datenbank", "viele Änderungen zu einer zusammenfassen"],
  DB_COMMIT: ["Datenbank", "und festschreiben"],
  DB_CLOSE: ["Datenbank", "die Datenbank schließen"],

  HTTP_TIMEOUT: ["Netz", "wie lange auf eine Antwort gewartet wird"],
  HTTP_GET: ["Netz", "holen und warten, bis es da ist"],
  HTTP_STATUS: ["Netz", "200 = alles gut, 404 = nicht da"],
  HTTP_GET_START: ["Netz", "anstoßen, Nummer zurück, Fenster läuft weiter"],
  HTTP_READY: ["Netz", "ist die Antwort da? (fragt nach, wartet nicht)"],
  HTTP_RESULT: ["Netz", "die Antwort abholen"],
};

// Aus den Programmen: Name -> frühestes Kapitel. Der Ordner `anhang` bleibt
// draußen; er hat keine Kapitelnummer und führt nichts ein.
function ausDemCode() {
  const SCHLUESSEL = new Set([
    "IF", "AND", "OR", "NOT", "FOR", "WHILE", "WEND", "NEXT", "THEN", "ELSE",
    "ELSEIF", "END", "SUB", "FUNCTION", "RETURN", "DIM", "CONST", "STEP",
    "EACH", "IN", "TO", "AS", "TRUE", "FALSE", "IMPORT", "TRY", "CATCH",
    "MOD", "PRINT", "CLASS", "NEW", "SELECT", "CASE", "DO", "LOOP", "UNTIL",
    "EXIT", "CONTINUE",
  ]);
  const erst = {};
  const codeDir = path.join(__dirname, "..", "..", "code");
  for (const ordner of fs.readdirSync(codeDir).sort()) {
    const m = /^kap(\d+)$/.exec(ordner);
    if (!m) continue;
    const kap = parseInt(m[1], 10);
    const voll = path.join(codeDir, ordner);
    for (const datei of fs.readdirSync(voll).filter((f) => f.endsWith(".dh"))) {
      const roh = fs.readFileSync(path.join(voll, datei), "utf8");
      // Erst Zeichenketten weg, dann Kommentare. Die Reihenfolge ist wichtig:
      // in "schluessel = 'runde'" steckt ein Apostroph, der sonst den Rest
      // der Zeile als Kommentar verschluckt und SQL-Woerter durchlaesst.
      const sauber = roh.split("\n")
        .map((z) => z.replace(/"[^"]*"/g, '""').replace(/'.*/, ""))
        .join("\n");
      const merke = (name) => {
        if (SCHLUESSEL.has(name.replace(/\$$/, ""))) return;
        if (erst[name] === undefined || kap < erst[name]) erst[name] = kap;
      };
      for (const t of sauber.matchAll(/\b([A-Z][A-Z0-9_]*\$?)\s*\(/g)) merke(t[1]);
      for (const t of sauber.matchAll(/\b(KEY_[A-Z0-9_]+)\b/g)) merke(t[1]);
    }
  }
  return erst;
}

module.exports = (H) => {
  const erst = ausDemCode();
  // KEY_NAME und KEY_NAME$ sind derselbe Befehl.
  if (erst["KEY_NAME"] !== undefined) {
    erst["KEY_NAME$"] = Math.min(erst["KEY_NAME$"] ?? 99, erst["KEY_NAME"]);
    delete erst["KEY_NAME"];
  }
  const konstanten = Object.keys(erst)
    .filter((n) => /^KEY_/.test(n) && BEFEHLE[n] === undefined)
    .sort((a, b) => erst[a] - erst[b] || a.localeCompare(b));
  for (const n of konstanten) delete erst[n];

  const zeilenFuer = (gruppe) => Object.keys(erst)
    .filter((n) => BEFEHLE[n] && BEFEHLE[n][0] === gruppe)
    .sort((a, b) => erst[a] - erst[b] || a.localeCompare(b))
    .map((n) => [{ text: n, mono: true }, BEFEHLE[n][1], String(erst[n])]);

  const ohne = Object.keys(erst).filter((n) => !BEFEHLE[n]).sort();
  const anzahl = Object.keys(erst).length;
  const codeDir = path.join(__dirname, "..", "..", "code");
  const programme = fs.readdirSync(codeDir)
    .filter((o) => /^kap\d+$/.test(o))
    .reduce((n, o) => n + fs.readdirSync(path.join(codeDir, o))
      .filter((f) => f.endsWith(".dh")).length, 0);

  const bloecke = [
    H.chapter("B · Die Befehle dieses Buchs"),

    H.p("Diese Liste ist keine Auswahl und kein Lehrplan. Sie enthält genau die " + anzahl + " Befehle, die in den " + programme + " Programmen neben diesem Buch wirklich vorkommen — nicht mehr, und vor allem nicht weniger."),

    H.p("Die letzte Spalte sagt, in welchem Kapitel der Befehl zum ersten Mal auftaucht. Wer eine Zeile nicht mehr versteht, weiß damit sofort, wo sie erklärt wird."),

    H.note("Der Anhang schreibt sich selbst. Beim Bauen des Buchs liest er die Programme unter code/ und zieht die Namen und das früheste Kapitel daraus. Eine von Hand gepflegte Liste wäre beim ersten neuen Kapitel veraltet gewesen, und niemand hätte es gemerkt — diese hier kann gar nicht veralten. Aus der Hand kommt nur der erklärende Halbsatz."),

    H.p("Drachenhauch kann erheblich mehr als das hier. Was die Sprache sonst noch mitbringt, steht im „Lehrbuch“ — der Anhang C sagt, wo."),
  ];

  for (const gruppe of GRUPPEN) {
    const zeilen = zeilenFuer(gruppe);
    if (!zeilen.length) continue;
    bloecke.push(H.h2(gruppe));
    bloecke.push(H.table(zeilen, {
      headers: ["Befehl", "Was er tut", "Kap."],
      widths: [3000, 5226, 800], mono: [0],
    }));
  }

  if (konstanten.length) {
    bloecke.push(H.h2("Tastennamen"));
    bloecke.push(H.p("Die Namen der Tasten sind feste Zahlen, keine Aufrufe — deshalb stehen sie ohne Klammern da. Diese hier kommen im Buch vor; es gibt sie für jeden Buchstaben, jede Ziffer und alle Sondertasten."));
    bloecke.push(H.table(
      konstanten.map((n) => [{ text: n, mono: true }, String(erst[n])]),
      { headers: ["Taste", "Kap."], widths: [3000, 800], mono: [0] },
    ));
  }

  if (ohne.length) {
    bloecke.push(H.h2("Noch ohne Beschreibung"));
    bloecke.push(H.p("Diese Befehle stehen in den Programmen, aber noch nicht in der Erklärungsliste dieses Anhangs. Das ist ein Fehler im Buch, kein Fehler in der Sprache."));
    bloecke.push(H.table(
      ohne.map((n) => [{ text: n, mono: true }, String(erst[n])]),
      { headers: ["Befehl", "Kap."], widths: [3000, 800], mono: [0] },
    ));
  }

  return bloecke;
};

// Damit Anhang C dieselbe Zahl nennen kann, ohne sie abzuschreiben: die
// Anzahl der Befehle, die in den Beispielprogrammen wirklich vorkommen.
// Eine zweite, von Hand gepflegte Zahl waere genau die Sorte Angabe, die
// beim naechsten Kapitel still falsch wird.
module.exports.anzahlBefehle = function () {
  const erst = ausDemCode();
  if (erst["KEY_NAME"] !== undefined) {
    erst["KEY_NAME$"] = Math.min(erst["KEY_NAME$"] ?? 99, erst["KEY_NAME"]);
    delete erst["KEY_NAME"];
  }
  for (const n of Object.keys(erst)) if (/^KEY_/.test(n) && BEFEHLE[n] === undefined) delete erst[n];
  return Object.keys(erst).length;
};
