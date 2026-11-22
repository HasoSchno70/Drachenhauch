const { stueck } = require("../code_lesen");

const F = "kap18/mediathek/StatistikFenster.dh";

module.exports = (H) => [
  H.chapter("Diagramme", 18),

  H.h2("Was wir bauen"),

  H.figure("kap18_statistik.png", "Die Statistik als Diagramme: links die Einträge je Art, rechts die Ausleihen der letzten sechs Monate. Ein Klick auf das Stück Buch hat die Liste darunter gefüllt.", 480, 320),

  H.p("Die Statistik aus Kapitel 15 war eine Tabelle: Buch 2, Film 1, Musik 1, Spiel 1. Das stimmt, aber man muss es lesen. In diesem Kapitel wird daraus ein Bild: ein Kuchen mit den Einträgen je Art und Balken mit den Ausleihen der letzten sechs Monate. Wer genau wissen will, was hinter einem Stück steckt, klickt darauf, und die Liste darunter zeigt es."),

  H.h2("Der Plan"),

  H.p("Die Bibliothek des Studios hat unter „Zeichnen“ ein Diagramm. Es ist eine Zeichenfläche, auf der ein Diagramm aus dem Modul chart sich in jedem Bild selbst zeichnet, im Thema der Oberfläche. Welche Art — Kuchen, Donut, Balken, Linie, Fläche, Tacho, Leiste, Lampen — und welchen Titel, stellt der Inspektor ein. Die Werte gibt der Code:"),

  H.bulletRich("Leeren() ", "verwirft die Werte."),
  H.bulletRich("Hinzufuegen(name, wert) ", "gibt einem Kuchen ein Stück oder einem einfachen Balkendiagramm einen Balken."),
  H.bulletRich("Reihe(name, werte) ", "gibt Balken oder Linie eine ganze Reihe von Werten; Beschriften(nr, text) schreibt unter einen Punkt."),
  H.bulletRich("Setzen, SetzenZahl, SetzenSchalter ", "stellen alles Weitere ein: Legende, Zahlen in den Stücken, Sprechblasen."),

  H.p("Und ein Klick auf ein Stück oder einen Balken? Das Diagramm merkt ihn beim Zeichnen; der Code fragt ihn danach ab."),

  H.h2("Schritt 1: das Fenster"),

  H.figure("kap18_entwurf.png", "Das Statistikfenster im Entwurf, der Kuchen gewählt. In der Gruppe Inhalt des Inspektors stehen seine Art und sein Titel; der Entwurf zeigt ein Muster der Art.", 480, 262),

  H.p("Das Statistikfenster wird 700 × 440 groß. Die Tabelle aus Kapitel 15 fliegt hinaus; an ihre Stelle kommen zwei Diagramme nebeneinander, je 320 × 240: diagArt mit der Art kuchen und dem Titel „Einträge je Art“, diagMonate mit balken und „Ausleihen je Monat“. Darunter ein Label hinweis, eine Liste liste und der Knopf zum Schließen."),

  H.h2("Schritt 2: der Kuchen"),

  H.p("Die Zahlen liefert die Abfrage statistik aus Kapitel 15 — Art, Einträge, verliehen. Das Fenster merkt sich die Arten in der Reihenfolge der Stücke; das braucht es später für den Klick:"),

  H.code(stueck(F, "SUB artenFuellen", "END SUB")),

  H.p("Hinzufuegen nimmt einen Namen und einen Wert; ohne Farbe nimmt das Diagramm die nächste der Palette. Die Legende steht rechts, und mit tooltip erscheint über dem Stück unter der Maus eine Sprechblase mit Name, Wert und Anteil."),

  H.h2("Schritt 3: die Balken"),

  H.p("Für die Balken gibt es keine fertige Abfrage, die auch die Monate ohne Ausleihe liefert. Darum zählt das Fenster sechs Monate bis heute ab, fragt je Monat die Datenbank und gibt die sechs Werte als eine Reihe:"),

  H.code(stueck(F, "SUB monateFuellen", "END SUB")),

  H.p("Der Monat wird als Text JJJJ-MM gebaut, denn so liefert ihn auch strftime('%Y-%m', von) in der Abfrage — die kleine Funktion anzahlImMonat im Modul Bestand vergleicht genau diese beiden Texte. Unter die Balken schreibt Beschriften nur den Monat, 09 statt 2026-09."),

  H.h2("Schritt 4: der Klick"),

  H.p("Das Diagramm wertet die Maus beim Zeichnen selbst aus: Ein Stück unter der Maus leuchtet auf und rückt ein wenig heraus. Wird geklickt, liefert CHART_CLICKED die Nummer des Punkts — bis zum nächsten Zeichnen. Abgefragt wird darum in Takt, das in jedem Bild nach den Ereignissen läuft:"),

  H.code(stueck(F, "SUB Takt", "END SUB")),

  H.p("Self.diagArt.Diagramm ist der CHART-Handle des Diagramms, für alles, wofür das Studio keine eigene Methode hat. Mit der Nummer findet das Fenster in seinen Listen, was der Punkt bedeutet, und zeigt es:"),

  H.code(stueck(F, "SUB artZeigen", "END SUB")),

  H.p("anzahlText$ im Modul Bestand schreibt „1 Eintrag“ und „2 Einträge“, je nach Zahl."),

  H.h2("Warum so?"),

  H.h3("Warum der Klick in Takt gefragt wird"),

  H.p("Die Zeichenfläche hat ein Ereignis Maus. Es kommt, wenn die Taste auf der Fläche gedrückt wird — aber bevor in diesem Bild gezeichnet wird. Erst das Zeichnen stellt fest, welches Stück unter der Maus lag; im Ereignis Maus weiß das Diagramm es also noch nicht. Takt läuft im nächsten Bild, und dann steht der Klick da. Das ist die Arbeitsteilung, die auch das Handbuch beschreibt: Ereignisse für das, was die Oberfläche meldet, Takt für alles, was der Code selbst abfragt."),

  H.h3("Warum auch leere Monate"),

  H.p("Eine Abfrage mit GROUP BY wie in Kapitel 15 liefert nur Monate, in denen etwas verliehen wurde. Als Balken sähe das aus, als folgten September und Oktober direkt auf Juni. Ein Diagramm über die Zeit braucht jeden Zeitpunkt, auch den mit null — die Lücke ist eine Auskunft."),

  H.h2("Was schiefgehen kann"),

  H.h3("Neu füllen"),

  H.p("Wer ein Diagramm noch einmal füllt — nach einer neuen Ausleihe, oder in der Übung unten für einen anderen Zeitraum —, ruft erst Leeren. Leeren verwirft die Werte, nicht die Reihen: Reihe mit demselben Namen gibt der alten Reihe neue Werte. Ohne Leeren hängte Hinzufuegen neue Stücke an die alten, und der Kuchen hätte jede Art zweimal."),

  H.h3("Nummern, die nicht passen"),

  H.p("CHART_CLICKED liefert die Nummer eines Punkts, nicht seinen Namen. Die Listen arten und monate müssen darum in genau der Reihenfolge gefüllt werden wie das Diagramm — beides geschieht in derselben Schleife. Wer das eine sortiert und das andere nicht, bekommt beim Klick auf Musik die Spiele."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Lass in monateFuellen die Schleife mit Beschriften weg. Was steht unter den Balken?"),
  H.bulletRich("Drehen 2. ", "Setze im Kuchen Setzen(\"werte\", \"innen\"). Was steht in den Stücken? Und mit SetzenSchalter(\"prozent\", TRUE) dazu?"),
  H.bulletRich("Drehen 3. ", "Frage den Klick auf den Kuchen in einer Methode diagArt_Maus ab statt in Takt. Was passiert beim Klick auf ein Stück?"),
  H.bulletRich("Erweitern 1. ", "Eine Klappliste rechts neben der Liste wählt den Zeitraum der Balken: sechs oder zwölf Monate."),
  H.bulletRich("Erweitern 2. ", "Eine Leiste zeigt, wie viel der Mediathek gerade verliehen ist, in Prozent."),
  H.bulletRich("Erfinden. ", "Welches Diagramm passt zu deiner Sammlung aus Kapitel 13? Bau es, und lass einen Klick zeigen, was dahintersteckt."),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Modul chart: ", "alle Arten, Reihen, Achsen, Farben, Tacho und Leiste, die Maus."),
  H.bulletRich("Handbuch, Drachenhauch Studio → Was ein Programm braucht: ", "das Diagramm als Control und seine Methoden."),
];
