const fs = require("fs");
const path = require("path");

// Die fertige Mediathek in Zahlen, gezaehlt am Stand von Kapitel 24 --
// aendert sich das Projekt, aendert sich der Satz mit.
const FERTIG = path.join(__dirname, "..", "..", "code", "kap24", "mediathek");
const dateien = fs.readdirSync(FERTIG);
const dh = dateien.filter((f) => f.endsWith(".dh"));
const zeilen = dh.reduce((n, f) => n + fs.readFileSync(path.join(FERTIG, f), "utf8")
  .replace(/\r/g, "").replace(/\n+$/, "").split("\n").length, 0);
const fenster = dateien.filter((f) => f.endsWith(".dhform")).length;
const berichte = dateien.filter((f) => f.endsWith(".dhreport")).length;
const ZAHL = ["null", "eins", "zwei", "drei", "vier", "fünf", "sechs", "sieben", "acht", "neun", "zehn", "elf", "zwölf"];
const zahl = (n) => (n < ZAHL.length ? ZAHL[n] : String(n));

module.exports = (H) => [
  H.chapter("Was eine Anwendung braucht", 0),

  H.p("Bevor die erste Zeile entsteht, ein Gang durch das, was am Ende dastehen wird: die fertige Mediathek, Fenster für Fenster, mit dem Kapitel, in dem jedes Stück gebaut wird. Dazu die Frage, was eine Anwendung eigentlich von einem Spiel unterscheidet, und welches der beiden Werkzeuge von Drachenhauch — die IDE oder das Studio — man wann nimmt."),

  H.h2("Spiel und Anwendung"),

  H.p("Ein Spiel rechnet in jedem Bild die Welt weiter. Die Rakete fliegt, ob jemand eine Taste drückt oder nicht; sechzig Mal in der Sekunde wird alles neu gezeichnet, weil sich fast immer etwas bewegt. Das Programm bestimmt, was geschieht, und die Spielerin greift ein."),

  H.p("Eine Anwendung wartet. Solange niemand klickt oder tippt, gibt es nichts zu tun — das Fenster zeigt, was es vorher zeigte. Kommt ein Klick, passiert genau eine Sache: Ein Eintrag wird gespeichert, ein Fenster geht auf, eine Liste wird gefiltert. Der Mensch bestimmt, was geschieht, und das Programm antwortet. Darum spricht man von Ereignissen: Ein Knopf wurde geklickt, ein Text hat sich geändert, ein Menüeintrag wurde gewählt."),

  H.p("In Drachenhauch laufen beide über dieselbe Schleife, Bild für Bild. Der Unterschied liegt darin, was darin steht. Im Spiel ist es die Welt; in der Anwendung sind es Fragen: Wurde dieser Knopf geklickt? Hat sich jenes Feld geändert? Teil I schreibt diese Fragen von Hand, um zu sehen, wie es geht; ab Teil II stellt sie das Studio selbst, und man schreibt nur noch die Antwort. Ein Programm aus dem Studio kann sogar schlafen, bis etwas passiert, und spart so den Akku eines Laptops; Kapitel 16 kommt darauf zurück."),

  H.p("Und eine Anwendung braucht vieles, was ein Spiel nicht kennt: Daten, die das Programmende überleben. Formulare, die falsche Eingaben abweisen. Listen zum Sortieren und Suchen, Ausdrucke, eine Bedienung mit der Tastatur, eine zweite Sprache, einen Installer. Wie viel das ist, zeigt der Rundgang."),

  H.h2("Ein Rundgang durch die Mediathek"),

  H.p("Die Mediathek verwaltet, was in einem Haushalt an Büchern, Filmen, Musik und Spielen steht — und wer sich gerade etwas davon geliehen hat. So sieht sie am Ende des Buchs aus:"),

  H.figure("kap00_mediathek.png", "Die fertige Mediathek: oben die Suche, darunter der Bestand als Tabelle, rechts eine Kachel mit dem gewählten Eintrag und seinem Cover, unten die Knöpfe für alles, was man mit einem Eintrag tun kann.", 480, 320),

  H.p("Das Hauptfenster ist eine Tabelle mit allem, was da ist. Ein Klick auf eine Spaltenüberschrift sortiert, das Suchfeld darüber filtert schon beim Tippen — nach Titel, Urheber, Art, Jahr oder dem Namen dessen, der etwas geliehen hat. Die Spalte „Verliehen an“ zeigt, wer was hat: Tetris ist gerade bei Lena. Rechts steht eine Kachel mit dem gewählten Eintrag und seinem Cover; sie ist ein eigener Baustein, der sich wie ein Knopf oder ein Feld in jedes Fenster ziehen lässt. Unten zählt eine Zeile die Einträge."),

  H.p("Darüber liegen vier Menüs. Datei zeigt eine Ausleihliste oder die Karteikarte eines Eintrags zum Drucken, sichert alles als Excel-Mappe, liest eine alte Liste ein und zeigt die Mediathek als Web-Seite im Heimnetz. Bearbeiten hat dieselben Befehle wie die Knöpfe, mit Kürzeln: Strg+N, Strg+E, Entf, und Strg+F springt in die Suche. Ansicht öffnet die Liste der überfälligen Ausleihen und die Statistik und schaltet zwischen dunklem und hellem Aussehen um. Hilfe prüft die Barrierefreiheit und sagt, welche Fassung läuft."),

  H.h3("Ein Eintrag"),

  H.figure("kap00_bearbeiten.png", "Bearbeiten …: ein eigenes Fenster für einen Eintrag. Solange es offen ist, wartet das Hauptfenster.", 480, 320),

  H.p("„Neu …“ und „Bearbeiten …“ öffnen dasselbe Formular. Wer die ISBN eines Buchs kennt, tippt sie ein und klickt auf „Nachschlagen“: Die Mediathek fragt eine Bibliothek im Netz und füllt Titel, Urheber, Jahr und Cover selbst aus. Das Formular prüft, bevor es speichert — ohne Titel geht nichts, und ins Feld für das Jahr lassen sich nur Ziffern tippen —, und sagt in einer Zeile über den Knöpfen, was nicht stimmt. Solange es offen ist, lässt sich im Hauptfenster nichts anklicken: Das Formular ist modal, es will erst beantwortet werden."),

  H.h3("Ausleihen und zurück"),

  H.figure("kap00_ausleihen.png", "Ausleihen …: Momo ist da und kann verliehen werden. Dieses Fenster ist nicht modal — das Hauptfenster bleibt bedienbar.", 480, 320),

  H.p("Das Fenster zum Ausleihen sagt, ob der gewählte Eintrag da ist oder bei wem er liegt. Ein Name und „Verleihen“ tragen die Ausleihe ein; „Zurück“ beendet sie. Anders als das Formular bleibt dieses Fenster offen, während man im Hauptfenster einen anderen Eintrag wählt — es folgt der Auswahl. Was zu lange weg ist, zeigt Ansicht → Überfällig, mit der Zahl der Tage, die längste Ausleihe zuerst."),

  H.h3("Zahlen und Bilder"),

  H.figure("kap00_statistik.png", "Ansicht → Statistik: Einträge je Art als Kuchen, Ausleihen je Monat als Balken. Ein Klick auf ein Stück zeigt darunter, was dazugehört.", 480, 320),

  H.p("Die Statistik zählt mit SQL, was in der Datenbank steht, und zeichnet zwei Diagramme. Sie sind nicht bloß Bilder: Ein Klick auf ein Stück des Kuchens oder einen Balken listet darunter die Einträge, aus denen es besteht."),

  H.h3("Papier und andere Programme"),

  H.p("Die Ausleihliste und die Karteikarte sind Berichte: im Studio entworfen wie ein Fenster, gefüllt aus der Datenbank, in einer Vorschau gezeigt und von dort gedruckt oder als PDF gesichert. Der Bestand geht als Excel-Mappe hinaus, und eine alte Liste aus einer Tabellenkalkulation kommt als CSV herein — mit einer Aufstellung der Zeilen, die nicht gepasst haben, statt eines stillen Fehlers."),

  H.h3("Für alle, überall"),

  H.p("Die Mediathek hat ein dunkles und ein helles Aussehen und merkt sich, welches man zuletzt gewählt hat. Sie lässt sich ganz ohne Maus bedienen; ein Bildschirmleser findet zu jedem Feld und Knopf einen Namen und sagt an, was ein Befehl getan hat. Auf einem System, das nicht Deutsch spricht, spricht sie Englisch. Und sie kommt als Installer zu anderen, läuft dort ohne Drachenhauch und legt ihre Daten in den Ordner, den das Betriebssystem für die Daten eines Benutzers vorsieht."),

  H.h3("Was dafür gebaut wird"),

  H.p(`Am Ende besteht das Projekt aus ${zahl(fenster)} Fenstern und ${zahl(berichte)} Berichten, die man im Studio zeichnet, und ${zeilen} Zeilen eigenem Code in ${dh.length} Dateien — Kommentare eingerechnet. Jedes Stück hat sein Kapitel:`),

  H.table([
    ["Ein Fenster, Knöpfe und Felder, die richtig sitzen", "1, 2, 9"],
    ["Eine Tabelle, die sortiert und filtert", "3, 14"],
    ["Daten, die bleiben: eine Datenbank", "4, 13"],
    ["Formulare, die prüfen", "5, 14"],
    ["Menüs, Kürzel und Rückfragen", "6, 21"],
    ["Mehrere Fenster, modal und nicht", "11"],
    ["Fehler finden", "12"],
    ["Abfragen, die zählen und rechnen", "15"],
    ["Daten aus dem Netz", "16"],
    ["Berichte und Drucken", "17"],
    ["Diagramme", "18"],
    ["Excel und CSV", "19"],
    ["Aussehen und eigene Bausteine", "20"],
    ["Tastatur und Bildschirmleser", "21"],
    ["Eine zweite Sprache", "22"],
    ["Ein Installer und der Benutzerordner", "23"],
    ["Die Mediathek im Heimnetz", "24"],
  ], { headers: ["Was eine Anwendung braucht", "Kapitel"], widths: [6826, 2200] }),

  H.h2("IDE und Studio"),

  H.p("Drachenhauch bringt zwei Werkzeuge mit. Beim Start fragt es „Wie möchtest du arbeiten?“ und bietet zwei Karten an: klassisch, die IDE, und modern, das Studio. Beide arbeiten mit derselben Sprache und derselben Laufzeit; was man in dem einen lernt, gilt im anderen."),

  H.figure("kap00_ide.png", "Die IDE mit der ersten Mediathek aus Kapitel 1: links die Dateien des Ordners, in der Mitte der Code, unten Ausgabe und Probleme.", 480, 300),

  H.p("Die IDE ist ein Editor für Programmtext. Man öffnet einen Ordner und darin eine Datei; das Programm steht ganz in seinen Zeilen, auch jedes Fenster und jeder Knopf. Sie ist das Werkzeug für Spiele, für kleine Helfer und für alles, was in eine oder wenige Dateien passt — und das richtige, um zu sehen, was unter einer Oberfläche passiert. Teil I dieses Buchs arbeitet mit ihr."),

  H.figure("kap00_studio.png", "Das Studio mit der fertigen Mediathek: links Navigator und Bibliothek, in der Mitte der Entwurf des Hauptfensters, rechts der Inspektor mit seinen Eigenschaften.", 480, 300),

  H.p("Das Studio ist ein Werkzeug für Anwendungen. Ein Projekt ist ein Ordner mit Fenstern, Berichten, einer Datenbank und Code. Fenster zeichnet man, indem man Bausteine aus der Bibliothek hineinzieht; ihre Eigenschaften stellt man im Inspektor ein; Code schreibt man an das Ereignis, zu dem er gehört — an den Klick eines Knopfs, an das Öffnen eines Fensters. Was dabei entsteht, ist am Ende gewöhnlicher Drachenhauch-Code: Beim Start schreibt das Studio ihn in einen Ordner _studio, und wer neugierig ist, kann dort nachlesen, was es aus dem Entwurf gemacht hat (Kapitel 8)."),

  H.table([
    ["ein Spiel, eine Demo, ein Werkzeug in einer Datei", "IDE"],
    ["ein Fenster mit Zeichenfläche, das man selbst malt", "IDE"],
    ["Formulare, Tabellen, mehrere Fenster", "Studio"],
    ["eine Datenbank, Berichte, Drucken", "Studio"],
    ["verstehen, wie die gui im Inneren arbeitet", "IDE"],
  ], { headers: ["Wer das bauen will …", "… nimmt"], widths: [6826, 2200] }),

  H.p("Die Grenze ist nicht hart. Auch in der IDE lassen sich Anwendungen bauen — Teil I tut genau das —, und auch im Studio lässt sich eine Zeichenfläche bemalen. Aber ab dem zweiten Fenster und der ersten Tabelle in einer Datenbank nimmt das Studio so viel Handarbeit ab, dass man den Weg von Hand nur noch geht, um ihn einmal gesehen zu haben."),

  H.h2("Mit welchem Stand man einsteigt"),

  H.p("Wer das Buch der Reihe nach liest, baut die Mediathek Schritt für Schritt selbst. Wer lieber in der Mitte einsteigt, findet jeden Stand fertig im Ordner buecher/anwendungen/code: Die Programme von Teil I liegen als einzelne Dateien in kap01 bis kap06, die Projekte des Studios in kap08/mediathek bis kap24/mediathek — jeweils so, wie sie am Ende des Kapitels aussehen. Wer mit Kapitel 15 beginnen will, kopiert kap14/mediathek an einen eigenen Ort und öffnet ihn im Studio. Eine Kopie, weil das Studio beim Start in den Ordner schreibt und die Mediathek ihre Datenbank darin ändert; das Original bleibt so, wie das Buch es beschreibt."),
];
