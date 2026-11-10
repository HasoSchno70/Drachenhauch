module.exports = (H) => [
  H.chapter("Der Trainer"),

  H.p("Jetzt wird zusammengesetzt. Der Karteikasten aus Kapitel 30 sucht aus, die Fragearten aus Kapitel 31 fragen, die Verwaltung aus Kapitel 32 liefert die Vokabeln, und das Netz aus Kapitel 29 füllt sie."),

  H.pmix(["Das fertige Programm steht als ", ["code/kap33/trainer.dh", true], " neben dem Buch. Es hat 640 Zeilen — und darin ist kein einziger Befehl, den du nicht kennst. Deshalb steht es hier nicht ganz abgedruckt; stattdessen die Stellen, an denen die Teile aneinandergefügt werden."]),

  H.figure("kap33_trainer.png", "Der allererste Start: Die Vokabel wird gezeigt, nicht abgefragt. Was danach kommt, entscheidet der Karteikasten.", 440, 300),

  H.note("Beim allerersten Start ist die Datenbank leer, und ein leeres Fenster sagt niemandem, wofür es gut ist. Deshalb liest der Trainer dann die Datei englisch_grund.txt ein, die neben ihm liegt — derselbe Handgriff wie beim Vorbefüllen in Kapitel 27. Danach passiert es nie wieder, denn ab da ist die Tabelle nicht mehr leer."),

  H.h2("Vier Reiter"),

  H.code([
    'reiter[0] = "Lernen"',
    'reiter[1] = "Vokabeln"',
    'reiter[2] = "Aus dem Netz"',
    'reiter[3] = "Fortschritt"',
  ]),

  H.p("Die Reihenfolge ist eine Aussage: Vorn steht, weswegen man das Programm öffnet. Verwaltung, Nachschub und Statistik sind Nebensachen — sie sind da, aber sie stehen nicht im Weg."),

  H.p("Und beim Start entscheidet das Programm selbst, wo es aufmacht:"),

  H.code([
    "' Auf dem Reiter starten, wo es etwas zu tun gibt.",
    'erg = DB_QUERY(con, "SELECT COUNT(*) FROM listen")',
    "DB_NEXT(erg)",
    "IF DB_GET_INT(erg, 0) = 0 THEN GUI_SET_ACTIVE_TAB(fenster, 2)",
    "DB_CLOSE_RESULT(erg)",
  ]),

  H.p("Wer noch keine Liste hat, landet direkt dort, wo er eine bekommt. Vier Zeilen, und niemand muss mehr raten, wo er anfangen soll."),

  H.h2("Ein Reiter, den es nicht gibt"),

  H.p("Auf dem Reiter „Lernen“ stehen mal vier Antwortknöpfe, mal ein Eingabefeld, mal gar nichts davon. Die Oberfläche wird aber nur EINMAL gebaut, vor der Schleife — so war das seit Kapitel 24."),

  H.p("Wie versteckt man dann etwas? Über die Reiterzuordnung:"),

  H.code([
    "CONST VERSTECKT = 99",
  ]),

  H.p("Zu Beginn jeder Frage verschwindet erst einmal alles:"),

  H.code([
    "        FOR i = 0 TO 3",
    "            GUI_SET_TAB(w_a[i], VERSTECKT)",
    "        NEXT",
    "        GUI_SET_TAB(w_tipp, VERSTECKT)",
    "        GUI_SET_TAB(w_ok, VERSTECKT)",
    "        GUI_SET_TAB(w_weiter, VERSTECKT)",
  ]),

  H.p("Und dann kommt nur zurück, was diese eine Frage braucht:"),

  H.code([
    "                GUI_SET_TAB(w_tipp, 0)",
    "                GUI_SET_TAB(w_ok, 0)",
  ]),

  H.pmix([["GUI_SET_TAB", true], " sagt, auf welche Seite ein Bedienelement gehört. Seite 99 gibt es nicht — also ist es nirgends zu sehen. Nachgemessen mit zwei Knöpfen nebeneinander: der auf Seite 0 war da, der auf Seite 9 nicht, und er reagierte auch nicht auf Klicks."]),

  H.p("Erst alles weg, dann das Nötige zurück — das ist derselbe Gedanke wie CLS am Anfang jedes Bildes. Wer stattdessen nur das gerade Störende versteckt, vergisst irgendwann einen Fall, und dann steht ein Knopf da, der nicht dazugehört."),

  H.note("Das ist kein dokumentierter Trick, sondern eine schlichte Folge der Regel. Solche Stellen sollte man kommentieren — hier tut es der Name VERSTECKT, der sagt, was gemeint ist, statt eine nackte 99 im Code stehen zu lassen. Wer in einem halben Jahr GUI_SET_TAB(w, 99) liest, denkt an einen Tippfehler. Bei GUI_SET_TAB(w, VERSTECKT) nicht."),

  H.h2("Die Leiter"),

  H.p("Hier kommen Karteikasten und Fragearten zusammen, und es sind sechs Zeilen:"),

  H.code([
    "FUNCTION muster_zu(fach AS INTEGER, gesehen AS INTEGER) AS INTEGER",
    "    IF gesehen = 0 THEN RETURN VORSTELLEN",
    "    IF fach <= 1 THEN RETURN WAHL_DE",
    "    IF fach = 2 THEN RETURN WAHL_FREMD",
    "    IF fach = 3 THEN RETURN TIPP_DE",
    "    RETURN TIPP_FREMD",
    "END FUNCTION",
  ]),

  H.p("Die erste Zeile ist die wichtigste des ganzen Programms. Eine Vokabel, die noch nie vor Augen war, wird nicht abgefragt, sondern gezeigt. Alles andere wäre eine Prüfung über Stoff, den es nie gab — und genau das tun erstaunlich viele Vokabelprogramme."),

  H.p("Danach steigt die Schwierigkeit mit dem Fach: erkennen, erkennen andersherum, schreiben, schreiben andersherum. Wer eine Vokabel gut kann, bekommt die harte Frage."),

  H.pmix(["Die fünf Namen sind wieder ", ["CONST", true], ", und das ist hier nicht Kosmetik: ", ['IF muster = TIPP_FREMD', true], " liest sich als Satz, ", ['IF muster = 4', true], " nicht."]),

  H.h2("Was drankommt"),

  H.code([
    'erg = DB_QUERY(con, "SELECT id, de, fremd, fach, " + _',
    '                    "gesehen FROM vokabeln " + _',
    '                    "WHERE liste = ? AND faellig <= ? " + _',
    '                    "ORDER BY faellig, fach, RANDOM() " + _',
    '                    "LIMIT 1", aktuell, runde)',
  ]),

  H.p("Derselbe Satz wie in Kapitel 30, um eine Bedingung erweitert: nur aus der gewählten Liste. Zwei Fragezeichen, zwei Werte dahinter — in derselben Reihenfolge, in der die Fragezeichen stehen."),

  H.h2("Was nach der Antwort passiert"),

  H.code([
    "IF geantwortet THEN",
    "    runde = runde + 1",
    "    IF korrekt THEN",
    "        PLAYSOUND(jubel)",
    "        v_fach = MIN(5, v_fach + 1)",
    '        DB_EXEC(con, "UPDATE vokabeln SET fach = ?, " + _',
    '                     "faellig = ?, gesehen = gesehen + 1, " + _',
    '                     "richtig = richtig + 1 WHERE id = ?", _',
    "                v_fach, runde + wartezeit(v_fach), v_id)",
    "    ELSE",
    "        PLAYSOUND(brumm)",
    '        DB_EXEC(con, "UPDATE vokabeln SET fach = 1, " + _',
    '                     "faellig = ?, gesehen = gesehen + 1, " + _',
    '                     "falsch = falsch + 1 WHERE id = ?", _',
    "                runde + wartezeit(1), v_id)",
    "    END IF",
    '    DB_EXEC(con, "UPDATE stand SET wert = ? WHERE " + _',
    "                 \"schluessel = 'runde'\", runde)",
    "    FOR i = 0 TO 3",
    "        GUI_SET_TAB(w_a[i], VERSTECKT)",
    "    NEXT",
    "    GUI_SET_TAB(w_tipp, VERSTECKT)",
    "    GUI_SET_TAB(w_ok, VERSTECKT)",
    "    GUI_SET_TAB(w_weiter, 0)",
    "    neu_vokabeln = TRUE",
    "END IF",
  ]),

  H.pmix(["Ein einziger ", ["UPDATE", true], " je Antwort, und er ändert vier Spalten auf einmal. Die Buchführung steht an genau einer Stelle — egal, ob die Antwort angeklickt oder eingetippt wurde. Die beiden Fragearten setzen nur ", ["korrekt", true], ", alles Weitere ist ihnen gleich."]),

  H.warn("Der erste Entwurf hatte das anders: Dort entschied die Buchung anhand des Urteilstextes, mit LEFT$(text, 4) = \"nein\". Das ging beim Tippen prompt schief, denn ein „fast“ zählt als richtig, fängt aber nicht mit „nein“ an. Ein Programm soll nie aus dem ablesen, was es hingeschrieben hat, was es gemeint hat. Dafür gibt es Variablen.", "Nicht am Text ablesen, was man weiß"),

  H.p("Und weil das Vorstellen keine Antwort ist, wird es getrennt gebucht:"),

  H.code([
    "IF GUI_CLICKED(w_weiter) THEN",
    "    IF v_id >= 0 AND muster = VORSTELLEN AND NOT geantwortet THEN",
    "        runde = runde + 1",
    '        DB_EXEC(con, "UPDATE vokabeln SET faellig = ?, " + _',
    '                     "gesehen = gesehen + 1 WHERE id = ?", _',
    "                runde + wartezeit(1), v_id)",
    '        DB_EXEC(con, "UPDATE stand SET wert = ? WHERE " + _',
    "                     \"schluessel = 'runde'\", runde)",
    "        neu_vokabeln = TRUE",
    "    END IF",
    "    neu_frage = TRUE",
    "END IF",
  ]),

  H.pmix(["Nur ", ["gesehen", true], " steigt, ", ["richtig", true], " und ", ["falsch", true], " bleiben. Deshalb stimmt die Trefferquote im Fortschritt: Was nie eine Frage war, kann auch keine falsche Antwort gewesen sein."]),

  H.h2("Fortschritt"),

  H.code([
    "        FOR i = 1 TO 5",
    "            GUI_SET_VALUE(w_balken[i], 0.0)",
    "        NEXT",
    "        IF LEN(vok_id) > 0 THEN",
    '            erg = DB_QUERY(con, "SELECT fach, COUNT(*) FROM " + _',
    '                                "vokabeln WHERE liste = ? " + _',
    '                                "GROUP BY fach", aktuell)',
    "            WHILE DB_NEXT(erg)",
    "                GUI_SET_VALUE(w_balken[DB_GET_INT(erg, 0)], _",
    "                              DB_GET_INT(erg, 1) * 1.0 / LEN(vok_id))",
    "            WEND",
    "            DB_CLOSE_RESULT(erg)",
    "        END IF",
  ]),

  H.pmix([["GUI_PROGRESS", true], " will einen Wert zwischen 0 und 1 — deshalb die Division. Und deshalb das ", ["* 1.0", true], ": Ohne es wären beide Zahlen ganz, und die Division würde abrunden. Bei 15 von 56 käme 0 heraus. Es ist dieselbe Falle wie bei ", ["\\", true], " in Kapitel 2, nur andersherum."]),

  H.p("Das Nullsetzen davor ist nötig, weil GROUP BY nur Fächer liefert, in denen etwas steht. Ein Fach, das gerade leer geworden ist, kommt in der Antwort gar nicht vor — und der Balken bliebe auf seinem alten Wert stehen."),

  H.h2("Ein zweites Paar Augen auf dieselben Daten"),

  H.pmix(["Am Ende von Kapitel 27 stand eine Aufgabe: ein zweites, winziges Programm schreiben, das dieselbe Datenbank öffnet. Hier ist sie eingelöst — ", ["code/kap33/fortschritt.dh", true], " hat keine Oberfläche, keinen Karteikasten und keine dreißig Zeilen. Es öffnet ", ["trainer.db", true], ", stellt eine Frage und malt fünf Balken."]),

  H.figure("kap33_fortschritt.png", "Nach dem allerersten Start: alles in Fach 1. Dieses Bild verändert sich mit jeder Antwort — und es kommt aus einem anderen Programm als der Trainer.", 440, 280),

  H.code([
    'con = DB_OPEN("trainer.db")',
    'erg = DB_QUERY(con, "SELECT fach, COUNT(*) FROM vokabeln GROUP BY fach")',
    "WHILE DB_NEXT(erg)",
    "    inhalt[DB_GET_INT(erg, 0)] = DB_GET_INT(erg, 1)",
    "    gesamt = gesamt + DB_GET_INT(erg, 1)",
    "WEND",
    "DB_CLOSE_RESULT(erg)",
    "DB_CLOSE(con)",
  ]),

  H.pmix(["Auffällig ist das ", ["DB_CLOSE", true], " noch VOR der Schleife: Gelesen wird einmal, danach wird nur noch gezeichnet. Ein Programm, das nichts ändert, muss die Datenbank nicht offen halten — und solange sie zu ist, kann der Trainer nebenan ungestört weiterschreiben."]),

  H.p("Das ist der eigentliche Gewinn aus Kapitel 27: Die Daten liegen nicht IM Programm. Sie liegen daneben, und jedes Programm, das SQL sprechen kann, kommt an sie heran — auch eines, das es heute noch nicht gibt."),

  H.h2("Eine Schrift für alle"),

  H.p("Kapitel 29 hat gezeigt, was passiert, wenn Akzente vorkommen: Die Runtime weicht auf eine Systemschrift aus, aber nur für die betroffenen Texte. Ein Vokabeltrainer, in dem jede zweite Zeile anders aussieht, ist keiner. Also lädt er selbst eine Schrift:"),

  H.code([
    "FUNCTION schrift_holen(groesse AS INTEGER) AS INTEGER",
    "    DIM p AS STRING",
    '    FOR EACH p IN SPLIT$("C:/Windows/Fonts/segoeui.ttf|" + _',
    '                         "/System/Library/Fonts/Supplemental/" + _',
    '                         "Arial.ttf|/usr/share/fonts/truetype/" + _',
    '                         "dejavu/DejaVuSans.ttf", "|")',
    "        IF FILEEXISTS(p) THEN RETURN LOADFONT(p, groesse)",
    "    NEXT",
    "    RETURN -1",
    "END FUNCTION",
  ]),

  H.pmix(["Drei Pfade, einer je System: Windows, macOS, Linux. Der erste, den es gibt, wird genommen. Findet sich keiner, kommt ", ["-1", true], " zurück, und das Programm bleibt bei der eingebauten Schrift — es bricht nicht ab."]),

  H.note("Dieses Muster — mehrere Möglichkeiten der Reihe nach probieren und mit einer harmlosen Antwort aufhören — ist überall dort richtig, wo etwas von der Umgebung abhängt: Schriften, Ordner, Geräte. Die Alternative wäre, den Pfad fest einzutragen; dann läuft das Programm auf genau einem Rechner."),

  H.h2("Weitergeben"),

  H.p("Ein Programm, das nur läuft, wenn man vorher etwas installiert, verschenkt man nicht. Deshalb kann dhrt aus einem Programm eine eigenständige Datei machen:"),

  H.code([
    "dhrt --export trainer.dh ausgabe --mit-daten",
  ]),

  H.p("Was dabei herauskommt, wurde nachgemessen:"),

  H.table([
    [{ text: "trainer.exe", mono: true }, "15 634 681 Bytes", "das Programm samt Runtime"],
    [{ text: "trainer.db", mono: true }, "20 480 Bytes", "automatisch mitkopiert"],
    [{ text: "katalog.txt", mono: true }, "237 Bytes", "automatisch mitkopiert"],
  ], { headers: ["Datei", "Größe", "Was es ist"], widths: [2200, 2600, 4226], mono: [0] }),

  H.p("Knapp fünfzehn Megabyte für eine Datei, die auf jedem Windows-Rechner startet, ohne dass irgendetwas installiert wäre. Das ist die ganze Runtime mit Grafik, Klang und Datenbank — sie liegt in der Exe."),

  H.tip("Nachgemessen", "Die exportierte Exe in einem leeren Ordner gestartet: Sie läuft, zeigt die Vokabeln aus der mitkopierten Datenbank und schreibt „groß = big“ mit korrektem ß. Die geladene Schrift funktioniert also auch im Bundle."),

  H.warn("--mit-daten kopiert mit, was es im Quelltext FINDET: \"trainer.db\" und \"katalog.txt\" stehen als Text da. Die Vokabellisten dagegen heißen im Programm dateien[i] — der Name entsteht erst beim Laufen, und deshalb kann kein Werkzeug ihn vorher kennen. Wer sie mitgeben will, kopiert sie von Hand daneben. Automatik hat immer eine Grenze, und es lohnt sich zu wissen, wo sie liegt.", "Was --mit-daten nicht sehen kann"),

  H.h2("Was fehlt — und warum das gut ist"),

  H.p("Der Trainer kann viel und längst nicht alles. Was ihm fehlt, ist keine Liste von Versäumnissen, sondern von Aufgaben:"),

  H.bullet("Das Zuordnen aus Kapitel 31 ist nicht eingebaut — es passt nicht in den Ein-Wort-Ablauf und bräuchte einen eigenen Modus."),
  H.bullet("Es gibt keine Aussprache. Töne kann Drachenhauch, aufgenommene Sprache müsste man mitliefern."),
  H.bullet("Nichts wird zwischen Rechnern abgeglichen. Die Datenbank liegt da, wo das Programm liegt."),
  H.bullet("Die Wartezeiten sind fest verdrahtet und lernen nicht aus dem eigenen Verlauf."),
  H.bullet("Es gibt kein Rückgängig. Wer sich verklickt, hat sich verklickt."),

  H.p("Jede dieser Lücken ist ein Nachmittag Arbeit mit dem, was du kannst. Genau so soll ein Abschlussprojekt enden — nicht fertig, sondern anfassbar."),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    ["Auf „Lernen“ ist gar nichts zu sehen", "Alle Bedienelemente stehen auf VERSTECKT — der Zweig für das aktuelle Muster fehlt."],
    ["Die Antwortknöpfe bleiben nach der Antwort stehen", "Sie werden im Zweig für „geantwortet“ nicht wieder versteckt."],
    ["Dieselbe neue Vokabel kommt immer wieder", "Beim Vorstellen wird gesehen nicht erhöht — muster_zu liefert dann ewig VORSTELLEN."],
    ["Die Trefferquote ist zu niedrig", "Das Vorstellen wird als falsche Antwort gebucht."],
    ["Alle Balken stehen auf 0%", "Die Division ist ganzzahlig. Mit * 1.0 rechnen."],
    ["Ein Balken bleibt stehen, obwohl das Fach leer ist", "GROUP BY liefert leere Fächer nicht. Vorher alle auf 0 setzen."],
    ["Beim Tippen passiert nichts", "Das Eingabefeld steht auf VERSTECKT, weil das Muster nicht als Tippen erkannt wurde."],
    ["Die Umlaute sehen anders aus als der Rest", "SETFONT fehlt oder LOADFONT hat -1 geliefert."],
    ["Die exportierte Exe findet ihre Daten nicht", "Die Dateien liegen nicht neben der Exe. --mit-daten sieht nur, was im Quelltext steht."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3600, 5426] }),

  H.h2("Aufgaben"),

  H.bullet("Bau das Zuordnen aus Kapitel 31 als fünften Reiter ein — mit sechs fälligen Vokabeln aus dem Karteikasten statt sechs zufälligen."),
  H.bullet("Zeig beim Lernen an, wie viele Vokabeln gerade fällig sind. Das ist der ehrlichste Fortschrittsbalken, den es gibt."),
  H.bullet("Lass die Wartezeiten pro Liste einstellen und speicher sie in der Tabelle stand."),
  H.bullet("Ergänze einen Knopf „das wusste ich doch“, der die letzte Antwort zurücknimmt."),
  H.bullet("Färb im Fortschritt die Balken nach Fach ein, wie im Bild aus Kapitel 30."),
  H.bullet("Exportiere den Trainer und gib ihn jemandem. Sieh zu, wie er ihn benutzt, und schreib auf, wo er stockt — das ist die wertvollste Fehlerliste, die du bekommen kannst."),

  H.h2("Und jetzt?"),

  H.p("Dreiunddreißig Kapitel weit ist aus einem Fenster mit einem Kreis darin ein Programm geworden, das Daten aus dem Netz holt, sie in einer Datenbank hält, sich merkt, was du kannst, und sich weitergeben lässt."),

  H.p("Wenn du eines aus diesem Buch mitnimmst, dann bitte nicht eine Liste von Befehlen. Nimm die Arbeitsweise mit: klein anfangen, sofort ansehen, was passiert, und nichts glauben, was du nicht gemessen hast. Beim Schreiben dieses Buchs hat genau das ein Dutzend Fehler gefunden — einen Smiley, der traurig guckte, ein Rechteck statt einer Linie, eine Zahl über 255, ein „nein“, das ein „fast“ verschluckte. Jeder davon war beim Lesen unsichtbar und beim Hinsehen offensichtlich."),

  H.p("Was du jetzt kannst, reicht für erstaunlich viel: Werkzeuge für dich selbst, kleine Spiele, Programme, die Dateien sortieren oder Zahlen zeichnen. Der nächste Schritt ist kein neues Kapitel, sondern ein eigenes Vorhaben — irgendetwas, das du gern hättest und das es noch nicht gibt."),

  H.p("Bau es. Und sieh es dir an, während es entsteht."),
];
