const fs = require("fs");
const path = require("path");
const { zeilen, stueck } = require("../code_lesen");

// Das SQL einer Abfrage, wie es im Datenbank-Element steht -- aus der
// Datei, nicht abgeschrieben.
function sql(rel, name) {
  const d = JSON.parse(fs.readFileSync(path.join(__dirname, "..", "..", "code", rel), "utf8"));
  return d.abfragen.find((a) => a.name === name).sql.split("\n");
}

module.exports = (H) => [
  H.chapter("Abfragen, die rechnen", 15),

  H.h2("Was wir bauen"),

  H.figure("kap15_ueberfaellig.png", "Ansicht → Überfällig: Tetris ist seit 40 Tagen bei Lena. Gerechnet hat das die Datenbank, in einer Abfrage.", 440, 293),

  H.p("Die Mediathek bekommt zurück, was ihr in Kapitel 14 gefehlt hat: die Spalte „Verliehen an“. Dazu zwei neue Fenster im Menü Ansicht — welche Ausleihen länger als vier Wochen laufen, und wie viele Einträge es je Art gibt —, und das Ausleihfenster sagt, seit wann ein Eintrag weg ist. Gerechnet wird dabei fast nichts im Programm: Das tun Abfragen, benannte SELECT im Datenbank-Element."),

  H.h2("Der Plan"),

  H.p("Eine Abfrage legt man im Reiter der Datenbank mit „Abfrage …“ an. Links steht sie unter den Tabellen, rechts ihr SQL und darunter das Ergebnis; gesichert wird das SQL erst, wenn es an der Datei läuft. Im Programm ist eine Abfrage zweierlei:"),

  H.bulletRich("eine Datenquelle ", "wie eine Tabelle — für eine Tabelle, Liste oder Klappliste im Inspektor. Hat sie eine Spalte id, liefert IdVon die id der gewählten Zeile."),
  H.bulletRich("ein Aufruf ", "im Code: Abfrage(\"bestand\") liefert ein DB_RESULT wie DB_QUERY."),

  H.p("Für die Mediathek braucht es drei Werkzeuge von SQL: JOIN holt Zeilen aus mehreren Tabellen zusammen, GROUP BY fasst Zeilen zusammen und zählt, und die Datumsfunktionen von SQLite rechnen mit Tagen."),

  H.h2("Schritt 1: wer hat was"),

  H.figure("kap15_abfrage.png", "Die Abfrage bestand im Reiter der Datenbank: oben ihr SQL, darunter das Ergebnis — Tetris mit Lena in der Spalte verliehen.", 480, 378),

  H.p("Die Abfrage bestand liefert die Einträge und, wo es einen gibt, den Freund, der einen gerade hat:"),

  H.code(sql("kap15/mediathek/Mediathek.dhdb", "bestand"), { out: true }),

  H.p("FROM eintrag AS e nimmt die Einträge und nennt die Tabelle kurz e. LEFT JOIN ausleihe AS a hängt an jeden Eintrag die Ausleihe, die auf ihn zeigt und noch nicht zurück ist — LEFT heißt: Ein Eintrag ohne solche Ausleihe bleibt trotzdem, mit leeren Spalten von a. Der zweite LEFT JOIN holt zur Ausleihe den Freund. Was SELECT nennt, sind die Spalten des Ergebnisses; AS gibt ihnen ihre Namen."),

  H.p("Die Tabelle des Hauptfensters bekommt die Datenquelle Mediathek.bestand, die Angezeigten Spalten titel;urheber;art;jahr;verliehen und einen fünften Kopf, „Verliehen an“. Sortierung und Suchfeld gelten wie bei einer Tabelle — gesucht wird jetzt auch im Namen. Weil eine Abfrage nicht die Tabelle des Fensters ist, lädt ein Klick keine Zeile; die gewählte id liefert IdVon:"),

  H.code(stueck("kap15/mediathek/Hauptfenster.dh", "SUB knopfBearbeiten_Klick", "END SUB")),

  H.h2("Schritt 2: was überfällig ist"),

  H.p("Überfällig ist eine Ausleihe, die nicht zurück ist und vor mehr als 28 Tagen begann. SQLite rechnet mit Daten im Format JJJJ-MM-TT: date('now', 'localtime', '-28 days') ist das Datum vor vier Wochen, und julianday macht aus einem Datum eine Tageszahl, mit der man rechnen kann:"),

  H.code(sql("kap15/mediathek/Mediathek.dhdb", "ueberfaellig"), { out: true }),

  H.p("Hier steht JOIN ohne LEFT: Zu einer Ausleihe gibt es immer einen Eintrag und einen Freund, und eine Zeile ohne beide wäre ohnehin falsch. Das Fenster Überfällig hat nur eine Tabelle mit dieser Datenquelle, Sortieren nach tage, absteigend — die längste zuerst — und einen Knopf zum Schließen."),

  H.h2("Schritt 3: zählen je Art"),

  H.code(sql("kap15/mediathek/Mediathek.dhdb", "statistik"), { out: true }),

  H.p("GROUP BY e.art macht aus allen Zeilen einer Art eine. COUNT(*) zählt, wie viele es waren; COUNT(a.id) zählt nur die, bei denen a.id nicht leer ist — also die verliehenen. Das Fenster Statistik zeigt das Ergebnis: Buch 2, Film 1, Musik 1, Spiel 1, und bei Spiel ist eines verliehen."),

  H.h2("Schritt 4: rechnen mit dem Zeit-Modul"),

  H.p("Manchmal ist ein Datum im Programm gefragt: Das Ausleihfenster soll sagen „Tetris ist seit 40 Tagen bei Lena.“ Das Datum steht als Text in der Datei; ZEIT_PARSE liest es, ZEIT_DIFF rechnet den Abstand zu jetzt in Sekunden (IMPORT \"zeit\" steht oben im Modul):"),

  H.code(stueck("kap15/mediathek/Bestand.dh", "FUNCTION seitTagen", "END FUNCTION")),

  H.p("Nach Verleihen und Zurück lädt das Ausleihfenster die Tabelle des Hauptfensters neu, damit die Spalte „Verliehen an“ stimmt. Die gewählte Zeile bleibt dabei gewählt, auch an einer Abfrage."),

  H.h2("Warum so?"),

  H.h3("Warum die Datenbank rechnet"),

  H.p("Die Statistik ließe sich auch im Programm zählen: alle Einträge laden, alle offenen Ausleihen laden, in einer Schleife je Art hochzählen. Das wären zwei Abfragen, eine Schleife und eine Tabelle für die Zwischenstände — und es wäre langsamer, je größer die Mediathek wird, weil jede Zeile erst ins Programm muss. Die Datenbank tut dasselbe in einer Anweisung, dort, wo die Daten liegen, und gibt nur das Ergebnis heraus: vier Zeilen."),

  H.p("Dazu kommt, dass eine Abfrage überall gleich rechnet. Die Datenquelle der Tabelle, das Fenster Überfällig und ein Bericht in Kapitel 17 können dieselbe Abfrage benutzen. Was „überfällig“ heißt, steht an genau einer Stelle."),

  H.h2("Was schiefgehen kann"),

  H.h3("Die Bedingung vergessen"),

  H.p("Wer in bestand „AND a.zurueck IS NULL“ weglässt, hängt an einen Eintrag jede Ausleihe, die er je hatte. Momo steht dann bei Jonas, obwohl Jonas ihn längst zurückgegeben hat — und hätte Momo zwei Ausleihen hinter sich, stünde er zweimal in der Tabelle. Eine Abfrage, die zu viele Zeilen liefert, meldet nichts; man muss das Ergebnis im Reiter der Datenbank ansehen."),

  H.h3("Ein Datum, das kein JJJJ-MM-TT ist"),

  H.p("SQLite vergleicht Datumsangaben als Text. Das geht gut, solange sie alle als 2026-11-03 dastehen: Dann ist der Text, der früher kommt, auch das frühere Datum. Steht irgendwo 3.11.2026, stimmt der Vergleich nicht mehr, und date und julianday liefern leer. Die Spaltenart Datum im Studio schreibt darum immer JJJJ-MM-TT."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Ersetze in bestand das erste LEFT JOIN durch JOIN. Was zeigt die Tabelle?"),
  H.bulletRich("Drehen 2. ", "Schiebe in bestand „a.zurueck IS NULL“ aus dem ON in ein WHERE am Ende. Wer fehlt danach, und warum?"),
  H.bulletRich("Drehen 3. ", "Mache in ueberfaellig aus -28 days -50 days. Was zeigt das Fenster?"),
  H.bulletRich("Erweitern 1. ", "Ein Fenster „Am häufigsten verliehen“ mit den zehn Titeln, die am öftesten ausgeliehen wurden."),
  H.bulletRich("Erweitern 2. ", "Ein Fenster „Ausleihen je Monat“: in jeder Zeile ein Monat (JJJJ-MM) und wie viele Ausleihen darin begannen."),
  H.bulletRich("Erfinden. ", "Stelle deiner Sammlung aus Kapitel 13 drei Fragen, die nur eine Abfrage beantworten kann, und schreibe für jede eine."),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Drachenhauch Studio → Datenbanken → Abfragen: ", "anlegen, ausführen, als Datenquelle und im Code."),
  H.bulletRich("Handbuch, Modul zeit: ", "ZEIT_PARSE, ZEIT_DIFF, ZEIT_PLUS und die übrigen Funktionen für Datum und Uhrzeit."),
];
