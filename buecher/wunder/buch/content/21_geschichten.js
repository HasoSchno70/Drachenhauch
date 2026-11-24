const { zeilen, stueck } = require("../code_lesen");

module.exports = (H) => [
  H.chapter("Geschichten, die warten", 21),

  H.h2("Was wir bauen"),

  H.figure("kap21_2_szene.png", "Eine kleine Szene auf einer Lichtung: Der Drache hat geniest, und am Himmel sind die Sterne angegangen.", 470, 294),

  H.p("Eine kleine Geschichte, die von selbst abläuft. Ein Drache kommt auf eine Lichtung und fragt, ob jemand da ist. Nach einer Pause kommt ein Igel und beschwert sich, dass er den Wald weckt. Der Drache entschuldigt sich — er übe nur Feuer —, niest dreimal Funken, und am Ende gehen die Sterne an. Solche Szenen gibt es in jedem Spiel: eine Zwischensequenz, ein Tutorial, eine Figur, die etwas erzählt. Das Schwierige daran ist nicht das Zeichnen, sondern das Warten: Das Programm muss 60-mal je Sekunde ein Bild zeichnen und dabei wissen, wo in der Geschichte es gerade ist. Dieses Kapitel zeigt zwei Arten, das aufzuschreiben — und warum die zweite sich liest wie ein Drehbuch."),

  H.h2("Der Plan"),

  H.bulletRich("Eine Bühne: ", "Figuren, die gehen und sprechen, und eine Lichtung — in einer eigenen Datei."),
  H.bulletRich("Der Ablauf als Zustandsautomat: ", "jeder Abschnitt ein nummerierter Zustand."),
  H.bulletRich("Der Ablauf als Koroutine: ", "eine Funktion, die mitten drin anhalten und später weitermachen kann."),
  H.bulletRich("Ein Treiber: ", "die Hauptschleife, die das Drehbuch weiterlaufen lässt, wenn das Warten vorbei ist."),

  H.h2("Schritt 1: die Bühne"),

  H.code(zeilen("kap21/figur.dh")),

  H.pmix(["Eine Figur hat einen Ort, ein Ziel und ein Tempo. ", ["gehe_zu", true], " setzt nur das Ziel; gegangen wird in ", ["schritt", true], ", Bild für Bild, und ", ["geht", true], " sagt, ob sie noch unterwegs ist. Beim Gehen hüpft sie, damit man es sieht. Steht etwas in text, zeichnet sie eine Sprechblase darüber, so breit wie der Text: ", ["TEXT_WIDTH", true], " misst ihn. Ganz unten steht ", ["sagen", true], ": Es setzt den Text und gibt zurück, wie lange man zum Lesen braucht — eine Sekunde und sechs Hundertstel je Buchstabe. Diese Datei binden beide Fassungen des Ablaufs mit ", ["IMPORT \"figur.dh\"", true], " ein; so unterscheiden sie sich nur in dem, worum es geht."]),

  H.h2("Schritt 2: der Ablauf als Zustandsautomat"),

  H.code(zeilen("kap21/1_automat.dh")),

  H.p("So schreibt man es, wenn man nur Schleifen und Variablen hat. Die Hauptschleife läuft jedes Bild einmal durch und muss jedes Mal neu herausfinden, wo sie ist. Dafür steht in zustand eine Nummer: 0 ist die Stille am Anfang, 1 der gehende Drache, 2 der sprechende und so weiter. SELECT CASE springt zum Zustand, und jeder prüft, ob er vorbei ist; dann bereitet er den nächsten vor und schaltet um. Das funktioniert, und für einen Abschnitt wie „der Drache geht, bis er steht“ ist es sogar übersichtlich. Aber schon nach sechs Zuständen ist die Geschichte schwer zu lesen: Was nacheinander geschieht, steht in Kästchen nebeneinander, und wer einen Satz einfügen will, muss Nummern verschieben. Und die drei Nieser, eine Schleife im Kopf, wären hier drei Zustände oder ein Zähler, den man selbst führt."),

  H.h2("Schritt 3: der Ablauf als Koroutine"),

  H.code(stueck("kap21/2_szene.dh", "FUNCTION szene", "END FUNCTION")),

  H.pmix(["Das ist die ganze Geschichte, von oben nach unten, wie ein Drehbuch. Das Besondere ist ", ["YIELD", true], ". Eine Funktion, die YIELD enthält, ist eine ", ["Koroutine", false], ": Ruft man sie auf, läuft sie noch nicht los, sondern liefert einen Griff vom Typ COROUTINE. Mit ", ["CORO_RESUME", true], " läuft sie bis zum nächsten YIELD, gibt dessen Wert heraus und hält an — mitten in der Funktion, mit allen lokalen Variablen. Beim nächsten CORO_RESUME geht es in der Zeile danach weiter, als wäre nichts gewesen. Selbst die FOR-Schleife mit den drei Niesern läuft so: Zwischen zwei Durchläufen hält die Koroutine an, und versuch behält seinen Wert."]),

  H.p("Was YIELD herausgibt, ist hier immer eine Zahl: wie lange gewartet werden soll. 1 heißt eine Sekunde, sagen(…) liefert die Lesezeit, und BIS_ALLE_STEHEN, −1, heißt: so lange, bis niemand mehr geht. Am Ende steht RETURN 0 — warum, steht unter „Was schiefgehen kann“."),

  H.h2("Schritt 4: der Treiber"),

  H.code(stueck("kap21/2_szene.dh", "DIM ablauf AS COROUTINE", "DIM warte AS FLOAT")),
  H.code(stueck("kap21/2_szene.dh", "' Ist das Warten vorbei", "END IF", 2)),

  H.pmix(["Die Hauptschleife ist jetzt einfach. ", ["szene()", true], " liefert den Griff, ", ["ablauf", true], ". In jedem Bild wird warte kleiner; ist es abgelaufen — oder steht bei BIS_ALLE_STEHEN niemand mehr —, geht das Drehbuch mit CORO_RESUME bis zum nächsten YIELD weiter, und dessen Wert ist das neue warte. ", ["CORO_DONE", true], " sagt, ob die Geschichte zu Ende ist. Die Leertaste setzt das Warten auf null: So überspringt man, was man schon gelesen hat. Der Rest der Schleife bewegt und zeichnet, wie jedes Bild. Das ganze Programm:"]),

  H.code(zeilen("kap21/2_szene.dh")),

  H.h2("Warum so?"),

  H.h3("Warum die Koroutine lesbarer ist"),

  H.p("Beide Fassungen tun dasselbe. Der Unterschied ist, wo der Ablauf steht. Im Zustandsautomaten steht er in den Übergängen: Um zu wissen, was nach Zustand 3 kommt, sucht man die Zeile naechster(4) und dann CASE 4. In der Koroutine steht er in der Reihenfolge der Zeilen — die nächste Zeile kommt als Nächstes, wie in jedem Programm ohne Warten. Das Programm merkt sich selbst, wo es steht, nämlich an der Stelle des YIELD; im Automaten muss man es sich mit zustand, uhr und dauer selbst merken. Darum schreibt man Abläufe, die über viele Bilder gehen — eine Szene, eine Anleitung, den Angriff eines Gegners —, gern als Koroutine."),

  H.h3("Warum der Zustandsautomat trotzdem seinen Platz hat"),

  H.p("Ein Automat ist besser, wenn es keine feste Reihenfolge gibt: Eine Spielfigur kann stehen, laufen, springen, fallen, und von fast jedem Zustand geht es in fast jeden anderen, je nachdem, was der Spieler drückt. Das ist kein Drehbuch, sondern ein Netz, und dafür sind nummerierte Zustände mit Übergängen genau richtig; das Modul animfsm hilft dabei. Faustregel: Gibt es ein „danach“, nimm eine Koroutine; gibt es ein „je nachdem“, einen Automaten."),

  H.h3("Warum die Koroutine nicht selbst wartet"),

  H.p("Die Koroutine könnte auch selbst die Sekunden zählen: in einer Schleife YIELD 0 und warte = warte − DELTA(). Dann müsste aber jedes Warten diese Schleife enthalten. So sagt das Drehbuch nur, wie lange, und der Treiber kümmert sich darum — an einer Stelle. Dort lässt sich auch das Überspringen einbauen, ohne das Drehbuch anzufassen."),

  H.h2("Was schiefgehen kann"),

  H.h3("Ein Helfer mit YIELD"),

  H.code(zeilen("kap21/fehler_helfer.dh")),
  H.code(["Der Drache kommt.",
          "-> warten 1.0",
          "Der Drache wartet.",
          "Der Drache spricht.",
          "-> warten 0.0"], { out: true }),
  H.code(["fehler_helfer.dh:11: Warnung: 'warten' ist eine Koroutine (sie enthaelt",
          "YIELD) -- als Anweisung aufgerufen, entsteht sie nur und laeuft nie. Das",
          "Ergebnis aufheben und mit CORO_RESUME fortsetzen: DIM c AS COROUTINE : c",
          "= warten(...)"], { out: true }),

  H.p("Es liegt nahe, das Warten in einen Helfer zu packen: warten(2). Aber ein Helfer mit YIELD ist selbst eine Koroutine, und als Anweisung aufgerufen liefert er nur seinen Griff, den niemand aufhebt. Zwischen „wartet“ und „spricht“ vergeht darum keine Zeit. YIELD hält immer nur die Funktion an, in der es steht, nie die, die sie aufgerufen hat. Beim Schreiben dieses Kapitels gab es dazu keine Meldung; jetzt warnt Drachenhauch. Wer die Schritte eines Helfers wirklich weitergeben will, hebt ihn auf und setzt ihn selbst fort — das Handbuch zeigt, wie. Einfacher ist meist, was die Szene tut: YIELD mit der Wartezeit."),

  H.h3("Eine Szene ohne YIELD"),

  H.p("Steht in szene kein einziges YIELD mehr — etwa weil man beim Umbauen alle gelöscht hat —, ist sie keine Koroutine, sondern eine gewöhnliche Funktion: szene() läuft sofort ganz durch und liefert ihre Zahl. Bisher landete die still in der COROUTINE-Variablen, und der Fehler kam erst beim nächsten CORO_RESUME. Jetzt warnt schon das Übersetzen, und die Zuweisung bricht ab: „Erwartet COROUTINE, erhalten FLOAT — eine Koroutine liefert nur der Aufruf einer FUNCTION oder SUB, die YIELD enthält“."),

  H.h3("Das Drehbuch ohne RETURN"),

  H.code(zeilen("kap21/fehler_ende.dh")),
  H.code(["warten: 1.5",
          "warten: 2.0"], { out: true }),
  H.code(["Laufzeitfehler in fehler_ende.dh:10: Zuweisung an 'warte': Erwartet",
          "FLOAT, erhalten NIL"], { out: true }),

  H.p("Das letzte CORO_RESUME, das die Koroutine beendet, liefert ihren RETURN-Wert. Eine SUB hat keinen, also kommt NIL zurück — und NIL passt nicht in eine FLOAT-Variable. Darum ist szene eine FUNCTION … AS FLOAT mit RETURN 0 am Ende: Der letzte Wert ist eine Wartezeit wie jede andere."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Ersetze das erste YIELD BIS_ALLE_STEHEN durch YIELD 0.5. Was passiert?"),
  H.bulletRich("Drehen 2. ", "Ändere in sagen die 0.06 auf 0.02. Für welche Sätze reicht die Zeit noch?"),
  H.bulletRich("Drehen 3. ", "Gib dem Igel das Tempo 300 statt 70. Muss das Drehbuch geändert werden?"),
  H.bulletRich("Erweitern 1. ", "Eine Zwischensequenz: Nach dem Gute-Nacht kommt eine Eule, und der Drache geht ab."),
  H.bulletRich("Erweitern 2. ", "Ein Tutorial, das auf den Spieler wartet: Vor jedem Niesen sagt der Drache, welche Taste es auslöst, und die Geschichte wartet, bis sie kommt."),
  H.bulletRich("Erfinden. ", "Der Spieler entscheidet mit: Der Igel fragt, ob der Drache weiterüben darf, und je nach Antwort geht die Geschichte anders weiter. Oder: eine Geschichte mit Kamerafahrt; Figuren, die gleichzeitig zwei Dinge tun (eine Koroutine je Figur); eine Erzählstimme mit SPEAK. Anhang A zeigt die Frage."),

  H.figure("kap21_antwort.png", "Erfinden: Der Igel fragt, und die Geschichte wartet auf J oder N.", 420, 262),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Sprache → Coroutines: YIELD: ", "CORO_RESUME, CORO_SEND, CORO_DONE, CORO_RESULT, CORO_CLOSE, und warum ein Helfer mit YIELD selbst eine Koroutine ist."),
  H.bulletRich("Handbuch, Module → animfsm: ", "Zustandsautomaten für Figuren, die zwischen vielen Zuständen wechseln."),
  H.bulletRich("examples/98_coroutines.dh: ", "Koroutinen als Zähler, Generator und Akkumulator."),
];
