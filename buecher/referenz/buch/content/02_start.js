module.exports = (H) => [
  H.chapter("Installation, Editor & Programme starten"),
  H.p("Bevor wir Code schreiben, sorgen wir dafür, dass du ihn auch ausführen kannst. Das ist schnell erledigt."),

  H.h2("Was du brauchst"),
  H.p("Drachenhauch besteht aus zwei Teilen: der Laufzeit „dhrt“, die deine Programme ausführt, und der Entwicklungsumgebung (IDE), in der du sie schreibst – sie ist selbst ein Drachenhauch-Programm und läuft auf dhrt. Editoren für Grafik, Musik und mehr findest du in der IDE im Menü „Werkzeuge“. Nach der Installation liegt alles bereit."),

  H.h2("Der Code-Editor"),
  H.p("Die IDE öffnest du über den Eintrag „Drachenhauch IDE“ im Startmenü; wer mit dem Quelltext arbeitet, startet sie mit dhrt run ide/ide.dh. Der Editor färbt deinen Code ein, schlägt Befehle vor und zeigt Fehler an, noch bevor du startest. Mit der Taste F5 läuft dein Programm sofort los."),
  H.bulletRich("Neues Programm: ", "Datei → Neu, dann lostippen."),
  H.bulletRich("Starten: ", "Taste F5 – ein Fenster (oder die Konsole) geht auf und führt dein Programm aus."),
  H.bulletRich("Speichern: ", "Strg+S. Drachenhauch-Programme haben die Endung .dh."),

  H.h2("Ein Programm von Hand starten"),
  H.p("Du kannst ein gespeichertes Programm auch direkt starten, ohne den Editor. In der Eingabeaufforderung, im Projektordner:"),
  H.code(['dhrt run mein_programm.dh']),
  H.p("dhrt wechselt dabei selbst ins Verzeichnis deiner Datei – wichtig, sobald dein Programm Bilder oder Klänge aus Unterordnern lädt. Findet die Eingabeaufforderung dhrt nicht, hake bei der Installation „Installationsordner zum PATH hinzufügen“ an."),

  H.h2("Konsole oder Fenster?"),
  H.p("Drachenhauch-Programme gibt es in zwei Geschmacksrichtungen. Solange du nur mit PRINT Text ausgibst und mit INPUT etwas einliest, läuft alles in der Konsole – einem schlichten Textfenster. Sobald du den Befehl SCREEN benutzt, öffnet sich ein echtes Grafikfenster, in dem du zeichnen, Tasten abfragen und Töne abspielen kannst. Beide Welten lernst du in diesem Buch kennen, und wir fangen ganz bewusst mit der Konsole an: Sie lenkt nicht ab und zeigt das Wesentliche."),
  H.note("Die Beispiele in den ersten Kapiteln sind Konsolen-Programme – sie geben Text aus, den du sofort siehst. Ab Teil IV kommt die Grafik dazu."),
];
