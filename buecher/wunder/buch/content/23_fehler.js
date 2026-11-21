const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Fehler, die man fängt", 23),

  H.h2("Was wir bauen"),

  H.figure("kap23_3_diaschau.png", "Die Diaschau zeigt sechs Landschaften; unten stehen drei Dateien, die sich nicht laden ließen, und warum.", 470, 294),

  H.p("Eine Diaschau: Sie nimmt jedes Bild aus einem Ordner, zeigt es vier Sekunden lang und blendet weich zum nächsten über; mit den Pfeiltasten blättert man selbst. Das Besondere ist, was sie mit Dateien macht, die keine Bilder sind. In jedem echten Ordner liegt so etwas: eine Datei, die beim Herunterladen abgebrochen ist, eine leere, eine, die nur so heißt. Ein Programm, das beim ersten Fehler stehen bleibt, ist dann nicht zu gebrauchen. Dieses hier schreibt unten auf, was nicht ging, und zeigt den Rest. Dabei geht es um die Frage, welche Fehler man fängt und welche man besser gar nicht erst entstehen lässt."),

  H.h2("Der Plan"),

  H.bulletRich("Ein Ordner: ", "sechs Bilder, eine Datei mit ihren Titeln — und drei Dateien, die nur so tun."),
  H.bulletRich("Erst ohne Netz: ", "was passiert, wenn man einfach lädt."),
  H.bulletRich("Mit Netz: ", "TRY und CATCH um jede Datei."),
  H.bulletRich("Eigene Fehler: ", "die Titeldatei prüfen, mit THROW melden und mit FINALLY aufräumen."),
  H.bulletRich("Vermeiden statt fangen: ", "was das Programm selbst falsch machen könnte."),

  H.h2("Schritt 1: ein Ordner voller Bilder"),

  H.code(zeilen("kap23/1_bilder.dh")),
  H.code(["10 Dateien in bilder/"], { out: true }),

  H.pmix(["Die Bilder malt das Programm selbst, nicht auf den Bildschirm, sondern in Bilder im Speicher: ", ["IMAGE_NEW", true], " legt eins an, ", ["IMAGE_GRADIENT", true], " füllt es mit einem Himmel, ", ["IMAGE_FILL_ELLIPSE", true], " setzt eine Sonne oder einen Mond hinein und ", ["IMAGE_FILL_POLY", true], " zwei Bergketten davor, jede ein Vieleck mit zufälligen Gipfeln. ", ["IMAGE_SAVE", true], " schreibt das Bild als PNG-Datei. So entstehen sechs Landschaften vom Morgen bis zum Schnee, dazu titel.txt mit einem Titel je Bild."]),

  H.pmix(["Dann kommen die drei, um die es eigentlich geht. notiz.png ist eine Textdatei mit falscher Endung. leer.png ist leer. Und halb.png ist die erste Hälfte von 2_mittag.png: ", ["FILE_READ_BYTES", true], " liest die Datei als Bytes, ", ["BUFFER_SLICE", true], " schneidet die vordere Hälfte ab, ", ["FILE_WRITE_BYTES", true], " schreibt sie zurück — so sieht eine Datei aus, deren Herunterladen mittendrin abbrach."]),

  H.h2("Schritt 2: erst ohne Netz"),

  H.code(zeilen("kap23/2_ohne_netz.dh")),
  H.code(["1_morgen.png: 640 x 400",
          "2_mittag.png: 640 x 400",
          "3_abend.png: 640 x 400",
          "4_nacht.png: 640 x 400",
          "5_nebel.png: 640 x 400",
          "6_schnee.png: 640 x 400"], { out: true }),
  H.code(["Laufzeitfehler in 2_ohne_netz.dh:8: LOADIMAGE: 'bilder/halb.png'",
          "beginnt wie ein PNG-Bild, ist aber beschaedigt oder unvollstaendig",
          "(6662 Byte)"], { out: true }),

  H.p("Die sechs echten Bilder lädt das Programm, bei halb.png bricht es ab — mit einer Meldung, die genau sagt, was los ist, aber eben ab. „fertig“ kommt nie, und leer.png und notiz.png werden gar nicht mehr versucht. Für einen Fehler im eigenen Programm ist das richtig: Lieber sofort anhalten und die Zeile nennen, als mit falschen Werten weiterrechnen. Aber diese Datei ist kein Fehler im Programm. Sie kommt von draußen, und das Programm kann nichts dafür, dass sie kaputt ist. Es soll sie überspringen."),

  H.h2("Schritt 3: mit Netz"),

  H.code(stueck("kap23/3_diaschau.dh", "' Jede Datei mit der Endung", "NEXT")),

  H.pmix([["TRY", true], " leitet einen Block ein, in dem ein Fehler das Programm nicht beendet. Geht in ihm etwas schief, springt es sofort in den Zweig ", ["CATCH fehler", true], ", und in fehler steht die Meldung als Text. Danach geht es hinter ", ["END TRY", true], " weiter, als wäre nichts gewesen — hier mit der nächsten Datei. Weil das ", ["ARRAY_PUSH", true], " für das Bild nach dem LOADIMAGE steht, kommt in bilder und namen nur, was sich laden ließ; die Meldung landet in probleme. Die Endung prüft das Programm vorher mit einer kleinen Funktion, ", ["endung$", true], ": titel.txt ist kein Bild und soll gar nicht erst versucht werden."]),

  H.h2("Schritt 4: eigene Fehler"),

  H.code(stueck("kap23/3_diaschau.dh", "FUNCTION titel_lesen", "END FUNCTION")),

  H.pmix(["Die Titeldatei hat ein eigenes Format: je Zeile Dateiname, Semikolon, Titel. Fehlt das Semikolon, ist die Datei kaputt — nicht für LOADIMAGE, sondern für uns. Das meldet ", ["THROW", true], ": ", ["THROW \"TITEL\", meldung", true], " löst einen Fehler aus wie jeder eingebaute, mit zwei Teilen. Der erste ist ein Code, ein kurzes Wort, auf das ein Programm reagieren kann; der zweite die Meldung für Menschen, hier mit Dateiname und Zeilennummer."]),

  H.pmix(["Die Datei ist mit ", ["OPENFILE", true], " geöffnet, und geöffnete Dateien muss man wieder schließen. Darum steht das Lesen in einem TRY mit ", ["FINALLY", true], ": Was dort steht, läuft immer — nach dem letzten Durchlauf, aber auch, wenn mitten im Lesen ein THROW kommt. Ein CATCH hat dieses TRY nicht. Es fängt nichts, es räumt nur auf, und der Fehler geht danach weiter zu dem, der titel_lesen gerufen hat:"]),

  H.code(stueck("kap23/3_diaschau.dh", "' Die Titel sind freiwillig", "END IF")),

  H.pmix(["Hier wird entschieden. Fehlt titel.txt ganz, ist das kein Fehler, denn die Titel sind freiwillig; ", ["FILEEXISTS", true], " fragt das vorher, statt es auf einen Fehler ankommen zu lassen. Gibt es die Datei, fängt das CATCH, was titel_lesen meldet, und ", ["ERROR_CODE$()", true], " sagt, welcher Fehler es war. Bei TITEL kommt die Meldung zu den Problemen, und die Bilder zeigen ihren Dateinamen. Jeder andere Fehler — einer, den wir nicht erwartet haben — wird mit ", ["THROW fehler", true], " weitergeworfen. Was man nicht versteht, soll man nicht verschlucken."]),

  H.h2("Schritt 5: zeigen"),

  H.code(stueck("kap23/3_diaschau.dh", "WHILE NOT QUITREQUESTED()", "WEND")),

  H.pmix(["Die Hauptschleife zeigt das Bild nr. Nach vier Sekunden, oder mit der rechten Pfeiltaste, kommt das nächste; ", ["MOD", true], " fängt nach dem letzten wieder beim ersten an. In der letzten Weile davor liegt das nächste schon darüber und wird von Bild zu Bild weniger durchsichtig — das ist die Überblendung. ", ["MAP_GET_OR", true], " holt den Titel und nimmt den Dateinamen, wenn es keinen gibt. Ganz unten stehen die Probleme, höchstens vier Zeilen, damit nichts aus dem Fenster ragt. Und ganz oben eine Frage, die kein CATCH ersetzt: Gibt es überhaupt ein Bild? Ohne die Frage hieße es bei einem leeren Ordner „Index außerhalb“, sobald bilder[nr] gezeichnet wird. Das ganze Programm:"]),

  H.code(zeilen("kap23/3_diaschau.dh")),

  H.h2("Warum so?"),

  H.h3("Welche Fehler man fängt"),

  H.p("Es gibt zwei Sorten Fehler. Die einen kommen von draußen: eine kaputte Datei, ein Netz, das nicht antwortet, eine Eingabe, die keine Zahl ist. Die kann das Programm nicht verhindern, nur mit ihnen rechnen — dafür ist TRY da. Die anderen macht das Programm selbst: ein Index, den es nicht gibt, ein vertippter Name, eine Division durch null. Die fängt man nicht, man vermeidet sie. Das leere bilder, die fehlende titel.txt: Beides fragt die Diaschau vorher, mit LEN und FILEEXISTS. Ein Fehler im eigenen Programm, der gefangen wird, ist nur versteckt."),

  H.h3("Warum das TRY so eng ist"),

  H.p("Um die Schleife steht kein großes TRY, sondern um jede Datei ein kleines, und darin nur zwei Zeilen. Je mehr in einem TRY steht, desto mehr Fehler fängt es, auch solche, die gar nicht von draußen kommen — und meldet sie dann als „Datei kaputt“. Wie das aussieht, steht unter „Was schiefgehen kann“. Ein enges TRY sagt genau, womit man rechnet."),

  H.h3("Warum ein Code"),

  H.p("Das CATCH für die Titel entscheidet nach ERROR_CODE$, nicht nach dem Text der Meldung. Texte ändern sich: Beim Schreiben dieses Kapitels hat sich sogar die Meldung von LOADIMAGE geändert (siehe unten). Wer auf ihren Wortlaut geprüft hätte, wäre jetzt kaputt. Ein Code wie TITEL ist eine Abmachung zwischen zwei Stellen im eigenen Programm, und die Meldung darf sich ändern, ohne dass etwas bricht. Eingebaute Fehler haben den Code \"\" — darum fällt jeder von ihnen beim Titel-CATCH in den Zweig „weiterwerfen“."),

  H.h2("Was schiefgehen kann"),

  H.h3("Ein TRY, das zu viel fängt"),

  H.code(zeilen("kap23/fehler_zuviel.dh")),
  H.code(["1_morgen.png: 640 Punkte breit",
          "2_mittag.png: 640 Punkte breit",
          "3_abend.png: kaputt, uebersprungen"], { out: true }),

  H.p("3_abend.png ist in Ordnung. Kaputt ist das Programm: Es hat nur zwei Breiten für drei Dateien, und breiten[2] gibt es nicht. Das TRY fängt aber alles, was in ihm passiert, und das CATCH sieht sich den Fehler nicht einmal an. So wird aus einem Fehler im Programm eine Meldung über eine Datei, und man sucht an der falschen Stelle. Zwei Dinge helfen: das TRY nur um das LOADIMAGE legen und im CATCH die Meldung ausgeben, statt eine eigene zu erfinden."),

  H.h3("Ein TRY ohne CATCH"),

  H.code(zeilen("kap23/fehler_still.dh")),
  H.code(["50",
          "20",
          "fertig"], { out: true }),
  H.code(["fehler_still.dh:6: Warnung: TRY ohne CATCH und ohne FINALLY verschluckt",
          "jeden Fehler still -- danach weiss niemand, dass einer da war. Mit CATCH e",
          "den Fehler ansehen (oder ausdruecklich schlucken: ein leeres CATCH);",
          "FINALLY raeumt nur auf und reicht ihn weiter"], { out: true }),

  H.p("Die Division durch null verschwindet spurlos: keine Zeile, keine Meldung, nur ein Wert weniger. Ein TRY ganz ohne CATCH und FINALLY verschluckt jeden Fehler, und das ist fast nie gemeint. Beim Schreiben dieses Kapitels sagte Drachenhauch dazu nichts; jetzt warnt es. Wer einen Fehler wirklich übergehen will, schreibt ein leeres CATCH — dann sieht jeder Leser, dass es Absicht ist."),

  H.h3("Ein Tippfehler, den kein Probelauf findet"),

  H.code(zeilen("kap23/fehler_ordner.dh")),
  H.code(["10 Dateien in bilder"], { out: true }),
  H.code(["fehler_ordner.dh:7: Warnung: 'oredner' wird hier gelesen, aber",
          "nirgends im Programm mit DIM oder CONST angelegt. Beim Laufen bricht",
          "diese Zeile ab (\"Variable 'oredner' nicht deklariert\"). Meintest du",
          "'ordner'?"], { out: true }),

  H.pmix(["Der Zweig für den leeren Ordner läuft fast nie — und darum fällt der Tippfehler darin bei keinem Probelauf auf. Erst wenn jemand die Diaschau auf einen leeren Ordner richtet, bricht sie ab, statt es zu sagen. Solche Fehler findet man nicht durch Ausprobieren, sondern durch Lesen, und das Lesen nimmt einem Drachenhauch ab: Schon beim Übersetzen, bevor die erste Zeile läuft, prüft es jeden Namen. Die Warnung steht oben in der Ausgabe, in der IDE ist die Zeile unterstrichen, und ", ["dhrt --check datei.dh", true], " prüft, ohne das Programm zu starten. Das ist die zweite Hälfte des Kapitels: Fehler von draußen fängt man, eigene findet man vorher."]),

  H.h3("Eine Meldung ohne Namen"),

  H.p("Bevor dieses Kapitel fertig war, meldete LOADIMAGE alle drei kaputten Dateien mit demselben englischen Satz von raylib, der Bibliothek darunter: „image data is null, either the file doesnt exist or the image type is unsupported“. Ohne Dateinamen — unten in der Diaschau standen drei gleiche Zeilen, und man erfuhr nicht, welche Dateien gemeint waren. Und „doesnt exist“ stimmte nicht, die Dateien waren ja da. Jetzt sagt LOADIMAGE, welche Datei es ist und was mit ihr los ist: leer, beschädigt oder gar kein Bild."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Nimm in 3_diaschau.dh das TRY um das Laden weg, lass nur die zwei ARRAY_PUSH stehen. Was passiert?"),
  H.bulletRich("Drehen 2. ", "Lösche in bilder/titel.txt bei einer Zeile das Semikolon. Was zeigt die Diaschau?"),
  H.bulletRich("Drehen 3. ", "Benenne bilder/titel.txt um. Was ändert sich, und warum gibt es keine Meldung?"),
  H.bulletRich("Erweitern 1. ", "Eine Diaschau mit Übergängen: reihum blenden, schieben und aus der Mitte wachsen."),
  H.bulletRich("Erweitern 2. ", "Ein Fortschrittsbalken: Bei vielen großen Bildern dauert das Laden. Lade eine Datei je Bild und zeig dabei, wie weit es ist."),
  H.bulletRich("Erfinden. ", "Eine Diaschau, die ihren Ordner beobachtet: Neue Bilder kommen dazu, geänderte werden neu geladen, und eine kaputte Datei wird wieder versucht, sobald sie sich ändert. Dazu ein eigener Fehler mit eigenem Code für Bilder, die zu klein sind. Oder: ein Protokoll der Probleme in einer Datei; eine Diaschau, die Titel aus einer CSV liest; ein Taschenrechner, der jede Eingabe übersteht. Anhang A zeigt die Diaschau, die beobachtet."),

  H.figure("kap23_uebergaenge.png", "Erweitern 1: Mitten im Schieben — der Mittag rutscht nach links hinaus, der Abend kommt von rechts.", 420, 262),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Sprache → Try / Catch / Throw: ", "FINALLY, Fehler-Codes mit ERROR_CODE$ und ERROR_LINE, und was mit einem RETURN im TRY geschieht."),
  H.bulletRich("Handbuch, Werkzeuge → dhrt --check: ", "übersetzen ohne auszuführen, und was der Übersetzer alles prüft."),
  H.bulletRich("Handbuch, Grafik-Befehle → Bilder: ", "LOADIMAGE, die IMAGE_-Befehle zum Malen in Bilder, und wie der Zwischenspeicher für Bilder arbeitet."),
];
