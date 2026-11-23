const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Code und Ereignisse", 10),

  H.h2("Was wir bauen"),

  H.figure("kap10_modul.png", "Das Projekt dieses Kapitels: Im Navigator stehen die App, die Klasse Eintrag, das Modul Beispiele und das Hauptfenster. Rechts der Code der Funktion beispiele, darüber ihre Kopfzeile.", 480, 268),

  H.p("Die Mediathek bekommt ihren Code. Die Einträge sind wieder Objekte einer Klasse, wie in Kapitel 3, die Beispiele kommen aus einem Modul, und das Fenster reagiert auf seine Ereignisse: Speichern nimmt einen Eintrag auf oder ändert ihn, ein Doppelklick in die Tabelle lädt einen Eintrag ins Formular, Entfernen nimmt ihn heraus. Das Fenster sieht aus wie in Kapitel 9."),

  H.h2("Der Plan"),

  H.p("Ein Studio-Projekt hat vier Arten von Teilen, und jede hat ihren Zweck:"),

  H.table([
    ["App", "das Programm als Ganzes: was vor dem ersten Fenster und nach dem letzten geschieht"],
    ["Fenster", "eine Klasse mit Aussehen (.dhform) und Code: Felder, Ereignisse, eigene Methoden"],
    ["Klasse", "ein Ding mit Daten und Verhalten, ohne Fenster — hier der Eintrag"],
    ["Modul", "Funktionen und SUBs, die keinem Ding gehören — hier die Beispiele"],
  ], { headers: ["Teil", "wofür"], widths: [1800, 7226] }),

  H.p("Projekt → Klasse hinzufügen legt eine Klasse an, Projekt → Modul hinzufügen ein Modul. Beide stehen danach im Navigator; ein Klick auf eine Methode öffnet ihren Code in einem Reiter."),

  H.h2("Schritt 1: die Klasse und das Modul"),

  H.p("Die Klasse Eintrag ist die aus Kapitel 3, Zeile für Zeile:"),

  H.code(stueck("kap10/mediathek/Eintrag.dh", "CLASS Eintrag", "END CLASS")),

  H.p("Im Studio stehen Init und zellen als zwei Methoden im Navigator, die vier DIM unter „Eigenschaften“. Das Modul Beispiele hat eine einzige Funktion, die die Beispiele baut:"),

  H.code(stueck("kap10/mediathek/Beispiele.dh", "FUNCTION beispiele", "END FUNCTION")),

  H.p("Ein Modul hat kein Self und keine NEW. Seine Funktionen ruft man im ganzen Projekt einfach beim Namen — beispiele(), wie einen eingebauten Befehl."),

  H.h2("Schritt 2: Felder und eigene Methoden des Fensters"),

  H.p("Das Fenster ist eine Klasse, und was es sich merken muss, wird ein Feld. Unter „Eigenschaften“ des Hauptfensters stehen zwei DIM:"),

  H.code(stueck("kap10/mediathek/Hauptfenster.dh", "DIM eintraege AS ARRAY OF Eintrag", "DIM bearbeitet AS INTEGER")),

  H.pmix(["Was mehrere Ereignisse brauchen, wird eine eigene Methode. Strg+Umschalt+M fragt nach Name, Parametern und Rückgabe; ohne Rückgabe wird es eine SUB. Die Mediathek hat drei: ", ["tabelleFuellen", true], " baut die Tabelle aus dem Feld, ", ["formularLeeren", true], " macht das Formular frei für einen neuen Eintrag, ", ["bearbeiten", true], " lädt einen in das Formular:"]),

  H.code(stueck("kap10/mediathek/Hauptfenster.dh", "SUB bearbeiten", "END SUB")),

  H.p("Ein Feld des Fensters und seine Controls erreicht eine Methode über Self — Self.eintraege, Self.feldTitel. Eine eigene Methode ruft man ebenso: Self.bearbeiten(z)."),

  H.h2("Schritt 3: Ereignisse"),

  H.p("Ein Doppelklick auf ein Control im Entwurf fragt, welches Ereignis es bekommen soll; ein Doppelklick auf eine leere Stelle fragt das Fenster. Jede Art hat ihre eigenen: ein Knopf Klick, ein Textfeld Geaendert, Enter, FokusRein und FokusRaus, eine Tabelle Geaendert und Doppelklick, das Fenster Geoeffnet, Schliessen und Groesse. Die Methode heißt dann nach Control und Ereignis — knopfSpeichern_Klick, tabelle_Doppelklick — und das Studio schreibt ihre Kopfzeile selbst."),

  H.code(stueck("kap10/mediathek/Hauptfenster.dh", "SUB Geoeffnet", "END SUB")),

  H.p("Geoeffnet läuft, sobald das Fenster aufgebaut ist: Die Beispiele kommen in das Feld, die Tabelle wird gefüllt, das Formular geleert. Speichern unterscheidet, ob ein neuer Eintrag entsteht oder ein bestehender geändert wird:"),

  H.code(stueck("kap10/mediathek/Hauptfenster.dh", "SUB knopfSpeichern_Klick", "END SUB")),

  H.p("Der Doppelklick ist eine Zeile — die Arbeit tut die eigene Methode:"),

  H.code(stueck("kap10/mediathek/Hauptfenster.dh", "SUB tabelle_Doppelklick", "END SUB")),

  H.h2("Warum so?"),

  H.h3("Was in ein Fenster gehört und was nicht"),

  H.p("Eine Faustregel: Was ohne Fenster Sinn hat, gehört nicht ins Fenster. Ein Eintrag ist ein Eintrag, ob ihn eine Tabelle zeigt, ein Bericht druckt oder ein Export nach Excel schreibt — darum ist er eine Klasse für sich. Die Beispiele braucht man beim ersten Start und vielleicht in einem Test, nie aber einen Knopf dafür — darum stehen sie in einem Modul. Im Fenster bleibt, was nur dort Sinn hat: welcher Eintrag gerade im Formular steht, wie die Tabelle gefüllt wird, was ein Klick tut."),

  H.p("Der Lohn kommt in Teil III: Wenn die Einträge in eine Datenbank ziehen, ändert sich das Modul, das sie lädt, und kaum etwas am Fenster. Und wenn ein zweites Fenster dazukommt (Kapitel 11), kennt es die Klasse Eintrag schon."),

  H.h3("Ereignisse statt einer Schleife"),

  H.p("In Teil I stand am Ende eine Schleife mit einer Abfrage für jeden Knopf, jeden Menüeintrag und jeden Dialog. Hier gibt es keine: Jedes Ereignis hat seine Methode, und das Studio schreibt die Abfragen in die erzeugte Basisklasse (Kapitel 8). Wer wissen will, was beim Klick auf „Speichern“ geschieht, öffnet knopfSpeichern_Klick und findet es dort — und nur dort."),

  H.h2("Was schiefgehen kann"),

  H.h3("Eine Methode, die zeigen heißt"),

  H.p("Wer in Teil I eine SUB zeigen geschrieben hat und sie ins Fenster mitnimmt, bekommt eine Warnung unter Probleme:"),

  H.code(["Hauptfenster.zeigen: Methode 'zeigen' in HAUPTFENSTER ueberschreibt",
          "'Zeigen' aus STUDIOFENSTER -- Gross-/Kleinschreibung zaehlt nicht,",
          "es ist derselbe Name. Jeder Aufruf von 'Zeigen' landet jetzt hier."], { out: true, studio: "Probleme" }),

  H.p("Jedes Fenster erbt von der Laufzeit des Studios eine Methode Zeigen, die es aufbaut und öffnet. Namen kennen in Drachenhauch keine Groß- und Kleinschreibung; zeigen ist darum derselbe Name, und die eigene Methode ersetzt die geerbte. Startet man trotzdem, ruft die Laufzeit zum Öffnen des Fensters Zeigen auf und landet in der eigenen Methode. Schreibt die — wie die aus Kapitel 3 — in die Statuszeile, gibt es die noch gar nicht: Das Programm hält mit einem Laufzeitfehler in Hauptfenster.zeigen, Zeile 1, und das Fenster geht nie auf. Die Abhilfe ist ein anderer Name: eintragZeigen."),

  H.h3("Ein Feld, das nicht initialisiert ist"),

  H.p("bearbeitet ist eine INTEGER und fängt bei 0 an, nicht bei -1. Ohne den Aufruf von formularLeeren in Geoeffnet stünde sie beim ersten Speichern auf 0 — und der erste neue Eintrag überschriebe Momo, den Eintrag in Zeile 0. Es gibt keine Meldung; man sieht nur, dass Momo fehlt. Ein Feld, das einen besonderen Anfangswert braucht, bekommt ihn darum in Geoeffnet."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Schreibe in tabelle_Doppelklick bearbeiten(...) statt Self.bearbeiten(...). Geht das?"),
  H.bulletRich("Drehen 2. ", "Lege eintraege nicht unter „Eigenschaften“ an, sondern mit DIM in Geoeffnet. Was sagt das Studio, und was geschieht beim Start?"),
  H.bulletRich("Drehen 3. ", "Nimm Self.formularLeeren() aus Geoeffnet heraus, tippe einen Titel ein und speichere. Was ist mit Momo?"),
  H.bulletRich("Erweitern 1. ", "Ein Modul Statistik mit einer Funktion zusammenfassung$, die die Einträge je Art zählt; die Statuszeile zeigt „5 Einträge -- Buch 2, Film 1, Spiel 1, Musik 1“."),
  H.bulletRich("Erweitern 2. ", "Die Klasse Eintrag bekommt eine Methode alsText$ — „Momo von Michael Ende (Buch, 1973)“ —, und ein Klick in die Tabelle zeigt den Eintrag so in der Statuszeile."),
  H.bulletRich("Erfinden. ", "Nimm dein Fenster aus Kapitel 9 und gib ihm Code: eine Klasse für das, was es zeigt, ein Modul für das, was ohne Fenster Sinn hat, und für jeden Knopf sein Ereignis."),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Drachenhauch Studio → Code und Ereignisse: ", "alle Ereignisse je Art, Methoden hinzufügen und entfernen, Vorschläge beim Tippen."),
  H.bulletRich("Handbuch, Drachenhauch Studio → Das Projekt: ", "Module, Klassen und was in welcher Datei steht."),
];
