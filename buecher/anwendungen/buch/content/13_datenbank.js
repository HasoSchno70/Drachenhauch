const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.part("Teil III · Daten richtig"),
  H.chapter("Das Datenbank-Element", 13),

  H.h2("Was wir bauen"),

  H.figure("kap13_spalten.png", "Der Reiter der Datenbank Mediathek: links ihre drei Tabellen, rechts die Spalten von ausleihe. Zwei davon verweisen auf andere Tabellen — mit einer Regel, was beim Löschen geschieht.", 480, 378),

  H.p("Seit Kapitel 10 lebt die Mediathek im Speicher. Was man im Programm ändert, ist beim nächsten Start vergessen, und wer Tetris an Lena verleiht, hat es nur bis zum Schließen verliehen. In Teil I stand alles in einer SQLite-Datei; jetzt zieht die Mediathek mit dieser Datei ins Studio um. Die Einträge aus Teil I kommen mit, und zwei Tabellen kommen dazu: die Freunde und die Ausleihen."),

  H.h2("Der Plan"),

  H.p("Ein Datenbank-Element hat zwei Teile. Der Aufbau — welche Tabellen, welche Spalten, welche Art — steht im Projekt, in Mediathek.dhdb. Die Daten stehen in einer SQLite-Datei im Projektordner, hier mediathek.db. Im Reiter der Datenbank sieht man beides: unter Spalten den Aufbau einer Tabelle, unter Daten ihre Zeilen. Jede Spalte hat eine Art:"),

  H.table([
    ["Text", "TEXT", "wie er ist"],
    ["Ganze Zahl", "INTEGER", "-4"],
    ["Kommazahl", "REAL", "2,5"],
    ["Geld", "INTEGER in Cent", "19,99 (steht als 1999 in der Datei)"],
    ["Datum", "TEXT", "2026-11-03"],
    ["Ja/Nein", "INTEGER 0/1", "ja oder nein"],
    ["Bild", "BLOB", "die Bytes einer Bilddatei"],
  ], { headers: ["Art", "in SQLite", "getippt"], widths: [2000, 2600, 4426] }),

  H.p("Die Spalte id hat jede Tabelle von selbst. Eine Spalte kann Pflicht sein (sie darf nicht leer bleiben) und Eindeutig (kein Wert zweimal). Und sie kann auf eine andere Tabelle verweisen: Dann hält sie die id einer Zeile dort. Im Programm ist das Element eine offene Verbindung unter seinem Namen, Mediathek — DB_QUERY(Mediathek, …) wie in Teil I, nur ohne DB_OPEN."),

  H.h2("Schritt 1: die Datei aus Teil I übernehmen"),

  H.figure("kap13_daten.png", "Die Tabelle eintrag unter Daten: die fünf Einträge aus Teil I, jeder mit seiner id.", 480, 378),

  H.p("Projekt → Datenbank aus Datei übernehmen fragt nach einer Datei; es ist die mediathek.db aus Kapitel 6. Das Studio kopiert sie in den Projektordner und liest ihren Aufbau: Die Tabelle eintrag hat titel (Text, Pflicht — in Teil I NOT NULL), urheber und art (Text) und jahr (Ganze Zahl). Das Element heißt nach der Datei Mediathek. Unter Daten stehen die Einträge, die man in Teil I aufgenommen hat; in einer Zelle lässt sich ein Wert ändern, und die Änderung steht sofort in der Datei."),

  H.h2("Schritt 2: Freunde und Ausleihen"),

  H.p("„Tabelle …“ legt eine neue Tabelle an, „Spalte hinzufügen …“ eine Spalte. Die Art wählt man in ihrer Zelle, Pflicht und Eindeutig sind Kästchen. Zwei Tabellen kommen dazu:"),

  H.table([
    [{ text: "freund.name", mono: true }, "Text", "Pflicht, Eindeutig — jeder Freund einmal"],
    [{ text: "freund.telefon", mono: true }, "Text", ""],
    [{ text: "ausleihe.eintrag", mono: true }, "Ganze Zahl", "Pflicht, Verweis auf eintrag, beim Löschen mitlöschen"],
    [{ text: "ausleihe.freund", mono: true }, "Ganze Zahl", "Pflicht, Verweis auf freund, beim Löschen verbieten"],
    [{ text: "ausleihe.von", mono: true }, "Datum", "Pflicht"],
    [{ text: "ausleihe.zurueck", mono: true }, "Datum", "leer, solange der Eintrag weg ist"],
  ], { headers: ["Spalte", "Art", "dazu"], widths: [2800, 1600, 4626] }),

  H.p("Eine Ausleihe ist eine Zeile: welcher Eintrag, an welchen Freund, von wann bis wann. Sie nennt den Eintrag und den Freund nicht beim Namen, sondern bei ihrer id — das ist der Verweis. „Beim Löschen“ sagt, was mit der Ausleihe geschieht, wenn ihr Eintrag oder ihr Freund gelöscht wird. Mit dem Eintrag geht auch seine Geschichte; einen Freund aber, der noch etwas hat, darf man nicht löschen."),

  H.h2("Schritt 3: der Code"),

  H.p("Was die Mediathek mit der Datenbank tut, sammelt ein neues Modul, Bestand. Beim ersten Start ohne Datei aus Teil I ist die Tabelle leer; dann kommen die Beispiele hinein, wie in Teil I:"),

  H.code(stueck("kap13/mediathek/Bestand.dh", "SUB einrichten", "END SUB")),

  H.p("Wer einen Eintrag gerade hat, steht nicht in der Tabelle eintrag, sondern ergibt sich aus zwei anderen: In ausleihe steht die id des Freundes, in freund sein Name. Eine Abfrage in der Abfrage holt ihn:"),

  H.code(stueck("kap13/mediathek/Bestand.dh", "FUNCTION verliehenAn$", "END FUNCTION")),

  H.p("Verleihen legt eine Zeile in ausleihe an. Den Freund sucht freundId beim Namen und legt ihn an, wenn es ihn noch nicht gibt; das heutige Datum rechnet die Datenbank selbst:"),

  H.code(stueck("kap13/mediathek/Bestand.dh", "SUB verleihen", "END SUB")),

  H.p("Das Hauptfenster ruft in Geoeffnet einrichten() und alleEintraege() statt beispiele(), nach dem Bearbeiten sichern(…) und beim Entfernen entfernen(…). Im Ausleihfenster rufen Verleihen und Zurück verleihen(…) und zurueckgeben(…) auf. Was das Programm schreibt, steht danach im Reiter der Datenbank unter Daten."),

  H.h2("Warum so?"),

  H.h3("Verweise statt Namen in jeder Zeile"),

  H.p("Man könnte in ausleihe einfach „Lena“ hineinschreiben. Dann stünde der Name in jeder Ausleihe, die Lena je hatte. Heißt sie später „Lena K.“, weil eine zweite Lena dazukommt, müsste man jede dieser Zeilen ändern — und eine vergessene gehörte niemandem mehr. Mit einem Verweis steht der Name genau einmal, in freund, und eine Ausleihe zeigt nur auf die Zeile. Dazu kommt, was ein Name nicht kann: Ein Verweis auf eine Zeile, die es nicht gibt, lehnt die Datenbank ab, und was beim Löschen geschieht, ist eine Regel statt einer Frage an jedes Programm, das die Datei benutzt."),

  H.h3("Mitlöschen oder verbieten"),

  H.p("Die beiden Verweise von ausleihe haben verschiedene Regeln, weil sie Verschiedenes bedeuten. Wird ein Eintrag entfernt, ist auch seine Ausleihgeschichte nichts mehr wert: Sie geht mit. Ein Freund aber, der noch Tetris hat, soll nicht verschwinden, nur weil jemand die Liste aufräumt — erst kommt das Spiel zurück. Die dritte Regel, leeren, passt zu einem Verweis, der fehlen darf; bei einer Pflichtspalte lässt das Studio sie nicht zu."),

  H.h2("Was schiefgehen kann"),

  H.h3("Ein Freund, der noch etwas hat"),

  H.p("Wer im Reiter der Datenbank Lena aus freund löscht, solange Tetris bei ihr ist, bekommt in der Statuszeile:"),

  H.code(["Auf die Zeile zeigt noch eine Zeile in ausleihe (Spalte freund) --",
          "erst dort löschen, oder dort „Beim Löschen“ auf „mitlöschen“ stellen."], { out: true }),

  H.p("Die Datei bleibt, wie sie war. Ein Programm, das dasselbe mit DB_EXEC versucht, bekommt dieselbe Auskunft hinter „FOREIGN KEY constraint failed“ als Laufzeitfehler."),

  H.h3("Zwei Dateien mit demselben Namen"),

  H.p("Übernehmen kopiert die Datei in den Projektordner. Das Programm aus Teil I arbeitet danach weiter auf der alten Datei in code/kap06, die Mediathek im Studio auf der Kopie — wer das eine startet und im anderen nachsieht, findet seine Änderung nicht. Ab hier gilt die Datei im Projektordner."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Lösche im Reiter der Datenbank unter Daten den Freund Lena, solange Tetris bei ihr ist. Was steht danach in der Datei?"),
  H.bulletRich("Drehen 2. ", "Entferne im Programm Tetris. Was steht danach in ausleihe?"),
  H.bulletRich("Drehen 3. ", "Stelle bei ausleihe.freund „Beim Löschen“ auf leeren. Was sagt das Studio?"),
  H.bulletRich("Erweitern 1. ", "Trage im Reiter der Datenbank bei Lena eine Telefonnummer ein. Das Ausleihfenster soll sie zeigen: „Tetris ist bei Lena (0151 2345678).“"),
  H.bulletRich("Erweitern 2. ", "Schlagwörter: Ein Eintrag kann mehrere haben, ein Schlagwort gehört zu mehreren Einträgen. Bücher bekommen „Kinderbuch“, alles vor 1960 „Klassiker“, und ein Klick auf eine Zeile zeigt sie in der Statuszeile: „Emil und die Detektive: Kinderbuch, Klassiker“."),
  H.bulletRich("Erfinden. ", "Nimm eine Sammlung, die du kennst — Rezepte, Pflanzen, Werkzeug —, und entwirf ihre Tabellen im Studio, mit mindestens einem Verweis. Überlege für jeden Verweis, was beim Löschen geschehen soll."),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Drachenhauch Studio → Datenbanken: ", "alle Arten, Übernehmen, Verweise und Löschregeln, was ein gebautes Programm mit der Datei tut."),
  H.bulletRich("Handbuch, Modul db: ", "DB_QUERY, DB_EXEC, Transaktionen und was ein Verweis in SQLite ist."),
];
