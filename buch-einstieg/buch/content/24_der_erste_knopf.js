module.exports = (H) => [
  H.part("Teil V — Fenster mit Knöpfen"),
  H.chapter("Der erste Knopf"),

  H.p("Alles, was du bisher gebaut hast, wurde mit Tasten bedient. Für Spiele ist das genau richtig. Für ein Programm, das Vokabeln verwaltet, wäre es eine Zumutung."),

  H.p("Was fehlt, sind Knöpfe, Eingabefelder und Listen — die Dinge, die jedes normale Programm hat. Drachenhauch bringt sie mit, und der Einstieg ist überraschend kurz."),

  H.h2("Ein Fenster mit einem Knopf"),

  H.code([
    'IMPORT "gui"',
    "",
    'SCREEN(640, 400, "Der erste Knopf")',
    "",
    "DIM fenster AS GUI_WINDOW",
    "DIM knopf AS GUI_WIDGET",
    "DIM anzeige AS GUI_WIDGET",
    "DIM zaehler AS INTEGER",
    "",
    'fenster = GUI_WINDOW("Zaehler", 120, 90, 400, 200)',
    'anzeige = GUI_LABEL(fenster, "noch nicht gedrueckt", 24, 40)',
    'knopf = GUI_BUTTON(fenster, "Druecken", 24, 90, 150, 36)',
    "zaehler = 0",
    "",
    "WHILE NOT QUITREQUESTED() AND NOT KEYPRESSED(KEY_ESCAPE)",
    "    CLS(RGB(28, 32, 50))",
    "    GUI_UPDATE()",
    "",
    "    IF GUI_CLICKED(knopf) THEN",
    "        zaehler = zaehler + 1",
    '        GUI_SET_TEXT(anzeige, "gedrueckt: " + STR$(zaehler))',
    "    END IF",
    "",
    "    GUI_DRAW()",
    "    FLIP()",
    "WEND",
  ]),

  H.figure("kap24_1_erster_knopf.png", "Ein Fenster mit Titelleiste, einer Beschriftung und einem Knopf. Das Fenster lässt sich mit der Maus verschieben.", 440, 280),

  H.h2("Zeile für Zeile"),

  H.pmix([['IMPORT "gui"', true], " ist diesmal wirklich nötig, und der Grund ist interessant. Die Klangbefehle aus Kapitel 10 liefen ohne Import, weil sie fest eingebaut sind. Hier geht es aber nicht nur um Befehle, sondern um SORTEN: ", ["GUI_WINDOW", true], " und ", ["GUI_WIDGET", true], " sind neue Kartonsorten, und die kommen aus dem Modul. Ohne die Zeile heißt es „Unbekannter Typ 'gui_window' -- fehlt IMPORT \"gui\"?“ — die Meldung sagt dir sogar, was zu tun ist."]),

  H.pmix([["GUI_WINDOW(titel, x, y, breite, hoehe)", true], " legt ein Fenster IM Fenster an. Das äußere ist das aus ", ["SCREEN", true], "; das innere ist eines, das man verschieben kann, das eine Titelleiste hat und in dem die Bedienelemente sitzen."]),

  H.pmix([["GUI_LABEL", true], " und ", ["GUI_BUTTON", true], " setzen Beschriftung und Knopf hinein. Ihre Stellen sind RELATIV zum Fenster: 24, 40 heißt 24 Punkte vom linken Rand des Fensters, nicht des Bildschirms. Verschiebst du das Fenster, wandert alles mit."]),

  H.h2("Einmal bauen, oft benutzen"),

  H.p("Das Entscheidende steht nicht in einer einzelnen Zeile, sondern in der Anordnung: Die Oberfläche wird VOR der Schleife gebaut, nicht darin."),

  H.p("Das ist ein anderer Gedanke als alles bisher. Bei Kreisen und Sprites hast du in jedem Bild alles neu gemalt. Ein Knopf dagegen ist ein Ding, das dauerhaft existiert — es wird einmal angelegt und bleibt dann. In der Schleife stehen nur noch drei Dinge:"),

  H.bulletRich("GUI_UPDATE() ", "— die Oberfläche sieht nach, was die Maus und die Tastatur gemacht haben."),
  H.bulletRich("Fragen stellen ", "— IF GUI_CLICKED(knopf) THEN. Das ist Polling: Man fragt in jedem Bild nach, ob etwas passiert ist."),
  H.bulletRich("GUI_DRAW() ", "— alles zeichnen."),

  H.warn("GUI_UPDATE gehört VOR die Fragen, GUI_DRAW dahinter. Steht GUI_UPDATE zu spät, hinkt die Bedienung ein Bild hinterher; steht GUI_DRAW zu früh, siehst du den Zustand von vorhin. Die Reihenfolge ist dieselbe wie bei der Spielschleife aus Kapitel 5 — nur heißt sie hier anders.", "Erst hören, dann fragen, dann malen"),

  H.pmix([["GUI_SET_TEXT", true], " ändert die Beschriftung. Auch das ist neu: Der Text wird nicht jedes Bild neu hingeschrieben, sondern EINMAL geändert, wenn sich etwas tut. Danach zeigt das Label ihn von selbst weiter."]),

  H.h2("Schieber und Schalter"),

  H.p("Ein Knopf löst etwas aus. Ein Schieber und ein Schalter halten dagegen einen Zustand, den man abfragt — und damit lässt sich der Farbmischer aus Kapitel 2 endlich richtig bauen:"),

  H.code([
    'fenster = GUI_WINDOW("Farbmischer", 20, 20, 300, 230)',
    'GUI_LABEL(fenster, "rot", 20, 34)',
    "rot = GUI_SLIDER(fenster, 90, 34, 170, 0, 255, 200)",
    'GUI_LABEL(fenster, "gruen", 20, 74)',
    "gruen = GUI_SLIDER(fenster, 90, 74, 170, 0, 255, 120)",
    'GUI_LABEL(fenster, "blau", 20, 114)',
    "blau = GUI_SLIDER(fenster, 90, 114, 170, 0, 255, 60)",
    'rahmen = GUI_CHECKBOX(fenster, "mit Rahmen", 20, 154, TRUE)',
  ]),

  H.code([
    "r = INT(GUI_VALUE(rot))",
    "g = INT(GUI_VALUE(gruen))",
    "b = INT(GUI_VALUE(blau))",
    "",
    "BOX(360, 60, 600, 300, RGB(r, g, b))",
    "IF GUI_CHECKED(rahmen) THEN",
    "    RECT(356, 56, 604, 304, RGB(240, 240, 250))",
    "END IF",
  ]),

  H.figure("kap24_2_schieber.png", "Drei Schieber, ein Schalter — und daneben das Ergebnis. In Kapitel 2 musste man dafür noch Zahlen im Quelltext ändern.", 440, 280),

  H.pmix([["GUI_SLIDER(fenster, x, y, breite, von, bis, start)", true], " legt einen Schieber an. ", ["GUI_VALUE", true], " liest seinen Wert — als Kommazahl, deshalb steht ", ["INT", true], " darum herum, denn ", ["RGB", true], " will ganze Zahlen. Genau dieselbe Falle wie in Kapitel 10."]),

  H.pmix([["GUI_CHECKBOX", true], " liefert einen Schalter, ", ["GUI_CHECKED", true], " fragt ihn ab. Weil das Ergebnis schon ein Ja oder Nein ist, steht es direkt im ", ["IF", true], " — ohne Vergleich, wie beim BOOLEAN aus Kapitel 7."]),

  H.p("Vergleich das einen Moment mit Kapitel 2. Dort stand die Farbe in drei Zahlen im Quelltext, und wer sie ändern wollte, musste das Programm bearbeiten und neu starten. Hier zieht man an einem Schieber. Es ist dasselbe Programm — nur bedienbar."),

  H.h2("Was es sonst noch gibt"),

  H.table([
    [{ text: "GUI_LABEL(win, text, x, y)", mono: true }, "Beschriftung"],
    [{ text: "GUI_BUTTON(win, text, x, y, b, h)", mono: true }, "Knopf"],
    [{ text: "GUI_CHECKBOX(win, text, x, y[, start])", mono: true }, "Schalter"],
    [{ text: "GUI_SLIDER(win, x, y, b, von, bis[, start])", mono: true }, "Schieber"],
    [{ text: "GUI_SPINNER(win, x, y, b, von, bis[, start])", mono: true }, "Zahlenfeld mit Plus und Minus"],
    [{ text: "GUI_SEPARATOR(win, x, y, b)", mono: true }, "Trennlinie"],
    [{ text: "GUI_CLICKED(w)", mono: true }, "wurde geklickt?"],
    [{ text: "GUI_VALUE(w)", mono: true }, "Wert eines Schiebers oder Zahlenfelds"],
    [{ text: "GUI_CHECKED(w)", mono: true }, "Zustand eines Schalters"],
    [{ text: "GUI_SET_TEXT(w, text)", mono: true }, "Beschriftung ändern"],
  ], { headers: ["Aufruf", "Was er tut"], widths: [4200, 4826], mono: [0] }),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    [{ text: "Unbekannter Typ 'gui_window'", mono: true }, "Die Zeile IMPORT \"gui\" fehlt ganz oben."],
    ["Der Knopf reagiert nicht", "GUI_UPDATE fehlt in der Schleife, oder es steht hinter der Abfrage."],
    ["Es ist nichts zu sehen", "GUI_DRAW fehlt — oder es steht vor dem CLS und wird gleich wieder übermalt."],
    ["Der Zähler springt um mehrere", "GUI_CLICKED wird mehrfach je Bild abgefragt. Einmal fragen, Ergebnis merken."],
    ["Die Oberfläche wird immer wieder neu gebaut", "GUI_WINDOW oder GUI_BUTTON stehen in der Schleife. Sie gehören davor."],
    [{ text: "RGB erwartet INTEGER", mono: true }, "GUI_VALUE liefert eine Kommazahl. INT darum herum."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3600, 5426] }),

  H.h2("Aufgaben"),

  H.bullet("Gib dem Zähler einen zweiten Knopf, der ihn wieder auf null setzt."),
  H.bullet("Bau einen dritten Knopf, der rückwärts zählt — und sorg dafür, dass der Zähler nicht unter null geht."),
  H.bullet("Zeig im Farbmischer die drei Zahlen als Text neben den Schiebern an."),
  H.bullet("Ergänze einen Schieber für die Größe des Farbfelds."),
  H.bullet("Bau das Instrument aus Kapitel 12 so um, dass die Klangform über Knöpfe statt über Zifferntasten gewählt wird."),
  H.bullet("Leg zwei Fenster nebeneinander an und schieb sie mit der Maus umher. Beobachte, was passiert, wenn sie sich überlappen."),

  H.p("Knöpfe lösen etwas aus, Schieber halten einen Wert. Was noch fehlt, damit man Vokabeln eingeben kann: ein Feld, in das man schreibt."),
];
