module.exports = (H) => [
  H.chapter("Projekt: Pong"),

  H.p("Du hast jetzt alles beisammen. Ein Fenster, Variablen, Schleifen, Zufall, Bewegung, Entscheidungen, Tastatur — sieben Kapitel, und keines davon war für sich genommen schwierig. In diesem Kapitel setzen wir sie zum ersten Mal vollständig zusammen."),

  H.p("Pong ist von 1972 und war eines der ersten Videospiele überhaupt. Zwei Striche, ein Punkt, zwei Zahlen. Es ist bis heute zu zweit erstaunlich spaßig, und es passt in achtzig Zeilen."),

  H.p("Wir bauen es in vier Stufen. Nach jeder Stufe läuft etwas — das ist keine Bequemlichkeit, sondern die wichtigste Arbeitsweise, die du aus diesem Buch mitnehmen kannst. Wer erst ganz zum Schluss startet, sucht seinen Fehler in achtzig Zeilen auf einmal."),

  H.h2("Stufe 1: das Feld"),

  H.code([
    'SCREEN(640, 400, "Pong")',
    "",
    "DIM i AS INTEGER",
    "DIM weiss AS INTEGER",
    "weiss = RGB(230, 235, 245)",
    "",
    "WHILE NOT QUITREQUESTED() AND NOT KEYPRESSED(KEY_ESCAPE)",
    "    CLS(RGB(12, 16, 30))",
    "",
    "    FOR i = 0 TO 380 STEP 24",
    "        BOX(318, i, 322, i + 12, RGB(60, 70, 95))",
    "    NEXT",
    "",
    "    BOX(30, 160, 42, 240, weiss)",
    "    BOX(598, 160, 610, 240, weiss)",
    "",
    "    FLIP()",
    "WEND",
  ]),

  H.figure("kap08_1_feld.png", "Noch bewegt sich nichts. Aber es sieht schon aus wie Pong.", 440, 280),

  H.p("Die Mittellinie ist gestrichelt, und dafür braucht es keinen besonderen Befehl: Eine Schleife setzt alle 24 Punkte ein zwölf Punkte hohes Kästchen. Zwölf gemalt, zwölf ausgelassen — fertig ist die Strichelung."),

  H.p("Die beiden Schläger sind schlicht Rechtecke, zwölf Punkte breit und achtzig hoch. Sie stehen noch fest."),

  H.h2("Stufe 2: die Schläger bewegen sich"),

  H.p("Jetzt bekommen beide Seiten ihre Tasten. Der linke Spieler nimmt W und S, der rechte die Pfeiltasten — so sitzen beide bequem an einer Tastatur."),

  H.p("Die Höhe der Schläger wandert dafür in zwei Kartons, und die festen Zahlen in den Malzeilen werden durch sie ersetzt:"),

  H.code([
    "DIM links AS FLOAT",
    "DIM rechts AS FLOAT",
    "links = 160",
    "rechts = 160",
  ]),

  H.code([
    "BOX(30, links, 42, links + 80, weiss)",
    "BOX(598, rechts, 610, rechts + 80, weiss)",
  ]),

  H.p("Und hinter das FLIP kommen die acht Zeilen, die du aus Kapitel 7 kennst — vier fürs Bewegen, vier fürs Begrenzen:"),

  H.code([
    "IF KEYPRESSED(KEY_W) THEN links = links - 6",
    "IF KEYPRESSED(KEY_S) THEN links = links + 6",
    "IF KEYPRESSED(KEY_UP) THEN rechts = rechts - 6",
    "IF KEYPRESSED(KEY_DOWN) THEN rechts = rechts + 6",
    "",
    "IF links < 0 THEN links = 0",
    "IF links > 320 THEN links = 320",
    "IF rechts < 0 THEN rechts = 0",
    "IF rechts > 320 THEN rechts = 320",
  ]),

  H.pmix(["Die 320 ist kein Zufall: Das Fenster ist 400 hoch, der Schläger 80 — also darf seine obere Kante höchstens bei 320 stehen, sonst hinge er unten heraus. Solche Zahlen rechnet man einmal im Kopf aus und schreibt sie hin. Später, wenn du eigene Befehle schreiben kannst, schreibst du stattdessen ", ["hoehe - schlaeger_hoehe", true], " und musst nie wieder nachdenken."]),

  H.tip("Jetzt schon ausprobieren", "Ruf jemanden dazu und bewegt beide Schläger gleichzeitig. Es passiert nichts weiter — und trotzdem ist der Moment, in dem zwei Menschen dasselbe Programm gleichzeitig bedienen, ein anderer als alles davor."),

  H.h2("Stufe 3: der Ball"),

  H.p("Der Ball braucht vier Kartons: wo er ist und wohin er sich bewegt. Genau wie der Ball im Kasten aus Kapitel 6."),

  H.code([
    "DIM bx AS FLOAT",
    "DIM by AS FLOAT",
    "DIM dx AS FLOAT",
    "DIM dy AS FLOAT",
    "bx = 320",
    "by = 200",
    "dx = 5",
    "dy = 3",
  ]),

  H.p("Gemalt wird er als Kreis, und bewegt wird er ganz unten in der Schleife:"),

  H.code([
    "CIRCLE(bx, by, 9, RGB(255, 210, 70))",
  ]),

  H.code([
    "bx = bx + dx",
    "by = by + dy",
    "IF by < 9 OR by > 390 THEN dy = -dy",
  ]),

  H.figure("kap08_3_ball.png", "Der Ball fliegt — und läuft seitlich noch aus dem Bild.", 440, 280),

  H.p("Oben und unten prallt er ab. Links und rechts fliegt er hinaus und kommt nicht wieder. Das ist Absicht: In Pong ist genau das ein Punkt für den Gegner, und darum kümmern wir uns in der letzten Stufe."),

  H.h2("Stufe 4: der Ball trifft die Schläger"),

  H.p("Das ist der Kern des Spiels und die einzige wirklich kniffelige Stelle. Die Frage lautet: Berührt der Ball gerade den linken Schläger?"),

  H.p("Dafür müssen vier Dinge gleichzeitig zutreffen. Wir prüfen sie in zwei Schritten: erst waagerecht, dann senkrecht. Zwei ineinandergeschachtelte Entscheidungen sind hier lesbarer als eine lange Zeile mit vier Bedingungen:"),

  H.code([
    "IF bx - 9 < 42 AND bx - 9 > 30 THEN",
    "    IF by > links AND by < links + 80 THEN",
    "        bx = 51",
    "        dx = -dx",
    "    END IF",
    "END IF",
  ]),

  H.figure("kap08_4_treffer.png", "Jetzt kommt der Ball zurück.", 440, 280),

  H.pmix([["bx - 9", true], " ist die linke Kante des Balls; die 9 ist sein Radius. Sie muss hinter der rechten Schlägerkante (42) liegen, aber noch vor der linken (30) — sonst hätte der Ball den Schläger längst durchquert."]),

  H.pmix([["by > links AND by < links + 80", true], " prüft die Höhe. ", ["links", true], " ist die Oberkante des Schlägers, ", ["links + 80", true], " die Unterkante. Liegt der Ballmittelpunkt dazwischen, sitzt der Treffer."]),

  H.pmix([["bx = 51", true], " schiebt den Ball auf die Schlägerkante zurück — dieselbe Korrektur wie beim abprallenden Ball in Kapitel 6, und aus demselben Grund. Ohne sie bliebe der Ball im Schläger stecken und würde bei jedem Bild erneut umgedreht: das Zittern aus dem Warnkasten."]),

  H.pmix([["dx = -dx", true], " ist der Abprall selbst. Wieder nur ein Minuszeichen."]),

  H.p("Für den rechten Schläger steht dasselbe noch einmal da, mit anderen Zahlen und mit rechts statt links. Zwei fast gleiche Blöcke nebeneinander sind unschön — merk dir das Gefühl. In Kapitel 12 lernst du, wie man daraus einen einzigen Befehl macht, den man zweimal aufruft."),

  H.warn("Diese Trefferprüfung hat eine Lücke, und die ist berühmt. Der Ball springt fünf Punkte je Bild. Wäre er sehr viel schneller — sagen wir dreißig —, könnte er in einem einzigen Schritt vom Vor-dem-Schläger zum Hinter-dem-Schläger springen, ohne je die Bedingung zu erfüllen. Er fliegt dann glatt durch. Bei fünf Punkten und zwölf Punkten Schlägerbreite kann das nicht passieren; sobald du das Spiel schneller machst, schon. Man nennt das Durchtunneln, und es ist der häufigste Fehler in selbstgebauten Spielen.", "Warum der Ball manchmal durch die Wand geht"),

  H.h2("Stufe 5: Punkte"),

  H.p("Fehlt nur noch, was aus einem Zeitvertreib ein Spiel macht: Wenn der Ball hinausfliegt, bekommt der andere einen Punkt, und der Ball beginnt in der Mitte von vorn."),

  H.code([
    "IF bx < -20 THEN",
    "    punkte_rechts = punkte_rechts + 1",
    "    bx = 320",
    "    by = 200",
    "    dx = 5",
    "END IF",
    "",
    "IF bx > 660 THEN",
    "    punkte_links = punkte_links + 1",
    "    bx = 320",
    "    by = 200",
    "    dx = -5",
    "END IF",
  ]),

  H.pmix(["Die ", ["-20", true], " und ", ["660", true], " liegen absichtlich außerhalb des Fensters: So sieht man den Ball noch kurz hinausfliegen, statt dass er am Rand verschwindet. Eine winzige Sache, die den Unterschied zwischen „funktioniert“ und „fühlt sich richtig an“ ausmacht."]),

  H.pmix([["dx = 5", true], " beziehungsweise ", ["dx = -5", true], " schickt den Ball zu dem, der gerade den Punkt kassiert hat. Das ist die übliche Regel und wirkt fair."]),

  H.p("Angezeigt werden die Punkte mit großer Schrift:"),

  H.code([
    "TEXT_SIZE(40)",
  ]),

  H.code([
    "TEXT(250, 24, STR$(punkte_links), weiss)",
    "TEXT(370, 24, STR$(punkte_rechts), weiss)",
  ]),

  H.pmix([["TEXT_SIZE", true], " stellt die Schriftgröße für alles ein, was danach mit ", ["TEXT", true], " gemalt wird — bis zum nächsten ", ["TEXT_SIZE", true], ". Die Zeile gehört deshalb vor die Schleife und nicht hinein."]),

  H.h2("Das fertige Spiel"),

  H.figure("kap08_pong.png", "Zwei zu zwei. Das Bild entstand ohne Spieler — deshalb ist der Ball zweimal an beiden Schlägern vorbeigekommen.", 440, 280),

  H.code([
    "' Pong. Linker Spieler: W und S. Rechter: Pfeil hoch und runter.",
    "",
    'SCREEN(640, 400, "Pong")',
    "",
    "DIM i AS INTEGER",
    "DIM weiss AS INTEGER",
    "DIM links AS FLOAT",
    "DIM rechts AS FLOAT",
    "DIM bx AS FLOAT",
    "DIM by AS FLOAT",
    "DIM dx AS FLOAT",
    "DIM dy AS FLOAT",
    "DIM punkte_links AS INTEGER",
    "DIM punkte_rechts AS INTEGER",
    "",
    "weiss = RGB(230, 235, 245)",
    "links = 160",
    "rechts = 160",
    "bx = 320",
    "by = 200",
    "dx = 5",
    "dy = 3",
    "punkte_links = 0",
    "punkte_rechts = 0",
    "TEXT_SIZE(40)",
    "",
    "WHILE NOT QUITREQUESTED() AND NOT KEYPRESSED(KEY_ESCAPE)",
    "    CLS(RGB(12, 16, 30))",
    "",
    "    FOR i = 0 TO 380 STEP 24",
    "        BOX(318, i, 322, i + 12, RGB(60, 70, 95))",
    "    NEXT",
    "",
    "    BOX(30, links, 42, links + 80, weiss)",
    "    BOX(598, rechts, 610, rechts + 80, weiss)",
    "    CIRCLE(bx, by, 9, RGB(255, 210, 70))",
    "",
    "    TEXT(250, 24, STR$(punkte_links), weiss)",
    "    TEXT(370, 24, STR$(punkte_rechts), weiss)",
    "    FLIP()",
    "",
    "    IF KEYPRESSED(KEY_W) THEN links = links - 6",
    "    IF KEYPRESSED(KEY_S) THEN links = links + 6",
    "    IF KEYPRESSED(KEY_UP) THEN rechts = rechts - 6",
    "    IF KEYPRESSED(KEY_DOWN) THEN rechts = rechts + 6",
    "",
    "    IF links < 0 THEN links = 0",
    "    IF links > 320 THEN links = 320",
    "    IF rechts < 0 THEN rechts = 0",
    "    IF rechts > 320 THEN rechts = 320",
    "",
    "    bx = bx + dx",
    "    by = by + dy",
    "    IF by < 9 OR by > 390 THEN dy = -dy",
    "",
    "    IF bx - 9 < 42 AND bx - 9 > 30 THEN",
    "        IF by > links AND by < links + 80 THEN",
    "            bx = 51",
    "            dx = -dx",
    "        END IF",
    "    END IF",
    "",
    "    IF bx + 9 > 598 AND bx + 9 < 610 THEN",
    "        IF by > rechts AND by < rechts + 80 THEN",
    "            bx = 589",
    "            dx = -dx",
    "        END IF",
    "    END IF",
    "",
    "    IF bx < -20 THEN",
    "        punkte_rechts = punkte_rechts + 1",
    "        bx = 320",
    "        by = 200",
    "        dx = 5",
    "    END IF",
    "",
    "    IF bx > 660 THEN",
    "        punkte_links = punkte_links + 1",
    "        bx = 320",
    "        by = 200",
    "        dx = -5",
    "    END IF",
    "WEND",
  ]),

  H.h2("Was hier eigentlich passiert ist"),

  H.p("Sieh dir das Programm noch einmal als Ganzes an. Es enthält keinen einzigen Befehl, den du nicht schon kanntest. Kein neues Kapitel Sprache, keine geheime Technik — nur Dinge aus den letzten sieben Kapiteln, in einer bestimmten Reihenfolge."),

  H.p("Das ist die eigentliche Nachricht dieses Kapitels: Programmieren besteht nicht darin, immer mehr Befehle zu kennen. Es besteht darin, die wenigen, die man kennt, richtig zusammenzusetzen."),

  H.p("Und der Aufbau der Schleife ist derselbe wie in Kapitel 5 — löschen, malen, zeigen, rechnen. Bei achtzig Zeilen sieht man ihn nur nicht mehr auf einen Blick. Genau deshalb stehen die vier Abschnitte durch Leerzeilen getrennt da."),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    ["Der Ball bleibt am Schläger kleben", "Die Rückschiebe-Zeile (bx = 51) fehlt. Ohne sie steckt der Ball im Schläger und wird jedes Bild neu umgedreht."],
    ["Der Ball fliegt durch den Schläger", "Er ist zu schnell für die Prüfung — siehe den Warnkasten. Oder eine der vier Bedingungen hat die falsche Zahl."],
    ["Nur ein Schläger bewegt sich", "Die Tastenabfragen stehen in einer ELSEIF-Kette statt in vier einzelnen IF."],
    ["Der Punktestand springt um zwei", "Der Ball wird nicht in die Mitte zurückgesetzt, ist also im nächsten Bild immer noch draußen — und zählt erneut."],
    ["Die Punkte stehen winzig da", "TEXT_SIZE fehlt, oder es steht nach den TEXT-Zeilen statt davor."],
    ["Es ruckelt beim Tippen", "Nicht das Programm, sondern die Tastatur: Viele Tastaturen melden nicht mehr als sechs gleichzeitig gedrückte Tasten zuverlässig."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3400, 5626] }),

  H.h2("Aufgaben"),

  H.bullet("Lass den Ball nach jedem Schlägertreffer ein wenig schneller werden. Ein Spiel, das anzieht, ist deutlich spannender."),
  H.bullet("Mach den Abprallwinkel davon abhängig, WO der Ball den Schläger trifft: an der Kante steiler, in der Mitte flacher. Das ist die wichtigste Regel des echten Pong, und sie steckt in einer einzigen Zeile."),
  H.bullet("Lass das Spiel bei elf Punkten enden und den Sieger anzeigen."),
  H.bullet("Bau einen Ton bei jedem Treffer ein — dafür brauchst du Teil II, aber schau ruhig vor."),
  H.bullet("Ersetze den rechten Spieler durch den Rechner: Der Schläger folgt einfach der Höhe des Balls. Gib ihm ein Tempolimit, sonst gewinnt er immer."),
  H.bullet("Male eine dünne Linie dort, wo der Ball zuletzt entlanggeflogen ist — die Spur aus Kapitel 5 als Nachziehschweif."),

  H.p("Das war das erste vollständige Spiel. Im nächsten Kapitel bauen wir ein zweites, und dafür brauchst du zum ersten Mal seit Kapitel 2 etwas wirklich Neues: einen Behälter für viele Dinge auf einmal."),
];
