module.exports = (H) => [
  H.chapter("Zusammenstoß"),

  H.p("Zwei Dinge bewegen sich über den Bildschirm, und irgendwann sind sie an derselben Stelle. Für dein Auge ist das offensichtlich. Für das Programm ist es das nicht — es weiß nur, welche Zahlen in welchen Kartons stehen."),

  H.p("Kollisionsprüfung heißt: aus diesen Zahlen die Frage zu beantworten, ob sich zwei Dinge berühren. Du hast das schon dreimal gemacht — beim Schläger in Pong, beim Kopf der Schlange, beim Schuss aufs Ziel. Jetzt sehen wir es uns richtig an."),

  H.h2("Wann berühren sich zwei Rechtecke?"),

  H.p("Die verblüffende Antwort: Man fragt nicht, wann sie sich berühren. Man fragt, wann sie es NICHT tun — und das ist viel einfacher."),

  H.p("Zwei Rechtecke berühren sich genau dann nicht, wenn eines der vier folgenden Dinge zutrifft: A ist ganz links von B. A ist ganz rechts von B. A ist ganz über B. A ist ganz unter B. Trifft keines davon zu, überlappen sie."),

  H.code([
    "FUNCTION ueberlappt(ax AS FLOAT, ay AS FLOAT, _",
    "                    ab AS FLOAT, ah AS FLOAT, _",
    "                    bx AS FLOAT, by AS FLOAT, _",
    "                    bb AS FLOAT, bh AS FLOAT) AS BOOLEAN",
    "    IF ax + ab < bx THEN RETURN FALSE",
    "    IF bx + bb < ax THEN RETURN FALSE",
    "    IF ay + ah < by THEN RETURN FALSE",
    "    IF by + bh < ay THEN RETURN FALSE",
    "    RETURN TRUE",
    "END FUNCTION",
  ]),

  H.figure("kap22_1_rechtecke.png", "Solange sie sich nicht berühren, ist das obere blau. Fahr es hinüber, und beide werden rot.", 440, 280),

  H.p("Vier Zeilen, vier Ausschlussgründe. Wer alle vier überlebt, hat getroffen. Diese Art zu denken — nicht die Bedingung suchen, sondern die Gegengründe abhaken — macht viele Prüfungen erstaunlich kurz, und man kann sie einzeln nachvollziehen."),

  H.pmix(["Die acht Parameter heißen ", ["ax, ay, ab, ah", true], " für das erste Rechteck (Stelle, Breite, Höhe) und ", ["bx, by, bb, bh", true], " für das zweite. Der Unterstrich am Zeilenende verteilt die lange Aufzählung auf vier Zeilen — sonst passte sie nicht auf diese Buchseite."]),

  H.tip("Der beste Versuch dieses Kapitels", "Starte das Programm und fahr das blaue Rechteck ganz langsam an das graue heran. Achte auf den Moment, in dem es umschlägt: Es genügt, dass sich die Kanten berühren. Ändere dann eines der vier < in ein <= und sieh, ob du den Unterschied bemerkst — das ist genau ein Punkt Unterschied, und darüber wird in Spielen erstaunlich viel diskutiert."),

  H.h2("Ein Spiel in zwanzig Zeilen"),

  H.p("Mit dieser einen Funktion und den Sprites aus Kapitel 19 lässt sich sofort etwas Spielbares bauen: sechs Münzen, ein Schiff, und wer eine berührt, sammelt sie ein."),

  H.code([
    "DIM mx[6] AS INTEGER",
    "DIM my[6] AS INTEGER",
    "DIM da[6] AS BOOLEAN",
    "",
    "FOR i = 0 TO 5",
    "    mx[i] = RANDINT(20, 580)",
    "    my[i] = RANDINT(20, 260)",
    "    da[i] = TRUE",
    "NEXT",
  ]),

  H.figure("kap22_2_einsammeln.png", "Sechs Münzen, ein Schiff, ein Zähler. Mehr braucht ein Spiel nicht, um eines zu sein.", 440, 280),

  H.pmix(["Drei parallele Arrays, wie du sie seit Kapitel 14 kennst: wo die Münze liegt und ob es sie noch gibt. Das ", ["da[i]", true], " ist wieder ein Schalter — eine eingesammelte Münze wird nicht gelöscht, sie wird nur nicht mehr gemalt und nicht mehr geprüft."]),

  H.code([
    "FOR i = 0 TO 5",
    "    IF da[i] THEN",
    "        IF ueberlappt(x + 6, y + 6, 20, 20, _",
    "                      mx[i] + 6, my[i] + 6, 20, 20) THEN",
    "            da[i] = FALSE",
    "            punkte = punkte + 1",
    "        END IF",
    "    END IF",
    "NEXT",
  ]),

  H.h2("Warum überall 6 und 20 steht"),

  H.p("Diese beiden Zahlen sind das eigentlich Lehrreiche an dem Programm. Die Sprites sind 32 mal 32 groß, aber die Figur darin ist kleiner — ringsum liegt durchsichtiger Rand, und das Schiff läuft nach oben spitz zu."),

  H.p("Würde man die vollen 32 Punkte prüfen, sammelte das Schiff Münzen ein, die es sichtbar gar nicht berührt hat. Das fühlt sich falsch an, und Spieler beschweren sich zu Recht. Also prüft man ein kleineres Rechteck: sechs Punkte vom Rand nach innen, zwanzig Punkte groß."),

  H.warn("Fast jedes Spiel prüft ein anderes Rechteck, als es zeichnet. Man nennt das die Trefferbox, und sie ist meist ETWAS KLEINER als die Figur. Der Grund ist nicht Genauigkeit, sondern Gefühl: Ein knapp verfehlter Treffer ärgert weniger als ein Treffer, den man nicht kommen sah. In vielen Spielen ist die Trefferbox des Spielers deshalb absichtlich winzig — das nennt man dann Kulanz und nicht Fehler.", "Die Trefferbox ist nicht das Bild"),

  H.h2("Runde Dinge"),

  H.p("Bei runden Dingen geht es noch einfacher, und dafür brauchst du nichts Neues — die Funktion aus Kapitel 13 reicht:"),

  H.code([
    "IF abstand(ax, ay, bx, by) < ra + rb THEN",
  ], { out: true }),

  H.p("Zwei Kreise berühren sich genau dann, wenn ihr Abstand kleiner ist als die Summe ihrer Radien. Eine Zeile, keine Sonderfälle, und für alles Kugelige die richtige Wahl — Bälle, Planeten, Funken, Blasen."),

  H.p("Rechtecke sind schneller zu prüfen und passen zu allem Kastenförmigen. Kreise sind kürzer hingeschrieben und wirken bei runden Dingen natürlicher. Beides ist richtig; die Frage ist nur, welche Form deiner Figur näher kommt."),

  H.h2("Wenn viele auf viele treffen"),

  H.p("In den Beispielen prüft ein Ding gegen sechs. Bei zwanzig Schüssen und dreißig Gegnern wären es sechshundert Prüfungen je Bild — und das ist noch völlig unproblematisch. Erst bei Tausenden lohnt sich Nachdenken."),

  H.note("Das Spiel des Lebens aus Kapitel 14 hat je Generation knapp 20 000 Nachbarschaftsprüfungen gemacht und lief mit vollen sechzig Bildern je Sekunde. Bevor du anfängst, Kollisionsprüfungen zu optimieren, miss nach, ob es überhaupt nötig ist. Meistens ist es das nicht."),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    ["Es wird nie ein Treffer erkannt", "Ein < zeigt in die falsche Richtung, oder Breite und Höhe sind vertauscht."],
    ["Es ist immer ein Treffer", "Die Funktion gibt am Anfang TRUE zurück, oder die vier Ausschlüsse fehlen."],
    ["Getroffen wird zu früh", "Es wird die volle Sprite-Größe geprüft statt der kleineren Trefferbox."],
    ["Eine Münze wird mehrfach gezählt", "Der Schalter wird nicht umgelegt — ohne da[i] = FALSE zählt dieselbe Berührung in jedem Bild neu."],
    ["Schnelle Dinge fliegen hindurch", "Das Durchtunneln aus Kapitel 8: Zwischen zwei Bildern war das Ding schon vorbei."],
    ["Die Prüfung greift an der falschen Stelle", "x und y eines Bildes sind die linke obere Ecke, bei CIRCLE aber die Mitte. Nicht mischen."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3400, 5626] }),

  H.h2("Aufgaben"),

  H.bullet("Male im Einsammel-Spiel die Trefferboxen als dünne Rahmen mit ein. Du siehst dann genau, was das Programm prüft."),
  H.bullet("Lass eine eingesammelte Münze an einer neuen Stelle wieder auftauchen, statt zu verschwinden."),
  H.bullet("Gib jeder Münze einen Wert zwischen eins und fünf und zeig ihn als Zahl daneben."),
  H.bullet("Bau einen Gegner ein, der das Schiff verfolgt. Berührt er es, ist das Spiel vorbei."),
  H.bullet("Setz die Kreisprüfung ein: Lass die Münzen als Kreise gelten und vergleiche, ob sich das Einsammeln anders anfühlt."),
  H.bullet("Zeig an, wie viele Prüfungen je Bild stattfinden — bei sechs Münzen sind es sechs. Bau auf sechzig aus und sieh nach, ob es langsamer wird."),

  H.p("Du hast jetzt alles für ein richtiges Spiel: Figuren, die du selbst gemalt hast, Bewegung, Animation, Steuerung, Klang und Kollision. Im nächsten Kapitel setzen wir es zusammen."),
];
