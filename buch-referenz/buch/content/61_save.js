module.exports = (H) => [
  H.chapter("Modul: save"),
  H.p("Highscores, Fortschritt, Einstellungen – vieles soll auch nach dem Schließen des Spiels erhalten bleiben. Das save-Modul macht das bequem: Du legst einen Save an, setzt darin benannte Werte (Zahlen, Texte, Wahrheitswerte) und schreibst ihn in eine Datei. Beim nächsten Start lädst du die Datei zurück. Im Hintergrund ist alles JSON, aber darum musst du dich nicht kümmern – du arbeitest nur mit Schlüssel und Wert (ähnlich wie bei den Szenen-Daten, nur dauerhaft)."),

  H.h2("Anlegen, laden, schreiben"),
  H.cmd("SAVE_NEW · SAVE_LOAD_OR_NEW · SAVE_EXISTS", 'SAVE_NEW()   SAVE_LOAD_OR_NEW(pfad$)   SAVE_EXISTS(pfad$)',
    "SAVE_NEW erzeugt einen leeren Save. SAVE_LOAD_OR_NEW ist der übliche Weg beim Programmstart: Gibt es die Datei, wird sie geladen, sonst bekommst du einen leeren Save (kein Fehler). SAVE_EXISTS prüft, ob eine Datei vorhanden ist. (SAVE_LOAD lädt strikt und wirft, wenn die Datei fehlt.)",
    [
      'IMPORT "save"',
      'DIM s AS SAVE_HANDLE',
      's = SAVE_LOAD_OR_NEW("spiel.save")',
    ]),
  H.cmd("SAVE_WRITE · SAVE_DELETE_FILE", 'SAVE_WRITE(s, pfad$)   SAVE_DELETE_FILE(pfad$)',
    "SAVE_WRITE schreibt den Save in eine Datei (anlegen oder überschreiben). SAVE_DELETE_FILE löscht eine Save-Datei (schadet nicht, wenn sie nicht existiert).",
    [
      'SAVE_WRITE(s, "spiel.save")',
    ]),

  H.h2("Werte setzen & lesen"),
  H.cmd("SAVE_SET_… · SAVE_GET_…", 'SAVE_SET_INT(s, key$, wert)   SAVE_GET_INT(s, key$)   (auch FLOAT/STRING/BOOL)',
    "SET legt einen Wert unter einem Schlüssel ab, GET liest ihn. Je eine Variante für INT, FLOAT, STRING und BOOL. SAVE_GET_* ist strikt – fehlt der Schlüssel (oder passt der Typ nicht), gibt es einen Fehler.",
    [
      'SAVE_SET_STRING(s, "name", "Anna")',
      'SAVE_SET_INT(s, "highscore", 4200)',
      'PRINT SAVE_GET_STRING(s, "name"); " "; SAVE_GET_INT(s, "highscore")',
    ], { out: ["Anna 4200"] }),
  H.cmd("SAVE_GET_…_OR", 'SAVE_GET_INT_OR(s, key$, default)   (auch FLOAT/STRING/BOOL)',
    "Liefert immer einen Wert: Fehlt der Schlüssel (oder passt der Typ nicht), kommt default. Genau richtig für optionale Werte wie einen noch nicht existierenden Highscore.",
    [
      'DIM hi AS INTEGER',
      'hi = SAVE_GET_INT_OR(s, "highscore", 0)   \' 0 beim ersten Start',
    ]),
  H.note("Für Versionierung deiner Speicherstände gibt es SAVE_VERSION(s) und SAVE_SET_VERSION(s, n). Damit erkennst du beim Laden ein altes Format und kannst es behutsam umwandeln, statt am veränderten Aufbau zu scheitern."),

  H.h2("Das Highscore-Muster"),
  H.p("Ein klassisches Beispiel: beim Start den Highscore laden (oder 0), am Spielende vergleichen und nur bei einem Rekord neu schreiben."),
  H.code([
    'IMPORT "save"',
    'DIM s AS SAVE_HANDLE',
    's = SAVE_LOAD_OR_NEW("highscore.save")',
    'DIM rekord AS INTEGER',
    'rekord = SAVE_GET_INT_OR(s, "highscore", 0)',
    '',
    '\' ... Spiel läuft, am Ende steht score fest ...',
    'IF score > rekord THEN',
    '    SAVE_SET_INT(s, "highscore", score)',
    '    SAVE_WRITE(s, "highscore.save")     \' dauerhaft sichern',
    'END IF',
  ]),
  H.tip("save oder json?", "Für Spielstände mit ein paar benannten Werten ist save genau richtig – einfach und mit Versionsfeld. Brauchst du beliebig verschachtelte Datenstrukturen (Listen, Objekte in Objekten), ist das json-Modul (Teil V) flexibler. save ist im Grunde ein komfortabler Aufsatz auf JSON für den häufigsten Fall."),
];
