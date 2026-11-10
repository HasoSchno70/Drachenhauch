module.exports = (H) => [
  H.chapter("Projekt: ein Instrument"),

  H.p("Zum Abschluss von Teil II bauen wir etwas, auf dem man wirklich spielen kann. Acht Tasten, acht Töne, vier Klangfarben und drei Oktaven — und niemand außer dir bestimmt, was passiert."),

  H.p("Das Programm ist etwa achtzig Zeilen lang und bringt genau einen neuen Gedanken mit: dass ein Array auch Klänge halten kann."),

  H.figure("kap12_instrument.png", "Acht Tasten, beschriftet wie auf der Tastatur. Wer eine hält, sieht sie leuchten.", 440, 280),

  H.h2("Vier Arrays nebeneinander"),

  H.p("Die Grundidee ist, dass alles, was es achtmal gibt, in ein Array wandert — und zwar so, dass Fach 3 überall dasselbe meint:"),

  H.code([
    "DIM tasten[8] AS INTEGER      ' welche Taste",
    "DIM grund[8] AS INTEGER       ' welche Frequenz",
    "DIM klang[8] AS SOUND         ' der fertige Ton",
    'DIM schild[8] AS STRING       \' was auf der Taste steht',
  ]),

  H.p("Fach 3 ist also: die Taste F, die Frequenz 349, der dazu gebaute Klang und die Beschriftung „F“. Vier Arrays, ein gemeinsamer Index — das nennt man parallele Arrays, und es ist die einfachste Art, mehrere Angaben zu einer Sache zusammenzuhalten."),

  H.pmix(["Neu ist ", ["DIM klang[8] AS SOUND", true], ": ein Array, in dem keine Zahlen liegen, sondern acht fertige Klänge. Genau dafür lohnt sich der Behälter — acht einzeln benannte Kartons wären hier nicht auszuhalten."]),

  H.pmix(["Ebenfalls neu: ", ["DIM schild[8] AS STRING", true], ", ein Array aus Text. Arrays halten alles, solange in einem Array nur eine Sorte steckt."]),

  H.p("Gefüllt werden sie stumpf, Zeile für Zeile:"),

  H.code([
    "tasten[0] = KEY_A",
    "tasten[1] = KEY_S",
    "tasten[2] = KEY_D",
    "tasten[3] = KEY_F",
    "",
    "grund[0] = 262",
    "grund[1] = 294",
    "grund[2] = 330",
    "grund[3] = 349",
  ]),

  H.pmix(["Bemerkenswert ist die erste Gruppe: ", ["KEY_A", true], " ist nichts Geheimnisvolles, sondern eine Zahl — und deshalb passt sie in ein Zahlen-Array. Später steht dann ", ["KEYPRESSED(tasten[i])", true], " da, und die Schleife fragt der Reihe nach alle acht Tasten ab."]),

  H.h2("Die Tasten malen"),

  H.code([
    "FOR i = 0 TO 7",
    "    lx = 24 + i * 74",
    "    farbe = dunkel",
    "    IF KEYPRESSED(tasten[i]) THEN farbe = hell",
    "    BOX(lx, 120, lx + 66, 300, farbe)",
    "    TEXT(lx + 26, 268, schild[i], RGB(60, 60, 70))",
    "NEXT",
  ]),

  H.p("Sechs Zeilen für acht Tasten samt Beschriftung und Leuchten. Das ist genau die Tastenanzeige aus Kapitel 7 — dort standen vier Kästchen einzeln untereinander, hier erledigt eine Schleife acht."),

  H.pmix([["lx = 24 + i * 74", true], " ist die linke Kante der Taste Nummer i: 24 Punkte Rand, dann je 74 weiter. Die Taste selbst ist 66 breit, also bleiben acht Punkte Fuge."]),

  H.p("Und wieder das Muster: erst die dunkle Farbe annehmen, dann bei gedrückter Taste überschreiben."),

  H.h2("Die Töne bauen — aber wann?"),

  H.p("Jetzt die eigentliche Frage des Kapitels. Die Klangform soll umschaltbar sein; also müssen die acht Klänge neu gebaut werden, sobald jemand umschaltet. Aber eben nur dann und nicht sechzigmal je Sekunde."),

  H.p("Die Lösung ist dieselbe wie beim Neustart der Schlange: ein Schalter, der sagt „hier ist etwas zu tun“."),

  H.code([
    "IF bauen THEN",
    "    bauen = FALSE",
    "    FOR i = 0 TO 7",
    "        klang[i] = AUDIO_TONE(grund[i] * oktave, 500, form, 0.4)",
    "    NEXT",
    "END IF",
  ]),

  H.pmix(["Der Block steht ganz oben in der Schleife. ", ["bauen", true], " ist vor der Schleife auf ", ["TRUE", true], " gesetzt, damit die Klänge beim ersten Durchgang entstehen. Danach setzt er sich selbst auf ", ["FALSE", true], " — und wird nur wieder wahr, wenn jemand eine Taste zum Umschalten drückt."]),

  H.pmix([["grund[i] * oktave", true], " ist die Oktavverschiebung. ", ["oktave", true], " ist eine Kommazahl: 1.0 ist die Grundlage, 2.0 doppelt so hoch, 0.5 halb so hoch. Verdoppeln heißt eine Oktave höher — das stand schon in Kapitel 10, und hier wird es benutzt."]),

  H.warn("Das ist genau die Stelle, an der die Regel aus dem letzten Kapitel greift: Klänge werden nicht in jedem Bild gebaut. Ohne den Schalter stünden hier acht AUDIO_TONE-Aufrufe je Bild, also 480 je Sekunde. Das Instrument würde nach kurzer Zeit stocken.", "Bauen ist teuer, Abspielen ist billig"),

  H.h2("Spielen"),

  H.code([
    "FOR i = 0 TO 7",
    "    IF KEYHIT(tasten[i]) THEN PLAYSOUND(klang[i])",
    "NEXT",
  ]),

  H.p("Zwei Zeilen für das ganze Instrument."),

  H.pmix(["Hier steht ", ["KEYHIT", true], " und nicht ", ["KEYPRESSED", true], ", und das ist der entscheidende Unterschied aus Kapitel 7. Mit ", ["KEYPRESSED", true], " würde jede gehaltene Taste sechzigmal je Sekunde einen neuen Ton anstoßen — ein Schnarren statt einer Note."]),

  H.p("Beim Malen ist es genau umgekehrt: Dort soll die Taste leuchten, SOLANGE sie gehalten wird. Deshalb steht oben KEYPRESSED und hier KEYHIT. Dasselbe Programm, dieselbe Taste, zwei verschiedene Fragen."),

  H.h2("Umschalten"),

  H.code([
    "IF KEYHIT(KEY_1) THEN",
    '    form = "sine"',
    "    bauen = TRUE",
    "END IF",
    "",
    "IF KEYHIT(KEY_UP) AND oktave < 4 THEN",
    "    oktave = oktave * 2",
    "    bauen = TRUE",
    "END IF",
  ]),

  H.p("Viermal dasselbe für die vier Klangformen, zweimal für die Oktave. Jeder dieser Blöcke tut zwei Dinge: den Wert ändern und den Schalter umlegen. Das Neubauen selbst steht nur an einer Stelle im Programm."),

  H.pmix(["Die Zusätze ", ["AND oktave < 4", true], " und ", ["AND oktave > 0.3", true], " begrenzen den Bereich. Ohne sie könnte man beliebig weit hinauf — und würde irgendwann Töne bauen, die kein Mensch mehr hört, das Programm aber weiterhin brav erzeugt."]),

  H.h2("Das ganze Instrument"),

  H.p("Das vollständige Programm steht in code/kap12/instrument.dh. Es ist lang, aber es enthält nichts, was nicht auf den letzten Seiten erklärt wurde:"),

  H.bulletRich("Vier parallele Arrays ", "— Taste, Frequenz, Klang, Beschriftung."),
  H.bulletRich("Ein Bau-Schalter ", "— acht Klänge entstehen nur, wenn sich etwas geändert hat."),
  H.bulletRich("Eine Malschleife ", "— acht Tasten, leuchtend wenn gehalten."),
  H.bulletRich("Eine Spielschleife ", "— KEYHIT je Taste, ein PLAYSOUND."),
  H.bulletRich("Sechs Umschaltblöcke ", "— Wert ändern, Schalter umlegen."),

  H.p("Damit endet Teil II. Rückblickend war das ein kurzer Teil: Der ganze Klang dieses Buches steckt in zwei Befehlen, AUDIO_TONE und PLAYSOUND, plus AUDIO_NOISE für alles, was kein Ton ist. Mehr braucht man erstaunlich lange nicht."),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    ["Die Taste schnarrt statt zu klingen", "KEYPRESSED statt KEYHIT beim Abspielen."],
    ["Die Taste leuchtet nur kurz auf", "KEYHIT statt KEYPRESSED beim Malen. Beides genau andersherum als beim Klang."],
    ["Nach dem Umschalten kommt kein Ton", "Der Schalter wird gesetzt, aber der Bau-Block steht hinter dem Abspielen. Er gehört an den Anfang der Schleife."],
    ["Das Instrument stockt nach einer Weile", "Der Bau-Block hat keinen Schalter und läuft in jedem Bild."],
    ["Zwei Tasten gleichzeitig gehen nicht", "Meist die Tastatur, nicht das Programm. Billige Tastaturen melden bestimmte Dreierkombinationen nicht."],
    [{ text: "Index ausserhalb", mono: true }, "Eine Schleife läuft bis 8 statt bis 7. Bei acht Fächern ist 7 das letzte."],
  ], { headers: ["Was du merkst", "Was meistens dahintersteckt"], widths: [3400, 5626] }),

  H.h2("Aufgaben"),

  H.bullet("Beschrifte die Tasten zusätzlich mit den Notennamen c, d, e, f, g, a, h, c."),
  H.bullet("Bau die schwarzen Tasten dazu: cis, dis, fis, gis, ais liegen bei 277, 311, 370, 415 und 466 Hertz. Leg sie auf W, E, T, Z und U."),
  H.bullet("Lass die zuletzt gespielte Note groß in der Mitte stehen."),
  H.bullet("Nimm auf, was gespielt wird: Schreib jede gedrückte Taste in ein Array und spiel die Folge auf Knopfdruck ab. Du brauchst dafür ein Array und einen Zähler — genau wie bei der Schlange."),
  H.bullet("Gib jeder Taste eine eigene Farbe, die sich über den Regenbogen verteilt."),
  H.bullet("Bau einen Metronom-Takt dazu: alle 500 Millisekunden ein kurzes Klicken aus AUDIO_NOISE."),

  H.p("Im nächsten Teil geht es um Ordnung. Deine Programme werden länger, und einiges darin steht inzwischen zweimal da — die beiden fast gleichen Schlägerblöcke in Pong etwa. Dagegen gibt es ein Mittel."),
];
