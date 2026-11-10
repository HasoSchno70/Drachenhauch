module.exports = (H) => [
  H.chapter("Daten, die bleiben"),

  H.p("In Kapitel 18 hast du eine Datei geschrieben und wieder gelesen. Das reicht erstaunlich weit — für eine Vokabelliste mit dreißig Einträgen ist es völlig in Ordnung."),

  H.p("Irgendwann reicht es nicht mehr. Nicht weil Dateien zu langsam wären, sondern weil man anfängt, Fragen zu stellen, die eine Textdatei nicht beantworten kann: Zeig mir alle Vokabeln, die ich dreimal falsch hatte. Sortiert nach dem letzten Mal. Und lösch die eine da, ohne die anderen anzufassen."),

  H.h2("Eine Datenbank in vier Zeilen"),

  H.code([
    'IMPORT "db"',
    "",
    "DIM con AS DB_CONN",
    'con = DB_OPEN("vokabeln.db")',
    'DB_EXEC(con, "CREATE TABLE IF NOT EXISTS vokabeln (" + _',
    '             "id INTEGER PRIMARY KEY, de TEXT, en TEXT)")',
  ]),

  H.pmix([["DB_OPEN", true], " öffnet eine Datenbankdatei — und legt sie an, wenn es sie noch nicht gibt. Danach ist sie da, mit allem, was beim letzten Mal drinstand."]),

  H.pmix([["CREATE TABLE IF NOT EXISTS", true], " legt die Tabelle an, aber nur beim ersten Mal. Diese Zeile darf bei jedem Start laufen; beim zweiten tut sie nichts. Das ist eine sehr angenehme Eigenschaft — das Programm muss nicht wissen, ob es schon einmal lief."]),

  H.p("Was in den Anführungszeichen steht, ist eine andere Sprache: SQL. Sie ist die übliche Art, mit Datenbanken zu reden, und für unsere Zwecke braucht man vier Sätze davon:"),

  H.table([
    [{ text: "CREATE TABLE ...", mono: true }, "eine Tabelle anlegen"],
    [{ text: "INSERT INTO ... VALUES ...", mono: true }, "eine Zeile hinzufügen"],
    [{ text: "SELECT ... FROM ... ORDER BY ...", mono: true }, "Zeilen holen, sortiert"],
    [{ text: "DELETE FROM ... WHERE ...", mono: true }, "Zeilen löschen"],
  ], { headers: ["Satz", "Was er tut"], widths: [3800, 5226], mono: [0] }),

  H.note("SQL ist eine eigene Sprache mit eigenen Büchern, und dieses hier bringt sie dir nicht bei. Die vier Sätze oben reichen aber für erstaunlich viele Programme — und sie sind fast englischer Text. „SELECT id, de, en FROM vokabeln ORDER BY de“ liest sich als: nimm diese drei Spalten aus dieser Tabelle, sortiert nach de."),

  H.h2("Hinzufügen"),

  H.code([
    'sql = "INSERT INTO vokabeln (de, en) VALUES (?, ?)"',
    "DB_EXEC(con, sql, de, en)",
  ]),

  H.warn("Die Fragezeichen sind wichtig, und zwar aus einem Grund, der weit über dieses Buch hinausreicht. Man KÖNNTE den Text zusammenkleben — \"... VALUES ('\" + de + \"')\". Dann aber zerlegt eine Vokabel mit einem Anführungszeichen darin den ganzen Befehl, und im schlimmsten Fall führt die Datenbank aus, was jemand ins Eingabefeld geschrieben hat. Das ist die berühmteste Sicherheitslücke überhaupt und heißt SQL-Injection. Mit Fragezeichen kann sie nicht passieren: Der Wert bleibt ein Wert und wird nie zum Befehl.", "Warum dort Fragezeichen stehen"),

  H.h2("Lesen"),

  H.code([
    'sql = "SELECT id, de, en FROM vokabeln ORDER BY de"',
    "erg = DB_QUERY(con, sql)",
    "WHILE DB_NEXT(erg)",
    "    ARRAY_PUSH(nummern, DB_GET_INT(erg, 0))",
    '    ARRAY_PUSH(zeilen, DB_GET_STRING(erg, 1) + "  ->  " + _',
    "                       DB_GET_STRING(erg, 2))",
    "WEND",
    "DB_CLOSE_RESULT(erg)",
  ]),

  H.pmix([["DB_QUERY", true], " stellt die Frage, ", ["DB_NEXT", true], " geht Zeile für Zeile durch die Antwort. Es liefert ", ["FALSE", true], ", wenn keine mehr kommt — daher die WHILE-Schleife. ", ["DB_GET_INT", true], " und ", ["DB_GET_STRING", true], " holen die Spalten, gezählt ab null."]),

  H.p("Und hier steckt der Gedanke aus Kapitel 25 wieder: Die Datenbank ist die Wahrheit, das Array ist nur der Zwischenspeicher fürs Anzeigen, und die Liste im Fenster zeigt das Array. Drei Schichten — und der Weg führt immer in dieselbe Richtung."),

  H.pmix(["Das zweite Array ", ["nummern", true], " ist der Kniff, ohne den das Löschen nicht ginge. Die Liste kennt nur ihre eigene Zeilennummer: 0, 1, 2. Die Datenbank kennt ihre ", ["id", true], ", und die kann ganz andere Zahlen haben, wenn zwischendurch etwas gelöscht wurde. Also merkt man sich beim Laden zu jeder Zeile die dazugehörige id."]),

  H.code([
    "nummer = GUI_LISTBOX_SELECTED(liste)",
    "IF nummer >= 0 THEN",
    '    sql = "DELETE FROM vokabeln WHERE id = ?"',
    "    DB_EXEC(con, sql, nummern[nummer])",
    "    neu_laden = TRUE",
    "END IF",
  ]),

  H.figure("kap27_1_vokabeln_db.png", "Dieselbe Oberfläche wie in Kapitel 25 — aber was hier steht, ist beim nächsten Start noch da.", 440, 280),

  H.h2("Nur laden, wenn nötig"),

  H.pmix(["Auffällig am Programm ist der Schalter ", ["neu_laden", true], ". Die Datenbank wird NICHT in jedem Bild abgefragt, sondern nur, wenn sich etwas geändert hat — nach dem Hinzufügen, nach dem Löschen, und einmal am Anfang."]),

  H.code([
    "IF neu_laden THEN",
    "    neu_laden = FALSE",
    "    REDIM(zeilen, 0)",
    "    REDIM(nummern, 0)",
    "    ' ... hier wird die Datenbank gelesen ...",
    "END IF",
  ]),

  H.p("Das ist derselbe Schalter wie beim Neustart der Schlange in Kapitel 9 und beim Bauen der Klänge im Instrument in Kapitel 12. Immer dieselbe Frage: Etwas ist teuer, und es muss nur passieren, wenn sich etwas geändert hat."),

  H.h2("Der freundliche erste Start"),

  H.code([
    'erg = DB_QUERY(con, "SELECT COUNT(*) FROM vokabeln")',
    "DB_NEXT(erg)",
    "IF DB_GET_INT(erg, 0) = 0 THEN",
    '    sql = "INSERT INTO vokabeln (de, en) VALUES (?, ?)"',
    '    DB_EXEC(con, sql, "Haus", "house")',
    '    DB_EXEC(con, sql, "Baum", "tree")',
    "END IF",
    "DB_CLOSE_RESULT(erg)",
  ]),

  H.p("Ist die Tabelle leer, legt das Programm ein paar Vokabeln an. Nur dann — beim zweiten Start ist sie ja nicht mehr leer. So steht beim ersten Öffnen etwas da, und man sieht sofort, wofür das Fenster gut ist."),

  H.tip("Nachgemessen", "Erster Lauf: vier Zeilen in der Datenbank. Zweiter Lauf: immer noch vier, nicht acht — die Vorbefüllung greift nur einmal. Und die Reihenfolge stimmt: Baum, Drache, Haus, Katze. Das ORDER BY macht die Datenbank, nicht das Programm."),

  H.h2("Datei oder Datenbank?"),

  H.table([
    ["Ein einzelner Wert (Bestwert, Einstellung)", "Datei", "eine Zeile, fertig"],
    ["Eine Liste, die man ganz liest und ganz schreibt", "Datei", "READLINES genügt"],
    ["Einzelne Einträge ändern oder löschen", "Datenbank", "sonst muss man die ganze Datei neu schreiben"],
    ["Sortieren, suchen, filtern", "Datenbank", "ORDER BY und WHERE erledigen es"],
    ["Mehr als ein paar hundert Einträge", "Datenbank", "sie liest nicht alles auf einmal ein"],
    ["Zusammenhängende Daten in mehreren Tabellen", "Datenbank", "dafür ist sie gemacht"],
  ], { headers: ["Was du hast", "Nimm", "Warum"], widths: [3400, 1400, 4226] }),

  H.p("Für den Vokabeltrainer fällt die Entscheidung leicht: Er soll sich merken, welche Vokabel wie oft richtig war, soll die schwierigen zuerst abfragen und einzelne löschen können. Das ist eine Datenbank."),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    [{ text: "Unbekannter Typ 'db_conn'", mono: true }, "IMPORT \"db\" fehlt."],
    [{ text: "no such table: vokabeln", mono: true }, "CREATE TABLE fehlt oder der Name ist verschrieben."],
    ["Die Liste bleibt leer", "Nach dem Ändern wird nicht neu geladen — der Schalter fehlt."],
    ["Es wird die falsche Zeile gelöscht", "Die Zeilennummer der Liste wurde als id benutzt. Sie sind nicht dasselbe."],
    ["Beim zweiten Start ist alles doppelt", "Die Vorbefüllung prüft nicht, ob schon etwas da ist."],
    ["Das Programm wird langsam", "Die Datenbank wird in jedem Bild abgefragt statt nur bei Änderungen."],
    ["Ein Eintrag mit Anführungszeichen zerlegt alles", "SQL wurde zusammengeklebt statt Fragezeichen zu benutzen."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3400, 5626] }),

  H.h2("Aufgaben"),

  H.bullet("Ergänze die Tabelle um eine Spalte für die Anzahl richtiger Antworten und zeig sie in der Liste mit an."),
  H.bullet("Bau ein Suchfeld ein: Was hineingeschrieben wird, filtert die Liste. Der SQL-Satz dafür braucht ein WHERE mit LIKE."),
  H.bullet("Füg einen Knopf hinzu, der die Reihenfolge zwischen deutsch und englisch umschaltet — das ist eine Änderung am ORDER BY."),
  H.bullet("Verhindere, dass dieselbe Vokabel zweimal angelegt wird."),
  H.bullet("Lass beim Löschen nachfragen, ob es wirklich sein soll. GUI_CONFIRM kann das."),
  H.bullet("Schreib ein zweites, winziges Programm, das dieselbe Datenbank öffnet und nur die Anzahl der Vokabeln ausgibt. Es beweist dir, dass die Daten wirklich außerhalb deines Programms liegen."),

  H.p("Damit endet Teil V — und damit auch die Vorbereitung. Du hast jetzt alles beisammen, was der Vokabeltrainer braucht: eine Oberfläche, eine Datenbank, Klang, Text, Arrays und eigene Befehle. Im letzten Teil setzen wir es zusammen."),
];
