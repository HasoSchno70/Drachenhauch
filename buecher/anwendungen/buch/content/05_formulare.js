const { zeilen, ohneKopf, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Formulare, die prüfen", 5),

  H.h2("Was wir bauen"),

  H.figure("kap05_pruefen.png", "Ein Klick auf „Speichern“ mit leerem Titel: Das Feld bekommt einen roten Rahmen und den Fokus, darunter steht, was fehlt.", 460, 315),

  H.p("Bis jetzt konnte man Einträge aufnehmen und entfernen, aber nicht ändern. Jetzt lädt ein Doppelklick einen Eintrag ins Formular, „Speichern“ schreibt ihn zurück, und „Neu“ leert das Formular für einen neuen. Bevor gespeichert wird, prüft das Formular, ob alles passt: Ein Titel muss sein, ein Jahr muss eines sein, und die Meldung steht dort, wo man gerade hinsieht."),

  H.h2("Der Plan"),

  H.p("Bisher stand für jedes Feld eine Zeile Code beim Lesen (GUI_TEXT), eine beim Schreiben (GUI_SET_TEXT) und ein Fragezeichen im SQL. Bei vier Feldern geht das; bei zwanzig wird es eine lange Liste, in der man leicht eines vergisst. Darum bekommt jedes Feld einen Namen — den seiner Spalte in der Datenbank —, und dann erledigen drei Befehle das ganze Formular: laden, speichern, leeren."),

  H.p("Auch das Prüfen bekommt einen festen Ort. Statt im Speichern-Knopf nachzusehen, ob der Titel leer ist, hängt an jedem Feld, was für es gilt. Ein Befehl prüft alle Felder auf einmal und sagt, wie viele nicht passen."),

  H.h2("Schritt 1: binden, laden, speichern"),

  H.code(stueck("kap05/1_bearbeiten.dh", "GUI_BIND(titel", "GUI_BIND(jahr")),

  H.pmix([["GUI_BIND", true], " gibt einem Feld einen Schlüssel — hier den Namen seiner Spalte in der Tabelle eintrag — und ordnet es einem Formular zu, das hier \"e\" heißt. Ein Fenster kann mehrere Formulare haben, etwa auf zwei Reitern; der Name hält sie auseinander. Zwei Felder brauchen dafür eine kleine Vorbereitung. Das Jahr wird ein Zahlenfeld, damit es als Zahl gespeichert wird:"]),

  H.code(stueck("kap05/1_bearbeiten.dh", "' Ein Zahlenfeld", "GUI_TEXTINPUT_SET(jahr")),

  H.p("Und die Auswahl der Art gibt jedem Eintrag seinen Text als Wert mit. Ohne das bände sie ihre Nummer — 1 statt Film —, und in der Datenbank stünde etwas anderes als in Kapitel 4:"),

  H.code(stueck("kap05/1_bearbeiten.dh", "DIM a AS INTEGER", "NEXT")),

  H.pmix(["Laden ist dann ein Befehl. ", ["GUI_FORM_LOAD", true], " liest aus der Tabelle eintrag die Zeile mit dieser Kennung und schreibt jede Spalte in das Feld mit ihrem Schlüssel:"]),

  H.code(stueck("kap05/1_bearbeiten.dh", "SUB bearbeiten", "END SUB")),

  H.pmix(["Die Variable ", ["bearbeitet", true], " merkt sich, welcher Eintrag im Formular steht; -1 heißt: ein neuer. ", ["GUI_FORM_SAVE", true], " nimmt sie und entscheidet selbst — bei -1 ein INSERT, das die neue Kennung zurückgibt, sonst ein UPDATE dieser Zeile:"]),

  H.code(stueck("kap05/1_bearbeiten.dh", "SUB formular_speichern", "END SUB")),

  H.p("Danach lädt die Mediathek alles neu aus der Datenbank und füllt die Tabelle neu. Das ist mehr Arbeit, als nur die eine geänderte Zeile nachzuführen, aber bei ein paar hundert Einträgen merkt man es nicht — und die Tabelle zeigt sicher, was gespeichert ist. Leeren ist der dritte Befehl:"),

  H.code(stueck("kap05/1_bearbeiten.dh", "SUB formular_leeren", "END SUB")),

  H.pmix([["GUI_FORM_CLEAR", true], " leert alle Felder des Formulars; die Art bekommt danach wieder ihre Vorgabe. „Speichern“ ist jetzt der Standard-Knopf, ein neuer Knopf „Neu“ ruft formular_leeren, und der Doppelklick ruft bearbeiten statt zeigen."]),

  H.figure("kap05_1_bearbeiten.png", "Der erste Stand: Formular, Tabelle und die Knöpfe Entfernen, Neu und Speichern.", 460, 315),

  H.h2("Schritt 2: Regeln am Feld"),

  H.p("Die Prüfung im Speichern-Knopf — ist der Titel leer? — wird durch Regeln ersetzt, die am Feld hängen:"),

  H.code(stueck("kap05/2_pruefen.dh", "' Was fuer jedes Feld gilt", "GUI_VALIDATE_LIVE")),

  H.pmix([["GUI_RULE", true], " hängt eine Regel an ein Feld: ", ["pflicht", true], " (darf nicht leer sein, eine Auswahl muss gewählt sein), ", ["laenge", true], " (so viele Zeichen), ", ["bereich", true], " (eine Zahl zwischen zwei Grenzen). Der letzte Text ist die Meldung; ohne ihn sagt die gui selbst etwas Passendes. Es gibt noch ", ["zahl", true], ", ", ["ganz", true], ", ", ["email", true], ", ", ["datum", true], " und ", ["muster", true], " für alles, was sich als regulärer Ausdruck schreiben lässt."]),

  H.pmix(["Wohin die Meldung kommt, sagt ", ["GUI_ERROR_LABEL", true], ": in eine Beschriftung, die sich alle Felder teilen und die zwischen Formular und Tabelle steht. ", ["GUI_VALIDATE_LIVE", true], " prüft ein Feld, sobald man es verlässt — nicht bei jedem Tastendruck, denn ein halb getipptes Jahr ist noch kein falsches. Beim Speichern prüft ein Befehl alles:"]),

  H.code(stueck("kap05/2_pruefen.dh", "' Alle Regeln auf einmal", "IF GUI_VALIDATE")),

  H.pmix([["GUI_VALIDATE", true], " liefert die Zahl der Felder, die nicht passen, setzt um jedes einen roten Rahmen und den Fokus in das erste — man muss nicht suchen, wo es hakt. Ist die Zahl 0, wird gespeichert."]),

  H.h2("Schritt 3: geändert oder nicht"),

  H.p("Wer einen Eintrag lädt, ein Wort ändert und dann doppelt auf einen anderen Eintrag klickt, verliert seine Änderung — der neue überschreibt sie im Formular. Das Formular weiß aber, ob es geändert ist:"),

  H.code(stueck("kap05/3_geaendert.dh", "' Ungespeicherte Eingaben", "END IF")),

  H.pmix([["GUI_FORM_CHANGED", true], " vergleicht jedes Feld mit dem Stand nach dem letzten Laden, Speichern oder Leeren. Ist etwas anders, lädt bearbeiten nichts, und die Statuszeile sagt, was zu tun ist. Dieselbe Frage entscheidet in jedem Bild, ob „Speichern“ überhaupt bedienbar ist:"]),

  H.code(stueck("kap05/3_geaendert.dh", "' Speichern nur, wenn", "GUI_SET_ENABLED")),

  H.p("Eine Stelle muss dafür noch mithelfen. formular_leeren setzt nach dem Leeren die Art auf „Buch“ — und das ist, verglichen mit dem leeren Stand, eine Änderung. Ohne die letzte Zeile hier wäre ein frisch geleertes Formular schon „geändert“:"),

  H.code(stueck("kap05/3_geaendert.dh", "SUB formular_leeren", "END SUB")),

  H.pmix([["GUI_FORM_CLEAN", true], " erklärt den jetzigen Stand zum sauberen. Das ganze Programm:"]),

  H.code(zeilen("kap05/3_geaendert.dh")),

  H.h2("Warum so?"),

  H.h3("Die Prüfung gehört ans Feld"),

  H.p("Prüft der Speichern-Knopf, steht das Wissen darüber, was ein gutes Jahr ist, an der falschen Stelle: Kommt ein zweiter Weg zum Speichern dazu — Strg+S in Kapitel 6, ein Import in Kapitel 19 —, muss man die Prüfung dorthin kopieren und beim nächsten Mal an beiden Stellen ändern. Am Feld gilt die Regel für jeden Weg. Und weil die gui weiß, welches Feld welche Regel verletzt, kann sie das Feld zeigen: roter Rahmen, Fokus, Meldung — ohne eine Zeile Code dafür."),

  H.h3("Leer ist keine falsche Eingabe"),

  H.p("Die Regel bereich 1450 bis 2100 sagt nichts darüber, ob ein Jahr eingegeben sein muss. Ein leeres Jahr kommt durch — und wird in der Datenbank NULL, kein Wert. Ob ein Feld leer sein darf, sagt allein pflicht. So sieht man jeder Regel an, was sie verlangt, und muss nicht raten, ob bereich zugleich „muss ausgefüllt sein“ heißt."),

  H.h3("Was keine Regel kann"),

  H.pmix(["Manches kann eine Regel nicht wissen: ob eine ISBN die richtige Prüfziffer hat, ob ein Buch einen Urheber braucht, ein Spiel aber nicht. Das prüft das Programm nach GUI_VALIDATE selbst und meldet es mit ", ["GUI_SET_ERROR(feld, meldung)", true], " — auf demselben Weg, mit rotem Rahmen und in derselben Zeile. Für den, der das Formular ausfüllt, gibt es keinen Unterschied. Die beiden Übungen zum Erweitern machen genau das."]),

  H.h2("Was schiefgehen kann"),

  H.h3("Ein Schlüssel, den es als Spalte nicht gibt"),

  H.code(stueck("kap05/fehler_spalte.dh", "GUI_BIND(titel", "PRINT GUI_FORM_SAVE")),
  H.code(["Laufzeitfehler in fehler_spalte.dh:14: GUI_FORM_SAVE: table eintrag",
          "has no column named name"], { out: true }),

  H.p("GUI_FORM_SAVE baut sein SQL aus den Schlüsseln. Heißt ein Schlüssel anders als die Spalte, kennt die Datenbank die Spalte nicht, und sie sagt es auf Englisch: Die Tabelle eintrag hat keine Spalte name. Der Schlüssel ist darum immer genau der Spaltenname."),

  H.h3("Eine Vorgabe, die als Änderung zählt"),

  H.p("Fehlt in formular_leeren das GUI_FORM_CLEAN, gibt es keine Meldung — aber das Formular ist nach „Neu“ sofort „geändert“, „Speichern“ ist bedienbar, obwohl nichts eingegeben ist, und ein Doppelklick auf einen Eintrag lädt ihn nicht, sondern sagt „Erst speichern oder mit Neu verwerfen.“. Alles, was das Programm nach dem Laden oder Leeren selbst in ein Feld schreibt, zählt für GUI_FORM_CHANGED als Änderung; wer eine Vorgabe setzt, erklärt den Stand danach mit GUI_FORM_CLEAN zum sauberen."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Nimm die Regel pflicht für den Titel heraus und speichere ein Formular, in dem nur ein Urheber steht. Hält die Datenbank das auf? In der Tabelle steht doch titel TEXT NOT NULL."),
  H.bulletRich("Drehen 2. ", "Nimm GUI_VALIDATE_LIVE heraus. Wann siehst du jetzt, dass ein Jahr nicht passt?"),
  H.bulletRich("Drehen 3. ", "Lade Metropolis, ändere nichts und klicke doppelt auf Tetris. Was passiert — und was, wenn du vorher im Feld Urheber ein Zeichen tippst und wieder löschst?"),
  H.bulletRich("Erweitern 1. ", "Ein Feld für die ISBN. Eine Regel lässt nur Ziffern und Striche zu, und vor dem Speichern prüft das Programm die Prüfziffer einer ISBN-13: die Ziffern abwechselnd mal 1 und mal 3 zusammengezählt, die Summe muss durch 10 teilbar sein. Die Spalte isbn kommt in eine bestehende Datenbank dazu."),
  H.bulletRich("Erweitern 2. ", "Ein Buch braucht einen Urheber, ein Film, ein Spiel und Musik nicht. Fehlt er bei einem Buch, steht die Meldung am Feld Urheber wie jede andere."),
  H.bulletRich("Erfinden. ", "Ein Formular mit Regeln für etwas, das du kennst: eine Anmeldung mit E-Mail und Geburtsdatum, eine Bestellung mit Anzahl und Postleitzahl. Probiere jede Regel einmal mit einer falschen Eingabe aus."),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Modul gui → Datenbindung: ", "was sich binden lässt, wie Werte gespeichert werden (Zahlen, Häkchen, Geld in Cent) und wie zwei Leute dieselbe Zeile bearbeiten, ohne sich zu überschreiben."),
  H.bulletRich("Handbuch, Modul gui → Formularprüfung: ", "alle Arten von Regeln und wie Meldungen angezeigt werden."),
  H.bulletRich("Beispiel examples/verein/: ", "eine Vereinsverwaltung mit gebundenen Formularen an einem Datenbank-Server."),
];
