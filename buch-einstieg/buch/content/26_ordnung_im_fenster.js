module.exports = (H) => [
  H.chapter("Ordnung im Fenster"),

  H.p("Zwei Knöpfe und ein Feld ordnen sich von selbst. Bei zwanzig Bedienelementen wird ein Fenster zur Rumpelkammer — und die Frage, was wohin gehört, wird wichtiger als die Frage, wie man einen Knopf anlegt."),

  H.p("Drachenhauch hat dafür drei Mittel, und mit denen lässt sich fast jede Oberfläche ordnen."),

  H.h2("Reiter"),

  H.p("Was nicht gleichzeitig gebraucht wird, gehört nicht gleichzeitig ins Fenster. Reiter teilen einen Bereich in mehrere Seiten:"),

  H.code([
    "DIM reiter[3] AS STRING",
    "",
    'reiter[0] = "Allgemein"',
    'reiter[1] = "Spiel"',
    'reiter[2] = "Ueber"',
    "",
    "GUI_TABS(fenster, reiter)",
  ]),

  H.p("Danach bekommt jedes Bedienelement gesagt, auf welche Seite es gehört:"),

  H.code([
    'name = GUI_TEXTINPUT(fenster, 24, 66, 300, 30, "dein Name")',
    "GUI_SET_TAB(name, 0)",
  ]),

  H.figure("kap26_1_reiter.png", "Drei Reiter, aber nur eine Seite ist zu sehen. Der Knopf unten gehört zu allen.", 440, 280),

  H.warn("Das Array hat DREI Fächer, weil es drei Reiter sind — DIM reiter[3] und die Nummern 0, 1, 2. Beim Schreiben dieses Kapitels stand dort zuerst DIM reiter[2], und das Programm brach ab mit „Index 2 ausserhalb [0..1] in Dimension 0“. Es ist genau die Falle, vor der Kapitel 9 warnt, und sie erwischt einen auch dann noch, wenn man den Warnkasten selbst geschrieben hat.", "Die Zahl ist die Anzahl"),

  H.pmix(["Bedienelemente ohne ", ["GUI_SET_TAB", true], " sind auf allen Seiten sichtbar. Das ist genau richtig für den „Fertig“-Knopf: Er gehört zum Fenster, nicht zu einer Seite."]),

  H.pmix([["GUI_ACTIVE_TAB(fenster)", true], " sagt, welcher Reiter gerade vorn ist. Damit kann das Programm reagieren — etwa nur dann rechnen, wenn die Seite überhaupt zu sehen ist."]),

  H.h2("Gruppen"),

  H.p("Innerhalb einer Seite fasst ein Rahmen zusammen, was zusammengehört:"),

  H.code([
    'gruppe = GUI_GROUPBOX(fenster, 24, 60, 300, 130, "Schwierigkeit")',
  ]),

  H.p("Der Rahmen ist reine Deko — er tut nichts, er umschließt nichts technisch. Die drei Schalter darin sind ganz normale Schalter, die zufällig innerhalb seiner Fläche liegen. Trotzdem ist der Unterschied für den Betrachter groß: Drei Kästchen mit einem Rahmen und einer Überschrift sind eine Entscheidung, drei Kästchen ohne sind drei Entscheidungen."),

  H.note("Das ist kein technischer, sondern ein gestalterischer Punkt — und einer der wenigen, die man wirklich lernen muss. Menschen lesen Nähe als Zusammengehörigkeit. Was zusammengehört, muss beieinanderstehen; was getrennt ist, braucht Abstand. Mehr Regeln braucht es für einfache Oberflächen kaum."),

  H.h2("Mitwachsen"),

  H.p("Ein Fenster, das man größer ziehen kann, wirft eine Frage auf: Was passiert dann mit den Bedienelementen? Die Vorgabe ist, dass sie oben links kleben bleiben — und unten rechts entsteht eine wachsende leere Fläche."),

  H.code([
    "GUI_WINDOW_RESIZABLE(fenster, TRUE)",
  ]),

  H.code([
    'GUI_SET_ANCHOR(fertig, "lb")',
  ]),

  H.pmix([["GUI_SET_ANCHOR", true], " sagt, an welchen Kanten ein Element klebt: ", ['"l"', true], " links, ", ['"r"', true], " rechts, ", ['"t"', true], " oben, ", ['"b"', true], " unten. ", ['"lb"', true], " heißt also links und unten — der Knopf bleibt beim Vergrößern unten links sitzen, statt in der Mitte zu hängen."]),

  H.p("Und wenn ein Element an zwei gegenüberliegenden Kanten klebt, DEHNT es sich: Eine Liste mit „lr“ wird beim Verbreitern des Fensters breiter. Das ist meist genau, was man will — und man muss nichts dafür ausrechnen."),

  H.h2("Eine Faustregel für den Aufbau"),

  H.p("Wer nicht weiß, wo er anfangen soll, kommt mit dieser Reihenfolge weit:"),

  H.bullet("Oben, was man zuerst tut (eingeben, suchen)."),
  H.bullet("In der Mitte, worum es geht (die Liste, der Inhalt)."),
  H.bullet("Unten, was abschließt (Fertig, Speichern, Schließen)."),
  H.bullet("Rechts oder daneben, was sich auf das Gewählte bezieht (Löschen, Bearbeiten)."),
  H.bullet("Was selten gebraucht wird, auf einen eigenen Reiter."),

  H.p("Das ist keine Kunst, sondern die Anordnung, die fast alle Programme benutzen — und deshalb die, die niemand erklären muss."),

  H.h2("Die Ordnungsmittel im Überblick"),

  H.table([
    [{ text: "GUI_TABS(win, beschriftungen)", mono: true }, "Reiterleiste anlegen"],
    [{ text: "GUI_SET_TAB(w, seite)", mono: true }, "Element einem Reiter zuordnen (-1 = überall)"],
    [{ text: "GUI_ACTIVE_TAB(win)", mono: true }, "welcher Reiter ist vorn"],
    [{ text: "GUI_GROUPBOX(win, x, y, b, h, titel)", mono: true }, "gerahmte Gruppe"],
    [{ text: "GUI_SEPARATOR(win, x, y, b)", mono: true }, "Trennlinie"],
    [{ text: "GUI_WINDOW_RESIZABLE(win, an)", mono: true }, "Fenster größenveränderbar"],
    [{ text: 'GUI_SET_ANCHOR(w, "lrtb")', mono: true }, "an welchen Kanten das Element klebt"],
    [{ text: "GUI_WINDOW_SCROLLABLE(win, an)", mono: true }, "Inhalt scrollt, wenn er nicht passt"],
  ], { headers: ["Aufruf", "Was er tut"], widths: [4200, 4826], mono: [0] }),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    [{ text: "Index ausserhalb [0..1]", mono: true }, "Das Array für die Reiter hat ein Fach zu wenig. Drei Reiter brauchen DIM r[3]."],
    ["Alle Bedienelemente sind immer sichtbar", "GUI_SET_TAB fehlt. Ohne Zuordnung gehört ein Element zu allen Seiten."],
    ["Ein Element verschwindet beim Verkleinern", "Es wird am Fensterrand abgeschnitten. Anker setzen oder eine Mindestgröße."],
    ["Der Knopf hängt beim Vergrößern in der Luft", "Ohne Anker klebt alles oben links."],
    ["Der Rahmen der Gruppe liegt über den Schaltern", "Die Gruppe wurde nach ihnen angelegt. Erst den Rahmen, dann den Inhalt."],
    ["Der Reiterwechsel tut nichts", "Alle Elemente hängen auf Seite 0 — oder auf gar keiner."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3600, 5426] }),

  H.h2("Aufgaben"),

  H.bullet("Füll den dritten Reiter „Ueber“ mit ein paar Zeilen Text über dein Programm."),
  H.bullet("Sorg dafür, dass die drei Schwierigkeitsschalter sich gegenseitig ausschließen — klickt man einen an, gehen die anderen aus."),
  H.bullet("Gib dem Einkaufszettel aus Kapitel 25 einen zweiten Reiter mit den Einstellungen."),
  H.bullet("Mach die Liste im Einkaufszettel mitwachsend, indem du sie an alle vier Kanten ankerst."),
  H.bullet("Setz eine Mindestgröße für das Fenster, damit man es nicht kleiner ziehen kann als seinen Inhalt."),
  H.bullet("Zeig unten immer an, welcher Reiter gerade vorn ist — und überleg dir, wofür das in einem echten Programm gut wäre."),

  H.p("Deine Oberfläche ist jetzt geordnet. Was noch fehlt, damit sie eine Anwendung wird: Die Daten müssen irgendwo hin, wo sie bleiben — und diesmal nicht in eine Textdatei."),
];
