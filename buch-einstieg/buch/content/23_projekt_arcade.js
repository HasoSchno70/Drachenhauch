module.exports = (H) => [
  H.chapter("Projekt: ein Arcade-Spiel"),

  H.p("Fünfzehn Gegner in Formation, ein Schiff, ein Schuss. Die Gegner wandern hin und her und rücken jedes Mal ein Stück näher, wenn sie am Rand umkehren. Wer sie alle erwischt, hat gewonnen; wer sie unten ankommen lässt, hat verloren."),

  H.p("Das ist das Grundgerüst eines der berühmtesten Spiele überhaupt, und es passt in gut hundertfünfzig Zeilen — von denen du fast jede schon einmal gesehen hast."),

  H.figure("kap23_arcade.png", "Fünfzehn Gegner, drei Reihen, ein Schiff. Die Formation wandert seitwärts.", 440, 280),

  H.h2("Was aus welchem Kapitel kommt"),

  H.table([
    ["Spielschleife, Bildzähler", "Kapitel 5"],
    ["Entscheidungen, Begrenzung am Rand", "Kapitel 6"],
    ["Steuerung, KEYHIT für den Schuss", "Kapitel 7"],
    ["Arrays für die Gegner", "Kapitel 9 und 14"],
    ["Eine eigene Funktion für die Trefferprüfung", "Kapitel 13"],
    ["Klänge für Schuss, Treffer und Ende", "Kapitel 11"],
    ["Sprites laden und Ausschnitte malen", "Kapitel 19"],
    ["Animation über die Feldnummer", "Kapitel 21"],
    ["Kollision zweier Rechtecke", "Kapitel 22"],
  ], { headers: ["Baustein", "Woher"], widths: [5600, 3426] }),

  H.p("Es kommt nichts Neues hinzu. Das ist der Punkt dieses Kapitels — und es lohnt sich, das einen Moment sacken zu lassen: Zweiundzwanzig Kapitel Grundlagen reichen für ein Spiel, das man wirklich spielen kann."),

  H.h2("Die Gegner in drei Arrays"),

  H.code([
    "DIM gx[15] AS FLOAT",
    "DIM gy[15] AS FLOAT",
    "DIM lebt[15] AS BOOLEAN",
  ]),

  H.p("Fünfzehn Gegner: fünf Spalten, drei Reihen. Beim Aufbau werden sie in einer Doppelschleife gesetzt, und die laufende Nummer ergibt sich aus Reihe und Spalte:"),

  H.code([
    "FOR ze = 0 TO 2",
    "    FOR sp = 0 TO 4",
    "        i = ze * 5 + sp",
    "        gx[i] = 70 + sp * 100",
    "        gy[i] = 40 + ze * 60",
    "        lebt[i] = TRUE",
    "    NEXT",
    "NEXT",
  ]),

  H.pmix([["i = ze * 5 + sp", true], " ist der Kniff, der ein Rechteck in eine Reihe verwandelt: Reihe 0 belegt die Nummern 0 bis 4, Reihe 1 die Nummern 5 bis 9, Reihe 2 die Nummern 10 bis 14. Man könnte auch ein zweidimensionales Array nehmen wie im Spiel des Lebens — aber weil die Gegner einzeln sterben und nicht als Gitter gedacht sind, ist die durchlaufende Nummer hier angenehmer."]),

  H.h2("Die Formation wandert"),

  H.p("Das ist die einzige wirklich knifflige Stelle des Spiels, und sie enthält einen Fehler, den fast jeder einmal macht."),

  H.code([
    "' Erst schauen, wie weit die Formation reicht, DANN umkehren --",
    "' sonst dreht sie bei zwei Gegnern am Rand zweimal und steht still.",
    "uebrig = 0",
    "tiefster = 0",
    "linkeste = 9999",
    "rechteste = -9999",
    "FOR i = 0 TO 14",
    "    IF lebt[i] THEN",
    "        uebrig = uebrig + 1",
    "        IF gy[i] > tiefster THEN tiefster = gy[i]",
    "        IF gx[i] < linkeste THEN linkeste = gx[i]",
    "        IF gx[i] > rechteste THEN rechteste = gx[i]",
    "    END IF",
    "NEXT",
    "",
    "IF linkeste < 4 OR rechteste > 604 THEN",
    "    richtung = -richtung",
    "    FOR i = 0 TO 14",
    "        gy[i] = gy[i] + 16",
    "    NEXT",
    "END IF",
    "",
    "FOR i = 0 TO 14",
    "    IF lebt[i] THEN gx[i] = gx[i] + richtung",
    "NEXT",
  ]),

  H.warn("Der naheliegende Weg wäre, in EINER Schleife zu bewegen und dabei zu prüfen, ob der Gegner den Rand berührt. Genau das habe ich zuerst geschrieben, und es ist falsch: Sind zwei Gegner gleichzeitig am Rand, dreht die Richtung zweimal um — also gar nicht —, und die Formation zittert an der Wand fest. Die Lösung ist, in drei Schritten zu denken: erst messen, wie weit die Formation reicht, dann EINMAL entscheiden, dann bewegen.", "Erst messen, dann entscheiden, dann bewegen"),

  H.p("Nebenbei wird in derselben Messschleife noch zweierlei erledigt: gezählt, wie viele Gegner noch leben, und gemerkt, wie tief der tiefste steht. Beides braucht das Spielende. Eine Schleife, drei Antworten — das ist keine Sparsamkeit, sondern verhindert, dass drei Schleifen mit derselben Bedingung auseinanderlaufen."),

  H.h2("Der Zustand des Spiels"),

  H.p("Ein Spiel ist nicht immer im selben Zustand. Es läuft, oder es ist gewonnen, oder es ist verloren. Dafür genügt ein Text:"),

  H.code([
    "IF uebrig = 0 THEN lage = \"Geschafft\"",
    "IF tiefster > 310 THEN",
    "    lage = \"Verloren\"",
    "    PLAYSOUND(ende)",
    "END IF",
  ]),

  H.p("Und ganz oben in der Schleife, direkt nach dem Malen, steht die Bremse:"),

  H.code([
    "IF KEYHIT(KEY_R) THEN neu = TRUE",
    "IF lage <> \"laeuft\" THEN",
    "    bild = bild + 1",
    "    CONTINUE",
    "END IF",
  ]),

  H.pmix([["CONTINUE", true], " ist neu: Es überspringt den Rest des Schleifendurchgangs und beginnt sofort den nächsten. Hier heißt das: Ist das Spiel vorbei, wird zwar noch gemalt und auf R gehört, aber nichts mehr bewegt, geschossen oder geprüft. Ohne diese fünf Zeilen liefen die Gegner nach dem Spielende munter weiter."]),

  H.note("Beachte, WO diese Prüfung steht: nach dem Malen und dem FLIP, aber vor allem, was rechnet. Das Bild bleibt also stehen und die Meldung ist zu sehen — nur die Welt friert ein. Die Reihenfolge im Schleifenrumpf ist bei Spielen fast immer die eigentliche Entscheidung."),

  H.h2("Die Trefferbox der Gegner"),

  H.code([
    "IF ueberlappt(sx, sy, 4, 12, _",
    "              gx[i] + 4, gy[i] + 6, 24, 20) THEN",
  ], { out: true }),

  H.p("Der Schuss ist vier Punkte breit und zwölf hoch — das ist genau das Rechteck, das auch gemalt wird. Beim Gegner dagegen wird ein kleineres Rechteck geprüft als das 32er-Sprite: vier Punkte vom linken Rand, sechs von oben, dann 24 mal 20."),

  H.p("Das ist die Trefferbox aus dem letzten Kapitel, und hier sieht man, wofür sie gut ist: Die Fühler des Gegners ragen oben heraus, und ein Schuss, der nur den Fühler streift, soll nicht zählen. Umgekehrt sind die Beine unten dünn — auch die zählen nicht mit."),

  H.h2("Das ganze Spiel"),

  H.p("Das vollständige Programm steht in code/kap23/arcade.dh — 164 Zeilen, von denen mehr als die Hälfte Vorbereitung und Abdruck sind. Es lohnt sich, es einmal von oben nach unten zu lesen und dabei die vier Abschnitte der Spielschleife wiederzuerkennen: löschen, malen, zeigen, rechnen."),

  H.p("Und dann solltest du es spielen. Nicht lange — zwei Minuten genügen, um zu merken, dass etwas fehlt. Genau das ist die interessanteste Frage dieses Kapitels."),

  H.h2("Was fehlt (und warum das gut ist)"),

  H.p("Ein paar Dinge fallen sofort auf. Die Gegner schießen nicht zurück. Es gibt nur ein Leben. Es gibt keine zweite Welle. Der Schuss ist ein Rechteck und kein Bild. Und wenn nur noch ein Gegner übrig ist, wandert er genauso gemächlich wie fünfzehn."),

  H.p("Nichts davon ist schwer nachzurüsten, und jedes einzelne ist eine gute Übung. Wichtiger ist die Einsicht dahinter: Ein Spiel wird nicht dadurch gut, dass man es größer macht, sondern dadurch, dass man es spielt und merkt, was stört. Diese Reihenfolge — bauen, spielen, ändern — ist bei Programmen aller Art dieselbe."),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    ["Die Formation zittert am Rand", "Gemessen und gedreht wird in derselben Schleife. Siehe den Warnkasten."],
    ["Nach dem Spielende läuft alles weiter", "Die CONTINUE-Bremse fehlt oder steht an der falschen Stelle."],
    ["Ein Schuss trifft zwei Gegner", "Nach dem Treffer wird fliegt nicht auf FALSE gesetzt."],
    ["Die Gegner rücken zu weit nach unten", "Beim Umkehren werden alle gy erhöht, auch die der toten — das ist Absicht und stört nicht, solange nur lebende geprüft werden."],
    ["Gewonnen wird sofort", "uebrig wird nach dem Zählen nicht zurückgesetzt, sondern bleibt von der letzten Runde stehen."],
    ["Beim Neustart bleiben tote Gegner tot", "Der neu-Block setzt lebt[i] nicht wieder auf TRUE."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3400, 5626] }),

  H.h2("Aufgaben"),

  H.bullet("Lass die Gegner schneller werden, je weniger übrig sind. Eine Zeile genügt, wenn richtung von uebrig abhängt."),
  H.bullet("Gib dem Spieler drei Leben und zeig sie als kleine Schiffe unten rechts an."),
  H.bullet("Lass gelegentlich einen Gegner eine Bombe fallen — ein Rechteck, das nach unten fliegt und das Schiff treffen kann."),
  H.bullet("Ersetz den Schuss durch ein selbst gemaltes Sprite."),
  H.bullet("Bau eine zweite Welle: Ist die Formation erledigt, kommt eine neue, die etwas tiefer beginnt."),
  H.bullet("Speichere den höchsten Punktestand in einer Datei, so wie in Kapitel 18 — dann überlebt er den Neustart."),

  H.p("Damit endet Teil IV. Du hast ein Spiel gebaut, dessen Figuren du selbst malen kannst. Im nächsten Teil verlassen wir die Spiele und bauen etwas, das ein Fenster mit Knöpfen hat — die Vorstufe zum Vokabeltrainer."),
];
