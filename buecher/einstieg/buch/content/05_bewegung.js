module.exports = (H) => [
  H.chapter("Bewegung"),

  H.p("Jedes Programm bis hierher hat ein Bild gemalt, drei Sekunden gewartet und sich verabschiedet. Das ändert sich jetzt grundlegend — und es ist der wichtigste Schritt in diesem ganzen Buch."),

  H.p("Denn Bewegung auf einem Bildschirm ist eine Täuschung. Nichts bewegt sich wirklich. Es wird sechzigmal in der Sekunde ein neues Bild gemalt, und auf jedem steht der Ball ein kleines Stück weiter als auf dem vorigen. Dein Auge macht daraus eine Bewegung — genau wie im Kino."),

  H.h2("Der erste Punkt, der wandert"),

  H.code([
    'SCREEN(640, 400, "Wanderer")',
    "",
    "DIM x AS INTEGER",
    "x = 0",
    "",
    "WHILE NOT QUITREQUESTED() AND NOT KEYPRESSED(KEY_ESCAPE)",
    "    CLS(RGB(15, 20, 40))",
    "    CIRCLE(x, 200, 25, RGB(255, 200, 40))",
    "    FLIP()",
    "    x = (x + 3) MOD 640",
    "WEND",
  ]),

  H.figure("kap05_1_wanderer.png", "Ein Standbild aus einer Bewegung. Der Punkt läuft nach rechts und beginnt links von vorn.", 440, 280),

  H.p("Der Punkt zieht gemächlich nach rechts, verschwindet am Rand und kommt links wieder herein. Das Fenster bleibt offen, bis du ESC drückst oder es schließt."),

  H.h2("Zeile für Zeile"),

  H.pmix([["WHILE ... WEND", true], " ist eine Schleife wie ", ["FOR ... NEXT", true], " — nur zählt sie nicht, sondern prüft eine Bedingung. Sie wiederholt alles zwischen ", ["WHILE", true], " und ", ["WEND", true], ", solange diese Bedingung zutrifft. Trifft sie irgendwann nicht mehr zu, geht es unterhalb des ", ["WEND", true], " weiter."]),

  H.pmix([["QUITREQUESTED()", true], " ist wahr, sobald jemand das Fenster schließen will — also auf das X oben rechts klickt. ", ["KEYPRESSED(KEY_ESCAPE)", true], " ist wahr, wenn die Taste ESC gedrückt wurde."]),

  H.pmix(["Davor steht jeweils ein ", ["NOT", true], ", und dazwischen ein ", ["AND", true], ". Zusammen liest sich die Zeile: „solange NICHT geschlossen werden soll UND NICHT ESC gedrückt wurde“. Also: solange keiner aufhören will, mach weiter."]),

  H.note("Diese Zeile wirst du in fast jedem Programm ab jetzt schreiben, Wort für Wort gleich. Lern sie nicht auswendig — schreib sie ab, so oft du sie brauchst. Nach dem zehnten Mal sitzt sie von selbst."),

  H.p("Der Rumpf der Schleife ist immer nach demselben Muster gebaut, und dieses Muster heißt die Spielschleife:"),

  H.bulletRich("Löschen. ", "CLS streicht das Fenster zu. Ohne diese Zeile bliebe das vorige Bild stehen."),
  H.bulletRich("Malen. ", "Alles, was zu sehen sein soll, wird neu gemalt — an den Stellen, wo es JETZT sein soll."),
  H.bulletRich("Zeigen. ", "FLIP klappt das fertige Bild nach vorn."),
  H.bulletRich("Rechnen. ", "Die Zahlen werden für das nächste Bild weitergestellt."),

  H.pmix(["Die vierte Zeile, ", ["x = (x + 3) MOD 640", true], ", ist die eigentliche Bewegung. Sie liest den alten Wert von x, zählt 3 dazu und legt das Ergebnis wieder in denselben Karton. Beim nächsten Durchgang steht der Kreis drei Punkte weiter rechts."]),

  H.pmix(["Das ", ["MOD 640", true], " sorgt für den Wiedereintritt links. MOD liefert den Rest beim Teilen: Solange x kleiner als 640 ist, ändert es nichts. Erreicht x aber 640, ist der Rest 0 — der Punkt springt an den linken Rand. Bei 641 wird er zu 1, und so fort."]),

  H.warn("Ein Gleichheitszeichen ist keine Behauptung. Die Zeile x = x + 3 wäre in der Mathematik unsinnig — keine Zahl ist um drei größer als sie selbst. Als Anweisung gelesen ergibt sie dagegen genau das Richtige: „nimm, was in x liegt, rechne drei dazu, und leg es zurück“. Wer bei dieser Zeile stolpert, ist in guter Gesellschaft; sie ist die berühmteste Hürde für Anfänger überhaupt.", "x = x + 3"),

  H.tip("Zwei Zahlen zum Drehen", "Ändere die 3 in eine 1 und in eine 20. Ändere die 640 in 300 — der Punkt springt dann schon in der Mitte zurück. Und dann nimm die CLS-Zeile weg und schau, was passiert. Die Antwort steht ein paar Seiten weiter."),

  H.h2("Etwas, das schneller wird"),

  H.p("Gleichmäßige Bewegung ist langweilig. Etwas, das fällt, wird schneller — und das schreibt sich fast von selbst hin, wenn man eine zweite Variable dazunimmt: nicht nur wo der Ball ist, sondern auch wie schnell er gerade ist."),

  H.code([
    'SCREEN(640, 400, "Fallender Ball")',
    "",
    "DIM y AS FLOAT",
    "DIM tempo AS FLOAT",
    "y = 40",
    "tempo = 0",
    "",
    "WHILE NOT QUITREQUESTED() AND NOT KEYPRESSED(KEY_ESCAPE)",
    "    CLS(RGB(15, 20, 40))",
    "    CIRCLE(320, y, 22, RGB(255, 90, 60))",
    "    FLIP()",
    "",
    "    tempo = tempo + 0.35",
    "    y = y + tempo",
    "WEND",
  ]),

  H.p("Zwei Zeilen unten machen die ganze Physik:"),

  H.bulletRich("tempo = tempo + 0.35 ", "— das Tempo wächst bei jedem Bild ein bisschen. Das ist die Schwerkraft."),
  H.bulletRich("y = y + tempo ", "— der Ball rückt um das aktuelle Tempo nach unten. Weil das Tempo wächst, wird der Schritt jedes Mal größer."),

  H.pmix(["Beide sind ", ["FLOAT", true], ", also Kommazahlen. Mit ganzen Zahlen ginge es nicht: Ein Tempo von 0,35 wäre gerundet 0, und der Ball bliebe für immer liegen."]),

  H.p("Der Ball fällt aus dem Bild und kommt nicht wieder — er hört ja nirgends auf zu fallen. Das Abprallen braucht eine Entscheidung, und die ist das Thema des nächsten Kapitels."),

  H.h2("Die Spur sichtbar machen"),

  H.p("„Er wird schneller“ ist leicht gesagt. Man kann es auch sehen — indem man alle Stationen des Falls auf einmal malt statt nacheinander:"),

  H.code([
    'SCREEN(640, 400, "Spur")',
    "",
    "DIM n AS INTEGER",
    "DIM x AS FLOAT",
    "DIM y AS FLOAT",
    "DIM tempo AS FLOAT",
    "",
    "WHILE NOT QUITREQUESTED() AND NOT KEYPRESSED(KEY_ESCAPE)",
    "    CLS(RGB(15, 20, 40))",
    "",
    "    x = 50",
    "    y = 30",
    "    tempo = 0",
    "    FOR n = 0 TO 24",
    "        CIRCLE(x, y, 12, RGB(255, 90, 60))",
    "        tempo = tempo + 0.9",
    "        y = y + tempo",
    "        x = x + 22",
    "    NEXT",
    "",
    "    FLIP()",
    "WEND",
  ]),

  H.figure("kap05_3_spur.png", "Fünfundzwanzig Stationen desselben Falls. Die Abstände nach unten werden größer — das ist die Beschleunigung.", 440, 280),

  H.p("Waagerecht liegen alle Kugeln gleich weit auseinander, denn x wächst jedes Mal um dieselben 22. Senkrecht dagegen rücken sie immer weiter auseinander. Genau das bedeutet „wird schneller“, und du siehst es hier auf einen Blick statt es glauben zu müssen."),

  H.p("Bemerkenswert ist der Aufbau: Innen steht eine FOR-Schleife, außen die WHILE-Schleife. Die innere malt in jedem Bild den ganzen Bogen von vorn — deshalb werden x, y und tempo direkt vor ihr wieder auf die Anfangswerte gesetzt. Ohne diese drei Zeilen wäre der Bogen nach dem ersten Bild verbraucht."),

  H.warn("Und hier die Antwort auf die Frage von vorhin: Wenn du CLS weglässt, bleibt KEIN Schweif stehen. In Drachenhauch beginnt jedes FLIP ein frisches Bild — was du siehst, ist immer nur das, was seit dem letzten FLIP gemalt wurde. Wer eine Spur will, muss sie absichtlich malen, so wie in diesem Programm. In manchen anderen Sprachen ist das anders, und alte Bücher raten deshalb gern zum Weglassen des CLS. Hier bekommst du damit nur ein flackerndes Nichts.", "Der Schweif, den es nicht gibt"),

  H.h2("Achtzig Tropfen"),

  H.p("Ein einzelner Punkt ist bescheiden. Mit einer Schleife in der Schleife wird daraus Regen:"),

  H.code([
    'SCREEN(640, 400, "Regen")',
    "",
    "DIM bild AS INTEGER",
    "DIM i AS INTEGER",
    "DIM x AS INTEGER",
    "DIM y AS INTEGER",
    "bild = 0",
    "",
    "WHILE NOT QUITREQUESTED() AND NOT KEYPRESSED(KEY_ESCAPE)",
    "    CLS(RGB(20, 25, 45))",
    "    FOR i = 0 TO 79",
    "        x = (i * 97) MOD 640",
    "        y = (i * 53 + bild * 7) MOD 440 - 40",
    "        LINE(x, y, x - 2, y + 14, RGB(150, 190, 255))",
    "        LINE(x + 1, y, x - 1, y + 14, RGB(150, 190, 255))",
    "    NEXT",
    "    FLIP()",
    "    bild = bild + 1",
    "WEND",
  ]),

  H.figure("kap05_4_regen.png", "Achtzig Tropfen, und kein einziger von ihnen ist irgendwo gespeichert.", 440, 280),

  H.p("Das Erstaunliche an diesem Programm ist, was NICHT darin steht: Nirgends merkt es sich, wo die achtzig Tropfen gerade sind. Es rechnet die Stellen bei jedem Bild neu aus."),

  H.pmix([["x = (i * 97) MOD 640", true], " streut die Tropfen über die Breite. Die 97 ist willkürlich gewählt — sie sorgt dafür, dass die Tropfen sich nicht in Reihen ordnen, sondern zerstreut wirken. Probier eine 100 aus, und du siehst plötzlich Muster."]),

  H.pmix([["y = (i * 53 + bild * 7) MOD 440 - 40", true], " ist die Fallbewegung. Der erste Teil verteilt die Tropfen über die Höhe, der zweite — ", ["bild * 7", true], " — schiebt alle gemeinsam nach unten, sieben Punkte je Bild. Das ", ["MOD 440", true], " lässt sie oben wieder anfangen, und das ", ["- 40", true], " am Ende schiebt den Wiedereintritt ein Stück über den oberen Rand, damit kein Tropfen mitten im Bild aus dem Nichts erscheint."]),

  H.pmix(["Der Karton ", ["bild", true], " zählt einfach mit, wie oft die Schleife schon gelaufen ist. Ein solcher Bildzähler ist ein außerordentlich nützliches Werkzeug: Alles, was sich gleichmäßig mit der Zeit ändern soll, kann sich an ihm festhalten."]),

  H.h2("Drei Monde"),

  H.p("Zum Schluss noch einmal die Uhrzeiger aus Kapitel 3 — diesmal drehen sie sich wirklich:"),

  H.code([
    'SCREEN(640, 400, "Monde")',
    "",
    "DIM winkel AS FLOAT",
    "DIM mx AS FLOAT",
    "DIM my AS FLOAT",
    "winkel = 0",
    "",
    "WHILE NOT QUITREQUESTED() AND NOT KEYPRESSED(KEY_ESCAPE)",
    "    CLS(RGB(10, 12, 28))",
    "",
    "    CIRCLEOUTLINE(320, 200, 90, RGB(40, 45, 70))",
    "    CIRCLEOUTLINE(320, 200, 140, RGB(40, 45, 70))",
    "    CIRCLEOUTLINE(320, 200, 185, RGB(40, 45, 70))",
    "    CIRCLE(320, 200, 40, RGB(255, 200, 60))",
    "",
    "    mx = 320 + COS(winkel) * 90",
    "    my = 200 + SIN(winkel) * 90",
    "    CIRCLE(mx, my, 12, RGB(120, 200, 255))",
    "    mx = 320 + COS(winkel * 0.6) * 140",
    "    my = 200 + SIN(winkel * 0.6) * 140",
    "    CIRCLE(mx, my, 9, RGB(255, 130, 160))",
    "    mx = 320 + COS(winkel * 0.35) * 185",
    "    my = 200 + SIN(winkel * 0.35) * 185",
    "    CIRCLE(mx, my, 6, RGB(180, 255, 190))",
    "",
    "    FLIP()",
    "    winkel = winkel + 0.04",
    "WEND",
  ]),

  H.figure("kap05_5_monde.png", "Ein Standbild verrät die Bewegung nicht — deshalb sind die Bahnen mitgemalt.", 440, 280),

  H.p("Alle drei Monde hängen an demselben Winkel; sie unterscheiden sich nur darin, mit welcher Zahl er multipliziert wird. Der innere bekommt den vollen Winkel und ist am schnellsten. Der mittlere bekommt 0,6 davon und braucht daher länger für eine Runde, der äußere mit 0,35 noch länger. Das entspricht sogar der Wirklichkeit: Weiter außen laufende Monde brauchen mehr Zeit."),

  H.p("Die drei blassen Ringe sind reine Zugabe fürs Buch. Auf dem laufenden Bildschirm braucht man sie nicht — man sieht ja, wohin die Monde ziehen. Auf einem gedruckten Standbild dagegen sähe man nur drei verstreute Punkte. Lass sie ruhig weg, wenn du das Programm für dich behältst."),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    ["Das Bild flackert wild", "FLIP steht in der Schleife an der falschen Stelle, oder es wird mehrmals aufgerufen. Reihenfolge: löschen, malen, FLIP."],
    ["Alles bleibt stehen", "Die Zeile, die weiterrechnet, steht außerhalb der Schleife — oder es fehlt das CLS und du siehst immer das erste Bild."],
    ["Das Fenster reagiert nicht mehr", "In der Schleife fehlt FLIP. Ohne FLIP kommt das Programm nie dazu, auf Tastatur und Fenster zu hören."],
    ["Der Ball bewegt sich nicht, obwohl gerechnet wird", "Die Variable ist INTEGER, und der Zuwachs ist kleiner als 1. Er wird auf 0 gerundet. FLOAT nehmen."],
    ["Das Programm lässt sich nicht beenden", "Die Bedingung im WHILE prüft nicht auf ESC oder auf QUITREQUESTED. Fenster notfalls über die Taskleiste schließen."],
    ["Alles rast viel zu schnell", "Die Schrittweite ist zu groß. Es wird sechzigmal je Sekunde gerechnet — drei Punkte je Bild sind schon 180 Punkte je Sekunde."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3400, 5626] }),

  H.h2("Aufgaben"),

  H.bullet("Lass den Wanderer nicht waagerecht, sondern schräg laufen — er braucht dafür ein zweites, mitwachsendes y."),
  H.bullet("Gib dem fallenden Ball einen Startschwung nach rechts, so dass er einen Bogen fliegt wie ein geworfener Stein."),
  H.bullet("Mach aus dem Regen Schnee: langsamer, runde Flocken statt Striche, und lass sie seitlich pendeln. Für das Pendeln hilft SIN mit dem Bildzähler."),
  H.bullet("Setz dem inneren Mond einen eigenen kleinen Mond an die Seite, der um IHN kreist. Du musst dafür seine Stelle in zwei Kartons zwischenspeichern."),
  H.bullet("Bau eine Uhr: ein großer Kreis, ein langer und ein kurzer Zeiger, die sich unterschiedlich schnell drehen."),
  H.bullet("Lass den Wanderer beim Verlassen des rechten Randes die Farbe wechseln. Tipp: Der Bildzähler und MOD reichen dafür aus."),

  H.p("Der fallende Ball fällt bisher aus dem Bild. Damit er abprallt, muss das Programm zum ersten Mal etwas entscheiden — und genau darum geht es im nächsten Kapitel."),
];
