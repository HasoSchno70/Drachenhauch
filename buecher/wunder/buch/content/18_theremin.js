const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Der Theremin", 18),

  H.h2("Was wir bauen"),

  H.figure("kap18_2_theremin.png", "Ein Theremin mit der Maus: links tief, rechts hoch, oben leise, unten laut. Die Spur zeigt die letzten zwei Sekunden Melodie, die Welle den Ton, stark verlangsamt.", 470, 294),

  H.p("Ein Theremin ist das Instrument, das man spielt, ohne es zu berühren: Die eine Hand bestimmt die Tonhöhe, die andere die Lautstärke, und der Ton gleitet ohne Stufen von einer Höhe zur nächsten. Bei uns übernimmt die Maus beides. Solange die linke Taste unten ist, klingt ein Ton; nach rechts wird er höher, nach oben leiser. Ein Netz aus Halbtönen hilft beim Zielen, oben steht der Name des Tons. In diesem Kapitel geht es darum, wie man Töne als Zahlen beschreibt, und warum Tonhöhe und Lautstärke beide nicht so gezählt werden, wie man zuerst denkt."),

  H.h2("Der Plan"),

  H.bulletRich("Eine Tonleiter aus Zahlen: ", "Ein Halbton höher heißt: die Frequenz mal 2^(1/12)."),
  H.bulletRich("Ein Ton, der nicht aufhört: ", "eine Sekunde Sinus, endlos wiederholt, Tonhöhe und Lautstärke in jedem Bild nachgeführt."),
  H.bulletRich("Links tief, rechts hoch: ", "Jede Oktave bekommt gleich viel Breite — die Frequenz wächst mit 2 hoch x."),
  H.bulletRich("Oben leise: ", "Die Lautstärke wird in Dezibel gezählt und erst am Ende in einen Faktor umgerechnet."),
  H.bulletRich("Ein Zittern: ", "Ein Modulator lässt die Lautstärke schwingen, ohne dass das Programm etwas tun muss."),

  H.h2("Schritt 1: eine Tonleiter aus Zahlen"),

  H.p("Ein Ton ist eine Schwingung, und seine Höhe ist die Zahl der Schwingungen je Sekunde: die Frequenz, gemessen in Hertz (Hz). Das A, nach dem Orchester stimmen, schwingt 440-mal je Sekunde. Dieses Programm spielt alle zwölf Halbtöne von einem A zum nächsten und schreibt ihre Frequenzen hin:"),

  H.code(zeilen("kap18/1_tonleiter.dh")),
  H.code(["A    220.0 Hz",
          "B    233.1 Hz",
          "H    246.9 Hz",
          "C    261.6 Hz",
          "Cis  277.2 Hz",
          "D    293.7 Hz",
          "Dis  311.1 Hz",
          "E    329.6 Hz",
          "F    349.2 Hz",
          "Fis  370.0 Hz",
          "G    392.0 Hz",
          "Gis  415.3 Hz",
          "A    440.0 Hz"], { out: true }),

  H.pmix(["Das obere A hat genau doppelt so viele Schwingungen wie das untere; das ist eine Oktave. Dazwischen liegen zwölf Halbtöne, und jeder ist um denselben Faktor höher als der vorige, nicht um dieselbe Zahl von Hertz: ", ["2 ^ (n / 12)", true], ". Zwölfmal malgenommen ergibt das genau 2. Darum werden die Abstände in Hertz nach oben immer größer — vom A zum B sind es 13 Hz, vom Gis zum A schon 25. ", ["AUDIO_NOTE", true], " baut eine Note mit Hüllkurve: Dreieckswelle, 250 ms lang, 5 ms Anschlag, kein Abfall, voll gehalten, 60 ms Ausklingen, halbe Lautstärke. ", ["AUDIO_PLAY", true], " spielt sie, und das Programm wartet mit SLEEP, bis die nächste dran ist."]),

  H.h2("Schritt 2: ein Ton, der nicht aufhört"),

  H.p("Ein Theremin spielt keine Noten, sondern einen einzigen Ton, der nie aufhört und nur Höhe und Lautstärke ändert. Den baut das Programm einmal und spielt ihn endlos:"),

  H.code(stueck("kap18/2_theremin.dh", "' Der Ton: eine Sekunde", "DIM kanal")),

  H.pmix([["AUDIO_TONE(TIEF, 1000, \"sine\")", true], " ist eine Sekunde Sinus mit 110 Hz. ", ["AUDIO_PLAY(ton, -1, 0)", true], " spielt ihn — −1 heißt endlos, 0 heißt stumm — und liefert einen Kanal, über den man den laufenden Ton verstellt. In jeder Runde der Schleife sagt das Programm dem Kanal, wie hoch und wie laut er gerade sein soll:"]),

  H.code(stueck("kap18/2_theremin.dh", "AUDIO_PITCH(kanal", "AUDIO_VOLUME(kanal")),

  H.p("AUDIO_PITCH spielt den Ton schneller ab: Faktor 2 ist eine Oktave höher, Faktor 4 zwei Oktaven. Der Ton selbst bleibt dieselbe Sekunde Sinus mit 110 Hz, nur liest die Soundkarte ihn schneller. Warum gerade eine Sekunde mit 110 Hz? Weil das genau 110 ganze Schwingungen sind. Die letzte endet dort, wo die erste beginnt, und die Wiederholung hat keine Naht. Beim Ausprobieren für dieses Buch hatte sie trotzdem eine: AUDIO_TONE blendet jeden Ton an Anfang und Ende 5 ms lang ein und aus, damit ein einzelner Piep nicht knackt — in einer Schleife wurde daraus ein kurzes Loch, viermal je Sekunde. Jetzt läuft ein Ton, der endlos gespielt wird, ohne diese Blenden."),

  H.h2("Schritt 3: links tief, rechts hoch"),

  H.p("Wie hoch soll der Ton an der Stelle x sein? Über die Breite verteilt das Programm vier Oktaven, von A mit 110 Hz bis A mit 1760 Hz, und jede bekommt gleich viel Platz:"),

  H.code(stueck("kap18/2_theremin.dh", "FUNCTION frequenz", "END FUNCTION")),

  H.pmix(["x / BREITE * OKTAVEN zählt, wie viele Oktaven die Maus über dem linken Rand steht — 0 ganz links, 4 ganz rechts, 1,5 bei 300. ", ["2 ^ 1.5", true], " macht daraus den Faktor: In jeder Oktave verdoppelt sich die Frequenz. Damit hat jeder Halbton auf dem Bildschirm dieselbe Breite, gut 16 Punkte, und das Netz aus senkrechten Linien ist gleichmäßig. Für die Anzeige geht es auch zurück: Welcher Ton ist das?"]),

  H.code(stueck("kap18/2_theremin.dh", "FUNCTION tonname$", "END FUNCTION")),

  H.pmix(["Rückwärts braucht man den Logarithmus: Die Frage „2 hoch wie viel ergibt f / 440?“ beantwortet ", ["LOG(f / 440) / LOG(2)", true], ". LOG ist der natürliche Logarithmus; geteilt durch LOG(2) wird er zum Logarithmus zur Basis 2. Mal 12 sind es Halbtöne, und plus 69 ist die Nummer des Tons, wie sie MIDI zählt, die Sprache der elektronischen Instrumente: Das A mit 440 Hz hat die 69, das tiefste C die 0. Die Nummer gerundet ist der nächste Ton; der Rest, mal 100, sind die Cent — ein Hundertstel Halbton. Bei ±50 Cent liegt man genau zwischen zwei Tönen."]),

  H.h2("Schritt 4: oben leise"),

  H.code(stueck("kap18/2_theremin.dh", "FUNCTION dezibel", "END FUNCTION", 2)),

  H.pmix(["Auch die Lautstärke wird nicht direkt verteilt. Ganz oben sind es −40 Dezibel (dB), ganz unten 0 dB, dazwischen gleichmäßig. Erst ", ["faktor", true], " macht daraus die Zahl, mit der die Welle malgenommen wird: ", ["10 ^ (db / 20)", true], ". 0 dB ist der Faktor 1, also volle Lautstärke; −20 dB ist 0,1; −40 dB ist 0,01. Warum dieser Umweg, steht unter „Warum so?“. Danach gleitet die Lautstärke zum Ziel, statt zu springen:"]),

  H.code(stueck("kap18/2_theremin.dh", "DIM ziel AS FLOAT", "v = v + ")),

  H.p("Je Bild legt v ein Viertel des Wegs zum Ziel zurück. Drückt man die Taste, schwillt der Ton in etwa einer Zehntelsekunde an, lässt man sie los, klingt er ebenso schnell aus — wie bei einem echten Theremin, wenn man die Hand vom Instrument zieht."),

  H.h2("Schritt 5: ein Zittern"),

  H.code(stueck("kap18/2_theremin.dh", "IF KEY_HIT(KEY_T)", "END IF", 2)),

  H.pmix(["Ein Tremolo ist ein schnelles Zittern der Lautstärke. Das Programm könnte es selbst ausrechnen, in jedem Bild mit SIN. Statt dessen bekommt das Audio-Modul einen ", ["Modulator", false], ": ", ["AUDIO_LFO_NEW(\"sine\", 6)", true], " ist ein Wert, der sechsmal je Sekunde zwischen −1 und +1 schwingt, von selbst. ", ["AUDIO_MODULATE", true], " bindet ihn an die Lautstärke des SFX-Busses, auf dem alle Klänge laufen, und bildet −1 auf 40 % und +1 auf 100 % ab. ", ["AUDIO_MOD_REMOVE", true], " löst ihn wieder, und der Bus kehrt zu seiner festen Lautstärke zurück. Auch das hat das Buch verändert: Bisher blieb der Bus auf dem letzten Wert des Zitterns stehen, bei jedem Mal einem anderen."]),

  H.h2("Schritt 6: das Bild"),

  H.p("Bleibt das, was man sieht: das Netz aus Halbtönen und Dezibel, eine Welle in der Mitte, deren Schwingungen enger werden, je höher der Ton, und die höher ausschlägt, je lauter er ist, und eine Spur der letzten zwei Sekunden. Ihre Dicke folgt den Dezibel, nicht dem Faktor:"),

  H.code(stueck("kap18/2_theremin.dh", "FUNCTION staerke", "END FUNCTION")),

  H.p("Die Welle schlägt mit dem Faktor aus, wie eine echte Schwingung: Bei −20 dB ist sie nur noch ein Zehntel so hoch, bei −40 dB ein flacher Strich. Die Spur dagegen wird so dick, wie es sich anhört. Die Farbe dreht sich mit jeder Oktave ein Stück weiter, und mit_deckkraft zerlegt sie wie in Kapitel 16 in Rot, Grün und Blau, um sie durchscheinend zu machen. Das ganze Programm:"),

  H.code(zeilen("kap18/2_theremin.dh")),

  H.h2("Warum so?"),

  H.h3("Warum Tonhöhe logarithmisch ist"),

  H.p("Das Ohr hört Verhältnisse, nicht Abstände. Von 110 nach 220 Hz klingt es genauso nach „eine Oktave höher“ wie von 880 nach 1760 Hz, obwohl der Abstand einmal 110 und einmal 880 Hz beträgt. Für das Ohr ist ein Schritt das Malnehmen mit einem Faktor, und Malnehmen wird erst zum Zusammenzählen, wenn man den Logarithmus nimmt. Darum sind die Tasten eines Klaviers alle gleich breit, obwohl die Frequenzen nach rechts immer schneller wachsen, und darum steht im Programm 2 hoch x. Die zwölf gleichen Schritte je Oktave heißen gleichstufige Stimmung; sie sind ein Kompromiss, bei dem keine Tonart besser klingt als eine andere — die Übung mit den zwei Stimmen hört den Unterschied."),

  H.h3("Warum Lautstärke auch"),

  H.p("Mit der Lautstärke ist es genauso. Ein Ton mit dem Faktor 0,5 klingt nicht halb so laut, sondern nur ein Stück leiser; halb so laut hört er sich erst bei etwa einem Drittel an. Das Ohr vergleicht Verhältnisse — von 1 nach 0,1 ist derselbe Schritt wie von 0,1 nach 0,01. Dezibel zählen diese Schritte: −20 dB ist ein Zehntel, −40 dB ein Hundertstel. Würde das Programm den Faktor gleichmäßig über die Höhe verteilen, wäre fast die ganze obere Hälfte des Fensters laut und alles Leise drängte sich in einen schmalen Streifen am oberen Rand. Mit Dezibel klingt jeder Zentimeter nach oben gleich viel leiser."),

  H.h3("Warum ein Modulator"),

  H.p("Das Tremolo könnte das Programm auch selbst rechnen. Aber das Programm kommt nur 60-mal je Sekunde vorbei, und wenn ein Bild länger dauert, weil Windows gerade etwas anderes tut, stockt das Zittern hörbar. Ein Modulator läuft auf dem Audio-Thread, der jeden einzelnen Abtastwert berechnet, 48 000-mal je Sekunde. Er stockt nie, und das Programm muss nichts dafür tun. Allerdings fährt ein Modulator nur, was an einem Bus hängt: Lautstärke, Balance, Filter, Hall, Verzerrung. Die Tonhöhe eines Kanals gehört nicht dazu — ein Vibrato, ein Zittern der Tonhöhe, muss das Programm darum selbst rechnen. Die zweite Übung tut das und misst, ob man es hört."),

  H.h2("Was schiefgehen kann"),

  H.h3("Die Frequenz gleichmäßig verteilt"),

  H.code(zeilen("kap18/fehler_linear.dh")),
  H.code(["A2 (110.0 Hz) bei x = 0",
          "A3 (220.0 Hz) bei x = 53",
          "A4 (440.0 Hz) bei x = 160",
          "A5 (880.0 Hz) bei x = 373",
          "A6 (1760.0 Hz) bei x = 800"], { out: true }),

  H.p("Die Rechnung sieht vernünftig aus: tief links, hoch rechts, dazwischen gleichmäßig. Gespielt ist sie ein Ärgernis. Die unterste Oktave drängt sich in 53 Punkte, ein Halbton ist dort gut vier Punkte breit — die Maus zittert mehr als das. Die oberste Oktave füllt mehr als die halbe Breite, und ein Halbton ist 35 Punkte breit. Wer das hört, sagt: Links ist alles verstimmt, rechts passiert kaum etwas."),

  H.h3("Die Frequenz statt des Faktors"),

  H.pmix(["AUDIO_PITCH will einen Faktor, keine Frequenz. Wer ", ["AUDIO_PITCH(kanal, f)", true], " schreibt, spielt den Ton f-mal so schnell. Ganz links, bei f = 110, wird aus 110 Hz ein Pfeifen von 12 100 Hz. Schon bei f = 220 wären es 24 200 Hz. So hoch kann die Soundkarte nicht: Sie rechnet mit 48 000 Abtastwerten je Sekunde und kann damit höchstens die Hälfte, 24 000 Hz, darstellen. Was darüber liegt, faltet sie zurück — gemessen kam bei f = 440 statt 48 400 Hz ein ganz gewöhnlicher Ton von 400 Hz heraus. Das klingt dann nicht kaputt, sondern nur falsch, und nach rechts geht es mal hinauf, mal hinunter. Richtig ist ", ["f / TIEF", true], ": wie oft so hoch wie der Ton, der gebaut wurde."]),

  H.h3("Eine Schleife, die nicht aufgeht"),

  H.p("Ein Ton von 1003 statt 1000 ms enthält 110,33 Schwingungen. Am Ende steht die Welle dann nicht wieder bei null, sondern bei 87 % ihres Ausschlags, und die Wiederholung beginnt bei null — ein Sprung, der knackt, bei Tonhöhe 4 viermal je Sekunde. Im Mitschnitt sprang die Welle an jeder Naht, bei 1000 ms an keiner. Die Regel: Eine Schleife braucht ganze Schwingungen, also bei ganzzahliger Frequenz eine ganze Sekunde, bei 220 Hz auch eine halbe."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Setze OKTAVEN auf 2 und auf 7. Wie breit ist ein Halbton jeweils, und wie spielt es sich?"),
  H.bulletRich("Drehen 2. ", "Ändere LEISE_DB auf −80. Was hört man in der oberen Hälfte des Fensters?"),
  H.bulletRich("Drehen 3. ", "Ersetze \"sine\" durch \"square\" und durch \"saw\". Welche Welle zeigt das Bild, welche hört man?"),
  H.bulletRich("Erweitern 1. ", "Einrasten: Mit E rastet der Ton auf die Tonleiter ein (a-Moll, die weißen Tasten des Klaviers). Zwischen zwei Tönen soll er kurz gleiten, statt zu springen."),
  H.bulletRich("Erweitern 2. ", "Vibrato: Mit V schwingt die Tonhöhe fünfmal je Sekunde um einen Viertelton auf und ab."),
  H.bulletRich("Erfinden. ", "Eine zweite Stimme, die eine Quinte höher mitspielt. Oder: ein Echo; eine Stimme für die rechte Maustaste; ein Akkord, der mitwandert; ein Ton, der nach dem Loslassen langsam ausklingt. Anhang A zeigt die zweite Stimme und hört, warum zwei Quinten nicht dasselbe sind."),

  H.figure("kap18_einrasten.png", "Erweitern 1: Eingerastet wird aus der schrägen Spur eine Treppe — jede Stufe ein Ton der Leiter. Wo zwei weiße Tasten keine schwarze zwischen sich haben (H und C, E und F), ist die Stufe halb so breit.", 420, 262),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Module → audio: ", "Kanäle, Busse, Effekte, Mitschnitt (AUDIO_RECORD_START) und die Uhr für Musik im Takt."),
  H.bulletRich("Handbuch, Module → Audio-Modulatoren: ", "LFO und Tweener, und welche Bus-Parameter sie fahren."),
  H.bulletRich("examples/190_tracker.dh: ", "ein ganzer Tracker in Drachenhauch, Noten aus AUDIO_NOTE."),
];
