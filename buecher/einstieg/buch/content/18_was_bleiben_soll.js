module.exports = (H) => [
  H.chapter("Was bleiben soll"),

  H.p("Alle Programme dieses Buchs haben eine Eigenschaft gemeinsam, die dir vielleicht nicht aufgefallen ist: Sie vergessen alles, sobald man sie beendet. Die Schlange kennt ihre beste Länge nicht, das Instrument merkt sich nicht, welche Klangform du zuletzt hattest, und die Bestenliste würfelt bei jedem Start von vorn."),

  H.p("Für ein Spiel ist das in Ordnung. Für alles andere nicht — und der Vokabeltrainer am Ende dieses Buchs wäre ohne Gedächtnis eine sinnlose Übung."),

  H.h2("Eine Datei schreiben"),

  H.code([
    "DIM aus AS FILE",
    "",
    'aus = OPENFILE("punkte.txt", "w")',
    'WRITELINE(aus, "Anna;95")',
    'WRITELINE(aus, "Bert;78")',
    "CLOSEFILE(aus)",
  ]),

  H.pmix([["DIM aus AS FILE", true], " ist wieder eine neue Sorte Karton — darin wohnt eine geöffnete Datei."]),

  H.pmix([["OPENFILE", true], " öffnet sie. Das zweite Argument sagt wofür: ", ['"w"', true], " für schreiben (die Datei wird dabei geleert), ", ['"r"', true], " für lesen, ", ['"a"', true], " für anhängen."]),

  H.pmix([["WRITELINE", true], " schreibt eine Zeile und hängt den Zeilenumbruch selbst an. ", ["CLOSEFILE", true], " schließt die Datei wieder."]),

  H.warn("CLOSEFILE nicht vergessen. Solange die Datei offen ist, kann ein Teil des Geschriebenen noch im Zwischenspeicher hängen und nicht wirklich auf der Platte stehen. Wer sein Programm hart abbricht, bevor geschlossen wurde, findet die Datei unter Umständen leer vor.", "Erst schließen, dann verlassen"),

  H.h2("Eine Datei lesen — auf die einfache Art"),

  H.code([
    "DIM zeilen AS ARRAY OF STRING",
    "",
    'zeilen = READLINES("punkte.txt")',
    "PRINT LEN(zeilen)",
  ]),

  H.code(["2"], { out: true }),

  H.pmix([["READLINES", true], " liest die ganze Datei und gibt sie als Array aus Zeilen zurück. Eine Zeile Code für eine ganze Datei — für alles, was in den Speicher passt, ist das der bequemste Weg, und die Dateien in diesem Buch sind winzig."]),

  H.p("Es geht auch Zeile für Zeile, und das braucht man bei sehr großen Dateien:"),

  H.code([
    "DIM ein AS FILE",
    "",
    'ein = OPENFILE("punkte.txt", "r")',
    "WHILE NOT ENDOFFILE(ein)",
    "    PRINT READLINE(ein)",
    "WEND",
    "CLOSEFILE(ein)",
  ]),

  H.h2("Ein Wert, der den Neustart überlebt"),

  H.p("Damit lässt sich endlich der Bestwert bauen, der bleibt. Das Muster dafür hat zwei Hälften. Beim Start nachsehen:"),

  H.code([
    'IF FILEEXISTS("bestwert.txt") THEN',
    '    zeilen = READLINES("bestwert.txt")',
    "    IF LEN(zeilen) > 0 THEN best = VAL(zeilen[0])",
    "END IF",
  ]),

  H.p("Und immer dann schreiben, wenn sich etwas geändert hat:"),

  H.code([
    "IF wurf > best THEN",
    "    best = wurf",
    '    aus = OPENFILE("bestwert.txt", "w")',
    "    WRITELINE(aus, STR$(best))",
    "    CLOSEFILE(aus)",
    "END IF",
  ]),

  H.figure("kap18_1_bestwert.png", "So sieht es direkt nach einem Neustart aus: Der Wurf ist wieder null, der Bestwert nicht.", 440, 280),

  H.pmix(["Die beiden Prüfungen am Anfang sind kein Übereifer. ", ["FILEEXISTS", true], " fragt, ob es die Datei überhaupt schon gibt — beim allerersten Start gibt es sie ja nicht. Und ", ["LEN(zeilen) > 0", true], " fängt den Fall ab, dass sie zwar existiert, aber leer ist. Beides passiert wirklich, und beides würde sonst das Programm beenden, bevor es angefangen hat."]),

  H.pmix([["VAL(zeilen[0])", true], " macht aus dem gelesenen Text wieder eine Zahl — in einer Datei steht immer Text, auch wenn eine Zahl gemeint ist."]),

  H.tip("Nachgemessen", "Das Muster wurde in drei getrennten Programmläufen geprüft, wobei jeder 137 dazuzählt: gelesen 0, geschrieben 137. Gelesen 137, geschrieben 274. Gelesen 274, geschrieben 411. In der Datei stand danach 411. Das Gedächtnis hält."),

  H.h2("Wo landet die Datei eigentlich?"),

  H.warn("Ein Dateiname ohne Pfad bezieht sich auf das Verzeichnis, in dem das PROGRAMM liegt — nicht auf das, aus dem du es gestartet hast. Steht dein Programm in Dokumente/Drachenhauch/kap18/, landet bestwert.txt genau dort. Beim Schreiben dieses Kapitels habe ich die Datei versehentlich woanders abgelegt und mich gewundert, warum das Programm sie nicht findet. Gemessen und bestätigt: Sie erscheint immer neben dem Programm.", "Neben dem Programm, nicht im Startverzeichnis"),

  H.h2("Mehrere Angaben in einer Zeile"),

  H.p("Eine Datei kennt nur Zeilen. Wenn eine Zeile mehrere Angaben tragen soll, trennt man sie durch ein Zeichen, das im Inhalt nicht vorkommt — ein Semikolon zum Beispiel:"),

  H.code([
    "Haus;house",
    "Baum;tree",
    "Katze;cat",
  ], { out: true }),

  H.p("Beim Lesen wird jede Zeile wieder zerlegt, mit dem SPLIT$ aus dem letzten Kapitel:"),

  H.code([
    "FOR i = 0 TO LEN(zeilen) - 1",
    '    teile = SPLIT$(zeilen[i], ";")',
    "    y = 80 + i * 50",
    "    TEXT(60, y, teile[0], schrift)",
    "    TEXT(300, y, teile[1], zweit)",
    "NEXT",
  ]),

  H.figure("kap18_2_vokabeln.png", "Fünf Zeilen aus einer Datei, jede in zwei Teile zerlegt. Das ist der Anfang des Vokabeltrainers.", 440, 280),

  H.p("Dieses Programm legt seine Datei beim ersten Start selbst an, wenn es sie nicht findet. Das ist eine ausgesprochen freundliche Gewohnheit: Der Benutzer muss nichts vorbereiten, und trotzdem kann er die Datei danach mit einem Texteditor öffnen und bearbeiten."),

  H.code([
    'IF NOT FILEEXISTS("vokabeln.txt") THEN',
    '    aus = OPENFILE("vokabeln.txt", "w")',
    '    WRITELINE(aus, "Haus;house")',
    '    WRITELINE(aus, "Baum;tree")',
    "    CLOSEFILE(aus)",
    "END IF",
  ]),

  H.note("Das Semikolon ist willkürlich gewählt. Man nimmt oft auch ein Komma oder einen Tabulator. Wichtig ist nur, dass das Trennzeichen im Inhalt selbst nicht vorkommt — eine Vokabel mit Semikolon darin würde die Zerlegung zerreißen. Für ernsthafte Fälle gibt es dafür fertige Formate; für unsere reicht ein Zeichen, das nicht vorkommt."),

  H.h2("Die Befehle im Überblick"),

  H.table([
    [{ text: 'OPENFILE(pfad, "w")', mono: true }, "zum Schreiben öffnen — leert die Datei"],
    [{ text: 'OPENFILE(pfad, "r")', mono: true }, "zum Lesen öffnen"],
    [{ text: 'OPENFILE(pfad, "a")', mono: true }, "zum Anhängen öffnen — Vorhandenes bleibt"],
    [{ text: "WRITELINE(f, text)", mono: true }, "eine Zeile schreiben"],
    [{ text: "READLINE(f)", mono: true }, "die nächste Zeile lesen"],
    [{ text: "ENDOFFILE(f)", mono: true }, "ist die Datei zu Ende?"],
    [{ text: "CLOSEFILE(f)", mono: true }, "schließen"],
    [{ text: "READLINES(pfad)", mono: true }, "die ganze Datei als Array aus Zeilen"],
    [{ text: "FILEEXISTS(pfad)", mono: true }, "gibt es die Datei?"],
    [{ text: "FILESIZE(pfad)", mono: true }, "wie groß ist sie in Zeichen"],
    [{ text: "DELETEFILE(pfad)", mono: true }, "löschen"],
  ], { headers: ["Aufruf", "Was er tut"], widths: [2800, 6226], mono: [0] }),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    ["Die Datei ist leer", "CLOSEFILE fehlt, oder die Datei wurde mit \"w\" geöffnet und danach nichts geschrieben."],
    ["Das Programm bricht beim Start ab", "Es liest eine Datei, die es noch nicht gibt. FILEEXISTS davor."],
    ["Die Datei ist nirgends zu finden", "Sie liegt neben dem Programm, nicht dort, wo du gestartet hast."],
    ["Beim zweiten Start ist alles weg", "Mit \"w\" geöffnet statt mit \"a\" — \"w\" leert die Datei."],
    [{ text: "Index ausserhalb", mono: true }, "Eine Zeile hatte nicht so viele Teile wie erwartet. Eine leere Zeile am Ende ist der häufigste Grund."],
    ["Aus 95 wird 0", "VAL vergessen, oder der Text enthält Leerzeichen. TRIM$ hilft."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3400, 5626] }),

  H.h2("Aufgaben"),

  H.bullet("Gib Snake einen Bestwert, der bleibt. Die längste je erreichte Schlange gehört in eine Datei."),
  H.bullet("Lass das Instrument aus Kapitel 12 seine zuletzt gewählte Klangform speichern und beim Start wieder einstellen."),
  H.bullet("Erweitere die Vokabelliste um eine dritte Spalte: wie oft die Vokabel schon richtig war. Beim Anzeigen soll sie mit erscheinen."),
  H.bullet("Schreib ein Programm, das eine Datei mit Zahlen liest und Summe, Mittelwert und Höchstwert anzeigt. Die Array-Helfer aus Kapitel 14 erledigen den Rest."),
  H.bullet("Bau ein Protokoll: Bei jedem Start hängt das Programm eine Zeile an eine Datei an. Nimm dafür den Modus zum Anhängen."),
  H.bullet("Lass die Bestenliste aus Kapitel 14 ihre zwölf Werte speichern und beim nächsten Start wieder anzeigen."),

  H.p("Damit endet Teil III. Deine Programme können jetzt Aufgaben bündeln, viele Dinge halten, nach Namen nachschlagen, Dinge mit Eigenschaften bauen, Text zerlegen und sich etwas merken. Was noch fehlt, damit daraus eine Anwendung wird, sind Knöpfe und Eingabefelder — und eigene Bilder."),
];
