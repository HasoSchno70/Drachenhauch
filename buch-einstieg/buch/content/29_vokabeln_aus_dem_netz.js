module.exports = (H) => [
  H.chapter("Vokabeln aus dem Netz"),

  H.p("Ein Vokabeltrainer, in den man erst zweihundert Wörter tippen muss, wird nie benutzt. Also holt er sie sich — aus dem Internet, in dem Moment, in dem man ihn startet."),

  H.p("Das klingt nach einem großen Schritt, ist aber ein einziger Befehl."),

  H.h2("Ein Befehl, und der Text ist da"),

  H.code([
    'IMPORT "html"',
    "",
    'text = HTTP_GET("https://example.org/liste.txt")',
  ]),

  H.pmix([["HTTP_GET", true], " tut genau das, was ein Browser tut, wenn du eine Adresse eintippst: Es fragt einen Rechner irgendwo nach einer Datei und liefert zurück, was er schickt. Der Unterschied ist nur, dass hier kein Fenster aufgeht, sondern ein ", ["STRING", true], " in einer Variablen landet."]),

  H.pmix(["Das Modul heißt ", ['"html"', true], ", weil dort auch die Befehle zum Zerlegen von Webseiten liegen. Wir brauchen davon nur den Abruf."]),

  H.h2("Wo die Listen liegen"),

  H.p("Die Vokabellisten dieses Buchs liegen im selben Ordner wie sein Quelltext, auf GitHub. Jede Datei hat dort eine Adresse, unter der der reine Text steht:"),

  H.code([
    'basis = "https://raw.githubusercontent.com/HasoSchno70/" + _',
    '        "Drachenhauch/main/buch-einstieg/vokabellisten/"',
  ]),

  H.note("Diese Wahl ist bewusst so getroffen. Man könnte auch eine Übersetzungs-Schnittstelle im Netz anzapfen — nur ist die morgen vielleicht kostenpflichtig, umgezogen oder abgeschaltet, und dann steht in einem gedruckten Buch eine Adresse, die ins Leere führt. Die Listen dieses Buchs liegen dort, wo das Buch liegt. Wer beides hat, hat alles."),

  H.p("Eine Liste sieht so aus — schlichter Text, eine Vokabel je Zeile:"),

  H.code([
    "# name: Englisch Grundwortschatz",
    "# sprache: Englisch",
    "# Zeilen mit # sind Anmerkungen. Sonst: deutsch;fremdsprache",
    "Haus;house",
    "Baum;tree",
    "Katze;cat",
  ]),

  H.p("Und daneben liegt ein Katalog, der sagt, welche Listen es überhaupt gibt:"),

  H.code([
    "# Verzeichnis der Vokabellisten dieses Buchs.",
    "# Zeilen mit # sind Anmerkungen. Sonst: datei;name",
    "englisch_grund.txt;Englisch Grundwortschatz",
    "franzoesisch_grund.txt;Französisch Grundwortschatz",
    "spanisch_grund.txt;Spanisch Grundwortschatz",
  ]),

  H.pmix(["Der Katalog steht in derselben Form da wie die Listen selbst: Anmerkungen mit ", ["#", true], ", Felder mit Strichpunkt. Ein Format, ein Leser — die Routine, die den Katalog zerlegt, zerlegt auch die Listen."]),

  H.warn("Auffällig ist, was NICHT im Katalog steht: die Sprache. Die steht in der Liste selbst, in ihrer Kopfzeile. Stünde sie an beiden Stellen, könnten die beiden Stellen sich widersprechen — genau der Fehler, gegen den Kapitel 28 die drei Tabellen angelegt hat. Jede Tatsache an genau einer Stelle, auch in Textdateien.", "Wo die Sprache steht"),

  H.h2("Text in Zeilen und Felder zerlegen"),

  H.p("Zwei Befehle sind neu, und beide sind so nützlich, dass man sie nie wieder vergisst:"),

  H.code([
    "zeilen = SPLIT$(text, CHR$(10))",
    'teile = SPLIT$("Haus;house", ";")',
  ]),

  H.pmix([["SPLIT$(text, trenner)", true], " zerschneidet einen Text an jedem Vorkommen des Trenners und liefert ein ", ["ARRAY OF STRING", true], ". Beim ersten Aufruf ist der Trenner ", ["CHR$(10)", true], " — das Zeichen für den Zeilenumbruch. Ein Text mit fünf Zeilen wird zu einem Array mit fünf Fächern."]),

  H.pmix([["CHR$(n)", true], " liefert das Zeichen mit der Nummer ", ["n", true], ". Die 10 ist der Zeilenumbruch. Man kann ihn nicht direkt zwischen Anführungszeichen schreiben — deshalb dieser Umweg."]),

  H.h2("Das Netz kann nein sagen"),

  H.p("Und damit zum wichtigsten Satz dieses Kapitels: Ein Abruf über das Netz geht schief. Nicht immer, aber oft genug, dass ein Programm damit rechnen muss. Das WLAN ist weg, der Rechner am anderen Ende antwortet nicht, die Datei ist umbenannt worden."),

  H.pmix(["Passiert das, bricht ", ["HTTP_GET", true], " das Programm ab — nachgemessen mit einer Adresse, die es (noch) nicht gab:"]),

  H.code([
    "Laufzeitfehler in x.dh:4: HTTP 404 Not Found - https://raw.git...",
  ], { out: true }),

  H.p("Dagegen gibt es ein Mittel, und es ist so grundlegend, dass fast jede Programmiersprache es kennt:"),

  H.code([
    "TRY",
    '    zeilen = SPLIT$(HTTP_GET(basis + "katalog.txt"), CHR$(10))',
    '    quelle = "aus dem Netz, Status " + STR$(HTTP_STATUS())',
    "CATCH e",
    '    zeilen = READLINES("katalog.txt")',
    '    quelle = "von der Platte, weil das Netz " + _',
    '             STR$(HTTP_STATUS()) + " sagte"',
    "END TRY",
  ]),

  H.pmix(["Zwischen ", ["TRY", true], " und ", ["CATCH", true], " steht, was gelingen soll. Geht es schief, springt das Programm sofort in den ", ["CATCH", true], "-Zweig — es bricht nicht ab. Die Variable dahinter (hier ", ["e", true], ") enthält die Fehlermeldung als Text."]),

  H.pmix([["HTTP_STATUS()", true], " ist auch nach einem Fehlschlag noch lesbar. Gemessen: nach einer fehlenden Datei steht dort ", ["404", true], ", nach einem gelungenen Abruf ", ["200", true], ". Die Zahlen sind dieselben, die dein Browser meldet, wenn eine Seite nicht da ist."]),

  H.warn("TRY ist kein Freibrief, alles hineinzupacken und nie wieder hinzusehen. Ein CATCH-Zweig, der stumm weitermacht, verwandelt einen Fehler in ein Rätsel. Hier tut er zwei sinnvolle Dinge: Er greift auf die Datei neben dem Programm zurück, und er SAGT, dass er es getan hat. Wer das Fenster ansieht, weiß sofort, woher die Zeilen kommen.", "Fangen heißt nicht verschweigen"),

  H.h2("Das ganze Programm"),

  H.code([
    "' Kapitel 29 -- den Katalog aus dem Internet holen und anzeigen.",
    "' Geht das Netz nicht, nimmt das Programm die Datei von der Platte.",
    "",
    'IMPORT "html"',
    "",
    'SCREEN(640, 400, "Aus dem Netz")',
    "",
    "DIM basis AS STRING",
    "DIM quelle AS STRING",
    "DIM zeilen AS ARRAY OF STRING",
    "DIM z AS STRING",
    "DIM y AS INTEGER",
    "",
    'basis = "https://raw.githubusercontent.com/HasoSchno70/" + _',
    '        "Drachenhauch/main/buch-einstieg/vokabellisten/"',
    "",
    "HTTP_TIMEOUT(8)",
    "TRY",
    '    zeilen = SPLIT$(HTTP_GET(basis + "katalog.txt"), CHR$(10))',
    '    quelle = "aus dem Netz, Status " + STR$(HTTP_STATUS())',
    "CATCH e",
    '    zeilen = READLINES("katalog.txt")',
    '    quelle = "von der Platte, weil das Netz " + _',
    '             STR$(HTTP_STATUS()) + " sagte"',
    "END TRY",
    "",
    "WHILE NOT QUITREQUESTED() AND NOT KEYPRESSED(KEY_ESCAPE)",
    "    CLS(RGB(28, 32, 50))",
    "    TEXT(20, 20, quelle, RGB(120, 220, 140))",
    '    TEXT(20, 44, STR$(LEN(zeilen)) + " Zeilen", RGB(150, 165, 190))',
    "    y = 84",
    "    FOR EACH z IN zeilen",
    "        TEXT(20, y, z, RGB(200, 210, 230))",
    "        y = y + 22",
    "    NEXT",
    "    FLIP()",
    "WEND",
  ]),

  H.figure("kap29_1_aus_dem_netz.png", "Text, der eben noch auf einem Rechner in einem anderen Land lag.", 440, 280),

  H.pmix([["HTTP_TIMEOUT(8)", true], " sagt, wie lange das Programm auf eine Antwort wartet, bevor es aufgibt: acht Sekunden. Ohne diese Zeile wären es zehn. Wer je vor einem Programm gesessen hat, das minutenlang nichts tut, weiß, warum man diese Zahl klein hält."]),

  H.h2("Ohne dass das Fenster stehenbleibt"),

  H.p("Das Programm oben hat einen Schönheitsfehler, den man erst bei langsamer Leitung bemerkt: Solange der Abruf läuft, tut es gar nichts. Kein Bild, keine Reaktion, das Fenster ist eingefroren. Bei acht Sekunden Wartezeit sind das im schlimmsten Fall acht Sekunden Totenstille."),

  H.p("Dagegen gibt es einen zweiten Weg, und er hat dieselbe Form wie das Polling aus Kapitel 24: anstoßen, weiterlaufen, in jedem Bild nachfragen."),

  H.code([
    "abruf = HTTP_GET_START(basis + dateien[0])",
  ]),

  H.code([
    "IF abruf >= 0 THEN",
    "    punkte = punkte + 1",
    '    GUI_SET_TEXT(stand, "hole" + REPEAT$(".", punkte \\ 10 + 1))',
    "    IF HTTP_READY(abruf) THEN",
    "        ' ... hier ist die Antwort da ...",
    "        abruf = -1",
    "    END IF",
    "END IF",
  ]),

  H.pmix([["HTTP_GET_START", true], " startet den Abruf und kehrt SOFORT zurück — mit einer Nummer, unter der man ihn wiederfindet. ", ["HTTP_READY(nummer)", true], " fragt nach, ob die Antwort da ist, und wartet dabei nicht. ", ["HTTP_RESULT(nummer)", true], " holt sie ab."]),

  H.pmix(["Die Zeile mit ", ["REPEAT$", true], " ist reine Höflichkeit: Sie hängt alle zehn Bilder einen Punkt an, so dass „hole…“ sich bewegt. Ein Programm, das erkennbar arbeitet, wirkt schneller als eines, das nur stillsteht — auch wenn es genauso lange braucht."]),

  H.warn("HTTP_RESULT wirft denselben Fehler wie HTTP_GET, wenn die Antwort ein 404 war — nur eben beim Abholen statt beim Starten. Das TRY gehört also um HTTP_RESULT herum, nicht um HTTP_GET_START. Gemessen: HTTP_GET_START lieferte brav die Nummer 0, und erst HTTP_RESULT sagte „HTTP 404 Not Found“.", "Der Fehler kommt später"),

  H.figure("kap29_2_liste_laden.png", "Sprache oben auswählen, Liste unten. Während des Ladens bleibt das Fenster bedienbar.", 440, 280),

  H.h2("Ein Leser für beide Formate"),

  H.p("Katalog und Vokabelliste haben dieselbe Form, also genügt eine Routine für beide:"),

  H.code([
    "SUB spalten_lesen(zeilen AS ARRAY OF STRING, _",
    "                  links AS ARRAY OF STRING, _",
    "                  rechts AS ARRAY OF STRING)",
    "    DIM roh AS STRING",
    "    DIM z AS STRING",
    "    DIM t AS ARRAY OF STRING",
    "    FOR EACH roh IN zeilen",
    "        z = TRIM$(roh)",
    '        IF z <> "" AND LEFT$(z, 1) <> "#" THEN',
    '            t = SPLIT$(z, ";")',
    "            IF LEN(t) >= 2 THEN",
    "                ARRAY_PUSH(links, TRIM$(t[0]))",
    "                ARRAY_PUSH(rechts, TRIM$(t[1]))",
    "            END IF",
    "        END IF",
    "    NEXT",
    "END SUB",
  ]),

  H.p("Vier Zeilen davon sind Vorsicht, und jede hat einen Grund:"),

  H.bulletRich("TRIM$(roh) ", "— Textdateien aus dem Netz enden je nach Herkunft mit einem unsichtbaren Wagenrücklauf. TRIM$ nimmt ihn mit weg."),
  H.bulletRich("z <> \"\" ", "— Leerzeilen sind keine Vokabeln."),
  H.bulletRich("LEFT$(z, 1) <> \"#\" ", "— Anmerkungen sind keine Vokabeln."),
  H.bulletRich("LEN(t) >= 2 ", "— eine Zeile ohne Strichpunkt hat nur ein Feld. Ohne diese Prüfung würde t[1] das Programm abbrechen."),

  H.pmix(["Neu ist hier, dass eine ", ["SUB", true], " Arrays bekommt und sie FÜLLT. Was du in Kapitel 13 über Parameter gelernt hast, gilt weiter — nur werden Arrays nicht kopiert, sondern hereingereicht. Was die SUB hineinschreibt, steht nachher draußen drin. Gemessen: nach zwei ", ["ARRAY_PUSH", true], " in der SUB hatte das Array draußen zwei Einträge."]),

  H.h2("Fremde Zeichen"),

  H.p("Ein Vokabeltrainer ist das erste Programm dieses Buchs, in dem Umlaute und Akzente nicht Zierrat sind, sondern Inhalt. „el ano“ und „el año“ sind zwei verschiedene Wörter, und nur eines davon heißt „das Jahr“."),

  H.p("Drei Dinge dazu, alle nachgemessen:"),

  H.bulletRich("LEN zählt Zeichen, nicht Bytes. ", "LEN(\"el año\") ist 6, obwohl das ñ zwei Bytes braucht. Auch MID$ und LEFT$ rechnen in Zeichen — mit Akzenten geht alles so, wie man es erwartet."),
  H.bulletRich("LOWER$ kennt Akzente. ", "LOWER$(\"EL AÑO\") liefert „el año“, nicht „el aÑo“."),
  H.bulletRich("Die eingebaute Schrift kennt nur ASCII. ", "Kommt ein Zeichen darüber hinaus vor, zeichnet die Runtime DIESEN Text mit einer Systemschrift. Das rettet die Darstellung — sieht aber eigenartig aus, weil im selben Bild plötzlich zwei Schriften stehen."),

  H.p("Das letzte Verhalten war beim Schreiben dieses Kapitels gut zu sehen: Von fünf Zeilen im Fenster stand genau eine in einer anderen Schrift — die mit dem „ö“ in „Französisch“. Der Ausweich-Font greift je Text, nicht je Programm."),

  H.p("Wer ein einheitliches Bild will, lädt selbst eine Schrift:"),

  H.code([
    'schrift = LOADFONT("C:/Windows/Fonts/segoeui.ttf", 20)',
    "IF schrift >= 0 THEN SETFONT(schrift)",
  ]),

  H.p("Danach zeichnet alles in derselben Schrift, mit Akzenten wie ohne. Genau das tut der Trainer in Kapitel 33 — und weil der Pfad auf jedem System anders heißt, probiert er mehrere durch."),

  H.h2("Die Netz-Befehle im Überblick"),

  H.table([
    [{ text: "HTTP_GET(url)", mono: true }, "holen und warten, bis es da ist"],
    [{ text: "HTTP_STATUS()", mono: true }, "200 = alles gut, 404 = nicht da"],
    [{ text: "HTTP_TIMEOUT(sekunden)", mono: true }, "wie lange gewartet wird"],
    [{ text: "HTTP_GET_START(url)", mono: true }, "anstoßen, Nummer zurück"],
    [{ text: "HTTP_READY(nummer)", mono: true }, "ist die Antwort da?"],
    [{ text: "HTTP_RESULT(nummer)", mono: true }, "Antwort abholen"],
    [{ text: "SPLIT$(text, trenner)", mono: true }, "Text zu Array zerlegen"],
    [{ text: "JOIN$(array, trenner)", mono: true }, "Array wieder zu Text"],
    [{ text: "CHR$(10)", mono: true }, "das Zeichen für den Zeilenumbruch"],
  ], { headers: ["Aufruf", "Was er tut"], widths: [3400, 5626], mono: [0] }),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    [{ text: "HTTP 404 Not Found", mono: true }, "Die Adresse stimmt nicht. Kopier sie in einen Browser — steht der Text da?"],
    [{ text: "Unbekanntes Builtin 'HTTP_GET'", mono: true }, "IMPORT \"html\" fehlt."],
    ["Das Fenster hängt beim Start", "Blockierendes HTTP_GET bei langsamer Leitung. HTTP_GET_START nehmen."],
    ["Die letzte Vokabel fehlt", "Die Datei endet ohne Zeilenumbruch, oder die Prüfung auf Leerzeilen wirft sie weg."],
    ["Hinter jeder Vokabel steht etwas Unsichtbares", "Wagenrücklauf am Zeilenende. TRIM$ um jede Zeile."],
    [{ text: "Index 1 ausserhalb [0..0]", mono: true }, "Eine Zeile ohne Strichpunkt. Die Prüfung LEN(t) >= 2 fehlt."],
    ["Akzente werden zu Fragezeichen", "Weder eigene Schrift noch Systemschrift gefunden. LOADFONT mit einem Pfad, den es gibt."],
    ["Zwei verschiedene Schriften im selben Bild", "Der Ausweich-Font greift nur bei den Texten mit Akzent. SETFONT für alle."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3600, 5426] }),

  H.h2("Aufgaben"),

  H.bullet("Lass das Programm zusätzlich anzeigen, wie viele Zeichen die Antwort hatte, und vergleich das mit der Dateigröße im Browser."),
  H.bullet("Bau eine vierte Liste — schreib eine eigene Textdatei im selben Format und lies sie mit READLINES ein."),
  H.bullet("Zeig beim Hintergrundabruf statt der Punkte einen wandernden Balken."),
  H.bullet("Lass den Abruf absichtlich scheitern, indem du einen Buchstaben in der Adresse änderst. Welche Zahl steht dann in HTTP_STATUS?"),
  H.bullet("Ergänze eine Kopfzeile „# stufe:“ in einer Liste und lies sie mit aus."),
  H.bullet("Schreib eine FUNCTION, die aus einem Zeilen-Array eine bestimmte Kopfzeile heraussucht. In Kapitel 32 wird genau die gebraucht."),

  H.p("Die Vokabeln sind da. Jetzt kommt die Frage, die einen Trainer von einer Liste unterscheidet: Welche davon ist als Nächstes dran?"),
];
