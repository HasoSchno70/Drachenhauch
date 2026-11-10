module.exports = (H) => [
  H.part("Teil VI — Der Vokabeltrainer"),
  H.chapter("Der Bauplan"),

  H.p("Sechs Teile lang hast du Werkzeug gesammelt. Jetzt wird daraus ein Programm, das man wirklich benutzt — und zwar eines, das mehr kann als ein Wort raten lassen."),

  H.p("Bevor die erste Zeile davon entsteht, wird es einen Moment lang unromantisch: Wir überlegen, was das Programm können soll und wo die Daten liegen. Das ist der langweiligste Teil eines Projekts und der einzige, den man nachher nicht mehr billig ändern kann."),

  H.h2("Was er können soll"),

  H.bulletRich("Vokabeln aus dem Netz holen ", "— fertige Listen, die nicht abgetippt werden müssen."),
  H.bulletRich("Mehrere Sprachen ", "— englisch, französisch, spanisch, und was sonst noch kommt."),
  H.bulletRich("Eigene Listen ", "— was in der Schule drankommt, steht in keiner fertigen Liste."),
  H.bulletRich("Nicht stumpf abfragen ", "— das Programm soll merken, was schwerfällt, und es öfter bringen."),
  H.bulletRich("Verschiedene Arten zu fragen ", "— erkennen ist leichter als abrufen, und beides braucht seinen Platz."),
  H.bulletRich("Fortschritt zeigen ", "— sonst weiß niemand, ob sich das Ganze lohnt."),

  H.p("Sechs Punkte, sechs Kapitel. Am Ende steht ein Programm, das alle sechs kann."),

  H.h2("Wo die Daten liegen"),

  H.p("Aus Kapitel 27 weißt du schon, dass so etwas in eine Datenbank gehört: einzelne Einträge ändern, sortieren, filtern — dafür ist sie da. Die Frage ist nur, in welche Tabellen."),

  H.p("Ein Anfänger legt gern EINE Tabelle an und schreibt alles hinein. Das geht schief, sobald es mehrere Listen gibt: Dann stünde neben jeder Vokabel noch einmal „Englisch Grundwortschatz“, sechsundfünfzigmal derselbe Text. Und wer die Liste umbenennt, muss sechsundfünfzig Zeilen ändern."),

  H.p("Deshalb drei Tabellen:"),

  H.table([
    [{ text: "listen", mono: true }, "welche Listen es gibt", "id, name, sprache"],
    [{ text: "vokabeln", mono: true }, "die Wörter selbst", "id, liste, de, fremd, fach, faellig, gesehen, richtig, falsch"],
    [{ text: "stand", mono: true }, "was sich das Programm merkt", "schluessel, wert"],
  ], { headers: ["Tabelle", "Wofür", "Spalten"], widths: [1600, 2600, 4826], mono: [0] }),

  H.pmix(["Die Spalte ", ["liste", true], " in ", ["vokabeln", true], " ist der ganze Trick: Sie hält die ", ["id", true], " aus der Tabelle ", ["listen", true], ". Der Name steht nur EINMAL da, und jede Vokabel zeigt darauf. In der Datenbanksprache heißt das ein Fremdschlüssel — man kann es sich als Pfeil vorstellen."]),

  H.note("Diese Aufteilung ist die halbe Miete jedes Datenbank-Programms, und sie hat einen Namen: Normalisierung. Die Faustregel dahinter ist kurz — jede Tatsache soll an genau einer Stelle stehen. Steht sie zweimal da, werden die beiden Stellen irgendwann verschieden, und dann weiß niemand mehr, welche stimmt."),

  H.h2("Die Datenbank zeichnet sich selbst"),

  H.p("Das erste Programm dieses Teils legt die drei Tabellen an — und fragt die Datenbank danach, was sie jetzt enthält. Nicht das Programm zeichnet den Bauplan, sondern die Datenbank beschreibt sich selbst:"),

  H.code([
    'erg = DB_QUERY(con, "SELECT name FROM sqlite_master " + _',
    "                    \"WHERE type = 'table' ORDER BY name\")",
    "WHILE DB_NEXT(erg)",
    "    ARRAY_PUSH(tabellen, DB_GET_STRING(erg, 0))",
    "WEND",
    "DB_CLOSE_RESULT(erg)",
  ]),

  H.pmix([["sqlite_master", true], " ist eine Tabelle, die SQLite selbst führt: In ihr steht, welche Tabellen es gibt. Man kann sie abfragen wie jede andere. Das ist ein ungewohnter Gedanke — die Datenbank kennt sich selbst und gibt Auskunft."]),

  H.code([
    'erg = DB_QUERY(con, "PRAGMA table_info(" + t + ")")',
    "WHILE DB_NEXT(erg)",
    "    TEXT(x + 12, y, DB_GET_STRING(erg, 1), RGB(170, 190, 220))",
    "    y = y + 22",
    "WEND",
    "DB_CLOSE_RESULT(erg)",
  ]),

  H.pmix([["PRAGMA table_info(name)", true], " liefert eine Zeile je Spalte. Spalte 1 davon ist der Spaltenname — deshalb ", ["DB_GET_STRING(erg, 1)", true], " und nicht ", ["0", true], "; auf Platz null steht die laufende Nummer."]),

  H.figure("kap28_1_bauplan.png", "Kein einziger Tabellenname steht im Zeichencode. Das Bild kommt aus der Datenbank.", 440, 280),

  H.warn("Der SQL-Text hier wird zusammengeklebt: \"PRAGMA table_info(\" + t + \")\". In Kapitel 27 stand, dass genau das gefährlich ist — und das gilt weiter. Der Unterschied: PRAGMA nimmt keine Fragezeichen, und t kommt nicht von einem Benutzer, sondern aus der Datenbank selbst. Wo ein Fragezeichen möglich ist, gehört ein Fragezeichen hin. Nur wo es das nicht ist, muss man sich vergewissern, woher der Text kommt.", "Die Ausnahme, die die Regel bestätigt"),

  H.h2("Was die Spalten bedeuten"),

  H.table([
    [{ text: "fach", mono: true }, "1 bis 5 — wie sicher die Vokabel sitzt"],
    [{ text: "faellig", mono: true }, "ab welcher Runde sie wieder drankommt"],
    [{ text: "gesehen", mono: true }, "wie oft sie schon vor Augen war"],
    [{ text: "richtig", mono: true }, "wie oft die Antwort stimmte"],
    [{ text: "falsch", mono: true }, "wie oft nicht"],
  ], { headers: ["Spalte", "Bedeutung"], widths: [2000, 7026], mono: [0] }),

  H.pmix(["Die letzten fünf Spalten sind das, was diesen Trainer von einer Abfrageliste unterscheidet. Ohne sie könnte das Programm nur würfeln. Mit ihnen kann es entscheiden — und in Kapitel 30 wirst du sehen, was für ein Unterschied das ist."]),

  H.pmix([["DEFAULT 1", true], " und ", ["DEFAULT 0", true], " im ", ["CREATE TABLE", true], " sparen Arbeit: Eine neue Vokabel landet automatisch in Fach 1 und ist sofort fällig. Man muss die Spalten beim Einfügen gar nicht erwähnen."]),

  H.h2("Der Weg durch diesen Teil"),

  H.table([
    ["29", "Vokabeln aus dem Netz", "HTTP, Textformat, Hintergrundabruf"],
    ["30", "Der Karteikasten", "welche Vokabel wann drankommt"],
    ["31", "Vier Arten zu fragen", "Multiple Choice, Tippen, Zuordnen"],
    ["32", "Eigene Listen", "anlegen, füllen, verwalten"],
    ["33", "Der Trainer", "alles zusammen, und weitergeben"],
  ], { headers: ["Kapitel", "Worum es geht", "Was dazukommt"], widths: [1000, 3200, 4826] }),

  H.p("Jedes dieser Kapitel bringt ein Programm, das für sich läuft. Am Ende werden sie zusammengesetzt — und du wirst merken, dass fast nichts Neues mehr dazukommt."),

  H.h2("Wenn etwas nicht geht"),

  H.table([
    [{ text: "no such table: sqlite_master", mono: true }, "Tippfehler — sie heißt genau so, mit Unterstrich."],
    ["Die Kästen bleiben leer", "PRAGMA table_info braucht den Tabellennamen ohne Anführungszeichen im SQL-Text."],
    ["Es erscheint nur eine Tabelle", "CREATE TABLE ist nur einmal aufgerufen worden, oder ein Name ist verschrieben."],
    [{ text: "Unbekannter Typ 'db_conn'", mono: true }, "IMPORT \"db\" fehlt."],
    ["Beim zweiten Start dieselben Spalten, obwohl geändert", "CREATE TABLE IF NOT EXISTS ändert nichts an einer Tabelle, die es schon gibt. Datei löschen oder ALTER TABLE."],
  ], { headers: ["Was du siehst", "Was meistens dahintersteckt"], widths: [3600, 5426] }),

  H.h2("Aufgaben"),

  H.bullet("Ergänze die Tabelle listen um eine Spalte für das Anlegedatum und lass sie mit anzeigen."),
  H.bullet("Zeig neben jedem Spaltennamen auch den Typ an — PRAGMA table_info liefert ihn in Spalte 2."),
  H.bullet("Lass das Programm zusätzlich zählen, wie viele Zeilen in jeder Tabelle stehen."),
  H.bullet("Überleg dir, welche Tabellen ein Rezeptbuch bräuchte. Schreib die Spalten auf, bevor du eine Zeile Code tippst."),
  H.bullet("Sieh dir an, was „SELECT sql FROM sqlite_master“ liefert — die Datenbank kennt sogar den Befehl, mit dem sie gebaut wurde."),

  H.p("Der Bauplan steht. Was fehlt, sind Vokabeln — und die holen wir uns im nächsten Kapitel aus dem Internet."),
];
