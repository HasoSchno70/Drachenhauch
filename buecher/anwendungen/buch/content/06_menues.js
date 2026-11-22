const { zeilen, ohneKopf, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Menüs, Kürzel, Dialoge", 6),

  H.h2("Was wir bauen"),

  H.figure("kap06_frage.png", "Metropolis ist gewählt, „Entfernen“ fragt nach. Dahinter ist alles abgedunkelt und nimmt keine Klicks an, bis eine Antwort da ist.", 460, 315),

  H.p("Die Mediathek bekommt eine Menüleiste mit den Befehlen, die sie kann, jeder mit seinem Tastenkürzel: Strg+N für einen neuen Eintrag, Strg+S zum Speichern, Entf zum Entfernen, Strg+Q zum Beenden. Bevor etwas entfernt wird, fragt ein Dialog nach. Und wer das Fenster schließt, während im Formular etwas Ungespeichertes steht, wird gefragt, was damit geschehen soll."),

  H.h2("Der Plan"),

  H.p("Ein Menü ist eine zweite Art, dieselben Befehle zu erreichen. Die Befehle selbst gibt es schon — formular_speichern, formular_leeren, auswahl_entfernen —, und das Menü ruft dieselben SUBs auf wie die Knöpfe. Neu sind die Kürzel, die man nicht in der Schleife abfragt, sondern an den Menüeintrag schreibt."),

  H.p("Und es kommt eine Frage dazu, die warten muss. Kapitel 1 hat gezeigt, dass man in einer Anwendung nicht mit einer eigenen Schleife wartet. Ein Dialog wartet darum auch nicht: Er öffnet sich, die Schleife läuft weiter, und die Antwort kommt in einem späteren Bild, wie ein Klick."),

  H.h2("Schritt 1: eine Menüleiste"),

  H.code(stueck("kap06/1_menue.dh", "DIM mDatei AS INTEGER", "miEntfernen = GUI_MENU_ITEM")),

  H.pmix([["GUI_MENU", true], " legt ein Menü in der Menüleiste des Fensters an, ", ["GUI_MENU_ITEM", true], " einen Eintrag darin, mit seinem Kürzel als drittem Argument. Kürzel schreibt man, wie man sie liest: \"Strg+S\", \"Strg+Umschalt+O\", \"F5\", \"Entf\". ", ["GUI_MENU_SEPARATOR", true], " zieht eine Trennlinie. Ein Menüeintrag meldet sich wie ein Knopf mit GUI_CLICKED — egal, ob man ihn angeklickt oder sein Kürzel gedrückt hat:"]),

  H.code(stueck("kap06/1_menue.dh", "' Knopf und Menue fuehren", "IF GUI_CLICKED(miBeenden)")),

  H.figure("kap06_menue.png", "Das Menü Datei. „Speichern“ ist ausgegraut — es gibt nichts zu speichern —, und sein Kürzel tut dann auch nichts.", 460, 315),

  H.p("Die Menüleiste braucht Platz und schiebt den Inhalt des Fensters nach unten. Die Spalte des Formulars darf darum nicht mehr so hoch sein wie das ganze Fenster, sondern nur so hoch wie der Bereich darunter:"),

  H.code(stueck("kap06/1_menue.dh", "' Die Menueleiste nimmt oben", "form = GUI_LAYOUT")),

  H.pmix(["Und „Speichern“ im Menü wird gesperrt, wann immer der Knopf gesperrt ist — mit ", ["GUI_MENU_ENABLE", true], " neben GUI_SET_ENABLED. Ein gesperrter Eintrag ist grau, nimmt keinen Klick an, und sein Kürzel schweigt."]),

  H.h2("Schritt 2: ein Dialog, der fragt"),

  H.code(stueck("kap06/2_nachfrage.dh", "SUB entfernen_fragen", "END SUB")),

  H.pmix([["GUI_DIALOG", true], " öffnet einen Dialog mit Titel, Text und Knöpfen; mit | getrennt stehen die Beschriftungen der Knöpfe. Er liefert sofort einen Griff vom Typ GUI_WINDOW zurück, und die Variable frage merkt ihn sich. Solange er offen ist, ist alles dahinter abgedunkelt und nimmt keine Eingabe an. Enter drückt den ersten Knopf, Esc und das Kreuz den letzten. Die Antwort holt die Schleife:"]),

  H.code(stueck("kap06/2_nachfrage.dh", "' Die Antwort gilt ein Bild lang", "END IF")),

  H.pmix([["GUI_ANSWER", true], " ist 0, solange der Dialog offen ist, und danach in genau einem Bild die Nummer des gedrückten Knopfs: 1 für „Entfernen“, 2 für „Abbrechen“. Erst dann wird entfernt. frage wird wieder -1 — ", ["nicht 0", true], ", denn 0 ist der Griff des ersten Fensters des Programms."]),

  H.h2("Schritt 3: nicht einfach schließen"),

  H.p("Bis jetzt endete das Programm, sobald jemand das Kreuz des Fensters drückte. Jetzt fragt es, wenn im Formular etwas Ungespeichertes steht:"),

  H.code(stueck("kap06/3_schliessen.dh", "SUB beenden_fragen", "END SUB")),

  H.p("Die Schleife läuft dafür nicht mehr, solange QUITREQUESTED() falsch ist, sondern solange beenden falsch ist — WHILE NOT beenden; das Programm entscheidet selbst, wann Schluss ist. Ganz oben in der Schleife steht:"),

  H.code(stueck("kap06/3_schliessen.dh", "IF WINDOW_CLOSE_REQUESTED()", "END IF")),

  H.pmix([["WINDOW_CLOSE_REQUESTED", true], " ist in dem Bild wahr, in dem jemand das Fenster schließen will — mit dem Kreuz, mit Alt+F4. Das Fenster bleibt dabei offen. QUITREQUESTED ohne das Kreuz kommt nur aus einem Testlauf mit fester Bildzahl, und dann endet das Programm gleich. Die Antwort auf die Frage wertet die Schleife aus wie in Schritt 2:"]),

  H.code(stueck("kap06/3_schliessen.dh", "IF ende >= 0 THEN", "END IF", 2)),

  H.p("„Speichern“ beendet nur, wenn danach wirklich nichts Ungespeichertes mehr da ist. Fehlt der Titel, hält die Regel aus Kapitel 5 das Speichern auf, das Fenster bleibt offen, und die Meldung sagt, was fehlt. Das ganze Programm:"),

  H.code(zeilen("kap06/3_schliessen.dh")),

  H.h2("Warum so?"),

  H.h3("Ein Kürzel steht nur im Menü"),

  H.p("Man könnte Strg+S auch in der Schleife abfragen, mit KEY_DOWN und KEYHIT. Steht das Kürzel dann zusätzlich im Menü, läuft der Befehl zweimal: einmal, weil die gui das Kürzel des Eintrags erkannt hat, und einmal durch die eigene Abfrage. Fehlt es im Menü, sieht niemand, dass es das Kürzel gibt. Am Menüeintrag ist es beides — sichtbar und einmal. Dazu kommt, was die gui sonst noch weiß: Ein gesperrter Eintrag hat kein Kürzel, ein Kürzel ohne Strg wie Entf gehört dem Textfeld, wenn die Schreibmarke darin steht, und ein offener Dialog lässt die Kürzel dahinter schweigen."),

  H.h3("Warum der Dialog nicht wartet"),

  H.pmix(["Es gibt auch Dialoge, die warten: ", ["GUI_CONFIRM", true], " öffnet einen Kasten des Betriebssystems und kehrt erst mit der Antwort zurück. Das ist bequem — aber solange er offen ist, steht die Schleife, und das Fenster dahinter zeichnet nicht. In einem Werkzeug, das man nebenher benutzt, ist das in Ordnung. GUI_DIALOG passt dagegen zu dem, wie dieses Buch Anwendungen baut: Die Schleife läuft weiter, die Antwort ist ein Ereignis wie ein Klick, und der Dialog sieht aus wie der Rest der Anwendung. Er geht auch im Browser, wo es keinen Kasten des Betriebssystems gibt."]),

  H.h3("Knöpfe, die sagen, was sie tun"),

  H.p("Ein Dialog mit „Ja“ und „Nein“ zwingt dazu, die Frage noch einmal zu lesen, bevor man antwortet. „Entfernen“ und „Abbrechen“ sagen, was passiert. Wer schnell klickt, klickt so seltener das Falsche — und Enter auf dem ersten Knopf tut das, was die Frage vorschlägt."),

  H.h2("Was schiefgehen kann"),

  H.h3("Eine Menüleiste über dem Formular"),

  H.p("Bekommt die Spalte die Höhe des ganzen Fensters statt der des Inhaltsbereichs, ist sie um die Menüleiste zu hoch. Die Knopfreihe unten rutscht dann aus dem Fenster — man sieht nur noch ihren oberen Rand. GUI_WINDOW_CONTENT_H liefert die richtige Höhe, und nach einer Größenänderung sorgt der Anker der Spalte dafür, dass es so bleibt."),

  H.h3("Der Dialog als 0"),

  H.code(stueck("kap06/fehler_null.dh", "DIM frage AS GUI_WINDOW", "IF frage >= 0")),
  H.code(["Laufzeitfehler in fehler_null.dh:8: GUI_ANSWER: das Fenster ist kein",
          "Dialog (GUI_DIALOG/GUI_PROMPT)"], { out: true }),

  H.p("Wer sich „kein Dialog offen“ mit 0 merkt statt mit -1, fragt GUI_ANSWER nach dem Fenster 0 — und das ist das erste Fenster des Programms, das Hauptfenster. Fenstergriffe zählen ab 0. GUI_ANSWER merkt, dass dieses Fenster kein Dialog ist, und sagt es. Mit -1 liefert GUI_ANSWER einfach 0, „keine Antwort“, und die Abfrage in der Schleife darf auch dann laufen, wenn gar nichts offen ist."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Frage Strg+S zusätzlich in der Schleife ab: IF KEY_DOWN(KEY_LCTRL) AND KEYHIT(KEY_S) THEN formular_speichern(). Tippe einen Titel ein und drücke Strg+S. Was passiert, und warum steht danach eine Meldung da?"),
  H.bulletRich("Drehen 2. ", "Ersetze \"Entfernen|Abbrechen\" durch \"janein\". Was steht jetzt auf den Knöpfen, und welche Nummer hat welcher?"),
  H.bulletRich("Drehen 3. ", "Setze die Schreibmarke in das Titelfeld, in dem schon etwas steht, und drücke Entf. Was wird entfernt?"),
  H.bulletRich("Erweitern 1. ", "Ein Eintrag „Entfernen rückgängig“ im Menü Bearbeiten holt die zuletzt entfernten Einträge zurück, mit ihren alten Kennungen. Er ist nur bedienbar, wenn es etwas zurückzuholen gibt."),
  H.bulletRich("Erweitern 2. ", "Ein Menü „Ansicht“ mit einem Häkchen-Eintrag „Filterzeile“, der die Filterzeile der Tabelle ein- und ausblendet."),
  H.bulletRich("Erfinden. ", "Gib einem deiner Programme aus den Kapiteln davor eine Menüleiste: jeden Befehl, den es hat, mit einem Kürzel, das man errät — und einen Dialog vor allem, was sich nicht rückgängig machen lässt."),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Modul gui → Menüs: ", "Untermenüs, Kontextmenüs, Häkchen und Sinnbilder, und welche Taste wann wem gehört."),
  H.bulletRich("Handbuch, Modul gui → Modale Dialoge: ", "GUI_DIALOG, GUI_PROMPT mit Eingabefeld, eigene modale Fenster, und wann GUI_CONFIRM die bessere Wahl ist."),
  H.bulletRich("Beispiel examples/193_gui_dialoge.dh: ", "alle Arten von Dialogen nebeneinander."),
];
