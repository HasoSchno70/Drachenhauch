module.exports = (H) => [
  H.chapter("Vier Arten zu fragen"),

  H.p("„Was heißt Haus?“ — „house.“ Fertig. So funktionieren die meisten Vokabelprogramme, und deshalb macht man sie nach drei Tagen nicht mehr auf."),

  H.p("Dabei ist Abfragen nicht eine Sache, sondern mehrere. Ein Wort wiederzuerkennen ist etwas ganz anderes, als es aus dem Nichts hinzuschreiben. Das eine kann man, lange bevor man das andere kann. Ein Trainer, der beides gleich behandelt, ist am Anfang zu streng und am Ende zu leicht."),

  H.p("Dieses Kapitel baut drei Programme, jedes mit einer eigenen Art zu fragen. In Kapitel 33 entscheidet dann das Fach aus dem Karteikasten, welche davon dran ist."),

  H.h2("Erstens: vier Antworten"),

  H.p("Die leichteste Form. Das Wort steht oben, vier Übersetzungen darunter, eine stimmt. Das ist Wiedererkennen — genau das Richtige für ein Wort, das man gerade erst gesehen hat."),

  H.figure("kap31_1_vier_antworten.png", "Drei Ablenker kommen aus derselben Liste. Tasten 1 bis 4 wählen aus.", 440, 300),

  H.p("Der interessante Teil ist, woher die drei falschen Antworten kommen:"),

  H.code([
    "FOR i = 1 TO 3",
    "    auswahl[i] = -1",
    "NEXT",
    "auswahl[0] = INT(RND() * LEN(de))",
    "i = 1",
    "WHILE i < 4",
    "    k = INT(RND() * LEN(de))",
    "    IF ARRAY_INDEXOF(auswahl, k) < 0 THEN",
    "        auswahl[i] = k",
    "        i = i + 1",
    "    END IF",
    "WEND",
  ]),

  H.pmix(["Auf Platz null steht die richtige Antwort. Danach werden drei weitere Nummern gezogen, und ", ["ARRAY_INDEXOF", true], " aus Kapitel 14 sorgt dafür, dass keine doppelt vorkommt. Wäre die richtige Antwort zweimal dabei, wäre die Frage unbeantwortbar."]),

  H.warn("Die drei Zeilen mit auswahl[i] = -1 davor sind kein Zierrat. Ein frisch angelegtes Zahlen-Array ist mit Nullen gefüllt — und dann findet ARRAY_INDEXOF die Null in den noch leeren Fächern und lehnt die Vokabel Nummer 0 für immer ab. Mit -1 vorbelegt kann das nicht passieren, weil es keine Vokabel -1 gibt.", "Warum -1 und nicht 0"),

  H.p("Danach wird gemischt — aber nur einmal, und ohne Schleife:"),

  H.code([
    "richtig = INT(RND() * 4)",
    "k = auswahl[richtig]",
    "auswahl[richtig] = auswahl[0]",
    "auswahl[0] = k",
  ]),

  H.pmix(["Ein Tausch, drei Zeilen, ein Hilfsspeicher ", ["k", true], ". Die richtige Antwort springt von Platz null auf einen zufälligen Platz, und was dort stand, geht auf die null. Danach weiß das Programm mit ", ["richtig", true], ", wo sie gelandet ist."]),

  H.note("Ein direktes auswahl[0] = auswahl[richtig] : auswahl[richtig] = auswahl[0] geht schief — nach der ersten Zeile ist der alte Wert weg. Deshalb der Umweg über k. Das ist dieselbe Falle wie beim Vertauschen zweier Variablen, und sie erwischt jeden einmal."),

  H.p("Der Rest ist Färben. Nach dem Tastendruck wird die richtige Antwort grün, die falsch gewählte rot:"),

  H.code([
    "f = 1",
    "IF gewaehlt >= 0 AND i = richtig THEN f = 2",
    "IF gewaehlt = i AND i <> richtig THEN f = 3",
  ]),

  H.pmix(["Und dazu ein Ton: ", ["AUDIO_TONE(880, 90, \"square\", 0.35)", true], " für richtig, ", ["AUDIO_TONE(160, 180, \"saw\", 0.35)", true], " für falsch. Hoch und kurz gegen tief und lang — die Klänge aus Kapitel 11, in ihrer natürlichsten Anwendung. Man weiß, ob es stimmte, bevor man hingesehen hat."]),

  H.h2("Zweitens: tippen"),

  H.p("Die ehrliche Form. Kein Anhaltspunkt, keine Auswahl — das Wort muss aus dem Kopf kommen."),

  H.figure("kap31_2_tippen.png", "Ein Eingabefeld, ein Knopf, und die Eingabetaste tut dasselbe.", 440, 260),

  H.p("Technisch ist das die einfachste der drei Fragearten und in Wahrheit die schwierigste. Denn jetzt muss das Programm entscheiden, ob zwei Texte „dasselbe“ sind — und das ist eine Zumutung."),

  H.bulletRich("„House“ statt „house“ ", "— soll gelten."),
  H.bulletRich("„the house“ statt „house“ ", "— soll gelten."),
  H.bulletRich("„hause“ statt „house“ ", "— ein Tippfehler. Soll auch gelten."),
  H.bulletRich("„horse“ statt „house“ ", "— ein anderes Wort. Soll nicht gelten."),

  H.p("Die ersten beiden sind leicht:"),

  H.code([
    "FUNCTION ohne_artikel(s AS STRING) AS STRING",
    "    DIM t AS STRING",
    "    DIM a AS STRING",
    "    t = TRIM$(LOWER$(s))",
    '    FOR EACH a IN SPLIT$("to |the |a |an |la |le |les |l\'|" + _',
    '                         "el |los |las |der |die |das ", "|")',
    "        IF LEFT$(t, LEN(a)) = a THEN RETURN TRIM$(MID$(t, LEN(a)))",
    "    NEXT",
    "    RETURN t",
    "END FUNCTION",
  ]),

  H.pmix([["LOWER$", true], " macht alles klein, ", ["TRIM$", true], " nimmt Leerzeichen weg, und dann wird der Reihe nach geprüft, ob der Text mit einem Artikel anfängt. Der Trick mit ", ["SPLIT$", true], " spart dreizehn ", ["IF", true], "-Zeilen: Die Artikel stehen als ein Text da, durch ", ["|", true], " getrennt, und werden zur Liste zerlegt."]),

  H.pmix([["MID$(t, LEN(a))", true], " mit nur zwei Werten heißt „ab hier bis zum Ende“. Und weil in Kapitel 17 gemessen wurde, dass ab null gezählt wird, ist ", ["LEN(a)", true], " genau die Stelle hinter dem Artikel."]),

  H.h2("Der Tippfehler — eine Zahl für Ähnlichkeit"),

  H.p("Der dritte Fall ist der interessante. Wie soll ein Programm merken, dass „hause“ ein verrutschter Buchstabe ist und „horse“ nicht?"),

  H.p("Es gibt dafür ein Maß, und es ist eine der schönsten kleinen Erfindungen der Informatik: Man zählt, wie viele einzelne Änderungen nötig sind, um aus einem Wort das andere zu machen. Erlaubt ist Einfügen, Löschen, Ersetzen. Diese Zahl heißt Editier-Abstand."),

  H.table([
    [{ text: "house / house", mono: true }, "0", "gleich"],
    [{ text: "house / hause", mono: true }, "1", "ein Buchstabe ersetzt"],
    [{ text: "house / hous", mono: true }, "1", "ein Buchstabe fehlt"],
    [{ text: "house / haus", mono: true }, "2", "zwei Änderungen"],
    [{ text: "el ano / el año", mono: true }, "1", "die Tilde fehlt"],
    [{ text: "cat / dog", mono: true }, "3", "nichts gemeinsam"],
  ], { headers: ["Vergleich", "Abstand", "Was passiert ist"], widths: [3000, 1200, 4826], mono: [0] }),

  H.p("Der Trainer lässt einen Abstand von 1 als „fast“ durchgehen und zeigt trotzdem die richtige Schreibweise. Ab 2 ist es ein anderes Wort."),

  H.p("Berechnet wird der Abstand so:"),

  H.code([
    "FUNCTION abstand(a AS STRING, b AS STRING) AS INTEGER",
    "    DIM zeile[0] AS INTEGER",
    "    DIM n AS INTEGER",
    "    DIM m AS INTEGER",
    "    DIM i AS INTEGER",
    "    DIM j AS INTEGER",
    "    DIM kosten AS INTEGER",
    "    DIM oben AS INTEGER",
    "    DIM schraeg AS INTEGER",
    "    n = LEN(a)",
    "    m = LEN(b)",
    "    REDIM(zeile, m + 1)",
    "    FOR j = 0 TO m",
    "        zeile[j] = j",
    "    NEXT",
    "    FOR i = 1 TO n",
    "        schraeg = zeile[0]",
    "        zeile[0] = i",
    "        FOR j = 1 TO m",
    "            oben = zeile[j]",
    "            kosten = 1",
    "            IF MID$(a, i - 1, 1) = MID$(b, j - 1, 1) THEN kosten = 0",
    "            zeile[j] = MIN(MIN(oben + 1, zeile[j - 1] + 1), _",
    "                           schraeg + kosten)",
    "            schraeg = oben",
    "        NEXT",
    "    NEXT",
    "    RETURN zeile[m]",
    "END FUNCTION",
  ]),

  H.p("Das ist das anspruchsvollste Stück Code in diesem Buch, und es lohnt sich, einen Moment davor stehenzubleiben."),

  H.p("Die Idee: Man denkt sich eine Tabelle, in der links die Buchstaben des einen Wortes stehen und oben die des anderen. In jedes Feld kommt der Abstand der beiden Wortanfänge bis dorthin. Ein Feld lässt sich immer aus seinen drei Nachbarn ausrechnen — dem oberen, dem linken und dem schrägen — und aus der Frage, ob die beiden Buchstaben gleich sind."),

  H.bulletRich("von oben ", "— ein Buchstabe gelöscht, also eins mehr."),
  H.bulletRich("von links ", "— ein Buchstabe eingefügt, also eins mehr."),
  H.bulletRich("von schräg ", "— ersetzt, also eins mehr, außer die Buchstaben sind gleich."),
  H.bulletRich("MIN von den dreien ", "— der günstigste Weg zählt."),

  H.pmix(["Der Kniff im Code ist, dass die ganze Tabelle nie entsteht. Es gibt nur EINE Zeile, ", ["zeile", true], ", und die wird von links nach rechts überschrieben. Was gerade überschrieben wird, war der obere Nachbar — deshalb wird es vorher in ", ["oben", true], " gerettet und dient im nächsten Schritt als schräger Nachbar. Statt einer Tabelle mit n×m Feldern braucht das Programm m+1."]),

  H.note("Dieses Verfahren heißt nach seinem Erfinder Levenshtein-Abstand und steckt in fast jedem Programm, das „Meintest du …?“ sagt. Man muss es nicht erfinden können, um es zu benutzen — aber wer einmal nachvollzogen hat, wie eine gesparte Tabelle funktioniert, sieht danach viele Programme mit anderen Augen."),

  H.warn("Es gibt eine Grenze, und sie ist gemessen: „windwo“ statt „window“ hat den Abstand 2, nicht 1 — zwei vertauschte Nachbarn kosten in dieser Rechnung zwei Änderungen. Ein häufiger Tippfehler wird also strenger bewertet als ein seltener. Wen das stört, sucht nach dem Damerau-Levenshtein-Abstand; der kennt das Vertauschen als eigene Änderung.", "Was der Abstand nicht sieht"),

  H.h2("Drittens: Paare zuordnen"),

  H.p("Die dritte Form ist die, bei der man aufhört zu merken, dass man lernt. Sechs deutsche Wörter links, sechs fremde rechts in anderer Reihenfolge, und man klickt zusammen, was zusammengehört."),

  H.figure("kap31_3_zuordnen.png", "Rechts ist gemischt. Ein Klick links, ein Klick rechts, und wenn es passt, bleibt es grün.", 440, 300),

  H.p("Gemischt wird mit einem Verfahren, das jeder Kartenspieler kennt — von hinten nach vorn, jede Karte mit einer zufälligen davor:"),

  H.code([
    "FOR i = PAARE - 1 TO 1 STEP -1",
    "    j = INT(RND() * (i + 1))",
    "    k = misch[i]",
    "    misch[i] = misch[j]",
    "    misch[j] = k",
    "NEXT",
  ]),

  H.pmix([["STEP -1", true], " lässt die Schleife rückwärts laufen. Danach steht in ", ["misch[i]", true], ", welches linke Wort zur rechten Zeile ", ["i", true], " gehört — und die Prüfung beim Klicken ist ein einziger Vergleich."]),

  H.p("Neu ist hier die Maus. Sie funktioniert wie die Tastatur in Kapitel 7, nur muss man sich den Tastendruck selbst merken:"),

  H.code([
    "vorher = gedrueckt",
    "gedrueckt = MOUSEBUTTON(0)",
    "mx = MOUSEX()",
    "my = MOUSEY()",
    "",
    "IF gedrueckt AND NOT vorher THEN",
    "    ' ... hier war gerade ein frischer Klick ...",
    "END IF",
  ]),

  H.pmix([["MOUSEBUTTON(0)", true], " sagt, ob die linke Taste GEDRÜCKT IST — nicht, ob sie gerade gedrückt WURDE. Das ist der Unterschied zwischen ", ["KEYDOWN", true], " und ", ["KEYPRESSED", true], " aus Kapitel 7, nur muss man ihn hier selbst herstellen: den Zustand vom letzten Bild merken und vergleichen. Ohne das würde ein Klick sechzigmal je Sekunde zählen."]),

  H.p("Und aus der Mausposition wird die angeklickte Zeile eine Rechnung:"),

  H.code([
    "i = (my - 60) \\ 52",
  ]),

  H.pmix(["Die Zeilen fangen bei 60 an und sind 52 hoch. Also: Abstand vom oberen Rand, geteilt durch die Zeilenhöhe. Der ", ["\\", true], " ist die ganzzahlige Division aus Kapitel 2 — und hier ist sie genau richtig, denn eine halbe Zeile gibt es nicht."]),

  H.h2("Und viertens: die Richtung"),

  H.p("Die vierte Art ist keine eigene Programmform, sondern ein Handgriff: Man dreht die Frage um. Statt „Was heißt Haus?“ heißt es „Was heißt house?“."),

  H.p("Das klingt nach nichts und ist ein großer Unterschied. Vom Deutschen ins Fremde ist schwerer — dort muss man produzieren. Vom Fremden ins Deutsche ist leichter — dort genügt Erkennen. Ein Trainer, der immer nur eine Richtung übt, erzeugt Vokabelkenntnis, die im Gespräch nicht abrufbar ist."),

  H.p("Im Code ist es ein Tausch von zwei Variablen, mehr nicht. In Kapitel 33 wird daraus eine Leiter:"),

  H.table([
    ["neu", "beide Seiten zeigen", "vorstellen, nicht prüfen"],
    ["1", "vier Antworten, deutsch → fremd", "wiedererkennen"],
    ["2", "vier Antworten, fremd → deutsch", "wiedererkennen, andere Richtung"],
    ["3", "tippen, deutsch → fremd", "abrufen"],
    ["4 und 5", "tippen, fremd → deutsch", "abrufen, andere Richtung"],
  ], { headers: ["Fach", "Wie gefragt wird", "Was geübt wird"], widths: [1200, 4000, 3826] }),

  H.p("Von leicht nach schwer, gesteuert vom Karteikasten. Wer eine Vokabel gut kann, bekommt die harte Frage; wer sie gerade erst gesehen hat, bekommt die leichte. Das ist der Unterschied zwischen einem Abfrageprogramm und einem Trainer."),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    ["Die richtige Antwort steht zweimal da", "Die Prüfung mit ARRAY_INDEXOF fehlt beim Ziehen der Ablenker."],
    ["Die erste Vokabel kommt nie als Ablenker vor", "Das Array ist mit Nullen vorbelegt statt mit -1."],
    ["Die richtige Antwort steht immer an derselben Stelle", "Der Tausch fehlt, oder richtig wird nach dem Tausch neu gewürfelt."],
    ["Nach dem Tausch sind zwei Antworten gleich", "Zuweisung ohne Hilfsvariable — der alte Wert ist verloren."],
    ["Ein Klick zählt viele Male", "MOUSEBUTTON ohne Vergleich mit dem letzten Bild."],
    ["Beim Tippen gilt nichts als richtig", "LOWER$ oder TRIM$ fehlt, oder es wird gegen den Text MIT Artikel verglichen."],
    ["Beim Tippen gilt alles als richtig", "Die Schranke steht bei 2 oder höher statt bei 1."],
    ["Das Urteil ist sofort wieder weg", "Die nächste Frage wird im selben Bild geholt. Ein Zähler oder ein Knopf muss dazwischen."],
    [{ text: "Variable 'key_one' nicht deklariert", mono: true }, "Die Zifferntasten heißen KEY_1 bis KEY_9."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3600, 5426] }),

  H.h2("Aufgaben"),

  H.bullet("Lass bei „vier Antworten“ die Ablenker nicht rein zufällig ziehen, sondern bevorzugt solche mit ähnlicher Länge. Die Frage wird spürbar schwerer."),
  H.bullet("Zeig beim Tippen an, welcher Buchstabe falsch war — der Abstand weiß es, du musst ihn nur mitschreiben lassen."),
  H.bullet("Bau in „vier Antworten“ eine Zeit ein: Wer in fünf Sekunden nicht antwortet, hat es nicht gewusst."),
  H.bullet("Lass beim Zuordnen die gelösten Paare nach oben rutschen, statt an ihrem Platz zu bleiben."),
  H.bullet("Ergänze im Zuordnen-Spiel eine Linie zwischen den beiden angeklickten Kästen."),
  H.bullet("Schreib eine kleine Testreihe, die abstand() mit zehn Wortpaaren aufruft und die Ergebnisse ausgibt. Vergleich sie mit dem, was du erwartet hättest."),

  H.p("Damit sind die Fragearten beisammen. Was noch fehlt, ist die Möglichkeit, eigene Wörter hineinzubringen — denn was in der Schule drankommt, steht in keiner fertigen Liste."),
];
