module.exports = (H) => [
  H.chapter("Vorwort"),

  H.p("Dieses Buch baut eine Anwendung: eine Mediathek für die Bücher, Filme, Spiele und Platten eines Haushalts. Am Ende hat sie Fenster mit Formularen und Tabellen, eine Datenbank, eine Suche, ein Diagramm, eine Ausleihliste zum Drucken und einen Export nach Excel; sie spricht Deutsch und Englisch, lässt sich mit der Tastatur und mit einem Bildschirmleser bedienen und wird als Programm weitergegeben, das ohne Drachenhauch läuft."),

  H.p("Der Weg dorthin hat zwei Hälften. Im ersten Teil entsteht eine schlichte Mediathek von Hand, mit dem gui-Modul in der IDE: jedes Fenster, jeder Knopf und jede Abfrage als Zeile Code. Ab dem zweiten Teil wird sie im Drachenhauch Studio noch einmal gebaut, wie man Anwendungen eigentlich baut — Fenster zeichnen, Eigenschaften einstellen, Ereignisse schreiben. Der Umweg ist Absicht: Wer einmal von Hand gesehen hat, was ein Knopf braucht, versteht, was das Studio ihm abnimmt, und weiß, wo er nachsehen muss, wenn etwas nicht tut, was es soll."),

  H.h2("Für wen"),

  H.p("Für alle, die die Sprache in den Grundzügen kennen: Variablen, Felder, Funktionen, eine erste Klasse. Das ist der Stoff des Buchs „Der Einstieg“ oder des Buchs „Kleine Wunder“ bis zu seinem zweiten Teil. Erfahrung mit Oberflächen braucht es nicht; was ein Ereignis ist und warum ein Fenster eine Klasse ist, erklärt das Buch."),

  H.h2("Wie ein Kapitel aufgebaut ist"),

  H.bulletRich("Was wir bauen: ", "ein Bild des Ergebnisses und zwei Sätze dazu."),
  H.bulletRich("Der Plan: ", "was entstehen soll, in Worten, bevor eine Zeile Code dasteht."),
  H.bulletRich("Schritt für Schritt: ", "der Code oder die Handgriffe im Studio, nach jedem Schritt ein Stand, der schon läuft."),
  H.bulletRich("Warum so? ", "die Entscheidungen, jeweils mit der naheliegenden Alternative und dem Grund, warum sie hier schlechter wäre."),
  H.bulletRich("Was schiefgehen kann: ", "echte Fehler mit dem Wortlaut, den dhrt und das Studio wirklich zeigen."),
  H.bulletRich("Übungen ", "in drei Stufen: Drehen, Erweitern, Erfinden. Lösungen in Anhang A."),
  H.bulletRich("Zum Weiterlesen: ", "die Stellen im Handbuch, die das Thema vollständig beschreiben."),

  H.h2("Die Programme und Bilder"),

  H.p("Jedes Programm, jedes Studio-Projekt und jede Lösung liegt im Ordner buecher/anwendungen/code des Drachenhauch-Repositorys. Das Buch druckt den Code aus diesen Dateien ab, und eine Prüfsammlung lässt sie bei jeder neuen Fassung laufen. Auch die Bilder entstehen aus ihnen: die Fenster der Programme mit dhrt bild, die Bilder des Studios, indem ein Skript das Studio startet und bedient. Ein Bild, das von Hand gemacht ist, veraltet still; diese lassen sich mit jeder Fassung neu ziehen."),

  H.note("Dieser Stand enthält Teil I, Kapitel 1 bis 7 — die Mediathek von Hand —, und als Probe für Teil II Kapitel 8 im Studio, alle mit ihren Lösungen. Die übrigen Kapitel folgen nach der Gliederung in docs/intern/entwurf-buecher.md; Kapitel 0, der Rundgang durch die fertige Mediathek, kommt, wenn sie fertig ist.", "Arbeitsstand"),
];
