module.exports = (H) => [
  H.chapter("Eigene Listen"),

  H.p("Fertige Listen sind bequem, aber sie enthalten nie das, was am Freitag drankommt. Ein Trainer, in den man nichts Eigenes eintragen kann, ist ein Spielzeug."),

  H.p("Das Programm dieses Kapitels kann beides: Listen aus dem Netz holen und Listen selbst anlegen. Es ist die Verwaltungshälfte des fertigen Trainers, und es läuft schon für sich allein."),

  H.figure("kap32_verwalten.png", "Oben die Liste wählen oder eine neue anlegen, darunter Vokabeln eintragen.", 440, 280),

  H.h2("Eine Liste anlegen — und ihre Nummer erfahren"),

  H.p("Beim Anlegen entsteht ein kleines Problem: Die neue Liste bekommt eine id, aber die vergibt die Datenbank. Woher weiß das Programm, welche es war?"),

  H.code([
    'DB_EXEC(con, "INSERT INTO listen (name, sprache) VALUES (?, ?)", _',
    "        name, sprache)",
    'erg = DB_QUERY(con, "SELECT last_insert_rowid()")',
    "DB_NEXT(erg)",
    "id = DB_GET_INT(erg, 0)",
    "DB_CLOSE_RESULT(erg)",
  ]),

  H.pmix([["last_insert_rowid()", true], " ist keine Tabelle und keine Spalte, sondern eine Frage an die Datenbank: „Welche Nummer hast du gerade vergeben?“ Sie gehört unmittelbar hinter das ", ["INSERT", true], " — dazwischen darf nichts anderes eingefügt werden, sonst bekommt man dessen Nummer."]),

  H.p("Davor steht noch eine Vorsichtsmaßnahme, und sie ist die halbe Funktion:"),

  H.code([
    'erg = DB_QUERY(con, "SELECT id FROM listen WHERE name = ?", name)',
    "IF DB_NEXT(erg) THEN",
    "    id = DB_GET_INT(erg, 0)",
    "    DB_CLOSE_RESULT(erg)",
    "    RETURN id",
    "END IF",
    "DB_CLOSE_RESULT(erg)",
  ]),

  H.p("Gibt es die Liste schon, wird ihre Nummer zurückgegeben statt einer neuen. Sonst hätte man nach dreimal „Holen“ dreimal denselben Englisch-Grundwortschatz im Auswahlfeld."),

  H.tip("Nachgemessen", "Dieselbe Datei zweimal geladen: Beim zweiten Mal lieferte liste_anlegen wieder die id 1, und in der Tabelle standen weiterhin drei Listen mit 56, 51 und 51 Vokabeln — nicht sechs Listen. Die Vokabeln selbst kämen allerdings doppelt hinein; wer das verhindern will, findet die passende Aufgabe am Kapitelende."),

  H.pmix([["DB_CLOSE_RESULT", true], " steht hier dreimal, und das ist kein Versehen: Auch der Zweig mit ", ["RETURN", true], " muss aufräumen, bevor er hinausspringt. Ein Ergebnis ist ein Stück Arbeitsspeicher, das die Datenbank für dich bereithält; wer es liegen lässt, gibt es nie zurück."]),

  H.tip("Nachgemessen", "Dieses Buch hatte an dieser Stelle zuerst eine dramatischere Begründung stehen: ein offenes Ergebnis halte die Tabelle fest, und irgendwann gehe kein Schreibzugriff mehr. Nachgeprüft stimmt das nicht — 5000 absichtlich offen gelassene Ergebnisse, kein Abbruch, und das INSERT danach lief anstandslos. Aufräumen bleibt richtig, aber aus dem gewöhnlichen Grund: Was man sich geben lässt, gibt man zurück."),

  H.h2("Die Kopfzeile lesen"),

  H.p("Jede Vokabelliste sagt selbst, wie sie heißt und welche Sprache sie ist. Das steht in ihren Anmerkungszeilen, und diese kleine Funktion holt es heraus:"),

  H.code([
    "FUNCTION kopfzeile(zeilen AS ARRAY OF STRING, feld AS STRING) AS STRING",
    "    DIM roh AS STRING",
    "    DIM marke AS STRING",
    '    marke = "# " + feld + ":"',
    "    FOR EACH roh IN zeilen",
    "        IF LEFT$(TRIM$(roh), LEN(marke)) = marke THEN",
    "            RETURN TRIM$(MID$(TRIM$(roh), LEN(marke)))",
    "        END IF",
    "    NEXT",
    '    RETURN ""',
    "END FUNCTION",
  ]),

  H.pmix(["Aufgerufen wird sie mit ", ['kopfzeile(zeilen, "sprache")', true], ", und sie sucht die Zeile, die mit ", ['"# sprache:"', true], " beginnt. Wird nichts gefunden, kommt ein leerer Text zurück — kein Fehler. Eine Liste ohne Kopfzeile ist keine kaputte Liste, sie ist nur karg."]),

  H.tip("Nachgemessen", "Mit den drei Listen dieses Buchs: „Englisch Grundwortschatz“ / „Englisch“, „Französisch Grundwortschatz“ / „Französisch“, „Spanisch Grundwortschatz“ / „Spanisch“ — Akzente inbegriffen, durch Datei, Netz und SQLite hindurch unverändert."),

  H.h2("Fünfhundert Vokabeln in einem Rutsch"),

  H.p("Beim Einlesen einer Liste stehen fünfzig bis mehrere hundert INSERT-Befehle hintereinander. Genau dabei zeigt sich eine Eigenschaft von Datenbanken, die man einmal gesehen haben muss:"),

  H.code([
    "DB_BEGIN(con)",
    "FOR EACH roh IN zeilen",
    "    ' ... hier steht das INSERT ...",
    "NEXT",
    "DB_COMMIT(con)",
  ]),

  H.pmix([["DB_BEGIN", true], " und ", ["DB_COMMIT", true], " klammern viele Änderungen zu EINER zusammen. Eine Transaktion — dasselbe Wort wie beim Geldüberweisen, und aus demselben Grund: Entweder alles oder nichts."]),

  H.tip("Nachgemessen", "Fünfhundert INSERT-Befehle, viermal gemessen. Ohne Transaktion: 0,91 bis 0,95 Sekunden. Mit Transaktion: 0,0022 Sekunden. Das ist Faktor vierhundert — und es ist kein Messfehler, sondern der Unterschied zwischen fünfhundertmal auf die Festplatte schreiben und einmal."),

  H.warn("Diese Zahl ist der Grund, warum jedes Programm, das viele Zeilen auf einmal schreibt, eine Transaktion braucht. Es ist auch der Grund, warum man sie NICHT um die ganze Programmlaufzeit legt: Solange sie offen ist, hat noch niemand etwas davon, und ein Absturz nimmt alles mit. Um einen Ladevorgang: ja. Um eine Spielschleife: nein.", "Wann eine Transaktion hingehört"),

  H.h2("Der freundliche erste Start"),

  H.p("Beim allerersten Aufruf ist die Datenbank leer, und ein leeres Fenster sagt niemandem, wofür es gut ist. Also legt das Programm dann selbst eine Liste an — aus der Textdatei, die neben ihm liegt:"),

  H.code([
    "' Beim allerersten Start eine Liste anlegen -- ein leeres Fenster",
    "' sagt niemandem, wofuer es gut ist.",
    'erg = DB_QUERY(con, "SELECT COUNT(*) FROM listen")',
    "DB_NEXT(erg)",
    "i = DB_GET_INT(erg, 0)",
    "DB_CLOSE_RESULT(erg)",
    'IF i = 0 AND FILEEXISTS("englisch_grund.txt") THEN',
    '    zeilen = READLINES("englisch_grund.txt")',
    '    text_einlesen(con, liste_anlegen(con, kopfzeile(zeilen, "name"), _',
    '                  kopfzeile(zeilen, "sprache")), zeilen)',
    "END IF",
  ]),

  H.pmix(["Derselbe Handgriff wie in Kapitel 27, nur eine Stufe größer: Damals waren es vier Vokabeln im Quelltext, hier ist es eine ganze Liste aus einer Datei. Die Bedingung ", ["i = 0", true], " sorgt dafür, dass es genau einmal passiert — beim zweiten Start ist die Tabelle nicht mehr leer."]),

  H.pmix(["Das ", ["FILEEXISTS", true], " daneben ist die zweite Hälfte: Fehlt die Datei, passiert eben nichts. Ein Programm, das beim Start abbricht, weil eine Beigabe fehlt, ist schlechter als eines, das ohne sie auskommt."]),

  H.h2("Eine Vokabel, zwei Vokabeln"),

  H.p("Ganz unten steht, wie viel drin ist — und dabei fällt eine Kleinigkeit auf, über die fast jedes Programm stolpert:"),

  H.code([
    "FUNCTION mehrzahl(n AS INTEGER, eins AS STRING, _",
    "                  viele AS STRING) AS STRING",
    '    IF n = 1 THEN RETURN STR$(n) + " " + eins',
    '    RETURN STR$(n) + " " + viele',
    "END FUNCTION",
  ]),

  H.p("„1 Listen“ steht in erstaunlich vielen Programmen, und es liest sich jedes Mal wie ein Fehler — was es ja auch ist. Fünf Zeilen räumen ihn aus, ein für alle Mal und für jedes Wort."),

  H.h2("Ein Auswahlfeld, das sich ändert"),

  H.p("Wenn eine Liste dazukommt, muss sie im Auswahlfeld auftauchen. Auch dafür gilt der Satz aus Kapitel 25: Das Array ist die Wahrheit, das Bedienelement zeigt es nur an."),

  H.code([
    "REDIM(listen_id, 0)",
    "REDIM(listen_name, 0)",
    'erg = DB_QUERY(con, "SELECT id, name FROM listen ORDER BY name")',
    "WHILE DB_NEXT(erg)",
    "    ARRAY_PUSH(listen_id, DB_GET_INT(erg, 0))",
    "    ARRAY_PUSH(listen_name, DB_GET_STRING(erg, 1))",
    "WEND",
    "DB_CLOSE_RESULT(erg)",
    "GUI_SET_DROPDOWN(w_liste, listen_name)",
  ]),

  H.pmix(["Zwei Arrays wieder, wie beim Löschen in Kapitel 27: ", ["listen_name", true], " ist das, was man sieht, ", ["listen_id", true], " das, womit die Datenbank arbeitet. Die Nummer im Auswahlfeld ist 0, 1, 2 — die ids können ganz andere sein."]),

  H.p("Und die Auswahl wird in jedem Bild abgefragt, aber nur bei einer Änderung passiert etwas:"),

  H.code([
    "i = GUI_DROPDOWN_SELECTED(w_liste)",
    "IF i >= 0 AND i < LEN(listen_id) AND listen_id[i] <> aktuell THEN",
    "    aktuell = listen_id[i]",
    "    neu_vokabeln = TRUE",
    "END IF",
  ]),

  H.pmix(["Drei Bedingungen mit ", ["AND", true], ", und jede fängt etwas ab: nichts ausgewählt, Auswahl zeigt auf ein Feld, das es nicht mehr gibt, oder es hat sich schlicht nichts geändert. Der Schalter ", ["neu_vokabeln", true], " ist wieder derselbe wie in Kapitel 27 — die Datenbank wird nur gelesen, wenn es etwas zu lesen gibt."]),

  H.h2("Zwei Reiter, zwei Aufgaben"),

  H.p("Die Oberfläche nutzt die Reiter aus Kapitel 26, und die Aufteilung ergibt sich von selbst: Was man oft tut, kommt nach vorn; was man selten tut, auf die zweite Seite."),

  H.code([
    'reiter[0] = "Vokabeln"',
    'reiter[1] = "Aus dem Netz"',
    "",
    'fenster = GUI_WINDOW("Verwalten", 20, 16, 600, 368)',
    "GUI_TABS(fenster, reiter)",
  ]),

  H.p("Und nach einem geglückten Abruf schaltet das Programm selbst zurück:"),

  H.code([
    "GUI_SET_ACTIVE_TAB(fenster, 0)",
  ]),

  H.note("Das ist eine dieser Kleinigkeiten, die ein Programm freundlich machen. Wer eine Liste holt, will sie danach sehen — nicht noch einmal auf einen Reiter klicken müssen. Solche Handgriffe kosten eine Zeile und sind der Unterschied zwischen „funktioniert“ und „lässt sich benutzen“."),

  H.h2("Das ganze Programm"),

  H.pmix(["Es steht als ", ["code/kap32/verwalten.dh", true], " neben dem Buch — 293 Zeilen, länger als alles bisher, aber nichts darin ist neu. Reiter aus Kapitel 26, Datenbank aus Kapitel 27, Netz aus Kapitel 29, Textzerlegung ebenfalls aus 29. Der einzige neue Befehl ist ", ["DB_BEGIN", true], "."]),

  H.p("Genau das ist der Punkt, an dem ein Buch wie dieses aufhört, Neues zu bringen: Ab hier baut man aus dem, was man hat."),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    ["Nach dem Holen ist die Liste dreimal da", "liste_anlegen prüft nicht, ob es den Namen schon gibt."],
    ["Die Vokabeln landen in der falschen Liste", "Die id aus last_insert_rowid wurde nicht direkt nach dem INSERT geholt."],
    ["Das Auswahlfeld bleibt leer", "GUI_SET_DROPDOWN fehlt nach dem Neuladen."],
    ["Beim Wechsel der Liste bleiben die alten Vokabeln stehen", "Der Schalter neu_vokabeln wird nicht gesetzt."],
    ["Das Einlesen dauert Sekunden", "DB_BEGIN und DB_COMMIT fehlen."],
    [{ text: "database is locked", mono: true }, "Eine zweite Verbindung schreibt, während die erste in einer Transaktion steckt — ein DB_BEGIN ohne DB_COMMIT."],
    ["Die Kopfzeile wird nicht erkannt", "In der Datei steht „#name:“ ohne Leerzeichen, gesucht wird aber „# name:“."],
    ["Neue Liste heißt „“", "Die Prüfung auf leere Eingabe fehlt vor dem Anlegen."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3600, 5426] }),

  H.h2("Aufgaben"),

  H.bullet("Verhindere, dass beim zweiten Holen derselben Liste die Vokabeln doppelt hineinkommen — ein DELETE vor dem Einlesen genügt."),
  H.bullet("Bau einen Knopf, der die gewählte Liste ganz löscht. Vergiss die Vokabeln darin nicht, und frag mit GUI_CONFIRM nach."),
  H.bullet("Ergänze ein Suchfeld über der Liste, das mit WHERE de LIKE ? filtert."),
  H.bullet("Bau eine Ausgabe: Die gewählte Liste wird als Textdatei im Buchformat geschrieben, mit Kopfzeilen."),
  H.bullet("Lass beim Eintippen die Eingabetaste dasselbe tun wie der Knopf „Dazu“."),
  H.bullet("Zeig neben jeder Liste, wie viele Vokabeln sie hat — ein GROUP BY über die Tabelle vokabeln liefert es."),
  H.bullet("Miss selbst nach, wie lange das Einlesen mit und ohne DB_BEGIN dauert. TIMER() gibt die Sekunden."),

  H.p("Damit sind alle Teile fertig. Im letzten Kapitel werden sie zusammengesetzt."),
];
