const { zeilen, ohneKopf, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.part("Teil I · Von Hand: das gui-Modul in der IDE"),

  H.p("Im ersten Teil entsteht eine schlichte Mediathek in einer einzigen Datei, geschrieben in der IDE. Jedes Fenster, jedes Feld und jeder Knopf wird von Hand angelegt, jede Bedienung von Hand abgefragt. Das ist mehr Arbeit, als später im Studio nötig ist — und genau darum lohnt es sich: Wer einmal gesehen hat, was unter der Oberfläche passiert, versteht, was das Studio ihm abnimmt, und kann eingreifen, wenn es einmal nicht reicht."),

  H.chapter("Ein Fenster und ein Knopf", 1),

  H.h2("Was wir bauen"),

  H.figure("kap01_leer.png", "Die erste Mediathek: ein Feld für den Titel, ein Knopf, darunter die Antwort. Hier hat jemand auf „Aufnehmen“ geklickt, ohne etwas einzutippen.", 460, 270),

  H.p("Ein Fenster mit einem Eingabefeld für einen Titel und einem Knopf „Aufnehmen“. Wer einen Titel eintippt und den Knopf drückt (oder Enter), nimmt ihn in die Mediathek auf; darunter steht, wie viele Titel es sind und welche. Ein leeres Feld wird freundlich abgewiesen."),

  H.h2("Der Plan"),

  H.p("Ein Spiel und eine Anwendung unterscheiden sich weniger, als man denkt. Beide haben eine Schleife, die Bild für Bild läuft. Der Unterschied liegt darin, wer bestimmt, was passiert: Im Spiel rechnet das Programm in jedem Bild die Welt weiter, ob jemand eine Taste drückt oder nicht. In einer Anwendung passiert fast nichts, bis jemand klickt oder tippt — und dann genau eine Sache."),

  H.p("Für die Mediathek heißt das:"),

  H.bulletRich("Einmal, am Anfang: ", "das Fenster bauen — eine Beschriftung, ein Eingabefeld, einen Knopf, eine Zeile für die Antwort."),
  H.bulletRich("In jedem Bild: ", "die gui die Maus und die Tasten verarbeiten lassen, dann fragen, ob der Knopf gedrückt wurde, und alles zeichnen."),
  H.bulletRich("Wenn der Knopf gedrückt wurde: ", "den Titel aus dem Feld lesen, prüfen, aufnehmen, die Antwort hinschreiben, das Feld leeren."),

  H.h2("Schritt 1: ein Fenster"),

  H.code(ohneKopf("kap01/1_fenster.dh")),

  H.pmix([["IMPORT \"gui\"", true], " holt das Modul für Oberflächen herein. ", ["SCREEN", true], " öffnet das Fenster des Programms, wie in jedem Spiel; die letzte Zahl ist der Maßstab, und 1 heißt: ein Punkt ist ein Bildpunkt (Spiele im Pixel-Look nehmen dort 2 oder 3). Darin legt ", ["GUI_WINDOW", true], " ein Fenster der gui an — mit Titel, Lage und Größe — und ", ["GUI_LABEL", true], " eine Beschriftung darin. Beide Befehle liefern einen Griff (", ["GUI_WINDOW", true], " und ", ["GUI_WIDGET", true], " sind seine Typen), über den man später an das Ding herankommt."]),

  H.p("Die Schleife ist kurz und wird es bleiben: GUI_UPDATE verarbeitet, was Maus und Tastatur seit dem letzten Bild getan haben; CLS löscht, GUI_DRAW zeichnet alle Fenster der gui, FLIP zeigt das Bild."),

  H.figure("kap01_1_fenster.png", "Das erste Fenster. Die blaue Leiste ist die Titelleiste des gui-Fensters — ein Fenster im Fenster des Programms.", 440, 260),

  H.h2("Schritt 2: Feld, Knopf, Antwort"),

  H.p("Ein gui-Fenster kann man verschieben, schließen und übereinanderlegen; das ist praktisch für Werkzeugfenster in einem Spiel. Eine Anwendung will meist, dass ihr Fenster das ganze Programmfenster ist. Dafür schaltet man den Rahmen des gui-Fensters ab:"),

  H.code(stueck("kap01/2_eingabe.dh", "GUI_WINDOW_CHROME")),

  H.p("Dann kommen die Bausteine, die man in jedem Formular braucht:"),

  H.code(stueck("kap01/2_eingabe.dh", "DIM beschriftung AS GUI_WIDGET", "status = GUI_LABEL")),

  H.pmix(["Die Zahlen sind Lage und Größe in Punkten, gemessen von der linken oberen Ecke des Fensters: ", ["GUI_TEXTINPUT(fenster, 80, 20, 280, 28, \"z. B. Momo\")", true], " ist ein Feld 80 Punkte vom linken Rand, 20 vom oberen, 280 breit und 28 hoch. Der letzte Text ist der Platzhalter — er steht grau im leeren Feld und verschwindet, sobald man tippt."]),

  H.p("In der Schleife kommt eine einzige Abfrage dazu:"),

  H.code(stueck("kap01/2_eingabe.dh", "IF GUI_CLICKED(knopf)", "END IF")),

  H.pmix([["GUI_CLICKED(knopf)", true], " ist in genau dem Bild wahr, in dem jemand den Knopf geklickt hat — gedrückt und über dem Knopf wieder losgelassen. ", ["GUI_TEXT(feld)", true], " liest, was im Feld steht, ", ["GUI_SET_TEXT(status, …)", true], " schreibt in die Beschriftung."]),

  H.figure("kap01_2_eingabe.png", "Der zweite Stand: ohne die Titelleiste füllt die gui das Programmfenster. Noch nimmt der Knopf alles an, auch ein leeres Feld.", 440, 260),

  H.h2("Schritt 3: die erste Mediathek"),

  H.p("Zum Aufnehmen fehlen drei Dinge: ein Ort für die Titel, eine Prüfung und Enter als zweiter Weg. Die Titel liegen in einem Feld, das mit jedem Titel wächst:"),

  H.code(stueck("kap01/3_mediathek.dh", "DIM titel AS ARRAY OF STRING")),

  H.p("Was beim Klick passiert, ist inzwischen zu lang für eine Zeile in der Schleife. Es bekommt eine eigene SUB:"),

  H.code(stueck("kap01/3_mediathek.dh", "SUB aufnehmen", "END SUB")),

  H.pmix([["TRIM$", true], " nimmt Leerzeichen vorn und hinten weg; wer nur die Leertaste gedrückt hat, hat nichts eingegeben. Ist das Feld leer, sagt die Statuszeile es, und sonst passiert nichts. Andernfalls wird der Titel angehängt, die Zeilen darunter zeigen den neuen Stand, und das Feld wird geleert. Zum Schluss bekommt das Feld den Fokus zurück — die Schreibmarke blinkt wieder darin, und man kann den nächsten Titel tippen, ohne erst hineinzuklicken."]),

  H.p("Enter als zweiter Weg ist eine Zeile:"),

  H.code(stueck("kap01/3_mediathek.dh", "GUI_WINDOW_DEFAULT")),

  H.p("Damit ist der Knopf der Standard-Knopf des Fensters: Enter in einem Eingabefeld drückt ihn, und er trägt einen hellen Rahmen, damit man sieht, was Enter tun wird. Das ganze Programm:"),

  H.code(zeilen("kap01/3_mediathek.dh")),

  H.figure("kap01_3_mediathek.png", "Die erste Mediathek beim Start. Der Rahmen um „Aufnehmen“ zeigt den Standard-Knopf, die Schreibmarke steht schon im Feld.", 440, 260),

  H.h2("Warum so?"),

  H.h3("Einmal bauen, in jedem Bild fragen"),

  H.p("Die gui arbeitet mit Bausteinen, die man EINMAL anlegt und die dann bleiben. Das Feld merkt sich selbst, was darin steht, wo die Schreibmarke ist, was markiert ist und was Strg+Z zurücknehmen würde. Das Programm fragt nur nach, was es wissen will. Die Alternative ist ein Baukasten, bei dem man in jedem Bild jeden Knopf neu hinzeichnet und dabei fragt, ob er gedrückt ist (das Modul ui arbeitet so). Das ist für ein paar Knöpfe in einem Spiel bequem, für ein Formular aber mühsam: Den Text jedes Felds, den Fokus, die Markierung müsste dann das Programm selbst halten."),

  H.h3("Ein Ereignis gilt genau ein Bild"),

  H.pmix([["GUI_CLICKED", true], " ist nicht „der Knopf ist gedrückt“, sondern „der Knopf wurde soeben geklickt“ — wahr in einem einzigen Bild und danach wieder falsch. Darum steht die Abfrage in der Schleife und wird in jedem Bild gestellt. Ein Klick geht so nie verloren und wird nie doppelt gezählt: Wer zehnmal klickt, bekommt zehn Bilder, in denen die Abfrage wahr ist, egal wie lange er die Maustaste jeweils hält."]),

  H.h3("Was das Programm setzt, ist keine Bedienung"),

  H.pmix(["Nach dem Aufnehmen leert ", ["GUI_SET_TEXT(feld, \"\")", true], " das Feld. Für die gui ist das keine Änderung durch den Benutzer: ", ["GUI_CHANGED(feld)", true], " bleibt falsch, und auch Strg+Z holt den alten Text nicht zurück. Das ist Absicht. Würde ein vom Programm gesetzter Wert als Bedienung zählen, löste jede Antwort des Programms das nächste Ereignis aus — und zwei Felder, die sich gegenseitig nachführen, schaukelten sich auf."]),

  H.h3("Enter über den Standard-Knopf"),

  H.pmix(["Man könnte Enter auch selbst abfragen, mit ", ["KEYHIT(KEY_ENTER)", true], ". Dann drückte Enter aber immer den Knopf — auch wenn der Fokus gerade in einem mehrzeiligen Textbereich steht, wo Enter eine neue Zeile sein soll, oder in einer Tabellenzelle, deren Bearbeitung Enter abschließt. ", ["GUI_WINDOW_DEFAULT", true], " kennt diese Fälle: Die Taste gehört zuerst dem Baustein mit dem Fokus und erst dann dem Standard-Knopf. Dasselbe gibt es für Esc mit ", ["GUI_WINDOW_CANCEL", true], "."]),

  H.h3("Schlafen, wenn nichts passiert"),

  H.pmix(["Ein Spiel zeichnet sechzigmal in der Sekunde, und das ist richtig, denn es bewegt sich ständig etwas. Die Mediathek ändert sich nur, wenn jemand etwas tut. ", ["WINDOW_WAIT_EVENTS(1)", true], " lässt FLIP warten, bis eine Eingabe kommt, höchstens eine Sekunde lang. Die gui weiß selbst, wann sie trotzdem ein Bild braucht (die Schreibmarke blinkt), und weckt das Programm rechtzeitig. Am Laptop ist das der Unterschied zwischen einer Anwendung, die den Akku in Ruhe lässt, und einer, die ihn leert."]),

  H.h2("Was schiefgehen kann"),

  H.h3("Das Modul fehlt"),

  H.code(zeilen("kap01/fehler_ohne_import.dh")),
  H.code(["fehler_ohne_import.dh:2: Compile-Fehler: Unbekannter Typ",
          "'gui_window' -- fehlt IMPORT \"gui\"?"], { out: true }),

  H.pmix(["Ohne ", ["IMPORT \"gui\"", true], " kennt Drachenhauch weder den Typ ", ["GUI_WINDOW", true], " noch die Befehle. Der Fehler kommt schon beim Übersetzen, bevor das Programm läuft, und die Meldung nennt die Abhilfe gleich mit. (Dass der Typ klein geschrieben dasteht, ist kein Fehler: Namen kennen in Drachenhauch keine Groß- und Kleinschreibung.)"]),

  H.h3("GUI_UPDATE vergessen"),

  H.p("Fehlt GUI_UPDATE in der Schleife, zeichnet das Programm sein Fenster tadellos — aber nichts reagiert. Ein Klick in das Feld setzt keine Schreibmarke, der Knopf leuchtet nicht auf, wenn die Maus darüber steht, und GUI_CLICKED wird nie wahr. Eine Meldung gibt es nicht, denn das Programm tut nichts Falsches; es fragt nur nie nach. Wer ein Fenster sieht, das nicht reagiert, sucht als Erstes nach GUI_UPDATE."),

  H.h3("Auf einen Klick warten"),

  H.p("Wer vom Spieleprogrammieren kommt, schreibt vielleicht so etwas:"),

  H.code(["' Warten, bis jemand klickt -- so nicht!",
          "WHILE NOT GUI_CLICKED(knopf)",
          "WEND"]),

  H.p("Das Fenster friert ein, und das System meldet nach ein paar Sekunden „Keine Rückmeldung“. In der inneren Schleife ruft niemand GUI_UPDATE, also kann GUI_CLICKED nie wahr werden, und niemand ruft FLIP, also bekommt das System keine Antwort. In einer Anwendung wartet nur die eine Schleife ganz außen; alles andere fragt nach und kehrt sofort zurück. Wie man trotzdem auf eine Antwort wartet — „Wirklich löschen?“ —, zeigt Kapitel 6."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Nimm die Zeile mit GUI_WINDOW_DEFAULT heraus. Was ändert sich am Aussehen, was an der Bedienung?"),
  H.bulletRich("Drehen 2. ", "Nimm TRIM$ heraus. Welche Eingabe kommt jetzt durch, die vorher abgewiesen wurde?"),
  H.bulletRich("Drehen 3. ", "Vertausche in der Schleife GUI_UPDATE und die Zeile mit GUI_CLICKED. Funktioniert der Knopf noch? Sage es vorher und begründe es."),
  H.bulletRich("Erweitern 1. ", "Ein zweiter Knopf „Leeren“ unten rechts leert die Mediathek wieder."),
  H.bulletRich("Erweitern 2. ", "Ein Titel, der schon in der Mediathek steht, wird abgewiesen — auch wenn er anders groß- und kleingeschrieben ist."),
  H.bulletRich("Erfinden. ", "Baue ein kleines Fenster für etwas aus deinem Alltag: einen Zähler für Gläser Wasser am Tag, eine Liste der Pflanzen, die heute Wasser brauchen, ein Feld, das eine Temperatur von Celsius in Fahrenheit umrechnet. Erfinden-Übungen haben keine Musterlösung."),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Modul gui → Konzept: ", "ein Aufbau, dann Polling pro Bild — die Schleife dieses Kapitels mit allen Einzelheiten."),
  H.bulletRich("Handbuch, Modul gui → Button, Label, TextInput: ", "was die drei Bausteine sonst noch können (Passwortfelder, Zahlenfelder, Umbruch, Ausrichtung)."),
  H.bulletRich("Handbuch, Modul gui → Schlafen, solange nichts passiert: ", "WINDOW_WAIT_EVENTS und wann die gui von selbst aufwacht."),
];
