// Teil I -- Vorwort + Willkommen
module.exports = (H) => [
  H.h1("Vorwort"),
  H.p("Es gibt zwei Sorten Mensch: die einen benutzen Programme, die anderen wollen wissen, wie zum Kuckuck die eigentlich funktionieren – und sie am liebsten gleich selbst bauen. Wenn du dieses Buch in der Hand hältst, gehörst du vermutlich zur zweiten Sorte. Herzlich willkommen, hier bist du genau richtig."),
  H.p("Ich heiße Hans Schnorrenberger und habe GameBasic gebaut, weil ich es leid war, Anfängern erklären zu müssen, warum man für ein einziges bewegtes Pixel erst drei Bibliotheken installieren, ein Dutzend Fehlermeldungen entschlüsseln und „aus Gründen“ irgendwo ein Semikolon setzen muss. Programmieren sollte sich nicht wie eine Mutprobe anfühlen, sondern wie das, was es ist: das Bauen kleiner Welten, in denen du die Regeln machst."),
  H.p("Dieses Buch ist beides zugleich: ein Lehrbuch, das dich vom ersten schwarzen Fenster bis zu Klassen, Modulen und Spielen an die Hand nimmt – und ein Nachschlagewerk, in dem jeder Befehl einzeln erklärt wird, mit einem kleinen Programm, das ihn zeigt. Du kannst es also von vorne nach hinten durchlesen oder gezielt nachschlagen, wenn du wissen willst, was ein bestimmter Befehl tut."),
  H.p("Ein Rat noch, bevor es losgeht: Tippe die Beispiele wirklich selbst ab und starte sie. Programmieren lernt man in den Fingern, nicht in den Augen – ungefähr so, wie man Fahrradfahren nicht aus einem Buch über Fahrräder lernt. Mach Fehler, viele sogar. Jeder Fehler, den du selbst gefunden hast, ist eine Lektion, die sitzt."),
  new (require("docx").Paragraph)({ spacing: { before: 240 },
    children: [new (require("docx").TextRun)({ text: "— Hans Schnorrenberger", italics: true, size: 22, color: "6A6A6A" })] }),

  H.h1("Wie du dieses Buch liest"),
  H.p("Das Buch ist in fünf Teile gegliedert. Teil I bringt dich zum Laufen: Was ist GameBasic, wie startest du ein Programm, und wie sieht das allererste aus. Teil II erklärt die Sprache selbst – Variablen, Schleifen, Funktionen, Klassen. Teil III ist die Referenz der eingebauten Befehle, nach Themen sortiert. Teil IV widmet sich Grafik, Sound und Spielen, Teil V den Modulen für Spezialaufgaben. Den Abschluss bildet ein Anhang mit Nachschlage-Tabellen: ein alphabetischer Index aller Befehle, sämtliche Tastencodes und Farb-Konstanten sowie eine Erklärung der häufigsten Fehlermeldungen."),
  H.bulletRich("Normaler Text ", "erklärt, was passiert und warum."),
  H.bulletRich("Code sieht so aus: ", "in einem grauen Kasten mit blauer Leiste, in Schreibmaschinenschrift. Diesen Teil tippst du ab."),
  H.bulletRich("Programm-Ausgabe ", "steht im grünen Kasten – das, was dein Programm auf den Bildschirm schreibt."),
  H.p("So sieht ein Beispiel mit seiner Ausgabe aus:"),
  H.code(['PRINT "Hallo, Welt!"', 'PRINT 3 + 4']),
  H.code(["Hallo, Welt!", "7"], { out: true }),
  H.tip("Selber tippen!", "Kopieren wäre schneller, aber abtippen bleibt im Kopf. Und wenn etwas schiefgeht: GameBasic sagt dir in klaren, ganzen Sätzen, was es nicht verstanden hat – meistens hast du nur einen Buchstaben vertippt."),
];
