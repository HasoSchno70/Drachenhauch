const { zeilen, ohneKopf, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Das Ziffernblatt", 4),

  H.h2("Was wir bauen"),

  H.figure("kap04_1_zehn_nach_zehn.png", "Das Ziffernblatt um zehn nach zehn — die Zeit, die Uhren in der Werbung zeigen, weil die Zeiger das Logo darunter einrahmen.", 460, 290),

  H.p("Eine Uhr mit Zifferblatt, sechzig Strichen, zwölf Zahlen und drei Zeigern, die die Zeit des Rechners zeigt. Der rote Sekundenzeiger springt jede Sekunde weiter, aber nicht schlagartig: Er gleitet in einer Fünftelsekunde zur nächsten Sekunde, wie bei einer Bahnhofsuhr."),

  H.p("In diesem Kapitel geht es um Ordnung: um Konstanten für alles, was feststeht, um Unterprogramme, die eine Uhr zu jeder beliebigen Zeit zeichnen können, und um eine eigene Datei, die man mit IMPORT in mehrere Programme holt."),

  H.h2("Der Plan"),

  H.bulletRich("Das Zifferblatt ", "ist ein Kreis mit Strichen und Zahlen. Es sieht immer gleich aus."),
  H.bulletRich("Ein Zeiger ", "ist eine Linie von der Mitte nach außen, gedreht um einen Winkel. Alle drei Zeiger sind dieselbe Rechnung mit anderer Länge, Dicke und Farbe."),
  H.bulletRich("Die Zeit ", "bestimmt nur die Winkel: 30 Grad je Stunde, 6 Grad je Minute, 6 Grad je Sekunde."),

  H.p("Daraus folgt eine Trennung: Ein Teil des Programms zeichnet eine Uhr, die man ihm sagt; ein anderer liest die Uhrzeit und sagt sie ihm. Der erste Teil kommt in eine eigene Datei."),

  H.h2("Schritt 1: die Teile der Uhr"),

  H.p("Die Datei uhr_teile.dh beginnt mit Konstanten — Werten, die einen Namen haben und sich nie ändern:"),

  H.code(stueck("kap04/uhr_teile.dh", "CONST UHR_X", "CONST SIGNAL")),

  H.pmix(["Die Farben stehen als ", ["&H", true], "-Zahlen da: sechs Stellen im Sechzehnersystem, je zwei für Rot, Grün und Blau. ", ["&H1C2233", true], " ist ein dunkles Blaugrau, ", ["&HE04040", true], " ein kräftiges Rot. Benannt sind sie nach ihrer Aufgabe, nicht nach ihrem Ton: BLATT, RAND, TINTE, SIGNAL. Wer der Uhr ein helles Zifferblatt geben will, ändert zwei Zeilen und muss nicht suchen, wo überall „das Dunkelblau“ benutzt wird."]),

  H.p("Dann ein Zeiger. Er bekommt die Mitte, einen Winkel in Grad, eine Länge, eine Dicke und eine Farbe:"),

  H.code(stueck("kap04/uhr_teile.dh", "SUB zeiger", "END SUB")),

  H.figure("kap04_winkel.png", "Wo 0 Grad ist: Auf der Uhr oben, für SIN und COS rechts. Gedreht wird in beiden Fällen im Uhrzeigersinn, weil y auf dem Bildschirm nach unten wächst.", 460, 255),

  H.pmix(["Das ", ["- 90", true], " in ", ["RAD(grad - 90)", true], " ist die Vierteldrehung zwischen den beiden Welten. Auf der Uhr ist 0 Grad oben, bei SIN und COS rechts. Wer 0 Grad hineingibt, will den Zeiger auf der 12 sehen; RAD(0 - 90) zeigt nach oben. Weil y nach unten wächst, dreht ein wachsender Winkel auf dem Bildschirm im Uhrzeigersinn — genau wie auf der Uhr. Ein Glück, und kein Zufall: Die y-Achse eines Bildschirms zeigt nach unten, weil die Zeilen eines Fernsehbildes von oben nach unten geschrieben wurden."]),

  H.p("Das Zifferblatt malt zwei Kreise, sechzig Striche und zwölf Zahlen:"),

  H.code(stueck("kap04/uhr_teile.dh", "SUB zifferblatt", "END SUB")),

  H.pmix([["IIF(bedingung, a, b)", true], " ist ein IF als Ausdruck: Es liefert a, wenn die Bedingung stimmt, sonst b. Jeder fünfte Strich beginnt weiter innen und ist dicker. Für die Zahlen setzt ", ["TEXT_SIZE", true], " die Schriftgröße passend zum Radius. TEXT schreibt von der linken oberen Ecke aus; damit eine Zahl mittig auf ihrem Platz sitzt, zieht das Programm die halbe Breite ab, die ", ["TEXT_WIDTH", true], " misst. Bei 10, 11 und 12 ist die Breite doppelt so groß wie bei den anderen — ohne die Messung säßen sie schief."]),

  H.p("Zum Schluss setzen zwei Unterprogramme alles zusammen. alleZeiger rechnet die Winkel aus, uhrZeichnen malt erst das Blatt und dann die Zeiger:"),

  H.code(stueck("kap04/uhr_teile.dh", "' Die drei Zeiger", "END SUB")),
  H.code(stueck("kap04/uhr_teile.dh", "' Die ganze Uhr", "END SUB")),

  H.pmix(["Der Stundenzeiger rückt mit den Minuten weiter: um halb elf steht er zwischen 10 und 11, nicht auf der 10. Darum ", ["(stunde MOD 12 + minute / 60) * 30", true], ". MOD 12 macht aus 22 Uhr eine 10. uhrZeichnen hat Vorgaben für Mitte und Radius, sodass ", ["uhrZeichnen(10, 10, 30)", true], " eine große Uhr in der Mitte malt — und ", ["uhrZeichnen(h, m, s, mx: 140, my: 230, r: 115)", true], " eine kleine woanders."]),

  H.p("Die Datei hat keine Schleife und kein SCREEN; sie zeichnet nur, wenn man sie fragt. Ein Programm holt sie mit IMPORT herein:"),

  H.code(ohneKopf("kap04/1_zehn_nach_zehn.dh")),

  H.pmix([["IMPORT \"uhr_teile.dh\"", true], " fügt die Datei an dieser Stelle ein, als stünde ihr Text da. Der Name ist ein Pfad, gerechnet vom Ordner des Programms, das man startet; liegen beide im selben Ordner, genügt der Name. Die Lösungen im Anhang liegen in einem anderen Ordner und schreiben darum IMPORT \"../kap04/uhr_teile.dh\". Das Bild oben am Kapitel ist genau dieses Programm."]),

  H.h2("Schritt 2: die echte Zeit"),

  H.p("Jetzt bekommt die Uhr die Zeit des Rechners. Das Modul time liefert sie auf mehrere Arten; für eine Uhr mit gleitendem Sekundenzeiger braucht man die Sekunden mit Bruchteil:"),

  H.code(stueck("kap04/2_uhr.dh", "' Sekunden seit 1970", "uhrZeichnen(std, mnt, sek)")),

  H.pmix([["TIME_UNIX()", true], " liefert die Sekunden seit dem 1. Januar 1970, eine große Kommazahl wie 1795632482,37. ", ["TIME_FROM_UNIX", true], " macht aus dem ganzen Teil einen Zeitpunkt in der Ortszeit, samt Sommerzeit; ", ["TIME_PART(zeit, \"stunde\")", true], " holt daraus Stunde, Minute oder Sekunde. Der Bruchteil — was nach dem Komma steht — sagt, wie weit die laufende Sekunde schon ist."]),

  H.p("In der ersten Fünftelsekunde jeder Sekunde (SPRUNG = 0,2) fährt der Zeiger von der vorigen Sekunde zur neuen; den Rest der Sekunde steht er still. Wie er fährt, bestimmt eine kleine Funktion:"),

  H.code(stueck("kap04/2_uhr.dh", "FUNCTION weich", "END FUNCTION")),

  H.p("Für x von 0 bis 1 liefert sie ebenfalls 0 bis 1, aber nicht gleichmäßig: Am Anfang und am Ende ändert sich der Wert kaum, in der Mitte schnell. Der Zeiger fährt also sanft an, eilt und bremst sanft — so bewegen sich Dinge, die eine Masse haben. Diese Kurve heißt in der Grafik Smoothstep und begegnet uns in Kapitel 11 wieder, wenn es um Übergänge geht. Das ganze Programm:"),

  H.code(zeilen("kap04/2_uhr.dh")),

  H.h2("Warum so?"),

  H.h3("Konstanten für alles, was feststeht"),

  H.p("Mitte, Radius und Farben ändern sich nie, während das Programm läuft. Als Konstanten haben sie einen Namen, der sagt, was sie sind, und sie stehen an einer Stelle. Wer die Uhr kleiner will, ändert UHR_R, und alles wächst mit: Striche, Zahlen, Zeiger, denn alle rechnen mit r. Und Drachenhauch schützt Konstanten: Eine Zuweisung an eine Konstante ist ein Fehler, keine stille Änderung."),

  H.h3("Konstanten in Großbuchstaben"),

  H.p("Drachenhauch unterscheidet Groß- und Kleinschreibung nicht. RADIUS, Radius und radius sind derselbe Name. Dass Konstanten in diesem Buch trotzdem groß geschrieben werden, hat einen anderen Grund: Man sieht ihnen an, dass sie Konstanten sind. Eine Zeile wie r * 0.13 rechnet mit etwas Veränderlichem, UHR_R * 0.13 mit etwas Festem. Und weil die Schreibweise nichts unterscheidet, darf es keinen zweiten Namen geben, der nur anders geschrieben ist — eine Variable tempo neben der Konstanten TEMPO ist ein Fehler (unten), ein Parameter radius neben RADIUS eine Falle (Kapitel 3). Die Großbuchstaben erinnern daran."),

  H.h3("Zeichnen und Zeit trennen"),

  H.p("uhrZeichnen weiß nichts von der Uhrzeit; es zeichnet, was man ihm sagt. Dadurch geht dreierlei, was mit einer Uhr, die sich die Zeit selbst holt, nicht ginge: ein Bild für dieses Buch, das bei jedem Bau zehn nach zehn zeigt; eine Stoppuhr, deren Zeiger eine gemessene Zeit zeigen statt der Uhrzeit; und drei Uhren für drei Städte. Die Lösungen im Anhang benutzen dieselbe Datei. Eine Datei, die etwas gut kann und von mehreren Programmen benutzt wird, ist der Anfang einer Bibliothek."),

  H.h3("Eine Quelle für die Zeit"),

  H.pmix(["Man könnte die Sekunde mit ", ["TIME_PART(TIME_NOW(), \"sekunde\")", true], " holen und den Bruchteil getrennt mit ", ["TIME_UNIX()", true], ". Aber zwischen den beiden Aufrufen kann die Sekunde wechseln, und dann passen sie nicht zusammen: Für ein Bild stünde der Zeiger eine Sekunde zu weit. Darum fragt das Programm einmal, nimmt beides aus derselben Zahl und kann sich nicht widersprechen."]),

  H.h2("Was schiefgehen kann"),

  H.h3("Eine Konstante ändern"),

  H.code(zeilen("kap04/fehler_const.dh")),
  H.code(["fehler_const.dh:3: Compile-Fehler: Zuweisung an die Konstante",
          "'radius' -- eine CONST behaelt ihren Wert. Was sich aendern soll,",
          "ist eine Variable (DIM)."], { out: true }),

  H.p("Der Fehler kommt vor dem Start; auch das erste PRINT läuft nicht, und in der IDE ist die Zeile schon beim Tippen unterstrichen. Wer eine Konstante ändern will, will in Wahrheit eine Variable. Ausgenommen ist nur ein Name, der in einer Funktion als Parameter oder lokale Variable die Konstante verdeckt (Kapitel 3) — dort meint er die Variable, und die darf sich ändern."),

  H.h3("Zwei Namen, die einer sind"),

  H.code(zeilen("kap04/fehler_kollision.dh")),
  H.code(["fehler_kollision.dh:2: Compile-Fehler: Namens-Kollision: 'tempo' ist",
          "zugleich eine Variable und eine CONST -- das geht nicht, weil",
          "Drachenhauch Gross-/Kleinschreibung ignoriert. Benenne eines um."], { out: true }),

  H.p("Dieser Fehler kommt vor dem Start. Er ist in Kapitel 1 schon einmal fast passiert: Die Warp-Taste braucht ein veränderliches Tempo neben dem festen TEMPO, und es heißt darum flug."),

  H.h3("Die Teile einer Zeit heißen deutsch"),

  H.code(zeilen("kap04/fehler_teil.dh")),
  H.code(["Laufzeitfehler in fehler_teil.dh:3: ZEIT_TEIL: 'hour' ist kein Feld",
          "-- moeglich sind jahr, monat, tag, stunde, minute, sekunde,",
          "wochentag"], { out: true }),

  H.p("Die Befehle des Moduls heißen englisch (TIME_PART), die Namen der Teile aber deutsch: \"stunde\", \"minute\", \"sekunde\". Die Meldung nennt den Befehl bei seinem deutschen Zweitnamen ZEIT_TEIL — es ist derselbe — und zählt alle erlaubten Teile auf."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Setze SPRUNG auf 1.0. Wie bewegt sich der Sekundenzeiger jetzt? Und mit 0.02?"),
  H.bulletRich("Drehen 2. ", "Ändere in zeiger RAD(grad - 90) in RAD(grad). Wohin zeigen die Zeiger um zwölf Uhr?"),
  H.bulletRich("Drehen 3. ", "Vertausche die Werte von BLATT und TINTE. Ist das Ergebnis noch lesbar? Warum genügt diese eine Änderung?"),
  H.bulletRich("Erweitern 1. ", "Eine Weltzeituhr: drei kleine Uhren nebeneinander für New York, Berlin und Tokio, darunter Stadt und Uhrzeit. Hinweis: uhrZeichnen mit mx, my und r; TIME_FROM_UNIX mit einigen Stunden mehr oder weniger."),
  H.bulletRich("Erweitern 2. ", "Eine Stoppuhr: Die Leertaste startet und hält an, R stellt auf null. Der Sekundenzeiger zeigt die gestoppten Sekunden, darunter steht die Zeit als Zahl."),
  H.bulletRich("Erfinden. ", "Eine Uhr, die mehr zeigt: die Mondphase wie bei teuren Armbanduhren, das Datum in einem Fenster, einen Wecker, eine Uhr, die rückwärts läuft. Anhang A zeigt die Mondphase."),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Modul zeit: ", "Zeitpunkte, Ortszeit und Unix-Zeit, rechnen mit Zeiten, ZEIT_FORMAT$ mit seinen Mustern."),
  H.bulletRich("Handbuch, Sprache → Variablen und Konstanten: ", "DIM, CONST und warum Namen keine Groß- und Kleinschreibung kennen."),
  H.bulletRich("Handbuch, Grafik-Befehle → Schrift: ", "TEXT_SIZE, TEXT_WIDTH, eigene Schriften."),
];
