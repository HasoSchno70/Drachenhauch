module.exports = (H) => [
  H.part("Teil II — Klang"),
  H.chapter("Der erste Ton"),

  H.p("Neun Kapitel lang war dein Rechner stumm. Das ändert sich mit vier Zeilen — und es ist erstaunlich, wie viel lebendiger ein Programm wird, sobald es sich bemerkbar macht."),

  H.h2("Vier Zeilen, ein Ton"),

  H.code([
    "DIM piep AS SOUND",
    "piep = AUDIO_TONE(440, 400)",
    "PLAYSOUND(piep)",
    "SLEEP(600)",
  ]),

  H.p("Kein Fenster, kein SCREEN, kein CLS. Klang braucht nichts davon. Starte es, und du hörst einen kurzen, klaren Ton."),

  H.pmix([["DIM piep AS SOUND", true], " ist eine neue Sorte Karton. Bisher gab es INTEGER, FLOAT, STRING und BOOLEAN — in einen ", ["SOUND", true], " passt ein fertiger Klang."]),

  H.pmix([["AUDIO_TONE(440, 400)", true], " baut diesen Klang. Die erste Zahl ist die Tonhöhe in Hertz, also Schwingungen je Sekunde; die zweite die Dauer in Millisekunden. 440 Hertz ist das Kammerton-A, nach dem Orchester stimmen. Vierhundert Millisekunden sind vier Zehntelsekunden."]),

  H.pmix([["PLAYSOUND(piep)", true], " spielt ihn ab. Und zwar nebenher: Der Befehl wartet nicht, bis der Ton zu Ende ist, sondern gibt sofort zurück. Das ist genau richtig — ein Spiel soll ja nicht stehenbleiben, weil es piepst."]),

  H.pmix([["SLEEP(600)", true], " ist deshalb nötig. Ohne diese Zeile wäre das Programm nach dem dritten Befehl fertig, würde sich beenden — und der Ton bräche mittendrin ab. Es sind absichtlich 600 statt 400: ein bisschen Luft am Ende."]),

  H.note("Beim Klang gibt es in diesem Buch keine Bilder zu sehen, und das ist ein echter Verlust — man kann Töne nun einmal nicht abdrucken. Bei diesem Teil musst du die Programme wirklich starten. Dafür sind sie kurz."),

  H.tip("Zwei Zahlen zum Drehen", "Setz die 440 auf 220 und auf 880. Du hörst denselben Ton eine Oktave tiefer und eine Oktave höher — Verdoppeln der Frequenz ist genau eine Oktave, und das ist keine Konvention, sondern Physik. Setz die Dauer auf 30 und auf 2000 (und das SLEEP entsprechend mit)."),

  H.h2("Wie ein Ton klingt"),

  H.pmix(["Ein Ton hat außer Höhe und Dauer noch eine Form. ", ["AUDIO_TONE", true], " nimmt sie als drittes Argument entgegen:"]),

  H.code([
    "DIM t AS SOUND",
    "",
    't = AUDIO_TONE(440, 400, "sine")',
    "PLAYSOUND(t)",
    "SLEEP(600)",
    "",
    't = AUDIO_TONE(440, 400, "square")',
    "PLAYSOUND(t)",
    "SLEEP(600)",
    "",
    't = AUDIO_TONE(440, 400, "saw")',
    "PLAYSOUND(t)",
    "SLEEP(600)",
  ]),

  H.table([
    [{ text: '"sine"', mono: true }, "weich und rund — eine Stimmgabel, eine Flöte", "die Vorgabe"],
    [{ text: '"square"', mono: true }, "hart und hohl — der Klang alter Spielkonsolen", "Laser, Piepser"],
    [{ text: '"saw"', mono: true }, "scharf und schnarrend — Streicher, Synthesizer", "Motoren, Alarm"],
    [{ text: '"triangle"', mono: true }, "zwischen sine und square, etwas kantig", "Bass, sanfte Melodien"],
  ], { headers: ["Form", "Wie es klingt", "Wofür"], widths: [1800, 4400, 2826] }),

  H.p("Dieselbe Tonhöhe, dieselbe Dauer, und trotzdem drei völlig verschiedene Eindrücke. Genau daran erkennt man Instrumente auseinander — eine Geige und eine Flöte auf demselben Ton unterscheiden sich nicht in der Höhe, sondern in der Form der Schwingung."),

  H.pmix(["Ein viertes Argument regelt die Lautstärke, von 0 bis 1: ", ['AUDIO_TONE(440, 400, "square", 0.3)', true], " ist derselbe Ton, nur leiser."]),

  H.h2("Eine Tonleiter aus acht Zahlen"),

  H.p("Töne sind Zahlen, und Zahlen kann man in ein Array legen. Damit wird aus einer Schleife eine Tonleiter:"),

  H.code([
    "DIM noten[8] AS INTEGER",
    "DIM i AS INTEGER",
    "DIM ton AS SOUND",
    "",
    "noten[0] = 262      ' c",
    "noten[1] = 294      ' d",
    "noten[2] = 330      ' e",
    "noten[3] = 349      ' f",
    "noten[4] = 392      ' g",
    "noten[5] = 440      ' a",
    "noten[6] = 494      ' h",
    "noten[7] = 523      ' c'",
    "",
    "FOR i = 0 TO 7",
    "    ton = AUDIO_TONE(noten[i], 250)",
    "    PLAYSOUND(ton)",
    "    SLEEP(300)",
    "NEXT",
  ]),

  H.p("Acht Töne, zweieinhalb Sekunden, und du hörst eine C-Dur-Tonleiter. Nachgemessen läuft das Programm 2496 Millisekunden — achtmal 300, plus der Start."),

  H.pmix(["Die Töne dauern 250 Millisekunden, gewartet wird aber 300. Diese 50 Millisekunden Stille dazwischen sind der Unterschied zwischen einzelnen Tönen und einem verschmierten Klangbrei. Nimm sie testweise weg — setz das ", ["SLEEP", true], " auf 250 — und hör dir an, wie die Töne ineinanderlaufen."]),

  H.pmix(["Die letzte Note heißt ", ["c'", true], " und ist genau doppelt so hoch wie die erste: 523 statt 262. Das ist die Oktave von eben. Alle acht Zahlen stehen als Kommentar dahinter, damit man beim Lesen weiß, was gemeint ist — genau dafür sind Kommentare da."]),

  H.warn("Diese Zahlen sind gerundet. Das c liegt genau bei 261,63 Hertz, das e bei 329,63. Für unsere Zwecke reicht die ganze Zahl; ein geübtes Ohr hört den Unterschied nicht. Wer es genau haben will, darf auch Kommazahlen einsetzen — AUDIO_TONE nimmt sie an.", "Krumme Zahlen"),

  H.h2("Eine Melodie"),

  H.p("Mit einem zweiten Array für die Längen wird daraus ein Lied. Hier die ersten beiden Zeilen von „Alle meine Entchen“:"),

  H.code([
    "DIM noten[11] AS INTEGER",
    "DIM dauer[11] AS INTEGER",
    "DIM i AS INTEGER",
    "DIM ton AS SOUND",
    "",
    "noten[0] = 262",
    "noten[1] = 294",
    "noten[2] = 330",
    "noten[3] = 349",
    "noten[4] = 392",
    "noten[5] = 392",
    "noten[6] = 440",
    "noten[7] = 440",
    "noten[8] = 440",
    "noten[9] = 440",
    "noten[10] = 392",
    "",
    "FOR i = 0 TO 9",
    "    dauer[i] = 300",
    "NEXT",
    "dauer[5] = 600",
    "dauer[10] = 600",
    "",
    "FOR i = 0 TO 10",
    "    ton = AUDIO_TONE(noten[i], dauer[i] - 40)",
    "    PLAYSOUND(ton)",
    "    SLEEP(dauer[i])",
    "NEXT",
  ]),

  H.p("Vier Sekunden lang, gemessen 4002 Millisekunden. Und es ist wirklich zu erkennen."),

  H.p("Der Kniff mit den Längen lohnt einen zweiten Blick. Erst bekommen alle elf Noten die Länge 300 — das erledigt eine kurze Schleife. Danach werden nur die beiden Ausnahmen überschrieben: die fünfte und die elfte Note dauern doppelt so lang, weil dort im Lied jeweils eine Zeile endet."),

  H.p("Das ist wieder das Muster aus Kapitel 7: erst den Normalfall setzen, dann die Ausnahmen darüberschreiben. Elf Zeilen wären es sonst, statt drei."),

  H.pmix(["Und noch ein Detail: ", ["dauer[i] - 40", true], " macht jeden Ton vierzig Millisekunden kürzer als seine Pause. Das ist die Stille zwischen den Tönen von eben, diesmal automatisch für jede Länge richtig."]),

  H.h2("Eine Sirene, die man sehen kann"),

  H.p("Zum Schluss ein Programm, das Klang und Bild verbindet — und dabei ganz nebenbei zeigt, wie man einen Verlauf sichtbar macht:"),

  H.figure("kap10_3_sirene.png", "Die Kurve zeigt die letzten 640 Frequenzwerte. Sie wandert nach links, während rechts neue hinzukommen.", 440, 280),

  H.code([
    'SCREEN(640, 400, "Sirene")',
    "",
    "DIM verlauf[640] AS INTEGER",
    "DIM n AS INTEGER",
    "DIM i AS INTEGER",
    "DIM hz AS FLOAT",
    "DIM ton AS SOUND",
    "DIM schrift AS INTEGER",
    "n = 0",
    "schrift = RGB(190, 200, 225)",
    "",
    "FOR i = 0 TO 639",
    "    verlauf[i] = 500",
    "NEXT",
    "",
    "WHILE NOT QUITREQUESTED() AND NOT KEYPRESSED(KEY_ESCAPE)",
    "    hz = 500 + SIN(RAD(n * 2)) * 300",
    "",
    "    FOR i = 0 TO 638",
    "        verlauf[i] = verlauf[i + 1]",
    "    NEXT",
    "    verlauf[639] = INT(hz)",
    "",
    "    CLS(RGB(15, 20, 40))",
    "    FOR i = 0 TO 639",
    "        PLOT(i, 360 - (verlauf[i] - 200) / 3, RGB(255, 170, 50))",
    "        PLOT(i, 361 - (verlauf[i] - 200) / 3, RGB(255, 170, 50))",
    "    NEXT",
    "    BOX(0, 372, 639, 399, RGB(30, 38, 62))",
    '    TEXT(240, 378, "Frequenz: " + STR$(INT(hz)) + " Hz", schrift)',
    "    FLIP()",
    "",
    "    IF n MOD 14 = 0 THEN",
    "        ton = AUDIO_TONE(hz, 260)",
    "        PLAYSOUND(ton)",
    "    END IF",
    "",
    "    n = n + 1",
    "WEND",
  ]),

  H.pmix(["Die Tonhöhe kommt aus dem Sinus von Kapitel 3: ", ["500 + SIN(...) * 300", true], " pendelt zwischen 200 und 800 Hertz. Das ist die Sirene."]),

  H.p("Das Nachrücken der Kurve kennst du aus Kapitel 9 — es ist derselbe Handgriff wie bei der Schlange, nur ohne Kopf. Jeder Wert rutscht ein Fach nach links, und ganz rechts kommt der neue hinzu. Was links hinausfällt, ist einfach weg."),

  H.pmix([["IF n MOD 14 = 0", true], " sorgt dafür, dass nicht sechzigmal je Sekunde ein neuer Ton angestoßen wird, sondern etwa viermal. Das ist derselbe Takt-Gedanke wie bei der Schlange. Setz statt der 14 eine 4 ein, und es wird zum nervösen Gezwitscher."]),

  H.warn("Hier stehen zwei INT() und eine Zeile mit FLOAT, und das ist kein Zufall: Die Frequenz ist eine Kommazahl, das Array hält ganze Zahlen. Ohne INT bricht das Programm ab mit „FLOAT 510.46 passt nicht verlustfrei in INTEGER“. Beim Schreiben dieses Kapitels ist mir genau das zweimal passiert — und der Compiler hat es beide Male NICHT vorher gemeldet, weil er den Text prüft und nicht die Werte.", "Wenn eine Kommazahl in ein ganzzahliges Fach soll"),

  H.h2("Eine Zeile, die du oft sehen wirst"),

  H.pmix(["In der Befehlsübersicht und in fremden Programmen steht bei diesen Klangbefehlen oft ein ", ['IMPORT "audio"', true], " obenan. Drachenhauch braucht das nicht — die Klangbefehle sind fest eingebaut, und alle Programme dieses Kapitels laufen ohne. Die Zeile schadet aber auch nichts; wundere dich nur nicht, wenn sie fehlt."]),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    ["Du hörst gar nichts", "Lautsprecher, Lautstärke, Kopfhörer — in dieser Reihenfolge prüfen. Das Programm meldet sich nicht, wenn niemand zuhört."],
    ["Der Ton bricht ab", "Das SLEEP am Ende fehlt oder ist kürzer als der Ton. PLAYSOUND wartet nicht."],
    ["Alle Töne kommen auf einmal", "Zwischen den PLAYSOUND-Aufrufen fehlt das SLEEP. Sie starten sonst alle im selben Augenblick."],
    [{ text: "passt nicht verlustfrei in INTEGER", mono: true }, "Eine Kommazahl soll in ein ganzzahliges Fach. INT() darum herum."],
    ["Es knackt zwischen den Tönen", "Sollte nicht passieren — erzeugte Töne bekommen automatisch ein kurzes Ein- und Ausblenden. Wenn doch, ist die Dauer sehr kurz gewählt."],
    ["Nach langer Laufzeit wird es langsam", "Bei sehr vielen erzeugten Tönen sammeln sich Klangpuffer an. Erzeug einen Ton EINMAL vor der Schleife statt in jedem Durchgang neu — dazu mehr im nächsten Kapitel."],
  ], { headers: ["Was du hörst", "Was meistens dahintersteckt"], widths: [3600, 5426] }),

  H.h2("Aufgaben"),

  H.bullet("Spiel die Tonleiter rückwärts. Eine Schleife mit STEP -1 genügt."),
  H.bullet("Lass die Tonleiter zweimal laufen: einmal als sine, einmal als square. Der Unterschied ist verblüffend."),
  H.bullet("Bau einen Weckton: dreimal derselbe kurze Piepser, dann eine Pause, und das Ganze fünfmal."),
  H.bullet("Verlängere die Melodie um die restlichen Zeilen von „Alle meine Entchen“. Die Noten stehen in jedem Liederbuch; die Frequenztabelle in diesem Kapitel hilft beim Übersetzen."),
  H.bullet("Mach aus der Sirene eine, die nicht schwingt, sondern gleichmäßig steigt und dann zurückspringt — wie ein Countdown."),
  H.bullet("Lass die Sirene ihre Farbe mit der Tonhöhe ändern: tief rot, hoch weiß."),

  H.p("Klang für sich ist eine Spielerei. Im nächsten Kapitel bekommt er eine Aufgabe — er sagt dir, dass etwas passiert ist."),
];
