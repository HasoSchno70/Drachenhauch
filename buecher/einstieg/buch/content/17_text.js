module.exports = (H) => [
  H.chapter("Text"),

  H.p("Text kam in diesem Buch bisher fast nur als Beschriftung vor. Dabei ist er eine Sorte Wert wie jede andere — und man kann mit ihm ähnlich viel anstellen wie mit Zahlen, nur eben andere Dinge."),

  H.h2("Ein Text hat eine Länge und Teile"),

  H.code([
    "DIM t AS STRING",
    't = "Drachenhauch"',
    "",
    "PRINT LEN(t)",
    "PRINT LEFT$(t, 6)",
    "PRINT RIGHT$(t, 5)",
    "PRINT MID$(t, 7, 5)",
  ]),

  H.code(["12", "Drache", "hauch", "hauch"], { out: true }),

  H.pmix([["LEN", true], " zählt die Zeichen — derselbe Befehl wie bei Arrays. ", ["LEFT$", true], " nimmt vorn ab, ", ["RIGHT$", true], " hinten, ", ["MID$", true], " aus der Mitte: erst ab wo, dann wie viele."]),

  H.warn("MID$ zählt ab NULL, nicht ab eins. MID$(t, 0, 3) liefert „Dra“, MID$(t, 1, 1) liefert „r“ — nicht „D“. Das ist konsequent zu den Arrays und ungewöhnlich für BASIC: In den meisten alten Dialekten beginnt MID$ bei eins. Wer von dort kommt, tippt sich hier einmal in die Irre. Dasselbe gilt für INSTR.", "Auch hier wird ab null gezählt"),

  H.pmix(["Zähl im Wort ", ["Drachenhauch", true], " einmal mit: D ist 0, r ist 1, a ist 2 — und Stelle 7 ist das zweite h. Fünf Zeichen ab dort sind „hauch“. Wer stattdessen bei eins zu zählen begänne, landete beim n und bekäme „nhauc“."]),

  H.h2("Suchen, ersetzen, umformen"),

  H.code([
    'PRINT INSTR(t, "hauch")',
    'PRINT INSTR(t, "xyz")',
    'PRINT REPLACE$(t, "hauch", "flug")',
    "PRINT UPPER$(t)",
    'PRINT TRIM$("  Luft  ")',
  ]),

  H.code(["7", "-1", "Drachenflug", "DRACHENHAUCH", "Luft"], { out: true }),

  H.pmix([["INSTR", true], " sagt, an welcher Stelle ein Textstück beginnt — und ", ["-1", true], ", wenn es gar nicht vorkommt. Diese ", ["-1", true], " ist wichtig: Ein Programm, das blind mit dem Ergebnis weiterrechnet, greift sonst an einer unmöglichen Stelle zu."]),

  H.pmix([["TRIM$", true], " wirft Leerzeichen am Anfang und Ende weg. Das braucht man ständig, sobald Text von außen kommt — aus einer Datei oder von einem Menschen."]),

  H.h2("Zwischen Text und Zahl"),

  H.table([
    [{ text: "STR$(42)", mono: true }, "macht aus der Zahl den Text „42“"],
    [{ text: 'VAL("42")', mono: true }, "macht aus dem Text die Zahl 42"],
    [{ text: "CHR$(65)", mono: true }, "macht aus der Nummer 65 das Zeichen „A“"],
    [{ text: 'ASC("A")', mono: true }, "macht aus dem Zeichen „A“ die Nummer 65"],
    [{ text: 'SPLIT$("a,b,c", ",")', mono: true }, "zerlegt in ein Array aus drei Teilen"],
    [{ text: 'JOIN$(teile, " | ")', mono: true }, "fügt ein Array wieder zu einer Zeile"],
  ], { headers: ["Aufruf", "Was er tut"], widths: [2800, 6226], mono: [0] }),

  H.pmix([["SPLIT$", true], " und ", ["JOIN$", true], " sind ein Paar und werden im nächsten Kapitel wichtig: Sie sind der übliche Weg, eine Zeile aus einer Datei in ihre Felder zu zerlegen und wieder zusammenzusetzen."]),

  H.note("Umlaute zählen als EIN Zeichen. LEN(\"Käse\") ist 4, und LEFT$ davon zwei Zeichen ergibt „Kä“. Das klingt selbstverständlich, ist es aber nicht — in vielen Sprachen zählt ein Umlaut als zwei, und Text zerbricht dann mitten im Buchstaben."),

  H.h2("Text, der sich selbst tippt"),

  H.p("Der erste Trick mit Text ist auch der hübscheste: Man zeigt einfach immer ein Zeichen mehr."),

  H.code([
    "TEXT(40, 180, LEFT$(satz, sichtbar), schrift)",
  ]),

  H.code([
    "takt = takt + 1",
    "IF takt MOD 4 = 0 AND sichtbar < LEN(satz) THEN",
    "    sichtbar = sichtbar + 1",
    "END IF",
  ]),

  H.figure("kap17_1_schreibmaschine.png", "Alle vier Bilder kommt ein Zeichen dazu. Der Balken dahinter blinkt.", 440, 280),

  H.pmix(["Das ist alles: ein Zähler, der langsam wächst, und ein ", ["LEFT$", true], ", das so viele Zeichen zeigt. Es sieht aufwendig aus und ist eine Zeile."]),

  H.p("Der blinkende Cursor dahinter braucht zwei Zutaten. Erstens den Takt-Kniff aus Kapitel 9, hier für das Blinken:"),

  H.code([
    "IF (takt \\ 20) MOD 2 = 0 THEN",
    "    BOX(40 + TEXT_WIDTH(LEFT$(satz, sichtbar)), 180, _",
    "        46 + TEXT_WIDTH(LEFT$(satz, sichtbar)), 206, cursor)",
    "END IF",
  ]),

  H.pmix(["Und zweitens ", ["TEXT_WIDTH", true], ", das misst, wie breit ein Text in der aktuellen Schriftgröße wird. Ohne diese Messung wüsste man nicht, wo der Text aufhört — die Zeichen sind ja verschieden breit."]),

  H.h2("Eine Laufschrift"),

  H.p("Der zweite Trick nutzt MID$: Man zeigt immer einen Ausschnitt und verschiebt dessen Anfang."),

  H.code([
    "doppelt = spruch + spruch",
  ]),

  H.code([
    "TEXT(20, 188, MID$(doppelt, start, fenster), schrift)",
  ]),

  H.code([
    "takt = takt + 1",
    "IF takt MOD 5 = 0 THEN",
    "    start = start + 1",
    "    IF start >= LEN(spruch) THEN start = 0",
    "END IF",
  ]),

  H.figure("kap17_2_laufschrift.png", "Ein Ausschnitt aus einem doppelt aneinandergehängten Text — dadurch gibt es keine Lücke beim Umbruch.", 440, 280),

  H.p("Der Kniff steckt in der ersten Zeile. Der Text wird an sich selbst gehängt, und der Ausschnitt wandert nur über die erste Hälfte. Dadurch ist am Ende immer noch genug übrig, um das Fenster zu füllen — die Laufschrift läuft nahtlos rundherum, ohne dass irgendwo ein Sonderfall stünde."),

  H.warn("Beim Schreiben dieses Programms hieß die Variable zuerst text — und lange Zeit brach das Programm damit ab: „'text' ist eine Variable vom Typ STRING und kann nicht wie eine Funktion aufgerufen werden.“ Inzwischen versteht Drachenhauch den Fall: hat die Variable einen festen Typ, bleibt TEXT(...) der Befehl, und text als STRING daneben stört nicht — genauso wie len = LEN(zeile) oder deg = DEG(winkel). Die Meldung kommt nur noch, wenn die Variable eine FUNCREF ist, denn dann könnte tatsächlich sie gemeint sein. Umbenannt habe ich sie trotzdem: Ein Name, der nach dem Befehl klingt, liest sich schlechter.", "Namen, die schon vergeben sind"),

  H.h2("Buchstaben zählen"),

  H.p("Zum Schluss etwas, das zwei Kapitel verbindet. Ein Text wird Zeichen für Zeichen durchgegangen, und jedes landet in einer Map — genau die Zählzeile aus Kapitel 15:"),

  H.code([
    "FOR i = 0 TO LEN(satz) - 1",
    "    c = MID$(satz, i, 1)",
    '    IF c <> " " THEN',
    "        MAPPUT(zaehler, c, MAPGETOR(zaehler, c, 0) + 1)",
    "    END IF",
    "NEXT",
  ]),

  H.figure("kap17_3_buchstaben.png", "Nachgezählt: Das N kommt siebenmal vor, das E fünfmal. Sechzehn verschiedene Buchstaben.", 440, 280),

  H.pmix([["MID$(satz, i, 1)", true], " holt genau ein Zeichen an der Stelle i. Damit wird ein Text zu etwas, das man durchlaufen kann wie ein Array — und weil ab null gezählt wird, läuft die Schleife von 0 bis ", ["LEN(satz) - 1", true], ", genau wie bei einem Array."]),

  H.p("Sechs Zeilen, und du hast eine Häufigkeitsauswertung. Genau so funktionieren im Kern auch die Verfahren, mit denen man Sprachen erkennt oder einfache Geheimschriften knackt: Im Deutschen ist das E mit Abstand am häufigsten, und daran ändert auch eine Verschlüsselung nichts, die nur Buchstaben vertauscht."),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    ["Das erste Zeichen fehlt", "MID$ ab 1 statt ab 0 begonnen."],
    ["Ein Zeichen zu wenig am Ende", "Die Schleife läuft bis LEN statt bis LEN minus eins."],
    [{ text: "kann nicht wie eine Funktion aufgerufen werden", mono: true }, "Eine FUNCREF-Variable heißt wie ein eingebauter Befehl. Umbenennen."],
    ["Die Suche findet immer etwas", "INSTR liefert -1 für „nicht gefunden“, nicht 0. Eine Prüfung auf „größer null“ übersieht das erste Zeichen."],
    ["Die Laufschrift stockt am Ende", "Der Text wurde nicht verdoppelt; MID$ läuft über das Ende hinaus."],
    ["Zahlen lassen sich nicht anhängen", "STR$ vergessen — oder umgekehrt VAL, wenn aus Text eine Zahl werden soll."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3800, 5226] }),

  H.h2("Aufgaben"),

  H.bullet("Lass die Schreibmaschine bei jedem Zeichen ein kurzes Klicken hören. AUDIO_NOISE mit sechzig Millisekunden passt gut."),
  H.bullet("Bau die Laufschrift so um, dass sie von unten nach oben läuft — wie ein Abspann."),
  H.bullet("Schreib eine Funktion, die einen Text rückwärts zurückgibt. Du brauchst dafür eine Schleife und MID$."),
  H.bullet("Prüf mit einer Funktion, ob ein Wort ein Palindrom ist — also vorwärts wie rückwärts gleich."),
  H.bullet("Zähl in der Buchstabenauswertung Groß- und Kleinschreibung getrennt, indem du das UPPER$ weglässt. Erkläre dir das Ergebnis."),
  H.bullet("Bau eine einfache Geheimschrift: Verschieb jeden Buchstaben um drei Stellen im Alphabet. CHR$ und ASC sind alles, was du brauchst."),

  H.p("Text kannst du jetzt zerlegen und zusammensetzen. Was noch fehlt, damit ein Programm etwas behalten kann: Es muss schreiben und lesen dürfen."),
];
