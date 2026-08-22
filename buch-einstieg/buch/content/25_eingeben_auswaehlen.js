module.exports = (H) => [
  H.chapter("Eingeben und auswählen"),

  H.p("Ein Knopf kann nur eines: gedrückt werden. Für ein Programm, das Vokabeln verwaltet, braucht es mehr — ein Feld zum Schreiben, eine Liste zum Auswählen, und die Möglichkeit, etwas wieder loszuwerden."),

  H.p("Das Programm dieses Kapitels ist ein Einkaufszettel. Es ist Zeile für Zeile das Gerüst, auf dem am Ende des Buchs der Vokabeltrainer steht."),

  H.h2("Ein Feld zum Schreiben"),

  H.code([
    'feld = GUI_TEXTINPUT(fenster, 20, 36, 280, 30, "was fehlt?")',
  ]),

  H.pmix(["Der letzte Text ist der Platzhalter: Er steht blass im leeren Feld und verschwindet, sobald jemand tippt. Das ist keine Verzierung, sondern spart eine Beschriftung daneben — und sagt in denselben Worten, was hineingehört."]),

  H.pmix(["Was drinsteht, holt ", ["GUI_TEXT(feld)", true], ". Und ", ["GUI_SET_TEXT(feld, \"\")", true], " leert es wieder — das braucht man nach jedem Hinzufügen, sonst muss der Benutzer selbst löschen."]),

  H.h2("Eine Liste"),

  H.code([
    "DIM eintraege[0] AS STRING",
    "",
    'ARRAY_PUSH(eintraege, "Milch")',
    'ARRAY_PUSH(eintraege, "Brot")',
    'ARRAY_PUSH(eintraege, "Aepfel")',
    "",
    "liste = GUI_LISTBOX(fenster, 20, 84, 280, 150, eintraege)",
  ]),

  H.figure("kap25_1_liste_fuellen.png", "Feld, Knopf, Liste, Zähler. Mehr braucht eine Verwaltung nicht, um eine zu sein.", 440, 280),

  H.p("Hier zeigt sich etwas Wichtiges über den Umgang mit Oberflächen: Die Liste im Fenster ist nicht die Wahrheit. Die Wahrheit steht im Array."),

  H.pmix(["Das Array ", ["eintraege", true], " hält die Daten. Die Liste zeigt sie nur an. Ändert sich das Array, muss man der Liste Bescheid sagen — mit ", ["GUI_SET_LISTBOX(liste, eintraege)", true], "."]),

  H.warn("Diese Trennung ist die wichtigste Gewohnheit dieses Teils. Wer versucht, seine Daten IN der Oberfläche zu halten, bekommt später Ärger: Man kann eine Liste nicht sortieren, nicht durchsuchen, nicht speichern. Ein Array kann das alles — und die Liste ist nur das Fenster darauf. In großen Programmen nennt man das die Trennung von Daten und Darstellung, und sie fängt genau hier an, mit diesen zwei Zeilen.", "Das Array ist die Wahrheit"),

  H.h2("Hinzufügen"),

  H.code([
    "IF GUI_CLICKED(dazu) THEN",
    "    was = TRIM$(GUI_TEXT(feld))",
    '    IF was <> "" THEN',
    "        ARRAY_PUSH(eintraege, was)",
    "        GUI_SET_LISTBOX(liste, eintraege)",
    '        GUI_SET_TEXT(feld, "")',
    "    END IF",
    "END IF",
  ]),

  H.p("Vier Schritte in der richtigen Reihenfolge: holen, prüfen, ins Array, Liste auffrischen, Feld leeren."),

  H.pmix(["Das ", ["TRIM$", true], " aus Kapitel 17 ist hier kein Luxus. Wer versehentlich ein Leerzeichen tippt, hätte sonst einen Eintrag, der aussieht wie „Milch “ und sich nie mit „Milch“ vergleichen lässt. Und die Prüfung auf den leeren Text verhindert, dass ein Klick auf den leeren Knopf eine leere Zeile anlegt."]),

  H.p("Beides sind zwei Zeilen, die niemand vermisst, solange man sein Programm selbst bedient — und die jeder vermisst, sobald es jemand anderes tut."),

  H.h2("Löschen"),

  H.code([
    "IF GUI_CLICKED(weg) THEN",
    "    nummer = GUI_LISTBOX_SELECTED(liste)",
    "    IF nummer >= 0 THEN",
    "        ARRAY_REMOVE_AT(eintraege, nummer)",
    "        GUI_SET_LISTBOX(liste, eintraege)",
    "    END IF",
    "END IF",
  ]),

  H.pmix([["GUI_LISTBOX_SELECTED", true], " liefert die Nummer der gewählten Zeile — oder ", ["-1", true], ", wenn nichts gewählt ist. Dieselbe ", ["-1", true], " wie bei ", ["INSTR", true], " in Kapitel 17, und sie muss genauso abgefangen werden: Ohne die Prüfung würde ein Klick auf „Löschen“ ohne Auswahl das Programm beenden."]),

  H.pmix([["ARRAY_REMOVE_AT(eintraege, nummer)", true], " nimmt den Eintrag aus dem Array. Alles dahinter rückt auf — genau wie beim Nachrücken der Schlange, nur macht es Drachenhauch hier selbst."]),

  H.h2("Ein Auswahlfeld"),

  H.p("Wenn es nur wenige feste Möglichkeiten gibt, ist eine Liste zu groß. Dann nimmt man ein Auswahlfeld, das erst beim Anklicken aufklappt:"),

  H.code([
    "DIM formen[3] AS STRING",
    "",
    'formen[0] = "Kreis"',
    'formen[1] = "Quadrat"',
    'formen[2] = "Ring"',
    "",
    "wahl = GUI_DROPDOWN(fenster, 90, 36, 140, 28, formen)",
  ]),

  H.code([
    "was = GUI_DROPDOWN_SELECTED(wahl)",
    "IF was = 0 THEN CIRCLE(450, 200, 90, farbe)",
    "IF was = 1 THEN BOX(360, 110, 540, 290, farbe)",
    "IF was = 2 THEN CIRCLEOUTLINE(450, 200, 90, farbe)",
  ]),

  H.figure("kap25_2_auswahl.png", "Das Auswahlfeld bestimmt, was daneben gemalt wird.", 440, 280),

  H.pmix([["GUI_DROPDOWN_SELECTED", true], " liefert die Nummer, ", ["GUI_DROPDOWN_TEXT", true], " den gewählten Text. Meist ist die Nummer bequemer — sie lässt sich vergleichen, ohne dass man sich beim Abtippen des Textes vertun kann."]),

  H.note("Die Bedienelemente werden hier zwischen Zeichenbefehle gemischt: BOX und CIRCLE malen direkt auf den Bildschirm, GUI_DRAW malt danach die Oberfläche darüber. Deshalb liegt das Fenster immer vorn. Wer es andersherum will, ruft GUI_DRAW vor den eigenen Malbefehlen auf."),

  H.h2("Die Bedienelemente im Überblick"),

  H.table([
    [{ text: "GUI_TEXTINPUT(win, x, y, b, h[, platzhalter])", mono: true }, "einzeiliges Eingabefeld"],
    [{ text: "GUI_TEXTAREA(win, x, y, b, h[, platzhalter])", mono: true }, "mehrzeiliges Feld"],
    [{ text: "GUI_LISTBOX(win, x, y, b, h, eintraege)", mono: true }, "Liste zum Auswählen"],
    [{ text: "GUI_DROPDOWN(win, x, y, b, h, eintraege)", mono: true }, "aufklappendes Auswahlfeld"],
    [{ text: "GUI_TEXT(w)", mono: true }, "Inhalt eines Eingabefelds"],
    [{ text: "GUI_LISTBOX_SELECTED(l)", mono: true }, "gewählte Zeile, oder -1"],
    [{ text: "GUI_LISTBOX_TEXT(l)", mono: true }, "Text der gewählten Zeile"],
    [{ text: "GUI_SET_LISTBOX(l, eintraege)", mono: true }, "Liste neu füllen"],
    [{ text: "GUI_DROPDOWN_SELECTED(d)", mono: true }, "gewählte Nummer"],
  ], { headers: ["Aufruf", "Was er tut"], widths: [4400, 4626], mono: [0] }),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    ["Die Liste bleibt leer", "GUI_SET_LISTBOX fehlt nach dem Ändern des Arrays. Die Liste merkt es nicht von selbst."],
    ["Das Programm bricht beim Löschen ab", "Es war nichts ausgewählt, und die Prüfung auf -1 fehlt."],
    ["Leere Zeilen in der Liste", "Die Prüfung auf leeren Text fehlt beim Hinzufügen."],
    ["Ein Eintrag wird nicht gefunden", "Führende oder folgende Leerzeichen. TRIM$ beim Eingeben."],
    ["Nach dem Hinzufügen steht der Text noch im Feld", "GUI_SET_TEXT(feld, \"\") fehlt."],
    ["Die Liste zeigt Altes", "Es wurde ein anderes Array geändert als das, aus dem die Liste gefüllt wird."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3600, 5426] }),

  H.h2("Aufgaben"),

  H.bullet("Sorg dafür, dass ein Eintrag nicht zweimal in die Liste kommt. ARRAY_INDEXOF aus Kapitel 14 sagt dir, ob er schon da ist."),
  H.bullet("Füge einen Knopf hinzu, der die Liste alphabetisch sortiert."),
  H.bullet("Lass die Eingabe auch auf die Eingabetaste reagieren, nicht nur auf den Knopf."),
  H.bullet("Speichere die Liste beim Beenden in eine Datei und lies sie beim Start wieder ein — mit den Befehlen aus Kapitel 18."),
  H.bullet("Bau ein zweites Eingabefeld dazu, so dass jeder Eintrag aus zwei Teilen besteht. Zeig sie in der Liste als „Teil eins — Teil zwei“ an."),
  H.bullet("Ergänze ein Auswahlfeld für die Abteilung (Obst, Backwaren, Sonstiges) und zeig sie vor jedem Eintrag an."),

  H.p("Die letzte Aufgabe ist übrigens keine Übung, sondern eine Ankündigung: Zwei Teile je Eintrag — das ist eine Vokabel."),
];
