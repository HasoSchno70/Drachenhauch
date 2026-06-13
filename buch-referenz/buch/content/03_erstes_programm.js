module.exports = (H) => [
  H.chapter("Dein erstes Programm"),
  H.p("Tradition ist Tradition: Das allererste Programm in jeder Sprache begrüßt die Welt. Tippe das hier ab und starte es:"),
  H.code(['PRINT "Hallo, Welt!"']),
  H.p("Drück F5 (oder starte die Datei wie im letzten Kapitel beschrieben). In der Konsole erscheint:"),
  H.code(["Hallo, Welt!"], { out: true }),
  H.p("Glückwunsch – du hast programmiert. Sehen wir uns an, was hier passiert ist. PRINT ist ein Befehl, der etwas auf den Bildschirm schreibt. Was es schreiben soll, steht dahinter. Der Text in doppelten Anführungszeichen heißt eine Zeichenkette (englisch string) – also einfach „Text“. Die Anführungszeichen gehören zur Sprache, nicht zur Ausgabe: Sie markieren, wo der Text anfängt und aufhört, und werden selbst nicht mitgedruckt."),

  H.h2("Mehrere Zeilen"),
  H.p("Jeder Befehl steht in seiner eigenen Zeile. Mehrere PRINTs untereinander geben mehrere Zeilen aus:"),
  H.code([
    'PRINT "Zeile eins"',
    'PRINT "Zeile zwei"',
    'PRINT "und drei"',
  ]),
  H.code(["Zeile eins", "Zeile zwei", "und drei"], { out: true }),

  H.h2("Rechnen mit PRINT"),
  H.p("PRINT kann nicht nur Text, sondern auch Zahlen und Rechnungen ausgeben. Steht hinter PRINT kein Text in Anführungszeichen, sondern eine Rechnung, dann rechnet GameBasic sie zuerst aus und druckt das Ergebnis:"),
  H.code([
    'PRINT 3 + 4',
    'PRINT 10 * 10',
    'PRINT 100 / 8',
  ]),
  H.code(["7", "100", "12.5"], { out: true }),
  H.pmix(["Beachte: ", ["100 / 8", true], " ergibt ", ["12.5", true], " – GameBasic rechnet bei Bedarf mit Kommazahlen weiter und rundet nicht einfach ab. Mehr dazu im Kapitel über Zahlen."]),

  H.h2("Text und Zahl zusammen"),
  H.p("Oft willst du Text und einen Wert in einer Zeile mischen. Mit einem Komma trennst du mehrere Dinge, die PRINT nacheinander ausgibt – mit einem Leerzeichen dazwischen:"),
  H.code(['PRINT "Drei mal vier ist", 3 * 4']),
  H.code(["Drei mal vier ist 12"], { out: true }),

  H.h2("Kommentare"),
  H.p("Mit einem Hochkomma (') schreibst du eine Notiz für dich selbst (oder den, der deinen Code später liest). Alles ab dem ' bis zum Zeilenende ignoriert GameBasic vollständig. Solche Kommentare erklären, was der Code tut, und richten nie Schaden an:"),
  H.code([
    "' Dieses Programm begruesst die Welt",
    'PRINT "Hallo!"        \' das hier wird ausgegeben',
    "' PRINT \"ich nicht\"   <- diese Zeile ist auskommentiert",
  ]),
  H.code(["Hallo!"], { out: true }),
  H.note("Kommentare sind keine Spielerei für Fortgeschrittene. Schon in kleinen Programmen helfen sie, den Überblick zu behalten – und in drei Wochen weißt du selbst nicht mehr, was du dir bei Zeile 40 gedacht hast."),

  H.h2("Ein erster Blick auf Grafik"),
  H.p("Damit du siehst, wohin die Reise geht: Dasselbe „Hallo“ als Grafik-Programm. Es öffnet ein Fenster und malt den Text hinein. Die Befehle darin lernst du in Teil IV im Detail – hier nur als Ausblick:"),
  H.code([
    'SCREEN(480, 320, "Mein erstes Fenster")',
    'CLS(RGB(20, 24, 40))                  \' Hintergrund dunkelblau',
    'TEXT(120, 140, "Hallo Welt!", RGB(255, 220, 0))',
    'FLIP()                                \' Gezeichnetes anzeigen',
    'SLEEP(3000)                           \' 3 Sekunden stehen lassen',
  ]),
  H.p("Mehr Befehle, mehr Möglichkeiten – aber dieselbe freundliche Klarheit. Zuerst aber bleiben wir in der Konsole und lernen die Sprache richtig kennen. Auf zum nächsten Kapitel: Variablen."),
];
