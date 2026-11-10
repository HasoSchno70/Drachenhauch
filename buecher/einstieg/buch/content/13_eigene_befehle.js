module.exports = (H) => [
  H.part("Teil III — Ordnung schaffen"),
  H.chapter("Eigene Befehle"),

  H.p("Deine Programme sind länger geworden. Pong hat achtzig Zeilen, Snake fast hundert, das Instrument ebenfalls. Und an ein paar Stellen steht inzwischen fast dasselbe zweimal da."),

  H.p("In Pong etwa: zwei Blöcke für die Trefferprüfung, links und rechts, Zeichen für Zeichen gleich bis auf die Zahlen. Das ist nicht nur unschön. Wer die Regel ändern will — etwa den Abprallwinkel —, muss daran denken, es an beiden Stellen zu tun. Und irgendwann denkt niemand mehr daran."),

  H.p("Dieses Kapitel bringt das Mittel dagegen: Du darfst dir eigene Befehle bauen."),

  H.h2("Ein Baum"),

  H.code([
    "SUB baum(x AS INTEGER, y AS INTEGER, groesse AS INTEGER)",
    "    DIM halb AS FLOAT",
    "    halb = groesse / 10",
    "    BOX(x - halb, y - groesse, x + halb, y, RGB(95, 65, 45))",
    "    CIRCLE(x, y - groesse, groesse / 2, RGB(55, 145, 75))",
    "END SUB",
  ]),

  H.p("Das ist ein eigener Befehl. Ab jetzt gibt es neben CIRCLE, BOX und TEXT auch baum — und er wird genauso benutzt:"),

  H.code([
    "baum(320, 340, 140)",
  ]),

  H.figure("kap13_1_baum.png", "Ein Stamm, eine Krone. Und ein Befehl, den es vorher nicht gab.", 440, 280),

  H.h2("Zeile für Zeile"),

  H.pmix([["SUB", true], " leitet einen eigenen Befehl ein — das Wort kommt von „subroutine“, also Unterprogramm. Dahinter steht der Name, den du frei wählst, und in Klammern, was der Befehl an Angaben braucht."]),

  H.pmix([["x AS INTEGER", true], " ist so eine Angabe, ein Parameter. Sie sieht aus wie eine DIM-Zeile, und genau so verhält sie sich: Innerhalb des Befehls gibt es einen Karton namens ", ["x", true], ", und beim Aufruf legt jemand etwas hinein."]),

  H.pmix([["END SUB", true], " schließt den Befehl ab, so wie ", ["NEXT", true], " eine Schleife schließt. Alles dazwischen ist das, was er tut."]),

  H.pmix(["Der Aufruf ", ["baum(320, 340, 140)", true], " legt 320 in x, 340 in y und 140 in groesse — der Reihe nach. Danach laufen die beiden Malzeilen mit diesen Werten, und danach geht es hinter dem Aufruf weiter."]),

  H.p("Die zwei Malzeilen sind unspektakulär: ein schmales braunes Rechteck als Stamm, ein grüner Kreis als Krone. Bemerkenswert ist nur, dass beide mit groesse rechnen und nicht mit festen Zahlen — der Stamm ist ein Fünftel der Höhe breit, die Krone halb so groß wie der Baum hoch ist. Deshalb ist ein kleiner Baum kein abgeschnittener großer, sondern ein richtiger kleiner."),

  H.h2("Und jetzt sechsundzwanzig davon"),

  H.code([
    "FOR i = 0 TO 25",
    "    baum(RANDINT(10, 630), RANDINT(260, 395), RANDINT(35, 95))",
    "NEXT",
  ]),

  H.figure("kap13_2_wald.png", "Ein Wald. Drei Zeilen, wenn man den Baum schon hat.", 440, 280),

  H.p("Das ist der Moment, für den dieses Kapitel da ist. Sechsundzwanzig Bäume, jeder an anderer Stelle und in anderer Größe — und der Aufwand dafür war eine einzige Zeile, weil das Malen eines Baums schon woanders steht."),

  H.pmix(["Die y-Werte liegen zwischen 260 und 395, also über die Wiese verteilt. Dadurch stehen weiter unten gemalte Bäume weiter vorn und überdecken die dahinter. Ein Tiefeneindruck aus nichts als der Reihenfolge — Kapitel 3 hatte schon gesagt, dass später Gemaltes über früherem liegt."]),

  H.tip("Ein Baum, viele Wälder", "Ändere jetzt die zwei Zeilen im SUB — mach die Krone dreieckig, gib ihr eine zweite kleinere darüber, färb den Stamm anders. Alle sechsundzwanzig Bäume ändern sich mit. DAS ist der eigentliche Gewinn, nicht die gesparten Zeilen."),

  H.h2("Ein Befehl, der etwas zurückgibt"),

  H.p("Manchmal soll ein eigener Befehl nichts malen, sondern etwas ausrechnen und das Ergebnis herausgeben. Dafür gibt es die zweite Sorte:"),

  H.code([
    "FUNCTION abstand(x1 AS FLOAT, y1 AS FLOAT, _",
    "                 x2 AS FLOAT, y2 AS FLOAT) AS FLOAT",
    "    RETURN SQR((x2 - x1) * (x2 - x1) + (y2 - y1) * (y2 - y1))",
    "END FUNCTION",
  ]),

  H.pmix(["Der Unterstrich am Ende der ersten Zeile heißt: hier geht es unten weiter. Praktisch, wenn eine Aufzählung zu lang für eine Zeile wird — und in einem Buch mit schmalen Spalten öfter nötig als sonst."]),

  H.pmix(["Drei Unterschiede zum SUB. Es heißt ", ["FUNCTION", true], " statt ", ["SUB", true], ". Hinter der Klammer steht ", ["AS FLOAT", true], " — was für eine Sorte Wert herauskommt. Und drinnen steht ", ["RETURN", true], ": „das hier ist die Antwort, und hier ist Schluss“."]),

  H.p("Benutzt wird sie überall dort, wo sonst eine Zahl stünde:"),

  H.code([
    "DIM d AS FLOAT",
    "d = abstand(100, 100, 400, 300)",
  ]),

  H.p("Die Rechnung darin ist der Satz des Pythagoras: Abstand in x mal sich selbst, plus Abstand in y mal sich selbst, davon die Wurzel. Du musst ihn nicht können — du musst nur wissen, dass diese Zeile den Abstand zweier Punkte liefert. Genau das ist der Sinn eines eigenen Befehls: Man schreibt die Umständlichkeit einmal auf und benutzt danach den Namen."),

  H.figure("kap13_3_naehe.png", "160 Punkte, 160 Aufrufe von abstand. Was nah am blauen Punkt liegt, glüht.", 440, 280),

  H.code([
    "FOR sy = 0 TO 9",
    "    FOR sx = 0 TO 15",
    "        d = abstand(px, py, sx * 40 + 20, sy * 40 + 20)",
    "        h = INT(255 - d)",
    "        IF h < 30 THEN h = 30",
    "        CIRCLE(sx * 40 + 20, sy * 40 + 20, 12, RGB(h, h \\ 2, 40))",
    "    NEXT",
    "NEXT",
  ]),

  H.p("Für jeden der 160 Punkte wird der Abstand zum gesteuerten Punkt bestimmt, und daraus wird die Helligkeit: nah ist hell, fern ist dunkel. Die Zeile mit der 30 sorgt dafür, dass auch die entferntesten Punkte noch schwach zu sehen sind."),

  H.p("Steuere den blauen Punkt mit den Pfeiltasten, und das ganze Feld reagiert. Ein Effekt, der aufwendig aussieht und aus einer einzigen Formel besteht."),

  H.h2("Was drinnen bleibt und was heraussieht"),

  H.p("Ein eigener Befehl hat seinen eigenen kleinen Vorrat an Kartons. Das ist wichtig, und man sieht es am besten an einem Beispiel:"),

  H.code([
    "DIM zaehler AS INTEGER",
    "zaehler = 100",
    "",
    "SUB test()",
    "    DIM zaehler AS INTEGER",
    "    zaehler = 7",
    '    PRINT "drin:  " + STR$(zaehler)',
    "END SUB",
    "",
    "test()",
    'PRINT "draussen: " + STR$(zaehler)',
  ]),

  H.code(["drin:  7", "draussen: 100"], { out: true }),

  H.p("Zwei Kartons, derselbe Name, kein Zusammenhang. Der eine gehört dem Hauptprogramm, der andere dem Befehl — und wenn der Befehl fertig ist, wird seiner weggeräumt."),

  H.p("Umgekehrt darf ein Befehl aber lesen, was draußen steht, solange er es nicht selbst anlegt:"),

  H.code([
    "SUB liest()",
    '    PRINT "global gelesen: " + STR$(zaehler)',
    "END SUB",
  ]),

  H.code(["global gelesen: 100"], { out: true }),

  H.note("Ein Befehl, der nur mit seinen Parametern arbeitet, ist leichter zu verstehen als einer, der nebenbei an globalen Kartons herumfingert — denn man sieht ihm am Aufruf an, was er tut. Das ist kein Gesetz, sondern eine Gewohnheit, die sich auszahlt, sobald das Programm größer wird als ein Bildschirm."),

  H.h2("Pong, ein Stück kürzer"),

  H.p("Zurück zum Ärgernis vom Anfang. So sah die Trefferprüfung in Pong aus — zweimal, einmal für jede Seite:"),

  H.code([
    "IF bx - 9 < 42 AND bx - 9 > 30 THEN",
    "    IF by > links AND by < links + 80 THEN",
    "        bx = 51",
    "        dx = -dx",
    "    END IF",
    "END IF",
  ]),

  H.p("Die Frage „berührt der Ball diesen Schläger?“ lässt sich herausziehen. Sie ist eine Frage, also wird es eine FUNCTION, und die Antwort ist ja oder nein, also BOOLEAN:"),

  H.code([
    "FUNCTION trifft(x AS FLOAT, y AS FLOAT, kante AS FLOAT, _",
    "                dicke AS FLOAT, oben AS FLOAT) AS BOOLEAN",
    "    IF x < kante OR x > kante + dicke THEN RETURN FALSE",
    "    IF y < oben OR y > oben + 80 THEN RETURN FALSE",
    "    RETURN TRUE",
    "END FUNCTION",
  ]),

  H.p("Der Aufbau der Funktion ist eine Denkweise, die sich lohnt: Statt eine große Bedingung zu bauen, prüft sie der Reihe nach die Gründe, warum es KEIN Treffer sein kann. Wer bis zum Ende kommt, hat getroffen. Das liest sich wie eine Checkliste und ist leichter zu erweitern als ein Bandwurm aus AND."),

  H.p("Damit werden aus den zwölf Zeilen der beiden Blöcke acht:"),

  H.code([
    "IF trifft(bx - 9, by, 30, 12, links) THEN",
    "    bx = 51",
    "    dx = -dx",
    "END IF",
    "",
    "IF trifft(bx + 9, by, 598, 12, rechts) THEN",
    "    bx = 589",
    "    dx = -dx",
    "END IF",
  ]),

  H.p("Wichtiger als die vier gesparten Zeilen ist, dass die Regel jetzt an einer Stelle steht. Wer sie ändert, ändert beide Schläger."),

  H.tip("Nachgemessen", "Beide Fassungen wurden 300 Bilder lang laufen gelassen und die Bilder danach verglichen: Sie sind Punkt für Punkt identisch. Am Spiel hat sich nichts geändert — nur an seinem Text. Genau das soll ein Umbau leisten, und genau so prüft man ihn nach."),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    [{ text: "Unbekanntes Builtin 'BAUM'", mono: true }, "Der Befehl ist nicht definiert — meist ein Tippfehler im Namen, oder das END SUB fehlt und alles darunter gehört noch dazu."],
    [{ text: "Erwartet AS nach Parametername", mono: true }, "Bei Parametern ist die Typangabe Pflicht: (x AS INTEGER), nicht (x)."],
    ["Der Befehl malt nichts", "Er wird nie aufgerufen. Ein SUB, das nur dasteht, tut nichts — es muss beim Namen genannt werden."],
    ["Die Änderung im SUB wirkt nicht", "Es gibt ihn zweimal, mit demselben Namen. Der zweite gewinnt."],
    ["Draußen steht ein anderer Wert als drinnen", "Kein Fehler, sondern Absicht: Ein DIM im Befehl legt einen eigenen Karton an. Wer den äußeren ändern will, lässt das DIM weg oder nimmt BYREF."],
    [{ text: "Erwartet END FUNCTION", mono: true }, "Ein Block ist offen geblieben. Eigene Befehle dürfen nicht ineinander stehen."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3600, 5426] }),

  H.h2("Aufgaben"),

  H.bullet("Gib dem Baum einen vierten Parameter für die Farbe der Krone und mal einen Herbstwald."),
  H.bullet("Schreib einen Befehl haus(x, y, breite), der ein Haus mit Dach und Fenster malt. Bau daraus ein Dorf."),
  H.bullet("Schreib eine FUNCTION, die zwei Zahlen bekommt und die größere zurückgibt — ohne MAX zu benutzen."),
  H.bullet("Bau die vier Kästchen der Tastenanzeige aus Kapitel 7 zu einem Befehl lampe(x, y, taste) um."),
  H.bullet("Zieh aus Snake die Umrechnung von Spalte und Zeile in Bildpunkte heraus: zwei Funktionen bildx(spalte) und bildy(zeile)."),
  H.bullet("Schreib einen Befehl, der eine Zahl als Balken darstellt — und benutz ihn, um in einem Programm drei Werte nebeneinander anzuzeigen."),

  H.p("Eigene Befehle bündeln, was ein Programm TUT. Im nächsten Kapitel geht es um das, was es SICH MERKT — Arrays können mehr, als Kapitel 9 gebraucht hat."),
];
