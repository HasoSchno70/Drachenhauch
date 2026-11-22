const { stueck, zeilen } = require("../code_lesen");

const M = "kap22/mediathek/";

// Ein paar Zeilen der Tafel, der Tabulator als Abstand.
function tafel(keys) {
  const z = zeilen(M + "sprache/en.txt").map((l) => l.split("\t"));
  return keys.map((k) => {
    const t = z.find((p) => p[0] === k);
    if (!t) throw new Error("sprache/en.txt: " + k);
    return (t[0] + "   ").padEnd(26) + t[1];
  });
}

module.exports = (H) => [
  H.chapter("Zwei Sprachen", 22),

  H.h2("Was wir bauen"),

  H.figure("kap22_englisch.png", "Die Mediathek auf Englisch: Menüs, Spaltenköpfe, Knöpfe und das Ausleihfenster sprechen Englisch. Die Art in der Tabelle („Buch“, „Spiel“) bleibt, wie sie in der Datenbank steht.", 480, 320),

  H.p("Eine Freundin in Dublin möchte ihre Sammlung auch so führen. In diesem Kapitel lernt die Mediathek Englisch — ohne dass ein Fenster doppelt entworfen oder eine Zeile Code für jede Sprache verzweigt wird. Auf einem englischen System spricht sie von selbst Englisch, auf einem deutschen Deutsch."),

  H.h2("Der Plan"),

  H.p("Ein Studio-Projekt schreibt seine Texte in einer Sprache, der seiner Quelle — hier Deutsch. Eine Tafel sprache/en.txt im Projektordner macht eine zweite daraus: je Zeile der deutsche Text, ein Tabulator, die Übersetzung. Drei Arten von Text gibt es dabei:"),

  H.bulletRich("Was im Entwurf steht, ", "übersetzt die Laufzeit von selbst: Fenstertitel, Beschriftungen, Platzhalter, Hinweise, Spaltenköpfe, Einträge einer Klappliste, Menüs samt Kürzeln, die Texte der Berichte."),
  H.bulletRich("Was der Code anzeigt, ", "geht durch tr$ — die Statuszeile, Meldungen, Ansagen."),
  H.bulletRich("Was in der Datenbank steht, ", "bleibt, wie es ist. Ein Titel ist ein Titel, auch auf Englisch."),

  H.h2("Schritt 1: Texte im Code"),

  H.p("tr$(\"Nichts gewählt\") liefert den Text in der gewählten Sprache; was die Tafel nicht kennt, bleibt, wie es ist. Ein Text mit Werten darin nimmt trf$ — erst wird übersetzt, dann eingesetzt:"),

  H.code(stueck(M + "Hauptfenster.dh", "SUB statusZeigen", "END SUB")),

  H.p("{0} ist der erste Wert der Liste, {1} der zweite. Das ist mehr als Bequemlichkeit: Im Ausleihfenster stehen drei Werte in einem Satz, und das Englische stellt sie anders:"),

  H.code(stueck(M + "AusleihFenster.dh", "SUB anzeigen", "END SUB")),

  H.p("Aus „{0} ist {1} bei {2}.“ wird in der Tafel „{0} has been with {2} {1}.“ — der Freund rückt vor die Zeit. Mit t + \" ist \" + seit + \" bei \" + bei wäre das nicht gegangen: Die Reihenfolge stünde im Code fest. Auch seit kommt schon übersetzt an, aus Bestand.dh:"),

  H.code(stueck(M + "Bestand.dh", "FUNCTION seitWann$", "END FUNCTION")),

  H.p("So geht es durch alle Fenster und Module: Jeder Text, den jemand liest, bekommt tr$ oder trf$. Was das Programm nur intern braucht — SQL, Spaltennamen, Dateinamen —, bleibt ohne."),

  H.h2("Schritt 2: die Tafel"),

  H.p("Welche Texte hineingehören, sammelt das Studio selbst: Projekt → „Sprachtafel ergänzen …“, Sprache en. Es geht durch jedes Fenster, jedes Menü, jeden Bericht und jedes tr$ und trf$ im Code und hängt an sprache/en.txt an, was ihr fehlt — bei der Mediathek 97 Texte, jeder mit einem Tabulator dahinter und noch ohne Übersetzung. Die schreibt man hinter den Tabulator, in einem Texteditor:"),

  H.code(tafel(["Suchen", "Neu ...", "{0} Einträge", "{0} ist {1} bei {2}.", "seit {0} Tagen", "Buch"]), { out: true }),

  H.p("Eine Zeile ohne Übersetzung lässt die Laufzeit aus; der Text bleibt deutsch, bis jemand sie füllt. Ruft man „Sprachtafel ergänzen“ nach einer Änderung noch einmal auf, kommen nur die neuen Texte dazu."),

  H.p("Einen Text findet die Sammlung nicht: tr$(zur) im Excel-Export, wo zur aus der Abfrage kommt und „noch da“ heißen kann. Was tr$ mit einer Variablen bekommt, steht nirgends als Literal — die Zeile „noch da“ kommt von Hand in die Tafel."),

  H.h2("Schritt 3: ausprobieren"),

  H.p("Welche Sprache gilt, entscheidet die Laufzeit vor App.Geoeffnet: Englisch, wenn das System nicht Deutsch spricht und die Tafel en.txt da ist, sonst die Sprache der Quelle. Wer auf einem deutschen System nachsehen will, schreibt für einen Lauf SpracheWaehlen(\"en\") an den Anfang von App.Geoeffnet, oder startet das Programm mit der Umgebungsvariablen DHRT_LANG=en."),

  H.p("Und dann sieht man sich jedes Fenster an. Englisch ist meist länger als Deutsch, manchmal kürzer — „Zurück“ wurde zu „Returned“ und passte nicht mehr in seinen Knopf. Das Ausleihfenster hat darum breitere Knöpfe bekommen."),

  H.h2("Schritt 4: Daten bleiben"),

  H.p("Die Art ist ein Sonderfall: Sie steht in der Datenbank, aber es gibt nur vier, und das Programm kennt sie. In der Klappliste des Bearbeitungsfensters stehen Book, Film, Game, Music — übersetzt wie jeder Eintrag einer Klappliste. Gespeichert wird trotzdem „Buch“: Das Studio gibt einer gebundenen Klappliste die Texte des Entwurfs als Werte mit. In der Datenbank steht also immer der deutsche Schlüssel, gleich in welcher Sprache jemand ihn gewählt hat."),

  H.p("Wo das Programm eine Art selbst zeigt, übersetzt es sie mit tr$ — in der Statistik:"),

  H.code(stueck(M + "StatistikFenster.dh", "SUB artZeigen", "END SUB")),

  H.p("Und wo es eine liest, nimmt es beide Schreibweisen an. Eine Liste aus Dublin darf „Book“ sagen; eingetragen wird „Buch“:"),

  H.code(stueck(M + "Austausch.dh", "FUNCTION artGenau$", "END FUNCTION")),

  H.h2("Warum so?"),

  H.h3("Warum der Schlüssel deutsch bleibt"),

  H.p("Viele Programme schreiben in den Code keine Texte, sondern Kennungen — status.count statt „{0} Einträge“ — und alle Sprachen in Tafeln. Das Studio nimmt den Text der Quelle selbst als Schlüssel. Wer den Code liest, liest, was auf dem Bildschirm steht; wer ein Fenster entwirft, schreibt hinein, was dort stehen soll, und muss sich keinen Namen dafür ausdenken. Fehlt eine Übersetzung, steht da der deutsche Text, keine Kennung, mit der niemand etwas anfangen kann."),

  H.p("Übersetzt wird erst beim Anzeigen. Alles, was das Programm sich merkt — in der Datenbank, in einem Vergleich, in einer Datei —, bleibt der Schlüssel. Sonst stünde in derselben Spalte einmal „Buch“ und einmal „Book“, je nachdem, wer gerade am Rechner saß, und die Statistik zählte zwei Arten, wo es eine ist."),

  H.p("Der Preis: Wer einen deutschen Text ändert, ändert den Schlüssel. Die alte Zeile der Tafel passt dann nicht mehr — Drehen 2 zeigt, was geschieht."),

  H.h2("Was schiefgehen kann"),

  H.h3("Zusammengesetzte Sätze"),

  H.p("t + \" entfernen?\" lässt sich nicht übersetzen: In der Tafel stünde nur „ entfernen?“, und im Englischen kommt das Verb nach vorn — „Remove Momo?“. Ein Satz mit Werten ist ein trf$ mit Platzhaltern, als ein Literal."),

  H.h3("Mit dem übersetzten Text vergleichen"),

  H.p("IF GUI_TEXT(Self.wahlArt) = \"Buch\" ist auf Englisch nie wahr — die Klappliste zeigt „Book“. Verglichen wird über die Nummer des Eintrags oder mit tr$(\"Buch\")."),

  H.h3("Ein Wort, zwei Bedeutungen"),

  H.p("„Zurück“ ist im Ausleihfenster ein Knopf und im Excel-Export eine Spalte; „Eintrag“ ist ein Fenstertitel und ein gezähltes Ding. Im Deutschen passt dasselbe Wort, im Englischen hier zufällig auch. In einer anderen Sprache vielleicht nicht — dann braucht der Code zwei verschiedene deutsche Texte, damit die Tafel zwei Übersetzungen haben kann."),

  H.h2("Übungen"),

  H.p("Lösungen und Antworten stehen in Anhang A."),

  H.bulletRich("Drehen 1. ", "Lösch in sprache/en.txt die Übersetzung hinter „{0} Einträge“, der Tabulator bleibt. Was steht auf Englisch in der Statuszeile?"),
  H.bulletRich("Drehen 2. ", "Ändere im Code „Erst einen Eintrag wählen.“ in „Bitte erst einen Eintrag wählen.“. Was zeigt das englische Programm? Was tut danach „Sprachtafel ergänzen“?"),
  H.bulletRich("Drehen 3. ", "Wähle im englischen Programm bei einem Eintrag die Art Book und sichere. Was steht in der Datenbank?"),
  H.bulletRich("Erweitern 1. ", "Die Spalte Type der Tabelle zeigt die Art auf Englisch."),
  H.bulletRich("Erweitern 2. ", "Eine dritte Sprache: Französisch. Auf einem französischen System spricht die Mediathek Französisch, auf einem deutschen Deutsch, sonst Englisch."),
  H.bulletRich("Erfinden. ", "Mach deine Sammlung aus Kapitel 13 zweisprachig. Welche Daten sind eigentlich Schlüssel, die man übersetzt zeigen sollte — wie die Art?"),

  H.h2("Zum Weiterlesen"),

  H.bulletRich("Handbuch, Drachenhauch Studio → Sprachen: ", "was von selbst übersetzt wird, SpracheWaehlen, Sprache$, tr$, trf$, die Lückenliste DH_STUDIO_SPRACHLUECKEN."),
  H.bulletRich("Handbuch, Standard-Built-ins → Betriebssystem: ", "SYSTEM_LANGUAGE$ und RUNTIME_LANGUAGE$."),
];
