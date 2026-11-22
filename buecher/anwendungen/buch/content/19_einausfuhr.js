const fs = require("fs");
const path = require("path");
const { stueck } = require("../code_lesen");

const A = "kap19/mediathek/Austausch.dh";

// Die alte Liste, wie sie im Ordner liegt: Windows-1252, Zeilenende CR LF.
function liste() {
  const p = path.join(__dirname, "..", "..", "code", "kap19", "alte_liste.csv");
  return fs.readFileSync(p, "latin1").replace(/\r/g, "").replace(/\n+$/, "").split("\n");
}

module.exports = (H) => [
  H.chapter("Ein- und Ausfuhr", 19),

  H.h2("Was wir bauen"),

  H.figure("kap19_einlesen.png", "Eine alte Liste eingelesen: vier Einträge sind neu, Momo war schon da, und drei Zeilen gingen nicht — mit ihrer Nummer und dem Grund.", 480, 320),

  H.p("Bevor es die Mediathek gab, gab es eine Liste in Excel. Die soll hinein, ohne dass jemand sie abtippt. Und umgekehrt soll die Mediathek hinaus können, als Excel-Mappe für alle, die keine Mediathek haben. Das Menü Datei bekommt zwei Einträge: „Als Excel-Mappe sichern …“ und „Liste einlesen …“."),

  H.h2("Der Plan"),

  H.p("Für beide Richtungen gibt es ein eigenes Werkzeug. Hinaus geht es mit dem Modul xlsx: Es schreibt echte Excel-Mappen, mit mehreren Blättern, einer fetten Kopfzeile, Zahlen als Zahlen und Daten als Datum. Lesen kann es nicht — herein kommt eine Liste als CSV, so wie Excel sie mit „Speichern unter … CSV“ sichert. Das liest CSV_LOAD, ein Befehl des Kerns."),

  H.p("Eine fremde Liste ist nie ganz sauber. Darum wird jede Zeile einzeln geprüft: Was passt, kommt in die Tabelle eintrag; was schon da ist, nicht noch einmal; und zu jeder Zeile, die nicht geht, merkt sich das Programm ihre Nummer und den Grund. Am Ende zeigt ein Fenster die Zahlen und die Liste der Meldungen."),

  H.h2("Schritt 1: die alte Liste"),

  H.p("So sieht die Liste aus, die im Ordner code/kap19 liegt:"),

  H.code(liste(), { out: true }),

  H.p("Sie hat alles, was eine echte Liste aus Excel hat. Excel trennt im deutschen Windows mit Semikolon, weil das Komma dort Dezimalzeichen ist. Die Datei ist in Windows-1252 gesichert, nicht in UTF-8 — das „ä“ in Kästner ist ein einziges Byte, 0xE4. Ein Titel, der selbst ein Semikolon enthält, steht in Anführungszeichen. Und es gibt Fehler: eine Zeile ohne Titel, eine Art, die die Mediathek nicht kennt, ein Jahr mit einem x, und Momo, den es schon gibt."),

  H.h2("Schritt 2: lesen, prüfen, eintragen"),

  H.p("Eine Klasse Einlesung sammelt das Ergebnis: die Zahl der Zeilen, wie viele neu und wie viele schon da waren, und die Meldungen. Gelesen wird im neuen Modul Austausch:"),

  H.code(stueck(A, "FUNCTION einlesen", "END FUNCTION")),

  H.p("Die Kodierung \"auto\" entscheidet selbst: Was gültiges UTF-8 ist, wird als UTF-8 gelesen, alles andere als Windows-1252. Den Trenner verrät die erste Zeile. CSV_LOAD liefert dann ein zweidimensionales Feld — t[zeile, spalte] —, und Anführungszeichen hat es schon aufgelöst."),

  H.p("Welche Spalte was ist, sagt die Kopfzeile: spalte(t, \"titel\") sucht den Kopf Titel, gleich wie geschrieben, und liefert seine Nummer. So darf die Liste ihre Spalten in jeder Reihenfolge haben, und eine Spalte, die das Programm nicht kennt, stört nicht."),

  H.p("Danach geht die Schleife durch die Zeilen ab der zweiten. Eine ganz leere überspringt sie, für jede andere gibt es eine Nummer — die Zeile, wie sie im Tabellenprogramm heißt, also ab 1 gezählt und mit der Kopfzeile. INSERT nimmt NIL für ein leeres Jahr: Dann bleibt die Spalte leer, statt 0 zu enthalten."),

  H.h2("Schritt 3: was an einer Zeile falsch sein kann"),

  H.p("Für jede Zeile fragt zeileFalsch$, was an ihr falsch ist, und liefert „“, wenn nichts:"),

  H.code(stueck(A, "FUNCTION zeileFalsch$", "END FUNCTION")),

  H.p("Die Art darf dabei klein geschrieben sein: artGenau$ macht aus „buch“ wieder „Buch“, sonst stünden in der Mediathek zwei Arten, die gleich aussehen. Ob ein Eintrag schon da ist, fragt schonDa nach Titel und Urheber. Alles läuft in einer Transaktion (Kapitel 4): Bricht das Programm mittendrin ab, ist die Tabelle wie vorher."),

  H.h2("Schritt 4: das Fenster"),

  H.p("EinlesenFenster ist modal und hat ein Label, eine Liste und OK. Es liest, sobald es aufgeht, und zeigt, was dabei herauskam; das Hauptfenster lädt danach seine Tabelle neu:"),

  H.code(stueck("kap19/mediathek/Hauptfenster.dh", "SUB miEinlesen_Aktion", "END SUB")),

  H.p("FILE_OPEN_DIALOG ist der Dialog des Systems; „csv,txt“ zeigt nur diese Dateien. Wer abbricht, bekommt „“, und dann geschieht nichts."),

  H.h2("Schritt 5: hinaus als Excel-Mappe"),

  H.code(stueck(A, "SUB alsExcel", "END SUB")),

  H.p("XLSX_NEW legt die Mappe mit dem ersten Blatt an, XLSX_SHEET ein weiteres, und alles danach geht in dieses. Das erste Blatt füllt blattBestand:"),

  H.code(stueck(A, "SUB blattBestand", "END SUB")),

  H.p("Zeile und Spalte zählen ab 0, (0, 0) ist A1; kopf schreibt die Überschriften, macht sie fett und setzt die Spaltenbreiten. Das Jahr kommt mit XLSX_SET_NUM als Zahl hinein, damit Excel damit rechnen und richtig sortieren kann. blattAusleihen nimmt die Abfrage ausleihbuch aus Kapitel 17; Beginn und Ende setzt XLSX_SET_DATE als Datum, aus den Sekunden, die ZEIT_PARSE liefert. Gesichert wird über FILE_SAVE_DIALOG(\"Als Excel-Mappe sichern\", \"mediathek.xlsx\", \"xlsx\") — das zweite Argument ist der vorgeschlagene Name, erst das dritte die Endung."),

  H.h2("Warum so?"),

  H.h3("Warum jede Zeile geprüft wird"),

  H.p("Ein Programm, das beim ersten Fehler abbricht, ist einfach zu schreiben und mühsam zu benutzen: Man bessert Zeile 5 aus, liest neu ein, scheitert an Zeile 6, bessert aus, liest neu ein … Bei einer Liste mit dreihundert Zeilen geht so ein Nachmittag hin. Ein Programm, das fehlerhafte Zeilen still überspringt, ist noch schlimmer: Die Daten fehlen, und niemand weiß es."),

  H.p("Darum prüft die Mediathek jede Zeile, nimmt, was geht, und sagt zu allem anderen, welche Zeile und warum. Wer die Liste ausbessert und noch einmal einliest, bekommt die guten Zeilen nicht doppelt — schonDa sorgt dafür. Einlesen darf man also beliebig oft."),

  H.h3("Warum Excel hinaus, aber CSV herein"),

  H.p("Eine CSV kennt keine Zahlen und kein Datum, nur Text. Wer eine Postleitzahl 01067 als CSV an Excel gibt, sieht 1067. Für eine Auswertung, die jemand liest, ist die Mappe darum das bessere Format. Herein ist es umgekehrt: Jedes Tabellenprogramm kann CSV sichern, und eine CSV ist schnell geprüft — ein Leser für Excel-Mappen müsste viel mehr können, um am Ende dieselben Zeilen zu liefern."),

  H.h2("Was schiefgehen kann"),

  H.h3("Die falsche Kodierung"),

  H.p("Ohne \"auto\" liest CSV_LOAD in UTF-8, und das Byte 0xE4 ist dort kein Zeichen. Statt Unsinn zu lesen, hält das Programm mit einer Meldung an, die Zeile und Stelle nennt und den Ausweg: „… ist sie meist cp1252 -- dann als letztes Argument \"cp1252\" mitgeben.“ Wer sicher weiß, woher seine Dateien kommen, schreibt die Kodierung fest hin; wer es nicht weiß, nimmt \"auto\"."),

  H.h3("Zahlen, die keine sind"),

  H.p("VAL(\"19x8\") ergibt 19, ohne Fehler. Hätte die Mediathek das Jahr nur mit VAL gelesen, stünde Lola rennt jetzt mit dem Jahr 19 in der Tabelle. Darum prüft zeileFalsch$ das Jahr vorher mit einem regulären Ausdruck: drei oder vier Ziffern, sonst nichts."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Lass in einlesen beim CSV_LOAD das \"auto\" weg und lies die alte Liste ein. Was geschieht?"),
  H.bulletRich("Drehen 2. ", "Lies die alte Liste zweimal hintereinander ein. Was sagt das Fenster beim zweiten Mal?"),
  H.bulletRich("Drehen 3. ", "Sichere eine Liste in UTF-8, mit Komma, und mit den Spalten in der Reihenfolge Jahr, Titel, Art, Urheber. Was muss am Programm geändert werden?"),
  H.bulletRich("Erweitern 1. ", "Der Bestand auch als CSV — so geschrieben, dass Excel ihn mit einem Doppelklick richtig öffnet."),
  H.bulletRich("Erweitern 2. ", "Die Zeilen mit Fehler kommen in eine eigene Datei neben der Liste, mit einer Spalte Fehler: dort ausbessern, dann diese Datei einlesen."),
  H.bulletRich("Erfinden. ", "Gibt es zu deiner Sammlung aus Kapitel 13 schon eine Liste? Lies sie ein — und schau, welche Fehler deine eigenen Daten haben."),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Befehle des Kerns → CSV und Textkodierung: ", "Trenner, Anführungszeichen, alle Kodierungen, FILE_ENCODING$."),
  H.bulletRich("Handbuch, Modul xlsx: ", "Blätter, Formate, Datum, Spaltenbreiten — und was der Schreiber nicht kann."),
];
