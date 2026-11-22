const fs = require("fs");
const path = require("path");
const { stueck } = require("../code_lesen");

// Das SQL einer Abfrage, wie es im Datenbank-Element steht.
function sql(rel, name) {
  const d = JSON.parse(fs.readFileSync(path.join(__dirname, "..", "..", "code", rel), "utf8"));
  return d.abfragen.find((a) => a.name === name).sql.split("\n");
}

module.exports = (H) => [
  H.part("Teil IV · Fertig machen"),
  H.chapter("Berichte und Drucken", 17),

  H.h2("Was wir bauen"),

  H.figure("kap17_karte.png", "Eine Karteikarte, A6 quer, aus den Daten eines Eintrags: Titel, Urheber, Art, Jahr, ISBN und das Cover aus Kapitel 16.", 480, 340),

  H.p("Bis hierher lebt die Mediathek auf dem Bildschirm. Jetzt kommt sie aufs Papier, auf zwei Arten: eine Ausleihliste, die zeigt, wer was wann hatte, nach Freunden geordnet und mit Summen, und eine Karteikarte je Eintrag für den Zettelkasten neben dem Regal. Beides öffnet das Menü Datei, in einer Vorschau mit Drucken und „Als PDF …“."),

  H.h2("Der Plan"),

  H.p("Ein Bericht ist im Studio ein eigenes Element, wie ein Fenster oder die Datenbank. Er beschreibt eine Seite in Millimetern, und auf ihr Bänder, die untereinander stehen. Wann ein Band erscheint, hängt von seiner Art ab:"),

  H.bulletRich("Berichtskopf und Berichtsfuß ", "einmal, am Anfang und am Ende."),
  H.bulletRich("Seitenkopf und Seitenfuß ", "auf jeder Seite oben und unten."),
  H.bulletRich("Detail ", "einmal je Zeile der Quelle."),
  H.bulletRich("Gruppenkopf und Gruppenfuß ", "vor und nach jeder Gruppe — allen Zeilen mit demselben Wert in einer Spalte."),

  H.p("In den Bändern stehen Elemente: Text, Felder (eine Spalte der Quelle), Summen, die Seitenzahl, das Datum, Linien, Rechtecke und Bilder. Die Quelle ist eine Tabelle oder Abfrage der Datenbank; ihre Bedingung darf Parameter haben, die das Programm beim Drucken mitgibt. Im Code ist ein Bericht ein Objekt unter seinem Namen, mit Vorschau, Drucken, Pdf und Erzeugen."),

  H.h2("Schritt 1: eine Abfrage für die Liste"),

  H.p("Die Ausleihliste braucht je Ausleihe den Freund, den Titel, Beginn, Ende und die Dauer. Das liefert eine neue Abfrage ausleihbuch:"),

  H.code(sql("kap17/mediathek/Mediathek.dhdb", "ausleihbuch"), { out: true }),

  H.p("COALESCE nimmt den ersten Wert, der nicht leer ist. Eine Ausleihe, die noch läuft, hat kein zurueck; dort steht dann „noch da“, und die Tage zählen bis heute. Neu ist hier nur COALESCE; den Rest kennt Kapitel 15."),

  H.h2("Schritt 2: die Ausleihliste"),

  H.figure("kap17_entwurf.png", "Der Entwurf der Ausleihliste: die Bänder untereinander, jedes mit seiner Leiste; Felder in eckigen Klammern, Summen mit Σ. Rechts die Spalten der Quelle und die Eigenschaften des Berichts.", 480, 216),

  H.p("Projekt → Bericht hinzufügen … legt einen Bericht an; er heißt Ausleihliste. Sein Reiter ist der Entwurf: oben die Leiste der Elemente, darunter die Seite, rechts die Spalten der Quelle und die Eigenschaften. Ist nichts gewählt, gehören die Eigenschaften dem Bericht: Titel Ausleihliste, Quelle Mediathek.ausleihbuch, Sortieren von. „Gruppe dazu“ mit der Spalte freund legt Gruppenkopf und Gruppenfuß an. Sortiert wird immer zuerst nach den Gruppen, dann nach „Sortieren“ — jeder Freund steht also einmal da, mit seinen Ausleihen nach Datum."),

  H.p("Die Elemente kommen am schnellsten aus den Spalten der Quelle: Eine Spalte, die man in ein Band zieht, wird dort ein Feld, im Seitenkopf eine Überschrift und im Gruppen- oder Berichtsfuß eine Summe. Im Detail stehen titel, von, zurueck und tage; von und zurueck haben das Format datum und erscheinen als 08.09.2026. Im Gruppenfuß zählt eine Summe mit der Funktion anzahl die Ausleihen, eine zweite addiert die Tage; dieselben beiden stehen im Berichtsfuß noch einmal, mit dem Bereich Bericht. Der Seitenfuß hat eine Seitenzahl, „Seite {n} von {m}“, der Berichtskopf den Titel und das Datum."),

  H.p("Vorschau … oben im Reiter setzt den Bericht mit den Daten der Datei im Projektordner — so sieht man jede Änderung sofort auf dem Papier."),

  H.figure("kap17_liste.png", "Die Ausleihliste, gesetzt: Jonas hatte Momo 15 Tage, Tetris ist seit 40 Tagen bei Lena. Gruppenfüße und Berichtsfuß zählen und addieren.", 480, 256),

  H.h2("Schritt 3: die Karteikarte"),

  H.p("Die Karteikarte ist ein zweiter Bericht, aber ganz anders gebaut. Papier A6, „Hochformat“ aus — das gibt 148 × 105 mm quer —, Ränder 8 mm, Quelle Mediathek.eintrag und die Bedingung id = :id. Der Doppelpunkt macht aus id einen Parameter: Welcher Eintrag gedruckt wird, sagt erst das Programm."),

  H.p("Gebraucht wird nur ein Band, das Detail, so hoch wie die Seite innerhalb der Ränder: 89 mm. Darin ein Rechteck als Rahmen, die Felder titel (14 pt, fett), urheber, art und jahr, darunter das Wort ISBN und das Feld isbn in Schreibmaschinenschrift. Rechts ein Bild-Element mit der Spalte cover: Es zeigt je Zeile das Bild aus dieser Bildspalte, eingepasst, ohne es zu verzerren. Unten eine Linie, „Mediathek“ und das Datum."),

  H.p("Nicht jeder Eintrag hat eine ISBN. Damit das Wort ISBN nicht allein dasteht, bekommt es die Eigenschaft „Nur Zeilen mit“: isbn <> \"\". Ein Element mit dieser Eigenschaft erscheint nur, wenn sein Band eine Zeile hat, auf die sie zutrifft; ein leerer Wert trifft nie."),

  H.h2("Schritt 4: im Programm"),

  H.p("Das Hauptfenster bekommt ein Menü Datei mit den Einträgen „Ausleihliste …“ und „Karteikarte …“:"),

  H.code(stueck("kap17/mediathek/Hauptfenster.dh", "SUB miAusleihliste_Aktion", "END SUB", 2)),

  H.p("Werte für die Parameter gibt man als MAP mit, {} heißt: keine. Vorschau zeigt ein Fenster mit den Seiten, Blättern, Drucken, „Als PDF …“ und Schließen und kehrt zurück, wenn es zu ist. Ohne Vorschau geht es auch: Ausleihliste.Pdf(\"liste.pdf\", {}) sichert gleich eine Datei, Ausleihliste.Drucken({}) schickt sie an den Drucker, und Erzeugen liefert das PDF als Handle für das Modul pdf."),

  H.h2("Warum so?"),

  H.h3("Warum ein Bericht kein Fenster ist, das man druckt"),

  H.p("Die Tabelle des Hauptfensters zeigt schon fast alles, was in der Ausleihliste steht. Warum sie nicht einfach drucken? Weil ein Fenster für den Bildschirm gebaut ist: Es ist so groß wie der Bildschirm, rollt, wenn die Zeilen nicht passen, und misst in Bildpunkten. Papier hat feste Seiten. Eine Liste mit 200 Ausleihen muss umbrechen, oben auf jeder Seite die Überschriften wiederholen, unten die Seitenzahl tragen und am Ende einer Gruppe zusammenzählen. Und sie muss in Millimetern stimmen, damit die Karteikarte auf eine Karteikarte passt."),

  H.p("Ein Bericht trennt darum, was gedruckt wird — die Zeilen der Quelle —, von wie es aussieht — Bänder und Elemente. Dieselbe Karteikarte gibt eine Seite für einen Eintrag oder hundert für hundert. Und weil der Bericht ein PDF erzeugt, ist er auf jedem Drucker gleich scharf, und man kann ihn als Datei weitergeben."),

  H.h2("Was schiefgehen kann"),

  H.h3("Ein Parameter fehlt"),

  H.p("Wer Karteikarte.Vorschau({}) schreibt, vergisst id. Der Bericht setzt dann nicht irgendetwas, sondern hält an: „Bericht: Parameter :id fehlt“. Eine id, zu der es keinen Eintrag gibt, ist dagegen kein Fehler — die Bedingung trifft keine Zeile, und es kommt eine leere Seite heraus."),

  H.h3("Ein Text, der nicht passt"),

  H.p("Ein Feld bricht nicht um. Was nicht in seine Breite passt, wird gekürzt und endet mit „…“. Auf der Karteikarte ist das der Titel: Mit 16 statt 14 pt passt „Die unendliche Geschichte“ nicht mehr hinein. Die Vorschau zeigt es; ein breiteres Feld oder eine kleinere Schrift hilft."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Stelle den Titel der Karteikarte auf 16 pt. Was steht auf der Karte von „Die unendliche Geschichte“?"),
  H.bulletRich("Drehen 2. ", "Stelle in der Ausleihliste die Summe der Tage im Gruppenfuß auf den Bereich Bericht. Was zeigen die Gruppenfüße?"),
  H.bulletRich("Drehen 3. ", "Leere die Bedingung der Karteikarte und rufe sie wie bisher mit {\"id\": id}. Wie viele Seiten hat die Vorschau?"),
  H.bulletRich("Erweitern 1. ", "Auf die Karteikarte kommt der Strichcode der ISBN — die Leiste der Elemente hat dafür „Strichcode“."),
  H.bulletRich("Erweitern 2. ", "Ein Jahresrückblick: alle Ausleihen des laufenden Jahres, nach Monaten gruppiert, mit einer Zahl je Monat und einer für das Jahr."),
  H.bulletRich("Erfinden. ", "Entwirf für deine Sammlung aus Kapitel 13 einen Bericht, den du wirklich ausdrucken würdest — eine Inventarliste, Etiketten, eine Seite je Stück."),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Drachenhauch Studio → Berichte: ", "alle Bänder, Elemente und Eigenschaften, der Entwurf mit Raster und Ausrichtungslinien, „Nur Zeilen mit“, Bild und Strichcode."),
  H.bulletRich("Handbuch, Modul pdf: ", "ein PDF von Hand setzen, PDF_PRINT, die Schriften."),
];
