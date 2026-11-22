const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Datenbindung", 14),

  H.h2("Was wir bauen"),

  H.figure("kap14_inspektor.png", "Die Tabelle des Hauptfensters ist gewählt. In der Gruppe Daten des Inspektors steht, woher sie ihre Zeilen holt: aus der Tabelle eintrag, vier Spalten, nach Titel sortiert, gefiltert über das Suchfeld.", 480, 262),

  H.p("In Kapitel 13 hat das Modul Bestand jede Zeile von Hand gelesen und geschrieben: SELECT in eine Klasse, die Klasse in die Tabelle, die Felder zurück in ein UPDATE. In diesem Kapitel übernimmt das die Bindung. Das Bearbeitungsfenster hängt an der Tabelle eintrag, jedes Feld an seiner Spalte; Laden und Sichern sind je ein Aufruf. Die Tabelle im Hauptfenster holt ihre Zeilen selbst, sortiert sie und filtert, während man ins Suchfeld tippt."),

  H.h2("Der Plan"),

  H.p("Die Bindung steht im Inspektor, in der Gruppe Daten. Drei Angaben genügen:"),

  H.bulletRich("Tabelle (am Fenster): ", "zu welcher Tabelle das Fenster gehört."),
  H.bulletRich("Spalte (an einem Feld): ", "welche Spalte das Feld zeigt und schreibt."),
  H.bulletRich("Datenquelle (an einer Liste, Tabelle oder Klappliste): ", "woher sie ihre Zeilen holt, dazu Angezeigte Spalten, Sortieren nach, Bedingung und Suchfeld."),

  H.p("Aus diesen Angaben schreibt das Studio Methoden in die Basisklasse des Fensters:"),

  H.table([
    ["Laden(id)", "holt die Zeile in die Felder; DatensatzId ist dann id"],
    ["Sichern()", "prüft die Regeln der Felder und schreibt — eine neue Zeile, wenn DatensatzId 0 ist"],
    ["Neu()", "leert die Felder und setzt die Vorgaben der Spalten"],
    ["Loeschen()", "löscht die Zeile, die gerade dasteht"],
    ["Ungesichert()", "hat sich seit Laden oder Sichern etwas geändert?"],
    ["Neuladen()", "holt die Zeilen jeder Liste mit Datenquelle neu"],
    ["IdVon(liste)", "die id der gewählten Zeile einer Liste"],
  ], { headers: ["Methode", "tut"], widths: [2400, 6626] }),

  H.h2("Schritt 1: das Bearbeitungsfenster binden"),

  H.p("Im Inspektor des EintragFenster bekommt das Fenster die Tabelle Mediathek.eintrag. Dann jedes Feld seine Spalte: feldTitel titel, feldUrheber urheber, wahlArt art, feldJahr jahr. Eine Klappliste mit festen Einträgen bindet an einer Textspalte den Text des gewählten Eintrags — in art steht danach „Film“, nicht die Nummer 1. Der Code des Fensters schrumpft auf das, was keine Bindung wissen kann:"),

  H.code(zeilen("kap14/mediathek/EintragFenster.dh")),

  H.p("Laden(id) füllt die vier Felder aus der Zeile. Sichern() prüft die Regeln, schreibt die Felder in die Zeile mit DatensatzId — oder legt eine neue an — und liefert, ob es geklappt hat. Die Prüfregel ist dieselbe wie in Kapitel 5: Dass titel in der Tabelle Pflicht ist, weiß das Fenster nicht von selbst."),

  H.h2("Schritt 2: die Tabelle an eine Datenquelle"),

  H.p("Über der Tabelle des Hauptfensters steht jetzt ein Suchfeld, feldSuche. Die Tabelle bekommt im Inspektor die Datenquelle Mediathek.eintrag, die Angezeigten Spalten titel;urheber;art;jahr, Sortieren nach titel und als Suchfeld feldSuche. Sie behält ihre Köpfe aus dem Entwurf, weil es so viele sind, wie sie Spalten zeigt. Beim Öffnen füllt das Fenster sie selbst; tippt man ins Suchfeld, zeigt sie nur noch Zeilen, in deren Spalten der Text vorkommt."),

  H.p("Auch das Hauptfenster bekommt die Tabelle Mediathek.eintrag. Dann lädt ein Klick auf eine Zeile sie — ohne Felder, aber DatensatzId ist danach ihre id, und nach Neuladen bleibt sie gewählt. Neu und Bearbeiten geben nur noch die id weiter:"),

  H.code(stueck("kap14/mediathek/Hauptfenster.dh", "FUNCTION bearbeitenLassen", "END FUNCTION")),

  H.p("Entfernen ist Loeschen(): Die Zeile geht aus der Tabelle eintrag, ihre Ausleihen gehen mit (der Verweis aus Kapitel 13), und die Liste lädt sich neu:"),

  H.code(stueck("kap14/mediathek/Hauptfenster.dh", "SUB knopfEntfernen_Klick", "END SUB")),

  H.p("Das Ausleihfenster folgt der Auswahl über die id:"),

  H.code(stueck("kap14/mediathek/Hauptfenster.dh", "SUB tabelle_Geaendert", "END SUB")),

  H.h2("Schritt 3: was übrig bleibt"),

  H.p("Die Klasse Eintrag braucht nur noch das Modul Beispiele. Bestand füllt die Beispiele in eine leere Datei — jetzt aus der App, bevor das erste Fenster seine Tabelle lädt — und gibt Auskünfte: wie viele Einträge, welcher Titel, bei wem. Das Ausleihfenster merkt sich nur noch die id des Eintrags und fragt die Datenbank, was es zeigen soll."),

  H.code(zeilen("kap14/mediathek/App.dh")),

  H.p("Eine Spalte fehlt: „Verliehen an“ steht nicht in der Tabelle eintrag, sondern ergibt sich aus ausleihe und freund. Die Datenquelle einer Tabelle kann sie nicht zeigen. Kapitel 15 holt sie mit einer Abfrage zurück."),

  H.h2("Warum so?"),

  H.h3("Was die Bindung tut"),

  H.p("In Kapitel 5 hat die Mediathek jedes Feld mit GUI_BIND an seine Spalte gehängt und mit GUI_FORM_LOAD und GUI_FORM_SAVE geladen und gesichert. Die Bindung im Studio tut genau dasselbe; nur schreibt das Studio die Aufrufe in die Basisklasse, statt dass man sie tippt. Wer wissen will, was Sichern() tut, findet es in _studio/EintragFenster_basis.dh: ein GUI_VALIDATE und ein GUI_FORM_SAVE, sonst nichts."),

  H.p("Das ist der Preis und der Gewinn zugleich. Was die Bindung kann, ist mit einem Haken erledigt und hat keine Tippfehler. Was sie nicht kann — eine Prüfregel, eine Spalte aus zwei Tabellen, eine Auskunft wie „bei wem“ —, steht weiter im eigenen Code, und man sieht an ihm genau, was das Programm über die Bindung hinaus tut."),

  H.h2("Was schiefgehen kann"),

  H.h3("Pflicht ist keine Prüfregel"),

  H.p("Ohne GUI_RULE in Geoeffnet nimmt das Fenster einen Eintrag ohne Titel an: Sichern() findet keine Regel, die verletzt ist, und schreibt einen leeren Text in titel. Die Datenbank lehnt ihn nicht ab — NOT NULL verbietet NULL, nicht den leeren Text. In der Tabelle steht danach eine leere Zeile."),

  H.h3("Ein Feld ohne Spalte"),

  H.p("Wer vergisst, einem Feld seine Spalte zu geben, bekommt keine Meldung. Laden füllt es nicht, Sichern schreibt es nicht; beim Bearbeiten von Metropolis bleibt Urheber leer, und was man dort einträgt, ist beim nächsten Öffnen fort. Ein Blick in den Inspektor jedes Feldes hilft — oder in die Basisklasse: Dort steht jede gebundene Spalte."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Lösche bei der Tabelle „Sortieren nach“. In welcher Reihenfolge stehen die Einträge?"),
  H.bulletRich("Drehen 2. ", "Tippe ins Suchfeld „19“. Was bleibt — und warum?"),
  H.bulletRich("Drehen 3. ", "Nimm der Tabelle im Entwurf einen ihrer vier Köpfe. Was steht danach über den Spalten?"),
  H.bulletRich("Erweitern 1. ", "Ein Menü Ansicht mit „Verliehen …“ öffnet ein Fenster mit einer Tabelle, die nur zeigt, was gerade verliehen ist — ohne eine Zeile Code für die Auswahl."),
  H.bulletRich("Erweitern 2. ", "Ein Fenster für die Freunde: links die Liste, rechts Name und Telefon, dazu Neu, Sichern und Löschen. Was geschieht, wenn man Lena löscht?"),
  H.bulletRich("Erfinden. ", "Binde die Tabellen deiner Sammlung aus Kapitel 13 an ein Fenster: eine Liste, ein Formular daneben, und so wenig eigener Code wie möglich."),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Drachenhauch Studio → Datenbanken → Fenster an eine Tabelle binden: ", "Ziehen aus dem Navigator, alle Methoden, Bilder, Verweise in einer Liste."),
  H.bulletRich("Handbuch, Modul gui → Datenbindung: ", "GUI_BIND, GUI_FORM_LOAD und GUI_FORM_SAVE — was die Bindung im Studio aufruft."),
];
