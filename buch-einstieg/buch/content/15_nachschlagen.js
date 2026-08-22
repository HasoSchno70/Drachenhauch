module.exports = (H) => [
  H.chapter("Nachschlagen"),

  H.p("Ein Array hat nummerierte Fächer. Das ist genau richtig, solange die Dinge eine natürliche Reihenfolge haben — die Glieder einer Schlange, fünfhundert Funken, zwölf Punktzahlen."),

  H.p("Oft will man aber nach etwas anderem suchen als nach einer Nummer. Wie viele Punkte hat Anna? Welche Farbe gehört zu „Himmel“? Wie oft wurde die Leertaste gedrückt? Für solche Fragen gibt es einen zweiten Behälter."),

  H.h2("Ein Register"),

  H.code([
    "DIM punkte AS MAP OF INTEGER",
    "",
    'MAPPUT(punkte, "Anna", 12)',
    'MAPPUT(punkte, "Bert", 7)',
    "",
    'PRINT MAPGET(punkte, "Anna")',
  ]),

  H.code(["12"], { out: true }),

  H.pmix(["So etwas heißt eine Map — ein Wörterbuch, ein Register. Statt Fach Nummer 3 gibt es den Eintrag ", ['"Anna"', true], ". Der Name heißt Schlüssel, das Gespeicherte heißt Wert."]),

  H.pmix([["DIM punkte AS MAP OF INTEGER", true], " legt sie an: eine Map, deren Werte ganze Zahlen sind. Die Schlüssel sind immer Text."]),

  H.pmix([["MAPPUT", true], " legt etwas ab, ", ["MAPGET", true], " holt es. Steht unter diesem Schlüssel schon etwas, wird es überschrieben — eine Map hat jeden Schlüssel höchstens einmal."]),

  H.p("Der Unterschied zum Array ist grundsätzlich. Bei einem Array musst du wissen, WO etwas liegt. Bei einer Map musst du wissen, WIE es heißt — und die Map kümmert sich darum, es zu finden."),

  H.h2("Der wichtigste Kasten dieses Kapitels"),

  H.warn("MAPGET auf einen Schlüssel, den es nicht gibt, bricht das Programm ab: „MAPGET: Schluessel 'gibtsnicht' nicht in Map“. Das ist kein Schönheitsfehler, sondern Absicht — die Map kann ja nichts erfinden. Es gibt zwei saubere Wege: vorher mit MAPHAS fragen, oder gleich MAPGETOR nehmen, das einen Ersatzwert mitbekommt.", "Was nicht da ist, gibt es nicht"),

  H.code([
    'PRINT MAPHAS(punkte, "Cilly")',
    'PRINT MAPGETOR(punkte, "Cilly", 0)',
  ]),

  H.code(["FALSE", "0"], { out: true }),

  H.h2("Alle Einträge durchgehen"),

  H.p("Eine Map hat keine Reihenfolge, aber sie kann dir alle ihre Schlüssel als Array geben — und ein Array kann man durchgehen und sortieren:"),

  H.code([
    "DIM namen AS ARRAY OF STRING",
    "",
    "namen = MAPKEYS(farben)",
    "SORT(namen)",
    "",
    "FOR i = 0 TO LEN(namen) - 1",
    "    y = 70 + i * 50",
    "    BOX(40, y, 160, y + 36, MAPGET(farben, namen[i]))",
    "    TEXT(180, y + 8, namen[i], schrift)",
    "NEXT",
  ]),

  H.figure("kap15_1_farbregister.png", "Sechs Einträge, alphabetisch. Die Reihenfolge kommt vom SORT, nicht von der Map.", 440, 280),

  H.pmix([["MAPKEYS", true], " liefert alle Schlüssel als Array aus Text. Das ", ["SORT", true], " danach ist wichtig: Eine Map merkt sich nicht, in welcher Reihenfolge etwas hineingelegt wurde. Ohne Sortieren stünde die Liste in einer Reihenfolge, die niemand vorhersagen kann — und die sich beim nächsten Start ändern darf."]),

  H.pmix(["Im Programm liegen Farben in der Map: ", ['MAPPUT(farben, "Feuer", RGB(240, 90, 50))', true], ". Dass das geht, liegt daran, dass ", ["RGB", true], " eine ganze Zahl liefert — eine Farbe IST eine Zahl, sie sieht nur nicht so aus."]),

  H.pmix([["MAPSIZE", true], " sagt, wie viele Einträge drin sind, ", ["MAPREMOVE", true], " wirft einen hinaus."]),

  H.h2("Zählen, ohne vorher zu wissen was"),

  H.p("Jetzt der Fall, für den Maps erfunden wurden. Ein Programm soll zählen, welche Tasten gedrückt werden — aber es weiß vorher nicht, welche das sein werden. Mit einem Array ginge das nur, indem man für jede denkbare Taste ein Fach anlegt."),

  H.code([
    "code = KEY_ANY_HIT()",
    "IF code >= 0 THEN",
    "    name = KEY_NAME$(code)",
    "    MAPPUT(zaehler, name, MAPGETOR(zaehler, name, 0) + 1)",
    "END IF",
  ]),

  H.p("Diese vierte Zeile ist die wichtigste des Kapitels. Man liest sie von innen nach außen:"),

  H.bulletRich("MAPGETOR(zaehler, name, 0) ", "— hol den bisherigen Stand für diese Taste, und wenn es noch keinen gibt, nimm null."),
  H.bulletRich("+ 1 ", "— eins dazu."),
  H.bulletRich("MAPPUT(zaehler, name, ...) ", "— und leg das Ergebnis wieder unter demselben Namen ab."),

  H.p("Damit legt sich die Map von selbst an. Beim ersten Druck auf eine Taste entsteht ihr Eintrag; bei jedem weiteren wächst er. Es gibt keine Vorbereitung, keine Liste möglicher Tasten, kein Nachdenken darüber, wie viele es werden könnten."),

  H.pmix([["KEY_ANY_HIT()", true], " ist neu: Es liefert den Code der zuletzt gedrückten Taste, oder ", ["-1", true], ", wenn gerade keine gedrückt wurde. ", ["KEY_NAME$", true], " macht daraus einen lesbaren Namen — aus der Leertaste wird ", ['"LEER"', true], ", aus dem linken Pfeil ", ['"LINKS"', true], "."]),

  H.figure("kap15_2_tastenzaehler.png", "Beim Start ist die Map leer, also gibt es nichts zu zeichnen. Drück ein paar Tasten, und die Balken wachsen.", 440, 280),

  H.p("Das Bild zeigt den leeren Anfang — dieses Programm musst du selbst bedienen. Tipp ein paar Sekunden lang wahllos herum, und du hast eine Auswertung deiner eigenen Tipperei."),

  H.h2("Wann Array, wann Map?"),

  H.table([
    ["Die Dinge haben eine Reihenfolge", "Array", "Glieder einer Schlange, Funken, Bilder einer Animation"],
    ["Es sind bekannt viele", "Array", "acht Tasten eines Instruments, zwölf Punktzahlen"],
    ["Du suchst über einen Namen", "Map", "Punkte je Spieler, Farbe je Bezeichnung, Übersetzung je Wort"],
    ["Du weißt vorher nicht, was vorkommt", "Map", "zählen, was auftaucht"],
    ["Es soll keine Doppelten geben", "Map", "jeder Schlüssel existiert höchstens einmal"],
  ], { headers: ["Woran du es erkennst", "Nimm", "Beispiele"], widths: [3000, 1000, 5026] }),

  H.p("Und oft nimmt man beides: In diesem Kapitel hält eine Map die Zahlen, und ein Array aus MAPKEYS bringt sie in eine Reihenfolge zum Anzeigen."),

  H.h2("Die Befehle im Überblick"),

  H.table([
    [{ text: "MAPPUT(m, k, w)", mono: true }, "ablegen oder überschreiben"],
    [{ text: "MAPGET(m, k)", mono: true }, "holen — bricht ab, wenn es den Schlüssel nicht gibt"],
    [{ text: "MAPGETOR(m, k, ersatz)", mono: true }, "holen, mit Ersatzwert für Unbekanntes"],
    [{ text: "MAPHAS(m, k)", mono: true }, "gibt es diesen Schlüssel?"],
    [{ text: "MAPSIZE(m)", mono: true }, "wie viele Einträge"],
    [{ text: "MAPKEYS(m)", mono: true }, "alle Schlüssel als Array aus Text"],
    [{ text: "MAPVALUES(m)", mono: true }, "alle Werte als Array"],
    [{ text: "MAPREMOVE(m, k)", mono: true }, "einen Eintrag entfernen"],
    [{ text: "MAPCLEAR(m)", mono: true }, "alles entfernen"],
  ], { headers: ["Aufruf", "Was er tut"], widths: [3000, 6026], mono: [0] }),

  H.note("Der Schlüssel ist immer Text. Willst du nach Zahlen nachschlagen, machst du mit STR$ Text daraus: MAPPUT(m, STR$(42), ...). Das klingt umständlich und ist in der Praxis völlig unauffällig."),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    [{ text: "MAPGET: Schluessel nicht in Map", mono: true }, "Der Eintrag existiert nicht. MAPHAS davor oder MAPGETOR statt MAPGET."],
    [{ text: "Erwartet OF nach MAP", mono: true }, "Es heißt DIM m AS MAP OF INTEGER — die Sorte der Werte gehört dazu."],
    ["Die Reihenfolge ändert sich bei jedem Start", "Kein Fehler: Eine Map hat keine. Schlüssel holen und sortieren."],
    ["Ein Eintrag ist verschwunden", "Zweimal MAPPUT mit demselben Schlüssel — der zweite hat den ersten überschrieben."],
    ["Der Zähler bleibt bei 1", "Statt MAPGETOR(...) + 1 wird eine feste 1 abgelegt."],
    ["Nichts wird angezeigt", "Die Map ist noch leer. Bei MAPKEYS kommt dann ein Array der Länge null — die Schleife läuft kein einziges Mal."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3600, 5426] }),

  H.h2("Aufgaben"),

  H.bullet("Erweitere das Farbregister um vier eigene Farben. Achte darauf, dass sie sich im Bild noch unterscheiden lassen."),
  H.bullet("Lass den Tastenzähler die häufigste Taste hervorheben — anderer Farbe, oder ein Stern darüber."),
  H.bullet("Bau eine Map, die Notennamen auf Frequenzen abbildet, und spiel das Instrument aus Kapitel 12 damit."),
  H.bullet("Zähl im Spiel des Lebens, wie oft jede Nachbarzahl von 0 bis 8 vorkommt, und zeig es als Balken."),
  H.bullet("Bau eine kleine Vokabelliste: deutsche Wörter als Schlüssel, englische als Werte. Zeig sie als zweispaltige Tabelle an — das ist der erste Schritt zum Abschlussprojekt dieses Buchs."),
  H.bullet("Lass den Tastenzähler nach Häufigkeit sortieren statt alphabetisch. Das ist kniffliger, als es aussieht — überleg dir, warum."),

  H.p("Arrays halten viele gleiche Dinge, Maps finden Dinge über einen Namen. Was beiden fehlt: Ein Ding mit mehreren Eigenschaften zusammenzuhalten. Genau darum geht es als Nächstes."),
];
