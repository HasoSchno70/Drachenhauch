// Anhang C: SQL kurz gefasst. Die Anweisungen stehen in
// code/anhang_c/beispiele.sql, ihre Ergebnisse in ausgabe.txt daneben --
// sql.dh fuehrt sie der Reihe nach aus, und die Pruefsammlung vergleicht
// seine Ausgabe mit ausgabe.txt. Was hier steht, hat SQLite so geliefert.
const fs = require("fs");
const path = require("path");
const { stueck } = require("../code_lesen");

const DIR = path.join(__dirname, "..", "..", "code", "anhang_c");

// "-- name" leitet einen Block ein; Leerzeilen am Ende fallen weg.
function bloecke(datei, kopf) {
  const b = new Map();
  let name = "";
  for (const z of fs.readFileSync(path.join(DIR, datei), "utf8").replace(/\r/g, "").split("\n")) {
    if (z.startsWith(kopf)) { name = z.slice(kopf.length).trim(); b.set(name, []); continue; }
    if (name) b.get(name).push(z);
  }
  for (const [k, v] of b) { while (v.length && v[v.length - 1].trim() === "") v.pop(); b.set(k, v); }
  return b;
}

const SQL = bloecke("beispiele.sql", "-- ");
const AUS = bloecke("ausgabe.txt", "== ");

// Eine Anweisung und darunter, was sie geliefert hat.
const beispiel = (H, name) => [H.code(SQL.get(name), { out: true }), H.code(AUS.get(name), { out: true })];

module.exports = (H) => [
  H.chapter("C · SQL kurz gefasst"),

  H.p("SQL ist die Sprache, in der man mit einer Datenbank spricht. Dieser Anhang zeigt die Formen, die in der Mediathek vorkommen, an einer kleinen Ausgabe ihrer drei Tabellen. Unter jeder Anweisung steht, was SQLite darauf antwortet; das Programm code/anhang_c/sql.dh führt sie alle aus. Groß und klein ist SQL egal; Schlüsselwörter groß zu schreiben ist nur Brauch."),

  H.h2("Tabellen anlegen und füllen"),

  H.code(SQL.get("anlegen"), { out: true }),

  H.p("Jede Tabelle hat eine Spalte id als INTEGER PRIMARY KEY: Die Datenbank vergibt die Nummer selbst, eins mehr als die größte. REFERENCES sagt, dass in ausleihe.eintrag die id eines Eintrags steht. NOT NULL verbietet eine leere Spalte."),

  H.code(SQL.get("einfuegen"), { out: true }),

  H.p("Texte stehen in einfachen Anführungszeichen; NULL heißt „nichts“ — hier: noch nicht zurück."),

  H.h2("Auswählen, sortieren, suchen"),

  ...beispiel(H, "auswaehlen"),

  H.p("WHERE wählt Zeilen aus, ORDER BY sortiert sie, aufsteigend; DESC dahinter dreht die Richtung um."),

  ...beispiel(H, "suchen"),

  H.p("LIKE vergleicht mit einem Muster: % steht für beliebig viel, _ für genau ein Zeichen. Bei den Buchstaben A bis Z ist LIKE groß und klein egal, bei Umlauten nicht. AND und OR verbinden Bedingungen; Klammern helfen, wenn beides vorkommt."),

  H.h2("Tabellen verbinden"),

  ...beispiel(H, "verbinden"),

  H.p("JOIN holt zu jeder Zeile von ausleihe die passende Zeile aus eintrag und freund; ON sagt, welche passt. AS gibt einer Tabelle einen kurzen Namen. Eine Ausleihe ohne passenden Eintrag fiele heraus."),

  ...beispiel(H, "alle"),

  H.p("LEFT JOIN behält jede Zeile der linken Tabelle, auch ohne Partner; die Spalten der rechten sind dann NULL. Die Bedingung a.zurueck IS NULL steht im ON und nicht im WHERE: Im WHERE fielen alle Einträge heraus, die gerade da sind. COALESCE nimmt den ersten Wert, der nicht NULL ist. Mit NULL vergleicht man über IS NULL, nie mit =."),

  H.h2("Zählen und rechnen"),

  ...beispiel(H, "zaehlen"),

  H.p("GROUP BY fasst die Zeilen mit gleicher Art zusammen, COUNT(*) zählt sie. SUM, AVG, MIN und MAX rechnen ebenso je Gruppe. COUNT(spalte) zählt nur die Zeilen, in denen die Spalte nicht NULL ist."),

  ...beispiel(H, "tage"),

  H.p("Ein Datum steht in SQLite als Text JJJJ-MM-TT. julianday macht daraus eine Zahl von Tagen, die man abziehen kann; CAST(… AS INTEGER) schneidet die Nachkommastellen ab. In der Mediathek steht statt des festen Tags julianday('now', 'localtime')."),

  ...beispiel(H, "monate"),

  H.p("strftime schreibt ein Datum in einer Form: %Y das Jahr, %m den Monat, %d den Tag."),

  H.h2("Ändern und löschen"),

  ...beispiel(H, "aendern"),

  ...beispiel(H, "loeschen"),

  H.p("UPDATE und DELETE ohne WHERE treffen jede Zeile der Tabelle. Eine Unterabfrage in Klammern liefert eine Liste, gegen die IN und NOT IN prüfen; hier fliegt nur heraus, was nie verliehen war:"),

  ...beispiel(H, "danach"),

  H.h2("Aus Drachenhauch"),

  H.p("Werte, die vom Benutzer kommen, gehören nie in den Text der Anweisung, sondern hinter sie, mit ? als Platzhalter. Dann darf ein Titel ein Hochkomma enthalten (Kapitel 4), und niemand kann über ein Feld eigenes SQL einschleusen:"),

  H.code(stueck("anhang_c/sql.dh", "DIM titel AS STRING", "PRINT DB_LAST_ROWID")),
  H.code(AUS.get("platzhalter"), { out: true }),

  H.p("Mehrere Änderungen, die nur zusammen gelten sollen, stehen in einer Transaktion. DB_COMMIT schreibt sie fest, DB_ROLLBACK nimmt alle zurück:"),

  H.code(stueck("anhang_c/sql.dh", "DB_BEGIN(db)", "IF DB_NEXT(r)")),
  H.code(AUS.get("transaktion"), { out: true }),

  H.table([
    [{ text: "DB_EXEC(db, sql, werte …)", mono: true }, "ändern: liefert die Zahl der geänderten Zeilen"],
    [{ text: "DB_QUERY(db, sql, werte …)", mono: true }, "abfragen: liefert ein Ergebnis"],
    [{ text: "DB_NEXT(r)", mono: true }, "zur nächsten Zeile; FALSE, wenn keine mehr kommt"],
    [{ text: "DB_GET_STRING(r, i) / DB_GET_INT(r, i)", mono: true }, "Spalte i der Zeile, ab 0"],
    [{ text: "DB_IS_NULL(r, i)", mono: true }, "steht dort NULL?"],
    [{ text: "DB_LAST_ROWID(db)", mono: true }, "die id der zuletzt eingefügten Zeile"],
  ], { headers: ["Befehl", "was er tut"], widths: [4400, 4626] }),
];
