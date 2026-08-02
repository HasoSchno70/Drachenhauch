module.exports = (H) => [
  H.part("Teil III — Eingebaute Befehle"),
  H.chapter("Konsole & Ein-/Ausgabe"),
  H.p("Mit Teil III beginnt der Nachschlage-Teil des Buches. Die folgenden Kapitel stellen die eingebauten Befehle von GameBasic geordnet nach Themen vor – jeweils mit Signatur, kurzer Erklärung und einem kleinen Beispiel. Du musst sie nicht am Stück lesen; nutze sie als Lexikon, wenn du einen bestimmten Befehl suchst."),
  H.p("Dieses erste Referenzkapitel sammelt die Befehle für die Textkonsole: ausgeben mit PRINT, einlesen mit INPUT, formatieren mit FORMAT$. Die Konsole ist die einfache textbasierte Ein- und Ausgabe – ganz ohne Grafikfenster. (Tastatur und Maus im Spielfenster behandelt Teil IV.)"),

  H.h2("Ausgabe"),
  H.cmd("PRINT", "PRINT [wert1] [, oder ; wert2 ...] [;]",
    ["Gibt die Werte aus und beginnt eine neue Zeile. Mehrere Werte trennst du mit Komma (Leerzeichen dazwischen) oder Semikolon (kein Zwischenraum). Ein Semikolon ganz am Ende unterdrückt den Zeilenumbruch, sodass der nächste PRINT direkt anschließt. PRINT ohne Argument gibt eine Leerzeile aus.",
     "Ausführlich erklärt im Kapitel „Ein- und Ausgabe“ (Teil II)."],
    [
      'PRINT "a", "b"       \' Komma: Leerzeichen',
      'PRINT "a"; "b"       \' Semikolon: kein Zwischenraum',
      'PRINT "Punkte: "; 42',
    ],
    { out: ["a b", "ab", "Punkte: 42"] }),

  H.h2("Eingabe"),
  H.cmd("INPUT", 'INPUT variable\nINPUT "Frage", variable',
    "Liest eine Zeile von der Tastatur in die Variable. Die Eingabe wird in den Typ der Variablen umgewandelt (Zahl, Text, Wahrheitswert). Optional steht ein Fragetext davor. Die Variable muss vorher mit DIM angelegt sein.",
    [
      'DIM name AS STRING',
      'INPUT "Wie heißt du? ", name',
      'PRINT "Hallo, " + name',
    ]),

  H.h2("Formatieren"),
  H.cmd("FORMAT$", 'FORMAT$(wert, format$)',
    "Formatiert einen Wert nach einer Format-Angabe im C-Stil und liefert einen String. Der Platzhalter beginnt mit %. Die wichtigsten: %d (ganze Zahl), %.Nf (Kommazahl mit N Nachkommastellen), %0Nd (Zahl mit führenden Nullen auf N Stellen), %x (hexadezimal), %s (String). Text um den Platzhalter herum bleibt erhalten.",
    [
      'PRINT FORMAT$(3.14159, "%.2f")',
      'PRINT FORMAT$(42, "%05d")',
      'PRINT FORMAT$(255, "%x")',
      'PRINT FORMAT$(7, "Level %03d")',
    ],
    { out: ["3.14", "00042", "ff", "Level 007"] }),
  H.note("Innerhalb eines f-Strings kannst du dieselben Format-Angaben direkt im Platzhalter nutzen: f\"FPS {wert:.1f}\". Das ist meist die bequemste Art zu formatieren – siehe Kapitel „Ein- und Ausgabe“."),

  H.h2("Hinweis zur Konsole und zum Spielfenster"),
  H.p("PRINT und INPUT arbeiten mit der Textkonsole. Sobald du ein Grafikfenster öffnest (mit SCREEN, siehe Teil IV), zeichnest du Text stattdessen mit TEXT an eine Bildposition, und Tastendrücke fragst du mit Befehlen wie KEYPRESSED oder dem Eingabe-Modul ab – nicht mit INPUT. Für Konsolen-Programme (reine Logik, kleine Helfer, Lernbeispiele) bleibt dieses Kapitel dein Handwerkszeug."),
  H.h2("Auf Tastendruck warten"),
  H.cmd("WAITKEY · INKEY$", 'WAITKEY()   INKEY$()',
    "WAITKEY hält das Programm an, bis eine Taste gedrückt wird – der klassische Abschluss („Weiter mit beliebiger Taste“). INKEY$ liefert das zuletzt getippte Zeichen ODER einen leeren Text, wenn gerade nichts anliegt; es wartet also NICHT.",
    [
      'PRINT "Weiter mit einer Taste ..."',
      'WAITKEY()',
      'PRINT "Es geht weiter!"',
    ]),
  H.tip("Wo geht es weiter?", "Zahl- und Textumwandlung für die Ausgabe (STR$, VAL, CHR$ …) findest du im Kapitel „Typumwandlung & Prüfung“ und bei den „Zeichenketten-Funktionen“. Grafiktext, Tastatur und Maus kommen in Teil IV („Grafik, Sound & Spiele“)."),
];
