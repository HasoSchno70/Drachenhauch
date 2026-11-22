const { zeilen, ohneKopf, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Daten, die bleiben", 4),

  H.h2("Was wir bauen"),

  H.figure("kap04_2_mediathek.png", "Die Mediathek sieht aus wie in Kapitel 3. Neu ist, was man nicht sieht: Die zehn Einträge kommen aus der Datei mediathek.db neben dem Programm.", 460, 315),

  H.p("Bis jetzt vergaß die Mediathek alles, sobald man sie schloss: Was im Feld stand, lag im Speicher, und der wird beim Beenden geräumt. Nach diesem Kapitel liegen die Einträge in einer Datenbank, einer einzigen Datei neben dem Programm. Was man aufnimmt oder entfernt, ist beim nächsten Start noch so."),

  H.h2("Der Plan"),

  H.p("Eine Datenbank ist mehr als eine Datei mit Text. Sie kennt die Form der Daten — eine Tabelle eintrag mit den Spalten titel, urheber, art und jahr —, sie findet schnell, was man sucht, und sie passt darauf auf, dass eine Änderung entweder ganz geschieht oder gar nicht, auch wenn mittendrin der Strom ausfällt. Drachenhauch bringt SQLite mit: Die ganze Datenbank ist eine Datei, es gibt keinen Server, den man einrichten müsste."),

  H.p("Mit der Datenbank spricht man in SQL, einer eigenen kleinen Sprache für Daten. Dieses Kapitel braucht davon vier Sätze: eine Tabelle anlegen (CREATE TABLE), eine Zeile hinzufügen (INSERT), Zeilen lesen (SELECT) und Zeilen entfernen (DELETE). Das Programm schickt diese Sätze als Text an die Datenbank, und die Werte — einen Titel, ein Jahr — schickt es getrennt davon hinterher."),

  H.h2("Schritt 1: eine Datei, die sich erinnert"),

  H.p("Zuerst ohne Fenster, damit nichts vom Wesentlichen ablenkt:"),

  H.code(ohneKopf("kap04/1_datei.dh")),

  H.code(["2  Metropolis (1927)", "1  Momo (1973)"], { out: true }),

  H.pmix([["DB_OPEN", true], " öffnet die Datei mediathek.db und legt sie an, wenn es sie noch nicht gibt. Der Griff, den es liefert, hat den Typ ", ["DB_CONN", true], " — eine Verbindung zur Datenbank. ", ["DB_EXEC", true], " schickt einen SQL-Satz, der nichts zurückliefert. Der erste beschreibt die Tabelle: eine Spalte id, die die Datenbank selbst hochzählt (INTEGER PRIMARY KEY), ein Titel, der nicht fehlen darf (NOT NULL), und die drei anderen Angaben. IF NOT EXISTS legt sie nur an, wenn es sie noch nicht gibt."]),

  H.pmix(["Im INSERT stehen für die Werte vier Fragezeichen. Das sind ", ["Parameter", true], ": Platzhalter, deren Werte hinter dem SQL-Text als eigene Argumente mitkommen, in derselben Reihenfolge. ", ["DB_QUERY", true], " schickt ein SELECT und liefert ein Ergebnis vom Typ ", ["DB_RESULT", true], ". ", ["DB_NEXT", true], " rückt darin eine Zeile weiter und ist FALSE, wenn keine mehr kommt; ", ["DB_GET_INT", true], " und ", ["DB_GET_STRING", true], " lesen eine Spalte der aktuellen Zeile, gezählt ab 0."]),

  H.p("Startet man das Programm ein zweites Mal, stehen vier Zeilen da: Metropolis mit 2 und 4, Momo mit 1 und 3. Die ersten beiden waren noch in der Datei, und die zweiten kamen dazu. Das ist der ganze Unterschied zu einem Feld im Speicher — und der Grund, warum die Mediathek gleich prüfen wird, ob die Datenbank schon etwas enthält, bevor sie Beispiele hineinschreibt."),

  H.h2("Schritt 2: die Mediathek in der Datenbank"),

  H.p("Die Mediathek aus Kapitel 3 behält ihr Feld von Einträgen und ihre Tabelle. Neu ist, dass das Feld beim Start aus der Datenbank gefüllt wird und jede Änderung zuerst in die Datenbank geht. Dafür bekommt jeder Eintrag seine Kennung, die id aus der Datenbank:"),

  H.code(stueck("kap04/eintrag.dh", "' Die Kennung, die", "DIM id AS INTEGER")),

  H.p("Laden heißt: alle Zeilen der Tabelle lesen und aus jeder einen Eintrag machen:"),

  H.code(stueck("kap04/2_mediathek.dh", "FUNCTION laden", "END FUNCTION")),

  H.pmix(["Ein Jahr darf fehlen. In der Datenbank steht dann NULL — „kein Wert“, nicht 0 und nicht der leere Text. ", ["DB_IS_NULL", true], " fragt danach, und dann bleibt das Jahr des Eintrags leer. Beim Speichern geht es umgekehrt: Ein leeres Jahr wird ", ["NIL", true], ", und das wird in der Datenbank zu NULL:"]),

  H.code(stueck("kap04/2_mediathek.dh", "SUB speichern", "END SUB")),

  H.pmix([["DB_LAST_ROWID", true], " liefert die Kennung, die die Datenbank der eben eingefügten Zeile gegeben hat. Der Eintrag merkt sie sich — sie ist das, woran er später wiedergefunden wird. Beim allerersten Start ist die Datenbank leer; dann kommen die Beispiele hinein:"]),

  H.code(stueck("kap04/2_mediathek.dh", "DIM mediathek AS ARRAY OF Eintrag", "END IF")),

  H.p("Aufnehmen ruft jetzt speichern, bevor der Eintrag ins Feld und in die Tabelle kommt. Und beim Entfernen geht für jede gewählte Zeile ein DELETE an die Datenbank — mit der Kennung des Eintrags, nicht mit seiner Zeilennummer:"),

  H.code(stueck("kap04/2_mediathek.dh", "FOR EACH z IN zeilen", "NEXT")),

  H.p("Ganz am Ende, nach der Schleife, schließt DB_CLOSE(db) die Verbindung."),

  H.h2("Schritt 3: ganz oder gar nicht"),

  H.p("Wer drei Einträge wählt und auf „Entfernen“ klickt, meint: diese drei. Geht beim zweiten DELETE etwas schief, wäre einer weg und zwei nicht — und die Tabelle zeigte etwas anderes als die Datei. Dagegen hilft eine Transaktion:"),

  H.code(stueck("kap04/3_transaktion.dh", "' Erst die Datenbank, ganz", "END TRY")),

  H.pmix([["DB_BEGIN", true], " beginnt die Transaktion. Alles danach ist vorläufig: Die Datenbank führt die DELETEs aus, schreibt sie aber erst mit ", ["DB_COMMIT", true], " endgültig fest. Geht vorher etwas schief, springt das Programm in den CATCH-Teil, und ", ["DB_ROLLBACK", true], " nimmt alles seit DB_BEGIN zurück. Erst wenn die Datenbank zugestimmt hat, kommen Feld und Tabelle an die Reihe — so zahlt sich die Regel aus Kapitel 3 aus, erst die Daten und dann die Anzeige zu ändern: Hat die Datenbank abgelehnt, zeigt die Tabelle noch, was wirklich gespeichert ist."]),

  H.p("Auch die Beispiele beim ersten Start kommen jetzt in einer Transaktion hinein:"),

  H.code(stueck("kap04/3_transaktion.dh", "IF LEN(mediathek) = 0 THEN", "END IF")),

  H.p("Hier geht es weniger ums Ganz-oder-gar-nicht als um Tempo, dazu gleich mehr. Das ganze Programm:"),

  H.code(zeilen("kap04/3_transaktion.dh")),

  H.h2("Warum so?"),

  H.h3("Nie Werte ins SQL kleben"),

  H.p("Statt der Fragezeichen könnte man den Titel einfach in den SQL-Text schreiben: \"VALUES ('\" + t + \"')\". Für Momo geht das. Für Sgt. Pepper's Lonely Hearts Club Band nicht — das Apostroph im Titel beendet für die Datenbank den Text, und der Rest ist kein SQL mehr (siehe „Was schiefgehen kann“). Ein böswilliger Titel könnte an dieser Stelle sogar ganze SQL-Sätze einschleusen; das heißt SQL-Injection und ist einer der häufigsten Fehler in Programmen, die mit Datenbanken arbeiten."),

  H.p("Mit Parametern kann das nicht passieren. Der SQL-Text und die Werte reisen getrennt, und die Datenbank weiß, dass ein Wert ein Wert ist — welche Zeichen er auch enthält. Darum gilt ohne Ausnahme: Was von außen kommt, geht als Parameter in die Datenbank."),

  H.h3("Warum eine Transaktion so viel schneller ist"),

  H.p("Ohne DB_BEGIN ist jedes einzelne INSERT seine eigene kleine Transaktion, und am Ende jeder Transaktion wartet die Datenbank, bis das Betriebssystem bestätigt, dass die Daten wirklich auf der Platte stehen. Das ist der Grund, warum nach einem Absturz nichts halb geschrieben ist — und es kostet Zeit. Auf dem Rechner, auf dem dieses Buch entstand, brauchten 500 INSERTs einzeln 0,89 Sekunden, in einer Transaktion 0,003. Für zehn Beispiele merkt man das kaum; für den Import einer Liste mit tausend Platten sehr wohl."),

  H.h3("Kennungen statt Zeilennummern"),

  H.p("Die Zeilennummer eines Eintrags ändert sich, sobald davor einer entfernt wird; seine Kennung in der Datenbank nie. Darum löscht das DELETE nach id und nicht nach Position, und darum merkt sich jeder Eintrag seine id. Feld und Tabelle stimmen in der Reihenfolge weiter überein wie in Kapitel 3 — aber die Verbindung zur Datei ist die Kennung."),

  H.h2("Was schiefgehen kann"),

  H.h3("Ein Apostroph im Wert"),

  H.code(stueck("kap04/fehler_apostroph.dh", "DIM t AS STRING", "DB_EXEC(db, \"INSERT")),
  H.code(["Laufzeitfehler in fehler_apostroph.dh:7: DB_EXEC: near \"s\": syntax",
          "error in INSERT INTO eintrag (titel) VALUES ('Sgt. Pepper's Lonely",
          "Hearts Club Band') at offset 49 -- steht ein ' in einem",
          "eingeklebten Wert? Werte mit ? uebergeben: DB_EXEC(db, \"...",
          "VALUES (?)\", wert)"], { out: true }),

  H.p("Die Meldung kommt zum Teil von SQLite selbst, auf Englisch: In der Nähe von „s“ ist ein Syntaxfehler, an Stelle 49 des SQL-Texts. Dort, hinter Pepper', ist für die Datenbank der Text zu Ende. Den Rest der Meldung hängt Drachenhauch an, weil ein Syntaxfehler zusammen mit einer ungeraden Zahl von Apostrophen fast immer genau das bedeutet. Die Abhilfe ist die aus Schritt 1."),

  H.h3("Die Datei ist gesperrt"),

  H.p("Eine SQLite-Datei kann von mehreren Programmen gelesen werden, aber nur eines darf gleichzeitig schreiben. Wer schreiben will, während ein anderes Programm mitten in einer Transaktion steckt, wartet:"),

  H.code(stueck("kap04/fehler_gesperrt.dh", "DIM b AS DB_CONN", "DB_EXEC(a, \"INSERT")),
  H.code(["Laufzeitfehler in fehler_gesperrt.dh:11: DB_EXEC: database is locked",
          "-- eine andere Verbindung schreibt gerade in die Datei (ein zweites",
          "Programm oder eine Transaktion ohne DB_COMMIT); nach 5 Sekunden",
          "aufgegeben"], { out: true }),

  H.p("Fünf Sekunden lang versucht es die Datenbank immer wieder, dann gibt sie auf. In der Praxis heißt das: Ein anderes Programm hat die Datei offen und schreibt gerade — oder das eigene Programm hat eine Transaktion begonnen und nie beendet. Die Mediathek aus Schritt 3 übersteht das: Ihr CATCH nimmt die Transaktion zurück, die Statuszeile sagt „Nichts entfernt“ mit dieser Meldung, und Feld und Tabelle bleiben, wie sie waren."),

  H.h3("Der vergessene Commit"),

  H.code(zeilen("kap04/fehler_commit.dh")),
  H.code(["aufgenommen", "0 Eintraege in der Datei"], { out: true }),

  H.p("Es gibt keine Fehlermeldung. Das INSERT hat funktioniert — innerhalb der Transaktion. DB_CLOSE hat die Verbindung geschlossen, ohne dass jemand COMMIT gesagt hat, und eine nicht festgeschriebene Transaktion verwirft die Datenbank. Wer DB_BEGIN schreibt, schreibt darum gleich das DB_COMMIT dazu, und das DB_ROLLBACK für den Fall, dass etwas schiefgeht."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Beende die Mediathek, lösche die Datei mediathek.db und starte neu. Was siehst du, und welche Kennungen haben die Einträge?"),
  H.bulletRich("Drehen 2. ", "Nimm IF NOT EXISTS aus dem CREATE TABLE in 1_datei.dh heraus und starte das Programm zweimal. Was passiert beim zweiten Mal?"),
  H.bulletRich("Drehen 3. ", "Ändere in laden ORDER BY id in ORDER BY titel. Was ändert sich in der Tabelle, und entfernt „Entfernen“ trotzdem die richtigen Einträge?"),
  H.bulletRich("Erweitern 1. ", "Ausleihen: Eine zweite Tabelle ausleihe merkt sich, an wen ein Eintrag verliehen ist. Ein Feld und zwei Knöpfe — „Verleihen“ und „Zurück“ — ändern das für die gewählte Zeile, und eine fünfte Spalte zeigt es. Wer einen verliehenen Eintrag entfernt, entfernt seine Ausleihe mit."),
  H.bulletRich("Erweitern 2. ", "Entfernen mit Nachfrage: Der erste Klick auf „Entfernen“ ändert die Beschriftung in „Wirklich?“ und sagt in der Statuszeile, wie viele es sind; erst ein zweiter Klick entfernt. Jede andere Bedienung nimmt die Frage zurück."),
  H.bulletRich("Erfinden. ", "Eine kleine Datenbank für etwas aus deinem Leben: ein Tagebuch mit Datum und Text, die Ergebnisse eines Spieleabends, die Kilometer jeder Radtour. Mit CREATE TABLE IF NOT EXISTS, Parametern und einem SELECT, das etwas zusammenzählt — SELECT SUM(km) FROM tour."),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Modul db: ", "alle Befehle, Typen in SQLite, Transaktionen, und wie dasselbe Programm mit PostgreSQL oder MySQL spricht."),
  H.bulletRich("Handbuch, Modul gui → An eine Datenbank hängen: ", "eine Tabelle, die ihre Zeilen selbst aus einer Abfrage holt — die Abkürzung, die das Studio in Teil III nimmt."),
  H.bulletRich("Die SQLite-Dokumentation (sqlite.org/lang.html): ", "jeder SQL-Satz, den SQLite versteht, mit Beispielen."),
];
