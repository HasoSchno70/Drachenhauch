const { stueck } = require("../code_lesen");

const E = "kap16/mediathek/EintragFenster.dh";

module.exports = (H) => [
  H.chapter("Aus dem Netz", 16),

  H.h2("Was wir bauen"),

  H.figure("kap16_nachschlagen.png", "ISBN eingefügt, „Nachschlagen“ geklickt: Titel, Urheber, Jahr und Cover kommen aus dem Netz. Das Fenster blieb dabei bedienbar.", 480, 320),

  H.p("Bisher hat jemand jeden Eintrag der Mediathek abgetippt. Bei Büchern geht es kürzer: Auf jedem steht eine ISBN, und Open Library, ein freier Katalog im Netz, kennt zu vielen davon Titel, Urheber, Erscheinungsjahr und ein Bild des Umschlags. Das Bearbeitungsfenster bekommt ein Feld ISBN und einen Knopf „Nachschlagen“; was die Antwort bringt, landet in den Feldern und im Bild daneben."),

  H.p("Das Netz ist langsam, verglichen mit allem, was die Mediathek bisher tat — eine Antwort kann eine Sekunde dauern oder nie kommen. Darum geht es in diesem Kapitel weniger um die Frage an Open Library als darum, wie ein Fenster auf eine Antwort wartet, ohne stehen zu bleiben."),

  H.h2("Der Plan"),

  H.p("Open Library beantwortet eine Frage, die als Adresse kommt: /api/books?bibkeys=ISBN:… mit format=json und jscmd=data liefert die Daten eines Buchs als JSON. Abgefragt wird im Hintergrund, mit vier Befehlen aus dem Modul html:"),

  H.bulletRich("HTTP_GET_START(url) ", "beginnt den Abruf und kehrt sofort zurück, mit einer Nummer für ihn."),
  H.bulletRich("HTTP_READY(abruf) ", "sagt, ob die Antwort da ist — oder ein Fehler."),
  H.bulletRich("HTTP_RESULT(abruf) ", "holt die Antwort ab; ein Fehler wird hier zum Laufzeitfehler, den TRY fängt."),
  H.bulletRich("HTTP_CANCEL(abruf) ", "verwirft einen Abruf, dessen Antwort niemand mehr braucht."),

  H.p("Wer fragt, ob die Antwort da ist, ist ein Timer: alle 100 Millisekunden einmal, nur solange ein Abruf läuft. Dazwischen arbeitet das Fenster wie immer."),

  H.h2("Schritt 1: Spalten und Fenster"),

  H.p("Die Tabelle eintrag bekommt zwei Spalten: isbn (Text) und cover (Bild). Wie in Kapitel 13 gilt das sofort für die Datei im Projektordner; die Einträge darin bleiben und haben die neuen Spalten leer."),

  H.p("Das Bearbeitungsfenster wird breiter. Über den vier Feldern stehen das Feld feldIsbn, gebunden an isbn, und der Knopf knopfNachschlagen. Rechts daneben liegt ein Bild-Control bildCover; im Inspektor steht bei ihm unter Bildspalte cover. Ein Bild-Control hängt nicht an der Bindung des Formulars, sondern an seiner Bildspalte: Laden(id) zeigt das Bild der Zeile, BildSetzen(control, pfad) setzt ein neues, und Sichern() schreibt es mit den Feldern in die Datei."),

  H.p("Dazu kommt der Timer, aus der Bibliothek unter „Unsichtbar“. Er liegt nicht im Fenster, sondern in einer Leiste unter dem Entwurf. Im Inspektor: Name abfrageTimer, Intervall 100, „Wiederholen“ an, „Läuft beim Start“ aus. Sein Ereignis heißt abfrageTimer_Aktion."),

  H.h2("Schritt 2: die Frage stellen"),

  H.p("Die Fensterklasse merkt sich den laufenden Abruf, was er holt und nach welcher ISBN. Geoeffnet setzt den Abruf auf -1 — keiner — und den Fokus ins Feld ISBN. Oben im Code stehen IMPORT \"html\", \"json\" und \"regex\" und die Adresse als Konstante QUELLE = \"https://openlibrary.org\"."),

  H.code(stueck(E, "SUB knopfNachschlagen_Klick", "END SUB")),

  H.p("Eine ISBN wird oft mit Bindestrichen geschrieben; REPLACE$ nimmt sie heraus, bevor gezählt wird. HTTP_TIMEOUT(10) gibt dem Server zehn Sekunden — das ist auch die Vorgabe, aber hier steht es, damit man es findet. Während der Abruf läuft, heißt der Knopf „Abbrechen“, und derselbe Klick bricht ab."),

  H.h2("Schritt 3: nachsehen, ob sie da ist"),

  H.code(stueck(E, "SUB abfrageTimer_Aktion", "END SUB")),

  H.p("Solange HTTP_READY nein sagt, tut der Timer nichts. Ein Fehler — kein Netz, keine Antwort in der Zeit — macht HTTP_READY ebenfalls wahr und kommt erst mit HTTP_RESULT heraus; darum steht dort das TRY."),

  H.h2("Schritt 4: die Antwort lesen"),

  H.p("Die Antwort für eine bekannte ISBN sieht so aus, gekürzt:"),

  H.code([
    "{",
    "  \"ISBN:9783522202602\": {",
    "    \"title\": \"Die unendliche Geschichte\",",
    "    \"authors\": [{\"name\": \"Michael Ende\", …}],",
    "    \"publish_date\": \"Jan 15, 2019\",",
    "    \"cover\": {\"small\": \"…\", \"medium\": \"…\", \"large\": \"…\"},",
    "    …",
    "  }",
    "}",
  ], { out: true }),

  H.p("Für eine ISBN, die Open Library nicht kennt, kommt nur {} zurück — kein Fehler, eine leere Antwort. Gelesen wird mit dem Modul json; ein Punkt im Pfad geht eine Ebene tiefer, eine Zahl wählt ein Element eines Felds:"),

  H.code(stueck(E, "SUB datenLesen", "END SUB")),

  H.p("Das Erscheinungsjahr steht nicht als Zahl da, sondern als Text, hier „Jan 15, 2019“. REGEX_FIND holt die erste Folge von vier Ziffern heraus. Gibt es ein Cover, ist der erste Abruf fertig und ein zweiter beginnt — der Timer läuft weiter, und holt merkt sich, dass jetzt ein Bild kommt."),

  H.h2("Schritt 5: das Cover"),

  H.code(stueck(E, "SUB coverLesen", "END SUB")),

  H.p("Ein Bild sind Bytes, kein Text; HTTP_BYTES() liefert den rohen Rumpf der letzten Antwort. Er kommt in eine Datei, die Datei ins Bild-Control, und die Datei wieder weg — das Bild ist jetzt im Fenster, und mit OK in der Datenbank. BildSetzen sagt FALSE, wenn die Bytes kein Bild waren."),

  H.p("Zum Schluss räumt fertig auf, was jeder Ausgang braucht:"),

  H.code(stueck(E, "SUB fertig", "END SUB")),

  H.p("Und wer das Fenster schließt, während ein Abruf läuft, verwirft ihn — sonst käme die Antwort in ein Fenster, das es nicht mehr gibt:"),

  H.code(stueck(E, "SUB Schliessen", "END SUB")),

  H.h2("Warum so?"),

  H.h3("Warum das Fenster nicht warten darf"),

  H.p("Das Modul html kennt auch HTTP_GET(url): ein Befehl, der die Antwort gleich als Text liefert. Er wäre kürzer — und er kehrt erst zurück, wenn die Antwort da ist. Bis dahin zeichnet das Programm kein Bild und nimmt keinen Klick an. Bei einer schnellen Antwort merkt man das kaum; bei einer langsamen steht das Fenster zehn Sekunden still, der Knopf „Abbrechen“ ist zwar da, aber niemand kann ihn drücken."),

  H.p("HTTP_GET_START gibt die Wartezeit an den Hintergrund ab. Das Fenster bleibt bedienbar: Man kann weiter tippen, abbrechen, das Fenster schließen. Der Preis ist, dass der Code in Stücke zerfällt — fragen, nachsehen, lesen, aufräumen — und dass er sich merken muss, wo er steht. Das tun die drei Felder abruf, holt und isbn."),

  H.h3("Warum ein Timer"),

  H.p("Nachsehen könnte man auch in Takt, dem Ereignis, das jedes Bild kommt. Aber ein Programm, das im Leerlauf schläft (App.Leerlauf), zeichnet ohne Eingabe nur selten ein Bild, und Takt käme dann vielleicht erst nach einer Sekunde. Ein Timer weckt das Programm, wenn er fällig ist. Und er läuft nur, solange es etwas zu fragen gibt: Starten beim Abruf, Stoppen in fertig."),

  H.h2("Was schiefgehen kann"),

  H.h3("Kein Netz"),

  H.p("Ohne Netz, oder wenn der Server nicht erreichbar ist, scheitert der Abruf gleich. Der Hinweis zeigt, was HTTP_RESULT geworfen hat: „Keine Antwort: HTTP-Fehler:“, die Adresse, was das Betriebssystem dazu sagt, und am Ende „keine Verbindung: kein Netz, oder der Server ist nicht erreichbar“. Das Programm läuft weiter; Titel und Urheber kann man immer noch tippen."),

  H.h3("Der Server antwortet nicht"),

  H.p("Ist die Verbindung da, aber die Antwort kommt nicht, wartet der Abruf so lange, wie HTTP_TIMEOUT erlaubt — hier zehn Sekunden — und scheitert dann mit „… keine Antwort in der Zeitgrenze (HTTP_TIMEOUT)“. Wer nicht so lange warten will, klickt „Abbrechen“. Abbrechen heißt dabei: Die Antwort wird verworfen, wenn sie kommt. Ein Abruf, der schon unterwegs ist, wird nicht zurückgeholt."),

  H.h3("Eine ISBN, die es nicht gibt"),

  H.p("Open Library antwortet dann mit {}, ganz ohne Fehler. Darum prüft datenLesen mit JSON_HAS, ob die gefragte ISBN in der Antwort steht, und sagt sonst „Unter dieser ISBN kennt Open Library nichts.“ Wer gleich JSON_GET_STRING fragte, bekäme einen Laufzeitfehler."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Lass in knopfNachschlagen_Klick die Zeile Self.abfrageTimer.Starten() weg und schlage eine ISBN nach. Was zeigt das Fenster, und was tut ein zweiter Klick?"),
  H.bulletRich("Drehen 2. ", "Nimm das REPLACE$ heraus (nur noch TRIM$) und schlage 978-3-522-20260-2 nach."),
  H.bulletRich("Drehen 3. ", "Setze QUELLE auf \"https://openlibrary.invalid\". Was sagt der Hinweis?"),
  H.bulletRich("Erweitern 1. ", "Ein Cache: Jede Antwort, die etwas kennt, kommt in eine Tabelle nachschlag (isbn, antwort). Wer dieselbe ISBN noch einmal nachschlägt, bekommt sie von dort, ohne Netz."),
  H.bulletRich("Erweitern 2. ", "Während der Abfrage zählt der Hinweis die Sekunden: „Frage Open Library ... 3 s“."),
  H.bulletRich("Erfinden. ", "Such dir für deine Sammlung aus Kapitel 13 einen Dienst im Netz, der zu ihr passt — Wetter, Postleitzahlen, Wechselkurse —, und lass ein Fenster dort nachschlagen, ohne stehen zu bleiben."),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Modul html: ", "alle HTTP-Befehle, Kopfzeilen, Zeitgrenze, Bytes statt Text, Abrufe im Hintergrund."),
  H.bulletRich("Handbuch, Modul json: ", "Pfade, JSON_HAS und JSON_KEY$ für einen Schlüssel mit Punkt."),
  H.bulletRich("Open Library, openlibrary.org/dev/docs/api/books: ", "was die Schnittstelle sonst noch liefert."),
];
