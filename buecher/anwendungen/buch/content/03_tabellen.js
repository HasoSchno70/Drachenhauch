const { zeilen, ohneKopf, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Listen und Tabellen", 3),

  H.h2("Was wir bauen"),

  H.figure("kap03_auswahl.png", "Die Mediathek mit Tabelle: oben das Formular aus Kapitel 2, darunter alle Einträge. „Metropolis“ und „Kind of Blue“ sind gewählt — das zweite mit Strg+Klick dazu.", 460, 315),

  H.p("Unter dem Formular steht jetzt die ganze Mediathek als Tabelle: Titel, Urheber, Art und Jahr, eine Zeile je Eintrag. Ein Klick auf einen Spaltenkopf sortiert, die Zeile unter dem Kopf filtert, mit Strg und Umschalt wählt man mehrere Zeilen auf einmal. „Aufnehmen“ hängt einen Eintrag an, „Entfernen“ nimmt die gewählten heraus, und ein Doppelklick zeigt einen Eintrag ganz in der Statuszeile."),

  H.h2("Der Plan"),

  H.p("Bis jetzt war ein Titel ein Text, und die Mediathek ein Feld von Texten. Ein Eintrag hat aber vier Angaben, und die gehören zusammen — darum wird er eine Klasse. Die Mediathek ist dann ein Feld von Einträgen."),

  H.p("Die Tabelle bekommt dieses Feld nicht. Sie bekommt für jeden Eintrag eine Zeile mit vier Texten, in derselben Reihenfolge wie im Feld: Zeile 0 ist Eintrag 0, Zeile 7 ist Eintrag 7. Was die Tabelle zeigt, ist eine Abschrift. Ändert sich die Mediathek, ändert das Programm zuerst das Feld und dann die Abschrift — und hält beide in derselben Reihenfolge."),

  H.h2("Schritt 1: Einträge und eine Tabelle"),

  H.p("Die Klasse und ein paar Beispiele bekommen eine eigene Datei, eintrag.dh, denn alle Programme dieses Kapitels brauchen sie:"),

  H.code(stueck("kap03/eintrag.dh", "CLASS Eintrag", "END CLASS")),

  H.pmix(["Vier Felder, Init füllt sie beim Anlegen, und ", ["zellen", true], " liefert die vier Texte, die eine Zeile der Tabelle braucht. Die Beispiele baut eine Funktion aus Texten, in denen die Angaben durch | getrennt sind — das ist kürzer als zehnmal NEW mit vier Argumenten:"]),

  H.code(stueck("kap03/eintrag.dh", "FUNCTION beispiele", "END FUNCTION")),

  H.pmix(["Das Programm holt die Datei mit ", ["IMPORT \"eintrag.dh\"", true], " herein — der Text der Datei steht dann da, als hätte man ihn an diese Stelle geschrieben. Danach die Tabelle:"]),

  H.code(stueck("kap03/1_tabelle.dh", "DIM tabelle AS GUI_WIDGET", "NEXT")),

  H.pmix([["GUI_TABLE", true], " legt sie an, ", ["GUI_TABLE_HEADERS", true], " gibt ihr die Spaltenköpfe, ", ["GUI_TABLE_COL_WIDTHS", true], " die Breiten in Punkten. ", ["GUI_TABLE_SET", true], " stellt das Aussehen ein: ", ["zebra", true], " färbt jede zweite Zeile ein wenig heller, damit das Auge in einer breiten Tabelle in der Zeile bleibt, und ", ["zeilenhoehe", true], " gibt den Zeilen etwas Luft. Die Schleife hängt für jeden Eintrag mit ", ["GUI_TABLE_ADD_ROW", true], " eine Zeile an."]),

  H.figure("kap03_sortiert.png", "Nach einem Klick auf „Jahr“: aufsteigend sortiert, der kleine Pfeil im Kopf sagt es. Ein zweiter Klick dreht die Richtung um.", 460, 242),

  H.pmix(["Sortieren muss man nicht programmieren. Ein Klick auf einen Spaltenkopf sortiert nach dieser Spalte, ein zweiter dreht die Richtung um. Die Tabelle sortiert Zahlen als Zahlen — sonst stünde 1000 vor 999 — und alles andere als Text."]),

  H.h2("Schritt 2: Formular und Tabelle zusammen"),

  H.p("Jetzt kommt das Formular aus Kapitel 2 über die Tabelle, in dieselbe Spalte. Die Tabelle bekommt das Gewicht 1 und damit allen Platz, der nach dem Formular und der Knopfreihe übrig ist; wird das Fenster höher, zeigt sie mehr Zeilen:"),

  H.code(stueck("kap03/2_aufnehmen.dh", "DIM tabelle AS GUI_WIDGET", "GUI_LAYOUT_ADD(form, tabelle, 1)")),

  H.pmix(["Neu sind zwei Einstellungen. ", ["filterzeile", true], " blendet unter dem Kopf ein kleines Feld je Spalte ein; wer dort tippt, sieht nur noch die Zeilen, die das Getippte in dieser Spalte enthalten, egal ob groß oder klein geschrieben. ", ["GUI_TABLE_PLACEHOLDER", true], " ist der Text, der in einer Tabelle ohne sichtbare Zeile steht — damit eine leere Tabelle nicht aussieht wie eine kaputte."]),

  H.p("Beim Aufnehmen entsteht ein neuer Eintrag aus den Feldern. Er kommt zuerst ins Feld der Mediathek und dann als Zeile in die Tabelle:"),

  H.code(stueck("kap03/2_aufnehmen.dh", "SUB aufnehmen", "END SUB")),

  H.pmix([["GUI_TABLE_ADD_ROW", true], " liefert die Nummer der neuen Zeile, und ", ["GUI_TABLE_SET_SELECTED", true], " wählt sie aus — so sieht man gleich, was dazugekommen ist. Das Jahr bleibt hier ein Text, so wie man es eintippt; mit Jahren gerechnet wird erst, wenn die Mediathek in einer Datenbank liegt."]),

  H.figure("kap03_2_aufnehmen.png", "Formular und Tabelle in einem Fenster. Unter jedem Spaltenkopf liegt ein Filterfeld.", 460, 315),

  H.h2("Schritt 3: wählen, entfernen, ansehen"),

  H.p("Eine Zeile wählt man mit einem Klick. Mehrere auf einmal erlaubt die Tabelle erst nach einer Einstellung:"),

  H.code(stueck("kap03/3_auswahl.dh", "GUI_TABLE_SET(tabelle, \"mehrfachauswahl\"")),

  H.p("Dann nimmt Strg+Klick eine Zeile dazu oder wieder heraus, und Umschalt+Klick wählt alles zwischen der zuletzt angeklickten Zeile und dieser. Ein neuer Knopf „Entfernen“ nimmt alle gewählten Einträge heraus:"),

  H.code(stueck("kap03/3_auswahl.dh", "SUB auswahl_entfernen", "END SUB")),

  H.pmix([["GUI_TABLE_SEL_COUNT", true], " sagt, wie viele Zeilen gewählt sind, ", ["GUI_TABLE_SEL_ROW(tabelle, i)", true], " welche — in der Reihenfolge, in der sie angeklickt wurden. Die Nummern kommen in ein Feld, und SORT mit TRUE sortiert es absteigend. Dann wird jede Nummer aus beiden entfernt, aus dem Feld der Mediathek mit ", ["ARRAY_REMOVE_AT", true], " und aus der Tabelle mit ", ["GUI_TABLE_REMOVE_ROW", true], ". Warum von hinten nach vorn, zeigt der Abschnitt „Was schiefgehen kann“."]),

  H.p("Der Doppelklick ist eine Abfrage in der Schleife, wie ein Klick auf einen Knopf:"),

  H.code(stueck("kap03/3_auswahl.dh", "IF GUI_DOUBLE_CLICKED(tabelle)", "END IF")),

  H.pmix([["GUI_DOUBLE_CLICKED", true], " ist in dem Bild wahr, in dem eine Zeile doppelt angeklickt wurde — oder Enter auf einer gewählten Zeile gedrückt; die Tastatur kann alles, was die Maus kann. ", ["GUI_TABLE_SELECTED", true], " ist dann die Zeile, und zeigen schreibt ihren Eintrag in die Statuszeile:"]),

  H.code(stueck("kap03/3_auswahl.dh", "SUB zeigen", "END SUB")),

  H.h2("Warum so?"),

  H.h3("Die Tabelle hält die Daten nicht"),

  H.p("Man könnte die Mediathek ganz in der Tabelle lassen — sie hat ja alle Texte — und mit GUI_TABLE_GET_CELL nachlesen, was man wissen will. Das geht, bis man mehr wissen will, als die Tabelle zeigt: wann ein Eintrag aufgenommen wurde, an wen er verliehen ist, wo das Cover liegt. Und es geht, bis die Tabelle etwas anders zeigen soll, als es gespeichert ist: ein Jahr rechtsbündig, eine Art übersetzt, ein Datum in der Schreibweise des Landes. Dann muss man aus der Anzeige wieder Daten machen, und das ist der Anfang vieler Fehler."),

  H.p("Darum gehören die Daten in eine Struktur des Programms — hier ein Feld von Einträgen, ab Kapitel 4 eine Datenbank —, und die Tabelle zeigt nur. Sie ist eine Ansicht. Das hat auch einen praktischen Vorteil: Die Klasse Eintrag kann man prüfen, speichern und durchsuchen, ohne dass ein Fenster offen ist."),

  H.h3("Sortieren und Filtern stellen nichts um"),

  H.p("Nach einem Klick auf „Jahr“ steht Metropolis oben. Ist Metropolis jetzt Zeile 0? Nein — und das ist der Grund, warum das Entfernen auch in einer sortierten Tabelle das Richtige trifft. Sortieren und Filtern ändern nur, in welcher Reihenfolge die Tabelle ihre Zeilen ZEIGT. Jede Zeilennummer, die man ihr gibt oder von ihr bekommt — GUI_TABLE_SELECTED, GUI_TABLE_SEL_ROW, GUI_TABLE_REMOVE_ROW —, ist die Nummer der Datenzeile, so wie sie angehängt wurde. Zeile 1 bleibt Metropolis, egal wo sie gerade steht, und Zeile 1 ist Eintrag 1 im Feld."),

  H.pmix(["Wer doch einmal wissen muss, was an welcher Stelle zu sehen ist — für einen Ausdruck der sortierten Liste etwa —, fragt ", ["GUI_TABLE_VIEW_ROW(tabelle, i)", true], ": Welche Datenzeile steht an sichtbarer Stelle i?"]),

  H.h3("Erst die Daten, dann die Anzeige"),

  H.p("In aufnehmen kommt der Eintrag zuerst ins Feld und dann in die Tabelle, in auswahl_entfernen wird zuerst aus dem Feld und dann aus der Tabelle entfernt. Die Reihenfolge ist eine Gewohnheit, die sich lohnt: Die Daten sind die Wahrheit, die Anzeige folgt ihnen. Geht beim Ändern der Daten etwas schief — in Kapitel 4 kann die Datenbank ablehnen —, hat die Anzeige noch nichts Falsches gezeigt."),

  H.h2("Was schiefgehen kann"),

  H.h3("Von vorn nach hinten entfernen"),

  H.p("Ohne das SORT in auswahl_entfernen kommen die Nummern in der Reihenfolge der Klicks; wer von oben nach unten klickt, entfernt von vorn nach hinten. Sind Metropolis (Zeile 1) und Tetris (Zeile 2) gewählt, passiert dann das:"),

  H.bulletRich("Zeile 1 entfernen: ", "Metropolis ist weg. Alles dahinter rückt eine Nummer auf — Tetris ist jetzt Zeile 1, Kind of Blue Zeile 2."),
  H.bulletRich("Zeile 2 entfernen: ", "Das ist jetzt Kind of Blue. Tetris bleibt."),

  H.p("Es gibt keine Meldung, das Programm entfernt nur das Falsche. Von hinten nach vorn ändert das Entfernen einer Zeile nichts an den Nummern davor, und die nächste Nummer stimmt noch. Dieselbe Regel gilt für jedes Feld, aus dem man in einer Schleife mehrere Dinge entfernt."),

  H.h3("Eine Zeile nur in der Tabelle"),

  H.p("Kommt eine Zeile in die Tabelle, aber kein Eintrag ins Feld, laufen beide auseinander: Die Tabelle hat elf Zeilen, das Feld zehn Einträge. Sichtbar wird das erst, wenn jemand die neue Zeile anfasst:"),

  H.code(stueck("kap03/fehler_nur_tabelle.dh", "' Aufgenommen", "PRINT mediathek[z].titel")),
  H.code(["Laufzeitfehler in fehler_nur_tabelle.dh:22: Index 10 ausserhalb",
          "[0..9] in Dimension 1 -- Felder zaehlen ab 0: DIM a[10] hat a[0]",
          "bis a[9]"], { out: true }),

  H.p("Die Tabelle hat eine Zeile 10, das Feld nicht. Schlimmer ist der umgekehrte Fall — ein Eintrag fehlt in der Tabelle, und ab da zeigt jede Zeile dahinter auf den Eintrag danach. Dann gibt es keine Meldung, sondern ein Doppelklick auf „Tetris“ zeigt „Kind of Blue“. Wer Daten und Anzeige getrennt hält, ändert sie darum immer an derselben Stelle zusammen: in einer SUB wie aufnehmen, nie verteilt über das Programm."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Sortiere die Tabelle nach Jahr, wähle „Tetris“ und entferne es. Welcher Eintrag verschwindet aus dem Feld mediathek? Sage es vorher und begründe es."),
  H.bulletRich("Drehen 2. ", "Nimm die Zeile mit \"mehrfachauswahl\" heraus. Was tut Strg+Klick jetzt, und was entfernt der Knopf?"),
  H.bulletRich("Drehen 3. ", "Tippe in die Filterzeile unter „Urheber“ das Wort ende. Welche Zeilen bleiben, und was zeigt ein Doppelklick auf die zweite?"),
  H.bulletRich("Erweitern 1. ", "Jede Art bekommt ihre Farbe: In der Spalte „Art“ stehen Bücher blau, Filme rot, Spiele grün und Musik gelb — auch bei neu aufgenommenen Einträgen."),
  H.bulletRich("Erweitern 2. ", "Ein Suchfeld über der Tabelle sucht in allen Spalten zugleich: Wer „ende“ tippt, sieht Momo und Die unendliche Geschichte, wer „19“ tippt, alles aus dem 20. Jahrhundert. Entfernen und Doppelklick müssen weiter den richtigen Eintrag treffen."),
  H.bulletRich("Erfinden. ", "Eine Tabelle für etwas, das du sammelst oder zählst: Pflanzen mit dem Tag, an dem sie zuletzt Wasser bekamen, die Spiele eines Vereins mit Ergebnis, Rezepte mit Zeit und Schwierigkeit. Mit einer eigenen Klasse für eine Zeile."),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Modul gui → Tabelle: ", "alles über Zellen, Farben, Sortieren, Filtern, Mehrfachauswahl und die Bedienung mit der Tastatur."),
  H.bulletRich("Handbuch, Modul gui → Zeilennummern: Daten oder Ansicht?: ", "warum alle Nummern Datenzeilen sind und wie man die sichtbare Reihenfolge abfragt."),
  H.bulletRich("Beispiel examples/157_gui_tabelle.dh: ", "eine Serverliste mit Bildern, Balken, Häkchen und Knöpfen in den Zellen."),
];
